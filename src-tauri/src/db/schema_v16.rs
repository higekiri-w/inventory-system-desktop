//! migration vP: `plu_slots` の作り直し（D-117 (1)。status を 4 値にし、`released_at` を外す）
//!
//! docs/function-design/22-mnt-migration.md §24 / MNT-03-D25、docs/db-design/plu-tables.md の
//! 「migration（D-117）」に基づく実装。形は 22 §21 の vT と同じ（1 つの TX で全行を写し、同じ TX で確かめる）。
//! `plu_slots` を外部キーで参照する表は無いので `foreign_keys` は変えない。

use super::{migration_tx, DbError};
use rusqlite::{params, Connection};

/// 手順 5 の確かめの失敗の文（22 §24 手順 5。作り直しを指示しない）
const VERIFY_FAILED: &str = "レジの PLU の枠の記録を新しい形へ写せなかったので、この更新を取りやめ、この更新の変更は戻しました（同じ起動で先に済んだ別の更新は残ります）。起動し直しても同じ所で止まるときは、管理者に知らせてください。前の版のアプリで開けないときは、管理者が更新の前の backup に戻します（migration-recovery-runbook）";

/// 手順 3〜4。status の対応づけは写しの `CASE` で行う（旧い表を `UPDATE` すると、旧い partial UNIQUE が
/// `external` を含むので、同じ JAN の `release_pending` と `external` / `active` の組で UNIQUE 違反になる）
const REBUILD_SQL: &str = "
CREATE TABLE plu_slots_new (
    memory_no INTEGER PRIMARY KEY CHECK(memory_no BETWEEN 217 AND 5000),
    scanning_code TEXT,
    status TEXT NOT NULL CHECK(status IN ('free','external','reserved','active')),
    reserved_at TEXT,
    activated_at TEXT,
    updated_at TEXT NOT NULL
);
INSERT INTO plu_slots_new (memory_no, scanning_code, status, reserved_at, activated_at, updated_at)
SELECT memory_no, scanning_code,
       CASE WHEN status = 'release_pending' THEN 'external' ELSE status END,
       reserved_at, activated_at, updated_at
FROM plu_slots;
DROP TABLE plu_slots;
ALTER TABLE plu_slots_new RENAME TO plu_slots;
CREATE UNIQUE INDEX idx_plu_slots_live_scanning_code
ON plu_slots(scanning_code)
WHERE scanning_code IS NOT NULL
  AND status IN ('reserved','active');
";

/// 手順 6 で消す key。`plu_register_snapshot_summary` はもう書かない・読まない。
/// `plu_register_snapshot_at` は、前の版の DB の照合日時で書出しの gate を通さないために消す（33 §16.8）
const DROPPED_SETTING_KEYS: [&str; 2] =
    ["plu_register_snapshot_summary", "plu_register_snapshot_at"];

/// memory_no・scanning_code・status・reserved_at・activated_at・updated_at
type SlotRow = (
    i64,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    String,
);

fn slot_rows(conn: &Connection) -> Result<Vec<SlotRow>, DbError> {
    let mut statement = conn.prepare(
        "SELECT memory_no, scanning_code, status, reserved_at, activated_at, updated_at
         FROM plu_slots ORDER BY memory_no",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// vP（MNT-03-D25）
pub(crate) fn apply_plu_slots_rebuild(conn: &Connection, version: i64) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    // 1. BEGIN（止める検査は置かない）
    conn.execute_batch("BEGIN;")
        .map_err(|e| DbError::MigrationFailed(format!("v{version} BEGIN失敗: {e}")))?;

    let result = (|| -> Result<(), DbError> {
        // 2. 作り直しの前の行。期待する status は写しの SQL と別に、ここで対応づける
        let mut expected = slot_rows(conn)?;
        for row in &mut expected {
            if row.2 == "release_pending" {
                row.2 = "external".to_string();
            }
        }

        // 3〜4. 作り直し
        conn.execute_batch(REBUILD_SQL)?;
        #[cfg(test)]
        tests::run_after_rebuild_hook(conn);

        // 5. memory_no ごとに全部の列を NULL を含めて比べる
        let actual = slot_rows(conn)?;
        if actual != expected {
            let differing = expected
                .iter()
                .zip(&actual)
                .filter(|(before, after)| before != after)
                .count();
            return Err(DbError::MigrationFailed(format!(
                "{VERIFY_FAILED}: plu_slots（行数: {} -> {}、値の違う枠: {differing}）",
                expected.len(),
                actual.len()
            )));
        }

        // 6. 使わなくなった key を消す
        for key in DROPPED_SETTING_KEYS {
            conn.execute("DELETE FROM app_settings WHERE key = ?1", [key])?;
        }

        // 7. 版を記録
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (?1, ?2)",
            params![version, now],
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        // 手順 5 の文はそのまま出す（起動の画面が DatabaseInit の文の後に添える）
        let message = match error {
            DbError::MigrationFailed(message) => message,
            other => format!("v{version} PLU の枠の作り直しの migration失敗: {other}"),
        };
        return Err(migration_tx::rollback_after_error(conn, message));
    }
    migration_tx::commit_transaction(conn, &format!("v{version} COMMIT失敗"))
}

/// vP の版（22 §24）。merge の順で番号を取り直すときは file 名・`mod` の行・登録とここを直す
#[cfg(test)]
pub(crate) const VP_VERSION: i64 = 16;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::{self, migrate_up_to};
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

    const JAN_A: &str = "4901234567894";
    const JAN_B: &str = "4901234567887";
    const JAN_C: &str = "4901234567870";

    /// vP の直前の版の DB に、5 つの status の行を置く。同じ JAN の `release_pending` と `active`（217・218）、
    /// 同じ JAN の `release_pending` と `external`（219・220）は v5 の partial UNIQUE の下で保存できる 2 組
    fn setup_before_vp() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("before-vp.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, VP_VERSION - 1).unwrap();
        // (memory_no, scanning_code, status, reserved_at, activated_at, released_at)
        let rows = [
            (217, Some(JAN_A), "active", Some("r217"), Some("a217"), None),
            (
                218,
                Some(JAN_A),
                "release_pending",
                None,
                Some("a218"),
                None,
            ),
            (219, Some(JAN_B), "external", None, None, None),
            (
                220,
                Some(JAN_B),
                "release_pending",
                Some("r220"),
                Some("a220"),
                None,
            ),
            (221, None, "free", None, None, Some("x221")),
            (222, Some(JAN_C), "reserved", Some("r222"), None, None),
            (5_000, Some("12345678EEEEEE"), "external", None, None, None),
        ];
        for (memory_no, code, status, reserved_at, activated_at, released_at) in rows {
            conn.execute(
                "UPDATE plu_slots SET scanning_code=?2, status=?3, reserved_at=?4, activated_at=?5,
                        released_at=?6, updated_at='u' || ?1
                 WHERE memory_no=?1",
                params![
                    memory_no,
                    code,
                    status,
                    reserved_at,
                    activated_at,
                    released_at
                ],
            )
            .unwrap();
        }
        conn.execute_batch(
            "INSERT INTO app_settings (key, value, updated_at) VALUES
                 ('plu_register_snapshot_summary', '{\"conflict_count\":1}', 'u'),
                 ('plu_register_snapshot_at', '2026-11-01T10:00:00', 'u');",
        )
        .unwrap();
        (dir, conn)
    }

    fn setting_keys(conn: &Connection) -> Vec<String> {
        dump(conn, "SELECT key FROM app_settings ORDER BY key")
            .into_iter()
            .map(|row| match &row[0] {
                Value::Text(key) => key.clone(),
                other => panic!("{other:?}"),
            })
            .collect()
    }

    /// schema（全 object の定義）と全表の全行
    fn snapshot(conn: &Connection) -> Vec<Vec<Vec<Value>>> {
        let mut out = vec![dump(
            conn,
            "SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name",
        )];
        let tables = dump(
            conn,
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        );
        for row in tables {
            let Value::Text(table) = &row[0] else {
                panic!("{row:?}")
            };
            out.push(dump(conn, &format!("SELECT * FROM \"{table}\" ORDER BY 1")));
        }
        out
    }

    #[test]
    fn test_migration_req907_plu_slots_rebuild_maps_release_pending_to_external() {
        // REQ-907 / MNT-03-D25 / AC1: 期待値は migration の前の行から test が独立に作る
        let (_dir, conn) = setup_before_vp();
        let before = dump(
            &conn,
            "SELECT memory_no, scanning_code, status, reserved_at, activated_at, updated_at
             FROM plu_slots ORDER BY memory_no",
        );
        let expected: Vec<Vec<Value>> = before
            .into_iter()
            .map(|mut row| {
                if row[2] == Value::Text("release_pending".to_string()) {
                    row[2] = Value::Text("external".to_string());
                }
                row
            })
            .collect();
        assert_eq!(expected.len(), 4_784);

        migration::migrate(&conn).unwrap();

        let after = dump(&conn, "SELECT * FROM plu_slots ORDER BY memory_no");
        assert_eq!(after, expected);
        let status_of = |memory_no: usize| after[memory_no - 217][2].clone();
        assert_eq!(status_of(218), Value::Text("external".to_string()));
        assert_eq!(status_of(220), Value::Text("external".to_string()));
        assert_eq!(status_of(217), Value::Text("active".to_string()));
        assert_eq!(status_of(222), Value::Text("reserved".to_string()));
        let columns: Vec<Value> = dump(&conn, "SELECT name FROM pragma_table_info('plu_slots')")
            .into_iter()
            .map(|mut row| row.remove(0))
            .collect();
        assert!(!columns.contains(&Value::Text("released_at".to_string())));
        assert_eq!(columns.len(), 6);
    }

    #[test]
    fn test_migration_req907_plu_slots_rebuild_unique_only_reserved_active() {
        // REQ-907 / plu-tables の partial UNIQUE: external は重複でき、reserved / active の同じ JAN の 2 行目は拒む
        let (_dir, conn) = setup_before_vp();
        migration::migrate(&conn).unwrap();
        let set = |memory_no: i64, code: &str, status: &str| {
            conn.execute(
                "UPDATE plu_slots SET scanning_code=?2, status=?3 WHERE memory_no=?1",
                params![memory_no, code, status],
            )
        };
        set(300, "D", "external").unwrap();
        set(301, "D", "external").unwrap();
        set(302, "D", "active").unwrap();
        assert!(set(303, "D", "active").is_err());
        assert!(set(303, "D", "reserved").is_err());
        set(304, "E", "reserved").unwrap();
        assert!(set(305, "E", "reserved").is_err());
        assert!(set(305, "E", "release_pending").is_err());
    }

    #[test]
    fn test_migration_req907_plu_slots_rebuild_verify_failure_rolls_back() {
        // REQ-907 / MNT-03-D25 手順 5: 写しの後に 2 枠の JAN が入れ替わると失敗し、何も残らない
        let (_dir, conn) = setup_before_vp();
        let before = snapshot(&conn);
        let _guard = set_after_rebuild(|conn| {
            conn.execute_batch(
                "UPDATE plu_slots SET scanning_code = NULL WHERE memory_no = 217;
                 UPDATE plu_slots SET scanning_code = '4901234567894' WHERE memory_no = 222;
                 UPDATE plu_slots SET scanning_code = '4901234567870' WHERE memory_no = 217;",
            )
            .unwrap();
        });

        let message = match migration::migrate(&conn).unwrap_err() {
            DbError::MigrationFailed(message) => message,
            other => panic!("MigrationFailed を期待: {other:?}"),
        };

        assert!(message.starts_with(VERIFY_FAILED), "{message}");
        assert!(message.contains("値の違う枠: 2"), "{message}");
        assert!(!message.contains(JAN_A) && !message.contains(JAN_C));
        assert_eq!(snapshot(&conn), before);
        assert!(setting_keys(&conn).contains(&"plu_register_snapshot_summary".to_string()));
        assert!(conn.is_autocommit());
    }

    #[test]
    fn test_migration_req907_plu_slots_rebuild_drops_summary_key() {
        // REQ-907 / plu-tables の手順 4: 2 つの key だけを消す
        let (_dir, conn) = setup_before_vp();
        let before = setting_keys(&conn);
        migration::migrate(&conn).unwrap();
        let expected: Vec<String> = before
            .into_iter()
            .filter(|key| {
                key != "plu_register_snapshot_summary" && key != "plu_register_snapshot_at"
            })
            .collect();
        assert!(!expected.is_empty());
        assert_eq!(setting_keys(&conn), expected);
    }
}
