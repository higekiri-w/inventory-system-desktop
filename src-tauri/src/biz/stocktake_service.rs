//! BIZ-06: 棚卸しロジック
//!
//! 棚卸しの開始・計数の保存・完了を管理し、仕入原価総額を固定する。
//! 年末の長期作業（10月〜大晦日）で、4000商品を順次カウントする。
//!
//! docs/function-design/35-biz-stocktake-service.md「計数の置き場と在庫の求め直しの契約」
//! （ADR SPEC-STK-CNT-D3・D5・D6）に基づく実装。計数は `stock_counts` に保存し（消さない・書き換えない）、
//! 写しは `biz::stock_ledger` が記録から求め直す。完了は在庫を動かさず評価額を固定する。

use crate::biz::csv_import_service::{missing_ranges, AffectedStocktake, SettlementGap};
use crate::biz::inventory_service::validate_business_date;
use crate::biz::stock_ledger::{
    self, is_after, read_ledger, recompute_stock, representative_count, settlement_no_within,
    usable_cursor, DerivedStock, Ledger, RecheckReason,
};
use crate::biz::unit_amount::{
    check_safe_integer, cost_line_centi, cost_total_yen, price_basis_quantity,
};
use crate::biz::BizError;
use crate::constants::MAX_STOCK_QUANTITY;
use crate::db::inventory_repo::{self, NewStockCount, StockCountRow};
use crate::db::product_repo::ProductStockUnit;
use crate::db::sales_repo::{self, SettlementRow};
use crate::db::stocktake_repo::{StockCopyRow, StocktakeCutoff, StocktakeItemRow};
use crate::db::{
    stocktake_repo, system_repo, DbConnection, DbError, NewOperationLog, NewStocktakeItem,
};

// ---------------------------------------------------------------------------
// 型定義（wire。42・40）
// ---------------------------------------------------------------------------

/// 計数の種類（「数を保存」「0 にする」= default、「入れ直す」= fresh）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum StockCountKind {
    Default,
    Fresh,
}

/// 要再確認の理由（40。ADR D3 の理由 1〜3、この順）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum RecountReason {
    UndeterminedSettlement,
    SameDateMovement,
    ConflictingCounts,
}

impl From<RecheckReason> for RecountReason {
    fn from(reason: RecheckReason) -> Self {
        match reason {
            RecheckReason::UndeterminedSettlement => Self::UndeterminedSettlement,
            RecheckReason::SameDateMovement => Self::SameDateMovement,
            RecheckReason::ConflictingCounts => Self::ConflictingCounts,
        }
    }
}

/// 回復先（40）。進行中の棚卸しの明細で数える / 棚卸しの外で今の数を入れる
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum CountRecoveryAction {
    ActiveCount,
    CountNow,
}

/// 完了を拒む理由（35 の表）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum CompletionBlockReason {
    Recheck,
    Uncounted,
    SettlementPending,
}

/// 回復型の code（40）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum StocktakeRecoveryCode {
    RecountRequired,
    CompletionBlocked,
}

/// 要再確認・完了の拒否・在庫連動の拒否の対象（40）。参照なしは null（架空の ID を使わない）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct CountRecoveryTarget {
    pub product_code: String,
    pub product_name: String,
    pub stock_unit: ProductStockUnit,
    /// 写しの在庫（target を作った TX の求め直しの後）
    pub stock: i64,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
    pub stocktake_item_id: Option<i64>,
    pub stocktake_id: Option<i64>,
    pub action: CountRecoveryAction,
    pub block_reason: Option<CompletionBlockReason>,
    /// 要再確認の対象のときだけ。重複なし、ADR D3 の順
    pub recount_reasons: Vec<RecountReason>,
}

/// 「取り込んでいない精算がある」の直し方（40）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct SettlementPendingHint {
    /// (ii) に当たった取り込まれていない精算回数の区間（昇順）
    pub missing: Vec<SettlementGap>,
    /// (i) に当たった明細の効く計数の日付の最大
    pub on_or_after: Option<String>,
    /// (iii) に当たった明細の効く計数の日付の最小
    pub before: Option<String>,
}

/// 拒否の回復型（40。`BizError::StocktakeGuard` が運ぶ）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct StocktakeRecovery {
    pub code: StocktakeRecoveryCode,
    pub targets: Vec<CountRecoveryTarget>,
    pub settlement_pending: Option<SettlementPendingHint>,
}

/// 計数の保存の要求（42）
#[derive(Debug, Clone, serde::Deserialize, specta::Type)]
pub struct SaveStockCountRequest {
    pub product_code: String,
    pub count_value: i64,
    pub fresh: bool,
    pub business_date: String,
    pub stocktake_item_id: Option<i64>,
}

/// 計数の保存の結果（42）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct StockCountSaveResult {
    pub id: i64,
    pub count_kind: StockCountKind,
    pub stock: i64,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
    pub recount_reasons: Vec<RecountReason>,
    pub later_count_exists: bool,
}

/// 後入力の確認と計数の欄の商品の文脈（42・35）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct CountPlacementPreview {
    pub later_count_exists: bool,
    pub product_name: String,
    pub stock_unit: ProductStockUnit,
    pub stock: i64,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
    pub recount_reasons: Vec<RecountReason>,
    pub count_value: Option<i64>,
    pub count_kind: Option<StockCountKind>,
    pub count_business_date: Option<String>,
}

/// 棚卸し開始結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StartStocktakeResult {
    pub stocktake_id: i64,
    pub item_count: usize,
}

/// 棚卸し完了リクエスト（`today` は CMD が渡す PC の今日。BIZ は時計で日付を判定しない）
#[derive(Debug)]
pub struct CompleteStocktakeRequest {
    pub stocktake_id: i64,
    pub today: String,
}

/// 棚卸し完了結果（在庫を動かさないので補正の一覧は持たない）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StocktakeResult {
    pub total_cost: i64,
    pub total_items: usize,
    /// D-2統合: 完了後の整合性チェック結果。失敗時はNone
    pub integrity_result: Option<crate::biz::integrity_service::IntegrityResult>,
}

/// 棚卸し明細（一覧と検索。35「読取り」）
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct StocktakeItemDetail {
    pub id: i64,
    pub stocktake_id: i64,
    pub product_code: String,
    pub name: String,
    pub department_name: String,
    pub system_stock: i64,
    pub actual_count: Option<i64>,
    pub counted_at: Option<String>,
    /// 写しの在庫
    pub current_stock: i64,
    pub is_discontinued: bool,
    pub stock_unit: ProductStockUnit,
    /// 効く計数（衝突のときは写しの土台の計数）
    pub count_value: Option<i64>,
    pub count_kind: Option<StockCountKind>,
    pub count_business_date: Option<String>,
    pub stock_range_low: Option<i64>,
    pub stock_range_high: Option<i64>,
    pub recount_reasons: Vec<RecountReason>,
    /// 効く計数の値 − その計数に入っている記録だけから求めた帳簿。計数が無ければ null
    pub difference: Option<i64>,
}

/// 棚卸し進捗（BIZ版）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StocktakeProgressBiz {
    pub stocktake_id: i64,
    pub status: String,
    pub total_items: usize,
    pub counted_items: usize,
    pub uncounted_items: usize,
}

/// 棚卸し記録詳細の状態（有限 wire 値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum StocktakeStatus {
    InProgress,
    Completed,
}

/// 棚卸し記録詳細の明細（65。完了した棚卸しは完了の時点のカーソルの効く計数）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct StocktakeRecordDetailItem {
    pub product_code: String,
    pub product_name: String,
    pub department_name: String,
    pub stock_unit: ProductStockUnit,
    pub count_value: Option<i64>,
    pub count_kind: Option<StockCountKind>,
    pub count_business_date: Option<String>,
    pub difference: Option<i64>,
    pub valuation_quantity: Option<i64>,
    pub valuation_cost_price_centi: Option<i64>,
    /// 差異が負で評価原価があるときの行のロス原価（1/100 円の十進の文字列。SPEC-UNIT-D6）
    pub loss_cost_centi: Option<String>,
}

/// 完了の後に入った記録による差の明細（参考。35「完了」）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct AfterCompletionDiffItem {
    pub product_code: String,
    pub product_name: String,
    pub stock_unit: ProductStockUnit,
    pub valuation_quantity: i64,
    pub stock_at_completion: i64,
    pub stock_at_completion_low: Option<i64>,
    pub stock_at_completion_high: Option<i64>,
    /// 円の十進の文字列（範囲の決まらない読取りの金額）
    pub valuation_diff_yen: String,
    pub valuation_diff_low_yen: Option<String>,
    pub valuation_diff_high_yen: Option<String>,
}

/// 完了の後の差（確定の差と要再確認の差を分け、確定の差だけを合計する）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
pub struct AfterCompletionDiff {
    pub confirmed: Vec<AfterCompletionDiffItem>,
    pub recheck: Vec<AfterCompletionDiffItem>,
    pub confirmed_total_yen: String,
}

/// 棚卸し記録詳細の wire DTO。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct StocktakeRecordDetail {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: StocktakeStatus,
    pub total_cost: Option<i64>,
    pub item_count: i64,
    pub items: Vec<StocktakeRecordDetailItem>,
    pub after_completion: Option<AfterCompletionDiff>,
}

// ---------------------------------------------------------------------------
// 共通
// ---------------------------------------------------------------------------

const COMPLETION_BLOCKED_MESSAGE: &str =
    "棚卸しを完了できません。数え直し・未計数・取り込んでいない精算のある商品があります";

fn count_kind(count: &StockCountRow) -> StockCountKind {
    if count.fresh {
        StockCountKind::Fresh
    } else {
        StockCountKind::Default
    }
}

fn reasons(derived: &DerivedStock) -> Vec<RecountReason> {
    derived
        .recheck_reasons
        .iter()
        .map(|reason| RecountReason::from(*reason))
        .collect()
}

fn range_ends(range: Option<(i64, i64)>) -> (Option<i64>, Option<i64>) {
    (range.map(|r| r.0), range.map(|r| r.1))
}

fn product_not_found(product_code: &str) -> BizError {
    BizError::NotFound(format!("商品が見つかりません: {product_code}"))
}

/// 日付の検査（形と暦 → 今日との比べ。Z2。BIZ は時計を読まず、CMD が渡す今日と比べる）
fn validate_count_date(business_date: &str, fresh: bool, today: &str) -> Result<(), BizError> {
    validate_business_date(business_date)?;
    if fresh && business_date != today {
        return Err(BizError::ValidationFailed(
            "入れ直す数の日付は今日だけです".to_string(),
        ));
    }
    if business_date > today {
        return Err(BizError::ValidationFailed(
            "未来の日付には保存できません".to_string(),
        ));
    }
    Ok(())
}

/// 計数の値の範囲（Z1: 0 以上、在庫の上限 999999999 以下）
fn validate_count_value(count_value: i64) -> Result<(), BizError> {
    if count_value < 0 {
        return Err(BizError::ValidationFailed(
            "数は0以上で入力してください".to_string(),
        ));
    }
    if count_value > MAX_STOCK_QUANTITY {
        return Err(BizError::ValidationFailed("数が大きすぎます".to_string()));
    }
    Ok(())
}

/// 要再確認の対象・拒否の対象を作る（40。値は写し。理由は写しが要再確認のときだけ）
fn recovery_target(
    conn: &DbConnection,
    copy: &StockCopyRow,
    block_reason: Option<CompletionBlockReason>,
) -> Result<CountRecoveryTarget, BizError> {
    let item = stocktake_repo::find_active_item_for_product(conn, &copy.product_code)?;
    let recount_reasons = if copy.stock_recheck {
        reasons(&stock_ledger::derive_stock(conn, &copy.product_code)?)
    } else {
        Vec::new()
    };
    Ok(CountRecoveryTarget {
        product_code: copy.product_code.clone(),
        product_name: copy.name.clone(),
        stock_unit: copy.stock_unit,
        stock: copy.stock_quantity,
        stock_range_low: copy.stock_range_low,
        stock_range_high: copy.stock_range_high,
        stocktake_item_id: item.map(|(item_id, _)| item_id),
        stocktake_id: item.map(|(_, stocktake_id)| stocktake_id),
        action: if item.is_some() {
            CountRecoveryAction::ActiveCount
        } else {
            CountRecoveryAction::CountNow
        },
        block_reason,
        recount_reasons,
    })
}

/// 商品コードの一覧のうち写しが要再確認の商品の対象（取込み・取消の結果、要再確認の一覧。40）
pub(crate) fn recheck_targets(
    conn: &DbConnection,
    product_codes: &[String],
) -> Result<Vec<CountRecoveryTarget>, BizError> {
    let mut targets = Vec::new();
    for code in product_codes {
        let copy =
            stocktake_repo::find_stock_copy(conn, code)?.ok_or_else(|| product_not_found(code))?;
        if copy.stock_recheck {
            targets.push(recovery_target(conn, &copy, None)?);
        }
    }
    Ok(targets)
}

/// 在庫連動を始める保存の拒否（30・40、ADR D8）。回復先は進行中の棚卸しに明細があれば `active_count`、
/// なければ `count_now`
pub(crate) fn recount_required(
    conn: &DbConnection,
    product_code: &str,
    message: &str,
) -> Result<BizError, BizError> {
    let copy = stocktake_repo::find_stock_copy(conn, product_code)?
        .ok_or_else(|| product_not_found(product_code))?;
    Ok(BizError::StocktakeGuard {
        message: message.to_string(),
        recovery: Box::new(StocktakeRecovery {
            code: StocktakeRecoveryCode::RecountRequired,
            targets: vec![recovery_target(conn, &copy, None)?],
            settlement_pending: None,
        }),
    })
}

/// 明細の表示用の写し（`actual_count`・`counted_at`）を効く計数に合わせる（35）
fn sync_item_copy(conn: &DbConnection, item_id: i64, product_code: &str) -> Result<(), BizError> {
    let ledger = read_ledger(conn, product_code)?;
    if let Some((rep, _)) = representative_count(&ledger)? {
        stocktake_repo::update_stocktake_item_count(
            conn,
            item_id,
            rep.count_value,
            &rep.counted_at,
        )?;
    }
    Ok(())
}

fn save_result(
    conn: &DbConnection,
    id: i64,
    product_code: &str,
    derived: &DerivedStock,
) -> Result<StockCountSaveResult, BizError> {
    let counts = inventory_repo::list_stock_counts_for_product(conn, product_code)?;
    let saved = counts
        .iter()
        .find(|count| count.id == id)
        .ok_or(BizError::DatabaseError(DbError::NotFound))?;
    let (low, high) = range_ends(derived.range);
    Ok(StockCountSaveResult {
        id,
        count_kind: count_kind(saved),
        stock: derived.stock,
        stock_range_low: low,
        stock_range_high: high,
        recount_reasons: reasons(derived),
        later_count_exists: counts
            .iter()
            .any(|other| other.id != id && is_after(other, saved)),
    })
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// 進行中の棚卸しを取得する（読み取り専用、TX不要）
///
/// UI-10-D1 / CMD-10 get_active_stocktake 用。IO層の薄いラッパー。
pub fn get_active_stocktake(
    conn: &DbConnection,
) -> Result<Option<stocktake_repo::Stocktake>, BizError> {
    Ok(stocktake_repo::find_active_stocktake(conn)?)
}

/// 明細の行に効く計数・差異・範囲・理由を足す（35「読取り」）
fn item_detail(
    conn: &DbConnection,
    row: StocktakeItemRow,
) -> Result<StocktakeItemDetail, BizError> {
    let ledger = read_ledger(conn, &row.product_code)?;
    let derived = stock_ledger::derive_ledger(&ledger)?;
    let rep = representative_count(&ledger)?;
    Ok(StocktakeItemDetail {
        id: row.id,
        stocktake_id: row.stocktake_id,
        product_code: row.product_code,
        name: row.name,
        department_name: row.department_name,
        system_stock: row.system_stock,
        actual_count: row.actual_count,
        counted_at: row.counted_at,
        current_stock: row.current_stock,
        is_discontinued: row.is_discontinued,
        stock_unit: row.stock_unit,
        count_value: rep.as_ref().map(|(count, _)| count.count_value),
        count_kind: rep.as_ref().map(|(count, _)| count_kind(count)),
        count_business_date: rep.as_ref().map(|(count, _)| count.business_date.clone()),
        stock_range_low: row.stock_range_low,
        stock_range_high: row.stock_range_high,
        recount_reasons: reasons(&derived),
        difference: rep.map(|(_, difference)| difference),
    })
}

/// 棚卸しアイテム一覧を取得する（読み取り専用、TX不要）
///
/// CMD-10 get_stocktake_items 用。BIZ-06-VAL-D1のページ下限を検証してIO層を呼ぶ。
/// items + progress を返す。
pub fn get_stocktake_items(
    conn: &DbConnection,
    stocktake_id: i64,
    department_id: Option<i64>,
    counted_only: Option<bool>,
    page: u32,
    per_page: u32,
) -> Result<
    (
        crate::db::PaginatedResult<StocktakeItemDetail>,
        stocktake_repo::StocktakeProgress,
    ),
    BizError,
> {
    if page < 1 {
        return Err(BizError::ValidationFailedAt {
            message: "ページ番号は1以上で指定してください".to_string(),
            field: "page".to_string(),
        });
    }
    if per_page < 1 {
        return Err(BizError::ValidationFailedAt {
            message: "1ページあたりの件数は1以上で指定してください".to_string(),
            field: "per_page".to_string(),
        });
    }

    let paginated = stocktake_repo::list_stocktake_items(
        conn,
        stocktake_id,
        department_id,
        counted_only,
        page,
        per_page,
    )?;
    let progress = stocktake_repo::get_stocktake_progress(conn, stocktake_id)?;
    let items = paginated
        .items
        .into_iter()
        .map(|row| item_detail(conn, row))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        crate::db::PaginatedResult {
            items,
            total_count: paginated.total_count,
            page: paginated.page,
            per_page: paginated.per_page,
        },
        progress,
    ))
}

/// 商品コードまたはJANコードで棚卸し明細を取得する（読み取り専用、TX不要）
///
/// UI-10-D2 / CMD-10 find_stocktake_item 用。
pub fn find_stocktake_item(
    conn: &DbConnection,
    stocktake_id: i64,
    code: &str,
) -> Result<Option<StocktakeItemDetail>, BizError> {
    stocktake_repo::find_stocktake_item_by_code(conn, stocktake_id, code)?
        .map(|row| item_detail(conn, row))
        .transpose()
}

/// 最後に完了した棚卸しを取得する（読み取り専用、TX不要）
///
/// UI-10-D5 / CMD-10 get_last_completed_stocktake 用。IO層の薄いラッパー。
pub fn get_last_completed_stocktake(
    conn: &DbConnection,
) -> Result<Option<stocktake_repo::LastStocktakeSummary>, BizError> {
    Ok(stocktake_repo::find_last_completed_stocktake(conn)?)
}

/// 精算が完了の時点の入力に入るか（精算回数 ≤ 完了の時点の精算の n。`after` が完了日なら、精算日 < 完了日の
/// 精算も入る = 完了の後の比較の範囲。35「完了」、ADR D6）
fn settlement_kept(
    settlement: &SettlementRow,
    settlement_n: Option<i64>,
    after: Option<&str>,
) -> bool {
    settlement_n.is_some_and(|no| settlement.settlement_no <= no)
        || after.is_some_and(|date| settlement.settlement_date.as_str() < date)
}

/// 完了した棚卸しのうち、指定の精算が完了の後の比較の範囲に入るもの（完了の日時の昇順）。
/// 範囲による候補で、記録の詳細のどの数が実際に変わるかは見ない（BIZ-03 の取消の前の読取り。32 §15.6b）
pub(crate) fn completed_stocktakes_comparing_settlement(
    conn: &DbConnection,
    settlement: &SettlementRow,
) -> Result<Vec<AffectedStocktake>, BizError> {
    let settlements = sales_repo::list_active_settlements(conn)?;
    Ok(stocktake_repo::list_completed_stocktake_cutoffs(conn)?
        .into_iter()
        .filter(|(_, completed_at, cutoff)| {
            settlement_kept(
                settlement,
                settlement_no_within(&settlements, cutoff.import_id),
                Some(completed_at.get(..10).unwrap_or(completed_at)),
            )
        })
        .map(|(stocktake_id, completed_at, _)| AffectedStocktake {
            stocktake_id,
            completed_at,
        })
        .collect())
}

/// 完了の時点の入力に絞る（計数・movement・精算をカーソル以下に。`after` があれば、完了の後に入れた
/// 完了の前の入出庫・初期在庫・精算の記録を足す。計数はカーソル以下だけ。35「完了」）。
/// 精算は完了の時点の精算の n（取込みのカーソル以下の今 active な取込みの最大の精算回数。ADR D6）以下で決める
fn ledger_at_cutoff(ledger: &Ledger, cutoff: StocktakeCutoff, after: Option<&str>) -> Ledger {
    let settlement_n = settlement_no_within(&ledger.settlements, cutoff.import_id);
    let settlements: Vec<SettlementRow> = ledger
        .settlements
        .iter()
        .filter(|settlement| settlement_kept(settlement, settlement_n, after))
        .cloned()
        .collect();
    let movements = ledger
        .movements
        .iter()
        .filter(|movement| match movement.settlement_no {
            Some(no) => settlements.iter().any(|s| s.settlement_no == no),
            None => {
                cutoff.movement_id.is_some_and(|id| movement.id <= id)
                    || after.is_some_and(|date| movement.business_date.as_str() <= date)
            }
        })
        .cloned()
        .collect();
    Ledger {
        counts: ledger
            .counts
            .iter()
            .filter(|count| cutoff.count_id.is_some_and(|id| count.id <= id))
            .cloned()
            .collect(),
        movements,
        linked_sale_imports: ledger
            .linked_sale_imports
            .iter()
            .filter(|import_id| settlements.iter().any(|s| s.import_id == **import_id))
            .copied()
            .collect(),
        settlements,
    }
}

/// 評価額（円。§20.5a の関数を 1 行に当て、丸めた後の総額）
fn valuation_yen(
    cost_price_centi: i64,
    quantity: i64,
    stock_unit: ProductStockUnit,
    product_code: &str,
) -> Result<i64, BizError> {
    let line = cost_line_centi(
        i128::from(cost_price_centi),
        quantity,
        price_basis_quantity(stock_unit),
        product_code,
    )?;
    cost_total_yen(&[line])
}

/// 完了の後の差（35「完了」、ADR D6）。読取りで求め、保存しない
fn after_completion_diff(
    conn: &DbConnection,
    items: &[stocktake_repo::StocktakeRecordItemRow],
    cutoff: StocktakeCutoff,
    completion_date: &str,
) -> Result<AfterCompletionDiff, BizError> {
    let (mut confirmed, mut recheck) = (Vec::new(), Vec::new());
    let mut confirmed_total: i128 = 0;
    for item in items {
        let (Some(valuation_quantity), Some(cost)) =
            (item.valuation_quantity, item.valuation_cost_price_centi)
        else {
            continue;
        };
        let ledger = ledger_at_cutoff(
            &read_ledger(conn, &item.product_code)?,
            cutoff,
            Some(completion_date),
        );
        let derived = stock_ledger::derive_ledger(&ledger)?;
        let yen = |quantity: i64| {
            valuation_yen(cost, quantity.max(0), item.stock_unit, &item.product_code)
        };
        let base = i128::from(yen(valuation_quantity)?);
        let diff =
            |quantity: i64| -> Result<i128, BizError> { Ok(i128::from(yen(quantity)?) - base) };
        let (low, high) = range_ends(derived.range);
        let quantity_differs = [Some(derived.stock), low, high]
            .into_iter()
            .flatten()
            .any(|quantity| quantity.max(0) != valuation_quantity);
        if !quantity_differs {
            continue;
        }
        let stock_diff = diff(derived.stock)?;
        let entry = AfterCompletionDiffItem {
            product_code: item.product_code.clone(),
            product_name: item.product_name.clone(),
            stock_unit: item.stock_unit,
            valuation_quantity,
            stock_at_completion: derived.stock,
            stock_at_completion_low: low,
            stock_at_completion_high: high,
            valuation_diff_yen: stock_diff.to_string(),
            valuation_diff_low_yen: low.map(diff).transpose()?.map(|v| v.to_string()),
            valuation_diff_high_yen: high.map(diff).transpose()?.map(|v| v.to_string()),
        };
        if derived.range.is_some() {
            recheck.push(entry);
        } else {
            confirmed_total += stock_diff;
            confirmed.push(entry);
        }
    }
    Ok(AfterCompletionDiff {
        confirmed,
        recheck,
        confirmed_total_yen: confirmed_total.to_string(),
    })
}

/// 棚卸し記録詳細を wire DTO として返す（65。明細と header から読み、補正の movement の節を持たない）。
///
/// 35-biz-stocktake-service.md「完了」/ REQ-206 / REQ-207
pub fn get_stocktake_record(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<StocktakeRecordDetail, BizError> {
    let core = match stocktake_repo::get_stocktake_record_detail(conn, stocktake_id) {
        Ok(core) => core,
        Err(DbError::NotFound) => {
            return Err(BizError::NotFound(format!(
                "棚卸し記録が見つかりません: {stocktake_id}"
            )))
        }
        Err(error) => return Err(BizError::DatabaseError(error)),
    };

    let status = match core.header.status.as_str() {
        "in_progress" => StocktakeStatus::InProgress,
        "completed" => StocktakeStatus::Completed,
        unexpected => {
            return Err(BizError::DatabaseError(DbError::QueryFailed(format!(
                "unexpected stocktake status: {unexpected}"
            ))))
        }
    };
    let completed = status == StocktakeStatus::Completed;
    let mut items = Vec::with_capacity(core.items.len());
    for row in &core.items {
        let mut ledger = read_ledger(conn, &row.product_code)?;
        if completed {
            ledger = ledger_at_cutoff(&ledger, core.cutoff, None);
        }
        let rep = representative_count(&ledger)?;
        let difference = rep.as_ref().map(|(_, difference)| *difference);
        let loss_cost_centi = match (difference, row.valuation_cost_price_centi) {
            (Some(difference), Some(cost)) if difference < 0 => Some(
                cost_line_centi(
                    i128::from(cost),
                    -difference,
                    price_basis_quantity(row.stock_unit),
                    &row.product_code,
                )?
                .to_string(),
            ),
            _ => None,
        };
        items.push(StocktakeRecordDetailItem {
            product_code: row.product_code.clone(),
            product_name: row.product_name.clone(),
            department_name: row.department_name.clone(),
            stock_unit: row.stock_unit,
            count_value: rep.as_ref().map(|(count, _)| count.count_value),
            count_kind: rep.as_ref().map(|(count, _)| count_kind(count)),
            count_business_date: rep.as_ref().map(|(count, _)| count.business_date.clone()),
            difference,
            valuation_quantity: row.valuation_quantity,
            valuation_cost_price_centi: row.valuation_cost_price_centi,
            loss_cost_centi,
        });
    }
    let after_completion = match (&core.header.completed_at, completed) {
        (Some(completed_at), true) => Some(after_completion_diff(
            conn,
            &core.items,
            core.cutoff,
            completed_at.get(..10).unwrap_or(completed_at),
        )?),
        _ => None,
    };

    Ok(StocktakeRecordDetail {
        id: core.header.id,
        started_at: core.header.started_at,
        completed_at: core.header.completed_at,
        status,
        total_cost: core.header.total_cost,
        item_count: core.item_count,
        items,
        after_completion,
    })
}

/// 棚卸し進捗を取得する（読み取り専用、TX不要。計数済み = 明細を参照する計数がある）
///
/// 35-biz-stocktake-service.md §20.6
pub fn get_stocktake_progress(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<StocktakeProgressBiz, BizError> {
    // 1. 棚卸し存在チェック
    let stocktake = stocktake_repo::find_stocktake_by_id(conn, stocktake_id)?.ok_or_else(|| {
        BizError::NotFound(format!("棚卸しが見つかりません: ID {}", stocktake_id))
    })?;

    // 2. DB進捗取得
    let db_progress = stocktake_repo::get_stocktake_progress(conn, stocktake_id)?;

    // 3. BIZ型に変換
    Ok(StocktakeProgressBiz {
        stocktake_id,
        status: stocktake.status,
        total_items: db_progress.total_items as usize,
        counted_items: db_progress.counted_items as usize,
        uncounted_items: db_progress.uncounted_items as usize,
    })
}

/// 計数を保存する（35「計数の保存」、ADR D5）。1 商品 1 TX で INSERT し写しを求め直す。
///
/// 検査の順: 形と暦 → 今日との比べ → 値の範囲 → TX の中で明細の進行中と商品の一致 → INSERT →
/// `recompute_stock` → 明細の写し。`today` は CMD が渡す PC の今日
pub fn save_stock_count(
    conn: &mut DbConnection,
    req: &SaveStockCountRequest,
    today: &str,
) -> Result<StockCountSaveResult, BizError> {
    validate_count_date(&req.business_date, req.fresh, today)?;
    validate_count_value(req.count_value)?;

    let tx = conn.transaction().map_err(DbError::from)?;
    if stocktake_repo::find_stock_copy(&tx, &req.product_code)?.is_none() {
        return Err(product_not_found(&req.product_code));
    }
    if let Some(item_id) = req.stocktake_item_id {
        let matches = stocktake_repo::find_stocktake_item_with_parent_status(&tx, item_id)?
            .is_some_and(|(item, status)| {
                status == "in_progress" && item.product_code == req.product_code
            });
        if !matches {
            return Err(BizError::ValidationFailed(
                "この明細は別の商品です".to_string(),
            ));
        }
    }
    // fresh のカーソルは INSERT と同じ TX で読む（ADR D3・D9。精算回数は鍵の n、判定の n は取込みのカーソルから）
    let (settlement_no, seen_movement_id, seen_import_id) = if req.fresh {
        (
            sales_repo::list_active_settlements(&tx)?
                .last()
                .map(|settlement| settlement.settlement_no),
            Some(inventory_repo::max_movement_id(&tx)?),
            sales_repo::max_csv_import_id(&tx)?,
        )
    } else {
        (None, None, None)
    };
    let id = inventory_repo::insert_stock_count(
        &tx,
        &NewStockCount {
            product_code: req.product_code.clone(),
            count_value: req.count_value,
            fresh: req.fresh,
            business_date: req.business_date.clone(),
            settlement_no,
            seen_movement_id,
            seen_import_id,
            stocktake_item_id: req.stocktake_item_id,
        },
    )?;
    let derived = recompute_stock(&tx, &req.product_code)?;
    if let Some(item_id) = req.stocktake_item_id {
        sync_item_copy(&tx, item_id, &req.product_code)?;
    }
    let result = save_result(&tx, id, &req.product_code, &derived)?;
    tx.commit().map_err(DbError::from)?;
    Ok(result)
}

/// 後入力の確認と計数の欄の商品の文脈（読取りだけ。35・42）
pub fn preview_stock_count_placement(
    conn: &DbConnection,
    product_code: &str,
    business_date: &str,
    today: &str,
) -> Result<CountPlacementPreview, BizError> {
    validate_count_date(business_date, false, today)?;
    let copy = stocktake_repo::find_stock_copy(conn, product_code)?
        .ok_or_else(|| product_not_found(product_code))?;
    let ledger = read_ledger(conn, product_code)?;
    // これから保存する既定の計数（どの計数よりも後に保存される）
    let planned = StockCountRow {
        id: i64::MAX,
        count_value: 0,
        fresh: false,
        business_date: business_date.to_string(),
        seen_movement_id: None,
        seen_import_id: None,
        counted_at: String::new(),
    };
    let later_count_exists = ledger.counts.iter().any(|count| is_after(count, &planned));
    let derived = stock_ledger::derive_ledger(&ledger)?;
    let rep = representative_count(&ledger)?;
    Ok(CountPlacementPreview {
        later_count_exists,
        product_name: copy.name,
        stock_unit: copy.stock_unit,
        stock: copy.stock_quantity,
        stock_range_low: copy.stock_range_low,
        stock_range_high: copy.stock_range_high,
        recount_reasons: reasons(&derived),
        count_value: rep.as_ref().map(|(count, _)| count.count_value),
        count_kind: rep.as_ref().map(|(count, _)| count_kind(count)),
        count_business_date: rep.map(|(count, _)| count.business_date),
    })
}

/// 未計数の明細を値 0 の既定の計数でまとめて片付ける（35「完了」の未計数を片付ける操作）。
/// 1 TX、商品ごとに求め直す。各計数は進行中の棚卸しの当該明細を参照する
pub fn zero_uncounted_items(
    conn: &mut DbConnection,
    stocktake_id: i64,
    product_codes: &[String],
    business_date: &str,
    today: &str,
) -> Result<Vec<StockCountSaveResult>, BizError> {
    validate_count_date(business_date, false, today)?;
    let tx = conn.transaction().map_err(DbError::from)?;
    // 書く前に全ての商品の明細を確かめる（1 つでも無ければ何も書かない）
    let mut items = Vec::with_capacity(product_codes.len());
    for code in product_codes {
        match stocktake_repo::find_active_item_for_product(&tx, code)? {
            Some((item_id, active_id)) if active_id == stocktake_id => items.push((code, item_id)),
            _ => {
                return Err(BizError::ValidationFailed(format!(
                    "この商品は棚卸しの対象にありません: {code}"
                )))
            }
        }
    }
    let mut results = Vec::with_capacity(items.len());
    for (code, item_id) in items {
        let id = inventory_repo::insert_stock_count(
            &tx,
            &NewStockCount {
                product_code: code.clone(),
                count_value: 0,
                fresh: false,
                business_date: business_date.to_string(),
                settlement_no: None,
                seen_movement_id: None,
                seen_import_id: None,
                stocktake_item_id: Some(item_id),
            },
        )?;
        let derived = recompute_stock(&tx, code)?;
        sync_item_copy(&tx, item_id, code)?;
        results.push(save_result(&tx, id, code, &derived)?);
    }
    tx.commit().map_err(DbError::from)?;
    Ok(results)
}

/// 要再確認の一覧（写しが要再確認の商品。棚卸しが無いときも読める。35「読取り」）
pub fn list_recheck_products(conn: &DbConnection) -> Result<Vec<CountRecoveryTarget>, BizError> {
    recheck_targets(conn, &stocktake_repo::list_recheck_product_codes(conn)?)
}

/// 棚卸しを開始する（TX + operation_log TX外。自動の 0 の入力は持たない。35「開始と明細」）
///
/// 35-biz-stocktake-service.md §20.3
pub fn start_stocktake(conn: &mut DbConnection) -> Result<StartStocktakeResult, BizError> {
    // 1. TX外: 進行中チェック
    if let Some(existing) = stocktake_repo::find_active_stocktake(conn)? {
        return Err(BizError::StocktakeInProgress(format!(
            "進行中の棚卸しがあります（ID: {}、開始日: {}）。完了してから新しい棚卸しを開始してください",
            existing.id, existing.started_at
        )));
    }

    // 2. TX外: 対象商品の取得（全商品、フィルタなし）
    let products = stocktake_repo::find_stocktake_eligible_products(conn)?;
    if products.is_empty() {
        return Err(BizError::ValidationFailed(
            "棚卸し対象の商品がありません".to_string(),
        ));
    }

    // 3. TX: header と明細（system_stock は作成時の写し、actual_count は NULL）
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let tx = conn.transaction().map_err(DbError::from)?;
    let stocktake_id = stocktake_repo::insert_stocktake(&tx, &now)?;
    for product in &products {
        stocktake_repo::insert_stocktake_item(
            &tx,
            &NewStocktakeItem {
                stocktake_id,
                product_code: product.product_code.clone(),
                system_stock: product.stock_quantity,
                actual_count: None,
            },
        )?;
    }
    tx.commit().map_err(DbError::from)?;

    // 4. TX外: 操作ログ記録（失敗は警告だけ）
    let item_count = products.len();
    let detail = serde_json::json!({
        "stocktake_id": stocktake_id,
        "item_count": item_count,
    })
    .to_string();
    let log = NewOperationLog {
        operation_type: "stocktake_start".to_string(),
        summary: format!("棚卸しを開始しました（対象: {}件）", item_count),
        detail_json: Some(detail),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }

    Ok(StartStocktakeResult {
        stocktake_id,
        item_count,
    })
}

/// 効く計数ごとの `settlement_pending` の判定（35 の表の (i)〜(iii)）。当たった条件を hint に積む
fn settlement_pending(
    counts: &[&StockCountRow],
    settlements: &[SettlementRow],
    hint: &mut SettlementPendingHint,
) -> bool {
    let present: Vec<i64> = settlements.iter().map(|s| s.settlement_no).collect();
    let mut pending = false;
    for count in counts {
        let x = count.business_date.as_str();
        let n = usable_cursor(count, settlements);
        // (i) 精算日が X 以上の取り込み済みの精算が無い
        if !settlements.iter().any(|s| s.settlement_date.as_str() >= x) {
            pending = true;
            if hint.on_or_after.as_deref().is_none_or(|max| x > max) {
                hint.on_or_after = Some(x.to_string());
            }
        }
        // (ii) 取り込まれていない k（fresh で n があれば k > n）の隣の精算が X を挟む
        for gap in missing_ranges(&present) {
            let date_of = |no: i64| {
                settlements
                    .iter()
                    .find(|s| s.settlement_no == no)
                    .map(|s| s.settlement_date.as_str())
            };
            let (Some(before), Some(after)) = (date_of(gap.from - 1), date_of(gap.to + 1)) else {
                continue;
            };
            let from = n.map_or(gap.from, |n| gap.from.max(n + 1));
            if from <= gap.to && before <= x && x <= after {
                pending = true;
                let clipped = SettlementGap { from, to: gap.to };
                if !hint.missing.contains(&clipped) {
                    hint.missing.push(clipped);
                }
            }
        }
        // (iii) 既定の計数（と n が NULL の fresh）で、精算日が X より前の取り込み済みの精算が無い
        if n.is_none() && !settlements.iter().any(|s| s.settlement_date.as_str() < x) {
            pending = true;
            if hint.before.as_deref().is_none_or(|min| x < min) {
                hint.before = Some(x.to_string());
            }
        }
    }
    pending
}

/// 棚卸しを完了する（35「完了」、ADR D6）。理由を全部調べてから拒み（何も書かない）、
/// 通れば評価数量・評価原価・総額・カーソルを固定する。在庫を動かさず、movement を作らない
pub fn complete_stocktake(
    conn: &mut DbConnection,
    req: &CompleteStocktakeRequest,
) -> Result<StocktakeResult, BizError> {
    let stocktake =
        stocktake_repo::find_stocktake_by_id(conn, req.stocktake_id)?.ok_or_else(|| {
            BizError::NotFound(format!("棚卸しが見つかりません: ID {}", req.stocktake_id))
        })?;
    if stocktake.status != "in_progress" {
        return Err(BizError::StocktakeNotInProgress(
            "この棚卸しは既に完了しています".to_string(),
        ));
    }

    let tx = conn.transaction().map_err(DbError::from)?;
    let items = stocktake_repo::list_items_for_completion(&tx, req.stocktake_id)?;
    let settlements = sales_repo::list_active_settlements(&tx)?;

    // 1. 拒否の理由を全部集める
    let mut targets = Vec::new();
    let mut hint = SettlementPendingHint {
        missing: Vec::new(),
        on_or_after: None,
        before: None,
    };
    let mut differences = 0_usize;
    for item in &items {
        let ledger = read_ledger(&tx, &item.product_code)?;
        let (effective, _) = stock_ledger::effective_counts(&ledger.counts)?;
        let mut blocks = Vec::new();
        if item.stock_recheck {
            blocks.push(CompletionBlockReason::Recheck);
        }
        if !(item.counted || item.is_discontinued && item.stock_quantity == 0) {
            blocks.push(CompletionBlockReason::Uncounted);
        }
        if settlement_pending(&effective, &settlements, &mut hint) {
            blocks.push(CompletionBlockReason::SettlementPending);
        }
        if !blocks.is_empty() {
            let copy = stocktake_repo::find_stock_copy(&tx, &item.product_code)?
                .ok_or_else(|| product_not_found(&item.product_code))?;
            for block in blocks {
                targets.push(recovery_target(&tx, &copy, Some(block))?);
            }
        }
        if representative_count(&ledger)?.is_some_and(|(_, difference)| difference != 0) {
            differences += 1;
        }
    }
    if !targets.is_empty() {
        hint.missing.sort_by_key(|gap| (gap.from, gap.to));
        let has_pending = targets
            .iter()
            .any(|t| t.block_reason == Some(CompletionBlockReason::SettlementPending));
        return Err(BizError::StocktakeGuard {
            message: COMPLETION_BLOCKED_MESSAGE.to_string(),
            recovery: Box::new(StocktakeRecovery {
                code: StocktakeRecoveryCode::CompletionBlocked,
                targets,
                settlement_pending: has_pending.then_some(hint),
            }),
        });
    }

    // 2. 評価数量・評価原価・総額（§20.5a の後に安全な整数の範囲）
    let mut line_centis: Vec<i128> = Vec::with_capacity(items.len());
    for item in &items {
        let valuation_quantity = item.stock_quantity.max(0);
        stocktake_repo::update_stocktake_item_valuation(
            &tx,
            item.id,
            item.cost_price_centi,
            valuation_quantity,
        )?;
        line_centis.push(cost_line_centi(
            i128::from(item.cost_price_centi),
            valuation_quantity,
            price_basis_quantity(item.stock_unit),
            &item.product_code,
        )?);
    }
    let total_cost = check_safe_integer(i128::from(cost_total_yen(&line_centis)?), "合計金額")?;

    // 3. カーソルと header（完了日は CMD が渡す今日）
    let cutoff = StocktakeCutoff {
        import_id: sales_repo::max_csv_import_id(&tx)?,
        count_id: inventory_repo::max_stock_count_id(&tx)?,
        movement_id: Some(inventory_repo::max_movement_id(&tx)?),
    };
    let completed_at = format!("{}T{}", req.today, chrono::Local::now().format("%H:%M:%S"));
    stocktake_repo::complete_stocktake(&tx, req.stocktake_id, total_cost, &completed_at, cutoff)?;
    tx.commit().map_err(DbError::from)?;

    // 4. TX外: 操作ログ（差異の件数は差異が null でなく 0 でない明細の数）
    let total_items = items.len();
    let detail = serde_json::json!({
        "stocktake_id": req.stocktake_id,
        "total_cost": total_cost,
        "total_items": total_items,
        "difference_count": differences,
    })
    .to_string();
    let log = NewOperationLog {
        operation_type: "stocktake_complete".to_string(),
        summary: format!(
            "棚卸しを確定しました（差異: {}件、仕入原価総額: ¥{}）",
            differences, total_cost
        ),
        detail_json: Some(detail),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }

    // 5. TX外: 整合性チェック自動実行（D-2統合）
    let integrity_result = match crate::biz::integrity_service::run_integrity_check(conn) {
        Ok(result) => Some(result),
        Err(e) => {
            tracing::warn!(error = %e, "整合性チェックに失敗");
            None
        }
    };

    Ok(StocktakeResult {
        total_cost,
        total_items,
        integrity_result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biz::stock_ledger::settlement_support::{roll_back, settlement, z004_movement};
    use crate::db::init_database;
    use crate::db::product_repo::{self, NewProduct};

    /// 計数の日付・完了の今日（D）と前後の日
    const D: &str = "2026-10-15";
    const D1: &str = "2026-10-14";
    const D2: &str = "2026-10-13";
    const D_NEXT: &str = "2026-10-16";

    fn save(
        conn: &mut DbConnection,
        code: &str,
        value: i64,
        date: &str,
        fresh: bool,
        item: Option<i64>,
    ) -> Result<StockCountSaveResult, BizError> {
        save_stock_count(
            conn,
            &SaveStockCountRequest {
                product_code: code.to_string(),
                count_value: value,
                fresh,
                business_date: date.to_string(),
                stocktake_item_id: item,
            },
            D,
        )
    }

    fn item_id(conn: &DbConnection, stocktake_id: i64, code: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM stocktake_items WHERE stocktake_id = ?1 AND product_code = ?2",
            rusqlite::params![stocktake_id, code],
            |row| row.get(0),
        )
        .unwrap()
    }

    /// 業務日 `date` の入庫の movement を入れ（写しは求め直さない）、id を返す
    fn receive(conn: &DbConnection, code: &str, quantity: i64, date: &str) -> i64 {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 reference_type, reference_id, created_at, business_date, phase)
             VALUES (?1, 'receiving', ?2, 0, 'receiving_record', 1, ?3 || 'T12:00:00', ?3, 0)",
            rusqlite::params![code, quantity, date],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn recompute(conn: &mut DbConnection, code: &str) {
        let tx = conn.transaction().unwrap();
        recompute_stock(&tx, code).unwrap();
        tx.commit().unwrap();
    }

    fn count_rows(conn: &DbConnection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM stock_counts", [], |row| row.get(0))
            .unwrap()
    }

    fn copy(conn: &DbConnection, code: &str) -> (i64, bool, Option<i64>, Option<i64>) {
        conn.query_row(
            "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
             FROM products WHERE product_code = ?1",
            [code],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap()
    }

    fn guard(result: Result<StocktakeResult, BizError>) -> StocktakeRecovery {
        match result {
            Err(BizError::StocktakeGuard { message, recovery }) => {
                assert_eq!(message, COMPLETION_BLOCKED_MESSAGE);
                assert_eq!(recovery.code, StocktakeRecoveryCode::CompletionBlocked);
                *recovery
            }
            other => panic!("StocktakeGuard を期待: {other:?}"),
        }
    }

    fn complete(conn: &mut DbConnection, stocktake_id: i64) -> Result<StocktakeResult, BizError> {
        complete_stocktake(
            conn,
            &CompleteStocktakeRequest {
                stocktake_id,
                today: D.to_string(),
            },
        )
    }

    fn header_status(conn: &DbConnection, stocktake_id: i64) -> String {
        conn.query_row(
            "SELECT status FROM stocktakes WHERE id = ?1",
            [stocktake_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    fn seed_product(conn: &DbConnection, product_code: &str, stock_quantity: i64) {
        let product = NewProduct {
            product_code: product_code.to_string(),
            jan_code: None,
            name: format!("商品{}", product_code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
        // 写しは記録から求め直すので、同じ値の初期在庫の movement も入れる（Matrix T3）
        if stock_quantity != 0 {
            crate::db::test_support::insert_initial_stock_movement(
                conn,
                product_code,
                stock_quantity,
                stock_quantity,
            );
        }
    }

    fn create_stocktake(conn: &DbConnection, status: &str) -> i64 {
        conn.execute(
            "INSERT INTO stocktakes (started_at, status) VALUES ('2026-10-01T09:00:00', ?1)",
            rusqlite::params![status],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn seed_stocktake_item(
        conn: &DbConnection,
        stocktake_id: i64,
        product_code: &str,
        system_stock: i64,
        actual_count: Option<i64>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO stocktake_items (stocktake_id, product_code, system_stock, actual_count)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![stocktake_id, product_code, system_stock, actual_count],
        )
        .unwrap();
        let item_id = conn.last_insert_rowid();
        // 計数済みは明細を参照する計数があること（35）。数のある明細には同じ値の計数を入れる
        if let Some(value) = actual_count {
            conn.execute(
                "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                     stocktake_item_id, counted_at)
                 VALUES (?1, ?2, 'default', '2026-10-01', ?3, '2026-10-01T10:00:00')",
                rusqlite::params![product_code, value, item_id],
            )
            .unwrap();
        }
        item_id
    }

    fn seed_product_with_jan(
        conn: &DbConnection,
        product_code: &str,
        jan_code: Option<&str>,
        stock_quantity: i64,
    ) {
        let product = NewProduct {
            product_code: product_code.to_string(),
            jan_code: jan_code.map(str::to_string),
            name: format!("商品{}", product_code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
        // 写しは記録から求め直すので、同じ値の初期在庫の movement も入れる（Matrix T3）
        if stock_quantity != 0 {
            crate::db::test_support::insert_initial_stock_movement(
                conn,
                product_code,
                stock_quantity,
                stock_quantity,
            );
        }
    }

    // ===== UI-10 T-R3: 新規 BIZ 薄ラッパー =====

    #[test]
    fn test_get_active_stocktake_req205_passes_through_repo_result() {
        // REQ-205: 棚卸し（BIZ get_active_stocktake は IO 結果を加工せず返す）
        let (_dir, conn) = setup_test_db();
        let st_id = create_stocktake(&conn, "in_progress");

        let result = get_active_stocktake(&conn).unwrap();

        let stocktake = result.expect("進行中棚卸しが返るべき");
        assert_eq!(stocktake.id, st_id);
        assert_eq!(stocktake.status, "in_progress");
    }

    #[test]
    fn test_get_active_stocktake_req205_converts_db_error_to_biz_error() {
        // REQ-205: 棚卸し（BIZ get_active_stocktake は DbError を BizError::DatabaseError に変換）
        let (_dir, conn) = setup_test_db();
        conn.execute("DROP TABLE stocktakes", []).unwrap();

        let result = get_active_stocktake(&conn);

        assert!(matches!(result, Err(BizError::DatabaseError(_))));
    }

    #[test]
    fn test_find_stocktake_item_req205_passes_through_repo_result() {
        // REQ-205: 棚卸し（BIZ find_stocktake_item は IO 結果を加工せず返す）
        let (_dir, conn) = setup_test_db();
        seed_product_with_jan(&conn, "BF-001", Some("4900000000101"), 12);
        let st_id = create_stocktake(&conn, "in_progress");
        let item_id = seed_stocktake_item(&conn, st_id, "BF-001", 12, Some(11));

        let result = find_stocktake_item(&conn, st_id, "4900000000101").unwrap();

        let item = result.expect("JAN一致の明細が返るべき");
        assert_eq!(item.id, item_id);
        assert_eq!(item.actual_count, Some(11));
    }

    #[test]
    fn test_find_stocktake_item_req205_converts_db_error_to_biz_error() {
        // REQ-205: 棚卸し（BIZ find_stocktake_item は DbError を BizError::DatabaseError に変換）
        let (_dir, conn) = setup_test_db();
        conn.execute("DROP TABLE products", []).unwrap();

        let result = find_stocktake_item(&conn, 1, "BROKEN");

        assert!(matches!(result, Err(BizError::DatabaseError(_))));
    }

    #[test]
    fn test_get_last_completed_stocktake_req205_passes_through_none() {
        // REQ-205: 棚卸し（BIZ get_last_completed_stocktake は完了済みなしを None で返す）
        let (_dir, conn) = setup_test_db();

        let result = get_last_completed_stocktake(&conn).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_get_last_completed_stocktake_req205_converts_db_error_to_biz_error() {
        // REQ-205: 棚卸し（BIZ get_last_completed_stocktake は DbError を BizError::DatabaseError に変換）
        let (_dir, conn) = setup_test_db();
        conn.execute("DROP TABLE stocktakes", []).unwrap();

        let result = get_last_completed_stocktake(&conn);

        assert!(matches!(result, Err(BizError::DatabaseError(_))));
    }

    // ===== get_stocktake_progress テスト =====

    #[test]
    fn test_get_progress_req205_normal() {
        // REQ-205: 棚卸し（進捗取得 — 正常）
        // BIZ-06 §20.6: 正常な進捗取得
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "PG-001", 10);
        seed_product(&conn, "PG-002", 5);
        seed_product(&conn, "PG-003", 3);
        let st_id = create_stocktake(&conn, "in_progress");
        seed_stocktake_item(&conn, st_id, "PG-001", 10, Some(10));
        seed_stocktake_item(&conn, st_id, "PG-002", 5, None);
        seed_stocktake_item(&conn, st_id, "PG-003", 3, Some(3));

        let progress = get_stocktake_progress(&conn, st_id).unwrap();
        assert_eq!(progress.stocktake_id, st_id);
        assert_eq!(progress.status, "in_progress");
        assert_eq!(progress.total_items, 3);
        assert_eq!(progress.counted_items, 2);
        assert_eq!(progress.uncounted_items, 1);
    }

    #[test]
    fn test_get_progress_req205_not_found() {
        // REQ-205: 棚卸し（進捗取得 — 存在しないID → NotFound）
        // BIZ-06 §20.6: 存在しないID → NotFound
        let (_dir, conn) = setup_test_db();
        let result = get_stocktake_progress(&conn, 9999);
        assert!(matches!(result, Err(BizError::NotFound(_))));
    }

    // ===== 計数の保存のテスト =====

    // テストヘルパー: カスタム商品seed
    fn seed_product_custom(
        conn: &DbConnection,
        product_code: &str,
        is_discontinued: bool,
        stock_quantity: i64,
        cost_price_centi: i64,
    ) {
        let product = NewProduct {
            product_code: product_code.to_string(),
            jan_code: None,
            name: format!("商品{}", product_code),
            department_id: 1,
            supplier_id: None,
            selling_price: 500,
            cost_price_centi,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity,
            stock_unit: "pcs".to_string(),
            is_discontinued,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(conn, &product).unwrap();
        // 写しは記録から求め直すので、同じ値の初期在庫の movement も入れる（Matrix T3）
        if stock_quantity != 0 {
            crate::db::test_support::insert_initial_stock_movement(
                conn,
                product_code,
                stock_quantity,
                stock_quantity,
            );
        }
    }

    // テストヘルパー: 在庫単位を指定する商品seed（評価額の基準数量 SPEC-STK-VAL-D1）
    fn seed_product_with_unit(
        conn: &DbConnection,
        product_code: &str,
        stock_unit: &str,
        stock_quantity: i64,
        cost_price_centi: i64,
    ) {
        seed_product_custom(conn, product_code, false, stock_quantity, cost_price_centi);
        conn.execute(
            "UPDATE products SET stock_unit = ?1 WHERE product_code = ?2",
            rusqlite::params![stock_unit, product_code],
        )
        .unwrap();
    }

    // ===== start_stocktake テスト =====

    #[test]
    fn test_start_stocktake_req205_normal() {
        // REQ-205: 棚卸し（棚卸し開始 — 正常開始 + item_count）
        // BIZ-06 §20.3: 正常開始 + item_count + system_stock値DB検証
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "SS-001", 10);
        seed_product(&conn, "SS-002", 5);

        let result = start_stocktake(&mut conn).unwrap();
        assert!(result.stocktake_id > 0);
        assert_eq!(result.item_count, 2);

        // DBクエリで stocktake_items を検証
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM stocktake_items WHERE stocktake_id = ?1",
                rusqlite::params![result.stocktake_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_start_stocktake_req205_already_in_progress() {
        // REQ-205: 棚卸し（棚卸し開始 — 進行中あり → StocktakeInProgress）
        // BIZ-06 §20.3: 進行中あり → StocktakeInProgress
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "SAI-001", 10);

        start_stocktake(&mut conn).unwrap(); // 1回目
        let result = start_stocktake(&mut conn); // 2回目
        assert!(matches!(result, Err(BizError::StocktakeInProgress(_))));
    }

    #[test]
    fn test_start_stocktake_req205_no_eligible() {
        // REQ-205: 棚卸し（棚卸し開始 — 商品0件 → ValidationFailed）
        // BIZ-06 §20.3: 商品0件 → ValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let result = start_stocktake(&mut conn);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    fn test_start_stocktake_req205_operation_log() {
        // REQ-205: 棚卸し（棚卸し開始 — operation_log記録）
        // BIZ-06 §20.3: operation_log記録確認
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "SOL-001", 10);

        start_stocktake(&mut conn).unwrap();

        let (op_type, summary): (String, String) = conn
            .query_row(
                "SELECT operation_type, summary FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(op_type, "stocktake_start");
        assert!(
            summary.contains("1件"),
            "summaryに件数を含むべき: {}",
            summary
        );
        // detail の key は stocktake_id・item_count の 2 つちょうど（35「操作ログ」、Matrix K8）
        let detail: String = conn
            .query_row(
                "SELECT detail_json FROM operation_logs WHERE operation_type = 'stocktake_start'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let detail: serde_json::Value = serde_json::from_str(&detail).unwrap();
        let mut keys: Vec<&String> = detail.as_object().unwrap().keys().collect();
        keys.sort();
        assert_eq!(keys, ["item_count", "stocktake_id"]);
    }

    // テストヘルパー: start_stocktake + 明細ごとに save_stock_count（既定、日付 D、明細つき）+
    // 35 の完了の (i)(iii) を満たす正常な行が全て 0 の Z004 を 2 本（精算日 D−1 と D。Matrix T2）
    fn start_and_count_all(
        conn: &mut DbConnection,
        counts: &[(&str, i64)], // [(product_code, count_value)]
    ) -> i64 {
        let result = start_stocktake(conn).unwrap();
        for (pc, count) in counts {
            let item = item_id(conn, result.stocktake_id, pc);
            save(conn, pc, *count, D, false, Some(item)).unwrap();
        }
        settlement(conn, 1, D1);
        settlement(conn, 2, D);
        result.stocktake_id
    }

    // ===== complete_stocktake テスト =====

    #[test]
    fn test_complete_req205_not_found() {
        // REQ-205: 棚卸し（確定 — 存在しないID → NotFound）
        // BIZ-06 §20.5: 存在しないID → NotFound
        let (_dir, mut conn) = setup_test_db();
        let req = CompleteStocktakeRequest {
            stocktake_id: 9999,
            today: D.to_string(),
        };
        let result = complete_stocktake(&mut conn, &req);
        assert!(matches!(result, Err(BizError::NotFound(_))));
    }

    #[test]
    fn test_complete_req205_already_completed() {
        // REQ-205: 棚卸し（確定 — 完了済みへの二重確定 → StocktakeNotInProgress）
        // BIZ-06 §20.5: 完了済み → StocktakeNotInProgress
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "CAC-001", 10);
        let st_id = start_and_count_all(&mut conn, &[("CAC-001", 10)]);

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        complete_stocktake(&mut conn, &req).unwrap(); // 1回目

        let result = complete_stocktake(&mut conn, &req); // 2回目
        assert!(matches!(result, Err(BizError::StocktakeNotInProgress(_))));
    }

    #[test]
    fn test_complete_req205_total_cost_multiple_products() {
        // REQ-205: 棚卸し（確定 — 複数商品のtotal_cost計算）
        // BIZ-06 §20.5: 複数商品のtotal_cost計算
        let (_dir, mut conn) = setup_test_db();
        seed_product_custom(&conn, "TC-001", false, 10, 50_000);
        seed_product_custom(&conn, "TC-002", false, 5, 100_000);
        let st_id = start_and_count_all(&mut conn, &[("TC-001", 3), ("TC-002", 2)]);

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        let result = complete_stocktake(&mut conn, &req).unwrap();
        // 500 円*3 + 1000 円*2 = 3500（原価は 1/100 円の 50000・100000）
        assert_eq!(result.total_cost, 3500);
    }

    #[test]
    fn test_complete_req205_total_cost_overflow() {
        // REQ-205: 棚卸し（確定 — total_costオーバーフロー → ValidationFailed）
        // BIZ-06 §20.5: オーバーフロー → ValidationFailed。原価は 1/100 円として読み、
        // 円の合計が i64 を越える条件を数量の × 100 で保つ（Matrix T2b）
        let (_dir, mut conn) = setup_test_db();
        seed_product_custom(&conn, "OV-001", false, 10, i64::MAX / 2 + 1);
        let st_id = start_and_count_all(&mut conn, &[("OV-001", 200)]);

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        let result = complete_stocktake(&mut conn, &req);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref msg)) if msg.contains("オーバーフロー")),
            "オーバーフローでValidationFailed: {:?}",
            result
        );
    }

    // ===== 評価額の計算（35 §20.5a SPEC-STK-VAL-D1〜D6） =====

    fn is_overflow<T: std::fmt::Debug>(result: &Result<T, BizError>) -> bool {
        matches!(result, Err(BizError::ValidationFailed(msg)) if msg.contains("オーバーフロー"))
    }

    fn stocktake_header(conn: &DbConnection, st_id: i64) -> (String, Option<i64>) {
        conn.query_row(
            "SELECT status, total_cost FROM stocktakes WHERE id = ?1",
            rusqlite::params![st_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    #[test]
    fn test_complete_req205_total_cost_length_product_per_meter() {
        // REQ-205 / SPEC-STK-VAL-D1・D4 / §20.5 ステップ 5c-d・8: 長さ商品の原価は 1 m あたり
        let (_dir, mut conn) = setup_test_db();
        seed_product_with_unit(&conn, "LEN-001", "m", 200, 38_500);
        seed_product_with_unit(&conn, "PCS-001", "pcs", 5, 30_000);
        let st_id = start_and_count_all(&mut conn, &[("LEN-001", 153), ("PCS-001", 5)]);

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        let result = complete_stocktake(&mut conn, &req).unwrap();
        // 385 円 × 1.53 = 589.05、300 円 × 5 = 1500、合計 2089.05 → 2089
        assert_eq!(result.total_cost, 2089);
        assert_eq!(stocktake_header(&conn, st_id).1, Some(2089));

        let detail: String = conn
            .query_row(
                "SELECT detail_json FROM operation_logs
                 WHERE operation_type = 'stocktake_complete' ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let detail: serde_json::Value = serde_json::from_str(&detail).unwrap();
        assert_eq!(detail["total_cost"], 2089);

        let valuation: i64 = conn
            .query_row(
                "SELECT valuation_cost_price_centi FROM stocktake_items
                 WHERE stocktake_id = ?1 AND product_code = 'LEN-001'",
                rusqlite::params![st_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            valuation, 38_500,
            "valuation_cost_price_centi は基準数量あたりの原価のまま"
        );
    }

    #[test]
    fn test_complete_req205_total_cost_rounds_half_up_at_yen() {
        // REQ-205 / SPEC-STK-VAL-D4: 保存値の総額は円未満を四捨五入（0.50 円 → 1、0.49 円 → 0）
        for (actual_count, expected) in [(50, 1), (49, 0)] {
            let (_dir, mut conn) = setup_test_db();
            seed_product_with_unit(&conn, "LEN-001", "m", 100, 100); // 1 m あたり 1 円
            let st_id = start_and_count_all(&mut conn, &[("LEN-001", actual_count)]);

            let req = CompleteStocktakeRequest {
                stocktake_id: st_id,
                today: D.to_string(),
            };
            let result = complete_stocktake(&mut conn, &req).unwrap();
            assert_eq!(result.total_cost, expected, "実カウント {actual_count} cm");
            assert_eq!(stocktake_header(&conn, st_id).1, Some(expected));
        }
    }

    #[test]
    fn test_complete_req205_negative_cost_rolls_back() {
        // REQ-205 / SPEC-STK-VAL-D5: 負の原価は確定を止め、header・明細・在庫を変えない
        let (_dir, mut conn) = setup_test_db();
        seed_product_with_unit(&conn, "NG-001", "pcs", 10, 300);
        seed_product_with_unit(&conn, "NG-002", "pcs", 10, 300);
        let st_id = start_and_count_all(&mut conn, &[("NG-001", 7), ("NG-002", 7)]);
        conn.execute(
            "UPDATE products SET cost_price_centi = -1 WHERE product_code = 'NG-002'",
            [],
        )
        .unwrap();

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        let result = complete_stocktake(&mut conn, &req);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref msg)) if msg.contains("NG-002")),
            "負の原価は ValidationFailed（商品コード入り）: {:?}",
            result
        );

        assert_eq!(
            stocktake_header(&conn, st_id),
            ("in_progress".to_string(), None)
        );
        let valued: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM stocktake_items
                 WHERE stocktake_id = ?1 AND valuation_cost_price_centi IS NOT NULL",
                rusqlite::params![st_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            valued, 0,
            "明細の valuation_cost_price_centi は NULL のまま"
        );
        let stocks: Vec<i64> = conn
            .prepare("SELECT stock_quantity FROM products ORDER BY product_code")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        // 計数 7 の保存が写しを 7 に求め直す（ADR D5）。拒否で写しが変わらない（期待を変える。Matrix T2）
        assert_eq!(stocks, vec![7, 7], "在庫は確定前のまま");
    }

    #[test]
    fn test_complete_req205_total_cost_yen_overflow() {
        // REQ-205 / SPEC-STK-VAL-D5: i128 の合計は収まるが円額が i64 を越える → ValidationFailed。
        // 原価は 1/100 円として読み、数量を × 100 にして条件を保つ（Matrix T2b）
        let (_dir, mut conn) = setup_test_db();
        seed_product_with_unit(&conn, "YO-001", "pcs", 1, i64::MAX / 2 + 1);
        seed_product_with_unit(&conn, "YO-002", "pcs", 1, i64::MAX / 2 + 1);
        let st_id = start_and_count_all(&mut conn, &[("YO-001", 100), ("YO-002", 100)]);

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        let result = complete_stocktake(&mut conn, &req);
        assert!(is_overflow(&result), "円額の桁あふれ: {:?}", result);
        assert_eq!(stocktake_header(&conn, st_id).0, "in_progress");
    }

    #[test]
    fn test_complete_req205_operation_log() {
        // REQ-205 / 35「操作ログ」（期待を変える。Matrix K8・T2）: summary の差異の件数は差異のある明細の数、
        // detail の key は stocktake_id・total_cost・total_items・difference_count の 4 つちょうど
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "OL-001", 10);
        seed_product(&conn, "OL-002", 5);
        let st_id = start_and_count_all(&mut conn, &[("OL-001", 8), ("OL-002", 5)]);

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        complete_stocktake(&mut conn, &req).unwrap();

        let (op_type, summary, detail): (String, String, Option<String>) = conn
            .query_row(
                "SELECT operation_type, summary, detail_json FROM operation_logs
                 WHERE operation_type = 'stocktake_complete' ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(op_type, "stocktake_complete");
        assert!(summary.contains("差異: 1件"), "{summary}");
        let detail: serde_json::Value = serde_json::from_str(&detail.unwrap()).unwrap();
        let mut keys: Vec<&String> = detail.as_object().unwrap().keys().collect();
        keys.sort();
        assert_eq!(
            keys,
            [
                "difference_count",
                "stocktake_id",
                "total_cost",
                "total_items"
            ]
        );
        assert_eq!(detail["stocktake_id"], st_id);
        assert_eq!(detail["difference_count"], 1);
    }

    // ===== D-2統合テスト =====

    #[test]
    fn test_complete_stocktake_req205_includes_integrity_result() {
        // REQ-205: 棚卸し（確定 — IntegrityResult が返る）
        // BIZ-06 D-2: 確定後に IntegrityResult が返る
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "D2-001", 10);
        let st_id = start_and_count_all(&mut conn, &[("D2-001", 10)]);

        let req = CompleteStocktakeRequest {
            stocktake_id: st_id,
            today: D.to_string(),
        };
        let result = complete_stocktake(&mut conn, &req).unwrap();

        assert!(
            result.integrity_result.is_some(),
            "D-2: integrity_result が返るべき"
        );
        let ir = result.integrity_result.unwrap();
        assert!(ir.checked_count > 0, "checked_count > 0");
    }

    #[test]
    fn test_get_stocktake_record_req206_not_found_message() {
        let (_dir, conn) = setup_test_db();

        let result = get_stocktake_record(&conn, 99_999);

        assert!(matches!(
            result,
            Err(BizError::NotFound(message)) if message.contains("棚卸し記録が見つかりません")
        ));
    }

    #[test]
    fn test_get_stocktake_record_req206_status_enum_and_fail_fast() {
        let (_dir, conn) = setup_test_db();
        let in_progress_id = create_stocktake(&conn, "in_progress");
        let completed_id = create_stocktake(&conn, "completed");

        let in_progress = get_stocktake_record(&conn, in_progress_id).unwrap();
        let completed = get_stocktake_record(&conn, completed_id).unwrap();
        assert_eq!(in_progress.status, StocktakeStatus::InProgress);
        assert_eq!(completed.status, StocktakeStatus::Completed);

        conn.execute_batch("PRAGMA ignore_check_constraints = ON;")
            .unwrap();
        let invalid_id = create_stocktake(&conn, "unexpected");
        let result = get_stocktake_record(&conn, invalid_id);
        assert!(matches!(result, Err(BizError::DatabaseError(_))));
    }

    // =====================================================================
    // b1: 計数の保存と読取り（Matrix の C。ADR D5、35）
    // =====================================================================

    #[test]
    fn test_save_stock_count_req205_default_saves_and_recomputes() {
        // REQ-205 / SPEC-STK-CNT-D5 / 35「計数の保存」/ Matrix C1
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C1-A", 10);
        let result = save(&mut conn, "C1-A", 8, D, false, None).unwrap();
        assert!(result.id > 0);
        assert_eq!(
            result,
            StockCountSaveResult {
                id: result.id,
                count_kind: StockCountKind::Default,
                stock: 8,
                stock_range_low: None,
                stock_range_high: None,
                recount_reasons: vec![],
                later_count_exists: false,
            }
        );
        let row: (String, i64, String, String, Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT product_code, count_value, count_kind, business_date, settlement_no, seen_movement_id
                 FROM stock_counts WHERE id = ?1",
                [result.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .unwrap();
        assert_eq!(
            row,
            ("C1-A".into(), 8, "default".into(), D.into(), None, None)
        );
        assert_eq!(copy(&conn, "C1-A"), (8, false, None, None));
    }

    #[test]
    fn test_save_stock_count_req205_fresh_stores_cursor_in_same_tx() {
        // REQ-205 / SPEC-STK-CNT-D3・D9 / Matrix C2: fresh は今日だけ、カーソルは最大の精算回数（無ければ NULL。
        // 鍵の n）と MAX(id) と取り消したものを含む MAX(csv_imports.id)（無ければ NULL。Codex の closure の Gated Amendment）
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C2-A", 10);
        type Cursor = (String, Option<i64>, Option<i64>, Option<i64>);
        let cursor = |conn: &DbConnection, id: i64| -> Cursor {
            conn.query_row(
                "SELECT count_kind, settlement_no, seen_movement_id, seen_import_id
                 FROM stock_counts WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap()
        };
        // 取込みの無い DB: settlement_no は NULL（0 にしない）
        let max_id: i64 = conn
            .query_row("SELECT MAX(id) FROM inventory_movements", [], |r| r.get(0))
            .unwrap();
        let first = save(&mut conn, "C2-A", 9, D, true, None).unwrap();
        assert_eq!(first.count_kind, StockCountKind::Fresh);
        assert_eq!(
            cursor(&conn, first.id),
            ("fresh".into(), None, Some(max_id), None)
        );
        // 取込みと movement のある DB
        settlement(&conn, 7, D1);
        let moved = receive(&conn, "C2-A", 1, D1);
        let last_import = settlement(&conn, 3, D2);
        let second = save(&mut conn, "C2-A", 9, D, true, None).unwrap();
        assert_eq!(
            cursor(&conn, second.id),
            ("fresh".into(), Some(7), Some(moved), Some(last_import))
        );
        // 今日以外の日付は拒む
        assert!(matches!(
            save(&mut conn, "C2-A", 9, D1, true, None),
            Err(BizError::ValidationFailed(_))
        ));

        // 列: 精算 100 = I1 → 99（A × −1）= I2 → fresh（c = I2）→ 100 を取消で、判定の n = 99（99 は入る）
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C2-B", 10);
        let i1 = settlement(&conn, 100, D);
        let i2 = settlement(&conn, 99, D);
        let sold = z004_movement(&conn, "C2-B", -1, i2);
        let saved = save(&mut conn, "C2-B", 9, D, true, None).unwrap();
        assert_eq!(
            cursor(&conn, saved.id),
            ("fresh".into(), Some(100), Some(sold), Some(i2))
        );
        roll_back(&conn, i1);
        recompute(&mut conn, "C2-B");
        assert_eq!(copy(&conn, "C2-B"), (9, false, None, None));
        // 最大の id の取込みが取り消されていても、取込みのカーソルはその id（取り消したものを含む MAX）
        let i3 = settlement(&conn, 101, D);
        roll_back(&conn, i3);
        let again = save(&mut conn, "C2-B", 9, D, true, None).unwrap();
        assert_eq!(cursor(&conn, again.id).3, Some(i3));
    }

    #[test]
    fn test_save_stock_count_req205_rejects_future_or_invalid_dates() {
        // REQ-205 / Z2 / Matrix C3: 未来・形の違う日付・実在しない日付は何も書かずに拒む
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C3-A", 10);
        for date in [D_NEXT, "2026-4-7", "2026-02-30"] {
            assert!(
                matches!(
                    save(&mut conn, "C3-A", 5, date, false, None),
                    Err(BizError::ValidationFailed(_))
                ),
                "{date}"
            );
        }
        assert!(matches!(
            save(&mut conn, "C3-A", 5, D_NEXT, false, None),
            Err(BizError::ValidationFailed(message)) if message == "未来の日付には保存できません"
        ));
        assert_eq!(count_rows(&conn), 0);
        assert_eq!(copy(&conn, "C3-A"), (10, false, None, None));
    }

    #[test]
    fn test_save_stock_count_req205_count_value_range() {
        // REQ-205 / Z1 / Matrix C4: 0 と 999999999 は保存でき、負と 1000000000 は拒む
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C4-A", 10);
        for bad in [-1, 1_000_000_000] {
            assert!(matches!(
                save(&mut conn, "C4-A", bad, D, false, None),
                Err(BizError::ValidationFailed(_))
            ));
        }
        assert_eq!(count_rows(&conn), 0);
        assert_eq!(save(&mut conn, "C4-A", 0, D, false, None).unwrap().stock, 0);
        assert_eq!(
            save(&mut conn, "C4-A", 999_999_999, D, true, None)
                .unwrap()
                .stock,
            999_999_999
        );
        assert_eq!(count_rows(&conn), 2);
    }

    #[test]
    fn test_save_stock_count_req205_rejects_mismatched_stocktake_item() {
        // REQ-205 / 35「計数の保存」/ Matrix C5: 進行中でない・存在しない・別の商品の明細は書く前に拒む
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C5-A", 10);
        seed_product(&conn, "C5-B", 10);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item_b = item_id(&conn, st_id, "C5-B");
        let done = create_stocktake(&conn, "completed");
        let done_item = seed_stocktake_item(&conn, done, "C5-A", 10, None);
        for item in [item_b, 999_999, done_item] {
            assert!(
                matches!(
                    save(&mut conn, "C5-A", 3, D, false, Some(item)),
                    Err(BizError::ValidationFailed(message)) if message == "この明細は別の商品です"
                ),
                "{item}"
            );
        }
        assert_eq!(count_rows(&conn), 0);
    }

    #[test]
    fn test_save_stock_count_req205_item_copy_follows_effective_count() {
        // REQ-205 / 35「計数の保存」/ Matrix C6: 明細の actual_count・counted_at は効く計数に揃う
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C6-A", 10);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "C6-A");
        let copy = |conn: &DbConnection| -> (Option<i64>, Option<String>) {
            conn.query_row(
                "SELECT actual_count, counted_at FROM stocktake_items WHERE id = ?1",
                [item],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap()
        };
        let counted_at = |conn: &DbConnection, id: i64| -> String {
            conn.query_row(
                "SELECT counted_at FROM stock_counts WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap()
        };
        // 効く計数（今日の 8）。保存の時刻を後の入力の時刻と区別できる過去の時刻にする
        conn.execute(
            "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                 stocktake_item_id, counted_at)
             VALUES ('C6-A', 8, 'default', ?1, ?2, '2026-10-15T10:00:00')",
            rusqlite::params![D, item],
        )
        .unwrap();
        // 後入力の古い日付の計数は効かない（値も時刻も先の効く計数のまま）
        let back = save(&mut conn, "C6-A", 9, D1, false, Some(item)).unwrap();
        assert!(back.later_count_exists);
        assert_eq!(
            copy(&conn),
            (Some(8), Some("2026-10-15T10:00:00".to_string()))
        );
        // fresh は先の計数を置き換える
        let fresh = save(&mut conn, "C6-A", 7, D, true, Some(item)).unwrap();
        assert_eq!(copy(&conn), (Some(7), Some(counted_at(&conn, fresh.id))));
        // 今日の既定の計数は後の計数として効く
        let latest = save(&mut conn, "C6-A", 6, D, false, Some(item)).unwrap();
        assert_eq!(copy(&conn), (Some(6), Some(counted_at(&conn, latest.id))));
    }

    #[test]
    fn test_preview_stock_count_placement_req205_later_count_exists() {
        // REQ-205 / SPEC-STK-CNT-D3「計数どうし」・D5 / Matrix C7（保存は拒まない。読取りだけ）
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C7-A", 10);
        seed_product(&conn, "C7-F", 10);
        save(&mut conn, "C7-A", 8, D, false, None).unwrap();
        save(&mut conn, "C7-F", 8, D, true, None).unwrap();
        let rows = count_rows(&conn);
        let later = |conn: &DbConnection, code: &str, date: &str| {
            preview_stock_count_placement(conn, code, date, D)
                .unwrap()
                .later_count_exists
        };
        assert!(later(&conn, "C7-A", D1));
        assert!(!later(&conn, "C7-A", D), "同じ日付の既定どうしは重なる");
        assert!(
            later(&conn, "C7-F", D1),
            "fresh の後の前の日の既定は fresh が後"
        );
        assert!(
            !later(&conn, "C7-F", D),
            "fresh の後の同じ日付の既定は重なる"
        );
        assert_eq!(count_rows(&conn), rows);
    }

    #[test]
    fn test_zero_uncounted_items_req205_one_tx_per_request() {
        // REQ-205 / SPEC-STK-CNT-D6 / 35「未計数を片付ける操作」/ Matrix C8
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C8-A", 5);
        seed_product(&conn, "C8-B", 3);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        seed_product(&conn, "C8-C", 0); // 開始の後に登録（明細なし）
        let codes = |list: &[&str]| list.iter().map(|c| c.to_string()).collect::<Vec<_>>();
        // 明細の無い商品が 1 つでもあれば何も書かない
        assert!(matches!(
            zero_uncounted_items(&mut conn, st_id, &codes(&["C8-A", "C8-C"]), D, D),
            Err(BizError::ValidationFailed(message)) if message == "この商品は棚卸しの対象にありません: C8-C"
        ));
        assert_eq!(count_rows(&conn), 0);
        // 未来の日付は拒む
        assert!(matches!(
            zero_uncounted_items(&mut conn, st_id, &codes(&["C8-A"]), D_NEXT, D),
            Err(BizError::ValidationFailed(_))
        ));
        let results =
            zero_uncounted_items(&mut conn, st_id, &codes(&["C8-A", "C8-B"]), D1, D).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|r| r.stock == 0 && r.count_kind == StockCountKind::Default));
        let referenced: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM stock_counts sc JOIN stocktake_items si ON si.id = sc.stocktake_item_id
                 WHERE si.stocktake_id = ?1 AND sc.count_value = 0 AND sc.business_date = ?2",
                rusqlite::params![st_id, D1],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(referenced, 2);
        // ほかの理由が無ければ完了できる（精算日 D−2 と D の取込み）
        settlement(&conn, 1, D2);
        settlement(&conn, 2, D);
        complete(&mut conn, st_id).unwrap();
    }

    /// 業務日 D の入出庫と同じ日付 D の既定の計数で要再確認にする（理由 2）
    fn make_recheck(conn: &mut DbConnection, code: &str, item: Option<i64>) {
        receive(conn, code, 2, D);
        save(conn, code, 10, D, false, item).unwrap();
    }

    #[test]
    fn test_list_recheck_products_req205_actions_reasons_and_range() {
        // REQ-205 / 35「読取り」/ 40 CountRecoveryTarget / Matrix C9
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C9-A", 0);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item_a = item_id(&conn, st_id, "C9-A");
        seed_product(&conn, "C9-B", 0);
        seed_product(&conn, "C9-C", 4);
        seed_product_with_unit(&conn, "C9-L", "m", 0, 100);
        make_recheck(&mut conn, "C9-A", Some(item_a));
        make_recheck(&mut conn, "C9-B", None);
        make_recheck(&mut conn, "C9-L", None);
        let rows = count_rows(&conn);
        let targets = list_recheck_products(&conn).unwrap();
        let reasons = vec![RecountReason::SameDateMovement];
        assert_eq!(
            targets,
            vec![
                CountRecoveryTarget {
                    product_code: "C9-A".into(),
                    product_name: "商品C9-A".into(),
                    stock_unit: ProductStockUnit::Pcs,
                    stock: 12,
                    stock_range_low: Some(10),
                    stock_range_high: Some(12),
                    stocktake_item_id: Some(item_a),
                    stocktake_id: Some(st_id),
                    action: CountRecoveryAction::ActiveCount,
                    block_reason: None,
                    recount_reasons: reasons.clone(),
                },
                CountRecoveryTarget {
                    product_code: "C9-B".into(),
                    product_name: "商品C9-B".into(),
                    stock_unit: ProductStockUnit::Pcs,
                    stock: 12,
                    stock_range_low: Some(10),
                    stock_range_high: Some(12),
                    stocktake_item_id: None,
                    stocktake_id: None,
                    action: CountRecoveryAction::CountNow,
                    block_reason: None,
                    recount_reasons: reasons.clone(),
                },
                CountRecoveryTarget {
                    product_code: "C9-L".into(),
                    product_name: "商品C9-L".into(),
                    stock_unit: ProductStockUnit::M,
                    stock: 12,
                    stock_range_low: Some(10),
                    stock_range_high: Some(12),
                    stocktake_item_id: None,
                    stocktake_id: None,
                    action: CountRecoveryAction::CountNow,
                    block_reason: None,
                    recount_reasons: reasons,
                },
            ]
        );
        assert_eq!(count_rows(&conn), rows);
    }

    #[test]
    fn test_get_stocktake_items_req205_effective_count_and_difference() {
        // REQ-205 / 35「読取り」/ Matrix C10: 差異 = 効く計数 − その計数に入っている記録だけの帳簿、
        // 計数済み = 明細を参照する計数がある
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "C10-A", 10);
        seed_product(&conn, "C10-B", 4);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item_a = item_id(&conn, st_id, "C10-A");
        // 開始の後、計数の前の日の入庫（帳簿 13。system_stock は 10 のまま）
        receive(&conn, "C10-A", 3, D1);
        recompute(&mut conn, "C10-A");
        save(&mut conn, "C10-A", 12, D, false, Some(item_a)).unwrap();
        // B は明細の写しだけに数がある（計数なし）→ 未計数
        conn.execute(
            "UPDATE stocktake_items SET actual_count = 4 WHERE product_code = 'C10-B'",
            [],
        )
        .unwrap();
        let (page, progress) = get_stocktake_items(&conn, st_id, None, None, 1, 50).unwrap();
        let a = &page.items[0];
        assert_eq!(
            (
                a.count_value,
                a.count_kind,
                a.count_business_date.as_deref(),
                a.current_stock,
                a.difference,
                a.stock_unit,
                a.recount_reasons.clone()
            ),
            (
                Some(12),
                Some(StockCountKind::Default),
                Some(D),
                12,
                Some(-1),
                ProductStockUnit::Pcs,
                vec![]
            )
        );
        let b = &page.items[1];
        assert_eq!((b.count_value, b.difference), (None, None));
        assert_eq!((progress.counted_items, progress.uncounted_items), (1, 1));
        let found = find_stocktake_item(&conn, st_id, "C10-A").unwrap().unwrap();
        assert_eq!(found.difference, Some(-1));
    }

    #[test]
    fn test_preview_stock_count_placement_req205_returns_count_context_without_stocktake() {
        // REQ-205 / 35・42 / Matrix C11: 棚卸しの明細が無くても計数の欄の文脈を返す
        let (_dir, mut conn) = setup_test_db();
        seed_product_with_unit(&conn, "C11-L", "m", 0, 100);
        seed_product(&conn, "C11-N", 7);
        make_recheck(&mut conn, "C11-L", None);
        let preview = preview_stock_count_placement(&conn, "C11-L", D, D).unwrap();
        assert_eq!(
            preview,
            CountPlacementPreview {
                later_count_exists: false,
                product_name: "商品C11-L".into(),
                stock_unit: ProductStockUnit::M,
                stock: 12,
                stock_range_low: Some(10),
                stock_range_high: Some(12),
                recount_reasons: vec![RecountReason::SameDateMovement],
                count_value: Some(10),
                count_kind: Some(StockCountKind::Default),
                count_business_date: Some(D.into()),
            }
        );
        let none = preview_stock_count_placement(&conn, "C11-N", D, D).unwrap();
        assert_eq!(
            (
                none.stock,
                none.count_value,
                none.count_kind,
                none.count_business_date
            ),
            (7, None, None, None)
        );
    }

    #[test]
    fn test_save_stock_count_req205_uses_today_argument_not_clock() {
        // REQ-205 / 35（BIZ は時計を読まない）/ Matrix W3
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "W3-A", 10);
        let request = |date: &str, fresh: bool| SaveStockCountRequest {
            product_code: "W3-A".into(),
            count_value: 5,
            fresh,
            business_date: date.into(),
            stocktake_item_id: None,
        };
        save_stock_count(&mut conn, &request("2000-01-05", true), "2000-01-05").unwrap();
        assert!(matches!(
            save_stock_count(&mut conn, &request("2000-01-06", false), "2000-01-05"),
            Err(BizError::ValidationFailed(message)) if message == "未来の日付には保存できません"
        ));
    }

    // =====================================================================
    // b1: 開始・完了（Matrix の K。ADR D6、35、§20.5a）
    // =====================================================================

    #[test]
    fn test_start_stocktake_req205_creates_items_without_auto_fill() {
        // REQ-205 / 35「開始と明細」/ Matrix K7: 廃番かつ在庫 0 も自動の 0 で埋めない
        let (_dir, mut conn) = setup_test_db();
        seed_product_custom(&conn, "K7-A", false, 10, 300);
        seed_product_custom(&conn, "K7-Z", true, 0, 100);
        let result = start_stocktake(&mut conn).unwrap();
        assert_eq!(result.item_count, 2);
        let actuals: Vec<Option<i64>> = conn
            .prepare("SELECT actual_count FROM stocktake_items ORDER BY product_code")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(actuals, vec![None, None]);
        assert_eq!(count_rows(&conn), 0);
    }

    #[test]
    fn test_complete_stocktake_req205_freezes_valuation_without_movements() {
        // REQ-205 / SPEC-STK-CNT-D6 / Matrix K1: 評価数量・カーソル・completed、movement を作らず写しを変えない
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "K1-A", 10);
        seed_product(&conn, "K1-B", 5);
        let st_id = start_and_count_all(&mut conn, &[("K1-A", 8), ("K1-B", 5)]);
        // 取込みのカーソルは取り消したものを含む MAX(csv_imports.id)（最大の精算回数 2 と値を離す）
        let rolled = settlement(&conn, 9, D2);
        roll_back(&conn, rolled);
        // 写しをわざと記録から食い違わせる（求め直すと 8 に戻る）
        conn.execute(
            "UPDATE products SET stock_quantity = 99 WHERE product_code = 'K1-A'",
            [],
        )
        .unwrap();
        let movements_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |r| r.get(0))
            .unwrap();
        let max_movement: i64 = conn
            .query_row("SELECT MAX(id) FROM inventory_movements", [], |r| r.get(0))
            .unwrap();
        let max_count: i64 = conn
            .query_row("SELECT MAX(id) FROM stock_counts", [], |r| r.get(0))
            .unwrap();

        complete(&mut conn, st_id).unwrap();

        let movements_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |r| r.get(0))
            .unwrap();
        assert_eq!(movements_after, movements_before);
        assert_eq!(copy(&conn, "K1-A").0, 99);
        let valuations: Vec<(String, i64, i64)> = conn
            .prepare(
                "SELECT product_code, valuation_quantity, valuation_cost_price_centi
                 FROM stocktake_items WHERE stocktake_id = ?1 ORDER BY product_code",
            )
            .unwrap()
            .query_map([st_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            valuations,
            vec![("K1-A".into(), 99, 300), ("K1-B".into(), 5, 300)]
        );
        let header: (
            String,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
        ) = conn
            .query_row(
                "SELECT status, cutoff_import_id, cutoff_count_id, cutoff_movement_id, completed_at
                 FROM stocktakes WHERE id = ?1",
                [st_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        assert_eq!(
            (&header.0[..], header.1, header.2, header.3),
            (
                "completed",
                Some(rolled),
                Some(max_count),
                Some(max_movement)
            )
        );
        assert!(header.4.unwrap().starts_with(D), "完了日は CMD が渡す今日");

        // 取込み・計数・movement の無い DB のカーソルは NULL・NULL・0
        let (_dir, mut conn) = setup_test_db();
        seed_product_custom(&conn, "K1-Z", true, 0, 100);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        complete(&mut conn, st_id).unwrap();
        let cursors: (Option<i64>, Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT cutoff_import_id, cutoff_count_id, cutoff_movement_id FROM stocktakes WHERE id = ?1",
                [st_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(cursors, (None, None, Some(0)));
    }

    fn pending_hint(recovery: &StocktakeRecovery) -> SettlementPendingHint {
        assert!(recovery
            .targets
            .iter()
            .any(|t| t.block_reason == Some(CompletionBlockReason::SettlementPending)));
        recovery.settlement_pending.clone().unwrap()
    }

    fn assert_nothing_written(conn: &DbConnection, st_id: i64) {
        assert_eq!(header_status(conn, st_id), "in_progress");
        let valued: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM stocktake_items WHERE stocktake_id = ?1
                 AND (valuation_quantity IS NOT NULL OR valuation_cost_price_centi IS NOT NULL)",
                [st_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(valued, 0);
    }

    #[test]
    fn test_complete_stocktake_req205_settlement_pending_three_conditions_and_hint() {
        // REQ-205 / SPEC-STK-CNT-D6 / 35 の表 settlement_pending (i)〜(iii) / 40 SettlementPendingHint / Matrix K2
        let setup = |settlements: &[(i64, &str)]| {
            let (dir, mut conn) = setup_test_db();
            seed_product(&conn, "K2-A", 10);
            let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
            let item = item_id(&conn, st_id, "K2-A");
            save(&mut conn, "K2-A", 10, D, false, Some(item)).unwrap();
            for (no, date) in settlements {
                settlement(&conn, *no, date);
            }
            (dir, conn, st_id)
        };
        // (i) 精算日が X 以上の精算が無い
        let (_d, mut conn, st_id) = setup(&[(1, D2)]);
        let hint = pending_hint(&guard(complete(&mut conn, st_id)));
        assert_eq!(
            hint,
            SettlementPendingHint {
                missing: vec![],
                on_or_after: Some(D.into()),
                before: None
            }
        );
        assert_nothing_written(&conn, st_id);
        // (iii) 精算日が X より前の精算が無い
        let (_d, mut conn, st_id) = setup(&[(1, D)]);
        let hint = pending_hint(&guard(complete(&mut conn, st_id)));
        assert_eq!(
            hint,
            SettlementPendingHint {
                missing: vec![],
                on_or_after: None,
                before: Some(D.into())
            }
        );
        // (ii) 取り込まれていない 2 の隣の精算 1（D−1）・3（D+1）が X を挟む
        let (_d, mut conn, st_id) = setup(&[(1, D1), (3, D_NEXT)]);
        let hint = pending_hint(&guard(complete(&mut conn, st_id)));
        assert_eq!(
            hint,
            SettlementPendingHint {
                missing: vec![SettlementGap { from: 2, to: 2 }],
                on_or_after: None,
                before: None
            }
        );
        // fresh で n があれば k ≤ n の欠けは見ない（(iii) も見ない）
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "K2-F", 10);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "K2-F");
        settlement(&conn, 1, D1);
        settlement(&conn, 3, D);
        save(&mut conn, "K2-F", 10, D, true, Some(item)).unwrap();
        complete(&mut conn, st_id).unwrap();
    }

    #[test]
    fn test_complete_stocktake_req205_settlement_pending_fresh_cursor_uses_import_ids() {
        // REQ-205 / SPEC-STK-CNT-D3・D6・D7（完了の (iii) の判定の n は取込みのカーソルから求める。
        // Codex の closure の Gated Amendment）/ Matrix K2 の列
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "K2N-A", 10);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "K2N-A");
        let i1 = settlement(&conn, 100, D);
        save(&mut conn, "K2N-A", 10, D, true, Some(item)).unwrap();
        let pending = |conn: &DbConnection| -> bool {
            let ledger = read_ledger(conn, "K2N-A").unwrap();
            let (effective, _) = stock_ledger::effective_counts(&ledger.counts).unwrap();
            let mut hint = SettlementPendingHint {
                missing: vec![],
                on_or_after: None,
                before: None,
            };
            settlement_pending(&effective, &ledger.settlements, &mut hint)
        };
        // 99 を後追いで取り込んでも判定の n = 100 なので (iii) は出ない
        settlement(&conn, 99, D);
        assert!(!pending(&conn));
        // 100 を取り消すと判定の n は NULL（99 は保存の後の取込み）で (iii)
        roll_back(&conn, i1);
        recompute(&mut conn, "K2N-A");
        let expect_pending = |conn: &mut DbConnection| {
            let recovery = guard(complete(conn, st_id));
            assert_eq!(
                recovery
                    .targets
                    .iter()
                    .map(|t| t.block_reason)
                    .collect::<Vec<_>>(),
                vec![Some(CompletionBlockReason::SettlementPending)],
                "A は 99・100 に行が無いので要再確認にならない"
            );
            assert_eq!(
                pending_hint(&recovery),
                SettlementPendingHint {
                    missing: vec![],
                    on_or_after: None,
                    before: Some(D.into())
                }
            );
        };
        expect_pending(&mut conn);
        // 100 を取り込み直しても、取り込み直した取込みは保存の後なので (iii) が残る
        settlement(&conn, 100, D);
        recompute(&mut conn, "K2N-A");
        expect_pending(&mut conn);
        // 入れ直すと判定の n = 100 になり完了できる
        save(&mut conn, "K2N-A", 10, D, true, Some(item)).unwrap();
        complete(&mut conn, st_id).unwrap();
    }

    #[test]
    fn test_complete_stocktake_req205_blocks_on_recheck() {
        // REQ-205 / SPEC-STK-CNT-D6 の 2 / Matrix K3
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "K3-A", 0);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "K3-A");
        make_recheck(&mut conn, "K3-A", Some(item));
        settlement(&conn, 1, D1);
        settlement(&conn, 2, D);
        let recovery = guard(complete(&mut conn, st_id));
        assert_eq!(recovery.targets.len(), 1);
        let target = &recovery.targets[0];
        assert_eq!(target.block_reason, Some(CompletionBlockReason::Recheck));
        assert_eq!(
            target.recount_reasons,
            vec![RecountReason::SameDateMovement]
        );
        assert_eq!(target.stocktake_item_id, Some(item));
        assert_eq!(recovery.settlement_pending, None);
        assert_nothing_written(&conn, st_id);
    }

    #[test]
    fn test_complete_stocktake_req205_uncounted_except_discontinued_zero() {
        // REQ-205 / SPEC-STK-CNT-D6 の 3 / Matrix K4: 理由は全部集めてから返す
        let (_dir, mut conn) = setup_test_db();
        seed_product_custom(&conn, "K4-A", false, 0, 300);
        seed_product_custom(&conn, "K4-B", false, 3, 300);
        seed_product_custom(&conn, "K4-C", true, 0, 300);
        seed_product_custom(&conn, "K4-D", true, 5, 300);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "K4-A");
        make_recheck(&mut conn, "K4-A", Some(item));
        settlement(&conn, 1, D1);
        settlement(&conn, 2, D);
        let recovery = guard(complete(&mut conn, st_id));
        let got: Vec<(String, Option<CompletionBlockReason>)> = recovery
            .targets
            .iter()
            .map(|t| (t.product_code.clone(), t.block_reason))
            .collect();
        assert_eq!(
            got,
            vec![
                ("K4-A".into(), Some(CompletionBlockReason::Recheck)),
                ("K4-B".into(), Some(CompletionBlockReason::Uncounted)),
                ("K4-D".into(), Some(CompletionBlockReason::Uncounted)),
            ]
        );
        assert!(recovery.targets[1].recount_reasons.is_empty());
        assert_nothing_written(&conn, st_id);
    }

    #[test]
    fn test_complete_stocktake_req205_total_cost_safe_integer_range() {
        // REQ-205 / 共通規則「入力の上限と安全な整数の範囲」(c) / Matrix K5
        let run = |products: usize| {
            let (dir, mut conn) = setup_test_db();
            let mut counts = Vec::new();
            for index in 0..products {
                let code = format!("K5-{index:02}");
                seed_product_custom(&conn, &code, false, 0, 99_999_999);
                counts.push(code);
            }
            let counts: Vec<(&str, i64)> =
                counts.iter().map(|c| (c.as_str(), 999_999_999)).collect();
            let st_id = start_and_count_all(&mut conn, &counts);
            let result = complete(&mut conn, st_id);
            (dir, conn, st_id, result)
        };
        let (_d, conn, st_id, result) = run(10);
        assert!(matches!(
            result,
            Err(BizError::ValidationFailed(message)) if message == "合計金額が大きすぎます"
        ));
        assert_nothing_written(&conn, st_id);
        let (_d, _conn, _st, result) = run(9);
        assert_eq!(result.unwrap().total_cost, 8_999_999_901_000_000);
    }

    // =====================================================================
    // b1: 記録の詳細と完了の後の差（Matrix の R。ADR D6、35、65）
    // =====================================================================

    fn record(conn: &DbConnection, st_id: i64) -> StocktakeRecordDetail {
        get_stocktake_record(conn, st_id).unwrap()
    }

    #[test]
    fn test_get_stocktake_record_req206_items_at_cutoff_with_loss_cost() {
        // REQ-206 / 35「完了」・65 / SPEC-UNIT-D6 / Matrix R1
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "R1-A", 10);
        seed_product_with_unit(&conn, "R1-L", "m", 200, 38_500);
        let st_id = start_and_count_all(&mut conn, &[("R1-A", 8), ("R1-L", 150)]);
        complete(&mut conn, st_id).unwrap();
        // 完了の後の計数はカーソルの外
        save(&mut conn, "R1-A", 3, D, true, None).unwrap();
        let detail = record(&conn, st_id);
        assert_eq!(detail.status, StocktakeStatus::Completed);
        assert_eq!(
            detail.items,
            vec![
                StocktakeRecordDetailItem {
                    product_code: "R1-A".into(),
                    product_name: "商品R1-A".into(),
                    department_name: detail.items[0].department_name.clone(),
                    stock_unit: ProductStockUnit::Pcs,
                    count_value: Some(8),
                    count_kind: Some(StockCountKind::Default),
                    count_business_date: Some(D.into()),
                    difference: Some(-2),
                    valuation_quantity: Some(8),
                    valuation_cost_price_centi: Some(300),
                    loss_cost_centi: Some("600".into()),
                },
                StocktakeRecordDetailItem {
                    product_code: "R1-L".into(),
                    product_name: "商品R1-L".into(),
                    department_name: detail.items[1].department_name.clone(),
                    stock_unit: ProductStockUnit::M,
                    count_value: Some(150),
                    count_kind: Some(StockCountKind::Default),
                    count_business_date: Some(D.into()),
                    difference: Some(-50),
                    valuation_quantity: Some(150),
                    valuation_cost_price_centi: Some(38_500),
                    loss_cost_centi: Some("19250".into()),
                },
            ]
        );
    }

    #[test]
    fn test_get_stocktake_record_req206_after_completion_diff_split() {
        // REQ-206 / SPEC-STK-CNT-D6 / 35「完了」/ Matrix R2: 確定の差と要再確認の差を分け、確定の差だけを合計
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "R2-A", 10);
        seed_product(&conn, "R2-B", 5);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let (item_a, item_b) = (item_id(&conn, st_id, "R2-A"), item_id(&conn, st_id, "R2-B"));
        save(&mut conn, "R2-A", 8, D1, false, Some(item_a)).unwrap();
        save(&mut conn, "R2-B", 5, D, false, Some(item_b)).unwrap();
        settlement(&conn, 1, D2);
        settlement(&conn, 2, D);
        complete(&mut conn, st_id).unwrap();
        // 完了の後に入れた完了日以前の日付の記録（A は計数の後で確定、B は計数と同じ日で決まらない）と、
        // 完了日より後の記録（範囲に入らない）
        receive(&conn, "R2-A", 2, D);
        receive(&conn, "R2-B", 1, D);
        receive(&conn, "R2-A", 4, D_NEXT);
        let diff = record(&conn, st_id).after_completion.unwrap();
        assert_eq!(
            diff,
            AfterCompletionDiff {
                confirmed: vec![AfterCompletionDiffItem {
                    product_code: "R2-A".into(),
                    product_name: "商品R2-A".into(),
                    stock_unit: ProductStockUnit::Pcs,
                    valuation_quantity: 8,
                    stock_at_completion: 10,
                    stock_at_completion_low: None,
                    stock_at_completion_high: None,
                    valuation_diff_yen: "6".into(),
                    valuation_diff_low_yen: None,
                    valuation_diff_high_yen: None,
                }],
                recheck: vec![AfterCompletionDiffItem {
                    product_code: "R2-B".into(),
                    product_name: "商品R2-B".into(),
                    stock_unit: ProductStockUnit::Pcs,
                    valuation_quantity: 5,
                    stock_at_completion: 6,
                    stock_at_completion_low: Some(5),
                    stock_at_completion_high: Some(6),
                    valuation_diff_yen: "3".into(),
                    valuation_diff_low_yen: Some("0".into()),
                    valuation_diff_high_yen: Some("3".into()),
                }],
                confirmed_total_yen: "6".into(),
            }
        );
    }

    #[test]
    fn test_get_stocktake_record_req206_after_completion_same_range_as_valuation() {
        // REQ-206 / SPEC-STK-CNT-D6（完了の後の比較の範囲 = 完了の時点にあった記録 + 完了の後に入れた
        // 完了の前の記録。計数はカーソル以下だけ）/ Matrix R2 (a)〜(d)
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "R2S-A", 10);
        seed_product(&conn, "R2S-B", 10);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let (item_a, item_b) = (
            item_id(&conn, st_id, "R2S-A"),
            item_id(&conn, st_id, "R2S-B"),
        );
        settlement(&conn, 100, D);
        save(&mut conn, "R2S-A", 5, D, true, Some(item_a)).unwrap();
        save(&mut conn, "R2S-B", 5, D, true, Some(item_b)).unwrap();
        // (a) 未来の日付の精算 101（A × 1）を完了の前に取り込む
        let s101 = settlement(&conn, 101, D_NEXT);
        z004_movement(&conn, "R2S-A", -1, s101);
        recompute(&mut conn, "R2S-A");
        // (b) 完了の前に業務日 T+1 の入庫（B × 2）
        receive(&conn, "R2S-B", 2, D_NEXT);
        recompute(&mut conn, "R2S-B");
        assert_eq!((copy(&conn, "R2S-A").0, copy(&conn, "R2S-B").0), (4, 7));
        complete(&mut conn, st_id).unwrap();
        let empty = AfterCompletionDiff {
            confirmed: vec![],
            recheck: vec![],
            confirmed_total_yen: "0".into(),
        };
        assert_eq!(record(&conn, st_id).after_completion.unwrap(), empty);
        // (c) 完了の後に業務日 T+1 の入庫を入れても差は出ない
        receive(&conn, "R2S-A", 2, D_NEXT);
        assert_eq!(record(&conn, st_id).after_completion.unwrap(), empty);
        // (d) 完了の後に保存した計数（日付 T = 完了日）は範囲に入らない
        save(&mut conn, "R2S-A", 3, D, false, None).unwrap();
        assert_eq!(record(&conn, st_id).after_completion.unwrap(), empty);
    }

    #[test]
    fn test_list_inventory_records_req206_stocktake_count_is_item_count() {
        // REQ-206 / 65（棚卸しの件数は明細の数。記録の詳細は補正の movement の節を持たない）/ Matrix R3
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "R3-A", 10);
        seed_product(&conn, "R3-B", 4);
        seed_product(&conn, "R3-C", 0);
        let st_id = start_and_count_all(&mut conn, &[("R3-A", 8), ("R3-B", 4), ("R3-C", 0)]);
        complete(&mut conn, st_id).unwrap();
        let query = crate::db::InventoryRecordQuery {
            record_type: Some("stocktake".to_string()),
            date_from: None,
            date_to: None,
            record_id: None,
            product_keyword: None,
            department_id: None,
            status: None,
            page: 1,
            per_page: 50,
        };
        let result = crate::db::disposal_repo::list_inventory_records(&conn, &query).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].item_count, 3);
        assert_eq!(record(&conn, st_id).item_count, 3);
    }

    #[test]
    fn test_get_stocktake_record_req206_after_completion_includes_backfilled_settlement() {
        // REQ-206 / SPEC-STK-CNT-D6（完了の後に埋めた欠けは、完了の時点の精算の n 以下なら範囲に入る。
        // 精算日を完了日と同じにして「精算日 < 完了日」の項では入らないようにする）/ Matrix R2 (e)
        const D3: &str = "2026-10-18";
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "R2E-C", 10);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "R2E-C");
        save(&mut conn, "R2E-C", 5, D, false, Some(item)).unwrap();
        for (no, date) in [(4, D1), (5, D), (6, D_NEXT), (8, D3)] {
            settlement(&conn, no, date);
        }
        recompute(&mut conn, "R2E-C");
        complete_stocktake(
            &mut conn,
            &CompleteStocktakeRequest {
                stocktake_id: st_id,
                today: D3.to_string(),
            },
        )
        .unwrap();
        // 欠けた 7（精算日 D3、C × 1）を後追いで取り込む。7 は A1 で計数に入らない
        let s7 = settlement(&conn, 7, D3);
        z004_movement(&conn, "R2E-C", -1, s7);
        assert_eq!(
            record(&conn, st_id).after_completion.unwrap(),
            AfterCompletionDiff {
                confirmed: vec![AfterCompletionDiffItem {
                    product_code: "R2E-C".into(),
                    product_name: "商品R2E-C".into(),
                    stock_unit: ProductStockUnit::Pcs,
                    valuation_quantity: 5,
                    stock_at_completion: 4,
                    stock_at_completion_low: None,
                    stock_at_completion_high: None,
                    valuation_diff_yen: "-3".into(),
                    valuation_diff_low_yen: None,
                    valuation_diff_high_yen: None,
                }],
                recheck: vec![],
                confirmed_total_yen: "-3".into(),
            }
        );
    }

    #[test]
    fn test_get_stocktake_record_req206_after_completion_ignores_reused_settlement_numbers() {
        // REQ-206 / SPEC-STK-CNT-D6（完了の後の差の範囲の精算は完了の時点の精算の n で決め、取消の後に
        // 同じ番号を使う取込みを数えない。Codex の closure の Gated Amendment）/ Matrix R6
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "R6-A", 10);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "R6-A");
        save(&mut conn, "R6-A", 5, D, false, Some(item)).unwrap();
        let i1 = settlement(&conn, 1, D1);
        let i2 = settlement(&conn, 2, D);
        complete(&mut conn, st_id).unwrap();
        // 完了の前の取込みを全て取り消し、初期化の後の新しい系列の精算 1・2 を取り込む
        roll_back(&conn, i1);
        roll_back(&conn, i2);
        let i3 = settlement(&conn, 1, D_NEXT);
        z004_movement(&conn, "R6-A", -1, i3);
        let i4 = settlement(&conn, 2, "2026-10-17");
        z004_movement(&conn, "R6-A", -1, i4);
        assert_eq!(
            record(&conn, st_id).after_completion.unwrap(),
            AfterCompletionDiff {
                confirmed: vec![],
                recheck: vec![],
                confirmed_total_yen: "0".into(),
            }
        );
    }

    #[test]
    fn test_get_stocktake_record_req206_after_completion_diff_clamps_negative() {
        // REQ-206 / SPEC-STK-CNT-D6（負の数量を評価の関数に渡さない）/ Matrix R5
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "R5-A", 0);
        let st_id = start_stocktake(&mut conn).unwrap().stocktake_id;
        let item = item_id(&conn, st_id, "R5-A");
        save(&mut conn, "R5-A", 0, D1, false, Some(item)).unwrap();
        receive(&conn, "R5-A", -2, D);
        recompute(&mut conn, "R5-A");
        settlement(&conn, 1, D2);
        settlement(&conn, 2, D);
        complete(&mut conn, st_id).unwrap();
        receive(&conn, "R5-A", 1, D);
        let detail = record(&conn, st_id);
        assert_eq!(detail.items[0].valuation_quantity, Some(0));
        assert_eq!(
            detail.after_completion.unwrap(),
            AfterCompletionDiff {
                confirmed: vec![],
                recheck: vec![],
                confirmed_total_yen: "0".into(),
            }
        );
    }
}
