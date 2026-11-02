//! BIZ-08: 日報取込みパイプライン

mod commit;
mod list;
mod parse;
mod rollback;
mod scan;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use commit::commit_daily_report_import;
#[allow(unused_imports)]
pub use list::list_daily_report_imports;
#[allow(unused_imports)]
pub use parse::parse_and_validate_daily_report;
#[allow(unused_imports)]
pub use parse::parse_and_validate_daily_report_with_sd_lookup;
#[allow(unused_imports)]
pub use rollback::rollback_daily_report_import;
#[allow(unused_imports)]
pub use scan::scan_register_sd_daily_reports;

use crate::db::sales_repo::DailyReportImport;
use crate::io::daily_report_parser::DailyReportSourceKind;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// BIZ-08-D4 の固定の文（§37.3 手順 8・§37.4 手順 4a）
pub(crate) const SAME_SETTLEMENT_MESSAGE: &str = "同じ精算の日報が、別のファイルからすでに取り込まれています。二重に数えないため、取り込みません。取り込み直すときは、前の取込みを取り消してから読み直してください。";
/// BIZ-08-D6 の固定の文（§37.3 手順 4・§37.4 手順 2a）
pub(crate) const SD_WITHOUT_SETTLEMENT_MESSAGE: &str =
    "この精算の日報は、SD のファイルから精算回数を確かめられないため、SD からは取り込めません。";

#[derive(Debug)]
pub struct DailyReportParseValidateResult {
    pub preview_data: DailyReportPreviewData,
    pub cached_preview: CachedDailyReportPreview,
}

#[derive(Debug, Clone)]
pub struct DailyReportInputFile {
    pub filename: String,
    pub bytes: Vec<u8>,
    /// SD の file だけ Some（IO-09 の relative_path）。写し（BIZ-08-D5）と精算回数の検査（BIZ-08-D6）に使う
    pub sd_relative_path: Option<String>,
    /// 手でファイルを選ぶ経路の dialog の path（UI-07-D16）。§37.3 手順 1a で SD 上かを確かめる
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportPreviewData {
    pub file_info: DailyReportFileInfo,
    pub totals: DailyReportTotals,
    pub payment_summary: Vec<DailyReportPaymentLinePreview>,
    pub department_summary: Vec<DailyReportDepartmentLinePreview>,
    pub warnings: Vec<DailyReportWarning>,
    pub duplicate_check: DailyReportDuplicateCheck,
    pub preview_created_at: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportFileInfo {
    pub report_date: String,
    pub bundle_hash: String,
    pub source_files: Vec<DailyReportSourceFileInfo>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportSourceFileInfo {
    pub source: DailyReportSourceKind,
    pub filename: String,
    pub file_hash: String,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportTotals {
    pub gross_amount: Option<i64>,
    pub net_amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportPaymentLinePreview {
    pub payment_key: String,
    pub label: String,
    pub amount: Option<i64>,
    pub count: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportDepartmentLinePreview {
    pub department_id: Option<i64>,
    pub raw_department_name: String,
    pub normalized_department_name: Option<String>,
    pub amount: i64,
    // 単位の数（IO-07-D2 の 100 倍の整数を戻した値）
    pub quantity: Option<f64>,
    pub count: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportWarning {
    pub code: String,
    pub message: String,
    pub source_file: Option<DailyReportSourceKind>,
    pub line_no: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, specta::Type)]
pub enum DailyReportDuplicateStatus {
    NoDuplicate,
    AlreadyImported,
    AdditionalImportConfirmationRequired,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportDuplicateCheck {
    pub status: DailyReportDuplicateStatus,
    pub same_date_imports: Vec<SameDateDailyReportImportSummary>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct SameDateDailyReportImportSummary {
    pub id: i64,
    pub source_filenames: Vec<String>,
    pub gross_amount: Option<i64>,
    pub net_amount: Option<i64>,
    pub imported_at: String,
}

#[derive(Debug, Clone)]
pub struct CachedDailyReportPreview {
    pub created_at: Instant,
    pub preview_data: DailyReportPreviewData,
    /// IO-07-D5。commit で保存し、TX 内の再検査に使う（BIZ-08-D4）
    pub settlement_no: Option<i64>,
    /// `sd_relative_path` が Some の file。commit で写しを書く（BIZ-08-D5）
    pub sd_source_files: Vec<SdSourceFileForCopy>,
    pub summary_lines: Vec<CachedDailyReportSummaryLine>,
    pub payment_lines: Vec<DailyReportPaymentLinePreview>,
    pub department_lines: Vec<CachedDailyReportDepartmentLine>,
    pub active_same_date_import_ids: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct SdSourceFileForCopy {
    pub source: DailyReportSourceKind,
    pub sd_relative_path: String,
    pub bytes: Vec<u8>,
}

/// SD の読み方の選択（37 §37.9）。CMD-12 が作って渡す（cmd → io は禁止のため IO の型にしない）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DailyReportSdSelection {
    /// 取外し可能な drive から探す
    Auto,
    /// 利用者が folder の選択で選んだ path
    Selected(PathBuf),
}

#[derive(Debug)]
pub struct DailyReportSdScanResult {
    pub scan: DailyReportSdScan,
    pub snapshot: DailyReportSdScanSnapshot,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportSdScan {
    pub volume_label: String,
    /// YYYY-MM-DD（窓の始まり）
    pub scanned_from: String,
    pub candidates: Vec<DailyReportSdCandidate>,
    /// 名前の規則に合わず読まなかった file の数（IO-09-D2）
    pub skipped_unknown_count: i64,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportSdCandidate {
    pub candidate_key: String,
    pub path_date: String,
    pub report_date: Option<String>,
    pub source_filenames: Vec<String>,
    pub status: DailyReportSdCandidateStatus,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, specta::Type)]
pub enum DailyReportSdCandidateStatus {
    NotImported,
    Imported,
    SameSettlementImported,
    Incomplete,
    Unreadable,
}

/// CMD-12 の scan cache に置く内部専用の snapshot
#[derive(Debug, Clone)]
pub struct DailyReportSdScanSnapshot {
    pub created_at: Instant,
    /// NotImported の束だけ
    pub files_by_candidate: HashMap<String, Vec<DailyReportInputFile>>,
}

#[derive(Debug, Clone)]
pub struct CachedDailyReportSummaryLine {
    pub line_key: String,
    pub label: String,
    pub amount: Option<i64>,
    pub quantity_hundredths: Option<i64>,
    pub count: Option<i64>,
    pub sort_order: i64,
}

/// commit 用の部門の行。個数は 100 倍の整数のまま持つ（§37.2）
#[derive(Debug, Clone)]
pub struct CachedDailyReportDepartmentLine {
    pub department_id: Option<i64>,
    pub raw_department_name: String,
    pub normalized_department_name: Option<String>,
    pub amount: i64,
    pub quantity_hundredths: Option<i64>,
    pub count: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct DailyReportImportResult {
    pub daily_report_import_id: i64,
    pub status: String,
    pub report_date: String,
    pub gross_amount: Option<i64>,
    pub net_amount: Option<i64>,
    pub warning_count: i64,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct DailyReportRollbackResult {
    pub daily_report_import_id: i64,
    pub status: String,
    pub rolled_back_at: Option<String>,
}

#[derive(Debug)]
pub struct ListDailyReportImportsQuery {
    pub page: i64,
    pub per_page: i64,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub status: Option<String>,
}

pub type DailyReportImportRow = DailyReportImport;

pub(crate) fn source_kind_order(source: DailyReportSourceKind) -> i64 {
    match source {
        DailyReportSourceKind::Z001 => 1,
        DailyReportSourceKind::Z002 => 2,
        DailyReportSourceKind::Z005 => 3,
    }
}

pub(crate) fn source_kind_db_value(source: DailyReportSourceKind) -> &'static str {
    match source {
        DailyReportSourceKind::Z001 => "Z001",
        DailyReportSourceKind::Z002 => "Z002",
        DailyReportSourceKind::Z005 => "Z005",
    }
}
