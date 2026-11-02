use crate::biz::csv_import_service::test_support::*;
use crate::biz::csv_import_service::*;
use crate::biz::BizError;
use crate::db::test_support::setup_test_db;
use crate::db::{product_repo, sales_repo};

fn write_surface(conn: &crate::db::DbConnection) -> [i64; 4] {
    [
        conn.query_row("SELECT COUNT(*) FROM csv_imports", [], |row| row.get(0))
            .unwrap(),
        conn.query_row("SELECT COUNT(*) FROM sale_records", [], |row| row.get(0))
            .unwrap(),
        conn.query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
            row.get(0)
        })
        .unwrap(),
        conn.query_row(
            "SELECT COALESCE(SUM(stock_quantity), 0) FROM products",
            [],
            |row| row.get(0),
        )
        .unwrap(),
    ]
}

// --- commit_csv_import テスト ---

#[test]
fn test_commit_req401_normal_flow() {
    // REQ-401: CSV取込み
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    create_test_product_with_jan(&conn, "TEST-002", "4912345678902", 5, true);
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "商品A", 3, 900),
            ("4912345678902", "商品B", 1, 200),
        ],
    );
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260321");
    let cached = build_cached(pv);

    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();

    assert_eq!(result.status, sales_repo::CsvImportStatus::Completed);
    assert_eq!(result.total_items, 2);
    assert_eq!(result.total_amount, 1100);
    assert_eq!(result.skipped_count, 0);

    // DB検証: 在庫が減っている
    let p1 = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p1.product.stock_quantity, 7); // 10 - 3
    let p2 = product_repo::find_by_product_code(&conn, "TEST-002")
        .unwrap()
        .unwrap();
    assert_eq!(p2.product.stock_quantity, 4); // 5 - 1
}

#[test]
fn test_commit_req401_toctou_check() {
    // REQ-401 / I-B2 / SPEC-SDI-D4: hash recheck はsnapshot mismatchより先にhard blockする。
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 3, 900)]);
    let pv = parse_and_build_cache(&conn, bytes.clone(), "Z004_260321");
    let file_hash = pv.preview_data.file_info.file_hash.clone();

    // プレビュー後にDBに同じhashを挿入（他の操作がcommitした状況をシミュレート）
    sales_repo::insert_csv_import(
        &conn,
        &sales_repo::NewCsvImport {
            filename: "sneaky".to_string(),
            settlement_date: "2026-03-21".to_string(),
            file_hash,
            total_items: 1,
            total_amount: 900,
            skipped_count: 0,
            status: "completed".to_string(),
            machine_no: "0001".to_string(),
            settlement_no: 1,
        },
    )
    .unwrap();

    let cached = build_cached(pv);
    let before = write_surface(&conn);
    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::ImportError(msg) => assert!(msg.contains("取込み済み")),
        e => panic!("Expected ImportError, got {:?}", e),
    }
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_commit_req401_pos_stock_sync_false() {
    // REQ-401: CSV取込み
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, false); // pos_stock_sync=false
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "生地A", 3, 900)]);
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260321");
    let cached = build_cached(pv);

    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    assert_eq!(result.status, sales_repo::CsvImportStatus::Completed);
    assert_eq!(result.total_items, 1);

    // DB検証: 在庫は動かない
    let p = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, 10); // 変わらず
}

#[test]
fn test_commit_req401_negative_stock_warning() {
    // REQ-401: CSV取込み
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 2, true); // 在庫2
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "商品A", 5, 1500), // 5個販売 → 在庫-3
        ],
    );
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260321");
    let cached = build_cached(pv);

    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    // 処理は完了する（INV-3: 負在庫は警告のみ）
    assert_eq!(result.status, sales_repo::CsvImportStatus::Completed);

    // DB検証: 在庫がマイナス
    let p = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, -3); // 2 - 5
}

#[test]
fn test_commit_req401_partial_with_errors() {
    // REQ-401: CSV取込み
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    // 1件マッチ + 1件アンマッチ
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "商品A", 3, 900),
            ("9999999999999", "未登録商品", 1, 100),
        ],
    );
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260321");
    let cached = build_cached(pv);

    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    assert_eq!(result.status, sales_repo::CsvImportStatus::CompletedPartial);
    assert_eq!(result.total_items, 1);
    assert_eq!(result.skipped_count, 1);
}

#[test]
fn test_commit_req401_sign_flip_inv1() {
    // REQ-401: CSV取込み
    // INV-1: quantity=3（売上帳票視点）→ inventory_quantity=-3（在庫視点）
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 100, true);
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 3, 900)]);
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260321");
    let cached = build_cached(pv);

    commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();

    // DB検証: 在庫が3減っている（100 - 3 = 97）
    let p = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, 97);
}

// --- 入力の上限（共通規則 SPEC-UNIT-D11 の続き、X2・X3） ---

#[test]
fn test_preview_req401_pos_quantity_limit_is_row_error() {
    // REQ-401 / X2: POS 由来の数量は絶対値 9999999 まで。越える行は invalid_number の行の error で、
    // 在庫を動かさない行（pos_stock_sync=false）も同じ検査を通る。ほかの行は取り込む
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "X2-NOSYNC", "4912345678901", 10, false);
    create_test_product_with_jan(&conn, "X2-SYNC", "4912345678902", 10, true);
    create_test_product_with_jan(&conn, "X2-EDGE", "4912345678903", 10, false);
    create_test_product_with_jan(&conn, "X2-RETURN", "4912345678904", 10, false);
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678901", "連動しない商品", 10_000_000, 100),
            ("4912345678902", "連動する商品", -10_000_000, -100),
            ("4912345678903", "上限の端", 9_999_999, 100),
            ("4912345678904", "上限の端の返品", -9_999_999, -100),
        ],
    );
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260321");
    let errors: Vec<(usize, CsvImportErrorType, &str)> = pv
        .error_rows
        .iter()
        .map(|row| (row.line_no, row.error_type, row.error_message.as_str()))
        .collect();
    assert_eq!(errors.len(), 2, "{errors:?}");
    for (_, error_type, message) in &errors {
        assert_eq!(*error_type, CsvImportErrorType::InvalidNumber);
        assert_eq!(*message, "数量が大きすぎます");
    }
    assert_eq!(pv.matched_rows.len(), 2);

    commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: build_cached(pv),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();

    let mut statement = conn
        .prepare("SELECT product_code, quantity FROM sale_records ORDER BY product_code")
        .unwrap();
    let saved: Vec<(String, i64)> = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        saved,
        vec![
            ("X2-EDGE".to_string(), 9_999_999),
            ("X2-RETURN".to_string(), -9_999_999)
        ]
    );
    for code in ["X2-NOSYNC", "X2-SYNC", "X2-EDGE", "X2-RETURN"] {
        let product = product_repo::find_by_product_code(&conn, code)
            .unwrap()
            .unwrap();
        assert_eq!(product.product.stock_quantity, 10, "{code}");
    }
}

#[test]
fn test_commit_req401_stock_limit_rolls_back_whole_tx() {
    // REQ-401 / X3: 販売の取込みで計算の後の在庫が -999999999 を下回ったら TX 全体を戻す
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "X3-FULL", "4912345678901", -999_999_999, true);
    create_test_product_with_jan(&conn, "X3-OTHER", "4912345678902", 10, true);
    let bytes = make_z004_bytes(
        1,
        "2026-03-21",
        &[
            ("4912345678902", "ほかの商品", 2, 200),
            ("4912345678901", "下限の商品", 1, 100),
        ],
    );
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260321");
    let before = write_surface(&conn);
    let result = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: build_cached(pv),
            today: TEST_TODAY.to_string(),
        },
    );
    assert!(
        matches!(result, Err(BizError::ValidationFailed(ref m)) if m == "在庫数が大きすぎます"),
        "{result:?}"
    );
    assert_eq!(write_surface(&conn), before);

    // 写しを記録から求め直すので、新しい値との差 +1 の movement を入れて写しと記録を揃える（Matrix T4）
    crate::db::test_support::insert_initial_stock_movement(&conn, "X3-FULL", 1, -999_999_998);
    conn.execute(
        "UPDATE products SET stock_quantity = -999999998 WHERE product_code = 'X3-FULL'",
        [],
    )
    .unwrap();
    let bytes = make_z004_bytes(1, "2026-03-22", &[("4912345678901", "下限の商品", 1, 100)]);
    let pv = parse_and_build_cache(&conn, bytes, "Z004_260322");
    commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: build_cached(pv),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    let product = product_repo::find_by_product_code(&conn, "X3-FULL")
        .unwrap()
        .unwrap();
    assert_eq!(product.product.stock_quantity, -999_999_999);
}

// --- 確定と取消の新しい経路（ADR SPEC-STK-CNT-D1・D2・D4・D7、32「計数の置き場と在庫の求め直しの契約」） ---

use crate::biz::stocktake_service::{
    save_stock_count, CountRecoveryAction, RecountReason, SaveStockCountRequest,
};
use crate::db::product_repo::ProductStockUnit;

const REGISTRATION_CHANGED: &str =
    "商品の登録が読み込みの後に変わりました。ファイルを選び直してください";

fn save_count(
    conn: &mut crate::db::DbConnection,
    code: &str,
    value: i64,
    date: &str,
    fresh: bool,
    today: &str,
) {
    save_stock_count(
        conn,
        &SaveStockCountRequest {
            product_code: code.to_string(),
            count_value: value,
            fresh,
            business_date: date.to_string(),
            stocktake_item_id: None,
        },
        today,
    )
    .unwrap();
}

fn commit_cached(
    conn: &mut crate::db::DbConnection,
    cached: CachedPreview,
    today: &str,
) -> Result<ImportResult, BizError> {
    commit_csv_import(
        conn,
        CommitRequest {
            cached_data: cached,
            today: today.to_string(),
        },
    )
}

fn preview_cached(conn: &crate::db::DbConnection, bytes: Vec<u8>) -> CachedPreview {
    build_cached(parse_and_build_cache(conn, bytes, "Z004.CSV"))
}

#[test]
fn test_commit_req401_same_date_other_settlement_needs_no_confirmation() {
    // REQ-401 / ADR D4（Matrix Z7）: 同じ精算日の別の精算は確認なしに別の取込みとして確定する
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let one = [("4912345678901", "商品A", 1, 100)];
    commit_bytes(&mut conn, make_z004_bytes(1, "2026-03-21", &one), "a").unwrap();
    let pv = parse_and_build_cache(
        &conn,
        make_z004_bytes(2, "2026-03-21", &[("4912345678901", "商品A", 2, 200)]),
        "b",
    );
    assert!(pv.preview_data.settlement_gaps.is_empty());
    commit_cached(&mut conn, build_cached(pv), TEST_TODAY).unwrap();
    let imports = sales_repo::find_imports_by_settlement_date(&conn, "2026-03-21").unwrap();
    assert_eq!(imports.len(), 2);
    assert_eq!(stock_copy(&conn, "TEST-001"), (7, false, None));
}

#[test]
fn test_commit_req401_writes_settlement_key_and_recomputes() {
    // REQ-401 / REQ-205 / ADR D1・D2・D3（Matrix Z8）: 精算の鍵・stock_linked・phase 1 の movement、
    // 求め直し、結果の recheck_targets（単位・写しの在庫・範囲つき）。同じ商品の複数の行の stock_after は同じ写しの値
    let today = "2026-04-10";
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    create_test_product_with_jan(&conn, "TEST-002", "4912345678902", 10, false);
    save_count(&mut conn, "TEST-001", 10, today, false, today);
    let pv = parse_and_build_cache(
        &conn,
        make_z004_bytes(
            1,
            today,
            &[
                ("4912345678901", "商品A", 1, 100),
                ("4912345678901", "商品A", 2, 200),
                ("4912345678902", "商品B", 1, 100),
            ],
        ),
        "Z004.CSV",
    );
    let result = commit_cached(&mut conn, build_cached(pv), today).unwrap();

    let key: (String, i64) = conn
        .query_row(
            "SELECT machine_no, settlement_no FROM csv_imports WHERE id = ?1",
            [result.csv_import_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(key, ("0001".to_string(), 1));
    let linked: Vec<(String, bool)> = conn
        .prepare("SELECT product_code, stock_linked FROM sale_records ORDER BY id")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        linked,
        vec![
            ("TEST-001".to_string(), true),
            ("TEST-001".to_string(), true),
            ("TEST-002".to_string(), false),
        ]
    );
    let movements: Vec<(i64, String, i64, Option<i64>, i64)> = conn
        .prepare(
            "SELECT quantity, business_date, phase, settlement_no, stock_after FROM inventory_movements
             WHERE reference_type = 'csv_import' ORDER BY id",
        )
        .unwrap()
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    // 計数の日付 = 精算日なので決まらない（範囲 7〜10、写しは 7）
    assert_eq!(
        movements,
        vec![
            (-1, today.to_string(), 1, Some(1), 7),
            (-2, today.to_string(), 1, Some(1), 7),
        ]
    );
    assert_eq!(stock_copy(&conn, "TEST-001"), (7, true, Some((7, 10))));
    let derived = crate::biz::stock_ledger::derive_stock(&conn, "TEST-001").unwrap();
    assert_eq!((derived.stock, derived.range), (7, Some((7, 10))));
    assert_eq!(result.recheck_targets.len(), 1);
    let target = &result.recheck_targets[0];
    assert_eq!(target.product_code, "TEST-001");
    assert_eq!(target.stock_unit, ProductStockUnit::Pcs);
    assert_eq!(
        (
            target.stock,
            target.stock_range_low,
            target.stock_range_high
        ),
        (7, Some(7), Some(10))
    );
    assert_eq!(target.action, CountRecoveryAction::CountNow);
    assert_eq!(
        target.recount_reasons,
        vec![RecountReason::UndeterminedSettlement]
    );
    assert!(result.settlement_gaps.is_empty());
    assert!(!result.settlement_date_after_today);
    // 連動しない商品は売上だけ
    assert_eq!(stock_copy(&conn, "TEST-002"), (10, false, None));
}

#[test]
fn test_commit_req401_zero_rows_file_is_imported_as_settlement() {
    // REQ-401 / ADR D4（Matrix Z9。legacy F4 の入力）: 正常な行が全て 0 の file は売上も movement も作らずに
    // 取込みとして確定し、精算の並びと欠けに入る
    let (_dir, mut conn) = setup_test_db();
    let before = write_surface(&conn);
    let result = commit_bytes(
        &mut conn,
        make_z004_bytes(1, "2026-03-21", &[("4912345678901", "空", 0, 0)]),
        "zero",
    )
    .unwrap();
    assert_eq!(result.total_items, 0);
    let after = write_surface(&conn);
    assert_eq!(after, [before[0] + 1, before[1], before[2], before[3]]);
    let next = parse_and_build_cache(
        &conn,
        make_z004_bytes(3, "2026-03-23", &[("4912345678901", "空", 0, 0)]),
        "next",
    );
    assert_eq!(
        next.preview_data
            .settlement_gaps
            .iter()
            .map(|g| (g.from, g.to))
            .collect::<Vec<_>>(),
        vec![(2, 2)]
    );
}

#[test]
fn test_commit_req401_shared_jan_row_records_sale_only() {
    // REQ-401 / ADR D4（Matrix Z10。legacy F5 の入力）: 共有 JAN の行は先頭の商品の売上だけ、
    // どの候補の movement も作らず stock_linked = 0、行の error にしない
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "FS-0001", "4912345678901", 10, true);
    create_test_product_with_jan(&conn, "FS-0002", "4912345678901", 10, true);
    let result = commit_bytes(
        &mut conn,
        make_z004_bytes(1, "2026-03-21", &[("4912345678901", "グループ", 2, 600)]),
        "shared",
    )
    .unwrap();
    assert_eq!(result.skipped_count, 0);
    let sale: (String, bool) = conn
        .query_row(
            "SELECT product_code, stock_linked FROM sale_records",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(sale, ("FS-0001".to_string(), false));
    let movements: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inventory_movements WHERE reference_type = 'csv_import'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(movements, 0);
    assert_eq!(stock_copy(&conn, "FS-0001"), (10, false, None));
    assert_eq!(stock_copy(&conn, "FS-0002"), (10, false, None));
}

#[test]
fn test_commit_req401_rejects_when_original_jan_now_matches_other_product() {
    // REQ-401 / 32 の preview と確定の引き当ての食い違い（Matrix Z13）: preview の JAN J → A の後に、
    // A の JAN を K、B の JAN を J にした（元の JAN の引当先が B に変わる）→ 何も書かずに拒否
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "A-001", "4912345678901", 10, true);
    create_test_product_with_jan(&conn, "B-001", "4912345678902", 10, true);
    let cached = preview_cached(
        &conn,
        make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品", 1, 100)]),
    );
    conn.execute_batch(
        "UPDATE products SET jan_code = '4912345678903' WHERE product_code = 'A-001';
         UPDATE products SET jan_code = '4912345678901' WHERE product_code = 'B-001';",
    )
    .unwrap();
    let before = write_surface(&conn);
    let result = commit_cached(&mut conn, cached, TEST_TODAY);
    assert!(matches!(result, Err(BizError::ImportError(ref m)) if m == REGISTRATION_CHANGED));
    assert_eq!(write_surface(&conn), before);

    // 引き当てた商品が無くなった（JAN の候補が無い）行も拒否
    let cached = preview_cached(
        &conn,
        make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品", 1, 100)]),
    );
    conn.execute(
        "UPDATE products SET jan_code = '4912345678904' WHERE product_code = 'B-001'",
        [],
    )
    .unwrap();
    let result = commit_cached(&mut conn, cached, TEST_TODAY);
    assert!(matches!(result, Err(BizError::ImportError(ref m)) if m == REGISTRATION_CHANGED));
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_commit_req401_rechecks_rules_in_tx() {
    // REQ-401 / 32 の確定の TX（Matrix Z15）: preview の後に別の取込みが確定したら、確定の TX で順 1〜5 を取り直す
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let one = [("4912345678901", "商品A", 1, 100)];
    // 順 5: preview の後に、小さい番号で後の精算日の取込みが確定した
    let five = preview_cached(&conn, make_z004_bytes(5, "2026-03-05", &one));
    commit_bytes(&mut conn, make_z004_bytes(4, "2026-03-06", &one), "four").unwrap();
    let before = write_surface(&conn);
    let result = commit_cached(&mut conn, five, TEST_TODAY);
    assert!(matches!(result, Err(BizError::ImportError(ref m))
        if m == "レジの日付か番号が前の精算と合いません。レジの時計か、レジの入替・初期化を確かめてください"));
    assert_eq!(write_surface(&conn), before);
    // 順 2: preview の後に別のレジの取込みが入った
    let six = preview_cached(&conn, make_z004_bytes(6, "2026-03-07", &one));
    sales_repo::insert_csv_import(
        &conn,
        &sales_repo::NewCsvImport {
            filename: "other".to_string(),
            settlement_date: "2026-03-07".to_string(),
            file_hash: "other-register".to_string(),
            total_items: 0,
            total_amount: 0,
            skipped_count: 0,
            status: "completed".to_string(),
            machine_no: "0002".to_string(),
            settlement_no: 1,
        },
    )
    .unwrap();
    let before = write_surface(&conn);
    let result = commit_cached(&mut conn, six, TEST_TODAY);
    assert!(matches!(result, Err(BizError::ImportError(ref m))
        if m == "レジの番号が前の精算と合いません。レジの入替・初期化を確かめてください"));
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_commit_req401_rejects_shared_or_newly_matched_jan_and_rechecks_settlement_from_cache() {
    // REQ-401 / 32「preview の cache の契約」の (4)（Matrix Z21）
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "A-001", "4912345678901", 10, true);
    let rows = [
        ("4912345678901", "商品A", 1, 100),
        ("4912345678909", "未登録", 1, 100),
    ];
    // (a) 単独だった JAN が確定の前に共有 JAN になった
    let cached = preview_cached(&conn, make_z004_bytes(1, "2026-03-21", &rows[..1]));
    create_test_product_with_jan(&conn, "A-002", "4912345678901", 0, false);
    let before = write_surface(&conn);
    assert!(matches!(commit_cached(&mut conn, cached, TEST_TODAY),
        Err(BizError::ImportError(ref m)) if m == REGISTRATION_CHANGED));
    assert_eq!(write_surface(&conn), before);
    conn.execute("DELETE FROM products WHERE product_code = 'A-002'", [])
        .unwrap();
    // (b) preview で未登録だった行の JAN に確定の前に商品ができた
    let cached = preview_cached(&conn, make_z004_bytes(1, "2026-03-21", &rows));
    assert_eq!(cached.error_rows.len(), 1);
    create_test_product_with_jan(&conn, "N-001", "4912345678909", 0, false);
    let before = write_surface(&conn);
    assert!(matches!(commit_cached(&mut conn, cached, TEST_TODAY),
        Err(BizError::ImportError(ref m)) if m == REGISTRATION_CHANGED));
    assert_eq!(write_surface(&conn), before);
    // (c) 順 1〜5 は cache の settlement で取り直す（preview の後に同じ精算の別の版が確定していれば順 4）。
    // 名指す取込みの ID を 1 以外にする（Matrix RB7。取消済みの取込みは比べる先にならない）
    super::parse_tests::insert_rolled_back_import(&conn, 6);
    let cached = preview_cached(&conn, make_z004_bytes(7, "2026-03-22", &rows[..1]));
    let v2 = commit_bytes(
        &mut conn,
        make_z004_bytes(7, "2026-03-22", &[("4912345678901", "商品A", 2, 200)]),
        "v2",
    )
    .unwrap();
    assert_ne!(v2.csv_import_id, 1);
    match commit_cached(&mut conn, cached, TEST_TODAY) {
        Err(BizError::SettlementConflict {
            message,
            csv_import_id,
        }) => {
            assert_eq!(
                message,
                format!(
                    "同じ精算と思われる別の資料が既に取り込まれています（精算 7、取込みID: {}）",
                    v2.csv_import_id
                )
            );
            assert_eq!(csv_import_id, v2.csv_import_id);
        }
        other => panic!("SettlementConflict を期待しました: {other:?}"),
    }
    // 対照: preview の後に変わったのが在庫連動の設定だけなら拒否せず、stock_linked は確定の時点の値
    let cached = preview_cached(&conn, make_z004_bytes(8, "2026-03-23", &rows[..1]));
    conn.execute(
        "UPDATE products SET pos_stock_sync = 0 WHERE product_code = 'A-001'",
        [],
    )
    .unwrap();
    let stock_before = stock_copy(&conn, "A-001").0;
    let result = commit_cached(&mut conn, cached, TEST_TODAY).unwrap();
    let linked: bool = conn
        .query_row(
            "SELECT stock_linked FROM sale_records WHERE csv_import_id = ?1",
            [result.csv_import_id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!linked);
    assert_eq!(stock_copy(&conn, "A-001").0, stock_before);
}

#[test]
fn test_commit_req401_settlement_no_reset_after_rollback_falls_back_to_dates() {
    // REQ-401 / REQ-205 / ADR D3・D7（Matrix Z23。round 3 の是正）: 取消で判定の n の取込みが無くなった後に
    // 番号の戻った精算 1 を確定すると、判定の n は NULL（取込みのカーソル以下の active な取込みが無い。
    // Codex の closure の Gated Amendment）で、日付で判定して要再確認にする
    let today = "2026-05-10";
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let one = [("4912345678901", "商品A", 1, 100)];
    let hundred = commit_bytes(&mut conn, make_z004_bytes(100, "2026-05-09", &one), "a").unwrap();
    save_count(&mut conn, "TEST-001", 5, today, true, today);
    // 対照: 取消の前（精算 100 が active）なら同じ file は順 5 で拒否
    let reset = make_z004_bytes(1, "2026-05-11", &one);
    assert!(matches!(
        parse::parse_and_validate(
            &conn,
            CsvParseAndValidateRequest {
                file_bytes: reset.clone(),
                filename: "reset".to_string(),
                today: today.to_string(),
            },
        ),
        Err(BizError::ImportError(_))
    ));
    crate::biz::csv_import_service::rollback_csv_import(&mut conn, hundred.csv_import_id).unwrap();
    assert_eq!(stock_copy(&conn, "TEST-001"), (5, false, None));
    let cached = preview_cached(&conn, reset);
    let result = commit_cached(&mut conn, cached, today).unwrap();
    assert_eq!(stock_copy(&conn, "TEST-001"), (4, true, Some((4, 5))));
    assert_eq!(result.recheck_targets.len(), 1);
    assert_eq!(result.recheck_targets[0].product_code, "TEST-001");
    assert_eq!(
        result.recheck_targets[0].recount_reasons,
        vec![RecountReason::UndeterminedSettlement]
    );
}

#[test]
fn test_commit_req401_reused_settlement_no_after_rollback_falls_back_to_dates() {
    // REQ-401 / REQ-205 / ADR D3・D6・D7（Matrix Z24。Codex の closure の P1 の反例、Gated Amendment）:
    // 精算 1 の後に入れ直し、精算 1 を取り消して同じ番号の新しい精算 1 を確定すると、番号の active な取込みが
    // あっても判定の n は取込みのカーソルから NULL になり、日付で判定して要再確認にする。完了も拒む
    use crate::biz::stocktake_service::{
        complete_stocktake, start_stocktake, CompleteStocktakeRequest, CompletionBlockReason,
    };
    let today = "2026-05-10";
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let one = [("4912345678901", "商品A", 1, 100)];
    let first = commit_bytes(&mut conn, make_z004_bytes(1, "2026-05-09", &one), "a").unwrap();
    let stocktake_id = start_stocktake(&mut conn).unwrap().stocktake_id;
    let item: i64 = conn
        .query_row(
            "SELECT id FROM stocktake_items WHERE stocktake_id = ?1 AND product_code = 'TEST-001'",
            [stocktake_id],
            |row| row.get(0),
        )
        .unwrap();
    save_stock_count(
        &mut conn,
        &SaveStockCountRequest {
            product_code: "TEST-001".to_string(),
            count_value: 5,
            fresh: true,
            business_date: today.to_string(),
            stocktake_item_id: Some(item),
        },
        today,
    )
    .unwrap();
    crate::biz::csv_import_service::rollback_csv_import(&mut conn, first.csv_import_id).unwrap();
    assert_eq!(stock_copy(&conn, "TEST-001"), (5, false, None));
    // 同じレジの同じ番号の精算 1（精算日 T+1）は順 1〜5 を通る（順 4 は取り消した取込みを比べず、
    // 順 5 は比べる active な取込みが無い）
    let cached = preview_cached(&conn, make_z004_bytes(1, "2026-05-11", &one));
    let result = commit_cached(&mut conn, cached, today).unwrap();
    assert_eq!(stock_copy(&conn, "TEST-001"), (4, true, Some((4, 5))));
    assert_eq!(result.recheck_targets.len(), 1);
    let target = &result.recheck_targets[0];
    assert_eq!(target.product_code, "TEST-001");
    assert_eq!(
        (
            target.stock,
            target.stock_range_low,
            target.stock_range_high
        ),
        (4, Some(4), Some(5))
    );
    assert_eq!(
        target.recount_reasons,
        vec![RecountReason::UndeterminedSettlement]
    );
    // 完了は要再確認と (iii)（計数の日より前の精算が無い）で拒む
    match complete_stocktake(
        &mut conn,
        &CompleteStocktakeRequest {
            stocktake_id,
            today: today.to_string(),
        },
    ) {
        Err(BizError::StocktakeGuard { recovery, .. }) => assert_eq!(
            recovery
                .targets
                .iter()
                .map(|t| t.block_reason)
                .collect::<Vec<_>>(),
            vec![
                Some(CompletionBlockReason::Recheck),
                Some(CompletionBlockReason::SettlementPending)
            ]
        ),
        other => panic!("StocktakeGuard を期待: {other:?}"),
    }
}
