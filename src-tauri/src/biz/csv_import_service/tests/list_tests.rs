use crate::biz::csv_import_service::{list, CsvImportErrorType};
use crate::biz::BizError;
use crate::db::sales_repo::{self, NewCsvImport, NewCsvImportError};
use crate::db::test_support::{seed_product, setup_test_db};
use crate::db::DbError;

// --- list_csv_imports テスト ---

#[test]
fn test_list_csv_imports_req401_normal() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // 2件のインポートを作成
    for i in 1..=2 {
        sales_repo::insert_csv_import(
            &conn,
            &NewCsvImport {
                filename: format!("Z004_{}", i),
                settlement_date: format!("2026-03-2{}", i),
                file_hash: format!("hash{}", i),
                total_items: i as i64,
                total_amount: i as i64 * 100,
                skipped_count: 0,
                status: "completed".to_string(),
                machine_no: "0001".to_string(),
                settlement_no: i as i64,
            },
        )
        .unwrap();
    }

    let result = list::list_csv_imports(&conn, 1, 10).unwrap();
    assert_eq!(result.total_count, 2);
    assert_eq!(result.items.len(), 2);
}

#[test]
fn test_list_csv_imports_req401_invalid_page() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    let result = list::list_csv_imports(&conn, 0, 10);
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ValidationFailed(msg) => assert!(msg.contains("ページパラメータ")),
        e => panic!("Expected ValidationFailed, got {:?}", e),
    }
}

#[test]
fn test_list_csv_imports_req401_invalid_per_page() {
    // REQ-401: CSV取込み
    let (_dir, conn) = setup_test_db();
    // per_page = 0
    let r1 = list::list_csv_imports(&conn, 1, 0);
    assert!(r1.is_err());

    // per_page = 101
    let r2 = list::list_csv_imports(&conn, 1, 101);
    assert!(r2.is_err());
    match r2.unwrap_err() {
        BizError::ValidationFailed(_) => {}
        e => panic!("Expected ValidationFailed, got {:?}", e),
    }
}

fn seed_detail_import(conn: &crate::db::DbConnection, status: &str) -> i64 {
    sales_repo::insert_csv_import(
        conn,
        &NewCsvImport {
            filename: "Z004_synthetic.csv".to_string(),
            settlement_date: "2026-08-03".to_string(),
            file_hash: "internal-file-hash".to_string(),
            total_items: 1,
            total_amount: 500,
            skipped_count: 0,
            status: status.to_string(),
            machine_no: "0001".to_string(),
            settlement_no: 1,
        },
    )
    .unwrap()
}

#[test]
fn test_get_csv_import_record_req206_not_found_message() {
    // REQ-206 / 32 §15.6a: IO NotFound を operator 向け BIZ error に変換する。
    let (_dir, conn) = setup_test_db();

    let result = list::get_csv_import_record(&conn, 99_999);

    match result.unwrap_err() {
        BizError::NotFound(message) => {
            assert_eq!(message, "CSV取込み記録が見つかりません: 99999");
        }
        other => panic!("NotFound を期待: {other:?}"),
    }
}

#[test]
fn test_get_csv_import_record_req206_error_row_mapping() {
    // REQ-206 / D-061: 4 error_type と field 写像を wire DTO で固定する。
    let (_dir, conn) = setup_test_db();
    let import_id = seed_detail_import(&conn, "completed_partial");
    let cases = [
        ("unmatched_product", CsvImportErrorType::UnmatchedProduct),
        ("invalid_format", CsvImportErrorType::InvalidFormat),
        ("invalid_jan", CsvImportErrorType::InvalidJan),
        ("invalid_number", CsvImportErrorType::InvalidNumber),
    ];
    for (index, (error_type, _)) in cases.iter().enumerate() {
        sales_repo::insert_csv_import_errors(
            &conn,
            &[NewCsvImportError {
                csv_import_id: import_id,
                source_line_no: (index + 10) as i64,
                normalized_jan: Some(format!("490000000000{index}")),
                raw_name: format!("エラー商品{index}"),
                raw_quantity: format!("q{index}"),
                raw_amount: format!("a{index}"),
                error_type: (*error_type).to_string(),
                error_message: format!("エラー{index}"),
            }],
        )
        .unwrap();
    }

    let detail = list::get_csv_import_record(&conn, import_id).unwrap();

    assert_eq!(detail.error_rows.len(), 4);
    for (index, (_, expected_type)) in cases.iter().enumerate() {
        let row = &detail.error_rows[index];
        assert_eq!(row.line_no, index + 10);
        assert_eq!(row.name, format!("エラー商品{index}"));
        assert_eq!(row.error_type, *expected_type);
    }
    let wire = serde_json::to_value(&detail).unwrap();
    assert!(wire.get("file_hash").is_none());
    assert!(matches!(
        CsvImportErrorType::from_db("future_error_type"),
        Err(DbError::QueryFailed(_))
    ));
}

#[test]
fn test_get_csv_import_record_req207_movement_source() {
    // REQ-207 / 32 §15.6a: source 規則は inventory_service の共有実装を使う。
    let (_dir, conn) = setup_test_db();
    seed_product(&conn, "CSV-DETAIL-P1");
    let import_id = seed_detail_import(&conn, "completed");
    conn.execute(
        "INSERT INTO inventory_movements
         (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase, settlement_no)
         VALUES ('CSV-DETAIL-P1', 'sale_auto', -1, 9, 'csv_import', ?1, NULL, 0, '2026-08-03T12:00:00', '2026-04-01', 1, 1)",
        rusqlite::params![import_id],
    )
    .unwrap();

    let detail = list::get_csv_import_record(&conn, import_id).unwrap();
    let source = detail.movements[0].source.as_ref().expect("source link");

    assert_eq!(source.label, format!("CSV取込み #{import_id}"));
    assert_eq!(source.route, format!("/csv-import/records/{import_id}"));
}
