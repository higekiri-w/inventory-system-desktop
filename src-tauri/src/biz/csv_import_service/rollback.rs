//! rollback_csv_import — CSV取込みのロールバック（論理無効化 + 求め直し）と、取消の前の読取り

use crate::biz::csv_import_service::{RollbackPreview, RollbackResult};
use crate::biz::stock_ledger::recompute_for_settlement_change;
use crate::biz::stocktake_service::{completed_stocktakes_comparing_settlement, recheck_targets};
use crate::biz::BizError;
use crate::db::sales_repo;
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::{DbConnection, DbError};

fn not_found(csv_import_id: i64) -> BizError {
    BizError::NotFound(format!(
        "CSV取込み記録が見つかりません: ID {}",
        csv_import_id
    ))
}

/// 取消の本体（TX の中）: 売上と movement の無効化、取込みの状態、求め直しと要再確認の一覧。
/// 取消と取消の前の読取りが同じこの関数を呼ぶ（32 BIZ-03-D2。規則を 2 つ持たない）
pub(crate) fn apply_rollback(
    tx: &rusqlite::Transaction<'_>,
    csv_import_id: i64,
) -> Result<RollbackResult, BizError> {
    let voided_sale_count = sales_repo::void_sale_records_by_import(tx, csv_import_id)?;
    let voided_movement_count =
        sales_repo::void_movements_by_reference(tx, "csv_import", csv_import_id)?.len();
    sales_repo::update_csv_import_status(
        tx,
        csv_import_id,
        sales_repo::CsvImportStatus::RolledBack.as_str(),
    )?;

    // 精算の一覧が変わるので、計数を持つ全ての商品とこの取込みの商品を求め直す（ADR D2）
    let codes: Vec<String> = recompute_for_settlement_change(tx, csv_import_id)?
        .into_iter()
        .map(|(code, _)| code)
        .collect();
    Ok(RollbackResult {
        success: true,
        voided_sale_count,
        voided_movement_count,
        recheck_targets: recheck_targets(tx, &codes)?,
    })
}

/// 取消の前の読取り: 取り消した後の要再確認・取り込み直す精算・名指す完了した棚卸しを返し、何も書かない
///
/// docs/function-design/32-biz-csv-import-service.md §15.6b（BIZ-03-D2）
pub fn preview_rollback_csv_import(
    conn: &mut DbConnection,
    csv_import_id: i64,
) -> Result<RollbackPreview, BizError> {
    // 1. 対象確認（無い・取消済みは TX を開かずに拒否）
    let import = sales_repo::find_csv_import_by_id(conn, csv_import_id)?
        .ok_or_else(|| not_found(csv_import_id))?;
    if import.status == sales_repo::CsvImportStatus::RolledBack {
        return Err(BizError::ValidationFailed(
            "この取込みは取り消し済みです".to_string(),
        ));
    }

    // 2. 精算回数（active な Z004 の精算の一覧から）
    let settlement = sales_repo::list_active_settlements(conn)?
        .into_iter()
        .find(|s| s.import_id == csv_import_id)
        .ok_or_else(|| not_found(csv_import_id))?;

    // 3. この取込みの精算が完了の後の比較の範囲に入る完了した棚卸し（35 の判定を BIZ-06 で）
    let affected_stocktakes = completed_stocktakes_comparing_settlement(conn, &settlement)?;

    // 4〜5. 取消の本体を TX の中で当て、手順 4 の成否によらず明示に閉じる（drop に頼らない。操作ログは書かない）。
    // finish は既定の DropBehavior::Rollback で ROLLBACK を出し、SQLite が自動で戻した TX では何もしない。
    // 閉じる失敗を先に返し、閉じられたら手順 4 の失敗（真因）をそのまま返す
    let tx = conn.transaction().map_err(DbError::from)?;
    let applied = apply_rollback(&tx, csv_import_id);
    tx.finish().map_err(DbError::from)?;
    let recheck_targets = applied?.recheck_targets;

    Ok(RollbackPreview {
        csv_import_id,
        settlement_no: settlement.settlement_no,
        recheck_targets,
        affected_stocktakes,
    })
}

/// 指定 csv_import の売上と movement を無効にし、求め直す（補償の movement は作らない。計数は消さない。ADR D7）
///
/// 冪等（既に rolled_back なら何もせず成功を返す）。
///
/// docs/function-design/32-biz-csv-import-service.md §15.5 と「計数の置き場と在庫の求め直しの契約」の取消
pub fn rollback_csv_import(
    conn: &mut DbConnection,
    csv_import_id: i64,
) -> Result<RollbackResult, BizError> {
    // 1. 対象確認
    let import = sales_repo::find_csv_import_by_id(conn, csv_import_id)?
        .ok_or_else(|| not_found(csv_import_id))?;

    // 冪等: 既に rolled_back なら何もせず成功
    if import.status == sales_repo::CsvImportStatus::RolledBack {
        return Ok(RollbackResult {
            success: true,
            voided_sale_count: 0,
            voided_movement_count: 0,
            recheck_targets: Vec::new(),
        });
    }

    // 2〜3. 取消の本体（無効化・状態・求め直し・要再確認の一覧）
    let tx = conn.transaction().map_err(DbError::from)?;
    let result = apply_rollback(&tx, csv_import_id)?;
    tx.commit().map_err(DbError::from)?;

    // 4. TX外: 操作ログ記録（best-effort）
    let detail = serde_json::json!({
        "csv_import_id": csv_import_id,
        "voided_sale_count": result.voided_sale_count,
        "voided_movement_count": result.voided_movement_count,
    })
    .to_string();
    let log = NewOperationLog {
        operation_type: "csv_rollback".to_string(),
        summary: format!("CSV取込みを取消しました: ID {}", csv_import_id),
        detail_json: Some(detail),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }

    Ok(result)
}
