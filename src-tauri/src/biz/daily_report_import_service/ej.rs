//! BIZ-08-D9: EJ の記録の取込み（37 §37.11 手順 1〜5）。閲覧（UI-09c）は (A-2) が足す

use crate::biz::BizError;
use crate::db::ej_repo::{self, EjNumberGap, EjRecordKindCode, EjRecordModeCode, NewEjRecord};
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::{DbConnection, DbError};
use crate::io::ej_parser::{
    self, EjDiagnosticCode, EjMode, EjParseError, EjRecord, EjRecordKind, EjRestoration,
    EjSettlementReport,
};
use crate::io::pos_source_copy;
use crate::io::register_sd;
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::OnceLock;

/// 保存済みの文を引く key の数の 1 回分（SQLite の引数の上限より十分小さい。24 §14.25）
const LOOKUP_CHUNK: usize = 400;
/// 結果で名指しする conflict の数（37 §37.11 手順 5）
const CONFLICT_LIMIT: usize = 100;
/// 安全な整数の範囲（2^53 − 1。37 §37.11 手順 2 の total_amount）
const MAX_SAFE_AMOUNT: i64 = 9_007_199_254_740_991;

#[derive(Debug, Clone)]
pub struct EjSourceFile {
    /// file 名（path の最後の要素）。表示と、手順 2 の伸びうるかの判定に使う
    pub filename: String,
    /// SD の scan から来た file だけ Some（写しを書く）
    pub sd_relative_path: Option<String>,
    pub bytes: Vec<u8>,
}

/// 手順 1〜3 の出力
#[derive(Debug)]
pub struct EjImportPlan {
    pub new_records: Vec<NewEjRecord>,
    /// new_records の記録を持つ file の添字（写しを書く file）
    pub files_with_new: Vec<usize>,
    pub already_count: i64,
    pub conflict_count: i64,
    pub held_count: i64,
    /// 手順 1 で外した file の名前（入力の順）
    pub unreadable_files: Vec<String>,
    /// (a) の鍵（4 桁の欄・番号の昇順。全件）
    conflicts: Vec<EjConflict>,
}

/// 手順 5 の出力
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct EjImportResult {
    pub inserted_count: i64,
    pub already_count: i64,
    pub conflict_count: i64,
    /// 4 桁の欄・番号の昇順、先頭 100 件
    pub conflicts: Vec<EjConflict>,
    pub held_count: i64,
    pub unreadable_files: Vec<String>,
    /// 保存の後の欠け
    pub gaps: Vec<EjNumberGap>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct EjConflict {
    pub machine_prefix: String,
    pub record_no: String,
    /// その鍵の記録が保存済みか。false は同じ呼出しの中の食い違いで、どの文も保存していない
    pub saved: bool,
}

/// 手順 1・2 を通った 1 file の記録（held は保留した記録）
struct ReadFile {
    records: Vec<(NewEjRecord, bool)>,
}

type Key = (String, String);

/// 37 §37.11 `plan_ej_records`: 手順 1〜3 を DB へ書かずに行う
pub fn plan_ej_records(
    conn: &DbConnection,
    files: &[EjSourceFile],
) -> Result<EjImportPlan, BizError> {
    let (read, unreadable_files) = read_files(files);
    plan_from(conn, &read, unreadable_files)
}

/// 37 §37.11 `import_ej_records`: 手順 1〜5
pub fn import_ej_records(
    conn: &mut DbConnection,
    files: Vec<EjSourceFile>,
    app_data_dir: &Path,
) -> Result<EjImportResult, BizError> {
    let (read, unreadable_files) = read_files(&files);
    let plan = plan_from(conn, &read, unreadable_files.clone())?;

    // 手順 4: 新しい記録を持つ SD の file を TX の前に写す（BIZ-08-D5。手で選んだ file は写さない）
    for file in plan
        .files_with_new
        .iter()
        .map(|index| &files[*index])
        .filter(|file| file.sd_relative_path.is_some())
    {
        let relative = file.sd_relative_path.as_deref().unwrap_or_default();
        pos_source_copy::save_pos_source_copy(app_data_dir, relative, &file.bytes).map_err(
            |error| {
                tracing::warn!(kind = ?error.kind(), error = %error, "EJ の写しを保存できませんでした");
                BizError::SourceCopyFailed(error.kind())
            },
        )?;
    }

    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;
    #[cfg(test)]
    hook::run(&tx);
    // 手順 3 を TX の中の保存済みの文でやり直す（plan の後に別の取込みが入った場合）
    let plan = plan_from(&tx, &read, unreadable_files)?;
    let imported_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    for record in &plan.new_records {
        ej_repo::insert_ej_record(&tx, record, &imported_at)?;
    }
    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    // 手順 5
    let inserted_count = plan.new_records.len() as i64;
    let log = NewOperationLog {
        operation_type: "ej_import".to_string(),
        summary: format!("EJ の控えの取込み: {inserted_count} 件"),
        detail_json: Some(
            serde_json::json!({
                "inserted_count": inserted_count,
                "already_count": plan.already_count,
                "conflict_count": plan.conflict_count,
                "held_count": plan.held_count,
                "unreadable_file_count": plan.unreadable_files.len(),
            })
            .to_string(),
        ),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }
    let gaps = ej_repo::list_ej_number_gaps(conn).unwrap_or_else(|error| {
        tracing::warn!(error = %error, "EJ の欠けを求められませんでした（取込みは済んでいます）");
        Vec::new()
    });
    Ok(EjImportResult {
        inserted_count,
        already_count: plan.already_count,
        conflict_count: plan.conflict_count,
        conflicts: plan.conflicts.into_iter().take(CONFLICT_LIMIT).collect(),
        held_count: plan.held_count,
        unreadable_files: plan.unreadable_files,
        gaps,
    })
}

/// 手順 1・2（DB を見ない）。読めた file の記録の列と、読めない file の名前（入力の順）
fn read_files(files: &[EjSourceFile]) -> (Vec<Option<ReadFile>>, Vec<String>) {
    let mut unreadable = Vec::new();
    let read = files
        .iter()
        .map(|file| {
            let read = read_file(file);
            if read.is_none() {
                unreadable.push(file.filename.clone());
            }
            read
        })
        .collect();
    (read, unreadable)
}

fn read_file(file: &EjSourceFile) -> Option<ReadFile> {
    // 手順 1 (i): 形（最終改行が無いことだけでは外さない）
    let shape = register_sd::measure_register_sd_shape(&file.bytes);
    if !(shape.cp932_strict && !shape.has_bom && shape.crlf_only) {
        tracing::warn!("EJ のファイルの形が外れているので読みませんでした");
        return None;
    }
    // 手順 1 (ii)
    let parsed = match ej_parser::parse_ej(&file.bytes) {
        Ok(parsed) => parsed,
        Err(error) => {
            let code = match error {
                EjParseError::Empty(_) => "Empty",
                EjParseError::DecodeFailed(_) => "DecodeFailed",
                EjParseError::NoRecords(_) => "NoRecords",
            };
            tracing::warn!(code, "EJ のファイルを読めませんでした");
            return None;
        }
    };
    // 手順 1 (iii): 実在の日時でない記録があれば file ごと使わない
    let bad_dates: Vec<usize> = parsed
        .records
        .iter()
        .filter(|record| {
            chrono::NaiveDateTime::parse_from_str(&record.printed_at, "%Y-%m-%d %H:%M").is_err()
        })
        .map(|record| record.header_line_no)
        .collect();
    if !bad_dates.is_empty() {
        tracing::warn!(line_no = ?bad_dates, "EJ の記録の日時が暦に無いので、ファイルを読みませんでした");
        return None;
    }

    // 手順 2: 伸びうるか（名前の 1 つの判定）と保留
    let grows = !fixed_name_rule().is_match(last_name(&file.filename));
    let missing_final_newline = parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == EjDiagnosticCode::MissingFinalNewline);
    let lines: Vec<&[u8]> = split_crlf(&file.bytes);
    let last = parsed.records.len().saturating_sub(1);
    let records = parsed
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let held = index == last && (grows || missing_final_newline);
            (new_record(record, &lines), held)
        })
        .collect();
    Some(ReadFile { records })
}

/// IO-09-D2 の EJ の名前の規則のうち連番つきの形（`(?i-u)`。29 §29.7.5 手順 4 と同じ）
fn fixed_name_rule() -> &'static Regex {
    static RULE: OnceLock<Regex> = OnceLock::new();
    RULE.get_or_init(|| Regex::new(r"(?i-u)^EJ[0-9]{6}_[0-9]{4}\.TXT$").expect("固定の正規表現"))
}

fn last_name(filename: &str) -> &str {
    filename.rsplit(['\\', '/']).next().unwrap_or(filename)
}

/// CRLF で分けた行（IO-08-D2 と同じ分け方。最後の CRLF の後の空の要素は行にしない）
fn split_crlf(bytes: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'\r' && bytes[index + 1] == b'\n' {
            lines.push(&bytes[start..index]);
            index += 2;
            start = index;
        } else {
            index += 1;
        }
    }
    if start < bytes.len() {
        lines.push(&bytes[start..]);
    }
    lines
}

fn decode_line(bytes: &[u8]) -> String {
    // parse_ej が CP932 strict で読めた file の行だけを渡す
    encoding_rs::SHIFT_JIS
        .decode_without_bom_handling(bytes)
        .0
        .into_owned()
}

/// 手順 2 の最後の項: 記録を `NewEjRecord` にする
fn new_record(record: &EjRecord, lines: &[&[u8]]) -> NewEjRecord {
    let mut body = Vec::with_capacity(record.body.len() + 2);
    for line_no in [record.header_line_no, record.header_line_no + 1] {
        body.push(decode_line(
            lines.get(line_no - 1).copied().unwrap_or_default(),
        ));
    }
    body.extend(record.body.iter().map(|line| line.text.clone()));
    let (kind, settlement_report, settlement_completed) = match &record.kind {
        EjRecordKind::Sale => (EjRecordKindCode::Sale, None, None),
        EjRecordKind::Cancelled => (EjRecordKindCode::Cancelled, None, None),
        EjRecordKind::CashMovement => (EjRecordKindCode::CashMovement, None, None),
        EjRecordKind::Receipt => (EjRecordKindCode::Receipt, None, None),
        EjRecordKind::Settlement { report, completed } => (
            EjRecordKindCode::Settlement,
            Some(
                match report {
                    EjSettlementReport::Daily => "daily",
                    EjSettlementReport::Plu => "plu",
                    EjSettlementReport::Attendance => "attendance",
                }
                .to_string(),
            ),
            Some(*completed),
        ),
        EjRecordKind::Inspection => (EjRecordKindCode::Inspection, None, None),
        EjRecordKind::Settings => (EjRecordKindCode::Settings, None, None),
        EjRecordKind::Training => (EjRecordKindCode::Training, None, None),
        EjRecordKind::Unclassified => (EjRecordKindCode::Unclassified, None, None),
    };
    let total_amount = match (&record.kind, &record.restoration) {
        (
            EjRecordKind::Sale,
            EjRestoration::Restored {
                items, adjustments, ..
            },
        ) => items
            .iter()
            .map(|item| item.amount)
            .chain(adjustments.iter().map(|adjustment| adjustment.amount))
            .try_fold(0_i64, i64::checked_add)
            .filter(|sum| sum.unsigned_abs() <= MAX_SAFE_AMOUNT as u64),
        _ => None,
    };
    NewEjRecord {
        machine_prefix: record.number_prefix.clone(),
        record_no: record.number.clone(),
        printed_at: record.printed_at.clone(),
        mode: match record.mode {
            EjMode::Normal => EjRecordModeCode::Normal,
            EjMode::Return => EjRecordModeCode::Return,
            EjMode::Training => EjRecordModeCode::Training,
            EjMode::Settlement => EjRecordModeCode::Settlement,
            EjMode::Inspection => EjRecordModeCode::Inspection,
            EjMode::Program => EjRecordModeCode::Program,
            EjMode::Program1 => EjRecordModeCode::Program1,
            EjMode::Program3 => EjRecordModeCode::Program3,
            EjMode::Off => EjRecordModeCode::Off,
            EjMode::Unrecognized(_) => EjRecordModeCode::Unrecognized,
        },
        kind,
        settlement_report,
        settlement_completed,
        total_amount,
        body_text: body.join("\n"),
    }
}

/// 手順 3: 鍵ごとに (a) conflict → (b) 済み → (c) 新 → (d) 保留 の順に当てる
fn plan_from(
    conn: &DbConnection,
    read: &[Option<ReadFile>],
    unreadable_files: Vec<String>,
) -> Result<EjImportPlan, BizError> {
    // 鍵 → (file の添字, 記録, 保留か) の列
    let mut by_key: BTreeMap<Key, Vec<(usize, &NewEjRecord, bool)>> = BTreeMap::new();
    for (index, file) in read.iter().enumerate() {
        for (record, held) in file.iter().flat_map(|file| &file.records) {
            by_key
                .entry((record.machine_prefix.clone(), record.record_no.clone()))
                .or_default()
                .push((index, record, *held));
        }
    }
    let keys: Vec<Key> = by_key.keys().cloned().collect();
    let mut saved: HashMap<Key, String> = HashMap::new();
    for chunk in keys.chunks(LOOKUP_CHUNK) {
        saved.extend(ej_repo::find_ej_record_texts(conn, chunk)?);
    }

    let mut plan = EjImportPlan {
        new_records: Vec::new(),
        files_with_new: Vec::new(),
        already_count: 0,
        conflict_count: 0,
        held_count: 0,
        unreadable_files,
        conflicts: Vec::new(),
    };
    let mut files_with_new = BTreeSet::new();
    for (key, entries) in &by_key {
        let saved_text = saved.get(key);
        let mut per_file: HashMap<usize, usize> = HashMap::new();
        for (index, _, _) in entries {
            *per_file.entry(*index).or_default() += 1;
        }
        let compared: Vec<&(usize, &NewEjRecord, bool)> =
            entries.iter().filter(|(_, _, held)| !held).collect();
        let compared_texts: BTreeSet<&str> = compared
            .iter()
            .map(|(_, record, _)| record.body_text.as_str())
            .collect();
        let conflict = per_file.values().any(|count| *count > 1)
            || compared_texts.len() > 1
            || compared_texts
                .first()
                .zip(saved_text)
                .is_some_and(|(text, saved)| text != saved);
        if conflict {
            plan.conflict_count += 1;
            plan.conflicts.push(EjConflict {
                machine_prefix: key.0.clone(),
                record_no: key.1.clone(),
                saved: saved_text.is_some(),
            });
            tracing::warn!(
                machine_prefix = %key.0,
                record_no = %key.1,
                "同じ番号で文の違う EJ の記録があるので保存しません"
            );
        } else if let Some((_, record, _)) = compared.first() {
            if saved_text.is_some() {
                plan.already_count += 1;
            } else {
                plan.new_records.push((*record).clone());
                files_with_new.extend(compared.iter().map(|(index, _, _)| *index));
            }
        } else if saved_text.is_some_and(|saved| {
            entries
                .iter()
                .any(|(_, record, _)| record.body_text == *saved)
        }) {
            plan.already_count += 1;
        } else {
            plan.held_count += 1;
        }
    }
    plan.files_with_new = files_with_new.into_iter().collect();
    Ok(plan)
}

/// E8 の差込み口: TX の中の照合の前に呼ぶ（test だけ）
#[cfg(test)]
pub(crate) mod hook {
    use crate::db::DbConnection;
    use std::cell::Cell;

    thread_local! {
        static BEFORE_TX_RECHECK: Cell<Option<fn(&DbConnection)>> = const { Cell::new(None) };
    }

    pub(crate) struct HookGuard;
    impl Drop for HookGuard {
        fn drop(&mut self) {
            BEFORE_TX_RECHECK.with(|hook| hook.set(None));
        }
    }

    pub(crate) fn arm(hook: fn(&DbConnection)) -> HookGuard {
        BEFORE_TX_RECHECK.with(|cell| cell.set(Some(hook)));
        HookGuard
    }

    pub(super) fn run(conn: &DbConnection) {
        if let Some(hook) = BEFORE_TX_RECHECK.with(Cell::get) {
            hook(conn);
        }
    }
}
