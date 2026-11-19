//! IO-08: EJパーサー
//!
//! カシオSR-S4000が SD に保存する電子ジャーナル（EJ）1 file を、取引単位の記録へ構造復元する。
//! 純関数。DB非依存。
//!
//! docs/function-design/29-io-ej-parser.md に基づく実装。

use encoding_rs::SHIFT_JIS;
use sha2::{Digest, Sha256};
use std::fmt;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// パース成功結果（復元不能の記録や診断があっても返る）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjParseResult {
    /// SHA-256ハッシュ（raw bytes基準、hex小文字64文字）
    pub file_hash: String,
    /// 最初の記録ヘッダより前の行（IO-08-D4）
    pub leading_lines: Vec<EjLine>,
    /// file の出現順の記録
    pub records: Vec<EjRecord>,
    pub diagnostics: Vec<EjDiagnostic>,
}

/// 1 行（行番号は 1 始まり、text は decode したままの 24 バイト分）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjLine {
    pub line_no: usize,
    pub text: String,
    pub kind: EjLineKind,
}

/// 行種（IO-08-D5）。記録の種類と記録内の位置で決まる
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EjLineKind {
    Separator,
    Quantity {
        quantity: i64,
        unit_price: i64,
    },
    /// 金額は印字の符号のまま（戻の印の直後だけ負）
    Item {
        name: String,
        amount: i64,
    },
    /// 折り返した明細の名称の行（IO-08-D5a）
    ItemName {
        name: String,
    },
    /// 直前の行の続きの金額だけの行（IO-08-D5a）
    Continued {
        amount: i64,
    },
    ItemCount {
        count: i64,
    },
    /// prefix だけで判定したラベル行。値は解釈しない
    Labeled {
        label: &'static str,
    },
    /// 訂正（IO-08-D6a）
    Correction {
        amount: i64,
    },
    Subtotal {
        amount: i64,
    },
    /// 率の行（`-N%`）。値を解釈しない
    DiscountRate,
    /// ％値引き。subtotal = 金額の後に `*`
    PercentDiscount {
        amount: i64,
        subtotal: bool,
    },
    /// マイナスキー（`－`）
    MinusKey {
        amount: i64,
    },
    /// 戻の印（`戻 ････`）
    ReturnMark,
    /// 取引中止の印（`取引中止 ････`）
    CancelMark,
    /// 番号印字（`＃ N`）。値を解釈しない
    NumberPrint,
    ReportTitle {
        leading_no: Option<String>,
        trailing_no: Option<String>,
    },
    ReportEnd,
    AmountOnly,
    /// 値を解釈しない文字の行（案内文・設定の印字・練習の本文）
    Text,
    Status {
        label: &'static str,
        result: String,
    },
    Unknown,
}

/// 記録（2 行ヘッダ + 本文）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjRecord {
    /// ヘッダ1行目の行番号（2行目は header_line_no + 1）
    pub header_line_no: usize,
    /// 見出しのモード欄。記録の種類は kind、返品の判定は mode（IO-08-D3a）
    pub mode: EjMode,
    pub kind: EjRecordKind,
    /// 印字日時（`YYYY-MM-DD HH:MM`、変換しない）
    pub printed_at: String,
    /// 番号行の4桁の欄（意味は未検証、文字列のまま）
    pub number_prefix: String,
    /// 番号行の6桁の番号（先頭0を保つ）
    pub number: String,
    pub body: Vec<EjLine>,
    pub restoration: EjRestoration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EjMode {
    Normal,
    Return,
    Training,
    Settlement,
    Inspection,
    Program,
    Program1,
    Program3,
    Off,
    Unrecognized(String),
}

/// 記録の種類（IO-08-D3a）。モード欄ではなく本文の行で決める
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EjRecordKind {
    /// 取引（区切りあり）。通常と戻の両方を含み、返品は mode で判定する
    Sale,
    /// 取引中止
    Cancelled,
    /// 入金 / 出金 / 替
    CashMovement,
    /// 領収書の発行
    Receipt,
    /// 精算票（Z）。completed は終わりの印字の有無
    Settlement {
        report: EjSettlementReport,
        completed: bool,
    },
    /// 点検票（X）
    Inspection,
    /// 設定の書込み・読込み・保存・印字
    Settings,
    /// 練習
    Training,
    /// どれにも当たらない（常に Unresolved）
    Unclassified,
}

/// 日計明細 / ＰＬＵ / 勤怠
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EjSettlementReport {
    Daily,
    Plu,
    Attendance,
}

/// 復元状態（IO-08-D8）。Unresolved は明細を持たない
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EjRestoration {
    Restored {
        items: Vec<EjItem>,
        item_count: i64,
        adjustments: Vec<EjAdjustment>,
    },
    NoItems,
    Unresolved {
        reasons: Vec<EjDiagnosticCode>,
    },
}

/// 復元済みの明細（同名の行を合算しない。返品でも符号を反転しない）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjItem {
    /// 名称行の行番号
    pub line_no: usize,
    pub name: String,
    /// 数量行が掛かる明細はその値、戻の印の明細は -1、それ以外は 1
    pub quantity: i64,
    /// 数量行があるときだけ
    pub unit_price: Option<i64>,
    pub amount: i64,
}

/// 値引き（IO-08-D6b）。明細へ配らない
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjAdjustment {
    /// ％値引き・マイナスキーの行の行番号
    pub line_no: usize,
    pub kind: EjAdjustmentKind,
    /// 印字の符号のまま（負）
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EjAdjustmentKind {
    /// 明細値引き。直前の明細の line_no
    ItemDiscount { item_line_no: usize },
    /// 小計値引き
    SubtotalDiscount,
    /// マイナスキー。掛かり先を解釈しない
    MinusKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjDiagnostic {
    /// 範囲 File のときは None。範囲 Record のときはヘッダ1行目
    pub line_no: Option<usize>,
    pub code: EjDiagnosticCode,
    pub scope: EjDiagnosticScope,
    /// code ごとの固定文言（行の生の文字列を含めない）
    pub message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EjDiagnosticCode {
    UnknownLine,
    InvalidWidth,
    UnrecognizedMode,
    InconsistentRecord,
    IncompleteRecord,
    MissingFinalNewline,
    LeadingFragment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EjDiagnosticScope {
    Line,
    Record,
    File,
}

/// 致命的エラー（部分結果を返さない）
#[derive(Debug)]
pub enum EjParseError {
    DecodeFailed(String),
    NoRecords(String),
    Empty(String),
}

impl fmt::Display for EjParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjParseError::DecodeFailed(msg)
            | EjParseError::NoRecords(msg)
            | EjParseError::Empty(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for EjParseError {}

const LINE_WIDTH: usize = 24;
const SEPARATOR: [u8; LINE_WIDTH] = [b'-'; LINE_WIDTH];
const TOTAL_LABELS: [&str; 12] = [
    "対象計",
    "対象計※",
    "内税",
    "内税※",
    "合  計",
    "お預り",
    "お  釣",
    "現金",
    "ｸﾚｼﾞｯﾄ電子M",
    "ｸｰﾎﾟﾝﾎﾟｲﾝﾄ払",
    "売掛",
    "商品券",
];
/// 合計域の支払行（IO-08-D5b）。`合  計` が無いときの照合の金額の候補
const PAYMENT_LABELS: [&str; 5] = ["現金", "ｸﾚｼﾞｯﾄ電子M", "ｸｰﾎﾟﾝﾎﾟｲﾝﾄ払", "売掛", "商品券"];
const REDUCED_TAX_NOTE: &str = "注）※は軽減税率適用";
const PAID_LABELS: [&str; 3] = ["入金", "出金", "替"];
const REPORT_LABELS: [&str; 12] = [
    "総売",
    "純売",
    "純客",
    "現金在高",
    "信在高",
    "貸在高",
    "券在高",
    "対象計",
    "内税",
    "消費税合計",
    "領収書",
    "取引中止",
];
const STATUS_LABELS: [&str; 5] = [
    "SD設定書込み",
    "SDｶｰﾄﾞ保存",
    "ｽﾏ-ﾄﾌｫﾝ送信",
    "SD設定読込み",
    "自動設定保存",
];

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// EJ 1 file の生バイト列を記録の列へ構造復元する
///
/// 29-io-ej-parser.md IO-08.1〜IO-08.8
pub fn parse_ej(raw_bytes: &[u8]) -> Result<EjParseResult, EjParseError> {
    if raw_bytes.is_empty() {
        return Err(EjParseError::Empty(
            "ファイルが空です。ファイル形式を確認してください".to_string(),
        ));
    }
    let file_hash = format!("{:x}", Sha256::digest(raw_bytes));

    // IO-08-D2: CRLF だけで分割し、改行を正規化しない
    let mut segments = split_crlf(raw_bytes);
    let missing_final_line = if segments.last().is_some_and(|s| s.is_empty()) {
        segments.pop();
        None
    } else {
        Some(segments.len())
    };

    let mut lines = Vec::with_capacity(segments.len());
    for bytes in segments {
        let (text, had_errors) = SHIFT_JIS.decode_without_bom_handling(bytes);
        if had_errors {
            return Err(EjParseError::DecodeFailed(
                "CP932デコードに失敗しました。ファイル形式を確認してください".to_string(),
            ));
        }
        lines.push(RawLine {
            bytes,
            text: text.into_owned(),
        });
    }

    // IO-08-D3: 記録ヘッダの位置
    let mut headers = Vec::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        if let Some(header) = parse_header(&lines[i], &lines[i + 1]) {
            headers.push((i, header));
            i += 2;
        } else {
            i += 1;
        }
    }
    if headers.is_empty() {
        return Err(EjParseError::NoRecords(
            "記録のヘッダがありません。ファイル形式を確認してください".to_string(),
        ));
    }

    let mut diagnostics = Vec::new();

    // IO-08-D4: 最初のヘッダより前は先頭断片
    let first_header = headers[0].0;
    if first_header > 0 {
        diagnostics.push(diagnostic(EjDiagnosticCode::LeadingFragment, None));
    }
    let mut leading_lines = Vec::with_capacity(first_header);
    for (index, raw) in lines[..first_header].iter().enumerate() {
        if !fixed_width(raw.bytes) {
            diagnostics.push(diagnostic(EjDiagnosticCode::InvalidWidth, Some(index + 1)));
        }
        leading_lines.push(EjLine {
            line_no: index + 1,
            text: raw.text.clone(),
            kind: EjLineKind::Unknown,
        });
    }

    let ends: Vec<usize> = headers
        .iter()
        .skip(1)
        .map(|(start, _)| *start)
        .chain([lines.len()])
        .collect();
    let records = headers
        .into_iter()
        .zip(ends)
        .map(|((start, header), end)| {
            build_record(
                header,
                start,
                &lines[start + 2..end],
                end == lines.len(),
                missing_final_line,
                &mut diagnostics,
            )
        })
        .collect();

    Ok(EjParseResult {
        file_hash,
        leading_lines,
        records,
        diagnostics,
    })
}

// ---------------------------------------------------------------------------
// 内部関数
// ---------------------------------------------------------------------------

struct RawLine<'a> {
    bytes: &'a [u8],
    text: String,
}

struct Header {
    mode: EjMode,
    printed_at: String,
    number_prefix: String,
    number: String,
}

/// CRLF で分割する。最後の要素は最後の CRLF の後（空なら改行で終わっている）
fn split_crlf(raw: &[u8]) -> Vec<&[u8]> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i + 1 < raw.len() {
        if raw[i] == b'\r' && raw[i + 1] == b'\n' {
            segments.push(&raw[start..i]);
            i += 2;
            start = i;
        } else {
            i += 1;
        }
    }
    segments.push(&raw[start..]);
    segments
}

/// IO-08-D2: 24 バイトちょうどで、孤立した CR / LF を含まない行。
/// CR / LF は trim で消えるため、ヘッダ検出と行の分類より前にここで弾く
fn fixed_width(bytes: &[u8]) -> bool {
    bytes.len() == LINE_WIDTH && !bytes.iter().any(|b| matches!(b, b'\r' | b'\n'))
}

/// IO-08-D3: モード欄5バイト + `YYYY-MM-DD HH:MM` + 空白3バイト / 空白13バイト + `NNNN-NNNNNN`
fn parse_header(first: &RawLine, second: &RawLine) -> Option<Header> {
    let (a, b) = (first.bytes, second.bytes);
    if !fixed_width(a) || !fixed_width(b) {
        return None;
    }
    let printed_at = &a[5..21];
    let date_ok = printed_at.iter().enumerate().all(|(i, c)| match i {
        4 | 7 => *c == b'-',
        10 => *c == b' ',
        13 => *c == b':',
        _ => c.is_ascii_digit(),
    });
    let number_ok = b[..13].iter().all(|c| *c == b' ')
        && b[13..17].iter().all(u8::is_ascii_digit)
        && b[17] == b'-'
        && b[18..].iter().all(u8::is_ascii_digit);
    if !date_ok || a[21..] != *b"   " || !number_ok {
        return None;
    }
    // 行全体の decode は成功済みで、6バイト目以降は ASCII のためモード欄だけの decode も失敗しない
    let (mode_field, _) = SHIFT_JIS.decode_without_bom_handling(&a[..5]);
    let mode = match mode_field.trim() {
        "" => EjMode::Normal,
        "戻" => EjMode::Return,
        "練習" => EjMode::Training,
        "精算" => EjMode::Settlement,
        "点検" => EjMode::Inspection,
        "PGM" => EjMode::Program,
        "PGM1" => EjMode::Program1,
        "PGM3" => EjMode::Program3,
        "OFF" => EjMode::Off,
        other => EjMode::Unrecognized(other.to_string()),
    };
    Some(Header {
        mode,
        printed_at: String::from_utf8_lossy(printed_at).into_owned(),
        number_prefix: String::from_utf8_lossy(&b[13..17]).into_owned(),
        number: String::from_utf8_lossy(&b[18..]).into_owned(),
    })
}

fn build_record(
    header: Header,
    start: usize,
    body: &[RawLine],
    is_last: bool,
    missing_final_line: Option<usize>,
    diagnostics: &mut Vec<EjDiagnostic>,
) -> EjRecord {
    let header_line_no = start + 1;
    let mut reasons = Vec::new();
    let mut report = |code, line_no, reasons: &mut Vec<EjDiagnosticCode>| {
        diagnostics.push(diagnostic(code, Some(line_no)));
        if !reasons.contains(&code) {
            reasons.push(code);
        }
    };

    let unrecognized = matches!(header.mode, EjMode::Unrecognized(_));
    if unrecognized {
        report(
            EjDiagnosticCode::UnrecognizedMode,
            header_line_no,
            &mut reasons,
        );
    }

    let kind = record_kind(&header.mode, body);
    // 番号印字の行を除いて数えた先頭の行（IO-08-D3a）
    let first_core = body.iter().position(|raw| !is_number_print(&raw.text));
    let last_core = body.iter().rposition(|raw| !is_number_print(&raw.text));
    let separator_at = body.iter().position(|raw| raw.bytes == SEPARATOR);
    let item_region_end = separator_at.unwrap_or(body.len());

    let mut lines: Vec<EjLine> = Vec::with_capacity(body.len());
    for (index, raw) in body.iter().enumerate() {
        let line_no = start + 3 + index;
        let text = raw.text.as_str();
        let line_kind = if !fixed_width(raw.bytes) {
            report(EjDiagnosticCode::InvalidWidth, line_no, &mut reasons);
            EjLineKind::Unknown
        } else {
            let first = Some(index) == first_core;
            let line_kind = match kind {
                // 未知のモードと練習は本文を分類しない（IO-08-D5d）
                EjRecordKind::Unclassified if unrecognized => EjLineKind::Unknown,
                EjRecordKind::Training => EjLineKind::Text,
                EjRecordKind::Settlement { report, .. } => {
                    classify_report(raw, Some(report), first)
                }
                EjRecordKind::Inspection | EjRecordKind::Unclassified => {
                    classify_report(raw, None, first)
                }
                EjRecordKind::Settings => classify_settings(raw),
                EjRecordKind::CashMovement if is_number_print(text) => EjLineKind::NumberPrint,
                EjRecordKind::CashMovement => labeled_kind(text, &PAID_LABELS),
                EjRecordKind::Receipt if is_number_print(text) => EjLineKind::NumberPrint,
                EjRecordKind::Receipt => receipt_label(text)
                    .map_or(EjLineKind::Unknown, |label| EjLineKind::Labeled { label }),
                EjRecordKind::Sale | EjRecordKind::Cancelled => match separator_at {
                    Some(at) if index == at => EjLineKind::Separator,
                    Some(at) if index > at => classify_totals(text, lines.last()),
                    _ => classify_items(
                        text,
                        lines.last(),
                        body[index + 1..item_region_end]
                            .first()
                            .map(|next| next.text.as_str()),
                        kind == EjRecordKind::Cancelled && Some(index) == last_core,
                    ),
                },
            };
            // 未知のモードは UnrecognizedMode 1 件で表し、本文の行ごとには出さない
            if line_kind == EjLineKind::Unknown && !unrecognized {
                report(EjDiagnosticCode::UnknownLine, line_no, &mut reasons);
            }
            line_kind
        };
        lines.push(EjLine {
            line_no,
            text: raw.text.clone(),
            kind: line_kind,
        });
    }

    // IO-08-D2 / D4: 改行の無い最後の行を含む記録は途中で切れたものとする
    let last_line_no = start + 2 + body.len();
    if let Some(line_no) =
        missing_final_line.filter(|n| (header_line_no..=last_line_no).contains(n))
    {
        report(EjDiagnosticCode::MissingFinalNewline, line_no, &mut reasons);
    }

    // 行単位の問題がある記録は、記録内の照合をせずに復元不能とする
    let restoration = if reasons.is_empty() {
        match check_record(&kind, &header.mode, &lines, is_last) {
            Ok(restoration) => restoration,
            Err(code) => {
                report(code, header_line_no, &mut reasons);
                EjRestoration::Unresolved { reasons }
            }
        }
    } else {
        EjRestoration::Unresolved { reasons }
    };

    EjRecord {
        header_line_no,
        mode: header.mode,
        kind,
        printed_at: header.printed_at,
        number_prefix: header.number_prefix,
        number: header.number,
        body: lines,
        restoration,
    }
}

/// トレーニングの表示の行（IO-08-D3a の 4）
fn is_training_display(text: &str) -> bool {
    text.starts_with("ﾄﾚｰﾆﾝｸﾞﾓｰﾄﾞを") || text.trim() == "･･････トレーニング･･････"
}

/// IO-08-D3a: 番号印字の行を除いた本文で、表を上から当てて記録の種類を決める
fn record_kind(mode: &EjMode, body: &[RawLine]) -> EjRecordKind {
    let core_lines: Vec<&RawLine> = body
        .iter()
        .filter(|raw| !is_number_print(&raw.text))
        .collect();
    let core: Vec<&str> = core_lines.iter().map(|raw| raw.text.as_str()).collect();
    if matches!(mode, EjMode::Unrecognized(_)) {
        return EjRecordKind::Unclassified;
    }
    if let Some(first) = core.first() {
        if let Some((report, _, _)) = parse_z_title(first) {
            let name = report_name(report);
            let completed = core[1..].iter().any(|text| text.trim() == name);
            return EjRecordKind::Settlement { report, completed };
        }
        if parse_x_title(first).is_some() {
            return EjRecordKind::Inspection;
        }
    }
    if *mode == EjMode::Training
        || (!core.is_empty() && core.iter().all(|text| is_training_display(text)))
    {
        return EjRecordKind::Training;
    }
    match mode {
        EjMode::Program | EjMode::Program1 | EjMode::Program3 | EjMode::Off => {
            return EjRecordKind::Settings
        }
        EjMode::Settlement | EjMode::Inspection => return EjRecordKind::Unclassified,
        _ => {}
    }
    match core.as_slice() {
        [only] if labeled(only, &PAID_LABELS).is_some() => EjRecordKind::CashMovement,
        [a, b, c]
            if receipt_label(a) == Some("一連No.")
                && receipt_label(b) == Some("領収No.")
                && receipt_label(c) == Some("領収書") =>
        {
            EjRecordKind::Receipt
        }
        [.., last]
            if is_mark(last, "取引中止")
                && !core_lines.iter().any(|raw| raw.bytes == SEPARATOR) =>
        {
            EjRecordKind::Cancelled
        }
        _ => EjRecordKind::Sale,
    }
}

/// IO-08-D6 / D7 / D7a: 行種がすべて既知の記録について、閉じと照合を判定する
fn check_record(
    kind: &EjRecordKind,
    mode: &EjMode,
    lines: &[EjLine],
    is_last: bool,
) -> Result<EjRestoration, EjDiagnosticCode> {
    match kind {
        EjRecordKind::Sale => restore_items(mode, lines),
        // IO-08-D7a: 取引中止の印（最後の行）より前の明細域が項目の規則を満たせば明細なし
        EjRecordKind::Cancelled => {
            let end = lines
                .iter()
                .position(|line| line.kind == EjLineKind::CancelMark)
                .ok_or(EjDiagnosticCode::IncompleteRecord)?;
            item_region(mode, &lines[..end])?;
            Ok(EjRestoration::NoItems)
        }
        // EOF を閉じの根拠にしない（IO-08-D4）。次のヘッダで閉じた中断は NoItems
        EjRecordKind::Settlement {
            completed: false, ..
        } if is_last => Err(EjDiagnosticCode::IncompleteRecord),
        // 題の無い精算・点検のモードの記録（未知のモードは行単位で先に Unresolved）
        EjRecordKind::Unclassified => Err(EjDiagnosticCode::IncompleteRecord),
        _ => Ok(EjRestoration::NoItems),
    }
}

/// IO-08-D6: 明細の復元と記録内の照合（数量×単価・点数・照合の金額）
fn restore_items(mode: &EjMode, lines: &[EjLine]) -> Result<EjRestoration, EjDiagnosticCode> {
    use EjDiagnosticCode::{IncompleteRecord, InconsistentRecord};

    let separator = lines
        .iter()
        .position(|line| line.kind == EjLineKind::Separator)
        .ok_or(IncompleteRecord)?;
    let (items, adjustments, saw_item) = item_region(mode, &lines[..separator])?;
    // 明細の項目の無い取引は未観測のため復元しない（すべて取り消した取引は復元する）
    if !saw_item {
        return Err(IncompleteRecord);
    }

    let totals = &lines[separator + 1..];
    let item_count = at_most_one(totals.iter().filter_map(|line| match line.kind {
        EjLineKind::ItemCount { count } => Some(count),
        _ => None,
    }))?
    .ok_or(IncompleteRecord)?;
    let total = reconciled_total(totals)?;

    let quantity_sum = items
        .iter()
        .try_fold(0i64, |sum, item| sum.checked_add(item.quantity));
    if quantity_sum != Some(item_count) || amount_sum(&items, &adjustments) != Some(total) {
        return Err(InconsistentRecord);
    }

    Ok(EjRestoration::Restored {
        items,
        item_count,
        adjustments,
    })
}

/// 有効な明細と値引きの金額の和
fn amount_sum(items: &[EjItem], adjustments: &[EjAdjustment]) -> Option<i64> {
    items
        .iter()
        .map(|item| item.amount)
        .chain(adjustments.iter().map(|adjustment| adjustment.amount))
        .try_fold(0i64, i64::checked_add)
}

/// 直前の項目（IO-08-D6a / D6b）
#[derive(PartialEq)]
enum Previous {
    Start,
    Item,
    ReturnItem,
    PercentDiscount,
    Subtotal,
    Other,
}

/// IO-08-D6 / D6a / D6b / D6d: 明細域の行を項目にまとめ、出現順に規則を当てる。
/// 有効な明細・値引きと、明細の項目（取り消したものを含む）があったかを返す
fn item_region(
    mode: &EjMode,
    region: &[EjLine],
) -> Result<(Vec<EjItem>, Vec<EjAdjustment>, bool), EjDiagnosticCode> {
    use EjDiagnosticCode::InconsistentRecord as Bad;
    use EjLineKind as K;

    let mut items: Vec<EjItem> = Vec::new();
    let mut adjustments: Vec<EjAdjustment> = Vec::new();
    let mut saw_item = false;
    let mut previous = Previous::Start;
    let mut i = 0;
    while i < region.len() {
        let kind_at = |at: usize| region.get(at).map(|line| &line.kind);
        // 小計の直後の項目は `*` つきの ％値引きだけ
        if previous == Previous::Subtotal && region[i].kind != K::DiscountRate {
            return Err(Bad);
        }
        match &region[i].kind {
            // 任意の数量行 + `Item`、または任意の数量行 + `ItemName` + `Continued`
            K::Quantity { .. } | K::Item { .. } | K::ItemName { .. } => {
                let (quantity, unit_price, at) = match region[i].kind {
                    K::Quantity {
                        quantity,
                        unit_price,
                    } => (quantity, Some(unit_price), i + 1),
                    _ => (1, None, i),
                };
                let (name, amount, next) = match kind_at(at) {
                    Some(K::Item { name, amount }) => (name, *amount, at + 1),
                    Some(K::ItemName { name }) => match kind_at(at + 1) {
                        Some(K::Continued { amount }) => (name, *amount, at + 2),
                        _ => return Err(Bad),
                    },
                    _ => return Err(Bad),
                };
                // 戻の印を持たない負の明細、数量 × 単価 ≠ 金額
                if amount < 0 || unit_price.is_some_and(|p| quantity.checked_mul(p) != Some(amount))
                {
                    return Err(Bad);
                }
                items.push(EjItem {
                    line_no: region[at].line_no,
                    name: name.clone(),
                    quantity,
                    unit_price,
                    amount,
                });
                saw_item = true;
                previous = Previous::Item;
                i = next;
                continue;
            }
            // IO-08-D6d: 空のモードで、戻の印の直後の負の明細は数量 -1
            K::ReturnMark => {
                let Some(K::Item { name, amount }) = kind_at(i + 1) else {
                    return Err(Bad);
                };
                if *mode != EjMode::Normal || *amount >= 0 {
                    return Err(Bad);
                }
                items.push(EjItem {
                    line_no: region[i + 1].line_no,
                    name: name.clone(),
                    quantity: -1,
                    unit_price: None,
                    amount: *amount,
                });
                saw_item = true;
                previous = Previous::ReturnItem;
                i += 2;
                continue;
            }
            // IO-08-D6a: 直前の項目の取消（負 = 明細、正 = ％値引き）
            K::Correction { amount } => {
                let cancels =
                    |target: Option<i64>| target.and_then(i64::checked_neg) == Some(*amount);
                match previous {
                    Previous::Item if *amount < 0 && cancels(items.last().map(|it| it.amount)) => {
                        items.pop();
                    }
                    Previous::PercentDiscount
                        if *amount > 0 && cancels(adjustments.last().map(|a| a.amount)) =>
                    {
                        adjustments.pop();
                    }
                    _ => return Err(Bad),
                }
                previous = Previous::Other;
            }
            // IO-08-D6b: 小計 = それまでの有効な明細と値引きの和
            K::Subtotal { amount } => {
                if amount_sum(&items, &adjustments) != Some(*amount) {
                    return Err(Bad);
                }
                previous = Previous::Subtotal;
            }
            // IO-08-D6b: 率の行 + ％値引き。`*` つきは小計値引き、`*` なしは明細値引き
            K::DiscountRate => {
                let Some(K::PercentDiscount { amount, subtotal }) = kind_at(i + 1) else {
                    return Err(Bad);
                };
                let kind = match (subtotal, &previous) {
                    (true, Previous::Subtotal) => EjAdjustmentKind::SubtotalDiscount,
                    (false, Previous::Item) => EjAdjustmentKind::ItemDiscount {
                        item_line_no: items.last().ok_or(Bad)?.line_no,
                    },
                    _ => return Err(Bad),
                };
                if *amount >= 0 {
                    return Err(Bad);
                }
                adjustments.push(EjAdjustment {
                    line_no: region[i + 1].line_no,
                    kind,
                    amount: *amount,
                });
                previous = Previous::PercentDiscount;
                i += 2;
                continue;
            }
            // IO-08-D6b: マイナスキーは直前の有効な明細（戻の明細を除く）の後で、金額は負
            K::MinusKey { amount } => {
                if previous != Previous::Item || *amount >= 0 {
                    return Err(Bad);
                }
                adjustments.push(EjAdjustment {
                    line_no: region[i].line_no,
                    kind: EjAdjustmentKind::MinusKey,
                    amount: *amount,
                });
                previous = Previous::Other;
            }
            _ => return Err(Bad),
        }
        i += 1;
    }
    if previous == Previous::Subtotal {
        return Err(Bad);
    }
    Ok((items, adjustments, saw_item))
}

/// 同じ種類の行が 2 つ以上あれば照合できない
fn at_most_one<T>(mut values: impl Iterator<Item = T>) -> Result<Option<T>, EjDiagnosticCode> {
    let first = values.next();
    if values.next().is_some() {
        return Err(EjDiagnosticCode::InconsistentRecord);
    }
    Ok(first)
}

/// IO-08-D6c: 照合の金額。`合  計` の行があればその金額、無ければ合計域の訂正で
/// 取り消されていない支払行がちょうど 1 行のときの金額
fn reconciled_total(totals: &[EjLine]) -> Result<i64, EjDiagnosticCode> {
    use EjDiagnosticCode::{IncompleteRecord, InconsistentRecord};

    // ラベルの行が金額 token を持たなければ、直後の Continued の金額を値とする
    let amount_at = |index: usize| -> Result<i64, EjDiagnosticCode> {
        let line = &totals[index];
        let EjLineKind::Labeled { label } = line.kind else {
            return Err(InconsistentRecord);
        };
        let rest = line.text[label.len()..].trim();
        if !rest.is_empty() {
            return parse_amount(rest)
                .map(|(amount, _)| amount)
                .ok_or(InconsistentRecord);
        }
        match totals.get(index + 1).map(|next| &next.kind) {
            Some(EjLineKind::Continued { amount }) => Ok(*amount),
            _ => Err(InconsistentRecord),
        }
    };

    let mut total_at = None;
    let mut payments: Vec<(usize, bool)> = Vec::new(); // (行, 取消済み)
    for (index, line) in totals.iter().enumerate() {
        match line.kind {
            EjLineKind::Labeled { label: "合  計" } => {
                if total_at.replace(index).is_some() {
                    return Err(InconsistentRecord);
                }
            }
            EjLineKind::Labeled { label } if PAYMENT_LABELS.contains(&label) => {
                payments.push((index, false));
            }
            // IO-08-D6a: 直前の行（折り返していれば Continued の前）の支払行の取消
            EjLineKind::Correction { amount } => {
                let mut target = index.checked_sub(1).ok_or(InconsistentRecord)?;
                if matches!(totals[target].kind, EjLineKind::Continued { .. }) {
                    target = target.checked_sub(1).ok_or(InconsistentRecord)?;
                }
                let payment = payments
                    .iter_mut()
                    .find(|(at, cancelled)| *at == target && !*cancelled)
                    .ok_or(InconsistentRecord)?;
                if amount_at(target)?.checked_neg() != Some(amount) {
                    return Err(InconsistentRecord);
                }
                payment.1 = true;
            }
            _ => {}
        }
    }

    if let Some(index) = total_at {
        return amount_at(index);
    }
    let mut live = payments.iter().filter(|(_, cancelled)| !cancelled);
    match (live.next(), live.next()) {
        (None, _) => Err(IncompleteRecord),
        (Some((index, _)), None) => amount_at(*index),
        _ => Err(InconsistentRecord),
    }
}

/// 取引・取引中止の明細域（IO-08-D5a / D5c）。prev は直前の行、next は明細域の直後の行。
/// cancel_last は取引中止の記録の最後の行（番号印字の行を除く）
fn classify_items(
    text: &str,
    prev: Option<&EjLine>,
    next: Option<&str>,
    cancel_last: bool,
) -> EjLineKind {
    if is_mark(text, "戻") {
        EjLineKind::ReturnMark
    } else if cancel_last && is_mark(text, "取引中止") {
        EjLineKind::CancelMark
    } else if is_discount_rate(text) {
        EjLineKind::DiscountRate
    } else if let Some(amount) = parse_labeled_amount(text, "訂正") {
        EjLineKind::Correction { amount }
    } else if let Some(amount) = parse_labeled_amount(text, "小計") {
        EjLineKind::Subtotal { amount }
    } else if let Some((amount, subtotal)) = parse_starred_amount(text, "％－") {
        EjLineKind::PercentDiscount { amount, subtotal }
    } else if let Some((amount, _)) = parse_starred_amount(text, "－") {
        EjLineKind::MinusKey { amount }
    } else if let Some((quantity, unit_price)) = parse_quantity(text) {
        EjLineKind::Quantity {
            quantity,
            unit_price,
        }
    } else if let Some((name, amount)) = parse_item(text) {
        EjLineKind::Item { name, amount }
    } else if !text.starts_with(char::is_whitespace)
        && next.and_then(item_continued).is_some()
        && !item_token_overflows(text)
        && !is_other_line_shape(text)
    {
        EjLineKind::ItemName {
            name: text.trim_end().to_string(),
        }
    } else if let Some(amount) = prev
        .filter(|prev| matches!(prev.kind, EjLineKind::ItemName { .. }))
        .and_then(|_| item_continued(text))
    {
        EjLineKind::Continued { amount }
    } else {
        EjLineKind::Unknown
    }
}

/// 明細域の続きの行の形: 空白で始まり、空白除去後が通貨記号つきの金額 token だけ
fn item_continued(text: &str) -> Option<i64> {
    if !text.starts_with(' ') {
        return None;
    }
    match parse_amount(text.trim())? {
        (amount, true) => Some(amount),
        _ => None,
    }
}

/// 表でほかの行種として形が定まった行（ItemName にしない。IO-08-D5a）。
/// Text / AmountOnly / Unknown は形が定まらないので含めない
fn is_other_line_shape(text: &str) -> bool {
    is_mark(text, "取引中止")
        || is_number_print(text)
        || parse_status(text).is_some()
        || labeled(text, &TOTAL_LABELS).is_some()
        || labeled(text, &PAID_LABELS).is_some()
        || labeled(text, &REPORT_LABELS).is_some()
        || text.trim() == REDUCED_TAX_NOTE
        || receipt_label(text).is_some()
        || report_title(text).is_some()
        || ["日計明細", "ＰＬＵ", "勤怠"].contains(&text.trim())
        || is_training_display(text)
}

/// 最後の語が金額 token の形（末尾の `※` / `*` を除く）で、値が i64 に収まらない行（ItemName にしない）
fn item_token_overflows(text: &str) -> bool {
    text.trim_end().rsplit_once(' ').is_some_and(|(_, token)| {
        let token = token.strip_suffix(['※', '*']).unwrap_or(token);
        matches!(amount_token(token), Some((None, _)))
    })
}

/// `戻` / `取引中止` + 半角空白 1 個以上 + `･` 1 個以上 + 空白
fn is_mark(text: &str, label: &str) -> bool {
    let Some(rest) = text.strip_prefix(label) else {
        return false;
    };
    let dots = rest.trim_start_matches(' ');
    dots.len() < rest.len()
        && dots.starts_with('･')
        && dots.trim_start_matches('･').trim().is_empty()
}

/// `-` + ASCII 数字 1 個以上 + `%` で始まり、残りが空白
fn is_discount_rate(text: &str) -> bool {
    text.strip_prefix('-')
        .and_then(|rest| rest.split_once('%'))
        .is_some_and(|(digits, rest)| {
            !digits.is_empty()
                && digits.bytes().all(|b| b.is_ascii_digit())
                && rest.trim().is_empty()
        })
}

/// ラベル + 空白 + 金額 token + 任意の `*`。`*` の有無を返す
fn parse_starred_amount(text: &str, label: &str) -> Option<(i64, bool)> {
    let rest = text.strip_prefix(label)?;
    if !rest.starts_with(' ') {
        return None;
    }
    let token = rest.trim();
    let (token, starred) = match token.strip_suffix('*') {
        Some(token) => (token, true),
        None => (token, false),
    };
    parse_amount(token).map(|(amount, _)| (amount, starred))
}

/// 取引の合計域（IO-08-D5a / D5b）。prev は直前の行
fn classify_totals(text: &str, prev: Option<&EjLine>) -> EjLineKind {
    // 直前がラベルだけの行（軽減税率の注記を除く）か `対象計` の率だけの行なら続きの行になれる
    let after_bare_label = prev.is_some_and(|prev| match prev.kind {
        EjLineKind::Labeled { label } if label != REDUCED_TAX_NOTE => prev
            .text
            .strip_prefix(label)
            .map(str::trim)
            .is_some_and(|rest| rest.is_empty() || (label.starts_with("対象計") && is_rate(rest))),
        _ => false,
    });
    if let Some(count) = parse_item_count(text) {
        EjLineKind::ItemCount { count }
    } else if let Some(label) = labeled(text, &TOTAL_LABELS) {
        EjLineKind::Labeled { label }
    } else if text.trim() == REDUCED_TAX_NOTE {
        EjLineKind::Labeled {
            label: REDUCED_TAX_NOTE,
        }
    } else if let Some(amount) = (after_bare_label && text.starts_with(' '))
        .then(|| parse_amount(text.trim()))
        .flatten()
    {
        EjLineKind::Continued { amount: amount.0 }
    } else if let Some(amount) = parse_labeled_amount(text, "訂正") {
        EjLineKind::Correction { amount }
    } else if is_number_print(text) {
        EjLineKind::NumberPrint
    } else {
        EjLineKind::Unknown
    }
}

/// `対象計` の率: ASCII 数字 1 個以上 + 任意の（`.` + ASCII 数字 1 個以上）+ `%`
fn is_rate(text: &str) -> bool {
    text.strip_suffix('%').is_some_and(|rate| {
        let (whole, fraction) = rate.split_once('.').unwrap_or((rate, "0"));
        parse_count(whole).is_some() && parse_count(fraction).is_some()
    })
}

/// 精算票・点検票（題の無い精算・点検のモードの記録を含む）の行（IO-08-D5d）
fn classify_report(raw: &RawLine, report: Option<EjSettlementReport>, first: bool) -> EjLineKind {
    let text = raw.text.as_str();
    if raw.bytes == SEPARATOR {
        EjLineKind::Separator
    } else if report.is_some_and(|report| text.trim() == report_name(report)) {
        EjLineKind::ReportEnd
    } else if let Some(title) = first.then(|| report_title(text)).flatten() {
        title
    } else if let Some(kind) = parse_status(text) {
        kind
    } else if let Some(label) = labeled(text, &REPORT_LABELS) {
        EjLineKind::Labeled { label }
    } else if text.starts_with(' ') && parse_amount(text.trim()).is_some() {
        EjLineKind::AmountOnly
    } else if is_number_print(text) {
        EjLineKind::NumberPrint
    } else if !text.starts_with(char::is_whitespace)
        && !text.contains(|c: char| c.is_ascii_digit() || ('０'..='９').contains(&c))
        && !has_currency(text)
    {
        EjLineKind::Text
    } else {
        EjLineKind::Unknown
    }
}

/// 設定の行（IO-08-D5d）。通貨記号の行は行種の分類より前に Unknown にする。
/// 判定は decode 後の文字列で行う（2 byte 目が 0x5C の漢字を通貨記号と誤らない）
fn classify_settings(raw: &RawLine) -> EjLineKind {
    let text = raw.text.as_str();
    if has_currency(text) {
        EjLineKind::Unknown
    } else if raw.bytes == SEPARATOR {
        EjLineKind::Separator
    } else if let Some(kind) = parse_status(text) {
        kind
    } else if is_number_print(text) {
        EjLineKind::NumberPrint
    } else if !text.contains(" 点") {
        EjLineKind::Text
    } else {
        EjLineKind::Unknown
    }
}

fn has_currency(text: &str) -> bool {
    text.contains(['\\', '￥'])
}

/// 空白除去後が `＃` + 半角空白 1 個以上 + ASCII 数字 1 個以上
fn is_number_print(text: &str) -> bool {
    text.trim()
        .strip_prefix("＃ ")
        .map(str::trim_start)
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// 領収書の 3 行（`一連No.N` / `領収No.N` / `領収書 金額`）。値は解釈しない
fn receipt_label(text: &str) -> Option<&'static str> {
    let text = text.trim();
    let numbered = |label: &str| {
        text.strip_prefix(label)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    };
    if numbered("一連No.") {
        Some("一連No.")
    } else if numbered("領収No.") {
        Some("領収No.")
    } else if text
        .strip_prefix("領収書 ")
        .is_some_and(|rest| parse_amount(rest.trim()).is_some())
    {
        Some("領収書")
    } else {
        None
    }
}

fn report_name(report: EjSettlementReport) -> &'static str {
    match report {
        EjSettlementReport::Daily => "日計明細",
        EjSettlementReport::Plu => "ＰＬＵ",
        EjSettlementReport::Attendance => "勤怠",
    }
}

fn is_title_no(word: &str) -> bool {
    word.len() == 4 && word.bytes().all(|b| b.is_ascii_digit())
}

/// Z の題: `NNNN 帳票名 [任意の語] Z NNNN`。両端の数字は意味を解釈しない
fn parse_z_title(text: &str) -> Option<(EjSettlementReport, String, String)> {
    let words: Vec<_> = text.split_whitespace().collect();
    let [leading, name, .., "Z", trailing] = words.as_slice() else {
        return None;
    };
    let report = match *name {
        "日計明細" => EjSettlementReport::Daily,
        "ＰＬＵ" => EjSettlementReport::Plu,
        "勤怠" => EjSettlementReport::Attendance,
        _ => return None,
    };
    (is_title_no(leading) && is_title_no(trailing))
        .then(|| (report, leading.to_string(), trailing.to_string()))
}

/// X の題: `[NNNN] 帳票名 X` か `X` だけ。先頭の数字を返す
fn parse_x_title(text: &str) -> Option<Option<String>> {
    let words: Vec<_> = text.split_whitespace().collect();
    let (leading, name) = match words.as_slice() {
        ["X"] => return Some(None),
        [leading, name, "X"] if is_title_no(leading) => (Some(leading.to_string()), *name),
        [name, "X"] => (None, *name),
        _ => return None,
    };
    ["日計明細", "ＰＬＵ", "在売点検"]
        .contains(&name)
        .then_some(leading)
}

fn report_title(text: &str) -> Option<EjLineKind> {
    if let Some((_, leading, trailing)) = parse_z_title(text) {
        return Some(EjLineKind::ReportTitle {
            leading_no: Some(leading),
            trailing_no: Some(trailing),
        });
    }
    parse_x_title(text).map(|leading_no| EjLineKind::ReportTitle {
        leading_no,
        trailing_no: None,
    })
}

/// ラベルで始まり、その直後が空白の行
fn labeled(text: &str, labels: &[&'static str]) -> Option<&'static str> {
    labels.iter().copied().find(|label| {
        text.strip_prefix(label)
            .is_some_and(|rest| rest.starts_with(' '))
    })
}

fn labeled_kind(text: &str, labels: &[&'static str]) -> EjLineKind {
    labeled(text, labels).map_or(EjLineKind::Unknown, |label| EjLineKind::Labeled { label })
}

/// 空白 + 1 以上の整数 + ` 点` + 空白 + `@` + 通貨記号なし・符号（`-` / `－`）なしの単価 + 空白。
/// 数量 0 は Σ数量にも Σ金額にも寄与せず照合を素通りするため受理しない
fn parse_quantity(text: &str) -> Option<(i64, i64)> {
    if !text.starts_with(' ') {
        return None;
    }
    let (quantity, rest) = text.trim().split_once(" 点")?;
    let price = rest.strip_prefix(' ')?.trim_start().strip_prefix('@')?;
    match (parse_count(quantity)?, parse_amount(price)?) {
        (quantity @ 1.., (unit_price, false)) if !price.starts_with(['-', '－']) => {
            Some((quantity, unit_price))
        }
        _ => None,
    }
}

/// 空白 + 整数（任意の半角 `-`）+ ` 点` + 空白
fn parse_item_count(text: &str) -> Option<i64> {
    if !text.starts_with(' ') {
        return None;
    }
    let count = text.trim().strip_suffix(" 点")?;
    match count.strip_prefix('-') {
        Some(digits) => parse_count(digits)?.checked_neg(),
        None => parse_count(count),
    }
}

/// ラベル + 空白 + 金額 token だけの行（`訂正` / `小計`）
fn parse_labeled_amount(text: &str, label: &str) -> Option<i64> {
    let rest = text.strip_prefix(label)?;
    if !rest.starts_with(' ') {
        return None;
    }
    parse_amount(rest.trim()).map(|(amount, _)| amount)
}

/// 名称 + 空白 + 通貨記号つきの金額 token。名称は空白でない文字で始まり、中の空白を許す
fn parse_item(text: &str) -> Option<(String, i64)> {
    if text.starts_with(char::is_whitespace) {
        return None;
    }
    let (name, token) = text.trim_end().rsplit_once(' ')?;
    let name = name.trim_end();
    // `※`（軽減税率の印）は通貨記号つきの金額の後だけ。解釈しない
    let (token, marked) = match token.strip_suffix('※') {
        Some(token) => (token, true),
        None => (token, false),
    };
    match parse_amount(token)? {
        _ if name.is_empty() => None,
        (amount, true) => Some((name.to_string(), amount)),
        // 通貨記号なしの負の金額（戻の印の明細）
        (amount, false) if amount < 0 && !marked => Some((name.to_string(), amount)),
        _ => None,
    }
}

/// 結果欄は文字列のまま保持し、成否を判断しない
fn parse_status(text: &str) -> Option<EjLineKind> {
    STATUS_LABELS.iter().copied().find_map(|label| {
        text.strip_prefix(label).map(|rest| EjLineKind::Status {
            label,
            result: rest.trim().to_string(),
        })
    })
}

fn parse_count(text: &str) -> Option<i64> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// 金額 token: 任意の符号（`-` / `－`）、任意の通貨記号（`\` / `￥`）、数字と桁区切り。
/// 符号・通貨記号・数字・桁区切りの幅（半角 / 全角）が token 内でそろわなければ受理しない。
/// 値と、通貨記号があったかを返す
fn parse_amount(token: &str) -> Option<(i64, bool)> {
    let (value, has_currency) = amount_token(token)?;
    Some((value?, has_currency))
}

/// 金額 token の形なら Some。値が i64 に収まらなければ値は None
fn amount_token(token: &str) -> Option<(Option<i64>, bool)> {
    // 負値は負のまま蓄積する（i64::MIN の絶対値は i64 に収まらない）。
    // 符号・通貨記号・数字・桁区切りの幅（半角 / 全角）を token 内でそろえる
    let (sign, mut wide, rest) = if let Some(rest) = token.strip_prefix('-') {
        (-1, Some(false), rest)
    } else if let Some(rest) = token.strip_prefix('－') {
        (-1, Some(true), rest)
    } else {
        (1, None, token)
    };
    let (currency, rest) = if let Some(rest) = rest.strip_prefix('\\') {
        (Some(false), rest)
    } else if let Some(rest) = rest.strip_prefix('￥') {
        (Some(true), rest)
    } else {
        (None, rest)
    };
    if let Some(is_wide) = currency {
        if *wide.get_or_insert(is_wide) != is_wide {
            return None;
        }
    }
    let has_currency = currency.is_some();

    let mut value: Option<i64> = Some(0);
    let mut after_digit = false;
    for c in rest.chars() {
        let (digit, is_wide) = match c {
            '0'..='9' | ',' => (c.to_digit(10), false),
            '０'..='９' => (Some(c as u32 - '０' as u32), true),
            '，' => (None, true),
            _ => return None,
        };
        if *wide.get_or_insert(is_wide) != is_wide {
            return None;
        }
        match digit {
            Some(d) => {
                value = value.and_then(|v| v.checked_mul(10)?.checked_add(sign * i64::from(d)));
                after_digit = true;
            }
            // 桁区切りは数字の後だけ（先頭・連続を受理しない）
            None if after_digit => after_digit = false,
            None => return None,
        }
    }
    if !after_digit {
        return None;
    }
    Some((value, has_currency))
}

/// IO-08-D8: code ごとの範囲と固定文言
fn diagnostic(code: EjDiagnosticCode, line_no: Option<usize>) -> EjDiagnostic {
    use EjDiagnosticCode::*;
    let (scope, message) = match code {
        UnknownLine => (
            EjDiagnosticScope::Line,
            "解釈できない行があります。この記録は明細を復元できません",
        ),
        InvalidWidth => (
            EjDiagnosticScope::Line,
            "行の長さが24バイトではありません。この記録は明細を復元できません",
        ),
        UnrecognizedMode => (
            EjDiagnosticScope::Record,
            "未知のモードの記録です。この記録は明細を復元できません",
        ),
        InconsistentRecord => (
            EjDiagnosticScope::Record,
            "取引内の数量・点数・合計が一致しません。この記録は明細を復元できません",
        ),
        IncompleteRecord => (
            EjDiagnosticScope::Record,
            "記録に必須の行がありません。この記録は明細を復元できません",
        ),
        MissingFinalNewline => (
            EjDiagnosticScope::Line,
            "ファイルの最後の行に改行がありません。最後の記録は途中で切れたものとして扱います",
        ),
        LeadingFragment => (
            EjDiagnosticScope::File,
            "最初の記録より前に行があります。ファイルは記録の途中から始まっています",
        ),
    };
    EjDiagnostic {
        line_no: if scope == EjDiagnosticScope::File {
            None
        } else {
            line_no
        },
        code,
        scope,
        message,
    }
}

// ===========================================================================
// テスト
// ===========================================================================

/// 合成の EJ の組立て（test だけ。IO-08 の tests と BIZ-08 の EJ の取込み・L3 の入力物が使う。TD-296）
#[cfg(test)]
pub(crate) mod synthetic {
    use super::*;

    // fixture の名称・金額・日時・番号はすべて架空の値
    pub(crate) const NAME_A: &str = "ｺﾞｳｾｲｱﾒ";
    pub(crate) const NAME_B: &str = "ｶｸｳｸｯｷｰ";
    pub(crate) const AT: &str = "2030-01-02 10:15";

    pub(crate) fn sjis(text: &str) -> Vec<u8> {
        let (bytes, _, unmappable) = SHIFT_JIS.encode(text);
        assert!(!unmappable, "fixture に CP932 で表せない文字があります");
        bytes.into_owned()
    }

    /// 24 バイトまで右を空白で埋める。超えたら fixture の誤りとして panic する
    pub(crate) fn row(text: &str) -> Vec<u8> {
        let mut bytes = sjis(text);
        assert!(
            bytes.len() <= LINE_WIDTH,
            "fixture の行が24バイトを超えています: {text}"
        );
        bytes.resize(LINE_WIDTH, b' ');
        bytes
    }

    pub(crate) fn rows(lines: &[String]) -> Vec<Vec<u8>> {
        lines.iter().map(|line| row(line)).collect()
    }

    /// 各行を CRLF で終える
    pub(crate) fn crlf(rows: Vec<Vec<u8>>) -> Vec<u8> {
        rows.into_iter()
            .flat_map(|mut bytes| {
                bytes.extend_from_slice(b"\r\n");
                bytes
            })
            .collect()
    }

    pub(crate) fn file(lines: &[String]) -> Vec<u8> {
        crlf(rows(lines))
    }

    pub(crate) fn parse(lines: &[String]) -> EjParseResult {
        parse_ej(&file(lines)).expect("Ok を期待しました")
    }

    /// 左と右の間を空白で埋めて 24 バイトにする
    pub(crate) fn lr(left: &str, right: &str) -> String {
        let width = sjis(left).len() + sjis(right).len();
        assert!(
            width <= LINE_WIDTH,
            "fixture の行が24バイトを超えています: {left}{right}"
        );
        format!("{left}{}{right}", " ".repeat(LINE_WIDTH - width))
    }

    /// 半角の `\`・数字・`,` を全角の `￥`・数字・`，` にする
    pub(crate) fn zen(ascii: &str) -> String {
        ascii
            .chars()
            .map(|c| match c {
                '\\' => '￥',
                ',' => '，',
                '0'..='9' => char::from_u32('０' as u32 + c.to_digit(10).unwrap()).unwrap(),
                other => other,
            })
            .collect()
    }

    /// `\1,234` / `-\100`
    pub(crate) fn yen(amount: i64) -> String {
        let digits = amount.abs().to_string();
        let mut grouped = String::new();
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                grouped.push(',');
            }
            grouped.push(c);
        }
        format!("{}\\{grouped}", if amount < 0 { "-" } else { "" })
    }

    pub(crate) fn header(mode: &str, at: &str, number: &str) -> Vec<String> {
        vec![
            format!("{mode}{}{at}", " ".repeat(5 - sjis(mode).len())),
            format!("{}0001-{number}", " ".repeat(13)),
        ]
    }

    pub(crate) fn sep() -> String {
        "-".repeat(LINE_WIDTH)
    }

    pub(crate) fn item(name: &str, amount: i64) -> String {
        lr(name, &yen(amount))
    }

    pub(crate) fn qty(quantity: i64, unit_price: &str) -> String {
        format!("  {quantity} 点   @{unit_price}")
    }

    pub(crate) fn count(n: i64) -> String {
        lr("", &format!("{n} 点"))
    }

    /// 合計域の全角数字の行
    pub(crate) fn wide(label: &str, amount: i64) -> String {
        lr(label, &zen(&yen(amount)))
    }

    /// 区切りから後（点数・対象計・内税・合計・お預り・お釣）
    pub(crate) fn totals(n: i64, total: i64) -> Vec<String> {
        let tendered = (total / 1000 + 1) * 1000;
        vec![
            sep(),
            count(n),
            lr("対象計", &yen(total)),
            lr("内税", &yen(total / 11)),
            wide("合  計", total),
            wide("お預り", tendered),
            wide("お  釣", tendered - total),
        ]
    }

    pub(crate) fn sale(
        mode: &str,
        number: &str,
        items: &[String],
        n: i64,
        total: i64,
    ) -> Vec<String> {
        [header(mode, AT, number), items.to_vec(), totals(n, total)].concat()
    }

    pub(crate) fn ok_sale(number: &str) -> Vec<String> {
        sale("", number, &[item(NAME_A, 120), item(NAME_B, 380)], 2, 500)
    }
}

#[cfg(test)]
mod tests {
    use super::synthetic::*;
    use super::*;
    use EjDiagnosticCode::*;

    fn settlement_body() -> Vec<String> {
        vec![
            "0007 日計明細  Z 0003".to_string(),
            lr("総売", "12"),
            lr("", &yen(4_560)),
            lr("純売", "12"),
            lr("", &yen(4_560)),
            lr("現金在高", &yen(4_560)),
            lr("純客", "5"),
            sep(),
            "日計明細".to_string(),
            lr("SDｶｰﾄﾞ保存", "正常終了"),
            lr("ｽﾏ-ﾄﾌｫﾝ送信", "正常終了"),
        ]
    }

    fn settlement(number: &str) -> Vec<String> {
        [
            header("精算", "2030-01-02 19:00", number),
            settlement_body(),
        ]
        .concat()
    }

    fn program(at: &str, number: &str) -> Vec<String> {
        [
            header("PGM", at, number),
            vec![sep(), lr("SD設定書込み", "正常終了"), sep()],
        ]
        .concat()
    }

    fn items_of(record: &EjRecord) -> &[EjItem] {
        match &record.restoration {
            EjRestoration::Restored { items, .. } => items,
            other => panic!("Restored を期待しました: {other:?}"),
        }
    }

    fn reasons_of(record: &EjRecord) -> &[EjDiagnosticCode] {
        match &record.restoration {
            EjRestoration::Unresolved { reasons } => reasons,
            other => panic!("Unresolved を期待しました: {other:?}"),
        }
    }

    /// (行番号, code, 範囲) の列
    fn diags(result: &EjParseResult) -> Vec<(Option<usize>, EjDiagnosticCode, EjDiagnosticScope)> {
        result
            .diagnostics
            .iter()
            .map(|d| (d.line_no, d.code, d.scope))
            .collect()
    }

    // -----------------------------------------------------------------------
    // 正常系
    // -----------------------------------------------------------------------

    // IO-08-D3 / D5 / D6: 通常販売の明細を復元し、日時と番号を文字列のまま返す
    #[test]
    fn parse_ej_normal_sale_restores_items() {
        let result = parse(&sale(
            "",
            "000123",
            &[item(NAME_A, 120), item(NAME_B, 1_380)],
            2,
            1_500,
        ));

        assert!(result.diagnostics.is_empty());
        assert!(result.leading_lines.is_empty());
        assert_eq!(result.records.len(), 1);
        let record = &result.records[0];
        assert_eq!(record.header_line_no, 1);
        assert_eq!(record.mode, EjMode::Normal);
        assert_eq!(record.printed_at, AT);
        assert_eq!(record.number_prefix, "0001");
        assert_eq!(record.number, "000123");
        assert_eq!(
            record.restoration,
            EjRestoration::Restored {
                items: vec![
                    EjItem {
                        line_no: 3,
                        name: NAME_A.to_string(),
                        quantity: 1,
                        unit_price: None,
                        amount: 120,
                    },
                    EjItem {
                        line_no: 4,
                        name: NAME_B.to_string(),
                        quantity: 1,
                        unit_price: None,
                        amount: 1_380,
                    },
                ],
                item_count: 2,
                adjustments: vec![],
            }
        );
    }

    // IO-08-D6: 数量行は直後の 1 明細だけに掛かる
    #[test]
    fn parse_ej_quantity_line_applies_to_next_item() {
        let result = parse(&sale(
            "",
            "000124",
            &[qty(3, "120"), item(NAME_A, 360), item(NAME_B, 200)],
            4,
            560,
        ));

        assert!(result.diagnostics.is_empty());
        let items = items_of(&result.records[0]);
        assert_eq!(items.len(), 2);
        assert_eq!((items[0].quantity, items[0].unit_price), (3, Some(120)));
        assert_eq!(items[0].amount, 360);
        assert_eq!((items[1].quantity, items[1].unit_price), (1, None));
    }

    // IO-08-D6: 同じ名称の行を合算しない
    #[test]
    fn parse_ej_repeated_same_name_lines_kept_separate() {
        let result = parse(&sale(
            "",
            "000125",
            &[item(NAME_A, 100), item(NAME_A, 100)],
            2,
            200,
        ));

        assert!(result.diagnostics.is_empty());
        let items = items_of(&result.records[0]);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.name == NAME_A && i.quantity == 1));
        assert_eq!((items[0].line_no, items[1].line_no), (3, 4));
    }

    // IO-08-D3 / D6: 返品モードは Return で、金額の符号を反転しない
    #[test]
    fn parse_ej_return_mode_keeps_positive_amounts() {
        let result = parse(&sale("戻", "000126", &[item(NAME_A, 500)], 1, 500));

        assert!(result.diagnostics.is_empty());
        let record = &result.records[0];
        assert_eq!(record.mode, EjMode::Return);
        let items = items_of(record);
        assert_eq!(items[0].amount, 500);
        assert_eq!(items[0].quantity, 1);
    }

    // IO-08-D6: 合計の無い現金ちょうどの取引は現金の金額で照合し、現金行を明細にしない
    #[test]
    fn parse_ej_exact_cash_tender_without_total_line() {
        let lines = [
            header("", AT, "000127"),
            vec![
                item(NAME_A, 300),
                item(NAME_B, 700),
                sep(),
                count(2),
                lr("対象計", &yen(1_000)),
                lr("内税", &yen(90)),
                wide("現金", 1_000),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        let items = items_of(&result.records[0]);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.name != "現金"));
    }

    // IO-08-D5: 区切りの前はラベルで始まる名称も明細
    #[test]
    fn parse_ej_item_named_like_label_before_separator_is_item() {
        let result = parse(&sale(
            "",
            "000128",
            &[item("現金 ﾄﾚｲ", 300), item(NAME_A, 200)],
            2,
            500,
        ));

        assert!(result.diagnostics.is_empty());
        let items = items_of(&result.records[0]);
        assert_eq!(items[0].name, "現金 ﾄﾚｲ");
        assert_eq!(items[0].amount, 300);
    }

    // IO-08-D7: 入金 / 出金 / 替は明細なし
    #[test]
    fn parse_ej_non_item_records_paid_in_paid_out_exchange() {
        let exchange = "替 ﾘｮｳｶﾞｴ".to_string();
        assert!(!exchange
            .chars()
            .any(|c| c.is_ascii_digit() || c == '\\' || c == '￥'));
        let lines = [
            header("", AT, "000130"),
            vec![wide("入金", 5_000)],
            header("", AT, "000131"),
            vec![wide("出金", 1_200)],
            header("", AT, "000132"),
            vec![exchange],
            ok_sale("000133"),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        for (record, label) in result.records.iter().zip(["入金", "出金", "替"]) {
            assert_eq!(record.restoration, EjRestoration::NoItems);
            assert_eq!(record.body[0].kind, EjLineKind::Labeled { label });
        }
        assert_eq!(items_of(&result.records[3]).len(), 2);
    }

    // IO-08-D5 / D7: 設定書込みと精算（返品だけの日の負値を含む）は明細なし
    #[test]
    fn parse_ej_settlement_and_program_records() {
        let return_only_day = [
            header("精算", "2030-01-03 19:00", "000142"),
            vec![
                "0008 日計明細  Z 0004".to_string(),
                lr("総売", "-1"),
                lr("", "-980"),
                lr("純売", "-1"),
                lr("", "-980"),
                lr("現金在高", "-980"),
                lr("対象計", "-980"),
                lr("内税", "-89"),
                lr("消費税合計", "-89"),
                lr("純客", "-1"),
                sep(),
                "日計明細".to_string(),
                lr("SDｶｰﾄﾞ保存", "正常終了"),
            ],
        ]
        .concat();
        let lines = [program(AT, "000140"), settlement("000141"), return_only_day].concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert!(result
            .records
            .iter()
            .all(|r| r.restoration == EjRestoration::NoItems));
        let modes: Vec<_> = result.records.iter().map(|r| r.mode.clone()).collect();
        assert_eq!(
            modes,
            [EjMode::Program, EjMode::Settlement, EjMode::Settlement]
        );
        let kinds: Vec<_> = result.records.iter().map(|r| r.kind.clone()).collect();
        let daily = EjRecordKind::Settlement {
            report: EjSettlementReport::Daily,
            completed: true,
        };
        assert_eq!(kinds, [EjRecordKind::Settings, daily.clone(), daily]);
        assert_eq!(
            result.records[1].body[0].kind,
            EjLineKind::ReportTitle {
                leading_no: Some("0007".to_string()),
                trailing_no: Some("0003".to_string()),
            }
        );
        assert_eq!(
            result.records[0].body[1].kind,
            EjLineKind::Status {
                label: "SD設定書込み",
                result: "正常終了".to_string(),
            }
        );
        let return_body = &result.records[2].body;
        assert_eq!(return_body[2].kind, EjLineKind::AmountOnly);
        assert_eq!(return_body[7].kind, EjLineKind::Labeled { label: "内税" });
    }

    // IO-08-D3 / D9: 1 file 内の複数精算と前日付の先頭記録をそのまま順に返す
    #[test]
    fn parse_ej_multiple_settlements_in_one_file_and_pre_dated_first_record() {
        let lines = [
            program("2030-01-01 23:50", "000100"),
            ok_sale("000101"),
            settlement("000102"),
            ok_sale("000103"),
            settlement("000104"),
            ok_sale("000105"),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        let numbers: Vec<_> = result.records.iter().map(|r| r.number.as_str()).collect();
        assert_eq!(
            numbers,
            ["000100", "000101", "000102", "000103", "000104", "000105"]
        );
        assert_eq!(result.records[0].printed_at, "2030-01-01 23:50");
        assert_eq!(result.records[0].restoration, EjRestoration::NoItems);
        assert_eq!(items_of(&result.records[5]).len(), 2);
    }

    // IO-08-D6: 金額 token の字形
    #[test]
    fn parse_ej_amount_formats() {
        assert_eq!(parse_amount("\\1,234"), Some((1_234, true)));
        assert_eq!(parse_amount("￥１２，３４５"), Some((12_345, true)));
        assert_eq!(parse_amount("-980"), Some((-980, false)));
        assert_eq!(parse_quantity("  1 点   @1,200"), Some((1, 1_200)));
        for mixed in [
            "￥1,234", "\\１２", "1，234", "\\1,,234", "\\,100", "\\100,",
        ] {
            assert_eq!(parse_amount(mixed), None, "受理してはいけない token");
        }

        let result = parse(&sale(
            "",
            "000150",
            &[qty(2, "1,200"), item(NAME_A, 2_400)],
            2,
            2_400,
        ));
        assert!(result.diagnostics.is_empty());
        assert_eq!(items_of(&result.records[0])[0].unit_price, Some(1_200));
    }

    // IO-08-D5: 金額 token は i64 の両端まで読み、その外は受理しない
    #[test]
    fn parse_ej_amount_i64_bounds() {
        assert_eq!(
            parse_amount("-9223372036854775808"),
            Some((i64::MIN, false))
        );
        assert_eq!(parse_amount("9223372036854775807"), Some((i64::MAX, false)));
        assert_eq!(parse_amount("-9223372036854775809"), None);
        assert_eq!(parse_amount("9223372036854775808"), None);
    }

    // IO-08-D1: file_hash は生バイトの SHA-256
    #[test]
    fn parse_ej_file_hash_is_raw_sha256() {
        let raw = file(&ok_sale("000151"));
        let result = parse_ej(&raw).unwrap();

        assert_eq!(result.file_hash, format!("{:x}", Sha256::digest(&raw)));
        assert_eq!(result.file_hash.len(), 64);
        assert!(result
            .file_hash
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
        let decoded = SHIFT_JIS.decode_without_bom_handling(&raw).0.into_owned();
        assert_ne!(
            result.file_hash,
            format!("{:x}", Sha256::digest(decoded.as_bytes()))
        );
    }

    // IO-08-D5 / D6: 名称の中の空白を名称に含める
    #[test]
    fn parse_ej_item_name_with_inner_space() {
        let result = parse(&sale("", "000152", &[item("ﾃｽﾄ ｲﾄ A", 200)], 1, 200));

        assert!(result.diagnostics.is_empty());
        let items = items_of(&result.records[0]);
        assert_eq!(items[0].name, "ﾃｽﾄ ｲﾄ A");
        assert_eq!(items[0].amount, 200);
    }

    // IO-08-D6: 0 円の明細を受理する
    #[test]
    fn parse_ej_zero_amount_item_is_restored() {
        let result = parse(&sale(
            "",
            "000153",
            &[item(NAME_A, 0), item(NAME_B, 250)],
            2,
            250,
        ));

        assert!(result.diagnostics.is_empty());
        assert_eq!(items_of(&result.records[0])[0].amount, 0);
    }

    // -----------------------------------------------------------------------
    // 復元不能
    // -----------------------------------------------------------------------

    // IO-08-D5 / D8: 明細域の未知の行はその記録だけを復元不能にする
    #[test]
    fn parse_ej_unknown_line_in_item_region_unresolves_record_only() {
        let first = ok_sale("000160");
        let broken = sale(
            "",
            "000161",
            &[item(NAME_A, 120), lr("ﾃｲｾｲﾋｮｳｼﾞ", ""), item(NAME_B, 380)],
            2,
            500,
        );
        let unknown_line_no = first.len() + 4;
        let result = parse(&[first, broken, ok_sale("000162")].concat());

        assert!(matches!(
            result.records[1].restoration,
            EjRestoration::Unresolved { .. }
        ));
        assert_eq!(reasons_of(&result.records[1]), [UnknownLine]);
        assert_eq!(
            diags(&result),
            [(Some(unknown_line_no), UnknownLine, EjDiagnosticScope::Line)]
        );
        assert_eq!(items_of(&result.records[0]).len(), 2);
        assert_eq!(items_of(&result.records[2]).len(), 2);
    }

    // IO-08-D5: 合計域の未知のラベル
    #[test]
    fn parse_ej_unknown_line_after_separator_unresolves_record() {
        let mut lines = ok_sale("000163");
        lines.insert(6, wide("ﾃｽﾄﾊﾞﾗｲ", 500));
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [UnknownLine]);
        assert_eq!(
            diags(&result),
            [(Some(7), UnknownLine, EjDiagnosticScope::Line)]
        );
    }

    // IO-08-D3: 未知のモードは Unrecognized で復元しない
    #[test]
    fn parse_ej_unrecognized_header_mode_unresolves_record() {
        let result = parse(&sale("ZZZ", "000164", &[item(NAME_A, 100)], 1, 100));

        let record = &result.records[0];
        assert_eq!(record.mode, EjMode::Unrecognized("ZZZ".to_string()));
        assert_eq!(reasons_of(record), [UnrecognizedMode]);
        assert_eq!(
            diags(&result),
            [(Some(1), UnrecognizedMode, EjDiagnosticScope::Record)]
        );
    }

    // IO-08-D6: 点数 ≠ 数量の合計
    #[test]
    fn parse_ej_item_count_mismatch_unresolves() {
        let result = parse(&sale(
            "",
            "000165",
            &[item(NAME_A, 100), item(NAME_B, 200)],
            3,
            300,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
        assert_eq!(
            diags(&result),
            [(Some(1), InconsistentRecord, EjDiagnosticScope::Record)]
        );
    }

    // IO-08-D6: 数量 × 単価 ≠ 金額
    #[test]
    fn parse_ej_quantity_price_mismatch_unresolves() {
        let result = parse(&sale(
            "",
            "000166",
            &[qty(2, "100"), item(NAME_A, 300)],
            2,
            300,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // IO-08-D6: 合計 ≠ 明細の金額の合計
    #[test]
    fn parse_ej_total_mismatch_unresolves() {
        let result = parse(&sale(
            "",
            "000167",
            &[item(NAME_A, 100), item(NAME_B, 200)],
            2,
            400,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // IO-08-D6: 合計行が無いときは現金の金額と明細の合計を照合する
    #[test]
    fn parse_ej_cash_mismatch_without_total_line_unresolves() {
        let result = parse(
            &[
                header("", AT, "000168"),
                vec![item(NAME_A, 100), sep(), count(1), wide("現金", 200)],
            ]
            .concat(),
        );

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // IO-08-D6: 明細の金額は 0 以上
    #[test]
    fn parse_ej_negative_item_amount_unresolves() {
        let result = parse(&sale(
            "",
            "000168",
            &[item(NAME_A, 300), item(NAME_B, -100)],
            2,
            200,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // IO-08-D6: 数量行の直後が明細でない（点数・合計は明細とそろえ、孤立した数量行だけで落とす）
    #[test]
    fn parse_ej_quantity_line_not_followed_by_item_unresolves() {
        let result = parse(&sale(
            "",
            "000169",
            &[item(NAME_A, 100), qty(2, "100")],
            1,
            100,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // IO-08-D4 / D6 / D7: EOF を記録の閉じにしない
    #[test]
    fn parse_ej_record_truncated_at_eof_unresolves() {
        let sale_cut = [
            ok_sale("000170"),
            header("", AT, "000171"),
            vec![item(NAME_A, 100), item(NAME_B, 200)],
        ]
        .concat();
        let result = parse(&sale_cut);
        assert_eq!(items_of(&result.records[0]).len(), 2);
        assert_eq!(reasons_of(&result.records[1]), [IncompleteRecord]);

        let settlement_cut: Vec<String> = settlement("000172").into_iter().take(6).collect();
        let result = parse(&settlement_cut);
        assert_eq!(reasons_of(&result.records[0]), [IncompleteRecord]);
    }

    // IO-08-D2: 最終改行なしは診断を出し、最後の行も記録に残す
    #[test]
    fn parse_ej_missing_final_crlf_reports_diagnostic() {
        let lines = [ok_sale("000173"), ok_sale("000174")].concat();
        let mut raw = file(&lines);
        raw.truncate(raw.len() - 2);
        let result = parse_ej(&raw).unwrap();

        let last = lines.len();
        assert_eq!(
            diags(&result),
            [(Some(last), MissingFinalNewline, EjDiagnosticScope::Line)]
        );
        assert_eq!(items_of(&result.records[0]).len(), 2);
        assert_eq!(reasons_of(&result.records[1]), [MissingFinalNewline]);
        assert_eq!(result.records[1].body.last().unwrap().line_no, last);
    }

    // IO-08-D2: 23 / 25 バイトの行・孤立した LF / CR を正規化せず InvalidWidth にする
    #[test]
    fn parse_ej_invalid_width_line_unresolves_record() {
        let mut short = row(&item(NAME_A, 120));
        short.remove(8);
        let mut long = row(&item(NAME_A, 120));
        long.insert(8, b' ');
        let mut lone_lf = row(&item(NAME_A, 120));
        lone_lf.push(b'\n');
        lone_lf.extend(row(&item(NAME_B, 380)));
        let mut lone_cr = row(&item(NAME_A, 120));
        lone_cr.push(b'\r');
        lone_cr.extend(row(&item(NAME_B, 380)));

        let mut all = rows(&ok_sale("000175"));
        let mut expected = Vec::new();
        for (n, broken) in [short, long, lone_lf, lone_cr].into_iter().enumerate() {
            all.extend(rows(&header("", AT, &format!("00017{}", 6 + n))));
            all.push(broken);
            expected.push((Some(all.len()), InvalidWidth, EjDiagnosticScope::Line));
            all.extend(rows(&totals(2, 500)));
        }
        let result = parse_ej(&crlf(all)).unwrap();

        assert_eq!(diags(&result), expected);
        assert_eq!(items_of(&result.records[0]).len(), 2);
        for record in &result.records[1..] {
            assert_eq!(reasons_of(record), [InvalidWidth]);
            assert_eq!(record.body[0].kind, EjLineKind::Unknown);
        }
    }

    // IO-08-D4: 最初のヘッダより前の行は先頭断片
    #[test]
    fn parse_ej_leading_lines_before_first_header_are_reported() {
        let lines = [vec![wide("お  釣", 500), sep()], ok_sale("000180")].concat();
        let result = parse(&lines);

        let leading: Vec<_> = result.leading_lines.iter().map(|l| l.line_no).collect();
        assert_eq!(leading, [1, 2]);
        assert_eq!(
            diags(&result),
            [(None, LeadingFragment, EjDiagnosticScope::File)]
        );
        let record = &result.records[0];
        assert_eq!(record.header_line_no, 3);
        assert_eq!(record.body.first().unwrap().line_no, 5);
        assert_eq!(items_of(record).len(), 2);
    }

    // IO-08-D5 / D6: 小数の数量は受理しない
    #[test]
    fn parse_ej_decimal_quantity_unresolves() {
        let result = parse(&sale(
            "",
            "000181",
            &["  1.3 点   @100".to_string(), item(NAME_A, 130)],
            1,
            130,
        ));

        assert_eq!(reasons_of(&result.records[0]), [UnknownLine]);
        assert_eq!(result.records[0].body[0].kind, EjLineKind::Unknown);
    }

    // IO-08-D5 / D7: 精算の本文の未知の行
    #[test]
    fn parse_ej_unknown_line_in_settlement_unresolves() {
        let mut lines = settlement("000182");
        lines.insert(3, lr("ﾃｽﾄｺｳﾓｸ", "12"));
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [UnknownLine]);
        assert_eq!(
            diags(&result),
            [(Some(4), UnknownLine, EjDiagnosticScope::Line)]
        );
    }

    // IO-08-D7: 本文が題で始まらない精算
    #[test]
    fn parse_ej_settlement_not_starting_with_title_unresolves() {
        let mut lines = settlement("000183");
        lines.remove(2);
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [IncompleteRecord]);
    }

    // IO-08-D5d / D7a: PGM の本文の通貨記号つきの行は未知の行。区切りだけの PGM は設定の NoItems
    #[test]
    fn parse_ej_program_body_with_other_line_unresolves() {
        let mut lines = program(AT, "000184");
        lines.insert(3, item(NAME_A, 100));
        lines.extend([header("PGM", AT, "000185"), vec![sep(), sep()]].concat());
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [UnknownLine]);
        assert_eq!(result.records[1].kind, EjRecordKind::Settings);
        assert_eq!(result.records[1].restoration, EjRestoration::NoItems);
    }

    // IO-08-D6: 点数はあるが合計も現金も無い
    #[test]
    fn parse_ej_count_without_total_or_cash_unresolves() {
        let lines = [
            header("", AT, "000186"),
            vec![
                item(NAME_A, 100),
                sep(),
                count(1),
                lr("対象計", &yen(100)),
                lr("内税", &yen(9)),
                wide("お預り", 100),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [IncompleteRecord]);
        assert_eq!(
            diags(&result),
            [(Some(1), IncompleteRecord, EjDiagnosticScope::Record)]
        );
    }

    // IO-08-D6: 合計はあるが点数の行が無い（合計は明細とそろえる）
    #[test]
    fn parse_ej_total_without_item_count_unresolves() {
        let mut lines = sale("", "000189", &[item(NAME_A, 100)], 1, 100);
        lines.remove(4); // 区切りの直後の点数行
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [IncompleteRecord]);
        assert_eq!(
            diags(&result),
            [(Some(1), IncompleteRecord, EjDiagnosticScope::Record)]
        );
    }

    // IO-08-D6: 本文 0 行の通常の記録
    #[test]
    fn parse_ej_header_only_record_unresolves() {
        let result = parse(&[header("", AT, "000187"), ok_sale("000188")].concat());

        assert!(result.records[0].body.is_empty());
        assert_eq!(reasons_of(&result.records[0]), [IncompleteRecord]);
        assert_eq!(items_of(&result.records[1]).len(), 2);
    }

    // IO-08-D6: 数量行の連続は後の行で上書きせず、前の行も使わず不一致にする
    // （同じ数量行を 2 つ並べ、どちらを採っても照合がそろう形にする）
    #[test]
    fn parse_ej_consecutive_quantity_lines_unresolve() {
        let result = parse(&sale(
            "",
            "000210",
            &[qty(2, "100"), qty(2, "100"), item(NAME_A, 200)],
            2,
            200,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
        assert_eq!(
            diags(&result),
            [(Some(1), InconsistentRecord, EjDiagnosticScope::Record)]
        );
    }

    // IO-08-D6: 点数・合計・（合計が無いときの）現金の行が 2 行あれば、値がそろっていても不一致
    #[test]
    fn parse_ej_duplicate_count_or_total_lines_unresolve() {
        let mut twice_count = ok_sale("000211");
        twice_count.insert(6, count(2));
        let mut twice_total = ok_sale("000212");
        twice_total.insert(9, wide("合  計", 500));
        let twice_cash = [
            header("", AT, "000213"),
            vec![
                item(NAME_A, 100),
                sep(),
                count(1),
                wide("現金", 100),
                wide("現金", 100),
            ],
        ]
        .concat();
        let result = parse(&[twice_count, twice_total, twice_cash].concat());

        let restorations: Vec<_> = result.records.iter().map(|r| &r.restoration).collect();
        let inconsistent = EjRestoration::Unresolved {
            reasons: vec![InconsistentRecord],
        };
        assert_eq!(restorations, [&inconsistent, &inconsistent, &inconsistent]);
    }

    // IO-08-D5 / D6: 数量 0 と `-` つきの単価の数量行は受理しない（数量 0 は照合を素通りするため）
    #[test]
    fn parse_ej_zero_quantity_or_negative_unit_price_unresolves() {
        let lines = [
            sale(
                "",
                "000214",
                &[qty(0, "100"), item("ﾃｲｾｲ", 0), item(NAME_A, 100)],
                1,
                100,
            ),
            sale(
                "",
                "000215",
                &[qty(1, "-100"), item("ﾃｲｾｲ", 0), item(NAME_A, 100)],
                1,
                100,
            ),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [UnknownLine]);
            assert_eq!(record.body[0].kind, EjLineKind::Unknown);
        }
        assert_eq!(parse_quantity("  2 点   @-100"), None);
        assert_eq!(parse_quantity("  1 点   @-0"), None);
        assert_eq!(parse_quantity("  1 点   @0"), Some((1, 0)));
    }

    // IO-08-D6: 区切りの前に明細が 1 件も無い
    #[test]
    fn parse_ej_no_item_before_separator_unresolves() {
        let result = parse(&sale("", "000216", &[], 0, 0));

        assert_eq!(reasons_of(&result.records[0]), [IncompleteRecord]);
    }

    // IO-08-D6: 照合に使うラベルの金額 token が読めない（現金行があっても合計を読み替えない）
    #[test]
    fn parse_ej_unreadable_total_amount_unresolves() {
        let lines = [
            header("", AT, "000217"),
            vec![
                item(NAME_A, 100),
                sep(),
                count(1),
                lr("合  計", "￥1,00"),
                wide("現金", 100),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // G-O3 / IO-08-D3a / D7: 戻のモードの入金も入金 / 出金 / 替の記録（明細なし）
    #[test]
    fn parse_ej_paid_in_line_in_return_mode_is_cash_movement() {
        let result = parse(&[header("戻", AT, "000218"), vec![wide("入金", 500)]].concat());

        assert_eq!(result.records[0].kind, EjRecordKind::CashMovement);
        assert_eq!(result.records[0].restoration, EjRestoration::NoItems);
    }

    // IO-08-D2 / D4: 先頭断片の行にも幅の検査をする
    #[test]
    fn parse_ej_invalid_width_leading_line_is_reported() {
        let mut short = row(&sep());
        short.pop();
        let mut all = vec![short];
        all.extend(rows(&ok_sale("000219")));
        let result = parse_ej(&crlf(all)).unwrap();

        assert_eq!(
            diags(&result),
            [
                (None, LeadingFragment, EjDiagnosticScope::File),
                (Some(1), InvalidWidth, EjDiagnosticScope::Line),
            ]
        );
        assert_eq!(items_of(&result.records[0]).len(), 2);
    }

    // IO-08-D2: 24 バイトの中に孤立した CR / LF がある行も幅違反とし、trim で隠さない
    #[test]
    fn parse_ej_lone_cr_or_lf_within_24_bytes_unresolves_record() {
        for control in *b"\r\n" {
            let mut all = rows(&ok_sale("000222"));
            all[5][1] = control; // 点数行の先頭の空白。幅は 24 のまま
            let result = parse_ej(&crlf(all)).unwrap();

            let record = &result.records[0];
            assert_eq!(reasons_of(record), [InvalidWidth]);
            assert_eq!(
                diags(&result),
                [(Some(6), InvalidWidth, EjDiagnosticScope::Line)]
            );
            assert_eq!(record.body[3].kind, EjLineKind::Unknown);
            assert!(record.body[3].text.contains(control as char), "生行を保つ");
        }
    }

    // IO-08-D2 / D3 / D4: CR / LF を含む行はヘッダにならず、先頭断片でも幅違反とする
    #[test]
    fn parse_ej_lone_cr_or_lf_in_header_or_leading_line_is_invalid_width() {
        for control in *b"\r\n" {
            // 2 件目のヘッダのモード欄（空白 5 バイト）の先頭
            let mut all = rows(&[ok_sale("000223"), ok_sale("000224")].concat());
            all[11][0] = control;
            let result = parse_ej(&crlf(all)).unwrap();

            assert_eq!(result.records.len(), 1);
            let record = &result.records[0];
            assert_eq!(reasons_of(record)[0], InvalidWidth);
            assert_eq!(
                result.diagnostics[0].code, InvalidWidth,
                "最初の診断はヘッダだった行"
            );
            assert_eq!(result.diagnostics[0].line_no, Some(12));

            // 先頭断片の行
            let mut leading = row(&sep());
            leading[0] = control;
            let mut all = vec![leading];
            all.extend(rows(&ok_sale("000225")));
            let result = parse_ej(&crlf(all)).unwrap();

            assert_eq!(
                diags(&result),
                [
                    (None, LeadingFragment, EjDiagnosticScope::File),
                    (Some(1), InvalidWidth, EjDiagnosticScope::Line),
                ]
            );
            assert_eq!(items_of(&result.records[0]).len(), 2);
        }
    }

    // IO-08-D5: 明細の金額は通貨記号つき、数量行の単価は通貨記号なしだけを受理する
    #[test]
    fn parse_ej_currency_symbol_required_on_item_and_rejected_on_unit_price() {
        let lines = [
            sale("", "000220", &[lr(NAME_A, "120")], 1, 120),
            sale("", "000221", &[qty(2, "\\100"), item(NAME_A, 200)], 2, 200),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [UnknownLine]);
            assert_eq!(record.body[0].kind, EjLineKind::Unknown);
        }
    }

    // -----------------------------------------------------------------------
    // 記録の種類と明細を持たない記録（IO-08-D3a / D5d / D7a）
    // -----------------------------------------------------------------------

    const DAILY_DONE: EjRecordKind = EjRecordKind::Settlement {
        report: EjSettlementReport::Daily,
        completed: true,
    };
    const DAILY_CUT: EjRecordKind = EjRecordKind::Settlement {
        report: EjSettlementReport::Daily,
        completed: false,
    };

    fn kinds(result: &EjParseResult) -> Vec<EjRecordKind> {
        result.records.iter().map(|r| r.kind.clone()).collect()
    }

    fn body_kinds(record: &EjRecord) -> Vec<EjLineKind> {
        record.body.iter().map(|l| l.kind.clone()).collect()
    }

    fn all_no_items(result: &EjParseResult) -> bool {
        result
            .records
            .iter()
            .all(|r| r.restoration == EjRestoration::NoItems)
    }

    fn training_mark() -> String {
        "･･････トレーニング･･････".to_string()
    }

    fn number_print(n: &str) -> String {
        lr(&format!("＃ {n}"), "")
    }

    /// 終わりの印字の無い日計明細 Z（題・値・送信の異常終了）
    fn cut_settlement_body() -> Vec<String> {
        vec![
            "0007 日計明細  Z 0003".to_string(),
            lr("総売", "12"),
            lr("", &yen(4_560)),
            lr("ｽﾏ-ﾄﾌｫﾝ送信", "異常終了"),
        ]
    }

    // G-K1 / IO-08-D3 / D3a: 精算票は題で決め、モード欄（空・点検・PGM）によらない
    #[test]
    fn parse_ej_settlement_kind_is_decided_by_title_not_mode() {
        let lines = [
            header("", AT, "000300"),
            settlement_body(),
            header("点検", AT, "000301"),
            settlement_body(),
            header("PGM", AT, "000302"),
            settlement_body(),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(kinds(&result), [DAILY_DONE, DAILY_DONE, DAILY_DONE]);
        assert!(all_no_items(&result));
        let modes: Vec<_> = result.records.iter().map(|r| r.mode.clone()).collect();
        assert_eq!(modes, [EjMode::Normal, EjMode::Inspection, EjMode::Program]);
    }

    // G-K2 / IO-08-D3a: 点検票の題はモード欄（空・精算）によらず Inspection
    #[test]
    fn parse_ej_inspection_title_in_blank_or_settlement_mode() {
        let body = vec![
            "0007 日計明細 X".to_string(),
            lr("総売", "12"),
            lr("", &yen(4_560)),
        ];
        let lines = [
            header("", AT, "000303"),
            body.clone(),
            header("精算", AT, "000304"),
            body,
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            kinds(&result),
            [EjRecordKind::Inspection, EjRecordKind::Inspection]
        );
        assert!(all_no_items(&result));
    }

    // G-K3 / IO-08-D2 / D3a / D8: 練習の記録は明細を返さず、本文は番号印字の行も Text。幅の検査はする
    #[test]
    fn parse_ej_training_records_return_no_items() {
        let training_sale = [
            header("練習", AT, "000305"),
            vec![item(NAME_A, 120), item(NAME_B, 380)],
            totals(2, 500),
            vec![training_mark(), number_print("0012")],
        ]
        .concat();
        let training_paid_in = [header("練習", AT, "000306"), vec![wide("入金", 1_000)]].concat();
        let result = parse(&[training_sale.clone(), training_paid_in].concat());

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            kinds(&result),
            [EjRecordKind::Training, EjRecordKind::Training]
        );
        assert!(all_no_items(&result));
        for record in &result.records {
            assert!(body_kinds(record).iter().all(|k| *k == EjLineKind::Text));
        }

        // 25 バイトの行を 1 行だけ持つ練習の取引
        let mut all = rows(&training_sale);
        all[2].insert(8, b' ');
        let result = parse_ej(&crlf(all)).unwrap();
        let record = &result.records[0];
        assert_eq!(record.kind, EjRecordKind::Training);
        assert_eq!(reasons_of(record), [InvalidWidth]);
        assert_eq!(
            diags(&result),
            [(Some(3), InvalidWidth, EjDiagnosticScope::Line)]
        );
        assert_eq!(record.body[0].kind, EjLineKind::Unknown);
    }

    // G-K4 / IO-08-D3a: 点検のモードのトレーニングの表示は Training
    #[test]
    fn parse_ej_training_display_in_inspection_mode() {
        let lines = [
            header("点検", AT, "000307"),
            vec!["ﾄﾚｰﾆﾝｸﾞﾓｰﾄﾞを開始します".to_string(), training_mark()],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(kinds(&result), [EjRecordKind::Training]);
        assert!(all_no_items(&result));
    }

    // G-K5 / IO-08-D3a / D8: 未知のモードは Unclassified で UnrecognizedMode 1 件だけ
    #[test]
    fn parse_ej_unknown_mode_is_unclassified() {
        let result = parse(&sale("ZZZ", "000308", &[item(NAME_A, 100)], 1, 100));

        let record = &result.records[0];
        assert_eq!(record.kind, EjRecordKind::Unclassified);
        assert_eq!(record.mode, EjMode::Unrecognized("ZZZ".to_string()));
        assert_eq!(reasons_of(record), [UnrecognizedMode]);
        assert_eq!(
            diags(&result),
            [(Some(1), UnrecognizedMode, EjDiagnosticScope::Record)]
        );
    }

    // G-K6 / IO-08-D3a: 精算のモードで題の無い記録は Unclassified + IncompleteRecord
    #[test]
    fn parse_ej_settlement_mode_without_title_is_unclassified() {
        let lines = [
            header("精算", AT, "000309"),
            settlement_body()[1..].to_vec(),
        ]
        .concat();
        let result = parse(&lines);

        let record = &result.records[0];
        assert_eq!(record.kind, EjRecordKind::Unclassified);
        assert_eq!(reasons_of(record), [IncompleteRecord]);
    }

    // G-K7 / IO-08-D3: モード欄の既知の値
    #[test]
    fn parse_ej_known_mode_values() {
        let lines = [
            header("練習", AT, "000310"),
            header("点検", AT, "000311"),
            header("PGM1", AT, "000312"),
            header("PGM3", AT, "000313"),
            header("OFF", AT, "000314"),
        ]
        .concat();
        let result = parse(&lines);

        let modes: Vec<_> = result.records.iter().map(|r| r.mode.clone()).collect();
        assert_eq!(
            modes,
            [
                EjMode::Training,
                EjMode::Inspection,
                EjMode::Program1,
                EjMode::Program3,
                EjMode::Off,
            ]
        );
    }

    // G-K8 / IO-08-D3a: 点検のモードの取引の本文は取引の文法で復元しない
    #[test]
    fn parse_ej_sale_body_in_inspection_mode_unresolves() {
        let result = parse(&sale("点検", "000315", &[item(NAME_A, 100)], 1, 100));

        let record = &result.records[0];
        assert_eq!(record.kind, EjRecordKind::Unclassified);
        assert_eq!(reasons_of(record), [UnknownLine]);
    }

    // G-S1 / IO-08-D5d / D7a: ＰＬＵ Z と勤怠 Z は完了した精算票
    #[test]
    fn parse_ej_plu_and_attendance_settlements() {
        let lines = [
            header("精算", AT, "000316"),
            vec![
                "0014 ＰＬＵ Z 0003".to_string(),
                "ﾃﾞｰﾀｦﾎｿﾞﾝｼﾏｼﾀ".to_string(),
                sep(),
                "ＰＬＵ".to_string(),
                lr("SDｶｰﾄﾞ保存", "正常終了"),
            ],
            header("精算", AT, "000317"),
            vec![
                "0021 勤怠 Z 0003".to_string(),
                sep(),
                "勤怠".to_string(),
                lr("SDｶｰﾄﾞ保存", "正常終了"),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            kinds(&result),
            [
                EjRecordKind::Settlement {
                    report: EjSettlementReport::Plu,
                    completed: true,
                },
                EjRecordKind::Settlement {
                    report: EjSettlementReport::Attendance,
                    completed: true,
                },
            ]
        );
        assert!(all_no_items(&result));
        assert_eq!(result.records[0].body[3].kind, EjLineKind::ReportEnd);
    }

    // G-S2 / IO-08-D5d: 精算票の追加のラベル行
    #[test]
    fn parse_ej_settlement_additional_labels() {
        let mut body = settlement_body();
        let extra = [
            lr("信在高", "1,000"),
            lr("貸在高", "-500"),
            lr("券在高", "300"),
            lr("領収書", "3 件"),
            lr("領収書 印紙", "1 件"),
            lr("取引中止", "2 件"),
        ];
        body.splice(6..6, extra.iter().cloned());
        let result = parse(&[header("精算", AT, "000318"), body].concat());

        assert!(result.diagnostics.is_empty());
        assert!(all_no_items(&result));
        let labels: Vec<_> = result.records[0].body[6..12]
            .iter()
            .map(|l| l.kind.clone())
            .collect();
        let expected: Vec<_> = ["信在高", "貸在高", "券在高", "領収書", "領収書", "取引中止"]
            .into_iter()
            .map(|label| EjLineKind::Labeled { label })
            .collect();
        assert_eq!(labels, expected);
    }

    // G-S3 / IO-08-D5d: 送信の異常終了の後の 4 桁の行と案内文の行
    #[test]
    fn parse_ej_settlement_transmission_failure_lines() {
        let mut body = settlement_body();
        body.pop();
        body.extend([
            lr("ｽﾏ-ﾄﾌｫﾝ送信", "異常終了"),
            lr("", "4012"),
            "ｿｳｼﾝﾆｼｯﾊﾟｲｼﾏｼﾀ".to_string(),
            "ｻｲｿｳｼﾝｼﾃｸﾀﾞｻｲ".to_string(),
        ]);
        let result = parse(&[header("精算", AT, "000319"), body].concat());

        assert!(result.diagnostics.is_empty());
        assert!(all_no_items(&result));
        let tail = &body_kinds(&result.records[0])[10..];
        assert_eq!(
            tail,
            [
                EjLineKind::Status {
                    label: "ｽﾏ-ﾄﾌｫﾝ送信",
                    result: "異常終了".to_string(),
                },
                EjLineKind::AmountOnly,
                EjLineKind::Text,
                EjLineKind::Text,
            ]
        );
    }

    // G-S4 / IO-08-D7a / D4: 次のヘッダで閉じた中断は NoItems、file の最後の記録なら IncompleteRecord
    #[test]
    fn parse_ej_interrupted_settlement() {
        let lines = [
            header("精算", AT, "000320"),
            cut_settlement_body(),
            settlement("000321"),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(kinds(&result), [DAILY_CUT, DAILY_DONE]);
        assert!(all_no_items(&result));

        let result = parse(&[header("精算", AT, "000322"), cut_settlement_body()].concat());
        let record = &result.records[0];
        assert_eq!(record.kind, DAILY_CUT);
        assert_eq!(reasons_of(record), [IncompleteRecord]);
    }

    // G-S5 / IO-08-D5d: 点検票の題の 4 形
    #[test]
    fn parse_ej_inspection_title_forms() {
        let titles = ["0000 日計明細 X", "0014 ＰＬＵ X", "在売点検 X", "X"];
        let lines: Vec<String> = titles
            .iter()
            .enumerate()
            .flat_map(|(n, title)| {
                [
                    header("点検", AT, &format!("00033{n}")),
                    vec![title.to_string(), lr("総売", "12")],
                ]
                .concat()
            })
            .collect();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert!(all_no_items(&result));
        assert!(result
            .records
            .iter()
            .all(|r| r.kind == EjRecordKind::Inspection));
        let titles: Vec<_> = result
            .records
            .iter()
            .map(|r| r.body[0].kind.clone())
            .collect();
        let title = |leading: Option<&str>| EjLineKind::ReportTitle {
            leading_no: leading.map(str::to_string),
            trailing_no: None,
        };
        assert_eq!(
            titles,
            [
                title(Some("0000")),
                title(Some("0014")),
                title(None),
                title(None)
            ]
        );
    }

    // G-S6 / IO-08-D5d: 精算票の Text は数字を含む行を通さない
    #[test]
    fn parse_ej_settlement_name_line_with_digit_unresolves() {
        let mut lines = settlement("000323");
        lines.insert(4, "ﾃｽﾄ2ｺｳﾓｸ".to_string());
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [UnknownLine]);
        assert_eq!(result.records[0].body[2].kind, EjLineKind::Unknown);
    }

    // G-O1 / IO-08-D5d / D7a: 設定の記録（状態・本文 0 行・設定の印字）は NoItems
    #[test]
    fn parse_ej_settings_records() {
        assert!(sjis("表").contains(&0x5C), "2 byte 目が 0x5C の漢字");
        let lines = [
            header("PGM", AT, "000324"),
            vec![lr("SD設定読込み", "正常終了")],
            header("OFF", AT, "000325"),
            vec![sep(), lr("自動設定保存", "正常終了"), sep()],
            header("PGM1", AT, "000326"),
            header("PGM3", AT, "000327"),
            vec![
                "ﾋｮｳｼﾞ 表 ｾｯﾃｲ".to_string(),
                String::new(),
                lr("ﾄｹｲ", "12:30"),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert!(result
            .records
            .iter()
            .all(|r| r.kind == EjRecordKind::Settings));
        assert!(all_no_items(&result));
        assert!(body_kinds(&result.records[3])
            .iter()
            .all(|k| *k == EjLineKind::Text));
    }

    // G-O2 / IO-08-D5d: 設定の記録に紛れた取引の形（通貨記号・状態ラベル + 通貨記号・` 点`）
    #[test]
    fn parse_ej_settings_reject_sale_lines() {
        let cases = [
            lr("ﾃｽﾄ", "\\100"),
            lr("自動設定保存", "\\100"),
            lr("SD設定読込み", "￥１００"),
            lr("", "1 点"),
        ];
        let lines: Vec<String> = cases
            .iter()
            .enumerate()
            .flat_map(|(n, line)| {
                [header("PGM", AT, &format!("00034{n}")), vec![line.clone()]].concat()
            })
            .collect();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(record.kind, EjRecordKind::Settings);
            assert_eq!(reasons_of(record), [UnknownLine]);
            assert_eq!(record.body[0].kind, EjLineKind::Unknown);
        }
    }

    // G-O4 / IO-08-D3a / D5d: 領収書の 3 行
    #[test]
    fn parse_ej_receipt_record() {
        let lines = [
            header("", AT, "000328"),
            vec![
                " 一連No.000327".to_string(),
                " 領収No.1".to_string(),
                wide("領収書", 1_500),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(kinds(&result), [EjRecordKind::Receipt]);
        assert!(all_no_items(&result));
        let labels: Vec<_> = ["一連No.", "領収No.", "領収書"]
            .into_iter()
            .map(|label| EjLineKind::Labeled { label })
            .collect();
        assert_eq!(body_kinds(&result.records[0]), labels);
    }

    // G-O5 / IO-08-D3a: 番号印字の行は入金・設定の記録の種類を変えない
    #[test]
    fn parse_ej_number_print_in_paid_in_and_settings() {
        let lines = [
            header("", AT, "000329"),
            vec![number_print("0012"), wide("入金", 1_000)],
            header("OFF", AT, "000330"),
            vec![
                sep(),
                lr("自動設定保存", "正常終了"),
                sep(),
                number_print("0013"),
                number_print("0014"),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            kinds(&result),
            [EjRecordKind::CashMovement, EjRecordKind::Settings]
        );
        assert!(all_no_items(&result));
        assert_eq!(result.records[0].body[0].kind, EjLineKind::NumberPrint);
        assert_eq!(result.records[1].body[3].kind, EjLineKind::NumberPrint);
        assert_eq!(result.records[1].body[4].kind, EjLineKind::NumberPrint);
    }

    // G-O6 / IO-08-D3a / D5 / D6c: 合計域の番号印字は照合を変えず、明細域の番号印字は未知の行
    #[test]
    fn parse_ej_number_print_in_sale() {
        let plain = [
            header("", AT, "000331"),
            vec![
                item(NAME_A, 120),
                item(NAME_B, 380),
                sep(),
                count(2),
                lr("対象計", &yen(500)),
                lr("内税", &yen(45)),
                wide("現金", 500),
            ],
        ]
        .concat();
        let printed = [
            plain.clone(),
            vec![number_print("0012"), number_print("0013")],
        ]
        .concat();
        let result = parse(&[plain, printed].concat());

        assert!(result.diagnostics.is_empty());
        assert_eq!(items_of(&result.records[0]).len(), 2);
        let shape = |record: &EjRecord| {
            items_of(record)
                .iter()
                .map(|i| (i.name.clone(), i.quantity, i.unit_price, i.amount))
                .collect::<Vec<_>>()
        };
        assert_eq!(shape(&result.records[0]), shape(&result.records[1]));
        assert!(matches!(
            result.records[1].restoration,
            EjRestoration::Restored { item_count: 2, ref adjustments, .. } if adjustments.is_empty()
        ));
        assert_eq!(result.records[1].body[7].kind, EjLineKind::NumberPrint);

        let in_items = [
            header("", AT, "000332"),
            vec![
                item(NAME_A, 120),
                number_print("0012"),
                sep(),
                count(1),
                wide("合  計", 120),
            ],
        ]
        .concat();
        let result = parse(&in_items);
        let record = &result.records[0];
        assert_eq!(reasons_of(record), [UnknownLine]);
        assert_eq!(record.body[1].kind, EjLineKind::Unknown);
    }

    // -----------------------------------------------------------------------
    // 取引の合計域（IO-08-D5a / D5b / D6c）
    // -----------------------------------------------------------------------

    /// 明細 A（金額 total）→ 区切り → 点数 1 → 合計域の行
    fn one_item_sale(number: &str, total: i64, tail: &[String]) -> Vec<String> {
        [
            header("", AT, number),
            vec![item(NAME_A, total), sep(), count(1)],
            tail.to_vec(),
        ]
        .concat()
    }

    fn restored_one(record: &EjRecord, amount: i64) {
        let items = items_of(record);
        assert_eq!(items.len(), 1);
        assert_eq!((items[0].name.as_str(), items[0].amount), (NAME_A, amount));
        assert!(matches!(
            record.restoration,
            EjRestoration::Restored { item_count: 1, ref adjustments, .. } if adjustments.is_empty()
        ));
    }

    // 合計域の全角の金額（`￥１，０００` / `－１，０００`）
    fn zen_amount(amount: i64) -> String {
        let text = zen(&yen(amount.abs()));
        if amount < 0 {
            format!("－{}", text.trim_start_matches('￥'))
        } else {
            text
        }
    }

    // IO-08-D5: 全角の `－` は全角の数字とだけ組む
    #[test]
    fn parse_ej_full_width_minus_amount_token() {
        assert_eq!(parse_amount("－１，０００"), Some((-1_000, false)));
        assert_eq!(parse_amount("－￥１００"), Some((-100, true)));
        for mixed in ["－1,000", "-１，０００", "－\\100", "-￥１００"] {
            assert_eq!(parse_amount(mixed), None, "受理してはいけない token");
        }
    }

    // G-T1 / IO-08-D5a: `対象計` の率の後の金額の折返し
    #[test]
    fn parse_ej_taxable_total_wraps_amount() {
        let result = parse(&one_item_sale(
            "000400",
            12_345,
            &[
                lr("対象計", "8.0%"),
                lr("", "\\12,345"),
                lr("内税", "\\914"),
                wide("合  計", 12_345),
            ],
        ));

        assert!(result.diagnostics.is_empty());
        restored_one(&result.records[0], 12_345);
        assert_eq!(
            result.records[0].body[4].kind,
            EjLineKind::Continued { amount: 12_345 }
        );
    }

    // G-T2 / IO-08-D5a / D6c: `合  計` の無い取引で折り返した支払行の金額を照合に使う
    #[test]
    fn parse_ej_wrapped_payment_line_without_total() {
        let result = parse(&one_item_sale(
            "000401",
            1_000,
            &["ｸﾚｼﾞｯﾄ電子M".to_string(), lr("", &zen_amount(1_000))],
        ));

        assert!(result.diagnostics.is_empty());
        restored_one(&result.records[0], 1_000);
    }

    // G-T3 / IO-08-D5a: `お預り` / `お  釣` の折返し
    #[test]
    fn parse_ej_tendered_and_change_wrap() {
        let result = parse(&one_item_sale(
            "000402",
            1_000,
            &[
                wide("合  計", 1_000),
                "お預り".to_string(),
                lr("", &zen_amount(10_000)),
                "お  釣".to_string(),
                lr("", &zen_amount(9_000)),
            ],
        ));

        assert!(result.diagnostics.is_empty());
        restored_one(&result.records[0], 1_000);
    }

    // G-T4 / IO-08-D5b / D6c: `合  計` も `現金` も無く、支払行 1 行の金額で照合する
    #[test]
    fn parse_ej_single_payment_line_without_total() {
        for (n, label) in ["ｸﾚｼﾞｯﾄ電子M", "売掛", "商品券"].into_iter().enumerate()
        {
            let result = parse(&one_item_sale(
                &format!("00041{n}"),
                1_000,
                &[wide(label, 1_000)],
            ));
            assert!(result.diagnostics.is_empty(), "{label}");
            restored_one(&result.records[0], 1_000);

            let result = parse(&one_item_sale(
                &format!("00042{n}"),
                1_000,
                &[wide(label, 999)],
            ));
            assert_eq!(
                reasons_of(&result.records[0]),
                [InconsistentRecord],
                "{label}"
            );
        }
    }

    // G-T5 / IO-08-D5b / D6c: `合  計` があれば支払行の併用でも照合は `合  計`
    #[test]
    fn parse_ej_total_with_mixed_payments() {
        let lines = [
            one_item_sale(
                "000430",
                1_000,
                &[
                    wide("合  計", 1_000),
                    wide("ｸｰﾎﾟﾝﾎﾟｲﾝﾄ払", 100),
                    wide("お預り", 1_000),
                    wide("お  釣", 100),
                ],
            ),
            one_item_sale(
                "000431",
                1_000,
                &[
                    wide("合  計", 1_000),
                    wide("商品券", 500),
                    wide("ｸﾚｼﾞｯﾄ電子M", 500),
                ],
            ),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        restored_one(&result.records[0], 1_000);
        restored_one(&result.records[1], 1_000);
    }

    // G-T6 / IO-08-D6a / D6c: 合計域の訂正は直前の支払行を取り消し、明細に効かない
    #[test]
    fn parse_ej_total_region_correction_cancels_payment() {
        let lines = [
            one_item_sale(
                "000440",
                1_000,
                &[
                    wide("合  計", 1_000),
                    wide("売掛", 1_000),
                    lr("訂正", "-1,000"),
                    wide("現金", 1_000),
                ],
            ),
            one_item_sale(
                "000441",
                1_000,
                &[
                    wide("合  計", 1_000),
                    wide("現金", 1_000),
                    lr("訂正", "-1,000"),
                    wide("現金", 1_000),
                ],
            ),
            // `合  計` なし: 取り消されていない支払行は `現金` の 1 行
            one_item_sale(
                "000442",
                100,
                &[wide("売掛", 100), lr("訂正", "-100"), wide("現金", 100)],
            ),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        restored_one(&result.records[0], 1_000);
        restored_one(&result.records[1], 1_000);
        restored_one(&result.records[2], 100);
        assert_eq!(
            result.records[2].body[4].kind,
            EjLineKind::Correction { amount: -100 }
        );
    }

    // G-T7 / IO-08-D5b: 軽減税率の行と `※` つきの明細
    #[test]
    fn parse_ej_reduced_tax_rate_lines() {
        let lines = [
            header("", AT, "000450"),
            vec![
                lr(NAME_A, "\\300※"),
                sep(),
                count(1),
                lr("対象計※ 8.0%", "\\300"),
                lr("内税※", "\\22"),
                wide("現金", 300),
                "注）※は軽減税率適用".to_string(),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        restored_one(&result.records[0], 300);
        assert_eq!(
            result.records[0].body[6].kind,
            EjLineKind::Labeled {
                label: "注）※は軽減税率適用"
            }
        );
    }

    // G-F9 / IO-08-D6c: `合  計` が無く支払行 2 行（和は明細の合計）は不一致、0 行は欠け
    #[test]
    fn parse_ej_payment_lines_without_total_are_not_summed() {
        let lines = [
            one_item_sale("000460", 1_000, &[wide("現金", 600), wide("売掛", 400)]),
            one_item_sale(
                "000461",
                1_000,
                &[lr("対象計", &yen(1_000)), lr("内税", &yen(90))],
            ),
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
        assert_eq!(reasons_of(&result.records[1]), [IncompleteRecord]);
    }

    // G-F10 / IO-08-D6a: 合計域の訂正の直前が支払行でない、金額が直前の支払行と合わない
    #[test]
    fn parse_ej_total_region_correction_mismatch_unresolves() {
        let lines = [
            one_item_sale(
                "000470",
                1_000,
                &[
                    lr("対象計", &yen(1_000)),
                    lr("訂正", "-1,000"),
                    wide("合  計", 1_000),
                ],
            ),
            one_item_sale(
                "000471",
                1_000,
                &[
                    wide("合  計", 1_000),
                    wide("売掛", 1_000),
                    lr("訂正", "-900"),
                    wide("現金", 1_000),
                ],
            ),
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
        assert_eq!(reasons_of(&result.records[1]), [InconsistentRecord]);
    }

    // G-F15 / IO-08-D6a / D6c: `合  計` の無い取引の照合の金額の候補
    #[test]
    fn parse_ej_payment_candidate_selection_without_total() {
        let lines = [
            // (a) 取り消した支払行だけ: 候補 0 行
            one_item_sale("000480", 100, &[wide("売掛", 100), lr("訂正", "-100")]),
            // (b) 候補 2 行で、先頭だけが明細の和と一致
            one_item_sale("000481", 100, &[wide("現金", 100), wide("売掛", 40)]),
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [IncompleteRecord]);
        assert_eq!(reasons_of(&result.records[1]), [InconsistentRecord]);
    }

    // -----------------------------------------------------------------------
    // 取引の明細域と取引中止（IO-08-D5a / D5c / D6a / D6b / D6d / D7a）
    // -----------------------------------------------------------------------

    const NAME_C: &str = "ﾃｽﾄｸﾞﾐ";
    const LONG_NAME: &str = "ﾅｶﾞｲﾒｲｼｮｳﾉ ﾃｽﾄｼｮｳﾋﾝ";

    fn correction(amount: &str) -> String {
        lr("訂正", amount)
    }

    fn rate(percent: u32) -> String {
        lr(&format!("-{percent}%"), "")
    }

    fn percent_off(amount: &str) -> String {
        lr("％－", amount)
    }

    fn minus_key(amount: &str) -> String {
        lr("－", amount)
    }

    fn return_mark() -> String {
        lr("戻 ････", "")
    }

    fn cancel_mark() -> String {
        lr("取引中止 ････", "")
    }

    /// 明細域の行 → 区切り → 点数 → 合計
    fn sale_with(number: &str, items: &[String], n: i64, total: i64) -> Vec<String> {
        [
            header("", AT, number),
            items.to_vec(),
            vec![sep(), count(n), wide("合  計", total)],
        ]
        .concat()
    }

    fn adjustments_of(record: &EjRecord) -> &[EjAdjustment] {
        match &record.restoration {
            EjRestoration::Restored { adjustments, .. } => adjustments,
            other => panic!("Restored を期待しました: {other:?}"),
        }
    }

    fn item_count_of(record: &EjRecord) -> i64 {
        match &record.restoration {
            EjRestoration::Restored { item_count, .. } => *item_count,
            other => panic!("Restored を期待しました: {other:?}"),
        }
    }

    fn names(record: &EjRecord) -> Vec<&str> {
        items_of(record).iter().map(|i| i.name.as_str()).collect()
    }

    fn adjustment_kinds(record: &EjRecord) -> Vec<(EjAdjustmentKind, i64)> {
        adjustments_of(record)
            .iter()
            .map(|a| (a.kind.clone(), a.amount))
            .collect()
    }

    // G-T8 / IO-08-D5 / D6d: 取引全体が負（戻の印の明細 × 2、全角 `－` の合計と支払）
    #[test]
    fn parse_ej_whole_negative_sale() {
        let lines = [
            header("", AT, "000500"),
            vec![
                return_mark(),
                lr(NAME_A, "-600"),
                return_mark(),
                lr(NAME_B, "-400"),
                sep(),
                count(-2),
                lr("対象計 8.0%", "-1,000"),
                lr("合  計", "－１，０００"),
                lr("売掛", "－１，０００"),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        let record = &result.records[0];
        let items: Vec<_> = items_of(record)
            .iter()
            .map(|i| (i.quantity, i.amount))
            .collect();
        assert_eq!(items, [(-1, -600), (-1, -400)]);
        assert_eq!(item_count_of(record), -2);
    }

    // G-F12 / IO-08-D5: 幅の混ざった全角 `－` / 半角 `-` の照合の金額は読めない
    #[test]
    fn parse_ej_mixed_width_negative_total_unresolves() {
        let negative = |number: &str, last: String| {
            [
                header("", AT, number),
                vec![return_mark(), lr(NAME_A, "-1,000"), sep(), count(-1), last],
            ]
            .concat()
        };
        let result = parse(
            &[
                negative("000501", lr("合  計", "－1,000")),
                negative("000502", lr("現金", "-１，０００")),
            ]
            .concat(),
        );

        for record in &result.records {
            assert_eq!(reasons_of(record), [InconsistentRecord]);
        }
    }

    // G-M1 / IO-08-D6a: 負の訂正は直前の明細を取り消す
    #[test]
    fn parse_ej_negative_correction_cancels_previous_item() {
        let result = parse(&sale_with(
            "000510",
            &[item(NAME_A, 120), item(NAME_B, 380), correction("-380")],
            1,
            120,
        ));

        assert!(result.diagnostics.is_empty());
        let record = &result.records[0];
        assert_eq!(names(record), [NAME_A]);
        assert_eq!(item_count_of(record), 1);
    }

    // G-M2 / IO-08-D6a: 数量行つきの明細・折り返した明細も取り消せる
    #[test]
    fn parse_ej_correction_cancels_quantity_and_wrapped_items() {
        let lines = [
            sale_with(
                "000511",
                &[
                    qty(3, "100"),
                    item(NAME_A, 300),
                    correction("-300"),
                    item(NAME_C, 200),
                ],
                1,
                200,
            ),
            sale_with(
                "000512",
                &[
                    LONG_NAME.to_string(),
                    lr("", "\\1,200"),
                    correction("-1,200"),
                    item(NAME_B, 200),
                ],
                1,
                200,
            ),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(names(&result.records[0]), [NAME_C]);
        assert_eq!(item_count_of(&result.records[0]), 1);
        assert_eq!(names(&result.records[1]), [NAME_B]);
    }

    // G-M3 / IO-08-D6a: 正の訂正は直前の ％値引きを取り消す
    #[test]
    fn parse_ej_positive_correction_cancels_percent_discount() {
        let result = parse(&sale_with(
            "000513",
            &[
                item(NAME_A, 500),
                rate(10),
                percent_off("-50"),
                correction("\\50"),
            ],
            1,
            500,
        ));

        assert!(result.diagnostics.is_empty());
        let record = &result.records[0];
        assert_eq!(names(record), [NAME_A]);
        assert!(adjustments_of(record).is_empty());
    }

    // G-M4 / IO-08-D6b: 小計値引きは adjustments に入り、小計の和はマイナスキーを含む
    #[test]
    fn parse_ej_subtotal_discount() {
        let lines = [
            sale_with(
                "000514",
                &[
                    item(NAME_A, 400),
                    item(NAME_B, 600),
                    lr("小計", "\\1,000"),
                    rate(10),
                    percent_off("-100*"),
                ],
                2,
                900,
            ),
            sale_with(
                "000515",
                &[
                    item(NAME_A, 1_000),
                    minus_key("-100"),
                    lr("小計", "\\900"),
                    rate(10),
                    percent_off("-90*"),
                ],
                1,
                810,
            ),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(names(&result.records[0]), [NAME_A, NAME_B]);
        assert_eq!(item_count_of(&result.records[0]), 2);
        assert_eq!(
            adjustment_kinds(&result.records[0]),
            [(EjAdjustmentKind::SubtotalDiscount, -100)]
        );
        assert_eq!(adjustments_of(&result.records[0])[0].line_no, 7);
        assert_eq!(
            adjustment_kinds(&result.records[1]),
            [
                (EjAdjustmentKind::MinusKey, -100),
                (EjAdjustmentKind::SubtotalDiscount, -90)
            ]
        );
    }

    // G-M5 / IO-08-D6b: 明細値引きの掛かり先は直前の明細の行
    #[test]
    fn parse_ej_item_discount_targets_previous_item() {
        let result = parse(&sale_with(
            "000516",
            &[
                item(NAME_A, 200),
                rate(20),
                percent_off("-40"),
                item(NAME_B, 300),
            ],
            2,
            460,
        ));

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            adjustment_kinds(&result.records[0]),
            [(EjAdjustmentKind::ItemDiscount { item_line_no: 3 }, -40)]
        );
        assert_eq!(names(&result.records[0]), [NAME_A, NAME_B]);
    }

    // G-M6 / IO-08-D5a: 名称だけの行 + 金額の行は 1 明細（数量行が前に付く形を含む）
    #[test]
    fn parse_ej_wrapped_item_name() {
        let lines = [
            sale_with(
                "000517",
                &[LONG_NAME.to_string(), lr("", "\\1,200")],
                1,
                1_200,
            ),
            sale_with(
                "000518",
                &[qty(2, "600"), LONG_NAME.to_string(), lr("", "\\1,200")],
                2,
                1_200,
            ),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        let first = &items_of(&result.records[0])[0];
        assert_eq!(
            (
                first.line_no,
                first.name.as_str(),
                first.quantity,
                first.amount
            ),
            (3, LONG_NAME, 1, 1_200)
        );
        let second = &items_of(&result.records[1])[0];
        assert_eq!(
            (
                second.line_no,
                second.quantity,
                second.unit_price,
                second.amount
            ),
            (11, 2, Some(600), 1_200)
        );
        assert_eq!(
            result.records[0].body[0].kind,
            EjLineKind::ItemName {
                name: LONG_NAME.to_string()
            }
        );
        assert_eq!(
            result.records[0].body[1].kind,
            EjLineKind::Continued { amount: 1_200 }
        );
    }

    // G-M7 / IO-08-D6b: マイナスキー（`*` の有無）は adjustments で、点数を変えない
    #[test]
    fn parse_ej_minus_key() {
        let lines = [
            sale_with("000519", &[item(NAME_A, 500), minus_key("-100*")], 1, 400),
            sale_with("000520", &[item(NAME_A, 500), minus_key("-50")], 1, 450),
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            adjustment_kinds(&result.records[0]),
            [(EjAdjustmentKind::MinusKey, -100)]
        );
        assert_eq!(
            adjustment_kinds(&result.records[1]),
            [(EjAdjustmentKind::MinusKey, -50)]
        );
        assert_eq!(item_count_of(&result.records[0]), 1);
    }

    // G-M8 / IO-08-D6d: 戻の印の直後の負の明細は数量 -1・負の金額
    #[test]
    fn parse_ej_return_marked_item() {
        let result = parse(&sale_with(
            "000521",
            &[item(NAME_A, 1_000), return_mark(), lr(NAME_B, "-300")],
            0,
            700,
        ));

        assert!(result.diagnostics.is_empty());
        let items = items_of(&result.records[0]);
        assert_eq!(items.len(), 2);
        assert_eq!(
            (items[1].name.as_str(), items[1].quantity, items[1].amount),
            (NAME_B, -1, -300)
        );
        assert_eq!(items[1].unit_price, None);
    }

    // G-M9 / IO-08-D6 / D6a: すべて取り消した取引は items 空の Restored
    #[test]
    fn parse_ej_all_items_cancelled_restores_empty() {
        let lines = [
            header("", AT, "000522"),
            vec![
                item(NAME_A, 100),
                correction("-100"),
                sep(),
                count(0),
                wide("現金", 0),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            result.records[0].restoration,
            EjRestoration::Restored {
                items: vec![],
                item_count: 0,
                adjustments: vec![],
            }
        );
    }

    // G-C1 / IO-08-D7a: 取引中止の記録は明細を返さない
    #[test]
    fn parse_ej_cancelled_transaction() {
        let lines = [
            header("", AT, "000530"),
            vec![
                item(NAME_A, 120),
                item(NAME_B, 380),
                correction("-380"),
                qty(2, "100"),
                item(NAME_C, 200),
                cancel_mark(),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(kinds(&result), [EjRecordKind::Cancelled]);
        assert!(all_no_items(&result));
        assert_eq!(result.records[0].body[5].kind, EjLineKind::CancelMark);
    }

    // G-C2 / IO-08-D7a: 値引き・マイナスキーを含む取引中止
    #[test]
    fn parse_ej_cancelled_transaction_with_discounts() {
        let lines = [
            header("", AT, "000531"),
            vec![
                item(NAME_A, 500),
                rate(10),
                percent_off("-50"),
                cancel_mark(),
            ],
            header("", AT, "000532"),
            vec![item(NAME_A, 500), minus_key("-100"), cancel_mark()],
        ]
        .concat();
        let result = parse(&lines);

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            kinds(&result),
            [EjRecordKind::Cancelled, EjRecordKind::Cancelled]
        );
        assert!(all_no_items(&result));
    }

    // G-C3 / IO-08-D7a: 取引中止の中の訂正の取消先が合わない
    #[test]
    fn parse_ej_cancelled_transaction_with_mismatched_correction() {
        let lines = [
            header("", AT, "000533"),
            vec![
                item(NAME_A, 120),
                item(NAME_B, 380),
                correction("-120"),
                cancel_mark(),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(result.records[0].kind, EjRecordKind::Cancelled);
        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // G-F1 / IO-08-D6a: 直前でない同額の明細への訂正は取り消さない
    #[test]
    fn parse_ej_correction_of_non_previous_item_unresolves() {
        let result = parse(&sale_with(
            "000540",
            &[item(NAME_A, 120), item(NAME_B, 380), correction("-120")],
            1,
            380,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // G-F2 / IO-08-D6a: 直前の項目の無い訂正・訂正の連続・金額 0 の訂正
    #[test]
    fn parse_ej_correction_without_previous_item_unresolves() {
        let lines = [
            sale_with("000541", &[correction("-100"), item(NAME_A, 100)], 1, 100),
            [
                header("", AT, "000542"),
                vec![
                    item(NAME_A, 100),
                    item(NAME_B, 200),
                    correction("-200"),
                    correction("-100"),
                    sep(),
                    count(0),
                    wide("現金", 0),
                ],
            ]
            .concat(),
            sale_with("000543", &[item(NAME_A, 100), correction("\\0")], 1, 100),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [InconsistentRecord]);
        }
    }

    // G-F3 / IO-08-D6b: 率の行と ％値引きは対
    #[test]
    fn parse_ej_rate_and_percent_discount_must_pair() {
        let lines = [
            sale_with(
                "000544",
                &[item(NAME_A, 500), rate(10), item(NAME_B, 500)],
                2,
                1_000,
            ),
            sale_with("000545", &[item(NAME_A, 500), percent_off("-50")], 1, 450),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [InconsistentRecord]);
        }
    }

    // G-F4 / IO-08-D6b: 小計の金額は前の明細の和
    #[test]
    fn parse_ej_subtotal_mismatch_unresolves() {
        let result = parse(&sale_with(
            "000546",
            &[
                item(NAME_A, 400),
                item(NAME_B, 600),
                lr("小計", "\\900"),
                rate(10),
                percent_off("-100*"),
            ],
            2,
            900,
        ));

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // G-F5 / IO-08-D6b: `*` つきは小計の後、`*` なしは明細の後
    #[test]
    fn parse_ej_subtotal_and_item_discount_are_not_swapped() {
        let lines = [
            sale_with(
                "000547",
                &[item(NAME_A, 1_000), rate(10), percent_off("-100*")],
                1,
                900,
            ),
            sale_with(
                "000548",
                &[
                    item(NAME_A, 1_000),
                    lr("小計", "\\1,000"),
                    rate(10),
                    percent_off("-100"),
                ],
                1,
                900,
            ),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [InconsistentRecord]);
        }
    }

    // G-F6 / IO-08-D6b: ％値引き・マイナスキーの金額は負
    #[test]
    fn parse_ej_positive_discount_unresolves() {
        let lines = [
            sale_with(
                "000549",
                &[item(NAME_A, 1_000), rate(10), percent_off("\\100")],
                1,
                1_100,
            ),
            sale_with(
                "000550",
                &[item(NAME_A, 1_000), minus_key("\\100")],
                1,
                1_100,
            ),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [InconsistentRecord]);
        }
    }

    // G-F7 / IO-08-D6d: 戻のモードでは戻の印を受けない
    #[test]
    fn parse_ej_return_mark_in_return_mode_unresolves() {
        let lines = [
            header("戻", AT, "000551"),
            vec![
                item(NAME_A, 1_000),
                return_mark(),
                lr(NAME_B, "-300"),
                sep(),
                count(0),
                wide("合  計", 700),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(reasons_of(&result.records[0]), [InconsistentRecord]);
    }

    // G-F8 / IO-08-D6d: 戻の印の無い負の明細、戻の印の直後の正の明細
    #[test]
    fn parse_ej_negative_item_without_return_mark_unresolves() {
        let lines = [
            sale_with("000552", &[item(NAME_A, 1_000), lr(NAME_B, "-300")], 0, 700),
            sale_with("000553", &[item(NAME_A, 1_000), item(NAME_B, -300)], 0, 700),
            sale_with(
                "000554",
                &[item(NAME_A, 1_000), return_mark(), item(NAME_B, 300)],
                2,
                1_300,
            ),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [InconsistentRecord]);
        }
    }

    // G-F11 / IO-08-D7a: 区切りのある取引の明細域の取引中止の印は未知の行
    #[test]
    fn parse_ej_cancel_mark_in_sale_unresolves() {
        let result = parse(&sale_with(
            "000555",
            &[item(NAME_A, 100), cancel_mark()],
            1,
            100,
        ));

        let record = &result.records[0];
        assert_eq!(record.kind, EjRecordKind::Sale);
        assert_eq!(reasons_of(record), [UnknownLine]);
        assert_eq!(record.body[1].kind, EjLineKind::Unknown);
    }

    // G-F13 / IO-08-D5 / D5a: 前の行の無い続きの行、読めない明細の行の後の続きの行
    #[test]
    fn parse_ej_continued_line_without_name_line_unresolves() {
        let lines = [
            sale_with("000556", &[item(NAME_A, 100), lr("", "\\1,200")], 2, 1_300),
            one_item_sale(
                "000557",
                100,
                &[wide("合  計", 100), lr("", &zen_amount(100))],
            ),
            sale_with(
                "000558",
                &[lr("A", "\\9223372036854775808"), lr("", &yen(100))],
                1,
                100,
            ),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [UnknownLine]);
        }
        let overflow = &result.records[2];
        assert_eq!(overflow.body[0].kind, EjLineKind::Unknown);
        assert_eq!(overflow.body[1].kind, EjLineKind::Unknown);
        let unknown_lines = result
            .diagnostics
            .iter()
            .filter(|d| d.code == UnknownLine && d.line_no >= Some(overflow.header_line_no))
            .count();
        assert_eq!(unknown_lines, 2);
    }

    // G-F14 / IO-08-D5: 小数の数量行と小数の点数は受理しない
    #[test]
    fn parse_ej_decimal_quantity_and_count_unresolve() {
        let lines = [
            sale_with(
                "000559",
                &["  1.5 点   @100".to_string(), item(NAME_A, 150)],
                1,
                150,
            ),
            [
                header("", AT, "000560"),
                vec![
                    item(NAME_A, 150),
                    sep(),
                    lr("", "1.5 点"),
                    wide("合  計", 150),
                ],
            ]
            .concat(),
        ]
        .concat();
        let result = parse(&lines);

        for record in &result.records {
            assert_eq!(reasons_of(record), [UnknownLine]);
        }
    }

    // G-F16 / IO-08-D5a: 明細域のほかの行種の形は名称の行にしない
    #[test]
    fn parse_ej_item_name_rejects_other_line_shapes() {
        let record_with = |x: &str| {
            parse(
                &[
                    header("", AT, "000561"),
                    vec![
                        item(NAME_B, 50),
                        x.to_string(),
                        lr("", "\\100"),
                        sep(),
                        count(2),
                        wide("合  計", 150),
                    ],
                ]
                .concat(),
            )
        };
        let shapes = [
            "取引中止 ････",
            "＃ 12",
            "SD設定書込み 正常終了",
            "合  計",
            "現金",
            "商品券",
            "対象計 8.0%",
            REDUCED_TAX_NOTE,
            "替 ｺｵｳｶﾝ",
            "一連No.000123",
            "領収No.1",
            "総売 12",
            "領収書 3 件",
            "取引中止 2 件",
            "ﾄﾚｰﾆﾝｸﾞﾓｰﾄﾞを開始します",
            "･･････トレーニング･･････",
            "日計明細",
            "0000 日計明細 X",
            "在売点検 X",
        ];
        for x in shapes {
            let result = record_with(x);
            let record = &result.records[0];
            assert_eq!(
                (&record.body[1].kind, &record.body[2].kind),
                (&EjLineKind::Unknown, &EjLineKind::Unknown),
                "X = {x}"
            );
            assert!(
                matches!(&record.restoration, EjRestoration::Unresolved { reasons } if reasons == &[UnknownLine]),
                "X = {x}: {:?}",
                record.restoration
            );
        }

        let control = record_with("ｼﾞｮｰｾｲ 500ml");
        assert!(control.diagnostics.is_empty());
        assert_eq!(items_of(&control.records[0]).len(), 2);
    }

    // G-F17 / IO-08-D5: 単価の全角の `－` も符号として拒む
    #[test]
    fn parse_ej_full_width_minus_unit_price_unresolves() {
        for price in ["－１", "－０", "-1", "-0"] {
            assert_eq!(
                parse_quantity(&format!(" 1 点   @{price}")),
                None,
                "@{price}"
            );
        }
        assert_eq!(parse_quantity(" 1 点   @100"), Some((1, 100)));

        let lines = [
            header("", AT, "000562"),
            vec![
                " 1 点 @－０".to_string(),
                item(NAME_A, 0),
                sep(),
                count(1),
                wide("合  計", 0),
            ],
            header("", AT, "000563"),
            vec![
                " 1 点 @－１".to_string(),
                lr(NAME_A, "-1"),
                sep(),
                count(1),
                lr("合  計", "－１"),
            ],
        ]
        .concat();
        let result = parse(&lines);

        assert_eq!(result.records.len(), 2);
        for record in &result.records {
            assert_eq!(record.body[0].kind, EjLineKind::Unknown);
            assert_eq!(reasons_of(record), [UnknownLine]);
        }
    }

    // G-F18 / IO-08-D5a: 合計域の続きの行の受け手はラベルだけの行と `対象計` の率だけの行
    #[test]
    fn parse_ej_totals_continued_requires_bare_label_or_taxable_rate() {
        let cases = [
            (
                vec![
                    wide("合  計", 100),
                    REDUCED_TAX_NOTE.to_string(),
                    lr("", &zen_amount(100)),
                ],
                5,
            ),
            (
                vec![lr("内税", "8.0%"), lr("", "\\9"), wide("合  計", 100)],
                4,
            ),
            (
                vec![
                    wide("合  計", 100),
                    lr("現金", "abc"),
                    lr("", &zen_amount(100)),
                ],
                5,
            ),
        ];
        for (tail, at) in cases {
            let result = parse(&one_item_sale("000564", 100, &tail));
            let record = &result.records[0];
            assert_eq!(record.body[at].kind, EjLineKind::Unknown, "{tail:?}");
            assert_eq!(reasons_of(record), [UnknownLine], "{tail:?}");
        }

        let control = parse(&one_item_sale(
            "000565",
            100,
            &[lr("対象計※", "8.0%"), lr("", "\\100"), wide("合  計", 100)],
        ));
        assert!(control.diagnostics.is_empty());
        assert_eq!(
            control.records[0].body[4].kind,
            EjLineKind::Continued { amount: 100 }
        );
    }

    // G-F19 / IO-08-D5 / D5a: 範囲外の金額の `MinusKey` の形は名称の行にしない
    #[test]
    fn parse_ej_item_name_rejects_starred_overflow() {
        for amount in ["-9223372036854775809*", "9223372036854775808*"] {
            let result = parse(&sale_with(
                "000567",
                &[lr("－", amount), lr("", &yen(100))],
                1,
                100,
            ));
            let record = &result.records[0];
            assert_eq!(
                (&record.body[0].kind, &record.body[1].kind),
                (&EjLineKind::Unknown, &EjLineKind::Unknown),
                "{amount}"
            );
            assert_eq!(reasons_of(record), [UnknownLine], "{amount}");
            let unknown_lines = result
                .diagnostics
                .iter()
                .filter(|d| d.code == UnknownLine)
                .count();
            assert_eq!(unknown_lines, 2, "{amount}");
        }
    }

    // G-T9 / IO-08-D6a / D6c: 合計域の訂正は折り返した支払行を取り消す
    #[test]
    fn parse_ej_total_region_correction_cancels_wrapped_payment() {
        let result = parse(&one_item_sale(
            "000566",
            100,
            &[
                "ｸﾚｼﾞｯﾄ電子M".to_string(),
                lr("", &zen_amount(100)),
                correction("-100"),
                wide("現金", 100),
            ],
        ));

        assert!(result.diagnostics.is_empty());
        restored_one(&result.records[0], 100);
        let body = &result.records[0].body;
        assert_eq!(body[4].kind, EjLineKind::Continued { amount: 100 });
        assert_eq!(body[5].kind, EjLineKind::Correction { amount: -100 });
    }

    // -----------------------------------------------------------------------
    // 致命的エラー
    // -----------------------------------------------------------------------

    // IO-08-D1: CP932 として不正なバイト列
    #[test]
    fn parse_ej_decode_failure_is_fatal() {
        let mut broken = vec![0x81, 0x20];
        broken.resize(LINE_WIDTH, b' ');
        let mut all = rows(&ok_sale("000190"));
        all.push(broken);
        match parse_ej(&crlf(all)) {
            Err(EjParseError::DecodeFailed(message)) => assert_eq!(
                message,
                "CP932デコードに失敗しました。ファイル形式を確認してください"
            ),
            other => panic!("DecodeFailed を期待しました: {other:?}"),
        }
    }

    // IO-08-D1: 記録ヘッダが 1 つも無い
    #[test]
    fn parse_ej_no_record_header_is_fatal() {
        let raw = file(&[item(NAME_A, 100), sep(), count(1)]);
        match parse_ej(&raw) {
            Err(EjParseError::NoRecords(message)) => assert_eq!(
                message,
                "記録のヘッダがありません。ファイル形式を確認してください"
            ),
            other => panic!("NoRecords を期待しました: {other:?}"),
        }
    }

    // IO-08-D1: 空入力
    #[test]
    fn parse_ej_empty_input_is_fatal() {
        match parse_ej(&[]) {
            Err(EjParseError::Empty(message)) => {
                assert_eq!(message, "ファイルが空です。ファイル形式を確認してください")
            }
            other => panic!("Empty を期待しました: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // 不変条件
    // -----------------------------------------------------------------------

    /// 7 code をすべて含む file（最後の行は改行なし）
    fn mixed_file() -> (Vec<u8>, usize) {
        let mut wide_row = row(&item(NAME_B, 380));
        wide_row.push(b' ');
        let mut all = rows(&[sep()]);
        all.extend(rows(&ok_sale("000200")));
        all.extend(rows(&sale(
            "",
            "000201",
            &[item(NAME_A, 120), lr("ﾃｲｾｲﾋｮｳｼﾞ", "")],
            1,
            120,
        )));
        all.extend(rows(&header("", AT, "000202")));
        all.push(wide_row);
        all.extend(rows(&totals(1, 380)));
        all.extend(rows(&sale("ZZZ", "000203", &[item(NAME_A, 100)], 1, 100)));
        all.extend(rows(&sale("", "000204", &[item(NAME_A, 100)], 2, 100)));
        all.extend(rows(&program(AT, "000205")));
        // G-X1: 診断を出さない新しい行種の記録（dedup した 7 code の順を崩さない）
        all.extend(rows(
            &[
                header("", AT, "000230"),
                vec![
                    item(NAME_A, 100),
                    lr("-10%", ""),
                    lr("％－", "-10"),
                    lr("取引中止 ････", ""),
                ],
            ]
            .concat(),
        ));
        all.extend(rows(
            &[
                header("練習", AT, "000231"),
                vec![item(NAME_A, 100), "･･････トレーニング･･････".to_string()],
            ]
            .concat(),
        ));
        all.extend(rows(
            &[
                header("精算", AT, "000232"),
                vec![
                    "0007 日計明細  Z 0003".to_string(),
                    lr("ｽﾏ-ﾄﾌｫﾝ送信", "異常終了"),
                ],
            ]
            .concat(),
        ));
        all.extend(rows(
            &[header("PGM3", AT, "000233"), vec!["ﾋｮｳｼﾞ ｾｯﾃｲ".to_string()]].concat(),
        ));
        all.extend(rows(
            &[
                header("", AT, "000234"),
                vec![
                    " 一連No.000233".to_string(),
                    " 領収No.1".to_string(),
                    wide("領収書", 100),
                ],
            ]
            .concat(),
        ));
        all.extend(rows(
            &[
                header("", AT, "000235"),
                vec![
                    "ﾅｶﾞｲﾒｲｼｮｳﾉ ﾃｽﾄｼｮｳﾋﾝ".to_string(),
                    lr("", "\\300"),
                    item(NAME_B, 200),
                    lr("訂正", "-200"),
                    item(NAME_A, 100),
                    lr("－", "-50"),
                    sep(),
                    count(2),
                    wide("合  計", 350),
                    lr("＃ 0012", ""),
                ],
            ]
            .concat(),
        ));
        all.extend(rows(&settlement("000206")));
        all.extend(rows(
            &[header("", AT, "000207"), vec![wide("入金", 3_000)]].concat(),
        ));
        all.extend(rows(&header("", AT, "000208")));
        all.extend(rows(&ok_sale("000209")));
        let n = all.len();
        let mut raw = crlf(all);
        raw.truncate(raw.len() - 2);
        (raw, n)
    }

    // 不変条件 / IO-08-D8: どの行も失われず、診断の文言に生の行が入らない
    #[test]
    fn parse_ej_every_line_is_accounted_for() {
        let (raw, n) = mixed_file();
        let result = parse_ej(&raw).unwrap();

        let body_lines: usize = result.records.iter().map(|r| r.body.len()).sum();
        assert_eq!(
            result.leading_lines.len() + 2 * result.records.len() + body_lines,
            n
        );

        let mut covered: Vec<usize> = result.leading_lines.iter().map(|l| l.line_no).collect();
        for record in &result.records {
            covered.extend([record.header_line_no, record.header_line_no + 1]);
            covered.extend(record.body.iter().map(|l| l.line_no));
        }
        covered.sort_unstable();
        assert_eq!(covered, (1..=n).collect::<Vec<_>>());

        // G-X1: 足した記録は診断を出さずに読める（dedup で隠れないよう個別に見る）
        let added: Vec<_> = result
            .records
            .iter()
            .filter(|r| ("000230"..="000235").contains(&r.number.as_str()))
            .map(|r| {
                (
                    r.kind.clone(),
                    matches!(r.restoration, EjRestoration::Unresolved { .. }),
                )
            })
            .collect();
        assert_eq!(
            added,
            [
                (EjRecordKind::Cancelled, false),
                (EjRecordKind::Training, false),
                (
                    EjRecordKind::Settlement {
                        report: EjSettlementReport::Daily,
                        completed: false,
                    },
                    false
                ),
                (EjRecordKind::Settings, false),
                (EjRecordKind::Receipt, false),
                (EjRecordKind::Sale, false),
            ]
        );

        assert!(!result.diagnostics.is_empty());
        for diagnostic in &result.diagnostics {
            for name in [
                NAME_A,
                NAME_B,
                "ﾃｲｾｲﾋｮｳｼﾞ",
                "ZZZ",
                "ﾅｶﾞｲﾒｲｼｮｳﾉ ﾃｽﾄｼｮｳﾋﾝ",
                "ﾋｮｳｼﾞ ｾｯﾃｲ",
            ] {
                assert!(
                    !diagnostic.message.contains(name),
                    "文言に名称が入っています"
                );
            }
        }
    }

    // IO-08-D8: 診断の message は code ごとの固定文言と完全一致する
    #[test]
    fn parse_ej_diagnostic_messages_are_fixed_texts() {
        let expected = |code| match code {
            UnknownLine => "解釈できない行があります。この記録は明細を復元できません",
            InvalidWidth => "行の長さが24バイトではありません。この記録は明細を復元できません",
            UnrecognizedMode => "未知のモードの記録です。この記録は明細を復元できません",
            InconsistentRecord => {
                "取引内の数量・点数・合計が一致しません。この記録は明細を復元できません"
            }
            IncompleteRecord => "記録に必須の行がありません。この記録は明細を復元できません",
            MissingFinalNewline => {
                "ファイルの最後の行に改行がありません。最後の記録は途中で切れたものとして扱います"
            }
            LeadingFragment => {
                "最初の記録より前に行があります。ファイルは記録の途中から始まっています"
            }
        };
        let (raw, _) = mixed_file();
        let result = parse_ej(&raw).unwrap();

        let mut codes: Vec<_> = result.diagnostics.iter().map(|d| d.code).collect();
        codes.dedup();
        assert_eq!(
            codes,
            [
                LeadingFragment,
                UnknownLine,
                InvalidWidth,
                UnrecognizedMode,
                InconsistentRecord,
                IncompleteRecord,
                MissingFinalNewline,
            ]
        );
        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.message, expected(diagnostic.code));
        }
    }

    // -----------------------------------------------------------------------
    // 実物確認（Coordinator 手元、CI では実行しない）
    // -----------------------------------------------------------------------

    /// 明細と値引きの digest（SHA-256 の先頭 16 桁）。値を出さずに base と突き合わせる（S3）
    fn restoration_digest(items: &[EjItem], adjustments: &[EjAdjustment]) -> String {
        let mut text = items
            .iter()
            .map(|item| {
                let unit_price = item.unit_price.map_or("-".to_string(), |p| p.to_string());
                format!(
                    "{}\x1f{}\x1f{}\x1f{}",
                    item.name, item.quantity, unit_price, item.amount
                )
            })
            .collect::<Vec<_>>()
            .join("\x1e");
        if !adjustments.is_empty() {
            let adjustments = adjustments
                .iter()
                .map(|adjustment| {
                    let kind = match adjustment.kind {
                        EjAdjustmentKind::ItemDiscount { .. } => "ItemDiscount",
                        EjAdjustmentKind::SubtotalDiscount => "SubtotalDiscount",
                        EjAdjustmentKind::MinusKey => "MinusKey",
                    };
                    format!("{kind}\x1f{}", adjustment.amount)
                })
                .collect::<Vec<_>>()
                .join("\x1e");
            text = format!("{text}\x1d{adjustments}");
        }
        format!("{:x}", Sha256::digest(text.as_bytes()))[..16].to_string()
    }

    // IO-08-D3a〜D7a: 実物 EJ の構造への一致。出力は件数・file 名・記録の番号・code・digest だけ
    #[test]
    #[ignore = "INVENTORY_EJ_PROBE_DIR に実物のEJを置いた dir を与えて手元で実行する"]
    fn real_ej_structure_probe() {
        use std::collections::BTreeMap;

        let dir = std::env::var_os("INVENTORY_EJ_PROBE_DIR")
            .expect("INVENTORY_EJ_PROBE_DIR が未設定です");
        let mut paths: Vec<_> = std::fs::read_dir(dir)
            .expect("dir を読めません")
            .map(|entry| entry.expect("dir の entry を読めません").path())
            .collect();
        paths.sort();

        let (mut read, mut skipped, mut fatal) = (0, 0, 0);
        let mut modes: BTreeMap<&str, usize> = BTreeMap::new();
        let mut restorations: BTreeMap<&str, usize> = BTreeMap::new();
        let mut kinds: BTreeMap<(String, &str), usize> = BTreeMap::new();
        let mut codes: BTreeMap<String, usize> = BTreeMap::new();
        let mut records: Vec<String> = Vec::new();
        for path in paths {
            let is_txt = path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"));
            if !is_txt {
                skipped += 1;
                continue;
            }
            read += 1;
            let file_name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let bytes = std::fs::read(&path).expect("file を読めません");
            let Ok(result) = parse_ej(&bytes) else {
                fatal += 1;
                continue;
            };
            for record in &result.records {
                let mode = match record.mode {
                    EjMode::Normal => "Normal",
                    EjMode::Return => "Return",
                    EjMode::Training => "Training",
                    EjMode::Settlement => "Settlement",
                    EjMode::Inspection => "Inspection",
                    EjMode::Program => "Program",
                    EjMode::Program1 => "Program1",
                    EjMode::Program3 => "Program3",
                    EjMode::Off => "Off",
                    EjMode::Unrecognized(_) => "Unrecognized",
                };
                *modes.entry(mode).or_default() += 1;
                let kind = format!("{:?}", record.kind);
                let number = &record.number;
                let restoration = match &record.restoration {
                    EjRestoration::Restored {
                        items, adjustments, ..
                    } => {
                        let digest = restoration_digest(items, adjustments);
                        records.push(format!("{file_name} {number} Restored {digest}"));
                        "Restored"
                    }
                    EjRestoration::NoItems => {
                        let digest = restoration_digest(&[], &[]);
                        records.push(format!("{file_name} {number} NoItems {digest}"));
                        "NoItems"
                    }
                    EjRestoration::Unresolved { reasons } => {
                        records.push(format!(
                            "{file_name} {number} Unresolved {kind} {reasons:?}"
                        ));
                        "Unresolved"
                    }
                };
                *restorations.entry(restoration).or_default() += 1;
                *kinds.entry((kind, restoration)).or_default() += 1;
            }
            for diagnostic in &result.diagnostics {
                *codes.entry(format!("{:?}", diagnostic.code)).or_default() += 1;
            }
        }

        println!("files_read {read} files_skipped {skipped} fatal {fatal}");
        println!("modes {modes:?}");
        println!("restorations {restorations:?}");
        println!("kinds {kinds:?}");
        println!("diagnostics {codes:?}");
        for line in &records {
            println!("record {line}");
        }
        assert!(read > 0, "読んだ file が 0 本です");
        assert_eq!(fatal, 0);
    }
}
