//! 返品・交換記録（create_return）

use crate::biz::csv_import_service::z004_moves_stock;
use crate::biz::BizError;
use crate::constants::MAX_LINE_QUANTITY;
use crate::db::inventory_repo::{MovementType, ReferenceType};
use crate::db::product_repo;
use crate::db::return_repo;
use crate::db::return_repo::{ReturnDirection, ReturnExchangeType};
use crate::db::system_repo;
use crate::db::{DbConnection, DbError};
use crate::io::image_manager;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::common::{
    apply_stock_change, compute_fingerprint, negative_stock_warning_message,
    validate_business_date, IDEMPOTENCY_KEY_MAX_LEN,
};

const ALLOWED_RECEIPT_IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

/// 領収書画像を保存して相対パスを返す。
///
/// 31-biz-inventory-service.md §12.9
pub fn save_receipt_image(
    app_data_dir: &Path,
    image_bytes: &[u8],
    extension: &str,
) -> Result<String, BizError> {
    let ext_lower = extension.to_lowercase();
    if !ALLOWED_RECEIPT_IMAGE_EXTENSIONS.contains(&ext_lower.as_str()) {
        return Err(BizError::ValidationFailedAt {
            message: format!(
                "不正な画像拡張子: {}（許可: {}）",
                extension,
                ALLOWED_RECEIPT_IMAGE_EXTENSIONS.join(", ")
            ),
            field: "extension".to_string(),
        });
    }

    image_manager::save_receipt_image(app_data_dir, image_bytes, extension).map_err(|error| {
        BizError::DatabaseError(DbError::QueryFailed(format!(
            "画像の保存でエラーが発生しました: {error}"
        )))
    })
}

/// 返品・交換記録リクエスト（31-biz-inventory-service.md §12.4）
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct ReturnCreateRequest {
    pub idempotency_key: String,
    pub return_type: ReturnExchangeType,
    pub return_date: String,
    pub register_processed: bool,
    pub receipt_image_path: Option<String>,
    pub note: Option<String>,
    pub items: Vec<ReturnItemInput>,
}

#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct ReturnItemInput {
    pub product_code: String,
    pub direction: ReturnDirection,
    pub quantity: i64,
}

/// 返品・交換記録の結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ReturnCreateResult {
    pub record_id: i64,
    pub created: bool,
    pub idempotent_replay: bool,
    pub stock_warnings: Vec<String>,
    /// この保存で在庫を動かした商品コード（昇順・重複なし。冪等の再送では記録の movement から作る。D-148 (3)）
    pub stock_applied_product_codes: Vec<String>,
}

fn validate_return_request_shape(req: &ReturnCreateRequest) -> Result<(), BizError> {
    if req.items.is_empty() {
        return Err(BizError::ValidationFailed(
            "明細が1件以上必要です".to_string(),
        ));
    }
    let mut has_in_item = false;
    let mut has_out_item = false;
    for (i, item) in req.items.iter().enumerate() {
        if item.direction == ReturnDirection::In {
            has_in_item = true;
        }
        if item.direction == ReturnDirection::Out {
            has_out_item = true;
        }
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
    }
    if req.return_type == ReturnExchangeType::Return && has_out_item {
        return Err(BizError::ValidationFailed(
            "返品では渡し明細を指定できません".to_string(),
        ));
    }
    if req.return_type == ReturnExchangeType::Exchange && (!has_in_item || !has_out_item) {
        return Err(BizError::ValidationFailed(
            "交換では戻り明細と渡し明細がそれぞれ1件以上必要です".to_string(),
        ));
    }
    // 業務日の形と暦の検査（31）。明細ごとに在庫を動かすかが分かれる前なので、
    // movement を作らない明細（レジで読んで戻した在庫連動の商品）だけの記録も同じ検査を通る
    validate_business_date(&req.return_date)?;

    Ok(())
}

fn compute_return_fingerprint(req: &ReturnCreateRequest) -> String {
    let header = format!(
        "{}|{}|{}",
        req.return_type.as_str(),
        req.return_date,
        req.register_processed
    );
    let item_lines: Vec<String> = req
        .items
        .iter()
        .map(|item| {
            format!(
                "{}|{}|{}",
                item.product_code,
                item.quantity,
                item.direction.as_str()
            )
        })
        .collect();
    compute_fingerprint(&header, &item_lines)
}

/// 冪等の再送の結果。この保存で在庫を動かした商品コードは、記録の movement（詳細の取得と同じ読取り）から作る
fn replay_result(conn: &DbConnection, record_id: i64) -> Result<ReturnCreateResult, BizError> {
    let codes: BTreeSet<String> = return_repo::get_return_record_detail(conn, record_id)?
        .movements
        .into_iter()
        .map(|movement| movement.product_code)
        .collect();
    Ok(ReturnCreateResult {
        record_id,
        created: false,
        idempotent_replay: true,
        stock_warnings: vec![],
        stock_applied_product_codes: codes.into_iter().collect(),
    })
}

/// 明細の商品の在庫をこの保存で動かすか（D-148 (2)、31 §12.4）。
///
/// register_processed=false（レジでバーコードを読んでいない）→ 動かす。
/// register_processed=true（レジで読んで戻した）→ Z004 の取込みの確定がその商品の在庫を動かすときだけ動かさない
/// （判定は Z004 の確定と同じ `z004_moves_stock`。JAN は保存された値をそのまま渡し、ここで正規化しない）。
fn applies_stock_on_save(
    conn: &DbConnection,
    register_processed: bool,
    product_code: &str,
) -> Result<bool, BizError> {
    if !register_processed {
        return Ok(true);
    }
    let product = product_repo::find_by_product_code(conn, product_code)?
        .ok_or_else(|| BizError::NotFound(format!("商品が見つかりません: {product_code}")))?;
    match product.product.jan_code.as_deref() {
        Some(jan) => Ok(!z004_moves_stock(
            jan,
            &product_repo::find_by_jan_code(conn, jan)?,
        )),
        None => Ok(true),
    }
}

/// 返品・交換記録を登録する
///
/// 明細ごとに、register_processed=false か、Z004 の取込みが在庫を動かさない商品なら、
/// direction に基づきこの保存で在庫を動かす（D-148）
///
/// 31-biz-inventory-service.md §12.4
pub fn create_return(
    conn: &mut DbConnection,
    req: ReturnCreateRequest,
) -> Result<ReturnCreateResult, BizError> {
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

    // DB参照不要なBIZ契約は replay より先に検証する。
    validate_return_request_shape(&req)?;

    // fingerprint（リクエストフィールドのみ使用、DB参照不要）
    let fingerprint = compute_return_fingerprint(&req);

    // 冪等性チェック（バリデーションより先に実行）
    if let Some((existing_id, existing_fp)) =
        return_repo::find_return_by_idempotency_key(conn, &normalized_key)?
    {
        if existing_fp == fingerprint {
            return replay_result(conn, existing_id);
        } else {
            return Err(BizError::IdempotencyConflict(
                "同じ冪等キーで異なる内容のリクエストです".to_string(),
            ));
        }
    }

    // バリデーション（DB参照あり）
    for item in req.items.iter() {
        if product_repo::find_by_product_code(conn, &item.product_code)?.is_none() {
            return Err(BizError::NotFound(format!(
                "商品が見つかりません: {}",
                item.product_code
            )));
        }
    }

    // TX
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    let record_id = match return_repo::insert_return_record(
        &tx,
        &crate::db::return_repo::NewReturnRecord {
            return_type: req.return_type.as_str().to_string(),
            return_date: req.return_date.clone(),
            register_processed: req.register_processed,
            receipt_image_path: req.receipt_image_path.clone(),
            note: req.note.clone(),
            idempotency_key: normalized_key.clone(),
            request_fingerprint: fingerprint.clone(),
        },
    ) {
        Ok(id) => id,
        Err(DbError::DuplicateKey(_)) => {
            let (existing_id, existing_fp) =
                return_repo::find_return_by_idempotency_key(&tx, &normalized_key)?.ok_or_else(
                    || {
                        BizError::DatabaseError(DbError::QueryFailed(
                            "DuplicateKey後にレコードが見つかりません".to_string(),
                        ))
                    },
                )?;
            if existing_fp == fingerprint {
                return replay_result(&tx, existing_id);
            } else {
                return Err(BizError::IdempotencyConflict(
                    "同じ冪等キーで異なる内容のリクエストです".to_string(),
                ));
            }
        }
        Err(e) => return Err(BizError::DatabaseError(e)),
    };

    let mut stock_warnings_map: BTreeMap<String, String> = BTreeMap::new();
    let mut stock_applied: BTreeSet<String> = BTreeSet::new();
    for item in &req.items {
        return_repo::insert_return_item(
            &tx,
            &crate::db::return_repo::NewReturnItem {
                return_record_id: record_id,
                product_code: item.product_code.clone(),
                direction: item.direction.as_str().to_string(),
                quantity: item.quantity,
            },
        )?;

        // Z004 の取込みが在庫に戻す明細は動かさない（二重に戻さない）。それ以外はこの保存で動かす
        if applies_stock_on_save(&tx, req.register_processed, &item.product_code)? {
            let qty = if item.direction == ReturnDirection::In {
                item.quantity
            } else {
                -item.quantity
            };
            let outcome = apply_stock_change(
                &tx,
                &item.product_code,
                &req.return_date,
                qty,
                MovementType::Return,
                ReferenceType::ReturnRecord,
                record_id,
                None,
            )?;
            if outcome.negative_stock_warning {
                stock_warnings_map.insert(
                    item.product_code.clone(),
                    negative_stock_warning_message(&item.product_code, &outcome),
                );
            }
            stock_applied.insert(item.product_code.clone());
        }
    }

    // BTreeMap → Vec（product_code 昇順・重複排除済み）
    let stock_warnings: Vec<String> = stock_warnings_map.into_values().collect();

    let detail = serde_json::json!({
        "record_type": "return_record",
        "record_id": record_id,
        "item_count": req.items.len(),
        "return_type": req.return_type,
        "register_processed": req.register_processed,
        "idempotency_key": normalized_key,
    });
    system_repo::insert_operation_log(
        &tx,
        &crate::db::system_repo::NewOperationLog {
            operation_type: "return_create".to_string(),
            summary: format!("返品記録作成（{}明細）", req.items.len()),
            detail_json: Some(detail.to_string()),
        },
    )?;

    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(ReturnCreateResult {
        record_id,
        created: true,
        idempotent_replay: false,
        stock_warnings,
        stock_applied_product_codes: stock_applied.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::biz::csv_import_service::test_support::create_test_product_with_jan;
    use crate::db::product_repo;
    use std::fs;

    const LEGACY_INVALID_EXTENSION_TEST_NAME: &str =
        "test_save_receipt_image_req905_invalid_extension_to_validation";

    fn make_return_req(key: &str, items: Vec<ReturnItemInput>) -> ReturnCreateRequest {
        ReturnCreateRequest {
            idempotency_key: key.to_string(),
            return_type: ReturnExchangeType::Return,
            return_date: "2026-04-07".to_string(),
            register_processed: false,
            receipt_image_path: None,
            note: None,
            items,
        }
    }

    fn return_item(code: &str, direction: &str, qty: i64) -> ReturnItemInput {
        ReturnItemInput {
            product_code: code.to_string(),
            direction: match direction {
                "in" => ReturnDirection::In,
                "out" => ReturnDirection::Out,
                _ => panic!("test helper received invalid direction"),
            },
            quantity: qty,
        }
    }

    #[test]
    fn test_return_fingerprint_fixed_value_req202() {
        let req = make_return_req("fp-return", vec![return_item("FP-RT", "in", 2)]);
        assert_eq!(
            compute_return_fingerprint(&req),
            "5605e45a7f2b3875dfb4c28c261de451eb5e930aa50955f2fdf29d474a9efebd"
        );
    }

    #[test]
    fn test_save_receipt_image_req906_accepts_supported_extensions() {
        // REQ-906 / SPEC-CMD11-D3: BIZ が許可集合を所有し、IO 保存へ委譲する。
        for extension in ["jpg", "jpeg", "png", "gif", "webp", "JPG"] {
            let dir = tempfile::tempdir().unwrap();

            let relative_path = save_receipt_image(dir.path(), b"image", extension).unwrap();

            assert!(relative_path.starts_with("images/receipts/"));
            assert!(dir.path().join(relative_path).exists());
        }
    }

    #[test]
    fn test_save_receipt_image_req906_invalid_extension_to_validation() {
        // REQ-906 / SPEC-CMD11-D3: message と field を独立転記で固定する。
        let dir = tempfile::tempdir().unwrap();

        let result = save_receipt_image(dir.path(), b"image", "bmp");

        assert!(
            matches!(
                result,
                Err(BizError::ValidationFailedAt { ref message, ref field })
                    if message == "不正な画像拡張子: bmp（許可: jpg, jpeg, png, gif, webp）"
                        && field == "extension"
            ),
            "{LEGACY_INVALID_EXTENSION_TEST_NAME} から移設した wire 契約"
        );
    }

    #[test]
    fn test_save_receipt_image_req906_wraps_filesystem_failure() {
        // REQ-906 / SPEC-CMD11-D3: filesystem error は DatabaseError へ伝搬する。
        let dir = tempfile::tempdir().unwrap();
        let not_a_directory = dir.path().join("file");
        fs::write(&not_a_directory, b"block directory creation").unwrap();

        let result = save_receipt_image(&not_a_directory, b"image", "jpg");

        assert!(matches!(result, Err(BizError::DatabaseError(_))));
    }

    #[test]
    fn test_create_return_req202_normal() {
        // REQ-202: 返品・交換記録 — 正常な返品記録作成（register_processed=false, direction=in → 在庫増加）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-001", 10);

        let req = make_return_req("ret-key-1", vec![return_item("RTN-001", "in", 2)]);
        let result = create_return(&mut conn, req).unwrap();
        assert!(result.created);
        assert!(!result.idempotent_replay);

        let p = product_repo::find_by_product_code(&conn, "RTN-001")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 12);
    }

    #[test]
    fn test_create_return_req202_operation_log_record_contract() {
        // REQ-202 / UI-11c-D7: 返品記録の operation_log が関連記録 contract を満たすこと
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-LOG", 10);

        let req = make_return_req("ret-log-key", vec![return_item("RTN-LOG", "in", 2)]);
        let result = create_return(&mut conn, req).unwrap();

        let (op_type, detail_str): (String, String) = conn
            .query_row(
                "SELECT operation_type, detail_json FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(op_type, "return_create");

        let detail: serde_json::Value = serde_json::from_str(&detail_str).unwrap();
        assert_eq!(detail["record_type"], "return_record");
        assert!(detail["record_id"].is_number());
        assert_eq!(detail["record_id"].as_i64(), Some(result.record_id));
        assert_eq!(detail["item_count"], 1);
        assert_eq!(detail["return_type"], "return");
        assert_eq!(detail["register_processed"], false);
        assert_eq!(detail["idempotency_key"], "ret-log-key");
    }

    /// 13 桁の数字の JAN を持ち、その JAN の商品が 1 つだけの在庫連動の商品（Z004 の取込みが在庫を動かす）
    fn create_linked_jan_product(conn: &DbConnection, code: &str, jan: &str, stock: i64) {
        create_test_product_with_jan(conn, code, jan, stock, true);
    }

    fn return_movement_count(conn: &DbConnection, code: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM inventory_movements WHERE product_code = ?1 AND reference_type = 'return_record'",
            [code],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn register_processed_return(key: &str, items: Vec<ReturnItemInput>) -> ReturnCreateRequest {
        let mut req = make_return_req(key, items);
        req.register_processed = true;
        req
    }

    #[test]
    fn test_create_return_req202_register_processed_linked_jan_no_stock_change() {
        // REQ-202 / D-148 (2) / B1: 「読んで戻した」かつ Z004 が在庫を動かす商品は、この保存で在庫を動かさない
        let (_dir, mut conn) = setup_test_db();
        create_linked_jan_product(&conn, "RTN-002", "4901234567887", 10);

        let req = register_processed_return("ret-key-2", vec![return_item("RTN-002", "in", 3)]);
        let result = create_return(&mut conn, req).unwrap();
        assert!(result.created);

        // 在庫不変・movement なし（Z004 と二重に戻らない）
        assert_eq!(stock_of(&conn, "RTN-002"), 10);
        assert_eq!(return_movement_count(&conn, "RTN-002"), 0);
        assert!(result.stock_applied_product_codes.is_empty());
    }

    #[test]
    fn test_create_return_req202_register_processed_without_jan_moves_stock() {
        // REQ-202 / D-148 (2) / B2: JAN 無しの商品は Z004 の行に当たらないので、「読んで戻した」でもこの保存で動かす
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-NOJAN", 10);

        let req = register_processed_return("ret-nojan", vec![return_item("RTN-NOJAN", "in", 3)]);
        let result = create_return(&mut conn, req).unwrap();

        assert_eq!(stock_of(&conn, "RTN-NOJAN"), 13);
        assert_eq!(return_movement_count(&conn, "RTN-NOJAN"), 1);
        assert_eq!(
            result.stock_applied_product_codes,
            vec!["RTN-NOJAN".to_string()]
        );
    }

    #[test]
    fn test_create_return_req202_register_processed_not_stock_linked_moves_stock() {
        // REQ-202 / D-148 (2) / B3: pos_stock_sync=false の商品は Z004 が在庫を動かさないので、この保存で動かす
        let (_dir, mut conn) = setup_test_db();
        create_test_product_with_jan(&conn, "RTN-NOSYNC", "4901234567887", 10, false);

        let req = register_processed_return("ret-nosync", vec![return_item("RTN-NOSYNC", "in", 2)]);
        let result = create_return(&mut conn, req).unwrap();

        assert_eq!(stock_of(&conn, "RTN-NOSYNC"), 12);
        assert_eq!(return_movement_count(&conn, "RTN-NOSYNC"), 1);
        assert_eq!(
            result.stock_applied_product_codes,
            vec!["RTN-NOSYNC".to_string()]
        );
    }

    #[test]
    fn test_create_return_req202_register_processed_shared_jan_moves_stock() {
        // REQ-202 / D-148 (2) / B4: 共有 JAN（候補 2）の商品は Z004 が在庫を動かさないので、この保存で動かす
        let (_dir, mut conn) = setup_test_db();
        create_linked_jan_product(&conn, "RTN-SHARE-A", "4901234567887", 10);
        create_linked_jan_product(&conn, "RTN-SHARE-B", "4901234567887", 10);

        let req =
            register_processed_return("ret-shared", vec![return_item("RTN-SHARE-A", "in", 1)]);
        let result = create_return(&mut conn, req).unwrap();

        assert_eq!(stock_of(&conn, "RTN-SHARE-A"), 11);
        assert_eq!(stock_of(&conn, "RTN-SHARE-B"), 10);
        assert_eq!(return_movement_count(&conn, "RTN-SHARE-A"), 1);
        assert_eq!(
            result.stock_applied_product_codes,
            vec!["RTN-SHARE-A".to_string()]
        );
    }

    #[test]
    fn test_create_return_req202_register_processed_exchange_mixed_lines() {
        // REQ-202 / D-148 (2) / B5: 交換の明細ごとに判定し、渡しは在庫を減らす
        let (_dir, mut conn) = setup_test_db();
        create_linked_jan_product(&conn, "MIX-IN-LINK", "4901234567887", 10);
        create_test_product(&conn, "MIX-IN-NOJAN", 10);
        create_linked_jan_product(&conn, "MIX-OUT-LINK", "4909876543214", 10);
        create_test_product(&conn, "MIX-OUT-NOJAN", 10);

        let mut req = register_processed_return(
            "ret-mixed",
            vec![
                return_item("MIX-IN-LINK", "in", 1),
                return_item("MIX-IN-NOJAN", "in", 2),
                return_item("MIX-OUT-LINK", "out", 3),
                return_item("MIX-OUT-NOJAN", "out", 4),
            ],
        );
        req.return_type = ReturnExchangeType::Exchange;
        let result = create_return(&mut conn, req).unwrap();

        assert_eq!(stock_of(&conn, "MIX-IN-LINK"), 10);
        assert_eq!(stock_of(&conn, "MIX-IN-NOJAN"), 12);
        assert_eq!(stock_of(&conn, "MIX-OUT-LINK"), 10);
        assert_eq!(stock_of(&conn, "MIX-OUT-NOJAN"), 6);
        assert_eq!(return_movement_count(&conn, "MIX-IN-LINK"), 0);
        assert_eq!(return_movement_count(&conn, "MIX-OUT-LINK"), 0);
        assert_eq!(
            result.stock_applied_product_codes,
            vec!["MIX-IN-NOJAN".to_string(), "MIX-OUT-NOJAN".to_string()]
        );
    }

    #[test]
    fn test_create_return_req202_register_processed_jan8_moves_stock() {
        // REQ-202 / D-148 (2) / B8: JAN-8 は Z004 の照合の形（13 桁）でないので、候補が 1 つの在庫連動でもこの保存で動かす
        let (_dir, mut conn) = setup_test_db();
        create_linked_jan_product(&conn, "RTN-JAN8", "49123456", 10);

        let req = register_processed_return("ret-jan8", vec![return_item("RTN-JAN8", "in", 1)]);
        let result = create_return(&mut conn, req).unwrap();

        assert_eq!(stock_of(&conn, "RTN-JAN8"), 11);
        assert_eq!(return_movement_count(&conn, "RTN-JAN8"), 1);
        assert_eq!(
            result.stock_applied_product_codes,
            vec!["RTN-JAN8".to_string()]
        );
    }

    #[test]
    fn test_create_return_req202_unprocessed_moves_every_line() {
        // REQ-202 / D-148 (2): 「読んでいない」は Z004 が在庫を動かす商品の明細も含めて全明細で動かす
        let (_dir, mut conn) = setup_test_db();
        create_linked_jan_product(&conn, "UNP-LINK", "4901234567887", 10);
        create_test_product(&conn, "UNP-NOJAN", 10);

        let req = make_return_req(
            "ret-unprocessed",
            vec![
                return_item("UNP-NOJAN", "in", 1),
                return_item("UNP-LINK", "in", 2),
            ],
        );
        let result = create_return(&mut conn, req).unwrap();

        assert_eq!(stock_of(&conn, "UNP-LINK"), 12);
        assert_eq!(stock_of(&conn, "UNP-NOJAN"), 11);
        assert_eq!(
            result.stock_applied_product_codes,
            vec!["UNP-LINK".to_string(), "UNP-NOJAN".to_string()]
        );
    }

    #[test]
    fn test_create_return_req202_replay_returns_stock_applied_codes() {
        // REQ-202 / D-148 (3) / B6: 冪等の再送も記録の movement から同じ列を返す（cache を古いまま残さない）
        let (_dir, mut conn) = setup_test_db();
        create_linked_jan_product(&conn, "RPL-LINK", "4901234567887", 10);
        create_test_product(&conn, "RPL-NOJAN-B", 10);
        create_test_product(&conn, "RPL-NOJAN-A", 10);

        let req = register_processed_return(
            "ret-replay-codes",
            vec![
                return_item("RPL-NOJAN-B", "in", 1),
                return_item("RPL-LINK", "in", 1),
                return_item("RPL-NOJAN-A", "in", 1),
            ],
        );
        let first = create_return(&mut conn, req.clone()).unwrap();
        let replay = create_return(&mut conn, req).unwrap();

        assert!(replay.idempotent_replay);
        assert_eq!(
            first.stock_applied_product_codes,
            vec!["RPL-NOJAN-A".to_string(), "RPL-NOJAN-B".to_string()]
        );
        assert_eq!(
            replay.stock_applied_product_codes,
            first.stock_applied_product_codes
        );
        assert_eq!(stock_of(&conn, "RPL-NOJAN-A"), 11, "再送で二重に動かさない");
    }

    #[test]
    fn test_create_return_req202_direction_in_increases_stock() {
        // REQ-202: 返品・交換記録 — direction=in → 在庫増加
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-003", 5);

        let req = make_return_req("ret-key-3", vec![return_item("RTN-003", "in", 3)]);
        create_return(&mut conn, req).unwrap();

        let p = product_repo::find_by_product_code(&conn, "RTN-003")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 8);
    }

    #[test]
    fn test_create_return_req202_direction_out_decreases_stock() {
        // REQ-202: 返品・交換記録 — exchange の direction=out → 在庫減少
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-004-IN", 10);
        create_test_product(&conn, "RTN-004", 10);

        let mut req = make_return_req(
            "ret-key-4",
            vec![
                return_item("RTN-004-IN", "in", 1),
                return_item("RTN-004", "out", 2),
            ],
        );
        req.return_type = ReturnExchangeType::Exchange;
        create_return(&mut conn, req).unwrap();

        let p = product_repo::find_by_product_code(&conn, "RTN-004")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 8);
    }

    #[test]
    fn test_create_return_req202_idempotent_replay() {
        // REQ-202: 返品・交換記録 — 冪等性リプレイ（副作用ゼロ）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-005", 10);

        let req = make_return_req("ret-key-5", vec![return_item("RTN-005", "in", 2)]);
        let r1 = create_return(&mut conn, req.clone()).unwrap();

        let log_count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();

        let r2 = create_return(&mut conn, req).unwrap();
        assert!(!r2.created);
        assert!(r2.idempotent_replay);
        assert_eq!(r2.record_id, r1.record_id);

        // 在庫は1回目の +2 のみ
        let p = product_repo::find_by_product_code(&conn, "RTN-005")
            .unwrap()
            .unwrap();
        assert_eq!(p.product.stock_quantity, 12);

        let log_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(log_count_before, log_count_after);
    }

    #[test]
    fn test_create_return_req202_idempotency_conflict() {
        // REQ-202: 返品・交換記録 — 同じキー+異なる内容 → IdempotencyConflict
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-006", 10);

        let req1 = make_return_req("ret-key-6", vec![return_item("RTN-006", "in", 2)]);
        create_return(&mut conn, req1).unwrap();

        let req2 = make_return_req("ret-key-6", vec![return_item("RTN-006", "in", 5)]);
        let result = create_return(&mut conn, req2);
        assert!(matches!(result, Err(BizError::IdempotencyConflict(_))));
    }

    #[test]
    fn test_create_return_req202_validation_empty_key() {
        // REQ-202: 返品・交換記録 — バリデーション: 空白のみの冪等性キー → エラー
        let (_dir, mut conn) = setup_test_db();
        let req = make_return_req("  ", vec![return_item("X", "in", 1)]);
        let result = create_return(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_return_req202_rejects_return_with_out_direction() {
        // REQ-202 / UI-03-D6: 返品は戻り(in)のみ。渡し(out)を含む返品はBIZで拒否する
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-012", 10);

        let req = make_return_req("ret-key-12", vec![return_item("RTN-012", "out", 1)]);
        let result = create_return(&mut conn, req);

        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_return_req202_rejects_exchange_missing_in_or_out() {
        // REQ-202 / UI-03-D7: 交換は戻り(in)と渡し(out)の両方が必要
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-013", 10);
        create_test_product(&conn, "RTN-014", 10);

        let mut only_in = make_return_req("ret-key-13", vec![return_item("RTN-013", "in", 1)]);
        only_in.return_type = ReturnExchangeType::Exchange;
        let mut only_out = make_return_req("ret-key-14", vec![return_item("RTN-014", "out", 1)]);
        only_out.return_type = ReturnExchangeType::Exchange;

        assert!(matches!(
            create_return(&mut conn, only_in),
            Err(BizError::ValidationFailed(_))
        ));
        assert!(matches!(
            create_return(&mut conn, only_out),
            Err(BizError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_create_return_req202_rejects_invalid_replay_before_idempotency_return() {
        // REQ-202 / UI-03-D6: 過去に存在し得る不正レコードも replay で許可しない
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-015", 10);

        let req = make_return_req("ret-key-15", vec![return_item("RTN-015", "out", 1)]);
        let header = format!(
            "{}|{}|{}",
            req.return_type.as_str(),
            req.return_date,
            req.register_processed
        );
        let item_lines: Vec<String> = req
            .items
            .iter()
            .map(|i| format!("{}|{}|{}", i.product_code, i.quantity, i.direction.as_str()))
            .collect();
        let fingerprint = compute_fingerprint(&header, &item_lines);
        let record_id = return_repo::insert_return_record(
            &conn,
            &crate::db::return_repo::NewReturnRecord {
                return_type: req.return_type.as_str().to_string(),
                return_date: req.return_date.clone(),
                register_processed: req.register_processed,
                receipt_image_path: req.receipt_image_path.clone(),
                note: req.note.clone(),
                idempotency_key: req.idempotency_key.clone(),
                request_fingerprint: fingerprint,
            },
        )
        .unwrap();
        return_repo::insert_return_item(
            &conn,
            &crate::db::return_repo::NewReturnItem {
                return_record_id: record_id,
                product_code: "RTN-015".to_string(),
                direction: "out".to_string(),
                quantity: 1,
            },
        )
        .unwrap();

        let result = create_return(&mut conn, req);

        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_create_return_req202_exchange_type() {
        // REQ-202: 返品・交換記録 — return_type="exchange" も正常に動作すること
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-010A", 10);
        create_test_product(&conn, "RTN-010B", 10);

        let mut req = make_return_req(
            "ret-key-10",
            vec![
                return_item("RTN-010A", "in", 1),
                return_item("RTN-010B", "out", 1),
            ],
        );
        req.return_type = ReturnExchangeType::Exchange;
        let result = create_return(&mut conn, req).unwrap();
        assert!(result.created);

        let pa = product_repo::find_by_product_code(&conn, "RTN-010A")
            .unwrap()
            .unwrap();
        assert_eq!(pa.product.stock_quantity, 11);

        let pb = product_repo::find_by_product_code(&conn, "RTN-010B")
            .unwrap()
            .unwrap();
        assert_eq!(pb.product.stock_quantity, 9);
    }

    #[test]
    fn test_create_return_req202_validation_quantity_zero() {
        // REQ-202: 返品・交換記録 — バリデーション: 数量 <= 0 → エラー
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "RTN-011", 10);

        let req = make_return_req("ret-key-11", vec![return_item("RTN-011", "in", 0)]);
        let result = create_return(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    // -----------------------------------------------------------------------
    // 入力の上限（共通規則 SPEC-UNIT-D11 の続き、X1・X3）
    // -----------------------------------------------------------------------

    const RETURN_TABLES: [&str; 4] = [
        "return_records",
        "return_items",
        "inventory_movements",
        "operation_logs",
    ];

    #[test]
    fn test_create_return_req202_quantity_limit() {
        // REQ-202 / X1: 1 明細の数量は 9999999 まで。1 つ外は何も書かない
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "LIM-RT", 0);
        create_return(
            &mut conn,
            make_return_req("lim-rt-ok", vec![return_item("LIM-RT", "in", 9_999_999)]),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "LIM-RT"), 9_999_999);

        let before = write_surface(&conn, &RETURN_TABLES);
        let result = create_return(
            &mut conn,
            make_return_req("lim-rt-over", vec![return_item("LIM-RT", "in", 10_000_000)]),
        );
        assert_rejected_without_writes(
            &conn,
            &RETURN_TABLES,
            &before,
            result,
            "明細1: 数量が大きすぎます",
        );
    }

    #[test]
    fn test_create_return_req202_quantity_limit_applies_to_register_processed() {
        // REQ-202 / X1: 在庫を動かさない明細（レジでバーコードを読んで戻した在庫連動の商品）も同じ上限の検査を通る
        let (_dir, mut conn) = setup_test_db();
        // D-148: 在庫を動かさない明細は Z004 が在庫を動かす商品（13 桁の JAN が 1 商品だけの在庫連動）
        create_linked_jan_product(&conn, "LIM-RP", "4901234567887", 7);
        let before = write_surface(&conn, &RETURN_TABLES);
        let mut over =
            make_return_req("lim-rp-over", vec![return_item("LIM-RP", "in", 10_000_000)]);
        over.register_processed = true;
        let result = create_return(&mut conn, over);
        assert_rejected_without_writes(
            &conn,
            &RETURN_TABLES,
            &before,
            result,
            "明細1: 数量が大きすぎます",
        );

        let mut edge = make_return_req("lim-rp-edge", vec![return_item("LIM-RP", "in", 9_999_999)]);
        edge.register_processed = true;
        create_return(&mut conn, edge).unwrap();
        let saved: i64 = conn
            .query_row("SELECT quantity FROM return_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(saved, 9_999_999);
        assert_eq!(stock_of(&conn, "LIM-RP"), 7, "在庫は変わらない");
    }

    #[test]
    fn test_create_return_req202_stock_limit_rolls_back_whole_tx() {
        // REQ-202 / X3: 計算の後の在庫が 999999999 を超えたら TX 全体を戻す
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "STK-FULL", 999_999_999);
        create_test_product(&conn, "STK-NEAR", 999_999_998);
        let before = write_surface(&conn, &RETURN_TABLES);
        let result = create_return(
            &mut conn,
            make_return_req("stk-over", vec![return_item("STK-FULL", "in", 1)]),
        );
        assert_rejected_without_writes(
            &conn,
            &RETURN_TABLES,
            &before,
            result,
            "在庫数が大きすぎます",
        );

        create_return(
            &mut conn,
            make_return_req("stk-edge", vec![return_item("STK-NEAR", "in", 1)]),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "STK-NEAR"), 999_999_999);
    }

    #[test]
    fn test_create_return_req202_stock_warning_formats_length_in_meters() {
        // REQ-202 / SPEC-UNIT-D3 の表の 10: 在庫警告の文は、長さの商品（m）の在庫を m の形で入れる
        let (_dir, mut conn) = setup_test_db();
        create_test_product_with_unit(&conn, "W-M", -130, "m");
        create_test_product_with_unit(&conn, "W-CM", -130, "m");
        create_test_product_with_unit(&conn, "W-PCS", -3, "pcs");
        let result = create_return(
            &mut conn,
            make_return_req(
                "warn-len",
                vec![
                    return_item("W-M", "in", 100),
                    return_item("W-CM", "in", 100),
                    return_item("W-PCS", "in", 1),
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
    fn test_create_return_req202_rejects_noncanonical_date() {
        // REQ-202 / Y1（4 業務の全部）/ Matrix W3
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "W3-RT", 3);
        let before = write_surface(&conn, &RETURN_TABLES);
        let mut req = make_return_req("w3-rt", vec![return_item("W3-RT", "in", 1)]);
        req.return_date = "2026-4-7".to_string();
        let result = create_return(&mut conn, req);
        assert_rejected_without_writes(
            &conn,
            &RETURN_TABLES,
            &before,
            result,
            "日付の形式が不正です（YYYY-MM-DD）",
        );
    }

    #[test]
    fn test_create_return_req202_register_processed_rejects_noncanonical_date() {
        // REQ-202 / Y1・31（register_processed=true〈レジでバーコードを読んで戻した〉の記録も同じ検査）/ Matrix W7
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "W7-RT", 3);
        for (i, (date, message)) in [
            ("2026-4-7", "日付の形式が不正です（YYYY-MM-DD）"),
            ("2026-02-30", "存在しない日付です"),
        ]
        .iter()
        .enumerate()
        {
            let before = write_surface(&conn, &RETURN_TABLES);
            let mut req = make_return_req(&format!("w7-{i}"), vec![return_item("W7-RT", "in", 1)]);
            req.register_processed = true;
            req.return_date = date.to_string();
            let result = create_return(&mut conn, req);
            assert_rejected_without_writes(&conn, &RETURN_TABLES, &before, result, message);
        }
    }
}
