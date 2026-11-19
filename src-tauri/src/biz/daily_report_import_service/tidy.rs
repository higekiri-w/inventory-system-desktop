//! BIZ-08-D11: 取り込んだ精算の Z を SD の `XZ_BKUP` へ移す（37 §37.12、D-149）

use crate::biz::daily_report_import_service::RegisterSdTidyFailure;
use crate::constants;
use crate::db::sales_repo;
use crate::db::DbConnection;
use crate::io::pos_source_copy;
use crate::io::register_sd::{self, RegisterSdArea, RegisterSdEntryKind, RegisterSdRoot};
use crate::io::register_sd_move::{
    self, RegisterSdGroupMove, RegisterSdMoveError, RegisterSdMoveItem,
};
use chrono::NaiveDate;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// 組の 1 本（読んだ bytes を写しと移動の確かめに使う）
struct GroupFile {
    relative_path: String,
    bytes: Vec<u8>,
}

/// 37 §37.12 `tidy_register_sd_imported`。DB へは書かない。失敗しても `Err` を返さない
pub(super) fn tidy_register_sd_imported(
    conn: &DbConnection,
    root: &RegisterSdRoot,
    from: NaiveDate,
    app_data_dir: &Path,
) -> Option<RegisterSdTidyFailure> {
    tidy_with_mover(
        conn,
        root,
        from,
        app_data_dir,
        register_sd_move::move_register_sd_group_to_backup,
    )
}

/// `tidy_register_sd_imported` の本体。IO-11 の呼出しを引数に取るのは、組の途中の失敗（移した後の確かめの
/// 食い違い等。組の file は同じ folder にあり、file system の権限では 1 本だけを止められない）を test が
/// 注入するため（`scan_with_roots` の `find_roots` と同じ形）
pub(super) fn tidy_with_mover(
    conn: &DbConnection,
    root: &RegisterSdRoot,
    from: NaiveDate,
    app_data_dir: &Path,
    move_group: impl Fn(&RegisterSdRoot, &[RegisterSdMoveItem]) -> RegisterSdGroupMove,
) -> Option<RegisterSdTidyFailure> {
    // 手順 1: 列挙の失敗は続く §37.9 手順 3 が利用者に出すので、ここでは重ねない
    let listing = match register_sd::list_register_sd_entries(root, from) {
        Ok(listing) => listing,
        Err(error) => {
            tracing::warn!(error = ?error, "SD の片付けの列挙ができませんでした");
            return None;
        }
    };

    // 手順 2: (相対 folder, 日, 接尾字) で組に分け、SD に書く前に移す組を全部決める
    let mut groups: BTreeMap<(String, u8, char), BTreeMap<u16, String>> = BTreeMap::new();
    for entry in listing.entries {
        let RegisterSdEntryKind::Z {
            series: series @ (1 | 2 | 4 | 5),
            day,
            suffix,
            backup_seq: None,
        } = entry.kind
        else {
            continue;
        };
        if entry.area != RegisterSdArea::Sales {
            continue;
        }
        let Some((folder, _)) = entry.relative_path.rsplit_once('\\') else {
            continue;
        };
        groups
            .entry((folder.to_string(), day, suffix))
            .or_default()
            .insert(series, entry.relative_path);
    }
    let mut planned: Vec<Vec<GroupFile>> = Vec::new();
    for files in groups.into_values() {
        match plan_group(conn, root, &files) {
            Ok(Some(group)) => planned.push(group),
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(error = %error, "SD の片付けの対象を DB で確かめられませんでした");
                return Some(RegisterSdTidyFailure::Other);
            }
        }
    }

    // 手順 3: 組ごとに、写してから移す。失敗したら以後の組を移さない
    let total: usize = planned.iter().map(Vec::len).sum();
    let mut moved_count = 0usize;
    for (index, group) in planned.iter().enumerate() {
        // 3a
        for file in group {
            if let Err(error) = pos_source_copy::save_pos_source_copy(
                app_data_dir,
                &file.relative_path,
                &file.bytes,
            ) {
                tracing::warn!(
                    kind = ?error.kind(),
                    error = %error,
                    relative_path = %file.relative_path,
                    not_moved = ?not_moved_paths(&planned[index..], 0),
                    moved = moved_count,
                    left = total - moved_count,
                    "SD の片付けの前に写しを保存できなかったので、片付けを止めました"
                );
                return Some(RegisterSdTidyFailure::PcCopy);
            }
        }
        // 3b
        let items: Vec<RegisterSdMoveItem> = group
            .iter()
            .map(|file| RegisterSdMoveItem {
                relative_path: file.relative_path.clone(),
                expected_len: file.bytes.len() as u64,
            })
            .collect();
        let RegisterSdGroupMove { moved, error } = move_group(root, &items);
        for (item, destination) in items.iter().zip(&moved) {
            tracing::info!(from = %item.relative_path, to = %destination, "SD の Z を XZ_BKUP へ移しました");
        }
        moved_count += moved.len();
        if let Some(error) = error {
            tracing::warn!(
                error = ?error,
                not_moved = ?not_moved_paths(&planned[index..], moved.len()),
                moved = moved_count,
                left = total - moved_count,
                "SD の片付けを途中で止めました"
            );
            return Some(match error {
                RegisterSdMoveError::Io { .. } | RegisterSdMoveError::Verify { .. } => {
                    RegisterSdTidyFailure::SdWrite
                }
                RegisterSdMoveError::InvalidPath
                | RegisterSdMoveError::NameRule
                | RegisterSdMoveError::NoFreeSequence => RegisterSdTidyFailure::Other,
            });
        }
    }
    // 手順 4
    None
}

/// 手順 4: 止めた時点でまだ移っていない片付けの対象の全部の、元の相対 path（中身は出さない）。
/// `rest` は止めた組から後ろの組で、止めた組は先頭の `moved_in_first` 本が移っている。
/// file は元の場所にあり、移す先の名前は決まっていないので出さない（packet の Recovery）
fn not_moved_paths(rest: &[Vec<GroupFile>], moved_in_first: usize) -> Vec<&str> {
    rest.iter()
        .flatten()
        .skip(moved_in_first)
        .map(|file| file.relative_path.as_str())
        .collect()
}

/// 手順 2 の 1 組: Z001・Z002・Z005 の全部が読めて completed の取込みの file と同じ hash なら、
/// Z004 を先にした移す順の組を返す（Z004 は読めればその組に付ける。J1 の案 B）
fn plan_group(
    conn: &DbConnection,
    root: &RegisterSdRoot,
    files: &BTreeMap<u16, String>,
) -> Result<Option<Vec<GroupFile>>, crate::db::DbError> {
    let mut daily = Vec::new();
    for series in [1, 2, 5] {
        let Some(relative_path) = files.get(&series) else {
            continue;
        };
        let Some(bytes) = read(root, relative_path) else {
            return Ok(None);
        };
        let hash = format!("{:x}", Sha256::digest(&bytes));
        if !sales_repo::exists_completed_daily_report_source_file_hash(conn, &hash)? {
            return Ok(None);
        }
        daily.push(GroupFile {
            relative_path: relative_path.clone(),
            bytes,
        });
    }
    // Z004 だけの組は移さない
    if daily.is_empty() {
        return Ok(None);
    }
    // 組の中は Z004 を先に（Z004 だけが `XZ` に残ると二度と移らない。BIZ-08-D11）
    let z004 = files.get(&4).and_then(|relative_path| {
        read(root, relative_path).map(|bytes| GroupFile {
            relative_path: relative_path.clone(),
            bytes,
        })
    });
    Ok(Some(z004.into_iter().chain(daily).collect()))
}

/// 組の file を読む。読めなければ診断 WARN で None（その組は移さない。Z004 なら組から外す）
fn read(root: &RegisterSdRoot, relative_path: &str) -> Option<Vec<u8>> {
    match register_sd::read_register_sd_file(
        root,
        relative_path,
        constants::CSV_IMPORT_FILE_SIZE_LIMIT as u64,
    ) {
        Ok(file) => Some(file.bytes),
        Err(error) => {
            tracing::warn!(error = ?error, relative_path, "SD の片付けで file を読めませんでした");
            None
        }
    }
}
