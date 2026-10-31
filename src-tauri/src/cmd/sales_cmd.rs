//! CMD-09: 売上集計コマンド群
//!
//! docs/function-design/42-cmd-sales-stocktake.md §22.4 に基づく実装。

use crate::biz::sales_service::{self, SalesMode, SalesReportType};
use crate::cmd::{AppState, CmdError};
use base64::{engine::general_purpose, Engine as _};
use tauri::State;

// ---------------------------------------------------------------------------
// レスポンス型
// ---------------------------------------------------------------------------

/// 売上CSVエクスポートレスポンス（フロントエンド返却用）
///
/// csv_bytes を base64 エンコードして返す。
/// フロントエンド側で base64デコード → Blob → ダウンロード保存する。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct SalesExportResponse {
    /// UTF-8 BOM付きCSVバイト列のbase64エンコード
    pub bytes_base64: String,
    /// 推奨ファイル名
    pub suggested_filename: String,
    /// MIMEタイプ
    pub content_type: String,
    /// 文字エンコーディング名
    pub encoding: String,
    /// エクスポート件数
    pub record_count: usize,
}

/// 指定日の売上レポートを取得する
#[tauri::command]
#[specta::specta]
pub fn get_daily_sales(
    state: State<AppState>,
    date: String,
) -> Result<sales_service::DailySalesReport, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    sales_service::get_daily_sales(&conn, &date).map_err(CmdError::from)
}

/// 指定月の売上レポートを取得する
///
/// mode は generated enum を直接受け取る。
#[tauri::command]
#[specta::specta]
pub fn get_monthly_sales(
    state: State<AppState>,
    month: String,
    mode: SalesMode,
) -> Result<sales_service::MonthlySalesReport, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    sales_service::get_monthly_sales(&conn, &month, mode).map_err(CmdError::from)
}

/// 売上データをCSVファイルとしてエクスポートする
///
/// 42-cmd-sales-stocktake.md §22.4 export_sales_csv
///
/// `report_type` は `SalesReportType` enum を直接受け取る（H-5 / Q-6 A 案）。
/// frontend bindings の `exportSalesCsv(reportType: SalesReportType, target: string)` で
/// `"daily" | "monthly_by_product" | "monthly_by_department"` literal union を渡し、
/// serde::Deserialize で直接 enum に変換される（文字列 validation 不要、type safety 最大化）。
#[tauri::command]
#[specta::specta]
pub fn export_sales_csv(
    state: State<AppState>,
    report_type: SalesReportType,
    target: String,
) -> Result<SalesExportResponse, CmdError> {
    // Step 1: DB接続取得
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;

    // Step 2: BIZ層呼び出し
    let result =
        sales_service::export_sales_csv(&conn, &report_type, &target).map_err(CmdError::from)?;

    // Step 3: base64エンコード + レスポンス構築
    Ok(SalesExportResponse {
        bytes_base64: general_purpose::STANDARD.encode(&result.csv_bytes),
        suggested_filename: result.suggested_filename,
        content_type: "text/csv".to_string(),
        encoding: "UTF-8".to_string(),
        record_count: result.count,
    })
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::CmdErrorKind;
    use crate::db;
    use crate::db::product_repo::{self, NewProduct};
    use crate::db::sales_repo::{
        self, NewDailyReportDepartmentLine, NewDailyReportImport, NewDailyReportPaymentLine,
        NewSaleRecord,
    };
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

    #[test]
    fn test_monthly_sales_req502_valid_modes() {
        // REQ-502 / CMD-09-CONV-D1: 実CMDが2つの正規modeを対応variantへ変換する。
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let by_product = get_monthly_sales(
            app.state::<AppState>(),
            "2026-03".to_string(),
            SalesMode::ByProduct,
        )
        .unwrap();
        assert!(matches!(by_product.mode, SalesMode::ByProduct));

        let by_department = get_monthly_sales(
            app.state::<AppState>(),
            "2026-03".to_string(),
            SalesMode::ByDepartment,
        )
        .unwrap();
        assert!(matches!(by_department.mode, SalesMode::ByDepartment));
    }

    #[test]
    fn test_export_sales_csv_req501_invalid_report_type() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // PR #66 で CMD signature 変更 (String → SalesReportType): 文字列 validation は
        // serde::Deserialize に委譲、tauri-specta が IPC 層で reject する。
        // 不正値が SalesReportType に deserialize できないことを直接検証する。
        let invalid_types = vec![
            "\"invalid\"",
            "\"Daily\"",
            "\"DAILY\"",
            "\"\"",
            "\"monthly\"",
        ];
        for json in invalid_types {
            let result: Result<SalesReportType, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "JSON {} should fail to deserialize into SalesReportType",
                json
            );
        }
    }

    #[test]
    fn test_export_sales_csv_req501_valid_report_types() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // PR #66 で CMD signature 変更 (String → SalesReportType): snake_case literal 3 種が
        // serde::Deserialize で SalesReportType variant に正しく deserialize される。
        let cases = vec![
            ("\"daily\"", SalesReportType::Daily),
            ("\"monthly_by_product\"", SalesReportType::MonthlyByProduct),
            (
                "\"monthly_by_department\"",
                SalesReportType::MonthlyByDepartment,
            ),
        ];
        for (json, expected) in cases {
            let result: SalesReportType =
                serde_json::from_str(json).expect("valid snake_case literal should deserialize");
            assert!(
                matches!(
                    (&result, &expected),
                    (SalesReportType::Daily, SalesReportType::Daily)
                        | (
                            SalesReportType::MonthlyByProduct,
                            SalesReportType::MonthlyByProduct
                        )
                        | (
                            SalesReportType::MonthlyByDepartment,
                            SalesReportType::MonthlyByDepartment
                        )
                ),
                "JSON {} should deserialize into expected variant",
                json
            );
        }
    }

    // -------------------------------------------------------------------
    // export_sales_csv 返却契約テスト（DB→BIZ→base64変換の統合検証）
    // -------------------------------------------------------------------

    fn setup_test_db() -> (tempfile::TempDir, db::DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    #[test]
    fn test_export_sales_csv_req501_response_contract() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // CMD-09: BIZ呼び出し → base64変換 → レスポンス構築の全パスを検証
        let (_dir, conn) = setup_test_db();

        // テストデータ投入
        let product = NewProduct {
            product_code: "P001".to_string(),
            jan_code: None,
            name: "テスト商品".to_string(),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: 0,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(&conn, &product).unwrap();
        let record = NewSaleRecord {
            csv_import_id: None,
            product_code: "P001".to_string(),
            sale_date: "2026-03-21".to_string(),
            quantity: 2,
            amount: 1000,
            source: "auto".to_string(),
            source_line_no: None,
            reason: None,
            note: None,
            stock_linked: false,
        };
        sales_repo::insert_sale_record(&conn, &record).unwrap();

        // BIZ層呼び出し → CMD層と同じ変換を実行
        let biz_result =
            sales_service::export_sales_csv(&conn, &SalesReportType::Daily, "2026-03-21").unwrap();
        let response = SalesExportResponse {
            bytes_base64: general_purpose::STANDARD.encode(&biz_result.csv_bytes),
            suggested_filename: biz_result.suggested_filename,
            content_type: "text/csv".to_string(),
            encoding: "UTF-8".to_string(),
            record_count: biz_result.count,
        };

        // 返却契約の検証
        assert_eq!(response.content_type, "text/csv");
        assert_eq!(response.encoding, "UTF-8");
        assert_eq!(response.record_count, 1);
        assert_eq!(response.suggested_filename, "sales_daily_2026-03-21.csv");

        // base64デコード → BOM + ヘッダ + データの検証
        let decoded = general_purpose::STANDARD
            .decode(&response.bytes_base64)
            .expect("base64 decode should succeed");
        assert_eq!(&decoded[..3], &[0xEF, 0xBB, 0xBF], "UTF-8 BOM");
        let csv_str = String::from_utf8(decoded[3..].to_vec()).unwrap();
        assert!(csv_str.starts_with("商品コード,商品名,部門,数量,金額,記録元\r\n"));
        assert!(csv_str.contains("P001"));
        assert!(csv_str.contains("POS"), "auto → POS translation");
    }

    #[test]
    fn test_get_daily_sales_cmd_passes_official_report_req501() {
        // REQ-501: 日次売上公式日報表示
        // CMD-09: command層はDTOを再定義せずBIZ結果を透過返却する
        let (_dir, conn) = setup_test_db();
        let import_id = sales_repo::insert_daily_report_import(
            &conn,
            &NewDailyReportImport {
                report_date: "2026-03-21".to_string(),
                source_adapter: "casio_sr_s4000".to_string(),
                bundle_hash: "cmd-official".to_string(),
                source_files_json: "[]".to_string(),
                gross_amount: Some(12000),
                net_amount: Some(11000),
                status: "completed".to_string(),
                note: None,
                settlement_no: None,
            },
        )
        .unwrap();
        sales_repo::insert_daily_report_payment_lines(
            &conn,
            &[NewDailyReportPaymentLine {
                daily_report_import_id: import_id,
                source_file: "Z002".to_string(),
                payment_key: "cash".to_string(),
                label: "現金".to_string(),
                amount: Some(11000),
                count: Some(7),
                sort_order: 1,
            }],
        )
        .unwrap();
        sales_repo::insert_daily_report_department_lines(
            &conn,
            &[NewDailyReportDepartmentLine {
                daily_report_import_id: import_id,
                source_file: "Z005".to_string(),
                department_id: Some(1),
                raw_department_name: "その他小物".to_string(),
                normalized_department_name: Some("その他小物".to_string()),
                amount: 11000,
                quantity_hundredths: Some(700),
                count: Some(3),
                sort_order: 1,
            }],
        )
        .unwrap();
        let state = AppState {
            db: Mutex::new(conn),
            preview_cache: Mutex::new(HashMap::new()),
            daily_report_preview_cache: Mutex::new(HashMap::new()),
            register_sd_scan_cache: Mutex::new(HashMap::new()),
        };

        let conn = state.db.lock().unwrap();
        let response = sales_service::get_daily_sales(&conn, "2026-03-21").unwrap();

        let official = response.official_daily_report.unwrap();
        assert_eq!(official.source_import_count, 1);
        assert_eq!(official.payment_lines[0].label, "現金");
    }

    #[test]
    fn test_export_sales_csv_req501_db_lock_failure() {
        // REQ-501: 日次売上（日付別商品売上集計）
        // CMD-09: Mutex poison 時に CmdError::internal が返ること
        let (_dir, conn) = setup_test_db();
        let state = AppState {
            db: Mutex::new(conn),
            preview_cache: Mutex::new(HashMap::new()),
            daily_report_preview_cache: Mutex::new(HashMap::new()),
            register_sd_scan_cache: Mutex::new(HashMap::new()),
        };

        // Mutex を poison させる
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = state.db.lock().unwrap();
            panic!("intentional poison");
        }));

        // poison 後の lock は Err を返す → CmdError::internal に変換
        let result: Result<(), CmdError> = state
            .db
            .lock()
            .map(|_| ())
            .map_err(|error| CmdError::internal("DB接続エラー", error));
        let err = result.unwrap_err();
        assert_eq!(err.kind, CmdErrorKind::Internal);
        assert_eq!(err.message, "DB接続エラー");
    }
}
