//! 入出庫の記録の取消（cancel_inventory_record）
//!
//! 31-biz-inventory-service.md §12.6c に基づく実装（D-163）。
//! 記録を消さず、header に取消の日時と理由を書き、その記録の movement（と手動販売の売上の行）を
//! 無効にして、触れた商品を求め直す（Z004 の取消と同じ形。逆向きの movement は作らない）。

use crate::biz::stock_ledger::recompute_stock;
use crate::biz::BizError;
use crate::db::inventory_common::{get_record_cancel, mark_record_canceled};
use crate::db::inventory_repo::ReferenceType;
use crate::db::{manual_sale_repo, product_repo, sales_repo, system_repo, DbConnection, DbError};

use std::collections::BTreeSet;

use super::common::{negative_stock_warning_message, StockChangeOutcome};

/// 記録の取消の結果（31 §12.6c）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct RecordCancelResult {
    /// 取消で在庫が負になった商品の文（作成と同じ文。商品コードの順・重複なし）
    pub stock_warnings: Vec<String>,
}

/// 取り消せる 5 つの記録の header の表と、文に出す記録の名前。
/// 取り消せる種類の集合はこの match だけが持つ（D-061・D-064）
fn cancelable(record_type: ReferenceType) -> Option<(&'static str, &'static str)> {
    match record_type {
        ReferenceType::ReceivingRecord => Some(("receiving_records", "入庫記録")),
        ReferenceType::ReturnRecord => Some(("return_records", "返品・交換記録")),
        ReferenceType::ManualSale => Some(("manual_sales", "手動販売記録")),
        ReferenceType::DisposalRecord => Some(("disposal_records", "廃棄・破損記録")),
        ReferenceType::TransferRecord => Some(("transfer_records", "詰め替え・切り分けの記録")),
        // CSV取込みの取消は BIZ-03 の rollback_csv_import。棚卸しは取消を持たない
        ReferenceType::CsvImport | ReferenceType::Stocktake => None,
    }
}

/// 有効な記録か（無ければ `NotFound`、取消済みなら `ValidationFailed`）
fn ensure_active(
    tx: &rusqlite::Transaction<'_>,
    table: &'static str,
    label: &str,
    record_id: i64,
) -> Result<(), BizError> {
    match get_record_cancel(tx, table, record_id) {
        Ok((None, _)) => Ok(()),
        Ok((Some(canceled_at), _)) => Err(BizError::ValidationFailed(format!(
            "この記録は {canceled_at} に取り消し済みです"
        ))),
        Err(DbError::NotFound) => Err(BizError::NotFound(format!(
            "{label}が見つかりません: {record_id}"
        ))),
        Err(e) => Err(BizError::DatabaseError(e)),
    }
}

/// 入庫・返品交換・手動販売・廃棄・振替の記録を取り消す（全て 1 つの TX）
///
/// 31-biz-inventory-service.md §12.6c
pub fn cancel_inventory_record(
    conn: &mut DbConnection,
    record_type: ReferenceType,
    record_id: i64,
    reason: &str,
) -> Result<RecordCancelResult, BizError> {
    // 1. 種類と理由（TX を開く前。DB の CHECK の trim は半角空白だけを除くので、全角空白・tab はここで止める）
    let Some((table, label)) = cancelable(record_type) else {
        return Err(BizError::ValidationFailed(
            "この種類の記録は取り消せません".to_string(),
        ));
    };
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(BizError::ValidationFailed(
            "取り消す理由を入れてください".to_string(),
        ));
    }

    // 2. header（無い記録・取消済みを拒む）。以後の失敗は TX ごと戻る（tx を commit せずに落とす）
    let tx = conn.transaction().map_err(DbError::from)?;
    ensure_active(&tx, table, label, record_id)?;

    // 3. 手動販売: 結んだ売上の行を無効にする。数が明細の数と違えば拒む（migration の前に保存した記録）
    let voided_sale_count = if record_type == ReferenceType::ManualSale {
        let voided = sales_repo::void_sale_records_by_manual_sale(&tx, record_id)?;
        if voided != manual_sale_repo::count_manual_sale_items(&tx, record_id)? {
            return Err(BizError::ValidationFailed(
                "この手動販売は売上の記録と結ばれていないため取り消せません（アプリの更新の前に保存した記録です）"
                    .to_string(),
            ));
        }
        voided
    } else {
        0
    };

    // 4. header に取消の日時（created_at と同じ形）と理由を書く
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    if mark_record_canceled(&tx, table, record_id, &now, reason)? != 1 {
        // 同じ TX の中なので通常は届かない。届いたら手順 2 と同じ文で拒む
        ensure_active(&tx, table, label, record_id)?;
        return Err(BizError::DatabaseError(DbError::QueryFailed(
            "取消の列を書けませんでした".to_string(),
        )));
    }

    // 5. movement を無効にし（Z004 の取消と同じ関数）、その商品を重複なく商品コードの順に求め直す。
    //    求め直しの上限の error は TX 全体を戻す
    let voided_movements =
        sales_repo::void_movements_by_reference(&tx, record_type.as_str(), record_id)?;
    let codes: BTreeSet<&str> = voided_movements
        .iter()
        .map(|movement| movement.product_code.as_str())
        .collect();
    // 6. 求め直した在庫が負の商品を知らせる（止めない。INV-3）
    let mut stock_warnings = Vec::new();
    for code in codes {
        let stock_after = recompute_stock(&tx, code)?.stock;
        if stock_after < 0 {
            let product = product_repo::find_by_product_code(&tx, code)?
                .ok_or_else(|| BizError::NotFound(format!("商品が見つかりません: {code}")))?;
            stock_warnings.push(negative_stock_warning_message(
                code,
                &StockChangeOutcome {
                    stock_after,
                    negative_stock_warning: true,
                    stock_unit: product.product.stock_unit,
                },
            ));
        }
    }

    // 7. 操作ログ（理由は header が持つので写さない）
    let detail = serde_json::json!({
        "record_type": record_type.as_str(),
        "record_id": record_id,
        "voided_movement_count": voided_movements.len(),
        "voided_sale_count": voided_sale_count,
    });
    system_repo::insert_operation_log(
        &tx,
        &system_repo::NewOperationLog {
            operation_type: "inventory_record_cancel".to_string(),
            summary: format!("{label}を取り消しました: ID {record_id}"),
            detail_json: Some(detail.to_string()),
        },
    )?;

    // 8. COMMIT
    tx.commit().map_err(DbError::from)?;
    Ok(RecordCancelResult { stock_warnings })
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::biz::integrity_service::run_integrity_check;
    use crate::biz::inventory_service::disposal::DisposalItemInput;
    use crate::biz::inventory_service::manual_sale::ManualSaleItemInput;
    use crate::biz::inventory_service::receiving::ReceivingItemInput;
    use crate::biz::inventory_service::returns::ReturnItemInput;
    use crate::biz::inventory_service::transfer::TransferItemInput;
    use crate::biz::inventory_service::{
        create_disposal, create_manual_sale, create_receiving, create_return, create_transfer,
        get_disposal_record, get_manual_sale_record, get_receiving_record, get_return_record,
        get_transfer_record, list_disposals, list_inventory_records, list_movements,
        list_receivings, list_returns, DisposalCreateRequest, ManualSaleCreateRequest,
        ReceivingCreateRequest, ReturnCreateRequest, TransferCreateRequest,
    };
    use crate::biz::product_service::get_stock_detail;
    use crate::biz::sales_service::{get_daily_sales, get_monthly_sales, SalesMode};
    use crate::biz::stock_ledger::settlement_support::settlement;
    use crate::biz::stock_ledger::{derive_stock, DerivedStock, RecheckReason};
    use crate::biz::stocktake_service::{
        complete_stocktake, get_stocktake_record, save_stock_count, start_stocktake,
        CompleteStocktakeRequest, SaveStockCountRequest,
    };
    use crate::constants::MAX_STOCK_QUANTITY;
    use crate::db::disposal_repo::{DisposalType, InventoryRecordQuery};
    use crate::db::inventory_common::ListQuery;
    use crate::db::inventory_repo::MovementQuery;
    use crate::db::manual_sale_repo::ManualSaleReason;
    use crate::db::return_repo::{ReturnDirection, ReturnExchangeType};
    use crate::db::test_support::dump;
    use crate::db::transfer_repo::TransferDirection;
    use rusqlite::types::Value;

    const D: &str = "2026-11-18";
    const D_NEXT: &str = "2026-11-19";
    const REASON: &str = "打ち間違い";
    const ALL_TYPES: [ReferenceType; 5] = [
        ReferenceType::ReceivingRecord,
        ReferenceType::ReturnRecord,
        ReferenceType::ManualSale,
        ReferenceType::DisposalRecord,
        ReferenceType::TransferRecord,
    ];

    // --- 保存の helper（合成の値。key は test の中で一意にする） ---

    fn receive(conn: &mut DbConnection, key: &str, code: &str, quantity: i64, date: &str) -> i64 {
        create_receiving(
            conn,
            ReceivingCreateRequest {
                idempotency_key: key.to_string(),
                supplier_id: None,
                receiving_date: date.to_string(),
                note: None,
                items: vec![ReceivingItemInput {
                    product_code: code.to_string(),
                    quantity,
                    cost_price_centi: 300,
                }],
            },
        )
        .unwrap()
        .record_id
    }

    fn dispose(conn: &mut DbConnection, key: &str, code: &str, quantity: i64, date: &str) -> i64 {
        create_disposal(
            conn,
            DisposalCreateRequest {
                idempotency_key: key.to_string(),
                disposal_date: date.to_string(),
                items: vec![DisposalItemInput {
                    product_code: code.to_string(),
                    disposal_type: DisposalType::Disposal,
                    quantity,
                    cost_price_centi: 300,
                    reason: "汚れ".to_string(),
                }],
            },
        )
        .unwrap()
        .record_id
    }

    fn sell(conn: &mut DbConnection, key: &str, code: &str, quantity: i64, amount: i64) -> i64 {
        create_manual_sale(
            conn,
            ManualSaleCreateRequest {
                idempotency_key: key.to_string(),
                sale_date: D.to_string(),
                reason: ManualSaleReason::PluUnregistered,
                note: None,
                items: vec![ManualSaleItemInput {
                    product_code: code.to_string(),
                    quantity,
                    amount,
                }],
                confirmation_token: None,
            },
        )
        .unwrap()
        .sale_id
        .unwrap()
    }

    fn give_back(
        conn: &mut DbConnection,
        key: &str,
        register_processed: bool,
        items: &[(&str, ReturnDirection, i64)],
    ) -> i64 {
        let exchange = items
            .iter()
            .any(|(_, direction, _)| *direction == ReturnDirection::Out);
        create_return(
            conn,
            ReturnCreateRequest {
                idempotency_key: key.to_string(),
                return_type: if exchange {
                    ReturnExchangeType::Exchange
                } else {
                    ReturnExchangeType::Return
                },
                return_date: D.to_string(),
                register_processed,
                receipt_image_path: None,
                note: None,
                items: items
                    .iter()
                    .map(|(code, direction, quantity)| ReturnItemInput {
                        product_code: code.to_string(),
                        direction: *direction,
                        quantity: *quantity,
                    })
                    .collect(),
            },
        )
        .unwrap()
        .record_id
    }

    fn transfer(
        conn: &mut DbConnection,
        key: &str,
        items: &[(&str, TransferDirection, i64)],
    ) -> i64 {
        create_transfer(
            conn,
            TransferCreateRequest {
                idempotency_key: key.to_string(),
                transfer_date: D.to_string(),
                items: items
                    .iter()
                    .map(|(code, direction, quantity)| TransferItemInput {
                        product_code: code.to_string(),
                        direction: *direction,
                        quantity: *quantity,
                    })
                    .collect(),
            },
        )
        .unwrap()
        .record_id
    }

    fn count(conn: &mut DbConnection, code: &str, value: i64, date: &str, fresh: bool) {
        count_item(conn, code, value, date, fresh, None);
    }

    fn count_item(
        conn: &mut DbConnection,
        code: &str,
        value: i64,
        date: &str,
        fresh: bool,
        item: Option<i64>,
    ) {
        save_stock_count(
            conn,
            &SaveStockCountRequest {
                product_code: code.to_string(),
                count_value: value,
                fresh,
                business_date: date.to_string(),
                stocktake_item_id: item,
            },
            date,
        )
        .unwrap();
    }

    fn cancel(conn: &mut DbConnection, record_type: ReferenceType, id: i64) -> RecordCancelResult {
        cancel_inventory_record(conn, record_type, id, REASON).unwrap()
    }

    // --- 読取りの helper ---

    fn rows(conn: &DbConnection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    /// その記録の movement の `is_voided`（id の順）
    fn voided(conn: &DbConnection, record_type: ReferenceType, id: i64) -> Vec<bool> {
        conn.prepare(
            "SELECT is_voided FROM inventory_movements
             WHERE reference_type = ?1 AND reference_id = ?2 ORDER BY id",
        )
        .unwrap()
        .query_map(rusqlite::params![record_type.as_str(), id], |row| {
            row.get(0)
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }

    fn cancel_columns(
        conn: &DbConnection,
        table: &'static str,
        id: i64,
    ) -> (Option<String>, Option<String>) {
        get_record_cancel(conn, table, id).unwrap()
    }

    /// 取消の操作ログの detail（新しい順の先頭）
    fn last_cancel_log(conn: &DbConnection) -> serde_json::Value {
        let detail: String = conn
            .query_row(
                "SELECT detail_json FROM operation_logs
                 WHERE operation_type = 'inventory_record_cancel' ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        serde_json::from_str(&detail).unwrap()
    }

    /// 写し（在庫・要再確認・範囲）
    fn copy(conn: &DbConnection, code: &str) -> (i64, bool, Option<i64>, Option<i64>) {
        conn.query_row(
            "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
             FROM products WHERE product_code = ?1",
            [code],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap()
    }

    /// 取消が書く表の全部の行（拒否の後に何も変わっていないことの比較に使う）
    fn snapshot(conn: &DbConnection) -> Vec<Vec<Vec<Value>>> {
        [
            "receiving_records",
            "return_records",
            "manual_sales",
            "disposal_records",
            "transfer_records",
            "sale_records",
            "inventory_movements",
            "products",
            "operation_logs",
        ]
        .iter()
        .map(|table| dump(conn, &format!("SELECT * FROM {table} ORDER BY rowid")))
        .collect()
    }

    fn assert_rejected(
        conn: &mut DbConnection,
        record_type: ReferenceType,
        id: i64,
        reason: &str,
        message: &str,
    ) {
        let before = snapshot(conn);
        let result = cancel_inventory_record(conn, record_type, id, reason);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref m)) if m == message),
            "{message}: {result:?}"
        );
        assert_eq!(snapshot(conn), before, "{message}");
    }

    #[test]
    fn test_cancel_receiving_voids_own_movements_and_recomputes_req208() {
        // REQ-208 / 31 §12.6c / TRACE-D4・D6 / §65.7.2・§65.8.2 / D-163（Test Plan T1）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "CR-001", 10);
        let other = receive(&mut conn, "cr-other", "CR-001", 2, D);
        let before_save = stock_of(&conn, "CR-001");
        let target = receive(&mut conn, "cr-target", "CR-001", 3, D);
        assert_eq!(stock_of(&conn, "CR-001"), 15);
        let tables = [
            "receiving_records",
            "receiving_items",
            "inventory_movements",
        ];
        let row_counts = tables.map(|table| rows(&conn, table));

        // 理由は前後の空白（全角を含む）を除いて残す
        let result = cancel_inventory_record(
            &mut conn,
            ReferenceType::ReceivingRecord,
            target,
            "　打ち間違い \t",
        )
        .unwrap();

        assert!(result.stock_warnings.is_empty());
        // 在庫は「その記録を保存する前」の値。別の入庫は残る
        assert_eq!(stock_of(&conn, "CR-001"), before_save);
        assert_eq!(
            voided(&conn, ReferenceType::ReceivingRecord, target),
            [true]
        );
        assert_eq!(
            voided(&conn, ReferenceType::ReceivingRecord, other),
            [false]
        );
        // 物理削除しない
        assert_eq!(tables.map(|table| rows(&conn, table)), row_counts);
        let (canceled_at, cancel_reason) = cancel_columns(&conn, "receiving_records", target);
        assert_eq!(
            canceled_at.map(|at| at.len()),
            Some("2026-11-21T10:00:00".len())
        );
        assert_eq!(cancel_reason.as_deref(), Some(REASON));
        assert_eq!(
            cancel_columns(&conn, "receiving_records", other),
            (None, None)
        );
        assert_eq!(
            last_cancel_log(&conn),
            serde_json::json!({
                "record_type": "receiving_record",
                "record_id": target,
                "voided_movement_count": 1,
                "voided_sale_count": 0,
            })
        );
        // 商品別の在庫変動の履歴に、取り消した記録の行は出ない（別の入庫は出る）
        let history = list_movements(
            &conn,
            &MovementQuery {
                product_code: "CR-001".to_string(),
                date_from: None,
                date_to: None,
                movement_type: None,
                page: 1,
                per_page: 50,
            },
        )
        .unwrap();
        let receiving_ids: Vec<Option<i64>> = history
            .items
            .iter()
            .filter_map(|item| item.movement.as_ref())
            .filter(|movement| movement.reference_type == Some(ReferenceType::ReceivingRecord))
            .map(|movement| movement.reference_id)
            .collect();
        assert_eq!(receiving_ids, [Some(other)]);
    }

    #[test]
    fn test_cancel_manual_sale_voids_linked_sale_records_req208() {
        // REQ-208 / 31 §12.5・§12.6c / §65.7.2 / POS §13 / 56・57（Test Plan T2）
        // 同じ日・同じ品・同じ額の手動販売 2 件（合成の 500 円ずつ）から片方だけを外す
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "CM-001", 10);
        let target = sell(&mut conn, "cm-target", "CM-001", 1, 500);
        let other = sell(&mut conn, "cm-other", "CM-001", 1, 500);
        let links = |conn: &DbConnection| {
            dump(
                conn,
                "SELECT manual_sale_id, is_voided FROM sale_records ORDER BY id",
            )
        };
        // 保存で元の手動販売を指す
        assert_eq!(
            links(&conn),
            vec![
                vec![Value::Integer(target), Value::Integer(0)],
                vec![Value::Integer(other), Value::Integer(0)],
            ]
        );
        let month_amount = |conn: &DbConnection| -> i64 {
            get_monthly_sales(conn, &D[..7], SalesMode::ByProduct)
                .unwrap()
                .items
                .iter()
                .map(|item| item.amount)
                .sum()
        };
        assert_eq!(get_daily_sales(&conn, D).unwrap().grand_total.amount, 1000);
        assert_eq!(month_amount(&conn), 1000);

        cancel(&mut conn, ReferenceType::ManualSale, target);

        assert_eq!(get_daily_sales(&conn, D).unwrap().grand_total.amount, 500);
        assert_eq!(month_amount(&conn), 500);
        assert_eq!(
            links(&conn),
            vec![
                vec![Value::Integer(target), Value::Integer(1)],
                vec![Value::Integer(other), Value::Integer(0)],
            ]
        );
        assert_eq!(stock_of(&conn, "CM-001"), 9);
        assert_eq!(voided(&conn, ReferenceType::ManualSale, target), [true]);
        assert_eq!(voided(&conn, ReferenceType::ManualSale, other), [false]);
        assert_eq!(last_cancel_log(&conn)["voided_sale_count"], 1);
    }

    #[test]
    fn test_cancel_return_and_disposal_void_only_saved_movements_req208() {
        // REQ-208 / 31 §12.6c の手順 5 / §65.7.2 / D-148 / 36（Test Plan T3）
        let (_dir, mut conn) = setup_test_db();
        for code in ["CT-LINK", "CT-IN", "CT-OUT", "CT-DSP"] {
            create_test_product(&conn, code, 10);
        }
        // レジで読んで戻した在庫連動の商品は、この保存で在庫を動かさない（movement が無い）
        conn.execute(
            "UPDATE products SET jan_code = '4901234567887' WHERE product_code = 'CT-LINK'",
            [],
        )
        .unwrap();
        let processed = give_back(
            &mut conn,
            "ct-processed",
            true,
            &[("CT-LINK", ReturnDirection::In, 1)],
        );
        assert!(voided(&conn, ReferenceType::ReturnRecord, processed).is_empty());
        cancel(&mut conn, ReferenceType::ReturnRecord, processed);
        assert_eq!(stock_of(&conn, "CT-LINK"), 10);
        assert!(cancel_columns(&conn, "return_records", processed)
            .0
            .is_some());
        assert_eq!(last_cancel_log(&conn)["voided_movement_count"], 0);

        // 交換は戻りと渡しの両方を戻す
        let exchange = give_back(
            &mut conn,
            "ct-exchange",
            false,
            &[
                ("CT-IN", ReturnDirection::In, 1),
                ("CT-OUT", ReturnDirection::Out, 2),
            ],
        );
        assert_eq!(
            (stock_of(&conn, "CT-IN"), stock_of(&conn, "CT-OUT")),
            (11, 8)
        );
        cancel(&mut conn, ReferenceType::ReturnRecord, exchange);
        assert_eq!(
            (stock_of(&conn, "CT-IN"), stock_of(&conn, "CT-OUT")),
            (10, 10)
        );
        assert_eq!(
            voided(&conn, ReferenceType::ReturnRecord, exchange),
            [true, true]
        );

        let disposal = dispose(&mut conn, "ct-disposal", "CT-DSP", 4, D);
        assert_eq!(stock_of(&conn, "CT-DSP"), 6);
        cancel(&mut conn, ReferenceType::DisposalRecord, disposal);
        assert_eq!(stock_of(&conn, "CT-DSP"), 10);

        // 取消の後も、記録から求め直した在庫と写しが合う
        assert_eq!(run_integrity_check(&conn).unwrap().mismatch_count, 0);
    }

    /// 同じ手順を、対象の記録を保存する DB と保存しない DB に流し、取消の後の求め直し・写しが
    /// 保存しなかった DB と同じことを確かめて、取消の前後の求め直しを返す
    fn cancel_and_compare_with_record_absent(
        scenario: impl Fn(&mut DbConnection, bool) -> Option<(ReferenceType, i64)>,
    ) -> (DerivedStock, DerivedStock) {
        const CODE: &str = "CH-001";
        let (_dir_with, mut with) = setup_test_db();
        let (_dir_without, mut without) = setup_test_db();
        for conn in [&with, &without] {
            create_test_product(conn, CODE, 10);
        }
        let (record_type, id) = scenario(&mut with, true).unwrap();
        assert!(scenario(&mut without, false).is_none());
        let before = derive_stock(&with, CODE).unwrap();
        cancel(&mut with, record_type, id);
        let after = derive_stock(&with, CODE).unwrap();
        assert_eq!(after, derive_stock(&without, CODE).unwrap());
        assert_eq!(copy(&with, CODE), copy(&without, CODE));
        assert_eq!(
            copy(&with, CODE),
            (
                after.stock,
                !after.recheck_reasons.is_empty(),
                after.range.map(|(low, _)| low),
                after.range.map(|(_, high)| high),
            )
        );
        (before, after)
    }

    fn settled(stock: i64) -> DerivedStock {
        DerivedStock {
            stock,
            range: None,
            recheck_reasons: Vec::new(),
        }
    }

    #[test]
    fn test_cancel_never_adds_recheck_and_matches_record_absent_req208() {
        // REQ-208 / REQ-303 / TRACE-D6 / ADR SPEC-STK-CNT-D2・D3 / D-163（Test Plan T4。H5 の回帰）
        // (a) 入庫の後に同じ日の fresh な計数: 取消で要再確認にならず、在庫は計数の値
        let (before, after) = cancel_and_compare_with_record_absent(|conn, with_record| {
            let id = with_record.then(|| receive(conn, "a", "CH-001", 3, D));
            count(conn, "CH-001", 12, D, true);
            id.map(|id| (ReferenceType::ReceivingRecord, id))
        });
        assert_eq!((before, after), (settled(12), settled(12)));

        // (b) 既定の計数と同じ日付の入庫（要再確認）: 取消で要再確認が解ける
        let (before, after) = cancel_and_compare_with_record_absent(|conn, with_record| {
            count(conn, "CH-001", 10, D, false);
            with_record.then(|| {
                (
                    ReferenceType::ReceivingRecord,
                    receive(conn, "b", "CH-001", 3, D),
                )
            })
        });
        assert_eq!(before.recheck_reasons, [RecheckReason::SameDateMovement]);
        assert_eq!(before.range, Some((10, 13)));
        assert_eq!(after, settled(10));

        // (c) 計数の後の日付の入庫: 取消で在庫がその数だけ戻る
        let (before, after) = cancel_and_compare_with_record_absent(|conn, with_record| {
            count(conn, "CH-001", 10, D, false);
            with_record.then(|| {
                (
                    ReferenceType::ReceivingRecord,
                    receive(conn, "c", "CH-001", 3, D_NEXT),
                )
            })
        });
        assert_eq!((before, after), (settled(13), settled(10)));

        // (d) 翌日付の出庫 -5 → 同じ日に fresh な計数 10 → 既定の計数 20 → その出庫を取り消す。
        // 計数の衝突の範囲は、除いた movement の分だけ広がる（狭まる向きを期待しない）。理由は前後とも衝突だけ
        let (before, after) = cancel_and_compare_with_record_absent(|conn, with_record| {
            let id = with_record.then(|| dispose(conn, "d", "CH-001", 5, D_NEXT));
            count(conn, "CH-001", 10, D, true);
            count(conn, "CH-001", 20, D, false);
            id.map(|id| (ReferenceType::DisposalRecord, id))
        });
        assert_eq!(before.range, Some((10, 15)));
        assert_eq!(after.range, Some((10, 20)));
        assert_eq!(before.recheck_reasons, [RecheckReason::ConflictingCounts]);
        assert_eq!(after.recheck_reasons, [RecheckReason::ConflictingCounts]);

        // (e) 入れ直しの日付（65 §65.5b の dialog が日付を名指す理由）:
        // 11/18 の入庫 +3 → 11/19 の既定の計数 10 → その入庫を取り消して入れ直す
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "CH-002", 0);
        let first = receive(&mut conn, "e-first", "CH-002", 3, D);
        count(&mut conn, "CH-002", 10, D_NEXT, false);
        cancel(&mut conn, ReferenceType::ReceivingRecord, first);
        assert_eq!(stock_of(&conn, "CH-002"), 10);
        // 元の日付で入れ直すと計数に入り、在庫は計数の値のまま
        let same_date = receive(&mut conn, "e-same-date", "CH-002", 3, D);
        assert_eq!(stock_of(&conn, "CH-002"), 10);
        // 作成の画面の既定の今日（11/21）で入れ直すと、計数の後の入庫になり 3 多い
        cancel(&mut conn, ReferenceType::ReceivingRecord, same_date);
        receive(&mut conn, "e-today", "CH-002", 3, "2026-11-21");
        assert_eq!(stock_of(&conn, "CH-002"), 13);
    }

    #[test]
    fn test_cancel_inventory_record_rejects_and_leaves_db_unchanged_req208() {
        // REQ-208 / 31 §12.6c の手順 1〜5 / TRACE-D4（Test Plan T5）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "CN-001", 10);
        create_test_product(&conn, "CN-002", 10);
        let receiving = receive(&mut conn, "cn-receiving", "CN-001", 3, D);

        // 空・半角空白・全角空白（U+3000）・tab だけの理由
        for reason in ["", "  ", "\u{3000}", "\t", " \u{3000}\t"] {
            assert_rejected(
                &mut conn,
                ReferenceType::ReceivingRecord,
                receiving,
                reason,
                "取り消す理由を入れてください",
            );
        }
        // 取り消せない種類
        for record_type in [ReferenceType::CsvImport, ReferenceType::Stocktake] {
            assert_rejected(
                &mut conn,
                record_type,
                1,
                REASON,
                "この種類の記録は取り消せません",
            );
        }
        // 無い ID
        for record_type in ALL_TYPES {
            let before = snapshot(&conn);
            let result = cancel_inventory_record(&mut conn, record_type, 9_999, REASON);
            assert!(matches!(result, Err(BizError::NotFound(_))), "{result:?}");
            assert_eq!(snapshot(&conn), before);
        }

        // 売上の行と結ばれていない手動販売（migration の前に保存した記録）
        let unlinked = sell(&mut conn, "cn-unlinked", "CN-001", 1, 500);
        conn.execute("UPDATE sale_records SET manual_sale_id = NULL", [])
            .unwrap();
        assert_rejected(
            &mut conn,
            ReferenceType::ManualSale,
            unlinked,
            REASON,
            "この手動販売は売上の記録と結ばれていないため取り消せません（アプリの更新の前に保存した記録です）",
        );
        assert_eq!(voided(&conn, ReferenceType::ManualSale, unlinked), [false]);

        // 2 明細の片方だけ結びの無い手動販売: 結んだ 1 行を無効にした後で数の違いに気づくので、
        // 拒否の前に書いた行が TX ごと戻る（上の例は結びが 1 つも無く、拒否の前に何も書かない）
        let partly_linked = create_manual_sale(
            &mut conn,
            ManualSaleCreateRequest {
                idempotency_key: "cn-partly-linked".to_string(),
                sale_date: D.to_string(),
                reason: ManualSaleReason::PluUnregistered,
                note: None,
                items: ["CN-001", "CN-002"]
                    .into_iter()
                    .map(|code| ManualSaleItemInput {
                        product_code: code.to_string(),
                        quantity: 1,
                        amount: 500,
                    })
                    .collect(),
                confirmation_token: None,
            },
        )
        .unwrap()
        .sale_id
        .unwrap();
        let unlinked_rows = conn
            .execute(
                "UPDATE sale_records SET manual_sale_id = NULL
                 WHERE manual_sale_id = ?1 AND product_code = 'CN-001'",
                [partly_linked],
            )
            .unwrap();
        assert_eq!(unlinked_rows, 1);
        assert_rejected(
            &mut conn,
            ReferenceType::ManualSale,
            partly_linked,
            REASON,
            "この手動販売は売上の記録と結ばれていないため取り消せません（アプリの更新の前に保存した記録です）",
        );
        assert_eq!(
            voided(&conn, ReferenceType::ManualSale, partly_linked),
            [false, false]
        );

        // 上限: 在庫が上限の商品を 1 売り、別の入庫で上限に戻してから、その販売を取り消す。
        // header・売上の行・movement を書いた後の求め直しで上限を 1 超え、TX 全体が戻る
        create_test_product(&conn, "CN-MAX", MAX_STOCK_QUANTITY);
        let sale = sell(&mut conn, "cn-max-sale", "CN-MAX", 1, 500);
        receive(&mut conn, "cn-max-receiving", "CN-MAX", 1, D);
        assert_eq!(stock_of(&conn, "CN-MAX"), MAX_STOCK_QUANTITY);
        assert_rejected(
            &mut conn,
            ReferenceType::ManualSale,
            sale,
            REASON,
            "在庫数が大きすぎます",
        );

        // 2 回目の取消（振替でも）: 拒み、取消の日時を変えない
        let transfer_id = transfer(
            &mut conn,
            "cn-transfer",
            &[
                ("CN-001", TransferDirection::Out, 1),
                ("CN-002", TransferDirection::In, 2),
            ],
        );
        for (record_type, table, id) in [
            (
                ReferenceType::ReceivingRecord,
                "receiving_records",
                receiving,
            ),
            (
                ReferenceType::TransferRecord,
                "transfer_records",
                transfer_id,
            ),
        ] {
            cancel(&mut conn, record_type, id);
            let canceled_at = cancel_columns(&conn, table, id).0.unwrap();
            assert_rejected(
                &mut conn,
                record_type,
                id,
                "もう一度",
                &format!("この記録は {canceled_at} に取り消し済みです"),
            );
        }
    }

    #[test]
    fn cancel_after_stocktake_completion_keeps_valuation() {
        // REQ-208 / ADR SPEC-STK-CNT-D6 / 31 §12.6c の設計判断 / 35「完了」（Test Plan T6）
        const TODAY: &str = "2026-11-19";
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "CS-A", 0);
        create_test_product(&conn, "CS-B", 0);
        // (a) 計数に含まれた入庫: 入庫 +3 → その入庫を見た fresh な計数 10
        let counted = receive(&mut conn, "cs-a", "CS-A", 3, D);
        let stocktake_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item_id = |conn: &DbConnection, code: &str| -> i64 {
            conn.query_row(
                "SELECT id FROM stocktake_items WHERE stocktake_id = ?1 AND product_code = ?2",
                rusqlite::params![stocktake_id, code],
                |row| row.get(0),
            )
            .unwrap()
        };
        let (item_a, item_b) = (item_id(&conn, "CS-A"), item_id(&conn, "CS-B"));
        count_item(&mut conn, "CS-A", 10, TODAY, true, Some(item_a));
        // (b) 計数の後で完了の日までの日付の入庫: 計数 10（前の日）→ 入庫 +3（完了の日）
        count_item(&mut conn, "CS-B", 10, D, false, Some(item_b));
        let after_count = receive(&mut conn, "cs-b", "CS-B", 3, TODAY);
        // 完了の条件（計数の日の前後に取り込み済みの精算がある。35 の表）を満たす
        settlement(&conn, 1, "2026-11-17");
        settlement(&conn, 2, TODAY);
        complete_stocktake(
            &mut conn,
            &CompleteStocktakeRequest {
                stocktake_id,
                today: TODAY.to_string(),
            },
        )
        .unwrap();

        // 凍結した評価の列（完了が書いた値）
        let valuation = |conn: &DbConnection| {
            (
                dump(
                    conn,
                    "SELECT product_code, valuation_quantity, valuation_cost_price_centi
                     FROM stocktake_items ORDER BY product_code",
                ),
                dump(conn, "SELECT total_cost FROM stocktakes"),
            )
        };
        let frozen = valuation(&conn);
        assert_eq!(frozen.0[0][1], Value::Integer(10));
        assert_eq!(frozen.0[1][1], Value::Integer(13));
        let difference = |conn: &DbConnection, code: &str| {
            get_stocktake_record(conn, stocktake_id)
                .unwrap()
                .items
                .into_iter()
                .find(|item| item.product_code == code)
                .unwrap()
                .difference
        };
        // 完了の後の差の行（確定と要再確認の両方）の (商品コード, 評価数量, 完了の時点の数)
        let after_completion = |conn: &DbConnection| -> Vec<(String, i64, i64)> {
            let diff = get_stocktake_record(conn, stocktake_id)
                .unwrap()
                .after_completion
                .unwrap();
            diff.confirmed
                .into_iter()
                .chain(diff.recheck)
                .map(|row| {
                    (
                        row.product_code,
                        row.valuation_quantity,
                        row.stock_at_completion,
                    )
                })
                .collect()
        };
        assert_eq!(difference(&conn, "CS-A"), Some(7));
        assert!(after_completion(&conn).is_empty());

        // (a) を取り消す: 評価は変わらず、求め直した数は 10 のまま、参考の差の行は出ない。差異は 7 → 10 に求め直る
        cancel(&mut conn, ReferenceType::ReceivingRecord, counted);
        assert_eq!(valuation(&conn), frozen);
        assert_eq!(stock_of(&conn, "CS-A"), 10);
        assert!(after_completion(&conn).is_empty());
        assert_eq!(difference(&conn, "CS-A"), Some(10));

        // (b) を取り消す: 評価は変わらず、参考の差の行が出て、完了の時点の数は 10
        cancel(&mut conn, ReferenceType::ReceivingRecord, after_count);
        assert_eq!(valuation(&conn), frozen);
        assert_eq!(after_completion(&conn), [("CS-B".to_string(), 13, 10)]);
    }

    #[test]
    fn inventory_records_status_canceled_for_five_record_types() {
        // REQ-208 / §65.4.1 / 31 §12.6a / 21 §10.5a・§10.5b / 44 §23.7b（Test Plan T7）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "CL-001", 10);
        create_test_product(&conn, "CL-002", 10);
        let ids = [
            receive(&mut conn, "cl-receiving", "CL-001", 3, "2026-11-20"),
            give_back(
                &mut conn,
                "cl-return",
                false,
                &[("CL-001", ReturnDirection::In, 1)],
            ),
            sell(&mut conn, "cl-sale", "CL-001", 1, 500),
            dispose(&mut conn, "cl-disposal", "CL-001", 1, D),
            transfer(
                &mut conn,
                "cl-transfer",
                &[
                    ("CL-001", TransferDirection::Out, 1),
                    ("CL-002", TransferDirection::In, 2),
                ],
            ),
        ];
        // 有効のまま残す記録（前の日の入庫）
        let active = receive(&mut conn, "cl-active", "CL-001", 2, D);
        for (record_type, id) in ALL_TYPES.into_iter().zip(ids) {
            cancel(&mut conn, record_type, id);
        }

        let list = |conn: &DbConnection, status: Option<&str>| -> Vec<(String, i64, String)> {
            list_inventory_records(
                conn,
                &InventoryRecordQuery {
                    record_type: None,
                    date_from: None,
                    date_to: None,
                    record_id: None,
                    product_keyword: None,
                    department_id: None,
                    status: status.map(str::to_string),
                    page: 1,
                    per_page: 50,
                },
            )
            .unwrap()
            .items
            .into_iter()
            .map(|row| (row.record_type, row.record_id, row.status))
            .collect()
        };
        let mut canceled: Vec<(String, i64, String)> = ALL_TYPES
            .into_iter()
            .zip(ids)
            .map(|(record_type, id)| (record_type.as_str().to_string(), id, "canceled".to_string()))
            .collect();
        let active_row = ("receiving_record".to_string(), active, "active".to_string());
        let sorted = |mut rows: Vec<(String, i64, String)>| {
            rows.sort();
            rows
        };
        canceled.sort();
        assert_eq!(sorted(list(&conn, Some("canceled"))), canceled);
        assert_eq!(list(&conn, Some("active")), vec![active_row.clone()]);
        let mut all = canceled.clone();
        all.push(active_row);
        assert_eq!(sorted(list(&conn, None)), sorted(all));

        // 詳細: 状態・取消の日時・理由
        let details = [
            {
                let d = get_receiving_record(&conn, ids[0]).unwrap();
                (d.status, d.canceled_at, d.cancel_reason, d.movements.len())
            },
            {
                let d = get_return_record(&conn, ids[1]).unwrap();
                (d.status, d.canceled_at, d.cancel_reason, d.movements.len())
            },
            {
                let d = get_manual_sale_record(&conn, ids[2]).unwrap();
                (d.status, d.canceled_at, d.cancel_reason, d.movements.len())
            },
            {
                let d = get_disposal_record(&conn, ids[3]).unwrap();
                (d.status, d.canceled_at, d.cancel_reason, d.movements.len())
            },
            {
                let d = get_transfer_record(&conn, ids[4]).unwrap();
                (d.status, d.canceled_at, d.cancel_reason, d.movements.len())
            },
        ];
        for (status, canceled_at, cancel_reason, movement_count) in details {
            assert_eq!(status, "canceled");
            assert!(canceled_at.is_some());
            assert_eq!(cancel_reason.as_deref(), Some(REASON));
            // 無効にした movement は関連 movement に出ない
            assert_eq!(movement_count, 0);
        }
        let kept = get_receiving_record(&conn, active).unwrap();
        assert_eq!(
            (kept.status.as_str(), kept.canceled_at, kept.cancel_reason),
            ("active", None, None)
        );

        // 作業画面の直近の一覧: 取消済みの行だけ canceled_at を持つ
        let query = ListQuery {
            page: 1,
            per_page: 10,
            date_from: None,
            date_to: None,
        };
        let receivings: Vec<(i64, bool)> = list_receivings(&conn, &query)
            .unwrap()
            .items
            .into_iter()
            .map(|row| (row.id, row.canceled_at.is_some()))
            .collect();
        assert_eq!(receivings, [(ids[0], true), (active, false)]);
        assert!(list_returns(&conn, &query).unwrap().items[0]
            .canceled_at
            .is_some());
        assert!(list_disposals(&conn, &query).unwrap().items[0]
            .canceled_at
            .is_some());

        // 商品の最終入庫日は取り消した入庫（11/20）を数えない
        assert_eq!(
            get_stock_detail(&conn, "CL-001")
                .unwrap()
                .last_receiving_date
                .as_deref(),
            Some(D)
        );
    }

    #[test]
    fn test_cancel_transfer_voids_every_line_and_recomputes_all_products_req208() {
        // REQ-208 / REQ-210 / 31 §12.6c の手順 5・6 / TRACE-D14 / §65.7.2 / D-163（Test Plan T13）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "CX-OUT", 10);
        create_test_product(&conn, "CX-IN1", 0);
        create_test_product(&conn, "CX-IN2", 5);
        // 同じ商品の別の振替と入庫（残る側）
        let other_transfer = transfer(
            &mut conn,
            "cx-other",
            &[
                ("CX-OUT", TransferDirection::Out, 1),
                ("CX-IN1", TransferDirection::In, 1),
            ],
        );
        let other_receiving = receive(&mut conn, "cx-receiving", "CX-IN2", 1, D);
        let codes = ["CX-OUT", "CX-IN1", "CX-IN2"];
        let before_save = codes.map(|code| stock_of(&conn, code));
        assert_eq!(before_save, [9, 1, 6]);

        // 減らす 1・増やす 2（明細 3、movement 3）
        let target = transfer(
            &mut conn,
            "cx-target",
            &[
                ("CX-OUT", TransferDirection::Out, 4),
                ("CX-IN1", TransferDirection::In, 2),
                ("CX-IN2", TransferDirection::In, 3),
            ],
        );
        assert_eq!(codes.map(|code| stock_of(&conn, code)), [5, 3, 9]);
        let tables = ["transfer_records", "transfer_items", "inventory_movements"];
        let row_counts = tables.map(|table| rows(&conn, table));

        let result = cancel(&mut conn, ReferenceType::TransferRecord, target);

        assert!(result.stock_warnings.is_empty());
        assert_eq!(codes.map(|code| stock_of(&conn, code)), before_save);
        assert_eq!(
            voided(&conn, ReferenceType::TransferRecord, target),
            [true; 3]
        );
        assert_eq!(
            voided(&conn, ReferenceType::TransferRecord, other_transfer),
            [false; 2]
        );
        assert_eq!(
            voided(&conn, ReferenceType::ReceivingRecord, other_receiving),
            [false]
        );
        assert_eq!(tables.map(|table| rows(&conn, table)), row_counts);
        assert_eq!(last_cancel_log(&conn)["voided_movement_count"], 3);
        assert_eq!(last_cancel_log(&conn)["record_type"], "transfer_record");

        // 増やした商品を売った後の取消: 止めずに、負になった商品を知らせる（INV-3）
        create_test_product(&conn, "CX-SOLD", 0);
        let sold_out = transfer(
            &mut conn,
            "cx-sold",
            &[
                ("CX-OUT", TransferDirection::Out, 1),
                ("CX-SOLD", TransferDirection::In, 2),
            ],
        );
        sell(&mut conn, "cx-sale", "CX-SOLD", 2, 1000);
        let result = cancel(&mut conn, ReferenceType::TransferRecord, sold_out);
        assert_eq!(stock_of(&conn, "CX-SOLD"), -2);
        assert_eq!(
            result.stock_warnings,
            ["CX-SOLD: 在庫がマイナスになりました（-2）"]
        );
    }
}
