//! 手動販売出庫のCRUD操作
//!
//! 21-io-inventory-repo.md §10.4 に基づく実装。

use super::inventory_common::{get_record_cancel, record_status};
use super::inventory_repo::MovementRecord;
use super::product_repo::{parse_stock_unit, ProductStockUnit};
use super::{DbConnection, DbError};

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ManualSaleReason {
    PluUnregistered,
    Other,
}

impl ManualSaleReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PluUnregistered => "plu_unregistered",
            Self::Other => "other",
        }
    }
}

fn parse_manual_sale_reason(value: String) -> rusqlite::Result<ManualSaleReason> {
    match value.as_str() {
        "plu_unregistered" => Ok(ManualSaleReason::PluUnregistered),
        "other" => Ok(ManualSaleReason::Other),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown manual sale reason: {value}").into(),
        )),
    }
}

/// 手動販売INSERT用
///
/// 21-io-inventory-repo.md §10.4
#[derive(Debug)]
pub struct NewManualSale {
    pub sale_date: String,
    pub reason: String,
    pub note: Option<String>,
    pub idempotency_key: String,
    pub request_fingerprint: String,
}

/// 手動販売明細INSERT用
#[derive(Debug)]
pub struct NewManualSaleItem {
    pub manual_sale_id: i64,
    pub product_code: String,
    pub quantity: i64,
    pub amount: i64,
}

/// 手動販売記録詳細の明細行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ManualSaleRecordDetailItem {
    pub id: i64,
    pub product_code: String,
    pub product_name: String,
    pub department_name: String,
    pub stock_unit: ProductStockUnit,
    pub quantity: i64,
    pub amount: i64,
}

/// 手動販売記録詳細
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ManualSaleRecordDetail {
    pub id: i64,
    pub sale_date: String,
    pub reason: ManualSaleReason,
    pub note: Option<String>,
    pub status: String,
    /// 取消の日時（`YYYY-MM-DDTHH:MM:SS`）。有効なら None（D-163）
    pub canceled_at: Option<String>,
    /// 取消の理由。有効なら None
    pub cancel_reason: Option<String>,
    pub created_at: String,
    pub items: Vec<ManualSaleRecordDetailItem>,
    pub total_amount: i64,
    pub movements: Vec<MovementRecord>,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// manual_sales に1行INSERT
///
/// 21-io-inventory-repo.md §10.4
pub fn insert_manual_sale(conn: &DbConnection, record: &NewManualSale) -> Result<i64, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO manual_sales (sale_date, reason, note, idempotency_key, request_fingerprint, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            record.sale_date,
            record.reason,
            record.note,
            record.idempotency_key,
            record.request_fingerprint,
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// manual_sale_items に1行INSERT
pub fn insert_manual_sale_item(
    conn: &DbConnection,
    item: &NewManualSaleItem,
) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO manual_sale_items (manual_sale_id, product_code, quantity, amount)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            item.manual_sale_id,
            item.product_code,
            item.quantity,
            item.amount,
        ],
    )?;
    Ok(())
}

/// 手動販売の明細の数を返す（記録の取消で、結んだ売上の行の数と比べる）
///
/// 21-io-inventory-repo.md §10.5b
pub fn count_manual_sale_items(conn: &DbConnection, manual_sale_id: i64) -> Result<u64, DbError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM manual_sale_items WHERE manual_sale_id = ?1",
        rusqlite::params![manual_sale_id],
        |row| row.get(0),
    )?;
    Ok(count as u64)
}

/// 手動販売記録の詳細を取得する。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_manual_sale_record_detail(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ManualSaleRecordDetail, DbError> {
    let header = conn.query_row(
        "SELECT id, sale_date, reason, note, created_at
         FROM manual_sales
         WHERE id = ?1",
        rusqlite::params![record_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        },
    );
    let (id, sale_date, reason, note, created_at) = match header {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(DbError::NotFound),
        Err(e) => return Err(DbError::from(e)),
    };
    let reason = parse_manual_sale_reason(reason).map_err(DbError::from)?;

    let (canceled_at, cancel_reason) = get_record_cancel(conn, "manual_sales", record_id)?;

    let mut item_stmt = conn.prepare(
        "SELECT
            msi.id,
            msi.product_code,
            p.name,
            d.name,
            p.stock_unit,
            msi.quantity,
            msi.amount
         FROM manual_sale_items msi
         JOIN products p ON p.product_code = msi.product_code
         JOIN departments d ON d.id = p.department_id
         WHERE msi.manual_sale_id = ?1
         ORDER BY msi.id ASC",
    )?;
    let items = item_stmt
        .query_map(rusqlite::params![record_id], |row| {
            Ok(ManualSaleRecordDetailItem {
                id: row.get(0)?,
                product_code: row.get(1)?,
                product_name: row.get(2)?,
                department_name: row.get(3)?,
                stock_unit: parse_stock_unit(row.get(4)?)?,
                quantity: row.get(5)?,
                amount: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let total_amount = items.iter().map(|item| item.amount).sum();

    let mut movement_stmt = conn.prepare(
        "SELECT id, product_code, movement_type, quantity, stock_after,
                reference_type, reference_id, note, created_at
         FROM inventory_movements
         WHERE reference_type = 'manual_sale'
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

    Ok(ManualSaleRecordDetail {
        id,
        sale_date,
        reason,
        note,
        status: record_status(canceled_at.as_deref()),
        canceled_at,
        cancel_reason,
        created_at,
        items,
        total_amount,
        movements,
    })
}

/// idempotency_key で手動販売を検索
pub fn find_manual_sale_by_idempotency_key(
    conn: &DbConnection,
    key: &str,
) -> Result<Option<(i64, String)>, DbError> {
    let result = conn.query_row(
        "SELECT id, request_fingerprint FROM manual_sales WHERE idempotency_key = ?1",
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
    fn test_insert_manual_sale_req203_normal() {
        // REQ-203: 手動販売出庫記録（正常INSERT）
        // FUNC-10.4: 手動販売の正常INSERT
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-MS");

        let ms = NewManualSale {
            sale_date: "2026-04-06".to_string(),
            reason: "plu_unregistered".to_string(),
            note: None,
            idempotency_key: "ms-key-1".to_string(),
            request_fingerprint: "ms-fp-1".to_string(),
        };
        let id = insert_manual_sale(&conn, &ms).unwrap();
        assert!(id > 0);

        let item = NewManualSaleItem {
            manual_sale_id: id,
            product_code: "TEST-MS".to_string(),
            quantity: 1,
            amount: 500,
        };
        insert_manual_sale_item(&conn, &item).unwrap();
    }

    #[test]
    fn test_insert_manual_sale_req203_check_reason() {
        // REQ-203: 手動販売出庫記録（CHECK制約 — reason 不正値）
        // FUNC-10.4: CHECK制約 — reason 不正値
        let (_dir, conn) = setup_test_db();
        let result = conn.execute(
            "INSERT INTO manual_sales (sale_date, reason, idempotency_key, request_fingerprint, created_at)
             VALUES ('2026-04-06', 'invalid_reason', 'chk-ms', 'fp', '2026-04-06T00:00:00')",
            [],
        );
        assert!(result.is_err(), "不正 reason は CHECK で拒否");
    }

    #[test]
    fn test_insert_manual_sale_req203_duplicate_idempotency_key() {
        // REQ-203: 手動販売出庫記録（idempotency_key 重複 → DuplicateKey）
        let (_dir, conn) = setup_test_db();
        let ms = NewManualSale {
            sale_date: "2026-04-06".to_string(),
            reason: "other".to_string(),
            note: None,
            idempotency_key: "ms-dup".to_string(),
            request_fingerprint: "fp1".to_string(),
        };
        insert_manual_sale(&conn, &ms).unwrap();

        let ms2 = NewManualSale {
            idempotency_key: "ms-dup".to_string(),
            ..ms
        };
        let result = insert_manual_sale(&conn, &ms2);
        assert!(matches!(result, Err(DbError::DuplicateKey(_))));
    }

    #[test]
    fn test_find_manual_sale_by_idempotency_key_req203_hit() {
        // REQ-203: 手動販売出庫記録（冪等性キー検索 — ヒット → Some）
        let (_dir, conn) = setup_test_db();
        let ms = NewManualSale {
            sale_date: "2026-04-06".to_string(),
            reason: "other".to_string(),
            note: None,
            idempotency_key: "ms-find-1".to_string(),
            request_fingerprint: "ms-fp-find".to_string(),
        };
        let id = insert_manual_sale(&conn, &ms).unwrap();
        let found = find_manual_sale_by_idempotency_key(&conn, "ms-find-1").unwrap();
        assert_eq!(found, Some((id, "ms-fp-find".to_string())));
    }

    #[test]
    fn test_find_manual_sale_by_idempotency_key_req203_miss() {
        // REQ-203: 手動販売出庫記録（冪等性キー検索 — ミス → None）
        let (_dir, conn) = setup_test_db();
        let found = find_manual_sale_by_idempotency_key(&conn, "nonexistent").unwrap();
        assert_eq!(found, None);
    }

    #[test]
    fn test_get_manual_sale_record_detail_req203_req206_includes_items_total_and_movements() {
        // REQ-203 / REQ-206: 手動販売詳細は明細、販売金額合計、関連movementを返す
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "MS-DET");

        let record_id = insert_manual_sale(
            &conn,
            &NewManualSale {
                sale_date: "2026-06-27".to_string(),
                reason: "plu_unregistered".to_string(),
                note: Some("店頭販売".to_string()),
                idempotency_key: "ms-detail".to_string(),
                request_fingerprint: "fp-detail".to_string(),
            },
        )
        .unwrap();
        insert_manual_sale_item(
            &conn,
            &NewManualSaleItem {
                manual_sale_id: record_id,
                product_code: "MS-DET".to_string(),
                quantity: 2,
                amount: 980,
            },
        )
        .unwrap();
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, note, is_voided, created_at, business_date, phase) \
             VALUES ('MS-DET', 'sale_manual', -2, 3, 'manual_sale', ?1, '手動販売', 0, '2026-06-27T11:00:00', '2026-04-01', 0)",
            rusqlite::params![record_id],
        )
        .unwrap();

        let detail = get_manual_sale_record_detail(&conn, record_id).unwrap();

        assert_eq!(detail.id, record_id);
        assert_eq!(detail.sale_date, "2026-06-27");
        assert_eq!(detail.reason, ManualSaleReason::PluUnregistered);
        assert_eq!(detail.note, Some("店頭販売".to_string()));
        assert_eq!(detail.status, "active");
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.items[0].product_code, "MS-DET");
        assert_eq!(detail.total_amount, 980);
        assert_eq!(detail.movements.len(), 1);
        assert_eq!(
            detail.movements[0].reference_type,
            Some(super::super::inventory_repo::ReferenceType::ManualSale)
        );
    }
}
