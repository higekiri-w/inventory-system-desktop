// IO層: ファイルI/O・フォーマット変換（DB非依存の純関数群）

use std::path::Path;

pub mod daily_report_parser;
pub mod ej_parser;
pub mod image_manager;
#[cfg(test)]
mod l3_fixtures;
pub mod plu_formatter;
pub mod pos_source_copy;
pub mod product_csv_importer;
pub mod register_sd;
pub mod register_sd_move;
pub mod report_csv_exporter;
pub mod z004_parser;

/// レジの SD の機種の folder（drive 直下からの 2 つの要素）。IO-09 と 71 MNT-01-D10 が共有する
/// 唯一の定義（29 §29.7.4 手順 1、D-114）。要素ごとに `join` し、区切りを含む 1 つの文字列にしない
pub(crate) const REGISTER_SD_FOLDER: [&str; 2] = ["CASIO", "SR500_550_4000"];

/// 既存の file を置き換えずに `source` を `destination` へ移す（71 §71.4.1 の上書きしない公開。
/// MNT-01-D7・D8 の公開と IO-11 の SD の移動が共有する 1 か所。29 §29.9.1）。
/// 移す先が既にあれば失敗し（`AlreadyExists`）、既存の file を変えない
pub(crate) fn move_no_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
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
        // 置き換えの flag を付けない: 既存の同じ名前があれば失敗する（71 §71.4.1）
        // SAFETY: どちらも NUL で終わる UTF-16 の列で、呼出しの間は生きている
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_req401_move_no_replace_does_not_replace_existing() {
        // REQ-401 / 71 §71.4.1 / IO-11-D1: 移す先が既にあれば置き換えず AlreadyExists。元も先も変えない
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.CSV");
        let destination = dir.path().join("destination.CSV");
        std::fs::write(&source, b"new").unwrap();
        std::fs::write(&destination, b"old").unwrap();
        let error = move_no_replace(&source, &destination).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&destination).unwrap(), b"old");
        assert_eq!(std::fs::read(&source).unwrap(), b"new");

        // 空いていれば移り、元の名前は無くなる
        let free = dir.path().join("free.CSV");
        move_no_replace(&source, &free).unwrap();
        assert_eq!(std::fs::read(&free).unwrap(), b"new");
        assert!(!source.try_exists().unwrap());
    }
}
