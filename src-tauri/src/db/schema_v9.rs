//! migration v9（vL）: 在庫の並びの鍵・計数の表・写しの列（D-118 (a)）
//!
//! docs/function-design/22-mnt-migration.md §19 / MNT-03-D17 / D-123 に基づく実装。
//! `inventory_movements` を作り直して並びの鍵の 3 列を足し、`products` に写しの 3 列を足し、
//! `stock_counts` を作る。どの表も `inventory_movements` を外部キーで参照しないので
//! `foreign_keys` は変えない。1 つの TX で、止まる検査は全て書込みの前に置く。

use super::system_repo::{self, NewOperationLog};
use super::{migration_tx, DbError};
use rusqlite::{params, Connection, OptionalExtension};

/// 在庫の上限（手順 8。v9 の時点の値として固定する。`constants::MAX_STOCK_QUANTITY` と同じ値を test で確かめる）
const MAX_STOCK_QUANTITY: i64 = 999_999_999;

/// 業務日の形の CHECK（tracking の `business_date`。長さ 10 で `YYYY-MM-DD` の数字の形）
const DATE_SHAPE: &str = "[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]";

const REDO_GUIDE: &str = "開発・試験 DB は作り直してください（22 §16 の作り直しの手順）";

/// 既存の movement の業務日（手順 2）。参照先の header の日付、初期在庫は `created_at` の先頭 10 文字
const BUSINESS_DATE_EXPR: &str = "CASE
    WHEN m.reference_type IS NULL THEN substr(m.created_at, 1, 10)
    WHEN m.reference_type = 'receiving_record'
        THEN (SELECT receiving_date FROM receiving_records WHERE id = m.reference_id)
    WHEN m.reference_type = 'return_record'
        THEN (SELECT return_date FROM return_records WHERE id = m.reference_id)
    WHEN m.reference_type = 'manual_sale'
        THEN (SELECT sale_date FROM manual_sales WHERE id = m.reference_id)
    WHEN m.reference_type = 'disposal_record'
        THEN (SELECT disposal_date FROM disposal_records WHERE id = m.reference_id)
END";

/// 手順 4 の新しい表（今の 10 列 + 並びの鍵の 3 列と phase・settlement_no の対の CHECK）
fn create_movements_new_sql() -> String {
    format!(
        "CREATE TABLE inventory_movements_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_code TEXT NOT NULL REFERENCES products(product_code),
    movement_type TEXT NOT NULL CHECK(movement_type IN ('sale_auto','sale_manual','receiving','return','disposal','stocktake')),
    quantity INTEGER NOT NULL,
    stock_after INTEGER NOT NULL,
    reference_type TEXT CHECK(reference_type IN ('csv_import','manual_sale','receiving_record','return_record','disposal_record','stocktake') OR reference_type IS NULL),
    reference_id INTEGER,
    note TEXT,
    is_voided BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    business_date TEXT NOT NULL CHECK(length(business_date) = 10 AND business_date GLOB '{DATE_SHAPE}'),
    phase INTEGER NOT NULL CHECK(phase IN (0,1)),
    settlement_no INTEGER,
    CHECK((phase = 1) = (settlement_no IS NOT NULL))
);
INSERT INTO inventory_movements_new (
    id, product_code, movement_type, quantity, stock_after, reference_type, reference_id,
    note, is_voided, created_at, business_date, phase, settlement_no
)
SELECT
    m.id, m.product_code, m.movement_type, m.quantity, m.stock_after, m.reference_type,
    m.reference_id, m.note, m.is_voided, m.created_at, {BUSINESS_DATE_EXPR}, 0, NULL
FROM inventory_movements m;
DROP TABLE inventory_movements;
ALTER TABLE inventory_movements_new RENAME TO inventory_movements;
CREATE INDEX idx_inventory_movements_product_date ON inventory_movements(product_code, created_at);
CREATE INDEX idx_inventory_movements_reference ON inventory_movements(reference_type, reference_id);
CREATE INDEX idx_inventory_movements_ledger ON inventory_movements(product_code, business_date, phase, settlement_no);"
    )
}

/// 手順 6・7（写しの 3 列と計数の表）
fn add_copy_columns_and_counts_sql() -> String {
    format!(
        "ALTER TABLE products ADD COLUMN stock_recheck INTEGER NOT NULL DEFAULT 0 CHECK(stock_recheck IN (0,1));
ALTER TABLE products ADD COLUMN stock_range_low INTEGER;
ALTER TABLE products ADD COLUMN stock_range_high INTEGER CHECK(
    (stock_recheck = 0 AND stock_range_low IS NULL AND stock_range_high IS NULL)
    OR (stock_recheck = 1 AND stock_range_low IS NOT NULL AND stock_range_high IS NOT NULL
        AND stock_range_low <= stock_range_high));
CREATE TABLE stock_counts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_code TEXT NOT NULL REFERENCES products(product_code),
    count_value INTEGER NOT NULL CHECK(count_value >= 0),
    count_kind TEXT NOT NULL CHECK(count_kind IN ('default','fresh')),
    business_date TEXT NOT NULL CHECK(length(business_date) = 10 AND business_date GLOB '{DATE_SHAPE}'),
    settlement_no INTEGER,
    seen_movement_id INTEGER,
    stocktake_item_id INTEGER REFERENCES stocktake_items(id),
    counted_at TEXT NOT NULL,
    CHECK((count_kind = 'default' AND settlement_no IS NULL AND seen_movement_id IS NULL)
        OR (count_kind = 'fresh' AND seen_movement_id IS NOT NULL))
);
CREATE INDEX idx_stock_counts_product_id ON stock_counts(product_code, id);"
    )
}

/// 手順 8 で直す商品（商品コード・単位・今の写し・和）
struct CopyFix {
    product_code: String,
    stock_unit: String,
    before: i64,
    after: i64,
}

/// v9 マイグレーション（MNT-03-D17 の vL）
pub(crate) fn apply_v9_stock_ledger(conn: &Connection, version: i64) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute_batch("BEGIN;")
        .map_err(|e| DbError::MigrationFailed(format!("v{version} BEGIN失敗: {e}")))?;

    let result = (|| -> Result<(), DbError> {
        // 1〜2・8（読取りだけ）。止まる DB では何も書かない
        check_no_stocktake_or_z004(conn)?;
        check_business_dates(conn)?;
        let fixes = copy_fixes(conn)?;

        // 3. sqlite_sequence と、手順 9 で比べる id の集合
        let sequence: Option<i64> = conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'inventory_movements'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let ids_before = movement_ids(conn)?;

        // 4〜5. 作り直し。sequence を作り直しの前の値へ戻す（作り直しは写した最大の id へ戻る）
        conn.execute_batch(&create_movements_new_sql())?;
        conn.execute(
            "DELETE FROM sqlite_sequence WHERE name = 'inventory_movements'",
            [],
        )?;
        if let Some(seq) = sequence {
            conn.execute(
                "INSERT INTO sqlite_sequence (name, seq) VALUES ('inventory_movements', ?1)",
                [seq],
            )?;
        }

        // 6〜7
        conn.execute_batch(&add_copy_columns_and_counts_sql())?;

        // 8. 写しを和へ直し、直した商品を 1 行の操作ログに残す
        write_copy_fixes(conn, &fixes)?;

        // 9. 行数と id の集合、foreign_key_check
        let ids_after = movement_ids(conn)?;
        if ids_after != ids_before {
            return Err(DbError::MigrationFailed(format!(
                "在庫の記録を新しい形へ写せなかった DB はこの版へ移せません。{REDO_GUIDE}: inventory_movements（行数と id の集合: {} -> {}）",
                ids_before.len(),
                ids_after.len()
            )));
        }
        let fk_tables: Vec<String> = {
            let mut statement =
                conn.prepare("SELECT DISTINCT \"table\" FROM pragma_foreign_key_check")?;
            let rows = statement
                .query_map([], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        if !fk_tables.is_empty() {
            return Err(DbError::MigrationFailed(format!(
                "在庫の記録を新しい形へ写せなかった DB はこの版へ移せません。{REDO_GUIDE}: inventory_movements（foreign_key_check: {}）",
                fk_tables.join("、")
            )));
        }

        // 10. 版を記録
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (?1, ?2)",
            params![version, now],
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        return Err(migration_tx::rollback_after_error(
            conn,
            format!("v{version} 在庫の並びの鍵の migration失敗: {error}"),
        ));
    }
    migration_tx::commit_transaction(conn, &format!("v{version} COMMIT失敗"))
}

/// 手順 1: 棚卸しか Z004 の取込みの記録がある DB は止める
fn check_no_stocktake_or_z004(conn: &Connection) -> Result<(), DbError> {
    let mut hits = Vec::new();
    for table in ["stocktakes", "csv_imports"] {
        let exists: bool = conn.query_row(
            &format!("SELECT EXISTS(SELECT 1 FROM {table})"),
            [],
            |row| row.get(0),
        )?;
        if exists {
            hits.push(table);
        }
    }
    let movement_hit: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM inventory_movements
             WHERE reference_type IN ('csv_import', 'stocktake') OR movement_type = 'stocktake')",
        [],
        |row| row.get(0),
    )?;
    if movement_hit {
        hits.push("inventory_movements");
    }
    if hits.is_empty() {
        return Ok(());
    }
    Err(DbError::MigrationFailed(format!(
        "棚卸しか商品別売上（Z004）の取込みの記録がある DB はこの版へ移せません。{REDO_GUIDE}: {}",
        hits.join("、")
    )))
}

/// 手順 2: 業務日を決められない movement（参照先なし・形の外れ・暦の外れ）がある DB は止める
fn check_business_dates(conn: &Connection) -> Result<(), DbError> {
    let hit: Option<Option<String>> = conn
        .query_row(
            &format!(
                "SELECT reference_type FROM (
                     SELECT m.id, m.reference_type, {BUSINESS_DATE_EXPR} AS d FROM inventory_movements m)
                 WHERE d IS NULL
                    OR NOT (length(d) = 10 AND d GLOB '{DATE_SHAPE}')
                    OR date(d) IS NOT d
                 ORDER BY id LIMIT 1"
            ),
            [],
            |row| row.get(0),
        )
        .optional()?;
    match hit {
        None => Ok(()),
        Some(reference_type) => Err(DbError::MigrationFailed(format!(
            "業務日を決められない在庫の記録がある DB はこの版へ移せません。{REDO_GUIDE}: inventory_movements（reference_type: {}）",
            reference_type.as_deref().unwrap_or("NULL")
        ))),
    }
}

/// 手順 8 の読取り: 和（計数も Z004 も棚卸しも無い DB なので求め直しと同じ）が写しと違う商品。
/// 和が在庫の上限の外の商品が 1 つでもあれば止める
fn copy_fixes(conn: &Connection) -> Result<Vec<CopyFix>, DbError> {
    let mut statement = conn.prepare(
        "SELECT p.product_code, p.stock_unit, p.stock_quantity,
                COALESCE((SELECT SUM(m.quantity) FROM inventory_movements m
                          WHERE m.product_code = p.product_code AND m.is_voided = 0), 0)
         FROM products p ORDER BY p.product_code",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok(CopyFix {
                product_code: row.get(0)?,
                stock_unit: row.get(1)?,
                before: row.get(2)?,
                after: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if rows
        .iter()
        .any(|row| !(-MAX_STOCK_QUANTITY..=MAX_STOCK_QUANTITY).contains(&row.after))
    {
        return Err(DbError::MigrationFailed(format!(
            "記録から求め直した在庫数が上限を超える商品がある DB はこの版へ移せません。{REDO_GUIDE}: products.stock_quantity"
        )));
    }
    Ok(rows
        .into_iter()
        .filter(|row| row.before != row.after)
        .collect())
}

/// 手順 8 の書込み: 写しを和へ直し、36 の `integrity_fix` の形に `trigger: migration` を足した 1 行を残す
fn write_copy_fixes(conn: &Connection, fixes: &[CopyFix]) -> Result<(), DbError> {
    if fixes.is_empty() {
        return Ok(());
    }
    let copy = |stock: i64| {
        serde_json::json!({
            "stock_quantity": stock,
            "stock_recheck": false,
            "stock_range_low": null,
            "stock_range_high": null,
        })
    };
    for fix in fixes {
        conn.execute(
            "UPDATE products SET stock_quantity = ?1 WHERE product_code = ?2",
            params![fix.after, fix.product_code],
        )?;
    }
    let detail = serde_json::json!({
        "fixed_count": fixes.len(),
        "skipped_count": 0,
        "adjustments": fixes.iter().map(|fix| serde_json::json!({
            "product_code": fix.product_code,
            "stock_unit": fix.stock_unit,
            "before": copy(fix.before),
            "after": copy(fix.after),
        })).collect::<Vec<_>>(),
        "trigger": "migration",
    });
    system_repo::insert_operation_log(
        conn,
        &NewOperationLog {
            operation_type: "integrity_fix".to_string(),
            summary: format!(
                "更新のときに在庫数を記録から求め直し、{}商品の在庫数を直しました",
                fixes.len()
            ),
            detail_json: Some(detail.to_string()),
        },
    )
}

fn movement_ids(conn: &Connection) -> Result<Vec<i64>, DbError> {
    let mut statement = conn.prepare("SELECT id FROM inventory_movements ORDER BY id")?;
    let ids = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::{self, migrate_up_to};
    use crate::db::migration_tx::{self, FailurePoint};
    use rusqlite::types::Value;

    fn dump(conn: &Connection, sql: &str) -> Vec<Vec<Value>> {
        let mut statement = conn.prepare(sql).unwrap();
        let columns = statement.column_count();
        statement
            .query_map([], |row| (0..columns).map(|i| row.get(i)).collect())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn version(conn: &Connection) -> (i64, i64) {
        conn.query_row(
            "SELECT MAX(version), COUNT(*) FROM schema_versions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    /// schema（全 object の定義）と全表の全行（失敗で何も変わらないことの比較に使う）
    fn snapshot(conn: &Connection) -> Vec<Vec<Vec<Value>>> {
        let mut out = vec![dump(
            conn,
            "SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name",
        )];
        let tables: Vec<String> = dump(
            conn,
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        )
        .into_iter()
        .map(|row| match &row[0] {
            Value::Text(name) => name.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
        for table in tables {
            out.push(dump(conn, &format!("SELECT * FROM \"{table}\" ORDER BY 1")));
        }
        out
    }

    /// 版 8（vL の直前）の DB
    fn setup_v8_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("v8.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, 8).unwrap();
        assert_eq!(version(&conn), (8, 8));
        (dir, conn)
    }

    fn seed_product(conn: &Connection, code: &str, stock: i64) {
        conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price,
                                   stock_quantity, stock_unit, created_at, updated_at)
             VALUES (?1, '合成の商品', 1, 100, 50, ?2, 'pcs', '2026-03-10T08:00:00', '2026-03-10T08:00:00')",
            params![code, stock],
        )
        .unwrap();
    }

    /// 最新の版（vB の後）の DB に商品を入れる（原価の列は `cost_price_centi`。Matrix T2）
    fn seed_product_latest(conn: &Connection, code: &str, stock: i64) {
        conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi,
                                   stock_quantity, stock_unit, created_at, updated_at)
             VALUES (?1, '合成の商品', 1, 100, 50, ?2, 'pcs', '2026-03-10T08:00:00', '2026-03-10T08:00:00')",
            params![code, stock],
        )
        .unwrap();
    }

    fn seed_movement(
        conn: &Connection,
        code: &str,
        movement_type: &str,
        quantity: i64,
        reference: Option<(&str, i64)>,
        created_at: &str,
    ) -> i64 {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 reference_type, reference_id, note, created_at)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, '合成', ?6)",
            params![
                code,
                movement_type,
                quantity,
                reference.map(|r| r.0),
                reference.map(|r| r.1),
                created_at
            ],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// 入庫・返品交換・手動販売・廃棄の header（日付は created_at と違う日）
    fn seed_headers(conn: &Connection, receiving_date: &str) {
        conn.execute(
            "INSERT INTO receiving_records (id, receiving_date, idempotency_key, request_fingerprint, created_at)
             VALUES (11, ?1, 'vl-rcv', 'fp', '2026-05-04T09:00:00')",
            [receiving_date],
        )
        .unwrap();
        conn.execute_batch(
            "INSERT INTO return_records (id, return_type, return_date, register_processed,
                 idempotency_key, request_fingerprint, created_at)
                 VALUES (12, 'return', '2026-05-06', 0, 'vl-ret', 'fp', '2026-05-07T09:00:00');
             INSERT INTO manual_sales (id, sale_date, reason, idempotency_key, request_fingerprint, created_at)
                 VALUES (13, '2026-05-08', 'other', 'vl-ms', 'fp', '2026-05-09T09:00:00');
             INSERT INTO disposal_records (id, disposal_date, idempotency_key, request_fingerprint, created_at)
                 VALUES (14, '2026-05-10', 'vl-dsp', 'fp', '2026-05-11T09:00:00');",
        )
        .unwrap();
    }

    fn migration_failed(conn: &Connection) -> String {
        match migration::migrate(conn).unwrap_err() {
            DbError::MigrationFailed(message) => message,
            other => panic!("MigrationFailed を期待: {other:?}"),
        }
    }

    fn assert_stops_without_changes(conn: &Connection, needles: &[&str]) {
        let before = snapshot(conn);
        let message = migration_failed(conn);
        for needle in needles {
            assert!(message.contains(needle), "{needle}: {message}");
        }
        assert_eq!(snapshot(conn), before, "{message}");
        assert_eq!(version(conn), (8, 8));
        assert!(conn.is_autocommit());
    }

    #[test]
    fn test_migration_req903_vl_fresh_database_has_keys_counts_and_copy_columns() {
        // REQ-903 / MNT-03-D17 手順 4〜7 / SPEC-STK-CNT-D1・D9 / Matrix M1
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::db::init_database(dir.path().join("fresh.db").to_str().unwrap()).unwrap();
        assert_eq!(
            version(&conn),
            (
                crate::db::migration::app_max_version(),
                crate::db::migration::app_max_version()
            )
        );
        let columns = |table: &str| -> Vec<Vec<Value>> {
            dump(
                &conn,
                &format!(
                    "SELECT name, type, \"notnull\", dflt_value FROM pragma_table_info('{table}')"
                ),
            )
        };
        let movements = columns("inventory_movements");
        assert_eq!(movements.len(), 13);
        assert_eq!(
            movements[10..].to_vec(),
            vec![
                vec![
                    Value::Text("business_date".into()),
                    Value::Text("TEXT".into()),
                    Value::Integer(1),
                    Value::Null
                ],
                vec![
                    Value::Text("phase".into()),
                    Value::Text("INTEGER".into()),
                    Value::Integer(1),
                    Value::Null
                ],
                vec![
                    Value::Text("settlement_no".into()),
                    Value::Text("INTEGER".into()),
                    Value::Integer(0),
                    Value::Null
                ],
            ]
        );
        let products = columns("products");
        assert_eq!(
            products[products.len() - 3..].to_vec(),
            vec![
                vec![
                    Value::Text("stock_recheck".into()),
                    Value::Text("INTEGER".into()),
                    Value::Integer(1),
                    Value::Text("0".into())
                ],
                vec![
                    Value::Text("stock_range_low".into()),
                    Value::Text("INTEGER".into()),
                    Value::Integer(0),
                    Value::Null
                ],
                vec![
                    Value::Text("stock_range_high".into()),
                    Value::Text("INTEGER".into()),
                    Value::Integer(0),
                    Value::Null
                ],
            ]
        );
        let counts: Vec<String> = columns("stock_counts")
            .into_iter()
            .map(|row| match &row[0] {
                Value::Text(name) => name.clone(),
                other => panic!("{other:?}"),
            })
            .collect();
        // v9 の 9 列がこの順で先頭に並ぶ（後の版の vZ が seen_import_id を足す。それは vZ の M1 が見る。
        // b1 Matrix T1）
        assert_eq!(
            counts[..9],
            [
                "id",
                "product_code",
                "count_value",
                "count_kind",
                "business_date",
                "settlement_no",
                "seen_movement_id",
                "stocktake_item_id",
                "counted_at"
            ]
        );
        let index_columns = |index: &str| -> Vec<Vec<Value>> {
            dump(
                &conn,
                &format!("SELECT name FROM pragma_index_info('{index}') ORDER BY seqno"),
            )
        };
        let text = |names: &[&str]| -> Vec<Vec<Value>> {
            names
                .iter()
                .map(|n| vec![Value::Text((*n).into())])
                .collect()
        };
        assert_eq!(
            index_columns("idx_inventory_movements_ledger"),
            text(&["product_code", "business_date", "phase", "settlement_no"])
        );
        assert_eq!(
            index_columns("idx_stock_counts_product_id"),
            text(&["product_code", "id"])
        );
        assert_eq!(
            index_columns("idx_inventory_movements_product_date"),
            text(&["product_code", "created_at"])
        );
        assert_eq!(
            index_columns("idx_inventory_movements_reference"),
            text(&["reference_type", "reference_id"])
        );
        assert!(dump(
            &conn,
            "SELECT name FROM sqlite_master WHERE name = 'inventory_movements_new'"
        )
        .is_empty());
    }

    #[test]
    fn test_migration_req201_vl_backfills_business_date_and_keeps_ids_and_sequence() {
        // REQ-201 / MNT-03-D17 手順 2〜5 / Y1・Y8 / Matrix M2
        let (_dir, conn) = setup_v8_db();
        seed_product(&conn, "VL-1", 5);
        seed_headers(&conn, "2026-05-03");
        let initial = seed_movement(&conn, "VL-1", "receiving", 4, None, "2026-03-15T10:00:00");
        seed_movement(
            &conn,
            "VL-1",
            "receiving",
            3,
            Some(("receiving_record", 11)),
            "2026-05-04T09:00:00",
        );
        seed_movement(
            &conn,
            "VL-1",
            "return",
            1,
            Some(("return_record", 12)),
            "2026-05-07T09:00:00",
        );
        seed_movement(
            &conn,
            "VL-1",
            "sale_manual",
            -2,
            Some(("manual_sale", 13)),
            "2026-05-09T09:00:00",
        );
        seed_movement(
            &conn,
            "VL-1",
            "disposal",
            -1,
            Some(("disposal_record", 14)),
            "2026-05-11T09:00:00",
        );
        let removed = seed_movement(&conn, "VL-1", "disposal", -9, None, "2026-05-12T09:00:00");
        conn.execute("DELETE FROM inventory_movements WHERE id = ?1", [removed])
            .unwrap();
        let rows_before = dump(&conn, "SELECT * FROM inventory_movements ORDER BY id");
        let sequence_before: i64 = conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'inventory_movements'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(sequence_before, removed);

        migrate_up_to(&conn, crate::db::VB_VERSION - 1).unwrap(); // products に行（Matrix T1 (b)）

        assert_eq!(version(&conn), (9, 9));
        let rows_after = dump(&conn, "SELECT * FROM inventory_movements ORDER BY id");
        assert_eq!(rows_after.len(), rows_before.len());
        let expected_dates = [
            "2026-03-15",
            "2026-05-03",
            "2026-05-06",
            "2026-05-08",
            "2026-05-10",
        ];
        for ((after, before), date) in rows_after.iter().zip(&rows_before).zip(expected_dates) {
            assert_eq!(after[..10], before[..], "ほかの列と id は同じ");
            assert_eq!(
                after[10..],
                [Value::Text(date.into()), Value::Integer(0), Value::Null]
            );
        }
        assert_eq!(rows_after[0][0], Value::Integer(initial));
        let sequence_after: i64 = conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'inventory_movements'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(sequence_after, sequence_before);
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 created_at, business_date, phase) VALUES ('VL-1', 'receiving', 1, 6, '2026-06-01T00:00:00', '2026-06-01', 0)",
            [],
        )
        .unwrap();
        assert_eq!(
            conn.last_insert_rowid(),
            removed + 1,
            "消した id を使い回さない"
        );
        // 写し 5 = 和 5 なので直さず、操作ログも書かない
        let logs: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(logs, 0);
    }

    #[test]
    fn test_migration_req205_vl_stops_on_stocktakes_or_csv_imports_without_changes() {
        // REQ-205 / MNT-03-D17 手順 1 (a) / SPEC-STK-CNT-D10 / Matrix M3
        let (_dir, conn) = setup_v8_db();
        conn.execute(
            "INSERT INTO stocktakes (started_at, status) VALUES ('2026-04-01T09:00:00', 'in_progress')",
            [],
        )
        .unwrap();
        assert_stops_without_changes(&conn, &["作り直し", "stocktakes"]);

        let (_dir, conn) = setup_v8_db();
        conn.execute(
            "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount,
                 skipped_count, status, imported_at)
             VALUES ('Z004.CSV', '2026-04-01', 'synthetic-hash', 0, 0, 0, 'completed', '2026-04-01T20:00:00')",
            [],
        )
        .unwrap();
        assert_stops_without_changes(&conn, &["作り直し", "csv_imports"]);
    }

    #[test]
    fn test_migration_req903_vl_stops_on_unmappable_movements() {
        // REQ-903 / MNT-03-D17 手順 1 (b)・手順 2 / Contract Probe P7 / Matrix M4
        type Seed = fn(&Connection);
        let cases: [(&str, Seed); 8] = [
            ("csv_import の movement", |c| {
                seed_movement(
                    c,
                    "VL-1",
                    "sale_auto",
                    -1,
                    Some(("csv_import", 1)),
                    "2026-04-01T00:00:00",
                );
            }),
            ("receiving で reference_type が stocktake", |c| {
                seed_movement(
                    c,
                    "VL-1",
                    "receiving",
                    1,
                    Some(("stocktake", 1)),
                    "2026-04-01T00:00:00",
                );
            }),
            ("movement_type が stocktake", |c| {
                seed_movement(c, "VL-1", "stocktake", 1, None, "2026-04-01T00:00:00");
            }),
            ("header の無い入庫", |c| {
                seed_movement(
                    c,
                    "VL-1",
                    "receiving",
                    1,
                    Some(("receiving_record", 99)),
                    "2026-04-01T00:00:00",
                );
            }),
            ("形の外れる日付", |c| {
                seed_headers(c, "2026/10/01");
                seed_movement(
                    c,
                    "VL-1",
                    "receiving",
                    1,
                    Some(("receiving_record", 11)),
                    "2026-04-01T00:00:00",
                );
            }),
            ("繰り上がる日付", |c| {
                seed_headers(c, "2026-02-30");
                seed_movement(
                    c,
                    "VL-1",
                    "receiving",
                    1,
                    Some(("receiving_record", 11)),
                    "2026-04-01T00:00:00",
                );
            }),
            ("日が範囲外", |c| {
                seed_headers(c, "2026-02-32");
                seed_movement(
                    c,
                    "VL-1",
                    "receiving",
                    1,
                    Some(("receiving_record", 11)),
                    "2026-04-01T00:00:00",
                );
            }),
            ("月が範囲外", |c| {
                seed_headers(c, "2026-13-01");
                seed_movement(
                    c,
                    "VL-1",
                    "receiving",
                    1,
                    Some(("receiving_record", 11)),
                    "2026-04-01T00:00:00",
                );
            }),
        ];
        for (name, seed) in cases {
            let (_dir, conn) = setup_v8_db();
            seed_product(&conn, "VL-1", 0);
            seed(&conn);
            let before = snapshot(&conn);
            let message = migration_failed(&conn);
            assert!(message.contains("作り直し"), "{name}: {message}");
            assert!(message.contains("inventory_movements"), "{name}: {message}");
            assert_eq!(snapshot(&conn), before, "{name}");
            assert_eq!(version(&conn), (8, 8), "{name}");
        }
        // header の無い入庫は reference_type を名指す
        let (_dir, conn) = setup_v8_db();
        seed_product(&conn, "VL-1", 0);
        seed_movement(
            &conn,
            "VL-1",
            "receiving",
            1,
            Some(("receiving_record", 99)),
            "2026-04-01T00:00:00",
        );
        assert!(migration_failed(&conn).contains("reference_type: receiving_record"));
        // 初期在庫は NULL と書く
        let (_dir, conn) = setup_v8_db();
        seed_product(&conn, "VL-1", 0);
        seed_movement(&conn, "VL-1", "receiving", 1, None, "2026-02-30T00:00:00");
        assert!(migration_failed(&conn).contains("reference_type: NULL"));
    }

    #[test]
    fn test_migration_req903_vl_stops_when_recomputed_stock_out_of_range() {
        // REQ-903 / MNT-03-D17 手順 8 / Y2 / Matrix M5
        let (_dir, conn) = setup_v8_db();
        seed_product(&conn, "VL-1", 0);
        seed_movement(
            &conn,
            "VL-1",
            "receiving",
            500_000_000,
            None,
            "2026-04-01T00:00:00",
        );
        seed_movement(
            &conn,
            "VL-1",
            "receiving",
            500_000_000,
            None,
            "2026-04-02T00:00:00",
        );
        assert_stops_without_changes(&conn, &["作り直し", "products", "stock_quantity"]);

        for edge in [999_999_999_i64, -999_999_999] {
            let (_dir, conn) = setup_v8_db();
            seed_product(&conn, "VL-1", edge);
            seed_movement(
                &conn,
                "VL-1",
                "receiving",
                edge,
                None,
                "2026-04-01T00:00:00",
            );
            migrate_up_to(&conn, crate::db::VB_VERSION - 1).unwrap(); // products に行（Matrix T1 (b)）
            assert_eq!(version(&conn), (9, 9), "{edge}");
        }
    }

    #[test]
    fn test_migration_req904_vl_fixes_copy_and_logs_trigger_migration() {
        // REQ-904 / MNT-03-D17 手順 8 / Y7 / SPEC-STK-CNT-D2・D10 / Matrix M6
        let (_dir, conn) = setup_v8_db();
        seed_product(&conn, "VL-A", 7);
        seed_product(&conn, "VL-B", 3);
        seed_movement(&conn, "VL-A", "receiving", 6, None, "2026-04-01T00:00:00");
        seed_movement(&conn, "VL-A", "receiving", -1, None, "2026-04-02T00:00:00");
        seed_movement(&conn, "VL-B", "receiving", 3, None, "2026-04-01T00:00:00");
        // 無効の movement は和に入れない
        let voided = seed_movement(&conn, "VL-B", "receiving", 40, None, "2026-04-03T00:00:00");
        conn.execute(
            "UPDATE inventory_movements SET is_voided = 1 WHERE id = ?1",
            [voided],
        )
        .unwrap();

        migrate_up_to(&conn, crate::db::VB_VERSION - 1).unwrap(); // products に行（Matrix T1 (b)）

        let stocks = dump(
            &conn,
            "SELECT product_code, stock_quantity, stock_recheck, stock_range_low, stock_range_high
             FROM products WHERE product_code LIKE 'VL-%' ORDER BY product_code",
        );
        assert_eq!(
            stocks,
            vec![
                vec![
                    Value::Text("VL-A".into()),
                    Value::Integer(5),
                    Value::Integer(0),
                    Value::Null,
                    Value::Null
                ],
                vec![
                    Value::Text("VL-B".into()),
                    Value::Integer(3),
                    Value::Integer(0),
                    Value::Null,
                    Value::Null
                ],
            ]
        );
        let logs: Vec<(String, String, String, String)> = {
            let mut statement = conn
                .prepare(
                    "SELECT operation_type, summary, detail_json, created_at FROM operation_logs",
                )
                .unwrap();
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            rows
        };
        assert_eq!(logs.len(), 1);
        let (operation_type, summary, detail, created_at) = &logs[0];
        assert_eq!(operation_type, "integrity_fix");
        assert_eq!(
            summary,
            "更新のときに在庫数を記録から求め直し、1商品の在庫数を直しました"
        );
        chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S").unwrap();
        let detail: serde_json::Value = serde_json::from_str(detail).unwrap();
        assert_eq!(
            detail,
            serde_json::json!({
                "fixed_count": 1,
                "skipped_count": 0,
                "adjustments": [{
                    "product_code": "VL-A",
                    "stock_unit": "pcs",
                    "before": {"stock_quantity": 7, "stock_recheck": false, "stock_range_low": null, "stock_range_high": null},
                    "after": {"stock_quantity": 5, "stock_recheck": false, "stock_range_low": null, "stock_range_high": null},
                }],
                "trigger": "migration",
            })
        );

        // vL の後の全商品で、求め直し（BIZ の derive_stock）が写しと同じ
        let counts: i64 = conn
            .query_row("SELECT COUNT(*) FROM stock_counts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(counts, 0);
        for row in dump(&conn, "SELECT product_code, stock_quantity FROM products") {
            let (Value::Text(code), Value::Integer(stock)) = (&row[0], &row[1]) else {
                panic!("{row:?}")
            };
            // v9 の DB に Z004 の取込みと計数は無い（取込みがあれば v9 で止まり、計数の保存の経路は b1）ので、
            // 精算の一覧と計数を空にして同じ求め直しを当てる（最新の読取りは vZ の列を読む。b1 Matrix T1）
            let ledger = crate::biz::stock_ledger::Ledger {
                movements: crate::db::inventory_repo::list_ledger_movements_for_product(
                    &conn, code,
                )
                .unwrap(),
                ..Default::default()
            };
            let derived = crate::biz::stock_ledger::derive_ledger(&ledger).unwrap();
            assert_eq!((derived.stock, derived.range), (*stock, None), "{code}");
        }

        // 写し＝和だけの DB では行を書かない
        let (_dir, conn) = setup_v8_db();
        seed_product(&conn, "VL-B", 3);
        seed_movement(&conn, "VL-B", "receiving", 3, None, "2026-04-01T00:00:00");
        migrate_up_to(&conn, crate::db::VB_VERSION - 1).unwrap(); // products に行（Matrix T1 (b)）
        let logs: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(logs, 0);
    }

    #[test]
    fn test_migration_req205_vl_stock_counts_checks() {
        // REQ-205 / SPEC-STK-CNT-D9（stock_counts の CHECK・FK）/ Matrix M7
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::db::init_database(dir.path().join("t.db").to_str().unwrap()).unwrap();
        seed_product_latest(&conn, "VL-1", 0);
        let insert = |code: &str,
                      value: i64,
                      kind: &str,
                      settlement: Option<i64>,
                      seen: Option<i64>,
                      item: Option<i64>| {
            conn.execute(
                "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                     settlement_no, seen_movement_id, stocktake_item_id, counted_at)
                 VALUES (?1, ?2, ?3, '2026-04-01', ?4, ?5, ?6, '2026-04-01T10:00:00')",
                params![code, value, kind, settlement, seen, item],
            )
        };
        assert!(insert("VL-1", 0, "default", None, None, None).is_ok());
        assert!(insert("VL-1", 4, "fresh", None, Some(0), None).is_ok());
        assert!(insert("VL-1", 4, "fresh", Some(3), Some(7), None).is_ok());
        assert!(insert("VL-1", -1, "default", None, None, None).is_err());
        assert!(insert("VL-1", 1, "other", None, None, None).is_err());
        assert!(insert("VL-1", 1, "default", None, Some(1), None).is_err());
        assert!(insert("VL-1", 1, "default", Some(1), None, None).is_err());
        assert!(insert("VL-1", 1, "fresh", None, None, None).is_err());
        assert!(insert("VL-NONE", 1, "default", None, None, None).is_err());
        assert!(insert("VL-1", 1, "default", None, None, Some(999)).is_err());
        let bad_date = conn.execute(
            "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date, counted_at)
             VALUES ('VL-1', 1, 'default', '2026-4-1', '2026-04-01T10:00:00')",
            [],
        );
        assert!(bad_date.is_err());
    }

    #[test]
    fn test_migration_req903_vl_copy_columns_check() {
        // REQ-903 / SPEC-STK-CNT-D9（写しの列の CHECK）/ Y3 / Contract Probe P1 / Matrix M8
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::db::init_database(dir.path().join("t.db").to_str().unwrap()).unwrap();
        seed_product_latest(&conn, "VL-1", 0);
        let set = |recheck: i64, low: Option<i64>, high: Option<i64>| {
            conn.execute(
                "UPDATE products SET stock_recheck = ?1, stock_range_low = ?2, stock_range_high = ?3
                 WHERE product_code = 'VL-1'",
                params![recheck, low, high],
            )
        };
        assert!(set(1, None, None).is_err());
        assert!(set(1, Some(5), None).is_err());
        assert!(set(1, Some(6), Some(5)).is_err());
        assert!(set(0, Some(4), Some(5)).is_err());
        assert!(set(2, None, None).is_err());
        assert!(set(1, Some(4), Some(5)).is_ok());
        assert!(set(1, Some(5), Some(5)).is_ok());
        assert!(set(0, None, None).is_ok());
        // 既定の INSERT（列を書かない）は 0・NULL・NULL
        seed_product_latest(&conn, "VL-2", 0);
        let row = dump(
            &conn,
            "SELECT stock_recheck, stock_range_low, stock_range_high FROM products WHERE product_code = 'VL-2'",
        );
        assert_eq!(row, vec![vec![Value::Integer(0), Value::Null, Value::Null]]);
    }

    #[test]
    fn test_migration_req903_vl_commit_failure_rolls_back_everything() {
        // REQ-903 / MNT-03-D1・MNT-03-D17 手順 9〜10 / Matrix M9
        let (_dir, conn) = setup_v8_db();
        seed_product(&conn, "VL-A", 7);
        seed_movement(&conn, "VL-A", "receiving", 5, None, "2026-04-01T00:00:00");
        let before = snapshot(&conn);
        let failures = migration_tx::fail_operations(&[FailurePoint::Commit]);
        let message = migration_failed(&conn);
        drop(failures);
        assert!(message.contains("v9 COMMIT失敗"), "{message}");
        assert!(message.contains("ROLLBACK 成功"), "{message}");
        assert_eq!(snapshot(&conn), before);
        assert_eq!(version(&conn), (8, 8));
        assert!(conn.is_autocommit());

        // 失敗を外せば同じ DB で当たる
        migrate_up_to(&conn, crate::db::VB_VERSION - 1).unwrap(); // products に行（Matrix T1 (b)）
        assert_eq!(version(&conn), (9, 9));
    }

    #[test]
    fn test_v9_req903_stock_limit_matches_runtime_constant() {
        // REQ-903 / MNT-03-D17 手順 8: 移行の上限は BIZ の検証と同じ値
        assert_eq!(MAX_STOCK_QUANTITY, crate::constants::MAX_STOCK_QUANTITY);
    }
}
