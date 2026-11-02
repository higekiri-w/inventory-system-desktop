//! MNT-01: バックアップ・リストア
//!
//! SQLiteデータベースのバックアップ作成・復元・自動チェックを提供する。
//!
//! docs/function-design/71-mnt-backup.md に基づく実装。

use super::restore::{RestoreFileOps, StdRestoreFileOps};
use crate::db::{self, DbConnection, DbError, NewOperationLog};
use crate::io::REGISTER_SD_FOLDER;
use std::path::{Path, PathBuf};

pub(crate) use crate::db::open_existing_database;

#[cfg(test)]
thread_local! {
    static SETTING_READ_FAILURE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    static METADATA_FAILURE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    /// 手順の直後に差し込む test の注入（71 §71.8 の注入境界と同じく本番の関数を通す）
    static STEP_HOOK: std::cell::RefCell<Option<StepHook>> = const { std::cell::RefCell::new(None) };
}

/// `(手順の名前, その手順の path)` を受け、`Err` でその手順を失敗にする
#[cfg(test)]
type StepHook = Box<dyn FnMut(&'static str, &Path) -> Result<(), String>>;

#[cfg(test)]
pub(crate) struct StepHookGuard;

#[cfg(test)]
impl Drop for StepHookGuard {
    fn drop(&mut self) {
        STEP_HOOK.with(|hook| *hook.borrow_mut() = None);
    }
}

#[cfg(test)]
pub(crate) fn set_step_hook(
    hook: impl FnMut(&'static str, &Path) -> Result<(), String> + 'static,
) -> StepHookGuard {
    STEP_HOOK.with(|slot| *slot.borrow_mut() = Some(Box::new(hook)));
    StepHookGuard
}

#[cfg(test)]
fn run_step_hook(step: &'static str, path: &Path) -> Result<(), String> {
    STEP_HOOK.with(|hook| match hook.borrow_mut().as_mut() {
        Some(hook) => hook(step, path),
        None => Ok(()),
    })
}

#[cfg(test)]
pub(crate) struct SettingReadFailureGuard {
    previous: Option<String>,
}

#[cfg(test)]
impl Drop for SettingReadFailureGuard {
    fn drop(&mut self) {
        SETTING_READ_FAILURE.with(|failure| {
            *failure.borrow_mut() = self.previous.take();
        });
    }
}

#[cfg(test)]
pub(crate) fn fail_setting_read(key: &str) -> SettingReadFailureGuard {
    let previous = SETTING_READ_FAILURE.with(|failure| failure.replace(Some(key.to_string())));
    SettingReadFailureGuard { previous }
}

#[cfg(test)]
struct MetadataFailureGuard(Option<String>);
#[cfg(test)]
impl Drop for MetadataFailureGuard {
    fn drop(&mut self) {
        METADATA_FAILURE.with(|f| *f.borrow_mut() = self.0.take());
    }
}
#[cfg(test)]
fn fail_metadata(path: &Path) -> MetadataFailureGuard {
    MetadataFailureGuard(
        METADATA_FAILURE.with(|f| f.replace(Some(path.to_string_lossy().into_owned()))),
    )
}
#[cfg(test)]
fn fail_any_metadata() -> MetadataFailureGuard {
    MetadataFailureGuard(METADATA_FAILURE.with(|f| f.replace(Some(String::new()))))
}

fn get_backup_setting(conn: &DbConnection, key: &str) -> Result<Option<String>, DbError> {
    #[cfg(test)]
    {
        SETTING_READ_FAILURE.with(|failure| {
            if failure.borrow().as_deref() == Some(key) {
                return Err(DbError::QueryFailed(format!(
                    "injected setting read failure: {key}"
                )));
            }
            db::system_repo::get_setting(conn, key)
        })
    }

    #[cfg(not(test))]
    {
        db::system_repo::get_setting(conn, key)
    }
}

fn read_metadata(path: &Path) -> std::io::Result<std::fs::Metadata> {
    #[cfg(test)]
    if METADATA_FAILURE.with(|failure| {
        failure
            .borrow()
            .as_deref()
            .is_some_and(|target| target.is_empty() || target == path.to_string_lossy())
    }) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "injected metadata failure",
        ));
    }
    std::fs::metadata(path)
}

pub use super::restore::RestoreError;

/// バックアップ snapshot へ crash-consistent に置換する（MNT-01-D1/D4/D5）。
pub fn restore_backup(
    current_conn: DbConnection,
    backup_path: &Path,
    db_path: &Path,
) -> Result<DbConnection, RestoreError> {
    super::restore::restore_backup(current_conn, backup_path, db_path)
}

pub(crate) fn reconcile_restore(db_path: &Path) -> Result<(), RestoreError> {
    super::restore::reconcile_restore(db_path).map(|_| ())
}

pub(crate) fn complete_reconciled_restore(
    conn: &DbConnection,
    db_path: &Path,
) -> Result<(), RestoreError> {
    super::restore::complete_reconciled_restore(conn, db_path)
}

// ---------------------------------------------------------------------------
// 定数
// ---------------------------------------------------------------------------

const BACKUP_PREFIX: &str = "inventory_backup_";
const BACKUP_EXT: &str = ".db";
/// 作業名の接尾辞（MNT-01-D7。`.db` で終わらないので正式名の規約に掛からない）
const PARTIAL_SUFFIX: &str = ".partial";
/// デフォルトのバックアップ保持日数（設定行が存在しない場合）
const DEFAULT_RETENTION_DAYS: u32 = 3;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// バックアップ作成結果
///
/// 71-mnt-backup.md §71.3
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct BackupResult {
    pub file_path: String,
    pub file_name: String,
    pub size_bytes: u64,
}

/// バックアップファイル情報（一覧表示用）
///
/// 71-mnt-backup.md §71.3
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct BackupInfo {
    pub file_name: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

/// 控えの確かめの結果（71 §71.12、MNT-01-D9）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct BackupInspection {
    pub file_name: String,
    /// §71.3 の規約に合えば `YYYY-MM-DD HH:MM:SS`
    pub created_at: Option<String>,
    pub size_bytes: u64,
    pub schema_version: i64,
    pub app_max_version: i64,
    /// `schema_version > app_max_version`
    pub newer_than_app: bool,
    pub quick_check_ok: bool,
    /// 版がアプリ以下で quick_check が ok のときだけ
    pub product_count: Option<i64>,
    /// 同上。`operation_logs.created_at` の最大
    pub last_operation_at: Option<String>,
}

/// 書く先を固定した path（71 §71.4 の「書く先の固定」。wire に出さない）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedPath {
    /// 正規化した元の path
    pub normalized: PathBuf,
    /// 判定した volume に固定した実体の path（書く・消すのはこちら）
    pub actual: PathBuf,
}

/// backup の保存先に書けない理由（71 §71.4、43 §43.6）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupDirError {
    /// 実体の path を得られない（Mount Manager の外の drive・network 等）
    Unpinnable,
    /// 保存先かその祖先の直下にレジの SD の folder がある
    RegisterSd { sd_folder: PathBuf },
}

impl std::fmt::Display for BackupDirError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unpinnable => write!(formatter, "保存先を固定できません"),
            Self::RegisterSd { sd_folder } => {
                write!(formatter, "レジの SD の上です ({})", sd_folder.display())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// プライベートヘルパー
// ---------------------------------------------------------------------------

/// 規約名 `inventory_backup_YYYYMMDD_HHMMSS.db` の日付・時刻の部分（71 §71.3 の完全一致）
///
/// stem は 15 bytes で `_` が 9 文字目、ほかの 14 文字は ASCII の数字だけ。
/// chrono の解析は空白・符号を受ける（`12 100`・`+202 1 1`）ので、解析の前に数字で限る。
/// 非 timestamp suffix（`_manual` 等）・多バイトの名前もここで落ちる（slice より前に見る）。
fn split_backup_stamp(filename: &str) -> Option<(&str, &str)> {
    let stem = filename.strip_prefix(BACKUP_PREFIX)?;
    let stem = stem.strip_suffix(BACKUP_EXT)?;
    let bytes = stem.as_bytes();
    let conforming = bytes.len() == 15
        && bytes.iter().enumerate().all(|(index, byte)| {
            if index == 8 {
                *byte == b'_'
            } else {
                byte.is_ascii_digit()
            }
        });
    if !conforming {
        return None;
    }
    Some((&stem[..8], &stem[9..]))
}

/// ファイル名からYYYYMMDD部分を抽出してNaiveDateに変換（保持日数の掃除の対象、§71.5 ステップ2a-2d）
fn extract_date_from_backup(filename: &str) -> Option<chrono::NaiveDate> {
    let (date_part, time_part) = split_backup_stamp(filename)?;
    chrono::NaiveTime::parse_from_str(time_part, "%H%M%S").ok()?;
    chrono::NaiveDate::parse_from_str(date_part, "%Y%m%d").ok()
}

/// ファイル名からYYYYMMDD_HHMMSS部分を抽出して "YYYY-MM-DD HH:MM:SS" に変換
fn extract_datetime_from_backup(filename: &str) -> Option<String> {
    let (date_part, time_part) = split_backup_stamp(filename)?;
    // 日付・時刻のパースチェック
    chrono::NaiveDate::parse_from_str(date_part, "%Y%m%d").ok()?;
    chrono::NaiveTime::parse_from_str(time_part, "%H%M%S").ok()?;

    Some(format!(
        "{}-{}-{} {}:{}:{}",
        &date_part[..4],
        &date_part[4..6],
        &date_part[6..8],
        &time_part[..2],
        &time_part[2..4],
        &time_part[4..6],
    ))
}

/// 作業名（MNT-01-D7）
fn partial_name(file_name: &str) -> String {
    format!("{file_name}{PARTIAL_SUFFIX}")
}

/// 共有の判定「使える控え」（71 §71.12）
fn is_usable_backup(inspection: &BackupInspection) -> bool {
    !inspection.newer_than_app && inspection.quick_check_ok
}

/// 絶対化と `.`・`..` の解決（71 §71.4 の「書く先の固定」の (1)）
#[cfg(not(windows))]
fn normalize_path(dir: &Path) -> std::io::Result<PathBuf> {
    use std::path::Component;

    let absolute = std::path::absolute(dir)?;
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

/// `GetFullPathNameW` で絶対化と `.`・`..`・`/`・末尾の空白とピリオドを解決する（71 §71.4 の (1)）
#[cfg(windows)]
fn normalize_path(dir: &Path) -> std::io::Result<PathBuf> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows_sys::Win32::Storage::FileSystem::GetFullPathNameW;
    let wide: Vec<u16> = dir
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut buffer = vec![0u16; 260];
    loop {
        let capacity = u32::try_from(buffer.len()).map_err(std::io::Error::other)?;
        let length = unsafe {
            GetFullPathNameW(
                wide.as_ptr(),
                capacity,
                buffer.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        } as usize;
        if length == 0 {
            return Err(std::io::Error::last_os_error());
        }
        if length < buffer.len() {
            return Ok(PathBuf::from(std::ffi::OsString::from_wide(
                &buffer[..length],
            )));
        }
        // 足りないときの戻り値は終端の null を含む必要な長さ
        buffer.resize(length, 0);
    }
}

/// 在る path の実体の path（71 §71.4 の (2)。symbolic link を辿る）
#[cfg(not(windows))]
fn final_path(existing: &Path) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(existing)
}

/// 在る path を開き、`\\?\Volume{GUID}\…` の実体の path を得て handle を閉じる（71 §71.4 の (2)）
#[cfg(windows)]
fn final_path(existing: &Path) -> std::io::Result<PathBuf> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, VOLUME_NAME_GUID,
    };
    let wide: Vec<u16> = existing
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // FILE_FLAG_OPEN_REPARSE_POINT を付けない: junction・link・mounted folder は辿った先になる。
    // access は 0（Contract Probe の P8 が PASS した形: access 0・share 3 種・BACKUP_SEMANTICS）
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error());
    }
    let mut buffer = vec![0u16; 260];
    let result = loop {
        let capacity = match u32::try_from(buffer.len()) {
            Ok(capacity) => capacity,
            Err(error) => break Err(std::io::Error::other(error)),
        };
        let length = unsafe {
            GetFinalPathNameByHandleW(handle, buffer.as_mut_ptr(), capacity, VOLUME_NAME_GUID)
        } as usize;
        if length == 0 {
            break Err(std::io::Error::last_os_error());
        }
        if length < buffer.len() {
            break Ok(PathBuf::from(std::ffi::OsString::from_wide(
                &buffer[..length],
            )));
        }
        // 足りないときの戻り値は終端の null を含む必要な長さ
        buffer.resize(length, 0);
    };
    // handle は保たない（junction の target を指すので、保っても drive 文字の移動は防げない）
    unsafe { CloseHandle(handle) };
    result
}

/// `path` かその祖先の直下にある `CASIO\SR500…`（`REGISTER_SD_FOLDER`）の directory
fn register_sd_folder_above(path: &Path) -> Option<PathBuf> {
    let [casio, model] = REGISTER_SD_FOLDER;
    path.ancestors()
        .map(|ancestor| ancestor.join(casio).join(model))
        .find(|candidate| candidate.is_dir())
}

/// 前回の中断の残りの作業名を消す（§71.4 手順 3。規約に合う名前の `.partial` だけ、失敗は warn）
fn remove_stale_partials(ops: &dyn RestoreFileOps, dir: &Path) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(dir = %dir.display(), error = %error, "作業名の掃除の一覧に失敗");
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                tracing::warn!(dir = %dir.display(), error = %error, "作業名の掃除の一覧に失敗");
                continue;
            }
        };
        let name = entry.file_name();
        let is_stale_partial = name
            .to_str()
            .and_then(|name| name.strip_suffix(PARTIAL_SUFFIX))
            .is_some_and(|stem| extract_datetime_from_backup(stem).is_some());
        if is_stale_partial {
            if let Err(error) = ops.remove_file(&entry.path()) {
                tracing::warn!(file = %entry.path().display(), error = %error, "残りの作業名の削除に失敗");
            }
        }
    }
}

/// 公開の前の失敗で作業名を消す（best-effort。消せなければ warn で、次回の手順 3 が消す）
fn discard_partial(ops: &dyn RestoreFileOps, partial: &Path) {
    match ops.remove_file(partial) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            tracing::warn!(file = %partial.display(), error = %error, "作業名の削除に失敗");
        }
    }
}

/// SQLite の URI の path の部分（71 §71.12 手順 1。`%`・`?`・`#` を 1 回の走査で符号化し、`\` は触らない）
fn sqlite_uri_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            '%' => encoded.push_str("%25"),
            '?' => encoded.push_str("%3f"),
            '#' => encoded.push_str("%23"),
            other => encoded.push(other),
        }
    }
    encoded
}

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// 書く先を正規化し、判定した volume に固定する（71 §71.4 の「書く先の固定」）
///
/// 既に在る最も近い祖先の実体の path に、まだ無い末尾の component を付けて `actual` にする。
pub fn pin_to_volume(dir: &Path) -> std::io::Result<PinnedPath> {
    #[cfg(test)]
    run_step_hook("pin_to_volume", dir).map_err(std::io::Error::other)?;
    let normalized = normalize_path(dir)?;
    let mut existing = normalized.as_path();
    loop {
        match std::fs::symlink_metadata(existing) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                existing = existing.parent().ok_or(error)?;
            }
            Err(error) => return Err(error),
        }
    }
    let rest = normalized
        .strip_prefix(existing)
        .map_err(std::io::Error::other)?
        .to_path_buf();
    let actual = final_path(existing)?;
    let actual = if rest.as_os_str().is_empty() {
        actual
    } else {
        actual.join(rest)
    };
    Ok(PinnedPath { normalized, actual })
}

/// backup の保存先を固定し、レジの SD の上でないことを確かめる（MNT-01-D10）
///
/// 正規化した元の path と実体の path の両方の祖先で判定する。
pub fn pin_backup_dir(backup_dir: &Path) -> Result<PinnedPath, BackupDirError> {
    let pinned = pin_to_volume(backup_dir).map_err(|error| {
        tracing::warn!(path = %backup_dir.display(), error = %error, "バックアップの保存先を固定できません");
        BackupDirError::Unpinnable
    })?;
    for base in [&pinned.normalized, &pinned.actual] {
        if let Some(sd_folder) = register_sd_folder_above(base) {
            return Err(BackupDirError::RegisterSd { sd_folder });
        }
    }
    Ok(pinned)
}

/// 控えを変えずに開き、版・`quick_check`・中身を返す（71 §71.12、MNT-01-D9）
pub fn inspect_backup(backup_path: &Path) -> Result<BackupInspection, DbError> {
    use rusqlite::OpenFlags;
    // 1. 読取り専用・作成なし・immutable の URI で開く（file と folder を変えない）
    let path = backup_path
        .to_str()
        .ok_or_else(|| DbError::QueryFailed("控えの path を文字列にできません".to_string()))?;
    let uri = format!("file:{}?immutable=1", sqlite_uri_path(path));
    let conn = rusqlite::Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| DbError::QueryFailed(format!("控えを開けません: {e}")))?;
    let size_bytes = read_metadata(backup_path)
        .map_err(|e| DbError::QueryFailed(format!("控えのサイズ取得に失敗: {e}")))?
        .len();

    // 2. 版（migrate と同じ関数）
    let schema_version = db::migration::read_current_version_without_ddl(&conn)?;
    let app_max_version = db::migration::app_max_version();
    let newer_than_app = schema_version > app_max_version;

    // 2a. このアプリの backup か
    if schema_version == 0 {
        return Err(DbError::QueryFailed(
            "このアプリのバックアップではありません（版がありません）".to_string(),
        ));
    }
    if !newer_than_app {
        let required: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('products', 'operation_logs', 'app_settings')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| DbError::QueryFailed(format!("必須の表の確認に失敗: {e}")))?;
        if required != 3 {
            return Err(DbError::QueryFailed(
                "このアプリのバックアップではありません（必須の表がありません）".to_string(),
            ));
        }
    }

    // 3. quick_check（1 行の ok だけが合格。実行できない破損も不合格）
    let quick_check_ok = conn
        .prepare("PRAGMA quick_check")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()
        })
        .is_ok_and(|rows| rows == ["ok"]);

    // 4. 中身（失敗は None）
    let (product_count, last_operation_at) = if !newer_than_app && quick_check_ok {
        (
            conn.query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
                .ok(),
            conn.query_row("SELECT MAX(created_at) FROM operation_logs", [], |row| {
                row.get::<_, Option<String>>(0)
            })
            .ok()
            .flatten(),
        )
    } else {
        (None, None)
    };

    let file_name = backup_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(BackupInspection {
        created_at: extract_datetime_from_backup(&file_name),
        file_name,
        size_bytes,
        schema_version,
        app_max_version,
        newer_than_app,
        quick_check_ok,
        product_count,
        last_operation_at,
    })
}

/// バックアップディレクトリを解決する
///
/// 71-mnt-backup.md §71.9
///
/// app_settings の backup_path を優先し、未設定/空なら app_data/backups をデフォルトとする。
pub fn resolve_backup_dir(conn: &DbConnection, app_data: &Path) -> Result<PathBuf, DbError> {
    Ok(get_backup_setting(conn, "backup_path")?
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| app_data.join("backups")))
}

/// SQLiteデータベースの安全なバックアップを作成する
///
/// 71-mnt-backup.md §71.4
///
/// 作業名へ `VACUUM INTO` し、共有の検査に通してから上書きしない公開で正式名にする（MNT-01-D7）。
/// 書く先は判定した volume に固定する（MNT-01-D10）。
pub fn create_backup(conn: &DbConnection, backup_dir: &Path) -> Result<BackupResult, DbError> {
    create_backup_with(
        conn,
        backup_dir,
        &StdRestoreFileOps,
        chrono::Local::now().naive_local(),
    )
}

fn create_backup_with(
    conn: &DbConnection,
    backup_dir: &Path,
    ops: &dyn RestoreFileOps,
    now: chrono::NaiveDateTime,
) -> Result<BackupResult, DbError> {
    // 0. 書く先を固定して確かめる。以後は実体の path で行う
    let pinned = pin_backup_dir(backup_dir)
        .map_err(|e| DbError::QueryFailed(format!("バックアップの保存先に書けません: {e}")))?;
    #[cfg(test)]
    run_step_hook("pinned", &pinned.actual).map_err(DbError::QueryFailed)?;
    let dir = pinned.actual;

    // 1. ディレクトリ作成
    std::fs::create_dir_all(&dir).map_err(|e| {
        DbError::QueryFailed(format!("バックアップディレクトリの作成に失敗: {}", e))
    })?;

    // 2. ファイル名生成
    let file_name = format!(
        "{}{}{}",
        BACKUP_PREFIX,
        now.format("%Y%m%d_%H%M%S"),
        BACKUP_EXT
    );
    let published = dir.join(&file_name);
    let partial = dir.join(partial_name(&file_name));

    // 3. 前回の残りの作業名を消す
    remove_stale_partials(ops, &dir);

    // 4. 作業名へ VACUUM INTO（シングルクォートをエスケープ）
    let escaped_path = partial.to_string_lossy().replace('\'', "''");
    let vacuum = conn
        .execute_batch(&format!("VACUUM INTO '{}'", escaped_path))
        .map_err(|e| e.to_string());
    #[cfg(test)]
    let vacuum = vacuum.and_then(|()| run_step_hook("vacuumed", &partial));
    if let Err(e) = vacuum {
        discard_partial(ops, &partial);
        return Err(DbError::QueryFailed(format!("VACUUM INTOに失敗: {}", e)));
    }

    // 4a. 共有の検査（使える控え）
    match inspect_backup(&partial) {
        Ok(inspection) if is_usable_backup(&inspection) => {}
        Ok(_) => {
            discard_partial(ops, &partial);
            return Err(DbError::QueryFailed(
                "作成したバックアップの検査に合格しませんでした".to_string(),
            ));
        }
        Err(e) => {
            discard_partial(ops, &partial);
            return Err(DbError::QueryFailed(format!(
                "作成したバックアップの検査に失敗: {}",
                e
            )));
        }
    }

    // 4b. sync → 上書きしない公開 → 親 directory の sync
    if let Err(e) = ops.sync_file(&partial) {
        discard_partial(ops, &partial);
        return Err(DbError::QueryFailed(format!(
            "バックアップの書込みの確定に失敗: {}",
            e
        )));
    }
    if let Err(e) = ops.publish_no_replace(&partial, &published) {
        discard_partial(ops, &partial);
        return Err(DbError::QueryFailed(format!(
            "バックアップの公開に失敗: {}",
            e
        )));
    }
    // 公開の後の失敗: 検査済みの正式名は残る（§71.4 のエラーハンドリング）
    ops.sync_parent(&published)
        .map_err(|e| DbError::QueryFailed(format!("バックアップの公開の確定に失敗: {}", e)))?;

    // 5. ファイルサイズ取得（実体の path の正式名）
    let size_bytes = read_metadata(&published)
        .map(|m| m.len())
        .map_err(|e| DbError::QueryFailed(format!("バックアップサイズ取得に失敗: {}", e)))?;

    // 6. 操作ログ記録（失敗してもバックアップは成功扱い）
    if let Err(e) = db::system_repo::insert_operation_log(
        conn,
        &NewOperationLog {
            operation_type: "backup_create".to_string(),
            summary: format!("バックアップを作成しました: {}", file_name),
            detail_json: Some(format!(
                r#"{{"file_name":"{}","size_bytes":{}}}"#,
                file_name, size_bytes
            )),
        },
    ) {
        tracing::warn!(error = %e, "バックアップ操作ログの記録に失敗");
    }

    // 7. 返す path は今までどおり {backup_dir}/{ファイル名}
    Ok(BackupResult {
        file_path: backup_dir.join(&file_name).to_string_lossy().to_string(),
        file_name,
        size_bytes,
    })
}

/// 保持日数を超えた古いバックアップファイルを削除する
///
/// 71-mnt-backup.md §71.5
pub fn cleanup_old_backups(backup_dir: &Path, retention_days: u32) -> Result<u32, std::io::Error> {
    // 0. 固定して確かめる（MNT-01-D10）。以後の一覧と削除は実体の path で行う
    let pinned = pin_backup_dir(backup_dir)
        .map_err(|e| std::io::Error::other(format!("バックアップの保存先で削除しません: {e}")))?;
    #[cfg(test)]
    run_step_hook("pinned", &pinned.actual).map_err(std::io::Error::other)?;
    let entries = match std::fs::read_dir(&pinned.actual) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e),
    };

    let today = chrono::Local::now().date_naive();
    let mut deleted = 0u32;

    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if let Some(file_date) = extract_date_from_backup(&filename_str) {
            let age = today - file_date;
            if age.num_days() > i64::from(retention_days) {
                if let Err(e) = std::fs::remove_file(entry.path()) {
                    tracing::warn!(
                        file = %filename_str,
                        error = %e,
                        "古いバックアップファイルの削除に失敗"
                    );
                } else {
                    deleted += 1;
                }
            }
        }
    }

    Ok(deleted)
}

/// バックアップディレクトリ内のバックアップファイル一覧を返す
///
/// 71-mnt-backup.md §71.6
///
/// 新しい順（created_at DESC）でソート。
pub fn list_backups(backup_dir: &Path) -> Result<Vec<BackupInfo>, std::io::Error> {
    let entries = match std::fs::read_dir(backup_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e),
    };

    let mut backups = Vec::new();

    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy().to_string();

        if let Some(created_at) = extract_datetime_from_backup(&filename_str) {
            let size_bytes = read_metadata(&entry.path())?.len();
            let file_path = entry.path().to_string_lossy().to_string();

            backups.push(BackupInfo {
                file_name: filename_str,
                file_path,
                size_bytes,
                created_at,
            });
        }
    }

    // 新しい順
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

/// 自動バックアップの条件を判定し、必要なら実行する
///
/// 71-mnt-backup.md §71.8
///
/// 起動時に呼ばれる。backup_enabled=1の場合のみ動作。
/// 今日のバックアップが無ければ即実行、あればbackup_time到達後に2回目実行。
pub fn check_auto_backup(conn: &DbConnection, backup_dir: &Path) -> Result<bool, DbError> {
    // 1. backup_enabled チェック
    let enabled = db::system_repo::get_setting(conn, "backup_enabled")?;
    if enabled.as_deref() != Some("1") {
        return Ok(false);
    }

    // 2. 今日の日付
    let now = chrono::Local::now();
    let today_str = now.format("%Y%m%d").to_string();
    let today_prefix = format!("{}{}_", BACKUP_PREFIX, today_str);

    // 3. 今日のバックアップファイルを走査
    let today_files = match std::fs::read_dir(backup_dir) {
        Ok(entries) => collect_today_backup_names(
            backup_dir,
            &today_prefix,
            entries.map(|e| e.map(|entry| entry.path())),
        )?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => vec![],
        Err(e) => {
            return Err(DbError::QueryFailed(format!(
                "バックアップディレクトリの読み取りに失敗: {}",
                e
            )));
        }
    };

    // 4. 今日のバックアップが無い → 即実行
    if today_files.is_empty() {
        create_backup(conn, backup_dir)?;
        run_cleanup(conn, backup_dir);
        return Ok(true);
    }

    // 5. backup_time チェック
    let backup_time = match db::system_repo::get_setting(conn, "backup_time")? {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(false),
    };

    // 6. HH:MM パース
    let target_time = match chrono::NaiveTime::parse_from_str(&backup_time, "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            tracing::warn!(
                backup_time = %backup_time,
                "backup_timeのパースに失敗（定時バックアップをスキップ）"
            );
            return Ok(false);
        }
    };

    // 7. 現在時刻がbackup_time より前 → スキップ
    if now.time() < target_time {
        return Ok(false);
    }

    // 8. backup_time以降のバックアップがあるか確認
    let target_hhmmss = target_time.format("%H%M%S").to_string();
    let has_post_time_backup = today_files.iter().any(|name| {
        // ファイル名から HHMMSS 部分を抽出
        name.strip_prefix(&today_prefix)
            .and_then(|rest| rest.strip_suffix(BACKUP_EXT))
            .map(|hhmmss| hhmmss >= target_hhmmss.as_str())
            .unwrap_or(false)
    });

    if has_post_time_backup {
        return Ok(false);
    }

    // 9. backup_time以降のバックアップなし → 実行
    create_backup(conn, backup_dir)?;
    run_cleanup(conn, backup_dir);
    Ok(true)
}

fn collect_today_backup_names<I>(
    backup_dir: &Path,
    today_prefix: &str,
    entries: I,
) -> Result<Vec<String>, DbError>
where
    I: IntoIterator<Item = std::io::Result<PathBuf>>,
{
    let mut names = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| {
            DbError::QueryFailed(format!(
                "バックアップディレクトリのエントリ読み取りに失敗 ({}): {}",
                backup_dir.display(),
                error
            ))
        })?;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        // 規約の完全一致（作業名・規約外の名前を数えない、§71.8 手順 3）
        if name.starts_with(today_prefix) && extract_datetime_from_backup(&name).is_some() {
            names.push(name);
        }
    }
    Ok(names)
}

/// cleanup_old_backups を設定値で実行するヘルパー（エラーはwarnのみ）
fn run_cleanup(conn: &DbConnection, backup_dir: &Path) {
    let retention_days = match get_backup_setting(conn, "backup_retention_days") {
        Ok(Some(value)) => match value.parse::<u32>() {
            Ok(days) => days,
            Err(e) => {
                tracing::warn!(
                    backup_retention_days = %value,
                    error = %e,
                    "backup_retention_daysのparseに失敗（cleanupをスキップ）"
                );
                return;
            }
        },
        Ok(None) => DEFAULT_RETENTION_DAYS,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "backup_retention_daysの読取に失敗（cleanupをスキップ）"
            );
            return;
        }
    };

    if let Err(e) = cleanup_old_backups(backup_dir, retention_days) {
        tracing::warn!(error = %e, "古いバックアップの削除に失敗");
    }
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

    #[test]
    fn test_backup_module_req901_exposes_no_create_restore_reconnect() {
        // REQ-901 / MNT-01 / SPEC-CMD11-IMPL-D5
        let _: fn(&str) -> Result<DbConnection, DbError> = open_existing_database;
    }

    fn write_backup_days_old(backup_dir: &Path, days: i64) -> PathBuf {
        std::fs::create_dir_all(backup_dir).unwrap();
        let date = (chrono::Local::now().date_naive() - chrono::Duration::days(days))
            .format("%Y%m%d")
            .to_string();
        let path = backup_dir.join(format!("{BACKUP_PREFIX}{date}_120000{BACKUP_EXT}"));
        std::fs::write(&path, "synthetic backup").unwrap();
        path
    }

    #[test]
    fn test_resolve_backup_dir_req901_d2_propagates_db_error() {
        // REQ-901 / MNT-01-D2 / Matrix C1
        let (dir, conn) = setup_test_db();
        let _failure = fail_setting_read("backup_path");

        let result = resolve_backup_dir(&conn, dir.path());

        assert!(
            matches!(result, Err(DbError::QueryFailed(message)) if message.contains("backup_path"))
        );
    }

    #[test]
    fn test_resolve_backup_dir_req901_d2_defaults_when_missing() {
        // REQ-901 / MNT-01-D2 / Matrix C2
        let (dir, conn) = setup_test_db();
        conn.execute("DELETE FROM app_settings WHERE key = 'backup_path'", [])
            .unwrap();

        assert_eq!(
            resolve_backup_dir(&conn, dir.path()).unwrap(),
            dir.path().join("backups")
        );
    }

    #[test]
    fn test_resolve_backup_dir_req901_d2_defaults_when_empty() {
        // REQ-901 / MNT-01-D2 / Matrix C3
        let (dir, conn) = setup_test_db();
        db::system_repo::upsert_setting(&conn, "backup_path", "").unwrap();

        assert_eq!(
            resolve_backup_dir(&conn, dir.path()).unwrap(),
            dir.path().join("backups")
        );
    }

    #[test]
    fn test_resolve_backup_dir_req901_d2_uses_configured_path() {
        // REQ-901 / MNT-01-D2 / Matrix C4
        let (dir, conn) = setup_test_db();
        let configured = dir.path().join("configured-backups");
        db::system_repo::upsert_setting(
            &conn,
            "backup_path",
            configured.to_string_lossy().as_ref(),
        )
        .unwrap();

        assert_eq!(resolve_backup_dir(&conn, dir.path()).unwrap(), configured);
    }

    #[test]
    fn test_resolve_backup_dir_req901_d2_all_callers_handle_result_contract() {
        // REQ-901 / MNT-01-D2 / Matrix C7
        let lib = include_str!("../lib.rs");
        let settings = include_str!("../cmd/settings_cmd.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert_eq!(lib.matches("mnt::backup::resolve_backup_dir").count(), 1);
        assert_eq!(settings.matches("backup::resolve_backup_dir").count(), 1);
        assert!(lib.contains("match mnt::backup::resolve_backup_dir"));
        assert!(settings.contains("backup::resolve_backup_dir(conn, &app_data).map_err(db_err)"));
    }

    #[test]
    fn test_run_cleanup_req901_d3_missing_setting_uses_default() {
        // REQ-901 / MNT-01-D3 / Matrix D1
        let (dir, conn) = setup_test_db();
        conn.execute(
            "DELETE FROM app_settings WHERE key = 'backup_retention_days'",
            [],
        )
        .unwrap();
        let backup_dir = dir.path().join("backups");
        let expired = write_backup_days_old(&backup_dir, 4);

        run_cleanup(&conn, &backup_dir);

        assert!(!expired.exists(), "未設定時は既定3日で削除するべき");
    }

    #[test]
    fn test_check_auto_backup_req901_d3_db_error_skips_cleanup_but_keeps_backup() {
        // REQ-901 / MNT-01-D3 / Matrix D2
        let (dir, conn) = setup_test_db();
        db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();
        let backup_dir = dir.path().join("backups");
        let expired = write_backup_days_old(&backup_dir, 4);
        let _failure = fail_setting_read("backup_retention_days");

        let (result, logs) = crate::test_tracing::capture(|| check_auto_backup(&conn, &backup_dir));

        assert!(result.unwrap(), "cleanup失敗でもbackup作成は成功するべき");
        assert!(
            expired.exists(),
            "保持日数を読めない場合は削除してはならない"
        );
        assert!(
            list_backups(&backup_dir).unwrap().len() >= 2,
            "既存syntheticファイルに加えて新規backupが作成されるべき"
        );
        assert!(
            logs.contains("WARN") && logs.contains("cleanup"),
            "logs={logs:?}"
        );
    }

    #[test]
    fn test_run_cleanup_req901_d3_parse_error_skips_cleanup() {
        // REQ-901 / MNT-01-D3 / Matrix D3
        let (dir, conn) = setup_test_db();
        db::system_repo::upsert_setting(&conn, "backup_retention_days", "abc").unwrap();
        let backup_dir = dir.path().join("backups");
        let expired = write_backup_days_old(&backup_dir, 4);

        let (_, logs) = crate::test_tracing::capture(|| run_cleanup(&conn, &backup_dir));

        assert!(expired.exists(), "parse不能時は削除してはならない");
        assert!(
            logs.contains("WARN") && logs.contains("parse"),
            "logs={logs:?}"
        );
    }

    #[test]
    fn test_run_cleanup_req901_d3_uses_valid_retention() {
        // REQ-901 / MNT-01-D3 / Matrix D4
        let (dir, conn) = setup_test_db();
        db::system_repo::upsert_setting(&conn, "backup_retention_days", "90").unwrap();
        let backup_dir = dir.path().join("backups");
        let four_days_old = write_backup_days_old(&backup_dir, 4);

        run_cleanup(&conn, &backup_dir);

        assert!(four_days_old.exists(), "設定した90日基準を使うべき");
    }

    // -----------------------------------------------------------------------
    // create_backup テスト（4件）
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_backup_req901_creates_file() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: VACUUM INTO でバックアップファイルが生成される
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        let result = create_backup(&conn, &backup_dir).unwrap();

        assert!(
            Path::new(&result.file_path).exists(),
            "バックアップファイルが存在するべき"
        );
        assert!(result.size_bytes > 0, "ファイルサイズが0より大きいべき");
    }

    #[test]
    fn test_create_backup_req901_filename_format() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: ファイル名が inventory_backup_YYYYMMDD_HHMMSS.db 形式
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        let result = create_backup(&conn, &backup_dir).unwrap();

        assert!(
            result.file_name.starts_with(BACKUP_PREFIX),
            "ファイル名が {} で始まるべき: {}",
            BACKUP_PREFIX,
            result.file_name
        );
        assert!(
            result.file_name.ends_with(BACKUP_EXT),
            "ファイル名が {} で終わるべき: {}",
            BACKUP_EXT,
            result.file_name
        );
        // YYYYMMDD_HHMMSS 部分が15文字
        let stem = result
            .file_name
            .strip_prefix(BACKUP_PREFIX)
            .unwrap()
            .strip_suffix(BACKUP_EXT)
            .unwrap();
        assert_eq!(stem.len(), 15, "日時部分は15文字であるべき: {}", stem);
    }

    #[test]
    fn test_create_backup_req901_data_integrity() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: バックアップDBに現在のデータが含まれる
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        // テストデータを挿入
        conn.execute(
            "INSERT INTO suppliers (name, created_at) VALUES ('テスト取引先', '2026-01-01T00:00:00')",
            [],
        )
        .unwrap();

        let result = create_backup(&conn, &backup_dir).unwrap();

        // バックアップDBを開いてデータ確認
        let backup_conn = rusqlite::Connection::open(&result.file_path).unwrap();
        let name: String = backup_conn
            .query_row(
                "SELECT name FROM suppliers WHERE name = 'テスト取引先'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "テスト取引先");
    }

    #[test]
    fn test_create_backup_req901_logs_operation() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: operation_type='backup_create' のログが記録される
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        create_backup(&conn, &backup_dir).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'backup_create'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "backup_create ログが1件記録されるべき");
    }

    // -----------------------------------------------------------------------
    // cleanup_old_backups テスト（2件）
    // -----------------------------------------------------------------------

    #[test]
    fn test_cleanup_old_backups_req901_deletes_expired() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: 保持日数超過ファイルが削除される
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        // 10日前のファイルを作成
        let old_date = (chrono::Local::now().date_naive() - chrono::Duration::days(10))
            .format("%Y%m%d")
            .to_string();
        let old_file = backup_dir.join(format!(
            "{}{}_{}{}",
            BACKUP_PREFIX, old_date, "120000", BACKUP_EXT
        ));
        std::fs::write(&old_file, "dummy").unwrap();

        let deleted = cleanup_old_backups(&backup_dir, 3).unwrap();

        assert_eq!(deleted, 1, "1件削除されるべき");
        assert!(!old_file.exists(), "古いファイルは削除されるべき");
    }

    #[test]
    fn test_cleanup_old_backups_req901_skips_non_timestamp_suffix() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01 §71.5 ステップ2a-2d / Matrix D5a: ファイル名は
        // inventory_backup_YYYYMMDD_HHMMSS.db の完全一致のみ削除対象。
        // 非 timestamp suffix（手動リネーム等）は保持日数超過でも削除しない
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        let old_date = (chrono::Local::now().date_naive() - chrono::Duration::days(10))
            .format("%Y%m%d")
            .to_string();
        // stem 長 15 だが suffix が時刻でない（"_manual" は separator 込みで 7 文字）
        let manual_file = backup_dir.join(format!("{BACKUP_PREFIX}{old_date}_manual{BACKUP_EXT}"));
        std::fs::write(&manual_file, "synthetic manual copy").unwrap();
        // separator 位置が '_' でないケース
        let no_sep_file = backup_dir.join(format!("{BACKUP_PREFIX}{old_date}x120000{BACKUP_EXT}"));
        std::fs::write(&no_sep_file, "synthetic").unwrap();

        let deleted = cleanup_old_backups(&backup_dir, 3).unwrap();

        assert_eq!(deleted, 0, "パターン不一致ファイルは削除件数に含めない");
        assert!(
            manual_file.exists(),
            "非 timestamp suffix は削除してはならない"
        );
        assert!(no_sep_file.exists(), "separator 不一致は削除してはならない");
    }

    #[test]
    fn test_cleanup_old_backups_req901_warns_and_continues_on_delete_failure() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01 §71.5 ステップ2g / Matrix D5b: 個別ファイルの削除失敗は
        // tracing::warn! を記録して次のファイルへ進む（全体を中断しない）
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        let old_date = (chrono::Local::now().date_naive() - chrono::Duration::days(10))
            .format("%Y%m%d")
            .to_string();
        // パターン一致名の「ディレクトリ」= remove_file が決定論的に失敗する
        let undeletable = backup_dir.join(format!("{BACKUP_PREFIX}{old_date}_120000{BACKUP_EXT}"));
        std::fs::create_dir_all(&undeletable).unwrap();
        // 通常の削除対象ファイル（処理順に依存せず削除継続を検証できる）
        let deletable = backup_dir.join(format!("{BACKUP_PREFIX}{old_date}_130000{BACKUP_EXT}"));
        std::fs::write(&deletable, "synthetic").unwrap();

        let (result, logs) = crate::test_tracing::capture(|| cleanup_old_backups(&backup_dir, 3));

        assert_eq!(
            result.unwrap(),
            1,
            "削除失敗があっても他ファイルの削除は継続"
        );
        assert!(undeletable.exists(), "削除失敗対象は残る");
        assert!(!deletable.exists(), "後続の削除対象は削除される");
        assert!(
            logs.contains("WARN") && logs.contains("削除に失敗"),
            "logs={logs:?}"
        );
    }

    #[test]
    fn test_cleanup_old_backups_req901_keeps_recent() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: 保持日数内のファイルが保持される
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        // 今日のファイルを作成
        let today = chrono::Local::now()
            .date_naive()
            .format("%Y%m%d")
            .to_string();
        let recent_file = backup_dir.join(format!(
            "{}{}_{}{}",
            BACKUP_PREFIX, today, "120000", BACKUP_EXT
        ));
        std::fs::write(&recent_file, "dummy").unwrap();

        // 2日前のファイルを作成
        let two_days_ago = (chrono::Local::now().date_naive() - chrono::Duration::days(2))
            .format("%Y%m%d")
            .to_string();
        let recent_file2 = backup_dir.join(format!(
            "{}{}_{}{}",
            BACKUP_PREFIX, two_days_ago, "120000", BACKUP_EXT
        ));
        std::fs::write(&recent_file2, "dummy").unwrap();

        let deleted = cleanup_old_backups(&backup_dir, 3).unwrap();

        assert_eq!(deleted, 0, "削除されるべきファイルは無い");
        assert!(recent_file.exists(), "今日のファイルは保持されるべき");
        assert!(recent_file2.exists(), "2日前のファイルは保持されるべき");
    }

    // -----------------------------------------------------------------------
    // list_backups テスト（2件）
    // -----------------------------------------------------------------------

    #[test]
    fn test_list_backups_req901_returns_sorted() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: 新しい順でBackupInfoが返される
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        // 古いファイル
        let file1 = backup_dir.join(format!("{}20260410_100000{}", BACKUP_PREFIX, BACKUP_EXT));
        std::fs::write(&file1, "old").unwrap();

        // 新しいファイル
        let file2 = backup_dir.join(format!("{}20260413_150000{}", BACKUP_PREFIX, BACKUP_EXT));
        std::fs::write(&file2, "new").unwrap();

        let list = list_backups(&backup_dir).unwrap();

        assert_eq!(list.len(), 2, "2件返されるべき");
        assert_eq!(
            list[0].created_at, "2026-04-13 15:00:00",
            "1番目は新しいファイル"
        );
        assert_eq!(
            list[1].created_at, "2026-04-10 10:00:00",
            "2番目は古いファイル"
        );
    }

    #[test]
    fn test_list_backups_req901_empty_dir() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: 空ディレクトリで空Vecが返される
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        let list = list_backups(&backup_dir).unwrap();
        assert!(list.is_empty(), "空のVecが返されるべき");
    }

    // -----------------------------------------------------------------------
    // restore_backup テスト（3件）
    // -----------------------------------------------------------------------

    #[test]
    fn test_restore_backup_req901_replaces_data() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: リストア後にバックアップ時点のデータに戻る
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("main.db");
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        // 初期状態のDBを作成
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();

        // データ挿入（バックアップに含まれるべき）
        conn.execute(
            "INSERT INTO suppliers (name, created_at) VALUES ('バックアップ前', '2026-01-01T00:00:00')",
            [],
        )
        .unwrap();

        // バックアップ作成
        let backup_result = create_backup(&conn, &backup_dir).unwrap();

        // バックアップ後に追加データ挿入
        conn.execute(
            "INSERT INTO suppliers (name, created_at) VALUES ('バックアップ後', '2026-01-02T00:00:00')",
            [],
        )
        .unwrap();

        // リストア前にバックアップ後のデータが存在することを確認
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM suppliers", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2, "リストア前は2件あるべき");

        // リストア実行
        let backup_path = Path::new(&backup_result.file_path);
        let new_conn = restore_backup(conn, backup_path, &db_path).unwrap();

        // リストア後のデータ確認
        let count: i64 = new_conn
            .query_row("SELECT COUNT(*) FROM suppliers", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "リストア後は1件（バックアップ時点）に戻るべき");

        let name: String = new_conn
            .query_row("SELECT name FROM suppliers LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "バックアップ前", "バックアップ時点のデータであるべき");
    }

    #[test]
    fn test_restore_backup_req901_nonexistent_file() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01-D4: 存在しない復元対象は元接続を壊さない recoverable error
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("main.db");
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();

        let nonexistent = dir.path().join("nonexistent.db");
        let result = restore_backup(conn, &nonexistent, &db_path);

        assert!(result.is_err(), "エラーが返されるべき");
        assert!(matches!(result.unwrap_err(), RestoreError::Recovered(_)));
    }

    #[test]
    fn test_restore_backup_req901_runs_migration() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: init_database がマイグレーション実行することを間接確認
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("main.db");
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();
        let backup_result = create_backup(&conn, &backup_dir).unwrap();

        // リストア実行（init_databaseが呼ばれてPRAGMA再設定 + マイグレーション実行）
        let backup_path = Path::new(&backup_result.file_path);
        let new_conn = restore_backup(conn, backup_path, &db_path).unwrap();

        // PRAGMA foreign_keys が再設定されていることを確認
        let fk_enabled: i64 = new_conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1, "foreign_keysが有効であるべき");

        // schema_versionsテーブルが存在することを確認（マイグレーション済み）
        let version: i64 = new_conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(version >= 1, "マイグレーションが実行済みであるべき");
    }

    // -----------------------------------------------------------------------
    // check_auto_backup テスト（4件）
    // -----------------------------------------------------------------------

    #[test]
    fn test_check_auto_backup_req901_disabled() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: backup_enabled=0 でスキップ
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        // backup_enabled を 0 に設定
        db::system_repo::upsert_setting(&conn, "backup_enabled", "0").unwrap();

        let result = check_auto_backup(&conn, &backup_dir).unwrap();
        assert!(!result, "バックアップ無効時はfalseが返されるべき");
    }

    #[test]
    fn test_check_auto_backup_req901_no_backup_today() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: 今日のバックアップなしで即実行
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        // backup_enabled を有効化
        db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();

        let result = check_auto_backup(&conn, &backup_dir).unwrap();
        assert!(result, "今日のバックアップが無ければ実行されるべき");

        // バックアップファイルが作成されたことを確認
        let list = list_backups(&backup_dir).unwrap();
        assert_eq!(list.len(), 1, "バックアップファイルが1件作成されるべき");
    }

    #[test]
    fn test_check_auto_backup_req901_already_backed_up() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: 今日のバックアップありでスキップ（backup_time未設定）
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        // backup_enabled を有効化
        db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();

        // 初期スキーマで backup_time='23:00' が設定されるため、明示的にクリア
        // （23時以降にテスト実行すると定時バックアップ判定に入り失敗するため）
        db::system_repo::upsert_setting(&conn, "backup_time", "").unwrap();

        // 今日のダミーバックアップファイルを作成
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        let dummy = backup_dir.join(format!(
            "{}{}_{}{}",
            BACKUP_PREFIX, today, "080000", BACKUP_EXT
        ));
        std::fs::write(&dummy, "dummy").unwrap();

        let result = check_auto_backup(&conn, &backup_dir).unwrap();
        assert!(
            !result,
            "今日のバックアップがあり、backup_time未設定ならスキップされるべき"
        );
    }

    #[test]
    fn test_check_auto_backup_req901_scheduled_time() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: backup_time到達で2回目のバックアップ実行
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();

        // backup_enabled を有効化
        db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();

        // backup_time を 00:00 に設定（テスト実行時には必ず過ぎている）
        db::system_repo::upsert_setting(&conn, "backup_time", "00:00").unwrap();

        // 今日の「早朝」バックアップ（00:00より前の00:00:00はありえないが、テスト用）
        // backup_time(00:00) 以前のバックアップとして判定されるよう、
        // check_auto_backup の「backup_time以降のバックアップがあるか」チェックを通過させる必要がある。
        // ただし 00:00 の場合、000000 >= 000000 は true なので、
        // backup_time を "00:01" にして 000000 < 000100 を成立させる。
        db::system_repo::upsert_setting(&conn, "backup_time", "00:01").unwrap();

        let today = chrono::Local::now().format("%Y%m%d").to_string();
        let early_backup = backup_dir.join(format!(
            "{}{}_{}{}",
            BACKUP_PREFIX, today, "000000", BACKUP_EXT
        ));
        std::fs::write(&early_backup, "dummy").unwrap();

        let result = check_auto_backup(&conn, &backup_dir).unwrap();
        assert!(
            result,
            "backup_time以降のバックアップが無ければ実行されるべき"
        );

        // 合計2ファイル（ダミー + 新規作成）
        let list = list_backups(&backup_dir).unwrap();
        assert!(
            list.len() >= 2,
            "ダミー + 新規のバックアップが存在するべき: 実際={}",
            list.len()
        );
    }

    #[test]
    fn test_check_auto_backup_req901_read_dir_error() {
        // REQ-901: バックアップ
        // Task: MNT-01
        // MNT-01: backup_dir がファイルの場合、NotFound以外のエラーで DbError を返す
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");

        // ディレクトリではなくファイルを作成 → read_dir が "Not a directory" で失敗
        std::fs::write(&backup_dir, "not a directory").unwrap();

        db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();

        let result = check_auto_backup(&conn, &backup_dir);
        assert!(result.is_err(), "read_dir失敗時はエラーが返されるべき");
        match result.unwrap_err() {
            DbError::QueryFailed(msg) => {
                assert!(
                    msg.contains("読み取りに失敗"),
                    "エラーメッセージが適切であるべき: {}",
                    msg
                );
            }
            other => panic!("QueryFailedが期待されるが、{:?} が返された", other),
        }
    }

    #[test]
    fn test_check_auto_backup_req901_entry_error_propagates_without_creating_backup() {
        // REQ-901 / MNT-01-D6: entry走査失敗は判定を不確実にするため伝搬し、
        // create / cleanup に進まない
        let dir = tempfile::tempdir().unwrap();
        let sentinel = dir.path().join("inventory_backup_20000101_010203.db");
        std::fs::write(&sentinel, b"synthetic existing backup").unwrap();
        let before = std::fs::read(&sentinel).unwrap();
        let entries = vec![Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "injected",
        ))];
        let result = collect_today_backup_names(dir.path(), "inventory_backup_20260724_", entries);
        assert!(
            matches!(result, Err(DbError::QueryFailed(message)) if message.contains("injected"))
        );
        assert_eq!(std::fs::read(&sentinel).unwrap(), before);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn test_list_backups_req901_metadata_error_propagates() {
        // REQ-901 / MNT-01-D6: metadata error で partial list を返さない
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("inventory_backup_20260724_010203.db");
        std::fs::write(&path, b"db").unwrap();
        let _failure = fail_metadata(&path);
        let error = list_backups(dir.path()).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("injected metadata failure"));
    }

    #[test]
    fn test_create_backup_req901_metadata_error_propagates_without_success_log() {
        // REQ-901 / MNT-01-D6: metadata error は成功 result / operation log にしない。
        // 71 §71.10 の D-114 の表: 注入は公開の後の正式名（実体の path）だけに当てる
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        let now = fixed_now("120000");
        let published = std::fs::canonicalize(dir.path())
            .unwrap()
            .join("backups")
            .join(name_at(now));
        let _failure = fail_metadata(&published);
        let result = create_backup_with(&conn, &backup_dir, &StdRestoreFileOps, now);
        assert!(matches!(result, Err(DbError::QueryFailed(message)) if message.contains("サイズ")));
        assert_eq!(backup_create_logs(&conn), 0);
        assert_eq!(
            std::fs::read_dir(&backup_dir).unwrap().count(),
            1,
            "VACUUM output may remain after metadata failure, but must not be reported as success"
        );
    }

    // -----------------------------------------------------------------------
    // D-114 (a): 作業名・公開・書く先の固定・名前の解析・控えの確かめ
    // -----------------------------------------------------------------------

    use std::collections::BTreeMap;
    use std::sync::Mutex;

    /// 今日の指定の時刻（同じ秒の 2 回目を作るため、時刻を注入する）
    fn fixed_now(hhmmss: &str) -> chrono::NaiveDateTime {
        let today = chrono::Local::now().date_naive();
        chrono::NaiveDateTime::new(
            today,
            chrono::NaiveTime::parse_from_str(hhmmss, "%H%M%S").unwrap(),
        )
    }

    fn name_at(now: chrono::NaiveDateTime) -> String {
        format!(
            "{}{}{}",
            BACKUP_PREFIX,
            now.format("%Y%m%d_%H%M%S"),
            BACKUP_EXT
        )
    }

    fn backup_create_logs(conn: &DbConnection) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'backup_create'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn names_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// folder の中身（file の bytes、link の指す先、directory）を link を辿らずに写す
    fn snapshot(root: &Path) -> BTreeMap<PathBuf, String> {
        fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, String>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                let kind = std::fs::symlink_metadata(&path).unwrap().file_type();
                if kind.is_symlink() {
                    let target = std::fs::read_link(&path).unwrap();
                    out.insert(relative, format!("link:{}", target.display()));
                } else if kind.is_dir() {
                    out.insert(relative, "dir".to_string());
                    walk(root, &path, out);
                } else {
                    out.insert(relative, format!("{:?}", std::fs::read(&path).unwrap()));
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(root, root, &mut out);
        out
    }

    fn make_register_sd(base: &Path) -> PathBuf {
        let [casio, model] = REGISTER_SD_FOLDER;
        let folder = base.join(casio).join(model);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("synthetic.txt"), b"synthetic register file").unwrap();
        folder
    }

    /// 公開の 3 点を記録し、指定の 1 点を失敗させる（restore の注入の型と同じ形）
    struct PublishOps {
        events: Mutex<Vec<&'static str>>,
        fail: Option<&'static str>,
        fail_remove: bool,
    }

    impl PublishOps {
        fn new(fail: Option<&'static str>) -> Self {
            Self {
                events: Mutex::new(Vec::new()),
                fail,
                fail_remove: false,
            }
        }
        fn step(&self, event: &'static str) -> std::io::Result<()> {
            self.events.lock().unwrap().push(event);
            if self.fail == Some(event) {
                return Err(std::io::Error::other(format!("injected {event} failure")));
            }
            Ok(())
        }
    }

    impl RestoreFileOps for PublishOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.rename(source, destination)
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.replace(source, destination)
        }
        fn publish_no_replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            self.step("publish_no_replace")?;
            StdRestoreFileOps.publish_no_replace(source, destination)
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            if self.fail_remove {
                return Err(std::io::Error::other("injected remove failure"));
            }
            StdRestoreFileOps.remove_file(path)
        }
        fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            StdRestoreFileOps.write_file(path, bytes)
        }
        fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            StdRestoreFileOps.read_file(path)
        }
        fn sync_file(&self, path: &Path) -> std::io::Result<()> {
            self.step("sync_file")?;
            StdRestoreFileOps.sync_file(path)
        }
        fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
            self.step("sync_parent")?;
            StdRestoreFileOps.sync_parent(path)
        }
        fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            StdRestoreFileOps.checkpoint(conn)
        }
    }

    /// `products` の root の page を壊す（schema と schema_versions は読めるまま）
    fn corrupt_products_page(path: &Path) {
        let (page_size, root): (i64, i64) = {
            let conn = rusqlite::Connection::open(path).unwrap();
            let page_size = conn
                .query_row("PRAGMA page_size", [], |row| row.get(0))
                .unwrap();
            let root = conn
                .query_row(
                    "SELECT rootpage FROM sqlite_master WHERE type = 'table' AND name = 'products'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            (page_size, root)
        };
        let mut bytes = std::fs::read(path).unwrap();
        let start = usize::try_from((root - 1) * page_size).unwrap();
        let end = start + usize::try_from(page_size).unwrap();
        bytes[start..end].fill(0xA5);
        std::fs::write(path, bytes).unwrap();
    }

    /// アプリの migrate を通した版 = app_max_version() の DB を作って閉じる
    fn app_db(path: &Path, fill: impl FnOnce(&DbConnection)) {
        let conn = db::init_database(path.to_str().unwrap()).unwrap();
        fill(&conn);
        drop(conn);
    }

    #[test]
    fn test_create_backup_req901_d7_vacuum_failure_leaves_no_published_file() {
        // REQ-901 / MNT-01-D7 / 71 §71.4 エラーハンドリング: VACUUM INTO の失敗で
        // 正式名も作業名も残さず、成功の操作ログを書かない
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        let _hook = set_step_hook(|step, path| {
            if step == "vacuumed" {
                assert!(path.exists(), "注入の前提: 作業名に途中の file がある");
                return Err("injected vacuum failure".to_string());
            }
            Ok(())
        });
        let result =
            create_backup_with(&conn, &backup_dir, &StdRestoreFileOps, fixed_now("100000"));
        assert!(matches!(result, Err(DbError::QueryFailed(message)) if message.contains("VACUUM")));
        assert_eq!(names_in(&backup_dir), Vec::<String>::new());
        assert_eq!(backup_create_logs(&conn), 0);

        // 作業名の削除も失敗すると warn で残り、次の create_backup の手順 3 が消す
        let mut ops = PublishOps::new(None);
        ops.fail_remove = true;
        let (result, logs) = crate::test_tracing::capture(|| {
            create_backup_with(&conn, &backup_dir, &ops, fixed_now("100100"))
        });
        assert!(result.is_err());
        let stale = partial_name(&name_at(fixed_now("100100")));
        assert_eq!(names_in(&backup_dir), vec![stale.clone()]);
        assert!(
            logs.contains("WARN") && logs.contains("作業名の削除に失敗"),
            "logs={logs:?}"
        );
        drop(_hook);

        let result =
            create_backup_with(&conn, &backup_dir, &StdRestoreFileOps, fixed_now("100200"))
                .unwrap();
        assert_eq!(names_in(&backup_dir), vec![result.file_name]);
        assert_eq!(backup_create_logs(&conn), 1);
    }

    #[test]
    fn test_create_backup_req901_d7_quick_check_failure_removes_partial_without_success_log() {
        // REQ-901 / MNT-01-D7 手順 4a: 壊れた作業名を公開しない
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        let _hook = set_step_hook(|step, path| {
            if step == "vacuumed" {
                corrupt_products_page(path);
                let inspection = inspect_backup(path).unwrap();
                assert!(
                    !inspection.quick_check_ok,
                    "注入の前提: quick_check が落ちる"
                );
            }
            Ok(())
        });
        let result =
            create_backup_with(&conn, &backup_dir, &StdRestoreFileOps, fixed_now("110000"));
        assert!(matches!(result, Err(DbError::QueryFailed(message)) if message.contains("検査")));
        assert_eq!(names_in(&backup_dir), Vec::<String>::new());
        assert_eq!(backup_create_logs(&conn), 0);
    }

    #[test]
    fn test_check_auto_backup_req901_d7_partial_and_nonconforming_not_counted_as_today() {
        // REQ-901 / MNT-01-D7 / 71 §71.8 手順 3: 作業名と規約外の名前を今日の backup と数えない
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        for leftover in [
            format!("{BACKUP_PREFIX}{today}_000000{BACKUP_EXT}{PARTIAL_SUFFIX}"),
            format!("{BACKUP_PREFIX}{today}_manual{BACKUP_EXT}"),
        ] {
            let (dir, conn) = setup_test_db();
            db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();
            db::system_repo::upsert_setting(&conn, "backup_time", "").unwrap();
            let backup_dir = dir.path().join("backups");
            std::fs::create_dir_all(&backup_dir).unwrap();
            std::fs::write(backup_dir.join(&leftover), b"synthetic").unwrap();

            assert!(
                check_auto_backup(&conn, &backup_dir).unwrap(),
                "{leftover} だけの dir では作るべき"
            );
            assert_eq!(list_backups(&backup_dir).unwrap().len(), 1, "{leftover}");
        }
    }

    #[test]
    fn test_create_backup_req901_d7_removes_only_conforming_stale_partial() {
        // REQ-901 / MNT-01-D7 / 71 §71.4 手順 3: 規約に合う作業名だけを消す
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();
        let stale = backup_dir.join(partial_name("inventory_backup_20200101_000000.db"));
        let other = backup_dir.join("notes.db.partial");
        std::fs::write(&stale, b"synthetic stale").unwrap();
        std::fs::write(&other, b"synthetic other use").unwrap();

        create_backup(&conn, &backup_dir).unwrap();

        assert!(!stale.exists(), "規約に合う残りの作業名は消す");
        assert_eq!(std::fs::read(&other).unwrap(), b"synthetic other use");
    }

    #[test]
    fn test_create_backup_req901_d7_publish_does_not_replace_existing() {
        // REQ-901 / MNT-01-D7 / 71 §71.4.1: 同じ秒の 2 回目は既存の正式名を残して失敗する
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();
        let now = fixed_now("120000");
        let existing = backup_dir.join(name_at(now));
        std::fs::write(&existing, b"synthetic verified backup").unwrap();

        let result = create_backup_with(&conn, &backup_dir, &StdRestoreFileOps, now);

        assert!(matches!(result, Err(DbError::QueryFailed(_))));
        assert_eq!(
            std::fs::read(&existing).unwrap(),
            b"synthetic verified backup"
        );
        assert_eq!(
            names_in(&backup_dir),
            vec![name_at(now)],
            "作業名が残らない"
        );
        assert_eq!(backup_create_logs(&conn), 0);
    }

    #[test]
    fn test_create_backup_req901_d7_publish_order_and_failure_points() {
        // REQ-901 / MNT-01-D7 / 71 §71.4 手順 4b・§71.4.1: sync → 公開 → 親の sync の順と、各点の失敗
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("ok");
        let ops = PublishOps::new(None);
        create_backup_with(&conn, &backup_dir, &ops, fixed_now("130000")).unwrap();
        assert_eq!(
            *ops.events.lock().unwrap(),
            vec!["sync_file", "publish_no_replace", "sync_parent"]
        );

        for (index, point) in ["sync_file", "publish_no_replace", "sync_parent"]
            .into_iter()
            .enumerate()
        {
            let (dir, conn) = setup_test_db();
            let backup_dir = dir.path().join("backups");
            let now = fixed_now(&format!("13010{index}"));
            let ops = PublishOps::new(Some(point));

            let result = create_backup_with(&conn, &backup_dir, &ops, now);

            assert!(matches!(result, Err(DbError::QueryFailed(_))), "{point}");
            assert_eq!(backup_create_logs(&conn), 0, "{point}");
            if point == "sync_parent" {
                // 公開の後の失敗: 検査済みの正式名が残る
                assert_eq!(names_in(&backup_dir), vec![name_at(now)], "{point}");
                let inspection = inspect_backup(&backup_dir.join(name_at(now))).unwrap();
                assert!(is_usable_backup(&inspection));
            } else {
                assert_eq!(names_in(&backup_dir), Vec::<String>::new(), "{point}");
            }
        }
    }

    #[test]
    fn test_create_backup_req901_d7_partial_name_invisible_to_previous_filters() {
        // REQ-901 / MNT-01-D7 / packet の Rollback: 作業名は (a) の前の一覧・掃除・今日の判定から見えない
        let (dir, conn) = setup_test_db();
        let backup_dir = dir.path().join("backups");
        let seen = std::rc::Rc::new(std::cell::RefCell::new(None));
        let captured = std::rc::Rc::clone(&seen);
        let _hook = set_step_hook(move |step, path| {
            if step == "vacuumed" {
                *captured.borrow_mut() = Some(path.to_path_buf());
            }
            Ok(())
        });
        let result = create_backup(&conn, &backup_dir).unwrap();
        let partial = seen.borrow().clone().expect("作業名の path");
        let partial = partial.file_name().unwrap().to_str().unwrap().to_string();
        assert_eq!(partial, partial_name(&result.file_name));

        let today_prefix = format!("{BACKUP_PREFIX}{}_", chrono::Local::now().format("%Y%m%d"));
        // (a) の前の今日の判定（前方一致と `.db` の後方一致）
        assert!(!(partial.starts_with(&today_prefix) && partial.ends_with(BACKUP_EXT)));
        // (a) の前の一覧・掃除（規約の完全一致）
        assert_eq!(extract_datetime_from_backup(&partial), None);
        assert_eq!(extract_date_from_backup(&partial), None);
    }

    #[test]
    fn test_create_backup_req901_d10_refuses_register_sd_root() {
        // REQ-901 / MNT-01-D10: 保存先の祖先にレジの SD の folder があれば書かない・消さない
        let dir = tempfile::tempdir().unwrap();
        let (_db_dir, conn) = setup_test_db();
        make_register_sd(dir.path());
        let backup_dir = dir.path().join("backups");
        let before = snapshot(dir.path());

        let created = create_backup(&conn, &backup_dir);
        let cleaned = cleanup_old_backups(&backup_dir, 3);

        assert!(matches!(created, Err(DbError::QueryFailed(_))));
        assert!(cleaned.is_err());
        assert_eq!(snapshot(dir.path()), before, "backups も作らない");
        assert_eq!(backup_create_logs(&conn), 0);
    }

    /// `{tmp}/E` を `{tmp}/vol-a` への link にし、判定の直後に `{tmp}/sd` への link へ替える
    #[cfg(unix)]
    fn swap_drive_after_pin(root: &Path) -> (PathBuf, StepHookGuard) {
        let vol_a = root.join("vol-a");
        let sd = root.join("sd");
        std::fs::create_dir_all(&vol_a).unwrap();
        make_register_sd(&sd);
        let drive = root.join("E");
        std::os::unix::fs::symlink(&vol_a, &drive).unwrap();
        let swapped = drive.clone();
        let guard = set_step_hook(move |step, _| {
            if step == "pinned" {
                std::fs::remove_file(&swapped).unwrap();
                std::os::unix::fs::symlink(&sd, &swapped).unwrap();
            }
            Ok(())
        });
        (drive.join("backups"), guard)
    }

    #[cfg(unix)]
    #[test]
    fn test_create_backup_req901_d10_writes_only_to_pinned_volume() {
        // REQ-901 / MNT-01-D10 / 71 §71.4 手順 0: 判定の後に drive 文字が SD へ移っても固定した volume に書く
        let dir = tempfile::tempdir().unwrap();
        let (_db_dir, conn) = setup_test_db();
        let sd_before = {
            make_register_sd(&dir.path().join("sd"));
            snapshot(&dir.path().join("sd"))
        };
        let (backup_dir, _hook) = swap_drive_after_pin(dir.path());

        let result = create_backup(&conn, &backup_dir).unwrap();

        assert_eq!(snapshot(&dir.path().join("sd")), sd_before);
        assert_eq!(
            names_in(&dir.path().join("vol-a/backups")),
            vec![result.file_name.clone()]
        );
        assert_eq!(
            result.file_path,
            backup_dir.join(&result.file_name).to_string_lossy()
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_cleanup_old_backups_req901_d10_deletes_only_on_pinned_volume() {
        // REQ-901 / MNT-01-D10 / 71 §71.5 手順 0: 掃除も固定した volume でだけ消す
        let dir = tempfile::tempdir().unwrap();
        make_register_sd(&dir.path().join("sd"));
        let pinned_old = write_backup_days_old(&dir.path().join("vol-a/backups"), 10);
        write_backup_days_old(&dir.path().join("sd/backups"), 10);
        let sd_before = snapshot(&dir.path().join("sd"));
        let (backup_dir, _hook) = swap_drive_after_pin(dir.path());

        let deleted = cleanup_old_backups(&backup_dir, 3).unwrap();

        assert_eq!(deleted, 1);
        assert!(!pinned_old.exists());
        assert_eq!(snapshot(&dir.path().join("sd")), sd_before);
    }

    #[cfg(unix)]
    #[test]
    fn test_pin_backup_dir_req901_d10_follows_links_to_actual_folder() {
        // REQ-901 / MNT-01-D10: link を含む保存先は実体の folder に作り、元と実体の両方の祖先で SD を拒む
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap();
        let (_db_dir, conn) = setup_test_db();

        // (1) 別の folder への link を含む保存先
        std::fs::create_dir_all(root.join("d/store")).unwrap();
        std::fs::create_dir_all(root.join("c")).unwrap();
        symlink(root.join("d/store"), root.join("c/backup-link")).unwrap();
        let result = create_backup(&conn, &root.join("c/backup-link/new")).unwrap();
        assert_eq!(names_in(&root.join("d/store/new")), vec![result.file_name]);

        // (2) link の先が SD を模した folder の上
        let sd = make_register_sd(&root.join("sd"));
        std::fs::create_dir_all(root.join("sd/store")).unwrap();
        std::fs::create_dir_all(root.join("c2")).unwrap();
        symlink(root.join("sd/store"), root.join("c2/backup-link")).unwrap();
        let sd_before = snapshot(&root.join("sd"));
        let target = root.join("c2/backup-link/new");
        assert_eq!(
            pin_backup_dir(&target),
            Err(BackupDirError::RegisterSd {
                sd_folder: sd.clone()
            })
        );
        assert!(create_backup(&conn, &target).is_err());
        assert!(cleanup_old_backups(&target, 3).is_err());
        assert_eq!(snapshot(&root.join("sd")), sd_before);

        // (3) link 自身が SD を模した folder の中にある
        let sd3 = make_register_sd(&root.join("sd3"));
        std::fs::create_dir_all(root.join("elsewhere")).unwrap();
        symlink(root.join("elsewhere"), root.join("sd3/link")).unwrap();
        let before = snapshot(&root.join("elsewhere"));
        let target = root.join("sd3/link/new");
        assert_eq!(
            pin_backup_dir(&target),
            Err(BackupDirError::RegisterSd { sd_folder: sd3 })
        );
        assert!(create_backup(&conn, &target).is_err());
        assert_eq!(snapshot(&root.join("elsewhere")), before);
    }

    #[test]
    fn test_pin_to_volume_req901_d10_normalizes_before_pinning() {
        // REQ-901 / MNT-01-D10 / 71 §71.4 の (1): 正規化してから固定し、判定と作成を同じ folder にする
        let dir = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap();
        let (_db_dir, conn) = setup_test_db();
        let configured = PathBuf::from(format!("{}/a/b/../c/./d", root.display()));

        let pinned = pin_to_volume(&configured).unwrap();
        assert_eq!(pinned.normalized, root.join("a/c/d"));
        assert_eq!(pinned.actual, root.join("a/c/d"));

        let result = create_backup(&conn, &configured).unwrap();
        assert_eq!(names_in(&root.join("a/c/d")), vec![result.file_name]);
        assert!(!root.join("a/b").exists(), "正規化の前の folder を作らない");
    }

    #[test]
    fn test_backup_names_req901_multibyte_nonconforming_names_do_not_panic() {
        // REQ-901 / 71 §71.3: stem が 15 bytes の多バイトの名前で、一覧・掃除・今日の判定・作業名の掃除が panic しない
        let (dir, conn) = setup_test_db();
        db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();
        db::system_repo::upsert_setting(&conn, "backup_time", "").unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();
        let odd = format!("{BACKUP_PREFIX}あああああ{BACKUP_EXT}");
        assert_eq!("あああああ".len(), 15, "fixture の前提");
        let odd_partial = partial_name(&odd);
        std::fs::write(backup_dir.join(&odd), b"synthetic").unwrap();
        std::fs::write(backup_dir.join(&odd_partial), b"synthetic").unwrap();

        assert_eq!(extract_datetime_from_backup(&odd), None);
        assert_eq!(extract_date_from_backup(&odd), None);
        assert!(list_backups(&backup_dir).unwrap().is_empty());
        assert_eq!(cleanup_old_backups(&backup_dir, 0).unwrap(), 0);
        // 今日の判定 → create_backup（作業名の掃除）→ 掃除
        assert!(check_auto_backup(&conn, &backup_dir).unwrap());
        assert_eq!(list_backups(&backup_dir).unwrap().len(), 1);
        assert!(backup_dir.join(&odd).exists());
        assert!(backup_dir.join(&odd_partial).exists());
    }

    /// chrono の `%Y%m%d`・`%H%M%S` が受ける・受けうる、数字でない文字を含む 15 bytes の stem
    /// （空白は日付・時刻のどちらでも受ける。`+`・`-` は `+202 1 1` の形の日付で受ける。
    /// 全角数字は chrono が受けないが、数字の判定の側で落とすことを固定する）
    fn nondigit_stamps() -> Vec<String> {
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        vec![
            "20200101_12 100".to_string(),
            format!("{today}_12 100"),
            "2026 101_120000".to_string(),
            "+202 1 1_120000".to_string(),
            "-202 1 1_120000".to_string(),
            "２０01_120000".to_string(),
        ]
    }

    #[test]
    fn test_backup_names_req901_nondigit_stamp_is_not_conforming() {
        // REQ-901 / 71 §71.3: 日付・時刻の 14 文字が ASCII の数字でない名前は規約に合わない
        for stamp in nondigit_stamps() {
            assert_eq!(stamp.len(), 15, "fixture の前提 {stamp:?}");
            let name = format!("{BACKUP_PREFIX}{stamp}{BACKUP_EXT}");
            assert_eq!(extract_datetime_from_backup(&name), None, "{name:?}");
            assert_eq!(extract_date_from_backup(&name), None, "{name:?}");
        }
    }

    #[test]
    fn test_backup_names_req901_d7_nondigit_stamp_not_removed_or_counted() {
        // REQ-901 / MNT-01-D7 / 71 §71.4 手順 3・§71.5・§71.8 手順 3: 数字でない日時の名前を
        // 作業名の掃除・保持日数の掃除で消さず、今日の判定に数えない
        let (dir, conn) = setup_test_db();
        db::system_repo::upsert_setting(&conn, "backup_enabled", "1").unwrap();
        db::system_repo::upsert_setting(&conn, "backup_time", "").unwrap();
        let backup_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backup_dir).unwrap();
        let mut leftovers = Vec::new();
        for stamp in nondigit_stamps() {
            let name = format!("{BACKUP_PREFIX}{stamp}{BACKUP_EXT}");
            leftovers.push(partial_name(&name));
            leftovers.push(name);
        }
        for name in &leftovers {
            std::fs::write(backup_dir.join(name), b"synthetic").unwrap();
        }

        assert_eq!(cleanup_old_backups(&backup_dir, 0).unwrap(), 0);
        assert!(list_backups(&backup_dir).unwrap().is_empty());
        // 今日の判定 → create_backup（作業名の掃除）→ 保持日数の掃除
        assert!(check_auto_backup(&conn, &backup_dir).unwrap());
        assert_eq!(list_backups(&backup_dir).unwrap().len(), 1);
        for name in &leftovers {
            assert!(
                backup_dir.join(name).exists(),
                "{name:?} を消してはならない"
            );
        }
    }

    #[test]
    fn test_inspect_backup_req901_d9_newer_than_app() {
        // REQ-901 / MNT-01-D9 / 71 §71.12 手順 2・2a: 新しすぎる版は表を見ずに newer_than_app
        let dir = tempfile::tempdir().unwrap();
        let newer = dir.path().join("newer.db");
        let version = db::test_support::write_newer_schema_db(&newer);
        let inspection = inspect_backup(&newer).unwrap();
        assert_eq!(inspection.schema_version, version);
        assert_eq!(inspection.app_max_version, db::migration::app_max_version());
        assert!(inspection.newer_than_app);
        assert!(!is_usable_backup(&inspection));

        // 版が i64::MAX で、アプリの表を持たない DB
        let max = dir.path().join("max.db");
        let conn = rusqlite::Connection::open(&max).unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO schema_versions (version, applied_at) VALUES (?1, '2026-09-25T00:00:00')",
            [i64::MAX],
        )
        .unwrap();
        drop(conn);
        let inspection = inspect_backup(&max).unwrap();
        assert_eq!(inspection.schema_version, i64::MAX);
        assert!(inspection.newer_than_app);
        assert_eq!(inspection.product_count, None);
    }

    #[test]
    fn test_inspect_backup_req901_d9_quick_check_failure() {
        // REQ-901 / MNT-01-D9 / 71 §71.12 手順 3・4: 壊れた控えは quick_check_ok = false で中身を返さない
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.db");
        app_db(&path, |_| {});
        corrupt_products_page(&path);

        let inspection = inspect_backup(&path).unwrap();

        assert!(!inspection.newer_than_app);
        assert!(!inspection.quick_check_ok);
        assert_eq!(inspection.product_count, None);
        assert_eq!(inspection.last_operation_at, None);
        assert!(!is_usable_backup(&inspection));
    }

    #[test]
    fn test_inspect_backup_req901_d9_leaves_file_and_folder_unchanged() {
        // REQ-901 / MNT-01-D9 / 71 §71.12 手順 1: WAL の header の DB を開いても file と folder を変えない
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wal.db");
        app_db(&path, |_| {});
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(
            (bytes[18], bytes[19]),
            (2, 2),
            "fixture の前提: header が WAL"
        );
        let before = snapshot(dir.path());

        let inspection = inspect_backup(&path).unwrap();

        assert!(is_usable_backup(&inspection));
        assert_eq!(
            snapshot(dir.path()),
            before,
            "-wal・-shm・journal を作らない"
        );
    }

    #[test]
    fn test_inspect_backup_req901_d9_uri_escapes_path_chars() {
        // REQ-901 / MNT-01-D9 / 71 §71.12 手順 1: `%`・`?`・`#` を 1 回の走査で符号化し、`\` は触らない
        let dir = tempfile::tempdir().unwrap();
        let plain = dir.path().join("plain.db");
        app_db(&plain, |_| {});
        // Linux では `\` を含む 1 つの合法な名前（URI の解析は OS に依らない）
        let odd_name = r"\\?\Volume{x}\d#1\50% done\x.db";
        // Linux の `\` は区切りでないので dir の下の 1 つの file 名になる（lint は Windows の区切りで見る）
        #[allow(clippy::join_absolute_paths)]
        let odd = dir.path().join(odd_name);
        std::fs::rename(&plain, &odd).unwrap();
        let before = snapshot(dir.path());

        let inspection = inspect_backup(&odd).unwrap();

        assert_eq!(inspection.schema_version, db::migration::app_max_version());
        assert!(inspection.quick_check_ok);
        assert_eq!(snapshot(dir.path()), before);
        assert_eq!(
            sqlite_uri_path(r"\\?\Volume{x}\d#1\50% done\x.db"),
            r"\\%3f\Volume{x}\d%231\50%25 done\x.db"
        );
    }

    #[test]
    fn test_inspect_backup_req901_d9_unreadable_is_error() {
        // REQ-901 / MNT-01-D9 / 71 §71.12 エラーハンドリング: 読めない file は Err で、file を作らない
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.db");
        let garbage = dir.path().join("garbage.db");
        std::fs::write(&garbage, vec![0x5Au8; 8192]).unwrap();
        let before = snapshot(dir.path());

        assert!(inspect_backup(&missing).is_err());
        assert!(inspect_backup(&garbage).is_err());

        assert!(!missing.exists(), "存在しない path に file を作らない");
        assert_eq!(snapshot(dir.path()), before);
    }

    #[test]
    fn test_inspect_backup_req901_d9_non_app_database_is_error() {
        // REQ-901 / MNT-01-D9 / 71 §71.12 手順 2a: 空の・別用途の DB は読めない控え
        let dir = tempfile::tempdir().unwrap();
        let empty = dir.path().join("empty.db");
        std::fs::write(&empty, b"").unwrap();
        let other = dir.path().join("other.db");
        rusqlite::Connection::open(&other)
            .unwrap()
            .execute_batch("CREATE TABLE notes (body TEXT);")
            .unwrap();
        let no_rows = dir.path().join("no-rows.db");
        rusqlite::Connection::open(&no_rows)
            .unwrap()
            .execute_batch(
                "CREATE TABLE schema_versions (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);",
            )
            .unwrap();
        let v1_without_products = dir.path().join("v1.db");
        rusqlite::Connection::open(&v1_without_products)
            .unwrap()
            .execute_batch(
                "CREATE TABLE schema_versions (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
                 INSERT INTO schema_versions VALUES (1, '2026-01-01T00:00:00');
                 CREATE TABLE operation_logs (id INTEGER PRIMARY KEY, created_at TEXT NOT NULL);
                 CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
            )
            .unwrap();

        for path in [&empty, &other, &no_rows, &v1_without_products] {
            assert!(
                matches!(inspect_backup(path), Err(DbError::QueryFailed(_))),
                "{}",
                path.display()
            );
        }
    }

    #[test]
    fn test_inspect_backup_req901_d9_counts_products_and_latest_operation() {
        // REQ-901 / MNT-01-D9 / 71 §71.12 手順 4: 商品の数と operation_logs.created_at の最大
        let dir = tempfile::tempdir().unwrap();
        let empty = dir.path().join("empty-app.db");
        app_db(&empty, |conn| {
            conn.execute("DELETE FROM operation_logs", []).unwrap();
            let products: i64 = conn
                .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
                .unwrap();
            assert_eq!(products, 0, "fixture の前提");
        });
        let inspection = inspect_backup(&empty).unwrap();
        assert!(is_usable_backup(&inspection));
        assert_eq!(inspection.product_count, Some(0));
        assert_eq!(inspection.last_operation_at, None);

        let filled = dir.path().join("filled-app.db");
        app_db(&filled, |conn| {
            conn.execute("DELETE FROM operation_logs", []).unwrap();
            for code in ["P-1", "P-2", "P-3"] {
                db::test_support::seed_product(conn, code);
            }
            for created_at in [
                "2026-01-02T09:00:00",
                "2026-03-04T18:30:00",
                "2026-02-01T12:00:00",
            ] {
                conn.execute(
                    "INSERT INTO operation_logs (operation_type, summary, created_at) \
                     VALUES ('synthetic', 'synthetic', ?1)",
                    [created_at],
                )
                .unwrap();
            }
        });
        let inspection = inspect_backup(&filled).unwrap();
        assert_eq!(inspection.product_count, Some(3));
        assert_eq!(
            inspection.last_operation_at.as_deref(),
            Some("2026-03-04T18:30:00")
        );
    }
}
