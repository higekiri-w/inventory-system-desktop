//! 在庫ドメイン共通型・関数
//!
//! ListQuery, validate_and_offset と、記録の取消の列（D-163）の読み書きを提供する。
//! 入庫・返品・手動販売・廃棄・振替の各リポジトリと BIZ の取消から参照される。

use super::{DbConnection, DbError};

/// 一覧取得の共通クエリパラメータ
///
/// 21-io-inventory-repo.md §10.1
#[derive(Debug)]
pub struct ListQuery {
    pub page: u32,
    pub per_page: u32,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

/// ListQuery のバリデーション + offset 計算
///
/// page < 1 or per_page < 1 → DbError::QueryFailed
/// (page-1) * per_page のオーバーフロー → DbError::QueryFailed
/// 戻り値: (per_page as i64, offset as i64)
pub fn validate_and_offset(query: &ListQuery) -> Result<(i64, i64), DbError> {
    if query.page < 1 {
        return Err(DbError::QueryFailed("page は 1 以上が必要です".to_string()));
    }
    if query.per_page < 1 {
        return Err(DbError::QueryFailed(
            "per_page は 1 以上が必要です".to_string(),
        ));
    }
    let page_minus_one = query
        .page
        .checked_sub(1)
        .ok_or_else(|| DbError::QueryFailed("page のオーバーフロー".to_string()))?;
    let offset = page_minus_one
        .checked_mul(query.per_page)
        .ok_or_else(|| DbError::QueryFailed("offset 計算のオーバーフロー".to_string()))?;
    let limit = i64::from(query.per_page);
    let offset = i64::from(offset);
    Ok((limit, offset))
}

/// 記録の取消の列（`canceled_at`・`cancel_reason`）を読む。記録が無ければ `DbError::NotFound`
///
/// `table` は取消の列を持つ 5 つの header の表名（`receiving_records`・`return_records`・`manual_sales`・
/// `disposal_records`・`transfer_records`）で、呼ぶ側の literal だけを渡す（入力から作らない）。
///
/// 21-io-inventory-repo.md §10.5b
pub fn get_record_cancel(
    conn: &DbConnection,
    table: &'static str,
    record_id: i64,
) -> Result<(Option<String>, Option<String>), DbError> {
    let result = conn.query_row(
        &format!("SELECT canceled_at, cancel_reason FROM {table} WHERE id = ?1"),
        rusqlite::params![record_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );
    match result {
        Ok(row) => Ok(row),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(DbError::NotFound),
        Err(e) => Err(DbError::from(e)),
    }
}

/// 有効な記録（`canceled_at IS NULL`）に取消の日時と理由を書き、変えた行数（0 か 1）を返す
///
/// 21-io-inventory-repo.md §10.5b
pub fn mark_record_canceled(
    conn: &DbConnection,
    table: &'static str,
    record_id: i64,
    canceled_at: &str,
    cancel_reason: &str,
) -> Result<usize, DbError> {
    Ok(conn.execute(
        &format!(
            "UPDATE {table} SET canceled_at = ?2, cancel_reason = ?3
             WHERE id = ?1 AND canceled_at IS NULL"
        ),
        rusqlite::params![record_id, canceled_at, cancel_reason],
    )?)
}

/// 取消の列から決まる記録の状態（65 §65.4.1。NULL なら `active`、値があれば `canceled`）
///
/// 21-io-inventory-repo.md §10.5b
pub fn record_status(canceled_at: Option<&str>) -> String {
    if canceled_at.is_some() {
        "canceled"
    } else {
        "active"
    }
    .to_string()
}
