//! Tauriコマンド層（CMD）
//!
//! ARCHITECTURE.md: UI → CMD → BIZ → IO の一方向。
//! CMD層は薄いラッパー。業務ルールを持たない。

pub mod csv_import_cmd;
pub mod daily_report_import_cmd;
pub mod disposal_cmd;
pub mod integrity_cmd;
pub mod inventory_cmd;
pub mod manual_sale_cmd;
pub mod plu_export_cmd;
pub mod product_cmd;
pub mod receiving_cmd;
pub mod return_cmd;
pub mod sales_cmd;
pub mod settings_cmd;
pub mod stocktake_cmd;

use crate::biz::csv_import_service::CachedPreview;
use crate::biz::daily_report_import_service::{
    CachedDailyReportPreview, DailyReportSdScanSnapshot,
};
use crate::biz::{BizError, DbConnection};
use std::collections::HashMap;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Tauri管理のアプリケーション状態
///
/// DB接続とCSV取込みPreviewキャッシュを保持する。
/// docs/function-design/41-cmd-pos.md §17.3
pub struct AppState {
    /// SQLite接続（Mutex で排他制御）
    pub db: Mutex<DbConnection>,
    /// CSV取込みPreviewキャッシュ（token → CachedPreview）
    pub preview_cache: Mutex<HashMap<String, CachedPreview>>,
    /// 日報取込みPreviewキャッシュ（token → CachedDailyReportPreview）
    pub daily_report_preview_cache: Mutex<HashMap<String, CachedDailyReportPreview>>,
    /// レジの SD の scan の snapshot（scan_token → snapshot。最新の 1 つだけ、CMD-12-D1）
    pub register_sd_scan_cache: Mutex<HashMap<String, DailyReportSdScanSnapshot>>,
}

// ---------------------------------------------------------------------------
// CmdError
// ---------------------------------------------------------------------------

/// UI向けエラー分類
///
/// docs/function-design/40-cmd-product.md §5.3 + 41-cmd-pos.md §17.4
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum CmdErrorKind {
    Validation,
    Duplicate,
    NotFound,
    Internal,
    ImportError,
    ExportError,
    IdempotencyConflict,
    StocktakeInProgress,
    StocktakeNotInProgress,
    StocktakeGuard,
    RestoreFailedRecovered,
    RestoreFailedUnrecoverable,
    RestoreDurabilityUnknown,
}

/// UI向け構造化エラー
///
/// docs/function-design/40-cmd-product.md §5.3 + 41-cmd-pos.md §17.4
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct CmdError {
    /// エラー分類（`CmdErrorKind` の13値）
    pub kind: CmdErrorKind,
    /// 利用者向け日本語メッセージ
    pub message: String,
    /// バリデーションエラー時のフィールド名
    pub field: Option<String>,
    /// 診断ログとの相関 ID（internal / restore_* 系のみ）
    pub error_id: Option<String>,
    /// 棚卸しの守りの拒否の回復先（`stocktake_guard` だけ Some。40）
    // 大きいので Box（serde と specta は中身の型として書くので wire は変わらない）
    pub stocktake_recovery: Option<Box<crate::biz::stocktake_service::StocktakeRecovery>>,
    /// 拒否の文が名指す取込みID（BIZ-03 の順 4 の拒否〈kind `import_error`〉だけ Some、ほかは null。
    /// TS では任意の field。40・41 §17.4）
    #[serde(default)]
    pub related_csv_import_id: Option<i64>,
}

impl CmdError {
    /// 内部エラー（DB接続取得失敗等）
    ///
    /// §70.7.1: CmdError::internal で直接生成するケースも ERROR ログを出力する。
    fn internal(message: &str, detail: impl std::fmt::Display) -> Self {
        Self::correlated(CmdErrorKind::Internal, message, detail)
    }

    pub(crate) fn restore_failed_recovered(message: &str) -> Self {
        Self::restore(CmdErrorKind::RestoreFailedRecovered, message)
    }

    pub(crate) fn restore_failed_unrecoverable(message: &str, detail: &str) -> Self {
        Self::restore_with_detail(CmdErrorKind::RestoreFailedUnrecoverable, message, detail)
    }

    pub(crate) fn restore_durability_unknown(message: &str, detail: &str) -> Self {
        Self::restore_with_detail(CmdErrorKind::RestoreDurabilityUnknown, message, detail)
    }

    fn restore(kind: CmdErrorKind, message: &str) -> Self {
        Self::restore_with_detail(kind, message, message)
    }

    fn restore_with_detail(kind: CmdErrorKind, message: &str, detail: &str) -> Self {
        Self::correlated(kind, message, detail)
    }

    fn correlated(kind: CmdErrorKind, message: &str, detail: impl std::fmt::Display) -> Self {
        let error_id = format!(
            "E-{}-{}",
            chrono::Local::now().format("%Y%m%d-%H%M%S"),
            &uuid::Uuid::new_v4().simple().to_string()[..4]
        );
        tracing::error!(kind = ?kind, error_id, detail = %detail, "CMD層相関エラー");
        Self {
            kind,
            message: message.to_string(),
            field: None,
            error_id: Some(error_id),
            stocktake_recovery: None,
            related_csv_import_id: None,
        }
    }
}

/// BizError → CmdError の変換
///
/// docs/function-design/40-cmd-product.md §5.3 + 41-cmd-pos.md §17.4
/// §70.7.1: エラー境界での1回記録。全variantで tracing::error! を出力する。
impl From<BizError> for CmdError {
    fn from(err: BizError) -> Self {
        if let BizError::DatabaseError(detail) = &err {
            return CmdError::internal(
                "データベースエラーが発生しました。もう一度お試しください",
                detail,
            );
        }
        // 45 §45.7: 写しの失敗は internal（UI が preview に戻り、同じ token で再試行できる）
        if let BizError::SourceCopyFailed(kind) = &err {
            return CmdError::internal(
                "SD から読んだファイルの写しを PC に保存できなかったため、取り込みませんでした。PC の空き容量を確かめて、もう一度取り込んでください。",
                format!("{kind:?}"),
            );
        }
        tracing::error!(error = %err, "CMD層エラー");
        match err {
            BizError::ValidationFailed(msg) => CmdError {
                kind: CmdErrorKind::Validation,
                message: msg,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::ValidationFailedAt { message, field } => CmdError {
                kind: CmdErrorKind::Validation,
                message,
                field: Some(field),
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::NotFound(msg) => CmdError {
                kind: CmdErrorKind::NotFound,
                message: msg,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::DuplicateProductCode(code) => CmdError {
                kind: CmdErrorKind::Duplicate,
                message: format!("この商品コードは既に使用されています: {}", code),
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::DatabaseError(_) | BizError::SourceCopyFailed(_) => {
                unreachable!("handled before generic logging")
            }
            BizError::ImportError(msg) => CmdError {
                kind: CmdErrorKind::ImportError,
                message: msg,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::SettlementConflict {
                message,
                csv_import_id,
            } => CmdError {
                kind: CmdErrorKind::ImportError,
                message,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: Some(csv_import_id),
            },
            BizError::ExportError(msg) => CmdError {
                kind: CmdErrorKind::ExportError,
                message: msg,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::IdempotencyConflict(msg) => CmdError {
                kind: CmdErrorKind::IdempotencyConflict,
                message: msg,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::StocktakeInProgress(msg) => CmdError {
                kind: CmdErrorKind::StocktakeInProgress,
                message: msg,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::StocktakeNotInProgress(msg) => CmdError {
                kind: CmdErrorKind::StocktakeNotInProgress,
                message: msg,
                field: None,
                error_id: None,
                stocktake_recovery: None,
                related_csv_import_id: None,
            },
            BizError::StocktakeGuard { message, recovery } => CmdError {
                kind: CmdErrorKind::StocktakeGuard,
                message,
                field: None,
                error_id: None,
                stocktake_recovery: Some(recovery),
                related_csv_import_id: None,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbError;
    use regex::Regex;

    #[test]
    fn test_cmd_error_req905_from_import_error() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        let biz_err = BizError::ImportError("テストエラー".to_string());
        let cmd_err: CmdError = biz_err.into();
        assert_eq!(cmd_err.kind, CmdErrorKind::ImportError);
        assert_eq!(cmd_err.message, "テストエラー");
        assert!(cmd_err.field.is_none());
    }

    #[test]
    fn test_cmd_error_req905_from_validation_failed() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        let biz_err = BizError::ValidationFailed("入力エラー".to_string());
        let cmd_err: CmdError = biz_err.into();
        assert_eq!(cmd_err.kind, CmdErrorKind::Validation);
        assert_eq!(cmd_err.message, "入力エラー");
    }

    #[test]
    fn test_cmd_error_req205_from_validation_failed_at_preserves_field() {
        // REQ-205 / BIZ-06-VAL-D1: BIZ由来のfield付きvalidationをwire tripleへ変換する。
        let biz_err = BizError::ValidationFailedAt {
            message: "ページ番号は1以上で指定してください".to_string(),
            field: "page".to_string(),
        };
        let cmd_err: CmdError = biz_err.into();

        assert_eq!(cmd_err.kind, CmdErrorKind::Validation);
        assert_eq!(cmd_err.message, "ページ番号は1以上で指定してください");
        assert_eq!(cmd_err.field.as_deref(), Some("page"));
    }

    #[test]
    fn test_cmd_error_req905_from_not_found() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        let biz_err = BizError::NotFound("見つかりません".to_string());
        let cmd_err: CmdError = biz_err.into();
        assert_eq!(cmd_err.kind, CmdErrorKind::NotFound);
    }

    #[test]
    fn test_cmd_error_req905_from_database_error() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        let db_err = DbError::QueryFailed("test".to_string());
        let biz_err = BizError::DatabaseError(db_err);
        let cmd_err: CmdError = biz_err.into();
        assert_eq!(cmd_err.kind, CmdErrorKind::Internal);
        assert!(cmd_err.message.contains("データベースエラー"));
        assert!(cmd_err.error_id.is_some());
    }

    #[test]
    fn test_cmd_error_req905_from_duplicate() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        let biz_err = BizError::DuplicateProductCode("TEST-001".to_string());
        let cmd_err: CmdError = biz_err.into();
        assert_eq!(cmd_err.kind, CmdErrorKind::Duplicate);
        assert!(cmd_err.message.contains("TEST-001"));
    }

    #[test]
    fn test_cmd_error_req905_restore_failure_kinds_are_stable() {
        // REQ-905 / MNT-01-D4 / Matrix F1
        assert_eq!(
            CmdError::restore_failed_recovered("recovered").kind,
            CmdErrorKind::RestoreFailedRecovered
        );
        assert_eq!(
            CmdError::restore_failed_unrecoverable("fatal", "detail").kind,
            CmdErrorKind::RestoreFailedUnrecoverable
        );
        assert_eq!(
            CmdError::restore_durability_unknown("unknown", "detail").kind,
            CmdErrorKind::RestoreDurabilityUnknown
        );
    }

    #[test]
    fn test_internal_error_id_req700_wire_log_match() {
        // REQ-700 / CMD-ERR-D1: wire と診断ログは同じ相関 ID を共有する。
        let raw_detail = "synthetic sqlite detail: /tmp/synthetic.db";
        let (cmd_err, logs) = crate::test_tracing::capture(|| {
            CmdError::internal("在庫情報の取得でエラーが発生しました", raw_detail)
        });
        let error_id = cmd_err.error_id.as_deref().expect("internal error_id");
        let pattern = Regex::new(r"^E-\d{8}-\d{6}-[0-9a-f]{4}$").unwrap();

        assert!(
            pattern.is_match(error_id),
            "unexpected error_id: {error_id}"
        );
        assert!(logs.contains(error_id), "captured logs: {logs:?}");
        assert!(logs.contains(raw_detail), "captured logs: {logs:?}");
    }

    #[test]
    fn test_internal_message_req700_excludes_raw_detail() {
        // REQ-700 / CMD-ERR-D2: raw detail は wire message に混ぜない。
        let raw_detail = "synthetic OS error: permission denied /tmp/synthetic.db";
        let cmd_err =
            CmdError::internal("バックアップ一覧の取得でエラーが発生しました", raw_detail);

        assert_eq!(
            cmd_err.message,
            "バックアップ一覧の取得でエラーが発生しました"
        );
        assert!(!cmd_err.message.contains(raw_detail));
        assert!(!cmd_err.message.contains("/tmp/synthetic.db"));
    }

    #[test]
    fn test_restore_kinds_req700_carry_error_id() {
        // REQ-700 / CMD-ERR-D1 / 68 §68.7: restore 3 kind は相関 ID を持つ。
        let errors = [
            CmdError::restore_failed_recovered("recovered"),
            CmdError::restore_failed_unrecoverable("fatal", "fatal detail"),
            CmdError::restore_durability_unknown("unknown", "unknown detail"),
        ];
        let pattern = Regex::new(r"^E-\d{8}-\d{6}-[0-9a-f]{4}$").unwrap();

        for error in errors {
            let error_id = error.error_id.as_deref().expect("restore error_id");
            assert!(
                pattern.is_match(error_id),
                "unexpected error_id: {error_id}"
            );
        }
    }

    #[test]
    fn test_plu_format_failure_req402_maps_to_export_error() {
        // REQ-402 / BIZ-04-D2: PLU 書出し失敗は import_error と分離する。
        let cmd_err: CmdError = BizError::ExportError("PLU生成失敗".to_string()).into();

        assert_eq!(cmd_err.kind, CmdErrorKind::ExportError);
        assert_eq!(cmd_err.message, "PLU生成失敗");
        assert!(cmd_err.error_id.is_none());
    }

    #[test]
    fn test_cmd_error_req905_from_idempotency_conflict() {
        // REQ-905: 設定管理（設定CRUD/エラー変換）
        let biz_err = BizError::IdempotencyConflict("競合".to_string());
        let cmd_err: CmdError = biz_err.into();
        assert_eq!(cmd_err.kind, CmdErrorKind::IdempotencyConflict);
    }

    #[test]
    fn test_cmd_error_req401_settlement_conflict_carries_related_csv_import_id() {
        // REQ-401 / 40 §5.3・41 §17.4（Matrix W3）: 順 4 の拒否は kind import_error のまま、名指す取込みID を
        // related_csv_import_id に Some で運び、ほかの全ての variant では None
        let message =
            "同じ精算と思われる別の資料が既に取り込まれています（精算 101、取込みID: 42）";
        let cmd_err: CmdError = BizError::SettlementConflict {
            message: message.to_string(),
            csv_import_id: 42,
        }
        .into();
        assert_eq!(cmd_err.kind, CmdErrorKind::ImportError);
        assert_eq!(cmd_err.message, message);
        assert_eq!(cmd_err.related_csv_import_id, Some(42));
        assert!(cmd_err.stocktake_recovery.is_none());
        let wire = serde_json::to_value(&cmd_err).unwrap();
        assert_eq!(wire["kind"], serde_json::json!("import_error"));
        assert_eq!(wire["related_csv_import_id"], serde_json::json!(42));
        for other in [
            BizError::ValidationFailed("v".to_string()),
            BizError::ValidationFailedAt {
                message: "v".to_string(),
                field: "page".to_string(),
            },
            BizError::NotFound("n".to_string()),
            BizError::DuplicateProductCode("d".to_string()),
            BizError::DatabaseError(DbError::NotFound),
            BizError::ImportError("i".to_string()),
            BizError::ExportError("e".to_string()),
            BizError::IdempotencyConflict("c".to_string()),
            BizError::StocktakeInProgress("p".to_string()),
            BizError::StocktakeNotInProgress("p".to_string()),
            BizError::StocktakeGuard {
                message: "g".to_string(),
                recovery: Box::new(crate::biz::stocktake_service::StocktakeRecovery {
                    code: crate::biz::stocktake_service::StocktakeRecoveryCode::RecountRequired,
                    targets: Vec::new(),
                    settlement_pending: None,
                }),
            },
        ] {
            let cmd_err: CmdError = other.into();
            assert!(
                cmd_err.related_csv_import_id.is_none(),
                "{:?}",
                cmd_err.kind
            );
            let wire = serde_json::to_value(&cmd_err).unwrap();
            assert_eq!(
                wire.get("related_csv_import_id"),
                Some(&serde_json::Value::Null),
                "{wire}"
            );
        }
    }

    #[test]
    fn test_cmd_error_req205_stocktake_guard_carries_recovery() {
        // REQ-205 / 40 の回復型（b1 Matrix W1）: StocktakeGuard は kind stocktake_guard と stocktake_recovery を Some で、
        // 既存の kind では None で運ぶ
        use crate::biz::stocktake_service::{StocktakeRecovery, StocktakeRecoveryCode};
        let recovery = StocktakeRecovery {
            code: StocktakeRecoveryCode::RecountRequired,
            targets: Vec::new(),
            settlement_pending: None,
        };
        let cmd_err: CmdError = BizError::StocktakeGuard {
            message: "守りの拒否".to_string(),
            recovery: Box::new(recovery.clone()),
        }
        .into();
        assert_eq!(cmd_err.kind, CmdErrorKind::StocktakeGuard);
        assert_eq!(cmd_err.message, "守りの拒否");
        assert_eq!(cmd_err.stocktake_recovery, Some(Box::new(recovery)));
        assert_eq!(
            serde_json::to_value(&cmd_err).unwrap()["kind"],
            serde_json::json!("stocktake_guard")
        );
        for other in [
            BizError::ValidationFailed("v".to_string()),
            BizError::ValidationFailedAt {
                message: "v".to_string(),
                field: "pos_stock_sync".to_string(),
            },
            BizError::NotFound("n".to_string()),
            BizError::ImportError("i".to_string()),
            BizError::StocktakeInProgress("p".to_string()),
            BizError::StocktakeNotInProgress("p".to_string()),
            BizError::DatabaseError(DbError::NotFound),
        ] {
            let cmd_err: CmdError = other.into();
            assert_ne!(cmd_err.kind, CmdErrorKind::StocktakeGuard);
            assert!(cmd_err.stocktake_recovery.is_none(), "{:?}", cmd_err.kind);
        }
    }
}
