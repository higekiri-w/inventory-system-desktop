## 10. IO-01 追加: 在庫変動リポジトリ（BIZ-02 用）

### 計数の置き場と在庫の求め直しの契約（D-118。(a)・b1・b2 は実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1・D2・D9）の IO の側。保存の意味は [tracking](../db-design/tracking-system-tables.md) の同名の契約。旧 ADR（時点証拠）の版の強制（D-109 で `#[cfg(test)]` に置いたもの）は置き換え、b1 で外した。(a)（ADR SPEC-STK-CNT-D12）で入ったのは、`NewMovement` の並びの鍵、`update_movement_stock_after`、求め直しの 2 つの読取り、写しの書込み（下の 1〜4 つ目の項目）。b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で入ったのは、精算の読取り、計数の保存（`insert_stock_count`）とカーソルの読取り、初期在庫の印（ADR D9）。b2（[Plan Packet](../plans/2026-10-23-stocktake-count-runtime-b2.md)）で入ったのは、在庫変動の履歴の 2 つの読取りと、初期在庫の印の式の共有（下の「在庫変動の履歴の読取り」）。

- `NewMovement` に並びの鍵（`business_date: String`〈YYYY-MM-DD〉、`settlement_no: Option<i64>`）を足し、`insert_movement` が保存する。`phase` は field に持たず、`insert_movement` が `settlement_no` から決めて書く（`Some` なら 1、`None` なら 0。phase と settlement_no の食い違いを型で作れないようにする。2026-10-18、D-118 の runtime の (a) の起票で決めた。DB の CHECK も同じ対を縛る）。`MovementRecord` は並びの鍵の列を持たない（業務記録の詳細の `movements` と同じ形のまま）。鍵は在庫変動の履歴の行が持つ（下の「在庫変動の履歴の読取り」）。
- (a) の間の書込みの関数: `update_movement_stock_after`（その TX で INSERT した movement の `stock_after` を、求め直しの後の写しの値に書く。[31](31-biz-inventory-service.md) の `apply_stock_change`。シグネチャは下の code block）。
- 求め直しの読取り（BIZ の `derive_stock`・`recompute_stock` が呼ぶ）: (a) は商品の計数の一覧（`list_stock_counts_for_product`）と、商品の `is_voided=0` の movement（`list_ledger_movements_for_product`）の 2 つ（シグネチャは下の code block。どちらも id の順で返し、窓の判定は BIZ が ADR D3 の表で行う）。active な Z004 の取込みの精算の一覧（`csv_imports` の `settlement_date`・`settlement_no`・`id`）と、決まらない精算の行（その取込みの `sale_records` の商品の個数・金額・`stock_linked`）の読取りは b1 で入った（[24](24-io-csv-import-repo.md) の `list_active_settlements`・`list_sale_import_ids`）。
- 写しの書込み: `update_stock_quantity` は数量に加えて `stock_recheck`・`stock_range_low`・`stock_range_high` を書く形に変え、呼ぶのは `recompute_stock` だけ（共通入出庫・Z004 の取消・旧棚卸しの確定・整合性の補正は全て `recompute_stock` を通る）。`ProductUpdates` の数量の欄は外した（[20](20-io-product-repo.md)）。和の関数 `sum_movements_by_product`・`sum_movements_for_product` は外した（整合性の検査は [31](31-biz-inventory-service.md) の `derive_stock` と比べる。§10.6）。
- 計数の保存（b1）: `insert_stock_count`（`stock_counts` へ 1 行 INSERT。UPDATE・DELETE の関数は持たない。fresh なら `settlement_no`・`seen_movement_id`・`seen_import_id` を書く。取込みのカーソルの読取りは [24](24-io-csv-import-repo.md) の「Z004 の取込みの最大の取込みID」）。`list_stock_counts_for_product` は `seen_import_id` も返す（BIZ が ADR D3 の判定の n を求める）。`counted_at` も返す（明細の写しへ写す。35）。
- 在庫変動の履歴の読取り（b2。[44](44-cmd-inventory.md) の `list_movements`）: `list_history_movements`（商品の `is_voided = 0` の movement を、業務日の範囲〈両端を含む〉と種別で絞り、今の `MovementRecord` の列に `business_date`・`settlement_no`・初期在庫の印を足して返す）と `list_history_counts`（商品の計数を業務日の範囲で絞り、`products` を JOIN した `stock_unit` と、`stocktake_items` を LEFT JOIN した `stocktake_id` を足して返す）。どちらも並べない（鍵の並べ替え・件数・ページは BIZ）。初期在庫の印の SQL の式は `inventory_repo` の 1 つの定数で、`list_ledger_movements_for_product` と共有する。旧 `list_movements`（`created_at` の降順・`date_to` に時刻を足す比べ・SQL の `LIMIT`/`OFFSET`）は外した。
- 入出庫の一覧の棚卸し差異の集計は [65](65-inventory-record-traceability.md) の新契約（計数は movement を作らない）。

(a) で足した関数のシグネチャ（D-118 の runtime の (a)。BIZ からだけ呼ぶので `pub(crate)`。`src-tauri/tests/design_compliance_test.rs` の allowlist には足さない）:

```rust
// その TX で INSERT した movement 1 行の stock_after を書く。更新が 1 行でなければ DbError::QueryFailed
pub(crate) fn update_movement_stock_after(conn: &DbConnection, movement_id: i64, stock_after: i64) -> Result<(), DbError>
// stock_counts の商品の行（id・count_value・count_kind・business_date・seen_movement_id・seen_import_id・counted_at）を id の順で。
// settlement_no（ADR D1 の鍵の n）は計数に入っているかの判定に使わないので読まない（b1。判定の n は seen_import_id から求める）
pub(crate) fn list_stock_counts_for_product(conn: &DbConnection, product_code: &str) -> Result<Vec<StockCountRow>, DbError>
// is_voided = 0 の movement（id・quantity・business_date・phase・settlement_no・初期在庫の印）を id の順で。
// 初期在庫の印は reference_type IS NULL AND movement_type = 'receiving'（ADR D9、b1。demo の seed の sale_auto は初期在庫にしない）
pub(crate) fn list_ledger_movements_for_product(conn: &DbConnection, product_code: &str) -> Result<Vec<LedgerMovementRow>, DbError>
```

b1 で足した関数のシグネチャ（`pub(crate)`。計数と完了のカーソルは同じ TX で読む）:

```rust
pub(crate) struct NewStockCount {
    product_code: String, count_value: i64, fresh: bool, business_date: String,
    settlement_no: Option<i64>,    // fresh だけ。ADR D1 の鍵の n（判定に使わない）
    seen_movement_id: Option<i64>, // fresh だけ。MAX(inventory_movements.id)（無ければ 0）
    seen_import_id: Option<i64>,   // fresh だけ。MAX(csv_imports.id)（取り消したものを含む。無ければ NULL）
    stocktake_item_id: Option<i64>,
}
// stock_counts へ 1 行 INSERT し id を返す（UPDATE・DELETE の関数は持たない）。呼ぶのは BIZ-06 の save_stock_count・zero_uncounted_items だけ
pub(crate) fn insert_stock_count(conn: &DbConnection, count: &NewStockCount) -> Result<i64, DbError>
// MAX(inventory_movements.id)（無ければ 0）。fresh な計数と完了のカーソル
pub(crate) fn max_movement_id(conn: &DbConnection) -> Result<i64, DbError>
// MAX(stock_counts.id)（無ければ None）。完了のカーソル
pub(crate) fn max_stock_count_id(conn: &DbConnection) -> Result<Option<i64>, DbError>
// 計数を持つ商品の一覧（Z004 の確定・取消の求め直しの対象。ADR の横断の表）
pub(crate) fn list_counted_product_codes(conn: &DbConnection) -> Result<Vec<String>, DbError>
```

b2 で足した関数のシグネチャ（`pub(crate)`。BIZ の `list_movements` だけが呼ぶ。`query` の `product_code`・`date_from`・`date_to`・`movement_type` を読み、ページは読まない）:

```rust
// movement の行（MovementRecord〈source は None〉・business_date・settlement_no・initial_stock）。並べない
pub(crate) fn list_history_movements(conn: &DbConnection, query: &MovementQuery) -> Result<Vec<HistoryMovementRow>, DbError>
// 計数の行（id・count_value・fresh・business_date・settlement_no・counted_at・stock_unit・stocktake_id）。並べない
pub(crate) fn list_history_counts(conn: &DbConnection, query: &MovementQuery) -> Result<Vec<HistoryCountRow>, DbError>
```

### 10.1 inventory_repo — 共通型

**ListQuery構造体**（入庫/返品/廃棄の一覧取得で共通）:
- page: u32（1始まり）
- per_page: u32（デフォルト50、上限200。200超はBIZ層でバリデーションエラー）
- date_from: Option<String>（YYYY-MM-DD）
- date_to: Option<String>（YYYY-MM-DD）

### 10.2 inventory_repo — 入庫リポジトリ

#### insert_receiving_record

**関数要求**: receiving_records に1行INSERTし、挿入されたIDを返す

**シグネチャ**:
```
fn insert_receiving_record(conn: &DbConnection, record: &NewReceivingRecord) -> Result<i64, DbError>
```

**NewReceivingRecord構造体**:
- supplier_id: Option<i64>
- receiving_date: String（YYYY-MM-DD）
- note: Option<String>
- idempotency_key: String
- request_fingerprint: String

**処理ステップ**:
1. INSERT INTO receiving_records (supplier_id, receiving_date, note, idempotency_key, request_fingerprint, created_at) VALUES (?, ?, ?, ?, ?, 現在日時)
2. last_insert_rowid() を返す

**エラーハンドリング**:
- supplier_id指定時のFK違反 → DbError::ForeignKeyViolation
- idempotency_key重複 → DbError::DuplicateKey（BIZ層でIdempotencyConflict判定に使用）
- その他SQL実行失敗 → DbError::QueryFailed

#### insert_receiving_item

**シグネチャ**:
```
fn insert_receiving_item(conn: &DbConnection, item: &NewReceivingItem) -> Result<(), DbError>
```

**NewReceivingItem構造体**:
- receiving_record_id: i64
- product_code: String
- quantity: i64
- cost_price_centi: i64

#### list_receiving_records

**関数要求**: 入庫記録一覧をページング取得。取引先名をJOINで取得

**シグネチャ**:
```
fn list_receiving_records(conn: &DbConnection, query: &ListQuery) -> Result<PaginatedResult<ReceivingRecordWithSupplier>, DbError>
```

**ReceivingRecordWithSupplier構造体**:
- id: i64, supplier_id: Option<i64>, supplier_name: Option<String>, receiving_date: String, note: Option<String>, created_at: String

**処理ステップ**:
1. WHERE句構築: date_from/date_to → receiving_date >= ? / receiving_date <= ?
2. COUNT(*) でtotal_count取得
3. ORDER BY receiving_date DESC, id DESC
4. LIMIT per_page OFFSET (page - 1) * per_page

#### get_receiving_record_detail

**関数要求**: 入庫記録詳細として、ヘッダ、明細、原価合計、関連 `inventory_movements` を取得する。

**シグネチャ**:
```
fn get_receiving_record_detail(conn: &DbConnection, record_id: i64) -> Result<ReceivingRecordDetail, DbError>
```

**ReceivingRecordDetail構造体**:
- id: i64, receiving_date: String, supplier_id: Option<i64>, supplier_name: Option<String>, note: Option<String>, status: String, created_at: String
- total_cost: i64
- items: Vec<ReceivingRecordDetailItem>
- movements: Vec<MovementRecord>

**ReceivingRecordDetailItem構造体**:
- id: i64, product_code: String, product_name: String, department_name: String, stock_unit: ProductStockUnit
- quantity: i64, cost_price_centi: i64, line_cost_centi: String（IO は空の文字列で返し、BIZ が埋める）

**処理ステップ**:
1. receiving_records のヘッダを record_id で取得する。存在しない場合は DbError::NotFound
2. receiving_items を products / departments と JOIN し、商品名・部門名・単位を付ける
3. IO は金額を計算しない。明細の数量・原価・単位だけを返し（`line_cost_centi` は空、`total_cost` は 0）、BIZ が [共通規則](10-common-rules.md) SPEC-UNIT-D6 で行の金額と合計を求めて埋める（[31](31-biz-inventory-service.md) の「単位と原価の精度の契約」。`quantity * cost_price_centi` は長さの商品で 100 倍になる）
4. inventory_movements から `reference_type='receiving_record'` かつ `reference_id=record_id` かつ `is_voided=0` の行を取得する
5. movements は BIZ 層で source link を補完するため、IO 層では `source=None` のまま返す

### 10.3 inventory_repo — 返品リポジトリ

#### insert_return_record

**シグネチャ**:
```
fn insert_return_record(conn: &DbConnection, record: &NewReturnRecord) -> Result<i64, DbError>
```

**NewReturnRecord構造体**:
- return_type: String（"return" / "exchange"）
- return_date: String
- register_processed: bool
- receipt_image_path: Option<String>
- note: Option<String>
- idempotency_key: String
- request_fingerprint: String

#### insert_return_item

**シグネチャ**:
```
fn insert_return_item(conn: &DbConnection, item: &NewReturnItem) -> Result<(), DbError>
```

**NewReturnItem構造体**:
- return_record_id: i64
- product_code: String
- direction: String（"in" / "out"）
- quantity: i64

#### list_return_records

**シグネチャ**:
```
fn list_return_records(conn: &DbConnection, query: &ListQuery) -> Result<PaginatedResult<ReturnRecordSummary>, DbError>
```

**ReturnRecordSummary構造体**:
- id: i64, return_type: ReturnExchangeType, return_date: String, register_processed: bool, note: Option<String>, created_at: String

#### get_return_record_detail

**関数要求**: 返品・交換記録詳細として、ヘッダ、明細、レシート画像パス、関連 `inventory_movements` を取得する。`register_processed=true` の記録は作成時に在庫を動かしていないため、movements が 0 件でも正常とする。

**シグネチャ**:
```
fn get_return_record_detail(conn: &DbConnection, record_id: i64) -> Result<ReturnRecordDetail, DbError>
```

**ReturnRecordDetail構造体**:
- id: i64, return_type: ReturnExchangeType, return_date: String, register_processed: bool, receipt_image_path: Option<String>, note: Option<String>, status: String, created_at: String
- items: Vec<ReturnRecordDetailItem>
- movements: Vec<MovementRecord>

**ReturnRecordDetailItem構造体**:
- id: i64, product_code: String, product_name: String, department_name: String, stock_unit: ProductStockUnit
- direction: ReturnDirection, quantity: i64

**処理ステップ**:
1. return_records のヘッダを record_id で取得する。存在しない場合は DbError::NotFound
2. return_items を products / departments と JOIN し、商品名・部門名・単位を付ける
3. inventory_movements から `reference_type='return_record'` かつ `reference_id=record_id` かつ `is_voided=0` の行を取得する
4. movements は BIZ 層で source link を補完するため、IO 層では `source=None` のまま返す

### 10.4 inventory_repo — 手動販売出庫リポジトリ

#### insert_manual_sale

**シグネチャ**:
```
fn insert_manual_sale(conn: &DbConnection, record: &NewManualSale) -> Result<i64, DbError>
```

**NewManualSale構造体**:
- sale_date: String
- reason: String（"plu_unregistered" / "other"）
- note: Option<String>
- idempotency_key: String
- request_fingerprint: String

#### insert_manual_sale_item

**シグネチャ**:
```
fn insert_manual_sale_item(conn: &DbConnection, item: &NewManualSaleItem) -> Result<(), DbError>
```

**NewManualSaleItem構造体**:
- manual_sale_id: i64
- product_code: String
- quantity: i64
- amount: i64

#### get_manual_sale_record_detail

**関数要求**: 手動販売記録詳細として、ヘッダ、明細、販売金額合計、関連 `inventory_movements` を取得する。`sale_records.id` ではなく `manual_sales.id` を業務記録IDとして扱う。

**シグネチャ**:
```
fn get_manual_sale_record_detail(conn: &DbConnection, record_id: i64) -> Result<ManualSaleRecordDetail, DbError>
```

**ManualSaleRecordDetail構造体**:
- id: i64, sale_date: String, reason: ManualSaleReason, note: Option<String>, status: String, created_at: String
- total_amount: i64
- items: Vec<ManualSaleRecordDetailItem>
- movements: Vec<MovementRecord>

**ManualSaleRecordDetailItem構造体**:
- id: i64, product_code: String, product_name: String, department_name: String, stock_unit: ProductStockUnit
- quantity: i64, amount: i64

**処理ステップ**:
1. manual_sales のヘッダを record_id で取得する。存在しない場合は DbError::NotFound
2. manual_sale_items を products / departments と JOIN し、商品名・部門名・単位を付ける
3. 各明細の amount を合計して total_amount にする
4. inventory_movements から `reference_type='manual_sale'` かつ `reference_id=record_id` かつ `is_voided=0` の行を取得する
5. movements は BIZ 層で source link を補完するため、IO 層では `source=None` のまま返す

### 10.5 inventory_repo — 廃棄リポジトリ

#### insert_disposal_record

**シグネチャ**:
```
fn insert_disposal_record(conn: &DbConnection, record: &NewDisposalRecord) -> Result<i64, DbError>
```

**NewDisposalRecord構造体**:
- disposal_date: String
- idempotency_key: String
- request_fingerprint: String

#### insert_disposal_item

**シグネチャ**:
```
fn insert_disposal_item(conn: &DbConnection, item: &NewDisposalItem) -> Result<(), DbError>
```

**NewDisposalItem構造体**:
- disposal_record_id: i64
- product_code: String
- disposal_type: String（"disposal" / "damage" / "other"）
- quantity: i64
- cost_price_centi: i64
- reason: String

#### list_disposal_records

**シグネチャ**:
```
fn list_disposal_records(conn: &DbConnection, query: &ListQuery) -> Result<PaginatedResult<DisposalRecordSummary>, DbError>
```

**DisposalRecordSummary構造体**:
- id: i64, disposal_date: String, created_at: String

#### list_inventory_records

**関数要求**: `/inventory/records` 用に、入庫・返品/交換・手動販売・廃棄/破損・CSV取込み・棚卸し記録を業務記録ヘッダ単位で検索・ページング取得する。明細 JOIN 条件に一致する場合でも返却行は各ヘッダ 1 件に集約する。

**シグネチャ**:
```
fn list_inventory_records(
    conn: &DbConnection,
    query: &InventoryRecordQuery,
) -> Result<PaginatedResult<InventoryRecordSummary>, DbError>
```

**InventoryRecordQuery構造体**:
- record_type: Option<String>（None / "all" / "receiving_record" / "return_record" / "manual_sale" / "disposal_record" / "csv_import" / "stocktake"）
- date_from: Option<String>（YYYY-MM-DD）
- date_to: Option<String>（YYYY-MM-DD）
- record_id: Option<i64>
- product_keyword: Option<String>（商品コード / JAN / 商品名）
- department_id: Option<i64>
- status: Option<String>（None は "all" 相当。正規化後の許容値は "all" / "active" / "canceled" / "in_progress"）
- page: u32
- per_page: u32

**InventoryRecordSummary構造体**:
- record_type: String
- record_id: i64
- business_date: String
- representative_item: String（汎用 fallback は "明細なし"。完了棚卸しの差異 0 件は "差異なし"、進行中棚卸しは UI で "-" 表示）
- item_count: i64
- status: String
- created_at: String
- detail_route: String

**処理ステップ**:
1. record_type / status が IO 層で未対応の場合は空ページを返す（BIZ 層の validation が通常の入口）。status は None を `all` とみなし、`all` / `active` / `canceled` / `in_progress` の 4 値だけを許容する
2. 対象 record_type ごとに同じ列構造の SELECT を作る
   - 入庫: `receiving_records` + `receiving_items`
   - 返品・交換: `return_records` + `return_items`
   - 手動販売: `manual_sales` + `manual_sale_items`
   - 廃棄・破損: `disposal_records` + `disposal_items`
   - CSV取込み: `csv_imports` + `sale_records`。TRACE-D6 の履歴保持方針により `sale_records.is_voided=1` も `item_count` / `representative_item` に含める
   - 棚卸し: `stocktakes` + 差異 `inventory_movements`。差異母集団は `reference_type='stocktake' AND is_voided=0` に必ず絞る
   - `status_expr`: CSV取込みは `CASE status WHEN 'rolled_back' THEN 'canceled' ELSE 'active' END`、棚卸しは `CASE status WHEN 'completed' THEN 'active' ELSE 'in_progress' END`、既存 4 種は `'active'` 固定を継続する
   - `date_expr`: CSV取込みは `settlement_date`、棚卸しは `DATE(COALESCE(completed_at, started_at))`。棚卸しの `DATE()` ラップは SELECT と日付範囲 WHERE の 2 箇所へ同じ式で入れ、外側 ORDER BY は `business_date` alias を継承するため変更しない
   - `created_at_col`: CSV取込みは `imported_at`、棚卸しは `started_at`。`csv_imports` / `stocktakes` のいずれにも `created_at` 列は存在しない
3. WHERE句構築: 業務日付、record_id、department_id、product_keyword を条件化する。日付条件は `date_expr` を式として直接 WHERE に使い、`header_alias.` を機械的に前置しない
4. 商品条件は各明細テーブルの `EXISTS` + products JOIN で判定し、ヘッダ重複を避ける。CSV取込みは `sale_records`、棚卸しは前項の差異 movement を母集団とするため、進行中棚卸しは product_keyword / department_id に構造的に hit しない
5. 6 種の `UNION ALL` を derived table で包み、status が `all` 以外のときは正規化済み `status` alias への WHERE を外側で 1 回だけ適用する。COUNT(*) と一覧 SELECT の双方が同じ外側 status 条件を使う
6. `UNION ALL` 後に COUNT(*) で total_count 取得
7. ORDER BY business_date DESC, record_id DESC, record_type ASC
8. LIMIT per_page OFFSET (page - 1) * per_page

#### get_disposal_record_detail

**関数要求**: 廃棄・破損記録詳細として、ヘッダ、明細、ロス原価、関連 `inventory_movements` を取得する。

**シグネチャ**:
```
fn get_disposal_record_detail(
    conn: &DbConnection,
    record_id: i64,
) -> Result<DisposalRecordDetail, DbError>
```

**DisposalRecordDetail構造体**:
- id: i64
- disposal_date: String
- status: String
- created_at: String
- total_loss_cost: i64
- items: Vec<DisposalRecordDetailItem>
- movements: Vec<MovementRecord>

**DisposalRecordDetailItem構造体**:
- id: i64
- product_code: String
- product_name: String
- department_name: String
- stock_unit: ProductStockUnit
- disposal_type: DisposalType
- quantity: i64
- cost_price_centi: i64
- reason: String
- line_loss_cost_centi: String（IO は空の文字列で返し、BIZ が埋める）

**処理ステップ**:
1. disposal_records のヘッダを record_id で取得する。存在しない場合は DbError::NotFound
2. disposal_items を products / departments と JOIN し、商品名・部門名・単位を付ける
3. IO は金額を計算しない（入庫の §10.2 と同じ）。`line_loss_cost_centi` は空、`total_loss_cost` は 0 で返し、BIZ が SPEC-UNIT-D6 で求めて埋める
4. inventory_movements から `reference_type='disposal_record'` かつ `reference_id=record_id` かつ `is_voided=0` の行を取得する
5. movements は BIZ 層で source link を補完するため、IO 層では `source=None` のまま返す

### 10.6 inventory_repo — movements集計（D-118 の (a) で外した）

`sum_movements_by_product`・`sum_movements_for_product`（`is_voided = 0` の movement の和）は、整合性の検査の相手を「記録から求め直した値」にしたので外した（[36](36-biz-integrity-check.md)）。計数が無い商品の求め直しの値は同じ和になる（[31](31-biz-inventory-service.md) の `derive_stock`。読取りは上の `list_ledger_movements_for_product`）。

外した関数のシグネチャ（履歴。production の code と呼出しは無い）:

```
fn sum_movements_by_product(conn: &DbConnection) -> Result<Vec<ProductMovementSum>, DbError>
fn sum_movements_for_product(conn: &DbConnection, product_code: &str) -> Result<i64, DbError>
```

---

### 10.6a inventory_repo — 在庫数直接更新

#### update_stock_quantity

**関数要求**: products の写しの 4 項目（`stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`）を指定値で上書き更新する。呼ぶのは BIZ の `recompute_stock` だけ（上の「計数の置き場と在庫の求め直しの契約」）

**シグネチャ**:
```
fn update_stock_quantity(conn: &DbConnection, product_code: &str, stock: i64, range: Option<(i64, i64)>) -> Result<bool, DbError>
```

**設計判断**: product_repo::update_product(ProductUpdates { stock_quantity }) を使わず、inventory_repo に専用関数を配置する。理由:（1）stock_quantity更新は在庫変動ドメインの責務、（2）1カラムだけの更新にProductUpdates構造体を組み立てるのはオーバーヘッド

**処理ステップ**:
1. UPDATE products SET stock_quantity = ?1, stock_recheck = ?2（`range.is_some()`）, stock_range_low = ?3, stock_range_high = ?4（`range` の両端、`None` なら NULL）, updated_at = ?5 WHERE product_code = ?6
2. affected_rows == 1 → Ok(true)
3. affected_rows == 0 → Ok(false)

### 10.7 inventory_repo — 冪等性チェック

#### find_by_idempotency_key

**関数要求**: idempotency_key で各ヘッダテーブルを検索する。冪等性チェック用

**シグネチャ（4テーブルそれぞれに1関数）**:
```
fn find_receiving_by_idempotency_key(conn: &DbConnection, key: &str) -> Result<Option<(i64, String)>, DbError>
fn find_return_by_idempotency_key(conn: &DbConnection, key: &str) -> Result<Option<(i64, String)>, DbError>
fn find_manual_sale_by_idempotency_key(conn: &DbConnection, key: &str) -> Result<Option<(i64, String)>, DbError>
fn find_disposal_by_idempotency_key(conn: &DbConnection, key: &str) -> Result<Option<(i64, String)>, DbError>
```

**戻り値**: Some((record_id, request_fingerprint)) または None

---

## 11. sales_repo — 売上レコード

### 11.1 insert_sale_record

**関数要求**: sale_records に1行INSERTし、挿入されたIDを返す。手動販売時に使用

**シグネチャ**:
```
fn insert_sale_record(conn: &DbConnection, record: &NewSaleRecord) -> Result<i64, DbError>
```

**NewSaleRecord構造体**:
- csv_import_id: Option<i64>（手動販売時はNone）
- product_code: String
- sale_date: String
- quantity: i64（売上帳票視点: 正=販売）
- amount: i64
- source: String（"auto" / "manual"）
- source_line_no: Option<i64>（手動販売時はNone）
- reason: Option<String>
- note: Option<String>
- stock_linked: bool（取込みのときの在庫連動。Z004 の確定だけが行の `pos_stock_sync` を渡し、手動の売上ほかは false。作る所が値を書く。[POS](../db-design/pos-tables.md)、b1）

**処理ステップ**:
1. INSERT INTO sale_records (..., is_voided, created_at) VALUES (..., 0, 現在日時)
2. last_insert_rowid() を返す

**sale_records (source='manual') の一意性保証**:
- 前提: manual_sales ヘッダ→items→sale_records の INSERT は create_manual_sale の単一TX内で完了
- TX外からの直接INSERT（source='manual'）は設計違反とする
- ヘッダの idempotency_key UNIQUE が一意性の根拠
