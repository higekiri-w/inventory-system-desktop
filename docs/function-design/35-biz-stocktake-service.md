## 20. BIZ-06: 棚卸しロジック

### 計数の置き場と在庫の求め直しの契約（D-118。b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D3・D5・D6）の BIZ-06 の側。b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で、旧い数の上書きと在庫への上書きの確定・force_fill・自動の 0 の入力を本節に置き換え、停止 ADR の入口を外した。以下の §20.2〜§20.10 は本節の現行の契約。旧 ADR（時点証拠）の計数 context（begin・token・版・DB 世代・source_cursor・再送の照会・abandon）・独立再実測の表・明細の kind（uncounted / measured / auto_filled / legacy）・force_fill・legacy の取消の保留・非連動化の回復・拒否の回復型の表（SPEC-STK-TIME-D1-R1）は持たない（置き換えた。D-109 で `#[cfg(test)]` に置いた時点証拠の code は b1 で外した）。

#### 計数の保存（SPEC-STK-CNT-D5）

`save_stock_count(tx, SaveStockCountRequest) -> Result<StockCountSaveResult, BizError>`。

- 入力: `product_code`、`count_value`（0 以上の整数。長さの商品は cm。上限は [共通規則](10-common-rules.md) の在庫の上限 `999999999`）、`fresh: bool`（「入れ直す」なら true、「数を保存」なら false）、`business_date`（YYYY-MM-DD。`fresh=false` のとき使う。UI の既定は PC の今日、「前の日の分を入れる」で選んだ日。`fresh=true` では today を使い、入力の値と違えば `ValidationFailed`）、`stocktake_item_id: Option<i64>`（棚卸しの画面から保存したとき）、`today`（CMD が渡す PC の今日。BIZ は時計を読まない）。
- 保存の形（[tracking](../db-design/tracking-system-tables.md) の `stock_counts`）: `fresh=false` は `count_kind='default'`。`fresh=true` は `count_kind='fresh'`、`business_date=today`、`settlement_no` = 同じ TX で読んだ active な Z004 の取込みの最大の精算回数（取込みが無ければ NULL。0 は実在しうる精算回数なので使わない。ADR D1 の並びの鍵の n）、`seen_movement_id` = 同じ TX で読んだ `MAX(inventory_movements.id)`（無ければ 0）、`seen_import_id` = 同じ TX で読んだ `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL。[24](24-io-csv-import-repo.md)。ADR D3 の判定の n を求める）。`business_date` が today より後なら `ValidationFailed`（「未来の日付には保存できません」）。
- 1 商品 1 TX で `stock_counts` へ INSERT し、[31](31-biz-inventory-service.md) の `recompute_stock` で写しを書き、`stocktake_item_id` があれば明細の `actual_count`・`counted_at`（表示用の写し）を効く計数に合わせる。明細が進行中の棚卸しのものでないとき、明細の商品が `product_code` と違うとき（同じ TX で確かめる）は、書く前に `ValidationFailed`（「この明細は別の商品です」）。
- 結果: 保存した計数の id・種類、求め直しの後の在庫（写しの値と範囲）、要再確認の理由（[40](40-cmd-product.md) の `recount_reasons` と同じ enum）、`later_count_exists: bool`。
- 後入力の確認（ADR D5）: 既定の計数の保存の前に UI が `preview_stock_count_placement(conn, product_code, business_date, today) -> CountPlacementPreview`（読取りだけ）を呼び、ADR D3 の「計数どうし」でその計数より後の計数が既にあるか（`later_count_exists`）を受ける（「後の数が既にあり、そちらが効きます」。保存は拒まない）。同じ結果は計数の欄の商品の文脈（商品名・単位・写しの在庫と範囲・要再確認の理由・効く計数の値と種類と日付。[42](42-cmd-sales-stocktake.md) の表）も持ち、棚卸しの明細の無い商品を「今の数を入れる」で開く画面（[73](73-ui-stocktake.md)）が読む（b1 の Plan Review round 1 の是正）。
- 計数は消さない・書き換えない（誤った数は「入れ直す」で直す。fresh な計数は、それより先に保存された計数を全て置き換える。ADR D3）。再送の照会・冪等の key は持たない（同じ値の既定の計数がもう 1 行入っても、重なる計数で値が同じなら 1 つとみなす）。

#### 開始と明細

- `start_stocktake` は今のまま進行中の棚卸しが無ければ header と明細（全ての商品。`system_stock` は作成時の帳簿、`actual_count` は NULL）を作る。棚卸し中の商品登録・商品一括 import は明細を足す（今のまま）。進行中があれば `StocktakeInProgress`、対象 0 件は `ValidationFailed`（今の文言）。
- 明細の「計数済み」は、その明細を参照する計数（`stock_counts.stocktake_item_id`）があること。

#### 完了（SPEC-STK-CNT-D6）

`complete_stocktake(tx, CompleteStocktakeRequest { stocktake_id, today })` は、TX の中で次を検査し、1 つでも当たれば何も書かずに `StocktakeGuard`（[40](40-cmd-product.md) の code `completion_blocked`、明細ごとの target と `block_reason`。`settlement_pending` があれば、取り込めば解ける精算を 40 の `SettlementPendingHint`〈欠けた精算回数の区間、(i) の日付の最大、(iii) の日付の最小〉で添える。b1 の Plan Review round 1 の是正）を返す。

| block_reason | 条件 |
|---|---|
| `settlement_pending` | 明細の効く計数（日付 X。fresh なら ADR D3 の判定の n〈`seen_import_id` 以下の取込みID を持つ、今 active な Z004 の取込みの最大の精算回数。無ければ NULL。b1 の Plan Review closure〈Codex〉の是正〉）について、(i) 精算日が X 以上の取り込み済みの精算が無い、または (ii) 取り込まれていない精算回数 k（fresh で n が NULL でなければ k > n）で、隣の取り込み済みの精算 a < k < b の精算日が d_a ≤ X ≤ d_b のものがある、または (iii) 既定の計数（と判定の n が NULL の fresh な計数）で、精算日が X より前の取り込み済みの精算が無い |
| `recheck` | 明細の商品の写しが要再確認（`stock_recheck=1`） |
| `uncounted` | 明細を参照する計数が無い。ただし、その時点で廃番かつ在庫（写し）が 0 の商品は対象外 |

- message: 「棚卸しを完了できません。数え直し・未計数・取り込んでいない精算のある商品があります」。UI は `block_reason` ごとに一覧を出す。その日の最後の精算を取り込んでいないことは判定できない（ADR D6 の H2）ので、UI の完了の確認の文で確かめる（[73](73-ui-stocktake.md)）。
- 未計数を片付ける操作: 「数を入れる」は通常の計数の保存。「0 にする」は `count_value=0`・`fresh=false` の計数の保存で、`business_date` は UI が選んだ日（既定は今日、今日より後は `ValidationFailed`。ADR D6）（`zero_uncounted_items(tx, stocktake_id, product_codes, business_date, today)` でまとめて保存できる。1 TX、商品ごとに求め直す）。各計数は進行中の棚卸しの当該明細を参照する（`stocktake_item_id` に明細の id を入れる。参照しない計数では明細が「計数済み」にならず、完了が `uncounted` で拒み続ける）。選んだ商品のどれかが進行中の棚卸しの明細を持たなければ、何も書かずに `ValidationFailed`（「この商品は棚卸しの対象にありません: {商品コード}」）（b1 の Plan Review round 2 の是正）。帳簿の数とみなす操作（force_fill）は持たない。
- 通れば、各明細の `valuation_cost_price_centi` にその時点の原価を、`valuation_quantity` に評価数量 `max(写しの在庫, 0)` を固定し、[§20.5a](#205a-評価額の計算価格の基準数量と店の丸め) の関数（SPEC-STK-VAL-D1〜D6）で `total_cost` を求め、stocktakes の完了の時点のカーソル（`cutoff_import_id`〈`MAX(csv_imports.id)`。取り消したものを含み、取込みが無ければ NULL〉・`cutoff_count_id`・`cutoff_movement_id`〈`MAX(inventory_movements.id)`、無ければ 0〉）を保存し、header を completed にする。完了は在庫を動かさず、movement を作らない。数量と積和は checked 演算。
- 完了した評価額は、後から入った記録（棚卸しの期間の日付を選んだ入出庫・遅れて取り込んだ Z004・完了の後に保存した計数〈前の日の分を入れたものを含む〉）で変えない（非遡及）。差は示す（ADR SPEC-STK-CNT-D6、owner の決定 TD-296）: `get_stocktake_record` が、明細ごとに「完了の時点の在庫」を [31](31-biz-inventory-service.md) の `derive_stock` と同じ規則で今の記録から求め直す。範囲は、完了の評価に入れた記録（完了の時点にあった記録。日付によらない）に、完了の後に入れた完了の前の入出庫・初期在庫・精算の記録を足したもの（計数は除く: 完了の後に保存した計数は、完了の前の日付を選んだものも次の観測で、完了の時点の在庫を置き換えない。b1 の Plan Review closure の是正）: 計数は id ≤ `cutoff_count_id` だけ。入出庫と初期在庫の movement（`phase = 0`）は id ≤ `cutoff_movement_id` か業務日 ≤ 完了日のもの（完了の前に保存した記録は `valuation_quantity` にも入っているので差に効かず、完了の後に入れた完了日以前の日付の記録が差になる）。精算は精算回数 ≤ 完了の時点の精算の n（`cutoff_import_id` 以下の取込みID を持つ、今 active な取込みの最大の精算回数。無ければ NULL で、この項は空。ADR D3 の判定の n と同じ求め方で、完了の後の取込み〈取り込み直した版・初期化の後の新しい系列〉の番号を数えない。b1 の Plan Review closure〈Codex〉の是正）のもの（精算日によらない。完了の後に埋めた欠けを含む）と精算日 < 完了日のもの（b1 の Plan Review round 3 の是正: 前は精算と入出庫を業務日で切り、完了の前に取り込んだ未来の日付の精算〈レジの時計のずれ〉と完了の前に入れた未来の日付の入出庫が、評価に入るのに比較から外れ、後の記録が無いのに差を出した）。その `max(…, 0)` と `valuation_quantity` を、凍結した `valuation_cost_price_centi` でそれぞれ §20.5a の関数で評価し、丸めた後の総額を差し引いて参考の差とする（負の数量を関数に渡さない。読取りで求め、保存しない）。求め直した完了の時点の在庫が要再確認（範囲 lo〜hi）の明細は、差を確定の差と分け、「要再確認」の印と差の範囲（`max(lo,0)` と `max(hi,0)` をそれぞれ `valuation_quantity` と比べた幅、評価額も同じ 2 端で評価した幅）で示し、確定の差の合計に入れない（決まらない記録による差を確定の数として出さない）。差のある明細の一覧（確定の差と要再確認の差を分ける）と確定の差の合計を返す。完了した `total_cost`・`valuation_cost_price_centi`・`valuation_quantity` は書き換えない。

#### 読取り

- 明細の一覧は、明細ごとに効く計数の値・種類・日付、写しの在庫と範囲、要再確認の理由、差異（効く計数の値 − その計数に入っている記録だけから求めた帳簿。計数が無ければ NULL）を返す。差異は表示用で、在庫を動かさない。
- 要再確認の一覧 `list_recheck_products(conn) -> Vec<CountRecoveryTarget>`（`stock_recheck=1` の商品。進行中の棚卸しに明細があれば action `active_count` と `stocktake_item_id`、なければ `count_now`）。棚卸しが無いときも読める（取込みの結果と同じ一覧を棚卸しの画面に出す。[73](73-ui-stocktake.md)）。一覧からの保存は fresh な計数で、`stocktake_item_id` があれば渡す（明細の `actual_count`・`counted_at` を置き換わった計数に揃える）。

b1 で足した関数のシグネチャ（`pub`。CMD-10 が呼び、`today` は CMD が渡す PC の今日。[42](42-cmd-sales-stocktake.md)）:

```rust
pub fn save_stock_count(conn: &mut DbConnection, req: &SaveStockCountRequest, today: &str) -> Result<StockCountSaveResult, BizError>
pub fn preview_stock_count_placement(conn: &DbConnection, product_code: &str, business_date: &str, today: &str) -> Result<CountPlacementPreview, BizError>
pub fn zero_uncounted_items(conn: &mut DbConnection, stocktake_id: i64, product_codes: &[String], business_date: &str, today: &str) -> Result<Vec<StockCountSaveResult>, BizError>
pub fn list_recheck_products(conn: &DbConnection) -> Result<Vec<CountRecoveryTarget>, BizError>
```

#### 操作ログ（b1 の Plan Review round 2 の是正）

開始と完了は今のとおり COMMIT の後（TX の外）に 1 行ずつ書き、書けなければ警告だけにする。計数の保存と「0 にする」は操作ログを書かない（今の数の入力と同じ。計数は `stock_counts` に残る）。

| operation_type | summary | detail_json の key |
|---|---|---|
| `stocktake_start` | 「棚卸しを開始しました（対象: {item_count}件）」（今のまま） | `stocktake_id`・`item_count`（`auto_filled_count` を外す。自動の 0 の入力をやめた） |
| `stocktake_complete` | 「棚卸しを確定しました（差異: {difference_count}件、仕入原価総額: ¥{total_cost}）」（形は今のまま） | `stocktake_id`・`total_cost`・`total_items`・`difference_count`（`adjusted_count`・`force_fill_used` を外す。補正の movement と force_fill は無い） |

- `difference_count` は、完了の時点の明細のうち、差異（上の「読取り」の差異を完了の時点のカーソルで求めた値。[65](65-inventory-record-traceability.md) の記録の詳細の差異と同じ値）が null でなく 0 でない明細の数。今の summary の「差異」は補正の movement を作った明細の数で、どちらも「数えた数が帳簿と違った商品の数」の意味を保つ（完了は在庫を動かさないが、差異は読取りで求まる）。
- 記録済みの旧い detail（`adjusted_count`・`force_fill_used`・`auto_filled_count` を持つ行）は書き換えない。[74](74-ui-operation-logs.md) は operation_type の label だけを読み、detail の key を読まない。

### 20.1 モジュール構成

```
src-tauri/src/
  biz/
    mod.rs                   -- pub mod stocktake_service を追加
    product_service.rs       -- 既存（BIZ-01）
    inventory_service/       -- 既存（BIZ-02、ディレクトリモジュール）
    csv_import_service.rs    -- 既存（BIZ-03）
    plu_export_service.rs    -- 既存（BIZ-04）
    stocktake_service.rs     -- 棚卸しの業務ロジック（本セクション）
    stock_ledger.rs          -- 記録からの求め直し（BIZ-02、31。計数の保存・0 にする・読取りが呼ぶ）
```

単一ファイル。分けるときは `design_compliance_test.rs` の 35 の行に module を足す。

---

### 20.2 型定義

wire に出る型の field は [42](42-cmd-sales-stocktake.md) の D-118 の節と b1 の packet の Boundary が正。ここでは BIZ の型の役割だけを書く。

- `StartStocktakeResult`: stocktake_id: i64、item_count: usize（作った明細の数。自動の 0 の入力はしない）
- `SaveStockCountRequest`・`StockCountSaveResult`・`CountPlacementPreview`: 計数の保存と後入力の確認（本書冒頭の「計数の保存」）
- `CompleteStocktakeRequest`: stocktake_id: i64、today: String（CMD が渡す PC の今日）
- `StocktakeResult`: total_cost: i64（§20.5a）、total_items: usize（明細の数）、integrity_result: Option\<IntegrityResult\>（D-2: 完了後の整合性チェック結果。失敗時は None）。在庫を動かさないので補正の一覧は持たない
- `StocktakeItemDetail`（一覧と検索の明細）: IO の行（[20](20-io-product-repo.md) §2.11 の `StocktakeItemRow`）に、効く計数（`count_value`・`count_kind`・`count_business_date`。衝突のときは写しの土台の計数）、要再確認の理由（`recount_reasons`）、差異（`difference`。本書冒頭の「読取り」）を足した BIZ の型
- `StocktakeProgress`（BIZ では `StocktakeProgressBiz`）: stocktake_id、status、total_items、counted_items（明細を参照する計数がある明細の数）、uncounted_items
- `StocktakeRecordDetail`・`StocktakeRecordDetailItem`・`AfterCompletionDiff`・`AfterCompletionDiffItem`: §20.6a

---

### 20.3 start_stocktake

**関数要求**: 新しい棚卸しを開始し、全対象商品の棚卸し明細を自動生成する

**シグネチャ**:
```
fn start_stocktake(
    conn: &mut DbConnection,
) -> Result<StartStocktakeResult, BizError>
```

**処理ステップ**:

1. **進行中チェック**（TX外）
   - stocktake_repo::find_active_stocktake(conn)
   - Some → BizError::StocktakeInProgress("進行中の棚卸しがあります（ID: {id}、開始日: {started_at}）。完了してから新しい棚卸しを開始してください")
2. **対象商品の取得**（TX外）
   - stocktake_repo::find_stocktake_eligible_products(conn) を呼び出し（全商品を返す。IO層ではフィルタなし）
   - 0件 → BizError::ValidationFailed("棚卸し対象の商品がありません")
   - 戻り値: Vec\<ProductForStocktake\> { product_code, stock_quantity, cost_price_centi, is_discontinued }
3. **TX開始**（conn.transaction()。RAII Drop で自動 ROLLBACK）
4. **棚卸しヘッダINSERT**
   - stocktake_repo::insert_stocktake(&tx, now) → stocktake_id
   - started_at = 現在日時、status = "in_progress"
5. **棚卸し明細の一括生成**
   - 各商品について stocktake_repo::insert_stocktake_item(&tx, &NewStocktakeItem { stocktake_id, product_code, system_stock: product.stock_quantity, actual_count: None })（自動の 0 の入力はしない。廃番かつ在庫 0 の明細は未計数のまま完了を止めない。本書冒頭の「完了」）
6. **COMMIT**（tx.commit()）
7. **TX外: 操作ログ記録**
   - system_repo::insert_operation_log(conn, &NewOperationLog { operation_type: "stocktake_start", summary: "棚卸しを開始しました（対象: {item_count}件）", detail_json: Some(detail_json) })
   - detail_json: { "stocktake_id": stocktake_id, "item_count": item_count }
   - 操作ログ記録失敗は警告のみ（業務処理のcommitは完了済み）
8. StartStocktakeResult { stocktake_id, item_count } を返す（item_count = 対象商品の件数）

**エラーハンドリング**:
- 進行中の棚卸しが存在 → BizError::StocktakeInProgress
- 対象商品0件 → BizError::ValidationFailed
- DB操作失敗 → RAII自動ROLLBACK → BizError::DatabaseError(DbError)

**設計判断 — 棚卸しカウント対象の母集団（issue #91 owner 回答 2026-08-22）**:
- `find_stocktake_eligible_products` が返す母集団は商品マスタに登録済みの全商品である（`is_discontinued` を問わない）。実店舗の除外基準は年数ではなく原価根拠の有無で、伝票保管義務範囲外で廃棄済み・取引先データなし・バーコードなし・販売に適さない見た目という原価根拠を欠く現物は、単品コードを付与しない = 商品マスタへ登録しないことで自然に母集団から外れる。規模は例年 1〜2 点、多い年で 4〜5 点。
- システム側で除外品を表現する専用フラグや状態は追加しない（owner 方針）。部門キーで商品非連動販売として扱い、現物が再流通する場合は新規商品として改めて登録する。

**設計判断 — 明細一括生成のパフォーマンス**:
- 最大4000件の INSERT を1TX内で実行する。SQLite の WAL モード + 単一接続で十分高速（数百ms以内）
- 一括INSERT文（VALUES多値）は SQLite の SQL長制限とデバッグ困難性のため不採用。1件ずつ insert_stocktake_item を呼ぶ方式

**入力例**: なし（引数なし）

**出力例**:
```
Ok(StartStocktakeResult {
    stocktake_id: 1,
    item_count: 3847,
})
```

---

### 20.3.1 get_stocktake_items

**関数要求**: 棚卸し明細一覧と進捗を取得する。pagination の下限条件は
BIZ層が所有し、CMD層は引数をそのまま渡す
（**BIZ-06-VAL-D1**）

**シグネチャ**:
```
fn get_stocktake_items(
    conn: &DbConnection,
    stocktake_id: i64,
    department_id: Option<i64>,
    counted_only: Option<bool>,
    page: u32,
    per_page: u32,
) -> Result<(PaginatedResult<StocktakeItemDetail>, StocktakeProgress), BizError>
```

**処理ステップ**:
1. page < 1 → `BizError::ValidationFailedAt { message: "ページ番号は1以上で指定してください", field: "page" }`
2. per_page < 1 → `BizError::ValidationFailedAt { message: "1ページあたりの件数は1以上で指定してください", field: "per_page" }`
3. `stocktake_repo::list_stocktake_items` と `stocktake_repo::get_stocktake_progress` を呼ぶ
4. 明細一覧と進捗を返す

per_page の上限は既存どおりIO層が D-031
`PAGINATION_MAX_PER_PAGE = 200` でクランプする。下限 validation の移設で
上限・クランプ意味論は変更しない。

---

### 20.4 save_stock_count・preview_stock_count_placement・zero_uncounted_items・list_recheck_products

契約は本書冒頭の「計数の保存」「完了」（未計数を片付ける操作）「読取り」、シグネチャは同じ節の code block。

**処理ステップ**（`save_stock_count`。検査の順は b1 の packet の S4）:

1. 形と暦（[31](31-biz-inventory-service.md) の `validate_business_date`）→ 今日との比べ（既定は今日以前、fresh は今日と同じ）→ 値の範囲（0 以上 `999999999`）。外れれば `ValidationFailed`（何も書かない）
2. TX 開始。`stocktake_item_id` があれば `find_stocktake_item_with_parent_status` で明細が進行中の棚卸しのもので商品が同じかを確かめる（違えば `ValidationFailed`、書く前）
3. fresh なら同じ TX でカーソル（active な Z004 の最大の精算回数、`max_movement_id`、`max_csv_import_id`）を読む
4. `insert_stock_count` → `recompute_stock`（[31](31-biz-inventory-service.md)）→ 明細があれば `update_stocktake_item_count`（表示用の写し）
5. COMMIT。結果は求め直しの後の写しと理由と `later_count_exists`。操作ログは書かない（頻度が高い。棚卸しの開始と完了だけ記録する）

`zero_uncounted_items` は書く前に全ての商品の明細を確かめ（1 つでも無ければ `ValidationFailed`）、1 TX で商品ごとに値 0 の既定の計数を保存して求め直す。`preview_stock_count_placement`・`list_recheck_products` は読取りだけで何も書かない。

---

### 20.5 complete_stocktake

**関数要求**: 棚卸しを完了する。完了を拒む理由（本書冒頭の「完了」の表）を全部調べてから拒み、通れば評価数量・評価原価・総額・カーソルを固定する。在庫を動かさず、movement を作らない

**シグネチャ**:
```
fn complete_stocktake(
    conn: &mut DbConnection,
    req: &CompleteStocktakeRequest,
) -> Result<StocktakeResult, BizError>
```

**処理ステップ**:

1. 棚卸しの存在確認と状態チェック（None → `NotFound`、完了済み → `StocktakeNotInProgress`）
2. TX 開始。`list_items_for_completion` で明細を読み、`settlement_pending`・`recheck`・`uncounted` を全て調べる。1 つでも当たれば何も書かずに `BizError::StocktakeGuard`（code `completion_blocked`、target ごとの `block_reason`、`settlement_pending` があれば `SettlementPendingHint`）
3. 各明細の `valuation_cost_price_centi` に今の原価、`valuation_quantity` に `max(写しの在庫, 0)` を `update_stocktake_item_valuation` で固定し、§20.5a の関数で商品別の金額を求める
4. `valuation_total_yen` の後に `check_safe_integer`（外れれば `ValidationFailed`「合計金額が大きすぎます」。TX 全体が戻る）
5. 完了の時点のカーソル（`max_csv_import_id`・`max_stock_count_id`・`max_movement_id`）を同じ TX で読み、`stocktake_repo::complete_stocktake` で header と一緒に保存する
6. COMMIT。TX 外で操作ログ（本書冒頭の「操作ログ」）と整合性チェック（D-2 統合。失敗しても完了を戻さず integrity_result = None）
7. StocktakeResult { total_cost, total_items, integrity_result } を返す

**設計判断 — operation_log TX外**: 第4段階の先決事項 D-6 のとおり、操作ログは業務の COMMIT の後に書き、書けなくても業務を戻さない。

**設計判断 — valuation_cost_price_centi のタイミング**: 完了の時点の products.cost_price_centi を使う（棚卸しは数か月に及び、その間に原価が変わる。税理士報告の仕入原価総額は完了の時点の原価）。DB_DESIGN.md の「確定時にproducts.cost_price_centiの値をコピーしてくる」と同じ。

**設計判断 — total_cost のオーバーフロー対策**: 中間（原価〈1/100 円〉 × 数量）と 1/100 円の合計は i128 で checked に計算し、最後の円額だけを i64 へ検査付きで変換し、wire の number で正確に運べるよう安全な整数の範囲を検査する（§20.5a SPEC-STK-VAL-D2、共通規則「入力の上限と安全な整数の範囲」の (c)）。

---

### 20.5a 評価額の計算（価格の基準数量と店の丸め）

契約 ID: SPEC-STK-VAL-D1〜D6（2026-09-25。Backlog「棚卸しの評価額が価格の基準数量を持たない」「評価額の丸めを店の規則に合わせる」起源）。完了（§20.5 の手順 3〜4）と記録の詳細の行のロス原価・完了の後の差（§20.6a）は、同じ関数で評価額を求める。関数は `stocktake_service` の非公開関数とする。

**シグネチャ**:
```
fn price_basis_quantity(unit: ProductStockUnit) -> i64
fn valuation_line_centi(cost_price_centi: i64, quantity: i64, basis: i64, product_code: &str) -> Result<i128, BizError>
fn valuation_total_yen(lines: &[i128]) -> Result<i64, BizError>
```

**処理ステップ**:

1. 各明細で、確定時評価原価（valuation_cost_price_centi）・評価数量・商品の在庫単位の基準数量から、商品別の金額（1/100 円、i128）を `valuation_line_centi` で求める
2. `valuation_total_yen` が商品別の金額を i128 の checked_add で合計する
3. 同じ関数が合計を円未満で四捨五入し、円額を i64 へ検査付きで変換して total_cost（円）とする

- **SPEC-STK-VAL-D1 価格の基準数量**: 商品の selling_price（円）/ cost_price_centi（1/100 円）は、在庫数量で「価格の基準数量」ぶんに対する価格である。基準数量は在庫単位で決まる: 個数の単位 = 1（1 個あたり）、`m` = 100（1 m あたり。在庫は cm の整数）。商品ごとの列は持たない。`price_basis_quantity` は `ProductStockUnit` の全 variant を網羅する match とし、wildcard arm を置かない（単位を足すと compile error になり、基準数量を決めずに単位を足せない）。
  - 理由: 店は長さ商品の残り・仕入れ伝票・値札をすべて m で扱い、値札は 1 m あたりである。レジでも数量 1 = 1 m で打てる（未運用）。出典は [project-memory](../project-memory.md) の Store Premises Facts（2026-09-14 / 2026-09-15）。在庫は cm の整数で持つ（[31](31-biz-inventory-service.md) の整数契約）。基準数量が単位で決まるため、利用者が商品ごとに基準数量を入力して誤る入口を作らない。
  - 不採用: 商品ごとの基準数量の列（migration・DTO・商品フォーム・商品 CSV の追加が要り、長さ商品ごとに 100 を入れる操作を利用者に任せる。今ある単位では単位と基準数量が一対一）。原価を 1 cm あたりの小数で持つ（伝票・値札の m 単価と食い違い、原価の列に小数の型が要る）。
  - 見直す条件: 単位の拡張で、単位だけでは基準数量が決まらない商品（箱で仕入れて 1 個ずつ売る商品の原価を箱あたりで記録する等）が店の回答で確かめられたとき。
- **SPEC-STK-VAL-D2 金額の表現**: 中間（原価〈1/100 円〉 × 数量、商、余り）・商品別の金額（1/100 円）・その合計は i128 で計算し、最後の円額だけを i64 へ検査付きで変換する。積と合計の加算は checked とする。原価が円の整数だった間（原価の lane の前）は円の原価を × 100 してから掛けた。`biz::unit_amount` の test の helper はこの形で 35 の円の golden を確かめる（共通規則 SPEC-UNIT-D6）。浮動小数（f32 / f64）を使わない。理由: 2 進の浮動小数は 0.005 の境界を正確に表せず、四捨五入を誤る（例: 1.005 は 1.00499… として保持され、小数第 2 位への四捨五入が 1.00 になる）。10 進小数の crate は追加しない（整数の分子と分母で正確に計算できる）。
- **SPEC-STK-VAL-D3 商品別の金額**: `valuation_line_centi`（実装は `biz::unit_amount::cost_line_centi`）は `cost_price_centi × quantity ÷ basis`（原価は 1/100 円）を 1/100 円未満で四捨五入した値を返す。四捨五入は割り算の正確な値に対して行う（商 q・余り r で `2r >= basis` なら q + 1）。これは店の「商品別の金額を小数第 3 位で四捨五入して小数第 2 位まで持つ」と同じ値になる。quantity は旧本体では actual_count、新方式では `max(補正後現在庫, 0)`。原価が 1/100 円なので、長さの商品（基準数量 100）では割り切れないことがあり、この四捨五入（SPEC-UNIT-D6 の行の金額の規則と同じ）が実際に効く（例: `8333 × 130 ÷ 100 = 10832.9` → `10833`）。個数の商品（基準数量 1）は割り切れる。
- **SPEC-STK-VAL-D4 総額**: `valuation_total_yen` は商品別の金額の合計（1/100 円）を円未満で四捨五入した円の整数を返す。`stocktakes.total_cost`（INTEGER、円）・StocktakeResult.total_cost（i64）・操作ログの「仕入原価総額: ¥{total_cost}」の型と表示は変えない。
- **SPEC-STK-VAL-D5 入力の検査**: cost_price_centi か quantity が負なら、`valuation_line_centi` が評価額を求めず `BizError::ValidationFailed("評価額を計算できません。原価か数量が負の値です（商品 {product_code}）")` を返す（検査と文言は関数の中の 1 か所。呼出し側は商品コードを渡すだけ）。中間の積か合計の加算が i128 を越える場合、または四捨五入した円額が i64 を越える場合は、既存の `BizError::ValidationFailed("仕入原価総額の計算でオーバーフローが発生しました")` を返す（i64 への変換の失敗もこの文言に固定する。既存の overflow test〈原価 `i64::MAX / 2 + 1`〈1/100 円として読む〉・数量 200〉は変換で初めて error になる）。どちらも確定の TX を ROLLBACK し、header・明細・在庫を変えない。呼出し元は `price_basis_quantity` の値（1 以上）を basis に渡す。
- **SPEC-STK-VAL-D6 非遡及**: 確定済みの total_cost と valuation_cost_price_centi は再計算しない（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md) SPEC-STK-CNT-D6 と同じ）。本契約より前に確定した記録の total_cost は旧式（原価 × 数量の整数積和）のまま残る。本契約は本番開始前に入るため、本番の記録に旧式は現れない（[初導入の前提](../project-memory.md)）。

**行のロス原価（b1）**: 棚卸し記録詳細の行のロス原価は BIZ が `valuation_line_centi` と同じ関数（|差異| × 評価原価 ÷ 基準数量、1/100 円未満を四捨五入）で求めた 1/100 円の十進の文字列で返し、画面は掛けずに `formatCentiYen` で出す（[65](65-inventory-record-traceability.md)、SPEC-UNIT-D6）。入庫の原価小計・原価合計、廃棄のロス原価、手動販売の金額の初期値は [共通規則](10-common-rules.md) SPEC-UNIT-D6・D7 が基準数量で割る。確定後に商品の在庫単位を変えると、記録詳細の数量表示とロス原価は現在の単位で計算される。

**単位の拡張（D-113）**: 単位は 10 個で、基準数量は単位から決まる（個数の 9 種 = 1、`m` = 100。[共通規則](10-common-rules.md) SPEC-UNIT-D1・D4）。本節の `price_basis_quantity`・`valuation_line_centi`・`valuation_total_yen` の計算は、`biz::unit_amount` の `price_basis_quantity`・`cost_line_centi`・`cost_total_yen` が同じ値で持つ（SPEC-UNIT-D6。原価の引数は 1/100 円で、呼ぶ側は確定時評価原価 `valuation_cost_price_centi` を `i128::from` でそのまま渡す。原価の lane、D-128。`決定（owner、D-113 J2）`）。商品別の金額は `原価(1/100 円) × 数量 ÷ 基準数量` を 1/100 円未満で四捨五入する。棚卸し記録詳細のロス原価は同じ関数を BIZ が使う（§20.6a）。棚卸しの実数の入力は、m の商品では m で受けて cm に直す（SPEC-UNIT-D3。[73](73-ui-stocktake.md)）。完了は `biz::unit_amount` の関数と商品の `stock_unit` で評価額を求める（D-118 の b1。補正の movement と note は作らない）。

---

### 20.6 get_stocktake_progress

**関数要求**: 現在の棚卸しの進捗状況を返す。棚卸し画面の進捗バー表示用

**シグネチャ**:
```
fn get_stocktake_progress(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<StocktakeProgress, BizError>
```

**処理ステップ**:

1. **棚卸しの存在確認**
   - stocktake_repo::find_stocktake_by_id(conn, stocktake_id)
   - None → BizError::NotFound("棚卸しが見つかりません: ID {stocktake_id}")
2. **進捗集計**
   - stocktake_repo::get_stocktake_progress(conn, stocktake_id) → StocktakeProgress { total_items, counted_items, uncounted_items }
3. StocktakeProgress { stocktake_id, total_items, counted_items, uncounted_items } を返す（status は stocktake から取得して付与）

**エラーハンドリング**:
- 棚卸しが見つからない → BizError::NotFound
- DB読み取り失敗 → BizError::DatabaseError(DbError)

---

### 20.6a get_stocktake_record

完了は補正の movement を作らない（D-118、b1）。明細の差異は本書冒頭の新契約の「読取り」の定義（効く計数の値 − その計数に入っている記録だけから求めた帳簿）で、完了した棚卸しは完了の時点のカーソルで求める。

D-118 の新方式は旧 header を持たない（棚卸しのある DB は作り直す。ADR SPEC-STK-CNT-D10）ので、header に移行の版を足さない。過去評価額を再算出しない。

**関数要求**: 棚卸し記録詳細を wire DTO として返す。読取りだけで何も書かない（[65](65-inventory-record-traceability.md) slice 4c の棚卸し詳細画面）

**シグネチャ**:
```
fn get_stocktake_record(
    conn: &DbConnection,
    stocktake_id: i64,
) -> Result<StocktakeRecordDetail, BizError>
```

**StocktakeStatus enum**（新設、BIZ-06 所有。D-061 有限 IPC 値の generated union）:
- InProgress（`in_progress`）/ Completed（`completed`）の 2 値。DB CHECK（[tracking-system-tables.md](../db-design/tracking-system-tables.md) 16-17）により 2 値保証
- 既存 `Stocktake` wire 型の `status: String` は既存 command の wire 互換のため変更しない（本 DTO のみ enum を用いる）

**StocktakeRecordDetail構造体**（wire DTO。BIZ-06 が所有する）:
- id: i64, started_at: String, completed_at: Option\<String\>, status: StocktakeStatus, total_cost: Option\<i64\>（棚卸し時原価総額。in_progress は None）
- item_count: i64（対象商品数）
- items: Vec\<StocktakeRecordDetailItem\>（明細ごとの効く計数〈完了した棚卸しは完了の時点のカーソルの効く計数、進行中は今の効く計数〉の値・種類・日付、差異、評価数量、評価原価、行のロス原価〈差異が負で評価原価があるときの 1/100 円の十進の文字列、§20.5a〉）
- after_completion: Option\<AfterCompletionDiff\>（完了していなければ None。本書冒頭の「完了」の後半: 確定の差と要再確認の差を分け、確定の差の合計を円の十進の文字列で）

**処理ステップ**:
1. stocktake_repo::get_stocktake_record_detail(conn, stocktake_id) を呼ぶ。IO 層の NotFound は「棚卸し記録が見つかりません」を含む BizError::NotFound、その他の IO エラーは BizError::DatabaseError に変換する
2. header.status の raw TEXT を StocktakeStatus へ変換する（DB CHECK により 2 値保証のため、想定外値は BizError::DatabaseError で fail-fast）
3. 明細ごとに記録を読み、完了した棚卸しはカーソルで切った記録から効く計数と差異を求め、行のロス原価を求める
4. 完了した棚卸しは、完了の後の比較の範囲（本書冒頭の「完了」）で完了の時点の在庫を求め直し、差のある明細を確定の差と要再確認の差に分けて返す

**エラーハンドリング**:
- 棚卸し不存在 → BizError::NotFound（「棚卸し記録が見つかりません」）
- DB 読み取り失敗 / 想定外 status 値 → BizError::DatabaseError

**設計ノート（差異の定義）**: 詳細画面の「差異」は本書冒頭の「読取り」の差異（完了した棚卸しは完了の時点のカーソルで求める）。補正の movement は作らないので、その quantity は使わない。

**共有の判定（BIZ-03 の取消の前の読取り）**: 精算が完了の時点の入力・完了の後の比較の範囲に入るか（精算回数 ≤ 完了の時点の精算の n、または精算日 < 完了日）は 1 つの判定の関数で決め、手順 3・4 と `completed_stocktakes_comparing_settlement(conn, settlement)`（`pub(crate)`）が使う。後者は完了した棚卸しのうち、指定の精算が完了の後の比較の範囲に入るものを完了の日時の昇順で返す（範囲による候補で、記録の詳細のどの数が変わるかは見ない。[32](32-biz-csv-import-service.md) §15.6b の手順 3 が呼び、規則を写さない）。完了した棚卸しの一覧は [20](20-io-product-repo.md) §2.11a の `list_completed_stocktake_cutoffs`。

---

### 20.7 非目的

このモジュールが**やらないこと**を明示する。責務境界の誤解を防ぐため。

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|
| 棚卸し中のCSV取込み処理 | 棚卸し中もCSV取込みは許可（SP-205-09）。取込みは計数を持つ商品を求め直す（ADR D2） | BIZ-03 |
| 棚卸し中の新規商品登録時のstocktake_items自動追加 | 商品登録のTX内で実施済み | BIZ-01（create_product ステップ6） |
| 整合性チェック（stock_quantity突合） | 棚卸し確定後の自動実行は第5段階で統合 | BIZ-07（PR-5で実装） |
| 棚卸し画面の表示・フィルタ・ページング | UI層の責務 | UI-10 |
| 物理DELETE | 棚卸しの削除はサポートしない。completedのまま保持 | — |
| 棚卸しの「中止」（途中破棄） | 初期バージョンでは不要。中断→再開で対応 | 将来拡張 |
| 操作ログのカウント入力ごとの記録 | 頻度が高すぎて有用なログが埋もれる | — |
| 棚卸し明細の一覧取得の業務ルール | 一覧取得はBIZ層の `get_stocktake_items` が `stocktake_repo::list_stocktake_items` + 進捗集計を束ねる薄いwrapperとして提供（2026-04-13 `882cec6` でCMD直ラップからBIZ経由に変更、[42-cmd-sales-stocktake.md](42-cmd-sales-stocktake.md) §22.5 参照）。フィルタ・ページングの実体はIO層 | BIZ-06 wrapper + stocktake_repo |

---

### 20.8 対応不変条件

| 不変条件 | 本モジュールでの対応 |
|---------|-----------------|
| INV-2: stock_after算出責任 | 本モジュールは movement を作らない（計数は `stock_counts`、完了は在庫を動かさない）。写しは `recompute_stock` だけが書く（31） |
| INV-3: 負在庫ポリシー | 計数の値は 0 以上（Z1）。写しは記録から求め直し、負の在庫を受ける。評価数量は `max(写しの在庫, 0)` |
| INV-8: products物理DELETE禁止 | 本モジュールは products を書かない（写しは BIZ-02 の求め直し）。DELETE 操作なし |
| INV-1a: 入力値は常に正数 | 計数の値は 0 以上を `save_stock_count`・`zero_uncounted_items` の書く前に検査する（0 は有効） |

---

### 20.9 stocktake_repo への依存

使う IO 関数のシグネチャと型は [20](20-io-product-repo.md) §2.9〜§2.11a（stocktake_repo）、[21](21-io-inventory-repo.md)（計数の保存とカーソル）、[24](24-io-csv-import-repo.md)（精算の読取りと取込みのカーソル）が正。本書に写さない。

---

### 20.10 BizError 追加バリアント

BIZ-06 で新たに使用する BizError バリアント:

```
enum BizError {
    ValidationFailed(String),     // 既存
    ValidationFailedAt { message: String, field: String }, // pagination field を保持するBIZ validation
    NotFound(String),             // 既存 — 棚卸し/明細/商品の不存在
    DuplicateProductCode(String), // 既存（本モジュールでは不使用）
    DatabaseError(DbError),       // 既存
    ImportError(String),          // 既存（本モジュールでは不使用）
    IdempotencyConflict(String),  // 既存（本モジュールでは不使用）
    StocktakeInProgress(String),  // ← 新規追加: 進行中の棚卸しが既に存在
    StocktakeNotInProgress(String), // ← 新規追加: 棚卸しが既に完了済み
    StocktakeGuard { message: String, recovery: StocktakeRecovery }, // b1: 完了の拒否（completion_blocked）と在庫連動を始める拒否（recount_required。30）
}
```

**StocktakeInProgress**: start_stocktake で進行中チェック失敗時。CMD層では CmdError { kind: "stocktake_in_progress" } に変換

**StocktakeNotInProgress**: 計数の保存・0 にする・complete_stocktake で完了済み棚卸しへの操作時。CMD層では CmdError { kind: "stocktake_not_in_progress" } に変換

**StocktakeGuard**: complete_stocktake の完了の拒否。CMD層では CmdError { kind: "stocktake_guard", stocktake_recovery: Some(...) } に変換（[40](40-cmd-product.md)）

**ValidationFailedAt**: get_stocktake_items の page / per_page 下限違反時。
CMD層では `kind="validation"` と BIZ の message / field をそのまま保持する。

---

### 更新履歴

| 日付 | PR | 内容 |
|------|-----|------|
| 2026-04-12 | PR #21 | 初版作成（BIZ-06 stocktake_service 4関数 + stocktake_repo 12関数） |
| 2026-04-12 | PR #21 | 確定用の明細の読取りの型を 5フィールド（IO設計書版）→ 3フィールド（id/product_code/actual_count）に統一。BIZ設計書を採用した理由: complete_stocktake の処理ステップ5で必要なのは更新対象IDと商品コードと実カウントのみで、system_stock や counted_at は product_repo::find_by_product_code から取得する方が責務分離として正しい |
| 2026-08-27 | （本 PR） | §20.6a get_stocktake_record（棚卸し記録詳細 read、65 slice 4c）+ StocktakeStatus enum 新設 + 差異定義（補正 movement 正）の設計ノートを追加 |
| 2026-08-30 | docs 整合性衛生 batch（本 PR） | §20.3 に棚卸しカウント対象の母集団（issue #91 owner 回答 2026-08-22）を設計判断として追加 |
| 2026-09-24 | ㉘ runtime ①（本 PR） | §20.0 現行buildの一時停止（SPEC-STOP-D1〜D3、BIZ-06 停止文言の正本）を追加。§20.3〜§20.5 は旧本体 `legacy_*` の記述と明記 |
| 2026-09-25 | 棚卸しの評価額（本 PR） | §20.5a 評価額の計算（SPEC-STK-VAL-D1〜D6: 価格の基準数量、1/100 円の商品別の金額、円未満を四捨五入した総額）を追加。§20.2 / §20.5 と新方式の total_cost の式を §20.5a へ寄せる |
| 2026-10-23 | stocktake-count-runtime-b1（D-118 の b1） | 冒頭の D-118 の節を現行に。§20.0 の停止の節を外し、§20.2〜§20.5・§20.6a・§20.8〜§20.10 を計数の保存・0 にする・完了の拒否と評価額の固定・記録の詳細の新しい経路に置き換えた（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)） |
| 2026-10-25 | z004-rollback-entry（PR #197） | §20.6a に、完了の後の比較の範囲の判定を BIZ-03 の取消の前の読取りと共有する関数 `completed_stocktakes_comparing_settlement` を足した（規則は変えない。[Plan Packet](../plans/2026-10-23-z004-rollback-entry.md)） |
