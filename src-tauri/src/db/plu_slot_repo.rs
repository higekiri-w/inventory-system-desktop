//! IO-01: PLU slot repository（REQ-907）

use super::{DbConnection, DbError};
use rusqlite::{params, OptionalExtension};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluSlotStatus {
    Free,
    External,
    Reserved,
    Active,
}

impl PluSlotStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::External => "external",
            Self::Reserved => "reserved",
            Self::Active => "active",
        }
    }

    fn parse(value: &str) -> rusqlite::Result<Self> {
        match value {
            "free" => Ok(Self::Free),
            "external" => Ok(Self::External),
            "reserved" => Ok(Self::Reserved),
            "active" => Ok(Self::Active),
            other => Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                format!("unknown plu slot status: {other}").into(),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluSlot {
    pub memory_no: i64,
    pub scanning_code: Option<String>,
    pub status: PluSlotStatus,
    pub reserved_at: Option<String>,
    pub activated_at: Option<String>,
    pub updated_at: String,
}

pub struct PluSlotUpdate<'a> {
    pub memory_no: i64,
    pub scanning_code: Option<&'a str>,
    pub status: PluSlotStatus,
    pub reserved_at: Option<&'a str>,
    pub activated_at: Option<&'a str>,
    pub updated_at: &'a str,
}

fn row_to_slot(row: &rusqlite::Row<'_>) -> rusqlite::Result<PluSlot> {
    let status: String = row.get(2)?;
    Ok(PluSlot {
        memory_no: row.get(0)?,
        scanning_code: row.get(1)?,
        status: PluSlotStatus::parse(&status)?,
        reserved_at: row.get(3)?,
        activated_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

const SLOT_COLUMNS: &str =
    "memory_no, scanning_code, status, reserved_at, activated_at, updated_at";

pub fn list_slots(conn: &DbConnection) -> Result<Vec<PluSlot>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SLOT_COLUMNS} FROM plu_slots ORDER BY memory_no"
    ))?;
    let slots = stmt
        .query_map([], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(slots)
}

pub fn find_slots_by_scanning_code(
    conn: &DbConnection,
    scanning_code: &str,
) -> Result<Vec<PluSlot>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SLOT_COLUMNS} FROM plu_slots WHERE scanning_code = ?1 ORDER BY memory_no"
    ))?;
    let slots = stmt
        .query_map(params![scanning_code], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(slots)
}

pub fn find_slots_by_status(
    conn: &DbConnection,
    status: PluSlotStatus,
) -> Result<Vec<PluSlot>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SLOT_COLUMNS} FROM plu_slots WHERE status = ?1 ORDER BY memory_no"
    ))?;
    let slots = stmt
        .query_map(params![status.as_str()], row_to_slot)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(slots)
}

pub fn find_slot_by_memory_no(
    conn: &DbConnection,
    memory_no: i64,
) -> Result<Option<PluSlot>, DbError> {
    Ok(conn
        .query_row(
            &format!("SELECT {SLOT_COLUMNS} FROM plu_slots WHERE memory_no = ?1"),
            params![memory_no],
            row_to_slot,
        )
        .optional()?)
}

pub fn find_min_free_slot(conn: &DbConnection) -> Result<Option<PluSlot>, DbError> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {SLOT_COLUMNS} FROM plu_slots WHERE status = 'free' ORDER BY memory_no LIMIT 1"
            ),
            [],
            row_to_slot,
        )
        .optional()?)
}

pub fn update_slot(conn: &DbConnection, update: PluSlotUpdate<'_>) -> Result<(), DbError> {
    let changed = conn.execute(
        "UPDATE plu_slots
         SET scanning_code=?2, status=?3, reserved_at=?4, activated_at=?5, updated_at=?6
         WHERE memory_no=?1",
        params![
            update.memory_no,
            update.scanning_code,
            update.status.as_str(),
            update.reserved_at,
            update.activated_at,
            update.updated_at
        ],
    )?;
    if changed != 1 {
        return Err(DbError::QueryFailed(format!(
            "PLU slot {} が見つかりません",
            update.memory_no
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::setup_test_db;

    #[test]
    fn test_plu_slot_repo_req907_order_lookup_min_free_update_and_rollback() {
        // REQ-907: A-S4
        let (_dir, mut conn) = setup_test_db();
        for memory_no in 217..=5_000 {
            conn.execute(
                "UPDATE plu_slots SET status='external', scanning_code=?2 WHERE memory_no=?1",
                params![memory_no, format!("SYNTH-{memory_no}")],
            )
            .unwrap();
        }
        for memory_no in [300, 217, 5_000, 401] {
            update_slot(
                &conn,
                PluSlotUpdate {
                    memory_no,
                    scanning_code: None,
                    status: PluSlotStatus::Free,
                    reserved_at: None,
                    activated_at: None,
                    updated_at: "2026-08-18T00:00:00",
                },
            )
            .unwrap();
        }
        assert_eq!(find_min_free_slot(&conn).unwrap().unwrap().memory_no, 217);
        let listed = list_slots(&conn).unwrap();
        assert_eq!(listed.first().unwrap().memory_no, 217);
        assert_eq!(listed.last().unwrap().memory_no, 5_000);

        update_slot(
            &conn,
            PluSlotUpdate {
                memory_no: 300,
                scanning_code: Some("4901234567894"),
                status: PluSlotStatus::Reserved,
                reserved_at: Some("2026-08-18T00:00:00"),
                activated_at: None,
                updated_at: "2026-08-18T00:00:00",
            },
        )
        .unwrap();
        assert_eq!(
            find_slots_by_scanning_code(&conn, "4901234567894").unwrap()[0].memory_no,
            300
        );

        let tx = conn.transaction().unwrap();
        update_slot(
            &tx,
            PluSlotUpdate {
                memory_no: 401,
                scanning_code: Some("ROLLBACK"),
                status: PluSlotStatus::Reserved,
                reserved_at: Some("2026-08-18T00:00:00"),
                activated_at: None,
                updated_at: "2026-08-18T00:00:00",
            },
        )
        .unwrap();
        tx.rollback().unwrap();
        assert_eq!(
            find_slot_by_memory_no(&conn, 401).unwrap().unwrap().status,
            PluSlotStatus::Free
        );
    }
}
