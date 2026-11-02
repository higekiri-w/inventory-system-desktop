//! CMD-10: 棚卸しコマンド群
//!
//! docs/function-design/42-cmd-sales-stocktake.md §22.5 に基づく実装。

use crate::biz::stocktake_service::StocktakeRecordDetail;
use crate::biz::stocktake_service::{
    self, CompleteStocktakeRequest, CountPlacementPreview, CountRecoveryTarget,
    SaveStockCountRequest, StockCountSaveResult,
};
use crate::biz::{LastStocktakeSummary, Stocktake, StocktakeItemDetail, StocktakeProgress};
use crate::cmd::{AppState, CmdError};
use tauri::State;

// ---------------------------------------------------------------------------
// CMD専用型
// ---------------------------------------------------------------------------

/// 棚卸しアイテム一覧レスポンス（CMD-10 get_stocktake_items 専用）
///
/// items（PaginatedResult相当）+ progress を1レスポンスにまとめる。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StocktakeItemListResponse {
    pub items: Vec<StocktakeItemDetail>,
    pub progress: StocktakeProgress,
    pub total_count: u32,
    pub page: u32,
    pub per_page: u32,
}

/// PC の今日（ローカルの日付）。BIZ は時計を読まず、CMD がここから 1 か所で渡す（42）
fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

// ---------------------------------------------------------------------------
// コマンド
// ---------------------------------------------------------------------------

/// 進行中の棚卸しを取得する
///
/// BIZ層（stocktake_service）経由で呼び出す。読み取り専用。
#[tauri::command]
#[specta::specta]
pub fn get_active_stocktake(state: State<AppState>) -> Result<Option<Stocktake>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::get_active_stocktake(&conn).map_err(CmdError::from)
}

/// 新しい棚卸しを開始する
///
/// 進行中の棚卸しが既にある場合は stocktake_in_progress エラー。
#[tauri::command]
#[specta::specta]
pub fn start_stocktake(
    state: State<AppState>,
) -> Result<stocktake_service::StartStocktakeResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::start_stocktake(&mut conn).map_err(CmdError::from)
}

/// 棚卸しアイテム一覧を取得する
///
/// BIZ層（stocktake_service）経由で呼び出す。読み取り専用。
#[tauri::command]
#[specta::specta]
pub fn get_stocktake_items(
    state: State<AppState>,
    stocktake_id: i64,
    department_id: Option<i64>,
    counted_only: Option<bool>,
    page: u32,
    per_page: u32,
) -> Result<StocktakeItemListResponse, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;

    let (paginated, progress) = stocktake_service::get_stocktake_items(
        &conn,
        stocktake_id,
        department_id,
        counted_only,
        page,
        per_page,
    )
    .map_err(CmdError::from)?;

    Ok(StocktakeItemListResponse {
        items: paginated.items,
        total_count: paginated.total_count,
        page: paginated.page,
        per_page: paginated.per_page,
        progress,
    })
}

/// 商品コードまたはJANコードで棚卸しアイテムを取得する
///
/// BIZ層（stocktake_service）経由で呼び出す。読み取り専用。
#[tauri::command]
#[specta::specta]
pub fn find_stocktake_item(
    state: State<AppState>,
    stocktake_id: i64,
    code: String,
) -> Result<Option<StocktakeItemDetail>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::find_stocktake_item(&conn, stocktake_id, &code).map_err(CmdError::from)
}

/// 最後に完了した棚卸しを取得する
///
/// 完了済みがない場合は None を返す。読み取り専用。
#[tauri::command]
#[specta::specta]
pub fn get_last_completed_stocktake(
    state: State<AppState>,
) -> Result<Option<LastStocktakeSummary>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::get_last_completed_stocktake(&conn).map_err(CmdError::from)
}

/// 棚卸し記録詳細を取得する。
///
/// docs/function-design/42-cmd-sales-stocktake.md §22.5 / REQ-206 / REQ-207
#[tauri::command]
#[specta::specta]
pub fn get_stocktake_record(
    state: State<AppState>,
    stocktake_id: i64,
) -> Result<StocktakeRecordDetail, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::get_stocktake_record(&conn, stocktake_id).map_err(CmdError::from)
}

/// 棚卸しを完了する（拒否は `stocktake_guard` の `completion_blocked`。42）
///
/// 完了後に整合性チェック（D-2統合）を自動実行し、結果を含めて返す。CMD は PC の今日を BIZ へ渡す。
#[tauri::command]
#[specta::specta]
pub fn complete_stocktake(
    state: State<AppState>,
    stocktake_id: i64,
) -> Result<stocktake_service::StocktakeResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    let req = CompleteStocktakeRequest {
        stocktake_id,
        today: today(),
    };
    stocktake_service::complete_stocktake(&mut conn, &req).map_err(CmdError::from)
}

/// 計数を保存する（既定の計数・fresh な計数。42、35「計数の保存」）。CMD は PC の今日を BIZ へ渡す
#[tauri::command]
#[specta::specta]
pub fn save_stock_count(
    state: State<AppState>,
    request: SaveStockCountRequest,
) -> Result<StockCountSaveResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::save_stock_count(&mut conn, &request, &today()).map_err(CmdError::from)
}

/// 後入力の確認と計数の欄の商品の文脈（読取りだけ。42）
#[tauri::command]
#[specta::specta]
pub fn preview_stock_count_placement(
    state: State<AppState>,
    product_code: String,
    business_date: String,
) -> Result<CountPlacementPreview, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::preview_stock_count_placement(&conn, &product_code, &business_date, &today())
        .map_err(CmdError::from)
}

/// 未計数の明細を値 0 の既定の計数でまとめて片付ける（42、35「完了」）
#[tauri::command]
#[specta::specta]
pub fn zero_uncounted_items(
    state: State<AppState>,
    stocktake_id: i64,
    product_codes: Vec<String>,
    business_date: String,
) -> Result<Vec<StockCountSaveResult>, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::zero_uncounted_items(
        &mut conn,
        stocktake_id,
        &product_codes,
        &business_date,
        &today(),
    )
    .map_err(CmdError::from)
}

/// 要再確認の一覧（棚卸しが無いときも読める。42）
#[tauri::command]
#[specta::specta]
pub fn list_recheck_products(state: State<AppState>) -> Result<Vec<CountRecoveryTarget>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    stocktake_service::list_recheck_products(&conn).map_err(CmdError::from)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::CmdErrorKind;
    use crate::db::test_support::{seed_product, setup_test_db};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tauri::Manager;

    fn app_state_for_test(conn: crate::db::DbConnection) -> AppState {
        AppState {
            db: Mutex::new(conn),
            preview_cache: Mutex::new(HashMap::new()),
            daily_report_preview_cache: Mutex::new(HashMap::new()),
            register_sd_scan_cache: Mutex::new(HashMap::new()),
        }
    }

    fn setup_in_progress_stocktake() -> (tempfile::TempDir, crate::db::DbConnection, i64, i64) {
        let (dir, conn) = setup_test_db();
        seed_product(&conn, "COUNT-001");
        conn.execute(
            "INSERT INTO stocktakes (started_at, status, total_cost)
             VALUES ('2026-10-01T09:00:00', 'in_progress', NULL)",
            [],
        )
        .unwrap();
        let stocktake_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
             VALUES (?1, 'COUNT-001', 0, NULL)",
            [stocktake_id],
        )
        .unwrap();
        let item_id = conn.last_insert_rowid();
        (dir, conn, stocktake_id, item_id)
    }

    #[test]
    fn test_get_stocktake_items_req205_page_zero_validation() {
        // REQ-205 / BIZ-06-VAL-D1: 実CMDのpage下限とwire tripleを独立転記で固定する。
        let (_dir, conn, stocktake_id, _item_id) = setup_in_progress_stocktake();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let err = get_stocktake_items(app.state::<AppState>(), stocktake_id, None, None, 0, 50)
            .unwrap_err();

        assert_eq!(err.kind, CmdErrorKind::Validation);
        assert_eq!(err.message, "ページ番号は1以上で指定してください");
        assert_eq!(err.field.as_deref(), Some("page"));
    }

    #[test]
    fn test_get_stocktake_items_req205_per_page_zero_validation() {
        // REQ-205 / BIZ-06-VAL-D1: 実CMDのper_page下限とwire tripleを独立転記で固定する。
        let (_dir, conn, stocktake_id, _item_id) = setup_in_progress_stocktake();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let err = get_stocktake_items(app.state::<AppState>(), stocktake_id, None, None, 1, 0)
            .unwrap_err();

        assert_eq!(err.kind, CmdErrorKind::Validation);
        assert_eq!(err.message, "1ページあたりの件数は1以上で指定してください");
        assert_eq!(err.field.as_deref(), Some("per_page"));
    }

    #[test]
    fn test_find_stocktake_item_req205_cmd_calls_command_and_returns_some() {
        // REQ-205: 棚卸し（CMD — 商品コード/JAN完全一致で棚卸し明細を返す）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "FI-001");
        conn.execute(
            "UPDATE products SET jan_code = '4900000001001', name = '検索対象商品'
             WHERE product_code = 'FI-001'",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO stocktakes (started_at, status, total_cost)
             VALUES ('2026-10-01T09:00:00', 'in_progress', NULL)",
            [],
        )
        .unwrap();
        let stocktake_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
             VALUES (?1, 'FI-001', 8, NULL)",
            [stocktake_id],
        )
        .unwrap();
        let item_id = conn.last_insert_rowid();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let result = find_stocktake_item(
            app.state::<AppState>(),
            stocktake_id,
            "4900000001001".to_string(),
        )
        .unwrap();

        let item = result.expect("JAN一致の棚卸し明細が返るべき");
        assert_eq!(item.id, item_id);
        assert_eq!(item.stocktake_id, stocktake_id);
        assert_eq!(item.product_code, "FI-001");
        assert_eq!(item.name, "検索対象商品");
        assert_eq!(item.system_stock, 8);
        assert!(item.actual_count.is_none());
    }

    #[test]
    fn test_find_stocktake_item_req205_cmd_calls_command_and_returns_none() {
        // REQ-205: 棚卸し（CMD — 対象なしはエラー化せず None を返す）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "FN-001");
        conn.execute(
            "INSERT INTO stocktakes (started_at, status, total_cost)
             VALUES ('2026-10-01T09:00:00', 'in_progress', NULL)",
            [],
        )
        .unwrap();
        let stocktake_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
             VALUES (?1, 'FN-001', 8, NULL)",
            [stocktake_id],
        )
        .unwrap();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let result = find_stocktake_item(
            app.state::<AppState>(),
            stocktake_id,
            "NO-MATCH".to_string(),
        )
        .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_get_last_completed_stocktake_req205_cmd_calls_command_and_returns_none() {
        // REQ-205: 棚卸し（CMD — 最終完了棚卸しは存在しない状態をエラーにしない）
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let result = get_last_completed_stocktake(app.state::<AppState>()).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_get_active_stocktake_req205_cmd_calls_command_and_returns_active() {
        // REQ-205: 棚卸し（CMD — get_active_stocktake は DB の進行中棚卸しを返す）
        let (_dir, conn) = setup_test_db();
        conn.execute(
            "INSERT INTO stocktakes (started_at, status, total_cost)
             VALUES ('2026-10-01T09:00:00', 'in_progress', NULL)",
            [],
        )
        .unwrap();
        let stocktake_id = conn.last_insert_rowid();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let result = get_active_stocktake(app.state::<AppState>()).unwrap();

        let stocktake = result.expect("進行中棚卸しが返るべき");
        assert_eq!(stocktake.id, stocktake_id);
        assert_eq!(stocktake.status, "in_progress");
    }

    #[test]
    fn test_get_stocktake_record_cmd_req206_not_found_kind() {
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let error = get_stocktake_record(app.state::<AppState>(), 99_999).unwrap_err();

        assert_eq!(error.kind, CmdErrorKind::NotFound);
        assert_eq!(error.message, "棚卸し記録が見つかりません: 99999");
        assert_eq!(error.field, None);
    }

    #[test]
    fn test_stocktake_cmd_req205_new_commands_registered() {
        // REQ-205 / 42 / Registration Obligations（b1 Matrix W2）: lib.rs の 2 つの一覧（collect_commands と
        // generate_handler）の棚卸しの command が 42 の表と完全一致する（新設 4 があり、外した command が無い）
        let source = include_str!("../lib.rs");
        let commands = |anchor: &str| -> Vec<&str> {
            let start = source.find(anchor).expect(anchor);
            let end = source[start..].find("])").expect("list end") + start;
            source[start..end]
                .lines()
                .filter_map(|line| line.trim().strip_prefix("cmd::stocktake_cmd::"))
                .map(|name| name.trim_end_matches(','))
                .collect()
        };
        let expected = vec![
            "get_active_stocktake",
            "start_stocktake",
            "get_stocktake_items",
            "find_stocktake_item",
            "get_last_completed_stocktake",
            "get_stocktake_record",
            "save_stock_count",
            "preview_stock_count_placement",
            "zero_uncounted_items",
            "list_recheck_products",
            "complete_stocktake",
        ];
        assert_eq!(commands("collect_commands!["), expected);
        assert_eq!(commands("tauri::generate_handler!["), expected);
    }

    #[test]
    fn test_complete_stocktake_cmd_req205_completion_blocked_wire() {
        // REQ-205 / 42・40（b1 Matrix W4）: complete_stocktake(stocktake_id) の completion_blocked の wire が完全一致
        let (_dir, conn, stocktake_id, item_id) = setup_in_progress_stocktake();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let error = complete_stocktake(app.state::<AppState>(), stocktake_id).unwrap_err();
        assert_eq!(
            serde_json::to_value(&error).unwrap(),
            serde_json::json!({
                "kind": "stocktake_guard",
                "message": "棚卸しを完了できません。数え直し・未計数・取り込んでいない精算のある商品があります",
                "field": null,
                "error_id": null,
                "related_csv_import_id": null,
                "stocktake_recovery": {
                    "code": "completion_blocked",
                    "targets": [{
                        "product_code": "COUNT-001",
                        "product_name": "テスト商品",
                        "stock_unit": "pcs",
                        "stock": 0,
                        "stock_range_low": null,
                        "stock_range_high": null,
                        "stocktake_item_id": item_id,
                        "stocktake_id": stocktake_id,
                        "action": "active_count",
                        "block_reason": "uncounted",
                        "recount_reasons": [],
                    }],
                    "settlement_pending": null,
                },
            })
        );
    }

    #[test]
    fn test_cmd_error_req205_recovery_targets_use_null_for_missing_refs() {
        // REQ-205 / 40（b1 Matrix W8）: 参照の無い CountRecoveryTarget は null（架空の id を使わない）。
        // recount_reasons は要再確認の対象のときだけ
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "REF-001");
        let error: CmdError = crate::biz::stocktake_service::recount_required(
            &conn,
            "REF-001",
            "連動しない間の売上があります。今の数を入れてから在庫連動にしてください",
        )
        .unwrap()
        .into();
        let target = &serde_json::to_value(&error).unwrap()["stocktake_recovery"]["targets"][0];
        assert_eq!(target["stocktake_item_id"], serde_json::Value::Null);
        assert_eq!(target["stocktake_id"], serde_json::Value::Null);
        assert_eq!(target["block_reason"], serde_json::Value::Null);
        assert_eq!(target["action"], serde_json::json!("count_now"));
        assert_eq!(target["recount_reasons"], serde_json::json!([]));

        // 要再確認の対象は理由を持つ（同じ日付の値の違う 2 つの計数 = 理由 3）
        for value in [3, 4] {
            crate::biz::stock_ledger::test_support::insert_count(
                &conn,
                "REF-001",
                value,
                "2026-04-01",
                None,
            );
        }
        let mut conn = conn;
        let tx = conn.transaction().unwrap();
        crate::biz::stock_ledger::recompute_stock(&tx, "REF-001").unwrap();
        tx.commit().unwrap();
        let targets = crate::biz::stocktake_service::list_recheck_products(&conn).unwrap();
        let target = serde_json::to_value(&targets[0]).unwrap();
        assert_eq!(
            target["recount_reasons"],
            serde_json::json!(["conflicting_counts"])
        );
        assert_eq!(target["stocktake_item_id"], serde_json::Value::Null);
    }
}
