//! 不変条件テスト + pub use リーク検出

#[cfg(test)]
mod tests {
    use super::super::disposal::{create_disposal, DisposalCreateRequest, DisposalItemInput};
    use super::super::manual_sale::{
        create_manual_sale, ManualSaleCreateRequest, ManualSaleItemInput,
    };
    use super::super::receiving::{create_receiving, ReceivingCreateRequest, ReceivingItemInput};
    use super::super::returns::{create_return, ReturnCreateRequest, ReturnItemInput};
    use super::super::test_support::*;
    use crate::db::disposal_repo::DisposalType;
    use crate::db::manual_sale_repo::ManualSaleReason;
    use crate::db::return_repo::{ReturnDirection, ReturnExchangeType};
    fn make_receiving_req(key: &str, items: Vec<ReceivingItemInput>) -> ReceivingCreateRequest {
        ReceivingCreateRequest {
            idempotency_key: key.to_string(),
            supplier_id: None,
            receiving_date: "2026-04-07".to_string(),
            note: None,
            items,
        }
    }

    fn receiving_item(code: &str, qty: i64, cost: i64) -> ReceivingItemInput {
        ReceivingItemInput {
            product_code: code.to_string(),
            quantity: qty,
            cost_price_centi: cost,
        }
    }

    fn make_return_req(key: &str, items: Vec<ReturnItemInput>) -> ReturnCreateRequest {
        ReturnCreateRequest {
            idempotency_key: key.to_string(),
            return_type: ReturnExchangeType::Return,
            return_date: "2026-04-07".to_string(),
            register_processed: false,
            receipt_image_path: None,
            note: None,
            items,
        }
    }

    fn return_item(code: &str, direction: &str, qty: i64) -> ReturnItemInput {
        ReturnItemInput {
            product_code: code.to_string(),
            direction: match direction {
                "in" => ReturnDirection::In,
                "out" => ReturnDirection::Out,
                _ => panic!("test helper received invalid direction"),
            },
            quantity: qty,
        }
    }

    fn make_manual_sale_req(key: &str, items: Vec<ManualSaleItemInput>) -> ManualSaleCreateRequest {
        ManualSaleCreateRequest {
            idempotency_key: key.to_string(),
            sale_date: "2026-04-07".to_string(),
            reason: ManualSaleReason::PluUnregistered,
            note: None,
            items,
            confirmation_token: None,
        }
    }

    fn manual_sale_item(code: &str, qty: i64, amount: i64) -> ManualSaleItemInput {
        ManualSaleItemInput {
            product_code: code.to_string(),
            quantity: qty,
            amount,
        }
    }

    fn make_disposal_req(key: &str, items: Vec<DisposalItemInput>) -> DisposalCreateRequest {
        DisposalCreateRequest {
            idempotency_key: key.to_string(),
            disposal_date: "2026-04-07".to_string(),
            items,
        }
    }

    fn disposal_item(code: &str, qty: i64, cost: i64, reason: &str) -> DisposalItemInput {
        DisposalItemInput {
            product_code: code.to_string(),
            disposal_type: DisposalType::Disposal,
            quantity: qty,
            cost_price_centi: cost,
            reason: reason.to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // 結果フラグ不変条件テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_result_flags_req201_receiving() {
        // REQ-201: 入庫記録 — INV-5: 冪等性 — 結果フラグ不変条件（入庫）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "FLAG-R1", 10);

        let req = make_receiving_req("flag-r1", vec![receiving_item("FLAG-R1", 1, 100)]);
        let r1 = create_receiving(&mut conn, req.clone()).unwrap();
        assert!(r1.created && !r1.idempotent_replay);

        let r2 = create_receiving(&mut conn, req).unwrap();
        assert!(!r2.created && r2.idempotent_replay);
    }

    #[test]
    fn test_result_flags_req202_return() {
        // REQ-202: 返品・交換記録 — INV-5: 冪等性 — 結果フラグ不変条件（返品）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "FLAG-RT1", 10);

        let req = make_return_req("flag-rt1", vec![return_item("FLAG-RT1", "in", 1)]);
        let r1 = create_return(&mut conn, req.clone()).unwrap();
        assert!(r1.created && !r1.idempotent_replay);

        let r2 = create_return(&mut conn, req).unwrap();
        assert!(!r2.created && r2.idempotent_replay);
    }

    #[test]
    fn test_result_flags_req204_disposal() {
        // REQ-204: 廃棄・破損記録 — INV-5: 冪等性 — 結果フラグ不変条件（廃棄）
        let (_dir, mut conn) = setup_test_db();
        create_test_product(&conn, "FLAG-D1", 10);

        let req = make_disposal_req("flag-d1", vec![disposal_item("FLAG-D1", 1, 100, "テスト")]);
        let r1 = create_disposal(&mut conn, req.clone()).unwrap();
        assert!(r1.created && !r1.idempotent_replay);

        let r2 = create_disposal(&mut conn, req).unwrap();
        assert!(!r2.created && r2.idempotent_replay);
    }

    #[test]
    fn test_result_flags_req203_manual_sale_three_states() {
        // REQ-203: 手動販売出庫 — INV-5: 冪等性 — 結果フラグ不変条件（手動販売3状態）
        let (_dir, mut conn) = setup_test_db();
        create_plu_exported_product(&conn, "FLAG-MS1", 10);

        // 状態1: 確認待ち
        let req1 = make_manual_sale_req("flag-ms1", vec![manual_sale_item("FLAG-MS1", 1, 500)]);
        let r1 = create_manual_sale(&mut conn, req1).unwrap();
        assert!(!r1.created && !r1.idempotent_replay && r1.needs_confirmation);

        // 状態2: 新規作成
        let mut req2 = make_manual_sale_req("flag-ms1", vec![manual_sale_item("FLAG-MS1", 1, 500)]);
        req2.confirmation_token = r1.confirmation_token;
        let r2 = create_manual_sale(&mut conn, req2).unwrap();
        assert!(r2.created && !r2.idempotent_replay && !r2.needs_confirmation);

        // 状態3: 冪等リプレイ
        let req3 = make_manual_sale_req("flag-ms1", vec![manual_sale_item("FLAG-MS1", 1, 500)]);
        let r3 = create_manual_sale(&mut conn, req3).unwrap();
        assert!(!r3.created && r3.idempotent_replay && !r3.needs_confirmation);
    }

    /// pub use リーク検出テスト
    ///
    /// mod.rs が re-export する全てのパブリックシンボルをインポートし、
    /// コンパイルが通ることで「公開 API に漏れがない」ことを保証する。
    #[test]
    fn test_pub_use_req201_leak_detection() {
        // REQ-201: 入庫記録 — pub use リーク検出: 公開APIに漏れがないこと
        // 型のインポートが通ればOK（実行時チェックは不要）
        use crate::biz::inventory_service::common::apply_stock_change;
        use crate::biz::inventory_service::{
            create_disposal,
            create_manual_sale,
            create_receiving,
            create_return,
            // 一覧取得
            list_disposals,
            list_receivings,
            list_returns,
            // disposal
            DisposalCreateRequest,
            DisposalCreateResult,
            // manual_sale
            ManualSaleCreateRequest,
            ManualSaleCreateResult,
            // receiving
            ReceivingCreateRequest,
            ReceivingCreateResult,
            // returns
            ReturnCreateRequest,
            ReturnCreateResult,
            // common
            StockChangeOutcome,
        };

        // 型が存在することのコンパイル時検証（実行時は何もしない）
        let _ = std::mem::size_of::<StockChangeOutcome>();
        let _ = std::mem::size_of::<ReceivingCreateRequest>();
        let _ = std::mem::size_of::<ReceivingItemInput>();
        let _ = std::mem::size_of::<ReceivingCreateResult>();
        let _ = std::mem::size_of::<ReturnCreateRequest>();
        let _ = std::mem::size_of::<ReturnItemInput>();
        let _ = std::mem::size_of::<ReturnCreateResult>();
        let _ = std::mem::size_of::<ManualSaleCreateRequest>();
        let _ = std::mem::size_of::<ManualSaleItemInput>();
        let _ = std::mem::size_of::<ManualSaleCreateResult>();
        let _ = std::mem::size_of::<DisposalCreateRequest>();
        let _ = std::mem::size_of::<DisposalItemInput>();
        let _ = std::mem::size_of::<DisposalCreateResult>();

        // 関数シンボルの存在確認
        // 31 §12.2: 第 1 引数は借りた transaction に限る（通常の接続に戻すとここでコンパイルが止まる）
        let _ = apply_stock_change as fn(&rusqlite::Transaction<'_>, _, _, _, _, _, _, _) -> _;
        let _ = create_receiving as fn(_, _) -> _;
        let _ = create_return as fn(_, _) -> _;
        let _ = create_manual_sale as fn(_, _) -> _;
        let _ = create_disposal as fn(_, _) -> _;
        let _ = list_receivings as fn(_, _) -> _;
        let _ = list_returns as fn(_, _) -> _;
        let _ = list_disposals as fn(_, _) -> _;
    }
}
