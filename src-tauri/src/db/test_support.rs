//! テスト共通ヘルパー

use super::DbConnection;
use crate::db::product_repo::{self, NewProduct};

pub fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = crate::db::init_database(db_path.to_str().unwrap()).unwrap();
    (dir, conn)
}

/// schema 版がアプリの最大 + 1 の DB を作って閉じる（MNT-03-D11 の fixture）。足した版を返す
pub fn write_newer_schema_db(path: &std::path::Path) -> i64 {
    let conn = crate::db::init_database(path.to_str().unwrap()).unwrap();
    let newer = crate::db::migration::app_max_version() + 1;
    conn.execute(
        "INSERT INTO schema_versions (version, applied_at) VALUES (?1, '2026-09-25T00:00:00')",
        [newer],
    )
    .unwrap();
    newer
}

/// テスト用に商品を1件挿入するヘルパー
pub fn seed_product(conn: &DbConnection, product_code: &str) {
    let product = NewProduct {
        product_code: product_code.to_string(),
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
    product_repo::insert_product(conn, &product).unwrap();
}

/// 在庫つきの商品の fixture が、写しと同じ値の初期在庫の movement を入れるヘルパー（Matrix T3・T4）
///
/// `movement_type` は `receiving`、`reference_type` は NULL、業務日は固定の `2026-04-01`。
/// 写しを記録から求め直す経路（D-118 (a)）を通す test が、写しと記録を揃えるのに使う
pub fn insert_initial_stock_movement(
    conn: &DbConnection,
    product_code: &str,
    quantity: i64,
    stock_after: i64,
) {
    conn.execute(
        "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
             note, created_at, business_date, phase)
         VALUES (?1, 'receiving', ?2, ?3, '初期在庫投入', '2026-04-01T09:00:00', '2026-04-01', 0)",
        rusqlite::params![product_code, quantity, stock_after],
    )
    .unwrap();
}

/// テスト用に取引先を作成するヘルパー
pub fn seed_supplier(conn: &DbConnection) -> i64 {
    conn.execute(
        "INSERT INTO suppliers (name, created_at) VALUES ('テスト取引先', '2026-04-06T00:00:00')",
        [],
    )
    .unwrap();
    conn.last_insert_rowid()
}

/// SQL の結果の全行（test の比較用）
pub(crate) fn dump(conn: &DbConnection, sql: &str) -> Vec<Vec<rusqlite::types::Value>> {
    let mut statement = conn.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| (0..columns).map(|i| row.get(i)).collect())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

/// `products` の定義（列の名前・順・型・notnull・既定値・pk、自身の FK、index）。
/// 作り直しの migration（v8 の vU・v10 の vB）の前後比較に使う
pub(crate) fn products_definition(conn: &DbConnection) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
    vec![
        dump(conn, "SELECT * FROM pragma_table_info('products')"),
        dump(conn, "SELECT * FROM pragma_foreign_key_list('products')"),
        dump(
            conn,
            "SELECT name, sql FROM sqlite_master
             WHERE type = 'index' AND tbl_name = 'products' ORDER BY name",
        ),
    ]
}
