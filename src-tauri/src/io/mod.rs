// IO層: ファイルI/O・フォーマット変換（DB非依存の純関数群）

use std::path::Path;
use std::sync::LazyLock;

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

/// 整数の形の規則（23 §13.4.1、D-103 の (1)、D-158）。Z004（IO-02）と日報（IO-07）が共有する唯一の検査。
/// カンマなしの整数か正しい3桁区切りの整数だけに一致し、カンマを除いた文字列（符号を含む）を返す。
/// trim しない: 前後の空白は呼ぶ側が値の全体に 1 回だけ除く（日報の個数の列は整数部だけを渡すので、
/// 小数点の前の空白を受けない）
pub(crate) fn strip_thousands_grouping(value: &str) -> Option<String> {
    static INT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^-?(?:[0-9]+|[1-9][0-9]{0,2}(?:,[0-9]{3})+)$")
            .expect("数値パターンのコンパイル失敗")
    });
    INT_RE.is_match(value).then(|| value.replace(',', ""))
}

#[cfg(test)]
mod tests {
    use super::daily_report_parser::{parse_optional_hundredths, parse_optional_i64, NumberError};
    use super::z004_parser::parse_z004_int;
    use super::*;

    /// 1 つの読取りの結果。個数の列は 100 倍の値
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Read {
        Value(i64),
        Empty,
        Invalid,
        TooLarge,
    }
    use Read::{Empty, Invalid, TooLarge, Value};

    fn daily(result: Result<Option<i64>, NumberError>) -> Read {
        match result {
            Ok(Some(value)) => Value(value),
            Ok(None) => Empty,
            Err(NumberError::Invalid) => Invalid,
            Err(NumberError::TooLarge) => TooLarge,
        }
    }

    /// 個数の 100 倍を 1 単位にそろえる。端数のある値はどの整数とも等しくしない
    fn units(read: Read) -> Option<Read> {
        match read {
            Value(hundredths) if hundredths % 100 != 0 => None,
            Value(hundredths) => Some(Value(hundredths / 100)),
            other => Some(other),
        }
    }

    /// 承認済みの差（ID、理由、正本）
    type Approved = (&'static str, &'static str, &'static str);
    const A1: Approved = (
        "A1",
        "日報の空の列は None（Z005 の金額だけ必須）",
        "29 §29.4",
    );
    const A2: Approved = (
        "A2",
        "上限は日報が絶対値 9999999999（個数 9999999.99）、Z004 は i32",
        "29 IO-07-D6",
    );
    const A3: Approved = ("A3", "個数の列は小数 2 桁までを受ける", "29 IO-07-D2");

    #[test]
    fn test_number_parse_req401_daily_matches_z004_except_approved_differences() {
        // REQ-401 / D-158 / D-103 (1): Z004 と日報の数の読取りに同じ入力の表を通す差分の test。
        // 行ごとに (Z004, 日報の整数, 日報の個数) の正確な期待を持ち、差は承認済みの差 A1〜A3 だけ
        let table: &[(&str, Read, Read, Read, Option<Approved>)] = &[
            ("1234", Value(1234), Value(1234), Value(123400), None),
            (" 1234 ", Value(1234), Value(1234), Value(123400), None),
            ("1,234", Value(1234), Value(1234), Value(123400), None),
            (
                "-12,345",
                Value(-12345),
                Value(-12345),
                Value(-1234500),
                None,
            ),
            (
                "1,000,000",
                Value(1000000),
                Value(1000000),
                Value(100000000),
                None,
            ),
            ("0123", Value(123), Value(123), Value(12300), None),
            ("-0", Value(0), Value(0), Value(0), None),
            ("0,123", Invalid, Invalid, Invalid, None),
            ("1,2,3", Invalid, Invalid, Invalid, None),
            ("1,23", Invalid, Invalid, Invalid, None),
            ("1,,234", Invalid, Invalid, Invalid, None),
            ("1234,567", Invalid, Invalid, Invalid, None),
            ("12,3456", Invalid, Invalid, Invalid, None),
            (",123", Invalid, Invalid, Invalid, None),
            ("123,", Invalid, Invalid, Invalid, None),
            ("1,234,", Invalid, Invalid, Invalid, None),
            ("¥1,234", Invalid, Invalid, Invalid, None),
            ("￥1234", Invalid, Invalid, Invalid, None),
            ("1 234", Invalid, Invalid, Invalid, None),
            ("1　234", Invalid, Invalid, Invalid, None),
            ("+5", Invalid, Invalid, Invalid, None),
            ("１２３", Invalid, Invalid, Invalid, None),
            ("1.5", Invalid, Invalid, Value(150), Some(A3)),
            ("1,234.5", Invalid, Invalid, Value(123450), Some(A3)),
            ("1 .5", Invalid, Invalid, Invalid, None),
            ("1,234 .5", Invalid, Invalid, Invalid, None),
            ("1\t.5", Invalid, Invalid, Invalid, None),
            ("1　.5", Invalid, Invalid, Invalid, None),
            ("1.234", Invalid, Invalid, Invalid, None),
            (".5", Invalid, Invalid, Invalid, None),
            ("1.", Invalid, Invalid, Invalid, None),
            ("9999999.99", Invalid, Invalid, Value(999999999), Some(A3)),
            ("", Invalid, Empty, Empty, Some(A1)),
            ("   ", Invalid, Empty, Empty, Some(A1)),
            ("-", Invalid, Invalid, Invalid, None),
            (
                "10000000",
                Value(10000000),
                Value(10000000),
                TooLarge,
                Some(A2),
            ),
            ("2147483648", Invalid, Value(2147483648), TooLarge, Some(A2)),
            ("10000000000", Invalid, TooLarge, TooLarge, Some(A2)),
            (
                "99999999999999999999",
                Invalid,
                TooLarge,
                TooLarge,
                Some(A2),
            ),
            ("9a", Invalid, Invalid, Invalid, None),
            ("abc", Invalid, Invalid, Invalid, None),
        ];

        let mut failures = Vec::new();
        for &(input, z004_expected, integer_expected, quantity_expected, approved) in table {
            let z004 = parse_z004_int(input).map_or(Invalid, |value| Value(value.into()));
            let integer = daily(parse_optional_i64(input));
            let quantity = daily(parse_optional_hundredths(input));
            let actual = (z004, integer, quantity);
            if actual != (z004_expected, integer_expected, quantity_expected) {
                failures.push(format!(
                    "{input:?}: (Z004, 日報の整数, 日報の個数) の期待 {:?} 実際 {actual:?}",
                    (z004_expected, integer_expected, quantity_expected)
                ));
            }
            let differs = z004 != integer || Some(integer) != units(quantity);
            match approved {
                None if differs => {
                    failures.push(format!("{input:?}: 承認済みの差の無い差 {actual:?}"))
                }
                Some((id, _, _)) if !differs => failures.push(format!(
                    "{input:?}: 承認済みの差 {id} の行の 3 つが等しい {actual:?}"
                )),
                Some((id, reason, source))
                    if !["A1", "A2", "A3"].contains(&id)
                        || reason.is_empty()
                        || source.is_empty() =>
                {
                    failures.push(format!("{input:?}: 承認済みの差の欄が不完全"))
                }
                _ => {}
            }
        }
        assert!(failures.is_empty(), "\n{}", failures.join("\n"));
    }

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
