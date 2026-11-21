//! BIZ-09: システム設定・操作ログロジック
//!
//! docs/function-design/38-biz-system-service.md に基づく実装。

use crate::biz::BizError;
use crate::db::system_repo::{self, AppSetting, OperationLog};
use crate::db::{DbConnection, PaginatedResult};

const INVALID_DATE_MESSAGE: &str = "開始日・終了日はYYYY-MM-DD形式で入力してください";
const REVERSED_DATE_RANGE_MESSAGE: &str =
    "開始日は終了日と同じ日か、それより前の日付にしてください";
const INVALID_BACKUP_RETENTION_DAYS_MESSAGE: &str = "保存日数は 1 以上の整数で入れてください";

/// 全設定を取得する。
pub fn get_all_settings(conn: &DbConnection) -> Result<Vec<AppSetting>, BizError> {
    system_repo::get_all_settings(conn).map_err(BizError::from)
}

/// 設定値を挿入または更新する。`backup_retention_days` だけ値を検査する（§38.3）。
pub fn upsert_setting(conn: &DbConnection, key: &str, value: &str) -> Result<(), BizError> {
    // 掃除（mnt::backup::run_cleanup）は u32 で読み、0 なら今日より前を全部消し、数字でなければ黙って止まる
    if key == "backup_retention_days"
        && !(value.bytes().all(|byte| byte.is_ascii_digit())
            && matches!(value.parse::<i32>(), Ok(days) if days >= 1))
    {
        return Err(BizError::ValidationFailed(
            INVALID_BACKUP_RETENTION_DAYS_MESSAGE.to_string(),
        ));
    }
    system_repo::upsert_setting(conn, key, value).map_err(BizError::from)
}

/// 操作ログを検索する。
pub fn list_operation_logs(
    conn: &DbConnection,
    page: u32,
    per_page: u32,
    operation_type: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<PaginatedResult<OperationLog>, BizError> {
    validate_log_date_range(start_date, end_date)?;
    system_repo::list_operation_logs(conn, page, per_page, operation_type, start_date, end_date)
        .map_err(BizError::from)
}

/// 操作ログに存在する operation_type 一覧を返す。
pub fn list_distinct_operation_types(conn: &DbConnection) -> Result<Vec<String>, BizError> {
    system_repo::find_distinct_operation_types(conn).map_err(BizError::from)
}

fn validate_log_date_range(
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<(), BizError> {
    let parse = |value: &str| {
        let bytes = value.as_bytes();
        let has_strict_ymd_shape = bytes.len() == 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && [0, 1, 2, 3, 5, 6, 8, 9]
                .iter()
                .all(|index| bytes[*index].is_ascii_digit());
        if !has_strict_ymd_shape {
            return Err(());
        }
        let year = value[0..4].parse().map_err(|_| ())?;
        let month = value[5..7].parse().map_err(|_| ())?;
        let day = value[8..10].parse().map_err(|_| ())?;
        chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or(())
    };
    let start = start_date
        .map(parse)
        .transpose()
        .map_err(|_| BizError::ValidationFailed(INVALID_DATE_MESSAGE.to_string()))?;
    let end = end_date
        .map(parse)
        .transpose()
        .map_err(|_| BizError::ValidationFailed(INVALID_DATE_MESSAGE.to_string()))?;
    if matches!((start, end), (Some(start), Some(end)) if start > end) {
        return Err(BizError::ValidationFailed(
            REVERSED_DATE_RANGE_MESSAGE.to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_get_all_settings_req905_returns_seeded_settings() {
        // REQ-905 / BIZ-09: BIZ 経由で初期設定を取得する。
        let (_dir, conn) = db::test_support::setup_test_db();

        let settings = get_all_settings(&conn).unwrap();

        assert!(settings
            .iter()
            .any(|setting| setting.key == "backup_enabled"));
    }

    #[test]
    fn test_upsert_setting_req905_persists_value() {
        // REQ-905 / BIZ-09: BIZ 経由で設定を upsert する。
        let (_dir, conn) = db::test_support::setup_test_db();

        upsert_setting(&conn, "stock_low_threshold", "5").unwrap();

        assert_eq!(
            system_repo::get_setting(&conn, "stock_low_threshold").unwrap(),
            Some("5".to_string())
        );
    }

    #[test]
    fn test_upsert_setting_req901_backup_retention_days_accepts_only_positive_integers() {
        // REQ-901 / BIZ-09 / MNT-01-D3: 保存日数は 1 以上の整数だけ保存し、それ以外は理由を返して保存しない
        let (_dir, conn) = db::test_support::setup_test_db();
        for invalid in [
            "0",
            "-1",
            "",
            "abc",
            "1.5",
            " 3",
            "3 ",
            "+3",
            "1e2",
            "３",
            "2147483648",
        ] {
            let result = upsert_setting(&conn, "backup_retention_days", invalid);
            assert!(
                matches!(
                    result,
                    Err(BizError::ValidationFailed(ref message))
                        if message == "保存日数は 1 以上の整数で入れてください"
                ),
                "{invalid:?} must be rejected"
            );
            assert_eq!(
                system_repo::get_setting(&conn, "backup_retention_days").unwrap(),
                Some("3".to_string()),
                "{invalid:?} must not be saved"
            );
        }
        for valid in ["1", "3", "30", "2147483647"] {
            upsert_setting(&conn, "backup_retention_days", valid).unwrap();
            assert_eq!(
                system_repo::get_setting(&conn, "backup_retention_days").unwrap(),
                Some(valid.to_string())
            );
        }
    }

    #[test]
    fn test_upsert_setting_req905_other_keys_are_not_validated() {
        // REQ-905 / BIZ-09: 保存日数の検査はほかの key に掛けない
        let (_dir, conn) = db::test_support::setup_test_db();

        upsert_setting(&conn, "stock_low_threshold", "0").unwrap();
        upsert_setting(&conn, "backup_time", "").unwrap();

        assert_eq!(
            system_repo::get_setting(&conn, "stock_low_threshold").unwrap(),
            Some("0".to_string())
        );
        assert_eq!(
            system_repo::get_setting(&conn, "backup_time").unwrap(),
            Some(String::new())
        );
    }

    #[test]
    fn test_list_logs_req902_date_validation_contract() {
        // REQ-902 / UI-11c-D2 / D-036 / D-037 / SPEC-CMD11-D2
        let (_dir, conn) = db::test_support::setup_test_db();
        assert!(
            list_operation_logs(&conn, 1, 20, None, Some("2026-07-10"), Some("2026-07-10")).is_ok()
        );
        for (field, invalid_date) in [
            ("start_date", "2026-7-01"),
            ("start_date", "2026-07-01x"),
            ("start_date", "2026/07/01"),
            ("start_date", "2026-07-01 "),
            ("start_date", "2026-02-30"),
            ("start_date", "２０２６-０７-０１"),
            ("end_date", "2026-7-01"),
            ("end_date", "2026-07-01x"),
            ("end_date", "2026/07/01"),
            ("end_date", "2026-07-01 "),
            ("end_date", "2026-02-30"),
            ("end_date", "２０２６-０７-０１"),
        ] {
            let result = match field {
                "start_date" => list_operation_logs(&conn, 1, 20, None, Some(invalid_date), None),
                "end_date" => list_operation_logs(&conn, 1, 20, None, None, Some(invalid_date)),
                _ => unreachable!(),
            };
            assert!(
                matches!(
                    result,
                    Err(BizError::ValidationFailed(ref message))
                        if message == "開始日・終了日はYYYY-MM-DD形式で入力してください"
                ),
                "{field}={invalid_date} must preserve the validation contract"
            );
        }

        let reversed =
            list_operation_logs(&conn, 1, 20, None, Some("2026-07-11"), Some("2026-07-10"));
        assert!(matches!(
            reversed,
            Err(BizError::ValidationFailed(ref message))
                if message == "開始日は終了日と同じ日か、それより前の日付にしてください"
        ));
    }

    #[test]
    fn test_list_operation_logs_req902_maps_repo_error_to_database_error() {
        // REQ-902 / BIZ-09 / SPEC-CMD11-IMPL-D4
        let (_dir, conn) = db::test_support::setup_test_db();

        let result = list_operation_logs(&conn, 0, 20, None, None, None);

        assert!(matches!(result, Err(BizError::DatabaseError(_))));
    }

    #[test]
    fn test_list_distinct_operation_types_req902_delegates_to_repo() {
        // REQ-902 / UI-11c-D4 / BIZ-09
        let (_dir, conn) = db::test_support::setup_test_db();
        system_repo::insert_operation_log(
            &conn,
            &db::NewOperationLog {
                operation_type: "z_unknown".to_string(),
                summary: "test".to_string(),
                detail_json: None,
            },
        )
        .unwrap();
        system_repo::insert_operation_log(
            &conn,
            &db::NewOperationLog {
                operation_type: "backup_create".to_string(),
                summary: "test".to_string(),
                detail_json: None,
            },
        )
        .unwrap();

        assert_eq!(
            list_distinct_operation_types(&conn).unwrap(),
            vec!["backup_create".to_string(), "z_unknown".to_string()]
        );
    }
}
