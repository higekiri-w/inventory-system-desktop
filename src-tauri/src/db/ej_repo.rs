//! EJ の記録の保存と照会（24 §14.25、D-121・BIZ-08-D9）
//!
//! `ej_records`（pos-tables の「SD の一括取込みと EJ の控えの契約」）を読み書きする。呼ぶのは BIZ-08 §37.11 だけで、
//! TX は BIZ-08 が持つ。insert-only（UPDATE・DELETE しない）。閲覧の関数は (A-2) が足す。

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
}
