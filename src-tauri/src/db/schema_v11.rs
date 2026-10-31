//! migration v11（vZ）: Z004 の精算の鍵・売上の連動・棚卸しの完了の列（D-118 (b1)）
//!
//! docs/function-design/22-mnt-migration.md §20 / MNT-03-D18 に基づく実装。
//! `inventory_movements` を作り直して Z004 の movement と phase を結ぶ CHECK を足し、
//! `csv_imports`・`sale_records`・`stocktake_items`・`stocktakes`・`stock_counts` に列を足す。
//! どの表も `inventory_movements` を外部キーで参照しないので `foreign_keys` は変えない。
//! 1 つの TX で、止まる検査は全て書込みの前に置く。

use super::{migration_tx, DbError};
use rusqlite::{params, Connection, OptionalExtension};

const REDO_GUIDE: &str = "開発・試験 DB は作り直してください（22 §16 の作り直しの手順）";

/// 業務日の形の CHECK（§19 と同じ）
const DATE_SHAPE: &str = "[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]";

/// 手順 3〜4 の作り直し（今の 13 列と §19 の CHECK に、Z004 と phase を結ぶ CHECK を足す。`IS` で比べる）
fn rebuild_movements_sql() -> String {
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
    CHECK((phase = 1) = (settlement_no IS NOT NULL)),
    CHECK((reference_type IS 'csv_import') = (phase = 1))
);
INSERT INTO inventory_movements_new (
    id, product_code, movement_type, quantity, stock_after, reference_type, reference_id,
    note, is_voided, created_at, business_date, phase, settlement_no
)
SELECT
    id, product_code, movement_type, quantity, stock_after, reference_type, reference_id,
    note, is_voided, created_at, business_date, phase, settlement_no
FROM inventory_movements;
DROP TABLE inventory_movements;
ALTER TABLE inventory_movements_new RENAME TO inventory_movements;
CREATE INDEX idx_inventory_movements_product_date ON inventory_movements(product_code, created_at);
CREATE INDEX idx_inventory_movements_reference ON inventory_movements(reference_type, reference_id);
CREATE INDEX idx_inventory_movements_ledger ON inventory_movements(product_code, business_date, phase, settlement_no);"
    )
}

/// 手順 5 の列の追加（行の無い表なので値を持つことは CHECK で書く。Contract Probe P1・P2）。
/// `stock_counts.seen_import_id` は行のある表にも足せる（既存の行は NULL = 保存の時点に取込みが無かった。P4）
const ADD_COLUMNS_SQL: &str = "
ALTER TABLE csv_imports ADD COLUMN machine_no TEXT CHECK(machine_no IS NOT NULL AND machine_no <> '');
ALTER TABLE csv_imports ADD COLUMN settlement_no INTEGER CHECK(settlement_no IS NOT NULL AND settlement_no >= 0);
ALTER TABLE sale_records ADD COLUMN stock_linked INTEGER NOT NULL DEFAULT 0 CHECK(stock_linked IN (0,1));
ALTER TABLE stocktake_items ADD COLUMN valuation_quantity INTEGER;
ALTER TABLE stocktakes ADD COLUMN cutoff_import_id INTEGER;
ALTER TABLE stocktakes ADD COLUMN cutoff_count_id INTEGER;
ALTER TABLE stocktakes ADD COLUMN cutoff_movement_id INTEGER;
ALTER TABLE stock_counts ADD COLUMN seen_import_id INTEGER CHECK(seen_import_id IS NULL OR (count_kind = 'fresh' AND seen_import_id >= 1));";

/// v11 マイグレーション（MNT-03-D18 の vZ）
pub(crate) fn apply_v11_z004_settlement_key(
    conn: &Connection,
    version: i64,
) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute_batch("BEGIN;")
        .map_err(|e| DbError::MigrationFailed(format!("v{version} BEGIN失敗: {e}")))?;

    let result = (|| -> Result<(), DbError> {
        // 1. 止まる検査（読取りだけ）
        check_no_stocktake_or_z004(conn)?;

        // 2. sqlite_sequence と、手順 6 で比べる id の集合
        let sequence: Option<i64> = conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'inventory_movements'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let ids_before = movement_ids(conn)?;

        // 3〜4. 作り直し。sequence を作り直しの前の値へ戻す
        conn.execute_batch(&rebuild_movements_sql())?;
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

        // 5. 列の追加
        conn.execute_batch(ADD_COLUMNS_SQL)?;

        // 6. 行数と id の集合、foreign_key_check
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

        // 7. 版を記録
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (?1, ?2)",
            params![version, now],
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        return Err(migration_tx::rollback_after_error(
            conn,
            format!("v{version} Z004の精算の鍵の migration失敗: {error}"),
        ));
    }
    migration_tx::commit_transaction(conn, &format!("v{version} COMMIT失敗"))
}

/// 手順 1: 棚卸しか Z004 の取込みの記録がある DB は止める（§19 の手順 1 と同じ文）
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
             WHERE reference_type = 'csv_import' OR phase = 1)",
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

fn movement_ids(conn: &Connection) -> Result<Vec<i64>, DbError> {
    let mut statement = conn.prepare("SELECT id FROM inventory_movements ORDER BY id")?;
    let ids = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

/// vZ の版（Matrix の M の前書き）。並走の migration が先に入って番号を取り直すときは、M の test ではここだけを直す
#[cfg(test)]
pub(crate) const VZ_VERSION: i64 = 11;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::{self, migrate_up_to};
    use crate::db::migration_tx::{self, FailurePoint};
    use crate::db::test_support::dump;
    use crate::db::VB_VERSION;
    use rusqlite::types::Value;

    fn version(conn: &Connection) -> (i64, i64) {
        conn.query_row(
            "SELECT MAX(version), COUNT(*) FROM schema_versions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    /// schema（全 object の定義）と全表の全行
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

    /// vZ の直前の版（vB）の DB
    fn setup_before_vz() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("before-vz.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, VZ_VERSION - 1).unwrap();
        assert_eq!(version(&conn), (VZ_VERSION - 1, VZ_VERSION - 1));
        (dir, conn)
    }

    fn setup_latest() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn =
            crate::db::init_database(dir.path().join("latest.db").to_str().unwrap()).unwrap();
        (dir, conn)
    }

    fn seed_product(conn: &Connection, code: &str) {
        conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi,
                                   stock_quantity, stock_unit, created_at, updated_at)
             VALUES (?1, '合成の商品', 1, 100, 5000, 0, 'pcs', '2026-03-10T08:00:00', '2026-03-10T08:00:00')",
            params![code],
        )
        .unwrap();
    }

    fn seed_movement(
        conn: &Connection,
        code: &str,
        movement_type: &str,
        quantity: i64,
        reference: Option<(&str, i64)>,
        business_date: &str,
        settlement_no: Option<i64>,
    ) -> rusqlite::Result<i64> {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 reference_type, reference_id, note, created_at, business_date, phase, settlement_no)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, '合成', ?6 || 'T09:00:00', ?6, ?7, ?8)",
            params![
                code,
                movement_type,
                quantity,
                reference.map(|r| r.0),
                reference.map(|r| r.1),
                business_date,
                i64::from(settlement_no.is_some()),
                settlement_no
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn seed_headers(conn: &Connection) {
        conn.execute_batch(
            "INSERT INTO receiving_records (id, receiving_date, idempotency_key, request_fingerprint, created_at)
                 VALUES (11, '2026-05-03', 'vz-rcv', 'fp', '2026-05-04T09:00:00');
             INSERT INTO return_records (id, return_type, return_date, register_processed,
                 idempotency_key, request_fingerprint, created_at)
                 VALUES (12, 'return', '2026-05-06', 0, 'vz-ret', 'fp', '2026-05-07T09:00:00');
             INSERT INTO manual_sales (id, sale_date, reason, idempotency_key, request_fingerprint, created_at)
                 VALUES (13, '2026-05-08', 'other', 'vz-ms', 'fp', '2026-05-09T09:00:00');
             INSERT INTO disposal_records (id, disposal_date, idempotency_key, request_fingerprint, created_at)
                 VALUES (14, '2026-05-10', 'vz-dsp', 'fp', '2026-05-11T09:00:00');",
        )
        .unwrap();
    }

    fn migration_failed(conn: &Connection) -> String {
        match migration::migrate(conn).unwrap_err() {
            DbError::MigrationFailed(message) => message,
            other => panic!("MigrationFailed を期待: {other:?}"),
        }
    }

    fn sequence(conn: &Connection) -> Option<i64> {
        conn.query_row(
            "SELECT seq FROM sqlite_sequence WHERE name = 'inventory_movements'",
            [],
            |row| row.get(0),
        )
        .optional()
        .unwrap()
    }

    fn column_names(conn: &Connection, table: &str) -> Vec<String> {
        dump(
            conn,
            &format!("SELECT name FROM pragma_table_info('{table}')"),
        )
        .into_iter()
        .map(|row| match &row[0] {
            Value::Text(name) => name.clone(),
            other => panic!("{other:?}"),
        })
        .collect()
    }

    #[test]
    fn test_vz_req903_fresh_db_has_columns_checks_and_indexes() {
        // REQ-903 / MNT-03-D18 手順 3〜5 / SPEC-STK-CNT-D1・D9 / Matrix M1
        let (_dir, conn) = setup_latest();
        let latest = crate::db::migration::app_max_version();
        assert_eq!(version(&conn), (latest, latest)); // 最新の版まで当たる（22 §17）
        let columns = |table: &str| -> Vec<Vec<Value>> {
            dump(
                &conn,
                &format!(
                    "SELECT name, type, \"notnull\", dflt_value FROM pragma_table_info('{table}')"
                ),
            )
        };
        let text = |s: &str| Value::Text(s.into());
        let last = |table: &str, n: usize| {
            let all = columns(table);
            all[all.len() - n..].to_vec()
        };
        assert_eq!(
            last("csv_imports", 2),
            vec![
                vec![
                    text("machine_no"),
                    text("TEXT"),
                    Value::Integer(0),
                    Value::Null
                ],
                vec![
                    text("settlement_no"),
                    text("INTEGER"),
                    Value::Integer(0),
                    Value::Null
                ],
            ]
        );
        assert_eq!(
            last("sale_records", 1),
            vec![vec![
                text("stock_linked"),
                text("INTEGER"),
                Value::Integer(1),
                text("0")
            ]]
        );
        assert_eq!(
            last("stocktake_items", 1),
            vec![vec![
                text("valuation_quantity"),
                text("INTEGER"),
                Value::Integer(0),
                Value::Null
            ]]
        );
        assert_eq!(
            last("stocktakes", 3),
            vec![
                vec![
                    text("cutoff_import_id"),
                    text("INTEGER"),
                    Value::Integer(0),
                    Value::Null
                ],
                vec![
                    text("cutoff_count_id"),
                    text("INTEGER"),
                    Value::Integer(0),
                    Value::Null
                ],
                vec![
                    text("cutoff_movement_id"),
                    text("INTEGER"),
                    Value::Integer(0),
                    Value::Null
                ],
            ]
        );
        assert_eq!(
            last("stock_counts", 1),
            vec![vec![
                text("seen_import_id"),
                text("INTEGER"),
                Value::Integer(0),
                Value::Null
            ]]
        );
        assert_eq!(column_names(&conn, "inventory_movements").len(), 13);
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'inventory_movements'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            sql.contains("CHECK((reference_type IS 'csv_import') = (phase = 1))"),
            "{sql}"
        );
        assert!(sql.contains("CHECK((phase = 1) = (settlement_no IS NOT NULL))"));
        let index_columns = |index: &str| -> Vec<Vec<Value>> {
            dump(
                &conn,
                &format!("SELECT name FROM pragma_index_info('{index}') ORDER BY seqno"),
            )
        };
        let names =
            |list: &[&str]| -> Vec<Vec<Value>> { list.iter().map(|n| vec![text(n)]).collect() };
        assert_eq!(
            index_columns("idx_inventory_movements_product_date"),
            names(&["product_code", "created_at"])
        );
        assert_eq!(
            index_columns("idx_inventory_movements_reference"),
            names(&["reference_type", "reference_id"])
        );
        assert_eq!(
            index_columns("idx_inventory_movements_ledger"),
            names(&["product_code", "business_date", "phase", "settlement_no"])
        );
        assert!(dump(
            &conn,
            "SELECT name FROM sqlite_master WHERE name = 'inventory_movements_new'"
        )
        .is_empty());
    }

    #[test]
    fn test_vz_req903_rebuild_keeps_rows_ids_and_sequence() {
        // REQ-903 / MNT-03-D18 手順 2〜4 / Matrix M2
        let (_dir, conn) = setup_before_vz();
        seed_product(&conn, "VZ-1");
        seed_headers(&conn);
        seed_movement(&conn, "VZ-1", "receiving", 4, None, "2026-03-15", None).unwrap();
        seed_movement(
            &conn,
            "VZ-1",
            "receiving",
            3,
            Some(("receiving_record", 11)),
            "2026-05-03",
            None,
        )
        .unwrap();
        seed_movement(
            &conn,
            "VZ-1",
            "return",
            1,
            Some(("return_record", 12)),
            "2026-05-06",
            None,
        )
        .unwrap();
        seed_movement(
            &conn,
            "VZ-1",
            "sale_manual",
            -2,
            Some(("manual_sale", 13)),
            "2026-05-08",
            None,
        )
        .unwrap();
        seed_movement(
            &conn,
            "VZ-1",
            "disposal",
            -1,
            Some(("disposal_record", 14)),
            "2026-05-10",
            None,
        )
        .unwrap();
        // demo の seed の形の売上（sale_auto、reference_type NULL）
        seed_movement(&conn, "VZ-1", "sale_auto", -1, None, "2026-05-12", None).unwrap();
        let removed =
            seed_movement(&conn, "VZ-1", "disposal", -9, None, "2026-05-13", None).unwrap();
        conn.execute("DELETE FROM inventory_movements WHERE id = ?1", [removed])
            .unwrap();
        let rows_before = dump(&conn, "SELECT * FROM inventory_movements ORDER BY id");
        assert_eq!(rows_before.len(), 6);
        let sequence_before = sequence(&conn);
        assert_eq!(sequence_before, Some(removed));

        migrate_up_to(&conn, VZ_VERSION).unwrap();

        assert_eq!(version(&conn), (VZ_VERSION, VZ_VERSION));
        assert_eq!(
            dump(&conn, "SELECT * FROM inventory_movements ORDER BY id"),
            rows_before
        );
        assert_eq!(sequence(&conn), sequence_before);
        seed_movement(&conn, "VZ-1", "receiving", 1, None, "2026-06-01", None).unwrap();
        assert_eq!(
            conn.last_insert_rowid(),
            removed + 1,
            "消した id を使い回さない"
        );
    }

    #[test]
    fn test_vz_req903_stops_without_changes_on_stocktake_or_z004_rows() {
        // REQ-903 / MNT-03-D18 手順 1 / SPEC-STK-CNT-D10 / Matrix M3
        type Seed = fn(&Connection);
        let cases: [(&str, Seed); 4] = [
            ("stocktakes", |c| {
                c.execute(
                    "INSERT INTO stocktakes (started_at, status) VALUES ('2026-04-01T09:00:00', 'in_progress')",
                    [],
                )
                .unwrap();
            }),
            ("csv_imports", |c| {
                c.execute(
                    "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount,
                         skipped_count, status, imported_at)
                     VALUES ('Z004.CSV', '2026-04-01', 'synthetic-hash', 0, 0, 0, 'completed', '2026-04-01T20:00:00')",
                    [],
                )
                .unwrap();
            }),
            ("inventory_movements", |c| {
                seed_movement(
                    c,
                    "VZ-1",
                    "sale_auto",
                    -1,
                    Some(("csv_import", 1)),
                    "2026-04-01",
                    None,
                )
                .unwrap();
            }),
            ("inventory_movements", |c| {
                seed_movement(
                    c,
                    "VZ-1",
                    "receiving",
                    1,
                    Some(("receiving_record", 1)),
                    "2026-04-01",
                    Some(5),
                )
                .unwrap();
            }),
        ];
        for (table, seed) in cases {
            let (_dir, conn) = setup_before_vz();
            conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
            seed_product(&conn, "VZ-1");
            seed(&conn);
            let before = snapshot(&conn);
            let message = migration_failed(&conn);
            assert!(message.contains("作り直し"), "{table}: {message}");
            assert!(
                message.contains(&format!("（22 §16 の作り直しの手順）: {table}")),
                "{table}: {message}"
            );
            assert_eq!(snapshot(&conn), before, "{table}");
            assert_eq!(version(&conn), (VZ_VERSION - 1, VZ_VERSION - 1), "{table}");
            assert!(conn.is_autocommit());
        }
    }

    #[test]
    fn test_vz_req401_reference_type_and_phase_check() {
        // REQ-401 / MNT-03-D18 手順 3 / Contract Probe P3 / Matrix M4
        let (_dir, conn) = setup_latest();
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        seed_product(&conn, "VZ-1");
        let csv = Some(("csv_import", 1));
        let rcv = Some(("receiving_record", 1));
        assert!(seed_movement(&conn, "VZ-1", "sale_auto", -1, csv, "2026-04-01", None).is_err());
        assert!(seed_movement(&conn, "VZ-1", "receiving", 1, rcv, "2026-04-01", Some(5)).is_err());
        assert!(seed_movement(&conn, "VZ-1", "receiving", 1, None, "2026-04-01", Some(5)).is_err());
        assert!(seed_movement(&conn, "VZ-1", "sale_auto", -1, csv, "2026-04-01", Some(0)).is_ok());
        assert!(seed_movement(&conn, "VZ-1", "sale_auto", -1, csv, "2026-04-01", Some(7)).is_ok());
        assert!(seed_movement(&conn, "VZ-1", "receiving", 1, None, "2026-04-01", None).is_ok());
    }

    #[test]
    fn test_vz_req401_csv_imports_settlement_key_checks() {
        // REQ-401 / MNT-03-D18 手順 5 / Contract Probe P1 / Matrix M5
        let (_dir, conn) = setup_latest();
        let insert = |machine: Option<&str>, settlement: Option<i64>| {
            conn.execute(
                "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount,
                     skipped_count, status, imported_at, machine_no, settlement_no)
                 VALUES ('Z004.CSV', '2026-04-01', 'h-' || ?1 || '-' || ?2, 0, 0, 0, 'completed',
                         '2026-04-01T20:00:00', ?1, ?2)",
                params![machine, settlement],
            )
        };
        assert!(insert(None, Some(1)).is_err());
        assert!(insert(Some(""), Some(1)).is_err());
        assert!(insert(Some("0001"), None).is_err());
        assert!(insert(Some("0001"), Some(-1)).is_err());
        assert!(insert(Some("0001"), Some(42)).is_ok());
        assert!(insert(Some("0001"), Some(0)).is_ok());
    }

    #[test]
    fn test_vz_req205_stock_linked_default_and_completion_columns() {
        // REQ-205 / MNT-03-D18 手順 5 / Matrix M6
        let (_dir, conn) = setup_latest();
        seed_product(&conn, "VZ-1");
        conn.execute(
            "INSERT INTO sale_records (product_code, sale_date, quantity, amount, source, created_at)
             VALUES ('VZ-1', '2026-04-01', 1, 100, 'manual', '2026-04-01T10:00:00')",
            [],
        )
        .unwrap();
        let linked: i64 = conn
            .query_row("SELECT stock_linked FROM sale_records", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(linked, 0);
        assert!(conn
            .execute("UPDATE sale_records SET stock_linked = 2", [])
            .is_err());
        assert!(conn
            .execute("UPDATE sale_records SET stock_linked = 1", [])
            .is_ok());
        conn.execute(
            "INSERT INTO stocktakes (started_at, status, cutoff_import_id, cutoff_count_id, cutoff_movement_id)
             VALUES ('2026-04-01T09:00:00', 'in_progress', NULL, NULL, NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, valuation_quantity)
             VALUES (1, 'VZ-1', 0, NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE stocktakes SET cutoff_import_id = 3, cutoff_count_id = 4, cutoff_movement_id = 5",
            [],
        )
        .unwrap();

        // stock_counts.seen_import_id（Codex の closure の Gated Amendment。packet の Contract Probe P4）:
        // 直前の版で fresh と default の行がある表にも足せて、既存の行は NULL
        let (_dir, conn) = setup_before_vz();
        seed_product(&conn, "VZ-2");
        conn.execute_batch(
            "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                 seen_movement_id, counted_at)
             VALUES ('VZ-2', 5, 'fresh', '2026-04-01', 0, '2026-04-01T10:00:00'),
                    ('VZ-2', 4, 'default', '2026-04-01', NULL, '2026-04-01T11:00:00');",
        )
        .unwrap();
        migrate_up_to(&conn, VZ_VERSION).unwrap();
        assert_eq!(
            dump(
                &conn,
                "SELECT count_kind, seen_import_id FROM stock_counts ORDER BY id"
            ),
            vec![
                vec![Value::Text("fresh".into()), Value::Null],
                vec![Value::Text("default".into()), Value::Null],
            ]
        );
        // fresh の NULL と 1 以上を受け、default の行の値と 0 を拒む
        let insert = |kind: &str, seen_import_id: Option<i64>| {
            conn.execute(
                "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                     seen_movement_id, seen_import_id, counted_at)
                 VALUES ('VZ-2', 1, ?1, '2026-04-02', CASE ?1 WHEN 'fresh' THEN 0 END, ?2,
                         '2026-04-02T10:00:00')",
                rusqlite::params![kind, seen_import_id],
            )
        };
        assert!(insert("fresh", None).is_ok());
        assert!(insert("fresh", Some(1)).is_ok());
        assert!(insert("default", None).is_ok());
        for (kind, value) in [("fresh", 0), ("default", 1)] {
            let error = insert(kind, Some(value)).unwrap_err().to_string();
            assert!(
                error.contains("CHECK constraint failed"),
                "{kind} {value}: {error}"
            );
        }
    }

    #[test]
    fn test_vz_req903_idempotent_and_keeps_jan_alias_triggers_when_present() {
        // REQ-903 / MNT-03-D18 / Matrix M7: 2 回目の migrate は何もしない。
        // §18 の trigger は vZ より先に入った版でだけ観測できる（今の版では trigger が無いので、
        // trigger があれば作り直しの後も同じ定義で残ることだけを見る）
        let (_dir, conn) = setup_latest();
        let triggers_before = dump(
            &conn,
            "SELECT name, sql FROM sqlite_master WHERE type = 'trigger' ORDER BY name",
        );
        let before = snapshot(&conn);
        migration::migrate(&conn).unwrap();
        assert_eq!(snapshot(&conn), before);
        let rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_versions WHERE version = ?1",
                [VZ_VERSION],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(rows, 1);
        assert_eq!(
            dump(
                &conn,
                "SELECT name, sql FROM sqlite_master WHERE type = 'trigger' ORDER BY name"
            ),
            triggers_before
        );
    }

    #[test]
    fn test_vz_req903_commit_failure_after_rebuild_rolls_back_everything() {
        // REQ-903 / MNT-03-D1・MNT-03-D18 手順 7 / Matrix M8
        let (_dir, conn) = setup_before_vz();
        seed_product(&conn, "VZ-1");
        seed_movement(&conn, "VZ-1", "receiving", 5, None, "2026-04-01", None).unwrap();
        let before = snapshot(&conn);
        let sequence_before = sequence(&conn);
        let failures = migration_tx::fail_operations(&[FailurePoint::Commit]);
        let message = migration_failed(&conn);
        drop(failures);
        assert!(
            message.contains(&format!("v{VZ_VERSION} COMMIT失敗")),
            "{message}"
        );
        assert!(message.contains("ROLLBACK 成功"), "{message}");
        assert_eq!(snapshot(&conn), before);
        assert_eq!(sequence(&conn), sequence_before);
        assert_eq!(version(&conn), (VZ_VERSION - 1, VZ_VERSION - 1));
        assert!(conn.is_autocommit());
        assert!(!column_names(&conn, "csv_imports").contains(&"machine_no".to_string()));

        // 失敗を外せば同じ DB で当たる
        migrate_up_to(&conn, VZ_VERSION).unwrap();
        assert_eq!(version(&conn), (VZ_VERSION, VZ_VERSION));
    }

    #[test]
    fn test_vz_req903_verification_failure_after_rebuild_rolls_back_everything() {
        // REQ-903 / MNT-03-D18 手順 6 / Matrix M9: 手順 1 は csv_imports の行だけを見るので通り、
        // 作り直しの後の全体の foreign_key_check が sale_records を返す
        let (_dir, conn) = setup_before_vz();
        seed_product(&conn, "VZ-1");
        seed_movement(&conn, "VZ-1", "receiving", 5, None, "2026-04-01", None).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        conn.execute(
            "INSERT INTO sale_records (csv_import_id, product_code, sale_date, quantity, amount, source, created_at)
             VALUES (999, 'VZ-1', '2026-04-01', 1, 100, 'auto', '2026-04-01T10:00:00')",
            [],
        )
        .unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        let before = snapshot(&conn);
        let message = migration_failed(&conn);
        assert!(message.contains("作り直し"), "{message}");
        assert!(message.contains("foreign_key_check"), "{message}");
        assert!(message.contains("sale_records"), "{message}");
        assert_eq!(snapshot(&conn), before);
        assert_eq!(version(&conn), (VZ_VERSION - 1, VZ_VERSION - 1));
        assert!(conn.is_autocommit());
        assert_eq!(VZ_VERSION - 1, VB_VERSION);
    }
}
