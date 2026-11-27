//! IO-04: PLUフォーマッター
//!
//! カシオレジスターツール（CV17）のスキャニングPLUインポート形式（タブ区切り .txt、CP932）を生成する。
//! 純関数。DB非依存。
//!
//! docs/function-design/25-io-plu-formatter.md に基づく実装。

use std::fmt;

use crate::constants::{PLU_EXPORT_LIMIT, SCANNING_PLU_MEMORY_START};

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// BIZ-04 から渡される行データ
///
/// IO層に定義し、BIZ層がインポートして使う（BIZ → IO 方向の型参照は許可）
#[derive(Debug, Clone)]
pub struct PluExportRow {
    pub memory_no: i64,
    pub product_code: String,
    pub jan_code: Option<String>,
    pub name: String,
    pub selling_price: i64,
    pub tax_rate: String,
    pub department_name: String,
}

/// PLUファイル生成結果
#[derive(Debug)]
pub struct PluFileOutput {
    /// CP932エンコード済みPLUファイルバイト列
    pub bytes: Vec<u8>,
    /// 推奨ファイル名（例: "PLU_20260408.txt"）
    pub suggested_filename: String,
    /// MIMEタイプ
    pub content_type: &'static str,
    /// 文字エンコーディング名
    pub encoding: &'static str,
}

/// PLUフォーマット処理のエラー型
#[derive(Debug)]
pub enum PluFormatError {
    /// CP932エンコード不能文字
    EncodingError {
        product_code: String,
        char: char,
        message: String,
    },
    /// スキャニングコード（JAN）の不正
    InvalidScanningCode {
        product_code: String,
        message: String,
    },
    /// 税区分マッピング不正
    TaxMappingError {
        product_code: String,
        tax_rate: String,
        message: String,
    },
    InvalidMemoryNo {
        memory_no: i64,
        message: String,
    },
}

impl fmt::Display for PluFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluFormatError::EncodingError { message, .. } => write!(f, "{}", message),
            PluFormatError::InvalidScanningCode { message, .. } => write!(f, "{}", message),
            PluFormatError::TaxMappingError { message, .. } => write!(f, "{}", message),
            PluFormatError::InvalidMemoryNo { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for PluFormatError {}

// ---------------------------------------------------------------------------
// 定数
// ---------------------------------------------------------------------------

/// PLUファイルヘッダ行（CV17 1.1.1 スキャニングPLU 11列、タブ区切り）
const TSV_HEADER: &str =
    "メモリNo.\tｽｷｬﾆﾝｸﾞｺｰﾄﾞ\t名称\t単価\t課税方式\t単品売り\t負単価\t品番PLU\tゼロ単価\t入力桁制限\t部門リンク";

/// 商品名の最大バイト数（CP932基準）
const NAME_MAX_BYTES: usize = 16;

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// PluExportRow リストからカシオレジスターツール用PLUファイルを生成する
///
/// 25-io-plu-formatter.md セクション12.3
pub fn generate_plu_tsv(rows: &[PluExportRow]) -> Result<PluFileOutput, PluFormatError> {
    let date = chrono::Local::now().format("%Y%m%d").to_string();
    generate_plu_tsv_with_date(rows, &date)
}

/// 日付を外部から注入可能なバージョン（テスト用内部関数）
///
/// 25-io-plu-formatter.md セクション12.6
pub(crate) fn generate_plu_tsv_with_date(
    rows: &[PluExportRow],
    date_str: &str,
) -> Result<PluFileOutput, PluFormatError> {
    let mut output = Vec::new();

    // ヘッダ行（CP932エンコード）
    let (header_bytes, _, _) = encoding_rs::SHIFT_JIS.encode(TSV_HEADER);
    output.extend_from_slice(&header_bytes);
    output.extend_from_slice(b"\r\n");

    // データ行
    for row in rows {
        output.extend_from_slice(&format_plu_row(row)?);
        output.extend_from_slice(b"\r\n");
    }

    Ok(PluFileOutput {
        bytes: output,
        suggested_filename: format!("PLU_{}.txt", date_str),
        content_type: "text/tab-separated-values",
        encoding: "CP932",
    })
}

/// 1 行の全 11 列（メモリNo.〜部門リンク）の CP932 の byte 列。行末の CRLF は含まない。
/// file に書く行と、BIZ-04 の confirm の照合の値（33 §16.5）の入力は、この関数の 1 つの出どころから作る
pub(crate) fn format_plu_row(row: &PluExportRow) -> Result<Vec<u8>, PluFormatError> {
    let mut output = Vec::new();
    if row.memory_no < SCANNING_PLU_MEMORY_START as i64 || row.memory_no > PLU_EXPORT_LIMIT as i64 {
        return Err(PluFormatError::InvalidMemoryNo {
            memory_no: row.memory_no,
            message: format!("PLUメモリNo.が範囲外です: {}", row.memory_no),
        });
    }
    let name_bytes = process_product_name(&row.name, &row.product_code)?;
    let tax = map_tax_rate(&row.tax_rate, &row.product_code)?;
    let scanning_code = validate_scanning_code(row)?;

    // メモリNo. + タブ + スキャニングコード + タブ
    let prefix = format!("{:06}\t{}\t", row.memory_no, scanning_code);
    let (prefix_bytes, _, had_replacements) = encoding_rs::SHIFT_JIS.encode(&prefix);
    if had_replacements {
        let bad_char = find_unmappable_char(scanning_code).unwrap_or('?');
        return Err(PluFormatError::EncodingError {
            product_code: row.product_code.clone(),
            char: bad_char,
            message: format!(
                "商品 {} のスキャニングコードにCP932非対応文字 '{}' が含まれています",
                row.product_code, bad_char
            ),
        });
    }
    output.extend_from_slice(&prefix_bytes);

    // 名称（CP932、16バイトパディング済み）
    output.extend_from_slice(&name_bytes);

    // タブ + 単価 + タブ + 課税方式 + 固定列 + 部門リンク
    let suffix = format!(
        "\t{}\t{}\tいいえ\tいいえ\tいいえ\tいいえ\t無し\t{}",
        row.selling_price, tax, row.department_name
    );
    let (suffix_bytes, _, had_replacements) = encoding_rs::SHIFT_JIS.encode(&suffix);
    if had_replacements {
        // 固定文字列部分はCP932安全なので、department_name が原因
        let bad_char = find_unmappable_char(&row.department_name).unwrap_or('?');
        return Err(PluFormatError::EncodingError {
            product_code: row.product_code.clone(),
            char: bad_char,
            message: format!(
                "商品 {} の部門名にCP932非対応文字 '{}' が含まれています",
                row.product_code, bad_char
            ),
        });
    }
    output.extend_from_slice(&suffix_bytes);
    Ok(output)
}

fn validate_scanning_code(row: &PluExportRow) -> Result<&str, PluFormatError> {
    let Some(jan_code) = row.jan_code.as_deref() else {
        return Err(scanning_code_error(&row.product_code));
    };
    if !is_valid_ean13_code(jan_code) {
        return Err(scanning_code_error(&row.product_code));
    }
    Ok(jan_code)
}

fn scanning_code_error(product_code: &str) -> PluFormatError {
    PluFormatError::InvalidScanningCode {
        product_code: product_code.to_string(),
        message: format!(
            "商品 {} のJANコードはスキャニングPLU書出しに使えません。商品マスタで13桁JANを確認してください",
            product_code
        ),
    }
}

pub(crate) fn is_valid_ean13_code(value: &str) -> bool {
    if value.len() != 13 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let digits: Vec<u32> = value.bytes().map(|b| u32::from(b - b'0')).collect();
    let sum: u32 = digits
        .iter()
        .take(12)
        .enumerate()
        .map(|(idx, digit)| if idx % 2 == 0 { *digit } else { *digit * 3 })
        .sum();
    let check = (10 - (sum % 10)) % 10;
    check == digits[12]
}

// ---------------------------------------------------------------------------
// 商品名加工パイプライン（25-io-plu-formatter.md セクション12.4）
// ---------------------------------------------------------------------------

/// 商品名をPLU用に加工してCP932の16バイトに収める
///
/// パイプライン順序固定:
/// 1. `_` → 半角スペース
/// 2. タブ/改行 → 半角スペース
/// 3. 全角カナ → 半角カナ
/// 4. CP932エンコード
/// 5. 16バイト切り詰め
/// 6. 16バイトパディング
fn process_product_name(name: &str, product_code: &str) -> Result<Vec<u8>, PluFormatError> {
    // Step 1: _ → 半角スペース
    let name = name.replace('_', " ");
    // Step 2: タブ/改行 → 半角スペース
    let name = name.replace(['\t', '\n', '\r'], " ");
    // Step 3: 全角カナ → 半角カナ
    let name = fullwidth_to_halfwidth_kana(&name);

    // Step 4: CP932エンコード
    let (encoded, _, had_replacements) = encoding_rs::SHIFT_JIS.encode(&name);
    if had_replacements {
        // had_replacements=true なら必ず特定できるはず。'?' はフォールバック（理論上到達しない）
        let bad_char = find_unmappable_char(&name).unwrap_or('?');
        return Err(PluFormatError::EncodingError {
            product_code: product_code.to_string(),
            char: bad_char,
            message: format!(
                "商品 {} の名称にCP932非対応文字 '{}' が含まれています",
                product_code, bad_char
            ),
        });
    }

    // Step 5: 16バイト切り詰め（マルチバイト境界考慮）
    let mut bytes = truncate_cp932(&encoded, NAME_MAX_BYTES);

    // Step 6: 16バイトパディング
    bytes.resize(NAME_MAX_BYTES, 0x20);

    Ok(bytes)
}

/// CP932バイト列をmax_bytesまで安全に切り詰める
///
/// マルチバイト文字の途中で切れないよう、先頭から走査して境界を尊重する
fn truncate_cp932(bytes: &[u8], max_bytes: usize) -> Vec<u8> {
    if bytes.len() <= max_bytes {
        return bytes.to_vec();
    }
    let mut i = 0;
    while i < max_bytes {
        if is_cp932_lead_byte(bytes[i]) {
            if i + 1 < max_bytes {
                i += 2;
            } else {
                break; // 2バイト文字が入りきらない
            }
        } else {
            i += 1;
        }
    }
    bytes[..i].to_vec()
}

/// CP932マルチバイト文字の先頭バイトかどうか判定
fn is_cp932_lead_byte(b: u8) -> bool {
    (0x81..=0x9F).contains(&b) || (0xE0..=0xFC).contains(&b)
}

/// UTF-8文字列からCP932でエンコードできない最初の文字を見つける
fn find_unmappable_char(s: &str) -> Option<char> {
    for ch in s.chars() {
        let mut buf = [0u8; 4];
        let utf8 = ch.encode_utf8(&mut buf);
        let (_, _, had_replacements) = encoding_rs::SHIFT_JIS.encode(utf8);
        if had_replacements {
            return Some(ch);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 課税方式マッピング（25-io-plu-formatter.md セクション12.5）
// ---------------------------------------------------------------------------

/// tax_rate → カシオレジスターツールの課税方式テキスト
fn map_tax_rate(tax_rate: &str, product_code: &str) -> Result<&'static str, PluFormatError> {
    match tax_rate {
        "10" => Ok("税1(内税)"),
        "8" => Ok("税2(内税)"),
        "0" => Ok("非課税"),
        _ => Err(PluFormatError::TaxMappingError {
            product_code: product_code.to_string(),
            tax_rate: tax_rate.to_string(),
            message: format!("税率'{}'はPLU書出しに対応していません", tax_rate),
        }),
    }
}

// ---------------------------------------------------------------------------
// 全角→半角カナ変換
// ---------------------------------------------------------------------------

/// 全角カタカナを半角カタカナに変換する
///
/// 濁点・半濁点付き文字は2文字に分解（例: ガ→ｶﾞ）
fn fullwidth_to_halfwidth_kana(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            // 濁点付きカタカナ → 基本文字 + ﾞ
            'ガ' => result.push_str("ｶﾞ"),
            'ギ' => result.push_str("ｷﾞ"),
            'グ' => result.push_str("ｸﾞ"),
            'ゲ' => result.push_str("ｹﾞ"),
            'ゴ' => result.push_str("ｺﾞ"),
            'ザ' => result.push_str("ｻﾞ"),
            'ジ' => result.push_str("ｼﾞ"),
            'ズ' => result.push_str("ｽﾞ"),
            'ゼ' => result.push_str("ｾﾞ"),
            'ゾ' => result.push_str("ｿﾞ"),
            'ダ' => result.push_str("ﾀﾞ"),
            'ヂ' => result.push_str("ﾁﾞ"),
            'ヅ' => result.push_str("ﾂﾞ"),
            'デ' => result.push_str("ﾃﾞ"),
            'ド' => result.push_str("ﾄﾞ"),
            'バ' => result.push_str("ﾊﾞ"),
            'ビ' => result.push_str("ﾋﾞ"),
            'ブ' => result.push_str("ﾌﾞ"),
            'ベ' => result.push_str("ﾍﾞ"),
            'ボ' => result.push_str("ﾎﾞ"),
            'ヴ' => result.push_str("ｳﾞ"),
            // 半濁点付きカタカナ → 基本文字 + ﾟ
            'パ' => result.push_str("ﾊﾟ"),
            'ピ' => result.push_str("ﾋﾟ"),
            'プ' => result.push_str("ﾌﾟ"),
            'ペ' => result.push_str("ﾍﾟ"),
            'ポ' => result.push_str("ﾎﾟ"),
            // 通常カタカナ（清音 + 小書き + その他）
            'ァ' => result.push('ｧ'),
            'ア' => result.push('ｱ'),
            'ィ' => result.push('ｨ'),
            'イ' => result.push('ｲ'),
            'ゥ' => result.push('ｩ'),
            'ウ' => result.push('ｳ'),
            'ェ' => result.push('ｪ'),
            'エ' => result.push('ｴ'),
            'ォ' => result.push('ｫ'),
            'オ' => result.push('ｵ'),
            'カ' => result.push('ｶ'),
            'キ' => result.push('ｷ'),
            'ク' => result.push('ｸ'),
            'ケ' => result.push('ｹ'),
            'コ' => result.push('ｺ'),
            'サ' => result.push('ｻ'),
            'シ' => result.push('ｼ'),
            'ス' => result.push('ｽ'),
            'セ' => result.push('ｾ'),
            'ソ' => result.push('ｿ'),
            'タ' => result.push('ﾀ'),
            'チ' => result.push('ﾁ'),
            'ツ' => result.push('ﾂ'),
            'テ' => result.push('ﾃ'),
            'ト' => result.push('ﾄ'),
            'ナ' => result.push('ﾅ'),
            'ニ' => result.push('ﾆ'),
            'ヌ' => result.push('ﾇ'),
            'ネ' => result.push('ﾈ'),
            'ノ' => result.push('ﾉ'),
            'ハ' => result.push('ﾊ'),
            'ヒ' => result.push('ﾋ'),
            'フ' => result.push('ﾌ'),
            'ヘ' => result.push('ﾍ'),
            'ホ' => result.push('ﾎ'),
            'マ' => result.push('ﾏ'),
            'ミ' => result.push('ﾐ'),
            'ム' => result.push('ﾑ'),
            'メ' => result.push('ﾒ'),
            'モ' => result.push('ﾓ'),
            'ヤ' => result.push('ﾔ'),
            'ャ' => result.push('ｬ'),
            'ユ' => result.push('ﾕ'),
            'ュ' => result.push('ｭ'),
            'ヨ' => result.push('ﾖ'),
            'ョ' => result.push('ｮ'),
            'ラ' => result.push('ﾗ'),
            'リ' => result.push('ﾘ'),
            'ル' => result.push('ﾙ'),
            'レ' => result.push('ﾚ'),
            'ロ' => result.push('ﾛ'),
            'ワ' => result.push('ﾜ'),
            'ヲ' => result.push('ｦ'),
            'ン' => result.push('ﾝ'),
            'ッ' => result.push('ｯ'),
            'ー' => result.push('ｰ'),
            // 全角句読点・記号
            '。' => result.push('｡'),
            '「' => result.push('｢'),
            '」' => result.push('｣'),
            '、' => result.push('､'),
            '・' => result.push('･'),
            // その他の文字はそのまま
            _ => result.push(ch),
        }
    }
    result
}

// ===========================================================================
// テスト
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用のPluExportRowを生成するヘルパー
    fn make_row(
        product_code: &str,
        jan_code: Option<&str>,
        name: &str,
        selling_price: i64,
        tax_rate: &str,
        department_name: &str,
    ) -> PluExportRow {
        PluExportRow {
            memory_no: 217,
            product_code: product_code.to_string(),
            jan_code: jan_code.map(String::from),
            name: name.to_string(),
            selling_price,
            tax_rate: tax_rate.to_string(),
            department_name: department_name.to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // ゴールデンテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_golden_single_product_req402() {
        // REQ-402: PLU書出し
        // 25-io-plu-formatter.md セクション12.3 入出力例
        let rows = vec![make_row(
            "4901234567894",
            Some("4901234567894"),
            "ハマナカ アミアミ極太 col.42",
            648,
            "10",
            "毛糸",
        )];

        let result = generate_plu_tsv_with_date(&rows, "20260408").unwrap();

        // CRLF改行確認
        assert!(
            result.bytes.windows(2).any(|w| w == b"\r\n"),
            "CRLF改行が含まれること"
        );

        // CP932デコード後に検証
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&result.bytes);

        // ヘッダ行が存在すること
        assert!(decoded.contains("メモリNo."));
        assert!(decoded.contains("ｽｷｬﾆﾝｸﾞｺｰﾄﾞ"));
        assert!(decoded.contains("入力桁制限"));
        let lines: Vec<&str> = decoded.split("\r\n").collect();
        assert!(lines.len() >= 2, "ヘッダ+データ行が必要");

        let header_fields: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(
            header_fields,
            vec![
                "メモリNo.",
                "ｽｷｬﾆﾝｸﾞｺｰﾄﾞ",
                "名称",
                "単価",
                "課税方式",
                "単品売り",
                "負単価",
                "品番PLU",
                "ゼロ単価",
                "入力桁制限",
                "部門リンク",
            ]
        );
        let data_fields: Vec<&str> = lines[1].split('\t').collect();
        // 現地profile: 通常PLU216枠使用後のスキャニングPLU開始番号
        assert_eq!(data_fields[0], "000217", "メモリNo.");
        assert_eq!(data_fields[1], "4901234567894", "ｽｷｬﾆﾝｸﾞｺｰﾄﾞ");
        // 名称: ハマナカ→ﾊﾏﾅｶ, アミアミ→ｱﾐｱﾐ, 極太は全角漢字(2byte), col.42は半角
        assert_eq!(data_fields[3], "648", "単価");
        assert_eq!(data_fields[4], "税1(内税)", "課税方式");
        assert_eq!(data_fields[5], "いいえ", "単品売り");
        assert_eq!(data_fields[6], "いいえ", "負単価");
        assert_eq!(data_fields[7], "いいえ", "品番PLU");
        assert_eq!(data_fields[8], "いいえ", "ゼロ単価");
        assert_eq!(data_fields[9], "無し", "入力桁制限");
        assert_eq!(data_fields[10], "毛糸", "部門リンク");

        // ファイル名
        assert_eq!(result.suggested_filename, "PLU_20260408.txt");
        assert_eq!(result.content_type, "text/tab-separated-values");
        assert_eq!(result.encoding, "CP932");
    }

    #[test]
    fn test_golden_crlf_line_endings_req402() {
        // REQ-402: PLU書出し
        // 全行がCRLFで終わること
        let rows = vec![
            make_row("P001", Some("4901234567894"), "テスト1", 100, "10", "毛糸"),
            make_row("P002", Some("4901234567887"), "テスト2", 200, "8", "布"),
        ];
        let result = generate_plu_tsv_with_date(&rows, "20260411").unwrap();

        // LFのみ（CRなし）が存在しないことを確認
        let bytes = &result.bytes;
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                assert!(
                    i > 0 && bytes[i - 1] == b'\r',
                    "位置 {} に LF のみ発見（CRLFでない）",
                    i
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // 16バイト切り詰め + パディング
    // -----------------------------------------------------------------------

    #[test]
    fn test_name_truncation_multibyte_boundary_req402() {
        // REQ-402: PLU書出し
        // 25-io-plu-formatter.md: マルチバイト文字の途中で切れないこと
        // "極太極太極太極太極" = 全角漢字9文字 → CP932で18バイト → 16バイトに切り詰め → 8文字(16バイト)
        let name_bytes = process_product_name("極太極太極太極太極", "TEST-001").unwrap();
        assert_eq!(name_bytes.len(), 16, "常に16バイト");

        // 16バイト目がマルチバイトの途中ではないことを確認
        // CP932で "極太極太極太極太" = 8文字 × 2バイト = 16バイト
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&name_bytes);
        let trimmed = decoded.trim_end();
        assert_eq!(trimmed, "極太極太極太極太");
    }

    #[test]
    fn test_name_truncation_15byte_plus_multibyte_req402() {
        // REQ-402: PLU書出し
        // 15バイト + 2バイト文字 → 2バイト文字は入りきらないので15バイト+パディング1バイト
        // 半角15文字 + 全角1文字 = 15 + 2 = 17バイト → 15バイト+1パディング
        let name_bytes = process_product_name("123456789012345太", "TEST-002").unwrap();
        assert_eq!(name_bytes.len(), 16);
        // 末尾はパディングスペース
        assert_eq!(name_bytes[15], 0x20);
    }

    #[test]
    fn test_name_truncation_exactly_16bytes_req402() {
        // REQ-402: PLU書出し
        // ちょうど16バイト → 切り詰めなし、パディングなし
        // 半角16文字 = 16バイト
        let name_bytes = process_product_name("1234567890ABCDEF", "TEST-EXACT").unwrap();
        assert_eq!(name_bytes.len(), 16);
        assert_eq!(&name_bytes[..], b"1234567890ABCDEF");
    }

    #[test]
    fn test_name_padding_short_name_req402() {
        // REQ-402: PLU書出し
        // 短い名前 → 16バイトまでパディング
        let name_bytes = process_product_name("ABC", "TEST-003").unwrap();
        assert_eq!(name_bytes.len(), 16);
        assert_eq!(&name_bytes[..3], b"ABC");
        assert!(name_bytes[3..].iter().all(|&b| b == 0x20));
    }

    // -----------------------------------------------------------------------
    // 商品名加工パイプライン
    // -----------------------------------------------------------------------

    #[test]
    fn test_underscore_replacement_req402() {
        // REQ-402: PLU書出し
        // 25-io-plu-formatter.md: _ → 半角スペース置換
        let name_bytes = process_product_name("A_B_C", "TEST-004").unwrap();
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&name_bytes);
        assert!(
            decoded.starts_with("A B C"),
            "_ が半角スペースに置換されること"
        );
    }

    #[test]
    fn test_tab_newline_replacement_req402() {
        // REQ-402: PLU書出し
        // タブ/改行 → 半角スペース
        let name_bytes = process_product_name("A\tB\nC\rD", "TEST-005").unwrap();
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&name_bytes);
        assert!(
            decoded.starts_with("A B C D"),
            "タブ/改行が半角スペースに置換されること"
        );
    }

    #[test]
    fn test_fullwidth_to_halfwidth_kana_conversion_req402() {
        // REQ-402: PLU書出し
        // 全角カナ → 半角カナ
        let result = fullwidth_to_halfwidth_kana("アイウエオ");
        assert_eq!(result, "ｱｲｳｴｵ");

        // 濁点付き
        let result = fullwidth_to_halfwidth_kana("ガギグゲゴ");
        assert_eq!(result, "ｶﾞｷﾞｸﾞｹﾞｺﾞ");

        // 半濁点付き
        let result = fullwidth_to_halfwidth_kana("パピプペポ");
        assert_eq!(result, "ﾊﾟﾋﾟﾌﾟﾍﾟﾎﾟ");

        // 長音
        let result = fullwidth_to_halfwidth_kana("ー");
        assert_eq!(result, "ｰ");

        // 混在
        let result = fullwidth_to_halfwidth_kana("ハマナカ");
        assert_eq!(result, "ﾊﾏﾅｶ");
    }

    // -----------------------------------------------------------------------
    // エラーテスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_unmappable_char_returns_error_req402() {
        // REQ-402: PLU書出し
        // 25-io-plu-formatter.md: CP932非対応文字 → EncodingError
        let rows = vec![make_row(
            "ERR-001",
            Some("4901234567894"),
            "テスト🎉",
            100,
            "10",
            "毛糸",
        )];
        let result = generate_plu_tsv_with_date(&rows, "20260411");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            PluFormatError::EncodingError {
                product_code,
                char: bad_char,
                message,
            } => {
                assert_eq!(product_code, "ERR-001");
                assert_eq!(bad_char, '🎉');
                assert!(message.contains("ERR-001"));
                assert!(message.contains("🎉"));
            }
            _ => panic!("EncodingError が期待されるが {:?} が返った", err),
        }
    }

    #[test]
    fn test_missing_or_invalid_scanning_code_returns_error_req402() {
        // REQ-402: PLU書出し
        // CV17 1.1.1 scanning PLU: product_code fallbackは禁止
        for (product_code, jan_code) in [
            ("JAN-NONE", None),
            ("JAN-SHORT", Some("12345678")),
            ("JAN-BAD-CHECK", Some("4901234567890")),
        ] {
            let rows = vec![make_row(
                product_code,
                jan_code,
                "テスト",
                100,
                "10",
                "毛糸",
            )];
            let result = generate_plu_tsv_with_date(&rows, "20260411");
            assert!(result.is_err());
            let err = result.unwrap_err();
            match err {
                PluFormatError::InvalidScanningCode {
                    product_code: actual,
                    message,
                } => {
                    assert_eq!(actual, product_code);
                    assert!(message.contains("JANコード"));
                }
                _ => panic!("InvalidScanningCode が期待されるが {:?} が返った", err),
            }
        }
    }

    #[test]
    fn test_department_name_unmappable_char_returns_error_req402() {
        // REQ-402: PLU書出し
        // レビュー指摘: department_name のCP932非対応文字も検出すること
        let rows = vec![make_row(
            "DPT-001",
            Some("4901234567894"),
            "テスト",
            100,
            "10",
            "毛糸🧶",
        )];
        let result = generate_plu_tsv_with_date(&rows, "20260411");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            PluFormatError::EncodingError {
                product_code,
                message,
                ..
            } => {
                assert_eq!(product_code, "DPT-001");
                assert!(message.contains("部門名"));
            }
            _ => panic!("EncodingError が期待されるが {:?} が返った", err),
        }
    }

    #[test]
    fn test_invalid_tax_rate_returns_error_req402() {
        // REQ-402: PLU書出し
        // 25-io-plu-formatter.md: 不正税率 → TaxMappingError
        let rows = vec![make_row(
            "ERR-002",
            Some("4901234567894"),
            "テスト",
            100,
            "15",
            "毛糸",
        )];
        let result = generate_plu_tsv_with_date(&rows, "20260411");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            PluFormatError::TaxMappingError {
                product_code,
                tax_rate,
                message,
            } => {
                assert_eq!(product_code, "ERR-002");
                assert_eq!(tax_rate, "15");
                assert!(message.contains("15"));
            }
            _ => panic!("TaxMappingError が期待されるが {:?} が返った", err),
        }
    }

    // -----------------------------------------------------------------------
    // 境界テスト
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_input_returns_header_only_req402() {
        // REQ-402: PLU書出し
        // 空リスト → ヘッダのみPLUファイル
        let result = generate_plu_tsv_with_date(&[], "20260411").unwrap();
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&result.bytes);
        let lines: Vec<&str> = decoded.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "ヘッダ行のみ");
        assert!(lines[0].contains("メモリNo."));
    }

    #[test]
    fn test_jan_code_none_does_not_use_product_code_req402() {
        // REQ-402: PLU書出し
        // CV17 1.1.1: jan_code=None → product_code fallbackは禁止
        let rows = vec![make_row("HZ-0001", None, "テスト", 500, "10", "ヘア雑貨")];
        let result = generate_plu_tsv_with_date(&rows, "20260411");
        assert!(matches!(
            result,
            Err(PluFormatError::InvalidScanningCode { .. })
        ));
    }

    #[test]
    fn test_selling_price_boundaries_req402() {
        // REQ-402: PLU書出し
        // selling_price = 0, 999999
        let rows = vec![
            make_row("P-ZERO", Some("4901234567894"), "商品A", 0, "0", "食品"),
            make_row(
                "P-MAX",
                Some("4901234567887"),
                "商品B",
                999999,
                "10",
                "毛糸",
            ),
        ];
        let result = generate_plu_tsv_with_date(&rows, "20260411").unwrap();
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&result.bytes);
        let lines: Vec<&str> = decoded.split("\r\n").collect();

        let fields_zero: Vec<&str> = lines[1].split('\t').collect();
        assert_eq!(fields_zero[3], "0");
        assert_eq!(fields_zero[4], "非課税");

        let fields_max: Vec<&str> = lines[2].split('\t').collect();
        assert_eq!(fields_max[3], "999999");
        assert_eq!(fields_max[4], "税1(内税)");
    }

    #[test]
    fn test_tax_rate_all_mappings_req402() {
        // REQ-402: PLU書出し
        // 全3パターンの課税方式マッピング
        assert_eq!(map_tax_rate("10", "X").unwrap(), "税1(内税)");
        assert_eq!(map_tax_rate("8", "X").unwrap(), "税2(内税)");
        assert_eq!(map_tax_rate("0", "X").unwrap(), "非課税");
    }

    #[test]
    fn test_memory_no_req907_is_zero_padded_and_range_checked() {
        // REQ-907: A-E2/A-E3 independent boundary oracle.
        let mut low = make_row("LOW", Some("4901234567894"), "low", 100, "10", "毛糸");
        low.memory_no = 217;
        let mut high = make_row("HIGH", Some("4901234567887"), "high", 100, "10", "毛糸");
        high.memory_no = 5_000;
        let output = generate_plu_tsv_with_date(&[low, high], "20260818").unwrap();
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&output.bytes);
        let lines: Vec<_> = decoded.split("\r\n").collect();
        assert!(lines[1].starts_with("000217\t"));
        assert!(lines[2].starts_with("005000\t"));

        for memory_no in [216, 5_001] {
            let mut invalid = make_row("BAD", Some("4901234567894"), "bad", 100, "10", "毛糸");
            invalid.memory_no = memory_no;
            assert!(matches!(
                generate_plu_tsv_with_date(&[invalid], "20260818"),
                Err(PluFormatError::InvalidMemoryNo { .. })
            ));
        }
    }
}
