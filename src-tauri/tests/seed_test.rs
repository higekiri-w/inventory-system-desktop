//! seed_demo_data bin の integration test
//!
//! 関数名は `test_` prefix を使わない。理由: `scripts/pre-push.sh` の REQ 番号チェック
//! (step ④) が `test_*` 関数を対象にするため、dev tooling 用テストは対象外にする。
//! `#[test]` 属性のみで cargo test に認識される。

use inventory_system_tauri_scaffold_lib::db::init_database;
use inventory_system_tauri_scaffold_lib::seed_demo::{
    run_seed, PLU_BUCKET_DEMO_COUNT, SALES_RECORDS,
};

const BASE_PRODUCT_COUNT: i64 = 100;

fn expected_product_count() -> i64 {
    BASE_PRODUCT_COUNT + i64::from(PLU_BUCKET_DEMO_COUNT)
}

/// 一時 DB ファイルを作り、マイグレーション済みの Connection を返す
fn setup_temp_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempfile::tempdir().expect("tempdir 作成失敗");
    let db_path = dir.path().join("seed_test.db");
    let conn = init_database(db_path.to_str().expect("db_path utf-8"))
        .expect("init_database 失敗 (テスト前提)");
    (dir, conn)
}

#[test]
fn seed_populates_products_with_plu_bucket_demo_rows() {
    let (_dir, mut conn) = setup_temp_db();
    let summary = run_seed(&mut conn, false).expect("run_seed 失敗");

    assert_eq!(
        i64::from(summary.products_inserted),
        expected_product_count()
    );
    assert_eq!(summary.products_skipped, 0);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
        .expect("SELECT products COUNT 失敗");
    assert_eq!(count, expected_product_count(), "products 件数");
}

#[test]
fn seed_populates_300_sale_records() {
    let (_dir, mut conn) = setup_temp_db();
    let summary = run_seed(&mut conn, false).expect("run_seed 失敗");

    assert_eq!(summary.sale_records_inserted, SALES_RECORDS);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sale_records", [], |row| row.get(0))
        .expect("SELECT sale_records COUNT 失敗");
    assert_eq!(
        count, SALES_RECORDS as i64,
        "sale_records は 300 件であるべき"
    );
}

#[test]
fn seed_populates_400_inventory_movements() {
    let (_dir, mut conn) = setup_temp_db();
    let _ = run_seed(&mut conn, false).expect("run_seed 失敗");

    // receiving 100 + sale_auto 300 = 400
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
            row.get(0)
        })
        .expect("SELECT inventory_movements COUNT 失敗");
    assert_eq!(count, 400, "inventory_movements は 400 件であるべき");

    let receiving: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inventory_movements WHERE movement_type = 'receiving'",
            [],
            |row| row.get(0),
        )
        .expect("SELECT receiving COUNT 失敗");
    assert_eq!(receiving, 100, "receiving は 100 件");

    let sale_auto: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM inventory_movements WHERE movement_type = 'sale_auto'",
            [],
            |row| row.get(0),
        )
        .expect("SELECT sale_auto COUNT 失敗");
    assert_eq!(sale_auto, 300, "sale_auto は 300 件");
}

#[test]
fn seed_is_idempotent() {
    let (_dir, mut conn) = setup_temp_db();
    let _ = run_seed(&mut conn, false).expect("1 回目 run_seed 失敗");

    let products_1: i64 = conn
        .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
        .expect("1 回目 products count");
    let sales_1: i64 = conn
        .query_row("SELECT COUNT(*) FROM sale_records", [], |row| row.get(0))
        .expect("1 回目 sales count");
    let movements_1: i64 = conn
        .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
            row.get(0)
        })
        .expect("1 回目 movements count");
    let suppliers_1: i64 = conn
        .query_row("SELECT COUNT(*) FROM suppliers", [], |row| row.get(0))
        .expect("1 回目 suppliers count");

    // 2 回目実行
    let summary_2 = run_seed(&mut conn, false).expect("2 回目 run_seed 失敗");

    // 2 回目は全件 skip される
    assert_eq!(
        summary_2.products_inserted, 0,
        "2 回目 products は 0 insert"
    );
    assert_eq!(
        i64::from(summary_2.products_skipped),
        expected_product_count(),
        "2 回目 products は全件 skip"
    );
    assert_eq!(
        summary_2.sale_records_inserted, 0,
        "2 回目 sales は 0 insert"
    );
    assert_eq!(
        summary_2.sale_records_skipped, SALES_RECORDS,
        "2 回目 sales は 300 skip"
    );

    let products_2: i64 = conn
        .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
        .expect("2 回目 products count");
    let sales_2: i64 = conn
        .query_row("SELECT COUNT(*) FROM sale_records", [], |row| row.get(0))
        .expect("2 回目 sales count");
    let movements_2: i64 = conn
        .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
            row.get(0)
        })
        .expect("2 回目 movements count");
    let suppliers_2: i64 = conn
        .query_row("SELECT COUNT(*) FROM suppliers", [], |row| row.get(0))
        .expect("2 回目 suppliers count");

    assert_eq!(products_1, products_2, "products 行数が変化してはいけない");
    assert_eq!(sales_1, sales_2, "sales 行数が変化してはいけない");
    assert_eq!(
        movements_1, movements_2,
        "movements 行数が変化してはいけない"
    );
    assert_eq!(
        suppliers_1, suppliers_2,
        "suppliers 行数が変化してはいけない"
    );
}

#[test]
fn seed_uses_deterministic_rng() {
    // 2 つの別 DB に seed を走らせ、product_code / jan_code / selling_price の列挙が
    // 完全一致することを確認する。これが一致すれば rand seed と日付基準は決定的。
    let (_dir1, mut conn1) = setup_temp_db();
    let (_dir2, mut conn2) = setup_temp_db();

    let _ = run_seed(&mut conn1, false).expect("DB1 run_seed");
    let _ = run_seed(&mut conn2, false).expect("DB2 run_seed");

    let rows1 = collect_products(&conn1);
    let rows2 = collect_products(&conn2);

    assert_eq!(rows1.len() as i64, expected_product_count());
    assert_eq!(rows2.len() as i64, expected_product_count());
    assert_eq!(
        rows1, rows2,
        "決定的 seed: 2 DB 間で product_code / jan_code / selling_price が一致するべき"
    );
}

#[test]
fn seed_products_have_valid_ean13_jan_for_plu_export() {
    // REQ-402 / CV17 1.1.1: UI-08 L3 用のデモ商品はスキャニングPLUに使える13桁JANを持つ。
    let (_dir, mut conn) = setup_temp_db();
    let _ = run_seed(&mut conn, false).expect("run_seed 失敗");

    let mut stmt = conn
        .prepare("SELECT product_code, jan_code FROM products ORDER BY product_code")
        .expect("prepare jan validation");
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .expect("query_map jan validation");

    let mut checked = 0;
    for row in rows {
        let (product_code, jan_code) = row.expect("row jan validation");
        if product_code == "D028-NO-JAN" || product_code == "D028-BAD-CHECK" {
            continue;
        }
        let jan_code = jan_code.unwrap_or_else(|| panic!("{product_code} jan_code is missing"));
        assert!(
            is_valid_ean13(&jan_code),
            "{product_code} jan_code should be valid EAN-13: {jan_code}"
        );
        checked += 1;
    }
    assert_eq!(checked, expected_product_count() - 2);
}

fn collect_products(conn: &rusqlite::Connection) -> Vec<(String, Option<String>, i64)> {
    let mut stmt = conn
        .prepare("SELECT product_code, jan_code, selling_price FROM products ORDER BY product_code")
        .expect("prepare collect_products");
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .expect("query_map collect_products");
    let mut out = Vec::new();
    for r in rows {
        out.push(r.expect("row collect_products"));
    }
    out
}

fn is_valid_ean13(value: &str) -> bool {
    if value.len() != 13 || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    let mut digits = value.chars().map(|ch| ch.to_digit(10).unwrap_or(0));
    let check_digit = value
        .chars()
        .last()
        .and_then(|ch| ch.to_digit(10))
        .unwrap_or(99);
    let sum: u32 = digits
        .by_ref()
        .take(12)
        .enumerate()
        .map(|(idx, digit)| if idx % 2 == 0 { digit } else { digit * 3 })
        .sum();
    (10 - (sum % 10)) % 10 == check_digit
}

#[test]
fn seed_produces_stockout_products() {
    // PR-3 (e): 色分け契約 H 検証用に在庫切れ (stock_quantity <= 0) 商品が seed される。
    let (_dir, mut conn) = setup_temp_db();
    let _ = run_seed(&mut conn, false).expect("run_seed 失敗");

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE stock_quantity <= 0",
            [],
            |row| row.get(0),
        )
        .expect("SELECT stockout COUNT 失敗");
    assert!(
        count >= 1,
        "在庫切れ (stock_quantity<=0) 商品が 1 件以上 seed されるべき"
    );
}

#[test]
fn seed_produces_low_stock_products() {
    // PR-3 (e): 在庫少 (stock_quantity > 0 かつ閾値以下) 商品が seed される。
    // stockout (=0) を在庫少と誤検出しないよう `> 0` 条件を必須にする (Codex P2-2)。
    // 色分け契約 H は在庫切れ / 在庫少を別表示する前提のため、low の陽性サンプルには
    // `>0` 条件が不可欠。閾値: pcs <= 3 (D-4 stock_low_threshold) / cm <= 500 (同 _fabric)。
    let (_dir, mut conn) = setup_temp_db();
    let _ = run_seed(&mut conn, false).expect("run_seed 失敗");

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products \
             WHERE stock_quantity > 0 \
               AND ((stock_unit = 'pcs' AND stock_quantity <= 3) \
                 OR (stock_unit = 'm' AND stock_quantity <= 500))",
            [],
            |row| row.get(0),
        )
        .expect("SELECT low-stock COUNT 失敗");
    assert!(
        count >= 1,
        "在庫少 (>0 かつ閾値以下) 商品が 1 件以上 seed されるべき"
    );
}

#[test]
fn seed_stockout_low_distributed_across_departments() {
    // PR-3 (e): stockout/low が単一部門集中でなく複数部門に分散する。
    // 各部門の index i==1 を在庫切れ、i==2 を在庫少に固定するため、
    // 6 部門それぞれに stockout/low が 1 件ずつ出る。
    let (_dir, mut conn) = setup_temp_db();
    let _ = run_seed(&mut conn, false).expect("run_seed 失敗");

    let stockout_dept_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT department_id) FROM products WHERE stock_quantity <= 0",
            [],
            |row| row.get(0),
        )
        .expect("SELECT stockout dept COUNT 失敗");
    assert!(
        stockout_dept_count >= 2,
        "在庫切れ商品が 2 部門以上に分散すべき (1 部門集中でない)"
    );

    let low_dept_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT department_id) FROM products \
             WHERE stock_quantity > 0 \
               AND ((stock_unit = 'pcs' AND stock_quantity <= 3) \
                 OR (stock_unit = 'm' AND stock_quantity <= 500))",
            [],
            |row| row.get(0),
        )
        .expect("SELECT low-stock dept COUNT 失敗");
    assert!(
        low_dept_count >= 2,
        "在庫少商品が 2 部門以上に分散すべき (1 部門集中でない)"
    );
}

#[test]
fn seed_reset_req205_clears_stock_counts() {
    // REQ-205 / b1 Matrix E1: --reset（delete_all）が stock_counts を先に消し、棚卸しの明細を参照する
    // 計数のある DB でも reset が通る（stock_counts は products と stocktake_items を外部キーで参照する）
    let (_dir, mut conn) = setup_temp_db();
    run_seed(&mut conn, false).expect("run_seed 失敗");
    let product_code: String = conn
        .query_row(
            "SELECT product_code FROM products ORDER BY product_code LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "INSERT INTO stocktakes (started_at, status) VALUES ('2026-10-01T09:00:00', 'in_progress')",
        [],
    )
    .unwrap();
    let stocktake_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock) VALUES (?1, ?2, 0)",
        rusqlite::params![stocktake_id, product_code],
    )
    .unwrap();
    let item_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
             stocktake_item_id, counted_at)
         VALUES (?1, 3, 'default', '2026-10-01', ?2, '2026-10-01T10:00:00')",
        rusqlite::params![product_code, item_id],
    )
    .unwrap();

    run_seed(&mut conn, true).expect("--reset の run_seed 失敗");
    let counts: i64 = conn
        .query_row("SELECT COUNT(*) FROM stock_counts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(counts, 0);
}
