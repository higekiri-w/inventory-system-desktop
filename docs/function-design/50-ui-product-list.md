# 50. UI-01a: 商品検索・一覧

> 対応仕様: REQ-103 / REQ-105 / UI-01a
>
> 入力ドキュメント: `docs/architecture/ui-task-specs.md` UI-01a、`docs/SCREEN_DESIGN.md` 商品と在庫画面、`docs/function-design/20-io-product-repo.md` `search_products`、`docs/function-design/30-biz-product-service.md` `search_products`、`docs/function-design/40-cmd-product.md` `search_products`

> **計数の置き場と在庫の求め直し（D-118、b2 で実装済み）**: 一覧の在庫の列は写し（`stock_quantity`）のまま。要再確認（`stock_recheck`）の商品は、在庫の欄に範囲（棚卸しの画面の要再確認の一覧と同じ `formatStockOrRange` の形「{low}〜{high} {単位}」。例「8〜9 個」）と「要再確認」の①状態の badge（文字と icon）を出す（[40](40-cmd-product.md) の商品の結果の列。owner の決定 TD-296 の「範囲で見せる」。部品は [66](66-ui-stock-movements.md) の見出しと同じ `StockWithRecheck`）。並べ替えと絞り込みは写しの値で行う（今のまま）。ホーム（[53](53-ui-home.md)）の在庫切れ・在庫少の件数も写しの値で数える（今のまま。要再確認は棚卸しの画面の一覧で扱う）。

> **商品と在庫の画面への統合（D-138。proposed・未実装）**: 本画面と在庫照会（[58](58-ui-stock-inquiry.md) UI-06a）を 1 つの画面「商品と在庫」（route `/products`）にする（owner の決定、repo 外の回答台帳 TD-424・TD-425）。画面・一覧・URL・道具・列・行の動き・0 件と失敗の契約は本書の UI-01a-D1・UI-01a-D14〜D19 と §50.4・§50.6・§50.7、在庫の状態（在庫切れ・在庫少・在庫あり）・在庫切れ・在庫少の一覧・行の詳細の中身は 58。今の build は 2 画面のままで、§50.3 の file と今の一覧の操作列（行の「修正」）・PLU の列は統合の前の形（後続の runtime の lane〈[backlog](../backlog.md) の「商品と在庫の画面の統合の runtime」〉が置き換える）。

## 50.1 位置付け

商品と在庫の画面は、商品を探し、在庫数と状態を確かめ、1 商品を直す・入庫する・在庫の増減を見るへ進む入口である（D-138）。商品名 / 商品コード / JAN コード / メーカー品番検索、状態（すべて / 在庫切れ・在庫少）、部門絞込み、廃番状態の切替、並替え、ページングを扱い、行の下に開く詳細から商品修正・在庫変動履歴・入庫記録へ、画面の上から商品登録へ進む（SPEC-PRV-D2）。発注の準備（在庫切れ・在庫少の商品を取引先ごとに見る）も同じ画面の状態「在庫切れ・在庫少」で行う。

この設計は UI 実装前の Design Phase 成果物であり、バックエンド関数 / CMD / DTO / DB 契約は既存の読取りの 3 つ（`search_products`・`list_low_stock`・`get_stock_detail`、UI-01a-D3）をそのまま使う。実装計画やテスト設計は Plan Packet / Test Design Matrix に置き、ここには実装者が迷わないための durable な設計判断を残す。

## 50.2 Design Intent Trace

| Spec / requirement ID | Decision ID | 設計判断 | 理由 / 捨てた案 |
|---|---|---|---|
| REQ-103 / REQ-301 / UI-01a | UI-01a-D1 | 初めて開いたら、状態「すべて」で廃番以外の全商品を商品コードの昇順に一覧表示する。`page=1`, `per_page=100`, `is_discontinued=false` を既定値にする。検索語を入れなくても一覧が出る（在庫照会の検索駆動表示〈旧 58 契約 I〉は置き換えた。D-138、owner の決定 TD-425）。 | 何を入れれば何が出るかを覚えていなくても、開けば商品が見える（Nielsen #6 記憶より認識）。在庫照会を開いて何も出ない形が owner の所感（TD-413）の起点で、店主は PC で在庫を見るとき全部の商品の一覧が助かると答えた（Q-101・TD-422）。捨てた案: 在庫切れ・在庫少を初めに出す（店主の答えと違う。状態のチップか「在庫切れ・在庫少」の入口で 1 回で開ける）、案内だけ（旧契約 I。開いて何も出ない）。（`per_page` 既定は UI 一覧の背骨 D Lane 2 で 50 → 100 へ変更、§50.4 参照） |
| REQ-103 / UI-01a | UI-01a-D2 | 検索条件、廃番モード、並替え、ページングは TanStack Router search params に持つ。 | F5 耐性、URL による再現、queryKey の安定、レビュー時の状態共有を優先する。ローカル `useState` のみは採用しない。 |
| REQ-103 / REQ-301 / REQ-302 / UI-01a | UI-01a-D3 | Tauri CMD は既存の読取りの 3 つだけを使う: 一覧は `commands.searchProducts(query)`、状態の欄と「在庫切れ・在庫少」の一覧は `commands.listLowStock(includeDiscontinued)`（[58](58-ui-stock-inquiry.md) 契約 H）、行の詳細は `commands.getStockDetail(productCode)`（58 §58.7）。部門の候補は `commands.listDepartments()`（UI-01a-D7）。wire（command・DTO・bindings）は変えない（D-138）。 | 既存 `ProductSearchQuery` が keyword / department / discontinued / sort / paging を持ち、在庫少の集合と 1 商品の詳細も既存の command が返すため、新規 CMD や BIZ 追加は不要。 |
| REQ-103 / UI-01a | UI-01a-D4 | UI-01a はページング UI を実装する。`perPage` は 50 / 100 / 200 の選択式にし、200 超を UI から送らない。 | 商品マスタは 4000 件規模がありうる。既存 IO 契約は 200 超クランプだが、UI は契約内の選択肢に制限する。 |
| REQ-103 / UI-01a | UI-01a-D5 | バーコードスキャナは HID キーボード入力として検索欄に入る前提にする。専用スキャンボタン / 連続スキャン UX は実装しない。 | UI_TECH_STACK §5.3 と UI-06a の方針を継承する。未設計の HW 連携を実装済みに見せない。 |
| REQ-103 / SP-103-08 | UI-01a-D6 | 生地は在庫数を単位付きで表示する。cm / m 表示切替はこの画面の初回実装では必須にせず、商品登録・修正や横断表示設定と合わせて再評価する。 | 現行 backend DTO は在庫数量と単位情報を返せる。cm / m 切替 UI は初回実装の範囲から外し、REQ を trace したうえで誤った局所設定を作らない。 |
| REQ-103 / UI-01a | UI-01a-D7 | 部門フィルタ候補は `list_departments` CMD で departments 全件を取得する。 | `search_products` の現在ページから候補を派生すると、検索条件・ページング・廃番状態で候補が欠ける。既存 IO `product_repo::list_departments` を BIZ/CMD 経由で公開する設計を採用する。 |
| REQ-103 / UI-01a | UI-01a-D8 | 廃番は専用の列を持たず、廃番商品のみ商品名セル内に `廃番` text badge を出し、行を `text-muted-foreground` にする。「表示中」badge は出さない。在庫の「状態」列（UI-01a-D16、[58](58-ui-stock-inquiry.md) 契約 H）は在庫の状態だけを出し、廃番を入れない。 | 表示中が大多数の一覧で全行に状態 badge を出すと密度が上がり、注目すべき廃番が埋もれる。色だけで符号化しない（[design-system/00-foundations.md §業務ステータスの視認性](../design-system/00-foundations.md)）ため text badge を併用する。部門列は維持し、L3 で密度過多なら次候補とする。 |
| REQ-103 / UI-01a | UI-01a-D9 | 検索欄を在庫照会と同型の live 型へ統一する（`debounceMs=200` / `type="search"` + native clear / 可視 Label「商品を検索」あり / 検索ボタン・`aria-label` なし）。 | Why = commit 型維持の業務理由が owner 調査で確認できず、共通化時の既存挙動温存だった。live 型でも HID スキャナの Enter 即時確定・IME 変換確定 Enter の誤発火防止・条件変更時の `page` reset は維持される。Rejected = commit 型の維持（画面間の操作一貫性を損なう理由が説明できない）（owner L3 判断 2026-08-03、gated amendment）。 |
| REQ-907 / D-117 | UI-01a-D10〜D12（撤去） | PLU の移行状態の列「PLU」（`対象外` / `未反映` / `反映済み`）、URL の `plu` 絞り込み、絞り込みに一致する全件の「PLU 対象にする / 対象から外す」と、その invalidation（D-052 C19）を持たない。どの商品をレジで売るかは商品フォームの「レジにバーコード登録する」と商品 CSV の `PLU対象` 列で決め、未反映はホームの通知バーと PLU 書出し画面が示す。 | 段階導入の道具を外す（D-117 (2)、owner 決定 TD-274）。本番の DB は無く、商品ごとの欄と CSV の列で対象を決められる。道具を作ったときの指摘はどれも道具の欠陥だった。runtime の lane が code・route の search・test を外す。 |
| REQ-103 / SP-103-04 / SPEC-PRV-D10 | UI-01a-D13 | 原価列を基本列に追加し、売価の右隣に置く（owner 裁定 2026-08-22）。 | 聞き取り第 3 陣 Q4 により PC 画面は通常客から見えず、店主の棚卸し・税理士対応・値付けは原価中心と確認できた。列密度より日常の参照性を優先する。 |
| REQ-103 / REQ-301 / REQ-302 / UI-01a / UI-06a | UI-01a-D14 | 商品検索・一覧と在庫照会を 1 つの画面「商品と在庫」にする。route は `/products`（今の UI-01a の route）。子の route（`/products/new`・`/products/$code/edit`・`/products/import`・`/products/plu-export`・`/products/price-revision`・`/stock/$code/movements`）の path は変えない。入口はサイドバーの「商品と在庫」（毎日の業務のエリア、今の在庫照会の位置）と「在庫切れ・在庫少」（入出庫のエリア、今の在庫少一覧の位置。`/products?status=low_stock` への deep-link、[52](52-ui-shared-layout.md) UI-12-D1）、ホームの入口 card の「商品と在庫」と「在庫切れ・在庫少」（[53](53-ui-home.md)）。商品管理のエリアから「商品検索・一覧」の項目を外す。画面名・ページの題・窓の題は「商品と在庫」。 | 2 つの一覧が同じ `search_products` を同じ検索語・部門で呼び、廃番の除外と商品コード順の並びも既定で同じで、同じ問いに答える一覧を 2 つ持つ理由が無い（packet 段 1 の O-1。GOV.UK *Do less*）。どちらの画面かと迷わせない（[SCREEN_DESIGN](../SCREEN_DESIGN.md) §4 の「商品と在庫の統合」）。route を `/products` にするのは、URL の道具（並べ替え・廃番の表示・表示件数）が多い側で、商品修正の `returnTo` の受け口（[51](51-ui-product-form.md) UI-01b-D2）と子の route 5 つが既に `/products` にあるから（触る link と受け口が少ない）。毎日の業務のエリアに置くのは使う頻度（在庫は毎日、商品の一覧は週に数回。SCREEN_DESIGN §1）。捨てた案: 探す画面と発注の準備の画面に分ける（案 B。店主の答え Q-101 が全部の商品の一覧で、推奨が上がる条件に当たらない）、今の 2 画面のまま何も出ない所と戻り先だけ直す（案 C。O-1 の重なりが残る）、route を `/stock` にする（商品修正の受け口と子の route の link を全部直す）、子の route を新しい親の下へ移す（デスクトップのアプリで URL は利用者に見えず、移す利点が無い）。失うもの: 商品を直すのが 1 click から 2 click（行を開く → 商品修正）、1 画面の道具が 6 つ（D-138）。 |
| REQ-103 / REQ-302 / UI-01a | UI-01a-D15 | URL state に `status`（`all` / `low_stock`）と `selected`（開いている行の商品コード）を足す（§50.4）。状態「在庫切れ・在庫少」（`status=low_stock`）では、検索の道具は検索欄・状態・部門だけを出し、廃番の表示・並べ替え・表示件数・pager は出さない（画面の上の「商品登録」の button は状態に依らず出す。§50.6）。廃番の表示・並べ替え・表示件数の URL の値は消さず、「すべて」へ戻すと元に戻る。`page` は状態の変更で §50.4 の条件の変更の規則に従い 1 に戻る。並べ順は一覧の上に「取引先順」と文字で出す。廃番は常に除く（[58](58-ui-stock-inquiry.md) の廃番除外）。 | 発注の準備は今の在庫切れ・在庫少の一覧が持つ検索語・部門の絞り込みより道具を足さない（repo 外の回答台帳 QU-004）。効かない道具を見せない（押して何も変わらない道具を置かない）。並べ順を見せる（原田本 Q5）。値を消さないので行き来しても条件を失わない（DSR-18 の戻す先の考え方）。捨てた案: 効かない道具を disabled で残す（6 つの道具のうち 3 つが押せないまま並ぶ）、「在庫切れ・在庫少」でも並べ替えとページを持つ（UI-06a-D1 の Revisit の条件〈100 件を恒常的に超える〉に当たらない）。 |
| REQ-103 / REQ-302 / UI-01a | UI-01a-D16 | 一覧の列を状態で分ける。「すべて」: 商品コード・商品名（廃番の badge と JAN を同じ欄）・部門・取引先・状態・在庫数・売価・原価の 8 列。「在庫切れ・在庫少」: 取引先・メーカー品番・商品コード・商品名・状態・在庫数・売価の 7 列で、取引先名 → 在庫数 → 商品名の順（[58](58-ui-stock-inquiry.md) UI-06a-D4）。どちらも操作の列を持たない（UI-01a-D17）。PLU の列は持たない（UI-01a-D10〜D12）。 | 発注書は取引先ごとに書き、発注書に書く番号はメーカー品番（台帳 L-130・L-174・TD-362）なので、発注の準備の一覧は取引先を先頭に、メーカー品番を出す（チャンク化）。在庫の状態 → 在庫数 → 売価の順（[03](../design-system/03-philosophy.md) の「配置の常識」）。原価は売価の右隣（UI-01a-D13）。列は 16px のまま横に収まるかを runtime の Windows native L3 で確かめる（[04](../design-system/04-backbone.md) 原則 11）。捨てた案: 1 つの列の組で両方を出す（今の在庫照会。発注の準備にメーカー品番が無い）。 |
| REQ-103 / REQ-301 / REQ-303 / UI-01a | UI-01a-D17 | 行のクリックで、その行の直下に詳細を開く（[02](../design-system/02-component-catalog.md) ⑫、中身は [58](58-ui-stock-inquiry.md) §58.7 の `StockDetailContent`）。開いている行をもう一度押すと閉じる（UI-06a-D5）。詳細から「商品修正」（`/products/$code/edit`、`returnTo` は今の search state を `buildProductListReturnTo` で組んだ値。[51](51-ui-product-form.md) UI-01b-D2）・「在庫変動履歴」（`/stock/$code/movements`、`returnTo` は今の URL〈`location.href`〉、UI-06a-D7）・「入庫記録」（`/inventory/receiving?productCode=<商品コード>`、その商品を明細に 1 行入れた状態で開く、[61](61-ui-receiving.md) UI-02-D18）へ進む。画面の上に「商品登録」（`/products/new`）を置く。 | 旧 §50.6 の「行クリックは商品修正へ遷移する」（code は操作の列の「修正」の button で、正本と食い違っていた。packet 段 1 の X-04）を置き換える。1 商品の在庫・最終入庫日・最終販売日は行を開いて見る（IBM Carbon の Progressive disclosure）。直した後・増減を見た後は来た一覧の条件へ戻る（DSR-18）。届いた商品をそのまま入庫へ進める（Fluent 2 *Relevant*）。捨てた案: 行に「修正」の button を残す（1 画面に行を開く動きと遷移する動きが並び、操作の列が増える）。失うもの: 商品を直すのが 2 click になる（D-138）。 |
| REQ-103 / REQ-301 / UI-01a | UI-01a-D18 | 何も出ない画面を作らない。初めて開いたときと条件を変えたときは、一覧か、一覧を出せない理由と次の一手を必ず出す: 読込みの失敗（§50.7 の Error）、絞り込みの 0 件、範囲外の page（[58](58-ui-stock-inquiry.md) UI-06a-D3 と同じ回復）、商品が 1 件も無い DB（§50.7 の Empty）。0 件の説明は選んでいる条件に合わせ、検索語を入れていないのに検索語を変えるよう促さない（[02](../design-system/02-component-catalog.md) ⑥ の空状態の説明、DSR-11）。 | 開いて何も出ないことが owner の所感の起点で、何を出していないかと次の一手を見せる（Nielsen #1）。今の在庫少の 0 件は検索語を入れていなくても「検索語を変えて」と言う（packet 段 1 の E-03）。 |
| REQ-103 / REQ-301 / UI-01a | UI-01a-D19 | `/stock`（今の在庫照会の route）は `/products` へ寄せる（route の redirect）。`q`・`dept`・`selected`・`page` を写し、`page` を写すときは `perPage=50`（統合の前の在庫照会の表示件数の既定）を足す。`status` は `stockout` と `low_stock` を `low_stock` に、それ以外を捨てる。在庫変動履歴の route `/stock/$code/movements` は寄せない。限定: 統合の前の在庫照会で利用者が選んだ表示件数 100・200 は URL に無いローカルの値なので再現できず、その `page` は別の行を指す（範囲外 page の回復〈UI-01a-D18〉か、`selected` が外れる〈[58](58-ui-stock-inquiry.md) §58.4〉。一覧と条件は残る）。 | 統合の前に作られた link・`returnTo`・F5 の URL が着地を失わない。`/products` の既定の表示件数は 100 なので、`page` だけを写すと同じ `page` が別の行を指す（`perPage=50` を足すと同じ行に着く）。`page` を写さない案は採らない（2 ページ目以降の `selected` も外れる）。今の在庫切れの一覧は「在庫切れ・在庫少」の部分集合なので `low_stock` へ寄せる（切れか少かは状態の列で見分ける）。 |

## 50.3 画面構成

実装時の想定ファイル:

```text
src/routes/products/index.tsx
src/features/products/ProductListPage.tsx
src/components/patterns/SearchBar.tsx        # 旧 ProductSearchBar、PR-B で統合（live 型 debounceMs=200、2026-08-03 owner L3 是正で commit 型から変更）
src/components/patterns/DepartmentFilter.tsx # 旧 features/products/components/DepartmentFilter、PR-B で統合
src/features/products/components/ProductTable.tsx
src/components/patterns/Pagination.tsx                  # 旧 features/products/components/ProductPagination、UI 一覧の背骨 D Lane 2 で統合
```

`src/routes/products/index.tsx` は route / search params / title を管理し、画面本体は `ProductListPage` に委譲する。`ProductListPage` は CMD 呼び出し、派生表示、子コンポーネントへの props 配線を担当する。業務判定や DB 操作は UI に置かない。

## 50.4 URL State

| URL param | 型 / 値 | 既定値 | CMD への変換 |
|---|---|---|---|
| `q` | string | `""` | trim 後に空なら `keyword = null`、非空なら `keyword = q` |
| `dept` | number or absent | absent | absent なら `department_id = null` |
| `discontinued` | `active` / `all` / `discontinued` | `active` | `active -> false`, `all -> null`, `discontinued -> true` |
| `sort` | `product_code` / `name` / `stock_quantity` / `selling_price` | `product_code` | `product_code -> ProductCode`, `name -> Name`, `stock_quantity -> StockQuantity`, `selling_price -> SellingPrice` |
| `dir` | `asc` / `desc` | `asc` | `asc -> Asc`, `desc -> Desc` |
| `page` | number >= 1 | `1` | `page` |
| `perPage` | `50` / `100` / `200` | `100` | `per_page` |
| `status` | `all` / `low_stock` | `all` | `all` は `searchProducts`、`low_stock` は `listLowStock` の結果を画面で絞る（[58](58-ui-stock-inquiry.md) §58.6 `filterAndSortLowStockList`）。旧値 `stockout` は `low_stock` として読む（UI-01a-D19） |
| `selected` | 商品コードの string（1〜100 文字） | absent | 行の詳細（`getStockDetail`）。数字だけの商品コードも string のまま往復させる（[66](66-ui-stock-movements.md) UI-06c-D9 の GA5 と同じ） |

検索語、部門、廃番モード、状態、並替え、`perPage` が変わったときは `page=1` に戻し、`selected` を外す（[58](58-ui-stock-inquiry.md) §58.4 の `selected` ライフサイクル）。ページ移動だけは現在の検索条件を維持する。`status=low_stock` では `discontinued`・`sort`・`dir`・`perPage` を使わないが URL から消さない（UI-01a-D15。`page` は状態の変更で 1 に戻る）。

**2026-09-03 更新履歴（UI 一覧の背骨 D Lane 2）**: `perPage` の既定値を `50` → `100` へ変更（`src/features/products/search.ts` `normalizePerPage`、1 件探索が主動線のため。URL 明示値が優先される挙動・共有定数 `LIST_PER_PAGE_OPTIONS` は不変）。

`q` は live 型（[59-ui-shared-patterns.md](59-ui-shared-patterns.md) §59.1 SearchBar、`debounceMs=200`）で、入力から 200ms 後に search param へ反映する。Enter は debounce を待たず即時反映し（trim なし）、IME 変換確定中の Enter は `event.isComposing` で除外して検索確定と取り違えない。CMD 呼び出し直前の `keyword` 変換でのみ trim する（UI-01a-D9、2026-08-03 owner L3 判断、gated amendment）。

**controlled value の責務分離（UI-01a-D9 追補、amendment Plan Review round 1 P1-1）**: SearchBar の controlled `value` には raw の `search.q ?? ""` を渡す。`normalizedSearch.q`（`normalizeString` = trim + 空→undefined）は CMD query の `keyword` 変換・filter 既定判定・`returnTo` の導出専用とし、入力欄の表示値には使わない。normalized 値を controlled value に結線すると、live 反映のたびに trim 済み値が入力欄へ書き戻され「trim なし」契約が main path で破れる。

## 50.5 CMD / DTO 契約

一覧は `commands.searchProducts(query: ProductSearchQuery)` を呼ぶ。状態の欄・「在庫切れ・在庫少」の一覧の `commands.listLowStock` と行の詳細の `commands.getStockDetail` の型は [58](58-ui-stock-inquiry.md) §58.2（UI-01a-D3）。

`ProductSearchQuery`（SPEC-PRV-D2 により keyword は `maker_code` を含む）:

```text
keyword: string | null
department_id: number | null
is_discontinued: boolean | null
sort_key: ProductSortKey
sort_order: SortOrder
page: number
per_page: number  // 上限 200。UI は 50 / 100 / 200 のみ送信し、200 超は IO 層でクランプされる
```

戻り値は `PaginatedResult<ProductWithRelations>` とし、`items` と `total_count` をページング UI の正とする。UI は `perPage` を最大 200 に制限するが、IO 層の 200 超クランプ契約は互換性のため維持される。

部門フィルタ候補は `commands.listDepartments()` で取得する。実装 PR では [30-biz-product-service.md §4.7](30-biz-product-service.md#47-list_departmentsbiz) と [40-cmd-product.md §5.4](40-cmd-product.md#54-各コマンドの関数仕様) に従って thin BIZ/CMD を追加し、generated binding を更新してから UI を配線する。

## 50.6 表示と操作

- 検索欄は商品名 / 商品コード / JAN コード / メーカー品番（`maker_code`）を同じ入力で扱う（SPEC-PRV-D2）。placeholder は「商品コード・商品名・JAN・メーカー品番で検索」とする。live 型（[59-ui-shared-patterns.md](59-ui-shared-patterns.md) §59.1、`debounceMs=200`）で入力から 200ms 後に search params を更新し、Enter は debounce を待たず即時反映する（IME 変換確定中の Enter は無視）。`type="search"` のネイティブ clear を使い、検索ボタンは持たない。可視 `Label htmlFor`「商品を検索」のみを accessible name とし、`aria-label` は持たない（UI-01a-D9、[component catalog ⑨](../design-system/02-component-catalog.md)）。
- 部門フィルタは `commands.listDepartments()` 由来の全 21 部門から選ぶ。検索結果の現在ページから候補を作らない。
- 廃番モードは `表示中` / `すべて` / `廃番のみ` の意味が日本語で分かる segmented control にする。
- 状態のチップは「すべて」「在庫切れ・在庫少」の 2 つ（[58](58-ui-stock-inquiry.md) UI-06a-D8）。
- テーブル列は状態で分ける（UI-01a-D16）。「すべて」は 商品コード、商品名、部門、取引先、状態、在庫数、売価、原価。原価は売価の右隣に置く（SPEC-PRV-D10）。廃番状態は専用列を持たず、商品名セル内に表す（UI-01a-D8）。状態の列の意味は 58 の契約 H。
- 商品コードと商品名は並べて見せ、商品コード単独で利用者判断を強制しない。
- 廃番状態は色だけで表さず、廃番商品のみ商品名セル内に `廃番` text badge を出し、行を muted 表示にする。「表示中」badge は出さない（UI-01a-D8）。
- 行クリックはその行の直下に詳細を開き、詳細の「商品修正」が商品修正へ遷移する（UI-01a-D17）。新規登録ボタン「商品登録」は商品登録へ遷移する。

## 50.7 Loading / Empty / Error

- Loading: 検索条件エリアは残し、一覧領域で loading を示す。
- Empty: 条件に一致する商品がない場合は、検索条件を維持したまま空状態を表示する。絞り込み（`q` / `dept` / `discontinued` / `status`）が既定値以外かつ 0 件のときは、既存の「商品を登録する」action を常設のまま維持し、`action` slot 内に「絞り込みを解除」ボタンを横並び併置する。押下で `q` / `dept` / `discontinued` / `status` と `page` を既定値へ戻す。`sort` / `dir` / `perPage` は変更しない。見出しと説明は条件に合わせ、次の順で最初に当たるものを出す（UI-01a-D18）: (1) 検索語があれば「該当する商品がありません」と「検索の語を変えるか、絞り込みを解除してください」。(2) 状態「在庫切れ・在庫少」で検索語・部門が無ければ「在庫切れ・在庫少の商品はありません」（説明なし。この状態は廃番の表示を使わないので、URL の `discontinued` の値を判定に入れない）。(3) それ以外（検索語が無く、部門か廃番の表示だけ）は「該当する商品がありません」と「絞り込みを解除してください」。絞り込みが既定値のまま 0 件（商品が 1 件も無い DB）のときは「商品がありません」と「商品を登録するか、一括インポートで取り込んでください」を出し、reset ボタンを出さない。
- Error: CMD 呼び出し失敗時は一覧領域にエラーを表示し、検索条件を編集できる状態を維持する。DB / CMD の失敗を UI 側で業務成功扱いにしない。状態「すべて」で在庫少の集合（`listLowStock`）だけが読めないときの状態の列は 58 の契約 H。
- Recovery: 条件変更または再試行で同じ `searchProducts` を再実行できるようにする。

## 50.8 テスト観点

UI-01a 実装時は、以下を trace ID 付きで検証する。

- UI-01a-D1: 引数なしで開くと `is_discontinued=false`, `page=1`, `per_page=100` の検索が走り、一覧が出る（URL 明示値がある場合はそちらを優先、UI 一覧の背骨 D Lane 2 で既定 50 → 100 へ変更）。
- UI-01a-D14〜D19: 統合の後の契約の test は、後続の runtime の lane（[backlog](../backlog.md) の「商品と在庫の画面の統合の runtime」）が足す。
- UI-01a-D2: URL search params の既定値、無効値補正、F5 相当の復元が効く。
- UI-01a-D3: `commands.searchProducts` に渡る payload が URL state と一致する。CMD は読取りの 3 つ（と部門の候補）だけを呼ぶ。
- UI-01a-D4: page / perPage の変更、検索条件変更時の page reset、total_count からの最終ページ計算。
- UI-01a-D5: 検索欄 Enter で検索できる。専用スキャンボタンを前提にしない。
- UI-01a-D6: 生地在庫の単位付き表示を壊さず、cm / m 切替を実装済みとして見せない。
- UI-01a-D7: 部門候補は `listDepartments` 由来で全件表示され、現在ページの検索結果から欠落しない。
- UI-01a-D8: 廃番の専用の列がなく（「状態」列は在庫の状態だけ、UI-01a-D16）、廃番商品のみ商品名セル内に `廃番` text badge が出て行が muted になる。「表示中」badge は出ない。
- UI-01a-D9: 検索欄が live 型（`debounceMs=200`）で動作し、Enter は debounce を待たず即時反映、IME 変換確定中の Enter は発火しない。`q` の変更・クリアで `page` が既定へ戻る。可視 Label「商品を検索」が input と結線され、検索ボタン・`aria-label` は持たない。
- UI-01a-D10〜D12（撤去）: 一覧に「PLU」列・`plu` の絞り込み・一括の PLU 対象化の操作が無い（URL の `plu` は無視される）。
- SPEC-PRV-D10: 「原価」列ヘッダが「売価」の右隣に存在し、値が基本列として表示される。

## 50.9 Deferred

- UI-01b 商品登録・修正 route の最終ファイル構成と form 設計。
- SP-103-08 の cm / m 表示切替 UI。初回 UI-01a 実装で扱う場合は、横断表示方針または商品管理内の設計判断を追加してから実装する。
- 専用バーコードスキャン UX / 連続スキャン検知。
