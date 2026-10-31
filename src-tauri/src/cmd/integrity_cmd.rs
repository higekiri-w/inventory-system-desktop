//! CMD-11 部分: 整合性チェックコマンド群
//!
//! docs/function-design/42-cmd-sales-stocktake.md §22.7 に基づく実装。
//! 本モジュールは BIZ-07 に対応する2コマンドのみを扱う。

use crate::biz::integrity_service;
use crate::cmd::{AppState, CmdError};
use tauri::State;

/// 在庫整合性チェックを実行する
///
/// 全商品の写しの 4 項目（在庫数・要再確認の印・範囲）と、記録から求め直した値を突合する（36）。
/// 読み取り専用。
#[tauri::command]
#[specta::specta]
pub fn run_integrity_check(
    state: State<AppState>,
) -> Result<integrity_service::IntegrityResult, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    integrity_service::run_integrity_check(&conn).map_err(CmdError::from)
}

/// 指定商品の在庫を整合性チェック結果に基づいて補正する
///
/// 写しと記録から求め直した値が違う商品だけ、写しを求め直しの結果に補正し、操作ログに記録する（36）。
#[tauri::command]
#[specta::specta]
pub fn fix_integrity(
    state: State<AppState>,
    product_codes: Vec<String>,
) -> Result<integrity_service::IntegrityFixResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    integrity_service::fix_integrity(&mut conn, &product_codes).map_err(CmdError::from)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::CmdErrorKind;
    use crate::db::test_support::setup_test_db;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tauri::Manager;

    fn app_state_for_test(conn: crate::db::DbConnection) -> AppState {
        AppState {
            db: Mutex::new(conn),
            preview_cache: Mutex::new(HashMap::new()),
            daily_report_preview_cache: Mutex::new(HashMap::new()),
            register_sd_scan_cache: Mutex::new(HashMap::new()),
        }
    }

    #[test]
    fn test_fix_integrity_req904_t6_empty_codes_validation() {
        // REQ-904 / T6: 実コマンド関数を空 codes で呼ぶ
        let (_dir, conn) = setup_test_db();
        let app = tauri::test::mock_builder()
            .manage(app_state_for_test(conn))
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();

        let err = fix_integrity(app.state::<AppState>(), vec![]).unwrap_err();

        assert_eq!(err.kind, CmdErrorKind::Validation);
        assert_eq!(err.message, "補正対象の商品が指定されていません");
        assert_eq!(err.field, None);
    }
}
