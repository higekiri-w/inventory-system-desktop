//! docs/diagrams/cross-feature-verification.md XFA-D1〜D5。
//! 実BIZを有限の操作列で実行する。期待値は合成業務イベントだけから更新する。

use crate::biz::csv_import_service::test_support::*;
use crate::biz::csv_import_service::*;
use crate::biz::stocktake_service::{self, CompletionBlockReason};
use crate::biz::{integrity_service, inventory_service, sales_service, BizError};
use crate::db::inventory_repo::{self, MovementType, NewMovement};
use crate::db::{self, DbConnection};
use rusqlite::types::Value;
use serde_json::json;
use std::collections::BTreeSet;

const DATE: &str = "2026-09-16";
const CODES: [&str; 2] = ["XFA-SYNC", "XFA-NOSYNC"];
const JANS: [&str; 2] = ["2900000000018", "2900000000025"];
const INITIAL: [i64; 2] = [10, 20];

#[derive(Clone, Copy, Debug)]
enum Operation {
    Receive,
    ManualSale,
    Dispose,
    Return,
    RegisterProcessedReturn,
    CsvSale,
    CsvReturn,
    CsvNoSync,
    Reopen,
}

const OPERATIONS: [Operation; 9] = [
    Operation::Receive,
    Operation::ManualSale,
    Operation::Dispose,
    Operation::Return,
    Operation::RegisterProcessedReturn,
    Operation::CsvSale,
    Operation::CsvReturn,
    Operation::CsvNoSync,
    Operation::Reopen,
];

#[derive(Debug)]
struct Model {
    stock: [i64; 2],
    // 商品 × source（auto / manual）ごとの (数量, 金額)。
    sales: [[(i64, i64); 2]; 2],
}

impl Model {
    fn sale(&mut self, sku: usize, source: usize, quantity: i64, amount: i64) {
        self.sales[sku][source].0 += quantity;
        self.sales[sku][source].1 += amount;
    }

    fn csv(&mut self, sku: usize, quantity: i64, amount: i64) {
        self.sale(sku, 0, quantity, amount);
        if sku == 0 {
            self.stock[sku] -= quantity;
        }
    }
}

#[derive(Clone)]
struct Import {
    id: i64,
    sku: usize,
    quantity: i32,
    amount: i32,
    bytes: Vec<u8>,
    active: bool,
}

struct World {
    // 接続を先にdropしてから一時directoryを消す（Windowsのopen-file制約にも従う）。
    conn: Option<DbConnection>,
    dir: tempfile::TempDir,
    model: Model,
    imports: Vec<Import>,
    receiving: Option<inventory_service::ReceivingCreateRequest>,
    trace: Vec<String>,
    /// 次に取り込む精算回数（新しい file ごとに 1 つ進める。取り込み直しは同じ file の番号のまま）
    next_no: i64,
}

impl World {
    fn new() -> Self {
        let (dir, conn) = db::test_support::setup_test_db();
        for sku in 0..CODES.len() {
            create_test_product_with_jan(&conn, CODES[sku], JANS[sku], INITIAL[sku], sku == 0);
            // 初期在庫の movement は create_test_product_with_jan が入れる（Matrix T3・T5）
        }
        Self {
            dir,
            conn: Some(conn),
            model: Model {
                stock: INITIAL,
                sales: [[(0, 0); 2]; 2],
            },
            imports: Vec::new(),
            receiving: None,
            trace: Vec::new(),
            next_no: 1,
        }
    }

    fn conn(&self) -> &DbConnection {
        self.conn.as_ref().unwrap()
    }
    fn conn_mut(&mut self) -> &mut DbConnection {
        self.conn.as_mut().unwrap()
    }

    fn perform(&mut self, operation: Operation) {
        self.trace.push(format!("{operation:?}"));
        let key = format!("xfa-{}", self.trace.len());
        match operation {
            Operation::Receive => {
                let request = serde_json::from_value::<inventory_service::ReceivingCreateRequest>(json!({
                    "idempotency_key":key, "supplier_id":null, "receiving_date":DATE, "note":null,
                    "items":[{"product_code":CODES[0],"quantity":3,"cost_price_centi":50}]
                })).unwrap();
                let result =
                    inventory_service::create_receiving(self.conn_mut(), request.clone()).unwrap();
                assert!(result.created);
                self.receiving = Some(request);
                self.model.stock[0] += 3;
            }
            Operation::ManualSale => {
                let request = serde_json::from_value(json!({
                    "idempotency_key":key, "sale_date":DATE, "reason":"other", "note":null,
                    "items":[{"product_code":CODES[0],"quantity":2,"amount":246}],
                    "confirmation_token":null
                }))
                .unwrap();
                let result =
                    inventory_service::create_manual_sale(self.conn_mut(), request).unwrap();
                assert!(result.created && !result.needs_confirmation);
                self.model.stock[0] -= 2;
                self.model.sale(0, 1, 2, 246);
            }
            Operation::Dispose => {
                let request = serde_json::from_value(json!({
                    "idempotency_key":key, "disposal_date":DATE,
                    "items":[{"product_code":CODES[0],"disposal_type":"disposal",
                        "quantity":1,"cost_price_centi":50,"reason":"合成検証"}]
                }))
                .unwrap();
                assert!(
                    inventory_service::create_disposal(self.conn_mut(), request)
                        .unwrap()
                        .created
                );
                self.model.stock[0] -= 1;
            }
            Operation::Return | Operation::RegisterProcessedReturn => {
                let register_processed = matches!(operation, Operation::RegisterProcessedReturn);
                let request = serde_json::from_value(json!({
                    "idempotency_key":key, "return_type":"return", "return_date":DATE,
                    "register_processed":register_processed, "receipt_image_path":null, "note":null,
                    "items":[{"product_code":CODES[0],"direction":"in","quantity":1}]
                }))
                .unwrap();
                assert!(
                    inventory_service::create_return(self.conn_mut(), request)
                        .unwrap()
                        .created
                );
                if !register_processed {
                    self.model.stock[0] += 1;
                }
            }
            Operation::CsvSale => {
                self.import_new(0, 2, 198);
            }
            Operation::CsvReturn => {
                self.import_new(0, -1, -73);
            }
            Operation::CsvNoSync => {
                self.import_new(1, 2, 164);
            }
            Operation::Reopen => {
                drop(self.conn.take());
                self.conn = Some(
                    db::init_database(self.dir.path().join("test.db").to_str().unwrap()).unwrap(),
                );
            }
        }
    }

    fn import_new(&mut self, sku: usize, quantity: i32, amount: i32) -> usize {
        // 合成の商品名を変え、同日でも別hashのファイルを作る。oracleはparse結果を使わない。
        // 精算回数は新しい file ごとに進める（32 の順 4: 同じ精算の別の版は取り込まない）
        let label = format!("合成{}-{}", self.trace.len(), self.imports.len());
        let no = self.next_no;
        self.next_no += 1;
        let bytes = make_z004_bytes(no, DATE, &[(JANS[sku], &label, quantity, amount)]);
        self.commit_bytes(sku, quantity, amount, bytes)
    }

    fn commit_bytes(&mut self, sku: usize, quantity: i32, amount: i32, bytes: Vec<u8>) -> usize {
        let preview = parse_and_build_cache(self.conn(), bytes.clone(), "XFA-synthetic.csv");
        let result = commit_csv_import(
            self.conn_mut(),
            CommitRequest {
                cached_data: build_cached(preview),
                today: DATE.to_string(),
            },
        )
        .unwrap();
        self.model.csv(sku, i64::from(quantity), i64::from(amount));
        self.imports.push(Import {
            id: result.csv_import_id,
            sku,
            quantity,
            amount,
            bytes,
            active: true,
        });
        self.imports.len() - 1
    }

    fn rollback(&mut self, index: usize) {
        self.trace.push(format!("Rollback(import={index})"));
        let import = self.imports[index].clone();
        let result = rollback_csv_import(self.conn_mut(), import.id).unwrap();
        assert!(result.success);
        if import.active {
            self.model.csv(
                import.sku,
                -i64::from(import.quantity),
                -i64::from(import.amount),
            );
            self.imports[index].active = false;
        } else {
            assert_eq!(result.voided_sale_count, 0);
            assert_eq!(result.voided_movement_count, 0);
        }
    }

    fn repeat_import(&mut self, index: usize) {
        self.trace.push(format!("RepeatImport(import={index})"));
        let import = self.imports[index].clone();
        if self
            .imports
            .iter()
            .any(|entry| entry.active && entry.bytes == import.bytes)
        {
            let before = self.business_snapshot();
            let result = parse_and_validate(
                self.conn(),
                CsvParseAndValidateRequest {
                    file_bytes: import.bytes,
                    filename: "same-hash.csv".into(),
                    today: DATE.to_string(),
                },
            );
            assert!(matches!(result, Err(BizError::ImportError(_))));
            assert_eq!(before, self.business_snapshot(), "prefix={:?}", self.trace);
        } else {
            self.commit_bytes(import.sku, import.quantity, import.amount, import.bytes);
        }
    }

    fn repeat_receiving(&mut self, conflict: bool) {
        self.trace
            .push(format!("RepeatReceiving(conflict={conflict})"));
        let before = self.business_snapshot();
        let mut request = self.receiving.as_ref().unwrap().clone();
        if conflict {
            request.items[0].quantity += 1;
        }
        let result = inventory_service::create_receiving(self.conn_mut(), request);
        if conflict {
            assert!(matches!(result, Err(BizError::IdempotencyConflict(_))));
        } else {
            let result = result.unwrap();
            assert!(result.idempotent_replay && !result.created);
        }
        assert_eq!(before, self.business_snapshot(), "prefix={:?}", self.trace);
    }

    fn stock(&self, sku: usize) -> i64 {
        self.conn()
            .query_row(
                "SELECT stock_quantity FROM products WHERE product_code=?1",
                [CODES[sku]],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn mismatch(&self) -> Option<String> {
        let stocks = [self.stock(0), self.stock(1)];
        let sums: Vec<i64> = CODES.iter().map(|code| {
            self.conn().query_row("SELECT COALESCE(SUM(quantity),0) FROM inventory_movements WHERE product_code=?1 AND is_voided=0", [code], |r| r.get(0)).unwrap()
        }).collect();
        let report = sales_service::get_daily_sales(self.conn(), DATE).unwrap();
        let mut sales = [[(0, 0); 2]; 2];
        for item in report.items {
            let Some(sku) = CODES.iter().position(|code| *code == item.product_code) else {
                return Some(format!("unexpected product {}", item.product_code));
            };
            let source = match item.source {
                sales_service::DailySaleSource::Auto => 0,
                sales_service::DailySaleSource::Manual => 1,
            };
            sales[sku][source].0 += item.quantity;
            sales[sku][source].1 += item.amount;
        }
        let expected_total = self
            .model
            .sales
            .iter()
            .flatten()
            .fold((0, 0), |sum, value| (sum.0 + value.0, sum.1 + value.1));
        // fixture は個数の商品だけなので、数量は count_points に入り length_cm は 0（SPEC-UNIT-D11）
        let total = (report.grand_total.count_points, report.grand_total.amount);
        if report.grand_total.length_cm != 0 {
            return Some(format!(
                "length_cm should be 0: {}",
                report.grand_total.length_cm
            ));
        }
        if stocks != self.model.stock
            || sums != self.model.stock
            || sales != self.model.sales
            || total != expected_total
        {
            Some(format!("expected={:?}; actual stock={stocks:?}, ledger={sums:?}, sales={sales:?}, total={total:?}", self.model))
        } else {
            None
        }
    }

    fn check(&self) {
        if let Some(error) = self.mismatch() {
            panic!("prefix={:?}\n{error}", self.trace);
        }
    }

    fn business_snapshot(&self) -> Vec<Vec<Vec<Value>>> {
        // 操作ログは拒否理由を記録してよい。業務データは全列比較し、件数だけにしない。
        [
            "products",
            "receiving_records",
            "receiving_items",
            "return_records",
            "return_items",
            "manual_sales",
            "manual_sale_items",
            "disposal_records",
            "disposal_items",
            "csv_imports",
            "csv_import_errors",
            "sale_records",
            "inventory_movements",
        ]
        .iter()
        .map(|table| {
            let mut stmt = self
                .conn()
                .prepare(&format!("SELECT * FROM {table} ORDER BY 1"))
                .unwrap();
            let width = stmt.column_count();
            stmt.query_map([], |row| (0..width).map(|col| row.get(col)).collect())
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        })
        .collect()
    }
}

#[test]
fn test_cross_feature_req201_req202_req203_req204_req401_model() {
    // REQ-201 / REQ-202 / REQ-203 / REQ-204 / REQ-401: XFA-D1,D3。
    let mut traces = 0;
    for first in OPERATIONS {
        for second in OPERATIONS {
            let mut world = World::new();
            world.check();
            for operation in [first, second, Operation::Receive, Operation::Reopen] {
                world.perform(operation);
                world.check();
            }
            world.repeat_receiving(false);
            world.check();
            world.repeat_receiving(true);
            world.check();
            world.trace.push("AdditionalSale".into());
            let sale = world.import_new(0, 3, 357);
            world.check();
            world.trace.push("AdditionalReturn".into());
            world.import_new(0, -2, -222);
            world.check();
            world.repeat_import(sale);
            world.check();
            world.rollback(sale);
            world.check();
            world.rollback(sale);
            world.check();
            world.repeat_import(sale);
            world.check();
            world.repeat_import(sale);
            world.check();
            world.rollback(sale);
            world.check();
            world.perform(Operation::Reopen);
            world.check();
            traces += 1;
        }
    }
    println!("XFA_NORMAL traces={traces}; model comparisons after every operation: PASS");
}

// --- STK-1・STK-2 と対照の trace（Matrix の X の節の「trace の新しい形」。ADR D3・D6、35） ---
//
// D = DATE を今日 T とする。World の初めに精算 1（精算日 D−1、正常な行が全て 0）を取り込む。
// oracle: (a) 未取込みの売上が無い時点で、SKU0 が要再確認なら範囲が物理の数を含み、そうでなければ写しが
// 物理の数と同じ（黙ってずれない）、(b) 完了が通ったら写しが完了の前と同じで物理の数と同じ、
// (c) trace ごとの completion_blocked の列が期待と同じ。

const PREVIOUS_DAY: &str = "2026-09-15";

#[derive(Clone, Copy, Debug)]
enum Temporal {
    /// 棚卸しを開始する
    Start,
    /// SKU0 の既定の計数（日付 D、値は物理の数）
    Count,
    /// SKU0 の fresh な計数（今日 D）
    Recount,
    /// 業務日 D の入出庫
    Move(Operation),
    /// 物理の数 −2。次の精算に載る
    PosSale,
    /// 次の精算回数・精算日 D の Z004 を、未取込みの売上の行で取り込む（無ければ全て 0 の file）
    Close,
    /// XFA-NOSYNC を 0 にしてから完了する。completion_blocked なら理由の集合を積んで続ける
    Complete,
    /// 保留した精算（STK-2 の fixture）を取り込む
    ImportHeld,
}

/// 保留した精算（STK-2）: 精算 2（精算日 D−2）の SKU0 × 2。物理の数は数える前に 8
struct Held {
    bytes: Vec<u8>,
    quantity: i64,
}

struct TemporalWorld {
    world: World,
    physical: i64,
    pending: i64,
    held: Option<Held>,
    stocktake: Option<i64>,
    zeroed: bool,
    blocked: Vec<BTreeSet<CompletionBlockReason>>,
    completed: bool,
}

fn zero_row(no: i64, date: &str) -> Vec<u8> {
    make_z004_bytes(no, date, &[(JANS[0], "空", 0, 0)])
}

impl TemporalWorld {
    fn new(late: bool) -> Self {
        let mut world = World::new();
        let held = if late {
            commit_bytes(world.conn_mut(), zero_row(1, "2026-09-13"), "s1").unwrap();
            commit_bytes(world.conn_mut(), zero_row(3, PREVIOUS_DAY), "s3").unwrap();
            world.next_no = 4;
            Some(Held {
                bytes: make_z004_bytes(2, "2026-09-14", &[(JANS[0], "遅い記録", 2, 198)]),
                quantity: 2,
            })
        } else {
            commit_bytes(world.conn_mut(), zero_row(1, PREVIOUS_DAY), "s1").unwrap();
            world.next_no = 2;
            None
        };
        let physical = INITIAL[0] - held.as_ref().map_or(0, |h| h.quantity);
        Self {
            world,
            physical,
            pending: 0,
            held,
            stocktake: None,
            zeroed: false,
            blocked: Vec::new(),
            completed: false,
        }
    }

    fn copy(&self) -> (i64, bool, Option<(i64, i64)>) {
        stock_copy(self.world.conn(), CODES[0])
    }

    fn item_id(&self, code: &str) -> i64 {
        self.world
            .conn()
            .query_row(
                "SELECT id FROM stocktake_items WHERE stocktake_id=?1 AND product_code=?2",
                (self.stocktake.unwrap(), code),
                |r| r.get(0),
            )
            .unwrap()
    }

    fn save(&mut self, fresh: bool) {
        let item = self.item_id(CODES[0]);
        stocktake_service::save_stock_count(
            self.world.conn_mut(),
            &stocktake_service::SaveStockCountRequest {
                product_code: CODES[0].to_string(),
                count_value: self.physical,
                fresh,
                business_date: DATE.to_string(),
                stocktake_item_id: Some(item),
            },
            DATE,
        )
        .unwrap();
    }

    fn act(&mut self, action: Temporal) -> Option<String> {
        match action {
            Temporal::Start => {
                self.stocktake = Some(
                    stocktake_service::start_stocktake(self.world.conn_mut())
                        .unwrap()
                        .stocktake_id,
                );
            }
            Temporal::Count => self.save(false),
            Temporal::Recount => self.save(true),
            Temporal::Move(operation) => {
                self.physical += match operation {
                    Operation::Receive => 3,
                    Operation::ManualSale => -2,
                    Operation::Dispose => -1,
                    Operation::Return => 1,
                    _ => panic!("not a physical-movement action"),
                };
                self.world.perform(operation);
            }
            Temporal::PosSale => {
                self.physical -= 2;
                self.pending += 2;
            }
            Temporal::Close => {
                let no = self.world.next_no;
                self.world.next_no += 1;
                let quantity = i32::try_from(self.pending).unwrap();
                let bytes = if quantity == 0 {
                    zero_row(no, DATE)
                } else {
                    make_z004_bytes(no, DATE, &[(JANS[0], "締め", quantity, quantity * 99)])
                };
                commit_bytes(self.world.conn_mut(), bytes, "close").unwrap();
                self.pending = 0;
            }
            Temporal::ImportHeld => {
                let held = self.held.take().expect("fixture must hold a settlement");
                commit_bytes(self.world.conn_mut(), held.bytes, "held").unwrap();
            }
            Temporal::Complete => {
                let stocktake_id = self.stocktake.unwrap();
                if !self.zeroed {
                    stocktake_service::zero_uncounted_items(
                        self.world.conn_mut(),
                        stocktake_id,
                        &[CODES[1].to_string()],
                        DATE,
                        DATE,
                    )
                    .unwrap();
                    self.zeroed = true;
                }
                let before = self.copy();
                let result = stocktake_service::complete_stocktake(
                    self.world.conn_mut(),
                    &stocktake_service::CompleteStocktakeRequest {
                        stocktake_id,
                        today: DATE.to_string(),
                    },
                );
                match result {
                    Ok(_) => {
                        self.completed = true;
                        let after = self.copy();
                        // (b) 完了は在庫を動かさず、写しは物理の数と同じ
                        if after != before || after.0 != self.physical || after.1 {
                            return Some(format!(
                                "(b) before={before:?} after={after:?} physical={}",
                                self.physical
                            ));
                        }
                    }
                    Err(BizError::StocktakeGuard { recovery, .. }) => self.blocked.push(
                        recovery
                            .targets
                            .iter()
                            .filter_map(|target| target.block_reason)
                            .collect(),
                    ),
                    Err(error) => panic!("fixture/BIZ completion error: {error:?}"),
                }
            }
        }
        // (a) 未取込みの売上が無い時点で、黙ってずれない
        if self.pending == 0 && self.held.is_none() {
            let (stock, recheck, range) = self.copy();
            let ok = match (recheck, range) {
                (true, Some((low, high))) => low <= self.physical && self.physical <= high,
                (false, None) => stock == self.physical,
                _ => false,
            };
            if !ok {
                return Some(format!(
                    "(a) copy={:?} physical={}",
                    self.copy(),
                    self.physical
                ));
            }
        }
        None
    }
}

/// trace を走らせ、completion_blocked の理由の集合の列を返す。oracle (a)(b) が破れたら Err
fn temporal_trace(
    late: bool,
    actions: &[Temporal],
) -> Result<Vec<Vec<CompletionBlockReason>>, String> {
    let mut world = TemporalWorld::new(late);
    for (index, action) in actions.iter().enumerate() {
        if let Some(failure) = world.act(*action) {
            let internal = integrity_service::run_integrity_check(world.world.conn()).unwrap();
            return Err(format!(
                "prefix={:?}; {failure}; internal mismatches={}",
                &actions[..=index],
                internal.mismatch_count
            ));
        }
    }
    assert!(
        world.completed,
        "fixture must complete the stocktake: {actions:?}"
    );
    Ok(world
        .blocked
        .into_iter()
        .map(|set| set.into_iter().collect())
        .collect())
}

#[test]
fn test_cross_feature_req205_controls() {
    // REQ-205 / REQ-401: XFA-D2（Matrix X3。期待を変える: 新しい完了は締めの精算を待ち、同じ日の記録で
    // 要再確認にする）。どの対照も oracle を破らない
    use CompletionBlockReason::{Recheck, SettlementPending};
    use Temporal::*;
    for (actions, expected) in [
        (vec![Start, Count, Close, Complete], vec![]),
        (
            vec![
                Start,
                Count,
                Move(Operation::ManualSale),
                Recount,
                Close,
                Complete,
            ],
            vec![],
        ),
        (
            vec![Start, PosSale, Count, Close, Complete, Recount, Complete],
            vec![vec![Recheck]],
        ),
        (
            vec![
                Start, Count, PosSale, Complete, Close, Complete, Recount, Complete,
            ],
            vec![vec![SettlementPending], vec![Recheck]],
        ),
    ] {
        assert_eq!(temporal_trace(false, &actions), Ok(expected), "{actions:?}");
    }
}

#[test]
fn test_cross_feature_req904_req203_model_detects_consistent_corruption() {
    // REQ-904 / REQ-203: XFA-D1,D3。棚卸し処理ではなく検査モデルの健全性を確認する。
    let world = World::new();
    world.check();
    world
        .conn()
        .execute(
            "UPDATE products SET stock_quantity=stock_quantity+1 WHERE product_code=?1",
            [CODES[0]],
        )
        .unwrap();
    inventory_repo::insert_movement(
        world.conn(),
        &NewMovement {
            product_code: CODES[0].into(),
            movement_type: MovementType::Receiving,
            quantity: 1,
            stock_after: INITIAL[0] + 1,
            reference_type: None,
            reference_id: None,
            note: Some("合成mutation".into()),
            business_date: "2026-04-01".to_string(),
            settlement_no: None,
        },
    )
    .unwrap();
    assert_eq!(
        integrity_service::run_integrity_check(world.conn())
            .unwrap()
            .mismatch_count,
        0
    );
    assert!(
        world.mismatch().is_some(),
        "independent model must reject coherent wrong data"
    );

    let mut world = World::new();
    world.perform(Operation::ManualSale);
    world.check();
    let before = sales_service::get_daily_sales(world.conn(), DATE).unwrap();
    world
        .conn()
        .execute(
            "UPDATE sale_records SET source='auto' WHERE source='manual'",
            [],
        )
        .unwrap();
    let after = sales_service::get_daily_sales(world.conn(), DATE).unwrap();
    assert_eq!(
        (before.grand_total.count_points, before.grand_total.amount),
        (after.grand_total.count_points, after.grand_total.amount)
    );
    assert_eq!(
        (before.grand_total.length_cm, after.grand_total.length_cm),
        (0, 0)
    );
    assert!(
        world.mismatch().is_some(),
        "source mismatch must fail even when totals agree"
    );
}

#[test]
fn test_cross_feature_req205_count_then_movement_is_kept() {
    // REQ-205 / REQ-201 / REQ-202 / REQ-203 / REQ-204 / REQ-401: XFA-D2,D4（STK-1。Matrix X1、
    // 停止 ADR SPEC-STOP-D3 の診断を新しい経路で PASS する回帰 test にした）。数えた後の入出庫が完了で
    // 打ち消されない。同じ日の記録は決まらないので 1 回目の完了は要再確認で止まり、入れ直しの後に通る
    use CompletionBlockReason::Recheck;
    use Temporal::*;
    for operation in [
        Operation::Receive,
        Operation::ManualSale,
        Operation::Dispose,
        Operation::Return,
    ] {
        let actions = [
            Start,
            Count,
            Move(operation),
            Close,
            Complete,
            Recount,
            Complete,
        ];
        assert_eq!(
            temporal_trace(false, &actions),
            Ok(vec![vec![Recheck]]),
            "{actions:?}"
        );
    }
    let actions = [Start, Count, PosSale, Close, Complete, Recount, Complete];
    assert_eq!(
        temporal_trace(false, &actions),
        Ok(vec![vec![Recheck]]),
        "{actions:?}"
    );
}

#[test]
fn test_cross_feature_req205_req401_late_import_not_double_counted() {
    // REQ-205 / REQ-401: XFA-D2,D4（STK-2。Matrix X2）。完了の後に届いた過去の精算が二重に減らない。
    // 精算 1（D−3）・3（D−1）を取り込み、精算 2（D−2、SKU0 × 2）を保留した World。欠けた 2 の隣の精算日が
    // 計数の日付 D を挟まないので完了は止まらず、精算 2 は計数の日付より前なので計数に入る
    use Temporal::*;
    let actions = [Start, Count, Close, Complete, ImportHeld];
    assert_eq!(temporal_trace(true, &actions), Ok(vec![]));

    // 完了の後の取込みで完了の評価額を書き換えない
    let mut world = TemporalWorld::new(true);
    for action in [Start, Count, Close, Complete] {
        assert_eq!(world.act(action), None);
    }
    let stocktake_id = world.stocktake.unwrap();
    let total = |world: &TemporalWorld| {
        stocktake_service::get_stocktake_record(world.world.conn(), stocktake_id)
            .unwrap()
            .total_cost
    };
    let before = total(&world);
    assert_eq!(world.act(ImportHeld), None);
    assert_eq!(world.copy(), (8, false, None));
    assert_eq!(total(&world), before);
}
