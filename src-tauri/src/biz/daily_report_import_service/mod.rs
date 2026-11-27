//! BIZ-08: 日報取込みパイプライン

mod bulk;
mod commit;
mod ej;
mod list;
mod parse;
mod rollback;
mod scan;
mod tidy;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use bulk::commit_register_sd_bulk_import;
#[allow(unused_imports)]
pub use commit::commit_daily_report_import;
#[allow(unused_imports)]
pub use ej::{
    find_ej_records_by_no, get_ej_record, import_ej_records, list_ej_records, plan_ej_records,
    EjConflict, EjImportPlan, EjImportResult, EjRecordDay, EjSourceFile,
};
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

use crate::biz::BizError;
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
    /// 取り込んだ精算の Z を `XZ_BKUP` へ移せなかった理由（37 §37.12、BIZ-08-D11）。None = 移す物が無い・全部移せた・片付けない
    pub tidy_failure: Option<RegisterSdTidyFailure>,
    /// EJ の数（37 §37.10 の「§37.9 に足すもの」(A)）。snapshot の ej_files と ej_unreadable_files がどちらも空なら None
    pub ej: Option<RegisterSdEjSummary>,
}

/// scan の EJ の数（37 §37.10）。55 UI-07-D17 のボタンの条件と一覧の文
#[derive(Debug, Clone, Serialize, PartialEq, Eq, specta::Type)]
pub struct RegisterSdEjSummary {
    /// plan_ej_records の new_records の数（保留した記録を含まない）
    pub new_count: i64,
    pub held_count: i64,
    pub conflict_count: i64,
    /// ej_unreadable_files の数 + plan_ej_records の unreadable_files の数
    pub unreadable_file_count: i64,
}

/// SD から読んだ 1 file（37 §37.10）
#[derive(Debug, Clone)]
pub struct SdSourceFile {
    /// IO-09 の RegisterSdEntry.relative_path
    pub sd_relative_path: String,
    pub bytes: Vec<u8>,
}

/// 37 §37.10 一括取込みの結果（BIZ の内部の型。wire は CMD-12 の RegisterSdBulkResponse）
#[derive(Debug)]
pub struct RegisterSdBulkResult {
    /// 処理の順（日報 → EJ）。止まったかは Failed の item の有無で決まる
    pub items: Vec<RegisterSdBulkItem>,
}

#[derive(Debug)]
pub struct RegisterSdBulkItem {
    pub source: RegisterSdBulkSource,
    pub outcome: RegisterSdBulkOutcome,
}

/// 項目の出所（45 §45.6c は wire にそのまま出す）
#[derive(Debug, Clone, Serialize, PartialEq, Eq, specta::Type)]
#[serde(tag = "type")]
pub enum RegisterSdBulkSource {
    DailyReport {
        candidate_key: String,
        report_date: Option<String>,
    },
    /// EJ の file 全部で 1 項目
    Ej,
}

#[derive(Debug)]
pub enum RegisterSdBulkOutcome {
    ImportedDailyReport(DailyReportImportResult),
    /// 新しい記録が 0 件でも ImportedEj（件数で示す）
    ImportedEj(EjImportResult),
    /// 業務の検査で取り込まない。message は BIZ の固定の文のまま
    Skipped {
        message: String,
    },
    /// 日報の同日追加の確認が要る（BIZ-08-D8 の外）
    NeedsIndividualConfirmation,
    /// 確定の段の失敗。ここで止まる。CMD が 45 §45.7 の変換を当てる
    Failed(BizError),
    /// 止まった後の項目
    NotAttempted,
}

/// 37 §37.12: 片付けの失敗の種類。画面の固定の文を選ぶだけ（55 UI-07-D15 (7)）。数と path は診断ログ
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, specta::Type)]
pub enum RegisterSdTidyFailure {
    /// SD の側の書込みの失敗（IO-11 の Io・移した後の確かめの食い違い）
    SdWrite,
    /// IO-10 の写しの失敗
    PcCopy,
    /// DB の読取りの失敗、写しの置き場所を得られない、IO-11 の InvalidPath・NameRule・NoFreeSequence
    Other,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportSdCandidate {
    pub candidate_key: String,
    pub path_date: String,
    pub report_date: Option<String>,
    pub source_filenames: Vec<String>,
    /// 束の精算回数（IO-07-D5、TD-592）。読めなかった束は None
    pub settlement_no: Option<i64>,
    pub status: DailyReportSdCandidateStatus,
    /// 数の列の誤り（`invalid_number`・`number_too_large`）で `Unreadable` の束だけ Some。帳票・列を名指して
    /// 管理者に知らせるよう伝える文（BIZ-08-D2a、§37.9 手順 5）。値・file 名・行番号を含まない
    pub unreadable_reason: Option<String>,
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
    /// 読めた EJ（形の検査の前。37 §37.10 の「§37.9 に足すもの」(A)）
    pub ej_files: Vec<SdSourceFile>,
    /// bytes を得られなかった EJ の file 名（相対 path の最後の要素）
    pub ej_unreadable_files: Vec<String>,
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

/// 一覧の行（37 §37.6、45 §45.6）。repo の行から内部の保存の形を外した wire の型
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DailyReportImportRow {
    pub id: i64,
    pub report_date: String,
    pub source_filenames: Vec<String>,
    /// 束の精算回数（IO-07-D5）。記録の無い取込みは None（TD-590）
    pub settlement_no: Option<i64>,
    pub gross_amount: Option<i64>,
    pub net_amount: Option<i64>,
    pub status: String,
    pub imported_at: String,
    pub rolled_back_at: Option<String>,
}

/// 保存の `source_files_json`（§37.4 手順 6）から各 file の `filename` を取り出す。読めなければ ImportError
pub(super) fn source_filenames(source_files_json: &str) -> Result<Vec<String>, BizError> {
    let files: Vec<serde_json::Value> = serde_json::from_str(source_files_json)
        .map_err(|_| BizError::ImportError("既存日報のファイル情報を読み取れません".to_string()))?;
    files
        .into_iter()
        .map(|file| {
            file.get("filename")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| {
                    BizError::ImportError("既存日報のファイル情報を読み取れません".to_string())
                })
        })
        .collect()
}

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
