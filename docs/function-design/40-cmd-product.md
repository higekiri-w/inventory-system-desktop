## 5. CMD-01: 商品コマンド群

### 計数の置き場と在庫の求め直しの契約（D-118。回復型は b1、商品の結果の写しの 3 列は b2 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md) の拒否と回復先を、message の解析なしで伝える。下の 1〜4 つ目の項目（`stocktake_guard` と回復型）は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で generated enum / bindings と同時に入った（§5.3 の kind は 13 値）。最後の段落（商品の結果の写しの 3 列）は b2（[Plan Packet](../plans/2026-10-23-stocktake-count-runtime-b2.md)）で bindings と同時に入った。旧 ADR（時点証拠）の計数 context の失効・保存先の変更・取込みの保留・取消の保留・準備照会の issue・非連動化の未調整（`sync_disabled_recovery`）の code と型は持たない（置き換えた）。

- CmdErrorKind へ `stocktake_guard` を加え、CmdError へ `stocktake_recovery: Option<StocktakeRecovery>` を加える。既存の kind では None、`stocktake_guard` では Some を必須とする。既存の field / error_id・internal の sanitize・restore の分類は変えない。`BizError::StocktakeGuard { message, recovery }` を CMD が変換するだけで、対象の選別や回復の可否を CMD で判定しない。
- StocktakeRecovery は `code` と `targets` を持つ。code は generated enum: `recount_required`（在庫連動を始める拒否。[30](30-biz-product-service.md)）/ `completion_blocked`（棚卸しの完了の拒否。[35](35-biz-stocktake-service.md)）。共有 JAN の在庫連動の拒否は商品フォーム・一括 import の入力の検査なので、回復型に載せず既存の `validation` kind（field `pos_stock_sync`、[30](30-biz-product-service.md)）で返す。Z004 の取込みの拒否（同じ精算の別版・精算回数の無い file・精算の順の食い違い）は対象の商品を持たないので回復型に載せず、既存の `import_error` kind と BIZ の message で返す（[32](32-biz-csv-import-service.md)）。StocktakeRecovery は `settlement_pending: Option<SettlementPendingHint>` も持つ（b1 の Plan Review round 1 の是正、2026-10-22。D-127 の「直し方を名指す」: 画面が「精算 103 を取り込んでください」「精算日が 10-04 以降の精算を取り込んでください」と出せるように）。`completion_blocked` で `block_reason` が `settlement_pending` の target が 1 つでもあるときだけ Some、ほかは None。SettlementPendingHint は `missing`（35 の `settlement_pending` の (ii) に当たった、取り込まれていない精算回数を連続した区間にまとめた一覧。要素は [32](32-biz-csv-import-service.md) の欠けの知らせと同じ `SettlementGap { from, to }`、昇順）、`on_or_after`（(i) に当たった明細の効く計数の日付 X の最大。精算日がこの日以降の精算を取り込めば (i) は全て解ける。当たらなければ null）、`before`（(iii) に当たった明細の X の最小。精算日がこの日より前の精算を取り込めば (iii) は全て解ける。当たらなければ null）を持つ。日付は `YYYY-MM-DD` の文字列、区間の端は 32 の順 1 の範囲の整数。
- targets の要素 CountRecoveryTarget は `product_code` / `product_name` / `stock_unit`（ProductStockUnit）/ `stock`（写しの在庫。i64）/ `stock_range_low`・`stock_range_high`（写しが要再確認のときの範囲の両端。Option、両方 Some か両方 None）/ `stocktake_item_id`（Option）/ `stocktake_id`（Option）/ `action` / `block_reason`（Option）/ `recount_reasons` を持つ（`stock_unit`・`stock`・範囲は b1 の Plan Review round 1 の是正、2026-10-22。取込みの結果・取消の結果・棚卸しの画面の要再確認の一覧〈[55](55-ui-csv-import.md)・[73](73-ui-stocktake.md)〉が「範囲」を単位どおりに出すのに要り、商品の読取りに写しの範囲が入るのは b2 なので、ここで運ぶ。値は target を作った TX の求め直しの後の写し）。action は `active_count`（進行中の棚卸しの明細で数える）/ `count_now`（棚卸しの外で今の数を入れる。明細が無い商品、進行中の棚卸しが無いとき）。`block_reason` は `completion_blocked` のときだけ Some で、`recheck`（要再確認）/ `uncounted`（未計数）/ `settlement_pending`（決まらない精算が未取込み）の generated enum。`recount_reasons` は要再確認の対象のとき、その商品の理由（`undetermined_settlement` / `same_date_movement` / `conflicting_counts` の generated enum。重複なし、この順）を入れ、他では空にする。参照なしは null で表し、架空の ID を使わない。
- 負数・非整数・範囲外は既存の validation を使う。UI は kind / code / action / block_reason を生成型で網羅し、未知の code を「そのまま続行」にしない。

商品の get / search の結果（`ProductWithRelations`。`get_product`・`search_products` の要素・`list_low_stock` の要素・`get_stock_detail` の `product`）は、写しの `stock_recheck: bool`・`stock_range_low: Option<i64>`・`stock_range_high: Option<i64>` を flatten した商品の field と同じ階層に持つ（在庫数の欄の範囲の表示に使う。[50](50-ui-product-list.md)・[51](51-ui-product-form.md)・[58](58-ui-stock-inquiry.md)・[66](66-ui-stock-movements.md)）。範囲は要再確認のときだけ両端が値（DB の CHECK）。`Product`（PLU の書出しと JAN の照合が読む行の写像）には足さない（wire に出ないため）。

b1 で共通 enum、全 CmdError constructor、BizError の変換、bindings、CMD_ERROR_KIND、describeError と feature の回復の分岐・mock / test を同じ変更で同期した。追加 payload に raw SQL・診断ログ・file 本体を載せない。

### 5.1 モジュール構成

```
src-tauri/src/
  cmd/
    mod.rs
    product_cmd.rs  -- 商品関連のTauriコマンド
```

### 5.2 CMD層のDB接続取得方針

BIZ層の関数が `&mut DbConnection` を要求するため、CMD層では `Mutex<Connection>` を AppState に保持し、
`.lock()` で可変借用を取得する。poison 時は `CmdError { kind: CmdErrorKind::Internal }` に変換する（unwrap() は使わない）。

```
// AppState 内
db: Mutex<Connection>

// CMD 関数内
let mut conn = state.db.lock().map_err(|_| CmdError { kind: CmdErrorKind::Internal, message: "DB接続エラー", field: None })?;
biz::product_service::create_product(&mut conn, req)?;
```

### 5.3 CMD層のエラー返却方針

TauriコマンドはUI向けのエラー種別に正規化して返す。String一本ではなく構造化されたエラー型を使い、UI側でエラー種別に応じた分岐（バリデーションエラーならフィールドハイライト、重複エラーなら特定メッセージ等）を可能にする。

```
struct CmdError {
    kind: CmdErrorKind,  // generated enum（D-061。下記の値一覧。POS連携追加分の由来は41-cmd-pos.md 17.4）
    message: String,  // 利用者向け日本語メッセージ（rawな技術詳細を含めない。CMD-ERR-D2）
    field: Option<String>,  // バリデーションエラー時のフィールド名
    error_id: Option<String>,  // 診断ログ相関ID（CMD-ERR-D1）。internal / restore_*系でのみSome
    stocktake_recovery: Option<StocktakeRecovery>,  // 棚卸しの守りの拒否の回復先（`stocktake_guard` だけ Some。冒頭の D-118 の節、b1）
    related_csv_import_id: Option<i64>,  // 拒否の文が名指す取込みID（Z004 の順 4 の拒否〈kind import_error〉だけ Some、ほかは null。TS では任意の field。41-cmd-pos.md 17.4）
}
```

**kind の enum 契約（D-061）**: `CmdErrorKind` は response 専用のため `#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]` + `#[serde(rename_all = "snake_case")]` の generated enum とし、`Deserialize` は付与しない。wire 表現は従来の snake_case 文字列と 1:1 完全一致（値・分岐・error_id 相関は不変）。値は 13 個（D-118 の b1 で `stocktake_guard` を足した）: `validation` / `duplicate` / `not_found` / `internal` / `import_error` / `export_error` / `idempotency_conflict` / `stocktake_in_progress` / `stocktake_not_in_progress` / `stocktake_guard` / `restore_failed_recovered` / `restore_failed_unrecoverable` / `restore_durability_unknown`。frontend は bindings 由来の literal union で分岐し、`CMD_ERROR_KIND` は生成 union に対する exhaustive 検査付きの定数 map とする。独自の手動 union は持たない。

**エラーID契約（CMD-ERR-D1）**: raw な失敗詳細を wire に載せず診断ログ側にのみ記録する kind（`internal` と `restore_failed_recovered` / `restore_failed_unrecoverable` / `restore_durability_unknown`）は、生成時に `error_id`（形式 `E-<YYYYMMDD-HHMMSS>-<4hex>`。chrono ローカル時刻 + uuid v4 短縮、新規依存なし）を発行し、response と `tracing::error!` の両方に同一値を載せる。利用者は画面に表示された error_id と診断ログ（[70-mnt-diagnostic-log.md](70-mnt-diagnostic-log.md)）の同一 error_id で事象を突合できる。日次ローテーションのログファイル特定を助けるため時刻併記形式とする。他の kind は None。

**internal の sanitize 契約（CMD-ERR-D2）**: `CmdError::internal(user_message, detail)` は
- `message` には操作文脈の日本語定型文言のみを載せる。raw な error Display（SQLite メッセージ、OS error、ファイルパス等）を `message` に混ぜてはならない（`format!("...: {}", e)` パターンの禁止）
- `detail`（raw な失敗詳細）は error_id とともに `tracing::error!` で診断ログへ記録する
- 旧記載の「操作ログID（operation_logs）」による相関は実装と乖離していたため、相関キーは error_id × 診断ログに確定した（D-053）

BizError → CmdError の変換ルール:
- BizError::ValidationFailed(msg) → CmdError { kind: CmdErrorKind::Validation, message: msg, field: None }
- BizError::ValidationFailedAt { message, field } → CmdError { kind: CmdErrorKind::Validation, message, field: Some(field) }
- BizError::DuplicateProductCode(code) → CmdError { kind: CmdErrorKind::Duplicate, message: "この商品コードは既に使用されています: {code}" }
- BizError::JanInUse(message) → CmdError { kind: CmdErrorKind::Duplicate, message, field: Some("jan_code") }（[30](30-biz-product-service.md) BIZ-01-D11。商品フォームは kind と field で分け、message を解析しない。`DuplicateProductCode` は field を持たないので、UI は field `jan_code` の有無で 2 つを分ける）
- BizError::NotFound(msg) → CmdError { kind: CmdErrorKind::NotFound, message: msg }
- BizError::DatabaseError(_) → CmdError { kind: CmdErrorKind::Internal, message: "データベースエラーが発生しました。もう一度お試しください", error_id: Some(発行) }
- BizError::StocktakeGuard { message, recovery } → CmdError { kind: CmdErrorKind::StocktakeGuard, message, stocktake_recovery: Some(recovery) }（冒頭の D-118 の節、b1）
- POS連携追加分（ImportError / SettlementConflict / IdempotencyConflict / ExportError）は 41-cmd-pos.md 17.4 を参照

### 5.4 各コマンドの関数仕様

CMD層は薄いラッパー。型変換→BIZ呼び出し→エラー正規化のみ。

#### create_product コマンド

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
#[specta::specta]
fn create_product(state: State<AppState>, req: ProductCreateRequest) -> Result<ProductCreateResult, CmdError>
```

**処理ステップ**:
1. state.db.lock() で接続を取得
2. biz::product_service::create_product(conn, req) を呼ぶ
3. Ok → ProductCreateResultをそのまま返す
4. Err(BizError) → CmdErrorに変換して返す

#### update_product コマンド

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
#[specta::specta]
fn update_product(state: State<AppState>, product_code: String, req: ProductUpdateRequest) -> Result<ProductUpdateResult, CmdError>
```

**部分更新wire契約（PRODUCT-PATCH-D1 / REQ-102）**:

- 通常field（`name`, `department_id`, `selling_price`, `cost_price_centi`, `tax_rate`, `pos_stock_sync`, `plu_target`）は、JSON omitted / nullのどちらも「更新しない」、valueは「設定する」。
- clear可能field（`supplier_id`, `maker_code`）は、JSON omittedが「更新しない」、nullが「clearする」、valueが「設定する」。
- generated commandの入力型は `ProductUpdateRequest_Deserialize` とし、全propertyをoptionalにする。frontend側の `Partial` aliasや型assertionでgenerated契約を迂回しない。
- command名、引数名、response、`CmdError`、BIZ呼出し順は変更しない。

**処理ステップ**: create_productと同パターン。biz::product_service::update_product()を呼ぶ

#### toggle_discontinue コマンド

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
#[specta::specta]
fn toggle_discontinue(state: State<AppState>, product_code: String) -> Result<bool, CmdError>
```

**処理ステップ**: biz::product_service::toggle_discontinue()を呼ぶ

#### search_products コマンド

**処理ステップ**: biz::product_service::search_products()を呼ぶ。PLU の移行 filter（`ProductSearchQuery.plu`）は D-117 (2) で撤去した（runtime の lane が DTO・bindings・UI と一緒に外す）。`supplier_id` / `include_unassigned` は UI-14 の取引先 filter としてそのまま BIZ-01 wrapper を通し、抽出判断は IO-01 が所有する（SPEC-PRVB-D1）。

#### list_departments コマンド

**関数要求**: UI-01a / UI-01b の部門選択候補として、部門マスタ全件を取得する。

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
#[specta::specta]
fn list_departments(state: State<AppState>) -> Result<Vec<Department>, CmdError>
```

**処理ステップ**:
1. state.db.lock() で接続を取得
2. biz::product_service::list_departments(&conn) を呼ぶ
3. Ok → `Vec<Department>` をそのまま返す
4. Err(BizError) → CmdErrorに変換して返す

**設計判断**: UI-01a の部門フィルタは、検索結果の現在ページから候補を派生しない。候補は departments 初期データ 21 件を正とし、`product_repo::list_departments` を BIZ 経由で公開する。

#### list_suppliers コマンド

**関数要求**: UI-01b の取引先選択候補として、取引先マスタ全件を取得する。

**シグネチャ（Tauriコマンド）**:
```
#[tauri::command]
#[specta::specta]
fn list_suppliers(state: State<AppState>) -> Result<Vec<Supplier>, CmdError>
```

**処理ステップ**:
1. state.db.lock() で接続を取得
2. biz::product_service::list_suppliers(&conn) を呼ぶ
3. Ok → `Vec<Supplier>` をそのまま返す
4. Err(BizError) → CmdErrorに変換して返す

**設計判断（SPEC-PRV-D6）**: UI-01b / UI-14 の取引先候補は complete master data から取得する。初回 UI-01b で保留した `find_or_create_supplier` の公開境界は、本 Design Phase で `create_supplier` CMD として消化する。

#### revise_product_price コマンド（SPEC-PRV-D5 / REQ-105）

```rust
#[tauri::command]
#[specta::specta]
fn revise_product_price(
    state: State<AppState>,
    input: PriceRevisionInput,
) -> Result<PriceRevisionResult, CmdError>
```

```rust
struct PriceRevisionInput {
    product_code: String,
    new_selling_price: i64,
    new_cost_price_centi: i64,
    assign_supplier_id: Option<i64>,
}

struct PriceRevisionResult {
    product_code: String,
    changed: bool,
    plu_dirty_set: bool,
    supplier_assigned: bool,
}
```

上限（D-113・D-128）: `new_selling_price` は 0 以上 `999999`、`new_cost_price_centi` は 1/100 円で 0 以上 `99999999`（`999999.99` 円）（越えれば BIZ が `売価が大きすぎます`・`原価が大きすぎます`）。下の `PriceHistoryEntry` の原価も 1/100 円の `old_cost_price_centi`・`new_cost_price_centi`。[共通規則](10-common-rules.md) SPEC-UNIT-D5、[30](30-biz-product-service.md) BIZ-01-D8。

CMD は DB 接続を取得し、BIZ-01 `revise_product_price` を呼んで結果をそのまま返す。負値は validation、不存在 `product_code` / `assign_supplier_id` は not-found の既存 `CmdErrorKind` へ正規化し、価格更新・履歴・操作ログ・supplier 紐付けの判断を CMD に持たない。

#### create_supplier コマンド（SPEC-PRV-D6 / REQ-106）

wire contract: `create_supplier(name: String) -> Result<Supplier, CmdError>`。

```rust
#[tauri::command]
#[specta::specta]
fn create_supplier(
    state: State<AppState>,
    name: String,
) -> Result<Supplier, CmdError>
```

BIZ-01 が name を trim し、空文字を validation error とする。同名が既にあれば既存 `Supplier { id, name }` を返す。UI-01b と UI-14 は成功後に `list_suppliers` を再取得する。

#### rename_supplier コマンド（SPEC-SUP-D3 / REQ-107）

wire contract: `rename_supplier(supplier_id: i64, name: String) -> Result<Supplier, CmdError>`。

```rust
#[tauri::command]
#[specta::specta]
fn rename_supplier(
    state: State<AppState>,
    supplier_id: i64,
    name: String,
) -> Result<Supplier, CmdError>
```

CMD は DB 接続を取得して `biz::product_service::rename_supplier` を呼び、結果をそのまま返す。空文字・同名衝突は `validation`、不存在 ID は `not_found` の既存 `CmdErrorKind` へ正規化する。trim、同値 no-op、`updated_at`、transaction、operation_log の判断を CMD に持たない。

#### merge_suppliers コマンド（SPEC-SUP-D4 / D8 / REQ-107）

wire contract: `merge_suppliers(source_id: i64, target_id: i64) -> Result<SupplierMergeResult, CmdError>`。

```rust
#[tauri::command]
#[specta::specta]
fn merge_suppliers(
    state: State<AppState>,
    source_id: i64,
    target_id: i64,
) -> Result<biz::product_service::SupplierMergeResult, CmdError>
```

`SupplierMergeResult { products_updated: i64, receiving_records_updated: i64 }` は BIZ-01 が所有し、CMD は qualified path で参照する（SPEC-PRVA-D5 準拠）。`source_id == target_id` は `validation`、どちらかの不存在は `not_found` に正規化する。2 UPDATE + DELETE、1 transaction、operation_log の判断を CMD に持たない。

#### list_suppliers_with_usage コマンド（SPEC-SUP-D8 / REQ-107）

wire contract: `list_suppliers_with_usage() -> Result<Vec<SupplierWithUsage>, CmdError>`。

```rust
#[tauri::command]
#[specta::specta]
fn list_suppliers_with_usage(
    state: State<AppState>,
) -> Result<Vec<biz::product_service::SupplierWithUsage>, CmdError>
```

`SupplierWithUsage { id: i64, name: String, product_count: i64, receiving_record_count: i64 }` は BIZ-01 が所有し、CMD は qualified path で参照する。CMD は DB 接続を取得して同名の BIZ wrapper を呼び、name 昇順の結果をそのまま返す。

3 command は実装 PR で `collect_commands!` / `generate_handler!` に登録し、`#[tauri::command]` / `#[specta::specta]` を対で付けて bindings を再生成する。既存 `list_suppliers` / `create_supplier` の wire contract は変更しない。

#### list_price_history コマンド（SPEC-PRV-D9 / REQ-102）

```rust
#[tauri::command]
#[specta::specta]
fn list_price_history(
    state: State<AppState>,
    product_code: String,
    limit: u32,
) -> Result<Vec<PriceHistoryEntry>, CmdError>
```

```rust
struct PriceHistoryEntry {
    id: i64,
    old_selling_price: i64,
    new_selling_price: i64,
    old_cost_price_centi: i64,
    new_cost_price_centi: i64,
    changed_at: String,
}
```

`changed_at DESC, id DESC` で返す。limit は既定 10・上限 100（超過は 100 に丸める）。`product_code` 不存在時は not-found error ではなく空配列を返す。

#### get_product コマンド

**処理ステップ**: product_repo::find_by_product_code()を呼ぶ。None → CmdError { kind: CmdErrorKind::NotFound }。応答型 `ProductWithRelations`（[20-io](20-io-product-repo.md) の JOIN 結果。`41-cmd` の `ProductResponse` は PLU dirty 一覧用の別型）は memory No. を含めない（CMD-01-D3 の `plu_memory_no` は D-117 (5) で撤去。runtime の lane が DTO・bindings・UI と一緒に外す）。

#### bulk_set_plu_target コマンド（CMD-01-D4。撤去の対象、D-117 (2)）

商品一覧の絞り込みに一致する全件の PLU 対象化（段階導入の道具）は撤去する（[30 §4.9.1](30-biz-product-service.md)）。runtime の lane が code・specta の登録・bindings・UI・test と一緒に消す。設計照合 test（`src-tauri/tests/design_compliance_test.rs`）が code の pub fn と突き合わせるので、撤去までは signature だけを残す。

```rust
#[tauri::command]
#[specta::specta]
fn bulk_set_plu_target(
    state: State<AppState>,
    filter: ProductBulkFilter,
    plu_target: bool,
) -> Result<BulkPluTargetResult, CmdError>
```
