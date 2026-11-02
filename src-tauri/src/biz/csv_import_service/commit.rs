//! commit_csv_import — Stage 4（Commit）
//!
//! 32「計数の置き場と在庫の求め直しの契約」の確定: TX の中で順 1〜5 と JAN の候補と連動の設定を取り直し、
//! 精算の鍵つきで取込み・売上（`stock_linked`）・在庫連動の行の movement（`phase = 1`）を書き、
//! 計数を持つ全ての商品とその取込みの商品を求め直す。

use crate::biz::csv_import_service::parse::{
    check_settlement_rules, settlement_gaps, settlement_identity,
};
use crate::biz::csv_import_service::{
    CommitRequest, CsvImportErrorType, ErrorRow, ImportResult, MatchedRow,
};
use crate::biz::stock_ledger::recompute_for_settlement_change;
use crate::biz::stocktake_service::recheck_targets;
use crate::biz::BizError;
use crate::db::inventory_repo::{self, MovementType, NewMovement, ReferenceType};
use crate::db::product_repo;
use crate::db::sales_repo::{self, NewCsvImport, NewCsvImportError, NewSaleRecord};
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::{DbConnection, DbError};
use crate::io::z004_parser::SettlementMetadata;
use std::collections::BTreeMap;

/// プレビュー済みデータをDBに書き込む（CMD層から受け取ったキャッシュデータを使いTX内で一括実行する）
///
/// docs/function-design/32-biz-csv-import-service.md §15.4 と「計数の置き場と在庫の求め直しの契約」の確定
pub fn commit_csv_import(
    conn: &mut DbConnection,
    req: CommitRequest,
) -> Result<ImportResult, BizError> {
    let result = execute_commit(conn, &req);

    // TX失敗時: csv_import_failed ログを記録（設計書§15.4）
    if let Err(ref e) = result {
        let fail_log = NewOperationLog {
            operation_type: "csv_import_failed".to_string(),
            summary: format!("CSV取込みに失敗しました: {}", e),
            detail_json: None,
        };
        if let Err(log_err) = system_repo::insert_operation_log(conn, &fail_log) {
            tracing::warn!(error = %log_err, "操作ログ記録に失敗");
        }
    }

    result
}

fn registration_changed() -> BizError {
    BizError::ImportError(
        "商品の登録が読み込みの後に変わりました。ファイルを選び直してください".to_string(),
    )
}

/// 確定の TX の照合（32「preview の cache の契約」の (4)）。preview の引き当てと違えば書く前に拒む。
/// 在庫連動は確定の時点の設定（共有 JAN の行は 0）を返す
fn recheck_matches(
    conn: &DbConnection,
    matched_rows: &[MatchedRow],
    error_rows: &[ErrorRow],
) -> Result<Vec<bool>, BizError> {
    let mut stock_linked = Vec::with_capacity(matched_rows.len());
    for row in matched_rows {
        let candidates = product_repo::find_by_jan_code(conn, &row.normalized_jan)?;
        let head = candidates.first().ok_or_else(registration_changed)?;
        let shared = candidates.len() > 1;
        if head.product_code != row.product_code || shared != row.shared_jan {
            return Err(registration_changed());
        }
        stock_linked.push(!shared && head.pos_stock_sync);
    }
    for row in error_rows {
        if row.error_type != CsvImportErrorType::UnmatchedProduct {
            continue;
        }
        if let Some(jan) = &row.normalized_jan {
            if !product_repo::find_by_jan_code(conn, jan)?.is_empty() {
                return Err(registration_changed());
            }
        }
    }
    Ok(stock_linked)
}

/// TX処理本体
fn execute_commit(conn: &mut DbConnection, req: &CommitRequest) -> Result<ImportResult, BizError> {
    let cached = &req.cached_data;
    let file_info = &cached.preview_data.file_info;
    let settlement = &cached.settlement;

    let tx = conn.transaction().map_err(DbError::from)?;

    // 1. 順 1〜5 を cache の精算の識別メタで取り直す（file を読み直さない）
    settlement_identity(
        Some(&SettlementMetadata {
            machine_no: Some(settlement.machine_no.clone()),
            report_kind: None,
            settlement_no: Some(settlement.settlement_no.to_string()),
            settled_at: None,
        }),
        &settlement.settlement_date,
    )?;
    check_settlement_rules(&tx, settlement, &file_info.file_hash, true)?;

    // 2. JAN の候補と在庫連動の設定を取り直す
    let stock_linked = recheck_matches(&tx, &cached.matched_rows, &cached.error_rows)?;

    // 3. csv_imports（精算の鍵つき）
    let import_id = sales_repo::insert_csv_import(
        &tx,
        &NewCsvImport {
            filename: file_info.filename.clone(),
            settlement_date: settlement.settlement_date.clone(),
            file_hash: file_info.file_hash.clone(),
            total_items: 0,
            total_amount: 0,
            skipped_count: 0,
            status: "completed".to_string(),
            machine_no: settlement.machine_no.clone(),
            settlement_no: settlement.settlement_no,
        },
    )?;

    // 4. 売上と在庫連動の行の movement（仮の stock_after で書き、求め直しの後に写しの値へ）
    let mut written: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for (row, linked) in cached.matched_rows.iter().zip(&stock_linked) {
        sales_repo::insert_sale_record(
            &tx,
            &NewSaleRecord {
                csv_import_id: Some(import_id),
                product_code: row.product_code.clone(),
                sale_date: settlement.settlement_date.clone(),
                quantity: row.quantity,
                amount: i64::from(row.amount),
                source: "auto".to_string(),
                source_line_no: Some(row.line_no as i64),
                reason: None,
                note: None,
                stock_linked: *linked,
            },
        )?;
        // 数量 0 で金額が 0 でない行は売上だけ
        if *linked && row.quantity != 0 {
            let current = product_repo::find_by_product_code(&tx, &row.product_code)?
                .ok_or_else(registration_changed)?
                .product
                .stock_quantity;
            // INV-1: 売上帳票視点→在庫視点。常に符号反転
            let movement_id = inventory_repo::insert_movement(
                &tx,
                &NewMovement {
                    product_code: row.product_code.clone(),
                    movement_type: MovementType::SaleAuto,
                    quantity: -row.quantity,
                    stock_after: current,
                    reference_type: Some(ReferenceType::CsvImport),
                    reference_id: Some(import_id),
                    note: None,
                    business_date: settlement.settlement_date.clone(),
                    settlement_no: Some(settlement.settlement_no),
                },
            )?;
            written
                .entry(row.product_code.clone())
                .or_default()
                .push(movement_id);
        }
    }

    // 5. error_rows の記録
    if !cached.error_rows.is_empty() {
        let errors: Vec<NewCsvImportError> = cached
            .error_rows
            .iter()
            .map(|e| NewCsvImportError {
                csv_import_id: import_id,
                source_line_no: e.line_no as i64,
                normalized_jan: e.normalized_jan.clone(),
                raw_name: e.name.clone(),
                raw_quantity: e.raw_quantity.clone(),
                raw_amount: e.raw_amount.clone(),
                error_type: e.error_type.as_str().to_string(),
                error_message: e.error_message.clone(),
            })
            .collect();
        sales_repo::insert_csv_import_errors(&tx, &errors)?;
    }

    // 6. 集計値の確定
    let total_items = cached.matched_rows.len() as i64;
    let total_amount: i64 = cached
        .matched_rows
        .iter()
        .map(|r| i64::from(r.amount))
        .sum();
    let skipped_count = cached.error_rows.len() as i64;
    let status = if cached.error_rows.is_empty() {
        sales_repo::CsvImportStatus::Completed
    } else {
        sales_repo::CsvImportStatus::CompletedPartial
    };
    sales_repo::update_csv_import_totals(
        &tx,
        import_id,
        total_items,
        total_amount,
        skipped_count,
        status.as_str(),
    )?;

    // 7. 精算の一覧が変わるので、計数を持つ全ての商品とこの取込みの商品を求め直す（ADR D2）
    let recomputed = recompute_for_settlement_change(&tx, import_id)?;
    for (code, derived) in &recomputed {
        for movement_id in written.get(code).into_iter().flatten() {
            inventory_repo::update_movement_stock_after(&tx, *movement_id, derived.stock)?;
        }
    }
    let codes: Vec<String> = recomputed.into_iter().map(|(code, _)| code).collect();
    let targets = recheck_targets(&tx, &codes)?;
    let gaps = settlement_gaps(&tx, settlement.settlement_no)?;

    tx.commit().map_err(DbError::from)?;

    // 8. TX外: 操作ログ記録（best-effort）
    let detail = serde_json::json!({
        "import_id": import_id,
        "filename": file_info.filename,
        "settlement_date": settlement.settlement_date,
        "total_items": total_items,
        "total_amount": total_amount,
        "skipped_count": skipped_count,
        "status": status.as_str(),
    })
    .to_string();
    let log = NewOperationLog {
        operation_type: "csv_import".to_string(),
        summary: format!(
            "CSV取込み完了: {}（{}件, ¥{}）",
            file_info.filename, total_items, total_amount
        ),
        detail_json: Some(detail),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }

    Ok(ImportResult {
        csv_import_id: import_id,
        status,
        total_items,
        total_amount,
        skipped_count,
        settlement_gaps: gaps,
        settlement_date_after_today: settlement.settlement_date.as_str() > req.today.as_str(),
        recheck_targets: targets,
    })
}
