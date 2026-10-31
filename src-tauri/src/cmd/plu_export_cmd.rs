//! CMD-08: PLU書出しコマンド群
//!
//! docs/function-design/41-cmd-pos.md §17.6 に基づく実装。

use crate::biz::plu_export_service::{
    self, ExportMode, PluExcludedReason, PluExportConfirmRequest, PluExportPrepareRequest,
    PluPreparedRow, PluRegisterSnapshotSummary,
};
use crate::biz::ProductStockUnit;
use crate::cmd::CmdErrorKind;
use crate::cmd::{AppState, CmdError};
use crate::constants::CSV_IMPORT_FILE_SIZE_LIMIT;
use base64::{engine::general_purpose, Engine as _};
use tauri::State;

// ---------------------------------------------------------------------------
// レスポンス型
// ---------------------------------------------------------------------------

/// PLUファイル生成レスポンス（フロントエンド返却用）
///
/// PluFileOutput の bytes を base64 エンコードして返す。
/// フロントエンド側で base64デコード → native save dialog の保存先へ書き込む。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct PluExportPrepareResponse {
    /// CP932バイト列のbase64エンコード
    pub bytes_base64: String,
    /// 推奨ファイル名（例: "PLU_20260408.txt"）
    pub suggested_filename: String,
    /// MIMEタイプ
    pub content_type: String,
    /// 文字エンコーディング名
    pub encoding: String,
    /// 書出し件数
    pub count: usize,
    /// PLUファイルに含めた商品コード一覧
    pub target_product_codes: Vec<String>,
    pub prepared_rows: Vec<PluPreparedRow>,
    /// PLUファイルに含めなかった商品一覧
    pub excluded: Vec<PluExcludedProductResponse>,
    /// PLU上限超過警告（互換維持フィールド）
    pub over_limit_warning: bool,
}

#[derive(Debug, serde::Serialize, specta::Type)]
pub struct PluExcludedProductResponse {
    pub product_code: String,
    pub jan_code: Option<String>,
    pub name: String,
    pub memory_no: Option<i64>,
    pub reason: String,
}

/// PLU保存済み確認レスポンス（フロントエンド返却用）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct PluExportConfirmResponse {
    pub updated_count: usize,
    pub confirmed_at: String,
}

/// PLU dirty 商品レスポンス（フロントエンド返却用）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ProductResponse {
    pub product_code: String,
    pub jan_code: Option<String>,
    pub name: String,
    pub department_id: i64,
    pub selling_price: i64,
    pub cost_price_centi: i64,
    /// 商品の単位。`stock_quantity` は在庫の数量（長さは cm）で、表示は単位で決める（SPEC-UNIT-D3 の表の 9）
    pub stock_unit: ProductStockUnit,
    pub stock_quantity: i64,
    pub plu_dirty: bool,
    pub plu_exported_at: Option<String>,
}

// ---------------------------------------------------------------------------
// コマンド
// ---------------------------------------------------------------------------

/// PLUファイルを生成して返す。DB状態は更新しない。
///
/// docs/function-design/41-cmd-pos.md §17.6 prepare_plu_export
#[tauri::command]
#[specta::specta]
pub fn prepare_plu_export(
    state: State<AppState>,
    mode: ExportMode,
) -> Result<PluExportPrepareResponse, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let req = PluExportPrepareRequest { mode };
    let result = plu_export_service::prepare_plu_export(&mut conn, req).map_err(CmdError::from)?;

    Ok(PluExportPrepareResponse {
        bytes_base64: general_purpose::STANDARD.encode(&result.plu_output.bytes),
        suggested_filename: result.plu_output.suggested_filename,
        content_type: result.plu_output.content_type.to_string(),
        encoding: result.plu_output.encoding.to_string(),
        count: result.count,
        target_product_codes: result.target_product_codes,
        prepared_rows: result.prepared_rows,
        excluded: result
            .excluded
            .into_iter()
            .map(|excluded| PluExcludedProductResponse {
                product_code: excluded.product_code,
                jan_code: excluded.jan_code,
                name: excluded.name,
                memory_no: excluded.memory_no,
                reason: excluded_reason_to_snake_case(&excluded.reason).to_string(),
            })
            .collect(),
        over_limit_warning: result.over_limit_warning,
    })
}

/// PLUファイル保存済み確認を受け、対象商品を未反映から外す
///
/// docs/function-design/41-cmd-pos.md §17.6 confirm_plu_export_saved
#[tauri::command]
#[specta::specta]
pub fn confirm_plu_export_saved(
    state: State<AppState>,
    product_codes: Vec<String>,
    prepared_rows: Vec<PluPreparedRow>,
) -> Result<PluExportConfirmResponse, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let req = PluExportConfirmRequest {
        product_codes,
        prepared_rows,
    };
    let result =
        plu_export_service::confirm_plu_export_saved(&mut conn, req).map_err(CmdError::from)?;

    Ok(PluExportConfirmResponse {
        updated_count: result.updated_count,
        confirmed_at: result.confirmed_at,
    })
}

#[tauri::command]
#[specta::specta]
pub fn import_plu_register_snapshot(
    state: State<AppState>,
    file_bytes: Vec<u8>,
) -> Result<PluRegisterSnapshotSummary, CmdError> {
    if file_bytes.len() > CSV_IMPORT_FILE_SIZE_LIMIT {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "ファイルサイズが上限(20MB)を超えています".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    plu_export_service::import_plu_register_snapshot(&mut conn, &file_bytes).map_err(CmdError::from)
}

#[tauri::command]
#[specta::specta]
pub fn get_plu_slot_summary(
    state: State<AppState>,
) -> Result<PluRegisterSnapshotSummary, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    plu_export_service::get_plu_slot_summary(&conn).map_err(CmdError::from)
}

/// PLU書出しが必要な商品一覧を返す
///
/// docs/function-design/41-cmd-pos.md §17.6 list_plu_dirty
#[tauri::command]
#[specta::specta]
pub fn list_plu_dirty(state: State<AppState>) -> Result<Vec<ProductResponse>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let products = plu_export_service::list_plu_dirty(&conn).map_err(CmdError::from)?;

    // Product → ProductResponse 変換
    let responses = products
        .into_iter()
        .map(|p| ProductResponse {
            product_code: p.product_code,
            jan_code: p.jan_code,
            name: p.name,
            department_id: p.department_id,
            selling_price: p.selling_price,
            cost_price_centi: p.cost_price_centi,
            stock_unit: p.stock_unit,
            stock_quantity: p.stock_quantity,
            plu_dirty: p.plu_dirty,
            plu_exported_at: p.plu_exported_at,
        })
        .collect();

    Ok(responses)
}

fn excluded_reason_to_snake_case(reason: &PluExcludedReason) -> &'static str {
    match reason {
        PluExcludedReason::MissingJan => "missing_jan",
        PluExcludedReason::InvalidJanFormat => "invalid_jan_format",
        PluExcludedReason::InvalidCheckDigit => "invalid_check_digit",
        PluExcludedReason::GroupPriceMismatch => "group_price_mismatch",
        PluExcludedReason::NoFreeSlot => "no_free_slot",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::setup_test_db;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tauri::Manager;

    #[test]
    fn test_import_plu_register_snapshot_req907_rejects_oversize_bytes() {
        // REQ-907: A-N1b CMD boundary uses the shared 20MB guard.
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(AppState {
                db: Mutex::new(conn),
                preview_cache: Mutex::new(HashMap::new()),
                daily_report_preview_cache: Mutex::new(HashMap::new()),
                register_sd_scan_cache: Mutex::new(HashMap::new()),
            })
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let error = import_plu_register_snapshot(
            app.state::<AppState>(),
            vec![0; crate::constants::CSV_IMPORT_FILE_SIZE_LIMIT + 1],
        )
        .unwrap_err();
        assert_eq!(error.kind, CmdErrorKind::Validation);
        assert_eq!(error.message, "ファイルサイズが上限(20MB)を超えています");
    }
}
