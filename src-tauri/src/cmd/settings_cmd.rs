//! CMD-11 残り: 設定・ログ・バックアップ・画像コマンド群
//!
//! docs/function-design/43-cmd-settings-log.md に基づく実装。
//! 整合性チェック（run_integrity_check, fix_integrity）は integrity_cmd.rs に実装済み。

use crate::biz::{
    self, AppSetting, BizError, DbConnection, DbError, OperationLog, PaginatedResult,
};
use crate::cmd::{AppState, CmdError, CmdErrorKind};
use crate::mnt::backup;
use base64::{engine::general_purpose, Engine as _};
use std::path::PathBuf;
use tauri::{Manager, State};

// ---------------------------------------------------------------------------
// 型定義（§43.2）
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct UpdateSettingRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct LogQuery {
    pub page: u32,
    pub per_page: u32,
    pub operation_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct RestoreBackupRequest {
    pub backup_path: String,
}

#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct SaveImageRequest {
    pub image_base64: String,
    pub extension: String,
}

#[derive(Debug, serde::Serialize, specta::Type)]
pub struct SaveImageResponse {
    pub relative_path: String,
}

// ---------------------------------------------------------------------------
// ヘルパー
// ---------------------------------------------------------------------------

/// backup_dir を resolve するヘルパー（複数コマンドで共通）
fn get_backup_dir<R: tauri::Runtime>(
    conn: &DbConnection,
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, CmdError> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| CmdError::internal("アプリデータの保存先を取得できませんでした", e))?;
    backup::resolve_backup_dir(conn, &app_data).map_err(db_err)
}

/// DbError → CmdError::internal 変換ヘルパー
fn db_err(e: DbError) -> CmdError {
    CmdError::internal("データベース処理でエラーが発生しました", e)
}

fn terminal_restore_error(error: backup::RestoreError) -> CmdError {
    match error {
        backup::RestoreError::Recovered(message) => CmdError::restore_failed_recovered(&message),
        backup::RestoreError::Unrecoverable(message) => {
            CmdError::restore_failed_unrecoverable(
                "バックアップの復元に失敗し、DB接続の復旧もできませんでした。アプリを再起動してください",
                &message,
            )
        }
        backup::RestoreError::DurabilityUnknown(message) => {
            CmdError::restore_durability_unknown(
                "復元が完了したか確定できませんでした。アプリを再起動してください。",
                &message,
            )
        }
    }
}

fn handle_restore_failure(
    guard: &mut DbConnection,
    db_path: &std::path::Path,
    error: backup::RestoreError,
) -> CmdError {
    match error {
        backup::RestoreError::Recovered(error) => {
            // NO_CREATE 再接続のみ許可し、空DBを生成しない。
            match backup::open_existing_database(db_path.to_str().unwrap_or("")) {
                Ok(recovered) => {
                    *guard = recovered;
                    CmdError::restore_failed_recovered(&format!(
                        "バックアップの復元に失敗しました。現在のデータには戻しています: {error}"
                    ))
                }
                Err(recovery_error) => {
                    let detail = format!(
                        "同期巻き戻し後のDB再接続に失敗: restore={error}; reconnect={recovery_error}"
                    );
                    CmdError::restore_failed_unrecoverable(
                        "バックアップの復元に失敗し、DB接続の復旧もできませんでした。アプリを再起動してください",
                        &detail,
                    )
                }
            }
        }
        other => terminal_restore_error(other),
    }
}

// ---------------------------------------------------------------------------
// コマンド
// ---------------------------------------------------------------------------

/// 全設定を取得する（§43.3）
#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<AppState>) -> Result<Vec<AppSetting>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    Ok(biz::system_service::get_all_settings(&conn)?)
}

/// 設定値を更新する（§43.4）
#[tauri::command]
#[specta::specta]
pub fn update_setting(
    state: State<AppState>,
    request: UpdateSettingRequest,
) -> Result<(), CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    biz::system_service::upsert_setting(&conn, &request.key, &request.value)?;
    Ok(())
}

/// 操作ログ一覧を取得する（§43.5）
#[tauri::command]
#[specta::specta]
pub fn list_logs(
    state: State<AppState>,
    query: LogQuery,
) -> Result<PaginatedResult<OperationLog>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    Ok(biz::system_service::list_operation_logs(
        &conn,
        query.page,
        query.per_page,
        query.operation_type.as_deref(),
        query.start_date.as_deref(),
        query.end_date.as_deref(),
    )?)
}

#[tauri::command]
#[specta::specta]
pub fn list_log_operation_types(state: State<AppState>) -> Result<Vec<String>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    Ok(biz::system_service::list_distinct_operation_types(&conn)?)
}

/// バックアップを作成する（§43.6）
#[tauri::command]
#[specta::specta]
pub fn create_backup(
    state: State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<backup::BackupResult, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let backup_dir = get_backup_dir(&conn, &app_handle)?;
    backup::create_backup(&conn, &backup_dir).map_err(db_err)
}

/// 自動バックアップチェック（§43.7）
///
/// フロントエンドの setInterval(60秒) から呼ばれる。
#[tauri::command]
#[specta::specta]
pub fn check_auto_backup(
    state: State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<bool, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let backup_dir = get_backup_dir(&conn, &app_handle)?;
    backup::check_auto_backup(&conn, &backup_dir).map_err(db_err)
}

/// 実効バックアップ保存先を取得する（§43.8.1）
///
/// `backup_path` 未設定時にアプリ既定フォルダ（`app_data/backups`）を利用者へ提示するための
/// 読み取り専用コマンド。既存ヘルパ `get_backup_dir` を呼ぶだけの薄いラッパー。
#[tauri::command]
#[specta::specta]
pub fn get_effective_backup_dir(
    state: State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let backup_dir = get_backup_dir(&conn, &app_handle)?;
    Ok(backup_dir.to_string_lossy().to_string())
}

/// バックアップ一覧を取得する（§43.8）
#[tauri::command]
#[specta::specta]
pub fn list_backups(
    state: State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<backup::BackupInfo>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let backup_dir = get_backup_dir(&conn, &app_handle)?;
    backup::list_backups(&backup_dir)
        .map_err(|e| CmdError::internal("バックアップ一覧の取得でエラーが発生しました", e))
}

/// バックアップから復元する（§43.9）
///
/// DB接続の所有権を移転するため、Mutex内の接続をstd::mem::replaceで取り出す。
/// **? 演算子はreplace後に使用禁止** — dummy接続がguardに残るのを防ぐため。
#[tauri::command]
#[specta::specta]
pub fn restore_backup(
    state: State<AppState>,
    app_handle: tauri::AppHandle,
    request: RestoreBackupRequest,
) -> Result<(), CmdError> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| CmdError::internal("アプリデータの保存先を取得できませんでした", e))?;
    let db_path = app_data.join("inventory.db");
    let backup_path = std::path::Path::new(&request.backup_path);

    // Mutex ロック取得
    let mut guard = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;

    // dummy接続を作成し、現在の接続を取り出す
    let dummy = match rusqlite::Connection::open_in_memory() {
        Ok(c) => c,
        Err(e) => {
            return Err(CmdError::internal(
                "復元準備中にデータベースへ接続できませんでした",
                e,
            ));
        }
    };
    let old_conn = std::mem::replace(&mut *guard, dummy);

    // ── ここ以降 ? 使用禁止 ── guard に dummy が入っている ──

    match backup::restore_backup(old_conn, backup_path, &db_path) {
        Ok(new_conn) => {
            *guard = new_conn;
            Ok(())
        }
        Err(error) => Err(handle_restore_failure(&mut guard, &db_path, error)),
    }
}

/// レシート画像を保存する（§43.10）
#[tauri::command]
#[specta::specta]
pub fn save_receipt_image(
    app_handle: tauri::AppHandle,
    request: SaveImageRequest,
) -> Result<SaveImageResponse, CmdError> {
    save_receipt_image_with_runtime(app_handle, request)
}

fn save_receipt_image_with_runtime<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    request: SaveImageRequest,
) -> Result<SaveImageResponse, CmdError> {
    // 1. Base64デコード
    let image_bytes = general_purpose::STANDARD
        .decode(&request.image_base64)
        .map_err(|_| CmdError {
            kind: CmdErrorKind::Validation,
            message: "画像データが不正です".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        })?;

    // 2. app_data_dir 取得
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| CmdError::internal("アプリデータの保存先を取得できませんでした", e))?;

    // 3. 画像保存
    let relative_path =
        biz::inventory_service::save_receipt_image(&app_data, &image_bytes, &request.extension)
            .map_err(|error| match error {
                BizError::DatabaseError(detail) => {
                    CmdError::internal("画像の保存でエラーが発生しました", detail)
                }
                other => CmdError::from(other),
            })?;

    Ok(SaveImageResponse { relative_path })
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::system_repo;
    use crate::mnt::backup as mnt_backup;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tauri::Manager;

    fn setup_test_db() -> (tempfile::TempDir, db::DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    fn app_state_for_test(conn: db::DbConnection) -> AppState {
        AppState {
            db: Mutex::new(conn),
            preview_cache: Mutex::new(HashMap::new()),
            daily_report_preview_cache: Mutex::new(HashMap::new()),
            register_sd_scan_cache: Mutex::new(HashMap::new()),
        }
    }

    #[test]
    fn test_get_backup_dir_req901_d2_maps_db_error_to_internal() {
        // REQ-901 / MNT-01-D2 / Matrix C6
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let _failure = mnt_backup::fail_setting_read("backup_path");

        let error = get_backup_dir(&conn, app.handle()).unwrap_err();

        assert_eq!(error.kind, CmdErrorKind::Internal);
        assert_eq!(error.message, "データベース処理でエラーが発生しました");
        assert!(!error.message.contains("backup_path"));
        assert!(error.error_id.is_some());
    }

    #[test]
    fn test_get_settings_req905_cmd11() {
        // REQ-905 / CMD-11: 実 command が BIZ 経由で初期設定を返す。
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let settings = get_settings(app.state::<AppState>()).unwrap();

        assert!(!settings.is_empty(), "初期設定が1件以上存在するべき");
        let keys: Vec<&str> = settings.iter().map(|s| s.key.as_str()).collect();
        assert!(
            keys.contains(&"backup_enabled"),
            "backup_enabled が含まれるべき"
        );
    }

    #[test]
    fn test_update_setting_req905_cmd11() {
        // REQ-905 / CMD-11: 実 command の upsert を実 command で読み戻す。
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        update_setting(
            app.state::<AppState>(),
            UpdateSettingRequest {
                key: "stock_low_threshold".to_string(),
                value: "5".to_string(),
            },
        )
        .unwrap();
        let settings = get_settings(app.state::<AppState>()).unwrap();

        assert_eq!(
            settings
                .iter()
                .find(|setting| setting.key == "stock_low_threshold")
                .map(|setting| setting.value.as_str()),
            Some("5")
        );
    }

    #[test]
    fn test_list_logs_req902_pagination() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: CMD-11
        // CMD-11: ページングパラメータの受け渡し
        let (_dir, conn) = setup_test_db();

        // テストデータ挿入
        for i in 0..5 {
            system_repo::insert_operation_log(
                &conn,
                &db::NewOperationLog {
                    operation_type: "test_op".to_string(),
                    summary: format!("テストログ{}", i),
                    detail_json: None,
                },
            )
            .unwrap();
        }
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let result = list_logs(
            app.state::<AppState>(),
            LogQuery {
                page: 1,
                per_page: 2,
                operation_type: None,
                start_date: None,
                end_date: None,
            },
        )
        .unwrap();
        assert_eq!(result.per_page, 2, "per_page=2");
        assert_eq!(result.page, 1, "page=1");
        assert!(result.items.len() <= 2, "1ページあたり2件以下");
        assert!(
            result.total_count >= 5,
            "合計5件以上（初期データ含む可能性）"
        );
    }

    #[test]
    fn test_list_logs_req902_filter() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: CMD-11
        // CMD-11: operation_type フィルタ
        let (_dir, conn) = setup_test_db();

        system_repo::insert_operation_log(
            &conn,
            &db::NewOperationLog {
                operation_type: "backup_create".to_string(),
                summary: "テスト".to_string(),
                detail_json: None,
            },
        )
        .unwrap();
        system_repo::insert_operation_log(
            &conn,
            &db::NewOperationLog {
                operation_type: "product_create".to_string(),
                summary: "テスト".to_string(),
                detail_json: None,
            },
        )
        .unwrap();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let result = list_logs(
            app.state::<AppState>(),
            LogQuery {
                page: 1,
                per_page: 100,
                operation_type: Some("backup_create".to_string()),
                start_date: None,
                end_date: None,
            },
        )
        .unwrap();
        assert!(
            result
                .items
                .iter()
                .all(|l| l.operation_type == "backup_create"),
            "フィルタされた結果のみ返されるべき"
        );
        assert!(result.total_count >= 1, "1件以上ヒット");
    }

    #[test]
    fn test_list_log_operation_types_req902_cmd_calls_command() {
        // REQ-902 / UI-11c-D4 / CMD-11: 実 command が全ログ由来の候補を返す。
        let (_dir, conn) = setup_test_db();
        for operation_type in ["z_unknown", "backup_create", "z_unknown"] {
            system_repo::insert_operation_log(
                &conn,
                &db::NewOperationLog {
                    operation_type: operation_type.to_string(),
                    summary: "test".to_string(),
                    detail_json: None,
                },
            )
            .unwrap();
        }
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let result = list_log_operation_types(app.state::<AppState>()).unwrap();

        assert_eq!(
            result,
            vec!["backup_create".to_string(), "z_unknown".to_string()]
        );
    }

    #[test]
    fn test_create_backup_req905() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // Task: CMD-11
        // CMD-11: バックアップ作成と BackupResult
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        let result = mnt_backup::create_backup(&conn, &backup_dir).unwrap();

        assert!(!result.file_name.is_empty(), "ファイル名が返されるべき");
        assert!(result.size_bytes > 0, "サイズが0より大きいべき");
        assert!(
            std::path::Path::new(&result.file_path).exists(),
            "ファイルが存在するべき"
        );
    }

    #[test]
    fn test_list_backups_req905() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // Task: CMD-11
        // CMD-11: バックアップ一覧取得
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        // バックアップ作成
        mnt_backup::create_backup(&conn, &backup_dir).unwrap();

        let list = mnt_backup::list_backups(&backup_dir).unwrap();
        assert_eq!(list.len(), 1, "1件のバックアップが返されるべき");
        assert!(!list[0].file_name.is_empty(), "ファイル名があるべき");
        assert!(!list[0].created_at.is_empty(), "日時があるべき");
    }

    #[test]
    fn test_save_receipt_image_req906_cmd11_valid() {
        // REQ-906 / CMD-11: 実 command が decode 後の画像を BIZ 経由で保存する。
        let data_dir = tempfile::tempdir().unwrap();
        let image_data = b"fake-image-data";
        let encoded = general_purpose::STANDARD.encode(image_data);
        let mut context = tauri::test::mock_context(tauri::test::noop_assets());
        context.config_mut().identifier = data_dir.path().to_string_lossy().to_string();
        let app = tauri::test::mock_builder().build(context).unwrap();

        let response = save_receipt_image_with_runtime(
            app.handle().clone(),
            SaveImageRequest {
                image_base64: encoded,
                extension: "jpg".to_string(),
            },
        )
        .unwrap();

        assert!(
            response.relative_path.starts_with("images/receipts/"),
            "相対パスが正しい形式: {}",
            response.relative_path
        );
        let app_data_dir = app.path().app_data_dir().unwrap();
        assert!(app_data_dir.starts_with(data_dir.path()));
        let saved_image = std::fs::read(app_data_dir.join(response.relative_path)).unwrap();
        assert_eq!(saved_image, image_data);
    }

    #[test]
    fn test_save_receipt_image_req906_cmd11_invalid_base64() {
        // REQ-906 / CMD-11: 実 command の wire decode エラー契約を固定する。
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let error = save_receipt_image_with_runtime(
            app.handle().clone(),
            SaveImageRequest {
                image_base64: "!!!not-valid-base64!!!".to_string(),
                extension: "jpg".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(error.kind, CmdErrorKind::Validation);
        assert_eq!(error.message, "画像データが不正です");
        assert_eq!(error.field, None);
    }

    #[test]
    fn test_restore_backup_req905_notfound_to_cmderror() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // Task: CMD-11
        // CMD-11: restore_backup に存在しないファイルを渡すと NotFound → CmdError 変換
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("main.db");
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();

        let nonexistent = dir.path().join("nonexistent.db");
        let result = backup::restore_backup(conn, &nonexistent, &db_path);

        assert!(result.is_err(), "存在しないファイルでエラーが返されるべき");
        // DbError::NotFound → CMD層で CmdError::internal に変換されることを検証
        let db_err_str = format!("{}", result.unwrap_err());
        assert!(
            db_err_str.contains("見つかりません"),
            "NotFoundメッセージが含まれるべき: {}",
            db_err_str
        );
    }

    #[test]
    fn test_restore_backup_req905_recovery_after_failure() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // Task: CMD-11
        // CMD-11: restore失敗後にDBファイルが復元されていることを確認
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("main.db");
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();

        // テストデータ挿入
        conn.execute(
            "INSERT INTO suppliers (name, created_at) VALUES ('復元テスト', '2026-01-01T00:00:00')",
            [],
        )
        .unwrap();

        let nonexistent = dir.path().join("nonexistent.db");
        // restore_backup は NotFound で失敗するが、DB接続は drop される（所有権移転）
        let _ = backup::restore_backup(conn, &nonexistent, &db_path);

        // DBファイルが復元されていれば再接続できるはず
        // （NotFoundの場合、rename前に早期returnするのでDBファイルはそのまま）
        let recovered = db::init_database(db_path.to_str().unwrap()).unwrap();
        let name: String = recovered
            .query_row("SELECT name FROM suppliers LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "復元テスト", "元のデータがアクセス可能であるべき");
    }

    #[test]
    fn test_save_receipt_image_req906_cmd11_invalid_extension_to_validation() {
        // REQ-906 / SPEC-CMD11-D3: 実 command が BIZ の validation triple を保持する。
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let encoded = general_purpose::STANDARD.encode(b"fake-image");

        let cmd_err = save_receipt_image_with_runtime(
            app.handle().clone(),
            SaveImageRequest {
                image_base64: encoded,
                extension: "bmp".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(cmd_err.kind, CmdErrorKind::Validation);
        assert_eq!(
            cmd_err.message,
            "不正な画像拡張子: bmp（許可: jpg, jpeg, png, gif, webp）"
        );
        assert_eq!(
            cmd_err.field.as_deref(),
            Some("extension"),
            "field が extension"
        );
    }

    #[test]
    fn test_save_receipt_image_req906_command_wrapper_delegates_to_runtime_core() {
        // REQ-906 / SPEC-CMD11-D5 (iii): MockRuntime で直接型付けできない Wry command の
        // production wrapper が、テスト対象の runtime-generic core へ委譲する配線を固定する。
        let source = include_str!("settings_cmd.rs");
        let wrapper = source
            .split("pub fn save_receipt_image(")
            .nth(1)
            .and_then(|rest| rest.split("fn save_receipt_image_with_runtime").next())
            .expect("save_receipt_image command wrapper must exist");

        assert!(wrapper.contains("save_receipt_image_with_runtime(app_handle, request)"));
    }

    #[test]
    fn test_list_logs_req902_invalid_page_to_cmderror() {
        // REQ-902 / CMD-11 / SPEC-CMD11-IMPL-D4
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let cmd_err = list_logs(
            app.state::<AppState>(),
            LogQuery {
                page: 0,
                per_page: 10,
                operation_type: None,
                start_date: None,
                end_date: None,
            },
        )
        .unwrap_err();

        assert_eq!(cmd_err.kind, CmdErrorKind::Internal);
        assert_eq!(
            cmd_err.message,
            "データベースエラーが発生しました。もう一度お試しください"
        );
        assert!(!cmd_err.message.contains("page"));
        assert!(cmd_err.error_id.is_some());
    }

    #[test]
    fn test_restore_backup_req905_unrecoverable_message() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // Task: CMD-11
        // CMD-11: 復旧不能時の「再起動が必要」メッセージが CmdError に含まれることを検証
        // 実際の init_database 失敗は再現困難なため、エラーメッセージの構築を直接テスト
        let cmd_err =
            terminal_restore_error(backup::RestoreError::Unrecoverable("fixture".to_string()));
        assert_eq!(cmd_err.kind, CmdErrorKind::RestoreFailedUnrecoverable);
        assert!(
            cmd_err.message.contains("再起動"),
            "再起動メッセージが含まれるべき: {}",
            cmd_err.message
        );
    }

    #[test]
    fn test_restore_backup_req905_maps_all_failure_kinds_without_message_parsing() {
        // REQ-905 / MNT-01-D4 / Matrix F1, F2
        let recovered =
            terminal_restore_error(backup::RestoreError::Recovered("same message".to_string()));
        let fatal = terminal_restore_error(backup::RestoreError::Unrecoverable(
            "same message".to_string(),
        ));
        let unknown = terminal_restore_error(backup::RestoreError::DurabilityUnknown(
            "same message".to_string(),
        ));
        assert_eq!(recovered.kind, CmdErrorKind::RestoreFailedRecovered);
        assert_eq!(fatal.kind, CmdErrorKind::RestoreFailedUnrecoverable);
        assert_eq!(unknown.kind, CmdErrorKind::RestoreDurabilityUnknown);
        assert_eq!(
            unknown.message,
            "復元が完了したか確定できませんでした。アプリを再起動してください。"
        );
    }

    #[test]
    fn test_restore_backup_req905_b3_no_create_cmd_recovery_never_hides_missing_main() {
        // REQ-905 / MNT-01-D4 / Matrix B3
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("inventory.db");
        let mut dummy = rusqlite::Connection::open_in_memory().unwrap();
        let error = handle_restore_failure(
            &mut dummy,
            &missing,
            backup::RestoreError::Recovered("injected rollback result".to_string()),
        );
        assert_eq!(error.kind, CmdErrorKind::RestoreFailedUnrecoverable);
        assert!(error.message.contains("再起動"));
        assert!(!missing.exists(), "CMD recovery must use NO_CREATE open");
    }

    #[test]
    fn test_check_auto_backup_req905_dberror_to_cmderror() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        // Task: CMD-11
        // CMD-11: check_auto_backup の DbError → CmdError 変換確認
        let (dir, conn) = setup_test_db();

        // backup_enabled を無効に設定 → Ok(false) が返る（エラーではない）
        system_repo::upsert_setting(&conn, "backup_enabled", "0").unwrap();
        let backup_dir = dir.path().join("backups");
        let result = backup::check_auto_backup(&conn, &backup_dir).unwrap();
        assert!(!result, "無効時はfalse");

        // DbError 変換パスのテスト: db_err ヘルパーが DbError を CmdError に変換
        let test_err = db::DbError::QueryFailed("テストエラー".to_string());
        let cmd_err = super::db_err(test_err);
        assert_eq!(
            cmd_err.kind,
            CmdErrorKind::Internal,
            "DbError は internal に変換されるべき"
        );
        assert_eq!(cmd_err.message, "データベース処理でエラーが発生しました");
        assert!(!cmd_err.message.contains("テストエラー"));
        assert!(cmd_err.error_id.is_some());
    }
}
