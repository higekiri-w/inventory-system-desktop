//! BIZ-03: CSV取込みパイプライン
//!
//! ARCHITECTURE.md タスク仕様 BIZ-03 + docs/function-design/32-biz-csv-import-service.md に基づく実装。
//! 4段階パイプライン: Parse → Validate → Preview → Commit。
//! キャッシュ管理はCMD層の責務（BIZ層はキャッシュを保持しない）。

mod commit;
mod list;
mod parse;
mod rollback;

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;

/// 欠けた精算回数の区間（32 の知らせ、40 の `SettlementPendingHint.missing`。1 つの番号なら from = to）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct SettlementGap {
    pub from: i64,
    pub to: i64,
}

/// 昇順で重複の無い精算回数の並びの間の欠けを、連続した区間にまとめて昇順で返す
/// （区間の数は欠けた番号の数に依らない。32）
pub(crate) fn missing_ranges(present: &[i64]) -> Vec<SettlementGap> {
    present
        .windows(2)
        .filter(|pair| pair[1] - pair[0] > 1)
        .map(|pair| SettlementGap {
            from: pair[0] + 1,
            to: pair[1] - 1,
        })
        .collect()
}

// CMD層が使用する API を一元化するため、各処理の再エクスポートを維持
#[allow(unused_imports)]
pub use commit::commit_csv_import;
#[allow(unused_imports)]
pub use list::{get_csv_import_record, list_csv_imports};
#[allow(unused_imports)]
pub use parse::parse_and_validate;
#[allow(unused_imports)]
pub use rollback::{preview_rollback_csv_import, rollback_csv_import};

use serde::Serialize;

// ---------------------------------------------------------------------------
// 型定義（設計書 §15.2 準拠、15型）
// ---------------------------------------------------------------------------

/// parse_and_validate のリクエスト
#[derive(Debug)]
pub struct CsvParseAndValidateRequest {
    /// Z004ファイルの生バイト列
    pub file_bytes: Vec<u8>,
    /// ファイル名（csv_imports.filename に記録する表示用）
    pub filename: String,
    /// CMD が渡す PC の今日（YYYY-MM-DD。精算日が今日より後の知らせ。BIZ は時計を読まない）
    pub today: String,
}

/// parse_and_validate の結果
#[derive(Debug)]
pub struct ParseValidateResult {
    /// フロントエンドに返すプレビューデータ
    pub preview_data: PreviewData,
    /// CMD層がキャッシュキーとして使用するUUID v4
    pub preview_token: String,
    /// CMD層がキャッシュに保存する（フロントエンドには返さない）
    pub matched_rows: Vec<MatchedRow>,
    /// CMD層がキャッシュに保存する（フロントエンドには返さない）
    pub error_rows: Vec<ErrorRow>,
    /// CMD層がキャッシュに保存する（順 1 を通った精算の識別メタ）
    pub settlement: CachedSettlement,
}

/// フロントエンドに返すプレビューデータ
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct PreviewData {
    pub file_info: FileInfo,
    pub matched_summary: MatchedSummary,
    pub error_summary: ErrorSummary,
    /// 知らせ (a): 取り込まれていない精算回数の区間（昇順。取込みを止めない）
    pub settlement_gaps: Vec<SettlementGap>,
    /// 知らせ (b): 精算日が PC の今日より後
    pub settlement_date_after_today: bool,
    pub preview_created_at: String,
}

/// ファイル情報
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct FileInfo {
    pub filename: String,
    /// YYYY-MM-DD
    pub settlement_date: String,
    /// SHA-256 hex、小文字64文字
    pub file_hash: String,
}

/// マッチ成功サマリ
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct MatchedSummary {
    /// 紐付け成功件数
    pub count: usize,
    /// matched_rows の amount 合計
    pub total_amount: i64,
    /// グループコード商品の紐付け警告等
    pub warnings: Vec<String>,
}

/// エラーサマリ
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct ErrorSummary {
    /// エラー行の総数
    pub count: usize,
    /// 最大100件（UI表示の上限）
    pub items: Vec<ErrorRow>,
}

/// マスタ照合成功行（サーバ側キャッシュに保持、フロントエンドには送らない。32「preview の cache の契約」）
#[derive(Debug, Clone)]
pub struct MatchedRow {
    /// Z004の行番号（1始まり）
    pub line_no: usize,
    /// IO-02 が正規化した 13 桁の JAN（確定の TX の照合に使う）
    pub normalized_jan: String,
    /// preview で引き当てた先頭の商品コード
    pub product_code: String,
    /// preview の時点でその JAN を持つ商品が 2 つ以上
    pub shared_jan: bool,
    /// 在庫の数量に換えた後の値（長さの商品は × 100 の cm）。売上帳票視点、正=販売、負=返品
    pub quantity: i64,
    pub amount: i32,
}

/// エラー行（フロント送信 + キャッシュ兼用）
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct ErrorRow {
    pub line_no: usize,
    /// JAN正規化前にエラーならNone
    pub normalized_jan: Option<String>,
    pub name: String,
    pub raw_quantity: String,
    pub raw_amount: String,
    /// "unmatched_product" / "invalid_format" / "invalid_jan" / "invalid_number"
    pub error_type: CsvImportErrorType,
    /// 利用者向け日本語メッセージ
    pub error_message: String,
}

/// commit_csv_import のリクエスト（CMD層が組み立て）
#[derive(Debug)]
pub struct CommitRequest {
    /// CMD層がキャッシュから復元したデータ
    pub cached_data: CachedPreview,
    /// CMD が渡す PC の今日（知らせのため）
    pub today: String,
}

/// 精算の識別メタ（parse の時点で順 1 を通った値。確定の TX はこれで順 1〜5 を取り直す）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedSettlement {
    pub machine_no: String,
    pub settlement_no: i64,
    pub settlement_date: String,
}

/// サーバ側メモリキャッシュ（CMD層の AppState で管理）
#[derive(Debug, Clone)]
pub struct CachedPreview {
    /// キャッシュ作成時刻（有効期限判定に使用）
    pub created_at: std::time::Instant,
    pub matched_rows: Vec<MatchedRow>,
    pub error_rows: Vec<ErrorRow>,
    /// フロントエンドに返した内容のコピー
    pub preview_data: PreviewData,
    pub settlement: CachedSettlement,
}

/// commit_csv_import の結果
#[derive(Debug, Serialize, specta::Type)]
pub struct ImportResult {
    pub csv_import_id: i64,
    /// "completed" / "completed_partial"
    pub status: crate::db::sales_repo::CsvImportStatus,
    pub total_items: i64,
    pub total_amount: i64,
    pub skipped_count: i64,
    pub settlement_gaps: Vec<SettlementGap>,
    pub settlement_date_after_today: bool,
    /// 求め直しの後に要再確認になった商品（40）
    pub recheck_targets: Vec<crate::biz::stocktake_service::CountRecoveryTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum CsvImportErrorType {
    UnmatchedProduct,
    InvalidFormat,
    InvalidJan,
    InvalidNumber,
}

impl CsvImportErrorType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnmatchedProduct => "unmatched_product",
            Self::InvalidFormat => "invalid_format",
            Self::InvalidJan => "invalid_jan",
            Self::InvalidNumber => "invalid_number",
        }
    }

    fn from_db(value: &str) -> Result<Self, crate::db::DbError> {
        match value {
            "unmatched_product" => Ok(Self::UnmatchedProduct),
            "invalid_format" => Ok(Self::InvalidFormat),
            "invalid_jan" => Ok(Self::InvalidJan),
            "invalid_number" => Ok(Self::InvalidNumber),
            other => Err(crate::db::DbError::QueryFailed(format!(
                "unknown csv import error type: {other}"
            ))),
        }
    }
}

/// CSV取込み記録詳細の wire DTO。
///
/// docs/function-design/32-biz-csv-import-service.md §15.6a
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct CsvImportRecordDetail {
    pub id: i64,
    pub filename: String,
    pub settlement_date: String,
    pub total_items: i64,
    pub total_amount: i64,
    pub skipped_count: i64,
    pub status: crate::db::sales_repo::CsvImportStatus,
    pub imported_at: String,
    pub items: Vec<crate::db::sales_repo::CsvImportRecordDetailItem>,
    pub error_rows: Vec<ErrorRow>,
    pub movements: Vec<crate::db::inventory_repo::MovementRecord>,
}

/// rollback_csv_import の結果（補償の movement は作らない。ADR D7）
#[derive(Debug, Serialize, specta::Type)]
pub struct RollbackResult {
    pub success: bool,
    pub voided_sale_count: u64,
    pub voided_movement_count: usize,
    /// 求め直しの後に要再確認になった商品（40）
    pub recheck_targets: Vec<crate::biz::stocktake_service::CountRecoveryTarget>,
}

/// preview_rollback_csv_import の結果（取消の前の読取り。何も書かない。32 §15.6b）
#[derive(Debug, Serialize, specta::Type)]
pub struct RollbackPreview {
    pub csv_import_id: i64,
    /// 取り込み直す精算の番号（この取込みの精算回数）
    pub settlement_no: i64,
    /// 取り消した後に要再確認である商品（取消の結果と同じ作り方。取り消す前から要再確認の商品を含む。40）
    pub recheck_targets: Vec<crate::biz::stocktake_service::CountRecoveryTarget>,
    /// この取込みの精算が完了の後の比較の範囲に入る完了した棚卸し（範囲による候補。完了の日時の昇順。35）
    pub affected_stocktakes: Vec<AffectedStocktake>,
}

/// 取消で記録の詳細の差異・ロス原価・参考の差が変わることがある完了した棚卸し（32 §15.6b）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct AffectedStocktake {
    pub stocktake_id: i64,
    pub completed_at: String,
}
