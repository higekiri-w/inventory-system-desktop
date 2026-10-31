## 在庫ドメイン不変条件

在庫関連処理で共通して遵守するルール。

**INV-1: quantity符号規約**
- inventory_movements.quantity は在庫視点（+増加/-減少）
- sale_records.quantity は売上帳票視点（+販売/-返品）
- SUM計算が自然になるよう、テーブルの目的に合わせた符号にする

**INV-1a: 入力値は常に正数**
- 全ての Request 構造体の quantity フィールドは正の整数のみ受け付ける（quantity <= 0 → BizError::ValidationFailed）
- 符号変換は BIZ 層の各業務関数内でのみ実施する:
  - create_receiving: apply_stock_change に +quantity
  - create_return (direction=in): +quantity
  - create_return (direction=out): -quantity
  - create_manual_sale: -quantity
  - create_disposal: -quantity
- IO層は符号変換を行わない。BIZ層から渡された値をそのまま記録する

**INV-2: stock_after算出責任**
- BIZ-02の共通在庫変動関数 (apply_stock_change) が、movement を仮の stock_after（今の写しの値）で INSERT した後、`recompute_stock` で記録から求め直した写しの値をその movement の stock_after に書く（D-118 の (a)。[31](31-biz-inventory-service.md) §12.2）
- IO層（inventory_repo の insert_movement・update_movement_stock_after）は渡された stock_after をそのまま記録する

**INV-3: 負在庫ポリシー**
- stock_after < 0 の場合は警告フラグをセットするが、処理は続行する
- 理由: 小規模店舗では入庫記録の遅延で一時的にマイナスになることがある。エラーにすると業務が止まる

**INV-4: is_voided の使用範囲**
- is_voided フラグは CSV 取込みロールバック時（BIZ-03）のみ使用する
- BIZ-02 では is_voided を操作しない。insert_movement は常に is_voided=0 で挿入する

**INV-5: 冪等性**
- 各ヘッダテーブルに idempotency_key (TEXT NOT NULL UNIQUE) と request_fingerprint (TEXT NOT NULL) を持つ
- CMD層がUUID v4でidempotency_keyを生成しBIZ層に渡す
- BIZ層がリクエスト内容からrequest_fingerprintを計算する
- idempotency_key重複時:
  - request_fingerprint一致 → 冪等な再送成功（既存レコードのIDを返す）
  - request_fingerprint不一致 → BizError::IdempotencyConflict
- 予約プレフィックス `__legacy__:` はマイグレーション時のバックフィル用。UUIDと衝突しない

**INV-6: CSV取込みの自然冪等性（file_hash方式）**
- csv_importsテーブルはINV-5のidempotency_key/request_fingerprintパターンを使わない
- file_hash（SHA-256、生バイト列＝デコード前のrawバイトから算出、hex小文字64文字）が自然な重複検知キー
- 重複判定: `file_hash一致 + status IN ('completed','completed_partial')` → ブロック
- ロールバック後（status='rolled_back'）の再取込みは許可
- settlement_date一致 + 別file_hash → 既存active import全件を示す追加確認後、新規分をinsert-onlyで取込む
- settlement_dateはgroup keyでありuniqueness keyではない。訂正は対象import IDのrollbackと再取込みを明示した2操作で行う
- スコープ: 単一店舗（1DB）
- 改行コード差異（CRLF/LF）は別hashとなるが、これは仕様として許容。同一内容でも改行コードが違えば別ファイル扱い
- file_hashにUNIQUE制約なし（DB_DESIGN.md確定済み: ロールバック後に同一hashが2行できるため）。競合防止はcommit TX内でのcheck-then-insertで対応
- **前提: SQLite単一接続（1人運用デスクトップ）**。マルチ接続化する場合は UNIQUE(file_hash) の条件付き再導入またはアプリ層排他ロックが必須

**INV-7: csv_import参照のinventory_movements制約**
- reference_type='csv_import' のinventory_movementsは、movement_type='sale_auto' のみ
- BIZ-03のcommit_csv_importが生成し、rollback_csv_importがvoidする
- void_movements_by_reference は reference_type + reference_id のみで絞り込む（movement_type条件は冗長のため付与しない。この不変条件が安全性の根拠）

**INV-8: products物理DELETE禁止**
- productsテーブルの行を物理DELETEしない。廃番管理は is_discontinued フラグ（論理削除）のみ
- 理由: inventory_movements, sale_records, receiving_items, return_items, disposal_items, manual_sale_items, stocktake_items, price_history の8テーブルがFK参照しており、物理削除は整合性を破綻させる

---

## 単位・数量・金額の共通規則

契約 ID: SPEC-UNIT-D1〜D11（2026-10-07、[decision-log](../decision-log.md) D-113）。単位と原価の分（D1〜D7・D9〜D11 と「入力の上限と安全な整数の範囲」）は現行の契約である（migration は [22](22-mnt-migration.md) §16 の v8 と vB〈v10〉）。次の 2 つは未実装で、項目ごとに lane の名前を書く（D-113）: D8 と `stock_quantity_from_pos`（Z004・EJ の lane）、（棚卸しの分〈D3 の表の 7・D11 の `StocktakeItemDetail`・棚卸しの実数の上限と確定の検査〉は D-118 の b1 で入った）。owner の判断 J1〜J4 は 2026-10-08 にすべて推奨の案に決まり（repo 外の回答台帳 TD-195）、該当の項目に `決定（owner、D-113 Jn）` と書いた。

2026-10-17 の見直し（D-119）で、原価の 1/100 円は既存の DB を変換せず開発・試験 DB の作り直しの起点で入れ（D5）、巻・cm はその起点で一覧から外し（D1）、単位の語は登録の後に個数の単位どうしで直せるようにした（D2）。D5・D1 の分は原価の lane（D-128）が vB で実装した。

**SPEC-UNIT-D1 単位の一覧と数量の種類**: 商品の単位（`products.stock_unit`、wire は generated enum `ProductStockUnit`）は次の 10 個の code に限る。code は ASCII で、一度決めた code の意味を変えない。表示の語は code から UI が引き、DB と wire には日本語を入れない。

| code | 表示の語 | 数量の種類 | 在庫の数量の 1 | 価格の基準数量 | 数量の入力 |
|---|---|---|---|---|---|
| `pcs` | 個 | 個数 | 1 個 | 1 | 整数 |
| `sheet` | 枚 | 個数 | 1 枚 | 1 | 整数 |
| `hon` | 本 | 個数 | 1 本 | 1 | 整数 |
| `bag` | 袋 | 個数 | 1 袋 | 1 | 整数 |
| `box` | 箱 | 個数 | 1 箱 | 1 | 整数 |
| `kumi` | 組 | 個数 | 1 組 | 1 | 整数 |
| `set` | セット | 個数 | 1 セット | 1 | 整数 |
| `ball` | 玉 | 個数 | 1 玉 | 1 | 整数 |
| `cho` | 丁 | 個数 | 1 丁 | 1 | 整数 |
| `m` | m | 長さ | 1 cm | 100（1 m あたり） | m で小数 2 桁まで（D3） |

- `roll`（巻）と `cm` は、開発・試験 DB の作り直しの起点（[22](22-mnt-migration.md) §16 の vB）で一覧から外した（D-119。owner の決定 U1 = d、repo 外の回答台帳 TD-297）。前提（最新の在庫 Excel で巻・cm の行が 0）は 2026-10-18 版の在庫 Excel で成り立った（[decision-log](../decision-log.md) D-128 (2)）。vB の前の版の DB（`roll`・`cm` を持つ 12 個の CHECK）は vB で止まり、作り直す。D10 の `在庫単位` の語も同じ一覧。
- 出典: 12 の語は問う側（owner）が示した一覧で、店主は「店で使わない数え方は無い」と答えた（2026-09-14。`project-memory.md` の Store Premises Facts）。店主が自分で挙げた語は個・玉・枚・本・丁で、巻・cm で数える商品の記録は無い（2026-09-12 版の在庫 Excel で巻・cm は 0 行。D-119 の前提の訂正）。重さ（g・kg）は使わない（同、owner 2026-09-13）。毛糸の数え方は「玉」で、在庫計数 Excel の毛糸の「個」は owner の入力の誤り（repo 外の回答台帳 TD-192。意味は通るので Excel はそのままでよい。Excel から「個」で入った毛糸は、登録の後に D2 で「玉」へ直せる）。`ball` は一覧に入れる（`決定（owner、D-113 J1）`、2026-10-08、TD-195。不採用: 入れずに毛糸を `個` で数える）。
- 数量の種類と価格の基準数量は code だけで決まる。商品ごとの列は持たない（[35 §20.5a](35-biz-stocktake-service.md#205a-評価額の計算価格の基準数量と店の丸め) SPEC-STK-VAL-D1 の理由のまま）。箱・袋で仕入れてばらす商品は「ばらした 1 個」を単位にし、1 個の原価を小数第 2 位まで持つ（D5）。店の答え（TD-058）は箱あたりの原価を記録するものではないので、SPEC-STK-VAL-D1 の「見直す条件」には当たらない。
- 巻で数えていた商品は `pcs`（個）で入れる（巻は個数で、m と換算しない。1 巻の長さは商品で違う）。`kumi`（組）と `set`（セット）は別の code（店は両方使う）。
- 長さの単位は `m` だけ。店は長さの商品の残り・仕入れ・値札を m で扱う。在庫は cm の整数で持つ（D2）。
- 不採用: (a) 日本語の語を code にする（表示の語を直すと DB・CSV・ログの値が変わる）。(b) 保存の基準（`pcs` / `cm`）と表示の単位を別の列にする（種類と基準数量が code から一意に決まり、2 列は同じことを 2 度持ち、組合せの CHECK が要る）。(c) `group`・`pair` 等の英語の code で 組・本・丁 を表す（意味の違う英語を当てると、後で読む人が数を誤る）。

**SPEC-UNIT-D2 在庫の数量の表現**: 在庫・明細・変動の数量は今までどおり整数（INV-1a・INV-3）。個数の単位は 1 単位、長さの単位は 1 cm を 1 とする。m は入力と表示の換算だけ（1 m = 100 cm）。10 cm 刻みを DB・BIZ で強制せず、1 cm まで受ける（owner の決定 U3 = a、D-119）。理由: 長さの 10 cm は刻みである（repo 外の回答台帳 TD-332）が、刻みを DB・BIZ で検査するかは店主の答え（repo 外の回答台帳の問い Q-098、TD-351。2026-10-21 に答えがあり、判断待ち）で決める（[decision-log](../decision-log.md) D-128 (3)）。決めるまでは 1 cm まで受ける。1 cm まで受ける方は受ける範囲が広いだけで、受けた値を黙って変えない。10 cm 刻みに限ると、店の実際の長さを拒みうる。店は棚卸しで cm までは数えない（メーカー記載の長さで数える）ので、1 cm より細かい在庫は要らない。
- 単位の変更: 数量の整数の意味は単位で決まるので、数量の種類か価格の基準数量の違う単位へ変えると、在庫と過去の明細（入庫の明細は今の単位を JOIN して金額を求める）の意味が黙って変わる。よって、種類をまたぐ変更（個数 ↔ 長さ）は登録の後に許さない（長さの単位は `m` だけなので、長さどうしの変更は無い）。商品修正と商品 CSV の上書き（[30](30-biz-product-service.md) §4.4・§4.9）は次の項目の変更だけを許し、外れた変更は BIZ が拒む（商品修正は `ValidationFailed`、商品 CSV は preview の行の error、commit は TX の中で再検証して TX 全体を戻す。BIZ-01-D8）。
- （D-119。owner の決定 U2 = b、repo 外の回答台帳 TD-297）個数の種類の単位どうし（D1 の表の 9 個）の変更だけを、商品修正と商品 CSV の上書きで許す（例: 毛糸を「個」から「玉」へ）。個数の単位は価格の基準数量がどれも 1 で、在庫・明細・売価・原価の数の意味が変わらない。長さの単位（`m`）の商品の単位は変えない。過去の記録の単位の語は今の単位を引いて出すので、変えた後は過去の記録も新しい語で出る（数は変わらない）。BIZ の検査・log・画面は [30](30-biz-product-service.md) BIZ-01-D8・§4.4 と [51](51-ui-product-form.md) UI-01b-D23。

**SPEC-UNIT-D3 長さの入力と表示**:
- 入力: `m` の商品の数量欄は、前後の空白を除いた文字列（今の整数の欄の `parseRequiredSafeInteger` と同じ `trim`）が `^[0-9]+(\.[0-9]{1,2})?$` に全体で一致するときだけ受け、浮動小数を通さず cm の整数にする（整数部 × 100 + 小数部を右に 0 を埋めた 2 桁）。例: `1.3` → 130、`1.27` → 127、`2` → 200。小数 3 桁以上・指数表記・符号・空は入力の error で、丸めない。cm にした値が 0 のとき（`0`・`0.00`）は、0 を受ける欄（商品登録の初期在庫、棚卸しの実数）以外では入力の error（文は下の「入力の error の文」）。cm にした値がその欄の上限（下の「入力の上限と安全な整数の範囲」。数量欄は `9999999` で `99999.99` を受け `100000` は error、初期在庫と棚卸しの実数の欄は `999999999` で `9999999.99` を受け `10000000` は error）を超えれば入力の error。TS は文字列の整数部と小数部を別々に整数にして組み、小数の number の掛け算を通さない（`0.29` → 29・`1.15` → 115・`4.35` → 435 は `Math.trunc(Number(s) * 100)` では 28・114・434 になる。上限が小さくなってもこの誤りは残る）。個数の単位の数量欄は整数の規則で、同じ上限を掛ける。wire には cm の整数を送る（DTO の数量の型は変えない）。適用する数量欄は、商品登録の初期在庫（[51](51-ui-product-form.md) UI-01b-D22）、入庫（[61](61-ui-receiving.md)）、返品・交換（[63](63-ui-return-exchange.md)）、手動販売（[62](62-ui-manual-sale.md) UI-04-D18）、廃棄（[64](64-ui-disposal.md)）、棚卸しの実数（[73](73-ui-stocktake.md)）と、商品 CSV の `初期在庫`（下の項）で全部。在庫少の基準の欄（[69](69-ui-threshold-settings.md)）は cm のまま（D10）。
- 商品 CSV の `初期在庫`（[30](30-biz-product-service.md) BIZ-01-D8）: 同じ行の `在庫単位` が `m` の行は、値を m の数として上の規則で cm にする（`25` → 2500、`26.15` → 2615。0 は受ける〈今の初期在庫と同じ〉。小数 3 桁以上・指数・符号は行の error `初期在庫の値が不正です: '{値}'`〈今の文〉で、丸めない。Rust は文字列の整数部と小数部を `i64` の checked の演算で組み、`f64` を通さない。cm にした値が在庫の上限 `999999999` を超えれば（上の UI の初期在庫の欄と同じ上限。`9999999.99` は受け、`10000000` は error）同じ行の error。10 cm 未満も上の規則どおり受ける）。個数の単位の行は整数の規則。Coordinator が店の事実（repo 外の回答台帳 L-214: 残り・仕入れ・値札は m）から決めた（2026-10-08）: 在庫計数 Excel の長さの商品は m で数えてあり、cm の整数で読むと `25` が 0.25 m で黙って入る。
- 入力の行の数量の加算: 同じ商品を入力の行へ再追加したときの加算（入庫・返品・手動販売・廃棄）と、行の統合（返品の方向の切替え・廃棄の種別と理由が同じ行）は、入力の文字列をこの規則で cm の整数（個数の単位は整数）にしてから整数で足し、表示の規則と同じ形（3 桁区切りなし）の入力の文字列に戻す。再追加で足す量は、その画面で商品を追加したときの数量の初期値（`m` の商品は `1` = 100 cm。手動販売は UI-04-D18）。例: `m` の `1.3` に再追加 → `2.3`、廃棄の `0.5` と `0.3` の統合 → `0.8`。文字列が規則に合わないときは今の fallback のまま（再追加はどの画面も初期値。統合は、廃棄は元の値〈統合先の文字列〉、返品の方向の切替えの合算は `"1"`）。
- 表示: 長さの数量を m で表すとき、符号・整数部（3 桁区切り）・小数部 2 桁を整数の演算で作り、小数部の末尾の 0 を落とす（130 → `1.3 m`、127 → `1.27 m`、200 → `2 m`、-30 → `-0.3 m`、123456 → `1,234.56 m`）。長さの単位（`m`）はこの m の形で出す（Coordinator が owner の決定 TD-203〈長さの商品の数量は m〉と TD-207 から決めた、2026-10-08。保存値は cm の整数のまま）。個数の単位は `10 個` `3 玉` の形（数と語の間に半角空白。今の `formatStockDisplay` と同じ）。数量を伴わない単位の表示（入力の行の単位欄）は D1 の表示の語。
- 表示する所（owner の決定、2026-10-08、repo 外の回答台帳 TD-203）: 長さの商品の数量を出す画面はすべて m で出し、単価は 1 m あたりで出す。在庫を cm の整数で持つ契約（D2）は変えない。次の表で全部（2026-10-08 に main `95c0aeb0` の現物で `rg -n '\b(quantity|stock_quantity|actual_count|system_stock|difference|stock_after|adjustment_quantity|current_stock)\b' src --glob '*.tsx' --glob '!*.test.*'` と bindings の数量の field を持つ型を当てて作った。site の file:line は Plan Packet `docs/archive/plans/2026-10-07-unit-extension.md` の申し送りの表）。

| # | 画面 | 出す値 | 長さの商品の表示（例） | 単位の渡し方 |
|---|---|---|---|---|
| 1 | 在庫照会・商品一覧・入出庫の候補・業務記録詳細（今の formatter の呼出し） | 在庫・明細の数量 | `27.5 m` | 各行の `stock_unit`（D11 の enum） |
| 2 | 日次売上の商品の明細 | 数量・単価 | `1.3 m`・`¥700/m` | `DailySaleItem` の `stock_unit` |
| 3 | 日次売上の部門小計・合計、販売点数のカード（日次売上・ホーム） | 数量の種類ごとの和（下の集計の規則） | `5 点・1.8 m` | `DeptSubtotal`・`GrandTotal` の `count_points`・`length_cm` |
| 4 | 月次売上の商品別・部門別、月間販売点数 | 数量の種類ごとの和（商品別の行はどちらか一方だけ） | 商品別 `1.3 m`・`3 点`、部門別 `5 点・1.8 m` | `MonthlySaleItem` の `count_points`・`length_cm` |
| 5 | 売上の CSV 出力（日次・月次の商品別・部門別） | `数量` の列 | 商品の行は長さの商品（`m`）が m の数 `1.3`（個数の商品の行は今と同じ整数）。集計の行（月次の部門別）は画面と同じ文字列 `5 点・1.8 m`（個数の商品だけの部門は今と同じ整数） | 上の 2〜4 の wire |
| 6 | 在庫変動の表（商品別の在庫変動の画面の `StockHistoryTable` と、入庫・返品・手動販売・廃棄・振替〈D-143〉・商品別売上 CSV の取込みの記録詳細の「関連する在庫変動」の `MovementTable` の 7 経路。棚卸しの記録詳細の表は D-118 の b1 で外れた） | 変動の数量・変動後の在庫、計数の行の数（D-118 の b2） | `+2.5 m`・`27.5 m`、計数の行は `数を確認（4.5 m、…）`（個数は `数を確認（8 個、…）`） | movement の行は `product_code → stock_unit` の対応（`ReadonlyMap<string, ProductStockUnit>`、必須の prop）を受け、行ごとに引く。商品別の画面は読んだ商品 1 つから、記録詳細は同じ記録の `detail.items`（各明細が `product_code` と `stock_unit` を持つ）から作る（`MovementRecord` は変えない）。対応に無い商品コードの行は今どおり整数で出す（明細と変動は同じ記録から作るので、ふつうは起きない）。計数の行の数は行の `count.stock_unit` で、在庫の欄と同じ `formatStockDisplay` の形（個数にも単位の語を付ける。[66](66-ui-stock-movements.md)） |
| 7 | 棚卸しの計数（一覧と入力欄の現在在庫・実数・差異）と確定の結果の一覧 | 現在在庫・実数・差異 | `27.5 m`・`-1.35 m` | `StocktakeItemDetail` と記録の詳細の明細に `stock_unit` を持つ（D-118 の b1 で実装。棚卸しの確定は在庫を動かさないので結果の一覧は持たない） |
| 8 | 整合性チェック | 保存している値・記録から求め直した値（要再確認なら範囲の端も） | `27.5 m`・`12 m（要再確認 11.5 m〜13 m）` | `IntegrityMismatch` の `stock_unit` |
| 9 | PLU 書出しの未書出しの一覧 | 在庫 | `27.5 m` | `ProductResponse` の `stock_unit` |
| 10 | 入庫・返品・手動販売・廃棄・振替〈D-143〉の保存の結果の在庫警告（BIZ の `stock_warnings` の文） | 保存の後の在庫 | `{商品コード}: 在庫がマイナスになりました（-0.3 m）` | BIZ が長さの商品の数量を D3 の m の形に整形して文に入れる（個数の商品の文は `（-2）`。TS の formatter と同じ規則の Rust の関数、golden は D9） |
| 11 | 整合性チェックの補正の結果（`StockAdjustment`） | 補正の前と後の在庫 | `27.5 m → 27 m` | `StockAdjustment` の `stock_unit` |
| 12 | 操作ログの整合性チェックの補正の要約（`integrity_fix` の detail_json） | 旧在庫・新在庫・補正の量 | `旧在庫 27.5 m → 新在庫 27 m`・`-0.5 m` | detail_json の各 adjustment の `stock_unit`。画面は key があれば m で出す |
| 13 | 商品修正の画面の現在庫（`ProductForm` の read-only の欄） | 在庫 | `27.5 m` | 読んだ商品の `stock_unit`（1 と同じ `formatStockDisplay` の形。要再確認なら 14 の範囲） |
| 14 | 商品の画面の在庫の欄で写しが要再確認のとき（商品一覧・在庫照会の一覧と詳細・商品修正の現在庫・在庫変動の見出し。D-118 の b2） | 写しの範囲の両端 | `4〜4.5 m`（個数は `8〜9 個`） | 商品の結果の `stock_unit`・`stock_range_low`・`stock_range_high`。棚卸しの画面の要再確認の一覧と同じ `formatStockOrRange`（単位の語は後ろに 1 回。8 の整合性チェックの書き方とは別） |

- 集計の規則（owner の決定、2026-10-08、repo 外の回答台帳 TD-206・TD-207）: 単位の違う商品をまたいだ点数・数量の集計（表の 3・4・5 の部門小計・合計・販売点数・月次の部門別、CSV の集計の行）は、数量の種類（D1）ごとの 2 本に分ける。個数の種類の 9 単位（`pcs` `sheet` `hon` `bag` `box` `kumi` `set` `ball` `cho`）は単位をまたいでそのまま足して `点` で出す（3 個 + 2 枚 = `5 点`。個数の単位は広い意味で点で括れる）。長さの種類（`m`）は点に入れず、cm の整数で足して m で出す（1.3 m + 0.5 m = 180 cm = `1.8 m`。長さが点に混ざると情報がごちゃまぜになる）。wire は 2 つの整数 `count_points: i64`（個数の種類の数量の和）と `length_cm: i64`（長さの種類の数量の和、cm）。和は checked で足し、i64 の溢れは `ValidationFailed`（`apply_stock_change` の今の `checked_add` と同じ扱い）。集計の読取りを安全な整数の範囲で検査して拒むことはしない（1 明細の数量の上限 `9999999` の下では、1 つの集計が `9007199254740991` を越えるのに 900,720,016 件の明細が要る。下の「入力の上限と安全な整数の範囲」の (d)）。TS で再集計する所（日次の部門小計、月次の合計）は各加算の後に `Number.isSafeInteger` を確かめ、外れればその欄に `—` を出す（壊れた DB でも画面が誤った数を出さないための分岐）。月次の商品別の行は、数量の種類を決めるために `stock_unit` を持ち（部門別の行は null）、表示はその単位で決める（長さの商品は `length_cm` を m で、0 でも `0 m`。個数の商品は `count_points` を `点` で、0 なら `0 点`）。販売と返品が相殺して 0 になった行も種類を失わない。部門別・合計の行は種類を持たず、両方 0 なら `0 点`（長さの商品だけが相殺した部門も `0 点`）。表示は、`count_points ≠ 0` か両方 0 なら `N 点`、`length_cm ≠ 0` なら D3 の形の m を出し、両方あれば `・` でつなぐ（3 個 + 2 枚 + 1.3 m + 0.5 m の日は `5 点・1.8 m`、長さだけの日は `1.3 m`、明細 0 行は `0 点`）。`N 点` の `N` は 3 桁区切りを付ける（符号は先頭で、区切りは絶対値の桁に付ける: `1,234 点`・`1,234 点・1.8 m`・`-1,234 点`。CSV の集計の行の文字列も同じで、`length_cm` が 0 の部門の整数には付けない）。個数の商品だけの日は、値は単位が 2 種だった頃の `quantity` と同じで、表示は 1,000 点未満なら同じ、販売点数のカードは 1,000 点以上で区切りが付く。レジの日報の点数との照合は日次売上の「レジ日報（公式）」の節（Z001 の表）が受け持ち、アプリの集計はレジに合わせない。数量の列の並べ替えは `(count_points, length_cm)` の組を辞書順で比べる（商品別の行では長さの商品〈count_points 0〉が個数の商品の前に、それぞれの中で数量の順）。`quantity` の名前を残さず 2 つの field に分けるのは、意味が変わる古い読み手を TS の型検査で止めるため。不採用: 単位ごとに 12 本に分ける（TD-206 の最初の案。TD-207 で 2 本に精密化）、レジの数量で足す（3 個 + 1.3 m = 4.3 点、TD-206 で不採用）、cm の値をそのまま点に足す（今の形。1.3 m が 130 点に数えられる）。
- 単価（表の 2）: `|金額| × 基準数量 ÷ |数量|` を整数の演算で円未満で四捨五入する（D7 と同じ規則。`910 × 100 ÷ 130 = 700`）。長さの商品は `¥700/m`、個数の商品は今どおり `¥700`。数量 0 は今どおり `—`。
- 表の外: レジの日報（Z001・Z005）の部門の数量（D-104、今のまま）。記録済みの文字列と値（在庫変動の `note`、操作ログの detail_json・summary。書き換えず、`stock_unit` の key が無い記録は今どおり整数で出す）。棚卸しの補正の note は D-118 の b1 で無くなった（完了は movement を作らない）。商品別売上 CSV の取込みの error 行の `raw_quantity`（file の生の文字列）。在庫少の基準の画面（D10）。数量の生の値で比べるだけの所（在庫少の一覧の並べ替え `src/features/stock-inquiry/lib/filter-low-stock-list.ts` の `filterAndSortLowStockList` の `a.stock_quantity - b.stock_quantity`、商品検索の「在庫数」の並べ替え `src/features/products/search.ts` の `PRODUCT_SORT_OPTIONS` の `stock_quantity`。単位をまたいで数の大小で並ぶ）。表を作るときは tsx の field に加え、Rust の文に数量を入れる所（`rg -n '在庫|数量|→' src-tauri/src/biz --glob '!*tests*'` の `format!`・`warning`・`note`・`summary`）と、`old_`/`new_` の数量の key（`rg -n 'old_stock|new_stock' src src-tauri/src`）を当てた（2026-10-08）。
- 入力の error の文（`m` の商品）: `数量は0.01以上で、小数は2桁までの m で入力してください`。0 を受ける欄（商品登録の初期在庫〈今の UI-01b の初期在庫は 0 以上、既定 0〉、棚卸しの実数）は `数量は0以上で、小数は2桁までの m で入力してください`。
- 不採用: 表示を小数 1 桁に丸める（127 cm が 1.3 m と見え、見た値を入れ直すと 130 cm になる）。`1 m 30 cm` の形を主の表示にする（表の桁がそろわず、入力の値と対応しない）。

**SPEC-UNIT-D4 価格の基準数量と POS の数量 1**: 売価・原価は「価格の基準数量」ぶんの値である（SPEC-STK-VAL-D1 を 10 単位へ広げる）。個数の単位は 1、長さの単位は 100（1 m あたり。値札・伝票と同じ）。不変条件: レジの数量 1 = 価格の基準数量（個数の商品は 1 単位、長さの商品は 1 m。店のレジは数量 1.3 × 1 m あたりの単価で打てる）。よって PLU の単価は売価のまま（[25](25-io-plu-formatter.md) IO-04 は変えない）。この不変条件が成り立たない単位を足すときは本節を見直す。
- 未決（店に聞いてから決める。owner の決定 U5 = c、D-119）: 長さの商品の在庫をどの道で減らすか。(a) 今のまま: 商品登録の在庫連動の自動提案は長さの単位で `false`（[51](51-ui-product-form.md) UI-01b-D6・D22）で、売った分は手動販売（[62](62-ui-manual-sale.md)）で記録する。(b) レジに数量を打って売り（PLU）、Z004 から引く（店の売り方の変更）。答えが無い間は (a) のまま。どちらでも本節の不変条件（レジの数量 1 = 1 m）と D8 の換算は変えない。答えが (b) のとき変わる所: 長さの商品を `plu_target`・`pos_stock_sync` の対象にする運用と、その前提になる JAN の無い商品の店内コード（[backlog](../backlog.md) の「JAN の無い商品の店内コードと色ごとのバーコードの表」）、在庫連動の自動提案の既定（51 UI-01b-D6・D22）、Z004 の取込みで長さの換算の経路が実際に通るか（D8。(a) の間は test でしか通らないが、作るかどうかは変わらない）。問いは repo 外の回答台帳の未回答の問いに置く（backlog の「店に聞く: 長さの商品の在庫の減らし方と原価の税」）。

**SPEC-UNIT-D5 原価の精度（1/100 円）**（D-113・D-128。`決定（owner、D-113 J2）`、2026-10-08、TD-195。原価の lane が vB で実装）: 原価は 1/100 円の整数で持つ。DB の列と wire の field は名前に `_centi` を付けて改名する（`products.cost_price` → `cost_price_centi` ほか、[master-tables](../db-design/master-tables.md) の「単位と原価の精度の契約」の表）。売価は円の整数のまま（レジの PLU の単価と値札が円の整数）。
- 入れ方（owner の決定 U4 = b、D-119。repo 外の回答台帳 TD-297）: 既存の DB の円の値を 1/100 円へ変換しない。開発・試験 DB を作り直した新しい起点で入れる（[22](22-mnt-migration.md) §16 の vB は、原価の列を持つ表に行が 1 つも無い DB にだけ当たり、列の名前を変えるだけで値を変えない）。owner は開発・試験 DB の作り直しを承認した（作り直す DB と保全の範囲は原価の lane の packet が示す）。本番の DB は無い。名前に `_centi` を付けるのは変換の有無に依らない守りで、残す（古い SQL・画面を名前の違いで止める）。
- 税の区分（[decision-log](../decision-log.md) D-128 (1)。店の答え、repo 外の回答台帳 TD-331）: 原価は税抜の値で持つ（税理士に出す仕入原価の総額の区分。[master-tables](../db-design/master-tables.md) の `cost_price_centi`）。在庫 Excel の原価も税抜で（台帳 TD-166）、初期投入は Excel の値をそのまま入れ、換算しない（[30](30-biz-product-service.md) BIZ-01-D8 の末尾）。画面は原価の欄の label と価格の節の説明に税抜と書く（[51](51-ui-product-form.md) UI-01b-D22）。棚卸しの評価額は税抜の仕入原価総額（[35 §20.5a](35-biz-stocktake-service.md#205a-評価額の計算価格の基準数量と店の丸め)。計算は変わらない。出力の文に書くのは評価額の出力を作る lane）。
- 入力の補助（D-119。owner の決定 U7 = b。TS の `unitCostFromPack`）: 箱・袋で仕入れてばらす商品のために、商品フォームの原価の欄に「袋の値段 ÷ 入り数」の補助を置く（[51](51-ui-product-form.md) UI-01b-D24）。補助は数量の種類が個数の単位の商品だけに出す（長さの単位〈`m`〉の商品では入り数の意味が決まらず、誤った原価が黙って入るので出さない）。袋の値段（下の入力の規則で 1/100 円の整数 P）と入り数（1 以上 `9999999` 以下の整数 n）から、`P ÷ n` を 1/100 円未満で四捨五入した値（商 q・余り r で `2r >= n` なら q + 1。店の「小数第 3 位で四捨五入して小数第 2 位まで」）を原価の欄に入れる（`1000` 円 ÷ 12 → 8333 = `83.33` 円）。結果だけを残し、袋の値段と入り数は保存しない（DB・wire に出さない）。入れた後の原価は利用者が直せ、保存の検証は下の入力の規則のまま。補助は TS の純関数で、結果は `P` 以下なので上限を越えない。
- 入力: 原価の欄は `^[0-9]+(\.[0-9]{1,2})?$` を浮動小数を通さず 1/100 円の整数にする（`83.33` → 8333、`1000` → 100000）。上限は `99999999`（`999999.99` 円。下の「入力の上限と安全な整数の範囲」。BIZ が検査し、超えれば `原価が大きすぎます`。`999999.99` は受け、`1000000` は error）。Rust は `product_service.rs` の `parse_hundredths`、TS は `src/lib/unit-quantity.ts` の `parseCostInput`（数量の m の入力と同じ関数を共有する）。error の文は `原価は0以上で、小数は2桁までで入力してください`。
- 表示: 100 で割り切れれば円の整数、そうでなければ小数 2 桁。前に半角の `¥`（U+00A5）を付け、整数部に 3 桁区切りの `,` を入れる（`¥1,000`・`¥83.33`・`¥0.05`）。原価を出す全ての画面は 1 つの TS の formatter（`src/features/stock-inquiry/lib/format-stock-display.ts` の `formatCostYen`）を使う。売価・合計など原価でない金額の記号は画面ごとの今の形のまま（全角の `￥` の画面もある）。入力の欄の初期値（入庫・廃棄の行に商品の原価を入れるとき、商品修正の画面を開いたとき、価格改定で新原価（案）を新原価の欄に入れるときと、現売価 0 の行で現原価を新原価の欄に入れるとき）は同じ規則で記号と桁区切りを付けない文字列（`1000`・`83.33`・`0.05`。`src/lib/unit-quantity.ts` の `formatCostInput`）にする。
- 理由: 店は箱・袋で仕入れてばらす商品の 1 個の原価を「1 袋の値段 ÷ 入り数を小数第 3 位で四捨五入して小数第 2 位まで」持つ（`project-memory.md` の Store Premises Facts、2026-09-29。台帳 TD-058）。円の整数では 83.33 が 83 になり、1,000 個で 330 円の評価額の差になる。名前を変えずに意味だけ 100 倍にすると、古い SQL・画面が黙って 100 倍の値を出す（D-104 と同じ理由）。
- 不採用: (a) 商品ごとに原価の基準数量（入り数）を持ち `1,000 円 / 12 個` の比で表す（店の計算〈83.33 円〉と違う値になり、TD-023 の商品別の金額と合わない。列と入力欄が増える）。(b) REAL（2 進の浮動小数で四捨五入を誤る）。(c) 10 進の文字列（`SUM` できない）。(d) 売価も 1/100 円にする（レジと値札が円の整数で、要る場面が無い）。

**SPEC-UNIT-D6 原価 × 数量の金額と丸め**: 原価 × 数量の金額はすべて BIZ の共通関数で求め、IO と UI の表示の値は BIZ が返した値を使う（UI の入力中の見込みの合計は D9 の twin）。
- 行（商品別）の金額 = `原価(1/100 円) × 数量 ÷ 基準数量` の正確な値を 1/100 円未満で四捨五入する（商 q・余り r で `2r >= 基準数量` なら q + 1。SPEC-STK-VAL-D3 と同じ）。
- 合計 = 行の金額（1/100 円）の和を円未満で四捨五入した円の整数（和 s の商 q・余り r で `r >= 50` なら q + 1。SPEC-STK-VAL-D4 と同じ）。
- 適用: 棚卸しの評価額（店の答え TD-023 のとおり。35 §20.5a）。廃棄のロス原価と入力画面の合計（店の直接の答えが無いので TD-023 に合わせる）。入庫の原価小計と原価合計（TD-023 と同じ。`決定（owner、D-113 J3）`、2026-10-08、TD-195。不採用: 行ごとに円未満を切り捨てる〈メーカーの伝票は切り捨てが多い、台帳 L-135。ばらした商品で伝票の合計と 1 円ずれる〉）。棚卸し記録詳細のロス原価（同じ関数。D-118 の b1 で BIZ が求める）。
- 原価・数量が負なら金額を求めず `ValidationFailed`（SPEC-STK-VAL-D5 と同じ。棚卸し記録詳細のロス原価は差異の絶対値を渡す）。中間は i128、合計の円は i64 へ検査付きで変換し、溢れは `ValidationFailed`。浮動小数を使わない。
- 原価の引数は i128 の 1/100 円（`cost_price_centi: i128`）。呼ぶ側は DB の `cost_price_centi`（i64）を `i128::from` でそのまま渡す。関数は `原価 × 数量` を `checked_mul` で求め、溢れは `ValidationFailed`。35 §20.5a の SPEC-STK-VAL-D3 の円の golden（境界と overflow〈`i64::MAX` と `92_233_720_368_547_759` の円の原価を含む〉）は、`unit_amount.rs` の test の helper が円の原価 a を `i128::from(a) * 100` にして確かめる（test だけ）。
- 行の金額の wire（入庫の `line_cost_centi`・廃棄の `line_loss_cost_centi`。棚卸し記録詳細のロス原価を BIZ へ移すときも同じ）は、1/100 円の整数の十進の文字列（`^[0-9]+$`。i128 の値を `to_string` で書く）。number にしない: 入庫・廃棄の 1 行は上限の中で `99999999 × 9999999 = 999999890000001` 以下だが、同じ形で運ぶ棚卸し記録詳細のロス原価の行は `99999999 × 999999999 = 99999998900000001` まであり、JS の number では `99999998900000000` になる（node の `Number("99999998900000001")` で確かめた、2026-10-11）。TS は文字列のまま整数部に 3 桁区切りを入れ、下 2 桁を小数部にして `円` を付ける（`"83250"` → `832.50 円`、`"99999998900000001"` → `999,999,989,000,000.01 円`、`"5"` → `0.05 円`）。number に直さない。合計（円）の wire は今の円の wire（number）のままで、入庫・廃棄の create と棚卸しの確定が保存の前に `±9007199254740991` で検査する（下の「入力の上限と安全な整数の範囲」の (c)）。

**SPEC-UNIT-D7 売価 × 数量の金額（手動販売の金額の初期値）**: `売価(円) × 数量 ÷ 基準数量` を円未満で四捨五入する（レジと同じ四捨五入〈台帳 L-136〉。`決定（owner、D-113 J4）`、2026-10-08、TD-195。不採用: 切り上げ〈店の切り売りの端数、台帳 L-135〉）。用途は [62](62-ui-manual-sale.md) UI-04-D18 の初期値だけで、利用者が直せる。

**SPEC-UNIT-D8 POS の数量（Z004・EJ）**: 本節は数量の型の契約だけを決め、Z004 の取込みの再開と EJ の取込みの配線は後続の lane（CASIO 固有）が行う。
- IO: 数量を小数 2 桁までの 100 倍の整数（`quantity_hundredths: i64`）で読む。浮動小数を通さない（D-104 の日報の個数と同じ表し方。helper は [29](29-io-daily-report-parser.md) IO-07-D2 の読み方を共有する）。レジの乗算の数量は 0.01〜9999.99（取説）で、店の実例は小数 1 桁（Z004・EJ・精算レシートとも。台帳 TD-147）。小数 3 桁以上・指数表記・空は今と同じ `InvalidNumber`。IO は商品を引かず、単位を知らない。
- BIZ: 商品を引いた後に在庫の数量へ換算する。個数の単位: `quantity_hundredths` が 100 で割り切れれば その商 を在庫の数量にし、割り切れなければその行を在庫に効かせず「整数でない数量」として利用者に示す（黙って丸めない。`1.00` は 1 として受ける）。長さの単位: 在庫の数量（cm）= `quantity_hundredths × 基準数量(100) ÷ 100` = `quantity_hundredths`（D4 の不変条件。1 m の 1/100 = 1 cm）。
- EJ の明細の金額の照合（今は `数量 × 単価 = 金額`）は、小数の数量では `quantity_hundredths × 単価 ÷ 100` を円未満で四捨五入した値と金額を比べる（レジの丸め、台帳 L-136）。実データでの確認は EJ の lane の Plan Gate の前。
- 金額は円の整数のまま。file の hash は raw bytes のまま（正規化した値で作り直さない）。日報（Z001・Z005）は D-104 のまま。

**SPEC-UNIT-D9 UI の twin と formatter**: UI の数量の換算・表示（D3。集計の規則で画面が数量の種類ごとに足す所〈日次売上の部門の小計、月次の合計〉と単価を含む）と、入力中の見込みの金額（廃棄の入力画面の合計、手動販売の金額の初期値）は TS の純関数で持つ。BIZ の関数と同じ意味の意図的な二重実装とし、同じ例の表（golden）を Rust と TS の両方の test に独立に写して意味のずれを止める（BIZ-01-D2 と同じ形）。保存された記録の金額は BIZ が返した値を表示する。BIZ が数量を文に入れる所（在庫警告、D3 の表の 10）は、同じ規則の Rust の関数 `format_length_m` で整形し、golden の表を TS の formatter と共有する。formatter は `ProductStockUnit` を受けて全 variant を網羅する（wildcard を置かない。単位を足すと compile error）。

**SPEC-UNIT-D10 在庫少の判定と商品 CSV**: 在庫少は、個数の単位（9 個）を一般の基準、長さの単位（`m`）を生地の基準で比べる（数量の種類の分岐。SQL の code の並びは BIZ が `ProductStockUnit::ALL` を `stock_unit_kind` で個数と長さの 2 つに分けて作り、repo は受けた並びで 2 つの `IN (…)` を組むだけ。どちらにも入らない単位が在庫少から黙って漏れることを test で止める）。基準の画面の語と cm の入力は変えない。商品 CSV の任意列 `在庫単位` は code（`pcs` 等）か表示の語（`個` 等。D1 の表の 10 語）を受け、code に正規化する。空は `pcs`（今どおり）。それ以外は preview の行の error。`原価` の列は D5 の入力の規則で小数 2 桁まで受け（D-128）、行の error は、先頭が `-` で残りが規則に合えば `原価は0以上で入力してください`、それ以外の形の誤りは D5 の error の文（直し方を名指す）。`在庫単位` の値が作り直しの起点（D1）で外した語と code（`巻`・`roll`・`cm`）のときは、行の error の文に直し方を添える（`cm` は m で入れて初期在庫の cm の数は 100 で割る、`巻` は 個 で入れる。巻は個数の単位で数の意味が同じ: D2）。この 2 つの文の出所は原価の lane の Plan Review の AI review。

**SPEC-UNIT-D11 wire の単位の型**: wire に出る `stock_unit` はすべて generated enum `ProductStockUnit` にする（記録詳細の明細 6 種〈入庫・返品・手動販売・廃棄・CSV 取込み・棚卸し〉を含む）。棚卸しの計数の明細 `StocktakeItemDetail` は `stock_unit` を持つ（D-118 の b1。m の実数の入力に要る。[73](73-ui-stocktake.md)）。repo は DB の値を `parse_stock_unit` で読み、一覧に無い値は読取りの error。file 由来の商品 CSV の行（`ImportRow.stock_unit`）は `String` のまま（D-061 / D-064 の二層）。D3 の「表示する所」のために、`DailySaleItem`・`IntegrityMismatch`・`StockAdjustment`・`ProductResponse` は `stock_unit: ProductStockUnit` を必須の field として持ち（optional にしない）、`DeptSubtotal`・`GrandTotal`・`MonthlySaleItem` は数量を `count_points: i64`・`length_cm: i64`（D3 の集計の規則、TD-206・TD-207）の 2 つの必須の field で持ち（`quantity` の名前は残さない）、`MonthlySaleItem` は `stock_unit: ProductStockUnit | null`（商品別の行だけ値。部門別は null）を持つ。`MovementRecord` は変えない（在庫変動の画面は商品を 1 つ読んでから表を出す）。

**入力の上限と安全な整数の範囲**（SPEC-UNIT-D11 の続き。2026-10-11 に書き直した）: DB へ書く値・wire に出す値・画面に出す値のうち、cm・1/100 円・点の整数と円の合計を、JS の number で正確に運べる範囲（`-9007199254740991` 以上 `9007199254740991` 以下。以下「安全な整数の範囲」）に収める。計算の経路ごとに計算の後の値を検査するのではなく、入力に現実的な上限を掛けて、この範囲に届く筋そのものを消す（owner の決定、2026-10-11、repo 外の回答台帳 TD-213。経緯は [decision-log](../decision-log.md) D-113）。

(a) 上限（1 つの値ごと。BIZ が検証で掛け〈入力は入口で、在庫は計算の後で〉、画面も同じ上限で先に止める。UI を通らない command でも BIZ が守る）:

| 値 | 上限 | 掛ける所 |
|---|---|---|
| 1 明細の数量（入庫・返品・手動販売・廃棄・振替）。在庫の数量の単位で数える（個数の商品は 1 単位、長さの商品は cm） | 1 以上 `9999999`（長さは `99999.99` m） | 振替（D-143）は `src-tauri/src/biz/inventory_service/transfer.rs` の `create_transfer` の明細の検証で、減らす明細と増やす明細の両方に同じ上限を掛ける（[31](31-biz-inventory-service.md) の同名の節）。ほかの 4 つの create の明細の検証（`quantity <= 0` の隣の `item.quantity > MAX_LINE_QUANTITY`: `src-tauri/src/biz/inventory_service/receiving.rs` の `create_receiving`・`src-tauri/src/biz/inventory_service/manual_sale.rs` の `create_manual_sale`・`src-tauri/src/biz/inventory_service/disposal.rs` の `create_disposal` と、返品の `src-tauri/src/biz/inventory_service/returns.rs` の `validate_return_request_shape`）。どれも在庫を動かすかどうかの分岐より前に全明細を回るので、在庫を動かさない明細（レジ処理済みの返品〈`register_processed`〉は `apply_stock_change` を通らずに `return_items` へ保存される: `src-tauri/src/biz/inventory_service/returns.rs` の `create_return` の `if !req.register_processed` の分岐）も同じ検査を通る |
| POS 由来の数量（商品別売上 CSV の取込み。後続の Z004・EJ の lane は D8 で在庫の数量へ換算した後の値） | 絶対値 `9999999`（符号と 0 は今の扱いのまま） | 取込みの preview が商品を引いて行を採る前（`src-tauri/src/biz/csv_import_service/parse.rs` の `parse_and_validate` の `constants::MAX_LINE_QUANTITY` の検査）。越える行は数値の不正と同じ `invalid_number` の行の error にして、`sale_records` にも在庫にも書かない（在庫に連動しない商品〈`pos_stock_sync=false`〉の行は `sale_records` だけに保存される: `src-tauri/src/biz/csv_import_service/commit.rs` の `execute_commit` の `for row in matched_rows`。この行も同じ検査を通る） |
| 在庫（計算の後の値を含む） | 絶対値 `999999999`（長さは `9999999.99` m。負の在庫は INV-3 のまま `-999999999` まで） | 在庫を書く所の全部が書く直前に `check_stock_range`（`src-tauri/src/biz/inventory_service/common.rs`）を通る。通す所は写しの求め直しの `recompute_stock`（`src-tauri/src/biz/stock_ledger.rs`。求めた値と要再確認の範囲の両端）の 1 か所で、在庫を書く経路は全部ここを通る: `apply_stock_change`（入庫・返品・手動販売・廃棄・振替〈D-143〉の 5 経路）、精算の一覧が変わる TX の求め直し（`recompute_for_settlement_change`。商品別売上 CSV の取込みと取消）、棚卸しの計数の保存（`save_stock_count`・`zero_uncounted_items`、D-118 の b1）、商品登録と商品 CSV の初期在庫と在庫連動の切替（`src-tauri/src/biz/product_service.rs`）、整合性の補正（`src-tauri/src/biz/integrity_service.rs` の `fix_integrity`）。入力の値は別に入口で止める: 商品登録の初期在庫（`validate_create_request` の `initial_stock`。0 以上）、商品 CSV の初期在庫（preview の `preview_import` の「初期在庫」と commit の `import_row_commit_error`。0 以上）、棚卸しの計数の値（0 以上） |
| 売価（円） | 0 以上 `999999` | 商品の create・update・価格改定・商品 CSV の検証（負の検査と同じ `price_range_error`: `src-tauri/src/biz/product_service.rs`。呼ぶ所は `validate_create_request`・`validate_update_request`・`revise_product_price`・`preview_import` と、商品 CSV の commit の `import_row_commit_error`） |
| 原価（1/100 円） | 0 以上 `99999999`（`999999.99` 円） | 商品の create・update・価格改定・商品 CSV（売価と同じ `price_range_error`: `src-tauri/src/biz/product_service.rs` の `validate_create_request`・`validate_update_request`・`revise_product_price`・`preview_import` と、商品 CSV の commit）と、入庫・廃棄の明細（`src-tauri/src/biz/inventory_service/receiving.rs` の `create_receiving`・`src-tauri/src/biz/inventory_service/disposal.rs` の `create_disposal` の `MAX_COST_PRICE_CENTI`）の検証（D5） |

- 上限を越えた入力は `ValidationFailed` で、何も書かない（TX の中で見つけたときは TX 全体を戻す）。画面は同じ上限で先に止めて入力の error を出す。file の行（商品 CSV、商品別売上 CSV）は行の error。文は `数量が大きすぎます`・`在庫数が大きすぎます`・`売価が大きすぎます`・`原価が大きすぎます`（D5）で、明細の検証は今の `明細{n}: ` を前に付ける。
- 4 つの値は定数で 1 か所に置き、各検証はそこから引く（Rust は `src-tauri/src/constants.rs`〈今の `CSV_IMPORT_LINE_LIMIT` と同じ file〉、TS は `src/lib/` の 1 file）。
- 数字の根拠: 1 明細の数量は、レジの乗算の数量の上限 9999.99（D8）を、長さの商品（999,999 cm）で 10 倍、個数の商品（9,999 個）で 1,000 倍に取った値。在庫は 1 明細の数量の約 100 倍。売価は PLU の単価の 6 桁（[25](25-io-plu-formatter.md) の境界テスト「selling_price = 999999 → "999999"（6桁上限）」）。原価は売価と同じ桁（円で 6 桁。1/100 円にすると小数 2 桁が足される）で、`売価 × 原価` が安全な整数の範囲に入り、価格改定の新原価（案）の TS の twin（`src/features/products/lib/price-revision-math.ts` の `deriveProposedCost`、number の掛け算）がそのまま正確になる（(b)）。
- 掛ける lane: 4 つの上限と、入庫・廃棄の合計の検査（(c)）は現行。原価の上限は原価の lane が `99999999`（1/100 円。定数 `MAX_COST_PRICE_CENTI`）にした（v8 の円の上限 `999999` から桁が増えるだけで、円での上限は同じ桁）。棚卸しの実数の上限と確定の検査は D-118 の b1 で入った。Z004 の整数の数量の換算は b1、小数の数量（POS の数量の換算の残り）は Z004・EJ の lane。移行の前に保存済みの値は [22](22-mnt-migration.md) §16 の vU の上限の検査が、原価の 6 列を含めて確かめる。原価の 1/100 円は原価の列を持つ表に行の無い DB にだけ入れる（22 §16 の vB、D-119）ので、円の値を 1/100 円へ移す経路は無い。

(b) 上限から導ける範囲（上限どうしの演算の結果。どれも安全な整数の範囲に入るので、個別の検査を置かない。2026-10-11 に python の整数で計算した）:

| 値 | 式 | 絶対値の最大 |
|---|---|---|
| 写しと movement の `stock_after`（入出庫・取込みの取消・棚卸しの補正の後に、記録から求め直した値） | 計数と記録の和（[31](31-biz-inventory-service.md) の「計数の置き場と在庫の求め直しの契約」。差分を写しに足す経路は無い） | 記録の数で決まり、上限からは導けない。この行だけは検査を置く: 和は `biz::stock_ledger` で `checked` に足して桁あふれを `ValidationFailed` にし、書く前に `recompute_stock` が在庫の上限で検査する |
| 棚卸しの差異・補正量と、変動の `quantity` | 実数 − 在庫 | `999999999 + 999999999 = 1999999998` |
| 手動販売の金額の初期値（円、D7） | 売価 × 数量 ÷ 基準数量 | `999999 × 9999999 ÷ 1 = 9999989000001` |
| 入庫・廃棄の 1 行の金額（D6） | 原価 × 数量 ÷ 基準数量 | `99999999 × 9999999 ÷ 1 = 999999890000001`（1/100 円。`9999998900000.01` 円） |
| 価格改定の新原価（案）と現掛率の TS の twin の積（SPEC-PRV-D4） | 新売価 × 現原価、現原価 × 10 | `999999 × 99999999 = 99999899000001`、`99999999 × 10 = 999999990` |

- 整合性の差の行は置かない（D-118 の (a) で整合性の画面と操作ログは差を出さず、保存している値と記録から求め直した値を並べる。[75](75-ui-integrity-check.md) §75.6）。
- 行の金額（1/100 円）は、上限の中でも棚卸しの行（`99999999 × 999999999 = 99999998900000001`）が安全な整数の範囲を越えるので、今までどおり十進の文字列で運ぶ（D6）。

(c) 上限からは言えない所（円の合計。保存の前に 1 回だけ検査する）: 明細の行数と商品の数には上限を置かない（店の実用を縛る）ので、円の合計はどの上限でも届く。入庫・廃棄は、上限の明細（個数の単位、数量 `9999999`、原価は上限）900 行までは範囲内で、901 行で越える（`8999999010000009` 円と `9009999008900009` 円）。棚卸しは、原価と実数（`999999999`）が上限の個数の単位の商品 9 つまでは範囲内で、10 個で越える（`8999999901000000` 円と `9999999890000000` 円）。合計を決める保存の操作が、書く前に検査する:

| 合計（円） | 検査する所 | 状態 |
|---|---|---|
| 入庫の `total_cost` | `create_receiving` | 現行 |
| 廃棄の `total_loss_cost` | `create_disposal` | 現行 |
| 棚卸しの `stocktakes.total_cost`（評価額の総額） | 棚卸しの確定 | 現行（D-118 の b1） |

- 検査: 明細から `cost_line_centi` と `cost_total_yen` で合計を求め、`check_safe_integer`（`±9007199254740991`）に通す。範囲外は `ValidationFailed`（`合計金額が大きすぎます`）で TX 全体を戻し、何も書かない。`cost_total_yen` 自身には検査を足さない（今の test の境界〈`i64::MAX`〉を保つ）。
- 保存の後の読取りは範囲で失敗しない: 入庫・廃棄の明細は保存の後に書き換えず、許す単位の変更（D2 の個数の単位どうし）では価格の基準数量が変わらないので、記録詳細が同じ明細から求め直す合計は、保存の前に検査した値と同じになる。棚卸しの総額は保存した値をそのまま出す。記録詳細の読取りに範囲の検査を置かない。v8 より前に保存した記録は、v8（vU）の上限の検査が同じ範囲で確かめる（[22](22-mnt-migration.md) §16）。
- 棚卸しの確定: 完了の TX が総額を同じ範囲で検査してから保存する（`src-tauri/src/biz/stocktake_service.rs` の `complete_stocktake` が `valuation_total_yen` の結果を `check_safe_integer` に通す。D-118 の b1）。
- 円の合計を wire・DB に新しく足す lane（例: 棚卸し記録詳細のロス原価の合計）は、その値を決める保存の操作に同じ検査を足す。
- TS の twin: 廃棄の入力画面の合計は `BigInt` で計算し（1/100 円の原価の例: 上限の明細 11 行に原価 `38`・数量 1 の 1 行を足した 1/100 円の和 `10999998790000049` は `109999987900000` 円だが、number で足すと `109999987900001` 円になる。node で確かめた、2026-10-11）、合計が範囲外なら合計の欄に `—` を出す（保存は上の検査が拒む）。手動販売の金額の初期値は number の整数の演算で正確（(b)）で、数量が上限を越えている間は金額を求めない（初期値は空。保存は数量の入力の error で止まる）。

(d) 集計（読取りを範囲で拒まない）: 日次・月次の数量の和（`DailySaleItem.quantity`、`count_points`・`length_cm`、部門小計・合計、売上の CSV）は、1 明細の数量が絶対値 `9999999` 以下なので、`9007199254740991` を越えるには 1 つの集計（1 日か 1 か月の、1 商品・1 部門・合計）に 900,720,016 件の明細が要る（900,720,015 件 × `9999999` = `9007199249279985` までは範囲内。1 回の取込みは 10,000 行までなので 90,073 回ぶん）。記録から求め直した値（整合性チェックが写しと比べる値。D-118 の (a)）は、在庫を書く経路がどれも同じ TX で記録を書いてから写しを求め直し、書く前に在庫の上限で検査する（`recompute_stock`。`apply_stock_change`、初期在庫、棚卸しの補正、取込みの取消、整合性の補正）ので、写しに等しく、絶対値 `999999999` 以下（計数の無い商品では、求め直した値は無効でない変動の和と同じ）。どちらも届かないので、読取りを安全な整数の範囲で検査して拒む契約と、拒んだ後の回復の契約は置かない。演算は checked にし、i64 の溢れは今までどおりの error（`ValidationFailed`。SQL の `SUM` の溢れは SQLite の error）で、これは範囲の契約ではなく既存の溢れの扱いである。TS の再集計は `Number.isSafeInteger` を外れたら `—`（D3 の集計の規則）。

(e) 表の外:

- 売上の金額と、その集計（POS の金額、手動販売の金額欄の値。円の wire）。
- レジの日報（Z001・Z005）の数量（D-104、今のまま）。
- 在庫少の基準（`app_settings` の `stock_low_threshold`・`stock_low_threshold_fabric`）。画面の入力は 1〜99999 の整数で（[69](69-ui-threshold-settings.md) UI-11a-D3）、BIZ は読んだ値を在庫の数量と比べるだけで、足し算・掛け算に入れない（`src-tauri/src/biz/product_service.rs` の `list_low_stock`、`src-tauri/src/db/product_repo.rs` の `list_low_stock_products` の `stock_quantity <= ?`）。基準の単位は D10 のまま（一般は個数の単位の数、生地は cm）で、本節の上限と検査は足さない。
- 上限の外の値を持つ DB（DB を直接書き換えた、壊れた）。移行は上限の外の保存済みの値を通さず（[22](22-mnt-migration.md) §16）、その後の書込みは (a)(c) が拒むので、アプリの操作では作られない。この状態では、演算の溢れは既存の error を返し、溢れない値はそのまま出る（number で正確に運べる保証は無い）。

**シグネチャ**（BIZ の module `biz::unit_amount`。35 §20.5a の計算もここにあり、棚卸し・入出庫・売上・在庫少が共有する）:
```
pub enum StockUnitKind { Count, Length }
pub fn stock_unit_kind(unit: ProductStockUnit) -> StockUnitKind
pub fn price_basis_quantity(unit: ProductStockUnit) -> i64
pub fn cost_line_centi(cost_price_centi: i128, quantity: i64, basis: i64, product_code: &str) -> Result<i128, BizError>
pub fn cost_total_yen(lines: &[i128]) -> Result<i64, BizError>
pub fn sale_amount_yen(selling_price: i64, quantity: i64, basis: i64) -> Result<i64, BizError>
pub fn stock_quantity_from_pos(quantity_hundredths: i64, unit: ProductStockUnit) -> Result<i64, PosQuantityError> // proposed・未実装（Z004・EJ の lane、D-113。D8）
pub fn check_safe_integer(value: i128, what: &str) -> Result<i64, BizError> // 円の合計の保存の前の検査（入庫・廃棄の create、棚卸しの確定）。±9007199254740991 の外は ValidationFailed
pub fn format_length_m(quantity_cm: i64) -> String // D3 の m の表示（-30 → "-0.3 m"）。在庫警告と売上の CSV で使う
```

---

## 共通型定義

### DbError列挙型

```
enum DbError {
    ConnectionFailed(String),
    PragmaFailed(String),
    MigrationFailed(String),
    QueryFailed(String),
    DuplicateKey(String),
    ForeignKeyViolation(String),
    NotFound,
    SchemaNewerThanApp { db_version: i64, app_max: i64 },  // DB の版がアプリより新しい（22 MNT-03-D11）
}
```

### PaginatedResult構造体

```
struct PaginatedResult<T> {
    items: Vec<T>,
    total_count: u32,
    page: u32,
    per_page: u32,  // 実際に適用された1ページ件数。上限挙動は各一覧APIの契約に従う
}
```

Pagination upper-bound policy is intentionally module-specific. D-031 introduced the real shared `PAGINATION_MAX_PER_PAGE = 200` constant for IO-layer clamps: `search_products`, stocktake item lists, and system log lists clamp to 200 and return the clamped value in `PaginatedResult.per_page`. D-081 raises the inventory movement / record BIZ lists' `MAX_PER_PAGE = 200` reject boundary without changing the reject mechanism, while sales import history lists keep their existing 100 reject behavior.

---

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | D-071 / SPEC-SDI-D1〜D4: CSV取込みの自然冪等性を同日別hashの追加取込みとper-import訂正へ改訂。 |
| 2026-10-07 | 単位の拡張 design lane | D-113 / SPEC-UNIT-D1〜D11（proposed・未実装）: 単位を 12 種へ、長さの m の入力と表示、価格の基準数量と POS の数量 1、原価の 1/100 円、原価 × 数量・売価 × 数量の丸め、POS の数量の 100 倍の整数、在庫少と商品 CSV の単位、wire の単位の enum を追加。 |
| 2026-10-08 | 単位の拡張 design lane | D-113 の owner の判断 J1〜J4 の決定（TD-195）を SPEC-UNIT-D1・D5・D6・D7 に反映（`ball` を入れる、原価は 1/100 円、入庫の丸めは TD-023 と同じ、手動販売の金額の初期値は四捨五入）。 |
| 2026-10-08 | 単位の拡張 design lane（Plan Review round 1） | SPEC-UNIT-D3 に表示する所の表（owner の決定 TD-203）と、数量の種類ごとの 2 本に分ける集計の規則（owner の決定 TD-206・TD-207）・単価の規則、商品 CSV の `初期在庫` の m（Coordinator の決定、L-214）、入力の行の数量の加算を追加。D2 に CSV の上書きで単位を変えないこと、D6 に原価の引数の i128 と行の金額の wire の十進の文字列、D11 に表示のための wire の field を追加。 |
| 2026-10-11 | 単位の拡張 design lane（Final Review、Gated Amendment 5） | SPEC-UNIT-D11 の続きの「安全な整数の範囲」（経路ごとに保証する所を並べた 12 行の表）を「入力の上限と安全な整数の範囲」に書き直した（owner の決定 TD-213）: 1 明細の数量・在庫・売価・原価の上限、上限から導ける範囲、円の合計の保存の前の検査（入庫・廃棄・棚卸しの確定）、集計の読取りを範囲で拒まないこと、表の外。SPEC-UNIT-D3 の m の入力に 0 の扱いと欄ごとの上限、D5 の原価の上限、D6 の行の金額の例を合わせた。 |
| 2026-10-11 | 単位の runtime の lane の起票 | 「入力の上限と安全な整数の範囲」の (e) 表の外に、在庫少の基準（`app_settings`）を 1 行足した（設計 lane の Final Review の P3）。契約の値は変えない。 |
| 2026-10-13 | 単位の runtime の lane | 単位の分（SPEC-UNIT-D1〜D4・D6・D7・D9〜D11、入力の上限と安全な整数の範囲）を現行の契約にした（migration v8）。D5（原価の lane）・D8（Z004・EJ の lane）・棚卸しの分（棚卸しの再開）は未実装のまま。集計の `N 点` の 3 桁区切り、表示する所の表に商品修正の画面の現在庫、表の外に数量の生の値の並べ替えを足した。 |
| 2026-10-17 | 単位・数量・原価の精度の見直しの design lane | D-119（proposed・未実装）: D5 の原価の 1/100 円を既存の DB の変換でなく開発・試験 DB の作り直しの起点（22 §16 の vB）で入れる、D1 の巻・cm をその起点で外す、D2 で単位の語を個数の単位どうしで直せる、D5 に「袋の値段 ÷ 入り数」の入力の補助を足した。D2 の 10 cm 刻みを強制しない理由と D1 の出典を事実に合わせ、D4・D5 に店に聞く問い（長さの商品の在庫の減らし方、原価の税）と答えが無い間の既定を書いた。 |
| 2026-10-21 | 原価の lane の起票 | D-128: SPEC-UNIT-D5 の税の区分を税抜に決めた（店の答え TD-331）。D5 の表示に記号（各画面の今の `¥`）と入力の欄の初期値の文字列を足した。D1 の巻・cm を外す前提が 2026-10-18 版の在庫 Excel で成り立ったと書いた。D2 の理由を、10 cm は刻み（TD-332）で検査は店主の答え（Q-098）の後、に直した（振舞いは変えない）。 |
| 2026-10-21 | 原価の lane（Plan Review round 1 の是正） | D5 の表示の記号を半角の `¥` に一意にした（今の画面は 3 つの形が混ざる。売価の記号は変えない）。D10 に商品 CSV の `原価` の行の error の分け方（負の文と D5 の形式の文）と、`在庫単位` の `巻`・`roll`・`cm` の行の error に直し方を添えることを足した（出所は AI review）。 |
| 2026-10-21 | 原価の lane（Plan Review round 2 の是正） | D5 の入力の欄の初期値の場所に、価格改定の新原価（案）と現売価 0 の行の現原価を足した。 |
| 2026-10-21 | 原価の lane の実装（PR #184） | D5（原価の 1/100 円・入力・補助・表示・税抜）と D1 の巻・cm を外した一覧（10 個）を実装し、`proposed・未実装` の注記を現行の記述に直した（migration は 22 §16 の vB〈v10〉）。D2・D3・D10 の `cm` の分岐、D6・「入力の上限と安全な整数の範囲」の (a)〜(c) の「原価が円の間」の値を外した。D10 の商品 CSV の `原価` と `在庫単位` の文、D5 の formatter と関数の名前を書いた。 |
