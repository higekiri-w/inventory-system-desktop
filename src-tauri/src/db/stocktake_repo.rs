//! 棚卸しヘッダ・棚卸し明細のCRUD操作
//!
//! 20-io-product-repo.md §2.9 に基づく実装。
//! IO-01: SQLiteデータアクセス層（stocktake_repository）

use super::product_repo::{parse_stock_unit, ProductStockUnit};
use super::{DbConnection, DbError};
use crate::constants::PAGINATION_MAX_PER_PAGE;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 棚卸しヘッダの行マッピング
///
/// db-design/tracking-system-tables.md stocktakes
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct Stocktake {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub total_cost: Option<i64>,
}

/// 棚卸し明細INSERT用
///
/// 20-io-product-repo.md §2.9
#[derive(Debug)]
pub struct NewStocktakeItem {
    pub stocktake_id: i64,
    pub product_code: String,
    pub system_stock: i64,
    pub actual_count: Option<i64>,
}

/// 棚卸し対象商品（start_stocktake用）
///
/// 20-io-product-repo.md §2.11 — 全商品を返す（フィルタなし）
#[derive(Debug, Clone)]
pub struct ProductForStocktake {
    pub product_code: String,
    pub stock_quantity: i64,
    pub cost_price_centi: i64,
    pub is_discontinued: bool,
}

/// 棚卸し明細の行マッピング
///
/// 20-io-product-repo.md §2.11
#[derive(Debug, Clone)]
pub struct StocktakeItem {
    pub id: i64,
    pub stocktake_id: i64,
    pub product_code: String,
    pub system_stock: i64,
    pub actual_count: Option<i64>,
    pub counted_at: Option<String>,
}

/// 棚卸し明細の行（商品名・部門名・単位・写しの範囲付き。効く計数・差異・理由は BIZ が求める。35）
///
/// 20-io-product-repo.md §2.11
#[derive(Debug, Clone)]
pub struct StocktakeItemRow {
    pub id: i64,
    pub stocktake_id: i64,
    pub product_code: String,
    pub name: String,
    pub department_name: String,
    pub system_stock: i64,
    pub actual_count: Option<i64>,
    pub counted_at: Option<String>,
    pub current_stock: i64,
    pub is_discontinued: bool,
    pub stock_unit: ProductStockUnit,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
}

/// 完了の検査と評価に使う明細（35「完了」）
#[derive(Debug, Clone)]
pub struct StocktakeItemForCompletion {
    pub id: i64,
    pub product_code: String,
    /// 明細を参照する計数がある（「計数済み」）
    pub counted: bool,
    pub is_discontinued: bool,
    pub stock_quantity: i64,
    pub stock_recheck: bool,
    pub cost_price_centi: i64,
    pub stock_unit: ProductStockUnit,
}

/// 商品の写し（要再確認の対象と計数の欄の文脈。40 の CountRecoveryTarget）
#[derive(Debug, Clone)]
pub struct StockCopyRow {
    pub product_code: String,
    pub name: String,
    pub stock_unit: ProductStockUnit,
    pub stock_quantity: i64,
    pub stock_recheck: bool,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
}

/// 完了の時点のカーソル（tracking。完了していなければ全て None）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StocktakeCutoff {
    /// 完了の時点の `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL。ADR D6）
    pub import_id: Option<i64>,
    pub count_id: Option<i64>,
    pub movement_id: Option<i64>,
}

/// 未入力明細
#[derive(Debug, Clone)]
pub struct UncountedItem {
    pub stocktake_item_id: i64,
    pub product_code: String,
}

/// 棚卸し進捗
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct StocktakeProgress {
    pub total_items: i64,
    pub counted_items: i64,
    pub uncounted_items: i64,
}

/// 前回完了棚卸しサマリ（UI-10-D5）
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct LastStocktakeSummary {
    pub stocktake_id: i64,
    pub completed_at: String,
    pub total_cost: i64,
}

/// 棚卸し記録詳細の明細の行（明細と header から読む。補正の movement から読まない。65）
#[derive(Debug, Clone)]
pub struct StocktakeRecordItemRow {
    pub product_code: String,
    pub product_name: String,
    pub department_name: String,
    pub stock_unit: ProductStockUnit,
    pub valuation_quantity: Option<i64>,
    pub valuation_cost_price_centi: Option<i64>,
}

/// 棚卸し記録詳細の IO 内部型。
#[derive(Debug)]
pub struct StocktakeRecordDetailCore {
    pub header: Stocktake,
    pub item_count: i64,
    pub items: Vec<StocktakeRecordItemRow>,
    pub cutoff: StocktakeCutoff,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// status='in_progress' の棚卸しを取得する
///
/// 複数存在する不整合時は started_at DESC, id DESC で最新を返す。
///
/// 20-io-product-repo.md §2.9
pub fn find_active_stocktake(conn: &DbConnection) -> Result<Option<Stocktake>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, completed_at, status, total_cost
         FROM stocktakes
         WHERE status = 'in_progress'
         ORDER BY started_at DESC, id DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => Ok(Some(Stocktake {
            id: row.get(0)?,
            started_at: row.get(1)?,
            completed_at: row.get(2)?,
            status: row.get(3)?,
            total_cost: row.get(4)?,
        })),
        None => Ok(None),
    }
}

/// 棚卸し明細を1行追加する
///
/// 棚卸し中の新規商品登録時に BIZ-01 から呼ばれる。
///
/// 20-io-product-repo.md §2.9
pub fn insert_stocktake_item(conn: &DbConnection, item: &NewStocktakeItem) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            item.stocktake_id,
            item.product_code,
            item.system_stock,
            item.actual_count,
        ],
    )?;
    Ok(())
}

/// 棚卸しヘッダを作成する（status='in_progress'）
///
/// 20-io-product-repo.md §2.11
pub fn insert_stocktake(conn: &DbConnection, started_at: &str) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO stocktakes (started_at, status) VALUES (?1, 'in_progress')",
        rusqlite::params![started_at],
    )?;
    Ok(conn.last_insert_rowid())
}

/// IDで棚卸しヘッダを取得する
///
/// 20-io-product-repo.md §2.11
pub fn find_stocktake_by_id(conn: &DbConnection, id: i64) -> Result<Option<Stocktake>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, completed_at, status, total_cost
         FROM stocktakes
         WHERE id = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(Stocktake {
            id: row.get(0)?,
            started_at: row.get(1)?,
            completed_at: row.get(2)?,
            status: row.get(3)?,
            total_cost: row.get(4)?,
        })),
        None => Ok(None),
    }
}

/// 最後に完了した棚卸しを取得する
///
/// UI-10-D5: 前回棚卸しサマリ表示用。未完了の棚卸しは対象外。
pub fn find_last_completed_stocktake(
    conn: &DbConnection,
) -> Result<Option<LastStocktakeSummary>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, completed_at, total_cost
         FROM stocktakes
         WHERE status = 'completed'
         ORDER BY completed_at DESC, id DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => Ok(Some(LastStocktakeSummary {
            stocktake_id: row.get(0)?,
            completed_at: row.get(1)?,
            total_cost: row.get(2)?,
        })),
        None => Ok(None),
    }
}

/// 棚卸しを確定する（status='completed'に更新）
///
/// in_progress 状態のみ更新可能。二重確定を防止する。
/// affected_rows=0 の場合は DbError::NotFound を返す。
///
/// 20-io-product-repo.md §2.11
pub fn complete_stocktake(
    conn: &DbConnection,
    id: i64,
    total_cost: i64,
    completed_at: &str,
    cutoff: StocktakeCutoff,
) -> Result<(), DbError> {
    let affected = conn.execute(
        "UPDATE stocktakes SET status = 'completed', total_cost = ?1, completed_at = ?2,
             cutoff_import_id = ?3, cutoff_count_id = ?4, cutoff_movement_id = ?5
         WHERE id = ?6 AND status = 'in_progress'",
        rusqlite::params![
            total_cost,
            completed_at,
            cutoff.import_id,
            cutoff.count_id,
            cutoff.movement_id,
            id
        ],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// 棚卸し明細のカウント値を更新する
///
/// 戻り値: true=更新成功、false=対象アイテムなし
///
/// 20-io-product-repo.md §2.11
pub fn update_stocktake_item_count(
    conn: &DbConnection,
    item_id: i64,
    actual_count: i64,
    counted_at: &str,
) -> Result<bool, DbError> {
    let affected = conn.execute(
        "UPDATE stocktake_items SET actual_count = ?1, counted_at = ?2 WHERE id = ?3",
        rusqlite::params![actual_count, counted_at, item_id],
    )?;
    Ok(affected == 1)
}

/// 棚卸し明細の評価原価と評価数量を固定する（完了の TX。35）
///
/// 20-io-product-repo.md §2.11
pub fn update_stocktake_item_valuation(
    conn: &DbConnection,
    item_id: i64,
    valuation_cost_price_centi: i64,
    valuation_quantity: i64,
) -> Result<(), DbError> {
    conn.execute(
        "UPDATE stocktake_items SET valuation_cost_price_centi = ?1, valuation_quantity = ?2
         WHERE id = ?3",
        rusqlite::params![valuation_cost_price_centi, valuation_quantity, item_id],
    )?;
    Ok(())
}

/// 未入力アイテム数を取得する
///
/// 20-io-product-repo.md §2.11
pub fn count_uncounted_items(conn: &DbConnection, stocktake_id: i64) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM stocktake_items si
         WHERE si.stocktake_id = ?1 AND NOT EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id)",
        rusqlite::params![stocktake_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// 棚卸し明細を親ステータス付きで取得する（計数の保存の明細の確かめ用。35）
///
/// 20-io-product-repo.md §2.11
pub fn find_stocktake_item_with_parent_status(
    conn: &DbConnection,
    item_id: i64,
) -> Result<Option<(StocktakeItem, String)>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.stocktake_id, si.product_code, si.system_stock,
                si.actual_count, si.counted_at, st.status
         FROM stocktake_items si
         JOIN stocktakes st ON si.stocktake_id = st.id
         WHERE si.id = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![item_id])?;
    match rows.next()? {
        Some(row) => {
            let item = StocktakeItem {
                id: row.get(0)?,
                stocktake_id: row.get(1)?,
                product_code: row.get(2)?,
                system_stock: row.get(3)?,
                actual_count: row.get(4)?,
                counted_at: row.get(5)?,
            };
            let status: String = row.get(6)?;
            Ok(Some((item, status)))
        }
        None => Ok(None),
    }
}

/// 商品コードまたはJANコードで棚卸し明細を1件取得する
///
/// UI-10-D2 / 73 §73.8: 同一JANで複数候補がある場合は si.id ASC の先頭を返す。
pub fn find_stocktake_item_by_code(
    conn: &DbConnection,
    stocktake_id: i64,
    code: &str,
) -> Result<Option<StocktakeItemRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.stocktake_id, si.product_code, p.name, d.name,
                si.system_stock, si.actual_count, si.counted_at, p.stock_quantity,
                p.is_discontinued, p.stock_unit, p.stock_range_low, p.stock_range_high
         FROM stocktake_items si
         JOIN products p ON si.product_code = p.product_code
         JOIN departments d ON p.department_id = d.id
         WHERE si.stocktake_id = ?1
           AND (si.product_code = ?2 OR p.jan_code = ?2)
         ORDER BY si.id ASC
         LIMIT 1",
    )?;
    let mut rows = stmt.query(rusqlite::params![stocktake_id, code])?;
    match rows.next()? {
        Some(row) => Ok(Some(item_row(row)?)),
        None => Ok(None),
    }
}

/// 未計数（明細を参照する計数が無い）明細の一覧を取得する
///
/// 20-io-product-repo.md §2.11
pub fn list_uncounted_items(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<Vec<UncountedItem>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.product_code FROM stocktake_items si
         WHERE si.stocktake_id = ?1 AND NOT EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id)
         ORDER BY si.id",
    )?;
    let rows = stmt.query_map(rusqlite::params![stocktake_id], |row| {
        Ok(UncountedItem {
            stocktake_item_id: row.get(0)?,
            product_code: row.get(1)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// 棚卸し進捗を取得する
///
/// 20-io-product-repo.md §2.11
pub fn get_stocktake_progress(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<StocktakeProgress, DbError> {
    conn.query_row(
        "SELECT COUNT(*) AS total,
                COALESCE(SUM(EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id)), 0) AS counted,
                COUNT(*) - COALESCE(SUM(EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id)), 0) AS uncounted
         FROM stocktake_items si
         WHERE si.stocktake_id = ?1",
        rusqlite::params![stocktake_id],
        |row| {
            Ok(StocktakeProgress {
                total_items: row.get(0)?,
                counted_items: row.get(1)?,
                uncounted_items: row.get(2)?,
            })
        },
    )
    .map_err(DbError::from)
}

/// 棚卸し対象商品を全件取得する（フィルタなし）
///
/// architecture/biz-task-specs.md BIZ-06 ステップ3+4 の和集合 = 全商品（自動の 0 の入力は持たない。35）。
///
/// 20-io-product-repo.md §2.11
pub fn find_stocktake_eligible_products(
    conn: &DbConnection,
) -> Result<Vec<ProductForStocktake>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code, stock_quantity, cost_price_centi, is_discontinued
         FROM products
         ORDER BY product_code ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ProductForStocktake {
            product_code: row.get(0)?,
            stock_quantity: row.get(1)?,
            cost_price_centi: row.get(2)?,
            is_discontinued: row.get(3)?,
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// 棚卸し明細の一覧取得（ページング・フィルタ付き）
///
/// 動的WHERE + Box<dyn ToSql> パターン（product_repo::search_products 踏襲）
///
/// 20-io-product-repo.md §2.11
pub fn list_stocktake_items(
    conn: &DbConnection,
    stocktake_id: i64,
    department_id: Option<i64>,
    counted_only: Option<bool>,
    page: u32,
    per_page: u32,
) -> Result<super::PaginatedResult<StocktakeItemRow>, DbError> {
    if page < 1 {
        return Err(DbError::QueryFailed("page must be >= 1".to_string()));
    }
    let per_page = per_page.min(PAGINATION_MAX_PER_PAGE);
    if per_page < 1 {
        return Err(DbError::QueryFailed("per_page must be >= 1".to_string()));
    }

    let mut conditions = vec!["si.stocktake_id = ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(stocktake_id)];

    if let Some(dept_id) = department_id {
        conditions.push(format!("p.department_id = ?{}", params.len() + 1));
        params.push(Box::new(dept_id));
    }

    if let Some(counted) = counted_only {
        if counted {
            conditions.push(
                "EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id)"
                    .to_string(),
            );
        } else {
            conditions.push(
                "NOT EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id)"
                    .to_string(),
            );
        }
    }

    let where_clause = format!("WHERE {}", conditions.join(" AND "));

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    // COUNT
    let count_sql = format!(
        "SELECT COUNT(*) FROM stocktake_items si
         JOIN products p ON si.product_code = p.product_code
         {}",
        where_clause
    );
    let total_count: u32 = conn.query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))?;

    // OFFSET
    let offset = page
        .checked_sub(1)
        .and_then(|p| p.checked_mul(per_page))
        .ok_or_else(|| DbError::QueryFailed("page/per_page overflow".to_string()))?;

    // DATA
    let data_sql = format!(
        "SELECT si.id, si.stocktake_id, si.product_code, p.name, d.name,
                si.system_stock, si.actual_count, si.counted_at, p.stock_quantity,
                p.is_discontinued, p.stock_unit, p.stock_range_low, p.stock_range_high
         FROM stocktake_items si
         JOIN products p ON si.product_code = p.product_code
         JOIN departments d ON p.department_id = d.id
         {} ORDER BY si.id ASC LIMIT {} OFFSET {}",
        where_clause, per_page, offset
    );
    let mut stmt = conn.prepare(&data_sql)?;
    let rows = stmt.query_map(params_ref.as_slice(), item_row)?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(super::PaginatedResult {
        items,
        total_count,
        page,
        per_page,
    })
}

/// 棚卸し記録のヘッダ、明細、完了の時点のカーソルを取得する（補正の movement は読まない。65）。
///
/// 20-io-product-repo.md §2.11a / REQ-206 / REQ-207
pub fn get_stocktake_record_detail(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<StocktakeRecordDetailCore, DbError> {
    let header = find_stocktake_by_id(conn, stocktake_id)?.ok_or(DbError::NotFound)?;
    let item_count = conn.query_row(
        "SELECT COUNT(*) FROM stocktake_items WHERE stocktake_id = ?1",
        [stocktake_id],
        |row| row.get(0),
    )?;
    let cutoff = conn.query_row(
        "SELECT cutoff_import_id, cutoff_count_id, cutoff_movement_id
         FROM stocktakes WHERE id = ?1",
        [stocktake_id],
        |row| {
            Ok(StocktakeCutoff {
                import_id: row.get(0)?,
                count_id: row.get(1)?,
                movement_id: row.get(2)?,
            })
        },
    )?;
    let mut item_stmt = conn.prepare(
        "SELECT si.product_code, p.name, d.name, p.stock_unit,
                si.valuation_quantity, si.valuation_cost_price_centi
         FROM stocktake_items si
         JOIN products p ON p.product_code = si.product_code
         JOIN departments d ON d.id = p.department_id
         WHERE si.stocktake_id = ?1
         ORDER BY si.product_code ASC",
    )?;
    let items = item_stmt
        .query_map([stocktake_id], |row| {
            Ok(StocktakeRecordItemRow {
                product_code: row.get(0)?,
                product_name: row.get(1)?,
                department_name: row.get(2)?,
                stock_unit: parse_stock_unit(row.get(3)?)?,
                valuation_quantity: row.get(4)?,
                valuation_cost_price_centi: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StocktakeRecordDetailCore {
        header,
        item_count,
        items,
        cutoff,
    })
}

/// 完了した棚卸しの ID・完了日時・完了の時点のカーソルを完了日時の昇順で返す
/// （BIZ-03 の取消の前の読取りが完了の後の比較の範囲を判定する。20 §2.11a）
pub(crate) fn list_completed_stocktake_cutoffs(
    conn: &DbConnection,
) -> Result<Vec<(i64, String, StocktakeCutoff)>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, completed_at, cutoff_import_id, cutoff_count_id, cutoff_movement_id
         FROM stocktakes WHERE status = 'completed'
         ORDER BY completed_at, id",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                StocktakeCutoff {
                    import_id: row.get(2)?,
                    count_id: row.get(3)?,
                    movement_id: row.get(4)?,
                },
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// 完了の検査と評価に使う明細を id の順で取得する（35「完了」）
pub(crate) fn list_items_for_completion(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<Vec<StocktakeItemForCompletion>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.product_code, EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id),
                p.is_discontinued, p.stock_quantity, p.stock_recheck, p.cost_price_centi, p.stock_unit
         FROM stocktake_items si
         JOIN products p ON p.product_code = si.product_code
         WHERE si.stocktake_id = ?1
         ORDER BY si.id",
    )?;
    let rows = stmt
        .query_map([stocktake_id], |row| {
            Ok(StocktakeItemForCompletion {
                id: row.get(0)?,
                product_code: row.get(1)?,
                counted: row.get(2)?,
                is_discontinued: row.get(3)?,
                stock_quantity: row.get(4)?,
                stock_recheck: row.get(5)?,
                cost_price_centi: row.get(6)?,
                stock_unit: parse_stock_unit(row.get(7)?)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// 進行中の棚卸しのその商品の明細（明細 ID・棚卸し ID）。無ければ None（40 の回復先）
pub(crate) fn find_active_item_for_product(
    conn: &DbConnection,
    product_code: &str,
) -> Result<Option<(i64, i64)>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.stocktake_id FROM stocktake_items si
         JOIN stocktakes st ON st.id = si.stocktake_id
         WHERE st.status = 'in_progress' AND si.product_code = ?1
         ORDER BY st.started_at DESC, st.id DESC, si.id ASC LIMIT 1",
    )?;
    let mut rows = stmt.query([product_code])?;
    match rows.next()? {
        Some(row) => Ok(Some((row.get(0)?, row.get(1)?))),
        None => Ok(None),
    }
}

/// 商品の写し（名前・単位・在庫・要再確認の印と範囲）。無ければ None
pub(crate) fn find_stock_copy(
    conn: &DbConnection,
    product_code: &str,
) -> Result<Option<StockCopyRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code, name, stock_unit, stock_quantity, stock_recheck,
                stock_range_low, stock_range_high
         FROM products WHERE product_code = ?1",
    )?;
    let mut rows = stmt.query([product_code])?;
    match rows.next()? {
        Some(row) => Ok(Some(StockCopyRow {
            product_code: row.get(0)?,
            name: row.get(1)?,
            stock_unit: parse_stock_unit(row.get(2)?)?,
            stock_quantity: row.get(3)?,
            stock_recheck: row.get(4)?,
            stock_range_low: row.get(5)?,
            stock_range_high: row.get(6)?,
        })),
        None => Ok(None),
    }
}

/// 写しが要再確認（`stock_recheck = 1`）の商品コードを商品コードの順で返す（35「読取り」）
pub(crate) fn list_recheck_product_codes(conn: &DbConnection) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT product_code FROM products WHERE stock_recheck = 1 ORDER BY product_code",
    )?;
    let rows = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn item_row(row: &rusqlite::Row) -> rusqlite::Result<StocktakeItemRow> {
    Ok(StocktakeItemRow {
        id: row.get(0)?,
        stocktake_id: row.get(1)?,
        product_code: row.get(2)?,
        name: row.get(3)?,
        department_name: row.get(4)?,
        system_stock: row.get(5)?,
        actual_count: row.get(6)?,
        counted_at: row.get(7)?,
        current_stock: row.get(8)?,
        is_discontinued: row.get(9)?,
        stock_unit: parse_stock_unit(row.get(10)?)?,
        stock_range_low: row.get(11)?,
        stock_range_high: row.get(12)?,
    })
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_database;
    use crate::db::product_repo::{self, NewProduct};

    fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    fn seed_product(conn: &DbConnection, product_code: &str) {
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

    fn seed_product_with_jan(conn: &DbConnection, product_code: &str, jan_code: Option<&str>) {
        let product = NewProduct {
            product_code: product_code.to_string(),
            jan_code: jan_code.map(str::to_string),
            name: format!("テスト商品{}", product_code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: 7,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
    }

    /// テスト用に棚卸しヘッダを直接SQLで作成するヘルパー
    fn create_stocktake(conn: &DbConnection, status: &str, started_at: &str) -> i64 {
        conn.execute(
            "INSERT INTO stocktakes (started_at, status) VALUES (?1, ?2)",
            rusqlite::params![started_at, status],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn create_completed_stocktake(
        conn: &DbConnection,
        started_at: &str,
        completed_at: &str,
        total_cost: i64,
    ) -> i64 {
        conn.execute(
            "INSERT INTO stocktakes (started_at, completed_at, status, total_cost)
             VALUES (?1, ?2, 'completed', ?3)",
            rusqlite::params![started_at, completed_at, total_cost],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    // ===== FUNC-2.9: find_active_stocktake テスト =====

    #[test]
    fn test_find_active_stocktake_req205_none() {
        // REQ-205: 棚卸し（進行中棚卸しなし → None）
        // FUNC-2.9: find_active_stocktake — 棚卸しなしでNone
        let (_dir, conn) = setup_test_db();
        let result = find_active_stocktake(&conn).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_find_active_stocktake_req205_in_progress() {
        // REQ-205: 棚卸し（進行中棚卸しあり → Some）
        // FUNC-2.9: find_active_stocktake — 進行中あり
        let (_dir, conn) = setup_test_db();
        let id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");

        let result = find_active_stocktake(&conn).unwrap();
        assert!(result.is_some());
        let stocktake = result.unwrap();
        assert_eq!(stocktake.id, id);
        assert_eq!(stocktake.status, "in_progress");
        assert!(stocktake.completed_at.is_none());
        assert!(stocktake.total_cost.is_none());
    }

    #[test]
    fn test_find_active_stocktake_req205_only_completed() {
        // REQ-205: 棚卸し（completedのみ → None）
        // FUNC-2.9: find_active_stocktake — completedのみでNone
        let (_dir, conn) = setup_test_db();
        create_stocktake(&conn, "completed", "2026-09-01T09:00:00");

        let result = find_active_stocktake(&conn).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_find_active_stocktake_req205_returns_latest_by_id() {
        // REQ-205: 棚卸し（複数進行中 → id DESC で最新を返す）
        // FUNC-2.9: find_active_stocktake — 同一started_atの2件でid DESCの方を返す
        let (_dir, conn) = setup_test_db();
        let id1 = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        let id2 = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        assert!(id2 > id1);

        let result = find_active_stocktake(&conn).unwrap().unwrap();
        assert_eq!(result.id, id2, "id DESCで新しい方を返すべき");
    }

    // ===== FUNC-2.9: insert_stocktake_item テスト =====

    #[test]
    fn test_insert_stocktake_item_req205_normal() {
        // REQ-205: 棚卸し（明細INSERT正常 — actual_count=None）
        // FUNC-2.9: insert_stocktake_item — 正常INSERT（actual_count=None=未入力）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "ST-001");
        let stocktake_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");

        let item = NewStocktakeItem {
            stocktake_id,
            product_code: "ST-001".to_string(),
            system_stock: 10,
            actual_count: None,
        };
        insert_stocktake_item(&conn, &item).unwrap();

        let (sys_stock, actual): (i64, Option<i64>) = conn
            .query_row(
                "SELECT system_stock, actual_count FROM stocktake_items
                 WHERE stocktake_id = ?1 AND product_code = ?2",
                rusqlite::params![stocktake_id, "ST-001"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(sys_stock, 10);
        assert!(actual.is_none(), "actual_count は NULL（未入力）");
    }

    #[test]
    fn test_insert_stocktake_item_req205_fk_violation_stocktake() {
        // REQ-205: 棚卸し（存在しないstocktake_id → FK違反）
        // FUNC-2.9: insert_stocktake_item — 存在しないstocktake_idでFK違反
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "ST-FK1");

        let item = NewStocktakeItem {
            stocktake_id: 9999,
            product_code: "ST-FK1".to_string(),
            system_stock: 0,
            actual_count: None,
        };
        let result = insert_stocktake_item(&conn, &item);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "不正なstocktake_idで ForeignKeyViolation が返るべき: {:?}",
            result
        );
    }

    #[test]
    fn test_insert_stocktake_item_req205_fk_violation_product() {
        // REQ-205: 棚卸し（存在しないproduct_code → FK違反）
        // FUNC-2.9: insert_stocktake_item — 存在しないproduct_codeでFK違反
        let (_dir, conn) = setup_test_db();
        let stocktake_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");

        let item = NewStocktakeItem {
            stocktake_id,
            product_code: "NONEXISTENT".to_string(),
            system_stock: 0,
            actual_count: None,
        };
        let result = insert_stocktake_item(&conn, &item);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "不正なproduct_codeで ForeignKeyViolation が返るべき: {:?}",
            result
        );
    }

    // テストヘルパー: 棚卸し明細を直接SQLで追加
    fn seed_stocktake_item(
        conn: &DbConnection,
        stocktake_id: i64,
        product_code: &str,
        system_stock: i64,
        actual_count: Option<i64>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![stocktake_id, product_code, system_stock, actual_count],
        )
        .unwrap();
        let item_id = conn.last_insert_rowid();
        // 計数済みは明細を参照する計数があること（35。Matrix T11）。数のある明細には同じ値の計数を入れる
        if let Some(value) = actual_count {
            conn.execute(
                "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                     stocktake_item_id, counted_at)
                 VALUES (?1, ?2, 'default', '2026-10-01', ?3, '2026-10-01T10:00:00')",
                rusqlite::params![product_code, value, item_id],
            )
            .unwrap();
        }
        item_id
    }

    // ===== UI-10 T-R1: find_stocktake_item_by_code =====

    #[test]
    fn test_find_stocktake_item_by_code_req205_product_code_match() {
        // REQ-205: 棚卸し（商品コード完全一致で棚卸し明細を取得）
        let (_dir, conn) = setup_test_db();
        seed_product_with_jan(&conn, "FC-001", Some("4900000000001"));
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        let item_id = seed_stocktake_item(&conn, st_id, "FC-001", 7, None);

        let result = find_stocktake_item_by_code(&conn, st_id, "FC-001").unwrap();

        let item = result.expect("商品コード一致の棚卸し明細が返るべき");
        assert_eq!(item.id, item_id);
        assert_eq!(item.product_code, "FC-001");
        assert_eq!(item.name, "テスト商品FC-001");
    }

    #[test]
    fn test_find_stocktake_item_by_code_req205_jan_code_match() {
        // REQ-205: 棚卸し（JANコード完全一致で棚卸し明細を取得）
        let (_dir, conn) = setup_test_db();
        seed_product_with_jan(&conn, "FJ-001", Some("4900000000002"));
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        let item_id = seed_stocktake_item(&conn, st_id, "FJ-001", 7, Some(6));

        let result = find_stocktake_item_by_code(&conn, st_id, "4900000000002").unwrap();

        let item = result.expect("JAN一致の棚卸し明細が返るべき");
        assert_eq!(item.id, item_id);
        assert_eq!(item.product_code, "FJ-001");
        assert_eq!(item.actual_count, Some(6));
    }

    #[test]
    fn test_find_stocktake_item_by_code_req205_none_for_no_match() {
        // REQ-205: 棚卸し（対象なしは None）
        let (_dir, conn) = setup_test_db();
        seed_product_with_jan(&conn, "FN-001", Some("4900000000003"));
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "FN-001", 7, None);

        let result = find_stocktake_item_by_code(&conn, st_id, "NO-MATCH").unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_find_stocktake_item_by_code_req205_same_jan_returns_lowest_item_id() {
        // REQ-205: 棚卸し（同一JAN複数候補は ORDER BY si.id ASC LIMIT 1 で決定的に返す）
        let (_dir, conn) = setup_test_db();
        seed_product_with_jan(&conn, "FD-001", Some("4900000000004"));
        seed_product_with_jan(&conn, "FD-002", Some("4900000000004"));
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        let first_item_id = seed_stocktake_item(&conn, st_id, "FD-002", 7, None);
        let second_item_id = seed_stocktake_item(&conn, st_id, "FD-001", 7, None);
        assert!(first_item_id < second_item_id);

        let result = find_stocktake_item_by_code(&conn, st_id, "4900000000004").unwrap();

        let item = result.expect("同一JAN候補のうち最小 si.id の明細が返るべき");
        assert_eq!(item.id, first_item_id);
        assert_eq!(item.product_code, "FD-002");
    }

    #[test]
    fn test_find_stocktake_item_by_code_req205_includes_is_discontinued() {
        // REQ-205 / UI-10-D13: is_discontinued が products.is_discontinued の実値を反映する
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "DISC-001", true, 7, 300, 1);
        seed_product_custom(&conn, "OK-001", false, 7, 300, 1);
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "DISC-001", 7, None);
        seed_stocktake_item(&conn, st_id, "OK-001", 7, None);

        let discontinued = find_stocktake_item_by_code(&conn, st_id, "DISC-001")
            .unwrap()
            .expect("廃番商品の明細が返るべき");
        assert!(
            discontinued.is_discontinued,
            "廃番商品は is_discontinued=true"
        );

        let normal = find_stocktake_item_by_code(&conn, st_id, "OK-001")
            .unwrap()
            .expect("非廃番商品の明細が返るべき");
        assert!(
            !normal.is_discontinued,
            "非廃番商品は is_discontinued=false"
        );
    }

    // ===== UI-10 T-R2: find_last_completed_stocktake =====

    #[test]
    fn test_find_last_completed_stocktake_req205_returns_latest_completed() {
        // REQ-205: 棚卸し（最後に完了した棚卸しを completed_at DESC, id DESC で取得）
        let (_dir, conn) = setup_test_db();
        create_completed_stocktake(&conn, "2026-09-01T09:00:00", "2026-09-30T18:00:00", 1000);
        let latest_id =
            create_completed_stocktake(&conn, "2026-10-01T09:00:00", "2026-10-31T18:00:00", 2000);
        create_stocktake(&conn, "in_progress", "2026-11-01T09:00:00");

        let result = find_last_completed_stocktake(&conn).unwrap();

        let stocktake = result.expect("完了済み棚卸しが返るべき");
        assert_eq!(stocktake.stocktake_id, latest_id);
        assert_eq!(stocktake.completed_at, "2026-10-31T18:00:00");
        assert_eq!(stocktake.total_cost, 2000);
    }

    #[test]
    fn test_find_last_completed_stocktake_req205_none() {
        // REQ-205: 棚卸し（完了済みがない場合は None）
        let (_dir, conn) = setup_test_db();
        create_stocktake(&conn, "in_progress", "2026-11-01T09:00:00");

        let result = find_last_completed_stocktake(&conn).unwrap();

        assert!(result.is_none());
    }

    // ===== FUNC-2.11: insert_stocktake テスト =====

    #[test]
    fn test_insert_stocktake_req205() {
        // REQ-205: 棚卸し（棚卸しヘッダ正常作成 — status='in_progress'）
        // FUNC-2.11: insert_stocktake — 正常作成、status='in_progress'
        let (_dir, conn) = setup_test_db();
        let id = insert_stocktake(&conn, "2026-10-01T09:00:00").unwrap();
        assert!(id > 0);

        let st = find_stocktake_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(st.status, "in_progress");
        assert_eq!(st.started_at, "2026-10-01T09:00:00");
        assert!(st.completed_at.is_none());
        assert!(st.total_cost.is_none());
    }

    // ===== FUNC-2.11: find_stocktake_by_id テスト =====

    #[test]
    fn test_find_stocktake_by_id_req205_found() {
        // REQ-205: 棚卸し（存在するID → Some）
        // FUNC-2.11: find_stocktake_by_id — 存在するIDでSome
        let (_dir, conn) = setup_test_db();
        let id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");

        let result = find_stocktake_by_id(&conn, id).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, id);
    }

    #[test]
    fn test_find_stocktake_by_id_req205_not_found() {
        // REQ-205: 棚卸し（存在しないID → None）
        // FUNC-2.11: find_stocktake_by_id — 存在しないIDでNone
        let (_dir, conn) = setup_test_db();
        let result = find_stocktake_by_id(&conn, 9999).unwrap();
        assert!(result.is_none());
    }

    // ===== FUNC-2.11: complete_stocktake テスト =====

    #[test]
    fn test_complete_stocktake_req205() {
        // REQ-205: 棚卸し（確定 — in_progress → completed）
        // FUNC-2.11: complete_stocktake — in_progress → completed
        let (_dir, conn) = setup_test_db();
        let id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");

        let cutoff = StocktakeCutoff {
            import_id: Some(7),
            count_id: None,
            movement_id: Some(0),
        };
        complete_stocktake(&conn, id, 150000, "2026-12-31T18:00:00", cutoff).unwrap();

        let st = find_stocktake_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(st.status, "completed");
        assert_eq!(st.total_cost, Some(150000));
        assert_eq!(st.completed_at, Some("2026-12-31T18:00:00".to_string()));
        // 完了の TX がカーソルも書く（35・tracking。期待を変える。Matrix T11）
        assert_eq!(
            get_stocktake_record_detail(&conn, id).unwrap().cutoff,
            cutoff
        );
    }

    #[test]
    fn test_complete_stocktake_req205_already_completed() {
        // REQ-205: 棚卸し（完了済みへの二重確定 → NotFound）
        // FUNC-2.11 R-4: complete_stocktake — 完了済みへの呼出し → NotFound
        let (_dir, conn) = setup_test_db();
        let id = create_stocktake(&conn, "completed", "2026-10-01T09:00:00");

        let result = complete_stocktake(
            &conn,
            id,
            100,
            "2026-12-31T18:00:00",
            StocktakeCutoff::default(),
        );
        assert!(
            matches!(result, Err(DbError::NotFound)),
            "完了済みの棚卸しに対して NotFound が返るべき: {:?}",
            result
        );
    }

    // ===== FUNC-2.11: update_stocktake_item_count テスト =====

    #[test]
    fn test_update_item_count_req205() {
        // REQ-205: 棚卸し（明細カウント正常更新）
        // FUNC-2.11: update_stocktake_item_count — 正常更新
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SC-001");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        let item_id = seed_stocktake_item(&conn, st_id, "SC-001", 10, None);

        let result = update_stocktake_item_count(&conn, item_id, 8, "2026-10-15T14:00:00").unwrap();
        assert!(result, "更新成功で true を返すべき");

        let (actual, counted_at): (Option<i64>, Option<String>) = conn
            .query_row(
                "SELECT actual_count, counted_at FROM stocktake_items WHERE id = ?1",
                rusqlite::params![item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(actual, Some(8));
        assert_eq!(counted_at, Some("2026-10-15T14:00:00".to_string()));
    }

    #[test]
    fn test_update_item_count_req205_not_found() {
        // REQ-205: 棚卸し（存在しないitem_id → false）
        // FUNC-2.11 R-4: update_stocktake_item_count — 存在しないitem_id → false
        let (_dir, conn) = setup_test_db();
        let result = update_stocktake_item_count(&conn, 9999, 5, "2026-10-15T14:00:00").unwrap();
        assert!(!result, "存在しないIDで false を返すべき");
    }

    // ===== FUNC-2.11: update_stocktake_item_valuation テスト =====

    #[test]
    fn test_update_item_valuation_req205() {
        // REQ-205: 棚卸し（明細評価原価の設定）
        // FUNC-2.11: update_stocktake_item_valuation — 評価原価の設定
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SV-001");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        let item_id = seed_stocktake_item(&conn, st_id, "SV-001", 10, Some(8));

        update_stocktake_item_valuation(&conn, item_id, 300, 8).unwrap();

        let (val, quantity): (Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT valuation_cost_price_centi, valuation_quantity FROM stocktake_items WHERE id = ?1",
                rusqlite::params![item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(val, Some(300));
        // 評価数量も固定する（35。期待を変える。Matrix T11）
        assert_eq!(quantity, Some(8));
    }

    // ===== FUNC-2.11: count_uncounted_items テスト =====

    #[test]
    fn test_count_uncounted_items_req205() {
        // REQ-205: 棚卸し（未入力件数のカウント）
        // FUNC-2.11: count_uncounted_items — NULL件数のカウント
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "CU-001");
        seed_product(&conn, "CU-002");
        seed_product(&conn, "CU-003");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "CU-001", 10, None); // 未入力
        seed_stocktake_item(&conn, st_id, "CU-002", 5, Some(5)); // 入力済み
        seed_stocktake_item(&conn, st_id, "CU-003", 3, None); // 未入力

        let count = count_uncounted_items(&conn, st_id).unwrap();
        assert_eq!(count, 2, "未入力は2件");
    }

    // テストヘルパー: カスタム商品をseed
    fn seed_product_custom(
        conn: &DbConnection,
        product_code: &str,
        is_discontinued: bool,
        stock_quantity: i64,
        cost_price_centi: i64,
        department_id: i64,
    ) {
        let product = NewProduct {
            product_code: product_code.to_string(),
            jan_code: None,
            name: format!("商品{}", product_code),
            department_id,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity,
            stock_unit: "pcs".to_string(),
            is_discontinued,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
    }

    // ===== FUNC-2.11: find_stocktake_item_with_parent_status テスト =====

    #[test]
    fn test_find_item_with_parent_status_req205_found() {
        // REQ-205: 棚卸し（明細と親ステータス正常取得）
        // FUNC-2.11: find_stocktake_item_with_parent_status — 正常取得
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "WP-001");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        let item_id = seed_stocktake_item(&conn, st_id, "WP-001", 10, None);

        let result = find_stocktake_item_with_parent_status(&conn, item_id).unwrap();
        assert!(result.is_some());
        let (item, status) = result.unwrap();
        assert_eq!(item.id, item_id);
        assert_eq!(item.product_code, "WP-001");
        assert_eq!(status, "in_progress");
    }

    #[test]
    fn test_find_item_with_parent_status_req205_not_found() {
        // REQ-205: 棚卸し（存在しない明細ID → None）
        // FUNC-2.11: find_stocktake_item_with_parent_status — 存在しないID
        let (_dir, conn) = setup_test_db();
        let result = find_stocktake_item_with_parent_status(&conn, 9999).unwrap();
        assert!(result.is_none());
    }

    // ===== FUNC-2.11: list_uncounted_items テスト =====

    #[test]
    fn test_list_uncounted_items_req205() {
        // REQ-205: 棚卸し（未入力明細一覧 — 未入力のみ返却）
        // FUNC-2.11: list_uncounted_items — 未入力のみ返却
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "LU-001");
        seed_product(&conn, "LU-002");
        seed_product(&conn, "LU-003");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "LU-001", 10, None);
        seed_stocktake_item(&conn, st_id, "LU-002", 5, Some(5));
        seed_stocktake_item(&conn, st_id, "LU-003", 3, None);

        let items = list_uncounted_items(&conn, st_id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].product_code, "LU-001");
        assert_eq!(items[1].product_code, "LU-003");
    }

    // ===== FUNC-2.11: get_stocktake_items_for_complete テスト =====

    // ===== FUNC-2.11: get_stocktake_progress テスト =====

    #[test]
    fn test_get_stocktake_progress_req205() {
        // REQ-205: 棚卸し（進捗計算 — 入力済み/未入力件数）
        // FUNC-2.11: get_stocktake_progress — 進捗計算
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "GP-001");
        seed_product(&conn, "GP-002");
        seed_product(&conn, "GP-003");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "GP-001", 10, Some(10));
        seed_stocktake_item(&conn, st_id, "GP-002", 5, None);
        seed_stocktake_item(&conn, st_id, "GP-003", 3, Some(3));

        let progress = get_stocktake_progress(&conn, st_id).unwrap();
        assert_eq!(progress.total_items, 3);
        assert_eq!(progress.counted_items, 2);
        assert_eq!(progress.uncounted_items, 1);
    }

    // ===== FUNC-2.11: find_stocktake_eligible_products テスト =====

    #[test]
    fn test_find_eligible_products_req205_mixed() {
        // REQ-205: 棚卸し（棚卸し対象商品取得 — 通常・廃番混在）
        // FUNC-2.11: find_stocktake_eligible_products — 全商品が返却される
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "EP-001", false, 10, 300, 1); // 通常
        seed_product_custom(&conn, "EP-002", true, 5, 200, 1); // 廃番stock>0
        seed_product_custom(&conn, "EP-003", true, 0, 100, 1); // 廃番stock=0

        let products = find_stocktake_eligible_products(&conn).unwrap();
        assert_eq!(products.len(), 3, "全3商品が返却されるべき");
    }

    #[test]
    fn test_find_eligible_products_req205_empty_db() {
        // REQ-205: 棚卸し（棚卸し対象商品取得 — 商品0件）
        // FUNC-2.11: find_stocktake_eligible_products — 商品0件
        let (_dir, conn) = setup_test_db();
        let products = find_stocktake_eligible_products(&conn).unwrap();
        assert!(products.is_empty());
    }

    #[test]
    fn test_find_eligible_products_req205_fields() {
        // REQ-205: 棚卸し（棚卸し対象商品フィールド値の確認）
        // FUNC-2.11: find_stocktake_eligible_products — フィールド値の確認
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "EF-001", true, 5, 250, 1);

        let products = find_stocktake_eligible_products(&conn).unwrap();
        assert_eq!(products.len(), 1);
        let p = &products[0];
        assert_eq!(p.product_code, "EF-001");
        assert_eq!(p.stock_quantity, 5);
        assert_eq!(p.cost_price_centi, 250);
        assert!(p.is_discontinued);
    }

    #[test]
    fn test_find_eligible_products_req205_includes_discontinued_zero() {
        // REQ-205: 棚卸し（廃番stock=0 の商品も対象に含む）
        // FUNC-2.11 R-4 + P2-1: 廃番stock=0 が返却されること
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "DZ-001", true, 0, 100, 1);
        seed_product_custom(&conn, "DZ-002", true, 0, 200, 1);

        let products = find_stocktake_eligible_products(&conn).unwrap();
        assert_eq!(products.len(), 2, "廃番stock=0 の商品も含まれるべき");
        assert!(products
            .iter()
            .all(|p| p.is_discontinued && p.stock_quantity == 0));
    }

    // ===== FUNC-2.11: list_stocktake_items テスト =====

    #[test]
    fn test_list_stocktake_items_req205_basic() {
        // REQ-205: 棚卸し（明細一覧取得 — ページング正常）
        // FUNC-2.11: list_stocktake_items — ページング正常
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "LS-001");
        seed_product(&conn, "LS-002");
        seed_product(&conn, "LS-003");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "LS-001", 10, Some(8));
        seed_stocktake_item(&conn, st_id, "LS-002", 5, None);
        seed_stocktake_item(&conn, st_id, "LS-003", 3, Some(3));

        let result = list_stocktake_items(&conn, st_id, None, None, 1, 10).unwrap();
        assert_eq!(result.total_count, 3);
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.page, 1);
        // current_stock は products.stock_quantity から取得
        assert_eq!(result.items[0].current_stock, 0); // seed_productのデフォルトは0
    }

    #[test]
    fn test_list_stocktake_items_req205_includes_is_discontinued() {
        // REQ-205 / UI-10-D13: is_discontinued が products.is_discontinued の実値を反映する
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "LDISC-001", true, 7, 300, 1);
        seed_product_custom(&conn, "LOK-001", false, 7, 300, 1);
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "LDISC-001", 7, None);
        seed_stocktake_item(&conn, st_id, "LOK-001", 7, None);

        let result = list_stocktake_items(&conn, st_id, None, None, 1, 10).unwrap();
        let discontinued = result
            .items
            .iter()
            .find(|item| item.product_code == "LDISC-001")
            .expect("廃番商品の明細が返るべき");
        assert!(
            discontinued.is_discontinued,
            "廃番商品は is_discontinued=true"
        );
        let normal = result
            .items
            .iter()
            .find(|item| item.product_code == "LOK-001")
            .expect("非廃番商品の明細が返るべき");
        assert!(
            !normal.is_discontinued,
            "非廃番商品は is_discontinued=false"
        );
    }

    #[test]
    fn test_list_stocktake_items_req205_dept_filter() {
        // REQ-205: 棚卸し（明細一覧取得 — 部門フィルタ）
        // FUNC-2.11: list_stocktake_items — 部門フィルタ
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "LD-001", false, 10, 300, 1); // 部門1
        seed_product_custom(&conn, "LD-002", false, 5, 200, 3); // 部門3
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "LD-001", 10, None);
        seed_stocktake_item(&conn, st_id, "LD-002", 5, None);

        let result = list_stocktake_items(&conn, st_id, Some(3), None, 1, 10).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].product_code, "LD-002");
    }

    #[test]
    fn test_list_stocktake_items_req205_counted_filter() {
        // REQ-205: 棚卸し（明細一覧取得 — 入力済み/未入力フィルタ）
        // FUNC-2.11: list_stocktake_items — 入力済みフィルタ
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "LC-001");
        seed_product(&conn, "LC-002");
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "LC-001", 10, Some(8));
        seed_stocktake_item(&conn, st_id, "LC-002", 5, None);

        // 入力済みのみ
        let counted = list_stocktake_items(&conn, st_id, None, Some(true), 1, 10).unwrap();
        assert_eq!(counted.total_count, 1);
        assert_eq!(counted.items[0].product_code, "LC-001");

        // 未入力のみ
        let uncounted = list_stocktake_items(&conn, st_id, None, Some(false), 1, 10).unwrap();
        assert_eq!(uncounted.total_count, 1);
        assert_eq!(uncounted.items[0].product_code, "LC-002");
    }

    #[test]
    fn test_list_stocktake_items_req205_dept_and_counted_combined() {
        // REQ-205: 棚卸し（明細一覧取得 — 部門 + 入力済み ANDフィルタ）
        // FUNC-2.11 R-4: 部門 + 入力済み の AND フィルタ
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "DC-001", false, 10, 300, 1); // 部門1 入力済み
        seed_product_custom(&conn, "DC-002", false, 5, 200, 1); // 部門1 未入力
        seed_product_custom(&conn, "DC-003", false, 3, 100, 3); // 部門3 入力済み
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "DC-001", 10, Some(8));
        seed_stocktake_item(&conn, st_id, "DC-002", 5, None);
        seed_stocktake_item(&conn, st_id, "DC-003", 3, Some(3));

        // 部門1 かつ 入力済み → DC-001のみ
        let result = list_stocktake_items(&conn, st_id, Some(1), Some(true), 1, 10).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].product_code, "DC-001");
    }

    #[test]
    fn test_list_stocktake_items_req205_dept_and_uncounted_combined() {
        // REQ-205 / P7-2 C4: 部門 + 未入力 の AND フィルタ
        // 期待値はこの test の seed 行から独立転記する。
        let (_dir, conn) = setup_test_db();
        seed_product_custom(&conn, "DU-001", false, 10, 300, 1); // 部門1 入力済み
        seed_product_custom(&conn, "DU-002", false, 5, 200, 1); // 部門1 未入力
        seed_product_custom(&conn, "DU-003", false, 3, 100, 3); // 部門3 未入力
        let st_id = create_stocktake(&conn, "in_progress", "2026-10-01T09:00:00");
        seed_stocktake_item(&conn, st_id, "DU-001", 10, Some(8));
        seed_stocktake_item(&conn, st_id, "DU-002", 5, None);
        seed_stocktake_item(&conn, st_id, "DU-003", 3, None);

        let result = list_stocktake_items(&conn, st_id, Some(1), Some(false), 1, 10).unwrap();
        let actual_codes: Vec<&str> = result
            .items
            .iter()
            .map(|item| item.product_code.as_str())
            .collect();

        assert_eq!(actual_codes, vec!["DU-002"]);
        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_get_stocktake_record_detail_req206_not_found() {
        let (_dir, conn) = setup_test_db();

        let result = get_stocktake_record_detail(&conn, 99_999);

        assert!(matches!(result, Err(DbError::NotFound)));
    }

    #[test]
    fn test_get_stocktake_record_detail_req206_header_and_item_count() {
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SRD-001");
        seed_product(&conn, "SRD-002");
        let stocktake_id =
            create_completed_stocktake(&conn, "2026-08-01T09:00:00", "2026-08-01T18:00:00", 600);
        seed_stocktake_item(&conn, stocktake_id, "SRD-001", 10, Some(10));
        seed_stocktake_item(&conn, stocktake_id, "SRD-002", 4, Some(3));
        conn.execute(
            "UPDATE stocktake_items SET valuation_cost_price_centi = 300 WHERE stocktake_id = ?1",
            [stocktake_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO inventory_movements
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase)
             VALUES ('SRD-002', 'stocktake', -1, 3, 'stocktake', ?1, 'synthetic correction', 0, '2026-08-01T18:00:00', '2026-04-01', 0)",
            [stocktake_id],
        )
        .unwrap();

        let detail = get_stocktake_record_detail(&conn, stocktake_id).unwrap();

        assert_eq!(detail.header.id, stocktake_id);
        assert_eq!(detail.header.total_cost, Some(600));
        assert_eq!(detail.item_count, 2);
        // 65 の D-118 の節・packet S2（期待を変えた。b1 Matrix T11）: 明細は補正の movement でなく明細から読む
        // （補正の movement が 1 行あっても明細の数 2）
        assert_eq!(detail.items.len(), 2);
    }
}
