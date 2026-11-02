//! 廃棄・破損記録のCRUD操作
//!
//! 21-io-inventory-repo.md §10.5 に基づく実装。

use super::inventory_common::{validate_and_offset, ListQuery};
use super::inventory_repo::MovementRecord;
use super::product_repo::{parse_stock_unit, ProductStockUnit};
use super::{DbConnection, DbError, PaginatedResult};

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DisposalType {
    Disposal,
    Damage,
    Other,
}

impl DisposalType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disposal => "disposal",
            Self::Damage => "damage",
            Self::Other => "other",
        }
    }
}

fn parse_disposal_type(value: String) -> rusqlite::Result<DisposalType> {
    match value.as_str() {
        "disposal" => Ok(DisposalType::Disposal),
        "damage" => Ok(DisposalType::Damage),
        "other" => Ok(DisposalType::Other),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown disposal_type: {value}").into(),
        )),
    }
}

/// 廃棄記録INSERT用
///
/// 21-io-inventory-repo.md §10.5
#[derive(Debug)]
pub struct NewDisposalRecord {
    pub disposal_date: String,
    pub idempotency_key: String,
    pub request_fingerprint: String,
}

/// 廃棄明細INSERT用
#[derive(Debug)]
pub struct NewDisposalItem {
    pub disposal_record_id: i64,
    pub product_code: String,
    pub disposal_type: String,
    pub quantity: i64,
    pub cost_price_centi: i64,
    pub reason: String,
}

/// 廃棄記録一覧表示用
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DisposalRecordSummary {
    pub id: i64,
    pub disposal_date: String,
    pub created_at: String,
}

/// 入出庫履歴ハブの検索条件
///
/// 65-inventory-record-traceability.md §65.4
#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct InventoryRecordQuery {
    pub record_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub record_id: Option<i64>,
    pub product_keyword: Option<String>,
    pub department_id: Option<i64>,
    pub status: Option<String>,
    pub page: u32,
    pub per_page: u32,
}

/// 入出庫履歴ハブの1行
///
/// 初回実装は disposal_record のみを実データとして返す。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct InventoryRecordSummary {
    pub record_type: String,
    pub record_id: i64,
    pub business_date: String,
    pub representative_item: String,
    pub item_count: i64,
    pub status: String,
    pub created_at: String,
    pub detail_route: String,
}

/// 廃棄・破損詳細の明細行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DisposalRecordDetailItem {
    pub id: i64,
    pub product_code: String,
    pub product_name: String,
    pub department_name: String,
    pub stock_unit: ProductStockUnit,
    pub disposal_type: DisposalType,
    pub quantity: i64,
    pub cost_price_centi: i64,
    pub reason: String,
    /// 行のロス原価（1/100 円の整数の十進の文字列。SPEC-UNIT-D6）
    // repo は空で返し、BIZ が必ず埋める
    pub line_loss_cost_centi: String,
}

/// 廃棄・破損詳細
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct DisposalRecordDetail {
    pub id: i64,
    pub disposal_date: String,
    pub status: String,
    pub created_at: String,
    pub items: Vec<DisposalRecordDetailItem>,
    /// ロス原価合計（円。行のロス原価の和を円未満で四捨五入した値。SPEC-UNIT-D6）
    // repo は 0 で返し、BIZ が必ず埋める
    pub total_loss_cost: i64,
    pub movements: Vec<MovementRecord>,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// disposal_records に1行INSERT
///
/// 21-io-inventory-repo.md §10.5
pub fn insert_disposal_record(
    conn: &DbConnection,
    record: &NewDisposalRecord,
) -> Result<i64, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO disposal_records (disposal_date, idempotency_key, request_fingerprint, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            record.disposal_date,
            record.idempotency_key,
            record.request_fingerprint,
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// disposal_items に1行INSERT
pub fn insert_disposal_item(conn: &DbConnection, item: &NewDisposalItem) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO disposal_items (disposal_record_id, product_code, disposal_type, quantity, cost_price_centi, reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            item.disposal_record_id,
            item.product_code,
            item.disposal_type,
            item.quantity,
            item.cost_price_centi,
            item.reason,
        ],
    )?;
    Ok(())
}

/// 廃棄記録一覧をページング取得
pub fn list_disposal_records(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<DisposalRecordSummary>, DbError> {
    let (limit, offset) = validate_and_offset(query)?;

    let mut where_clauses = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref from) = query.date_from {
        where_clauses.push(format!("disposal_date >= ?{}", params.len() + 1));
        params.push(Box::new(from.clone()));
    }
    if let Some(ref to) = query.date_to {
        where_clauses.push(format!("disposal_date <= ?{}", params.len() + 1));
        params.push(Box::new(to.clone()));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM disposal_records {}", where_sql);
    let total_count: u32 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| row.get(0),
    )?;

    let data_sql = format!(
        "SELECT id, disposal_date, created_at \
         FROM disposal_records {} \
         ORDER BY disposal_date DESC, id DESC \
         LIMIT ?{} OFFSET ?{}",
        where_sql,
        params.len() + 1,
        params.len() + 2,
    );
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let mut stmt = conn.prepare(&data_sql)?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
            |row| {
                Ok(DisposalRecordSummary {
                    id: row.get(0)?,
                    disposal_date: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PaginatedResult {
        items: rows,
        total_count,
        page: query.page,
        per_page: query.per_page,
    })
}

/// 入出庫履歴ハブ用に業務記録をヘッダ単位で取得する。
///
/// 65-inventory-record-traceability.md §65.4 / TRACE-D1
/// 入庫 / 返品・交換 / 手動販売 / 廃棄・破損 / CSV取込み / 棚卸しを横断して返す。
pub fn list_inventory_records(
    conn: &DbConnection,
    query: &InventoryRecordQuery,
) -> Result<PaginatedResult<InventoryRecordSummary>, DbError> {
    let list_query = ListQuery {
        page: query.page,
        per_page: query.per_page,
        date_from: None,
        date_to: None,
    };
    let (limit, offset) = validate_and_offset(&list_query)?;

    if !matches!(
        query.status.as_deref(),
        None | Some("all") | Some("active") | Some("canceled") | Some("in_progress")
    ) {
        return Ok(PaginatedResult {
            items: Vec::new(),
            total_count: 0,
            page: query.page,
            per_page: query.per_page,
        });
    }

    struct RecordSpec {
        record_type: &'static str,
        header_table: &'static str,
        header_alias: &'static str,
        item_table: &'static str,
        item_alias_prefix: &'static str,
        item_fk_col: &'static str,
        date_col: &'static str,
        route_prefix: &'static str,
        status_expr: Option<&'static str>,
        date_expr: Option<&'static str>,
        created_at_col: Option<&'static str>,
    }

    const SPECS: &[RecordSpec] = &[
        RecordSpec {
            record_type: "receiving_record",
            header_table: "receiving_records",
            header_alias: "rr",
            item_table: "receiving_items",
            item_alias_prefix: "ri",
            item_fk_col: "receiving_record_id",
            date_col: "receiving_date",
            route_prefix: "/inventory/receiving/records",
            status_expr: None,
            date_expr: None,
            created_at_col: None,
        },
        RecordSpec {
            record_type: "return_record",
            header_table: "return_records",
            header_alias: "ret",
            item_table: "return_items",
            item_alias_prefix: "rti",
            item_fk_col: "return_record_id",
            date_col: "return_date",
            route_prefix: "/inventory/return/records",
            status_expr: None,
            date_expr: None,
            created_at_col: None,
        },
        RecordSpec {
            record_type: "manual_sale",
            header_table: "manual_sales",
            header_alias: "ms",
            item_table: "manual_sale_items",
            item_alias_prefix: "msi",
            item_fk_col: "manual_sale_id",
            date_col: "sale_date",
            route_prefix: "/inventory/manual-sale/records",
            status_expr: None,
            date_expr: None,
            created_at_col: None,
        },
        RecordSpec {
            record_type: "disposal_record",
            header_table: "disposal_records",
            header_alias: "dr",
            item_table: "disposal_items",
            item_alias_prefix: "di",
            item_fk_col: "disposal_record_id",
            date_col: "disposal_date",
            route_prefix: "/inventory/disposal/records",
            status_expr: None,
            date_expr: None,
            created_at_col: None,
        },
        RecordSpec {
            record_type: "csv_import",
            header_table: "csv_imports",
            header_alias: "ci",
            item_table: "sale_records",
            item_alias_prefix: "sr",
            item_fk_col: "csv_import_id",
            date_col: "settlement_date",
            route_prefix: "/csv-import/records",
            status_expr: Some(
                "CASE {header_alias}.status WHEN 'rolled_back' THEN 'canceled' ELSE 'active' END",
            ),
            date_expr: Some("{header_alias}.settlement_date"),
            created_at_col: Some("imported_at"),
        },
        RecordSpec {
            record_type: "stocktake",
            header_table: "stocktakes",
            header_alias: "st",
            // 件数と絞り込みは明細（入庫・廃棄と同じく明細の数。計数は movement を作らない。65）
            item_table: "stocktake_items",
            item_alias_prefix: "sti",
            item_fk_col: "stocktake_id",
            date_col: "completed_at",
            route_prefix: "/stocktake/records",
            status_expr: Some(
                "CASE {header_alias}.status WHEN 'completed' THEN 'active' ELSE 'in_progress' END",
            ),
            date_expr: Some(
                "DATE(COALESCE({header_alias}.completed_at, {header_alias}.started_at))",
            ),
            created_at_col: Some("started_at"),
        },
    ];

    let selected_specs: Vec<&RecordSpec> = match query.record_type.as_deref() {
        None | Some("all") => SPECS.iter().collect(),
        Some(record_type) => SPECS
            .iter()
            .filter(|spec| spec.record_type == record_type)
            .collect(),
    };

    if selected_specs.is_empty() {
        return Ok(PaginatedResult {
            items: Vec::new(),
            total_count: 0,
            page: query.page,
            per_page: query.per_page,
        });
    }

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut select_sqls = Vec::new();

    for spec in selected_specs {
        let mut where_clauses = Vec::new();
        let date_expr = spec
            .date_expr
            .map(|expr| expr.replace("{header_alias}", spec.header_alias))
            .unwrap_or_else(|| format!("{}.{}", spec.header_alias, spec.date_col));
        let status_expr = spec
            .status_expr
            .map(|expr| expr.replace("{header_alias}", spec.header_alias))
            .unwrap_or_else(|| "'active'".to_string());
        let created_at_col = spec.created_at_col.unwrap_or("created_at");

        if let Some(ref from) = query.date_from {
            where_clauses.push(format!("{} >= ?{}", date_expr, params.len() + 1));
            params.push(Box::new(from.clone()));
        }
        if let Some(ref to) = query.date_to {
            where_clauses.push(format!("{} <= ?{}", date_expr, params.len() + 1));
            params.push(Box::new(to.clone()));
        }
        if let Some(record_id) = query.record_id {
            where_clauses.push(format!("{}.id = ?{}", spec.header_alias, params.len() + 1));
            params.push(Box::new(record_id));
        }
        if let Some(department_id) = query.department_id {
            where_clauses.push(format!(
                "EXISTS (
                    SELECT 1
                    FROM {item_table} {item_alias}_dept
                    JOIN products p_dept ON p_dept.product_code = {item_alias}_dept.product_code
                    WHERE {item_alias}_dept.{item_fk_col} = {header_alias}.id
                      AND p_dept.department_id = ?{param_index}
                )",
                item_table = spec.item_table,
                item_alias = spec.item_alias_prefix,
                item_fk_col = spec.item_fk_col,
                header_alias = spec.header_alias,
                param_index = params.len() + 1
            ));
            params.push(Box::new(department_id));
        }
        if let Some(ref keyword) = query.product_keyword {
            let trimmed = keyword.trim();
            if !trimmed.is_empty() {
                where_clauses.push(format!(
                    "EXISTS (
                        SELECT 1
                        FROM {item_table} {item_alias}_kw
                        JOIN products p_kw ON p_kw.product_code = {item_alias}_kw.product_code
                        WHERE {item_alias}_kw.{item_fk_col} = {header_alias}.id
                          AND (
                            p_kw.product_code LIKE ?{param_index}
                            OR p_kw.name LIKE ?{param_index}
                            OR COALESCE(p_kw.jan_code, '') LIKE ?{param_index}
                          )
                    )",
                    item_table = spec.item_table,
                    item_alias = spec.item_alias_prefix,
                    item_fk_col = spec.item_fk_col,
                    header_alias = spec.header_alias,
                    param_index = params.len() + 1
                ));
                params.push(Box::new(format!("%{}%", trimmed)));
            }
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // filter / representative / item_count は同じ明細母集団を表す
        let item_count_expr = format!(
            "(SELECT COUNT(*)
              FROM {item_table} {item_alias}_count
              WHERE {item_alias}_count.{item_fk_col} = {header_alias}.id)",
            item_table = spec.item_table,
            item_alias = spec.item_alias_prefix,
            item_fk_col = spec.item_fk_col,
            header_alias = spec.header_alias,
        );

        select_sqls.push(format!(
            "SELECT
                '{record_type}' AS record_type,
                {header_alias}.id AS record_id,
                {date_expr} AS business_date,
                COALESCE(
                    (
                        SELECT p.name
                        FROM {item_table} {item_alias}_rep
                        JOIN products p ON p.product_code = {item_alias}_rep.product_code
                        WHERE {item_alias}_rep.{item_fk_col} = {header_alias}.id
                        ORDER BY p.product_code ASC
                        LIMIT 1
                    ),
                    '明細なし'
                ) AS representative_item,
                {item_count_expr} AS item_count,
                {status_expr} AS status,
                {header_alias}.{created_at_col} AS created_at,
                '{route_prefix}/' || {header_alias}.id AS detail_route
             FROM {header_table} {header_alias}
             {where_sql}",
            record_type = spec.record_type,
            header_alias = spec.header_alias,
            date_expr = date_expr,
            item_table = spec.item_table,
            item_alias = spec.item_alias_prefix,
            item_fk_col = spec.item_fk_col,
            item_count_expr = item_count_expr,
            status_expr = status_expr,
            created_at_col = created_at_col,
            route_prefix = spec.route_prefix,
            header_table = spec.header_table,
            where_sql = where_sql,
        ));
    }

    let union_sql = select_sqls.join(" UNION ALL ");
    let status_where_sql = match query.status.as_deref() {
        Some(status) if status != "all" => {
            let placeholder = params.len() + 1;
            params.push(Box::new(status.to_string()));
            format!("WHERE status = ?{placeholder}")
        }
        _ => String::new(),
    };
    let count_sql = format!(
        "SELECT COUNT(*) FROM ({}) records {}",
        union_sql, status_where_sql
    );
    let total_count: u32 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| row.get(0),
    )?;

    let data_sql = format!(
        "SELECT record_type, record_id, business_date, representative_item,
                item_count, status, created_at, detail_route
         FROM ({}) records
         {}
         ORDER BY business_date DESC, record_id DESC, record_type ASC
         LIMIT ?{} OFFSET ?{}",
        union_sql,
        status_where_sql,
        params.len() + 1,
        params.len() + 2,
    );
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let mut stmt = conn.prepare(&data_sql)?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
            |row| {
                Ok(InventoryRecordSummary {
                    record_type: row.get(0)?,
                    record_id: row.get(1)?,
                    business_date: row.get(2)?,
                    representative_item: row.get(3)?,
                    item_count: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    detail_route: row.get(7)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PaginatedResult {
        items: rows,
        total_count,
        page: query.page,
        per_page: query.per_page,
    })
}

/// 廃棄・破損記録の詳細を取得する。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_disposal_record_detail(
    conn: &DbConnection,
    record_id: i64,
) -> Result<DisposalRecordDetail, DbError> {
    let header = conn.query_row(
        "SELECT id, disposal_date, created_at FROM disposal_records WHERE id = ?1",
        rusqlite::params![record_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    );
    let (id, disposal_date, created_at) = match header {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(DbError::NotFound),
        Err(e) => return Err(DbError::from(e)),
    };

    let mut item_stmt = conn.prepare(
        "SELECT
            di.id,
            di.product_code,
            p.name,
            d.name,
            p.stock_unit,
            di.disposal_type,
            di.quantity,
            di.cost_price_centi,
            di.reason
         FROM disposal_items di
         JOIN products p ON p.product_code = di.product_code
         JOIN departments d ON d.id = p.department_id
         WHERE di.disposal_record_id = ?1
         ORDER BY di.id ASC",
    )?;
    let items = item_stmt
        .query_map(rusqlite::params![record_id], |row| {
            Ok(DisposalRecordDetailItem {
                id: row.get(0)?,
                product_code: row.get(1)?,
                product_name: row.get(2)?,
                department_name: row.get(3)?,
                stock_unit: parse_stock_unit(row.get(4)?)?,
                disposal_type: parse_disposal_type(row.get(5)?)?,
                quantity: row.get(6)?,
                cost_price_centi: row.get(7)?,
                reason: row.get(8)?,
                line_loss_cost_centi: String::new(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut movement_stmt = conn.prepare(
        "SELECT id, product_code, movement_type, quantity, stock_after,
                reference_type, reference_id, note, created_at
         FROM inventory_movements
         WHERE reference_type = 'disposal_record'
           AND reference_id = ?1
           AND is_voided = 0
         ORDER BY created_at ASC, id ASC",
    )?;
    let movements = movement_stmt
        .query_map(rusqlite::params![record_id], |row| {
            Ok(MovementRecord {
                id: row.get(0)?,
                product_code: row.get(1)?,
                movement_type: super::inventory_repo::parse_movement_type(row.get(2)?)?,
                quantity: row.get(3)?,
                stock_after: row.get(4)?,
                reference_type: super::inventory_repo::parse_reference_type(row.get(5)?),
                reference_id: row.get(6)?,
                source: None,
                note: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DisposalRecordDetail {
        id,
        disposal_date,
        status: "active".to_string(),
        created_at,
        items,
        total_loss_cost: 0,
        movements,
    })
}

/// idempotency_key で廃棄記録を検索
pub fn find_disposal_by_idempotency_key(
    conn: &DbConnection,
    key: &str,
) -> Result<Option<(i64, String)>, DbError> {
    let result = conn.query_row(
        "SELECT id, request_fingerprint FROM disposal_records WHERE idempotency_key = ?1",
        rusqlite::params![key],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );
    match result {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::from(e)),
    }
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::*;
    use crate::db::DbError;

    fn inventory_query(record_type: &str) -> InventoryRecordQuery {
        InventoryRecordQuery {
            record_type: Some(record_type.to_string()),
            date_from: None,
            date_to: None,
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: None,
            page: 1,
            per_page: 100,
        }
    }

    fn seed_csv_import(
        conn: &DbConnection,
        status: &str,
        settlement_date: &str,
        imported_at: &str,
        product_code: &str,
        is_voided: bool,
    ) -> i64 {
        seed_product(conn, product_code);
        conn.execute(
            "INSERT INTO csv_imports
             (filename, settlement_date, file_hash, total_items, total_amount, skipped_count, status, imported_at,
              machine_no, settlement_no)
             VALUES (?1, ?2, ?3, 1, 500, 0, ?4, ?5, '0001', (SELECT COUNT(*) + 1 FROM csv_imports))",
            rusqlite::params![
                format!("{product_code}.csv"),
                settlement_date,
                format!("hash-{product_code}"),
                status,
                imported_at,
            ],
        )
        .unwrap();
        let import_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sale_records
             (csv_import_id, product_code, sale_date, quantity, amount, source, source_line_no, is_voided, created_at)
             VALUES (?1, ?2, ?3, 1, 500, 'auto', 1, ?4, ?5)",
            rusqlite::params![import_id, product_code, settlement_date, is_voided, imported_at],
        )
        .unwrap();
        import_id
    }

    fn seed_stocktake(
        conn: &DbConnection,
        status: &str,
        started_at: &str,
        completed_at: Option<&str>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO stocktakes (started_at, completed_at, status, total_cost)
             VALUES (?1, ?2, ?3, 0)",
            rusqlite::params![started_at, completed_at, status],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_stocktake_item(conn: &DbConnection, stocktake_id: i64, product_code: &str) {
        conn.execute(
            "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
             VALUES (?1, ?2, 1, 1)",
            rusqlite::params![stocktake_id, product_code],
        )
        .unwrap();
    }

    fn seed_stocktake_movement(
        conn: &DbConnection,
        stocktake_id: i64,
        product_code: &str,
        is_voided: bool,
        created_at: &str,
    ) {
        conn.execute(
            "INSERT INTO inventory_movements
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase)
             VALUES (?1, 'stocktake', 1, 1, 'stocktake', ?2, '差異', ?3, ?4, '2026-04-01', 0)",
            rusqlite::params![product_code, stocktake_id, is_voided, created_at],
        )
        .unwrap();
    }

    fn seed_existing_four_records(conn: &DbConnection) {
        for code in ["IR-RCV-X", "IR-RET-X", "IR-MS-X", "IR-DSP-X"] {
            seed_product(conn, code);
        }
        conn.execute(
            "INSERT INTO receiving_records
             (receiving_date, note, idempotency_key, request_fingerprint, created_at)
             VALUES ('2026-08-01', '入庫', 'six-rcv', 'fp', '2026-08-01T09:00:00')",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price_centi)
             VALUES (?1, 'IR-RCV-X', 1, 100)",
            rusqlite::params![id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO return_records
             (return_type, return_date, register_processed, idempotency_key, request_fingerprint, created_at)
             VALUES ('return', '2026-08-02', 0, 'six-ret', 'fp', '2026-08-02T10:00:00')",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO return_items (return_record_id, product_code, direction, quantity)
             VALUES (?1, 'IR-RET-X', 'in', 1)",
            rusqlite::params![id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO manual_sales
             (sale_date, reason, note, idempotency_key, request_fingerprint, created_at)
             VALUES ('2026-08-03', 'other', '手動販売', 'six-ms', 'fp', '2026-08-03T11:00:00')",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO manual_sale_items (manual_sale_id, product_code, quantity, amount)
             VALUES (?1, 'IR-MS-X', 1, 500)",
            rusqlite::params![id],
        )
        .unwrap();

        let id = insert_disposal_record(
            conn,
            &NewDisposalRecord {
                disposal_date: "2026-08-04".to_string(),
                idempotency_key: "six-dsp".to_string(),
                request_fingerprint: "fp".to_string(),
            },
        )
        .unwrap();
        insert_disposal_item(
            conn,
            &NewDisposalItem {
                disposal_record_id: id,
                product_code: "IR-DSP-X".to_string(),
                disposal_type: "damage".to_string(),
                quantity: 1,
                cost_price_centi: 100,
                reason: "破損".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn test_insert_disposal_record_req204_normal() {
        // REQ-204: 廃棄・破損記録（正常INSERT）
        // FUNC-10.5: 廃棄の正常INSERT
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-DSP");

        let dr = NewDisposalRecord {
            disposal_date: "2026-04-06".to_string(),
            idempotency_key: "dsp-key-1".to_string(),
            request_fingerprint: "dsp-fp-1".to_string(),
        };
        let id = insert_disposal_record(&conn, &dr).unwrap();
        assert!(id > 0);

        let item = NewDisposalItem {
            disposal_record_id: id,
            product_code: "TEST-DSP".to_string(),
            disposal_type: "damage".to_string(),
            quantity: 2,
            cost_price_centi: 300,
            reason: "袋破れ".to_string(),
        };
        insert_disposal_item(&conn, &item).unwrap();
    }

    #[test]
    fn test_insert_disposal_req204_check_disposal_type() {
        // REQ-204: 廃棄・破損記録（CHECK制約 — disposal_type 不正値）
        // FUNC-10.5: CHECK制約 — disposal_type 不正値
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-DCHK");
        let dr = NewDisposalRecord {
            disposal_date: "2026-04-06".to_string(),
            idempotency_key: "dsp-chk-1".to_string(),
            request_fingerprint: "fp".to_string(),
        };
        let id = insert_disposal_record(&conn, &dr).unwrap();

        let insert = |disposal_type: &str| {
            conn.execute(
                "INSERT INTO disposal_items (disposal_record_id, product_code, disposal_type, quantity, cost_price_centi, reason)
                 VALUES (?1, 'TEST-DCHK', ?2, 1, 100, 'test')",
                rusqlite::params![id, disposal_type],
            )
        };
        // 列名の誤りで常に失敗して緑にならないよう、CHECK の文と正の対照を見る（Matrix T2a (i)）
        let result = insert("invalid");
        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("CHECK constraint failed")),
            "不正 disposal_type は CHECK で拒否: {result:?}"
        );
        assert!(insert("damage").is_ok(), "正しい disposal_type は通る");
    }

    #[test]
    fn test_disposal_items_req204_reason_no_check() {
        // REQ-204: 廃棄・破損記録（reason は自由記述 — CHECK制約なし）
        // disposal_items.reason は CHECK なし（自由記述）— 任意の文字列が入ること
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-DRSN");
        let dr = NewDisposalRecord {
            disposal_date: "2026-04-06".to_string(),
            idempotency_key: "dsp-rsn-1".to_string(),
            request_fingerprint: "fp".to_string(),
        };
        let id = insert_disposal_record(&conn, &dr).unwrap();

        let item = NewDisposalItem {
            disposal_record_id: id,
            product_code: "TEST-DRSN".to_string(),
            disposal_type: "other".to_string(),
            quantity: 1,
            cost_price_centi: 50,
            reason: "任意の理由テキスト — 自由記述OK".to_string(),
        };
        insert_disposal_item(&conn, &item).unwrap(); // CHECK なしなのでOK
    }

    #[test]
    fn test_insert_disposal_req204_duplicate_idempotency_key() {
        // REQ-204: 廃棄・破損記録（idempotency_key 重複 → DuplicateKey）
        let (_dir, conn) = setup_test_db();
        let dr = NewDisposalRecord {
            disposal_date: "2026-04-06".to_string(),
            idempotency_key: "dsp-dup".to_string(),
            request_fingerprint: "fp1".to_string(),
        };
        insert_disposal_record(&conn, &dr).unwrap();

        let dr2 = NewDisposalRecord {
            idempotency_key: "dsp-dup".to_string(),
            ..dr
        };
        let result = insert_disposal_record(&conn, &dr2);
        assert!(matches!(result, Err(DbError::DuplicateKey(_))));
    }

    #[test]
    fn test_find_disposal_by_idempotency_key_req204_hit() {
        // REQ-204: 廃棄・破損記録（冪等性キー検索 — ヒット → Some）
        let (_dir, conn) = setup_test_db();
        let dr = NewDisposalRecord {
            disposal_date: "2026-04-06".to_string(),
            idempotency_key: "dsp-find-1".to_string(),
            request_fingerprint: "dsp-fp-find".to_string(),
        };
        let id = insert_disposal_record(&conn, &dr).unwrap();
        let found = find_disposal_by_idempotency_key(&conn, "dsp-find-1").unwrap();
        assert_eq!(found, Some((id, "dsp-fp-find".to_string())));
    }

    #[test]
    fn test_find_disposal_by_idempotency_key_req204_miss() {
        // REQ-204: 廃棄・破損記録（冪等性キー検索 — ミス → None）
        let (_dir, conn) = setup_test_db();
        let found = find_disposal_by_idempotency_key(&conn, "nonexistent").unwrap();
        assert_eq!(found, None);
    }

    #[test]
    fn test_list_disposal_records_req204_pagination() {
        // REQ-204: 廃棄・破損記録（list ページング）
        let (_dir, conn) = setup_test_db();
        for i in 1..=5 {
            let dr = NewDisposalRecord {
                disposal_date: format!("2026-04-0{}", i),
                idempotency_key: format!("list-dsp-{}", i),
                request_fingerprint: "fp".to_string(),
            };
            insert_disposal_record(&conn, &dr).unwrap();
        }
        let q = ListQuery {
            page: 2,
            per_page: 2,
            date_from: None,
            date_to: None,
        };
        let result = list_disposal_records(&conn, &q).unwrap();
        assert_eq!(result.total_count, 5);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.page, 2);
    }

    #[test]
    fn test_list_disposal_records_req204_page_zero() {
        // REQ-204: 廃棄・破損記録（page=0 → QueryFailed）
        let (_dir, conn) = setup_test_db();
        let q = ListQuery {
            page: 0,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        assert!(matches!(
            list_disposal_records(&conn, &q),
            Err(DbError::QueryFailed(_))
        ));
    }

    #[test]
    fn test_get_disposal_record_detail_req204_includes_items_total_and_movements() {
        // REQ-204 / REQ-206: 廃棄・破損詳細は明細、ロス原価合計、関連movementを返す
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "DSP-DET-1");
        seed_product(&conn, "DSP-DET-2");

        let record_id = insert_disposal_record(
            &conn,
            &NewDisposalRecord {
                disposal_date: "2026-06-27".to_string(),
                idempotency_key: "dsp-detail".to_string(),
                request_fingerprint: "fp-detail".to_string(),
            },
        )
        .unwrap();
        insert_disposal_item(
            &conn,
            &NewDisposalItem {
                disposal_record_id: record_id,
                product_code: "DSP-DET-1".to_string(),
                disposal_type: "damage".to_string(),
                quantity: 2,
                cost_price_centi: 120,
                reason: "袋破れ".to_string(),
            },
        )
        .unwrap();
        insert_disposal_item(
            &conn,
            &NewDisposalItem {
                disposal_record_id: record_id,
                product_code: "DSP-DET-2".to_string(),
                disposal_type: "disposal".to_string(),
                quantity: 3,
                cost_price_centi: 80,
                reason: "期限切れ".to_string(),
            },
        )
        .unwrap();
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase) \
             VALUES \
             ('DSP-DET-1', 'disposal', -2, 8, 'disposal_record', ?1, '袋破れ', 0, '2026-06-27T10:00:00', '2026-04-01', 0), \
             ('DSP-DET-2', 'disposal', -3, 4, 'disposal_record', ?1, '期限切れ', 0, '2026-06-27T10:01:00', '2026-04-01', 0)",
            rusqlite::params![record_id],
        )
        .unwrap();

        let detail = get_disposal_record_detail(&conn, record_id).unwrap();

        assert_eq!(detail.id, record_id);
        assert_eq!(detail.disposal_date, "2026-06-27");
        assert_eq!(detail.status, "active");
        assert_eq!(detail.items.len(), 2);
        assert_eq!(detail.movements.len(), 2);
        assert_eq!(detail.items[0].product_code, "DSP-DET-1");
        assert_eq!(detail.items[0].product_name, "テスト商品");
        // 金額は BIZ が求める（SPEC-UNIT-D6）。repo は数量・原価・単位を返す
        assert_eq!(detail.items[0].quantity, 2);
        assert_eq!(detail.items[0].cost_price_centi, 120);
        assert_eq!(detail.items[0].stock_unit, ProductStockUnit::Pcs);
        assert_eq!(detail.items[1].quantity, 3);
        assert_eq!(detail.items[1].cost_price_centi, 80);
    }

    #[test]
    fn test_get_disposal_record_detail_req204_not_found() {
        // REQ-204 / REQ-206: 存在しない廃棄・破損詳細は NotFound
        let (_dir, conn) = setup_test_db();

        let result = get_disposal_record_detail(&conn, 999);

        assert!(matches!(result, Err(DbError::NotFound)));
    }

    #[test]
    fn test_list_inventory_records_req206_disposal_header_once_for_matching_items() {
        // REQ-206 / TRACE-D1: 明細JOINで一致しても履歴一覧はヘッダ単位で1行だけ返す
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "DSP-LIST-1");
        seed_product(&conn, "DSP-LIST-2");

        let record_id = insert_disposal_record(
            &conn,
            &NewDisposalRecord {
                disposal_date: "2026-06-27".to_string(),
                idempotency_key: "dsp-list".to_string(),
                request_fingerprint: "fp-list".to_string(),
            },
        )
        .unwrap();
        for product_code in ["DSP-LIST-1", "DSP-LIST-2"] {
            insert_disposal_item(
                &conn,
                &NewDisposalItem {
                    disposal_record_id: record_id,
                    product_code: product_code.to_string(),
                    disposal_type: "damage".to_string(),
                    quantity: 1,
                    cost_price_centi: 100,
                    reason: "破損".to_string(),
                },
            )
            .unwrap();
        }

        let query = InventoryRecordQuery {
            record_type: Some("disposal_record".to_string()),
            date_from: Some("2026-06-01".to_string()),
            date_to: Some("2026-06-30".to_string()),
            record_id: None,
            product_keyword: Some("DSP-LIST".to_string()),
            department_id: None,
            status: None,
            page: 1,
            per_page: 20,
        };
        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].record_type, "disposal_record");
        assert_eq!(result.items[0].record_id, record_id);
        assert_eq!(result.items[0].representative_item, "テスト商品");
        assert_eq!(result.items[0].item_count, 2);
        assert_eq!(
            result.items[0].detail_route,
            format!("/inventory/disposal/records/{}", record_id)
        );
    }

    #[test]
    fn test_list_inventory_records_req206_returns_all_record_types() {
        // REQ-201 / REQ-202 / REQ-203 / REQ-204 / REQ-206:
        // 入出庫履歴は入庫、返品・交換、手動販売、廃棄・破損を横断して返す
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "IR-RCV");
        seed_product(&conn, "IR-RET");
        seed_product(&conn, "IR-MS");
        seed_product(&conn, "IR-DSP");

        conn.execute(
            "INSERT INTO receiving_records \
             (receiving_date, note, idempotency_key, request_fingerprint, created_at) \
             VALUES ('2026-06-27', '入庫', 'ir-rcv', 'fp', '2026-06-27T09:00:00')",
            [],
        )
        .unwrap();
        let receiving_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price_centi) \
             VALUES (?1, 'IR-RCV', 1, 100)",
            rusqlite::params![receiving_id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO return_records \
             (return_type, return_date, register_processed, idempotency_key, request_fingerprint, created_at) \
             VALUES ('return', '2026-06-27', 0, 'ir-ret', 'fp', '2026-06-27T10:00:00')",
            [],
        )
        .unwrap();
        let return_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO return_items (return_record_id, product_code, direction, quantity) \
             VALUES (?1, 'IR-RET', 'in', 1)",
            rusqlite::params![return_id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO manual_sales \
             (sale_date, reason, note, idempotency_key, request_fingerprint, created_at) \
             VALUES ('2026-06-27', 'other', '手動販売', 'ir-ms', 'fp', '2026-06-27T11:00:00')",
            [],
        )
        .unwrap();
        let manual_sale_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO manual_sale_items (manual_sale_id, product_code, quantity, amount) \
             VALUES (?1, 'IR-MS', 1, 500)",
            rusqlite::params![manual_sale_id],
        )
        .unwrap();

        let disposal_id = insert_disposal_record(
            &conn,
            &NewDisposalRecord {
                disposal_date: "2026-06-27".to_string(),
                idempotency_key: "ir-dsp".to_string(),
                request_fingerprint: "fp".to_string(),
            },
        )
        .unwrap();
        insert_disposal_item(
            &conn,
            &NewDisposalItem {
                disposal_record_id: disposal_id,
                product_code: "IR-DSP".to_string(),
                disposal_type: "damage".to_string(),
                quantity: 1,
                cost_price_centi: 100,
                reason: "破損".to_string(),
            },
        )
        .unwrap();

        let query = InventoryRecordQuery {
            record_type: Some("all".to_string()),
            date_from: Some("2026-06-27".to_string()),
            date_to: Some("2026-06-27".to_string()),
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: Some("active".to_string()),
            page: 1,
            per_page: 20,
        };
        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 4);
        assert_eq!(result.items.len(), 4);
        let mut record_types = result
            .items
            .iter()
            .map(|item| item.record_type.as_str())
            .collect::<Vec<_>>();
        record_types.sort_unstable();
        assert_eq!(
            record_types,
            vec![
                "disposal_record",
                "manual_sale",
                "receiving_record",
                "return_record"
            ]
        );
        assert!(result.items.iter().any(
            |item| item.detail_route == format!("/inventory/receiving/records/{receiving_id}")
        ));
        assert!(result
            .items
            .iter()
            .any(|item| item.detail_route == format!("/inventory/return/records/{return_id}")));
        assert!(result
            .items
            .iter()
            .any(|item| item.detail_route
                == format!("/inventory/manual-sale/records/{manual_sale_id}")));
        assert!(result
            .items
            .iter()
            .any(|item| item.detail_route == format!("/inventory/disposal/records/{disposal_id}")));
    }

    #[test]
    fn test_list_inventory_records_req206_six_types() {
        // REQ-206 / 65 §65.10 slice 4d: 6種を業務日付降順で横断する
        let (_dir, conn) = setup_test_db();
        seed_existing_four_records(&conn);
        seed_csv_import(
            &conn,
            "completed",
            "2026-08-05",
            "2026-08-05T12:00:00",
            "IR-CSV-X",
            false,
        );
        let stocktake_id = seed_stocktake(
            &conn,
            "completed",
            "2026-08-06T08:00:00",
            Some("2026-08-06T18:00:00"),
        );
        seed_product(&conn, "IR-ST-X");
        seed_stocktake_movement(&conn, stocktake_id, "IR-ST-X", false, "2026-08-06T18:00:00");

        let result = list_inventory_records(&conn, &inventory_query("all")).unwrap();

        assert_eq!(result.total_count, 6);
        assert_eq!(
            result
                .items
                .iter()
                .map(|row| row.record_type.as_str())
                .collect::<Vec<_>>(),
            vec![
                "stocktake",
                "csv_import",
                "disposal_record",
                "manual_sale",
                "return_record",
                "receiving_record",
            ]
        );
    }

    #[test]
    fn test_list_inventory_records_req206_status_normalization() {
        // REQ-206 / 65 §65.6.1: CSV 3値・棚卸し2値を正規化する
        let (_dir, conn) = setup_test_db();
        let completed = seed_csv_import(
            &conn,
            "completed",
            "2026-08-01",
            "2026-08-01T10:00:00",
            "CSV-COMPLETED",
            false,
        );
        let partial = seed_csv_import(
            &conn,
            "completed_partial",
            "2026-08-02",
            "2026-08-02T10:00:00",
            "CSV-PARTIAL",
            false,
        );
        let rolled_back = seed_csv_import(
            &conn,
            "rolled_back",
            "2026-08-03",
            "2026-08-03T10:00:00",
            "CSV-ROLLED",
            true,
        );
        let in_progress = seed_stocktake(&conn, "in_progress", "2026-08-04T09:00:00", None);
        let stocktake_completed = seed_stocktake(
            &conn,
            "completed",
            "2026-08-05T09:00:00",
            Some("2026-08-05T18:00:00"),
        );

        let result = list_inventory_records(&conn, &inventory_query("all")).unwrap();
        let status_for = |record_type: &str, id: i64| {
            result
                .items
                .iter()
                .find(|row| row.record_type == record_type && row.record_id == id)
                .map(|row| row.status.as_str())
                .unwrap()
        };

        assert_eq!(status_for("csv_import", completed), "active");
        assert_eq!(status_for("csv_import", partial), "active");
        assert_eq!(status_for("csv_import", rolled_back), "canceled");
        assert_eq!(status_for("stocktake", in_progress), "in_progress");
        assert_eq!(status_for("stocktake", stocktake_completed), "active");
    }

    #[test]
    fn test_list_inventory_records_req206_csv_item_count_includes_voided() {
        // REQ-206 / D-d: rollback後もvoided明細を件数・代表商品へ含める
        let (_dir, conn) = setup_test_db();
        let import_id = seed_csv_import(
            &conn,
            "rolled_back",
            "2026-08-03",
            "2026-08-03T10:00:00",
            "CSV-A-VOIDED",
            true,
        );
        conn.execute(
            "UPDATE products SET name = '取消済みCSV商品' WHERE product_code = 'CSV-A-VOIDED'",
            [],
        )
        .unwrap();
        seed_product(&conn, "CSV-Z-ACTIVE");
        conn.execute(
            "UPDATE products SET name = '有効CSV商品' WHERE product_code = 'CSV-Z-ACTIVE'",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sale_records
             (csv_import_id, product_code, sale_date, quantity, amount, source, source_line_no, is_voided, created_at)
             VALUES (?1, 'CSV-Z-ACTIVE', '2026-08-03', 1, 500, 'auto', 2, 0, '2026-08-03T10:00:00')",
            rusqlite::params![import_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE csv_imports SET total_items = 2 WHERE id = ?1",
            rusqlite::params![import_id],
        )
        .unwrap();

        let result = list_inventory_records(&conn, &inventory_query("csv_import")).unwrap();
        let record = result
            .items
            .iter()
            .find(|row| row.record_id == import_id)
            .unwrap();

        assert_eq!(record.item_count, 2);
        assert_eq!(record.representative_item, "取消済みCSV商品");
        assert_ne!(record.representative_item, "明細なし");
    }

    #[test]
    fn test_list_inventory_records_req206_filter_active_excludes() {
        // REQ-206 / 65 §65.4.1: 外側 status WHERE は data/count の双方に効く
        let (_dir, conn) = setup_test_db();
        seed_existing_four_records(&conn);
        seed_csv_import(
            &conn,
            "rolled_back",
            "2026-08-05",
            "2026-08-05T10:00:00",
            "CSV-CANCELED",
            true,
        );
        seed_stocktake(&conn, "in_progress", "2026-08-06T09:00:00", None);
        let mut query = inventory_query("all");
        query.status = Some("active".to_string());

        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 4);
        assert_eq!(result.items.len(), 4);
        assert!(result.items.iter().all(|row| row.status == "active"));
    }

    #[test]
    fn test_list_inventory_records_req206_filter_in_progress() {
        // REQ-206: 進行中 filter は棚卸し行を非空で返す
        let (_dir, conn) = setup_test_db();
        let stocktake_id = seed_stocktake(&conn, "in_progress", "2026-08-06T09:00:00", None);
        let mut query = inventory_query("all");
        query.status = Some("in_progress".to_string());

        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].record_type, "stocktake");
        assert_eq!(result.items[0].record_id, stocktake_id);
    }

    #[test]
    fn test_list_inventory_records_req206_filter_canceled() {
        // REQ-206: 取消済み filter は rolled_back CSV のみを非空で返す
        let (_dir, conn) = setup_test_db();
        seed_existing_four_records(&conn);
        let csv_id = seed_csv_import(
            &conn,
            "rolled_back",
            "2026-08-05",
            "2026-08-05T10:00:00",
            "CSV-CANCELED",
            true,
        );
        seed_stocktake(
            &conn,
            "completed",
            "2026-08-06T09:00:00",
            Some("2026-08-06T18:00:00"),
        );
        let mut query = inventory_query("all");
        query.status = Some("canceled".to_string());

        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].record_type, "csv_import");
        assert_eq!(result.items[0].record_id, csv_id);
    }

    #[test]
    fn test_list_inventory_records_req206_status_gate_four_values() {
        // REQ-206 / 21-io §10.5: canceled/in_progress は gate を通り未知値だけ空ページ
        let (_dir, conn) = setup_test_db();
        seed_csv_import(
            &conn,
            "rolled_back",
            "2026-08-05",
            "2026-08-05T10:00:00",
            "CSV-GATE",
            true,
        );
        seed_stocktake(&conn, "in_progress", "2026-08-06T09:00:00", None);

        for status in ["canceled", "in_progress"] {
            let mut query = inventory_query("all");
            query.status = Some(status.to_string());
            assert_eq!(
                list_inventory_records(&conn, &query).unwrap().total_count,
                1
            );
        }
        let mut query = inventory_query("all");
        query.status = Some("unknown_status".to_string());
        assert_eq!(
            list_inventory_records(&conn, &query).unwrap().total_count,
            0
        );
    }

    #[test]
    fn test_list_inventory_records_req206_stocktake_item_count_diff_only() {
        // REQ-206 / 65: 棚卸し item_count は明細の数
        let (_dir, conn) = setup_test_db();
        let stocktake_id = seed_stocktake(
            &conn,
            "completed",
            "2026-08-06T09:00:00",
            Some("2026-08-06T18:00:00"),
        );
        for index in 0..5 {
            let code = format!("ST-ITEM-{index}");
            seed_product(&conn, &code);
            conn.execute(
                "INSERT INTO stocktake_items
                 (stocktake_id, product_code, system_stock, actual_count, valuation_cost_price_centi, counted_at)
                 VALUES (?1, ?2, 10, 9, 100, '2026-08-06T12:00:00')",
                rusqlite::params![stocktake_id, code],
            )
            .unwrap();
        }
        for (code, is_voided) in [
            ("ST-ITEM-0", false),
            ("ST-ITEM-1", false),
            ("ST-ITEM-2", true),
        ] {
            seed_stocktake_movement(&conn, stocktake_id, code, is_voided, "2026-08-06T18:00:00");
        }

        let result = list_inventory_records(&conn, &inventory_query("stocktake")).unwrap();

        // 65: 棚卸しの件数は明細の数（movement からは数えない。期待を変える。Matrix T2・R3）
        assert_eq!(result.items[0].item_count, 5);
    }

    #[test]
    fn test_list_inventory_records_req206_cross_type_id_collision() {
        // REQ-206 / D-f: 同一数値IDの他種別 movement を棚卸しへ混入させない
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "AA-STOCK-OWN");
        seed_product(&conn, "ZZ-COLLISION-GHOST");
        conn.execute(
            "UPDATE products SET department_id = 2 WHERE product_code = 'ZZ-COLLISION-GHOST'",
            [],
        )
        .unwrap();
        let disposal_id = insert_disposal_record(
            &conn,
            &NewDisposalRecord {
                disposal_date: "2026-08-06".to_string(),
                idempotency_key: "collision-disposal".to_string(),
                request_fingerprint: "fp".to_string(),
            },
        )
        .unwrap();
        let stocktake_id = seed_stocktake(
            &conn,
            "completed",
            "2026-08-06T09:00:00",
            Some("2026-08-06T18:00:00"),
        );
        assert_eq!(disposal_id, stocktake_id);
        // 65: 棚卸しの件数と絞り込みは明細（fixture を明細に。期待は変えない）
        seed_stocktake_item(&conn, stocktake_id, "AA-STOCK-OWN");
        conn.execute(
            "INSERT INTO inventory_movements
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase)
             VALUES ('ZZ-COLLISION-GHOST', 'disposal', -1, 0, 'disposal_record', ?1, '別種別', 0, '2026-08-06T17:00:00', '2026-04-01', 0)",
            rusqlite::params![disposal_id],
        )
        .unwrap();

        let result = list_inventory_records(&conn, &inventory_query("stocktake")).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].representative_item, "テスト商品");
        assert_eq!(result.items[0].item_count, 1);

        let mut own_keyword = inventory_query("stocktake");
        own_keyword.product_keyword = Some("AA-STOCK-OWN".to_string());
        assert_eq!(
            list_inventory_records(&conn, &own_keyword)
                .unwrap()
                .total_count,
            1
        );

        let mut ghost_keyword = inventory_query("stocktake");
        ghost_keyword.product_keyword = Some("ZZ-COLLISION-GHOST".to_string());
        assert_eq!(
            list_inventory_records(&conn, &ghost_keyword)
                .unwrap()
                .total_count,
            0
        );

        let mut ghost_department = inventory_query("stocktake");
        ghost_department.department_id = Some(2);
        assert_eq!(
            list_inventory_records(&conn, &ghost_department)
                .unwrap()
                .total_count,
            0
        );
    }

    #[test]
    fn test_list_inventory_records_req206_csv_business_date_settlement() {
        // REQ-206 / D-c: CSV業務日は imported_at ではなく settlement_date
        let (_dir, conn) = setup_test_db();
        seed_csv_import(
            &conn,
            "completed",
            "2026-08-05",
            "2026-08-09T10:00:00",
            "CSV-DATE",
            false,
        );
        let mut settlement_query = inventory_query("csv_import");
        settlement_query.date_from = Some("2026-08-05".to_string());
        settlement_query.date_to = Some("2026-08-05".to_string());
        assert_eq!(
            list_inventory_records(&conn, &settlement_query)
                .unwrap()
                .total_count,
            1
        );

        let mut imported_query = inventory_query("csv_import");
        imported_query.date_from = Some("2026-08-09".to_string());
        imported_query.date_to = Some("2026-08-09".to_string());
        assert_eq!(
            list_inventory_records(&conn, &imported_query)
                .unwrap()
                .total_count,
            0
        );
    }

    #[test]
    fn test_list_inventory_records_req206_stocktake_date_fallback() {
        // REQ-206 / D-c: in_progress は completed_at NULL 時に started_at 日を使う
        let (_dir, conn) = setup_test_db();
        seed_stocktake(&conn, "in_progress", "2026-08-06T09:30:00", None);
        let mut query = inventory_query("stocktake");
        query.date_from = Some("2026-08-06".to_string());
        query.date_to = Some("2026-08-06".to_string());

        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].business_date, "2026-08-06");
    }

    #[test]
    fn test_list_inventory_records_req206_stocktake_single_day_boundary() {
        // REQ-206 / D-c: DATE() により完了日18時台も単日検索へ含める
        let (_dir, conn) = setup_test_db();
        seed_stocktake(
            &conn,
            "completed",
            "2026-08-05T09:00:00",
            Some("2026-08-06T18:45:00"),
        );
        let mut query = inventory_query("stocktake");
        query.date_from = Some("2026-08-06".to_string());
        query.date_to = Some("2026-08-06".to_string());

        assert_eq!(
            list_inventory_records(&conn, &query).unwrap().total_count,
            1
        );
    }

    #[test]
    fn test_list_inventory_records_req206_filter_and_count_same_population() {
        // REQ-206: keyword判定と item_count は同じ非void stocktake movement 母集団を使う
        let (_dir, conn) = setup_test_db();
        for code in ["POP-A", "POP-B", "POP-VOID"] {
            seed_product(&conn, code);
        }
        let stocktake_id = seed_stocktake(
            &conn,
            "completed",
            "2026-08-06T09:00:00",
            Some("2026-08-06T18:00:00"),
        );
        // 65: 母集団は明細（fixture を明細に。明細を持たない POP-VOID は数えない。期待は変えない）
        seed_stocktake_item(&conn, stocktake_id, "POP-A");
        seed_stocktake_item(&conn, stocktake_id, "POP-B");
        let mut query = inventory_query("stocktake");
        query.product_keyword = Some("POP-B".to_string());

        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].item_count, 2);
    }

    #[test]
    fn test_list_inventory_records_req206_in_progress_filter_matches_items() {
        // REQ-206 / 65 の D-118 の節（棚卸しの件数と絞り込みは明細。packet S2）/ b1 Matrix T2（期待を変えた）:
        // 進行中の棚卸しも明細の商品と部門で絞り込みに当たる（前は差異 movement が無いので当たらなかった）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "INPROGRESS-TARGET");
        let stocktake_id = seed_stocktake(&conn, "in_progress", "2026-08-06T09:00:00", None);
        conn.execute(
            "INSERT INTO stocktake_items
             (stocktake_id, product_code, system_stock, actual_count, valuation_cost_price_centi, counted_at)
             VALUES (?1, 'INPROGRESS-TARGET', 10, 10, 100, '2026-08-06T12:00:00')",
            rusqlite::params![stocktake_id],
        )
        .unwrap();
        assert_eq!(
            list_inventory_records(&conn, &inventory_query("stocktake"))
                .unwrap()
                .total_count,
            1
        );

        let mut keyword_query = inventory_query("stocktake");
        keyword_query.product_keyword = Some("INPROGRESS-TARGET".to_string());
        assert_eq!(
            list_inventory_records(&conn, &keyword_query)
                .unwrap()
                .total_count,
            1
        );

        let mut department_query = inventory_query("stocktake");
        department_query.department_id = Some(1);
        assert_eq!(
            list_inventory_records(&conn, &department_query)
                .unwrap()
                .total_count,
            1
        );
    }

    #[test]
    fn test_list_inventory_records_req206_existing_four_types_unchanged() {
        // REQ-206: 既存4種は active・明細件数・filter 挙動を維持する
        let (_dir, conn) = setup_test_db();
        seed_existing_four_records(&conn);
        let mut query = inventory_query("all");
        query.department_id = Some(1);
        query.status = Some("active".to_string());

        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.total_count, 4);
        assert!(result.items.iter().all(|row| row.status == "active"));
        assert!(result.items.iter().all(|row| row.item_count == 1));
    }

    #[test]
    fn test_list_inventory_records_req206_created_at_col_mapping() {
        // REQ-206 / 21-io §10.5: 記録日時は CSV=imported_at、棚卸し=started_at
        let (_dir, conn) = setup_test_db();
        let csv_id = seed_csv_import(
            &conn,
            "completed",
            "2026-08-05",
            "2026-08-09T10:15:00",
            "CSV-CREATED",
            false,
        );
        let stocktake_id = seed_stocktake(
            &conn,
            "completed",
            "2026-08-01T08:30:00",
            Some("2026-08-06T18:45:00"),
        );
        let result = list_inventory_records(&conn, &inventory_query("all")).unwrap();
        let csv = result
            .items
            .iter()
            .find(|row| row.record_type == "csv_import" && row.record_id == csv_id)
            .unwrap();
        let stocktake = result
            .items
            .iter()
            .find(|row| row.record_type == "stocktake" && row.record_id == stocktake_id)
            .unwrap();

        assert_eq!(csv.created_at, "2026-08-09T10:15:00");
        assert_ne!(csv.created_at, csv.business_date);
        assert_eq!(stocktake.created_at, "2026-08-01T08:30:00");
        assert_ne!(stocktake.created_at, stocktake.business_date);
    }
}
