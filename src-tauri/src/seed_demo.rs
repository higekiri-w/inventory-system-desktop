//! デモデータ投入ロジック（開発ツール用）
//!
//! `src/bin/seed_demo_data.rs` から呼び出す seed 処理本体。
//! Phase 2 UI-00 以降の UI 開発で手動 SQL 投入なしにダッシュボード数値が確認できるよう、
//! 商品 100 件 / 取引先 5 件 / 売上 30 日分 (300 件) / 在庫変動 400 件 を投入する。
//!
//! ## 特徴
//! - 決定的動作: `rand::rngs::StdRng::seed_from_u64(42)` で乱数固定、日付も 2026-03-22 基準固定
//! - 冪等性: 全 phase + オプショナル reset を 1 トランザクションで実行。partial failure 時は
//!   自動 rollback で reset 前の状態に復帰。正常完了後の再実行は `ON CONFLICT DO NOTHING`
//!   + SELECT gate で全 skip
//! - 部門は既存 21 部門（migration 初期投入）を参照し追加しない
//! - 在庫数 bucket (PR-3): 各部門の index `i==1` を在庫切れ (0)、`i==2` を在庫少
//!   (cm 300 / pcs 2) に固定し、色分け契約 H（在庫切れ / 在庫少の別表示）を seed データで
//!   検証可能にする。残り (`i>=3`) は従来ランダム値で rng 消費順序を保持する。
//!   **前提**: 各部門 `count>=2`（DEPARTMENT_SEED_PLAN 最小 16）。`count==1` の部門を追加すると
//!   `i==2` が出ず在庫少が欠けるため、新規部門追加時は count を 2 以上にすること
//! - 商品JANは CV17 1.1.1 のスキャニングPLU検証に使える13桁EANを生成する。JANなし商品の
//!   業務フローは UI-01b で扱うが、デモseedは UI-08 L3 を阻害しないデータにする。

use crate::biz::unit_amount::{
    price_basis_quantity, sale_amount_yen, stock_unit_kind, StockUnitKind,
};
use crate::db::product_repo::ProductStockUnit;
use crate::db::DbError;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusqlite::Connection;
use std::collections::HashMap;

// -------- 定数 --------

/// 決定的 seed 日付 (YYYY-MM-DD)
pub const BASE_DATE: &str = "2026-03-22";
/// 初期 receiving timestamp
pub const BASE_TIMESTAMP: &str = "2026-03-22T09:00:00";
/// 売上を生成する日数（BASE_DATE から）
pub const SALES_DAYS: u32 = 30;
/// 生成する sale_records 件数
pub const SALES_RECORDS: u32 = 300;
/// 乱数の決定 seed
pub const RNG_SEED: u64 = 42;

/// 部門別 seed 仕様 (department_id, 件数, product_code_prefix, stock_unit, name_template)
///
/// 毛糸 (id=3) と 針 (id=20) は DB 上 `code_prefix=NULL` のため seed 独自で `KE` / `NR` を割当。
/// 他 4 部門は schema_v1 初期投入の `code_prefix` と揃えた。
pub const DEPARTMENT_SEED_PLAN: &[(i64, u32, &str, &str, &str)] = &[
    (3, 17, "KE", "pcs", "毛糸"),
    (8, 17, "NU", "m", "無地生地"),
    (4, 17, "SY", "pcs", "手芸材料"),
    (18, 17, "BT", "pcs", "ボタン"),
    (19, 16, "FS", "pcs", "ファスナー"),
    (20, 16, "NR", "pcs", "針"),
];

/// D-028 PLU書出し確認用に追加する固定商品数。
pub const PLU_BUCKET_DEMO_COUNT: u32 = 6;

/// 架空の取引先 5 件
pub const SUPPLIER_NAMES: &[&str] = &[
    "鈴木糸工業",
    "京都手芸材料",
    "大阪ボタン商事",
    "東京ファスナー",
    "日本手芸卸",
];

// -------- Error 型 --------

/// seed 処理用エラー（dev tooling 限定）
#[derive(Debug)]
pub enum SeedError {
    Db(DbError),
}

impl std::fmt::Display for SeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedError::Db(e) => write!(f, "DBエラー: {}", e),
        }
    }
}

impl std::error::Error for SeedError {}

impl From<DbError> for SeedError {
    fn from(e: DbError) -> Self {
        SeedError::Db(e)
    }
}

impl From<rusqlite::Error> for SeedError {
    fn from(e: rusqlite::Error) -> Self {
        SeedError::Db(DbError::from(e))
    }
}

// -------- 公開 API --------

/// seed 完了サマリ
#[derive(Debug, Default, Clone)]
pub struct SeedSummary {
    pub suppliers_inserted: u32,
    pub suppliers_skipped: u32,
    pub products_inserted: u32,
    pub products_skipped: u32,
    pub receiving_movements_inserted: u32,
    pub receiving_movements_skipped: u32,
    pub sale_records_inserted: u32,
    pub sale_records_skipped: u32,
    pub sale_movements_inserted: u32,
    pub sale_movements_skipped: u32,
    pub negative_stock_warnings: u32,
}

/// 対象 DB を全 DELETE（departments / app_settings / schema_versions は保護）
///
/// `--reset` フラグ経由で呼ばれる。seed 対象テーブルのみを FK 順序で削除する。
pub fn delete_all(conn: &Connection) -> Result<(), SeedError> {
    // stock_counts は products と stocktake_items を外部キーで参照するので先に消す（D-118 b1）
    let tables = [
        "stock_counts",
        "inventory_movements",
        "sale_records",
        "price_history",
        "stocktake_items",
        "stocktakes",
        "receiving_items",
        "receiving_records",
        "return_items",
        "return_records",
        "manual_sale_items",
        "manual_sales",
        "disposal_items",
        "disposal_records",
        "csv_import_errors",
        "csv_imports",
        "products",
        "suppliers",
    ];
    for t in tables {
        conn.execute(&format!("DELETE FROM {}", t), [])?;
    }
    Ok(())
}

/// seed メインエントリ: suppliers → products → 初期 receiving → sales の順で投入。
///
/// `reset=true` の場合は TX 内で `delete_all` を先に実行してから seed する。
/// 全 phase を 1 トランザクションで実行し、途中失敗時は自動 rollback で呼び出し前の状態に戻る。
pub fn run_seed(conn: &mut Connection, reset: bool) -> Result<SeedSummary, SeedError> {
    let mut summary = SeedSummary::default();
    let tx = conn.transaction()?;
    if reset {
        delete_all(&tx)?;
    }
    seed_suppliers(&tx, &mut summary)?;
    let specs = seed_products(&tx, &mut summary)?;
    seed_initial_receiving(&tx, &specs, &mut summary)?;
    seed_sales(&tx, &specs, &mut summary)?;
    recompute_all_stocks(&tx)?;
    tx.commit()?;
    Ok(summary)
}

/// 写しを記録へ揃える（seed は写しを `seed_products` の初期値で置き、sale_auto の movement で和だけを
/// 減らすので、揃えないと最初の書込みで写しが和へ跳ぶ。D-118 (a)、Matrix T2）
fn recompute_all_stocks(tx: &rusqlite::Transaction<'_>) -> Result<(), SeedError> {
    let codes: Vec<String> = {
        let mut stmt = tx.prepare("SELECT product_code FROM products ORDER BY product_code")?;
        let rows = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for code in codes {
        crate::biz::stock_ledger::recompute_stock(tx, &code)
            .map_err(|e| SeedError::Db(DbError::QueryFailed(e.to_string())))?;
    }
    Ok(())
}

// -------- Phase 1: suppliers --------

fn seed_suppliers(conn: &Connection, summary: &mut SeedSummary) -> Result<(), SeedError> {
    let mut stmt = conn.prepare(
        "INSERT INTO suppliers (name, created_at) VALUES (?1, ?2) \
         ON CONFLICT(name) DO NOTHING",
    )?;
    for name in SUPPLIER_NAMES {
        let n = stmt.execute(rusqlite::params![name, BASE_TIMESTAMP])?;
        if n > 0 {
            summary.suppliers_inserted += 1;
        } else {
            summary.suppliers_skipped += 1;
        }
    }
    Ok(())
}

// -------- Phase 2: products --------

/// 後続 phase で使う商品情報（in-memory）
#[derive(Debug, Clone)]
pub struct ProductSpec {
    pub product_code: String,
    pub selling_price: i64,
    pub stock_quantity: i64,
    pub stock_unit: ProductStockUnit,
}

fn seed_products(
    conn: &Connection,
    summary: &mut SeedSummary,
) -> Result<Vec<ProductSpec>, SeedError> {
    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let mut specs: Vec<ProductSpec> = Vec::with_capacity(100);

    let supplier_ids: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT id FROM suppliers ORDER BY id")?;
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
        let mut ids = Vec::new();
        for r in rows {
            ids.push(r?);
        }
        ids
    };
    if supplier_ids.is_empty() {
        return Err(SeedError::Db(DbError::QueryFailed(
            "suppliers が空: seed_suppliers を先に実行してください".to_string(),
        )));
    }

    let mut stmt = conn.prepare(
        "INSERT INTO products (
            product_code, jan_code, name, department_id, supplier_id,
            selling_price, cost_price_centi, tax_rate, stock_quantity, stock_unit,
            is_discontinued, plu_dirty, plu_exported_at, plu_target, pos_stock_sync,
            created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, '10', ?8, ?9,
            0, 1, NULL, ?10, 1,
            ?11, ?11
        ) ON CONFLICT(product_code) DO NOTHING",
    )?;

    for (dept_id, count, prefix, stock_unit, name_label) in DEPARTMENT_SEED_PLAN {
        let unit = ProductStockUnit::from_code(stock_unit).ok_or_else(|| {
            SeedError::Db(DbError::QueryFailed(format!(
                "seed の在庫単位が一覧にありません: {stock_unit}"
            )))
        })?;
        // 数量の種類で在庫の桁と在庫少の基準が変わる（SPEC-UNIT-D10）
        let is_length = stock_unit_kind(unit) == StockUnitKind::Length;
        for i in 1..=*count {
            let product_code = format!("{}{:04}", prefix, i);
            let jan_code = gen_ean13(&mut rng);
            let name = format!("{} #{:02}", name_label, i);
            let supplier_idx = rng.gen_range(0..supplier_ids.len());
            let supplier_id = supplier_ids[supplier_idx];
            let selling_price: i64 = rng.gen_range(100..=5000);
            // 原価は売価 × 0.6 円を 1/100 円で正確に持つ（浮動小数を通さない。SPEC-UNIT-D5）
            let cost_price_centi: i64 = selling_price * 60;
            // 在庫数は normal 値を draw した上で index i bucket で上書きする (PR-3 (e))。
            // drawn_stock は normal 件 (i>=3) で実際に使うことで rng カーソルを保持し、
            // 後続の jan_code/selling_price の rng 列を不変に保つ
            // (seed_uses_deterministic_rng が無変更で pass)。
            let drawn_stock: i64 = if is_length {
                rng.gen_range(500..=5000)
            } else {
                rng.gen_range(10..=200)
            };
            // 色分け契約 H 検証用に各部門の先頭 2 件を在庫切れ / 在庫少に固定する。
            let stock_quantity: i64 = match i {
                1 => 0, // 在庫切れ (stockout、cm/pcs 共通)
                2 => {
                    if is_length {
                        300 // 在庫少 (cm 300 <= 閾値 500、D-4 stock_low_threshold_fabric)
                    } else {
                        2 // 在庫少 (pcs 2 <= 閾値 3、D-4 stock_low_threshold)
                    }
                }
                _ => drawn_stock, // 従来ランダム normal (cm 500-5000 / pcs 10-200)
            };

            let n = stmt.execute(rusqlite::params![
                &product_code,
                &jan_code,
                &name,
                dept_id,
                supplier_id,
                selling_price,
                cost_price_centi,
                stock_quantity,
                stock_unit,
                true,
                BASE_TIMESTAMP,
            ])?;
            if n > 0 {
                summary.products_inserted += 1;
            } else {
                summary.products_skipped += 1;
            }
            specs.push(ProductSpec {
                product_code,
                selling_price,
                stock_quantity,
                stock_unit: unit,
            });
        }
    }
    seed_plu_bucket_demo_products(
        &mut stmt,
        &mut summary.products_inserted,
        &mut summary.products_skipped,
    )?;
    Ok(specs)
}

fn seed_plu_bucket_demo_products(
    stmt: &mut rusqlite::Statement<'_>,
    inserted: &mut u32,
    skipped: &mut u32,
) -> Result<(), SeedError> {
    struct DemoProduct {
        code: &'static str,
        jan: Option<&'static str>,
        name: &'static str,
        selling_price: i64,
        plu_target: bool,
    }

    let demos = [
        DemoProduct {
            code: "D028-NO-JAN",
            jan: None,
            name: "D-028 JANなし独自コード",
            selling_price: 500,
            plu_target: false,
        },
        DemoProduct {
            code: "D028-DEDUP-A",
            jan: Some("4901234567894"),
            name: "D-028 同一JAN 同価格A",
            selling_price: 600,
            plu_target: true,
        },
        DemoProduct {
            code: "D028-DEDUP-B",
            jan: Some("4901234567894"),
            name: "D-028 同一JAN 同価格B",
            selling_price: 600,
            plu_target: true,
        },
        DemoProduct {
            code: "D028-MISMATCH-A",
            jan: Some("4901234567801"),
            name: "D-028 同一JAN 価格不一致A",
            selling_price: 700,
            plu_target: true,
        },
        DemoProduct {
            code: "D028-MISMATCH-B",
            jan: Some("4901234567801"),
            name: "D-028 同一JAN 価格不一致B",
            selling_price: 710,
            plu_target: true,
        },
        DemoProduct {
            code: "D028-BAD-CHECK",
            jan: Some("4901234567890"),
            name: "D-028 JANチェック不正",
            selling_price: 800,
            plu_target: true,
        },
    ];

    for product in demos {
        let n = stmt.execute(rusqlite::params![
            product.code,
            product.jan,
            product.name,
            1_i64,
            Option::<i64>::None,
            product.selling_price,
            (product.selling_price as f64 * 0.6) as i64,
            0_i64,
            "pcs",
            product.plu_target,
            BASE_TIMESTAMP,
        ])?;
        if n > 0 {
            *inserted += 1;
        } else {
            *skipped += 1;
        }
    }

    Ok(())
}

// -------- Phase 3: 初期 receiving inventory_movements --------

fn seed_initial_receiving(
    conn: &Connection,
    specs: &[ProductSpec],
    summary: &mut SeedSummary,
) -> Result<(), SeedError> {
    // inventory_movements にはユニーク制約がないため、既存有無は
    // (product_code + movement_type='receiving' + created_at=BASE_TIMESTAMP) で判定し、
    // 2 回目以降は insert を skip することで冪等性を確保する。
    let mut exists_stmt = conn.prepare(
        "SELECT COUNT(*) FROM inventory_movements \
         WHERE product_code = ?1 AND movement_type = 'receiving' AND created_at = ?2",
    )?;
    let mut insert_stmt = conn.prepare(
        "INSERT INTO inventory_movements (
            product_code, movement_type, quantity, stock_after,
            reference_type, reference_id, note, is_voided, created_at, business_date, phase
        ) VALUES (?1, 'receiving', ?2, ?3, NULL, NULL, NULL, 0, ?4, ?5, 0)",
    )?;

    for spec in specs {
        let exists: i64 = exists_stmt.query_row(
            rusqlite::params![&spec.product_code, BASE_TIMESTAMP],
            |row| row.get(0),
        )?;
        if exists > 0 {
            summary.receiving_movements_skipped += 1;
            continue;
        }
        insert_stmt.execute(rusqlite::params![
            &spec.product_code,
            spec.stock_quantity,
            spec.stock_quantity,
            BASE_TIMESTAMP,
            BASE_DATE,
        ])?;
        summary.receiving_movements_inserted += 1;
    }
    Ok(())
}

// -------- Phase 4: sale_records + sale_auto inventory_movements --------

fn seed_sales(
    conn: &Connection,
    specs: &[ProductSpec],
    summary: &mut SeedSummary,
) -> Result<(), SeedError> {
    let mut rng = StdRng::seed_from_u64(RNG_SEED.wrapping_add(1));

    let mut current_stock: HashMap<String, i64> = specs
        .iter()
        .map(|s| (s.product_code.clone(), s.stock_quantity))
        .collect();
    let price_lookup: HashMap<String, i64> = specs
        .iter()
        .map(|s| (s.product_code.clone(), s.selling_price))
        .collect();

    // 冪等性: 既に source='auto' かつ csv_import_id IS NULL の sale_records が SALES_RECORDS 件以上
    // ある場合は全件 skip。これで重複 sale の積み増しを防ぐ。
    let existing_auto_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sale_records WHERE source = 'auto' AND csv_import_id IS NULL",
        [],
        |row| row.get(0),
    )?;
    if existing_auto_count >= SALES_RECORDS as i64 {
        summary.sale_records_skipped = SALES_RECORDS;
        summary.sale_movements_skipped = SALES_RECORDS;
        return Ok(());
    }

    let mut sale_stmt = conn.prepare(
        "INSERT INTO sale_records (
            csv_import_id, product_code, sale_date, quantity, amount,
            source, source_line_no, reason, note, is_voided, created_at
        ) VALUES (NULL, ?1, ?2, ?3, ?4, 'auto', NULL, NULL, NULL, 0, ?5)",
    )?;
    let mut movement_stmt = conn.prepare(
        "INSERT INTO inventory_movements (
            product_code, movement_type, quantity, stock_after,
            reference_type, reference_id, note, is_voided, created_at, business_date, phase
        ) VALUES (?1, 'sale_auto', ?2, ?3, NULL, NULL, NULL, 0, ?4, ?5, 0)",
    )?;

    for _ in 0..SALES_RECORDS {
        let spec_idx = rng.gen_range(0..specs.len());
        let product_code = specs[spec_idx].product_code.clone();
        let basis = price_basis_quantity(specs[spec_idx].stock_unit);
        let selling_price = price_lookup.get(&product_code).copied().ok_or_else(|| {
            SeedError::Db(DbError::QueryFailed(format!(
                "price_lookup に {} が無い",
                product_code
            )))
        })?;

        let day_offset = rng.gen_range(0..SALES_DAYS);
        let sale_date = date_plus_days(day_offset);
        let created_at = format!("{}T12:00:00", sale_date);
        let quantity: i64 = rng.gen_range(1..=5);
        // 売価は基準数量ぶんの値（長さは 1 m あたり。SPEC-UNIT-D7）
        let amount: i64 = sale_amount_yen(selling_price, quantity, basis)
            .map_err(|e| SeedError::Db(DbError::QueryFailed(e.to_string())))?;

        let stock_entry = current_stock.entry(product_code.clone()).or_insert(0);
        let new_stock = *stock_entry - quantity;
        if new_stock < 0 {
            summary.negative_stock_warnings += 1;
            tracing::warn!(
                product_code = %product_code,
                before = *stock_entry,
                after = new_stock,
                "seed: sale で在庫が負になりました (補正なしで続行)"
            );
        }
        *stock_entry = new_stock;

        sale_stmt.execute(rusqlite::params![
            &product_code,
            &sale_date,
            quantity,
            amount,
            &created_at,
        ])?;
        summary.sale_records_inserted += 1;

        movement_stmt.execute(rusqlite::params![
            &product_code,
            -quantity,
            new_stock,
            &created_at,
            &sale_date,
        ])?;
        summary.sale_movements_inserted += 1;
    }
    Ok(())
}

// -------- ユーティリティ --------

fn gen_ean13(rng: &mut StdRng) -> String {
    let mut body = String::with_capacity(12);
    body.push_str("49");
    for _ in 0..10 {
        body.push(char::from_digit(rng.gen_range(0..10), 10).unwrap_or('0'));
    }
    let sum: u32 = body
        .chars()
        .enumerate()
        .map(|(idx, ch)| {
            let digit = ch.to_digit(10).unwrap_or(0);
            if idx % 2 == 0 {
                digit
            } else {
                digit * 3
            }
        })
        .sum();
    let check = (10 - (sum % 10)) % 10;
    format!("{}{}", body, check)
}

/// `BASE_DATE` + offset_days を `YYYY-MM-DD` で返す。
///
/// BASE_DATE は定数で有効な日付を指しているため、fallback の `from_ymd_opt` も必ず Some を返す。
/// 万一両方失敗した場合は seed 処理を続行できないが、BASE_DATE は人間可読の定数なのでここで panic はしない。
/// 代わりに空文字を返し、呼び出し側の INSERT で `NOT NULL` 制約違反として検出される設計とする。
fn date_plus_days(offset: u32) -> String {
    use chrono::NaiveDate;
    let base = NaiveDate::parse_from_str(BASE_DATE, "%Y-%m-%d")
        .ok()
        .or_else(|| NaiveDate::from_ymd_opt(2026, 3, 22));
    match base {
        Some(b) => {
            let d = b + chrono::Duration::days(offset as i64);
            d.format("%Y-%m-%d").to_string()
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biz::plu_export_service::{
        prepare_plu_export, ExportMode, PluExcludedReason, PluExportPrepareRequest,
    };
    use crate::db::init_database;

    #[test]
    fn test_seed_req402_supports_plu_export_three_bucket_demo() {
        // REQ-402 / D-028: seed DB で Full prepare が成功し、代表化/対象外理由を確認できる。
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("seed-test.db");
        let mut conn = init_database(db_path.to_str().unwrap()).unwrap();
        run_seed(&mut conn, false).unwrap();
        crate::db::system_repo::upsert_setting(
            &conn,
            "plu_register_snapshot_at",
            "2026-08-18T00:00:00",
        )
        .unwrap();

        let result = prepare_plu_export(
            &mut conn,
            PluExportPrepareRequest {
                mode: ExportMode::Full,
            },
        )
        .expect("Full prepare should succeed with D-028 demo buckets");

        assert_eq!(result.count, 101);
        assert_eq!(result.target_product_codes.len(), 102);
        assert!(result
            .target_product_codes
            .contains(&"D028-DEDUP-A".to_string()));
        assert!(result
            .target_product_codes
            .contains(&"D028-DEDUP-B".to_string()));
        assert_eq!(result.excluded.len(), 3);
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "D028-BAD-CHECK"
                && matches!(excluded.reason, PluExcludedReason::InvalidCheckDigit)
        }));
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "D028-MISMATCH-A"
                && matches!(excluded.reason, PluExcludedReason::GroupPriceMismatch)
        }));
        assert!(result.excluded.iter().any(|excluded| {
            excluded.product_code == "D028-MISMATCH-B"
                && matches!(excluded.reason, PluExcludedReason::GroupPriceMismatch)
        }));
    }
}
