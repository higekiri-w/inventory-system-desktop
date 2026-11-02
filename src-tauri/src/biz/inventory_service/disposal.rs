//! 廃棄・破損記録（create_disposal）

use crate::biz::unit_amount::{
    check_safe_integer, cost_line_centi, cost_total_yen, price_basis_quantity,
};
use crate::biz::BizError;
use crate::constants::{MAX_COST_PRICE_CENTI, MAX_LINE_QUANTITY};
use crate::db::disposal_repo;
use crate::db::disposal_repo::DisposalType;
use crate::db::inventory_repo::{MovementType, ReferenceType};
use crate::db::product_repo;
use crate::db::system_repo;
use crate::db::{DbConnection, DbError};

use std::collections::BTreeMap;

use super::common::{
    apply_stock_change, compute_fingerprint, negative_stock_warning_message,
    validate_business_date, IDEMPOTENCY_KEY_MAX_LEN,
};

/// 廃棄・破損記録リクエスト（31-biz-inventory-service.md §12.6）
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct DisposalCreateRequest {
    pub idempotency_key: String,
    pub disposal_date: String,
    pub items: Vec<DisposalItemInput>,
}

#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct DisposalItemInput {
    pub product_code: String,
    pub disposal_type: DisposalType,
    pub quantity: i64,
    pub cost_price_centi: i64,
    pub reason: String,
}

/// 廃棄・破損記録の結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DisposalCreateResult {
    pub record_id: i64,
    pub created: bool,
    pub idempotent_replay: bool,
    pub stock_warnings: Vec<String>,
}

fn compute_disposal_fingerprint(req: &DisposalCreateRequest) -> String {
    let item_lines: Vec<String> = req
        .items
        .iter()
        .map(|item| {
            format!(
                "{}|{}|{}|{}|{}",
                item.product_code,
                item.quantity,
                item.cost_price_centi,
                item.disposal_type.as_str(),
                item.reason
            )
        })
        .collect();
    compute_fingerprint(&req.disposal_date, &item_lines)
}

/// 廃棄・破損記録を登録し、在庫を減少させる
///
/// 31-biz-inventory-service.md §12.6
pub fn create_disposal(
    conn: &mut DbConnection,
    req: DisposalCreateRequest,
) -> Result<DisposalCreateResult, BizError> {
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

    // 明細空チェック（fingerprint 計算より先に実行）
    if req.items.is_empty() {
        return Err(BizError::ValidationFailed(
            "明細が1件以上必要です".to_string(),
        ));
    }

    // 業務日の形と暦の検査（DB を見ない検証の段。冪等の再送の判定と TX より前。31）
    validate_business_date(&req.disposal_date)?;

    // fingerprint（リクエストフィールドのみ使用、DB参照不要）
    let fingerprint = compute_disposal_fingerprint(&req);

    // 冪等性チェック（バリデーションより先に実行）
    if let Some((existing_id, existing_fp)) =
        disposal_repo::find_disposal_by_idempotency_key(conn, &normalized_key)?
    {
        if existing_fp == fingerprint {
            return Ok(DisposalCreateResult {
                record_id: existing_id,
                created: false,
                idempotent_replay: true,
                stock_warnings: vec![],
            });
        } else {
            return Err(BizError::IdempotencyConflict(
                "同じ冪等キーで異なる内容のリクエストです".to_string(),
            ));
        }
    }

    // バリデーション（DB参照あり）
    let mut line_centis = Vec::with_capacity(req.items.len());
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
        if item.cost_price_centi < 0 {
            return Err(BizError::ValidationFailed(format!(
                "明細{}: 原価は0以上必要です",
                i + 1
            )));
        }
        if item.cost_price_centi > MAX_COST_PRICE_CENTI {
            return Err(BizError::ValidationFailed(format!(
                "明細{}: 原価が大きすぎます",
                i + 1
            )));
        }
        if item.reason.trim().is_empty() {
            return Err(BizError::ValidationFailed(format!(
                "明細{}: 理由は必須です",
                i + 1
            )));
        }
        let product =
            product_repo::find_by_product_code(conn, &item.product_code)?.ok_or_else(|| {
                BizError::NotFound(format!("商品が見つかりません: {}", item.product_code))
            })?;
        line_centis.push(cost_line_centi(
            i128::from(item.cost_price_centi),
            item.quantity,
            price_basis_quantity(product.product.stock_unit),
            &item.product_code,
        )?);
    }
    // 円の合計は最初の書込みの前に安全な整数の範囲で確かめる（共通規則 (c)、X8）
    check_safe_integer(i128::from(cost_total_yen(&line_centis)?), "合計金額")?;

    // TX
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    let record_id = match disposal_repo::insert_disposal_record(
        &tx,
        &crate::db::disposal_repo::NewDisposalRecord {
            disposal_date: req.disposal_date.clone(),
            idempotency_key: normalized_key.clone(),
            request_fingerprint: fingerprint.clone(),
        },
    ) {
        Ok(id) => id,
        Err(DbError::DuplicateKey(_)) => {
            let (existing_id, existing_fp) =
                disposal_repo::find_disposal_by_idempotency_key(&tx, &normalized_key)?.ok_or_else(
                    || {
                        BizError::DatabaseError(DbError::QueryFailed(
                            "DuplicateKey後にレコードが見つかりません".to_string(),
                        ))
                    },
                )?;
            if existing_fp == fingerprint {
                return Ok(DisposalCreateResult {
                    record_id: existing_id,
                    created: false,
                    idempotent_replay: true,
                    stock_warnings: vec![],
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
        disposal_repo::insert_disposal_item(
            &tx,
            &crate::db::disposal_repo::NewDisposalItem {
                disposal_record_id: record_id,
                product_code: item.product_code.clone(),
                disposal_type: item.disposal_type.as_str().to_string(),
                quantity: item.quantity,
                cost_price_centi: item.cost_price_centi,
                reason: item.reason.clone(),
            },
        )?;

        let outcome = apply_stock_change(
            &tx,
            &item.product_code,
            &req.disposal_date,
            -item.quantity,
            MovementType::Disposal,
            ReferenceType::DisposalRecord,
            record_id,
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
        "record_type": "disposal_record",
        "record_id": record_id,
        "item_count": req.items.len(),
        "warning_count": stock_warnings.len(),
        "idempotency_key": normalized_key,
    });
    system_repo::insert_operation_log(
        &tx,
        &crate::db::system_repo::NewOperationLog {
            operation_type: "disposal_create".to_string(),
            summary: format!("廃棄記録作成（{}明細）", req.items.len()),
            detail_json: Some(detail.to_string()),
        },
    )?;

    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(DisposalCreateResult {
        record_id,
        created: true,
        idempotent_replay: false,
        stock_warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::db::product_repo;

    fn make_disposal_req(key: &str, items: Vec<DisposalItemInput>) -> DisposalCreateRequest {
        DisposalCreateRequest {
            idempotency_key: key.to_string(),
            disposal_date: "2026-04-07".to_string(),
            items,
        }
    }

    fn disposal_item(code: &str, qty: i64, cost: i64, reason: &str) -> DisposalItemInput {
        DisposalItemInput {
            product_code: code.to_string(),
            disposal_type: DisposalType::Disposal,
            quantity: qty,
            cost_price_centi: cost,
            reason: reason.to_string(),
        }
    }

    #[test]
    fn test_disposal_fingerprint_fixed_value_req204() {
        let req = DisposalCreateRequest {
            idempotency_key: "fp-disposal".to_string(),
            disposal_date: "2026-04-07".to_string(),
            items: vec![DisposalItemInput {
                product_code: "FP-DSP".to_string(),
                disposal_type: DisposalType::Damage,
                quantity: 2,
                cost_price_centi: 100,
                reason: "破損".to_string(),
            }],
        };
        assert_eq!(
            compute_disposal_fingerprint(&req),
            "d1c78978cf52125a62b1302360649f40d771265ce4b8ed82926834b92de6a1d1"
        );
    }

    #[test]
    fn test_create_disposal_req204_normal() {
        // REQ-204: 廃棄・破損記録 — 正常な廃棄記録作成
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-001", 10);

        let req = make_disposal_req(
            "dsp-key-1",
            vec![disposal_item("DSP-001", 2, 300, "袋破れ")],
        );
        let result = create_disposal(&mut conn, req).unwrap();
        assert!(result.created);
        assert!(!result.idempotent_replay);

        // 在庫減少
        let p = product_repo::find_by_product_code(&conn, "DSP-001")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 8);
    }

    #[test]
    fn test_create_disposal_req204_operation_log_record_contract() {
        // REQ-204 / UI-11c-D7: 廃棄記録の operation_log が関連記録 contract を満たすこと
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-LOG", 10);

        let req = make_disposal_req(
            "dsp-log-key",
            vec![disposal_item("DSP-LOG", 2, 300, "袋破れ")],
        );
        let result = create_disposal(&mut conn, req).unwrap();

        let (op_type, detail_str): (String, String) = conn
            .query_row(
                "SELECT operation_type, detail_json FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(op_type, "disposal_create");

        let detail: serde_json::Value = serde_json::from_str(&detail_str).unwrap();
        assert_eq!(detail["record_type"], "disposal_record");
        assert!(detail["record_id"].is_number());
        assert_eq!(detail["record_id"].as_i64(), Some(result.record_id));
        assert_eq!(detail["item_count"], 1);
        assert_eq!(detail["warning_count"], 0);
        assert_eq!(detail["idempotency_key"], "dsp-log-key");
    }

    #[test]
    fn test_create_disposal_req204_stock_decrease_confirmed() {
        // REQ-204: 廃棄・破損記録 — 在庫減少の詳細確認（movement レコード含む）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-002", 15);

        let req = make_disposal_req(
            "dsp-key-2",
            vec![disposal_item("DSP-002", 5, 200, "色焼け")],
        );
        create_disposal(&mut conn, req).unwrap();

        let (qty, mt): (i64, String) = conn
            .query_row(
                "SELECT quantity, movement_type FROM inventory_movements WHERE product_code = 'DSP-002' AND reference_type = 'disposal_record'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(qty, -5); // 在庫視点で負
        assert_eq!(mt, "disposal");
    }

    #[test]
    fn test_create_disposal_req204_idempotent_replay() {
        // REQ-204: 廃棄・破損記録 — 冪等性リプレイ（副作用ゼロ）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-003", 10);

        let req = make_disposal_req("dsp-key-3", vec![disposal_item("DSP-003", 2, 300, "破損")]);
        let r1 = create_disposal(&mut conn, req.clone()).unwrap();

        let log_count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();

        let r2 = create_disposal(&mut conn, req).unwrap();
        assert!(!r2.created);
        assert!(r2.idempotent_replay);
        assert_eq!(r2.record_id, r1.record_id);

        // 在庫は1回だけ減少
        let p = product_repo::find_by_product_code(&conn, "DSP-003")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 8);

        let log_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(log_count_before, log_count_after);
    }

    #[test]
    fn test_create_disposal_req204_idempotency_conflict() {
        // REQ-204: 廃棄・破損記録 — 同じキー+異なる内容 → IdempotencyConflict
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-004", 10);

        let req1 = make_disposal_req(
            "dsp-key-4",
            vec![disposal_item("DSP-004", 2, 300, "袋破れ")],
        );
        create_disposal(&mut conn, req1).unwrap();

        let req2 = make_disposal_req(
            "dsp-key-4",
            vec![disposal_item("DSP-004", 5, 300, "袋破れ")],
        );
        let result = create_disposal(&mut conn, req2);
        assert!(matches!(result, Err(BizError::IdempotencyConflict(_))));
    }

    #[test]
    fn test_create_disposal_req204_stock_warnings_dedup_by_product_code() {
        // REQ-204: 廃棄・破損記録 — P2-2回帰: 同一product_codeが複数明細で警告→1件に集約、後勝ち
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "WARN-DUP", 0);

        let req = make_disposal_req(
            "warn-dup-test",
            vec![
                disposal_item("WARN-DUP", 1, 100, "テスト1"),
                disposal_item("WARN-DUP", 2, 100, "テスト2"),
            ],
        );

        let result = create_disposal(&mut conn, req).unwrap();
        // 同一product_codeなので警告は1件に集約
        assert_eq!(result.stock_warnings.len(), 1);
        // 後勝ち: 2回目の変動後の stock_after が使われる (0 - 1 - 2 = -3)
        assert!(result.stock_warnings[0].contains("-3"));
    }

    #[test]
    fn test_create_disposal_req204_validation_empty_key() {
        // REQ-204: 廃棄・破損記録 — バリデーション: 空白のみの冪等性キー → エラー
        let (_dir, mut conn) = setup_test_db();
        let req = make_disposal_req("  ", vec![disposal_item("X", 1, 100, "理由")]);
        let result = create_disposal(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_disposal_req204_validation_empty_reason() {
        // REQ-204: 廃棄・破損記録 — reason 空文字 → エラー
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-006", 10);

        let req = make_disposal_req("dsp-key-6", vec![disposal_item("DSP-006", 1, 100, "")]);
        let result = create_disposal(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_disposal_req204_validation_quantity_zero() {
        // REQ-204: 廃棄・破損記録 — バリデーション: 数量 <= 0 → エラー
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-008", 10);

        let req = make_disposal_req("dsp-key-8", vec![disposal_item("DSP-008", 0, 100, "理由")]);
        let result = create_disposal(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_disposal_req204_validation_negative_cost() {
        // REQ-204: 廃棄・破損記録 — バリデーション: 原価 < 0 → エラー
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "DSP-009", 10);

        let req = make_disposal_req("dsp-key-9", vec![disposal_item("DSP-009", 1, -1, "理由")]);
        let result = create_disposal(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    // -----------------------------------------------------------------------
    // 入力の上限と安全な整数の範囲（共通規則 SPEC-UNIT-D11 の続き、X1・X3・X6・X8）
    // -----------------------------------------------------------------------

    const DISPOSAL_TABLES: [&str; 4] = [
        "disposal_records",
        "disposal_items",
        "inventory_movements",
        "operation_logs",
    ];

    #[test]
    fn test_create_disposal_req204_quantity_and_cost_limits() {
        // REQ-204 / X1・X6: 1 明細の数量は 9999999、原価は 99999999（1/100 円）まで。1 つ外は何も書かない
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "LIM-D", 0);
        create_disposal(
            &mut conn,
            make_disposal_req(
                "lim-d-ok",
                vec![disposal_item("LIM-D", 9_999_999, 99_999_999, "合成")],
            ),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "LIM-D"), -9_999_999);

        for (key, quantity, cost, message) in [
            ("lim-d-qty", 10_000_000, 300, "明細2: 数量が大きすぎます"),
            ("lim-d-cost", 1, 100_000_000, "明細2: 原価が大きすぎます"),
        ] {
            let before = write_surface(&conn, &DISPOSAL_TABLES);
            let result = create_disposal(
                &mut conn,
                make_disposal_req(
                    key,
                    vec![
                        disposal_item("LIM-D", 1, 300, "合成"),
                        disposal_item("LIM-D", quantity, cost, "合成"),
                    ],
                ),
            );
            assert_rejected_without_writes(&conn, &DISPOSAL_TABLES, &before, result, message);
        }
    }

    #[test]
    fn test_create_disposal_req204_stock_limit_rolls_back_whole_tx() {
        // REQ-204 / X3: 計算の後の在庫が -999999999 を下回ったら TX 全体を戻す
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "STK-FULL", -999_999_999);
        create_test_product(&conn, "STK-NEAR", -999_999_998);
        create_test_product(&conn, "STK-OTHER", 10);
        let before = write_surface(&conn, &DISPOSAL_TABLES);
        let result = create_disposal(
            &mut conn,
            make_disposal_req(
                "stk-over",
                vec![
                    disposal_item("STK-OTHER", 5, 300, "合成"),
                    disposal_item("STK-FULL", 1, 300, "合成"),
                ],
            ),
        );
        assert_rejected_without_writes(
            &conn,
            &DISPOSAL_TABLES,
            &before,
            result,
            "在庫数が大きすぎます",
        );
        assert_eq!(stock_of(&conn, "STK-OTHER"), 10);

        create_disposal(
            &mut conn,
            make_disposal_req("stk-edge", vec![disposal_item("STK-NEAR", 1, 300, "合成")]),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "STK-NEAR"), -999_999_999);
    }

    /// X8 の fixture: 上限に近い明細（原価 99999900〈999999 円〉・数量 9999999）を A1〜A9 に 100 行ずつ（900 行、8999990100000900 円）
    fn limit_lines_900() -> Vec<(String, i64, i64)> {
        (1..=9)
            .flat_map(|product| {
                std::iter::repeat_n((format!("X8-A{product}"), 9_999_999, 99_999_900), 100)
            })
            .collect()
    }

    fn disposal_items_from(lines: &[(String, i64, i64)]) -> Vec<DisposalItemInput> {
        lines
            .iter()
            .map(|(code, quantity, cost)| disposal_item(code, *quantity, *cost, "合成"))
            .collect()
    }

    fn setup_total_limit_db() -> (tempfile::TempDir, DbConnection) {
        let (dir, conn) = setup_test_db();
        for product in 1..=9 {
            create_test_product(&conn, &format!("X8-A{product}"), 0);
        }
        create_test_product(&conn, "X8-B", 0);
        (dir, conn)
    }

    #[test]
    fn test_create_disposal_req204_total_loss_cost_safe_integer_limit() {
        // REQ-204 / X8: 円の合計は最初の書込みの前に ±9007199254740991 で確かめる
        let (_dir, mut conn) = setup_total_limit_db();
        let lines = limit_lines_900();
        let result = create_disposal(
            &mut conn,
            make_disposal_req("x8-900", disposal_items_from(&lines)),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "X8-A1"), -999_999_900);
        let detail =
            crate::biz::inventory_service::get_disposal_record(&conn, result.record_id).unwrap();
        assert_eq!(detail.total_loss_cost, 8_999_990_100_000_900);

        let (_dir, mut conn) = setup_total_limit_db();
        let mut edge = limit_lines_900();
        edge.push(("X8-B".to_string(), 7_209_161, 99_999_900));
        edge.push(("X8-B".to_string(), 1, 94_925_200));
        let result = create_disposal(
            &mut conn,
            make_disposal_req("x8-edge", disposal_items_from(&edge)),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "X8-B"), -7_209_162);
        let detail =
            crate::biz::inventory_service::get_disposal_record(&conn, result.record_id).unwrap();
        assert_eq!(detail.total_loss_cost, 9_007_199_254_740_991);

        let mut one_over = limit_lines_900();
        one_over.push(("X8-B".to_string(), 7_209_161, 99_999_900));
        one_over.push(("X8-B".to_string(), 1, 94_925_300));
        let mut lines_901 = limit_lines_900();
        lines_901.push(("X8-B".to_string(), 9_999_999, 99_999_900));
        for (key, lines) in [("x8-over", one_over), ("x8-901", lines_901)] {
            let (_dir, mut conn) = setup_total_limit_db();
            let before = write_surface(&conn, &DISPOSAL_TABLES);
            let result = create_disposal(
                &mut conn,
                make_disposal_req(key, disposal_items_from(&lines)),
            );
            assert_rejected_without_writes(
                &conn,
                &DISPOSAL_TABLES,
                &before,
                result,
                "合計金額が大きすぎます",
            );
        }
    }

    #[test]
    fn test_create_disposal_req204_stock_warning_formats_length_in_meters() {
        // REQ-204 / SPEC-UNIT-D3 の表の 10: 在庫警告の文は、長さの商品（m）の在庫を m の形で入れる
        let (_dir, mut conn) = setup_test_db();
        create_test_product_with_unit(&conn, "W-M", 100, "m");
        create_test_product_with_unit(&conn, "W-CM", 100, "m");
        create_test_product_with_unit(&conn, "W-PCS", 1, "pcs");
        let result = create_disposal(
            &mut conn,
            make_disposal_req(
                "warn-len",
                vec![
                    disposal_item("W-M", 130, 300, "合成"),
                    disposal_item("W-CM", 130, 300, "合成"),
                    disposal_item("W-PCS", 3, 300, "合成"),
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
    fn test_create_disposal_req204_rejects_noncanonical_date() {
        // REQ-204 / Y1（4 業務の全部）/ Matrix W3
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "W3-DSP", 3);
        let before = write_surface(&conn, &DISPOSAL_TABLES);
        let mut req = make_disposal_req("w3-dsp", vec![disposal_item("W3-DSP", 1, 100, "破損")]);
        req.disposal_date = "2026-4-7".to_string();
        let result = create_disposal(&mut conn, req);
        assert_rejected_without_writes(
            &conn,
            &DISPOSAL_TABLES,
            &before,
            result,
            "日付の形式が不正です（YYYY-MM-DD）",
        );
    }
}
