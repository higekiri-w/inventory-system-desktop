> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（UI-06c / REQ-303）、SCREEN_DESIGN.md（商品と在庫画面 / 入出庫履歴・在庫変動追跡）、44-cmd-inventory.md（CMD-06 list_movements）、58-ui-stock-inquiry.md（在庫照会からの導線）、65-inventory-record-traceability.md（完成形 traceability contract）

## 66. UI-06c: 商品別在庫変動履歴

> **計数の置き場と在庫の求め直し（D-118、b2 で実装済み）**: 計数は movement を作らないので、履歴に計数を「数を確認（{数}、{日付}）」・「数を入れ直し（{数}、{日付}）」の行として movement と並びの鍵の順に混ぜて出す（[65](65-inventory-record-traceability.md) の同名の契約。結果の型・並び・絞り込みは [44](44-cmd-inventory.md) の `list_movements`）。`stock_after` は保存した時点の写しの値で、後入力で前に記録が入っても書き換えない。見出しの現在庫は写しで、要再確認なら範囲と「要再確認」の badge と案内を出す。表と見出しの規則は §66.4・§66.5（2026-10-23、b2 の起票で決めた）。

### 66.1 概要

- **対応 REQ**: REQ-303（在庫変動履歴）+ REQ-207（在庫変動から元業務記録へ戻る）
- **route**: `/stock/$code/movements`
- **呼び出す CMD**:
  - `getStockDetail(productCode)` — 見出し、現在庫、商品名、部門表示
  - `listMovements(query: MovementQuery)` — 商品別の在庫変動の履歴（movement と計数の行）
- **主動線**:
  1. 商品と在庫の画面（`/products`、D-138。今の build は `/stock`）の行の詳細から「在庫変動履歴」を開く
  2. 商品別に、movement と計数の行を並びの鍵の新しい順（業務日の順）に確認する
  3. movement の行の元記録ラベルから業務記録詳細へ、計数の行の元記録ラベルから棚卸しの記録の詳細へ遷移する
  4. 「商品と在庫へ戻る」で `returnTo`（遷移元の URL）へ戻る。欠落・不正時は `/products?q=$code&selected=$code&discontinued=all`（UI-06c-D9。D-138 の後の値。今の build は label「在庫照会へ戻る」・先 `/stock`）
- **初回実装の非対象**: CSV出力、印刷、取消/訂正、元 movement 関係表示、横断 `/inventory/records`、業務記録詳細画面そのもの。

**関数要求**: 商品コードごとの在庫変動履歴を URL search state 付きで表示し、movement と計数の各行の業務日と保存の日時、種別、増減、変動後在庫、元業務記録リンク、備考を operator-facing な日本語 UI として確認できるようにする。

**シグネチャ**:

```ts
export function StockMovementsPage(props: {
  productCode: string;
  search: StockMovementsSearch;
  onSearchChange: (updater: (prev: StockMovementsSearch) => StockMovementsSearch) => void;
}): JSX.Element;

export function useStockMovements(args: {
  productCode: string;
  search: NormalizedStockMovementsSearch;
}): {
  productQuery: UseQueryResult<StockDetail>;
  movementsQuery: UseQueryResult<PaginatedResult<StockHistoryItem>>;
};
```

**処理ステップ**:

1. Route が path param `code` と search params `dateFrom/dateTo/type/page/returnTo` を zod で検証し、不正値を fallback する。
2. `StockMovementsPage` が search params を `NormalizedStockMovementsSearch` に正規化する。
3. `useStockMovements` が `getStockDetail(code)` と `listMovements(MovementQuery)` を独立 query として呼ぶ。
4. 商品 query 成功時は商品名、商品コード、部門、現在庫（要再確認なら範囲と badge、その下に要再確認の案内）を表示する。失敗時は商品情報だけ inline warning にする。
5. 履歴 query 成功時は `StockHistoryTable` に渡し、「日付」（業務日・Z004 の精算回数・保存の日時）、種別、増減、変動後在庫、元記録、備考を BIZ の順のまま表示する。
6. filter 変更時は `onSearchChange` で該当 search param を更新し、`page` を 1 に戻す。
7. pagination 操作時は `page` だけを更新する。

**エラーハンドリング**:

- `getStockDetail` 失敗時は商品サマリ欄に「商品情報の取得に失敗しました」を表示し、movement list は表示継続する。
- `listMovements` 失敗時は destructive Alert「在庫変動履歴の取得に失敗しました」を表示する。
- movement の `source` と計数の `count.source` が null の場合は行を落とさず、「元記録なし」を表示する。
- 未知の `movement_type` はエラーにせず、元文字列を outline Badge で表示する。

### 66.2 Design Decisions

| ID | 決定 | 理由 / 棄却案 |
|---|---|---|
| UI-06c-D1 | route は `/stock/$code/movements` とし、商品コードを path param に置く。 | 商品別台帳であり、F5 / bookmark / 共有時に対象商品が一意になる。query だけで `product_code` を持つ案は在庫照会本体の検索 state と衝突しやすいため棄却。 |
| UI-06c-D2 | 検索条件の search params は `dateFrom` / `dateTo` / `type` / `page` とし、戻り先の `returnTo` を別途保持する（UI-06c-D9）。 | perPage は既定 50 + `Select`（50 / 100 / 200、owner 直回答 E10、2026-09-05）。perPage はローカル state（L3-D3）。`returnTo` は検索 query に渡さない。 |
| UI-06c-D3 | product header と movement list は 2 useQuery とし、部分障害を許容する。 | movement が失敗しても商品名・現在庫を表示して対象商品を確認できる。商品詳細が失敗しても movement は商品コード単位で取得できる。 |
| UI-06c-D4 | movement 種別は frontend で日本語ラベルへ変換し、未知種別は元文字列を表示する。 | backend contract は string。表示不能にせず調査可能性を優先する。未知値で落とす案は legacy/corrupt row の追跡を妨げるため棄却。 |
| UI-06c-D5 | 増減数量は `+N` / `-N` と日本語の「増加」「減少」ラベルで示し、色だけに頼らない。長さの商品は `+2.5 m` の形で出し、表は `product_code → stock_unit` の対応を受ける（個数の商品と対応に無い行は語を付けない整数。[共通規則](10-common-rules.md) SPEC-UNIT-D3 の「表示する所」の表の 6 行目）。 | DSR-08。業務上の意味を非IT利用者が判別できる必要がある。 |
| UI-06c-D6 | movement の `source`・計数の `count.source` がある行だけ「元記録」リンクを出し、ない行は「元記録なし」と表示する。 | PR #112 の source-link contract を使う。初期在庫や legacy 行、棚卸しの明細を参照しない計数は行自体を表示し、リンク欠落だけを明示する。 |
| UI-06c-D7 | 元記録 route がまだ未実装でも link URL は `source.route` をそのまま表示対象にする。 | UI-06c の責務は movement から元記録へ戻るための contract 表示。未実装詳細 route の完成は後続スライスで扱う。 |
| UI-06c-D8 | Windows native L3 は必要。 | 新規 operator-facing 調査画面であり、表の密度、数量符号、元記録リンク、戻り導線の視認性を確認する必要がある。 |
| UI-06c-D9 | D-138 の後（proposed・未実装）は、pin する pathname と fallback の先を `/products` に、label を「商品と在庫へ戻る」にする（`/stock` は `/products` へ寄せる、[50](50-ui-product-list.md) UI-01a-D19。統合の前に作られた `/stock` の `returnTo` は pin 不一致で fallback に落ち、`q`・`selected` で同じ商品に着く）。fallback の search は `{ q: code, selected: code, discontinued: "all" }`（廃番の商品の履歴から戻っても、既定の廃番の表示〈表示中〉で行が消えない。見出しの query の成否に頼らず常に付ける）。限定: `q` は部分一致なので、その商品コードを含む商品が表示件数を超えて該当の商品が 1 ページ目に無いと `selected` は外れる（[58](58-ui-stock-inquiry.md) §58.4。一覧は `q` で絞られたまま残り、pager で届く）。限定: `returnTo` の 500 字の上限は受け口ごとに着地が違う（どちらも今の挙動のまま）。在庫変動履歴の `returnTo`（§66.3、`src/features/stock-movements/types.ts:41` の `.max(500)`）が長い検索語で超えると schema が `undefined` にし、既定の戻り先（上の fallback）に落ちる。来た一覧の条件は失うが同じ商品に着く。業務記録詳細の 6 つの route の `returnTo`（`src/routes/inventory/receiving.records.$recordId.tsx:11` ほか、同じ上限）に渡す入れ子の `detailReturnTo` が超えると、その詳細の「前の画面へ戻る」は詳細の既定 `/inventory/records`（`src/features/inventory-records/ReceivingRecordDetailPage.tsx:45` ほか 5 つの `*RecordDetailPage.tsx`）に着き、在庫変動履歴と商品は失う。上限は D-138 の前からあり、統合の画面の URL は key（[50](50-ui-product-list.md) §50.4）が増えるので少し早く届く。上限を上げる案は [backlog](../backlog.md) の「商品と在庫の画面の統合の runtime」の候補（入れ子の長さの設計が要る）。仕組み（`returnToLinkProps`・`null` の分岐・`Link` の `search` props・`detailReturnTo`）は変えない。以下の `/stock` と「在庫照会へ戻る」は今の build の値。「在庫照会へ戻る」は `returnToLinkProps(search.returnTo, { pathname: "/stock" })`（`src/lib/return-to.ts`、DSR-18）で `/stock` へ pin する。pin が通れば `<Link {...backLinkProps}>`（`to` = pathname、`search` = router の search object）、欠落・不正・pin 不一致で helper が `null` を返したら `<Link to="/stock" search={{ q: code, selected: code }}>` で在庫照会へ遷移する（`null` の分岐は型で強制される）。文字列 URL では組まない（数字だけの商品コードが `parseSearch` の `JSON.parse` で number 化し、route の `z.string().catch(undefined)` で落ちるため、GA5）。label は「在庫照会へ戻る」のまま（`returnTo` も fallback も在庫照会に着地するため、DSR-18 の「前の画面へ戻る」label は使わない）。元記録 link へ渡す `detailReturnTo` は `/stock/${encodeURIComponent(code)}/movements` に、正規化した search（`dateFrom` / `dateTo`、`all` 以外の `type`、2 以上の `page`）と `returnTo` をこの順の object にして `defaultStringifySearch`（router の既定の search 直列化）で付けて組み、`returnTo` を入れ子で載せる（2026-09-25）。`stockInquirySearchSchema.selected` の上限は `q` と同じ 100 文字（旧 20、returnTo 衛生 R3、SPEC-RETURNTO-HYGIENE-2026-09-17 C6）。`updateSearch` / `resetFilters` は `...prev` を spread 済みで追加変更なし。 | 監査 NAV-1 起源。直接アクセス・業務記録詳細からの入口では在庫照会の元条件が存在しないため、商品コード検索（LIKE、`src-tauri/src/db/product_repo.rs` の test `test_search_products_req103_keyword_product_code`）で対象商品を 1 件以上含む list を出し、`selected` で展開する。在庫変動履歴の元記録 link から業務記録詳細へ行って戻ると在庫変動履歴の URL は `detailReturnTo` で再現されるため、そこに `returnTo` が無いと「在庫照会へ戻る」が fallback へ落ちる。棄却案: fallback を `/stock`（商品を見失う、現行と同じ不便）/ fallback を `/stock?selected=<code>`（現行のまま、NAV-1 未解消）/ 文字列 `/stock?q=…&selected=…` を `to` に渡す（`parseSearch` が JSON.parse するため数字だけの商品コードが number 化し `z.string()` で落ちる、GA5）/ 入れ子を避けて sessionStorage に持つ（URL 以外の state を増やす、F5 で消える）/ `detailReturnTo` を `URLSearchParams` で手組みする（2026-09-25 まで。JSON として読める string 値を引用符なしで出すため、受け側の `parseSearch` で number 等に変わる。日付・enum・page と `/` 始まりの `returnTo`〈または `returnTo` なし〉では出力が router の直列化と 1 byte も同じだが、route schema が受ける `/` 始まりでない不正な `returnTo`〈例 `123`〉は number 等に化ける〈新旧とも不正で「在庫照会へ戻る」は既定の戻り先に着地し、router の直列化へ寄せて変わるのは詳細 link の href だけ。例外は引用符ごとの `"/stock"` のような JSON の文字列 literal で、手組みは往復で引用符を外して `/stock` へ着地させていた〉。string の param を足すと GA5 と同じ機序になる）/ `detailReturnTo` に `useRouterState` の `location.href` を使う（URL にある未検証の値・既定値の `page=1`・元の key 順をそのまま運び、正規化した形が変わる。page は props で受けた検証済み search から組む）。`selected` の上限を 20 のまま据え置く（商品コードに長さ上限が無いのに `q` より狭い、GA5 系の未解消 gap）。 |

### 66.3 URL Search State

```ts
type StockMovementsSearch = {
  dateFrom?: string; // YYYY-MM-DD
  dateTo?: string;   // YYYY-MM-DD
  type?: "all" | "receiving" | "return" | "sale_auto" | "sale_manual" | "disposal" | "stocktake";
  page?: number;     // 1 始まり。未指定は 1
  returnTo?: string; // 商品と在庫（今の build は在庫照会）からの遷移元 URL（max 500）
};
```

- 欠落・不正な `returnTo`（`/` 始まりでない、解析不能、base origin 不一致、解決後の pathname が `//` 始まり、pin する pathname と不一致、500 字超）は fallback する。500 字超は正しい一覧の URL でも起こる（長い検索語）。そのときは来た一覧の条件を失い既定の戻り先で同じ商品に着く限定として扱い、上限は変えない（UI-06c-D9）。この `returnTo` を入れ子に持って業務記録詳細へ渡す `detailReturnTo` が 500 字を超えたときは別の受け口の話で、詳細の「前の画面へ戻る」は詳細の既定 `/inventory/records` に着き、在庫変動履歴と商品は失う（UI-06c-D9）。D-138 の後は `/products` へ `search={{ q: code, selected: code, discontinued: "all" }}`（UI-06c-D9。今の build は `/stock` へ `search={{ q: code, selected: code }}`）。着地先は `/products?q=$code&selected=$code&discontinued=all` と同じ意味だが、`Link` の `search` props で組み、文字列 URL では組まない。非 string / 500 字超は schema で `undefined` とし、URL は `returnToLinkProps` で DSR-15 の origin / pathname 検証を通し、解決した pathname を exact pin する（UI-06c-D9。D-138 の後は `/products`、今の build は `/stock`）。
- filter 変更・page 送り・reset でも `returnTo` は保持する。
- 不正な `dateFrom` / `dateTo` は `undefined` に fallback する。
- 不正な `type` は `"all"` に fallback する。
- `page < 1` または数値化できない page は `1` に fallback する。
- `dateFrom` / `dateTo` / `type` を変更したら `page=1` に戻す。
- `per_page` は既定 50、`Select` で 50 / 100 / 200。CMD/BIZ の上限 200 を超える選択肢は置かない。

### 66.4 Data Flow

本 query を stale 化する mutation 集合は [D-052](../decision-log.md) と `src/lib/invalidation-contract.ts` を正本とし、本書へ producer 一覧を複製しない。`queryKeys.stockMovements.root()` は product / list key の共通 prefix とする。

```
Route params: code
Route search: dateFrom, dateTo, type, page, returnTo
  ↓
StockMovementsPage
  ├ getStockDetail(code)
  └ listMovements({
       product_code: code,
       date_from: dateFrom ?? null,
       date_to: dateTo ?? null,
       movement_type: type === "all" ? null : type,
       page,
       per_page: perPage,
     })                                   → PaginatedResult<StockHistoryItem>（BIZ の順）
  └ listRecheckProducts()（写しが要再確認のときだけ。73 と同じ useRecheckProducts）
       ↓
MovementSummary（StockWithRecheck + StockRecheckNotice）+ StockHistoryTable + Pagination
```

### 66.5 UI 表示

#### Header

- `PageHeader` title: `在庫変動履歴`
- actions:
  - `商品と在庫へ戻る` → `returnTo` が `/products` へ pin されて解決すればその `to`/`search`、欠落・不正・pin 不一致（helper が `null`）なら `/products` に `search={{ q: code, selected: code, discontinued: "all" }}`（UI-06c-D9、`returnToLinkProps` で検証。D-138、proposed・未実装。今の build は `在庫照会へ戻る` で `/stock` へ pin し、fallback は `/stock` に `search={{ q: code, selected: code }}`）

#### 商品サマリ

商品詳細 query 成功時:

- 商品名
- 商品コード
- 部門
- 現在庫（`StockWithRecheck`。写しが要再確認（`stock_recheck`）なら、範囲（棚卸しの画面の要再確認の一覧と同じ部品の形「{low}〜{high} {単位}」。単位の語は後ろに 1 回で、例「8〜9 個」「4〜4.5 m」）と、「要再確認」の状態の badge（[02](../design-system/02-component-catalog.md) の①状態。warning の tone と icon と文字で、色だけで示さない）を出す。要再確認でなければ今の `formatStockDisplay`。[50](50-ui-product-list.md)・[58](58-ui-stock-inquiry.md)・[51](51-ui-product-form.md) の在庫の欄と同じ部品）
- 要再確認なら見出しの下に、[58](58-ui-stock-inquiry.md) の詳細と同じ要再確認の案内（`StockRecheckNotice`。理由と棚卸しの画面への link）

商品詳細 query 失敗時:

- destructive Alert ではなく inline warning として「商品情報の取得に失敗しました」を表示する。
- movement list は表示を継続する。

#### Filters

種別の絞り込みの選択肢と label は変えない。「すべて」は movement と計数、「棚卸し」は棚卸しの movement と計数、ほかはその種類の movement だけ。日付の絞り込みは業務日の両端を含む（[44](44-cmd-inventory.md)）。

- 日付範囲: `dateFrom` / `dateTo` の date input
- 種別: Select
  - `all`: すべて
  - `receiving`: 入庫
  - `return`: 返品・交換
  - `sale_auto`: POS売上
  - `sale_manual`: 手動販売
  - `disposal`: 廃棄・破損
  - `stocktake`: 棚卸し

#### Table

表は履歴の行（`StockHistoryItem`）を描く在庫変動の画面だけの表（`StockHistoryTable`）。業務記録の詳細（入庫・返品交換・手動販売・廃棄・CSV 取込み）の「関連する在庫変動」の表（`MovementTable`、`MovementRecord` の一覧、日時は `created_at`）は変えず、種別の label・増減・元記録の書き方を共有する。行は BIZ の順のまま出す（画面で並べ替えない）。

列:

1. 日付
2. 種別
3. 増減
4. 変動後在庫
5. 元記録
6. 備考

表示規則:

- 「日付」は行の `business_date`（`YYYY-MM-DD`）で、Z004 の行（`settlement_no` を持つ movement）は後ろに「（精算 {n}）」、下の行に小さく「保存 {`saved_at` の YYYY-MM-DD HH:mm}」を出す（後から入れた記録が、保存の時刻でなく日付の位置に並ぶことを読めるように）。fresh な計数の `settlement_no` は並びの鍵にだけ使い、精算の番号を付けない（44）。変換できない保存の日時は元文字列を表示する。
- 種別は日本語 Badge。未知値は outline Badge で元文字列を表示する。
- 計数の行: 「種別」に 65 の文「数を確認（{数}、{日付}）」（既定の計数）・「数を入れ直し（{数}、{日付}）」（fresh な計数）を出す。{数} は `count.count_value` を `count.stock_unit` で在庫の欄と同じ `formatStockDisplay` の形にしたもの（例「8 個」「4.5 m」。単位は行が持つので、`getStockDetail` が失敗した部分障害〈UI-06c-D3〉でも長さの商品の計数 450 は「4.5 m」と出す。movement の行は UI-06c-D5 のとおり `product_code → stock_unit` の対応で描き、対応が無ければ語を付けない整数。[共通規則](10-common-rules.md) SPEC-UNIT-D3。計数は在庫の数そのものなので、movement の行の増減・変動後在庫の形〈表の 6。個数は単位の語を付けない数〉でなく、在庫の欄に合わせる）、{日付} は `business_date`。「増減」と「変動後在庫」は「—」（計数は差分で在庫を動かさず、保存の時点の写しを持たない）。「元記録」は `count.source` があれば label の link、無ければ「元記録なし」。「備考」は「—」。
- 増減は `+N` / `-N` の tabular 表示と `増加` / `減少` / `変動なし` ラベルを併記する。
- 元記録は movement の `source`（計数の行は `count.source`）があれば `label` のリンク、なければ muted text `元記録なし`。
- 備考が null/blank の場合は `—`。

### 66.6 Error / Empty / Loading

- movement loading: table 領域に Skeleton 3 行。
- movement error: destructive Alert「在庫変動履歴の取得に失敗しました」。条件変更か再試行を促す。
- movement empty: EmptyState「在庫変動履歴がありません」。商品に movement がない、または検索条件に該当しないことを説明する。**filter-empty reset action**（2026-08-03 batch B、[02-component-catalog.md](../design-system/02-component-catalog.md) ⑥）: `dateFrom` / `dateTo` / `type` のいずれかが既定値以外、かつ結果 0 件のときは EmptyState に「絞り込みを解除」ボタンを表示する。押下で `dateFrom` / `dateTo` / `type` と `page`（§66.3）をすべて既定値へ戻す。商品自体に movement がない（既定値のまま 0 件）ときは表示しない。
- product detail error: 商品サマリだけ inline warning。movement error と独立させる。

### 66.7 Tests

- REQ-303 / UI-06c-D9: `returnTo` の正規化と fallback、`detailReturnTo` への入れ子。
- UI-06c-D9（GA5）: 数字だけの商品コードでも fallback の `q` / `selected` が string で往復する（T2-num、`/products` の route schema〈`src/features/products/search.ts`〉まで通す。D-138 の後。今の build の test は `stockInquirySearchSchema` まで通す）。
- UI-06c-D9（returnTo 衛生 R3、SPEC-RETURNTO-HYGIENE-2026-09-17）: `/products` 以外の pathname（統合の前の `/stock` を含む）は pin 不一致として fallback へ（T4）。`selected` は 100 文字まで受理する（T7）。501 字の `returnTo` は fallback へ（§66.3）。D-138 の後。今の build の test は `/stock` 以外を pin 不一致とし、T7 は `src/features/stock-inquiry/types.test.ts`。
- UI-06c-D9（SPEC-NAV-RETURN-TYPES-2026-09-25）: `detailReturnTo` を受け側の route schema で parse すると元の search に戻る（JSON として読める string の `returnTo` を含む）。`/` 始まりでない不正な `returnTo` では戻る導線が新旧とも既定の戻り先に着地する。商品と在庫（数字だけの商品コード）→ 在庫変動履歴 → 業務記録詳細 → 「前の画面へ戻る」→「商品と在庫へ戻る」の実 router 往復で、各段の href が出発時と文字列一致する（D-138 の後。今の build の test は在庫照会と「在庫照会へ戻る」で同じ往復を見る）。

- REQ-303 / UI-06c-D1: route params の `code` を `listMovements.product_code` に渡す。
- REQ-303 / UI-06c-D2: search params を `MovementQuery` に変換し、filter 変更時は page を 1 に戻す。
- REQ-303 / UI-06c-D4/D5/D6: movement table が種別、増減、変動後在庫、元記録リンク、元記録なしを表示する。
- REQ-303 / UI-06c-D3: product detail query の失敗時も movement list を表示する。
- REQ-207 / UI-06c-D6/D7: `source.route` をリンク先に使う。
- UI-06c-D8: Windows native L3 で、商品と在庫の画面からの導線（D-138 の後。今の build は在庫照会）、filter、pagination、元記録リンクの視認性を確認する。
