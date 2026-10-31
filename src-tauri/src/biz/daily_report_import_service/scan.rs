//! BIZ-08-D3: レジの SD から読む日報の候補（37 §37.9 手順 1〜9。手順 10 は PLU の runtime の lane）

use crate::biz::daily_report_import_service::parse::{build_bundle_hash, log_parse_diagnostics};
use crate::biz::daily_report_import_service::{
    DailyReportInputFile, DailyReportSdCandidate, DailyReportSdCandidateStatus, DailyReportSdScan,
    DailyReportSdScanResult, DailyReportSdScanSnapshot, DailyReportSdSelection,
};
use crate::biz::BizError;
use crate::constants;
use crate::db::sales_repo;
use crate::db::DbConnection;
use crate::io::daily_report_parser::{self, DailyReportSourceFile};
use crate::io::register_sd::{
    self, RegisterSdArea, RegisterSdEntry, RegisterSdEntryKind, RegisterSdError, RegisterSdRoot,
};
use chrono::NaiveDate;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

const WINDOW_DAYS: i64 = 30;
const NO_SD_MESSAGE: &str =
    "SD が見つかりません。レジの SD をこの PC に差してから、もう一度読んでください。";
const MANY_SD_MESSAGE: &str = "SD が 2 枚以上見つかりました。売上を読む SD だけを差してください。";
const NOT_REGISTER_SD_MESSAGE: &str = "選んだ場所はレジの SD ではありません。SD の中の CASIO の folder か、SD そのものを選んでください。";
const MISSING_SALES_AREA_MESSAGE: &str =
    "SD に売上の folder（XZ）がありません。レジの SD か確かめてください。";
const READ_FAILED_MESSAGE: &str =
    "SD を読めませんでした。SD を差し直して、もう一度読んでください。";
const LINK_OUTSIDE_MESSAGE: &str = "SD の中に、SD の外の場所を指す folder かファイル（リンク）があるため、読みませんでした。レジの SD そのものを差して読むか、リンクの無い SD の写しを選んでください。";

/// 37 §37.9 `scan_register_sd_daily_reports`。DB へは書かない
pub fn scan_register_sd_daily_reports(
    conn: &DbConnection,
    selection: DailyReportSdSelection,
    today: NaiveDate,
) -> Result<DailyReportSdScanResult, BizError> {
    scan_with_roots(conn, selection, today, register_sd::find_register_sd_roots)
}

/// 候補の組（scan の中だけの内部の型）
struct Group {
    area: RegisterSdArea,
    path_date: NaiveDate,
    /// Z001・Z002・Z005 の順
    files: [Option<RegisterSdEntry>; 3],
}

struct Candidate {
    area: RegisterSdArea,
    sort_path: String,
    settlement_no: Option<i64>,
    path_date: NaiveDate,
    wire: DailyReportSdCandidate,
    files: Vec<DailyReportInputFile>,
}

pub(super) fn scan_with_roots(
    conn: &DbConnection,
    selection: DailyReportSdSelection,
    today: NaiveDate,
    find_roots: impl FnOnce() -> Result<Vec<RegisterSdRoot>, RegisterSdError>,
) -> Result<DailyReportSdScanResult, BizError> {
    // 手順 1: root を決める（推測で選ばない）
    let root = match selection {
        DailyReportSdSelection::Auto => {
            let mut roots = find_roots().map_err(sd_error)?;
            match roots.len() {
                0 => return Err(BizError::ImportError(NO_SD_MESSAGE.to_string())),
                1 => roots.remove(0),
                _ => return Err(BizError::ImportError(MANY_SD_MESSAGE.to_string())),
            }
        }
        DailyReportSdSelection::Selected(path) => {
            register_sd::resolve_register_sd_root(&path).map_err(sd_error)?
        }
    };

    // 手順 2: 窓の始まり
    let default_from = today - chrono::Duration::days(WINDOW_DAYS);
    let from = match sales_repo::latest_completed_daily_report_date(conn)? {
        Some(latest) => match NaiveDate::parse_from_str(&latest, "%Y-%m-%d") {
            Ok(latest) => latest.min(default_from),
            Err(error) => {
                tracing::warn!(error = %error, latest, "取込み済みの対象日を読めないので窓を既定にします");
                default_from
            }
        },
        None => default_from,
    };

    // 手順 3: 列挙（途中の失敗は全体の失敗）
    let listing = register_sd::list_register_sd_entries(&root, from).map_err(sd_error)?;
    let skipped_unknown_count = listing
        .entries
        .iter()
        .filter(|entry| entry.kind == RegisterSdEntryKind::Unknown)
        .count() as i64;

    // 手順 4: (area、folder、日、接尾字、連番) が同じ Z001 / Z002 / Z005 を組にする
    let mut groups: BTreeMap<(String, u8, char, Option<u16>), Group> = BTreeMap::new();
    for entry in listing.entries {
        let RegisterSdEntryKind::Z {
            series,
            day,
            suffix,
            backup_seq,
        } = entry.kind
        else {
            continue;
        };
        let Some(slot) = [1, 2, 5].iter().position(|wanted| *wanted == series) else {
            continue;
        };
        let (Some(path_date), Some((folder, _))) =
            (entry.date, entry.relative_path.rsplit_once('\\'))
        else {
            continue;
        };
        let group = groups
            .entry((folder.to_string(), day, suffix, backup_seq))
            .or_insert_with(|| Group {
                area: entry.area,
                path_date,
                files: [None, None, None],
            });
        group.files[slot] = Some(entry);
    }

    // 手順 5〜7
    let mut by_key: HashMap<String, Candidate> = HashMap::new();
    for group in groups.into_values() {
        let candidate = read_group(conn, &root, group)?;
        let key = candidate.wire.candidate_key.clone();
        match by_key.get(&key) {
            // 手順 6: 同じ bytes は 1 つに。XZ を優先し、同じ area なら Z001 の相対 path の辞書順で最初
            Some(kept)
                if (kept.area_rank(), &kept.sort_path)
                    <= (candidate.area_rank(), &candidate.sort_path) => {}
            _ => {
                by_key.insert(key, candidate);
            }
        }
    }

    // 手順 8: path_date の新しい順、同じ日は精算回数の大きい順（読めない束は後ろ、その中は相対 path の順）
    let mut candidates: Vec<Candidate> = by_key.into_values().collect();
    candidates.sort_by(|a, b| {
        (
            Reverse(a.path_date),
            a.settlement_no.is_none(),
            Reverse(a.settlement_no),
            &a.sort_path,
        )
            .cmp(&(
                Reverse(b.path_date),
                b.settlement_no.is_none(),
                Reverse(b.settlement_no),
                &b.sort_path,
            ))
    });

    // 手順 9: NotImported の束だけを snapshot に持つ
    let mut files_by_candidate = HashMap::new();
    let mut wire = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if candidate.wire.status == DailyReportSdCandidateStatus::NotImported {
            files_by_candidate.insert(candidate.wire.candidate_key.clone(), candidate.files);
        }
        wire.push(candidate.wire);
    }
    Ok(DailyReportSdScanResult {
        scan: DailyReportSdScan {
            volume_label: root.volume_label,
            scanned_from: from.format("%Y-%m-%d").to_string(),
            candidates: wire,
            skipped_unknown_count,
        },
        snapshot: DailyReportSdScanSnapshot {
            created_at: Instant::now(),
            files_by_candidate,
        },
    })
}

impl Candidate {
    fn area_rank(&self) -> u8 {
        match self.area {
            RegisterSdArea::Sales => 0,
            RegisterSdArea::Backup => 1,
        }
    }
}

/// 手順 5・7: 組を読み、状態を決める
fn read_group(
    conn: &DbConnection,
    root: &RegisterSdRoot,
    group: Group,
) -> Result<Candidate, BizError> {
    let present: Vec<&RegisterSdEntry> = group.files.iter().flatten().collect();
    let sort_path = present
        .first()
        .map(|entry| entry.relative_path.clone())
        .unwrap_or_default();
    let source_filenames: Vec<String> = present.iter().map(|entry| file_name(entry)).collect();
    let fallback = |status| Candidate {
        area: group.area,
        sort_path: sort_path.clone(),
        settlement_no: None,
        path_date: group.path_date,
        wire: DailyReportSdCandidate {
            candidate_key: format!("sd:{sort_path}"),
            path_date: group.path_date.format("%Y-%m-%d").to_string(),
            report_date: None,
            source_filenames: source_filenames.clone(),
            status,
        },
        files: Vec::new(),
    };
    if present.len() != 3 {
        return Ok(fallback(DailyReportSdCandidateStatus::Incomplete));
    }

    let mut files = Vec::with_capacity(3);
    for entry in &present {
        let read = match register_sd::read_register_sd_file(
            root,
            &entry.relative_path,
            constants::CSV_IMPORT_FILE_SIZE_LIMIT as u64,
        ) {
            Ok(read) => read,
            Err(RegisterSdError::TooLarge { .. }) => {
                return Ok(fallback(DailyReportSdCandidateStatus::Unreadable))
            }
            Err(error) => return Err(sd_error(error)),
        };
        let shape = read.shape;
        // IO-09-D4: 形が外れた file は推測で読まない
        if !(shape.cp932_strict && !shape.has_bom && shape.crlf_only && shape.ends_with_crlf) {
            return Ok(fallback(DailyReportSdCandidateStatus::Unreadable));
        }
        files.push(DailyReportInputFile {
            filename: file_name(entry),
            bytes: read.bytes,
            sd_relative_path: Some(entry.relative_path.clone()),
            source_path: None,
        });
    }
    let parsed = daily_report_parser::parse_daily_report_bundle(
        files
            .iter()
            .map(|file| DailyReportSourceFile {
                filename: file.filename.clone(),
                bytes: file.bytes.clone(),
            })
            .collect(),
    );
    if !parsed.parse_errors.is_empty() {
        log_parse_diagnostics(&parsed.parse_errors);
        return Ok(fallback(DailyReportSdCandidateStatus::Unreadable));
    }
    let bundle_hash = build_bundle_hash(&parsed.source_files);

    // 手順 7
    let status =
        if sales_repo::find_blocking_daily_report_by_bundle_hash(conn, &bundle_hash)?.is_some() {
            DailyReportSdCandidateStatus::Imported
        } else {
            match (parsed.settlement_no, &parsed.report_date) {
                (Some(settlement_no), Some(report_date))
                    if sales_repo::find_same_settlement_daily_report_import(
                        conn,
                        report_date,
                        settlement_no,
                        &bundle_hash,
                    )?
                    .is_some() =>
                {
                    DailyReportSdCandidateStatus::SameSettlementImported
                }
                (Some(_), Some(_)) => DailyReportSdCandidateStatus::NotImported,
                // BIZ-08-D6: 精算回数の無い束は取り込めない
                _ => DailyReportSdCandidateStatus::Unreadable,
            }
        };
    Ok(Candidate {
        area: group.area,
        sort_path,
        settlement_no: parsed.settlement_no,
        path_date: group.path_date,
        wire: DailyReportSdCandidate {
            candidate_key: bundle_hash,
            path_date: group.path_date.format("%Y-%m-%d").to_string(),
            report_date: parsed.report_date,
            source_filenames,
            status,
        },
        files,
    })
}

fn file_name(entry: &RegisterSdEntry) -> String {
    entry
        .relative_path
        .rsplit('\\')
        .next()
        .unwrap_or(&entry.relative_path)
        .to_string()
}

/// 37 §37.9 手順 1・3: IO-09 の失敗を固定の文にする（raw の path を文に入れない）
fn sd_error(error: RegisterSdError) -> BizError {
    tracing::warn!(error = ?error, "レジの SD を読めませんでした");
    let message = match error {
        RegisterSdError::NotRegisterSd => NOT_REGISTER_SD_MESSAGE,
        RegisterSdError::MissingSalesArea => MISSING_SALES_AREA_MESSAGE,
        RegisterSdError::InvalidPath => LINK_OUTSIDE_MESSAGE,
        RegisterSdError::TooLarge { .. } | RegisterSdError::Io { .. } => READ_FAILED_MESSAGE,
    };
    BizError::ImportError(message.to_string())
}
