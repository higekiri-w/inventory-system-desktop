//! IO-09: レジの SD の列挙と読取り（D-111）
//!
//! docs/function-design/29-io-daily-report-parser.md §29.7 に基づく実装。
//! SD に書かない（IO-09-D3）。この module が使う file system の API は許可の列だけで、
//! test `test_req401_module_uses_only_allowed_fs_api` が source を検査する。

use super::REGISTER_SD_FOLDER;
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSdRoot {
    /// `<drive>:\CASIO\SR500_550_4000` の実 path
    pub path: PathBuf,
    /// 表示用。ドライブ名（`G:` 等）
    pub volume_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSdLocatedFile {
    pub root: RegisterSdRoot,
    /// root からの相対 path（区切りは `\`）
    pub relative_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterSdArea {
    /// `XZ`
    Sales,
    /// `XZ_BKUP`
    Backup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterSdEntryKind {
    Z {
        series: u16,
        day: u8,
        suffix: char,
        backup_seq: Option<u16>,
    },
    Ej {
        backup_seq: Option<u16>,
    },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSdEntry {
    /// root からの相対 path（区切りは `\`。例 `XZ\2026\10\Z001_06 .CSV`）
    pub relative_path: String,
    pub area: RegisterSdArea,
    pub kind: RegisterSdEntryKind,
    pub date: Option<NaiveDate>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSdListing {
    pub entries: Vec<RegisterSdEntry>,
    pub backup_area_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSdFile {
    pub relative_path: String,
    pub bytes: Vec<u8>,
    pub shape: RegisterSdShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterSdShape {
    pub cp932_strict: bool,
    pub has_bom: bool,
    pub crlf_only: bool,
    pub ends_with_crlf: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterSdError {
    NotRegisterSd,
    MissingSalesArea,
    InvalidPath,
    TooLarge {
        relative_path: String,
    },
    /// relative_path は root からの相対 path。`check_selected_file_present` だけは選んだ path の文字列
    Io {
        relative_path: String,
        kind: std::io::ErrorKind,
    },
}

/// 29 §29.7.4 手順 1: file system を見る間だけ、その thread の error mode を
/// `SEM_FAILCRITICALERRORS` にする（空の読取り機の口で Windows の error の dialog を出さない）。
/// drop で元の値へ戻す（途中で返る経路を含む）。Windows 以外は何もしない。
struct ErrorModeGuard {
    #[cfg(windows)]
    previous: Option<u32>,
}

impl ErrorModeGuard {
    fn new() -> Self {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Diagnostics::Debug::{
                SetThreadErrorMode, SEM_FAILCRITICALERRORS, THREAD_ERROR_MODE,
            };
            let mut previous: THREAD_ERROR_MODE = 0;
            // SAFETY: previous は有効な書込み先。設定は呼んだ thread だけに効く
            let set = unsafe { SetThreadErrorMode(SEM_FAILCRITICALERRORS, &mut previous) };
            Self {
                previous: (set != 0).then_some(previous),
            }
        }
        #[cfg(not(windows))]
        {
            Self {}
        }
    }
}

#[cfg(windows)]
impl Drop for ErrorModeGuard {
    fn drop(&mut self) {
        use windows_sys::Win32::System::Diagnostics::Debug::SetThreadErrorMode;
        if let Some(previous) = self.previous {
            // SAFETY: 前の値へ戻すだけで、戻す前の値は受け取らない。drop の中で失敗を返す先は無い
            let restored = unsafe { SetThreadErrorMode(previous, std::ptr::null_mut()) };
            if restored == 0 {
                tracing::warn!("thread の error mode を元に戻せませんでした");
            }
        }
    }
}

/// 29 §29.7.4 手順 1・2（IO-09-D1）
pub fn find_register_sd_roots() -> Result<Vec<RegisterSdRoot>, RegisterSdError> {
    let _guard = ErrorModeGuard::new();
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};
        use windows_sys::Win32::System::WindowsProgramming::DRIVE_REMOVABLE;
        // SAFETY: 引数を取らない
        let mask = unsafe { GetLogicalDrives() };
        let mut roots = Vec::new();
        for index in 0..26u8 {
            if mask & (1u32 << index) == 0 {
                continue;
            }
            let letter = char::from(b'A' + index);
            let drive = format!("{letter}:\\");
            let wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
            // SAFETY: wide は NUL で終わる UTF-16 の列で、呼出しの間は生きている
            if unsafe { GetDriveTypeW(wide.as_ptr()) } != DRIVE_REMOVABLE {
                continue;
            }
            if let Some(path) = folder_under(Path::new(&drive)) {
                roots.push(RegisterSdRoot {
                    path,
                    volume_label: format!("{letter}:"),
                });
            }
        }
        Ok(roots)
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

/// 29 §29.7.4 手順 3（IO-09-D1）
pub fn resolve_register_sd_root(selected: &Path) -> Result<RegisterSdRoot, RegisterSdError> {
    let _guard = ErrorModeGuard::new();
    let [casio, model] = REGISTER_SD_FOLDER;
    let name_is = |path: &Path, expected: &str| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(expected))
    };
    let root = if name_is(selected, model)
        && selected
            .parent()
            .is_some_and(|parent| name_is(parent, casio))
        && selected.is_dir()
    {
        Some(selected.to_path_buf())
    } else if name_is(selected, casio) {
        child_dir(selected, model)
    } else {
        folder_under(selected)
    };
    let path = root.ok_or(RegisterSdError::NotRegisterSd)?;
    Ok(RegisterSdRoot {
        volume_label: volume_label(&path),
        path,
    })
}

/// 29 §29.7.4 手順 4: 手で選んだ file が root の下かを path の文字列だけで決める（純関数）
pub fn locate_in_register_sd_roots(
    roots: &[RegisterSdRoot],
    path: &Path,
) -> Option<RegisterSdLocatedFile> {
    let file_parts = absolute_parts(path.to_str()?)?;
    roots.iter().find_map(|root| {
        let root_parts = absolute_parts(root.path.to_str()?)?;
        if file_parts.len() <= root_parts.len()
            || !root_parts
                .iter()
                .zip(&file_parts)
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
        {
            return None;
        }
        Some(RegisterSdLocatedFile {
            root: root.clone(),
            relative_path: file_parts[root_parts.len()..].join("\\"),
        })
    })
}

/// 29 §29.7.4 手順 5: その時点で通常の file として在り、metadata を読めるか
pub fn check_selected_file_present(path: &Path) -> Result<(), RegisterSdError> {
    let _guard = ErrorModeGuard::new();
    let io_error = |kind| RegisterSdError::Io {
        relative_path: path.display().to_string(),
        kind,
    };
    let metadata = fs::metadata(path).map_err(|error| io_error(error.kind()))?;
    if metadata.is_file() {
        Ok(())
    } else {
        Err(io_error(std::io::ErrorKind::InvalidInput))
    }
}

/// 29 §29.7.5（IO-09-D2・D5）
pub fn list_register_sd_entries(
    root: &RegisterSdRoot,
    from: NaiveDate,
) -> Result<RegisterSdListing, RegisterSdError> {
    let _guard = ErrorModeGuard::new();
    let root_real = fs::canonicalize(&root.path).map_err(|error| io_error("", &error))?;
    let mut sales = None;
    let mut backup = None;
    for (name, _) in read_dir_entries(&root_real, "")? {
        if name.eq_ignore_ascii_case("XZ") {
            sales = Some(name);
        } else if name.eq_ignore_ascii_case("XZ_BKUP") {
            backup = Some(name);
        }
    }
    let sales = sales.ok_or(RegisterSdError::MissingSalesArea)?;

    let mut entries = Vec::new();
    for (area, area_name) in [
        (RegisterSdArea::Sales, Some(sales)),
        (RegisterSdArea::Backup, backup.clone()),
    ] {
        let Some(area_name) = area_name else { continue };
        let area_real = inside_root(&root_real, &root_real.join(&area_name), &area_name)?;
        if !is_dir(&area_real, &area_name)? {
            if area == RegisterSdArea::Sales {
                return Err(RegisterSdError::MissingSalesArea);
            }
            continue;
        }
        list_area(&root_real, &area_real, &area_name, area, from, &mut entries)?;
    }
    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(RegisterSdListing {
        entries,
        backup_area_present: backup.is_some(),
    })
}

/// 29 §29.7.6（IO-09-D3・D4・D5）
pub fn read_register_sd_file(
    root: &RegisterSdRoot,
    relative_path: &str,
    max_bytes: u64,
) -> Result<RegisterSdFile, RegisterSdError> {
    let _guard = ErrorModeGuard::new();
    let parts: Vec<&str> = relative_path.split('\\').collect();
    if parts.iter().any(|part| {
        part.is_empty()
            || *part == "."
            || *part == ".."
            || part.contains(':')
            || part.contains('/')
            || Path::new(part).has_root()
    }) {
        return Err(RegisterSdError::InvalidPath);
    }
    let root_real =
        fs::canonicalize(&root.path).map_err(|error| io_error(relative_path, &error))?;
    let joined = parts
        .iter()
        .fold(root_real.clone(), |path, part| path.join(part));
    let real = inside_root(&root_real, &joined, relative_path)?;
    let file = File::open(&real).map_err(|error| io_error(relative_path, &error))?;
    let bytes = read_limited(file, max_bytes, relative_path)?;
    let shape = measure_shape(&bytes);
    Ok(RegisterSdFile {
        relative_path: relative_path.to_string(),
        bytes,
        shape,
    })
}

/// 29 §29.7.6 手順 2: `max_bytes + 1` byte まで読み、超えたら全体を読まずに `TooLarge`
fn read_limited(
    reader: impl Read,
    max_bytes: u64,
    relative_path: &str,
) -> Result<Vec<u8>, RegisterSdError> {
    let mut bytes = Vec::new();
    reader
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| io_error(relative_path, &error))?;
    if bytes.len() as u64 > max_bytes {
        return Err(RegisterSdError::TooLarge {
            relative_path: relative_path.to_string(),
        });
    }
    Ok(bytes)
}

/// 29 §29.7.6 手順 3（IO-09-D4）: 測るだけで、直さない
fn measure_shape(bytes: &[u8]) -> RegisterSdShape {
    let decoded = encoding_rs::SHIFT_JIS.decode_without_bom_handling_and_without_replacement(bytes);
    let mut crlf_only = decoded
        .as_ref()
        .is_none_or(|text| !text.contains('\u{0085}'));
    for (index, byte) in bytes.iter().enumerate() {
        let lone_cr = *byte == b'\r' && bytes.get(index + 1) != Some(&b'\n');
        let lone_lf = *byte == b'\n' && (index == 0 || bytes[index - 1] != b'\r');
        if lone_cr || lone_lf {
            crlf_only = false;
        }
    }
    RegisterSdShape {
        cp932_strict: decoded.is_some(),
        has_bom: bytes.starts_with(&[0xEF, 0xBB, 0xBF]),
        crlf_only,
        ends_with_crlf: bytes.ends_with(b"\r\n"),
    }
}

fn list_area(
    root_real: &Path,
    area_real: &Path,
    area_name: &str,
    area: RegisterSdArea,
    from: NaiveDate,
    entries: &mut Vec<RegisterSdEntry>,
) -> Result<(), RegisterSdError> {
    for (name, entry) in read_dir_entries(area_real, area_name)? {
        let relative = format!("{area_name}\\{name}");
        if let Some(year) = parse_digits(&name, 4) {
            if year < from.year() {
                continue;
            }
            let year_real = inside_root(root_real, &entry.path(), &relative)?;
            if is_dir(&year_real, &relative)? {
                list_year(root_real, &year_real, &relative, area, year, from, entries)?;
                continue;
            }
        }
        let Some(size) = file_size(root_real, &entry.path(), &relative)? else {
            continue;
        };
        let (kind, date) = classify_ej(&name, area);
        // `XZ` 直下の EJ（伸び続ける file、SD-09）は from によらず返す
        if area == RegisterSdArea::Backup && date.is_some_and(|date| date < from) {
            continue;
        }
        entries.push(RegisterSdEntry {
            relative_path: relative,
            area,
            kind,
            date,
            size_bytes: size,
        });
    }
    Ok(())
}

fn list_year(
    root_real: &Path,
    year_real: &Path,
    year_relative: &str,
    area: RegisterSdArea,
    year: i32,
    from: NaiveDate,
    entries: &mut Vec<RegisterSdEntry>,
) -> Result<(), RegisterSdError> {
    for (month_name, month_entry) in read_dir_entries(year_real, year_relative)? {
        let Some(month) = parse_digits(&month_name, 2) else {
            continue;
        };
        let month = month as u32;
        if !(1..=12).contains(&month) || (year, month) < (from.year(), from.month()) {
            continue;
        }
        let month_relative = format!("{year_relative}\\{month_name}");
        let month_real = inside_root(root_real, &month_entry.path(), &month_relative)?;
        if !is_dir(&month_real, &month_relative)? {
            continue;
        }
        for (file_name, file_entry) in read_dir_entries(&month_real, &month_relative)? {
            let file_relative = format!("{month_relative}\\{file_name}");
            let Some(size) = file_size(root_real, &file_entry.path(), &file_relative)? else {
                continue;
            };
            let (kind, date) = classify_z(&file_name, area, year, month);
            if date.is_some_and(|date| date < from) {
                continue;
            }
            entries.push(RegisterSdEntry {
                relative_path: file_relative,
                area,
                kind,
                date,
                size_bytes: size,
            });
        }
    }
    Ok(())
}

fn classify_z(
    name: &str,
    area: RegisterSdArea,
    year: i32,
    month: u32,
) -> (RegisterSdEntryKind, Option<NaiveDate>) {
    static Z_NAME: OnceLock<Regex> = OnceLock::new();
    let re = Z_NAME.get_or_init(|| {
        Regex::new(r"(?i)^Z(\d{3})_(\d{2})([ A-Z])(_(\d{4}))?\.CSV$").expect("Z の名前の規則")
    });
    let Some(caps) = re.captures(name) else {
        return (RegisterSdEntryKind::Unknown, None);
    };
    let backup_seq = caps.get(5).and_then(|seq| seq.as_str().parse().ok());
    if backup_seq.is_some() != (area == RegisterSdArea::Backup) {
        return (RegisterSdEntryKind::Unknown, None);
    }
    let (Some(series), Some(day)) = (caps[1].parse().ok(), caps[2].parse::<u8>().ok()) else {
        return (RegisterSdEntryKind::Unknown, None);
    };
    let Some(date) = NaiveDate::from_ymd_opt(year, month, u32::from(day)) else {
        return (RegisterSdEntryKind::Unknown, None);
    };
    let suffix = caps[3].chars().next().unwrap_or(' ').to_ascii_uppercase();
    (
        RegisterSdEntryKind::Z {
            series,
            day,
            suffix,
            backup_seq,
        },
        Some(date),
    )
}

fn classify_ej(name: &str, area: RegisterSdArea) -> (RegisterSdEntryKind, Option<NaiveDate>) {
    static EJ_NAME: OnceLock<Regex> = OnceLock::new();
    let re = EJ_NAME.get_or_init(|| {
        Regex::new(r"(?i)^EJ(\d{2})(\d{2})(\d{2})(_(\d{4}))?\.TXT$").expect("EJ の名前の規則")
    });
    let Some(caps) = re.captures(name) else {
        return (RegisterSdEntryKind::Unknown, None);
    };
    let backup_seq = caps.get(5).and_then(|seq| seq.as_str().parse().ok());
    if backup_seq.is_some() != (area == RegisterSdArea::Backup) {
        return (RegisterSdEntryKind::Unknown, None);
    }
    let date = match (
        caps[1].parse::<i32>(),
        caps[2].parse::<u32>(),
        caps[3].parse::<u32>(),
    ) {
        (Ok(yy), Ok(mm), Ok(dd)) => NaiveDate::from_ymd_opt(2000 + yy, mm, dd),
        _ => None,
    };
    match date {
        Some(date) => (RegisterSdEntryKind::Ej { backup_seq }, Some(date)),
        None => (RegisterSdEntryKind::Unknown, None),
    }
}

/// 返す file の実体を確かめて size を返す。file でない（directory への link 等）は None
fn file_size(
    root_real: &Path,
    path: &Path,
    relative_path: &str,
) -> Result<Option<u64>, RegisterSdError> {
    let real = inside_root(root_real, path, relative_path)?;
    let metadata = fs::metadata(&real).map_err(|error| io_error(relative_path, &error))?;
    Ok(metadata.is_file().then_some(metadata.len()))
}

/// IO-09-D5: 実体の path に解き、解いた root の下でなければ `InvalidPath`
fn inside_root(
    root_real: &Path,
    path: &Path,
    relative_path: &str,
) -> Result<PathBuf, RegisterSdError> {
    let real = fs::canonicalize(path).map_err(|error| io_error(relative_path, &error))?;
    if real.starts_with(root_real) {
        Ok(real)
    } else {
        Err(RegisterSdError::InvalidPath)
    }
}

fn is_dir(real: &Path, relative_path: &str) -> Result<bool, RegisterSdError> {
    fs::metadata(real)
        .map(|metadata| metadata.is_dir())
        .map_err(|error| io_error(relative_path, &error))
}

/// 読めた名前と entry の列。途中の失敗は `Io`（途中までの一覧を返さない）
fn read_dir_entries(
    dir: &Path,
    relative_path: &str,
) -> Result<Vec<(String, fs::DirEntry)>, RegisterSdError> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| io_error(relative_path, &error))? {
        let entry = entry.map_err(|error| io_error(relative_path, &error))?;
        // UTF-16 にならない名前は規則に合わないので読まない
        if let Some(name) = entry.file_name().to_str() {
            entries.push((name.to_string(), entry));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

/// `CASIO\SR500_550_4000` を `base` の下に探す（大文字小文字を区別しない）
fn folder_under(base: &Path) -> Option<PathBuf> {
    let [casio, model] = REGISTER_SD_FOLDER;
    child_dir(&child_dir(base, casio)?, model)
}

fn child_dir(parent: &Path, name: &str) -> Option<PathBuf> {
    let exact = parent.join(name);
    if exact.is_dir() {
        return Some(exact);
    }
    fs::read_dir(parent).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        (entry.file_name().to_str()?.eq_ignore_ascii_case(name) && path.is_dir()).then_some(path)
    })
}

fn volume_label(path: &Path) -> String {
    match path.components().next() {
        Some(Component::Prefix(prefix)) => prefix.as_os_str().to_string_lossy().into_owned(),
        _ => path.display().to_string(),
    }
}

/// `\` と `/` で区切った絶対 path の要素の列（drive 文字の形と `/` で始まる形）
fn absolute_parts(text: &str) -> Option<Vec<&str>> {
    let bytes = text.as_bytes();
    let parts: Vec<&str> = if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/')
    {
        std::iter::once(&text[..2])
            .chain(text[3..].split(['\\', '/']))
            .collect()
    } else if matches!(bytes.first(), Some(b'\\' | b'/')) {
        text[1..].split(['\\', '/']).collect()
    } else {
        return None;
    };
    if parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return None;
    }
    Some(parts)
}

fn parse_digits(name: &str, len: usize) -> Option<i32> {
    if name.len() == len && name.bytes().all(|byte| byte.is_ascii_digit()) {
        name.parse().ok()
    } else {
        None
    }
}

fn io_error(relative_path: &str, error: &std::io::Error) -> RegisterSdError {
    RegisterSdError::Io {
        relative_path: relative_path.to_string(),
        kind: error.kind(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// 一時 directory に `CASIO\SR500_550_4000` を作り、その root を返す
    fn make_root(base: &Path) -> RegisterSdRoot {
        let path = REGISTER_SD_FOLDER
            .iter()
            .fold(base.to_path_buf(), |path, part| path.join(part));
        fs::create_dir_all(&path).unwrap();
        RegisterSdRoot {
            path,
            volume_label: "T:".to_string(),
        }
    }

    /// root の下に `\` 区切りの相対 path で合成の file を置く
    fn put(root: &RegisterSdRoot, relative: &str, bytes: &[u8]) {
        let path = relative
            .split('\\')
            .fold(root.path.clone(), |path, part| path.join(part));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn listed(
        listing: &RegisterSdListing,
    ) -> BTreeMap<String, (RegisterSdEntryKind, Option<NaiveDate>)> {
        listing
            .entries
            .iter()
            .map(|entry| (entry.relative_path.clone(), (entry.kind, entry.date)))
            .collect()
    }

    #[test]
    fn test_req401_resolve_accepts_drive_casio_and_model_folder() {
        // REQ-401 / IO-09-D1: drive 相当・CASIO・SR500_550_4000 のどれを選んでも root になる（大文字小文字を混ぜる）
        let dir = tempfile::tempdir().unwrap();
        let [casio, model] = REGISTER_SD_FOLDER;
        let path = dir
            .path()
            .join(casio.to_ascii_lowercase())
            .join(model.to_ascii_lowercase());
        fs::create_dir_all(&path).unwrap();
        for selected in [
            dir.path().to_path_buf(),
            dir.path().join(casio.to_ascii_lowercase()),
            path.clone(),
        ] {
            let root = resolve_register_sd_root(&selected).unwrap();
            assert_eq!(root.path, path, "{}", selected.display());
        }
    }

    #[test]
    fn test_req401_resolve_rejects_non_register_folder() {
        // REQ-401 / IO-09-D1: `CASIO\SR500_550_4000` の無い folder は NotRegisterSd
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("CASIO").join("OTHER")).unwrap();
        fs::create_dir_all(dir.path().join("SR500_550_4000")).unwrap();
        for selected in [
            dir.path().to_path_buf(),
            dir.path().join("CASIO"),
            dir.path().join("SR500_550_4000"),
            dir.path().join("missing"),
        ] {
            assert_eq!(
                resolve_register_sd_root(&selected),
                Err(RegisterSdError::NotRegisterSd),
                "{}",
                selected.display()
            );
        }
    }

    #[test]
    fn test_req401_find_roots_is_empty_off_windows() {
        // REQ-401 / IO-09-D1: Windows 以外では自動の発見は空の列（予備の経路だけ）
        if cfg!(not(windows)) {
            assert_eq!(find_register_sd_roots(), Ok(Vec::new()));
        }
    }

    #[test]
    fn test_req401_list_classifies_observed_names() {
        // REQ-401 / IO-09-D2: 観測した名前の場合（接尾字・連番・大文字小文字・EJ の置き場）を分類する
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        for name in [
            "XZ\\2026\\10\\Z001_06 .CSV",
            "XZ\\2026\\10\\Z001_06A.CSV",
            "XZ\\2026\\10\\Z001_06B.CSV",
            "XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV",
            "XZ\\EJ261006.TXT",
            "XZ_BKUP\\EJ261005_0001.TXT",
            "XZ\\ej261004.txt",
            "XZ\\Ej261003.TXT",
        ] {
            put(&root, name, b"x");
        }
        let listing = list_register_sd_entries(&root, date(2026, 10, 1)).unwrap();
        assert!(listing.backup_area_present);
        let z = |suffix, backup_seq| RegisterSdEntryKind::Z {
            series: 1,
            day: 6,
            suffix,
            backup_seq,
        };
        let expected: BTreeMap<String, (RegisterSdEntryKind, Option<NaiveDate>)> = [
            (
                "XZ\\2026\\10\\Z001_06 .CSV",
                z(' ', None),
                date(2026, 10, 6),
            ),
            (
                "XZ\\2026\\10\\Z001_06A.CSV",
                z('A', None),
                date(2026, 10, 6),
            ),
            (
                "XZ\\2026\\10\\Z001_06B.CSV",
                z('B', None),
                date(2026, 10, 6),
            ),
            (
                "XZ_BKUP\\2026\\10\\Z001_06 _0001.CSV",
                z(' ', Some(1)),
                date(2026, 10, 6),
            ),
            (
                "XZ\\EJ261006.TXT",
                RegisterSdEntryKind::Ej { backup_seq: None },
                date(2026, 10, 6),
            ),
            (
                "XZ_BKUP\\EJ261005_0001.TXT",
                RegisterSdEntryKind::Ej {
                    backup_seq: Some(1),
                },
                date(2026, 10, 5),
            ),
            (
                "XZ\\ej261004.txt",
                RegisterSdEntryKind::Ej { backup_seq: None },
                date(2026, 10, 4),
            ),
            (
                "XZ\\Ej261003.TXT",
                RegisterSdEntryKind::Ej { backup_seq: None },
                date(2026, 10, 3),
            ),
        ]
        .into_iter()
        .map(|(path, kind, date)| (path.to_string(), (kind, Some(date))))
        .collect();
        assert_eq!(listed(&listing), expected);
        let areas: Vec<_> = listing
            .entries
            .iter()
            .map(|entry| (entry.relative_path.as_str(), entry.area))
            .collect();
        assert!(areas.contains(&("XZ_BKUP\\EJ261005_0001.TXT", RegisterSdArea::Backup)));
        assert!(areas.contains(&("XZ\\EJ261006.TXT", RegisterSdArea::Sales)));
    }

    #[test]
    #[cfg(unix)]
    fn test_req401_list_returns_unknown_without_reading() {
        // REQ-401 / IO-09-D2: 規則に合わない名前は Unknown で返し、中身を読まない
        // （読めない権限にした file でも列挙が成功する = 開いていない）
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let unknown = [
            "XZ\\2026\\02\\NOTE.DAT",
            "XZ\\2026\\02\\Z001_06 _0001.CSV",
            "XZ_BKUP\\2026\\02\\Z001_06 .CSV",
            "XZ\\2026\\02\\Z001_31 .CSV",
        ];
        for name in unknown {
            put(&root, name, b"secret");
            let path = name
                .split('\\')
                .fold(root.path.clone(), |p, part| p.join(part));
            fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        }
        let listing = list_register_sd_entries(&root, date(2026, 2, 1)).unwrap();
        let got = listed(&listing);
        for name in unknown {
            assert_eq!(
                got.get(name),
                Some(&(RegisterSdEntryKind::Unknown, None)),
                "{name}"
            );
        }
        assert_eq!(got.len(), unknown.len());
        assert_eq!(
            listing
                .entries
                .iter()
                .find(|entry| entry.relative_path == unknown[0])
                .unwrap()
                .size_bytes,
            6
        );
    }

    #[test]
    fn test_req401_list_keeps_other_series() {
        // REQ-401 / IO-09-D2: 系列番号は決め打ちにしない
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ\\2026\\10\\Z006_06 .CSV", b"x");
        put(&root, "XZ\\2026\\10\\Z123_06 .CSV", b"x");
        let listing = list_register_sd_entries(&root, date(2026, 10, 1)).unwrap();
        let series: Vec<u16> = listing
            .entries
            .iter()
            .filter_map(|entry| match entry.kind {
                RegisterSdEntryKind::Z { series, .. } => Some(series),
                _ => None,
            })
            .collect();
        assert_eq!(series, vec![6, 123]);
        assert!(!listing.backup_area_present);
    }

    #[test]
    fn test_req401_list_filters_by_from_month_and_date() {
        // REQ-401 / IO-09-D2: 窓の外の (年, 月) の folder に入らず、from より前の Z・EJ を返さない（XZ 直下の EJ を除く）
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        for name in [
            "XZ\\2026\\09\\Z001_30 .CSV",
            "XZ\\2025\\12\\Z001_30 .CSV",
            "XZ\\2026\\10\\Z001_02 .CSV",
            "XZ\\2026\\10\\Z001_03 .CSV",
            "XZ\\EJ250101.TXT",
            "XZ_BKUP\\EJ261002_0001.TXT",
            "XZ_BKUP\\EJ261003_0001.TXT",
        ] {
            put(&root, name, b"x");
        }
        let listing = list_register_sd_entries(&root, date(2026, 10, 3)).unwrap();
        let names: Vec<&str> = listing
            .entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "XZ\\2026\\10\\Z001_03 .CSV",
                "XZ\\EJ250101.TXT",
                "XZ_BKUP\\EJ261003_0001.TXT"
            ]
        );
    }

    #[test]
    fn test_req401_list_area_presence() {
        // REQ-401 / IO-09-D2: XZ_BKUP は無くてよく、XZ が無ければ MissingSalesArea
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        assert_eq!(
            list_register_sd_entries(&root, date(2026, 10, 1)),
            Err(RegisterSdError::MissingSalesArea)
        );
        put(&root, "XZ_BKUP\\EJ261005_0001.TXT", b"x");
        assert_eq!(
            list_register_sd_entries(&root, date(2026, 10, 1)),
            Err(RegisterSdError::MissingSalesArea)
        );
        let only_sales = make_root(&dir.path().join("second"));
        put(&only_sales, "XZ\\EJ261006.TXT", b"x");
        let listing = list_register_sd_entries(&only_sales, date(2026, 10, 1)).unwrap();
        assert!(!listing.backup_area_present);
        assert_eq!(listing.entries.len(), 1);
    }

    fn snapshot_tree(dir: &Path) -> BTreeMap<PathBuf, (u64, String, std::time::SystemTime)> {
        let mut out = BTreeMap::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            for entry in fs::read_dir(&current).unwrap() {
                let path = entry.unwrap().path();
                let metadata = fs::metadata(&path).unwrap();
                if metadata.is_dir() {
                    stack.push(path.clone());
                    out.insert(path, (0, String::new(), metadata.modified().unwrap()));
                } else {
                    let hash = format!(
                        "{:x}",
                        <sha2::Sha256 as sha2::Digest>::digest(fs::read(&path).unwrap())
                    );
                    out.insert(path, (metadata.len(), hash, metadata.modified().unwrap()));
                }
            }
        }
        out
    }

    #[test]
    fn test_req401_scan_leaves_tree_unchanged() {
        // REQ-401 / IO-09-D3: list と read の前後で tree の名前・size・hash・更新時刻が変わらない
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ\\2026\\10\\Z001_06 .CSV", b"a\r\n");
        put(&root, "XZ_BKUP\\2026\\10\\Z001_05 _0001.CSV", b"b\r\n");
        put(&root, "XZ\\EJ261006.TXT", b"c\r\n");
        let before = snapshot_tree(dir.path());
        let listing = list_register_sd_entries(&root, date(2026, 10, 1)).unwrap();
        for entry in &listing.entries {
            read_register_sd_file(&root, &entry.relative_path, 1024).unwrap();
        }
        assert_eq!(snapshot_tree(dir.path()), before);
    }

    #[test]
    fn test_req401_read_rejects_escaping_paths() {
        // REQ-401 / IO-09-D3: root の外を指す相対 path は開かない
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ\\a.CSV", b"x");
        for bad in [
            "..\\x",
            "C:\\x",
            "\\x",
            "XZ\\\\a.CSV",
            "XZ\\.\\a.CSV",
            "XZ/../../x",
            "",
        ] {
            assert_eq!(
                read_register_sd_file(&root, bad, 1024),
                Err(RegisterSdError::InvalidPath),
                "{bad:?}"
            );
        }
        assert!(read_register_sd_file(&root, "XZ\\a.CSV", 1024).is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_req401_list_and_read_reject_links_outside_root() {
        // REQ-401 / IO-09-D5: root の下の link が外を指すなら列挙・読取りとも InvalidPath（開かない）
        use std::os::unix::fs::symlink;
        let max_bytes = 4;
        let big = b"outside-bytes-larger-than-max";
        let rel = "XZ\\2026\\10\\Z001_06 .CSV";

        // (1)(2) XZ が root の外への directory の link
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("outside");
        fs::create_dir_all(outside.join("XZ").join("2026").join("10")).unwrap();
        fs::write(
            outside
                .join("XZ")
                .join("2026")
                .join("10")
                .join("Z001_06 .CSV"),
            big,
        )
        .unwrap();
        let root = make_root(&dir.path().join("sd"));
        symlink(outside.join("XZ"), root.path.join("XZ")).unwrap();
        assert_eq!(
            list_register_sd_entries(&root, date(2026, 10, 1)),
            Err(RegisterSdError::InvalidPath)
        );
        assert_eq!(
            read_register_sd_file(&root, rel, max_bytes),
            Err(RegisterSdError::InvalidPath)
        );

        // (3) XZ は実体、その下の file だけが外への link
        let dir3 = tempfile::tempdir().unwrap();
        let outside_file = dir3.path().join("outside.CSV");
        fs::write(&outside_file, big).unwrap();
        let root3 = make_root(&dir3.path().join("sd"));
        let month = root3.path.join("XZ").join("2026").join("10");
        fs::create_dir_all(&month).unwrap();
        symlink(&outside_file, month.join("Z001_06 .CSV")).unwrap();
        assert_eq!(
            list_register_sd_entries(&root3, date(2026, 10, 1)),
            Err(RegisterSdError::InvalidPath)
        );
        assert_eq!(
            read_register_sd_file(&root3, rel, max_bytes),
            Err(RegisterSdError::InvalidPath)
        );

        // (4) root の下を指す link と、root より上に link を含む root は通す
        let dir4 = tempfile::tempdir().unwrap();
        let real = dir4.path().join("real");
        let root4 = make_root(&real);
        put(&root4, "XZ\\2026\\10\\Z001_06A.CSV", b"inner\r\n");
        symlink(
            root4
                .path
                .join("XZ")
                .join("2026")
                .join("10")
                .join("Z001_06A.CSV"),
            root4
                .path
                .join("XZ")
                .join("2026")
                .join("10")
                .join("Z001_06 .CSV"),
        )
        .unwrap();
        symlink(&real, dir4.path().join("alias")).unwrap();
        let alias_root = RegisterSdRoot {
            path: REGISTER_SD_FOLDER
                .iter()
                .fold(dir4.path().join("alias"), |path, part| path.join(part)),
            volume_label: "T:".to_string(),
        };
        let listing = list_register_sd_entries(&alias_root, date(2026, 10, 1)).unwrap();
        assert_eq!(listing.entries.len(), 2);
        let file = read_register_sd_file(&alias_root, rel, 1024).unwrap();
        assert_eq!(file.bytes, b"inner\r\n");
    }

    /// 読んだ byte の数を数える reader
    struct CountingReader {
        remaining: usize,
        read: usize,
    }

    impl Read for CountingReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = buf.len().min(self.remaining);
            buf[..n].fill(b'a');
            self.remaining -= n;
            self.read += n;
            Ok(n)
        }
    }

    #[test]
    fn test_req401_read_stops_at_limit() {
        // REQ-401 / IO-09-D3 §29.7.6 手順 2: max_bytes + 1 byte までしか読まず TooLarge
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        put(&root, "XZ\\big.CSV", &[b'a'; 11]);
        put(&root, "XZ\\fit.CSV", &[b'a'; 10]);
        assert_eq!(
            read_register_sd_file(&root, "XZ\\big.CSV", 10),
            Err(RegisterSdError::TooLarge {
                relative_path: "XZ\\big.CSV".to_string()
            })
        );
        assert_eq!(
            read_register_sd_file(&root, "XZ\\fit.CSV", 10)
                .unwrap()
                .bytes
                .len(),
            10
        );

        let max_bytes = 1000;
        let mut reader = CountingReader {
            remaining: max_bytes * 4,
            read: 0,
        };
        let result = read_limited(&mut reader, max_bytes as u64, "x");
        assert!(matches!(result, Err(RegisterSdError::TooLarge { .. })));
        assert!(reader.read <= max_bytes + 1, "読んだ byte: {}", reader.read);
    }

    #[test]
    fn test_req401_read_measures_shape() {
        // REQ-401 / IO-09-D4: 形を測るだけで、bytes は元のまま返す
        let dir = tempfile::tempdir().unwrap();
        let root = make_root(dir.path());
        let cases: [(&str, &[u8], RegisterSdShape); 6] = [
            (
                "ok",
                b"a,b\r\nc\r\n",
                RegisterSdShape {
                    cp932_strict: true,
                    has_bom: false,
                    crlf_only: true,
                    ends_with_crlf: true,
                },
            ),
            (
                "bom",
                b"\xEF\xBB\xBFa\r\n",
                RegisterSdShape {
                    cp932_strict: false,
                    has_bom: true,
                    crlf_only: true,
                    ends_with_crlf: true,
                },
            ),
            (
                "lf",
                b"a\nb\r\n",
                RegisterSdShape {
                    cp932_strict: true,
                    has_bom: false,
                    crlf_only: false,
                    ends_with_crlf: true,
                },
            ),
            (
                "cr",
                b"a\rb\r\n",
                RegisterSdShape {
                    cp932_strict: true,
                    has_bom: false,
                    crlf_only: false,
                    ends_with_crlf: true,
                },
            ),
            (
                "noeol",
                b"a\r\nb",
                RegisterSdShape {
                    cp932_strict: true,
                    has_bom: false,
                    crlf_only: true,
                    ends_with_crlf: false,
                },
            ),
            (
                "notcp932",
                b"\x81\x20\r\n",
                RegisterSdShape {
                    cp932_strict: false,
                    has_bom: false,
                    crlf_only: true,
                    ends_with_crlf: true,
                },
            ),
        ];
        for (name, bytes, shape) in cases {
            let rel = format!("XZ\\{name}.CSV");
            put(&root, &rel, bytes);
            let file = read_register_sd_file(&root, &rel, 1024).unwrap();
            assert_eq!(file.shape, shape, "{name}");
            assert_eq!(file.bytes, bytes, "{name}");
        }
    }

    #[test]
    fn test_req401_check_selected_file_present_rejects_missing_and_dirs() {
        // REQ-401 / IO-09 §29.7.4 手順 5: 在る通常の file だけ Ok
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.CSV");
        fs::write(&file, b"x").unwrap();
        assert_eq!(check_selected_file_present(&file), Ok(()));
        let gone = dir.path().join("gone.CSV");
        fs::write(&gone, b"x").unwrap();
        fs::remove_file(&gone).unwrap();
        for (path, kind) in [
            (gone, std::io::ErrorKind::NotFound),
            (
                PathBuf::from("/nonexistent-drive/a.CSV"),
                std::io::ErrorKind::NotFound,
            ),
            (dir.path().to_path_buf(), std::io::ErrorKind::InvalidInput),
        ] {
            assert_eq!(
                check_selected_file_present(&path),
                Err(RegisterSdError::Io {
                    relative_path: path.display().to_string(),
                    kind
                })
            );
        }
    }

    #[test]
    fn test_req401_locate_in_roots_maps_paths() {
        // REQ-401 / IO-09 §29.7.4 手順 4: OS に依らない文字列の判定（`\` と `/`、大文字小文字を区別しない）
        let [casio, model] = REGISTER_SD_FOLDER;
        let root = |path: String| RegisterSdRoot {
            path: PathBuf::from(path),
            volume_label: "E:".to_string(),
        };
        let windows_roots = [root(format!("E:\\{casio}\\{model}"))];
        let unix_roots = [root(format!("/tmp/t/{casio}/{model}"))];
        let lower = (casio.to_ascii_lowercase(), model.to_ascii_lowercase());
        let expected = "XZ\\2026\\10\\Z001_06 .CSV";
        for (roots, path) in [
            (
                &windows_roots,
                format!("e:\\{}\\{}\\XZ\\2026\\10\\Z001_06 .CSV", lower.0, lower.1),
            ),
            (
                &windows_roots,
                format!("E:/{casio}/{model}/XZ/2026/10/Z001_06 .CSV"),
            ),
            (
                &unix_roots,
                format!("/tmp/t/{}/{model}/XZ/2026/10/Z001_06 .CSV", lower.0),
            ),
        ] {
            let located = locate_in_register_sd_roots(roots, Path::new(&path)).unwrap();
            assert_eq!(located.relative_path, expected, "{path}");
            assert_eq!(located.root, roots[0]);
        }
        for (roots, path) in [
            (&windows_roots, format!("E:\\{casio}\\{model}X\\a.CSV")),
            (&windows_roots, "C:\\EcrDatas\\a.CSV".to_string()),
            (&windows_roots, "XZ\\a.CSV".to_string()),
            (
                &windows_roots,
                format!("E:\\{casio}\\{model}\\..\\x\\a.CSV"),
            ),
            (&windows_roots, format!("E:\\{casio}\\{model}")),
            (&unix_roots, format!("/tmp/t/{casio}/{model}X/a.CSV")),
            (&unix_roots, "/tmp/other/a.CSV".to_string()),
            (
                &unix_roots,
                format!("/tmp/t/{casio}/{model}/XZ/../../x/a.CSV"),
            ),
        ] {
            assert_eq!(
                locate_in_register_sd_roots(roots, Path::new(&path)),
                None,
                "{path}"
            );
        }
        assert_eq!(
            locate_in_register_sd_roots(&[], Path::new(&format!("E:\\{casio}\\{model}\\a.CSV"))),
            None
        );
    }

    // ------------------------------------------------------------------
    // IO-09-D3 の source の検査と、folder 名の定数の 1 か所の検査
    // ------------------------------------------------------------------

    /// 本体と test の境の属性（architecture_test の行の追跡に掛からないよう分けて書く）
    const TEST_ATTR: &str = concat!("#[cfg(", "test)]");

    /// `cfg(test)` の属性 より前の本体から、行と block の comment を外した文字列。
    /// `cfg(test)` の属性つきの `mod <名>;` の宣言は本体の途中にも置かれる（`io/mod.rs` の
    /// `l3_fixtures`）ので、境とみなさず外す（その file は `test_only_paths` が扱う）
    fn production_text(source: &str) -> String {
        let declarations = Regex::new(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*;").unwrap();
        let source = declarations.replace_all(source, "");
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

    /// IO-09-D3 の許可の列の外の出現を返す（空なら green）
    fn fs_api_violations(source: &str) -> Vec<String> {
        const FS_ITEMS: [&str; 9] = [
            "self",
            "File",
            "ReadDir",
            "DirEntry",
            "Metadata",
            "FileType",
            "read_dir",
            "metadata",
            "canonicalize",
        ];
        const WINDOWS_ITEMS: [&str; 6] = [
            "GetLogicalDrives",
            "GetDriveTypeW",
            "DRIVE_REMOVABLE",
            "SetThreadErrorMode",
            "SEM_FAILCRITICALERRORS",
            "THREAD_ERROR_MODE",
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
            r"\buse\s+std\b[^;]*\bas\b",
        ] {
            for found in Regex::new(forbidden).unwrap().find_iter(&text) {
                out.push(found.as_str().to_string());
            }
        }
        for caps in Regex::new(r"\.\s*(set_len|set_modified|set_times|set_permissions)\s*\(")
            .unwrap()
            .captures_iter(&text)
        {
            out.push(caps[1].to_string());
        }
        for found in Regex::new(r"\bcrate\s*::").unwrap().find_iter(&text) {
            out.push(found.as_str().to_string());
        }
        for caps in Regex::new(r"\bsuper\s*::\s*(\x7B[^\x7D]*\x7D|\*|\w+)")
            .unwrap()
            .captures_iter(&text)
        {
            for item in group_items(&caps[1]) {
                if item != "REGISTER_SD_FOLDER" {
                    out.push(format!("super::{item}"));
                }
            }
        }
        for caps in Regex::new(r"\bwindows_sys\s*::([^;]*)")
            .unwrap()
            .captures_iter(&text)
        {
            let tree = &caps[1];
            let leaves = match tree.find('\u{7B}') {
                Some(start) => group_items(&tree[start..]),
                None => vec![tree.rsplit("::").next().unwrap_or("").trim().to_string()],
            };
            for leaf in leaves {
                if !WINDOWS_ITEMS.contains(&leaf.as_str()) {
                    out.push(format!("windows_sys::{leaf}"));
                }
            }
        }
        out
    }

    fn source_of(relative: &str) -> String {
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap()
    }

    #[test]
    fn test_req401_module_uses_only_allowed_fs_api() {
        // REQ-401 / IO-09-D3: この module は許可の列の file system の API だけを使う
        let source = source_of("src/io/register_sd.rs");
        assert_eq!(fs_api_violations(&source), Vec::<String>::new());
        // 本体と test の境は file の末尾の `mod tests` だけ（本体の途中に cfg(test) を置かない）
        assert_eq!(
            source.find(TEST_ATTR),
            source.find(concat!("#[cfg(", "test)]\n", "mod tests {"))
        );

        let body = source.split(TEST_ATTR).next().unwrap();
        for mutant in [
            "fn m() { let _ = File::create(\"x\"); }",
            "fn m() { let _ = fs::copy(\"a\", \"b\"); }",
            "fn m() { let _ = std::fs::OpenOptions::new(); }",
            "use std::io::Write;",
            "fn m(file: &File) { let _ = file.set_modified(std::time::SystemTime::now()); }",
            "use std::{fs as f};",
            "use std::fs::{self as f};",
            "use std::fs as f;",
            "use std::fs::File as F;",
            "use std::io::{Write as W};",
            "use std::fs::*;",
            "use super::pos_source_copy;",
            "use super::image_manager;",
            "use crate::db::sales_repo;",
            "use super::{REGISTER_SD_FOLDER, pos_source_copy};",
            "use windows_sys::Win32::Storage::FileSystem::DeleteFileW;",
            "use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, CreateFileW};",
        ] {
            let mutated = format!("{body}\n{mutant}\n");
            assert!(
                !fs_api_violations(&mutated).is_empty(),
                "red にならない: {mutant}"
            );
        }
        let allowed =
            format!("{body}\nfn m() -> [&'static str; 2] {{ super::REGISTER_SD_FOLDER }}\n");
        assert_eq!(fs_api_violations(&allowed), Vec::<String>::new());
    }

    /// `cfg(test)` の属性つきの `mod <名>;` で読み込まれる file と directory
    fn test_only_paths(files: &[(PathBuf, String)]) -> Vec<PathBuf> {
        let re = Regex::new(r"#\[cfg\(test\)\]\s*mod\s+(\w+)\s*;").unwrap();
        let mut out = Vec::new();
        for (path, text) in files {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("");
            let dir = path.parent().unwrap().to_path_buf();
            let base = if matches!(stem, "mod" | "lib" | "main") {
                dir
            } else {
                dir.join(stem)
            };
            for caps in re.captures_iter(text) {
                out.push(base.join(format!("{}.rs", &caps[1])));
                out.push(base.join(&caps[1]));
            }
        }
        out
    }

    /// 本体（`cfg(test)` の属性 より前）に folder 名の literal を持つ file と、その数
    fn folder_name_definitions(files: &[(PathBuf, String)]) -> Vec<(PathBuf, usize)> {
        let skipped = test_only_paths(files);
        files
            .iter()
            .filter(|(path, _)| !skipped.iter().any(|skip| path.starts_with(skip)))
            .filter_map(|(path, text)| {
                let count = production_text(text).matches("SR500_550_4000").count();
                (count > 0).then(|| (path.clone(), count))
            })
            .collect()
    }

    fn all_sources(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                all_sources(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = fs::read_to_string(&path).unwrap();
                out.push((path, text));
            }
        }
    }

    #[test]
    fn test_req401_register_sd_folder_name_has_one_definition() {
        // REQ-401 / 29 §29.7.4 手順 1・71 MNT-01-D10: folder 名は io/mod.rs の 1 つの定数だけが持つ
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        all_sources(&src, &mut files);
        assert_eq!(
            folder_name_definitions(&files),
            vec![(src.join("io").join("mod.rs"), 1)]
        );

        // mutant: 別の file の本体に literal を足すと red
        let mut mutated = files.clone();
        mutated.push((
            src.join("mnt").join("offsite.rs"),
            "const X: &str = \"SR500_550_4000\";".into(),
        ));
        assert_ne!(
            folder_name_definitions(&mutated),
            vec![(src.join("io").join("mod.rs"), 1)]
        );
        // `cfg(test)` の属性つきの `mod <名>;` の file の literal は数えない
        let mut test_file = files.clone();
        test_file.push((
            src.join("biz")
                .join("daily_report_import_service")
                .join("tests.rs"),
            "fn tree() { let _ = \"SR500_550_4000\"; }".into(),
        ));
        assert_eq!(
            folder_name_definitions(&test_file),
            vec![(src.join("io").join("mod.rs"), 1)]
        );
    }
}
