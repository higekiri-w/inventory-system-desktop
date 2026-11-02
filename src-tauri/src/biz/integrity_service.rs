//! BIZ-07: 整合性チェックロジック
//!
//! 商品の写し（`stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`）と、
//! 記録から求め直した値（`biz::stock_ledger::derive_stock`）を突合し、違う商品を補正する。
//! 写しは在庫を動かす全ての TX で求め直すので、違いは不具合か DB の外からの編集でだけ起きる。
//!
//! docs/function-design/36-biz-integrity-check.md と「計数の置き場と在庫の求め直しの契約」に基づく実装。

use crate::biz::stock_ledger::{derive_stock, recompute_stock, DerivedStock};
use crate::biz::BizError;
use crate::db::product_repo::{ProductStockUnit, StockCopyRow};
use crate::db::{product_repo, system_repo, DbConnection, NewOperationLog};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 整合性チェック結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct IntegrityResult {
    pub mismatches: Vec<IntegrityMismatch>,
    pub mismatch_count: usize,
    pub checked_count: usize,
    /// 在庫連動の商品と同じ JAN を別の商品が持つ組（移行の前からの違反。補正の対象にしない。36、ADR D10）
    pub shared_jan_conflicts: Vec<SharedJanConflict>,
}

/// 共有 JAN の違反の組（JAN と、その JAN を持つ全ての商品コード〈昇順〉。36）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct SharedJanConflict {
    pub jan_code: String,
    pub product_codes: Vec<String>,
}

/// 写しの 4 項目（保存している値、または記録から求め直した値。36）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct StockCopy {
    pub stock_quantity: i64,
    pub stock_recheck: bool,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
}

/// 写しと求めた値で違う項目（重複なし、この順。36）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityField {
    Quantity,
    Recheck,
    Range,
}

/// 不整合アイテム
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct IntegrityMismatch {
    pub product_code: String,
    pub name: String,
    /// 商品の単位。数量は在庫の数量（長さは cm）で、表示は単位で決める（SPEC-UNIT-D3）
    pub stock_unit: ProductStockUnit,
    /// 保存している値
    pub copy: StockCopy,
    /// 記録から求め直した値
    pub derived: StockCopy,
    pub differing_fields: Vec<IntegrityField>,
}

/// 整合性補正結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct IntegrityFixResult {
    pub fixed_count: usize,
    pub skipped_count: usize,
    pub adjustments: Vec<StockAdjustment>,
}

/// 在庫補正詳細（差は持たず、画面は before → after を並べる。75 §75.6）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StockAdjustment {
    pub product_code: String,
    /// 商品の単位（SPEC-UNIT-D3 の表の 11）
    pub stock_unit: ProductStockUnit,
    pub before: StockCopy,
    pub after: StockCopy,
}

impl From<&StockCopyRow> for StockCopy {
    fn from(row: &StockCopyRow) -> Self {
        StockCopy {
            stock_quantity: row.stock_quantity,
            stock_recheck: row.stock_recheck,
            stock_range_low: row.stock_range_low,
            stock_range_high: row.stock_range_high,
        }
    }
}

impl From<&DerivedStock> for StockCopy {
    fn from(derived: &DerivedStock) -> Self {
        StockCopy {
            stock_quantity: derived.stock,
            stock_recheck: derived.range.is_some(),
            stock_range_low: derived.range.map(|r| r.0),
            stock_range_high: derived.range.map(|r| r.1),
        }
    }
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

fn differing_fields(copy: &StockCopy, derived: &StockCopy) -> Vec<IntegrityField> {
    [
        (
            IntegrityField::Quantity,
            copy.stock_quantity != derived.stock_quantity,
        ),
        (
            IntegrityField::Recheck,
            copy.stock_recheck != derived.stock_recheck,
        ),
        (
            IntegrityField::Range,
            (copy.stock_range_low, copy.stock_range_high)
                != (derived.stock_range_low, derived.stock_range_high),
        ),
    ]
    .into_iter()
    .filter_map(|(field, differs)| differs.then_some(field))
    .collect()
}

/// 全商品の整合性をチェックする（読み取り専用、TX不要）
///
/// 36-biz-integrity-check.md §21.3
pub fn run_integrity_check(conn: &DbConnection) -> Result<IntegrityResult, BizError> {
    // 1. 全商品の写しを取得し、記録から求め直した値と比べる（読取りは範囲で拒まない）
    let products = product_repo::find_all_stock_quantities(conn)?;
    let checked_count = products.len();
    let mut mismatches = Vec::new();
    for row in &products {
        let copy = StockCopy::from(row);
        let derived = StockCopy::from(&derive_stock(conn, &row.product_code)?);
        let differing_fields = differing_fields(&copy, &derived);
        if !differing_fields.is_empty() {
            mismatches.push(IntegrityMismatch {
                product_code: row.product_code.clone(),
                name: row.name.clone(),
                stock_unit: row.stock_unit,
                copy,
                derived,
                differing_fields,
            });
        }
    }

    let mismatch_count = mismatches.len();

    // 2. TX外: 操作ログ記録（各 mismatch は応答と同じ key と入れ子。36）
    let detail = serde_json::json!({
        "checked_count": checked_count,
        "mismatch_count": mismatch_count,
        "mismatches": mismatches.iter().map(|m| {
            serde_json::json!({
                "product_code": m.product_code,
                "stock_unit": m.stock_unit,
                "copy": m.copy,
                "derived": m.derived,
                "differing_fields": m.differing_fields,
            })
        }).collect::<Vec<_>>(),
    })
    .to_string();
    let log = NewOperationLog {
        operation_type: "integrity_check".to_string(),
        summary: format!(
            "整合性チェック実行: {}件中{}件の不整合",
            checked_count, mismatch_count
        ),
        detail_json: Some(detail),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }

    // 共有 JAN の違反の一覧（写しの比較の結果とは別。操作ログには記録しない。36）
    let shared_jan_conflicts = product_repo::list_shared_jan_conflicts(conn)?
        .into_iter()
        .map(|(jan_code, product_codes)| SharedJanConflict {
            jan_code,
            product_codes,
        })
        .collect();

    Ok(IntegrityResult {
        mismatches,
        mismatch_count,
        checked_count,
        shared_jan_conflicts,
    })
}

/// 指定商品の写しを記録から求め直した値へ補正する（TX。movement は作らない）
///
/// 36-biz-integrity-check.md §21.4
pub fn fix_integrity(
    conn: &mut DbConnection,
    product_codes: &[String],
) -> Result<IntegrityFixResult, BizError> {
    use crate::db::DbError;

    // 1. 入力バリデーション
    if product_codes.is_empty() {
        return Err(BizError::ValidationFailed(
            "補正対象の商品が指定されていません".to_string(),
        ));
    }

    // 2. TX開始
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    // 3. 各商品について、写しと求めた値が違えば recompute_stock の結果を書く。
    //    写しの行は 1 回だけ消費し、同じ商品の 2 回目以降は skip に数える
    let mut copies: HashMap<String, StockCopyRow> = product_repo::find_all_stock_quantities(&tx)?
        .into_iter()
        .map(|row| (row.product_code.clone(), row))
        .collect();
    let mut skipped_count = 0usize;
    let mut adjustments = Vec::new();
    for product_code in product_codes {
        let Some(row) = copies.remove(product_code) else {
            skipped_count += 1;
            continue;
        };
        let before = StockCopy::from(&row);
        if before == StockCopy::from(&derive_stock(&tx, product_code)?) {
            skipped_count += 1;
            continue;
        }
        // D-051 / BIZ-07-D2: movement を追加しない。上限の外は TX 全体を戻す（recompute_stock）
        let after = StockCopy::from(&recompute_stock(&tx, product_code)?);
        adjustments.push(StockAdjustment {
            product_code: product_code.clone(),
            stock_unit: row.stock_unit,
            before,
            after,
        });
    }
    let fixed_count = adjustments.len();

    // 4. TX内: 操作ログ記録（BIZ-07-D3、失敗時は補正ごと rollback。trigger は書かない）
    let detail = serde_json::json!({
        "fixed_count": fixed_count,
        "skipped_count": skipped_count,
        "adjustments": adjustments.iter().map(|a| {
            serde_json::json!({
                "product_code": a.product_code,
                "stock_unit": a.stock_unit,
                "before": a.before,
                "after": a.after,
            })
        }).collect::<Vec<_>>(),
    })
    .to_string();
    let log = NewOperationLog {
        operation_type: "integrity_fix".to_string(),
        summary: format!("{}件の在庫を補正しました", fixed_count),
        detail_json: Some(detail),
    };
    system_repo::insert_operation_log(&tx, &log)?;

    // 5. COMMIT
    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(IntegrityFixResult {
        fixed_count,
        skipped_count,
        adjustments,
    })
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_database;
    use crate::db::inventory_repo::{self, MovementType, NewMovement};
    use crate::db::product_repo::{self, NewProduct};

    fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    fn seed_product(conn: &DbConnection, product_code: &str, stock_quantity: i64) {
        let product = NewProduct {
            product_code: product_code.to_string(),
            jan_code: None,
            name: format!("商品{}", product_code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
    }

    fn add_movement(conn: &DbConnection, product_code: &str, quantity: i64) {
        inventory_repo::insert_movement(
            conn,
            &NewMovement {
                product_code: product_code.to_string(),
                movement_type: MovementType::Receiving,
                quantity,
                stock_after: quantity,
                reference_type: None,
                reference_id: None,
                note: None,
                business_date: "2026-04-01".to_string(),
                settlement_no: None,
            },
        )
        .unwrap();
    }

    fn add_voided_movement(conn: &DbConnection, product_code: &str, quantity: i64) {
        let movement_id = inventory_repo::insert_movement(
            conn,
            &NewMovement {
                product_code: product_code.to_string(),
                movement_type: MovementType::Receiving,
                quantity,
                stock_after: quantity,
                reference_type: None,
                reference_id: None,
                note: None,
                business_date: "2026-04-01".to_string(),
                settlement_no: None,
            },
        )
        .unwrap();
        conn.execute(
            "UPDATE inventory_movements SET is_voided = 1 WHERE id = ?1",
            [movement_id],
        )
        .unwrap();
    }

    fn stock_quantity(conn: &DbConnection, product_code: &str) -> i64 {
        conn.query_row(
            "SELECT stock_quantity FROM products WHERE product_code = ?1",
            [product_code],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn movement_count(conn: &DbConnection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    #[derive(Debug, PartialEq, Eq)]
    struct MovementSnapshot {
        id: i64,
        product_code: String,
        movement_type: String,
        quantity: i64,
        stock_after: i64,
        reference_type: Option<String>,
        reference_id: Option<i64>,
        note: Option<String>,
        is_voided: bool,
        created_at: String,
    }

    fn movement_snapshot(conn: &DbConnection) -> Vec<MovementSnapshot> {
        let mut stmt = conn
            .prepare(
                "SELECT id, product_code, movement_type, quantity, stock_after,
                        reference_type, reference_id, note, is_voided, created_at
                 FROM inventory_movements ORDER BY id",
            )
            .unwrap();
        stmt.query_map([], |row| {
            Ok(MovementSnapshot {
                id: row.get(0)?,
                product_code: row.get(1)?,
                movement_type: row.get(2)?,
                quantity: row.get(3)?,
                stock_after: row.get(4)?,
                reference_type: row.get(5)?,
                reference_id: row.get(6)?,
                note: row.get(7)?,
                is_voided: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    }

    fn product_snapshot(
        conn: &DbConnection,
        product_code: &str,
    ) -> (serde_json::Value, i64, String) {
        let product = product_repo::find_by_product_code(conn, product_code)
            .unwrap()
            .unwrap()
            .product;
        let stock_quantity = product.stock_quantity;
        let updated_at = product.updated_at.clone();
        let mut invariant_columns = serde_json::to_value(product).unwrap();
        let invariant_columns = invariant_columns.as_object_mut().unwrap();
        invariant_columns.remove("stock_quantity");
        invariant_columns.remove("updated_at");
        (
            serde_json::Value::Object(invariant_columns.clone()),
            stock_quantity,
            updated_at,
        )
    }

    /// 印 0・範囲なしの写し
    fn plain(stock_quantity: i64) -> StockCopy {
        StockCopy {
            stock_quantity,
            stock_recheck: false,
            stock_range_low: None,
            stock_range_high: None,
        }
    }

    fn plain_json(stock_quantity: i64) -> serde_json::Value {
        serde_json::json!({
            "stock_quantity": stock_quantity,
            "stock_recheck": false,
            "stock_range_low": null,
            "stock_range_high": null,
        })
    }

    // ===== run_integrity_check テスト =====

    #[test]
    fn test_run_integrity_check_req904_all_consistent() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.3: 全件一致 → mismatches空
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "IC-001", 10);
        add_movement(&conn, "IC-001", 10);

        let result = run_integrity_check(&conn).unwrap();
        assert_eq!(result.checked_count, 1);
        assert_eq!(result.mismatch_count, 0);
        assert!(result.mismatches.is_empty());
    }

    #[test]
    fn test_run_integrity_check_req904_mismatch_detected() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.3: 差異あり → mismatches返却
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "IM-001", 10); // stock=10
        add_movement(&conn, "IM-001", 7); // movements=7 → 差異3

        let result = run_integrity_check(&conn).unwrap();
        assert_eq!(result.mismatch_count, 1);
        let m = &result.mismatches[0];
        assert_eq!(m.product_code, "IM-001");
        assert_eq!(m.copy, plain(10));
        assert_eq!(m.derived, plain(7));
        assert_eq!(m.differing_fields, vec![IntegrityField::Quantity]);
    }

    #[test]
    fn test_run_integrity_check_req904_no_movements_product() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.3: movements 0件 → 記録から求め直した値 0 として扱い
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "NM-001", 5); // stock=5, movements=0

        let result = run_integrity_check(&conn).unwrap();
        assert_eq!(result.mismatch_count, 1);
        assert_eq!(result.mismatches[0].copy, plain(5));
        assert_eq!(result.mismatches[0].derived, plain(0));
    }

    #[test]
    fn test_run_integrity_check_req904_operation_log() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.3: 操作ログ記録確認
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "OL-001", 10);
        add_movement(&conn, "OL-001", 10);

        run_integrity_check(&conn).unwrap();

        let op_type: String = conn
            .query_row(
                "SELECT operation_type FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(op_type, "integrity_check");
    }

    #[test]
    fn test_run_integrity_check_req904_empty_db() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.3: 商品0件 → checked_count=0
        let (_dir, conn) = setup_test_db();
        let result = run_integrity_check(&conn).unwrap();
        assert_eq!(result.checked_count, 0);
        assert_eq!(result.mismatch_count, 0);
    }

    // ===== fix_integrity テスト =====

    #[test]
    fn test_fix_integrity_req904_t1_log_insert_failure() {
        // REQ-904 / BIZ-07-D3 / T1: 操作ログ失敗時は全補正をロールバックする
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "T1-A", 10);
        seed_product(&conn, "T1-B", 3);
        add_movement(&conn, "T1-A", 7);
        add_movement(&conn, "T1-B", 8);
        let stocks_before = [stock_quantity(&conn, "T1-A"), stock_quantity(&conn, "T1-B")];
        let movements_before = movement_count(&conn);
        conn.execute_batch(
            "CREATE TRIGGER fail_integrity_fix_log
             BEFORE INSERT ON operation_logs
             WHEN NEW.operation_type = 'integrity_fix'
             BEGIN
                 SELECT RAISE(ABORT, 'synthetic integrity_fix log failure');
             END;",
        )
        .unwrap();

        let result = fix_integrity(&mut conn, &["T1-A".to_string(), "T1-B".to_string()]);

        assert!(matches!(result, Err(BizError::DatabaseError(_))));
        assert_eq!(
            [stock_quantity(&conn, "T1-A"), stock_quantity(&conn, "T1-B"),],
            stocks_before
        );
        assert_eq!(movement_count(&conn), movements_before);
    }

    #[test]
    fn test_fix_integrity_req904_t2_audit_detail_and_skips() {
        // REQ-904 / BIZ-07-D3 / INV-4 / T2: 具体値の監査ログと skipped 契約
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "T2-DOWN", 10);
        seed_product(&conn, "T2-UP", 3);
        seed_product(&conn, "T2-VOID", 6);
        seed_product(&conn, "T2-ZERO", 5);
        seed_product(&conn, "T2-SAME", 2);
        add_movement(&conn, "T2-DOWN", 7);
        add_movement(&conn, "T2-UP", 8);
        add_movement(&conn, "T2-VOID", 4);
        add_voided_movement(&conn, "T2-VOID", 100);
        add_movement(&conn, "T2-SAME", 2);

        let result = fix_integrity(
            &mut conn,
            &[
                "T2-DOWN".to_string(),
                "T2-UP".to_string(),
                "T2-VOID".to_string(),
                "T2-ZERO".to_string(),
                "T2-SAME".to_string(),
                "T2-MISSING".to_string(),
            ],
        )
        .unwrap();

        assert_eq!(result.fixed_count, 4);
        assert_eq!(result.skipped_count, 2);
        let (log_count, detail_json): (i64, String) = conn
            .query_row(
                "SELECT COUNT(*), detail_json FROM operation_logs
                 WHERE operation_type = 'integrity_fix'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(log_count, 1);
        let detail: serde_json::Value = serde_json::from_str(&detail_json).unwrap();
        assert_eq!(detail["fixed_count"], serde_json::json!(4));
        assert_eq!(detail["skipped_count"], serde_json::json!(2));
        assert_eq!(
            detail["adjustments"],
            serde_json::json!([
                {"product_code": "T2-DOWN", "stock_unit": "pcs", "before": plain_json(10), "after": plain_json(7)},
                {"product_code": "T2-UP", "stock_unit": "pcs", "before": plain_json(3), "after": plain_json(8)},
                {"product_code": "T2-VOID", "stock_unit": "pcs", "before": plain_json(6), "after": plain_json(4)},
                {"product_code": "T2-ZERO", "stock_unit": "pcs", "before": plain_json(5), "after": plain_json(0)}
            ])
        );
    }

    #[test]
    fn test_fix_integrity_req904_t3_movements_and_products_unchanged() {
        // REQ-904 / BIZ-07-D2 / INV-8 / T3: movement 原本と product 行を破壊しない
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "T3-A", 10);
        seed_product(&conn, "T3-B", 3);
        add_movement(&conn, "T3-A", 7);
        add_movement(&conn, "T3-B", 8);
        add_voided_movement(&conn, "T3-B", 20);
        let movements_before = movement_snapshot(&conn);
        let movement_count_before = movement_count(&conn);
        let products_before = [
            ("T3-A", product_snapshot(&conn, "T3-A"), 7),
            ("T3-B", product_snapshot(&conn, "T3-B"), 8),
        ];

        fix_integrity(&mut conn, &["T3-A".to_string(), "T3-B".to_string()]).unwrap();

        assert_eq!(movement_count(&conn), movement_count_before);
        assert_eq!(movement_snapshot(&conn), movements_before);
        for (product_code, (invariant_before, stock_before, updated_at_before), expected_stock) in
            products_before
        {
            let per_product_before = movements_before
                .iter()
                .filter(|movement| movement.product_code == product_code)
                .count() as i64;
            let per_product_after: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM inventory_movements WHERE product_code = ?1",
                    [product_code],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(per_product_after, per_product_before);

            let (invariant_after, stock_after, updated_at_after) =
                product_snapshot(&conn, product_code);
            assert_eq!(stock_after, expected_stock, "{product_code}");
            assert_ne!(stock_after, stock_before, "{product_code}");
            assert!(updated_at_after >= updated_at_before, "{product_code}");
            assert_eq!(invariant_after, invariant_before, "{product_code}");
        }
    }

    #[test]
    fn test_fix_integrity_req904_t4_recheck_converges() {
        // REQ-904 / BIZ-07-D4 / T4: 成功直後は補正対象が mismatch に現れない
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "T4-A", 10);
        seed_product(&conn, "T4-B", 3);
        add_movement(&conn, "T4-A", 7);
        add_movement(&conn, "T4-B", 8);

        let fixed = fix_integrity(&mut conn, &["T4-A".to_string(), "T4-B".to_string()]).unwrap();
        let rechecked = run_integrity_check(&conn).unwrap();

        for adjustment in fixed.adjustments {
            assert!(rechecked
                .mismatches
                .iter()
                .all(|mismatch| mismatch.product_code != adjustment.product_code));
        }
    }

    #[test]
    fn test_fix_integrity_req904_t5_stock_equals_non_voided_sum() {
        // REQ-904 / BIZ-07-D1 / INV-4 / T5: DB の派生関係を SQL で直接検査する
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "T5-MOVED", 10);
        seed_product(&conn, "T5-ZERO", 5);
        add_movement(&conn, "T5-MOVED", 7);
        add_voided_movement(&conn, "T5-MOVED", 100);

        fix_integrity(&mut conn, &["T5-MOVED".to_string(), "T5-ZERO".to_string()]).unwrap();

        for product_code in ["T5-MOVED", "T5-ZERO"] {
            let (stock, non_voided_sum): (i64, i64) = conn
                .query_row(
                    "SELECT p.stock_quantity,
                            COALESCE((SELECT SUM(im.quantity)
                                      FROM inventory_movements im
                                      WHERE im.product_code = p.product_code
                                        AND im.is_voided = 0), 0)
                     FROM products p WHERE p.product_code = ?1",
                    [product_code],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(stock, non_voided_sum, "{product_code}");
        }
    }

    #[test]
    fn test_fix_integrity_req904_normal() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4: 差異あり → 補正実行
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "FI-001", 10);
        add_movement(&conn, "FI-001", 7); // movements=7, stock=10

        let result = fix_integrity(&mut conn, &["FI-001".to_string()]).unwrap();
        assert_eq!(result.fixed_count, 1);
        assert_eq!(result.adjustments[0].before, plain(10));
        assert_eq!(result.adjustments[0].after, plain(7));

        // DB検証: stock_quantity = 有効な movement の和
        let stock: i64 = conn
            .query_row(
                "SELECT stock_quantity FROM products WHERE product_code = 'FI-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stock, 7);
    }

    #[test]
    fn test_fix_integrity_req904_empty_codes() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4: 空配列 → ValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let result = fix_integrity(&mut conn, &[]);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_fix_integrity_req904_product_not_found() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4: 存在しない商品 → スキップ
        let (_dir, mut conn) = setup_test_db();
        let result = fix_integrity(&mut conn, &["NONEXISTENT".to_string()]).unwrap();
        assert_eq!(result.fixed_count, 0);
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn test_fix_integrity_req904_no_difference() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4: 差異なし → スキップ
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "FN-001", 10);
        add_movement(&conn, "FN-001", 10);

        let result = fix_integrity(&mut conn, &["FN-001".to_string()]).unwrap();
        assert_eq!(result.fixed_count, 0);
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn test_fix_integrity_req904_recheck_consistent() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4 P1修正: 補正後に再チェックで不整合0件
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "FM-001", 10);
        add_movement(&conn, "FM-001", 7);

        // 1回目: 不整合検出
        let check1 = run_integrity_check(&conn).unwrap();
        assert_eq!(check1.mismatch_count, 1);

        // 補正実行
        fix_integrity(&mut conn, &["FM-001".to_string()]).unwrap();

        // 2回目: 不整合なし
        let check2 = run_integrity_check(&conn).unwrap();
        assert_eq!(check2.mismatch_count, 0, "補正後は不整合0件であるべき");
    }

    #[test]
    fn test_fix_integrity_req904_multiple_products() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4: 複数商品の一括補正
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "FMU-001", 10);
        seed_product(&conn, "FMU-002", 5);
        add_movement(&conn, "FMU-001", 8);
        add_movement(&conn, "FMU-002", 3);

        let result =
            fix_integrity(&mut conn, &["FMU-001".to_string(), "FMU-002".to_string()]).unwrap();
        assert_eq!(result.fixed_count, 2);
        assert_eq!(result.adjustments.len(), 2);
    }

    #[test]
    fn test_fix_integrity_req904_operation_log() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4: 操作ログ記録確認
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "FOL-001", 10);
        add_movement(&conn, "FOL-001", 7);

        fix_integrity(&mut conn, &["FOL-001".to_string()]).unwrap();

        let op_type: String = conn
            .query_row(
                "SELECT operation_type FROM operation_logs
                 WHERE operation_type = 'integrity_fix' ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(op_type, "integrity_fix");
    }

    #[test]
    fn test_fix_integrity_req904_negative_ledger_sum() {
        // REQ-904: 整合性チェック（在庫数突合/修復）
        // BIZ-07 §21.4 P3-2: 負の記録の和 → stock_quantityを負に補正（INV-3準拠）
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "FNG-001", 5);
        // movements: +10, -13 = -3
        add_movement(&conn, "FNG-001", 10);
        inventory_repo::insert_movement(
            &conn,
            &NewMovement {
                product_code: "FNG-001".to_string(),
                movement_type: MovementType::SaleAuto,
                quantity: -13,
                stock_after: -3,
                reference_type: None,
                reference_id: None,
                note: None,
                business_date: "2026-04-01".to_string(),
                settlement_no: None,
            },
        )
        .unwrap();

        let result = fix_integrity(&mut conn, &["FNG-001".to_string()]).unwrap();
        assert_eq!(result.fixed_count, 1);
        assert_eq!(result.adjustments[0].after, plain(-3));

        let stock: i64 = conn
            .query_row(
                "SELECT stock_quantity FROM products WHERE product_code = 'FNG-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stock, -3, "負の記録の和に合わせる");
    }

    #[test]
    fn test_fix_integrity_req904_stock_limit_rolls_back_whole_tx() {
        // REQ-904 / 共通規則 (a)（X3・X10）: 補正の後の在庫が上限の外なら TX 全体を戻す。読取りは拒まない
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "INT-OVER", 5);
        add_movement(&conn, "INT-OVER", 1_000_000_000);
        seed_product(&conn, "INT-EDGE", 5);
        add_movement(&conn, "INT-EDGE", 999_999_999);
        seed_product(&conn, "INT-OTHER", 5);
        add_movement(&conn, "INT-OTHER", 8);
        let log_count = |conn: &DbConnection| -> i64 {
            conn.query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'integrity_fix'",
                [],
                |row| row.get(0),
            )
            .unwrap()
        };
        let stock = |conn: &DbConnection, code: &str| -> i64 {
            product_repo::find_by_product_code(conn, code)
                .unwrap()
                .unwrap()
                .product
                .stock_quantity
        };

        // 読取りは範囲で拒まず、不一致を返す
        let check = run_integrity_check(&conn).unwrap();
        assert_eq!(check.mismatch_count, 3);

        let result = fix_integrity(
            &mut conn,
            &["INT-OTHER".to_string(), "INT-OVER".to_string()],
        );
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref m)) if m == "在庫数が大きすぎます"),
            "{result:?}"
        );
        assert_eq!(stock(&conn, "INT-OVER"), 5);
        assert_eq!(stock(&conn, "INT-OTHER"), 5, "同じ補正のほかの商品も戻る");
        assert_eq!(log_count(&conn), 0);

        // その商品を対象から外せば、ほかの商品は補正できる。上限の端は成功する
        let result = fix_integrity(
            &mut conn,
            &["INT-OTHER".to_string(), "INT-EDGE".to_string()],
        )
        .unwrap();
        assert_eq!(result.fixed_count, 2);
        assert_eq!(stock(&conn, "INT-OTHER"), 8);
        assert_eq!(stock(&conn, "INT-EDGE"), 999_999_999);
    }

    fn set_unit(conn: &DbConnection, product_code: &str, unit: &str) {
        conn.execute(
            "UPDATE products SET stock_unit = ?1 WHERE product_code = ?2",
            rusqlite::params![unit, product_code],
        )
        .unwrap();
    }

    fn latest_log_detail(conn: &DbConnection, operation_type: &str) -> serde_json::Value {
        let detail_json: String = conn
            .query_row(
                "SELECT detail_json FROM operation_logs WHERE operation_type = ?1
                 ORDER BY id DESC LIMIT 1",
                [operation_type],
                |row| row.get(0),
            )
            .unwrap();
        serde_json::from_str(&detail_json).unwrap()
    }

    #[test]
    fn test_integrity_req904_stock_unit_in_response_and_operation_logs() {
        // REQ-904 / SPEC-UNIT-D3 の表の 8・11・12（X10）: 応答と、新しく書く 2 つの操作ログに行ごとの単位を運ぶ
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "IU-M", 2750);
        set_unit(&conn, "IU-M", "m");
        add_movement(&conn, "IU-M", 2700);
        seed_product(&conn, "IU-PCS", 5);
        add_movement(&conn, "IU-PCS", 8);

        let check = run_integrity_check(&conn).unwrap();
        let units: Vec<(&str, ProductStockUnit, i64, i64)> = check
            .mismatches
            .iter()
            .map(|m| {
                (
                    m.product_code.as_str(),
                    m.stock_unit,
                    m.copy.stock_quantity,
                    m.derived.stock_quantity,
                )
            })
            .collect();
        assert_eq!(
            units,
            vec![
                ("IU-M", ProductStockUnit::M, 2750, 2700),
                ("IU-PCS", ProductStockUnit::Pcs, 5, 8)
            ]
        );
        // integrity_check の JSON は応答の DTO と別に組むので、JSON を直接確かめる
        assert_eq!(
            latest_log_detail(&conn, "integrity_check"),
            serde_json::json!({
                "checked_count": 2,
                "mismatch_count": 2,
                "mismatches": [
                    {"product_code": "IU-M", "stock_unit": "m", "copy": plain_json(2750), "derived": plain_json(2700), "differing_fields": ["quantity"]},
                    {"product_code": "IU-PCS", "stock_unit": "pcs", "copy": plain_json(5), "derived": plain_json(8), "differing_fields": ["quantity"]}
                ]
            })
        );

        let fixed = fix_integrity(&mut conn, &["IU-M".to_string(), "IU-PCS".to_string()]).unwrap();
        let adjustments: Vec<(&str, ProductStockUnit, i64, i64)> = fixed
            .adjustments
            .iter()
            .map(|a| {
                (
                    a.product_code.as_str(),
                    a.stock_unit,
                    a.before.stock_quantity,
                    a.after.stock_quantity,
                )
            })
            .collect();
        assert_eq!(
            adjustments,
            vec![
                ("IU-M", ProductStockUnit::M, 2750, 2700),
                ("IU-PCS", ProductStockUnit::Pcs, 5, 8)
            ]
        );
        assert_eq!(
            latest_log_detail(&conn, "integrity_fix")["adjustments"],
            serde_json::json!([
                {"product_code": "IU-M", "stock_unit": "m", "before": plain_json(2750), "after": plain_json(2700)},
                {"product_code": "IU-PCS", "stock_unit": "pcs", "before": plain_json(5), "after": plain_json(8)}
            ])
        );
    }

    /// 計数を直に入れる（(a) には保存の経路が無い）
    fn insert_count(conn: &DbConnection, code: &str, value: i64, date: &str) {
        crate::biz::stock_ledger::test_support::insert_count(conn, code, value, date, None);
    }

    fn set_copy(conn: &DbConnection, code: &str, copy: &StockCopy) {
        conn.execute(
            "UPDATE products SET stock_quantity = ?1, stock_recheck = ?2, stock_range_low = ?3,
                 stock_range_high = ?4 WHERE product_code = ?5",
            rusqlite::params![
                copy.stock_quantity,
                copy.stock_recheck,
                copy.stock_range_low,
                copy.stock_range_high,
                code
            ],
        )
        .unwrap();
    }

    fn ranged(stock_quantity: i64, low: i64, high: i64) -> StockCopy {
        StockCopy {
            stock_quantity,
            stock_recheck: true,
            stock_range_low: Some(low),
            stock_range_high: Some(high),
        }
    }

    #[test]
    fn test_integrity_req904_check_compares_copy_with_derived() {
        // REQ-904 / 36（突合の相手は記録から求め直した値で、和ではない）/ Matrix I1
        let (_dir, conn) = setup_test_db();
        // 計数 6 のある商品: 和は 13 だが、写し 6 は求めた値と同じなので不一致に出ない
        seed_product(&conn, "I1-SAME", 6);
        add_movement(&conn, "I1-SAME", 13);
        insert_count(&conn, "I1-SAME", 6, "2026-05-10");
        // 写し 13（和と同じ）は求めた値 6 と違うので出る
        seed_product(&conn, "I1-SUM", 13);
        add_movement(&conn, "I1-SUM", 13);
        insert_count(&conn, "I1-SUM", 6, "2026-05-10");

        let result = run_integrity_check(&conn).unwrap();
        let codes: Vec<(&str, i64, i64)> = result
            .mismatches
            .iter()
            .map(|m| {
                (
                    m.product_code.as_str(),
                    m.copy.stock_quantity,
                    m.derived.stock_quantity,
                )
            })
            .collect();
        assert_eq!(codes, vec![("I1-SUM", 13, 6)]);
        // 読取りは写しを書かない
        assert_eq!(stock_quantity(&conn, "I1-SUM"), 13);
    }

    #[test]
    fn test_integrity_req904_flag_or_range_only_difference_is_reported() {
        // REQ-904 / 36（数が同じで印・範囲だけ違う商品も一覧に出す）/ Matrix I2
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "I2", 5);
        add_movement(&conn, "I2", 5);
        set_copy(&conn, "I2", &ranged(5, 4, 6));

        let result = run_integrity_check(&conn).unwrap();
        assert_eq!(result.mismatch_count, 1);
        let m = &result.mismatches[0];
        assert_eq!(
            (m.copy.clone(), m.derived.clone()),
            (ranged(5, 4, 6), plain(5))
        );
        assert_eq!(
            m.differing_fields,
            vec![IntegrityField::Recheck, IntegrityField::Range]
        );

        // 印は同じで範囲だけ違う: 計数の衝突（8 と 11）で求めた範囲 8〜11 に対し、写しは 7〜12
        seed_product(&conn, "I2-RANGE", 8);
        insert_count(&conn, "I2-RANGE", 8, "2026-05-10");
        insert_count(&conn, "I2-RANGE", 11, "2026-05-10");
        set_copy(&conn, "I2-RANGE", &ranged(8, 7, 12));
        let result = run_integrity_check(&conn).unwrap();
        let m = result
            .mismatches
            .iter()
            .find(|m| m.product_code == "I2-RANGE")
            .unwrap();
        assert_eq!(m.derived, ranged(8, 8, 11));
        assert_eq!(m.differing_fields, vec![IntegrityField::Range]);
    }

    #[test]
    fn test_integrity_req904_differing_fields_order() {
        // REQ-904 / 36（differing_fields は quantity・recheck・range の順で重複なし）/ Matrix I3
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "I3", 7);
        add_movement(&conn, "I3", 5);
        set_copy(&conn, "I3", &ranged(7, 6, 8));
        let result = run_integrity_check(&conn).unwrap();
        assert_eq!(
            result.mismatches[0].differing_fields,
            vec![
                IntegrityField::Quantity,
                IntegrityField::Recheck,
                IntegrityField::Range
            ]
        );
        assert_eq!(
            serde_json::to_value(&result.mismatches[0].differing_fields).unwrap(),
            serde_json::json!(["quantity", "recheck", "range"])
        );
    }

    #[test]
    fn test_integrity_req904_fix_writes_recompute_result_without_movement() {
        // REQ-904 / 36・BIZ-07-D2（補正は求め直しの結果を書き、movement を作らない）/ Matrix I4
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "I4", 10);
        add_movement(&conn, "I4", 10);
        insert_count(&conn, "I4", 8, "2026-05-10");
        insert_count(&conn, "I4", 11, "2026-05-10");
        let movements_before = movement_count(&conn);

        let result = fix_integrity(&mut conn, &["I4".to_string()]).unwrap();
        assert_eq!((result.fixed_count, result.skipped_count), (1, 0));
        assert_eq!(result.adjustments[0].before, plain(10));
        assert_eq!(result.adjustments[0].after, ranged(8, 8, 11));
        let derived = StockCopy::from(&derive_stock(&conn, "I4").unwrap());
        let copy = StockCopy::from(
            product_repo::find_all_stock_quantities(&conn)
                .unwrap()
                .iter()
                .find(|row| row.product_code == "I4")
                .unwrap(),
        );
        assert_eq!(copy, derived);
        assert_eq!(movement_count(&conn), movements_before);

        // 同じ商品をもう一度: 違いが無いので skip
        let result = fix_integrity(&mut conn, &["I4".to_string()]).unwrap();
        assert_eq!((result.fixed_count, result.skipped_count), (0, 1));
    }

    #[test]
    fn test_integrity_req904_fix_log_before_after_shape() {
        // REQ-904 / 36・74（integrity_fix の detail は before・after の 4 項目。trigger を書かない）/ Matrix I5
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "I5", 9);
        add_movement(&conn, "I5", 4);
        set_copy(&conn, "I5", &ranged(9, 3, 9));
        fix_integrity(&mut conn, &["I5".to_string()]).unwrap();
        assert_eq!(
            latest_log_detail(&conn, "integrity_fix"),
            serde_json::json!({
                "fixed_count": 1,
                "skipped_count": 0,
                "adjustments": [{
                    "product_code": "I5",
                    "stock_unit": "pcs",
                    "before": {"stock_quantity": 9, "stock_recheck": true, "stock_range_low": 3, "stock_range_high": 9},
                    "after": plain_json(4),
                }],
            })
        );
    }

    #[test]
    fn test_integrity_req904_fix_duplicate_code_fixes_once() {
        // REQ-904 / 36（同じ商品が複数回来ても補正と明細は 1 回、2 回目以降は skip）
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "A", 7);
        add_movement(&conn, "A", 5);

        let result = fix_integrity(&mut conn, &["A".to_string(), "A".to_string()]).unwrap();
        assert_eq!((result.fixed_count, result.skipped_count), (1, 1));
        assert_eq!(result.adjustments.len(), 1);
        assert_eq!(stock_quantity(&conn, "A"), 5);
        assert_eq!(
            latest_log_detail(&conn, "integrity_fix"),
            serde_json::json!({
                "fixed_count": 1,
                "skipped_count": 1,
                "adjustments": [{
                    "product_code": "A",
                    "stock_unit": "pcs",
                    "before": plain_json(7),
                    "after": plain_json(5),
                }],
            })
        );
    }

    #[test]
    fn test_integrity_req904_check_log_copy_derived_shape() {
        // REQ-904 / 36（integrity_check の各 mismatch は copy・derived・differing_fields）/ Matrix I6
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "I6", 5);
        add_movement(&conn, "I6", 5);
        set_copy(&conn, "I6", &ranged(5, 4, 6));
        run_integrity_check(&conn).unwrap();
        assert_eq!(
            latest_log_detail(&conn, "integrity_check"),
            serde_json::json!({
                "checked_count": 1,
                "mismatch_count": 1,
                "mismatches": [{
                    "product_code": "I6",
                    "stock_unit": "pcs",
                    "copy": {"stock_quantity": 5, "stock_recheck": true, "stock_range_low": 4, "stock_range_high": 6},
                    "derived": plain_json(5),
                    "differing_fields": ["recheck", "range"],
                }],
            })
        );
    }

    #[test]
    fn test_run_integrity_check_req904_lists_shared_jan_conflicts() {
        // REQ-904 / ADR SPEC-STK-CNT-D10・D4 / 36（b1 Matrix I1）: 在庫連動の商品と同じ JAN の別の商品の組を
        // JAN の昇順・商品コードの昇順で返す。補正の対象にせず、写しの比較の結果を変えない
        let (_dir, conn) = setup_test_db();
        let seed = |code: &str, jan: &str, linked: bool| {
            seed_product(&conn, code, 0);
            conn.execute(
                "UPDATE products SET jan_code = ?2, pos_stock_sync = ?3 WHERE product_code = ?1",
                rusqlite::params![code, jan, linked],
            )
            .unwrap();
        };
        seed("SJ-B", "4900000000002", false);
        seed("SJ-A", "4900000000002", true);
        seed("SJ-C", "4900000000003", false);
        seed("SJ-D", "4900000000003", false);
        seed("SJ-E", "4900000000004", true);
        seed("SJ-H", "4900000000001", false);
        seed("SJ-G", "4900000000001", true);
        seed("SJ-F", "4900000000001", true);
        let result = run_integrity_check(&conn).unwrap();
        assert_eq!(
            result.shared_jan_conflicts,
            vec![
                SharedJanConflict {
                    jan_code: "4900000000001".to_string(),
                    product_codes: vec!["SJ-F".into(), "SJ-G".into(), "SJ-H".into()],
                },
                SharedJanConflict {
                    jan_code: "4900000000002".to_string(),
                    product_codes: vec!["SJ-A".into(), "SJ-B".into()],
                },
            ]
        );
        assert_eq!(result.mismatch_count, 0);
        let pos_stock_sync: Vec<bool> = conn
            .prepare("SELECT pos_stock_sync FROM products ORDER BY product_code")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            pos_stock_sync,
            vec![true, false, false, false, true, true, true, false]
        );
    }

    #[test]
    fn test_run_integrity_check_req904_no_mismatch_after_settlements_and_counts() {
        // REQ-904 / REQ-401 / REQ-205 / 36（b1 Matrix I2）: 精算の窓を持つ derive_stock と写しの比較は、
        // Z004 の確定・取消・計数の後に差異なし
        use crate::biz::csv_import_service::test_support::{
            commit_bytes, create_test_product_with_jan, make_z004_bytes,
        };
        use crate::biz::stocktake_service::{save_stock_count, SaveStockCountRequest};
        let day = "2026-04-10";
        let (_dir, mut conn) = setup_test_db();
        create_test_product_with_jan(&conn, "I2-LINK", "4912345678901", 10, true);
        create_test_product_with_jan(&conn, "I2-OFF", "4912345678902", 10, false);
        let rows = [
            ("4912345678901", "連動", 1, 100),
            ("4912345678902", "連動しない", 1, 100),
        ];
        let assert_no_mismatch = |conn: &DbConnection, step: &str| {
            let result = run_integrity_check(conn).unwrap();
            assert_eq!(result.mismatch_count, 0, "{step}: {:?}", result.mismatches);
        };
        let first = commit_bytes(&mut conn, make_z004_bytes(1, day, &rows), "1").unwrap();
        assert_no_mismatch(&conn, "確定");
        let count = |conn: &mut DbConnection, fresh: bool| {
            save_stock_count(
                conn,
                &SaveStockCountRequest {
                    product_code: "I2-LINK".to_string(),
                    count_value: 8,
                    fresh,
                    business_date: day.to_string(),
                    stocktake_item_id: None,
                },
                day,
            )
            .unwrap();
        };
        count(&mut conn, false);
        assert_no_mismatch(&conn, "既定の計数");
        commit_bytes(&mut conn, make_z004_bytes(2, "2026-04-11", &rows), "2").unwrap();
        assert_no_mismatch(&conn, "計数の後の確定");
        crate::biz::csv_import_service::rollback_csv_import(&mut conn, first.csv_import_id)
            .unwrap();
        assert_no_mismatch(&conn, "取消");
        count(&mut conn, true);
        assert_no_mismatch(&conn, "fresh な計数");
    }
}
