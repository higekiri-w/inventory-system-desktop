use super::*;
use crate::biz::BizError;
use crate::constants;
use crate::db::product_repo::{self, NewProduct};
use crate::db::sales_repo;
use crate::db::test_support::setup_test_db;
use std::time::{Duration, Instant};

fn encode_cp932(text: &str) -> Vec<u8> {
    let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(text);
    encoded.to_vec()
}

fn source_file(filename: &str, text: &str) -> DailyReportInputFile {
    DailyReportInputFile {
        filename: filename.to_string(),
        bytes: encode_cp932(text),
        sd_relative_path: None,
        source_path: None,
    }
}

fn daily_report_preamble(date: &str) -> String {
    format!(
        "\"マシンNo.   \",\"01\",\"\",\"\"\r\n\"ファイル    \",\"synthetic\",\"\",\"\"\r\n\"モード      \",\"精算\",\"\",\"\"\r\n\"精算回数    \",\"0001\",\"\",\"\"\r\n\"日付        \",\"{date}\",\"\",\"\"\r\n\"時刻        \",\"12:34\",\"\",\"\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数/件数   \",\"金額        \"\r\n"
    )
}

fn z001_with_lines(date: &str, lines: &[(&str, &str, &str, &str)]) -> DailyReportInputFile {
    let mut text = daily_report_preamble(date);
    for (code, label, quantity_or_count, amount) in lines {
        text.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\"\r\n",
            code, label, quantity_or_count, amount
        ));
    }
    source_file("Z001_260321.CSV", &text)
}

fn z001(date: &str) -> DailyReportInputFile {
    z001_with_lines(
        date,
        &[("101", "総売", "8", "12000"), ("201", "純売", "7", "11000")],
    )
}

fn z002(date: &str) -> DailyReportInputFile {
    source_file(
        "Z002_260321.CSV",
        &format!(
            "{}\"01\",\"現金\",\"7\",\"11000\"\r\n\"03\",\"クレジット\",\"1\",\"1000\"\r\n",
            daily_report_preamble(date)
        ),
    )
}

fn z005_with_department(date: &str, department: &str) -> DailyReportInputFile {
    source_file(
        "Z005_260321.CSV",
        &format!(
            "\"マシンNo.   \",\"01\"\r\n\"ファイル    \",\"synthetic\"\r\n\"モード      \",\"精算\"\r\n\"精算回数    \",\"0001\"\r\n\"日付        \",\"{date}\"\r\n\"時刻        \",\"12:34\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数        \",\"金額        \"\r\n\"01\",\"{}\",\"4\",\"3000\"\r\n\"02\",\"毛糸\",\"5\",\"8000\"\r\n",
            department
        ),
    )
}

fn z005(date: &str) -> DailyReportInputFile {
    z005_with_department(date, "その他小物")
}

fn valid_files() -> Vec<DailyReportInputFile> {
    vec![z001("2026-03-21"), z002("2026-03-21"), z005("2026-03-21")]
}

fn count_rows(conn: &crate::db::DbConnection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
        row.get(0)
    })
    .unwrap()
}

fn write_surface(conn: &crate::db::DbConnection) -> [i64; 7] {
    [
        count_rows(conn, "daily_report_imports"),
        count_rows(conn, "daily_report_summary_lines"),
        count_rows(conn, "daily_report_payment_lines"),
        count_rows(conn, "daily_report_department_lines"),
        count_rows(conn, "sale_records"),
        count_rows(conn, "inventory_movements"),
        conn.query_row(
            "SELECT COALESCE(SUM(stock_quantity), 0) FROM products",
            [],
            |row| row.get(0),
        )
        .unwrap(),
    ]
}

fn seed_product_with_stock(conn: &crate::db::DbConnection, product_code: &str, stock: i64) {
    product_repo::insert_product(
        conn,
        &NewProduct {
            product_code: product_code.to_string(),
            jan_code: Some(format!(
                "4900000000{}",
                &product_code[product_code.len() - 3..]
            )),
            name: format!("合成商品 {}", product_code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: stock,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        },
    )
    .unwrap();
}

#[test]
fn test_daily_report_req401_parse_preview_happy_path() {
    // REQ-401 / BIZ-08: previewに対象日、bundle_hash、支払/部門、warningを返す
    let (_dir, conn) = setup_test_db();
    let result = parse_and_validate_daily_report(&conn, valid_files()).unwrap();

    assert_eq!(result.preview_data.file_info.report_date, "2026-03-21");
    assert_eq!(result.preview_data.file_info.bundle_hash.len(), 64);
    assert_eq!(result.preview_data.totals.gross_amount, Some(12000));
    assert_eq!(result.preview_data.totals.net_amount, Some(11000));
    assert_eq!(result.preview_data.payment_summary.len(), 2);
    assert_eq!(result.preview_data.department_summary.len(), 2);
    assert!(result.preview_data.warnings.is_empty());
    assert_eq!(
        result.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::NoDuplicate
    );
}

#[test]
fn test_daily_report_req401_parse_error_logs_parse_failed() {
    // REQ-401 / BIZ-08-D1: IO parse detailはdiagnostic専用、operator境界は汎用に保つ
    let (_dir, conn) = setup_test_db();
    let malformed_z001 = z001_with_lines("2026-03-21", &[("101", "総売", "not-a-number", "12000")]);
    let (result, diagnostic) = crate::test_tracing::capture(|| {
        parse_and_validate_daily_report(
            &conn,
            vec![malformed_z001, z002("2026-03-21"), z005("2026-03-21")],
        )
    });

    match result {
        Err(BizError::ImportError(message)) => {
            assert_eq!(message, "日報ファイルの解析に失敗しました");
        }
        other => panic!("expected generic import error, got {other:?}"),
    }
    assert!(
        diagnostic.contains("source_file=Some(Z001)"),
        "{diagnostic}"
    );
    assert!(diagnostic.contains("filename=None"), "{diagnostic}");
    assert!(diagnostic.contains("line_no=Some(9)"), "{diagnostic}");
    assert!(
        diagnostic.contains("error_type=invalid_number"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("error_message=Z001の個数/件数列を変換できません"),
        "{diagnostic}"
    );
    assert_eq!(count_rows(&conn, "daily_report_imports"), 0);
    let (operation_type, summary, detail_json): (String, String, Option<String>) = conn
        .query_row(
            "SELECT operation_type, summary, detail_json
             FROM operation_logs
             WHERE operation_type = 'daily_report_parse_failed'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(operation_type, "daily_report_parse_failed");
    assert_eq!(summary, "日報ファイルの解析に失敗しました");
    assert_eq!(detail_json, None);
}

#[test]
fn test_daily_report_req401_future_date_rejected() {
    // REQ-401 / BIZ-08: 未来日の日報はcommit不可
    let (_dir, conn) = setup_test_db();
    let result = parse_and_validate_daily_report(
        &conn,
        vec![z001("2099-01-01"), z002("2099-01-01"), z005("2099-01-01")],
    );

    assert!(matches!(result, Err(BizError::ImportError(_))));
}

#[test]
fn test_daily_report_req401_bundle_hash_stable_by_source_order() {
    // REQ-401 / BIZ-08: bundle_hashは入力順ではなくZ001→Z002→Z005順で安定する
    let (_dir, conn) = setup_test_db();
    let normal = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let reversed = parse_and_validate_daily_report(
        &conn,
        vec![z005("2026-03-21"), z002("2026-03-21"), z001("2026-03-21")],
    )
    .unwrap();

    assert_eq!(
        normal.preview_data.file_info.bundle_hash,
        reversed.preview_data.file_info.bundle_hash
    );
}

#[test]
fn test_daily_report_req401_duplicate_already_imported() {
    // REQ-401 / I-B1 / SPEC-SDI-D1: active同一bundleはAlreadyImported、rollback後は再preview可能。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let first =
        commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();

    let second = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    assert_eq!(
        second.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::AlreadyImported
    );

    rollback_daily_report_import(&mut conn, first.daily_report_import_id).unwrap();
    let retry = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    assert_eq!(
        retry.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::NoDuplicate
    );
}

#[test]
fn test_daily_report_req401_additional_confirmation_for_same_date_different_bundle() {
    // REQ-401 / I-W3 / SPEC-SDI-D3: 同日別bundleは全件・全files/totalsのordered snapshot付き追加確認。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let first =
        commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();

    let second_files = with_settlements(
        vec![
            z001_with_lines(
                "2026-03-21",
                &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
            ),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ],
        "0002",
    );
    let second = parse_and_validate_daily_report(&conn, second_files).unwrap();
    let second =
        commit_daily_report_import(&mut conn, second.cached_preview, true, _dir.path()).unwrap();
    let third = parse_and_validate_daily_report(
        &conn,
        with_settlements(
            vec![
                z001_with_lines(
                    "2026-03-21",
                    &[
                        ("101", "総売", "10", "14000"),
                        ("201", "純売", "9", "12500"),
                    ],
                ),
                z002("2026-03-21"),
                z005("2026-03-21"),
            ],
            "0003",
        ),
    )
    .unwrap();
    assert_eq!(
        third.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired
    );
    let summaries = &third.preview_data.duplicate_check.same_date_imports;
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].id, second.daily_report_import_id);
    assert_eq!(summaries[1].id, first.daily_report_import_id);
    assert_eq!(summaries[0].source_filenames.len(), 3);
    assert_eq!(summaries[0].gross_amount, Some(13000));
    assert_eq!(summaries[0].net_amount, Some(12000));
    assert!(!summaries[0].imported_at.is_empty());
}

#[test]
fn test_daily_report_req401_corrupt_same_date_source_files_metadata_fails_safe() {
    // REQ-401 / I-W3 / SPEC-SDI-D3: 既存metadata破損時はfilenameを捏造せずpreviewを安全側errorにする。
    let (_dir, conn) = setup_test_db();
    sales_repo::insert_daily_report_import(
        &conn,
        &sales_repo::NewDailyReportImport {
            report_date: "2026-03-21".to_string(),
            source_adapter: "casio_sr_s4000".to_string(),
            bundle_hash: "corrupt-existing-bundle".to_string(),
            source_files_json: "{not-json".to_string(),
            gross_amount: Some(1),
            net_amount: Some(1),
            status: "completed".to_string(),
            note: None,
            settlement_no: None,
        },
    )
    .unwrap();

    let error = parse_and_validate_daily_report(
        &conn,
        vec![
            z001_with_lines(
                "2026-03-21",
                &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
            ),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ],
    )
    .unwrap_err();
    assert!(
        matches!(error, BizError::ImportError(ref message) if message == "既存日報のファイル情報を読み取れません")
    );
}

#[test]
fn test_daily_report_req401_unmatched_department_warns_but_previews() {
    // REQ-401 / BIZ-08: 部門名未対応はwarningで、department_id=Noneのままpreview可能
    let (_dir, conn) = setup_test_db();
    let result = parse_and_validate_daily_report(
        &conn,
        vec![
            z001("2026-03-21"),
            z002("2026-03-21"),
            z005_with_department("2026-03-21", "未対応部門"),
        ],
    )
    .unwrap();

    assert!(result
        .preview_data
        .warnings
        .iter()
        .any(|w| w.code == "unmatched_department"
            && w.source_file == Some(DailyReportSourceKind::Z005)));
    assert!(result
        .preview_data
        .department_summary
        .iter()
        .any(|line| line.raw_department_name == "未対応部門" && line.department_id.is_none()));
}

#[test]
fn test_daily_report_req401_commit_inserts_parent_lines_and_log() {
    // REQ-401 / BIZ-08: commitは親/3系統明細/operation_logを作る
    let (_dir, mut conn) = setup_test_db();
    let parsed = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let result =
        commit_daily_report_import(&mut conn, parsed.cached_preview, false, _dir.path()).unwrap();

    assert_eq!(result.status, "completed");
    assert!(result.daily_report_import_id > 0);
    for table in [
        "daily_report_imports",
        "daily_report_summary_lines",
        "daily_report_payment_lines",
        "daily_report_department_lines",
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(count > 0, "{} should have rows", table);
    }
    let log_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'daily_report_import'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(log_count, 1);
}

#[test]
fn test_daily_report_req401_commit_same_date_is_insert_only() {
    // REQ-401 / I-B4 / SPEC-SDI-D1: 追加確定でも既存completedと全childを保持する。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let first_result =
        commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();

    let second_files = with_settlements(
        vec![
            z001_with_lines(
                "2026-03-21",
                &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
            ),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ],
        "0002",
    );
    let second = parse_and_validate_daily_report(&conn, second_files).unwrap();
    let second_result =
        commit_daily_report_import(&mut conn, second.cached_preview, true, _dir.path()).unwrap();

    assert_ne!(
        first_result.daily_report_import_id,
        second_result.daily_report_import_id
    );
    let old_status: String = conn
        .query_row(
            "SELECT status FROM daily_report_imports WHERE id = ?1",
            [first_result.daily_report_import_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(old_status, "completed");
    assert_eq!(first_result.gross_amount, Some(12000));
    assert_eq!(second_result.gross_amount, Some(13000));
    for (import_id, expected_max_summary) in [
        (first_result.daily_report_import_id, 12000_i64),
        (second_result.daily_report_import_id, 13000_i64),
    ] {
        let max_summary: i64 = conn
            .query_row(
                "SELECT MAX(amount) FROM daily_report_summary_lines WHERE daily_report_import_id = ?1",
                [import_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(max_summary, expected_max_summary);
        for table in [
            "daily_report_payment_lines",
            "daily_report_department_lines",
        ] {
            let child_count: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM {table} WHERE daily_report_import_id = ?1"),
                    [import_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(child_count > 0, "{table} child rows must remain");
        }
    }
}

#[test]
fn test_daily_report_req401_commit_does_not_write_sale_records_or_stock() {
    // REQ-401 / BIZ-08 / D-025: 日報commit単体はsale_records/movements/products.stock_quantityを汚染しない
    let (_dir, mut conn) = setup_test_db();
    seed_product_with_stock(&conn, "DR-001", 17);
    let parsed = parse_and_validate_daily_report(&conn, valid_files()).unwrap();

    commit_daily_report_import(&mut conn, parsed.cached_preview, false, _dir.path()).unwrap();

    let stock: i64 = conn
        .query_row(
            "SELECT stock_quantity FROM products WHERE product_code = 'DR-001'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stock, 17);
    assert_eq!(count_rows(&conn, "sale_records"), 0);
    assert_eq!(count_rows(&conn, "inventory_movements"), 0);
}

#[test]
fn test_daily_report_req401_commit_additional_unconfirmed_validation_failed() {
    // REQ-401 / I-B5 / SPEC-SDI-D3: 追加確認required + false はValidationFailed。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();

    let second_files = with_settlements(
        vec![
            z001_with_lines(
                "2026-03-21",
                &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
            ),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ],
        "0002",
    );
    let second = parse_and_validate_daily_report(&conn, second_files).unwrap();
    assert_eq!(
        second.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired
    );

    let before = write_surface(&conn);
    let result = commit_daily_report_import(&mut conn, second.cached_preview, false, _dir.path());
    assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_daily_report_req401_commit_no_duplicate_confirmed_validation_failed() {
    // REQ-401 / I-B5 / SPEC-SDI-D3: NoDuplicate + true は副作用なしValidationFailed。
    let (_dir, mut conn) = setup_test_db();
    let parsed = parse_and_validate_daily_report(&conn, valid_files()).unwrap();

    let before = write_surface(&conn);
    let result = commit_daily_report_import(&mut conn, parsed.cached_preview, true, _dir.path());

    assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    assert_eq!(count_rows(&conn, "daily_report_imports"), 0);
    assert_eq!(count_rows(&conn, "daily_report_payment_lines"), 0);
    assert_eq!(count_rows(&conn, "daily_report_department_lines"), 0);
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_daily_report_req401_commit_expired_preview_import_error() {
    // REQ-401 / BIZ-08: TTLを過ぎたpreviewはcommit不可
    let (_dir, mut conn) = setup_test_db();
    let mut parsed = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    parsed.cached_preview.created_at =
        Instant::now() - Duration::from_secs(constants::PREVIEW_CACHE_TTL_SECS + 1);

    let result = commit_daily_report_import(&mut conn, parsed.cached_preview, false, _dir.path());
    assert!(matches!(result, Err(BizError::ImportError(_))));
}

#[test]
fn test_daily_report_req401_stale_additional_preview_same_bundle_conflicts() {
    // REQ-401 / I-B2 / SPEC-SDI-D4: hash-first recheckでstale previewの同一bundle二重取込みを防ぐ。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();

    let second_files = with_settlements(
        vec![
            z001_with_lines(
                "2026-03-21",
                &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
            ),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ],
        "0002",
    );
    let stale_second = parse_and_validate_daily_report(&conn, second_files.clone()).unwrap();
    assert_eq!(
        stale_second.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired
    );

    let current_second = parse_and_validate_daily_report(&conn, second_files).unwrap();
    commit_daily_report_import(&mut conn, current_second.cached_preview, true, _dir.path())
        .unwrap();

    let before = write_surface(&conn);
    let result =
        commit_daily_report_import(&mut conn, stale_second.cached_preview, true, _dir.path());
    assert!(matches!(result, Err(BizError::IdempotencyConflict(_))));

    let completed_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM daily_report_imports WHERE report_date = '2026-03-21' AND status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(completed_count, 2);
    let total_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM daily_report_imports WHERE report_date = '2026-03-21'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(total_count, 2);
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_daily_report_req401_snapshot_change_requires_repreview_without_side_effects() {
    // REQ-401 / I-W5 / SPEC-SDI-D4: active ID snapshot差分は副作用なしで拒否する。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();

    let stale = parse_and_validate_daily_report(
        &conn,
        with_settlements(
            vec![
                z001_with_lines(
                    "2026-03-21",
                    &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
                ),
                z002("2026-03-21"),
                z005("2026-03-21"),
            ],
            "0002",
        ),
    )
    .unwrap();
    let concurrent = parse_and_validate_daily_report(
        &conn,
        with_settlements(
            vec![
                z001_with_lines(
                    "2026-03-21",
                    &[
                        ("101", "総売", "10", "14000"),
                        ("201", "純売", "9", "12500"),
                    ],
                ),
                z002("2026-03-21"),
                z005("2026-03-21"),
            ],
            "0003",
        ),
    )
    .unwrap();
    commit_daily_report_import(&mut conn, concurrent.cached_preview, true, _dir.path()).unwrap();
    let before: i64 = conn
        .query_row("SELECT COUNT(*) FROM daily_report_imports", [], |row| {
            row.get(0)
        })
        .unwrap();
    let before_surface = write_surface(&conn);

    let error =
        commit_daily_report_import(&mut conn, stale.cached_preview, true, _dir.path()).unwrap_err();
    assert!(
        matches!(error, BizError::ImportError(ref message) if message == "同日の取込み状況が変わりました。再度プレビューしてください")
    );
    let after: i64 = conn
        .query_row("SELECT COUNT(*) FROM daily_report_imports", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(after, before);
    assert_eq!(write_surface(&conn), before_surface);
}

#[test]
fn test_daily_report_req401_snapshot_replace_requires_repreview_without_side_effects() {
    // REQ-401 / I-W5 / SPEC-SDI-D4: 同件数のactive parent ID置換もexact snapshotで検出する。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let first =
        commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();
    let stale = parse_and_validate_daily_report(
        &conn,
        with_settlements(
            vec![
                z001_with_lines(
                    "2026-03-21",
                    &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
                ),
                z002("2026-03-21"),
                z005("2026-03-21"),
            ],
            "0002",
        ),
    )
    .unwrap();
    rollback_daily_report_import(&mut conn, first.daily_report_import_id).unwrap();
    let replacement = parse_and_validate_daily_report(
        &conn,
        with_settlements(
            vec![
                z001_with_lines(
                    "2026-03-21",
                    &[
                        ("101", "総売", "10", "14000"),
                        ("201", "純売", "9", "12500"),
                    ],
                ),
                z002("2026-03-21"),
                z005("2026-03-21"),
            ],
            "0003",
        ),
    )
    .unwrap();
    commit_daily_report_import(&mut conn, replacement.cached_preview, false, _dir.path()).unwrap();
    let before = write_surface(&conn);

    let error =
        commit_daily_report_import(&mut conn, stale.cached_preview, true, _dir.path()).unwrap_err();

    assert!(
        matches!(error, BizError::ImportError(ref message) if message == "同日の取込み状況が変わりました。再度プレビューしてください")
    );
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_daily_report_req401_commit_already_imported_conflict() {
    // REQ-401 / I-B5 / SPEC-SDI-D3: AlreadyImported previewは副作用なしIdempotencyConflict。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();
    let second = parse_and_validate_daily_report(&conn, valid_files()).unwrap();

    let before = write_surface(&conn);
    let result = commit_daily_report_import(&mut conn, second.cached_preview, false, _dir.path());
    assert!(matches!(result, Err(BizError::IdempotencyConflict(_))));
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_daily_report_req401_rollback_idempotent_and_no_stock_change() {
    // REQ-401 / I-B8 / SPEC-SDI-D2: rollback冪等再実行は親statusのみで在庫系列を変えない。
    let (_dir, mut conn) = setup_test_db();
    let parsed = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let committed =
        commit_daily_report_import(&mut conn, parsed.cached_preview, false, _dir.path()).unwrap();

    let first = rollback_daily_report_import(&mut conn, committed.daily_report_import_id).unwrap();
    let second = rollback_daily_report_import(&mut conn, committed.daily_report_import_id).unwrap();
    assert_eq!(first.status, "rolled_back");
    assert_eq!(second.status, "rolled_back");
    let rollback_log_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'daily_report_rollback'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(rollback_log_count, 1);

    for table in ["sale_records", "inventory_movements"] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "{} must stay empty", table);
    }
}

#[test]
fn test_daily_report_req401_rollback_keeps_same_date_sibling() {
    // REQ-401 / I-B8 / SPEC-SDI-D2,D7: 日報rollbackはper-importで同日siblingを保持する。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let first =
        commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();
    let second_files = with_settlements(
        vec![
            z001_with_lines(
                "2026-03-21",
                &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
            ),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ],
        "0002",
    );
    let second = parse_and_validate_daily_report(&conn, second_files).unwrap();
    let second =
        commit_daily_report_import(&mut conn, second.cached_preview, true, _dir.path()).unwrap();

    rollback_daily_report_import(&mut conn, first.daily_report_import_id).unwrap();

    let active = sales_repo::find_daily_report_imports_by_report_date(&conn, "2026-03-21").unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, second.daily_report_import_id);
    let aggregate = sales_repo::get_completed_daily_report_aggregate(&conn, "2026-03-21")
        .unwrap()
        .unwrap();
    assert_eq!(aggregate.source_import_count, 1);
    assert_eq!(aggregate.gross_amount, Some(13000));
    assert_eq!(aggregate.net_amount, Some(12000));
    assert!(aggregate
        .payment_lines
        .iter()
        .any(|line| line.payment_key == "cash" && line.amount == Some(11000)));

    rollback_daily_report_import(&mut conn, first.daily_report_import_id).unwrap();
    let repeated = sales_repo::get_completed_daily_report_aggregate(&conn, "2026-03-21")
        .unwrap()
        .unwrap();
    assert_eq!(repeated.gross_amount, Some(13000));
}

#[test]
fn test_daily_report_rollback_req401_logs_exact_selected_id_and_survives_log_failure() {
    // REQ-401 / I-B9 / SPEC-SDI-D2: daily log失敗もbusiness rollbackを戻さず、成功logは選択IDを記録する。
    let (_dir, mut conn) = setup_test_db();
    let first = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    let first =
        commit_daily_report_import(&mut conn, first.cached_preview, false, _dir.path()).unwrap();
    let second = parse_and_validate_daily_report(
        &conn,
        with_settlements(
            vec![
                z001_with_lines(
                    "2026-03-21",
                    &[("101", "総売", "9", "13000"), ("201", "純売", "8", "12000")],
                ),
                z002("2026-03-21"),
                z005("2026-03-21"),
            ],
            "0002",
        ),
    )
    .unwrap();
    let second =
        commit_daily_report_import(&mut conn, second.cached_preview, true, _dir.path()).unwrap();

    conn.execute_batch(
        "CREATE TRIGGER fail_daily_rollback_log
         BEFORE INSERT ON operation_logs
         WHEN NEW.operation_type = 'daily_report_rollback'
         BEGIN SELECT RAISE(FAIL, 'synthetic log failure'); END;",
    )
    .unwrap();
    let result = rollback_daily_report_import(&mut conn, first.daily_report_import_id).unwrap();
    assert_eq!(result.status, "rolled_back");
    let sibling = sales_repo::find_daily_report_import_by_id(&conn, second.daily_report_import_id)
        .unwrap()
        .unwrap();
    assert_eq!(sibling.status, "completed");

    conn.execute_batch("DROP TRIGGER fail_daily_rollback_log;")
        .unwrap();
    rollback_daily_report_import(&mut conn, second.daily_report_import_id).unwrap();
    let (summary, detail): (String, String) = conn
        .query_row(
            "SELECT summary, detail_json FROM operation_logs
             WHERE operation_type = 'daily_report_rollback' ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(summary.contains(&format!("ID {}", second.daily_report_import_id)));
    assert!(detail.contains(&format!(
        r#""daily_report_import_id":{}"#,
        second.daily_report_import_id
    )));
}

#[test]
fn test_daily_report_req401_missing_required_summary() {
    // REQ-401 / BIZ-08: gross_sales/net_salesがともに導出不可ならcommit不可
    let (_dir, conn) = setup_test_db();
    let files = vec![
        z001_with_lines("2026-03-21", &[("999", "客数", "8", "")]),
        z002("2026-03-21"),
        z005("2026-03-21"),
    ];
    let result = parse_and_validate_daily_report(&conn, files);
    assert!(matches!(result, Err(BizError::ImportError(_))));
}

#[test]
fn test_daily_report_req401_list_validation_and_result() {
    // REQ-401 / BIZ-08: listはpage/per_page境界を検証し、repo結果を返す
    let (_dir, mut conn) = setup_test_db();
    let parsed = parse_and_validate_daily_report(&conn, valid_files()).unwrap();
    commit_daily_report_import(&mut conn, parsed.cached_preview, false, _dir.path()).unwrap();

    let page = list_daily_report_imports(
        &conn,
        ListDailyReportImportsQuery {
            page: 1,
            per_page: 10,
            date_from: Some("2026-03-01".to_string()),
            date_to: Some("2026-03-31".to_string()),
            status: None,
        },
    )
    .unwrap();
    assert_eq!(page.total_count, 1);

    let invalid = list_daily_report_imports(
        &conn,
        ListDailyReportImportsQuery {
            page: 0,
            per_page: 10,
            date_from: None,
            date_to: None,
            status: None,
        },
    );
    assert!(matches!(invalid, Err(BizError::ValidationFailed(_))));
}

const SETTLEMENT_MISMATCH_MESSAGE: &str = "別の精算の日報ファイルが混ざっています。ファイル名の「Z001」「Z002」「Z005」より後ろが同じ 3 つを選び直してください。";

/// 合成ファイルの精算回数（`0001`）を別の値に置き換える
fn with_settlement(file: DailyReportInputFile, number: &str) -> DailyReportInputFile {
    let (text, _, _) = encoding_rs::SHIFT_JIS.decode(&file.bytes);
    let replaced = text.replace(
        "\"精算回数    \",\"0001\"",
        &format!("\"精算回数    \",\"{number}\""),
    );
    assert_ne!(replaced, text, "精算回数の行があること");
    source_file(&file.filename, &replaced)
}

/// 束の 3 本とも精算回数を替える（bytes の違う束ごとに別の精算にする。BIZ-08-D4 に当てない）
fn with_settlements(files: Vec<DailyReportInputFile>, number: &str) -> Vec<DailyReportInputFile> {
    files
        .into_iter()
        .map(|file| with_settlement(file, number))
        .collect()
}

fn assert_settlement_mismatch_rejected(files: Vec<DailyReportInputFile>) {
    let (_dir, conn) = setup_test_db();
    let (result, diagnostic) =
        crate::test_tracing::capture(|| parse_and_validate_daily_report(&conn, files));

    match result {
        Err(BizError::ImportError(message)) => assert_eq!(message, SETTLEMENT_MISMATCH_MESSAGE),
        other => panic!("expected settlement mismatch import error, got {other:?}"),
    }
    assert!(
        diagnostic.contains("error_type=settlement_mismatch"),
        "{diagnostic}"
    );
    let (summary, detail_json): (String, Option<String>) = conn
        .query_row(
            "SELECT summary, detail_json FROM operation_logs
             WHERE operation_type = 'daily_report_parse_failed'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(summary, SETTLEMENT_MISMATCH_MESSAGE);
    assert_eq!(detail_json, None);
    assert_eq!(count_rows(&conn, "daily_report_imports"), 0);
}

#[test]
fn test_daily_report_req401_settlement_mismatch_guides_reselection() {
    // REQ-401 / BIZ-08-D2: 精算回数の違う束は選び直しを案内する固定の文。raw detail は出さない
    assert_settlement_mismatch_rejected(vec![
        z001("2026-03-21"),
        with_settlement(z002("2026-03-21"), "0002"),
        with_settlement(z005("2026-03-21"), "0002"),
    ]);

    // 他の parse error と同時でも BIZ-08-D2 の文を優先する
    assert_settlement_mismatch_rejected(vec![
        z001_with_lines("2026-03-21", &[("101", "総売", "not-a-number", "12000")]),
        with_settlement(z002("2026-03-21"), "0002"),
        with_settlement(z005("2026-03-21"), "0002"),
    ]);
}

#[test]
fn test_daily_report_req401_decimal_quantity_preview_and_commit() {
    // REQ-401 / §37.2: preview は単位の数、cache と commit は 100 倍の整数（wire の f64 から戻さない）
    let (_dir, mut conn) = setup_test_db();
    let z005_decimal = source_file(
        "Z005_260321.CSV",
        &format!(
            "{}\"0001\",\"その他小物\",\"1.3\",\"3000\"\r\n",
            daily_report_preamble("2026-03-21")
        ),
    );
    let files = vec![
        z001_with_lines(
            "2026-03-21",
            &[
                ("0001", "総売", "1.3", "12000"),
                ("0002", "純売", "7", "11000"),
            ],
        ),
        z002("2026-03-21"),
        z005_decimal,
    ];
    let mut parsed = parse_and_validate_daily_report(&conn, files).unwrap();

    assert_eq!(
        parsed.preview_data.department_summary[0].quantity,
        Some(1.3)
    );
    assert_eq!(
        parsed.cached_preview.department_lines[0].quantity_hundredths,
        Some(130)
    );
    assert_eq!(
        parsed.cached_preview.summary_lines[0].quantity_hundredths,
        Some(130)
    );

    parsed.cached_preview.preview_data.department_summary[0].quantity = Some(9.99);
    commit_daily_report_import(&mut conn, parsed.cached_preview, false, _dir.path()).unwrap();

    let department: Option<i64> = conn
        .query_row(
            "SELECT quantity_hundredths FROM daily_report_department_lines
             WHERE raw_department_name = 'その他小物'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(department, Some(130));
    let gross: Option<i64> = conn
        .query_row(
            "SELECT quantity_hundredths FROM daily_report_summary_lines
             WHERE line_key = 'gross_sales'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(gross, Some(130));
}

// ---------------------------------------------------------------------------
// D-111: SD 直読み（BIZ-08-D3〜D6、§37.3 手順 1a・9、§37.4 手順 2a・2b・4a・6）
// 固定の文は設計正本（37 §37.3・§37.9、45 §45.7）から写す
// ---------------------------------------------------------------------------

use crate::io::register_sd::{RegisterSdError, RegisterSdRoot};
use crate::io::REGISTER_SD_FOLDER;
use std::path::{Path, PathBuf};

const D4_TEXT: &str = "同じ精算の日報が、別のファイルからすでに取り込まれています。二重に数えないため、取り込みません。取り込み直すときは、前の取込みを取り消してから読み直してください。";
const D6_TEXT: &str =
    "この精算の日報は、SD のファイルから精算回数を確かめられないため、SD からは取り込めません。";

/// 3 本とも同じ精算回数の合成の束（総売上で bytes を変える）
fn bundle(date: &str, settlement: &str, gross: &str) -> Vec<DailyReportInputFile> {
    with_settlements(
        vec![
            z001_with_lines(
                date,
                &[("101", "総売", "8", gross), ("201", "純売", "7", "11000")],
            ),
            z002(date),
            z005(date),
        ],
        settlement,
    )
}

/// Z002・Z005 の精算回数の行を外した束（Z001 だけが読める。settlement_no は None）
fn bundle_without_settlement(date: &str) -> Vec<DailyReportInputFile> {
    let mut files = bundle(date, "0005", "12000");
    for file in files.iter_mut().skip(1) {
        let (text, _, _) = encoding_rs::SHIFT_JIS.decode(&file.bytes);
        let replaced = text
            .replace("\"精算回数    \",\"0005\",\"\",\"\"\r\n", "")
            .replace("\"精算回数    \",\"0005\"\r\n", "");
        assert_ne!(replaced, text);
        *file = source_file(&file.filename, &replaced);
    }
    files
}

fn sd_root_at(base: &Path) -> RegisterSdRoot {
    let path = REGISTER_SD_FOLDER
        .iter()
        .fold(base.to_path_buf(), |path, part| path.join(part));
    std::fs::create_dir_all(path.join("XZ")).unwrap();
    RegisterSdRoot {
        path,
        volume_label: "T:".to_string(),
    }
}

fn sd_path(root: &RegisterSdRoot, relative: &str) -> PathBuf {
    relative
        .split('\\')
        .fold(root.path.clone(), |path, part| path.join(part))
}

fn put_sd(root: &RegisterSdRoot, relative: &str, bytes: &[u8]) {
    let path = sd_path(root, relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

/// 束の 3 本を `{folder}\Z00k_{name}` に置く
fn put_bundle(root: &RegisterSdRoot, folder: &str, name: &str, files: &[DailyReportInputFile]) {
    for (kind, file) in ["Z001", "Z002", "Z005"].iter().zip(files) {
        put_sd(root, &format!("{folder}\\{kind}_{name}"), &file.bytes);
    }
}

/// SD の候補から作る入力（§45.6b の snapshot と同じ形）
fn as_sd_files(
    files: Vec<DailyReportInputFile>,
    folder: &str,
    name: &str,
) -> Vec<DailyReportInputFile> {
    files
        .into_iter()
        .zip(["Z001", "Z002", "Z005"])
        .map(|(mut file, kind)| {
            file.filename = format!("{kind}_{name}");
            file.sd_relative_path = Some(format!("{folder}\\{kind}_{name}"));
            file
        })
        .collect()
}

fn scan_selected(
    conn: &crate::db::DbConnection,
    root: &RegisterSdRoot,
    today: chrono::NaiveDate,
) -> Result<DailyReportSdScanResult, BizError> {
    scan::scan_with_roots(
        conn,
        DailyReportSdSelection::Selected(root.path.clone()),
        today,
        || panic!("Selected では自動の探索を呼ばない"),
    )
}

fn day(text: &str) -> chrono::NaiveDate {
    chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
}

fn status_of(scan: &DailyReportSdScan) -> Vec<(String, DailyReportSdCandidateStatus)> {
    scan.candidates
        .iter()
        .map(|candidate| (candidate.source_filenames[0].clone(), candidate.status))
        .collect()
}

fn import(
    conn: &mut crate::db::DbConnection,
    dir: &Path,
    files: Vec<DailyReportInputFile>,
    confirmed: bool,
) -> i64 {
    let parsed = parse_and_validate_daily_report(conn, files).unwrap();
    commit_daily_report_import(conn, parsed.cached_preview, confirmed, dir)
        .unwrap()
        .daily_report_import_id
}

fn source_files_json(conn: &crate::db::DbConnection, id: i64) -> serde_json::Value {
    let text: String = conn
        .query_row(
            "SELECT source_files_json FROM daily_report_imports WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap();
    serde_json::from_str(&text).unwrap()
}

fn copies_under(app_data_dir: &Path) -> Vec<String> {
    let base = app_data_dir.join("pos-sources");
    let mut out = Vec::new();
    let mut stack = vec![base.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(
                    path.strip_prefix(app_data_dir)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    out.sort();
    out
}

#[test]
fn test_req401_scan_window_from_latest_import_or_30_days() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 2: completed の最大の対象日と today − 30 日の早い方
    let (dir, mut conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let today = day("2026-03-30");
    let from = |conn: &crate::db::DbConnection| {
        scan_selected(conn, &root, today).unwrap().scan.scanned_from
    };
    assert_eq!(from(&conn), "2026-02-28", "取込みなし");
    // rolled_back だけの日は数えない
    let rolled = import(
        &mut conn,
        dir.path(),
        bundle("2026-01-20", "0011", "12000"),
        false,
    );
    rollback_daily_report_import(&mut conn, rolled).unwrap();
    assert_eq!(from(&conn), "2026-02-28", "rolled_back だけ");
    import(
        &mut conn,
        dir.path(),
        bundle("2026-01-29", "0002", "12000"),
        false,
    );
    assert_eq!(from(&conn), "2026-01-29", "60 日前の取込み");
    // 最大が昨日なら today − 30 日（窓の始まりは today − 30 日より遅くならない）
    import(
        &mut conn,
        dir.path(),
        bundle("2026-03-29", "0003", "12000"),
        false,
    );
    assert_eq!(from(&conn), "2026-02-28", "最大は昨日");
}

#[test]
fn test_req401_scan_same_day_two_settlements_are_two_candidates() {
    // REQ-401 / BIZ-08-D3 / SD-08: 同じ日の 2 回の精算（接尾字が空白と A）は 2 つの候補
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "21 .CSV",
        &bundle("2026-03-21", "0005", "12000"),
    );
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "21A.CSV",
        &bundle("2026-03-21", "0006", "13000"),
    );
    let scan = scan_selected(&conn, &root, day("2026-03-25")).unwrap().scan;
    assert_eq!(
        status_of(&scan),
        vec![
            (
                "Z001_21A.CSV".to_string(),
                DailyReportSdCandidateStatus::NotImported
            ),
            (
                "Z001_21 .CSV".to_string(),
                DailyReportSdCandidateStatus::NotImported
            ),
        ]
    );
}

#[test]
fn test_req401_scan_same_bytes_in_both_areas_collapse() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 6: 同じ bytes は 1 つにまとめ、XZ の path を残す
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let files = bundle("2026-03-21", "0005", "12000");
    put_bundle(&root, "XZ\\2026\\03", "21 .CSV", &files);
    put_bundle(&root, "XZ_BKUP\\2026\\03", "21 _0002.CSV", &files);
    put_bundle(&root, "XZ_BKUP\\2026\\03", "21 _0001.CSV", &files);
    let result = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    assert_eq!(result.scan.candidates.len(), 1);
    let key = &result.scan.candidates[0].candidate_key;
    assert_eq!(
        result.scan.candidates[0].source_filenames[0],
        "Z001_21 .CSV"
    );
    let paths: Vec<_> = result.snapshot.files_by_candidate[key]
        .iter()
        .map(|file| file.sd_relative_path.clone().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "XZ\\2026\\03\\Z001_21 .CSV",
            "XZ\\2026\\03\\Z002_21 .CSV",
            "XZ\\2026\\03\\Z005_21 .CSV"
        ]
    );

    // XZ が無ければ XZ_BKUP の中の Z001 の相対 path の辞書順で最初の組
    let (dir2, conn2) = setup_test_db();
    let root2 = sd_root_at(&dir2.path().join("sd"));
    put_bundle(&root2, "XZ_BKUP\\2026\\03", "21 _0002.CSV", &files);
    put_bundle(&root2, "XZ_BKUP\\2026\\03", "21 _0001.CSV", &files);
    let result2 = scan_selected(&conn2, &root2, day("2026-03-25")).unwrap();
    assert_eq!(
        result2.scan.candidates[0].source_filenames[0],
        "Z001_21 _0001.CSV"
    );
}

#[test]
fn test_req401_scan_imported_by_hash_and_rolled_back_is_not_imported() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 7: 手で選んで取り込んだ同じ bytes は Imported、取消の後は NotImported
    let (dir, mut conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let files = bundle("2026-03-21", "0005", "12000");
    put_bundle(&root, "XZ_BKUP\\2026\\03", "21 _0001.CSV", &files);
    let id = import(&mut conn, dir.path(), files, false);
    let scan = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    assert_eq!(
        scan.scan.candidates[0].status,
        DailyReportSdCandidateStatus::Imported
    );
    assert!(scan.snapshot.files_by_candidate.is_empty());
    rollback_daily_report_import(&mut conn, id).unwrap();
    let scan = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    assert_eq!(
        scan.scan.candidates[0].status,
        DailyReportSdCandidateStatus::NotImported
    );
}

#[test]
#[cfg(unix)]
fn test_req401_scan_incomplete_group_is_not_read() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 4: Z002 の無い組は Incomplete で、組の file を読まない
    use std::os::unix::fs::PermissionsExt;
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let files = bundle("2026-03-21", "0005", "12000");
    put_sd(&root, "XZ\\2026\\03\\Z001_21 .CSV", &files[0].bytes);
    put_sd(&root, "XZ\\2026\\03\\Z005_21 .CSV", &files[2].bytes);
    for name in ["Z001_21 .CSV", "Z005_21 .CSV"] {
        let path = sd_path(&root, &format!("XZ\\2026\\03\\{name}"));
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o000)).unwrap();
    }
    let result = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    let candidate = &result.scan.candidates[0];
    assert_eq!(candidate.status, DailyReportSdCandidateStatus::Incomplete);
    assert_eq!(candidate.candidate_key, "sd:XZ\\2026\\03\\Z001_21 .CSV");
    assert_eq!(candidate.source_filenames, ["Z001_21 .CSV", "Z005_21 .CSV"]);
    assert!(result.snapshot.files_by_candidate.is_empty());
}

#[test]
fn test_req401_scan_shape_violation_is_unreadable() {
    // REQ-401 / BIZ-08-D3 / IO-09-D4: Z005 だけ BOM 付き・孤立 LF の組は Unreadable で snapshot に入らない
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let files = bundle("2026-03-21", "0005", "12000");
    let mut bom = files.clone();
    bom[2].bytes = [b"\xEF\xBB\xBF".as_slice(), &bom[2].bytes].concat();
    // Z005 だけ改行が LF（孤立 LF）
    let mut lf = bundle("2026-03-22", "0006", "12000");
    let (text, _, _) = encoding_rs::SHIFT_JIS.decode(&lf[2].bytes);
    lf[2] = source_file(&lf[2].filename, &text.replace("\r\n", "\n"));
    put_bundle(&root, "XZ\\2026\\03", "21 .CSV", &bom);
    put_bundle(&root, "XZ\\2026\\03", "22 .CSV", &lf);
    let result = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    assert_eq!(
        status_of(&result.scan),
        vec![
            (
                "Z001_22 .CSV".to_string(),
                DailyReportSdCandidateStatus::Unreadable
            ),
            (
                "Z001_21 .CSV".to_string(),
                DailyReportSdCandidateStatus::Unreadable
            ),
        ]
    );
    assert!(result.snapshot.files_by_candidate.is_empty());
}

#[test]
#[cfg(unix)]
fn test_req401_scan_io_failure_returns_error_without_candidates() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 5: 2 組目の読取りの IO error で全体を失敗にし、途中の候補を返さない
    use std::os::unix::fs::PermissionsExt;
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "21 .CSV",
        &bundle("2026-03-21", "0005", "12000"),
    );
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "22 .CSV",
        &bundle("2026-03-22", "0006", "12000"),
    );
    let path = sd_path(&root, "XZ\\2026\\03\\Z002_22 .CSV");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o000)).unwrap();
    match scan_selected(&conn, &root, day("2026-03-25")) {
        Err(BizError::ImportError(message)) => assert_eq!(
            message,
            "SD を読めませんでした。SD を差し直して、もう一度読んでください。"
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
#[cfg(unix)]
fn test_req401_scan_root_and_read_failures_use_fixed_texts() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 1・3: root と読取りの失敗は固定の文（推測で選ばない）
    use std::os::unix::fs::PermissionsExt;
    let (dir, conn) = setup_test_db();
    let today = day("2026-03-25");
    let text = |result: Result<DailyReportSdScanResult, BizError>| match result {
        Err(BizError::ImportError(message)) => message,
        other => panic!("{other:?}"),
    };
    let one = sd_root_at(&dir.path().join("one"));
    let two = sd_root_at(&dir.path().join("two"));
    let auto = |roots: Vec<RegisterSdRoot>| {
        scan::scan_with_roots(&conn, DailyReportSdSelection::Auto, today, move || {
            Ok(roots)
        })
    };
    assert_eq!(
        text(auto(vec![])),
        "SD が見つかりません。レジの SD をこの PC に差してから、もう一度読んでください。"
    );
    assert_eq!(
        text(auto(vec![one.clone(), two.clone()])),
        "SD が 2 枚以上見つかりました。売上を読む SD だけを差してください。"
    );
    assert!(auto(vec![one.clone()]).is_ok());
    std::fs::create_dir_all(dir.path().join("plain")).unwrap();
    assert_eq!(
        text(scan::scan_with_roots(
            &conn,
            DailyReportSdSelection::Selected(dir.path().join("plain")),
            today,
            || panic!("呼ばない"),
        )),
        "選んだ場所はレジの SD ではありません。SD の中の CASIO の folder か、SD そのものを選んでください。"
    );
    std::fs::remove_dir(two.path.join("XZ")).unwrap();
    assert_eq!(
        text(scan_selected(&conn, &two, today)),
        "SD に売上の folder（XZ）がありません。レジの SD か確かめてください。"
    );
    put_bundle(
        &one,
        "XZ\\2026\\03",
        "21 .CSV",
        &bundle("2026-03-21", "0005", "12000"),
    );
    let month = sd_path(&one, "XZ\\2026\\03");
    std::fs::set_permissions(&month, std::fs::Permissions::from_mode(0o000)).unwrap();
    let io_text = text(scan_selected(&conn, &one, today));
    std::fs::set_permissions(&month, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        io_text,
        "SD を読めませんでした。SD を差し直して、もう一度読んでください。"
    );

    let linked = sd_root_at(&dir.path().join("linked"));
    let outside = dir.path().join("outside-xz");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::remove_dir(linked.path.join("XZ")).unwrap();
    std::os::unix::fs::symlink(&outside, linked.path.join("XZ")).unwrap();
    assert_eq!(
        text(scan_selected(&conn, &linked, today)),
        "SD の中に、SD の外の場所を指す folder かファイル（リンク）があるため、読みませんでした。レジの SD そのものを差して読むか、リンクの無い SD の写しを選んでください。"
    );
    // 自動の探索の失敗も読めない文
    assert_eq!(
        text(scan::scan_with_roots(
            &conn,
            DailyReportSdSelection::Auto,
            today,
            || {
                Err(RegisterSdError::Io {
                    relative_path: String::new(),
                    kind: std::io::ErrorKind::Other,
                })
            }
        )),
        "SD を読めませんでした。SD を差し直して、もう一度読んでください。"
    );
}

#[test]
fn test_req401_scan_candidates_sorted_newest_first() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 8: 新しい日付順、同じ日は精算回数の大きい順
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "05 .CSV",
        &bundle("2026-03-05", "0004", "12000"),
    );
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "06 .CSV",
        &bundle("2026-03-06", "0005", "12000"),
    );
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "06A.CSV",
        &bundle("2026-03-06", "0006", "13000"),
    );
    let scan = scan_selected(&conn, &root, day("2026-03-25")).unwrap().scan;
    let names: Vec<_> = status_of(&scan).into_iter().map(|(name, _)| name).collect();
    assert_eq!(names, ["Z001_06A.CSV", "Z001_06 .CSV", "Z001_05 .CSV"]);
}

#[test]
fn test_req401_scan_same_settlement_other_bytes_is_same_settlement_imported() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 7 / BIZ-08-D4: 同じ日・同じ精算回数の別の bytes は SameSettlementImported
    let (dir, mut conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    import(
        &mut conn,
        dir.path(),
        bundle("2026-03-21", "0005", "12000"),
        false,
    );
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "21 .CSV",
        &bundle("2026-03-21", "0005", "12001"),
    );
    let result = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    assert_eq!(
        result.scan.candidates[0].status,
        DailyReportSdCandidateStatus::SameSettlementImported
    );
    assert!(result.snapshot.files_by_candidate.is_empty());
}

#[test]
fn test_req401_scan_snapshot_holds_only_not_imported() {
    // REQ-401 / BIZ-08-D3 §37.9 手順 9: snapshot は NotImported の束だけ
    let (dir, mut conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let imported = bundle("2026-03-10", "0011", "12000");
    import(&mut conn, dir.path(), imported.clone(), false);
    import(
        &mut conn,
        dir.path(),
        bundle("2026-03-11", "0002", "12000"),
        false,
    );
    put_bundle(&root, "XZ\\2026\\03", "10 .CSV", &imported);
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "11 .CSV",
        &bundle("2026-03-11", "0002", "12001"),
    );
    put_sd(&root, "XZ\\2026\\03\\Z001_12 .CSV", &imported[0].bytes);
    let mut unreadable = bundle("2026-03-13", "0003", "12000");
    unreadable[0].bytes.push(b'\n');
    put_bundle(&root, "XZ\\2026\\03", "13 .CSV", &unreadable);
    let fresh = bundle("2026-03-14", "0004", "12000");
    put_bundle(&root, "XZ\\2026\\03", "14 .CSV", &fresh);
    let result = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    use DailyReportSdCandidateStatus::*;
    assert_eq!(
        status_of(&result.scan)
            .into_iter()
            .map(|(_, status)| status)
            .collect::<Vec<_>>(),
        [
            NotImported,
            Unreadable,
            Incomplete,
            SameSettlementImported,
            Imported
        ]
    );
    let keys: Vec<_> = result.snapshot.files_by_candidate.keys().cloned().collect();
    assert_eq!(keys, [result.scan.candidates[0].candidate_key.clone()]);
    assert_eq!(result.snapshot.files_by_candidate[&keys[0]].len(), 3);
}

#[test]
fn test_req401_scan_bundle_without_settlement_no_is_unreadable() {
    // REQ-401 / BIZ-08-D6: 精算回数の読めない束は scan で Unreadable、snapshot に入らない
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    put_bundle(
        &root,
        "XZ\\2026\\03",
        "21 .CSV",
        &bundle_without_settlement("2026-03-21"),
    );
    let result = scan_selected(&conn, &root, day("2026-03-25")).unwrap();
    assert_eq!(
        result.scan.candidates[0].status,
        DailyReportSdCandidateStatus::Unreadable
    );
    assert_eq!(
        result.scan.candidates[0].candidate_key.len(),
        64,
        "bundle_hash を作れた束"
    );
    assert!(result.snapshot.files_by_candidate.is_empty());
}

#[test]
fn test_req401_sd_bundle_without_settlement_no_is_rejected() {
    // REQ-401 / BIZ-08-D6 §37.3 手順 4: SD の経路の None の束は preview で止める
    let (_dir, conn) = setup_test_db();
    let files = as_sd_files(
        bundle_without_settlement("2026-03-21"),
        "XZ\\2026\\03",
        "21 .CSV",
    );
    match parse_and_validate_daily_report_with_sd_lookup(&conn, files, || Ok(vec![])) {
        Err(BizError::ImportError(message)) => assert_eq!(message, D6_TEXT),
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_req401_manual_bundle_without_settlement_no_still_previews() {
    // REQ-401 / BIZ-08-D6: SD の root の外の file を手で選んだ None の束は今どおり preview できる
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let pc = dir.path().join("EcrDatas");
    std::fs::create_dir_all(&pc).unwrap();
    let files: Vec<_> = bundle_without_settlement("2026-03-21")
        .into_iter()
        .map(|mut file| {
            let path = pc.join(&file.filename);
            std::fs::write(&path, &file.bytes).unwrap();
            file.source_path = Some(path);
            file
        })
        .collect();
    let result =
        parse_and_validate_daily_report_with_sd_lookup(&conn, files, || Ok(vec![root])).unwrap();
    assert_eq!(result.cached_preview.settlement_no, None);
    assert!(result.cached_preview.sd_source_files.is_empty());
}

/// SD の root の下に束を置き、その path を source_path に持つ手で選んだ入力
fn manual_sd_files(
    root: &RegisterSdRoot,
    files: Vec<DailyReportInputFile>,
) -> Vec<DailyReportInputFile> {
    files
        .into_iter()
        .zip(["Z001", "Z002", "Z005"])
        .map(|(mut file, kind)| {
            let relative = format!("XZ\\2026\\03\\{kind}_21 .CSV");
            put_sd(root, &relative, &file.bytes);
            file.filename = format!("{kind}_21 .CSV");
            file.source_path = Some(sd_path(root, &relative));
            file
        })
        .collect()
}

#[test]
fn test_req401_manual_sd_bundle_without_settlement_no_is_rejected() {
    // REQ-401 / BIZ-08-D6 / §37.3 手順 1a (ii): 手で選んだ SD の上の None の束も止める
    let (dir, conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let files = manual_sd_files(&root, bundle_without_settlement("2026-03-21"));
    let roots = vec![root];
    match parse_and_validate_daily_report_with_sd_lookup(&conn, files, || Ok(roots)) {
        Err(BizError::ImportError(message)) => assert_eq!(message, D6_TEXT),
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_req401_sd_root_lookup_failure_stops_preview() {
    // REQ-401 / §37.3 手順 1a (i): SD 上かを確かめられないときは PC の file として通さない
    let (dir, conn) = setup_test_db();
    let pc = dir.path().join("pc");
    std::fs::create_dir_all(&pc).unwrap();
    let files: Vec<_> = bundle("2026-03-21", "0005", "12000")
        .into_iter()
        .map(|mut file| {
            let path = pc.join(&file.filename);
            std::fs::write(&path, &file.bytes).unwrap();
            file.source_path = Some(path);
            file
        })
        .collect();
    let result = parse_and_validate_daily_report_with_sd_lookup(&conn, files, || {
        Err(RegisterSdError::Io {
            relative_path: String::new(),
            kind: std::io::ErrorKind::Other,
        })
    });
    match result {
        Err(BizError::ImportError(message)) => assert_eq!(
            message,
            "選んだファイルがレジの SD の上にあるかを確かめられませんでした。もう一度選んでください。"
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_req401_manual_sd_file_removed_before_preview_is_rejected() {
    // REQ-401 / §37.3 手順 1a (iii): SD を抜いた後の SD の file を PC の file として通さない
    let (dir, conn) = setup_test_db();
    let sd = dir.path().join("sd");
    let root = sd_root_at(&sd);
    let files = manual_sd_files(&root, bundle("2026-03-21", "0005", "12000"));
    std::fs::remove_dir_all(&sd).unwrap();
    match parse_and_validate_daily_report_with_sd_lookup(&conn, files, || Ok(vec![])) {
        Err(BizError::ImportError(message)) => assert_eq!(
            message,
            "選んだファイルが見つからなくなりました。SD から選んだときは、SD をこの PC に差したまま、もう一度選んでください。"
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_req401_commit_rejects_sd_cache_without_settlement_no() {
    // REQ-401 / BIZ-08-D6 §37.4 手順 2a: preview を経ない cache でも、写しも DB も変えずに止める
    let (dir, mut conn) = setup_test_db();
    let files = as_sd_files(
        bundle("2026-03-21", "0005", "12000"),
        "XZ\\2026\\03",
        "21 .CSV",
    );
    let mut cached = parse_and_validate_daily_report(&conn, files)
        .unwrap()
        .cached_preview;
    assert!(!cached.sd_source_files.is_empty());
    cached.settlement_no = None;
    let app = dir.path().join("app");
    let before = write_surface(&conn);
    match commit_daily_report_import(&mut conn, cached, false, &app) {
        Err(BizError::ImportError(message)) => assert_eq!(message, D6_TEXT),
        other => panic!("{other:?}"),
    }
    assert_eq!(write_surface(&conn), before);
    assert!(copies_under(&app).is_empty());
}

#[test]
fn test_req401_same_settlement_different_bytes_is_rejected() {
    // REQ-401 / BIZ-08-D4 §37.3 手順 8: 同じ日・同じ精算回数の別の bytes は追加確認で通さない
    let (dir, mut conn) = setup_test_db();
    import(
        &mut conn,
        dir.path(),
        bundle("2026-03-21", "0005", "12000"),
        false,
    );
    match parse_and_validate_daily_report(&conn, bundle("2026-03-21", "0005", "12001")) {
        Err(BizError::ImportError(message)) => assert_eq!(message, D4_TEXT),
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_req401_same_date_other_settlement_still_confirms() {
    // REQ-401 / BIZ-08-D4: 同じ日の別の精算（精算回数 6）は今どおり追加確認
    let (dir, mut conn) = setup_test_db();
    import(
        &mut conn,
        dir.path(),
        bundle("2026-03-21", "0005", "12000"),
        false,
    );
    let preview =
        parse_and_validate_daily_report(&conn, bundle("2026-03-21", "0006", "13000")).unwrap();
    assert_eq!(
        preview.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired
    );
}

#[test]
fn test_req401_null_settlement_no_is_not_compared() {
    // REQ-401 / BIZ-08-D4: NULL の既存行と None の束は照合しない
    let (dir, mut conn) = setup_test_db();
    import(
        &mut conn,
        dir.path(),
        bundle_without_settlement("2026-03-21"),
        false,
    );
    let preview =
        parse_and_validate_daily_report(&conn, bundle("2026-03-21", "0005", "13000")).unwrap();
    assert_eq!(
        preview.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired
    );
    let (dir2, mut conn2) = setup_test_db();
    import(
        &mut conn2,
        dir2.path(),
        bundle("2026-03-21", "0005", "12000"),
        false,
    );
    let mut none_bundle = bundle_without_settlement("2026-03-21");
    none_bundle[0] = with_settlement(
        z001_with_lines(
            "2026-03-21",
            &[("101", "総売", "8", "14000"), ("201", "純売", "7", "11000")],
        ),
        "0005",
    );
    let preview = parse_and_validate_daily_report(&conn2, none_bundle).unwrap();
    assert_eq!(preview.cached_preview.settlement_no, None);
    assert_eq!(
        preview.preview_data.duplicate_check.status,
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired
    );
}

#[test]
fn test_req401_commit_rechecks_same_settlement_in_tx() {
    // REQ-401 / BIZ-08-D4 §37.4 手順 4a: preview の後に入った同じ精算を TX の中で止める
    let (dir, mut conn) = setup_test_db();
    let existing = sales_repo::insert_daily_report_import(
        &conn,
        &sales_repo::NewDailyReportImport {
            report_date: "2026-03-21".to_string(),
            source_adapter: "casio_sr_s4000".to_string(),
            bundle_hash: "H1".to_string(),
            source_files_json: "[]".to_string(),
            gross_amount: Some(1),
            net_amount: Some(1),
            status: "completed".to_string(),
            note: None,
            settlement_no: Some(5),
        },
    )
    .unwrap();
    // preview の後に入った形: preview の時点の照合を通った cache を直接作る
    let (dir_p, conn_p) = setup_test_db();
    let mut cached =
        parse_and_validate_daily_report(&conn_p, bundle("2026-03-21", "0005", "12000"))
            .unwrap()
            .cached_preview;
    drop((dir_p, conn_p));
    cached.preview_data.duplicate_check.status =
        DailyReportDuplicateStatus::AdditionalImportConfirmationRequired;
    cached.active_same_date_import_ids = vec![existing];
    let before = write_surface(&conn);
    match commit_daily_report_import(&mut conn, cached, true, dir.path()) {
        Err(BizError::ImportError(message)) => assert_eq!(message, D4_TEXT),
        other => panic!("{other:?}"),
    }
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_req401_commit_stores_settlement_no() {
    // REQ-401 / BIZ-08-D4 §37.4 手順 6: 束の精算回数を保存する
    let (dir, mut conn) = setup_test_db();
    let id = import(
        &mut conn,
        dir.path(),
        bundle("2026-03-21", "0007", "12000"),
        false,
    );
    let stored: Option<i64> = conn
        .query_row(
            "SELECT settlement_no FROM daily_report_imports WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, Some(7));
}

#[test]
#[cfg(unix)]
fn test_req401_copy_failure_aborts_commit() {
    // REQ-401 / BIZ-08-D5 §37.4 手順 2b: 写しを書けなければ取り込まない
    use std::os::unix::fs::PermissionsExt;
    let (dir, mut conn) = setup_test_db();
    let files = as_sd_files(
        bundle("2026-03-21", "0005", "12000"),
        "XZ\\2026\\03",
        "21 .CSV",
    );
    let cached = parse_and_validate_daily_report(&conn, files)
        .unwrap()
        .cached_preview;
    let app = dir.path().join("readonly-app");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::set_permissions(&app, std::fs::Permissions::from_mode(0o555)).unwrap();
    let before = write_surface(&conn);
    let result = commit_daily_report_import(&mut conn, cached, false, &app);
    std::fs::set_permissions(&app, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        matches!(result, Err(BizError::SourceCopyFailed(_))),
        "{result:?}"
    );
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_req401_cached_preview_holds_only_sd_files() {
    // REQ-401 / §37.3 手順 9: sd_source_files は SD の file だけ（source・相対 path・bytes）
    let (dir, conn) = setup_test_db();
    // (a) SD の候補の 3 本
    let files = as_sd_files(
        bundle("2026-03-21", "0005", "12000"),
        "XZ\\2026\\03",
        "21 .CSV",
    );
    let cached = parse_and_validate_daily_report(&conn, files.clone())
        .unwrap()
        .cached_preview;
    let got: Vec<_> = cached
        .sd_source_files
        .iter()
        .map(|file| {
            (
                file.source,
                file.sd_relative_path.clone(),
                file.bytes.clone(),
            )
        })
        .collect();
    let expected: Vec<_> = [
        crate::io::daily_report_parser::DailyReportSourceKind::Z001,
        crate::io::daily_report_parser::DailyReportSourceKind::Z002,
        crate::io::daily_report_parser::DailyReportSourceKind::Z005,
    ]
    .into_iter()
    .zip(&files)
    .map(|(source, file)| {
        (
            source,
            file.sd_relative_path.clone().unwrap(),
            file.bytes.clone(),
        )
    })
    .collect();
    assert_eq!(got, expected);

    // (b) root の外の 3 本は空、(c) 1 本だけ root の下なら 1 件
    let root = sd_root_at(&dir.path().join("sd"));
    let pc = dir.path().join("pc");
    std::fs::create_dir_all(&pc).unwrap();
    let mut files: Vec<_> = bundle("2026-03-21", "0005", "12000")
        .into_iter()
        .map(|mut file| {
            let path = pc.join(&file.filename);
            std::fs::write(&path, &file.bytes).unwrap();
            file.source_path = Some(path);
            file
        })
        .collect();
    let roots = vec![root.clone()];
    let cached =
        parse_and_validate_daily_report_with_sd_lookup(&conn, files.clone(), || Ok(roots.clone()))
            .unwrap()
            .cached_preview;
    assert!(cached.sd_source_files.is_empty());
    put_sd(&root, "XZ\\2026\\03\\Z002_260321.CSV", &files[1].bytes);
    files[1].source_path = Some(sd_path(&root, "XZ\\2026\\03\\Z002_260321.CSV"));
    let cached = parse_and_validate_daily_report_with_sd_lookup(&conn, files, || Ok(roots))
        .unwrap()
        .cached_preview;
    assert_eq!(cached.sd_source_files.len(), 1);
    assert_eq!(
        cached.sd_source_files[0].sd_relative_path,
        "XZ\\2026\\03\\Z002_260321.CSV"
    );
}

#[test]
fn test_req401_rejected_in_tx_leaves_copy_but_no_copy_path() {
    // REQ-401 / BIZ-08-D5 の保証の範囲: TX の中で拒んだ束の写しは残りうるが、DB に copy_path は増えない
    let (dir, mut conn) = setup_test_db();
    let files = as_sd_files(
        bundle("2026-03-21", "0005", "12000"),
        "XZ\\2026\\03",
        "21 .CSV",
    );
    let cached = parse_and_validate_daily_report(&conn, files.clone())
        .unwrap()
        .cached_preview;
    // 同じ bundle_hash の completed を先に入れる（PC から取り込んだ形）
    import(
        &mut conn,
        dir.path(),
        bundle("2026-03-21", "0005", "12000"),
        false,
    );
    let app = dir.path().join("app");
    let before = write_surface(&conn);
    let result = commit_daily_report_import(&mut conn, cached, false, &app);
    assert!(
        matches!(result, Err(BizError::IdempotencyConflict(_))),
        "{result:?}"
    );
    assert_eq!(write_surface(&conn), before);
    let all: Vec<String> = conn
        .prepare("SELECT source_files_json FROM daily_report_imports")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|row| row.unwrap())
        .collect();
    assert!(
        all.iter().all(|json| !json.contains("copy_path")),
        "{all:?}"
    );
}

#[test]
fn test_req401_commit_records_copy_paths() {
    // REQ-401 / BIZ-08-D5 §37.4 手順 6: source_files_json に sd_relative_path と copy_path、写しは束と同じ bytes
    let (dir, mut conn) = setup_test_db();
    let files = as_sd_files(
        bundle("2026-03-21", "0005", "12000"),
        "XZ\\2026\\03",
        "21 .CSV",
    );
    let app = dir.path().join("app");
    let id = import(&mut conn, &app, files.clone(), false);
    let json = source_files_json(&conn, id);
    let items = json.as_array().unwrap();
    assert_eq!(items.len(), 3);
    for (item, file) in items.iter().zip(&files) {
        assert_eq!(
            item["sd_relative_path"],
            file.sd_relative_path.clone().unwrap().as_str()
        );
        let copy_path = item["copy_path"].as_str().unwrap();
        assert_eq!(std::fs::read(app.join(copy_path)).unwrap(), file.bytes);
        // 既存の 4 つの key は変えない
        for key in ["source", "filename", "file_hash", "size_bytes"] {
            assert!(item.get(key).is_some(), "{key}");
        }
    }
    assert_eq!(
        items[0]["copy_path"],
        "pos-sources/casio-sr-s4000/sd/XZ/2026/03/Z001_21 .CSV"
    );
}

#[test]
fn test_req401_pc_bundle_has_no_copy() {
    // REQ-401 / BIZ-08-D5: PC 上の file を手で選んだ束は写さない
    let (dir, mut conn) = setup_test_db();
    let app = dir.path().join("app");
    let id = import(
        &mut conn,
        &app,
        bundle("2026-03-21", "0005", "12000"),
        false,
    );
    assert!(copies_under(&app).is_empty());
    let json = source_files_json(&conn, id);
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item.get("copy_path").is_none()));
}

#[test]
fn test_req401_manual_sd_files_are_copied() {
    // REQ-401 / BIZ-08-D5 / §37.3 手順 1a (ii): 手で選んだ SD の上の file は写す
    let (dir, mut conn) = setup_test_db();
    let root = sd_root_at(&dir.path().join("sd"));
    let files = manual_sd_files(&root, bundle("2026-03-21", "0005", "12000"));
    let roots = vec![root];
    let cached = parse_and_validate_daily_report_with_sd_lookup(&conn, files, || Ok(roots))
        .unwrap()
        .cached_preview;
    let app = dir.path().join("app");
    let id = commit_daily_report_import(&mut conn, cached, false, &app)
        .unwrap()
        .daily_report_import_id;
    assert_eq!(
        copies_under(&app),
        [
            "pos-sources/casio-sr-s4000/sd/XZ/2026/03/Z001_21 .CSV",
            "pos-sources/casio-sr-s4000/sd/XZ/2026/03/Z002_21 .CSV",
            "pos-sources/casio-sr-s4000/sd/XZ/2026/03/Z005_21 .CSV",
        ]
    );
    let json = source_files_json(&conn, id);
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item.get("copy_path").is_some() && item.get("sd_relative_path").is_some()));
}

#[test]
fn test_req401_first_commit_creates_copy_directory() {
    // REQ-401 / BIZ-08-D5 / IO-10 §29.8.3 手順 3: pos-sources の無い app data でも写しと行ができる
    let (dir, mut conn) = setup_test_db();
    let app = dir.path().join("empty-app");
    std::fs::create_dir_all(&app).unwrap();
    let files = as_sd_files(
        bundle("2026-03-21", "0005", "12000"),
        "XZ\\2026\\03",
        "21 .CSV",
    );
    import(&mut conn, &app, files, false);
    assert_eq!(copies_under(&app).len(), 3);
    assert_eq!(count_rows(&conn, "daily_report_imports"), 1);
}

#[test]
fn test_req401_source_filenames_accepts_missing_copy_fields() {
    // REQ-401 / §37.4 手順 6: copy_path の無い既存の source_files_json でも同日の summary を読める
    let (dir, mut conn) = setup_test_db();
    import(
        &mut conn,
        dir.path(),
        bundle("2026-03-21", "0005", "12000"),
        false,
    );
    let files = as_sd_files(
        bundle("2026-03-21", "0006", "13000"),
        "XZ\\2026\\03",
        "21A.CSV",
    );
    import(&mut conn, &dir.path().join("app"), files, true);
    let preview =
        parse_and_validate_daily_report(&conn, bundle("2026-03-21", "0007", "14000")).unwrap();
    let summaries = &preview.preview_data.duplicate_check.same_date_imports;
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].source_filenames[0], "Z001_21A.CSV");
    assert_eq!(summaries[1].source_filenames[0], "Z001_260321.CSV");
}

#[test]
#[ignore = "INVENTORY_L3_SD_TREE_OUT に出力先を与えて L3 の前に手元で実行する"]
fn write_l3_register_sd_tree() {
    // L3-1・L3-3 の合成の SD の tree。日付は生成した日の前日・前々日（scan の窓の中にする）
    let out = std::env::var_os("INVENTORY_L3_SD_TREE_OUT").expect(
        "INVENTORY_L3_SD_TREE_OUT に出力先の folder を与えてください（何も書いていません）",
    );
    let root = sd_root_at(Path::new(&out));
    let today = chrono::Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    let before = today - chrono::Duration::days(2);
    let folder = |date: chrono::NaiveDate| date.format("XZ\\%Y\\%m").to_string();
    let name = |date: chrono::NaiveDate, suffix: char| format!("{}{suffix}.CSV", date.format("%d"));
    let text = |date: chrono::NaiveDate| date.format("%Y-%m-%d").to_string();
    // L3-1: 3 本そろい、精算回数あり
    put_bundle(
        &root,
        &folder(yesterday),
        &name(yesterday, ' '),
        &bundle(&text(yesterday), "0101", "12000"),
    );
    // L3-3: Z002 の無い組
    let incomplete = bundle(&text(before), "0102", "12000");
    put_sd(
        &root,
        &format!("{}\\Z001_{}", folder(before), name(before, 'A')),
        &incomplete[0].bytes,
    );
    put_sd(
        &root,
        &format!("{}\\Z005_{}", folder(before), name(before, 'A')),
        &incomplete[2].bytes,
    );
    // L3-3: Z005 だけ BOM 付きの組
    let mut bom = bundle(&text(before), "0103", "13000");
    bom[2].bytes = [b"\xEF\xBB\xBF".as_slice(), &bom[2].bytes].concat();
    put_bundle(&root, &folder(before), &name(before, 'B'), &bom);
    println!("合成の SD の tree を書きました: {}", root.path.display());
}
