# テーブル定義（マスタ）

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D2・D4・D8・D9）の商品の側。以下の既存カラム表は現行スキーマを記す。旧 ADR（時点証拠）の `stock_revision`・`pos_sync_disabled_revision` は足さない。(a)（ADR SPEC-STK-CNT-D12）で入ったのは 1・2 つ目の項目（写しの 3 列は migration v9〈[22 §19](../function-design/22-mnt-migration.md)〉、写しを書くのは `biz::stock_ledger` の `recompute_stock` だけ）。3・4 つ目の項目（在庫連動を始める検査と共有 JAN の不変条件）は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で入った。

- products へ写しの列を足す: `stock_recheck INTEGER NOT NULL DEFAULT 0`（0/1 の CHECK。要再確認なら 1）、`stock_range_low INTEGER NULL`・`stock_range_high INTEGER NULL`（`stock_recheck=1` なら両方 NOT NULL で low ≤ high、0 なら両方 NULL。table の CHECK）。`stock_quantity` と合わせて、BIZ の求め直しの関数だけが書く（[tracking](tracking-system-tables.md) の同名の契約）。
- `stock_quantity` は記録から求め直した写しで、既存の行の更新は求め直しの関数だけを通す（`update_stock_quantity` の呼出し元はこの関数 1 つ）。新しい商品の INSERT の初期数量は、同じ TX で作る初期在庫の movement から求め直した値と等しい。
- `pos_stock_sync` の `DEFAULT 1` は変えない。在庫連動を始める（false → true）ときの検査は [30](../function-design/30-biz-product-service.md) の新契約（連動しない間の Z004 の売上が効く計数より後にあれば拒否）。
- 共有 JAN は DB に存在できる。BIZ-01 の不変条件は、変更の後に在庫連動の商品と同じ JAN を持つ別の商品があってはならない、の 1 つ（どの経路・どちらの商品を変える場合でも拒む。30 の新契約）。jan_code への全件 UNIQUE と既存の pos_stock_sync の一括変更はしない。

## 単位と原価の精度の契約

[共通規則](../function-design/10-common-rules.md) SPEC-UNIT-D1・D5 の DB の側。単位と原価の列は現行のスキーマ（migration v8 と vB〈v10〉）。migration の手順は [22](../function-design/22-mnt-migration.md) §16（原価の改名と単位の CHECK の 10 個は vB）。

- `products.stock_unit` の CHECK は 10 個の code（`pcs` `sheet` `hon` `bag` `box` `kumi` `set` `ball` `cho` `m`）。既定値は `pcs`。`ball` を含める（`決定（owner、D-113 J1）`、2026-10-08、TD-195）。v8（vU）が 12 個に広げ、`roll`・`cm` は開発・試験 DB の作り直しの起点（22 §16 の vB の手順 4）で外した（D-119。前提の巻・cm 0 行は 2026-10-18 版の在庫 Excel で成り立った。[decision-log](../decision-log.md) D-128 (2)）。
- 原価の列は 1/100 円の整数（D-113。`決定（owner、D-113 J2）`、2026-10-08、TD-195）。vB が既存の値を変換せず改名した。vB は原価の列を持つ表に行の無い DB（開発・試験 DB を作り直した起点）にだけ当たる（22 §16、D-119）。対象は次の 6 列で全部:

| 表 | vB の前の列 | 今の列 | 意味 |
|---|---|---|---|
| products | cost_price | cost_price_centi | 原価（1/100 円、価格の基準数量あたり） |
| receiving_items | cost_price | cost_price_centi | 入庫時点の原価（同） |
| disposal_items | cost_price | cost_price_centi | 廃棄時点の原価（同） |
| stocktake_items | valuation_cost_price | valuation_cost_price_centi | 確定時の評価原価（同、NULL 可のまま） |
| price_history | old_cost | old_cost_centi | 変更前の原価（同） |
| price_history | new_cost | new_cost_centi | 変更後の原価（同） |

- 売価（`selling_price`・`price_history.old_selling` / `new_selling`）・`stocktakes.total_cost`（円）・日報と売上の金額は円の整数のまま。
- 数量の列（`stock_quantity`・明細の `quantity`・`inventory_movements.quantity` 等）は整数のまま（SPEC-UNIT-D2。長さは 1 cm）。

---

> **親文書**: [DB_DESIGN.md](../DB_DESIGN.md)

---

## 1. products（商品マスタ）

### 役割
システムの中心テーブル。全ての操作（売上、入庫、返品、棚卸し）がこのテーブルの商品を参照する。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| product_code | TEXT | PK | システム管理の一意コード。JAN個別管理商品はJANそのまま、それ以外は独自コード（例: HZ-0046, NU-0003）。100 文字以内（UTF-16 code unit）。取込み経路で BIZ-01-D5（30-biz）が保証し、DB CHECK は置かない |
| jan_code | TEXT | NULLABLE, INDEX | JANコード（JAN-8 または JAN-13 = ASCII 数字 8/13 桁。形式 validation は手入力 create 経路（51 UI-01b-D17 / 30-biz BIZ-01-D1）と商品 CSV の取込み（30-biz BIZ-01-D10、D-140）が持ち、Z004 の取込みと既存行は対象外）。複数商品が同じJANを共有する場合がある（グループコード）。NULLならJAN無し商品 |
| name | TEXT | NOT NULL | 商品名（例: ハマナカ アミアミ極太 col.42） |
| department_id | INTEGER | FK → departments.id, NOT NULL | 所属部門 |
| supplier_id | INTEGER | FK → suppliers.id, NULLABLE | 主な取引先（任意） |
| selling_price | INTEGER | NOT NULL | 売価（税込、円）。価格の基準数量あたり（下記の設計意図） |
| cost_price_centi | INTEGER | NOT NULL | 原価（1/100 円、税抜。売価は税込）。価格の基準数量あたり（下記の設計意図）。精度と税の区分は共通規則 SPEC-UNIT-D5（D-128） |
| tax_rate | TEXT | NOT NULL, DEFAULT '10' | 消費税率。'10' / '8' / '0'（非課税） |
| maker_code | TEXT | NULLABLE | メーカー品番（例: H180-005-42）。JAN無しでもメーカー品番だけある場合あり |
| stock_quantity | INTEGER | NOT NULL, DEFAULT 0 | 在庫数（個 or 枚 or cm。長さの商品〈`m`〉は cm の整数） |
| stock_unit | TEXT | NOT NULL, DEFAULT 'pcs' | 数量単位。10 個の code（CHECK。上の「単位と原価の精度の契約」） |
| is_discontinued | BOOLEAN | NOT NULL, DEFAULT 0 | 廃番フラグ。1=廃番 |
| plu_dirty | BOOLEAN | NOT NULL, DEFAULT 1 | PLU書出し必要フラグ。新規登録/売価変更でON、UI-08でTSV保存後に利用者が書出し済み確認した時点でOFF |
| plu_exported_at | TEXT | NULLABLE | 最後にアプリ側でPLU書出し済みにした日時。NULLなら未書出し。PCツール受理やレジ反映の証明ではない |
| pos_stock_sync | BOOLEAN | NOT NULL, DEFAULT 1 | CSV取込みで在庫自動減算するか。0=しない（生地カテゴリ等） |
| plu_target | BOOLEAN | NOT NULL, DEFAULT 0 | スキャニングPLU書出し対象か。0=対象外（JANなし商品、JANあり・使ってないグループコード商品、スポット特売品）。migration v3 で追加（D-028） |
| created_at | TEXT | NOT NULL | 作成日時（ISO 8601） |
| updated_at | TEXT | NOT NULL | 更新日時（ISO 8601） |

### 設計意図
- **product_codeとjan_codeを分けた理由（2026-03-28 利用者ヒアリングで判明）**: 布やファスナーのグループコード商品は、複数の色違い/サイズ違いが同じJANを共有する。JANをPKにすると1レコードにまとまってしまう。product_code（独自コード）をPKにして、JANは参考カラムとして持つことで、グループ単位の管理と個別管理を両立
- **jan_codeにINDEXを張る理由**: CSV取込み時にZ004のJANコードでproductsを検索する。INDEXがないと4000件のフルスキャンになる
- **jan_codeがNULLABLEな理由**: ボタンのように品番もJANもない商品がある
- **JANの無い商品の規模（店主回答、2026-03）**: 主に生地とヘア雑貨で、全体の約2割
- **jan_codeのUNIQUE制約をつけない理由**: 同じJANを複数商品が共有するケースがある（グループコード）。UNIQUE制約をつけると登録できなくなる
- **jan_code の形式 validation を DB に置かない理由（2026-08-11、JAN 専用欄正規化 change）**: JAN-8/13 + チェックディジット検証は手入力 create 経路の BIZ 保存時（BIZ-01-D1）と frontend（51 UI-01b-D17）、商品 CSV の取込みの preview と commit（BIZ-01-D10、2026-10-27、D-140）が所有し、DB CHECK は追加しない。既存 DB 行には非 JAN 値・非 13 桁値が実在し得るため（既存データ互換。BIZ-01-D10 より前の CSV の取込みは検証しなかった）、DB 制約は既存データの migration を強制してしまう。UNIQUE を付けない既存判断も不変。
- **selling_price / cost_price_centi をINTEGERにした理由**: 浮動小数点の丸め誤差を避ける。売価は円の整数、原価はばらした商品の 1 個の原価を小数第 2 位まで持つので 1/100 円の整数（共通規則 SPEC-UNIT-D5）
- **価格の基準数量（2026-09-25、[35 §20.5a](../function-design/35-biz-stocktake-service.md#205a-評価額の計算価格の基準数量と店の丸め) SPEC-STK-VAL-D1）**: selling_price / cost_price_centi は、在庫数量で「価格の基準数量」ぶんに対する価格である。基準数量は stock_unit で決まり、個数の 9 単位 = 1（1 個あたり）、`m` = 100（1 m あたり。[共通規則](../function-design/10-common-rules.md) SPEC-UNIT-D4）。反物などの長さ商品は在庫を cm で持ち、価格は値札・仕入れ伝票と同じ 1 m あたりで登録する（店は残り・仕入れ・値札をすべて m で扱う。レジでも数量 1 = 1 m で打てる、未運用）。基準数量の列は持たない（理由・不採用案・見直す条件は 35 §20.5a）。棚卸しの評価額、入庫の原価小計、廃棄のロス原価、手動販売の金額の初期値（[62](../function-design/62-ui-manual-sale.md) UI-04-D18）はこの基準数量で割る（共通規則 SPEC-UNIT-D6・D7）。棚卸し記録詳細のロス原価も BIZ が同じ基準数量で割る（D-118 の b1。[35](../function-design/35-biz-stocktake-service.md) §20.5a）
- **plu_dirtyフラグの理由**: 「レジ登録データ作成」画面の差分書出しモード（REQ-402）で「前回以降に変更があった商品」を高速に抽出するため。売価変更・新規登録でON、TSV生成だけではOFFにせず、UI-08で保存後に利用者が書出し済み確認した時点でOFFにする（D-027）。
- **plu_exported_atの理由**: plu_dirtyだけでは「未書出し」と「アプリ側では書出し済み」の区別がつかない。書出し済み確認日時を記録しておけば「最後にアプリでPLU TSVを作成・保存済みにしたのはいつか」が分かる。ただしPCツール受理やレジ側反映はこの値では証明しない（コードと名称の反映は次の締めの Z004 の照合で分かるが、単価などは分からない。D-117 (7)）。
- **pos_stock_syncの理由（指摘#9対応）**: REQ-401「生地カテゴリの在庫減算除外」の判定カラム。長さの単位（`m`）だけで判定すると長さの商品でもCSV連動したい商品が将来出たときに困る。明示的なフラグで制御する
- **plu_targetの理由（D-028、2026-07-03）**: 「レジでスキャン販売する商品か」は JAN の有無と相関するが一致しない（JANあり・使ってないグループコード商品、valid JAN でも PLU 枠を割かないスポット特売品が現場に実在する）。jan_code からの純導出では表現できないため、pos_stock_sync と同型の明示フラグにする。UI-01b が jan_code の 13 桁数字判定から初期値を提案し、利用者が変更できる。DB DEFAULT は安全側の 0（レジ登録は opt-in）。plu_target=0 の商品は PLU 書出し抽出と UI-00 PLU未反映通知の対象外になり、部門売り + 手動販売出庫（UI-04）で扱う。インデックスは追加しない（plu_dirty との複合検索は数千行規模のためフルスキャンで十分）
- **stock_quantityを商品テーブルに持つ理由**: 毎回inventory_movementsを集計するより、現在の在庫数をここにキャッシュしておく方が画面表示が速い。写しの 4 項目（`stock_quantity`・要再確認の印・範囲）が記録から求め直した値と合っているかは整合性チェックで突合する（D-118 の (a)）

### 他の案と不採用理由
- **INTEGER連番IDをPKにする案**: リレーションのJOINは速くなるが、8テーブルが参照するため毎回ID↔コード変換が必要。4000商品規模ではTEXT PKでも問題ないため不採用
- **在庫数をinventory_movementsからの集計のみで管理する案**: 正確だが、商品一覧画面で4000件の在庫数を表示するたびに全変動履歴をSUMするのはパフォーマンスが不安。キャッシュ＋定期的な整合性チェックの方が現実的
- **jan_codeをPKにする案（初期設計）**: グループコード商品の存在が判明したため不採用。同じJANで複数商品を管理できない

### 困りそうなケース
- **独自コードの接頭辞変更**: product_codeがPKなので全参照を書き換える必要がある。→「独自コードは変更不可」を仕様で確定済み（SP-102-04）
- **CSV取込み時にJANコードで複数商品がヒットする場合**: グループコード商品。Z004で売れた商品が「グループ全体で3個」なのか「特定の色が3個」なのか判別できない。→ 同じ JAN で色・柄が違う品は別の商品で、その群は全色を独自コードにし、JAN を `jan_code` に持たせる（repo 外の回答台帳 TD-334・TD-344・TD-352）。例外（D-141、proposed〈owner の確認待ち〉・実装済み〈PR #212、L3 待ち〉）: 1 色目を先に商品フォームで登録すると商品コードが JAN になり、2 色目以降を独自コードで加えても 1 色目の商品コードは JAN のまま残る（商品コードは主キーで、修正の画面でも読取専用。変えると全参照の書換えになる、上の項と同じ理由）。既存の商品を振り直さない（[30](../function-design/30-biz-product-service.md) BIZ-01-D11 の TD-352 からの例外の項）。今の照合の規則は [pos-tables](pos-tables.md) の Z004 のマスタ照合（`ORDER BY product_code ASC` の先頭へ紐付ける）で、先頭へ割り当てる限界は [backlog](../backlog.md) の DATA-2。共有 JAN の行を在庫連動の外（売上だけ記録）にする形は D-118 の Compatibility と本書の頭の「計数の置き場と在庫の求め直しの契約」が決めた後の形（(b) は proposed・未実装）で、今の build は先頭へ紐付ける
- **布の管理パターンが複数ある**: 反物（m管理。在庫は cm の整数）、カット生地（枚管理）、パッチワーク（枚管理）、セール品（不定期）。stock_unitの'pcs'が「個」にも「枚」にも使える設計。反物はstock_unit='m'

### 布の管理5パターンとDB上の扱い

| パターン | JAN | メーカー品番 | product_code | stock_unit | 在庫の意味 |
|---------|-----|-----------|-------------|-----------|----------|
| 反物 | 無し | あり/なし | 独自コード（NU-xxxx） | m | 残り何 m（在庫は cm の整数） |
| 2mカット生地 | 無し | 無し | 独自コード（NU-xxxx） | pcs | 何枚 |
| パッチワーク（セット仕入れ） | あり（使ってない） | あり/なし | 独自コード（NU-xxxx） | pcs | 何枚 |
| 定番ファスナー（オーダーサイズ） | グループコード | あり | 独自コード（FS-xxxx） | pcs | 何本 |
| セール品 | あり/なし | あり/なし | JANまたは独自コード | pcs | 何個 |

### 業務シナリオ例
```
入庫: ハマナカ アミアミ極太 col.42を12個入庫
→ products.stock_quantity: 5 → 17（+12）
→ products.updated_at: 更新

精算後のCSV取込み: 同商品が3個売れた
→ products.stock_quantity: 17 → 14（-3）

売価変更: 594円 → 648円
→ products.selling_price: 648
→ products.plu_dirty: 1（書出し必要）
→ price_historyに変更前後を記録

廃番特価: 商品Xを正規売価594円→特価300円に値下げ
→ products.selling_price: 300
→ products.plu_dirty: 1
→ price_historyに594→300を記録
→ PLU TSV保存→PCツール投入を確認→UI-08で書出し済みにする→レジに反映→通常通りJANスキャンで販売
→ 全部売れたらis_discontinued=1
```

### 廃番特価の対応方針（2026-03-29 利用者ヒアリングで確定）

**背景**: 廃番特価品は約150品。全色値下げが大半、指定色値下げは少数。現状はJANを無視して金額打ち（部門キー）しており、Z004に載らず在庫が減らない問題がある。

**方針**: 新しい仕組みは不要。既存の売価変更＋PLU書出しフローで対処する。
- **全色値下げ（大半）**: products.selling_priceを特価に変更→plu_dirty=1→PLU書出し→レジでJANスキャン。元の正規売価はprice_historyに残る
- **指定色だけ値下げ（少数）**: 色ごとに別JANがあるなら上記と同じ。グループコード商品（同一JAN）の色指定特価が必要になったら、その色だけ独自コードで別商品化（案2）。現時点では発生頻度が低いため都度対応

---

## 1a. product_jan_aliases（前の JAN の別名、proposed・未実装、D-122）

### 役割

商品の JAN を直したときに、前の JAN のバーコードが付いた現物が店にあると答えた商品について、前の JAN をその商品の別名として持つ（owner の決定、repo 外の回答台帳 TD-284・TD-286。D-122）。売上の取込み（[32](../function-design/32-biz-csv-import-service.md) §15.3 手順 4 b）は、JAN を持つ商品が無い行でだけ別名を引き、同じ商品へたどる。PLU の枠（[plu-tables](plu-tables.md)）とは独立した表で、枠の状態・照合・書出しは別名を読まず書かない（D-122 (2)）。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---|---|---|---|
| old_jan | TEXT | PRIMARY KEY, CHECK(old_jan <> '') | 前の JAN。直す前の `products.jan_code` の文字列をそのまま持つ（数に変えない。照合は `products.jan_code` と同じ文字列の一致） |
| product_code | TEXT | NOT NULL, REFERENCES products(product_code) | 別名でたどる先の商品 |
| created_at | TEXT | NOT NULL | 別名を書いた日時（日時書式規約） |

1 つの前の JAN は 1 つの商品だけを指す（PRIMARY KEY）。1 つの商品は複数の別名を持てる（JAN を 2 回直したとき）。

### 不変条件と trigger

**どの商品も持たない JAN だけが別名になれる**（D-122 (3)）。DB の trigger で両方向を縛る（経路ごとの消去を書かない）:

| trigger | 契機 | 処理 |
|---|---|---|
| `product_jan_aliases_yield_on_insert` | `AFTER INSERT ON products`（`NEW.jan_code IS NOT NULL`） | `DELETE FROM product_jan_aliases WHERE old_jan = NEW.jan_code` |
| `product_jan_aliases_yield_on_update` | `AFTER UPDATE OF jan_code ON products`（`NEW.jan_code IS NOT NULL`） | 同じ |
| `product_jan_aliases_reject_held` | `BEFORE INSERT ON product_jan_aliases`（`products` にその JAN を持つ行がある） | `RAISE(ABORT, 'jan_alias_held_by_product')` |

- 商品がその JAN を持った時点で別名は消える（商品フォームの新規、商品 CSV の新規・上書き、将来の経路、test の fixture のどれでも）。消えた別名は戻らない（その後にその商品が JAN を手放しても、前の別名は生き返らない）。
- 別名の行は書き換えない（repo に更新の関数を置かない。消すのは上の trigger だけ）。別名の UPDATE は DB では拒まない（trigger は INSERT だけを縛る）。別名を更新する経路を足すときは、同じ不変条件の `BEFORE UPDATE` の拒否の trigger を同じ migration の規則（22 §18）で足す。商品は物理削除しない（INV-8）ので、`product_code` の参照は切れない。
- 書く経路は BIZ-01 の商品 CSV の上書き（[30](../function-design/30-biz-product-service.md) BIZ-01-D9）だけ。読む経路は BIZ-03 の商品の引き当て（32 §15.3 手順 4 b）と、BIZ-01 の preview の warning（30 BIZ-01-D9 の「preview の warning」）と commit の判定表の J5（30 §4.9.2）だけ（問いの判定は `products.jan_code` だけを読み、別名を読まない）。

### migration（D-122、runtime の lane）

表と 3 つの trigger を作る migration を 1 本足す（番号はその lane が決める。既存の表を変えないので DB の作り直しは要らない。[22 §18](../function-design/22-mnt-migration.md)）。`products` を作り直す migration は、作り直しの前に `product_jan_aliases_reject_held` を消し、作り直しの後に 3 つの trigger を作り直す（22 §18 の規則。`DROP TABLE products` は `products` の 2 つの trigger を消し、残った別名の表の側の trigger が `products` を参照するため `ALTER TABLE products_new RENAME TO products` が失敗する）。

### 設計意図

- **商品の側に別の表を置く理由**: 別名を PLU の枠（`plu_slots`）の列に持たせる案は、枠の状態の CHECK・照合の遷移・書出しと継ぎ目を作り、枠の遷移のたびに別名の扱いを足す必要があった（D-117 の Plan Review round 2 の 4 件。D-122 の Alternatives）。別名は「売上の行の JAN からどの商品へたどるか」の規則で、枠がレジにあるかどうかに依らない。
- **trigger で消す理由**: 消す処理を JAN を書く経路ごとに書く形は、経路の挙げ漏れで古い別名が生き返る（同じ round 2 の指摘）。商品の JAN の書込みは全て `products` の INSERT / UPDATE を通るので、表の trigger 1 か所で全ての経路を覆う。

---

## 2. departments（部門マスタ）

### 役割
商品の分類（部門）を管理。Z005の部門名との対応付け、独自コードの自動発番の管理も担う。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 部門ID |
| name | TEXT | NOT NULL, UNIQUE | 部門名（例: 毛糸、ヘア雑貨） |
| z005_name | TEXT | NULLABLE | Z005でのCSV上の部門名（整合性検証用） |
| code_prefix | TEXT | NULLABLE | 独自コードの接頭辞（例: F, H）。NULLなら独自コード発番対象外 |
| next_seq | INTEGER | NOT NULL, DEFAULT 1 | 次に発番する連番の下限。発番時に、同じ接頭辞の既存の独自コードの番号の最大 + 1 まで上げてから使う（30-biz BIZ-01-D6） |
| created_at | TEXT | NOT NULL | 作成日時 |

### 設計意図
- **z005_nameの理由**: システムの部門名とZ005のCSV上の部門名が微妙に違う可能性への対策。整合性検証（REQ-403）でZ005の部門名をこのカラムでマッピングする
- **code_prefix + next_seqの理由**: 独自コードの自動発番をDBで管理。「ヘア雑貨」部門でcode_prefix="HZ", next_seq=47なら、次に登録する独自コード商品は"HZ-0047"。発番後にnext_seqを+1する。トランザクションで排他制御すれば番号の重複が起きない。商品 CSV の取込みは next_seq を進めないため、取り込んだ `HZ-0047` などと重ならないよう、発番は既存のコードの番号の最大の次と next_seq の大きい方を使う（30-biz BIZ-01-D6、D-106）

### 他の案と不採用理由
- **独自コード発番を別テーブル（code_sequences）で管理する案**: 部門と1:1の関係なので分ける意味が薄い。departmentsに持たせた方がシンプル

### 困りそうなケース
- **新しい部門が増えたとき**: 例えば「レース」部門を追加したい→INSERTするだけ。code_prefixも設定すれば独自コード発番もすぐ使える
- **Z005に存在しない部門を作りたいとき**: z005_nameをNULLにすれば整合性検証の対象外になる

### 独自コードルール（C-1、2026-03-29 確定）

**形式**: `接頭辞-4桁以上の連番`（例: HZ-0001, NU-0023, BT-0001。HZ-9999 の次は HZ-10000）
- 接頭辞は2文字のアルファベット大文字で統一
- JANコード（数字始まり）と絶対に衝突しない
- next_seq は部門ごと、既存の番号の最大は接頭辞ごと（30-biz BIZ-01-D6）
- 9999 を超えたら 5 桁以上で続ける（owner 決定 2026-10-04 TD-109。各部門数百品の想定だが、上限で止めない）

### 初期データ（全21部門、C-1/C-3 2026-03-29 確定）

部門の数や割当を今後変える可能性を owner は否定していない（owner回答2026-09-19）。

| id | name | z005_name | code_prefix | JAN無し商品 | 備考 |
|----|------|-----------|-------------|-----------|------|
| 1 | その他小物 | その他小物 | KM | あり | |
| 2 | ヘア雑貨 | ヘア雑貨 | HZ | あり | |
| 3 | 毛糸 | 毛糸 | NULL | 基本JAN有り | 特売品は廃番特価フローで対応 |
| 4 | 手芸等材料 | 手芸等材料 | SY | あり | |
| 5 | 他化粧品 | 他化粧品 | NULL | JAN有り | |
| 6 | バッグ | バッグ | BG | あり | |
| 7 | エプロン | エプロン | AP | あり | |
| 8 | 布 | 布 | NU | あり | 反物(m)/カット生地(枚)/パッチワーク(枚) |
| 9 | 糸 | 糸 | NULL | なし | |
| 10 | ゴム | ゴム | GM | あり | |
| 11 | 食品 | 食品 | NULL | 扱いなし | 在庫管理対象外 |
| 12 | 宅急便 | 宅急便 | NULL | - | 在庫管理対象外。サービス取次（ヤマト/佐川）。Z005に載るがシステムでは追跡しない |
| 13 | トワニー | トワニー | NULL | JAN有り | |
| 14 | 雑貨 | 雑貨 | ZK | あり | |
| 15 | 帽子 | 帽子 | NULL | なし | |
| 16 | ビューティ関連 | ビューティ関連 | NULL | なし | |
| 17 | 本 | 本 | NULL | 13 桁 JAN（EAN-13/ISBN-13）有り | ISBN-10 非対応（owner 裁定 2026-08-11。バーコードなし本・ISBN-10 のみの古書は登録不能 — Plans.md backlog で追跡） |
| 18 | ボタン | ボタン | BT | あり | 中分類でレジ打ち |
| 19 | ファスナー | ファスナー | FS | あり | グループコード商品含む |
| 20 | 針 | 針 | NULL | ほぼJAN有り | 単品JAN発行可能 |
| 21 | ノンリンク | ノンリンク | NULL | 使ってない | |

---

## 3. suppliers（取引先マスタ）

### 役割
メーカー/ブランド（値上げ連絡の主体）の名前をマスタ管理する（SPEC-PRV-D1）。owner 語彙ではこれを「取引先」と呼ぶ。入力揺れ防止のためにテーブル化し、商品登録・入庫記録・一括価格改定で候補となる。問屋（発注チャネル）は保持しない。

### カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|---|----|------|
| id | INTEGER | PK AUTOINCREMENT | 取引先ID |
| name | TEXT | NOT NULL, UNIQUE | 取引先名（例: ハマナカ、取引先B） |
| created_at | TEXT | NOT NULL | 作成日時 |
| updated_at | TEXT | NULLABLE | 最終改名日時（migration v6 で追加。既存行は created_at で backfill） |

### 設計意図
- **シンプルにname一つだけ**: 住所・電話番号等は発注システムの領域（スコープ外）。在庫管理システムでは値上げ連絡の主体となるメーカー/ブランドの識別だけが必要
- **漸進補完（SPEC-PRV-D1 / SPEC-PRV-D6）**: 約 80 社を事前一括投入せず、UI-01b / UI-14 から必要なメーカー/ブランドを追加する。`products.supplier_id` は NULL 可のまま、UI-14 で取引先を指定して行確定したときに未設定の商品だけへ漸進的に設定し、既存値は上書きしない
- **問屋チャネルを保持しない**: 本システムは発注を扱わず、値上げ連絡の絞り込み主体はメーカー/ブランドであるため、問屋テーブルや種別カラムは追加しない
- **名称変更（SPEC-SUP-D3 / D5）**: UI-15 の改名は `name` を更新し、実際に名称が変わったときだけ `updated_at` を現在日時へ更新する。同値 no-op では更新しない。migration v6 適用時の既存行は `updated_at = created_at` で backfill し、追加後の列は SQLite 制約に合わせて NULLABLE のままにする。新規作成時は NULL を許容し、改名前の値として扱う
- **重複統合（SPEC-SUP-D4）**: suppliers を参照するテーブルは `products.supplier_id` と `receiving_records.supplier_id` の 2 つ。統合は 1 transaction で両方を残す側へ付け替え、参照が 0 になった消す側の suppliers 行を DELETE する。参照が残った状態の削除経路や単独削除機能は設けない

### 困りそうなケース
- **一回きりの仕入れ先**: 「その他」レコードを1件入れておき、入庫記録の備考に詳細を書く
- **別実体を誤って統合した場合**: 統合は過去の products / receiving_records 参照を一括で付け替える不可逆操作であり、自動復元しない。UI-15 は対象名と影響件数を示す 2 段階確認を必須にする
