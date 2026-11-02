//! CMD-07: CSV取込みコマンド群
//!
//! docs/function-design/41-cmd-pos.md §17.5 に基づく実装。
//! CMD層は薄いラッパー。キャッシュ管理とBizError→CmdError変換のみ。

use crate::biz::csv_import_service::{
    self, CachedPreview, CommitRequest, CsvParseAndValidateRequest, PreviewData,
};
use crate::cmd::{AppState, CmdError, CmdErrorKind};
use crate::constants;
use std::time::Instant;
use tauri::State;

// ---------------------------------------------------------------------------
// レスポンス型
// ---------------------------------------------------------------------------

/// parse_and_validate_csv のレスポンス（フロントエンド返却用）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ParseAndValidateResponse {
    /// プレビューデータ（表示用）
    pub preview_data: PreviewData,
    /// commit時に送り返すトークン
    pub preview_token: String,
}

/// PC の今日（ローカルの日付）。BIZ は時計を読まず、CMD がここから渡す（41）
fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

// ---------------------------------------------------------------------------
// コマンド
// ---------------------------------------------------------------------------

/// Z004ファイルを受け取り、プレビューデータを返す
///
/// docs/function-design/41-cmd-pos.md §17.5 parse_and_validate_csv
#[tauri::command]
#[specta::specta]
pub fn parse_and_validate_csv(
    state: State<AppState>,
    file_bytes: Vec<u8>,
    filename: String,
) -> Result<ParseAndValidateResponse, CmdError> {
    // 1. サイズチェック（防御的。BIZ層にも同じチェックあり）
    if file_bytes.len() > constants::CSV_IMPORT_FILE_SIZE_LIMIT {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "ファイルサイズが上限(20MB)を超えています".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }

    // 2. DB接続取得 → BIZ呼び出し → DB解放
    let result = {
        let conn = state
            .db
            .lock()
            .map_err(|error| CmdError::internal("DB接続エラー", error))?;
        let req = CsvParseAndValidateRequest {
            file_bytes,
            filename,
            today: today(),
        };
        csv_import_service::parse_and_validate(&conn, req).map_err(CmdError::from)?
    }; // db lock解放

    // 3. キャッシュ保存（db lock解放後にcache lock — デッドロック防止）
    let preview_token = result.preview_token.clone();
    let response = ParseAndValidateResponse {
        preview_data: result.preview_data.clone(),
        preview_token: preview_token.clone(),
    };

    {
        let mut cache = state
            .preview_cache
            .lock()
            .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;

        // FIFO eviction: 上限超過時は最古を削除
        if cache.len() >= constants::PREVIEW_CACHE_LIMIT {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.created_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            preview_token,
            CachedPreview {
                created_at: Instant::now(),
                matched_rows: result.matched_rows,
                error_rows: result.error_rows,
                preview_data: result.preview_data,
                settlement: result.settlement,
            },
        );
    }

    Ok(response)
}

/// プレビュー済みデータの取込みを確定する
///
/// docs/function-design/41-cmd-pos.md §17.5 commit_csv_import
#[tauri::command]
#[specta::specta]
pub fn commit_csv_import(
    state: State<AppState>,
    preview_token: String,
) -> Result<csv_import_service::ImportResult, CmdError> {
    // 1. UUID形式バリデーション
    if uuid::Uuid::parse_str(&preview_token).is_err() {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "不正なプレビュートークンです".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }

    // 2. キャッシュからデータ取得（clone で保持。成功時のみ remove）
    let cached_data = {
        let cache = state
            .preview_cache
            .lock()
            .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;

        match cache.get(&preview_token) {
            None => {
                return Err(CmdError {
                    kind: CmdErrorKind::ImportError,
                    message: "プレビューが見つかりません。再度ファイルを選択してください"
                        .to_string(),
                    field: None,
                    error_id: None,
                    stocktake_recovery: None,
                    related_csv_import_id: None,
                });
            }
            Some(cached) => {
                // TTL検証（30分）
                if cached.created_at.elapsed().as_secs() > constants::PREVIEW_CACHE_TTL_SECS {
                    drop(cache);
                    // 期限切れ → remove
                    let mut cache = state
                        .preview_cache
                        .lock()
                        .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
                    cache.remove(&preview_token);
                    return Err(CmdError {
                        kind: CmdErrorKind::ImportError,
                        message:
                            "プレビューの有効期限が切れました（30分）。再度ファイルを選択してください"
                                .to_string(),
                        field: None,
                        error_id: None,
                        stocktake_recovery: None,
                        related_csv_import_id: None,
                    });
                }
                cached.clone()
            }
        }
    }; // cache lock解放

    // 3. DB接続取得 + BIZ呼び出し（cache lock なし — デッドロック防止）
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;

    let req = CommitRequest {
        cached_data,
        today: today(),
    };

    // token は確定の成功のときだけ消す（拒否・失敗では残す。41）
    let result = csv_import_service::commit_csv_import(&mut conn, req).map_err(CmdError::from)?;
    drop(conn); // db lock解放
    let mut cache = state
        .preview_cache
        .lock()
        .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
    cache.remove(&preview_token);
    Ok(result)
}

/// CSV取込みをロールバック（論理無効化）する
///
/// docs/function-design/41-cmd-pos.md §17.5 rollback_csv_import
#[tauri::command]
#[specta::specta]
pub fn rollback_csv_import(
    state: State<AppState>,
    csv_import_id: i64,
) -> Result<csv_import_service::RollbackResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    csv_import_service::rollback_csv_import(&mut conn, csv_import_id).map_err(CmdError::from)
}

/// 取消の前の読取り（取り消した後の要再確認・取り込み直す精算・名指す完了した棚卸し。何も書かない）
///
/// docs/function-design/41-cmd-pos.md §17.5 preview_rollback_csv_import
#[tauri::command]
#[specta::specta]
pub fn preview_rollback_csv_import(
    state: State<AppState>,
    csv_import_id: i64,
) -> Result<csv_import_service::RollbackPreview, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    csv_import_service::preview_rollback_csv_import(&mut conn, csv_import_id)
        .map_err(CmdError::from)
}

/// CSV取込み履歴の一覧を返す
///
/// docs/function-design/41-cmd-pos.md §17.5 list_csv_imports
#[tauri::command]
#[specta::specta]
pub fn list_csv_imports(
    state: State<AppState>,
    page: u32,
    per_page: u32,
) -> Result<crate::biz::PaginatedResult<crate::biz::CsvImport>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    csv_import_service::list_csv_imports(&conn, page, per_page).map_err(CmdError::from)
}

/// CSV取込み記録の詳細を返す。
///
/// docs/function-design/41-cmd-pos.md §17.5 get_csv_import_record
#[tauri::command]
#[specta::specta]
pub fn get_csv_import_record(
    state: State<AppState>,
    import_id: i64,
) -> Result<csv_import_service::CsvImportRecordDetail, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    csv_import_service::get_csv_import_record(&conn, import_id).map_err(CmdError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::setup_test_db;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tauri::Manager;

    /// layout A の Z004（レジ 0001、精算 1、2026-03-21、商品A × 1）
    fn cmd_z004_bytes() -> Vec<u8> {
        let text = "\"マシンNo.   \",\"0001\"\r\n\"ファイル    \",\"Z004\"\r\n\"精算回数    \",\"0001\"\r\n\
                    \"日付        \",\"2026-03-21\"\r\n\"時刻        \",\"20:00\"\r\n\r\n\
                    \"レコード    \",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ \",\"キャラクター\",\"個数        \",\"金額        \"\r\n\
                    \"1\",\"4912345678901\",\"商品A\",\"1\",\"300\"";
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(text);
        encoded.into_owned()
    }

    fn app_with(conn: crate::db::DbConnection) -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .manage(AppState {
                db: Mutex::new(conn),
                preview_cache: Mutex::new(HashMap::new()),
                daily_report_preview_cache: Mutex::new(HashMap::new()),
                register_sd_scan_cache: Mutex::new(HashMap::new()),
            })
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
    }

    fn insert_import(conn: &crate::db::DbConnection, status: &str, hash: &str) -> i64 {
        crate::db::sales_repo::insert_csv_import(
            conn,
            &crate::db::sales_repo::NewCsvImport {
                filename: format!("Z004_{status}"),
                settlement_date: "2026-03-21".into(),
                file_hash: hash.into(),
                total_items: 1,
                total_amount: 100,
                skipped_count: 0,
                status: status.into(),
                machine_no: "0001".into(),
                settlement_no: 1,
            },
        )
        .unwrap()
    }

    #[test]
    fn test_get_csv_import_record_cmd_req206_not_found_kind() {
        // REQ-206 / 41 §17.5: production command 実呼びで not_found wire kind を固定する。
        let (_dir, conn) = setup_test_db();
        let app = app_with(conn);

        let error = get_csv_import_record(app.state::<AppState>(), 99_999).unwrap_err();

        assert_eq!(error.kind, CmdErrorKind::NotFound);
        assert_eq!(error.message, "CSV取込み記録が見つかりません: 99999");
    }

    #[test]
    fn test_csv_cmd_req401_commit_keeps_token_on_rejection_and_removes_on_success() {
        // REQ-401 / 41（Matrix W5）: commit_csv_import(preview_token)。拒否・失敗では token を残し、
        // 成功のときだけ消す（message の文字列で消す分岐を持たない）
        let (_dir, conn) = setup_test_db();
        conn.execute(
            "INSERT INTO products (product_code, jan_code, name, department_id, selling_price,
             cost_price_centi, tax_rate, stock_quantity, stock_unit, is_discontinued, plu_dirty,
             pos_stock_sync, created_at, updated_at)
             VALUES ('TEST-001', '4912345678901', '商品A', 1, 300, 100, '10', 0, 'pcs', 0, 1, 1,
                     '2026-03-21T08:00:00', '2026-03-21T08:00:00')",
            [],
        )
        .unwrap();
        let app = app_with(conn);
        let token =
            parse_and_validate_csv(app.state::<AppState>(), cmd_z004_bytes(), "Z004.CSV".into())
                .unwrap()
                .preview_token;
        let has_token = |app: &tauri::App<tauri::test::MockRuntime>| {
            app.state::<AppState>()
                .preview_cache
                .lock()
                .unwrap()
                .contains_key(&token)
        };

        // preview の後に同じ精算の別の版が確定した → 順 4 で拒否、token は残る。名指す取込みの ID は
        // 1 でも最大でもない値にする（前後に比べる先にならない取消済みの取込み。Matrix RB7）
        let other = {
            let state = app.state::<AppState>();
            let conn = state.db.lock().unwrap();
            insert_import(&conn, "rolled_back", "before");
            let other = insert_import(&conn, "completed", "other");
            let after = insert_import(&conn, "rolled_back", "after");
            assert!(other != 1 && other < after);
            other
        };
        for _ in 0..2 {
            let error = commit_csv_import(app.state::<AppState>(), token.clone()).unwrap_err();
            assert_eq!(error.kind, CmdErrorKind::ImportError);
            assert_eq!(
                error.message,
                format!(
                    "同じ精算と思われる別の資料が既に取り込まれています（精算 1、取込みID: {other}）"
                )
            );
            assert_eq!(error.related_csv_import_id, Some(other));
            assert!(has_token(&app));
        }

        // 別の版を取り消すと同じ token で確定でき、token が消える
        rollback_csv_import(app.state::<AppState>(), other).unwrap();
        let result = commit_csv_import(app.state::<AppState>(), token.clone()).unwrap();
        assert_eq!(result.total_items, 1);
        assert!(!has_token(&app));
    }

    #[test]
    fn test_preview_rollback_csv_import_cmd_req401_maps_error_kinds() {
        // REQ-401 / 41 §17.5 preview_rollback_csv_import（Matrix W1）: 無い ID は not_found、取消済みは
        // validation、成功は BIZ の精算回数をそのまま返し、何も書かない（取込みは active のまま）
        let (_dir, conn) = setup_test_db();
        let completed = insert_import(&conn, "completed", "completed-hash");
        let rolled_back = insert_import(&conn, "rolled_back", "rolled-back-hash");
        let app = app_with(conn);

        let preview = preview_rollback_csv_import(app.state::<AppState>(), completed).unwrap();
        assert_eq!(
            (preview.csv_import_id, preview.settlement_no),
            (completed, 1)
        );
        let status: String = app
            .state::<AppState>()
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT status FROM csv_imports WHERE id = ?1",
                [completed],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "completed");
        let missing = preview_rollback_csv_import(app.state::<AppState>(), 99_999).unwrap_err();
        assert_eq!(missing.kind, CmdErrorKind::NotFound);
        assert_eq!(missing.message, "CSV取込み記録が見つかりません: ID 99999");
        let done = preview_rollback_csv_import(app.state::<AppState>(), rolled_back).unwrap_err();
        assert_eq!(done.kind, CmdErrorKind::Validation);
        assert_eq!(done.message, "この取込みは取り消し済みです");
    }

    #[test]
    fn test_csv_cmd_req401_rollback_voids_and_is_idempotent() {
        // REQ-401 / 41 §17.5（Matrix T10 の「期待を変える」: 停止の assert を外し、取消の CMD が新しい経路を通る）。
        // 実在は取り消し、rolled_back 済みは冪等の成功、不存在は not_found
        let (_dir, conn) = setup_test_db();
        let completed = insert_import(&conn, "completed", "completed-hash");
        let rolled_back = insert_import(&conn, "rolled_back", "rolled-back-hash");
        let app = app_with(conn);

        let result = rollback_csv_import(app.state::<AppState>(), completed).unwrap();
        assert!(result.success);
        assert!(result.recheck_targets.is_empty());
        let status: String = app
            .state::<AppState>()
            .db
            .lock()
            .unwrap()
            .query_row(
                "SELECT status FROM csv_imports WHERE id = ?1",
                [completed],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "rolled_back");
        let again = rollback_csv_import(app.state::<AppState>(), rolled_back).unwrap();
        assert!(again.success);
        assert_eq!(
            (again.voided_sale_count, again.voided_movement_count),
            (0, 0)
        );
        let missing = rollback_csv_import(app.state::<AppState>(), 99_999).unwrap_err();
        assert_eq!(missing.kind, CmdErrorKind::NotFound);
    }
}
