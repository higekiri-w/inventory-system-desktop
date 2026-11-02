//! 入庫・返品・廃棄の一覧取得（repo直呼び防止ラッパー）
//!
//! 44-cmd-inventory.md §23.10 に基づく実装。
//! BIZ層でper_page上限バリデーションを行い、IO層のリポジトリに委譲する。

use crate::biz::stocktake_service::StockCountKind;
use crate::biz::unit_amount::{cost_line_centi, cost_total_yen, price_basis_quantity};
use crate::biz::BizError;
use crate::db::disposal_repo::{
    self, DisposalRecordDetail, DisposalRecordSummary, InventoryRecordQuery, InventoryRecordSummary,
};
use crate::db::inventory_common::ListQuery;
use crate::db::inventory_repo::{
    self, MovementQuery, MovementRecord, MovementSourceLink, MovementType, ReferenceType,
};
use crate::db::manual_sale_repo::{self, ManualSaleRecordDetail};
use crate::db::product_repo::ProductStockUnit;
use crate::db::receiving_repo::ReceivingRecordDetail;
use crate::db::receiving_repo::{self, ReceivingRecordWithSupplier};
use crate::db::return_repo::{self, ReturnRecordDetail, ReturnRecordSummary};
use crate::db::{DbConnection, DbError, PaginatedResult};

/// per_page の上限
const MAX_PER_PAGE: u32 = 200;

/// ページパラメータのバリデーション
fn validate_page_params(query: &ListQuery) -> Result<(), BizError> {
    if query.page < 1 || query.per_page < 1 || query.per_page > MAX_PER_PAGE {
        return Err(BizError::ValidationFailed(
            "ページパラメータが不正です".to_string(),
        ));
    }
    Ok(())
}

/// 入庫記録一覧を返す
///
/// 44-cmd-inventory.md §23.2
pub fn list_receivings(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<ReceivingRecordWithSupplier>, BizError> {
    validate_page_params(query)?;
    Ok(receiving_repo::list_receiving_records(conn, query)?)
}

/// 返品記録一覧を返す
///
/// 44-cmd-inventory.md §23.5
pub fn list_returns(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<ReturnRecordSummary>, BizError> {
    validate_page_params(query)?;
    Ok(return_repo::list_return_records(conn, query)?)
}

/// 廃棄記録一覧を返す
///
/// 44-cmd-inventory.md §23.7
pub fn list_disposals(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<DisposalRecordSummary>, BizError> {
    validate_page_params(query)?;
    Ok(disposal_repo::list_disposal_records(conn, query)?)
}

/// 入出庫履歴ハブ用に業務記録一覧を返す。
///
/// 65-inventory-record-traceability.md §65.4 / TRACE-D1
pub fn list_inventory_records(
    conn: &DbConnection,
    query: &InventoryRecordQuery,
) -> Result<PaginatedResult<InventoryRecordSummary>, BizError> {
    let page_query = ListQuery {
        page: query.page,
        per_page: query.per_page,
        date_from: None,
        date_to: None,
    };
    validate_page_params(&page_query)?;

    if !matches!(
        query.record_type.as_deref(),
        None | Some("all")
            | Some("receiving_record")
            | Some("return_record")
            | Some("manual_sale")
            | Some("disposal_record")
            | Some("csv_import")
            | Some("stocktake")
    ) {
        return Err(BizError::ValidationFailed(
            "未対応の記録種別です".to_string(),
        ));
    }
    if !matches!(
        query.status.as_deref(),
        None | Some("all") | Some("active") | Some("canceled") | Some("in_progress")
    ) {
        return Err(BizError::ValidationFailed("未対応の状態です".to_string()));
    }

    Ok(disposal_repo::list_inventory_records(conn, query)?)
}

/// 入庫記録詳細を返す。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_receiving_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ReceivingRecordDetail, BizError> {
    match receiving_repo::get_receiving_record_detail(conn, record_id) {
        Ok(mut detail) => {
            // 行の金額と合計は BIZ が共通関数で求める（SPEC-UNIT-D6、X7）
            let mut line_centis = Vec::with_capacity(detail.items.len());
            for item in &mut detail.items {
                let centi = cost_line_centi(
                    i128::from(item.cost_price_centi),
                    item.quantity,
                    price_basis_quantity(item.stock_unit),
                    &item.product_code,
                )?;
                item.line_cost_centi = centi.to_string();
                line_centis.push(centi);
            }
            detail.total_cost = cost_total_yen(&line_centis)?;
            for movement in &mut detail.movements {
                movement.source =
                    resolve_movement_source(&movement.reference_type, &movement.reference_id);
            }
            Ok(detail)
        }
        Err(DbError::NotFound) => Err(BizError::NotFound(format!(
            "入庫記録が見つかりません: {}",
            record_id
        ))),
        Err(err) => Err(BizError::DatabaseError(err)),
    }
}

/// 返品・交換記録詳細を返す。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_return_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ReturnRecordDetail, BizError> {
    match return_repo::get_return_record_detail(conn, record_id) {
        Ok(mut detail) => {
            for movement in &mut detail.movements {
                movement.source =
                    resolve_movement_source(&movement.reference_type, &movement.reference_id);
            }
            Ok(detail)
        }
        Err(DbError::NotFound) => Err(BizError::NotFound(format!(
            "返品・交換記録が見つかりません: {}",
            record_id
        ))),
        Err(err) => Err(BizError::DatabaseError(err)),
    }
}

/// 手動販売記録詳細を返す。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_manual_sale_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ManualSaleRecordDetail, BizError> {
    match manual_sale_repo::get_manual_sale_record_detail(conn, record_id) {
        Ok(mut detail) => {
            for movement in &mut detail.movements {
                movement.source =
                    resolve_movement_source(&movement.reference_type, &movement.reference_id);
            }
            Ok(detail)
        }
        Err(DbError::NotFound) => Err(BizError::NotFound(format!(
            "手動販売記録が見つかりません: {}",
            record_id
        ))),
        Err(err) => Err(BizError::DatabaseError(err)),
    }
}

/// 廃棄・破損記録詳細を返す。
///
/// 65-inventory-record-traceability.md §65.5
pub fn get_disposal_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<DisposalRecordDetail, BizError> {
    match disposal_repo::get_disposal_record_detail(conn, record_id) {
        Ok(mut detail) => {
            // 行のロス原価と合計は BIZ が共通関数で求める（SPEC-UNIT-D6、X7）
            let mut line_centis = Vec::with_capacity(detail.items.len());
            for item in &mut detail.items {
                let centi = cost_line_centi(
                    i128::from(item.cost_price_centi),
                    item.quantity,
                    price_basis_quantity(item.stock_unit),
                    &item.product_code,
                )?;
                item.line_loss_cost_centi = centi.to_string();
                line_centis.push(centi);
            }
            detail.total_loss_cost = cost_total_yen(&line_centis)?;
            for movement in &mut detail.movements {
                movement.source =
                    resolve_movement_source(&movement.reference_type, &movement.reference_id);
            }
            Ok(detail)
        }
        Err(DbError::NotFound) => Err(BizError::NotFound(format!(
            "廃棄・破損記録が見つかりません: {}",
            record_id
        ))),
        Err(err) => Err(BizError::DatabaseError(err)),
    }
}

/// 在庫変動の履歴の行の種類（44 の D-118 の節。D-061 の生成の literal union）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum StockHistoryKind {
    Movement,
    Count,
}

/// 在庫変動の履歴の計数（44 の D-118 の節）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StockCountHistory {
    pub id: i64,
    pub count_value: i64,
    /// 計数の商品の今の単位（画面は計数の値をこの単位で描く。66 の UI-06c-D3 の部分障害）
    pub stock_unit: ProductStockUnit,
    pub count_kind: StockCountKind,
    pub stocktake_id: Option<i64>,
    /// `stocktake_id` があるときだけ棚卸しの記録への link
    pub source: Option<MovementSourceLink>,
}

/// 在庫変動の履歴の 1 行（44 の D-118 の節）。`kind` が `movement` なら `movement` だけ、`count` なら `count` だけが値
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StockHistoryItem {
    pub kind: StockHistoryKind,
    /// movement の業務日（Z004 は精算日）か計数の日付
    pub business_date: String,
    /// Z004 の movement の精算回数と、fresh な計数の保存の時点の精算回数（並びの鍵）。ほかは None
    pub settlement_no: Option<i64>,
    /// movement の `created_at` か計数の `counted_at`
    pub saved_at: String,
    pub movement: Option<MovementRecord>,
    pub count: Option<StockCountHistory>,
}

/// 並びの鍵 `(business_date, phase, settlement_no または -1, after_settlement)`（ADR SPEC-STK-CNT-D1）に、
/// 全順序にする `saved_at`・種類（movement 0 / 計数 1）・id を足したもの
type HistoryKey = (String, i64, i64, i64, String, u8, i64);

/// 在庫変動の履歴（movement と計数）を並びの鍵の新しい順に返す
///
/// 44-cmd-inventory.md §23.10 と D-118 の b2 の節。ページの検査 → 日付の検査 → 2 つの読取り →
/// 鍵で並べる → 件数 → ページを切る → `source` を補う
pub fn list_movements(
    conn: &DbConnection,
    query: &MovementQuery,
) -> Result<PaginatedResult<StockHistoryItem>, BizError> {
    if query.page < 1 || query.per_page < 1 || query.per_page > MAX_PER_PAGE {
        return Err(BizError::ValidationFailed(
            "ページパラメータが不正です".to_string(),
        ));
    }
    for date in [&query.date_from, &query.date_to].into_iter().flatten() {
        super::validate_business_date(date)?;
    }

    let mut rows: Vec<(HistoryKey, StockHistoryItem)> =
        inventory_repo::list_history_movements(conn, query)?
            .into_iter()
            .map(|row| {
                let key = (
                    row.business_date.clone(),
                    i64::from(row.settlement_no.is_some()),
                    row.settlement_no.unwrap_or(-1),
                    if row.initial_stock { -1 } else { 0 },
                    row.movement.created_at.clone(),
                    0,
                    row.movement.id,
                );
                let item = StockHistoryItem {
                    kind: StockHistoryKind::Movement,
                    business_date: row.business_date,
                    settlement_no: row.settlement_no,
                    saved_at: row.movement.created_at.clone(),
                    movement: Some(row.movement),
                    count: None,
                };
                (key, item)
            })
            .collect();
    // 計数は「すべて」と「棚卸し」でだけ含める（44）
    if matches!(query.movement_type, None | Some(MovementType::Stocktake)) {
        rows.extend(
            inventory_repo::list_history_counts(conn, query)?
                .into_iter()
                .map(|row| {
                    // fresh は (T, 1, n, 1)、n が NULL なら 3 つ目を -1。既定は (D, 0, NULL, 0)
                    let fresh = i64::from(row.fresh);
                    let key = (
                        row.business_date.clone(),
                        fresh,
                        row.settlement_no.unwrap_or(-1),
                        fresh,
                        row.counted_at.clone(),
                        1,
                        row.id,
                    );
                    let item = StockHistoryItem {
                        kind: StockHistoryKind::Count,
                        business_date: row.business_date,
                        settlement_no: row.settlement_no,
                        saved_at: row.counted_at,
                        movement: None,
                        count: Some(StockCountHistory {
                            id: row.id,
                            count_value: row.count_value,
                            stock_unit: row.stock_unit,
                            count_kind: if row.fresh {
                                StockCountKind::Fresh
                            } else {
                                StockCountKind::Default
                            },
                            stocktake_id: row.stocktake_id,
                            source: None,
                        }),
                    };
                    (key, item)
                }),
        );
    }
    // 鍵の降順（昇順の全順序を逆にしたもの）
    rows.sort_by(|a, b| b.0.cmp(&a.0));

    let total_count = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    let offset = (query.page as usize - 1).saturating_mul(query.per_page as usize);
    let items = rows
        .into_iter()
        .skip(offset)
        .take(query.per_page as usize)
        .map(|(_, mut item)| {
            if let Some(movement) = item.movement.as_mut() {
                movement.source =
                    resolve_movement_source(&movement.reference_type, &movement.reference_id);
            }
            if let Some(count) = item.count.as_mut() {
                count.source =
                    resolve_movement_source(&Some(ReferenceType::Stocktake), &count.stocktake_id);
            }
            item
        })
        .collect();
    Ok(PaginatedResult {
        items,
        total_count,
        page: query.page,
        per_page: query.per_page,
    })
}

/// movement の reference から元業務記録の表示情報を解決する。
///
/// 65-inventory-record-traceability.md TRACE-D2 / §65.3 / §65.8.2
pub(crate) fn resolve_movement_source(
    reference_type: &Option<ReferenceType>,
    reference_id: &Option<i64>,
) -> Option<MovementSourceLink> {
    let reference_type = reference_type.as_ref()?;
    let reference_id = (*reference_id)?;
    let (label_prefix, route_prefix) = match reference_type {
        ReferenceType::ReceivingRecord => ("入庫記録", "/inventory/receiving/records"),
        ReferenceType::ReturnRecord => ("返品・交換", "/inventory/return/records"),
        ReferenceType::ManualSale => ("手動販売出庫", "/inventory/manual-sale/records"),
        ReferenceType::DisposalRecord => ("廃棄・破損", "/inventory/disposal/records"),
        ReferenceType::CsvImport => ("CSV取込み", "/csv-import/records"),
        ReferenceType::Stocktake => ("棚卸し", "/stocktake/records"),
    };

    Some(MovementSourceLink {
        label: format!("{} #{}", label_prefix, reference_id),
        route: format!("{}/{}", route_prefix, reference_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_db() -> DbConnection {
        let conn = db::init_database(":memory:").unwrap();
        db::migration::migrate(&conn).unwrap();
        conn
    }

    fn seed_product_with_stock(conn: &DbConnection, code: &str, stock: i64, unit: &str) {
        let product = crate::db::product_repo::NewProduct {
            product_code: code.to_string(),
            jan_code: None,
            name: format!("テスト商品_{}", code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: stock,
            stock_unit: unit.to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        crate::db::product_repo::insert_product(conn, &product).unwrap();
    }

    #[test]
    fn test_list_receivings_req201_empty_table() {
        // REQ-201: 入庫記録 — 入庫記録一覧: 空テーブル → 0件
        let conn = setup_db();
        let query = ListQuery {
            page: 1,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let result = list_receivings(&conn, &query).unwrap();
        assert_eq!(result.total_count, 0);
        assert!(result.items.is_empty());
        assert_eq!(result.page, 1);
        assert_eq!(result.per_page, 10);
    }

    #[test]
    fn test_list_returns_req202_empty_table() {
        // REQ-202: 返品・交換記録 — 返品記録一覧: 空テーブル → 0件
        let conn = setup_db();
        let query = ListQuery {
            page: 1,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let result = list_returns(&conn, &query).unwrap();
        assert_eq!(result.total_count, 0);
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_list_disposals_req204_empty_table() {
        // REQ-204: 廃棄・破損記録 — 廃棄記録一覧: 空テーブル → 0件
        let conn = setup_db();
        let query = ListQuery {
            page: 1,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let result = list_disposals(&conn, &query).unwrap();
        assert_eq!(result.total_count, 0);
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_list_receivings_req201_per_page_exceeds_max() {
        // REQ-201: 入庫記録 — 入庫記録一覧: per_page 上限超え → バリデーションエラー
        let conn = setup_db();
        let query = ListQuery {
            page: 1,
            per_page: 201,
            date_from: None,
            date_to: None,
        };
        let err = list_receivings(&conn, &query).unwrap_err();
        match err {
            BizError::ValidationFailed(msg) => {
                assert!(msg.contains("ページパラメータ"));
            }
            _ => panic!("expected ValidationFailed, got {:?}", err),
        }
    }

    #[test]
    fn test_list_returns_req202_page_zero() {
        // REQ-202: 返品・交換記録 — 返品記録一覧: page=0 → バリデーションエラー
        let conn = setup_db();
        let query = ListQuery {
            page: 0,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let err = list_returns(&conn, &query).unwrap_err();
        match err {
            BizError::ValidationFailed(_) => {}
            _ => panic!("expected ValidationFailed, got {:?}", err),
        }
    }

    #[test]
    fn test_list_disposals_req204_per_page_zero() {
        // REQ-204: 廃棄・破損記録 — 廃棄記録一覧: per_page=0 → バリデーションエラー
        let conn = setup_db();
        let query = ListQuery {
            page: 1,
            per_page: 0,
            date_from: None,
            date_to: None,
        };
        let err = list_disposals(&conn, &query).unwrap_err();
        match err {
            BizError::ValidationFailed(_) => {}
            _ => panic!("expected ValidationFailed, got {:?}", err),
        }
    }

    #[test]
    fn test_list_movements_req303_valid_movement_type() {
        // REQ-303: 在庫変動履歴 — 有効な movement_type → バリデーション通過（空結果OK）
        let conn = setup_db();
        let query = MovementQuery {
            product_code: "TEST-001".to_string(),
            date_from: None,
            date_to: None,
            movement_type: Some(crate::db::inventory_repo::MovementType::Receiving),
            page: 1,
            per_page: 10,
        };
        // 有効な movement_type ならバリデーションを通過（空結果OK）
        let result = list_movements(&conn, &query).unwrap();
        assert_eq!(result.total_count, 0);
    }

    #[test]
    fn test_list_movements_accepts_per_page_200() {
        let conn = setup_db();
        let query = MovementQuery {
            product_code: "TEST-001".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 200,
        };

        let result = list_movements(&conn, &query).unwrap();

        assert_eq!(result.per_page, 200);
    }

    #[test]
    fn test_resolve_movement_source_req207_known_references() {
        // REQ-207 / TRACE-D2: movement reference から元業務記録 link を解決する
        let cases = [
            (
                ReferenceType::ReceivingRecord,
                42,
                "入庫記録 #42",
                "/inventory/receiving/records/42",
            ),
            (
                ReferenceType::ReturnRecord,
                7,
                "返品・交換 #7",
                "/inventory/return/records/7",
            ),
            (
                ReferenceType::ManualSale,
                9,
                "手動販売出庫 #9",
                "/inventory/manual-sale/records/9",
            ),
            (
                ReferenceType::DisposalRecord,
                11,
                "廃棄・破損 #11",
                "/inventory/disposal/records/11",
            ),
            (
                ReferenceType::CsvImport,
                3,
                "CSV取込み #3",
                "/csv-import/records/3",
            ),
            (
                ReferenceType::Stocktake,
                5,
                "棚卸し #5",
                "/stocktake/records/5",
            ),
        ];

        for (reference_type, reference_id, expected_label, expected_route) in cases {
            let source = resolve_movement_source(&Some(reference_type), &Some(reference_id))
                .expect("known reference should resolve");
            assert_eq!(source.label, expected_label);
            assert_eq!(source.route, expected_route);
        }
    }

    #[test]
    fn test_resolve_movement_source_req303_null_reference() {
        // REQ-303: 初期在庫など NULL reference の movement は source なしで表示可能
        assert!(resolve_movement_source(&None, &Some(1)).is_none());
        assert!(resolve_movement_source(&Some(ReferenceType::ReceivingRecord), &None).is_none());
    }

    #[test]
    fn test_list_movements_req303_includes_source_link() {
        // REQ-303 / TRACE-D2: list_movements は既存 reference に加えて source link を返す
        let conn = setup_db();
        seed_product_with_stock(&conn, "MV-SRC", 10, "pcs");
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, reference_type, reference_id, is_voided, created_at, business_date, phase) \
             VALUES ('MV-SRC', 'receiving', 5, 15, 'receiving_record', 42, 0, '2026-03-15T10:00:00', '2026-04-01', 0)",
            [],
        )
        .unwrap();

        let query = MovementQuery {
            product_code: "MV-SRC".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 50,
        };
        let result = list_movements(&conn, &query).unwrap();

        let movement = result.items[0].movement.as_ref().expect("movement row");
        let source = movement.source.as_ref().expect("source link");
        assert_eq!(source.label, "入庫記録 #42");
        assert_eq!(source.route, "/inventory/receiving/records/42");
    }

    #[test]
    fn test_list_movements_req303_null_reference_has_no_source() {
        // REQ-303: reference_type/reference_id が NULL の既存行も source=None で残す
        let conn = setup_db();
        seed_product_with_stock(&conn, "MV-NULL", 10, "pcs");
        conn.execute(
            "INSERT INTO inventory_movements \
             (product_code, movement_type, quantity, stock_after, is_voided, created_at, business_date, phase) \
             VALUES ('MV-NULL', 'receiving', 5, 15, 0, '2026-03-15T10:00:00', '2026-04-01', 0)",
            [],
        )
        .unwrap();

        let query = MovementQuery {
            product_code: "MV-NULL".to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 50,
        };
        let result = list_movements(&conn, &query).unwrap();

        assert_eq!(result.total_count, 1);
        assert!(result.items[0]
            .movement
            .as_ref()
            .expect("movement row")
            .source
            .is_none());
    }

    // ===== D-118 b2: 在庫変動の履歴（movement と計数。Matrix H1〜H9） =====

    /// movement を 1 行入れる（phase は settlement_no から。id を返す）
    #[allow(clippy::too_many_arguments)]
    fn insert_history_movement(
        conn: &DbConnection,
        code: &str,
        movement_type: &str,
        quantity: i64,
        reference: Option<(&str, i64)>,
        business_date: &str,
        settlement_no: Option<i64>,
        created_at: &str,
    ) -> i64 {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 reference_type, reference_id, is_voided, created_at, business_date, phase, settlement_no)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, 0, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                code,
                movement_type,
                quantity,
                reference.map(|r| r.0),
                reference.map(|r| r.1),
                created_at,
                business_date,
                i64::from(settlement_no.is_some()),
                settlement_no,
            ],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// 計数を 1 行入れる（fresh は seen_movement_id を持つ。id を返す）
    #[allow(clippy::too_many_arguments)]
    fn insert_history_count(
        conn: &DbConnection,
        code: &str,
        value: i64,
        fresh: bool,
        business_date: &str,
        settlement_no: Option<i64>,
        counted_at: &str,
        stocktake_item_id: Option<i64>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                 settlement_no, seen_movement_id, stocktake_item_id, counted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                code,
                value,
                if fresh { "fresh" } else { "default" },
                business_date,
                settlement_no,
                fresh.then_some(0_i64),
                stocktake_item_id,
                counted_at,
            ],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn history_query(code: &str) -> MovementQuery {
        MovementQuery {
            product_code: code.to_string(),
            date_from: None,
            date_to: None,
            movement_type: None,
            page: 1,
            per_page: 200,
        }
    }

    /// 行を (種類, id) にする（movement は "m"、計数は "c"）
    fn history_ids(items: &[StockHistoryItem]) -> Vec<(&'static str, i64)> {
        items
            .iter()
            .map(|item| match (item.kind, &item.movement, &item.count) {
                (StockHistoryKind::Movement, Some(m), None) => ("m", m.id),
                (StockHistoryKind::Count, None, Some(c)) => ("c", c.id),
                other => panic!("kind と片方だけの値が食い違う: {other:?}"),
            })
            .collect()
    }

    const D1: &str = "2026-10-01";
    const D2: &str = "2026-10-02";

    /// Matrix H の fixture の ids（保存の順に入れる。保存の時刻の順と鍵の順が違う形）
    struct HistoryFixture {
        initial: i64,
        z101: i64,
        default9: i64,
        receiving: i64,
        z102_voided: i64,
        fresh10: i64,
        default8: i64,
        sale_auto: i64,
        z102_again: i64,
    }

    fn seed_history_fixture(conn: &DbConnection, code: &str) -> HistoryFixture {
        seed_product_with_stock(conn, code, 0, "pcs");
        let initial = insert_history_movement(
            conn,
            code,
            "receiving",
            10,
            None,
            D2,
            None,
            "2026-10-02T09:00:01",
        );
        let z101 = insert_history_movement(
            conn,
            code,
            "sale_auto",
            -1,
            Some(("csv_import", 1)),
            D1,
            Some(101),
            "2026-10-02T09:00:02",
        );
        let default9 =
            insert_history_count(conn, code, 9, false, D2, None, "2026-10-02T09:00:03", None);
        let receiving = insert_history_movement(
            conn,
            code,
            "receiving",
            2,
            Some(("receiving_record", 7)),
            D2,
            None,
            "2026-10-02T09:00:04",
        );
        let z102_voided = insert_history_movement(
            conn,
            code,
            "sale_auto",
            -1,
            Some(("csv_import", 2)),
            D2,
            Some(102),
            "2026-10-02T09:00:05",
        );
        let fresh10 = insert_history_count(
            conn,
            code,
            10,
            true,
            D2,
            Some(102),
            "2026-10-02T09:00:06",
            None,
        );
        let default8 =
            insert_history_count(conn, code, 8, false, D1, None, "2026-10-02T09:00:07", None);
        let sale_auto = insert_history_movement(
            conn,
            code,
            "sale_auto",
            -1,
            None,
            D2,
            None,
            "2026-10-02T09:00:08",
        );
        conn.execute(
            "UPDATE inventory_movements SET is_voided = 1 WHERE id = ?1",
            [z102_voided],
        )
        .unwrap();
        let z102_again = insert_history_movement(
            conn,
            code,
            "sale_auto",
            -1,
            Some(("csv_import", 3)),
            D2,
            Some(102),
            "2026-10-02T09:00:09",
        );
        HistoryFixture {
            initial,
            z101,
            default9,
            receiving,
            z102_voided,
            fresh10,
            default8,
            sale_auto,
            z102_again,
        }
    }

    /// H1 の fixture の期待の並び（ADR D1 の鍵から手で求めた新しい順）
    fn expected_history_order(f: &HistoryFixture) -> Vec<(&'static str, i64)> {
        vec![
            ("c", f.fresh10),    // (D2,1,102,1)
            ("m", f.z102_again), // (D2,1,102,0)
            ("m", f.sale_auto),  // (D2,0,NULL,0) 保存 09:00:08
            ("m", f.receiving),  // (D2,0,NULL,0) 保存 09:00:04
            ("c", f.default9),   // (D2,0,NULL,0) 保存 09:00:03
            ("m", f.initial),    // (D2,0,NULL,-1)
            ("m", f.z101),       // (D1,1,101,0)
            ("c", f.default8),   // (D1,0,NULL,0)
        ]
    }

    fn table_rows(conn: &DbConnection) -> (i64, i64) {
        let count = |table: &str| -> i64 {
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
        };
        (count("inventory_movements"), count("stock_counts"))
    }

    #[test]
    fn test_list_movements_req303_req205_orders_counts_and_movements_by_ledger_key() {
        // REQ-303 / REQ-205 / SPEC-STK-CNT-D1・D9（44 の D-118 の b2 の節、65）: movement と計数を 1 つの履歴に混ぜ、
        // 鍵の降順で返す。取り消した movement は出ない（H6）、初期在庫の印は receiving だけ（H8）。Matrix H1
        let conn = setup_db();
        let f = seed_history_fixture(&conn, "H-1");
        let before = table_rows(&conn);

        let result = list_movements(&conn, &history_query("H-1")).unwrap();

        assert_eq!(history_ids(&result.items), expected_history_order(&f));
        assert_eq!(result.total_count, 8);
        assert!(!history_ids(&result.items).contains(&("m", f.z102_voided)));
        // 行の値（業務日・精算回数・保存の時刻・種類）
        let fresh = &result.items[0];
        assert_eq!(fresh.kind, StockHistoryKind::Count);
        assert_eq!(
            (
                fresh.business_date.as_str(),
                fresh.settlement_no,
                fresh.saved_at.as_str()
            ),
            (D2, Some(102), "2026-10-02T09:00:06")
        );
        let count = fresh.count.as_ref().unwrap();
        assert_eq!(count.count_value, 10);
        assert_eq!(count.count_kind, StockCountKind::Fresh);
        assert_eq!(count.stock_unit, ProductStockUnit::Pcs);
        let z101 = &result.items[6];
        assert_eq!(z101.kind, StockHistoryKind::Movement);
        assert_eq!(
            (
                z101.business_date.as_str(),
                z101.settlement_no,
                z101.saved_at.as_str()
            ),
            (D1, Some(101), "2026-10-02T09:00:02")
        );
        let default8 = &result.items[7];
        assert_eq!(
            (default8.business_date.as_str(), default8.settlement_no),
            (D1, None)
        );
        assert_eq!(
            default8.count.as_ref().unwrap().count_kind,
            StockCountKind::Default
        );
        // 読取りだけで何も書かない
        assert_eq!(table_rows(&conn), before);

        // 別の商品: 取込みの無いときに保存した fresh な計数（n が NULL）の後に精算 0 の Z004 を取り込んだ
        seed_product_with_stock(&conn, "H-1B", 0, "pcs");
        let fresh_null = insert_history_count(
            &conn,
            "H-1B",
            5,
            true,
            D2,
            None,
            "2026-10-02T10:00:00",
            None,
        );
        let z0 = insert_history_movement(
            &conn,
            "H-1B",
            "sale_auto",
            -1,
            Some(("csv_import", 4)),
            D2,
            Some(0),
            "2026-10-02T11:00:00",
        );
        let result = list_movements(&conn, &history_query("H-1B")).unwrap();
        assert_eq!(
            history_ids(&result.items),
            vec![("m", z0), ("c", fresh_null)]
        );

        // 計数の無い商品は movement だけの履歴
        seed_product_with_stock(&conn, "H-1C", 0, "pcs");
        let only = insert_history_movement(
            &conn,
            "H-1C",
            "receiving",
            3,
            None,
            D1,
            None,
            "2026-10-01T09:00:00",
        );
        let result = list_movements(&conn, &history_query("H-1C")).unwrap();
        assert_eq!(history_ids(&result.items), vec![("m", only)]);
    }

    #[test]
    fn test_list_movements_req303_same_key_rows_break_ties_by_saved_at_kind_and_id() {
        // REQ-303 / SPEC-STK-CNT-D1（44）: 鍵の同じ行は saved_at・movement を計数より前・id の昇順を逆にした順。Matrix H2
        let conn = setup_db();
        seed_product_with_stock(&conn, "H-2", 0, "pcs");
        let same = "2026-10-02T09:00:00";
        let count_a = insert_history_count(&conn, "H-2", 4, false, D2, None, same, None);
        // 入庫は reference を付ける（NULL だと初期在庫の印で鍵が変わる）
        let receiving = |id: i64, saved_at: &str| {
            insert_history_movement(
                &conn,
                "H-2",
                "receiving",
                1,
                Some(("receiving_record", id)),
                D2,
                None,
                saved_at,
            )
        };
        let mv_a = receiving(1, same);
        let mv_b = receiving(2, same);
        let count_b = insert_history_count(&conn, "H-2", 5, false, D2, None, same, None);
        let later = receiving(3, "2026-10-02T09:00:01");
        let early = insert_history_count(
            &conn,
            "H-2",
            6,
            false,
            D2,
            None,
            "2026-10-02T08:59:59",
            None,
        );

        let result = list_movements(&conn, &history_query("H-2")).unwrap();

        assert_eq!(
            history_ids(&result.items),
            vec![
                ("m", later),
                ("c", count_b),
                ("c", count_a),
                ("m", mv_b),
                ("m", mv_a),
                ("c", early),
            ]
        );
    }

    #[test]
    fn test_list_movements_req303_date_filter_uses_business_date_for_both_kinds() {
        // REQ-303（44 の D-118 の b2 の節）: 日付の絞り込みは業務日の両端を含み、movement と計数の両方に当たる。Matrix H3
        let conn = setup_db();
        seed_product_with_stock(&conn, "H-3", 0, "pcs");
        let d3 = "2026-10-03";
        let mv = insert_history_movement(
            &conn,
            "H-3",
            "receiving",
            1,
            Some(("receiving_record", 1)),
            D1,
            None,
            "2026-10-03T09:00:00",
        );
        let count = insert_history_count(
            &conn,
            "H-3",
            4,
            false,
            D1,
            None,
            "2026-10-03T10:00:00",
            None,
        );
        let query = |from: &str, to: &str| MovementQuery {
            date_from: Some(from.to_string()),
            date_to: Some(to.to_string()),
            ..history_query("H-3")
        };

        let on_d1 = list_movements(&conn, &query(D1, D1)).unwrap();
        assert_eq!(history_ids(&on_d1.items), vec![("c", count), ("m", mv)]);
        assert_eq!(on_d1.total_count, 2);
        let on_d3 = list_movements(&conn, &query(d3, d3)).unwrap();
        assert!(on_d3.items.is_empty());
        assert_eq!(on_d3.total_count, 0);
        // 両端を含む（from だけ・to だけでも同じ軸）
        let from_d1 = list_movements(&conn, &query(D1, d3)).unwrap();
        assert_eq!(from_d1.total_count, 2);
        let until_d1 = list_movements(
            &conn,
            &MovementQuery {
                date_to: Some(D1.to_string()),
                ..history_query("H-3")
            },
        )
        .unwrap();
        assert_eq!(until_d1.total_count, 2);
        let from_d2 = list_movements(
            &conn,
            &MovementQuery {
                date_from: Some(D2.to_string()),
                ..history_query("H-3")
            },
        )
        .unwrap();
        assert_eq!(from_d2.total_count, 0);
    }

    #[test]
    fn test_list_movements_req303_type_filter_includes_counts_only_for_all_and_stocktake() {
        // REQ-303（44 の D-118 の b2 の節）: 種別が None なら全部、stocktake なら棚卸しの movement と計数、
        // ほかの種別は movement だけ。Matrix H4
        let conn = setup_db();
        let f = seed_history_fixture(&conn, "H-4");
        let stocktake_mv = insert_history_movement(
            &conn,
            "H-4",
            "stocktake",
            1,
            Some(("stocktake", 1)),
            D1,
            None,
            "2026-10-02T09:00:10",
        );
        let with_type = |movement_type: Option<MovementType>| MovementQuery {
            movement_type,
            ..history_query("H-4")
        };

        let all = list_movements(&conn, &with_type(None)).unwrap();
        assert_eq!(all.total_count, 9);
        assert!(history_ids(&all.items).contains(&("c", f.default8)));
        assert!(history_ids(&all.items).contains(&("m", stocktake_mv)));

        let stocktake = list_movements(&conn, &with_type(Some(MovementType::Stocktake))).unwrap();
        assert_eq!(
            history_ids(&stocktake.items),
            vec![
                ("c", f.fresh10),
                ("c", f.default9),
                ("m", stocktake_mv),
                ("c", f.default8),
            ]
        );
        assert_eq!(stocktake.total_count, 4);

        let receiving = list_movements(&conn, &with_type(Some(MovementType::Receiving))).unwrap();
        assert_eq!(
            history_ids(&receiving.items),
            vec![("m", f.receiving), ("m", f.initial)]
        );
        assert_eq!(receiving.total_count, 2);
    }

    #[test]
    fn test_list_movements_req303_paginates_after_merging_and_counts_both_kinds() {
        // REQ-303（44 の D-118 の b2 の節）: 混ぜて並べた後にページを切り、件数は movement と計数の合計。Matrix H5
        let conn = setup_db();
        let f = seed_history_fixture(&conn, "H-5");
        let page = |page: u32| {
            list_movements(
                &conn,
                &MovementQuery {
                    page,
                    per_page: 3,
                    ..history_query("H-5")
                },
            )
            .unwrap()
        };

        let mut joined = Vec::new();
        for n in 1..=3 {
            let result = page(n);
            assert_eq!(result.total_count, 8);
            assert_eq!((result.page, result.per_page), (n, 3));
            joined.extend(history_ids(&result.items));
        }
        assert_eq!(joined, expected_history_order(&f));
        assert!(page(4).items.is_empty());
    }

    #[test]
    fn test_list_movements_req207_count_source_links_to_stocktake_record() {
        // REQ-207 / TRACE-D2（44・65）: 明細を参照する計数は棚卸しの記録への link、参照しない計数は無し。Matrix H7
        let conn = setup_db();
        seed_product_with_stock(&conn, "H-7", 0, "pcs");
        let mut item_ids = Vec::new();
        let mut stocktake_ids = Vec::new();
        for items in [2, 1] {
            conn.execute(
                "INSERT INTO stocktakes (started_at, completed_at, status) VALUES ('2026-10-01T09:00:00', '2026-10-01T18:00:00', 'completed')",
                [],
            )
            .unwrap();
            let stocktake_id = conn.last_insert_rowid();
            stocktake_ids.push(stocktake_id);
            for _ in 0..items {
                conn.execute(
                    "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
                     VALUES (?1, 'H-7', 0, 3)",
                    [stocktake_id],
                )
                .unwrap();
                item_ids.push(conn.last_insert_rowid());
            }
        }
        let second_stocktake = stocktake_ids[1];
        let referenced_item = item_ids[2];
        assert_ne!(
            second_stocktake, referenced_item,
            "fixture の前提: 2 つの id が違う"
        );
        let linked = insert_history_count(
            &conn,
            "H-7",
            3,
            false,
            D1,
            None,
            "2026-10-01T10:00:00",
            Some(referenced_item),
        );
        let unlinked =
            insert_history_count(&conn, "H-7", 3, true, D1, None, "2026-10-01T11:00:00", None);
        let mv = insert_history_movement(
            &conn,
            "H-7",
            "disposal",
            -1,
            Some(("disposal_record", 15)),
            D1,
            None,
            "2026-10-01T12:00:00",
        );

        let result = list_movements(&conn, &history_query("H-7")).unwrap();

        assert_eq!(
            history_ids(&result.items),
            vec![("c", unlinked), ("m", mv), ("c", linked)]
        );
        let unlinked_count = result.items[0].count.as_ref().unwrap();
        assert_eq!(unlinked_count.stocktake_id, None);
        assert!(unlinked_count.source.is_none());
        let movement_source = result.items[1]
            .movement
            .as_ref()
            .unwrap()
            .source
            .as_ref()
            .unwrap();
        assert_eq!(movement_source.label, "廃棄・破損 #15");
        let linked_count = result.items[2].count.as_ref().unwrap();
        assert_eq!(linked_count.stocktake_id, Some(second_stocktake));
        let source = linked_count.source.as_ref().expect("棚卸しの記録への link");
        assert_eq!(source.label, format!("棚卸し #{second_stocktake}"));
        assert_eq!(
            source.route,
            format!("/stocktake/records/{second_stocktake}")
        );
    }

    #[test]
    fn test_list_movements_req303_rejects_invalid_date_filter() {
        // REQ-303（44・31 の validate_business_date）: 形・暦の外の日付の絞り込みは ValidationFailed。Matrix H9
        let conn = setup_db();
        seed_history_fixture(&conn, "H-9");
        for (from, to, message) in [
            (Some("2026-4-7"), None, "日付の形式が不正です（YYYY-MM-DD）"),
            (None, Some("2026-02-30"), "存在しない日付です"),
        ] {
            let err = list_movements(
                &conn,
                &MovementQuery {
                    date_from: from.map(str::to_string),
                    date_to: to.map(str::to_string),
                    ..history_query("H-9")
                },
            )
            .unwrap_err();
            match err {
                BizError::ValidationFailed(msg) => assert_eq!(msg, message),
                other => panic!("expected ValidationFailed, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_list_inventory_records_req206_rejects_invalid_page_params() {
        // REQ-206 / TRACE-D1: 入出庫履歴ハブは per_page 上限200をBIZで検証する
        let conn = setup_db();
        let query = crate::db::disposal_repo::InventoryRecordQuery {
            record_type: Some("disposal_record".to_string()),
            date_from: None,
            date_to: None,
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: None,
            page: 1,
            per_page: 201,
        };

        let err = list_inventory_records(&conn, &query).unwrap_err();

        assert!(matches!(err, BizError::ValidationFailed(_)));
    }

    #[test]
    fn test_list_inventory_records_accepts_per_page_200() {
        let conn = setup_db();
        let query = crate::db::disposal_repo::InventoryRecordQuery {
            record_type: None,
            date_from: None,
            date_to: None,
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: None,
            page: 1,
            per_page: 200,
        };

        let result = list_inventory_records(&conn, &query).unwrap();

        assert_eq!(result.per_page, 200);
    }

    #[test]
    fn test_list_inventory_records_req206_rejects_unknown_record_type() {
        // REQ-206: 未対応のrecord_typeはBIZ validationとして拒否する
        let conn = setup_db();
        let query = crate::db::disposal_repo::InventoryRecordQuery {
            record_type: Some("unknown_type".to_string()),
            date_from: None,
            date_to: None,
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: None,
            page: 1,
            per_page: 20,
        };

        let err = list_inventory_records(&conn, &query).unwrap_err();

        assert!(matches!(err, BizError::ValidationFailed(msg) if msg.contains("記録種別")));
    }

    #[test]
    fn test_list_inventory_records_req206_allowlist_six_types_four_statuses() {
        // REQ-206 / 44-cmd §23.10: 値域だけを6種・4値へ拡張し未知値は拒否する
        let conn = setup_db();
        for record_type in ["csv_import", "stocktake"] {
            let query = crate::db::disposal_repo::InventoryRecordQuery {
                record_type: Some(record_type.to_string()),
                date_from: None,
                date_to: None,
                record_id: None,
                product_keyword: None,
                department_id: None,
                status: None,
                page: 1,
                per_page: 20,
            };
            assert!(list_inventory_records(&conn, &query).is_ok());
        }
        for status in ["all", "active", "canceled", "in_progress"] {
            let query = crate::db::disposal_repo::InventoryRecordQuery {
                record_type: Some("all".to_string()),
                date_from: None,
                date_to: None,
                record_id: None,
                product_keyword: None,
                department_id: None,
                status: Some(status.to_string()),
                page: 1,
                per_page: 20,
            };
            assert!(list_inventory_records(&conn, &query).is_ok());
        }

        let query = crate::db::disposal_repo::InventoryRecordQuery {
            record_type: Some("all".to_string()),
            date_from: None,
            date_to: None,
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: Some("unknown_status".to_string()),
            page: 1,
            per_page: 20,
        };
        let error = list_inventory_records(&conn, &query).unwrap_err();
        assert!(matches!(error, BizError::ValidationFailed(msg) if msg.contains("状態")));
    }

    #[test]
    fn test_get_disposal_record_req204_maps_missing_to_not_found() {
        // REQ-204 / REQ-206: 廃棄・破損詳細の未存在IDはBIZ NotFoundに変換する
        let conn = setup_db();

        let err = get_disposal_record(&conn, 404).unwrap_err();

        assert!(matches!(err, BizError::NotFound(msg) if msg.contains("廃棄・破損")));
    }

    // -----------------------------------------------------------------------
    // 記録詳細の行の金額と合計（SPEC-UNIT-D6、X7）と wire の単位（SPEC-UNIT-D11）
    // -----------------------------------------------------------------------

    use crate::biz::inventory_service::{
        create_disposal, create_receiving, DisposalCreateRequest, ReceivingCreateRequest,
    };
    use crate::db::product_repo::ProductStockUnit;

    /// (商品コード, 数量, 原価) の明細で入庫して記録詳細を返す
    fn receive(
        conn: &mut DbConnection,
        key: &str,
        lines: &[(&str, i64, i64)],
    ) -> ReceivingRecordDetail {
        let items = serde_json::json!(lines
            .iter()
            .map(|(code, quantity, cost)| {
                serde_json::json!({"product_code": code, "quantity": quantity, "cost_price_centi": cost})
            })
            .collect::<Vec<_>>());
        let req: ReceivingCreateRequest = serde_json::from_value(serde_json::json!({
            "idempotency_key": key,
            "supplier_id": null,
            "receiving_date": "2026-10-01",
            "note": null,
            "items": items,
        }))
        .unwrap();
        let result = create_receiving(conn, req).unwrap();
        get_receiving_record(conn, result.record_id).unwrap()
    }

    fn dispose(
        conn: &mut DbConnection,
        key: &str,
        lines: &[(&str, i64, i64)],
    ) -> DisposalRecordDetail {
        let items = serde_json::json!(lines
            .iter()
            .map(|(code, quantity, cost)| {
                serde_json::json!({
                    "product_code": code, "disposal_type": "damage", "quantity": quantity,
                    "cost_price_centi": cost, "reason": "合成の理由",
                })
            })
            .collect::<Vec<_>>());
        let req: DisposalCreateRequest = serde_json::from_value(serde_json::json!({
            "idempotency_key": key,
            "disposal_date": "2026-10-01",
            "items": items,
        }))
        .unwrap();
        let result = create_disposal(conn, req).unwrap();
        get_disposal_record(conn, result.record_id).unwrap()
    }

    fn seed_amount_products(conn: &DbConnection) {
        seed_product_with_stock(conn, "AM-M", 100_000, "m");
        seed_product_with_stock(conn, "AM-CM", 100_000, "m");
        seed_product_with_stock(conn, "AM-PCS", 0, "pcs");
        seed_product_with_stock(conn, "AM-PCS2", 100, "pcs");
    }

    #[test]
    fn test_get_receiving_record_req201_line_cost_centi_from_biz() {
        // REQ-201 / REQ-206 / SPEC-UNIT-D6（X7）: 行の金額は BIZ が基準数量で割って求め、1/100 円の文字列で返す
        let mut conn = setup_db();
        seed_amount_products(&conn);

        // 単位の混ざった伝票: 行ごとに基準数量で割り、合計は 1/100 円で足してから丸める（174 ではない）
        let mixed = receive(
            &mut conn,
            "am-mixed",
            &[("AM-M", 50, 33_300), ("AM-CM", 50, 100), ("AM-PCS", 3, 200)],
        );
        let lines: Vec<&str> = mixed
            .items
            .iter()
            .map(|item| item.line_cost_centi.as_str())
            .collect();
        assert_eq!(lines, vec!["16650", "50", "600"]);
        assert_eq!(mixed.total_cost, 173);
        assert_eq!(mixed.items[0].stock_unit, ProductStockUnit::M);

        // 長さの商品 2.5 m・原価 333 円（33300）
        let length = receive(&mut conn, "am-length", &[("AM-M", 250, 33_300)]);
        assert_eq!(length.items[0].line_cost_centi, "83250");
        assert_eq!(length.total_cost, 833);

        // 個数の商品（数量 4・原価 250 円〈25000〉）は今と同じ額
        let count = receive(&mut conn, "am-count", &[("AM-PCS", 4, 25_000)]);
        assert_eq!(count.items[0].line_cost_centi, "100000");
        assert_eq!(count.total_cost, 1000);

        // 上限の明細
        let limit = receive(&mut conn, "am-limit", &[("AM-PCS", 9_999_999, 99_999_900)]);
        assert_eq!(limit.items[0].line_cost_centi, "999998900000100");
        assert_eq!(limit.total_cost, 9_999_989_000_001);
    }

    #[test]
    fn test_get_disposal_record_req204_line_loss_cost_centi_from_biz() {
        // REQ-204 / REQ-206 / SPEC-UNIT-D6（X7）: 廃棄のロス原価も同じ関数で求める
        let mut conn = setup_db();
        seed_amount_products(&conn);

        let mixed = dispose(
            &mut conn,
            "am-mixed",
            &[
                ("AM-M", 50, 33_300),
                ("AM-CM", 50, 100),
                ("AM-PCS2", 3, 200),
            ],
        );
        let lines: Vec<&str> = mixed
            .items
            .iter()
            .map(|item| item.line_loss_cost_centi.as_str())
            .collect();
        assert_eq!(lines, vec!["16650", "50", "600"]);
        assert_eq!(mixed.total_loss_cost, 173);

        // 長さの商品 0.5 m・原価 333 円（33300）
        let length = dispose(&mut conn, "am-length", &[("AM-M", 50, 33_300)]);
        assert_eq!(length.items[0].line_loss_cost_centi, "16650");
        assert_eq!(length.total_loss_cost, 167);

        // 個数の商品（数量 2・原価 120 円 と数量 3・原価 80 円）は今と同じ額
        let count = dispose(
            &mut conn,
            "am-count",
            &[("AM-PCS2", 2, 12_000), ("AM-PCS2", 3, 8_000)],
        );
        let lines: Vec<&str> = count
            .items
            .iter()
            .map(|item| item.line_loss_cost_centi.as_str())
            .collect();
        assert_eq!(lines, vec!["24000", "24000"]);
        assert_eq!(count.total_loss_cost, 480);

        let limit = dispose(&mut conn, "am-limit", &[("AM-PCS", 9_999_999, 99_999_900)]);
        assert_eq!(limit.items[0].line_loss_cost_centi, "999998900000100");
    }

    #[test]
    fn test_receiving_req201_line_cost_uses_centi_directly() {
        // REQ-201 / SPEC-UNIT-D5・D6 / Matrix C4: 1/100 円の原価をそのまま行の金額に使う（* 100 が残れば 100 倍）
        let mut conn = setup_db();
        seed_amount_products(&conn);
        let count = receive(&mut conn, "c4-count", &[("AM-PCS", 3, 8333)]);
        assert_eq!(count.items[0].cost_price_centi, 8333);
        assert_eq!(count.items[0].line_cost_centi, "24999");
        assert_eq!(count.total_cost, 250);
        // 1 m あたり 83.33 円の m の商品 1.3 m
        let length = receive(&mut conn, "c4-length", &[("AM-M", 130, 8333)]);
        assert_eq!(length.items[0].line_cost_centi, "10833");
        assert_eq!(length.total_cost, 108);
    }

    #[test]
    fn test_record_detail_req206_line_amounts_from_centi() {
        // REQ-204 / REQ-206 / SPEC-UNIT-D6 / Matrix C5: 廃棄の記録詳細も 1/100 円の原価から C4 と同じ値
        let mut conn = setup_db();
        seed_amount_products(&conn);
        let count = dispose(&mut conn, "c5-count", &[("AM-PCS2", 3, 8333)]);
        assert_eq!(count.items[0].cost_price_centi, 8333);
        assert_eq!(count.items[0].line_loss_cost_centi, "24999");
        assert_eq!(count.total_loss_cost, 250);
        let length = dispose(&mut conn, "c5-length", &[("AM-M", 130, 8333)]);
        assert_eq!(length.items[0].line_loss_cost_centi, "10833");
        assert_eq!(length.total_loss_cost, 108);
    }

    /// (商品コード, 数量, 原価) の明細の入庫・廃棄の要求（結果をそのまま返す）
    fn receiving_req(key: &str, lines: &[(String, i64, i64)]) -> ReceivingCreateRequest {
        serde_json::from_value(serde_json::json!({
            "idempotency_key": key,
            "supplier_id": null,
            "receiving_date": "2026-10-01",
            "note": null,
            "items": lines.iter().map(|(code, quantity, cost)| serde_json::json!({
                "product_code": code, "quantity": quantity, "cost_price_centi": cost,
            })).collect::<Vec<_>>(),
        }))
        .unwrap()
    }

    fn disposal_req(key: &str, lines: &[(String, i64, i64)]) -> DisposalCreateRequest {
        serde_json::from_value(serde_json::json!({
            "idempotency_key": key,
            "disposal_date": "2026-10-01",
            "items": lines.iter().map(|(code, quantity, cost)| serde_json::json!({
                "product_code": code, "disposal_type": "damage", "quantity": quantity,
                "cost_price_centi": cost, "reason": "合成の理由",
            })).collect::<Vec<_>>(),
        }))
        .unwrap()
    }

    fn table_counts(conn: &DbConnection) -> Vec<i64> {
        [
            "receiving_records",
            "receiving_items",
            "disposal_records",
            "disposal_items",
            "inventory_movements",
            "operation_logs",
        ]
        .iter()
        .map(|table| {
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
        })
        .collect()
    }

    #[test]
    fn test_receiving_disposal_req903_total_safe_integer_with_centi_limits() {
        // REQ-201 / REQ-204 / REQ-903 / 共通規則 (c) / Matrix C6: 上限の明細（数量 9999999・原価 99999999）
        // 900 行は通り（合計 8999999010000009 円）、901 行（9009999008900009 円）は合計金額が大きすぎます
        let limit_lines = |rows: usize| -> Vec<(String, i64, i64)> {
            (0..rows)
                .map(|i| (format!("C6-A{}", i / 100 + 1), 9_999_999, 99_999_999))
                .collect()
        };
        let mut conn = setup_db();
        for product in 1..=10 {
            seed_product_with_stock(&conn, &format!("C6-A{product}"), 0, "pcs");
        }
        let created =
            create_receiving(&mut conn, receiving_req("c6-r-900", &limit_lines(900))).unwrap();
        let detail = get_receiving_record(&conn, created.record_id).unwrap();
        assert_eq!(detail.total_cost, 8_999_999_010_000_009);
        let created =
            create_disposal(&mut conn, disposal_req("c6-d-900", &limit_lines(900))).unwrap();
        let detail = get_disposal_record(&conn, created.record_id).unwrap();
        assert_eq!(detail.total_loss_cost, 8_999_999_010_000_009);

        let before = table_counts(&conn);
        let receiving = create_receiving(&mut conn, receiving_req("c6-r-901", &limit_lines(901)));
        let disposal = create_disposal(&mut conn, disposal_req("c6-d-901", &limit_lines(901)));
        for result in [receiving.map(|_| ()), disposal.map(|_| ())] {
            assert!(
                matches!(result, Err(BizError::ValidationFailed(ref m)) if m == "合計金額が大きすぎます"),
                "{result:?}"
            );
        }
        assert_eq!(table_counts(&conn), before);
    }

    #[test]
    fn test_receiving_disposal_req201_req204_idempotency_precedes_cost_limit() {
        // REQ-201 / REQ-204 / 31 の処理ステップ 0・1 / Matrix C22: 保存済みの冪等キーで原価だけ上限の外に変えた要求は
        // 原価の検査より先に IdempotencyConflict で止まり、何も書かない
        let mut conn = setup_db();
        seed_product_with_stock(&conn, "C22-A", 10, "pcs");
        let line = |cost: i64| vec![("C22-A".to_string(), 1, cost)];
        create_receiving(&mut conn, receiving_req("c22-r", &line(8333))).unwrap();
        create_disposal(&mut conn, disposal_req("c22-d", &line(8333))).unwrap();
        let before = table_counts(&conn);
        let receiving = create_receiving(&mut conn, receiving_req("c22-r", &line(100_000_000)));
        assert!(
            matches!(receiving, Err(BizError::IdempotencyConflict(_))),
            "{receiving:?}"
        );
        let disposal = create_disposal(&mut conn, disposal_req("c22-d", &line(100_000_000)));
        assert!(
            matches!(disposal, Err(BizError::IdempotencyConflict(_))),
            "{disposal:?}"
        );
        assert_eq!(table_counts(&conn), before);
    }

    #[test]
    fn test_get_record_req206_unknown_stock_unit_is_read_error() {
        // REQ-206 / SPEC-UNIT-D11: DB に一覧に無い単位がある記録の詳細の読取りは error
        let mut conn = setup_db();
        seed_amount_products(&conn);
        let detail = receive(&mut conn, "am-unknown", &[("AM-PCS", 1, 100)]);
        conn.execute_batch(
            "PRAGMA ignore_check_constraints = ON;
             UPDATE products SET stock_unit = 'kg' WHERE product_code = 'AM-PCS';
             PRAGMA ignore_check_constraints = OFF;",
        )
        .unwrap();

        let result = get_receiving_record(&conn, detail.id);
        assert!(
            matches!(result, Err(BizError::DatabaseError(_))),
            "{result:?}"
        );
    }
}
