//! ビジネスロジック層
//!
//! ARCHITECTURE.md: UI → CMD → BIZ → IO の一方向。
//! トランザクション管理はBIZ層が制御する。

pub mod csv_import_service;
pub mod daily_report_import_service;
pub mod integrity_service;
pub mod inventory_service;
pub mod jan_code;
pub mod plu_export_service;
pub mod product_service;
pub mod sales_service;
pub mod stock_ledger;
pub mod stocktake_service;
pub mod system_service;
pub mod unit_amount;

// CMD層がBIZ経由で直接使用するDB型の re-export（CMD→db直接依存を避けるために維持）
#[allow(unused_imports)]
pub use crate::db::product_repo::{Department, Supplier};
#[allow(unused_imports)]
pub use crate::db::product_repo::{
    ProductBulkFilter, ProductSearchQuery, ProductStockUnit, ProductWithRelations,
};
#[allow(unused_imports)]
pub use crate::db::sales_repo::CsvImport;
#[allow(unused_imports)]
pub use crate::db::stocktake_repo::{LastStocktakeSummary, Stocktake, StocktakeProgress};
#[allow(unused_imports)]
pub use crate::db::system_repo::{AppSetting, OperationLog};
#[allow(unused_imports)]
pub use crate::db::DbConnection;
#[allow(unused_imports)]
pub use crate::db::DbError;
#[allow(unused_imports)]
pub use crate::db::PaginatedResult;
#[allow(unused_imports)]
pub use stocktake_service::StocktakeItemDetail;
// CMD-02〜05 用 re-export
pub use crate::db::disposal_repo::{
    DisposalRecordDetail, DisposalRecordSummary, InventoryRecordQuery, InventoryRecordSummary,
};
pub use crate::db::inventory_common::ListQuery;
pub use crate::db::manual_sale_repo::ManualSaleRecordDetail;
pub use crate::db::receiving_repo::{ReceivingRecordDetail, ReceivingRecordWithSupplier};
pub use crate::db::return_repo::{ReturnRecordDetail, ReturnRecordSummary};
// CMD-06 用 re-export
pub use crate::db::inventory_repo::MovementQuery;
// 業務記録の詳細の wire 型（D-083 の re-export の許可表に載る。在庫変動の画面は StockHistoryItem）
#[allow(unused_imports)]
pub use crate::db::inventory_repo::MovementRecord;
pub use crate::db::product_repo::StockDetail;
pub use inventory_service::StockHistoryItem;

use std::fmt;

/// BIZ層のエラー型
///
/// 30-biz-product-service.md §4.7
#[derive(Debug)]
pub enum BizError {
    /// 入力バリデーション失敗
    ValidationFailed(String),
    /// 入力バリデーション失敗（wire field付き）
    ValidationFailedAt { message: String, field: String },
    /// 対象レコードが見つからない
    NotFound(String),
    /// 商品コード重複
    DuplicateProductCode(String),
    /// IO層エラーのラップ
    DatabaseError(DbError),
    /// インポートエラー（BIZ-03 CSV取込みパイプライン）
    ImportError(String),
    /// 同じ精算の別の版が active（BIZ-03 の順 4 の拒否。文が名指す取込みID を構造でも持つ。32）
    SettlementConflict { message: String, csv_import_id: i64 },
    /// エクスポートエラー（BIZ-04 PLU書出し）
    ExportError(String),
    /// 同じ冪等キーで異なる内容のリクエスト
    IdempotencyConflict(String),
    /// 棚卸し進行中（BIZ-06）
    StocktakeInProgress(String),
    /// 棚卸し未進行（BIZ-06）
    StocktakeNotInProgress(String),
    /// SD から読んだ原本の写しを保存できなかった（BIZ-08-D5）。io error の種類は診断にだけ使う
    SourceCopyFailed(std::io::ErrorKind),
    /// 棚卸しの守りの拒否と回復先（40。完了の拒否・在庫連動を始める拒否）
    StocktakeGuard {
        message: String,
        /// 大きいので Box（`Result<_, BizError>` の Err を小さく保つ。wire の形は変わらない）
        recovery: Box<stocktake_service::StocktakeRecovery>,
    },
}

impl fmt::Display for BizError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BizError::ValidationFailed(msg) => write!(f, "バリデーションエラー: {}", msg),
            BizError::ValidationFailedAt { message, field } => {
                write!(f, "バリデーションエラー({}): {}", field, message)
            }
            BizError::NotFound(msg) => write!(f, "見つかりません: {}", msg),
            BizError::DuplicateProductCode(code) => {
                write!(f, "商品コード重複: {}", code)
            }
            BizError::DatabaseError(e) => write!(f, "データベースエラー: {}", e),
            BizError::ImportError(msg) => write!(f, "インポートエラー: {}", msg),
            BizError::SettlementConflict { message, .. } => {
                write!(f, "インポートエラー: {}", message)
            }
            BizError::ExportError(msg) => write!(f, "エクスポートエラー: {}", msg),
            BizError::IdempotencyConflict(msg) => write!(f, "冪等性キー競合: {}", msg),
            BizError::StocktakeInProgress(msg) => write!(f, "棚卸し進行中: {}", msg),
            BizError::StocktakeNotInProgress(msg) => write!(f, "棚卸し未進行: {}", msg),
            BizError::SourceCopyFailed(kind) => write!(f, "SD の写しの保存に失敗: {:?}", kind),
            BizError::StocktakeGuard { message, .. } => write!(f, "棚卸しの確認: {}", message),
        }
    }
}

impl std::error::Error for BizError {}

/// DbError → BizError の自動変換
///
/// DuplicateKey は DuplicateProductCode に正規化しない（呼び出し元で判断）。
/// INSERT 時の DuplicateKey → DuplicateProductCode の変換は各BIZ関数内で明示的に行う。
impl From<DbError> for BizError {
    fn from(err: DbError) -> Self {
        BizError::DatabaseError(err)
    }
}
