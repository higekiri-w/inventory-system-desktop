//! CMD-01: 商品コマンド群
//!
//! docs/function-design/40-cmd-product.md §5.4 + 42-cmd-sales-stocktake.md §22.6 に基づく実装。

use crate::biz::product_service::{self, ImportRow};
use crate::biz::{
    Department, PaginatedResult, ProductBulkFilter, ProductSearchQuery, ProductWithRelations,
    Supplier,
};
use crate::cmd::{AppState, CmdError, CmdErrorKind};
use crate::constants;
use tauri::State;

// ---------------------------------------------------------------------------
// 商品CRUD（§5.4）
// ---------------------------------------------------------------------------

/// 商品を新規登録する
#[tauri::command]
#[specta::specta]
pub fn create_product(
    state: State<AppState>,
    req: product_service::ProductCreateRequest,
) -> Result<product_service::ProductCreateResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::create_product(&mut conn, req).map_err(CmdError::from)
}

/// 商品情報を更新する
#[tauri::command]
#[specta::specta]
pub fn update_product(
    state: State<AppState>,
    product_code: String,
    req: product_service::ProductUpdateRequest,
) -> Result<product_service::ProductUpdateResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::update_product(&mut conn, &product_code, &req).map_err(CmdError::from)
}

/// 商品価格を改定する。
#[tauri::command]
#[specta::specta]
pub fn revise_product_price(
    state: State<AppState>,
    input: product_service::PriceRevisionInput,
) -> Result<product_service::PriceRevisionResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::revise_product_price(&mut conn, input).map_err(CmdError::from)
}

/// 廃番状態を切り替える
///
/// 戻り値は is_discontinued の新しい値。
#[tauri::command]
#[specta::specta]
pub fn toggle_discontinue(state: State<AppState>, product_code: String) -> Result<bool, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::toggle_discontinue(&mut conn, &product_code).map_err(CmdError::from)
}

/// 商品を検索する（ページング対応）
#[tauri::command]
#[specta::specta]
pub fn search_products(
    state: State<AppState>,
    query: ProductSearchQuery,
) -> Result<PaginatedResult<ProductWithRelations>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::search_products(&conn, query).map_err(CmdError::from)
}

/// 現在の filter 一致全件の PLU 対象状態を更新する。
#[tauri::command]
#[specta::specta]
pub fn bulk_set_plu_target(
    state: State<AppState>,
    filter: ProductBulkFilter,
    plu_target: bool,
) -> Result<product_service::BulkPluTargetResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::bulk_set_plu_target(&mut conn, filter, plu_target).map_err(CmdError::from)
}

/// 部門選択候補を全件取得する
#[tauri::command]
#[specta::specta]
pub fn list_departments(state: State<AppState>) -> Result<Vec<Department>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::list_departments(&conn).map_err(CmdError::from)
}

/// 取引先選択候補を全件取得する
#[tauri::command]
#[specta::specta]
pub fn list_suppliers(state: State<AppState>) -> Result<Vec<Supplier>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::list_suppliers(&conn).map_err(CmdError::from)
}

/// 取引先を作成する（同名は既存行を返す）。
#[tauri::command]
#[specta::specta]
pub fn create_supplier(state: State<AppState>, name: String) -> Result<Supplier, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::create_supplier(&conn, name).map_err(CmdError::from)
}

/// 取引先名を変更する。
#[tauri::command]
#[specta::specta]
pub fn rename_supplier(
    state: State<AppState>,
    supplier_id: i64,
    name: String,
) -> Result<Supplier, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::rename_supplier(&mut conn, supplier_id, name).map_err(CmdError::from)
}

/// 重複した取引先を残す側へ統合する。
#[tauri::command]
#[specta::specta]
pub fn merge_suppliers(
    state: State<AppState>,
    source_id: i64,
    target_id: i64,
) -> Result<product_service::SupplierMergeResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::merge_suppliers(&mut conn, source_id, target_id).map_err(CmdError::from)
}

/// 取引先全件を関連商品数・入庫記録数付きで返す。
#[tauri::command]
#[specta::specta]
pub fn list_suppliers_with_usage(
    state: State<AppState>,
) -> Result<Vec<product_service::SupplierWithUsage>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::list_suppliers_with_usage(&conn).map_err(CmdError::from)
}

/// 商品の価格履歴を取得する。
#[tauri::command]
#[specta::specta]
pub fn list_price_history(
    state: State<AppState>,
    product_code: String,
    limit: u32,
) -> Result<Vec<product_service::PriceHistoryEntry>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::list_price_history(&conn, product_code, limit).map_err(CmdError::from)
}

/// 商品詳細を取得する
///
/// BIZ層（product_service）経由で呼び出す。
#[tauri::command]
#[specta::specta]
pub fn get_product(
    state: State<AppState>,
    product_code: String,
) -> Result<ProductWithRelations, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::get_product(&conn, &product_code).map_err(CmdError::from)
}

// ---------------------------------------------------------------------------
// 一括インポート（§22.6）
// ---------------------------------------------------------------------------

/// 商品マスタCSVのプレビューを返す（読み取り専用）
///
/// ファイル内容を解析し、有効行・エラー行・重複行に分類して返す。
/// DB書込みは行わない。
#[tauri::command]
#[specta::specta]
pub fn preview_import(
    state: State<AppState>,
    file_bytes: Vec<u8>,
) -> Result<product_service::ImportPreview, CmdError> {
    if file_bytes.len() > constants::CSV_IMPORT_FILE_SIZE_LIMIT {
        return Err(CmdError {
            kind: CmdErrorKind::Validation,
            message: "ファイルサイズが上限(20MB)を超えています".to_string(),
            field: None,
            error_id: None,
            stocktake_recovery: None,
            related_csv_import_id: None,
        });
    }
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::preview_import(&conn, &file_bytes).map_err(CmdError::from)
}

/// プレビュー済みの一括インポートを確定する
///
/// valid_rows の INSERT/UPDATE と、overwrite_codes の上書き処理をトランザクション内で実行。
#[tauri::command]
#[specta::specta]
pub fn commit_import(
    state: State<AppState>,
    valid_rows: Vec<ImportRow>,
    overwrite_codes: Vec<String>,
) -> Result<product_service::ProductImportResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::commit_import(&mut conn, valid_rows, overwrite_codes).map_err(CmdError::from)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::setup_test_db;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tauri::Manager;

    #[test]
    fn test_preview_import_req104_empty_file_validation() {
        // REQ-104 / BIZ-01-VAL-D1: 実CMDを空bytesで呼び、wire tripleを独立転記で固定する。
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

        let err = preview_import(app.state::<AppState>(), vec![]).unwrap_err();

        assert_eq!(err.kind, CmdErrorKind::Validation);
        assert_eq!(err.message, "ファイルが空です");
        assert_eq!(err.field, None);
    }

    #[test]
    fn test_import_products_req104_rejects_oversize_file() {
        // REQ-104 / UI-01c-D15: 上限ちょうどは CMD guard を通過し、+1 は CMD で早期拒否する。
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

        let mut boundary = vec![0xEF, 0xBB, 0xBF];
        boundary.extend_from_slice(
            "商品コード,商品名,部門ID,売価,原価,税率\nPI-001,商品,1,500,300,10\n".as_bytes(),
        );
        boundary.resize(crate::constants::CSV_IMPORT_FILE_SIZE_LIMIT, b'\n');
        let boundary_result = preview_import(app.state::<AppState>(), boundary);
        assert!(
            boundary_result.is_ok(),
            "上限ちょうどの有効CSVは受理されるべき: {boundary_result:?}"
        );

        let err = preview_import(
            app.state::<AppState>(),
            vec![0; crate::constants::CSV_IMPORT_FILE_SIZE_LIMIT + 1],
        )
        .unwrap_err();
        assert_eq!(err.kind, CmdErrorKind::Validation);
        assert_eq!(err.message, "ファイルサイズが上限(20MB)を超えています");
        assert_eq!(err.field, None);
    }
}
