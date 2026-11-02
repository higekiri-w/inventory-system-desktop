//! parse_and_validate — Stage 1+2+3（Parse + Validate + Preview）

use crate::biz::csv_import_service::{
    missing_ranges, CachedSettlement, CsvParseAndValidateRequest, ErrorRow, ErrorSummary, FileInfo,
    MatchedRow, MatchedSummary, ParseValidateResult, PreviewData, SettlementGap,
};
use crate::biz::unit_amount::{stock_unit_kind, StockUnitKind};
use crate::biz::BizError;
use crate::constants;
use crate::db::product_repo;
use crate::db::sales_repo::{self, CsvImportStatus};
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::DbConnection;
use crate::io::z004_parser::{self, ParseErrorType, SettlementMetadata, Z004ParseError};
use uuid::Uuid;

/// 精算回数の上限（JavaScript の安全な整数。32 の表の順 1）
const MAX_SETTLEMENT_NO: i64 = 9_007_199_254_740_991;

/// 順 1: machine_no と settlement_no（ASCII の数字だけで 0 以上 `9007199254740991` 以下）を持つ（32）
pub(super) fn settlement_identity(
    metadata: Option<&SettlementMetadata>,
    settlement_date: &str,
) -> Result<CachedSettlement, BizError> {
    let reject = || {
        BizError::ImportError(
            "精算回数の無い file は取り込めません。レジの SD の商品別の売上（Z004）を選んでください"
                .to_string(),
        )
    };
    let metadata = metadata.ok_or_else(reject)?;
    let machine_no = metadata
        .machine_no
        .clone()
        .filter(|machine| !machine.is_empty())
        .ok_or_else(reject)?;
    let raw = metadata.settlement_no.as_deref().ok_or_else(reject)?;
    if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(reject());
    }
    let settlement_no = raw
        .parse::<i64>()
        .ok()
        .filter(|no| *no <= MAX_SETTLEMENT_NO)
        .ok_or_else(reject)?;
    Ok(CachedSettlement {
        machine_no,
        settlement_no,
        settlement_date: settlement_date.to_string(),
    })
}

/// 順 2〜5（preview と確定の TX の両方で取り直す。32 の表）。順 3 の文は呼び出し元で違う
pub(super) fn check_settlement_rules(
    conn: &DbConnection,
    settlement: &CachedSettlement,
    file_hash: &str,
    in_commit: bool,
) -> Result<(), BizError> {
    // 順 2: machine_no は取り消した取込みとも比べる（照合先は active を優先する）
    if let Some(other) = sales_repo::find_machine_no_mismatch(conn, &settlement.machine_no)? {
        let message = if other.status == CsvImportStatus::RolledBack {
            format!(
                "取り消した取込みとレジの番号が合いません（取込みID: {}、精算 {}、取込み日: {}、レジ: {}。この file のレジ: {}）。どちらが店のレジの資料かを確かめてください",
                other.import_id,
                other.settlement_no,
                other.imported_at,
                other.machine_no,
                settlement.machine_no
            )
        } else {
            "レジの番号が前の精算と合いません。レジの入替・初期化を確かめてください".to_string()
        };
        return Err(BizError::ImportError(message));
    }
    // 順 3: 同じ hash の active な取込み（今の重複の拒否のまま）
    if let Some(existing) = sales_repo::find_blocking_import_by_file_hash(conn, file_hash)? {
        return Err(BizError::ImportError(if in_commit {
            "このファイルは既に取込み済みです".to_string()
        } else {
            format!(
                "このファイルは取込み済みです（取込みID: {}、取込み日: {}）",
                existing.id, existing.imported_at
            )
        }));
    }
    // 順 4: 同じ精算の active な取込みで hash が違う（取り消した取込みは比べない）。
    // 名指す取込みID は文と error の構造の両方に入れる（画面が詳細への link を出す。55 UI-07-D18）
    if let Some((csv_import_id, _)) = sales_repo::find_active_imports_for_settlement(
        conn,
        &settlement.machine_no,
        settlement.settlement_no,
    )?
    .first()
    {
        return Err(BizError::SettlementConflict {
            message: format!(
                "同じ精算と思われる別の資料が既に取り込まれています（精算 {}、取込みID: {}）",
                settlement.settlement_no, csv_import_id
            ),
            csv_import_id: *csv_import_id,
        });
    }
    // 順 5: 精算日と精算回数の順の食い違い（active だけと比べる）
    let mismatched = sales_repo::list_active_settlements(conn)?
        .iter()
        .any(|other| {
            (other.settlement_no < settlement.settlement_no
                && other.settlement_date > settlement.settlement_date)
                || (other.settlement_no > settlement.settlement_no
                    && other.settlement_date < settlement.settlement_date)
        });
    if mismatched {
        return Err(BizError::ImportError(
            "レジの日付か番号が前の精算と合いません。レジの時計か、レジの入替・初期化を確かめてください"
                .to_string(),
        ));
    }
    Ok(())
}

/// 知らせ (a): active な取込みとこの精算を合わせた最小から最大の間の欠けの区間（32）
pub(super) fn settlement_gaps(
    conn: &DbConnection,
    settlement_no: i64,
) -> Result<Vec<SettlementGap>, BizError> {
    let mut present: Vec<i64> = sales_repo::list_active_settlements(conn)?
        .iter()
        .map(|s| s.settlement_no)
        .collect();
    present.push(settlement_no);
    present.sort_unstable();
    present.dedup();
    Ok(missing_ranges(&present))
}

/// Z004ファイルを解析し、マスタ照合後のプレビューデータを返す
///
/// 業務テーブルへの書き込みなし（parse失敗時のoperation_log記録は例外）。
/// preview_token を生成して返す（キャッシュ保存はCMD層の責務）。
///
/// docs/function-design/32-biz-csv-import-service.md §15.3
pub fn parse_and_validate(
    conn: &DbConnection,
    req: CsvParseAndValidateRequest,
) -> Result<ParseValidateResult, BizError> {
    // 1. サイズガード
    if req.file_bytes.len() > constants::CSV_IMPORT_FILE_SIZE_LIMIT {
        return Err(BizError::ImportError(
            "ファイルサイズが上限（20MB）を超えています".to_string(),
        ));
    }

    // 2. Stage 1: Parse（IO-02 委譲）
    let parse_result = match z004_parser::parse_z004(&req.file_bytes) {
        Ok(result) => result,
        Err(e) => {
            let msg = match &e {
                Z004ParseError::DecodeFailed(_) => {
                    "Z004ファイルの解析に失敗しました: CP932デコードエラー"
                }
                Z004ParseError::NoDataLines(_) => {
                    "Z004ファイルの解析に失敗しました: データ行がありません"
                }
                Z004ParseError::NoSettlementDate(_) => {
                    "Z004ファイルの解析に失敗しました: 精算日を抽出できません"
                }
                Z004ParseError::ImportError(_) => {
                    "Z004ファイルの解析に失敗しました: snapshot形式エラー"
                }
            };
            log_parse_failure(conn, msg);
            return Err(BizError::ImportError(msg.to_string()));
        }
    };

    // parsed_rows が 0件かつ parse_errors が非空
    if parse_result.parsed_rows.is_empty() && !parse_result.parse_errors.is_empty() {
        let msg = "有効なデータがありません";
        log_parse_failure(conn, msg);
        return Err(BizError::ImportError(msg.to_string()));
    }

    // 2a. settlement_date 業務妥当性チェック（精算日が今日より後は拒まず知らせる。32 の知らせ (b)）
    chrono::NaiveDate::parse_from_str(&parse_result.settlement_date, "%Y-%m-%d").map_err(|_| {
        BizError::ImportError(format!(
            "精算日が不正な日付です: {}",
            parse_result.settlement_date
        ))
    })?;

    // 2b. 順 1: 精算の識別メタ
    let settlement = settlement_identity(
        parse_result.settlement_metadata.as_ref(),
        &parse_result.settlement_date,
    )?;

    // 3. 行数ガード
    if parse_result.total_data_lines > constants::CSV_IMPORT_LINE_LIMIT {
        return Err(BizError::ImportError(
            "データ行数が上限（10,000行）を超えています".to_string(),
        ));
    }

    // 4. Stage 2: Validate
    let mut matched_rows: Vec<MatchedRow> = Vec::new();
    let mut error_rows: Vec<ErrorRow> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 4a-b. 実データ行のマスタ照合
    for row in &parse_result.parsed_rows {
        // 空レコード除外（エラーにもカウントしない）
        if row.quantity == 0 && row.amount == 0 {
            continue;
        }

        // 数量は商品を引いた後に在庫の数量へ換える（長さの単位は × 100 の cm。SPEC-UNIT-D4・D8）
        let products = product_repo::find_by_jan_code(conn, &row.normalized_jan)?;
        let per_register_unit = match products.first().map(|p| stock_unit_kind(p.stock_unit)) {
            Some(StockUnitKind::Length) => 100,
            _ => 1,
        };
        let quantity = i64::from(row.quantity) * per_register_unit;
        // POS 由来の数量は換えた後の絶対値の上限まで（共通規則 (a)、X2）。在庫を動かさない行も同じ検査を通る
        if quantity.abs() > constants::MAX_LINE_QUANTITY {
            error_rows.push(ErrorRow {
                line_no: row.line_no,
                normalized_jan: Some(row.normalized_jan.clone()),
                name: row.name.clone(),
                raw_quantity: row.quantity.to_string(),
                raw_amount: row.amount.to_string(),
                error_type: super::CsvImportErrorType::InvalidNumber,
                error_message: "数量が大きすぎます".to_string(),
            });
            continue;
        }

        match products.len() {
            0 => {
                error_rows.push(ErrorRow {
                    line_no: row.line_no,
                    normalized_jan: Some(row.normalized_jan.clone()),
                    name: row.name.clone(),
                    raw_quantity: row.quantity.to_string(),
                    raw_amount: row.amount.to_string(),
                    error_type: super::CsvImportErrorType::UnmatchedProduct,
                    error_message: format!("JAN {} に該当する商品がありません", row.normalized_jan),
                });
            }
            count => {
                // 複数ヒット → 先頭を採用 + warning（売上だけ、在庫連動の外。ADR D4）
                if count > 1 {
                    warnings.push(format!(
                        "JAN {} は複数商品に紐付いています（{} を使用）",
                        row.normalized_jan, products[0].product_code
                    ));
                }
                matched_rows.push(MatchedRow {
                    line_no: row.line_no,
                    normalized_jan: row.normalized_jan.clone(),
                    product_code: products[0].product_code.clone(),
                    shared_jan: count > 1,
                    quantity,
                    amount: row.amount,
                });
            }
        }
    }

    // 4c. parse_errors を ErrorRow にマージ
    for pe in &parse_result.parse_errors {
        let error_type = match pe.error_type {
            ParseErrorType::InvalidFormat => super::CsvImportErrorType::InvalidFormat,
            ParseErrorType::InvalidJan => super::CsvImportErrorType::InvalidJan,
            ParseErrorType::InvalidNumber => super::CsvImportErrorType::InvalidNumber,
        };
        error_rows.push(ErrorRow {
            line_no: pe.line_no,
            normalized_jan: None,
            name: pe.raw_name.clone().unwrap_or_default(),
            raw_quantity: pe.raw_quantity.clone().unwrap_or_default(),
            raw_amount: pe.raw_amount.clone().unwrap_or_default(),
            error_type,
            error_message: pe.error_message.clone(),
        });
    }

    // 正常な行が全て 0 の file も精算の識別メタだけを持つ取込みとして受ける（ADR D4。空の拒否はしない）

    // 5. Stage 3: Preview
    // 5a. 順 2〜5（順 3 は file_hash の重複）。同じ精算日の別の精算は確認なしに進む（ADR D4）
    check_settlement_rules(conn, &settlement, &parse_result.file_hash, false)?;
    // 5b. 知らせ
    let settlement_gaps = settlement_gaps(conn, settlement.settlement_no)?;
    let settlement_date_after_today = parse_result.settlement_date.as_str() > req.today.as_str();

    // 5c. PreviewData 構築
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    // items は最大100件。invalid_jan の行を line_no の昇順で先に選び、残りの枠を
    // 他の行から line_no の昇順で埋め、選んだ items を line_no の昇順で並べる（§15.2 / §15.3 5c）
    let mut error_summary_items = error_rows.clone();
    error_summary_items.sort_by_key(|r| {
        (
            r.error_type != super::CsvImportErrorType::InvalidJan,
            r.line_no,
        )
    });
    error_summary_items.truncate(100);
    error_summary_items.sort_by_key(|r| r.line_no);

    let preview_data = PreviewData {
        file_info: FileInfo {
            filename: req.filename,
            settlement_date: parse_result.settlement_date,
            file_hash: parse_result.file_hash,
        },
        matched_summary: MatchedSummary {
            count: matched_rows.len(),
            total_amount: matched_rows.iter().map(|r| r.amount as i64).sum(),
            warnings,
        },
        error_summary: ErrorSummary {
            count: error_rows.len(),
            items: error_summary_items,
        },
        settlement_gaps,
        settlement_date_after_today,
        preview_created_at: now,
    };

    // 6. preview_token 生成
    let preview_token = Uuid::new_v4().to_string();

    Ok(ParseValidateResult {
        preview_data,
        preview_token,
        matched_rows,
        error_rows,
        settlement,
    })
}

/// parse失敗時の操作ログ記録（best-effort）
fn log_parse_failure(conn: &DbConnection, message: &str) {
    let log = NewOperationLog {
        operation_type: "csv_import_parse_failed".to_string(),
        summary: message.to_string(),
        detail_json: None,
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }
}
