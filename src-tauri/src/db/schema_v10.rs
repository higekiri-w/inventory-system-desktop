//! migration v10（vB）: 原価を 1/100 円の列へ、単位から巻・cm を外す（作り直しの起点）
//!
//! docs/function-design/22-mnt-migration.md §16 / MNT-03-D14 / D-119・D-128 に基づく実装。
//! 原価の列を持つ 5 つの表に行の無い DB にだけ当たり、値は変えない。行のある DB は何も変えずに止まる。

use super::{migration_tx, DbError};
use rusqlite::{params, Connection};

/// 手順 2 の起点の検査で数える表（原価の列を持つ表と `products`）
const ORIGIN_TABLES: [&str; 5] = [
    "products",
    "receiving_items",
    "disposal_items",
    "stocktake_items",
    "price_history",
];

/// 手順 3 の改名（表・古い列名・新しい列名）
const COST_RENAMES: [(&str, &str, &str); 6] = [
    ("products", "cost_price", "cost_price_centi"),
    ("receiving_items", "cost_price", "cost_price_centi"),
    ("disposal_items", "cost_price", "cost_price_centi"),
    (
        "stocktake_items",
        "valuation_cost_price",
        "valuation_cost_price_centi",
    ),
    ("price_history", "old_cost", "old_cost_centi"),
    ("price_history", "new_cost", "new_cost_centi"),
];

/// 手順 4: v9 の後の `products`（v8 の列と §19 の写しの 3 列）を写し、`stock_unit` の CHECK だけを 10 個にする
const REBUILD_PRODUCTS_SQL: &str = "
CREATE TABLE products_new (
    product_code TEXT PRIMARY KEY,
    jan_code TEXT,
    name TEXT NOT NULL,
    department_id INTEGER NOT NULL REFERENCES departments(id),
    supplier_id INTEGER REFERENCES suppliers(id),
    selling_price INTEGER NOT NULL,
    cost_price_centi INTEGER NOT NULL,
    tax_rate TEXT NOT NULL DEFAULT '10' CHECK(tax_rate IN ('10','8','0')),
    maker_code TEXT,
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    stock_unit TEXT NOT NULL DEFAULT 'pcs' CHECK(stock_unit IN ('pcs','sheet','hon','bag','box','kumi','set','ball','cho','m')),
    is_discontinued BOOLEAN NOT NULL DEFAULT 0,
    plu_dirty BOOLEAN NOT NULL DEFAULT 1,
    plu_exported_at TEXT,
    pos_stock_sync BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    plu_target BOOLEAN NOT NULL DEFAULT 0,
    stock_recheck INTEGER NOT NULL DEFAULT 0 CHECK(stock_recheck IN (0,1)),
    stock_range_low INTEGER,
    stock_range_high INTEGER CHECK(
    (stock_recheck = 0 AND stock_range_low IS NULL AND stock_range_high IS NULL)
    OR (stock_recheck = 1 AND stock_range_low IS NOT NULL AND stock_range_high IS NOT NULL
        AND stock_range_low <= stock_range_high))
);
INSERT INTO products_new SELECT
    product_code, jan_code, name, department_id, supplier_id, selling_price, cost_price_centi,
    tax_rate, maker_code, stock_quantity, stock_unit, is_discontinued, plu_dirty,
    plu_exported_at, pos_stock_sync, created_at, updated_at, plu_target,
    stock_recheck, stock_range_low, stock_range_high
FROM products;
DROP TABLE products;
ALTER TABLE products_new RENAME TO products;
CREATE INDEX idx_products_jan_code ON products(jan_code);
CREATE INDEX idx_products_department_id ON products(department_id);
CREATE INDEX idx_products_is_discontinued ON products(is_discontinued);
";

/// v10 マイグレーション（MNT-03-D14 の vB）。`foreign_keys` の退避と復元は v8 と同じ形（TX の外）
pub(crate) fn apply_v10_cost_centi_origin(conn: &Connection, version: i64) -> Result<(), DbError> {
    // 1. TX の外で foreign_keys を読み、OFF にする
    let original_fk: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|e| DbError::MigrationFailed(format!("v{version} foreign_keys読取失敗: {e}")))?;
    conn.execute_batch("PRAGMA foreign_keys = OFF;")
        .map_err(|e| DbError::MigrationFailed(format!("v{version} foreign_keys=OFF失敗: {e}")))?;

    let result = apply_v10_inner(conn, version);

    // transaction を閉じられない場合は復元を試みず、接続破棄必須として返す（MNT-03-D1）
    if !conn.is_autocommit() {
        let original_error = result
            .as_ref()
            .err()
            .map(ToString::to_string)
            .unwrap_or_else(|| "migration result was unexpectedly successful".to_string());
        tracing::error!(
            version,
            error = %original_error,
            "migration transaction could not be closed; foreign_keys restore skipped and connection must be discarded"
        );
        return Err(DbError::MigrationFailed(format!(
            "v{version} migration失敗: {original_error}（transaction 状態不明のためforeign_keysを復元せず、接続破棄必須）"
        )));
    }

    // 6（後半）. TX が閉じた後に foreign_keys を元の値に戻す
    let restore_result = migration_tx::restore_foreign_keys(conn, version, original_fk);
    match (result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(original_error), Err(restore_error)) => {
            tracing::error!(
                version,
                error = %restore_error,
                original_error = %original_error,
                "migration failed and foreign_keys restore also failed"
            );
            Err(DbError::MigrationFailed(format!(
                "{original_error}; foreign_keys復元失敗も発生: {restore_error}"
            )))
        }
    }
}

/// 手順 2〜6（foreign_keys が OFF の間の 1 つの TX）。MigrationFailed は文をそのまま、ほかは版を添えて返す
fn apply_v10_inner(conn: &Connection, version: i64) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute_batch("BEGIN;")
        .map_err(|e| DbError::MigrationFailed(format!("v{version} BEGIN失敗: {e}")))?;

    let result = (|| -> Result<(), DbError> {
        // 2. 起点の検査（読取りだけ。何かを変える前）
        check_origin_is_empty(conn)?;

        // 3. 原価の 6 列の改名（行が無いので値は変えない）
        for (table, from, to) in COST_RENAMES {
            conn.execute_batch(&format!(
                "ALTER TABLE {table} RENAME COLUMN {from} TO {to};"
            ))?;
        }

        // 4. products の作り直し（stock_unit の CHECK を 10 個に）
        conn.execute_batch(REBUILD_PRODUCTS_SQL)?;

        // 5. foreign_key_check と 6 列の名前
        verify(conn, version)?;

        // 6. 版を記録
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (?1, ?2)",
            params![version, now],
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        let message = match error {
            DbError::MigrationFailed(message) => message,
            other => format!("v{version} 原価と単位の起点の migration失敗: {other}"),
        };
        return Err(migration_tx::rollback_after_error(conn, message));
    }
    migration_tx::commit_transaction(conn, &format!("v{version} COMMIT失敗"))
}

/// 手順 2: 5 つの表を外部キーに頼らず別々に数え、行のある表を全部挙げて止める
fn check_origin_is_empty(conn: &Connection) -> Result<(), DbError> {
    let mut hits = Vec::new();
    for table in ORIGIN_TABLES {
        let exists: bool = conn.query_row(
            &format!("SELECT EXISTS(SELECT 1 FROM {table})"),
            [],
            |row| row.get(0),
        )?;
        if exists {
            hits.push(table);
        }
    }
    if hits.is_empty() {
        return Ok(());
    }
    Err(DbError::MigrationFailed(format!(
        "商品か原価の記録がある DB はこの版へ移せません。原価を 1/100 円で持つ版は空の DB から始めます。開発・試験 DB は作り直してください（22 §16 の作り直しの手順）: {}",
        hits.join("・")
    )))
}

/// 手順 5: foreign_key_check が 0 行、6 列の新しい名前があり古い名前が無い
fn verify(conn: &Connection, version: i64) -> Result<(), DbError> {
    let fk_errors: i64 =
        conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    if fk_errors != 0 {
        return Err(DbError::MigrationFailed(format!(
            "v{version} FK整合性エラー: {fk_errors}件の違反を検出"
        )));
    }
    for (table, from, to) in COST_RENAMES {
        let has = |column: &str| -> Result<bool, DbError> {
            Ok(conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2)",
                params![table, column],
                |row| row.get(0),
            )?)
        };
        if !has(to)? || has(from)? {
            return Err(DbError::MigrationFailed(format!(
                "v{version} 列名の検証失敗: {table}.{from} を {to} へ改名できていません"
            )));
        }
    }
    Ok(())
}

/// vB の版（Matrix の B の前書き）。並走の migration が先に入って番号を取り直すときは、B の test ではここだけを直す
#[cfg(test)]
pub(crate) const VB_VERSION: i64 = 10;

/// vB の前の版（列名 `cost_price`）の DB に商品を 1 行入れる（vB の起点の検査と作り直しの起動の test の fixture）
#[cfg(test)]
pub(crate) fn insert_pre_vb_product(conn: &Connection, product_code: &str) {
    conn.execute(
        "INSERT INTO products (product_code, name, department_id, selling_price, cost_price,
             stock_unit, created_at, updated_at)
         VALUES (?1, '合成の旧い商品', 1, 100, 50, 'pcs', '2026-10-21T00:00:00', '2026-10-21T00:00:00')",
        params![product_code],
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::{self, migrate_up_to};
    use crate::db::migration_tx::{self, FailurePoint};
    use crate::db::product_repo::ProductStockUnit;
    use crate::db::test_support::{dump, products_definition};
    use rusqlite::types::Value;

    const STOP_PREFIX: &str = "商品か原価の記録がある DB はこの版へ移せません。原価を 1/100 円で持つ版は空の DB から始めます。開発・試験 DB は作り直してください（22 §16 の作り直しの手順）: ";

    fn version(conn: &Connection) -> (i64, i64) {
        conn.query_row(
            "SELECT MAX(version), COUNT(*) FROM schema_versions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    fn foreign_keys(conn: &Connection) -> i64 {
        conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap()
    }

    /// vB の直前の版の空の DB
    fn setup_before_vb() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("before-vb.db")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate_up_to(&conn, VB_VERSION - 1).unwrap();
        assert_eq!(version(&conn), (VB_VERSION - 1, VB_VERSION - 1));
        (dir, conn)
    }

    fn setup_latest() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn =
            crate::db::init_database(dir.path().join("latest.db").to_str().unwrap()).unwrap();
        (dir, conn)
    }

    /// schema（全 object の定義）と全表の全行（止まったときに何も変わらないことの比較に使う）
    fn snapshot(conn: &Connection) -> Vec<Vec<Vec<Value>>> {
        let mut out = vec![dump(
            conn,
            "SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name",
        )];
        for row in dump(
            conn,
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
        ) {
            let Value::Text(table) = &row[0] else {
                panic!("{row:?}")
            };
            out.push(dump(conn, &format!("SELECT * FROM \"{table}\" ORDER BY 1")));
        }
        out
    }

    /// 全 object（table・index・trigger・view）の type・name・tbl_name と、空白を除いた SQL
    /// （`products` は作り直しで改行と `,` の位置が変わるので、空白を除いて比べる）
    fn schema_objects(conn: &Connection) -> Vec<Vec<Value>> {
        let mut rows = dump(
            conn,
            "SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name",
        );
        for row in &mut rows {
            if let Value::Text(sql) = &row[3] {
                row[3] = Value::Text(sql.split_whitespace().collect());
            }
        }
        rows
    }

    fn column_names(conn: &Connection, table: &str) -> Vec<String> {
        dump(
            conn,
            &format!("SELECT name FROM pragma_table_info('{table}')"),
        )
        .into_iter()
        .map(|row| match &row[0] {
            Value::Text(name) => name.clone(),
            other => panic!("{other:?}"),
        })
        .collect()
    }

    fn migration_failed(conn: &Connection) -> String {
        match migration::migrate(conn).unwrap_err() {
            DbError::MigrationFailed(message) => message,
            other => panic!("MigrationFailed を期待: {other:?}"),
        }
    }

    /// vB の前の版の列名で、5 つの表のどれか 1 つに 1 行を入れる（子の表の行は foreign_keys = OFF で）
    fn seed_origin_row(conn: &Connection, table: &str) {
        let sql = match table {
            "products" => return insert_pre_vb_product(conn, "VB-1"),
            "receiving_items" => {
                "INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price)
                 VALUES (1, 'VB-NONE', 1, 50)"
            }
            "disposal_items" => {
                "INSERT INTO disposal_items (disposal_record_id, product_code, disposal_type, quantity, cost_price, reason)
                 VALUES (1, 'VB-NONE', 'other', 1, 50, '合成')"
            }
            "stocktake_items" => {
                "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, valuation_cost_price)
                 VALUES (1, 'VB-NONE', 0, 50)"
            }
            "price_history" => {
                "INSERT INTO price_history (product_code, old_selling, new_selling, old_cost, new_cost, changed_at)
                 VALUES ('VB-NONE', 100, 110, 50, 55, '2026-10-21T00:00:00')"
            }
            other => panic!("{other}"),
        };
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        conn.execute(sql, []).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    }

    const NEW_COLUMNS: [(&str, &str); 6] = [
        ("products", "cost_price_centi"),
        ("receiving_items", "cost_price_centi"),
        ("disposal_items", "cost_price_centi"),
        ("stocktake_items", "valuation_cost_price_centi"),
        ("price_history", "old_cost_centi"),
        ("price_history", "new_cost_centi"),
    ];
    const OLD_COLUMNS: [(&str, &str); 6] = [
        ("products", "cost_price"),
        ("receiving_items", "cost_price"),
        ("disposal_items", "cost_price"),
        ("stocktake_items", "valuation_cost_price"),
        ("price_history", "old_cost"),
        ("price_history", "new_cost"),
    ];

    /// 6 列の名前が `expected` の側にだけある
    fn assert_cost_columns(
        conn: &Connection,
        present: [(&str, &str); 6],
        absent: [(&str, &str); 6],
    ) {
        for (table, column) in present {
            assert!(
                column_names(conn, table).iter().any(|name| name == column),
                "{table}.{column} が無い"
            );
        }
        for (table, column) in absent {
            assert!(
                !column_names(conn, table).iter().any(|name| name == column),
                "{table}.{column} が残る"
            );
        }
    }

    #[test]
    fn test_migration_req903_vb_fresh_database_renames_cost_columns() {
        // REQ-903 / MNT-03-D14 手順 3 / Matrix B1: 新規の DB は vB まで当たり、6 列が _centi の名前になる
        let (_dir, conn) = setup_latest();
        assert_eq!(
            version(&conn),
            (
                crate::db::migration::app_max_version(),
                crate::db::migration::app_max_version()
            )
        );
        assert_cost_columns(&conn, NEW_COLUMNS, OLD_COLUMNS);
    }

    #[test]
    fn test_migration_req903_vb_rebuild_keeps_columns_indexes_and_fk() {
        // REQ-903 / MNT-03-D14 手順 4 / Matrix B2: 作り直しの後の products の定義は cost_price の名前だけが違い、
        // DB の全 object の SQL は 6 列の改名と stock_unit の CHECK だけが違う（DROP TABLE が trigger 等を黙って消す退行を拾う）
        let (_dir, conn) = setup_before_vb();
        let mut expected = products_definition(&conn);
        let mut renamed = 0;
        for column in &mut expected[0] {
            if column[1] == Value::Text("cost_price".into()) {
                column[1] = Value::Text("cost_price_centi".into());
                renamed += 1;
            }
        }
        assert_eq!(renamed, 1);
        assert_eq!(expected[0].len(), 21, "v8 の 18 列と §19 の写しの 3 列");
        assert_eq!(expected[1].len(), 2, "departments・suppliers への外部キー");
        assert_eq!(expected[2].len(), 4, "自動の index と 3 つの index");
        // 全 object の期待: 6 列の改名と products の stock_unit の CHECK だけが違う（trigger・view・他の表と index は同じ）
        let mut expected_objects = schema_objects(&conn);
        assert!(expected_objects
            .iter()
            .any(|row| row[1] == Value::Text("stock_counts".into())
                && matches!(&row[3], Value::Text(sql) if sql.contains("REFERENCESproducts("))));
        let mut edit = |table: &str, from: &str, to: &str| {
            let row = expected_objects
                .iter_mut()
                .find(|row| {
                    row[0] == Value::Text("table".into()) && row[1] == Value::Text(table.into())
                })
                .unwrap();
            let Value::Text(sql) = &row[3] else {
                panic!("{row:?}")
            };
            assert_eq!(sql.matches(from).count(), 1, "{table}: {from}");
            row[3] = Value::Text(sql.replace(from, to));
        };
        for ((table, old), (_, new)) in OLD_COLUMNS.into_iter().zip(NEW_COLUMNS) {
            edit(table, &format!("{old}INTEGER"), &format!("{new}INTEGER"));
        }
        edit(
            "products",
            "CHECK(stock_unitIN('pcs','sheet','hon','bag','box','roll','kumi','set','ball','cho','m','cm'))",
            "CHECK(stock_unitIN('pcs','sheet','hon','bag','box','kumi','set','ball','cho','m'))",
        );

        migrate_up_to(&conn, VB_VERSION).unwrap(); // vB だけを当てる（Matrix T1。22 §19 の先例）

        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
        assert_eq!(products_definition(&conn), expected);
        assert_eq!(schema_objects(&conn), expected_objects);
        assert!(dump(&conn, "SELECT * FROM pragma_foreign_key_check").is_empty());
        assert!(dump(
            &conn,
            "SELECT name FROM sqlite_master WHERE name = 'products_new'"
        )
        .is_empty());
        assert_eq!(foreign_keys(&conn), 1, "foreign_keys は元の値");
    }

    #[test]
    fn test_migration_req903_vb_check_codes_match_enum() {
        // REQ-903 / MNT-03-D14 手順 4 / SPEC-UNIT-D1・D-128 / Matrix B3: vB の CHECK と enum の一覧が同じ
        let codes: Vec<String> = ProductStockUnit::ALL
            .iter()
            .map(|unit| format!("'{}'", unit.as_str()))
            .collect();
        let check = format!("CHECK(stock_unit IN ({}))", codes.join(","));
        assert!(REBUILD_PRODUCTS_SQL.contains(&check), "{check}");

        let (_dir, conn) = setup_latest();
        let insert = |code: &str, unit: &str| {
            conn.execute(
                "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi,
                     stock_unit, created_at, updated_at)
                 VALUES (?1, '合成の商品', 1, 100, 8333, ?2, '2026-10-21T00:00:00', '2026-10-21T00:00:00')",
                params![code, unit],
            )
        };
        for unit in ProductStockUnit::ALL {
            insert(&format!("VB-{}", unit.as_str()), unit.as_str()).unwrap();
        }
        for unit in ["roll", "cm", "kg"] {
            let error = insert(&format!("VB-X-{unit}"), unit).unwrap_err();
            assert!(
                error.to_string().contains("CHECK constraint failed"),
                "{unit}: {error}"
            );
        }
    }

    #[test]
    fn test_migration_req903_vb_stops_on_rows_in_each_table_without_changes() {
        // REQ-903 / MNT-03-D14 手順 2 / Matrix B4: 5 つの表のどれか 1 つに 1 行あれば、何も変えずに止まる
        // case は 22 §16 手順 2 の表名の literal で回す（production の一覧から表が外れる退行を拾う）
        const SPEC_ORIGIN_TABLES: [&str; 5] = [
            "products",
            "receiving_items",
            "disposal_items",
            "stocktake_items",
            "price_history",
        ];
        assert_eq!(ORIGIN_TABLES, SPEC_ORIGIN_TABLES);
        for table in SPEC_ORIGIN_TABLES {
            let (_dir, conn) = setup_before_vb();
            seed_origin_row(&conn, table);
            let before = snapshot(&conn);

            let message = migration_failed(&conn);

            assert_eq!(message, format!("{STOP_PREFIX}{table}"), "{table}");
            assert_eq!(snapshot(&conn), before, "{table}");
            assert_cost_columns(&conn, OLD_COLUMNS, NEW_COLUMNS);
            assert_eq!(version(&conn), (VB_VERSION - 1, VB_VERSION - 1), "{table}");
            assert_eq!(foreign_keys(&conn), 1, "{table}");
            assert!(conn.is_autocommit(), "{table}");
        }
    }

    #[test]
    fn test_migration_req903_vb_stop_message_lists_all_tables_with_rows() {
        // REQ-903 / MNT-03-D14 手順 2 / Matrix B5: 行のある表を全部、決まった並びで挙げる
        let (_dir, conn) = setup_before_vb();
        seed_origin_row(&conn, "price_history");
        seed_origin_row(&conn, "products");
        assert_eq!(
            migration_failed(&conn),
            format!("{STOP_PREFIX}products・price_history")
        );
        assert_eq!(version(&conn), (VB_VERSION - 1, VB_VERSION - 1));
    }

    #[test]
    fn test_migration_req903_vb_verify_or_commit_failure_rolls_back() {
        // REQ-903 / MNT-03-D14 手順 5・6 / MNT-03-D1 / Matrix B6: 検証か COMMIT の失敗で全部戻り、foreign_keys を戻す
        // (1) 手順 5 の検証の失敗: 5 つの表に入らない子の行（inventory_movements）が products を指さない
        for original_fk in [1_i64, 0] {
            let (_dir, conn) = setup_before_vb();
            conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
            conn.execute(
                "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                     created_at, business_date, phase)
                 VALUES ('VB-NO-PARENT', 'receiving', 1, 1, '2026-10-21T00:00:00', '2026-10-21', 0)",
                [],
            )
            .unwrap();
            conn.execute_batch(&format!("PRAGMA foreign_keys = {original_fk};"))
                .unwrap();
            let before = snapshot(&conn);

            let message = migration_failed(&conn);

            assert!(message.contains("FK整合性エラー"), "{message}");
            assert_eq!(snapshot(&conn), before);
            assert_cost_columns(&conn, OLD_COLUMNS, NEW_COLUMNS);
            assert_eq!(version(&conn), (VB_VERSION - 1, VB_VERSION - 1));
            assert_eq!(foreign_keys(&conn), original_fk);
            assert!(conn.is_autocommit());
        }
        // (2) COMMIT の失敗（ROLLBACK は成功）
        let (_dir, conn) = setup_before_vb();
        let before = snapshot(&conn);
        let failures = migration_tx::fail_operations(&[FailurePoint::Commit]);
        let message = migration_failed(&conn);
        drop(failures);
        assert!(
            message.contains(&format!("v{VB_VERSION} COMMIT失敗")),
            "{message}"
        );
        assert!(message.contains("ROLLBACK 成功"), "{message}");
        assert_eq!(snapshot(&conn), before);
        assert_cost_columns(&conn, OLD_COLUMNS, NEW_COLUMNS);
        assert_eq!(version(&conn), (VB_VERSION - 1, VB_VERSION - 1));
        assert_eq!(foreign_keys(&conn), 1);
        // 失敗を外せば同じ DB で当たる
        migrate_up_to(&conn, VB_VERSION).unwrap(); // vB だけを当てる（Matrix T1）
        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
    }

    #[test]
    fn test_migration_req903_vb_rerun_is_noop() {
        // REQ-903 / MNT-03-D14 / Matrix B7: 2 回目の migrate は何もしない
        let (_dir, conn) = setup_latest();
        let before = snapshot(&conn);
        migration::migrate(&conn).unwrap();
        assert_eq!(snapshot(&conn), before);
        let vb_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_versions WHERE version = ?1",
                [VB_VERSION],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(vb_rows, 1);
    }

    /// 最新の版の products に 1 行を入れる（列は cost_price_centi）
    fn insert_latest_product(
        conn: &Connection,
        code: &str,
        tax_rate: &str,
        recheck: i64,
        low: Option<i64>,
        high: Option<i64>,
    ) -> rusqlite::Result<usize> {
        conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi,
                 tax_rate, stock_unit, created_at, updated_at, stock_recheck, stock_range_low, stock_range_high)
             VALUES (?1, '合成の商品', 1, 100, 8333, ?2, 'pcs', '2026-10-21T00:00:00', '2026-10-21T00:00:00', ?3, ?4, ?5)",
            params![code, tax_rate, recheck, low, high],
        )
    }

    fn assert_check_failed(result: rusqlite::Result<usize>, case: &str) {
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("CHECK constraint failed"),
            "{case}: {error}"
        );
    }

    #[test]
    fn test_migration_req903_vb_keeps_stock_range_check() {
        // REQ-903 / MNT-03-D14 手順 4 / §19 / Matrix B8: 作り直しの後も写しの 3 列の CHECK が効く
        let (_dir, conn) = setup_latest();
        assert_check_failed(
            insert_latest_product(&conn, "VB-R1", "10", 1, None, None),
            "recheck 1 で範囲 NULL",
        );
        assert_check_failed(
            insert_latest_product(&conn, "VB-R2", "10", 1, Some(6), Some(5)),
            "low > high",
        );
        // 対照: 正しい範囲は通る
        insert_latest_product(&conn, "VB-R3", "10", 1, Some(5), Some(5)).unwrap();
        insert_latest_product(&conn, "VB-R4", "10", 0, None, None).unwrap();
    }

    #[test]
    fn test_migration_req903_vb_keeps_tax_rate_check() {
        // REQ-903 / MNT-03-D14 手順 4 / Matrix B10: 作り直しの後も tax_rate の CHECK が効く
        let (_dir, conn) = setup_latest();
        assert_check_failed(
            insert_latest_product(&conn, "VB-T1", "5", 0, None, None),
            "tax_rate 5",
        );
        // 対照: 10 は通る
        insert_latest_product(&conn, "VB-T2", "10", 0, None, None).unwrap();
    }
}
