## 23. CMD-02〜05: 入出庫コマンド群 / CMD-06: 在庫照会コマンド群

### 23.1 モジュール構成

```
src-tauri/src/
  cmd/
    mod.rs               -- pub mod receiving_cmd 等を追加
    receiving_cmd.rs     -- 入庫コマンド（CMD-02）
    return_cmd.rs        -- 返品・交換コマンド（CMD-03）
    manual_sale_cmd.rs   -- 手動販売出庫コマンド（CMD-04）
    disposal_cmd.rs      -- 廃棄・破損コマンド（CMD-05）
    inventory_cmd.rs     -- 在庫照会コマンド（CMD-06）
  biz/
    inventory_service/
      mod.rs             -- list系ラッパー3関数を追加
  db/
    product_repo.rs      -- get_stock_detail, list_low_stock_products を追加
    inventory_repo.rs    -- 在庫変動の履歴の 2 つの読取り（list_history_movements・list_history_counts）
```

---

### 23.2 CMD層の原則（17.2節を継承）

17.2節（41-cmd-pos.md）の原則をそのまま適用:
- 薄いラッパー: state.db.lock() → BIZ呼出し → BizError→CmdError変換
- 業務バリデーション、ビジネスロジックは持たない

**CMD-02〜06はpreview_cacheを使用しない**。DB接続のみ。

**list操作のレイヤー呼び出し**: 既存パターン（csv_import_cmd → csv_import_service → sales_repo）に倣い、list操作もCMD→BIZ→IOの順で呼び出す。BIZ層にはページパラメータのバリデーション + repo直呼び防止ラッパーを置く。

---

### 23.3 BizError → CmdError 変換

新規BizError variantの追加は不要。既存の変換ルール（40-cmd-product.md §5.3 + 41-cmd-pos.md §17.4 + 42-cmd-sales-stocktake.md §22.3）で全コマンドのエラーをカバーできる。

CMD-02〜05で発生しうるBizError:

| BizError | CmdError.kind | 発生場面 |
|----------|--------------|---------|
| ValidationFailed(msg) | "validation" | 入力バリデーション（空items、不正日付等） |
| NotFound(msg) | "not_found" | 存在しない商品コード |
| IdempotencyConflict(msg) | "idempotency_conflict" | 同一冪等キーで異なるリクエスト |
| DatabaseError(_) | "internal" | IO層エラー |

CMD-06で発生しうるBizError:

| BizError | CmdError.kind | 発生場面 |
|----------|--------------|---------|
| NotFound(msg) | "not_found" | get_stock_detail: 存在しない商品コード |
| ValidationFailed(msg) | "validation" | list_low_stock/list_movements: 不正なパラメータ |
| DatabaseError(_) | "internal" | IO層エラー |

---

### 23.4 CMD-02: 入庫コマンド群

#### create_receiving

**関数要求**: 入庫記録を作成し、在庫を更新する

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn create_receiving(
    state: State<AppState>,
    req: ReceivingCreateRequest,
) -> Result<ReceivingCreateResult, CmdError>
```

**入力型**: BIZ層の `ReceivingCreateRequest` をそのまま使用
```
struct ReceivingCreateRequest {
    idempotency_key: String,
    supplier_id: Option<i64>,
    receiving_date: String,       // YYYY-MM-DD
    note: Option<String>,
    items: Vec<ReceivingItemInput>,
}
struct ReceivingItemInput {
    product_code: String,
    quantity: i64,
    cost_price_centi: i64,
}
```

**出力型**: BIZ層の `ReceivingCreateResult` をそのまま使用
```
struct ReceivingCreateResult {
    record_id: i64,
    created: bool,
    idempotent_replay: bool,
    stock_warnings: Vec<String>,
    cost_diffs: Vec<CostDiff>,
}
struct CostDiff {
    product_code: String,
    product_name: String,
    master_cost_price_centi: i64,
    received_cost_price_centi: i64,
}
```

`cost_diffs` は SPEC-PRV-D8 / REQ-209 の field 追加だけであり、既存 field の意味・名称は変えない。既存 consumer は `cost_diffs` を無視できる。新 consumer の UI-02 は保存成功時だけ読み、idempotent replay の空配列ではダイアログを出さない。

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&mut conn`）
2. biz::inventory_service::create_receiving(&mut conn, req) を呼ぶ
3. Ok → ReceivingCreateResult をそのまま返す
4. Err(BizError) → CmdError に変換して返す

**入力例**:
```json
{
  "req": {
    "idempotency_key": "recv-20260321-001",
    "supplier_id": 1,
    "receiving_date": "2026-03-21",
    "note": "定期入荷",
    "items": [
      { "product_code": "4976383262108", "quantity": 12, "cost_price_centi": 11100 },
      { "product_code": "4976383262207", "quantity": 8, "cost_price_centi": 11100 }
    ]
  }
}
```

**出力例**:
```json
{
  "record_id": 5,
  "created": true,
  "idempotent_replay": false,
  "stock_warnings": []
}
```

---

#### list_receivings

**関数要求**: 入庫記録の一覧をページング取得する

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn list_receivings(
    state: State<AppState>,
    page: u32,
    per_page: u32,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<PaginatedResult<ReceivingRecordWithSupplier>, CmdError>
```

**入力パラメータ**（個別引数方式。CMD層で `ListQuery` を組み立ててBIZ層に渡す）:
- `page: u32` — 1始まり
- `per_page: u32` — 上限200（BIZ層でバリデーション）
- `date_from: Option<String>` — YYYY-MM-DD
- `date_to: Option<String>` — YYYY-MM-DD

**出力型**: `PaginatedResult<ReceivingRecordWithSupplier>`
```
struct ReceivingRecordWithSupplier {
    id: i64,
    supplier_id: Option<i64>,
    supplier_name: Option<String>,
    receiving_date: String,
    note: Option<String>,
    created_at: String,
}
```

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&conn`）
2. 個別パラメータから `ListQuery { page, per_page, date_from, date_to }` を組み立てる
3. biz::inventory_service::list_receivings(&conn, &query) を呼ぶ
4. Ok → PaginatedResult をそのまま返す
5. Err(BizError) → CmdError に変換して返す

**入力例**:
```json
{
  "page": 1,
  "per_page": 20,
  "date_from": "2026-03-01",
  "date_to": null
}
```

**出力例**:
```json
{
  "items": [
    {
      "id": 5,
      "supplier_id": 1,
      "supplier_name": "ハマナカ",
      "receiving_date": "2026-03-21",
      "note": "定期入荷",
      "created_at": "2026-03-21T10:30:00"
    }
  ],
  "total_count": 12,
  "page": 1,
  "per_page": 20
}
```

---

### 23.5 CMD-03: 返品・交換コマンド群

#### create_return

**関数要求**: 返品・交換記録を作成する。register_processedフラグに応じて在庫を更新する

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn create_return(
    state: State<AppState>,
    req: ReturnCreateRequest,
) -> Result<ReturnCreateResult, CmdError>
```

**入力型**: BIZ層の `ReturnCreateRequest` をそのまま使用
```
struct ReturnCreateRequest {
    idempotency_key: String,
    return_type: ReturnExchangeType, // "return" | "exchange"
    return_date: String,          // YYYY-MM-DD
    register_processed: bool,
    receipt_image_path: Option<String>,
    note: Option<String>,
    items: Vec<ReturnItemInput>,
}
struct ReturnItemInput {
    product_code: String,
    direction: ReturnDirection,   // "in" | "out"
    quantity: i64,
}
```

**出力型**: BIZ層の `ReturnCreateResult` をそのまま使用
```
struct ReturnCreateResult {
    record_id: i64,
    created: bool,
    idempotent_replay: bool,
    stock_warnings: Vec<String>,
}
```

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&mut conn`）
2. biz::inventory_service::create_return(&mut conn, req) を呼ぶ
3. Ok → ReturnCreateResult をそのまま返す
4. Err(BizError) → CmdError に変換して返す

**入力不変条件**:
return_type / direction の値域は generated enum が検査し、exchange片側不足などの業務不変条件は BIZ-02 が最終防御する。CMD-03 は薄いラッパーとして request を中継し、BIZ validation error を `CmdError.kind="validation"` に変換する。

**入力例**:
```json
{
  "req": {
    "idempotency_key": "ret-20260321-001",
    "return_type": "exchange",
    "return_date": "2026-03-21",
    "register_processed": false,
    "receipt_image_path": null,
    "note": "色交換",
    "items": [
      { "product_code": "HZ-0012", "direction": "in", "quantity": 1 },
      { "product_code": "HZ-0013", "direction": "out", "quantity": 1 }
    ]
  }
}
```

**出力例**:
```json
{
  "record_id": 3,
  "created": true,
  "idempotent_replay": false,
  "stock_warnings": []
}
```

---

#### list_returns

**関数要求**: 返品記録の一覧をページング取得する

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn list_returns(
    state: State<AppState>,
    page: u32,
    per_page: u32,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<PaginatedResult<ReturnRecordSummary>, CmdError>
```

**出力型**: `PaginatedResult<ReturnRecordSummary>`
```
struct ReturnRecordSummary {
    id: i64,
    return_type: ReturnExchangeType,
    return_date: String,
    register_processed: bool,
    note: Option<String>,
    created_at: String,
}
```

**処理ステップ**: list_receivings と同パターン（BIZ経由）

---

### 23.6 CMD-04: 手動販売出庫コマンド群

#### create_manual_sale

**関数要求**: 手動販売出庫を記録する。PLU登録済み警告を含む2段階確認フロー対応

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn create_manual_sale(
    state: State<AppState>,
    req: ManualSaleCreateRequest,
) -> Result<ManualSaleCreateResult, CmdError>
```

**入力型**: BIZ層の `ManualSaleCreateRequest` をそのまま使用
```
struct ManualSaleCreateRequest {
    idempotency_key: String,
    sale_date: String,            // YYYY-MM-DD
    reason: ManualSaleReason,     // "plu_unregistered" | "other"
    note: Option<String>,
    items: Vec<ManualSaleItemInput>,
    confirmation_token: Option<String>,  // PLU警告確認後の再送時に使用
}
struct ManualSaleItemInput {
    product_code: String,
    quantity: i64,
    amount: i64,
}
```

**出力型**: BIZ層の `ManualSaleCreateResult` をそのまま使用
```
struct ManualSaleCreateResult {
    sale_id: Option<i64>,
    created: bool,
    idempotent_replay: bool,
    plu_warnings: Vec<String>,
    stock_warnings: Vec<String>,
    needs_confirmation: bool,
    confirmation_token: Option<String>,
}
```

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&mut conn`）
2. biz::inventory_service::create_manual_sale(&mut conn, req) を呼ぶ
3. Ok → ManualSaleCreateResult をそのまま返す
4. Err(BizError) → CmdError に変換して返す

**2段階確認フロー**:
1回目: confirmation_token=None で送信 → needs_confirmation=true, plu_warnings 付きで返る
2回目: 1回目で返されたconfirmation_token を付けて再送信 → 実際の登録が実行される

---

### 23.7 CMD-05: 廃棄・破損コマンド群

#### create_disposal

**関数要求**: 廃棄・破損記録を作成し、在庫を減少させる

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn create_disposal(
    state: State<AppState>,
    req: DisposalCreateRequest,
) -> Result<DisposalCreateResult, CmdError>
```

**入力型**: BIZ層の `DisposalCreateRequest` をそのまま使用
```
struct DisposalCreateRequest {
    idempotency_key: String,
    disposal_date: String,        // YYYY-MM-DD
    items: Vec<DisposalItemInput>,
}
struct DisposalItemInput {
    product_code: String,
    disposal_type: DisposalType,  // "disposal" | "damage" | "other"
    quantity: i64,
    cost_price_centi: i64,
    reason: String,
}
```

**出力型**: BIZ層の `DisposalCreateResult` をそのまま使用
```
struct DisposalCreateResult {
    record_id: i64,
    created: bool,
    idempotent_replay: bool,
    stock_warnings: Vec<String>,
}
```

**処理ステップ**: create_receiving と同パターン

**enum 契約化（D-061 / D-064）**: 本節群（§23.5〜§23.7）の `return_type` / `direction` / `disposal_type` と、§23.6 手動販売の `manual_sales.reason`（family (5)、CHECK 制約付き 2 値）は generated enum で検査する。値・wire 表現・分岐意味論は不変。**§23.7 廃棄の `reason`（`DisposalItemInput.reason` 等）は CHECK 制約なしの自由記述 field であり D-061 の対象外** — 値集合が存在しないため enum 化してはならない。

---

#### list_disposals

**関数要求**: 廃棄記録の一覧をページング取得する

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn list_disposals(
    state: State<AppState>,
    page: u32,
    per_page: u32,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<PaginatedResult<DisposalRecordSummary>, CmdError>
```

**出力型**: `PaginatedResult<DisposalRecordSummary>`
```
struct DisposalRecordSummary {
    id: i64,
    disposal_date: String,
    created_at: String,
}
```

**処理ステップ**: list_receivings と同パターン（BIZ経由）

---

#### list_inventory_records

**関数要求**: 入出庫履歴ハブ `/inventory/records` 用に、業務記録をヘッダ単位でページング取得する。入庫・返品/交換・手動販売・廃棄/破損・CSV取込み・棚卸しの 6 種を実データとして返す。

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn list_inventory_records(
    state: State<AppState>,
    query: InventoryRecordQuery,
) -> Result<PaginatedResult<InventoryRecordSummary>, CmdError>
```

**入力型**: BIZ層の `InventoryRecordQuery` をそのまま使用
```
struct InventoryRecordQuery {
    record_type: Option<String>,      // None | "all" | "receiving_record" | "return_record" | "manual_sale" | "disposal_record" | "csv_import" | "stocktake"
    date_from: Option<String>,        // YYYY-MM-DD
    date_to: Option<String>,          // YYYY-MM-DD
    record_id: Option<i64>,
    product_keyword: Option<String>,  // 商品コード / JAN / 商品名
    department_id: Option<i64>,
    status: Option<String>,           // None は "all" 相当 | "all" | "active" | "canceled" | "in_progress"
    page: u32,
    per_page: u32,
}
```

**出力型**:
```
struct InventoryRecordSummary {
    record_type: String,
    record_id: i64,
    business_date: String,
    representative_item: String,       // 汎用 fallback は "明細なし"。棚卸しの差異 0 件は "差異なし"
    item_count: i64,
    status: String,
    created_at: String,
    detail_route: String,
}
```

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&conn`）
2. biz::inventory_service::list_inventory_records(&conn, &query) を呼ぶ
3. Ok → PaginatedResult をそのまま返す
4. Err(BizError) → CmdError に変換して返す

**制約**: `record_type` が未指定または `all` の場合は 6 種を返す。status は None を `all` とみなし、`all` / `active` / `canceled` / `in_progress` の 4 値を許容する。未対応の record_type / status は BIZ 層で validation error にする。

---

#### get_receiving_record

**関数要求**: 入庫記録詳細を取得し、明細、原価合計、関連在庫変動を表示できる形で返す。

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn get_receiving_record(
    state: State<AppState>,
    record_id: i64,
) -> Result<ReceivingRecordDetail, CmdError>
```

**出力型**:
```
struct ReceivingRecordDetail {
    id: i64,
    receiving_date: String,
    supplier_id: Option<i64>,
    supplier_name: Option<String>,
    note: Option<String>,
    status: String,
    created_at: String,
    items: Vec<ReceivingRecordDetailItem>,
    total_cost: i64,
    movements: Vec<MovementRecord>,
}
```

明細の `stock_unit` は `ProductStockUnit`、行の金額は 1/100 円の十進の文字列 `line_cost_centi`、合計の `total_cost` は円の整数（D-113）。明細の原価は `cost_price_centi`（1/100 円。原価の lane、D-128）。[31](31-biz-inventory-service.md) の「単位と原価の精度の契約」と [共通規則](10-common-rules.md) SPEC-UNIT-D5・D6・D11。

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&conn`）
2. biz::inventory_service::get_receiving_record(&conn, record_id) を呼ぶ
3. Ok → ReceivingRecordDetail をそのまま返す
4. Err(BizError::NotFound) → CmdError.kind="not_found" に変換して返す
5. Err(other) → CmdError に変換して返す

---

#### get_return_record

**関数要求**: 返品・交換記録詳細を取得し、明細、レジ戻し済み状態、レシート画像パス、関連在庫変動を表示できる形で返す。

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn get_return_record(
    state: State<AppState>,
    record_id: i64,
) -> Result<ReturnRecordDetail, CmdError>
```

**出力型**:
```
struct ReturnRecordDetail {
    id: i64,
    return_type: ReturnExchangeType,
    return_date: String,
    register_processed: bool,
    receipt_image_path: Option<String>,
    note: Option<String>,
    status: String,
    created_at: String,
    items: Vec<ReturnRecordDetailItem>,
    movements: Vec<MovementRecord>,
}
```

**処理ステップ**: get_receiving_record と同パターン。`register_processed=true` の記録は movements が 0 件でも正常に返す。

---

#### get_manual_sale_record

**関数要求**: 手動販売記録詳細を取得し、明細、販売金額合計、関連在庫変動を表示できる形で返す。

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn get_manual_sale_record(
    state: State<AppState>,
    record_id: i64,
) -> Result<ManualSaleRecordDetail, CmdError>
```

**出力型**:
```
struct ManualSaleRecordDetail {
    id: i64,
    sale_date: String,
    reason: ManualSaleReason,
    note: Option<String>,
    status: String,
    created_at: String,
    items: Vec<ManualSaleRecordDetailItem>,
    total_amount: i64,
    movements: Vec<MovementRecord>,
}
```

**処理ステップ**: get_receiving_record と同パターン。業務記録IDは `manual_sales.id` であり、`sale_records.id` ではない。

---

#### get_disposal_record

**関数要求**: 廃棄・破損記録詳細を取得し、明細、ロス原価合計、関連在庫変動を表示できる形で返す。

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn get_disposal_record(
    state: State<AppState>,
    record_id: i64,
) -> Result<DisposalRecordDetail, CmdError>
```

**出力型**:
```
struct DisposalRecordDetail {
    id: i64,
    disposal_date: String,
    status: String,
    created_at: String,
    total_loss_cost: i64,
    items: Vec<DisposalRecordDetailItem>,
    movements: Vec<MovementRecord>,
}

struct DisposalRecordDetailItem {
    id: i64,
    product_code: String,
    product_name: String,
    department_name: String,
    stock_unit: ProductStockUnit,
    disposal_type: DisposalType,
    quantity: i64,
    cost_price_centi: i64,
    reason: String,
    line_loss_cost_centi: String, // 1/100 円の整数の十進の文字列
}
```

`line_loss_cost_centi` は BIZ が求め、`total_loss_cost` は円の整数（D-113）。明細の原価は `cost_price_centi`（1/100 円。原価の lane、D-128）。[31](31-biz-inventory-service.md) の「単位と原価の精度の契約」と [共通規則](10-common-rules.md) SPEC-UNIT-D5・D6・D11。

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&conn`）
2. biz::inventory_service::get_disposal_record(&conn, record_id) を呼ぶ
3. Ok → DisposalRecordDetail をそのまま返す
4. Err(BizError::NotFound) → CmdError.kind="not_found" に変換して返す
5. Err(other) → CmdError に変換して返す

---

### 23.8 CMD-06: 在庫照会コマンド群

#### get_stock_detail

**関数要求**: 商品の在庫詳細情報を取得する（最終入庫日・最終販売日を含む）

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn get_stock_detail(
    state: State<AppState>,
    product_code: String,
) -> Result<StockDetail, CmdError>
```

**出力型（新規）**:
```
struct StockDetail {
    product: ProductWithRelations,
    last_receiving_date: Option<String>,  // 最終入庫日（YYYY-MM-DD）
    last_sale_date: Option<String>,       // 最終販売日（YYYY-MM-DD）
}
```

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&conn`）
2. biz::product_service::get_stock_detail(&conn, &product_code) を呼ぶ
3. Ok → StockDetail をそのまま返す
4. Err(BizError::NotFound) → CmdError { kind: "not_found" }
5. Err(other) → CmdError に変換

**入力例**:
```json
{
  "product_code": "4976383262108"
}
```

**出力例**:
```json
{
  "product": {
    "product_code": "4976383262108",
    "jan_code": "4976383262108",
    "name": "ハマナカ アミアミ極太 col.42",
    "department_id": 3,
    "selling_price": 648,
    "cost_price_centi": 11100,
    "stock_quantity": 14,
    "stock_unit": "pcs",
    "is_discontinued": false,
    "department_name": "毛糸",
    "supplier_name": "ハマナカ"
  },
  "last_receiving_date": "2026-03-21",
  "last_sale_date": "2026-03-23"
}
```

---

#### list_low_stock

**関数要求**: 在庫が閾値以下の商品を一覧取得する。数量の種類（個数・長さ）ごとに異なる閾値を適用する

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn list_low_stock(
    state: State<AppState>,
    include_discontinued: bool,
) -> Result<Vec<ProductWithRelations>, CmdError>
```

**入力型**:
```
include_discontinued: bool   // true=廃番含む、false=廃番除外
```

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&conn`）
2. biz::product_service::list_low_stock(&conn, include_discontinued) を呼ぶ
   - BIZ層内部で system_repo::get_setting で `stock_low_threshold`（個数の単位用）と `stock_low_threshold_fabric`（長さの単位用）を取得
   - product_repo::list_low_stock_products に、単位の 2 つの並びと閾値を渡す
3. Ok → Vec<ProductWithRelations> をそのまま返す
4. Err(BizError) → CmdError に変換して返す

**設計判断 — 閾値をCMD層に渡さない理由**: 閾値はapp_settings（DB）に格納されている。CMD層が閾値を取得してIO層に渡す構造だとCMD層がDBに2回アクセスすることになる。BIZ層で閾値取得とクエリを一括で行う方がシンプル。

**入力例**:
```json
{
  "include_discontinued": false
}
```

**出力例**:
```json
[
  {
    "product_code": "4976383262108",
    "name": "ハマナカ アミアミ極太 col.42",
    "stock_quantity": 2,
    "stock_unit": "pcs",
    "department_name": "毛糸"
  },
  {
    "product_code": "NU-0015",
    "name": "リバティ柄 タナローン",
    "stock_quantity": 300,
    "stock_unit": "m",
    "department_name": "布"
  }
]
```
※ 上記例ではpcs閾値=3, cm閾値=500の場合。stock_quantity=2 <= 3、stock_quantity=300 <= 500 でヒット

---

#### list_movements

**関数要求**: 商品別の在庫変動の履歴（movement と計数）をフィルタ付きでページング取得する。計数（`stock_counts`）は movement を作らないので、商品の movement と計数を 1 つの履歴に混ぜ、並びの鍵の順で返す（D-118 の b2。[ADR](../adr/2026-10-16-stocktake-count-placement.md) SPEC-STK-CNT-D1・D12 と [65](65-inventory-record-traceability.md) の D-118 の節の 1 つ目の項の CMD-06 の側。2026-10-23、b2 の起票で決めた）

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
fn list_movements(
    state: State<AppState>,
    query: MovementQuery,
) -> Result<PaginatedResult<StockHistoryItem>, CmdError>
```

**入力型**:
```
struct MovementQuery {
    product_code: String,
    date_from: Option<String>,        // YYYY-MM-DD（業務日）
    date_to: Option<String>,          // YYYY-MM-DD（業務日）
    movement_type: Option<MovementType>, // "sale_auto" | "sale_manual" | "receiving" | "return" | "disposal" | "stocktake"
    page: u32,
    per_page: u32,
}
```

**出力型**:
```
struct StockHistoryItem {
    kind: StockHistoryKind,           // "movement" | "count"
    business_date: String,            // movement の業務日（Z004 は精算日）か計数の日付
    settlement_no: Option<i64>,       // Z004 の movement の精算回数と、fresh な計数の保存の時点の精算回数
    saved_at: String,                 // movement の created_at か計数の counted_at
    movement: Option<MovementRecord>, // kind が movement のときだけ Some
    count: Option<StockCountHistory>, // kind が count のときだけ Some
}
struct StockCountHistory {
    id: i64,
    count_value: i64,
    stock_unit: ProductStockUnit,     // 計数の商品の今の products.stock_unit
    count_kind: StockCountKind,       // "default" | "fresh"（[42](42-cmd-sales-stocktake.md) の生成型）
    stocktake_id: Option<i64>,        // 計数が参照する棚卸しの明細の棚卸し
    source: Option<MovementSourceLink>, // stocktake_id があるときだけ Some
}
struct MovementRecord {               // 形は変えない（業務記録の詳細の movements と同じ型）
    id: i64,
    product_code: String,
    movement_type: MovementType,
    quantity: i64,
    stock_after: i64,
    reference_type: Option<ReferenceType>,
    reference_id: Option<i64>,
    source: Option<MovementSourceLink>,
    note: Option<String>,
    created_at: String,
}
struct MovementSourceLink {
    label: String,
    route: String,
}
```

- 出力: `kind` は generated literal union（D-061）。`settlement_no` は Z004 の movement の精算回数と、fresh な計数の保存の時点の精算回数（無ければ None）で、ほかは None（画面が「日付」の列に「（精算 {n}）」を付けるのは Z004 の movement の行だけで、fresh な計数の `settlement_no` は並びの鍵にだけ使う。[66](66-ui-stock-movements.md)）。`saved_at` は movement の `created_at` か計数の `counted_at`（どちらも PC の時刻の `%Y-%m-%dT%H:%M:%S`）。`stock_unit` は計数の商品の今の単位で、画面は計数の値をこの単位で描く（商品情報の取得〈`get_stock_detail`〉だけが失敗しても、長さの商品の計数 450 を「4.5 m」と出せる。[66](66-ui-stock-movements.md) の UI-06c-D3 の部分障害）。計数の `source` は label「棚卸し #{id}」、route `/stocktake/records/{id}`（§23.10 の source の解決の棚卸しの行と同じ）。`MovementRecord` の形は変えない（並びの鍵は履歴の行が持つ）。`total_count` は絞り込みの後の movement と計数の合計。
- 並び: ADR SPEC-STK-CNT-D1 の鍵 `(business_date, phase, settlement_no, after_settlement)` の降順（新しい順）。movement は、Z004（`settlement_no` を持つ）が `(d_n, 1, n, 0)`、初期在庫（`reference_type` が NULL で `movement_type` が `receiving`。ADR D9）が `(D, 0, NULL, -1)`、ほかが `(D, 0, NULL, 0)`。計数は、既定が `(D, 0, NULL, 0)`、fresh が `(T, 1, n, 1)`（n が NULL なら 3 つ目を -1 とみなす）。鍵の同じ行は、`saved_at`、movement を計数より前、id の順を昇順に足して全順序にし、全体を逆にする。鍵は表示の順にだけ使い、計数に入っているかを示さない（ADR D1）。`stock_after` は保存した時点の写しのまま並べ直さない（65）。
- 絞り込み: `date_from`・`date_to` は `business_date` の両端を含む範囲で、movement と計数の両方に当てる（並びと同じ軸。保存の時刻では絞らない）。BIZ が値を [31](31-biz-inventory-service.md) の `validate_business_date` で確かめ、外れたら `ValidationFailed`（何も読まない）。`movement_type` が None なら movement と計数の全部、`stocktake` なら `movement_type = 'stocktake'` の movement と計数（計数は棚卸しの数の記録）、ほかの値ならその種類の movement だけ（計数を含めない）。`is_voided = 1` の movement は出さない。計数は消さない記録なので全部出す。ページの検査（page ≥ 1、1 ≤ per_page ≤ 200）は BIZ。
- 合成の置き場: BIZ（§23.10 の `list_movements`）。IO は §23.9 の 2 つの読取り（どちらも `pub(crate)` で、並べない）を持つ。BIZ が鍵で並べ、件数を数え、ページを切り、`source` を補う。初期在庫の印の SQL の式は `inventory_repo` の 1 つの定数にし、求め直しの読取り（[21](21-io-inventory-repo.md) の `list_ledger_movements_for_product`）と共有する。理由: 鍵の後ろ 2 つ（fresh な計数の `after_settlement` と n が NULL のときの -1、初期在庫の -1）は ADR D1 の規則で、SQL の `ORDER BY` に写すと同じ規則を Rust と SQL の 2 か所に持つ（ADR の Rejected Options の「同じ規則を SQL と Rust の 2 か所に書く危険」と同じ理由）。1 商品の履歴を全部読んでから並べるので、読む量は 1 商品の行の数に比例する（単店舗の 1 商品で数千行までを見込む。越えて遅くなれば、鍵を列にした SQL の並べ替えへ移す）。

**処理ステップ**:
1. state.db.lock() でDB接続を取得（`&conn`）
2. biz::inventory_service::list_movements(&conn, &query) を呼ぶ
3. Ok → PaginatedResult をそのまま返す
4. Err(BizError) → CmdError に変換して返す

**設計判断 — is_voided=0 の暗黙フィルタ**: IO層の movement の読取りで `WHERE is_voided = 0` を常に付与する。ロールバック済みの変動は在庫照会画面に表示しない。

**設計判断 — source link の責務**: IO層は movement の `reference_type/reference_id` と計数の `stocktake_id` を取得し、BIZ層が `source` を解決する。既知の `reference_type` は `label` と `route` を返す。`reference_type` または `reference_id` が NULL、または未知の値の場合は movement 行を残したまま `source=None` にする。棚卸しの明細を参照しない計数は `source=None`。

**enum 契約化（D-061 / D-064）**: `movement_type` / `reference_type` は DB 層 enum（MovementType/ReferenceType）を generated enum として wire 露出する（reference_type は Option）。値・wire 表現・分岐意味論は不変で、不明な reference_type のみ REQ-303 に従って None へ fallback する。`StockHistoryKind` も generated enum（`"movement" | "count"`）。

**入力例**:
```json
{
  "query": {
    "product_code": "4976383262108",
    "date_from": "2026-03-01",
    "date_to": "2026-03-31",
    "movement_type": null,
    "page": 1,
    "per_page": 50
  }
}
```

**出力例**:
```json
{
  "items": [
    {
      "kind": "movement",
      "business_date": "2026-03-23",
      "settlement_no": 102,
      "saved_at": "2026-03-23T19:45:00",
      "movement": {
        "id": 42,
        "product_code": "4976383262108",
        "movement_type": "sale_auto",
        "quantity": -3,
        "stock_after": 14,
        "reference_type": "csv_import",
        "reference_id": 3,
        "source": {
          "label": "CSV取込み #3",
          "route": "/csv-import/records/3"
        },
        "note": null,
        "created_at": "2026-03-23T19:45:00"
      },
      "count": null
    },
    {
      "kind": "count",
      "business_date": "2026-03-23",
      "settlement_no": null,
      "saved_at": "2026-03-23T20:10:00",
      "movement": null,
      "count": {
        "id": 4,
        "count_value": 14,
        "stock_unit": "pcs",
        "count_kind": "default",
        "stocktake_id": 2,
        "source": {
          "label": "棚卸し #2",
          "route": "/stocktake/records/2"
        }
      }
    }
  ],
  "total_count": 8,
  "page": 1,
  "per_page": 50
}
```

---

### 23.9 CMD-06用 IO層新規関数

CMD-06の3コマンドが呼び出すIO層関数。既存のリポジトリファイルに追加する。

#### product_repo::get_stock_detail

**関数要求**: 商品の在庫詳細を、最終入庫日・最終販売日付きで取得する

**シグネチャ**:
```
fn get_stock_detail(
    conn: &DbConnection,
    product_code: &str,
) -> Result<StockDetail, DbError>
```

**SQL概要**:
```sql
SELECT p.*, d.name AS dept_name, s.name AS supplier_name,
       (SELECT MAX(rr.receiving_date)
        FROM receiving_items ri
        JOIN receiving_records rr ON ri.receiving_record_id = rr.id
        WHERE ri.product_code = p.product_code) AS last_receiving_date,
       (SELECT MAX(sr.sale_date)
        FROM sale_records sr
        WHERE sr.product_code = p.product_code AND sr.is_voided = 0) AS last_sale_date
FROM products p
LEFT JOIN departments d ON p.department_id = d.id
LEFT JOIN suppliers s ON p.supplier_id = s.id
WHERE p.product_code = ?1
```

**エラー**: 商品が見つからない場合は `DbError::NotFound` を返す。

---

#### product_repo::list_low_stock_products

**関数要求**: 在庫が閾値以下の商品を一覧取得する

**シグネチャ**:
```
fn list_low_stock_products(
    conn: &DbConnection,
    count_units: &[ProductStockUnit],
    threshold_count: i64,
    length_units: &[ProductStockUnit],
    threshold_length: i64,
    include_discontinued: bool,
) -> Result<Vec<ProductWithRelations>, DbError>
```

**SQL概要**:
```sql
SELECT p.*, d.name AS dept_name, s.name AS supplier_name
FROM products p
LEFT JOIN departments d ON p.department_id = d.id
LEFT JOIN suppliers s ON p.supplier_id = s.id
WHERE ((p.stock_unit IN (…count_units) AND p.stock_quantity <= ?)   -- 個数の単位、threshold_count
    OR (p.stock_unit IN (…length_units) AND p.stock_quantity <= ?))  -- 長さの単位、threshold_length
  [AND p.is_discontinued = 0]  -- include_discontinued=false の場合
ORDER BY p.stock_quantity ASC, p.name ASC
```

2 つの `IN (…)` の code は bind で渡す。並びは BIZ が `ProductStockUnit::ALL` を数量の種類で分けて作って渡し（個数の 9 個の code と、`m`）、repo は受けた並びで SQL を組むだけ（db の層は数量の種類を知らない）。どちらにも入らない単位が在庫少から漏れないことを test で止める（D-113、[共通規則](10-common-rules.md) SPEC-UNIT-D10）。

**ページング不要の理由**: 閾値以下の商品は通常少数（数十件程度）。4000商品中、在庫少は一覧で全件表示する（architecture/ui-task-specs.md UI-06b仕様）。

---

#### inventory_repo::list_history_movements / list_history_counts

**関数要求**: 商品別の在庫変動の履歴の入力（movement と計数）を絞り込みの後で読む。並べない（並び・件数・ページは §23.10 の BIZ）

**シグネチャ**（どちらも `pub(crate)`。BIZ の `list_movements` だけが呼ぶ）:
```
fn list_history_movements(
    conn: &DbConnection,
    query: &MovementQuery,
) -> Result<Vec<HistoryMovementRow>, DbError>   // MovementRecord（source は None）・business_date・settlement_no・initial_stock

fn list_history_counts(
    conn: &DbConnection,
    query: &MovementQuery,
) -> Result<Vec<HistoryCountRow>, DbError>      // id・count_value・fresh・business_date・settlement_no・counted_at・stock_unit・stocktake_id
```

**SQL概要**:
```sql
SELECT id, product_code, movement_type, quantity, stock_after, reference_type,
       reference_id, note, created_at, business_date, settlement_no,
       reference_type IS NULL AND movement_type = 'receiving'   -- 初期在庫の印（inventory_repo の 1 つの定数）
FROM inventory_movements
WHERE product_code = ?1 AND is_voided = 0
  AND (?2 IS NULL OR business_date >= ?2) AND (?3 IS NULL OR business_date <= ?3)   -- date_from・date_to
  AND (?4 IS NULL OR movement_type = ?4)                                             -- movement_type

SELECT sc.id, sc.count_value, sc.count_kind, sc.business_date, sc.settlement_no,
       sc.counted_at, p.stock_unit, si.stocktake_id
FROM stock_counts sc
JOIN products p ON p.product_code = sc.product_code
LEFT JOIN stocktake_items si ON si.id = sc.stocktake_item_id
WHERE sc.product_code = ?1
  AND (?2 IS NULL OR sc.business_date >= ?2) AND (?3 IS NULL OR sc.business_date <= ?3)
```

計数を含めるか（`movement_type` が None か `stocktake` のときだけ）は BIZ が決め、含めないときは `list_history_counts` を呼ばない。

**MovementQuery / MovementRecord / MovementSourceLink 型定義**: inventory_repo に配置。CMD層は biz/mod.rs 経由で re-export されたものを使用する。`MovementRecord.source` は BIZ層で付与するため、IO層のSQL select対象には含めない。`StockHistoryItem`・`StockHistoryKind`・`StockCountHistory` は BIZ（`inventory_service`）に置く（`count_kind` が BIZ の生成型 `StockCountKind` のため）。

---

### 23.10 BIZ層 listラッパー

CMD-02〜06のlist操作で使用するBIZ層ラッパー。既存の `csv_import_service::list_csv_imports`（list.rs）と同じパターン。

#### inventory_service に追加する関数

```
// 入庫一覧
fn list_receivings(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<ReceivingRecordWithSupplier>, BizError>

// 返品一覧
fn list_returns(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<ReturnRecordSummary>, BizError>

// 廃棄一覧
fn list_disposals(
    conn: &DbConnection,
    query: &ListQuery,
) -> Result<PaginatedResult<DisposalRecordSummary>, BizError>

// 入出庫履歴ハブ
fn list_inventory_records(
    conn: &DbConnection,
    query: &InventoryRecordQuery,
) -> Result<PaginatedResult<InventoryRecordSummary>, BizError>

// 公開シグネチャは不変。内部の record_type / status allowlist だけを 6 種・4 値へ拡張する

// 業務記録詳細
fn get_receiving_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ReceivingRecordDetail, BizError>

fn get_return_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ReturnRecordDetail, BizError>

fn get_manual_sale_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<ManualSaleRecordDetail, BizError>

fn get_disposal_record(
    conn: &DbConnection,
    record_id: i64,
) -> Result<DisposalRecordDetail, BizError>

// 在庫変動の履歴（movement と計数）
fn list_movements(
    conn: &DbConnection,
    query: &MovementQuery,
) -> Result<PaginatedResult<StockHistoryItem>, BizError>
```

各関数の処理:
1. ページパラメータのバリデーション（page >= 1, 1 <= per_page <= 200）
2. 対応するrepo関数を呼び出し
3. `DbError` → `BizError::DatabaseError` に変換して返す

`list_movements` は、ページの検査 → 日付の検査（`validate_business_date`）→ §23.9 の 2 つの読取り（計数は種別が None か `stocktake` のときだけ）→ 鍵で並べる（§23.8）→ 件数 → ページを切る → `source` を補う（movement は `reference_type/reference_id` から、計数は `stocktake_id` から棚卸しの行で）の順に処理する。source を解決できない行も一覧に残し（`source=None`）、一覧取得全体をエラーにはしない。

#### product_service に追加する関数

```
// 在庫詳細
fn get_stock_detail(
    conn: &DbConnection,
    product_code: &str,
) -> Result<StockDetail, BizError>

// 在庫少一覧
fn list_low_stock(
    conn: &DbConnection,
    include_discontinued: bool,
) -> Result<Vec<ProductWithRelations>, BizError>
```

list_low_stock の処理:
1. system_repo::get_setting(conn, "stock_low_threshold") で個数の単位の閾値を取得（デフォルト3）
2. system_repo::get_setting(conn, "stock_low_threshold_fabric") で長さの単位の閾値（cm）を取得（デフォルト500）
3. `ProductStockUnit::ALL` を数量の種類で個数と長さの 2 つの並びに分ける
4. product_repo::list_low_stock_products(conn, count_units, threshold_count, length_units, threshold_length, include_discontinued) を呼び出し

---

### 23.11 非目的

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|
| 入力バリデーション（空items、不正日付等） | BIZ層の責務 | BIZ-02 inventory_service |
| 在庫変動処理（movements記録と、記録からの写しの求め直し） | BIZ層の責務 | BIZ-02 inventory_service |
| 冪等性チェック（同一キーの重複リクエスト検知） | BIZ層の責務 | BIZ-02 inventory_service |
| PLU登録済み警告の判定 | BIZ層の責務 | BIZ-02 inventory_service |
| 閾値の取得と適用 | BIZ層の責務 | BIZ-01 product_service |
| ページパラメータ上限チェック | BIZ層の責務 | 各BIZ listラッパー |

---

### 23.12 対応不変条件

| 不変条件 | 本モジュールでの対応 |
|---------|-----------------|
| INV-1: quantity符号変換 | BIZ層に委譲。CMD層は符号変換に関与しない |
| INV-2: movements記録と写しの求め直しの同時性 | BIZ層のトランザクション内で保証 |
| INV-3: 冪等性（idempotency_key） | BIZ層に委譲。CMD層はキーをそのまま中継 |
| INV-5: 在庫マイナス警告（エラーではなく警告） | BIZ層が stock_warnings に含めて返す |
| INV-8: products物理DELETE禁止 | CMD層はproductsのDELETE操作を持たない |

---

### 23.13 lib.rs へのコマンド登録

```
// lib.rs の invoke_handler に追加
.invoke_handler(tauri::generate_handler![
    // CMD-01（既存）
    ...
    // CMD-02: 入庫
    cmd::receiving_cmd::create_receiving,
    cmd::receiving_cmd::list_receivings,
    // CMD-03: 返品・交換
    cmd::return_cmd::create_return,
    cmd::return_cmd::list_returns,
    // CMD-04: 手動販売出庫
    cmd::manual_sale_cmd::create_manual_sale,
    // CMD-05: 廃棄・破損
    cmd::disposal_cmd::create_disposal,
    cmd::disposal_cmd::list_disposals,
    cmd::disposal_cmd::list_inventory_records,
    cmd::receiving_cmd::get_receiving_record,
    cmd::return_cmd::get_return_record,
    cmd::manual_sale_cmd::get_manual_sale_record,
    cmd::disposal_cmd::get_disposal_record,
    // CMD-06: 在庫照会
    cmd::inventory_cmd::get_stock_detail,
    cmd::inventory_cmd::list_low_stock,
    cmd::inventory_cmd::list_movements,
    // CMD-07〜11（既存）
    ...
])
```
