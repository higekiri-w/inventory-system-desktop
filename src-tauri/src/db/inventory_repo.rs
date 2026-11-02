//! 在庫変動の共通型・関数
//!
//! 21-io-inventory-repo.md §2.7 に基づく実装。
//!
//! DB CHECK 制約の有限値を enum と canonical wire 文字列で一元管理する。

use super::{DbConnection, DbError};

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 在庫変動種別（db-design/tracking-system-tables.md: inventory_movements.movement_type CHECK制約に対応）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    SaleAuto,
    SaleManual,
    Receiving,
    Return,
    Disposal,
    Stocktake,
}

impl MovementType {
    pub const fn as_str(self) -> &'static str {
        match self {
            MovementType::SaleAuto => "sale_auto",
            MovementType::SaleManual => "sale_manual",
            MovementType::Receiving => "receiving",
            MovementType::Return => "return",
            MovementType::Disposal => "disposal",
            MovementType::Stocktake => "stocktake",
        }
    }
}

pub(super) fn parse_movement_type(value: String) -> rusqlite::Result<MovementType> {
    match value.as_str() {
        "sale_auto" => Ok(MovementType::SaleAuto),
        "sale_manual" => Ok(MovementType::SaleManual),
        "receiving" => Ok(MovementType::Receiving),
        "return" => Ok(MovementType::Return),
        "disposal" => Ok(MovementType::Disposal),
        "stocktake" => Ok(MovementType::Stocktake),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown movement_type: {value}").into(),
        )),
    }
}

/// 参照先種別（db-design/tracking-system-tables.md: inventory_movements.reference_type CHECK制約に対応）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    CsvImport,
    ManualSale,
    ReceivingRecord,
    ReturnRecord,
    DisposalRecord,
    Stocktake,
}

impl ReferenceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            ReferenceType::CsvImport => "csv_import",
            ReferenceType::ManualSale => "manual_sale",
            ReferenceType::ReceivingRecord => "receiving_record",
            ReferenceType::ReturnRecord => "return_record",
            ReferenceType::DisposalRecord => "disposal_record",
            ReferenceType::Stocktake => "stocktake",
        }
    }
}

pub(super) fn parse_reference_type(value: Option<String>) -> Option<ReferenceType> {
    match value.as_deref() {
        Some("csv_import") => Some(ReferenceType::CsvImport),
        Some("manual_sale") => Some(ReferenceType::ManualSale),
        Some("receiving_record") => Some(ReferenceType::ReceivingRecord),
        Some("return_record") => Some(ReferenceType::ReturnRecord),
        Some("disposal_record") => Some(ReferenceType::DisposalRecord),
        Some("stocktake") => Some(ReferenceType::Stocktake),
        Some(_) | None => None,
    }
}

/// 在庫変動INSERT用
///
/// 21-io-inventory-repo.md §2.7
#[derive(Debug)]
pub struct NewMovement {
    pub product_code: String,
    pub movement_type: MovementType,
    pub quantity: i64,
    pub stock_after: i64,
    pub reference_type: Option<ReferenceType>,
    pub reference_id: Option<i64>,
    pub note: Option<String>,
    /// 並びの鍵の業務日（YYYY-MM-DD。形と暦の検査は書く側の BIZ。SPEC-STK-CNT-D1）
    pub business_date: String,
    /// 精算回数（Z004 だけ。`Some` なら phase 1、`None` なら phase 0 を `insert_movement` が書く）
    pub settlement_no: Option<i64>,
}

/// 求め直しが読む計数の 1 行（21 `list_stock_counts_for_product`）
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StockCountRow {
    pub id: i64,
    pub count_value: i64,
    /// `true` なら fresh（「入れ直す」）、`false` なら default（「数を保存」「0 にする」）
    pub fresh: bool,
    pub business_date: String,
    // 保存した settlement_no（ADR D1 の鍵の n）は判定に使わないので読まない（取り違えを型で作れないようにする）
    pub seen_movement_id: Option<i64>,
    /// fresh だけ。保存の時点の `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL）。
    /// ADR D3 の判定の n はこれ以下の取込みID を持つ active な取込みから求める
    pub seen_import_id: Option<i64>,
    /// 保存の時刻（判定には使わない。明細の表示用の写しへ写す。35）
    pub counted_at: String,
}

/// 求め直しが読む有効な movement の 1 行（21 `list_ledger_movements_for_product`）
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LedgerMovementRow {
    pub id: i64,
    pub quantity: i64,
    pub business_date: String,
    pub phase: i64,
    pub settlement_no: Option<i64>,
    /// 初期在庫の印（`reference_type` が NULL で `movement_type` が `receiving`。商品の登録・一括 import が作る。
    /// demo の seed の `sale_auto`〈`reference_type` NULL〉は初期在庫にしない。ADR D9）
    pub initial_stock: bool,
}

/// 計数の保存（21 `insert_stock_count`。`stock_counts` へ 1 行。書換え・削除の関数は持たない）
#[derive(Debug, Clone)]
pub(crate) struct NewStockCount {
    pub product_code: String,
    pub count_value: i64,
    /// `true` なら fresh（「入れ直す」）
    pub fresh: bool,
    pub business_date: String,
    /// fresh だけ。保存の時点の active な Z004 の最大の精算回数（取込みが無ければ NULL。ADR D1 の鍵の n）
    pub settlement_no: Option<i64>,
    /// fresh だけ。保存の時点の `MAX(inventory_movements.id)`（無ければ 0）
    pub seen_movement_id: Option<i64>,
    /// fresh だけ。保存の時点の `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL。24）
    pub seen_import_id: Option<i64>,
    pub stocktake_item_id: Option<i64>,
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// inventory_movements に1行INSERTし、挿入されたIDを返す
///
/// 全ての在庫変動記録の共通入口。
///
/// 21-io-inventory-repo.md §2.7
pub fn insert_movement(conn: &DbConnection, movement: &NewMovement) -> Result<i64, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let ref_type_str = movement.reference_type.as_ref().map(|r| r.as_str());
    // phase は settlement_no から決める（食い違いを型で作れないようにする。21、Y8）
    let phase = i64::from(movement.settlement_no.is_some());
    conn.execute(
        "INSERT INTO inventory_movements (
            product_code, movement_type, quantity, stock_after,
            reference_type, reference_id, note, is_voided, created_at,
            business_date, phase, settlement_no
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            movement.product_code,
            movement.movement_type.as_str(),
            movement.quantity,
            movement.stock_after,
            ref_type_str,
            movement.reference_id,
            movement.note,
            now,
            movement.business_date,
            phase,
            movement.settlement_no,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 写し（`stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`）を書く
///
/// 呼ぶのは BIZ の `recompute_stock` だけ（21、SPEC-STK-CNT-D2）。`range` が `Some` なら要再確認。
/// 対象なしは false。
///
/// 21-io-inventory-repo.md §10.6
pub fn update_stock_quantity(
    conn: &DbConnection,
    product_code: &str,
    stock: i64,
    range: Option<(i64, i64)>,
) -> Result<bool, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let affected = conn.execute(
        "UPDATE products SET stock_quantity = ?1, stock_recheck = ?2, stock_range_low = ?3,
             stock_range_high = ?4, updated_at = ?5
         WHERE product_code = ?6",
        rusqlite::params![
            stock,
            range.is_some(),
            range.map(|r| r.0),
            range.map(|r| r.1),
            now,
            product_code
        ],
    )?;
    Ok(affected == 1)
}

/// その TX で INSERT した movement 1 行の stock_after を書く。更新が 1 行でなければ QueryFailed
///
/// 21-io-inventory-repo.md（D-118 (a)）
pub(crate) fn update_movement_stock_after(
    conn: &DbConnection,
    movement_id: i64,
    stock_after: i64,
) -> Result<(), DbError> {
    let affected = conn.execute(
        "UPDATE inventory_movements SET stock_after = ?1 WHERE id = ?2",
        rusqlite::params![stock_after, movement_id],
    )?;
    if affected != 1 {
        return Err(DbError::QueryFailed(format!(
            "stock_after の更新対象が1行ではありません: movement_id={movement_id}, 件数={affected}"
        )));
    }
    Ok(())
}

/// 商品の計数を id の順で返す（求め直しの読取り。21）
pub(crate) fn list_stock_counts_for_product(
    conn: &DbConnection,
    product_code: &str,
) -> Result<Vec<StockCountRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, count_value, count_kind, business_date, seen_movement_id, seen_import_id,
                counted_at
         FROM stock_counts WHERE product_code = ?1 ORDER BY id",
    )?;
    let rows = stmt
        .query_map([product_code], |row| {
            Ok(StockCountRow {
                id: row.get(0)?,
                count_value: row.get(1)?,
                fresh: row.get::<_, String>(2)? == "fresh",
                business_date: row.get(3)?,
                seen_movement_id: row.get(4)?,
                seen_import_id: row.get(5)?,
                counted_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// 初期在庫の印の SQL の式（`reference_type` が NULL で `movement_type` が `receiving`。ADR D9）。
/// 求め直しの読取りと在庫変動の履歴の読取りが共有する（44 の `list_movements`）
const INITIAL_STOCK_SQL: &str = "reference_type IS NULL AND movement_type = 'receiving'";

/// 商品の `is_voided = 0` の movement を id の順で返す（求め直しの読取り。21）
pub(crate) fn list_ledger_movements_for_product(
    conn: &DbConnection,
    product_code: &str,
) -> Result<Vec<LedgerMovementRow>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT id, quantity, business_date, phase, settlement_no, {INITIAL_STOCK_SQL}
         FROM inventory_movements WHERE product_code = ?1 AND is_voided = 0 ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([product_code], |row| {
            Ok(LedgerMovementRow {
                id: row.get(0)?,
                quantity: row.get(1)?,
                business_date: row.get(2)?,
                phase: row.get(3)?,
                settlement_no: row.get(4)?,
                initial_stock: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// 計数を 1 行保存し、id を返す（21。BIZ-06 の `save_stock_count`・`zero_uncounted_items` だけが呼ぶ）
pub(crate) fn insert_stock_count(
    conn: &DbConnection,
    count: &NewStockCount,
) -> Result<i64, DbError> {
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
             settlement_no, seen_movement_id, seen_import_id, stocktake_item_id, counted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            count.product_code,
            count.count_value,
            if count.fresh { "fresh" } else { "default" },
            count.business_date,
            count.settlement_no,
            count.seen_movement_id,
            count.seen_import_id,
            count.stocktake_item_id,
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// `MAX(inventory_movements.id)`（無ければ 0。fresh な計数と完了のカーソル）
pub(crate) fn max_movement_id(conn: &DbConnection) -> Result<i64, DbError> {
    Ok(conn.query_row(
        "SELECT COALESCE(MAX(id), 0) FROM inventory_movements",
        [],
        |row| row.get(0),
    )?)
}

/// `MAX(stock_counts.id)`（無ければ NULL。完了のカーソル）
pub(crate) fn max_stock_count_id(conn: &DbConnection) -> Result<Option<i64>, DbError> {
    Ok(conn.query_row("SELECT MAX(id) FROM stock_counts", [], |row| row.get(0))?)
}

/// 計数を持つ商品の一覧（Z004 の確定・取消の求め直しの対象。ADR の横断の表）
pub(crate) fn list_counted_product_codes(conn: &DbConnection) -> Result<Vec<String>, DbError> {
    let mut stmt =
        conn.prepare("SELECT DISTINCT product_code FROM stock_counts ORDER BY product_code")?;
    let rows = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// CMD-06: 在庫変動履歴用型・関数
// ---------------------------------------------------------------------------

/// 在庫変動履歴の検索条件
///
/// 44-cmd-inventory.md §23.8 list_movements
#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct MovementQuery {
    pub product_code: String,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub movement_type: Option<MovementType>,
    pub page: u32,
    pub per_page: u32,
}

/// 在庫変動の元業務記録リンク
///
/// 44-cmd-inventory.md §23.8 list_movements
#[derive(Debug, Clone, PartialEq, serde::Serialize, specta::Type)]
pub struct MovementSourceLink {
    pub label: String,
    pub route: String,
}

/// 在庫変動履歴1件
///
/// 44-cmd-inventory.md §23.8 list_movements
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct MovementRecord {
    pub id: i64,
    pub product_code: String,
    pub movement_type: MovementType,
    pub quantity: i64,
    pub stock_after: i64,
    pub reference_type: Option<ReferenceType>,
    pub reference_id: Option<i64>,
    pub source: Option<MovementSourceLink>,
    pub note: Option<String>,
    pub created_at: String,
}

/// 在庫変動の履歴の movement の 1 行（44 の `list_movements`。並びは BIZ）
#[derive(Debug)]
pub(crate) struct HistoryMovementRow {
    /// `source` は BIZ が補う
    pub movement: MovementRecord,
    pub business_date: String,
    pub settlement_no: Option<i64>,
    pub initial_stock: bool,
}

/// 在庫変動の履歴の計数の 1 行（44 の `list_movements`。並びは BIZ）
#[derive(Debug)]
pub(crate) struct HistoryCountRow {
    pub id: i64,
    pub count_value: i64,
    /// `true` なら fresh（「入れ直す」）
    pub fresh: bool,
    pub business_date: String,
    pub settlement_no: Option<i64>,
    pub counted_at: String,
    /// 計数の商品の今の単位
    pub stock_unit: super::product_repo::ProductStockUnit,
    /// 計数が参照する棚卸しの明細の棚卸し
    pub stocktake_id: Option<i64>,
}

/// 商品の `is_voided = 0` の movement を、業務日の範囲（両端を含む）と種別で絞って返す（並べない。44）
pub(crate) fn list_history_movements(
    conn: &DbConnection,
    query: &MovementQuery,
) -> Result<Vec<HistoryMovementRow>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT id, product_code, movement_type, quantity, stock_after, reference_type,
                reference_id, note, created_at, business_date, settlement_no, {INITIAL_STOCK_SQL}
         FROM inventory_movements
         WHERE product_code = ?1 AND is_voided = 0
           AND (?2 IS NULL OR business_date >= ?2) AND (?3 IS NULL OR business_date <= ?3)
           AND (?4 IS NULL OR movement_type = ?4)"
    ))?;
    let rows = stmt
        .query_map(
            rusqlite::params![
                query.product_code,
                query.date_from,
                query.date_to,
                query.movement_type.map(MovementType::as_str),
            ],
            |row| {
                Ok(HistoryMovementRow {
                    movement: MovementRecord {
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
                    },
                    business_date: row.get(9)?,
                    settlement_no: row.get(10)?,
                    initial_stock: row.get(11)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// 商品の計数を、業務日の範囲（両端を含む）で絞って返す（並べない。44）
pub(crate) fn list_history_counts(
    conn: &DbConnection,
    query: &MovementQuery,
) -> Result<Vec<HistoryCountRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT sc.id, sc.count_value, sc.count_kind, sc.business_date, sc.settlement_no,
                sc.counted_at, p.stock_unit, si.stocktake_id
         FROM stock_counts sc
         JOIN products p ON p.product_code = sc.product_code
         LEFT JOIN stocktake_items si ON si.id = sc.stocktake_item_id
         WHERE sc.product_code = ?1
           AND (?2 IS NULL OR sc.business_date >= ?2) AND (?3 IS NULL OR sc.business_date <= ?3)",
    )?;
    let rows = stmt
        .query_map(
            rusqlite::params![query.product_code, query.date_from, query.date_to],
            |row| {
                Ok(HistoryCountRow {
                    id: row.get(0)?,
                    count_value: row.get(1)?,
                    fresh: row.get::<_, String>(2)? == "fresh",
                    business_date: row.get(3)?,
                    settlement_no: row.get(4)?,
                    counted_at: row.get(5)?,
                    stock_unit: super::product_repo::parse_stock_unit(row.get(6)?)?,
                    stocktake_id: row.get(7)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::inventory_common::{validate_and_offset, ListQuery};
    use crate::db::test_support::*;
    use crate::db::DbError;

    #[test]
    fn test_movement_type_as_str_req201() {
        // REQ-201: 在庫変動記録（MovementType enum — 全バリアントの文字列変換）
        // FUNC-2.7: MovementType enum — 全バリアントの文字列変換
        assert_eq!(MovementType::SaleAuto.as_str(), "sale_auto");
        assert_eq!(MovementType::SaleManual.as_str(), "sale_manual");
        assert_eq!(MovementType::Receiving.as_str(), "receiving");
        assert_eq!(MovementType::Return.as_str(), "return");
        assert_eq!(MovementType::Disposal.as_str(), "disposal");
        assert_eq!(MovementType::Stocktake.as_str(), "stocktake");
    }

    #[test]
    fn test_reference_type_as_str_req201() {
        // REQ-201: 在庫変動記録（ReferenceType enum — 全バリアントの文字列変換）
        // FUNC-2.7: ReferenceType enum — 全バリアントの文字列変換
        assert_eq!(ReferenceType::CsvImport.as_str(), "csv_import");
        assert_eq!(ReferenceType::ManualSale.as_str(), "manual_sale");
        assert_eq!(ReferenceType::ReceivingRecord.as_str(), "receiving_record");
        assert_eq!(ReferenceType::ReturnRecord.as_str(), "return_record");
        assert_eq!(ReferenceType::DisposalRecord.as_str(), "disposal_record");
        assert_eq!(ReferenceType::Stocktake.as_str(), "stocktake");
    }

    #[test]
    fn test_reference_type_unknown_falls_back_to_none_req303() {
        assert_eq!(
            parse_reference_type(Some("legacy_reference".to_string())),
            None
        );
        assert_eq!(parse_reference_type(None), None);
    }

    #[test]
    fn test_movement_type_unknown_is_internal_req303() {
        assert!(parse_movement_type("legacy_movement".to_string()).is_err());
    }

    #[test]
    fn test_insert_movement_req201_normal() {
        // REQ-201: 在庫変動記録（insert_movement 正常INSERT）
        // FUNC-2.7: insert_movement — 正常INSERT（receiving）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-001");

        let movement = NewMovement {
            product_code: "TEST-001".to_string(),
            movement_type: MovementType::Receiving,
            quantity: 10,
            stock_after: 10,
            reference_type: Some(ReferenceType::ReceivingRecord),
            reference_id: Some(1),
            note: Some("入庫テスト".to_string()),
            business_date: "2026-04-01".to_string(),
            settlement_no: None,
        };
        let id = insert_movement(&conn, &movement).unwrap();
        assert!(id > 0, "挿入されたIDは正の整数");

        // is_voided=0, created_at非NULL、ISO 8601形式を確認
        let (is_voided, created_at): (bool, String) = conn
            .query_row(
                "SELECT is_voided, created_at FROM inventory_movements WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(!is_voided, "is_voidedは0であるべき");
        // chrono でパース成功 = ISO 8601 形式
        chrono::NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%dT%H:%M:%S")
            .expect("created_at は %Y-%m-%dT%H:%M:%S 形式であるべき");
    }

    #[test]
    fn test_insert_movement_req201_null_reference() {
        // REQ-201: 在庫変動記録（insert_movement reference_type=None）
        // FUNC-2.7: insert_movement — reference_type=None（初期在庫投入パターン）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-002");

        let movement = NewMovement {
            product_code: "TEST-002".to_string(),
            movement_type: MovementType::Receiving,
            quantity: 5,
            stock_after: 5,
            reference_type: None,
            reference_id: None,
            note: Some("初期在庫投入".to_string()),
            business_date: "2026-04-01".to_string(),
            settlement_no: None,
        };
        let id = insert_movement(&conn, &movement).unwrap();
        assert!(id > 0);

        let ref_type: Option<String> = conn
            .query_row(
                "SELECT reference_type FROM inventory_movements WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(ref_type.is_none(), "reference_type は NULL");
    }

    #[test]
    fn test_insert_movement_req201_fk_violation_product() {
        // REQ-201: 在庫変動記録（存在しないproduct_codeでFK違反）
        // FUNC-2.7: insert_movement — 存在しないproduct_codeでFK違反
        let (_dir, conn) = setup_test_db();

        let movement = NewMovement {
            product_code: "NONEXISTENT".to_string(),
            movement_type: MovementType::Receiving,
            quantity: 1,
            stock_after: 1,
            reference_type: None,
            reference_id: None,
            note: None,
            business_date: "2026-04-01".to_string(),
            settlement_no: None,
        };
        let result = insert_movement(&conn, &movement);
        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation(_))),
            "不正なproduct_codeで ForeignKeyViolation が返るべき: {:?}",
            result
        );
    }

    fn key_movement(code: &str, date: &str, settlement_no: Option<i64>) -> NewMovement {
        NewMovement {
            product_code: code.to_string(),
            movement_type: MovementType::Receiving,
            quantity: 2,
            stock_after: 2,
            reference_type: None,
            reference_id: None,
            note: None,
            business_date: date.to_string(),
            settlement_no,
        }
    }

    #[test]
    fn test_insert_movement_req201_writes_key_and_phase_from_settlement_no() {
        // REQ-201 / 21（phase は settlement_no から決める）/ SPEC-STK-CNT-D1・Y8
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "KEY-1");
        let key = |id: i64| -> (String, i64, Option<i64>) {
            conn.query_row(
                "SELECT business_date, phase, settlement_no FROM inventory_movements WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap()
        };
        let id = insert_movement(&conn, &key_movement("KEY-1", "2026-05-03", None)).unwrap();
        assert_eq!(key(id), ("2026-05-03".to_string(), 0, None));
        // vZ の CHECK: phase = 1 は Z004 の movement だけ（fixture を csv_import に。Matrix T10）
        let z004 = NewMovement {
            movement_type: MovementType::SaleAuto,
            reference_type: Some(ReferenceType::CsvImport),
            reference_id: Some(1),
            ..key_movement("KEY-1", "2026-05-04", Some(0))
        };
        let id = insert_movement(&conn, &z004).unwrap();
        assert_eq!(key(id), ("2026-05-04".to_string(), 1, Some(0)));
    }

    #[test]
    fn test_inventory_repo_req201_business_date_check_rejects_noncanonical() {
        // REQ-201 / Y1（DB の CHECK は形だけを見る）/ Matrix W4
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "KEY-2");
        for date in ["2026-4-7", "2026/04/07", " 2026-04-07", "2026-04-07 ", ""] {
            assert!(
                insert_movement(&conn, &key_movement("KEY-2", date, None)).is_err(),
                "{date:?}"
            );
        }
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_update_movement_stock_after_req201_exactly_one_row() {
        // REQ-201 / 21 update_movement_stock_after（1 行でなければ QueryFailed）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "KEY-3");
        let id = insert_movement(&conn, &key_movement("KEY-3", "2026-05-03", None)).unwrap();
        update_movement_stock_after(&conn, id, 11).unwrap();
        let stock_after: i64 = conn
            .query_row(
                "SELECT stock_after FROM inventory_movements WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stock_after, 11);
        assert!(matches!(
            update_movement_stock_after(&conn, id + 1, 11),
            Err(DbError::QueryFailed(_))
        ));
    }

    // -----------------------------------------------------------------------
    // validate_and_offset テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_and_offset_req201_normal() {
        // REQ-201: 在庫変動記録（validate_and_offset 正常系）
        // FUNC-10.1: 正常系 page=2, per_page=10 → offset=10
        let q = ListQuery {
            page: 2,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let (limit, offset) = validate_and_offset(&q).unwrap();
        assert_eq!(limit, 10);
        assert_eq!(offset, 10);
    }

    #[test]
    fn test_validate_and_offset_req201_page_zero() {
        // REQ-201: 在庫変動記録（validate_and_offset page=0 エラー）
        // FUNC-10.1: page=0 → QueryFailed
        let q = ListQuery {
            page: 0,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let result = validate_and_offset(&q);
        assert!(
            matches!(result, Err(DbError::QueryFailed(_))),
            "page=0 は拒否: {:?}",
            result
        );
    }

    #[test]
    fn test_validate_and_offset_req201_per_page_zero() {
        // REQ-201: 在庫変動記録（validate_and_offset per_page=0 エラー）
        // FUNC-10.1: per_page=0 → QueryFailed
        let q = ListQuery {
            page: 1,
            per_page: 0,
            date_from: None,
            date_to: None,
        };
        let result = validate_and_offset(&q);
        assert!(
            matches!(result, Err(DbError::QueryFailed(_))),
            "per_page=0 は拒否: {:?}",
            result
        );
    }

    #[test]
    fn test_validate_and_offset_req201_overflow() {
        // REQ-201: 在庫変動記録（validate_and_offset オーバーフロー防止）
        // FUNC-10.1: 巨大値で (page-1)*per_page がオーバーフローしないこと
        let q = ListQuery {
            page: u32::MAX,
            per_page: u32::MAX,
            date_from: None,
            date_to: None,
        };
        let result = validate_and_offset(&q);
        assert!(
            matches!(result, Err(DbError::QueryFailed(_))),
            "overflow は拒否: {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // update_stock_quantity テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_stock_quantity_req201_normal() {
        // REQ-201: 在庫数更新（正常更新 → true）
        // FUNC-10.6: 正常更新 → true
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-UPD");

        let updated = update_stock_quantity(&conn, "TEST-UPD", 42, None).unwrap();
        assert!(updated);

        let qty: i64 = conn
            .query_row(
                "SELECT stock_quantity FROM products WHERE product_code = 'TEST-UPD'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(qty, 42);
    }

    #[test]
    fn test_update_stock_quantity_req205_writes_flag_and_range() {
        // REQ-205 / 21（写しの 4 項目を書く。SPEC-STK-CNT-D2）/ Matrix T6: 範囲つきの書込みと、範囲なしで 0・NULL へ戻す
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-RNG");
        let copy = |conn: &DbConnection| -> (i64, bool, Option<i64>, Option<i64>) {
            conn.query_row(
                "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
                 FROM products WHERE product_code = 'TEST-RNG'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap()
        };
        assert!(update_stock_quantity(&conn, "TEST-RNG", 5, Some((4, 6))).unwrap());
        assert_eq!(copy(&conn), (5, true, Some(4), Some(6)));
        assert!(update_stock_quantity(&conn, "TEST-RNG", 7, None).unwrap());
        assert_eq!(copy(&conn), (7, false, None, None));
        // DB の CHECK が low > high を拒む
        assert!(update_stock_quantity(&conn, "TEST-RNG", 7, Some((8, 6))).is_err());
        assert_eq!(copy(&conn), (7, false, None, None));
    }

    #[test]
    fn test_update_stock_quantity_req201_nonexistent() {
        // REQ-201: 在庫数更新（存在しない商品 → false）
        // FUNC-10.6: 存在しない商品 → false
        let (_dir, conn) = setup_test_db();
        let updated = update_stock_quantity(&conn, "NONEXISTENT", 10, None).unwrap();
        assert!(!updated);
    }

    #[test]
    fn test_update_stock_quantity_req201_updates_timestamp() {
        // REQ-201: 在庫数更新（updated_at が更新されること）
        // FUNC-10.6: updated_at が更新されること
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-TS");

        let before: String = conn
            .query_row(
                "SELECT updated_at FROM products WHERE product_code = 'TEST-TS'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // わずかに待って updated_at が変わることを確認
        std::thread::sleep(std::time::Duration::from_millis(10));
        update_stock_quantity(&conn, "TEST-TS", 99, None).unwrap();

        let after: String = conn
            .query_row(
                "SELECT updated_at FROM products WHERE product_code = 'TEST-TS'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        // chrono の秒精度なのでタイミング次第で同じ可能性があるが、形式チェック
        chrono::NaiveDateTime::parse_from_str(&after, "%Y-%m-%dT%H:%M:%S")
            .expect("updated_at は正しい形式であるべき");
        // 少なくとも before 以降であること
        assert!(after >= before, "updated_at は更新前以降であるべき");
    }

    // -----------------------------------------------------------------------
    // 既存テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_db_check_constraint_req201_movement_type() {
        // REQ-201: CHECK制約（不正な movement_type を拒否）
        // FUNC-2.7: DB CHECK制約 — 不正な movement_type を生SQLで直接INSERT
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "TEST-CHK");

        let result = conn.execute(
            "INSERT INTO inventory_movements (
                product_code, movement_type, quantity, stock_after,
                is_voided, created_at
            , business_date, phase) VALUES ('TEST-CHK', 'invalid_type', 1, 1, 0, '2026-04-04T00:00:00', '2026-04-01', 0)",
            [],
        );
        assert!(
            result.is_err(),
            "不正な movement_type は CHECK制約で弾かれるべき"
        );
    }
}
