//! migration v2: 冪等性カラム追加（4テーブル再作成）
//!
//! 22-mnt-migration.md §9 に基づく実装。
//! receiving_records, return_records, manual_sales, disposal_records に
//! idempotency_key + request_fingerprint カラムを NOT NULL で追加する。
//!
//! SQLite の ALTER TABLE ADD COLUMN は NOT NULL + デフォルト値なしに対応しないため、
//! テーブル再作成方式で実装する。

use super::{migration_tx, DbError};
use rusqlite::Connection;

/// v2 マイグレーション: 冪等性カラム追加
///
/// Custom マイグレーションのため、TX管理 + schema_versions INSERT まで本関数が責任を持つ。
///
/// ## 処理フロー
/// 1. PRAGMA foreign_keys の現在値を保存
/// 2. PRAGMA foreign_keys = OFF（TX外）
/// 3. BEGIN IMMEDIATE
/// 4. 4テーブル再作成（完全DDL、列マッピング明示）
/// 5. バックフィル: '__legacy__:' || id
/// 6. UNIQUE INDEX 作成
/// 7. PRAGMA foreign_key_check → 0件でなければ ROLLBACK + Err
/// 8. schema_versions INSERT（version引数を使用）
/// 9. COMMIT
/// 10. transaction 閉塞確認後、PRAGMA foreign_keys を元の値に復元して再読取検証
pub fn apply_v2_idempotency(conn: &Connection, version: i64) -> Result<(), DbError> {
    // 1. 現在の foreign_keys 設定を保存
    let original_fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|e| {
            DbError::MigrationFailed(format!("v{} foreign_keys読取失敗: {}", version, e))
        })?;

    // 2. foreign_keys OFF（TX外で実行必須）
    conn.execute_batch("PRAGMA foreign_keys = OFF;")
        .map_err(|e| {
            DbError::MigrationFailed(format!("v{} foreign_keys=OFF失敗: {}", version, e))
        })?;

    // 3-9 を inner で実行し、transaction 閉塞時だけ 10 の foreign_keys 復元へ進む
    let result = apply_v2_inner(conn, version);

    // 10. transaction を閉じられない場合は復元を試みず、接続破棄必須として返す。
    if !conn.is_autocommit() {
        let original_error = result
            .as_ref()
            .err()
            .map(ToString::to_string)
            .unwrap_or_else(|| "migration result was unexpectedly successful".to_string());
        tracing::error!(
            version,
            error = %original_error,
            "migration transaction could not be closed; foreign_keys restore skipped and connection must be discarded"
        );
        return Err(DbError::MigrationFailed(format!(
            "v{version} migration失敗: {original_error}（transaction 状態不明のためforeign_keysを復元せず、接続破棄必須）"
        )));
    }

    let restore_result = migration_tx::restore_foreign_keys(conn, version, original_fk);
    match (result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(original_error), Err(restore_error)) => {
            tracing::error!(
                version,
                error = %restore_error,
                original_error = %original_error,
                "migration failed and foreign_keys restore also failed"
            );
            Err(DbError::MigrationFailed(format!(
                "{original_error}; foreign_keys復元失敗も発生: {restore_error}"
            )))
        }
    }
}

/// v2 マイグレーションの本体（foreign_keys OFF/ON の間で実行される）
fn apply_v2_inner(conn: &Connection, version: i64) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    // 3. BEGIN IMMEDIATE
    conn.execute_batch("BEGIN IMMEDIATE;")
        .map_err(|e| DbError::MigrationFailed(format!("v{} BEGIN失敗: {}", version, e)))?;

    // 4-6. 4テーブル再作成
    if let Err(e) = recreate_tables(conn) {
        return Err(migration_tx::rollback_after_error(
            conn,
            format!("v{} テーブル再作成失敗: {}", version, e),
        ));
    }

    // 7. FK 整合性チェック
    let fk_errors: i64 =
        match conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        }) {
            Ok(count) => count,
            Err(e) => {
                return Err(migration_tx::rollback_after_error(
                    conn,
                    format!("v{} FK整合性チェック実行失敗: {}", version, e),
                ));
            }
        };

    if fk_errors != 0 {
        return Err(migration_tx::rollback_after_error(
            conn,
            format!("v{} FK整合性エラー: {}件の違反を検出", version, fk_errors),
        ));
    }

    // 8. schema_versions INSERT（version引数をプレースホルダ経由）
    if let Err(e) = conn.execute(
        "INSERT INTO schema_versions (version, applied_at) VALUES (?1, ?2)",
        rusqlite::params![version, now],
    ) {
        return Err(migration_tx::rollback_after_error(
            conn,
            format!("v{} バージョン記録失敗: {}", version, e),
        ));
    }

    // 9. COMMIT
    migration_tx::commit_transaction(conn, &format!("v{} COMMIT失敗", version))?;

    Ok(())
}

/// 4テーブルを再作成して idempotency_key + request_fingerprint を追加
fn recreate_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    // --- receiving_records ---
    conn.execute_batch(
        "CREATE TABLE receiving_records_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            supplier_id INTEGER REFERENCES suppliers(id),
            receiving_date TEXT NOT NULL,
            note TEXT,
            idempotency_key TEXT NOT NULL CHECK(length(idempotency_key) > 0),
            request_fingerprint TEXT NOT NULL CHECK(length(request_fingerprint) > 0),
            created_at TEXT NOT NULL
        );

        INSERT INTO receiving_records_new (id, supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at)
        SELECT id, supplier_id, receiving_date, note, '__legacy__:' || id, '__legacy__', created_at
        FROM receiving_records;

        DROP TABLE receiving_records;
        ALTER TABLE receiving_records_new RENAME TO receiving_records;
        CREATE UNIQUE INDEX idx_receiving_records_idempotency ON receiving_records(idempotency_key);",
    )?;

    // --- return_records ---
    conn.execute_batch(
        "CREATE TABLE return_records_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            return_type TEXT NOT NULL CHECK(return_type IN ('return','exchange')),
            return_date TEXT NOT NULL,
            register_processed BOOLEAN NOT NULL DEFAULT 1,
            receipt_image_path TEXT,
            note TEXT,
            idempotency_key TEXT NOT NULL CHECK(length(idempotency_key) > 0),
            request_fingerprint TEXT NOT NULL CHECK(length(request_fingerprint) > 0),
            created_at TEXT NOT NULL
        );

        INSERT INTO return_records_new (id, return_type, return_date, register_processed, receipt_image_path, note, idempotency_key, request_fingerprint, created_at)
        SELECT id, return_type, return_date, register_processed, receipt_image_path, note, '__legacy__:' || id, '__legacy__', created_at
        FROM return_records;

        DROP TABLE return_records;
        ALTER TABLE return_records_new RENAME TO return_records;
        CREATE UNIQUE INDEX idx_return_records_idempotency ON return_records(idempotency_key);",
    )?;

    // --- manual_sales ---
    conn.execute_batch(
        "CREATE TABLE manual_sales_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sale_date TEXT NOT NULL,
            reason TEXT NOT NULL CHECK(reason IN ('plu_unregistered','other')),
            note TEXT,
            idempotency_key TEXT NOT NULL CHECK(length(idempotency_key) > 0),
            request_fingerprint TEXT NOT NULL CHECK(length(request_fingerprint) > 0),
            created_at TEXT NOT NULL
        );

        INSERT INTO manual_sales_new (id, sale_date, reason, note, idempotency_key, request_fingerprint, created_at)
        SELECT id, sale_date, reason, note, '__legacy__:' || id, '__legacy__', created_at
        FROM manual_sales;

        DROP TABLE manual_sales;
        ALTER TABLE manual_sales_new RENAME TO manual_sales;
        CREATE UNIQUE INDEX idx_manual_sales_idempotency ON manual_sales(idempotency_key);",
    )?;

    // --- disposal_records ---
    conn.execute_batch(
        "CREATE TABLE disposal_records_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            disposal_date TEXT NOT NULL,
            idempotency_key TEXT NOT NULL CHECK(length(idempotency_key) > 0),
            request_fingerprint TEXT NOT NULL CHECK(length(request_fingerprint) > 0),
            created_at TEXT NOT NULL
        );

        INSERT INTO disposal_records_new (id, disposal_date, idempotency_key, request_fingerprint, created_at)
        SELECT id, disposal_date, '__legacy__:' || id, '__legacy__', created_at
        FROM disposal_records;

        DROP TABLE disposal_records;
        ALTER TABLE disposal_records_new RENAME TO disposal_records;
        CREATE UNIQUE INDEX idx_disposal_records_idempotency ON disposal_records(idempotency_key);",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::init_database;
    use crate::db::migration_tx::{self, FailurePoint};
    use crate::db::schema_v1;
    use crate::db::DbError;
    use rusqlite::Connection;

    /// テスト用: v1-only DB を作成するヘルパー
    fn setup_v1_only_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        // schema_versions テーブル作成
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_versions (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();

        // v1 スキーマ適用
        conn.execute_batch(schema_v1::get_initial_schema()).unwrap();

        // v1 バージョン記録
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (1, '2026-04-06T00:00:00')",
            [],
        )
        .unwrap();

        (dir, conn)
    }

    fn insert_fk_violation(conn: &Connection) {
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        conn.execute(
            "INSERT INTO receiving_records (id, supplier_id, receiving_date, note, created_at) \
             VALUES (99, 99999, '2026-04-06', 'synthetic FK violation', '2026-04-06T00:00:00')",
            [],
        )
        .unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    }

    #[test]
    fn test_v2_req903_d1_commit_and_rollback_failure_requires_connection_discard() {
        // REQ-903 / MNT-03-D1 / Matrix E4
        let (_dir, conn) = setup_v1_only_db();
        let failures = migration_tx::fail_operations(&[
            FailurePoint::Commit,
            FailurePoint::Rollback,
            FailurePoint::ForeignKeyRestore,
        ]);

        let (result, logs) = crate::test_tracing::capture(|| super::apply_v2_idempotency(&conn, 2));

        let DbError::MigrationFailed(message) = result.unwrap_err() else {
            panic!("MigrationFailedを期待")
        };
        assert!(
            message.contains("transaction 状態不明"),
            "message={message}"
        );
        assert!(message.contains("接続破棄必須"), "message={message}");
        assert!(
            !message.contains("foreign_keys restore"),
            "transaction未閉塞時はFK復元を試みてはならない: {message}"
        );
        assert!(!conn.is_autocommit());
        assert!(
            logs.contains("ERROR") && logs.contains("ROLLBACK"),
            "logs={logs:?}"
        );

        drop(failures);
        conn.execute_batch("ROLLBACK;").unwrap();
    }

    #[test]
    fn test_v2_req903_d1_foreign_keys_restore_is_verified() {
        // REQ-903 / MNT-03-D1 / Matrix E5
        let (_dir, conn) = setup_v1_only_db();
        let _failure = migration_tx::fail_operations(&[FailurePoint::ForeignKeyRestoreNoop]);

        let error = super::apply_v2_idempotency(&conn, 2).unwrap_err();

        assert!(matches!(error, DbError::MigrationFailed(message)
            if message.contains("foreign_keys復元検証失敗")
                && message.contains("expected=1")
                && message.contains("actual=0")));
    }

    #[test]
    fn test_v2_req903_d1_foreign_keys_restore_failure_is_logged_and_reported() {
        // REQ-903 / MNT-03-D1 / Matrix E7 (restore failure)
        let (_dir, conn) = setup_v1_only_db();
        insert_fk_violation(&conn);
        let _failure = migration_tx::fail_operations(&[FailurePoint::ForeignKeyRestore]);

        let (result, logs) = crate::test_tracing::capture(|| super::apply_v2_idempotency(&conn, 2));

        let DbError::MigrationFailed(message) = result.unwrap_err() else {
            panic!("MigrationFailedを期待")
        };
        assert!(
            message.contains("FK整合性エラー") && message.contains("foreign_keys復元失敗"),
            "message={message}"
        );
        assert!(
            logs.contains("ERROR") && logs.contains("foreign_keys"),
            "logs={logs:?}"
        );
    }

    #[test]
    fn test_v2_req903_d1_foreign_keys_verify_failure_is_logged_and_reported() {
        // REQ-903 / MNT-03-D1 / Matrix E7 (re-read failure)
        let (_dir, conn) = setup_v1_only_db();
        insert_fk_violation(&conn);
        let _failure = migration_tx::fail_operations(&[FailurePoint::ForeignKeyVerify]);

        let (result, logs) = crate::test_tracing::capture(|| super::apply_v2_idempotency(&conn, 2));

        let DbError::MigrationFailed(message) = result.unwrap_err() else {
            panic!("MigrationFailedを期待")
        };
        assert!(
            message.contains("FK整合性エラー") && message.contains("foreign_keys再読取失敗"),
            "message={message}"
        );
        assert!(
            logs.contains("ERROR") && logs.contains("foreign_keys"),
            "logs={logs:?}"
        );
    }

    /// 新規DBでv1→v2が正常適用されること
    #[test]
    fn test_v2_req903_applied_on_fresh_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        // init_database は最新の版まで適用する
        let max_version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(max_version, crate::db::migration::app_max_version()); // 最新の版（b1 Matrix T1）

        // 4テーブルに idempotency_key カラムが存在する
        for table in &[
            "receiving_records",
            "return_records",
            "manual_sales",
            "disposal_records",
        ] {
            let has_col: bool = conn
                .prepare(&format!("SELECT idempotency_key FROM {} LIMIT 0", table))
                .is_ok();
            assert!(has_col, "{} に idempotency_key カラムが存在しない", table);

            let has_fp: bool = conn
                .prepare(&format!(
                    "SELECT request_fingerprint FROM {} LIMIT 0",
                    table
                ))
                .is_ok();
            assert!(
                has_fp,
                "{} に request_fingerprint カラムが存在しない",
                table
            );
        }
    }

    /// v1既存データからのバックフィル + 親子FK保持確認
    #[test]
    fn test_v2_req903_backfill_with_parent_child_fk() {
        let (_dir, conn) = setup_v1_only_db();

        // v1 スキーマでテストデータを挿入
        // 取引先
        conn.execute(
            "INSERT INTO suppliers (id, name, created_at) VALUES (1, 'テスト取引先', '2026-04-06T00:00:00')",
            [],
        ).unwrap();

        // 商品
        conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price, tax_rate, stock_unit, created_at, updated_at) \
             VALUES ('TEST-001', 'テスト商品', 1, 500, 300, '10', 'pcs', '2026-04-06T00:00:00', '2026-04-06T00:00:00')",
            [],
        ).unwrap();

        // 入庫ヘッダ + 明細
        conn.execute(
            "INSERT INTO receiving_records (id, supplier_id, receiving_date, note, created_at) \
             VALUES (1, 1, '2026-04-06', 'テスト入庫', '2026-04-06T00:00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO receiving_items (id, receiving_record_id, product_code, quantity, cost_price) \
             VALUES (1, 1, 'TEST-001', 10, 300)",
            [],
        ).unwrap();

        // 返品ヘッダ + 明細
        conn.execute(
            "INSERT INTO return_records (id, return_type, return_date, register_processed, note, created_at) \
             VALUES (1, 'return', '2026-04-06', 0, 'テスト返品', '2026-04-06T00:00:00')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO return_items (id, return_record_id, product_code, direction, quantity) \
             VALUES (1, 1, 'TEST-001', 'in', 2)",
            [],
        )
        .unwrap();

        // v2 適用。products に行があるので vB（MNT-03-D14）の直前の版まで（Matrix T1 (b)）
        super::super::migration::migrate_up_to(&conn, crate::db::VB_VERSION - 1).unwrap();

        // バックフィル確認
        let key: String = conn
            .query_row(
                "SELECT idempotency_key FROM receiving_records WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(key, "__legacy__:1");

        let fp: String = conn
            .query_row(
                "SELECT request_fingerprint FROM receiving_records WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(fp, "__legacy__");

        // 返品の バックフィル
        let return_key: String = conn
            .query_row(
                "SELECT idempotency_key FROM return_records WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(return_key, "__legacy__:1");

        // 親子FK保持確認: receiving_items が receiving_records を参照できる
        let item_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM receiving_items ri \
                 INNER JOIN receiving_records rr ON ri.receiving_record_id = rr.id",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(item_count, 1, "入庫: 親子FK保持");

        // 親子FK保持確認: return_items が return_records を参照できる
        let return_item_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM return_items ri \
                 INNER JOIN return_records rr ON ri.return_record_id = rr.id",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(return_item_count, 1, "返品: 親子FK保持");
    }

    /// NOT NULL 制約: idempotency_key に NULL は拒否
    #[test]
    fn test_v2_req903_not_null_constraint() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let result = conn.execute(
            "INSERT INTO receiving_records (supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at) \
             VALUES (NULL, '2026-04-06', NULL, NULL, 'fp', '2026-04-06T00:00:00')",
            [],
        );
        assert!(result.is_err(), "idempotency_key=NULL は拒否されるべき");
    }

    /// CHECK 制約: 空文字は拒否
    #[test]
    fn test_v2_req903_check_constraint_empty_string() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let result = conn.execute(
            "INSERT INTO receiving_records (supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at) \
             VALUES (NULL, '2026-04-06', NULL, '', 'fp', '2026-04-06T00:00:00')",
            [],
        );
        assert!(
            result.is_err(),
            "idempotency_key='' は CHECK で拒否されるべき"
        );

        let result = conn.execute(
            "INSERT INTO receiving_records (supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at) \
             VALUES (NULL, '2026-04-06', NULL, 'key1', '', '2026-04-06T00:00:00')",
            [],
        );
        assert!(
            result.is_err(),
            "request_fingerprint='' は CHECK で拒否されるべき"
        );
    }

    /// UNIQUE 制約: 重複 idempotency_key は拒否
    #[test]
    fn test_v2_req903_unique_constraint_idempotency_key() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        conn.execute(
            "INSERT INTO receiving_records (supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at) \
             VALUES (NULL, '2026-04-06', NULL, 'unique-key-1', 'fp1', '2026-04-06T00:00:00')",
            [],
        ).unwrap();

        let result = conn.execute(
            "INSERT INTO receiving_records (supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at) \
             VALUES (NULL, '2026-04-06', NULL, 'unique-key-1', 'fp2', '2026-04-06T00:00:00')",
            [],
        );
        assert!(
            result.is_err(),
            "重複 idempotency_key は UNIQUE で拒否されるべき"
        );
    }

    /// 再マイグレーション: v2適用済みDBに対して再実行がスキップされること
    #[test]
    fn test_v2_req903_idempotent_rerun() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // 1回目
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(v, crate::db::migration::app_max_version()); // 最新の版（b1 Matrix T1）
        drop(conn);

        // 2回目
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            v,
            crate::db::migration::app_max_version(),
            "バージョンは変わらない"
        );

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_versions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            count,
            crate::db::migration::app_max_version(),
            "v1 から最新の版まで 1 レコードずつ"
        );
    }

    /// foreign_keys=ON 復元テスト（成功経路）
    #[test]
    fn test_v2_req903_foreign_keys_restored_on_success() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1, "v2成功後に foreign_keys=ON が復元されるべき");
    }

    /// v1のみの既存テストが引き続き通ること（リグレッション）
    #[test]
    fn test_v2_req903_existing_v1_tables_intact() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        // departments 初期データ
        let dept_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM departments", [], |row| row.get(0))
            .unwrap();
        assert_eq!(dept_count, 21);

        // app_settings 初期データ
        let settings_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(settings_count, 9);
    }

    /// 4テーブル全てに idempotency UNIQUE INDEX が作成されること
    #[test]
    fn test_v2_req903_idempotency_indexes_exist() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        let expected = vec![
            "idx_receiving_records_idempotency",
            "idx_return_records_idempotency",
            "idx_manual_sales_idempotency",
            "idx_disposal_records_idempotency",
        ];

        for idx in &expected {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?1",
                    rusqlite::params![idx],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "インデックス '{}' が存在しない", idx);
        }
    }

    /// FK不整合データによる migration 失敗時の foreign_keys 復元テスト
    #[test]
    fn test_v2_req903_foreign_keys_restored_on_failure() {
        let (_dir, conn) = setup_v1_only_db();

        // FK不整合データを注入: 存在しない supplier_id = 99999
        insert_fk_violation(&conn);

        // v2 migration 実行 → FK check で失敗するはず
        let result = super::apply_v2_idempotency(&conn, 2);

        // 1. エラーが MigrationFailed で FK 関連メッセージを含む
        assert!(
            matches!(&result, Err(DbError::MigrationFailed(msg)) if msg.contains("FK")),
            "FK整合性エラーで MigrationFailed を期待: {:?}",
            result
        );

        // 2. foreign_keys が ON に復元されている
        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1, "失敗後も foreign_keys=ON が復元されるべき");

        // 3. schema_versions が v1 のまま（v2 未記録）
        let max_version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(max_version, 1, "v2 は記録されていないこと");

        // 4. receiving_records_new テーブルが残っていない（ROLLBACK 保証）
        let new_table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='receiving_records_new'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            !new_table_exists,
            "ROLLBACK により receiving_records_new は残らないこと"
        );
    }
}
