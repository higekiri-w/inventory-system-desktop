//! migration vT: 振替（詰め替え・切り分け）の種類と振替の記録の表（D-143）
//!
//! docs/function-design/22-mnt-migration.md §21 / MNT-03-D22 に基づく実装。
//! `inventory_movements` を作り直して `movement_type` の CHECK に `'transfer'`、
//! `reference_type` の CHECK に `'transfer_record'` を足し、`transfer_records`・`transfer_items` を作る。
//! どの表も `inventory_movements` を外部キーで参照しないので `foreign_keys` は変えない。
//! 止める検査は持たない（CHECK に値を足すだけ。22 §21 手順 1）。1 つの TX。

use super::{migration_tx, DbError};
use rusqlite::{params, Connection, OptionalExtension};

/// 業務日の形の CHECK（§19・§20 と同じ）
const DATE_SHAPE: &str = "[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]";

/// 手順 5 の確かめの失敗の文（22 §21 手順 5 の vT の文。作り直しを指示しない）
const VERIFY_FAILED: &str = "在庫の記録を新しい形へ写せなかったので、この更新を取りやめ、この更新の変更は戻しました（同じ起動で先に済んだ別の更新は残ります）。起動し直しても同じ所で止まるときは、管理者に知らせてください。前の版のアプリで開けないときは、管理者が更新の前の backup に戻します（migration-recovery-runbook）";

/// 手順 3〜4 の作り直し（今の 13 列と §19・§20 の CHECK。2 つの CHECK の値の一覧の末尾に振替の値を足す）と、
/// 振替の記録の 2 表（transaction §11a-11b、DB_DESIGN のインデックス方針）
fn rebuild_sql() -> String {
    format!(
        "CREATE TABLE inventory_movements_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_code TEXT NOT NULL REFERENCES products(product_code),
    movement_type TEXT NOT NULL CHECK(movement_type IN ('sale_auto','sale_manual','receiving','return','disposal','stocktake','transfer')),
    quantity INTEGER NOT NULL,
    stock_after INTEGER NOT NULL,
    reference_type TEXT CHECK(reference_type IN ('csv_import','manual_sale','receiving_record','return_record','disposal_record','stocktake','transfer_record') OR reference_type IS NULL),
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
CREATE INDEX idx_inventory_movements_ledger ON inventory_movements(product_code, business_date, phase, settlement_no);
CREATE TABLE transfer_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transfer_date TEXT NOT NULL CHECK(length(transfer_date) = 10 AND transfer_date GLOB '{DATE_SHAPE}'),
    idempotency_key TEXT NOT NULL CHECK(length(idempotency_key) > 0),
    request_fingerprint TEXT NOT NULL CHECK(length(request_fingerprint) > 0),
    created_at TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_transfer_records_idempotency ON transfer_records(idempotency_key);
CREATE TABLE transfer_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transfer_record_id INTEGER NOT NULL REFERENCES transfer_records(id),
    product_code TEXT NOT NULL REFERENCES products(product_code),
    direction TEXT NOT NULL CHECK(direction IN ('out','in')),
    quantity INTEGER NOT NULL CHECK(quantity > 0)
);
CREATE UNIQUE INDEX idx_transfer_items_record_product ON transfer_items(transfer_record_id, product_code);"
    )
}

/// vT（MNT-03-D22）
pub(crate) fn apply_stock_transfer(conn: &Connection, version: i64) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    // 1. BEGIN（止める検査は置かない）
    conn.execute_batch("BEGIN;")
        .map_err(|e| DbError::MigrationFailed(format!("v{version} BEGIN失敗: {e}")))?;

    let result = (|| -> Result<(), DbError> {
        // 2. sqlite_sequence と、手順 5 で比べる行数と id の集合
        let sequence: Option<i64> = conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'inventory_movements'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let ids_before = movement_ids(conn)?;

        // 3〜4. 作り直しと 2 表。sequence を作り直しの前の値へ戻す
        conn.execute_batch(&rebuild_sql())?;
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
        #[cfg(test)]
        tests::run_after_rebuild_hook(conn);

        // 5. 行数と id の集合、DB 全体の foreign_key_check
        let ids_after = movement_ids(conn)?;
        if ids_after != ids_before {
            return Err(DbError::MigrationFailed(format!(
                "{VERIFY_FAILED}: inventory_movements（行数と id の集合: {} -> {}）",
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
                "{VERIFY_FAILED}: inventory_movements（foreign_key_check: {}）",
                fk_tables.join("、")
            )));
        }

        // 6. 版を記録
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (?1, ?2)",
            params![version, now],
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        // 手順 5 の文はそのまま出す（22 §21 手順 5。起動の画面が DatabaseInit の文の後に添える）
        let message = match error {
            DbError::MigrationFailed(message) => message,
            other => format!("v{version} 詰め替え・切り分けの種類の migration失敗: {other}"),
        };
        return Err(migration_tx::rollback_after_error(conn, message));
    }
    migration_tx::commit_transaction(conn, &format!("v{version} COMMIT失敗"))
}

fn movement_ids(conn: &Connection) -> Result<Vec<i64>, DbError> {
    let mut statement = conn.prepare("SELECT id FROM inventory_movements ORDER BY id")?;
    let ids = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

/// vT の版（22 §21）。merge の順で番号を取り直すときは file 名・`mod` の行・登録とここを直す
#[cfg(test)]
pub(crate) const VT_VERSION: i64 = 13;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::{self, migrate_up_to};
    use crate::db::migration_tx::{self as tx_helpers, FailurePoint};
    use crate::db::test_support::dump;
    use rusqlite::types::Value;
    use std::cell::Cell;

    thread_local! {
        /// 手順 5 の確かめを失敗させる test の hook（作り直しの直後に呼ぶ）
        static AFTER_REBUILD: Cell<Option<fn(&Connection)>> = const { Cell::new(None) };
    }

    pub(super) fn run_after_rebuild_hook(conn: &Connection) {
        if let Some(hook) = AFTER_REBUILD.with(Cell::get) {
            hook(conn);
        }
    }

    struct HookGuard;
    impl Drop for HookGuard {
        fn drop(&mut self) {
            AFTER_REBUILD.with(|hook| hook.set(None));
        }
    }
    fn set_after_rebuild(hook: fn(&Connection)) -> HookGuard {
        AFTER_REBUILD.with(|cell| cell.set(Some(hook)));
        HookGuard
    }

    /// 22 §21 手順 5 の vT の文（詳細の括弧の前まで。設計正本から写す）
    const SPEC_VERIFY_MESSAGE: &str = "在庫の記録を新しい形へ写せなかったので、この更新を取りやめ、この更新の変更は戻しました（同じ起動で先に済んだ別の更新は残ります）。起動し直しても同じ所で止まるときは、管理者に知らせてください。前の版のアプリで開けないときは、管理者が更新の前の backup に戻します（migration-recovery-runbook）";

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

    /// vT の直前の版の DB
    fn setup_before_vt() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("before-vt.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, VT_VERSION - 1).unwrap();
        assert_eq!(version(&conn), (VT_VERSION - 1, VT_VERSION - 1));
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

    #[allow(clippy::too_many_arguments)]
    fn seed_movement(
        conn: &Connection,
        code: &str,
        movement_type: &str,
        quantity: i64,
        stock_after: i64,
        reference: Option<(&str, i64)>,
        business_date: &str,
        settlement_no: Option<i64>,
    ) -> rusqlite::Result<i64> {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 reference_type, reference_id, note, created_at, business_date, phase, settlement_no)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, '合成 ' || ?2, ?7 || 'T09:00:00', ?7, ?8, ?9)",
            params![
                code,
                movement_type,
                quantity,
                stock_after,
                reference.map(|r| r.0),
                reference.map(|r| r.1),
                business_date,
                i64::from(settlement_no.is_some()),
                settlement_no
            ],
        )?;
        Ok(conn.last_insert_rowid())
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

    fn movements(conn: &Connection) -> Vec<Vec<Value>> {
        dump(conn, "SELECT * FROM inventory_movements ORDER BY id")
    }

    fn table_info(conn: &Connection) -> Vec<Vec<Value>> {
        dump(
            conn,
            "SELECT * FROM pragma_table_info('inventory_movements')",
        )
    }

    /// 表の名前ごとの全行（schema を除く）
    fn rows_by_table(conn: &Connection) -> std::collections::BTreeMap<String, Vec<Vec<Value>>> {
        dump(
            conn,
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        )
        .into_iter()
        .map(|row| match &row[0] {
            Value::Text(name) => (
                name.clone(),
                dump(conn, &format!("SELECT * FROM \"{name}\" ORDER BY 1")),
            ),
            other => panic!("{other:?}"),
        })
        .collect()
    }

    fn migration_failed(conn: &Connection) -> String {
        match migration::migrate(conn).unwrap_err() {
            DbError::MigrationFailed(message) => message,
            other => panic!("MigrationFailed を期待: {other:?}"),
        }
    }

    /// 入庫・返品交換・手動販売・廃棄・初期在庫・Z004（phase 1）の movement と、計数・棚卸しを持つ直前の版の DB
    fn seed_business_rows(conn: &Connection) {
        seed_product(conn, "VT-1");
        seed_product(conn, "VT-2");
        conn.execute_batch(
            "INSERT INTO receiving_records (id, receiving_date, idempotency_key, request_fingerprint, created_at)
                 VALUES (11, '2026-05-03', 'vt-rcv', 'fp', '2026-05-04T09:00:00');
             INSERT INTO return_records (id, return_type, return_date, register_processed,
                 idempotency_key, request_fingerprint, created_at)
                 VALUES (12, 'exchange', '2026-05-06', 0, 'vt-ret', 'fp', '2026-05-07T09:00:00');
             INSERT INTO manual_sales (id, sale_date, reason, idempotency_key, request_fingerprint, created_at)
                 VALUES (13, '2026-05-08', 'other', 'vt-ms', 'fp', '2026-05-09T09:00:00');
             INSERT INTO disposal_records (id, disposal_date, idempotency_key, request_fingerprint, created_at)
                 VALUES (14, '2026-05-10', 'vt-dsp', 'fp', '2026-05-11T09:00:00');
             INSERT INTO csv_imports (id, filename, settlement_date, file_hash, total_items, total_amount,
                 skipped_count, status, imported_at, machine_no, settlement_no)
                 VALUES (15, 'Z004.CSV', '2026-05-12', 'vt-hash', 1, 100, 0, 'completed',
                         '2026-05-12T20:00:00', '0001', 7);
             INSERT INTO stocktakes (id, started_at, status) VALUES (16, '2026-05-13T09:00:00', 'in_progress');
             INSERT INTO stocktake_items (stocktake_id, product_code, system_stock) VALUES (16, 'VT-1', 4);
             INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                 seen_movement_id, counted_at)
             VALUES ('VT-1', 5, 'fresh', '2026-05-13', 7, '2026-05-13T10:00:00'),
                    ('VT-2', 4, 'default', '2026-05-13', NULL, '2026-05-13T11:00:00');",
        )
        .unwrap();
        // 初期在庫（receiving・reference NULL）
        seed_movement(conn, "VT-1", "receiving", 10, 10, None, "2026-05-01", None).unwrap();
        seed_movement(
            conn,
            "VT-1",
            "receiving",
            3,
            13,
            Some(("receiving_record", 11)),
            "2026-05-03",
            None,
        )
        .unwrap();
        seed_movement(
            conn,
            "VT-1",
            "return",
            1,
            14,
            Some(("return_record", 12)),
            "2026-05-06",
            None,
        )
        .unwrap();
        seed_movement(
            conn,
            "VT-2",
            "return",
            -1,
            -1,
            Some(("return_record", 12)),
            "2026-05-06",
            None,
        )
        .unwrap();
        seed_movement(
            conn,
            "VT-1",
            "sale_manual",
            -2,
            12,
            Some(("manual_sale", 13)),
            "2026-05-08",
            None,
        )
        .unwrap();
        seed_movement(
            conn,
            "VT-1",
            "disposal",
            -1,
            11,
            Some(("disposal_record", 14)),
            "2026-05-10",
            None,
        )
        .unwrap();
        seed_movement(
            conn,
            "VT-1",
            "sale_auto",
            -1,
            10,
            Some(("csv_import", 15)),
            "2026-05-12",
            Some(7),
        )
        .unwrap();
        conn.execute(
            "UPDATE inventory_movements SET is_voided = 1 WHERE movement_type = 'sale_manual'",
            [],
        )
        .unwrap();
    }

    #[test]
    fn vt_stock_transfer_m1_copies_every_column_and_id_on_empty_and_previous_db() {
        // REQ-903 / MNT-03-D22 手順 3・4 / Test Plan M1: 空の DB と直前の版の DB の両方で、全列と id が同じ
        // 空の DB
        let (_dir, conn) = setup_before_vt();
        let columns_before = table_info(&conn);
        assert_eq!(columns_before.len(), 13, "22 §21 手順 3 の今の 13 列");
        migrate_up_to(&conn, VT_VERSION).unwrap();
        assert_eq!(version(&conn), (VT_VERSION, VT_VERSION));
        assert_eq!(table_info(&conn), columns_before, "列の名前・順・型・制約");
        assert!(movements(&conn).is_empty());

        // 直前の版の DB（行の各列が違う値を持つ）
        let (_dir, conn) = setup_before_vt();
        seed_business_rows(&conn);
        let rows_before = movements(&conn);
        assert_eq!(rows_before.len(), 7);
        let columns_before = table_info(&conn);
        migrate_up_to(&conn, VT_VERSION).unwrap();
        assert_eq!(version(&conn), (VT_VERSION, VT_VERSION));
        assert_eq!(table_info(&conn), columns_before);
        assert_eq!(movements(&conn), rows_before, "全列と id");
        assert!(dump(
            &conn,
            "SELECT name FROM sqlite_master WHERE name = 'inventory_movements_new'"
        )
        .is_empty());
    }

    #[test]
    fn vt_stock_transfer_m2_restores_sqlite_sequence_after_tail_delete() {
        // REQ-903 / MNT-03-D22 手順 4 / Test Plan M2: 末尾の行を消した fixture で、次の id が前の sequence + 1
        let (_dir, conn) = setup_before_vt();
        seed_product(&conn, "VT-1");
        seed_movement(&conn, "VT-1", "receiving", 4, 4, None, "2026-03-15", None).unwrap();
        let removed =
            seed_movement(&conn, "VT-1", "disposal", -9, -5, None, "2026-05-13", None).unwrap();
        conn.execute("DELETE FROM inventory_movements WHERE id = ?1", [removed])
            .unwrap();
        let sequence_before = sequence(&conn);
        assert_eq!(sequence_before, Some(removed));

        migrate_up_to(&conn, VT_VERSION).unwrap();

        assert_eq!(sequence(&conn), sequence_before);
        seed_movement(&conn, "VT-1", "receiving", 1, 5, None, "2026-06-01", None).unwrap();
        assert_eq!(
            conn.last_insert_rowid(),
            removed + 1,
            "消した id を使い回さない"
        );
    }

    #[test]
    fn vt_stock_transfer_m3_checks_accept_transfer_and_indexes_exist() {
        // REQ-903 / MNT-03-D22 手順 3・4 / Test Plan M3
        let (_dir, conn) = setup_before_vt();
        migrate_up_to(&conn, VT_VERSION).unwrap();
        seed_product(&conn, "VT-1");
        assert!(seed_movement(
            &conn,
            "VT-1",
            "transfer",
            -3,
            -3,
            Some(("transfer_record", 1)),
            "2026-06-01",
            None
        )
        .is_ok());
        let convert = seed_movement(&conn, "VT-1", "convert", 1, 1, None, "2026-06-01", None)
            .unwrap_err()
            .to_string();
        assert!(convert.contains("CHECK constraint failed"), "{convert}");
        let reference = seed_movement(
            &conn,
            "VT-1",
            "transfer",
            1,
            1,
            Some(("convert_record", 1)),
            "2026-06-01",
            None,
        )
        .unwrap_err()
        .to_string();
        assert!(reference.contains("CHECK constraint failed"), "{reference}");

        let text = |s: &str| Value::Text(s.into());
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
        // 2 表の UNIQUE の index（DB_DESIGN のインデックス方針）
        let unique = |index: &str| -> Vec<Vec<Value>> {
            dump(
                &conn,
                &format!(
                    "SELECT \"unique\" FROM pragma_index_list(
                         (SELECT tbl_name FROM sqlite_master WHERE name = '{index}'))
                     WHERE name = '{index}'"
                ),
            )
        };
        assert_eq!(
            index_columns("idx_transfer_records_idempotency"),
            names(&["idempotency_key"])
        );
        assert_eq!(
            unique("idx_transfer_records_idempotency"),
            vec![vec![Value::Integer(1)]]
        );
        assert_eq!(
            index_columns("idx_transfer_items_record_product"),
            names(&["transfer_record_id", "product_code"])
        );
        assert_eq!(
            unique("idx_transfer_items_record_product"),
            vec![vec![Value::Integer(1)]]
        );
    }

    #[test]
    fn vt_stock_transfer_m4_transfer_tables_reject_invalid_rows() {
        // REQ-903 / transaction §11a-11b / Test Plan M4
        let (_dir, conn) = setup_before_vt();
        migrate_up_to(&conn, VT_VERSION).unwrap();
        seed_product(&conn, "VT-1");
        seed_product(&conn, "VT-2");
        let record = |date: &str, key: &str| {
            conn.execute(
                "INSERT INTO transfer_records (transfer_date, idempotency_key, request_fingerprint, created_at)
                 VALUES (?1, ?2, 'fp', '2026-06-01T10:00:00')",
                params![date, key],
            )
        };
        assert!(record("2026-06-01", "vt-ok").is_ok());
        let record_id = conn.last_insert_rowid();
        for (date, key) in [
            ("2026-06-01", ""),
            ("2026-6-1", "vt-shape"),
            ("2026/06/01", "vt-slash"),
        ] {
            let error = record(date, key).unwrap_err().to_string();
            assert!(
                error.contains("CHECK constraint failed"),
                "{date} {key}: {error}"
            );
        }
        let item = |code: &str, direction: &str, quantity: i64| {
            conn.execute(
                "INSERT INTO transfer_items (transfer_record_id, product_code, direction, quantity)
                 VALUES (?1, ?2, ?3, ?4)",
                params![record_id, code, direction, quantity],
            )
        };
        assert!(item("VT-1", "out", 3).is_ok());
        for (code, direction, quantity) in [("VT-2", "x", 1), ("VT-2", "in", 0), ("VT-2", "in", -1)]
        {
            let error = item(code, direction, quantity).unwrap_err().to_string();
            assert!(
                error.contains("CHECK constraint failed"),
                "{direction} {quantity}: {error}"
            );
        }
        // 同じ記録の同じ商品の 2 行目（向きを問わない）
        let duplicate = item("VT-1", "in", 1).unwrap_err().to_string();
        assert!(
            duplicate.contains("UNIQUE constraint failed"),
            "{duplicate}"
        );
        assert!(item("VT-2", "in", 6).is_ok());
    }

    #[test]
    fn vt_stock_transfer_m5_verification_failure_rolls_back_with_spec_message() {
        // REQ-903 / MNT-03-D22 手順 5 / Test Plan M5: 確かめの失敗 2 つで schema・値・版が何も変わらず、
        // message は 22 §21 手順 5 の文（詳細の括弧の中を除く）と一字一句同じ
        let detail_of = |message: &str| -> String {
            let prefix = format!("{SPEC_VERIFY_MESSAGE}: inventory_movements（");
            assert!(message.starts_with(&prefix), "{message}");
            assert!(message.ends_with('）'), "{message}");
            message[prefix.len()..message.len() - '）'.len_utf8()].to_string()
        };
        for forbidden in [
            "作り直し",
            "DB は変わっていません",
            "前の版のアプリで開いて",
            "開発者に知らせて",
        ] {
            assert!(!SPEC_VERIFY_MESSAGE.contains(forbidden), "{forbidden}");
        }

        // (1) 行数と id の集合の不一致（作り直しの直後に 1 行消える）
        let (_dir, conn) = setup_before_vt();
        seed_product(&conn, "VT-1");
        seed_movement(&conn, "VT-1", "receiving", 5, 5, None, "2026-04-01", None).unwrap();
        seed_movement(&conn, "VT-1", "receiving", 1, 6, None, "2026-04-02", None).unwrap();
        let before = snapshot(&conn);
        let hook = set_after_rebuild(|conn| {
            conn.execute("DELETE FROM inventory_movements WHERE id = 1", [])
                .unwrap();
        });
        let message = migration_failed(&conn);
        drop(hook);
        assert_eq!(detail_of(&message), "行数と id の集合: 2 -> 1");
        assert_eq!(snapshot(&conn), before);
        assert_eq!(version(&conn), (VT_VERSION - 1, VT_VERSION - 1));
        assert!(conn.is_autocommit());

        // (2) ほかの表の FK の違反（vT の写しと関わりの無い表でも止める）
        let (_dir, conn) = setup_before_vt();
        seed_product(&conn, "VT-1");
        seed_movement(&conn, "VT-1", "receiving", 5, 5, None, "2026-04-01", None).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        conn.execute(
            "INSERT INTO sale_records (csv_import_id, product_code, sale_date, quantity, amount, source, created_at)
             VALUES (999, 'VT-1', '2026-04-01', 1, 100, 'auto', '2026-04-01T10:00:00')",
            [],
        )
        .unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        let before = snapshot(&conn);
        let message = migration_failed(&conn);
        assert_eq!(detail_of(&message), "foreign_key_check: sale_records");
        assert_eq!(snapshot(&conn), before);
        assert_eq!(version(&conn), (VT_VERSION - 1, VT_VERSION - 1));
        assert!(conn.is_autocommit());

        // 失敗を外せば同じ DB で当たる（直し方は code の修正で、作り直さない）
        let (_dir, conn) = setup_before_vt();
        seed_product(&conn, "VT-1");
        seed_movement(&conn, "VT-1", "receiving", 5, 5, None, "2026-04-01", None).unwrap();
        let failures = tx_helpers::fail_operations(&[FailurePoint::Commit]);
        let message = migration_failed(&conn);
        drop(failures);
        assert!(
            message.contains(&format!("v{VT_VERSION} COMMIT失敗")),
            "{message}"
        );
        assert_eq!(version(&conn), (VT_VERSION - 1, VT_VERSION - 1));
        migrate_up_to(&conn, VT_VERSION).unwrap();
        assert_eq!(version(&conn), (VT_VERSION, VT_VERSION));
    }

    #[test]
    fn vt_stock_transfer_m6_business_db_with_z004_and_counts_migrates_unchanged() {
        // REQ-903 / MNT-03-D22 手順 1（止める検査を置かない）/ Test Plan M6: 値のある DB で止まらず、写しも変わらない
        let (_dir, conn) = setup_before_vt();
        seed_business_rows(&conn);
        conn.execute(
            "UPDATE products SET stock_quantity = 10, stock_recheck = 1, stock_range_low = 9,
                 stock_range_high = 10 WHERE product_code = 'VT-1'",
            [],
        )
        .unwrap();
        let before = rows_by_table(&conn);

        migrate_up_to(&conn, VT_VERSION).unwrap();

        assert_eq!(version(&conn), (VT_VERSION, VT_VERSION));
        let mut after = rows_by_table(&conn);
        // 新しい 2 表は空、schema_versions は版の行が 1 つ増える。ほかの全表の全行は同じ
        assert!(after.remove("transfer_records").unwrap().is_empty());
        assert!(after.remove("transfer_items").unwrap().is_empty());
        let versions_after = after.remove("schema_versions").unwrap();
        let mut before = before;
        let versions_before = before.remove("schema_versions").unwrap();
        assert_eq!(versions_after.len(), versions_before.len() + 1);
        assert_eq!(
            &versions_after[..versions_before.len()],
            versions_before.as_slice()
        );
        assert_eq!(after, before);
        let stock: (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
                 FROM products WHERE product_code = 'VT-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(stock, (10, 1, 9, 10));
    }
}
