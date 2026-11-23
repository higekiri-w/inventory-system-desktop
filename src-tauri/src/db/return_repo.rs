//! 返品・交換記録のCRUD操作
//!
//! 21-io-inventory-repo.md §10.3 に基づく実装。

use super::inventory_common::{get_record_cancel, record_status, validate_and_offset, ListQuery};
use super::inventory_repo::MovementRecord;
use super::product_repo::{parse_stock_unit, ProductStockUnit};
use super::{DbConnection, DbError, PaginatedResult};

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReturnExchangeType {
    Return,
    Exchange,
}

impl ReturnExchangeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Return => "return",
            Self::Exchange => "exchange",
        }
    }
}

fn parse_return_exchange_type(value: String) -> rusqlite::Result<ReturnExchangeType> {
    match value.as_str() {
        "return" => Ok(ReturnExchangeType::Return),
        "exchange" => Ok(ReturnExchangeType::Exchange),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown return_type: {value}").into(),
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReturnDirection {
    In,
    Out,
}

impl ReturnDirection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
        }
    }
}

fn parse_return_direction(value: String) -> rusqlite::Result<ReturnDirection> {
    match value.as_str() {
        "in" => Ok(ReturnDirection::In),
        "out" => Ok(ReturnDirection::Out),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown return direction: {value}").into(),
        )),
    }
}

/// 返品記録INSERT用
///
/// 21-io-inventory-repo.md §10.3
#[derive(Debug)]
pub struct NewReturnRecord {
    pub return_type: String,
    pub return_date: String,
    pub register_processed: bool,
    pub receipt_image_path: Option<String>,
    pub note: Option<String>,
    pub idempotency_key: String,
    pub request_fingerprint: String,
}

/// 返品明細INSERT用
#[derive(Debug)]
pub struct NewReturnItem {
    pub return_record_id: i64,
    pub product_code: String,
    pub direction: String,
    pub quantity: i64,
}

/// 返品記録一覧表示用
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ReturnRecordSummary {
    pub id: i64,
    pub return_type: ReturnExchangeType,
    pub return_date: String,
    pub register_processed: bool,
    pub note: Option<String>,
    pub created_at: String,
    /// 取消の日時。有効なら None（直近の一覧の「取消済み」。D-163）
    pub canceled_at: Option<String>,
}

/// 返品・交換記録詳細の明細行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ReturnRecordDetailItem {
    pub id: i64,
    pub product_code: String,
    pub product_name: String,
    pub department_name: String,
    pub stock_unit: ProductStockUnit,
    pub direction: ReturnDirection,
    pub quantity: i64,
}

/// 返品・交換記録詳細
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ReturnRecordDetail {
    pub id: i64,
    pub return_type: ReturnExchangeType,
    pub return_date: String,
    pub register_processed: bool,
    pub receipt_image_path: Option<String>,
    pub note: Option<String>,
    pub status: String,
    /// 取消の日時（`YYYY-MM-DDTHH:MM:SS`）。有効なら None（D-163）
    pub canceled_at: Option<String>,
    /// 取消の理由。有効なら None
    pub cancel_reason: Option<String>,
    pub created_at: String,
    pub items: Vec<ReturnRecordDetailItem>,
    pub movements: Vec<MovementRecord>,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// return_records に1行INSERT
///
/// 21-io-inventory-repo.md §10.3
pub fn insert_return_record(conn: &DbConnection, record: &NewReturnRecord) -> Result<i64, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO return_records (return_type, return_date, register_processed, receipt_image_path, note, idempotency_key, request_fingerprint, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            record.return_type,
            record.return_date,
            record.register_processed,
            record.receipt_image_path,
            record.note,
            record.idempotency_key,
            record.request_fingerprint,
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// return_items に1行INSERT
pub fn insert_return_item(conn: &DbConnection, item: &NewReturnItem) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO return_items (return_record_id, product_code, direction, quantity)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            item.return_record_id,
            item.product_code,
            item.direction,
            item.quantity,
        ],
    )?;
    Ok(())
}

/// 返品記録一覧をページング取得
pub fn list_return_records(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<ReturnRecordSummary>, DbError> {
    let (limit, offset) = validate_and_offset(query)?;

    let mut where_clauses = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref from) = query.date_from {
        where_clauses.push(format!("return_date >= ?{}", params.len() + 1));
        params.push(Box::new(from.clone()));
    }
    if let Some(ref to) = query.date_to {
        where_clauses.push(format!("return_date <= ?{}", params.len() + 1));
        params.push(Box::new(to.clone()));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM return_records {}", where_sql);
    let total_count: u32 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| row.get(0),
    )?;

    let data_sql = format!(
        "SELECT id, return_type, return_date, register_processed, note, created_at, canceled_at \
         FROM return_records {} \
         ORDER BY return_date DESC, id DESC \
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
                Ok(ReturnRecordSummary {
                    id: row.get(0)?,
                    return_type: parse_return_exchange_type(row.get(1)?)?,
                    return_date: row.get(2)?,
                    register_processed: row.get(3)?,
                    note: row.get(4)?,
                    created_at: row.get(5)?,
                    canceled_at: row.get(6)?,
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

/// 返品・交換記録の詳細を取得する。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_return_record_detail(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ReturnRecordDetail, DbError> {
    let header = conn.query_row(
        "SELECT id, return_type, return_date, register_processed, receipt_image_path, note, created_at
         FROM return_records
         WHERE id = ?1",
        rusqlite::params![record_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
            ))
        },
    );
    let (id, return_type, return_date, register_processed, receipt_image_path, note, created_at) =
        match header {
            Ok(row) => row,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(DbError::NotFound),
            Err(e) => return Err(DbError::from(e)),
        };
    let return_type = parse_return_exchange_type(return_type).map_err(DbError::from)?;

    let (canceled_at, cancel_reason) = get_record_cancel(conn, "return_records", record_id)?;

    let mut item_stmt = conn.prepare(
        "SELECT
            ri.id,
            ri.product_code,
            p.name,
            d.name,
            p.stock_unit,
            ri.direction,
            ri.quantity
         FROM return_items ri
         JOIN products p ON p.product_code = ri.product_code
         JOIN departments d ON d.id = p.department_id
         WHERE ri.return_record_id = ?1
         ORDER BY ri.id ASC",
    )?;
    let items = item_stmt
        .query_map(rusqlite::params![record_id], |row| {
            Ok(ReturnRecordDetailItem {
                id: row.get(0)?,
                product_code: row.get(1)?,
                product_name: row.get(2)?,
                department_name: row.get(3)?,
                stock_unit: parse_stock_unit(row.get(4)?)?,
                direction: parse_return_direction(row.get(5)?)?,
                quantity: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut movement_stmt = conn.prepare(
        "SELECT id, product_code, movement_type, quantity, stock_after,
                reference_type, reference_id, note, created_at
         FROM inventory_movements
         WHERE reference_type = 'return_record'
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

    Ok(ReturnRecordDetail {
        id,
        return_type,
        return_date,
        register_processed,
        receipt_image_path,
        note,
        status: record_status(canceled_at.as_deref()),
        canceled_at,
        cancel_reason,
        created_at,
        items,
        movements,
    })
}

/// idempotency_key で返品記録を検索
pub fn find_return_by_idempotency_key(
    conn: &DbConnection,
    key: &str,
) -> Result<Option<(i64, String)>, DbError> {
    let result = conn.query_row(
        "SELECT id, request_fingerprint FROM return_records WHERE idempotency_key = ?1",
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

    #[test]
    fn test_insert_return_record_req202_normal() {
        // REQ-202: 返品・交換記録（ヘッダ+明細の正常INSERT）
        // FUNC-10.3: 返品ヘッダ+明細の正常INSERT
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-RET");

        let record = NewReturnRecord {
            return_type: "return".to_string(),
            return_date: "2026-04-06".to_string(),
            register_processed: false,
            receipt_image_path: None,
            note: Some("テスト返品".to_string()),
            idempotency_key: "ret-key-1".to_string(),
            request_fingerprint: "ret-fp-1".to_string(),
        };
        let record_id = insert_return_record(&conn, &record).unwrap();
        assert!(record_id > 0);

        let item = NewReturnItem {
            return_record_id: record_id,
            product_code: "TEST-RET".to_string(),
            direction: "in".to_string(),
            quantity: 2,
        };
        insert_return_item(&conn, &item).unwrap();
    }

    #[test]
    fn test_insert_return_req202_check_return_type() {
        // REQ-202: 返品・交換記録（CHECK制約 — return_type 不正値）
        // FUNC-10.3: CHECK制約 — return_type 不正値
        let (_dir, conn) = setup_test_db();
        let result = conn.execute(
            "INSERT INTO return_records (return_type, return_date, register_processed, idempotency_key, request_fingerprint, created_at)
             VALUES ('invalid', '2026-04-06', 0, 'chk-1', 'fp', '2026-04-06T00:00:00')",
            [],
        );
        assert!(result.is_err(), "不正 return_type は CHECK で拒否");
    }

    #[test]
    fn test_insert_return_req202_check_direction() {
        // REQ-202: 返品・交換記録（CHECK制約 — direction 不正値）
        // FUNC-10.3: CHECK制約 — direction 不正値
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-DIR");
        let r = NewReturnRecord {
            return_type: "return".to_string(),
            return_date: "2026-04-06".to_string(),
            register_processed: false,
            receipt_image_path: None,
            note: None,
            idempotency_key: "dir-chk-1".to_string(),
            request_fingerprint: "fp".to_string(),
        };
        let rid = insert_return_record(&conn, &r).unwrap();

        let result = conn.execute(
            "INSERT INTO return_items (return_record_id, product_code, direction, quantity)
             VALUES (?1, 'TEST-DIR', 'invalid', 1)",
            rusqlite::params![rid],
        );
        assert!(result.is_err(), "不正 direction は CHECK で拒否");
    }

    #[test]
    fn test_insert_return_req202_duplicate_idempotency_key() {
        // REQ-202: 返品・交換記録（idempotency_key 重複 → DuplicateKey）
        let (_dir, conn) = setup_test_db();
        let r1 = NewReturnRecord {
            return_type: "return".to_string(),
            return_date: "2026-04-06".to_string(),
            register_processed: false,
            receipt_image_path: None,
            note: None,
            idempotency_key: "ret-dup".to_string(),
            request_fingerprint: "fp1".to_string(),
        };
        insert_return_record(&conn, &r1).unwrap();

        let r2 = NewReturnRecord {
            idempotency_key: "ret-dup".to_string(),
            ..r1
        };
        let result = insert_return_record(&conn, &r2);
        assert!(matches!(result, Err(DbError::DuplicateKey(_))));
    }

    #[test]
    fn test_find_return_by_idempotency_key_req202_hit() {
        // REQ-202: 返品・交換記録（冪等性キー検索 — ヒット → Some）
        let (_dir, conn) = setup_test_db();
        let r = NewReturnRecord {
            return_type: "exchange".to_string(),
            return_date: "2026-04-06".to_string(),
            register_processed: true,
            receipt_image_path: None,
            note: None,
            idempotency_key: "ret-find-1".to_string(),
            request_fingerprint: "ret-fp-find".to_string(),
        };
        let id = insert_return_record(&conn, &r).unwrap();
        let found = find_return_by_idempotency_key(&conn, "ret-find-1").unwrap();
        assert_eq!(found, Some((id, "ret-fp-find".to_string())));
    }

    #[test]
    fn test_find_return_by_idempotency_key_req202_miss() {
        // REQ-202: 返品・交換記録（冪等性キー検索 — ミス → None）
        let (_dir, conn) = setup_test_db();
        let found = find_return_by_idempotency_key(&conn, "nonexistent").unwrap();
        assert_eq!(found, None);
    }

    #[test]
    fn test_list_return_records_req202_pagination() {
        // REQ-202: 返品・交換記録（list ページング）
        let (_dir, conn) = setup_test_db();
        for i in 1..=4 {
            let r = NewReturnRecord {
                return_type: "return".to_string(),
                return_date: format!("2026-04-0{}", i),
                register_processed: false,
                receipt_image_path: None,
                note: None,
                idempotency_key: format!("list-ret-{}", i),
                request_fingerprint: "fp".to_string(),
            };
            insert_return_record(&conn, &r).unwrap();
        }
        let q = ListQuery {
            page: 1,
            per_page: 2,
            date_from: None,
            date_to: None,
        };
        let result = list_return_records(&conn, &q).unwrap();
        assert_eq!(result.total_count, 4);
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn test_list_return_records_req202_page_zero() {
        // REQ-202: 返品・交換記録（page=0 → QueryFailed）
        let (_dir, conn) = setup_test_db();
        let q = ListQuery {
            page: 0,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        assert!(matches!(
            list_return_records(&conn, &q),
            Err(DbError::QueryFailed(_))
        ));
    }

    #[test]
    fn test_get_return_record_detail_req202_req206_includes_items_receipt_and_movements() {
        // REQ-202 / REQ-206: 返品・交換詳細は明細、レシートパス、関連movementを返す
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "RET-DET");

        let record_id = insert_return_record(
            &conn,
            &NewReturnRecord {
                return_type: "exchange".to_string(),
                return_date: "2026-06-27".to_string(),
                register_processed: false,
                receipt_image_path: Some("receipts/ret-detail.png".to_string()),
                note: Some("サイズ交換".to_string()),
                idempotency_key: "ret-detail".to_string(),
                request_fingerprint: "fp-detail".to_string(),
            },
        )
        .unwrap();
        insert_return_item(
            &conn,
            &NewReturnItem {
                return_record_id: record_id,
                product_code: "RET-DET".to_string(),
                direction: "in".to_string(),
                quantity: 1,
            },
        )
        .unwrap();
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase) \
             VALUES ('RET-DET', 'return', 1, 6, 'return_record', ?1, '返品', 0, '2026-06-27T10:00:00', '2026-04-01', 0)",
            rusqlite::params![record_id],
        )
        .unwrap();

        let detail = get_return_record_detail(&conn, record_id).unwrap();

        assert_eq!(detail.id, record_id);
        assert_eq!(detail.return_type, ReturnExchangeType::Exchange);
        assert_eq!(detail.return_date, "2026-06-27");
        assert!(!detail.register_processed);
        assert_eq!(
            detail.receipt_image_path,
            Some("receipts/ret-detail.png".to_string())
        );
        assert_eq!(detail.note, Some("サイズ交換".to_string()));
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.items[0].product_code, "RET-DET");
        assert_eq!(detail.items[0].direction, ReturnDirection::In);
        assert_eq!(detail.movements.len(), 1);
        assert_eq!(
            detail.movements[0].reference_type,
            Some(super::super::inventory_repo::ReferenceType::ReturnRecord)
        );
    }
}
