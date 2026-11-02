## 12. BIZ-02: 在庫変動ロジック

### 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1〜D3）の BIZ の中心。旧 ADR（時点証拠）の版（`stock_revision`）の契約は置き換えた。(a)（ADR SPEC-STK-CNT-D12）で入ったのは、`biz::stock_ledger`（`derive_stock`・`recompute_stock`。要再確認の理由は `same_date_movement`・`conflicting_counts` の 2 つ）、`apply_stock_change` の並びの鍵と `stock_after` の書き方、入出庫 4 業務の `stock_after`、業務日の形と暦の検査（`biz::inventory_service::common::validate_business_date`）、(a) の求め直しの範囲（下の該当の項目）。精算の窓（「精算 k」の行・理由 1〈`undetermined_settlement`〉・取り込み済みの精算の一覧の読取り）、求め直す商品の横断の表の Z004 の行、Z004 の新しい確定と取消は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で入った（Z004 の movement は `phase = 1`、業務日 = 精算日）。

- 求め直しの関数（新しい module `biz::stock_ledger`。シグネチャは下の「求め直しの関数のシグネチャ」）: `derive_stock`（読取りだけ。整合性の検査〈[36](36-biz-integrity-check.md)〉と完了の後の差〈[35](35-biz-stocktake-service.md)〉が使う）と、`recompute_stock`（`derive_stock` と同じ規則で求め、写し〈`stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`〉を同じ TX で書く）。`DerivedStock` は `stock`（写しの値）、`range: Option<(i64, i64)>`（要再確認のときの最小と最大）、`recheck_reasons: Vec<RecheckReason>`（`undetermined_settlement` / `same_date_movement` / `conflicting_counts`。ADR D3 の理由 1〜3。(a) の enum は理由 2・3 の 2 つで、b1 が `undetermined_settlement` を足した）。中で、その商品の計数（`stock_counts`）と `is_voided=0` の movement と取り込み済みの精算の一覧（`csv_imports` の active の取込みID・`settlement_date`・`settlement_no`。取込みID は fresh な計数の判定の n を取込みのカーソルと比べて求めるのに使う。ADR D3）を読み、ADR D3 の窓の判定表で、効く計数・それに入っていない記録の和・決まらない記録・範囲を求める。写しを書くのは `recompute_stock` だけ（[21](21-io-inventory-repo.md) の `update_stock_quantity` を呼ぶのはこの関数 1 つ）。`phase=1` の movement（Z004）は、その `settlement_no` の精算として ADR D3 の判定表の「精算 k」の行で判定する（入出庫の行で判定しない）。
- 求め直す商品は ADR の横断の表の「求め直す商品」の列（D2 の原理: 求め直しの入力が変わる TX は、その入力に依る全ての商品を求め直す）。Z004 の確定・取消は、計数を持つ全ての商品と、その取込みの売上か movement を持つ全ての商品を同じ TX で求め直す（[32](32-biz-csv-import-service.md)）。
- 決まらない精算の行の判定（ADR D3 の理由 1）は、その精算の取込みに、その商品の有効な Z004 の movement があるか、`sale_records`（`csv_import_id` 付き、`is_voided=0`、`stock_linked=1`）にその商品の行が個数 ≠ 0 または金額 ≠ 0 であること。今の `pos_stock_sync` を見ない（連動を切っても、連動中に作った movement と行で理由と範囲が残る。ADR D8）。
- `apply_stock_change` は「保存した数 + 差分」をやめ、movement を並びの鍵付きで INSERT してから、その商品を `recompute_stock` で求め直す。署名に並びの鍵（`business_date` と、Z004 のときだけ `settlement_no: Option<i64>`）を足す（(a)。b1 で Z004 の確定が `apply_stock_change` を使わなくなったので、`settlement_no` の引数は b1 で外し、呼ぶのは入出庫 4 業務だけ）。入庫・返品交換（レジ未処理）・手動販売・廃棄の 4 つの `create_*` は header の日付と `None` を渡し（`phase=0`。header の日付の意味は品が動いた日〈入庫は品が届いた日で、納品書の日付ではない〉で、ADR D3 の入出庫の窓はこの日付に依る）、Z004 の確定は `apply_stock_change` を行ごとに呼ばず（b1 で `src-tauri/src/biz/csv_import_service/commit.rs` の `execute_commit` の `apply_stock_change` の呼出しを置き換えた）、在庫連動の行の movement を全部、仮の `stock_after` で `insert_movement`（[21](21-io-inventory-repo.md)）に精算日と精算回数を渡して書き（`phase=1`、`reference_type='csv_import'`。tracking の CHECK で揃う）、全ての行を書いた後に、横断の表の商品をまとめて `recompute_stock` で求め直し、商品ごとにその TX で書いた movement の `stock_after` を `update_movement_stock_after` で写しの値にする（32）。`stock_after` は、同じ TX で求め直した写しの値を、その TX で書いた新しい movement に書く。順は、movement を仮の `stock_after`（NOT NULL のため）で INSERT し、`recompute_stock` の後に同じ TX でその TX の新しい movement の `stock_after` だけを UPDATE する（Z004 の同じ商品の複数の行も同じ値。後入力で前に入った記録があっても既存の movement は書き換えない）。在庫の上限の検査（`999999999`）は `recompute_stock` が写しを書く直前の 1 か所に置く（求めた値と範囲の両端に当てる）。
- 入出庫 4 業務の `stock_after`: `apply_stock_change` は呼出しごとに、その呼出しが INSERT した movement（id で指す）の `stock_after` だけを、その直後の `recompute_stock` の写しの値で書く。同じ要求に同じ商品の明細が複数あれば、今と同じく明細の順の途中の値になる（計数が無い間は値が今と同じ。ADR SPEC-STK-CNT-D12 の (a)）。
- 業務日の形と暦の検査（D-118 の runtime の (a) で足した `validate_business_date`。業務日を書く側の検査の唯一の置き場。移行が既存の記録の日付に当てる検査は [22 §19](22-mnt-migration.md) の手順 2）: `biz` の共有の関数 1 つで、(1) 形: 長さ 10 バイトの ASCII で、5・8 文字目が `-`、ほかの 8 文字が数字（`[0-9]{4}-[0-9]{2}-[0-9]{2}`）。合わなければ「日付の形式が不正です（YYYY-MM-DD）」。(2) 暦: 形の合う文字列を `chrono::NaiveDate::parse_from_str(_, "%Y-%m-%d")` で読み、読めなければ「存在しない日付です」。どちらも `ValidationFailed`（文は今の `sales_service.rs` の `validate_date` と同じ）。形を先に見るのは、chrono の `%Y-%m-%d` が `2026-4-7`・` 2026-04-07`・`+2026-04-07` を読み、`-0001-04-07`・`+10000-04-07` は読んで同じ文字列に書き戻すので、読めることも書き戻しの一致も 10 文字の形を保証しないため（形の合う文字列では、読めることと SQLite の `date(d) IS d` が一致し、読んだ日付の書き戻しは入力と同じ。packet の Contract Probe P6・P7）。入庫・返品交換・手動販売・廃棄の 4 つの `create_*` と `sales_service` の日付の検査がこの関数を使う。4 つの `create_*` は header の日付（`receiving_date`・`return_date`・`sale_date`・`disposal_date`）を、DB を見ない検証の段（冪等の再送の判定と TX より前。返品交換の今の `validate_return_request_shape` と同じ段）で確かめ、外れたら何も書かない。返品交換は `register_processed` で在庫を動かすかが分かれる前に確かめるので、movement を作らないレジ処理済みの返品・交換の保存も同じ検査を通る。窓の判定（ADR D3）は日付を文字列の大小で比べるので、形の違う日付を黙って通さない（DB の CHECK は形だけを見る。[tracking](../db-design/tracking-system-tables.md)）。初期在庫の movement の業務日は、商品を登録した TX の PC の今日（`chrono::Local` を `%Y-%m-%d` で書く）。
- 求め直しは、計数・入出庫・初期在庫の窓（ADR D3 の判定表の「入出庫の movement」「初期在庫の movement」の行と「計数どうし」、要再確認の理由 2・3）と、b1 で足した精算の窓（「精算 k」の行・理由 1・取り込み済みの精算の一覧の読取り）を持つ。計数どうしの前後が循環して効く計数が決まらないときは、和へ倒さずに既存の `BizError::DatabaseError`（`DbError::QueryFailed` に「計数の前後を決められません」の文）で止まり、写しを書かない。循環しないことは既定の計数の日付が保存の日以前であることが前提で、その前提は計数の保存の検査（[35](35-biz-stocktake-service.md)）が守る。
- 冪等な要求の再送は新しい movement を作らない（今のまま）。レジ処理済みの返品は movement を作らない（今のまま）。
- list_inventory_records の棚卸し差異の集計は [65](65-inventory-record-traceability.md) の新契約に従う（計数は movement を作らない）。

求め直しの関数のシグネチャ（D-118 の runtime の (a)。crate の外から呼ばないので `pub(crate)`。`recompute_stock` は `apply_stock_change` と同じく第 1 引数を借りた transaction にし、TX の内側に限ることを型で強制する。`src-tauri/tests/design_compliance_test.rs` の allowlist には足さない）:

```rust
pub(crate) fn derive_stock(conn: &DbConnection, product_code: &str) -> Result<DerivedStock, BizError>
pub(crate) fn recompute_stock(tx: &rusqlite::Transaction<'_>, product_code: &str) -> Result<DerivedStock, BizError>
```

### 単位と原価の精度の契約（D-113）

[共通規則](10-common-rules.md) SPEC-UNIT-D2・D5・D6・D11 の BIZ-02 の側。単位・上限・行の金額・原価の 1/100 円は現行の契約。

- 入庫・返品・手動販売・廃棄の Request の `quantity` は今どおり在庫の数量の正の整数（長さの商品は cm）。m の入力は UI が cm に直して送る（SPEC-UNIT-D3）。BIZ の検証（INV-1a）は上限を持つ（[共通規則](10-common-rules.md) の「入力の上限と安全な整数の範囲」）: 1 明細の数量は 1 以上 `9999999`。`quantity <= 0` の検証の隣（`src-tauri/src/biz/inventory_service/receiving.rs` の `create_receiving`・`src-tauri/src/biz/inventory_service/manual_sale.rs` の `create_manual_sale`・`src-tauri/src/biz/inventory_service/disposal.rs` の `create_disposal`、返品は `src-tauri/src/biz/inventory_service/returns.rs` の `validate_return_request_shape`）で、在庫を動かすかどうかの分岐より前に全明細を検査する（レジ処理済みの返品〈`register_processed`〉の明細は `apply_stock_change` を通らずに `return_items` へ保存される: `src-tauri/src/biz/inventory_service/returns.rs` の `create_return` の `if !req.register_processed` の分岐）。越えれば `ValidationFailed`（`明細{n}: 数量が大きすぎます`）で何も書かない。`apply_stock_change` は、計算の後の `stock_after` の絶対値が在庫の上限 `999999999` を越えれば、書く前に `ValidationFailed`（`在庫数が大きすぎます`）で TX 全体を戻す（`recompute_stock` が写しを書く直前の検査。上の D-118 の節）。`create_receiving`・`create_disposal` は、保存の前に明細から行の金額（`cost_line_centi`）と合計（`cost_total_yen`）を求めて `check_safe_integer`（`±9007199254740991`）に通し、範囲外は `ValidationFailed`（`合計金額が大きすぎます`）で TX 全体を戻す（在庫・記録・明細・変動・操作ログを書かない。`src-tauri/src/biz/inventory_service/receiving.rs`・`src-tauri/src/biz/inventory_service/disposal.rs` の `check_safe_integer` の呼出し）。保存した明細は書き換えないので、記録詳細（§12.6a）が同じ明細から求める合計は検査した値と同じになり、記録詳細の読取りに範囲の検査を置かない。
- 入庫・廃棄の明細の原価は `cost_price_centi`（1/100 円、`決定（owner、D-113 J2）`。原価の lane、D-128）。BIZ は明細の原価も 0 以上・`99999999`（`999999.99` 円。`MAX_COST_PRICE_CENTI`）以下を負の検査の隣（`src-tauri/src/biz/inventory_service/receiving.rs` の `create_receiving`・`src-tauri/src/biz/inventory_service/disposal.rs` の `create_disposal` の `MAX_COST_PRICE_CENTI` の検査）で検査し、超えれば `ValidationFailed("原価が大きすぎます")`（SPEC-UNIT-D5。UI を通らない IPC でも上限を超える値を保存しない）。§12.3 の原価の差分（CostDiff）は 1/100 円の値で完全一致を比べる。§12.8 の fingerprint の item 行の原価は 1/100 円の 10 進文字列にする。原価の 1/100 円は開発・試験 DB を作り直した起点で入れる（D-119、[22](22-mnt-migration.md) §16 の vB）ので、円の原価で作った fingerprint の記録はその DB に無い。
- §12.6a の入庫・廃棄の記録詳細は、明細の数量・原価・商品の単位から行の金額と合計を BIZ が求める（SPEC-UNIT-D6。`biz::unit_amount`）。IO は金額を計算しない（[21](21-io-inventory-repo.md) §10.2・§10.5 の手順 3）。wire は行の金額を 1/100 円の整数の十進の文字列（`line_cost_centi`・`line_loss_cost_centi`。i128 の値を損失なく運ぶ。SPEC-UNIT-D6）、合計を円の整数（`total_cost`・`total_loss_cost`、今の型のまま）で返す。入庫の丸めは TD-023 と同じ（`決定（owner、D-113 J3）`）、廃棄も TD-023 に合わせる。
- 記録詳細と入出庫の行の `stock_unit` は `ProductStockUnit`（SPEC-UNIT-D11）。
- 入庫・返品・手動販売・廃棄の保存の結果の在庫警告 `stock_warnings`（`{商品コード}: 在庫がマイナスになりました（{stock_after}）`。`src-tauri/src/biz/inventory_service/` の `receiving.rs`・`returns.rs`・`manual_sale.rs`・`disposal.rs` の `create_*` の 4 か所が同じ関数〈`negative_stock_warning_message`〉で書く）は、長さの商品（`m`）の `stock_after` を SPEC-UNIT-D3 の m の形に整形して入れる（-30 → `（-0.3 m）`）。個数の商品の文は `（-2）`。整形は TS の formatter と同じ規則の Rust の関数 1 つ（`format_length_m`）で行い、golden の表を共有する（SPEC-UNIT-D9）。owner の決定 TD-203（SPEC-UNIT-D3 の表示する所の表の 10）。

### 12.1 モジュール構成

```
src-tauri/src/
  biz/
    mod.rs                -- pub mod inventory_service を追加
    product_service.rs    -- 既存（BIZ-01）
    inventory_service.rs  -- 在庫変動の業務ロジック（本セクション）
```

### 12.2 apply_stock_change（共通在庫変動 — 内部関数）

**関数要求**: 商品の在庫数を変動させ、inventory_movements に履歴を記録する。全ての入出庫処理から呼ばれる共通内部関数

**シグネチャ**:
```
fn apply_stock_change(      // pub(crate)
    conn: &Transaction<'_>, // 借りた transaction（rusqlite::Transaction）。通常の接続は渡せない
    product_code: &str,
    business_date: &str,    // 並びの鍵の業務日（YYYY-MM-DD。形と暦は呼出し元が validate_business_date で検査済み）
    quantity: i64,          // 在庫視点: +増加 / -減少（BIZ層が符号変換済み）
    movement_type: MovementType,
    reference_type: ReferenceType,
    reference_id: i64,
    note: Option<&str>,
) -> Result<StockChangeOutcome, BizError>
```

**StockChangeOutcome構造体**:
- stock_after: i64（求め直した写しの値）
- negative_stock_warning: bool（stock_after < 0 なら true）
- stock_unit: ProductStockUnit（商品の単位。在庫警告の文の整形に使う）

**前提条件**: この関数は呼び出し元のトランザクション内で実行される。第 1 引数を借りた transaction（`&rusqlite::Transaction<'_>`）にし、TX の内側に限ることを型で強制する（通常の接続を渡すコードはコンパイルできない。repo 関数へは `Deref` で `&DbConnection` として渡る）。自身ではトランザクションを開始しない。inventory_movements の INSERT・写しの求め直し（`recompute_stock`）・その movement の `stock_after` の UPDATE は同一TX内で常にセットで実行される。いずれかが失敗した場合、呼び出し元のTX全体がROLLBACKされる（rusqlite::Transaction RAII による自動ROLLBACK）

**処理ステップ**:
1. product_repo::find_by_product_code(conn, product_code) → product
   - None → BizError::NotFound("商品が見つかりません: {product_code}")
2. inventory_repo::insert_movement(conn, &NewMovement { product_code, movement_type, quantity, stock_after: 今の写しの stock_quantity（仮の値。NOT NULL のため）, reference_type: Some(reference_type), reference_id: Some(reference_id), note, business_date, settlement_no }) → movement_id
3. biz::stock_ledger::recompute_stock(conn, product_code) → stock_after = 求め直した写しの値（写しを書く。在庫の上限の外は書く前に `ValidationFailed("在庫数が大きすぎます")` で TX 全体を戻す）
4. [21](21-io-inventory-repo.md) の `update_movement_stock_after(conn, movement_id, stock_after)`（この呼出しが INSERT した movement だけ）
5. StockChangeOutcome { stock_after, negative_stock_warning: stock_after < 0, stock_unit: product.stock_unit } を返す

---

### 12.3 create_receiving（入庫記録）

**関数要求**: 入庫記録ヘッダと明細を登録し、各明細について在庫を増加させる

**シグネチャ**:
```
fn create_receiving(conn: &mut DbConnection, req: ReceivingCreateRequest) -> Result<ReceivingCreateResult, BizError>
```

**ReceivingCreateRequest構造体**:
- idempotency_key: String（UI/caller が1保存試行単位の安定キーとして生成し、CMD層は中継する。BIZ層は空文字・長さ制限・重複時 fingerprint 一致を検証する）
- supplier_id: Option<i64>
- receiving_date: String（YYYY-MM-DD）
- note: Option<String>
- items: Vec<ReceivingItemInput>

**ReceivingItemInput構造体**:
- product_code: String
- quantity: i64（正の整数のみ）
- cost_price_centi: i64

**ReceivingCreateResult構造体**:
- record_id: i64
- created: bool
- idempotent_replay: bool
- stock_warnings: Vec<String>
- cost_diffs: Vec<CostDiff>（SPEC-PRV-D8 / REQ-209）

**CostDiff構造体**:
- product_code: String
- product_name: String
- master_cost_price_centi: i64
- received_cost_price_centi: i64

**不変条件**: created=true ↔ idempotent_replay=false、created=false ↔ idempotent_replay=true。この2組み合わせ以外は生成しない

**処理ステップ**:
0. **業務日の検査（DB を見ない検証の段。冪等性チェックと TX より前）**: `validate_business_date(&req.receiving_date)`。外れたら `ValidationFailed` で何も書かない（上の「業務日の形と暦の検査」）
1. **冪等性チェック**
   - inventory_repo::find_receiving_by_idempotency_key(conn, &req.idempotency_key)
   - Some((existing_id, existing_fp)) の場合:
     - request_fingerprint を計算し、existing_fp と比較
     - 一致 → ReceivingCreateResult { record_id: existing_id, created: false, idempotent_replay: true, stock_warnings: [], cost_diffs: [] } を返す
     - 不一致 → BizError::IdempotencyConflict("同じ冪等キーで異なる内容のリクエストです")
2. **バリデーション（TX外）**
   a. items が空 → BizError::ValidationFailed("明細が1件以上必要です")
   b. 各item: quantity <= 0 → ValidationFailed、cost_price_centi < 0 → ValidationFailed、product_codeの存在チェック
   c. supplier_idがSome → 存在チェック
3. **request_fingerprint を計算**
4. **トランザクション開始**（conn.transaction()）
5. inventory_repo::insert_receiving_record(&tx, record) → record_id
6. 各item: insert_receiving_item + apply_stock_change(receiving_date, None, +quantity, Receiving, ReceivingRecord, record_id)
7. system_repo::insert_operation_log(operation_type="receiving_create")
8. **COMMIT**
9. ReceivingCreateResult の既存 field（record_id / created=true / idempotent_replay=false / stock_warnings）を構成する
10. **COMMIT 後の原価差分検出（SPEC-PRV-D8）**
   - 保存 transaction とは別の読取りで、各明細の `cost_price_centi` と現在の `products.cost_price_centi` を完全一致比較する
   - 不一致を `CostDiff { product_code, product_name, master_cost_price_centi, received_cost_price_centi }` として `cost_diffs` に積み、ReceivingCreateResult として返す
   - 差分検出は **保存 transaction の成否に影響しない**。検出失敗を理由に保存済み入庫を rollback しない
   - idempotent replay 時は step 1 で `cost_diffs: []` を返し、再提示しない

**request_fingerprint 正規化**:
- ヘッダ行: "{supplier_id}|{receiving_date}"
- item行: "{product_code}|{quantity}|{cost_price_centi}"
- item行を行文字列全体の辞書順ASCでソート
- ヘッダ行 + item行を "\n" で結合 → SHA-256 hex digest（小文字64文字）
- 型の正規化: 整数→10進文字列（先頭ゼロなし）、Option None→"null"、日付→YYYY-MM-DD（10文字固定）

---

### 12.4 create_return（返品・交換記録）

**関数要求**: 返品・交換記録を登録する。register_processed フラグに基づき在庫を動かすかどうかを分岐する

**シグネチャ**:
```
fn create_return(conn: &mut DbConnection, req: ReturnCreateRequest) -> Result<ReturnCreateResult, BizError>
```

**ReturnCreateRequest構造体**:
- idempotency_key: String
- return_type: ReturnExchangeType（"return" / "exchange"）
- return_date: String（YYYY-MM-DD）
- register_processed: bool
- receipt_image_path: Option<String>
- note: Option<String>
- items: Vec<ReturnItemInput>

**ReturnItemInput構造体**:
- product_code: String
- direction: ReturnDirection（"in" / "out"）
- quantity: i64（正の整数のみ）

**ReturnCreateResult構造体**:
- record_id: i64
- created: bool
- idempotent_replay: bool
- stock_warnings: Vec<String>

**処理ステップ**:
0. **業務日の検査（DB を見ない検証の段。冪等性チェックと TX より前）**: `validate_return_request_shape` の中で `validate_business_date(&req.return_date)`。register_processed の分岐より前なので、movement を作らないレジ処理済みの返品・交換も同じ検査を通る
1. **冪等性チェック**（create_receivingと同パターン）
2. **バリデーション**: items空チェック、return_type と direction の組み合わせ検証、各item の quantity/product_code検証（値域は enum 型で検査済み）
   - return_type == "return": direction は全itemで "in" のみ許可する。1件以上の "in" item が必要
   - return_type == "exchange": "in" item と "out" item を少なくとも1件ずつ要求する
   - 上記に違反した場合は BizError::ValidationFailed を返し、insert_return_record / insert_return_item / stock change は実行しない
3. **request_fingerprint計算**
   - ヘッダ行: "{return_type}|{return_date}|{register_processed}"
   - item行: "{product_code}|{quantity}|{direction}"
4. **TX開始**
5. insert_return_record → record_id
6. 各item:
   - insert_return_item
   - register_processed == true → 在庫は動かさない（CSV取込みで自動反映されるため）
   - register_processed == false:
     - direction == "in" → apply_stock_change(return_date, None, +quantity, Return, ReturnRecord, record_id)
     - direction == "out" → apply_stock_change(return_date, None, -quantity, Return, ReturnRecord, record_id)
7. **操作ログ**
8. **COMMIT**

**設計判断 — register_processed分岐**:
- register_processed=1: レジで「戻し」操作済み。次回CSV精算データに反映される。BIZ-02で在庫を動かすと二重計上
- register_processed=0: レジ未処理。CSVに反映されないためここで在庫を動かす

**設計判断 — return_type と direction の業務不変条件**:
- 返品（return）は商品が戻る記録なので、明細 direction は "in" のみ許可する。"out" を含む返品は帳面上の意味が崩れるため BIZ-02 で拒否する
- 交換（exchange）は「戻った商品」と「渡した商品」の対を記録するため、"in" と "out" の両方を少なくとも1件ずつ要求する。片側だけなら返品または別の出庫種別として扱う
- UI-03 は保存前に日本語エラーを出すが、generated command はUI以外からも呼べるため、最終防御は BIZ-02 が持つ

**enum 契約化（D-061 / D-064）**: `return_type` / `direction` は generated enum 型で検査し、文字列比較 validation は退役した。正常値と DB CHECK 防御は不変。

---

### 12.5 create_manual_sale（手動販売出庫）

**関数要求**: レジを通さない販売を手動記録する。PLU登録済み商品への警告チェック（token方式）を含む

**シグネチャ**:
```
fn create_manual_sale(conn: &mut DbConnection, req: ManualSaleCreateRequest) -> Result<ManualSaleCreateResult, BizError>
```

**ManualSaleCreateRequest構造体**:
- idempotency_key: String
- sale_date: String（YYYY-MM-DD）
- reason: ManualSaleReason（"plu_unregistered" / "other"）
- note: Option<String>
- items: Vec<ManualSaleItemInput>
- confirmation_token: Option<String>（初回はNone、警告確認後に再呼出時はSome）

**ManualSaleItemInput構造体**:
- product_code: String
- quantity: i64（正の整数のみ）
- amount: i64

**ManualSaleCreateResult構造体**:
- sale_id: Option<i64>（needs_confirmation=true の場合は None）
- created: bool
- idempotent_replay: bool
- plu_warnings: Vec<String>
- stock_warnings: Vec<String>
- needs_confirmation: bool
- confirmation_token: Option<String>（警告時にtoken生成して返却）

**処理ステップ**:
0. **業務日の検査（DB を見ない検証の段。冪等性チェックと TX より前）**: `validate_business_date(&req.sale_date)`
1. **冪等性チェック**（同パターン）
2. **バリデーション**: items空チェック、reason検証、各item の quantity/amount/product_code検証
3. **PLU登録済みチェック**:
   - 各item の product: plu_dirty == false かつ plu_exported_at IS NOT NULL → plu_warnings に追加
   - plu_warnings が非空:
     - confirmation_token == None → token生成して即リターン（needs_confirmation=true）
     - confirmation_token == Some(token) → ハッシュ再計算して一致確認。不一致→ValidationFailed
4. **request_fingerprint計算**
   - ヘッダ行: "{sale_date}|{reason}"
   - item行: "{product_code}|{quantity}|{amount}"
5. **TX開始**
6. insert_manual_sale → sale_id
7. 各item:
   - insert_manual_sale_item
   - sales_repo::insert_sale_record(source="manual", quantity=+item.quantity[売上帳票視点])
   - apply_stock_change(sale_date, None, -quantity[在庫視点], SaleManual, ManualSale, sale_id)
8. **操作ログ**(operation_type="manual_sale_create")
9. **COMMIT**
10. ManualSaleCreateResult { sale_id, created: true, idempotent_replay: false, ... needs_confirmation: false }

**PLU warning_token 方式**:
- ハッシュ対象: product_code + plu_dirty + plu_exported_at + quantity + amount を正規化ソート → SHA-256
- plu_exported_at を含める（安全優先。PLU書出しと手動販売の同時進行はほぼないためUXコスト極小）

**設計判断 — sale_records.quantity の符号**:
INV-1に従い、sale_records.quantity は売上帳票視点で正の値。inventory_movements.quantity は在庫視点で負の値。符号変換は create_manual_sale 内で行う

---

### 12.6 create_disposal（廃棄・破損記録）

**関数要求**: 廃棄・破損記録を登録し、在庫を減少させる

**シグネチャ**:
```
fn create_disposal(conn: &mut DbConnection, req: DisposalCreateRequest) -> Result<DisposalCreateResult, BizError>
```

**DisposalCreateRequest構造体**:
- idempotency_key: String
- disposal_date: String（YYYY-MM-DD）
- items: Vec<DisposalItemInput>

**DisposalItemInput構造体**:
- product_code: String
- disposal_type: DisposalType（"disposal" / "damage" / "other"）
- quantity: i64（正の整数のみ）
- cost_price_centi: i64
- reason: String

**DisposalCreateResult構造体**:
- record_id: i64
- created: bool
- idempotent_replay: bool
- stock_warnings: Vec<String>

**処理ステップ**:
0. **業務日の検査（DB を見ない検証の段。冪等性チェックと TX より前）**: `validate_business_date(&req.disposal_date)`
1. **冪等性チェック**（同パターン）
2. **バリデーション**: items空、各item の quantity/cost_price_centi/reason/product_code検証（disposal_type 値域は enum 型で検査済み）
3. **request_fingerprint計算**
   - ヘッダ行: "{disposal_date}"
   - item行: "{product_code}|{quantity}|{cost_price_centi}|{disposal_type}|{reason}"
4. **TX開始**
5. insert_disposal_record → record_id
6. 各item: insert_disposal_item + apply_stock_change(disposal_date, None, -quantity, Disposal, DisposalRecord, record_id)
7. **操作ログ**(operation_type="disposal_create")
8. **COMMIT**

**enum 契約化（D-061 / D-064）**: `disposal_type` は generated enum 型で検査し、文字列比較 validation は退役した。正常値と DB CHECK 防御は不変。

---

### 12.6a 業務記録詳細 read 関数

**関数要求**: 入出庫履歴・在庫変動追跡の詳細画面用に、業務記録ヘッダ、明細、関連 `inventory_movements` を読み取り専用で返す。BIZ 層は IO 層の detail に対して `MovementRecord.source` を補完し、存在しない記録は `BizError::NotFound` に変換する。

**シグネチャ**:
```
fn get_receiving_record(conn: &DbConnection, record_id: i64) -> Result<ReceivingRecordDetail, BizError>
fn get_return_record(conn: &DbConnection, record_id: i64) -> Result<ReturnRecordDetail, BizError>
fn get_manual_sale_record(conn: &DbConnection, record_id: i64) -> Result<ManualSaleRecordDetail, BizError>
fn get_disposal_record(conn: &DbConnection, record_id: i64) -> Result<DisposalRecordDetail, BizError>
```

**処理ステップ**:
1. 対応する IO detail 関数を呼ぶ
2. IO 層の NotFound は記録種別名を含む `BizError::NotFound` に変換する
3. その他の IO エラーは `BizError::DatabaseError` に変換する
4. detail.movements の各行に対して `reference_type/reference_id` から `MovementSourceLink` を補完する

**設計判断 — read-only の責務分離**:
詳細取得は過去記録の確認用途であり、取消/訂正、CSV出力、印刷、画像 asset 表示は別 slice に分ける。CMD 層は read command を thin wrapper とし、表示用集計（原価合計、販売金額合計）は IO/BIZ の DTO に入れて UI で再計算しない。

---

### 12.7 エラーハンドリング（BizError 追加）

```
enum BizError {
    ValidationFailed(String),
    NotFound(String),
    DuplicateProductCode(String),
    DatabaseError(DbError),
    ImportError(String),
    IdempotencyConflict(String),  // ← 新規追加
}
```

**IdempotencyConflict**: 同じ冪等キーで異なる内容のリクエスト。CMD層では CmdError { kind: "idempotency_conflict" } に変換

---

### 12.8 request_fingerprint 正規化仕様

**原則**: 業務結果に影響するフィールドのみ含める。備考・画像パス等の補足情報は除外

**関数別フィールド一覧**:

| 関数 | ヘッダ（含める） | ヘッダ（除外） | item（含める） | item（除外） |
|------|----------------|---------------|---------------|-------------|
| create_receiving | supplier_id, receiving_date | note | product_code, quantity, cost_price_centi | — |
| create_return | return_type, return_date, register_processed | receipt_image_path, note | product_code, direction, quantity | — |
| create_manual_sale | sale_date, reason | note | product_code, quantity, amount | — |
| create_disposal | disposal_date | — | product_code, disposal_type, quantity, cost_price_centi, reason | — |

**正規化手順**:
1. ヘッダ固有値を先頭行に生成
2. 各itemの行文字列を生成
3. item行を行文字列全体の辞書順ASCでソート（同一product_codeでも安定）
4. ヘッダ行 + ソート済みitem行を "\n" で結合
5. SHA-256 hex digest（小文字64文字）

**型の正規化ルール**:
- 整数: 10進文字列、符号付き、先頭ゼロなし（"500", "-3"）
- Option<T>: None → "null", Some(v) → vの文字列表現
- bool: "true" / "false"
- 日付: YYYY-MM-DD（10文字固定）。時刻部分があれば切り捨て
- 文字列: UTF-8そのまま

---

### 12.9 領収書画像保存の validation 所有（D-060 (c)、順12）

領収書画像の拡張子 validation は BIZ 層（inventory_service）が所有する（D-060）。`receipt_image_path` は返品記録（§12.4）の要素であり、画像保存は returns domain に属する。

- 責務分担: CMD（`save_receipt_image`、function-design 43 §43.10）は Base64 decode の wire 型変換のみを行い、decode 済みバイト列と拡張子を BIZ へ渡す。BIZ が拡張子（`jpg|jpeg|png|gif|webp`）を検証して `BizError::ValidationFailedAt` を返し、通過後に `io::image_manager`（IO-06）を呼ぶ
- IO 層（28-io-image-manager.md）の拡張子 check は防御として現状維持する（DB CHECK 制約と同型。正常系で到達しない。ARCH-VAL-D1 は CMD/BIZ 間の二重判定禁止であり、IO 防御は対象外）
- wire 契約（kind / message / field）は移行前と不変

**シグネチャ**:
```rust
pub fn save_receipt_image(
    app_data_dir: &Path,
    image_bytes: &[u8],
    extension: &str,
) -> Result<String, BizError>
```

**処理ステップ**:
1. `extension` を小文字化し、`jpg|jpeg|png|gif|webp` のいずれかであることを検証する。
2. 検証通過後、元の `extension` と decode 済み `image_bytes` を `io::image_manager::save_receipt_image` へ渡す。
3. IO 層が返したアプリデータディレクトリからの相対パスをそのまま返す。

**エラーハンドリング**:
- 拡張子不正は `BizError::ValidationFailedAt { message: "不正な画像拡張子: {extension}（許可: jpg, jpeg, png, gif, webp）", field: "extension" }`。
- IO 層の失敗は `BizError::DatabaseError` へ変換する。CMD wire 上の internal message は移行前と不変。
