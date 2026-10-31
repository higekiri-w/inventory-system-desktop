//! DB接続管理・リポジトリモジュール群
//!
//! 20-io-product-repo.md §2 に基づく実装。
//! IO-01: SQLiteデータアクセス層

// モジュール宣言（アルファベット順）
pub mod disposal_repo;
pub mod inventory_common;
pub mod inventory_repo;
pub mod manual_sale_repo;
pub mod migration;
mod migration_tx;
pub mod plu_slot_repo;
pub mod product_repo;
pub mod receiving_repo;
pub mod return_repo;
pub mod sales_repo;
pub mod stocktake_repo;
pub mod system_repo;

// 内部モジュール（非公開）
mod schema_v1;
mod schema_v10;
#[cfg(test)]
pub(crate) use schema_v10::{insert_pre_vb_product, VB_VERSION};
mod schema_v11;
mod schema_v12;
mod schema_v2;
mod schema_v3;
mod schema_v4;
mod schema_v5;
mod schema_v6;
mod schema_v7;
mod schema_v8;
mod schema_v9;

#[cfg(test)]
pub(crate) mod test_support;

// BIZ/CMD層で使用する型の集約点として re-export を維持し、未参照シンボルを許可する
#[allow(unused_imports)]
pub use disposal_repo::{
    DisposalRecordDetail, DisposalRecordSummary, InventoryRecordQuery, InventoryRecordSummary,
    NewDisposalItem, NewDisposalRecord,
};
#[allow(unused_imports)]
pub use inventory_common::ListQuery;
#[allow(unused_imports)]
pub use inventory_repo::{
    MovementQuery, MovementRecord, MovementSourceLink, MovementType, NewMovement, ReferenceType,
};
#[allow(unused_imports)]
pub use manual_sale_repo::{
    ManualSaleRecordDetail, ManualSaleRecordDetailItem, NewManualSale, NewManualSaleItem,
};
#[allow(unused_imports)]
pub use product_repo::{
    Department, NewPriceHistory, NewProduct, Product, ProductForPlu, ProductSearchQuery,
    ProductUpdates, ProductWithRelations, SortKey, SortOrder, StockDetail, Supplier,
};
#[allow(unused_imports)]
pub use receiving_repo::{
    NewReceivingItem, NewReceivingRecord, ReceivingRecordDetail, ReceivingRecordDetailItem,
    ReceivingRecordWithSupplier,
};
#[allow(unused_imports)]
pub use return_repo::{
    NewReturnItem, NewReturnRecord, ReturnRecordDetail, ReturnRecordDetailItem, ReturnRecordSummary,
};
#[allow(unused_imports)]
pub use sales_repo::{
    CsvImport, DailyReportImport, DailySaleRow, MonthlySaleDeptRow, MonthlySaleProductRow,
    NewCsvImport, NewCsvImportError, NewDailyReportDepartmentLine, NewDailyReportImport,
    NewDailyReportPaymentLine, NewDailyReportSummaryLine, NewSaleRecord, VoidedMovement,
};
#[allow(unused_imports)]
pub use stocktake_repo::{
    LastStocktakeSummary, NewStocktakeItem, ProductForStocktake, Stocktake, StocktakeItem,
    StocktakeItemRow, StocktakeProgress, UncountedItem,
};
#[allow(unused_imports)]
pub use system_repo::{AppSetting, NewOperationLog, OperationLog};

use std::fmt;

/// DB接続の型エイリアス
///
/// 20-io-product-repo.md に基づく。rusqlite::Connection をそのまま使う。
/// トランザクション管理はBIZ層が conn.transaction() で制御する。
pub type DbConnection = rusqlite::Connection;

/// DB層のエラー型
///
/// 20-io-product-repo.md §2.10
#[derive(Debug)]
pub enum DbError {
    /// DB接続の確立に失敗
    ConnectionFailed(String),
    /// PRAGMA設定に失敗
    PragmaFailed(String),
    /// マイグレーション実行に失敗
    MigrationFailed(String),
    /// SQLクエリ実行に失敗
    QueryFailed(String),
    /// 主キーまたはユニーク制約違反
    DuplicateKey(String),
    /// 外部キー制約違反
    ForeignKeyViolation(String),
    /// レコードが見つからない
    NotFound,
    /// DB の schema 版がアプリの扱える最大より新しい（MNT-03-D11）
    SchemaNewerThanApp { db_version: i64, app_max: i64 },
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::ConnectionFailed(msg) => write!(f, "DB接続エラー: {}", msg),
            DbError::PragmaFailed(msg) => write!(f, "PRAGMA設定エラー: {}", msg),
            DbError::MigrationFailed(msg) => write!(f, "マイグレーションエラー: {}", msg),
            DbError::QueryFailed(msg) => write!(f, "クエリ実行エラー: {}", msg),
            DbError::DuplicateKey(key) => write!(f, "重複キー: {}", key),
            DbError::ForeignKeyViolation(msg) => write!(f, "外部キー制約違反: {}", msg),
            DbError::NotFound => write!(f, "レコードが見つかりません"),
            DbError::SchemaNewerThanApp {
                db_version,
                app_max,
            } => write!(
                f,
                "データの版がこのアプリより新しいため開けません（データの版: {}、このアプリが扱える版: {}）",
                db_version, app_max
            ),
        }
    }
}

impl std::error::Error for DbError {}

/// rusqlite::Error → DbError の変換
///
/// SQLiteのエラーコードに基づいて適切なDbErrorバリアントに変換する:
/// - SQLITE_CONSTRAINT_PRIMARYKEY (1555) / SQLITE_CONSTRAINT_UNIQUE (2067) → DuplicateKey
/// - SQLITE_CONSTRAINT_FOREIGNKEY (787) → ForeignKeyViolation
/// - その他 → QueryFailed
impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        match &err {
            rusqlite::Error::SqliteFailure(sqlite_err, msg) => {
                let code = sqlite_err.extended_code;
                if code == 1555 || code == 2067 {
                    // SQLITE_CONSTRAINT_PRIMARYKEY / SQLITE_CONSTRAINT_UNIQUE
                    DbError::DuplicateKey(msg.clone().unwrap_or_default())
                } else if code == 787 {
                    // SQLITE_CONSTRAINT_FOREIGNKEY
                    DbError::ForeignKeyViolation(msg.clone().unwrap_or_default())
                } else {
                    DbError::QueryFailed(err.to_string())
                }
            }
            _ => DbError::QueryFailed(err.to_string()),
        }
    }
}

/// ページネーション付き検索結果
///
/// 20-io-product-repo.md §2.10
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: u32,
    pub page: u32,
    pub per_page: u32,
}

/// DB接続を初期化する
///
/// 20-io-product-repo.md §2.2:
/// SQLiteファイルを開き、PRAGMA設定とマイグレーションを実行して、
/// 使用可能な接続を返す。
pub fn init_database(db_path: &str) -> Result<DbConnection, DbError> {
    // 1. ファイルを開く（なければ新規作成）
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| DbError::ConnectionFailed(e.to_string()))?;

    configure_database(conn, db_path)
}

/// 既存DBのみを開く。障害復旧経路で空DBを新規生成しないための NO_CREATE 接続。
pub(crate) fn open_existing_database(db_path: &str) -> Result<DbConnection, DbError> {
    let conn =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|e| DbError::ConnectionFailed(e.to_string()))?;
    configure_database(conn, db_path)
}

fn configure_database(conn: DbConnection, db_path: &str) -> Result<DbConnection, DbError> {
    // 2. PRAGMA foreign_keys = ON
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| DbError::PragmaFailed(format!("foreign_keys: {}", e)))?;

    // 3. PRAGMA journal_mode = WAL
    conn.execute_batch("PRAGMA journal_mode = WAL;")
        .map_err(|e| DbError::PragmaFailed(format!("journal_mode: {}", e)))?;

    // 4. PRAGMA busy_timeout = 5000
    conn.execute_batch("PRAGMA busy_timeout = 5000;")
        .map_err(|e| DbError::PragmaFailed(format!("busy_timeout: {}", e)))?;

    // 5. マイグレーション実行（MNT-03）
    migration::migrate(&conn)?;

    // 6. 初期化完了ログ（§70.7.4）
    let schema_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_versions",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    tracing::info!(schema_version, db_path, "DB初期化完了");

    Ok(conn)
}

/// 旧パス（CWD/inventory.db）から新パス（app_data_dir/inventory.db）へDB移行する
///
/// 旧DBを NO_CREATE で開き、VACUUM INTO により WAL を含む単一 snapshot を一時名へ生成する。
/// 完成後は no-clobber publish し、旧DB一式は保持する（MNT-03-D2/D3/D4）。
///
/// 戻り値: Ok(true) = 移行実行、Ok(false) = 移行不要（旧DB無し or 新DB既存）
pub fn migrate_legacy_db(
    old_dir: &std::path::Path,
    new_dir: &std::path::Path,
) -> Result<bool, std::io::Error> {
    migrate_legacy_db_with_ops(old_dir, new_dir, &StdLegacyMigrationOps)
}

trait LegacyMigrationOps {
    fn try_exists(&self, path: &std::path::Path) -> std::io::Result<bool>;
    fn before_open(&self, _path: &std::path::Path) -> std::io::Result<()> {
        Ok(())
    }
    fn vacuum_into(
        &self,
        conn: &rusqlite::Connection,
        destination: &std::path::Path,
    ) -> std::io::Result<()>;
    fn publish_no_clobber(
        &self,
        source: &std::path::Path,
        destination: &std::path::Path,
    ) -> std::io::Result<()>;
    fn remove_file(&self, path: &std::path::Path) -> std::io::Result<()>;
}

struct StdLegacyMigrationOps;

impl LegacyMigrationOps for StdLegacyMigrationOps {
    fn try_exists(&self, path: &std::path::Path) -> std::io::Result<bool> {
        path.try_exists()
    }

    fn vacuum_into(
        &self,
        conn: &rusqlite::Connection,
        destination: &std::path::Path,
    ) -> std::io::Result<()> {
        let escaped = destination.to_string_lossy().replace('\'', "''");
        conn.execute_batch(&format!("VACUUM INTO '{escaped}'"))
            .map_err(sqlite_io_error)
    }

    fn publish_no_clobber(
        &self,
        source: &std::path::Path,
        destination: &std::path::Path,
    ) -> std::io::Result<()> {
        // hard_link の destination 作成は Windows / Unix とも既存名を置換しない。
        std::fs::hard_link(source, destination)
    }

    fn remove_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        std::fs::remove_file(path)
    }
}

fn sqlite_io_error(error: rusqlite::Error) -> std::io::Error {
    std::io::Error::other(error.to_string())
}

fn migrate_legacy_db_with_ops(
    old_dir: &std::path::Path,
    new_dir: &std::path::Path,
    ops: &dyn LegacyMigrationOps,
) -> Result<bool, std::io::Error> {
    let old_db = old_dir.join("inventory.db");
    let new_db = new_dir.join("inventory.db");
    let staging_db = new_dir.join("inventory.db.migrating");

    // try_exists の I/O error は skip に変換しない（MNT-03-D4）。
    if ops.try_exists(&new_db)? {
        return Ok(false);
    }
    if !ops.try_exists(&old_db)? {
        return Ok(false);
    }

    // 前回失敗の一時ファイルだけを掃除する。最終名には一切触れない。
    if ops.try_exists(&staging_db)? {
        ops.remove_file(&staging_db)?;
    }

    ops.before_open(&old_db)?;
    let old_conn =
        rusqlite::Connection::open_with_flags(&old_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(sqlite_io_error)?;

    let result = (|| {
        ops.vacuum_into(&old_conn, &staging_db)?;
        // 直前再確認に加え、publish primitive 自体も no-clobber にして TOCTOU を閉じる。
        if ops.try_exists(&new_db)? {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "migration destination appeared before publish",
            ));
        }
        ops.publish_no_clobber(&staging_db, &new_db)?;
        ops.remove_file(&staging_db)?;
        Ok(true)
    })();

    if result.is_err() && ops.try_exists(&staging_db).unwrap_or(false) {
        if let Err(error) = ops.remove_file(&staging_db) {
            tracing::warn!(path = %staging_db.display(), %error, "失敗した移行一時ファイルの削除に失敗");
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DB初期化が成功し、PRAGMAが正しく設定されること
    #[test]
    fn test_init_database_req903_pragmas() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        // PRAGMA foreign_keys = ON
        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1, "foreign_keys should be ON");

        // PRAGMA journal_mode = WAL
        let journal: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal, "wal", "journal_mode should be WAL");

        // PRAGMA busy_timeout = 5000
        let timeout: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000, "busy_timeout should be 5000");
    }

    /// 同じDBに対して2回初期化しても成功すること（再マイグレーションスキップ）
    #[test]
    fn test_init_database_req903_twice_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let _conn1 = init_database(db_path.to_str().unwrap()).unwrap();
        drop(_conn1);
        let _conn2 = init_database(db_path.to_str().unwrap()).unwrap();
    }

    /// DbError の From<rusqlite::Error> 変換テスト
    #[test]
    fn test_db_error_req903_from_rusqlite_duplicate_key() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();

        // 同じPKで2回INSERTして DuplicateKey エラーを発生させる
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('test_key', 'v1', '2026-04-03T00:00:00')",
            [],
        )
        .unwrap();

        let result = conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('test_key', 'v2', '2026-04-03T00:00:00')",
            [],
        );

        let err: DbError = result.unwrap_err().into();
        assert!(
            matches!(err, DbError::DuplicateKey(_)),
            "重複キーエラーが DuplicateKey に変換されるべき: {:?}",
            err
        );
    }

    // -----------------------------------------------------------------------
    // migrate_legacy_db テスト（MNT-03-D2/D3/D4）
    // -----------------------------------------------------------------------

    fn create_legacy_wal_fixture(path: &std::path::Path) -> rusqlite::Connection {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA wal_autocheckpoint=0;
             CREATE TABLE legacy_rows (value TEXT NOT NULL);
             INSERT INTO legacy_rows VALUES ('main');
             PRAGMA wal_checkpoint(TRUNCATE);
             INSERT INTO legacy_rows VALUES ('wal-frame');",
        )
        .unwrap();
        let wal = std::path::PathBuf::from(format!("{}-wal", path.display()));
        assert!(
            std::fs::metadata(wal).unwrap().len() > 32,
            "WAL frame fixture required"
        );
        conn
    }

    #[derive(Clone, Copy)]
    enum LegacyFailpoint {
        NewExists,
        OldExists,
        Vacuum,
        Publish,
        StagingUnlink,
        DestinationRace,
        DeleteBeforeOpen,
    }

    struct InjectedLegacyOps {
        failpoint: LegacyFailpoint,
        old_db: std::path::PathBuf,
        new_db: std::path::PathBuf,
    }

    impl InjectedLegacyOps {
        fn new(
            failpoint: LegacyFailpoint,
            old_dir: &std::path::Path,
            new_dir: &std::path::Path,
        ) -> Self {
            Self {
                failpoint,
                old_db: old_dir.join("inventory.db"),
                new_db: new_dir.join("inventory.db"),
            }
        }
    }

    struct PrepublishRaceOps {
        destination: std::path::PathBuf,
        destination_checks: std::sync::atomic::AtomicUsize,
        publish_called: std::sync::atomic::AtomicBool,
    }

    impl PrepublishRaceOps {
        fn new(destination: std::path::PathBuf) -> Self {
            Self {
                destination,
                destination_checks: std::sync::atomic::AtomicUsize::new(0),
                publish_called: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    impl LegacyMigrationOps for PrepublishRaceOps {
        fn try_exists(&self, path: &std::path::Path) -> std::io::Result<bool> {
            if path == self.destination {
                let check = self
                    .destination_checks
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return Ok(check >= 1);
            }
            path.try_exists()
        }
        fn vacuum_into(
            &self,
            conn: &rusqlite::Connection,
            destination: &std::path::Path,
        ) -> std::io::Result<()> {
            StdLegacyMigrationOps.vacuum_into(conn, destination)
        }
        fn publish_no_clobber(
            &self,
            source: &std::path::Path,
            destination: &std::path::Path,
        ) -> std::io::Result<()> {
            self.publish_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            StdLegacyMigrationOps.publish_no_clobber(source, destination)
        }
        fn remove_file(&self, path: &std::path::Path) -> std::io::Result<()> {
            StdLegacyMigrationOps.remove_file(path)
        }
    }

    impl LegacyMigrationOps for InjectedLegacyOps {
        fn try_exists(&self, path: &std::path::Path) -> std::io::Result<bool> {
            if (matches!(self.failpoint, LegacyFailpoint::NewExists) && path == self.new_db)
                || (matches!(self.failpoint, LegacyFailpoint::OldExists) && path == self.old_db)
            {
                return Err(std::io::Error::other("injected exists failure"));
            }
            path.try_exists()
        }
        fn before_open(&self, path: &std::path::Path) -> std::io::Result<()> {
            if matches!(self.failpoint, LegacyFailpoint::DeleteBeforeOpen) {
                std::fs::remove_file(path)?;
            }
            Ok(())
        }
        fn vacuum_into(
            &self,
            conn: &rusqlite::Connection,
            destination: &std::path::Path,
        ) -> std::io::Result<()> {
            if matches!(self.failpoint, LegacyFailpoint::Vacuum) {
                return Err(std::io::Error::other("injected VACUUM failure"));
            }
            StdLegacyMigrationOps.vacuum_into(conn, destination)
        }
        fn publish_no_clobber(
            &self,
            source: &std::path::Path,
            destination: &std::path::Path,
        ) -> std::io::Result<()> {
            if matches!(self.failpoint, LegacyFailpoint::Publish) {
                return Err(std::io::Error::other("injected publish failure"));
            }
            if matches!(self.failpoint, LegacyFailpoint::DestinationRace) {
                std::fs::write(destination, b"racing destination")?;
            }
            StdLegacyMigrationOps.publish_no_clobber(source, destination)
        }
        fn remove_file(&self, path: &std::path::Path) -> std::io::Result<()> {
            if matches!(self.failpoint, LegacyFailpoint::StagingUnlink)
                && path.ends_with("inventory.db.migrating")
                && self.new_db.exists()
            {
                return Err(std::io::Error::other("injected staging unlink failure"));
            }
            StdLegacyMigrationOps.remove_file(path)
        }
    }

    #[test]
    fn test_migrate_legacy_db_req903_copies_all_files() {
        // REQ-903 / Matrix M1: 実 WAL frame を単一 snapshot へ統合
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let old_db = old_dir.path().join("inventory.db");
        let source_conn = create_legacy_wal_fixture(&old_db);

        assert!(migrate_legacy_db(old_dir.path(), new_dir.path()).unwrap());
        let migrated = rusqlite::Connection::open(new_dir.path().join("inventory.db")).unwrap();
        let rows: i64 = migrated
            .query_row("SELECT COUNT(*) FROM legacy_rows", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 2, "WAL 内 row も移行される");
        assert!(old_db.exists(), "旧 snapshot は保持する");
        drop(source_conn);
    }

    #[test]
    fn test_migrate_legacy_db_req903_skips_when_new_exists() {
        // REQ-903 / Matrix M7
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        std::fs::write(old_dir.path().join("inventory.db"), "old").unwrap();
        std::fs::write(new_dir.path().join("inventory.db"), "new").unwrap();
        assert!(!migrate_legacy_db(old_dir.path(), new_dir.path()).unwrap());
        assert_eq!(
            std::fs::read(new_dir.path().join("inventory.db")).unwrap(),
            b"new"
        );
    }

    #[test]
    fn test_migrate_legacy_db_req903_skips_when_old_missing() {
        // REQ-903 / Matrix M7
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        assert!(!migrate_legacy_db(old_dir.path(), new_dir.path()).unwrap());
        assert!(!new_dir.path().join("inventory.db").exists());
    }

    #[test]
    fn test_migrate_legacy_db_req903_without_wal_shm() {
        // REQ-903: main-only fixture remains supported
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let source = rusqlite::Connection::open(old_dir.path().join("inventory.db")).unwrap();
        source
            .execute_batch("CREATE TABLE old_data(value TEXT);")
            .unwrap();
        drop(source);
        assert!(migrate_legacy_db(old_dir.path(), new_dir.path()).unwrap());
        assert!(new_dir.path().join("inventory.db").exists());
        assert!(!new_dir.path().join("inventory.db.migrating").exists());
    }

    #[test]
    fn test_migrate_legacy_db_req903_failure_injection_is_retryable() {
        // REQ-903 / Matrix M2, M3a
        for failpoint in [LegacyFailpoint::Vacuum, LegacyFailpoint::Publish] {
            let old_dir = tempfile::tempdir().unwrap();
            let new_dir = tempfile::tempdir().unwrap();
            let source = rusqlite::Connection::open(old_dir.path().join("inventory.db")).unwrap();
            source
                .execute_batch("CREATE TABLE retry_data(value TEXT);")
                .unwrap();
            drop(source);
            assert!(migrate_legacy_db_with_ops(
                old_dir.path(),
                new_dir.path(),
                &InjectedLegacyOps::new(failpoint, old_dir.path(), new_dir.path())
            )
            .is_err());
            assert!(!new_dir.path().join("inventory.db").exists());
            assert!(!new_dir.path().join("inventory.db.migrating").exists());
            assert!(migrate_legacy_db(old_dir.path(), new_dir.path()).unwrap());
        }
    }

    #[test]
    fn test_migrate_legacy_db_req903_publish_is_no_clobber() {
        // REQ-903 / Matrix M4
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let source = rusqlite::Connection::open(old_dir.path().join("inventory.db")).unwrap();
        source
            .execute_batch("CREATE TABLE race_data(value TEXT);")
            .unwrap();
        drop(source);
        assert!(migrate_legacy_db_with_ops(
            old_dir.path(),
            new_dir.path(),
            &InjectedLegacyOps::new(
                LegacyFailpoint::DestinationRace,
                old_dir.path(),
                new_dir.path(),
            )
        )
        .is_err());
        assert_eq!(
            std::fs::read(new_dir.path().join("inventory.db")).unwrap(),
            b"racing destination"
        );

        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let source = rusqlite::Connection::open(old_dir.path().join("inventory.db")).unwrap();
        source
            .execute_batch("CREATE TABLE prepublish_race(value TEXT);")
            .unwrap();
        drop(source);
        let destination = new_dir.path().join("inventory.db");
        let ops = PrepublishRaceOps::new(destination.clone());
        assert!(migrate_legacy_db_with_ops(old_dir.path(), new_dir.path(), &ops).is_err());
        assert!(!ops.publish_called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!destination.exists());
        assert!(!new_dir.path().join("inventory.db.migrating").exists());
    }

    #[test]
    fn test_migrate_legacy_db_req903_new_and_old_exists_errors_are_not_skipped() {
        // REQ-903 / Matrix M5
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let old_db = old_dir.path().join("inventory.db");
        let source = rusqlite::Connection::open(&old_db).unwrap();
        source
            .execute_batch("CREATE TABLE toctou_data(value TEXT);")
            .unwrap();
        drop(source);
        assert!(migrate_legacy_db_with_ops(
            old_dir.path(),
            new_dir.path(),
            &InjectedLegacyOps::new(LegacyFailpoint::NewExists, old_dir.path(), new_dir.path(),)
        )
        .is_err());
        assert!(migrate_legacy_db_with_ops(
            old_dir.path(),
            new_dir.path(),
            &InjectedLegacyOps::new(LegacyFailpoint::OldExists, old_dir.path(), new_dir.path(),)
        )
        .is_err());
    }

    #[test]
    fn test_migrate_legacy_db_req903_no_create_after_exists_check() {
        // REQ-903 / Matrix M6
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let old_db = old_dir.path().join("inventory.db");
        let source = rusqlite::Connection::open(&old_db).unwrap();
        source
            .execute_batch("CREATE TABLE toctou_data(value TEXT);")
            .unwrap();
        drop(source);

        assert!(migrate_legacy_db_with_ops(
            old_dir.path(),
            new_dir.path(),
            &InjectedLegacyOps::new(
                LegacyFailpoint::DeleteBeforeOpen,
                old_dir.path(),
                new_dir.path(),
            )
        )
        .is_err());
        assert!(
            !old_db.exists(),
            "NO_CREATE open must not recreate the source"
        );
        assert!(!new_dir.path().join("inventory.db").exists());
    }

    #[test]
    fn test_migrate_legacy_db_req903_m3b_staging_unlink_failure_self_heals_on_restart() {
        // REQ-903 / Matrix M3b: publish 済み snapshot を保持して fail-closed、次回は new DB skip。
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let old_db = old_dir.path().join("inventory.db");
        let new_db = new_dir.path().join("inventory.db");
        let source = rusqlite::Connection::open(&old_db).unwrap();
        source
            .execute_batch(
                "CREATE TABLE source_data(value TEXT);
                 INSERT INTO source_data(value) VALUES ('complete snapshot');",
            )
            .unwrap();
        drop(source);

        let result = migrate_legacy_db_with_ops(
            old_dir.path(),
            new_dir.path(),
            &InjectedLegacyOps::new(
                LegacyFailpoint::StagingUnlink,
                old_dir.path(),
                new_dir.path(),
            ),
        );
        assert!(result.is_err(), "post-link unlink failure must fail closed");
        assert!(new_db.exists(), "published snapshot remains complete");

        assert!(!migrate_legacy_db(old_dir.path(), new_dir.path()).unwrap());
        let reopened = init_database(new_db.to_str().unwrap()).unwrap();
        let value: String = reopened
            .query_row("SELECT value FROM source_data", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "complete snapshot");
    }

    #[test]
    fn test_open_existing_database_req903_never_creates_missing_file() {
        // REQ-903 / MNT-03-D4 / Matrix M6
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.db");
        assert!(open_existing_database(path.to_str().unwrap()).is_err());
        assert!(!path.exists(), "NO_CREATE open must not create an empty DB");
    }
}
