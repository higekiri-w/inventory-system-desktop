//! 振替（詰め替え・切り分け）の記録（create_transfer / get_transfer_record）
//!
//! 31-biz-inventory-service.md §12.6b に基づく実装（D-143）。

use crate::biz::BizError;
use crate::constants::MAX_LINE_QUANTITY;
use crate::db::inventory_repo::{MovementType, ReferenceType};
use crate::db::product_repo;
use crate::db::system_repo;
use crate::db::transfer_repo::{
    self, NewTransferItem, NewTransferRecord, TransferDirection, TransferRecordDetail,
};
use crate::db::{DbConnection, DbError};

use std::collections::{BTreeMap, HashSet};

use super::common::{
    apply_stock_change, compute_fingerprint, negative_stock_warning_message,
    validate_business_date, IDEMPOTENCY_KEY_MAX_LEN,
};
use super::list::resolve_movement_source;

/// 振替の記録のリクエスト（31 §12.6b）
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct TransferCreateRequest {
    pub idempotency_key: String,
    /// YYYY-MM-DD（詰め替え・切り分けた日）
    pub transfer_date: String,
    pub items: Vec<TransferItemInput>,
}

#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct TransferItemInput {
    pub product_code: String,
    pub direction: TransferDirection,
    /// 正の整数。その商品の在庫の数量の単位（個数は 1 単位、長さは cm。SPEC-UNIT-D2）
    pub quantity: i64,
}

/// 振替の記録の結果（廃棄の `DisposalCreateResult` と同じ形）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct TransferCreateResult {
    pub record_id: i64,
    pub created: bool,
    pub idempotent_replay: bool,
    pub stock_warnings: Vec<String>,
}

/// §12.8 の正規化: ヘッダ行 `{transfer_date}`、item 行 `{direction}|{product_code}|{quantity}`
fn compute_transfer_fingerprint(req: &TransferCreateRequest) -> String {
    let item_lines: Vec<String> = req
        .items
        .iter()
        .map(|item| {
            format!(
                "{}|{}|{}",
                item.direction.as_str(),
                item.product_code,
                item.quantity
            )
        })
        .collect();
    compute_fingerprint(&req.transfer_date, &item_lines)
}

fn replay_or_conflict(
    existing: (i64, String),
    fingerprint: &str,
) -> Result<TransferCreateResult, BizError> {
    let (existing_id, existing_fp) = existing;
    if existing_fp == fingerprint {
        Ok(TransferCreateResult {
            record_id: existing_id,
            created: false,
            idempotent_replay: true,
            stock_warnings: vec![],
        })
    } else {
        Err(BizError::IdempotencyConflict(
            "同じ冪等キーで異なる内容のリクエストです".to_string(),
        ))
    }
}

/// 振替の記録を登録し、減らす商品の在庫を減らし、増やす商品の在庫を増やす（1 つの TX）
///
/// 31-biz-inventory-service.md §12.6b
pub fn create_transfer(
    conn: &mut DbConnection,
    req: TransferCreateRequest,
) -> Result<TransferCreateResult, BizError> {
    // 0. 冪等キーの検査
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

    // 1. 明細の形の検査（DB を見ない）
    if req.items.is_empty() {
        return Err(BizError::ValidationFailed(
            "明細が1件以上必要です".to_string(),
        ));
    }
    let out_count = req
        .items
        .iter()
        .filter(|item| item.direction == TransferDirection::Out)
        .count();
    let in_count = req.items.len() - out_count;
    if out_count == 0 || in_count == 0 {
        return Err(BizError::ValidationFailed(
            "減らす商品と増やす商品を1つ以上ずつ入れてください".to_string(),
        ));
    }

    // 2. 業務日の検査
    validate_business_date(&req.transfer_date)?;

    // 3〜4. fingerprint と冪等性チェック
    let fingerprint = compute_transfer_fingerprint(&req);
    if let Some(existing) = transfer_repo::find_transfer_by_idempotency_key(conn, &normalized_key)?
    {
        return replay_or_conflict(existing, &fingerprint);
    }

    // 5. 明細の検証（DB 参照あり。明細の番号は items の順の 1 から）
    let mut seen = HashSet::new();
    for (i, item) in req.items.iter().enumerate() {
        let n = i + 1;
        if item.quantity <= 0 {
            return Err(BizError::ValidationFailed(format!(
                "明細{n}: 数量は1以上必要です"
            )));
        }
        if item.quantity > MAX_LINE_QUANTITY {
            return Err(BizError::ValidationFailed(format!(
                "明細{n}: 数量が大きすぎます"
            )));
        }
        // 向きを問わない（減らす商品を同じ記録で増やすことはできない）
        if !seen.insert(item.product_code.as_str()) {
            return Err(BizError::ValidationFailed(format!(
                "明細{n}: 同じ商品が2回入っています（{}）",
                item.product_code
            )));
        }
        if product_repo::find_by_product_code(conn, &item.product_code)?.is_none() {
            return Err(BizError::NotFound(format!(
                "商品が見つかりません: {}",
                item.product_code
            )));
        }
    }

    // 6. TX
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    // 7. ヘッダ（冪等キーの重複は再送の判定へ）
    let record_id = match transfer_repo::insert_transfer_record(
        &tx,
        &NewTransferRecord {
            transfer_date: req.transfer_date.clone(),
            idempotency_key: normalized_key.clone(),
            request_fingerprint: fingerprint.clone(),
        },
    ) {
        Ok(id) => id,
        Err(DbError::DuplicateKey(_)) => {
            let existing = transfer_repo::find_transfer_by_idempotency_key(&tx, &normalized_key)?
                .ok_or_else(|| {
                BizError::DatabaseError(DbError::QueryFailed(
                    "DuplicateKey後にレコードが見つかりません".to_string(),
                ))
            })?;
            return replay_or_conflict(existing, &fingerprint);
        }
        Err(e) => return Err(BizError::DatabaseError(e)),
    };

    // 8. 明細ごとに明細と movement（減らす明細は -quantity、増やす明細は +quantity）
    let mut stock_warnings_map: BTreeMap<String, String> = BTreeMap::new();
    for item in &req.items {
        transfer_repo::insert_transfer_item(
            &tx,
            &NewTransferItem {
                transfer_record_id: record_id,
                product_code: item.product_code.clone(),
                direction: item.direction,
                quantity: item.quantity,
            },
        )?;
        let signed = match item.direction {
            TransferDirection::Out => -item.quantity,
            TransferDirection::In => item.quantity,
        };
        let outcome = apply_stock_change(
            &tx,
            &item.product_code,
            &req.transfer_date,
            signed,
            MovementType::Transfer,
            ReferenceType::TransferRecord,
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
    // 商品コードの昇順・重複なし
    let stock_warnings: Vec<String> = stock_warnings_map.into_values().collect();

    // 9. 操作ログ
    let detail = serde_json::json!({
        "record_type": "transfer_record",
        "record_id": record_id,
        "out_count": out_count,
        "in_count": in_count,
        "warning_count": stock_warnings.len(),
        "idempotency_key": normalized_key,
    });
    system_repo::insert_operation_log(
        &tx,
        &system_repo::NewOperationLog {
            operation_type: "transfer_create".to_string(),
            summary: format!("詰め替え・切り分け記録作成（{}明細）", req.items.len()),
            detail_json: Some(detail.to_string()),
        },
    )?;

    // 10. COMMIT
    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(TransferCreateResult {
        record_id,
        created: true,
        idempotent_replay: false,
        stock_warnings,
    })
}

/// 振替の記録の詳細を返す（movement の source を補う）
///
/// 31-biz-inventory-service.md §12.6a・§12.6b
pub fn get_transfer_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<TransferRecordDetail, BizError> {
    match transfer_repo::get_transfer_record_detail(conn, record_id) {
        Ok(mut detail) => {
            for movement in &mut detail.movements {
                movement.source =
                    resolve_movement_source(&movement.reference_type, &movement.reference_id);
            }
            Ok(detail)
        }
        Err(DbError::NotFound) => Err(BizError::NotFound(format!(
            "詰め替え・切り分けの記録が見つかりません: {}",
            record_id
        ))),
        Err(err) => Err(BizError::DatabaseError(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::biz::inventory_service::list_inventory_records;
    use crate::biz::stocktake_service::{save_stock_count, SaveStockCountRequest};
    use crate::db::disposal_repo::InventoryRecordQuery;
    use crate::db::inventory_repo;
    use rusqlite::types::Value;

    const DATE: &str = "2026-06-10";
    const TABLES: [&str; 4] = [
        "transfer_records",
        "transfer_items",
        "inventory_movements",
        "operation_logs",
    ];

    fn out(code: &str, quantity: i64) -> TransferItemInput {
        TransferItemInput {
            product_code: code.to_string(),
            direction: TransferDirection::Out,
            quantity,
        }
    }

    fn into(code: &str, quantity: i64) -> TransferItemInput {
        TransferItemInput {
            product_code: code.to_string(),
            direction: TransferDirection::In,
            quantity,
        }
    }

    fn req(key: &str, date: &str, items: Vec<TransferItemInput>) -> TransferCreateRequest {
        TransferCreateRequest {
            idempotency_key: key.to_string(),
            transfer_date: date.to_string(),
            items,
        }
    }

    fn movements_of(conn: &DbConnection, record_id: i64) -> Vec<Vec<Value>> {
        crate::db::test_support::dump(
            conn,
            &format!(
                "SELECT product_code, movement_type, quantity, stock_after, reference_type,
                        reference_id, business_date, phase, settlement_no
                 FROM inventory_movements
                 WHERE reference_type = 'transfer_record' AND reference_id = {record_id}
                 ORDER BY id"
            ),
        )
    }

    fn copy_of(conn: &DbConnection, code: &str) -> (i64, bool, Option<i64>, Option<i64>) {
        conn.query_row(
            "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
             FROM products WHERE product_code = ?1",
            [code],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap()
    }

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    fn movement_row(code: &str, quantity: i64, stock_after: i64, record_id: i64) -> Vec<Value> {
        vec![
            text(code),
            text("transfer"),
            Value::Integer(quantity),
            Value::Integer(stock_after),
            text("transfer_record"),
            Value::Integer(record_id),
            text(DATE),
            Value::Integer(0),
            Value::Null,
        ]
    }

    // -----------------------------------------------------------------------
    // B: create_transfer（31 §12.6b）
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_transfer_d143_b1_signs_reference_and_business_date() {
        // D-143 (1)(2) / 31 §12.6b 手順 8 / Test Plan B1
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-B", 0);

        let result = create_transfer(
            &mut conn,
            req("b1", DATE, vec![out("TR-A", 3), into("TR-B", 6)]),
        )
        .unwrap();

        assert!(result.created);
        assert!(!result.idempotent_replay);
        assert!(result.stock_warnings.is_empty());
        assert_eq!(
            movements_of(&conn, result.record_id),
            vec![
                movement_row("TR-A", -3, 7, result.record_id),
                movement_row("TR-B", 6, 6, result.record_id),
            ]
        );
        assert_eq!(stock_of(&conn, "TR-A"), 7);
        assert_eq!(stock_of(&conn, "TR-B"), 6);
    }

    #[test]
    fn test_create_transfer_d143_b2_multiple_lines_each_get_movement_and_item() {
        // D-143 (1) / 31 §12.6b / Test Plan B2: 元 2 行・先 1 行
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A1", 10);
        create_test_product(&conn, "TR-A2", 4);
        create_test_product(&conn, "TR-B", 1);

        let result = create_transfer(
            &mut conn,
            req(
                "b2",
                DATE,
                vec![out("TR-A1", 2), into("TR-B", 5), out("TR-A2", 1)],
            ),
        )
        .unwrap();

        assert_eq!(
            movements_of(&conn, result.record_id),
            vec![
                movement_row("TR-A1", -2, 8, result.record_id),
                movement_row("TR-B", 5, 6, result.record_id),
                movement_row("TR-A2", -1, 3, result.record_id),
            ]
        );
        let items: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM transfer_items WHERE transfer_record_id = ?1",
                [result.record_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(items, 3);
    }

    #[test]
    fn test_create_transfer_d143_b3_requires_both_directions() {
        // 31 §12.6b 手順 1 / Test Plan B3
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-B", 0);
        let both = "減らす商品と増やす商品を1つ以上ずつ入れてください";
        for (key, items, message) in [
            ("b3-out", vec![out("TR-A", 1)], both),
            ("b3-in", vec![into("TR-B", 1)], both),
            ("b3-empty", vec![], "明細が1件以上必要です"),
        ] {
            let before = write_surface(&conn, &TABLES);
            let result = create_transfer(&mut conn, req(key, DATE, items));
            assert_rejected_without_writes(&conn, &TABLES, &before, result, message);
        }
    }

    #[test]
    fn test_create_transfer_d143_b4_rejects_same_product_twice() {
        // 31 §12.6b 手順 5 / Test Plan B4: 向きを問わず同じ商品の 2 回目
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-B", 0);
        for (key, items, message) in [
            (
                "b4-cross",
                vec![out("TR-A", 1), into("TR-A", 1)],
                "明細2: 同じ商品が2回入っています（TR-A）",
            ),
            (
                "b4-same",
                vec![out("TR-A", 1), into("TR-B", 1), into("TR-B", 2)],
                "明細3: 同じ商品が2回入っています（TR-B）",
            ),
        ] {
            let before = write_surface(&conn, &TABLES);
            let result = create_transfer(&mut conn, req(key, DATE, items));
            assert_rejected_without_writes(&conn, &TABLES, &before, result, message);
        }
    }

    #[test]
    fn test_create_transfer_d143_b5_line_quantity_limits_on_both_sides() {
        // 10「入力の上限と安全な整数の範囲」(a) / 31 §12.6b 手順 5 / Test Plan B5
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 0);
        create_test_product(&conn, "TR-B", 0);
        create_transfer(
            &mut conn,
            req(
                "b5-ok",
                DATE,
                vec![out("TR-A", 9_999_999), into("TR-B", 9_999_999)],
            ),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "TR-A"), -9_999_999);
        assert_eq!(stock_of(&conn, "TR-B"), 9_999_999);

        for (quantity, message) in [
            (10_000_000, "数量が大きすぎます"),
            (0, "数量は1以上必要です"),
            (-1, "数量は1以上必要です"),
        ] {
            // 減らす側（明細1）と増やす側（明細2）
            for (key, items, n) in [
                (
                    format!("b5-out-{quantity}"),
                    vec![out("TR-A", quantity), into("TR-B", 1)],
                    1,
                ),
                (
                    format!("b5-in-{quantity}"),
                    vec![out("TR-A", 1), into("TR-B", quantity)],
                    2,
                ),
            ] {
                let before = write_surface(&conn, &TABLES);
                let result = create_transfer(&mut conn, req(&key, DATE, items));
                assert_rejected_without_writes(
                    &conn,
                    &TABLES,
                    &before,
                    result,
                    &format!("明細{n}: {message}"),
                );
            }
        }
    }

    #[test]
    fn test_create_transfer_d143_b6_length_quantity_is_cm_without_conversion() {
        // SPEC-UNIT-D2 / Test Plan B6: 長さの商品の数量は cm のまま（BIZ で換算しない）
        let (_dir, mut conn) = setup_test_db();
        create_test_product_with_unit(&conn, "TR-M", 1000, "m");
        create_test_product(&conn, "TR-B", 0);
        create_transfer(
            &mut conn,
            req("b6", DATE, vec![out("TR-M", 300), into("TR-B", 6)]),
        )
        .unwrap();
        assert_eq!(stock_of(&conn, "TR-M"), 700);
    }

    #[test]
    fn test_create_transfer_d143_b7_business_date_shape_and_calendar() {
        // 31 §12.6b 手順 2 / Test Plan B7
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-B", 0);
        for (date, message) in [
            ("2026-4-7", "日付の形式が不正です（YYYY-MM-DD）"),
            ("2026-02-30", "存在しない日付です"),
        ] {
            let before = write_surface(&conn, &TABLES);
            let result = create_transfer(
                &mut conn,
                req("b7", date, vec![out("TR-A", 1), into("TR-B", 1)]),
            );
            assert_rejected_without_writes(&conn, &TABLES, &before, result, message);
        }
    }

    #[test]
    fn test_create_transfer_d143_b8_missing_product_writes_nothing() {
        // 31 §12.6b 手順 5 / Test Plan B8
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        let before = write_surface(&conn, &TABLES);
        let result = create_transfer(
            &mut conn,
            req("b8", DATE, vec![out("TR-A", 1), into("TR-NONE", 1)]),
        );
        assert!(
            matches!(result, Err(BizError::NotFound(ref m)) if m == "商品が見つかりません: TR-NONE"),
            "{result:?}"
        );
        assert_eq!(write_surface(&conn, &TABLES), before);
    }

    #[test]
    fn test_create_transfer_d143_b9_failure_inside_tx_rolls_back_everything() {
        // 31 §12.6b 手順 8・D-118 / Test Plan B9: 元の movement を先に書いた後、先の在庫が上限を越える
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-FULL", 999_999_999);
        let before = write_surface(&conn, &TABLES);
        let copy_before = copy_of(&conn, "TR-A");
        let result = create_transfer(
            &mut conn,
            req("b9", DATE, vec![out("TR-A", 1), into("TR-FULL", 1)]),
        );
        assert_rejected_without_writes(&conn, &TABLES, &before, result, "在庫数が大きすぎます");
        assert_eq!(copy_of(&conn, "TR-A"), copy_before);
    }

    #[test]
    fn test_create_transfer_d143_b10_idempotency_key_checks() {
        // 31 §12.6b 手順 0 / Test Plan B10
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-B", 0);
        let long = "k".repeat(256);
        for (key, message) in [
            ("", "冪等性キーは必須です"),
            ("   ", "冪等性キーは必須です"),
            (long.as_str(), "冪等性キーは255文字以内です"),
        ] {
            let before = write_surface(&conn, &TABLES);
            let result = create_transfer(
                &mut conn,
                req(key, DATE, vec![out("TR-A", 1), into("TR-B", 1)]),
            );
            assert_rejected_without_writes(&conn, &TABLES, &before, result, message);
        }
    }

    #[test]
    fn test_create_transfer_d143_b11_replay_and_conflict_with_direction_in_fingerprint() {
        // 31 §12.6b 手順 3・4、§12.8 / Test Plan B11
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-B", 10);
        let first = create_transfer(
            &mut conn,
            req("b11", DATE, vec![out("TR-A", 3), into("TR-B", 3)]),
        )
        .unwrap();
        let before = write_surface(&conn, &TABLES);

        // 明細の順を入れ替えた同じ内容は replay
        let replay = create_transfer(
            &mut conn,
            req("b11", DATE, vec![into("TR-B", 3), out("TR-A", 3)]),
        )
        .unwrap();
        assert!(replay.idempotent_replay);
        assert!(!replay.created);
        assert_eq!(replay.record_id, first.record_id);
        assert_eq!(write_surface(&conn, &TABLES), before);

        // 向きだけが違う内容は衝突
        let conflict = create_transfer(
            &mut conn,
            req("b11", DATE, vec![into("TR-A", 3), out("TR-B", 3)]),
        );
        assert!(
            matches!(conflict, Err(BizError::IdempotencyConflict(_))),
            "{conflict:?}"
        );
        assert_eq!(write_surface(&conn, &TABLES), before);
    }

    #[test]
    fn test_create_transfer_d143_b12_warnings_and_operation_log() {
        // 31 §12.6b 手順 8・9 / Test Plan B12
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-Z", 1);
        create_test_product(&conn, "TR-Y", 0);
        create_test_product(&conn, "TR-B", 0);
        let result = create_transfer(
            &mut conn,
            req(
                "b12",
                DATE,
                vec![out("TR-Z", 3), out("TR-Y", 2), into("TR-B", 4)],
            ),
        )
        .unwrap();
        assert_eq!(
            result.stock_warnings,
            vec![
                "TR-Y: 在庫がマイナスになりました（-2）".to_string(),
                "TR-Z: 在庫がマイナスになりました（-2）".to_string(),
            ]
        );
        let logs: Vec<(String, String, String)> = {
            let mut statement = conn
                .prepare("SELECT operation_type, summary, detail_json FROM operation_logs")
                .unwrap();
            let rows = statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            rows
        };
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].0, "transfer_create");
        assert_eq!(logs[0].1, "詰め替え・切り分け記録作成（3明細）");
        let detail: serde_json::Value = serde_json::from_str(&logs[0].2).unwrap();
        assert_eq!(detail["record_type"], "transfer_record");
        assert_eq!(detail["record_id"].as_i64(), Some(result.record_id));
        assert_eq!(detail["out_count"], 2);
        assert_eq!(detail["in_count"], 1);
        assert_eq!(detail["warning_count"], 2);
        assert_eq!(detail["idempotency_key"], "b12");
    }

    #[test]
    fn test_create_transfer_d143_b13_cost_price_and_price_history_unchanged() {
        // D-143 (4) / Test Plan B13: 振替は商品の原価を変えない
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 10);
        create_test_product(&conn, "TR-B", 0);
        conn.execute(
            "UPDATE products SET cost_price_centi = 12345 WHERE product_code = 'TR-B'",
            [],
        )
        .unwrap();
        let costs = |conn: &DbConnection| -> (Vec<Vec<Value>>, i64) {
            (
                crate::db::test_support::dump(
                    conn,
                    "SELECT product_code, cost_price_centi FROM products ORDER BY product_code",
                ),
                conn.query_row("SELECT COUNT(*) FROM price_history", [], |row| row.get(0))
                    .unwrap(),
            )
        };
        let before = costs(&conn);
        create_transfer(
            &mut conn,
            req("b13", DATE, vec![out("TR-A", 2), into("TR-B", 4)]),
        )
        .unwrap();
        assert_eq!(costs(&conn), before);
    }

    // -----------------------------------------------------------------------
    // L: 在庫の求め直し（D-118、ADR SPEC-STK-CNT-D3・D9）
    // -----------------------------------------------------------------------

    fn count(
        conn: &mut DbConnection,
        code: &str,
        value: i64,
        date: &str,
        fresh: bool,
        today: &str,
    ) {
        save_stock_count(
            conn,
            &SaveStockCountRequest {
                product_code: code.to_string(),
                count_value: value,
                fresh,
                business_date: date.to_string(),
                stocktake_item_id: None,
            },
            today,
        )
        .unwrap();
    }

    #[test]
    fn test_create_transfer_d143_l1_same_date_default_count_needs_recheck() {
        // ADR D3 の理由 2 / Test Plan L1: 同じ日付の既定の計数がある減らす商品は要再確認（範囲 4〜5）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-A", 0);
        create_test_product(&conn, "TR-B", 0);
        count(&mut conn, "TR-B", 5, DATE, false, DATE);
        create_transfer(
            &mut conn,
            req("l1", DATE, vec![out("TR-B", 1), into("TR-A", 1)]),
        )
        .unwrap();
        let (_, recheck, low, high) = copy_of(&conn, "TR-B");
        assert!(recheck);
        assert_eq!((low, high), (Some(4), Some(5)));
        let (stock, recheck, low, high) = copy_of(&conn, "TR-A");
        assert_eq!((stock, recheck, low, high), (1, false, None, None));
    }

    #[test]
    fn test_create_transfer_d143_l2_before_count_included_after_count_not() {
        // ADR D3 の判定表（入出庫の行）/ Test Plan L2: 業務日で判定する（保存の時刻でない）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-P1", 0);
        create_test_product(&conn, "TR-P2", 0);
        create_test_product(&conn, "TR-Q", 0);
        let d = DATE;
        count(&mut conn, "TR-P1", 10, d, false, "2026-06-11");
        count(&mut conn, "TR-P2", 10, d, false, "2026-06-11");
        // 計数の保存の後に、計数より前の日付（D-1）と後の日付（D+1）の振替を保存する
        create_transfer(
            &mut conn,
            req(
                "l2-before",
                "2026-06-09",
                vec![out("TR-P1", 2), into("TR-Q", 2)],
            ),
        )
        .unwrap();
        create_transfer(
            &mut conn,
            req(
                "l2-after",
                "2026-06-11",
                vec![out("TR-P2", 2), into("TR-Q", 2)],
            ),
        )
        .unwrap();
        assert_eq!(copy_of(&conn, "TR-P1"), (10, false, None, None));
        assert_eq!(copy_of(&conn, "TR-P2"), (8, false, None, None));
    }

    #[test]
    fn test_create_transfer_d143_l3_fresh_count_after_transfer_includes_it() {
        // ADR D3（fresh な計数）/ Test Plan L3: 振替の後に入れ直すと、同じ日付の振替は入る
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-P", 10);
        create_test_product(&conn, "TR-Q", 0);
        create_transfer(
            &mut conn,
            req("l3", DATE, vec![out("TR-P", 1), into("TR-Q", 1)]),
        )
        .unwrap();
        count(&mut conn, "TR-P", 7, DATE, true, DATE);
        assert_eq!(copy_of(&conn, "TR-P"), (7, false, None, None));
    }

    #[test]
    fn test_create_transfer_d143_l4_transfer_movement_is_not_initial_stock() {
        // ADR D9 / Test Plan L4: 振替の movement は初期在庫の印に当たらない
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-P", 0);
        create_test_product(&conn, "TR-Q", 0);
        create_transfer(
            &mut conn,
            req("l4", DATE, vec![out("TR-P", 1), into("TR-Q", 1)]),
        )
        .unwrap();
        for code in ["TR-P", "TR-Q"] {
            let rows = inventory_repo::list_ledger_movements_for_product(&conn, code).unwrap();
            assert_eq!(rows.len(), 1, "{code}");
            assert!(!rows[0].initial_stock, "{code}");
        }
    }

    // -----------------------------------------------------------------------
    // R: 記録の追跡（21 §10.5a、44 §23.7、65 TRACE-D14）
    // -----------------------------------------------------------------------

    fn records_query(record_type: &str) -> InventoryRecordQuery {
        InventoryRecordQuery {
            record_type: Some(record_type.to_string()),
            date_from: None,
            date_to: None,
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: None,
            page: 1,
            per_page: 50,
        }
    }

    fn seed_transfer_and_disposal(conn: &mut DbConnection) -> i64 {
        create_test_product(conn, "TR-A", 10);
        create_test_product(conn, "TR-B", 0);
        create_test_product(conn, "TR-C", 0);
        super::super::disposal::create_disposal(
            conn,
            super::super::disposal::DisposalCreateRequest {
                idempotency_key: "r-dsp".to_string(),
                disposal_date: DATE.to_string(),
                items: vec![super::super::disposal::DisposalItemInput {
                    product_code: "TR-A".to_string(),
                    disposal_type: crate::db::disposal_repo::DisposalType::Damage,
                    quantity: 1,
                    cost_price_centi: 100,
                    reason: "合成".to_string(),
                }],
            },
        )
        .unwrap();
        create_transfer(
            conn,
            req(
                "r-tr",
                DATE,
                vec![out("TR-A", 2), into("TR-B", 3), into("TR-C", 1)],
            ),
        )
        .unwrap()
        .record_id
    }

    #[test]
    fn test_list_inventory_records_d143_r1_transfer_record_type() {
        // 21 §10.5a・44 §23.7 / Test Plan R1
        let (_dir, mut conn) = setup_test_db();
        let record_id = seed_transfer_and_disposal(&mut conn);

        let only = list_inventory_records(&conn, &records_query("transfer_record")).unwrap();
        assert_eq!(only.total_count, 1);
        let row = &only.items[0];
        assert_eq!(row.record_type, "transfer_record");
        assert_eq!(row.record_id, record_id);
        assert_eq!(row.business_date, DATE);
        assert_eq!(row.item_count, 3);
        assert_eq!(row.status, "active");
        assert_eq!(
            row.detail_route,
            format!("/inventory/transfer/records/{record_id}")
        );

        let all = list_inventory_records(&conn, &records_query("all")).unwrap();
        let mut types: Vec<&str> = all
            .items
            .iter()
            .map(|row| row.record_type.as_str())
            .collect();
        types.sort();
        assert_eq!(types, vec!["disposal_record", "transfer_record"]);
    }

    #[test]
    fn test_list_inventory_records_d143_r2_filters_hit_increase_side() {
        // 65 TRACE-D14 / Test Plan R2: 増やす側の商品の絞り込みでも当たる
        let (_dir, mut conn) = setup_test_db();
        let record_id = seed_transfer_and_disposal(&mut conn);
        conn.execute(
            "UPDATE products SET department_id = 2 WHERE product_code = 'TR-C'",
            [],
        )
        .unwrap();

        let mut keyword = records_query("transfer_record");
        keyword.product_keyword = Some("TR-C".to_string());
        let hit = list_inventory_records(&conn, &keyword).unwrap();
        assert_eq!(
            hit.items.iter().map(|r| r.record_id).collect::<Vec<_>>(),
            vec![record_id]
        );

        let mut department = records_query("all");
        department.department_id = Some(2);
        let hit = list_inventory_records(&conn, &department).unwrap();
        assert_eq!(
            hit.items
                .iter()
                .map(|r| (r.record_type.as_str(), r.record_id))
                .collect::<Vec<_>>(),
            vec![("transfer_record", record_id)]
        );
    }

    #[test]
    fn test_get_transfer_record_d143_r3_detail_with_sources() {
        // 21 §10.5a / Test Plan R3: 減らす明細を先、明細の id の順、単位つき、関連 movement に source
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "TR-B", 0);
        create_test_product_with_unit(&conn, "TR-M", 1000, "m");
        let record_id = create_transfer(
            &mut conn,
            req("r3", DATE, vec![into("TR-B", 6), out("TR-M", 300)]),
        )
        .unwrap()
        .record_id;

        let detail = get_transfer_record(&conn, record_id).unwrap();

        assert_eq!(detail.id, record_id);
        assert_eq!(detail.transfer_date, DATE);
        assert_eq!(detail.status, "active");
        let items: Vec<(
            &str,
            TransferDirection,
            i64,
            crate::db::product_repo::ProductStockUnit,
        )> = detail
            .items
            .iter()
            .map(|i| {
                (
                    i.product_code.as_str(),
                    i.direction,
                    i.quantity,
                    i.stock_unit,
                )
            })
            .collect();
        assert_eq!(
            items,
            vec![
                (
                    "TR-M",
                    TransferDirection::Out,
                    300,
                    crate::db::product_repo::ProductStockUnit::M
                ),
                (
                    "TR-B",
                    TransferDirection::In,
                    6,
                    crate::db::product_repo::ProductStockUnit::Pcs
                ),
            ]
        );
        assert_eq!(detail.movements.len(), 2);
        for movement in &detail.movements {
            let source = movement.source.as_ref().expect("source を補う");
            assert_eq!(source.label, format!("詰め替え・切り分け #{record_id}"));
            assert_eq!(
                source.route,
                format!("/inventory/transfer/records/{record_id}")
            );
        }
    }

    #[test]
    fn test_get_transfer_record_d143_r4_not_found_and_unknown_type() {
        // 21 §10.5a・44 §23.7 / Test Plan R4
        let (_dir, conn) = setup_test_db();
        let missing = get_transfer_record(&conn, 404);
        assert!(
            matches!(missing, Err(BizError::NotFound(ref m)) if m == "詰め替え・切り分けの記録が見つかりません: 404"),
            "{missing:?}"
        );
        let unknown = list_inventory_records(&conn, &records_query("unknown_type"));
        assert!(
            matches!(unknown, Err(BizError::ValidationFailed(ref m)) if m == "未対応の記録種別です"),
            "{unknown:?}"
        );
    }

    #[test]
    fn test_resolve_movement_source_d143_r5_transfer_record_link() {
        // 65 TRACE-D2・D14 / Test Plan R5
        let source =
            resolve_movement_source(&Some(ReferenceType::TransferRecord), &Some(7)).unwrap();
        assert_eq!(source.label, "詰め替え・切り分け #7");
        assert_eq!(source.route, "/inventory/transfer/records/7");
    }
}
