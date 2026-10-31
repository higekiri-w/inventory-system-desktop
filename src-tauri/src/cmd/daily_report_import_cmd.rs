//! CMD-12: 日報取込みコマンド群
//!
//! CMD層は薄いラッパー。token/cache管理とBizError→CmdError変換のみを持つ。

use crate::biz::daily_report_import_service::{
    self, CachedDailyReportPreview, DailyReportImportResult, DailyReportImportRow,
    DailyReportInputFile, DailyReportPreviewData, DailyReportRollbackResult, DailyReportSdScan,
    DailyReportSdSelection, ListDailyReportImportsQuery,
};
use crate::biz::PaginatedResult;
use crate::cmd::{AppState, CmdError, CmdErrorKind};
use crate::constants;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{Manager, State};

#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct DailyReportSourceFileRequest {
    pub filename: String,
    pub file_bytes: Vec<u8>,
    /// native dialog で選んだ file の path（UI-07-D16、§45.3）。省略と空文字は None
    #[serde(default)]
    pub source_path: Option<String>,
}

/// §45.6a の応答
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct RegisterSdScanResponse {
    pub scan: DailyReportSdScan,
    pub scan_token: String,
}

const SCAN_EXPIRED_MESSAGE: &str =
    "SD を読んでから時間がたちました。もう一度 SD を読んでください。";

#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DailyReportPreviewResponse {
    pub preview_data: DailyReportPreviewData,
    pub preview_token: String,
}

#[tauri::command]
#[specta::specta]
pub fn parse_and_validate_daily_report(
    state: State<AppState>,
    files: Vec<DailyReportSourceFileRequest>,
) -> Result<DailyReportPreviewResponse, CmdError> {
    validate_daily_report_files(&files)?;
    let source_files = files
        .into_iter()
        .map(|file| DailyReportInputFile {
            filename: file.filename,
            bytes: file.file_bytes,
            sd_relative_path: None,
            source_path: file
                .source_path
                .filter(|path| !path.is_empty())
                .map(PathBuf::from),
        })
        .collect();
    preview_and_cache(&state, source_files)
}

/// §45.3 手順 3〜6（§45.6b 手順 4 も同じ）: BIZ-08 の preview を作り、token で cache に置く
fn preview_and_cache(
    state: &AppState,
    files: Vec<DailyReportInputFile>,
) -> Result<DailyReportPreviewResponse, CmdError> {
    let result = {
        let conn = state
            .db
            .lock()
            .map_err(|error| CmdError::internal("DB接続エラー", error))?;
        daily_report_import_service::parse_and_validate_daily_report(&conn, files)
            .map_err(CmdError::from)?
    };

    let preview_token = uuid::Uuid::new_v4().to_string();
    let response = DailyReportPreviewResponse {
        preview_data: result.preview_data,
        preview_token: preview_token.clone(),
    };

    let mut cache = state
        .daily_report_preview_cache
        .lock()
        .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
    evict_oldest_daily_report_preview(&mut cache);
    cache.insert(preview_token, result.cached_preview);

    Ok(response)
}

/// §45.6a（CMD-12-D1）: レジの SD を BIZ-08 §37.9 で読み、候補と scan_token を返す
#[tauri::command]
#[specta::specta]
pub fn scan_register_sd(
    state: State<AppState>,
    selected_path: Option<String>,
) -> Result<RegisterSdScanResponse, CmdError> {
    scan_register_sd_with_state(&state, selected_path, chrono::Local::now().date_naive())
}

fn scan_register_sd_with_state(
    state: &AppState,
    selected_path: Option<String>,
    today: chrono::NaiveDate,
) -> Result<RegisterSdScanResponse, CmdError> {
    let selection = match selected_path {
        None => DailyReportSdSelection::Auto,
        Some(path) if path.is_empty() => {
            return Err(CmdError {
                kind: CmdErrorKind::Validation,
                message: "読む場所を選んでください".to_string(),
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            })
        }
        Some(path) => DailyReportSdSelection::Selected(PathBuf::from(path)),
    };
    let result = {
        let conn = state
            .db
            .lock()
            .map_err(|error| CmdError::internal("DB接続エラー", error))?;
        daily_report_import_service::scan_register_sd_daily_reports(&conn, selection, today)
            .map_err(CmdError::from)?
    };
    let scan_token = uuid::Uuid::new_v4().to_string();
    let mut cache = state
        .register_sd_scan_cache
        .lock()
        .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
    // 同時に持つ scan は最新の 1 つだけ
    cache.clear();
    cache.insert(scan_token.clone(), result.snapshot);
    Ok(RegisterSdScanResponse {
        scan: result.scan,
        scan_token,
    })
}

/// §45.6b（CMD-12-D1）: scan の snapshot の 1 候補から preview を作る。snapshot は消さない
#[tauri::command]
#[specta::specta]
pub fn parse_and_validate_daily_report_from_sd(
    state: State<AppState>,
    scan_token: String,
    candidate_key: String,
) -> Result<DailyReportPreviewResponse, CmdError> {
    parse_from_sd_with_state(&state, scan_token, candidate_key)
}

fn parse_from_sd_with_state(
    state: &AppState,
    scan_token: String,
    candidate_key: String,
) -> Result<DailyReportPreviewResponse, CmdError> {
    if uuid::Uuid::parse_str(&scan_token).is_err() {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "不正なトークンです".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }
    let files = {
        let cache = state
            .register_sd_scan_cache
            .lock()
            .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
        let snapshot = cache
            .get(&scan_token)
            .filter(|snapshot| {
                snapshot.created_at.elapsed().as_secs() <= constants::PREVIEW_CACHE_TTL_SECS
            })
            .ok_or_else(|| CmdError {
                kind: CmdErrorKind::ImportError,
                message: SCAN_EXPIRED_MESSAGE.to_string(),
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            })?;
        snapshot
            .files_by_candidate
            .get(&candidate_key)
            .cloned()
            .ok_or_else(|| CmdError {
                kind: CmdErrorKind::Validation,
                message: "この精算は取り込めません".to_string(),
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            })?
    };
    preview_and_cache(state, files)
}

#[tauri::command]
#[specta::specta]
pub fn commit_daily_report_import(
    state: State<AppState>,
    app: tauri::AppHandle,
    preview_token: String,
    additional_import_confirmed: bool,
) -> Result<DailyReportImportResult, CmdError> {
    // §45.4 手順 3a: 写しの置き場所（BIZ-08-D5）。失敗しても preview の cache は残る
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| CmdError::internal("アプリデータの保存先を取得できませんでした", e))?;
    commit_daily_report_import_with_state(
        &state,
        &app_data_dir,
        preview_token,
        additional_import_confirmed,
    )
}

fn commit_daily_report_import_with_state(
    state: &AppState,
    app_data_dir: &Path,
    preview_token: String,
    additional_import_confirmed: bool,
) -> Result<DailyReportImportResult, CmdError> {
    validate_preview_token(&preview_token)?;
    let cached_preview = {
        let mut cache = state
            .daily_report_preview_cache
            .lock()
            .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
        let Some(cached) = cache.get(&preview_token) else {
            return Err(CmdError {
                kind: CmdErrorKind::ImportError,
                message: "プレビューが見つかりません。再度ファイルを選択してください".to_string(),
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            });
        };
        if cached.created_at.elapsed().as_secs() > constants::PREVIEW_CACHE_TTL_SECS {
            cache.remove(&preview_token);
            return Err(CmdError {
                kind: CmdErrorKind::ImportError,
                message: "プレビューの有効期限が切れました（30分）。再度ファイルを選択してください"
                    .to_string(),
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            });
        }
        cached.clone()
    };

    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    match daily_report_import_service::commit_daily_report_import(
        &mut conn,
        cached_preview,
        additional_import_confirmed,
        app_data_dir,
    ) {
        Ok(result) => {
            drop(conn);
            let mut cache = state
                .daily_report_preview_cache
                .lock()
                .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
            cache.remove(&preview_token);
            Ok(result)
        }
        Err(err) => {
            if err
                .to_string()
                .contains("同日の取込み状況が変わりました。再度プレビューしてください")
            {
                drop(conn);
                let mut cache = state
                    .daily_report_preview_cache
                    .lock()
                    .map_err(|error| CmdError::internal("キャッシュ取得エラー", error))?;
                cache.remove(&preview_token);
            }
            Err(CmdError::from(err))
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn rollback_daily_report_import(
    state: State<AppState>,
    daily_report_import_id: i64,
) -> Result<DailyReportRollbackResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    daily_report_import_service::rollback_daily_report_import(&mut conn, daily_report_import_id)
        .map_err(CmdError::from)
}

#[tauri::command]
#[specta::specta]
pub fn list_daily_report_imports(
    state: State<AppState>,
    page: i64,
    per_page: i64,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<PaginatedResult<DailyReportImportRow>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    daily_report_import_service::list_daily_report_imports(
        &conn,
        ListDailyReportImportsQuery {
            page,
            per_page,
            date_from,
            date_to,
            status: None,
        },
    )
    .map_err(CmdError::from)
}

fn validate_daily_report_files(files: &[DailyReportSourceFileRequest]) -> Result<(), CmdError> {
    if files.len() != 3 {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "Z001/Z002/Z005の3ファイルを選択してください".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }
    if files
        .iter()
        .any(|file| file.file_bytes.len() > constants::CSV_IMPORT_FILE_SIZE_LIMIT)
    {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "ファイルサイズが上限(20MB)を超えています".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }
    Ok(())
}

fn validate_preview_token(preview_token: &str) -> Result<(), CmdError> {
    if uuid::Uuid::parse_str(preview_token).is_err() {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "不正なプレビュートークンです".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }
    Ok(())
}

fn evict_oldest_daily_report_preview(cache: &mut HashMap<String, CachedDailyReportPreview>) {
    if cache.len() >= constants::PREVIEW_CACHE_LIMIT {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, cached)| cached.created_at)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biz::daily_report_import_service::{
        DailyReportDuplicateCheck, DailyReportDuplicateStatus, DailyReportFileInfo,
        DailyReportTotals,
    };
    use crate::db::test_support::setup_test_db;
    use std::time::{Duration, Instant};

    fn request(filename: &str, size: usize) -> DailyReportSourceFileRequest {
        DailyReportSourceFileRequest {
            filename: filename.to_string(),
            file_bytes: vec![0; size],
            source_path: None,
        }
    }

    fn cached(created_at: Instant) -> CachedDailyReportPreview {
        CachedDailyReportPreview {
            created_at,
            preview_data: DailyReportPreviewData {
                file_info: DailyReportFileInfo {
                    report_date: "2026-03-21".to_string(),
                    bundle_hash: "hash".to_string(),
                    source_files: Vec::new(),
                },
                totals: DailyReportTotals {
                    gross_amount: Some(1),
                    net_amount: Some(1),
                },
                payment_summary: Vec::new(),
                department_summary: Vec::new(),
                warnings: Vec::new(),
                duplicate_check: DailyReportDuplicateCheck {
                    status: DailyReportDuplicateStatus::NoDuplicate,
                    same_date_imports: Vec::new(),
                },
                preview_created_at: "2026-03-21T10:00:00".to_string(),
            },
            settlement_no: None,
            sd_source_files: Vec::new(),
            summary_lines: Vec::new(),
            payment_lines: Vec::new(),
            department_lines: Vec::new(),
            active_same_date_import_ids: Vec::new(),
        }
    }

    fn cached_with_status(
        created_at: Instant,
        status: DailyReportDuplicateStatus,
    ) -> CachedDailyReportPreview {
        let mut cached = cached(created_at);
        cached.preview_data.duplicate_check.status = status;
        cached
    }

    fn app_state() -> (tempfile::TempDir, AppState) {
        let (dir, conn) = setup_test_db();
        (
            dir,
            AppState {
                db: std::sync::Mutex::new(conn),
                preview_cache: std::sync::Mutex::new(HashMap::new()),
                daily_report_preview_cache: std::sync::Mutex::new(HashMap::new()),
                register_sd_scan_cache: std::sync::Mutex::new(HashMap::new()),
            },
        )
    }

    #[test]
    fn test_daily_report_cmd_req401_validates_three_files() {
        let err = validate_daily_report_files(&[request("Z001.csv", 1)]).unwrap_err();
        assert_eq!(err.kind, CmdErrorKind::Validation);

        let ok = validate_daily_report_files(&[
            request("Z001.csv", 1),
            request("Z002.csv", 1),
            request("Z005.csv", 1),
        ]);
        assert!(ok.is_ok());
    }

    #[test]
    fn test_daily_report_cmd_req401_validates_size_limit() {
        let err = validate_daily_report_files(&[
            request("Z001.csv", 1),
            request("Z002.csv", constants::CSV_IMPORT_FILE_SIZE_LIMIT + 1),
            request("Z005.csv", 1),
        ])
        .unwrap_err();

        assert_eq!(err.kind, CmdErrorKind::Validation);
    }

    #[test]
    fn test_daily_report_cmd_req401_validates_uuid_token() {
        assert!(validate_preview_token("not-a-uuid").is_err());
        assert!(validate_preview_token(&uuid::Uuid::new_v4().to_string()).is_ok());
    }

    #[test]
    fn test_daily_report_cmd_req401_cache_miss_and_expiry_return_import_error() {
        // REQ-401 / CMD-12: cache miss / 期限切れ token は import_error
        let (_dir, state) = app_state();
        let missing_token = uuid::Uuid::new_v4().to_string();
        let missing =
            commit_daily_report_import_with_state(&state, _dir.path(), missing_token, false)
                .expect_err("cache miss should fail");
        assert_eq!(missing.kind, CmdErrorKind::ImportError);

        let expired_token = uuid::Uuid::new_v4().to_string();
        state.daily_report_preview_cache.lock().unwrap().insert(
            expired_token.clone(),
            cached(Instant::now() - Duration::from_secs(constants::PREVIEW_CACHE_TTL_SECS + 1)),
        );
        let expired = commit_daily_report_import_with_state(
            &state,
            _dir.path(),
            expired_token.clone(),
            false,
        )
        .expect_err("expired cache should fail");
        assert_eq!(expired.kind, CmdErrorKind::ImportError);
        assert!(!state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .contains_key(&expired_token));
    }

    #[test]
    fn test_daily_report_cmd_req401_cache_lifecycle_success_removes_failure_keeps() {
        // REQ-401 / CMD-12: commit成功でtoken削除、BIZ失敗では再試行用にcacheを残す
        let (_dir, state) = app_state();
        let success_token = uuid::Uuid::new_v4().to_string();
        state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .insert(success_token.clone(), cached(Instant::now()));

        commit_daily_report_import_with_state(&state, _dir.path(), success_token.clone(), false)
            .unwrap();
        assert!(!state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .contains_key(&success_token));

        let failure_token = uuid::Uuid::new_v4().to_string();
        state.daily_report_preview_cache.lock().unwrap().insert(
            failure_token.clone(),
            cached_with_status(Instant::now(), DailyReportDuplicateStatus::AlreadyImported),
        );

        let failed = commit_daily_report_import_with_state(
            &state,
            _dir.path(),
            failure_token.clone(),
            false,
        )
        .expect_err("BIZ failure should keep cache");
        assert_eq!(failed.kind, CmdErrorKind::IdempotencyConflict);
        assert!(state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .contains_key(&failure_token));
    }

    #[test]
    fn test_daily_report_cmd_req401_fifo_eviction_removes_oldest() {
        let mut cache = HashMap::new();
        for index in 0..constants::PREVIEW_CACHE_LIMIT {
            cache.insert(
                format!("token-{}", index),
                cached(Instant::now() - Duration::from_secs((index + 1) as u64)),
            );
        }

        evict_oldest_daily_report_preview(&mut cache);

        assert_eq!(cache.len(), constants::PREVIEW_CACHE_LIMIT - 1);
        assert!(!cache.contains_key(&format!("token-{}", constants::PREVIEW_CACHE_LIMIT - 1)));
    }

    #[test]
    fn test_daily_report_cmd_req401_snapshot_mismatch_deletes_preview_token() {
        // REQ-401 / I-W5 / SPEC-SDI-D4: mismatch tokenは破棄し新previewを要求する。
        let (_dir, state) = app_state();
        let concurrent_id = crate::db::sales_repo::insert_daily_report_import(
            &state.db.lock().unwrap(),
            &crate::db::sales_repo::NewDailyReportImport {
                report_date: "2026-03-21".into(),
                source_adapter: "casio_sr_s4000".into(),
                bundle_hash: "concurrent".into(),
                source_files_json: "[]".into(),
                gross_amount: Some(1),
                net_amount: Some(1),
                status: "completed".into(),
                note: None,
                settlement_no: None,
            },
        )
        .unwrap();
        let token = uuid::Uuid::new_v4().to_string();
        state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .insert(token.clone(), cached(Instant::now()));

        let error =
            commit_daily_report_import_with_state(&state, _dir.path(), token.clone(), false)
                .unwrap_err();
        assert_eq!(
            error.message,
            "同日の取込み状況が変わりました。再度プレビューしてください"
        );
        assert!(!state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .contains_key(&token));
        let retry =
            commit_daily_report_import_with_state(&state, _dir.path(), token, false).unwrap_err();
        assert_eq!(retry.kind, CmdErrorKind::ImportError);

        let new_token = uuid::Uuid::new_v4().to_string();
        let mut fresh = cached_with_status(
            Instant::now(),
            DailyReportDuplicateStatus::AdditionalImportConfirmationRequired,
        );
        fresh.active_same_date_import_ids = vec![concurrent_id];
        fresh.preview_data.file_info.bundle_hash = "fresh-hash".to_string();
        state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .insert(new_token.clone(), fresh);
        let committed =
            commit_daily_report_import_with_state(&state, _dir.path(), new_token.clone(), true)
                .unwrap();
        assert_eq!(committed.status, "completed");
        assert!(!state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .contains_key(&new_token));
    }

    // -----------------------------------------------------------------------
    // CMD-12-D1（D-111）: scan cache と SD からの preview、写しの失敗の再試行
    // -----------------------------------------------------------------------

    use crate::biz::daily_report_import_service::{
        DailyReportSdCandidateStatus, DailyReportSdScanSnapshot,
    };

    const SOURCE_COPY_FAILED_TEXT: &str = "SD から読んだファイルの写しを PC に保存できなかったため、取り込みませんでした。PC の空き容量を確かめて、もう一度取り込んでください。";

    /// 合成の Z001 / Z002 / Z005 の bytes（layout A、CP932、CRLF）
    fn z_bytes(kind: &str, date: &str, settlement: &str) -> Vec<u8> {
        let rows = match kind {
            "Z001" => "\"101\",\"総売\",\"8\",\"12000\"\r\n\"201\",\"純売\",\"7\",\"11000\"\r\n",
            "Z002" => "\"01\",\"現金\",\"7\",\"11000\"\r\n",
            _ => "\"01\",\"その他小物\",\"4\",\"3000\"\r\n",
        };
        let text = format!(
            "\"マシンNo.   \",\"01\",\"\",\"\"\r\n\"精算回数    \",\"{settlement}\",\"\",\"\"\r\n\"日付        \",\"{date}\",\"\",\"\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数/件数   \",\"金額        \"\r\n{rows}"
        );
        encoding_rs::SHIFT_JIS.encode(&text).0.to_vec()
    }

    /// 一時 directory にレジの SD の tree を作り、XZ に 1 組を置いて root の path を返す
    fn sd_tree(base: &Path, date: &str, settlement: &str) -> PathBuf {
        let root = crate::io::REGISTER_SD_FOLDER
            .iter()
            .fold(base.to_path_buf(), |path, part| path.join(part));
        let (year, rest) = date.split_at(4);
        let month_dir = root.join("XZ").join(year).join(&rest[1..3]);
        std::fs::create_dir_all(&month_dir).unwrap();
        for kind in ["Z001", "Z002", "Z005"] {
            std::fs::write(
                month_dir.join(format!("{kind}_{} .CSV", &date[8..10])),
                z_bytes(kind, date, settlement),
            )
            .unwrap();
        }
        root
    }

    fn today() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()
    }

    fn snapshot(created_at: Instant, key: &str) -> DailyReportSdScanSnapshot {
        let mut files_by_candidate = HashMap::new();
        files_by_candidate.insert(key.to_string(), Vec::new());
        DailyReportSdScanSnapshot {
            created_at,
            files_by_candidate,
        }
    }

    #[test]
    fn test_req401_from_sd_rejects_expired_scan() {
        // REQ-401 / CMD-12-D1 §45.6b 手順 2: 31 分前の scan と知らない token は import_error と固定の文
        let (_dir, state) = app_state();
        let token = uuid::Uuid::new_v4().to_string();
        state.register_sd_scan_cache.lock().unwrap().insert(
            token.clone(),
            snapshot(Instant::now() - Duration::from_secs(31 * 60), "k"),
        );
        for token in [token, uuid::Uuid::new_v4().to_string()] {
            let error = parse_from_sd_with_state(&state, token, "k".into()).unwrap_err();
            assert_eq!(error.kind, CmdErrorKind::ImportError);
            assert_eq!(
                error.message,
                "SD を読んでから時間がたちました。もう一度 SD を読んでください。"
            );
        }
        let invalid =
            parse_from_sd_with_state(&state, "not-a-uuid".into(), "k".into()).unwrap_err();
        assert_eq!(invalid.kind, CmdErrorKind::Validation);
    }

    #[test]
    fn test_req401_from_sd_rejects_unknown_candidate() {
        // REQ-401 / CMD-12-D1 §45.6b 手順 3: snapshot に無い candidate_key は validation
        let (_dir, state) = app_state();
        let token = uuid::Uuid::new_v4().to_string();
        state
            .register_sd_scan_cache
            .lock()
            .unwrap()
            .insert(token.clone(), snapshot(Instant::now(), "known"));
        let error = parse_from_sd_with_state(&state, token, "other".into()).unwrap_err();
        assert_eq!(error.kind, CmdErrorKind::Validation);
        assert!(state.daily_report_preview_cache.lock().unwrap().is_empty());
    }

    #[test]
    fn test_req401_new_scan_replaces_cache() {
        // REQ-401 / CMD-12-D1 §45.6a 手順 3: 2 回目の scan の後、1 回目の scan_token は使えない
        let (dir, state) = app_state();
        let root = sd_tree(&dir.path().join("sd"), "2026-03-21", "0005");
        let selected = Some(root.display().to_string());
        let first = scan_register_sd_with_state(&state, selected.clone(), today()).unwrap();
        let key = first.scan.candidates[0].candidate_key.clone();
        let second = scan_register_sd_with_state(&state, selected, today()).unwrap();
        assert_ne!(first.scan_token, second.scan_token);
        assert_eq!(state.register_sd_scan_cache.lock().unwrap().len(), 1);
        let error = parse_from_sd_with_state(&state, first.scan_token, key.clone()).unwrap_err();
        assert_eq!(error.kind, CmdErrorKind::ImportError);
        assert!(parse_from_sd_with_state(&state, second.scan_token, key).is_ok());
        // 空文字の選択は validation
        let empty = scan_register_sd_with_state(&state, Some(String::new()), today()).unwrap_err();
        assert_eq!(empty.kind, CmdErrorKind::Validation);
    }

    #[test]
    #[cfg(unix)]
    fn test_req401_copy_failure_keeps_preview_token_for_retry() {
        // REQ-401 / BIZ-08-D5 / §45.7: 写しの失敗は internal と固定の文で、同じ token で再試行できる
        use std::os::unix::fs::PermissionsExt;
        let (dir, state) = app_state();
        let root = sd_tree(&dir.path().join("sd"), "2026-03-21", "0005");
        let scan =
            scan_register_sd_with_state(&state, Some(root.display().to_string()), today()).unwrap();
        let candidate = &scan.scan.candidates[0];
        assert_eq!(candidate.status, DailyReportSdCandidateStatus::NotImported);
        let preview = parse_from_sd_with_state(
            &state,
            scan.scan_token.clone(),
            candidate.candidate_key.clone(),
        )
        .unwrap();

        let readonly = dir.path().join("readonly-app");
        std::fs::create_dir_all(&readonly).unwrap();
        std::fs::set_permissions(&readonly, std::fs::Permissions::from_mode(0o555)).unwrap();
        let error = commit_daily_report_import_with_state(
            &state,
            &readonly,
            preview.preview_token.clone(),
            false,
        )
        .unwrap_err();
        std::fs::set_permissions(&readonly, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(error.kind, CmdErrorKind::Internal);
        assert_eq!(error.message, SOURCE_COPY_FAILED_TEXT);
        assert!(state
            .daily_report_preview_cache
            .lock()
            .unwrap()
            .contains_key(&preview.preview_token));

        let writable = dir.path().join("app");
        let committed = commit_daily_report_import_with_state(
            &state,
            &writable,
            preview.preview_token.clone(),
            false,
        )
        .unwrap();
        assert_eq!(committed.status, "completed");
        let month = writable
            .join("pos-sources")
            .join("casio-sr-s4000")
            .join("sd")
            .join("XZ")
            .join("2026")
            .join("03");
        assert_eq!(std::fs::read_dir(month).unwrap().count(), 3);

        // 取り込んだ後の scan で同じ束は Imported になり、その key では preview を作らない
        let rescan =
            scan_register_sd_with_state(&state, Some(root.display().to_string()), today()).unwrap();
        assert_eq!(
            rescan.scan.candidates[0].status,
            DailyReportSdCandidateStatus::Imported
        );
        let error = parse_from_sd_with_state(
            &state,
            rescan.scan_token,
            rescan.scan.candidates[0].candidate_key.clone(),
        )
        .unwrap_err();
        assert_eq!(error.kind, CmdErrorKind::Validation);
    }
}
