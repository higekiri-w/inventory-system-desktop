//! migration v14: EJ の記録の文の表（`ej_records`、D-121・BIZ-08-D9）
//!
//! docs/function-design/22-mnt-migration.md §22 / MNT-03-D23 に基づく実装。
//! 列と CHECK の正本は docs/db-design/pos-tables.md の「SD の一括取込みと EJ の控えの契約」。
//! 表を足すだけで既存の行を変えない。

/// v14 マイグレーション: `ej_records` と index
pub(crate) fn get_v14_ej_records_schema() -> &'static str {
    "CREATE TABLE ej_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    machine_prefix TEXT NOT NULL CHECK(length(machine_prefix) = 4),
    record_no TEXT NOT NULL CHECK(length(record_no) = 6),
    printed_at TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('normal','return','training','settlement','inspection','program','program1','program3','off','unrecognized')),
    kind TEXT NOT NULL CHECK(kind IN ('sale','cancelled','cash_movement','receipt','settlement','inspection','settings','training','unclassified')),
    settlement_report TEXT CHECK(settlement_report IN ('daily','plu','attendance')),
    settlement_completed INTEGER CHECK(settlement_completed IN (0,1)),
    total_amount INTEGER,
    body_text TEXT NOT NULL,
    imported_at TEXT NOT NULL,
    UNIQUE(machine_prefix, record_no)
);
CREATE INDEX idx_ej_records_printed_at ON ej_records(printed_at);"
}

#[cfg(test)]
mod tests {
    use crate::db::migration::migrate_up_to;
    use rusqlite::Connection;

    const V14: i64 = 14;

    fn migrated() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("v14.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, V14).unwrap();
        (dir, conn)
    }

    fn insert(conn: &Connection, prefix: &str, no: &str, mode: &str) -> rusqlite::Result<usize> {
        conn.execute(
            "INSERT INTO ej_records (machine_prefix, record_no, printed_at, mode, kind, body_text, imported_at)
             VALUES (?1, ?2, '2030-01-02 10:15', ?3, 'sale', 'x', '2030-01-02T10:20:00')",
            rusqlite::params![prefix, no, mode],
        )
    }

    #[test]
    fn test_req401_v14_creates_ej_records_with_index_and_unique() {
        // REQ-401 / 22 §22 MNT-03-D23（Test Plan M1）: 列・printed_at の index・UNIQUE の index がある
        let (_dir, conn) = migrated();
        let columns: Vec<(String, String, bool)> = conn
            .prepare(
                "SELECT name, type, \"notnull\" FROM pragma_table_info('ej_records') ORDER BY cid",
            )
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let expected = [
            ("id", "INTEGER", false),
            ("machine_prefix", "TEXT", true),
            ("record_no", "TEXT", true),
            ("printed_at", "TEXT", true),
            ("mode", "TEXT", true),
            ("kind", "TEXT", true),
            ("settlement_report", "TEXT", false),
            ("settlement_completed", "INTEGER", false),
            ("total_amount", "INTEGER", false),
            ("body_text", "TEXT", true),
            ("imported_at", "TEXT", true),
        ];
        assert_eq!(
            columns,
            expected
                .iter()
                .map(|(name, ty, notnull)| (name.to_string(), ty.to_string(), *notnull))
                .collect::<Vec<_>>()
        );
        let indexes: Vec<(String, bool)> = conn
            .prepare("SELECT name, \"unique\" FROM pragma_index_list('ej_records') ORDER BY name")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(indexes.contains(&("idx_ej_records_printed_at".to_string(), false)));
        assert_eq!(indexes.iter().filter(|(_, unique)| *unique).count(), 1);
    }

    #[test]
    fn test_req401_v14_rejects_bad_values_and_duplicate_key() {
        // REQ-401 / pos-tables の ej_records（Test Plan M2）: CHECK と UNIQUE(machine_prefix, record_no)
        let (_dir, conn) = migrated();
        assert!(insert(&conn, "0001", "000101", "x").is_err());
        assert!(insert(&conn, "0001", "12345", "normal").is_err());
        assert!(insert(&conn, "001", "000101", "normal").is_err());
        insert(&conn, "0001", "000101", "normal").unwrap();
        assert!(insert(&conn, "0001", "000101", "normal").is_err());
        insert(&conn, "0002", "000101", "normal").unwrap();
        let bad_kind = conn.execute(
            "INSERT INTO ej_records (machine_prefix, record_no, printed_at, mode, kind, body_text, imported_at)
             VALUES ('0001', '000102', '2030-01-02 10:15', 'normal', 'x', 'x', 'x')",
            [],
        );
        assert!(bad_kind.is_err());
        let bad_report = conn.execute(
            "INSERT INTO ej_records (machine_prefix, record_no, printed_at, mode, kind, settlement_report, body_text, imported_at)
             VALUES ('0001', '000103', '2030-01-02 10:15', 'settlement', 'settlement', 'x', 'x', 'x')",
            [],
        );
        assert!(bad_report.is_err());
    }
}
