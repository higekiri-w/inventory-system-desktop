use crate::biz::daily_report_import_service::{
    source_kind_order, CachedDailyReportDepartmentLine, CachedDailyReportPreview,
    CachedDailyReportSummaryLine, DailyReportDepartmentLinePreview, DailyReportDuplicateCheck,
    DailyReportDuplicateStatus, DailyReportFileInfo, DailyReportInputFile,
    DailyReportParseValidateResult, DailyReportPaymentLinePreview, DailyReportPreviewData,
    DailyReportSourceFileInfo, DailyReportTotals, DailyReportWarning,
    SameDateDailyReportImportSummary, SdSourceFileForCopy, SAME_SETTLEMENT_MESSAGE,
    SD_WITHOUT_SETTLEMENT_MESSAGE,
};
use crate::biz::BizError;
use crate::constants;
use crate::db::product_repo;
use crate::db::sales_repo;
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::DbConnection;
use crate::io::daily_report_parser::{self, DailyReportSourceFile, DailyReportSourceKind};
use crate::io::register_sd::{self, RegisterSdError, RegisterSdRoot};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Instant;

const SETTLEMENT_MISMATCH_MESSAGE: &str = "別の精算の日報ファイルが混ざっています。ファイル名の「Z001」「Z002」「Z005」より後ろが同じ 3 つを選び直してください。";

const SD_LOOKUP_FAILED_MESSAGE: &str =
    "選んだファイルがレジの SD の上にあるかを確かめられませんでした。もう一度選んでください。";
const SELECTED_FILE_MISSING_MESSAGE: &str = "選んだファイルが見つからなくなりました。SD から選んだときは、SD をこの PC に差したまま、もう一度選んでください。";

/// 37 §37.3。手順 1a の root の列は IO-09 `find_register_sd_roots` から取る
pub fn parse_and_validate_daily_report(
    conn: &DbConnection,
    files: Vec<DailyReportInputFile>,
) -> Result<DailyReportParseValidateResult, BizError> {
    parse_and_validate_daily_report_with_sd_lookup(conn, files, register_sd::find_register_sd_roots)
}

/// 37 §37.3。手順 1a の root の列を取る処理を引数で受ける（test は一時 directory の root か失敗を渡す）
pub fn parse_and_validate_daily_report_with_sd_lookup(
    conn: &DbConnection,
    mut files: Vec<DailyReportInputFile>,
    find_sd_roots: impl FnOnce() -> Result<Vec<RegisterSdRoot>, RegisterSdError>,
) -> Result<DailyReportParseValidateResult, BizError> {
    if files
        .iter()
        .any(|file| file.bytes.len() > constants::CSV_IMPORT_FILE_SIZE_LIMIT)
    {
        return Err(BizError::ImportError(
            "ファイルサイズが上限（20MB）を超えています".to_string(),
        ));
    }

    resolve_selected_file_origins(&mut files, find_sd_roots)?;
    // 手順 9 の sd_source_files のため、SD の file の (file 名, 相対 path, bytes) を先に取っておく
    let sd_files: Vec<(String, String, Vec<u8>)> = files
        .iter()
        .filter_map(|file| {
            file.sd_relative_path
                .clone()
                .map(|relative| (file.filename.clone(), relative, file.bytes.clone()))
        })
        .collect();

    let source_files = files
        .into_iter()
        .map(|file| DailyReportSourceFile {
            filename: file.filename,
            bytes: file.bytes,
        })
        .collect();
    let parse_result = daily_report_parser::parse_daily_report_bundle(source_files);
    if !parse_result.parse_errors.is_empty() {
        log_parse_diagnostics(&parse_result.parse_errors);
        // BIZ-08-D2: 別の精算の混在は選び直しで直るので、error_type だけから固定の文を作る
        let message = if parse_result
            .parse_errors
            .iter()
            .any(|error| error.error_type == "settlement_mismatch")
        {
            SETTLEMENT_MISMATCH_MESSAGE
        } else {
            "日報ファイルの解析に失敗しました"
        };
        log_parse_failure(conn, message);
        return Err(BizError::ImportError(message.to_string()));
    }

    let report_date = parse_result.report_date.ok_or_else(|| {
        log_parse_failure(conn, "日報ファイルの対象日を抽出できません");
        BizError::ImportError("日報ファイルの対象日を抽出できません".to_string())
    })?;
    let parsed_date =
        chrono::NaiveDate::parse_from_str(&report_date, "%Y-%m-%d").map_err(|_| {
            BizError::ImportError(format!("日報対象日が不正な日付です: {}", report_date))
        })?;
    if parsed_date > chrono::Local::now().date_naive() {
        return Err(BizError::ImportError(format!(
            "日報対象日が未来の日付です: {}",
            report_date
        )));
    }
    // BIZ-08-D6: SD の経路は精算回数のある束だけ
    let settlement_no = parse_result.settlement_no;
    if !sd_files.is_empty() && settlement_no.is_none() {
        log_parse_failure(conn, SD_WITHOUT_SETTLEMENT_MESSAGE);
        return Err(BizError::ImportError(
            SD_WITHOUT_SETTLEMENT_MESSAGE.to_string(),
        ));
    }

    let bundle_hash = build_bundle_hash(&parse_result.source_files);

    let gross_amount = parse_result
        .summary_lines
        .iter()
        .find(|line| line.line_key == "gross_sales")
        .and_then(|line| line.amount);
    let net_amount = parse_result
        .summary_lines
        .iter()
        .find(|line| line.line_key == "net_sales")
        .and_then(|line| line.amount);
    if gross_amount.is_none() && net_amount.is_none() {
        return Err(BizError::ImportError(
            "必須サマリ（総売上または純売上）を導出できません".to_string(),
        ));
    }

    let departments = product_repo::list_departments(conn)?;
    let department_ids_by_name: HashMap<String, i64> = departments
        .into_iter()
        .map(|department| (department.name, department.id))
        .collect();
    let mut warnings = Vec::new();
    let department_lines: Vec<CachedDailyReportDepartmentLine> = parse_result
        .department_lines
        .iter()
        .map(|line| {
            let department_id = line
                .normalized_department_name
                .as_ref()
                .and_then(|name| department_ids_by_name.get(name).copied());
            if department_id.is_none() {
                warnings.push(DailyReportWarning {
                    code: "unmatched_department".to_string(),
                    message: format!("部門マスタに一致しません: {}", line.raw_department_name),
                    source_file: Some(DailyReportSourceKind::Z005),
                    line_no: None,
                });
            }
            CachedDailyReportDepartmentLine {
                department_id,
                raw_department_name: line.raw_department_name.clone(),
                normalized_department_name: line.normalized_department_name.clone(),
                amount: line.amount,
                quantity_hundredths: line.quantity_hundredths,
                count: line.count,
                sort_order: line.sort_order,
            }
        })
        .collect();
    let department_summary: Vec<DailyReportDepartmentLinePreview> = department_lines
        .iter()
        .map(|line| DailyReportDepartmentLinePreview {
            department_id: line.department_id,
            raw_department_name: line.raw_department_name.clone(),
            normalized_department_name: line.normalized_department_name.clone(),
            amount: line.amount,
            quantity: line
                .quantity_hundredths
                .map(daily_report_parser::quantity_hundredths_to_units),
            count: line.count,
            sort_order: line.sort_order,
        })
        .collect();

    let blocking = sales_repo::find_blocking_daily_report_by_bundle_hash(conn, &bundle_hash)?;
    // BIZ-08-D4: 同じ精算を別の bytes で取り込まない（追加確認で通さない）
    if let (None, Some(settlement_no)) = (&blocking, settlement_no) {
        if sales_repo::find_same_settlement_daily_report_import(
            conn,
            &report_date,
            settlement_no,
            &bundle_hash,
        )?
        .is_some()
        {
            log_parse_failure(conn, SAME_SETTLEMENT_MESSAGE);
            return Err(BizError::ImportError(SAME_SETTLEMENT_MESSAGE.to_string()));
        }
    }
    let same_date = sales_repo::find_daily_report_imports_by_report_date(conn, &report_date)?;
    let same_date_imports = same_date
        .iter()
        .map(|item| {
            Ok(SameDateDailyReportImportSummary {
                id: item.id,
                source_filenames: source_filenames(&item.source_files_json)?,
                gross_amount: item.gross_amount,
                net_amount: item.net_amount,
                imported_at: item.imported_at.clone(),
            })
        })
        .collect::<Result<Vec<_>, BizError>>()?;
    let status = if blocking.is_some() {
        DailyReportDuplicateStatus::AlreadyImported
    } else if same_date_imports.is_empty() {
        DailyReportDuplicateStatus::NoDuplicate
    } else {
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired
    };
    let duplicate_check = DailyReportDuplicateCheck {
        status,
        same_date_imports,
    };

    let source_files: Vec<DailyReportSourceFileInfo> = parse_result
        .source_files
        .iter()
        .map(|source| DailyReportSourceFileInfo {
            source: source.source,
            filename: source.filename.clone(),
            file_hash: source.file_hash.clone(),
            size_bytes: source.size_bytes,
        })
        .collect();
    let summary_lines: Vec<CachedDailyReportSummaryLine> = parse_result
        .summary_lines
        .iter()
        .map(|line| CachedDailyReportSummaryLine {
            line_key: line.line_key.clone(),
            label: line.label.clone(),
            amount: line.amount,
            quantity_hundredths: line.quantity_hundredths,
            count: line.count,
            sort_order: line.sort_order,
        })
        .collect();
    let payment_summary: Vec<DailyReportPaymentLinePreview> = parse_result
        .payment_lines
        .iter()
        .map(|line| DailyReportPaymentLinePreview {
            payment_key: line.payment_key.clone(),
            label: line.label.clone(),
            amount: line.amount,
            count: line.count,
            sort_order: line.sort_order,
        })
        .collect();

    let preview_data = DailyReportPreviewData {
        file_info: DailyReportFileInfo {
            report_date,
            bundle_hash,
            source_files,
        },
        totals: DailyReportTotals {
            gross_amount,
            net_amount,
        },
        payment_summary: payment_summary.clone(),
        department_summary,
        warnings: warnings.clone(),
        duplicate_check,
        preview_created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    };
    // 手順 9: SD の file だけを IO-07 が判定した系列と組にして持つ
    let sd_source_files = sd_files
        .into_iter()
        .filter_map(|(filename, sd_relative_path, bytes)| {
            let source = parse_result
                .source_files
                .iter()
                .find(|parsed| parsed.filename == filename)?
                .source;
            Some(SdSourceFileForCopy {
                source,
                sd_relative_path,
                bytes,
            })
        })
        .collect();
    let cached_preview = CachedDailyReportPreview {
        created_at: Instant::now(),
        preview_data: preview_data.clone(),
        settlement_no,
        sd_source_files,
        summary_lines,
        payment_lines: payment_summary,
        department_lines,
        active_same_date_import_ids: preview_data
            .duplicate_check
            .same_date_imports
            .iter()
            .map(|item| item.id)
            .collect(),
    };

    Ok(DailyReportParseValidateResult {
        preview_data,
        cached_preview,
    })
}

/// 手順 1a: 手で選んだ file が SD の上かを決める（preview の時の 1 回だけ）
fn resolve_selected_file_origins(
    files: &mut [DailyReportInputFile],
    find_sd_roots: impl FnOnce() -> Result<Vec<RegisterSdRoot>, RegisterSdError>,
) -> Result<(), BizError> {
    if !files
        .iter()
        .any(|file| file.sd_relative_path.is_none() && file.source_path.is_some())
    {
        return Ok(());
    }
    let roots = find_sd_roots().map_err(|error| {
        tracing::warn!(error = ?error, "SD の root を確かめられませんでした");
        BizError::ImportError(SD_LOOKUP_FAILED_MESSAGE.to_string())
    })?;
    for file in files.iter_mut() {
        let (None, Some(source_path)) = (&file.sd_relative_path, &file.source_path) else {
            continue;
        };
        match register_sd::locate_in_register_sd_roots(&roots, source_path) {
            Some(located) => file.sd_relative_path = Some(located.relative_path),
            None => register_sd::check_selected_file_present(source_path).map_err(|error| {
                tracing::warn!(error = ?error, "選んだファイルが見つかりません");
                BizError::ImportError(SELECTED_FILE_MISSING_MESSAGE.to_string())
            })?,
        }
    }
    Ok(())
}

fn source_filenames(source_files_json: &str) -> Result<Vec<String>, BizError> {
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

pub(super) fn build_bundle_hash(
    sources: &[daily_report_parser::ParsedDailyReportSourceFile],
) -> String {
    let mut parts: Vec<_> = sources.iter().collect();
    parts.sort_by_key(|source| source_kind_order(source.source));
    let joined = parts
        .iter()
        .map(|source| {
            format!(
                "{:?}:{}:{}",
                source.source, source.file_hash, source.size_bytes
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(super) fn log_parse_failure(conn: &DbConnection, message: &str) {
    let log = NewOperationLog {
        operation_type: "daily_report_parse_failed".to_string(),
        summary: message.to_string(),
        detail_json: None,
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }
}

pub(super) fn log_parse_diagnostics(errors: &[daily_report_parser::DailyReportParseError]) {
    for error in errors {
        tracing::warn!(
            source_file = ?error.source_file,
            filename = ?error.filename,
            line_no = ?error.line_no,
            error_type = %error.error_type,
            error_message = %error.error_message,
            "日報ファイルparse error"
        );
    }
}
