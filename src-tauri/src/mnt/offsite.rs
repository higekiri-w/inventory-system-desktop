//! MNT-01-D8・D11: PC の外の控え（D-114、D-130）
//!
//! docs/function-design/71-mnt-backup.md §71.11 に基づく実装。
//! 用意した取外し可能な媒体（目印と PC 側の状態の両方で見分ける）へ、確かめ済みの最新の backup を
//! 作業名へ写し、読み戻しの SHA-256 で照合してから上書きしない公開で正式名にする。状態は DB の外。

use super::backup::{
    extract_date_from_backup, extract_datetime_from_backup, inspect_backup, is_usable_backup,
    list_backups, partial_name, pin_to_volume, BackupInfo,
};
use super::restore::{RestoreFileOps, StdRestoreFileOps};
use crate::io::register_sd::{removable_drive_roots, ErrorModeGuard};
use crate::io::REGISTER_SD_FOLDER;
use chrono::{NaiveDate, NaiveDateTime};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

// ---------------------------------------------------------------------------
// 定数（§71.11.1）
// ---------------------------------------------------------------------------

const OFFSITE_DIR_NAME: &str = "InventoryBackup";
/// 媒体ごとに残す写しの数（新しい順）
const OFFSITE_KEEP: usize = 30;
/// これ以上前なら「控えが古い」
const OFFSITE_STALE_DAYS: i64 = 3;
const MARKER_FILE: &str = "offsite-medium.json";
const STATE_FILE: &str = "offsite-backup.json";
/// 目印・状態の file の作業名の接尾辞（`.db` で終わらないので backup の規約に掛からない）
const JSON_TEMP_SUFFIX: &str = ".tmp";
const LABEL_PREFIX: &str = "控え ";
const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const STAMP_FORMAT: &str = "%Y%m%d_%H%M%S";

/// §71.11.4 の排他。ponytail: process 全体の lock。媒体ごとの lock は複数の媒体を同時に扱う要件が出たとき
static OFFSITE_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// 型（§71.11.1・§71.11.2・§71.11.4・§71.11.5、43 の DTO の形）
// ---------------------------------------------------------------------------

/// 最後の確認の失敗の種類（wire は snake_case）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum OffsiteFailureKind {
    RegisterSd,
    NotRemovable,
    StorageFull,
    VerifyMismatch,
    Io,
    StateUnreadable,
    SourceUnverified,
    BackupDirIsMediumFolder,
}

/// mnt::offsite の関数が返す失敗（§71.11.1。CMD が 43 の表の固定の文へ写す）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OffsiteError {
    RegisterSd { sd_folder: PathBuf },
    NotRemovable,
    StorageFull,
    VerifyMismatch,
    Io,
    StateUnreadable,
    SourceUnverified,
    BackupDirIsMediumFolder,
    MarkerUnsupported,
    Redirected,
    StateUnsupported,
    NoMediumAttached,
    StateResetFailed,
    StopFailed,
}

impl OffsiteError {
    /// 同じ名前の 8 つは `Some(同じ値)`、用意・作り直し・やめるだけの 6 つは `None`
    pub(crate) fn kind(&self) -> Option<OffsiteFailureKind> {
        match self {
            Self::RegisterSd { .. } => Some(OffsiteFailureKind::RegisterSd),
            Self::NotRemovable => Some(OffsiteFailureKind::NotRemovable),
            Self::StorageFull => Some(OffsiteFailureKind::StorageFull),
            Self::VerifyMismatch => Some(OffsiteFailureKind::VerifyMismatch),
            Self::Io => Some(OffsiteFailureKind::Io),
            Self::StateUnreadable => Some(OffsiteFailureKind::StateUnreadable),
            Self::SourceUnverified => Some(OffsiteFailureKind::SourceUnverified),
            Self::BackupDirIsMediumFolder => Some(OffsiteFailureKind::BackupDirIsMediumFolder),
            Self::MarkerUnsupported
            | Self::Redirected
            | Self::StateUnsupported
            | Self::NoMediumAttached
            | Self::StateResetFailed
            | Self::StopFailed => None,
        }
    }
}

/// 用意した媒体の見え方（§71.11.2）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct OffsiteMediumView {
    pub label: String,
    pub drive_root: String,
    /// 媒体の `InventoryBackup\` の最新の正式名（reparse point の entry を除く）
    pub newest_copy: Option<String>,
}

/// 確認の結果（§71.11.4。wire は `kind` で分ける tagged enum）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OffsiteCheckResult {
    NotPrepared,
    NoLocalBackup,
    MediumMissing,
    UpToDate,
    Copied {
        file_name: String,
        labels: Vec<String>,
    },
}

/// 画面とホームが出す状態（§71.11.5）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct OffsiteBackupStatus {
    pub prepared: bool,
    pub last_success_at: Option<String>,
    pub last_success_label: Option<String>,
    pub days_since_last_success: Option<i64>,
    pub stale: bool,
    pub attached: Vec<OffsiteMediumView>,
    pub last_failure_kind: Option<OffsiteFailureKind>,
    pub local_backup_old: bool,
}

/// 媒体の目印（`{root}\InventoryBackup\offsite-medium.json`）。wire に出さない
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct OffsiteMediumMarker {
    format: u32,
    medium_id: String,
    label: String,
    #[serde(default)]
    prepared_at: String,
}

/// PC 側の状態（`{app_data_dir}\offsite-backup.json`）。wire に出さない
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct OffsiteState {
    format: u32,
    media: Vec<OffsiteMediumEntry>,
    last_success: Option<OffsiteCopyRecord>,
    last_failure: Option<OffsiteFailureRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct OffsiteMediumEntry {
    medium_id: String,
    label: String,
    prepared_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct OffsiteCopyRecord {
    at: String,
    file_name: String,
    medium_id: String,
    sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct OffsiteFailureRecord {
    at: String,
    kind: OffsiteFailureKind,
    medium_id: Option<String>,
}

impl OffsiteState {
    fn empty() -> Self {
        Self {
            format: 1,
            media: Vec::new(),
            last_success: None,
            last_failure: None,
        }
    }
}

// ---------------------------------------------------------------------------
// 注入境界（§71.11.3 の root の列、§71.11.4 の排他の test）
// ---------------------------------------------------------------------------

/// 公開関数と test が共有する内部入口へ渡すもの。本番は取外し可能な root の列・今の時刻・std の file 操作
struct Env<'a> {
    /// 取外し可能な drive の root の列（`E:\` の形）
    roots: Vec<PathBuf>,
    now: NaiveDateTime,
    ops: &'a dyn RestoreFileOps,
    /// test の観測点（lock の取得の前後・写す直前・公開の直前・掃除の直前）。本番は何もしない
    observe: &'a dyn Fn(&'static str),
}

#[cfg(test)]
type TestObserve = Box<dyn Fn(&'static str)>;

#[cfg(test)]
thread_local! {
    static TEST_ROOTS: std::cell::RefCell<Option<Vec<PathBuf>>> = const { std::cell::RefCell::new(None) };
    static TEST_OBSERVE: std::cell::RefCell<Option<TestObserve>> = const { std::cell::RefCell::new(None) };
}

/// test が公開関数（CMD の test から）に root の列と観測点を渡す。drop で外す
#[cfg(test)]
pub(crate) struct TestMediaGuard;

#[cfg(test)]
impl Drop for TestMediaGuard {
    fn drop(&mut self) {
        TEST_ROOTS.with(|roots| *roots.borrow_mut() = None);
        TEST_OBSERVE.with(|observe| *observe.borrow_mut() = None);
    }
}

#[cfg(test)]
pub(crate) fn set_test_media(
    roots: Vec<PathBuf>,
    observe: impl Fn(&'static str) + 'static,
) -> TestMediaGuard {
    TEST_ROOTS.with(|slot| *slot.borrow_mut() = Some(roots));
    TEST_OBSERVE.with(|slot| *slot.borrow_mut() = Some(Box::new(observe)));
    TestMediaGuard
}

fn production_roots() -> Vec<PathBuf> {
    #[cfg(test)]
    if let Some(roots) = TEST_ROOTS.with(|roots| roots.borrow().clone()) {
        return roots;
    }
    removable_drive_roots()
}

fn production_observe(_step: &'static str) {
    #[cfg(test)]
    TEST_OBSERVE.with(|observe| {
        if let Some(observe) = observe.borrow().as_ref() {
            observe(_step);
        }
    });
}

/// 本番の入口の `Env`。媒体を探す入口は `production_roots()` を、媒体を使わない入口（やめる）は空の列を渡す
fn with_production_env<T>(roots: Vec<PathBuf>, run: impl FnOnce(&Env) -> T) -> T {
    let env = Env {
        roots,
        now: chrono::Local::now().naive_local(),
        ops: &StdRestoreFileOps,
        observe: &production_observe,
    };
    run(&env)
}

/// §71.11.4 の lock を入口で 1 回だけ取る。守るデータの無い `Mutex<()>` なので poison は受けて続ける
struct OffsiteLock<'a> {
    _guard: MutexGuard<'static, ()>,
    observe: &'a dyn Fn(&'static str),
}

impl Drop for OffsiteLock<'_> {
    fn drop(&mut self) {
        (self.observe)("lock_released");
    }
}

fn lock<'a>(env: &Env<'a>) -> OffsiteLock<'a> {
    (env.observe)("lock_requested");
    let guard = OFFSITE_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    (env.observe)("lock_acquired");
    OffsiteLock {
        _guard: guard,
        observe: env.observe,
    }
}

// ---------------------------------------------------------------------------
// 公開関数（71 のシグネチャ）
// ---------------------------------------------------------------------------

/// USB メモリを PC の外の控えの保存先として用意する（§71.11.2）
pub fn prepare_offsite_medium(
    app_data_dir: &Path,
    backup_dir: &Path,
    selected: &Path,
) -> Result<OffsiteMediumView, OffsiteError> {
    let _mode = ErrorModeGuard::new();
    with_production_env(production_roots(), |env| {
        prepare_with(env, app_data_dir, backup_dir, selected)
    })
}

/// 用意した媒体が見えていれば、まだ写していない最新の backup を写して確かめる（§71.11.4）
pub fn check_offsite_backup(
    app_data_dir: &Path,
    backup_dir: &Path,
) -> Result<OffsiteCheckResult, OffsiteError> {
    let _mode = ErrorModeGuard::new();
    with_production_env(production_roots(), |env| {
        check_with(env, app_data_dir, backup_dir)
    })
}

/// 画面とホームが出す状態を返す。書かない・lock を取らない（§71.11.5）
pub fn offsite_status(
    app_data_dir: &Path,
    backup_dir: &Path,
    today: NaiveDate,
) -> OffsiteBackupStatus {
    let _mode = ErrorModeGuard::new();
    status_with(&production_roots(), app_data_dir, backup_dir, today)
}

/// 読めない記録を作り直す（§71.11.5a）
pub fn reset_offsite_state(
    app_data_dir: &Path,
    backup_dir: &Path,
) -> Result<OffsiteCheckResult, OffsiteError> {
    let _mode = ErrorModeGuard::new();
    with_production_env(production_roots(), |env| {
        reset_with(env, app_data_dir, backup_dir)
    })
}

/// PC の外の控えをやめる（§71.11.5b）。媒体を探さず・読み書きしないので、drive を列挙しない
pub fn stop_offsite_backup(app_data_dir: &Path) -> Result<(), OffsiteError> {
    with_production_env(Vec::new(), |env| stop_with(env, app_data_dir))
}

/// 保存先に選ぼうとする folder を確かめる。書かない・lock を取らない（§71.11.5c）
pub fn check_backup_dir_choice(selected: &Path) -> Result<(), OffsiteError> {
    let _mode = ErrorModeGuard::new();
    choice_with(&production_roots(), selected)
}

// ---------------------------------------------------------------------------
// 内部入口（本番と test が共有）
// ---------------------------------------------------------------------------

fn prepare_with(
    env: &Env<'_>,
    app_data_dir: &Path,
    backup_dir: &Path,
    selected: &Path,
) -> Result<OffsiteMediumView, OffsiteError> {
    let _lock = lock(env);
    // 1. 選んだ path の drive の root
    let root = drive_root(selected, &env.roots);
    // 2. 取外し可能か
    if !env
        .roots
        .iter()
        .any(|candidate| same_root(candidate, &root))
    {
        return Err(OffsiteError::NotRemovable);
    }
    let pinned_root = pin_to_volume(&root).map_err(|error| {
        tracing::warn!(root = %root.display(), error = %error, "媒体の root を固定できません");
        OffsiteError::Io
    })?;
    // 3. レジの SD（MNT-01-D10。書く先の祖先は root だけ）。返す folder は drive 文字の形
    if is_register_sd(&root, &pinned_root.actual) {
        return Err(OffsiteError::RegisterSd {
            sd_folder: register_sd_folder(&root),
        });
    }
    let dir = pinned_root.actual.join(OFFSITE_DIR_NAME);
    let marker_path = dir.join(MARKER_FILE);
    // 3a. 状態を読む（目印を書く前に止める）
    let mut state = match read_state(app_data_dir) {
        StateRead::Missing => OffsiteState::empty(),
        StateRead::Readable(state) => state,
        StateRead::Unsupported | StateRead::Unreadable => {
            return Err(OffsiteError::StateUnreadable)
        }
    };
    // 3b. reparse point を追わない（目印の作業名も。手順 4 の書込みは作業名を辿って開く）
    for path in [&dir, &marker_path, &json_temp_path(&marker_path)] {
        if is_reparse(path).map_err(warn_io("reparse point の判定に失敗"))? {
            return Err(OffsiteError::Redirected);
        }
    }
    // 3c. PC の中の保存先と同じ folder（MNT-01-D11。folder がまだ無くても止める）
    if is_backup_dir(backup_dir, &dir) {
        return Err(OffsiteError::BackupDirIsMediumFolder);
    }
    // 4. 目印
    let marker = match read_marker(&marker_path).map_err(warn_io("目印の読取りに失敗"))? {
        MarkerRead::Usable(marker) => marker,
        MarkerRead::Unsupported => return Err(OffsiteError::MarkerUnsupported),
        MarkerRead::Rewrite => {
            std::fs::create_dir_all(&dir).map_err(medium_error)?;
            let marker = OffsiteMediumMarker {
                format: 1,
                medium_id: uuid::Uuid::new_v4().to_string(),
                label: next_label(&state.media),
                prepared_at: env.now.format(DATETIME_FORMAT).to_string(),
            };
            publish_json(env.ops, &marker_path, &marker)
                .map_err(|error| medium_error(error.into_io()))?;
            marker
        }
    };
    // 5. 状態に足す（同じ medium_id があれば変えない）
    if !state
        .media
        .iter()
        .any(|entry| entry.medium_id == marker.medium_id)
    {
        state.media.push(OffsiteMediumEntry {
            medium_id: marker.medium_id.clone(),
            label: marker.label.clone(),
            prepared_at: marker.prepared_at.clone(),
        });
        // PC 側の状態の書込みの失敗は空き不足を含め Io（StorageFull は媒体に限る）
        publish_json(env.ops, &state_path(app_data_dir), &state).map_err(|error| {
            tracing::warn!(error = %error.into_io(), "PC の外の控えの状態の書込みに失敗");
            OffsiteError::Io
        })?;
    }
    let label = state
        .media
        .iter()
        .find(|entry| entry.medium_id == marker.medium_id)
        .map_or_else(|| marker.label.clone(), |entry| entry.label.clone());
    // 6. 続けて確認を 1 回（lock を取らない本体。失敗は last_failure に残り、用意を取り消さない）
    if let Err(error) = check_body(env, app_data_dir, backup_dir) {
        tracing::warn!(error = ?error, "用意の直後の写しに失敗");
    }
    // 7.
    Ok(OffsiteMediumView {
        label,
        drive_root: root.to_string_lossy().into_owned(),
        newest_copy: newest_copy(&dir),
    })
}

fn check_with(
    env: &Env<'_>,
    app_data_dir: &Path,
    backup_dir: &Path,
) -> Result<OffsiteCheckResult, OffsiteError> {
    let _lock = lock(env);
    check_body(env, app_data_dir, backup_dir)
}

/// §71.11.4 の本体（lock を取らない。用意の手順 6・作り直しの手順 7 もここを呼ぶ）
fn check_body(
    env: &Env<'_>,
    app_data_dir: &Path,
    backup_dir: &Path,
) -> Result<OffsiteCheckResult, OffsiteError> {
    // 1. 状態
    let mut state = match read_state(app_data_dir) {
        StateRead::Missing => return Ok(OffsiteCheckResult::NotPrepared),
        StateRead::Readable(state) => state,
        StateRead::Unsupported | StateRead::Unreadable => {
            return Err(OffsiteError::StateUnreadable)
        }
    };
    if state.media.is_empty() {
        return Ok(OffsiteCheckResult::NotPrepared);
    }
    // 2. 写す元（正式名だけ）
    let source = match list_backups(backup_dir) {
        Ok(backups) => match backups.into_iter().next() {
            Some(source) => source,
            None => return Ok(OffsiteCheckResult::NoLocalBackup),
        },
        Err(error) => {
            tracing::warn!(error = %error, "写す元の一覧に失敗");
            return Err(record_failure(
                env,
                app_data_dir,
                &mut state,
                OffsiteError::Io,
                None,
            ));
        }
    };
    // 3. 用意済みの媒体
    let media: Vec<Medium> = find_media(&env.roots)
        .into_iter()
        .filter(|medium| {
            state
                .media
                .iter()
                .any(|entry| entry.medium_id == medium.marker.medium_id)
        })
        .collect();
    if media.is_empty() {
        return Ok(OffsiteCheckResult::MediumMissing);
    }
    if let Some(medium) = media
        .iter()
        .find(|medium| is_backup_dir(backup_dir, &medium.dir))
    {
        let medium_id = Some(medium.marker.medium_id.clone());
        return Err(record_failure(
            env,
            app_data_dir,
            &mut state,
            OffsiteError::BackupDirIsMediumFolder,
            medium_id,
        ));
    }
    // 3a. 写す必要があるときだけ写す元を共有の検査にかける
    let source_path = Path::new(&source.file_path);
    let needs_copy = media
        .iter()
        .any(|medium| !has_same_copy(&medium.dir, &source));
    if !needs_copy {
        return Ok(OffsiteCheckResult::UpToDate);
    }
    match inspect_backup(source_path) {
        Ok(inspection) if is_usable_backup(&inspection) => {}
        result => {
            tracing::warn!(result = ?result.map(|_| ()), "写す元が共有の検査に通りません");
            return Err(record_failure(
                env,
                app_data_dir,
                &mut state,
                OffsiteError::SourceUnverified,
                None,
            ));
        }
    }
    // 4. 媒体ごと
    let mut source_sha256: Option<String> = None;
    let mut copied: Vec<&Medium> = Vec::new();
    let mut failure: Option<(OffsiteError, String)> = None;
    for medium in &media {
        if has_same_copy(&medium.dir, &source) {
            continue;
        }
        let sha256 = match &source_sha256 {
            Some(sha256) => sha256.clone(),
            None => {
                // 4c. 元の file の SHA-256
                let bytes = env
                    .ops
                    .read_file(source_path)
                    .map_err(warn_io("写す元の読取りに失敗"))
                    .map_err(|error| record_failure(env, app_data_dir, &mut state, error, None))?;
                let sha256 = sha256_hex(&bytes);
                source_sha256 = Some(sha256.clone());
                sha256
            }
        };
        match copy_to_medium(env, medium, source_path, &source.file_name, &sha256) {
            Ok(()) => copied.push(medium),
            Err(error) => failure = Some((error, medium.marker.medium_id.clone())),
        }
    }
    // 5・6. 記録
    let labels: Vec<String> = copied
        .iter()
        .map(|medium| label_of(&state, &medium.marker))
        .collect();
    if let (Some(last), Some(sha256)) = (copied.last(), &source_sha256) {
        state.last_success = Some(OffsiteCopyRecord {
            at: env.now.format(DATETIME_FORMAT).to_string(),
            file_name: source.file_name.clone(),
            medium_id: last.marker.medium_id.clone(),
            sha256: sha256.clone(),
        });
    }
    if let Some((error, medium_id)) = failure {
        return Err(record_failure(
            env,
            app_data_dir,
            &mut state,
            error,
            Some(medium_id),
        ));
    }
    if copied.is_empty() {
        return Ok(OffsiteCheckResult::UpToDate);
    }
    state.last_failure = None;
    match publish_json(env.ops, &state_path(app_data_dir), &state) {
        Ok(()) => {}
        // 置き換えの後の親 directory の sync の失敗は成功のまま（§71.11.4 のエラーの表）
        Err(JsonWriteError::AfterReplace(error)) => {
            tracing::warn!(error = %error, "成功の記録の置き換えの後の sync に失敗");
        }
        Err(JsonWriteError::BeforeReplace(error)) => {
            tracing::warn!(error = %error, "成功の記録の書込みに失敗");
            return Err(OffsiteError::Io);
        }
    }
    Ok(OffsiteCheckResult::Copied {
        file_name: source.file_name,
        labels,
    })
}

/// §71.11.4 手順 4b〜4g（1 つの媒体）
fn copy_to_medium(
    env: &Env<'_>,
    medium: &Medium,
    source_path: &Path,
    file_name: &str,
    sha256: &str,
) -> Result<(), OffsiteError> {
    let dir = &medium.dir;
    let published = dir.join(file_name);
    let partial = dir.join(partial_name(file_name));
    // 4b. 残っている作業名（規約に合う名前・reparse point でないもの）を消す
    for (name, path) in entries_of(dir) {
        let conforming = name
            .strip_suffix(super::backup::PARTIAL_SUFFIX)
            .is_some_and(|stem| extract_datetime_from_backup(stem).is_some());
        if conforming && is_reparse(&path).ok() == Some(false) {
            if let Err(error) = env.ops.remove_file(&path) {
                tracing::warn!(file = %path.display(), error = %error, "残りの作業名の削除に失敗");
            }
        }
    }
    // 4d. 目印と reparse point を確かめ直し、既存なら失敗する copy → sync
    (env.observe)("before_copy");
    verify_medium(medium)?;
    if let Err(error) = env.ops.copy_new(source_path, &partial) {
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            discard_partial(env.ops, &partial);
        }
        return Err(medium_error(error));
    }
    if let Err(error) = env.ops.sync_file(&partial) {
        discard_partial(env.ops, &partial);
        return Err(medium_error(error));
    }
    // 4e. 読み戻しの照合
    // ponytail: file 全体を memory に読む。控えが数百 MB を超えるなら stream で hash する
    let readback = match env.ops.read_file(&partial) {
        Ok(bytes) => bytes,
        Err(error) => {
            discard_partial(env.ops, &partial);
            return Err(medium_error(error));
        }
    };
    if sha256_hex(&readback) != sha256 {
        discard_partial(env.ops, &partial);
        return Err(OffsiteError::VerifyMismatch);
    }
    // 4f. 確かめ直して上書きしない公開 → 親 directory の sync
    (env.observe)("before_publish");
    if let Err(error) = verify_medium(medium) {
        discard_partial(env.ops, &partial);
        return Err(error);
    }
    if let Err(error) = env.ops.publish_no_replace(&partial, &published) {
        discard_partial(env.ops, &partial);
        return Err(medium_error(error));
    }
    // 公開の後の失敗: 検査済みの正式名は残す
    env.ops.sync_parent(&published).map_err(medium_error)?;
    // 4g. 掃除（確かめ直し。消せなければ warn で続ける）
    (env.observe)("before_cleanup");
    verify_medium(medium)?;
    let mut copies: Vec<(String, PathBuf)> = official_copies(dir);
    copies.sort_by(|a, b| b.0.cmp(&a.0));
    for (name, path) in copies.into_iter().skip(OFFSITE_KEEP) {
        if name == file_name {
            continue;
        }
        if let Err(error) = env.ops.remove_file(&path) {
            tracing::warn!(file = %path.display(), error = %error, "媒体の古い写しの削除に失敗");
        }
    }
    Ok(())
}

fn status_with(
    roots: &[PathBuf],
    app_data_dir: &Path,
    backup_dir: &Path,
    today: NaiveDate,
) -> OffsiteBackupStatus {
    let local_backup_old = match list_backups(backup_dir) {
        Ok(backups) => backups
            .first()
            .and_then(|newest| extract_date_from_backup(&newest.file_name))
            .is_none_or(|date| (today - date).num_days() >= OFFSITE_STALE_DAYS),
        // 原因を決めつけない（PC の中の一覧の失敗は画面の一覧が出す）
        Err(_) => false,
    };
    let state = match read_state(app_data_dir) {
        StateRead::Missing => {
            return OffsiteBackupStatus {
                prepared: false,
                last_success_at: None,
                last_success_label: None,
                days_since_last_success: None,
                stale: false,
                attached: Vec::new(),
                last_failure_kind: None,
                local_backup_old,
            }
        }
        StateRead::Readable(state) => state,
        // 読めない記録は知らせる側に倒す
        StateRead::Unsupported | StateRead::Unreadable => {
            return OffsiteBackupStatus {
                prepared: true,
                last_success_at: None,
                last_success_label: None,
                days_since_last_success: None,
                stale: true,
                attached: Vec::new(),
                last_failure_kind: Some(OffsiteFailureKind::StateUnreadable),
                local_backup_old,
            }
        }
    };
    let prepared = !state.media.is_empty();
    let last_success = state.last_success.as_ref();
    let days_since_last_success = last_success.and_then(|record| {
        let copied_on = NaiveDateTime::parse_from_str(&record.at, DATETIME_FORMAT)
            .ok()?
            .date();
        let content_on = extract_date_from_backup(&record.file_name).unwrap_or(copied_on);
        Some((today - copied_on.min(content_on)).num_days())
    });
    let attached = find_media(roots)
        .into_iter()
        .filter_map(|medium| {
            let entry = state
                .media
                .iter()
                .find(|entry| entry.medium_id == medium.marker.medium_id)?;
            Some(OffsiteMediumView {
                label: entry.label.clone(),
                drive_root: medium.drive_root.to_string_lossy().into_owned(),
                newest_copy: newest_copy(&medium.dir),
            })
        })
        .collect();
    OffsiteBackupStatus {
        prepared,
        last_success_at: last_success.map(|record| record.at.clone()),
        last_success_label: last_success.and_then(|record| {
            state
                .media
                .iter()
                .find(|entry| entry.medium_id == record.medium_id)
                .map(|entry| entry.label.clone())
        }),
        days_since_last_success,
        stale: prepared && days_since_last_success.is_none_or(|days| days >= OFFSITE_STALE_DAYS),
        attached,
        last_failure_kind: state.last_failure.as_ref().map(|failure| failure.kind),
        local_backup_old,
    }
}

fn reset_with(
    env: &Env<'_>,
    app_data_dir: &Path,
    backup_dir: &Path,
) -> Result<OffsiteCheckResult, OffsiteError> {
    let _lock = lock(env);
    let path = state_path(app_data_dir);
    // 2. 無い・読める → 作り直さない。新しい版の記録は触らない
    match read_state(app_data_dir) {
        StateRead::Missing | StateRead::Readable(_) => {
            return check_body(env, app_data_dir, backup_dir)
        }
        StateRead::Unsupported => return Err(OffsiteError::StateUnsupported),
        StateRead::Unreadable => {}
    }
    // 3. 目印のある媒体（状態との照合を外す）
    let mut media: Vec<OffsiteMediumEntry> = Vec::new();
    for medium in find_media(&env.roots) {
        if media
            .iter()
            .all(|entry| entry.medium_id != medium.marker.medium_id)
        {
            media.push(OffsiteMediumEntry {
                medium_id: medium.marker.medium_id,
                label: medium.marker.label,
                prepared_at: medium.marker.prepared_at,
            });
        }
    }
    if media.is_empty() {
        return Err(OffsiteError::NoMediumAttached);
    }
    let state = OffsiteState {
        media,
        ..OffsiteState::empty()
    };
    let reset_failed = |step: &'static str| {
        move |error: std::io::Error| {
            tracing::warn!(step, error = %error, "PC の外の控えの記録の作り直しに失敗");
            OffsiteError::StateResetFailed
        }
    };
    // 4. 新しい状態の作業名（前回の中断の作業名は上書きする）
    let temp = json_temp_path(&path);
    let bytes = serde_json::to_vec_pretty(&state)
        .map_err(|error| reset_failed("serialize")(std::io::Error::other(error)))?;
    env.ops
        .write_file(&temp, &bytes)
        .map_err(reset_failed("temp write"))?;
    env.ops
        .sync_file(&temp)
        .map_err(reset_failed("temp sync"))?;
    // 5. 読めない原本を退避名へ copy（既にある名前は上書きしない）
    let aside = sibling_with_suffix(
        &path,
        &format!(".unreadable-{}", env.now.format(STAMP_FORMAT)),
    );
    if let Err(error) = env.ops.copy_new(&path, &aside) {
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            discard_partial(env.ops, &aside);
        }
        return Err(reset_failed("set aside copy")(error));
    }
    env.ops
        .sync_file(&aside)
        .map_err(reset_failed("set aside sync"))?;
    env.ops
        .sync_parent(&aside)
        .map_err(reset_failed("set aside directory sync"))?;
    // 6. 置き換え → 親 directory の sync
    env.ops
        .replace(&temp, &path)
        .map_err(reset_failed("replace"))?;
    env.ops
        .sync_parent(&path)
        .map_err(reset_failed("directory sync"))?;
    // 7.
    check_body(env, app_data_dir, backup_dir)
}

fn stop_with(env: &Env<'_>, app_data_dir: &Path) -> Result<(), OffsiteError> {
    let _lock = lock(env);
    let path = state_path(app_data_dir);
    // 2. 無ければ既に用意の前
    match std::fs::symlink_metadata(&path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            tracing::warn!(error = %error, "PC の外の控えの状態の確認に失敗");
            return Err(OffsiteError::StopFailed);
        }
    }
    // 3. 読める・読めない・format に依らず、上書きしない操作で退避名へ移す → 親 directory の sync
    let stopped = sibling_with_suffix(&path, &format!(".stopped-{}", env.now.format(STAMP_FORMAT)));
    let stop_failed = |error: std::io::Error| {
        tracing::warn!(error = %error, "PC の外の控えをやめる操作に失敗");
        OffsiteError::StopFailed
    };
    env.ops
        .publish_no_replace(&path, &stopped)
        .map_err(stop_failed)?;
    env.ops.sync_parent(&stopped).map_err(stop_failed)?;
    Ok(())
}

fn choice_with(roots: &[PathBuf], selected: &Path) -> Result<(), OffsiteError> {
    // 状態を読まず、目印のある媒体の InventoryBackup を全部止める（読めない root は飛ばす）
    if find_media(roots)
        .iter()
        .any(|medium| is_backup_dir(selected, &medium.dir))
    {
        return Err(OffsiteError::BackupDirIsMediumFolder);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 媒体を探す（§71.11.3）
// ---------------------------------------------------------------------------

struct Medium {
    /// drive 文字の形の root（表示用）
    drive_root: PathBuf,
    /// root を固定した実体の path の `InventoryBackup`（読み書きはこちら）
    dir: PathBuf,
    marker: OffsiteMediumMarker,
}

/// 目印が format 1 で読める媒体（レジの SD・固定できない root・reparse point を除く）
fn find_media(roots: &[PathBuf]) -> Vec<Medium> {
    let mut media = Vec::new();
    for root in roots {
        let Ok(pinned) = pin_to_volume(root) else {
            continue;
        };
        if is_register_sd(root, &pinned.actual) {
            continue;
        }
        let dir = pinned.actual.join(OFFSITE_DIR_NAME);
        let marker_path = dir.join(MARKER_FILE);
        if is_reparse(&dir).ok() != Some(false) || is_reparse(&marker_path).ok() != Some(false) {
            continue;
        }
        if let Ok(MarkerRead::Usable(marker)) = read_marker(&marker_path) {
            media.push(Medium {
                drive_root: root.clone(),
                dir,
                marker,
            });
        }
    }
    media
}

/// 作業名へ写す直前・公開の直前・掃除の直前の確かめ直し（入れ替わり・reparse point は Io）
fn verify_medium(medium: &Medium) -> Result<(), OffsiteError> {
    let marker_path = medium.dir.join(MARKER_FILE);
    let unchanged = is_reparse(&medium.dir).ok() == Some(false)
        && is_reparse(&marker_path).ok() == Some(false)
        && matches!(
            read_marker(&marker_path),
            Ok(MarkerRead::Usable(marker)) if marker.medium_id == medium.marker.medium_id
        );
    if unchanged {
        Ok(())
    } else {
        tracing::warn!(dir = %medium.dir.display(), "媒体の目印が照合した値と違います");
        Err(OffsiteError::Io)
    }
}

/// PC の中の保存先と媒体の InventoryBackup が同じ実体の folder か（MNT-01-D11）。
/// 保存先を固定できなければ同じと見なさない
fn is_backup_dir(backup_dir: &Path, medium_dir: &Path) -> bool {
    match pin_to_volume(backup_dir) {
        Ok(pinned) => same_components(&pinned.actual, medium_dir),
        Err(error) => {
            tracing::warn!(path = %backup_dir.display(), error = %error, "保存先を固定できません");
            false
        }
    }
}

fn same_components(left: &Path, right: &Path) -> bool {
    let mut left = left.components();
    let mut right = right.components();
    loop {
        match (left.next(), right.next()) {
            (None, None) => return true,
            (Some(a), Some(b)) if component_eq(a.as_os_str(), b.as_os_str()) => {}
            _ => return false,
        }
    }
}

#[cfg(windows)]
fn component_eq(a: &std::ffi::OsStr, b: &std::ffi::OsStr) -> bool {
    a.eq_ignore_ascii_case(b)
}

#[cfg(not(windows))]
fn component_eq(a: &std::ffi::OsStr, b: &std::ffi::OsStr) -> bool {
    a == b
}

/// 選んだ path の drive の root（root の下の folder を選んでも root）
#[cfg(windows)]
fn drive_root(selected: &Path, _roots: &[PathBuf]) -> PathBuf {
    use std::path::Component;
    let mut root = PathBuf::new();
    for component in selected.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => root.push(component.as_os_str()),
            _ => break,
        }
    }
    root
}

/// Windows 以外（test）: 注入した root の列のうち、選んだ path を含む最も長いもの
#[cfg(not(windows))]
fn drive_root(selected: &Path, roots: &[PathBuf]) -> PathBuf {
    roots
        .iter()
        .filter(|root| selected.starts_with(root))
        .max_by_key(|root| root.components().count())
        .cloned()
        .unwrap_or_else(|| selected.to_path_buf())
}

#[cfg(windows)]
fn same_root(a: &Path, b: &Path) -> bool {
    a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
}

#[cfg(not(windows))]
fn same_root(a: &Path, b: &Path) -> bool {
    a == b
}

/// root の直下のレジの SD の folder の path（MNT-01-D10）
fn register_sd_folder(root: &Path) -> PathBuf {
    let [casio, model] = REGISTER_SD_FOLDER;
    root.join(casio).join(model)
}

/// drive 文字の root か、それを固定した実体の root に、レジの SD の folder がある。
/// 判定と固定の間に drive 文字の割当てが替わっても、読み書きする volume の側で見る
fn is_register_sd(root: &Path, pinned: &Path) -> bool {
    [root, pinned]
        .iter()
        .any(|root| register_sd_folder(root).is_dir())
}

/// reparse point（junction・symbolic link・mounted folder）か。無ければ false
#[cfg(not(windows))]
fn is_reparse(path: &Path) -> std::io::Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_symlink()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// `GetFileAttributesW` の `FILE_ATTRIBUTE_REPARSE_POINT`（link 自身の属性）。無ければ false
#[cfg(windows)]
fn is_reparse(path: &Path) -> std::io::Result<bool> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileAttributesW, FILE_ATTRIBUTE_REPARSE_POINT, INVALID_FILE_ATTRIBUTES,
    };
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: wide は NUL で終わる UTF-16 の列で、呼出しの間は生きている
    let attributes = unsafe { GetFileAttributesW(wide.as_ptr()) };
    if attributes == INVALID_FILE_ATTRIBUTES {
        let error = std::io::Error::last_os_error();
        return if error.kind() == std::io::ErrorKind::NotFound {
            Ok(false)
        } else {
            Err(error)
        };
    }
    Ok(attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0)
}

// ---------------------------------------------------------------------------
// 媒体の上の file
// ---------------------------------------------------------------------------

/// folder の entry の (名前, path)。名前が UTF-8 でない・読めない entry は飛ばす
fn entries_of(dir: &Path) -> Vec<(String, PathBuf)> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(dir = %dir.display(), error = %error, "媒体の一覧に失敗");
            }
            return Vec::new();
        }
    };
    entries
        .filter_map(|entry| match entry {
            Ok(entry) => entry
                .file_name()
                .into_string()
                .ok()
                .map(|name| (name, entry.path())),
            Err(error) => {
                tracing::warn!(dir = %dir.display(), error = %error, "媒体の一覧に失敗");
                None
            }
        })
        .collect()
}

/// 正式名の写し（§71.3 の規約に完全に合う名前の file、reparse point の entry を除く）
fn official_copies(dir: &Path) -> Vec<(String, PathBuf)> {
    entries_of(dir)
        .into_iter()
        .filter(|(name, path)| {
            extract_datetime_from_backup(name).is_some()
                && is_reparse(path).ok() == Some(false)
                && path.is_file()
        })
        .collect()
}

fn newest_copy(dir: &Path) -> Option<String> {
    official_copies(dir).into_iter().map(|(name, _)| name).max()
}

/// 同じ名前の正式名（reparse point でない file）があり size が同じ（§71.11.4 手順 4a）
fn has_same_copy(dir: &Path, source: &BackupInfo) -> bool {
    let path = dir.join(&source.file_name);
    is_reparse(&path).ok() == Some(false)
        && std::fs::symlink_metadata(&path)
            .is_ok_and(|metadata| metadata.is_file() && metadata.len() == source.size_bytes)
}

fn discard_partial(ops: &dyn RestoreFileOps, partial: &Path) {
    if is_reparse(partial).ok() != Some(false) {
        return;
    }
    match ops.remove_file(partial) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            tracing::warn!(file = %partial.display(), error = %error, "作業名の削除に失敗");
        }
    }
}

fn medium_error(error: std::io::Error) -> OffsiteError {
    tracing::warn!(error = %error, "媒体の読み書きに失敗");
    if error.kind() == std::io::ErrorKind::StorageFull {
        OffsiteError::StorageFull
    } else {
        OffsiteError::Io
    }
}

fn warn_io(context: &'static str) -> impl Fn(std::io::Error) -> OffsiteError {
    move |error| {
        tracing::warn!(error = %error, "{context}");
        OffsiteError::Io
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// ---------------------------------------------------------------------------
// 目印と状態の file（§71.11.1）
// ---------------------------------------------------------------------------

enum MarkerRead {
    Usable(OffsiteMediumMarker),
    /// format が 1 でない数（新しい版のアプリの目印）
    Unsupported,
    /// 無い・JSON として読めない・format が無い・format 1 で medium_id か label が無い
    Rewrite,
}

fn read_marker(path: &Path) -> std::io::Result<MarkerRead> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(MarkerRead::Rewrite)
        }
        Err(error) => return Err(error),
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Ok(MarkerRead::Rewrite);
    };
    match value.get("format") {
        Some(serde_json::Value::Number(format)) if format.as_u64() == Some(1) => {}
        Some(serde_json::Value::Number(_)) => return Ok(MarkerRead::Unsupported),
        _ => return Ok(MarkerRead::Rewrite),
    }
    Ok(match serde_json::from_value::<OffsiteMediumMarker>(value) {
        Ok(marker) if !marker.medium_id.is_empty() && !marker.label.is_empty() => {
            MarkerRead::Usable(marker)
        }
        _ => MarkerRead::Rewrite,
    })
}

enum StateRead {
    Missing,
    Readable(OffsiteState),
    /// format が 1 でない数（新しい版のアプリの記録）
    Unsupported,
    Unreadable,
}

fn state_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(STATE_FILE)
}

fn read_state(app_data_dir: &Path) -> StateRead {
    let bytes = match std::fs::read(state_path(app_data_dir)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return StateRead::Missing,
        Err(error) => {
            tracing::warn!(error = %error, "PC の外の控えの状態を読めません");
            return StateRead::Unreadable;
        }
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return StateRead::Unreadable;
    };
    match value.get("format") {
        Some(serde_json::Value::Number(format)) if format.as_u64() == Some(1) => {}
        Some(serde_json::Value::Number(_)) => return StateRead::Unsupported,
        _ => return StateRead::Unreadable,
    }
    match serde_json::from_value::<OffsiteState>(value) {
        Ok(state) => StateRead::Readable(state),
        Err(_) => StateRead::Unreadable,
    }
}

/// 最後の確認の失敗を状態に残して返す（書けなければ warn。読めない状態には書かない）
fn record_failure(
    env: &Env<'_>,
    app_data_dir: &Path,
    state: &mut OffsiteState,
    error: OffsiteError,
    medium_id: Option<String>,
) -> OffsiteError {
    if let Some(kind) = error.kind() {
        state.last_failure = Some(OffsiteFailureRecord {
            at: env.now.format(DATETIME_FORMAT).to_string(),
            kind,
            medium_id,
        });
        if let Err(write_error) = publish_json(env.ops, &state_path(app_data_dir), state) {
            tracing::warn!(error = %write_error.into_io(), "失敗の記録の書込みに失敗");
        }
    }
    error
}

enum JsonWriteError {
    /// 作業名の書込み・sync・置き換えそのもの（前の内容が残る）
    BeforeReplace(std::io::Error),
    /// 置き換えの後の親 directory の sync（新しい内容で読める）
    AfterReplace(std::io::Error),
}

impl JsonWriteError {
    fn into_io(self) -> std::io::Error {
        match self {
            Self::BeforeReplace(error) | Self::AfterReplace(error) => error,
        }
    }
}

/// 作業名へ書いて sync → 置き換え → 親 directory の sync（restore の manifest と同じ順）
fn publish_json<T: serde::Serialize>(
    ops: &dyn RestoreFileOps,
    path: &Path,
    value: &T,
) -> Result<(), JsonWriteError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| JsonWriteError::BeforeReplace(std::io::Error::other(error)))?;
    let temp = json_temp_path(path);
    ops.write_file(&temp, &bytes)
        .map_err(JsonWriteError::BeforeReplace)?;
    ops.sync_file(&temp)
        .map_err(JsonWriteError::BeforeReplace)?;
    ops.replace(&temp, path)
        .map_err(JsonWriteError::BeforeReplace)?;
    ops.sync_parent(path).map_err(JsonWriteError::AfterReplace)
}

fn json_temp_path(path: &Path) -> PathBuf {
    sibling_with_suffix(path, JSON_TEMP_SUFFIX)
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// 「控え N」の N = 状態の札の最大 + 1（数 + 1 にすると失った札と重なる）
fn next_label(media: &[OffsiteMediumEntry]) -> String {
    let max = media
        .iter()
        .filter_map(|entry| entry.label.strip_prefix(LABEL_PREFIX)?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("{LABEL_PREFIX}{}", max + 1)
}

fn label_of(state: &OffsiteState, marker: &OffsiteMediumMarker) -> String {
    state
        .media
        .iter()
        .find(|entry| entry.medium_id == marker.medium_id)
        .map_or_else(|| marker.label.clone(), |entry| entry.label.clone())
}

// ---------------------------------------------------------------------------
// テスト（Test Design Matrix 2026-10-23-backup-offsite-runtime-b。合成の DB・目印・状態だけ）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::collections::BTreeMap;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex as StdMutex, OnceLock};
    use std::time::Duration;

    // ---- fixture -----------------------------------------------------------

    fn now() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2026-11-02 09:00:00", DATETIME_FORMAT).unwrap()
    }

    fn today() -> NaiveDate {
        now().date()
    }

    fn noop(_: &'static str) {}

    fn std_env(roots: &[PathBuf]) -> Env<'static> {
        Env {
            roots: roots.to_vec(),
            now: now(),
            ops: &StdRestoreFileOps,
            observe: &noop,
        }
    }

    fn env_with<'a>(roots: &[PathBuf], ops: &'a dyn RestoreFileOps) -> Env<'a> {
        Env {
            roots: roots.to_vec(),
            now: now(),
            ops,
            observe: &noop,
        }
    }

    struct Fx {
        _tmp: tempfile::TempDir,
        base: PathBuf,
        app: PathBuf,
        local: PathBuf,
        root: PathBuf,
    }

    impl Fx {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let base = std::fs::canonicalize(tmp.path()).unwrap();
            let app = base.join("app");
            let local = base.join("local");
            let root = base.join("E");
            for dir in [&app, &local, &root] {
                std::fs::create_dir_all(dir).unwrap();
            }
            Self {
                _tmp: tmp,
                base,
                app,
                local,
                root,
            }
        }
        fn roots(&self) -> Vec<PathBuf> {
            vec![self.root.clone()]
        }
        fn medium(&self) -> PathBuf {
            self.root.join(OFFSITE_DIR_NAME)
        }
        fn state_file(&self) -> PathBuf {
            self.app.join(STATE_FILE)
        }
        fn prepare(&self) -> OffsiteMediumView {
            prepare_with(&std_env(&self.roots()), &self.app, &self.local, &self.root).unwrap()
        }
        fn check(&self) -> Result<OffsiteCheckResult, OffsiteError> {
            check_with(&std_env(&self.roots()), &self.app, &self.local)
        }
        fn state(&self) -> OffsiteState {
            match read_state(&self.app) {
                StateRead::Readable(state) => state,
                _ => panic!("状態が読めない"),
            }
        }
        fn status(&self) -> OffsiteBackupStatus {
            status_with(&self.roots(), &self.app, &self.local, today())
        }
    }

    /// 使える控えの bytes（アプリの migrate を通した DB を VACUUM INTO したもの）
    fn backup_bytes() -> &'static [u8] {
        static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
        BYTES.get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let conn = db::init_database(dir.path().join("src.db").to_str().unwrap()).unwrap();
            let out = dir.path().join("out.db");
            conn.execute_batch(&format!("VACUUM INTO '{}'", out.display()))
                .unwrap();
            drop(conn);
            std::fs::read(out).unwrap()
        })
    }

    fn name_on(date: NaiveDate, hms: &str) -> String {
        format!("inventory_backup_{}_{hms}.db", date.format("%Y%m%d"))
    }

    fn put_backup(dir: &Path, name: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, backup_bytes()).unwrap();
        path
    }

    fn today_name() -> String {
        name_on(today(), "080000")
    }

    /// folder の中身（file の bytes、link の指す先、directory）を link を辿らずに写す
    fn snapshot(root: &Path) -> BTreeMap<PathBuf, String> {
        fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, String>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
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
                    let bytes = std::fs::read(&path).unwrap();
                    out.insert(relative, sha256_hex(&bytes));
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(root, root, &mut out);
        out
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    fn write_state(fx: &Fx, value: &serde_json::Value) {
        std::fs::write(fx.state_file(), serde_json::to_vec(value).unwrap()).unwrap();
    }

    fn write_marker(root: &Path, value: &serde_json::Value) {
        let dir = root.join(OFFSITE_DIR_NAME);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(MARKER_FILE), serde_json::to_vec(value).unwrap()).unwrap();
    }

    fn make_register_sd(root: &Path) -> PathBuf {
        let [casio, model] = REGISTER_SD_FOLDER;
        let folder = root.join(casio).join(model);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("synthetic.txt"), b"synthetic register file").unwrap();
        folder
    }

    fn sha_of(path: &Path) -> String {
        // oracle は実装と別に計算する
        let digest = Sha256::digest(std::fs::read(path).unwrap());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// `products` の root の page を壊す（schema と版は読めるまま）
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

    // ---- 注入の型 -------------------------------------------------------------

    /// 操作を `操作 file名` で記録し、指定の 1 点（操作・file 名）を失敗させる
    struct TestOps {
        events: StdMutex<Vec<String>>,
        fail: Option<(&'static str, String, std::io::ErrorKind)>,
        corrupt_partial_read: bool,
    }

    impl TestOps {
        fn new() -> Self {
            Self {
                events: StdMutex::new(Vec::new()),
                fail: None,
                corrupt_partial_read: false,
            }
        }
        fn failing(op: &'static str, name: &str, kind: std::io::ErrorKind) -> Self {
            Self {
                fail: Some((op, name.to_string(), kind)),
                ..Self::new()
            }
        }
        fn step(&self, op: &'static str, path: &Path) -> std::io::Result<()> {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            self.events.lock().unwrap().push(format!("{op} {name}"));
            match &self.fail {
                Some((fail_op, fail_name, kind)) if *fail_op == op && *fail_name == name => {
                    Err(std::io::Error::new(*kind, format!("injected {op} failure")))
                }
                _ => Ok(()),
            }
        }
        fn events(&self) -> Vec<String> {
            self.events.lock().unwrap().clone()
        }
    }

    impl RestoreFileOps for TestOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.rename(source, destination)
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            self.step("replace", destination)?;
            StdRestoreFileOps.replace(source, destination)
        }
        fn publish_no_replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            self.step("publish_no_replace", destination)?;
            StdRestoreFileOps.publish_no_replace(source, destination)
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            self.step("remove_file", path)?;
            StdRestoreFileOps.remove_file(path)
        }
        fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            self.step("write_file", path)?;
            StdRestoreFileOps.write_file(path, bytes)
        }
        fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            self.step("read_file", path)?;
            let mut bytes = StdRestoreFileOps.read_file(path)?;
            if self.corrupt_partial_read && path.to_string_lossy().ends_with(".partial") {
                bytes[100] ^= 0xFF;
            }
            Ok(bytes)
        }
        fn sync_file(&self, path: &Path) -> std::io::Result<()> {
            self.step("sync_file", path)?;
            StdRestoreFileOps.sync_file(path)
        }
        fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
            self.step("sync_parent", path)?;
            StdRestoreFileOps.sync_parent(path)
        }
        fn checkpoint(&self, conn: &db::DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            StdRestoreFileOps.checkpoint(conn)
        }
        fn copy_new(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            let name = destination
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            self.events.lock().unwrap().push(format!("copy_new {name}"));
            match &self.fail {
                Some(("copy_new", fail_name, kind)) if *fail_name == name => {
                    // 写す途中で止まった形: 作業名に一部だけ書いてから失敗する
                    std::fs::write(destination, &backup_bytes()[..512]).unwrap();
                    Err(std::io::Error::new(*kind, "injected copy failure"))
                }
                _ => StdRestoreFileOps.copy_new(source, destination),
            }
        }
    }

    // ---- MNT-01-D10 / D8 ----------------------------------------------------

    #[test]
    fn test_offsite_req901_d10_prepare_and_copy_refuse_register_sd_root() {
        // REQ-901 / MNT-01-D10 / 71 §71.11.2 手順 3・§71.11.3: レジの SD を用意しない・写さない
        let fx = Fx::new();
        let sd = fx.base.join("sd");
        std::fs::create_dir_all(&sd).unwrap();
        let sd_folder = make_register_sd(&sd);
        put_backup(&fx.local, &today_name());
        let before = snapshot(&sd);

        let result = prepare_with(
            &std_env(std::slice::from_ref(&sd)),
            &fx.app,
            &fx.local,
            &sd.join("x"),
        );
        assert_eq!(result, Err(OffsiteError::RegisterSd { sd_folder }));
        assert_eq!(snapshot(&sd), before);
        assert!(!fx.state_file().exists());

        // 用意済みの媒体の root に後から SD の folder が現れても、探す規則が見ない
        fx.prepare();
        let medium_before = snapshot(&fx.root);
        make_register_sd(&fx.root);
        put_backup(&fx.local, &name_on(today(), "090000"));
        assert_eq!(fx.check(), Ok(OffsiteCheckResult::MediumMissing));
        let mut expected = medium_before;
        expected.extend(
            snapshot(&fx.root)
                .into_iter()
                .filter(|(path, _)| path.starts_with("CASIO")),
        );
        assert_eq!(snapshot(&fx.root), expected, "InventoryBackup に書かない");
    }

    #[test]
    fn test_prepare_offsite_req901_d8_rejects_non_removable() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 手順 2: 取外し可能な root の列（drive の種類の判定の結果）に無い
        let fx = Fx::new();
        let before = snapshot(&fx.base);
        let result = prepare_with(&std_env(&[]), &fx.app, &fx.local, &fx.root);
        assert_eq!(result, Err(OffsiteError::NotRemovable));
        assert_eq!(snapshot(&fx.base), before, "何も書かない");
    }

    #[test]
    fn test_prepare_offsite_req901_d8_reuses_existing_marker_and_keeps_copies() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 手順 4: 読める目印は書き換えず、folder の写しを消さない
        let fx = Fx::new();
        let first = fx.prepare();
        assert_eq!(first.label, "控え 1");
        let marker = std::fs::read(fx.medium().join(MARKER_FILE)).unwrap();
        put_backup(
            &fx.medium(),
            &name_on(today() - chrono::Duration::days(1), "120000"),
        );
        let before = snapshot(&fx.medium());

        // 同じ PC でもう一度
        assert_eq!(fx.prepare().label, "控え 1");
        assert_eq!(snapshot(&fx.medium()), before);
        // 別の PC での登録し直し（状態が無い）
        std::fs::remove_file(fx.state_file()).unwrap();
        assert_eq!(fx.prepare().label, "控え 1");
        assert_eq!(
            std::fs::read(fx.medium().join(MARKER_FILE)).unwrap(),
            marker
        );
        assert_eq!(snapshot(&fx.medium()), before);
        assert_eq!(fx.state().media.len(), 1);
    }

    #[test]
    fn test_prepare_offsite_req901_d8_unknown_marker_format_is_refused() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 手順 4: format が 1 でない目印を作り直さない
        for marker in [
            serde_json::json!({"format": 2, "medium_id": "m-new", "label": {"n": 1}}),
            serde_json::json!({"format": 2, "medium_id": "m-new"}),
        ] {
            let fx = Fx::new();
            write_marker(&fx.root, &marker);
            let before = (snapshot(&fx.root), snapshot(&fx.app));
            let result = prepare_with(&std_env(&fx.roots()), &fx.app, &fx.local, &fx.root);
            assert_eq!(result, Err(OffsiteError::MarkerUnsupported));
            assert_eq!((snapshot(&fx.root), snapshot(&fx.app)), before);
        }
    }

    #[test]
    fn test_prepare_offsite_req901_d8_label_is_max_plus_one() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 手順 4: 札は最大 + 1（数 + 1 でない）
        let fx = Fx::new();
        write_state(
            &fx,
            &serde_json::json!({"format": 1, "media": [{"medium_id": "m-2", "label": "控え 2", "prepared_at": "2026-10-01 09:00:00"}], "last_success": null, "last_failure": null}),
        );
        assert_eq!(fx.prepare().label, "控え 3");
        let labels: Vec<String> = fx
            .state()
            .media
            .into_iter()
            .map(|entry| entry.label)
            .collect();
        assert_eq!(labels, ["控え 2", "控え 3"]);
    }

    #[test]
    fn test_offsite_req901_d8_marker_and_state_publish_order() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.1: 作業名へ書いて sync → 置き換え → 親 directory の sync
        let fx = Fx::new();
        let ops = TestOps::new();
        prepare_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local, &fx.root).unwrap();
        assert_eq!(
            ops.events(),
            [
                "write_file offsite-medium.json.tmp",
                "sync_file offsite-medium.json.tmp",
                "replace offsite-medium.json",
                "sync_parent offsite-medium.json",
                "write_file offsite-backup.json.tmp",
                "sync_file offsite-backup.json.tmp",
                "replace offsite-backup.json",
                "sync_parent offsite-backup.json",
            ]
        );
    }

    #[test]
    fn test_prepare_offsite_req901_d8_state_unreadable_writes_nothing() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 手順 3a: 読めない状態では媒体に何も書かない
        let fx = Fx::new();
        std::fs::write(fx.state_file(), b"{ not json").unwrap();
        let before = (snapshot(&fx.root), snapshot(&fx.app));
        let result = prepare_with(&std_env(&fx.roots()), &fx.app, &fx.local, &fx.root);
        assert_eq!(result, Err(OffsiteError::StateUnreadable));
        assert_eq!((snapshot(&fx.root), snapshot(&fx.app)), before);
    }

    #[test]
    fn test_prepare_offsite_req901_d8_state_write_full_is_io() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 エラーハンドリング: PC 側の状態の空き不足は Io（USB の空きと伝えない）
        let fx = Fx::new();
        let ops = TestOps::failing(
            "write_file",
            "offsite-backup.json.tmp",
            std::io::ErrorKind::StorageFull,
        );
        let result = prepare_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local, &fx.root);
        assert_eq!(result, Err(OffsiteError::Io));
    }

    #[cfg(unix)]
    #[test]
    fn test_prepare_offsite_req901_d8_redirected_inventory_dir_is_refused() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 手順 3b: InventoryBackup・目印の reparse point を追わない
        use std::os::unix::fs::symlink;
        let fx = Fx::new();
        let elsewhere = fx.base.join("elsewhere");
        std::fs::create_dir_all(&elsewhere).unwrap();
        symlink(&elsewhere, fx.medium()).unwrap();
        let before = snapshot(&fx.base);
        let result = prepare_with(&std_env(&fx.roots()), &fx.app, &fx.local, &fx.root);
        assert_eq!(result, Err(OffsiteError::Redirected));
        assert_eq!(snapshot(&fx.base), before);

        // 目印の file だけを link にする
        let fx = Fx::new();
        let target = fx.base.join("marker-target.json");
        std::fs::write(&target, b"{}").unwrap();
        std::fs::create_dir_all(fx.medium()).unwrap();
        symlink(&target, fx.medium().join(MARKER_FILE)).unwrap();
        let before = snapshot(&fx.base);
        let result = prepare_with(&std_env(&fx.roots()), &fx.app, &fx.local, &fx.root);
        assert_eq!(result, Err(OffsiteError::Redirected));
        assert_eq!(snapshot(&fx.base), before);
    }

    #[cfg(unix)]
    #[test]
    fn test_prepare_offsite_req901_d8_redirected_marker_temp_is_refused() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.2 手順 3b・§71.11.3: 目印の作業名の reparse point を追わない
        // （媒体の外の file を目印の JSON で書き換えない・作らない）
        use std::os::unix::fs::symlink;
        for dangling in [false, true] {
            let fx = Fx::new();
            let target = fx.base.join("outside-target");
            if !dangling {
                std::fs::write(&target, b"outside bytes").unwrap();
            }
            std::fs::create_dir_all(fx.medium()).unwrap();
            symlink(&target, fx.medium().join("offsite-medium.json.tmp")).unwrap();
            let before = snapshot(&fx.base);

            let result = prepare_with(&std_env(&fx.roots()), &fx.app, &fx.local, &fx.root);
            assert_eq!(
                result,
                Err(OffsiteError::Redirected),
                "dangling = {dangling}"
            );
            if dangling {
                assert!(!target.exists(), "dangling link の転送先を作らない");
            } else {
                assert_eq!(std::fs::read(&target).unwrap(), b"outside bytes");
            }
            assert!(
                std::fs::symlink_metadata(fx.medium().join(MARKER_FILE)).is_err(),
                "dangling = {dangling}: 目印を書かない"
            );
            assert!(!fx.state_file().exists(), "dangling = {dangling}");
            assert_eq!(snapshot(&fx.base), before, "dangling = {dangling}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_prepare_offsite_req901_d11_backup_dir_same_folder_is_refused() {
        // REQ-901 / MNT-01-D11 / 71 §71.11.2 手順 3c: 同じ実体の folder なら書かずに止める
        use std::os::unix::fs::symlink;
        // case 1: backup_dir = 媒体の InventoryBackup（正式名 1 本）
        // case 2: 別の一時 directory の link がそこを指す
        // case 3: まだ無い InventoryBackup
        for case in 1..=3 {
            let fx = Fx::new();
            let backup_dir = match case {
                1 => {
                    put_backup(&fx.medium(), &today_name());
                    fx.medium()
                }
                2 => {
                    put_backup(&fx.medium(), &today_name());
                    let link = fx.base.join("link-to-medium");
                    symlink(fx.medium(), &link).unwrap();
                    link
                }
                _ => fx.medium(),
            };
            let before = snapshot(&fx.root);
            let result = prepare_with(&std_env(&fx.roots()), &fx.app, &backup_dir, &fx.root);
            assert_eq!(
                result,
                Err(OffsiteError::BackupDirIsMediumFolder),
                "case {case}"
            );
            assert_eq!(snapshot(&fx.root), before, "case {case}");
            assert!(!fx.state_file().exists(), "case {case}");
        }
        // 対の case: InventoryBackup の中の folder と媒体の root は止めない
        for pair in ["sub", "root"] {
            let fx = Fx::new();
            let backup_dir = if pair == "sub" {
                fx.medium().join("sub")
            } else {
                fx.root.clone()
            };
            let result = prepare_with(&std_env(&fx.roots()), &fx.app, &backup_dir, &fx.root);
            assert!(result.is_ok(), "{pair}: {result:?}");
        }
    }

    // ---- 排他（§71.11.4） -------------------------------------------------------

    /// copy を channel で待たせる注入（待ちは recv_timeout、lock の中で panic しない）
    struct BlockingOps {
        started: mpsc::Sender<()>,
        release: StdMutex<mpsc::Receiver<()>>,
    }

    impl RestoreFileOps for BlockingOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.rename(source, destination)
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.replace(source, destination)
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.remove_file(path)
        }
        fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            StdRestoreFileOps.write_file(path, bytes)
        }
        fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            StdRestoreFileOps.read_file(path)
        }
        fn sync_file(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.sync_file(path)
        }
        fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.sync_parent(path)
        }
        fn checkpoint(&self, conn: &db::DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            StdRestoreFileOps.checkpoint(conn)
        }
        fn copy_new(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            let _ = self.started.send(());
            let released = self
                .release
                .lock()
                .map_err(|_| std::io::Error::other("release poisoned"))?
                .recv_timeout(Duration::from_secs(20));
            if released.is_err() {
                return Err(std::io::Error::other("release timed out"));
            }
            StdRestoreFileOps.copy_new(source, destination)
        }
    }

    #[test]
    fn test_offsite_req901_d8_serializes_check_with_prepare_reset_stop() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 の排他: 確認の copy の間、用意・作り直し・停止は本体へ入らない
        for case in ["prepare", "reset", "stop"] {
            let fx = Fx::new();
            fx.prepare();
            let name = today_name();
            put_backup(&fx.local, &name);
            let events: Arc<StdMutex<Vec<String>>> = Arc::new(StdMutex::new(Vec::new()));
            let (started_tx, started_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let (requested_tx, requested_rx) = mpsc::channel();
            let (acquired_tx, acquired_rx) = mpsc::channel();
            let roots = fx.roots();

            std::thread::scope(|scope| {
                let check_events = Arc::clone(&events);
                let fx_ref = &fx;
                let check_roots = roots.clone();
                let checker = scope.spawn(move || {
                    let ops = BlockingOps {
                        started: started_tx,
                        release: StdMutex::new(release_rx),
                    };
                    let observe = move |step: &'static str| {
                        check_events.lock().unwrap().push(format!("check:{step}"));
                    };
                    let env = Env {
                        roots: check_roots,
                        now: now(),
                        ops: &ops,
                        observe: &observe,
                    };
                    check_with(&env, &fx_ref.app, &fx_ref.local)
                });
                started_rx
                    .recv_timeout(Duration::from_secs(20))
                    .expect("確認の copy が始まらない");

                let other_events = Arc::clone(&events);
                let other_roots = roots.clone();
                let other = scope.spawn(move || {
                    let requested_tx = StdMutex::new(requested_tx);
                    let acquired_tx = StdMutex::new(acquired_tx);
                    let observe = move |step: &'static str| {
                        other_events.lock().unwrap().push(format!("{case}:{step}"));
                        let sender = match step {
                            "lock_requested" => requested_tx.lock().unwrap().send(()),
                            "lock_acquired" => acquired_tx.lock().unwrap().send(()),
                            _ => Ok(()),
                        };
                        let _ = sender;
                    };
                    let env = Env {
                        roots: other_roots,
                        now: now() + chrono::Duration::seconds(1),
                        ops: &StdRestoreFileOps,
                        observe: &observe,
                    };
                    match case {
                        "prepare" => {
                            prepare_with(&env, &fx_ref.app, &fx_ref.local, &fx_ref.root).map(|_| ())
                        }
                        "reset" => reset_with(&env, &fx_ref.app, &fx_ref.local).map(|_| ()),
                        _ => stop_with(&env, &fx_ref.app),
                    }
                });
                requested_rx
                    .recv_timeout(Duration::from_secs(20))
                    .expect("競合操作が lock を試みない");
                // 確認の copy を放す前は本体へ入らない
                assert!(
                    acquired_rx
                        .recv_timeout(Duration::from_millis(300))
                        .is_err(),
                    "{case}: 確認の copy の間に lock を取った"
                );
                release_tx.send(()).unwrap();
                assert_eq!(
                    checker.join().unwrap(),
                    Ok(OffsiteCheckResult::Copied {
                        file_name: name.clone(),
                        labels: vec!["控え 1".to_string()],
                    })
                );
                assert_eq!(other.join().unwrap(), Ok(()), "{case}");
            });

            let events = events.lock().unwrap().clone();
            let position = |event: &str| {
                events
                    .iter()
                    .position(|recorded| recorded == event)
                    .unwrap_or_else(|| panic!("{event} が無い: {events:?}"))
            };
            assert!(
                position("check:lock_released") < position(&format!("{case}:lock_acquired")),
                "{case}: {events:?}"
            );
            match case {
                "stop" => {
                    assert!(!fx.state_file().exists(), "停止の後に状態を作り直さない");
                    let stopped = names(&fx.app)
                        .into_iter()
                        .find(|name| name.starts_with("offsite-backup.json.stopped-"))
                        .expect("退避の file");
                    let aside: OffsiteState =
                        serde_json::from_slice(&std::fs::read(fx.app.join(stopped)).unwrap())
                            .unwrap();
                    assert_eq!(aside.last_success.unwrap().file_name, name);
                    let status = fx.status();
                    assert!(!status.prepared && !status.stale);
                    assert_eq!(fx.check(), Ok(OffsiteCheckResult::NotPrepared));
                    assert!(!fx.state_file().exists());
                }
                _ => {
                    assert_eq!(fx.state().last_success.unwrap().file_name, name, "{case}");
                }
            }
        }
    }

    #[test]
    fn test_offsite_req901_d8_poisoned_lock_does_not_stop_entrances() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 の排他: lock を持った thread の panic（poison）の後も、
        // 確認・用意・作り直し・やめるの入口が結果を返す
        struct ClearPoison;
        impl Drop for ClearPoison {
            fn drop(&mut self) {
                // 後始末: process 全体の lock を、ほかの test のために poison の無い状態へ戻す
                OFFSITE_LOCK.clear_poison();
            }
        }
        let fx = Fx::new();
        put_backup(&fx.local, &today_name());
        let _cleanup = ClearPoison;

        // 本番と同じ入口で lock を取った直後に panic させる（panic の message は test の出力に出る）
        let panicked = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let observe = |step: &'static str| {
                        if step == "lock_acquired" {
                            panic!("lock を持ったままの panic（test の注入）");
                        }
                    };
                    let env = Env {
                        roots: fx.roots(),
                        now: now(),
                        ops: &StdRestoreFileOps,
                        observe: &observe,
                    };
                    check_with(&env, &fx.app, &fx.local)
                })
                .join()
        });
        assert!(panicked.is_err(), "注入した panic が起きていない");
        assert!(OFFSITE_LOCK.is_poisoned(), "lock が poison されていない");

        let env = std_env(&fx.roots());
        assert_eq!(
            prepare_with(&env, &fx.app, &fx.local, &fx.root).map(|view| view.label),
            Ok("控え 1".to_string())
        );
        assert_eq!(
            check_with(&env, &fx.app, &fx.local),
            Ok(OffsiteCheckResult::UpToDate)
        );
        std::fs::write(fx.state_file(), b"{ broken json").unwrap();
        assert_eq!(
            reset_with(&env, &fx.app, &fx.local),
            Ok(OffsiteCheckResult::UpToDate)
        );
        assert_eq!(stop_with(&env, &fx.app), Ok(()));
    }

    // ---- 写し（§71.11.4） -------------------------------------------------------

    #[test]
    fn test_check_offsite_req901_d8_not_prepared_and_medium_missing_write_nothing() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 1・3
        let fx = Fx::new();
        put_backup(&fx.local, &today_name());
        let before = (snapshot(&fx.app), snapshot(&fx.root));
        assert_eq!(fx.check(), Ok(OffsiteCheckResult::NotPrepared));
        assert_eq!((snapshot(&fx.app), snapshot(&fx.root)), before);

        let prepared = Fx::new();
        prepared.prepare();
        put_backup(&prepared.local, &today_name());
        let before = (snapshot(&prepared.app), snapshot(&prepared.root));
        assert_eq!(
            check_with(&std_env(&[]), &prepared.app, &prepared.local),
            Ok(OffsiteCheckResult::MediumMissing)
        );
        assert_eq!((snapshot(&prepared.app), snapshot(&prepared.root)), before);
    }

    #[test]
    fn test_check_offsite_req901_d8_ignores_unmarked_and_foreign_media() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.3: 目印の無い媒体・別の PC で用意した媒体に書かない
        let fx = Fx::new();
        fx.prepare();
        put_backup(&fx.local, &today_name());
        let unmarked = fx.base.join("F");
        let foreign = fx.base.join("G");
        std::fs::create_dir_all(&unmarked).unwrap();
        std::fs::create_dir_all(&foreign).unwrap();
        write_marker(
            &foreign,
            &serde_json::json!({"format": 1, "medium_id": "other-pc", "label": "控え 1", "prepared_at": "2026-10-01 09:00:00"}),
        );
        let before = (snapshot(&unmarked), snapshot(&foreign), snapshot(&fx.app));
        let roots = [unmarked.clone(), foreign.clone()];
        assert_eq!(
            check_with(&std_env(&roots), &fx.app, &fx.local),
            Ok(OffsiteCheckResult::MediumMissing)
        );
        assert_eq!(
            (snapshot(&unmarked), snapshot(&foreign), snapshot(&fx.app)),
            before
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_check_offsite_req901_d11_backup_dir_same_folder_writes_nothing() {
        // REQ-901 / MNT-01-D11 / 71 §71.11.4 手順 3: 同じ folder なら写さず・消さず、last_failure に残す
        use std::os::unix::fs::symlink;
        for case in ["direct", "link"] {
            let fx = Fx::new();
            fx.prepare();
            let medium_id = fx.state().media[0].medium_id.clone();
            // PC の中の最新の正式名を媒体に同じ名前・同じ size で置く（判定が無ければ UpToDate になる形）
            put_backup(&fx.medium(), &today_name());
            let backup_dir = if case == "direct" {
                fx.medium()
            } else {
                let link = fx.base.join("local-link");
                symlink(fx.medium(), &link).unwrap();
                link
            };
            let before = snapshot(&fx.root);
            let result = check_with(&std_env(&fx.roots()), &fx.app, &backup_dir);
            assert_eq!(result, Err(OffsiteError::BackupDirIsMediumFolder), "{case}");
            let state = fx.state();
            assert_eq!(
                state
                    .last_failure
                    .map(|failure| (failure.kind, failure.medium_id)),
                Some((OffsiteFailureKind::BackupDirIsMediumFolder, Some(medium_id))),
            );
            assert_eq!(state.last_success, None);
            assert_eq!(snapshot(&fx.root), before, "{case}");
            assert_eq!(
                status_with(&fx.roots(), &fx.app, &backup_dir, today()).last_failure_kind,
                Some(OffsiteFailureKind::BackupDirIsMediumFolder)
            );
            // 保存先を戻した次の確認で写る
            put_backup(&fx.local, &name_on(today(), "090000"));
            assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        }
    }

    #[test]
    fn test_check_offsite_req901_d8_copy_publishes_verified_copy_and_records_success() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 4・5
        let fx = Fx::new();
        fx.prepare();
        let name = today_name();
        let source = put_backup(&fx.local, &name);
        assert_eq!(
            fx.check(),
            Ok(OffsiteCheckResult::Copied {
                file_name: name.clone(),
                labels: vec!["控え 1".to_string()],
            })
        );
        assert_eq!(
            std::fs::read(fx.medium().join(&name)).unwrap(),
            std::fs::read(&source).unwrap()
        );
        let state = fx.state();
        let success = state.last_success.unwrap();
        assert_eq!(success.file_name, name);
        assert_eq!(success.sha256, sha_of(&source));
        assert_eq!(success.at, "2026-11-02 09:00:00");
        assert_eq!(success.medium_id, state.media[0].medium_id);
        assert_eq!(state.last_failure, None);
        assert!(!names(&fx.medium())
            .iter()
            .any(|name| name.ends_with(".partial")));
    }

    #[test]
    fn test_check_offsite_req901_d8_unverified_source_is_not_copied() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 3a: 壊れた・空の・別用途の DB を写さない
        for case in ["corrupt", "empty", "foreign"] {
            let fx = Fx::new();
            fx.prepare();
            let name = today_name();
            let path = fx.local.join(&name);
            match case {
                "corrupt" => {
                    put_backup(&fx.local, &name);
                    corrupt_products_page(&path);
                }
                "empty" => std::fs::write(&path, b"").unwrap(),
                _ => {
                    let conn = rusqlite::Connection::open(&path).unwrap();
                    conn.execute_batch("CREATE TABLE other (id INTEGER)")
                        .unwrap();
                }
            }
            let result = fx.check();
            assert_eq!(result, Err(OffsiteError::SourceUnverified), "{case}");
            assert!(!fx.medium().join(&name).exists(), "{case}");
            let state = fx.state();
            assert_eq!(state.last_success, None);
            assert_eq!(
                state
                    .last_failure
                    .map(|failure| (failure.kind, failure.medium_id)),
                Some((OffsiteFailureKind::SourceUnverified, None))
            );
        }
    }

    #[test]
    fn test_check_offsite_req901_d8_source_path_with_uri_chars_is_inspected() {
        // REQ-901 / MNT-01-D8 / 71 §71.12 手順 1: 写す元は共有の inspect_backup の符号化を通る
        let fx = Fx::new();
        let local = fx.base.join("a#b%c");
        std::fs::create_dir_all(&local).unwrap();
        prepare_with(&std_env(&fx.roots()), &fx.app, &local, &fx.root).unwrap();
        let name = today_name();
        put_backup(&local, &name);
        assert!(matches!(
            check_with(&std_env(&fx.roots()), &fx.app, &local),
            Ok(OffsiteCheckResult::Copied { .. })
        ));
        assert!(fx.medium().join(&name).exists());
    }

    #[test]
    fn test_check_offsite_req901_d8_source_list_error_is_io() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 2: 一覧の失敗は NoLocalBackup に倒さない
        let fx = Fx::new();
        fx.prepare();
        let not_a_dir = fx.base.join("local-file");
        std::fs::write(&not_a_dir, b"x").unwrap();
        let before = snapshot(&fx.root);
        let result = check_with(&std_env(&fx.roots()), &fx.app, &not_a_dir);
        assert_eq!(result, Err(OffsiteError::Io));
        let state = fx.state();
        assert_eq!(
            state
                .last_failure
                .map(|failure| (failure.kind, failure.medium_id)),
            Some((OffsiteFailureKind::Io, None))
        );
        assert_eq!(state.last_success, None);
        assert_eq!(snapshot(&fx.root), before);
    }

    #[test]
    fn test_check_offsite_req901_d8_interruption_before_publish() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 のエラーの表: 公開の前の失敗は正式名を作らない
        let name = today_name();
        let partial = partial_name(&name);
        let points = [
            (
                "copy_new",
                partial.clone(),
                std::io::ErrorKind::Other,
                OffsiteFailureKind::Io,
            ),
            (
                "copy_new",
                partial.clone(),
                std::io::ErrorKind::StorageFull,
                OffsiteFailureKind::StorageFull,
            ),
            (
                "sync_file",
                partial.clone(),
                std::io::ErrorKind::Other,
                OffsiteFailureKind::Io,
            ),
            (
                "publish_no_replace",
                name.clone(),
                std::io::ErrorKind::Other,
                OffsiteFailureKind::Io,
            ),
        ];
        for (op, file, kind, expected) in points {
            let fx = Fx::new();
            fx.prepare();
            put_backup(&fx.local, &name);
            let ops = TestOps::failing(op, &file, kind);
            let result = check_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local);
            let expected_error = if expected == OffsiteFailureKind::StorageFull {
                OffsiteError::StorageFull
            } else {
                OffsiteError::Io
            };
            assert_eq!(result, Err(expected_error), "{op}");
            assert!(!fx.medium().join(&name).exists(), "{op}: 正式名を作らない");
            let state = fx.state();
            assert_eq!(state.last_success, None, "{op}");
            assert_eq!(
                state.last_failure.map(|failure| failure.kind),
                Some(expected)
            );
            // 抜去で残った作業名は次の確認が消して写し直す
            std::fs::write(fx.medium().join(&partial), b"left over").unwrap();
            assert!(
                matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })),
                "{op}"
            );
            assert!(!fx.medium().join(&partial).exists());
            assert_eq!(
                std::fs::read(fx.medium().join(&name)).unwrap(),
                backup_bytes(),
                "{op}"
            );
        }
    }

    #[test]
    fn test_check_offsite_req901_d8_interruption_after_publish() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 のエラーの表: 公開の後の sync の失敗は正式名を残し、成功を記録しない
        let fx = Fx::new();
        fx.prepare();
        let name = today_name();
        put_backup(&fx.local, &name);
        let ops = TestOps::failing("sync_parent", &name, std::io::ErrorKind::Other);
        assert_eq!(
            check_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
            Err(OffsiteError::Io)
        );
        assert_eq!(
            std::fs::read(fx.medium().join(&name)).unwrap(),
            backup_bytes()
        );
        let state = fx.state();
        assert_eq!(state.last_success, None);
        assert_eq!(
            state.last_failure.map(|failure| failure.kind),
            Some(OffsiteFailureKind::Io)
        );
        // 次は同じ名前・同じ size があるので写さず、状態を変えない
        let before = std::fs::read(fx.state_file()).unwrap();
        assert_eq!(fx.check(), Ok(OffsiteCheckResult::UpToDate));
        assert_eq!(std::fs::read(fx.state_file()).unwrap(), before);
        // 新しい backup を写せた時に直る
        let newer = name_on(today(), "100000");
        put_backup(&fx.local, &newer);
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        assert_eq!(fx.state().last_success.unwrap().file_name, newer);
    }

    #[test]
    fn test_check_offsite_req901_d8_success_record_sync_failure_stays_copied() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 のエラーの表（runtime (b) の起票で決めた行）
        let fx = Fx::new();
        fx.prepare();
        let name = today_name();
        put_backup(&fx.local, &name);
        let ops = TestOps::failing("sync_parent", STATE_FILE, std::io::ErrorKind::Other);
        let (result, logs) = crate::test_tracing::capture(|| {
            check_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local)
        });
        assert!(
            matches!(result, Ok(OffsiteCheckResult::Copied { .. })),
            "{result:?}"
        );
        assert!(
            logs.contains("成功の記録の置き換えの後の sync に失敗"),
            "{logs}"
        );
        let state = fx.state();
        assert_eq!(state.last_success.unwrap().file_name, name);
        assert_eq!(state.last_failure, None);

        // 対の case: 置き換えそのものの失敗は Io で、状態の file は前のまま
        let fx = Fx::new();
        fx.prepare();
        put_backup(&fx.local, &name);
        let before = std::fs::read(fx.state_file()).unwrap();
        let ops = TestOps::failing("replace", STATE_FILE, std::io::ErrorKind::Other);
        assert_eq!(
            check_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
            Err(OffsiteError::Io)
        );
        assert_eq!(std::fs::read(fx.state_file()).unwrap(), before);
    }

    #[test]
    fn test_check_offsite_req901_d8_medium_swap_after_discovery_writes_nothing() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.3 の媒体の固定: 照合の後に入れ替わった媒体へ書かない
        for point in ["before_copy", "before_publish"] {
            let fx = Fx::new();
            fx.prepare();
            let name = today_name();
            put_backup(&fx.local, &name);
            let marker_path = fx.medium().join(MARKER_FILE);
            let observe = |step: &'static str| {
                if step == point {
                    write_marker(
                        marker_path.parent().unwrap().parent().unwrap(),
                        &serde_json::json!({"format": 1, "medium_id": "swapped", "label": "控え 1", "prepared_at": "2026-10-01 09:00:00"}),
                    );
                }
            };
            let ops = TestOps::new();
            let env = Env {
                roots: fx.roots(),
                now: now(),
                ops: &ops,
                observe: &observe,
            };
            assert_eq!(
                check_with(&env, &fx.app, &fx.local),
                Err(OffsiteError::Io),
                "{point}"
            );
            assert_eq!(
                names(&fx.medium()),
                [MARKER_FILE.to_string()],
                "{point}: 作業名・正式名を残さない"
            );
            assert_eq!(
                fx.state().last_failure.map(|failure| failure.kind),
                Some(OffsiteFailureKind::Io)
            );
            // どの時点の確かめ直しが止めたかを操作の記録で分ける（後段の確かめ直しに拾わせない）
            let called = |op: &str| ops.events().iter().any(|event| event.starts_with(op));
            if point == "before_copy" {
                assert!(
                    !called("copy_new "),
                    "作業名へ写す前に止める: {:?}",
                    ops.events()
                );
            } else {
                assert!(called("copy_new "), "{:?}", ops.events());
                assert!(
                    !called("publish_no_replace "),
                    "公開の前に止める: {:?}",
                    ops.events()
                );
            }
        }

        // 掃除の直前（手順 4g）: 公開の後に入れ替わった媒体の古い写しを 1 本も消さない
        let fx = Fx::new();
        fx.prepare();
        let name = today_name();
        put_backup(&fx.local, &name);
        let base_day = today() - chrono::Duration::days(40);
        let old: Vec<String> = (0..31)
            .map(|offset| name_on(base_day + chrono::Duration::days(offset), "120000"))
            .collect();
        for old_name in &old {
            std::fs::write(fx.medium().join(old_name), b"old copy").unwrap();
        }
        let observe = |step: &'static str| {
            if step == "before_cleanup" {
                write_marker(
                    &fx.root,
                    &serde_json::json!({"format": 1, "medium_id": "swapped", "label": "控え 1", "prepared_at": "2026-10-01 09:00:00"}),
                );
            }
        };
        let ops = TestOps::new();
        let env = Env {
            roots: fx.roots(),
            now: now(),
            ops: &ops,
            observe: &observe,
        };
        assert_eq!(check_with(&env, &fx.app, &fx.local), Err(OffsiteError::Io));
        for old_name in &old {
            assert_eq!(
                std::fs::read(fx.medium().join(old_name)).unwrap(),
                b"old copy",
                "{old_name}"
            );
        }
        assert!(
            !ops.events()
                .iter()
                .any(|event| event.starts_with("remove_file ")),
            "{:?}",
            ops.events()
        );
        let state = fx.state();
        assert_eq!(state.last_success, None, "成功の記録を書かない");
        assert_eq!(
            state.last_failure.map(|failure| failure.kind),
            Some(OffsiteFailureKind::Io)
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_check_offsite_req901_d8_reparse_points_are_not_followed() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.3 の「reparse point を追わない」
        use std::os::unix::fs::symlink;
        let name = today_name();
        // (1) 用意済みの媒体の InventoryBackup を link に替える
        let fx = Fx::new();
        fx.prepare();
        put_backup(&fx.local, &name);
        let elsewhere = fx.base.join("elsewhere");
        std::fs::rename(fx.medium(), &elsewhere).unwrap();
        symlink(&elsewhere, fx.medium()).unwrap();
        let before = snapshot(&elsewhere);
        assert_eq!(fx.check(), Ok(OffsiteCheckResult::MediumMissing));
        assert_eq!(snapshot(&elsewhere), before);

        // (2) 正式名を転送先の file への link にする: 写し済みとせず、link・転送先を変えない
        let fx = Fx::new();
        fx.prepare();
        put_backup(&fx.local, &name);
        let target = fx.base.join("target.db");
        std::fs::write(&target, backup_bytes()).unwrap();
        symlink(&target, fx.medium().join(&name)).unwrap();
        assert_eq!(
            fx.check(),
            Err(OffsiteError::Io),
            "公開が既存の名前で失敗する"
        );
        assert_eq!(std::fs::read(&target).unwrap(), backup_bytes());
        assert!(std::fs::symlink_metadata(fx.medium().join(&name))
            .unwrap()
            .file_type()
            .is_symlink());

        // 作業名の entry（4b で残る link と、4b の後・4d の直前に現れる link、dangling も）
        for (when, dangling) in [("left", false), ("left", true), ("appears", false)] {
            let fx = Fx::new();
            fx.prepare();
            put_backup(&fx.local, &name);
            let target = fx.base.join("partial-target");
            if !dangling {
                std::fs::write(&target, b"target bytes").unwrap();
            }
            let partial = fx.medium().join(partial_name(&name));
            if when == "left" {
                symlink(&target, &partial).unwrap();
            }
            let observe = |step: &'static str| {
                if when == "appears" && step == "before_copy" {
                    symlink(&target, &partial).unwrap();
                }
            };
            let env = Env {
                roots: fx.roots(),
                now: now(),
                ops: &StdRestoreFileOps,
                observe: &observe,
            };
            assert_eq!(check_with(&env, &fx.app, &fx.local), Err(OffsiteError::Io));
            let state = fx.state();
            assert_eq!(state.last_success, None);
            assert_eq!(
                state.last_failure.map(|failure| failure.kind),
                Some(OffsiteFailureKind::Io)
            );
            assert!(!fx.medium().join(&name).exists());
            assert!(std::fs::symlink_metadata(&partial)
                .unwrap()
                .file_type()
                .is_symlink());
            if dangling {
                assert!(!target.exists(), "dangling link の転送先を作らない");
            } else {
                assert_eq!(std::fs::read(&target).unwrap(), b"target bytes");
            }
        }

        // (3) 掃除（手順 4g）: 規約の名前の link を写しに数えず・消さず、本物の 30 本を残す。
        // link の名前が最も古い case（数えると link が消える）と最も新しい case（数えると本物が 1 本消える）
        for link_days in [-60, 60] {
            let fx = Fx::new();
            fx.prepare();
            put_backup(&fx.local, &name);
            let real: Vec<String> = (1..=29)
                .map(|offset| name_on(today() - chrono::Duration::days(offset), "120000"))
                .collect();
            for real_name in &real {
                std::fs::write(fx.medium().join(real_name), b"old copy").unwrap();
            }
            let target = fx.base.join("link-target.db");
            std::fs::write(&target, b"target bytes").unwrap();
            let link_name = name_on(today() + chrono::Duration::days(link_days), "120000");
            symlink(&target, fx.medium().join(&link_name)).unwrap();

            assert!(
                matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })),
                "{link_days}"
            );
            let mut expected = real.clone();
            expected.extend([name.clone(), link_name.clone(), MARKER_FILE.to_string()]);
            expected.sort();
            assert_eq!(names(&fx.medium()), expected, "{link_days}");
            assert!(std::fs::symlink_metadata(fx.medium().join(&link_name))
                .unwrap()
                .file_type()
                .is_symlink());
            assert_eq!(std::fs::read(&target).unwrap(), b"target bytes");
        }
    }

    #[test]
    fn test_check_offsite_req901_d8_publish_does_not_replace_existing_copy() {
        // REQ-901 / 71 §71.4.1: 同じ名前で size の違う正式名を上書きしない
        let fx = Fx::new();
        fx.prepare();
        let name = today_name();
        put_backup(&fx.local, &name);
        std::fs::write(fx.medium().join(&name), b"different size").unwrap();
        assert_eq!(fx.check(), Err(OffsiteError::Io));
        assert_eq!(
            std::fs::read(fx.medium().join(&name)).unwrap(),
            b"different size"
        );
        assert!(!fx.medium().join(partial_name(&name)).exists());
    }

    #[test]
    fn test_check_offsite_req901_d8_removes_only_conforming_partial() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 4b
        let fx = Fx::new();
        fx.prepare();
        put_backup(&fx.local, &today_name());
        let conforming = partial_name(&name_on(today() - chrono::Duration::days(1), "120000"));
        std::fs::write(fx.medium().join("unrelated.partial"), b"keep").unwrap();
        std::fs::write(fx.medium().join(&conforming), b"left over").unwrap();
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        assert!(fx.medium().join("unrelated.partial").exists());
        assert!(!fx.medium().join(&conforming).exists());
    }

    #[test]
    fn test_check_offsite_req901_d8_verify_mismatch_removes_partial_and_records_failure() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 4e
        let fx = Fx::new();
        fx.prepare();
        let name = today_name();
        put_backup(&fx.local, &name);
        let ops = TestOps {
            corrupt_partial_read: true,
            ..TestOps::new()
        };
        assert_eq!(
            check_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
            Err(OffsiteError::VerifyMismatch)
        );
        assert_eq!(names(&fx.medium()), [MARKER_FILE.to_string()]);
        assert_eq!(
            fx.state().last_failure.map(|failure| failure.kind),
            Some(OffsiteFailureKind::VerifyMismatch)
        );
    }

    #[test]
    fn test_check_offsite_req901_d8_up_to_date_skips_copy() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 4a
        let fx = Fx::new();
        fx.prepare();
        let name = today_name();
        put_backup(&fx.local, &name);
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        let copy = fx.medium().join(&name);
        let modified = std::fs::metadata(&copy).unwrap().modified().unwrap();
        let state = std::fs::read(fx.state_file()).unwrap();
        let ops = TestOps::new();
        assert_eq!(
            check_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
            Ok(OffsiteCheckResult::UpToDate)
        );
        assert!(ops.events().is_empty(), "{:?}", ops.events());
        assert_eq!(
            std::fs::metadata(&copy).unwrap().modified().unwrap(),
            modified
        );
        assert_eq!(std::fs::read(fx.state_file()).unwrap(), state);
    }

    #[test]
    fn test_check_offsite_req901_d8_retention_keeps_newest_and_foreign_files() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.4 手順 4g
        // case 1: 31 本 + 今写す 1 本 + 規約外の 1 file → 最も古い 2 本だけが消える（残り 30 本）
        let fx = Fx::new();
        fx.prepare();
        let base_day = today() - chrono::Duration::days(40);
        let old: Vec<String> = (0..31)
            .map(|offset| name_on(base_day + chrono::Duration::days(offset), "120000"))
            .collect();
        for name in &old {
            std::fs::write(fx.medium().join(name), b"old copy").unwrap();
        }
        std::fs::write(fx.medium().join("notes.txt"), b"foreign").unwrap();
        let name = today_name();
        put_backup(&fx.local, &name);
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        let mut expected: Vec<String> = old[2..].to_vec();
        expected.push(name.clone());
        expected.push(MARKER_FILE.to_string());
        expected.push("notes.txt".to_string());
        expected.sort();
        assert_eq!(names(&fx.medium()), expected);

        // case 2: 今写す file より名前の新しい正式名が 30 本（PC の時計が遅れた場合）。今写した file は消さない
        let fx = Fx::new();
        fx.prepare();
        let future: Vec<String> = (1..=30)
            .map(|offset| name_on(today() + chrono::Duration::days(offset), "120000"))
            .collect();
        for name in &future {
            std::fs::write(fx.medium().join(name), b"future copy").unwrap();
        }
        put_backup(&fx.local, &name);
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        let mut expected = future.clone();
        expected.push(name.clone());
        expected.push(MARKER_FILE.to_string());
        expected.sort();
        assert_eq!(names(&fx.medium()), expected);
    }

    #[test]
    fn test_check_offsite_req901_d8_multibyte_names_on_medium_do_not_panic() {
        // REQ-901 / 71 §71.3（媒体の側）: 規約外の多バイトの名前を無視する
        let fx = Fx::new();
        fx.prepare();
        let odd = [
            "inventory_backup_あああああ.db",
            "inventory_backup_あああああ.db.partial",
        ];
        for name in odd {
            std::fs::write(fx.medium().join(name), b"odd").unwrap();
        }
        put_backup(&fx.local, &today_name());
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        for name in odd {
            assert!(fx.medium().join(name).exists(), "{name}");
        }
    }

    // ---- 状態（§71.11.5） -------------------------------------------------------

    #[test]
    fn test_offsite_status_req901_d8_state_unreadable_is_stale_not_unprepared() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5
        let fx = Fx::new();
        std::fs::write(fx.state_file(), b"\x00\x01 broken").unwrap();
        let before = snapshot(&fx.app);
        let status = fx.status();
        assert!(status.prepared && status.stale);
        assert_eq!(
            status.last_failure_kind,
            Some(OffsiteFailureKind::StateUnreadable)
        );
        assert_eq!(snapshot(&fx.app), before, "書かない");
    }

    #[test]
    fn test_offsite_status_req901_d8_local_backup_old() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5 の local_backup_old
        let cases: [(Option<i64>, bool); 4] = [
            (Some(10), true),
            (Some(2), false),
            (Some(3), true),
            (None, true),
        ];
        for (days, expected) in cases {
            let fx = Fx::new();
            fx.prepare();
            if let Some(days) = days {
                put_backup(
                    &fx.local,
                    &name_on(today() - chrono::Duration::days(days), "120000"),
                );
            }
            let status = fx.status();
            assert_eq!(status.local_backup_old, expected, "{days:?}");
            assert!(
                status.stale,
                "stale は local_backup_old で変わらない（まだ写せていない）"
            );
        }
        let fx = Fx::new();
        fx.prepare();
        let not_a_dir = fx.base.join("local-file");
        std::fs::write(&not_a_dir, b"x").unwrap();
        let status = status_with(&fx.roots(), &fx.app, &not_a_dir, today());
        assert!(!status.local_backup_old, "一覧の Err は false");
    }

    #[test]
    fn test_offsite_status_req901_d8_stale_boundary_three_days() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5 の規則（>= 3 日）と、古い控えの日付から数える
        let fx = Fx::new();
        fx.prepare();
        assert!(fx.status().stale, "写せていない");
        let medium_id = fx.state().media[0].medium_id.clone();
        let with_success = |copied_days: i64, content_days: i64| {
            let copied = today() - chrono::Duration::days(copied_days);
            let mut state = fx.state();
            state.last_success = Some(OffsiteCopyRecord {
                at: format!("{} 09:00:00", copied.format("%Y-%m-%d")),
                file_name: name_on(today() - chrono::Duration::days(content_days), "120000"),
                medium_id: medium_id.clone(),
                sha256: "0".repeat(64),
            });
            std::fs::write(fx.state_file(), serde_json::to_vec(&state).unwrap()).unwrap();
            fx.status()
        };
        let two = with_success(2, 2);
        assert_eq!((two.days_since_last_success, two.stale), (Some(2), false));
        let three = with_success(3, 3);
        assert_eq!(
            (three.days_since_last_success, three.stale),
            (Some(3), true)
        );
        let old_content = with_success(0, 10);
        assert_eq!(
            (old_content.days_since_last_success, old_content.stale),
            (Some(10), true)
        );
        assert_eq!(old_content.last_success_label.as_deref(), Some("控え 1"));
    }

    #[test]
    fn test_offsite_req901_d8_stale_clears_only_after_new_copy() {
        // REQ-901 / MNT-01-D8・UI-00-D12 の producer: 5 日前の状態 → 実際の確認 → 状態の file → status
        let five_days_ago = today() - chrono::Duration::days(5);
        let setup = || {
            let fx = Fx::new();
            fx.prepare();
            let mut state = fx.state();
            state.last_success = Some(OffsiteCopyRecord {
                at: format!("{} 09:00:00", five_days_ago.format("%Y-%m-%d")),
                file_name: name_on(five_days_ago, "080000"),
                medium_id: state.media[0].medium_id.clone(),
                sha256: "0".repeat(64),
            });
            std::fs::write(fx.state_file(), serde_json::to_vec(&state).unwrap()).unwrap();
            put_backup(&fx.local, &today_name());
            fx
        };
        let fx = setup();
        let status = fx.status();
        assert_eq!(
            (status.days_since_last_success, status.stale),
            (Some(5), true)
        );
        // 媒体なし: 成功の記録を変えない
        assert_eq!(
            check_with(&std_env(&[]), &fx.app, &fx.local),
            Ok(OffsiteCheckResult::MediumMissing)
        );
        assert_eq!(fx.status().days_since_last_success, Some(5));
        // 差した後
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        let status = fx.status();
        assert_eq!(
            (
                status.days_since_last_success,
                status.stale,
                status.last_failure_kind
            ),
            (Some(0), false, None)
        );
        assert_eq!(fx.state().last_success.unwrap().file_name, today_name());

        // 対の case: 当日の同名・同 size の写しが既にあれば UpToDate で 5 日前のまま
        let fx = setup();
        put_backup(&fx.medium(), &today_name());
        assert_eq!(fx.check(), Ok(OffsiteCheckResult::UpToDate));
        let status = fx.status();
        assert_eq!(
            (status.days_since_last_success, status.stale),
            (Some(5), true)
        );
        // 対の case: copy の失敗なら知らせを保つ
        let fx = setup();
        let ops = TestOps::failing(
            "copy_new",
            &partial_name(&today_name()),
            std::io::ErrorKind::Other,
        );
        assert_eq!(
            check_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
            Err(OffsiteError::Io)
        );
        let status = fx.status();
        assert_eq!(
            (status.days_since_last_success, status.stale),
            (Some(5), true)
        );
    }

    #[test]
    fn test_restore_req901_d8_does_not_touch_offsite_state() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.1: 状態は DB の外。復元で巻き戻らない
        let fx = Fx::new();
        let db_path = fx.app.join("inventory.db");
        let conn = db::init_database(db_path.to_str().unwrap()).unwrap();
        let backup = super::super::backup::create_backup(&conn, &fx.local).unwrap();
        fx.prepare();
        let before = std::fs::read(fx.state_file()).unwrap();
        let conn =
            super::super::backup::restore_backup(conn, Path::new(&backup.file_path), &db_path)
                .unwrap();
        drop(conn);
        assert_eq!(std::fs::read(fx.state_file()).unwrap(), before);
    }

    // ---- やめる（§71.11.5b） ---------------------------------------------------

    #[test]
    fn test_stop_offsite_backup_req901_d8_sets_aside_state_and_keeps_media() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5b
        for kind in ["readable", "broken", "format2"] {
            let fx = Fx::new();
            fx.prepare();
            put_backup(
                &fx.local,
                &name_on(today() - chrono::Duration::days(1), "120000"),
            );
            assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
            let marker = std::fs::read(fx.medium().join(MARKER_FILE)).unwrap();
            match kind {
                "broken" => std::fs::write(fx.state_file(), b"broken").unwrap(),
                "format2" => write_state(&fx, &serde_json::json!({"format": 2})),
                _ => {}
            }
            put_backup(&fx.local, &today_name());
            let original = std::fs::read(fx.state_file()).unwrap();
            let medium_before = snapshot(&fx.root);
            let local_before = snapshot(&fx.local);

            assert_eq!(stop_with(&std_env(&fx.roots()), &fx.app), Ok(()), "{kind}");
            let stopped = fx.app.join("offsite-backup.json.stopped-20261102_090000");
            assert_eq!(std::fs::read(&stopped).unwrap(), original, "{kind}");
            assert!(!fx.state_file().exists());
            let status = fx.status();
            assert!(!status.prepared && !status.stale, "{kind}");
            assert_eq!(fx.check(), Ok(OffsiteCheckResult::NotPrepared));
            assert_eq!(snapshot(&fx.root), medium_before);
            assert_eq!(snapshot(&fx.local), local_before);

            // また用意すれば同じ目印・札で写る
            let view = fx.prepare();
            assert_eq!(view.label, "控え 1");
            assert_eq!(
                std::fs::read(fx.medium().join(MARKER_FILE)).unwrap(),
                marker
            );
            assert_eq!(view.newest_copy.as_deref(), Some(today_name().as_str()));
        }
    }

    #[test]
    fn test_stop_offsite_backup_req901_d8_failure_keeps_state() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5b 手順 3: 移せなければ何も変えない
        let fx = Fx::new();
        fx.prepare();
        let original = std::fs::read(fx.state_file()).unwrap();
        let ops = TestOps::failing(
            "publish_no_replace",
            "offsite-backup.json.stopped-20261102_090000",
            std::io::ErrorKind::Other,
        );
        assert_eq!(
            stop_with(&env_with(&fx.roots(), &ops), &fx.app),
            Err(OffsiteError::StopFailed)
        );
        assert_eq!(std::fs::read(fx.state_file()).unwrap(), original);
        // 同じ秒の退避名が既にある
        std::fs::write(
            fx.app.join("offsite-backup.json.stopped-20261102_090000"),
            b"x",
        )
        .unwrap();
        assert_eq!(
            stop_with(&std_env(&fx.roots()), &fx.app),
            Err(OffsiteError::StopFailed)
        );
        assert_eq!(std::fs::read(fx.state_file()).unwrap(), original);
        assert!(fx.status().prepared);
    }

    #[test]
    fn test_stop_offsite_backup_req901_d8_sync_failure_after_move() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5b 手順 3: 移した後の sync の失敗も StopFailed、status は用意の前
        let fx = Fx::new();
        fx.prepare();
        let ops = TestOps::failing(
            "sync_parent",
            "offsite-backup.json.stopped-20261102_090000",
            std::io::ErrorKind::Other,
        );
        assert_eq!(
            stop_with(&env_with(&fx.roots(), &ops), &fx.app),
            Err(OffsiteError::StopFailed)
        );
        assert!(!fx.state_file().exists());
        assert!(!fx.status().prepared);
    }

    // ---- 保存先の選択（§71.11.5c） ----------------------------------------------

    #[cfg(unix)]
    #[test]
    fn test_check_backup_dir_choice_req901_d11_refuses_only_medium_folder() {
        // REQ-901 / MNT-01-D11 / 71 §71.11.5c: 状態を読まず、目印のある媒体の InventoryBackup だけを止める
        use std::os::unix::fs::symlink;
        let fx = Fx::new();
        write_marker(
            &fx.root,
            &serde_json::json!({"format": 1, "medium_id": "not-in-state", "label": "控え 4", "prepared_at": "2026-10-01 09:00:00"}),
        );
        let unmarked = fx.base.join("F");
        std::fs::create_dir_all(unmarked.join(OFFSITE_DIR_NAME)).unwrap();
        let link = fx.base.join("link-to-medium");
        symlink(fx.medium(), &link).unwrap();
        let roots = [fx.root.clone(), unmarked.clone()];
        let before = (snapshot(&fx.base), names(&fx.app));

        for refused in [fx.medium(), link.clone()] {
            assert_eq!(
                choice_with(&roots, &refused),
                Err(OffsiteError::BackupDirIsMediumFolder),
                "{}",
                refused.display()
            );
        }
        for allowed in [
            fx.medium().join("sub"),
            fx.root.clone(),
            unmarked.join(OFFSITE_DIR_NAME),
            fx.local.clone(),
        ] {
            assert_eq!(
                choice_with(&roots, &allowed),
                Ok(()),
                "{}",
                allowed.display()
            );
        }
        assert_eq!((snapshot(&fx.base), names(&fx.app)), before, "書かない");
        assert!(!fx.state_file().exists());
    }

    // ---- 作り直し（§71.11.5a） ---------------------------------------------------

    fn broken_state_with_medium() -> Fx {
        let fx = Fx::new();
        fx.prepare();
        std::fs::write(fx.state_file(), b"{ broken json").unwrap();
        put_backup(&fx.local, &today_name());
        fx
    }

    #[test]
    fn test_reset_offsite_state_req901_d8_sets_aside_and_reregisters() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5a
        let fx = broken_state_with_medium();
        let marker: OffsiteMediumMarker =
            serde_json::from_slice(&std::fs::read(fx.medium().join(MARKER_FILE)).unwrap()).unwrap();
        let result = reset_with(&std_env(&fx.roots()), &fx.app, &fx.local);
        assert_eq!(
            result,
            Ok(OffsiteCheckResult::Copied {
                file_name: today_name(),
                labels: vec![marker.label.clone()],
            })
        );
        assert_eq!(
            std::fs::read(
                fx.app
                    .join("offsite-backup.json.unreadable-20261102_090000")
            )
            .unwrap(),
            b"{ broken json"
        );
        let state = fx.state();
        assert_eq!(state.media.len(), 1);
        assert_eq!(state.media[0].medium_id, marker.medium_id);
        assert_eq!(state.media[0].label, marker.label);
        assert_eq!(state.last_success.unwrap().file_name, today_name());
    }

    #[test]
    fn test_reset_offsite_state_req901_d8_unsupported_format_is_untouched() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5a 手順 2
        let fx = Fx::new();
        fx.prepare();
        write_state(&fx, &serde_json::json!({"format": 2, "media": "new shape"}));
        let before = snapshot(&fx.app);
        assert_eq!(
            reset_with(&std_env(&fx.roots()), &fx.app, &fx.local),
            Err(OffsiteError::StateUnsupported)
        );
        assert_eq!(snapshot(&fx.app), before);
    }

    #[test]
    fn test_reset_offsite_state_req901_d8_no_medium_changes_nothing() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5a 手順 3
        let fx = broken_state_with_medium();
        let before = snapshot(&fx.app);
        assert_eq!(
            reset_with(&std_env(&[]), &fx.app, &fx.local),
            Err(OffsiteError::NoMediumAttached)
        );
        assert_eq!(snapshot(&fx.app), before);
        let status = fx.status();
        assert!(status.prepared && status.stale);
        assert_eq!(
            status.last_failure_kind,
            Some(OffsiteFailureKind::StateUnreadable)
        );
    }

    #[test]
    fn test_reset_offsite_state_req901_d8_failure_keeps_unreadable_and_retry_recovers() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5a: 置き換えより前の失敗は読めない記録のまま、押し直せる
        let aside = "offsite-backup.json.unreadable-20261102_090000";
        let points: [(&'static str, &str); 6] = [
            ("write_file", "offsite-backup.json.tmp"),
            ("sync_file", "offsite-backup.json.tmp"),
            ("copy_new", aside),
            ("sync_file", aside),
            ("sync_parent", aside),
            ("replace", STATE_FILE),
        ];
        for (op, file) in points {
            let fx = broken_state_with_medium();
            let ops = TestOps::failing(op, file, std::io::ErrorKind::Other);
            assert_eq!(
                reset_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
                Err(OffsiteError::StateResetFailed),
                "{op} {file}"
            );
            assert_eq!(
                std::fs::read(fx.state_file()).unwrap(),
                b"{ broken json",
                "{op}"
            );
            assert_eq!(
                fx.status().last_failure_kind,
                Some(OffsiteFailureKind::StateUnreadable)
            );
            // 押し直し（前回の作業名・退避が残っていても、次の秒で成功する）
            let mut retry = std_env(&fx.roots());
            retry.now = now() + chrono::Duration::seconds(1);
            assert!(
                matches!(
                    reset_with(&retry, &fx.app, &fx.local),
                    Ok(OffsiteCheckResult::Copied { .. })
                ),
                "{op}"
            );
            assert_eq!(fx.state().media.len(), 1);
        }
        // 同じ秒の退避名が先にある
        let fx = broken_state_with_medium();
        std::fs::write(fx.app.join(aside), b"earlier").unwrap();
        assert_eq!(
            reset_with(&std_env(&fx.roots()), &fx.app, &fx.local),
            Err(OffsiteError::StateResetFailed)
        );
        assert_eq!(std::fs::read(fx.app.join(aside)).unwrap(), b"earlier");
        assert_eq!(std::fs::read(fx.state_file()).unwrap(), b"{ broken json");
    }

    #[test]
    fn test_reset_offsite_state_req901_d8_failure_after_replace_reports_readable_state() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5a の失敗の後の状態
        // (1) 置き換えの後の親 directory の sync
        let fx = broken_state_with_medium();
        let ops = TestOps::failing("sync_parent", STATE_FILE, std::io::ErrorKind::Other);
        assert_eq!(
            reset_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
            Err(OffsiteError::StateResetFailed)
        );
        assert_eq!(fx.state().media.len(), 1);
        let status = fx.status();
        assert_eq!(
            (status.prepared, status.stale, status.last_failure_kind),
            (true, true, None)
        );
        // (2) 手順 7 の写しの copy
        let fx = broken_state_with_medium();
        let ops = TestOps::failing(
            "copy_new",
            &partial_name(&today_name()),
            std::io::ErrorKind::Other,
        );
        assert_eq!(
            reset_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local),
            Err(OffsiteError::Io)
        );
        assert_eq!(fx.status().last_failure_kind, Some(OffsiteFailureKind::Io));
        assert!(fx
            .app
            .join("offsite-backup.json.unreadable-20261102_090000")
            .exists());
    }

    #[test]
    fn test_reset_offsite_state_req901_d8_step7_state_sync_failure_stays_copied() {
        // REQ-901 / MNT-01-D8 / 71 §71.11.5a 手順 7 の中の成功の記録の sync の失敗は成功のまま
        let fx = broken_state_with_medium();
        // 1 回目の sync_parent offsite-backup.json は作り直しの手順 6 なので、2 回目だけ失敗させる
        struct SecondStateSync {
            inner: TestOps,
            seen: StdMutex<u32>,
        }
        impl RestoreFileOps for SecondStateSync {
            fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
                self.inner.try_exists(path)
            }
            fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
                self.inner.rename(source, destination)
            }
            fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
                self.inner.replace(source, destination)
            }
            fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
                self.inner.copy(source, destination)
            }
            fn remove_file(&self, path: &Path) -> std::io::Result<()> {
                self.inner.remove_file(path)
            }
            fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
                self.inner.write_file(path, bytes)
            }
            fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
                self.inner.read_file(path)
            }
            fn sync_file(&self, path: &Path) -> std::io::Result<()> {
                self.inner.sync_file(path)
            }
            fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
                if path.file_name().unwrap() == STATE_FILE {
                    let mut seen = self.seen.lock().unwrap();
                    *seen += 1;
                    if *seen == 2 {
                        return Err(std::io::Error::other("injected success record sync"));
                    }
                }
                self.inner.sync_parent(path)
            }
            fn checkpoint(
                &self,
                conn: &db::DbConnection,
            ) -> Result<(i64, i64, i64), rusqlite::Error> {
                self.inner.checkpoint(conn)
            }
            fn copy_new(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
                self.inner.copy_new(source, destination)
            }
        }
        let ops = SecondStateSync {
            inner: TestOps::new(),
            seen: StdMutex::new(0),
        };
        let result = reset_with(&env_with(&fx.roots(), &ops), &fx.app, &fx.local);
        assert!(
            matches!(result, Ok(OffsiteCheckResult::Copied { .. })),
            "{result:?}"
        );
        assert_eq!(*ops.seen.lock().unwrap(), 2);
        let status = fx.status();
        assert_eq!(
            (
                status.stale,
                status.last_failure_kind,
                status.last_success_at.as_deref()
            ),
            (false, None, Some("2026-11-02 09:00:00"))
        );
    }

    // ---- Rollback ---------------------------------------------------------------

    #[test]
    fn test_offsite_req901_d8_files_outside_previous_version_paths() {
        // REQ-901 / Rollback: (b) の補助 file は既存の一覧・名前解析の対象外。正式名の写しだけが有効
        let fx = Fx::new();
        fx.prepare();
        put_backup(&fx.local, &today_name());
        assert!(matches!(fx.check(), Ok(OffsiteCheckResult::Copied { .. })));
        std::fs::write(fx.state_file(), b"broken").unwrap();
        assert!(reset_with(&std_env(&fx.roots()), &fx.app, &fx.local).is_ok());
        let mut stop_env = std_env(&fx.roots());
        stop_env.now = now() + chrono::Duration::seconds(1);
        stop_with(&stop_env, &fx.app).unwrap();
        // 作業名も置く
        std::fs::write(fx.app.join("offsite-backup.json.tmp"), b"tmp").unwrap();
        std::fs::write(fx.medium().join("offsite-medium.json.tmp"), b"tmp").unwrap();

        for dir in [&fx.app, &fx.local, &fx.medium()] {
            let listed: Vec<String> = list_backups(dir)
                .unwrap()
                .into_iter()
                .map(|info| info.file_name)
                .collect();
            let conforming: Vec<String> = names(dir)
                .into_iter()
                .filter(|name| extract_datetime_from_backup(name).is_some())
                .collect();
            assert_eq!(listed, conforming, "{}", dir.display());
            assert!(listed.iter().all(|name| !name.contains("offsite")));
        }
        assert_eq!(
            list_backups(&fx.medium()).unwrap()[0].file_name,
            today_name(),
            "正式名の写しは一覧に出る"
        );
        // backup_dir と app data が同じ場合も補助 file は対象外
        let same = Fx::new();
        prepare_with(&std_env(&same.roots()), &same.app, &same.app, &same.root).unwrap();
        assert_eq!(list_backups(&same.app).unwrap().len(), 0);
    }
}
