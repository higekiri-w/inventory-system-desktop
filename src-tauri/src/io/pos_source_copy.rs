//! IO-10: SD から読んだ原本の写しをアプリのデータ folder に保存する（D-111）
//!
//! docs/function-design/29-io-daily-report-parser.md §29.8 に基づく実装。
//! SD には書かない。書く先は `{app_data_dir}/pos-sources/casio-sr-s4000/sd/` の下だけ。

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// 写しの置き場（app_data_dir からの相対、要素ごと）
const COPY_ROOT: [&str; 3] = ["pos-sources", "casio-sr-s4000", "sd"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosSourceCopyResult {
    /// app_data_dir からの相対 path（区切りは `/`）
    pub relative_path: String,
    /// false = 同じ path に同じ bytes がすでにあり、書かなかった
    pub written: bool,
}

/// 29 §29.8.2
pub fn save_pos_source_copy(
    app_data_dir: &Path,
    sd_relative_path: &str,
    bytes: &[u8],
) -> Result<PosSourceCopyResult, io::Error> {
    save_with_writer(app_data_dir, sd_relative_path, bytes, |file, bytes| {
        file.write_all(bytes)
    })
}

fn save_with_writer(
    app_data_dir: &Path,
    sd_relative_path: &str,
    bytes: &[u8],
    write: impl FnOnce(&mut File, &[u8]) -> io::Result<()>,
) -> Result<PosSourceCopyResult, io::Error> {
    // 手順 1: IO-09-D3 と同じ規則で相対 path を検証する（`/` も区切りとして扱い、要素に混ぜない）
    let parts: Vec<&str> = sd_relative_path.split(['\\', '/']).collect();
    if parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == ".." || part.contains(':'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "写しの相対 path が不正です",
        ));
    }
    let relative: Vec<&str> = COPY_ROOT.iter().copied().chain(parts).collect();

    // 手順 2: 書く先を決める。既存の file は上書きしない
    let hash = sha256_hex(bytes);
    let default_relative = relative.join("/");
    let target = match existing_hash(&app_data_dir.join(relative.join("/")))? {
        None => default_relative,
        Some(existing) if existing == hash => {
            return Ok(PosSourceCopyResult {
                relative_path: default_relative,
                written: false,
            })
        }
        Some(_) => {
            let mut suffixed = relative.clone();
            let name = suffixed.pop().unwrap_or_default();
            let renamed = match name.rfind('.') {
                Some(dot) => format!("{}~{}{}", &name[..dot], &hash[..12], &name[dot..]),
                None => format!("{}~{}", name, &hash[..12]),
            };
            suffixed.push(&renamed);
            let suffixed_relative = suffixed.join("/");
            match existing_hash(&app_data_dir.join(&suffixed_relative))? {
                None => suffixed_relative,
                Some(existing) if existing == hash => {
                    return Ok(PosSourceCopyResult {
                        relative_path: suffixed_relative,
                        written: false,
                    })
                }
                Some(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "写しの置き場に別の内容の file があります",
                    ))
                }
            }
        }
    };

    // 手順 3・4: directory を作り、一時 file に書いて sync してから最終の名前へ rename する
    let final_path = app_data_dir.join(&target);
    let dir = final_path
        .parent()
        .ok_or_else(|| io::Error::other("写しの置き場の directory を決められません"))?;
    fs::create_dir_all(dir)?;
    let temp_path = dir.join(format!(".copy-{}.tmp", uuid::Uuid::new_v4().simple()));
    let written = (|| {
        let mut file = File::create_new(&temp_path)?;
        write(&mut file, bytes)?;
        file.flush()?;
        file.sync_all()
    })()
    .and_then(|()| fs::rename(&temp_path, &final_path));
    if let Err(error) = written {
        if let Err(remove_error) = fs::remove_file(&temp_path) {
            if remove_error.kind() != io::ErrorKind::NotFound {
                tracing::warn!(error = %remove_error, "写しの一時 file を消せませんでした");
            }
        }
        return Err(error);
    }
    Ok(PosSourceCopyResult {
        relative_path: target,
        written: true,
    })
}

/// 在れば SHA-256、無ければ None。読めない（権限等）は error のまま返す
fn existing_hash(path: &Path) -> io::Result<Option<String>> {
    match File::open(path) {
        Ok(mut file) => {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Ok(Some(sha256_hex(&bytes)))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REL: &str = "XZ\\2026\\10\\Z001_06 .CSV";
    const FINAL: &str = "pos-sources/casio-sr-s4000/sd/XZ/2026/10/Z001_06 .CSV";

    #[test]
    fn test_req401_creates_missing_directory_before_temp_file() {
        // REQ-401 / IO-10 §29.8.3 手順 3: 初回は保存先の directory を作ってから書く
        let dir = tempfile::tempdir().unwrap();
        let result = save_pos_source_copy(dir.path(), REL, b"abc").unwrap();
        assert_eq!(
            result,
            PosSourceCopyResult {
                relative_path: FINAL.to_string(),
                written: true
            }
        );
        assert_eq!(fs::read(dir.path().join(FINAL)).unwrap(), b"abc");
        let leftovers: Vec<_> = fs::read_dir(dir.path().join(FINAL).parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(leftovers.len(), 1, "一時 file が残っている: {leftovers:?}");
    }

    #[test]
    fn test_req401_same_bytes_not_rewritten() {
        // REQ-401 / IO-10 §29.8.3 手順 2: 同じ bytes は書かない（更新時刻も変えない）
        let dir = tempfile::tempdir().unwrap();
        save_pos_source_copy(dir.path(), REL, b"abc").unwrap();
        let before = fs::metadata(dir.path().join(FINAL))
            .unwrap()
            .modified()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let second = save_pos_source_copy(dir.path(), REL, b"abc").unwrap();
        assert_eq!(second.relative_path, FINAL);
        assert!(!second.written);
        let after = fs::metadata(dir.path().join(FINAL))
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_req401_different_bytes_get_hash_suffixed_name() {
        // REQ-401 / IO-10 §29.8.3 手順 2: 違う bytes は上書きせず `~` + SHA-256 の先頭 12 桁の名前
        let dir = tempfile::tempdir().unwrap();
        save_pos_source_copy(dir.path(), REL, b"first").unwrap();
        let second = save_pos_source_copy(dir.path(), REL, b"second").unwrap();
        let hash12 = &sha256_hex(b"second")[..12];
        let expected = format!("pos-sources/casio-sr-s4000/sd/XZ/2026/10/Z001_06 ~{hash12}.CSV");
        assert_eq!(second.relative_path, expected);
        assert!(second.written);
        assert_eq!(fs::read(dir.path().join(FINAL)).unwrap(), b"first");
        assert_eq!(fs::read(dir.path().join(&expected)).unwrap(), b"second");
        // 同じ bytes の 3 回目は書かない
        let third = save_pos_source_copy(dir.path(), REL, b"second").unwrap();
        assert_eq!(third.relative_path, expected);
        assert!(!third.written);
    }

    #[test]
    fn test_req401_writes_via_temp_and_rename() {
        // REQ-401 / IO-10 §29.8.3 手順 4: 書込みの途中で失敗しても最終の名前に file を残さない
        let dir = tempfile::tempdir().unwrap();
        let error = save_with_writer(dir.path(), REL, b"abc", |file, bytes| {
            file.write_all(&bytes[..1])?;
            Err(io::Error::other("injected"))
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "injected");
        let final_path = dir.path().join(FINAL);
        assert!(!final_path.exists());
        let leftovers = fs::read_dir(final_path.parent().unwrap()).unwrap().count();
        assert_eq!(leftovers, 0, "一時 file が残っている");
    }

    #[test]
    fn test_req401_rejects_escaping_paths() {
        // REQ-401 / IO-10 §29.8.3 手順 1: app data の外を指す相対 path は書かない
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("app");
        for bad in [
            "..\\x",
            "C:\\x",
            "\\x",
            "XZ\\\\x",
            "XZ\\..\\..\\..\\..\\x",
            "../x",
            "XZ/./x",
        ] {
            let error = save_pos_source_copy(&base, bad, b"abc").unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput, "{bad}");
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
