//! CMD-06: 在庫照会コマンド群
//!
//! 44-cmd-inventory.md §23.8 に基づく実装。

use crate::biz::{
    inventory_service, product_service, MovementQuery, PaginatedResult, ProductWithRelations,
    StockDetail, StockHistoryItem,
};
use crate::cmd::{AppState, CmdError};
use tauri::State;

/// 商品の在庫詳細を取得する（最終入庫日・最終販売日を含む）
///
/// 44-cmd-inventory.md §23.8 get_stock_detail
#[tauri::command]
#[specta::specta]
pub fn get_stock_detail(
    state: State<AppState>,
    product_code: String,
) -> Result<StockDetail, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::get_stock_detail(&conn, &product_code).map_err(CmdError::from)
}

/// 在庫が閾値以下の商品を一覧取得する
///
/// 44-cmd-inventory.md §23.8 list_low_stock
#[tauri::command]
#[specta::specta]
pub fn list_low_stock(
    state: State<AppState>,
    include_discontinued: bool,
) -> Result<Vec<ProductWithRelations>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    product_service::list_low_stock(&conn, include_discontinued).map_err(CmdError::from)
}

/// 商品別の在庫変動の履歴（movement と計数）をフィルタ付きでページング取得する
///
/// 44-cmd-inventory.md §23.8 list_movements
#[tauri::command]
#[specta::specta]
pub fn list_movements(
    state: State<AppState>,
    query: MovementQuery,
) -> Result<PaginatedResult<StockHistoryItem>, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    inventory_service::list_movements(&conn, &query).map_err(CmdError::from)
}

#[cfg(test)]
mod tests {
    use crate::biz::BizError;
    use crate::cmd::{CmdError, CmdErrorKind};
    use crate::db;

    fn setup_db() -> db::DbConnection {
        let conn = db::init_database(":memory:").unwrap();
        db::migration::migrate(&conn).unwrap();
        conn
    }

    /// get_stock_detail: 存在しない商品 → BizError::NotFound → CmdError { kind: "not_found" }
    #[test]
    fn test_get_stock_detail_req301_not_found() {
        // REQ-301: 商品別在庫照会
        let conn = setup_db();
        let err = crate::biz::product_service::get_stock_detail(&conn, "NONEXISTENT").unwrap_err();
        let cmd_err: CmdError = err.into();
        assert_eq!(cmd_err.kind, CmdErrorKind::NotFound);
    }

    /// list_low_stock: デフォルト閾値で空テーブル → 空Vec
    #[test]
    fn test_list_low_stock_req302_empty() {
        // REQ-302: 在庫少一覧
        let conn = setup_db();
        let result = crate::biz::product_service::list_low_stock(&conn, false).unwrap();
        assert!(result.is_empty());
    }

    /// list_movements: BIZ経由の統合テスト（空テーブル）
    #[test]
    fn test_list_movements_req303_empty() {
        // REQ-303: 在庫変動履歴
        let conn = setup_db();
        let query = crate::biz::MovementQuery {
            product_code: "TEST-001".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 20,
        };
        let result = crate::biz::inventory_service::list_movements(&conn, &query).unwrap();
        assert_eq!(result.total_count, 0);
        assert!(result.items.is_empty());
    }

    /// list_movements: per_page > 200 → ValidationFailed
    #[test]
    fn test_list_movements_req303_per_page_exceeds_max() {
        // REQ-303: 在庫変動履歴
        let conn = setup_db();
        let query = crate::biz::MovementQuery {
            product_code: "TEST-001".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 201,
        };
        let err = crate::biz::inventory_service::list_movements(&conn, &query).unwrap_err();
        match err {
            BizError::ValidationFailed(_) => {}
            _ => panic!("expected ValidationFailed, got {:?}", err),
        }
    }

    // -----------------------------------------------------------------------
    // データ投入込みの統合テスト
    // -----------------------------------------------------------------------

    fn seed_product_with_stock(conn: &db::DbConnection, code: &str, stock: i64, unit: &str) {
        let product = crate::db::product_repo::NewProduct {
            product_code: code.to_string(),
            jan_code: None,
            name: format!("テスト商品_{}", code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: stock,
            stock_unit: unit.to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        crate::db::product_repo::insert_product(conn, &product).unwrap();
    }

    fn insert_movement(conn: &db::DbConnection, code: &str, mt: &str, qty: i64, is_voided: bool) {
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, is_voided, created_at, business_date, phase) \
             VALUES (?1, ?2, ?3, 0, ?4, '2026-03-15T10:00:00', '2026-04-01', 0)",
            rusqlite::params![code, mt, qty, is_voided as i32],
        )
        .unwrap();
    }

    /// get_stock_detail: 商品存在時に最終入庫日・最終販売日を返す
    #[test]
    fn test_get_stock_detail_req301_with_data() {
        // REQ-301: 商品別在庫照会
        let conn = setup_db();
        seed_product_with_stock(&conn, "SD-001", 10, "pcs");
        // 入庫記録
        conn.execute(
            "INSERT INTO receiving_records (idempotency_key, request_fingerprint, supplier_id, receiving_date, note, created_at) \
             VALUES ('test-key-1', 'fp1', NULL, '2026-03-20', NULL, '2026-03-20T10:00:00')",
            [],
        )
        .unwrap();
        let rec_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price_centi) \
             VALUES (?1, 'SD-001', 5, 300)",
            rusqlite::params![rec_id],
        )
        .unwrap();
        // 売上記録
        conn.execute(
            "INSERT INTO sale_records (product_code, sale_date, quantity, amount, source, is_voided, created_at) \
             VALUES ('SD-001', '2026-03-22', 2, 1000, 'auto', 0, '2026-03-22T19:00:00')",
            [],
        )
        .unwrap();

        let detail = crate::biz::product_service::get_stock_detail(&conn, "SD-001").unwrap();
        assert_eq!(detail.product.product.product_code, "SD-001");
        assert_eq!(detail.last_receiving_date, Some("2026-03-20".to_string()));
        assert_eq!(detail.last_sale_date, Some("2026-03-22".to_string()));
    }

    /// list_low_stock: pcs/cm閾値分岐テスト
    #[test]
    fn test_list_low_stock_req302_threshold_branching() {
        // REQ-302: 在庫少一覧
        let conn = setup_db();
        // pcs商品: stock=2（閾値3以下 → ヒット）
        seed_product_with_stock(&conn, "LS-PCS", 2, "pcs");
        // cm商品: stock=300（閾値500以下 → ヒット）
        seed_product_with_stock(&conn, "LS-CM", 300, "m");
        // pcs商品: stock=10（閾値3超 → ヒットしない）
        seed_product_with_stock(&conn, "LS-OK", 10, "pcs");

        let result = crate::biz::product_service::list_low_stock(&conn, false).unwrap();
        assert_eq!(result.len(), 2);
        let codes: Vec<&str> = result
            .iter()
            .map(|p| p.product.product_code.as_str())
            .collect();
        assert!(codes.contains(&"LS-PCS"));
        assert!(codes.contains(&"LS-CM"));
        assert!(!codes.contains(&"LS-OK"));
    }

    /// list_low_stock: 数量の種類の分岐（個数の単位は一般の基準、長さの単位は生地の基準）
    #[test]
    fn test_list_low_stock_req302_branches_by_unit_kind() {
        // REQ-302 / SPEC-UNIT-D10: 新しい単位が在庫少から漏れない
        let conn = setup_db();
        seed_product_with_stock(&conn, "LK-BALL", 2, "ball"); // 一般の基準 3 以下 → 出る
        seed_product_with_stock(&conn, "LK-M-LOW", 400, "m"); // 生地の基準 500 以下 → 出る
        seed_product_with_stock(&conn, "LK-M-OK", 600, "m"); // 生地の基準を超える → 出ない
        seed_product_with_stock(&conn, "LK-PCS-OK", 4, "pcs"); // 一般の基準を超える → 出ない
        seed_product_with_stock(&conn, "LK-BALL-OK", 400, "ball"); // 長さの基準で比べない → 出ない

        let result = crate::biz::product_service::list_low_stock(&conn, false).unwrap();
        let mut codes: Vec<&str> = result
            .iter()
            .map(|p| p.product.product_code.as_str())
            .collect();
        codes.sort_unstable();
        assert_eq!(codes, vec!["LK-BALL", "LK-M-LOW"]);
    }

    /// list_low_stock: 10 個の単位を 1 つずつ在庫 0 で置くと 10 個とも出る
    #[test]
    fn test_list_low_stock_req302_every_unit_is_listed() {
        // REQ-302 / SPEC-UNIT-D10
        let conn = setup_db();
        for unit in crate::db::product_repo::ProductStockUnit::ALL {
            seed_product_with_stock(&conn, &format!("LA-{}", unit.as_str()), 0, unit.as_str());
        }

        let result = crate::biz::product_service::list_low_stock(&conn, false).unwrap();
        let mut units: Vec<&str> = result
            .iter()
            .map(|p| p.product.stock_unit.as_str())
            .collect();
        units.sort_unstable();
        let mut expected: Vec<&str> = crate::db::product_repo::ProductStockUnit::ALL
            .iter()
            .map(|unit| unit.as_str())
            .collect();
        expected.sort_unstable();
        assert_eq!(units.len(), 10);
        assert_eq!(units, expected);
    }

    /// list_low_stock: include_discontinued=false で廃番除外
    #[test]
    fn test_list_low_stock_req302_exclude_discontinued() {
        // REQ-302: 在庫少一覧
        let conn = setup_db();
        seed_product_with_stock(&conn, "LS-ACT", 1, "pcs");
        seed_product_with_stock(&conn, "LS-DIS", 1, "pcs");
        conn.execute(
            "UPDATE products SET is_discontinued = 1 WHERE product_code = 'LS-DIS'",
            [],
        )
        .unwrap();

        let result = crate::biz::product_service::list_low_stock(&conn, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].product.product_code, "LS-ACT");

        // include_discontinued=true で廃番も含む
        let result = crate::biz::product_service::list_low_stock(&conn, true).unwrap();
        assert_eq!(result.len(), 2);
    }

    /// list_movements: is_voided=0 フィルタ（voided行は除外される）
    #[test]
    fn test_list_movements_req303_voided_excluded() {
        // REQ-303: 在庫変動履歴
        let conn = setup_db();
        seed_product_with_stock(&conn, "MV-001", 10, "pcs");
        insert_movement(&conn, "MV-001", "receiving", 5, false);
        insert_movement(&conn, "MV-001", "sale_auto", -2, true); // voided

        let query = crate::biz::MovementQuery {
            product_code: "MV-001".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 50,
        };
        let result = crate::biz::inventory_service::list_movements(&conn, &query).unwrap();
        assert_eq!(result.total_count, 1); // voided行は除外
        assert_eq!(
            result.items[0].movement.as_ref().unwrap().movement_type,
            crate::db::inventory_repo::MovementType::Receiving
        );
    }

    /// list_movements: movement_type フィルタ
    #[test]
    fn test_list_movements_req303_type_filter() {
        // REQ-303: 在庫変動履歴
        let conn = setup_db();
        seed_product_with_stock(&conn, "MV-002", 10, "pcs");
        insert_movement(&conn, "MV-002", "receiving", 5, false);
        insert_movement(&conn, "MV-002", "sale_auto", -2, false);

        let query = crate::biz::MovementQuery {
            product_code: "MV-002".to_string(),
            date_from: None,
            date_to: None,
            movement_type: Some(crate::db::inventory_repo::MovementType::Receiving),
            page: 1,
            per_page: 50,
        };
        let result = crate::biz::inventory_service::list_movements(&conn, &query).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(
            result.items[0].movement.as_ref().unwrap().movement_type,
            crate::db::inventory_repo::MovementType::Receiving
        );
    }

    /// list_movements: CMD経由で source link を含む
    #[test]
    fn test_list_movements_req207_source_link_through_biz() {
        // REQ-207 / TRACE-D2: CMD は BIZ が解決した元記録 link をそのまま返す
        let conn = setup_db();
        seed_product_with_stock(&conn, "MV-CMD", 10, "pcs");
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, is_voided, created_at, business_date, phase) \
             VALUES ('MV-CMD', 'disposal', -2, 8, 'disposal_record', 15, 0, '2026-03-15T10:00:00', '2026-04-01', 0)",
            [],
        )
        .unwrap();

        let query = crate::biz::MovementQuery {
            product_code: "MV-CMD".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 20,
        };
        let result = crate::biz::inventory_service::list_movements(&conn, &query).unwrap();
        let movement = result.items[0].movement.as_ref().expect("movement row");
        let source = movement.source.as_ref().expect("source link");

        assert_eq!(source.label, "廃棄・破損 #15");
        assert_eq!(source.route, "/inventory/disposal/records/15");
    }

    /// list_movements: 実 CMD が BIZ の履歴（movement と計数）をそのまま返す
    #[test]
    fn test_list_movements_req303_cmd_returns_history_with_counts() {
        // REQ-303（44 の D-118 の b2 の節）: CMD は BIZ の PaginatedResult<StockHistoryItem> を変えずに返す。Matrix W2
        use crate::biz::inventory_service::StockHistoryKind;
        use crate::cmd::AppState;
        use std::collections::HashMap;
        use std::sync::Mutex;
        use tauri::Manager;

        let conn = setup_db();
        seed_product_with_stock(&conn, "MV-HIST", 10, "pcs");
        insert_movement(&conn, "MV-HIST", "receiving", 5, false);
        conn.execute(
            "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date, counted_at)
             VALUES ('MV-HIST', 7, 'default', '2026-04-02', '2026-04-02T10:00:00')",
            [],
        )
        .unwrap();
        let app = tauri::test::mock_builder()
            .manage(AppState {
                db: Mutex::new(conn),
                preview_cache: Mutex::new(HashMap::new()),
                daily_report_preview_cache: Mutex::new(HashMap::new()),
                register_sd_scan_cache: Mutex::new(HashMap::new()),
            })
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let query = crate::biz::MovementQuery {
            product_code: "MV-HIST".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 20,
        };

        let result = super::list_movements(app.state::<AppState>(), query).unwrap();

        assert_eq!(result.total_count, 2);
        let kinds: Vec<_> = result.items.iter().map(|item| item.kind).collect();
        assert_eq!(kinds, [StockHistoryKind::Count, StockHistoryKind::Movement]);
        let count = result.items[0].count.as_ref().expect("count row");
        assert_eq!(count.count_value, 7);
        assert_eq!(result.items[0].business_date, "2026-04-02");
    }
}
