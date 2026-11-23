//! migration v15（vC）: 入出庫の記録の取消の列と、手動販売の売上の行の結び（D-163）
//!
//! docs/function-design/22-mnt-migration.md §23 / MNT-03-D24 に基づく実装。
//! 列の意味の正本は docs/db-design/transaction-tables.md §4-5 の「取消の列」と pos-tables.md §13。
//! NULL を許す列と index を足すだけで、既存の行の値を変えない。

/// vC マイグレーション: 5 つの header の `canceled_at`・`cancel_reason` と `sale_records.manual_sale_id`
pub(crate) fn get_v15_record_cancel_schema() -> &'static str {
    "ALTER TABLE receiving_records ADD COLUMN canceled_at TEXT;
ALTER TABLE receiving_records ADD COLUMN cancel_reason TEXT CHECK ((canceled_at IS NULL) = (cancel_reason IS NULL) AND (cancel_reason IS NULL OR length(trim(cancel_reason)) > 0));
ALTER TABLE return_records ADD COLUMN canceled_at TEXT;
ALTER TABLE return_records ADD COLUMN cancel_reason TEXT CHECK ((canceled_at IS NULL) = (cancel_reason IS NULL) AND (cancel_reason IS NULL OR length(trim(cancel_reason)) > 0));
ALTER TABLE manual_sales ADD COLUMN canceled_at TEXT;
ALTER TABLE manual_sales ADD COLUMN cancel_reason TEXT CHECK ((canceled_at IS NULL) = (cancel_reason IS NULL) AND (cancel_reason IS NULL OR length(trim(cancel_reason)) > 0));
ALTER TABLE disposal_records ADD COLUMN canceled_at TEXT;
ALTER TABLE disposal_records ADD COLUMN cancel_reason TEXT CHECK ((canceled_at IS NULL) = (cancel_reason IS NULL) AND (cancel_reason IS NULL OR length(trim(cancel_reason)) > 0));
ALTER TABLE transfer_records ADD COLUMN canceled_at TEXT;
ALTER TABLE transfer_records ADD COLUMN cancel_reason TEXT CHECK ((canceled_at IS NULL) = (cancel_reason IS NULL) AND (cancel_reason IS NULL OR length(trim(cancel_reason)) > 0));
ALTER TABLE sale_records ADD COLUMN manual_sale_id INTEGER REFERENCES manual_sales(id) CHECK (manual_sale_id IS NULL OR (source = 'manual' AND csv_import_id IS NULL));
CREATE INDEX idx_sale_records_manual_sale_id ON sale_records(manual_sale_id);"
}

#[cfg(test)]
mod tests {
    use crate::db::migration::migrate_up_to;
    use crate::db::test_support::{dump, seed_product};
    use rusqlite::Connection;

    const VC_VERSION: i64 = 15;
    const HEADERS: [&str; 5] = [
        "receiving_records",
        "return_records",
        "manual_sales",
        "disposal_records",
        "transfer_records",
    ];

    fn open(dir: &tempfile::TempDir, last_version: i64) -> Connection {
        let conn = Connection::open(dir.path().join("vc.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, last_version).unwrap();
        conn
    }

    /// 直前の版の DB に、5 つの記録・手動販売の売上の行・Z004 の売上の行を 1 つずつ入れる（合成の値）
    fn seed_rows(conn: &Connection) {
        seed_product(conn, "VC-001");
        conn.execute_batch(
            "INSERT INTO receiving_records (receiving_date, idempotency_key, request_fingerprint, created_at)
                 VALUES ('2026-11-18', 'k-r', 'fp', '2026-11-18T10:00:00');
             INSERT INTO return_records (return_type, return_date, idempotency_key, request_fingerprint, created_at)
                 VALUES ('return', '2026-11-18', 'k-ret', 'fp', '2026-11-18T10:00:00');
             INSERT INTO manual_sales (sale_date, reason, idempotency_key, request_fingerprint, created_at)
                 VALUES ('2026-11-18', 'other', 'k-ms', 'fp', '2026-11-18T10:00:00');
             INSERT INTO disposal_records (disposal_date, idempotency_key, request_fingerprint, created_at)
                 VALUES ('2026-11-18', 'k-d', 'fp', '2026-11-18T10:00:00');
             INSERT INTO transfer_records (transfer_date, idempotency_key, request_fingerprint, created_at)
                 VALUES ('2026-11-18', 'k-t', 'fp', '2026-11-18T10:00:00');
             INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount,
                     skipped_count, status, imported_at, machine_no, settlement_no)
                 VALUES ('Z004.CSV', '2026-11-18', 'hash', 1, 500, 0, 'completed', '2026-11-18T20:00:00', '0001', 1);
             INSERT INTO sale_records (csv_import_id, product_code, sale_date, quantity, amount, source, created_at)
                 VALUES (NULL, 'VC-001', '2026-11-18', 1, 500, 'manual', '2026-11-18T10:00:00'),
                        (1, 'VC-001', '2026-11-18', 1, 500, 'auto', '2026-11-18T20:00:00');",
        )
        .unwrap();
    }

    fn cancel_sql(table: &str, canceled_at: &str, reason: &str) -> String {
        format!("UPDATE {table} SET canceled_at = {canceled_at}, cancel_reason = {reason}")
    }

    #[test]
    fn migration_v15_adds_cancel_columns_and_manual_sale_link() {
        // REQ-208 / 22 §23 MNT-03-D24 / transaction §4-5・POS §13（Test Plan T9）
        // 空の DB でも当たる
        let empty = tempfile::tempdir().unwrap();
        open(&empty, VC_VERSION);

        let dir = tempfile::tempdir().unwrap();
        let conn = open(&dir, VC_VERSION - 1);
        seed_rows(&conn);
        let tables = HEADERS.iter().copied().chain(["sale_records"]);
        let before: Vec<_> = tables
            .clone()
            .map(|table| dump(&conn, &format!("SELECT * FROM {table} ORDER BY id")))
            .collect();

        migrate_up_to(&conn, VC_VERSION).unwrap();

        // 既存の行は値と行数が変わらず、新しい列は全て NULL（末尾に足した列を除いて前と同じ）
        for (table, rows_before) in tables.zip(&before) {
            let added = if table == "sale_records" { 1 } else { 2 };
            let rows_after = dump(&conn, &format!("SELECT * FROM {table} ORDER BY id"));
            assert_eq!(rows_after.len(), rows_before.len(), "{table}");
            for (after, row_before) in rows_after.iter().zip(rows_before) {
                let (kept, new) = after.split_at(after.len() - added);
                assert_eq!(kept, row_before.as_slice(), "{table}");
                assert!(
                    new.iter()
                        .all(|value| *value == rusqlite::types::Value::Null),
                    "{table}"
                );
            }
        }
        for table in HEADERS {
            let columns: Vec<(String, String, bool)> = conn
                .prepare(&format!(
                    "SELECT name, type, \"notnull\" FROM pragma_table_info('{table}')
                     WHERE name IN ('canceled_at', 'cancel_reason') ORDER BY cid"
                ))
                .unwrap()
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            assert_eq!(
                columns,
                vec![
                    ("canceled_at".to_string(), "TEXT".to_string(), false),
                    ("cancel_reason".to_string(), "TEXT".to_string(), false),
                ],
                "{table}"
            );
            // 片方だけ・半角空白だけの理由は拒み、両方ありは通る（全角空白・tab は DB を通り、BIZ が拒む）
            for (canceled_at, reason) in [
                ("'2026-11-21T10:00:00'", "NULL"),
                ("NULL", "'打ち間違い'"),
                ("'2026-11-21T10:00:00'", "'  '"),
            ] {
                assert!(
                    conn.execute(&cancel_sql(table, canceled_at, reason), [])
                        .is_err(),
                    "{table}: {canceled_at} / {reason}"
                );
            }
            conn.execute(
                &cancel_sql(table, "'2026-11-21T10:00:00'", "'打ち間違い'"),
                [],
            )
            .unwrap();
        }

        let index: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_index_list('sale_records')
                 WHERE name = 'idx_sale_records_manual_sale_id'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index, 1);
        let link = |source: &str, manual_sale_id: i64| {
            conn.execute(
                "UPDATE sale_records SET manual_sale_id = ?2 WHERE source = ?1",
                rusqlite::params![source, manual_sale_id],
            )
        };
        // Z004 の行（source='auto'）と、無い手動販売の ID は拒む
        assert!(link("auto", 1).is_err());
        assert!(link("manual", 999).is_err());
        assert_eq!(link("manual", 1).unwrap(), 1);
    }
}
