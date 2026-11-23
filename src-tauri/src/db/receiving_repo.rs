//! 入庫記録のCRUD操作
//!
//! 21-io-inventory-repo.md §10.2 に基づく実装。

use super::inventory_common::{get_record_cancel, record_status, validate_and_offset, ListQuery};
use super::inventory_repo::MovementRecord;
use super::product_repo::{parse_stock_unit, ProductStockUnit};
use super::{DbConnection, DbError, PaginatedResult};

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 入庫記録INSERT用
///
/// 21-io-inventory-repo.md §10.2
#[derive(Debug)]
pub struct NewReceivingRecord {
    pub supplier_id: Option<i64>,
    pub receiving_date: String,
    pub note: Option<String>,
    pub idempotency_key: String,
    pub request_fingerprint: String,
}

/// 入庫明細INSERT用
#[derive(Debug)]
pub struct NewReceivingItem {
    pub receiving_record_id: i64,
    pub product_code: String,
    pub quantity: i64,
    pub cost_price_centi: i64,
}

/// 入庫記録一覧表示用（supplier_name JOIN済み）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ReceivingRecordWithSupplier {
    pub id: i64,
    pub supplier_id: Option<i64>,
    pub supplier_name: Option<String>,
    pub receiving_date: String,
    pub note: Option<String>,
    pub created_at: String,
    /// 取消の日時。有効なら None（直近の一覧の「取消済み」。D-163）
    pub canceled_at: Option<String>,
}

/// 入庫記録詳細の明細行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ReceivingRecordDetailItem {
    pub id: i64,
    pub product_code: String,
    pub product_name: String,
    pub department_name: String,
    pub stock_unit: ProductStockUnit,
    pub quantity: i64,
    pub cost_price_centi: i64,
    /// 行の金額（1/100 円の整数の十進の文字列。SPEC-UNIT-D6）
    // repo は空で返し、BIZ が必ず埋める
    pub line_cost_centi: String,
}

/// 入庫記録詳細
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ReceivingRecordDetail {
    pub id: i64,
    pub receiving_date: String,
    pub supplier_id: Option<i64>,
    pub supplier_name: Option<String>,
    pub note: Option<String>,
    pub status: String,
    /// 取消の日時（`YYYY-MM-DDTHH:MM:SS`）。有効なら None（D-163）
    pub canceled_at: Option<String>,
    /// 取消の理由。有効なら None
    pub cancel_reason: Option<String>,
    pub created_at: String,
    pub items: Vec<ReceivingRecordDetailItem>,
    /// 原価合計（円。行の金額の和を円未満で四捨五入した値。SPEC-UNIT-D6）
    // repo は 0 で返し、BIZ が必ず埋める
    pub total_cost: i64,
    pub movements: Vec<MovementRecord>,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// receiving_records に1行INSERT
///
/// 21-io-inventory-repo.md §10.2
pub fn insert_receiving_record(
    conn: &DbConnection,
    record: &NewReceivingRecord,
) -> Result<i64, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO receiving_records (supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            record.supplier_id,
            record.receiving_date,
            record.note,
            record.idempotency_key,
            record.request_fingerprint,
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// receiving_items に1行INSERT
pub fn insert_receiving_item(conn: &DbConnection, item: &NewReceivingItem) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO receiving_items (receiving_record_id, product_code, quantity, cost_price_centi)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            item.receiving_record_id,
            item.product_code,
            item.quantity,
            item.cost_price_centi,
        ],
    )?;
    Ok(())
}

/// 入庫記録一覧をページング取得（取引先名JOIN）
///
/// ORDER BY receiving_date DESC, id DESC で安定ソート
pub fn list_receiving_records(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<ReceivingRecordWithSupplier>, DbError> {
    let (limit, offset) = validate_and_offset(query)?;

    let mut where_clauses = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref from) = query.date_from {
        where_clauses.push(format!("rr.receiving_date >= ?{}", params.len() + 1));
        params.push(Box::new(from.clone()));
    }
    if let Some(ref to) = query.date_to {
        where_clauses.push(format!("rr.receiving_date <= ?{}", params.len() + 1));
        params.push(Box::new(to.clone()));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // total_count
    let count_sql = format!("SELECT COUNT(*) FROM receiving_records rr {}", where_sql);
    let total_count: u32 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| row.get(0),
    )?;

    // data
    let data_sql = format!(
        "SELECT rr.id, rr.supplier_id, s.name, rr.receiving_date, rr.note, rr.created_at, rr.canceled_at \
         FROM receiving_records rr \
         LEFT JOIN suppliers s ON rr.supplier_id = s.id \
         {} \
         ORDER BY rr.receiving_date DESC, rr.id DESC \
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
                Ok(ReceivingRecordWithSupplier {
                    id: row.get(0)?,
                    supplier_id: row.get(1)?,
                    supplier_name: row.get(2)?,
                    receiving_date: row.get(3)?,
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

/// 入庫記録の詳細を取得する。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_receiving_record_detail(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ReceivingRecordDetail, DbError> {
    let header = conn.query_row(
        "SELECT rr.id, rr.receiving_date, rr.supplier_id, s.name, rr.note, rr.created_at
         FROM receiving_records rr
         LEFT JOIN suppliers s ON s.id = rr.supplier_id
         WHERE rr.id = ?1",
        rusqlite::params![record_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
            ))
        },
    );
    let (id, receiving_date, supplier_id, supplier_name, note, created_at) = match header {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(DbError::NotFound),
        Err(e) => return Err(DbError::from(e)),
    };

    let (canceled_at, cancel_reason) = get_record_cancel(conn, "receiving_records", record_id)?;

    let mut item_stmt = conn.prepare(
        "SELECT
            ri.id,
            ri.product_code,
            p.name,
            d.name,
            p.stock_unit,
            ri.quantity,
            ri.cost_price_centi
         FROM receiving_items ri
         JOIN products p ON p.product_code = ri.product_code
         JOIN departments d ON d.id = p.department_id
         WHERE ri.receiving_record_id = ?1
         ORDER BY ri.id ASC",
    )?;
    let items = item_stmt
        .query_map(rusqlite::params![record_id], |row| {
            Ok(ReceivingRecordDetailItem {
                id: row.get(0)?,
                product_code: row.get(1)?,
                product_name: row.get(2)?,
                department_name: row.get(3)?,
                stock_unit: parse_stock_unit(row.get(4)?)?,
                quantity: row.get(5)?,
                cost_price_centi: row.get(6)?,
                line_cost_centi: String::new(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut movement_stmt = conn.prepare(
        "SELECT id, product_code, movement_type, quantity, stock_after,
                reference_type, reference_id, note, created_at
         FROM inventory_movements
         WHERE reference_type = 'receiving_record'
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

    Ok(ReceivingRecordDetail {
        id,
        receiving_date,
        supplier_id,
        supplier_name,
        note,
        status: record_status(canceled_at.as_deref()),
        canceled_at,
        cancel_reason,
        created_at,
        items,
        total_cost: 0,
        movements,
    })
}

/// idempotency_key で入庫記録を検索（冪等性チェック用）
///
/// 戻り値: Some((record_id, request_fingerprint)) または None
pub fn find_receiving_by_idempotency_key(
    conn: &DbConnection,
    key: &str,
) -> Result<Option<(i64, String)>, DbError> {
    let result = conn.query_row(
        "SELECT id, request_fingerprint FROM receiving_records WHERE idempotency_key = ?1",
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
    fn test_insert_receiving_record_req201_normal() {
        // REQ-201: 入庫記録（ヘッダ+明細の正常INSERT）
        // FUNC-10.2: 入庫ヘッダ+明細の正常INSERT
        let (_dir, conn) = setup_test_db();
        let supplier_id = seed_supplier(&conn);
        seed_product(&conn, "TEST-RCV");

        let record = NewReceivingRecord {
            supplier_id: Some(supplier_id),
            receiving_date: "2026-04-06".to_string(),
            note: Some("テスト入庫".to_string()),
            idempotency_key: "rcv-key-1".to_string(),
            request_fingerprint: "rcv-fp-1".to_string(),
        };
        let record_id = insert_receiving_record(&conn, &record).unwrap();
        assert!(record_id > 0);

        let item = NewReceivingItem {
            receiving_record_id: record_id,
            product_code: "TEST-RCV".to_string(),
            quantity: 10,
            cost_price_centi: 300,
        };
        insert_receiving_item(&conn, &item).unwrap();

        // 明細が保存されていること
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM receiving_items WHERE receiving_record_id = ?1",
                rusqlite::params![record_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_receiving_req201_fk_violation_supplier() {
        // REQ-201: 入庫記録（不正 supplier_id → ForeignKeyViolation）
        // FUNC-10.2: 不正 supplier_id → ForeignKeyViolation
        let (_dir, conn) = setup_test_db();
        let record = NewReceivingRecord {
            supplier_id: Some(9999),
            receiving_date: "2026-04-06".to_string(),
            note: None,
            idempotency_key: "rcv-fk-1".to_string(),
            request_fingerprint: "fp".to_string(),
        };
        let result = insert_receiving_record(&conn, &record);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "{:?}",
            result
        );
    }

    #[test]
    fn test_insert_receiving_item_req201_fk_violation_product() {
        // REQ-201: 入庫記録（不正 product_code → ForeignKeyViolation）
        // FUNC-10.2: 不正 product_code → ForeignKeyViolation
        let (_dir, conn) = setup_test_db();
        let record = NewReceivingRecord {
            supplier_id: None,
            receiving_date: "2026-04-06".to_string(),
            note: None,
            idempotency_key: "rcv-fk-2".to_string(),
            request_fingerprint: "fp".to_string(),
        };
        let record_id = insert_receiving_record(&conn, &record).unwrap();

        let item = NewReceivingItem {
            receiving_record_id: record_id,
            product_code: "NONEXISTENT".to_string(),
            quantity: 1,
            cost_price_centi: 100,
        };
        let result = insert_receiving_item(&conn, &item);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "{:?}",
            result
        );
    }

    #[test]
    fn test_insert_receiving_item_req201_fk_violation_record() {
        // REQ-201: 入庫記録（不正 receiving_record_id → ForeignKeyViolation）
        // FUNC-10.2: 不正 receiving_record_id → ForeignKeyViolation
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-FKR");
        let item = NewReceivingItem {
            receiving_record_id: 9999,
            product_code: "TEST-FKR".to_string(),
            quantity: 1,
            cost_price_centi: 100,
        };
        let result = insert_receiving_item(&conn, &item);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "{:?}",
            result
        );
    }

    #[test]
    fn test_insert_receiving_req201_duplicate_idempotency_key() {
        // REQ-201: 入庫記録（idempotency_key 重複 → DuplicateKey）
        // FUNC-10.2: idempotency_key 重複 → DuplicateKey
        let (_dir, conn) = setup_test_db();
        let r = NewReceivingRecord {
            supplier_id: None,
            receiving_date: "2026-04-06".to_string(),
            note: None,
            idempotency_key: "dup-key".to_string(),
            request_fingerprint: "fp1".to_string(),
        };
        insert_receiving_record(&conn, &r).unwrap();

        let r2 = NewReceivingRecord {
            supplier_id: None,
            receiving_date: "2026-04-07".to_string(),
            note: None,
            idempotency_key: "dup-key".to_string(),
            request_fingerprint: "fp2".to_string(),
        };
        let result = insert_receiving_record(&conn, &r2);
        assert!(
            matches!(result, Err(DbError::DuplicateKey(_))),
            "{:?}",
            result
        );
    }

    #[test]
    fn test_list_receiving_records_req201_pagination() {
        // REQ-201: 入庫記録（list ページング + supplier_name JOIN）
        // FUNC-10.2: list ページング + supplier_name JOIN
        let (_dir, conn) = setup_test_db();
        let sid = seed_supplier(&conn);

        for i in 1..=5 {
            let r = NewReceivingRecord {
                supplier_id: Some(sid),
                receiving_date: format!("2026-04-0{}", i),
                note: None,
                idempotency_key: format!("list-rcv-{}", i),
                request_fingerprint: "fp".to_string(),
            };
            insert_receiving_record(&conn, &r).unwrap();
        }

        let q = ListQuery {
            page: 1,
            per_page: 3,
            date_from: None,
            date_to: None,
        };
        let result = list_receiving_records(&conn, &q).unwrap();
        assert_eq!(result.total_count, 5);
        assert_eq!(result.items.len(), 3);
        assert_eq!(
            result.items[0].supplier_name,
            Some("テスト取引先".to_string())
        );
        // DESC順: 最新が先
        assert!(result.items[0].receiving_date >= result.items[1].receiving_date);
    }

    #[test]
    fn test_list_receiving_records_req201_date_filter() {
        // REQ-201: 入庫記録（日付フィルタ）
        // FUNC-10.2: 日付フィルタ
        let (_dir, conn) = setup_test_db();
        for i in 1..=3 {
            let r = NewReceivingRecord {
                supplier_id: None,
                receiving_date: format!("2026-04-0{}", i),
                note: None,
                idempotency_key: format!("df-rcv-{}", i),
                request_fingerprint: "fp".to_string(),
            };
            insert_receiving_record(&conn, &r).unwrap();
        }

        let q = ListQuery {
            page: 1,
            per_page: 50,
            date_from: Some("2026-04-02".to_string()),
            date_to: Some("2026-04-02".to_string()),
        };
        let result = list_receiving_records(&conn, &q).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.items[0].receiving_date, "2026-04-02");
    }

    #[test]
    fn test_list_receiving_records_req201_page_zero() {
        // REQ-201: 入庫記録（page=0 → QueryFailed）
        // FUNC-10.2: page=0 → QueryFailed
        let (_dir, conn) = setup_test_db();
        let q = ListQuery {
            page: 0,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let result = list_receiving_records(&conn, &q);
        assert!(matches!(result, Err(DbError::QueryFailed(_))));
    }

    #[test]
    fn test_find_receiving_by_idempotency_key_req201_hit() {
        // REQ-201: 入庫記録（冪等性キー検索 — ヒット → Some((id, fingerprint))）
        // FUNC-10.7: ヒット → Some((id, fingerprint))
        let (_dir, conn) = setup_test_db();
        let r = NewReceivingRecord {
            supplier_id: None,
            receiving_date: "2026-04-06".to_string(),
            note: None,
            idempotency_key: "find-rcv-1".to_string(),
            request_fingerprint: "fp-find-1".to_string(),
        };
        let id = insert_receiving_record(&conn, &r).unwrap();
        let found = find_receiving_by_idempotency_key(&conn, "find-rcv-1").unwrap();
        assert_eq!(found, Some((id, "fp-find-1".to_string())));
    }

    #[test]
    fn test_find_receiving_by_idempotency_key_req201_miss() {
        // REQ-201: 入庫記録（冪等性キー検索 — ミス → None）
        // FUNC-10.7: ミス → None
        let (_dir, conn) = setup_test_db();
        let found = find_receiving_by_idempotency_key(&conn, "nonexistent").unwrap();
        assert_eq!(found, None);
    }

    #[test]
    fn test_get_receiving_record_detail_req201_req206_includes_items_total_and_movements() {
        // REQ-201 / REQ-206: 入庫詳細は明細、原価合計、関連movementを返す
        let (_dir, conn) = setup_test_db();
        let supplier_id = seed_supplier(&conn);
        seed_product(&conn, "RCV-DET");

        let record_id = insert_receiving_record(
            &conn,
            &NewReceivingRecord {
                supplier_id: Some(supplier_id),
                receiving_date: "2026-06-27".to_string(),
                note: Some("納品書あり".to_string()),
                idempotency_key: "rcv-detail".to_string(),
                request_fingerprint: "fp-detail".to_string(),
            },
        )
        .unwrap();
        insert_receiving_item(
            &conn,
            &NewReceivingItem {
                receiving_record_id: record_id,
                product_code: "RCV-DET".to_string(),
                quantity: 4,
                cost_price_centi: 250,
            },
        )
        .unwrap();
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase) \
             VALUES ('RCV-DET', 'receiving', 4, 14, 'receiving_record', ?1, '入庫', 0, '2026-06-27T09:00:00', '2026-04-01', 0)",
            rusqlite::params![record_id],
        )
        .unwrap();

        let detail = get_receiving_record_detail(&conn, record_id).unwrap();

        assert_eq!(detail.id, record_id);
        assert_eq!(detail.receiving_date, "2026-06-27");
        assert_eq!(detail.supplier_name, Some("テスト取引先".to_string()));
        assert_eq!(detail.note, Some("納品書あり".to_string()));
        assert_eq!(detail.status, "active");
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.items[0].product_code, "RCV-DET");
        // 金額は BIZ が求める（SPEC-UNIT-D6）。repo は数量・原価・単位を返す
        assert_eq!(detail.items[0].quantity, 4);
        assert_eq!(detail.items[0].cost_price_centi, 250);
        assert_eq!(detail.items[0].stock_unit, ProductStockUnit::Pcs);
        assert_eq!(detail.movements.len(), 1);
        assert_eq!(
            detail.movements[0].reference_type,
            Some(super::super::inventory_repo::ReferenceType::ReceivingRecord)
        );
    }
}
