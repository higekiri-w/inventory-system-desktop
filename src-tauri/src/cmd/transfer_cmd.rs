//! CMD-13: 振替（詰め替え・切り分け）コマンド群
//!
//! 44-cmd-inventory.md §23.7a に基づく実装（D-143）。一覧の command は作らない
//! （`list_inventory_records` に `record_type: "transfer_record"` を渡して読む）。

use crate::biz::inventory_service;
use crate::cmd::{AppState, CmdError};
use tauri::State;

/// 振替の記録を作成する
///
/// 44-cmd-inventory.md §23.7a create_transfer
#[tauri::command]
#[specta::specta]
pub fn create_transfer(
    state: State<AppState>,
    req: inventory_service::TransferCreateRequest,
) -> Result<inventory_service::TransferCreateResult, CmdError> {
    let mut conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    inventory_service::create_transfer(&mut conn, req).map_err(CmdError::from)
}

/// 振替の記録の詳細を返す
///
/// 44-cmd-inventory.md §23.7a get_transfer_record
#[tauri::command]
#[specta::specta]
pub fn get_transfer_record(
    state: State<AppState>,
    record_id: i64,
) -> Result<inventory_service::TransferRecordDetail, CmdError> {
    let conn = state
        .db
        .lock()
        .map_err(|error| CmdError::internal("DB接続エラー", error))?;
    inventory_service::get_transfer_record(&conn, record_id).map_err(CmdError::from)
}

#[cfg(test)]
mod tests {
    use crate::biz::inventory_service::{self, TransferCreateRequest};
    use crate::cmd::{CmdError, CmdErrorKind};
    use crate::db;

    #[test]
    fn test_transfer_cmd_d143_b14_maps_biz_errors_to_cmd_kinds() {
        // 44 §23.7a・§23.3 / Test Plan B14: create_transfer と get_transfer_record の error の写像
        let mut conn = db::init_database(":memory:").unwrap();
        let validation = inventory_service::create_transfer(
            &mut conn,
            TransferCreateRequest {
                idempotency_key: "b14".to_string(),
                transfer_date: "2026-06-10".to_string(),
                items: vec![],
            },
        )
        .unwrap_err();
        assert_eq!(CmdError::from(validation).kind, CmdErrorKind::Validation);

        let not_found = inventory_service::get_transfer_record(&conn, 404).unwrap_err();
        assert_eq!(CmdError::from(not_found).kind, CmdErrorKind::NotFound);

        let conflict =
            crate::biz::BizError::IdempotencyConflict("同じ冪等キーで異なる内容".to_string());
        assert_eq!(
            CmdError::from(conflict).kind,
            CmdErrorKind::IdempotencyConflict
        );
    }
}
