//! IO-02: Z004パーサー
//!
//! カシオSR-S4000のZ004ファイル（CP932/CSV）を構造化データに変換する。
//! 純関数。DB非依存。
//!
//! docs/function-design/23-io-z004-parser.md に基づく実装。

use sha2::{Digest, Sha256};
use std::fmt;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// Z004パース成功結果（行単位エラーがあっても返る）
#[derive(Debug)]
pub struct ParseResult {
    /// 精算日（YYYY-MM-DD、入力 shape に応じたメタ行から抽出）
    pub settlement_date: String,
    /// 正常にパースできたデータ行
    pub parsed_rows: Vec<ParsedRow>,
    /// 行単位のパースエラー
    pub parse_errors: Vec<ParseError>,
    /// ヘッダ行より後の非空行でパースを試みた総数（Ok(Some)+Ok(None)+Err）
    pub total_data_lines: usize,
    /// SHA-256ハッシュ（raw bytes基準、hex小文字64文字。INV-6準拠）
    pub file_hash: String,
    /// 精算の識別メタ（layout A のメタ行。従来 shape は None）。23-io-z004-parser.md 時点証拠契約
    pub settlement_metadata: Option<SettlementMetadata>,
}

/// 精算の識別メタ。番号は意味が検証されるまで文字列のまま（先頭の 0 を保つ）。
/// parser は精算系列・同一性・時計の信用を認定しない
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettlementMetadata {
    /// 「マシンNo.」行の値
    pub machine_no: Option<String>,
    /// 「ファイル」行の値（帳票種別）
    pub report_kind: Option<String>,
    /// 「精算回数」行の値
    pub settlement_no: Option<String>,
    /// 精算日と「時刻」行の `HH:MM` を結合した `YYYY-MM-DDTHH:MM`（分精度、秒や 0 時を補わない）
    pub settled_at: Option<String>,
}

/// 正常にパースできた1データ行
#[derive(Debug, Clone)]
pub struct ParsedRow {
    /// ファイル内行番号（1始まり）
    pub line_no: usize,
    /// 正規化後13桁JANコード
    pub normalized_jan: String,
    /// Z004上の商品名（そのまま）
    pub name: String,
    /// 数量（マイナス=返品）
    pub quantity: i32,
    /// 金額（マイナス=返品）
    pub amount: i32,
}

/// レジ全スロット占有 snapshot の1行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluRegisterSlot {
    pub memory_no: i64,
    pub raw_code: Option<String>,
}

/// 行単位パースエラー（他の行の処理は継続）
#[derive(Debug, Clone)]
pub struct ParseError {
    pub line_no: usize,
    pub error_type: ParseErrorType,
    pub error_message: String,
    /// パース途中で取得できた商品名（取得前のエラーではNone）
    pub raw_name: Option<String>,
    pub raw_quantity: Option<String>,
    pub raw_amount: Option<String>,
}

/// パースエラーの種別
///
/// db-design/pos-tables.md csv_import_errors の error_type CHECK制約に対応
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum ParseErrorType {
    /// フィールド数不正等の構造エラー
    InvalidFormat,
    /// JANコード正規化失敗
    InvalidJan,
    /// 数量・金額の数値変換失敗
    InvalidNumber,
}

/// 致命的エラー（ファイル全体の処理を中断）
#[derive(Debug)]
pub enum Z004ParseError {
    /// CP932デコード失敗
    DecodeFailed(String),
    /// 2行未満（ヘッダ行すらない）
    NoDataLines(String),
    /// ヘッダまたは精算日を抽出不能
    NoSettlementDate(String),
    /// 全スロット snapshot の構造が契約を満たさない。
    ImportError(String),
}

impl fmt::Display for Z004ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Z004ParseError::DecodeFailed(msg) => write!(f, "{}", msg),
            Z004ParseError::NoDataLines(msg) => write!(f, "{}", msg),
            Z004ParseError::NoSettlementDate(msg) => write!(f, "{}", msg),
            Z004ParseError::ImportError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Z004ParseError {}

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// Z004ファイルの生バイト列を構造化データに変換する
///
/// 23-io-z004-parser.md セクション13.3
pub fn parse_z004(raw_bytes: &[u8]) -> Result<ParseResult, Z004ParseError> {
    // Step 1: file_hash算出（raw bytesから。INV-6準拠）
    let mut hasher = Sha256::new();
    hasher.update(raw_bytes);
    let hash_result = hasher.finalize();
    let file_hash = format!("{:x}", hash_result);

    // Step 2: CP932 strictデコード
    let (decoded, had_errors) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(raw_bytes);
    if had_errors {
        return Err(Z004ParseError::DecodeFailed(
            "CP932デコードに失敗しました。ファイル形式を確認してください".to_string(),
        ));
    }

    // Step 3: 改行正規化（\u{0085}, \r\n, \r → \n）
    let normalized = decoded
        .replace("\u{0085}", "\n")
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    // Step 4: 行分割（空行は除去しない — 行番号保持）
    let lines: Vec<&str> = normalized.split('\n').collect();

    // Step 5: 2行未満チェック
    if lines.len() < 2 {
        return Err(Z004ParseError::NoDataLines(
            "データ行がありません。ファイル形式を確認してください".to_string(),
        ));
    }

    // Step 6: 従来 shape と layout A を判定し、ヘッダ位置と精算日を確定
    let conventional_date = extract_iso_date(lines[0]);
    let is_conventional_shape =
        conventional_date.is_some() && is_conventional_header_line(lines[1]);

    let (header_index, settlement_date, settlement_metadata) = if is_conventional_shape {
        (
            1,
            conventional_date.expect("従来 shape 判定済みの日付が存在する"),
            None,
        )
    } else {
        const HEADER_SCAN_LIMIT: usize = 20;
        let header_index = lines
            .iter()
            .take(HEADER_SCAN_LIMIT)
            .position(|line| is_layout_a_header_line(line))
            .ok_or_else(|| {
                Z004ParseError::NoSettlementDate(
                    "ヘッダ行を検出できません。ファイル形式を確認してください".to_string(),
                )
            })?;

        let metadata = &lines[..header_index];
        let labeled_date = metadata.iter().find_map(|line| {
            let fields = split_csv_fields(line);
            fields
                .first()
                .filter(|label| label.contains("日付"))
                .and_then(|_| extract_normalized_date(line))
        });
        let fallback_date = || {
            metadata
                .iter()
                .find_map(|line| extract_normalized_date(line))
        };
        let settlement_date = labeled_date.or_else(fallback_date).ok_or_else(|| {
            Z004ParseError::NoSettlementDate(
                "精算日を抽出できません。ファイル形式を確認してください".to_string(),
            )
        })?;

        let settlement_metadata = extract_settlement_metadata(metadata, &settlement_date);
        (header_index, settlement_date, Some(settlement_metadata))
    };

    // Step 7: 検出したヘッダ行をスキップ
    // Step 8: ヘッダ行より後をパース
    let mut parsed_rows = Vec::new();
    let mut parse_errors = Vec::new();
    let mut total_data_lines: usize = 0;

    for (i, line) in lines.iter().enumerate().skip(header_index + 1) {
        let line_no = i + 1; // 1始まり

        // 空行スキップ（カウントしない）
        if line.trim().is_empty() {
            continue;
        }

        total_data_lines += 1;

        match parse_data_line(line, line_no) {
            Ok(Some(row)) => parsed_rows.push(row),
            Ok(None) => {} // 売上の無い枠（SPEC-Z4A-D8）— スキップ
            Err(error) => parse_errors.push(error),
        }
    }

    Ok(ParseResult {
        settlement_date,
        parsed_rows,
        parse_errors,
        total_data_lines,
        file_hash,
        settlement_metadata,
    })
}

/// Z004 全スロットダンプを占有 snapshot として fail-closed で読む。
pub fn parse_plu_register_snapshot(
    raw_bytes: &[u8],
) -> Result<Vec<PluRegisterSlot>, Z004ParseError> {
    let (decoded, had_errors) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(raw_bytes);
    if had_errors {
        return Err(Z004ParseError::DecodeFailed(
            "CP932デコードに失敗しました。ファイル形式を確認してください".to_string(),
        ));
    }
    let normalized = decoded
        .replace("\u{0085}", "\n")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    const HEADER_SCAN_LIMIT: usize = 20;
    let header_index = lines
        .iter()
        .take(HEADER_SCAN_LIMIT)
        .position(|line| is_layout_a_header_line(line))
        .ok_or_else(|| {
            Z004ParseError::ImportError(
                "ヘッダ行を検出できません。ファイル形式を確認してください".to_string(),
            )
        })?;

    let data_lines: Vec<&str> = lines[header_index + 1..]
        .iter()
        .copied()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if data_lines.len() != 5_000 {
        return Err(Z004ParseError::ImportError(format!(
            "PLUスロット行数が不正です（期待: 5000, 実際: {}）",
            data_lines.len()
        )));
    }

    let mut seen = vec![false; 5_001];
    let mut slots = Vec::with_capacity(5_000);
    for line in data_lines {
        let fields = split_csv_fields(line);
        if fields.len() != 5 {
            return Err(Z004ParseError::ImportError(format!(
                "PLUスロット行のフィールド数が不正です（実際: {}）",
                fields.len()
            )));
        }
        let memory_no: usize = fields[0].trim().parse().map_err(|_| {
            Z004ParseError::ImportError(format!("memory No.が不正です: '{}'", fields[0]))
        })?;
        if !(1..=5_000).contains(&memory_no) || seen[memory_no] {
            return Err(Z004ParseError::ImportError(format!(
                "memory No.が範囲外または重複しています: {memory_no}"
            )));
        }
        seen[memory_no] = true;

        let raw = &fields[1];
        let raw_code = if raw.len() == 14 && raw.bytes().all(|byte| byte == b'0') {
            None
        } else if raw.len() == 14
            && raw.as_bytes()[..13].iter().all(u8::is_ascii_digit)
            && raw.as_bytes()[13] == b'E'
        {
            Some(raw[..13].to_string())
        } else if raw.is_empty() {
            None
        } else {
            Some(raw.clone())
        };
        slots.push(PluRegisterSlot {
            memory_no: memory_no as i64,
            raw_code,
        });
    }
    if seen[1..].iter().any(|seen| !seen) {
        return Err(Z004ParseError::ImportError(
            "memory No.が欠落しています".to_string(),
        ));
    }
    slots.sort_by_key(|slot| slot.memory_no);
    Ok(slots)
}

// ---------------------------------------------------------------------------
// 内部関数
// ---------------------------------------------------------------------------

/// Z004の1データ行をパースする
///
/// 23-io-z004-parser.md セクション13.4
fn parse_data_line(line: &str, line_no: usize) -> Result<Option<ParsedRow>, ParseError> {
    // Step 1: CSVフィールド分割（ダブルクォート対応）
    let fields = split_csv_fields(line);
    if fields.len() != 5 {
        return Err(ParseError {
            line_no,
            error_type: ParseErrorType::InvalidFormat,
            error_message: format!(
                "行{}: フィールド数が不正です（期待: 5, 実際: {}）",
                line_no,
                fields.len()
            ),
            raw_name: fields.get(2).map(|s| s.to_string()),
            raw_quantity: fields.get(3).map(|s| s.to_string()),
            raw_amount: fields.get(4).map(|s| s.to_string()),
        });
    }

    let record_no = fields[0].trim();
    let scanning_code_raw = &fields[1];
    let name_raw = &fields[2];
    let quantity_raw = &fields[3];
    let amount_raw = &fields[4];
    let row_error = |error_type, error_message| ParseError {
        line_no,
        error_type,
        error_message,
        raw_name: Some(name_raw.to_string()),
        raw_quantity: Some(quantity_raw.to_string()),
        raw_amount: Some(amount_raw.to_string()),
    };

    // Step 2-3: 個数・金額を先に読む（§13.4.1）
    let quantity = parse_z004_int(quantity_raw).ok_or_else(|| {
        row_error(
            ParseErrorType::InvalidNumber,
            format!(
                "行{}: 数量が数値ではありません: '{}'",
                line_no, quantity_raw
            ),
        )
    })?;
    let amount = parse_z004_int(amount_raw).ok_or_else(|| {
        row_error(
            ParseErrorType::InvalidNumber,
            format!("行{}: 金額が数値ではありません: '{}'", line_no, amount_raw),
        )
    })?;

    // Step 4-5: 売上の有無で分類する（§13.4.2、SPEC-Z4A-D8）
    let has_sales = quantity != 0 || amount != 0;
    let normalized_jan = match normalize_jan(scanning_code_raw, line_no) {
        Ok(Some(jan)) => jan, // 0/0 でも返す（除外は BIZ-03 Stage 2）
        Ok(None) | Err(_) if !has_sales => return Ok(None), // 売上の無い枠
        Ok(None) => {
            return Err(row_error(
                ParseErrorType::InvalidJan,
                format!(
                    "行{}: 商品コードの無い枠（メモリNo.{}）に売上があります。PLU の登録を消した枠の売上などで、在庫には反映されません",
                    line_no, record_no
                ),
            ));
        }
        Err(msg) => return Err(row_error(ParseErrorType::InvalidJan, msg)),
    };

    // Step 6: 成功
    Ok(Some(ParsedRow {
        line_no,
        normalized_jan,
        name: name_raw.to_string(),
        quantity,
        amount,
    }))
}

/// 個数・金額の整数の読み取り（23 §13.4.1、SPEC-Z4A-D7）。
/// カンマなしの整数か、正しい3桁区切りのカンマ付きの整数だけを受理する。
fn parse_z004_int(raw: &str) -> Option<i32> {
    static INT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^-?(?:[0-9]+|[1-9][0-9]{0,2}(?:,[0-9]{3})+)$")
            .expect("数値パターンのコンパイル失敗")
    });
    let trimmed = raw.trim();
    if !INT_RE.is_match(trimmed) {
        return None;
    }
    trimmed.replace(',', "").parse().ok()
}

/// layout A のメタ行から精算の識別メタを抽出する（ラベルは前後の空白を除いて完全一致）。
/// 「ファイル」行が無い・空なら machine_no / settlement_no も None にする（帳票種別の無い番号だけの組を作らない）
fn extract_settlement_metadata(metadata: &[&str], settlement_date: &str) -> SettlementMetadata {
    let value_of = |label: &str| {
        metadata.iter().find_map(|line| {
            let fields = split_csv_fields(line);
            (fields.len() >= 2 && fields[0].trim() == label)
                .then(|| fields[1].trim().to_string())
                .filter(|value| !value.is_empty())
        })
    };
    let settled_at = value_of("時刻")
        .filter(|time| chrono::NaiveTime::parse_from_str(time, "%H:%M").is_ok() && time.len() == 5)
        .map(|time| format!("{settlement_date}T{time}"));
    let Some(report_kind) = value_of("ファイル") else {
        return SettlementMetadata {
            settled_at,
            ..SettlementMetadata::default()
        };
    };
    SettlementMetadata {
        machine_no: value_of("マシンNo."),
        report_kind: Some(report_kind),
        settlement_no: value_of("精算回数"),
        settled_at,
    }
}

/// 従来 shape の1行目から `YYYY-MM-DD` を抽出する。
fn extract_iso_date(line: &str) -> Option<String> {
    let date_re = regex::Regex::new(r"\d{4}-\d{2}-\d{2}").expect("日付パターンのコンパイル失敗");
    date_re
        .find(line)
        .map(|matched| matched.as_str().to_string())
}

/// layout A のメタ行から日付を抽出し、`YYYY-MM-DD` に正規化する。
fn extract_normalized_date(line: &str) -> Option<String> {
    let date_re = regex::Regex::new(
        r"(?x)
        (?P<year>\d{4})
        (?:
            -(?P<dash_month>\d{1,2})-(?P<dash_day>\d{1,2})
          | /(?P<slash_month>\d{1,2})/(?P<slash_day>\d{1,2})
        )",
    )
    .expect("日付パターンのコンパイル失敗");
    let captures = date_re.captures(line)?;
    let year = captures.name("year")?.as_str();
    let month = captures
        .name("dash_month")
        .or_else(|| captures.name("slash_month"))?
        .as_str()
        .parse::<u8>()
        .ok()?;
    let day = captures
        .name("dash_day")
        .or_else(|| captures.name("slash_day"))?
        .as_str()
        .parse::<u8>()
        .ok()?;
    Some(format!("{year}-{month:02}-{day:02}"))
}

/// 従来 shape の2行目に対する中間強度検査。
/// コード label は全角「コード」と半角カナ「ｺｰﾄﾞ」の両形を受理する
/// （実ファイルのヘッダ第2フィールドは半角カナ「ｽｷｬﾆﾝｸﾞｺｰﾄﾞ」。SPEC-Z4A-D1/D2）。
fn contains_code_label(field: &str) -> bool {
    field.contains("コード") || field.contains("ｺｰﾄﾞ")
}

fn is_conventional_header_line(line: &str) -> bool {
    let fields = split_csv_fields(line);
    fields.len() == 5 && contains_code_label(&fields[1])
}

/// layout A の5フィールド・位置アンカー付きヘッダ検査。
fn is_layout_a_header_line(line: &str) -> bool {
    let fields = split_csv_fields(line);
    fields.len() == 5 && contains_code_label(&fields[1]) && fields[4].contains("金額")
}

/// Z004のスキャニングコードをJANコード13桁に正規化する
///
/// 23-io-z004-parser.md セクション13.5
pub(crate) fn normalize_jan(raw: &str, line_no: usize) -> Result<Option<String>, String> {
    let trimmed = raw.trim();

    // 全桁ゼロ → Ok(None)（設計書13.5: 13桁/14桁ゼロが対象。売上の有無による
    // 分類は呼出し側の§13.4.2。他の桁数の全ゼロも同じくOk(None)で、0/0なら読み飛ばし、
    // 売上があれば商品コードの無い枠として InvalidJan になる）
    if !trimmed.is_empty() && trimmed.chars().all(|c| c == '0') {
        return Ok(None);
    }

    let mut chars: Vec<char> = trimmed.chars().collect();
    let len = chars.len();

    // 14桁 + 末尾ASCII英字 → 末尾除去で13桁化
    if len == 14 && chars[13].is_ascii_alphabetic() {
        chars.pop();
    }

    let normalized: String = chars.iter().collect();

    // 13桁 + 全数字
    if normalized.len() == 13 && normalized.chars().all(|c| c.is_ascii_digit()) {
        Ok(Some(normalized))
    } else {
        Err(format!(
            "行{}: JANコード '{}' を正規化できません",
            line_no, raw
        ))
    }
}

/// CSVフィールドをダブルクォート対応で分割する
///
/// 仕様: ダブルクォート囲み除去、内部カンマ保護、""→"エスケープ、囲みなし許容
fn split_csv_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    // "" → " エスケープ
                    current.push('"');
                    chars.next();
                } else {
                    // クォート終了
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

// ===========================================================================
// テスト
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// CP932エンコードされたテストデータを生成するヘルパー
    fn encode_cp932(text: &str) -> Vec<u8> {
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(text);
        encoded.to_vec()
    }

    /// 最小限の有効Z004データ（ヘッダ+1データ行）
    fn make_valid_z004(data_lines: &str) -> Vec<u8> {
        let text = format!(
            "精算日報 2026-03-21 テスト店舗\r\nNo,コード,名称,個数,金額\r\n{}",
            data_lines
        );
        encode_cp932(&text)
    }

    // -----------------------------------------------------------------------
    // 正常系
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_z004_req401_single_product() {
        // REQ-401: CSV取込み
        // 13.3: 正常パース（1商品）
        let raw = make_valid_z004("\"1\",\"4976383262108\",\"ﾊﾏﾅｶ ｱﾐｱﾐ極太\",3,1782");
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.settlement_date, "2026-03-21");
        assert_eq!(result.parsed_rows.len(), 1);
        assert_eq!(result.parse_errors.len(), 0);

        let row = &result.parsed_rows[0];
        assert_eq!(row.line_no, 3);
        assert_eq!(row.normalized_jan, "4976383262108");
        assert_eq!(row.name, "ﾊﾏﾅｶ ｱﾐｱﾐ極太");
        assert_eq!(row.quantity, 3);
        assert_eq!(row.amount, 1782);
    }

    #[test]
    fn test_parse_z004_req401_t25_conventional_shape_has_no_settlement_metadata() {
        // REQ-401 / IO-02 / T25: 従来 shape は識別メタを作らない（各項目 None、日付は従来どおり）
        let raw = make_valid_z004("\"1\",\"4976383262108\",\"テスト商品\",3,1782");
        let result = parse_z004(&raw).unwrap();
        assert_eq!(result.settlement_date, "2026-03-21");
        assert_eq!(result.settlement_metadata, None);
    }

    #[test]
    fn test_parse_z004_req401_multiple_products() {
        // REQ-401: CSV取込み
        // 13.3: 複数商品
        let raw = make_valid_z004(
            "\"1\",\"4976383262108\",\"商品A\",3,1782\r\n\"2\",\"4973167902615\",\"商品B\",1,385",
        );
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.parsed_rows.len(), 2);
        assert_eq!(result.parsed_rows[0].normalized_jan, "4976383262108");
        assert_eq!(result.parsed_rows[1].normalized_jan, "4973167902615");
        assert_eq!(result.total_data_lines, 2);
    }

    #[test]
    fn test_parse_z004_req401_settlement_date_extraction() {
        // REQ-401: CSV取込み
        // 13.3 Step 6: settlement_date抽出
        let raw = make_valid_z004("\"1\",\"4976383262108\",\"A\",1,100");
        let result = parse_z004(&raw).unwrap();
        assert_eq!(result.settlement_date, "2026-03-21");
    }

    #[test]
    fn test_parse_z004_req401_file_hash() {
        // REQ-401: CSV取込み
        // INV-6: file_hash = SHA-256(raw_bytes), hex小文字64文字
        let raw = make_valid_z004("\"1\",\"4976383262108\",\"A\",1,100");
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.file_hash.len(), 64);
        assert!(
            result.file_hash.chars().all(|c| c.is_ascii_hexdigit()),
            "hex文字のみ"
        );
        assert_eq!(
            result.file_hash,
            result.file_hash.to_lowercase(),
            "小文字のみ"
        );

        // 同じ入力 → 同じハッシュ
        let result2 = parse_z004(&raw).unwrap();
        assert_eq!(result.file_hash, result2.file_hash);
    }

    #[test]
    fn test_parse_z004_req401_negative_values_allowed() {
        // REQ-401: CSV取込み
        // 13.6: 返品値（quantity < 0, amount < 0）許可
        let raw = make_valid_z004("\"1\",\"4976383262108\",\"返品商品\",-1,-385");
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.parsed_rows.len(), 1);
        assert_eq!(result.parsed_rows[0].quantity, -1);
        assert_eq!(result.parsed_rows[0].amount, -385);
    }

    // -----------------------------------------------------------------------
    // 致命的エラー
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_z004_req401_decode_failed() {
        // REQ-401: CSV取込み
        // 13.3 Step 2: CP932デコード失敗
        // 0x80 の後に 0x00 が続くのはCP932マルチバイトとして不正
        let invalid_bytes: Vec<u8> = vec![0x80, 0x00, 0xFF];
        let result = parse_z004(&invalid_bytes);
        assert!(matches!(result, Err(Z004ParseError::DecodeFailed(_))));
    }

    #[test]
    fn test_parse_z004_req401_no_data_lines() {
        // REQ-401: CSV取込み
        // 13.3 Step 5: 2行未満
        let raw = encode_cp932("1行のみ");
        let result = parse_z004(&raw);
        assert!(matches!(result, Err(Z004ParseError::NoDataLines(_))));
    }

    #[test]
    fn test_parse_z004_req401_no_settlement_date() {
        // REQ-401: CSV取込み
        // 13.3 Step 6: 日付抽出不能
        let raw = encode_cp932("日付のない1行目\r\nヘッダ行\r\nデータ行");
        let result = parse_z004(&raw);
        assert!(matches!(result, Err(Z004ParseError::NoSettlementDate(_))));
    }

    // -----------------------------------------------------------------------
    // 行単位エラー
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_z004_req401_invalid_format() {
        // REQ-401: CSV取込み
        // 13.4: フィールド数不正
        let raw = make_valid_z004("\"1\",\"4976383262108\",\"商品A\"");
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.parsed_rows.len(), 0);
        assert_eq!(result.parse_errors.len(), 1);
        assert_eq!(
            result.parse_errors[0].error_type,
            ParseErrorType::InvalidFormat
        );
        assert_eq!(result.parse_errors[0].line_no, 3);
    }

    #[test]
    fn test_parse_z004_req401_invalid_number() {
        // REQ-401: CSV取込み
        // 13.4: 数量が数値でない
        let raw = make_valid_z004("\"1\",\"4976383262108\",\"商品A\",abc,100");
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.parse_errors.len(), 1);
        assert_eq!(
            result.parse_errors[0].error_type,
            ParseErrorType::InvalidNumber
        );
        assert!(result.parse_errors[0].error_message.contains("数量"));
    }

    #[test]
    fn test_parse_z004_req401_line_no_preserved() {
        // REQ-401: CSV取込み
        // parse_errors の line_no が正しい行番号であること
        let raw = make_valid_z004(
            "\"1\",\"4976383262108\",\"正常\",1,100\r\n\"2\",\"INVALID\",\"エラー\",1,100",
        );
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.parsed_rows.len(), 1);
        assert_eq!(result.parse_errors.len(), 1);
        assert_eq!(result.parsed_rows[0].line_no, 3);
        assert_eq!(result.parse_errors[0].line_no, 4);
    }

    #[test]
    fn test_parse_z004_req401_comma_grouped_numbers() {
        // REQ-401 / SPEC-Z4A-D7: 3桁区切りのカンマ付きの個数・金額を受理する（23 §13.4.1）
        let raw = make_valid_z004(
            "\"1\",\"4976383262108\",\"A\",\"1\",\"1,234\"\r\n\
             \"2\",\"4976383262115\",\"B\",\"-1\",\"-12,345\"\r\n\
             \"3\",\"4976383262122\",\"C\",\"1,000\",\"1,000,000\"",
        );
        let result = parse_z004(&raw).unwrap();

        assert!(result.parse_errors.is_empty(), "{:?}", result.parse_errors);
        let values: Vec<(i32, i32)> = result
            .parsed_rows
            .iter()
            .map(|r| (r.quantity, r.amount))
            .collect();
        assert_eq!(values, vec![(1, 1234), (-1, -12345), (1000, 1_000_000)]);
    }

    #[test]
    fn test_parse_z004_req401_malformed_comma_invalid_number() {
        // REQ-401 / SPEC-Z4A-D7 / SPEC-Z4A-D8: 区切りの崩れ・小数・符号+・範囲外は InvalidNumber
        let cases = [
            ("4976383262108", "1", "1,23"),
            ("4976383262108", "1", "12,3456"),
            ("4976383262108", "1", "1,,234"),
            ("4976383262108", "1", ",123"),
            ("4976383262108", "1", "1,234,"),
            ("4976383262108", "1", "0,123"),
            ("4976383262108", "1", "1.5"),
            ("4976383262108", "1", "+5"),
            ("4976383262108", "1", "2,147,483,648"),
            ("4976383262108", "1,2", "100"),
            ("00000000000000", "1.5", "500"),
            ("00000000000000", "0", "1,23"),
        ];
        let lines: Vec<String> = cases
            .iter()
            .enumerate()
            .map(|(i, (code, q, a))| format!("\"{}\",\"{code}\",\"X\",\"{q}\",\"{a}\"", i + 1))
            .collect();
        let result = parse_z004(&make_valid_z004(&lines.join("\r\n"))).unwrap();

        assert!(result.parsed_rows.is_empty());
        assert_eq!(result.parse_errors.len(), cases.len());
        for (err, (_, q, a)) in result.parse_errors.iter().zip(cases.iter()) {
            assert_eq!(err.error_type, ParseErrorType::InvalidNumber, "{q} / {a}");
            assert_eq!(err.raw_quantity.as_deref(), Some(*q));
            assert_eq!(err.raw_amount.as_deref(), Some(*a));
        }
    }

    #[test]
    fn test_parse_z004_req401_zero_code_with_sales_is_invalid_jan() {
        // REQ-401 / SPEC-Z4A-D8 / SPEC-Z4A-D6: コード全桁0で売上のある行は InvalidJan（D-103）
        let cases = [
            ("", "1", "500"),
            ("", "0", "500"),
            ("", "-1", "0"),
            ("PLU0001", "1", "300"),
        ];
        let mut lines = Vec::new();
        let mut expected = Vec::new();
        for code in ["00000000000000", "0000000000000"] {
            for (name, q, a) in cases {
                let record_no = format!("{}", 41 + expected.len());
                lines.push(format!(
                    "\"{record_no}\",\"{code}\",\"{name}\",\"{q}\",\"{a}\""
                ));
                expected.push((record_no, q, a));
            }
        }
        let result = parse_z004(&make_valid_z004(&lines.join("\r\n"))).unwrap();

        assert!(result.parsed_rows.is_empty());
        assert_eq!(result.parse_errors.len(), expected.len());
        for (err, (record_no, q, a)) in result.parse_errors.iter().zip(expected.iter()) {
            assert_eq!(err.error_type, ParseErrorType::InvalidJan);
            assert!(err.error_message.contains("商品コードの無い枠"));
            assert!(
                err.error_message.contains(&format!("メモリNo.{record_no}")),
                "{}",
                err.error_message
            );
            assert_eq!(err.raw_quantity.as_deref(), Some(*q));
            assert_eq!(err.raw_amount.as_deref(), Some(*a));
        }
    }

    // -----------------------------------------------------------------------
    // JAN正規化
    // -----------------------------------------------------------------------

    #[test]
    fn test_normalize_jan_req401_13_digits() {
        // REQ-401: CSV取込み
        assert_eq!(
            normalize_jan("4976383262108", 1).unwrap(),
            Some("4976383262108".to_string())
        );
    }

    #[test]
    fn test_normalize_jan_req401_14_with_letter_suffix() {
        // REQ-401: CSV取込み
        // 14桁 + 末尾E → 末尾除去で13桁化
        assert_eq!(
            normalize_jan("4976383262108E", 1).unwrap(),
            Some("4976383262108".to_string())
        );
    }

    #[test]
    fn test_normalize_jan_req401_all_zeros_13() {
        // REQ-401: CSV取込み
        assert_eq!(normalize_jan("0000000000000", 1).unwrap(), None);
    }

    #[test]
    fn test_normalize_jan_req401_all_zeros_14() {
        // REQ-401: CSV取込み
        assert_eq!(normalize_jan("00000000000000", 1).unwrap(), None);
    }

    #[test]
    fn test_normalize_jan_req401_12_digits_error() {
        // REQ-401: CSV取込み
        assert!(normalize_jan("497638326210", 1).is_err());
    }

    #[test]
    fn test_normalize_jan_req401_14_digits_no_letter_error() {
        // REQ-401: CSV取込み
        // 14桁末尾が数字 → 不正
        assert!(normalize_jan("49763832621089", 1).is_err());
    }

    #[test]
    fn test_normalize_jan_req401_non_numeric_error() {
        // REQ-401: CSV取込み
        assert!(normalize_jan("ABCDEFGHIJKLM", 1).is_err());
    }

    // -----------------------------------------------------------------------
    // 空行・空スロット
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_z004_req401_empty_lines_skipped() {
        // REQ-401: CSV取込み
        // 空行はスキップ＋total_data_linesに含まれない
        let raw = make_valid_z004(
            "\"1\",\"4976383262108\",\"A\",1,100\r\n\r\n\"2\",\"4973167902615\",\"B\",2,200",
        );
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.parsed_rows.len(), 2);
        assert_eq!(result.total_data_lines, 2, "空行はカウントしない");
    }

    #[test]
    fn test_parse_z004_req401_all_zero_jan_skipped() {
        // REQ-401: CSV取込み
        // 全桁ゼロJAN → Ok(None) = 空スロット
        let raw = make_valid_z004("\"1\",\"00000000000000\",\"\",0,0");
        let result = parse_z004(&raw).unwrap();

        assert_eq!(
            result.parsed_rows.len(),
            0,
            "空スロットは parsed_rows に入らない"
        );
        assert_eq!(result.parse_errors.len(), 0, "エラーにもならない");
        assert_eq!(result.total_data_lines, 1, "パース試行としてカウントされる");
    }

    // -----------------------------------------------------------------------
    // CSVダブルクォート処理
    // -----------------------------------------------------------------------

    #[test]
    fn test_csv_req401_quoted_fields() {
        // REQ-401: CSV取込み
        // クォート囲みフィールド → 外側クォート除去
        let fields = split_csv_fields("\"A\",\"B\",\"C\"");
        assert_eq!(fields, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_csv_req401_comma_inside_quotes() {
        // REQ-401: CSV取込み
        // クォート内カンマ → 区切りとして扱わない
        let fields = split_csv_fields("\"A,B\",C");
        assert_eq!(fields, vec!["A,B", "C"]);
    }

    #[test]
    fn test_csv_req401_escaped_quotes() {
        // REQ-401: CSV取込み
        // "" → "
        let fields = split_csv_fields("\"A\"\"B\",C");
        assert_eq!(fields, vec!["A\"B", "C"]);
    }

    #[test]
    fn test_csv_req401_mixed_quoted_unquoted() {
        // REQ-401: CSV取込み
        // 囲みなしフィールドとの混在
        let fields = split_csv_fields("\"1\",\"4976383262108\",商品名,3,1782");
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0], "1");
        assert_eq!(fields[1], "4976383262108");
        assert_eq!(fields[2], "商品名");
        assert_eq!(fields[3], "3");
        assert_eq!(fields[4], "1782");
    }
}

#[cfg(test)]
mod layout_a_tests {
    use super::*;

    // 実ファイル形状（2026-08-17 機械抽出）: CP932 12byte 固定幅 padding、第2フィールドは半角カナ
    const LAYOUT_A_HEADER: &str =
        "\"レコード    \",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ \",\"キャラクター\",\"個数        \",\"金額        \"";
    // 全角「コード」形（旧 synthetic 形）。アンカーの全角受理を独立に拘束するために保持
    const LAYOUT_A_HEADER_FULLWIDTH: &str = "\"メモリNo.\",\"コード\",\"名称\",\"個数\",\"金額\"";

    fn encode_cp932(text: &str) -> Vec<u8> {
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(text);
        encoded.to_vec()
    }

    fn layout_a_text(metadata: &[&str], data_lines: &[&str]) -> String {
        let mut lines = metadata.to_vec();
        lines.push(LAYOUT_A_HEADER);
        lines.extend_from_slice(data_lines);
        lines.join("\r\n")
    }

    fn synthetic_layout_a_fixture() -> Vec<u8> {
        // 実ファイル形状 exact（メタ6行の実ラベル + 7行目空行 + 8行目ヘッダ。値は synthetic）
        encode_cp932(&layout_a_text(
            &[
                "\"マシンNo.   \",\"0001\"",
                "\"ファイル    \",\"Z004_SYNTH\"",
                "\"モード      \",\"SYNTH\"",
                "\"精算回数    \",\"0042\"",
                "\"日付        \",\"2026-08-15\"",
                "\"時刻        \",\"18:30\"",
                "",
            ],
            &[
                "\"1\",\"9999999999990E\",\"合成商品A\",\"2\",\"600\"",
                "\"2\",\"8888888888880E\",\"合成返品B\",\"-1\",\"-250\"",
                "\"3\",\"12345678EEEEEE\",\"合成独自C\",\"1\",\"100\"",
                "\"4\",\"0000000000000\",\"空スロット13\",\"0\",\"0\"",
                "\"5\",\"00000000000000\",\"空スロット14\",\"0\",\"0\"",
            ],
        ))
    }

    fn assert_no_settlement_date(error: Z004ParseError, expected_message: &str) {
        match error {
            Z004ParseError::NoSettlementDate(message) => {
                assert_eq!(message, expected_message);
            }
            other => panic!("NoSettlementDate を期待しました: {other:?}"),
        }
    }

    #[test]
    fn test_parse_z004_req401_layout_a_full_shape() {
        // REQ-401 / SPEC-Z4A-D1/D2/D3/D4: layout A の完全形状と出力契約
        let raw = synthetic_layout_a_fixture();
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.settlement_date, "2026-08-15");
        assert_eq!(result.parsed_rows.len(), 2);
        assert_eq!(result.parse_errors.len(), 1);
        assert_eq!(result.total_data_lines, 5);
        assert_eq!(
            result.file_hash,
            "fc73326f99ac4c2d15823460f86f97d20674d49ba45ae52c0ebf318444e36fe8"
        );

        let sale = &result.parsed_rows[0];
        assert_eq!(sale.line_no, 9);
        assert_eq!(sale.normalized_jan, "9999999999990");
        assert_eq!(sale.name, "合成商品A");
        assert_eq!(sale.quantity, 2);
        assert_eq!(sale.amount, 600);

        let returned = &result.parsed_rows[1];
        assert_eq!(returned.line_no, 10);
        assert_eq!(returned.normalized_jan, "8888888888880");
        assert_eq!(returned.name, "合成返品B");
        assert_eq!(returned.quantity, -1);
        assert_eq!(returned.amount, -250);
    }

    #[test]
    fn test_parse_z004_req401_layout_a_settlement_date_iso() {
        // REQ-401 / SPEC-Z4A-D3: ISO 形式の日付ラベル値を採用する
        let result = parse_z004(&synthetic_layout_a_fixture()).unwrap();
        assert_eq!(result.settlement_date, "2026-08-15");
    }

    #[test]
    fn test_parse_z004_req401_layout_a_settlement_date_slash_padded() {
        // REQ-401 / SPEC-Z4A-D3: YYYY/M/D をゼロ埋めする
        let raw = encode_cp932(&layout_a_text(
            &["\"管理No.\",\"SYNTH-0002\"", "\"日付\",\"2026/8/5\""],
            &["\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\""],
        ));

        let result = parse_z004(&raw).unwrap();
        assert_eq!(result.settlement_date, "2026-08-05");
    }

    #[test]
    fn test_parse_z004_req401_layout_a_8digit_code_invalid_jan() {
        // REQ-401 / SPEC-Z4A-D5: 8桁 + E パディングは行単位エラーで可視化する
        let result = parse_z004(&synthetic_layout_a_fixture()).unwrap();

        assert_eq!(result.parsed_rows.len(), 2, "他の正常行は処理を継続する");
        assert_eq!(result.parse_errors.len(), 1);
        assert_eq!(result.parse_errors[0].line_no, 11);
        assert_eq!(
            result.parse_errors[0].error_type,
            ParseErrorType::InvalidJan
        );
        assert_eq!(
            result.parse_errors[0].raw_name.as_deref(),
            Some("合成独自C")
        );
    }

    #[test]
    fn test_parse_z004_req401_layout_a_meta_line_count_tolerance() {
        // REQ-401 / SPEC-Z4A-D2: メタ行数を固定値にしない
        for metadata in [
            vec![
                "\"管理No.\",\"SYNTH-5\"",
                "\"ファイル\",\"Z004\"",
                "\"帳票\",\"PLU別売上\"",
                "\"日付\",\"2026-08-15\"",
                "\"時刻\",\"18:30\"",
            ],
            vec![
                "\"管理No.\",\"SYNTH-7\"",
                "\"ファイル\",\"Z004\"",
                "\"帳票\",\"PLU別売上\"",
                "\"番号\",\"42\"",
                "\"日付\",\"2026-08-15\"",
                "\"時刻\",\"18:30\"",
                "\"予備\",\"synthetic\"",
            ],
        ] {
            let raw = encode_cp932(&layout_a_text(
                &metadata,
                &["\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\""],
            ));
            let result = parse_z004(&raw).unwrap();
            assert_eq!(result.parsed_rows.len(), 1);
        }
    }

    #[test]
    fn test_parse_z004_req401_layout_a_fullwidth_header_variant() {
        // REQ-401 / SPEC-Z4A-D2: 全角「コード」ヘッダも受理する（アンカーの全角側を独立拘束）
        let raw = encode_cp932(&format!(
            "{}\r\n{}\r\n{}",
            "\"日付\",\"2026-08-15\"",
            LAYOUT_A_HEADER_FULLWIDTH,
            "\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\""
        ));
        let result = parse_z004(&raw).unwrap();
        assert_eq!(result.settlement_date, "2026-08-15");
        assert_eq!(result.parsed_rows.len(), 1);
    }

    #[test]
    fn test_parse_z004_req401_layout_a_slot_dump_counts() {
        // REQ-401 / SPEC-Z4A-D6: 全スロットダンプと全ゼロ skip
        let mut data_lines = vec![
            "\"1\",\"9999999999990E\",\"合成商品A\",\"3\",\"900\"".to_string(),
            "\"2\",\"8888888888880E\",\"合成商品B\",\"1\",\"250\"".to_string(),
        ];
        data_lines.extend(
            (3..=5_000).map(|slot| format!("\"{slot}\",\"00000000000000\",\"\",\"0\",\"0\"")),
        );
        let data_refs: Vec<&str> = data_lines.iter().map(String::as_str).collect();
        let raw = encode_cp932(&layout_a_text(
            &["\"管理No.\",\"SYNTH-5000\"", "\"日付\",\"2026-08-15\""],
            &data_refs,
        ));

        let result = parse_z004(&raw).unwrap();
        assert_eq!(result.total_data_lines, 5_000);
        assert_eq!(result.parsed_rows.len(), 2);
        assert_eq!(result.parse_errors.len(), 0);
        assert_eq!(result.parsed_rows[0].quantity, 3);
        assert_eq!(result.parsed_rows[1].amount, 250);
    }

    #[test]
    fn test_parse_z004_req401_non_jan_code_without_sales_skipped() {
        // REQ-401 / SPEC-Z4A-D8 / SPEC-Z4A-D5 / SPEC-Z4A-D4: 非JANの0/0は読み飛ばし、売上ありはnormalize_janの文言
        let raw = encode_cp932(&layout_a_text(
            &["\"日付\",\"2026-08-15\""],
            &[
                "\"1\",\"9999999999990E\",\"合成商品A\",\"2\",\"600\"",
                "\"2\",\"12345678EEEEEE\",\"合成独自0\",\"0\",\"0\"",
                "\"3\",\"INVALID\",\"合成不正\",\"0\",\"0\"",
                "\"4\",\"12345678EEEEEE\",\"合成独自C\",\"1\",\"100\"",
            ],
        ));
        let result = parse_z004(&raw).unwrap();

        assert_eq!(result.total_data_lines, 4);
        assert_eq!(result.parsed_rows.len(), 1);
        assert_eq!(result.parse_errors.len(), 1);
        let err = &result.parse_errors[0];
        assert_eq!(err.line_no, 6);
        assert_eq!(err.error_type, ParseErrorType::InvalidJan);
        assert!(err.error_message.contains("正規化できません"));
        assert!(!err.error_message.contains("商品コードの無い枠"));
    }

    #[test]
    fn test_parse_z004_req401_valid_jan_zero_row_kept() {
        // REQ-401 / SPEC-Z4A-D8: 13桁JANの0/0はIOで捨てずParsedRowで返す（除外はBIZ-03）
        let raw = encode_cp932(&layout_a_text(
            &["\"日付\",\"2026-08-15\""],
            &["\"1\",\"9999999999990E\",\"合成商品A\",\"0\",\"0\""],
        ));
        let result = parse_z004(&raw).unwrap();

        assert!(result.parse_errors.is_empty());
        assert_eq!(result.parsed_rows.len(), 1);
        assert_eq!(result.parsed_rows[0].normalized_jan, "9999999999990");
        assert_eq!(result.parsed_rows[0].quantity, 0);
        assert_eq!(result.parsed_rows[0].amount, 0);
    }

    #[test]
    fn test_parse_z004_req401_layout_a_datelike_meta_first_line() {
        let raw = encode_cp932(&layout_a_text(
            &[
                "\"管理No.\",\"RUN-2026-01-02\"",
                "\"コード\",\"decoy\",\"金額\",\"x\",\"not-anchor\"",
                "\"帳票\",\"PLU別売上\"",
                "\"日付\",\"2026/8/5\"",
                "\"時刻\",\"18:30\"",
            ],
            &["\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\""],
        ));

        let result = parse_z004(&raw).unwrap();
        // SPEC-Z4A-D1: 日付様の先頭メタ値でも従来 shape へ誤ルーティングしない。
        assert_eq!(result.parsed_rows.len(), 1);
        // SPEC-Z4A-D3: 最初の一致ではなく「日付」ラベル行を優先する。
        assert_eq!(result.settlement_date, "2026-08-05");
        // SPEC-Z4A-D2: 誤位置のラベルを持つ decoy をヘッダと認識しない。
        assert_eq!(result.parsed_rows[0].line_no, 7);
        assert_eq!(result.total_data_lines, 1);
    }

    #[test]
    fn test_parse_z004_req401_layout_a_five_field_meta_decoy() {
        // REQ-401 / SPEC-Z4A-D1: 5フィールドだけの2行目を従来 shape と誤認しない
        let raw = encode_cp932(&layout_a_text(
            &[
                "\"管理No.\",\"RUN-2026-01-02\"",
                "\"decoy\",\"not-code\",\"x\",\"y\",\"金額\"",
                "\"日付\",\"2026-08-15\"",
            ],
            &["\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\""],
        ));

        let result = parse_z004(&raw).unwrap();
        assert_eq!(result.settlement_date, "2026-08-15");
        assert_eq!(result.parsed_rows.len(), 1);
        assert_eq!(result.parsed_rows[0].line_no, 5);
        assert_eq!(result.total_data_lines, 1);
    }

    #[test]
    fn test_parse_z004_req401_layout_a_no_date_fails() {
        // REQ-401 / SPEC-Z4A-D3: ヘッダがあっても日付なしは安全停止する
        let raw = encode_cp932(&layout_a_text(
            &["\"管理No.\",\"SYNTH-NO-DATE\"", "\"時刻\",\"18:30\""],
            &["\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\""],
        ));

        assert_no_settlement_date(
            parse_z004(&raw).unwrap_err(),
            "精算日を抽出できません。ファイル形式を確認してください",
        );
    }

    #[test]
    fn test_parse_z004_req401_layout_a_no_header_fails() {
        // REQ-401 / SPEC-Z4A-D2: 20行目は受理、21行目は走査上限超過で停止する
        let preamble_19: Vec<String> = (1..=19)
            .map(|line| {
                if line == 2 {
                    "\"日付\",\"2026-08-15\"".to_string()
                } else {
                    format!("\"メタ{line}\",\"synthetic\"")
                }
            })
            .collect();
        let mut accepted_lines = preamble_19.clone();
        accepted_lines.push(LAYOUT_A_HEADER.to_string());
        accepted_lines.push("\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\"".to_string());
        let accepted = encode_cp932(&accepted_lines.join("\r\n"));
        assert_eq!(parse_z004(&accepted).unwrap().parsed_rows.len(), 1);

        let mut preamble_20 = preamble_19;
        preamble_20.push("\"メタ20\",\"synthetic\"".to_string());
        preamble_20.push(LAYOUT_A_HEADER.to_string());
        let rejected = encode_cp932(&preamble_20.join("\r\n"));
        assert_no_settlement_date(
            parse_z004(&rejected).unwrap_err(),
            "ヘッダ行を検出できません。ファイル形式を確認してください",
        );
    }

    #[test]
    fn test_parse_z004_req401_unrecognized_shape_fails() {
        // REQ-401 / SPEC-Z4A-D1: 二形状外は部分結果を返さない
        let raw =
            encode_cp932("\"メタ\",\"synthetic\"\r\n\"別メタ\",\"value\"\r\n\"終端\",\"value\"");
        assert_no_settlement_date(
            parse_z004(&raw).unwrap_err(),
            "ヘッダ行を検出できません。ファイル形式を確認してください",
        );
    }

    fn metadata_of(metadata: &[&str]) -> SettlementMetadata {
        let raw = encode_cp932(&layout_a_text(
            metadata,
            &["\"1\",\"9999999999990E\",\"合成商品\",\"1\",\"100\""],
        ));
        parse_z004(&raw).unwrap().settlement_metadata.unwrap()
    }

    #[test]
    fn test_parse_z004_req401_t24_layout_a_settlement_metadata() {
        // REQ-401 / IO-02 / SPEC-STK-TIME-D3 / T24: 識別メタを文字列のまま（先頭の 0 を保って）返す
        let raw = synthetic_layout_a_fixture();
        let result = parse_z004(&raw).unwrap();
        assert_eq!(result.settlement_date, "2026-08-15");
        assert_eq!(
            result.settlement_metadata,
            Some(SettlementMetadata {
                machine_no: Some("0001".to_string()),
                report_kind: Some("Z004_SYNTH".to_string()),
                settlement_no: Some("0042".to_string()),
                settled_at: Some("2026-08-15T18:30".to_string()),
            })
        );

        // 時刻行の値が壊れている → settled_at だけ None（日付・他のメタは返る）
        for broken in ["18:3", "25:00", "18:30:00", "", "1:05"] {
            let time_line = format!("\"時刻        \",\"{broken}\"");
            let metadata = metadata_of(&[
                "\"マシンNo.   \",\"0001\"",
                "\"ファイル    \",\"Z004_SYNTH\"",
                "\"モード      \",\"SYNTH\"",
                "\"精算回数    \",\"0042\"",
                "\"日付        \",\"2026-08-15\"",
                &time_line,
            ]);
            assert_eq!(metadata.settled_at, None, "{broken}");
            assert_eq!(metadata.machine_no.as_deref(), Some("0001"));
            assert_eq!(metadata.settlement_no.as_deref(), Some("0042"));
        }

        // 「ファイル」行だけが無い → report_kind も番号も None（番号だけの組を作らない）
        let metadata = metadata_of(&[
            "\"マシンNo.   \",\"0001\"",
            "\"モード      \",\"SYNTH\"",
            "\"精算回数    \",\"0042\"",
            "\"日付        \",\"2026-08-15\"",
            "\"時刻        \",\"18:30\"",
        ]);
        assert_eq!(metadata.report_kind, None);
        assert_eq!(metadata.machine_no, None);
        assert_eq!(metadata.settlement_no, None);
        assert_eq!(metadata.settled_at.as_deref(), Some("2026-08-15T18:30"));

        // report_kind は「モード」行から作らない
        let metadata = metadata_of(&[
            "\"モード      \",\"SYNTH\"",
            "\"日付        \",\"2026-08-15\"",
        ]);
        assert_eq!(metadata, SettlementMetadata::default());
    }
}

#[cfg(test)]
mod plu_register_snapshot_tests {
    use super::*;

    fn snapshot_bytes(row_count: usize, override_row: Option<(usize, &str)>) -> Vec<u8> {
        let mut text = String::from(
            "\"meta\",\"synthetic\"\r\n\"メモリNo.\",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ\",\"名称\",\"個数\",\"金額\"\r\n",
        );
        for memory_no in 1..=row_count {
            let code = override_row
                .filter(|(target, _)| *target == memory_no)
                .map_or("00000000000000", |(_, code)| code);
            text.push_str(&format!("{memory_no},{code},,0,0\r\n"));
        }
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode(&text);
        bytes.into_owned()
    }

    fn snapshot_bytes_with_memory_numbers(
        memory_numbers: impl IntoIterator<Item = usize>,
    ) -> Vec<u8> {
        let mut text = String::from(
            "\"meta\",\"synthetic\"\r\n\"メモリNo.\",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ\",\"名称\",\"個数\",\"金額\"\r\n",
        );
        for memory_no in memory_numbers {
            text.push_str(&format!("{memory_no},00000000000000,,0,0\r\n"));
        }
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode(&text);
        bytes.into_owned()
    }

    #[test]
    fn test_parse_plu_register_snapshot_req907_normalizes_only_register_padding() {
        // REQ-907: A-N2 / A-N2c
        let mut text = String::from(
            "\"meta\",\"synthetic\"\r\n\"メモリNo.\",\"ｽｷｬﾆﾝｸﾞｺｰﾄﾞ\",\"名称\",\"個数\",\"金額\"\r\n",
        );
        for memory_no in 1..=5_000 {
            let code = match memory_no {
                1 => "4901234567894E",
                2 => "12345678EEEEEE",
                3 => "123456789012EE",
                _ => "00000000000000",
            };
            text.push_str(&format!("{memory_no},{code},,0,0\r\n"));
        }
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode(&text);
        let slots = parse_plu_register_snapshot(&bytes).unwrap();
        assert_eq!(slots.len(), 5_000);
        assert_eq!(slots[0].raw_code.as_deref(), Some("4901234567894"));
        assert_eq!(slots[1].raw_code.as_deref(), Some("12345678EEEEEE"));
        assert_eq!(slots[2].raw_code.as_deref(), Some("123456789012EE"));
        assert_eq!(slots[3].raw_code, None);
    }

    #[test]
    fn test_parse_plu_register_snapshot_req907_rejects_incomplete_snapshot() {
        // REQ-907: A-N1b
        for row_count in [4_999, 5_001] {
            let error = parse_plu_register_snapshot(&snapshot_bytes(row_count, None)).unwrap_err();
            assert!(
                matches!(error, Z004ParseError::ImportError(ref message) if message.contains("PLUスロット行数"))
            );
        }
        assert!(matches!(
            parse_plu_register_snapshot(b"meta\r\nwrong-header\r\n"),
            Err(Z004ParseError::ImportError(_))
        ));
        let duplicate = (1..5_000).chain(std::iter::once(4_999));
        assert!(matches!(
            parse_plu_register_snapshot(&snapshot_bytes_with_memory_numbers(duplicate)),
            Err(Z004ParseError::ImportError(_))
        ));
        assert!(matches!(
            parse_plu_register_snapshot(&snapshot_bytes_with_memory_numbers(0..5_000)),
            Err(Z004ParseError::ImportError(_))
        ));
    }
}
