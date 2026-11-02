//! 手動販売出庫（create_manual_sale）

use crate::biz::BizError;
use crate::constants::MAX_LINE_QUANTITY;
use crate::db::inventory_repo::{MovementType, ReferenceType};
use crate::db::manual_sale_repo;
use crate::db::manual_sale_repo::ManualSaleReason;
use crate::db::product_repo;
use crate::db::sales_repo;
use crate::db::system_repo;
use crate::db::{DbConnection, DbError};
use sha2::{Digest, Sha256};

use std::collections::BTreeMap;

use super::common::{
    apply_stock_change, compute_fingerprint, negative_stock_warning_message,
    validate_business_date, IDEMPOTENCY_KEY_MAX_LEN,
};

/// 手動販売出庫リクエスト（31-biz-inventory-service.md §12.5）
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct ManualSaleCreateRequest {
    pub idempotency_key: String,
    pub sale_date: String,
    pub reason: ManualSaleReason,
    pub note: Option<String>,
    pub items: Vec<ManualSaleItemInput>,
    pub confirmation_token: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct ManualSaleItemInput {
    pub product_code: String,
    pub quantity: i64,
    pub amount: i64,
}

/// 手動販売出庫の結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ManualSaleCreateResult {
    pub sale_id: Option<i64>,
    pub created: bool,
    pub idempotent_replay: bool,
    pub plu_warnings: Vec<String>,
    pub stock_warnings: Vec<String>,
    pub needs_confirmation: bool,
    pub confirmation_token: Option<String>,
}

/// confirmation_token を計算する（31-biz-inventory-service.md §12.5）
///
/// 各アイテム行に PLU 状態を含めてハッシュする。
/// PLU状態の変化も検知可能。
fn compute_confirmation_token(items_with_plu: &[String]) -> String {
    let mut sorted = items_with_plu.to_vec();
    sorted.sort();
    let mut hasher = Sha256::new();
    for (i, item) in sorted.iter().enumerate() {
        if i > 0 {
            hasher.update(b"\n");
        }
        hasher.update(item.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn compute_manual_sale_fingerprint(req: &ManualSaleCreateRequest) -> String {
    let header = format!("{}|{}", req.sale_date, req.reason.as_str());
    let item_lines: Vec<String> = req
        .items
        .iter()
        .map(|item| format!("{}|{}|{}", item.product_code, item.quantity, item.amount))
        .collect();
    compute_fingerprint(&header, &item_lines)
}

/// 手動販売出庫を記録する
///
/// PLU登録済み商品への警告チェック（confirmation_token方式）を含む。
/// 優先順位: 冪等性リプレイ > 確認フロー
///
/// 31-biz-inventory-service.md §12.5
pub fn create_manual_sale(
    conn: &mut DbConnection,
    req: ManualSaleCreateRequest,
) -> Result<ManualSaleCreateResult, BizError> {
    let normalized_key = req.idempotency_key.trim().to_string();
    if normalized_key.is_empty() {
        return Err(BizError::ValidationFailed(
            "冪等性キーは必須です".to_string(),
        ));
    }
    if normalized_key.len() > IDEMPOTENCY_KEY_MAX_LEN {
        return Err(BizError::ValidationFailed(format!(
            "冪等性キーは{}文字以内です",
            IDEMPOTENCY_KEY_MAX_LEN
        )));
    }

    // 明細空チェック + reason チェック（fingerprint 計算より先に実行）
    if req.items.is_empty() {
        return Err(BizError::ValidationFailed(
            "明細が1件以上必要です".to_string(),
        ));
    }

    // 業務日の形と暦の検査（DB を見ない検証の段。冪等の再送の判定と TX より前。31）
    validate_business_date(&req.sale_date)?;

    // fingerprint（リクエストフィールドのみ使用、DB参照不要）
    let fingerprint = compute_manual_sale_fingerprint(&req);

    // 冪等性チェック（バリデーション・確認フローより優先）
    if let Some((existing_id, existing_fp)) =
        manual_sale_repo::find_manual_sale_by_idempotency_key(conn, &normalized_key)?
    {
        if existing_fp == fingerprint {
            return Ok(ManualSaleCreateResult {
                sale_id: Some(existing_id),
                created: false,
                idempotent_replay: true,
                plu_warnings: vec![],
                stock_warnings: vec![],
                needs_confirmation: false,
                confirmation_token: None,
            });
        } else {
            return Err(BizError::IdempotencyConflict(
                "同じ冪等キーで異なる内容のリクエストです".to_string(),
            ));
        }
    }

    // 商品存在確認 + PLUチェック + バリデーション（DB参照あり）
    let mut plu_warnings = Vec::new();
    let mut token_lines = Vec::new();
    for (i, item) in req.items.iter().enumerate() {
        if item.quantity <= 0 {
            return Err(BizError::ValidationFailed(format!(
                "明細{}: 数量は1以上必要です",
                i + 1
            )));
        }
        if item.quantity > MAX_LINE_QUANTITY {
            return Err(BizError::ValidationFailed(format!(
                "明細{}: 数量が大きすぎます",
                i + 1
            )));
        }
        if item.amount < 0 {
            return Err(BizError::ValidationFailed(format!(
                "明細{}: 金額は0以上必要です",
                i + 1
            )));
        }
        let product =
            product_repo::find_by_product_code(conn, &item.product_code)?.ok_or_else(|| {
                BizError::NotFound(format!("商品が見つかりません: {}", item.product_code))
            })?;

        // PLU登録済みチェック
        let p = &product.product;
        let plu_exported_str = p.plu_exported_at.as_deref().unwrap_or("null").to_string();
        if !p.plu_dirty && p.plu_exported_at.is_some() {
            plu_warnings.push(format!(
                "{}: この商品はレジで打てます（PLU登録済み）",
                item.product_code
            ));
        }
        token_lines.push(format!(
            "{},{},{},{},{}",
            item.product_code, item.quantity, item.amount, p.plu_dirty, plu_exported_str
        ));
    }

    plu_warnings.sort();
    plu_warnings.dedup();

    // 確認フロー（冪等性チェック通過後）
    if !plu_warnings.is_empty() {
        match &req.confirmation_token {
            None => {
                // 確認待ち: DB未変更で返却
                let token = compute_confirmation_token(&token_lines);
                return Ok(ManualSaleCreateResult {
                    sale_id: None,
                    created: false,
                    idempotent_replay: false,
                    plu_warnings,
                    stock_warnings: vec![],
                    needs_confirmation: true,
                    confirmation_token: Some(token),
                });
            }
            Some(provided_token) => {
                let expected = compute_confirmation_token(&token_lines);
                if *provided_token != expected {
                    return Err(BizError::ValidationFailed(
                        "確認トークンが不正です".to_string(),
                    ));
                }
                // トークン一致 → 続行
            }
        }
    }
    // plu_warnings が空 && confirmation_token.is_some() の場合 → token は無視して通常処理

    // TX
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    let sale_id = match manual_sale_repo::insert_manual_sale(
        &tx,
        &crate::db::manual_sale_repo::NewManualSale {
            sale_date: req.sale_date.clone(),
            reason: req.reason.as_str().to_string(),
            note: req.note.clone(),
            idempotency_key: normalized_key.clone(),
            request_fingerprint: fingerprint.clone(),
        },
    ) {
        Ok(id) => id,
        Err(DbError::DuplicateKey(_)) => {
            let (existing_id, existing_fp) =
                manual_sale_repo::find_manual_sale_by_idempotency_key(&tx, &normalized_key)?
                    .ok_or_else(|| {
                        BizError::DatabaseError(DbError::QueryFailed(
                            "DuplicateKey後にレコードが見つかりません".to_string(),
                        ))
                    })?;
            if existing_fp == fingerprint {
                return Ok(ManualSaleCreateResult {
                    sale_id: Some(existing_id),
                    created: false,
                    idempotent_replay: true,
                    plu_warnings: vec![],
                    stock_warnings: vec![],
                    needs_confirmation: false,
                    confirmation_token: None,
                });
            } else {
                return Err(BizError::IdempotencyConflict(
                    "同じ冪等キーで異なる内容のリクエストです".to_string(),
                ));
            }
        }
        Err(e) => return Err(BizError::DatabaseError(e)),
    };

    let mut stock_warnings_map: BTreeMap<String, String> = BTreeMap::new();
    for item in &req.items {
        manual_sale_repo::insert_manual_sale_item(
            &tx,
            &crate::db::manual_sale_repo::NewManualSaleItem {
                manual_sale_id: sale_id,
                product_code: item.product_code.clone(),
                quantity: item.quantity,
                amount: item.amount,
            },
        )?;

        // INV-1: sale_records.quantity は売上帳票視点で正の値
        sales_repo::insert_sale_record(
            &tx,
            &crate::db::sales_repo::NewSaleRecord {
                csv_import_id: None,
                product_code: item.product_code.clone(),
                sale_date: req.sale_date.clone(),
                quantity: item.quantity,
                amount: item.amount,
                source: "manual".to_string(),
                source_line_no: None,
                reason: Some(req.reason.as_str().to_string()),
                note: req.note.clone(),
                stock_linked: false,
            },
        )?;

        // INV-1: inventory_movements.quantity は在庫視点で負の値
        let outcome = apply_stock_change(
            &tx,
            &item.product_code,
            &req.sale_date,
            -item.quantity,
            MovementType::SaleManual,
            ReferenceType::ManualSale,
            sale_id,
            None,
        )?;
        if outcome.negative_stock_warning {
            stock_warnings_map.insert(
                item.product_code.clone(),
                negative_stock_warning_message(&item.product_code, &outcome),
            );
        }
    }

    // BTreeMap → Vec（product_code 昇順・重複排除済み）
    let stock_warnings: Vec<String> = stock_warnings_map.into_values().collect();

    let detail = serde_json::json!({
        "record_type": "manual_sale",
        "record_id": sale_id,
        "item_count": req.items.len(),
        "warning_count": stock_warnings.len(),
        "idempotency_key": normalized_key,
    });
    system_repo::insert_operation_log(
        &tx,
        &crate::db::system_repo::NewOperationLog {
            operation_type: "manual_sale_create".to_string(),
            summary: format!("手動販売出庫作成（{}明細）", req.items.len()),
            detail_json: Some(detail.to_string()),
        },
    )?;

    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(ManualSaleCreateResult {
        sale_id: Some(sale_id),
        created: true,
        idempotent_replay: false,
        plu_warnings,
        stock_warnings,
        needs_confirmation: false,
        confirmation_token: None,
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::db::product_repo;

    fn make_manual_sale_req(key: &str, items: Vec<ManualSaleItemInput>) -> ManualSaleCreateRequest {
        ManualSaleCreateRequest {
            idempotency_key: key.to_string(),
            sale_date: "2026-04-07".to_string(),
            reason: ManualSaleReason::PluUnregistered,
            note: None,
            items,
            confirmation_token: None,
        }
    }

    fn manual_sale_item(code: &str, qty: i64, amount: i64) -> ManualSaleItemInput {
        ManualSaleItemInput {
            product_code: code.to_string(),
            quantity: qty,
            amount,
        }
    }

    #[test]
    fn test_manual_sale_fingerprint_fixed_value_req203() {
        let req = make_manual_sale_req("fp-manual-sale", vec![manual_sale_item("FP-MS", 2, 500)]);
        assert_eq!(
            compute_manual_sale_fingerprint(&req),
            "32685035d322b97a54cc38d489140f962a819abecddfc5c95aae22460f9bb3fd"
        );
    }

    #[test]
    fn test_create_manual_sale_req203_normal() {
        // REQ-203: 手動販売出庫 — 正常な手動販売出庫（PLU未登録商品 → 確認不要）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-001", 10);

        let req = make_manual_sale_req("ms-key-1", vec![manual_sale_item("MS-001", 2, 1000)]);
        let result = create_manual_sale(&mut conn, req).unwrap();
        assert!(result.created);
        assert!(!result.idempotent_replay);
        assert!(!result.needs_confirmation);
        assert!(result.plu_warnings.is_empty());
        assert!(result.sale_id.is_some());

        // 在庫減少
        let p = product_repo::find_by_product_code(&conn, "MS-001")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 8);
    }

    #[test]
    fn test_create_manual_sale_req203_operation_log_record_contract() {
        // REQ-203 / UI-11c-D7: 手動販売の operation_log が関連記録 contract を満たすこと
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-LOG", 10);

        let req = make_manual_sale_req("ms-log-key", vec![manual_sale_item("MS-LOG", 2, 1000)]);
        let result = create_manual_sale(&mut conn, req).unwrap();
        let sale_id = result.sale_id.unwrap();

        let (op_type, detail_str): (String, String) = conn
            .query_row(
                "SELECT operation_type, detail_json FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(op_type, "manual_sale_create");

        let detail: serde_json::Value = serde_json::from_str(&detail_str).unwrap();
        let detail_object = detail.as_object().unwrap();
        assert_eq!(detail["record_type"], "manual_sale");
        assert!(detail["record_id"].is_number());
        assert_eq!(detail["record_id"].as_i64(), Some(sale_id));
        assert!(!detail_object.contains_key("sale_id"));
        assert_eq!(detail["item_count"], 1);
        assert_eq!(detail["warning_count"], 0);
        assert_eq!(detail["idempotency_key"], "ms-log-key");
    }

    #[test]
    fn test_create_manual_sale_req203_plu_warning_needs_confirmation() {
        // REQ-203: 手動販売出庫 — PLU登録済み商品 → needs_confirmation=true, DB未変更
        let (_dir, mut conn) = setup_test_db();
        create_plu_exported_product(&conn, "MS-002", 10);

        let req = make_manual_sale_req("ms-key-2", vec![manual_sale_item("MS-002", 1, 500)]);

        // 各テーブルの件数を記録
        let ms_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM manual_sales", [], |r| r.get(0))
            .unwrap();
        let msi_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM manual_sale_items", [], |r| r.get(0))
            .unwrap();
        let sr_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM sale_records", [], |r| r.get(0))
            .unwrap();
        let mv_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |r| r.get(0))
            .unwrap();
        let ol_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();

        let result = create_manual_sale(&mut conn, req).unwrap();
        assert!(!result.created);
        assert!(!result.idempotent_replay);
        assert!(result.needs_confirmation);
        assert!(result.confirmation_token.is_some());
        assert!(result.sale_id.is_none());
        assert!(!result.plu_warnings.is_empty());

        // DB未変更を検証
        let ms_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM manual_sales", [], |r| r.get(0))
            .unwrap();
        let msi_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM manual_sale_items", [], |r| r.get(0))
            .unwrap();
        let sr_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM sale_records", [], |r| r.get(0))
            .unwrap();
        let mv_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |r| r.get(0))
            .unwrap();
        let ol_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();

        assert_eq!(ms_before, ms_after);
        assert_eq!(msi_before, msi_after);
        assert_eq!(sr_before, sr_after);
        assert_eq!(mv_before, mv_after);
        assert_eq!(ol_before, ol_after);

        // 在庫不変
        let p = product_repo::find_by_product_code(&conn, "MS-002")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 10);
    }

    #[test]
    fn test_create_manual_sale_req203_plu_confirm_with_token() {
        // REQ-203: 手動販売出庫 — PLU警告 → token取得 → tokenで再呼出 → 正常作成
        let (_dir, mut conn) = setup_test_db();
        create_plu_exported_product(&conn, "MS-003", 10);

        // 1回目: token取得
        let req1 = make_manual_sale_req("ms-key-3", vec![manual_sale_item("MS-003", 1, 500)]);
        let r1 = create_manual_sale(&mut conn, req1).unwrap();
        assert!(r1.needs_confirmation);
        let token = r1.confirmation_token.unwrap();

        // 2回目: token付きで再呼出
        let mut req2 = make_manual_sale_req("ms-key-3", vec![manual_sale_item("MS-003", 1, 500)]);
        req2.confirmation_token = Some(token);
        let r2 = create_manual_sale(&mut conn, req2).unwrap();
        assert!(r2.created);
        assert!(!r2.needs_confirmation);
        assert!(r2.sale_id.is_some());

        let p = product_repo::find_by_product_code(&conn, "MS-003")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 9);
    }

    #[test]
    fn test_create_manual_sale_req203_token_mismatch() {
        // REQ-203: 手動販売出庫 — 不正なトークン → ValidationFailed
        let (_dir, mut conn) = setup_test_db();
        create_plu_exported_product(&conn, "MS-004", 10);

        let mut req = make_manual_sale_req("ms-key-4", vec![manual_sale_item("MS-004", 1, 500)]);
        req.confirmation_token = Some("invalid-token".to_string());
        let result = create_manual_sale(&mut conn, req);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref msg)) if msg.contains("確認トークン"))
        );
    }

    #[test]
    fn test_create_manual_sale_req203_idempotent_replay_over_confirmation() {
        // REQ-203: 手動販売出庫 — 冪等リプレイが確認フローより優先
        let (_dir, mut conn) = setup_test_db();
        create_plu_exported_product(&conn, "MS-005", 10);

        // まず token 付きで正常作成
        let req1 = make_manual_sale_req("ms-key-5", vec![manual_sale_item("MS-005", 1, 500)]);
        let r1 = create_manual_sale(&mut conn, req1).unwrap();
        let token = r1.confirmation_token.unwrap();

        let mut req2 = make_manual_sale_req("ms-key-5", vec![manual_sale_item("MS-005", 1, 500)]);
        req2.confirmation_token = Some(token);
        let r2 = create_manual_sale(&mut conn, req2).unwrap();
        assert!(r2.created);

        // 3回目: 同じキーで呼出 → replay（confirmation_token なしでも）
        let req3 = make_manual_sale_req("ms-key-5", vec![manual_sale_item("MS-005", 1, 500)]);
        let r3 = create_manual_sale(&mut conn, req3).unwrap();
        assert!(!r3.created);
        assert!(r3.idempotent_replay);
        assert!(!r3.needs_confirmation);

        // 在庫は1回だけ減少（10-1=9）
        let p = product_repo::find_by_product_code(&conn, "MS-005")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 9);
    }

    #[test]
    fn test_create_manual_sale_req203_sale_record_inserted() {
        // REQ-203: 手動販売出庫 — sale_records に source="manual" で記録されること（INV-1: 売上帳票視点で正の値）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-006", 10);

        let req = make_manual_sale_req("ms-key-6", vec![manual_sale_item("MS-006", 2, 1000)]);
        create_manual_sale(&mut conn, req).unwrap();

        let (qty, amount, source): (i64, i64, String) = conn
            .query_row(
                "SELECT quantity, amount, source FROM sale_records WHERE product_code = 'MS-006'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(qty, 2); // 売上帳票視点で正
        assert_eq!(amount, 1000);
        assert_eq!(source, "manual");
    }

    #[test]
    fn test_create_manual_sale_req203_idempotent_replay_basic() {
        // REQ-203: 手動販売出庫 — 冪等性リプレイ（基本）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-007", 10);

        let req = make_manual_sale_req("ms-key-7", vec![manual_sale_item("MS-007", 1, 500)]);
        let r1 = create_manual_sale(&mut conn, req.clone()).unwrap();
        assert!(r1.created);

        let r2 = create_manual_sale(&mut conn, req).unwrap();
        assert!(!r2.created);
        assert!(r2.idempotent_replay);
        assert_eq!(r2.sale_id, r1.sale_id);
    }

    #[test]
    fn test_create_manual_sale_req203_idempotent_replay_does_not_duplicate_operation_log() {
        // REQ-203: 冪等性リプレイで operation_log が重複しないこと
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-LOG-REPLAY", 10);

        let req = make_manual_sale_req(
            "ms-log-replay-key",
            vec![manual_sale_item("MS-LOG-REPLAY", 1, 500)],
        );
        let first = create_manual_sale(&mut conn, req.clone()).unwrap();

        let log_count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();

        let replay = create_manual_sale(&mut conn, req).unwrap();
        assert!(!replay.created);
        assert!(replay.idempotent_replay);
        assert_eq!(replay.sale_id, first.sale_id);

        let log_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(log_count_before, log_count_after);
    }

    #[test]
    fn test_create_manual_sale_req203_idempotency_conflict() {
        // REQ-203: 手動販売出庫 — 同じキー+異なる内容 → IdempotencyConflict
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-008", 10);

        let req1 = make_manual_sale_req("ms-key-8", vec![manual_sale_item("MS-008", 1, 500)]);
        create_manual_sale(&mut conn, req1).unwrap();

        let req2 = make_manual_sale_req("ms-key-8", vec![manual_sale_item("MS-008", 3, 1500)]);
        let result = create_manual_sale(&mut conn, req2);
        assert!(matches!(result, Err(BizError::IdempotencyConflict(_))));
    }

    #[test]
    fn test_create_manual_sale_req203_validation_empty_items() {
        // REQ-203: 手動販売出庫 — バリデーション: 空の明細 → エラー
        let (_dir, mut conn) = setup_test_db();
        let req = make_manual_sale_req("ms-key-9", vec![]);
        let result = create_manual_sale(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_manual_sale_req203_validation_quantity_zero() {
        // REQ-203: 手動販売出庫 — バリデーション: 数量 <= 0 → エラー
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-011", 10);

        let req = make_manual_sale_req("ms-key-11", vec![manual_sale_item("MS-011", 0, 500)]);
        let result = create_manual_sale(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_manual_sale_req203_validation_negative_amount() {
        // REQ-203: 手動販売出庫 — バリデーション: 金額 < 0 → エラー
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "MS-012", 10);

        let req = make_manual_sale_req("ms-key-12", vec![manual_sale_item("MS-012", 1, -1)]);
        let result = create_manual_sale(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    // -----------------------------------------------------------------------
    // 入力の上限（共通規則 SPEC-UNIT-D11 の続き、X1・X3）
    // -----------------------------------------------------------------------

    const MANUAL_SALE_TABLES: [&str; 5] = [
        "manual_sales",
        "manual_sale_items",
        "sale_records",
        "inventory_movements",
        "operation_logs",
    ];

    #[test]
    fn test_create_manual_sale_req203_quantity_limit() {
        // REQ-203 / X1: 1 明細の数量は 9999999 まで。1 つ外は何も書かない
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "LIM-MS", 0);
        create_manual_sale(
            &mut conn,
            make_manual_sale_req(
                "lim-ms-ok",
                vec![manual_sale_item("LIM-MS", 9_999_999, 500)],
            ),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "LIM-MS"), -9_999_999);

        let before = write_surface(&conn, &MANUAL_SALE_TABLES);
        let result = create_manual_sale(
            &mut conn,
            make_manual_sale_req(
                "lim-ms-over",
                vec![manual_sale_item("LIM-MS", 10_000_000, 500)],
            ),
        );
        assert_rejected_without_writes(
            &conn,
            &MANUAL_SALE_TABLES,
            &before,
            result,
            "明細1: 数量が大きすぎます",
        );
    }

    #[test]
    fn test_create_manual_sale_req203_stock_limit_rolls_back_whole_tx() {
        // REQ-203 / X3: 計算の後の在庫が -999999999 を下回ったら TX 全体を戻す
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "STK-FULL", -999_999_999);
        create_test_product(&conn, "STK-NEAR", -999_999_998);
        let before = write_surface(&conn, &MANUAL_SALE_TABLES);
        let result = create_manual_sale(
            &mut conn,
            make_manual_sale_req("stk-over", vec![manual_sale_item("STK-FULL", 1, 500)]),
        );
        assert_rejected_without_writes(
            &conn,
            &MANUAL_SALE_TABLES,
            &before,
            result,
            "在庫数が大きすぎます",
        );

        create_manual_sale(
            &mut conn,
            make_manual_sale_req("stk-edge", vec![manual_sale_item("STK-NEAR", 1, 500)]),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "STK-NEAR"), -999_999_999);
    }

    #[test]
    fn test_create_manual_sale_req203_stock_warning_formats_length_in_meters() {
        // REQ-203 / SPEC-UNIT-D3 の表の 10: 在庫警告の文は、長さの商品（m）の在庫を m の形で入れる
        let (_dir, mut conn) = setup_test_db();
        create_test_product_with_unit(&conn, "W-M", 100, "m");
        create_test_product_with_unit(&conn, "W-CM", 100, "m");
        create_test_product_with_unit(&conn, "W-PCS", 1, "pcs");
        let result = create_manual_sale(
            &mut conn,
            make_manual_sale_req(
                "warn-len",
                vec![
                    manual_sale_item("W-M", 130, 910),
                    manual_sale_item("W-CM", 130, 910),
                    manual_sale_item("W-PCS", 3, 1500),
                ],
            ),
        )
        .unwrap();
        assert_eq!(
            result.stock_warnings,
            vec![
                "W-CM: 在庫がマイナスになりました（-0.3 m）".to_string(),
                "W-M: 在庫がマイナスになりました（-0.3 m）".to_string(),
                "W-PCS: 在庫がマイナスになりました（-2）".to_string(),
            ]
        );
    }

    #[test]
    fn test_create_manual_sale_req203_rejects_noncanonical_date() {
        // REQ-203 / Y1（4 業務の全部）/ Matrix W3
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "W3-MS", 3);
        let before = write_surface(&conn, &MANUAL_SALE_TABLES);
        let mut req = make_manual_sale_req("w3-ms", vec![manual_sale_item("W3-MS", 1, 500)]);
        req.sale_date = "2026-4-7".to_string();
        let result = create_manual_sale(&mut conn, req);
        assert_rejected_without_writes(
            &conn,
            &MANUAL_SALE_TABLES,
            &before,
            result,
            "日付の形式が不正です（YYYY-MM-DD）",
        );
    }
}
