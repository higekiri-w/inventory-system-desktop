use crate::biz::csv_import_service::CsvImportErrorType;
use crate::biz::plu_export_service::ExportMode;
use crate::biz::sales_service::{DailySaleSource, SalesMode};
use crate::db::disposal_repo::DisposalType;
use crate::db::inventory_repo::{MovementType, ReferenceType};
use crate::db::manual_sale_repo::ManualSaleReason;
use crate::db::product_repo::{ProductStockUnit, ProductTaxRate};
use crate::db::return_repo::{ReturnDirection, ReturnExchangeType};
use crate::db::sales_repo::CsvImportStatus;

/// `ProductStockUnit` の variant と wire の文字列の対（SPEC-UNIT-D1 の表の順）
const PRODUCT_STOCK_UNIT_WIRE: [(ProductStockUnit, &str); 10] = [
    (ProductStockUnit::Pcs, "pcs"),
    (ProductStockUnit::Sheet, "sheet"),
    (ProductStockUnit::Hon, "hon"),
    (ProductStockUnit::Bag, "bag"),
    (ProductStockUnit::Box, "box"),
    (ProductStockUnit::Kumi, "kumi"),
    (ProductStockUnit::Set, "set"),
    (ProductStockUnit::Ball, "ball"),
    (ProductStockUnit::Cho, "cho"),
    (ProductStockUnit::M, "m"),
];

/// wildcard の無い match で次の variant を引く（variant を足すと compile error でここを直させる）
fn next_product_stock_unit(unit: ProductStockUnit) -> Option<ProductStockUnit> {
    use ProductStockUnit as U;
    match unit {
        U::Pcs => Some(U::Sheet),
        U::Sheet => Some(U::Hon),
        U::Hon => Some(U::Bag),
        U::Bag => Some(U::Box),
        U::Box => Some(U::Kumi),
        U::Kumi => Some(U::Set),
        U::Set => Some(U::Ball),
        U::Ball => Some(U::Cho),
        U::Cho => Some(U::M),
        U::M => None,
    }
}

#[test]
fn test_product_stock_unit_all_req101_lists_every_variant_once() {
    // REQ-101 / SPEC-UNIT-D1・D10（X12）: ALL は 10 個・重複なしで、対の表の variant の並びと一致する
    let all = ProductStockUnit::ALL;
    assert_eq!(all.len(), 10);
    let by_match: Vec<ProductStockUnit> =
        std::iter::successors(Some(ProductStockUnit::Pcs), |unit| {
            next_product_stock_unit(*unit)
        })
        .collect();
    assert_eq!(by_match, all, "match で数えた variant と ALL が一致する");
    let codes: std::collections::HashSet<&str> = all.iter().map(|unit| unit.as_str()).collect();
    assert_eq!(codes.len(), all.len(), "重複なし");
    assert_eq!(all, PRODUCT_STOCK_UNIT_WIRE.map(|(unit, _)| unit));
    // 一覧に無い kg と、vB で外した roll・cm（D-128）は wire でも code でも受けない
    for code in ["kg", "roll", "cm"] {
        assert!(serde_json::from_str::<ProductStockUnit>(&format!("\"{code}\"")).is_err());
        assert_eq!(ProductStockUnit::from_code(code), None, "{code}");
    }
    assert_eq!(
        ProductStockUnit::from_code("ball"),
        Some(ProductStockUnit::Ball)
    );
}

fn assert_roundtrip<T>(value: T, wire: &str)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        format!("\"{wire}\"")
    );
    assert_eq!(
        serde_json::from_str::<T>(&format!("\"{wire}\"")).unwrap(),
        value
    );
}

fn assert_serialize<T: serde::Serialize>(value: T, wire: &str) {
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        format!("\"{wire}\"")
    );
}

#[test]
fn test_request_response_enum_wire_roundtrip_req202_req203_req204_req401_req502() {
    for (value, wire) in [
        (ReturnExchangeType::Return, "return"),
        (ReturnExchangeType::Exchange, "exchange"),
    ] {
        assert_roundtrip(value, wire);
    }
    for (value, wire) in [(ReturnDirection::In, "in"), (ReturnDirection::Out, "out")] {
        assert_roundtrip(value, wire);
    }
    for (value, wire) in [
        (DisposalType::Disposal, "disposal"),
        (DisposalType::Damage, "damage"),
        (DisposalType::Other, "other"),
    ] {
        assert_roundtrip(value, wire);
    }
    for (value, wire) in [
        (ManualSaleReason::PluUnregistered, "plu_unregistered"),
        (ManualSaleReason::Other, "other"),
    ] {
        assert_roundtrip(value, wire);
    }
    for (value, wire) in [
        (ProductTaxRate::Rate10, "10"),
        (ProductTaxRate::Rate8, "8"),
        (ProductTaxRate::Rate0, "0"),
    ] {
        assert_roundtrip(value, wire);
    }
    for (value, wire) in PRODUCT_STOCK_UNIT_WIRE {
        assert_roundtrip(value, wire);
        assert_eq!(value.as_str(), wire);
    }
    for (value, wire) in [(ExportMode::Full, "full"), (ExportMode::Diff, "diff")] {
        assert_roundtrip(value, wire);
    }
    for (value, wire) in [
        (SalesMode::ByProduct, "by_product"),
        (SalesMode::ByDepartment, "by_department"),
    ] {
        assert_roundtrip(value, wire);
    }
    for (value, wire) in [
        (MovementType::SaleAuto, "sale_auto"),
        (MovementType::SaleManual, "sale_manual"),
        (MovementType::Receiving, "receiving"),
        (MovementType::Return, "return"),
        (MovementType::Disposal, "disposal"),
        (MovementType::Stocktake, "stocktake"),
    ] {
        assert_roundtrip(value, wire);
    }
}

#[test]
fn test_response_only_enum_wire_serialize_req303_req501() {
    for (value, wire) in [
        (ReferenceType::CsvImport, "csv_import"),
        (ReferenceType::ManualSale, "manual_sale"),
        (ReferenceType::ReceivingRecord, "receiving_record"),
        (ReferenceType::ReturnRecord, "return_record"),
        (ReferenceType::DisposalRecord, "disposal_record"),
        (ReferenceType::Stocktake, "stocktake"),
    ] {
        assert_serialize(value, wire);
    }
    // D-118 b2（Matrix W1）: 在庫変動の履歴の行の種類
    for (value, wire) in [
        (
            crate::biz::inventory_service::StockHistoryKind::Movement,
            "movement",
        ),
        (
            crate::biz::inventory_service::StockHistoryKind::Count,
            "count",
        ),
    ] {
        assert_serialize(value, wire);
    }
    for (value, wire) in [
        (DailySaleSource::Auto, "auto"),
        (DailySaleSource::Manual, "manual"),
    ] {
        assert_serialize(value, wire);
    }
    for (value, wire) in [
        (CsvImportStatus::Completed, "completed"),
        (CsvImportStatus::CompletedPartial, "completed_partial"),
        (CsvImportStatus::RolledBack, "rolled_back"),
    ] {
        assert_serialize(value, wire);
    }
    for (value, wire) in [
        (CsvImportErrorType::UnmatchedProduct, "unmatched_product"),
        (CsvImportErrorType::InvalidFormat, "invalid_format"),
        (CsvImportErrorType::InvalidJan, "invalid_jan"),
        (CsvImportErrorType::InvalidNumber, "invalid_number"),
    ] {
        assert_serialize(value, wire);
    }
    // D-118 b1（Matrix W6、packet の横断の約束 Z14）: 計数・完了・要再確認の一覧・在庫連動の拒否の有限の値
    {
        use crate::biz::stocktake_service::{
            CompletionBlockReason, CountRecoveryAction, RecountReason, StockCountKind,
            StocktakeRecoveryCode,
        };
        for (value, wire) in [
            (StockCountKind::Default, "default"),
            (StockCountKind::Fresh, "fresh"),
        ] {
            assert_serialize(value, wire);
        }
        for (value, wire) in [
            (
                RecountReason::UndeterminedSettlement,
                "undetermined_settlement",
            ),
            (RecountReason::SameDateMovement, "same_date_movement"),
            (RecountReason::ConflictingCounts, "conflicting_counts"),
        ] {
            assert_serialize(value, wire);
        }
        for (value, wire) in [
            (CountRecoveryAction::ActiveCount, "active_count"),
            (CountRecoveryAction::CountNow, "count_now"),
        ] {
            assert_serialize(value, wire);
        }
        for (value, wire) in [
            (CompletionBlockReason::Recheck, "recheck"),
            (CompletionBlockReason::Uncounted, "uncounted"),
            (
                CompletionBlockReason::SettlementPending,
                "settlement_pending",
            ),
        ] {
            assert_serialize(value, wire);
        }
        for (value, wire) in [
            (StocktakeRecoveryCode::RecountRequired, "recount_required"),
            (
                StocktakeRecoveryCode::CompletionBlocked,
                "completion_blocked",
            ),
        ] {
            assert_serialize(value, wire);
        }
        assert_serialize(crate::cmd::CmdErrorKind::StocktakeGuard, "stocktake_guard");
    }
}

#[test]
fn test_request_enum_invalid_literals_are_rejected_req202_req203_req204_req303_req401_req502() {
    macro_rules! rejects {
        ($ty:ty) => {
            assert!(serde_json::from_str::<$ty>("\"invalid\"").is_err())
        };
    }
    rejects!(ReturnExchangeType);
    rejects!(ReturnDirection);
    rejects!(DisposalType);
    rejects!(ManualSaleReason);
    rejects!(ProductTaxRate);
    rejects!(ProductStockUnit);
    assert!(serde_json::from_str::<ProductStockUnit>("\"kg\"").is_err());
    rejects!(ExportMode);
    rejects!(SalesMode);
    rejects!(MovementType);

    #[derive(serde::Deserialize)]
    struct Query {
        movement_type: Option<MovementType>,
    }
    assert_eq!(
        serde_json::from_str::<Query>(r#"{"movement_type":null}"#)
            .unwrap()
            .movement_type,
        None
    );
    assert_eq!(
        serde_json::from_str::<Query>(r#"{}"#)
            .unwrap()
            .movement_type,
        None
    );
    assert!(serde_json::from_str::<Query>(r#"{"movement_type":"invalid"}"#).is_err());
}

#[test]
fn test_canonical_wire_matches_serde_req202_req203_req204_req303_req401() {
    macro_rules! parity {
        ($value:expr) => {{
            let value = $value;
            assert_eq!(
                serde_json::to_string(&value).unwrap(),
                format!("\"{}\"", value.as_str())
            );
        }};
    }
    parity!(ReturnExchangeType::Return);
    parity!(ReturnExchangeType::Exchange);
    parity!(ReturnDirection::In);
    parity!(ReturnDirection::Out);
    parity!(DisposalType::Disposal);
    parity!(DisposalType::Damage);
    parity!(DisposalType::Other);
    parity!(ManualSaleReason::PluUnregistered);
    parity!(ManualSaleReason::Other);
    parity!(CsvImportStatus::Completed);
    parity!(CsvImportStatus::CompletedPartial);
    parity!(CsvImportStatus::RolledBack);
    parity!(CsvImportErrorType::UnmatchedProduct);
    parity!(CsvImportErrorType::InvalidFormat);
    parity!(CsvImportErrorType::InvalidJan);
    parity!(CsvImportErrorType::InvalidNumber);
    parity!(ProductTaxRate::Rate10);
    parity!(ProductTaxRate::Rate8);
    parity!(ProductTaxRate::Rate0);
    for (unit, _) in PRODUCT_STOCK_UNIT_WIRE {
        parity!(unit);
    }
    parity!(MovementType::SaleAuto);
    parity!(MovementType::SaleManual);
    parity!(MovementType::Receiving);
    parity!(MovementType::Return);
    parity!(MovementType::Disposal);
    parity!(MovementType::Stocktake);
    parity!(ReferenceType::CsvImport);
    parity!(ReferenceType::ManualSale);
    parity!(ReferenceType::ReceivingRecord);
    parity!(ReferenceType::ReturnRecord);
    parity!(ReferenceType::DisposalRecord);
    parity!(ReferenceType::Stocktake);
}
