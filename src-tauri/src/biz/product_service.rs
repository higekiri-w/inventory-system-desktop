//! 商品管理の業務ロジック（BIZ-01）
//!
//! 30-biz-product-service.md §4 に基づく実装。
//! IO層の関数を組み合わせ、トランザクション制御付きの業務ロジックを提供する。

use crate::db::product_repo::{self, NewPriceHistory, NewProduct, ProductUpdates};
use crate::db::system_repo::{self, NewOperationLog};
use crate::db::{inventory_repo, stocktake_repo, DbConnection, DbError, PaginatedResult};

use super::stock_ledger::{recompute_stock, unlinked_sales_counted};
use super::stocktake_service;
use super::unit_amount::{stock_unit_kind, StockUnitKind};
use super::{jan_code, BizError};
use crate::constants::{MAX_COST_PRICE_CENTI, MAX_SELLING_PRICE, MAX_STOCK_QUANTITY};
use crate::db::inventory_repo::{MovementType, NewMovement};
use crate::db::product_repo::{
    ProductBulkFilter, ProductSearchQuery, ProductStockUnit, ProductTaxRate, ProductWithRelations,
};
use crate::db::stocktake_repo::NewStocktakeItem;

pub use crate::db::product_repo::PriceHistoryEntry;

// ---------------------------------------------------------------------------
// 型定義
// ---------------------------------------------------------------------------

/// 商品登録リクエスト（FUNC-4.2）
#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct ProductCreateRequest {
    pub jan_code: Option<String>,
    pub name: String,
    pub department_id: i64,
    pub selling_price: i64,
    pub cost_price_centi: i64,
    pub tax_rate: ProductTaxRate,
    pub stock_unit: ProductStockUnit,
    pub initial_stock: i64,
    pub maker_code: Option<String>,
    pub supplier_id: Option<i64>,
    pub pos_stock_sync: bool,
    pub plu_target: bool,
    /// BIZ-01-D11: JAN を持ったまま独自コードで登録し、同じ JAN の商品の群に加える。
    /// 商品フォームは `JanInUse` を受けた確認のダイアログからだけ true を送る（51 UI-01b-D26）。
    /// jan_code が None なら意味を持たない
    pub shared_jan_custom_code: bool,
}

/// 商品登録結果（FUNC-4.2）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ProductCreateResult {
    pub product_code: String,
    pub warnings: Vec<String>,
}

/// 商品更新リクエスト（FUNC-4.4）
#[derive(Debug, Default, serde::Deserialize, specta::Type)]
#[serde(default)]
pub struct ProductUpdateRequest {
    pub name: Option<String>,
    pub department_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_nullable_update_field")]
    #[specta(type = Option<Option<i64>>)]
    pub supplier_id: Option<Option<i64>>,
    pub selling_price: Option<i64>,
    pub cost_price_centi: Option<i64>,
    pub tax_rate: Option<ProductTaxRate>,
    #[serde(default, deserialize_with = "deserialize_nullable_update_field")]
    #[specta(type = Option<Option<String>>)]
    pub maker_code: Option<Option<String>>,
    pub pos_stock_sync: Option<bool>,
    pub plu_target: Option<bool>,
    /// 個数の単位どうしの変更だけを許す（SPEC-UNIT-D2・BIZ-01-D8）
    pub stock_unit: Option<ProductStockUnit>,
}

/// 商品更新結果（FUNC-4.4）
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ProductUpdateResult {
    pub warnings: Vec<String>,
}

/// 価格改定入力（SPEC-PRV-D5）。
#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct PriceRevisionInput {
    pub product_code: String,
    pub new_selling_price: i64,
    pub new_cost_price_centi: i64,
    pub assign_supplier_id: Option<i64>,
}

/// 価格改定結果（SPEC-PRV-D5）。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct PriceRevisionResult {
    pub product_code: String,
    pub changed: bool,
    pub plu_dirty_set: bool,
    pub supplier_assigned: bool,
}

/// 取引先統合で付け替えた参照件数。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct SupplierMergeResult {
    pub products_updated: i64,
    pub receiving_records_updated: i64,
}

/// UI-15 一覧用の取引先と利用件数。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct SupplierWithUsage {
    pub id: i64,
    pub name: String,
    pub product_count: i64,
    pub receiving_record_count: i64,
}

fn deserialize_nullable_update_field<'de, D, T>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Option<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

// ---------------------------------------------------------------------------
// Failpoint（テスト専用）
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod failpoint {
    use std::{cell::Cell, thread::LocalKey};

    thread_local! {
        pub static CREATE_PRODUCT_AFTER_INSERT: Cell<bool> = const { Cell::new(false) };
        pub static CREATE_PRODUCT_AFTER_MOVEMENT: Cell<bool> = const { Cell::new(false) };
        pub static UPDATE_PRODUCT_AFTER_PRICE_HISTORY: Cell<bool> = const { Cell::new(false) };
        pub static BULK_SET_PLU_TARGET_AFTER_SECOND_UPDATE: Cell<bool> = const { Cell::new(false) };
    }

    // test が明示 spawn した thread から BIZ を呼ぶ場合は武装が届かない（意図的に武装するなら spawn 先で arm する）。

    /// RAII ガード — Drop 時にフラグを自動リセット（並列テスト汚染防止）
    pub struct FailpointGuard(&'static LocalKey<Cell<bool>>);
    impl Drop for FailpointGuard {
        fn drop(&mut self) {
            self.0.with(|flag| flag.set(false));
        }
    }
    pub fn arm(flag: &'static LocalKey<Cell<bool>>) -> FailpointGuard {
        flag.with(|flag| flag.set(true));
        FailpointGuard(flag)
    }
}

// ---------------------------------------------------------------------------
// 関数
// ---------------------------------------------------------------------------

/// 独自コードを生成する（FUNC-4.3）
///
/// department の code_prefix + "-" + 4 桁以上の 0 埋め連番（9999 の次は 10000）。
/// 連番は、同じ接頭辞の既存の独自コードの番号の最大 + 1 と next_seq の大きい方で、
/// 既存の番号を飛ばし、間の抜けは使わない（BIZ-01-D6）。
/// 連番の更新を呼び出し元の TX に含めるため、借りた transaction だけを受け取る（31 §12.2 と同じ形）。
fn generate_custom_code(
    conn: &rusqlite::Transaction<'_>,
    department_id: i64,
) -> Result<String, BizError> {
    let department = product_repo::find_department_by_id(conn, department_id)?
        .ok_or_else(|| BizError::ValidationFailed("部門が見つかりません".to_string()))?;

    let prefix = department.code_prefix.ok_or_else(|| {
        BizError::ValidationFailed("この部門は独自コード発番に対応していません".to_string())
    })?;

    // 3. 既存の番号の最大（部門を問わず全商品）。i64 の上限は next_seq の更新の前に止める
    let head = format!("{prefix}-");
    let max = product_repo::list_product_codes_by_prefix(conn, &head)?
        .iter()
        .filter_map(|code| custom_code_seq(code, &head))
        .max();
    let overflow =
        || BizError::ValidationFailed("この部門の独自コードの番号を振れません".to_string());
    let floor = match max {
        Some(m) => m.checked_add(1).ok_or_else(overflow)?,
        None => 1,
    };
    department
        .next_seq
        .max(floor)
        .checked_add(1)
        .ok_or_else(overflow)?;

    // 4〜5. next_seq を floor まで上げてから 1 つ進める
    product_repo::raise_next_seq(conn, department_id, floor)?;
    let seq = product_repo::increment_next_seq(conn, department_id)?;
    let code = format!("{}-{:04}", prefix, seq);

    // 7. 重複チェック（step 3〜5 により起きないが、推論の誤りでデータを壊さないため残す）
    if product_repo::find_by_product_code(conn, &code)?.is_some() {
        return Err(BizError::DuplicateProductCode(code));
    }

    Ok(code)
}

/// `{接頭辞}-` の後ろが 1 文字以上の ASCII 数字だけで i64 に収まれば番号を返す（BIZ-01-D6 step 3b）
fn custom_code_seq(code: &str, head: &str) -> Option<i64> {
    let digits = code.strip_prefix(head)?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// 商品を新規登録する（FUNC-4.2）
///
/// generate_custom_code 内の increment_next_seq が DB を更新するため、
/// TX はコード決定の前に開始する（30-biz-product-service.md のステップ2）。
pub fn create_product(
    conn: &mut DbConnection,
    req: ProductCreateRequest,
) -> Result<ProductCreateResult, BizError> {
    // 1. バリデーション（TX外、読み取りのみ）
    validate_create_request(&req, conn)?;

    // 2. BEGIN（コード決定を TX 内で行うため先に開始）
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    // 3. product_code の決定（TX内。30 §4.2 手順 3a・3a2・3b、BIZ-01-D11）
    let mut warnings = Vec::new();
    let product_code = match &req.jan_code {
        Some(jan) if !req.shared_jan_custom_code => {
            // 3a. JAN を他の商品が使っていれば名指して止める（何も書かない）
            if let Some(message) = jan_in_use_message(&tx, jan)? {
                return Err(BizError::JanInUse(message));
            }
            jan.clone()
        }
        Some(jan) => {
            // 3a2. 独自コードを振り、同じ JAN の在庫連動の商品を同じ TX で外す（BIZ-01-D7 のログと warnings）
            let code = generate_custom_code(&tx, req.department_id)?;
            for other in product_repo::find_by_jan_code(&tx, jan)? {
                if other.pos_stock_sync {
                    unlink_shared_jan_product(&tx, &other)?;
                    warnings.push(format!(
                        "同じ JAN の商品 {}「{}」の「POS販売で在庫を減らす」を「減らさない」にしました",
                        other.product_code, other.name
                    ));
                }
            }
            code
        }
        None => generate_custom_code(&tx, req.department_id)?,
    };

    // 3b. 共有 JAN の不変条件（変更の後の状態で、書く前に。30）
    if let Some(others) = shared_jan_conflict(
        &tx,
        &product_code,
        req.jan_code.as_deref(),
        req.pos_stock_sync,
        &JanOverlay::new(),
    )? {
        return Err(BizError::ValidationFailedAt {
            message: shared_jan_message(&others),
            field: "pos_stock_sync".to_string(),
        });
    }

    // 4. products INSERT
    let new_product = NewProduct {
        product_code: product_code.clone(),
        jan_code: req.jan_code.clone(),
        name: req.name.clone(),
        department_id: req.department_id,
        supplier_id: req.supplier_id,
        selling_price: req.selling_price,
        cost_price_centi: req.cost_price_centi,
        tax_rate: req.tax_rate.as_str().to_string(),
        maker_code: req.maker_code.clone(),
        stock_quantity: req.initial_stock,
        stock_unit: req.stock_unit.as_str().to_string(),
        is_discontinued: false,
        plu_dirty: true,
        plu_exported_at: None,
        plu_target: req.plu_target,
        pos_stock_sync: req.pos_stock_sync,
    };

    match product_repo::insert_product(&tx, &new_product) {
        Ok(()) => {}
        Err(DbError::DuplicateKey(_)) => {
            return Err(BizError::DuplicateProductCode(product_code));
        }
        Err(e) => return Err(BizError::DatabaseError(e)),
    }

    #[cfg(test)]
    if failpoint::CREATE_PRODUCT_AFTER_INSERT.with(|f| f.get()) {
        return Err(BizError::DatabaseError(DbError::QueryFailed(
            "failpoint: create_product_after_insert".into(),
        )));
    }

    // 5. 初期在庫の記録
    if req.initial_stock > 0 {
        let movement = NewMovement {
            product_code: product_code.clone(),
            movement_type: MovementType::Receiving,
            quantity: req.initial_stock,
            stock_after: req.initial_stock,
            reference_type: None,
            reference_id: None,
            note: Some("初期在庫投入".to_string()),
            business_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            settlement_no: None,
        };
        inventory_repo::insert_movement(&tx, &movement)?;
    }
    // 写しは初期在庫の movement から求め直す（初期在庫 0 は movement を作らず、求めた値 0 が INSERT の値と同じ。30）
    recompute_stock(&tx, &product_code)?;

    #[cfg(test)]
    if failpoint::CREATE_PRODUCT_AFTER_MOVEMENT.with(|f| f.get()) {
        return Err(BizError::DatabaseError(DbError::QueryFailed(
            "failpoint: create_product_after_movement".into(),
        )));
    }

    // 6. 棚卸し中チェック＋自動追加
    if let Some(stocktake) = stocktake_repo::find_active_stocktake(&tx)? {
        let item = NewStocktakeItem {
            stocktake_id: stocktake.id,
            product_code: product_code.clone(),
            system_stock: req.initial_stock,
            actual_count: None,
        };
        stocktake_repo::insert_stocktake_item(&tx, &item)?;
    }

    // 7. 操作ログ
    let log = NewOperationLog {
        operation_type: "product_create".to_string(),
        summary: format!("商品を登録しました: {} ({})", req.name, product_code),
        detail_json: None,
    };
    system_repo::insert_operation_log(&tx, &log)?;

    // 8. COMMIT
    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(ProductCreateResult {
        product_code,
        warnings,
    })
}

/// 新規登録で入れた JAN `jan` を、JAN か商品コードとして持つ商品を名指す文（BIZ-01-D11）。無ければ None。
/// 並びは商品コードの昇順、印は「商品コードだけ同じ」「POS販売で在庫を減らす」「廃番」の順（商品フォームの画面の語）
fn jan_in_use_message(conn: &DbConnection, jan: &str) -> Result<Option<String>, BizError> {
    let mut users = product_repo::find_by_jan_code(conn, jan)?;
    if let Some(found) = product_repo::find_by_product_code(conn, jan)? {
        if found.product.jan_code.as_deref() != Some(jan) {
            users.push(found.product);
        }
    }
    if users.is_empty() {
        return Ok(None);
    }
    users.sort_by(|a, b| a.product_code.cmp(&b.product_code));
    let named: Vec<String> = users
        .iter()
        .map(|p| {
            let marks: Vec<&str> = [
                (p.jan_code.as_deref() != Some(jan), "商品コードだけ同じ"),
                (p.pos_stock_sync, "POS販売で在庫を減らす"),
                (p.is_discontinued, "廃番"),
            ]
            .into_iter()
            .filter_map(|(hit, mark)| hit.then_some(mark))
            .collect();
            let suffix = if marks.is_empty() {
                String::new()
            } else {
                format!("（{}）", marks.join("・"))
            };
            format!("{}「{}」{}", p.product_code, p.name, suffix)
        })
        .collect();
    Ok(Some(format!(
        "この JAN（{jan}）はほかの商品が使っています: {}",
        named.join("、")
    )))
}

/// 同じ JAN の群の在庫連動の商品を在庫連動なしにし、求め直し、BIZ-01-D7 の形のログを書く（30 §4.2 手順 3a2。
/// §4.4 の update の在庫連動の切替と同じ形）
fn unlink_shared_jan_product(
    tx: &rusqlite::Transaction<'_>,
    product: &product_repo::Product,
) -> Result<(), BizError> {
    product_repo::update_product(
        tx,
        &product.product_code,
        &ProductUpdates {
            pos_stock_sync: Some(false),
            ..Default::default()
        },
    )?;
    recompute_stock(tx, &product.product_code)?;
    let detail = serde_json::json!({ "pos_stock_sync": { "old": true, "new": false } });
    system_repo::insert_operation_log(
        tx,
        &NewOperationLog {
            operation_type: "product_update".to_string(),
            summary: format!("商品を更新しました: {}", product.product_code),
            detail_json: Some(detail.to_string()),
        },
    )?;
    Ok(())
}

/// 商品情報を更新する（FUNC-4.4）
///
/// 売価/原価が変わった場合は price_history に記録し、売価変更時は plu_dirty=true。
pub fn update_product(
    conn: &mut DbConnection,
    product_code: &str,
    req: &ProductUpdateRequest,
) -> Result<ProductUpdateResult, BizError> {
    // 1. 既存商品の取得
    let existing = product_repo::find_by_product_code(conn, product_code)?
        .ok_or_else(|| BizError::NotFound("商品が見つかりません".to_string()))?;

    // 2. バリデーション
    validate_update_request(req, existing.product.stock_unit, conn)?;

    // 3. BEGIN
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    // 3b. 在庫連動を始める（false → true）保存だけ: 共有 JAN の不変条件を先に、次に連動しない間の売上（30、ADR D4・D8）
    if req.pos_stock_sync == Some(true) && !existing.product.pos_stock_sync {
        if let Some(others) = shared_jan_conflict(
            &tx,
            product_code,
            existing.product.jan_code.as_deref(),
            true,
            &JanOverlay::new(),
        )? {
            return Err(BizError::ValidationFailedAt {
                message: shared_jan_message(&others),
                field: "pos_stock_sync".to_string(),
            });
        }
        if !unlinked_sales_counted(&tx, product_code)? {
            return Err(stocktake_service::recount_required(
                &tx,
                product_code,
                MSG_RECOUNT_REQUIRED,
            )?);
        }
    }

    // 4. 価格変更チェック + price_history
    let new_selling = req.selling_price.unwrap_or(existing.product.selling_price);
    let new_cost_centi = req
        .cost_price_centi
        .unwrap_or(existing.product.cost_price_centi);
    let selling_changed =
        req.selling_price.is_some() && req.selling_price != Some(existing.product.selling_price);
    let cost_changed = req.cost_price_centi.is_some()
        && req.cost_price_centi != Some(existing.product.cost_price_centi);

    if selling_changed || cost_changed {
        let history = NewPriceHistory {
            product_code: product_code.to_string(),
            old_selling: existing.product.selling_price,
            new_selling,
            old_cost_centi: existing.product.cost_price_centi,
            new_cost_centi,
        };
        product_repo::insert_price_history(&tx, &history)?;
    }

    #[cfg(test)]
    if failpoint::UPDATE_PRODUCT_AFTER_PRICE_HISTORY.with(|f| f.get()) {
        return Err(BizError::DatabaseError(DbError::QueryFailed(
            "failpoint: update_product_after_price_history".into(),
        )));
    }

    // 5. products UPDATE（plu_dirty は売価変更時、またはPLU対象化時のみ true）
    let plu_target_enabled = req.plu_target == Some(true) && !existing.product.plu_target;
    let mut updates = ProductUpdates {
        name: req.name.clone(),
        department_id: req.department_id,
        supplier_id: req.supplier_id,
        selling_price: req.selling_price,
        cost_price_centi: req.cost_price_centi,
        tax_rate: req.tax_rate.map(|rate| rate.as_str().to_string()),
        maker_code: req.maker_code.clone(),
        pos_stock_sync: req.pos_stock_sync,
        plu_target: req.plu_target,
        // 同じ単位なら SET に出さない。単位は plu_dirty の理由にならない（BIZ-01-D8）
        stock_unit: req
            .stock_unit
            .filter(|unit| *unit != existing.product.stock_unit),
        ..Default::default()
    };
    if selling_changed || plu_target_enabled {
        updates.plu_dirty = Some(true);
    }

    product_repo::update_product(&tx, product_code, &updates)?;
    // 連動の切替の TX はその商品を求め直す（値は変わらない。30）
    if req
        .pos_stock_sync
        .is_some_and(|linked| linked != existing.product.pos_stock_sync)
    {
        recompute_stock(&tx, product_code)?;
    }
    if existing.product.plu_target && req.plu_target == Some(false) {
        if let Some(jan_code) = existing.product.jan_code.as_deref() {
            super::plu_export_service::release_plu_slot_for_jan(&tx, jan_code)?;
        }
    }

    // 6. 操作ログ
    // 変えた field ごとの変更前後をJSON記録（30-biz-product-service.md §4.4 ステップ6・BIZ-01-D7）
    use serde_json::json;
    let prev = &existing.product;
    let mut changes = serde_json::Map::new();
    let mut record = |key: &str, old: serde_json::Value, new: Option<serde_json::Value>| {
        // request が値を持ち（clear の null を含む）、更新前と違うときだけ書く
        if let Some(new) = new.filter(|new| *new != old) {
            changes.insert(key.to_string(), json!({ "old": old, "new": new }));
        }
    };
    record(
        "name",
        json!(prev.name),
        req.name.as_ref().map(|v| json!(v)),
    );
    record(
        "department_id",
        json!(prev.department_id),
        req.department_id.map(|v| json!(v)),
    );
    record(
        "supplier_id",
        json!(prev.supplier_id),
        req.supplier_id.map(|v| json!(v)),
    );
    record(
        "selling_price",
        json!(prev.selling_price),
        req.selling_price.map(|v| json!(v)),
    );
    record(
        "cost_price_centi",
        json!(prev.cost_price_centi),
        req.cost_price_centi.map(|v| json!(v)),
    );
    record(
        "tax_rate",
        json!(prev.tax_rate.as_str()),
        req.tax_rate.map(|v| json!(v.as_str())),
    );
    record(
        "maker_code",
        json!(prev.maker_code),
        req.maker_code.as_ref().map(|v| json!(v)),
    );
    record(
        "pos_stock_sync",
        json!(prev.pos_stock_sync),
        req.pos_stock_sync.map(|v| json!(v)),
    );
    record(
        "plu_target",
        json!(prev.plu_target),
        req.plu_target.map(|v| json!(v)),
    );
    record(
        "stock_unit",
        json!(prev.stock_unit.as_str()),
        req.stock_unit.map(|v| json!(v.as_str())),
    );
    let detail = (!changes.is_empty()).then(|| serde_json::Value::Object(changes).to_string());
    let log = NewOperationLog {
        operation_type: "product_update".to_string(),
        summary: format!("商品を更新しました: {}", product_code),
        detail_json: detail,
    };
    system_repo::insert_operation_log(&tx, &log)?;

    // 7. COMMIT
    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(ProductUpdateResult { warnings: vec![] })
}

/// 売価・原価だけを安全に改定し、未紐付けの場合だけ取引先を補完する。
pub fn revise_product_price(
    conn: &mut DbConnection,
    input: PriceRevisionInput,
) -> Result<PriceRevisionResult, BizError> {
    let existing = product_repo::find_by_product_code(conn, &input.product_code)?
        .ok_or_else(|| BizError::NotFound("商品が見つかりません".to_string()))?;

    if let Some(message) =
        selling_and_cost_range_error(input.new_selling_price, input.new_cost_price_centi)
    {
        return Err(BizError::ValidationFailed(message));
    }

    if let Some(supplier_id) = input.assign_supplier_id {
        product_repo::find_supplier_by_id(conn, supplier_id)?
            .ok_or_else(|| BizError::NotFound("取引先が見つかりません".to_string()))?;
    }

    let selling_changed = input.new_selling_price != existing.product.selling_price;
    let cost_changed = input.new_cost_price_centi != existing.product.cost_price_centi;
    let changed = selling_changed || cost_changed;
    let supplier_assigned =
        existing.product.supplier_id.is_none() && input.assign_supplier_id.is_some();

    if !changed && !supplier_assigned {
        return Ok(PriceRevisionResult {
            product_code: input.product_code,
            changed: false,
            plu_dirty_set: false,
            supplier_assigned: false,
        });
    }

    let tx = conn
        .transaction()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;

    if changed {
        product_repo::insert_price_history(
            &tx,
            &NewPriceHistory {
                product_code: input.product_code.clone(),
                old_selling: existing.product.selling_price,
                new_selling: input.new_selling_price,
                old_cost_centi: existing.product.cost_price_centi,
                new_cost_centi: input.new_cost_price_centi,
            },
        )?;
    }

    #[cfg(test)]
    if changed && failpoint::UPDATE_PRODUCT_AFTER_PRICE_HISTORY.with(|f| f.get()) {
        return Err(BizError::DatabaseError(DbError::QueryFailed(
            "failpoint: update_product_after_price_history".into(),
        )));
    }

    let updates = ProductUpdates {
        selling_price: selling_changed.then_some(input.new_selling_price),
        cost_price_centi: cost_changed.then_some(input.new_cost_price_centi),
        supplier_id: supplier_assigned.then_some(input.assign_supplier_id),
        plu_dirty: selling_changed.then_some(true),
        ..Default::default()
    };
    product_repo::update_product(&tx, &input.product_code, &updates)?;

    if changed {
        system_repo::insert_operation_log(
            &tx,
            &NewOperationLog {
                operation_type: "product_price_revise".to_string(),
                summary: format!("商品価格を改定しました: {}", input.product_code),
                detail_json: Some(format!(
                    r#"{{"selling_price":{{"old":{},"new":{}}},"cost_price_centi":{{"old":{},"new":{}}}}}"#,
                    existing.product.selling_price,
                    input.new_selling_price,
                    existing.product.cost_price_centi,
                    input.new_cost_price_centi
                )),
            },
        )?;
    }

    tx.commit()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;

    Ok(PriceRevisionResult {
        product_code: input.product_code,
        changed,
        plu_dirty_set: selling_changed,
        supplier_assigned,
    })
}

/// 商品の廃番フラグを反転する（FUNC-4.5）
pub fn toggle_discontinue(conn: &mut DbConnection, product_code: &str) -> Result<bool, BizError> {
    let existing = product_repo::find_by_product_code(conn, product_code)?
        .ok_or_else(|| BizError::NotFound("商品が見つかりません".to_string()))?;

    let new_status = !existing.product.is_discontinued;

    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    let updates = ProductUpdates {
        is_discontinued: Some(new_status),
        plu_dirty: new_status.then_some(true),
        plu_target: new_status.then_some(false),
        ..Default::default()
    };
    product_repo::update_product(&tx, product_code, &updates)?;
    if new_status {
        if let Some(jan_code) = existing.product.jan_code.as_deref() {
            super::plu_export_service::release_plu_slot_for_jan(&tx, jan_code)?;
        }
    }

    let action = if new_status { "廃番" } else { "復帰" };
    let log = NewOperationLog {
        operation_type: "product_discontinue".to_string(),
        summary: format!("商品を{}しました: {}", action, product_code),
        detail_json: None,
    };
    system_repo::insert_operation_log(&tx, &log)?;

    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    Ok(new_status)
}

/// 商品コードで商品詳細を取得する
///
/// CMD-01 get_product 用。BIZ層ではロジックなし。IO層のラッパー。
pub fn get_product(
    conn: &DbConnection,
    product_code: &str,
) -> Result<ProductWithRelations, BizError> {
    product_repo::find_by_product_code(conn, product_code)?
        .ok_or_else(|| BizError::NotFound(format!("商品が見つかりません: {}", product_code)))
}

/// 検索条件に基づいて商品一覧を取得する（FUNC-4.6）
///
/// BIZ層では追加の業務ロジックなし。IO層のラッパー。
pub fn search_products(
    conn: &DbConnection,
    query: ProductSearchQuery,
) -> Result<PaginatedResult<ProductWithRelations>, BizError> {
    product_repo::search_products(conn, &query).map_err(BizError::from)
}

/// filter 一致全件の PLU 対象状態を 1 transaction で更新する。
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct BulkPluTargetResult {
    pub matched_count: u32,
    pub updated_count: u32,
    pub invalid_jan_skipped_count: u32,
    pub discontinued_skipped_count: u32,
}

pub fn bulk_set_plu_target(
    conn: &mut DbConnection,
    mut filter: ProductBulkFilter,
    plu_target: bool,
) -> Result<BulkPluTargetResult, BizError> {
    filter.keyword = filter
        .keyword
        .take()
        .map(|keyword| keyword.trim().to_string())
        .filter(|keyword| !keyword.is_empty());

    let tx = conn
        .transaction()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;
    let products = product_repo::find_products_for_bulk_plu_target(&tx, &filter)?;
    let matched_count = u32::try_from(products.len()).unwrap_or(u32::MAX);
    let mut updated_count = 0_u32;
    let mut invalid_jan_skipped_count = 0_u32;
    let mut discontinued_skipped_count = 0_u32;

    for product in &products {
        if plu_target {
            if product.plu_target {
                continue;
            }
            if product.is_discontinued {
                discontinued_skipped_count += 1;
                continue;
            }
            if !is_plu_target_eligible_jan(product.jan_code.as_deref()) {
                invalid_jan_skipped_count += 1;
                continue;
            }
            product_repo::update_product(
                &tx,
                &product.product_code,
                &ProductUpdates {
                    plu_target: Some(true),
                    plu_dirty: Some(true),
                    ..Default::default()
                },
            )?;
        } else {
            if !product.plu_target {
                continue;
            }
            product_repo::update_product(
                &tx,
                &product.product_code,
                &ProductUpdates {
                    plu_target: Some(false),
                    ..Default::default()
                },
            )?;
            if let Some(jan_code) = product.jan_code.as_deref() {
                super::plu_export_service::release_plu_slot_for_jan(&tx, jan_code)?;
            }
        }
        updated_count += 1;

        #[cfg(test)]
        if updated_count == 2
            && failpoint::BULK_SET_PLU_TARGET_AFTER_SECOND_UPDATE.with(|f| f.get())
        {
            return Err(BizError::DatabaseError(DbError::QueryFailed(
                "failpoint: bulk_set_plu_target_after_second_update".into(),
            )));
        }
    }

    let detail = serde_json::json!({
        "filter": {
            "keyword": filter.keyword,
            "department_id": filter.department_id,
            "is_discontinued": filter.is_discontinued,
            "plu": filter.plu,
        },
        "plu_target": plu_target,
        "matched_count": matched_count,
        "updated_count": updated_count,
        "invalid_jan_skipped_count": invalid_jan_skipped_count,
        "discontinued_skipped_count": discontinued_skipped_count,
    });
    system_repo::insert_operation_log(
        &tx,
        &NewOperationLog {
            operation_type: "product_bulk_plu_target".to_string(),
            summary: format!(
                "PLU対象一括更新: 要求={}, 一致{}件、更新{}件、JAN不備{}件、廃番{}件",
                plu_target,
                matched_count,
                updated_count,
                invalid_jan_skipped_count,
                discontinued_skipped_count
            ),
            detail_json: Some(detail.to_string()),
        },
    )?;
    tx.commit()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;

    Ok(BulkPluTargetResult {
        matched_count,
        updated_count,
        invalid_jan_skipped_count,
        discontinued_skipped_count,
    })
}

/// 部門選択候補を全件取得する（FUNC-4.7 / UI-01a-D7）
///
/// BIZ層では追加の業務ロジックなし。検索結果から候補を派生せず、IO層の部門 master data を返す。
pub fn list_departments(conn: &DbConnection) -> Result<Vec<product_repo::Department>, BizError> {
    product_repo::list_departments(conn).map_err(BizError::from)
}

/// 取引先選択候補を全件取得する（FUNC-4.7.1 / UI-01b-D7）
///
/// BIZ層では追加の業務ロジックなし。商品や検索結果から候補を派生せず、IO層の取引先 master data を返す。
pub fn list_suppliers(conn: &DbConnection) -> Result<Vec<product_repo::Supplier>, BizError> {
    product_repo::list_suppliers(conn).map_err(BizError::from)
}

/// 取引先名を正規化し、同名がなければ作成する。
pub fn create_supplier(
    conn: &DbConnection,
    name: String,
) -> Result<product_repo::Supplier, BizError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(BizError::ValidationFailed(
            "取引先名を入力してください".to_string(),
        ));
    }
    product_repo::find_or_create_supplier(conn, trimmed).map_err(BizError::from)
}

/// 取引先名を改名し、同一 transaction で監査ログを記録する。
pub fn rename_supplier(
    conn: &mut DbConnection,
    supplier_id: i64,
    name: String,
) -> Result<product_repo::Supplier, BizError> {
    let existing = product_repo::find_supplier_by_id(conn, supplier_id)?
        .ok_or_else(|| BizError::NotFound("取引先が見つかりません".to_string()))?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(BizError::ValidationFailed(
            "取引先名を入力してください".to_string(),
        ));
    }
    if trimmed == existing.name {
        return Ok(existing);
    }
    if product_repo::list_suppliers(conn)?
        .iter()
        .any(|supplier| supplier.id != supplier_id && supplier.name == trimmed)
    {
        return Err(BizError::ValidationFailed(
            "同じ名前の取引先があります。重複している場合は「統合」を使ってください。".to_string(),
        ));
    }

    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let tx = conn
        .transaction()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;
    let renamed = match product_repo::rename_supplier(&tx, supplier_id, trimmed, &now) {
        Ok(supplier) => supplier,
        Err(DbError::DuplicateKey(_)) => {
            return Err(BizError::ValidationFailed(
                "同じ名前の取引先があります。重複している場合は「統合」を使ってください。"
                    .to_string(),
            ))
        }
        Err(error) => return Err(BizError::from(error)),
    };
    let detail = serde_json::json!({
        "supplier_id": supplier_id,
        "old_name": existing.name,
        "new_name": renamed.name,
    });
    system_repo::insert_operation_log(
        &tx,
        &NewOperationLog {
            operation_type: "supplier_rename".to_string(),
            summary: format!("取引先名を変更しました: {}", supplier_id),
            detail_json: Some(detail.to_string()),
        },
    )?;
    tx.commit()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;
    Ok(renamed)
}

/// 重複した取引先を残す側へ統合し、同一 transaction で監査ログを記録する。
pub fn merge_suppliers(
    conn: &mut DbConnection,
    source_id: i64,
    target_id: i64,
) -> Result<SupplierMergeResult, BizError> {
    if source_id == target_id {
        return Err(BizError::ValidationFailed(
            "同じ取引先同士は統合できません".to_string(),
        ));
    }
    let source = product_repo::find_supplier_by_id(conn, source_id)?
        .ok_or_else(|| BizError::NotFound("統合元の取引先が見つかりません".to_string()))?;
    let target = product_repo::find_supplier_by_id(conn, target_id)?
        .ok_or_else(|| BizError::NotFound("統合先の取引先が見つかりません".to_string()))?;

    let tx = conn
        .transaction()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;
    let (products_updated, receiving_records_updated) =
        product_repo::merge_suppliers(&tx, source_id, target_id)?;
    let detail = serde_json::json!({
        "source_id": source.id,
        "source_name": source.name,
        "target_id": target.id,
        "target_name": target.name,
        "products_updated": products_updated,
        "receiving_records_updated": receiving_records_updated,
    });
    system_repo::insert_operation_log(
        &tx,
        &NewOperationLog {
            operation_type: "supplier_merge".to_string(),
            summary: format!("取引先を統合しました: {} -> {}", source_id, target_id),
            detail_json: Some(detail.to_string()),
        },
    )?;
    tx.commit()
        .map_err(|error| BizError::DatabaseError(DbError::from(error)))?;
    Ok(SupplierMergeResult {
        products_updated,
        receiving_records_updated,
    })
}

/// 取引先全件を利用件数付きで name 昇順に返す。
pub fn list_suppliers_with_usage(conn: &DbConnection) -> Result<Vec<SupplierWithUsage>, BizError> {
    Ok(product_repo::list_suppliers_with_usage(conn)?
        .into_iter()
        .map(|row| SupplierWithUsage {
            id: row.supplier.id,
            name: row.supplier.name,
            product_count: row.product_count,
            receiving_record_count: row.receiving_record_count,
        })
        .collect())
}

/// 商品の価格履歴を新しい順で取得する。
pub fn list_price_history(
    conn: &DbConnection,
    product_code: String,
    limit: u32,
) -> Result<Vec<PriceHistoryEntry>, BizError> {
    product_repo::list_price_history(conn, &product_code, limit).map_err(BizError::from)
}

/// 商品の在庫詳細を取得する（最終入庫日・最終販売日付き）
///
/// 44-cmd-inventory.md §23.8 get_stock_detail
pub fn get_stock_detail(
    conn: &DbConnection,
    product_code: &str,
) -> Result<product_repo::StockDetail, BizError> {
    product_repo::get_stock_detail(conn, product_code).map_err(|e| match e {
        DbError::NotFound => BizError::NotFound(format!("商品が見つかりません: {}", product_code)),
        other => BizError::from(other),
    })
}

/// 在庫が閾値以下の商品を一覧取得する
///
/// 44-cmd-inventory.md §23.8 list_low_stock
/// 閾値は app_settings から取得し、未設定時はデフォルト値を使用する
pub fn list_low_stock(
    conn: &DbConnection,
    include_discontinued: bool,
) -> Result<Vec<ProductWithRelations>, BizError> {
    let threshold_count = system_repo::get_setting(conn, "stock_low_threshold")?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(3);
    let threshold_length = system_repo::get_setting(conn, "stock_low_threshold_fabric")?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(500);
    // 個数の単位は一般の基準、長さの単位は生地の基準（SPEC-UNIT-D10）。全 variant をどちらかに入れる
    let (count_units, length_units) = low_stock_unit_groups();
    Ok(product_repo::list_low_stock_products(
        conn,
        &count_units,
        threshold_count,
        &length_units,
        threshold_length,
        include_discontinued,
    )?)
}

/// 全単位を数量の種類で個数と長さの 2 つの並びに分ける（在庫少の SQL に渡す）
fn low_stock_unit_groups() -> (Vec<ProductStockUnit>, Vec<ProductStockUnit>) {
    ProductStockUnit::ALL
        .into_iter()
        .partition(|unit| stock_unit_kind(*unit) == StockUnitKind::Count)
}

// ---------------------------------------------------------------------------
// バリデーション（内部関数）
// ---------------------------------------------------------------------------

fn should_default_plu_target(jan_code: Option<&str>) -> bool {
    jan_code.is_some_and(|jan| jan.len() == 13 && jan.chars().all(|ch| ch.is_ascii_digit()))
}

fn is_plu_target_eligible_jan(jan_code: Option<&str>) -> bool {
    jan_code.is_some_and(|jan| jan.len() == 13 && jan_code::validate(jan).is_ok())
}

/// 売価・原価の範囲（共通規則「入力の上限と安全な整数の範囲」(a)）。外れたら利用者向けの文を返す
fn price_range_error(label: &str, value: i64, max: i64) -> Option<String> {
    if value < 0 {
        Some(format!("{label}は0以上で入力してください"))
    } else if value > max {
        Some(format!("{label}が大きすぎます"))
    } else {
        None
    }
}

/// 売価と原価を範囲で確かめ、最初に外れた 1 つの文を返す
fn selling_and_cost_range_error(selling_price: i64, cost_price_centi: i64) -> Option<String> {
    price_range_error("売価", selling_price, MAX_SELLING_PRICE)
        .or_else(|| price_range_error("原価", cost_price_centi, MAX_COST_PRICE_CENTI))
}

const MSG_STOCK_TOO_LARGE: &str = "在庫数が大きすぎます";

/// 単位の表示の語（SPEC-UNIT-D1 の表）。商品 CSV の `在庫単位` の語と error の文に使う
fn stock_unit_label(unit: ProductStockUnit) -> &'static str {
    match unit {
        ProductStockUnit::Pcs => "個",
        ProductStockUnit::Sheet => "枚",
        ProductStockUnit::Hon => "本",
        ProductStockUnit::Bag => "袋",
        ProductStockUnit::Box => "箱",
        ProductStockUnit::Kumi => "組",
        ProductStockUnit::Set => "セット",
        ProductStockUnit::Ball => "玉",
        ProductStockUnit::Cho => "丁",
        ProductStockUnit::M => "m",
    }
}

/// 商品 CSV の `在庫単位`（code か表示の語）を単位にする（SPEC-UNIT-D10）
fn normalize_stock_unit(value: &str) -> Option<ProductStockUnit> {
    ProductStockUnit::from_code(value).or_else(|| {
        ProductStockUnit::ALL
            .into_iter()
            .find(|unit| stock_unit_label(*unit) == value)
    })
}

const MSG_STOCK_UNIT_KIND: &str = "在庫単位は個数の単位どうしでだけ変えられます";

/// 単位を変えてよいか（SPEC-UNIT-D2・BIZ-01-D8）。同じか、どちらも個数の単位。商品修正・CSV の preview・commit が共有する
fn stock_unit_change_allowed(existing: ProductStockUnit, new: ProductStockUnit) -> bool {
    existing == new
        || (stock_unit_kind(existing) == StockUnitKind::Count
            && stock_unit_kind(new) == StockUnitKind::Count)
}

fn stock_unit_changed_message(existing: ProductStockUnit, csv_value: &str) -> String {
    format!(
        "{MSG_STOCK_UNIT_KIND}（登録: {}、CSV: {}）",
        stock_unit_label(existing),
        csv_value
    )
}

/// 上書きが単位だけを変えるか（BIZ-01-D8 の上書きの `plu_dirty`）。
/// UPDATE に渡す `Some` の列のうち単位と plu_dirty の他が全て既存と同じなら真（比べる列は UPDATE が書く列と同じ）
fn overwrite_changes_unit_only(updates: &ProductUpdates, existing: &product_repo::Product) -> bool {
    fn same<T: PartialEq>(new: &Option<T>, old: &T) -> bool {
        new.as_ref().is_none_or(|new| new == old)
    }
    // field を足したら compile error で比べるかを決めさせるため `..` を置かない
    let ProductUpdates {
        jan_code,
        name,
        department_id,
        supplier_id,
        selling_price,
        cost_price_centi,
        tax_rate,
        maker_code,
        stock_unit,
        is_discontinued,
        plu_dirty: _,
        plu_exported_at,
        plu_target,
        pos_stock_sync,
    } = updates;
    stock_unit.is_some()
        && same(jan_code, &existing.jan_code)
        && same(name, &existing.name)
        && same(department_id, &existing.department_id)
        && same(supplier_id, &existing.supplier_id)
        && same(selling_price, &existing.selling_price)
        && same(cost_price_centi, &existing.cost_price_centi)
        && same(&tax_rate.as_deref(), &existing.tax_rate.as_str())
        && same(maker_code, &existing.maker_code)
        && same(is_discontinued, &existing.is_discontinued)
        && same(plu_exported_at, &existing.plu_exported_at)
        && same(plu_target, &existing.plu_target)
        && same(pos_stock_sync, &existing.pos_stock_sync)
}

/// 小数 2 桁までの数（`^[0-9]+(\.[0-9]{1,2})?$`）を 100 倍の整数にする（浮動小数を通さない）。
/// m の数 → cm（SPEC-UNIT-D3）と、原価の円 → 1/100 円（SPEC-UNIT-D5）が使う。
///
/// 規則に合わなければ `None`、i64 に収まらなければ `Some(None)`。
fn parse_hundredths(value: &str) -> Option<Option<i64>> {
    let (whole, fraction) = match value.split_once('.') {
        Some((whole, fraction)) if (1..=2).contains(&fraction.len()) => (whole, fraction),
        Some(_) => return None,
        None => (value, ""),
    };
    let all_digits = |text: &str| text.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || !all_digits(whole) || !all_digits(fraction) {
        return None;
    }
    // 小数部は右に 0 を埋めた 2 桁（"3" → 30、"" → 0）
    let fraction_hundredths = format!("{fraction:0<2}").parse::<i64>().ok()?;
    Some(
        whole
            .parse::<i64>()
            .ok()
            .and_then(|units| units.checked_mul(100))
            .and_then(|hundredths| hundredths.checked_add(fraction_hundredths)),
    )
}

const MSG_COST_FORMAT: &str = "原価は0以上で、小数は2桁までで入力してください";

/// 商品 CSV の `原価` を 1/100 円にする（SPEC-UNIT-D10、Y2）。外れたら行の error の文
///
/// 先頭が `-` で残りが規則に合えば負の文、規則に合うが i64 に収まらなければ上限の文、
/// それ以外の形の誤り（小数 3 桁・指数・`+`・`.5`・`5.`・空）は形式の文（直し方を名指す）。
fn parse_cost_cell(value: &str) -> Result<i64, String> {
    if value
        .strip_prefix('-')
        .is_some_and(|rest| parse_hundredths(rest).is_some())
    {
        return Err("原価は0以上で入力してください".to_string());
    }
    match parse_hundredths(value) {
        None => Err(MSG_COST_FORMAT.to_string()),
        Some(None) => Err("原価が大きすぎます".to_string()),
        Some(Some(cost)) => match price_range_error("原価", cost, MAX_COST_PRICE_CENTI) {
            Some(message) => Err(message),
            None => Ok(cost),
        },
    }
}

/// vB で外した単位（`cm` と `巻`・`roll`）にだけ添える直し方（SPEC-UNIT-D10・D-128）
fn retired_stock_unit_hint(value: &str) -> Option<&'static str> {
    match value {
        "cm" => Some(
            "長さは m で入れてください。初期在庫も m の数〈cm の数を 100 で割る〉にしてください",
        ),
        "巻" | "roll" => Some("巻で数える商品は 個 で入れてください。数は変わりません"),
        _ => None,
    }
}

/// 商品 CSV の `在庫単位` の行の error（preview と commit の再検証が同じ文を使う）
fn stock_unit_invalid_message(value: &str) -> String {
    match retired_stock_unit_hint(value) {
        Some(hint) => format!("在庫単位が正しくありません: {value}（{hint}）"),
        None => format!("在庫単位が正しくありません: {value}"),
    }
}

/// 商品 CSV の commit の再検証（wire の行を信頼しない。BIZ-01-D8）。外れた最初の 1 件の文を返す
fn import_row_commit_error(
    row: &ImportRow,
    existing_unit: Option<ProductStockUnit>,
) -> Option<String> {
    if let Some(message) = selling_and_cost_range_error(row.selling_price, row.cost_price_centi) {
        return Some(message);
    }
    // 単位は 10 個の code（新規の行と上書きの行）
    let unit = match row.stock_unit.as_deref() {
        Some(code) => match ProductStockUnit::from_code(code) {
            Some(unit) => Some(unit),
            None => return Some(stock_unit_invalid_message(code)),
        },
        None => None,
    };
    match (existing_unit, unit) {
        // 上書きの行: 単位は個数の単位どうしでだけ変える。初期在庫は読まない
        (Some(existing), Some(unit)) if !stock_unit_change_allowed(existing, unit) => {
            Some(stock_unit_changed_message(existing, unit.as_str()))
        }
        (Some(_), _) => None,
        // 新規の行: 初期在庫は 0 以上で上限まで
        (None, _) => match row.initial_stock {
            Some(stock) if stock < 0 => Some(format!("初期在庫の値が不正です: '{stock}'")),
            Some(stock) if stock > MAX_STOCK_QUANTITY => Some(MSG_STOCK_TOO_LARGE.to_string()),
            _ => None,
        },
    }
}

fn validate_create_request(
    req: &ProductCreateRequest,
    conn: &DbConnection,
) -> Result<(), BizError> {
    if req.name.trim().is_empty() {
        return Err(BizError::ValidationFailed("商品名は必須です".to_string()));
    }
    if let Some(message) = selling_and_cost_range_error(req.selling_price, req.cost_price_centi) {
        return Err(BizError::ValidationFailed(message));
    }
    if req.initial_stock < 0 {
        return Err(BizError::ValidationFailed(
            "初期在庫は0以上で入力してください".to_string(),
        ));
    }
    if req.initial_stock > MAX_STOCK_QUANTITY {
        return Err(BizError::ValidationFailed(MSG_STOCK_TOO_LARGE.to_string()));
    }
    // department 存在チェック
    if product_repo::find_department_by_id(conn, req.department_id)?.is_none() {
        return Err(BizError::ValidationFailed(
            "指定された部門が存在しません".to_string(),
        ));
    }
    // supplier 存在チェック（指定時のみ）
    if let Some(supplier_id) = req.supplier_id {
        let suppliers = product_repo::list_suppliers(conn)?;
        if !suppliers.iter().any(|s| s.id == supplier_id) {
            return Err(BizError::ValidationFailed(
                "指定された取引先が存在しません".to_string(),
            ));
        }
    }
    if let Some(jan_code) = req.jan_code.as_deref() {
        jan_code::validate(jan_code)
            .map_err(|error| BizError::ValidationFailed(error.message().to_string()))?;
    }
    Ok(())
}

fn validate_update_request(
    req: &ProductUpdateRequest,
    existing_unit: ProductStockUnit,
    conn: &DbConnection,
) -> Result<(), BizError> {
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err(BizError::ValidationFailed("商品名は必須です".to_string()));
        }
    }
    if let Some(message) = req
        .selling_price
        .and_then(|price| price_range_error("売価", price, MAX_SELLING_PRICE))
        .or_else(|| {
            req.cost_price_centi
                .and_then(|cost| price_range_error("原価", cost, MAX_COST_PRICE_CENTI))
        })
    {
        return Err(BizError::ValidationFailed(message));
    }
    if let Some(dept_id) = req.department_id {
        if product_repo::find_department_by_id(conn, dept_id)?.is_none() {
            return Err(BizError::ValidationFailed(
                "指定された部門が存在しません".to_string(),
            ));
        }
    }
    if let Some(Some(supplier_id)) = req.supplier_id {
        let suppliers = product_repo::list_suppliers(conn)?;
        if !suppliers.iter().any(|s| s.id == supplier_id) {
            return Err(BizError::ValidationFailed(
                "指定された取引先が存在しません".to_string(),
            ));
        }
    }
    // 単位は手順 2 の今の検査の後（BIZ-01-D8）
    if req
        .stock_unit
        .is_some_and(|unit| !stock_unit_change_allowed(existing_unit, unit))
    {
        return Err(BizError::ValidationFailed(MSG_STOCK_UNIT_KIND.to_string()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 共有 JAN と在庫連動を始める検査（30「計数の置き場と在庫の求め直しの契約」、ADR SPEC-STK-CNT-D4・D8）
// ---------------------------------------------------------------------------

/// preview で先の行を当てた仮の状態（商品コード → 変更の後の JAN と在庫連動）
type JanOverlay = std::collections::HashMap<String, (Option<String>, bool)>;

/// 在庫連動を始める保存の拒否の文（30。一括 import の行の error も同じ文）
const MSG_RECOUNT_REQUIRED: &str =
    "連動しない間の売上があります。今の数を入れてから在庫連動にしてください";

fn shared_jan_message(others: &[String]) -> String {
    format!(
        "同じ JAN の商品（{}）があるため、在庫連動にできません",
        others.join("、")
    )
}

/// 変更の後に商品 `code` が JAN `jan`・在庫連動 `linked` を持つとき、不変条件（在庫連動の商品と同じ JAN を
/// 別の商品が持たない）が破れるなら、同じ JAN を持つほかの商品コード（昇順）を返す。
/// `overlay` は DB の今の状態に重ねる仮の変更（preview の先の行）
fn shared_jan_conflict(
    conn: &DbConnection,
    code: &str,
    jan: Option<&str>,
    linked: bool,
    overlay: &JanOverlay,
) -> Result<Option<Vec<String>>, BizError> {
    let Some(jan) = jan else {
        return Ok(None);
    };
    let mut others: Vec<(String, bool)> = product_repo::find_by_jan_code(conn, jan)?
        .into_iter()
        .filter(|p| p.product_code != code && !overlay.contains_key(&p.product_code))
        .map(|p| (p.product_code, p.pos_stock_sync))
        .collect();
    others.extend(
        overlay
            .iter()
            .filter(|(other, (other_jan, _))| *other != code && other_jan.as_deref() == Some(jan))
            .map(|(other, (_, other_linked))| (other.clone(), *other_linked)),
    );
    if others.is_empty() || !(linked || others.iter().any(|(_, other_linked)| *other_linked)) {
        return Ok(None);
    }
    let mut codes: Vec<String> = others.into_iter().map(|(other, _)| other).collect();
    codes.sort();
    Ok(Some(codes))
}

/// 一括 import の 1 行の共有 JAN と在庫連動を始める検査（30。preview の行の error と確定の拒否で同じ文）。
/// 共有 JAN を先に返す（数えても直らない拒否を先に名指す）
fn import_row_link_error(
    conn: &DbConnection,
    row: &ImportRow,
    existing: Option<&product_repo::Product>,
    overlay: &JanOverlay,
) -> Result<Option<String>, BizError> {
    let linked = row
        .pos_stock_sync
        .unwrap_or(existing.is_none_or(|p| p.pos_stock_sync));
    if let Some(others) = shared_jan_conflict(
        conn,
        &row.product_code,
        row.jan_code.as_deref(),
        linked,
        overlay,
    )? {
        return Ok(Some(shared_jan_message(&others)));
    }
    let starts_link = existing.is_some_and(|p| !p.pos_stock_sync) && linked;
    if starts_link && !unlinked_sales_counted(conn, &row.product_code)? {
        return Ok(Some(MSG_RECOUNT_REQUIRED.to_string()));
    }
    Ok(None)
}

/// 一括 import の確定の TX で当たった行の拒否（30。最初に当たった行を名指し、確定全体を戻す）
fn import_commit_link_error(product_code: &str, message: &str) -> BizError {
    BizError::ValidationFailed(format!(
        "商品 {product_code} の行: {message}。読み込みの後に登録か売上が変わったか、上書きを選んだほかの行と重なったため、取込みを止めました（どの行も取り込んでいません）。その行をスキップにするか、ファイルを選び直してもう一度読み込んでください"
    ))
}

// ---------------------------------------------------------------------------
// 一括インポート（BIZ-01 §4.7-4.8）
// ---------------------------------------------------------------------------

/// 一括インポート用行データ
///
/// オプション項目は None = CSV に値なし。デフォルト化は commit_import の INSERT 時のみ。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ImportRow {
    pub line_no: usize,
    pub product_code: String,
    pub name: String,
    pub department_id: i64,
    pub selling_price: i64,
    pub cost_price_centi: i64,
    pub tax_rate: String,
    pub stock_unit: Option<String>,
    pub initial_stock: Option<i64>,
    pub jan_code: Option<String>,
    pub maker_code: Option<String>,
    pub supplier_id: Option<i64>,
    pub pos_stock_sync: Option<bool>,
    #[serde(default)]
    pub plu_target: Option<bool>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// バリデーションエラー行
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ImportErrorRow {
    pub line_no: usize,
    pub raw_data: std::collections::HashMap<String, String>,
    pub errors: Vec<String>,
}

/// 重複行（product_code が既存）
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ImportDuplicateRow {
    pub line_no: usize,
    pub import_row: ImportRow,
    pub existing_product_code: String,
}

/// インポートプレビュー結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ImportPreview {
    pub valid_rows: Vec<ImportRow>,
    pub error_rows: Vec<ImportErrorRow>,
    pub duplicate_rows: Vec<ImportDuplicateRow>,
}

/// インポート結果
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ProductImportResult {
    pub created_count: usize,
    pub updated_count: usize,
    pub skipped_count: usize,
}

const PRODUCT_CODE_TOO_LONG_MESSAGE: &str = "商品コードは100文字以内で入力してください";

/// 商品コードが上限（BIZ-01-D5、UTF-16 code unit）を超えるか
fn product_code_exceeds_max_len(code: &str) -> bool {
    code.encode_utf16().count() > crate::constants::PRODUCT_CODE_MAX_LEN
}

/// Excel の指数の形（`4.90123E+12`・`5E+12`）か。BIZ-01-D10 F1
fn is_exponent_form(value: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|byte| byte.is_ascii_digit());
    let Some((mantissa, exponent)) = value.split_once(['E', 'e']) else {
        return false;
    };
    exponent.strip_prefix('+').is_some_and(digits)
        && mantissa
            .split_once('.')
            .map_or(digits(mantissa), |(int, frac)| digits(int) && digits(frac))
}

/// 商品 CSV の商品コードと JANコードの形の行の error（BIZ-01-D10）。行の値だけを見る。
/// preview（§4.8 手順 4）と commit（§4.9 手順 0）が同じ関数を呼ぶ。文の順は表の順（F1〜F4）
fn import_code_shape_errors(product_code: &str, jan_code: Option<&str>) -> Vec<String> {
    let exponent = |column: &str, value: &str| {
        format!(
            "{column}が指数の形です: '{value}'。Excel が桁を落として書いた値です。元の Excel でその列を文字列の値にして（TEXT の列）保存し直してください。セルを選んだときの上の欄（数式バー）に E+ が出るなら、元の資料から全部の桁を入れ直してください"
        )
    };
    let mut errors = Vec::new();
    if is_exponent_form(product_code) {
        errors.push(exponent("商品コード", product_code));
    }
    match jan_code.filter(|jan| !jan.is_empty()) {
        None => {
            if product_code.len() == 13 && product_code.bytes().all(|byte| byte.is_ascii_digit()) {
                errors.push(format!(
                    "商品コードが13桁の数字でJANコードが空です: '{product_code}'。JANを商品コードにした商品は、JANコードにも同じ値を入れてください"
                ));
            }
        }
        Some(jan) if is_exponent_form(jan) => errors.push(exponent("JANコード", jan)),
        Some(jan) => match jan_code::validate(jan) {
            Ok(()) => {}
            Err(jan_code::JanCodeValidationError::InvalidLengthOrCharacters) => {
                errors.push(format!(
                    "JANコードが8桁か13桁の数字ではありません: '{jan}'。先頭の0が落ちていないかを見て、商品のバーコードの数字を元の資料から文字列で入れ直してください"
                ));
            }
            Err(jan_code::JanCodeValidationError::InvalidCheckDigit) => {
                errors.push(format!(
                    "JANコードのチェックディジットが合いません: '{jan}'。商品のバーコードの数字を元の資料から入れ直してください"
                ));
            }
        },
    }
    errors
}

/// 商品マスタCSVファイルをプレビューする（読み取り専用、DB書込みなし）
///
/// 30-biz-product-service.md §4.7 / BIZ-01-VAL-D1
pub fn preview_import(conn: &DbConnection, file_bytes: &[u8]) -> Result<ImportPreview, BizError> {
    if file_bytes.len() > crate::constants::CSV_IMPORT_FILE_SIZE_LIMIT {
        return Err(BizError::ImportError(
            "ファイルサイズが上限（20MB）を超えています".to_string(),
        ));
    }
    if file_bytes.is_empty() {
        return Err(BizError::ValidationFailed("ファイルが空です".to_string()));
    }

    // 1. IO-03 呼出し
    let parse_result = crate::io::product_csv_importer::parse_product_csv(file_bytes)
        .map_err(BizError::ImportError)?;

    // 2. ヘッダ検証
    let required_headers = ["商品コード", "商品名", "部門ID", "売価", "原価", "税率"];
    let missing: Vec<&str> = required_headers
        .iter()
        .filter(|h| !parse_result.headers.contains(&h.to_string()))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(BizError::ImportError(format!(
            "必須列が不足しています: {}",
            missing.join(", ")
        )));
    }

    // 部門リストをキャッシュ（N+1回避）
    let departments = product_repo::list_departments(conn)?;
    let dept_ids: std::collections::HashSet<i64> = departments.iter().map(|d| d.id).collect();

    // 取引先リストをキャッシュ
    let suppliers = product_repo::list_suppliers(conn)?;
    let supplier_ids: std::collections::HashSet<i64> = suppliers.iter().map(|s| s.id).collect();

    let mut valid_rows = Vec::new();
    let mut error_rows = Vec::new();
    let mut duplicate_rows = Vec::new();

    // #2 (P1): IO-03 の parse_errors を error_rows にマージ
    for parse_err in &parse_result.parse_errors {
        error_rows.push(ImportErrorRow {
            line_no: parse_err.line_no,
            raw_data: std::collections::HashMap::new(),
            errors: vec![parse_err.error_message.clone()],
        });
    }

    // #5 (P2): CSV内重複検出用
    let mut seen_codes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for row in &parse_result.rows {
        let line_no = row.line_no; // IO-03 が保持する元CSVの行番号
        let mut errors = Vec::new();

        // 3. 各行バリデーション
        let product_code = row.fields.get("商品コード").cloned().unwrap_or_default();
        if product_code.is_empty() {
            errors.push("商品コードが空です".to_string());
        } else if product_code_exceeds_max_len(&product_code) {
            errors.push(PRODUCT_CODE_TOO_LONG_MESSAGE.to_string());
        }

        // #5 (P2): CSV内重複チェック
        if !product_code.is_empty() && !seen_codes.insert(product_code.clone()) {
            errors.push("CSV内で商品コードが重複しています".to_string());
        }

        let name = row.fields.get("商品名").cloned().unwrap_or_default();
        if name.is_empty() {
            errors.push("商品名が空です".to_string());
        }

        let department_id = row.fields.get("部門ID").and_then(|v| v.parse::<i64>().ok());
        match department_id {
            None => errors.push("部門IDが不正です".to_string()),
            Some(id) if !dept_ids.contains(&id) => {
                errors.push(format!("部門ID {} が存在しません", id));
            }
            _ => {}
        }

        let selling_price = row.fields.get("売価").and_then(|v| v.parse::<i64>().ok());
        match selling_price {
            None => errors.push("売価が不正です".to_string()),
            Some(p) => errors.extend(price_range_error("売価", p, MAX_SELLING_PRICE)),
        }

        // 原価: 小数 2 桁までの円を 1/100 円にする（SPEC-UNIT-D5・D10、Y2）
        let cost_price_centi = match parse_cost_cell(
            row.fields
                .get("原価")
                .map(String::as_str)
                .unwrap_or_default(),
        ) {
            Ok(cost) => Some(cost),
            Err(message) => {
                errors.push(message);
                None
            }
        };

        let tax_rate = row.fields.get("税率").cloned().unwrap_or_default();
        if !["10", "8", "0"].contains(&tax_rate.as_str()) {
            errors.push("税率は '10', '8', '0' のいずれかで入力してください".to_string());
        }

        // オプション項目（R2-5: None のまま保持、デフォルト化しない）
        // #3 (P2): 値あり+parse失敗をサイレント None にせずエラー化
        // 在庫単位: code か表示の語を code に正規化する。空は None（INSERT で pcs）。SPEC-UNIT-D10
        let raw_stock_unit = row
            .fields
            .get("在庫単位")
            .map(String::as_str)
            .unwrap_or_default();
        let unit = normalize_stock_unit(raw_stock_unit);
        let unit_invalid = !raw_stock_unit.is_empty() && unit.is_none();
        if unit_invalid {
            errors.push(stock_unit_invalid_message(raw_stock_unit));
        }
        let stock_unit = unit.map(|unit| unit.as_str().to_string());

        // 初期在庫: 0 以上で在庫の上限まで。m の行だけ m の数として cm にする（SPEC-UNIT-D3、X3）。
        // 単位が決まらない行では読まない
        let initial_stock = match row.fields.get("初期在庫") {
            Some(v) if !v.is_empty() && !unit_invalid => {
                let parsed = if unit == Some(ProductStockUnit::M) {
                    parse_hundredths(v)
                } else {
                    v.parse::<i64>().ok().filter(|n| *n >= 0).map(Some)
                };
                match parsed {
                    Some(Some(n)) if n <= MAX_STOCK_QUANTITY => Some(n),
                    Some(_) => {
                        errors.push(MSG_STOCK_TOO_LARGE.to_string());
                        None
                    }
                    None => {
                        errors.push(format!("初期在庫の値が不正です: '{}'", v));
                        None
                    }
                }
            }
            _ => None,
        };

        let jan_code =
            row.fields
                .get("JANコード")
                .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) });
        // BIZ-01-D10: 重複の分類より前なので上書きの候補の行にも当たる
        errors.extend(import_code_shape_errors(&product_code, jan_code.as_deref()));
        let maker_code = row.fields.get("メーカー品番").and_then(|v| {
            if v.is_empty() {
                None
            } else {
                Some(v.clone())
            }
        });

        let supplier_id = match row.fields.get("取引先ID") {
            Some(v) if !v.is_empty() => match v.parse::<i64>() {
                Ok(n) => Some(n),
                Err(_) => {
                    errors.push(format!("取引先IDの値が不正です: '{}'", v));
                    None
                }
            },
            _ => None,
        };

        let pos_stock_sync = match row.fields.get("POS在庫連動") {
            Some(v) if !v.is_empty() => match v.as_str() {
                "1" | "true" | "はい" => Some(true),
                "0" | "false" | "いいえ" => Some(false),
                _ => {
                    errors.push(format!(
                        "POS在庫連動の値が不正です: '{}' (1/0/true/false)",
                        v
                    ));
                    None
                }
            },
            _ => None,
        };

        let mut warnings = Vec::new();
        let plu_target = match row.fields.get("PLU対象") {
            Some(value) if !value.is_empty() => match value.as_str() {
                "1" if is_plu_target_eligible_jan(jan_code.as_deref()) => Some(true),
                "1" => {
                    warnings.push("JAN が13桁でないため対象外として取り込みます".to_string());
                    Some(false)
                }
                "0" => Some(false),
                _ => {
                    errors.push(format!("PLU対象の値が不正です: '{}' (1/0/空欄)", value));
                    None
                }
            },
            _ => None,
        };

        // supplier_id 存在確認（R2-6対応）
        if let Some(sid) = supplier_id {
            if !supplier_ids.contains(&sid) {
                errors.push(format!("取引先ID {} が存在しません", sid));
            }
        }

        // 既存の商品の行（上書きの候補）は個数の単位どうしでだけ単位を変えられる（SPEC-UNIT-D2・BIZ-01-D8）
        let existing = if product_code.is_empty() {
            None
        } else {
            product_repo::find_by_product_code(conn, &product_code)?
        };
        if let (Some(existing), Some(unit)) = (&existing, unit) {
            if !stock_unit_change_allowed(existing.product.stock_unit, unit) {
                errors.push(stock_unit_changed_message(
                    existing.product.stock_unit,
                    raw_stock_unit,
                ));
            }
        }

        if !errors.is_empty() {
            error_rows.push(ImportErrorRow {
                line_no,
                raw_data: row.fields.clone(),
                errors,
            });
            continue;
        }

        let import_row = ImportRow {
            line_no,
            product_code: product_code.clone(),
            name,
            department_id: department_id.unwrap(),
            selling_price: selling_price.unwrap(),
            cost_price_centi: cost_price_centi.unwrap(),
            tax_rate,
            stock_unit,
            initial_stock,
            jan_code,
            maker_code,
            supplier_id,
            pos_stock_sync,
            plu_target,
            warnings,
        };

        // 4. 重複チェック
        match existing {
            Some(_) => {
                duplicate_rows.push(ImportDuplicateRow {
                    line_no,
                    import_row,
                    existing_product_code: product_code,
                });
            }
            None => {
                valid_rows.push(import_row);
            }
        }
    }

    // 5. 共有 JAN と在庫連動を始める検査（30）: 新規の行は CSV の順に先の新規の行を当てた状態で、
    // 重複の行は全ての新規の行を当てた状態で判定し、当たる行だけを error 行にする
    let raw_data = |line_no: usize| {
        parse_result
            .rows
            .iter()
            .find(|row| row.line_no == line_no)
            .map(|row| row.fields.clone())
            .unwrap_or_default()
    };
    let mut overlay = JanOverlay::new();
    let mut linked_valid_rows = Vec::with_capacity(valid_rows.len());
    for row in valid_rows {
        match import_row_link_error(conn, &row, None, &overlay)? {
            Some(message) => error_rows.push(ImportErrorRow {
                line_no: row.line_no,
                raw_data: raw_data(row.line_no),
                errors: vec![message],
            }),
            None => {
                overlay.insert(
                    row.product_code.clone(),
                    (row.jan_code.clone(), row.pos_stock_sync.unwrap_or(true)),
                );
                linked_valid_rows.push(row);
            }
        }
    }
    let mut linked_duplicate_rows = Vec::with_capacity(duplicate_rows.len());
    for duplicate in duplicate_rows {
        let existing = product_repo::find_by_product_code(conn, &duplicate.existing_product_code)?;
        match import_row_link_error(
            conn,
            &duplicate.import_row,
            existing.as_ref().map(|e| &e.product),
            &overlay,
        )? {
            Some(message) => error_rows.push(ImportErrorRow {
                line_no: duplicate.line_no,
                raw_data: raw_data(duplicate.line_no),
                errors: vec![message],
            }),
            None => linked_duplicate_rows.push(duplicate),
        }
    }

    Ok(ImportPreview {
        valid_rows: linked_valid_rows,
        error_rows,
        duplicate_rows: linked_duplicate_rows,
    })
}

/// プレビュー済みの行をDBに一括登録する（TX）
///
/// 30-biz-product-service.md §4.8
/// create_product / update_product のBIZ関数はネストTXを発生させるため、
/// 直接 repo 関数を呼ぶ。price_history は記録しない。
pub fn commit_import(
    conn: &mut DbConnection,
    valid_rows: Vec<ImportRow>,
    overwrite_codes: Vec<String>,
) -> Result<ProductImportResult, BizError> {
    // BIZ-01-D5: wire から来る行を信頼せず、TX を開く前に商品コードの上限を再検証する
    if valid_rows
        .iter()
        .any(|row| product_code_exceeds_max_len(&row.product_code))
    {
        return Err(BizError::ValidationFailed(
            PRODUCT_CODE_TOO_LONG_MESSAGE.to_string(),
        ));
    }
    // BIZ-01-D10: 商品コードと JANコードの形も TX の前に。最初に当たった行の最初の文
    if let Some(message) = valid_rows.iter().find_map(|row| {
        import_code_shape_errors(&row.product_code, row.jan_code.as_deref())
            .into_iter()
            .next()
    }) {
        return Err(BizError::ValidationFailed(message));
    }

    let overwrite_set: std::collections::HashSet<String> = overwrite_codes.into_iter().collect();

    // 1. TX開始
    let tx = conn
        .transaction()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    let mut created_count = 0usize;
    let mut updated_count = 0usize;
    let mut skipped_count = 0usize;

    // 進行中棚卸しのチェック（TX内で1回だけ）
    let active_stocktake = stocktake_repo::find_active_stocktake(&tx)?;

    for row in &valid_rows {
        if overwrite_set.contains(&row.product_code) {
            let existing = product_repo::find_by_product_code(&tx, &row.product_code)?;
            // 上書きの対象が無い行は下で skip する（書かない）ので、再検証は対象がある行だけ
            if let Some(message) = existing.as_ref().and_then(|existing| {
                import_row_commit_error(row, Some(existing.product.stock_unit))
            }) {
                return Err(BizError::ValidationFailed(message));
            }
            // 共有 JAN と在庫連動を始める検査（TX のその時点の状態で、書く前に。30）
            if let Some(existing) = existing.as_ref() {
                if let Some(message) =
                    import_row_link_error(&tx, row, Some(&existing.product), &JanOverlay::new())?
                {
                    return Err(import_commit_link_error(&row.product_code, &message));
                }
            }
            let toggles_link = existing.as_ref().is_some_and(|existing| {
                row.pos_stock_sync
                    .is_some_and(|linked| linked != existing.product.pos_stock_sync)
            });
            let old_jan = existing
                .as_ref()
                .and_then(|existing| existing.product.jan_code.clone());
            let was_plu_target = existing
                .as_ref()
                .is_some_and(|existing| existing.product.plu_target);
            // 単位の変更（前, 後）。再検証を通った行なので個数の単位どうし（BIZ-01-D8）
            let unit_change = existing.as_ref().and_then(|existing| {
                let unit = row
                    .stock_unit
                    .as_deref()
                    .and_then(ProductStockUnit::from_code)?;
                (unit != existing.product.stock_unit).then_some((existing.product.stock_unit, unit))
            });
            // UPDATE: ImportRow → ProductUpdates マッピング（R2-1 + R3-1修正）
            let mut updates = ProductUpdates {
                jan_code: Some(row.jan_code.clone()),
                name: Some(row.name.clone()),
                department_id: Some(row.department_id),
                selling_price: Some(row.selling_price),
                cost_price_centi: Some(row.cost_price_centi),
                tax_rate: Some(row.tax_rate.clone()),
                supplier_id: row.supplier_id.map(Some),
                maker_code: row.maker_code.clone().map(Some),
                pos_stock_sync: row.pos_stock_sync,
                stock_unit: unit_change.map(|(_, new)| new),
                plu_dirty: Some(true),
                is_discontinued: None,
                plu_exported_at: None,
                plu_target: row.plu_target,
            };
            // 単位だけの上書きは plu_dirty を変えない（BIZ-01-D8）
            if existing
                .as_ref()
                .is_some_and(|existing| overwrite_changes_unit_only(&updates, &existing.product))
            {
                updates.plu_dirty = None;
            }
            // #4 (P2): 戻り値チェック。対象が存在しない場合はスキップ
            let updated = product_repo::update_product(&tx, &row.product_code, &updates)?;
            if updated && toggles_link {
                // 連動の切替はその商品を求め直す（値は変わらない。30）
                recompute_stock(&tx, &row.product_code)?;
            }
            if updated {
                let target_turned_off = was_plu_target && row.plu_target == Some(false);
                if target_turned_off || old_jan != row.jan_code {
                    if let Some(old_jan) = old_jan.as_deref() {
                        super::plu_export_service::release_plu_slot_for_jan(&tx, old_jan)?;
                    }
                }
                // 単位を変えた商品ごとに TX の中でログを書く。失敗すれば TX 全体を戻す（BIZ-01-D8）
                if let Some((old, new)) = unit_change {
                    let detail = serde_json::json!({
                        "stock_unit": { "old": old.as_str(), "new": new.as_str() }
                    });
                    system_repo::insert_operation_log(
                        &tx,
                        &NewOperationLog {
                            operation_type: "product_update".to_string(),
                            summary: format!("商品を更新しました: {}", row.product_code),
                            detail_json: Some(detail.to_string()),
                        },
                    )?;
                }
                updated_count += 1;
            } else {
                skipped_count += 1;
            }
        } else {
            if let Some(message) = import_row_commit_error(row, None) {
                return Err(BizError::ValidationFailed(message));
            }
            if let Some(message) = import_row_link_error(&tx, row, None, &JanOverlay::new())? {
                return Err(import_commit_link_error(&row.product_code, &message));
            }
            // INSERT: デフォルト化は INSERT 時のみ（R2-5対応）
            let stock_unit = row.stock_unit.clone().unwrap_or_else(|| "pcs".to_string());
            let initial_stock = row.initial_stock.unwrap_or(0);
            let pos_stock_sync = row.pos_stock_sync.unwrap_or(true);

            let new_product = NewProduct {
                product_code: row.product_code.clone(),
                jan_code: row.jan_code.clone(),
                name: row.name.clone(),
                department_id: row.department_id,
                supplier_id: row.supplier_id,
                selling_price: row.selling_price,
                cost_price_centi: row.cost_price_centi,
                tax_rate: row.tax_rate.clone(),
                maker_code: row.maker_code.clone(),
                stock_quantity: initial_stock,
                stock_unit,
                is_discontinued: false,
                plu_dirty: true,
                plu_exported_at: None,
                plu_target: row
                    .plu_target
                    .unwrap_or_else(|| should_default_plu_target(row.jan_code.as_deref())),
                pos_stock_sync,
            };
            product_repo::insert_product(&tx, &new_product)?;

            // 初期在庫 > 0 → movement 記録
            if initial_stock > 0 {
                inventory_repo::insert_movement(
                    &tx,
                    &NewMovement {
                        product_code: row.product_code.clone(),
                        movement_type: MovementType::Receiving,
                        quantity: initial_stock,
                        stock_after: initial_stock,
                        reference_type: None,
                        reference_id: None,
                        note: Some("一括インポートによる初期在庫".to_string()),
                        business_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                        settlement_no: None,
                    },
                )?;
            }
            // 写しは初期在庫の movement から求め直す（30）
            recompute_stock(&tx, &row.product_code)?;

            // 進行中棚卸し → stocktake_item 自動追加
            if let Some(ref st) = active_stocktake {
                stocktake_repo::insert_stocktake_item(
                    &tx,
                    &NewStocktakeItem {
                        stocktake_id: st.id,
                        product_code: row.product_code.clone(),
                        system_stock: initial_stock,
                        actual_count: None,
                    },
                )?;
            }

            created_count += 1;
        }
    }

    // 3. COMMIT
    tx.commit()
        .map_err(|e| BizError::DatabaseError(DbError::from(e)))?;

    // 4. TX外: 操作ログ記録
    let detail = serde_json::json!({
        "created_count": created_count,
        "updated_count": updated_count,
    })
    .to_string();
    let log = NewOperationLog {
        operation_type: "product_import".to_string(),
        summary: format!(
            "商品一括インポート: 新規{}件、更新{}件",
            created_count, updated_count
        ),
        detail_json: Some(detail),
    };
    if let Err(e) = system_repo::insert_operation_log(conn, &log) {
        tracing::warn!(error = %e, "操作ログ記録に失敗");
    }

    Ok(ProductImportResult {
        created_count,
        updated_count,
        skipped_count,
    })
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_database;
    use serial_test::serial;

    fn setup_test_db() -> (tempfile::TempDir, DbConnection) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = init_database(db_path.to_str().unwrap()).unwrap();
        (dir, conn)
    }

    fn seed_named_supplier(conn: &DbConnection, name: &str) -> i64 {
        product_repo::find_or_create_supplier(conn, name)
            .unwrap()
            .id
    }

    fn seed_supplier_product(conn: &DbConnection, code: &str, supplier_id: i64) {
        product_repo::insert_product(
            conn,
            &NewProduct {
                product_code: code.to_string(),
                jan_code: None,
                name: format!("テスト商品{code}"),
                department_id: 1,
                supplier_id: Some(supplier_id),
                selling_price: 500,
                cost_price_centi: 300,
                tax_rate: "10".to_string(),
                maker_code: None,
                stock_quantity: 0,
                stock_unit: "pcs".to_string(),
                is_discontinued: false,
                plu_dirty: true,
                plu_exported_at: None,
                plu_target: true,
                pos_stock_sync: true,
            },
        )
        .unwrap();
    }

    fn seed_supplier_receiving(conn: &DbConnection, key: &str, supplier_id: i64) {
        conn.execute(
            "INSERT INTO receiving_records
             (supplier_id,receiving_date,note,idempotency_key,request_fingerprint,created_at)
             VALUES(?1,'2026-08-25',NULL,?2,?2,'2026-08-25T10:00:00')",
            rusqlite::params![supplier_id, key],
        )
        .unwrap();
    }

    fn synthetic_ean13_req907(data: u64) -> String {
        let body = format!("{:012}", data % 1_000_000_000_000);
        let sum: u32 = body
            .bytes()
            .enumerate()
            .map(|(index, byte)| u32::from(byte - b'0') * if index % 2 == 0 { 1 } else { 3 })
            .sum();
        format!("{}{}", body, (10 - sum % 10) % 10)
    }

    fn insert_bulk_product_req907(
        conn: &DbConnection,
        code: &str,
        name: &str,
        jan_code: Option<&str>,
        department_id: i64,
        state: (bool, bool, bool),
    ) {
        let (is_discontinued, plu_target, plu_dirty) = state;
        product_repo::insert_product(
            conn,
            &NewProduct {
                product_code: code.to_string(),
                jan_code: jan_code.map(str::to_string),
                name: name.to_string(),
                department_id,
                supplier_id: None,
                selling_price: 100,
                cost_price_centi: 50,
                tax_rate: "10".to_string(),
                maker_code: None,
                stock_quantity: 0,
                stock_unit: "pcs".to_string(),
                is_discontinued,
                plu_dirty,
                plu_exported_at: None,
                plu_target,
                pos_stock_sync: true,
            },
        )
        .unwrap();
    }

    fn bulk_filter_req907(keyword: &str) -> ProductBulkFilter {
        ProductBulkFilter {
            keyword: Some(keyword.to_string()),
            department_id: Some(2),
            is_discontinued: None,
            plu: Some(crate::db::product_repo::PluMigrationFilter::All),
        }
    }

    fn default_create_request() -> ProductCreateRequest {
        ProductCreateRequest {
            jan_code: None,
            name: "テスト商品".to_string(),
            department_id: 2, // ヘア雑貨（code_prefix=HZ）
            selling_price: 500,
            cost_price_centi: 300,
            tax_rate: ProductTaxRate::Rate10,
            stock_unit: ProductStockUnit::Pcs,
            initial_stock: 0,
            maker_code: None,
            supplier_id: None,
            pos_stock_sync: true,
            plu_target: false,
            shared_jan_custom_code: false,
        }
    }

    // ===== FUNC-4.3: generate_custom_code テスト =====

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_normal() {
        // REQ-101: 独自コード発番 — 正常発番（HZ-0001 形式）
        // FUNC-4.3: 正常発番 — HZ-0001 形式
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        let code = generate_custom_code(&tx, 2).unwrap(); // ヘア雑貨
        assert_eq!(code, "HZ-0001");
        tx.commit().unwrap();
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_no_prefix() {
        // REQ-101: 独自コード発番 — code_prefix=NULLの部門はValidationFailed
        // FUNC-4.3: code_prefix=NULL の部門 → ValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        let result = generate_custom_code(&tx, 3); // 毛糸（code_prefix=NULL）
        assert!(
            matches!(result, Err(BizError::ValidationFailed(_))),
            "code_prefix=NULLで ValidationFailed: {:?}",
            result
        );
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_sequential() {
        // REQ-101: 独自コード発番 — 連番インクリメント（HZ-0001, HZ-0002）
        // FUNC-4.3: 連番 — 2回呼んで HZ-0001, HZ-0002
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        let code1 = generate_custom_code(&tx, 2).unwrap();
        // code1 の商品を INSERT して重複チェックを通す
        let p = NewProduct {
            product_code: code1.clone(),
            jan_code: None,
            name: "テスト1".to_string(),
            department_id: 2,
            supplier_id: None,
            selling_price: 100,
            cost_price_centi: 50,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: 0,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(&tx, &p).unwrap();
        let code2 = generate_custom_code(&tx, 2).unwrap();
        assert_eq!(code1, "HZ-0001");
        assert_eq!(code2, "HZ-0002");
        tx.commit().unwrap();
    }

    #[test]
    fn test_generate_custom_code_req101_requires_borrowed_transaction() {
        // REQ-101 / FUNC-4.3: 連番の更新を TX の外へ漏らさないため、第 1 引数は借りた transaction に限る
        // （31 §12.2 の apply_stock_change と同じ形。通常の接続に戻すとここでコンパイルが止まる）
        let _ =
            generate_custom_code as fn(&rusqlite::Transaction<'_>, i64) -> Result<String, BizError>;
    }

    #[test]
    #[serial]
    fn test_create_product_req101_takkyubin_rejects_custom_code() {
        // REQ-101 / FUNC-4.3 処理ステップ 2: 宅急便（部門 12、master-tables §2 で code_prefix=NULL）は
        // JAN 空の登録で独自コードを発番せず、書込みの前に ValidationFailed で止まる
        let (_dir, mut conn) = setup_test_db();
        let req = ProductCreateRequest {
            department_id: 12,
            ..default_create_request()
        };

        let result = create_product(&mut conn, req);
        match result {
            Err(BizError::ValidationFailed(msg)) => {
                assert_eq!(msg, "この部門は独自コード発番に対応していません");
            }
            other => panic!("宅急便の JAN 空の登録は ValidationFailed であるべき: {other:?}"),
        }

        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |row| row.get(0)).unwrap() };
        assert_eq!(count("SELECT COUNT(*) FROM products"), 0);
        assert_eq!(count("SELECT COUNT(*) FROM inventory_movements"), 0);
        assert_eq!(count("SELECT next_seq FROM departments WHERE id = 12"), 1);
    }

    // ===== FUNC-4.2: create_product テスト =====

    #[test]
    #[serial]
    fn test_create_product_req101_jan() {
        // REQ-101: JAN有りで正常登録
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000015".to_string());
        req.name = "ハマナカ アミアミ極太".to_string();
        req.department_id = 3; // 毛糸

        let result = create_product(&mut conn, req).unwrap();
        assert_eq!(result.product_code, "2000000000015");
    }

    #[test]
    #[serial]
    fn test_create_product_req101_custom_code() {
        // REQ-101: JAN無しで独自コード発番
        let (_dir, mut conn) = setup_test_db();
        let req = default_create_request(); // jan_code=None, department=ヘア雑貨

        let result = create_product(&mut conn, req).unwrap();
        assert_eq!(result.product_code, "HZ-0001");
    }

    #[test]
    #[serial]
    fn test_create_product_req101_stores_plu_target_from_request() {
        // REQ-101 / REQ-402: 商品登録時のPLU対象フラグはrequest値を保存する。
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000022".to_string());
        req.department_id = 3;
        req.plu_target = true;

        let result = create_product(&mut conn, req).unwrap();

        let found = product_repo::find_by_product_code(&conn, &result.product_code)
            .unwrap()
            .unwrap();
        assert!(found.product.plu_target);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_initial_stock() {
        // REQ-101: 初期在庫ありで inventory_movements が記録される
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.initial_stock = 10;

        let result = create_product(&mut conn, req).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM inventory_movements WHERE product_code = ?1",
                rusqlite::params![result.product_code],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "初期在庫の inventory_movement が記録されるべき");
    }

    type CopyAndInitialMovements = (
        (i64, bool, Option<i64>, Option<i64>),
        Vec<(i64, String, i64)>,
    );

    /// 写しの 4 項目と、初期在庫の movement（reference_type NULL）の (数量, 業務日, stock_after) の一覧
    fn copy_and_initial_movements(conn: &DbConnection, code: &str) -> CopyAndInitialMovements {
        let copy = conn
            .query_row(
                "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
                 FROM products WHERE product_code = ?1",
                [code],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT quantity, business_date, stock_after FROM inventory_movements
                 WHERE product_code = ?1 AND reference_type IS NULL ORDER BY id",
            )
            .unwrap();
        let movements = stmt
            .query_map([code], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        (copy, movements)
    }

    #[test]
    #[serial]
    fn test_create_product_req101_initial_stock_movement_and_copy_match() {
        // REQ-101 / 30（初期在庫の movement と求め直し）/ SPEC-STK-CNT-D2 / Matrix E3
        let (_dir, mut conn) = setup_test_db();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut req = default_create_request();
        req.initial_stock = 5;
        let created = create_product(&mut conn, req).unwrap();
        assert_eq!(
            copy_and_initial_movements(&conn, &created.product_code),
            ((5, false, None, None), vec![(5, today, 5)])
        );

        let mut req = default_create_request();
        req.initial_stock = 0;
        let created = create_product(&mut conn, req).unwrap();
        assert_eq!(
            copy_and_initial_movements(&conn, &created.product_code),
            ((0, false, None, None), vec![])
        );
    }

    #[test]
    #[serial]
    fn test_create_product_req101_stocktake_auto_add() {
        // REQ-101: 棚卸し中に商品登録で stocktake_items 自動追加
        let (_dir, mut conn) = setup_test_db();

        // 棚卸し開始（直接SQL）
        conn.execute(
            "INSERT INTO stocktakes (started_at, status) VALUES ('2026-10-01T09:00:00', 'in_progress')",
            [],
        )
        .unwrap();

        let req = default_create_request();
        let result = create_product(&mut conn, req).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM stocktake_items WHERE product_code = ?1",
                rusqlite::params![result.product_code],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "棚卸し中は stocktake_items に自動追加");
    }

    #[test]
    #[serial]
    fn test_create_product_req101_operation_log() {
        // REQ-101: operation_logs に product_create が記録される
        let (_dir, mut conn) = setup_test_db();
        let req = default_create_request();
        create_product(&mut conn, req).unwrap();

        let op_type: String = conn
            .query_row(
                "SELECT operation_type FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(op_type, "product_create");
    }

    #[test]
    #[serial]
    fn test_create_product_req101_validation_name_empty() {
        // REQ-101: 商品登録 — 商品名空はValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.name = "".to_string();
        let result = create_product(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    #[serial]
    fn test_create_product_req101_validation_selling_price_negative() {
        // REQ-101: 商品登録 — 売価マイナスはValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.selling_price = -1;
        let result = create_product(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    #[serial]
    fn test_create_product_req101_validation_cost_price_negative() {
        // REQ-101: 商品登録 — 原価マイナスはValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.cost_price_centi = -1;
        let result = create_product(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    #[serial]
    fn test_create_product_req101_validation_department_not_found() {
        // REQ-101: 商品登録 — 存在しない部門IDはValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.department_id = 9999;
        let result = create_product(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    #[serial]
    fn test_create_product_req101_validation_supplier_not_found() {
        // REQ-101: 商品登録 — 存在しない取引先IDはValidationFailed
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.supplier_id = Some(9999);
        let result = create_product(&mut conn, req);
        assert!(matches!(result, Err(BizError::ValidationFailed(_))));
    }

    #[test]
    #[serial]
    fn test_create_product_req101_accepts_valid_jan_8_and_jan_13() {
        for jan_code in ["4901234567887", "96385074", "49123456"] {
            let (_dir, mut conn) = setup_test_db();
            let mut req = default_create_request();
            req.jan_code = Some(jan_code.to_string());
            req.department_id = 3;

            let result = create_product(&mut conn, req).unwrap();
            assert_eq!(result.product_code, jan_code);
        }
    }

    #[test]
    #[serial]
    fn test_create_product_req101_rejects_invalid_jan_wire_with_exact_message() {
        for jan_code in ["123456789012", "490123456788A"] {
            let (_dir, mut conn) = setup_test_db();
            let mut req = default_create_request();
            req.jan_code = Some(jan_code.to_string());
            req.department_id = 3;

            let result = create_product(&mut conn, req);
            assert!(matches!(
                result,
                Err(BizError::ValidationFailed(ref message))
                    if message == "JANコードは13桁または8桁で入力してください"
            ));
        }

        for jan_code in ["4901234567890", "49123457"] {
            let (_dir, mut conn) = setup_test_db();
            let mut req = default_create_request();
            req.jan_code = Some(jan_code.to_string());
            req.department_id = 3;

            let result = create_product(&mut conn, req);
            assert!(matches!(
                result,
                Err(BizError::ValidationFailed(ref message))
                    if message
                        == "JANコードのチェックディジットが一致しません。入力値を確認してください"
            ));
        }
    }

    #[test]
    #[serial]
    fn test_create_product_req101_does_not_normalize_fullwidth_or_spaced_jan_wire() {
        for jan_code in ["４９０１２３４５６７８８７", " 4901234567887 "] {
            let (_dir, mut conn) = setup_test_db();
            let mut req = default_create_request();
            req.jan_code = Some(jan_code.to_string());
            req.department_id = 3;

            let result = create_product(&mut conn, req);
            assert!(matches!(
                result,
                Err(BizError::ValidationFailed(ref message))
                    if message == "JANコードは13桁または8桁で入力してください"
            ));
        }
    }

    #[test]
    fn test_should_default_plu_target_req402_uses_normalized_ascii_ean_13_domain() {
        let cases = [
            (None, false),
            (Some("4901234567887"), true),
            (Some("96385074"), false),
            (Some("４９０１２３４５６７８８７"), false),
            (Some(" 4901234567887 "), false),
            (Some(" ４９０１２３４５６７８８７ "), false),
            (Some("490123456788A"), false),
            (Some("123456789012"), false),
            (Some("12345678901234"), false),
        ];

        for (jan_code, expected) in cases {
            assert_eq!(
                should_default_plu_target(jan_code),
                expected,
                "jan={jan_code:?}"
            );
        }
    }

    #[test]
    fn test_should_default_plu_target_req907_preserves_invalid_check_digit_compatibility() {
        assert!(should_default_plu_target(Some("2000000000038")));
    }

    #[test]
    #[serial]
    fn test_create_product_req101_duplicate_jan() {
        // REQ-101 / BIZ-01-D11: 商品登録 — 同じ JAN の 2 つ目は JanInUse で、使っている商品を名指す
        // （旧い期待の DuplicateProductCode は BIZ-01-D11 で置き換えた。D-141）
        let (_dir, mut conn) = setup_test_db();
        let mut req1 = default_create_request();
        req1.jan_code = Some("2000000000039".to_string());
        req1.department_id = 3;
        create_product(&mut conn, req1).unwrap();

        let mut req2 = default_create_request();
        req2.jan_code = Some("2000000000039".to_string());
        req2.department_id = 3;
        let result = create_product(&mut conn, req2);
        match result {
            Err(BizError::JanInUse(message)) => assert_eq!(
                message,
                "この JAN（2000000000039）はほかの商品が使っています: 2000000000039「テスト商品」（POS販売で在庫を減らす）"
            ),
            other => panic!("JanInUse を期待しました: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn test_create_product_req101_skips_directly_inserted_custom_code() {
        // REQ-101: 商品登録 — 直接入った独自コード HZ-0001 を飛ばして HZ-0002 を振る（BIZ-01-D6）
        // 旧前提（既存の HZ-0001 で DuplicateProductCode）は BIZ-01-D6 で廃止。INSERT 時の DuplicateKey の正規化には元から届いていなかった
        let (_dir, mut conn) = setup_test_db();

        // 直接 IO 層で商品を挿入（BIZ 層の重複チェックをバイパス）
        let p = NewProduct {
            product_code: "HZ-0001".to_string(),
            jan_code: None,
            name: "先行登録".to_string(),
            department_id: 2,
            supplier_id: None,
            selling_price: 100,
            cost_price_centi: 50,
            tax_rate: "10".to_string(),
            maker_code: None,
            stock_quantity: 0,
            stock_unit: "pcs".to_string(),
            is_discontinued: false,
            plu_dirty: true,
            plu_exported_at: None,
            plu_target: true,
            pos_stock_sync: true,
        };
        product_repo::insert_product(&conn, &p).unwrap();

        // department=2(HZ), next_seq=1 でも既存の HZ-0001 を飛ばす
        let result = create_product(&mut conn, default_create_request()).unwrap();
        assert_eq!(result.product_code, "HZ-0002");
    }

    // ===== BIZ-01-D6: 既存の独自コードを飛ばす発番 =====

    fn seed_code(conn: &DbConnection, code: &str, department_id: i64) {
        product_repo::insert_product(
            conn,
            &NewProduct {
                product_code: code.to_string(),
                jan_code: None,
                name: "合成".to_string(),
                department_id,
                supplier_id: None,
                selling_price: 100,
                cost_price_centi: 50,
                tax_rate: "10".to_string(),
                maker_code: None,
                stock_quantity: 0,
                stock_unit: "pcs".to_string(),
                is_discontinued: false,
                plu_dirty: true,
                plu_exported_at: None,
                plu_target: true,
                pos_stock_sync: true,
            },
        )
        .unwrap();
    }

    fn next_seq_of(conn: &DbConnection, department_id: i64) -> i64 {
        product_repo::find_department_by_id(conn, department_id)
            .unwrap()
            .unwrap()
            .next_seq
    }

    fn product_count(conn: &DbConnection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
            .unwrap()
    }

    fn assert_seq_overflow_unchanged(conn: &mut DbConnection, expected_next_seq: i64) {
        let tx = conn.transaction().unwrap();
        let before = product_count(&tx);
        let result = generate_custom_code(&tx, 2);
        match result {
            Err(BizError::ValidationFailed(msg)) => {
                assert_eq!(msg, "この部門の独自コードの番号を振れません")
            }
            other => panic!("ValidationFailed であるべき: {other:?}"),
        }
        assert_eq!(product_count(&tx), before);
        assert_eq!(next_seq_of(&tx, 2), expected_next_seq);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_custom_code_after_csv_import() {
        // REQ-101 / REQ-104: T1 商品 CSV で HZ-0001・HZ-0002 を取り込んだ後も登録できる（BIZ-01-D6、§4.9 step 2c）
        let (_dir, mut conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &["HZ-0001,合成A,2,500,300,10", "HZ-0002,合成B,2,500,300,10"],
        );
        let preview = preview_import(&conn, &csv).unwrap();
        assert_eq!(preview.valid_rows.len(), 2);
        commit_import(&mut conn, preview.valid_rows, vec![]).unwrap();
        assert_eq!(next_seq_of(&conn, 2), 1, "取込みは next_seq を進めない");

        let r1 = create_product(&mut conn, default_create_request()).unwrap();
        assert_eq!(r1.product_code, "HZ-0003");
        let r2 = create_product(&mut conn, default_create_request()).unwrap();
        assert_eq!(r2.product_code, "HZ-0004");
        assert_eq!(next_seq_of(&conn, 2), 5);
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_skips_gap_after_max() {
        // REQ-101: T2 抜けた番号を使わず最大の次から振る（BIZ-01-D6、TD-108）
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        seed_code(&tx, "HZ-0001", 2);
        seed_code(&tx, "HZ-0005", 2);
        assert_eq!(generate_custom_code(&tx, 2).unwrap(), "HZ-0006");
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_keeps_next_seq_above_max() {
        // REQ-101: T3 next_seq が最大 + 1 より大きければ next_seq を使う（下げない）
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        tx.execute("UPDATE departments SET next_seq = 10 WHERE id = 2", [])
            .unwrap();
        seed_code(&tx, "HZ-0003", 2);
        assert_eq!(generate_custom_code(&tx, 2).unwrap(), "HZ-0010");
        assert_eq!(next_seq_of(&tx, 2), 11);
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_ignores_non_numbered_codes() {
        // REQ-101: T4 番号として数えるのは `HZ-` + ASCII 数字だけで i64 に収まるもの（BIZ-01-D6 step 3b）
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        for code in [
            "HZ-00A1",
            "HZ-0009X",
            "HZ-",
            "hz-0009",
            "HZX-0009",
            "SY-0009",
            "HZ-+0009",
            "HZ-99999999999999999999",
        ] {
            seed_code(&tx, code, 2);
        }
        assert_eq!(generate_custom_code(&tx, 2).unwrap(), "HZ-0001");
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_counts_codes_in_other_departments() {
        // REQ-101: T5 別の部門の ID で入った HZ- のコードも飛ばす（部門を問わない）
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        seed_code(&tx, "HZ-0007", 1);
        assert_eq!(generate_custom_code(&tx, 2).unwrap(), "HZ-0008");
    }

    #[test]
    #[serial]
    fn test_create_product_req101_rollback_restores_raised_next_seq() {
        // REQ-101: T6 登録の失敗で next_seq の引上げも戻り、再試行で同じ番号（§4.2 / §4.3 の注）
        let (_dir, mut conn) = setup_test_db();
        seed_code(&conn, "HZ-0003", 2);
        {
            let _guard = failpoint::arm(&failpoint::CREATE_PRODUCT_AFTER_INSERT);
            assert!(create_product(&mut conn, default_create_request()).is_err());
        }
        assert_eq!(next_seq_of(&conn, 2), 1, "引上げも rollback で戻る");
        let result = create_product(&mut conn, default_create_request()).unwrap();
        assert_eq!(result.product_code, "HZ-0004");
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_rejects_seq_overflow() {
        // REQ-101: T7a 既存の番号の最大が i64::MAX → 更新の前に ValidationFailed（step 3c）
        let (_dir, mut conn) = setup_test_db();
        seed_code(&conn, "HZ-9223372036854775807", 2);
        assert_seq_overflow_unchanged(&mut conn, 1);
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_rejects_overflow_after_max() {
        // REQ-101: T7b 最大の次が i64::MAX → cand + 1 が溢れるので更新の前に ValidationFailed（step 3d）
        let (_dir, mut conn) = setup_test_db();
        seed_code(&conn, "HZ-9223372036854775806", 2);
        assert_seq_overflow_unchanged(&mut conn, 1);
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_rejects_overflow_from_next_seq() {
        // REQ-101: T7c next_seq が i64::MAX → 更新の前に ValidationFailed（step 3d）
        let (_dir, mut conn) = setup_test_db();
        conn.execute(
            "UPDATE departments SET next_seq = ?1 WHERE id = 2",
            [i64::MAX],
        )
        .unwrap();
        assert_seq_overflow_unchanged(&mut conn, i64::MAX);
    }

    #[test]
    #[serial]
    fn test_generate_custom_code_req101_continues_past_9999() {
        // REQ-101: T8 HZ-9999 の次は HZ-10000（C-1、TD-109）
        let (_dir, mut conn) = setup_test_db();
        let tx = conn.transaction().unwrap();
        seed_code(&tx, "HZ-9999", 2);
        assert_eq!(generate_custom_code(&tx, 2).unwrap(), "HZ-10000");
        assert_eq!(next_seq_of(&tx, 2), 10001);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_rollback_after_insert() {
        // REQ-101: 商品登録 — INSERT後failpointでロールバック → productsに残らない
        // TX rollback: INSERT 後に failpoint で失敗 → products に残らない
        let (_dir, mut conn) = setup_test_db();
        let _guard = failpoint::arm(&failpoint::CREATE_PRODUCT_AFTER_INSERT);

        let req = default_create_request();
        let result = create_product(&mut conn, req);
        assert!(result.is_err());

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0, "rollback で products に商品が残っていないこと");

        // next_seq も巻き戻る（TX内で increment されたが rollback）
        let dept = product_repo::find_department_by_id(&conn, 2)
            .unwrap()
            .unwrap();
        assert_eq!(dept.next_seq, 1, "next_seq が巻き戻っているべき");
    }

    #[test]
    #[serial]
    fn test_failpoint_arm_is_not_visible_to_spawned_thread() {
        let _guard = failpoint::arm(&failpoint::CREATE_PRODUCT_AFTER_INSERT);

        let result = std::thread::spawn(|| {
            let (_dir, mut conn) = setup_test_db();
            create_product(&mut conn, default_create_request())
        })
        .join()
        .expect("spawn 先の test thread が panic しないこと");

        assert!(
            result.is_ok(),
            "spawn 先では現 thread の failpoint が未武装であること: {result:?}"
        );
    }

    #[test]
    #[serial]
    fn test_create_product_req101_rollback_after_movement() {
        // REQ-101: 商品登録 — movement挿入後failpointでロールバック → 全行巻き戻る
        // TX深層rollback: movement 挿入後に failpoint → 全て巻き戻る
        let (_dir, mut conn) = setup_test_db();
        let _guard = failpoint::arm(&failpoint::CREATE_PRODUCT_AFTER_MOVEMENT);

        let mut req = default_create_request();
        req.initial_stock = 10; // movement を発生させる
        let result = create_product(&mut conn, req);
        assert!(result.is_err());

        let prod_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
            .unwrap();
        let mov_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(prod_count, 0, "products が巻き戻っていること");
        assert_eq!(mov_count, 0, "inventory_movements が巻き戻っていること");
    }

    // ===== FUNC-4.4: update_product テスト =====

    #[test]
    #[serial]
    fn test_update_product_req102_price_change() {
        // REQ-102: 売価変更で price_history 記録 + plu_dirty=true
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000046".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let update_req = ProductUpdateRequest {
            selling_price: Some(999),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000046", &update_req).unwrap();

        // price_history が記録された
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM price_history WHERE product_code = '2000000000046'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // plu_dirty=true
        let found = product_repo::find_by_product_code(&conn, "2000000000046")
            .unwrap()
            .unwrap();
        assert!(found.product.plu_dirty);
    }

    #[test]
    #[serial]
    fn test_update_product_req102_cost_only_no_plu_dirty() {
        // REQ-102: 原価のみ変更で price_history あり、plu_dirty 変更なし
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000053".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        // plu_dirty を false にリセット
        product_repo::update_product(
            &conn,
            "2000000000053",
            &ProductUpdates {
                plu_dirty: Some(false),
                ..Default::default()
            },
        )
        .unwrap();

        let update_req = ProductUpdateRequest {
            cost_price_centi: Some(999),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000053", &update_req).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM price_history WHERE product_code = '2000000000053'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "price_history が記録されるべき");

        let found = product_repo::find_by_product_code(&conn, "2000000000053")
            .unwrap()
            .unwrap();
        assert!(
            !found.product.plu_dirty,
            "原価のみ変更では plu_dirty は変わらない"
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_sets_plu_dirty_when_plu_target_turns_on() {
        // REQ-102 / REQ-402: PLU対象 0→1 は未反映化し、1→0 は未反映を増やさない。
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000060".to_string());
        req.department_id = 3;
        req.plu_target = false;
        create_product(&mut conn, req).unwrap();

        product_repo::update_product(
            &conn,
            "2000000000060",
            &ProductUpdates {
                plu_dirty: Some(false),
                ..Default::default()
            },
        )
        .unwrap();

        update_product(
            &mut conn,
            "2000000000060",
            &ProductUpdateRequest {
                plu_target: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let enabled = product_repo::find_by_product_code(&conn, "2000000000060")
            .unwrap()
            .unwrap();
        assert!(enabled.product.plu_target);
        assert!(enabled.product.plu_dirty);

        product_repo::update_product(
            &conn,
            "2000000000060",
            &ProductUpdates {
                plu_dirty: Some(false),
                ..Default::default()
            },
        )
        .unwrap();

        update_product(
            &mut conn,
            "2000000000060",
            &ProductUpdateRequest {
                plu_target: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        let disabled = product_repo::find_by_product_code(&conn, "2000000000060")
            .unwrap()
            .unwrap();
        assert!(!disabled.product.plu_target);
        assert!(!disabled.product.plu_dirty);
    }

    #[test]
    #[serial]
    fn test_update_product_req102_not_found() {
        // REQ-102: 商品更新 — 存在しない商品コードはNotFound
        let (_dir, mut conn) = setup_test_db();
        let req = ProductUpdateRequest {
            name: Some("存在しない".to_string()),
            ..Default::default()
        };
        let result = update_product(&mut conn, "NONEXISTENT", &req);
        assert!(matches!(result, Err(BizError::NotFound(_))));
    }

    #[test]
    #[serial]
    fn test_update_product_req102_validation_department_not_found() {
        // REQ-102: 商品更新 — 不正department_idはValidationFailed
        // update_product: 不正 department_id → ValidationFailed（P1レビュー指摘対応）
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000077".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let update_req = ProductUpdateRequest {
            department_id: Some(9999),
            ..Default::default()
        };
        let result = update_product(&mut conn, "2000000000077", &update_req);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(_))),
            "不正department_idでValidationFailed: {:?}",
            result
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_validation_supplier_not_found() {
        // REQ-102: 商品更新 — 不正supplier_idはValidationFailed
        // update_product: 不正 supplier_id → ValidationFailed（P1レビュー指摘対応）
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000084".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let update_req = ProductUpdateRequest {
            supplier_id: Some(Some(9999)),
            ..Default::default()
        };
        let result = update_product(&mut conn, "2000000000084", &update_req);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(_))),
            "不正supplier_idでValidationFailed: {:?}",
            result
        );
    }

    #[test]
    #[serial]
    fn test_create_product_req101_validation_initial_stock_negative() {
        // REQ-101: 商品登録 — 初期在庫マイナスはValidationFailed
        // create_product: initial_stock < 0 → ValidationFailed（P2レビュー指摘対応）
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.initial_stock = -1;
        let result = create_product(&mut conn, req);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(_))),
            "負の初期在庫でValidationFailed: {:?}",
            result
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_detail_json_recorded() {
        // REQ-102: 商品更新 — 売価変更でdetail_jsonに変更前後が記録される
        // update_product: 売価変更で detail_json に変更前後が記録される（P2レビュー指摘対応）
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000091".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let update_req = ProductUpdateRequest {
            selling_price: Some(999),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &update_req).unwrap();

        let detail: Option<String> = conn
            .query_row(
                "SELECT detail_json FROM operation_logs WHERE operation_type = 'product_update' ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(detail.is_some(), "detail_json が記録されるべき");
        let json = detail.unwrap();
        assert!(json.contains("\"old\":500"), "変更前の売価が含まれるべき");
        assert!(json.contains("\"new\":999"), "変更後の売価が含まれるべき");
    }

    /// 最新の product_update の detail_json を JSON として読む（NULL なら None）。
    fn latest_update_detail(conn: &DbConnection) -> Option<serde_json::Value> {
        let detail: Option<String> = conn
            .query_row(
                "SELECT detail_json FROM operation_logs WHERE operation_type = 'product_update' ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        detail.map(|s| serde_json::from_str(&s).expect("detail_json は JSON として読めるべき"))
    }

    fn detail_keys(detail: &serde_json::Value) -> Vec<String> {
        let mut keys: Vec<String> = detail.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    }

    #[test]
    #[serial]
    fn test_update_product_req102_detail_json_non_price_fields() {
        // REQ-102 / BIZ-01-D7: 価格以外の 5 field を変えると、それぞれの変更前後が記録される
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000091".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let update_req = ProductUpdateRequest {
            name: Some("改名後の商品".to_string()),
            department_id: Some(4),
            tax_rate: Some(ProductTaxRate::Rate8),
            pos_stock_sync: Some(false),
            plu_target: Some(true),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &update_req).unwrap();

        let detail = latest_update_detail(&conn).expect("detail_json が記録されるべき");
        assert_eq!(
            detail_keys(&detail),
            [
                "department_id",
                "name",
                "plu_target",
                "pos_stock_sync",
                "tax_rate"
            ]
        );
        assert_eq!(
            detail["name"],
            serde_json::json!({"old": "テスト商品", "new": "改名後の商品"})
        );
        assert_eq!(
            detail["department_id"],
            serde_json::json!({"old": 3, "new": 4})
        );
        assert_eq!(
            detail["tax_rate"],
            serde_json::json!({"old": "10", "new": "8"})
        );
        assert_eq!(
            detail["pos_stock_sync"],
            serde_json::json!({"old": true, "new": false})
        );
        assert_eq!(
            detail["plu_target"],
            serde_json::json!({"old": false, "new": true})
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_detail_json_clearable_fields() {
        // REQ-102 / BIZ-01-D7 / PRODUCT-PATCH-D1: 取引先・メーカー品番の設定と clear が null で記録される
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000091".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();
        let supplier_id = seed_named_supplier(&conn, "T2取引先");

        // (i) 無し → 値
        let set_req = ProductUpdateRequest {
            supplier_id: Some(Some(supplier_id)),
            maker_code: Some(Some("MK-T2".to_string())),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &set_req).unwrap();
        let detail = latest_update_detail(&conn).expect("detail_json が記録されるべき");
        assert_eq!(detail_keys(&detail), ["maker_code", "supplier_id"]);
        assert_eq!(
            detail["supplier_id"],
            serde_json::json!({"old": null, "new": supplier_id})
        );
        assert_eq!(
            detail["maker_code"],
            serde_json::json!({"old": null, "new": "MK-T2"})
        );

        // (ii) 値 → clear
        let clear_req = ProductUpdateRequest {
            supplier_id: Some(None),
            maker_code: Some(None),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &clear_req).unwrap();
        let detail = latest_update_detail(&conn).expect("clear も detail_json に記録されるべき");
        assert_eq!(detail_keys(&detail), ["maker_code", "supplier_id"]);
        assert_eq!(
            detail["supplier_id"],
            serde_json::json!({"old": supplier_id, "new": null})
        );
        assert_eq!(
            detail["maker_code"],
            serde_json::json!({"old": "MK-T2", "new": null})
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_detail_json_unchanged_fields_omitted() {
        // REQ-102 / BIZ-01-D7: 値を持っても同じ field は書かず、変更が無ければ NULL
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000091".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        // (i) 同じ売価・同じ名前・違う原価 → cost_price_centi だけ
        let req_i = ProductUpdateRequest {
            selling_price: Some(500),
            cost_price_centi: Some(320),
            name: Some("テスト商品".to_string()),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &req_i).unwrap();
        let detail = latest_update_detail(&conn).expect("detail_json が記録されるべき");
        assert_eq!(detail_keys(&detail), ["cost_price_centi"]);
        assert_eq!(
            detail["cost_price_centi"],
            serde_json::json!({"old": 300, "new": 320})
        );

        // (ii) 全 field を更新前と同じ値で送る → NULL（clear 可能 field は Some(None)）
        let req_ii = ProductUpdateRequest {
            name: Some("テスト商品".to_string()),
            department_id: Some(3),
            supplier_id: Some(None),
            selling_price: Some(500),
            cost_price_centi: Some(320),
            tax_rate: Some(ProductTaxRate::Rate10),
            maker_code: Some(None),
            pos_stock_sync: Some(true),
            plu_target: Some(false),
            stock_unit: Some(ProductStockUnit::Pcs),
        };
        update_product(&mut conn, "2000000000091", &req_ii).unwrap();
        assert_eq!(latest_update_detail(&conn), None, "変更が無ければ NULL");

        // (ii-b) 同じ単位と新しい商品名 → 商品名は更新し、key は name だけ（BIZ-01-D8 の同じ単位）
        let req_ii_b = ProductUpdateRequest {
            name: Some("同じ単位で改名".to_string()),
            stock_unit: Some(ProductStockUnit::Pcs),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &req_ii_b).unwrap();
        let found = product_repo::find_by_product_code(&conn, "2000000000091")
            .unwrap()
            .unwrap();
        assert_eq!(found.product.name, "同じ単位で改名");
        assert_eq!(found.product.stock_unit, ProductStockUnit::Pcs);
        let detail = latest_update_detail(&conn).expect("detail_json が記録されるべき");
        assert_eq!(detail_keys(&detail), ["name"]);

        // (iii) 売価だけ → selling_price だけ（cost_price_centi を書かない）
        let req_iii = ProductUpdateRequest {
            selling_price: Some(999),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &req_iii).unwrap();
        let detail = latest_update_detail(&conn).expect("detail_json が記録されるべき");
        assert_eq!(detail_keys(&detail), ["selling_price"]);
        assert_eq!(
            detail["selling_price"],
            serde_json::json!({"old": 500, "new": 999})
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_detail_json_escapes_name() {
        // REQ-102 / BIZ-01-D7: `"` と `\` を含む名前でも壊れた JSON を作らない
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000091".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let new_name = "改名\"引用\\逆斜線";
        let update_req = ProductUpdateRequest {
            name: Some(new_name.to_string()),
            ..Default::default()
        };
        update_product(&mut conn, "2000000000091", &update_req).unwrap();

        let detail = latest_update_detail(&conn).expect("detail_json が記録されるべき");
        assert_eq!(detail["name"]["new"], serde_json::json!(new_name));
        assert_eq!(detail["name"]["old"], serde_json::json!("テスト商品"));
    }

    #[test]
    #[serial]
    fn test_update_product_req102_log_failure_rolls_back_all() {
        // REQ-102 / 30 §4.4 step 6・7: 操作ログの INSERT が失敗すると商品・price_history・ログが戻る
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000091".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();
        conn.execute_batch(
            "CREATE TRIGGER fail_product_update_log
             BEFORE INSERT ON operation_logs
             WHEN NEW.operation_type = 'product_update'
             BEGIN SELECT RAISE(ABORT, 'synthetic operation log failure'); END;",
        )
        .unwrap();
        let count = |conn: &DbConnection, sql: &str| -> i64 {
            conn.query_row(sql, [], |row| row.get(0)).unwrap()
        };
        let ph_sql = "SELECT COUNT(*) FROM price_history WHERE product_code = '2000000000091'";
        let log_sql = "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'product_update'";
        let ph_before = count(&conn, ph_sql);

        // (i) 売価と名前を変える
        let req_i = ProductUpdateRequest {
            selling_price: Some(999),
            name: Some("改名後の商品".to_string()),
            ..Default::default()
        };
        assert!(update_product(&mut conn, "2000000000091", &req_i).is_err());
        let found = product_repo::find_by_product_code(&conn, "2000000000091")
            .unwrap()
            .unwrap();
        assert_eq!(found.product.selling_price, 500, "売価が元のまま");
        assert_eq!(found.product.name, "テスト商品", "名前が元のまま");
        assert_eq!(count(&conn, ph_sql), ph_before, "price_history が戻る");
        assert_eq!(count(&conn, log_sql), 0, "product_update のログが残らない");

        // (ii) 名前だけを変える
        let req_ii = ProductUpdateRequest {
            name: Some("改名後の商品".to_string()),
            ..Default::default()
        };
        assert!(update_product(&mut conn, "2000000000091", &req_ii).is_err());
        let found = product_repo::find_by_product_code(&conn, "2000000000091")
            .unwrap()
            .unwrap();
        assert_eq!(found.product.name, "テスト商品", "名前が元のまま");
        assert_eq!(count(&conn, log_sql), 0, "product_update のログが残らない");
    }

    /// 商品を単位つきで登録し、plu_dirty を 0 にする（単位の変更の test 用）
    fn create_unit_product(conn: &mut DbConnection, jan: &str, unit: ProductStockUnit) {
        let mut req = default_create_request();
        req.jan_code = Some(jan.to_string());
        req.department_id = 3;
        req.stock_unit = unit;
        create_product(conn, req).unwrap();
        product_repo::update_product(
            conn,
            jan,
            &ProductUpdates {
                plu_dirty: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    }

    fn product_update_log_count(conn: &DbConnection) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'product_update'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    #[serial]
    fn test_update_product_req102_stock_unit_between_count_units() {
        // REQ-102 / SPEC-UNIT-D2 / BIZ-01-D7・D8: 個数の単位どうしの変更は通り、数・明細・価格・plu_dirty を変えない
        let (_dir, mut conn) = setup_test_db();
        create_unit_product(&mut conn, "2000000009018", ProductStockUnit::Pcs);
        let received = crate::biz::inventory_service::create_receiving(
            &mut conn,
            serde_json::from_value(serde_json::json!({
                "idempotency_key": "unit-relabel-receiving",
                "supplier_id": null,
                "receiving_date": "2026-10-01",
                "note": null,
                "items": [{"product_code": "2000000009018", "quantity": 12, "cost_price_centi": 300}],
            }))
            .unwrap(),
        )
        .unwrap();
        product_repo::update_product(
            &conn,
            "2000000009018",
            &ProductUpdates {
                plu_dirty: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        let snapshot = |conn: &DbConnection| {
            let product = product_repo::find_by_product_code(conn, "2000000009018")
                .unwrap()
                .unwrap()
                .product;
            let item_quantity =
                crate::biz::inventory_service::get_receiving_record(conn, received.record_id)
                    .unwrap()
                    .items[0]
                    .quantity;
            let price_history: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM price_history WHERE product_code = '2000000009018'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            (
                product.stock_quantity,
                product.selling_price,
                product.cost_price_centi,
                item_quantity,
                price_history,
                product.plu_dirty,
            )
        };
        let before = snapshot(&conn);
        assert_eq!(before.0, 12);
        assert_eq!(before.3, 12);
        assert!(!before.5);

        let req = ProductUpdateRequest {
            stock_unit: Some(ProductStockUnit::Ball),
            ..Default::default()
        };
        update_product(&mut conn, "2000000009018", &req).unwrap();

        let found = product_repo::find_by_product_code(&conn, "2000000009018")
            .unwrap()
            .unwrap();
        assert_eq!(found.product.stock_unit, ProductStockUnit::Ball);
        assert_eq!(
            snapshot(&conn),
            before,
            "数・明細・価格・plu_dirty は変わらない"
        );
        assert_eq!(
            latest_update_detail(&conn),
            Some(serde_json::json!({"stock_unit": {"old": "pcs", "new": "ball"}}))
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_rejects_cross_kind_stock_unit() {
        // REQ-102 / SPEC-UNIT-D2 / BIZ-01-D8: 個数 ↔ 長さの変更は拒み、何も書かない
        let (_dir, mut conn) = setup_test_db();
        create_unit_product(&mut conn, "2000000009018", ProductStockUnit::Pcs);
        create_unit_product(&mut conn, "2000000009025", ProductStockUnit::M);
        for (code, from, to) in [
            ("2000000009018", ProductStockUnit::Pcs, ProductStockUnit::M),
            ("2000000009025", ProductStockUnit::M, ProductStockUnit::Pcs),
        ] {
            let before = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap()
                .product;
            let logs_before = product_update_log_count(&conn);
            let req = ProductUpdateRequest {
                stock_unit: Some(to),
                ..Default::default()
            };
            let result = update_product(&mut conn, code, &req);
            assert!(
                matches!(result, Err(BizError::ValidationFailed(ref message)) if message == "在庫単位は個数の単位どうしでだけ変えられます"),
                "{code}: {result:?}"
            );
            let after = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap()
                .product;
            assert_eq!(after.stock_unit, from);
            assert_eq!(after.updated_at, before.updated_at);
            assert_eq!(product_update_log_count(&conn), logs_before);
        }
    }

    #[test]
    #[serial]
    fn test_update_product_req102_checks_unit_last() {
        // REQ-102 / BIZ-01-D8: 単位の検査は手順 2 の今の検査の後（取引先の error が単位の error より先）。
        // 長さどうしの変更を拒む case は、長さの単位が m だけになり当たる組が無いので外した（D-128。
        // 種類をまたぐ変更の拒否は test_update_product_req102_rejects_cross_kind_stock_unit）
        let (_dir, mut conn) = setup_test_db();
        create_unit_product(&mut conn, "2000000009018", ProductStockUnit::Pcs);
        let req = ProductUpdateRequest {
            supplier_id: Some(Some(9999)),
            stock_unit: Some(ProductStockUnit::M),
            ..Default::default()
        };
        let result = update_product(&mut conn, "2000000009018", &req);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref message)) if message == "指定された取引先が存在しません"),
            "{result:?}"
        );
    }

    #[test]
    #[serial]
    fn test_update_product_req102_stock_unit_null_keeps_unit() {
        // REQ-102 / PRODUCT-PATCH-D1 / 30 §4.4 手順 6: stock_unit が null なら単位を変えず、ログ行は書き detail は NULL
        let (_dir, mut conn) = setup_test_db();
        create_unit_product(&mut conn, "2000000009018", ProductStockUnit::Pcs);
        let logs_before = product_update_log_count(&conn);
        let req: ProductUpdateRequest = serde_json::from_str(r#"{"stock_unit":null}"#).unwrap();
        update_product(&mut conn, "2000000009018", &req).unwrap();
        let found = product_repo::find_by_product_code(&conn, "2000000009018")
            .unwrap()
            .unwrap();
        assert_eq!(found.product.stock_unit, ProductStockUnit::Pcs);
        assert_eq!(product_update_log_count(&conn), logs_before + 1);
        assert_eq!(latest_update_detail(&conn), None);
    }

    #[test]
    fn test_product_update_request_req102_ordinary_fields_omitted_null_value() {
        // REQ-102 / PRODUCT-PATCH-D1: 通常8 fieldは omitted/null=no update、value=set。
        let req: ProductUpdateRequest = serde_json::from_str("{}").unwrap();
        assert_eq!(req.stock_unit, None);
        assert_eq!(req.name, None);
        assert_eq!(req.department_id, None);
        assert_eq!(req.selling_price, None);
        assert_eq!(req.cost_price_centi, None);
        assert_eq!(req.tax_rate, None);
        assert_eq!(req.pos_stock_sync, None);
        assert_eq!(req.plu_target, None);

        let req: ProductUpdateRequest = serde_json::from_str(
            r#"{
                "name":null,
                "department_id":null,
                "selling_price":null,
                "cost_price_centi":null,
                "tax_rate":null,
                "pos_stock_sync":null,
                "plu_target":null,
                "stock_unit":null
            }"#,
        )
        .unwrap();
        assert_eq!(req.stock_unit, None);
        assert_eq!(req.name, None);
        assert_eq!(req.department_id, None);
        assert_eq!(req.selling_price, None);
        assert_eq!(req.cost_price_centi, None);
        assert_eq!(req.tax_rate, None);
        assert_eq!(req.pos_stock_sync, None);
        assert_eq!(req.plu_target, None);

        let req: ProductUpdateRequest = serde_json::from_str(
            r#"{
                "name":"更新名",
                "department_id":4,
                "selling_price":1200,
                "cost_price_centi":700,
                "tax_rate":"8",
                "pos_stock_sync":false,
                "plu_target":true,
                "stock_unit":"ball"
            }"#,
        )
        .unwrap();
        assert_eq!(req.stock_unit, Some(ProductStockUnit::Ball));
        assert_eq!(req.name, Some("更新名".to_string()));
        assert_eq!(req.department_id, Some(4));
        assert_eq!(req.selling_price, Some(1200));
        assert_eq!(req.cost_price_centi, Some(700));
        assert_eq!(req.tax_rate, Some(ProductTaxRate::Rate8));
        assert_eq!(req.pos_stock_sync, Some(false));
        assert_eq!(req.plu_target, Some(true));
    }

    #[test]
    fn test_product_update_request_clearable_fields_omitted_null_value_req102() {
        // REQ-102 / PRODUCT-PATCH-D1: clear可能2 fieldは omitted=no update、null=clear、value=set。
        let req: ProductUpdateRequest = serde_json::from_str("{}").unwrap();
        assert_eq!(req.supplier_id, None);
        assert_eq!(req.maker_code, None);

        let req: ProductUpdateRequest =
            serde_json::from_str(r#"{"supplier_id":null,"maker_code":null}"#).unwrap();
        assert_eq!(req.supplier_id, Some(None));
        assert_eq!(req.maker_code, Some(None));

        let req: ProductUpdateRequest =
            serde_json::from_str(r#"{"supplier_id":3,"maker_code":"A-1"}"#).unwrap();
        assert_eq!(req.supplier_id, Some(Some(3)));
        assert_eq!(req.maker_code, Some(Some("A-1".to_string())));
    }

    #[test]
    #[serial]
    fn test_update_product_req102_rollback_after_price_history() {
        // REQ-102: 商品更新 — price_history INSERT後failpointでロールバック → 全行巻き戻る
        // TX rollback: price_history INSERT 後に failpoint → 全て巻き戻る
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000107".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let _guard = failpoint::arm(&failpoint::UPDATE_PRODUCT_AFTER_PRICE_HISTORY);

        let update_req = ProductUpdateRequest {
            selling_price: Some(999),
            ..Default::default()
        };
        let result = update_product(&mut conn, "2000000000107", &update_req);
        assert!(result.is_err());

        // price_history が巻き戻っている
        let ph_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM price_history WHERE product_code = '2000000000107'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(ph_count, 0, "price_history が巻き戻っているべき");

        // products の selling_price が変わっていない
        let found = product_repo::find_by_product_code(&conn, "2000000000107")
            .unwrap()
            .unwrap();
        assert_eq!(found.product.selling_price, 500, "selling_price が元のまま");
    }

    // ===== FUNC-4.5: toggle_discontinue テスト =====

    #[test]
    #[serial]
    fn test_toggle_discontinue_req102_to_discontinued() {
        // REQ-102: false→true（廃番化）
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000114".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        let new_status = toggle_discontinue(&mut conn, "2000000000114").unwrap();
        assert!(new_status, "廃番になるべき");

        let found = product_repo::find_by_product_code(&conn, "2000000000114")
            .unwrap()
            .unwrap();
        assert!(found.product.is_discontinued);
        assert!(found.product.plu_dirty);
    }

    #[test]
    #[serial]
    fn test_toggle_discontinue_req102_to_active() {
        // REQ-102: true→false（復帰）
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("2000000000121".to_string());
        req.department_id = 3;
        create_product(&mut conn, req).unwrap();

        toggle_discontinue(&mut conn, "2000000000121").unwrap(); // → true
        let new_status = toggle_discontinue(&mut conn, "2000000000121").unwrap(); // → false
        assert!(!new_status, "復帰するべき");
    }

    #[test]
    #[serial]
    fn test_toggle_discontinue_req102_not_found() {
        // REQ-102: 廃番切替 — 存在しない商品コードはNotFound
        let (_dir, mut conn) = setup_test_db();
        let result = toggle_discontinue(&mut conn, "NONEXISTENT");
        assert!(matches!(result, Err(BizError::NotFound(_))));
    }

    // ===== FUNC-4.6: search_products テスト =====

    #[test]
    #[serial]
    fn test_search_products_req103_biz_wrapper() {
        // REQ-103: 商品検索 — BIZ層ラッパー経由の正常検索（結合確認）
        // BIZ経由の正常検索（結合確認）
        let (_dir, mut conn) = setup_test_db();
        let req = default_create_request();
        create_product(&mut conn, req).unwrap();

        let query = ProductSearchQuery {
            keyword: None,
            department_id: None,
            supplier_id: None,
            include_unassigned: false,
            is_discontinued: None,
            plu: None,
            sort_key: product_repo::SortKey::ProductCode,
            sort_order: product_repo::SortOrder::Asc,
            page: 1,
            per_page: 50,
        };
        let result = search_products(&conn, query).unwrap();
        assert_eq!(result.total_count, 1);
    }

    #[test]
    #[serial]
    fn test_list_departments_req103_biz_wrapper_returns_all_departments() {
        // REQ-103 / UI-01a-D7: 部門候補は search_products の現在ページではなく、
        // BIZ -> IO の list_departments から master data 全件を取得する。
        let (_dir, conn) = setup_test_db();

        let result = list_departments(&conn).unwrap();

        assert_eq!(result.len(), 21, "初期部門 master data 全件を返すべき");
        assert_eq!(result[0].id, 1, "ORDER BY id ASC を維持するべき");
        assert!(
            result.iter().any(|department| department.id == 21),
            "現在ページの検索結果ではなく全件候補であるべき"
        );
    }

    #[test]
    #[serial]
    fn test_list_suppliers_req101_biz_wrapper_returns_all_suppliers() {
        // REQ-101 / UI-01b-D7: 取引先候補は current product / current page ではなく、
        // BIZ -> IO の list_suppliers から master data 全件を取得する。
        let (_dir, conn) = setup_test_db();
        product_repo::find_or_create_supplier(&conn, "テスト取引先A").unwrap();
        product_repo::find_or_create_supplier(&conn, "テスト取引先B").unwrap();

        let result = list_suppliers(&conn).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "テスト取引先A");
        assert_eq!(result[1].name, "テスト取引先B");
    }

    // ===== preview_import テスト =====

    fn make_csv(headers: &str, rows: &[&str]) -> Vec<u8> {
        let mut csv = format!("{}\n", headers);
        for row in rows {
            csv.push_str(&format!("{}\n", row));
        }
        // UTF-8 BOM付き
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(csv.as_bytes());
        bytes
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_normal() {
        // REQ-104: 一括インポート — 正常CSV → valid_rows
        // BIZ-01 §4.7: 正常CSV → valid_rows
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &["PI-001,テスト商品,1,500,300,10"],
        );
        let result = preview_import(&conn, &csv).unwrap();
        assert_eq!(result.valid_rows.len(), 1);
        assert_eq!(result.error_rows.len(), 0);
        assert_eq!(result.duplicate_rows.len(), 0);
        assert_eq!(result.valid_rows[0].product_code, "PI-001");
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_parse_error() {
        // REQ-104: 一括インポート — デコード失敗はImportError
        // BIZ-01 §4.7: デコード失敗 → ImportError
        let result = preview_import(
            &setup_test_db().1,
            &[0xFF, 0xFE, 0x00], // 不正なバイト列
        );
        assert!(matches!(result, Err(BizError::ImportError(_))));
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_rejects_oversize_file() {
        // REQ-104 / UI-01c-D15: CMD を bypass しても BIZ 安全網が上限超過を拒否する。
        let (_dir, conn) = setup_test_db();
        let result = preview_import(
            &conn,
            &vec![0; crate::constants::CSV_IMPORT_FILE_SIZE_LIMIT + 1],
        );

        assert!(matches!(
            result,
            Err(BizError::ImportError(ref message))
                if message == "ファイルサイズが上限（20MB）を超えています"
        ));
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_missing_headers() {
        // REQ-104: 一括インポート — 必須列不足はImportError
        // BIZ-01 §4.7: 必須列不足 → ImportError
        let (_dir, conn) = setup_test_db();
        let csv = make_csv("商品コード,商品名", &["PI-002,テスト"]);
        let result = preview_import(&conn, &csv);
        assert!(matches!(result, Err(BizError::ImportError(ref msg)) if msg.contains("必須列")));
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_validation_errors() {
        // REQ-104: 一括インポート — 売価マイナスはerror_rows
        // BIZ-01 §4.7: 売価マイナス → error_rows
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &["PI-003,テスト商品,1,-100,300,10"],
        );
        let result = preview_import(&conn, &csv).unwrap();
        assert_eq!(result.error_rows.len(), 1);
        assert!(result.error_rows[0]
            .errors
            .iter()
            .any(|e| e.contains("0以上")));
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_product_code_length_limit() {
        // REQ-104 / BIZ-01-D5: 商品コードは UTF-16 code unit で 100 まで
        let (_dir, conn) = setup_test_db();
        let a100 = "A".repeat(100);
        let a101 = "A".repeat(101);
        let supp50 = "\u{2000B}".repeat(50); // 100 unit
        let supp51 = "\u{2000B}".repeat(51); // 102 unit
        let kana100 = "あ".repeat(100); // 300 byte
        let kana101 = "あ".repeat(101);
        let rows: Vec<String> = [&a100, &a101, &supp50, &supp51, &kana100, &kana101]
            .iter()
            .map(|code| format!("{code},テスト商品,1,500,300,10"))
            .collect();
        let row_refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        let csv = make_csv("商品コード,商品名,部門ID,売価,原価,税率", &row_refs);

        let result = preview_import(&conn, &csv).unwrap();

        let valid: Vec<&str> = result
            .valid_rows
            .iter()
            .map(|row| row.product_code.as_str())
            .collect();
        assert_eq!(
            valid,
            vec![a100.as_str(), supp50.as_str(), kana100.as_str()]
        );
        let errored: Vec<(&str, Vec<String>)> = result
            .error_rows
            .iter()
            .map(|row| (row.raw_data["商品コード"].as_str(), row.errors.clone()))
            .collect();
        let expected_errors = vec!["商品コードは100文字以内で入力してください".to_string()];
        assert_eq!(
            errored,
            vec![
                (a101.as_str(), expected_errors.clone()),
                (supp51.as_str(), expected_errors.clone()),
                (kana101.as_str(), expected_errors),
            ]
        );
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_duplicates() {
        // REQ-104: 一括インポート — 既存商品はduplicate_rows
        // BIZ-01 §4.7: 既存商品 → duplicate_rows
        let (_dir, mut conn) = setup_test_db();
        let req = default_create_request();
        create_product(&mut conn, req).unwrap(); // product_code は独自コード発番

        // 発番された商品コードを取得
        let existing: String = conn
            .query_row(
                "SELECT product_code FROM products ORDER BY product_code LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &[&format!("{},重複商品,1,500,300,10", existing)],
        );
        let result = preview_import(&conn, &csv).unwrap();
        assert_eq!(result.duplicate_rows.len(), 1);
        assert_eq!(result.duplicate_rows[0].existing_product_code, existing);
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_mixed() {
        // REQ-104: 一括インポート — 正常+エラー+重複の混在
        // BIZ-01 §4.7: 正常+エラー+重複の混在
        let (_dir, mut conn) = setup_test_db();
        // 既存商品を作成
        let mut req = default_create_request();
        req.name = "既存商品".to_string();
        create_product(&mut conn, req).unwrap();
        let existing: String = conn
            .query_row(
                "SELECT product_code FROM products ORDER BY product_code LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &[
                "NEW-001,新規商品,1,500,300,10",                // 正常
                "ERR-001,,1,500,300,10",                        // エラー（名前空）
                &format!("{},重複商品,1,500,300,10", existing), // 重複
            ],
        );
        let result = preview_import(&conn, &csv).unwrap();
        assert_eq!(result.valid_rows.len(), 1);
        assert_eq!(result.error_rows.len(), 1);
        assert_eq!(result.duplicate_rows.len(), 1);
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_optional_fields_none() {
        // REQ-104: 一括インポート — 必須列のみのCSV → オプション項目はNone（デフォルト化しない）
        // BIZ-01 §4.7 P3-3 + R2-5: 必須列のみのCSV → オプション項目は None
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &["OPT-001,テスト商品,1,500,300,10"],
        );
        let result = preview_import(&conn, &csv).unwrap();
        assert_eq!(result.valid_rows.len(), 1);
        let row = &result.valid_rows[0];
        assert!(
            row.stock_unit.is_none(),
            "stock_unit は None（デフォルト化しない）"
        );
        assert!(row.initial_stock.is_none(), "initial_stock は None");
        assert!(row.pos_stock_sync.is_none(), "pos_stock_sync は None");
        assert!(row.jan_code.is_none(), "jan_code は None");
        assert!(row.maker_code.is_none(), "maker_code は None");
        assert!(row.supplier_id.is_none(), "supplier_id は None");
    }

    #[test]
    #[serial]
    fn test_preview_import_req907_accepts_plu_target_one_zero_and_blank() {
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率,JANコード,PLU対象",
            &[
                "PLU-C1-ON,対象商品,1,500,300,10,2000000000039,1",
                "PLU-C1-OFF,対象外商品,1,500,300,10,,0",
                "PLU-C1-DEFAULT,既定商品,1,500,300,10,2000000000015,",
            ],
        );

        let result = preview_import(&conn, &csv).unwrap();
        assert!(result.error_rows.is_empty());
        assert_eq!(result.valid_rows[0].plu_target, Some(true));
        assert!(result.valid_rows[0].warnings.is_empty());
        assert_eq!(result.valid_rows[1].plu_target, Some(false));
        assert!(result.valid_rows[1].warnings.is_empty());
        assert_eq!(result.valid_rows[2].plu_target, None);
        assert!(result.valid_rows[2].warnings.is_empty());
    }

    #[test]
    #[serial]
    fn test_preview_import_req907_warns_and_disables_plu_target_without_13_digit_jan() {
        // REQ-907 / 30 §4.8 手順 4: `PLU対象=1` の warning は空か 8 桁の JAN。形の誤った JAN は BIZ-01-D10 の行の error
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率,JANコード,PLU対象",
            &[
                "PLU-C2-NONE,JANなし,1,500,300,10,,1",
                "PLU-C2-SHORT,8桁JAN,1,500,300,10,49123456,1",
                "PLU-C2-CHECK,不正CD,1,500,300,10,2000000000038,1",
            ],
        );

        let result = preview_import(&conn, &csv).unwrap();
        assert_eq!(result.valid_rows.len(), 2);
        for row in &result.valid_rows {
            assert_eq!(row.plu_target, Some(false), "{}", row.product_code);
            assert_eq!(
                row.warnings,
                vec!["JAN が13桁でないため対象外として取り込みます"]
            );
        }
        assert_eq!(
            shape_error_rows(&result),
            vec![(4, vec![shape_f4("2000000000038")])]
        );
    }

    // ===== BIZ-01-D10: 商品コードと JANコードの形（oracle は 30 BIZ-01-D10 の表の文から写す） =====

    fn shape_f1(column: &str, value: &str) -> String {
        format!(
            "{column}が指数の形です: '{value}'。Excel が桁を落として書いた値です。元の Excel でその列を文字列の値にして（TEXT の列）保存し直してください。セルを選んだときの上の欄（数式バー）に E+ が出るなら、元の資料から全部の桁を入れ直してください"
        )
    }

    fn shape_f2(value: &str) -> String {
        format!(
            "商品コードが13桁の数字でJANコードが空です: '{value}'。JANを商品コードにした商品は、JANコードにも同じ値を入れてください"
        )
    }

    fn shape_f3(value: &str) -> String {
        format!(
            "JANコードが8桁か13桁の数字ではありません: '{value}'。先頭の0が落ちていないかを見て、商品のバーコードの数字を元の資料から文字列で入れ直してください"
        )
    }

    fn shape_f4(value: &str) -> String {
        format!(
            "JANコードのチェックディジットが合いません: '{value}'。商品のバーコードの数字を元の資料から入れ直してください"
        )
    }

    fn shape_error_rows(preview: &ImportPreview) -> Vec<(usize, Vec<String>)> {
        let mut rows: Vec<(usize, Vec<String>)> = preview
            .error_rows
            .iter()
            .map(|row| (row.line_no, row.errors.clone()))
            .collect();
        rows.sort();
        rows
    }

    fn valid_codes(preview: &ImportPreview) -> Vec<&str> {
        preview
            .valid_rows
            .iter()
            .map(|row| row.product_code.as_str())
            .collect()
    }

    const SHAPE_HEADER: &str = "商品コード,商品名,部門ID,売価,原価,税率,JANコード";

    #[test]
    #[serial]
    fn test_preview_import_req104_rejects_exponent_form_codes() {
        // REQ-104 / BIZ-01-D10 F1: 2 列、`E`/`e`、小数点の有無。両方なら 商品コード → JANコード の順
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            SHAPE_HEADER,
            &[
                "4.90123E+12,指数の商品コード,1,500,300,10,4901234567887",
                "5E+12,小数点なし,1,500,300,10,4901234567887",
                "EXP-OWN,指数のJAN,1,500,300,10,4.90123e+12",
                "4.9E+12,両方,1,500,300,10,4.9E+12",
                "EXP-OK,正しい行,1,500,300,10,",
            ],
        );

        let preview = preview_import(&conn, &csv).unwrap();

        assert_eq!(
            shape_error_rows(&preview),
            vec![
                (2, vec![shape_f1("商品コード", "4.90123E+12")]),
                (3, vec![shape_f1("商品コード", "5E+12")]),
                (4, vec![shape_f1("JANコード", "4.90123e+12")]),
                (
                    5,
                    vec![
                        shape_f1("商品コード", "4.9E+12"),
                        shape_f1("JANコード", "4.9E+12")
                    ]
                ),
            ]
        );
        assert_eq!(valid_codes(&preview), vec!["EXP-OK"]);
        assert!(preview.duplicate_rows.is_empty());
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_rejects_13_digit_code_without_jan() {
        // REQ-104 / BIZ-01-D10 F2: 13 桁ちょうどの数字の商品コードで JANコード が空。列が無いときも
        let (_dir, conn) = setup_test_db();
        for (header, rows) in [
            (
                SHAPE_HEADER,
                [
                    "4901234567887,JANが空,1,500,300,10,",
                    "F2-OK,正しい行,1,500,300,10,",
                ],
            ),
            (
                "商品コード,商品名,部門ID,売価,原価,税率",
                [
                    "4901234567887,JANの列なし,1,500,300,10",
                    "F2-OK,正しい行,1,500,300,10",
                ],
            ),
        ] {
            let preview = preview_import(&conn, &make_csv(header, &rows)).unwrap();
            assert_eq!(
                shape_error_rows(&preview),
                vec![(2, vec![shape_f2("4901234567887")])],
                "{header}"
            );
            assert_eq!(valid_codes(&preview), vec!["F2-OK"], "{header}");
        }
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_validates_jan_shape_and_check_digit() {
        // REQ-104 / BIZ-01-D10 F3・F4: BIZ-01-D1 の validator を共有し、文は CSV の文
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            SHAPE_HEADER,
            &[
                "JAN-7,7桁,1,500,300,10,4912345",
                "JAN-12,12桁,1,500,300,10,490123456788",
                "JAN-WIDE,全角,1,500,300,10,４９０１２３４５６７８８７",
                "JAN-CD13,CD誤り13桁,1,500,300,10,4901234567890",
                "JAN-CD8,CD誤り8桁,1,500,300,10,49123457",
                "JAN-OK13,正しい13桁,1,500,300,10,4901234567887",
                "JAN-OK8,正しい8桁,1,500,300,10,49123456",
            ],
        );

        let preview = preview_import(&conn, &csv).unwrap();

        assert_eq!(
            shape_error_rows(&preview),
            vec![
                (2, vec![shape_f3("4912345")]),
                (3, vec![shape_f3("490123456788")]),
                (4, vec![shape_f3("４９０１２３４５６７８８７")]),
                (5, vec![shape_f4("4901234567890")]),
                (6, vec![shape_f4("49123457")]),
            ]
        );
        assert_eq!(valid_codes(&preview), vec!["JAN-OK13", "JAN-OK8"]);
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_shape_check_applies_to_overwrite_candidates() {
        // REQ-104 / 30 §4.8 手順 4: 形の検査は重複の分類より前なので上書きの候補の行にも当たる。
        // 13 桁の数字でない商品の JAN を消す上書きは意図した契約（BIZ-01-D8 の上書きの段）
        let (_dir, mut conn) = setup_test_db();
        let mut req = default_create_request();
        req.jan_code = Some("4901234567887".to_string());
        assert_eq!(
            create_product(&mut conn, req).unwrap().product_code,
            "4901234567887"
        );
        seed_jan_product(&conn, "OWN-NOJAN", None, true);
        seed_jan_product(&conn, "OWN-JAN", Some("49123456"), true);

        let preview = preview_import(
            &conn,
            &make_csv(
                SHAPE_HEADER,
                &[
                    "4901234567887,上書きでJANが空,1,500,300,10,",
                    "OWN-NOJAN,上書きでCD誤り,1,500,300,10,4901234567890",
                    "OWN-JAN,上書きでJANを消す,1,500,300,10,",
                ],
            ),
        )
        .unwrap();
        assert_eq!(
            shape_error_rows(&preview),
            vec![
                (2, vec![shape_f2("4901234567887")]),
                (3, vec![shape_f4("4901234567890")]),
            ]
        );
        assert!(preview.valid_rows.is_empty());
        let duplicates: Vec<&str> = preview
            .duplicate_rows
            .iter()
            .map(|row| row.existing_product_code.as_str())
            .collect();
        assert_eq!(duplicates, vec!["OWN-JAN"]);

        // (c) 同じ JAN の上書きは重複行（同じ商品コードなので別の CSV）
        let same_jan = preview_import(
            &conn,
            &make_csv(
                SHAPE_HEADER,
                &["4901234567887,上書きで同じJAN,1,500,300,10,4901234567887"],
            ),
        )
        .unwrap();
        assert!(same_jan.error_rows.is_empty());
        assert_eq!(same_jan.duplicate_rows.len(), 1);
        assert_eq!(
            same_jan.duplicate_rows[0].existing_product_code,
            "4901234567887"
        );

        // (d) を上書きすると JAN が空になる
        let clear_row = preview.duplicate_rows[0].import_row.clone();
        commit_import(&mut conn, vec![clear_row], vec!["OWN-JAN".to_string()]).unwrap();
        let cleared = product_repo::find_by_product_code(&conn, "OWN-JAN")
            .unwrap()
            .unwrap();
        assert_eq!(cleared.product.jan_code, None);
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_keeps_accepted_code_forms() {
        // REQ-104 / BIZ-01-D10 の「通るもの」
        let (_dir, conn) = setup_test_db();
        let header = "商品コード,商品名,部門ID,売価,原価,税率,JANコード,POS在庫連動";
        let preview = preview_import(
            &conn,
            &make_csv(
                header,
                &[
                    "KEEP-OWN,独自コードでJANが空,1,500,300,10,,",
                    "4901234567887,同じJAN,1,500,300,10,4901234567887,",
                    "KEEP-E8A,8桁JAN,1,500,300,10,49123456,",
                    "KEEP-E8B,8桁JAN,1,500,300,10,96385074,",
                    "KEEP-STORE,店内コード,1,500,300,10,2000000000039,",
                    "KEEP-SHARE-1,共有JAN,1,500,300,10,2000000000046,0",
                    "KEEP-SHARE-2,共有JAN,1,500,300,10,2000000000046,0",
                    "490123456788,12桁の商品コード,1,500,300,10,,",
                    "49012345678870,14桁の商品コード,1,500,300,10,,",
                ],
            ),
        )
        .unwrap();
        assert!(preview.error_rows.is_empty(), "{:?}", preview.error_rows);
        assert_eq!(
            valid_codes(&preview),
            vec![
                "KEEP-OWN",
                "4901234567887",
                "KEEP-E8A",
                "KEEP-E8B",
                "KEEP-STORE",
                "KEEP-SHARE-1",
                "KEEP-SHARE-2",
                "490123456788",
                "49012345678870",
            ]
        );

        // 13 桁の数字の商品コードに違う正しい JAN（同じ商品コードなので別の CSV）
        let other_jan = preview_import(
            &conn,
            &make_csv(
                header,
                &["4901234567887,違う正しいJAN,1,500,300,10,4901234567894,"],
            ),
        )
        .unwrap();
        assert!(
            other_jan.error_rows.is_empty(),
            "{:?}",
            other_jan.error_rows
        );
        assert_eq!(valid_codes(&other_jan), vec!["4901234567887"]);
    }

    #[test]
    #[serial]
    fn test_preview_import_req907_rejects_plu_target_synonyms_and_invalid_values() {
        let (_dir, conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率,JANコード,PLU対象",
            &[
                "PLU-C3-TRUE,true値,1,500,300,10,2000000000039,true",
                "PLU-C3-YES,はい値,1,500,300,10,2000000000039,はい",
                "PLU-C3-TWO,2値,1,500,300,10,2000000000039,2",
                "PLU-C3-TRIM,trim値,1,500,300,10,2000000000039, 1 ",
            ],
        );

        let result = preview_import(&conn, &csv).unwrap();
        assert_eq!(result.error_rows.len(), 3);
        assert_eq!(result.valid_rows.len(), 1);
        assert_eq!(result.valid_rows[0].plu_target, Some(true));
        for row in result.error_rows {
            assert!(row
                .errors
                .iter()
                .any(|error| error.contains("PLU対象の値が不正です")));
        }
    }

    // ===== commit_import テスト =====

    fn make_import_row(product_code: &str, name: &str) -> ImportRow {
        ImportRow {
            line_no: 1,
            product_code: product_code.to_string(),
            name: name.to_string(),
            department_id: 1,
            selling_price: 500,
            cost_price_centi: 30_000, // CSV の原価 300 円と同じ値（1/100 円）
            tax_rate: "10".to_string(),
            stock_unit: None,
            initial_stock: None,
            jan_code: None,
            maker_code: None,
            supplier_id: None,
            pos_stock_sync: None,
            plu_target: None,
            warnings: Vec::new(),
        }
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_all_new() {
        // REQ-104: 一括インポート — 全行新規登録
        // BIZ-01 §4.8: 全行新規登録
        let (_dir, mut conn) = setup_test_db();
        let rows = vec![
            make_import_row("CI-001", "商品A"),
            make_import_row("CI-002", "商品B"),
        ];
        let result = commit_import(&mut conn, rows, vec![]).unwrap();
        assert_eq!(result.created_count, 2);
        assert_eq!(result.updated_count, 0);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE product_code LIKE 'CI-%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_missing_or_empty_stock_unit_defaults_to_pcs() {
        let (_dir, mut conn) = setup_test_db();
        let missing_csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &["SU-MISSING,欠落単位,1,500,300,10"],
        );
        let empty_csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率,在庫単位",
            &["SU-EMPTY,空単位,1,500,300,10,"],
        );
        let mut rows = preview_import(&conn, &missing_csv).unwrap().valid_rows;
        rows.extend(preview_import(&conn, &empty_csv).unwrap().valid_rows);

        commit_import(&mut conn, rows, vec![]).unwrap();
        let units: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT stock_unit FROM products WHERE product_code LIKE 'SU-%' ORDER BY product_code",
                )
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        assert_eq!(units, vec!["pcs", "pcs"]);
    }

    const UNIT_CSV_HEADERS: &str = "商品コード,商品名,部門ID,売価,原価,税率,在庫単位,初期在庫";

    /// `在庫単位,初期在庫` の組を 1 行ずつの CSV にして preview する（商品コードは `{prefix}-{行番号}`）
    fn preview_unit_rows(conn: &DbConnection, prefix: &str, cells: &[&str]) -> ImportPreview {
        let rows: Vec<String> = cells
            .iter()
            .enumerate()
            .map(|(index, cell)| format!("{prefix}-{index},合成の商品,1,500,300,10,{cell}"))
            .collect();
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        preview_import(conn, &make_csv(UNIT_CSV_HEADERS, &rows)).unwrap()
    }

    /// 商品・価格の履歴・変動・操作ログが変わらないことの比較に使う
    fn import_side_effects(conn: &DbConnection) -> (Vec<String>, i64, i64, i64) {
        let mut statement = conn
            .prepare(
                "SELECT product_code || '|' || name || '|' || selling_price || '|' || cost_price_centi
                        || '|' || stock_quantity || '|' || stock_unit
                 FROM products ORDER BY product_code",
            )
            .unwrap();
        let products = statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();
        let count = |table: &str| -> i64 {
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
        };
        (
            products,
            count("price_history"),
            count("inventory_movements"),
            count("operation_logs"),
        )
    }

    fn assert_commit_rejected(
        conn: &mut DbConnection,
        rows: Vec<ImportRow>,
        overwrite_codes: &[&str],
        expected_message: &str,
    ) {
        let before = import_side_effects(conn);
        let overwrite_codes = overwrite_codes
            .iter()
            .map(|code| code.to_string())
            .collect();
        let result = commit_import(conn, rows, overwrite_codes);
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref message)) if message.starts_with(expected_message)),
            "{expected_message}: {result:?}"
        );
        assert_eq!(import_side_effects(conn), before, "{expected_message}");
    }

    fn product_stock_and_unit(conn: &DbConnection, product_code: &str) -> (i64, String) {
        conn.query_row(
            "SELECT stock_quantity, stock_unit FROM products WHERE product_code = ?1",
            [product_code],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_nonempty_invalid_stock_unit_rolls_back() {
        // REQ-104 / SPEC-UNIT-D10 / BIZ-01-D8: 一覧に無い単位は preview の行の error。
        // preview を通さない commit は ValidationFailed で TX 全体を戻す
        let (_dir, mut conn) = setup_test_db();
        let preview = preview_unit_rows(&conn, "SU", &["kg,25.155", "pcs,1"]);
        assert_eq!(preview.valid_rows.len(), 1, "他の行は valid");
        assert_eq!(preview.error_rows.len(), 1);
        // 単位が決まらない行の初期在庫は読まない（error は単位の 1 つだけ）
        assert_eq!(
            preview.error_rows[0].errors,
            vec!["在庫単位が正しくありません: kg".to_string()]
        );

        let mut invalid = make_import_row("SU-KG", "不正単位");
        invalid.stock_unit = Some("kg".to_string());
        assert_commit_rejected(
            &mut conn,
            vec![make_import_row("SU-OK", "同じ取込みの行"), invalid],
            &[],
            "在庫単位が正しくありません: kg",
        );
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE product_code IN ('SU-KG', 'SU-OK')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_stock_unit_code_and_label_normalized() {
        // REQ-104 / SPEC-UNIT-D10 / BIZ-01-D8（X12）: code か表示の語を code に正規化する。空は None
        let (_dir, mut conn) = setup_test_db();
        let preview = preview_unit_rows(&conn, "UL", &["玉,", " m ,", "pcs,", "セット,", ","]);
        assert!(preview.error_rows.is_empty(), "{:?}", preview.error_rows);
        let units: Vec<Option<&str>> = preview
            .valid_rows
            .iter()
            .map(|row| row.stock_unit.as_deref())
            .collect();
        assert_eq!(
            units,
            vec![Some("ball"), Some("m"), Some("pcs"), Some("set"), None]
        );
        // 10 個の code と 10 個の表示の語がすべて通る
        let labels = [
            ("pcs", "個"),
            ("sheet", "枚"),
            ("hon", "本"),
            ("bag", "袋"),
            ("box", "箱"),
            ("kumi", "組"),
            ("set", "セット"),
            ("ball", "玉"),
            ("cho", "丁"),
            ("m", "m"),
        ];
        for (code, label) in labels {
            for cell in [code, label] {
                let preview = preview_unit_rows(&conn, "UL12", &[&format!("{cell},")]);
                assert_eq!(
                    preview.valid_rows[0].stock_unit.as_deref(),
                    Some(code),
                    "{cell}"
                );
            }
        }

        // commit: 正規化した code で入り、空は pcs
        commit_import(&mut conn, preview.valid_rows, vec![]).unwrap();
        assert_eq!(product_stock_and_unit(&conn, "UL-0").1, "ball");
        assert_eq!(product_stock_and_unit(&conn, "UL-4").1, "pcs");
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_initial_stock_in_meters_for_m_rows() {
        // REQ-104 / SPEC-UNIT-D3 / BIZ-01-D8（X3）: m の行だけ初期在庫を m の数として cm にする
        let (_dir, mut conn) = setup_test_db();
        let accepted = [
            ("m,25", 2500),
            ("m,26.15", 2615),
            ("m,0", 0),
            ("m,0.05", 5),
            ("m,1.15", 115), // f64 で掛けると 114
            ("m,9999999.99", 999_999_999),
            ("pcs,25", 25),
            (",25", 25),
            ("pcs,999999999", 999_999_999),
            ("pcs,0", 0),
        ];
        let cells: Vec<&str> = accepted.iter().map(|(cell, _)| *cell).collect();
        let preview = preview_unit_rows(&conn, "IM", &cells);
        assert!(preview.error_rows.is_empty(), "{:?}", preview.error_rows);
        let stocks: Vec<Option<i64>> = preview
            .valid_rows
            .iter()
            .map(|row| row.initial_stock)
            .collect();
        let expected: Vec<Option<i64>> = accepted.iter().map(|(_, cm)| Some(*cm)).collect();
        assert_eq!(stocks, expected);

        let rejected = [
            ("m,10000000", "在庫数が大きすぎます"),
            ("m,25.155", "初期在庫の値が不正です: '25.155'"),
            ("m,-1", "初期在庫の値が不正です: '-1'"),
            ("m,1e2", "初期在庫の値が不正です: '1e2'"),
            ("m,.5", "初期在庫の値が不正です: '.5'"),
            ("m,1.", "初期在庫の値が不正です: '1.'"),
            ("pcs,25.5", "初期在庫の値が不正です: '25.5'"),
            (",25.5", "初期在庫の値が不正です: '25.5'"),
            ("pcs,-1", "初期在庫の値が不正です: '-1'"),
            ("pcs,1000000000", "在庫数が大きすぎます"),
        ];
        let mut cells: Vec<&str> = rejected.iter().map(|(cell, _)| *cell).collect();
        cells.push("pcs,3");
        let preview = preview_unit_rows(&conn, "IR", &cells);
        assert_eq!(preview.valid_rows.len(), 1, "他の行は valid");
        let errors: Vec<Vec<String>> = preview
            .error_rows
            .iter()
            .map(|row| row.errors.clone())
            .collect();
        let expected: Vec<Vec<String>> = rejected
            .iter()
            .map(|(_, message)| vec![message.to_string()])
            .collect();
        assert_eq!(errors, expected);

        // commit: m の 25 は在庫 2500 cm で入る
        let preview = preview_unit_rows(&conn, "IC", &["m,25", "pcs,25"]);
        commit_import(&mut conn, preview.valid_rows, vec![]).unwrap();
        assert_eq!(
            product_stock_and_unit(&conn, "IC-0"),
            (2500, "m".to_string())
        );
        assert_eq!(
            product_stock_and_unit(&conn, "IC-1"),
            (25, "pcs".to_string())
        );
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_revalidates_initial_stock_of_new_rows() {
        // REQ-104 / BIZ-01-D8（X3）: commit は preview を信頼せず、新規の行の初期在庫を 0 以上で上限までと確かめる
        let (_dir, mut conn) = setup_test_db();
        for (stock, message) in [
            (-1, "初期在庫の値が不正です: '-1'"),
            (1_000_000_000, "在庫数が大きすぎます"),
        ] {
            let mut bad = make_import_row("IS-BAD", "初期在庫が範囲外");
            bad.initial_stock = Some(stock);
            let mut good = make_import_row("IS-GOOD", "同じ取込みの行");
            good.initial_stock = Some(5);
            assert_commit_rejected(&mut conn, vec![good, bad], &[], message);
        }
        let mut edge = make_import_row("IS-EDGE", "上限の端");
        edge.initial_stock = Some(999_999_999);
        commit_import(&mut conn, vec![edge], vec![]).unwrap();
        assert_eq!(product_stock_and_unit(&conn, "IS-EDGE").0, 999_999_999);
    }

    #[test]
    #[serial]
    fn test_import_req104_overwrite_rejects_cross_kind_stock_unit() {
        // REQ-104 / SPEC-UNIT-D2 / BIZ-01-D8（X12）: 上書きで種類をまたぐ単位の変更は拒む。
        // preview は行の error、commit は再検証する（長さの単位は m だけ。CSV の cm は行の error: Matrix U4）
        let (_dir, mut conn) = setup_test_db();
        let mut existing = make_import_row("OU-M", "合成の反物");
        existing.stock_unit = Some("m".to_string());
        commit_import(&mut conn, vec![existing], vec![]).unwrap();
        // 在庫 100 cm は入庫の明細（数量 100・原価 333 円〈33300〉）から。単位が変わると、この明細の金額の意味も変わる
        let received = crate::biz::inventory_service::create_receiving(
            &mut conn,
            serde_json::from_value(serde_json::json!({
                "idempotency_key": "ou-m-receiving",
                "supplier_id": null,
                "receiving_date": "2026-10-01",
                "note": null,
                "items": [{"product_code": "OU-M", "quantity": 100, "cost_price_centi": 33300}],
            }))
            .unwrap(),
        )
        .unwrap();
        let line_cost = |conn: &DbConnection| -> String {
            crate::biz::inventory_service::get_receiving_record(conn, received.record_id)
                .unwrap()
                .items[0]
                .line_cost_centi
                .clone()
        };
        assert_eq!(line_cost(&conn), "33300");

        let csv = make_csv(
            UNIT_CSV_HEADERS,
            &[
                "OU-M,合成の反物,1,500,300,10,玉,",
                "OU-NEW,合成の商品,1,500,300,10,玉,",
            ],
        );
        let preview = preview_import(&conn, &csv).unwrap();
        assert_eq!(preview.valid_rows.len(), 1, "他の行は valid");
        assert!(preview.duplicate_rows.is_empty());
        assert_eq!(
            preview.error_rows[0].errors,
            vec!["在庫単位は個数の単位どうしでだけ変えられます（登録: m、CSV: 玉）".to_string()]
        );

        // 個数の商品を m で読む行も行の error
        commit_import(
            &mut conn,
            vec![make_import_row("OU-PCS", "合成の糸")],
            vec![],
        )
        .unwrap();
        let csv = make_csv(UNIT_CSV_HEADERS, &["OU-PCS,合成の糸,1,500,300,10,m,"]);
        let preview = preview_import(&conn, &csv).unwrap();
        assert!(preview.duplicate_rows.is_empty());
        let errors: Vec<Vec<String>> = preview
            .error_rows
            .iter()
            .map(|row| row.errors.clone())
            .collect();
        assert_eq!(
            errors,
            vec![vec![
                "在庫単位は個数の単位どうしでだけ変えられます（登録: 個、CSV: m）".to_string()
            ]]
        );

        // preview を通さない wire
        let mut changed = make_import_row("OU-M", "合成の反物");
        changed.stock_unit = Some("ball".to_string());
        assert_commit_rejected(
            &mut conn,
            vec![make_import_row("OU-OTHER", "同じ取込みの行"), changed],
            &["OU-M"],
            "在庫単位は個数の単位どうしでだけ変えられます（登録: m、CSV: ball）",
        );
        assert_eq!(
            product_stock_and_unit(&conn, "OU-M"),
            (100, "m".to_string())
        );
        assert_eq!(line_cost(&conn), "33300");

        // 同じ単位と空の上書きは成功し、単位は m のまま。上書きは初期在庫を使わない
        for (unit_cell, stock_cell) in [("m", "5"), ("", "")] {
            let csv = make_csv(
                UNIT_CSV_HEADERS,
                &[&format!(
                    "OU-M,上書きの後,1,600,300,10,{unit_cell},{stock_cell}"
                )],
            );
            let preview = preview_import(&conn, &csv).unwrap();
            assert!(preview.error_rows.is_empty(), "{:?}", preview.error_rows);
            let rows: Vec<ImportRow> = preview
                .duplicate_rows
                .into_iter()
                .map(|row| row.import_row)
                .collect();
            let result = commit_import(&mut conn, rows, vec!["OU-M".to_string()]).unwrap();
            assert_eq!(result.updated_count, 1);
            assert_eq!(
                product_stock_and_unit(&conn, "OU-M"),
                (100, "m".to_string())
            );
        }

        // 上書きの行の初期在庫も preview は新規の行と同じ規則で読む（負は行の error）
        let csv = make_csv(UNIT_CSV_HEADERS, &["OU-M,上書きの後,1,600,300,10,,-1"]);
        let preview = preview_import(&conn, &csv).unwrap();
        assert_eq!(
            preview.error_rows[0].errors,
            vec!["初期在庫の値が不正です: '-1'".to_string()]
        );
        // commit は上書きの行の初期在庫を読まない
        let mut ignored = make_import_row("OU-M", "上書きの後");
        ignored.initial_stock = Some(-1);
        commit_import(&mut conn, vec![ignored], vec!["OU-M".to_string()]).unwrap();
        assert_eq!(
            product_stock_and_unit(&conn, "OU-M"),
            (100, "m".to_string())
        );
    }

    /// 商品 CSV の上書きの test 用に、単位 `unit` の商品を新規の行で作る
    fn commit_unit_product(conn: &mut DbConnection, code: &str, unit: &str) {
        let mut row = make_import_row(code, "合成の商品");
        row.stock_unit = Some(unit.to_string());
        commit_import(conn, vec![row], vec![]).unwrap();
    }

    fn product_plu_dirty(conn: &DbConnection, code: &str) -> bool {
        product_repo::find_by_product_code(conn, code)
            .unwrap()
            .unwrap()
            .product
            .plu_dirty
    }

    /// product_update のログの (summary, detail_json) を古い順に返す
    fn product_update_logs(conn: &DbConnection) -> Vec<(String, Option<serde_json::Value>)> {
        let mut statement = conn
            .prepare(
                "SELECT summary, detail_json FROM operation_logs
                 WHERE operation_type = 'product_update' ORDER BY id",
            )
            .unwrap();
        statement
            .query_map([], |row| {
                let detail: Option<String> = row.get(1)?;
                Ok((
                    row.get(0)?,
                    detail.map(|detail| serde_json::from_str(&detail).unwrap()),
                ))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    #[serial]
    fn test_preview_import_req104_overwrite_allows_count_unit_change() {
        // REQ-104 / SPEC-UNIT-D2 / BIZ-01-D8: 個数の単位どうしの上書きは重複行になる（語でも code でも）。preview は書かない
        let (_dir, mut conn) = setup_test_db();
        commit_unit_product(&mut conn, "OC-PCS", "pcs");
        for cell in ["玉", "ball"] {
            let csv = make_csv(
                UNIT_CSV_HEADERS,
                &[&format!("OC-PCS,合成の商品,1,500,300,10,{cell},")],
            );
            let preview = preview_import(&conn, &csv).unwrap();
            assert!(
                preview.error_rows.is_empty(),
                "{cell}: {:?}",
                preview.error_rows
            );
            assert_eq!(preview.duplicate_rows.len(), 1, "{cell}");
            assert_eq!(
                preview.duplicate_rows[0].import_row.stock_unit.as_deref(),
                Some("ball"),
                "{cell}"
            );
            assert_eq!(
                product_stock_and_unit(&conn, "OC-PCS").1,
                "pcs",
                "preview は書かない"
            );
        }
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_overwrite_revalidates_stock_unit_wire() {
        // REQ-104 / BIZ-01-D8: commit は上書きの行の単位を再検証し、外れれば TX 全体を戻す（前の正しい行・ログも残らない）
        let (_dir, mut conn) = setup_test_db();
        commit_unit_product(&mut conn, "RV-PCS", "pcs");
        commit_unit_product(&mut conn, "RV-M", "m");
        commit_unit_product(&mut conn, "RV-OK", "pcs");
        for (code, unit, message) in [
            (
                "RV-PCS",
                "m",
                "在庫単位は個数の単位どうしでだけ変えられます（登録: 個、CSV: m）",
            ),
            // 外した単位の code は preview と同じ直し方の文（retired_stock_unit_hint、Matrix U4）
            (
                "RV-M",
                "cm",
                "在庫単位が正しくありません: cm（長さは m で入れてください。初期在庫も m の数〈cm の数を 100 で割る〉にしてください）",
            ),
            ("RV-PCS", "kg", "在庫単位が正しくありません: kg"),
        ] {
            let mut good = make_import_row("RV-OK", "合成の商品");
            good.stock_unit = Some("ball".to_string());
            let mut bad = make_import_row(code, "合成の商品");
            bad.stock_unit = Some(unit.to_string());
            assert_commit_rejected(
                &mut conn,
                vec![make_import_row("RV-NEW", "同じ取込みの新規"), good, bad],
                &["RV-OK", code],
                message,
            );
            assert!(product_update_logs(&conn).is_empty(), "{message}");
        }
        assert_eq!(product_stock_and_unit(&conn, "RV-OK").1, "pcs");
        assert!(product_repo::find_by_product_code(&conn, "RV-NEW")
            .unwrap()
            .is_none());
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_unit_only_overwrite_keeps_plu_dirty() {
        // REQ-104 / BIZ-01-D8: 単位だけの上書きは plu_dirty を変えない（0 は 0、1 は 1）。任意列が無い・空の CSV でも同じ
        let (_dir, mut conn) = setup_test_db();
        let headers = [
            "商品コード,商品名,部門ID,売価,原価,税率,在庫単位".to_string(),
            "商品コード,商品名,部門ID,売価,原価,税率,在庫単位,JANコード,メーカー品番,取引先ID,POS在庫連動,PLU対象"
                .to_string(),
        ];
        for (header_index, headers) in headers.iter().enumerate() {
            for initial in [false, true] {
                let code = format!("UO-{header_index}-{initial}");
                commit_unit_product(&mut conn, &code, "pcs");
                product_repo::update_product(
                    &conn,
                    &code,
                    &ProductUpdates {
                        plu_target: Some(true),
                        plu_dirty: Some(initial),
                        ..Default::default()
                    },
                )
                .unwrap();
                let extra = if header_index == 0 { "" } else { ",,,,," };
                let csv = make_csv(
                    headers,
                    &[&format!("{code},合成の商品,1,500,300,10,玉{extra}")],
                );
                let preview = preview_import(&conn, &csv).unwrap();
                assert!(preview.error_rows.is_empty(), "{:?}", preview.error_rows);
                let rows: Vec<ImportRow> = preview
                    .duplicate_rows
                    .into_iter()
                    .map(|row| row.import_row)
                    .collect();
                assert_eq!(rows.len(), 1);
                commit_import(&mut conn, rows, vec![code.clone()]).unwrap();
                assert_eq!(product_stock_and_unit(&conn, &code).1, "ball", "{code}");
                assert_eq!(product_plu_dirty(&conn, &code), initial, "{code}");
            }
        }
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_overwrite_with_other_changes_sets_plu_dirty() {
        // REQ-104 / BIZ-01-D8・§4.9 手順 2 d.: 単位と他の 1 列が変わる上書き、単位が空・同じ上書きは plu_dirty=1
        let (_dir, mut conn) = setup_test_db();
        let supplier = seed_named_supplier(&conn, "合成の取引先");
        let other_supplier = seed_named_supplier(&conn, "別の取引先");
        let supplier = supplier.to_string();
        let other_supplier = other_supplier.to_string();
        // 登録と同じ値の列（在庫単位の他）: 商品名・部門・売価・原価・税率・在庫単位・JAN・メーカー品番・取引先・在庫連動・PLU対象
        let cases: [(&str, usize, &str); 12] = [
            ("JAN", 7, ""),
            ("商品名", 1, "別の名前"),
            ("部門", 2, "2"),
            ("売価", 3, "600"),
            ("原価", 4, "350"),
            ("税率", 5, "8"),
            ("取引先", 9, other_supplier.as_str()),
            ("メーカー品番", 8, "MK-2"),
            ("在庫連動", 10, "0"),
            ("PLU対象", 11, "1"),
            ("(b) 単位が空", 6, ""),
            ("(c) 単位が同じで売価が違う", 3, "600"),
        ];
        let mut lines = Vec::new();
        let mut codes = Vec::new();
        for (index, (label, column, value)) in cases.iter().enumerate() {
            let code = format!("OD-{index}");
            let jan = synthetic_ean13_req907(910 + index as u64);
            let mut row = make_import_row(&code, "合成の商品");
            row.jan_code = Some(jan.clone());
            row.maker_code = Some("MK-1".to_string());
            row.supplier_id = Some(supplier.parse().unwrap());
            row.pos_stock_sync = Some(true);
            row.plu_target = Some(false);
            commit_import(&mut conn, vec![row], vec![]).unwrap();
            product_repo::update_product(
                &conn,
                &code,
                &ProductUpdates {
                    plu_dirty: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
            let mut cells = vec![
                code.clone(),
                "合成の商品".to_string(),
                "1".to_string(),
                "500".to_string(),
                "300".to_string(),
                "10".to_string(),
                "玉".to_string(),
                jan,
                "MK-1".to_string(),
                supplier.clone(),
                "1".to_string(),
                "0".to_string(),
            ];
            cells[*column] = value.to_string();
            if label.starts_with("(c)") {
                cells[6] = "個".to_string();
            }
            lines.push(cells.join(","));
            codes.push(code);
        }
        let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率,在庫単位,JANコード,メーカー品番,取引先ID,POS在庫連動,PLU対象",
            &lines,
        );
        let preview = preview_import(&conn, &csv).unwrap();
        assert!(preview.error_rows.is_empty(), "{:?}", preview.error_rows);
        let rows: Vec<ImportRow> = preview
            .duplicate_rows
            .into_iter()
            .map(|row| row.import_row)
            .collect();
        assert_eq!(rows.len(), cases.len());
        commit_import(&mut conn, rows, codes.clone()).unwrap();
        for ((label, _, _), code) in cases.iter().zip(&codes) {
            assert!(
                product_plu_dirty(&conn, code),
                "{label}: plu_dirty=1 になるべき"
            );
        }
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_overwrite_logs_stock_unit_change_per_product() {
        // REQ-104 / BIZ-01-D7・D8: 単位を変えた商品ごとに TX の中で product_update を 1 行（key は stock_unit だけ）
        let (_dir, mut conn) = setup_test_db();
        for code in ["LG-A", "LG-B", "LG-C"] {
            commit_unit_product(&mut conn, code, "pcs");
        }
        let mut a = make_import_row("LG-A", "合成の商品");
        a.stock_unit = Some("ball".to_string());
        let mut b = make_import_row("LG-B", "改名した商品");
        b.stock_unit = Some("bag".to_string());
        let c = make_import_row("LG-C", "名前だけ変えた商品");
        commit_import(
            &mut conn,
            vec![a, b, c],
            vec!["LG-A".to_string(), "LG-B".to_string(), "LG-C".to_string()],
        )
        .unwrap();
        assert_eq!(
            product_update_logs(&conn),
            vec![
                (
                    "商品を更新しました: LG-A".to_string(),
                    Some(serde_json::json!({"stock_unit": {"old": "pcs", "new": "ball"}}))
                ),
                (
                    "商品を更新しました: LG-B".to_string(),
                    Some(serde_json::json!({"stock_unit": {"old": "pcs", "new": "bag"}}))
                ),
            ]
        );
        let imports: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'product_import'
                 AND summary = '商品一括インポート: 新規0件、更新3件'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(imports, 1, "件数の要約のログは今のまま");
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_overwrite_log_failure_rolls_back_all() {
        // REQ-104 / BIZ-01-D8: 上書きの product_update のログの INSERT が失敗すると TX 全体が戻る
        let (_dir, mut conn) = setup_test_db();
        commit_unit_product(&mut conn, "LF-PCS", "pcs");
        conn.execute_batch(
            "CREATE TRIGGER fail_product_update_log
             BEFORE INSERT ON operation_logs
             WHEN NEW.operation_type = 'product_update'
             BEGIN SELECT RAISE(ABORT, 'synthetic operation log failure'); END;",
        )
        .unwrap();
        let before = import_side_effects(&conn);
        let mut changed = make_import_row("LF-PCS", "合成の商品");
        changed.stock_unit = Some("ball".to_string());
        let result = commit_import(
            &mut conn,
            vec![make_import_row("LF-NEW", "同じ取込みの新規"), changed],
            vec!["LF-PCS".to_string()],
        );
        assert!(result.is_err(), "{result:?}");
        assert_eq!(import_side_effects(&conn), before);
        assert_eq!(product_stock_and_unit(&conn, "LF-PCS").1, "pcs");
        assert!(product_repo::find_by_product_code(&conn, "LF-NEW")
            .unwrap()
            .is_none());
    }

    #[test]
    #[serial]
    fn test_import_req104_price_limits_in_preview_and_commit() {
        // REQ-104 / 共通規則 (a)（X5・X6）/ BIZ-01-D8 / Matrix C3: 売価は 0 以上 999999、原価は 0 以上 999999.99 円（99999999）。commit は新規と上書きの全行を再検証する
        let (_dir, mut conn) = setup_test_db();
        let csv = make_csv(
            "商品コード,商品名,部門ID,売価,原価,税率",
            &[
                "PL-EDGE,上限の端,1,999999,999999.99,10",
                "PL-SELL,売価が外,1,1000000,300,10",
                "PL-COST,原価が外,1,500,1000000,10",
            ],
        );
        let preview = preview_import(&conn, &csv).unwrap();
        assert_eq!(preview.valid_rows.len(), 1);
        let errors: Vec<Vec<String>> = preview
            .error_rows
            .iter()
            .map(|row| row.errors.clone())
            .collect();
        assert_eq!(
            errors,
            vec![
                vec!["売価が大きすぎます".to_string()],
                vec!["原価が大きすぎます".to_string()]
            ]
        );
        commit_import(&mut conn, preview.valid_rows, vec![]).unwrap();

        let with_prices = |code: &str, selling_price: i64, cost_price_centi: i64| ImportRow {
            selling_price,
            cost_price_centi,
            ..make_import_row(code, "再検証の行")
        };
        // (i) 新規の行 (ii)〜(v) 上書きの行。どれも同じ取込みのほかの行を含めて何も変わらない
        for (code, selling_price, cost_price_centi, overwrite, message) in [
            ("PL-NEW", 1_000_000, 300, false, "売価が大きすぎます"),
            ("PL-NEW", 500, 100_000_000, false, "原価が大きすぎます"),
            ("PL-NEW", -1, 300, false, "売価は0以上で入力してください"),
            ("PL-NEW", 500, -1, false, "原価は0以上で入力してください"),
            ("PL-EDGE", 1_000_000, 300, true, "売価が大きすぎます"),
            ("PL-EDGE", 500, 100_000_000, true, "原価が大きすぎます"),
            ("PL-EDGE", -1, 300, true, "売価は0以上で入力してください"),
            ("PL-EDGE", 500, -1, true, "原価は0以上で入力してください"),
        ] {
            let overwrite_codes: &[&str] = if overwrite { &["PL-EDGE"] } else { &[] };
            assert_commit_rejected(
                &mut conn,
                vec![
                    make_import_row("PL-OTHER", "同じ取込みの行"),
                    with_prices(code, selling_price, cost_price_centi),
                ],
                overwrite_codes,
                message,
            );
        }

        // 上書きの行の上限の端は成功して保存される
        commit_import(
            &mut conn,
            vec![with_prices("PL-EDGE", 999_999, 99_999_999)],
            vec!["PL-EDGE".to_string()],
        )
        .unwrap();
        let prices: (i64, i64) = conn
            .query_row(
                "SELECT selling_price, cost_price_centi FROM products WHERE product_code = 'PL-EDGE'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(prices, (999_999, 99_999_999));
    }

    #[test]
    #[serial]
    fn test_import_req104_cost_column_parses_two_decimals() {
        // REQ-104 / SPEC-UNIT-D5・D10 / Y2 / Matrix C7: 原価は小数 2 桁までの円を浮動小数を通さず 1/100 円にする
        let (_dir, conn) = setup_test_db();
        let accepted = [
            ("83.33", 8333),
            ("1000", 100_000),
            ("0", 0),
            ("0.5", 50),
            ("0.29", 29), // f64 で掛けると 28
            ("999999.99", 99_999_999),
        ];
        let format_error = "原価は0以上で、小数は2桁までで入力してください";
        let rejected = [
            ("83.333", format_error),
            ("1e3", format_error),
            ("+5", format_error),
            (".5", format_error),
            ("5.", format_error),
            ("", format_error),
            ("-1", "原価は0以上で入力してください"),
            ("-0", "原価は0以上で入力してください"),
            ("1000000", "原価が大きすぎます"),
            ("99999999999999999999", "原価が大きすぎます"),
        ];
        let rows: Vec<String> = accepted
            .iter()
            .map(|(cost, _)| *cost)
            .chain(rejected.iter().map(|(cost, _)| *cost))
            .enumerate()
            .map(|(index, cost)| format!("C7-{index:02},合成の商品,1,500,{cost},10"))
            .collect();
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let preview = preview_import(
            &conn,
            &make_csv("商品コード,商品名,部門ID,売価,原価,税率", &rows),
        )
        .unwrap();
        let costs: Vec<i64> = preview
            .valid_rows
            .iter()
            .map(|row| row.cost_price_centi)
            .collect();
        assert_eq!(
            costs,
            accepted.iter().map(|(_, centi)| *centi).collect::<Vec<_>>()
        );
        let errors: Vec<Vec<String>> = preview
            .error_rows
            .iter()
            .map(|row| row.errors.clone())
            .collect();
        assert_eq!(
            errors,
            rejected
                .iter()
                .map(|(_, message)| vec![message.to_string()])
                .collect::<Vec<_>>()
        );
    }

    #[test]
    #[serial]
    fn test_import_req104_stock_unit_accepts_ten_codes_and_words() {
        // REQ-104 / SPEC-UNIT-D10・D-128 / Matrix U4: 10 個の code と語は通り、外した巻・cm は直し方を添えた行の error
        let (_dir, conn) = setup_test_db();
        let ten = [
            "pcs",
            "個",
            "sheet",
            "枚",
            "hon",
            "本",
            "bag",
            "袋",
            "box",
            "箱",
            "kumi",
            "組",
            "set",
            "セット",
            "ball",
            "玉",
            "cho",
            "丁",
            "m",
        ];
        let cells: Vec<String> = ten.iter().map(|unit| format!("{unit},")).collect();
        let cells: Vec<&str> = cells.iter().map(String::as_str).collect();
        let preview = preview_unit_rows(&conn, "U4OK", &cells);
        assert!(preview.error_rows.is_empty(), "{:?}", preview.error_rows);
        assert_eq!(preview.valid_rows.len(), ten.len());

        let preview = preview_unit_rows(&conn, "U4NG", &["cm,130", "巻,3", "roll,3", "kg,3"]);
        let errors: Vec<Vec<String>> = preview
            .error_rows
            .iter()
            .map(|row| row.errors.clone())
            .collect();
        assert_eq!(
            errors,
            vec![
                vec!["在庫単位が正しくありません: cm（長さは m で入れてください。初期在庫も m の数〈cm の数を 100 で割る〉にしてください）".to_string()],
                vec!["在庫単位が正しくありません: 巻（巻で数える商品は 個 で入れてください。数は変わりません）".to_string()],
                vec!["在庫単位が正しくありません: roll（巻で数える商品は 個 で入れてください。数は変わりません）".to_string()],
                vec!["在庫単位が正しくありません: kg".to_string()],
            ]
        );
    }

    #[test]
    #[serial]
    fn test_product_req101_cost_centi_upper_bound() {
        // REQ-101 / REQ-102 / REQ-105 / SPEC-UNIT-D5 / Matrix C1: 原価 99999999 は受け、100000000 と負は拒む
        let (_dir, mut conn) = setup_test_db();
        let created = create_product(
            &mut conn,
            ProductCreateRequest {
                cost_price_centi: 99_999_999,
                ..default_create_request()
            },
        )
        .unwrap();
        let code = created.product_code;
        let stored_cost = |conn: &DbConnection| {
            product_repo::find_by_product_code(conn, &code)
                .unwrap()
                .unwrap()
                .product
                .cost_price_centi
        };
        assert_eq!(stored_cost(&conn), 99_999_999);
        for (cost, message) in [
            (100_000_000, "原価が大きすぎます"),
            (-1, "原価は0以上で入力してください"),
        ] {
            let create = create_product(
                &mut conn,
                ProductCreateRequest {
                    cost_price_centi: cost,
                    ..default_create_request()
                },
            );
            let update = update_product(
                &mut conn,
                &code,
                &ProductUpdateRequest {
                    cost_price_centi: Some(cost),
                    ..Default::default()
                },
            );
            let revise = revise_product_price(
                &mut conn,
                PriceRevisionInput {
                    product_code: code.clone(),
                    new_selling_price: 500,
                    new_cost_price_centi: cost,
                    assign_supplier_id: None,
                },
            );
            for (path, result) in [
                ("create", create.map(|_| ())),
                ("update", update.map(|_| ())),
                ("revise", revise.map(|_| ())),
            ] {
                assert!(
                    matches!(result, Err(BizError::ValidationFailed(ref m)) if m == message),
                    "{path} {cost}: {result:?}"
                );
            }
            assert_eq!(stored_cost(&conn), 99_999_999, "{cost}");
        }
        update_product(
            &mut conn,
            &code,
            &ProductUpdateRequest {
                cost_price_centi: Some(8333),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(stored_cost(&conn), 8333);
    }

    #[test]
    #[serial]
    fn test_product_req102_req105_not_found_precedes_cost_limit() {
        // REQ-102 / REQ-105 / 30 §4.4 手順 1 / Matrix C22: 存在しない商品は原価の検査より先に NotFound で、何も書かない
        let (_dir, mut conn) = setup_test_db();
        let counts = |conn: &DbConnection| -> Vec<i64> {
            ["products", "price_history", "operation_logs"]
                .iter()
                .map(|table| {
                    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })
                    .unwrap()
                })
                .collect()
        };
        let before = counts(&conn);
        let update = update_product(
            &mut conn,
            "C22-NONE",
            &ProductUpdateRequest {
                cost_price_centi: Some(100_000_000),
                ..Default::default()
            },
        );
        assert!(matches!(update, Err(BizError::NotFound(_))), "{update:?}");
        let revise = revise_product_price(
            &mut conn,
            PriceRevisionInput {
                product_code: "C22-NONE".to_string(),
                new_selling_price: 500,
                new_cost_price_centi: 100_000_000,
                assign_supplier_id: None,
            },
        );
        assert!(matches!(revise, Err(BizError::NotFound(_))), "{revise:?}");
        assert_eq!(counts(&conn), before);
    }

    #[test]
    fn test_list_low_stock_req302_unit_groups_cover_every_unit_once() {
        // REQ-302 / SPEC-UNIT-D10: BIZ が repo へ渡す 2 つの並びは合わせて 10 個で重なりが無い（個数 9・長さ 1）
        let (count_units, length_units) = low_stock_unit_groups();
        assert_eq!(count_units.len(), 9);
        assert_eq!(length_units, vec![ProductStockUnit::M]);
        let mut all: Vec<&str> = count_units
            .iter()
            .chain(&length_units)
            .map(|unit| unit.as_str())
            .collect();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), ProductStockUnit::ALL.len());
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_overwrite() {
        // REQ-104: 一括インポート — 上書き対象の更新
        // BIZ-01 §4.8: 上書き対象の更新
        let (_dir, mut conn) = setup_test_db();
        // 既存商品を作成
        let existing = make_import_row("CO-001", "旧名前");
        commit_import(&mut conn, vec![existing], vec![]).unwrap();

        // 上書き
        let updated = ImportRow {
            name: "新名前".to_string(),
            selling_price: 999,
            ..make_import_row("CO-001", "新名前")
        };
        let result = commit_import(&mut conn, vec![updated], vec!["CO-001".to_string()]).unwrap();
        assert_eq!(result.updated_count, 1);
        assert_eq!(result.created_count, 0);

        let name: String = conn
            .query_row(
                "SELECT name FROM products WHERE product_code = 'CO-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "新名前");
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_overwrite_preserves_optional_fields() {
        // REQ-104: 一括インポート — 上書き時に任意列未指定は既存値保持
        // BIZ-01 §4.8 R3-2: 上書き時に任意列未指定 → 既存値保持
        let (_dir, mut conn) = setup_test_db();
        // 既存商品を作成（stock_unit="m", supplier_id=None）
        let mut existing = make_import_row("OP-001", "既存商品");
        existing.stock_unit = Some("m".to_string());
        commit_import(&mut conn, vec![existing], vec![]).unwrap();

        // 上書き: stock_unit/pos_stock_sync は None（変更しない）
        let updated = make_import_row("OP-001", "更新商品"); // stock_unit = None
        commit_import(&mut conn, vec![updated], vec!["OP-001".to_string()]).unwrap();

        let stock_unit: String = conn
            .query_row(
                "SELECT stock_unit FROM products WHERE product_code = 'OP-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stock_unit, "m", "stock_unit は既存値 'm' が保持されるべき");
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_derives_plu_target_like_backfill_and_keeps_on_overwrite() {
        // REQ-104 / REQ-402: 新規は13桁数字JANだけPLU対象化し、上書きでは既存値を保持する。
        let (_dir, mut conn) = setup_test_db();
        let mut valid = make_import_row("IMP-VALID", "有効JAN");
        valid.jan_code = Some("4901234567894".to_string());
        let mut no_jan = make_import_row("IMP-NO-JAN", "JANなし");
        no_jan.jan_code = None;
        // 12 桁と英字の JAN は §4.9 手順 0（BIZ-01-D10 F3）で TX の前に止まる（DB を変えない）
        for jan in ["123456789012", "49012345678A4"] {
            let mut bad = make_import_row("IMP-BAD-JAN", "形の誤ったJAN");
            bad.jan_code = Some(jan.to_string());
            let before = product_count(&conn);
            let result = commit_import(&mut conn, vec![bad], vec![]);
            assert!(
                matches!(result, Err(BizError::ValidationFailed(ref msg)) if *msg == shape_f3(jan)),
                "{result:?}"
            );
            assert_eq!(product_count(&conn), before, "{jan}");
        }
        let mut short = make_import_row("IMP-SHORT", "8桁");
        short.jan_code = Some("96385074".to_string());

        commit_import(&mut conn, vec![valid, no_jan, short], vec![]).unwrap();

        for (code, expected) in [
            ("IMP-VALID", true),
            ("IMP-NO-JAN", false),
            ("IMP-SHORT", false),
        ] {
            let product = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap();
            assert_eq!(product.product.plu_target, expected, "{code}");
        }

        let mut overwrite = make_import_row("IMP-VALID", "有効JAN更新");
        // IMP-SHORT と別の正しい 8 桁（在庫連動の商品どうしで JAN を共有しない。30 の D-118 の節）
        overwrite.jan_code = Some("49123456".to_string());
        commit_import(&mut conn, vec![overwrite], vec!["IMP-VALID".to_string()]).unwrap();

        let overwritten = product_repo::find_by_product_code(&conn, "IMP-VALID")
            .unwrap()
            .unwrap();
        assert!(
            overwritten.product.plu_target,
            "overwrite keeps existing plu_target even when incoming JAN is not targetable"
        );
    }

    #[test]
    #[serial]
    fn test_commit_import_req907_blank_plu_target_derives_for_new_and_preserves_overwrite() {
        let (_dir, mut conn) = setup_test_db();
        let mut existing = make_import_row("PLU-C4-KEEP", "既存対象外");
        existing.jan_code = Some("2000000000015".to_string());
        existing.plu_target = Some(false);
        commit_import(&mut conn, vec![existing], vec![]).unwrap();

        let mut new_default = make_import_row("PLU-C4-NEW", "既定対象");
        new_default.jan_code = Some("2000000000039".to_string());
        // チェックディジットの合わない JAN の行は §4.9 手順 0（BIZ-01-D10）で止まる。
        // should_default_plu_target の互換は test_should_default_plu_target_req907_preserves_invalid_check_digit_compatibility
        let mut overwrite_blank = make_import_row("PLU-C4-KEEP", "空欄上書き");
        overwrite_blank.jan_code = Some("2000000000015".to_string());
        commit_import(
            &mut conn,
            vec![new_default, overwrite_blank],
            vec!["PLU-C4-KEEP".to_string()],
        )
        .unwrap();

        assert!(
            product_repo::find_by_product_code(&conn, "PLU-C4-NEW")
                .unwrap()
                .unwrap()
                .product
                .plu_target
        );
        assert!(
            !product_repo::find_by_product_code(&conn, "PLU-C4-KEEP")
                .unwrap()
                .unwrap()
                .product
                .plu_target
        );
    }

    #[test]
    #[serial]
    fn test_commit_import_req907_applies_explicit_plu_target_and_releases_old_slot() {
        let (_dir, mut conn) = setup_test_db();
        let old_jan = "2000000000015";
        let new_jan = "2000000000039";

        let mut turn_off = make_import_row("PLU-C5-OFF", "対象解除");
        turn_off.jan_code = Some(old_jan.to_string());
        turn_off.plu_target = Some(true);
        let mut turn_on = make_import_row("PLU-C5-ON", "対象化");
        turn_on.jan_code = Some(new_jan.to_string());
        turn_on.plu_target = Some(false);
        commit_import(&mut conn, vec![turn_off, turn_on], vec![]).unwrap();
        conn.execute(
            "UPDATE plu_slots SET scanning_code=?1, status='reserved', reserved_at='2026-08-19T00:00:00' WHERE memory_no=217",
            rusqlite::params![old_jan],
        )
        .unwrap();

        let mut off_update = make_import_row("PLU-C5-OFF", "対象解除後");
        off_update.jan_code = Some(old_jan.to_string());
        off_update.plu_target = Some(false);
        let mut on_update = make_import_row("PLU-C5-ON", "対象化後");
        on_update.jan_code = Some(new_jan.to_string());
        on_update.plu_target = Some(true);
        let mut explicit_new_off = make_import_row("PLU-C5-NEW-OFF", "新規対象外");
        explicit_new_off.jan_code = Some("2000000000046".to_string());
        explicit_new_off.plu_target = Some(false);
        commit_import(
            &mut conn,
            vec![off_update, on_update, explicit_new_off],
            vec!["PLU-C5-OFF".to_string(), "PLU-C5-ON".to_string()],
        )
        .unwrap();

        let off = product_repo::find_by_product_code(&conn, "PLU-C5-OFF")
            .unwrap()
            .unwrap();
        let on = product_repo::find_by_product_code(&conn, "PLU-C5-ON")
            .unwrap()
            .unwrap();
        let new_off = product_repo::find_by_product_code(&conn, "PLU-C5-NEW-OFF")
            .unwrap()
            .unwrap();
        assert!(!off.product.plu_target);
        assert!(on.product.plu_target && on.product.plu_dirty);
        assert!(!new_off.product.plu_target);

        let slot: (String, Option<String>) = conn
            .query_row(
                "SELECT status, scanning_code FROM plu_slots WHERE memory_no=217",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(slot, ("free".to_string(), None));
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_initial_stock_movement() {
        // REQ-104: 一括インポート — initial_stock > 0 → movement記録
        // BIZ-01 §4.8: initial_stock > 0 → movement記録
        let (_dir, mut conn) = setup_test_db();
        let mut row = make_import_row("IM-001", "初期在庫あり");
        row.initial_stock = Some(10);
        commit_import(&mut conn, vec![row], vec![]).unwrap();

        let qty: i64 = conn
            .query_row(
                "SELECT quantity FROM inventory_movements WHERE product_code = 'IM-001' AND movement_type = 'receiving'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(qty, 10);
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_initial_stock_movement_and_copy_match() {
        // REQ-104 / 30（新規行の初期在庫の movement と求め直し、上書きの行は在庫を変えない）/ Matrix E3
        let (_dir, mut conn) = setup_test_db();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut row = make_import_row("IM-E3", "初期在庫 4");
        row.initial_stock = Some(4);
        commit_import(&mut conn, vec![row], vec![]).unwrap();
        let expected = ((4, false, None, None), vec![(4, today, 4)]);
        assert_eq!(copy_and_initial_movements(&conn, "IM-E3"), expected);

        // 上書きの行は初期在庫を見ず、在庫も movement も変えない
        let mut row = make_import_row("IM-E3", "上書き");
        row.initial_stock = Some(99);
        commit_import(&mut conn, vec![row], vec!["IM-E3".to_string()]).unwrap();
        assert_eq!(copy_and_initial_movements(&conn, "IM-E3"), expected);

        let mut row = make_import_row("IM-E3-ZERO", "初期在庫 0");
        row.initial_stock = Some(0);
        commit_import(&mut conn, vec![row], vec![]).unwrap();
        assert_eq!(
            copy_and_initial_movements(&conn, "IM-E3-ZERO"),
            ((0, false, None, None), vec![])
        );
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_active_stocktake() {
        // REQ-104: 一括インポート — 進行中棚卸し → stocktake_item自動追加
        // BIZ-01 §4.8: 進行中棚卸し → stocktake_item自動追加
        let (_dir, mut conn) = setup_test_db();
        // 棚卸しを直接作成
        conn.execute(
            "INSERT INTO stocktakes (started_at, status) VALUES ('2026-10-01T09:00:00', 'in_progress')",
            [],
        )
        .unwrap();

        let row = make_import_row("AS-001", "棚卸し中追加");
        commit_import(&mut conn, vec![row], vec![]).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM stocktake_items WHERE product_code = 'AS-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "stocktake_item が自動追加されるべき");
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_no_stocktake() {
        // REQ-104: 一括インポート — 棚卸しなし → stocktake_item作成なし
        // BIZ-01 §4.8: 棚卸しなし → stocktake_item作成なし
        let (_dir, mut conn) = setup_test_db();
        let row = make_import_row("NS-001", "棚卸しなし");
        commit_import(&mut conn, vec![row], vec![]).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM stocktake_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_operation_log() {
        // REQ-104: 一括インポート — 操作ログ記録
        // BIZ-01 §4.8: 操作ログ記録
        let (_dir, mut conn) = setup_test_db();
        let row = make_import_row("OL-001", "ログテスト");
        commit_import(&mut conn, vec![row], vec![]).unwrap();

        let op_type: String = conn
            .query_row(
                "SELECT operation_type FROM operation_logs WHERE operation_type = 'product_import' ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(op_type, "product_import");
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_empty_rows() {
        // REQ-104: 一括インポート — 空配列は created_count=0
        // BIZ-01 §4.8: 空配列
        let (_dir, mut conn) = setup_test_db();
        let result = commit_import(&mut conn, vec![], vec![]).unwrap();
        assert_eq!(result.created_count, 0);
        assert_eq!(result.updated_count, 0);
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_rejects_overlong_product_code_without_writes() {
        // REQ-104 / BIZ-01-D5: wire から来た行も TX の前に再検証し、DB を変えない
        let (_dir, mut conn) = setup_test_db();
        let counts = |conn: &DbConnection| -> (i64, i64, i64) {
            let count = |table: &str| -> i64 {
                conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap()
            };
            (
                count("products"),
                count("operation_logs"),
                count("inventory_movements"),
            )
        };
        let before = counts(&conn);
        let mut good_row = make_import_row("LEN-OK", "正常商品");
        good_row.initial_stock = Some(5);
        let long_row = make_import_row(&"A".repeat(101), "長すぎる商品");

        let result = commit_import(&mut conn, vec![good_row, long_row], vec![]);

        assert!(matches!(
            result,
            Err(BizError::ValidationFailed(ref msg))
                if msg == "商品コードは100文字以内で入力してください"
        ));
        assert_eq!(counts(&conn), before);
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_revalidates_code_shape_before_tx() {
        // REQ-104 / 30 §4.9 手順 0 / BIZ-01-D10: preview を通らない wire の行は TX の前に、
        // 最初に当たった行の最初の文で止め、DB を変えない（正しい行も入らない）
        let (_dir, mut conn) = setup_test_db();
        seed_jan_product(&conn, "SHAPE-OWN", None, true);
        let counts = |conn: &DbConnection| -> (i64, i64) {
            let count = |table: &str| -> i64 {
                conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap()
            };
            (count("products"), count("inventory_movements"))
        };
        let shaped = |code: &str, jan: Option<&str>| {
            let mut row = make_import_row(code, "形の誤った行");
            row.jan_code = jan.map(str::to_string);
            row
        };
        let cases = [
            (
                shaped("4.9E+12", Some("4.9E+12")),
                shape_f1("商品コード", "4.9E+12"),
            ),
            (shaped("4901234567887", None), shape_f2("4901234567887")),
            (shaped("SHAPE-F3", Some("4912345")), shape_f3("4912345")),
            (
                shaped("SHAPE-F4", Some("4901234567890")),
                shape_f4("4901234567890"),
            ),
        ];
        let before = counts(&conn);
        for (bad_row, expected) in cases {
            let mut good_row = make_import_row("SHAPE-OK", "正しい行");
            good_row.initial_stock = Some(5);
            let result = commit_import(&mut conn, vec![good_row, bad_row], vec![]);
            assert!(
                matches!(result, Err(BizError::ValidationFailed(ref msg)) if *msg == expected),
                "{result:?}"
            );
            assert_eq!(counts(&conn), before, "{expected}");
        }

        // 上書きの行でも同じ
        let result = commit_import(
            &mut conn,
            vec![shaped("SHAPE-OWN", Some("4901234567890"))],
            vec!["SHAPE-OWN".to_string()],
        );
        assert!(
            matches!(result, Err(BizError::ValidationFailed(ref msg)) if *msg == shape_f4("4901234567890")),
            "{result:?}"
        );
        assert_eq!(counts(&conn), before);
        let existing = product_repo::find_by_product_code(&conn, "SHAPE-OWN")
            .unwrap()
            .unwrap();
        assert_eq!(existing.product.jan_code, None);
    }

    #[test]
    #[serial]
    fn test_commit_import_req104_rollback_on_failure() {
        // REQ-104: 一括インポート — FK違反で全行ロールバック
        // BIZ-01 §4.8 P3-5: FK違反で全行ロールバック
        let (_dir, mut conn) = setup_test_db();
        let good_row = make_import_row("RB-001", "正常商品");
        let mut bad_row = make_import_row("RB-002", "不正商品");
        bad_row.department_id = 9999; // 存在しない部門 → FK違反

        let result = commit_import(&mut conn, vec![good_row, bad_row], vec![]);
        assert!(result.is_err(), "FK違反でエラーを返すべき");

        // RB-001 もロールバックされている
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE product_code = 'RB-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "TX全体がロールバックされるべき");
    }

    #[test]
    #[serial]
    fn test_bulk_set_plu_target_req907_updates_all_filtered_rows_without_paging() {
        let (_dir, mut conn) = setup_test_db();
        for index in 0..250 {
            let matched = index < 230;
            let code = format!("BL1-{index:03}");
            let name = if matched {
                "BULK907 対象"
            } else {
                "対象外"
            };
            let jan = synthetic_ean13_req907(200_000_000_000 + index);
            insert_bulk_product_req907(&conn, &code, name, Some(&jan), 2, (false, false, false));
        }

        let result = bulk_set_plu_target(&mut conn, bulk_filter_req907("BULK907"), true).unwrap();
        let updated: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE name LIKE '%BULK907%' AND plu_target=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result.matched_count, 230);
        assert_eq!(result.updated_count, 230);
        assert_eq!(updated, 230);
    }

    #[test]
    #[serial]
    fn test_bulk_set_plu_target_req907_skips_invalid_jan_and_discontinued() {
        let (_dir, mut conn) = setup_test_db();
        let valid_a = synthetic_ean13_req907(210_000_000_001);
        let valid_b = synthetic_ean13_req907(210_000_000_002);
        let valid_c = synthetic_ean13_req907(210_000_000_003);
        let rows = [
            ("BL2-A", Some(valid_a.as_str()), false),
            ("BL2-B", Some(valid_b.as_str()), false),
            ("BL2-NONE", None, false),
            ("BL2-EIGHT", Some("49123456"), false),
            ("BL2-BAD", Some("2000000000047"), false),
            ("BL2-DISC", Some(valid_c.as_str()), true),
        ];
        for (code, jan, discontinued) in rows {
            insert_bulk_product_req907(
                &conn,
                code,
                "BL2 FILTER",
                jan,
                2,
                (discontinued, false, false),
            );
        }

        let result =
            bulk_set_plu_target(&mut conn, bulk_filter_req907("BL2 FILTER"), true).unwrap();
        assert_eq!(result.matched_count, 6);
        assert_eq!(result.updated_count, 2);
        assert_eq!(result.invalid_jan_skipped_count, 3);
        assert_eq!(result.discontinued_skipped_count, 1);
        let targets: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE name='BL2 FILTER' AND plu_target=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(targets, 2);
    }

    #[test]
    #[serial]
    fn test_bulk_set_plu_target_req907_preserves_existing_targets() {
        let (_dir, mut conn) = setup_test_db();
        for (index, target, dirty) in [(0, true, false), (1, true, true), (2, false, false)] {
            let code = format!("BL3-{index}");
            let jan = synthetic_ean13_req907(220_000_000_000 + index);
            insert_bulk_product_req907(
                &conn,
                &code,
                "BL3 FILTER",
                Some(&jan),
                2,
                (false, target, dirty),
            );
            conn.execute(
                "UPDATE products SET updated_at='2000-01-01T00:00:00' WHERE product_code=?1",
                [&code],
            )
            .unwrap();
        }
        let result =
            bulk_set_plu_target(&mut conn, bulk_filter_req907("BL3 FILTER"), true).unwrap();
        let rows: Vec<(String, bool, bool, String)> = {
            let mut stmt = conn
                .prepare("SELECT product_code, plu_target, plu_dirty, updated_at FROM products WHERE name='BL3 FILTER' ORDER BY product_code")
                .unwrap();
            stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
        };
        assert_eq!(result.updated_count, 1);
        assert_eq!(
            (rows[0].1, rows[0].2, rows[0].3.as_str()),
            (true, false, "2000-01-01T00:00:00")
        );
        assert_eq!(
            (rows[1].1, rows[1].2, rows[1].3.as_str()),
            (true, true, "2000-01-01T00:00:00")
        );
        assert_eq!((rows[2].1, rows[2].2), (true, true));
        assert_ne!(rows[2].3, "2000-01-01T00:00:00");
    }

    #[test]
    #[serial]
    fn test_bulk_set_plu_target_req907_turns_off_and_releases_slots() {
        let (_dir, mut conn) = setup_test_db();
        let shared = synthetic_ean13_req907(230_000_000_001);
        let active = synthetic_ean13_req907(230_000_000_002);
        insert_bulk_product_req907(
            &conn,
            "BL4-A",
            "BL4 FILTER",
            Some(&shared),
            2,
            (false, true, true),
        );
        insert_bulk_product_req907(
            &conn,
            "BL4-B",
            "BL4 FILTER",
            Some(&active),
            2,
            (false, true, false),
        );
        insert_bulk_product_req907(
            &conn,
            "BL4-C",
            "BL4 FILTER",
            Some(&shared),
            2,
            (false, true, true),
        );
        insert_bulk_product_req907(&conn, "BL4-D", "BL4 FILTER", None, 2, (false, false, false));
        conn.execute("UPDATE plu_slots SET scanning_code=?1,status='reserved',reserved_at='2026-08-19T00:00:00' WHERE memory_no=300", [&shared]).unwrap();
        conn.execute("UPDATE plu_slots SET scanning_code=?1,status='active',activated_at='2026-08-19T00:00:00' WHERE memory_no=301", [&active]).unwrap();

        let result =
            bulk_set_plu_target(&mut conn, bulk_filter_req907("BL4 FILTER"), false).unwrap();
        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE name='BL4 FILTER' AND plu_target=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let reserved: (Option<String>, String) = conn
            .query_row(
                "SELECT scanning_code,status FROM plu_slots WHERE memory_no=300",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let active_status: String = conn
            .query_row(
                "SELECT status FROM plu_slots WHERE memory_no=301",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(result.updated_count, 3);
        assert_eq!(remaining, 0);
        assert_eq!(reserved, (None, "free".to_string()));
        assert_eq!(active_status, "release_pending");
    }

    #[test]
    #[serial]
    fn test_bulk_set_plu_target_req907_rolls_back_products_slots_and_log() {
        let (_dir, mut conn) = setup_test_db();
        let jan_a = synthetic_ean13_req907(240_000_000_001);
        let jan_b = synthetic_ean13_req907(240_000_000_002);
        for (code, jan) in [("BL5-A", &jan_a), ("BL5-B", &jan_b)] {
            insert_bulk_product_req907(
                &conn,
                code,
                "BL5 FILTER",
                Some(jan),
                2,
                (false, true, true),
            );
        }
        conn.execute(
            "UPDATE plu_slots SET scanning_code=?1,status='reserved' WHERE memory_no=310",
            [&jan_a],
        )
        .unwrap();
        let before_logs: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        let _guard = failpoint::arm(&failpoint::BULK_SET_PLU_TARGET_AFTER_SECOND_UPDATE);
        assert!(bulk_set_plu_target(&mut conn, bulk_filter_req907("BL5 FILTER"), false).is_err());
        let targets: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE name='BL5 FILTER' AND plu_target=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let slot: (Option<String>, String) = conn
            .query_row(
                "SELECT scanning_code,status FROM plu_slots WHERE memory_no=310",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let after_logs: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(targets, 2);
        assert_eq!(slot, (Some(jan_a), "reserved".to_string()));
        assert_eq!(after_logs, before_logs);
    }

    #[test]
    #[serial]
    fn test_bulk_set_plu_target_req907_logs_normalized_filter_without_jans() {
        let (_dir, mut conn) = setup_test_db();
        let jan = synthetic_ean13_req907(250_000_000_001);
        insert_bulk_product_req907(
            &conn,
            "BL6-A",
            "BL6 FILTER",
            Some(&jan),
            2,
            (false, false, false),
        );
        let result =
            bulk_set_plu_target(&mut conn, bulk_filter_req907("  BL6 FILTER  "), true).unwrap();
        let (operation_type, summary, detail): (String, String, String) = conn.query_row(
            "SELECT operation_type,summary,detail_json FROM operation_logs ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
        assert_eq!(operation_type, "product_bulk_plu_target");
        assert!(summary.contains("一致1件、更新1件、JAN不備0件、廃番0件"));
        assert!(detail.contains("\"keyword\":\"BL6 FILTER\""));
        assert!(detail.contains("\"plu_target\":true"));
        assert_eq!(result.matched_count, 1);
        assert!(!summary.contains(&jan));
        assert!(!detail.contains(&jan));
    }

    #[test]
    #[serial]
    fn test_bulk_set_plu_target_req907_matches_search_plu_filter() {
        use crate::db::product_repo::{PluMigrationFilter, SortKey, SortOrder};
        let (_dir, mut conn) = setup_test_db();
        for (index, target, dirty) in [
            (0, true, true),
            (1, true, true),
            (2, true, false),
            (3, false, false),
        ] {
            let jan = synthetic_ean13_req907(260_000_000_000 + index);
            insert_bulk_product_req907(
                &conn,
                &format!("BL7-{index}"),
                "BL7 FILTER",
                Some(&jan),
                2,
                (false, target, dirty),
            );
        }
        for (code, department_id, is_discontinued) in [
            ("BL7-DEPT-DECOY", 3_i64, false),
            ("BL7-DISC-DECOY", 2_i64, true),
        ] {
            let jan = synthetic_ean13_req907(260_000_000_100 + department_id as u64);
            insert_bulk_product_req907(
                &conn,
                code,
                "BL7 FILTER",
                Some(&jan),
                department_id,
                (is_discontinued, true, true),
            );
        }
        let query = ProductSearchQuery {
            keyword: Some("BL7 FILTER".into()),
            department_id: Some(2),
            supplier_id: None,
            include_unassigned: false,
            is_discontinued: Some(false),
            plu: Some(PluMigrationFilter::Pending),
            sort_key: SortKey::Name,
            sort_order: SortOrder::Asc,
            page: 1,
            per_page: 200,
        };
        let search_count = search_products(&conn, query).unwrap().total_count;
        let pending = ProductBulkFilter {
            is_discontinued: Some(false),
            plu: Some(PluMigrationFilter::Pending),
            ..bulk_filter_req907("BL7 FILTER")
        };
        let pending_result = bulk_set_plu_target(&mut conn, pending, false).unwrap();
        let all_result = bulk_set_plu_target(
            &mut conn,
            ProductBulkFilter {
                is_discontinued: Some(false),
                ..bulk_filter_req907("BL7 FILTER")
            },
            false,
        )
        .unwrap();
        assert!(search_count > 0);
        assert_eq!(pending_result.matched_count, search_count);
        assert_eq!(all_result.matched_count, 4);
        assert_ne!(all_result.matched_count, pending_result.matched_count);
    }

    fn create_price_revision_product(conn: &mut DbConnection) -> String {
        create_product(conn, default_create_request())
            .unwrap()
            .product_code
    }

    fn revision_input(product_code: &str) -> PriceRevisionInput {
        PriceRevisionInput {
            product_code: product_code.to_string(),
            new_selling_price: 700,
            new_cost_price_centi: 400,
            assign_supplier_id: None,
        }
    }

    fn price_revision_log_count(conn: &DbConnection) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'product_price_revise'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn test_revise_product_price_req105_not_found_product() {
        let (_dir, mut conn) = setup_test_db();
        let error = revise_product_price(&mut conn, revision_input("MISSING")).unwrap_err();
        assert!(matches!(error, BizError::NotFound(_)));
    }

    #[test]
    fn test_revise_product_price_req105_validation_negative_selling_price() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        let mut input = revision_input(&code);
        input.new_selling_price = -1;
        assert!(matches!(
            revise_product_price(&mut conn, input),
            Err(BizError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_revise_product_price_req105_validation_negative_cost_price() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        let mut input = revision_input(&code);
        input.new_cost_price_centi = -1;
        assert!(matches!(
            revise_product_price(&mut conn, input),
            Err(BizError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_revise_product_price_req105_no_op_when_unchanged() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        conn.execute(
            "UPDATE products SET updated_at = '2000-01-01T00:00:00' WHERE product_code = ?1",
            [&code],
        )
        .unwrap();
        let result = revise_product_price(
            &mut conn,
            PriceRevisionInput {
                product_code: code.clone(),
                new_selling_price: 500,
                new_cost_price_centi: 300,
                assign_supplier_id: None,
            },
        )
        .unwrap();
        assert!(!result.changed);
        let updated_at: String = conn
            .query_row(
                "SELECT updated_at FROM products WHERE product_code = ?1",
                [&code],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(updated_at, "2000-01-01T00:00:00");
        assert!(product_repo::list_price_history(&conn, &code, 10)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_revise_product_price_req105_no_op_writes_no_operation_log() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        let before = price_revision_log_count(&conn);
        revise_product_price(
            &mut conn,
            PriceRevisionInput {
                product_code: code,
                new_selling_price: 500,
                new_cost_price_centi: 300,
                assign_supplier_id: None,
            },
        )
        .unwrap();
        assert_eq!(price_revision_log_count(&conn), before);
    }

    #[test]
    fn test_revise_product_price_req105_writes_price_history_four_values() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        revise_product_price(&mut conn, revision_input(&code)).unwrap();
        let history = product_repo::list_price_history(&conn, &code, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].old_selling_price, 500);
        assert_eq!(history[0].new_selling_price, 700);
        assert_eq!(history[0].old_cost_price_centi, 300);
        assert_eq!(history[0].new_cost_price_centi, 400);
    }

    #[test]
    fn test_revise_product_price_req105_writes_operation_log() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        revise_product_price(&mut conn, revision_input(&code)).unwrap();
        assert_eq!(price_revision_log_count(&conn), 1);
    }

    #[test]
    fn test_revise_product_price_req105_plu_dirty_on_selling_change() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        conn.execute(
            "UPDATE products SET plu_dirty = 0 WHERE product_code = ?1",
            [&code],
        )
        .unwrap();
        let result = revise_product_price(&mut conn, revision_input(&code)).unwrap();
        let product = product_repo::find_by_product_code(&conn, &code)
            .unwrap()
            .unwrap();
        assert!(result.plu_dirty_set);
        assert!(product.product.plu_dirty);
    }

    #[test]
    fn test_revise_product_price_req105_cost_only_no_plu_dirty() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        conn.execute(
            "UPDATE products SET plu_dirty = 0 WHERE product_code = ?1",
            [&code],
        )
        .unwrap();
        let result = revise_product_price(
            &mut conn,
            PriceRevisionInput {
                product_code: code.clone(),
                new_selling_price: 500,
                new_cost_price_centi: 400,
                assign_supplier_id: None,
            },
        )
        .unwrap();
        let product = product_repo::find_by_product_code(&conn, &code)
            .unwrap()
            .unwrap();
        assert!(!result.plu_dirty_set);
        assert!(!product.product.plu_dirty);
    }

    #[test]
    fn test_revise_product_price_req106_assigns_null_supplier_id() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        let supplier = create_supplier(&conn, "合成取引先A".to_string()).unwrap();
        let mut input = revision_input(&code);
        input.assign_supplier_id = Some(supplier.id);
        let result = revise_product_price(&mut conn, input).unwrap();
        let product = product_repo::find_by_product_code(&conn, &code)
            .unwrap()
            .unwrap();
        assert!(result.supplier_assigned);
        assert_eq!(product.product.supplier_id, Some(supplier.id));
    }

    #[test]
    fn test_revise_product_price_req106_keeps_existing_supplier_id() {
        let (_dir, mut conn) = setup_test_db();
        let first = create_supplier(&conn, "合成取引先A".to_string()).unwrap();
        let second = create_supplier(&conn, "合成取引先B".to_string()).unwrap();
        let mut request = default_create_request();
        request.supplier_id = Some(first.id);
        let code = create_product(&mut conn, request).unwrap().product_code;
        let mut input = revision_input(&code);
        input.assign_supplier_id = Some(second.id);
        let result = revise_product_price(&mut conn, input).unwrap();
        let product = product_repo::find_by_product_code(&conn, &code)
            .unwrap()
            .unwrap();
        assert!(!result.supplier_assigned);
        assert_eq!(product.product.supplier_id, Some(first.id));
    }

    #[test]
    fn test_revise_product_price_req106_unknown_supplier_id_not_found() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        let mut input = revision_input(&code);
        input.assign_supplier_id = Some(999_999);
        let error = revise_product_price(&mut conn, input).unwrap_err();
        assert!(matches!(error, BizError::NotFound(_)));
        let product = product_repo::find_by_product_code(&conn, &code)
            .unwrap()
            .unwrap();
        assert_eq!(product.product.selling_price, 500);
    }

    #[test]
    fn test_revise_product_price_req106_assigns_supplier_when_price_unchanged() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        let supplier = create_supplier(&conn, "合成取引先".to_string()).unwrap();
        let before_logs = price_revision_log_count(&conn);
        let result = revise_product_price(
            &mut conn,
            PriceRevisionInput {
                product_code: code.clone(),
                new_selling_price: 500,
                new_cost_price_centi: 300,
                assign_supplier_id: Some(supplier.id),
            },
        )
        .unwrap();
        assert!(!result.changed);
        assert!(result.supplier_assigned);
        assert!(product_repo::list_price_history(&conn, &code, 10)
            .unwrap()
            .is_empty());
        assert_eq!(price_revision_log_count(&conn), before_logs);
    }

    #[test]
    fn test_revise_product_price_req105_persists_large_values_exactly() {
        // REQ-105 / 共通規則 (a)（X5・X6）: 上限の端は商品と履歴に正確に保存され、1 つ外は何も変えない
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        revise_product_price(
            &mut conn,
            PriceRevisionInput {
                product_code: code.clone(),
                new_selling_price: 999_999,
                new_cost_price_centi: 99_999_999,
                assign_supplier_id: None,
            },
        )
        .unwrap();
        let product = product_repo::find_by_product_code(&conn, &code)
            .unwrap()
            .unwrap();
        let history = product_repo::list_price_history(&conn, &code, 10).unwrap();
        assert_eq!(product.product.selling_price, 999_999);
        assert_eq!(product.product.cost_price_centi, 99_999_999);
        assert_eq!(history[0].new_selling_price, 999_999);
        assert_eq!(history[0].new_cost_price_centi, 99_999_999);

        let before_logs = price_revision_log_count(&conn);
        for (new_selling_price, new_cost_price_centi, message) in [
            (1_000_000, 300, "売価が大きすぎます"),
            (500, 100_000_000, "原価が大きすぎます"),
        ] {
            let result = revise_product_price(
                &mut conn,
                PriceRevisionInput {
                    product_code: code.clone(),
                    new_selling_price,
                    new_cost_price_centi,
                    assign_supplier_id: None,
                },
            );
            assert!(
                matches!(result, Err(BizError::ValidationFailed(ref m)) if m == message),
                "{message}: {result:?}"
            );
            let product = product_repo::find_by_product_code(&conn, &code)
                .unwrap()
                .unwrap();
            assert_eq!(product.product.selling_price, 999_999);
            assert_eq!(product.product.cost_price_centi, 99_999_999);
            assert_eq!(
                product_repo::list_price_history(&conn, &code, 10)
                    .unwrap()
                    .len(),
                history.len()
            );
            assert_eq!(price_revision_log_count(&conn), before_logs);
        }
    }

    #[test]
    #[serial]
    fn test_create_update_product_req101_req102_price_and_initial_stock_limits() {
        // REQ-101 / REQ-102 / 共通規則 (a)（X3・X5・X6）/ Matrix C1: 売価は 999999、原価は 99999999（1/100 円）、初期在庫は 999999999 まで
        let (_dir, mut conn) = setup_test_db();
        let edge = create_product(
            &mut conn,
            ProductCreateRequest {
                selling_price: 999_999,
                cost_price_centi: 99_999_999,
                initial_stock: 999_999_999,
                ..default_create_request()
            },
        )
        .unwrap();
        let stored = product_repo::find_by_product_code(&conn, &edge.product_code)
            .unwrap()
            .unwrap()
            .product;
        assert_eq!(
            (
                stored.selling_price,
                stored.cost_price_centi,
                stored.stock_quantity
            ),
            (999_999, 99_999_999, 999_999_999)
        );

        let product_count = |conn: &DbConnection| -> i64 {
            conn.query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
                .unwrap()
        };
        for (selling_price, cost_price_centi, initial_stock, message) in [
            (1_000_000, 300, 0, "売価が大きすぎます"),
            (500, 100_000_000, 0, "原価が大きすぎます"),
            (500, 300, 1_000_000_000, "在庫数が大きすぎます"),
        ] {
            let before = product_count(&conn);
            let result = create_product(
                &mut conn,
                ProductCreateRequest {
                    selling_price,
                    cost_price_centi,
                    initial_stock,
                    ..default_create_request()
                },
            );
            assert!(
                matches!(result, Err(BizError::ValidationFailed(ref m)) if m == message),
                "{message}: {result:?}"
            );
            assert_eq!(product_count(&conn), before);
        }

        for (selling_price, cost_price_centi, message) in [
            (Some(1_000_000), None, "売価が大きすぎます"),
            (None, Some(100_000_000), "原価が大きすぎます"),
        ] {
            let result = update_product(
                &mut conn,
                &edge.product_code,
                &ProductUpdateRequest {
                    selling_price,
                    cost_price_centi,
                    ..Default::default()
                },
            );
            assert!(
                matches!(result, Err(BizError::ValidationFailed(ref m)) if m == message),
                "{message}: {result:?}"
            );
        }
        update_product(
            &mut conn,
            &edge.product_code,
            &ProductUpdateRequest {
                selling_price: Some(999_999),
                cost_price_centi: Some(99_999_999),
                ..Default::default()
            },
        )
        .unwrap();
    }

    #[test]
    #[serial]
    fn test_revise_product_price_req105_tx_atomicity_rollback() {
        let (_dir, mut conn) = setup_test_db();
        let code = create_price_revision_product(&mut conn);
        let before_logs = price_revision_log_count(&conn);
        let _guard = failpoint::arm(&failpoint::UPDATE_PRODUCT_AFTER_PRICE_HISTORY);
        assert!(revise_product_price(&mut conn, revision_input(&code)).is_err());
        let product = product_repo::find_by_product_code(&conn, &code)
            .unwrap()
            .unwrap();
        assert_eq!(product.product.selling_price, 500);
        assert!(product_repo::list_price_history(&conn, &code, 10)
            .unwrap()
            .is_empty());
        assert_eq!(price_revision_log_count(&conn), before_logs);
    }

    #[test]
    fn test_create_supplier_req106_trims_and_creates() {
        let (_dir, conn) = setup_test_db();
        let supplier = create_supplier(&conn, "  合成取引先  ".to_string()).unwrap();
        assert_eq!(supplier.name, "合成取引先");
        assert_eq!(product_repo::list_suppliers(&conn).unwrap().len(), 1);
    }

    #[test]
    fn test_create_supplier_req106_rejects_empty_name() {
        let (_dir, conn) = setup_test_db();
        assert!(matches!(
            create_supplier(&conn, " \t ".to_string()),
            Err(BizError::ValidationFailed(_))
        ));
        assert!(product_repo::list_suppliers(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_create_supplier_req106_returns_existing_row_for_duplicate_name() {
        let (_dir, conn) = setup_test_db();
        let first = create_supplier(&conn, "合成取引先".to_string()).unwrap();
        let second = create_supplier(&conn, " 合成取引先 ".to_string()).unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(product_repo::list_suppliers(&conn).unwrap().len(), 1);
    }

    #[test]
    fn test_rename_supplier_trims_and_rejects_empty() {
        let (_dir, mut conn) = setup_test_db();
        let supplier_id = seed_named_supplier(&conn, "旧取引先");
        let renamed = rename_supplier(&mut conn, supplier_id, "  新  ".to_string()).unwrap();
        assert_eq!(renamed.name, "新");
        assert!(matches!(
            rename_supplier(&mut conn, supplier_id, "   ".to_string()),
            Err(BizError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_rename_supplier_conflict_returns_validation() {
        let (_dir, mut conn) = setup_test_db();
        let source_id = seed_named_supplier(&conn, "変更元");
        seed_named_supplier(&conn, "既存取引先");
        assert!(matches!(
            rename_supplier(&mut conn, source_id, "既存取引先".to_string()),
            Err(BizError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_rename_supplier_same_name_noop() {
        let (_dir, mut conn) = setup_test_db();
        let supplier_id = seed_named_supplier(&conn, "同値取引先");
        conn.execute(
            "UPDATE suppliers SET updated_at='2026-08-01T00:00:00' WHERE id=?1",
            [supplier_id],
        )
        .unwrap();
        let before_logs: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        let result = rename_supplier(&mut conn, supplier_id, " 同値取引先 ".to_string()).unwrap();
        let updated_at: Option<String> = conn
            .query_row(
                "SELECT updated_at FROM suppliers WHERE id=?1",
                [supplier_id],
                |row| row.get(0),
            )
            .unwrap();
        let after_logs: i64 = conn
            .query_row("SELECT COUNT(*) FROM operation_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(result.name, "同値取引先");
        assert_eq!(updated_at.as_deref(), Some("2026-08-01T00:00:00"));
        assert_eq!(after_logs, before_logs);
    }

    #[test]
    fn test_rename_supplier_updates_updated_at_and_logs() {
        let (_dir, mut conn) = setup_test_db();
        let supplier_id = seed_named_supplier(&conn, "改名前");
        conn.execute(
            "UPDATE suppliers SET updated_at='2000-01-01T00:00:00' WHERE id=?1",
            [supplier_id],
        )
        .unwrap();
        let renamed = rename_supplier(&mut conn, supplier_id, "改名後".to_string()).unwrap();
        let (updated_at, operation_type, detail): (Option<String>, String, String) = conn
            .query_row(
                "SELECT s.updated_at,l.operation_type,l.detail_json
                 FROM suppliers s CROSS JOIN operation_logs l
                 WHERE s.id=?1 ORDER BY l.id DESC LIMIT 1",
                [supplier_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(renamed.name, "改名後");
        assert_ne!(updated_at.as_deref(), Some("2000-01-01T00:00:00"));
        assert_eq!(operation_type, "supplier_rename");
        assert!(detail.contains(&supplier_id.to_string()));
        assert!(detail.contains("改名前"));
        assert!(detail.contains("改名後"));
        assert!(matches!(
            rename_supplier(&mut conn, 999_999, "不存在".to_string()),
            Err(BizError::NotFound(_))
        ));

        let (_rollback_dir, mut rollback_conn) = setup_test_db();
        let rollback_supplier_id = seed_named_supplier(&rollback_conn, "ログ失敗改名前");
        rollback_conn
            .execute(
                "UPDATE suppliers SET updated_at='2001-01-01T00:00:00' WHERE id=?1",
                [rollback_supplier_id],
            )
            .unwrap();
        rollback_conn
            .execute_batch(
                "CREATE TRIGGER fail_supplier_rename_log
                 BEFORE INSERT ON operation_logs
                 BEGIN SELECT RAISE(ABORT, 'synthetic operation log failure'); END;",
            )
            .unwrap();
        assert!(rename_supplier(
            &mut rollback_conn,
            rollback_supplier_id,
            "ログ失敗改名後".to_string()
        )
        .is_err());
        let (name, updated_at): (String, Option<String>) = rollback_conn
            .query_row(
                "SELECT name,updated_at FROM suppliers WHERE id=?1",
                [rollback_supplier_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let log_count: i64 = rollback_conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type='supplier_rename'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "ログ失敗改名前");
        assert_eq!(updated_at.as_deref(), Some("2001-01-01T00:00:00"));
        assert_eq!(log_count, 0);
    }

    #[test]
    fn test_merge_suppliers_repoints_products_and_receiving_records_then_deletes() {
        let (_dir, mut conn) = setup_test_db();
        let source_id = seed_named_supplier(&conn, "統合元");
        let target_id = seed_named_supplier(&conn, "統合先");
        seed_supplier_product(&conn, "MERGE-S1", source_id);
        seed_supplier_product(&conn, "MERGE-S2", source_id);
        seed_supplier_product(&conn, "MERGE-T1", target_id);
        seed_supplier_receiving(&conn, "merge-receiving", source_id);

        let result = merge_suppliers(&mut conn, source_id, target_id).unwrap();
        let source_products: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE supplier_id=?1",
                [source_id],
                |row| row.get(0),
            )
            .unwrap();
        let target_products: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE supplier_id=?1",
                [target_id],
                |row| row.get(0),
            )
            .unwrap();
        let source_receivings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM receiving_records WHERE supplier_id=?1",
                [source_id],
                |row| row.get(0),
            )
            .unwrap();
        let target_receivings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM receiving_records WHERE supplier_id=?1",
                [target_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            (result.products_updated, result.receiving_records_updated),
            (2, 1)
        );
        assert_eq!((source_products, target_products), (0, 3));
        assert_eq!((source_receivings, target_receivings), (0, 1));
        assert!(product_repo::find_supplier_by_id(&conn, source_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_merge_suppliers_rejects_same_id() {
        let (_dir, mut conn) = setup_test_db();
        let supplier_id = seed_named_supplier(&conn, "自己統合不可");
        assert!(matches!(
            merge_suppliers(&mut conn, supplier_id, supplier_id),
            Err(BizError::ValidationFailed(_))
        ));
        assert!(product_repo::find_supplier_by_id(&conn, supplier_id)
            .unwrap()
            .is_some());
    }

    #[test]
    fn test_merge_suppliers_not_found() {
        let (_dir, mut conn) = setup_test_db();
        let source_id = seed_named_supplier(&conn, "再実行元");
        let target_id = seed_named_supplier(&conn, "再実行先");
        assert!(matches!(
            merge_suppliers(&mut conn, 999_999, target_id),
            Err(BizError::NotFound(_))
        ));
        assert!(matches!(
            merge_suppliers(&mut conn, source_id, 999_999),
            Err(BizError::NotFound(_))
        ));
        merge_suppliers(&mut conn, source_id, target_id).unwrap();
        assert!(matches!(
            merge_suppliers(&mut conn, source_id, target_id),
            Err(BizError::NotFound(_))
        ));
    }

    #[test]
    fn test_merge_suppliers_single_tx_rollback_on_failure() {
        let (_dir, mut conn) = setup_test_db();
        let source_id = seed_named_supplier(&conn, "失敗統合元");
        let target_id = seed_named_supplier(&conn, "失敗統合先");
        seed_supplier_product(&conn, "ROLLBACK-S1", source_id);
        seed_supplier_receiving(&conn, "rollback-receiving", source_id);
        conn.execute_batch(
            "CREATE TRIGGER fail_supplier_merge_receiving
             BEFORE UPDATE OF supplier_id ON receiving_records
             BEGIN SELECT RAISE(ABORT, 'synthetic merge failure'); END;",
        )
        .unwrap();

        assert!(merge_suppliers(&mut conn, source_id, target_id).is_err());
        let source_products: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE supplier_id=?1",
                [source_id],
                |row| row.get(0),
            )
            .unwrap();
        let source_receivings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM receiving_records WHERE supplier_id=?1",
                [source_id],
                |row| row.get(0),
            )
            .unwrap();
        let logs: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type='supplier_merge'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!((source_products, source_receivings, logs), (1, 1, 0));
        assert!(product_repo::find_supplier_by_id(&conn, source_id)
            .unwrap()
            .is_some());

        let (_log_dir, mut log_conn) = setup_test_db();
        let log_source_id = seed_named_supplier(&log_conn, "ログ失敗統合元");
        let log_target_id = seed_named_supplier(&log_conn, "ログ失敗統合先");
        seed_supplier_product(&log_conn, "LOG-ROLLBACK-S1", log_source_id);
        seed_supplier_receiving(&log_conn, "log-rollback-receiving", log_source_id);
        log_conn
            .execute_batch(
                "CREATE TRIGGER fail_supplier_merge_log
                 BEFORE INSERT ON operation_logs
                 BEGIN SELECT RAISE(ABORT, 'synthetic operation log failure'); END;",
            )
            .unwrap();
        assert!(merge_suppliers(&mut log_conn, log_source_id, log_target_id).is_err());
        let source_products: i64 = log_conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE supplier_id=?1",
                [log_source_id],
                |row| row.get(0),
            )
            .unwrap();
        let source_receivings: i64 = log_conn
            .query_row(
                "SELECT COUNT(*) FROM receiving_records WHERE supplier_id=?1",
                [log_source_id],
                |row| row.get(0),
            )
            .unwrap();
        let log_count: i64 = log_conn
            .query_row(
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type='supplier_merge'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!((source_products, source_receivings, log_count), (1, 1, 0));
        assert!(product_repo::find_supplier_by_id(&log_conn, log_source_id)
            .unwrap()
            .is_some());
    }

    #[test]
    fn test_merge_suppliers_writes_operation_log() {
        let (_dir, mut conn) = setup_test_db();
        let source_id = seed_named_supplier(&conn, "監査統合元");
        let target_id = seed_named_supplier(&conn, "監査統合先");
        seed_supplier_product(&conn, "AUDIT-S1", source_id);
        seed_supplier_product(&conn, "AUDIT-S2", source_id);
        seed_supplier_receiving(&conn, "audit-receiving", source_id);
        merge_suppliers(&mut conn, source_id, target_id).unwrap();

        let (operation_type, detail): (String, String) = conn
            .query_row(
                "SELECT operation_type,detail_json FROM operation_logs ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(operation_type, "supplier_merge");
        let detail: serde_json::Value = serde_json::from_str(&detail).unwrap();
        assert_eq!(detail["source_id"], source_id);
        assert_eq!(detail["source_name"], "監査統合元");
        assert_eq!(detail["target_id"], target_id);
        assert_eq!(detail["target_name"], "監査統合先");
        assert_eq!(detail["products_updated"], 2);
        assert_eq!(detail["receiving_records_updated"], 1);
    }

    // ===== 共有 JAN と在庫連動を始める検査（30「計数の置き場と在庫の求め直しの契約」、ADR D4・D8。b1 Matrix J） =====

    use crate::biz::stock_ledger::settlement_support::{settlement, z004_sale};
    use crate::biz::stock_ledger::test_support::insert_count;
    use crate::biz::stocktake_service::{CountRecoveryAction, StocktakeRecoveryCode};

    fn seed_jan_product(conn: &DbConnection, code: &str, jan: Option<&str>, linked: bool) {
        product_repo::insert_product(
            conn,
            &NewProduct {
                product_code: code.to_string(),
                jan_code: jan.map(str::to_string),
                name: format!("商品{code}"),
                department_id: 1,
                supplier_id: None,
                selling_price: 500,
                cost_price_centi: 300,
                tax_rate: "10".to_string(),
                maker_code: None,
                stock_quantity: 0,
                stock_unit: "pcs".to_string(),
                is_discontinued: false,
                plu_dirty: true,
                plu_exported_at: None,
                plu_target: false,
                pos_stock_sync: linked,
            },
        )
        .unwrap();
    }

    fn jan(n: u64) -> String {
        synthetic_ean13_req907(4_900_000_000_000 + n)
    }

    fn rejected_at(result: Result<impl std::fmt::Debug, BizError>) -> (String, String) {
        match result {
            Err(BizError::ValidationFailedAt { message, field }) => (message, field),
            other => panic!("ValidationFailedAt を期待しました: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn test_create_product_req101_rejects_shared_jan_with_stock_linked() {
        // REQ-101 / ADR D4 / 30（b1 Matrix J1、BIZ-01-D11 で置き換え）: JAN を他の商品が使っていれば
        // step 3a の JanInUse が先に止める（旧い期待の field jan_code の経路は無くなった）。
        // D4 の拒否は step 3a2 の経路（shared_jan_custom_code）で、変更の後の状態で検査する
        let (_dir, mut conn) = setup_test_db();
        let (linked_jan, unlinked_jan) = (jan(1), jan(2));
        seed_jan_product(&conn, "B-LINKED", Some(&linked_jan), true);
        seed_jan_product(&conn, "C-UNLINKED", Some(&unlinked_jan), false);
        let before = product_count(&conn);
        for (jan_code, linked) in [
            (&linked_jan, false),
            (&unlinked_jan, true),
            (&unlinked_jan, false),
        ] {
            let result = create_product(
                &mut conn,
                ProductCreateRequest {
                    jan_code: Some(jan_code.clone()),
                    pos_stock_sync: linked,
                    ..default_create_request()
                },
            );
            assert!(
                matches!(result, Err(BizError::JanInUse(_))),
                "{jan_code} linked={linked}: {result:?}"
            );
        }
        assert_eq!(product_count(&conn), before);
        // 連動の新しい商品を群に加える → field pos_stock_sync（手順 3b）
        let (message, field) = rejected_at(create_product(
            &mut conn,
            ProductCreateRequest {
                jan_code: Some(unlinked_jan.clone()),
                pos_stock_sync: true,
                shared_jan_custom_code: true,
                ..default_create_request()
            },
        ));
        assert_eq!(
            message,
            "同じ JAN の商品（C-UNLINKED）があるため、在庫連動にできません"
        );
        assert_eq!(field, "pos_stock_sync");
        assert_eq!(product_count(&conn), before);
        // 連動しない商品どうしの組は作れる
        create_product(
            &mut conn,
            ProductCreateRequest {
                jan_code: Some(unlinked_jan),
                pos_stock_sync: false,
                shared_jan_custom_code: true,
                ..default_create_request()
            },
        )
        .unwrap();
        assert_eq!(product_count(&conn), before + 1);
    }

    // ===== BIZ-01-D11: JAN を他の商品が使っているときの知らせと独自コードの道（D-141、Matrix B1〜B10） =====

    fn jan_in_use(result: Result<impl std::fmt::Debug, BizError>) -> String {
        match result {
            Err(BizError::JanInUse(message)) => message,
            other => panic!("JanInUse を期待しました: {other:?}"),
        }
    }

    fn count_rows(conn: &DbConnection, sql: &str) -> i64 {
        conn.query_row(sql, [], |row| row.get(0)).unwrap()
    }

    fn operation_log_count(conn: &DbConnection) -> i64 {
        count_rows(conn, "SELECT COUNT(*) FROM operation_logs")
    }

    fn is_linked(conn: &DbConnection, code: &str) -> bool {
        product_repo::find_by_product_code(conn, code)
            .unwrap()
            .unwrap()
            .product
            .pos_stock_sync
    }

    fn rename_product(conn: &DbConnection, code: &str, name: &str, discontinued: bool) {
        conn.execute(
            "UPDATE products SET name = ?2, is_discontinued = ?3 WHERE product_code = ?1",
            rusqlite::params![code, name, discontinued],
        )
        .unwrap();
    }

    fn shared_jan_request(jan_code: &str) -> ProductCreateRequest {
        ProductCreateRequest {
            jan_code: Some(jan_code.to_string()),
            name: "2 色目".to_string(),
            pos_stock_sync: false,
            shared_jan_custom_code: true,
            ..default_create_request()
        }
    }

    #[test]
    #[serial]
    fn test_create_product_req101_rejects_jan_in_use_naming_all_products() {
        // REQ-101 / BIZ-01-D11 (1)（Matrix B1）: JAN と商品コードの 2 つで集め、商品コードの昇順に、
        // 印（商品コードだけ同じ・POS販売で在庫を減らす・廃番）をこの順に付けて全てを名指す。何も書かない
        let (_dir, mut conn) = setup_test_db();
        let (j, k) = (jan(11), jan(12));
        seed_jan_product(&conn, &j, Some(&k), true);
        rename_product(&conn, &j, "青の布", false);
        seed_jan_product(&conn, "HZ-0003", Some(&j), false);
        rename_product(&conn, "HZ-0003", "黄の布", true);
        seed_jan_product(&conn, "HZ-0002", Some(&j), true);
        rename_product(&conn, "HZ-0002", "赤の布", true);
        seed_jan_product(&conn, "HZ-0001", Some(&j), false);
        rename_product(&conn, "HZ-0001", "白の布", false);
        let (products, seq, logs) = (
            product_count(&conn),
            next_seq_of(&conn, 2),
            operation_log_count(&conn),
        );

        let message = jan_in_use(create_product(
            &mut conn,
            ProductCreateRequest {
                jan_code: Some(j.clone()),
                ..default_create_request()
            },
        ));

        assert_eq!(
            message,
            format!(
                "この JAN（{j}）はほかの商品が使っています: {j}「青の布」（商品コードだけ同じ・POS販売で在庫を減らす）、HZ-0001「白の布」、HZ-0002「赤の布」（POS販売で在庫を減らす・廃番）、HZ-0003「黄の布」（廃番）"
            )
        );
        assert!(!message.contains("在庫連動"), "{message}");
        assert_eq!(product_count(&conn), products);
        assert_eq!(next_seq_of(&conn, 2), seq);
        assert_eq!(operation_log_count(&conn), logs);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_jan_in_use_by_product_code_only() {
        // REQ-101 / BIZ-01-D11（Matrix B2）: 商品コードだけが J の商品も名指し、群（同じ jan_code）でないので外さない
        let (_dir, mut conn) = setup_test_db();
        let (j, k) = (jan(13), jan(14));
        seed_jan_product(&conn, &j, Some(&k), true);
        rename_product(&conn, &j, "青の布", false);

        let message = jan_in_use(create_product(
            &mut conn,
            ProductCreateRequest {
                jan_code: Some(j.clone()),
                ..default_create_request()
            },
        ));
        assert_eq!(
            message,
            format!(
                "この JAN（{j}）はほかの商品が使っています: {j}「青の布」（商品コードだけ同じ・POS販売で在庫を減らす）"
            )
        );

        let result = create_product(&mut conn, shared_jan_request(&j)).unwrap();
        assert!(is_linked(&conn, &j), "商品コードだけ同じ商品は外さない");
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_shared_jan_custom_code_unlinks_group() {
        // REQ-101 / BIZ-01-D11 (2)（Matrix B4）: 独自コードを振り JAN を持たせ、同じ JAN の在庫連動の商品だけを
        // 同じ TX で外し、BIZ-01-D7 の形のログと warnings で名指す。既存の商品の商品コードは変えない（TD-352 からの例外）
        let (_dir, mut conn) = setup_test_db();
        let j = jan(15);
        seed_jan_product(&conn, &j, Some(&j), true);
        rename_product(&conn, &j, "赤の布", false);
        seed_jan_product(&conn, "NU-0001", Some(&j), false);
        rename_product(&conn, "NU-0001", "白の布", false);

        let message = jan_in_use(create_product(
            &mut conn,
            ProductCreateRequest {
                jan_code: Some(j.clone()),
                ..default_create_request()
            },
        ));
        assert_eq!(
            message,
            format!(
                "この JAN（{j}）はほかの商品が使っています: {j}「赤の布」（POS販売で在庫を減らす）、NU-0001「白の布」"
            )
        );
        let stock_before = count_rows(
            &conn,
            &format!("SELECT stock_quantity FROM products WHERE product_code = '{j}'"),
        );

        let result = create_product(&mut conn, shared_jan_request(&j)).unwrap();

        assert_eq!(result.product_code, "HZ-0001");
        let created = product_repo::find_by_product_code(&conn, "HZ-0001")
            .unwrap()
            .unwrap()
            .product;
        assert_eq!(created.jan_code.as_deref(), Some(j.as_str()));
        assert!(!created.pos_stock_sync);
        assert!(!is_linked(&conn, &j));
        assert!(!is_linked(&conn, "NU-0001"));
        // detail は BIZ-01-D7 の形（JSON の値で比べる。key の並びは update_product と同じ serde_json の順）
        let logs: Vec<(String, serde_json::Value)> = conn
            .prepare(
                "SELECT summary, detail_json FROM operation_logs
                 WHERE operation_type = 'product_update' ORDER BY id",
            )
            .unwrap()
            .query_map([], |row| {
                let detail: String = row.get(1)?;
                Ok((row.get(0)?, serde_json::from_str(&detail).unwrap()))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            logs,
            vec![(
                format!("商品を更新しました: {j}"),
                serde_json::json!({ "pos_stock_sync": { "old": true, "new": false } })
            )]
        );
        assert_eq!(
            result.warnings,
            vec![format!(
                "同じ JAN の商品 {j}「赤の布」の「POS販売で在庫を減らす」を「減らさない」にしました"
            )]
        );
        assert_eq!(
            count_rows(
                &conn,
                &format!("SELECT stock_quantity FROM products WHERE product_code = '{j}'"),
            ),
            stock_before
        );
        assert!(
            product_repo::find_by_product_code(&conn, &j)
                .unwrap()
                .is_some(),
            "1 色目の商品コードは J のまま"
        );
    }

    #[test]
    #[serial]
    fn test_create_product_req101_shared_jan_custom_code_rejects_linked_new_and_rolls_back() {
        // REQ-101 / 30 §4.2 手順 3b（Matrix B5）: 群に在庫連動の新しい商品を加えると field pos_stock_sync で拒み、
        // 3a2 で外した在庫連動・next_seq・商品・ログが全て戻る
        let (_dir, mut conn) = setup_test_db();
        let j = jan(16);
        seed_jan_product(&conn, &j, Some(&j), true);
        let (products, seq, logs) = (
            product_count(&conn),
            next_seq_of(&conn, 2),
            operation_log_count(&conn),
        );

        let (message, field) = rejected_at(create_product(
            &mut conn,
            ProductCreateRequest {
                pos_stock_sync: true,
                ..shared_jan_request(&j)
            },
        ));

        assert_eq!(
            message,
            format!("同じ JAN の商品（{j}）があるため、在庫連動にできません")
        );
        assert_eq!(field, "pos_stock_sync");
        assert!(is_linked(&conn, &j));
        assert_eq!(product_count(&conn), products);
        assert_eq!(next_seq_of(&conn, 2), seq);
        assert_eq!(operation_log_count(&conn), logs);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_shared_jan_custom_code_requires_code_prefix() {
        // REQ-101 / BIZ-01-D6・BIZ-01-D11（Matrix B6）: 接頭辞の無い部門は発番の前に止まり、外しは残らない
        let (_dir, mut conn) = setup_test_db();
        let j = jan(17);
        seed_jan_product(&conn, &j, Some(&j), true);

        let result = create_product(
            &mut conn,
            ProductCreateRequest {
                department_id: 3, // 毛糸（code_prefix=NULL）
                ..shared_jan_request(&j)
            },
        );

        match result {
            Err(BizError::ValidationFailed(message)) => {
                assert_eq!(message, "この部門は独自コード発番に対応していません");
            }
            other => panic!("ValidationFailed を期待しました: {other:?}"),
        }
        assert!(is_linked(&conn, &j));
    }

    #[test]
    #[serial]
    fn test_create_product_req101_shared_jan_custom_code_without_group() {
        // REQ-101 / BIZ-01-D11（Matrix B7）: J をどの商品も使っていなくても失敗にせず独自コードで登録する（外す商品 0）
        let (_dir, mut conn) = setup_test_db();
        let j = jan(18);

        let result = create_product(&mut conn, shared_jan_request(&j)).unwrap();

        assert_eq!(result.product_code, "HZ-0001");
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        let created = product_repo::find_by_product_code(&conn, "HZ-0001")
            .unwrap()
            .unwrap()
            .product;
        assert_eq!(created.jan_code.as_deref(), Some(j.as_str()));
        assert_eq!(
            count_rows(
                &conn,
                "SELECT COUNT(*) FROM operation_logs WHERE operation_type = 'product_update'"
            ),
            0
        );
    }

    #[test]
    #[serial]
    fn test_create_product_req101_shared_jan_custom_code_rolls_back_on_late_failure() {
        // REQ-101 / 30 §4.2 エラーハンドリング（Matrix B8）: INSERT の後の失敗で、3a2 の外しも全て戻る
        let (_dir, mut conn) = setup_test_db();
        let j = jan(19);
        seed_jan_product(&conn, &j, Some(&j), true);
        let (products, seq, logs) = (
            product_count(&conn),
            next_seq_of(&conn, 2),
            operation_log_count(&conn),
        );
        {
            let _guard = failpoint::arm(&failpoint::CREATE_PRODUCT_AFTER_INSERT);
            assert!(create_product(&mut conn, shared_jan_request(&j)).is_err());
        }
        assert!(is_linked(&conn, &j));
        assert_eq!(product_count(&conn), products);
        assert_eq!(next_seq_of(&conn, 2), seq);
        assert_eq!(operation_log_count(&conn), logs);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_shared_jan_custom_code_ignored_without_jan() {
        // REQ-101 / BIZ-01-D11（Matrix B9）: jan_code が None なら flag は意味を持たず、今どおり発番する
        let (_dir, mut conn) = setup_test_db();
        let result = create_product(
            &mut conn,
            ProductCreateRequest {
                shared_jan_custom_code: true,
                ..default_create_request()
            },
        )
        .unwrap();
        assert_eq!(result.product_code, "HZ-0001");
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    }

    #[test]
    #[serial]
    fn test_create_product_req101_shared_jan_custom_code_recomputes_unlinked() {
        // REQ-101 / BIZ-01-D11 (2)・ADR SPEC-STK-CNT-D2（Matrix B10）: 外した商品を同じ TX で求め直す
        // （写しをわざと記録からの期待値と食い違わせて観測する）
        let (_dir, mut conn) = setup_test_db();
        let j = jan(20);
        seed_jan_product(&conn, &j, Some(&j), true);
        inventory_repo::insert_movement(
            &conn,
            &NewMovement {
                product_code: j.clone(),
                movement_type: MovementType::Receiving,
                quantity: 5,
                stock_after: 5,
                reference_type: None,
                reference_id: None,
                note: None,
                business_date: "2026-10-01".to_string(),
                settlement_no: None,
            },
        )
        .unwrap();
        conn.execute(
            "UPDATE products SET stock_quantity = 77, stock_recheck = 1, stock_range_low = 70,
                 stock_range_high = 80 WHERE product_code = ?1",
            [&j],
        )
        .unwrap();

        create_product(&mut conn, shared_jan_request(&j)).unwrap();

        let copy: (i64, i64, Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT stock_quantity, stock_recheck, stock_range_low, stock_range_high
                 FROM products WHERE product_code = ?1",
                [&j],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(copy, (5, 0, None, None));
    }

    #[test]
    #[serial]
    fn test_update_product_req102_rejects_stock_link_on_shared_jan() {
        // REQ-102 / ADR D4 / 30（b1 Matrix J2）: update は在庫連動を始める保存だけを検査する。
        // 連動しない B と C が同じ JAN を持つとき、C を在庫連動にする保存を拒み、ほかの欄だけの保存は通る
        let (_dir, mut conn) = setup_test_db();
        let shared = jan(3);
        seed_jan_product(&conn, "B-001", Some(&shared), false);
        seed_jan_product(&conn, "C-001", Some(&shared), false);
        let (message, field) = rejected_at(update_product(
            &mut conn,
            "C-001",
            &ProductUpdateRequest {
                pos_stock_sync: Some(true),
                ..Default::default()
            },
        ));
        assert_eq!(
            message,
            "同じ JAN の商品（B-001）があるため、在庫連動にできません"
        );
        assert_eq!(field, "pos_stock_sync");
        let product = product_repo::find_by_product_code(&conn, "C-001")
            .unwrap()
            .unwrap();
        assert!(!product.product.pos_stock_sync);
        update_product(
            &mut conn,
            "C-001",
            &ProductUpdateRequest {
                name: Some("名前だけ変える".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
    }

    fn import_error_messages(preview: &ImportPreview) -> Vec<(usize, String)> {
        let mut rows: Vec<(usize, String)> = preview
            .error_rows
            .iter()
            .map(|row| (row.line_no, row.errors.join(" / ")))
            .collect();
        rows.sort();
        rows
    }

    #[test]
    #[serial]
    fn test_import_req104_shared_jan_row_error_both_directions_and_commit_recheck() {
        // REQ-104 / ADR D4 / 30 の判定の状態と確定の拒否（b1 Matrix J3。round 1・round 2 の是正）
        let (_dir, mut conn) = setup_test_db();
        let (ja, jc, jd, jn) = (jan(11), jan(12), jan(13), jan(14));
        seed_jan_product(&conn, "A-LINK", Some(&ja), true);
        seed_jan_product(&conn, "C-OFF", Some(&jc), false);
        seed_jan_product(&conn, "D-OFF", Some(&jd), false);
        let header = "商品コード,商品名,部門ID,売価,原価,税率,JANコード,POS在庫連動";
        let csv = make_csv(
            header,
            &[
                &format!("A-LINK,連動の上書き,1,500,300,10,{jc},"),
                &format!("C-OFF,非連動の上書き,1,500,300,10,{ja},"),
                &format!("N-1,新規の連動,1,500,300,10,{jd},1"),
                &format!("N-2,新規の先,1,500,300,10,{jn},1"),
                &format!("N-3,新規の後,1,500,300,10,{jn},1"),
                "N-4,正常,1,500,300,10,,",
            ],
        );
        let preview = preview_import(&conn, &csv).unwrap();
        let message =
            |codes: &str| format!("同じ JAN の商品（{codes}）があるため、在庫連動にできません");
        assert_eq!(
            import_error_messages(&preview),
            vec![
                (2, message("C-OFF")),
                (3, message("A-LINK")),
                (4, message("D-OFF")),
                (6, message("N-2")),
            ]
        );
        let valid: Vec<&str> = preview
            .valid_rows
            .iter()
            .map(|row| row.product_code.as_str())
            .collect();
        assert_eq!(valid, vec!["N-2", "N-4"]);
        assert!(preview.duplicate_rows.is_empty());

        // 確定 (a): preview の後に別の経路で同じ JAN の連動の商品が登録された → 何も書かずに全体を戻す
        let jx = jan(15);
        let csv = make_csv(
            header,
            &[
                "N-5,正常,1,500,300,10,,",
                &format!("X-1,連動,1,500,300,10,{jx},1"),
            ],
        );
        let preview = preview_import(&conn, &csv).unwrap();
        assert_eq!(preview.valid_rows.len(), 2);
        seed_jan_product(&conn, "Y-LINK", Some(&jx), true);
        let before = product_count(&conn);
        let error = commit_import(&mut conn, preview.valid_rows, vec![]).unwrap_err();
        let BizError::ValidationFailed(text) = error else {
            panic!("ValidationFailed を期待しました: {error:?}")
        };
        assert_eq!(
            text,
            format!(
                "商品 X-1 の行: {}。読み込みの後に登録か売上が変わったか、上書きを選んだほかの行と重なったため、取込みを止めました（どの行も取り込んでいません）。その行をスキップにするか、ファイルを選び直してもう一度読み込んでください",
                message("Y-LINK")
            )
        );
        assert_eq!(product_count(&conn), before);

        // 確定 (b): 上書きを選んだ 2 行が同じ新しい JAN になり片方が在庫連動 → 後の行を名指して全体を戻す
        let (jp1, jp2, jz) = (jan(16), jan(17), jan(18));
        seed_jan_product(&conn, "P-1", Some(&jp1), true);
        seed_jan_product(&conn, "P-2", Some(&jp2), false);
        let mut p1 = make_import_row("P-1", "上書き1");
        p1.jan_code = Some(jz.clone());
        let mut p2 = make_import_row("P-2", "上書き2");
        p2.jan_code = Some(jz.clone());
        let error = commit_import(
            &mut conn,
            vec![p1, p2],
            vec!["P-1".to_string(), "P-2".to_string()],
        )
        .unwrap_err();
        assert!(
            matches!(&error, BizError::ValidationFailed(text) if text.starts_with(&format!("商品 P-2 の行: {}。", message("P-1")))),
            "{error:?}"
        );
        let p1 = product_repo::find_by_product_code(&conn, "P-1")
            .unwrap()
            .unwrap();
        assert_eq!(p1.product.jan_code.as_deref(), Some(jp1.as_str()));
    }

    const RECOUNT_REQUIRED: &str =
        "連動しない間の売上があります。今の数を入れてから在庫連動にしてください";

    fn start_link(conn: &mut DbConnection, code: &str) -> Result<ProductUpdateResult, BizError> {
        update_product(
            conn,
            code,
            &ProductUpdateRequest {
                pos_stock_sync: Some(true),
                ..Default::default()
            },
        )
    }

    #[test]
    #[serial]
    fn test_update_product_req102_start_stock_link_judges_settlements_by_import_cursor() {
        // REQ-102 / ADR D3・D8（在庫連動を始める検査の判定の n も取込みのカーソルで求める。
        // Codex の closure の Gated Amendment）/ b1 Matrix J7: 精算 1 の後の入れ直し → 取消 → 同じ番号の
        // 新しい精算 1 の連動なしの売上は、計数に「入る」と確かめられないので在庫連動を始められない
        use crate::biz::stock_ledger::settlement_support::{insert_fresh_count, roll_back};
        let (_dir, mut conn) = setup_test_db();
        seed_jan_product(&conn, "P-REUSED", None, false);
        let i1 = settlement(&conn, 1, "2026-05-09");
        insert_fresh_count(&conn, "P-REUSED", 5, "2026-05-10");
        roll_back(&conn, i1);
        let i2 = settlement(&conn, 1, "2026-05-11");
        z004_sale(&conn, "P-REUSED", i2, 1, 100, false);
        match start_link(&mut conn, "P-REUSED") {
            Err(BizError::StocktakeGuard { message, recovery }) => {
                assert_eq!(message, RECOUNT_REQUIRED);
                assert_eq!(recovery.code, StocktakeRecoveryCode::RecountRequired);
            }
            other => panic!("recount_required を期待しました: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn test_update_product_req102_start_stock_link_requires_recount() {
        // REQ-102 / ADR D8 / 30（b1 Matrix J4）: 連動なしの Z004 の売上が全て効く計数に「入る」ときだけ在庫連動を
        // 始められる。「入らない」「決まらない」・衝突・計数が無く売上がある → recount_required と回復先。
        // 共有 JAN にも当たる商品は共有 JAN の拒否を先に返す
        let (_dir, mut conn) = setup_test_db();
        let shared = jan(21);
        for code in [
            "P-NOCOUNT",
            "P-SAMEDAY",
            "P-CONFLICT",
            "P-COUNTED",
            "P-BOTH",
        ] {
            seed_jan_product(
                &conn,
                code,
                (code == "P-BOTH").then_some(shared.as_str()),
                false,
            );
        }
        seed_jan_product(&conn, "Q-SHARED", Some(&shared), false);
        seed_jan_product(&conn, "P-ZERO", None, false);
        let import_id = settlement(&conn, 1, "2026-04-02");
        for code in [
            "P-NOCOUNT",
            "P-SAMEDAY",
            "P-CONFLICT",
            "P-COUNTED",
            "P-BOTH",
        ] {
            z004_sale(&conn, code, import_id, 1, 100, false);
        }
        // 数量・金額とも 0 の行は売上として数えない
        z004_sale(&conn, "P-ZERO", import_id, 0, 0, false);
        insert_count(&conn, "P-SAMEDAY", 5, "2026-04-02", None);
        insert_count(&conn, "P-CONFLICT", 1, "2026-04-03", None);
        insert_count(&conn, "P-CONFLICT", 2, "2026-04-03", None);
        insert_count(&conn, "P-COUNTED", 5, "2026-04-03", None);

        for code in ["P-NOCOUNT", "P-SAMEDAY", "P-CONFLICT"] {
            match start_link(&mut conn, code) {
                Err(BizError::StocktakeGuard { message, recovery }) => {
                    assert_eq!(message, RECOUNT_REQUIRED, "{code}");
                    assert_eq!(recovery.code, StocktakeRecoveryCode::RecountRequired);
                    assert_eq!(recovery.targets.len(), 1);
                    assert_eq!(recovery.targets[0].product_code, code);
                    assert_eq!(recovery.targets[0].action, CountRecoveryAction::CountNow);
                    assert_eq!(recovery.targets[0].stocktake_item_id, None);
                    assert!(recovery.settlement_pending.is_none());
                }
                other => panic!("{code}: recount_required を期待しました: {other:?}"),
            }
            let product = product_repo::find_by_product_code(&conn, code)
                .unwrap()
                .unwrap();
            assert!(!product.product.pos_stock_sync, "{code}");
        }
        let (message, field) = rejected_at(start_link(&mut conn, "P-BOTH"));
        assert_eq!(
            message,
            "同じ JAN の商品（Q-SHARED）があるため、在庫連動にできません"
        );
        assert_eq!(field, "pos_stock_sync");
        start_link(&mut conn, "P-COUNTED").unwrap();
        start_link(&mut conn, "P-ZERO").unwrap();

        // 進行中の棚卸しに明細があれば回復先は active_count と明細
        let stocktake = crate::biz::stocktake_service::start_stocktake(&mut conn).unwrap();
        match start_link(&mut conn, "P-NOCOUNT") {
            Err(BizError::StocktakeGuard { recovery, .. }) => {
                let target = &recovery.targets[0];
                assert_eq!(target.action, CountRecoveryAction::ActiveCount);
                assert_eq!(target.stocktake_id, Some(stocktake.stocktake_id));
                assert!(target.stocktake_item_id.is_some());
            }
            other => panic!("recount_required を期待しました: {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn test_import_req104_start_stock_link_row_error() {
        // REQ-104 / ADR D8 / 30（b1 Matrix J5）: 一括 import の上書きで連動を始める行も同じ検査で行の error。
        // 共有 JAN にも当たる行は共有 JAN の文を先に返す。確定の TX で当たれば確定全体を戻す
        let (_dir, mut conn) = setup_test_db();
        let shared = jan(31);
        seed_jan_product(&conn, "C-SALE", None, false);
        seed_jan_product(&conn, "E-BOTH", Some(&shared), false);
        seed_jan_product(&conn, "F-SHARED", Some(&shared), false);
        seed_jan_product(&conn, "G-LATER", None, false);
        let import_id = settlement(&conn, 1, "2026-04-02");
        z004_sale(&conn, "C-SALE", import_id, 1, 100, false);
        z004_sale(&conn, "E-BOTH", import_id, 1, 100, false);
        let header = "商品コード,商品名,部門ID,売価,原価,税率,JANコード,POS在庫連動";
        let preview = preview_import(
            &conn,
            &make_csv(
                header,
                &[
                    "C-SALE,連動にする,1,500,300,10,,1",
                    &format!("E-BOTH,連動にする,1,500,300,10,{shared},1"),
                    "G-LATER,連動にする,1,500,300,10,,1",
                ],
            ),
        )
        .unwrap();
        assert_eq!(
            import_error_messages(&preview),
            vec![
                (2, RECOUNT_REQUIRED.to_string()),
                (
                    3,
                    "同じ JAN の商品（F-SHARED）があるため、在庫連動にできません".to_string()
                ),
            ]
        );
        assert_eq!(preview.duplicate_rows.len(), 1);
        // preview の後に連動なしの売上を取り込んだ → 確定の TX で当たり全体を戻す
        z004_sale(&conn, "G-LATER", import_id, 1, 100, false);
        let rows = preview
            .duplicate_rows
            .into_iter()
            .map(|row| row.import_row)
            .collect();
        let error = commit_import(&mut conn, rows, vec!["G-LATER".to_string()]).unwrap_err();
        assert!(
            matches!(&error, BizError::ValidationFailed(text)
                if text.starts_with(&format!("商品 G-LATER の行: {RECOUNT_REQUIRED}。"))),
            "{error:?}"
        );
        let product = product_repo::find_by_product_code(&conn, "G-LATER")
            .unwrap()
            .unwrap();
        assert!(!product.product.pos_stock_sync);
    }

    #[test]
    #[serial]
    fn test_update_product_req102_toggle_stock_link_recomputes() {
        // REQ-102 / ADR D8 / 30（b1 Matrix J6）: true → false は記録も確認も要らず、連動の切替の TX で
        // その商品を求め直す（写しをわざと食い違わせて観測する）
        let (_dir, mut conn) = setup_test_db();
        seed_jan_product(&conn, "T-1", None, true);
        let copy = |conn: &DbConnection| -> i64 {
            conn.query_row(
                "SELECT stock_quantity FROM products WHERE product_code = 'T-1'",
                [],
                |row| row.get(0),
            )
            .unwrap()
        };
        for linked in [false, true] {
            conn.execute(
                "UPDATE products SET stock_quantity = 77 WHERE product_code = 'T-1'",
                [],
            )
            .unwrap();
            update_product(
                &mut conn,
                "T-1",
                &ProductUpdateRequest {
                    pos_stock_sync: Some(linked),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(copy(&conn), 0, "linked={linked}");
        }
        let movements: i64 = conn
            .query_row("SELECT COUNT(*) FROM inventory_movements", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(movements, 0);
    }
}
