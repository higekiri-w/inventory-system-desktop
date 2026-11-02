use crate::biz::csv_import_service::test_support::*;
use crate::biz::csv_import_service::*;
use crate::biz::BizError;
use crate::db::test_support::setup_test_db;
use crate::db::{product_repo, sales_repo};

/// commit してからrollbackをテストするヘルパー
fn commit_import(
    conn: &mut crate::db::DbConnection,
    bytes: Vec<u8>,
    filename: &str,
) -> ImportResult {
    let pv = parse_and_build_cache(conn, bytes, filename);
    let cached = build_cached(pv);
    commit_csv_import(
        conn,
        CommitRequest {
            cached_data: cached,
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap()
}

// --- rollback_csv_import テスト ---

#[test]
fn test_rollback_req401_normal() {
    // REQ-401: CSV取込み
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 3, 900)]);
    let import_result = commit_import(&mut conn, bytes, "Z004_260321");

    // 在庫が減っていることを確認
    let p = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, 7); // 10 - 3

    // ロールバック
    let rb = rollback_csv_import(&mut conn, import_result.csv_import_id).unwrap();
    assert!(rb.success);
    assert_eq!(rb.voided_sale_count, 1);
    assert_eq!(rb.voided_movement_count, 1);

    // DB検証: 在庫が元に戻っている
    let p = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, 10);
}

#[test]
fn test_rollback_req401_idempotent() {
    // REQ-401: CSV取込み
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 3, 900)]);
    let import_result = commit_import(&mut conn, bytes, "Z004_260321");

    // 1回目のロールバック
    let rb1 = rollback_csv_import(&mut conn, import_result.csv_import_id).unwrap();
    assert!(rb1.success);
    assert_eq!(rb1.voided_sale_count, 1);

    // 2回目のロールバック（冪等）
    let rb2 = rollback_csv_import(&mut conn, import_result.csv_import_id).unwrap();
    assert!(rb2.success);
    assert_eq!(rb2.voided_sale_count, 0);
    assert_eq!(rb2.voided_movement_count, 0);

    // DB検証: 在庫は1回目のロールバック後のまま
    let p = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(p.product.stock_quantity, 10);
}

#[test]
fn test_rollback_req401_not_found() {
    // REQ-401: CSV取込み
    let (_dir, mut conn) = setup_test_db();
    let result = rollback_csv_import(&mut conn, 999);
    assert!(result.is_err());
    match result.unwrap_err() {
        BizError::NotFound(msg) => assert!(msg.contains("999")),
        e => panic!("Expected NotFound, got {:?}", e),
    }
}

#[test]
fn test_rollback_req401_same_day_is_per_import_with_negative_return() {
    // REQ-401 / I-B6 / SPEC-SDI-D2,D7: 対象importだけを取消し、同日返品siblingを保持する。
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let first = commit_import(
        &mut conn,
        make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 2, 600)]),
        "Z004_0001",
    );
    let second_preview = parse_and_build_cache(
        &conn,
        make_z004_bytes(2, "2026-03-21", &[("4912345678901", "返品A", -1, -300)]),
        "Z004_0002",
    );
    let second = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: build_cached(second_preview),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();

    rollback_csv_import(&mut conn, first.csv_import_id).unwrap();

    let active =
        crate::db::sales_repo::find_imports_by_settlement_date(&conn, "2026-03-21").unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, second.csv_import_id);
    let product = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(product.product.stock_quantity, 11);
    let remaining_sales = sales_repo::get_daily_sales_records(&conn, "2026-03-21").unwrap();
    assert_eq!(remaining_sales.len(), 1);
    assert_eq!(remaining_sales[0].quantity, -1);
    assert_eq!(remaining_sales[0].amount, -300);
    assert_eq!(remaining_sales[0].source, "auto");
}

#[test]
fn test_rollback_req401_selected_negative_return_reverses_only_return_contribution() {
    // REQ-401 / I-B7 / SPEC-SDI-D2: 負数返品importを取消すと返品寄与だけを逆転しfirst saleを保持する。
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let first = commit_import(
        &mut conn,
        make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 2, 600)]),
        "Z004_0001",
    );
    let return_preview = parse_and_build_cache(
        &conn,
        make_z004_bytes(2, "2026-03-21", &[("4912345678901", "返品A", -1, -300)]),
        "Z004_0002",
    );
    let returned = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: build_cached(return_preview),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();
    let before = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(before.product.stock_quantity, 9);

    rollback_csv_import(&mut conn, returned.csv_import_id).unwrap();

    let after = product_repo::find_by_product_code(&conn, "TEST-001")
        .unwrap()
        .unwrap();
    assert_eq!(after.product.stock_quantity, 8);
    let active = sales_repo::find_imports_by_settlement_date(&conn, "2026-03-21").unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, first.csv_import_id);
    let first_active_sale: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sale_records WHERE csv_import_id = ?1 AND is_voided = 0",
            [first.csv_import_id],
            |row| row.get(0),
        )
        .unwrap();
    let return_active_sale: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sale_records WHERE csv_import_id = ?1 AND is_voided = 0",
            [returned.csv_import_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(first_active_sale, 1);
    assert_eq!(return_active_sale, 0);
}

#[test]
fn test_import_rollback_req401_logs_exact_selected_id_and_preserves_commit_on_log_failure() {
    // REQ-401 / I-B9 / SPEC-SDI-D2: log失敗はbusiness rollbackを戻さず、成功logは選択IDを記録する。
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let first = commit_import(
        &mut conn,
        make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 1, 300)]),
        "Z004_0001",
    );
    let second_preview = parse_and_build_cache(
        &conn,
        make_z004_bytes(2, "2026-03-21", &[("4912345678901", "商品B", 2, 600)]),
        "Z004_0002",
    );
    let second = commit_csv_import(
        &mut conn,
        CommitRequest {
            cached_data: build_cached(second_preview),
            today: TEST_TODAY.to_string(),
        },
    )
    .unwrap();

    conn.execute_batch(
        "CREATE TRIGGER fail_csv_rollback_log
         BEFORE INSERT ON operation_logs
         WHEN NEW.operation_type = 'csv_rollback'
         BEGIN SELECT RAISE(FAIL, 'synthetic log failure'); END;",
    )
    .unwrap();
    let result = rollback_csv_import(&mut conn, first.csv_import_id).unwrap();
    assert!(result.success);
    let first_record = sales_repo::find_csv_import_by_id(&conn, first.csv_import_id)
        .unwrap()
        .unwrap();
    let second_record = sales_repo::find_csv_import_by_id(&conn, second.csv_import_id)
        .unwrap()
        .unwrap();
    assert_eq!(first_record.status, sales_repo::CsvImportStatus::RolledBack);
    assert_eq!(second_record.status, sales_repo::CsvImportStatus::Completed);

    conn.execute_batch("DROP TRIGGER fail_csv_rollback_log;")
        .unwrap();
    rollback_csv_import(&mut conn, second.csv_import_id).unwrap();
    let (summary, detail): (String, String) = conn
        .query_row(
            "SELECT summary, detail_json FROM operation_logs
             WHERE operation_type = 'csv_rollback' ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(summary.contains(&format!("ID {}", second.csv_import_id)));
    assert!(detail.contains(&format!(r#""csv_import_id":{}"#, second.csv_import_id)));
}

#[test]
fn test_rollback_req401_stock_limit_rolls_back_whole_tx() {
    // REQ-401 / 共通規則 (a)（X3）: 取込みの取消の補正の後の在庫が上限の外なら、取消を TX 全体で戻す
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    let bytes = make_z004_bytes(1, "2026-03-21", &[("4912345678901", "商品A", 1, 100)]);
    let import_result = commit_import(&mut conn, bytes, "Z004_260321");
    let set_stock = |conn: &crate::db::DbConnection, stock: i64| {
        // 写しを記録から求め直すので、新しい値との差の movement を入れて写しと記録を揃える（Matrix T4）
        let current: i64 = conn
            .query_row(
                "SELECT stock_quantity FROM products WHERE product_code = 'TEST-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        crate::db::test_support::insert_initial_stock_movement(
            conn,
            "TEST-001",
            stock - current,
            stock,
        );
        conn.execute(
            "UPDATE products SET stock_quantity = ?1 WHERE product_code = 'TEST-001'",
            [stock],
        )
        .unwrap();
    };
    let state = |conn: &crate::db::DbConnection| -> (i64, i64, i64) {
        conn.query_row(
            "SELECT (SELECT stock_quantity FROM products WHERE product_code = 'TEST-001'),
                    (SELECT COUNT(*) FROM sale_records WHERE is_voided = 1),
                    (SELECT COUNT(*) FROM inventory_movements WHERE is_voided = 1)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap()
    };

    set_stock(&conn, 999_999_999);
    let result = rollback_csv_import(&mut conn, import_result.csv_import_id);
    assert!(
        matches!(result, Err(BizError::ValidationFailed(ref m)) if m == "在庫数が大きすぎます"),
        "{result:?}"
    );
    assert_eq!(state(&conn), (999_999_999, 0, 0));
    let import = sales_repo::find_csv_import_by_id(&conn, import_result.csv_import_id)
        .unwrap()
        .unwrap();
    assert_ne!(import.status, sales_repo::CsvImportStatus::RolledBack);

    set_stock(&conn, 999_999_998);
    rollback_csv_import(&mut conn, import_result.csv_import_id).unwrap();
    assert_eq!(state(&conn), (999_999_999, 1, 1));
}

#[test]
fn test_rollback_req401_voids_and_recomputes_without_compensation() {
    // REQ-401 / REQ-205 / ADR D2・D7（Matrix Z14）: 取消は売上と movement を無効にし、求め直す商品を
    // 求め直す。補償の movement を作らず、計数を消さない。結果に recheck_targets
    let day = "2026-04-10";
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "TEST-001", "4912345678901", 10, true);
    crate::biz::stocktake_service::save_stock_count(
        &mut conn,
        &crate::biz::stocktake_service::SaveStockCountRequest {
            product_code: "TEST-001".to_string(),
            count_value: 10,
            fresh: false,
            business_date: day.to_string(),
            stocktake_item_id: None,
        },
        day,
    )
    .unwrap();
    let one = [("4912345678901", "商品A", 1, 100)];
    let first = commit_import(&mut conn, make_z004_bytes(1, day, &one), "a");
    let second = commit_import(&mut conn, make_z004_bytes(2, "2026-04-11", &one), "b");
    let counts = |conn: &crate::db::DbConnection| -> (i64, i64) {
        conn.query_row(
            "SELECT (SELECT COUNT(*) FROM inventory_movements), (SELECT COUNT(*) FROM stock_counts)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    };
    let before = counts(&conn);
    // 計数の日付の精算 1 を取り消すと、精算 2 が計数の日付の後の最初の 1 本で決まらない（要再確認のまま）
    let rb = rollback_csv_import(&mut conn, first.csv_import_id).unwrap();
    assert_eq!(counts(&conn), before);
    assert_eq!(stock_copy(&conn, "TEST-001"), (9, true, Some((9, 10))));
    assert_eq!(rb.recheck_targets.len(), 1);
    assert_eq!(rb.recheck_targets[0].product_code, "TEST-001");
    let voided: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inventory_movements WHERE reference_id = ?1 AND is_voided = 1",
            [first.csv_import_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(voided, 1);
    // 両方を取り消すと計数だけが残り、要再確認が解ける
    let rb = rollback_csv_import(&mut conn, second.csv_import_id).unwrap();
    assert_eq!(counts(&conn), before);
    assert_eq!(stock_copy(&conn, "TEST-001"), (10, false, None));
    assert!(rb.recheck_targets.is_empty());
    let derived = crate::biz::stock_ledger::derive_stock(&conn, "TEST-001").unwrap();
    assert_eq!((derived.stock, derived.range), (10, None));
}

// --- preview_rollback_csv_import テスト（32 §15.6b、BIZ-03-D2） ---

const DAY: &str = "2026-04-10";
const NEXT_DAY: &str = "2026-04-11";
const JAN_A: &str = "4912345678901";
const JAN_B: &str = "4912345678902";

/// 既定の計数（「数を保存」）
fn save_default_count(conn: &mut crate::db::DbConnection, code: &str, value: i64, date: &str) {
    crate::biz::stocktake_service::save_stock_count(
        conn,
        &crate::biz::stocktake_service::SaveStockCountRequest {
            product_code: code.to_string(),
            count_value: value,
            fresh: false,
            business_date: date.to_string(),
            stocktake_item_id: None,
        },
        NEXT_DAY,
    )
    .unwrap();
}

/// ADR D3 の A1 の形: 精算 101（DAY、B だけ）・102（NEXT_DAY、A）、A の既定の計数 10（DAY）。
/// DAY に精算 101 があるので 102 は数に入らず、A は在庫 9 で要再確認でない。101 の取込み ID を返す
fn a1_fixture(conn: &mut crate::db::DbConnection) -> i64 {
    create_test_product_with_jan(conn, "A-001", JAN_A, 10, true);
    create_test_product_with_jan(conn, "B-001", JAN_B, 10, true);
    save_default_count(conn, "A-001", 10, DAY);
    let first = commit_import(
        conn,
        make_z004_bytes(101, DAY, &[(JAN_B, "商品B", 1, 100)]),
        "z004-0101",
    );
    commit_import(
        conn,
        make_z004_bytes(102, NEXT_DAY, &[(JAN_A, "商品A", 1, 100)]),
        "z004-0102",
    );
    assert_eq!(stock_copy(conn, "A-001"), (9, false, None));
    first.csv_import_id
}

/// preview が書かないことを見る面（写し・無効の印・取込みの状態・操作ログ・連番）
fn write_surface(conn: &crate::db::DbConnection) -> String {
    conn.query_row(
        "SELECT (SELECT group_concat(product_code || ':' || stock_quantity || ':' || stock_recheck || ':'
                     || ifnull(stock_range_low, '-') || ':' || ifnull(stock_range_high, '-'), ',')
                 FROM products)
             || '|' || (SELECT group_concat(id || ':' || is_voided, ',') FROM sale_records)
             || '|' || (SELECT group_concat(id || ':' || is_voided, ',') FROM inventory_movements)
             || '|' || (SELECT group_concat(id || ':' || status, ',') FROM csv_imports)
             || '|' || (SELECT COUNT(*) FROM operation_logs)
             || '|' || (SELECT group_concat(name || ':' || seq, ',') FROM sqlite_sequence)",
        [],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn test_preview_rollback_csv_import_req401_recheck_after_losing_count_day_settlement() {
    // REQ-401 / ADR SPEC-STK-CNT-D3 の A1（Matrix RB5）: 計数の日の精算を取り消すと、計数を持つだけの商品が
    // 要再確認になることを取り消す前に名指す（A は 101 に行が無く、計数を持つ商品としてだけ求め直される）
    let (_dir, mut conn) = setup_test_db();
    let first = a1_fixture(&mut conn);

    let preview = preview_rollback_csv_import(&mut conn, first).unwrap();

    assert_eq!(preview.recheck_targets.len(), 1);
    let target = &preview.recheck_targets[0];
    assert_eq!(target.product_code, "A-001");
    assert_eq!(
        (target.stock_range_low, target.stock_range_high),
        (Some(9), Some(10))
    );
    assert_eq!(
        target.recount_reasons,
        vec![crate::biz::stocktake_service::RecountReason::UndeterminedSettlement]
    );
}

#[test]
fn test_preview_rollback_csv_import_req401_matches_rollback_recheck_targets() {
    // REQ-401 / BIZ-03-D2（Matrix RB1）: preview の要再確認は、続けて取り消した結果の要再確認と同じ。
    // 取り消す前から要再確認の商品（C: 同じ日に違う数）も両方に入る
    let (_dir, mut conn) = setup_test_db();
    let first = a1_fixture(&mut conn);
    create_test_product_with_jan(&conn, "C-001", "4912345678903", 5, true);
    save_default_count(&mut conn, "C-001", 5, DAY);
    save_default_count(&mut conn, "C-001", 6, DAY);
    assert!(stock_copy(&conn, "C-001").1, "C は取り消す前から要再確認");

    let preview = preview_rollback_csv_import(&mut conn, first).unwrap();
    let rolled_back = rollback_csv_import(&mut conn, first).unwrap();

    assert_eq!(preview.recheck_targets, rolled_back.recheck_targets);
    let codes: Vec<&str> = preview
        .recheck_targets
        .iter()
        .map(|t| t.product_code.as_str())
        .collect();
    assert_eq!(codes, vec!["A-001", "C-001"]);
}

#[test]
fn test_preview_rollback_csv_import_req401_writes_nothing() {
    // REQ-401 / BIZ-03-D2（Matrix RB2）: preview は取消を TX で当てて戻し、写し・無効の印・取込みの状態・
    // 操作ログ・連番のどれも変えない
    let (_dir, mut conn) = setup_test_db();
    let first = a1_fixture(&mut conn);
    let before = write_surface(&conn);

    let preview = preview_rollback_csv_import(&mut conn, first).unwrap();

    assert!(!preview.recheck_targets.is_empty());
    assert_eq!(write_surface(&conn), before);
    // 取消の本体は preview の後も通常どおり当たる
    rollback_csv_import(&mut conn, first).unwrap();
    assert_ne!(write_surface(&conn), before);
}

#[test]
fn test_preview_rollback_csv_import_req401_rejects_missing_and_rolled_back() {
    // REQ-401 / 32 §15.6b の手順 1（Matrix RB3）: 無い ID は NotFound、取消済みは ValidationFailed で、何も書かない
    let (_dir, mut conn) = setup_test_db();
    let first = a1_fixture(&mut conn);
    rollback_csv_import(&mut conn, first).unwrap();
    let before = write_surface(&conn);

    match preview_rollback_csv_import(&mut conn, 999) {
        Err(BizError::NotFound(msg)) => {
            assert_eq!(msg, "CSV取込み記録が見つかりません: ID 999")
        }
        other => panic!("NotFound を期待しました: {other:?}"),
    }
    match preview_rollback_csv_import(&mut conn, first) {
        Err(BizError::ValidationFailed(msg)) => assert_eq!(msg, "この取込みは取り消し済みです"),
        other => panic!("ValidationFailed を期待しました: {other:?}"),
    }
    assert_eq!(write_surface(&conn), before);
}

#[test]
fn test_preview_rollback_csv_import_req401_names_settlement_no() {
    // REQ-401 / 32 §15.6b の手順 2（Matrix RB4）: 取り込み直す精算の番号はその取込みの精算回数（0 も 0）
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "A-001", JAN_A, 10, true);
    let line = [(JAN_A, "商品A", 1, 100)];
    let zero = commit_import(&mut conn, make_z004_bytes(0, DAY, &line), "z004-0000");
    let later = commit_import(
        &mut conn,
        make_z004_bytes(101, NEXT_DAY, &line),
        "z004-0101",
    );

    let preview_zero = preview_rollback_csv_import(&mut conn, zero.csv_import_id).unwrap();
    let preview_later = preview_rollback_csv_import(&mut conn, later.csv_import_id).unwrap();

    assert_eq!(
        (preview_zero.csv_import_id, preview_zero.settlement_no),
        (zero.csv_import_id, 0)
    );
    assert_eq!(
        (preview_later.csv_import_id, preview_later.settlement_no),
        (later.csv_import_id, 101)
    );
}

#[test]
fn test_preview_rollback_csv_import_req401_affected_completed_stocktakes() {
    // REQ-401 / 32 §15.6b の手順 3・35 の完了の後の比較の範囲（Matrix RB6）: 範囲は
    // 精算回数 ≤ 完了の時点の精算の n、または精算日 < 完了日。範囲による候補で、記録の詳細の数が変わるかは見ない
    let (_dir, mut conn) = setup_test_db();
    create_test_product_with_jan(&conn, "B-001", JAN_B, 10, true);
    let line = [(JAN_B, "商品B", 1, 100)];
    let before_1 = commit_import(&mut conn, make_z004_bytes(101, "2026-04-01", &line), "a");
    let before_2 = commit_import(&mut conn, make_z004_bytes(102, "2026-04-02", &line), "b");
    // 完了した棚卸し 2 つ（id の順と完了の日時の順を逆にする）と進行中の棚卸し 1 つ。明細は無い
    let stocktake = |conn: &crate::db::DbConnection, status: &str, completed_at: Option<&str>| {
        conn.execute(
            "INSERT INTO stocktakes (started_at, status, completed_at, total_cost, cutoff_import_id)
             VALUES ('2026-04-01T09:00:00', ?1, ?2, 0, ?3)",
            rusqlite::params![status, completed_at, completed_at.map(|_| before_2.csv_import_id)],
        )
        .unwrap();
        conn.last_insert_rowid()
    };
    let late = stocktake(&conn, "completed", Some("2026-04-05T10:00:00"));
    let early = stocktake(&conn, "completed", Some("2026-04-03T09:00:00"));
    stocktake(&conn, "in_progress", None);
    // 完了の後に取り込んだ取込み: 精算日が late の完了日より前 / 完了日の後で精算回数が n（102）より大きい
    let backfilled = commit_import(&mut conn, make_z004_bytes(103, "2026-04-04", &line), "c");
    let after = commit_import(&mut conn, make_z004_bytes(104, "2026-04-06", &line), "d");
    let named = |conn: &mut crate::db::DbConnection, id: i64| -> Vec<(i64, String)> {
        preview_rollback_csv_import(conn, id)
            .unwrap()
            .affected_stocktakes
            .into_iter()
            .map(|s| (s.stocktake_id, s.completed_at))
            .collect()
    };
    let both = vec![
        (early, "2026-04-03T09:00:00".to_string()),
        (late, "2026-04-05T10:00:00".to_string()),
    ];

    // (a)(d)(e) 完了の前に取り込んだ取込みは両方の完了した棚卸しを完了の日時の昇順で名指し、進行中を名指さない
    assert_eq!(named(&mut conn, before_1.csv_import_id), both);
    assert_eq!(named(&mut conn, before_2.csv_import_id), both);
    // (b) 完了の後に取り込んだ、精算日 < 完了日の取込みは late だけ（early の完了日 04-03 より後の精算日）
    assert_eq!(
        named(&mut conn, backfilled.csv_import_id),
        vec![(late, "2026-04-05T10:00:00".to_string())]
    );
    // (c) 精算日 ≥ 完了日で精算回数 > n の取込みはどれも名指さない
    assert!(named(&mut conn, after.csv_import_id).is_empty());
    // (e) の前提: 明細の無い棚卸しは取り消しても記録の詳細の数が変わらない（それでも名指す）
    let record = |conn: &crate::db::DbConnection| {
        format!(
            "{:?}",
            crate::biz::stocktake_service::get_stocktake_record(conn, late).unwrap()
        )
    };
    let record_before = record(&conn);
    rollback_csv_import(&mut conn, before_1.csv_import_id).unwrap();
    assert_eq!(record(&conn), record_before);

    // (f)(g) 等値境界（OR のもう片側が偽の取込み）。精算 201・202 は精算日がどちらも完了日 04-20 と同じ日で、
    // 完了の日時は 0 時でない（日時の全体で比べると精算日 < 完了の日時になる）。完了の時点の精算の n は 201
    let (_dir2, mut conn2) = setup_test_db();
    create_test_product_with_jan(&conn2, "B-001", JAN_B, 10, true);
    let at_n = commit_import(&mut conn2, make_z004_bytes(201, "2026-04-20", &line), "e");
    let completed_at = "2026-04-20T18:00:00";
    conn2
        .execute(
            "INSERT INTO stocktakes (started_at, status, completed_at, total_cost, cutoff_import_id)
             VALUES ('2026-04-20T09:00:00', 'completed', ?1, 0, ?2)",
            rusqlite::params![completed_at, at_n.csv_import_id],
        )
        .unwrap();
    let boundary = conn2.last_insert_rowid();
    let same_day_after = commit_import(&mut conn2, make_z004_bytes(202, "2026-04-20", &line), "f");
    // (f) 精算回数 = n（精算日 = 完了日で日付の側は偽）は名指す
    assert_eq!(
        named(&mut conn2, at_n.csv_import_id),
        vec![(boundary, completed_at.to_string())]
    );
    // (g) 精算回数 > n で精算日 = 完了日（日付の部分で比べる）は名指さない
    assert!(named(&mut conn2, same_day_after.csv_import_id).is_empty());
}
