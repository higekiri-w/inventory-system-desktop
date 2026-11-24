//! EJ の記録の保存と照会（24 §14.25、D-121・BIZ-08-D9）
//!
//! `ej_records`（pos-tables の「SD の一括取込みと EJ の控えの契約」）を読み書きする。呼ぶのは BIZ-08 §37.11 だけで、
//! TX は BIZ-08 が持つ。insert-only（UPDATE・DELETE しない）。閲覧（UI-09c）は日・番号・id で読むだけ。

use super::{DbConnection, DbError};
use std::collections::HashMap;

/// 見出しのモード欄（IO-08 の `EjMode`）。wire と DB の値は snake_case で、`ej_records.mode` の CHECK と同じ
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum EjRecordModeCode {
    Normal,
    Return,
    Training,
    Settlement,
    Inspection,
    Program,
    Program1,
    Program3,
    Off,
    Unrecognized,
}

impl EjRecordModeCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Return => "return",
            Self::Training => "training",
            Self::Settlement => "settlement",
            Self::Inspection => "inspection",
            Self::Program => "program",
            Self::Program1 => "program1",
            Self::Program3 => "program3",
            Self::Off => "off",
            Self::Unrecognized => "unrecognized",
        }
    }

    /// DB の TEXT から読む（外れた値は変換の失敗。D-061 (c)、先例 sales_repo の `parse_csv_import_status`）
    pub(crate) fn from_db(index: usize, value: &str) -> rusqlite::Result<Self> {
        Ok(match value {
            "normal" => Self::Normal,
            "return" => Self::Return,
            "training" => Self::Training,
            "settlement" => Self::Settlement,
            "inspection" => Self::Inspection,
            "program" => Self::Program,
            "program1" => Self::Program1,
            "program3" => Self::Program3,
            "off" => Self::Off,
            "unrecognized" => Self::Unrecognized,
            value => return Err(unknown_text(index, "mode", value)),
        })
    }
}

/// 記録の種類（IO-08 の `EjRecordKind`）。`ej_records.kind` の CHECK と同じ値
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum EjRecordKindCode {
    Sale,
    Cancelled,
    CashMovement,
    Receipt,
    Settlement,
    Inspection,
    Settings,
    Training,
    Unclassified,
}

impl EjRecordKindCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sale => "sale",
            Self::Cancelled => "cancelled",
            Self::CashMovement => "cash_movement",
            Self::Receipt => "receipt",
            Self::Settlement => "settlement",
            Self::Inspection => "inspection",
            Self::Settings => "settings",
            Self::Training => "training",
            Self::Unclassified => "unclassified",
        }
    }

    /// DB の TEXT から読む（外れた値は変換の失敗）
    pub(crate) fn from_db(index: usize, value: &str) -> rusqlite::Result<Self> {
        Ok(match value {
            "sale" => Self::Sale,
            "cancelled" => Self::Cancelled,
            "cash_movement" => Self::CashMovement,
            "receipt" => Self::Receipt,
            "settlement" => Self::Settlement,
            "inspection" => Self::Inspection,
            "settings" => Self::Settings,
            "training" => Self::Training,
            "unclassified" => Self::Unclassified,
            value => return Err(unknown_text(index, "kind", value)),
        })
    }
}

fn unknown_text(index: usize, column: &str, value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        rusqlite::types::Type::Text,
        format!("unknown ej_records.{column}: {value}").into(),
    )
}

/// 24 §14.25 の insert の入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEjRecord {
    pub machine_prefix: String,
    pub record_no: String,
    pub printed_at: String,
    pub mode: EjRecordModeCode,
    pub kind: EjRecordKindCode,
    pub settlement_report: Option<String>,
    pub settlement_completed: Option<bool>,
    pub total_amount: Option<i64>,
    pub body_text: String,
}

/// 番号の欠けの区間（37 §37.11）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct EjNumberGap {
    /// 4 桁の欄
    pub machine_prefix: String,
    /// 6 桁（先頭 0 を保つ）
    pub from_no: String,
    /// 6 桁。from_no と同じなら 1 つの番号
    pub to_no: String,
}

/// 一覧の 1 行（37 §37.11）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct EjRecordSummary {
    pub id: i64,
    pub machine_prefix: String,
    /// 6 桁
    pub record_no: String,
    /// `YYYY-MM-DD HH:MM`（記録のまま）
    pub printed_at: String,
    pub kind: EjRecordKindCode,
    pub mode: EjRecordModeCode,
    /// 円。復元できた取引だけ
    pub total_amount: Option<i64>,
}

/// 控えの文（37 §37.11）。body_text は保存したまま（行末の空白を保つ）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct EjRecordDetail {
    pub summary: EjRecordSummary,
    pub body_text: String,
}

/// 番号で探した 1 件（37 §37.11）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct EjRecordRef {
    pub id: i64,
    /// `YYYY-MM-DD`
    pub date: String,
}

const SUMMARY_COLUMNS: &str = "id, machine_prefix, record_no, printed_at, kind, mode, total_amount";

fn row_to_summary(row: &rusqlite::Row) -> rusqlite::Result<EjRecordSummary> {
    Ok(EjRecordSummary {
        id: row.get(0)?,
        machine_prefix: row.get(1)?,
        record_no: row.get(2)?,
        printed_at: row.get(3)?,
        kind: EjRecordKindCode::from_db(4, &row.get::<_, String>(4)?)?,
        mode: EjRecordModeCode::from_db(5, &row.get::<_, String>(5)?)?,
        total_amount: row.get(6)?,
    })
}

/// 保存済みの key（machine_prefix, record_no）→ body_text。key が多いときは呼び手が分けて呼ぶ
pub fn find_ej_record_texts(
    conn: &DbConnection,
    keys: &[(String, String)],
) -> Result<HashMap<(String, String), String>, DbError> {
    if keys.is_empty() {
        return Ok(HashMap::new());
    }
    let values = vec!["(?, ?)"; keys.len()].join(", ");
    let sql = format!(
        "SELECT machine_prefix, record_no, body_text FROM ej_records
         WHERE (machine_prefix, record_no) IN (VALUES {values})"
    );
    let params: Vec<&dyn rusqlite::ToSql> = keys
        .iter()
        .flat_map(|(prefix, no)| [prefix as &dyn rusqlite::ToSql, no as &dyn rusqlite::ToSql])
        .collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(((row.get(0)?, row.get(1)?), row.get(2)?))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// UNIQUE に当たったら `DbError`（BIZ は TX の中で先に比べるので通常は当たらない）
pub fn insert_ej_record(
    conn: &DbConnection,
    record: &NewEjRecord,
    imported_at: &str,
) -> Result<i64, DbError> {
    conn.execute(
        "INSERT INTO ej_records (machine_prefix, record_no, printed_at, mode, kind,
             settlement_report, settlement_completed, total_amount, body_text, imported_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            record.machine_prefix,
            record.record_no,
            record.printed_at,
            record.mode.as_str(),
            record.kind.as_str(),
            record.settlement_report,
            record.settlement_completed,
            record.total_amount,
            record.body_text,
            imported_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// `date` の日（`date` 00:00 以上、翌日 `next_date` 00:00 未満）の記録を欄・番号の順に最大 1001 件
/// （BIZ が 1000 件を超えたかを知るため）
pub fn list_ej_records_on_date(
    conn: &DbConnection,
    date: &str,
    next_date: &str,
) -> Result<Vec<EjRecordSummary>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SUMMARY_COLUMNS} FROM ej_records
         WHERE printed_at >= ?1 || ' 00:00' AND printed_at < ?2 || ' 00:00'
         ORDER BY machine_prefix, record_no LIMIT 1001"
    ))?;
    let rows = stmt.query_map([date, next_date], row_to_summary)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// 記録のある前の日と次の日（`next_date` は `date` の翌日）
pub fn find_ej_adjacent_dates(
    conn: &DbConnection,
    date: &str,
    next_date: &str,
) -> Result<(Option<String>, Option<String>), DbError> {
    let one = |sql: &str, param: &str| -> Result<Option<String>, DbError> {
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query_map([param], |row| row.get(0))?;
        Ok(rows.next().transpose()?)
    };
    let prev = one(
        "SELECT substr(printed_at, 1, 10) FROM ej_records
         WHERE printed_at < ?1 || ' 00:00' ORDER BY printed_at DESC LIMIT 1",
        date,
    )?;
    let next = one(
        "SELECT substr(printed_at, 1, 10) FROM ej_records
         WHERE printed_at >= ?1 || ' 00:00' ORDER BY printed_at ASC LIMIT 1",
        next_date,
    )?;
    Ok((prev, next))
}

/// 記録のある最も新しい日（記録が無ければ None）
pub fn find_latest_ej_date(conn: &DbConnection) -> Result<Option<String>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT substr(printed_at, 1, 10) FROM ej_records ORDER BY printed_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query_map([], |row| row.get(0))?;
    Ok(rows.next().transpose()?)
}

/// 同じ番号（6 桁）の記録を全ての 4 桁の欄から、日付・id の昇順で
pub fn find_ej_records_by_no(
    conn: &DbConnection,
    record_no: &str,
) -> Result<Vec<EjRecordRef>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, substr(printed_at, 1, 10) AS date FROM ej_records
         WHERE record_no = ?1 ORDER BY date, id",
    )?;
    let rows = stmt.query_map([record_no], |row| {
        Ok(EjRecordRef {
            id: row.get(0)?,
            date: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// id の記録と文（無ければ None）
pub fn get_ej_record_detail(
    conn: &DbConnection,
    id: i64,
) -> Result<Option<EjRecordDetail>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SUMMARY_COLUMNS}, body_text FROM ej_records WHERE id = ?1"
    ))?;
    let mut rows = stmt.query_map([id], |row| {
        Ok(EjRecordDetail {
            summary: row_to_summary(row)?,
            body_text: row.get(7)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

/// 4 桁の欄ごとに番号の昇順で隣との差が 2 以上の所を区間（前 + 1 〜 後 − 1）にして、欄・番号の昇順で返す
pub fn list_ej_number_gaps(conn: &DbConnection) -> Result<Vec<EjNumberGap>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT machine_prefix, prev + 1, cur - 1 FROM (
             SELECT machine_prefix, CAST(record_no AS INTEGER) AS cur,
                    LAG(CAST(record_no AS INTEGER)) OVER (
                        PARTITION BY machine_prefix ORDER BY CAST(record_no AS INTEGER)
                    ) AS prev
             FROM ej_records
         )
         WHERE prev IS NOT NULL AND cur - prev >= 2
         ORDER BY machine_prefix, cur",
    )?;
    let rows = stmt.query_map([], |row| {
        let from: i64 = row.get(1)?;
        let to: i64 = row.get(2)?;
        Ok(EjNumberGap {
            machine_prefix: row.get(0)?,
            from_no: format!("{from:06}"),
            to_no: format!("{to:06}"),
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::setup_test_db;

    fn record(prefix: &str, no: &str, body: &str) -> NewEjRecord {
        NewEjRecord {
            machine_prefix: prefix.to_string(),
            record_no: no.to_string(),
            printed_at: "2030-01-02 10:15".to_string(),
            mode: EjRecordModeCode::Normal,
            kind: EjRecordKindCode::Sale,
            settlement_report: None,
            settlement_completed: None,
            total_amount: Some(500),
            body_text: body.to_string(),
        }
    }

    #[test]
    fn test_req401_ej_repo_texts_insert_and_gaps() {
        // REQ-401 / 24 §14.25: 保存済みの文を key で引き、同じ key の 2 回目は DbError、欠けは欄ごとの区間
        let (_dir, conn) = setup_test_db();
        for (prefix, no) in [
            ("0001", "000101"),
            ("0001", "000104"),
            ("0001", "000106"),
            ("0002", "000001"),
            ("0002", "000002"),
        ] {
            insert_ej_record(&conn, &record(prefix, no, &format!("{prefix}-{no}")), "t").unwrap();
        }
        assert!(insert_ej_record(&conn, &record("0001", "000101", "other"), "t").is_err());
        let texts = find_ej_record_texts(
            &conn,
            &[
                ("0001".to_string(), "000104".to_string()),
                ("0001".to_string(), "000999".to_string()),
                ("0002".to_string(), "000001".to_string()),
            ],
        )
        .unwrap();
        assert_eq!(texts.len(), 2);
        assert_eq!(
            texts[&("0001".to_string(), "000104".to_string())],
            "0001-000104"
        );
        assert!(find_ej_record_texts(&conn, &[]).unwrap().is_empty());
        let gaps = list_ej_number_gaps(&conn).unwrap();
        let gap = |from: &str, to: &str| EjNumberGap {
            machine_prefix: "0001".to_string(),
            from_no: from.to_string(),
            to_no: to.to_string(),
        };
        assert_eq!(gaps, vec![gap("000102", "000103"), gap("000105", "000105")]);
    }

    fn insert_at(conn: &DbConnection, prefix: &str, no: &str, printed_at: &str) -> i64 {
        let record = NewEjRecord {
            printed_at: printed_at.to_string(),
            ..record(prefix, no, &format!("{prefix}-{no}"))
        };
        insert_ej_record(conn, &record, "t").unwrap()
    }

    #[test]
    fn test_req908_ej_repo_r1_records_on_date_bounds_and_1001() {
        // REQ-908 / 24 §14.25（Test Plan R1・M1）: 前の日の 23:59 と翌日の 00:00 は入らず、1001 件まで返す
        let (_dir, conn) = setup_test_db();
        insert_at(&conn, "0001", "000001", "2030-01-01 23:59");
        insert_at(&conn, "0001", "000002", "2030-01-02 00:00");
        insert_at(&conn, "0001", "000003", "2030-01-02 23:59");
        insert_at(&conn, "0001", "000004", "2030-01-03 00:00");
        let day = list_ej_records_on_date(&conn, "2030-01-02", "2030-01-03").unwrap();
        let numbers: Vec<&str> = day.iter().map(|r| r.record_no.as_str()).collect();
        assert_eq!(numbers, vec!["000002", "000003"]);
        assert_eq!(day[0].kind, EjRecordKindCode::Sale);
        assert_eq!(day[0].mode, EjRecordModeCode::Normal);
        assert_eq!(day[0].total_amount, Some(500));

        for no in 100..1101 {
            insert_at(&conn, "0002", &format!("{no:06}"), "2030-02-01 09:00");
        }
        let many = list_ej_records_on_date(&conn, "2030-02-01", "2030-02-02").unwrap();
        assert_eq!(many.len(), 1001);
        assert_eq!(many[0].record_no, "000100");
    }

    #[test]
    fn test_req908_ej_repo_r2_adjacent_and_latest_dates() {
        // REQ-908 / 24 §14.25（Test Plan R2）: 前後は記録のある日（空の日を飛ばす）、端は None、空の表で最新は None
        let (_dir, conn) = setup_test_db();
        assert_eq!(find_latest_ej_date(&conn).unwrap(), None);
        assert_eq!(
            find_ej_adjacent_dates(&conn, "2030-01-05", "2030-01-06").unwrap(),
            (None, None)
        );
        insert_at(&conn, "0001", "000001", "2030-01-01 10:00");
        insert_at(&conn, "0001", "000002", "2030-01-05 10:00");
        insert_at(&conn, "0001", "000003", "2030-01-09 10:00");
        assert_eq!(
            find_ej_adjacent_dates(&conn, "2030-01-05", "2030-01-06").unwrap(),
            (
                Some("2030-01-01".to_string()),
                Some("2030-01-09".to_string())
            )
        );
        assert_eq!(
            find_ej_adjacent_dates(&conn, "2030-01-01", "2030-01-02").unwrap(),
            (None, Some("2030-01-05".to_string()))
        );
        assert_eq!(
            find_ej_adjacent_dates(&conn, "2030-01-09", "2030-01-10").unwrap(),
            (Some("2030-01-05".to_string()), None)
        );
        assert_eq!(
            find_latest_ej_date(&conn).unwrap(),
            Some("2030-01-09".to_string())
        );
    }

    #[test]
    fn test_req908_ej_repo_r3_by_no_across_prefixes() {
        // REQ-908 / 24 §14.25（Test Plan R3）: 別の 4 桁の欄の同じ番号も返し、(date, id) の昇順
        let (_dir, conn) = setup_test_db();
        let late = insert_at(&conn, "0001", "000102", "2030-01-03 10:00");
        let early = insert_at(&conn, "0002", "000102", "2030-01-02 10:00");
        insert_at(&conn, "0001", "000103", "2030-01-02 10:00");
        assert_eq!(
            find_ej_records_by_no(&conn, "000102").unwrap(),
            vec![
                EjRecordRef {
                    id: early,
                    date: "2030-01-02".to_string()
                },
                EjRecordRef {
                    id: late,
                    date: "2030-01-03".to_string()
                },
            ]
        );
        assert!(find_ej_records_by_no(&conn, "000999").unwrap().is_empty());
    }

    #[test]
    fn test_req908_ej_repo_r4_detail_keeps_text_and_missing_is_none() {
        // REQ-908 / 24 §14.25（Test Plan R4）: 文は保存したまま（行末の空白と "\n" のつなぎ）、無い id は None
        let (_dir, conn) = setup_test_db();
        let body = "000101 2030/01/02 10:15  \nﾘﾝｺﾞ        ¥100 \n合  計      ¥100   ";
        let id = insert_ej_record(&conn, &record("0001", "000101", body), "t").unwrap();
        let detail = get_ej_record_detail(&conn, id).unwrap().unwrap();
        assert_eq!(detail.body_text, body);
        assert_eq!(detail.summary.id, id);
        assert_eq!(detail.summary.record_no, "000101");
        assert_eq!(get_ej_record_detail(&conn, id + 999).unwrap(), None);
    }

    #[test]
    fn test_req908_ej_repo_r5_mode_kind_read_by_explicit_match() {
        // REQ-908 / 24 §14.25・D-061 (c)（Test Plan R5・M2）: CHECK の外の値は DbError、全 variant は as_str と往復する
        assert!(EjRecordModeCode::from_db(5, "bogus").is_err());
        assert!(EjRecordKindCode::from_db(4, "bogus").is_err());
        assert!(matches!(
            DbError::from(EjRecordModeCode::from_db(5, "").unwrap_err()),
            DbError::QueryFailed(_)
        ));
        assert!(matches!(
            DbError::from(EjRecordKindCode::from_db(4, "Sale").unwrap_err()),
            DbError::QueryFailed(_)
        ));
    }
}
