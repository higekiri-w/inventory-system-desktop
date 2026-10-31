//! 振替（詰め替え・切り分け）の記録のリポジトリ
//!
//! 21-io-inventory-repo.md §10.5a に基づく実装（D-143）。
//! 在庫の変動は BIZ の `apply_stock_change` が書き、この module は書かない。

use super::inventory_repo::{parse_movement_type, parse_reference_type, MovementRecord};
use super::product_repo::{parse_stock_unit, ProductStockUnit};
use super::{DbConnection, DbError};

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 振替の明細の向き（transfer_items.direction の CHECK と同じ値の集合。D-061・D-064）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum TransferDirection {
    /// 減らす商品（元）
    Out,
    /// 増やす商品（先）
    In,
}

impl TransferDirection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::In => "in",
        }
    }
}

fn parse_transfer_direction(value: String) -> rusqlite::Result<TransferDirection> {
    match value.as_str() {
        "out" => Ok(TransferDirection::Out),
        "in" => Ok(TransferDirection::In),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown transfer direction: {value}").into(),
        )),
    }
}

/// 振替の記録の INSERT 用
#[derive(Debug)]
pub struct NewTransferRecord {
    pub transfer_date: String,
    pub idempotency_key: String,
    pub request_fingerprint: String,
}

/// 振替の明細の INSERT 用
#[derive(Debug)]
pub struct NewTransferItem {
    pub transfer_record_id: i64,
    pub product_code: String,
    pub direction: TransferDirection,
    /// 正の整数。向きは direction で持つ
    pub quantity: i64,
}

/// 振替の記録の詳細の明細行
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct TransferRecordDetailItem {
    pub id: i64,
    pub product_code: String,
    pub product_name: String,
    pub department_name: String,
    pub stock_unit: ProductStockUnit,
    pub direction: TransferDirection,
    pub quantity: i64,
}

/// 振替の記録の詳細
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct TransferRecordDetail {
    pub id: i64,
    pub transfer_date: String,
    /// 今は常に `"active"`（取消は持たない）
    pub status: String,
    pub created_at: String,
    /// 減らす明細を先、同じ向きの中は明細の id の順
    pub items: Vec<TransferRecordDetailItem>,
    pub movements: Vec<MovementRecord>,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// transfer_records に 1 行 INSERT し、id を返す（冪等キーの重複は `DbError::DuplicateKey`）
///
/// 21-io-inventory-repo.md §10.5a
pub fn insert_transfer_record(
    conn: &DbConnection,
    record: &NewTransferRecord,
) -> Result<i64, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO transfer_records (transfer_date, idempotency_key, request_fingerprint, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            record.transfer_date,
            record.idempotency_key,
            record.request_fingerprint,
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// transfer_items に 1 行 INSERT
///
/// 21-io-inventory-repo.md §10.5a
pub fn insert_transfer_item(conn: &DbConnection, item: &NewTransferItem) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO transfer_items (transfer_record_id, product_code, direction, quantity)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            item.transfer_record_id,
            item.product_code,
            item.direction.as_str(),
            item.quantity,
        ],
    )?;
    Ok(())
}

/// idempotency_key で振替の記録を探し、`(record_id, request_fingerprint)` を返す
///
/// 21-io-inventory-repo.md §10.5a
pub fn find_transfer_by_idempotency_key(
    conn: &DbConnection,
    key: &str,
) -> Result<Option<(i64, String)>, DbError> {
    let result = conn.query_row(
        "SELECT id, request_fingerprint FROM transfer_records WHERE idempotency_key = ?1",
        rusqlite::params![key],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );
    match result {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::from(e)),
    }
}

/// 振替の記録の詳細（ヘッダ・明細・関連 movement）。movement の `source` は BIZ が補う
///
/// 21-io-inventory-repo.md §10.5a
pub fn get_transfer_record_detail(
    conn: &DbConnection,
    record_id: i64,
) -> Result<TransferRecordDetail, DbError> {
    let header = conn.query_row(
        "SELECT id, transfer_date, created_at FROM transfer_records WHERE id = ?1",
        rusqlite::params![record_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    );
    let (id, transfer_date, created_at) = match header {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(DbError::NotFound),
        Err(e) => return Err(DbError::from(e)),
    };

    let mut item_stmt = conn.prepare(
        "SELECT ti.id, ti.product_code, p.name, d.name, p.stock_unit, ti.direction, ti.quantity
         FROM transfer_items ti
         JOIN products p ON p.product_code = ti.product_code
         JOIN departments d ON d.id = p.department_id
         WHERE ti.transfer_record_id = ?1
         ORDER BY CASE ti.direction WHEN 'out' THEN 0 ELSE 1 END, ti.id",
    )?;
    let items = item_stmt
        .query_map(rusqlite::params![record_id], |row| {
            Ok(TransferRecordDetailItem {
                id: row.get(0)?,
                product_code: row.get(1)?,
                product_name: row.get(2)?,
                department_name: row.get(3)?,
                stock_unit: parse_stock_unit(row.get(4)?)?,
                direction: parse_transfer_direction(row.get(5)?)?,
                quantity: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut movement_stmt = conn.prepare(
        "SELECT id, product_code, movement_type, quantity, stock_after,
                reference_type, reference_id, note, created_at
         FROM inventory_movements
         WHERE reference_type = 'transfer_record'
           AND reference_id = ?1
           AND is_voided = 0
         ORDER BY created_at ASC, id ASC",
    )?;
    let movements = movement_stmt
        .query_map(rusqlite::params![record_id], |row| {
            Ok(MovementRecord {
                id: row.get(0)?,
                product_code: row.get(1)?,
                movement_type: parse_movement_type(row.get(2)?)?,
                quantity: row.get(3)?,
                stock_after: row.get(4)?,
                reference_type: parse_reference_type(row.get(5)?),
                reference_id: row.get(6)?,
                source: None,
                note: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(TransferRecordDetail {
        id,
        transfer_date,
        status: "active".to_string(),
        created_at,
        items,
        movements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::*;

    fn record(conn: &DbConnection, key: &str) -> i64 {
        insert_transfer_record(
            conn,
            &NewTransferRecord {
                transfer_date: "2026-06-01".to_string(),
                idempotency_key: key.to_string(),
                request_fingerprint: "fp".to_string(),
            },
        )
        .unwrap()
    }

    fn item(conn: &DbConnection, record_id: i64, code: &str, direction: TransferDirection) {
        insert_transfer_item(
            conn,
            &NewTransferItem {
                transfer_record_id: record_id,
                product_code: code.to_string(),
                direction,
                quantity: 2,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_transfer_repo_d143_duplicate_key_and_find() {
        // 21 §10.5a: 冪等キーの重複は DuplicateKey、find は (id, fingerprint)
        let (_dir, conn) = setup_test_db();
        let id = record(&conn, "tr-key");
        let duplicate = insert_transfer_record(
            &conn,
            &NewTransferRecord {
                transfer_date: "2026-06-02".to_string(),
                idempotency_key: "tr-key".to_string(),
                request_fingerprint: "fp2".to_string(),
            },
        );
        assert!(matches!(duplicate, Err(DbError::DuplicateKey(_))));
        assert_eq!(
            find_transfer_by_idempotency_key(&conn, "tr-key").unwrap(),
            Some((id, "fp".to_string()))
        );
        assert_eq!(
            find_transfer_by_idempotency_key(&conn, "none").unwrap(),
            None
        );
    }

    #[test]
    fn test_get_transfer_record_detail_d143_r3_out_items_first_then_id_order() {
        // 21 §10.5a / Test Plan R3: 減らす明細を先、同じ向きの中は明細の id の順。単位つき、source は None
        let (_dir, conn) = setup_test_db();
        for code in ["TR-A", "TR-B", "TR-C"] {
            seed_product(&conn, code);
        }
        conn.execute(
            "UPDATE products SET stock_unit = 'm' WHERE product_code = 'TR-C'",
            [],
        )
        .unwrap();
        let id = record(&conn, "tr-detail");
        item(&conn, id, "TR-A", TransferDirection::In);
        item(&conn, id, "TR-C", TransferDirection::Out);
        item(&conn, id, "TR-B", TransferDirection::In);

        let detail = get_transfer_record_detail(&conn, id).unwrap();

        assert_eq!(detail.status, "active");
        assert_eq!(detail.transfer_date, "2026-06-01");
        let order: Vec<(&str, TransferDirection)> = detail
            .items
            .iter()
            .map(|item| (item.product_code.as_str(), item.direction))
            .collect();
        assert_eq!(
            order,
            vec![
                ("TR-C", TransferDirection::Out),
                ("TR-A", TransferDirection::In),
                ("TR-B", TransferDirection::In),
            ]
        );
        assert_eq!(detail.items[0].stock_unit, ProductStockUnit::M);
        assert!(detail.movements.is_empty());
    }

    #[test]
    fn test_get_transfer_record_detail_d143_r4_not_found() {
        // 21 §10.5a / Test Plan R4: 無い id は NotFound
        let (_dir, conn) = setup_test_db();
        assert!(matches!(
            get_transfer_record_detail(&conn, 999),
            Err(DbError::NotFound)
        ));
    }
}
