//! BIZ-02: 在庫数の求め直し（記録が正、`products.stock_quantity` ほかは写し）
//!
//! docs/function-design/31-biz-inventory-service.md「計数の置き場と在庫の求め直しの契約」と
//! ADR SPEC-STK-CNT-D2・D3 に基づく実装。計数・入出庫・初期在庫・精算（Z004）の窓
//! （判定表の全ての行と「計数どうし」）と、要再確認の理由 1〜3 を持つ。

use crate::biz::inventory_service::check_stock_range;
use crate::biz::BizError;
use crate::db::inventory_repo::{self, LedgerMovementRow, StockCountRow};
use crate::db::sales_repo::{self, SettlementRow};
use crate::db::{DbConnection, DbError};

/// 要再確認の理由（ADR D3。wire の `RecountReason` と同じ順）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RecheckReason {
    /// 理由 1: 効く計数にとって決まらない精算に、その商品の Z004 の movement か連動の売上の行がある
    UndeterminedSettlement,
    /// 理由 2: 効く計数にとって決まらない入出庫の movement がある
    SameDateMovement,
    /// 理由 3: 効く計数の候補が重なり、値が違う
    ConflictingCounts,
}

/// 記録から求め直した在庫（写しの値・要再確認のときの最小と最大・理由）
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DerivedStock {
    pub stock: i64,
    pub range: Option<(i64, i64)>,
    pub recheck_reasons: Vec<RecheckReason>,
}

/// 求め直しの入力（1 商品の計数・有効な movement・active な精算の一覧・連動の売上を持つ取込み）
#[derive(Debug, Clone, Default)]
pub(crate) struct Ledger {
    pub counts: Vec<StockCountRow>,
    pub movements: Vec<LedgerMovementRow>,
    /// active な Z004 の取込みの精算（settlement_no の昇順）
    pub settlements: Vec<SettlementRow>,
    /// 取込みのときに在庫連動だった有効な売上（個数 ≠ 0 か金額 ≠ 0）を持つ active な取込みの ID
    pub linked_sale_imports: Vec<i64>,
}

fn overflow() -> BizError {
    BizError::ValidationFailed("在庫数計算オーバーフロー".to_string())
}

fn add(a: i64, b: i64) -> Result<i64, BizError> {
    a.checked_add(b).ok_or_else(overflow)
}

/// 計数 `a` の窓が `b` の窓より完全に後か（ADR D3 の「計数どうし」）
pub(crate) fn is_after(a: &StockCountRow, b: &StockCountRow) -> bool {
    match (a.fresh, b.fresh) {
        // fresh はそれより先に保存された全ての計数より後。後に保存された fresh には負ける
        (true, true) => a.id > b.id,
        // fresh の後に保存された既定の計数（日付 D、fresh の今日 T）: D < T なら fresh が後
        (true, false) => b.id < a.id || b.business_date < a.business_date,
        (false, true) => b.id < a.id && a.business_date > b.business_date,
        // 既定どうしは日付で決め、同じ日付は重なる
        (false, false) => a.business_date > b.business_date,
    }
}

/// 記録が計数に入っているか（ADR D3 の判定表）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Window {
    Included,
    Undetermined,
    NotIncluded,
}

/// 取込みのカーソル以下の取込みID を持つ active な取込みの最大の精算回数（ADR D3 の判定の n と、
/// D6 の完了の時点の精算の n）。カーソルが NULL か、その中に active な取込みが無ければ None。
/// 取込みID は使い回されないので、取消の後に同じ番号を使う取込みを数えない
pub(crate) fn settlement_no_within(
    settlements: &[SettlementRow],
    import_cursor: Option<i64>,
) -> Option<i64> {
    let cursor = import_cursor?;
    settlements
        .iter()
        .filter(|s| s.import_id <= cursor)
        .map(|s| s.settlement_no)
        .max()
}

/// fresh な計数の判定の n（ADR D3）。保存した `settlement_no`（D1 の鍵の n）は使わない
pub(crate) fn usable_cursor(count: &StockCountRow, settlements: &[SettlementRow]) -> Option<i64> {
    if !count.fresh {
        return None;
    }
    settlement_no_within(settlements, count.seen_import_id)
}

fn by_date(date: &str, count_date: &str) -> std::cmp::Ordering {
    date.cmp(count_date)
}

/// 計数に対する精算 k（精算日 d_k）の判定（ADR D3 の「精算 k」の行と A1）。
/// 「D に精算があるか」「最初の 1 本」は取り込み済み（active）の精算で決める
pub(crate) fn judge_settlement(
    count: &StockCountRow,
    settlement_no: i64,
    settlement_date: &str,
    settlements: &[SettlementRow],
) -> Window {
    let n = usable_cursor(count, settlements);
    if n.is_some_and(|n| settlement_no <= n) {
        return Window::Included;
    }
    let date = count.business_date.as_str();
    match by_date(settlement_date, date) {
        std::cmp::Ordering::Less => Window::Included,
        std::cmp::Ordering::Equal => Window::Undetermined,
        std::cmp::Ordering::Greater => {
            if settlements.iter().any(|s| s.settlement_date == date) {
                return Window::NotIncluded;
            }
            let first = settlements
                .iter()
                .filter(|s| {
                    n.is_none_or(|n| s.settlement_no > n) && s.settlement_date.as_str() > date
                })
                .map(|s| s.settlement_no)
                .min();
            if first == Some(settlement_no) {
                Window::Undetermined
            } else {
                Window::NotIncluded
            }
        }
    }
}

/// 計数に対する movement の判定（Z004 の movement は精算 k として判定する）
fn judge(count: &StockCountRow, movement: &LedgerMovementRow, ledger: &Ledger) -> Window {
    // 初期在庫は窓を持たない申告で、どの計数よりも前
    if movement.initial_stock {
        return Window::Included;
    }
    if let Some(k) = movement.settlement_no {
        return judge_settlement(count, k, &movement.business_date, &ledger.settlements);
    }
    if count.fresh && movement.id <= count.seen_movement_id.unwrap_or(0) {
        return Window::Included;
    }
    match by_date(&movement.business_date, &count.business_date) {
        std::cmp::Ordering::Less => Window::Included,
        std::cmp::Ordering::Equal => Window::Undetermined,
        std::cmp::Ordering::Greater => Window::NotIncluded,
    }
}

/// 1 つの計数を土台にした値（写し・最小・最大）と理由 1・2 の有無
struct FromCount {
    stock: i64,
    low: i64,
    high: i64,
    undetermined_settlement: bool,
    undetermined_movement: bool,
}

fn from_count(count: &StockCountRow, ledger: &Ledger) -> Result<FromCount, BizError> {
    let mut base = count.count_value;
    let (mut positive, mut negative) = (0_i64, 0_i64);
    let (mut undetermined_settlement, mut undetermined_movement) = (false, false);
    for movement in &ledger.movements {
        match judge(count, movement, ledger) {
            Window::Included => {}
            Window::NotIncluded => base = add(base, movement.quantity)?,
            Window::Undetermined => {
                if movement.settlement_no.is_some() {
                    undetermined_settlement = true;
                } else {
                    undetermined_movement = true;
                }
                if movement.quantity >= 0 {
                    positive = add(positive, movement.quantity)?;
                } else {
                    negative = add(negative, movement.quantity)?;
                }
            }
        }
    }
    // 理由 1 の行: movement の無い連動の売上（数量 0 で金額 ≠ 0 ほか）も、決まらない精算なら要再確認
    for import_id in &ledger.linked_sale_imports {
        if let Some(settlement) = ledger
            .settlements
            .iter()
            .find(|s| s.import_id == *import_id)
        {
            if judge_settlement(
                count,
                settlement.settlement_no,
                &settlement.settlement_date,
                &ledger.settlements,
            ) == Window::Undetermined
            {
                undetermined_settlement = true;
            }
        }
    }
    let low = add(base, negative)?;
    let high = add(base, positive)?;
    Ok(FromCount {
        stock: add(low, positive)?,
        low,
        high,
        undetermined_settlement,
        undetermined_movement,
    })
}

/// 効く計数（ほかのどの計数よりも前でない計数）。値の小さい順、同じ値なら fresh を先にする。
/// 衝突が無ければ代表の 1 つだけ。計数があるのに決まらない（前後の循環）は誤り
pub(crate) fn effective_counts(
    counts: &[StockCountRow],
) -> Result<(Vec<&StockCountRow>, bool), BizError> {
    let mut effective: Vec<&StockCountRow> = counts
        .iter()
        .filter(|count| !counts.iter().any(|other| is_after(other, count)))
        .collect();
    // 計数があるのに効く計数が無いのは前後が循環するとき（既定の日付 ≤ 保存の日 ≤ fresh の今日なので
    // 保存の経路からは起きない）。和へ黙って倒さずに止める
    if effective.is_empty() && !counts.is_empty() {
        return Err(BizError::DatabaseError(DbError::QueryFailed(
            "計数の前後を決められません".to_string(),
        )));
    }
    effective.sort_by_key(|count| (count.count_value, !count.fresh, count.id));
    let conflicting = effective
        .iter()
        .any(|count| count.count_value != effective[0].count_value);
    if !conflicting {
        effective.truncate(1);
    }
    Ok((effective, conflicting))
}

/// 入力から在庫を求める（読取りだけ。完了の時点の在庫は入力を絞って呼ぶ。35）
pub(crate) fn derive_ledger(ledger: &Ledger) -> Result<DerivedStock, BizError> {
    let (effective, conflicting) = effective_counts(&ledger.counts)?;
    if effective.is_empty() {
        // 効く計数が無ければ、精算を含む全ての有効な movement の和
        let mut stock = 0_i64;
        for movement in &ledger.movements {
            stock = add(stock, movement.quantity)?;
        }
        return Ok(DerivedStock {
            stock,
            range: None,
            recheck_reasons: Vec::new(),
        });
    }
    let mut result: Option<(i64, i64, i64)> = None;
    let mut reasons = Vec::new();
    for count in &effective {
        let part = from_count(count, ledger)?;
        if part.undetermined_settlement {
            reasons.push(RecheckReason::UndeterminedSettlement);
        }
        if part.undetermined_movement {
            reasons.push(RecheckReason::SameDateMovement);
        }
        result = Some(match result {
            None => (part.stock, part.low, part.high),
            Some((first, lo, hi)) => (first, lo.min(part.low), hi.max(part.high)),
        });
    }
    if conflicting {
        reasons.push(RecheckReason::ConflictingCounts);
    }
    reasons.sort();
    reasons.dedup();
    let (stock, low, high) = result.ok_or_else(overflow)?;
    Ok(DerivedStock {
        stock,
        range: (!reasons.is_empty()).then_some((low, high)),
        recheck_reasons: reasons,
    })
}

/// 効く計数の代表（衝突のときは写しの土台の計数）と差異（代表の値 − その計数に入っている記録だけから
/// 求めた帳簿。35「読取り」）。計数が無ければ None
pub(crate) fn representative_count(
    ledger: &Ledger,
) -> Result<Option<(StockCountRow, i64)>, BizError> {
    let (effective, _) = effective_counts(&ledger.counts)?;
    let Some(rep) = effective.first().map(|count| (*count).clone()) else {
        return Ok(None);
    };
    let book = derive_ledger(&Ledger {
        counts: ledger
            .counts
            .iter()
            .filter(|count| is_after(&rep, count))
            .cloned()
            .collect(),
        movements: ledger
            .movements
            .iter()
            .filter(|movement| judge(&rep, movement, ledger) == Window::Included)
            .cloned()
            .collect(),
        settlements: ledger.settlements.clone(),
        linked_sale_imports: Vec::new(),
    })?
    .stock;
    let difference = rep.count_value.checked_sub(book).ok_or_else(overflow)?;
    Ok(Some((rep, difference)))
}

/// 1 商品の求め直しの入力を読む（21・24）
pub(crate) fn read_ledger(conn: &DbConnection, product_code: &str) -> Result<Ledger, BizError> {
    Ok(Ledger {
        counts: inventory_repo::list_stock_counts_for_product(conn, product_code)?,
        movements: inventory_repo::list_ledger_movements_for_product(conn, product_code)?,
        settlements: sales_repo::list_active_settlements(conn)?,
        linked_sale_imports: sales_repo::list_sale_import_ids(conn, product_code, true)?,
    })
}

/// 在庫連動を始めてよいか（ADR SPEC-STK-CNT-D8、30）: 連動なしで取り込んだ Z004 の売上が全て、効く計数に
/// 「入る」ときだけ true。「入らない」「決まらない」売上・効く計数の衝突・効く計数が無く売上があるときは false
pub(crate) fn unlinked_sales_counted(
    conn: &DbConnection,
    product_code: &str,
) -> Result<bool, BizError> {
    let imports = sales_repo::list_sale_import_ids(conn, product_code, false)?;
    if imports.is_empty() {
        return Ok(true);
    }
    let ledger = read_ledger(conn, product_code)?;
    let (effective, conflicting) = effective_counts(&ledger.counts)?;
    let Some(count) = effective.first().filter(|_| !conflicting) else {
        return Ok(false);
    };
    Ok(imports.iter().all(|import_id| {
        ledger
            .settlements
            .iter()
            .find(|s| s.import_id == *import_id)
            .is_some_and(|s| {
                judge_settlement(
                    count,
                    s.settlement_no,
                    &s.settlement_date,
                    &ledger.settlements,
                ) == Window::Included
            })
    }))
}

/// 記録から在庫を求める（読取りだけ。整合性の検査が使う。31）
pub(crate) fn derive_stock(
    conn: &DbConnection,
    product_code: &str,
) -> Result<DerivedStock, BizError> {
    derive_ledger(&read_ledger(conn, product_code)?)
}

/// `derive_stock` と同じ規則で求め、写しの 4 項目を同じ TX で書く。
/// 上限の検査は書く直前のここだけ（求めた値と範囲の両端。Y2・Y3）。外れたら呼出し元の TX 全体が戻る
pub(crate) fn recompute_stock(
    tx: &rusqlite::Transaction<'_>,
    product_code: &str,
) -> Result<DerivedStock, BizError> {
    let derived = derive_stock(tx, product_code)?;
    check_stock_range(derived.stock)?;
    if let Some((low, high)) = derived.range {
        check_stock_range(low)?;
        check_stock_range(high)?;
    }
    if !inventory_repo::update_stock_quantity(tx, product_code, derived.stock, derived.range)? {
        return Err(BizError::NotFound(format!(
            "商品が見つかりません: {product_code}"
        )));
    }
    Ok(derived)
}

/// 精算の一覧が変わる TX（Z004 の確定・取消）の求め直し: 計数を持つ全ての商品と、
/// その取込みの売上か movement を持つ全ての商品（在庫連動の on / off を問わない。ADR の横断の表）。
/// 求め直した商品コードと結果を商品コードの順で返す
pub(crate) fn recompute_for_settlement_change(
    tx: &rusqlite::Transaction<'_>,
    import_id: i64,
) -> Result<Vec<(String, DerivedStock)>, BizError> {
    let mut targets = inventory_repo::list_counted_product_codes(tx)?;
    targets.extend(sales_repo::list_import_product_codes(tx, import_id)?);
    targets.sort();
    targets.dedup();
    // ponytail: 商品ごとに精算の一覧を読み直す（O(商品 × 取込み)）。数千商品で遅ければ一覧を 1 回だけ読む
    targets
        .into_iter()
        .map(|code| recompute_stock(tx, &code).map(|derived| (code, derived)))
        .collect()
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::db::DbConnection;

    /// 計数を直に入れる（(a) には保存の経路が無い。test だけ）。fresh なら `seen` を持つ
    pub(crate) fn insert_count(
        conn: &DbConnection,
        product_code: &str,
        value: i64,
        date: &str,
        fresh_seen: Option<i64>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                 seen_movement_id, counted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '2026-04-01T10:00:00')",
            rusqlite::params![
                product_code,
                value,
                if fresh_seen.is_some() {
                    "fresh"
                } else {
                    "default"
                },
                date,
                fresh_seen
            ],
        )
        .unwrap();
        conn.last_insert_rowid()
    }
}

#[cfg(test)]
pub(crate) mod settlement_support {
    use crate::db::DbConnection;

    /// active な Z004 の取込み（レジ 0001）を直に入れ、import ID を返す（test だけ）
    pub(crate) fn settlement(conn: &DbConnection, settlement_no: i64, date: &str) -> i64 {
        conn.execute(
            "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount,
                 skipped_count, status, imported_at, machine_no, settlement_no)
             VALUES ('Z004.CSV', ?1, 'hash-' || ?2 || '-' || abs(random()), 0, 0, 0, 'completed',
                     ?1 || 'T20:00:00', '0001', ?2)",
            rusqlite::params![date, settlement_no],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// その取込みの Z004 の movement（phase 1）
    pub(crate) fn z004_movement(
        conn: &DbConnection,
        code: &str,
        quantity: i64,
        import_id: i64,
    ) -> i64 {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 reference_type, reference_id, created_at, business_date, phase, settlement_no)
             SELECT ?1, 'sale_auto', ?2, 0, 'csv_import', id, settlement_date || 'T20:00:00',
                    settlement_date, 1, settlement_no
             FROM csv_imports WHERE id = ?3",
            rusqlite::params![code, quantity, import_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// その取込みの売上の行（取込みのときの連動 `linked`）
    pub(crate) fn z004_sale(
        conn: &DbConnection,
        code: &str,
        import_id: i64,
        quantity: i64,
        amount: i64,
        linked: bool,
    ) {
        conn.execute(
            "INSERT INTO sale_records (csv_import_id, product_code, sale_date, quantity, amount,
                 source, created_at, stock_linked)
             SELECT id, ?1, settlement_date, ?2, ?3, 'auto', settlement_date || 'T20:00:00', ?4
             FROM csv_imports WHERE id = ?5",
            rusqlite::params![code, quantity, amount, linked, import_id],
        )
        .unwrap();
    }

    /// 取込みを取り消した状態にする（売上と movement を無効に）
    pub(crate) fn roll_back(conn: &DbConnection, import_id: i64) {
        conn.execute_batch(&format!(
            "UPDATE csv_imports SET status = 'rolled_back' WHERE id = {import_id};
             UPDATE sale_records SET is_voided = 1 WHERE csv_import_id = {import_id};
             UPDATE inventory_movements SET is_voided = 1
                 WHERE reference_type = 'csv_import' AND reference_id = {import_id};"
        ))
        .unwrap();
    }

    /// fresh な計数を直に入れる。カーソルは保存と同じく今の DB から読む（active な最大の精算回数〈鍵の n〉、
    /// `MAX(inventory_movements.id)`、取り消したものを含む `MAX(csv_imports.id)`）
    pub(crate) fn insert_fresh_count(
        conn: &DbConnection,
        code: &str,
        value: i64,
        today: &str,
    ) -> i64 {
        conn.execute(
            "INSERT INTO stock_counts (product_code, count_value, count_kind, business_date,
                 settlement_no, seen_movement_id, seen_import_id, counted_at)
             VALUES (?1, ?2, 'fresh', ?3,
                     (SELECT MAX(settlement_no) FROM csv_imports
                      WHERE status IN ('completed','completed_partial')),
                     (SELECT COALESCE(MAX(id), 0) FROM inventory_movements),
                     (SELECT MAX(id) FROM csv_imports), ?3 || 'T10:00:00')",
            rusqlite::params![code, value, today],
        )
        .unwrap();
        conn.last_insert_rowid()
    }
}

#[cfg(test)]
mod tests {
    use super::settlement_support::*;
    use super::test_support::insert_count;
    use super::*;
    use crate::db::test_support::{seed_product, setup_test_db};

    const D: &str = "2026-05-10";

    /// 業務日 `date` の入出庫の movement（初期在庫なら reference_type NULL）を入れ、id を返す
    fn movement(conn: &DbConnection, code: &str, quantity: i64, date: &str) -> i64 {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 reference_type, reference_id, created_at, business_date, phase)
             VALUES (?1, 'receiving', ?2, 0, 'receiving_record', 1, '2026-05-20T10:00:00', ?3, 0)",
            rusqlite::params![code, quantity, date],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn initial(conn: &DbConnection, code: &str, quantity: i64, date: &str) {
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 created_at, business_date, phase)
             VALUES (?1, 'receiving', ?2, ?2, '2026-05-20T10:00:00', ?3, 0)",
            rusqlite::params![code, quantity, date],
        )
        .unwrap();
    }

    fn derived(stock: i64, range: Option<(i64, i64)>, reasons: &[RecheckReason]) -> DerivedStock {
        DerivedStock {
            stock,
            range,
            recheck_reasons: reasons.to_vec(),
        }
    }

    fn copy_of(conn: &DbConnection, code: &str) -> (i64, bool, Option<i64>, Option<i64>) {
        conn.query_row(
            "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
             FROM products WHERE product_code = ?1",
            [code],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap()
    }

    #[test]
    fn test_stock_ledger_req201_no_counts_equals_sum_of_valid_movements() {
        // REQ-201 / SPEC-STK-CNT-D2（効く計数が無ければ全ての有効な movement の和）/ Matrix L1・T6
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L1");
        seed_product(&conn, "L1-EMPTY");
        initial(&conn, "L1", 6, "2026-05-01");
        movement(&conn, "L1", -2, "2026-05-02");
        movement(&conn, "L1", 9, "2026-05-03");
        let voided = movement(&conn, "L1", 40, "2026-05-04");
        conn.execute(
            "UPDATE inventory_movements SET is_voided = 1 WHERE id = ?1",
            [voided],
        )
        .unwrap();
        assert_eq!(derive_stock(&conn, "L1").unwrap(), derived(13, None, &[]));
        // movement なし → 0
        assert_eq!(
            derive_stock(&conn, "L1-EMPTY").unwrap(),
            derived(0, None, &[])
        );
    }

    #[test]
    fn test_stock_ledger_req205_default_count_vs_movement_dates() {
        // REQ-205 / SPEC-STK-CNT-D3（既定の計数 × 入出庫: E < D 入る、E = D 決まらない、E > D 入らない）/ Matrix L2
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L2");
        movement(&conn, "L2", 3, "2026-05-09");
        movement(&conn, "L2", -2, "2026-05-11");
        insert_count(&conn, "L2", 10, D, None);
        movement(&conn, "L2", 4, D);
        assert_eq!(
            derive_stock(&conn, "L2").unwrap(),
            derived(12, Some((8, 12)), &[RecheckReason::SameDateMovement])
        );
    }

    #[test]
    fn test_stock_ledger_req205_fresh_count_cursor_boundary() {
        // REQ-205 / SPEC-STK-CNT-D3（fresh の計数 × 入出庫: i ≤ m 入る、i > m は日付で）/ Matrix L3
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L3");
        // id m（カーソル）の movement は日付が T でも入る
        let m = movement(&conn, "L3", 5, D);
        insert_count(&conn, "L3", 20, D, Some(m));
        assert_eq!(derive_stock(&conn, "L3").unwrap(), derived(20, None, &[]));
        // id m+1 で業務日 T−1 は入る
        movement(&conn, "L3", 7, "2026-05-09");
        assert_eq!(derive_stock(&conn, "L3").unwrap(), derived(20, None, &[]));
        // 業務日 T+1 は入らない（足す）
        movement(&conn, "L3", -3, "2026-05-11");
        assert_eq!(derive_stock(&conn, "L3").unwrap(), derived(17, None, &[]));
        // id m+1 以降で業務日 T は決まらない（写しで足し、範囲を持つ）
        movement(&conn, "L3", 2, D);
        assert_eq!(
            derive_stock(&conn, "L3").unwrap(),
            derived(19, Some((17, 19)), &[RecheckReason::SameDateMovement])
        );

        // カーソル 0（movement の無い時点の fresh）: どの movement も日付で決まる
        seed_product(&conn, "L3-0");
        insert_count(&conn, "L3-0", 4, D, Some(0));
        movement(&conn, "L3-0", 1, "2026-05-09");
        assert_eq!(derive_stock(&conn, "L3-0").unwrap(), derived(4, None, &[]));
        movement(&conn, "L3-0", 6, D);
        assert_eq!(
            derive_stock(&conn, "L3-0").unwrap(),
            derived(10, Some((4, 10)), &[RecheckReason::SameDateMovement])
        );
    }

    #[test]
    fn test_stock_ledger_req205_initial_stock_always_included() {
        // REQ-205 / SPEC-STK-CNT-D3（初期在庫はどの計数にも入る）/ Matrix L4
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L4");
        insert_count(&conn, "L4", 3, D, None);
        insert_count(&conn, "L4", 3, "2026-05-09", Some(0));
        // 計数の日付より後に登録した初期在庫
        initial(&conn, "L4", 8, "2026-05-15");
        assert_eq!(derive_stock(&conn, "L4").unwrap(), derived(3, None, &[]));
        seed_product(&conn, "L4-F");
        insert_count(&conn, "L4-F", 2, D, Some(0));
        initial(&conn, "L4-F", 8, D);
        assert_eq!(derive_stock(&conn, "L4-F").unwrap(), derived(2, None, &[]));
    }

    #[test]
    fn test_stock_ledger_req205_later_default_count_wins() {
        // REQ-205 / SPEC-STK-CNT-D3（既定どうしは日付で決める）/ Matrix L5
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L5");
        // 後の日付の計数を先に保存しても、日付の後の計数が効く
        insert_count(&conn, "L5", 9, "2026-05-12", None);
        insert_count(&conn, "L5", 30, "2026-05-08", None);
        assert_eq!(derive_stock(&conn, "L5").unwrap(), derived(9, None, &[]));
    }

    #[test]
    fn test_stock_ledger_req205_fresh_replaces_earlier_counts() {
        // REQ-205 / SPEC-STK-CNT-D3（fresh はそれより先に保存された全ての計数より後）/ Matrix L6
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L6");
        insert_count(&conn, "L6", 12, D, None);
        insert_count(&conn, "L6", 12, "2026-05-11", None);
        insert_count(&conn, "L6", 5, D, Some(0));
        assert_eq!(derive_stock(&conn, "L6").unwrap(), derived(5, None, &[]));
        // fresh どうしは保存の順
        insert_count(&conn, "L6", 6, D, Some(0));
        assert_eq!(derive_stock(&conn, "L6").unwrap(), derived(6, None, &[]));
    }

    #[test]
    fn test_stock_ledger_req205_default_after_fresh_three_cases() {
        // REQ-205 / SPEC-STK-CNT-D3（fresh の後に保存した既定の計数: D < T / D = T / D > T）/ Matrix L7
        let (_dir, conn) = setup_test_db();
        // D < T: fresh が後で、既定は効かない
        seed_product(&conn, "L7-LT");
        insert_count(&conn, "L7-LT", 5, D, Some(0));
        insert_count(&conn, "L7-LT", 9, "2026-05-09", None);
        assert_eq!(derive_stock(&conn, "L7-LT").unwrap(), derived(5, None, &[]));
        // D > T: 既定が後
        seed_product(&conn, "L7-GT");
        insert_count(&conn, "L7-GT", 5, D, Some(0));
        insert_count(&conn, "L7-GT", 9, "2026-05-11", None);
        assert_eq!(derive_stock(&conn, "L7-GT").unwrap(), derived(9, None, &[]));
        // D = T: 重なる。値が違えば衝突
        seed_product(&conn, "L7-EQ");
        insert_count(&conn, "L7-EQ", 5, D, Some(0));
        insert_count(&conn, "L7-EQ", 9, D, None);
        assert_eq!(
            derive_stock(&conn, "L7-EQ").unwrap(),
            derived(5, Some((5, 9)), &[RecheckReason::ConflictingCounts])
        );
        // D = T で同値: fresh を代表にし、その窓とカーソルで判定する（カーソル以前の同じ日付の記録は入る）
        seed_product(&conn, "L7-SAME");
        let m = movement(&conn, "L7-SAME", 4, D);
        insert_count(&conn, "L7-SAME", 7, D, Some(m));
        insert_count(&conn, "L7-SAME", 7, D, None);
        assert_eq!(
            derive_stock(&conn, "L7-SAME").unwrap(),
            derived(7, None, &[])
        );
    }

    #[test]
    fn test_stock_ledger_req205_cyclic_counts_are_error_not_sum() {
        // REQ-205 / SPEC-STK-CNT-D3（計数どうしの前後が循環する DB の直の編集は、和へ倒さず誤りで止める）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "LC");
        movement(&conn, "LC", 4, "2026-05-01");
        insert_count(&conn, "LC", 1, "2026-05-12", None); // C0: fresh より先に保存（F が後）
        insert_count(&conn, "LC", 2, D, Some(0)); // F（今日 D）
        insert_count(&conn, "LC", 3, "2026-05-11", None); // C1: F の後で D より後（F より後）、C0 より前
        assert!(matches!(
            derive_stock(&conn, "LC"),
            Err(BizError::DatabaseError(DbError::QueryFailed(message))) if message == "計数の前後を決められません"
        ));
    }

    #[test]
    fn test_stock_ledger_req205_conflicting_counts_range_and_min_base() {
        // REQ-205 / SPEC-STK-CNT-D2・D3 の理由 3（衝突は各土台の範囲を合わせ、最小の値を写しの土台）/ Matrix L8
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L8");
        insert_count(&conn, "L8", 11, D, None);
        insert_count(&conn, "L8", 8, D, None);
        movement(&conn, "L8", -1, "2026-05-12");
        assert_eq!(
            derive_stock(&conn, "L8").unwrap(),
            derived(7, Some((7, 10)), &[RecheckReason::ConflictingCounts])
        );
    }

    #[test]
    fn test_stock_ledger_req205_equal_overlapping_counts_not_conflict() {
        // REQ-205 / SPEC-STK-CNT-D3（重なる同値は 1 つとみなす）/ Matrix L9
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L9");
        insert_count(&conn, "L9", 9, D, None);
        insert_count(&conn, "L9", 9, D, None);
        assert_eq!(derive_stock(&conn, "L9").unwrap(), derived(9, None, &[]));
    }

    #[test]
    fn test_stock_ledger_req205_range_with_positive_and_negative_undetermined() {
        // REQ-205 / SPEC-STK-CNT-D2（範囲は決まらない記録を適用する・しないの最小と最大）/ Matrix L10
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L10");
        insert_count(&conn, "L10", 10, D, None);
        movement(&conn, "L10", 3, D);
        movement(&conn, "L10", -2, D);
        assert_eq!(
            derive_stock(&conn, "L10").unwrap(),
            derived(11, Some((8, 13)), &[RecheckReason::SameDateMovement])
        );
    }

    #[test]
    fn test_stock_ledger_req205_reasons_dedup_and_order() {
        // REQ-205 / SPEC-STK-CNT-D3（理由は重複なし、same_date_movement → conflicting_counts の順）/ Matrix L11
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "L11");
        insert_count(&conn, "L11", 6, D, None);
        insert_count(&conn, "L11", 4, D, None);
        movement(&conn, "L11", 1, D);
        movement(&conn, "L11", 2, D);
        assert_eq!(
            derive_stock(&conn, "L11").unwrap(),
            derived(
                7,
                Some((4, 9)),
                &[
                    RecheckReason::SameDateMovement,
                    RecheckReason::ConflictingCounts
                ]
            )
        );
    }

    #[test]
    fn test_stock_ledger_req903_recompute_rejects_out_of_range_value_and_range_end() {
        // REQ-903 / Y2・Y3（上限は写しを書く直前、値と範囲の両端）/ Matrix L13
        let (_dir, mut conn) = setup_test_db();
        let expect_rejected = |conn: &mut DbConnection, code: &str| {
            let tx = conn.transaction().unwrap();
            match recompute_stock(&tx, code) {
                Err(BizError::ValidationFailed(message)) => {
                    assert_eq!(message, "在庫数が大きすぎます", "{code}")
                }
                other => panic!("{code}: ValidationFailed を期待: {other:?}"),
            }
            drop(tx);
            assert_eq!(copy_of(conn, code), (0, false, None, None), "{code}");
        };
        seed_product(&conn, "L13-V");
        initial(&conn, "L13-V", 999_999_999, "2026-05-01");
        movement(&conn, "L13-V", 1, "2026-05-02");
        expect_rejected(&mut conn, "L13-V");
        // 範囲の上の端だけが外れる（写しは上限の中）
        seed_product(&conn, "L13-R");
        insert_count(&conn, "L13-R", 0, D, None);
        movement(&conn, "L13-R", 1_000_000_000, D);
        movement(&conn, "L13-R", -1, D);
        assert_eq!(
            derive_stock(&conn, "L13-R").unwrap(),
            derived(
                999_999_999,
                Some((-1, 1_000_000_000)),
                &[RecheckReason::SameDateMovement]
            )
        );
        expect_rejected(&mut conn, "L13-R");
        // 範囲の下の端だけが外れる
        seed_product(&conn, "L13-LOW");
        insert_count(&conn, "L13-LOW", 0, D, None);
        movement(&conn, "L13-LOW", -1_000_000_000, D);
        movement(&conn, "L13-LOW", 1, D);
        assert_eq!(
            derive_stock(&conn, "L13-LOW").unwrap(),
            derived(
                -999_999_999,
                Some((-1_000_000_000, 1)),
                &[RecheckReason::SameDateMovement]
            )
        );
        expect_rejected(&mut conn, "L13-LOW");
        // 端の 999999999・−999999999 は通る
        seed_product(&conn, "L13-OK");
        insert_count(&conn, "L13-OK", 0, D, None);
        movement(&conn, "L13-OK", -999_999_999, D);
        movement(&conn, "L13-OK", 999_999_999, D);
        let tx = conn.transaction().unwrap();
        recompute_stock(&tx, "L13-OK").unwrap();
        tx.commit().unwrap();
        assert_eq!(
            copy_of(&conn, "L13-OK"),
            (0, true, Some(-999_999_999), Some(999_999_999))
        );
        // i64 の和の溢れも書かずに止まる
        seed_product(&conn, "L13-OVF");
        initial(&conn, "L13-OVF", i64::MAX, "2026-05-01");
        initial(&conn, "L13-OVF", 1, "2026-05-01");
        assert!(matches!(
            derive_stock(&conn, "L13-OVF"),
            Err(BizError::ValidationFailed(message)) if message == "在庫数計算オーバーフロー"
        ));
    }

    #[test]
    fn test_stock_ledger_req205_recompute_writes_quantity_flag_and_range() {
        // REQ-205 / SPEC-STK-CNT-D2（recompute_stock が写しの 4 項目を書く）/ Matrix L14
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "L14");
        insert_count(&conn, "L14", 8, D, None);
        insert_count(&conn, "L14", 11, D, None);
        let tx = conn.transaction().unwrap();
        let result = recompute_stock(&tx, "L14").unwrap();
        tx.commit().unwrap();
        assert_eq!(
            result,
            derived(8, Some((8, 11)), &[RecheckReason::ConflictingCounts])
        );
        assert_eq!(copy_of(&conn, "L14"), (8, true, Some(8), Some(11)));
        // 「入れ直す」で解けたら 0・NULL に戻る
        insert_count(&conn, "L14", 10, D, Some(0));
        let tx = conn.transaction().unwrap();
        recompute_stock(&tx, "L14").unwrap();
        tx.commit().unwrap();
        assert_eq!(copy_of(&conn, "L14"), (10, false, None, None));
        // 対象の無い商品は NotFound
        let tx = conn.transaction().unwrap();
        assert!(matches!(
            recompute_stock(&tx, "L14-NONE"),
            Err(BizError::NotFound(_))
        ));
    }

    // ---- b1: 精算の窓（Matrix の L。ADR D3 の「精算 k」の行・A1・理由 1） ----

    const US: RecheckReason = RecheckReason::UndeterminedSettlement;

    #[test]
    fn test_stock_ledger_req205_default_count_settlement_window() {
        // REQ-205 / SPEC-STK-CNT-D3（既定の計数 × 精算: d_k < D 入る、d_k = D は全て決まらない、
        // d_k > D は D に精算があれば入らない〈A1〉）/ b1 Matrix L1
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL1");
        let s1 = settlement(&conn, 1, "2026-05-09");
        let s2 = settlement(&conn, 2, D);
        let s3 = settlement(&conn, 3, D);
        let s4 = settlement(&conn, 4, "2026-05-11");
        z004_movement(&conn, "SL1", -1, s1);
        z004_movement(&conn, "SL1", -2, s2);
        z004_movement(&conn, "SL1", -4, s3);
        z004_movement(&conn, "SL1", -8, s4);
        insert_count(&conn, "SL1", 10, D, None);
        // 入らない 4 を足した 2 が土台、決まらない 2・3（−6）を適用した −4 が写し
        assert_eq!(
            derive_stock(&conn, "SL1").unwrap(),
            derived(-4, Some((-4, 2)), &[US])
        );
    }

    #[test]
    fn test_stock_ledger_req205_default_count_first_settlement_after_quiet_day() {
        // REQ-205 / SPEC-STK-CNT-D3（D に精算が無いとき、d_k > D の最初の 1 本だけ決まらない）/ b1 Matrix L2
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL2");
        settlement(&conn, 1, "2026-05-09");
        let s2 = settlement(&conn, 2, "2026-05-12");
        let s3 = settlement(&conn, 3, "2026-05-13");
        z004_movement(&conn, "SL2", -2, s2);
        z004_movement(&conn, "SL2", -4, s3);
        insert_count(&conn, "SL2", 10, D, None);
        assert_eq!(
            derive_stock(&conn, "SL2").unwrap(),
            derived(4, Some((4, 6)), &[US])
        );
    }

    #[test]
    fn test_stock_ledger_req205_fresh_count_settlement_cursor() {
        // REQ-205 / SPEC-STK-CNT-D3（fresh: k ≤ n 入る。k > n は d_k < T 入る、= T 決まらない、
        // > T は T に精算があれば入らない、無ければ最初の 1 本だけ決まらない）/ b1 Matrix L3
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL3");
        let s1 = settlement(&conn, 1, "2026-05-09");
        let s2 = settlement(&conn, 2, D);
        z004_movement(&conn, "SL3", -1, s1);
        z004_movement(&conn, "SL3", -2, s2);
        insert_fresh_count(&conn, "SL3", 10, D);
        let s3 = settlement(&conn, 3, D);
        let s4 = settlement(&conn, 4, "2026-05-11");
        z004_movement(&conn, "SL3", -4, s3);
        z004_movement(&conn, "SL3", -8, s4);
        assert_eq!(
            derive_stock(&conn, "SL3").unwrap(),
            derived(-2, Some((-2, 2)), &[US])
        );

        // T に精算が無い: n より後で T より後の最初の 1 本だけ決まらない
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL3B");
        let s1 = settlement(&conn, 1, "2026-05-09");
        z004_movement(&conn, "SL3B", -1, s1);
        insert_fresh_count(&conn, "SL3B", 10, D);
        let s2 = settlement(&conn, 2, "2026-05-12");
        let s3 = settlement(&conn, 3, "2026-05-13");
        z004_movement(&conn, "SL3B", -2, s2);
        z004_movement(&conn, "SL3B", -4, s3);
        assert_eq!(
            derive_stock(&conn, "SL3B").unwrap(),
            derived(4, Some((4, 6)), &[US])
        );
    }

    #[test]
    fn test_stock_ledger_req205_fresh_count_without_imports_uses_dates() {
        // REQ-205 / SPEC-STK-CNT-D3（n が NULL なら全ての精算を日付で判定。精算 0 は入らない）/ b1 Matrix L4
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL4");
        insert_fresh_count(&conn, "SL4", 10, D);
        let s0 = settlement(&conn, 0, "2026-05-11");
        z004_movement(&conn, "SL4", -3, s0);
        assert_eq!(
            derive_stock(&conn, "SL4").unwrap(),
            derived(7, Some((7, 10)), &[US])
        );
    }

    #[test]
    fn test_stock_ledger_req205_undetermined_settlement_movement_recheck() {
        // REQ-205 / SPEC-STK-CNT-D3 の理由 1（決まらない精算にその商品の movement）、範囲の両端 / b1 Matrix L5
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL5");
        seed_product(&conn, "SL5-OTHER");
        let s1 = settlement(&conn, 1, D);
        z004_movement(&conn, "SL5", -2, s1);
        z004_movement(&conn, "SL5", 3, s1);
        z004_movement(&conn, "SL5-OTHER", -9, s1);
        insert_count(&conn, "SL5", 10, D, None);
        assert_eq!(
            derive_stock(&conn, "SL5").unwrap(),
            derived(11, Some((8, 13)), &[US])
        );
    }

    #[test]
    fn test_stock_ledger_req205_undetermined_settlement_uses_stock_linked_at_import() {
        // REQ-205 / SPEC-STK-CNT-D3 の理由 1（movement の無い行: 取込みのときの stock_linked = 1 で
        // 個数 ≠ 0 か金額 ≠ 0。今の pos_stock_sync を見ない）/ b1 Matrix L6
        let (_dir, conn) = setup_test_db();
        for code in ["SL6-A", "SL6-B", "SL6-C"] {
            seed_product(&conn, code);
            insert_count(&conn, code, 10, D, None);
        }
        let s1 = settlement(&conn, 1, D);
        // A: 連動だった金額だけの行。今は連動を外している
        z004_sale(&conn, "SL6-A", s1, 0, 500, true);
        conn.execute(
            "UPDATE products SET pos_stock_sync = 0 WHERE product_code = 'SL6-A'",
            [],
        )
        .unwrap();
        // B: 連動しなかった行（今は連動）。C: 連動だが個数・金額とも 0
        z004_sale(&conn, "SL6-B", s1, 1, 100, false);
        z004_sale(&conn, "SL6-C", s1, 0, 0, true);
        assert_eq!(
            derive_stock(&conn, "SL6-A").unwrap(),
            derived(10, Some((10, 10)), &[US])
        );
        assert_eq!(
            derive_stock(&conn, "SL6-B").unwrap(),
            derived(10, None, &[])
        );
        assert_eq!(
            derive_stock(&conn, "SL6-C").unwrap(),
            derived(10, None, &[])
        );
    }

    #[test]
    fn test_stock_ledger_req205_rolled_back_import_excluded_from_settlements() {
        // REQ-205 / SPEC-STK-CNT-D3・D7（取り消した取込みは精算の一覧に入らない）/ b1 Matrix L7
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL7");
        let s1 = settlement(&conn, 1, D);
        z004_movement(&conn, "SL7", -1, s1);
        roll_back(&conn, s1);
        let s2 = settlement(&conn, 2, "2026-05-11");
        z004_movement(&conn, "SL7", -3, s2);
        insert_count(&conn, "SL7", 10, D, None);
        // D に active な精算が無いので、2 が最初の 1 本で決まらない
        assert_eq!(
            derive_stock(&conn, "SL7").unwrap(),
            derived(7, Some((7, 10)), &[US])
        );
    }

    #[test]
    fn test_stock_ledger_req205_seed_sale_without_reference_is_windowed() {
        // REQ-205 / SPEC-STK-CNT-D9（初期在庫の印は reference_type NULL かつ receiving。
        // demo の seed の sale_auto は業務日で窓を持つ）/ b1 Matrix L8
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL8");
        insert_count(&conn, "SL8", 10, D, None);
        conn.execute(
            "INSERT INTO inventory_movements (product_code, movement_type, quantity, stock_after,
                 created_at, business_date, phase)
             VALUES ('SL8', 'sale_auto', -1, 0, '2026-05-11T10:00:00', '2026-05-11', 0)",
            [],
        )
        .unwrap();
        initial(&conn, "SL8", 4, "2026-05-11");
        assert_eq!(derive_stock(&conn, "SL8").unwrap(), derived(9, None, &[]));
    }

    #[test]
    fn test_stock_ledger_req205_sum_includes_settlement_movements_without_counts() {
        // REQ-205 / SPEC-STK-CNT-D2（計数の無い商品は phase 1 を含む全ての有効な movement の和。
        // (a) の誤りの止まりを外した）/ b1 Matrix L9
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "SL9");
        initial(&conn, "SL9", 5, "2026-05-01");
        let s1 = settlement(&conn, 1, D);
        z004_movement(&conn, "SL9", -2, s1);
        assert_eq!(derive_stock(&conn, "SL9").unwrap(), derived(3, None, &[]));
        let tx = conn.transaction().unwrap();
        recompute_stock(&tx, "SL9").unwrap();
        tx.commit().unwrap();
        assert_eq!(copy_of(&conn, "SL9"), (3, false, None, None));
    }

    #[test]
    fn test_stock_ledger_req205_recheck_reason_order_and_dedup() {
        // REQ-205 / SPEC-STK-CNT-D3（理由は undetermined_settlement・same_date_movement・
        // conflicting_counts の順で重複なし）/ b1 Matrix L10
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL10");
        insert_count(&conn, "SL10", 6, D, None);
        insert_count(&conn, "SL10", 4, D, None);
        movement(&conn, "SL10", 1, D);
        let s1 = settlement(&conn, 1, D);
        z004_movement(&conn, "SL10", -1, s1);
        z004_movement(&conn, "SL10", -1, s1);
        assert_eq!(
            derive_stock(&conn, "SL10").unwrap(),
            derived(
                3,
                Some((2, 7)),
                &[
                    US,
                    RecheckReason::SameDateMovement,
                    RecheckReason::ConflictingCounts
                ]
            )
        );
    }

    #[test]
    fn test_stock_ledger_req205_gap_errs_on_undetermined_side_and_resolves_after_backfill() {
        // REQ-205 / SPEC-STK-CNT-D3（「最初の 1 本」は取り込み済みの精算で決める。欠けは決まらない側へ倒れ、
        // 後追いの取込みで決まる）/ b1 Matrix L11
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL11");
        settlement(&conn, 1, "2026-05-09");
        let s3 = settlement(&conn, 3, "2026-05-12");
        z004_movement(&conn, "SL11", -2, s3);
        insert_count(&conn, "SL11", 10, D, None);
        assert_eq!(
            derive_stock(&conn, "SL11").unwrap(),
            derived(8, Some((8, 10)), &[US])
        );
        // 欠けた 2（05-11、この商品の行なし）を後追いで取り込むと、2 が最初の 1 本になり 3 は入らない
        settlement(&conn, 2, "2026-05-11");
        assert_eq!(derive_stock(&conn, "SL11").unwrap(), derived(8, None, &[]));
    }

    #[test]
    fn test_stock_ledger_req205_fresh_settlement_cursor_uses_import_ids() {
        // REQ-205 / SPEC-STK-CNT-D3・D7（fresh な計数の判定の n は、取込みのカーソル c 以下の取込みID を持つ
        // 今 active な取込みの最大の精算回数。Codex の closure の Gated Amendment）/ b1 Matrix L13
        const T: &str = D;
        // (a) Codex の反例: 精算 1（T−1）→ 入れ直す → 取消 → 新しい精算 1（T+1）
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL13A");
        let i1 = settlement(&conn, 1, "2026-05-09");
        insert_fresh_count(&conn, "SL13A", 5, T);
        roll_back(&conn, i1);
        let i2 = settlement(&conn, 1, "2026-05-11");
        z004_movement(&conn, "SL13A", -1, i2);
        // 判定の n は NULL（c 以下の active な取込みが無い）。i2 は T より後の最初の 1 本で決まらない
        assert_eq!(
            derive_stock(&conn, "SL13A").unwrap(),
            derived(4, Some((4, 5)), &[US])
        );

        // (b) 取消と取り込み直し。精算回数（99・100）と取込みID（1〜3）を離しておく
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL13B");
        let i1 = settlement(&conn, 99, T);
        z004_movement(&conn, "SL13B", -1, i1);
        let i2 = settlement(&conn, 100, T);
        insert_fresh_count(&conn, "SL13B", 5, T);
        // 対照: 取消の前は 99・100 とも入る
        assert_eq!(derive_stock(&conn, "SL13B").unwrap(), derived(5, None, &[]));
        // 100 を取り消すと判定の n は c 以下に残る 99 で、99 は入ったまま
        roll_back(&conn, i2);
        assert_eq!(derive_stock(&conn, "SL13B").unwrap(), derived(5, None, &[]));
        // 正しい版の 100 は保存の後の取込みなので判定の n にならず、精算日 T で決まらない（過剰側。TD-400）
        let i3 = settlement(&conn, 100, T);
        z004_movement(&conn, "SL13B", -1, i3);
        assert_eq!((i1, i2, i3), (1, 2, 3));
        assert_eq!(
            derive_stock(&conn, "SL13B").unwrap(),
            derived(4, Some((4, 5)), &[US])
        );
        // 入れ直すと解ける
        insert_fresh_count(&conn, "SL13B", 5, T);
        assert_eq!(derive_stock(&conn, "SL13B").unwrap(), derived(5, None, &[]));

        // (c) 判定の n でない精算の取り込み直し（Amendment の review の Codex の反例。Gated Amendment の 2 本目）:
        // 取り込み直した 99 は判定の n の算出に入らないが、n = 100 のままなので k ≤ n で入り、要再確認にならない
        let (_dir, conn) = setup_test_db();
        seed_product(&conn, "SL13C");
        let i1 = settlement(&conn, 99, T);
        z004_movement(&conn, "SL13C", -1, i1);
        let i2 = settlement(&conn, 100, T);
        insert_fresh_count(&conn, "SL13C", 5, T);
        assert_eq!(derive_stock(&conn, "SL13C").unwrap(), derived(5, None, &[]));
        roll_back(&conn, i1);
        assert_eq!(derive_stock(&conn, "SL13C").unwrap(), derived(5, None, &[]));
        let i3 = settlement(&conn, 99, T);
        z004_movement(&conn, "SL13C", -1, i3);
        assert_eq!((i1, i2, i3), (1, 2, 3));
        assert_eq!(derive_stock(&conn, "SL13C").unwrap(), derived(5, None, &[]));
    }

    #[test]
    fn test_stock_ledger_req205_req401_recompute_targets_for_settlement_change() {
        // REQ-205 / REQ-401 / ADR SPEC-STK-CNT-D2 の横断の表（Z004 の確定・取消の求め直し）/ b1 Matrix L12:
        // 求め直す商品 = 計数を持つ全ての商品 + その取込みの売上か movement を持つ全ての商品（連動の on / off を問わない）。
        // 写しをわざと記録から食い違わせ、求め直しの後に derive_stock と一致することで観測する
        let (_dir, mut conn) = setup_test_db();
        seed_product(&conn, "SL12-COUNTED");
        seed_product(&conn, "SL12-UNLINKED");
        seed_product(&conn, "SL12-OTHER");
        insert_count(&conn, "SL12-COUNTED", 5, "2026-04-01", None);
        let import_id = settlement(&conn, 1, "2026-04-02");
        z004_sale(&conn, "SL12-UNLINKED", import_id, 1, 100, false);
        let desync = |conn: &DbConnection| {
            conn.execute(
                "UPDATE products SET stock_quantity = 77 WHERE product_code LIKE 'SL12-%'",
                [],
            )
            .unwrap();
        };
        let copy = |conn: &DbConnection, code: &str| -> i64 {
            conn.query_row(
                "SELECT stock_quantity FROM products WHERE product_code = ?1",
                [code],
                |row| row.get(0),
            )
            .unwrap()
        };
        for rolled_back in [false, true] {
            if rolled_back {
                roll_back(&conn, import_id);
            }
            desync(&conn);
            let tx = conn.transaction().unwrap();
            let codes: Vec<String> = recompute_for_settlement_change(&tx, import_id)
                .unwrap()
                .into_iter()
                .map(|(code, _)| code)
                .collect();
            tx.commit().unwrap();
            assert_eq!(codes, vec!["SL12-COUNTED", "SL12-UNLINKED"]);
            for code in ["SL12-COUNTED", "SL12-UNLINKED"] {
                assert_eq!(
                    copy(&conn, code),
                    derive_stock(&conn, code).unwrap().stock,
                    "{code}"
                );
            }
            // 計数も取込みの行も持たない商品は求め直さない
            assert_eq!(copy(&conn, "SL12-OTHER"), 77);
        }
    }
}
