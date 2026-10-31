//! IO-07: Z001/Z002/Z005 日報bundleパーサー
//!
//! CASIO SR-S4000の日報ファイル束をDB非依存の構造化データへ変換する。
//! docs/function-design/29-io-daily-report-parser.md に基づく実装。

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DailyReportSourceFile {
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, specta::Type)]
pub enum DailyReportSourceKind {
    Z001,
    Z002,
    Z005,
}

#[derive(Debug, Clone)]
pub struct ParsedDailyReportSourceFile {
    pub source: DailyReportSourceKind,
    pub filename: String,
    pub file_hash: String,
    pub size_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct DailyReportSummaryLine {
    pub line_key: String,
    pub label: String,
    pub amount: Option<i64>,
    pub quantity_hundredths: Option<i64>,
    pub count: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone)]
pub struct DailyReportPaymentLine {
    pub payment_key: String,
    pub label: String,
    pub amount: Option<i64>,
    pub count: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone)]
pub struct DailyReportDepartmentLine {
    pub raw_department_name: String,
    pub normalized_department_name: Option<String>,
    pub amount: i64,
    pub quantity_hundredths: Option<i64>,
    pub count: Option<i64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone)]
pub struct DailyReportParseError {
    pub source_file: Option<DailyReportSourceKind>,
    pub filename: Option<String>,
    pub line_no: Option<i64>,
    pub error_type: String,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct DailyReportParseResult {
    pub report_date: Option<String>,
    pub source_files: Vec<ParsedDailyReportSourceFile>,
    pub summary_lines: Vec<DailyReportSummaryLine>,
    pub payment_lines: Vec<DailyReportPaymentLine>,
    pub department_lines: Vec<DailyReportDepartmentLine>,
    pub parse_errors: Vec<DailyReportParseError>,
    /// 束の精算回数（IO-07-D5）。3 本とも読めて値がそろい、`i64` に収まる束だけ `Some`
    pub settlement_no: Option<i64>,
}

pub fn parse_daily_report_bundle(files: Vec<DailyReportSourceFile>) -> DailyReportParseResult {
    let mut result = DailyReportParseResult {
        report_date: None,
        source_files: Vec::new(),
        summary_lines: Vec::new(),
        payment_lines: Vec::new(),
        department_lines: Vec::new(),
        parse_errors: Vec::new(),
        settlement_no: None,
    };

    let mut seen_sources = HashSet::new();
    let mut source_dates: HashMap<DailyReportSourceKind, String> = HashMap::new();
    let mut settlement_numbers: HashSet<u64> = HashSet::new();
    let mut settlement_readable = 0;

    for file in files {
        let Some(source) = detect_source(&file.filename) else {
            result.parse_errors.push(parse_error(
                None,
                Some(file.filename),
                None,
                "unknown_source",
                "Z001/Z002/Z005以外のファイルです",
            ));
            continue;
        };

        if !seen_sources.insert(source) {
            result.parse_errors.push(parse_error(
                Some(source),
                Some(file.filename),
                None,
                "duplicate_source",
                "同じsourceのファイルが複数あります",
            ));
            continue;
        }

        let file_hash = sha256_hex(&file.bytes);
        result.source_files.push(ParsedDailyReportSourceFile {
            source,
            filename: file.filename.clone(),
            file_hash,
            size_bytes: file.bytes.len(),
        });

        let (decoded, had_errors) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(&file.bytes);
        if had_errors {
            result.parse_errors.push(parse_error(
                Some(source),
                Some(file.filename),
                None,
                "decode_failed",
                "CP932デコードに失敗しました",
            ));
            continue;
        }

        let normalized = decoded
            .replace("\u{0085}", "\n")
            .replace("\u{2026}", "\n")
            .replace("\r\n", "\n")
            .replace('\r', "\n");

        if let Some(number) = settlement_number(&normalized) {
            settlement_readable += 1;
            settlement_numbers.insert(number);
        }

        if let Some(date) = extract_date(&normalized) {
            source_dates.insert(source, date);
        } else {
            result.parse_errors.push(parse_error(
                Some(source),
                Some(file.filename.clone()),
                None,
                "invalid_date",
                "対象日を抽出できません",
            ));
        }

        match source {
            DailyReportSourceKind::Z001 => {
                parse_z001(
                    &normalized,
                    &mut result.summary_lines,
                    &mut result.parse_errors,
                );
            }
            DailyReportSourceKind::Z002 => {
                parse_z002(
                    &normalized,
                    &mut result.payment_lines,
                    &mut result.parse_errors,
                );
            }
            DailyReportSourceKind::Z005 => {
                parse_z005(
                    &normalized,
                    &mut result.department_lines,
                    &mut result.parse_errors,
                );
            }
        }
    }

    for source in [
        DailyReportSourceKind::Z001,
        DailyReportSourceKind::Z002,
        DailyReportSourceKind::Z005,
    ] {
        if !seen_sources.contains(&source) {
            result.parse_errors.push(parse_error(
                Some(source),
                None,
                None,
                "missing_source",
                "必須sourceのファイルがありません",
            ));
        }
    }

    let unique_dates: HashSet<&String> = source_dates.values().collect();
    if seen_sources.len() == 3 && unique_dates.len() == 1 {
        result.report_date = unique_dates.into_iter().next().cloned();
    } else if seen_sources.len() == 3 {
        result.parse_errors.push(parse_error(
            None,
            None,
            None,
            "invalid_date",
            "Z001/Z002/Z005の日付が一致しません",
        ));
    }

    // IO-07-D3: 精算回数を読めたファイルが 2 本以上で値が違えば別の精算の混在
    if settlement_readable >= 2 && settlement_numbers.len() > 1 {
        result.parse_errors.push(parse_error(
            None,
            None,
            None,
            "settlement_mismatch",
            "Z001/Z002/Z005の精算回数が一致しません",
        ));
    }
    // IO-07-D5: 3 本とも読めてそろう束だけ。i64 に収まらない値は読めないのと同じ
    if settlement_readable == 3 && settlement_numbers.len() == 1 {
        result.settlement_no = settlement_numbers
            .into_iter()
            .next()
            .and_then(|number| i64::try_from(number).ok());
    }

    result
}

/// ヘッダより前の「精算回数」の行（第1列が `精算回数`、第2列が数字だけ）を整数で読む（IO-07-D3）
fn settlement_number(text: &str) -> Option<u64> {
    for line in text.split('\n') {
        let fields = split_csv_fields(line);
        if is_header_fields(&fields) {
            return None;
        }
        if fields[0].trim() == "精算回数" {
            let value = fields.get(1)?.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            return value.parse().ok();
        }
    }
    None
}

fn detect_source(filename: &str) -> Option<DailyReportSourceKind> {
    let upper = filename.to_ascii_uppercase();
    if upper.contains("Z001") {
        Some(DailyReportSourceKind::Z001)
    } else if upper.contains("Z002") {
        Some(DailyReportSourceKind::Z002)
    } else if upper.contains("Z005") {
        Some(DailyReportSourceKind::Z005)
    } else {
        None
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn parse_z001(
    text: &str,
    rows: &mut Vec<DailyReportSummaryLine>,
    errors: &mut Vec<DailyReportParseError>,
) {
    let data_rows = data_rows_after_header(text, DailyReportSourceKind::Z001, errors);
    for (index, (line_no, fields)) in data_rows.into_iter().enumerate() {
        let code = fields[0].trim();
        let label = fields[1].trim().to_string();
        if code.is_empty() && label.is_empty() {
            continue;
        }
        let sort_order = (index + 1) as i64;
        let line_key = summary_line_key(&label, sort_order);
        let is_gross_sales = line_key == "gross_sales";
        // IO-07-D2: 総売の行の第3列は個数（小数 2 桁まで）、他の行は件数（整数）
        let first_value = if is_gross_sales {
            parse_optional_hundredths(&fields[2])
        } else {
            parse_optional_i64(&fields[2])
        };
        let first_value = match first_value {
            Ok(value) => value,
            Err(()) => {
                errors.push(parse_error(
                    Some(DailyReportSourceKind::Z001),
                    None,
                    Some(line_no),
                    "invalid_number",
                    "Z001の個数/件数列を変換できません",
                ));
                continue;
            }
        };
        let amount = match parse_optional_i64(&fields[3]) {
            Ok(value) => value,
            Err(()) => {
                errors.push(parse_error(
                    Some(DailyReportSourceKind::Z001),
                    None,
                    Some(line_no),
                    "invalid_number",
                    "Z001の金額列を変換できません",
                ));
                continue;
            }
        };
        let (quantity, count) = if is_gross_sales {
            (first_value, None)
        } else {
            (None, first_value)
        };
        rows.push(DailyReportSummaryLine {
            line_key,
            label,
            amount,
            quantity_hundredths: quantity,
            count,
            sort_order,
        });
    }
}

fn data_rows_after_header(
    text: &str,
    source: DailyReportSourceKind,
    errors: &mut Vec<DailyReportParseError>,
) -> Vec<(i64, Vec<String>)> {
    let lines: Vec<&str> = text.split('\n').collect();
    if let Some(header_index) = lines.iter().position(|line| {
        let fields = split_csv_fields(line);
        is_header_fields(&fields)
    }) {
        let mut rows = Vec::new();
        for (line_index, line) in lines.iter().enumerate().skip(header_index + 1) {
            if line.trim().is_empty() {
                continue;
            }
            let fields = split_csv_fields(line);
            if fields.len() != 4 {
                errors.push(parse_error(
                    Some(source),
                    None,
                    Some((line_index + 1) as i64),
                    "invalid_format",
                    "日報CSVのデータ行が4列ではありません",
                ));
                continue;
            }
            rows.push(((line_index + 1) as i64, fields));
        }
        if rows.is_empty() {
            errors.push(parse_error(
                Some(source),
                None,
                None,
                "invalid_format",
                "日報CSVのデータ行がありません",
            ));
        }
        return rows;
    }

    let fields = quoted_fields(text);
    if let Some(header_index) = fields.windows(4).position(is_header_fields) {
        let data = &fields[header_index + 4..];
        if data.is_empty() || !data.len().is_multiple_of(4) {
            errors.push(parse_error(
                Some(source),
                None,
                None,
                "invalid_format",
                "日報CSVの連結データ行が4列反復ではありません",
            ));
            return Vec::new();
        }
        return data
            .chunks(4)
            .enumerate()
            .map(|(index, chunk)| ((index + 1) as i64, chunk.to_vec()))
            .collect();
    }

    errors.push(parse_error(
        Some(source),
        None,
        None,
        "invalid_format",
        "日報CSVのヘッダ行を検出できません",
    ));
    Vec::new()
}

fn is_header_fields(fields: &[String]) -> bool {
    fields.len() == 4
        && fields[0].trim().contains("レコード")
        && fields[1].trim().contains("キャラクター")
        && fields[3].trim().contains("金額")
}

fn quoted_fields(text: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut field = String::new();
        for inner in chars.by_ref() {
            if inner == '"' {
                break;
            }
            field.push(inner);
        }
        fields.push(field);
    }
    fields
}

fn parse_z002(
    text: &str,
    rows: &mut Vec<DailyReportPaymentLine>,
    errors: &mut Vec<DailyReportParseError>,
) {
    let data_rows = data_rows_after_header(text, DailyReportSourceKind::Z002, errors);
    for (index, (line_no, fields)) in data_rows.into_iter().enumerate() {
        let code = fields[0].trim();
        let label = fields[1].trim().to_string();
        if code.is_empty() && label.is_empty() {
            continue;
        }
        let count = match parse_optional_i64(&fields[2]) {
            Ok(value) => value,
            Err(()) => {
                errors.push(parse_error(
                    Some(DailyReportSourceKind::Z002),
                    None,
                    Some(line_no),
                    "invalid_number",
                    "Z002の件数列を変換できません",
                ));
                continue;
            }
        };
        let amount = match parse_optional_i64(&fields[3]) {
            Ok(value) => value,
            Err(()) => {
                errors.push(parse_error(
                    Some(DailyReportSourceKind::Z002),
                    None,
                    Some(line_no),
                    "invalid_number",
                    "Z002の金額列を変換できません",
                ));
                continue;
            }
        };
        let sort_order = (index + 1) as i64;
        rows.push(DailyReportPaymentLine {
            payment_key: payment_key(&label, sort_order),
            label,
            amount,
            count,
            sort_order,
        });
    }
}

fn parse_z005(
    text: &str,
    rows: &mut Vec<DailyReportDepartmentLine>,
    errors: &mut Vec<DailyReportParseError>,
) {
    let data_rows = data_rows_after_header(text, DailyReportSourceKind::Z005, errors);
    for (index, (line_no, fields)) in data_rows.into_iter().enumerate() {
        let code = fields[0].trim();
        let raw_department_name = fields[1].trim().to_string();
        if code.is_empty() && raw_department_name.is_empty() {
            continue;
        }
        let quantity = match parse_optional_hundredths(&fields[2]) {
            Ok(value) => value,
            Err(()) => {
                errors.push(parse_error(
                    Some(DailyReportSourceKind::Z005),
                    None,
                    Some(line_no),
                    "invalid_number",
                    "Z005の個数列を変換できません",
                ));
                continue;
            }
        };
        let amount = match parse_required_i64(&fields[3]) {
            Ok(value) => value,
            Err(()) => {
                errors.push(parse_error(
                    Some(DailyReportSourceKind::Z005),
                    None,
                    Some(line_no),
                    "invalid_number",
                    "Z005の金額列を変換できません",
                ));
                continue;
            }
        };
        rows.push(DailyReportDepartmentLine {
            normalized_department_name: Some(raw_department_name.clone()),
            raw_department_name,
            amount,
            quantity_hundredths: quantity,
            count: None,
            sort_order: (index + 1) as i64,
        });
    }
}

/// 100 倍の整数の個数を単位の数へ戻す（IO-07-D2）
pub fn quantity_hundredths_to_units(quantity_hundredths: i64) -> f64 {
    quantity_hundredths as f64 / 100.0
}

fn parse_optional_i64(value: &str) -> Result<Option<i64>, ()> {
    let cleaned = clean_number(value);
    if cleaned.is_empty() {
        return Ok(None);
    }
    cleaned.parse::<i64>().map(Some).map_err(|_| ())
}

/// 個数を小数 2 桁までの 100 倍の整数で読む。浮動小数を通さない（IO-07-D2）
fn parse_optional_hundredths(value: &str) -> Result<Option<i64>, ()> {
    let cleaned = clean_number(value);
    if cleaned.is_empty() {
        return Ok(None);
    }
    let (integer, fraction) = match cleaned.split_once('.') {
        Some((integer, fraction)) if (1..=2).contains(&fraction.len()) => (integer, fraction),
        Some(_) => return Err(()),
        None => (cleaned.as_str(), ""),
    };
    let (negative, digits) = match integer.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, integer),
    };
    let all_digits = |text: &str| text.bytes().all(|byte| byte.is_ascii_digit());
    if digits.is_empty() || !all_digits(digits) || !all_digits(fraction) {
        return Err(());
    }
    let fraction_hundredths: i64 = format!("{fraction:0<2}").parse().map_err(|_| ())?;
    let magnitude = digits
        .parse::<i64>()
        .ok()
        .and_then(|units| units.checked_mul(100))
        .and_then(|hundredths| hundredths.checked_add(fraction_hundredths))
        .ok_or(())?;
    Ok(Some(if negative { -magnitude } else { magnitude }))
}

fn parse_required_i64(value: &str) -> Result<i64, ()> {
    parse_optional_i64(value)?.ok_or(())
}

fn clean_number(value: &str) -> String {
    value.trim().replace([',', '￥', '¥', ' '], "")
}

/// 鍵はラベルだけで決める。「レコード」列は行の位置で鍵に使わない（IO-07-D4）
fn summary_line_key(label: &str, sort_order: i64) -> String {
    if label.contains("総売") || label.eq_ignore_ascii_case("gross_sales") {
        "gross_sales".to_string()
    } else if label.contains("純売") || label.eq_ignore_ascii_case("net_sales") {
        "net_sales".to_string()
    } else {
        fallback_key("summary", sort_order)
    }
}

fn payment_key(label: &str, sort_order: i64) -> String {
    if label.contains("現金") || label.eq_ignore_ascii_case("cash") {
        "cash".to_string()
    } else if label.contains("クレジット") || label.eq_ignore_ascii_case("credit") {
        "credit".to_string()
    } else {
        fallback_key("payment", sort_order)
    }
}

fn fallback_key(prefix: &str, sort_order: i64) -> String {
    format!("{}_{}", prefix, sort_order)
}

fn extract_date(text: &str) -> Option<String> {
    let date_re =
        regex::Regex::new(r"(20\d{2})[-/](\d{1,2})[-/](\d{1,2})").expect("date regex must compile");
    date_re.captures(text).and_then(|caps| {
        let month = caps[2].parse::<u32>().ok()?;
        let day = caps[3].parse::<u32>().ok()?;
        Some(format!("{}-{month:02}-{day:02}", &caps[1]))
    })
}

fn parse_error(
    source_file: Option<DailyReportSourceKind>,
    filename: Option<String>,
    line_no: Option<i64>,
    error_type: &str,
    error_message: &str,
) -> DailyReportParseError {
    DailyReportParseError {
        source_file,
        filename,
        line_no,
        error_type: error_type.to_string(),
        error_message: error_message.to_string(),
    }
}

fn split_csv_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ',' {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    fields.push(current);
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_cp932(text: &str) -> Vec<u8> {
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(text);
        encoded.to_vec()
    }

    fn source_file(filename: &str, text: &str) -> DailyReportSourceFile {
        DailyReportSourceFile {
            filename: filename.to_string(),
            bytes: encode_cp932(text),
        }
    }

    fn preamble(date: &str) -> String {
        preamble_with_settlement_line(date, &settlement_line("0001"))
    }

    /// 精算回数の行（引用符付き）。省くときは空文字を渡す
    fn settlement_line(number: &str) -> String {
        format!("\"精算回数    \",\"{number}\",\"\",\"\"\r\n")
    }

    fn preamble_with_settlement_line(date: &str, settlement_line: &str) -> String {
        format!(
            "\"マシンNo.   \",\"01\",\"\",\"\"\r\n\"ファイル    \",\"synthetic\",\"\",\"\"\r\n\"モード      \",\"精算\",\"\",\"\"\r\n{settlement_line}\"日付        \",\"{date}\",\"\",\"\"\r\n\"時刻        \",\"12:34\",\"\",\"\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数/件数   \",\"金額        \"\r\n"
        )
    }

    /// 合成の 4 列の行を持つ日報ファイル（データの 1 行目は 9 行目。精算回数の行を省くと 8 行目）
    fn report_file(
        filename: &str,
        date: &str,
        settlement_line: &str,
        rows: &[(&str, &str, &str, &str)],
    ) -> DailyReportSourceFile {
        let mut text = preamble_with_settlement_line(date, settlement_line);
        for (code, label, value, amount) in rows {
            text.push_str(&format!(
                "\"{code}\",\"{label}\",\"{value}\",\"{amount}\"\r\n"
            ));
        }
        source_file(filename, &text)
    }

    fn has_error(
        result: &DailyReportParseResult,
        error_type: &str,
        source: Option<DailyReportSourceKind>,
        line_no: Option<i64>,
    ) -> bool {
        result.parse_errors.iter().any(|error| {
            error.error_type == error_type
                && error.source_file == source
                && error.line_no == line_no
        })
    }

    fn has_error_type(result: &DailyReportParseResult, error_type: &str) -> bool {
        result
            .parse_errors
            .iter()
            .any(|error| error.error_type == error_type)
    }

    fn z001(date: &str) -> DailyReportSourceFile {
        source_file(
            "Z001_260321.CSV",
            &format!(
                "{}\"101\",\"総売\",\"8\",\"12000\"\r\n\"201\",\"純売\",\"7\",\"11000\"\r\n",
                preamble(date)
            ),
        )
    }

    fn z002(date: &str) -> DailyReportSourceFile {
        source_file(
            "Z002_260321.CSV",
            &format!(
                "{}\"01\",\"現金\",\"7\",\"11000\"\r\n\"03\",\"クレジット\",\"1\",\"1000\"\r\n",
                preamble(date)
            ),
        )
    }

    fn z005(date: &str) -> DailyReportSourceFile {
        source_file(
            "Z005_260321.CSV",
            &format!(
                "\"マシンNo.   \",\"01\"\r\n\"ファイル    \",\"synthetic\"\r\n\"モード      \",\"精算\"\r\n\"精算回数    \",\"0001\"\r\n\"日付        \",\"{date}\"\r\n\"時刻        \",\"12:34\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数        \",\"金額        \"\r\n\"01\",\"その他小物\",\"4\",\"3000\"\r\n\"02\",\"毛糸\",\"5\",\"8000\"\r\n"
            ),
        )
    }

    #[test]
    fn test_parse_daily_report_req401_happy_path() {
        // REQ-401 / IO-07: Z001/Z002/Z005 bundleを日報行へ正規化する
        let result = parse_daily_report_bundle(vec![
            z001("2026-03-21"),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ]);

        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        assert_eq!(result.report_date.as_deref(), Some("2026-03-21"));
        assert_eq!(result.source_files.len(), 3);
        assert!(result
            .source_files
            .iter()
            .all(|file| file.file_hash.len() == 64
                && file.file_hash == file.file_hash.to_lowercase()
                && file.size_bytes > 0));
        assert_eq!(result.summary_lines.len(), 2);
        assert_eq!(result.summary_lines[0].line_key, "gross_sales");
        assert_eq!(result.summary_lines[0].amount, Some(12000));
        assert_eq!(result.summary_lines[0].quantity_hundredths, Some(800));
        assert_eq!(result.summary_lines[0].count, None);
        assert_eq!(result.summary_lines[1].line_key, "net_sales");
        assert_eq!(result.summary_lines[1].amount, Some(11000));
        assert_eq!(result.summary_lines[1].quantity_hundredths, None);
        assert_eq!(result.summary_lines[1].count, Some(7));
        assert_eq!(result.payment_lines.len(), 2);
        assert_eq!(result.payment_lines[0].payment_key, "cash");
        assert_eq!(result.payment_lines[0].count, Some(7));
        assert_eq!(result.payment_lines[0].amount, Some(11000));
        assert_eq!(result.department_lines.len(), 2);
        assert_eq!(result.department_lines[0].raw_department_name, "その他小物");
        assert_eq!(result.department_lines[0].amount, 3000);
        assert_eq!(result.department_lines[0].quantity_hundredths, Some(400));
        assert_eq!(result.department_lines[0].count, None);
    }

    #[test]
    fn test_parse_daily_report_req401_committed_fixture_bundle() {
        // REQ-401 / IO-07: commit済みCP932 fixture bundleを実ファイルから受理する
        let fixture_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/daily-report");
        let files = ["Z001_260321.CSV", "Z002_260321.CSV", "Z005_260321.CSV"]
            .into_iter()
            .map(|filename| DailyReportSourceFile {
                filename: filename.to_string(),
                bytes: std::fs::read(fixture_dir.join(filename))
                    .unwrap_or_else(|error| panic!("failed to read {filename}: {error}")),
            })
            .collect();

        let result = parse_daily_report_bundle(files);

        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        assert_eq!(result.report_date.as_deref(), Some("2026-03-21"));
        let gross_sales = result
            .summary_lines
            .iter()
            .find(|line| line.line_key == "gross_sales")
            .expect("gross_sales must exist");
        assert_eq!(gross_sales.amount, Some(12000));
        assert_eq!(gross_sales.quantity_hundredths, Some(800));
        let cash = result
            .payment_lines
            .iter()
            .find(|line| line.payment_key == "cash")
            .expect("cash must exist");
        assert_eq!(cash.amount, Some(11000));
        assert_eq!(cash.count, Some(7));
        let other_goods = result
            .department_lines
            .iter()
            .find(|line| line.raw_department_name == "その他小物")
            .expect("その他小物 must exist");
        assert_eq!(other_goods.amount, 3000);
        assert_eq!(other_goods.quantity_hundredths, Some(400));
    }

    #[test]
    fn test_parse_daily_report_req401_missing_source() {
        // REQ-401 / IO-07: Z001/Z002/Z005の欠損はparse error
        let result = parse_daily_report_bundle(vec![z001("2026-03-21"), z002("2026-03-21")]);

        assert!(result
            .parse_errors
            .iter()
            .any(|error| error.error_type == "missing_source"
                && error.source_file == Some(DailyReportSourceKind::Z005)));
    }

    #[test]
    fn test_parse_daily_report_req401_duplicate_and_unknown_source() {
        // REQ-401 / IO-07: source重複と未知sourceはcommit不可エラー
        let unknown = source_file("Z009_260321.CSV", "\"2026-03-21\",\"ignored\"");
        let result = parse_daily_report_bundle(vec![
            z001("2026-03-21"),
            z001("2026-03-21"),
            z002("2026-03-21"),
            z005("2026-03-21"),
            unknown,
        ]);

        assert!(result
            .parse_errors
            .iter()
            .any(|error| error.error_type == "duplicate_source"
                && error.source_file == Some(DailyReportSourceKind::Z001)));
        assert!(result
            .parse_errors
            .iter()
            .any(|error| error.error_type == "unknown_source"
                && error.filename.as_deref() == Some("Z009_260321.CSV")));
    }

    #[test]
    fn test_parse_daily_report_req401_decode_failed() {
        // REQ-401 / IO-07: CP932 strict decode失敗はsource単位のparse error
        let result = parse_daily_report_bundle(vec![
            DailyReportSourceFile {
                filename: "Z001_260321.CSV".to_string(),
                bytes: vec![0x80, 0x00, 0xFF],
            },
            z002("2026-03-21"),
            z005("2026-03-21"),
        ]);

        assert!(result
            .parse_errors
            .iter()
            .any(|error| error.error_type == "decode_failed"
                && error.source_file == Some(DailyReportSourceKind::Z001)));
    }

    #[test]
    fn test_parse_daily_report_req401_date_mismatch() {
        // REQ-401 / IO-07: 3ファイルの日付不一致はinvalid_date
        let result = parse_daily_report_bundle(vec![
            z001("2026-03-21"),
            z002("2026-03-22"),
            z005("2026-03-21"),
        ]);

        assert_eq!(result.report_date, None);
        assert!(result
            .parse_errors
            .iter()
            .any(|error| error.error_type == "invalid_date"));
    }

    #[test]
    fn test_parse_daily_report_req401_invalid_number() {
        // REQ-401 / IO-07: source別数値列の変換失敗はinvalid_number
        let invalid_z005 = source_file(
            "Z005_260321.CSV",
            "\"マシンNo.   \",\"01\"\r\n\"ファイル    \",\"synthetic\"\r\n\"モード      \",\"精算\"\r\n\"精算回数    \",\"0001\"\r\n\"日付        \",\"2026-03-21\"\r\n\"時刻        \",\"12:34\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数        \",\"金額        \"\r\n\"01\",\"その他小物\",\"4\",\"not-number\"\r\n",
        );
        let result =
            parse_daily_report_bundle(vec![z001("2026-03-21"), z002("2026-03-21"), invalid_z005]);

        assert!(result
            .parse_errors
            .iter()
            .any(|error| error.error_type == "invalid_number"
                && error.source_file == Some(DailyReportSourceKind::Z005)));
    }

    #[test]
    fn test_parse_daily_report_req401_invalid_format_z002_z005() {
        // REQ-401 / IO-07: header後の4列データ行が崩れた行構造はinvalid_format
        let invalid_z002 = source_file(
            "Z002_260321.CSV",
            &format!("{}\"01\",\"現金\",\"7\"\r\n", preamble("2026-03-21")),
        );
        let invalid_z005 = source_file(
            "Z005_260321.CSV",
            "\"マシンNo.   \",\"01\"\r\n\"ファイル    \",\"synthetic\"\r\n\"モード      \",\"精算\"\r\n\"精算回数    \",\"0001\"\r\n\"日付        \",\"2026-03-21\"\r\n\"時刻        \",\"12:34\"\r\n\r\n\"レコード    \",\"キャラクター\",\"個数        \",\"金額        \"\r\n\"01\",\"その他小物\",\"4\"\r\n",
        );
        let result =
            parse_daily_report_bundle(vec![z001("2026-03-21"), invalid_z002, invalid_z005]);

        assert!(result.parse_errors.iter().any(|error| {
            error.error_type == "invalid_format"
                && error.source_file == Some(DailyReportSourceKind::Z002)
        }));
        assert!(result.parse_errors.iter().any(|error| {
            error.error_type == "invalid_format"
                && error.source_file == Some(DailyReportSourceKind::Z005)
        }));
    }

    #[test]
    fn test_parse_daily_report_req401_source_shapes() {
        // REQ-401 / IO-07: 匿名化shape
        // Z001/Z002/Z005=7行プリアンブル+header+4列データ行
        let result = parse_daily_report_bundle(vec![
            z001("2026-03-21"),
            z002("2026-03-21"),
            z005("2026-03-21"),
        ]);

        assert_eq!(result.summary_lines.len(), 2);
        assert_eq!(result.payment_lines.len(), 2);
        assert_eq!(result.department_lines.len(), 2);
        assert_eq!(result.summary_lines[0].sort_order, 1);
        assert_eq!(result.payment_lines[1].sort_order, 2);
        assert_eq!(result.department_lines[1].sort_order, 2);
    }

    #[test]
    fn test_parse_daily_report_req401_layout_b_concatenated_shape_supported() {
        // REQ-401 / IO-07: エクスポート機能の連結型layout Bも4列データ行として正規化する
        let layout_b_z002 = source_file(
            "Z002_260321.CSV",
            "\"出力\",\"PC\"\"内容\",\"日計\"\"日付\",\"2026-03-21\"\"レコード\",\"キャラクター\",\"個数/件数\",\"金額\"\"0001\",\"現金\",\"7\",\"11000\"\"0003\",\"クレジット\",\"1\",\"1000\"",
        );
        let layout_b_z005 = source_file(
            "Z005_260321.CSV",
            "\"出力\",\"PC\"\"内容\",\"日計\"\"日付\",\"2026-03-21\"\"レコード\",\"キャラクター\",\"個数\",\"金額\"\"0001\",\"その他小物\",\"4\",\"3000\"\"0002\",\"毛糸\",\"5\",\"8000\"",
        );

        let result =
            parse_daily_report_bundle(vec![z001("2026-03-21"), layout_b_z002, layout_b_z005]);

        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        assert_eq!(result.payment_lines.len(), 2);
        assert_eq!(result.payment_lines[0].payment_key, "cash");
        assert_eq!(result.payment_lines[0].count, Some(7));
        assert_eq!(result.payment_lines[0].amount, Some(11000));
        assert_eq!(result.department_lines.len(), 2);
        assert_eq!(result.department_lines[0].quantity_hundredths, Some(400));
        assert_eq!(result.department_lines[0].amount, 3000);
    }

    #[test]
    fn test_parse_daily_report_req401_decimal_quantity_hundredths() {
        // REQ-401 / IO-07-D2: 個数（Z001 の総売・Z005）は小数 2 桁までを浮動小数を通さず 100 倍の整数にする
        let s = settlement_line("0001");
        let result = parse_daily_report_bundle(vec![
            report_file(
                "Z001_01.CSV",
                "2026-09-29",
                &s,
                &[
                    ("0001", "総売", "1.3", "12000"),
                    ("0002", "純売", "7", "11000"),
                ],
            ),
            report_file(
                "Z002_01.CSV",
                "2026-09-29",
                &s,
                &[("0001", "現金", "7", "11000")],
            ),
            report_file(
                "Z005_01.CSV",
                "2026-09-29",
                &s,
                &[
                    ("0001", "部門A", "2.25", "100"),
                    ("0002", "部門B", "-0.5", "-50"),
                    ("0003", "部門C", "1,234.5", "5000"),
                    ("0004", "部門D", "4", "400"),
                    ("0005", "部門E", "1.13", "1"),
                    ("0006", "部門F", "0.29", "1"),
                    ("0007", "部門G", "9999.99", "1"),
                    ("0008", "部門H", "0", "1"),
                    ("0009", "部門I", "0.0", "1"),
                    ("0010", "部門J", "1.05", "1"),
                ],
            ),
        ]);

        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        assert_eq!(result.summary_lines[0].line_key, "gross_sales");
        assert_eq!(result.summary_lines[0].quantity_hundredths, Some(130));
        assert_eq!(result.summary_lines[0].count, None);
        assert_eq!(result.summary_lines[1].quantity_hundredths, None);
        assert_eq!(result.summary_lines[1].count, Some(7));
        let quantities: Vec<Option<i64>> = result
            .department_lines
            .iter()
            .map(|line| line.quantity_hundredths)
            .collect();
        assert_eq!(
            quantities,
            [225, -50, 123450, 400, 113, 29, 999999, 0, 0, 105].map(Some)
        );
    }

    #[test]
    fn test_parse_daily_report_req401_decimal_quantity_rejects_invalid_shapes() {
        // REQ-401 / IO-07-D2: 小数 3 桁以上・`.5`・`1.`・`1.2.3` は丸めず invalid_number
        let s = settlement_line("0001");
        for invalid in ["1.234", ".5", "1.", "1.2.3", "-.5", "1.-5"] {
            let result = parse_daily_report_bundle(vec![
                report_file(
                    "Z001_01.CSV",
                    "2026-09-29",
                    &s,
                    &[
                        ("0001", "総売", invalid, "12000"),
                        ("0002", "純売", "7", "11000"),
                    ],
                ),
                report_file(
                    "Z002_01.CSV",
                    "2026-09-29",
                    &s,
                    &[("0001", "現金", "7", "11000")],
                ),
                report_file(
                    "Z005_01.CSV",
                    "2026-09-29",
                    &s,
                    &[("0001", "部門A", invalid, "100")],
                ),
            ]);
            assert!(
                has_error(
                    &result,
                    "invalid_number",
                    Some(DailyReportSourceKind::Z001),
                    Some(9)
                ),
                "Z001 {invalid}: {:?}",
                result.parse_errors
            );
            assert!(
                has_error(
                    &result,
                    "invalid_number",
                    Some(DailyReportSourceKind::Z005),
                    Some(9)
                ),
                "Z005 {invalid}: {:?}",
                result.parse_errors
            );
        }
    }

    #[test]
    fn test_parse_daily_report_req401_count_column_stays_integer() {
        // REQ-401 / IO-07-D2: 件数の列（Z001 の総売以外、Z002）は整数のまま。小数は invalid_number
        let s = settlement_line("0001");
        let result = parse_daily_report_bundle(vec![
            report_file(
                "Z001_01.CSV",
                "2026-09-29",
                &s,
                &[("0001", "総売", "3", "100"), ("0002", "純売", "1.5", "90")],
            ),
            report_file(
                "Z002_01.CSV",
                "2026-09-29",
                &s,
                &[("0001", "現金", "1.5", "100")],
            ),
            report_file(
                "Z005_01.CSV",
                "2026-09-29",
                &s,
                &[("0001", "部門A", "3", "100")],
            ),
        ]);
        assert!(
            has_error(
                &result,
                "invalid_number",
                Some(DailyReportSourceKind::Z001),
                Some(10)
            ),
            "{:?}",
            result.parse_errors
        );
        assert!(
            has_error(
                &result,
                "invalid_number",
                Some(DailyReportSourceKind::Z002),
                Some(9)
            ),
            "{:?}",
            result.parse_errors
        );

        let result = parse_daily_report_bundle(vec![
            report_file(
                "Z001_01.CSV",
                "2026-09-29",
                &s,
                &[("0001", "総売", "3", "100"), ("0002", "純売", "7", "90")],
            ),
            report_file(
                "Z002_01.CSV",
                "2026-09-29",
                &s,
                &[("0001", "現金", "7", "100")],
            ),
            report_file(
                "Z005_01.CSV",
                "2026-09-29",
                &s,
                &[("0001", "部門A", "3", "100")],
            ),
        ]);
        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        assert_eq!(result.summary_lines[1].count, Some(7));
        assert_eq!(result.summary_lines[1].quantity_hundredths, None);
        assert_eq!(result.payment_lines[0].count, Some(7));
        assert_eq!(result.department_lines[0].count, None);
    }

    #[test]
    fn test_daily_report_req401_quantity_hundredths_to_units() {
        // REQ-401 / IO-07-D2: 100 倍の整数を単位の数（f64）へ戻す
        assert_eq!(quantity_hundredths_to_units(130), 1.3);
        assert_eq!(quantity_hundredths_to_units(400), 4.0);
        assert_eq!(quantity_hundredths_to_units(-50), -0.5);
        assert_eq!(quantity_hundredths_to_units(125), 1.25);
        assert_eq!(serde_json::to_string(&1.3_f64).unwrap(), "1.3");
        assert_eq!(
            serde_json::to_string(&quantity_hundredths_to_units(130)).unwrap(),
            "1.3"
        );
    }

    #[test]
    fn test_parse_daily_report_req401_settlement_mismatch() {
        // REQ-401 / IO-07-D3: 精算回数を読めた 2 本以上が違えば settlement_mismatch。日付の判定とは別
        let rows = [("0001", "総売", "3", "100"), ("0002", "純売", "3", "90")];
        let mismatched = parse_daily_report_bundle(vec![
            report_file(
                "Z001_28 _0001.CSV",
                "2026-09-28",
                &settlement_line("0001"),
                &rows,
            ),
            report_file(
                "Z002_28A_0002.CSV",
                "2026-09-28",
                &settlement_line("0002"),
                &[("0001", "現金", "3", "90")],
            ),
            report_file(
                "Z005_28A_0002.CSV",
                "2026-09-28",
                &settlement_line("0002"),
                &[("0001", "部門A", "3", "90")],
            ),
        ]);
        assert!(
            has_error(&mismatched, "settlement_mismatch", None, None),
            "{:?}",
            mismatched.parse_errors
        );
        assert!(mismatched
            .parse_errors
            .iter()
            .filter(|error| error.error_type == "settlement_mismatch")
            .all(|error| !error.error_message.is_empty() && error.filename.is_none()));
        assert!(!has_error_type(&mismatched, "invalid_date"));
        assert_eq!(mismatched.report_date.as_deref(), Some("2026-09-28"));

        // T9: 日付も精算回数も違えば両方を返す
        let both = parse_daily_report_bundle(vec![
            report_file(
                "Z001_28 _0001.CSV",
                "2026-09-28",
                &settlement_line("0001"),
                &rows,
            ),
            report_file(
                "Z002_29 _0002.CSV",
                "2026-09-29",
                &settlement_line("0002"),
                &[("0001", "現金", "3", "90")],
            ),
            report_file(
                "Z005_28 _0001.CSV",
                "2026-09-28",
                &settlement_line("0001"),
                &[("0001", "部門A", "3", "90")],
            ),
        ]);
        assert!(
            has_error_type(&both, "invalid_date"),
            "{:?}",
            both.parse_errors
        );
        assert!(
            has_error_type(&both, "settlement_mismatch"),
            "{:?}",
            both.parse_errors
        );
    }

    #[test]
    fn test_parse_daily_report_req401_settlement_number_read_from_preamble_only() {
        // REQ-401 / IO-07-D3: 精算回数はヘッダより前の行だけを整数で読み、読めた 2 本以上だけを比べる
        let z001_rows = [("0001", "総売", "3", "100"), ("0002", "純売", "3", "90")];
        let z002_rows = [("0001", "現金", "3", "90")];
        let z005_rows = [("0001", "部門A", "3", "90")];
        let date = "2026-09-28";

        // (a) 引用符なしの `精算回数,5,,` と `"0005"`・`"5"` は同じ値
        let a = parse_daily_report_bundle(vec![
            report_file("Z001_a.CSV", date, "精算回数,5,,\r\n", &z001_rows),
            report_file("Z002_a.CSV", date, &settlement_line("0005"), &z002_rows),
            report_file("Z005_a.CSV", date, &settlement_line("5"), &z005_rows),
        ]);
        assert!(a.parse_errors.is_empty(), "(a) {:?}", a.parse_errors);

        // (b) 精算回数の行の無い Z005 は比べない（読めた 2 本は一致）
        let b = parse_daily_report_bundle(vec![
            report_file("Z001_b.CSV", date, &settlement_line("0001"), &z001_rows),
            report_file("Z002_b.CSV", date, &settlement_line("0001"), &z002_rows),
            report_file("Z005_b.CSV", date, "", &z005_rows),
        ]);
        assert!(b.parse_errors.is_empty(), "(b) {:?}", b.parse_errors);

        // (c) ヘッダの後の第1列 `精算回数` の行は精算回数として読まない（Z002 は読めない 1 本）
        let c = parse_daily_report_bundle(vec![
            report_file("Z001_c.CSV", date, &settlement_line("0001"), &z001_rows),
            report_file(
                "Z002_c.CSV",
                date,
                "",
                &[
                    ("0001", "現金", "3", "90"),
                    ("精算回数", "0009", "1", "100"),
                ],
            ),
            report_file("Z005_c.CSV", date, &settlement_line("0001"), &z005_rows),
        ]);
        assert!(c.parse_errors.is_empty(), "(c) {:?}", c.parse_errors);
        let row = c
            .payment_lines
            .iter()
            .find(|line| line.label == "0009")
            .expect("ヘッダの後の行は Z002 の支払の行として残る");
        assert_eq!(row.payment_key, "payment_2");

        // (d) 読める 2 本が不一致で残り 1 本が読めない束は settlement_mismatch
        let d = parse_daily_report_bundle(vec![
            report_file("Z001_d.CSV", date, &settlement_line("0001"), &z001_rows),
            report_file("Z002_d.CSV", date, &settlement_line("0002"), &z002_rows),
            report_file("Z005_d.CSV", date, "", &z005_rows),
        ]);
        assert!(
            has_error_type(&d, "settlement_mismatch"),
            "(d) {:?}",
            d.parse_errors
        );
    }

    #[test]
    fn test_parse_daily_report_req401_record_column_is_row_position() {
        // REQ-401 / IO-07-D4: 「レコード」列は行の位置。鍵はラベルだけで決める
        let s = settlement_line("0001");
        let result = parse_daily_report_bundle(vec![
            report_file(
                "Z001_01.CSV",
                "2026-09-28",
                &s,
                &[
                    ("0001", "総売", "3", "100"),
                    ("0002", "純売", "3", "90"),
                    ("101", "合成項目甲", "1", "10"),
                    ("201", "合成項目乙", "1", "10"),
                ],
            ),
            report_file(
                "Z002_01.CSV",
                "2026-09-28",
                &s,
                &[
                    ("0001", "現金", "3", "90"),
                    ("0002", "合成支払甲", "1", "10"),
                    ("0003", "合成支払乙", "1", "10"),
                    ("03", "合成支払丙", "1", "10"),
                    ("01", "合成支払丁", "1", "10"),
                    ("0006", "合成ｸﾚｼﾞｯﾄ甲", "1", "10"),
                    ("0007", "合成ｸﾚｼﾞｯﾄ乙", "1", "10"),
                ],
            ),
            report_file(
                "Z005_01.CSV",
                "2026-09-28",
                &s,
                &[("0001", "部門A", "3", "90")],
            ),
        ]);

        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        let summary_keys: Vec<&str> = result
            .summary_lines
            .iter()
            .map(|line| line.line_key.as_str())
            .collect();
        assert_eq!(
            summary_keys,
            ["gross_sales", "net_sales", "summary_3", "summary_4"]
        );
        let payment_keys: Vec<&str> = result
            .payment_lines
            .iter()
            .map(|line| line.payment_key.as_str())
            .collect();
        assert_eq!(
            payment_keys,
            [
                "cash",
                "payment_2",
                "payment_3",
                "payment_4",
                "payment_5",
                "payment_6",
                "payment_7"
            ]
        );
    }

    /// 3 本とも同じ精算回数の行を持つ束
    fn settlement_bundle(numbers: [&str; 3]) -> DailyReportParseResult {
        let date = "2026-09-28";
        parse_daily_report_bundle(vec![
            report_file(
                "Z001_s.CSV",
                date,
                &settlement_line(numbers[0]),
                &[("0001", "総売", "3", "100"), ("0002", "純売", "3", "90")],
            ),
            report_file(
                "Z002_s.CSV",
                date,
                &settlement_line(numbers[1]),
                &[("0001", "現金", "3", "90")],
            ),
            report_file(
                "Z005_s.CSV",
                date,
                &settlement_line(numbers[2]),
                &[("0001", "部門A", "3", "90")],
            ),
        ])
    }

    #[test]
    fn test_req401_settlement_no_is_some_when_all_three_agree() {
        // REQ-401 / IO-07-D5: 3 本とも読めてそろう束だけ Some（先頭の 0 は値に影響しない）
        let result = settlement_bundle(["0005", "5", "05"]);
        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        assert_eq!(result.settlement_no, Some(5));
    }

    #[test]
    fn test_req401_settlement_no_is_none_when_any_unreadable() {
        // REQ-401 / IO-07-D5: Z001 だけ読め、Z002 / Z005 が layout B（精算回数の行が無い）の束は None
        let date = "2026-09-28";
        let z001 = report_file(
            "Z001_b.CSV",
            date,
            &settlement_line("0005"),
            &[("0001", "総売", "3", "100"), ("0002", "純売", "3", "90")],
        );
        let z002 = source_file(
            "Z002_b.CSV",
            "\"出力\",\"PC\"\"内容\",\"日計\"\"日付\",\"2026-09-28\"\"レコード\",\"キャラクター\",\"個数/件数\",\"金額\"\"0001\",\"現金\",\"3\",\"90\"",
        );
        let z005 = source_file(
            "Z005_b.CSV",
            "\"出力\",\"PC\"\"内容\",\"日計\"\"日付\",\"2026-09-28\"\"レコード\",\"キャラクター\",\"個数\",\"金額\"\"0001\",\"部門A\",\"3\",\"90\"",
        );
        let result = parse_daily_report_bundle(vec![z001, z002, z005]);
        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        assert_eq!(result.settlement_no, None);
        // 1 本だけ精算回数の行を欠く layout A の束も None
        let mut missing = settlement_bundle(["0005", "0005", "0005"]);
        assert_eq!(missing.settlement_no, Some(5));
        missing = parse_daily_report_bundle(vec![
            report_file(
                "Z001_m.CSV",
                date,
                &settlement_line("0005"),
                &[("0001", "総売", "3", "100"), ("0002", "純売", "3", "90")],
            ),
            report_file(
                "Z002_m.CSV",
                date,
                &settlement_line("0005"),
                &[("0001", "現金", "3", "90")],
            ),
            report_file("Z005_m.CSV", date, "", &[("0001", "部門A", "3", "90")]),
        ]);
        assert!(
            missing.parse_errors.is_empty(),
            "{:?}",
            missing.parse_errors
        );
        assert_eq!(missing.settlement_no, None);
    }

    #[test]
    fn test_req401_settlement_no_is_none_on_mismatch() {
        // REQ-401 / IO-07-D5: settlement_mismatch の束は None
        let result = settlement_bundle(["0005", "0005", "0006"]);
        assert!(has_error_type(&result, "settlement_mismatch"));
        assert_eq!(result.settlement_no, None);
    }

    #[test]
    fn test_req401_settlement_no_out_of_i64_is_none_and_zero_is_some() {
        // REQ-401 / IO-07-D5: i64 に収まらない値は None（負の値に化けない）、i64::MAX と 0 は Some
        let over = settlement_bundle([
            "9223372036854775808",
            "9223372036854775808",
            "9223372036854775808",
        ]);
        assert!(over.parse_errors.is_empty(), "{:?}", over.parse_errors);
        assert_eq!(over.settlement_no, None);

        let max = settlement_bundle([
            "9223372036854775807",
            "9223372036854775807",
            "9223372036854775807",
        ]);
        assert!(max.parse_errors.is_empty(), "{:?}", max.parse_errors);
        assert_eq!(max.settlement_no, Some(i64::MAX));

        let zero = settlement_bundle(["0000", "0000", "0000"]);
        assert!(zero.parse_errors.is_empty(), "{:?}", zero.parse_errors);
        assert_eq!(zero.settlement_no, Some(0));
    }
}
