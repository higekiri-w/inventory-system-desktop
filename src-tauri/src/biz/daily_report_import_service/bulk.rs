//! BIZ-08-D7: SD の締めの一括取込み（37 §37.10 手順 1 の (a)・(c)、手順 2・4・5・6）。
//! Z004（手順 1 (b)・手順 3）は runtime (B) が足す

use crate::biz::daily_report_import_service::ej::import_ej_records_after_scan;
use crate::biz::daily_report_import_service::scan::last_element;
use crate::biz::daily_report_import_service::{
    commit_daily_report_import, parse_and_validate_daily_report, DailyReportDuplicateStatus,
    DailyReportInputFile, DailyReportSdScanSnapshot, EjSourceFile, RegisterSdBulkItem,
    RegisterSdBulkOutcome, RegisterSdBulkResult, RegisterSdBulkSource,
};
use crate::biz::BizError;
use crate::db::DbConnection;
use crate::io::daily_report_parser::{self, DailyReportSourceFile};
use std::path::Path;

const ALREADY_IMPORTED_MESSAGE: &str = "取込み済みです";

/// 37 §37.10 `commit_register_sd_bulk_import`。項目の失敗は `Ok` の中の outcome で返す
pub fn commit_register_sd_bulk_import(
    conn: &mut DbConnection,
    snapshot: &DailyReportSdScanSnapshot,
    app_data_dir: &Path,
) -> Result<RegisterSdBulkResult, BizError> {
    let mut items = Vec::new();
    let mut stopped = false;

    // 手順 1 (a)・2: 日報を精算の順に 1 束ずつ
    for (candidate_key, files, report_date) in ordered_bundles(snapshot) {
        let source = RegisterSdBulkSource::DailyReport {
            candidate_key: candidate_key.clone(),
            report_date,
        };
        let outcome = if stopped {
            RegisterSdBulkOutcome::NotAttempted
        } else {
            import_bundle(conn, files.clone(), app_data_dir)
        };
        stopped |= matches!(outcome, RegisterSdBulkOutcome::Failed(_));
        items.push(RegisterSdBulkItem { source, outcome });
    }

    // 手順 1 (c)・4: EJ の file 全部で 1 項目（どちらも空なら項目を作らない）
    if !snapshot.ej_files.is_empty() || !snapshot.ej_unreadable_files.is_empty() {
        let outcome = if stopped {
            RegisterSdBulkOutcome::NotAttempted
        } else {
            let files = snapshot
                .ej_files
                .iter()
                .map(|file| EjSourceFile {
                    filename: last_element(&file.sd_relative_path).to_string(),
                    sd_relative_path: Some(file.sd_relative_path.clone()),
                    bytes: file.bytes.clone(),
                })
                .collect();
            // scan で読めなかった file を前に足す（結果と操作ログの数を同じにする）
            match import_ej_records_after_scan(
                conn,
                files,
                snapshot.ej_unreadable_files.clone(),
                app_data_dir,
            ) {
                Ok(result) => RegisterSdBulkOutcome::ImportedEj(result),
                Err(error) => RegisterSdBulkOutcome::Failed(error),
            }
        };
        items.push(RegisterSdBulkItem {
            source: RegisterSdBulkSource::Ej,
            outcome,
        });
    }
    Ok(RegisterSdBulkResult { items })
}

/// 手順 1 (a): 中身の (report_date, 精算回数) の昇順。parse の失敗した束は先頭。
/// 鍵が等しければ `XZ` の束を先にし、最後に candidate_key の辞書順（files_by_candidate は HashMap なので順を決める）
fn ordered_bundles(
    snapshot: &DailyReportSdScanSnapshot,
) -> Vec<(&String, &Vec<DailyReportInputFile>, Option<String>)> {
    let mut bundles: Vec<_> = snapshot
        .files_by_candidate
        .iter()
        .map(|(key, files)| {
            let parsed = daily_report_parser::parse_daily_report_bundle(
                files
                    .iter()
                    .map(|file| DailyReportSourceFile {
                        filename: file.filename.clone(),
                        bytes: file.bytes.clone(),
                    })
                    .collect(),
            );
            let report_date = parsed.report_date;
            let content_key = match (
                parsed.parse_errors.is_empty(),
                &report_date,
                parsed.settlement_no,
            ) {
                (true, Some(date), Some(no)) => Some((date.clone(), no)),
                _ => None,
            };
            let in_sales = files
                .first()
                .and_then(|file| file.sd_relative_path.as_deref())
                .is_some_and(|path| {
                    // IO-09 は大文字小文字を区別せずに売上の folder を受ける（小文字の `xz` も XZ）
                    path.split('\\')
                        .next()
                        .is_some_and(|first| first.eq_ignore_ascii_case("XZ"))
                });
            (
                (content_key, !in_sales, key.clone()),
                (key, files, report_date),
            )
        })
        .collect();
    bundles.sort_by(|a, b| a.0.cmp(&b.0));
    bundles.into_iter().map(|(_, bundle)| bundle).collect()
}

/// 手順 2: §37.3 → §37.4。業務の拒否は Skipped で続け、確定の段の失敗は Failed で止める
fn import_bundle(
    conn: &mut DbConnection,
    files: Vec<DailyReportInputFile>,
    app_data_dir: &Path,
) -> RegisterSdBulkOutcome {
    let parsed = match parse_and_validate_daily_report(conn, files) {
        Ok(parsed) => parsed,
        Err(
            BizError::ImportError(message)
            | BizError::IdempotencyConflict(message)
            | BizError::ValidationFailed(message),
        ) => return RegisterSdBulkOutcome::Skipped { message },
        Err(error) => return RegisterSdBulkOutcome::Failed(error),
    };
    match parsed.preview_data.duplicate_check.status {
        DailyReportDuplicateStatus::AlreadyImported => RegisterSdBulkOutcome::Skipped {
            message: ALREADY_IMPORTED_MESSAGE.to_string(),
        },
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired => {
            RegisterSdBulkOutcome::NeedsIndividualConfirmation
        }
        DailyReportDuplicateStatus::NoDuplicate => {
            match commit_daily_report_import(conn, parsed.cached_preview, false, app_data_dir) {
                Ok(result) => RegisterSdBulkOutcome::ImportedDailyReport(result),
                Err(error) => RegisterSdBulkOutcome::Failed(error),
            }
        }
    }
}
