# テーブル定義（在庫追跡・棚卸し・システム）

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D1・D2・D3・D9・D10）の保存の形。配布の単位 (a)（ADR SPEC-STK-CNT-D12）の列・表・index と移行は migration v9（[22 §19](../function-design/22-mnt-migration.md)、`src-tauri/src/db/schema_v9.rs`）で今の schema にある。b1 の分（下の各節に書く）は migration v11（[22 §20](../function-design/22-mnt-migration.md)、`src-tauri/src/db/schema_v11.rs`）で入った。

### 並びの鍵と計数（SPEC-STK-CNT-D1・D3）

(a) で入ったのは、`inventory_movements` の 3 列と phase・settlement_no の対の CHECK、`stock_counts` の表と index（v9）。`reference_type` と phase を結ぶ CHECK と、`stock_counts` へ保存する経路（計数の入力）は b1（migration v11 と BIZ-06）で入った。

| 保存先 | 足す列 | 制約と意味 |
|---|---|---|
| inventory_movements | business_date TEXT | NOT NULL、YYYY-MM-DD（CHECK: 長さ 10 で `[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]` に合う。窓の判定は日付を文字列の大小で比べるので、`2026-4-7` のような形を DB に入れない。実在する日付かを含む形と暦の検査は、書く側が [31](../function-design/31-biz-inventory-service.md) の「業務日の形と暦の検査」、移行が [22 §19](../function-design/22-mnt-migration.md) の手順 2 の各 1 か所）。入出庫は header の日付（`receiving_date`・`return_date`・`sale_date`・`disposal_date`）、Z004 は `csv_imports.settlement_date`、初期在庫（`reference_type` が NULL で `movement_type` が `receiving`。ADR D9）は登録日。業務記録の取消は今は無い（65）。足すときは、逆向きの movement に元の業務記録の日付を持たせる |
| inventory_movements | phase INTEGER | NOT NULL、0/1 の CHECK。Z004 の movement は 1、ほかは 0 |
| inventory_movements | settlement_no INTEGER | phase=1 なら NOT NULL（`csv_imports.settlement_no`）、phase=0 なら NULL（table の CHECK）。`reference_type='csv_import'` と phase=1 は同じ行で揃う（table の CHECK: `(reference_type IS 'csv_import') = (phase = 1)`。`=` で比べると `reference_type` が NULL のとき NULL になり CHECK が通るので、`IS` で比べる）。列と phase・settlement_no の対の CHECK は (a) の migration（[22 §19](../function-design/22-mnt-migration.md)）、`reference_type` と phase を結ぶ CHECK は (b) の最初の lane〈b1〉の migration〈[22 §20](../function-design/22-mnt-migration.md)〉が足す（ADR SPEC-STK-CNT-D12） |
| stock_counts（新設） | id INTEGER PK AUTOINCREMENT、product_code TEXT FK、count_value INTEGER、count_kind TEXT、business_date TEXT、settlement_no INTEGER、seen_movement_id INTEGER、stocktake_item_id INTEGER FK NULL、counted_at TEXT、seen_import_id INTEGER（b1 の migration〈[22 §20](../function-design/22-mnt-migration.md)〉が足す） | append-only（UPDATE・DELETE しない）。count_value は 0 以上の整数（在庫の数量の単位。長さは cm）。count_kind は `default`（「数を保存」「0 にする」）/ `fresh`（「入れ直す」）の CHECK。`fresh` なら seen_movement_id（保存の時点の `MAX(inventory_movements.id)`、無ければ 0。id は 1 から始まる）が NOT NULL で、settlement_no は保存の時点で取り込み済みの active な Z004 の最大の精算回数（取込みが無ければ NULL。精算回数 0 は実在しうるので 0 を印に使わない）で、ADR D1 の並びの鍵の n（計数に入っているかの判定には使わない）。seen_import_id は保存の時点の `MAX(csv_imports.id)`（取り消したものを含む。取込みが無ければ NULL。id は 1 から）で、ADR D3 の判定の n（seen_import_id 以下の取込みID を持つ、今 active な Z004 の取込みの最大の精算回数。無ければ NULL）を求め直しのたびに求める（保存しない）。判定の n が NULL の fresh な計数には、どの精算も「k ≤ n」に当たらない。`default` なら settlement_no・seen_movement_id・seen_import_id は NULL（table の CHECK: `count_kind='default'` なら settlement_no と seen_movement_id が NULL、`count_kind='fresh'` なら seen_movement_id が NOT NULL。seen_import_id は列の CHECK `seen_import_id IS NULL OR (count_kind = 'fresh' AND seen_import_id >= 1)`。movement のカーソルは 0 が「無い」で、取込みのカーソルは NULL が「無い」: 列を後から足すので既存の fresh の行に値を入れられず、NULL がその行の意味〈保存の時点に取込みが無かった〉と同じ。22 §20 の手順 5）。fresh の business_date は保存の日（今日）。stocktake_item_id は棚卸しの画面から保存した計数だけが持つ。counted_at は保存の時刻（表示用。窓の判定に使わない）。id は保存の順で、fresh な計数が先に保存された計数を置き換える判定と表示に使う。v9 の DDL は product_code・count_value・count_kind・business_date・counted_at を NOT NULL にし、business_date に inventory_movements.business_date と同じ形の CHECK（長さ 10 で `[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]` に合う）を置く |

- 鍵は ADR D1 の表（movement は `(business_date, phase, settlement_no, 0)`、初期在庫は 4 つ目を −1 とみなす、既定の計数は `(business_date, 0, NULL, 0)`、fresh な計数は `(business_date, 1, settlement_no, 1)` で settlement_no が NULL なら 3 つ目を −1 とみなす）。記録が計数に入っているかは鍵の大小を使わず ADR D3 の窓の判定表で決める。
- index: stock_counts の `(product_code, id)`（`idx_stock_counts_product_id`）、inventory_movements の `(product_code, business_date, phase, settlement_no)`（`idx_inventory_movements_ledger`）。

### 求め直しと写し（SPEC-STK-CNT-D2・D3）

(a) で入ったのは、この節の全部（求め直しは `biz::stock_ledger`）。精算 k の窓と要再確認の理由 1 は b1 で入った（ADR D12）。

- 写しは [master](master-tables.md) の `products.stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`。BIZ の求め直しの関数だけが書く（[31](../function-design/31-biz-inventory-service.md) の同名の節）。
- 求め方: 効く計数（ADR D3 の「計数どうし」）の値 + ADR D3 の判定表でその計数に「入っていない」`is_voided=0` の movement の quantity の和。計数が無ければ全ての `is_voided=0` の movement の和。決まらない記録は写しの値では適用し、範囲は適用する・しないの最小と最大。計数の衝突は、計数値が最小のものを写しの土台にする。
- `stock_after` は、その movement を保存した TX で求め直した写しの値（後入力で前に記録が入っても書き換えない）。在庫推移を鍵の順で求め直して見せる要求は持たない（[65](../function-design/65-inventory-record-traceability.md)）。
- 計数は movement を作らない（棚卸しの補正の movement〈`movement_type='stocktake'`〉は新しく作らない。既存の行は D10 の作り直しで無くなる）。

### 棚卸し（SPEC-STK-CNT-D6）

この節は全部 b1 で入った（migration v11 の列と BIZ-06 の完了）。

- `stocktakes` と `stocktake_items` は対象の一覧と評価の保存に残す。`stocktake_items.actual_count`・`counted_at` は、その明細の効く計数を表示のために写す（計数の値の正本は stock_counts）。`system_stock` は明細を作った時点の帳簿で、差異の計算に使わない参考値（表示する差異は 35 の新契約）。
- 完了の TX で `valuation_cost_price_centi`（その時点の原価）と `total_cost` を固定する（今のまま。評価数量は `max(その時点の在庫, 0)`）。足す列: `stocktake_items.valuation_quantity INTEGER NULL`（完了の TX で評価数量を保存。完了の前は NULL）と、stocktakes の完了の時点のカーソル `cutoff_import_id`・`cutoff_count_id`・`cutoff_movement_id`（INTEGER NULL。完了の時点の `MAX(csv_imports.id)`〈取り消したものを含む。取込みが無ければ NULL〉・`stock_counts` の最大 id・`inventory_movements` の最大 id〈無ければ 0〉。完了の前は NULL）。完了の後の比較は、完了の時点にあった記録（カーソル以下）に、完了の後に入れた完了の前の入出庫・初期在庫・精算の記録を足して求める（計数はカーソル以下だけ。精算は、完了の時点の精算の n〈`cutoff_import_id` 以下の取込みID を持つ、今 active な取込みの最大の精算回数〉以下の精算回数のものと、精算日が完了日より前のもの。ADR D6）。どれも完了の後に書き換えない。完了の後に入った記録による差は読取りで求め、保存しない（[35](../function-design/35-biz-stocktake-service.md) の新契約、owner の決定 TD-296）。
- 自動の 0 の入力（auto_filled・force_fill）を持たない。完了の時点で廃番かつ在庫 0 の商品の明細は、未計数でも完了を止めない（計数の記録を作らない）。

### 移行（SPEC-STK-CNT-D10）

(a) で入ったのは、この節の全部（v9）。

- 棚卸し（`stocktakes`）か Z004 の取込み（`csv_imports`）を 1 件でも持つ DB では、migration は何も変えずに失敗する（そうした開発・試験 DB はリリースの前まで作り直してよい。古い file は消さずに移す。D-123）。
- 持たない DB では、同じ TX で列と表を足し、既存の movement の business_date を参照先の業務記録の日付（`reference_type` が NULL の初期在庫は `created_at` の日付）で埋め、phase=0 にし、全商品を求め直して写しを書く。求め直した値が移行の前の `stock_quantity` と違う商品は、件数と商品ごとの old / new を同じ TX の `operation_logs`（`integrity_fix` と同じ detail の形に `"trigger": "migration"` を足す）に残し、起動の後に整合性の画面で知らせる。失敗は全体を戻す。手順・止まる条件・`sqlite_sequence` の扱いは [22 §19](../function-design/22-mnt-migration.md)。

> **親文書**: [DB_DESIGN.md](../DB_DESIGN.md)

---

## 14. inventory_movements（在庫変動履歴）

### 役割
全ての在庫増減を時系列で記録する。在庫変動履歴画面（REQ-303）のデータソース。「在庫がおかしいとき、なぜそうなったかを追跡する」ためのテーブル。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 変動ID |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| movement_type | TEXT | NOT NULL, CHECK(movement_type IN ('sale_auto','sale_manual','receiving','return','disposal','stocktake')) | 変動種別 |
| quantity | INTEGER | NOT NULL | 変動数量。在庫視点で常にプラス=増加、マイナス=減少 |
| stock_after | INTEGER | NOT NULL | 変動後の在庫数 |
| reference_type | TEXT | NULLABLE, CHECK(reference_type IN ('csv_import','manual_sale','receiving_record','return_record','disposal_record','stocktake') OR reference_type IS NULL) | 参照先テーブル名 |
| reference_id | INTEGER | NULLABLE | 参照先レコードID |
| note | TEXT | NULLABLE | 備考 |
| is_voided | BOOLEAN | NOT NULL, DEFAULT 0 | 論理無効化フラグ。ロールバック時に1 |
| created_at | TEXT | NOT NULL | 作成日時（YYYY-MM-DDTHH:MM:SS） |

### 符号規約（指摘#2対応、2026-03-28 確定）

sale_recordsとinventory_movementsで符号の意味が異なる。混同防止のため明文化する。

| テーブル | 視点 | プラスの意味 | マイナスの意味 |
|---------|------|-----------|------------|
| sale_records | 売上帳票視点 | 販売（売上増） | 返品（売上減） |
| inventory_movements | 在庫視点 | 在庫増加（入庫/返品戻り） | 在庫減少（販売/廃棄） |

例: Z004で返品1個（マイナス）を取り込んだ場合
- sale_records: quantity=-1, amount=-385（売上がマイナス）
- inventory_movements: quantity=+1（在庫が1個戻る）, movement_type='sale_auto'

### movement_typeの値

| 値 | 意味 | quantityの符号 | 発生元 |
|---|------|---------|--------|
| sale_auto | CSV取込みによる販売 | マイナス（返品時はプラス） | REQ-401 |
| sale_manual | 手動販売出庫 | マイナス | REQ-203 |
| receiving | 仕入入庫 | プラス | REQ-201 |
| return | 返品・交換（レジ未処理分のみ） | プラス（戻り） or マイナス（渡し） | REQ-202 |
| disposal | 廃棄・破損 | マイナス | REQ-204 |
| stocktake | 棚卸し補正 | プラス or マイナス | REQ-205 |

### 設計意図
- **全ての在庫変動を1テーブルに集約した理由**: 在庫変動履歴画面で「この商品に何が起きたか」を時系列で表示するため。入庫・販売・返品・廃棄・棚卸しの全てがここに入る
- **reference_type + reference_idの理由（ポリモーフィック関連）**: 「この変動の元の操作」を追跡する。例えばreference_type="receiving_record", reference_id=42なら、入庫記録ID:42が原因。SQLiteでは外部キー制約で強制できないため、アプリケーション側で整合性を担保
- **reference_typeの許容値をCHECK制約で固定した理由（指摘#11対応）**: ポリモーフィック関連のリスク軽減。アプリが想定外の値を書き込むのを防ぐ
- **stock_afterの理由**: 変動後の在庫数を記録しておくと、在庫推移のグラフ表示が高速になる。毎回前のレコードからの累積計算が不要。D-118 の runtime からは、その movement を保存した TX で求め直した写しの値で、後入力で前に記録が入っても書き換えない（上の「計数の置き場と在庫の求め直しの契約」）
- **is_voidedの理由（指摘#3対応）**: CSVロールバック時に物理削除ではなく論理無効化で統一。操作ログとの整合性を保つ
- **通常取消と is_voided の使い分け（2026-06-27 追加）**: 業務記録の通常取消は元 movement を隠さず、逆方向 movement を追加して追跡可能にする。`is_voided=1` は CSV取込み rollback のように取込み自体を通常履歴から外す用途に限定する。完成形では `movement_kind` / `reversal_of_movement_id` の追加を検討する（[65-inventory-record-traceability.md](../function-design/65-inventory-record-traceability.md) §65.6）。

### 困りそうなケース
- **レジ戻し処理済みの返品**: register_processed=1の返品は、CSV取込み時にZ004のマイナス値を読んで、sale_recordsにはquantity=-1（売上帳票視点）、inventory_movementsにはquantity=+1（在庫視点: 在庫が1個戻る）として記録される。return_recordsは帳面記録のみで、独自のinventory_movementsは作らない。これにより二重計上を防ぐ

---

## 15. price_history（価格変更履歴）

### 役割
商品の売価・原価の変更履歴。商品修正画面の「価格履歴」セクション（REQ-102）のデータソース。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 履歴ID |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| old_selling | INTEGER | NOT NULL | 変更前売価 |
| new_selling | INTEGER | NOT NULL | 変更後売価 |
| old_cost_centi | INTEGER | NOT NULL | 変更前原価（1/100 円） |
| new_cost_centi | INTEGER | NOT NULL | 変更後原価（1/100 円） |
| changed_at | TEXT | NOT NULL | 変更日時 |

### 設計意図
- **inventory_movementsとは別テーブルにした理由**: 価格変更は在庫の増減を伴わない。在庫変動履歴テーブルに混ぜると変動種別が増えて複雑になる
- **書込み契機（SPEC-PRV-D9）**: UI-01b の手動修正、UI-14 の一括改定、UI-02 の入庫原価差分承諾の 3 種で、売価または原価が変わったときに old/new 4 値を記録する
- **契機カラムは設けない**: 現 schema の履歴値と `changed_at` だけでは 3 契機を確実に導出できず、価格履歴閲覧にも契機表示を要求しないため schema 変更は行わない

---

## 16-17. stocktakes + stocktake_items（棚卸し）

### 役割
年末棚卸しの記録。一人で数千点の商品を回る作業（10 月からの準備と、大晦日に終える正式カウント。[73](../function-design/73-ui-stocktake.md) §73.1）に対応する中断・再開機能付き。

### stocktakes カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 棚卸しID |
| started_at | TEXT | NOT NULL | 開始日時 |
| completed_at | TEXT | NULLABLE | 完了日時。NULLなら作業中 |
| status | TEXT | NOT NULL, DEFAULT 'in_progress' | 状態。'in_progress' / 'completed' |
| cutoff_import_id | INTEGER | NULLABLE | 完了の時点の `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL。v11、D-118 の b1） |
| cutoff_count_id | INTEGER | NULLABLE | 完了の時点の `MAX(stock_counts.id)`（v11） |
| cutoff_movement_id | INTEGER | NULLABLE | 完了の時点の `MAX(inventory_movements.id)`（無ければ 0。v11） |
| total_cost | INTEGER | NULLABLE | 仕入原価総額（円、税理士報告用）。確定時に計算。商品別の金額を1/100円で求めて合計し、円未満を四捨五入する（[35 §20.5a](../function-design/35-biz-stocktake-service.md#205a-評価額の計算価格の基準数量と店の丸め) SPEC-STK-VAL-D3 / D4） |

### stocktake_items カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 明細ID |
| stocktake_id | INTEGER | FK → stocktakes.id, NOT NULL | 親ヘッダ |
| product_code | TEXT | FK → products.product_code, NOT NULL | 商品コード |
| system_stock | INTEGER | NOT NULL | 明細を作った時点の帳簿（参考値。下の設計意図の「system_stockの時点」） |
| actual_count | INTEGER | NULLABLE | 実カウント数。NULLなら未入力（D-118 の runtime からは、その明細の効く計数〈`stock_counts`〉を表示のために写した値） |
| valuation_cost_price_centi | INTEGER | NULLABLE | 確定時の評価原価（1/100 円、価格の基準数量あたり。[master-tables](master-tables.md) products の価格の基準数量） |
| counted_at | TEXT | NULLABLE | カウント日時（YYYY-MM-DDTHH:MM:SS）。NULLなら未入力 |
| valuation_quantity | INTEGER | NULLABLE | 完了で固定した評価数量 `max(写しの在庫, 0)`（v11、D-118 の b1。完了の前は NULL） |

### 設計意図
- **system_stockを明細に持つ理由**: 棚卸し中もCSV取込みで在庫が動く（SP-205-09修正）。
- **system_stockの時点**（2026-10-16、D-118 で改めた。それまでの SPEC-STK-TIME-D8-L1 は置き換えた）: 明細を作った時点（棚卸しの開始時、または棚卸し中の商品登録時）の帳簿で、差異の計算に使わない参考値。計数の値の正本は `stock_counts`、差異の表示は [35](../function-design/35-biz-stocktake-service.md) の新契約（効く計数の値と、ADR D3 の判定表でその計数に入っている記録だけから求めた帳簿の差）。計数の保存（35 §20.4）でも更新しない。
- **actual_countがNULLABLE**: 4000商品中、まだカウントしていない商品はNULL。進捗バーの「未入力」は、その明細を参照する計数（`stock_counts.stocktake_item_id`）が無い明細の数（D-118 の b1。actual_count は表示用の写し）
- **valuation_cost_price_centiの理由（指摘#4対応）**: 棚卸し確定時の原価を固定保存。商品マスタの原価が後から変わってもtotal_costがブレない。棚卸し確定時にproducts.cost_price_centiの値をコピーしてくる
- **total_costの理由**: 棚卸し確定時に、全商品の商品別の金額（valuation_cost_price_centi・数量・価格の基準数量から1/100円で求める）を合計し、円未満を四捨五入した仕入原価総額を算出（SP-205-08、税理士報告用、店の端数の規則。35 §20.5a）。1/100円の商品別の金額は保存しない（報告するのは総額だけ）

### 困りそうなケースと対応方針（2026-03-28 確定）

**ケース1: 棚卸し中に商品が新規登録された**
- 問題: stocktake_itemsに対応する行がなく、棚卸し一覧に表示されない。total_cost（仕入原価総額）が過小になり税理士報告に影響
- 対応方針: **案A（商品登録時に自動追加）を採用**。商品新規登録（REQ-101）の処理で、status='in_progress'のstocktakesがあれば、stocktake_itemsに自動INSERT（actual_count=NULL, system_stock=登録時の在庫数）。棚卸し画面に「未入力」として自動的に現れる
- 不採用案: 案B（棚卸し画面を開いたときに差分チェック）は、画面を開かないまま確定するフローがあると漏れるリスクがあるため不採用

**ケース2: 年に2回棚卸しをしたい**
- 対応方針: status='in_progress'のstocktakesが1件でもあれば、新しい棚卸しの開始をブロック。「進行中の棚卸しを完了してから新しい棚卸しを始めてください」と案内。完了後は新規作成可能
- チェック箇所: 棚卸し画面の「新規棚卸し開始」ボタン押下時にアプリ側でチェック

**ケース3: 棚卸し中に商品が廃番になった**
- 問題: 既にstocktake_itemsに行があるが、商品が廃番になった。カウント対象にすべきか
- 対応方針: 在庫が0になっている廃番商品はカウント不要。在庫が残っている廃番商品はカウント対象（実際に棚に残っている可能性がある）。D-118 の runtime からは、actual_count=0 の自動入力をせず、完了の時点で廃番かつ在庫 0 の商品の明細を未計数でも完了を止めない対象外として扱う（[35](../function-design/35-biz-stocktake-service.md) の新契約。計数の記録を作らない）

---

## 18. operation_logs（操作ログ）

### 役割
システムの主要操作を日時付きで記録。トラブル時の追跡用。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | ログID |
| operation_type | TEXT | NOT NULL | 操作種別（例: product_create, csv_import, backup等） |
| summary | TEXT | NOT NULL | 概要（1行。操作ログ画面のテーブルに表示） |
| detail_json | TEXT | NULLABLE | 詳細情報（JSON文字列。例: 変更前後の値等） |
| created_at | TEXT | NOT NULL | 操作日時 |

### 設計意図
- **detail_jsonの理由**: 操作の種類によって保持したい情報が違う。CSV取込みならファイル名・件数・金額、商品修正なら変更前後のフィールド名・値。カラムをいちいち追加するよりJSON文字列で柔軟に格納する
- **業務記録との役割分担（2026-06-27 追加）**: operation_logs は監査・保守ログであり、入出庫の明細・金額・取消/訂正の正本ではない。関連 record_type / record_id を detail_json に含めてもよいが、在庫変動の根拠表示は業務記録詳細と inventory_movements が担う。

---

## 19. app_settings（アプリ設定）

### 役割
システム全体の設定値をキー・バリューで格納。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| key | TEXT | PK | 設定キー |
| value | TEXT | NOT NULL | 設定値 |
| updated_at | TEXT | NOT NULL | 更新日時 |

### 初期データ例

| key | value | 説明 |
|-----|-------|------|
| stock_low_threshold | 3 | 在庫少の閾値（個） |
| stock_low_threshold_fabric | 500 | 在庫少の閾値（cm、生地用） |
| backup_enabled | 1 | 自動バックアップON/OFF |
| backup_time | 23:00 | バックアップ時刻 |
| backup_path | C:\在庫管理\backup\ | バックアップ保存先 |
| backup_retention_days | 3 | バックアップ保持日数 |
| tax_rate_standard | 10 | 標準税率 |
| tax_rate_reduced | 8 | 軽減税率 |
| last_plu_export_at | 2026-03-21T15:00:00 | 最後のPLU書出し日時 |
| log_retention_days | 3000 | 操作ログ保持日数（帳簿書類の 7 年に足りる長さ。[72](../function-design/72-mnt-log-manager.md) MNT-02-D1、D-136） |
| log_last_cleanup_date | 2026-03-29 | 最後にログ削除チェックした日付 |
