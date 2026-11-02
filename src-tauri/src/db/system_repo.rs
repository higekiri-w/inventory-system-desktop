//! 操作ログ・アプリ設定のCRUD操作
//!
//! 20-io-product-repo.md §2.8 に基づく実装。
//! IO-01: SQLiteデータアクセス層（system_repository）

use super::{DbConnection, DbError};
use crate::constants::PAGINATION_MAX_PER_PAGE;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 操作ログINSERT用
///
/// 20-io-product-repo.md §2.8
///
/// operation_type は種類が多く今後も増えるため enum にしない。
/// 命名規約（エンティティ_動詞）は BIZ 層で担保する。
#[derive(Debug)]
pub struct NewOperationLog {
    pub operation_type: String,
    pub summary: String,
    pub detail_json: Option<String>,
}

/// アプリ設定の行マッピング（第6段階追加）
///
/// 20-io-product-repo.md §2.8
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

/// 操作ログの行マッピング（第6段階追加）
///
/// 20-io-product-repo.md §2.8
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct OperationLog {
    pub id: i64,
    pub operation_type: String,
    pub summary: String,
    pub detail_json: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// operation_logs に1行INSERTする
///
/// 20-io-product-repo.md §2.8
pub fn insert_operation_log(conn: &DbConnection, log: &NewOperationLog) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO operation_logs (operation_type, summary, detail_json, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![log.operation_type, log.summary, log.detail_json, now],
    )?;
    Ok(())
}

/// app_settingsからキーに対応する値を取得する（第6段階追加）
///
/// 20-io-product-repo.md §2.8
pub fn get_setting(conn: &DbConnection, key: &str) -> Result<Option<String>, DbError> {
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
    let result = stmt.query_row(rusqlite::params![key], |row| row.get::<_, String>(0));
    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::from(e)),
    }
}

/// app_settingsの全キー・値ペアを取得する（第6段階追加）
///
/// 20-io-product-repo.md §2.8
pub fn get_all_settings(conn: &DbConnection) -> Result<Vec<AppSetting>, DbError> {
    let mut stmt = conn.prepare("SELECT key, value, updated_at FROM app_settings ORDER BY key")?;
    let rows = stmt.query_map([], |row| {
        Ok(AppSetting {
            key: row.get(0)?,
            value: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;
    let mut settings = Vec::new();
    for row in rows {
        settings.push(row?);
    }
    Ok(settings)
}

/// app_settingsにキー・値を挿入または更新する（第6段階追加）
///
/// 20-io-product-repo.md §2.8
pub fn upsert_setting(conn: &DbConnection, key: &str, value: &str) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
        rusqlite::params![key, value, now],
    )?;
    Ok(())
}

/// 操作ログをページング付きで取得する（第6段階追加）
///
/// 20-io-product-repo.md §2.8
pub fn list_operation_logs(
    conn: &DbConnection,
    page: u32,
    per_page: u32,
    operation_type: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<super::PaginatedResult<OperationLog>, DbError> {
    if page < 1 {
        return Err(DbError::QueryFailed("page must be >= 1".to_string()));
    }
    let per_page = per_page.min(PAGINATION_MAX_PER_PAGE);
    if per_page < 1 {
        return Err(DbError::QueryFailed("per_page must be >= 1".to_string()));
    }

    let page_i64 = i64::from(page);
    let per_page_i64 = i64::from(per_page);
    let offset = (page_i64 - 1) * per_page_i64;

    let mut conditions = Vec::new();
    let mut filter_values = Vec::new();
    if let Some(value) = operation_type {
        filter_values.push(value.to_string());
        conditions.push(format!("operation_type = ?{}", filter_values.len()));
    }
    if let Some(value) = start_date {
        filter_values.push(format!("{value}T00:00:00"));
        conditions.push(format!("created_at >= ?{}", filter_values.len()));
    }
    if let Some(value) = end_date {
        let next = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|e| DbError::QueryFailed(format!("invalid end_date: {e}")))?
            .succ_opt()
            .ok_or_else(|| DbError::QueryFailed("end_date out of range".to_string()))?;
        filter_values.push(format!("{}T00:00:00", next.format("%Y-%m-%d")));
        conditions.push(format!("created_at < ?{}", filter_values.len()));
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // データ取得
    let data_sql = format!(
        "SELECT id, operation_type, summary, detail_json, created_at
         FROM operation_logs {}
         ORDER BY created_at DESC, id DESC
         LIMIT ?{} OFFSET ?{}",
        where_clause,
        filter_values.len() + 1,
        filter_values.len() + 2,
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = filter_values
        .iter()
        .cloned()
        .map(|v| Box::new(v) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    params.push(Box::new(per_page_i64));
    params.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&data_sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(OperationLog {
            id: row.get(0)?,
            operation_type: row.get(1)?,
            summary: row.get(2)?,
            detail_json: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    // 件数取得（同じWHERE句）
    let count_params: Vec<Box<dyn rusqlite::types::ToSql>> = filter_values
        .into_iter()
        .map(|v| Box::new(v) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    let count_refs: Vec<&dyn rusqlite::types::ToSql> =
        count_params.iter().map(|p| p.as_ref()).collect();

    let count_sql = format!("SELECT COUNT(*) FROM operation_logs {where_clause}");
    let total_count: u32 = conn.query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))?;

    Ok(super::PaginatedResult {
        items,
        total_count,
        page,
        per_page,
    })
}

pub fn find_distinct_operation_types(conn: &DbConnection) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT operation_type FROM operation_logs ORDER BY operation_type ASC",
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// 指定日数を超えた古い操作ログを削除する（第6段階追加）
///
/// 20-io-product-repo.md §2.8
pub fn delete_old_logs(conn: &DbConnection, retention_days: u32) -> Result<usize, DbError> {
    // MNT-02-D1: 日付の範囲の外（引けない）なら、それより前の記録は無いので 0 件
    let Some(cutoff) = chrono::Local::now()
        .date_naive()
        .checked_sub_days(chrono::Days::new(u64::from(retention_days)))
    else {
        return Ok(0);
    };
    let cutoff_str = format!("{}T00:00:00", cutoff.format("%Y-%m-%d"));

    conn.execute(
        "DELETE FROM operation_logs WHERE created_at < ?1",
        rusqlite::params![cutoff_str],
    )?;

    Ok(conn.changes() as usize)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_database;

    fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    #[test]
    fn test_insert_operation_log_req902_with_detail() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: insert_operation_log — detail_json ありで正常INSERT
        let (_dir, conn) = setup_test_db();

        let log = NewOperationLog {
            operation_type: "product_create".to_string(),
            summary: "商品を登録しました: テスト商品 (TEST-001)".to_string(),
            detail_json: Some(r#"{"product_code":"TEST-001"}"#.to_string()),
        };
        insert_operation_log(&conn, &log).unwrap();

        let (op_type, summary, detail, created_at): (String, String, Option<String>, String) = conn
            .query_row(
                "SELECT operation_type, summary, detail_json, created_at
                 FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(op_type, "product_create");
        assert_eq!(summary, "商品を登録しました: テスト商品 (TEST-001)");
        assert!(detail.is_some());
        // created_at が ISO 8601 形式
        chrono::NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%dT%H:%M:%S")
            .expect("created_at は %Y-%m-%dT%H:%M:%S 形式であるべき");
    }

    #[test]
    fn test_insert_operation_log_req902_without_detail() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: insert_operation_log — detail_json = None で正常INSERT
        let (_dir, conn) = setup_test_db();

        let log = NewOperationLog {
            operation_type: "product_discontinue".to_string(),
            summary: "商品を廃番にしました".to_string(),
            detail_json: None,
        };
        insert_operation_log(&conn, &log).unwrap();

        let detail: Option<String> = conn
            .query_row(
                "SELECT detail_json FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(detail.is_none(), "detail_json は NULL");
    }

    #[test]
    fn test_insert_operation_log_req902_multiple() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: insert_operation_log — 複数件INSERTで件数確認
        let (_dir, conn) = setup_test_db();

        for i in 0..3 {
            let log = NewOperationLog {
                operation_type: "product_create".to_string(),
                summary: format!("商品{}", i),
                detail_json: None,
            };
            insert_operation_log(&conn, &log).unwrap();
        }

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'product_create'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    // -----------------------------------------------------------------------
    // get_setting（第6段階追加）
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_setting_req905_existing_key() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // FUNC-2.8: get_setting — 初期データのキーを取得できる
        let (_dir, conn) = setup_test_db();

        let value = get_setting(&conn, "stock_low_threshold").unwrap();
        assert_eq!(value, Some("3".to_string()));
    }

    #[test]
    fn test_get_setting_req905_missing_key() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // FUNC-2.8: get_setting — 存在しないキー → None
        let (_dir, conn) = setup_test_db();

        let value = get_setting(&conn, "nonexistent_key").unwrap();
        assert!(value.is_none(), "存在しないキーは None");
    }

    // -----------------------------------------------------------------------
    // get_all_settings（第6段階追加）
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_all_settings_req905_returns_initial() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // FUNC-2.8: get_all_settings — 初期設定9件が全件取得される（ORDER BY key）
        let (_dir, conn) = setup_test_db();

        let settings = get_all_settings(&conn).unwrap();
        // schema_v1.rs の初期INSERTは9件
        assert_eq!(settings.len(), 9, "初期設定は9件");

        // ORDER BY key でソートされているか確認
        let keys: Vec<&str> = settings.iter().map(|s| s.key.as_str()).collect();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(
            keys, sorted_keys,
            "キーがアルファベット順にソートされている"
        );

        // 既知のキーが含まれているか
        assert!(
            keys.contains(&"backup_enabled"),
            "backup_enabled が含まれるべき"
        );
        assert!(
            keys.contains(&"log_retention_days"),
            "log_retention_days が含まれるべき"
        );
    }

    // -----------------------------------------------------------------------
    // upsert_setting（第6段階追加）
    // -----------------------------------------------------------------------

    #[test]
    fn test_upsert_setting_req905_insert_new() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // FUNC-2.8: upsert_setting — 新規キーの挿入
        let (_dir, conn) = setup_test_db();

        upsert_setting(&conn, "new_key", "new_value").unwrap();

        let value = get_setting(&conn, "new_key").unwrap();
        assert_eq!(value, Some("new_value".to_string()));
    }

    #[test]
    fn test_upsert_setting_req905_update_existing() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // FUNC-2.8: upsert_setting — 既存キーの更新（value + updated_at変更確認）
        let (_dir, conn) = setup_test_db();

        // 更新前のupdated_atを取得
        let before: String = conn
            .query_row(
                "SELECT updated_at FROM app_settings WHERE key = 'backup_time'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        upsert_setting(&conn, "backup_time", "08:00").unwrap();

        let value = get_setting(&conn, "backup_time").unwrap();
        assert_eq!(value, Some("08:00".to_string()));

        let after: String = conn
            .query_row(
                "SELECT updated_at FROM app_settings WHERE key = 'backup_time'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_ne!(before, after, "updated_at が更新されるべき");
    }

    // -----------------------------------------------------------------------
    // list_operation_logs（第6段階追加）
    // -----------------------------------------------------------------------

    /// テスト用にoperation_logsにN件挿入するヘルパー
    fn insert_test_logs(conn: &super::DbConnection, count: usize, op_type: &str) {
        for i in 0..count {
            let log = NewOperationLog {
                operation_type: op_type.to_string(),
                summary: format!("テストログ {}", i),
                detail_json: None,
            };
            insert_operation_log(conn, &log).unwrap();
        }
    }

    fn insert_dated_log(conn: &super::DbConnection, op_type: &str, created_at: &str) {
        conn.execute(
            "INSERT INTO operation_logs (operation_type, summary, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![op_type, format!("{op_type}-{created_at}"), created_at],
        )
        .unwrap();
    }

    #[test]
    fn test_list_operation_logs_req902_date_range_row_count_predicate_equivalence() {
        // REQ-902 / UI-11c-D2/D3 / D-037
        let (_dir, conn) = setup_test_db();
        insert_dated_log(&conn, "product_create", "2026-07-09T23:59:59");
        insert_dated_log(&conn, "product_create", "2026-07-10T00:00:00");
        insert_dated_log(&conn, "product_create", "2026-07-10T23:59:59");
        insert_dated_log(&conn, "product_create", "2026-07-11T00:00:00");
        insert_dated_log(&conn, "backup_create", "2026-07-10T12:00:00");

        let result = list_operation_logs(
            &conn,
            1,
            1,
            Some("product_create"),
            Some("2026-07-10"),
            Some("2026-07-10"),
        )
        .unwrap();
        assert_eq!(
            result.items.len(),
            1,
            "items fixture is intentionally paged"
        );
        assert_eq!(
            result.total_count, 2,
            "count uses the identical compound predicate"
        );
        assert!(result.items[0].created_at.starts_with("2026-07-10"));
    }

    #[test]
    fn test_list_operation_logs_req902_one_sided_and_end_exclusive() {
        // REQ-902 / UI-11c-D2 / D-037
        let (_dir, conn) = setup_test_db();
        insert_dated_log(&conn, "unknown_future_type", "2026-07-09T23:59:59");
        insert_dated_log(&conn, "unknown_future_type", "2026-07-10T00:00:00");
        insert_dated_log(&conn, "unknown_future_type", "2026-07-10T23:59:59.999");
        insert_dated_log(&conn, "unknown_future_type", "2026-07-11T00:00:00");

        let from = list_operation_logs(&conn, 1, 20, None, Some("2026-07-10"), None).unwrap();
        assert_eq!(from.total_count, 3);
        let through = list_operation_logs(&conn, 1, 20, None, None, Some("2026-07-10")).unwrap();
        assert_eq!(through.total_count, 3);
        assert!(through
            .items
            .iter()
            .all(|item| item.created_at.as_str() < "2026-07-11T00:00:00"));
    }

    #[test]
    fn test_find_distinct_operation_types_req902_dedup_order_unknown_and_empty() {
        // REQ-902 / UI-11c-D4
        let (_dir, conn) = setup_test_db();
        assert_eq!(
            find_distinct_operation_types(&conn).unwrap(),
            Vec::<String>::new()
        );
        insert_dated_log(&conn, "z_unknown", "2026-07-10T00:00:00");
        insert_dated_log(&conn, "backup_create", "2026-07-10T01:00:00");
        insert_dated_log(&conn, "z_unknown", "2026-07-10T02:00:00");
        assert_eq!(
            find_distinct_operation_types(&conn).unwrap(),
            vec!["backup_create".to_string(), "z_unknown".to_string()]
        );
    }

    #[test]
    fn test_list_operation_logs_req902_empty() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: list_operation_logs — ログなし → 空結果、total_count=0
        let (_dir, conn) = setup_test_db();

        let result = list_operation_logs(&conn, 1, 10, None, None, None).unwrap();
        assert_eq!(result.items.len(), 0);
        assert_eq!(result.total_count, 0);
        assert_eq!(result.page, 1);
        assert_eq!(result.per_page, 10);
    }

    #[test]
    fn test_list_operation_logs_req902_pagination() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: list_operation_logs — 複数ページのページング動作
        let (_dir, conn) = setup_test_db();
        insert_test_logs(&conn, 5, "product_create");

        // ページ1: 2件ずつ
        let page1 = list_operation_logs(&conn, 1, 2, None, None, None).unwrap();
        assert_eq!(page1.items.len(), 2);
        assert_eq!(page1.total_count, 5);
        assert_eq!(page1.page, 1);

        // ページ2
        let page2 = list_operation_logs(&conn, 2, 2, None, None, None).unwrap();
        assert_eq!(page2.items.len(), 2);

        // ページ3: 残り1件
        let page3 = list_operation_logs(&conn, 3, 2, None, None, None).unwrap();
        assert_eq!(page3.items.len(), 1);
    }

    #[test]
    fn test_list_operation_logs_req902_max_page_returns_empty_without_overflow() {
        // REQ-902 / UI-11c-D8: 巨大なpositive pageでもoffset計算をoverflowさせない
        let (_dir, conn) = setup_test_db();
        insert_test_logs(&conn, 1, "product_create");

        let result = list_operation_logs(&conn, u32::MAX, 20, None, None, None).unwrap();

        assert!(result.items.is_empty());
        assert_eq!(result.total_count, 1);
        assert_eq!(result.page, u32::MAX);
        assert_eq!(result.per_page, 20);
    }

    #[test]
    fn test_list_operation_logs_req902_filter_type() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: list_operation_logs — operation_typeフィルタ
        let (_dir, conn) = setup_test_db();
        insert_test_logs(&conn, 3, "product_create");
        insert_test_logs(&conn, 2, "csv_import");

        let result = list_operation_logs(&conn, 1, 10, Some("csv_import"), None, None).unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.total_count, 2);
        for item in &result.items {
            assert_eq!(item.operation_type, "csv_import");
        }
    }

    #[test]
    fn test_list_operation_logs_req902_order_desc() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: list_operation_logs — created_at DESC, id DESC の順序
        let (_dir, conn) = setup_test_db();
        insert_test_logs(&conn, 3, "product_create");

        let result = list_operation_logs(&conn, 1, 10, None, None, None).unwrap();
        assert_eq!(result.items.len(), 3);

        // id が降順であること（同一 created_at 秒の場合も id で決定的）
        for i in 0..result.items.len() - 1 {
            assert!(
                result.items[i].id >= result.items[i + 1].id,
                "id が降順であるべき: {} >= {}",
                result.items[i].id,
                result.items[i + 1].id
            );
        }
    }

    #[test]
    fn test_list_operation_logs_req902_per_page_clamp() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: list_operation_logs — per_page > 200 → 200にクランプ
        let (_dir, conn) = setup_test_db();
        insert_test_logs(&conn, 3, "product_create");

        let result = list_operation_logs(&conn, 1, 999, None, None, None).unwrap();
        assert_eq!(result.per_page, 200, "per_page は200にクランプされるべき");
        assert_eq!(result.items.len(), 3, "データは全件取得される");
    }

    // -----------------------------------------------------------------------
    // delete_old_logs（第6段階追加）
    // -----------------------------------------------------------------------

    #[test]
    fn test_delete_old_logs_req902_removes_old() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: delete_old_logs — 古いログが削除される
        let (_dir, conn) = setup_test_db();

        // 400日前のログを手動INSERT
        conn.execute(
            "INSERT INTO operation_logs (operation_type, summary, created_at)
             VALUES ('old_op', '古いログ', '2025-01-01T10:00:00')",
            [],
        )
        .unwrap();
        // 今日のログ
        insert_test_logs(&conn, 1, "recent_op");

        let deleted = delete_old_logs(&conn, 365).unwrap();
        assert_eq!(deleted, 1, "365日より古い1件が削除されるべき");

        // 今日のログは残っている
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "最近のログは残っている");
    }

    #[test]
    fn test_delete_old_logs_req902_preserves_recent() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: delete_old_logs — 保持期間内のログは残る
        let (_dir, conn) = setup_test_db();

        // 今日のログを3件
        insert_test_logs(&conn, 3, "product_create");

        let deleted = delete_old_logs(&conn, 365).unwrap();
        assert_eq!(deleted, 0, "保持期間内のログは削除されない");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3, "3件すべて残っている");
    }

    #[test]
    fn test_delete_old_logs_req902_out_of_range_returns_zero() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // FUNC-2.8: delete_old_logs — 日付の範囲の外になる保持日数（u32::MAX）では
        // panic せず何も消さず Ok(0)（MNT-02-D1）
        let (_dir, conn) = setup_test_db();

        let old = (chrono::Local::now().date_naive() - chrono::Days::new(3001))
            .format("%Y-%m-%dT10:00:00")
            .to_string();
        insert_dated_log(&conn, "old_op", &old);

        let deleted = delete_old_logs(&conn, u32::MAX).unwrap();
        assert_eq!(deleted, 0, "範囲の外の保持日数では何も消さない");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "3001日前のログは残っている");
    }
}
