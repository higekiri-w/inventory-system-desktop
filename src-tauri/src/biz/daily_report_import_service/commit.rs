use crate::biz::daily_report_import_service::{
    source_kind_db_value, CachedDailyReportPreview, DailyReportDuplicateStatus,
    DailyReportImportResult, SAME_SETTLEMENT_MESSAGE, SD_WITHOUT_SETTLEMENT_MESSAGE,
};
use crate::biz::BizError;
use crate::constants;
use crate::db::sales_repo::{
    self, NewDailyReportDepartmentLine, NewDailyReportImport, NewDailyReportPaymentLine,
    NewDailyReportSummaryLine,
};
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::{DbConnection, DbError};
use crate::io::daily_report_parser::DailyReportSourceKind;
use crate::io::pos_source_copy;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

/// `source_files_json` の 1 要素（内部の serialize 用。既存の 4 つの key は wire 型と同じ名前と形）
#[derive(Serialize)]
struct StoredSourceFile<'a> {
    source: DailyReportSourceKind,
    filename: &'a str,
    file_hash: &'a str,
    size_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    sd_relative_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    copy_path: Option<String>,
}

pub fn commit_daily_report_import(
    conn: &mut DbConnection,
    cached_preview: CachedDailyReportPreview,
    additional_import_confirmed: bool,
    app_data_dir: &Path,
) -> Result<DailyReportImportResult, BizError> {
    if cached_preview.created_at.elapsed().as_secs() > constants::PREVIEW_CACHE_TTL_SECS {
        return Err(BizError::ImportError(
            "プレビューの有効期限が切れています".to_string(),
        ));
    }
    match cached_preview.preview_data.duplicate_check.status {
        DailyReportDuplicateStatus::AlreadyImported => {
            return Err(BizError::IdempotencyConflict(
                "この日報bundleは取込み済みです".to_string(),
            ));
        }
        DailyReportDuplicateStatus::NoDuplicate => {
            if additional_import_confirmed {
                return Err(BizError::ValidationFailed(
                    "追加確認の指定がプレビュー結果と一致しません".to_string(),
                ));
            }
        }
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired => {
            if !additional_import_confirmed {
                return Err(BizError::ValidationFailed(
                    "追加確認の指定がプレビュー結果と一致しません".to_string(),
                ));
            }
        }
    }

    // 手順 2a（BIZ-08-D6）: SD の経路で精算回数の無い cache は、写しも DB も変えずに止める
    if !cached_preview.sd_source_files.is_empty() && cached_preview.settlement_no.is_none() {
        return Err(BizError::ImportError(
            SD_WITHOUT_SETTLEMENT_MESSAGE.to_string(),
        ));
    }
    // 手順 2b（BIZ-08-D5）: TX の前に写しを書く。1 本でも失敗したら DB を変えずに止める
    let mut copy_paths: HashMap<DailyReportSourceKind, String> = HashMap::new();
    for file in &cached_preview.sd_source_files {
        let copied = pos_source_copy::save_pos_source_copy(
            app_data_dir,
            &file.sd_relative_path,
            &file.bytes,
        )
        .map_err(|error| {
            tracing::warn!(kind = ?error.kind(), error = %error, "SD の写しを保存できませんでした");
            BizError::SourceCopyFailed(error.kind())
        })?;
        copy_paths.insert(file.source, copied.relative_path);
    }

    let result = execute_commit(conn, &cached_preview, copy_paths);
    if result.is_err() {
        let log = NewOperationLog {
            operation_type: "daily_report_import_failed".to_string(),
            summary: "日報取込みに失敗しました".to_string(),
            detail_json: None,
        };
        if let Err(e) = system_repo::insert_operation_log(conn, &log) {
            tracing::warn!(error = %e, "操作ログ記録に失敗");
        }
    }
    result
}

fn execute_commit(
    conn: &mut DbConnection,
    cached_preview: &CachedDailyReportPreview,
    mut copy_paths: HashMap<DailyReportSourceKind, String>,
) -> Result<DailyReportImportResult, BizError> {
    let report_date = &cached_preview.preview_data.file_info.report_date;
    let bundle_hash = &cached_preview.preview_data.file_info.bundle_hash;
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    if sales_repo::find_blocking_daily_report_by_bundle_hash(&tx, bundle_hash)?.is_some() {
        return Err(BizError::IdempotencyConflict(
            "この日報bundleは取込み済みです".to_string(),
        ));
    }
    // 手順 4a（BIZ-08-D4）の TX 内の再検査
    if let Some(settlement_no) = cached_preview.settlement_no {
        if sales_repo::find_same_settlement_daily_report_import(
            &tx,
            report_date,
            settlement_no,
            bundle_hash,
        )?
        .is_some()
        {
            return Err(BizError::ImportError(SAME_SETTLEMENT_MESSAGE.to_string()));
        }
    }
    let same_date = sales_repo::find_daily_report_imports_by_report_date(&tx, report_date)?;
    let current_ids: Vec<_> = same_date.iter().map(|item| item.id).collect();
    if current_ids != cached_preview.active_same_date_import_ids {
        return Err(BizError::ImportError(
            "同日の取込み状況が変わりました。再度プレビューしてください".to_string(),
        ));
    }

    // 手順 6: SD の file には sd_relative_path と copy_path を足す（PC 上の file は持たない）
    let stored: Vec<StoredSourceFile> = cached_preview
        .preview_data
        .file_info
        .source_files
        .iter()
        .map(|file| StoredSourceFile {
            source: file.source,
            filename: &file.filename,
            file_hash: &file.file_hash,
            size_bytes: file.size_bytes,
            sd_relative_path: cached_preview
                .sd_source_files
                .iter()
                .find(|sd| sd.source == file.source)
                .map(|sd| sd.sd_relative_path.as_str()),
            copy_path: copy_paths.remove(&file.source),
        })
        .collect();
    let source_files_json =
        serde_json::to_string(&stored).map_err(|e| BizError::ImportError(e.to_string()))?;
    let note = if cached_preview.preview_data.warnings.is_empty() {
        None
    } else {
        Some(
            cached_preview
                .preview_data
                .warnings
                .iter()
                .map(|warning| warning.message.clone())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    };

    let import_id = sales_repo::insert_daily_report_import(
        &tx,
        &NewDailyReportImport {
            report_date: report_date.clone(),
            source_adapter: "casio_sr_s4000".to_string(),
            bundle_hash: bundle_hash.clone(),
            source_files_json,
            gross_amount: cached_preview.preview_data.totals.gross_amount,
            net_amount: cached_preview.preview_data.totals.net_amount,
            status: "completed".to_string(),
            note,
            settlement_no: cached_preview.settlement_no,
        },
    )?;
    let summary_rows: Vec<NewDailyReportSummaryLine> = cached_preview
        .summary_lines
        .iter()
        .map(|line| NewDailyReportSummaryLine {
            daily_report_import_id: import_id,
            source_file: source_kind_db_value(
                crate::io::daily_report_parser::DailyReportSourceKind::Z001,
            )
            .to_string(),
            line_key: line.line_key.clone(),
            label: line.label.clone(),
            amount: line.amount,
            quantity_hundredths: line.quantity_hundredths,
            count: line.count,
            sort_order: line.sort_order,
        })
        .collect();
    sales_repo::insert_daily_report_summary_lines(&tx, &summary_rows)?;

    let payment_rows: Vec<NewDailyReportPaymentLine> = cached_preview
        .payment_lines
        .iter()
        .map(|line| NewDailyReportPaymentLine {
            daily_report_import_id: import_id,
            source_file: source_kind_db_value(
                crate::io::daily_report_parser::DailyReportSourceKind::Z002,
            )
            .to_string(),
            payment_key: line.payment_key.clone(),
            label: line.label.clone(),
            amount: line.amount,
            count: line.count,
            sort_order: line.sort_order,
        })
        .collect();
    sales_repo::insert_daily_report_payment_lines(&tx, &payment_rows)?;

    let department_rows: Vec<NewDailyReportDepartmentLine> = cached_preview
        .department_lines
        .iter()
        .map(|line| NewDailyReportDepartmentLine {
            daily_report_import_id: import_id,
            source_file: source_kind_db_value(
                crate::io::daily_report_parser::DailyReportSourceKind::Z005,
            )
            .to_string(),
            department_id: line.department_id,
            raw_department_name: line.raw_department_name.clone(),
            normalized_department_name: line.normalized_department_name.clone(),
            amount: line.amount,
            quantity_hundredths: line.quantity_hundredths,
            count: line.count,
            sort_order: line.sort_order,
        })
        .collect();
    sales_repo::insert_daily_report_department_lines(&tx, &department_rows)?;

    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    let log = NewOperationLog {
        operation_type: "daily_report_import".to_string(),
        summary: format!("日報取込み完了: {}", report_date),
        detail_json: Some(
            serde_json::json!({
                "daily_report_import_id": import_id,
                "report_date": report_date,
                "bundle_hash": bundle_hash,
            })
            .to_string(),
        ),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }

    Ok(DailyReportImportResult {
        daily_report_import_id: import_id,
        status: "completed".to_string(),
        report_date: report_date.clone(),
        gross_amount: cached_preview.preview_data.totals.gross_amount,
        net_amount: cached_preview.preview_data.totals.net_amount,
        warning_count: cached_preview.preview_data.warnings.len() as i64,
    })
}
