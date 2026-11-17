//! MNT-01-D1/D4/D5: crash-consistent restore transaction and startup reconciliation.

use crate::db::{self, DbConnection, NewOperationLog};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MANIFEST_SUFFIX: &str = ".restore_manifest";
const MANIFEST_TEMP_SUFFIX: &str = ".restore_manifest.tmp";
const BACKUP_SUFFIX: &str = ".restore_backup";
const RESTORE_OPERATION: &str = "backup_restore";

#[derive(Debug)]
pub enum RestoreError {
    Recovered(String),
    Unrecoverable(String),
    DurabilityUnknown(String),
}

impl fmt::Display for RestoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recovered(message)
            | Self::Unrecoverable(message)
            | Self::DurabilityUnknown(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RestoreError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestPhase {
    Active,
    Committed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreManifest {
    attempt_id: String,
    original_files: BTreeSet<String>,
    phase: ManifestPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileState {
    Clean,
    Recovered,
    CommittedPendingLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogClassification {
    NoMatch,
    AlreadyPresent,
    Failed,
}

#[derive(Clone)]
struct RestorePaths {
    main: PathBuf,
    wal: PathBuf,
    shm: PathBuf,
    main_backup: PathBuf,
    wal_backup: PathBuf,
    shm_backup: PathBuf,
    manifest: PathBuf,
    manifest_temp: PathBuf,
}

impl RestorePaths {
    fn new(db_path: &Path) -> Self {
        let named = |suffix: &str| PathBuf::from(format!("{}{suffix}", db_path.display()));
        Self {
            main: db_path.to_path_buf(),
            wal: named("-wal"),
            shm: named("-shm"),
            main_backup: named(BACKUP_SUFFIX),
            wal_backup: named(&format!("-wal{BACKUP_SUFFIX}")),
            shm_backup: named(&format!("-shm{BACKUP_SUFFIX}")),
            manifest: named(MANIFEST_SUFFIX),
            manifest_temp: named(MANIFEST_TEMP_SUFFIX),
        }
    }

    fn triples(&self) -> [(&'static str, &Path, &Path); 3] {
        [
            ("main", &self.main, &self.main_backup),
            ("wal", &self.wal, &self.wal_backup),
            ("shm", &self.shm, &self.shm_backup),
        ]
    }
}

/// file 操作の差し替え口（restore と backup の公開で共有する。71 §71.4.1）
pub(super) trait RestoreFileOps {
    fn try_exists(&self, path: &Path) -> std::io::Result<bool>;
    fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()>;
    fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()>;
    /// 既存の file を置き換えずに `source` を `destination` へ移す（71 §71.4.1）。
    /// 既存の注入の型は本物の操作へ委ねる
    fn publish_no_replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
        StdRestoreFileOps.publish_no_replace(source, destination)
    }
    fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64>;
    fn remove_file(&self, path: &Path) -> std::io::Result<()>;
    fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
    fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>>;
    fn sync_file(&self, path: &Path) -> std::io::Result<()>;
    fn sync_parent(&self, path: &Path) -> std::io::Result<()>;
    fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error>;
    /// 既存の entry があれば失敗する copy（`create_new` で開いて `io::copy`。sync は呼ぶ側。
    /// 71 §71.11.4 手順 4d・§71.11.5a 手順 5）。既存の注入の型は本物の操作へ委ねる
    fn copy_new(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
        StdRestoreFileOps.copy_new(source, destination)
    }
}

pub(super) struct StdRestoreFileOps;

impl RestoreFileOps for StdRestoreFileOps {
    fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
        path.try_exists()
    }
    fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
        std::fs::rename(source, destination)
    }
    fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
        #[cfg(not(windows))]
        {
            std::fs::rename(source, destination)
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };
            let source: Vec<u16> = source
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let destination: Vec<u16> = destination
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let result = unsafe {
                MoveFileExW(
                    source.as_ptr(),
                    destination.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if result == 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }
    fn publish_no_replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
        #[cfg(not(windows))]
        {
            // test だけの経路: hard_link は `destination` が既にあれば error を返す
            std::fs::hard_link(source, destination)?;
            std::fs::remove_file(source)
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_WRITE_THROUGH};
            let source: Vec<u16> = source
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let destination: Vec<u16> = destination
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            // 置き換えの flag を付けない: 既存の正式名があれば失敗する（71 §71.4.1）
            let result = unsafe {
                MoveFileExW(
                    source.as_ptr(),
                    destination.as_ptr(),
                    MOVEFILE_WRITE_THROUGH,
                )
            };
            if result == 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }
    fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
        std::fs::copy(source, destination)
    }
    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_file(path)
    }
    fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.write_all(bytes)
    }
    fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(path)?.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
    fn sync_file(&self, path: &Path) -> std::io::Result<()> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?
            .sync_all()
    }
    fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other("path has no parent"))?;
        #[cfg(not(windows))]
        {
            File::open(parent)?.sync_all()
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Foundation::{
                CloseHandle, GENERIC_WRITE, INVALID_HANDLE_VALUE,
            };
            use windows_sys::Win32::Storage::FileSystem::{
                CreateFileW, FlushFileBuffers, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE,
                FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
            };
            let parent: Vec<u16> = parent
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let handle = unsafe {
                CreateFileW(
                    parent.as_ptr(),
                    GENERIC_WRITE,
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
            let flushed = unsafe { FlushFileBuffers(handle) };
            let error = if flushed == 0 {
                Some(std::io::Error::last_os_error())
            } else {
                None
            };
            unsafe { CloseHandle(handle) };
            match error {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }
    fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
    }
    fn copy_new(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
        // 既存の entry（reparse point・dangling link を含む）を開かず・上書きしない
        let mut input = File::open(source)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;
        std::io::copy(&mut input, &mut output)
    }
}

fn io_message(context: &str, error: impl fmt::Display) -> String {
    format!("{context}: {error}")
}

fn exists(ops: &dyn RestoreFileOps, path: &Path) -> Result<bool, RestoreError> {
    ops.try_exists(path)
        .map_err(|error| RestoreError::Unrecoverable(io_message("ファイル存在確認に失敗", error)))
}

fn remove_if_exists(ops: &dyn RestoreFileOps, path: &Path) -> std::io::Result<bool> {
    if ops.try_exists(path)? {
        ops.remove_file(path)?;
        return Ok(true);
    }
    Ok(false)
}

fn write_manifest(
    ops: &dyn RestoreFileOps,
    paths: &RestorePaths,
    manifest: &RestoreManifest,
    replace_existing: bool,
) -> Result<(), (String, bool)> {
    let bytes = serde_json::to_vec(manifest).map_err(|error| (error.to_string(), false))?;
    ops.write_file(&paths.manifest_temp, &bytes)
        .map_err(|error| (io_message("manifest temp write", error), false))?;
    ops.sync_file(&paths.manifest_temp)
        .map_err(|error| (io_message("manifest temp sync", error), false))?;
    let rename_result = if replace_existing {
        ops.replace(&paths.manifest_temp, &paths.manifest)
    } else {
        ops.rename(&paths.manifest_temp, &paths.manifest)
    };
    rename_result.map_err(|error| (io_message("manifest rename", error), false))?;
    ops.sync_parent(&paths.manifest)
        .map_err(|error| (io_message("manifest directory sync", error), true))
}

fn delete_manifest(ops: &dyn RestoreFileOps, paths: &RestorePaths) -> std::io::Result<()> {
    remove_if_exists(ops, &paths.manifest)?;
    remove_if_exists(ops, &paths.manifest_temp)?;
    ops.sync_parent(&paths.manifest)
}

fn actual_backup_set(
    ops: &dyn RestoreFileOps,
    paths: &RestorePaths,
) -> std::io::Result<BTreeSet<String>> {
    let mut set = BTreeSet::new();
    for (name, _, backup) in paths.triples() {
        if ops.try_exists(backup)? {
            set.insert(name.to_string());
        }
    }
    Ok(set)
}

fn restore_recorded_backups(
    ops: &dyn RestoreFileOps,
    paths: &RestorePaths,
    names: &BTreeSet<String>,
    remove_all_originals: bool,
) -> std::io::Result<()> {
    if remove_all_originals {
        for (_, original, _) in paths.triples() {
            remove_if_exists(ops, original)?;
        }
        // R1/step8 は不可信な元名側を全削除した事実を永続化してから旧世代を戻す。
        ops.sync_parent(&paths.main)?;
    }
    for (name, original, backup) in paths.triples() {
        if names.contains(name) {
            if !remove_all_originals {
                remove_if_exists(ops, original)?;
            }
            ops.rename(backup, original)?;
            ops.sync_parent(original)?;
        }
    }
    if names.is_empty() || (remove_all_originals && !names.contains("main")) {
        ops.sync_parent(&paths.main)?;
    }
    Ok(())
}

fn rollback_after_failure(
    ops: &dyn RestoreFileOps,
    paths: &RestorePaths,
    evacuated: &BTreeSet<String>,
    reason: String,
    remove_new_generation: bool,
) -> RestoreError {
    if remove_new_generation {
        for (_, original, _) in paths.triples() {
            if let Err(error) = remove_if_exists(ops, original) {
                tracing::error!(%error, path = %original.display(), "新世代ファイルの除去に失敗");
                return RestoreError::Unrecoverable(format!(
                    "{reason}; 新世代ファイル除去にも失敗: {error}"
                ));
            }
        }
    }
    match restore_recorded_backups(ops, paths, evacuated, false) {
        Ok(()) => {
            if let Err(error) = delete_manifest(ops, paths) {
                tracing::error!(%error, "同期巻き戻し後の manifest 削除に失敗");
                return RestoreError::Unrecoverable(format!("{reason}; manifest cleanup: {error}"));
            }
            RestoreError::Recovered(reason)
        }
        Err(error) => {
            tracing::error!(%error, "退避巻き戻し自体に失敗（致命的）");
            RestoreError::Unrecoverable(format!("{reason}; 退避巻き戻しにも失敗: {error}"))
        }
    }
}

pub(super) fn restore_backup(
    current_conn: DbConnection,
    backup_path: &Path,
    db_path: &Path,
) -> Result<DbConnection, RestoreError> {
    restore_backup_with_ops(current_conn, backup_path, db_path, &StdRestoreFileOps)
}

fn restore_backup_with_ops(
    current_conn: DbConnection,
    backup_path: &Path,
    db_path: &Path,
    ops: &dyn RestoreFileOps,
) -> Result<DbConnection, RestoreError> {
    let paths = RestorePaths::new(db_path);
    if !exists(ops, backup_path)? {
        return Err(RestoreError::Recovered(
            "復元対象が見つかりません".to_string(),
        ));
    }

    let temp_exists = exists(ops, &paths.manifest_temp)?;
    if temp_exists {
        return Err(RestoreError::Unrecoverable(
            "前回の復元 manifest 一時ファイルが残っています。アプリを再起動してください"
                .to_string(),
        ));
    }
    if exists(ops, &paths.manifest)? {
        let bytes = ops
            .read_file(&paths.manifest)
            .map_err(|e| RestoreError::Unrecoverable(io_message("residual manifest read", e)))?;
        let residual: RestoreManifest = serde_json::from_slice(&bytes)
            .map_err(|e| RestoreError::Unrecoverable(io_message("residual manifest parse", e)))?;
        if residual.phase != ManifestPhase::Committed {
            return Err(RestoreError::Unrecoverable(
                "前回の active restore が残っています。アプリを再起動してください".to_string(),
            ));
        }
        cleanup_committed(
            &current_conn,
            ops,
            &paths,
            &residual,
            true,
            "バックアップからの復元を起動時に確定しました",
        )
        .map_err(RestoreError::Unrecoverable)?;
    } else if !actual_backup_set(ops, &paths)
        .map_err(|e| RestoreError::Unrecoverable(e.to_string()))?
        .is_empty()
    {
        return Err(RestoreError::Unrecoverable(
            "前回の復元遺物が残っています。アプリを再起動してください".to_string(),
        ));
    }

    match ops.checkpoint(&current_conn) {
        Ok((busy, log_frames, checkpointed_frames)) => {
            if busy == 0 {
                tracing::info!(
                    busy,
                    log_frames,
                    checkpointed_frames,
                    "restore checkpoint completed"
                );
            } else {
                tracing::warn!(
                    busy,
                    log_frames,
                    checkpointed_frames,
                    "restore checkpoint incomplete; WAL/SHM 一式退避を継続"
                );
            }
        }
        Err(error) => {
            tracing::warn!(%error, "restore checkpoint failed; WAL/SHM 一式退避を継続");
        }
    }

    // 接続 close により空 WAL/SHM が消える場合があるため、記録集合は close 後に確定する。
    drop(current_conn);

    let mut original_files = BTreeSet::new();
    for (name, original, _) in paths.triples() {
        if exists(ops, original)? {
            original_files.insert(name.to_string());
        }
    }
    if !original_files.contains("main") {
        return Err(RestoreError::Unrecoverable(
            "現在のDB本体がありません".to_string(),
        ));
    }

    let mut manifest = RestoreManifest {
        attempt_id: uuid::Uuid::new_v4().to_string(),
        original_files: original_files.clone(),
        phase: ManifestPhase::Active,
    };
    if let Err((message, _renamed)) = write_manifest(ops, &paths, &manifest, false) {
        if let Err(error) = remove_if_exists(ops, &paths.manifest_temp) {
            tracing::warn!(path = %paths.manifest_temp.display(), error = %error, "復元manifest一時ファイルのcleanupに失敗（継続）");
        }
        return Err(RestoreError::Recovered(message));
    }

    let mut evacuated = BTreeSet::new();
    for (name, original, backup) in paths.triples() {
        if original_files.contains(name) {
            if let Err(error) = ops.rename(original, backup) {
                let reason = io_message(&format!("{name} の退避に失敗"), error);
                return Err(rollback_after_failure(
                    ops, &paths, &evacuated, reason, false,
                ));
            }
            evacuated.insert(name.to_string());
            if let Err(error) = ops.sync_parent(original) {
                return Err(rollback_after_failure(
                    ops,
                    &paths,
                    &evacuated,
                    io_message(&format!("{name} 退避 directory sync に失敗"), error),
                    false,
                ));
            }
        }
    }

    let install_result = (|| -> Result<DbConnection, String> {
        ops.copy(backup_path, &paths.main)
            .map_err(|error| io_message("復元DB copy に失敗", error))?;
        ops.sync_file(&paths.main)
            .map_err(|error| io_message("復元DB sync に失敗", error))?;
        ops.sync_parent(&paths.main)
            .map_err(|error| io_message("復元DB directory sync に失敗", error))?;
        db::open_existing_database(paths.main.to_str().unwrap_or(""))
            .map_err(|error| io_message("復元DB open に失敗", error))
    })();

    let new_conn = match install_result {
        Ok(conn) => conn,
        Err(reason) => {
            return Err(rollback_after_failure(
                ops, &paths, &evacuated, reason, true,
            ))
        }
    };

    manifest.phase = ManifestPhase::Committed;
    if let Err((message, renamed)) = write_manifest(ops, &paths, &manifest, true) {
        if renamed {
            tracing::error!(%message, "committed manifest の永続性が不明");
            return Err(RestoreError::DurabilityUnknown(message));
        }
        drop(new_conn);
        tracing::error!(%message, "committed manifest 更新前に失敗。active reconcile を要求");
        return Err(RestoreError::Unrecoverable(message));
    }

    // committed 後の cleanup/log 失敗は復元成功を取り消さない。次回起動で補完する。
    let backup_name = backup_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("不明なバックアップ");
    let summary = format!("バックアップから復元しました: {backup_name}");
    if let Err(error) = cleanup_committed(&new_conn, ops, &paths, &manifest, false, &summary) {
        tracing::warn!(%error, attempt_id = %manifest.attempt_id, "復元後処理を次回起動へ持ち越し");
    }
    Ok(new_conn)
}

fn classify_restore_log(
    conn: &DbConnection,
    attempt_id: &str,
) -> Result<LogClassification, crate::db::DbError> {
    let mut statement =
        conn.prepare("SELECT detail_json FROM operation_logs WHERE operation_type = ?1")?;
    let rows = statement.query_map([RESTORE_OPERATION], |row| row.get::<_, Option<String>>(0))?;
    let mut exact_match = false;
    let mut failed = false;
    for row in rows {
        let detail = match row {
            Ok(detail) => detail,
            Err(_) => {
                failed = true;
                continue;
            }
        };
        let Some(detail) = detail else {
            continue;
        };
        match serde_json::from_str::<serde_json::Value>(&detail) {
            Ok(value) if value.get("attempt_id").and_then(|id| id.as_str()) == Some(attempt_id) => {
                exact_match = true;
            }
            Ok(value) => match value.get("attempt_id") {
                None => {}
                Some(id) if id.as_str().is_some() => {}
                Some(_) => failed = true,
            },
            Err(_) => failed = true,
        }
    }
    Ok(if exact_match {
        LogClassification::AlreadyPresent
    } else if failed {
        LogClassification::Failed
    } else {
        LogClassification::NoMatch
    })
}

fn cleanup_committed(
    conn: &DbConnection,
    ops: &dyn RestoreFileOps,
    paths: &RestorePaths,
    manifest: &RestoreManifest,
    delete_on_failed_classification: bool,
    summary: &str,
) -> Result<(), String> {
    for (_, _, backup) in paths.triples() {
        if remove_if_exists(ops, backup).map_err(|e| io_message("退避 cleanup", e))? {
            ops.sync_parent(backup)
                .map_err(|e| io_message("退避 cleanup sync", e))?;
        }
    }

    let classification = match classify_restore_log(conn, &manifest.attempt_id) {
        Ok(classification) => classification,
        Err(error) if delete_on_failed_classification => {
            tracing::warn!(
                %error,
                attempt_id = %manifest.attempt_id,
                "restore log lookup に失敗したため best-effort で committed manifest を解放"
            );
            return delete_manifest(ops, paths)
                .map_err(|e| io_message("committed manifest cleanup", e));
        }
        Err(error) => return Err(error.to_string()),
    };

    match classification {
        LogClassification::AlreadyPresent => {}
        LogClassification::NoMatch => {
            if let Err(error) = db::system_repo::insert_operation_log(
                conn,
                &NewOperationLog {
                    operation_type: RESTORE_OPERATION.to_string(),
                    summary: summary.to_string(),
                    detail_json: Some(
                        serde_json::json!({ "attempt_id": manifest.attempt_id }).to_string(),
                    ),
                },
            ) {
                if delete_on_failed_classification {
                    tracing::warn!(
                        %error,
                        attempt_id = %manifest.attempt_id,
                        "restore log INSERT に失敗したため best-effort で committed manifest を解放"
                    );
                } else {
                    return Err(error.to_string());
                }
            }
        }
        LogClassification::Failed if !delete_on_failed_classification => {
            return Err("既存 restore log の分類に失敗".to_string())
        }
        LogClassification::Failed => {
            tracing::warn!(attempt_id = %manifest.attempt_id, "分類不能 restore log を残して committed manifest を解放");
        }
    }
    delete_manifest(ops, paths).map_err(|e| io_message("committed manifest cleanup", e))
}

pub(super) fn reconcile_restore(db_path: &Path) -> Result<ReconcileState, RestoreError> {
    reconcile_restore_with_ops(db_path, &StdRestoreFileOps)
}

fn reconcile_restore_with_ops(
    db_path: &Path,
    ops: &dyn RestoreFileOps,
) -> Result<ReconcileState, RestoreError> {
    let paths = RestorePaths::new(db_path);
    if remove_if_exists(ops, &paths.manifest_temp)
        .map_err(|e| RestoreError::Unrecoverable(io_message("T0 cleanup", e)))?
    {
        ops.sync_parent(&paths.manifest_temp)
            .map_err(|e| RestoreError::Unrecoverable(io_message("T0 sync", e)))?;
    }
    let backups = actual_backup_set(ops, &paths)
        .map_err(|e| RestoreError::Unrecoverable(io_message("退避集合の確認", e)))?;
    if !exists(ops, &paths.manifest)? {
        return if backups.is_empty() {
            Ok(ReconcileState::Clean)
        } else {
            Err(RestoreError::Unrecoverable(
                "manifest なしで復元退避ファイルが残っています".to_string(),
            ))
        };
    }

    let bytes = match ops.read_file(&paths.manifest) {
        Ok(bytes) => bytes,
        Err(error) if backups.is_empty() => {
            delete_manifest(ops, &paths)
                .map_err(|e| RestoreError::Unrecoverable(io_message("R6 cleanup", e)))?;
            tracing::warn!(%error, "退避を伴わない読取不能 manifest を削除");
            return Ok(ReconcileState::Clean);
        }
        Err(error) => {
            return Err(RestoreError::Unrecoverable(io_message(
                "退避を伴う manifest を読み取れない",
                error,
            )))
        }
    };
    let manifest: RestoreManifest = match serde_json::from_slice(&bytes) {
        Ok(manifest) => manifest,
        Err(error) if backups.is_empty() => {
            delete_manifest(ops, &paths)
                .map_err(|e| RestoreError::Unrecoverable(io_message("R6 cleanup", e)))?;
            tracing::warn!(%error, "退避を伴わない不正 manifest を削除");
            return Ok(ReconcileState::Clean);
        }
        Err(error) => {
            return Err(RestoreError::Unrecoverable(io_message(
                "退避を伴う manifest が不正",
                error,
            )))
        }
    };

    match manifest.phase {
        ManifestPhase::Committed => {
            for (_, _, backup) in paths.triples() {
                if remove_if_exists(ops, backup)
                    .map_err(|e| RestoreError::Unrecoverable(io_message("R4 cleanup", e)))?
                {
                    ops.sync_parent(backup)
                        .map_err(|e| RestoreError::Unrecoverable(io_message("R4 sync", e)))?;
                }
            }
            Ok(ReconcileState::CommittedPendingLog)
        }
        ManifestPhase::Active if backups == manifest.original_files => {
            restore_recorded_backups(ops, &paths, &backups, true)
                .map_err(|e| RestoreError::Unrecoverable(io_message("R1/R2 reconcile", e)))?;
            delete_manifest(ops, &paths).map_err(|e| {
                RestoreError::Unrecoverable(io_message("R1/R2 manifest cleanup", e))
            })?;
            Ok(ReconcileState::Recovered)
        }
        ManifestPhase::Active if backups.is_subset(&manifest.original_files) => {
            restore_recorded_backups(ops, &paths, &backups, false)
                .map_err(|e| RestoreError::Unrecoverable(io_message("R1/R2 reconcile", e)))?;
            delete_manifest(ops, &paths).map_err(|e| {
                RestoreError::Unrecoverable(io_message("R1/R2 manifest cleanup", e))
            })?;
            Ok(ReconcileState::Recovered)
        }
        ManifestPhase::Active => Err(RestoreError::Unrecoverable(
            "manifest 記録外の復元退避ファイルがあります".to_string(),
        )),
    }
}

pub(super) fn complete_reconciled_restore(
    conn: &DbConnection,
    db_path: &Path,
) -> Result<(), RestoreError> {
    let ops = StdRestoreFileOps;
    let paths = RestorePaths::new(db_path);
    if !exists(&ops, &paths.manifest)? {
        return Ok(());
    }
    let bytes = ops
        .read_file(&paths.manifest)
        .map_err(|e| RestoreError::Unrecoverable(io_message("manifest read", e)))?;
    let manifest: RestoreManifest = serde_json::from_slice(&bytes)
        .map_err(|e| RestoreError::Unrecoverable(io_message("committed manifest parse", e)))?;
    if manifest.phase != ManifestPhase::Committed {
        return Err(RestoreError::Unrecoverable(
            "active manifest が初期化後まで残っています".to_string(),
        ));
    }
    cleanup_committed(
        conn,
        &ops,
        &paths,
        &manifest,
        false,
        "バックアップからの復元を起動時に確定しました",
    )
    .map_err(RestoreError::Unrecoverable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum InjectedFailure {
        None,
        EvacuateWal,
        EvacuateShm,
        CorruptInstall,
        RollbackOfRollback,
        RollbackWalAfterMain,
        CommittedDirectorySync,
        CheckpointSql,
        ManifestRead,
        ManifestTempWriteAndCleanup,
    }

    struct InjectedOps {
        busy_checkpoint: bool,
        failure: InjectedFailure,
        parent_sync_count: Mutex<usize>,
    }

    impl InjectedOps {
        fn new(failure: InjectedFailure) -> Self {
            Self {
                busy_checkpoint: false,
                failure,
                parent_sync_count: Mutex::new(0),
            }
        }
        fn busy() -> Self {
            Self {
                busy_checkpoint: true,
                ..Self::new(InjectedFailure::None)
            }
        }
    }

    impl RestoreFileOps for InjectedOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            let destination_text = destination.to_string_lossy();
            let source_text = source.to_string_lossy();
            if matches!(
                self.failure,
                InjectedFailure::EvacuateWal | InjectedFailure::RollbackOfRollback
            ) && destination_text.ends_with("-wal.restore_backup")
            {
                return Err(std::io::Error::other("injected WAL evacuation failure"));
            }
            if matches!(
                self.failure,
                InjectedFailure::EvacuateShm | InjectedFailure::RollbackWalAfterMain
            ) && destination_text.ends_with("-shm.restore_backup")
            {
                return Err(std::io::Error::other("injected SHM evacuation failure"));
            }
            if self.failure == InjectedFailure::RollbackOfRollback
                && source_text.ends_with(".restore_backup")
                && !source_text.ends_with("-wal.restore_backup")
            {
                return Err(std::io::Error::other(
                    "injected rollback-of-rollback failure",
                ));
            }
            if self.failure == InjectedFailure::RollbackWalAfterMain
                && source_text.ends_with("-wal.restore_backup")
            {
                return Err(std::io::Error::other(
                    "injected WAL rollback-after-main failure",
                ));
            }
            StdRestoreFileOps.rename(source, destination)
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.replace(source, destination)
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            if matches!(
                self.failure,
                InjectedFailure::CorruptInstall | InjectedFailure::RollbackOfRollback
            ) {
                std::fs::write(destination, b"not a sqlite database")?;
                std::fs::write(
                    PathBuf::from(format!("{}-wal", destination.display())),
                    b"new-wal",
                )?;
                std::fs::write(
                    PathBuf::from(format!("{}-shm", destination.display())),
                    b"new-shm",
                )?;
                return Ok(21);
            }
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            if self.failure == InjectedFailure::ManifestTempWriteAndCleanup
                && path.to_string_lossy().ends_with(MANIFEST_TEMP_SUFFIX)
            {
                return Err(std::io::Error::other(
                    "injected manifest temp cleanup failure",
                ));
            }
            StdRestoreFileOps.remove_file(path)
        }
        fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            if self.failure == InjectedFailure::ManifestTempWriteAndCleanup
                && path.to_string_lossy().ends_with(MANIFEST_TEMP_SUFFIX)
            {
                StdRestoreFileOps.write_file(path, bytes)?;
                return Err(std::io::Error::other(
                    "injected manifest temp write failure",
                ));
            }
            StdRestoreFileOps.write_file(path, bytes)
        }
        fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            if self.failure == InjectedFailure::ManifestRead {
                return Err(std::io::Error::other("injected manifest read failure"));
            }
            StdRestoreFileOps.read_file(path)
        }
        fn sync_file(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.sync_file(path)
        }
        fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
            let mut count = self.parent_sync_count.lock().unwrap();
            *count += 1;
            if self.failure == InjectedFailure::CommittedDirectorySync && *count == 4 {
                return Err(std::io::Error::other(
                    "injected committed directory sync failure",
                ));
            }
            StdRestoreFileOps.sync_parent(path)
        }
        fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            if self.failure == InjectedFailure::CheckpointSql {
                Err(rusqlite::Error::InvalidQuery)
            } else if self.busy_checkpoint {
                Ok((1, 1, 0))
            } else {
                StdRestoreFileOps.checkpoint(conn)
            }
        }
    }

    struct CommittedInterruptOps {
        fail_at: usize,
        committed: Mutex<bool>,
        operation_count: Mutex<usize>,
    }

    struct RecordingOps {
        events: Mutex<Vec<&'static str>>,
    }

    struct ReconcileInterruptOps {
        fail_after: usize,
        operation_count: Mutex<usize>,
    }

    #[derive(Clone, Copy)]
    enum PhaseUpdateFailure {
        TempWrite,
        TempSync,
        CanonicalReplace,
    }

    struct PhaseUpdateFailureOps {
        stage: PhaseUpdateFailure,
        manifest_write_count: Mutex<usize>,
        manifest_sync_count: Mutex<usize>,
    }

    impl PhaseUpdateFailureOps {
        fn new(stage: PhaseUpdateFailure) -> Self {
            Self {
                stage,
                manifest_write_count: Mutex::new(0),
                manifest_sync_count: Mutex::new(0),
            }
        }
    }

    impl RestoreFileOps for PhaseUpdateFailureOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.rename(source, destination)
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            if matches!(self.stage, PhaseUpdateFailure::CanonicalReplace) {
                return Err(std::io::Error::other("injected canonical replace failure"));
            }
            StdRestoreFileOps.replace(source, destination)
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.remove_file(path)
        }
        fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            if path.to_string_lossy().ends_with(MANIFEST_TEMP_SUFFIX) {
                let mut count = self.manifest_write_count.lock().unwrap();
                *count += 1;
                if *count == 2 && matches!(self.stage, PhaseUpdateFailure::TempWrite) {
                    return Err(std::io::Error::other(
                        "injected committed temp write failure",
                    ));
                }
            }
            StdRestoreFileOps.write_file(path, bytes)
        }
        fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            StdRestoreFileOps.read_file(path)
        }
        fn sync_file(&self, path: &Path) -> std::io::Result<()> {
            if path.to_string_lossy().ends_with(MANIFEST_TEMP_SUFFIX) {
                let mut count = self.manifest_sync_count.lock().unwrap();
                *count += 1;
                if *count == 2 && matches!(self.stage, PhaseUpdateFailure::TempSync) {
                    return Err(std::io::Error::other(
                        "injected committed temp sync failure",
                    ));
                }
            }
            StdRestoreFileOps.sync_file(path)
        }
        fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.sync_parent(path)
        }
        fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            StdRestoreFileOps.checkpoint(conn)
        }
    }

    impl ReconcileInterruptOps {
        fn new(fail_after: usize) -> Self {
            Self {
                fail_after,
                operation_count: Mutex::new(0),
            }
        }

        fn after_mutation(&self) -> std::io::Result<()> {
            let mut count = self.operation_count.lock().unwrap();
            *count += 1;
            if *count == self.fail_after {
                Err(std::io::Error::other(format!(
                    "injected reconcile interruption after operation {}",
                    self.fail_after
                )))
            } else {
                Ok(())
            }
        }
    }

    impl RestoreFileOps for ReconcileInterruptOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.rename(source, destination)?;
            self.after_mutation()
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.replace(source, destination)?;
            self.after_mutation()
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.remove_file(path)?;
            self.after_mutation()
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
            StdRestoreFileOps.sync_parent(path)?;
            self.after_mutation()
        }
        fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            StdRestoreFileOps.checkpoint(conn)
        }
    }

    impl RecordingOps {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }
        fn record(&self, event: &'static str) {
            self.events.lock().unwrap().push(event);
        }
    }

    impl RestoreFileOps for RecordingOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            self.record("rename");
            StdRestoreFileOps.rename(source, destination)
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            self.record("replace");
            StdRestoreFileOps.replace(source, destination)
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.remove_file(path)
        }
        fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            self.record("write");
            StdRestoreFileOps.write_file(path, bytes)
        }
        fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            StdRestoreFileOps.read_file(path)
        }
        fn sync_file(&self, path: &Path) -> std::io::Result<()> {
            self.record("sync_file");
            StdRestoreFileOps.sync_file(path)
        }
        fn sync_parent(&self, path: &Path) -> std::io::Result<()> {
            self.record("sync_parent");
            StdRestoreFileOps.sync_parent(path)
        }
        fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            StdRestoreFileOps.checkpoint(conn)
        }
    }

    impl CommittedInterruptOps {
        fn new(fail_at: usize) -> Self {
            Self {
                fail_at,
                committed: Mutex::new(false),
                operation_count: Mutex::new(0),
            }
        }
        fn maybe_interrupt(&self) -> std::io::Result<()> {
            if !*self.committed.lock().unwrap() {
                return Ok(());
            }
            let mut count = self.operation_count.lock().unwrap();
            *count += 1;
            if *count == self.fail_at {
                Err(std::io::Error::other(format!(
                    "injected committed interruption {}",
                    self.fail_at
                )))
            } else {
                Ok(())
            }
        }
    }

    impl RestoreFileOps for CommittedInterruptOps {
        fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
            StdRestoreFileOps.try_exists(path)
        }
        fn rename(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.rename(source, destination)
        }
        fn replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdRestoreFileOps.replace(source, destination)?;
            *self.committed.lock().unwrap() = true;
            Ok(())
        }
        fn copy(&self, source: &Path, destination: &Path) -> std::io::Result<u64> {
            StdRestoreFileOps.copy(source, destination)
        }
        fn remove_file(&self, path: &Path) -> std::io::Result<()> {
            self.maybe_interrupt()?;
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
            self.maybe_interrupt()?;
            StdRestoreFileOps.sync_parent(path)
        }
        fn checkpoint(&self, conn: &DbConnection) -> Result<(i64, i64, i64), rusqlite::Error> {
            StdRestoreFileOps.checkpoint(conn)
        }
    }

    fn database_with_supplier(path: &Path, name: &str) -> DbConnection {
        let conn = db::init_database(path.to_str().unwrap()).unwrap();
        conn.execute(
            "INSERT INTO suppliers (name, created_at) VALUES (?1, '2026-07-18T00:00:00')",
            [name],
        )
        .unwrap();
        conn
    }

    fn supplier_names(conn: &DbConnection) -> Vec<String> {
        let mut statement = conn
            .prepare("SELECT name FROM suppliers ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn write_test_manifest(
        paths: &RestorePaths,
        phase: ManifestPhase,
        files: &[&str],
        attempt: &str,
    ) {
        let manifest = RestoreManifest {
            attempt_id: attempt.to_string(),
            original_files: files.iter().map(|name| (*name).to_string()).collect(),
            phase,
        };
        write_manifest(&StdRestoreFileOps, paths, &manifest, false).unwrap();
    }

    fn assert_no_restore_artifacts(paths: &RestorePaths) {
        for path in [
            &paths.main_backup,
            &paths.wal_backup,
            &paths.shm_backup,
            &paths.manifest,
            &paths.manifest_temp,
        ] {
            assert!(!path.exists(), "unexpected artifact: {}", path.display());
        }
    }

    #[test]
    fn test_restore_req901_manifest_temp_cleanup_failure_warns_and_preserves_recovered_error() {
        // REQ-901 / MNT-01-D1/D4/D5/D6: initial manifest write の主失敗を
        // cleanup failure で置換せず、path/error/context 付き WARN を残す
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let conn = database_with_supplier(&db_path, "old");
        drop(database_with_supplier(&backup_path, "new"));
        let paths = RestorePaths::new(&db_path);
        let ops = InjectedOps::new(InjectedFailure::ManifestTempWriteAndCleanup);

        let (error, logs) = crate::test_tracing::capture(|| {
            restore_backup_with_ops(conn, &backup_path, &db_path, &ops).unwrap_err()
        });

        match error {
            RestoreError::Recovered(message) => {
                assert!(message.contains("injected manifest temp write failure"));
                assert!(!message.contains("cleanup failure"));
            }
            other => panic!("Recovered が期待されるが {other:?}"),
        }
        assert!(logs.contains("WARN") && logs.contains("cleanupに失敗"));
        assert!(logs.contains("injected manifest temp cleanup failure"));
        assert!(logs.contains(paths.manifest_temp.to_string_lossy().as_ref()));
        assert!(paths.main.exists(), "manifest 前なので現行 DB は未変更");
        assert!(!paths.main_backup.exists());
        assert!(!paths.manifest.exists());
        assert!(
            paths.manifest_temp.exists(),
            "注入した cleanup failure の残置を観測する"
        );
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        assert_eq!(supplier_names(&reopened), vec!["old"]);
    }

    #[test]
    fn test_restore_req901_b1_evacuation_failure_preserves_real_wal_snapshot() {
        // REQ-901 / Matrix B1: WAL frame existence assert + deterministic rename failpoint.
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let conn = database_with_supplier(&db_path, "old-main");
        conn.execute_batch("PRAGMA wal_autocheckpoint=0; INSERT INTO suppliers (name, created_at) VALUES ('old-wal', '2026-07-18T00:00:01');").unwrap();
        let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
        assert!(
            std::fs::metadata(&wal_path).unwrap().len() > 32,
            "WAL frame fixture required"
        );
        let reader = rusqlite::Connection::open(&db_path).unwrap();
        reader
            .execute_batch("BEGIN; SELECT COUNT(*) FROM suppliers;")
            .unwrap();
        drop(database_with_supplier(&backup_path, "new"));

        for failure in [InjectedFailure::EvacuateWal, InjectedFailure::EvacuateShm] {
            let ops = InjectedOps {
                busy_checkpoint: true,
                ..InjectedOps::new(failure)
            };
            let current = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
            let error = restore_backup_with_ops(current, &backup_path, &db_path, &ops).unwrap_err();
            assert!(matches!(error, RestoreError::Recovered(_)));
            let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
            assert_eq!(supplier_names(&reopened), vec!["old-main", "old-wal"]);
            drop(reopened);
            assert_no_restore_artifacts(&RestorePaths::new(&db_path));
        }
        drop(conn);
        drop(reader);
    }

    #[test]
    fn test_restore_req901_b2_checkpoint_complete_and_busy_never_mix_old_wal() {
        // REQ-901 / Matrix B2: 3列標準系・busy=1・SQL Err のいずれも一式退避で混在を防ぐ。
        for ops in [
            InjectedOps::new(InjectedFailure::None),
            InjectedOps::busy(),
            InjectedOps::new(InjectedFailure::CheckpointSql),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("inventory.db");
            let backup_path = dir.path().join("replacement.db");
            let conn = database_with_supplier(&db_path, "old");
            conn.execute_batch("PRAGMA wal_autocheckpoint=0; INSERT INTO suppliers (name, created_at) VALUES ('old-wal', '2026-07-18T00:00:01');").unwrap();
            assert!(
                std::fs::metadata(format!("{}-wal", db_path.display()))
                    .unwrap()
                    .len()
                    > 32
            );
            drop(database_with_supplier(&backup_path, "new-only"));
            let restored = restore_backup_with_ops(conn, &backup_path, &db_path, &ops).unwrap();
            assert_eq!(supplier_names(&restored), vec!["new-only"]);
            drop(restored);
            let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
            assert_eq!(supplier_names(&reopened), vec!["new-only"]);
        }
    }

    #[test]
    fn test_restore_req901_newer_schema_backup_rolls_back_to_current() {
        // REQ-901 / MNT-03-D11（波及）/ Matrix T10: 新しすぎる版の backup は差し替え後の open で拒否され、現在の DB へ戻る
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let conn = database_with_supplier(&db_path, "old");
        let newer = crate::db::test_support::write_newer_schema_db(&backup_path);
        {
            let backup = rusqlite::Connection::open(&backup_path).unwrap();
            backup
                .execute(
                    "INSERT INTO suppliers (name, created_at) VALUES ('new', '2026-07-18T00:00:00')",
                    [],
                )
                .unwrap();
        }

        let error = restore_backup_with_ops(
            conn,
            &backup_path,
            &db_path,
            &InjectedOps::new(InjectedFailure::None),
        )
        .unwrap_err();

        match error {
            RestoreError::Recovered(message) => {
                assert!(
                    message.contains(&format!("データの版: {newer}")),
                    "版の detail が必要: {message}"
                );
            }
            other => panic!("Recovered が期待されるが {other:?}"),
        }
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        assert_eq!(supplier_names(&reopened), vec!["old"]);
        drop(reopened);
        assert_no_restore_artifacts(&RestorePaths::new(&db_path));
    }

    #[test]
    fn test_restore_req901_b5_removes_new_generation_sidecars_on_rollback() {
        // REQ-901 / Matrix B5
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let conn = database_with_supplier(&db_path, "old");
        drop(database_with_supplier(&backup_path, "new"));
        let error = restore_backup_with_ops(
            conn,
            &backup_path,
            &db_path,
            &InjectedOps::new(InjectedFailure::CorruptInstall),
        )
        .unwrap_err();
        assert!(matches!(error, RestoreError::Recovered(_)));
        assert!(!PathBuf::from(format!("{}-wal", db_path.display())).exists());
        assert!(!PathBuf::from(format!("{}-shm", db_path.display())).exists());
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        assert_eq!(supplier_names(&reopened), vec!["old"]);
    }

    #[test]
    fn test_restore_req901_b12_rollback_failure_is_fatal_and_reconcilable() {
        // REQ-901 / Matrix B3, B12
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let conn = database_with_supplier(&db_path, "old");
        drop(database_with_supplier(&backup_path, "new"));
        let (error, logs) = crate::test_tracing::capture(|| {
            restore_backup_with_ops(
                conn,
                &backup_path,
                &db_path,
                &InjectedOps::new(InjectedFailure::RollbackOfRollback),
            )
            .unwrap_err()
        });
        assert!(matches!(error, RestoreError::Unrecoverable(_)));
        assert!(logs.contains("ERROR"), "captured logs: {logs:?}");
        assert!(logs.contains("退避巻き戻し自体に失敗（致命的）"));
        let paths = RestorePaths::new(&db_path);
        assert!(paths.main_backup.exists() && paths.manifest.exists());
        assert!(db::open_existing_database(db_path.to_str().unwrap()).is_err());
        assert!(
            !db_path.exists(),
            "NO_CREATE recovery must not manufacture an empty DB"
        );
        assert_eq!(
            reconcile_restore(&db_path).unwrap(),
            ReconcileState::Recovered
        );
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        assert_eq!(supplier_names(&reopened), vec!["old"]);
        assert_no_restore_artifacts(&paths);
    }

    #[test]
    fn test_restore_req901_b12_partial_rollback_failure_reconciles_remaining_wal() {
        // REQ-901 / Matrix B12: main 復帰後の WAL 巻き戻し失敗も R2 へ収束する。
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let conn = database_with_supplier(&db_path, "old-main");
        conn.execute_batch(
            "PRAGMA wal_autocheckpoint=0;
             INSERT INTO suppliers (name, created_at)
             VALUES ('old-wal', '2026-07-18T00:00:01');",
        )
        .unwrap();
        let reader = rusqlite::Connection::open(&db_path).unwrap();
        reader
            .execute_batch("BEGIN; SELECT COUNT(*) FROM suppliers;")
            .unwrap();
        drop(database_with_supplier(&backup_path, "new"));
        let ops = InjectedOps {
            busy_checkpoint: true,
            ..InjectedOps::new(InjectedFailure::RollbackWalAfterMain)
        };

        let error = restore_backup_with_ops(conn, &backup_path, &db_path, &ops).unwrap_err();
        assert!(matches!(error, RestoreError::Unrecoverable(_)));
        let paths = RestorePaths::new(&db_path);
        assert!(paths.main.exists());
        assert!(paths.wal_backup.exists() && paths.manifest.exists());
        drop(reader);
        assert_eq!(
            reconcile_restore(&db_path).unwrap(),
            ReconcileState::Recovered
        );
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        assert_eq!(supplier_names(&reopened), vec!["old-main", "old-wal"]);
        assert_no_restore_artifacts(&paths);
    }

    #[test]
    fn test_restore_req901_b4_reconcile_r1_r2_r4_and_reinterruption() {
        // REQ-901 / Matrix B4, B9
        for subset in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("inventory.db");
            let paths = RestorePaths::new(&db_path);
            drop(database_with_supplier(&db_path, "old"));
            std::fs::rename(&paths.main, &paths.main_backup).unwrap();
            if subset {
                drop(database_with_supplier(&paths.main, "interrupted-new"));
                write_test_manifest(&paths, ManifestPhase::Active, &["main", "wal"], "r2");
            } else {
                write_test_manifest(&paths, ManifestPhase::Active, &["main"], "r1");
            }
            assert_eq!(
                reconcile_restore(&db_path).unwrap(),
                ReconcileState::Recovered
            );
            let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
            assert_eq!(supplier_names(&reopened), vec!["old"]);
            assert_no_restore_artifacts(&paths);
        }

        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let paths = RestorePaths::new(&db_path);
        drop(database_with_supplier(&db_path, "new"));
        std::fs::write(&paths.main_backup, b"old artifact").unwrap();
        write_test_manifest(&paths, ManifestPhase::Committed, &["main"], "r4-attempt");
        assert_eq!(
            reconcile_restore(&db_path).unwrap(),
            ReconcileState::CommittedPendingLog
        );
        assert!(!paths.main_backup.exists() && paths.manifest.exists());
        let conn = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        complete_reconciled_restore(&conn, &db_path).unwrap();
        assert_no_restore_artifacts(&paths);
    }

    #[test]
    fn test_restore_req901_b4_r1_removes_unrecorded_new_generation_sidecars() {
        // REQ-901 / Matrix B4: R1 は記録集合外を含む元名側の新世代を全除去する。
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let paths = RestorePaths::new(&db_path);
        let old = database_with_supplier(&db_path, "old");
        drop(old);
        std::fs::rename(&paths.main, &paths.main_backup).unwrap();
        std::fs::write(&paths.main, b"untrusted-new-main").unwrap();
        std::fs::write(&paths.wal, b"unrecorded-new-wal").unwrap();
        std::fs::write(&paths.shm, b"unrecorded-new-shm").unwrap();
        write_test_manifest(&paths, ManifestPhase::Active, &["main"], "r1-sidecars");

        assert_eq!(
            reconcile_restore(&db_path).unwrap(),
            ReconcileState::Recovered
        );
        assert!(!paths.wal.exists(), "R1 must remove new-generation WAL");
        assert!(!paths.shm.exists(), "R1 must remove new-generation SHM");
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        assert_eq!(supplier_names(&reopened), vec!["old"]);
        assert_no_restore_artifacts(&paths);
    }

    #[test]
    fn test_restore_req901_b9_new_restore_rejects_temp_only_residual() {
        // REQ-901 / Matrix B9: T0 は startup reconcile 専用。新規 restore は temp 遺物を保持して拒否する。
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let paths = RestorePaths::new(&db_path);
        let conn = database_with_supplier(&db_path, "old");
        drop(database_with_supplier(&backup_path, "new"));
        std::fs::write(&paths.manifest_temp, b"previous-attempt").unwrap();

        let error =
            restore_backup_with_ops(conn, &backup_path, &db_path, &StdRestoreFileOps).unwrap_err();
        assert!(matches!(error, RestoreError::Unrecoverable(_)));
        assert_eq!(
            std::fs::read(&paths.manifest_temp).unwrap(),
            b"previous-attempt",
            "新規 restore は前回 temp を勝手に解消しない"
        );
        assert_eq!(
            supplier_names(&db::open_existing_database(db_path.to_str().unwrap()).unwrap()),
            vec!["old"]
        );

        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let paths = RestorePaths::new(&db_path);
        let conn = database_with_supplier(&db_path, "old");
        drop(database_with_supplier(&backup_path, "new"));
        write_test_manifest(
            &paths,
            ManifestPhase::Committed,
            &["main"],
            "committed-with-temp",
        );
        std::fs::write(&paths.manifest_temp, b"conflicting-attempt").unwrap();
        let error =
            restore_backup_with_ops(conn, &backup_path, &db_path, &StdRestoreFileOps).unwrap_err();
        assert!(matches!(error, RestoreError::Unrecoverable(_)));
        assert!(paths.manifest.exists() && paths.manifest_temp.exists());
    }

    #[test]
    fn test_restore_req901_b4_reconcile_all_original_sidecar_sets() {
        // REQ-901 / Matrix B4: main / main+wal / main+shm / main+wal+shm.
        let source_dir = tempfile::tempdir().unwrap();
        let source_db = source_dir.path().join("source.db");
        let source = database_with_supplier(&source_db, "main-row");
        source
            .execute_batch(
                "PRAGMA wal_autocheckpoint=0;
             PRAGMA wal_checkpoint(TRUNCATE);
             INSERT INTO suppliers (name, created_at) VALUES ('wal-row', '2026-07-18T00:00:02');",
            )
            .unwrap();
        let source_wal = PathBuf::from(format!("{}-wal", source_db.display()));
        let source_shm = PathBuf::from(format!("{}-shm", source_db.display()));
        assert!(std::fs::metadata(&source_wal).unwrap().len() > 32);
        let main_bytes = std::fs::read(&source_db).unwrap();
        let wal_bytes = std::fs::read(&source_wal).unwrap();
        let shm_bytes = std::fs::read(&source_shm).unwrap();
        drop(source);

        for files in [
            vec!["main"],
            vec!["main", "wal"],
            vec!["main", "shm"],
            vec!["main", "wal", "shm"],
        ] {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("inventory.db");
            let paths = RestorePaths::new(&db_path);
            std::fs::write(&paths.main_backup, &main_bytes).unwrap();
            if files.contains(&"wal") {
                std::fs::write(&paths.wal_backup, &wal_bytes).unwrap();
            }
            if files.contains(&"shm") {
                std::fs::write(&paths.shm_backup, &shm_bytes).unwrap();
            }
            write_test_manifest(&paths, ManifestPhase::Active, &files, "sidecar-set");

            assert_eq!(
                reconcile_restore(&db_path).unwrap(),
                ReconcileState::Recovered
            );
            let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
            assert!(supplier_names(&reopened).contains(&"main-row".to_string()));
            if files.contains(&"wal") {
                assert!(supplier_names(&reopened).contains(&"wal-row".to_string()));
            }
            assert_no_restore_artifacts(&paths);
        }
    }

    #[test]
    fn test_restore_req901_b4_every_r1_r2_mutation_reinterruption_converges() {
        // REQ-901 / Matrix B4: 元集合4種 × 全退避prefix × reconcile各mutation/sync直後。
        let source_dir = tempfile::tempdir().unwrap();
        let source_db = source_dir.path().join("source.db");
        let source = database_with_supplier(&source_db, "old-main");
        source
            .execute_batch(
                "PRAGMA wal_autocheckpoint=0;
                 PRAGMA wal_checkpoint(TRUNCATE);
                 INSERT INTO suppliers (name, created_at)
                 VALUES ('old-wal', '2026-07-18T00:00:02');",
            )
            .unwrap();
        let source_paths = RestorePaths::new(&source_db);
        assert!(std::fs::metadata(&source_paths.wal).unwrap().len() > 32);
        let main_bytes = std::fs::read(&source_paths.main).unwrap();
        let wal_bytes = std::fs::read(&source_paths.wal).unwrap();
        let shm_bytes = std::fs::read(&source_paths.shm).unwrap();

        for recorded in [
            vec!["main"],
            vec!["main", "wal"],
            vec!["main", "shm"],
            vec!["main", "wal", "shm"],
        ] {
            for evacuated_count in 0..=recorded.len() {
                let mut observed_interruption = false;
                for fail_after in 1..=20 {
                    let dir = tempfile::tempdir().unwrap();
                    let db_path = dir.path().join("inventory.db");
                    let paths = RestorePaths::new(&db_path);
                    for (name, original, _) in paths.triples() {
                        if recorded.contains(&name) {
                            let bytes = match name {
                                "main" => &main_bytes,
                                "wal" => &wal_bytes,
                                "shm" => &shm_bytes,
                                _ => unreachable!(),
                            };
                            std::fs::write(original, bytes).unwrap();
                        }
                    }
                    for (_name, original, backup) in paths
                        .triples()
                        .into_iter()
                        .filter(|(name, _, _)| recorded.contains(name))
                        .take(evacuated_count)
                    {
                        std::fs::rename(original, backup).unwrap();
                    }
                    write_test_manifest(
                        &paths,
                        ManifestPhase::Active,
                        &recorded,
                        "all-reinterruptions",
                    );

                    if evacuated_count == recorded.len() {
                        // R1: attempt生成世代は記録集合外のsidecarを含め全て不可信。
                        std::fs::write(&paths.main, b"untrusted-main").unwrap();
                        std::fs::write(&paths.wal, b"untrusted-wal").unwrap();
                        std::fs::write(&paths.shm, b"untrusted-shm").unwrap();
                    }

                    let first = reconcile_restore_with_ops(
                        &db_path,
                        &ReconcileInterruptOps::new(fail_after),
                    );
                    if first.is_ok() {
                        break;
                    }
                    observed_interruption = true;
                    reconcile_restore(&db_path).unwrap();
                    let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
                    let names = supplier_names(&reopened);
                    assert!(names.contains(&"old-main".to_string()));
                    assert_eq!(
                        names.contains(&"old-wal".to_string()),
                        recorded.contains(&"wal"),
                        "recorded={recorded:?}, evacuated_count={evacuated_count}, fail_after={fail_after}"
                    );
                    drop(reopened);
                    assert_no_restore_artifacts(&paths);
                }
                assert!(
                    observed_interruption,
                    "fixture must exercise at least one failpoint: {recorded:?}/{evacuated_count}"
                );
            }
        }
        drop(source);
    }

    #[test]
    fn test_restore_req901_b6_b7_fail_closed_branches_preserve_artifacts() {
        // REQ-901 / Matrix B6, B7
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let paths = RestorePaths::new(&db_path);

        write_test_manifest(&paths, ManifestPhase::Active, &["main"], "r3");
        std::fs::write(&paths.main_backup, b"main").unwrap();
        std::fs::write(&paths.wal_backup, b"unexpected").unwrap();
        let before_manifest = std::fs::read(&paths.manifest).unwrap();
        assert!(reconcile_restore(&db_path).is_err());
        assert_eq!(std::fs::read(&paths.manifest).unwrap(), before_manifest);
        assert_eq!(std::fs::read(&paths.wal_backup).unwrap(), b"unexpected");

        let dir = tempfile::tempdir().unwrap();
        let paths = RestorePaths::new(&dir.path().join("inventory.db"));
        std::fs::write(&paths.main_backup, b"must-survive-r5").unwrap();
        assert!(reconcile_restore(&paths.main).is_err());
        assert_eq!(
            std::fs::read(&paths.main_backup).unwrap(),
            b"must-survive-r5"
        );

        std::fs::write(&paths.manifest, b"not-json").unwrap();
        assert!(reconcile_restore(&paths.main).is_err());
        assert_eq!(
            std::fs::read(&paths.main_backup).unwrap(),
            b"must-survive-r5"
        );

        std::fs::remove_file(&paths.main_backup).unwrap();
        assert_eq!(
            reconcile_restore(&paths.main).unwrap(),
            ReconcileState::Clean
        );
        assert!(
            !paths.manifest.exists(),
            "R6 removes malformed manifest without backups"
        );

        let dir = tempfile::tempdir().unwrap();
        let paths = RestorePaths::new(&dir.path().join("inventory.db"));
        write_test_manifest(&paths, ManifestPhase::Active, &["main"], "read-error-r6");
        assert_eq!(
            reconcile_restore_with_ops(
                &paths.main,
                &InjectedOps::new(InjectedFailure::ManifestRead),
            )
            .unwrap(),
            ReconcileState::Clean
        );
        assert!(
            !paths.manifest.exists(),
            "R6 removes unreadable manifest without backups"
        );
    }

    #[test]
    fn test_restore_req901_b8_log_classification_five_fixtures() {
        // REQ-901 / Matrix B8: 各 fixture は独立させ、集約結果と件数を厳密検査する。
        fn assert_fixture(
            rows: &[Option<&str>],
            expected: LogClassification,
            expected_count: i64,
            marker_remains: bool,
        ) {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("inventory.db");
            let paths = RestorePaths::new(&db_path);
            let conn = database_with_supplier(&db_path, "data");
            write_test_manifest(&paths, ManifestPhase::Committed, &["main"], "target");
            for detail in rows {
                db::system_repo::insert_operation_log(
                    &conn,
                    &NewOperationLog {
                        operation_type: RESTORE_OPERATION.to_string(),
                        summary: "fixture".to_string(),
                        detail_json: detail.map(str::to_string),
                    },
                )
                .unwrap();
            }
            assert_eq!(classify_restore_log(&conn, "target").unwrap(), expected);
            let completed = complete_reconciled_restore(&conn, &db_path);
            assert_eq!(completed.is_err(), marker_remains);
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'backup_restore'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, expected_count);
            assert_eq!(paths.manifest.exists(), marker_remains);
            if expected == LogClassification::NoMatch {
                let summary: String = conn
                    .query_row(
                        "SELECT summary FROM operation_logs
                         WHERE operation_type = 'backup_restore'
                           AND json_extract(detail_json, '$.attempt_id') = 'target'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert!(summary.contains("起動時に確定"));
            }
        }

        assert_fixture(&[None], LogClassification::NoMatch, 2, false);
        assert_fixture(
            &[Some(r#"{"other":"missing"}"#)],
            LogClassification::NoMatch,
            2,
            false,
        );
        assert_fixture(
            &[Some(r#"{"attempt_id":"different"}"#)],
            LogClassification::NoMatch,
            2,
            false,
        );
        assert_fixture(&[Some("not-json")], LogClassification::Failed, 1, true);
        assert_fixture(
            &[Some("not-json"), Some(r#"{"attempt_id":"target"}"#)],
            LogClassification::AlreadyPresent,
            2,
            false,
        );

        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let conn = database_with_supplier(&db_path, "data");
        conn.execute(
            "INSERT INTO operation_logs
             (operation_type, summary, detail_json, created_at)
             VALUES ('backup_restore', 'invalid typed row', X'00', '2026-07-18T00:00:00')",
            [],
        )
        .unwrap();
        db::system_repo::insert_operation_log(
            &conn,
            &NewOperationLog {
                operation_type: RESTORE_OPERATION.to_string(),
                summary: "exact after decode error".to_string(),
                detail_json: Some(r#"{"attempt_id":"target"}"#.to_string()),
            },
        )
        .unwrap();
        assert_eq!(
            classify_restore_log(&conn, "target").unwrap(),
            LogClassification::AlreadyPresent,
            "REQ-901 / Matrix B8: exact match wins over a prior row decode error"
        );
    }

    #[test]
    fn test_restore_req901_b9_committed_cleanup_converges_to_exactly_one_log() {
        // REQ-901 / Matrix B9: exact log + committed manifest is idempotent.
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let paths = RestorePaths::new(&db_path);
        let conn = database_with_supplier(&db_path, "new");
        write_test_manifest(&paths, ManifestPhase::Committed, &["main"], "attempt-once");
        db::system_repo::insert_operation_log(
            &conn,
            &NewOperationLog {
                operation_type: RESTORE_OPERATION.to_string(),
                summary: "already committed".to_string(),
                detail_json: Some(r#"{"attempt_id":"attempt-once"}"#.to_string()),
            },
        )
        .unwrap();
        complete_reconciled_restore(&conn, &db_path).unwrap();
        complete_reconciled_restore(&conn, &db_path).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'backup_restore'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        assert_no_restore_artifacts(&paths);
    }

    #[test]
    fn test_restore_req901_b9_committed_escape_hatch_releases_all_log_failures() {
        // REQ-901 / Matrix B9: committed の最終補完は lookup/INSERT Failed でも新規 restore を塞がない。
        for lookup_failure in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("inventory.db");
            let backup_path = dir.path().join("replacement.db");
            let paths = RestorePaths::new(&db_path);
            let conn = database_with_supplier(&db_path, "current-committed");
            drop(database_with_supplier(&backup_path, "replacement"));
            write_test_manifest(&paths, ManifestPhase::Committed, &["main"], "escape-hatch");
            std::fs::write(&paths.main_backup, b"obsolete-old-generation").unwrap();
            if lookup_failure {
                conn.execute_batch("DROP TABLE operation_logs;").unwrap();
            } else {
                conn.execute_batch(
                    "CREATE TRIGGER reject_restore_log
                     BEFORE INSERT ON operation_logs
                     WHEN NEW.operation_type = 'backup_restore'
                     BEGIN SELECT RAISE(ABORT, 'injected insert failure'); END;",
                )
                .unwrap();
            }

            let restored =
                restore_backup_with_ops(conn, &backup_path, &db_path, &StdRestoreFileOps).unwrap();
            assert_eq!(supplier_names(&restored), vec!["replacement"]);
            assert_no_restore_artifacts(&paths);
        }
    }

    #[test]
    fn test_restore_req901_b9_each_committed_cleanup_interruption_converges() {
        // REQ-901 / Matrix B9: backup unlink/sync, manifest unlink/sync の各直後を注入。
        for fail_at in 1..=5 {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("inventory.db");
            let backup_path = dir.path().join("replacement.db");
            let conn = database_with_supplier(&db_path, "old");
            drop(database_with_supplier(&backup_path, "new"));
            let restore_result = restore_backup_with_ops(
                conn,
                &backup_path,
                &db_path,
                &CommittedInterruptOps::new(fail_at),
            );
            match restore_result {
                Ok(restored) => drop(restored),
                Err(RestoreError::DurabilityUnknown(_)) if fail_at == 1 => {}
                Err(other) => panic!("unexpected fail_at={fail_at}: {other:?}"),
            }

            let paths = RestorePaths::new(&db_path);
            let state = reconcile_restore(&db_path).unwrap();
            let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
            if state == ReconcileState::CommittedPendingLog {
                complete_reconciled_restore(&reopened, &db_path).unwrap();
            }
            assert_eq!(supplier_names(&reopened), vec!["new"]);
            let count: i64 = reopened
                .query_row(
                    "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'backup_restore'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "fail_at={fail_at}");
            assert_no_restore_artifacts(&paths);
        }
    }

    #[test]
    fn test_restore_req901_durability_unknown_only_after_committed_rename() {
        // REQ-901 / Matrix F2 / D5(e)(ii)
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let backup_path = dir.path().join("replacement.db");
        let conn = database_with_supplier(&db_path, "old");
        drop(database_with_supplier(&backup_path, "new"));
        let error = restore_backup_with_ops(
            conn,
            &backup_path,
            &db_path,
            &InjectedOps::new(InjectedFailure::CommittedDirectorySync),
        )
        .unwrap_err();
        assert!(matches!(error, RestoreError::DurabilityUnknown(_)));
        let paths = RestorePaths::new(&db_path);
        let manifest: RestoreManifest =
            serde_json::from_slice(&std::fs::read(&paths.manifest).unwrap()).unwrap();
        assert_eq!(manifest.phase, ManifestPhase::Committed);
        assert_eq!(
            reconcile_restore(&db_path).unwrap(),
            ReconcileState::CommittedPendingLog
        );
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        complete_reconciled_restore(&reopened, &db_path).unwrap();
        assert_eq!(supplier_names(&reopened), vec!["new"]);
        assert_no_restore_artifacts(&paths);
    }

    #[test]
    fn test_restore_req901_b9_phase_update_pre_rename_failures_reconcile_old_snapshot() {
        // REQ-901 / Matrix B9 / D5(e)(i): committed rename 前は active を残し再起動必須。
        for stage in [
            PhaseUpdateFailure::TempWrite,
            PhaseUpdateFailure::TempSync,
            PhaseUpdateFailure::CanonicalReplace,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("inventory.db");
            let backup_path = dir.path().join("replacement.db");
            let conn = database_with_supplier(&db_path, "old");
            drop(database_with_supplier(&backup_path, "new"));

            let error = restore_backup_with_ops(
                conn,
                &backup_path,
                &db_path,
                &PhaseUpdateFailureOps::new(stage),
            )
            .unwrap_err();
            assert!(matches!(error, RestoreError::Unrecoverable(_)));
            let paths = RestorePaths::new(&db_path);
            assert!(paths.manifest.exists() && paths.main_backup.exists());
            assert_eq!(
                reconcile_restore(&db_path).unwrap(),
                ReconcileState::Recovered
            );
            let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
            assert_eq!(supplier_names(&reopened), vec!["old"]);
            assert_no_restore_artifacts(&paths);
        }
    }

    #[test]
    fn test_restore_req901_manifest_write_sync_rename_directory_sync_order() {
        // REQ-901 / MNT-01-D5 / Matrix B9, X1(d)
        let dir = tempfile::tempdir().unwrap();
        let paths = RestorePaths::new(&dir.path().join("inventory.db"));
        let manifest = RestoreManifest {
            attempt_id: "ordering".to_string(),
            original_files: ["main".to_string()].into_iter().collect(),
            phase: ManifestPhase::Active,
        };
        let ops = RecordingOps::new();
        write_manifest(&ops, &paths, &manifest, false).unwrap();
        assert_eq!(
            *ops.events.lock().unwrap(),
            vec!["write", "sync_file", "rename", "sync_parent"]
        );
    }

    /// `open` の直後の位置から対応する `close` までの中身を返す（入れ子を数える）
    fn balanced(text: &str, open: char, close: char) -> &str {
        let mut depth = 0usize;
        for (index, ch) in text.char_indices() {
            if ch == open {
                depth += 1;
            } else if ch == close {
                depth -= 1;
                if depth == 0 {
                    return &text[1..index];
                }
            }
        }
        panic!("対応する {close} が無い");
    }

    /// `StdRestoreFileOps` の `publish_no_replace` の Windows の分岐の `MoveFileExW` の flags
    fn windows_publish_flags(source: &str) -> Result<String, String> {
        let production = source
            .split(concat!("#[cfg(", "test)]\nmod tests {"))
            .next()
            .unwrap();
        let std_impl = production
            .split("impl RestoreFileOps for StdRestoreFileOps")
            .nth(1)
            .ok_or("StdRestoreFileOps の impl が無い")?;
        let after_fn = std_impl
            .split("fn publish_no_replace(")
            .nth(1)
            .ok_or("publish_no_replace の実装が無い")?;
        let body = balanced(&after_fn[after_fn.find('{').unwrap()..], '{', '}');
        if body.contains("MOVEFILE_REPLACE_EXISTING") {
            return Err("MOVEFILE_REPLACE_EXISTING がある".into());
        }
        let windows = body
            .split("#[cfg(windows)]")
            .nth(1)
            .ok_or("Windows の分岐が無い")?;
        if windows.matches("MoveFileExW(").count() != 1 {
            return Err("MoveFileExW の呼出しが 1 つでない".into());
        }
        let call = windows.split("MoveFileExW(").nth(1).unwrap();
        let args = balanced(&format!("({call}"), '(', ')').to_string();
        let mut depth = 0i32;
        let mut parts = vec![String::new()];
        for ch in args.chars() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(String::new());
                    continue;
                }
                _ => {}
            }
            parts.last_mut().unwrap().push(ch);
        }
        let parts: Vec<String> = parts
            .into_iter()
            .map(|part| part.trim().to_string())
            .filter(|part| !part.is_empty())
            .collect();
        if parts.len() != 3 {
            return Err(format!("MoveFileExW の引数が 3 つでない: {parts:?}"));
        }
        Ok(parts[2].clone())
    }

    #[test]
    fn test_publish_no_replace_req901_d7_windows_flags_exclude_replace_existing() {
        // REQ-901 / MNT-01-D7 / 71 §71.4.1: Linux の CI は #[cfg(windows)] を compile しないので、
        // Windows の公開が既存の正式名を置き換えないことを source で見る（restore の replace は見ない）
        let source = include_str!("restore.rs");
        let flags = windows_publish_flags(source).unwrap();
        assert!(flags.contains("MOVEFILE_WRITE_THROUGH"), "flags={flags}");
        assert!(
            !flags.contains("MOVEFILE_REPLACE_EXISTING"),
            "flags={flags}"
        );

        // mutant: publish_no_replace の本体の中だけを書き換える（restore の replace は触らない）
        let impl_at = source
            .find("impl RestoreFileOps for StdRestoreFileOps")
            .unwrap();
        let fn_at = impl_at + source[impl_at..].find("fn publish_no_replace(").unwrap();
        let fn_at = fn_at + source[fn_at..].find("#[cfg(windows)]").unwrap();
        let fn_end = fn_at + source[fn_at..].find("fn copy(").unwrap();
        for (from, to) in [
            (
                "MOVEFILE_WRITE_THROUGH,",
                "MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,",
            ),
            ("MOVEFILE_WRITE_THROUGH,", "0,"),
            ("MoveFileExW(", "std_fs_rename("),
        ] {
            let region = &source[fn_at..fn_end];
            assert_eq!(
                region.matches(from).count(),
                1,
                "前提の文字列が無い: {from}"
            );
            let mutated = format!(
                "{}{}{}",
                &source[..fn_at],
                region.replacen(from, to, 1),
                &source[fn_end..]
            );
            assert!(
                !windows_publish_flags(&mutated).is_ok_and(|flags| {
                    flags.contains("MOVEFILE_WRITE_THROUGH")
                        && !flags.contains("MOVEFILE_REPLACE_EXISTING")
                }),
                "red にならない: {to}"
            );
        }
    }

    #[test]
    fn test_restore_req901_d15_live_db_path_keeps_current_db() {
        // REQ-901 / UI-11b-D15 隣接（runtime (b)）: 「控えを選んで確かめる」でアプリの今の DB を選んでも DB を失わない
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("inventory.db");
        let conn = database_with_supplier(&db_path, "今の行");
        let names_of = |dir: &Path| {
            let mut names: Vec<String> = std::fs::read_dir(dir)
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .filter(|name| !name.ends_with("-wal") && !name.ends_with("-shm"))
                .collect();
            names.sort();
            names
        };
        let before = names_of(dir.path());

        let result = restore_backup(conn, &db_path, &db_path);

        assert!(
            matches!(result, Err(RestoreError::Recovered(_))),
            "Recovered 以外: {:?}",
            result.as_ref().err()
        );
        let reopened = db::open_existing_database(db_path.to_str().unwrap()).unwrap();
        assert_eq!(supplier_names(&reopened), ["今の行"]);
        assert_eq!(
            names_of(dir.path()),
            before,
            "退避・manifest の遺物を残さない"
        );
    }
}
