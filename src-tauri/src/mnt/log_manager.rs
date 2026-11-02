//! MNT-02: 操作ログ管理
//!
//! 操作ログの自動削除を提供する。アプリ起動時に1日1回実行。
//!
//! docs/function-design/72-mnt-log-manager.md に基づく実装。

use crate::db::{self, DbConnection, DbError, NewOperationLog};

/// 既定のログ保持日数（app_settings に `log_retention_days` の行が無い場合。MNT-02-D1）
const DEFAULT_RETENTION_DAYS: u32 = 3000;

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// 操作ログの自動削除を実行する（1日1回制限）
///
/// 72-mnt-log-manager.md §72.4
///
/// アプリ起動時にsetup hookから呼ばれる。
/// `log_last_cleanup_date` が今日と同じ場合はスキップ。
pub fn cleanup_old_logs(conn: &DbConnection) -> Result<(), DbError> {
    let today = chrono::Local::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    // 1. 最終クリーンアップ日を取得
    if let Some(date_str) = db::system_repo::get_setting(conn, "log_last_cleanup_date")? {
        // 2. 今日と比較（パース失敗は無視して続行）
        if let Ok(last_date) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            if last_date == today {
                return Ok(()); // 同日 → スキップ
            }
        }
    }

    // 3. 保持日数を取得（MNT-02-D1: 行が無ければ既定。読めない値・0 は保持日数を
    //    確定できないので、削除も日付の更新もせずに返す）
    let retention_days = match db::system_repo::get_setting(conn, "log_retention_days")? {
        None => DEFAULT_RETENTION_DAYS,
        Some(value) => match value.parse::<u32>() {
            Ok(days) if days > 0 => days,
            _ => {
                tracing::warn!(
                    value = %value,
                    "log_retention_days を保持日数として読めないため、操作ログの自動削除を飛ばした"
                );
                return Ok(());
            }
        },
    };

    // 4. 古いログを削除（日付の範囲の外になる保持日数は 0 件）
    let deleted = db::system_repo::delete_old_logs(conn, retention_days)?;

    // 5. 削除件数 > 0 ならログに記録
    if deleted > 0 {
        let cutoff_date = (today - chrono::Duration::days(i64::from(retention_days)))
            .format("%Y-%m-%d")
            .to_string();
        db::system_repo::insert_operation_log(
            conn,
            &NewOperationLog {
                operation_type: "log_cleanup".to_string(),
                summary: format!("操作ログを{}件削除しました（{}以前）", deleted, cutoff_date),
                detail_json: None,
            },
        )?;
    }

    // 6. 最終クリーンアップ日を更新
    db::system_repo::upsert_setting(conn, "log_last_cleanup_date", &today_str)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    /// 古いログを手動INSERTするヘルパー
    fn insert_log_with_date(conn: &DbConnection, created_at: &str) {
        conn.execute(
            "INSERT INTO operation_logs (operation_type, summary, created_at)
             VALUES ('test_op', 'テスト', ?1)",
            rusqlite::params![created_at],
        )
        .unwrap();
    }

    /// 今日から `days` 日前の 10:00 の `created_at`
    fn days_ago(today: chrono::NaiveDate, days: u64) -> String {
        (today - chrono::Days::new(days))
            .format("%Y-%m-%dT10:00:00")
            .to_string()
    }

    fn count_logs(conn: &DbConnection, operation_type: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM operation_logs WHERE operation_type = ?1",
            rusqlite::params![operation_type],
            |row| row.get(0),
        )
        .unwrap()
    }

    /// `f` を今日の日付で呼び、呼ぶ前後で日付が変わっていなければ結果を返す。
    /// 0 時を跨いだら同じ日の中でやり直す（境の日ちょうどの記録と今日の日付を見る test の
    /// 偽の red を避ける。Plan Review round 3 の P3）。assert は戻り値に対して呼び出し側で行う
    fn on_same_day<T>(mut f: impl FnMut(chrono::NaiveDate) -> T) -> T {
        loop {
            let today = chrono::Local::now().date_naive();
            let out = f(today);
            if chrono::Local::now().date_naive() == today {
                return out;
            }
        }
    }

    #[test]
    fn test_cleanup_old_logs_req902_first_run() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02: log_last_cleanup_date 未存在時にクリーンアップ実行
        let (_dir, conn) = setup_test_db();

        // log_last_cleanup_date は初期データに含まれない
        let before = db::system_repo::get_setting(&conn, "log_last_cleanup_date").unwrap();
        assert!(before.is_none(), "初期データにlog_last_cleanup_dateは無い");

        // 古いログを挿入（既定の保持日数 3000 日より前。MNT-02-D1）
        insert_log_with_date(&conn, &days_ago(chrono::Local::now().date_naive(), 3001));

        // cleanup実行
        cleanup_old_logs(&conn).unwrap();

        // 古いログが削除された
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'test_op'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "古いログは削除されるべき");
    }

    #[test]
    fn test_cleanup_old_logs_req902_same_day_skip() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02: 同日2回目の呼び出しでスキップ
        let (_dir, conn) = setup_test_db();

        // 1回目: 実行される
        cleanup_old_logs(&conn).unwrap();

        // 古いログを2回目の前に挿入（既定の保持日数 3000 日より前。MNT-02-D1）
        insert_log_with_date(&conn, &days_ago(chrono::Local::now().date_naive(), 3001));

        // 2回目: スキップされる
        cleanup_old_logs(&conn).unwrap();

        // 古いログが残っている（2回目はスキップされたため）
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'test_op'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "2回目はスキップされるため古いログは残る");
    }

    #[test]
    fn test_cleanup_old_logs_req902_deletes_expired() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02-D1: 初期データの保持日数（3000 日）で、3001日前のログが削除される
        let (_dir, conn) = setup_test_db();

        insert_log_with_date(&conn, &days_ago(chrono::Local::now().date_naive(), 3001));

        cleanup_old_logs(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'test_op'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "3001日前のログは削除されるべき");
    }

    #[test]
    fn test_cleanup_old_logs_req902_keeps_recent() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02-D1: 初期データの保持日数（3000 日）で、3000日前のログが保持される（境の日）
        let count = on_same_day(|today| {
            let (_dir, conn) = setup_test_db();
            insert_log_with_date(&conn, &days_ago(today, 3000));
            cleanup_old_logs(&conn).unwrap();
            count_logs(&conn, "test_op")
        });
        assert_eq!(count, 1, "3000日前のログは保持されるべき");
    }

    #[test]
    fn test_cleanup_old_logs_req902_records_cleanup() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02: 削除後にoperation_type='log_cleanup'のログが記録される
        let (_dir, conn) = setup_test_db();

        // 既定の保持日数 3000 日より前（MNT-02-D1）
        insert_log_with_date(&conn, &days_ago(chrono::Local::now().date_naive(), 3001));

        cleanup_old_logs(&conn).unwrap();

        let summary: String = conn
            .query_row(
                "SELECT summary FROM operation_logs WHERE operation_type = 'log_cleanup'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            summary.contains("1件削除しました"),
            "削除件数がsummaryに含まれるべき: {}",
            summary
        );
    }

    #[test]
    fn test_cleanup_old_logs_req902_updates_date() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02: 実行後にlog_last_cleanup_dateが今日に更新される
        let (_dir, conn) = setup_test_db();

        cleanup_old_logs(&conn).unwrap();

        let date = db::system_repo::get_setting(&conn, "log_last_cleanup_date")
            .unwrap()
            .expect("log_last_cleanup_date が設定されているべき");
        let today_str = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(date, today_str, "今日の日付が記録されるべき");
    }

    #[test]
    fn test_log_retention_days_req902_initial_value_is_3000() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02-D1: 新しい DB の log_retention_days は "3000"（schema_v1 の初期データ）
        let (_dir, conn) = setup_test_db();

        let value = db::system_repo::get_setting(&conn, "log_retention_days").unwrap();
        assert_eq!(value.as_deref(), Some("3000"));
    }

    #[test]
    fn test_cleanup_old_logs_req902_missing_setting_uses_default_3000() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02-D1: 設定の行が無いときは既定 3000 日で消す（3001日前は消え、3000日前は残る）
        let (expired, kept) = on_same_day(|today| {
            let (_dir, conn) = setup_test_db();
            conn.execute(
                "DELETE FROM app_settings WHERE key = 'log_retention_days'",
                [],
            )
            .unwrap();
            insert_log_with_date(&conn, &days_ago(today, 3001));
            insert_log_with_date(&conn, &days_ago(today, 3000));
            cleanup_old_logs(&conn).unwrap();
            let remaining: Vec<String> = conn
                .prepare("SELECT created_at FROM operation_logs WHERE operation_type = 'test_op'")
                .unwrap()
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            (
                remaining.contains(&days_ago(today, 3001)),
                remaining.contains(&days_ago(today, 3000)),
            )
        });
        assert!(!expired, "3001日前のログは既定 3000 日で削除されるべき");
        assert!(kept, "3000日前のログは既定 3000 日で保持されるべき");
    }

    #[test]
    fn test_cleanup_old_logs_req902_unreadable_or_zero_retention_skips() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02-D1: 保持日数が整数として読めない・0 のときは何も消さず、
        // log_last_cleanup_date も更新せず、log_cleanup も記録しない
        for value in ["abc", "-1", "0", ""] {
            let (_dir, conn) = setup_test_db();
            db::system_repo::upsert_setting(&conn, "log_retention_days", value).unwrap();
            let today = chrono::Local::now().date_naive();
            insert_log_with_date(&conn, &days_ago(today, 3001));
            insert_log_with_date(&conn, &days_ago(today, 1));

            cleanup_old_logs(&conn).unwrap();

            assert_eq!(
                count_logs(&conn, "test_op"),
                2,
                "値 {value:?} では何も消さないべき"
            );
            assert_eq!(
                db::system_repo::get_setting(&conn, "log_last_cleanup_date").unwrap(),
                None,
                "値 {value:?} では log_last_cleanup_date を更新しないべき"
            );
            assert_eq!(
                count_logs(&conn, "log_cleanup"),
                0,
                "値 {value:?} では log_cleanup を記録しないべき"
            );
        }
    }

    #[test]
    fn test_cleanup_old_logs_req902_out_of_range_retention_deletes_nothing_and_updates_date() {
        // REQ-902: ログ管理（操作ログ記録/一覧/自動削除）
        // Task: MNT-02
        // MNT-02-D1: 日付の範囲の外になる保持日数（u32::MAX）では panic せず 0 件。
        // §72.4 の手順 4→6 で log_last_cleanup_date は今日になる
        let (result, count, cleanup_count, last_date, today) = on_same_day(|today| {
            let (_dir, conn) = setup_test_db();
            db::system_repo::upsert_setting(&conn, "log_retention_days", "4294967295").unwrap();
            insert_log_with_date(&conn, &days_ago(today, 3001));
            let result = cleanup_old_logs(&conn);
            (
                result,
                count_logs(&conn, "test_op"),
                count_logs(&conn, "log_cleanup"),
                db::system_repo::get_setting(&conn, "log_last_cleanup_date").unwrap(),
                today,
            )
        });
        assert!(
            result.is_ok(),
            "範囲の外の値で Ok(()) を返すべき: {result:?}"
        );
        assert_eq!(count, 1, "3001日前のログは残るべき");
        assert_eq!(cleanup_count, 0, "0 件なので log_cleanup は記録しないべき");
        assert_eq!(
            last_date,
            Some(today.format("%Y-%m-%d").to_string()),
            "log_last_cleanup_date は今日になるべき"
        );
    }
}
