//! migration v12（vS）: 日報の束の精算回数の列（D-111）
//!
//! docs/function-design/22-mnt-migration.md §17 / MNT-03-D15 に基づく実装。
//! NULL を許す列を足すだけで、既存の行の値を変えない（D-111 より前の取込みは NULL のまま）。

/// vS マイグレーション: `daily_report_imports.settlement_no` を足す
pub(crate) fn get_v12_daily_report_settlement_no_schema() -> &'static str {
    "ALTER TABLE daily_report_imports ADD COLUMN settlement_no INTEGER;"
}

#[cfg(test)]
mod tests {
    use crate::db::migration::{self, migrate_up_to};
    use rusqlite::Connection;

    const VS_VERSION: i64 = 12;

    fn version(conn: &Connection) -> (i64, i64) {
        conn.query_row(
            "SELECT MAX(version), COUNT(*) FROM schema_versions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    #[test]
    fn test_req401_adds_daily_report_settlement_no_nullable() {
        // REQ-401 / 22 §17 MNT-03-D15: 日報の行がある直前の版の DB に NULL を許す INTEGER の列を足す
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("before-vs.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, VS_VERSION - 1).unwrap();
        conn.execute(
            "INSERT INTO daily_report_imports (report_date, source_adapter, bundle_hash,
                 source_files_json, gross_amount, net_amount, status, imported_at)
             VALUES ('2026-10-05', 'casio_sr_s4000', 'H-old', '[]', 1, 1, 'completed', '2026-10-05T10:00:00')",
            [],
        )
        .unwrap();

        migrate_up_to(&conn, VS_VERSION).unwrap();
        assert_eq!(version(&conn), (VS_VERSION, VS_VERSION));
        let existing: Option<i64> = conn
            .query_row(
                "SELECT settlement_no FROM daily_report_imports WHERE bundle_hash = 'H-old'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(existing, None);
        let column: (String, i64) = conn
            .query_row(
                "SELECT type, \"notnull\" FROM pragma_table_info('daily_report_imports')
                 WHERE name = 'settlement_no'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(column, ("INTEGER".to_string(), 0));

        // 再実行で重複して適用しない
        migration::migrate(&conn).unwrap();
        migration::migrate(&conn).unwrap();
        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_versions WHERE version = ?1",
                [VS_VERSION],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(applied, 1);

        conn.execute(
            "INSERT INTO daily_report_imports (report_date, source_adapter, bundle_hash,
                 source_files_json, gross_amount, net_amount, status, imported_at, settlement_no)
             VALUES ('2026-10-06', 'casio_sr_s4000', 'H-new', '[]', 1, 1, 'completed', '2026-10-06T10:00:00', 5)",
            [],
        )
        .unwrap();
        let stored: i64 = conn
            .query_row(
                "SELECT settlement_no FROM daily_report_imports WHERE bundle_hash = 'H-new'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, 5);
    }
}
