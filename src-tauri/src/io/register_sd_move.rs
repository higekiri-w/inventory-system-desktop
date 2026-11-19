//! IO-11: レジの SD の `XZ` の Z を `XZ_BKUP` へ移す（D-149）
//!
//! docs/function-design/29-io-daily-report-parser.md §29.9 に基づく実装。
//! 同じ精算の名前の Z の組を、CV17 の取込みと同じ形の名前で、組に共通の 1 つの連番を付けて移す。
//! 移す先の folder が無ければ作り、file は既存の file を置き換えず（`move_no_replace`）、bytes を書かない改名で移す
//! （IO-11-D1）。連番の空きと再開は、移す先の名前を IO-09 の分類（`classify_z`）で見て決める。
//! この module が使う file system の API と crate の中の名前は許可の列だけで（IO-11-D3）、
//! test `test_req401_module_uses_only_allowed_fs_api` が source を検査する。

use super::move_no_replace;
use super::register_sd::{
    classify_z, inside_root, split_relative_path, ErrorModeGuard, RegisterSdArea,
    RegisterSdEntryKind, RegisterSdRoot,
};
use std::fs;
use std::path::{Path, PathBuf};

/// 移す系列（29 §29.9.3 手順 1。EJ とほかの系列は移さない）
const MOVED_SERIES: [u16; 4] = [1, 2, 4, 5];
/// 移す先の area の folder 名
const BACKUP_AREA: &str = "XZ_BKUP";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSdMoveItem {
    /// IO-09 の RegisterSdEntry.relative_path（`XZ\yyyy\mm\Znnn_dda.CSV`）
    pub relative_path: String,
    /// BIZ が読んだ bytes の長さ（移した後の確かめ）
    pub expected_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSdGroupMove {
    /// 移した先の root からの相対 path（`XZ_BKUP\yyyy\mm\Znnn_dda_nnnn.CSV`）、移した順
    pub moved: Vec<String>,
    /// 止めた理由。None = 組の全部を移した
    pub error: Option<RegisterSdMoveError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterSdMoveError {
    /// §29.7.6 手順 1 の検査、または実体の path が root の外（IO-09-D5）
    InvalidPath,
    /// 組の形が違う
    NameRule,
    /// 手順 4 の連番が 0001〜9999 に無い
    NoFreeSequence,
    /// 作る・移す・確かめる途中の失敗（書込み禁止・抜いた・同時に同じ名前ができた〈AlreadyExists〉を含む）
    Io {
        relative_path: String,
        kind: std::io::ErrorKind,
    },
    /// 移動は成功と返ったが、移した先の size が違うか元の名前が残る
    Verify { relative_path: String },
}

/// 手順 1 で確かめた組の 1 本
struct Checked<'a> {
    item: &'a RegisterSdMoveItem,
    parts: Vec<&'a str>,
    series: u16,
    /// 組の全部で同じ
    settlement: SettlementName,
}

/// 精算の名前（folder の年・月と、IO-09 の分類の日・接尾字）
#[derive(Clone, Copy, PartialEq, Eq)]
struct SettlementName {
    year: i32,
    month: u32,
    day: u8,
    suffix: char,
}

/// 29 §29.9.2
pub fn move_register_sd_group_to_backup(
    root: &RegisterSdRoot,
    group: &[RegisterSdMoveItem],
) -> RegisterSdGroupMove {
    let _guard = ErrorModeGuard::new();
    let mut moved = Vec::new();
    let error = move_group(root, group, &mut moved).err();
    RegisterSdGroupMove { moved, error }
}

fn move_group(
    root: &RegisterSdRoot,
    group: &[RegisterSdMoveItem],
    moved: &mut Vec<String>,
) -> Result<(), RegisterSdMoveError> {
    // 手順 1: 名前と組の形（SD に書かない）
    let checked = check_group(group)?;
    let first = &checked[0];
    let (year, month) = (first.parts[1], first.parts[2]);

    // 手順 2: 組の各 file の実体が root の下の通常の file
    let root_real = fs::canonicalize(&root.path).map_err(|error| io_error("", &error))?;
    let mut sources = Vec::with_capacity(checked.len());
    for file in &checked {
        let joined = file
            .parts
            .iter()
            .fold(root_real.clone(), |path, part| path.join(part));
        let real = resolve_inside(&root_real, &joined, &file.item.relative_path)?;
        let metadata =
            fs::metadata(&real).map_err(|error| io_error(&file.item.relative_path, &error))?;
        if !metadata.is_file() {
            return Err(RegisterSdMoveError::Io {
                relative_path: file.item.relative_path.clone(),
                kind: std::io::ErrorKind::InvalidInput,
            });
        }
        sources.push(real);
    }

    // 手順 3: `XZ_BKUP\yyyy\mm` を用意する（既存の段の大文字小文字はそのまま使う）
    let (area_real, area_name) = ensure_dir(&root_real, &root_real, BACKUP_AREA, "")?;
    let (year_real, year_name) = ensure_dir(&root_real, &area_real, year, &area_name)?;
    let year_relative = format!("{area_name}\\{year_name}");
    let (month_real, month_name) = ensure_dir(&root_real, &year_real, month, &year_relative)?;
    let month_relative = format!("{year_relative}\\{month_name}");

    // 手順 4: 組に共通の連番
    let taken = taken_sequences(&month_real, &month_relative, first.settlement)?;
    let in_group: Vec<u16> = checked.iter().map(|file| file.series).collect();
    let sequence = choose_sequence(&taken, &in_group)?;

    // 手順 5: 渡された順に 1 本ずつ移し、確かめてから次へ（IO-11-D2）
    for (file, source) in checked.iter().zip(&sources) {
        let name = with_sequence(file.parts[3], sequence);
        let destination = month_real.join(&name);
        let relative = &file.item.relative_path;
        move_no_replace(source, &destination).map_err(|error| io_error(relative, &error))?;
        moved.push(format!("{month_relative}\\{name}"));
        verify_moved(source, &destination, file.item)?;
    }
    Ok(())
}

/// 手順 5 の確かめ: 先が通常の file で size が `expected_len`、元の名前が無い。違えば `Verify`、読めなければ `Io`
fn verify_moved(
    source: &Path,
    destination: &Path,
    item: &RegisterSdMoveItem,
) -> Result<(), RegisterSdMoveError> {
    let relative = &item.relative_path;
    let metadata = fs::metadata(destination).map_err(|error| io_error(relative, &error))?;
    let source_left = source
        .try_exists()
        .map_err(|error| io_error(relative, &error))?;
    if !metadata.is_file() || metadata.len() != item.expected_len || source_left {
        return Err(RegisterSdMoveError::Verify {
            relative_path: relative.clone(),
        });
    }
    Ok(())
}

/// 手順 1: `XZ\yyyy\mm\` の連番の無い Z001・Z002・Z004・Z005 で、folder・日・接尾字が揃い、系列が重ならない 1〜4 本
fn check_group(group: &[RegisterSdMoveItem]) -> Result<Vec<Checked<'_>>, RegisterSdMoveError> {
    let mut checked = Vec::with_capacity(group.len());
    for item in group {
        let parts =
            split_relative_path(&item.relative_path).ok_or(RegisterSdMoveError::InvalidPath)?;
        checked.push((item, parts));
    }
    if checked.is_empty() || checked.len() > MOVED_SERIES.len() {
        return Err(RegisterSdMoveError::NameRule);
    }
    let mut out: Vec<Checked> = Vec::with_capacity(checked.len());
    for (item, parts) in checked {
        let [area, year, month, name] = parts.as_slice() else {
            return Err(RegisterSdMoveError::NameRule);
        };
        let (Some(year_number), Some(month_number)) = (digits(year, 4), digits(month, 2)) else {
            return Err(RegisterSdMoveError::NameRule);
        };
        if !area.eq_ignore_ascii_case("XZ") {
            return Err(RegisterSdMoveError::NameRule);
        }
        let (
            RegisterSdEntryKind::Z {
                series,
                day,
                suffix,
                backup_seq: None,
            },
            Some(_),
        ) = classify_z(
            name,
            RegisterSdArea::Sales,
            year_number as i32,
            month_number,
        )
        else {
            return Err(RegisterSdMoveError::NameRule);
        };
        if !MOVED_SERIES.contains(&series) || out.iter().any(|file| file.series == series) {
            return Err(RegisterSdMoveError::NameRule);
        }
        let settlement = SettlementName {
            year: year_number as i32,
            month: month_number,
            day,
            suffix,
        };
        if out
            .first()
            .is_some_and(|first| first.settlement != settlement)
        {
            return Err(RegisterSdMoveError::NameRule);
        }
        out.push(Checked {
            item,
            parts,
            series,
            settlement,
        });
    }
    Ok(out)
}

/// 手順 3 の 1 段: `parent` の下の `name`（大文字小文字を区別しない）を、在れば使い、無ければ作る。
/// 実体が root の下の directory であることを確かめる
fn ensure_dir(
    root_real: &Path,
    parent: &Path,
    name: &str,
    parent_relative: &str,
) -> Result<(PathBuf, String), RegisterSdMoveError> {
    let relative = |found: &str| {
        if parent_relative.is_empty() {
            found.to_string()
        } else {
            format!("{parent_relative}\\{found}")
        }
    };
    let existing = fs::read_dir(parent)
        .map_err(|error| io_error(parent_relative, &error))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_str().map(str::to_string))
                .map_err(|error| io_error(parent_relative, &error))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .find(|found| found.eq_ignore_ascii_case(name));
    let found = match existing {
        Some(found) => found,
        None => {
            match fs::create_dir(parent.join(name)) {
                Ok(()) => {}
                // 同時に作られた（作った folder は失敗しても消さない。29 §29.9.3 手順 3）
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(io_error(&relative(name), &error)),
            }
            name.to_string()
        }
    };
    let relative = relative(&found);
    let real = resolve_inside(root_real, &parent.join(&found), &relative)?;
    let metadata = fs::metadata(&real).map_err(|error| io_error(&relative, &error))?;
    if !metadata.is_dir() {
        return Err(RegisterSdMoveError::Io {
            relative_path: relative,
            kind: std::io::ErrorKind::InvalidInput,
        });
    }
    Ok((real, found))
}

/// 移す先の folder の entry を 1 回の `read_dir` で読み、IO-09 の分類（`classify_z`）で組と同じ日・接尾字の
/// Z になる名前の (系列, 連番) を返す。綴りが違っても IO-09 が同じ Z と読む名前（ASCII の大文字小文字の違い）は
/// 同じ名前として数え、IO-09 が Unknown と読む名前（ASCII でない文字を含む名前など）は数えない
/// （移した file を IO-09 が組に戻すときの見方と揃える。29 §29.9.3 手順 4）
fn taken_sequences(
    dir: &Path,
    relative: &str,
    settlement: SettlementName,
) -> Result<Vec<(u16, u16)>, RegisterSdMoveError> {
    let mut taken = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| io_error(relative, &error))? {
        let entry = entry.map_err(|error| io_error(relative, &error))?;
        let name = entry.file_name();
        // UTF-16 にならない名前は IO-09 も読まない（規則に合わない）
        let Some(name) = name.to_str() else {
            continue;
        };
        if let (
            RegisterSdEntryKind::Z {
                series,
                day,
                suffix,
                backup_seq: Some(sequence),
            },
            _,
        ) = classify_z(
            name,
            RegisterSdArea::Backup,
            settlement.year,
            settlement.month,
        ) {
            if (day, suffix) == (settlement.day, settlement.suffix) {
                taken.push((series, sequence));
            }
        }
    }
    Ok(taken)
}

/// 29 §29.9.3 手順 4: 組に共通の連番を選ぶ。`taken` は移す先に在る、組と同じ日・接尾字の (系列, 連番)
fn choose_sequence(taken: &[(u16, u16)], in_group: &[u16]) -> Result<u16, RegisterSdMoveError> {
    let present = |series: u16, sequence: u16| taken.contains(&(series, sequence));
    let others: Vec<u16> = MOVED_SERIES
        .iter()
        .copied()
        .filter(|series| !in_group.contains(series))
        .collect();
    // (a) 前の片付けの続き: 組に無い系列の名前が在り、組の名前が全部空いている m。
    // 候補の中で組に無い系列が全部在る m を先に選び、無いときだけ最大の m（別に大きい連番の
    // 単独の同じ名前があっても、前に移した連番へ揃える）
    let resumable: Vec<u16> = (1..=9999)
        .filter(|&m| {
            others.iter().any(|&series| present(series, m))
                && in_group.iter().all(|&series| !present(series, m))
        })
        .collect();
    let complete = resumable
        .iter()
        .copied()
        .filter(|&m| others.iter().all(|&series| present(series, m)))
        .max();
    if let Some(m) = complete.or_else(|| resumable.iter().copied().max()) {
        return Ok(m);
    }
    // (b) 4 系列のどれの名前も無い最小の n
    (1..=9999)
        .find(|&n| MOVED_SERIES.iter().all(|&series| !present(series, n)))
        .ok_or(RegisterSdMoveError::NoFreeSequence)
}

/// 拡張子の前に `_` と 4 桁の連番を足す（`Z001_06 .CSV` → `Z001_06 _0001.CSV`）
fn with_sequence(name: &str, sequence: u16) -> String {
    match name.rfind('.') {
        Some(dot) => format!("{}_{sequence:04}{}", &name[..dot], &name[dot..]),
        None => format!("{name}_{sequence:04}"),
    }
}

fn digits(text: &str, len: usize) -> Option<u32> {
    (text.len() == len && text.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

/// IO-09-D5: 実体の path に解き（解けなければ `Io`）、root の下でなければ `InvalidPath`。
/// 先に解いて `Io` を分けてから IO-09 の `inside_root` で比べる（IO-09 の error の型を
/// この module に持ち込まない。IO-11-D3 の名前の許可の列）
fn resolve_inside(
    root_real: &Path,
    path: &Path,
    relative_path: &str,
) -> Result<PathBuf, RegisterSdMoveError> {
    fs::canonicalize(path).map_err(|error| io_error(relative_path, &error))?;
    inside_root(root_real, path, relative_path).map_err(|_| RegisterSdMoveError::InvalidPath)
}

fn io_error(relative_path: &str, error: &std::io::Error) -> RegisterSdMoveError {
    RegisterSdMoveError::Io {
        relative_path: relative_path.to_string(),
        kind: error.kind(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::register_sd::list_register_sd_entries;
    use crate::io::REGISTER_SD_FOLDER;
    use chrono::NaiveDate;
    use regex::Regex;
    use std::collections::BTreeMap;

    fn make_root(base: &Path) -> RegisterSdRoot {
        let path = REGISTER_SD_FOLDER
            .iter()
            .fold(base.to_path_buf(), |path, part| path.join(part));
        fs::create_dir_all(path.join("XZ")).unwrap();
        RegisterSdRoot {
            path,
            volume_label: "T:".to_string(),
        }
    }

    fn path_of(root: &RegisterSdRoot, relative: &str) -> PathBuf {
        relative
            .split('\\')
            .fold(root.path.clone(), |path, part| path.join(part))
    }

    fn put(root: &RegisterSdRoot, relative: &str, bytes: &[u8]) {
        let path = path_of(root, relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    /// root の下の全 file の相対 path（`\` 区切り）と bytes
    fn tree(root: &RegisterSdRoot) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        let mut stack = vec![root.path.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let relative = path
                        .strip_prefix(&root.path)
                        .unwrap()
                        .to_string_lossy()
                        .replace('/', "\\");
                    out.insert(relative, fs::read(&path).unwrap_or_default());
                }
            }
        }
        out
    }

    /// `XZ\2026\10\Z00k_06 .CSV` に bytes `Z00k` を置き、組の項目を返す（渡した系列の順）
    fn put_group(
        root: &RegisterSdRoot,
        series: &[u16],
        name_tail: &str,
    ) -> Vec<RegisterSdMoveItem> {
        series
            .iter()
            .map(|series| {
                let relative = format!("XZ\\2026\\10\\Z{series:03}_{name_tail}");
                let bytes = format!("Z{series:03}{name_tail}\r\n");
                put(root, &relative, bytes.as_bytes());
                RegisterSdMoveItem {
                    relative_path: relative,
                    expected_len: bytes.len() as u64,
                }
            })
            .collect()
    }

    #[test]
    fn test_req401_moves_group_with_cv17_shape_name() {
        // REQ-401 / IO-11-D1 §29.9.3 手順 4・5: CV17 と同じ形（拡張子の前に `_0001`、接尾字の空白を残す）で移す
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        let before = tree(&root);
        let result = move_register_sd_group_to_backup(&root, &group);
        assert_eq!(result.error, None);
        assert_eq!(
            result.moved,
            [
                "XZ_BKUP\\2026\\10\\Z004_06 _0001.CSV",
                "XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV",
                "XZ_BKUP\\2026\\10\\Z002_06 _0001.CSV",
                "XZ_BKUP\\2026\\10\\Z005_06 _0001.CSV",
            ]
        );
        let after = tree(&root);
        for (item, moved) in group.iter().zip(&result.moved) {
            assert!(!after.contains_key(&item.relative_path));
            assert_eq!(after[moved], before[&item.relative_path]);
        }
        assert_eq!(after.len(), before.len());
    }

    #[test]
    fn test_req401_skips_taken_sequence_without_replacing() {
        // REQ-401 / IO-11-D1 / 71 §71.4.1: 前の組が `_0001` を使っていれば `_0002` で移し、既存の bytes を変えない
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        for series in [1, 2, 4, 5] {
            put(
                &root,
                &format!("XZ_BKUP\\2026\\10\\Z{series:03}_06 _0001.CSV"),
                b"earlier",
            );
        }
        let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        let result = move_register_sd_group_to_backup(&root, &group);
        assert_eq!(result.error, None);
        assert!(result.moved.iter().all(|path| path.ends_with("_0002.CSV")));
        let after = tree(&root);
        for series in [1, 2, 4, 5] {
            assert_eq!(
                after[&format!("XZ_BKUP\\2026\\10\\Z{series:03}_06 _0001.CSV")],
                b"earlier"
            );
        }
    }

    #[test]
    fn test_req401_taken_sequence_is_seen_by_io09_classification() {
        // REQ-401 / IO-11-D1 §29.9.3 手順 4 (b)（PR #228 の closure 797 の P2、Gated Amendment 1）: 連番の空きは、
        // 移す先の名前を IO-09 の分類（`classify_z`）で見る。綴りが違っても IO-09 が同じ日・接尾字・系列・連番の Z と
        // 読む名前（ASCII の大文字小文字の違い。小文字の `z002_06 _0001.csv`）が在れば、その連番は使わず `_0002` に
        // 揃える。IO-09 が Unknown と読む名前（ASCII でない綴り。U+017F の `.CſV`）は占有にせず、組は `_0001`。
        // 移す名前は元の綴りのままで、先に在った file は変わらない
        for (taken, sequence) in [("z002_06 _0001.csv", "0002"), ("Z002_06 _0001.CſV", "0001")] {
            let dir = tempfile::tempdir().unwrap();
            let root = make_root(dir.path());
            let taken = format!("XZ_BKUP\\2026\\10\\{taken}");
            put(&root, &taken, b"other");
            let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
            let result = move_register_sd_group_to_backup(&root, &group);
            assert_eq!(result.error, None, "{taken}");
            assert_eq!(
                result.moved,
                [4, 1, 2, 5]
                    .map(|series| format!("XZ_BKUP\\2026\\10\\Z{series:03}_06 _{sequence}.CSV")),
                "{taken}"
            );
            assert_eq!(tree(&root)[&taken], b"other");
        }

        // 別の日・別の接尾字の Z は、同じ連番でも占有にしない（組は `_0001`）
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ_BKUP\\2026\\10\\Z001_05 _0001.CSV", b"other day");
        put(
            &root,
            "XZ_BKUP\\2026\\10\\Z001_06A_0001.CSV",
            b"other suffix",
        );
        let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        let result = move_register_sd_group_to_backup(&root, &group);
        assert_eq!(result.error, None);
        assert!(
            result.moved.iter().all(|path| path.ends_with("_0001.CSV")),
            "{:?}",
            result.moved
        );
    }

    #[test]
    fn test_req401_group_shares_sequence_in_asymmetric_tree() {
        // REQ-401 / IO-11 手順 4 (b): `Z001_06 _0001.CSV` だけがある先でも、組は 1 つの連番 `_0002` に揃う
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV", b"other");
        let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        let result = move_register_sd_group_to_backup(&root, &group);
        assert_eq!(result.error, None);
        assert_eq!(result.moved.len(), 4);
        assert!(
            result.moved.iter().all(|path| path.ends_with("_0002.CSV")),
            "{:?}",
            result.moved
        );
    }

    #[test]
    fn test_req401_resumed_group_reuses_sequence() {
        // REQ-401 / IO-11 手順 4 (a): 移す先に別の `Z001_06 _0001.CSV` があり、組の Z004・Z001 を `_0002` で
        // 移した後に残りの Z002・Z005 を渡すと `_0002` で移る。別の `Z004_06 _0001.CSV` もあるので、
        // 組に無い系列（Z001・Z004）が全部在る連番は `_0001` と `_0002` の 2 つで、その最大を選ぶ
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV", b"other");
        put(&root, "XZ_BKUP\\2026\\10\\Z004_06 _0001.CSV", b"other");
        let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        let first = move_register_sd_group_to_backup(&root, &group[..2]);
        assert_eq!(first.error, None);
        assert!(first.moved.iter().all(|path| path.ends_with("_0002.CSV")));
        let rest = move_register_sd_group_to_backup(&root, &group[2..]);
        assert_eq!(rest.error, None);
        assert_eq!(
            rest.moved,
            [
                "XZ_BKUP\\2026\\10\\Z002_06 _0002.CSV",
                "XZ_BKUP\\2026\\10\\Z005_06 _0002.CSV",
            ]
        );

        // 組に無い系列が全部は在らない連番しか無いとき（Z001 だけの `_0001` と `_0003`）は、その最大へ合わせる
        let dir2 = tempfile::tempdir().unwrap();
        let root2 = make_root(dir2.path());
        put(&root2, "XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV", b"other");
        put(&root2, "XZ_BKUP\\2026\\10\\Z001_06 _0003.CSV", b"other");
        let rest2 = put_group(&root2, &[2, 5], "06 .CSV");
        let result2 = move_register_sd_group_to_backup(&root2, &rest2);
        assert_eq!(result2.error, None);
        assert_eq!(
            result2.moved,
            [
                "XZ_BKUP\\2026\\10\\Z002_06 _0003.CSV",
                "XZ_BKUP\\2026\\10\\Z005_06 _0003.CSV",
            ]
        );
    }

    #[test]
    fn test_req401_resumed_group_is_seen_by_io09_classification() {
        // REQ-401 / IO-11-D1 §29.9.3 手順 4 (a)（PR #228 の closure 797 の P2、Gated Amendment 1）: 前の片付けの
        // 続きも IO-09 の分類で見る。前に移した Z004・Z001 の `_0002` を、残りの先頭（Z002）の綴りが小文字の
        // 拡張子（`.csv`）でも見つけて揃える。別の `Z001_06 _0001.CSV` があるので、先頭の綴りから作った名前を
        // 大文字小文字を畳まずに比べると残りは `_0001` へ行く
        let resumed_tree = |head_tail: &str| {
            let dir = tempfile::tempdir().unwrap();
            let root = make_root(dir.path());
            put(&root, "XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV", b"other");
            put(&root, "XZ_BKUP\\2026\\10\\Z004_06 _0002.CSV", b"moved");
            put(&root, "XZ_BKUP\\2026\\10\\Z001_06 _0002.CSV", b"moved");
            let rest = [
                put_group(&root, &[2], head_tail),
                put_group(&root, &[5], "06 .CSV"),
            ]
            .concat();
            (dir, root, rest)
        };
        let (_dir, root, rest) = resumed_tree("06 .csv");
        let result = move_register_sd_group_to_backup(&root, &rest);
        assert_eq!(result.error, None);
        assert_eq!(
            result.moved,
            [
                "XZ_BKUP\\2026\\10\\Z002_06 _0002.csv",
                "XZ_BKUP\\2026\\10\\Z005_06 _0002.CSV",
            ]
        );

        // 残りの先頭が ASCII でない綴り（U+017F の `.CſV`）なら、IO-09 が Unknown と読む名前なので組にしない:
        // `NameRule` で 1 本も移さない（29 §29.9.3 手順 1）
        let (_dir, root, rest) = resumed_tree("06 .CſV");
        let before = tree(&root);
        let result = move_register_sd_group_to_backup(&root, &rest);
        assert_eq!(result.error, Some(RegisterSdMoveError::NameRule));
        assert!(result.moved.is_empty(), "{:?}", result.moved);
        assert_eq!(tree(&root), before);
    }

    #[test]
    fn test_req401_no_free_sequence_when_all_taken() {
        // REQ-401 / IO-11 手順 4: 0001〜9999 のどの連番も 4 系列のどれかの名前で埋まっていれば NoFreeSequence。
        // 9999 本の tree を作らず、移す先に在る (系列, 連番) の列（`taken_sequences` が返す形）を直接渡す
        let taken: Vec<(u16, u16)> = (1..=9999).map(|n| (1, n)).collect();
        assert_eq!(
            choose_sequence(&taken, &[4, 1, 2, 5]),
            Err(RegisterSdMoveError::NoFreeSequence)
        );
        // 最後の連番だけが空いていれば 9999 を選ぶ
        assert_eq!(choose_sequence(&taken[..9998], &[4, 1, 2, 5]), Ok(9999));
    }

    #[test]
    fn test_req401_resumed_group_ignores_larger_lone_sequence() {
        // REQ-401 / IO-11 手順 4 (a)（closure 748 の P3-A）: 別に大きい連番の単独の同じ名前
        // （`Z001_06 _0003.CSV`）があっても、残りは前に移した連番（Z004・Z001 の `_0001`）へ揃う
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ_BKUP\\2026\\10\\Z001_06 _0003.CSV", b"lone");
        let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        let first = move_register_sd_group_to_backup(&root, &group[..2]);
        assert_eq!(first.error, None);
        assert!(first.moved.iter().all(|path| path.ends_with("_0001.CSV")));
        let rest = move_register_sd_group_to_backup(&root, &group[2..]);
        assert_eq!(rest.error, None);
        assert_eq!(
            rest.moved,
            [
                "XZ_BKUP\\2026\\10\\Z002_06 _0001.CSV",
                "XZ_BKUP\\2026\\10\\Z005_06 _0001.CSV",
            ]
        );
        assert_eq!(tree(&root)["XZ_BKUP\\2026\\10\\Z001_06 _0003.CSV"], b"lone");
    }

    #[test]
    fn test_req401_listed_as_backup_after_move() {
        // REQ-401 / IO-11-D1: 移した file を IO-09 が `XZ_BKUP` の Z（backup_seq = Some(1)）に分類する
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let group = put_group(&root, &[4, 1, 2, 5], "06A.CSV");
        assert_eq!(move_register_sd_group_to_backup(&root, &group).error, None);
        let listing =
            list_register_sd_entries(&root, NaiveDate::from_ymd_opt(2026, 10, 1).unwrap()).unwrap();
        assert_eq!(listing.entries.len(), 4);
        for entry in &listing.entries {
            assert_eq!(
                entry.area,
                RegisterSdArea::Backup,
                "{}",
                entry.relative_path
            );
            assert!(matches!(
                entry.kind,
                RegisterSdEntryKind::Z {
                    day: 6,
                    suffix: 'A',
                    backup_seq: Some(1),
                    ..
                }
            ));
        }
    }

    #[test]
    fn test_req401_creates_missing_backup_folders() {
        // REQ-401 / IO-11 手順 3: `XZ_BKUP`・年・月が無ければ作る。既存の段の大文字小文字はそのまま使う
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let group = put_group(&root, &[1], "06 .CSV");
        let result = move_register_sd_group_to_backup(&root, &group);
        assert_eq!(result.error, None);
        assert_eq!(result.moved, ["XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV"]);

        let dir2 = tempfile::tempdir().unwrap();
        let root2 = make_root(dir2.path());
        fs::create_dir_all(root2.path.join("xz_bkup")).unwrap();
        let group2 = put_group(&root2, &[1], "06 .CSV");
        let result2 = move_register_sd_group_to_backup(&root2, &group2);
        assert_eq!(result2.error, None);
        assert_eq!(result2.moved, ["xz_bkup\\2026\\10\\Z001_06 _0001.CSV"]);
    }

    #[test]
    fn test_req401_backup_area_not_directory_is_error() {
        // REQ-401 / IO-11 手順 3: `XZ_BKUP` が file なら上書きせず Io（InvalidInput）で、1 本も移さない
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ_BKUP", b"not a folder");
        let group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        let before = tree(&root);
        let result = move_register_sd_group_to_backup(&root, &group);
        assert_eq!(
            result.error,
            Some(RegisterSdMoveError::Io {
                relative_path: "XZ_BKUP".to_string(),
                kind: std::io::ErrorKind::InvalidInput,
            })
        );
        assert!(result.moved.is_empty());
        assert_eq!(tree(&root), before);
    }

    #[test]
    fn test_req401_rejects_outside_root_and_bad_paths() {
        // REQ-401 / IO-11 手順 1・2 / IO-09-D5: 名前の規則・組の形・root の外を、書く前に止める
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let good = put_group(&root, &[1, 2, 4, 5], "06 .CSV");
        put(&root, "XZ\\EJ261006.TXT", b"ej");
        put(&root, "XZ\\2026\\10\\Z003_06 .CSV", b"z3");
        put(&root, "XZ\\2026\\10\\NOTE.DAT", b"x");
        put(&root, "XZ_BKUP\\2026\\10\\Z001_05 _0001.CSV", b"b");
        put(&root, "XZ_BKUP\\2026\\10\\Z001_06 .CSV", b"nb");
        put(&root, "XZ\\2026\\10\\Z002_07 .CSV", b"d7");
        put(&root, "XZ\\2026\\10\\Z002_06A.CSV", b"sa");
        put(&root, "XZ\\2026\\09\\Z002_06 .CSV", b"m9");
        let item = |relative: &str| RegisterSdMoveItem {
            relative_path: relative.to_string(),
            expected_len: 2,
        };
        let before = tree(&root);
        let cases: Vec<(Vec<RegisterSdMoveItem>, RegisterSdMoveError)> = vec![
            (vec![item("..\\x")], RegisterSdMoveError::InvalidPath),
            (
                vec![item("XZ\\..\\XZ\\2026\\10\\Z001_06 .CSV")],
                RegisterSdMoveError::InvalidPath,
            ),
            (vec![item("C:\\XZ")], RegisterSdMoveError::InvalidPath),
            (
                vec![item("XZ\\EJ261006.TXT")],
                RegisterSdMoveError::NameRule,
            ),
            (
                vec![item("XZ\\2026\\10\\Z003_06 .CSV")],
                RegisterSdMoveError::NameRule,
            ),
            (
                vec![item("XZ\\2026\\10\\NOTE.DAT")],
                RegisterSdMoveError::NameRule,
            ),
            (
                vec![item("XZ_BKUP\\2026\\10\\Z001_05 _0001.CSV")],
                RegisterSdMoveError::NameRule,
            ),
            // `XZ` でない場所の、連番の無い Z の名前
            (
                vec![item("XZ_BKUP\\2026\\10\\Z001_06 .CSV")],
                RegisterSdMoveError::NameRule,
            ),
            (
                vec![good[0].clone(), item("XZ\\2026\\10\\Z002_07 .CSV")],
                RegisterSdMoveError::NameRule,
            ),
            (
                vec![good[0].clone(), item("XZ\\2026\\10\\Z002_06A.CSV")],
                RegisterSdMoveError::NameRule,
            ),
            (
                vec![good[0].clone(), item("XZ\\2026\\09\\Z002_06 .CSV")],
                RegisterSdMoveError::NameRule,
            ),
            (
                vec![good[0].clone(), good[0].clone()],
                RegisterSdMoveError::NameRule,
            ),
            (vec![], RegisterSdMoveError::NameRule),
            (
                [good.clone(), vec![good[0].clone()]].concat(),
                RegisterSdMoveError::NameRule,
            ),
        ];
        for (group, expected) in cases {
            let result = move_register_sd_group_to_backup(&root, &group);
            assert_eq!(result.error, Some(expected), "{group:?}");
            assert!(result.moved.is_empty());
        }
        assert_eq!(tree(&root), before);
        assert!(!root
            .path
            .join("XZ_BKUP")
            .join("2026")
            .join("10")
            .join("Z001_06 _0001.CSV")
            .exists());

        // root の外を指す link（Windows 以外で作れる形）
        #[cfg(unix)]
        {
            let outside_dir = tempfile::tempdir().unwrap();
            let outside = outside_dir.path().join("outside.CSV");
            fs::write(&outside, b"outside").unwrap();
            let linked = make_root(&dir.path().join("linked"));
            let month = linked.path.join("XZ").join("2026").join("10");
            fs::create_dir_all(&month).unwrap();
            std::os::unix::fs::symlink(&outside, month.join("Z001_06 .CSV")).unwrap();
            let result =
                move_register_sd_group_to_backup(&linked, &[item("XZ\\2026\\10\\Z001_06 .CSV")]);
            assert_eq!(result.error, Some(RegisterSdMoveError::InvalidPath));
            assert!(result.moved.is_empty());
            assert_eq!(fs::read(&outside).unwrap(), b"outside");
            assert!(!linked.path.join("XZ_BKUP").exists());

            // 移す先の段（`XZ_BKUP`）が root の外を指す link: 1 本も移さず、外に何も作らない
            let outside_backup = tempfile::tempdir().unwrap();
            let linked_backup = make_root(&dir.path().join("linked-backup"));
            let group = put_group(&linked_backup, &[1], "06 .CSV");
            std::os::unix::fs::symlink(outside_backup.path(), linked_backup.path.join("XZ_BKUP"))
                .unwrap();
            let result = move_register_sd_group_to_backup(&linked_backup, &group);
            assert_eq!(result.error, Some(RegisterSdMoveError::InvalidPath));
            assert!(result.moved.is_empty());
            assert!(path_of(&linked_backup, &group[0].relative_path).exists());
            assert_eq!(fs::read_dir(outside_backup.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn test_req401_verify_detects_size_mismatch() {
        // REQ-401 / IO-11-D2 手順 5: 移した先の size が `expected_len` と違えば Verify で止める
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let mut group = put_group(&root, &[4, 1], "06 .CSV");
        group[1].expected_len += 1;
        let result = move_register_sd_group_to_backup(&root, &group);
        assert_eq!(
            result.error,
            Some(RegisterSdMoveError::Verify {
                relative_path: "XZ\\2026\\10\\Z001_06 .CSV".to_string()
            })
        );
        assert_eq!(result.moved.len(), 2);

        // 元の名前が残るのも Verify。`move_no_replace` は成功すれば元を残さないので、手順 5 の確かめに
        // 元も先もある状態を直接渡す（元が無くなれば通る）
        let left = put_group(&root, &[2], "06 .CSV");
        let source = path_of(&root, &left[0].relative_path);
        let destination = path_of(&root, "XZ_BKUP\\2026\\10\\Z002_06 _0001.CSV");
        fs::copy(&source, &destination).unwrap();
        assert_eq!(
            verify_moved(&source, &destination, &left[0]),
            Err(RegisterSdMoveError::Verify {
                relative_path: left[0].relative_path.clone()
            })
        );
        fs::remove_file(&source).unwrap();
        assert_eq!(verify_moved(&source, &destination, &left[0]), Ok(()));
    }

    #[test]
    fn test_req401_group_stops_at_first_failure() {
        // REQ-401 / IO-11-D2: 1 本目で失敗したら組の残りを移さない
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let mut group = put_group(&root, &[4, 1, 2, 5], "06 .CSV");
        group[0].expected_len = 0;
        let result = move_register_sd_group_to_backup(&root, &group);
        assert!(matches!(
            result.error,
            Some(RegisterSdMoveError::Verify { .. })
        ));
        assert_eq!(result.moved, ["XZ_BKUP\\2026\\10\\Z004_06 _0001.CSV"]);
        let after = tree(&root);
        for item in &group[1..] {
            assert!(
                after.contains_key(&item.relative_path),
                "{}",
                item.relative_path
            );
        }
    }

    // ------------------------------------------------------------------
    // IO-11-D3 の source の検査
    // ------------------------------------------------------------------

    /// 本体と test の境の属性（architecture_test の行の追跡に掛からないよう分けて書く）
    const TEST_ATTR: &str = concat!("#[cfg(", "test)]");

    /// `cfg(test)` の属性 より前の本体から、行と block の comment を外した文字列
    fn production_text(source: &str) -> String {
        let body = source.split(TEST_ATTR).next().unwrap_or("");
        let without_block = Regex::new(r"(?s)/\*.*?\*/").unwrap().replace_all(body, "");
        without_block
            .lines()
            .map(|line| line.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn group_items(group: &str) -> Vec<String> {
        group
            .trim_matches(|c| c == '{' || c == '}')
            .split(',')
            .map(|item| item.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|item| !item.is_empty())
            .collect()
    }

    /// `crate::`・`super::` で始まる path の葉（`prefix::{a, b}` は `prefix::a`・`prefix::b`）
    fn crate_paths(text: &str) -> Vec<String> {
        let mut out = Vec::new();
        for caps in Regex::new(r"\b(crate|super)\s*::\s*([\w\s:{},*]*)")
            .unwrap()
            .captures_iter(text)
        {
            let path: String = caps[2].split_whitespace().collect::<Vec<_>>().join(" ");
            let base = if &caps[1] == "crate" {
                "crate::"
            } else {
                "super::"
            };
            match path.find('{') {
                Some(start) => {
                    let prefix = path[..start].trim().trim_end_matches("::").trim();
                    let end = path.rfind('}').map_or(path.len(), |end| end + 1);
                    for item in group_items(&path[start..end]) {
                        out.push(format!("{base}{prefix}::{item}"));
                    }
                }
                None => out.push(format!("{base}{}", path.trim().trim_end_matches("::"))),
            }
        }
        out
    }

    /// IO-11-D3 の許可の列の外の出現を返す（空なら green）
    fn violations(source: &str) -> Vec<String> {
        const FS_ITEMS: [&str; 10] = [
            "self",
            "File",
            "ReadDir",
            "DirEntry",
            "Metadata",
            "FileType",
            "read_dir",
            "metadata",
            "canonicalize",
            "create_dir",
        ];
        const CRATE_NAMES: [&str; 10] = [
            "super::register_sd::RegisterSdRoot",
            "super::register_sd::RegisterSdArea",
            "super::register_sd::RegisterSdEntryKind",
            "super::register_sd::ErrorModeGuard",
            "super::register_sd::classify_z",
            "super::register_sd::inside_root",
            "super::register_sd::split_relative_path",
            "super::move_no_replace",
            "crate::io::move_no_replace",
            "crate::io::register_sd::RegisterSdRoot",
        ];
        let text = production_text(source);
        let mut out = Vec::new();
        for caps in Regex::new(r"\bfs\s*::\s*(\x7B[^\x7D]*\x7D|\*|\w+)")
            .unwrap()
            .captures_iter(&text)
        {
            for item in group_items(&caps[1]) {
                if !FS_ITEMS.contains(&item.as_str()) {
                    out.push(format!("fs::{item}"));
                }
            }
        }
        for caps in Regex::new(r"\bFile\s*::\s*(\w+)")
            .unwrap()
            .captures_iter(&text)
        {
            if &caps[1] != "open" {
                out.push(format!("File::{}", &caps[1]));
            }
        }
        for forbidden in [
            r"\bOpenOptions\b",
            r"\bWrite\b",
            r"\bwindows_sys\b",
            r"\buse\s+std\b[^;]*\bas\b",
            r"\b(remove_file|remove_dir\w*|rename|hard_link|soft_link|symlink\w*|set_permissions|set_len|set_modified|set_times)\b",
        ] {
            for found in Regex::new(forbidden).unwrap().find_iter(&text) {
                out.push(found.as_str().to_string());
            }
        }
        for path in crate_paths(&text) {
            if !CRATE_NAMES.contains(&path.as_str()) {
                out.push(path);
            }
        }
        out
    }

    #[test]
    fn test_req401_module_uses_only_allowed_fs_api() {
        // REQ-401 / IO-11-D3: この module は IO-09-D3 の許可の列に create_dir と try_exists を足した API と、
        // 許可した crate の中の名前だけを使う
        let source = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/io/register_sd_move.rs"),
        )
        .unwrap();
        assert_eq!(violations(&source), Vec::<String>::new());
        // 本体と test の境は file の末尾の `mod tests` だけ
        assert_eq!(
            source.find(TEST_ATTR),
            source.find(concat!("#[cfg(", "test)]\n", "mod tests {"))
        );

        let body = source.split(TEST_ATTR).next().unwrap();
        for mutant in [
            "fn m() { let _ = fs::remove_file(\"x\"); }",
            "fn m() { let _ = std::fs::remove_file(\"x\"); }",
            "fn m() { let _ = fs::rename(\"a\", \"b\"); }",
            "fn m() { let _ = std::fs::rename(\"a\", \"b\"); }",
            "fn m() { let _ = fs::copy(\"a\", \"b\"); }",
            "fn m() { let _ = fs::write(\"a\", b\"b\"); }",
            "fn m() { let _ = fs::hard_link(\"a\", \"b\"); }",
            "fn m() { let _ = File::create(\"x\"); }",
            "fn m() { let _ = std::fs::OpenOptions::new(); }",
            "use std::io::Write;",
            "use std::io::{Write as W};",
            "use std::fs as f;",
            "use std::{fs as f};",
            "use std::fs::{self as f};",
            "use std::fs::*;",
            "use super::pos_source_copy;",
            "use super::pos_source_copy::save_pos_source_copy;",
            "use crate::db::sales_repo;",
            "use crate::mnt::restore;",
            "use crate::biz::BizError;",
            "use super::*;",
            "use super::register_sd::{inside_root, read_register_sd_file};",
            "use super::register_sd::inside_root as f;",
            "use windows_sys::Win32::Storage::FileSystem::DeleteFileW;",
        ] {
            let mutated = format!("{body}\n{mutant}\n");
            assert!(!violations(&mutated).is_empty(), "red にならない: {mutant}");
        }
        let allowed = format!(
            "{body}\nfn m(a: &Path, b: &Path) -> std::io::Result<()> {{ crate::io::move_no_replace(a, b) }}\n"
        );
        assert_eq!(violations(&allowed), Vec::<String>::new());
    }
}
