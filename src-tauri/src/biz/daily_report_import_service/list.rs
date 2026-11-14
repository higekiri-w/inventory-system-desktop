use crate::biz::daily_report_import_service::{
    source_filenames, DailyReportImportRow, ListDailyReportImportsQuery,
};
use crate::biz::BizError;
use crate::db::sales_repo;
use crate::db::{DbConnection, PaginatedResult};

pub fn list_daily_report_imports(
    conn: &DbConnection,
    query: ListDailyReportImportsQuery,
) -> Result<PaginatedResult<DailyReportImportRow>, BizError> {
    if query.page < 1 || query.per_page < 1 || query.per_page > 100 {
        return Err(BizError::ValidationFailed(
            "ページパラメータが不正です".to_string(),
        ));
    }
    if let Some(status) = &query.status {
        if !matches!(status.as_str(), "completed" | "rolled_back") {
            return Err(BizError::ValidationFailed(
                "statusパラメータが不正です".to_string(),
            ));
        }
    }

    let page = sales_repo::list_daily_report_imports(
        conn,
        query.page as u32,
        query.per_page as u32,
        query.date_from.as_deref(),
        query.date_to.as_deref(),
    )?;
    // 読めない source_files_json は一覧全体を ImportError にする（空の名前で黙って通さない。37 §37.6）
    let items = page
        .items
        .into_iter()
        .map(|item| {
            Ok(DailyReportImportRow {
                source_filenames: source_filenames(&item.source_files_json)?,
                id: item.id,
                report_date: item.report_date,
                settlement_no: item.settlement_no,
                gross_amount: item.gross_amount,
                net_amount: item.net_amount,
                status: item.status,
                imported_at: item.imported_at,
                rolled_back_at: item.rolled_back_at,
            })
        })
        .collect::<Result<Vec<_>, BizError>>()?;
    Ok(PaginatedResult {
        items,
        total_count: page.total_count,
        page: page.page,
        per_page: page.per_page,
    })
}
