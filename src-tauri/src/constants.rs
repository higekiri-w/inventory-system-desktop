// 第4段階（POS連携）の定数定義
// docs・BIZ・CMD で同一値を参照するため一元管理

// --- Pagination ---
/// 一覧系IO層で共有する per_page の最大値（D-031）。
pub const PAGINATION_MAX_PER_PAGE: u32 = 200;

// --- PLU書出し ---
/// PLUメモリ全体の上限数（カシオ SR-S4000 の仕様）
pub const PLU_EXPORT_LIMIT: usize = 5000;

/// SR-S4000 の工場出荷時配分（取説確認済み）で通常PLUが使用しているメモリ数。
///
/// SR-S4000 の PLU 総枠 5000 は通常PLUとスキャニングPLUで共有される。
/// 本来は通常PLUをSD/CV17へ書き込んだ結果の件数から導く値であり、店舗operator向け設定ではない。
/// 2026-07-02 field gate では通常PLU 216枠使用により、スキャニングPLU開始が217として観測された。
pub const DEFAULT_STANDARD_PLU_MEMORY_COUNT: usize = 216;

/// 通常PLU使用数からスキャニングPLUの開始メモリNo.を求める。
pub const fn scanning_plu_memory_start(standard_plu_count: usize) -> usize {
    standard_plu_count + 1
}

/// 通常PLU使用数からスキャニングPLUの実効書出し上限を求める。
pub const fn scanning_plu_export_limit(standard_plu_count: usize) -> usize {
    PLU_EXPORT_LIMIT.saturating_sub(standard_plu_count)
}

/// 現地観測値に基づくスキャニングPLU開始メモリNo.
pub const SCANNING_PLU_MEMORY_START: usize =
    scanning_plu_memory_start(DEFAULT_STANDARD_PLU_MEMORY_COUNT);

/// 現地観測値に基づくスキャニングPLUの実効書出し上限。
pub const SCANNING_PLU_EXPORT_LIMIT: usize =
    scanning_plu_export_limit(DEFAULT_STANDARD_PLU_MEMORY_COUNT);

/// release_pending の clear 行を出力・confirm 時に解放する。
pub const PLU_CLEAR_ROW_ENABLED: bool = true;

// --- CSV取込み ---
/// Z004ファイルのサイズ上限（バイト数）
pub const CSV_IMPORT_FILE_SIZE_LIMIT: usize = 20 * 1024 * 1024; // 20MB

/// Z004ファイルのデータ行数上限
pub const CSV_IMPORT_LINE_LIMIT: usize = 10_000;

// --- 商品マスタ ---
/// 商品コードの長さ上限（BIZ-01-D5）。
///
/// UTF-16 code unit で数える。frontend の route search（`selected` / `q`）の
/// zod `.max(100)` と同じ数え方で、登録できる商品コードは在庫照会でも選べる。
pub const PRODUCT_CODE_MAX_LEN: usize = 100;

// --- 入力の上限と安全な整数の範囲（10-common-rules.md SPEC-UNIT-D11 の続き (a)） ---
/// 1 明細の数量の上限（入庫・返品・手動販売・廃棄・POS 由来の数量。長さは cm）
pub const MAX_LINE_QUANTITY: i64 = 9_999_999;

/// 在庫の絶対値の上限（計算の後の値を含む。長さは cm）
pub const MAX_STOCK_QUANTITY: i64 = 999_999_999;

/// 売価の上限（円）
pub const MAX_SELLING_PRICE: i64 = 999_999;

/// 原価の上限（1/100 円。999999.99 円。SPEC-UNIT-D5）
pub const MAX_COST_PRICE_CENTI: i64 = 99_999_999;

/// JavaScript の number で正確に運べる整数の絶対値の上限（2^53 - 1）
pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// レジの日報（Z001・Z002・Z005）の金額・件数の絶対値の上限（IO-07-D6、D-142）
pub const MAX_DAILY_REPORT_INTEGER: i64 = 9_999_999_999;

/// レジの日報の個数（100 倍の整数）の絶対値の上限。`9999999.99`（IO-07-D6、D-142）
pub const MAX_DAILY_REPORT_QUANTITY_HUNDREDTHS: i64 = 999_999_999;

// --- プレビューキャッシュ ---
/// preview_cache の最大エントリ数（FIFO: 上限超過時に最古を削除）
pub const PREVIEW_CACHE_LIMIT: usize = 10;

/// preview_token の有効期限（秒）
pub const PREVIEW_CACHE_TTL_SECS: u64 = 30 * 60; // 30分

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanning_plu_memory_profile_req402_uses_shared_total_memory() {
        // REQ-402: PLU書出し
        // SR-S4000: PLU総枠5000を通常PLUとスキャニングPLUで共有する
        assert_eq!(scanning_plu_memory_start(250), 251);
        assert_eq!(scanning_plu_export_limit(250), 4750);
        assert_eq!(DEFAULT_STANDARD_PLU_MEMORY_COUNT, 216);
        assert_eq!(SCANNING_PLU_MEMORY_START, 217);
        assert_eq!(SCANNING_PLU_EXPORT_LIMIT, 4784);
    }
}
