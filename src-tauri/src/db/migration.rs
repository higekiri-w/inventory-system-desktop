//! DBスキーマのマイグレーション実行（MNT-03）
//!
//! 22-mnt-migration.md §3 + §9 に基づく実装。
//! schema_versionsテーブルでバージョン管理し、未適用のマイグレーションを順番に実行する。
//!
//! ## MigrationKind
//! - Sql: 実行器が BEGIN→SQL→schema_versions INSERT→COMMIT を管理
//! - Custom: 関数側が TX管理 + schema_versions INSERT まで全責任を持つ
//!   （PRAGMA操作等でTX外処理が必要な場合用）

use super::migration_tx;
use super::schema_v1;
use super::schema_v10;
use super::schema_v11;
use super::schema_v12;
use super::schema_v2;
use super::schema_v3;
use super::schema_v4;
use super::schema_v5;
use super::schema_v6;
use super::schema_v7;
use super::schema_v8;
use super::schema_v9;
use super::DbError;
use rusqlite::Connection;

/// マイグレーション種別
enum MigrationKind {
    /// 実行器側が BEGIN→SQL→schema_versions INSERT→COMMIT を管理
    Sql(&'static str),
    /// 関数側が TX管理 + schema_versions INSERT まで全責任を持つ
    Custom(fn(&Connection, i64) -> Result<(), DbError>),
}

/// マイグレーション定義
struct Migration {
    version: i64,
    description: &'static str,
    kind: MigrationKind,
}

/// マイグレーション一覧（バージョン順）
fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "初期スキーマ（20テーブル + インデックス + 初期データ）",
            kind: MigrationKind::Sql(schema_v1::get_initial_schema()),
        },
        Migration {
            version: 2,
            description: "冪等性カラム追加（4テーブル再作成）",
            kind: MigrationKind::Custom(schema_v2::apply_v2_idempotency),
        },
        Migration {
            version: 3,
            description: "PLU対象フラグ追加（products.plu_target）",
            kind: MigrationKind::Custom(schema_v3::apply_v3_plu_target),
        },
        Migration {
            version: 4,
            description: "日報取込みテーブル追加（daily_report_imports + lines）",
            kind: MigrationKind::Sql(schema_v4::get_v4_daily_report_schema()),
        },
        Migration {
            version: 5,
            description: "PLU slot永続割当（plu_slots + 事前投入）",
            kind: MigrationKind::Custom(schema_v5::apply_v5_plu_slots),
        },
        Migration {
            version: 6,
            description: "suppliers.updated_at 追加",
            kind: MigrationKind::Custom(schema_v6::apply_v6_supplier_updated_at),
        },
        Migration {
            version: 7,
            description: "日報の個数を100倍の整数へ",
            kind: MigrationKind::Custom(schema_v7::apply_v7_daily_report_quantity_hundredths),
        },
        Migration {
            version: 8,
            description: "単位を12種へ（products.stock_unit の CHECK、保存済みの値の上限の検査）",
            kind: MigrationKind::Custom(schema_v8::apply_v8_stock_units),
        },
        Migration {
            version: 9,
            description: "在庫の並びの鍵・計数の表・写しの列（D-118 (a)）",
            kind: MigrationKind::Custom(schema_v9::apply_v9_stock_ledger),
        },
        Migration {
            version: 10,
            description: "原価を1/100円の列へ、単位から巻・cmを外す（作り直しの起点）",
            kind: MigrationKind::Custom(schema_v10::apply_v10_cost_centi_origin),
        },
        Migration {
            version: 11,
            description: "Z004の精算の鍵・売上の連動・棚卸しの完了の列（D-118 (b1)）",
            kind: MigrationKind::Custom(schema_v11::apply_v11_z004_settlement_key),
        },
        Migration {
            version: 12,
            description: "日報の束の精算回数の列（daily_report_imports.settlement_no）",
            kind: MigrationKind::Sql(schema_v12::get_v12_daily_report_settlement_no_schema()),
        },
    ]
}

/// アプリが扱える最大の schema 版（`migrations()` から導く）
pub(crate) fn app_max_version() -> i64 {
    migrations()
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0)
}

/// DDL を発行せずに、同じ接続で現在の schema 版を読む（MNT-03-D11）
///
/// `schema_versions` 表が無ければ版 0。存在の確認・読取りのその他の失敗は
/// `MigrationFailed` にし、版 0 に倒さない。
fn read_current_version_without_ddl(conn: &Connection) -> Result<i64, DbError> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_versions')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| DbError::MigrationFailed(format!("schema_versionsの存在確認失敗: {}", e)))?;
    if !exists {
        return Ok(0);
    }
    get_current_version(conn)
}

/// schema_versionsテーブルを作成する（存在しなければ）
fn ensure_schema_versions_table(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_versions (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| DbError::MigrationFailed(format!("schema_versionsテーブル作成失敗: {}", e)))
}

/// 現在のスキーマバージョンを取得する
fn get_current_version(conn: &Connection) -> Result<i64, DbError> {
    let version: Option<i64> = conn
        .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
            row.get(0)
        })
        .map_err(|e| DbError::MigrationFailed(format!("バージョン取得失敗: {}", e)))?;
    Ok(version.unwrap_or(0))
}

/// Sql 種別のマイグレーションをトランザクション内で実行する
fn apply_sql_migration(
    conn: &Connection,
    version: i64,
    description: &str,
    sql: &str,
) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    conn.execute_batch("BEGIN;")
        .map_err(|e| DbError::MigrationFailed(format!("v{} BEGIN失敗: {}", version, e)))?;

    if let Err(e) = conn.execute_batch(sql) {
        return Err(migration_tx::rollback_after_error(
            conn,
            format!("v{} ({}) SQL実行失敗: {}", version, description, e),
        ));
    }

    if let Err(e) = conn.execute(
        "INSERT INTO schema_versions (version, applied_at) VALUES (?1, ?2)",
        rusqlite::params![version, now],
    ) {
        return Err(migration_tx::rollback_after_error(
            conn,
            format!("v{} バージョン記録失敗: {}", version, e),
        ));
    }

    migration_tx::commit_transaction(conn, &format!("v{} COMMIT失敗", version))?;

    Ok(())
}

/// schema_versionsテーブルを確認し、未適用のマイグレーションを順番に実行する
///
/// 22-mnt-migration.md §3.2:
/// 1. DDL を発行せずに同じ接続で現在の最大バージョンを読む（schema_versions が無ければ 0、MNT-03-D11）
/// 2. アプリの最大バージョンより新しければ SchemaNewerThanApp で拒否する（DDL も migration も始めない、MNT-03-D11）
/// 3. schema_versionsテーブルを確保する（CREATE TABLE IF NOT EXISTS）
/// 4. マイグレーションリストから current_version より大きいものを取り出す
/// 5. 各マイグレーションを順番に BEGIN → 実行 → バージョン記録 → COMMIT（失敗は ROLLBACK して MigrationFailed）
/// 6. 全て成功 → Ok(())
pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    migrate_through(conn, i64::MAX)
}

/// 版 `last_version` までだけを当てる test 用の migrate（22 §16 の vB・§19 の vL の test。期待は変えない）
#[cfg(test)]
pub(crate) fn migrate_up_to(conn: &Connection, last_version: i64) -> Result<(), DbError> {
    migrate_through(conn, last_version)
}

fn migrate_through(conn: &Connection, last_version: i64) -> Result<(), DbError> {
    // 1〜2. 論理的な書込み（DDL・BEGIN・INSERT）より前に版を読んで比較する（MNT-03-D11）
    let current_version = read_current_version_without_ddl(conn)?;
    let app_max = app_max_version();
    if current_version > app_max {
        return Err(DbError::SchemaNewerThanApp {
            db_version: current_version,
            app_max,
        });
    }

    // 3. schema_versionsテーブルを確保
    ensure_schema_versions_table(conn)?;

    // 4〜5. 未適用のマイグレーションを順番に実行
    for migration in migrations() {
        if migration.version > current_version && migration.version <= last_version {
            match &migration.kind {
                MigrationKind::Sql(sql) => {
                    apply_sql_migration(conn, migration.version, migration.description, sql)?;
                }
                MigrationKind::Custom(func) => {
                    func(conn, migration.version)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::{
        init_database,
        migration_tx::{self, FailurePoint},
        schema_v1, schema_v2, schema_v3, schema_v4, schema_v5, schema_v6, schema_v7, DbError,
        VB_VERSION,
    };
    use rusqlite::types::Value;
    use rusqlite::Connection;

    fn setup_minimal_migration_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_apply_sql_migration_req903_d1_preserves_original_error_after_rollback() {
        // REQ-903 / MNT-03-D1 / Matrix E1
        let conn = setup_minimal_migration_db();

        let error = super::apply_sql_migration(
            &conn,
            7,
            "failure-sql-summary",
            "CREATE TABLE transient(id INTEGER); INVALID SQL;",
        )
        .unwrap_err();

        assert!(conn.is_autocommit());
        assert!(matches!(error, DbError::MigrationFailed(message)
            if message.contains("v7") && message.contains("failure-sql-summary")));
    }

    #[test]
    fn test_apply_sql_migration_req903_d1_combines_rollback_failure() {
        // REQ-903 / MNT-03-D1 / Matrix E2
        let conn = setup_minimal_migration_db();
        let _failure = migration_tx::fail_operations(&[FailurePoint::Rollback]);

        let (result, logs) = crate::test_tracing::capture(|| {
            super::apply_sql_migration(
                &conn,
                8,
                "original-sql-error",
                "CREATE TABLE transient(id INTEGER); INVALID SQL;",
            )
        });

        let DbError::MigrationFailed(message) = result.unwrap_err() else {
            panic!("MigrationFailedを期待")
        };
        assert!(message.contains("original-sql-error"), "message={message}");
        assert!(
            message.contains("injected ROLLBACK failure"),
            "message={message}"
        );
        assert!(
            message.contains("transaction 状態不明"),
            "message={message}"
        );
        assert!(
            logs.contains("ERROR") && logs.contains("ROLLBACK"),
            "logs={logs:?}"
        );
    }

    #[test]
    fn test_apply_sql_migration_req903_d1_commit_failure_rolls_back_and_allows_retry() {
        // REQ-903 / MNT-03-D1 / Matrix E3
        let conn = setup_minimal_migration_db();
        let failure = migration_tx::fail_operations(&[FailurePoint::Commit]);

        let error = super::apply_sql_migration(
            &conn,
            9,
            "commit-failure",
            "CREATE TABLE committed_after_retry(id INTEGER);",
        )
        .unwrap_err();

        let DbError::MigrationFailed(message) = error else {
            panic!("MigrationFailedを期待")
        };
        assert!(
            message.contains("injected COMMIT failure"),
            "併合メッセージは実 COMMIT エラーを含むべき（context 固定文言だけでは不可）: {message}"
        );
        assert!(message.contains("ROLLBACK 成功"), "message={message}");
        assert!(
            conn.is_autocommit(),
            "COMMIT失敗後にtransactionを閉じるべき"
        );
        drop(failure);

        super::apply_sql_migration(
            &conn,
            9,
            "commit-failure",
            "CREATE TABLE committed_after_retry(id INTEGER);",
        )
        .unwrap();
        assert!(conn.is_autocommit());
    }

    #[test]
    fn test_apply_sql_migration_req903_d1_wal_writer_lock_fails_on_write_then_retries() {
        // REQ-903 / MNT-03-D1 / Matrix E3b
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wal-lock.db");
        let writer = Connection::open(&path).unwrap();
        let migrating = Connection::open(&path).unwrap();
        writer
            .execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=0;")
            .unwrap();
        migrating
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA busy_timeout=0;
                 CREATE TABLE schema_versions (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL
                 );",
            )
            .unwrap();
        writer
            .execute_batch("BEGIN IMMEDIATE; CREATE TABLE writer_holds_lock(id INTEGER);")
            .unwrap();

        let error = super::apply_sql_migration(
            &migrating,
            10,
            "wal-write-contention",
            "CREATE TABLE succeeds_after_unlock(id INTEGER);",
        )
        .unwrap_err();
        assert!(matches!(error, DbError::MigrationFailed(message)
            if message.contains("SQL実行失敗") && message.contains("locked")));
        assert!(
            migrating.is_autocommit(),
            "write失敗後はROLLBACK済みであるべき"
        );

        writer.execute_batch("ROLLBACK;").unwrap();
        super::apply_sql_migration(
            &migrating,
            10,
            "wal-write-contention",
            "CREATE TABLE succeeds_after_unlock(id INTEGER);",
        )
        .unwrap();
    }

    #[test]
    fn test_migration_req903_d1_all_transaction_sites_use_shared_helper() {
        // REQ-903 / MNT-03-D1 / Matrix E6
        let migration = include_str!("migration.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let schema_v2 = include_str!("schema_v2.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let schema_v3 = include_str!("schema_v3.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let schema_v8 = include_str!("schema_v8.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let schema_v9 = include_str!("schema_v9.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let schema_v10 = include_str!("schema_v10.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let schema_v11 = include_str!("schema_v11.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        for source in [
            migration, schema_v2, schema_v3, schema_v8, schema_v9, schema_v10, schema_v11,
        ] {
            assert!(!source.contains("execute_batch(\"ROLLBACK;\").ok()"));
            assert!(!source.contains("execute_batch(\"COMMIT;\")"));
        }
        assert_eq!(migration.matches("rollback_after_error").count(), 2);
        assert_eq!(schema_v2.matches("rollback_after_error").count(), 4);
        assert_eq!(schema_v3.matches("rollback_after_error").count(), 2);
        assert_eq!(schema_v8.matches("rollback_after_error").count(), 1);
        assert_eq!(schema_v9.matches("rollback_after_error").count(), 1);
        assert_eq!(schema_v10.matches("rollback_after_error").count(), 1);
        assert_eq!(schema_v11.matches("rollback_after_error").count(), 1);
        assert_eq!(migration.matches("commit_transaction").count(), 1);
        assert_eq!(schema_v2.matches("commit_transaction").count(), 1);
        assert_eq!(schema_v3.matches("commit_transaction").count(), 1);
        assert_eq!(schema_v8.matches("commit_transaction").count(), 1);
        assert_eq!(schema_v9.matches("commit_transaction").count(), 1);
        assert_eq!(schema_v10.matches("commit_transaction").count(), 1);
        assert_eq!(schema_v11.matches("commit_transaction").count(), 1);
    }

    // -----------------------------------------------------------------------
    // MNT-03-D11: 新しすぎる版の DB を論理的な書込みの前に拒否する
    // -----------------------------------------------------------------------

    fn schema_version_rows(conn: &Connection) -> Vec<i64> {
        let mut statement = conn
            .prepare("SELECT version FROM schema_versions ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn schema_objects(conn: &Connection) -> Vec<(String, String, Option<String>)> {
        let mut statement = conn
            .prepare("SELECT type, name, sql FROM sqlite_master ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn table_row_counts(conn: &Connection) -> Vec<(String, i64)> {
        let tables: Vec<String> = {
            let mut statement = conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        tables
            .into_iter()
            .map(|table| {
                let count = conn
                    .query_row(&format!("SELECT COUNT(*) FROM \"{table}\""), [], |row| {
                        row.get(0)
                    })
                    .unwrap();
                (table, count)
            })
            .collect()
    }

    fn file_sha256(path: &std::path::Path) -> String {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
    }

    fn dir_file_names(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn test_migrate_req903_d11_rejects_newer_schema_without_writes() {
        // REQ-903 / MNT-03-D11 / Matrix T6
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let newer = crate::db::test_support::write_newer_schema_db(&db_path);
        let app_max = super::app_max_version();

        let conn = Connection::open(&db_path).unwrap();
        let versions_before = schema_version_rows(&conn);
        let objects_before = schema_objects(&conn);

        let error = super::migrate(&conn).unwrap_err();

        assert!(matches!(
            error,
            DbError::SchemaNewerThanApp { db_version, app_max: max }
                if db_version == newer && db_version == app_max + 1 && max == app_max
        ));
        assert_eq!(schema_version_rows(&conn), versions_before);
        assert_eq!(schema_objects(&conn), objects_before);
    }

    #[test]
    fn test_migrate_req903_d11_same_version_is_noop() {
        // REQ-903 / MNT-03-D11 / Matrix T7: 同じ版は拒否しない（比較の境界）
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        let versions_before = schema_version_rows(&conn);
        assert_eq!(versions_before.last(), Some(&super::app_max_version()));

        super::migrate(&conn).unwrap();

        assert_eq!(schema_version_rows(&conn), versions_before);
    }

    #[test]
    fn test_init_database_req903_d11_newer_schema_leaves_file_bytes_unchanged() {
        // REQ-903 / MNT-03-D11 / Matrix T8 (i): 正常に閉じた DB は file の bytes も変わらない
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let newer = crate::db::test_support::write_newer_schema_db(&db_path);
        let hash_before = file_sha256(&db_path);
        let files_before = dir_file_names(dir.path());

        let error = init_database(db_path.to_str().unwrap()).unwrap_err();
        assert!(matches!(
            error,
            DbError::SchemaNewerThanApp { db_version, .. } if db_version == newer
        ));

        assert_eq!(file_sha256(&db_path), hash_before);
        assert_eq!(dir_file_names(dir.path()), files_before);
    }

    #[test]
    fn test_init_database_req903_d11_newer_schema_in_wal_keeps_logical_content() {
        // REQ-903 / MNT-03-D11 / Matrix T8 (ii): 版が WAL にだけある DB も拒否し、論理内容を変えない
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        drop(init_database(db_path.to_str().unwrap()).unwrap());
        let app_max = super::app_max_version();
        let writer = Connection::open(&db_path).unwrap();
        writer
            .execute_batch(
                "PRAGMA wal_autocheckpoint=0;
                 PRAGMA wal_checkpoint(TRUNCATE);",
            )
            .unwrap();
        writer
            .execute(
                "INSERT INTO schema_versions (version, applied_at) VALUES (?1, '2026-09-25T00:00:00')",
                [app_max + 1],
            )
            .unwrap();
        let wal_path = std::path::PathBuf::from(format!("{}-wal", db_path.display()));
        assert!(
            std::fs::metadata(&wal_path).unwrap().len() > 32,
            "WAL frame fixture required"
        );
        let counts_before = table_row_counts(&writer);
        let versions_before = schema_version_rows(&writer);

        let error = init_database(db_path.to_str().unwrap()).unwrap_err();
        assert!(matches!(
            error,
            DbError::SchemaNewerThanApp { db_version, app_max: max }
                if db_version == app_max + 1 && max == app_max
        ));

        assert_eq!(table_row_counts(&writer), counts_before);
        assert_eq!(schema_version_rows(&writer), versions_before);
    }

    fn setup_v1_only_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_versions (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
        conn.execute_batch(schema_v1::get_initial_schema()).unwrap();
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (1, '2026-04-06T00:00:00')",
            [],
        )
        .unwrap();
        (dir, conn)
    }

    fn setup_v2_only_db() -> (tempfile::TempDir, Connection) {
        let (dir, conn) = setup_v1_only_db();
        schema_v2::apply_v2_idempotency(&conn, 2).unwrap();
        (dir, conn)
    }

    fn setup_v4_only_db() -> (tempfile::TempDir, Connection) {
        let (dir, conn) = setup_v2_only_db();
        schema_v3::apply_v3_plu_target(&conn, 3).unwrap();
        super::apply_sql_migration(
            &conn,
            4,
            "日報取込みテーブル追加（daily_report_imports + lines）",
            schema_v4::get_v4_daily_report_schema(),
        )
        .unwrap();
        (dir, conn)
    }

    #[test]
    fn test_migration_v6_adds_updated_at_with_backfill() {
        // REQ-107 / SPEC-SUPI-D1: v5 DB の既存行を created_at で backfill し、nullable を維持する。
        let (dir, conn) = setup_v4_only_db();
        schema_v5::apply_v5_plu_slots(&conn, 5).unwrap();
        conn.execute_batch(
            "INSERT INTO suppliers(name,created_at) VALUES
             ('テスト取引先A','2026-08-01T10:00:00'),
             ('テスト取引先B','2026-08-02T11:00:00');",
        )
        .unwrap();

        super::migrate(&conn).unwrap();
        let rows: Vec<(String, String, Option<String>)> = {
            let mut stmt = conn
                .prepare("SELECT name,created_at,updated_at FROM suppliers ORDER BY name")
                .unwrap();
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        assert_eq!(
            rows,
            vec![
                (
                    "テスト取引先A".to_string(),
                    "2026-08-01T10:00:00".to_string(),
                    Some("2026-08-01T10:00:00".to_string()),
                ),
                (
                    "テスト取引先B".to_string(),
                    "2026-08-02T11:00:00".to_string(),
                    Some("2026-08-02T11:00:00".to_string()),
                ),
            ]
        );

        conn.execute(
            "INSERT INTO suppliers(name,created_at) VALUES('テスト取引先C','2026-08-03T12:00:00')",
            [],
        )
        .unwrap();
        let new_updated_at: Option<String> = conn
            .query_row(
                "SELECT updated_at FROM suppliers WHERE name='テスト取引先C'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(new_updated_at, None);

        super::migrate(&conn).unwrap();
        let version_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_versions WHERE version=6",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version_count, 1);
        drop(conn);
        drop(dir);
    }

    /// schema_v1 の20テーブル + schema_versions + v5 plu_slots が存在すること
    #[test]
    fn test_migration_req903_creates_all_tables() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let expected_tables = vec![
            "schema_versions",
            "departments",
            "suppliers",
            "products",
            "receiving_records",
            "receiving_items",
            "return_records",
            "return_items",
            "manual_sales",
            "manual_sale_items",
            "disposal_records",
            "disposal_items",
            "csv_imports",
            "csv_import_errors",
            "sale_records",
            "inventory_movements",
            "price_history",
            "stocktakes",
            "stocktake_items",
            "operation_logs",
            "app_settings",
            "plu_slots",
            "stock_counts",
        ];

        for table_name in &expected_tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                    rusqlite::params![table_name],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "テーブル '{}' が存在しません", table_name);
        }
    }

    /// マイグレーション済みDBに対して再マイグレーションがスキップされること
    #[test]
    fn test_migration_req903_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // 1回目
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        let v1: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(v1, super::app_max_version()); // 最新の版（b1 Matrix T1）
        drop(conn);

        // 2回目（同じDBに対して再初期化）
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        let v2: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(v2, v1, "バージョンが変わってはいけない");

        // schema_versions のレコードは版ごとに 1 件（v1 から最新の版まで。b1 Matrix T1）
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_versions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            count,
            super::app_max_version(),
            "マイグレーションレコードは v1 から最新の版まで 1 件ずつ"
        );
    }

    #[test]
    fn test_migration_req903_v3_adds_plu_target_and_backfills_valid_ean13() {
        let (_dir, conn) = setup_v2_only_db();
        conn.execute(
            "INSERT INTO products (product_code, jan_code, name, department_id, selling_price, cost_price, tax_rate, stock_unit, is_discontinued, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, 500, 300, '10', 'pcs', ?4, '2026-04-06T00:00:00', '2026-04-06T00:00:00')",
            rusqlite::params!["VALID", "4901234567894", "valid", false],
        )
        .unwrap();
        for (code, jan, discontinued) in [
            ("NULLJAN", None, false),
            ("SHORT", Some("123456789012"), false),
            ("ALPHA", Some("49012345678A4"), false),
            ("DISC", Some("4901234567894"), true),
        ] {
            conn.execute(
                "INSERT INTO products (product_code, jan_code, name, department_id, selling_price, cost_price, tax_rate, stock_unit, is_discontinued, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 1, 500, 300, '10', 'pcs', ?4, '2026-04-06T00:00:00', '2026-04-06T00:00:00')",
                rusqlite::params![code, jan, code, discontinued],
            )
            .unwrap();
        }

        // products に行があるので vB（MNT-03-D14）の直前の版まで（Matrix T1 (b)）
        super::migrate_up_to(&conn, VB_VERSION - 1).unwrap();

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 9);

        let rows: Vec<(String, bool)> = {
            let mut stmt = conn
                .prepare("SELECT product_code, plu_target FROM products ORDER BY product_code")
                .unwrap();
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        assert_eq!(
            rows,
            vec![
                ("ALPHA".to_string(), false),
                ("DISC".to_string(), false),
                ("NULLJAN".to_string(), false),
                ("SHORT".to_string(), false),
                ("VALID".to_string(), true),
            ]
        );
    }

    #[test]
    fn test_migration_req401_v4_creates_daily_report_tables_and_indexes() {
        // REQ-401: SALES日報取込み
        // DB 12b-12e: Z001/Z002/Z005日報取込み用の親子テーブルと検索indexを追加する
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, super::app_max_version()); // 最新の版（b1 Matrix T1）

        for table_name in &[
            "daily_report_imports",
            "daily_report_summary_lines",
            "daily_report_payment_lines",
            "daily_report_department_lines",
        ] {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                    rusqlite::params![table_name],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "テーブル '{}' が存在しません", table_name);
        }

        for index_name in &[
            "idx_daily_report_imports_report_date",
            "idx_daily_report_imports_bundle_hash",
            "idx_daily_report_summary_lines_import_id",
            "idx_daily_report_payment_lines_import_id",
            "idx_daily_report_department_lines_import_department",
        ] {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?1",
                    rusqlite::params![index_name],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "インデックス '{}' が存在しません", index_name);
        }
    }

    #[test]
    fn test_migration_req907_v5_seeds_all_plu_slots() {
        // REQ-907: A-S1 migration v5 oracle values are intentionally independent.
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let (count, min_memory, max_memory, free_count): (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), MIN(memory_no), MAX(memory_no),
                        SUM(CASE WHEN status = 'free' THEN 1 ELSE 0 END)
                 FROM plu_slots",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            (count, min_memory, max_memory, free_count),
            (4_784, 217, 5_000, 4_784)
        );

        let (max_version, version_count): (i64, i64) = conn
            .query_row(
                "SELECT MAX(version), COUNT(*) FROM schema_versions",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            (max_version, version_count),
            (super::app_max_version(), super::app_max_version())
        );
    }

    #[test]
    fn test_migration_req907_upgrades_v4_database_and_seeds_all_plu_slots() {
        // REQ-907 / B-F1: v1〜v4 適用済み DB から v5 だけを適用する。
        let (_dir, conn) = setup_v4_only_db();
        super::migrate(&conn).unwrap();
        let max_version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        let slot_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM plu_slots", [], |row| row.get(0))
            .unwrap();
        assert_eq!(max_version, super::app_max_version()); // 最新の版（b1 Matrix T1）
        assert_eq!(slot_count, 4_784);
    }

    #[test]
    fn test_migration_req907_v5_rejects_out_of_range_and_unknown_status() {
        // REQ-907: A-S2 independent boundary oracle.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let conn = init_database(path.to_str().unwrap()).unwrap();
        for sql in [
            "INSERT INTO plu_slots(memory_no,status,updated_at) VALUES(216,'free','x')",
            "INSERT INTO plu_slots(memory_no,status,updated_at) VALUES(5001,'free','x')",
            "UPDATE plu_slots SET status='unknown' WHERE memory_no=217",
        ] {
            let error: DbError = conn.execute(sql, []).unwrap_err().into();
            assert!(matches!(error, DbError::QueryFailed(_)));
        }
    }

    #[test]
    fn test_migration_req907_v5_partial_unique_excludes_release_pending() {
        // REQ-907: A-S3 combinations are intentionally independent from production DDL.
        fn set(
            conn: &Connection,
            no: i64,
            code: Option<&str>,
            status: &str,
        ) -> Result<(), DbError> {
            conn.execute(
                "UPDATE plu_slots SET scanning_code=?2,status=?3 WHERE memory_no=?1",
                rusqlite::params![no, code, status],
            )?;
            Ok(())
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let conn = init_database(path.to_str().unwrap()).unwrap();

        set(&conn, 217, Some("A"), "active").unwrap();
        assert!(set(&conn, 218, Some("A"), "active").is_err());
        set(&conn, 217, Some("B"), "external").unwrap();
        assert!(set(&conn, 218, Some("B"), "reserved").is_err());
        set(&conn, 217, Some("B2"), "external").unwrap();
        assert!(set(&conn, 218, Some("B2"), "active").is_err());
        set(&conn, 217, Some("C"), "active").unwrap();
        set(&conn, 218, Some("C"), "release_pending").unwrap();
        set(&conn, 219, Some("C"), "release_pending").unwrap();
        set(&conn, 220, Some("D"), "external").unwrap();
        set(&conn, 221, Some("D"), "release_pending").unwrap();
        set(&conn, 222, None, "free").unwrap();
        set(&conn, 223, None, "free").unwrap();
    }

    #[test]
    fn test_migration_req401_v4_daily_report_constraints() {
        // REQ-401: SALES日報取込み
        // DB 12b-12e: status/source_adapter/source_file/FK制約をDBでも保持する
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        conn.execute(
            "INSERT INTO daily_report_imports (
                report_date, source_adapter, bundle_hash, source_files_json,
                gross_amount, net_amount, status, imported_at, note
             ) VALUES (
                '2026-03-21', 'casio_sr_s4000', 'hash-ok', '[]',
                12000, 11000, 'completed', '2026-03-21T18:00:00', NULL
             )",
            [],
        )
        .unwrap();
        let import_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO daily_report_summary_lines (
                daily_report_import_id, source_file, line_key, label, amount, quantity_hundredths, count, sort_order
             ) VALUES (?1, 'Z001', 'gross_sales', '総売上', 12000, NULL, NULL, 1)",
            rusqlite::params![import_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO daily_report_payment_lines (
                daily_report_import_id, source_file, payment_key, label, amount, count, sort_order
             ) VALUES (?1, 'Z002', 'cash', '現金', 11000, 8, 1)",
            rusqlite::params![import_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO daily_report_department_lines (
                daily_report_import_id, source_file, department_id, raw_department_name,
                normalized_department_name, amount, quantity_hundredths, count, sort_order
             ) VALUES (?1, 'Z005', 1, 'その他小物', 'その他小物', 3000, 400, NULL, 1)",
            rusqlite::params![import_id],
        )
        .unwrap();

        let invalid_adapter = conn.execute(
            "INSERT INTO daily_report_imports (
                report_date, source_adapter, bundle_hash, source_files_json,
                status, imported_at
             ) VALUES ('2026-03-22', 'unknown_adapter', 'hash-ng', '[]', 'completed', '2026-03-22T18:00:00')",
            [],
        );
        assert!(invalid_adapter.is_err(), "source_adapter CHECK が必要");

        let invalid_status = conn.execute(
            "INSERT INTO daily_report_imports (
                report_date, source_adapter, bundle_hash, source_files_json,
                status, imported_at
             ) VALUES ('2026-03-22', 'casio_sr_s4000', 'hash-ng2', '[]', 'failed', '2026-03-22T18:00:00')",
            [],
        );
        assert!(invalid_status.is_err(), "status CHECK が必要");

        let invalid_source = conn.execute(
            "INSERT INTO daily_report_summary_lines (
                daily_report_import_id, source_file, line_key, label, sort_order
             ) VALUES (?1, 'Z005', 'gross_sales', '総売上', 1)",
            rusqlite::params![import_id],
        );
        assert!(
            invalid_source.is_err(),
            "summary_lines.source_file は Z001 固定"
        );

        let invalid_department = conn.execute(
            "INSERT INTO daily_report_department_lines (
                daily_report_import_id, source_file, department_id, raw_department_name,
                amount, sort_order
             ) VALUES (?1, 'Z005', 99999, '未登録部門', 1000, 2)",
            rusqlite::params![import_id],
        );
        assert!(invalid_department.is_err(), "department_id FK が必要");
    }

    // -----------------------------------------------------------------------
    // MNT-03-D12: migration v7（日報の個数を100倍の整数へ）
    // -----------------------------------------------------------------------

    const V7_TABLES: [&str; 2] = [
        "daily_report_summary_lines",
        "daily_report_department_lines",
    ];

    fn setup_v6_only_db() -> (tempfile::TempDir, Connection) {
        let (dir, conn) = setup_v4_only_db();
        schema_v5::apply_v5_plu_slots(&conn, 5).unwrap();
        schema_v6::apply_v6_supplier_updated_at(&conn, 6).unwrap();
        (dir, conn)
    }

    /// v6 の列名 `quantity` で 2 表に合成の行を入れる（count は並び順の値）
    fn seed_v6_daily_report_lines(
        conn: &Connection,
        summary: &[Option<i64>],
        department: &[Option<i64>],
    ) {
        conn.execute(
            "INSERT INTO daily_report_imports (
                report_date, source_adapter, bundle_hash, source_files_json,
                status, imported_at
             ) VALUES ('2026-09-29', 'casio_sr_s4000', 'v7-hash', '[]', 'completed', '2026-09-29T18:00:00')",
            [],
        )
        .unwrap();
        let import_id = conn.last_insert_rowid();
        for (index, quantity) in summary.iter().enumerate() {
            conn.execute(
                "INSERT INTO daily_report_summary_lines (
                    daily_report_import_id, source_file, line_key, label, amount, quantity, count, sort_order
                 ) VALUES (?1, 'Z001', 'summary', '合成', NULL, ?2, ?3, ?3)",
                rusqlite::params![import_id, quantity, index as i64 + 1],
            )
            .unwrap();
        }
        for (index, quantity) in department.iter().enumerate() {
            conn.execute(
                "INSERT INTO daily_report_department_lines (
                    daily_report_import_id, source_file, department_id, raw_department_name,
                    normalized_department_name, amount, quantity, count, sort_order
                 ) VALUES (?1, 'Z005', NULL, '合成部門', '合成部門', 0, ?2, ?3, ?3)",
                rusqlite::params![import_id, quantity, index as i64 + 1],
            )
            .unwrap();
        }
    }

    fn column_names(conn: &Connection, table: &str) -> Vec<String> {
        let mut statement = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        statement
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn column_values(conn: &Connection, table: &str, column: &str) -> Vec<Option<i64>> {
        let mut statement = conn
            .prepare(&format!("SELECT {column} FROM {table} ORDER BY id"))
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn has_v7(conn: &Connection) -> bool {
        schema_version_rows(conn).contains(&7)
    }

    /// v7 が失敗したとき、2 表とも列名が `quantity`・値が不変・v7 が記録されないこと
    fn assert_v7_untouched(conn: &Connection, summary: &[Option<i64>], department: &[Option<i64>]) {
        for (table, expected) in V7_TABLES.into_iter().zip([summary, department]) {
            let columns = column_names(conn, table);
            assert!(
                columns.contains(&"quantity".to_string()),
                "{table}: {columns:?}"
            );
            assert!(
                !columns.contains(&"quantity_hundredths".to_string()),
                "{table}: {columns:?}"
            );
            assert_eq!(column_values(conn, table, "quantity"), expected, "{table}");
        }
        assert!(!has_v7(conn));
        assert!(conn.is_autocommit());
    }

    #[test]
    fn test_migration_req401_v7_scales_daily_report_quantity() {
        // REQ-401 / MNT-03-D12: 既存の個数を改名して 100 倍（NULL は NULL、負も 100 倍）
        let (_dir, conn) = setup_v6_only_db();
        let values = [Some(7), None, Some(-2)];
        seed_v6_daily_report_lines(&conn, &values, &values);

        super::migrate(&conn).unwrap();

        for table in V7_TABLES {
            let columns = column_names(&conn, table);
            assert!(
                columns.contains(&"quantity_hundredths".to_string()),
                "{columns:?}"
            );
            assert!(!columns.contains(&"quantity".to_string()), "{columns:?}");
            assert_eq!(
                column_values(&conn, table, "quantity_hundredths"),
                vec![Some(700), None, Some(-200)],
                "{table}"
            );
            assert_eq!(
                column_values(&conn, table, "count"),
                vec![Some(1), Some(2), Some(3)],
                "{table}"
            );
        }
        assert!(has_v7(&conn));

        // 再実行で v7 を重複適用しない
        super::migrate(&conn).unwrap();
        assert_eq!(
            schema_version_rows(&conn)
                .iter()
                .filter(|v| **v == 7)
                .count(),
            1
        );
        assert_eq!(
            column_values(
                &conn,
                "daily_report_department_lines",
                "quantity_hundredths"
            ),
            vec![Some(700), None, Some(-200)]
        );
    }

    #[test]
    fn test_migration_req401_v7_quantity_range_boundary() {
        // REQ-401 / MNT-03-D12: 範囲内の最大 ±(i64::MAX / 100) は成功、範囲外と i64::MIN は範囲検査で止める
        let limit = i64::MAX / 100;
        let (_dir, conn) = setup_v6_only_db();
        let values = [Some(limit), Some(limit), Some(-limit)];
        seed_v6_daily_report_lines(&conn, &values, &values);
        super::migrate(&conn).unwrap();
        for table in V7_TABLES {
            assert_eq!(
                column_values(&conn, table, "quantity_hundredths"),
                vec![Some(limit * 100), Some(limit * 100), Some(-limit * 100)],
                "{table}"
            );
            let non_integer: i64 = conn
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM {table} WHERE typeof(quantity_hundredths) <> 'integer'"
                    ),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(non_integer, 0, "{table}");
        }
        assert!(has_v7(&conn));

        for (bad_table, bad_value) in [
            ("daily_report_summary_lines", limit + 1),
            ("daily_report_department_lines", -limit - 1),
            ("daily_report_summary_lines", i64::MIN),
            ("daily_report_department_lines", i64::MIN),
        ] {
            let (_dir, conn) = setup_v6_only_db();
            let mut summary = vec![Some(7)];
            let mut department = vec![Some(7)];
            if bad_table == "daily_report_summary_lines" {
                summary.push(Some(bad_value));
            } else {
                department.push(Some(bad_value));
            }
            seed_v6_daily_report_lines(&conn, &summary, &department);

            let error = super::migrate(&conn).unwrap_err();

            let DbError::MigrationFailed(message) = error else {
                panic!("MigrationFailedを期待: {bad_table} {bad_value}")
            };
            assert!(message.contains("範囲検査"), "{bad_value}: {message}");
            assert!(message.contains(bad_table), "{bad_value}: {message}");
            assert_v7_untouched(&conn, &summary, &department);
        }
    }

    #[test]
    fn test_migration_req401_v7_verification_failure_rolls_back() {
        // REQ-401 / MNT-03-D12 / MNT-03-D1: v7 本体の検証が不一致を捕まえ、2 表・値・版を rollback する
        for (trigger_step, department) in [
            ("1", vec![Some(7), Some(5)]),
            (
                "CASE WHEN NEW.quantity >= 0 THEN 1 ELSE -1 END",
                vec![Some(7), Some(-2)],
            ),
        ] {
            let (_dir, conn) = setup_v6_only_db();
            let summary = vec![Some(7)];
            seed_v6_daily_report_lines(&conn, &summary, &department);
            // 改名前の列名で書いた trigger は RENAME COLUMN で本文が書き換わり、100 倍の後に余りを作る
            conn.execute_batch(&format!(
                "CREATE TRIGGER v7_test_bump AFTER UPDATE ON daily_report_department_lines
                 BEGIN
                     UPDATE daily_report_department_lines
                     SET quantity = quantity + {trigger_step} WHERE id = NEW.id;
                 END;"
            ))
            .unwrap();

            let error = super::migrate(&conn).unwrap_err();

            let DbError::MigrationFailed(message) = error else {
                panic!("MigrationFailedを期待")
            };
            assert!(message.contains("検証失敗"), "{message}");
            assert!(!message.contains("範囲検査"), "{message}");
            assert_v7_untouched(&conn, &summary, &department);

            conn.execute_batch("DROP TRIGGER v7_test_bump;").unwrap();
            super::migrate(&conn).unwrap();
            assert!(has_v7(&conn));
            assert_eq!(
                column_values(
                    &conn,
                    "daily_report_department_lines",
                    "quantity_hundredths"
                ),
                department
                    .iter()
                    .map(|value| value.map(|value| value * 100))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_migration_req401_v7_empty_and_all_null_tables() {
        // REQ-401 / MNT-03-D12: 新規 DB・行 0・全行 NULL の表でも v7 が成功する
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        assert!(has_v7(&conn));
        for table in V7_TABLES {
            let columns = column_names(&conn, table);
            assert!(
                columns.contains(&"quantity_hundredths".to_string()),
                "{columns:?}"
            );
            assert!(!columns.contains(&"quantity".to_string()), "{columns:?}");
        }

        let (_dir, conn) = setup_v6_only_db();
        super::migrate(&conn).unwrap();
        assert!(has_v7(&conn));

        let (_dir, conn) = setup_v6_only_db();
        seed_v6_daily_report_lines(&conn, &[None, None], &[None, None]);
        super::migrate(&conn).unwrap();
        assert!(has_v7(&conn));
        for table in V7_TABLES {
            assert_eq!(
                column_values(&conn, table, "quantity_hundredths"),
                vec![None, None],
                "{table}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // MNT-03-D13: migration v8（単位を 12 個の code へ。products の作り直しと上限の検査）
    // -----------------------------------------------------------------------

    const V8_UNIT_CODES: [&str; 12] = [
        "pcs", "sheet", "hon", "bag", "box", "roll", "kumi", "set", "ball", "cho", "m", "cm",
    ];

    /// v8 の上限の検査が読む表（値が変わらないことの比較に使う）
    const V8_CHECKED_TABLES: [&str; 11] = [
        "products",
        "receiving_items",
        "return_items",
        "manual_sale_items",
        "disposal_items",
        "sale_records",
        "inventory_movements",
        "price_history",
        "stocktakes",
        "stocktake_items",
        "schema_versions",
    ];

    fn setup_v7_only_db() -> (tempfile::TempDir, Connection) {
        let (dir, conn) = setup_v6_only_db();
        schema_v7::apply_v7_daily_report_quantity_hundredths(&conn, 7).unwrap();
        (dir, conn)
    }

    use crate::db::test_support::{dump, products_definition};

    fn foreign_keys(conn: &Connection) -> i64 {
        conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap()
    }

    /// v8 が読む全表の全行と `products` の定義（失敗で何も変わらないことの比較に使う）
    fn v8_snapshot(conn: &Connection) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
        let mut snapshot = products_definition(conn);
        for table in V8_CHECKED_TABLES {
            snapshot.push(dump(conn, &format!("SELECT * FROM {table} ORDER BY 1")));
        }
        snapshot
    }

    /// `ball`（v8 で足す code）の商品を INSERT できるか（CHECK が 2 code か 12 code かの判定）
    fn can_insert_unit(conn: &Connection, product_code: &str, unit: &str) -> bool {
        conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price,
                                   stock_unit, created_at, updated_at)
             VALUES (?1, '合成の商品', 1, 100, 50, ?2, '2026-10-01T00:00:00', '2026-10-01T00:00:00')",
            rusqlite::params![product_code, unit],
        )
        .is_ok()
    }

    /// 全列に値のある商品（pcs・cm・負の在庫・discontinued・plu_target=1）と、8 つの子の表の行
    fn seed_v7_products_with_children(conn: &Connection) {
        conn.execute_batch(
            "INSERT INTO suppliers (id, name, created_at, updated_at)
                 VALUES (1, '合成の取引先', '2026-10-01T00:00:00', '2026-10-01T00:00:00');
             INSERT INTO products (product_code, jan_code, name, department_id, supplier_id,
                 selling_price, cost_price, tax_rate, maker_code, stock_quantity, stock_unit,
                 is_discontinued, plu_dirty, plu_exported_at, pos_stock_sync, created_at, updated_at,
                 plu_target) VALUES
                 ('V8-PCS', '4900000000017', '合成の個数の商品', 1, 1, 700, 400, '10', 'MK-1', 20, 'pcs',
                  0, 0, '2026-10-02T09:00:00', 1, '2026-10-01T00:00:00', '2026-10-03T00:00:00', 1),
                 ('V8-CM', '4900000000024', '合成の長さの商品', 8, 1, 980, 385, '8', 'MK-2', -30, 'cm',
                  1, 1, '2026-10-02T10:00:00', 0, '2026-10-01T01:00:00', '2026-10-03T01:00:00', 0),
                 ('V8-CM2', '4900000000031', '合成の長さの商品 2', 8, 1, 1200, 600, '0', 'MK-3', 2750, 'cm',
                  0, 1, '2026-10-02T11:00:00', 1, '2026-10-01T02:00:00', '2026-10-03T02:00:00', 1);
             INSERT INTO receiving_records (id, supplier_id, receiving_date, note, idempotency_key,
                 request_fingerprint, created_at)
                 VALUES (1, 1, '2026-10-01', NULL, 'v8-rcv', 'fp', '2026-10-01T00:00:00');
             INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price)
                 VALUES (1, 'V8-PCS', 4, 250);
             INSERT INTO return_records (id, return_type, return_date, register_processed,
                 idempotency_key, request_fingerprint, created_at)
                 VALUES (1, 'return', '2026-10-01', 0, 'v8-ret', 'fp', '2026-10-01T00:00:00');
             INSERT INTO return_items (return_record_id, product_code, direction, quantity)
                 VALUES (1, 'V8-CM', 'in', 30);
             INSERT INTO manual_sales (id, sale_date, reason, idempotency_key, request_fingerprint,
                 created_at) VALUES (1, '2026-10-01', 'other', 'v8-ms', 'fp', '2026-10-01T00:00:00');
             INSERT INTO manual_sale_items (manual_sale_id, product_code, quantity, amount)
                 VALUES (1, 'V8-CM2', 130, 1560);
             INSERT INTO disposal_records (id, disposal_date, idempotency_key, request_fingerprint,
                 created_at) VALUES (1, '2026-10-01', 'v8-dsp', 'fp', '2026-10-01T00:00:00');
             INSERT INTO disposal_items (disposal_record_id, product_code, disposal_type, quantity,
                 cost_price, reason) VALUES (1, 'V8-PCS', 'damage', 2, 120, '合成の理由');
             INSERT INTO sale_records (product_code, sale_date, quantity, amount, source, created_at)
                 VALUES ('V8-PCS', '2026-10-01', 3, 2100, 'manual', '2026-10-01T00:00:00');
             INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 created_at) VALUES ('V8-CM', 'disposal', -30, -30, '2026-10-01T00:00:00');
             INSERT INTO price_history (product_code, old_selling, new_selling, old_cost, new_cost,
                 changed_at) VALUES ('V8-PCS', 650, 700, 380, 400, '2026-10-01T00:00:00');
             INSERT INTO stocktakes (id, started_at, completed_at, status, total_cost)
                 VALUES (1, '2026-10-01T00:00:00', '2026-10-01T01:00:00', 'completed', 8000);
             INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count,
                 valuation_cost_price, counted_at)
                 VALUES (1, 'V8-PCS', 20, 20, 400, '2026-10-01T00:30:00');",
        )
        .unwrap();
    }

    fn max_version_and_count(conn: &Connection) -> (i64, i64) {
        conn.query_row(
            "SELECT MAX(version), COUNT(*) FROM schema_versions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    fn migration_failed_message(error: DbError) -> String {
        match error {
            DbError::MigrationFailed(message) => message,
            other => panic!("MigrationFailedを期待: {other:?}"),
        }
    }

    #[test]
    fn test_migration_req903_v8_rebuild_keeps_rows_definition_indexes_and_fk() {
        // REQ-903 / MNT-03-D13 vU / Contract Probe P3: 作り直しの前後で行・列の定義・index・FK が同じ
        let (_dir, conn) = setup_v7_only_db();
        seed_v7_products_with_children(&conn);
        let rows_before = dump(&conn, "SELECT * FROM products ORDER BY product_code");
        let definition_before = products_definition(&conn);
        let units_before = dump(
            &conn,
            "SELECT stock_unit, COUNT(*) FROM products GROUP BY stock_unit ORDER BY stock_unit",
        );
        let counts_before = table_row_counts(&conn);
        assert_eq!(rows_before.len(), 3);
        assert_eq!(rows_before[0].len(), 18, "v7 の products は 18 列");
        assert!(!can_insert_unit(&conn, "V8-BALL-BEFORE", "ball"));

        super::migrate_up_to(&conn, 8).unwrap();

        assert_eq!(
            dump(&conn, "SELECT * FROM products ORDER BY product_code"),
            rows_before
        );
        // 定義の違いは stock_unit の CHECK だけ（pragma_table_info は CHECK を含まない）
        assert_eq!(products_definition(&conn), definition_before);
        assert_eq!(
            dump(
                &conn,
                "SELECT stock_unit, COUNT(*) FROM products GROUP BY stock_unit ORDER BY stock_unit"
            ),
            units_before
        );
        let mut counts_after = table_row_counts(&conn);
        let mut counts_expected = counts_before;
        for counts in [&mut counts_after, &mut counts_expected] {
            counts.retain(|(table, _)| table != "schema_versions");
        }
        assert_eq!(counts_after, counts_expected);
        assert!(dump(&conn, "SELECT * FROM pragma_foreign_key_check").is_empty());
        assert!(dump(
            &conn,
            "SELECT name FROM sqlite_master WHERE name = 'products_new'"
        )
        .is_empty());
        assert_eq!(foreign_keys(&conn), 1, "foreign_keys は元の値");
        assert_eq!(max_version_and_count(&conn), (8, 8));

        // 12 個の code をすべて INSERT でき、一覧に無い kg は CHECK で拒む
        for code in V8_UNIT_CODES {
            assert!(
                can_insert_unit(&conn, &format!("V8-NEW-{code}"), code),
                "{code}"
            );
        }
        assert!(!can_insert_unit(&conn, "V8-NEW-kg", "kg"));
        // 子の表の FK が効く（子のある商品の DELETE が拒まれる）
        assert!(conn
            .execute("DELETE FROM products WHERE product_code = 'V8-PCS'", [])
            .is_err());
        for child in [
            "receiving_items",
            "return_items",
            "manual_sale_items",
            "disposal_items",
            "sale_records",
            "inventory_movements",
            "price_history",
            "stocktake_items",
        ] {
            let parents = dump(
                &conn,
                &format!("SELECT \"table\" FROM pragma_foreign_key_list('{child}') WHERE \"table\" = 'products'"),
            );
            assert_eq!(parents.len(), 1, "{child} は products を参照する");
        }

        // 再実行で v8 を 2 度当てない
        super::migrate_up_to(&conn, 8).unwrap();
        assert_eq!(max_version_and_count(&conn), (8, 8));
    }

    #[test]
    fn test_migration_req903_v8_fresh_database_and_empty_tables() {
        // REQ-903 / MNT-03-D13: 新規の DB（v1 から v8）と行 0 の v7 の DB で成功する
        let dir = tempfile::tempdir().unwrap();
        let conn = init_database(dir.path().join("fresh.db").to_str().unwrap()).unwrap();
        assert_eq!(
            max_version_and_count(&conn),
            (super::app_max_version(), super::app_max_version())
        );
        // v8 の CHECK（12 個）は v8 まで当てた新規の DB で確かめる。最新の版の CHECK は vB の test（Matrix T1 (c)）
        let conn = Connection::open(dir.path().join("fresh-v8.db")).unwrap();
        super::migrate_up_to(&conn, 8).unwrap();
        assert_eq!(max_version_and_count(&conn), (8, 8));
        for code in V8_UNIT_CODES {
            assert!(
                can_insert_unit(&conn, &format!("V8-FRESH-{code}"), code),
                "{code}"
            );
        }
        assert!(!can_insert_unit(&conn, "V8-FRESH-kg", "kg"));

        let (_dir, conn) = setup_v7_only_db();
        let definition_before = products_definition(&conn);
        // 期待の列の定義は vB の前の名前（cost_price）なので vB の直前の版まで（Matrix T1 (b)）
        super::migrate_up_to(&conn, VB_VERSION - 1).unwrap();
        assert_eq!(max_version_and_count(&conn), (9, 9));
        let mut definition_expected = definition_before;
        // vL（22 §19 手順 6）が products の末尾に写しの 3 列を足す。FK と index は同じ
        definition_expected[0].extend(
            [
                (18, "stock_recheck", 1, Value::Text("0".into())),
                (19, "stock_range_low", 0, Value::Null),
                (20, "stock_range_high", 0, Value::Null),
            ]
            .map(|(cid, name, notnull, default)| {
                vec![
                    Value::Integer(cid),
                    Value::Text(name.into()),
                    Value::Text("INTEGER".into()),
                    Value::Integer(notnull),
                    default,
                    Value::Integer(0),
                ]
            }),
        );
        assert_eq!(products_definition(&conn), definition_expected);
    }

    #[test]
    fn test_migration_req903_v8_fk_check_failure_rolls_back() {
        // REQ-903 / MNT-03-D13 手順 6 (b): foreign_key_check が 0 行でなければ表・値・版を戻し、foreign_keys を元の値に戻す
        for original_fk in [1_i64, 0] {
            let (_dir, conn) = setup_v7_only_db();
            seed_v7_products_with_children(&conn);
            conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
            conn.execute(
                "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after, created_at)
                 VALUES ('V8-NO-PARENT', 'receiving', 1, 1, '2026-10-01T00:00:00')",
                [],
            )
            .unwrap();
            conn.execute_batch(&format!("PRAGMA foreign_keys = {original_fk};"))
                .unwrap();
            let before = v8_snapshot(&conn);

            let message = migration_failed_message(super::migrate(&conn).unwrap_err());

            assert!(message.contains("FK整合性エラー"), "{message}");
            assert_eq!(v8_snapshot(&conn), before);
            assert_eq!(max_version_and_count(&conn), (7, 7));
            assert_eq!(foreign_keys(&conn), original_fk);
            assert!(conn.is_autocommit());
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            assert!(
                !can_insert_unit(&conn, "V8-BALL", "ball"),
                "CHECK は 2 code のまま"
            );
        }
    }

    #[test]
    fn test_migration_req903_v8_commit_and_fk_restore_failures() {
        // REQ-903 / MNT-03-D13 / MNT-03-D1: COMMIT・ROLLBACK・foreign_keys の復元の失敗の扱い
        // (1) COMMIT が失敗し ROLLBACK は成功: 何も変わらない
        {
            let (_dir, conn) = setup_v7_only_db();
            seed_v7_products_with_children(&conn);
            let before = v8_snapshot(&conn);
            let failures = migration_tx::fail_operations(&[FailurePoint::Commit]);
            let message = migration_failed_message(super::migrate(&conn).unwrap_err());
            drop(failures);
            assert!(message.contains("v8 COMMIT失敗"), "{message}");
            assert_eq!(v8_snapshot(&conn), before);
            assert_eq!(max_version_and_count(&conn), (7, 7));
            assert_eq!(foreign_keys(&conn), 1);
            assert!(
                !can_insert_unit(&conn, "V8-BALL", "ball"),
                "CHECK は 2 code のまま"
            );
        }
        // (2) COMMIT も ROLLBACK も失敗: transaction の状態が不明で、foreign_keys を戻そうとしない
        {
            let (_dir, conn) = setup_v7_only_db();
            let failures =
                migration_tx::fail_operations(&[FailurePoint::Commit, FailurePoint::Rollback]);
            let message = migration_failed_message(super::migrate(&conn).unwrap_err());
            drop(failures);
            assert!(message.contains("transaction 状態不明"), "{message}");
            assert!(message.contains("接続破棄必須"), "{message}");
            assert!(!conn.is_autocommit());
            assert_eq!(foreign_keys(&conn), 0, "復元を試みていない");
        }
        // (3) COMMIT の後で foreign_keys の復元が効かない: 版 8 は残り、DB は新しい版として完全
        {
            let (dir, conn) = setup_v7_only_db();
            seed_v7_products_with_children(&conn);
            let failures = migration_tx::fail_operations(&[FailurePoint::ForeignKeyRestoreNoop]);
            let message = migration_failed_message(super::migrate(&conn).unwrap_err());
            drop(failures);
            assert!(message.contains("foreign_keys復元検証失敗"), "{message}");
            assert!(message.contains("expected=1"), "{message}");
            assert!(message.contains("actual=0"), "{message}");
            assert_eq!(max_version_and_count(&conn), (8, 8));
            assert!(
                can_insert_unit(&conn, "V8-BALL", "ball"),
                "CHECK は 12 code"
            );
            drop(conn);

            // 接続を破棄して同じ file を開き直すと、v8 を当て直さずに開ける。開き直しは最新の版まで当てるので、
            // fixture が入れた全ての表の行を子から消してから開く（vL の手順 1・vB の手順 2 で止まらないように。
            // 止まり自体は vL・vB の test）
            let raw = Connection::open(dir.path().join("test.db")).unwrap();
            raw.execute_batch(
                "DELETE FROM stocktake_items; DELETE FROM stocktakes;
                 DELETE FROM price_history; DELETE FROM inventory_movements;
                 DELETE FROM sale_records; DELETE FROM disposal_items; DELETE FROM disposal_records;
                 DELETE FROM manual_sale_items; DELETE FROM manual_sales;
                 DELETE FROM return_items; DELETE FROM return_records;
                 DELETE FROM receiving_items; DELETE FROM receiving_records;
                 DELETE FROM products; DELETE FROM suppliers;",
            )
            .unwrap();
            assert!(dump(&raw, "SELECT * FROM pragma_foreign_key_check").is_empty());
            drop(raw);
            let conn = init_database(dir.path().join("test.db").to_str().unwrap()).unwrap();
            assert_eq!(foreign_keys(&conn), 1);
            assert_eq!(
                max_version_and_count(&conn),
                (super::app_max_version(), super::app_max_version())
            );
            assert_eq!(
                schema_version_rows(&conn)
                    .iter()
                    .filter(|v| **v == 8)
                    .count(),
                1,
                "v8 を当て直さない"
            );
        }
        // (4) COMMIT の後で foreign_keys の復元が error: 版 8 は残る
        {
            let (_dir, conn) = setup_v7_only_db();
            let failures = migration_tx::fail_operations(&[FailurePoint::ForeignKeyRestore]);
            let message = migration_failed_message(super::migrate(&conn).unwrap_err());
            drop(failures);
            assert!(message.contains("foreign_keys復元失敗"), "{message}");
            assert_eq!(max_version_and_count(&conn), (8, 8));
        }
    }

    /// 上限の明細（数量 9999999・原価 999999）を `rows` 行持つ入庫と廃棄の記録を 1 つずつ足す
    fn seed_v8_limit_line_records(conn: &Connection, record_id: i64, rows: i64) {
        conn.execute_batch(&format!(
            "INSERT INTO receiving_records (id, receiving_date, idempotency_key, request_fingerprint, created_at)
                 VALUES ({record_id}, '2026-10-01', 'v8-rcv-{record_id}', 'fp', '2026-10-01T00:00:00');
             INSERT INTO disposal_records (id, disposal_date, idempotency_key, request_fingerprint, created_at)
                 VALUES ({record_id}, '2026-10-01', 'v8-dsp-{record_id}', 'fp', '2026-10-01T00:00:00');
             WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < {rows})
             INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price)
                 SELECT {record_id}, 'V8-PCS', 9999999, 999999 FROM n;
             WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < {rows})
             INSERT INTO disposal_items (disposal_record_id, product_code, disposal_type, quantity, cost_price, reason)
                 SELECT {record_id}, 'V8-PCS', 'other', 9999999, 999999, '合成' FROM n;"
        ))
        .unwrap();
    }

    #[test]
    fn test_migration_req903_v8_limit_check_accepts_boundary_values() {
        // REQ-903 / MNT-03-D13 vU の上限の検査（X13）: 各列の端の値は通り、値が変わらない
        let (_dir, conn) = setup_v7_only_db();
        seed_v7_products_with_children(&conn);
        conn.execute_batch(
            "UPDATE products SET stock_quantity = 999999999, selling_price = 999999, cost_price = 999999
                 WHERE product_code = 'V8-PCS';
             UPDATE products SET stock_quantity = -999999999, cost_price = -999999
                 WHERE product_code = 'V8-CM';
             UPDATE receiving_items SET quantity = 9999999, cost_price = 999999;
             INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price)
                 VALUES (1, 'V8-CM', 1, -999999);
             UPDATE return_items SET quantity = 9999999;
             UPDATE manual_sale_items SET quantity = 9999999;
             UPDATE disposal_items SET quantity = 9999999, cost_price = 999999;
             INSERT INTO disposal_items (disposal_record_id, product_code, disposal_type, quantity, cost_price, reason)
                 VALUES (1, 'V8-CM', 'other', 1, -999999, '合成');
             UPDATE sale_records SET quantity = 9999999;
             INSERT INTO sale_records (product_code, sale_date, quantity, amount, source, created_at)
                 VALUES ('V8-PCS', '2026-10-01', -9999999, -1, 'manual', '2026-10-01T00:00:00');
             UPDATE inventory_movements SET quantity = -1999999998, stock_after = -999999999;
             INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after, created_at)
                 VALUES ('V8-PCS', 'stocktake', 1999999998, 999999999, '2026-10-01T00:00:00');
             UPDATE price_history SET old_selling = 999999, new_selling = 999999,
                 old_cost = 999999, new_cost = -999999;
             INSERT INTO price_history (product_code, old_selling, new_selling, old_cost, new_cost, changed_at)
                 VALUES ('V8-CM', 0, 0, -999999, 999999, '2026-10-01T00:00:00');
             UPDATE stocktakes SET total_cost = 9007199254740991;
             INSERT INTO stocktakes (id, started_at, status, total_cost)
                 VALUES (2, '2026-10-02T00:00:00', 'in_progress', NULL);
             UPDATE stocktake_items SET system_stock = 999999999, actual_count = 999999999,
                 valuation_cost_price = 999999;
             INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count, valuation_cost_price)
                 VALUES (2, 'V8-CM', -999999999, NULL, NULL), (2, 'V8-CM2', 0, 0, -999999);",
        )
        .unwrap();
        // 上限の明細 900 行の記録（和 8999990100000900）
        seed_v8_limit_line_records(&conn, 2, 900);
        let total: i64 = conn
            .query_row(
                "SELECT SUM(quantity * cost_price) FROM receiving_items WHERE receiving_record_id = 2",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total, 8_999_990_100_000_900);
        let mut before = v8_snapshot(&conn);

        super::migrate_up_to(&conn, 8).unwrap();

        let mut after = v8_snapshot(&conn);
        // schema_versions（最後の要素）だけ 1 行増える
        assert_eq!(after.pop().unwrap().len(), 8);
        assert_eq!(before.pop().unwrap().len(), 7);
        assert_eq!(after, before);
        assert_eq!(foreign_keys(&conn), 1);
    }

    #[test]
    fn test_migration_req903_v8_limit_check_rejects_out_of_range_values() {
        // REQ-903 / MNT-03-D13 vU の上限の検査（X13）: どれか 1 つが 1 つ外なら、何も変えずに止まる
        let cases: [(&str, &str, &str); 24] = [
            (
                "receiving_items",
                "quantity",
                "UPDATE receiving_items SET quantity = 10000000",
            ),
            (
                "return_items",
                "quantity",
                "UPDATE return_items SET quantity = 10000000",
            ),
            (
                "manual_sale_items",
                "quantity",
                "UPDATE manual_sale_items SET quantity = 10000000",
            ),
            (
                "disposal_items",
                "quantity",
                "UPDATE disposal_items SET quantity = 10000000",
            ),
            (
                "sale_records",
                "quantity",
                "UPDATE sale_records SET quantity = 10000000",
            ),
            (
                "sale_records",
                "quantity",
                "UPDATE sale_records SET quantity = -10000000",
            ),
            (
                "products",
                "stock_quantity",
                "UPDATE products SET stock_quantity = 1000000000 WHERE product_code = 'V8-PCS'",
            ),
            (
                "products",
                "stock_quantity",
                "UPDATE products SET stock_quantity = -1000000000 WHERE product_code = 'V8-CM'",
            ),
            (
                "inventory_movements",
                "stock_after",
                "UPDATE inventory_movements SET stock_after = 1000000000",
            ),
            (
                "stocktake_items",
                "system_stock",
                "UPDATE stocktake_items SET system_stock = -1000000000",
            ),
            (
                "stocktake_items",
                "actual_count",
                "UPDATE stocktake_items SET actual_count = 1000000000",
            ),
            (
                "inventory_movements",
                "quantity",
                "UPDATE inventory_movements SET quantity = 1999999999",
            ),
            (
                "inventory_movements",
                "quantity",
                "UPDATE inventory_movements SET quantity = -1999999999",
            ),
            (
                "products",
                "selling_price",
                "UPDATE products SET selling_price = 1000000 WHERE product_code = 'V8-CM2'",
            ),
            (
                "price_history",
                "old_selling",
                "UPDATE price_history SET old_selling = 1000000",
            ),
            (
                "price_history",
                "new_selling",
                "UPDATE price_history SET new_selling = 1000000",
            ),
            (
                "products",
                "cost_price",
                "UPDATE products SET cost_price = 1000000 WHERE product_code = 'V8-CM2'",
            ),
            (
                "receiving_items",
                "cost_price",
                "UPDATE receiving_items SET cost_price = 1000000",
            ),
            (
                "disposal_items",
                "cost_price",
                "UPDATE disposal_items SET cost_price = -1000000",
            ),
            (
                "stocktake_items",
                "valuation_cost_price",
                "UPDATE stocktake_items SET valuation_cost_price = 1000000",
            ),
            (
                "price_history",
                "old_cost",
                "UPDATE price_history SET old_cost = 1000000",
            ),
            (
                "price_history",
                "new_cost",
                "UPDATE price_history SET new_cost = 1000000",
            ),
            (
                "stocktakes",
                "total_cost",
                "UPDATE stocktakes SET total_cost = 9007199254740992",
            ),
            (
                "stocktakes",
                "total_cost",
                "UPDATE stocktakes SET total_cost = -9007199254740992",
            ),
        ];
        for (table, column, sql) in cases {
            let (_dir, conn) = setup_v7_only_db();
            seed_v7_products_with_children(&conn);
            conn.execute_batch(sql).unwrap();
            let before = v8_snapshot(&conn);

            let message = migration_failed_message(super::migrate(&conn).unwrap_err());

            assert!(message.contains("上限の検査"), "{sql}: {message}");
            assert!(
                message.contains(&format!("{table}.{column}")),
                "{sql}: {message}"
            );
            assert_eq!(v8_snapshot(&conn), before, "{sql}");
            assert_eq!(max_version_and_count(&conn), (7, 7), "{sql}");
            assert_eq!(foreign_keys(&conn), 1, "{sql}");
            assert!(conn.is_autocommit(), "{sql}");
        }
    }

    #[test]
    fn test_migration_req903_v8_limit_check_rejects_record_total_over_safe_integer() {
        // REQ-903 / MNT-03-D13 vU の上限の検査（X13）: 記録ごとの和。上限の明細 901 行（9009990089000901）は止まる
        for (table, record_column, extra_row) in [
            (
                "receiving_items",
                "receiving_record_id",
                "INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price)
                     VALUES (2, 'V8-PCS', 9999999, 999999)",
            ),
            (
                "disposal_items",
                "disposal_record_id",
                "INSERT INTO disposal_items (disposal_record_id, product_code, disposal_type, quantity, cost_price, reason)
                     VALUES (2, 'V8-PCS', 'other', 9999999, 999999, '合成')",
            ),
        ] {
            let (_dir, conn) = setup_v7_only_db();
            seed_v7_products_with_children(&conn);
            seed_v8_limit_line_records(&conn, 2, 900);
            conn.execute_batch(extra_row).unwrap();
            let total: i64 = conn
                .query_row(
                    &format!("SELECT SUM(quantity * cost_price) FROM {table} WHERE {record_column} = 2"),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(total, 9_009_990_089_000_901);
            let before = v8_snapshot(&conn);

            let message = migration_failed_message(super::migrate(&conn).unwrap_err());

            assert!(message.contains("上限の検査"), "{message}");
            for word in [table, record_column, "quantity", "cost_price"] {
                assert!(message.contains(word), "{word}: {message}");
            }
            assert_eq!(v8_snapshot(&conn), before);
            assert_eq!(max_version_and_count(&conn), (7, 7));
            assert_eq!(foreign_keys(&conn), 1);
        }
    }

    #[test]
    fn test_migration_req401_v4_keeps_existing_sales_tables_intact() {
        // REQ-401: SALES日報取込み
        // D-025: 日報モデル追加時もZ004/sale_records/inventory_movementsの既存表を壊さない
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        for sql in &[
            "SELECT id, filename, settlement_date, file_hash, total_items, total_amount, skipped_count, status, imported_at FROM csv_imports LIMIT 0",
            "SELECT id, csv_import_id, product_code, sale_date, quantity, amount, source, source_line_no, reason, note, is_voided, created_at FROM sale_records LIMIT 0",
            "SELECT id, product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at FROM inventory_movements LIMIT 0",
        ] {
            conn.prepare(sql).unwrap();
        }
    }

    /// departments初期データ（21部門）が正しく投入されていること
    /// REQ-903: 期待は master-tables §2「初期データ（全21部門）」の表から写す（schema_v1.rs から写さない）。
    /// code_prefix は SQL の NULL（None）と空文字（Some("")）を区別して比べる。
    #[test]
    fn test_initial_departments_req903_data() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        // (id, name = z005_name, code_prefix)。next_seq は全行 1
        let expected: [(i64, &str, Option<&str>); 21] = [
            (1, "その他小物", Some("KM")),
            (2, "ヘア雑貨", Some("HZ")),
            (3, "毛糸", None),
            (4, "手芸等材料", Some("SY")),
            (5, "他化粧品", None),
            (6, "バッグ", Some("BG")),
            (7, "エプロン", Some("AP")),
            (8, "布", Some("NU")),
            (9, "糸", None),
            (10, "ゴム", Some("GM")),
            (11, "食品", None),
            (12, "宅急便", None),
            (13, "トワニー", None),
            (14, "雑貨", Some("ZK")),
            (15, "帽子", None),
            (16, "ビューティ関連", None),
            (17, "本", None),
            (18, "ボタン", Some("BT")),
            (19, "ファスナー", Some("FS")),
            (20, "針", None),
            (21, "ノンリンク", None),
        ];
        let expected: Vec<(i64, String, String, Option<String>, i64)> = expected
            .iter()
            .map(|(id, name, prefix)| {
                (
                    *id,
                    name.to_string(),
                    name.to_string(),
                    prefix.map(str::to_string),
                    1,
                )
            })
            .collect();

        let mut stmt = conn
            .prepare(
                "SELECT id, name, z005_name, code_prefix, next_seq FROM departments ORDER BY id",
            )
            .unwrap();
        let actual: Vec<(i64, String, String, Option<String>, i64)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(
            actual, expected,
            "departments の初期データは master-tables §2 の 21 行と一致する"
        );
    }

    /// app_settings初期データが正しく投入されていること
    #[test]
    fn test_initial_app_settings_req903_data() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let threshold: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'stock_low_threshold'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(threshold, "3");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 9, "app_settings初期値は9件");
    }

    /// CHECK制約が動作すること（不正値でINSERTが失敗すること）
    #[test]
    fn test_check_constraints_req903_reject_invalid_values() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        // products.tax_rate に不正値 → 失敗すべき（列名の誤りで常に失敗しないよう CHECK の文まで見る。Matrix T2a (i)）
        let result = conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi, tax_rate, stock_unit, created_at, updated_at)
             VALUES ('TEST001', 'テスト商品', 1, 100, 50, '15', 'pcs', '2026-04-03T00:00:00', '2026-04-03T00:00:00')",
            [],
        );
        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("CHECK constraint failed")),
            "tax_rate '15' は CHECK制約で弾かれるべき: {result:?}"
        );

        // products.stock_unit に不正値 → 失敗すべき
        let result = conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi, tax_rate, stock_unit, created_at, updated_at)
             VALUES ('TEST002', 'テスト商品2', 1, 100, 50, '10', 'kg', '2026-04-03T00:00:00', '2026-04-03T00:00:00')",
            [],
        );
        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("CHECK constraint failed")),
            "stock_unit 'kg' は CHECK制約で弾かれるべき: {result:?}"
        );

        // 正常値 → 成功すべき
        let result = conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi, tax_rate, stock_unit, created_at, updated_at)
             VALUES ('TEST003', 'テスト商品3', 1, 100, 50, '10', 'pcs', '2026-04-03T00:00:00', '2026-04-03T00:00:00')",
            [],
        );
        assert!(result.is_ok(), "正常値は INSERT できるべき");
    }

    /// 外部キー制約が動作すること
    #[test]
    fn test_foreign_key_req903_constraints() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        // 存在しない department_id → 失敗すべき（列名の誤りで常に失敗しないよう文まで見る。Matrix T2a (i)）
        let insert = |code: &str, department_id: i64| {
            conn.execute(
                "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi, tax_rate, stock_unit, created_at, updated_at)
                 VALUES (?1, 'テスト商品', ?2, 100, 50, '10', 'pcs', '2026-04-03T00:00:00', '2026-04-03T00:00:00')",
                rusqlite::params![code, department_id],
            )
        };
        let result = insert("TEST001", 9999);
        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("FOREIGN KEY constraint failed")),
            "存在しないdepartment_idは外部キー制約で弾かれるべき: {result:?}"
        );
        // 対照: 存在する部門なら通る
        assert!(insert("TEST002", 1).is_ok());
    }

    /// インデックスが作成されていること
    #[test]
    fn test_indexes_req903_exist() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let expected_indexes = vec![
            "idx_products_jan_code",
            "idx_products_department_id",
            "idx_products_is_discontinued",
            "idx_csv_imports_file_hash",
            "idx_sale_records_sale_date",
            "idx_sale_records_product_date",
            "idx_sale_records_csv_import_id",
            "idx_inventory_movements_product_date",
            "idx_inventory_movements_reference",
            "idx_stocktake_items_stocktake_product",
        ];

        for index_name in &expected_indexes {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?1",
                    rusqlite::params![index_name],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "インデックス '{}' が存在しません", index_name);
        }
    }
}
