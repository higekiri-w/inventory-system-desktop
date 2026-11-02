//! 商品・部門・取引先・価格履歴のCRUD操作
//!
//! 20-io-product-repo.md §2.3〜2.6 に基づく実装。
//! IO-01: SQLiteデータアクセス層（product_repository）

use super::{DbConnection, DbError, PaginatedResult};
use crate::constants::PAGINATION_MAX_PER_PAGE;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ProductTaxRate {
    #[serde(rename = "10")]
    Rate10,
    #[serde(rename = "8")]
    Rate8,
    #[serde(rename = "0")]
    Rate0,
}

impl ProductTaxRate {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rate10 => "10",
            Self::Rate8 => "8",
            Self::Rate0 => "0",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ProductStockUnit {
    Pcs,
    Sheet,
    Hon,
    Bag,
    Box,
    Kumi,
    Set,
    Ball,
    Cho,
    M,
}

impl ProductStockUnit {
    /// 全 variant（SPEC-UNIT-D1 の表の順）。在庫少の並びと DB の読取りがここから作る
    pub const ALL: [Self; 10] = [
        Self::Pcs,
        Self::Sheet,
        Self::Hon,
        Self::Bag,
        Self::Box,
        Self::Kumi,
        Self::Set,
        Self::Ball,
        Self::Cho,
        Self::M,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pcs => "pcs",
            Self::Sheet => "sheet",
            Self::Hon => "hon",
            Self::Bag => "bag",
            Self::Box => "box",
            Self::Kumi => "kumi",
            Self::Set => "set",
            Self::Ball => "ball",
            Self::Cho => "cho",
            Self::M => "m",
        }
    }

    /// code から variant を引く。一覧に無い値は `None`
    pub(crate) fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|unit| unit.as_str() == code)
    }
}

fn parse_tax_rate(value: String) -> rusqlite::Result<ProductTaxRate> {
    match value.as_str() {
        "10" => Ok(ProductTaxRate::Rate10),
        "8" => Ok(ProductTaxRate::Rate8),
        "0" => Ok(ProductTaxRate::Rate0),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown product tax_rate: {value}").into(),
        )),
    }
}

pub(crate) fn parse_stock_unit(value: String) -> rusqlite::Result<ProductStockUnit> {
    ProductStockUnit::from_code(&value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown product stock_unit: {value}").into(),
        )
    })
}

/// 商品マスタの行マッピング（products テーブル全18カラム）
///
/// db-design/master-tables.md products
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct Product {
    pub product_code: String,
    pub jan_code: Option<String>,
    pub name: String,
    pub department_id: i64,
    pub supplier_id: Option<i64>,
    pub selling_price: i64,
    pub cost_price_centi: i64,
    pub tax_rate: ProductTaxRate,
    pub maker_code: Option<String>,
    pub stock_quantity: i64,
    pub stock_unit: ProductStockUnit,
    pub is_discontinued: bool,
    pub plu_dirty: bool,
    pub plu_exported_at: Option<String>,
    pub plu_target: bool,
    pub pos_stock_sync: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 商品 + 部門名 + 取引先名（LEFT JOIN結果）
///
/// 20-io-product-repo.md §2.3 find_by_product_code
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ProductWithRelations {
    #[serde(flatten)]
    pub product: Product,
    pub department_name: String,
    pub supplier_name: Option<String>,
    pub plu_memory_no: Option<i64>,
    /// 写しの要再確認と範囲（ADR SPEC-STK-CNT-D2・D9。範囲は要再確認のときだけ両端が値。40）
    pub stock_recheck: bool,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
}

/// 商品 + 部門名（INNER JOIN結果、PLU書出し用）
///
/// 20-io-product-repo.md §2.3 find_active_products_for_plu / find_plu_dirty_products_for_plu
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductForPlu {
    #[serde(flatten)]
    pub product: Product,
    pub department_name: String,
}

/// 商品INSERT用（created_at/updated_atは関数内で自動設定）
///
/// 20-io-product-repo.md §2.3 insert_product
#[derive(Debug)]
pub struct NewProduct {
    pub product_code: String,
    pub jan_code: Option<String>,
    pub name: String,
    pub department_id: i64,
    pub supplier_id: Option<i64>,
    pub selling_price: i64,
    pub cost_price_centi: i64,
    pub tax_rate: String,
    pub maker_code: Option<String>,
    pub stock_quantity: i64,
    pub stock_unit: String,
    pub is_discontinued: bool,
    pub plu_dirty: bool,
    pub plu_exported_at: Option<String>,
    pub plu_target: bool,
    pub pos_stock_sync: bool,
}

/// 商品UPDATE用（Someのフィールドだけ更新）
///
/// 20-io-product-repo.md §2.3 update_product
///
/// ## nullable フィールドの扱い
/// DB上NULLableなカラム（supplier_id, maker_code）は `Option<Option<T>>`:
/// - `None` → 更新しない
/// - `Some(None)` → NULLにする
/// - `Some(Some(v))` → 値を更新
///
/// jan_code は商品修正時に変更不可（FUNCTION_DESIGN参照）。
/// plu_exported_at: Option<Option<String>> — None=変更なし、Some(None)=NULLに設定、Some(Some(v))=値を設定
#[derive(Debug, Default)]
pub struct ProductUpdates {
    pub jan_code: Option<Option<String>>,
    pub name: Option<String>,
    pub department_id: Option<i64>,
    pub supplier_id: Option<Option<i64>>,
    pub selling_price: Option<i64>,
    pub cost_price_centi: Option<i64>,
    pub tax_rate: Option<String>,
    pub maker_code: Option<Option<String>>,
    /// 値を検査しない（個数の単位どうしの判定は BIZ、BIZ-01-D8）
    pub stock_unit: Option<ProductStockUnit>,
    pub is_discontinued: Option<bool>,
    pub plu_dirty: Option<bool>,
    pub plu_exported_at: Option<Option<String>>,
    pub plu_target: Option<bool>,
    pub pos_stock_sync: Option<bool>,
}

/// 部門マスタの行マッピング
///
/// db-design/master-tables.md departments
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct Department {
    pub id: i64,
    pub name: String,
    pub z005_name: Option<String>,
    pub code_prefix: Option<String>,
    pub next_seq: i64,
    pub created_at: String,
}

/// 取引先マスタの行マッピング
///
/// db-design/master-tables.md suppliers
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct Supplier {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

/// UI-15 一覧用の IO 内部行。wire DTO への変換は BIZ-01 が所有する。
#[derive(Debug)]
pub(crate) struct SupplierUsageRow {
    pub supplier: Supplier,
    pub product_count: i64,
    pub receiving_record_count: i64,
}

/// 価格履歴INSERT用
///
/// 20-io-product-repo.md §2.6
#[derive(Debug)]
pub struct NewPriceHistory {
    pub product_code: String,
    pub old_selling: i64,
    pub new_selling: i64,
    pub old_cost_centi: i64,
    pub new_cost_centi: i64,
}

/// 価格履歴の読取り用行マッピング
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct PriceHistoryEntry {
    pub id: i64,
    pub old_selling_price: i64,
    pub new_selling_price: i64,
    pub old_cost_price_centi: i64,
    pub new_cost_price_centi: i64,
    pub changed_at: String,
}

/// 商品検索条件
///
/// 20-io-product-repo.md §2.3 search_products
#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct ProductSearchQuery {
    /// 商品名/product_code/jan_code の部分一致
    pub keyword: Option<String>,
    pub department_id: Option<i64>,
    #[serde(default)]
    pub supplier_id: Option<i64>,
    #[serde(default)]
    pub include_unassigned: bool,
    /// None=全件、Some(false)=現行品のみ、Some(true)=廃番のみ
    pub is_discontinued: Option<bool>,
    #[serde(default)]
    pub plu: Option<PluMigrationFilter>,
    pub sort_key: SortKey,
    pub sort_order: SortOrder,
    /// 1始まり。0以下は DbError::QueryFailed
    pub page: u32,
    /// 1以上。0は DbError::QueryFailed。デフォルト50
    pub per_page: u32,
}

/// PLU 対象一括更新用の商品検索条件（ページングなし）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ProductBulkFilter {
    pub keyword: Option<String>,
    pub department_id: Option<i64>,
    pub is_discontinued: Option<bool>,
    pub plu: Option<PluMigrationFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PluMigrationFilter {
    All,
    Target,
    Pending,
    Synced,
    Excluded,
}

/// 検索ソートキー
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum SortKey {
    Name,
    ProductCode,
    StockQuantity,
    SellingPrice,
}

/// 検索ソート順
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum SortOrder {
    Asc,
    Desc,
}

// ---------------------------------------------------------------------------
// 内部ヘルパー
// ---------------------------------------------------------------------------

/// rusqlite::Row → Product の変換（18カラム、インデックス0〜17）
fn row_to_product(row: &rusqlite::Row) -> rusqlite::Result<Product> {
    Ok(Product {
        product_code: row.get(0)?,
        jan_code: row.get(1)?,
        name: row.get(2)?,
        department_id: row.get(3)?,
        supplier_id: row.get(4)?,
        selling_price: row.get(5)?,
        cost_price_centi: row.get(6)?,
        tax_rate: parse_tax_rate(row.get(7)?)?,
        maker_code: row.get(8)?,
        stock_quantity: row.get(9)?,
        stock_unit: parse_stock_unit(row.get(10)?)?,
        is_discontinued: row.get(11)?,
        plu_dirty: row.get(12)?,
        plu_exported_at: row.get(13)?,
        plu_target: row.get(14)?,
        pos_stock_sync: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

/// rusqlite::Row → ProductWithRelations の変換（18カラム + dept_name + supplier_name + plu_memory_no + 写しの 3 列）
fn row_to_product_with_relations(row: &rusqlite::Row) -> rusqlite::Result<ProductWithRelations> {
    let product = row_to_product(row)?;
    Ok(ProductWithRelations {
        product,
        department_name: row.get(18)?,
        supplier_name: row.get(19)?,
        plu_memory_no: row.get(20)?,
        stock_recheck: row.get(21)?,
        stock_range_low: row.get(22)?,
        stock_range_high: row.get(23)?,
    })
}

/// rusqlite::Row → ProductForPlu の変換（18カラム + dept_name）
fn row_to_product_for_plu(row: &rusqlite::Row) -> rusqlite::Result<ProductForPlu> {
    let product = row_to_product(row)?;
    Ok(ProductForPlu {
        product,
        department_name: row.get(18)?,
    })
}

// ---------------------------------------------------------------------------
// Department 関数
// ---------------------------------------------------------------------------

/// 部門IDで部門を1件取得する
///
/// 20-io-product-repo.md §2.4
pub fn find_department_by_id(conn: &DbConnection, id: i64) -> Result<Option<Department>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, z005_name, code_prefix, next_seq, created_at
         FROM departments WHERE id = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(Department {
            id: row.get(0)?,
            name: row.get(1)?,
            z005_name: row.get(2)?,
            code_prefix: row.get(3)?,
            next_seq: row.get(4)?,
            created_at: row.get(5)?,
        })),
        None => Ok(None),
    }
}

/// 全部門を取得する（ORDER BY id ASC）
///
/// 20-io-product-repo.md §2.4
pub fn list_departments(conn: &DbConnection) -> Result<Vec<Department>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, z005_name, code_prefix, next_seq, created_at
         FROM departments ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Department {
            id: row.get(0)?,
            name: row.get(1)?,
            z005_name: row.get(2)?,
            code_prefix: row.get(3)?,
            next_seq: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    let mut depts = Vec::new();
    for row in rows {
        depts.push(row?);
    }
    Ok(depts)
}

/// departments の next_seq を+1し、インクリメント前の値を返す
///
/// 独自コード発番用。トランザクション内で呼ばれることを前提とする。
///
/// 20-io-product-repo.md §2.4
pub fn increment_next_seq(conn: &DbConnection, department_id: i64) -> Result<i64, DbError> {
    let current_seq: i64 = conn
        .query_row(
            "SELECT next_seq FROM departments WHERE id = ?1",
            rusqlite::params![department_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
            other => DbError::from(other),
        })?;

    conn.execute(
        "UPDATE departments SET next_seq = next_seq + 1 WHERE id = ?1",
        rusqlite::params![department_id],
    )?;

    Ok(current_seq)
}

/// departments の next_seq が at_least より小さいときだけ at_least へ上げる（下げない）
///
/// 独自コード発番用（BIZ-01-D6）。トランザクション内で呼ばれることを前提とする。
///
/// 20-io-product-repo.md §2.4
pub fn raise_next_seq(
    conn: &DbConnection,
    department_id: i64,
    at_least: i64,
) -> Result<(), DbError> {
    let changed = conn.execute(
        "UPDATE departments SET next_seq = MAX(next_seq, ?2) WHERE id = ?1",
        rusqlite::params![department_id, at_least],
    )?;
    if changed == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// product_code が prefix で始まる商品の product_code を全件返す（廃番を含む）
///
/// `LIKE` は大文字・小文字を同一視し `%` `_` を特殊文字にするため使わない（BIZ-01-D6）。
///
/// 20-io-product-repo.md §2.3
pub fn list_product_codes_by_prefix(
    conn: &DbConnection,
    prefix: &str,
) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code FROM products WHERE substr(product_code, 1, length(?1)) = ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![prefix], |row| row.get(0))?;
    Ok(rows.collect::<Result<Vec<String>, _>>()?)
}

// ---------------------------------------------------------------------------
// Supplier 関数
// ---------------------------------------------------------------------------

/// 全取引先を取得する
///
/// 20-io-product-repo.md §2.5
pub fn list_suppliers(conn: &DbConnection) -> Result<Vec<Supplier>, DbError> {
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM suppliers ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Supplier {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;
    let mut suppliers = Vec::new();
    for row in rows {
        suppliers.push(row?);
    }
    Ok(suppliers)
}

/// 名前で取引先を検索し、なければ作成して返す
///
/// INSERT 失敗（DuplicateKey）時は再SELECTする防御的パターン（同時実行対策）。
///
/// 20-io-product-repo.md §2.5
pub fn find_or_create_supplier(conn: &DbConnection, name: &str) -> Result<Supplier, DbError> {
    // まず既存を検索
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM suppliers WHERE name = ?1")?;
    let mut rows = stmt.query(rusqlite::params![name])?;
    if let Some(row) = rows.next()? {
        return Ok(Supplier {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        });
    }
    drop(rows);
    drop(stmt);

    // 存在しなければ作成
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    match conn.execute(
        "INSERT INTO suppliers (name, created_at) VALUES (?1, ?2)",
        rusqlite::params![name, now],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            Ok(Supplier {
                id,
                name: name.to_string(),
                created_at: now,
            })
        }
        Err(e) => {
            // 同時実行で DuplicateKey → 再SELECTで取得
            let db_err = DbError::from(e);
            if matches!(db_err, DbError::DuplicateKey(_)) {
                let mut stmt =
                    conn.prepare("SELECT id, name, created_at FROM suppliers WHERE name = ?1")?;
                let mut rows = stmt.query(rusqlite::params![name])?;
                match rows.next()? {
                    Some(row) => Ok(Supplier {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                    }),
                    None => Err(DbError::QueryFailed(
                        "取引先のINSERT後にレコードが見つかりません".to_string(),
                    )),
                }
            } else {
                Err(db_err)
            }
        }
    }
}

/// ID で取引先を1件取得する
///
/// 存在しない場合は Ok(None) を返す。
/// BIZ-02: 入庫記録の supplier_id 存在確認に使用。
pub fn find_supplier_by_id(conn: &DbConnection, id: i64) -> Result<Option<Supplier>, DbError> {
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM suppliers WHERE id = ?1")?;
    let mut rows = stmt.query(rusqlite::params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(Supplier {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })),
        None => Ok(None),
    }
}

/// BIZ で検証済みの取引先名へ改名する。
pub fn rename_supplier(
    conn: &DbConnection,
    supplier_id: i64,
    name: &str,
    updated_at: &str,
) -> Result<Supplier, DbError> {
    let updated = conn.execute(
        "UPDATE suppliers SET name=?1,updated_at=?2 WHERE id=?3",
        rusqlite::params![name, updated_at, supplier_id],
    )?;
    if updated == 0 {
        return Err(DbError::NotFound);
    }
    find_supplier_by_id(conn, supplier_id)?.ok_or(DbError::NotFound)
}

/// source の全参照を target へ付け替え、source を削除する。
pub fn merge_suppliers(
    conn: &DbConnection,
    source_id: i64,
    target_id: i64,
) -> Result<(i64, i64), DbError> {
    let products_updated = conn.execute(
        "UPDATE products SET supplier_id=?1 WHERE supplier_id=?2",
        rusqlite::params![target_id, source_id],
    )? as i64;
    let receiving_records_updated = conn.execute(
        "UPDATE receiving_records SET supplier_id=?1 WHERE supplier_id=?2",
        rusqlite::params![target_id, source_id],
    )? as i64;
    let deleted = conn.execute("DELETE FROM suppliers WHERE id=?1", [source_id])?;
    if deleted == 0 {
        return Err(DbError::NotFound);
    }
    Ok((products_updated, receiving_records_updated))
}

/// 取引先全件を商品・入庫記録の独立件数付きで name 昇順に返す。
pub(crate) fn list_suppliers_with_usage(
    conn: &DbConnection,
) -> Result<Vec<SupplierUsageRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT s.id,s.name,s.created_at,
                (SELECT COUNT(*) FROM products p WHERE p.supplier_id=s.id),
                (SELECT COUNT(*) FROM receiving_records r WHERE r.supplier_id=s.id)
         FROM suppliers s
         ORDER BY s.name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SupplierUsageRow {
            supplier: Supplier {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            },
            product_count: row.get(3)?,
            receiving_record_count: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

// ---------------------------------------------------------------------------
// Product 関数
// ---------------------------------------------------------------------------

/// product_code で商品を1件取得する（部門名・取引先名付き）
///
/// 20-io-product-repo.md §2.3
pub fn find_by_product_code(
    conn: &DbConnection,
    product_code: &str,
) -> Result<Option<ProductWithRelations>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT p.product_code, p.jan_code, p.name, p.department_id, p.supplier_id,
                p.selling_price, p.cost_price_centi, p.tax_rate, p.maker_code,
                p.stock_quantity, p.stock_unit, p.is_discontinued,
                p.plu_dirty, p.plu_exported_at, p.plu_target, p.pos_stock_sync,
                p.created_at, p.updated_at,
                d.name AS dept_name,
                s.name AS supplier_name,
                (SELECT ps.memory_no FROM plu_slots ps
                 WHERE ps.scanning_code = p.jan_code
                   AND ps.status IN ('reserved','active','release_pending')
                 ORDER BY CASE WHEN ps.status IN ('reserved','active') THEN 0 ELSE 1 END,
                          ps.memory_no
                 LIMIT 1) AS plu_memory_no,
                p.stock_recheck, p.stock_range_low, p.stock_range_high
         FROM products p
         LEFT JOIN departments d ON p.department_id = d.id
         LEFT JOIN suppliers s ON p.supplier_id = s.id
         WHERE p.product_code = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![product_code])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_product_with_relations(row)?)),
        None => Ok(None),
    }
}

/// 商品を1件INSERTする
///
/// product_code の重複チェックは呼び出し元（BIZ-01）の責務。
///
/// 20-io-product-repo.md §2.3
pub fn insert_product(conn: &DbConnection, product: &NewProduct) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO products (
            product_code, jan_code, name, department_id, supplier_id,
            selling_price, cost_price_centi, tax_rate, maker_code,
            stock_quantity, stock_unit, is_discontinued,
            plu_dirty, plu_exported_at, plu_target, pos_stock_sync,
            created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        rusqlite::params![
            product.product_code,
            product.jan_code,
            product.name,
            product.department_id,
            product.supplier_id,
            product.selling_price,
            product.cost_price_centi,
            product.tax_rate,
            product.maker_code,
            product.stock_quantity,
            product.stock_unit,
            product.is_discontinued,
            product.plu_dirty,
            product.plu_exported_at,
            product.plu_target,
            product.pos_stock_sync,
            now,
            now,
        ],
    )?;
    Ok(())
}

/// jan_code で商品を検索する（複数ヒットの可能性あり）
///
/// ORDER BY product_code ASC で返す。
///
/// 20-io-product-repo.md §2.3
pub fn find_by_jan_code(conn: &DbConnection, jan_code: &str) -> Result<Vec<Product>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code, jan_code, name, department_id, supplier_id,
                selling_price, cost_price_centi, tax_rate, maker_code,
                stock_quantity, stock_unit, is_discontinued,
                plu_dirty, plu_exported_at, plu_target, pos_stock_sync,
                created_at, updated_at
         FROM products WHERE jan_code = ?1 ORDER BY product_code ASC",
    )?;
    let rows = stmt.query_map(rusqlite::params![jan_code], row_to_product)?;
    let mut products = Vec::new();
    for row in rows {
        products.push(row?);
    }
    Ok(products)
}

/// 在庫連動の商品と同じ JAN を別の商品が持つ組（JAN の昇順、商品コードの昇順。36 の共有 JAN の違反の一覧）
pub(crate) fn list_shared_jan_conflicts(
    conn: &DbConnection,
) -> Result<Vec<(String, Vec<String>)>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT jan_code, product_code FROM products
         WHERE jan_code IN (SELECT jan_code FROM products WHERE jan_code IS NOT NULL
                            GROUP BY jan_code HAVING COUNT(*) > 1 AND MAX(pos_stock_sync) = 1)
         ORDER BY jan_code, product_code",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for (jan, code) in rows {
        match groups.last_mut() {
            Some((last, codes)) if *last == jan => codes.push(code),
            _ => groups.push((jan, vec![code])),
        }
    }
    Ok(groups)
}

/// plu_dirty=1 の商品一覧を返す（PLU書出しの差分モード、BIZ-04 list_plu_dirty 用）
///
/// 20-io-product-repo.md §2.3 find_plu_dirty_products
pub fn find_plu_dirty_products(conn: &DbConnection) -> Result<Vec<Product>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code, jan_code, name, department_id, supplier_id,
                selling_price, cost_price_centi, tax_rate, maker_code,
                stock_quantity, stock_unit, is_discontinued,
                plu_dirty, plu_exported_at, plu_target, pos_stock_sync,
                created_at, updated_at
         FROM products WHERE plu_dirty = 1 AND plu_target = 1 ORDER BY product_code ASC",
    )?;
    let rows = stmt.query_map([], row_to_product)?;
    let mut products = Vec::new();
    for row in rows {
        products.push(row?);
    }
    Ok(products)
}

/// plu_dirty=1 の商品一覧を部門名付きで返す（PLU書出し差分モード、IO-04入力用）
///
/// 20-io-product-repo.md §2.3 find_plu_dirty_products_for_plu
pub fn find_plu_dirty_products_for_plu(conn: &DbConnection) -> Result<Vec<ProductForPlu>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT p.product_code, p.jan_code, p.name, p.department_id, p.supplier_id,
                p.selling_price, p.cost_price_centi, p.tax_rate, p.maker_code,
                p.stock_quantity, p.stock_unit, p.is_discontinued,
                p.plu_dirty, p.plu_exported_at, p.plu_target, p.pos_stock_sync,
                p.created_at, p.updated_at,
                d.name as department_name
         FROM products p
         INNER JOIN departments d ON p.department_id = d.id
         WHERE p.plu_dirty = 1 AND p.plu_target = 1
         ORDER BY p.product_code ASC",
    )?;
    let rows = stmt.query_map([], row_to_product_for_plu)?;
    let mut products = Vec::new();
    for row in rows {
        products.push(row?);
    }
    Ok(products)
}

/// 有効な（廃番でない）商品一覧を返す（PLU書出しの全件モード用）
///
/// 20-io-product-repo.md §2.3 find_active_products
pub fn find_active_products(conn: &DbConnection) -> Result<Vec<Product>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code, jan_code, name, department_id, supplier_id,
                selling_price, cost_price_centi, tax_rate, maker_code,
                stock_quantity, stock_unit, is_discontinued,
                plu_dirty, plu_exported_at, plu_target, pos_stock_sync,
                created_at, updated_at
         FROM products WHERE is_discontinued = 0 ORDER BY product_code ASC",
    )?;
    let rows = stmt.query_map([], row_to_product)?;
    let mut products = Vec::new();
    for row in rows {
        products.push(row?);
    }
    Ok(products)
}

/// 有効な商品一覧を部門名付きで返す（PLU書出し全件モード、IO-04入力用）
///
/// 20-io-product-repo.md §2.3 find_active_products_for_plu
pub fn find_active_products_for_plu(conn: &DbConnection) -> Result<Vec<ProductForPlu>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT p.product_code, p.jan_code, p.name, p.department_id, p.supplier_id,
                p.selling_price, p.cost_price_centi, p.tax_rate, p.maker_code,
                p.stock_quantity, p.stock_unit, p.is_discontinued,
                p.plu_dirty, p.plu_exported_at, p.plu_target, p.pos_stock_sync,
                p.created_at, p.updated_at,
                d.name as department_name
         FROM products p
         INNER JOIN departments d ON p.department_id = d.id
         WHERE p.is_discontinued = 0 AND p.plu_target = 1
         ORDER BY p.product_code ASC",
    )?;
    let rows = stmt.query_map([], row_to_product_for_plu)?;
    let mut products = Vec::new();
    for row in rows {
        products.push(row?);
    }
    Ok(products)
}

/// 検索条件に基づいて商品一覧をページング取得する
///
/// page >= 1, per_page >= 1 でなければ DbError::QueryFailed を返す。
///
/// 20-io-product-repo.md §2.3
pub fn search_products(
    conn: &DbConnection,
    query: &ProductSearchQuery,
) -> Result<PaginatedResult<ProductWithRelations>, DbError> {
    // ページングパラメータのガード
    if query.page < 1 {
        return Err(DbError::QueryFailed("page must be >= 1".to_string()));
    }
    if query.per_page < 1 {
        return Err(DbError::QueryFailed("per_page must be >= 1".to_string()));
    }
    let per_page = query.per_page.min(PAGINATION_MAX_PER_PAGE);

    // WHERE句の構築
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref keyword) = query.keyword {
        let like = format!("%{}%", keyword);
        let idx = params.len() + 1;
        conditions.push(format!(
            "(p.name LIKE ?{idx} OR p.product_code LIKE ?{idx} OR p.jan_code LIKE ?{idx} OR p.maker_code LIKE ?{idx})",
        ));
        params.push(Box::new(like));
    }

    if let Some(dept_id) = query.department_id {
        conditions.push(format!("p.department_id = ?{}", params.len() + 1));
        params.push(Box::new(dept_id));
    }

    if let Some(supplier_id) = query.supplier_id {
        let idx = params.len() + 1;
        if query.include_unassigned {
            conditions.push(format!("(p.supplier_id = ?{idx} OR p.supplier_id IS NULL)"));
        } else {
            conditions.push(format!("p.supplier_id = ?{idx}"));
        }
        params.push(Box::new(supplier_id));
    }

    if let Some(discontinued) = query.is_discontinued {
        conditions.push(format!("p.is_discontinued = ?{}", params.len() + 1));
        params.push(Box::new(discontinued));
    }

    match query.plu {
        Some(PluMigrationFilter::Target) => conditions.push("p.plu_target = 1".to_string()),
        Some(PluMigrationFilter::Pending) => {
            conditions.push("p.plu_target = 1 AND p.plu_dirty = 1".to_string());
        }
        Some(PluMigrationFilter::Synced) => {
            conditions.push("p.plu_target = 1 AND p.plu_dirty = 0".to_string());
        }
        Some(PluMigrationFilter::Excluded) => {
            conditions.push("p.plu_target = 0".to_string());
        }
        Some(PluMigrationFilter::All) | None => {}
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // ORDER BY句
    let order_column = match query.sort_key {
        SortKey::Name => "p.name",
        SortKey::ProductCode => "p.product_code",
        SortKey::StockQuantity => "p.stock_quantity",
        SortKey::SellingPrice => "p.selling_price",
    };
    let order_dir = match query.sort_order {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    // COUNT取得
    let count_sql = format!("SELECT COUNT(*) FROM products p {}", where_clause);
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let total_count: u32 = conn.query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))?;

    // データ取得
    let offset = query
        .page
        .checked_sub(1)
        .and_then(|p| p.checked_mul(per_page))
        .ok_or_else(|| DbError::QueryFailed("page/per_page overflow".to_string()))?;
    let data_sql = format!(
        "SELECT p.product_code, p.jan_code, p.name, p.department_id, p.supplier_id,
                p.selling_price, p.cost_price_centi, p.tax_rate, p.maker_code,
                p.stock_quantity, p.stock_unit, p.is_discontinued,
                p.plu_dirty, p.plu_exported_at, p.plu_target, p.pos_stock_sync,
                p.created_at, p.updated_at,
                d.name AS dept_name,
                s.name AS supplier_name,
                (SELECT ps.memory_no FROM plu_slots ps
                 WHERE ps.scanning_code = p.jan_code
                   AND ps.status IN ('reserved','active','release_pending')
                 ORDER BY CASE WHEN ps.status IN ('reserved','active') THEN 0 ELSE 1 END,
                          ps.memory_no
                 LIMIT 1) AS plu_memory_no,
                p.stock_recheck, p.stock_range_low, p.stock_range_high
         FROM products p
         LEFT JOIN departments d ON p.department_id = d.id
         LEFT JOIN suppliers s ON p.supplier_id = s.id
         {} ORDER BY {} {} LIMIT {} OFFSET {}",
        where_clause, order_column, order_dir, per_page, offset
    );
    let mut stmt = conn.prepare(&data_sql)?;
    let rows = stmt.query_map(params_ref.as_slice(), |row| {
        row_to_product_with_relations(row)
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(PaginatedResult {
        items,
        total_count,
        page: query.page,
        per_page,
    })
}

/// PLU 対象一括更新用に filter 一致全件をページングなしで返す。
pub fn find_products_for_bulk_plu_target(
    conn: &DbConnection,
    filter: &ProductBulkFilter,
) -> Result<Vec<Product>, DbError> {
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref keyword) = filter.keyword {
        let like = format!("%{}%", keyword);
        let idx = params.len() + 1;
        conditions.push(format!(
            "(p.name LIKE ?{idx} OR p.product_code LIKE ?{idx} OR p.jan_code LIKE ?{idx} OR p.maker_code LIKE ?{idx})"
        ));
        params.push(Box::new(like));
    }
    if let Some(department_id) = filter.department_id {
        conditions.push(format!("p.department_id = ?{}", params.len() + 1));
        params.push(Box::new(department_id));
    }
    if let Some(is_discontinued) = filter.is_discontinued {
        conditions.push(format!("p.is_discontinued = ?{}", params.len() + 1));
        params.push(Box::new(is_discontinued));
    }
    match filter.plu {
        Some(PluMigrationFilter::Target) => conditions.push("p.plu_target = 1".to_string()),
        Some(PluMigrationFilter::Pending) => {
            conditions.push("p.plu_target = 1 AND p.plu_dirty = 1".to_string());
        }
        Some(PluMigrationFilter::Synced) => {
            conditions.push("p.plu_target = 1 AND p.plu_dirty = 0".to_string());
        }
        Some(PluMigrationFilter::Excluded) => conditions.push("p.plu_target = 0".to_string()),
        Some(PluMigrationFilter::All) | None => {}
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT p.product_code, p.jan_code, p.name, p.department_id, p.supplier_id,
                p.selling_price, p.cost_price_centi, p.tax_rate, p.maker_code,
                p.stock_quantity, p.stock_unit, p.is_discontinued,
                p.plu_dirty, p.plu_exported_at, p.plu_target, p.pos_stock_sync,
                p.created_at, p.updated_at
         FROM products p {where_clause} ORDER BY p.product_code ASC"
    );
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_ref.as_slice(), row_to_product)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// 商品の指定フィールドを更新する
///
/// Someのフィールドだけ更新。updated_at は常に更新。
/// 該当商品なしの場合は Ok(false) を返す。
///
/// 20-io-product-repo.md §2.3
pub fn update_product(
    conn: &DbConnection,
    product_code: &str,
    updates: &ProductUpdates,
) -> Result<bool, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    // SET句の動的構築
    let mut set_clauses = vec!["updated_at = ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
    let mut idx = 2;

    if let Some(ref name) = updates.name {
        set_clauses.push(format!("name = ?{}", idx));
        params.push(Box::new(name.clone()));
        idx += 1;
    }
    if let Some(ref jan_code) = updates.jan_code {
        set_clauses.push(format!("jan_code = ?{}", idx));
        params.push(Box::new(jan_code.clone()));
        idx += 1;
    }
    if let Some(dept_id) = updates.department_id {
        set_clauses.push(format!("department_id = ?{}", idx));
        params.push(Box::new(dept_id));
        idx += 1;
    }
    if let Some(ref supplier_id) = updates.supplier_id {
        set_clauses.push(format!("supplier_id = ?{}", idx));
        // Option<Option<i64>>: Some(None) → NULL, Some(Some(v)) → v
        let val: Option<i64> = *supplier_id;
        params.push(Box::new(val));
        idx += 1;
    }
    if let Some(price) = updates.selling_price {
        set_clauses.push(format!("selling_price = ?{}", idx));
        params.push(Box::new(price));
        idx += 1;
    }
    if let Some(cost) = updates.cost_price_centi {
        set_clauses.push(format!("cost_price_centi = ?{}", idx));
        params.push(Box::new(cost));
        idx += 1;
    }
    if let Some(ref rate) = updates.tax_rate {
        set_clauses.push(format!("tax_rate = ?{}", idx));
        params.push(Box::new(rate.clone()));
        idx += 1;
    }
    if let Some(ref maker_code) = updates.maker_code {
        set_clauses.push(format!("maker_code = ?{}", idx));
        let val: Option<String> = maker_code.clone();
        params.push(Box::new(val));
        idx += 1;
    }
    if let Some(unit) = updates.stock_unit {
        set_clauses.push(format!("stock_unit = ?{}", idx));
        params.push(Box::new(unit.as_str()));
        idx += 1;
    }
    if let Some(disc) = updates.is_discontinued {
        set_clauses.push(format!("is_discontinued = ?{}", idx));
        params.push(Box::new(disc));
        idx += 1;
    }
    if let Some(dirty) = updates.plu_dirty {
        set_clauses.push(format!("plu_dirty = ?{}", idx));
        params.push(Box::new(dirty));
        idx += 1;
    }
    if let Some(ref exported_at) = updates.plu_exported_at {
        set_clauses.push(format!("plu_exported_at = ?{}", idx));
        // Option<Option<String>>: Some(None) → NULL, Some(Some(v)) → v
        let val: Option<String> = exported_at.clone();
        params.push(Box::new(val));
        idx += 1;
    }
    if let Some(target) = updates.plu_target {
        set_clauses.push(format!("plu_target = ?{}", idx));
        params.push(Box::new(target));
        idx += 1;
    }
    if let Some(sync) = updates.pos_stock_sync {
        set_clauses.push(format!("pos_stock_sync = ?{}", idx));
        params.push(Box::new(sync));
        idx += 1;
    }

    // WHERE句のパラメータ
    params.push(Box::new(product_code.to_string()));

    let sql = format!(
        "UPDATE products SET {} WHERE product_code = ?{}",
        set_clauses.join(", "),
        idx
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let affected = conn.execute(&sql, params_ref.as_slice())?;

    Ok(affected > 0)
}

// ---------------------------------------------------------------------------
// Price History 関数
// ---------------------------------------------------------------------------

/// 価格変更履歴を1行INSERTする
///
/// 20-io-product-repo.md §2.6
pub fn insert_price_history(conn: &DbConnection, history: &NewPriceHistory) -> Result<(), DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO price_history (product_code, old_selling, new_selling, old_cost_centi, new_cost_centi, changed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            history.product_code,
            history.old_selling,
            history.new_selling,
            history.old_cost_centi,
            history.new_cost_centi,
            now,
        ],
    )?;
    Ok(())
}

/// 商品コードに紐付く価格履歴を新しい順で取得する。
pub fn list_price_history(
    conn: &DbConnection,
    product_code: &str,
    limit: u32,
) -> Result<Vec<PriceHistoryEntry>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, old_selling, new_selling, old_cost_centi, new_cost_centi, changed_at
         FROM price_history
         WHERE product_code = ?1
         ORDER BY changed_at DESC, id DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(
        rusqlite::params![product_code, i64::from(limit.min(100))],
        |row| {
            Ok(PriceHistoryEntry {
                id: row.get(0)?,
                old_selling_price: row.get(1)?,
                new_selling_price: row.get(2)?,
                old_cost_price_centi: row.get(3)?,
                new_cost_price_centi: row.get(4)?,
                changed_at: row.get(5)?,
            })
        },
    )?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// 整合性の突合が読む商品の写しの 4 項目と単位・名前（BIZ-07、36）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockCopyRow {
    pub product_code: String,
    pub name: String,
    pub stock_unit: ProductStockUnit,
    pub stock_quantity: i64,
    pub stock_recheck: bool,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
}

/// 全商品の写しの 4 項目と単位・名前を商品コードの順で取得する（BIZ-07 整合性チェック用）
///
/// 20-io-product-repo.md の `find_all_stock_quantities` の項
pub fn find_all_stock_quantities(conn: &DbConnection) -> Result<Vec<StockCopyRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code, name, stock_unit, stock_quantity, stock_recheck,
                stock_range_low, stock_range_high
         FROM products ORDER BY product_code ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(StockCopyRow {
                product_code: row.get(0)?,
                name: row.get(1)?,
                stock_unit: parse_stock_unit(row.get(2)?)?,
                stock_quantity: row.get(3)?,
                stock_recheck: row.get(4)?,
                stock_range_low: row.get(5)?,
                stock_range_high: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// CMD-06: 在庫照会用関数
// ---------------------------------------------------------------------------

/// 在庫詳細（商品情報 + 最終入庫日 + 最終販売日）
///
/// 44-cmd-inventory.md §23.9 get_stock_detail
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StockDetail {
    pub product: ProductWithRelations,
    pub last_receiving_date: Option<String>,
    pub last_sale_date: Option<String>,
}

/// 商品の在庫詳細を、最終入庫日・最終販売日付きで取得する
///
/// 44-cmd-inventory.md §23.9 get_stock_detail
pub fn get_stock_detail(conn: &DbConnection, product_code: &str) -> Result<StockDetail, DbError> {
    let mut stmt = conn.prepare(
        "SELECT p.product_code, p.jan_code, p.name, p.department_id, p.supplier_id,
                p.selling_price, p.cost_price_centi, p.tax_rate, p.maker_code,
                p.stock_quantity, p.stock_unit, p.is_discontinued,
                p.plu_dirty, p.plu_exported_at, p.plu_target, p.pos_stock_sync,
                p.created_at, p.updated_at,
                d.name AS dept_name,
                s.name AS supplier_name,
                (SELECT ps.memory_no FROM plu_slots ps
                 WHERE ps.scanning_code = p.jan_code
                   AND ps.status IN ('reserved','active','release_pending')
                 ORDER BY CASE WHEN ps.status IN ('reserved','active') THEN 0 ELSE 1 END,
                          ps.memory_no
                 LIMIT 1) AS plu_memory_no,
                p.stock_recheck, p.stock_range_low, p.stock_range_high,
                (SELECT MAX(rr.receiving_date)
                 FROM receiving_items ri
                 JOIN receiving_records rr ON ri.receiving_record_id = rr.id
                 WHERE ri.product_code = p.product_code) AS last_receiving_date,
                (SELECT MAX(sr.sale_date)
                 FROM sale_records sr
                 WHERE sr.product_code = p.product_code AND sr.is_voided = 0) AS last_sale_date
         FROM products p
         LEFT JOIN departments d ON p.department_id = d.id
         LEFT JOIN suppliers s ON p.supplier_id = s.id
         WHERE p.product_code = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![product_code])?;
    match rows.next()? {
        Some(row) => {
            let product = row_to_product_with_relations(row)?;
            Ok(StockDetail {
                product,
                last_receiving_date: row.get(24)?,
                last_sale_date: row.get(25)?,
            })
        }
        None => Err(DbError::NotFound),
    }
}

/// 在庫が閾値以下の商品を一覧取得する
///
/// 44-cmd-inventory.md §23.9 list_low_stock_products。個数の単位は一般の基準、長さの単位は生地の基準で比べる
/// （SPEC-UNIT-D10）。単位の 2 つの並びは BIZ が作って渡す（db の層は数量の種類を知らない）
pub fn list_low_stock_products(
    conn: &DbConnection,
    count_units: &[ProductStockUnit],
    threshold_count: i64,
    length_units: &[ProductStockUnit],
    threshold_length: i64,
    include_discontinued: bool,
) -> Result<Vec<ProductWithRelations>, DbError> {
    let discontinued_clause = if include_discontinued {
        ""
    } else {
        "AND p.is_discontinued = 0"
    };
    let placeholders = |units: &[ProductStockUnit]| vec!["?"; units.len()].join(",");
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    params.extend(
        count_units
            .iter()
            .map(|unit| unit.as_str().to_string().into()),
    );
    params.push(threshold_count.into());
    params.extend(
        length_units
            .iter()
            .map(|unit| unit.as_str().to_string().into()),
    );
    params.push(threshold_length.into());
    let sql = format!(
        "SELECT p.product_code, p.jan_code, p.name, p.department_id, p.supplier_id,
                p.selling_price, p.cost_price_centi, p.tax_rate, p.maker_code,
                p.stock_quantity, p.stock_unit, p.is_discontinued,
                p.plu_dirty, p.plu_exported_at, p.plu_target, p.pos_stock_sync,
                p.created_at, p.updated_at,
                d.name AS dept_name,
                s.name AS supplier_name,
                (SELECT ps.memory_no FROM plu_slots ps
                 WHERE ps.scanning_code = p.jan_code
                   AND ps.status IN ('reserved','active','release_pending')
                 ORDER BY CASE WHEN ps.status IN ('reserved','active') THEN 0 ELSE 1 END,
                          ps.memory_no
                 LIMIT 1) AS plu_memory_no,
                p.stock_recheck, p.stock_range_low, p.stock_range_high
         FROM products p
         LEFT JOIN departments d ON p.department_id = d.id
         LEFT JOIN suppliers s ON p.supplier_id = s.id
         WHERE ((p.stock_unit IN ({}) AND p.stock_quantity <= ?)
             OR (p.stock_unit IN ({}) AND p.stock_quantity <= ?))
         {}
         ORDER BY p.stock_quantity ASC, p.name ASC",
        placeholders(count_units),
        placeholders(length_units),
        discontinued_clause
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        row_to_product_with_relations(row)
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_database;

    /// テスト用DB接続を作成するヘルパー
    fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    /// テスト用商品を作成するヘルパー
    fn create_test_product(product_code: &str, name: &str, department_id: i64) -> NewProduct {
        NewProduct {
            product_code: product_code.to_string(),
            jan_code: None,
            name: name.to_string(),
            department_id,
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
        }
    }

    // ===== FUNC-2.4: Department関数テスト =====

    #[test]
    fn test_find_department_by_id_req101_existing() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.4: find_department_by_id — 存在するIDで部門が取得できる
        let (_dir, conn) = setup_test_db();
        let dept = find_department_by_id(&conn, 1).unwrap();
        assert!(dept.is_some(), "id=1 の部門が存在するべき");
        let dept = dept.unwrap();
        assert_eq!(dept.name, "その他小物");
        assert_eq!(dept.code_prefix, Some("KM".to_string()));
    }

    #[test]
    fn test_find_department_by_id_req101_nonexistent() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.4: find_department_by_id — 存在しないIDでNone
        let (_dir, conn) = setup_test_db();
        let dept = find_department_by_id(&conn, 9999).unwrap();
        assert!(dept.is_none(), "存在しないIDはNoneを返すべき");
    }

    #[test]
    fn test_list_departments_req101_returns_all_21() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.4: list_departments — 初期データ21部門を全件取得
        let (_dir, conn) = setup_test_db();
        let depts = list_departments(&conn).unwrap();
        assert_eq!(depts.len(), 21, "初期データは21部門");
    }

    #[test]
    fn test_list_departments_req101_ordered_by_id() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.4: list_departments — ORDER BY id ASC で返る
        let (_dir, conn) = setup_test_db();
        let depts = list_departments(&conn).unwrap();
        for i in 1..depts.len() {
            assert!(
                depts[i].id > depts[i - 1].id,
                "部門はid昇順で返るべき: id={} の後に id={}",
                depts[i - 1].id,
                depts[i].id
            );
        }
    }

    #[test]
    fn test_increment_next_seq_req101_normal() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.4: increment_next_seq — 正常な連番インクリメント
        let (_dir, conn) = setup_test_db();
        let seq = increment_next_seq(&conn, 2).unwrap();
        assert_eq!(seq, 1, "最初の発番は1を返すべき");

        // 2回目は2を返す
        let seq = increment_next_seq(&conn, 2).unwrap();
        assert_eq!(seq, 2, "2回目の発番は2を返すべき");

        // DBの next_seq が 3 になっていることを確認
        let dept = find_department_by_id(&conn, 2).unwrap().unwrap();
        assert_eq!(dept.next_seq, 3, "next_seqは3になっているべき");
    }

    #[test]
    fn test_increment_next_seq_req101_nonexistent_department() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.4: increment_next_seq — 存在しない部門IDでNotFound
        let (_dir, conn) = setup_test_db();
        let result = increment_next_seq(&conn, 9999);
        assert!(
            matches!(result, Err(DbError::NotFound)),
            "存在しない部門IDで NotFound エラーが返るべき: {:?}",
            result
        );
    }

    #[test]
    fn test_raise_next_seq_req101_raises_only() {
        // REQ-101: 商品登録（独自コード発番）
        // FUNC-2.4 / BIZ-01-D6: raise_next_seq — 上げるだけで下げない。部門が無ければ NotFound
        let (_dir, conn) = setup_test_db();
        raise_next_seq(&conn, 2, 5).unwrap();
        assert_eq!(
            find_department_by_id(&conn, 2).unwrap().unwrap().next_seq,
            5
        );
        raise_next_seq(&conn, 2, 3).unwrap();
        assert_eq!(
            find_department_by_id(&conn, 2).unwrap().unwrap().next_seq,
            5,
            "小さい値では下げない"
        );
        assert!(matches!(
            raise_next_seq(&conn, 9999, 5),
            Err(DbError::NotFound)
        ));
    }

    #[test]
    fn test_list_product_codes_by_prefix_req101_exact_prefix() {
        // REQ-101: 商品登録（独自コード発番）
        // FUNC-2.3 / BIZ-01-D6: list_product_codes_by_prefix — 大文字・小文字を区別し、% を wildcard にしない。廃番を含む
        let (_dir, conn) = setup_test_db();
        for code in ["HZ-0001", "hz-0009", "HZX-0009", "H%-0001"] {
            insert_product(&conn, &create_test_product(code, "合成", 2)).unwrap();
        }
        let mut discontinued = create_test_product("HZ-0002", "合成廃番", 2);
        discontinued.is_discontinued = true;
        insert_product(&conn, &discontinued).unwrap();

        let mut codes = list_product_codes_by_prefix(&conn, "HZ-").unwrap();
        codes.sort();
        assert_eq!(codes, vec!["HZ-0001", "HZ-0002"]);
        assert_eq!(
            list_product_codes_by_prefix(&conn, "H%-").unwrap(),
            vec!["H%-0001"]
        );
        assert!(list_product_codes_by_prefix(&conn, "QQ-")
            .unwrap()
            .is_empty());
    }

    // ===== FUNC-2.5: Supplier関数テスト =====

    #[test]
    fn test_list_suppliers_req101_empty() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.5: list_suppliers — 初期データなしで空リスト
        let (_dir, conn) = setup_test_db();
        let suppliers = list_suppliers(&conn).unwrap();
        assert!(suppliers.is_empty(), "初期データに取引先はないので空");
    }

    #[test]
    fn test_find_or_create_supplier_req101_creates_new() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.5: find_or_create_supplier — 新規作成
        let (_dir, conn) = setup_test_db();
        let supplier = find_or_create_supplier(&conn, "ハマナカ").unwrap();
        assert_eq!(supplier.name, "ハマナカ");
        assert!(supplier.id > 0);

        // 作成後に list_suppliers で1件取れる
        let suppliers = list_suppliers(&conn).unwrap();
        assert_eq!(suppliers.len(), 1);
    }

    #[test]
    fn test_find_or_create_supplier_req101_finds_existing() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.5: find_or_create_supplier — 既存レコードを返す
        let (_dir, conn) = setup_test_db();
        let s1 = find_or_create_supplier(&conn, "ハマナカ").unwrap();
        let s2 = find_or_create_supplier(&conn, "ハマナカ").unwrap();
        assert_eq!(s1.id, s2.id, "同名は同じレコードを返すべき");

        let suppliers = list_suppliers(&conn).unwrap();
        assert_eq!(suppliers.len(), 1, "重複作成されていないこと");
    }

    #[test]
    fn test_list_suppliers_with_usage_counts() {
        // REQ-107 / SPEC-SUP-D8: 2 参照表を独立集約し、0 件を含め name 昇順で返す。
        let (_dir, conn) = setup_test_db();
        let supplier_a = find_or_create_supplier(&conn, "あ取引先").unwrap().id;
        let supplier_b = find_or_create_supplier(&conn, "か取引先").unwrap().id;
        find_or_create_supplier(&conn, "さ取引先").unwrap();

        for (code, supplier_id) in [
            ("SUP-A-1", supplier_a),
            ("SUP-A-2", supplier_a),
            ("SUP-B-1", supplier_b),
        ] {
            let mut product = create_test_product(code, "テスト商品", 1);
            product.supplier_id = Some(supplier_id);
            insert_product(&conn, &product).unwrap();
        }
        for (key, supplier_id) in [("usage-a-1", supplier_a), ("usage-a-2", supplier_a)] {
            conn.execute(
                "INSERT INTO receiving_records
                 (supplier_id,receiving_date,note,idempotency_key,request_fingerprint,created_at)
                 VALUES(?1,'2026-08-25',NULL,?2,?2,'2026-08-25T10:00:00')",
                rusqlite::params![supplier_id, key],
            )
            .unwrap();
        }

        let rows = list_suppliers_with_usage(&conn).unwrap();
        let actual: Vec<_> = rows
            .into_iter()
            .map(|row| {
                (
                    row.supplier.name,
                    row.product_count,
                    row.receiving_record_count,
                )
            })
            .collect();
        assert_eq!(
            actual,
            vec![
                ("あ取引先".to_string(), 2, 2),
                ("か取引先".to_string(), 1, 0),
                ("さ取引先".to_string(), 0, 0),
            ]
        );
    }

    // ===== FUNC-2.3: Product CRUDテスト =====

    #[test]
    fn test_insert_product_req101_normal() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.3: insert_product — 正常INSERT
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("4976383262108", "ハマナカ アミアミ極太", 3);
        insert_product(&conn, &product).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE product_code = ?1",
                rusqlite::params!["4976383262108"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_product_req101_duplicate_key() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.3: insert_product — PK重複でDuplicateKey
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("TEST-001", "テスト商品A", 1);
        insert_product(&conn, &product).unwrap();

        let product2 = create_test_product("TEST-001", "テスト商品B", 1);
        let result = insert_product(&conn, &product2);
        assert!(
            matches!(result, Err(DbError::DuplicateKey(_))),
            "PK重複で DuplicateKey エラーが返るべき: {:?}",
            result
        );
    }

    #[test]
    fn test_insert_product_req101_fk_violation_department() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.3: insert_product — 不正department_idでForeignKeyViolation
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("TEST-FK", "FK違反テスト", 9999);
        let result = insert_product(&conn, &product);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "不正なdepartment_idで ForeignKeyViolation が返るべき: {:?}",
            result
        );
    }

    #[test]
    fn test_insert_product_req101_fk_violation_supplier() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // FUNC-2.3: insert_product — 不正supplier_idでForeignKeyViolation
        let (_dir, conn) = setup_test_db();
        let mut product = create_test_product("TEST-FKS", "FK違反テスト", 1);
        product.supplier_id = Some(9999);
        let result = insert_product(&conn, &product);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "不正なsupplier_idで ForeignKeyViolation が返るべき: {:?}",
            result
        );
    }

    #[test]
    fn test_find_by_product_code_req103_existing() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: find_by_product_code — 存在する商品をJOIN付きで取得
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("FIND-001", "検索テスト商品", 1);
        insert_product(&conn, &product).unwrap();

        let found = find_by_product_code(&conn, "FIND-001").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.product.name, "検索テスト商品");
        assert_eq!(found.department_name, "その他小物");
    }

    #[test]
    fn test_find_by_product_code_req103_nonexistent() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: find_by_product_code — 存在しないコードでNone
        let (_dir, conn) = setup_test_db();
        let found = find_by_product_code(&conn, "NONEXISTENT").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_find_by_product_code_req103_null_supplier() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: find_by_product_code — supplier_id=NULLでLEFT JOINがNone
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("NULL-SUP", "取引先なし商品", 1);
        insert_product(&conn, &product).unwrap();

        let found = find_by_product_code(&conn, "NULL-SUP").unwrap().unwrap();
        assert!(
            found.supplier_name.is_none(),
            "supplier_id=NULLならsupplier_nameもNone"
        );
    }

    #[test]
    fn test_find_by_product_code_req103_with_supplier() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: find_by_product_code — supplier_id設定時にsupplier_name取得
        let (_dir, conn) = setup_test_db();
        let supplier = find_or_create_supplier(&conn, "ハマナカ").unwrap();
        let mut product = create_test_product("WITH-SUP", "取引先あり商品", 1);
        product.supplier_id = Some(supplier.id);
        insert_product(&conn, &product).unwrap();

        let found = find_by_product_code(&conn, "WITH-SUP").unwrap().unwrap();
        assert_eq!(found.supplier_name, Some("ハマナカ".to_string()));
    }

    #[test]
    fn test_find_by_jan_code_req103_no_match() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: find_by_jan_code — 該当なしで空Vec
        let (_dir, conn) = setup_test_db();
        let products = find_by_jan_code(&conn, "0000000000000").unwrap();
        assert!(products.is_empty());
    }

    #[test]
    fn test_find_by_jan_code_req103_single() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: find_by_jan_code — 1件ヒット
        let (_dir, conn) = setup_test_db();
        let mut product = create_test_product("JAN-001", "JAN検索テスト", 1);
        product.jan_code = Some("4976383262108".to_string());
        insert_product(&conn, &product).unwrap();

        let products = find_by_jan_code(&conn, "4976383262108").unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].product_code, "JAN-001");
    }

    #[test]
    fn test_find_by_jan_code_req103_multiple_ordered_by_product_code() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: find_by_jan_code — 複数ヒット時にORDER BY product_code ASC
        let (_dir, conn) = setup_test_db();
        let mut p1 = create_test_product("FS-0002", "ファスナーB", 19);
        p1.jan_code = Some("4976383999999".to_string());
        insert_product(&conn, &p1).unwrap();

        let mut p2 = create_test_product("FS-0001", "ファスナーA", 19);
        p2.jan_code = Some("4976383999999".to_string());
        insert_product(&conn, &p2).unwrap();

        let products = find_by_jan_code(&conn, "4976383999999").unwrap();
        assert_eq!(products.len(), 2);
        // ORDER BY product_code ASC であること
        assert_eq!(products[0].product_code, "FS-0001");
        assert_eq!(products[1].product_code, "FS-0002");
    }

    // ===== FUNC-2.3: search_products テスト =====

    /// デフォルト検索条件を作るヘルパー
    fn default_search_query() -> ProductSearchQuery {
        ProductSearchQuery {
            keyword: None,
            department_id: None,
            supplier_id: None,
            include_unassigned: false,
            is_discontinued: None,
            plu: None,
            sort_key: SortKey::ProductCode,
            sort_order: SortOrder::Asc,
            page: 1,
            per_page: 50,
        }
    }

    #[test]
    fn test_search_products_req907_filters_plu_migration_states() {
        let (_dir, conn) = setup_test_db();
        let fixtures = [
            ("PLU-EXCLUDED-A", false, false, false),
            ("PLU-PENDING", true, true, false),
            ("PLU-SYNCED", true, false, false),
            ("PLU-EXCLUDED-D", false, false, true),
        ];
        for (code, target, dirty, discontinued) in fixtures {
            let mut product = create_test_product(code, code, 1);
            product.plu_target = target;
            product.plu_dirty = dirty;
            product.is_discontinued = discontinued;
            insert_product(&conn, &product).unwrap();
        }

        let cases = [
            (PluMigrationFilter::All, 4),
            (PluMigrationFilter::Target, 2),
            (PluMigrationFilter::Pending, 1),
            (PluMigrationFilter::Synced, 1),
            (PluMigrationFilter::Excluded, 2),
        ];
        for (filter, expected) in cases {
            let mut query = default_search_query();
            query.plu = Some(filter);
            let result = search_products(&conn, &query).unwrap();
            assert_eq!(result.total_count, expected, "filter={filter:?}");
        }

        let mut active_excluded = default_search_query();
        active_excluded.plu = Some(PluMigrationFilter::Excluded);
        active_excluded.is_discontinued = Some(false);
        assert_eq!(
            search_products(&conn, &active_excluded)
                .unwrap()
                .total_count,
            1
        );
    }

    #[test]
    fn test_product_search_query_req907_deserializes_optional_plu_filter() {
        let base = serde_json::json!({
            "keyword": null,
            "department_id": null,
            "is_discontinued": null,
            "sort_key": "ProductCode",
            "sort_order": "Asc",
            "page": 1,
            "per_page": 50
        });
        let omitted: ProductSearchQuery = serde_json::from_value(base.clone()).unwrap();
        assert_eq!(omitted.plu, None);

        let mut with_null = base.clone();
        with_null["plu"] = serde_json::Value::Null;
        let with_null: ProductSearchQuery = serde_json::from_value(with_null).unwrap();
        assert_eq!(with_null.plu, None);

        let mut with_pending = base.clone();
        with_pending["plu"] = serde_json::json!("pending");
        let with_pending: ProductSearchQuery = serde_json::from_value(with_pending).unwrap();
        assert_eq!(with_pending.plu, Some(PluMigrationFilter::Pending));

        let mut bogus = base;
        bogus["plu"] = serde_json::json!("bogus");
        assert!(serde_json::from_value::<ProductSearchQuery>(bogus).is_err());
    }

    /// テストデータ（複数商品）を投入するヘルパー
    fn seed_products_for_search(conn: &DbConnection) {
        let products = vec![
            {
                let mut p = create_test_product("HZ-0001", "ヘアゴムA", 2);
                p.selling_price = 300;
                p.stock_quantity = 10;
                p
            },
            {
                let mut p = create_test_product("HZ-0002", "ヘアゴムB", 2);
                p.selling_price = 500;
                p.stock_quantity = 5;
                p.is_discontinued = true;
                p
            },
            {
                let mut p = create_test_product("KM-0001", "ボタンA", 1);
                p.selling_price = 100;
                p.stock_quantity = 20;
                p
            },
            {
                let mut p = create_test_product("4976383262108", "ハマナカ アミアミ極太", 3);
                p.jan_code = Some("4976383262108".to_string());
                p.selling_price = 594;
                p.stock_quantity = 15;
                p
            },
        ];
        for p in &products {
            insert_product(conn, p).unwrap();
        }
    }

    /// REQ-105 取引先 filter 用 fixture。
    /// supplier A 2件 / supplier B 1件 / 未設定 2件を、部門 1 / 2 に分散して投入する。
    fn seed_products_for_supplier_search(conn: &DbConnection) -> (i64, i64) {
        let supplier_a = find_or_create_supplier(conn, "取引先A").unwrap();
        let supplier_b = find_or_create_supplier(conn, "取引先B").unwrap();
        let fixtures = [
            ("SUP-A-D1", 1, Some(supplier_a.id)),
            ("SUP-A-D2", 2, Some(supplier_a.id)),
            ("SUP-B-D1", 1, Some(supplier_b.id)),
            ("SUP-NULL-D1", 1, None),
            ("SUP-NULL-D2", 2, None),
        ];
        for (code, department_id, supplier_id) in fixtures {
            let mut product = create_test_product(code, code, department_id);
            product.supplier_id = supplier_id;
            insert_product(conn, &product).unwrap();
        }
        (supplier_a.id, supplier_b.id)
    }

    fn result_codes(result: &PaginatedResult<ProductWithRelations>) -> Vec<&str> {
        result
            .items
            .iter()
            .map(|item| item.product.product_code.as_str())
            .collect()
    }

    #[test]
    fn test_search_products_req105_supplier_filter_matches_only_selected_supplier() {
        // REQ-105 / SPEC-PRVB-D1: Some(A) + false は A の商品だけを返す。
        let (_dir, conn) = setup_test_db();
        let (supplier_a, _) = seed_products_for_supplier_search(&conn);
        let mut query = default_search_query();
        query.supplier_id = Some(supplier_a);
        query.include_unassigned = false;

        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result_codes(&result), vec!["SUP-A-D1", "SUP-A-D2"]);
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_search_products_req105_supplier_filter_includes_unassigned_when_flag_set() {
        // REQ-105 / SPEC-PRVB-D1: Some(A) + true は A と未設定を返す。
        let (_dir, conn) = setup_test_db();
        let (supplier_a, _) = seed_products_for_supplier_search(&conn);
        let mut query = default_search_query();
        query.supplier_id = Some(supplier_a);
        query.include_unassigned = true;

        let result = search_products(&conn, &query).unwrap();
        assert_eq!(
            result_codes(&result),
            vec!["SUP-A-D1", "SUP-A-D2", "SUP-NULL-D1", "SUP-NULL-D2"]
        );
        assert_eq!(result.total_count, 4);
    }

    #[test]
    fn test_search_products_req105_no_supplier_filter_when_unspecified() {
        // REQ-105 / SPEC-PRVB-D1: supplier 未指定時は flag を無視して全件を返す。
        let (_dir, conn) = setup_test_db();
        seed_products_for_supplier_search(&conn);
        let mut query = default_search_query();
        query.supplier_id = None;
        query.include_unassigned = true;

        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.total_count, 5);
        assert_eq!(result.items.len(), 5);
    }

    #[test]
    fn test_search_products_req105_supplier_filter_combines_with_other_conditions() {
        // REQ-105 / SPEC-PRVB-D1: supplier OR 条件を括弧で閉じ、部門条件と AND 結合する。
        let (_dir, conn) = setup_test_db();
        let (supplier_a, _) = seed_products_for_supplier_search(&conn);
        let mut query = default_search_query();
        query.supplier_id = Some(supplier_a);
        query.include_unassigned = true;
        query.department_id = Some(1);
        query.keyword = Some("D1".to_string());

        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result_codes(&result), vec!["SUP-A-D1", "SUP-NULL-D1"]);
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_search_products_req103_all() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — 条件なしで全件取得
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let query = default_search_query();
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.total_count, 4);
        assert_eq!(result.items.len(), 4);
        assert_eq!(result.page, 1);
    }

    #[test]
    fn test_search_products_req103_keyword_name() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — 商品名キーワード部分一致
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let mut query = default_search_query();
        query.keyword = Some("ヘアゴム".to_string());
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.total_count, 2, "ヘアゴムA,Bがヒット");
    }

    #[test]
    fn test_search_products_req103_keyword_product_code() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — product_code/jan_codeキーワード部分一致
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let mut query = default_search_query();
        query.keyword = Some("4976383".to_string());
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.total_count, 1, "JANコード部分一致で1件");
    }

    #[test]
    fn test_search_products_req103_department_filter() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — department_idフィルタ
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let mut query = default_search_query();
        query.department_id = Some(2);
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.total_count, 2, "ヘア雑貨部門は2件");
    }

    #[test]
    fn test_search_products_req103_discontinued_filter() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — is_discontinuedフィルタ（現行品/廃番）
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);
        let mut query = default_search_query();
        query.is_discontinued = Some(false);
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.total_count, 3, "現行品は3件");

        // 廃番のみ
        query.is_discontinued = Some(true);
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.total_count, 1, "廃番は1件");
    }

    #[test]
    fn test_search_products_req103_all_filter_combinations() {
        // REQ-103 / P7-2 C1: keyword × department × discontinued の全8組合せ
        // 期待値は共有 seed とこの test 専用 seed の投入行から独立転記する。
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);
        let mut discontinued_product = create_test_product("HM-0001", "ハマナカ 廃番毛糸", 3);
        discontinued_product.selling_price = 50;
        discontinued_product.stock_quantity = 0;
        discontinued_product.is_discontinued = true;
        insert_product(&conn, &discontinued_product).unwrap();

        let cases = [
            (
                false,
                false,
                false,
                vec!["4976383262108", "HM-0001", "HZ-0001", "HZ-0002", "KM-0001"],
            ),
            (true, false, false, vec!["4976383262108", "HM-0001"]),
            (false, true, false, vec!["4976383262108", "HM-0001"]),
            (false, false, true, vec!["HM-0001", "HZ-0002"]),
            (true, true, false, vec!["4976383262108", "HM-0001"]),
            (true, false, true, vec!["HM-0001"]),
            (false, true, true, vec!["HM-0001"]),
            (true, true, true, vec!["HM-0001"]),
        ];

        for (keyword, department, discontinued, expected_codes) in cases {
            let mut query = default_search_query();
            query.keyword = keyword.then(|| "ハマナカ".to_string());
            query.department_id = department.then_some(3);
            query.is_discontinued = discontinued.then_some(true);

            let result = search_products(&conn, &query).unwrap();
            let actual_codes: Vec<&str> = result
                .items
                .iter()
                .map(|item| item.product.product_code.as_str())
                .collect();

            assert_eq!(
                actual_codes, expected_codes,
                "filter combination keyword={keyword}, department={department}, discontinued={discontinued}"
            );
            assert_eq!(result.total_count as usize, expected_codes.len());
        }

        // 空集合 negative oracle は非空組合せとは別入力で維持する。
        let mut empty_query = default_search_query();
        empty_query.keyword = Some("存在しない商品".to_string());
        empty_query.department_id = Some(3);
        empty_query.is_discontinued = Some(true);
        let empty_result = search_products(&conn, &empty_query).unwrap();
        assert!(empty_result.items.is_empty());
        assert_eq!(empty_result.total_count, 0);
    }

    #[test]
    fn test_search_products_req103_combined_filters_with_pagination() {
        // REQ-103 / P7-2 C1: filter と pagination 併用時の結果集合
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let mut query = default_search_query();
        query.keyword = Some("ヘアゴム".to_string());
        query.department_id = Some(2);
        query.page = 2;
        query.per_page = 1;

        let result = search_products(&conn, &query).unwrap();
        let actual_codes: Vec<&str> = result
            .items
            .iter()
            .map(|item| item.product.product_code.as_str())
            .collect();

        assert_eq!(actual_codes, vec!["HZ-0002"]);
        assert_eq!(result.total_count, 2);
        assert_eq!(result.page, 2);
        assert_eq!(result.per_page, 1);
    }

    #[test]
    fn test_search_products_req103_sort_by_selling_price_desc() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — ソート（売価降順）
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let mut query = default_search_query();
        query.sort_key = SortKey::SellingPrice;
        query.sort_order = SortOrder::Desc;
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.items[0].product.selling_price, 594);
        assert_eq!(result.items[3].product.selling_price, 100);
    }

    #[test]
    fn test_search_products_req103_pagination() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — ページング（page/per_page/total_count）
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let mut query = default_search_query();
        query.per_page = 2;
        query.page = 1;
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.items.len(), 2, "1ページ目は2件");
        assert_eq!(result.total_count, 4, "total_countは全件数");

        query.page = 2;
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.items.len(), 2, "2ページ目も2件");

        query.page = 3;
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.items.len(), 0, "3ページ目は0件");
    }

    #[test]
    fn test_search_products_req103_per_page_clamps_to_max() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3 / D-031: search_products — per_page > 200 は200にクランプ
        let (_dir, conn) = setup_test_db();
        seed_products_for_search(&conn);

        let mut query = default_search_query();
        query.per_page = 500;
        let result = search_products(&conn, &query).unwrap();

        assert_eq!(result.per_page, 200, "per_page は200にクランプされるべき");
        assert_eq!(result.items.len(), 4, "テストデータ4件は1ページに収まる");
    }

    #[test]
    fn test_search_products_req103_page_zero_returns_error() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — page=0でQueryFailedエラー
        let (_dir, conn) = setup_test_db();
        let mut query = default_search_query();
        query.page = 0;
        let result = search_products(&conn, &query);
        assert!(
            matches!(result, Err(DbError::QueryFailed(_))),
            "page=0 は QueryFailed エラーが返るべき: {:?}",
            result
        );
    }

    #[test]
    fn test_search_products_req103_per_page_zero_returns_error() {
        // REQ-103: 商品検索（コード検索/JAN検索/一覧検索/ページング）
        // FUNC-2.3: search_products — per_page=0でQueryFailedエラー
        let (_dir, conn) = setup_test_db();
        let mut query = default_search_query();
        query.per_page = 0;
        let result = search_products(&conn, &query);
        assert!(
            matches!(result, Err(DbError::QueryFailed(_))),
            "per_page=0 は QueryFailed エラーが返るべき: {:?}",
            result
        );
    }

    // ===== FUNC-2.3/2.6: update_product + insert_price_history テスト =====

    #[test]
    fn test_update_product_req102_partial_update() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — 一部フィールドのみ更新、他は維持
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("UPD-001", "更新前の名前", 1);
        insert_product(&conn, &product).unwrap();

        let updates = ProductUpdates {
            name: Some("更新後の名前".to_string()),
            selling_price: Some(999),
            ..Default::default()
        };
        let updated = update_product(&conn, "UPD-001", &updates).unwrap();
        assert!(updated, "更新成功でtrueが返るべき");

        let found = find_by_product_code(&conn, "UPD-001").unwrap().unwrap();
        assert_eq!(found.product.name, "更新後の名前");
        assert_eq!(found.product.selling_price, 999);
        // 変更していないフィールドは元のまま
        assert_eq!(found.product.cost_price_centi, 300);
    }

    #[test]
    fn test_update_product_req102_sets_stock_unit() {
        // REQ-102: 商品更新。20 §2.3: ProductUpdates.stock_unit を SET し、None なら変えない
        let (_dir, conn) = setup_test_db();
        insert_product(&conn, &create_test_product("UPD-UNIT", "単位", 1)).unwrap();

        let none = ProductUpdates {
            name: Some("単位そのまま".to_string()),
            ..Default::default()
        };
        assert!(update_product(&conn, "UPD-UNIT", &none).unwrap());
        let found = find_by_product_code(&conn, "UPD-UNIT").unwrap().unwrap();
        assert_eq!(found.product.stock_unit, ProductStockUnit::Pcs);

        let updates = ProductUpdates {
            stock_unit: Some(ProductStockUnit::Ball),
            ..Default::default()
        };
        assert!(update_product(&conn, "UPD-UNIT", &updates).unwrap());
        let found = find_by_product_code(&conn, "UPD-UNIT").unwrap().unwrap();
        assert_eq!(found.product.stock_unit, ProductStockUnit::Ball);
    }

    #[test]
    fn test_update_product_req102_nonexistent_returns_false() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — 該当なしでfalse
        let (_dir, conn) = setup_test_db();
        let updates = ProductUpdates {
            name: Some("存在しない".to_string()),
            ..Default::default()
        };
        let updated = update_product(&conn, "NONEXISTENT", &updates).unwrap();
        assert!(!updated, "該当なしでfalseが返るべき");
    }

    #[test]
    fn test_update_product_req102_no_fields_still_updates_timestamp() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — 全フィールドNoneでもupdated_atは更新される
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("UPD-TS", "タイムスタンプテスト", 1);
        insert_product(&conn, &product).unwrap();

        let before = find_by_product_code(&conn, "UPD-TS")
            .unwrap()
            .unwrap()
            .product
            .updated_at;

        // 少し待つ必要はない — chronoの精度で秒が同じ可能性がある
        // updated_atが上書きされることだけを確認
        let updates = ProductUpdates::default();
        let updated = update_product(&conn, "UPD-TS", &updates).unwrap();
        assert!(updated, "全Noneでもtrue（updated_atは更新される）");

        let after = find_by_product_code(&conn, "UPD-TS")
            .unwrap()
            .unwrap()
            .product
            .updated_at;
        // updated_at が存在すること（値自体のチェックは秒精度の問題があるため省略）
        assert!(!after.is_empty());
        // before と after が同じ日時フォーマットであること
        assert_eq!(before.len(), after.len());
    }

    #[test]
    fn test_update_product_req102_set_supplier_to_null() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — Option<Option<T>>でsupplier_idをNULLに設定
        let (_dir, conn) = setup_test_db();
        let supplier = find_or_create_supplier(&conn, "テスト取引先").unwrap();
        let mut product = create_test_product("UPD-NULL", "NULL化テスト", 1);
        product.supplier_id = Some(supplier.id);
        insert_product(&conn, &product).unwrap();

        // supplier_id が設定されていることを確認
        let before = find_by_product_code(&conn, "UPD-NULL").unwrap().unwrap();
        assert!(before.product.supplier_id.is_some());

        // Some(None) で NULL に更新
        let updates = ProductUpdates {
            supplier_id: Some(None),
            ..Default::default()
        };
        update_product(&conn, "UPD-NULL", &updates).unwrap();

        let after = find_by_product_code(&conn, "UPD-NULL").unwrap().unwrap();
        assert!(
            after.product.supplier_id.is_none(),
            "supplier_id が NULL になるべき"
        );
        assert!(after.supplier_name.is_none());
    }

    #[test]
    fn test_update_product_req102_set_maker_code_to_null() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — Option<Option<T>>でmaker_codeをNULLに設定
        let (_dir, conn) = setup_test_db();
        let mut product = create_test_product("UPD-MK", "メーカーコードテスト", 1);
        product.maker_code = Some("H180-005-42".to_string());
        insert_product(&conn, &product).unwrap();

        let updates = ProductUpdates {
            maker_code: Some(None),
            ..Default::default()
        };
        update_product(&conn, "UPD-MK", &updates).unwrap();

        let after = find_by_product_code(&conn, "UPD-MK").unwrap().unwrap();
        assert!(
            after.product.maker_code.is_none(),
            "maker_code が NULL になるべき"
        );
    }

    #[test]
    fn test_update_product_req102_set_plu_exported_at_value() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — plu_exported_at に値を設定
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("UPD-PLU1", "PLU書出しテスト", 1);
        insert_product(&conn, &product).unwrap();

        let updates = ProductUpdates {
            plu_exported_at: Some(Some("2026-04-08T15:00:00".to_string())),
            ..Default::default()
        };
        update_product(&conn, "UPD-PLU1", &updates).unwrap();

        let after = find_by_product_code(&conn, "UPD-PLU1").unwrap().unwrap();
        assert_eq!(
            after.product.plu_exported_at.as_deref(),
            Some("2026-04-08T15:00:00"),
            "plu_exported_at が設定されるべき"
        );
    }

    #[test]
    fn test_update_product_req102_set_plu_exported_at_to_null() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — plu_exported_at を NULL に設定
        let (_dir, conn) = setup_test_db();
        let mut product = create_test_product("UPD-PLU2", "PLU書出しNULLテスト", 1);
        product.plu_exported_at = Some("2026-04-01T12:00:00".to_string());
        insert_product(&conn, &product).unwrap();

        let updates = ProductUpdates {
            plu_exported_at: Some(None),
            ..Default::default()
        };
        update_product(&conn, "UPD-PLU2", &updates).unwrap();

        let after = find_by_product_code(&conn, "UPD-PLU2").unwrap().unwrap();
        assert!(
            after.product.plu_exported_at.is_none(),
            "plu_exported_at が NULL になるべき"
        );
    }

    #[test]
    fn test_update_product_req102_plu_exported_at_unspecified_no_change() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.3: update_product — plu_exported_at 未指定時は変更なし
        let (_dir, conn) = setup_test_db();
        let mut product = create_test_product("UPD-PLU3", "PLU書出し未指定テスト", 1);
        product.plu_exported_at = Some("2026-04-01T12:00:00".to_string());
        insert_product(&conn, &product).unwrap();

        let updates = ProductUpdates {
            name: Some("名前だけ変更".to_string()),
            // plu_exported_at は None（未指定）→ 変更されない
            ..Default::default()
        };
        update_product(&conn, "UPD-PLU3", &updates).unwrap();

        let after = find_by_product_code(&conn, "UPD-PLU3").unwrap().unwrap();
        assert_eq!(
            after.product.plu_exported_at.as_deref(),
            Some("2026-04-01T12:00:00"),
            "plu_exported_at は変更されないべき"
        );
    }

    #[test]
    fn test_insert_price_history_req102_normal() {
        // REQ-102: 商品更新（売価変更/PLU dirty/価格履歴）
        // FUNC-2.6: insert_price_history — 正常INSERT、値の保存確認
        let (_dir, conn) = setup_test_db();
        let product = create_test_product("PH-001", "価格履歴テスト", 1);
        insert_product(&conn, &product).unwrap();

        let history = NewPriceHistory {
            product_code: "PH-001".to_string(),
            old_selling: 500,
            new_selling: 600,
            old_cost_centi: 300,
            new_cost_centi: 350,
        };
        insert_price_history(&conn, &history).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM price_history WHERE product_code = 'PH-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // 値が正しく保存されていることを確認
        let (old_s, new_s): (i64, i64) = conn
            .query_row(
                "SELECT old_selling, new_selling FROM price_history WHERE product_code = 'PH-001'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(old_s, 500);
        assert_eq!(new_s, 600);
    }

    // -----------------------------------------------------------------------
    // find_supplier_by_id テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_supplier_by_id_req101_exists() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // BIZ-02: supplier_id 存在確認（正常系）
        let (_dir, conn) = setup_test_db();
        let supplier = find_or_create_supplier(&conn, "テスト取引先A").unwrap();

        let found = find_supplier_by_id(&conn, supplier.id).unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, supplier.id);
        assert_eq!(found.name, "テスト取引先A");
    }

    #[test]
    fn test_find_supplier_by_id_req101_not_found() {
        // REQ-101: 商品登録（部門/取引先/商品INSERT/独自コード発番）
        // BIZ-02: supplier_id 存在確認（存在しないID）
        let (_dir, conn) = setup_test_db();

        let found = find_supplier_by_id(&conn, 99999).unwrap();
        assert!(found.is_none());
    }

    // -----------------------------------------------------------------------
    // PLU専用クエリ テスト（BIZ-04用）
    // -----------------------------------------------------------------------

    /// テスト用に商品を直接INSERTするヘルパー
    fn insert_test_product_for_plu(
        conn: &DbConnection,
        product_code: &str,
        department_id: i64,
        is_discontinued: bool,
        plu_dirty: bool,
    ) {
        conn.execute(
            "INSERT INTO products (product_code, name, department_id, selling_price, cost_price_centi,
                    tax_rate, stock_quantity, stock_unit, is_discontinued, plu_dirty,
                    plu_target, pos_stock_sync, created_at, updated_at)
             VALUES (?1, ?2, ?3, 500, 300, '10', 10, 'pcs', ?4, ?5, 1, 1,
                    '2026-04-11T00:00:00', '2026-04-11T00:00:00')",
            rusqlite::params![
                product_code,
                format!("テスト商品{}", product_code),
                department_id,
                is_discontinued,
                plu_dirty,
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_find_active_products_for_plu_req402_returns_with_department_name() {
        // REQ-402: PLU書出し（PLU対象商品/dirtyフラグ）
        // BIZ-04: active商品がdepartment_name付きで返ること
        let (_dir, conn) = setup_test_db();
        // 部門ID=1 は "その他小物"（初期データ）
        insert_test_product_for_plu(&conn, "PLU-0001", 1, false, true);
        insert_test_product_for_plu(&conn, "PLU-0002", 1, false, false);
        insert_test_product_for_plu(&conn, "PLU-DISC", 1, true, true); // 廃番

        let result = find_active_products_for_plu(&conn).unwrap();
        assert_eq!(result.len(), 2, "廃番を除いた2件が返る");
        assert_eq!(result[0].product.product_code, "PLU-0001");
        assert_eq!(result[0].department_name, "その他小物");
        assert_eq!(result[1].product.product_code, "PLU-0002");
    }

    #[test]
    fn test_find_plu_dirty_products_for_plu_req402_returns_dirty_only() {
        // REQ-402: PLU書出し（PLU対象商品/dirtyフラグ）
        // BIZ-04: plu_dirty=1の商品のみ部門名付きで返ること
        let (_dir, conn) = setup_test_db();
        insert_test_product_for_plu(&conn, "PLU-D001", 1, false, true); // dirty
        insert_test_product_for_plu(&conn, "PLU-D002", 1, false, false); // clean

        let result = find_plu_dirty_products_for_plu(&conn).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].product.product_code, "PLU-D001");
        assert_eq!(result[0].department_name, "その他小物");
    }

    #[test]
    fn test_find_plu_dirty_products_req402_returns_dirty_only() {
        // REQ-402: PLU書出し（PLU対象商品/dirtyフラグ）
        // BIZ-04: plu_dirty=1の商品のみ返ること（department_nameなし版）
        let (_dir, conn) = setup_test_db();
        insert_test_product_for_plu(&conn, "PLU-P001", 1, false, true); // dirty
        insert_test_product_for_plu(&conn, "PLU-P002", 1, false, false); // clean

        let result = find_plu_dirty_products(&conn).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].product_code, "PLU-P001");
        assert!(result[0].plu_dirty);
    }

    #[test]
    fn test_find_plu_queries_req402_filter_plu_target() {
        // REQ-402 / D-028: PLU対象外の商品は未反映/全件PLUクエリに含めない
        let (_dir, conn) = setup_test_db();
        insert_test_product_for_plu(&conn, "PLU-TARGET", 1, false, true);
        insert_test_product_for_plu(&conn, "PLU-NOTARGET", 1, false, true);
        update_product(
            &conn,
            "PLU-NOTARGET",
            &ProductUpdates {
                plu_target: Some(false),
                ..Default::default()
            },
        )
        .unwrap();

        let active = find_active_products_for_plu(&conn).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].product.product_code, "PLU-TARGET");

        let dirty_plu = find_plu_dirty_products_for_plu(&conn).unwrap();
        assert_eq!(dirty_plu.len(), 1);
        assert_eq!(dirty_plu[0].product.product_code, "PLU-TARGET");

        let dirty = find_plu_dirty_products(&conn).unwrap();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].product_code, "PLU-TARGET");
    }

    #[test]
    fn test_find_plu_queries_req402_empty_result() {
        // REQ-402: PLU書出し（PLU対象商品/dirtyフラグ）
        // BIZ-04: 対象0件 → Ok(空Vec)
        let (_dir, conn) = setup_test_db();
        // 商品を1つも登録しない

        let active = find_active_products_for_plu(&conn).unwrap();
        assert!(active.is_empty());

        let dirty_plu = find_plu_dirty_products_for_plu(&conn).unwrap();
        assert!(dirty_plu.is_empty());

        let dirty = find_plu_dirty_products(&conn).unwrap();
        assert!(dirty.is_empty());

        let active_plain = find_active_products(&conn).unwrap();
        assert!(active_plain.is_empty());
    }

    // ===== FUNC-2.11: find_all_stock_quantities テスト =====

    #[test]
    fn test_find_all_stock_quantities_req301() {
        // REQ-301: 在庫照会（在庫数一括取得）
        // FUNC-2.11: find_all_stock_quantities — 全商品の在庫数取得
        let (_dir, conn) = setup_test_db();
        let p1 = create_test_product("ASQ-001", "商品A", 1);
        let mut p2 = create_test_product("ASQ-002", "商品B", 1);
        p2.stock_quantity = 15;
        insert_product(&conn, &p1).unwrap();
        insert_product(&conn, &p2).unwrap();

        let result = find_all_stock_quantities(&conn).unwrap();
        assert_eq!(result.len(), 2);
        // ORDER BY product_code ASC。写しの 4 項目の行（Matrix T6。数と単位は前と同じ値）
        let row = |code: &str, name: &str, stock: i64| StockCopyRow {
            product_code: code.to_string(),
            name: name.to_string(),
            stock_unit: ProductStockUnit::Pcs,
            stock_quantity: stock,
            stock_recheck: false,
            stock_range_low: None,
            stock_range_high: None,
        };
        assert_eq!(result[0], row("ASQ-001", "商品A", 0));
        assert_eq!(result[1], row("ASQ-002", "商品B", 15));
        conn.execute(
            "UPDATE products SET stock_recheck = 1, stock_range_low = 14, stock_range_high = 16
             WHERE product_code = 'ASQ-002'",
            [],
        )
        .unwrap();
        let result = find_all_stock_quantities(&conn).unwrap();
        assert_eq!(
            result[1],
            StockCopyRow {
                stock_recheck: true,
                stock_range_low: Some(14),
                stock_range_high: Some(16),
                ..row("ASQ-002", "商品B", 15)
            }
        );
    }

    #[test]
    fn test_search_products_req105_keyword_matches_maker_code() {
        let (_dir, conn) = setup_test_db();
        let mut product = create_test_product("MK-001", "合成商品", 1);
        product.maker_code = Some("MAKER-ALPHA-42".to_string());
        insert_product(&conn, &product).unwrap();
        let mut query = default_search_query();
        query.keyword = Some("ALPHA-42".to_string());
        let result = search_products(&conn, &query).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].product.product_code, "MK-001");
    }

    #[test]
    fn test_search_products_req103_keyword_matches_existing_three_columns() {
        let (_dir, conn) = setup_test_db();
        let mut by_name = create_test_product("LEG-001", "NEEDLE-NAME", 1);
        by_name.jan_code = Some("4900000000001".to_string());
        let mut by_code = create_test_product("CODE-NEEDLE", "合成B", 1);
        by_code.jan_code = Some("4900000000002".to_string());
        let mut by_jan = create_test_product("LEG-003", "合成C", 1);
        by_jan.jan_code = Some("4909876543210".to_string());
        insert_product(&conn, &by_name).unwrap();
        insert_product(&conn, &by_code).unwrap();
        insert_product(&conn, &by_jan).unwrap();
        for (keyword, expected) in [
            ("NEEDLE-NAME", "LEG-001"),
            ("CODE-NEEDLE", "CODE-NEEDLE"),
            ("987654321", "LEG-003"),
        ] {
            let mut query = default_search_query();
            query.keyword = Some(keyword.to_string());
            let result = search_products(&conn, &query).unwrap();
            assert_eq!(result.items.len(), 1, "keyword={keyword}");
            assert_eq!(result.items[0].product.product_code, expected);
        }
    }

    #[test]
    fn test_find_products_for_bulk_plu_target_req105_keyword_matches_maker_code() {
        let (_dir, conn) = setup_test_db();
        let mut product = create_test_product("BULK-MK-001", "合成商品", 1);
        product.maker_code = Some("BULK-MAKER-Z9".to_string());
        insert_product(&conn, &product).unwrap();
        let result = find_products_for_bulk_plu_target(
            &conn,
            &ProductBulkFilter {
                keyword: Some("MAKER-Z9".to_string()),
                department_id: None,
                is_discontinued: None,
                plu: None,
            },
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].product_code, "BULK-MK-001");
    }

    fn insert_history_fixture(conn: &DbConnection, product_code: &str, count: u32) {
        insert_product(conn, &create_test_product(product_code, "履歴対象商品", 1)).unwrap();
        for index in 0..count {
            conn.execute(
                "INSERT INTO price_history
                 (product_code, old_selling, new_selling, old_cost_centi, new_cost_centi, changed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    product_code,
                    i64::from(index),
                    i64::from(index + 1),
                    i64::from(index + 10),
                    i64::from(index + 11),
                    format!("2026-08-22T00:{:02}:{:02}", (index / 60) % 60, index % 60),
                ],
            )
            .unwrap();
        }
    }

    #[test]
    fn test_list_price_history_req102_desc_order() {
        let (_dir, conn) = setup_test_db();
        insert_history_fixture(&conn, "HIST-ORDER", 3);
        let result = list_price_history(&conn, "HIST-ORDER", 10).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].new_selling_price, 3);
        assert_eq!(result[1].new_selling_price, 2);
        assert_eq!(result[2].new_selling_price, 1);
    }

    #[test]
    fn test_list_price_history_req102_id_desc_tie_break_on_same_changed_at() {
        let (_dir, conn) = setup_test_db();
        insert_product(
            &conn,
            &create_test_product("HIST-TIE-BREAK", "履歴同時刻商品", 1),
        )
        .unwrap();
        for (id, changed_at) in [
            (101_i64, "2026-08-22T12:00:00"),
            (102_i64, "2026-08-22T12:00:00"),
            (103_i64, "2026-08-21T12:00:00"),
        ] {
            conn.execute(
                "INSERT INTO price_history
                 (id, product_code, old_selling, new_selling, old_cost_centi, new_cost_centi, changed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    id,
                    "HIST-TIE-BREAK",
                    id + 10,
                    id + 20,
                    id + 30,
                    id + 40,
                    changed_at,
                ],
            )
            .unwrap();
        }

        let result = list_price_history(&conn, "HIST-TIE-BREAK", 10).unwrap();
        let actual_ids: Vec<i64> = result.iter().map(|entry| entry.id).collect();

        assert_eq!(actual_ids, [102, 101, 103]);
        assert_eq!([result[0].id, result[1].id], [102, 101]);
    }

    #[test]
    fn test_list_price_history_req102_respects_limit() {
        let (_dir, conn) = setup_test_db();
        insert_history_fixture(&conn, "HIST-LIMIT", 6);
        let result = list_price_history(&conn, "HIST-LIMIT", 4).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].new_selling_price, 6);
    }

    #[test]
    fn test_list_price_history_req102_clamps_limit_over_max() {
        let (_dir, conn) = setup_test_db();
        insert_history_fixture(&conn, "HIST-CLAMP", 105);
        let result = list_price_history(&conn, "HIST-CLAMP", 101).unwrap();
        assert_eq!(result.len(), 100);
        assert_eq!(result[0].new_selling_price, 105);
    }

    #[test]
    fn test_list_price_history_req102_unknown_product_returns_empty() {
        let (_dir, conn) = setup_test_db();
        let result = list_price_history(&conn, "UNKNOWN-HISTORY", 10).unwrap();
        assert!(result.is_empty());
    }

    // ===== D-118 b2: 商品の結果の写しの 3 列（Matrix P1・P2） =====

    #[test]
    fn test_product_reads_req301_req303_carry_copy_recheck_and_range() {
        // REQ-301 / REQ-303 / SPEC-STK-CNT-D9（20・40）: 商品の読取り 4 つが写しの 3 列を DB のとおりに返す。
        // get_stock_detail は列の位置のずれを最終入庫日・最終販売日で見る（Matrix P1）
        let (_dir, conn) = setup_test_db();
        let mut recheck = create_test_product("CPY-REC", "要再確認の商品", 1);
        recheck.stock_quantity = 9;
        let mut plain = create_test_product("CPY-PLN", "要再確認でない商品", 1);
        plain.stock_quantity = 3;
        insert_product(&conn, &recheck).unwrap();
        insert_product(&conn, &plain).unwrap();
        conn.execute(
            "UPDATE products SET stock_recheck = 1, stock_range_low = 7, stock_range_high = 12
             WHERE product_code = 'CPY-REC'",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO receiving_records (idempotency_key, request_fingerprint, supplier_id, receiving_date, note, created_at)
             VALUES ('cpy-key', 'cpy-fp', NULL, '2026-03-20', NULL, '2026-03-20T10:00:00')",
            [],
        )
        .unwrap();
        let record_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price_centi)
             VALUES (?1, 'CPY-REC', 5, 300)",
            rusqlite::params![record_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sale_records (product_code, sale_date, quantity, amount, source, is_voided, created_at)
             VALUES ('CPY-REC', '2026-03-22', 2, 1000, 'auto', 0, '2026-03-22T19:00:00')",
            [],
        )
        .unwrap();
        let copy = |p: &ProductWithRelations| {
            (
                p.product.product_code.clone(),
                p.stock_recheck,
                p.stock_range_low,
                p.stock_range_high,
            )
        };
        let rec = ("CPY-REC".to_string(), true, Some(7), Some(12));
        let pln = ("CPY-PLN".to_string(), false, None, None);

        let found = find_by_product_code(&conn, "CPY-REC").unwrap().unwrap();
        assert_eq!(copy(&found), rec);
        let found = find_by_product_code(&conn, "CPY-PLN").unwrap().unwrap();
        assert_eq!(copy(&found), pln);

        let searched = search_products(&conn, &default_search_query()).unwrap();
        let mut searched: Vec<_> = searched.items.iter().map(copy).collect();
        searched.sort();
        assert_eq!(searched, vec![pln.clone(), rec.clone()]);

        let detail = get_stock_detail(&conn, "CPY-REC").unwrap();
        assert_eq!(copy(&detail.product), rec);
        assert_eq!(detail.last_receiving_date.as_deref(), Some("2026-03-20"));
        assert_eq!(detail.last_sale_date.as_deref(), Some("2026-03-22"));

        let low = list_low_stock_products(
            &conn,
            &[ProductStockUnit::Pcs],
            100,
            &[ProductStockUnit::M],
            0,
            false,
        )
        .unwrap();
        let low: Vec<_> = low.iter().map(copy).collect();
        assert_eq!(low, vec![pln, rec]);
    }

    #[test]
    fn test_product_with_relations_req303_serializes_copy_fields_flat() {
        // REQ-303 / SPEC-STK-CNT-D9（40）: wire の key は flatten された商品の field と同じ階層（Matrix P2）
        let (_dir, conn) = setup_test_db();
        insert_product(&conn, &create_test_product("CPY-JSON", "写しの商品", 1)).unwrap();
        conn.execute(
            "UPDATE products SET stock_recheck = 1, stock_range_low = -1, stock_range_high = 0
             WHERE product_code = 'CPY-JSON'",
            [],
        )
        .unwrap();
        let found = find_by_product_code(&conn, "CPY-JSON").unwrap().unwrap();
        let json = serde_json::to_value(&found).unwrap();
        assert_eq!(json["product_code"], "CPY-JSON");
        assert_eq!(json["stock_recheck"], true);
        assert_eq!(json["stock_range_low"], -1);
        assert_eq!(json["stock_range_high"], 0);
    }
}
