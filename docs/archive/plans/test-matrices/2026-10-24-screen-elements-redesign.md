# Test Design Matrix: 画面ごとの要素の見直し（商品と在庫の統合、D-138）

本 lane は docs だけで、自動 test を足さない。下の行は後続の runtime の lane（[backlog](../../../backlog.md) の「商品と在庫の画面の統合の runtime」）が実装する test の設計で、設計正本の節と D-138 から導いた。本 lane が自分で回す検査は packet の AC6・AC7（docs の検査、差分が docs だけ）だけ。

「既存」は今の repo にある test file（`rg --files src | rg '<file>'` で在ることを確かめた）で、runtime で期待を改める。「新規」は runtime が足す test の名前の案。

## Risk

Risk: R3

## Contracts Under Test

- UI-01a-D1（[50](../../../function-design/50-ui-product-list.md) §50.2）: 引数なしで開くと状態「すべて」で廃番以外の全商品が商品コード順・100 件・page 1 で出る。旧 58 契約 I（検索駆動表示）の撤去
- UI-01a-D14: 1 つの画面「商品と在庫」、route `/products`、子の route の path は不変、サイドバーとホームの入口
- UI-01a-D15 / §50.4: URL state の `status`（`all` / `low_stock`）・`selected`。「在庫切れ・在庫少」で検索欄・状態・部門だけを出し、廃番の表示・並べ替え・表示件数の URL の値を消さない。条件の変更（状態を含む）で `page` を 1 に戻し `selected` を外す
- UI-01a-D16: 一覧の列を状態で分ける（「すべて」8 列、「在庫切れ・在庫少」7 列）、操作の列なし、PLU の列なし
- UI-01a-D17: 行のクリックで直下に詳細、再クリックで閉じる、詳細から商品修正（`returnTo`）・在庫変動履歴（`returnTo`）・入庫記録（`productCode`、廃番は出さない）
- UI-01a-D18 / §50.7 / catalog ⑥: 何も出ない画面を作らない（失敗・0 件・範囲外 page・商品 0 件）。0 件の説明は条件に合わせる
- UI-01a-D19: `/stock` を `/products` へ寄せる（`q`・`dept`・`selected`・`page` を写し、`page` を写すときは `perPage=50` を足す、`stockout`・`low_stock` を `low_stock` に）
- [58](../../../function-design/58-ui-stock-inquiry.md) 契約 H（D-138 の置換え）: どの状態でも在庫切れ / 在庫少 / 在庫あり。「すべて」は `listLowStock(true)` の集合で在庫少を決める。集合の読込み中・失敗の扱い。集合の key は D-052 の invalidation の集合（`queryKeys.lowStock(false)` を挙げる entry の全部）と独立 oracle に入る
- 58 §58.4 の `selected` ライフサイクル（D-138）: 条件の変更で外す、今の一覧に無ければ外す、検索語なしの `selected` は外さない（検索前ガードの撤去）
- 58 UI-06a-D8 / 廃番除外（置換え）: チップ 2 つ、在庫切れ・在庫少を 1 つの一覧に（廃番を除く）、検索欄は 4 列（`maker_code` を含む）
- 58 UI-06a-D7 / [66](../../../function-design/66-ui-stock-movements.md) UI-06c-D9（D-138）: 在庫変動履歴の `returnTo` は `/products` の URL（`location.href`）、「商品と在庫へ戻る」の pin と fallback（`/products?q=<code>&selected=<code>&discontinued=all`）。`returnTo` は 500 字までで、超えたら fallback に落ちる限定（66 §66.3）
- [51](../../../function-design/51-ui-product-form.md) UI-01b-D2（D-138）: 作る側 `buildProductListReturnTo` と受け口 `parseProductListSearchFromReturnTo` に `status`・`selected` を足す（許す範囲〈`/products` とその search〉は変えない、作る側は手組みのまま）。統合の画面の `returnTo` で保存・戻るが来た一覧とその行へ戻る。例外: 在庫連動を始める拒否（`recount_required`）の回復で棚卸しを経た後の保存は `/products` に着く（b1〈PR #188〉の Plan Review round 3 の決定）
- [61](../../../function-design/61-ui-receiving.md) UI-02-D10・[62](../../../function-design/62-ui-manual-sale.md) UI-04-D12・[63](../../../function-design/63-ui-return-exchange.md) UI-03-D15・[64](../../../function-design/64-ui-disposal.md) UI-05-D11: 保存の結果の保存した商品ごとの「在庫を見る」、引数なしの在庫の画面への link が無い、上の戻る導線「商品と在庫へ」
- 61 UI-02-D18: `productCode` で開くと明細が 1 行入り URL から外れる
- catalog ⑮ の取引 4 画面の先頭の列（61 §61.5）: 入庫の候補と明細に部門・現在庫
- [52](../../../function-design/52-ui-shared-layout.md) UI-12-D1（D-138）: 「商品と在庫」と「在庫切れ・在庫少」の排他 active
- [53](../../../function-design/53-ui-home.md) UI-00-D13: 入口 card の 3・4 枚目
- [77](../../../function-design/77-ui-bulk-price-revision.md) の 0 件の導線の label（「商品と在庫を開く」）と、`/products` へ行く他の導線・文言の画面名（[60](../../../function-design/60-ui-product-import.md) UI-01c-D11 の「商品と在庫へ戻る」ほか、下の Adjacent Pattern Audit）

## Failure Modes

- FM1: 引数なしで開くと一覧が出ない（旧契約 I が残る）か、廃番が混ざる・並びが違う
- FM2: 「すべて」で在庫少の商品を「在庫あり」と出す（集合を読まない・読んだ集合が廃番を含まず廃番の在庫少を見落とす）
- FM3: 在庫少の集合だけが読めないのに「在庫あり」と言い切る、または一覧ごと隠す
- FM4: 状態を切り替えると他の URL の値（並べ替え・廃番の表示・表示件数）が消え、「すべて」へ戻すと条件を失う。または効かない道具が「在庫切れ・在庫少」に出る
- FM5: 「在庫切れ・在庫少」で検索欄がメーカー品番に当たらない（「すべて」と当たる列が違う）、廃番が混ざる、在庫切れか在庫少の片方しか出ない
- FM6: 詳細の「商品修正」の保存・戻るが来た一覧（`status=low_stock` を含む）へ戻らない
- FM7: 数字だけの商品コードが `q`・`selected`・`productCode` の往復で number になり、着地で商品を見失う
- FM8: `/stock` の旧 URL・旧 `returnTo` が着地を失う（404、条件の喪失、`stockout` で空の一覧）
- FM9: 入出庫の保存の結果から引数なしの在庫の画面へ行く導線が残る、または「在庫を見る」が別の商品を開く
- FM10: `productCode` で開いた入庫で F5 のたびに明細が増える、廃番・不明の商品が入る、部分一致で似たコードの商品が入る
- FM11: 0 件の説明が検索語を入れていないのに検索語を変えるよう促す
- FM12: サイドバーで 2 項目が同時に active、またはどちらも active にならない
- FM13: 範囲外 page・商品 0 件の DB で何も出ない
- FM14: 統合の画面に PLU の列・`plu` の絞り込み・一括の操作が残る、または行の「修正」の操作の列が残る
- FM15: 入出庫・商品・閾値の保存の後に、在庫数は新しく状態の欄だけ保存の前の集合のまま（集合の key が D-052 の invalidation に無い）
- FM16: 検索語なしの `selected` が外れて行が開かない（検索前ガードが残る）
- FM17: `/products` へ行く導線の label が旧い画面名のまま（77 の「商品一覧を開く」、一括インポートの「商品一覧へ戻る」「商品一覧を確認」、商品フォームの not found の案内）

## Test Matrix

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| UI-01a-D1 | FM1 | hook（既存 `src/features/products/hooks/useProductList.test.tsx` を改める） | `test_ui01a_d1_default_lists_all_active_products` | 引数なしで `searchProducts` が呼ばれない、または payload が `is_discontinued=false`・`sort_key=ProductCode`・`sort_order=Asc`・`page=1`・`per_page=100` でない。非空の期待: 合成 3 商品が商品コード順に出る |
| UI-01a-D1 / 旧契約 I の撤去 | FM1 | page（既存 `src/features/stock-inquiry/StockInquiryPage.test.tsx` の検索前の案内の test を統合の page の test へ移し、期待を反転） | `test_ui01a_d1_no_search_placeholder_on_open` | 開いた直後に「商品コード、商品名、または JAN コードで検索してください」が出る（旧文言が exact で 0 件、一覧の行が 1 件以上） |
| 契約 H | FM2 | unit（既存 `src/features/stock-inquiry/lib/derive-stock-state.test.ts` を改める） | `test_contract_h_membership_decides_low_in_all` | 在庫 2・集合に入る商品が「すべて」で `ok` になる、在庫 2・集合に入らない商品が `low` になる、在庫 0 が `stockout` 以外になる。各組に非空の期待 |
| 契約 H | FM2 | hook（新規、統合の page の hook） | `test_contract_h_all_reads_low_stock_including_discontinued` | 「すべて」で `listLowStock` が呼ばれない、または引数が `true` でない。廃番の表示「すべて」で在庫少の廃番の商品が「在庫少」にならない |
| 契約 H | FM3 | page（新規） | `test_contract_h_low_stock_failure_shows_unknown_not_ok` | `listLowStock` reject・`searchProducts` 成功で、在庫 > 0 の行の状態の欄が「—」でない（「在庫あり」「在庫少」の label や Skeleton が出る）、一覧が隠れる、知らせと「もう一度読む」が出ない。在庫 0 の行は「在庫切れ」のまま（非空の対照） |
| 契約 H | FM3 | page（新規） | `test_contract_h_low_stock_pending_shows_skeleton` | 集合の読込み中に、在庫 > 0 の行の状態の欄が Skeleton でない（「在庫あり」「在庫少」の label や「—」が出る）。在庫 0 の行は「在庫切れ」のまま |
| 契約 H / 58 §58.8 #1 | FM3 | page（新規） | `test_contract_h_low_stock_list_failure_shows_alert` | `status=low_stock` で `listLowStock` が reject のとき、一覧の失敗の Alert と再試行が出ない、0 件の空状態（「在庫切れ・在庫少の商品はありません」）が出る（失敗を 0 件として扱う）。読込み中に一覧の Skeleton でなく空状態が出る |
| 契約 H / D-052 | FM15 | unit（既存 `src/lib/invalidation-contract.meta.test.ts`・`src/lib/invalidation-contract.static.test.ts` と独立 oracle `src/test/invalidation-oracle.ts` を改める） | `test_d052_low_stock_set_key_invalidated` | `queryKeys.lowStock(false)` を挙げる entry のどれかに統合の画面の集合の key が無い、production の SSOT と独立 oracle が食い違う |
| UI-01a-D15 / §50.4 | FM4 | page（新規、実 router） | `test_ui01a_d15_low_stock_hides_tools_and_keeps_url` | `status=low_stock` で廃番の表示・並べ替え・表示件数・pager が出る、「取引先順」の文字が無い、「すべて」へ戻すと `sort=name&dir=desc&perPage=200&discontinued=all` が失われる、戻った一覧が商品名の降順で並ばない（[58](../../../function-design/58-ui-stock-inquiry.md) UI-06a-D4 の「すべて」は 50 §50.4 に従う。`searchProducts` の payload が `sort_key=Name`・`sort_order=Desc`・`per_page=200` でない、mock が商品名の降順で返した合成 3 商品〈取引先順と違う順にする〉の行が mock の順のまま出ない〈画面で並べ直さない〉） |
| UI-01a-D15 / §50.4 | FM4 | page（新規） | `test_ui01a_d15_condition_change_resets_page_and_selected` | `q`・`dept`・`status`・`discontinued`・`sort`・`dir`・`perPage` の変更で `page` か `selected` が残る。page の移動だけで条件が変わる |
| UI-06a-D8 / 廃番除外 | FM5 | unit（既存 `src/features/stock-inquiry/lib/filter-low-stock-list.test.ts` を改める） | `test_ui06a_d8_low_stock_single_list_maker_code_excludes_discontinued` | `low_stock` で在庫 0 と在庫 2 の両方が出ない、廃番が出る、`q` に `maker_code` の部分を入れて当たらない。並びが取引先名 → 在庫数 → 商品名でない |
| UI-06a-D8 | FM5 | component（既存 `src/features/stock-inquiry/components/StatusChips.test.tsx` を改める） | `test_ui06a_d8_two_chips` | チップが「すべて」「在庫切れ・在庫少」の 2 つでない（「在庫切れ」「在庫少」単独の label が exact で 0 件） |
| UI-01a-D8 | FM14 | component（既存 `src/features/products/components/ProductTable.test.tsx:113` の「状態」見出しの不在の assert を統合の画面の表の test へ移して改める） | `test_ui01a_d8_discontinued_badge_not_column` | 廃番が専用の列で出る、「状態」列に廃番が入る、廃番の商品名セルに「廃番」の badge が無い、「状態」列の見出しが無い（UI-01a-D16） |
| UI-01a-D16 | FM14 | component（既存 `src/features/stock-inquiry/components/ProductListTable.test.tsx` を改める） | `test_ui01a_d16_columns_by_status` | 「すべて」の列見出しが 商品コード・商品名・部門・取引先・状態・在庫数・売価・原価 の順でない、「在庫切れ・在庫少」が 取引先・メーカー品番・商品コード・商品名・状態・在庫数・売価 でない、「PLU」「操作」の見出しがある。展開行の colSpan が列数と違う |
| UI-01a-D17 / UI-01b-D2 | FM6 | page（新規、実 router） | `test_ui01a_d17_edit_returns_to_low_stock_list` | `/products?status=low_stock&dept=3&selected=X` の詳細の「商品修正」→ 保存・戻るで `status=low_stock&dept=3` に戻らない、X の行が開かない |
| UI-01b-D2 | FM6・FM7 | unit（既存 `src/features/products/lib/return-to.test.ts` を改める）+ route（新規、実 router） | `test_ui01b_d2_return_to_round_trips_status_selected_numeric_code` | `buildProductListReturnTo({ status: "low_stock", dept: 3, q: "1234567890123", selected: "1234567890123" })` を `parseProductListSearchFromReturnTo` で読んだ値が、`status`・`selected` を落とす、`q`・`selected` が string の `"1234567890123"` でない（number か引用符つき）。実 router で商品修正の保存の後、`/products` の route の parse の後の `q`・`selected` が string でない、その行が開かない。作る側を router の href（`location.href`）にすると `q` が引用符つきの検索語になって落ちる |
| UI-01a-D17 | FM14 | component（既存 `src/features/stock-inquiry/components/StockDetailContent.test.tsx` を改める） | `test_ui01a_d17_detail_links` | 「商品修正」の `returnTo` が今の search state の `buildProductListReturnTo` の値でない、「入庫記録」の href に `productCode` が無い、廃番の商品で「入庫記録」が出る、「在庫変動履歴」の `returnTo` の pathname が `/products` でない |
| UI-01a-D17 / UI-06a-D5 | — | component（既存 `ProductListTable.test.tsx`） | `test_ui01a_d17_row_click_toggles_detail` | 行のクリックで詳細が開かない、もう一度押して閉じない、行に遷移の button がある |
| §50.4 / UI-06c-D9 | FM7 | route（新規、実 router） | `test_products_route_numeric_code_round_trip` | `search={{ q: "1234567890123", selected: "1234567890123" }}` で開いた後の parse で `q`・`selected` が string の `"1234567890123"` でない、該当の行が開かない（値は JSON の数として読めるものにする。`00123` は先頭の 0 で `JSON.parse` が失敗し、引用符つきの直列化の筋を通らない） |
| 58 §58.4 | FM16 | page（新規） | `test_selected_without_q_opens_detail` | `/products?selected=X`（`q` なし、X は既定の一覧の 1 ページ目にある）で X の詳細が開かない、`selected` が URL から外れる、`getStockDetail` が呼ばれない |
| UI-01a-D19 | FM8 | route（新規、実 router） | `test_stock_redirects_to_products_with_mapping` | `/stock?status=stockout&q=a&dept=2&page=3&selected=X` が `/products?status=low_stock&q=a&dept=2&page=3&perPage=50&selected=X` に着かない（`perPage=50` が無い）。`page` の無い `/stock?q=a` に `perPage` が付く。`status=low_stock` も `low_stock`、`status=all` と不正値は `status` なし。直接の `/products?status=stockout` が route の parse の後に `low_stock` にならない（§50.4） |
| UI-06c-D9（D-138） | FM8 | page（既存 `src/features/stock-movements/StockMovementsPage.test.tsx` の該当の test を改める） | `test_ui06c_d9_back_pins_products` | 「商品と在庫へ戻る」の label でない、`returnTo=/products?...` で元の search に戻らない、`returnTo=/stock?...`・欠落・501 字の `returnTo` で `/products?q=<code>&selected=<code>&discontinued=all` に着かない（数字だけの商品コード、廃番の商品を含む。廃番の商品で行が開かない）。500 字の正しい `returnTo` で元の search に戻らない |
| UI-06c-D9 / [66](../../../function-design/66-ui-stock-movements.md) §66.3（業務記録詳細の側、D-138 で変えない今の挙動） | — | route（実 router。使えない `returnTo` で詳細の「前の画面へ戻る」が `/inventory/records` を指すことは、既存 `src/features/inventory-records/ReturnToFlow.test.tsx` の「T9 REQ-207 / DSR-15: 解析不能になる returnTo でも業務記録詳細が描画され、前の画面へ戻る が既定 hub を指す」が assert している。501 字の `returnTo` を入れる既存の test は無いので runtime の lane で test を足す） | `test_ui06c_d9_record_detail_over_limit_returns_to_records` | 業務記録詳細の route（例 `/inventory/receiving/records/$recordId`）を 501 字の `returnTo` で開いて、「前の画面へ戻る」が詳細の既定 `/inventory/records` でない（在庫変動履歴や `/products` を指す、詳細が描画されない）。500 字の正しい `returnTo` で元の値に戻らない。上の在庫変動履歴の行（501 字で `/products?q=<code>&selected=<code>&discontinued=all`）と着地が違うことの対 |
| UI-02-D10 ほか 4 画面 | FM9 | page（既存 `src/features/receiving/ReceivingPage.test.tsx`・`src/features/return-exchange/ReturnExchangePage.test.tsx`・`src/features/manual-sale/ManualSalePage.test.tsx`・`src/features/disposal/DisposalPage.test.tsx` を改める） | `test_result_panel_view_stock_links_per_saved_item` | 保存の結果に保存した 2 商品の「在庫を見る」が無い、押した先の `/products` の route の parse の後の `q`・`selected` が各商品コードの string でない（数字だけの商品コードを含む。href の文字列は見ない）、その行が開かない、「在庫照会へ戻る」「在庫照会へ」の文言が残る（exact で 0 件） |
| UI-02-D18 | FM10 | page（新規） | `test_ui02_d18_product_code_prefills_once` | `?productCode=X` で明細が 1 行にならない、URL に `productCode` が残る、再 mount（F5 相当）で 2 行になる、`getStockDetail` でなく `searchProducts` で引く |
| UI-02-D18 | FM10 | page（新規） | `test_ui02_d18_unknown_or_discontinued_not_added` | 不明（`not_found`）・廃番の商品コードで明細が入る、案内の文が出ない |
| catalog ⑮ / 61 §61.5・62 §62.5・63 §63.5・64 §64.5 | — | component（入庫は既存 `src/features/receiving/ReceivingPage.test.tsx` に足す。手動販売・返品交換・廃棄の明細の見出しの順は、既存 `src/features/manual-sale/ManualSalePage.test.tsx`・`src/features/return-exchange/ReturnExchangePage.test.tsx`・`src/features/disposal/DisposalPage.test.tsx` の「⑮ SC19: %s は現在庫を単位付きで表示し、数量と単位を同じ cell に添える」が assert している。3 画面の複数件候補の見出しの順を assert する既存の test は無いので runtime の lane で足す） | `test_transaction_candidate_and_line_columns` | 4 画面の複数件候補と明細の先頭の列が 商品コード・商品名・部門・現在庫 でない。名指す文: 入庫は [61](../../../function-design/61-ui-receiving.md) §61.5 の候補と明細表、手動販売は [62](../../../function-design/62-ui-manual-sale.md) §62.5 の候補（後ろに売価）と明細表、返品交換は [63](../../../function-design/63-ui-return-exchange.md) §63.5 の候補と明細表と `docs/SCREEN_DESIGN.md` の返品・交換画面の明細、廃棄は [64](../../../function-design/64-ui-disposal.md) §64.5 の候補（後ろに原価）と明細表（62〜64 の候補と返品の明細は Plan Review round 3 で「商品名、商品コード」から直した文） |
| UI-01a-D18 / catalog ⑥ | FM11 | page（新規） | `test_ui01a_d18_empty_description_matches_conditions` | 検索語なし・部門だけの 0 件で「検索の語を変える」が出る、検索語ありで出ない、「在庫切れ・在庫少」の 0 件の見出しが「在庫切れ・在庫少の商品はありません」でない |
| UI-01a-D18 | FM13 | page（新規） | `test_ui01a_d18_no_blank_screen_cases` | 範囲外 page で「先頭ページに戻る」が出ない（既存の UI-06a-D3 の test を移す）、商品 0 件の DB（絞り込み既定）で「商品がありません」と「商品を登録する」が出ない・「絞り込みを解除」が出る、一覧の失敗で Alert が出ない |
| UI-12-D1（D-138） | FM12 | component（既存 `src/components/layout/SidebarLink.test.tsx`・`src/config/navigation.test.ts` を改める） | `test_ui12_d1_products_exclusive_active` | `/products` で「商品と在庫」だけ、`/products?status=low_stock` で「在庫切れ・在庫少」だけが active でない。「商品検索・一覧」「在庫照会」「在庫少一覧」の項目が残る |
| UI-00-D13 | — | component（既存 `src/features/home/HomePage.test.tsx`） | `test_ui00_d13_quick_action_cards` | 3・4 枚目が「商品と在庫」（`/products`）・「在庫切れ・在庫少」（`/products?status=low_stock`）でない |
| 77 Empty | FM17 | page（既存 `src/features/products/PriceRevisionPage.test.tsx` の「filter なしの 0 件は商品一覧への導線、filter ありの 0 件は「条件に一致する商品がありません」と「絞り込みを解除」を出す」を改める） | `test_ui14_empty_link_label` | 絞り込みなしの 0 件の導線が「商品と在庫を開く」でない、先が `/products` でない、旧 label「商品一覧を開く」が exact で残る |
| 60 UI-01c-D11 / 商品フォームの not found / 69 §69.2・§69.9・§69.12 / 02 ⑧ | FM17 | component（既存 `src/features/products/ProductImportPage.test.tsx`・`src/features/products/ProductFormPage.test.tsx` を改める。在庫少の基準の画面の説明は既存 `src/features/threshold-settings/ThresholdSettingsPage.test.tsx` の「ui11a wording shows the h1 and required field labels matching UI-11a-D6」、廃番の確認の説明は既存 `src/features/products/components/ProductForm.test.tsx` の「discontinue requires confirmation and cancel keeps state」の文言の assert を改める） | `test_old_screen_name_labels_absent` | 一括インポートの見出しの戻ると完了の link が「商品と在庫へ戻る」でない、先が `/products` でない、商品フォームの not found の案内が「商品と在庫へ戻って」で始まらない。在庫少の基準の画面の区画の説明が [69](../../../function-design/69-ui-threshold-settings.md) §69.9 の D-138 の後の文「保存すると、ホームと「商品と在庫」の在庫少の判定にすぐ反映されます」でない（§69.2 の反映先の画面と §69.12 L3-1 の開く画面も同じ「商品と在庫」）。廃番の確認の dialog の説明が [02](../../../design-system/02-component-catalog.md) ⑧ の D-138 の後の文 `商品「{productName}」は商品と在庫の通常表示から外れます。あとから「表示に戻す」で戻せます。` でない。旧い画面名「商品一覧」「在庫照会」が exact で残る |
| UI-01a-D10〜D12 | FM14 | page（既存 `src/features/products/ProductListPage.test.tsx`） | `test_ui01a_no_plu_tools_on_merged_screen` | 統合の画面に「PLU」列・`plu` の絞り込み・一括の対象化の button がある（D-117 の runtime が先なら既存の test のまま） |

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 一覧（`searchProducts`） | 引数なしで全商品（UI-01a-D1） | Skeleton、道具は操作できる | 行を出す | 入出庫・CSV 取込み・商品の保存の後（D-052 の SSOT、今の consumer の集合に統合の画面を入れる） | 条件の変更 | 戻る・`returnTo` で同じ条件 | F5 で URL から同じ一覧 | Alert + 再試行（§50.7） | 再試行で同じ payload | `test_ui01a_d1_*`、`test_ui01a_d18_no_blank_screen_cases` |
| 在庫少の集合（`listLowStock(true)`） | 「すべて」で 1 回 | 在庫 > 0 の状態の欄を Skeleton | 状態を決める | D-052 の `queryKeys.lowStock(false)` を挙げる entry の全部（在庫が動く mutation と閾値の保存。契約 H） | 「もう一度読む」 | 同上 | 同上 | 「—」と知らせ（契約 H） | 「もう一度読む」 | `test_contract_h_*`、`test_d052_low_stock_set_key_invalidated` |
| 開いている行（`selected`） | なし（1 件なら自動展開、UI-06a-D5）。検索語なしの `selected` も開く（58 §58.4） | 詳細の読込み中 | 詳細 | 条件の変更で外す。今の一覧に無ければ外す | 行の再選択 | `returnTo` で同じ行 | F5 で同じ行 | 詳細の inline エラー | 別の行・再試行 | `test_ui01a_d15_condition_change_resets_page_and_selected`、`test_products_route_numeric_code_round_trip`、`test_selected_without_q_opens_detail` |
| 入庫の `productCode` | URL に 1 つ | `getStockDetail` の読込み中 | 明細 1 行、URL から外す | — | — | — | F5 で再度入らない | 案内の文、明細なし | 商品追加欄から入れる | `test_ui02_d18_*` |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| 引数なしの `/stock` への link（E-02） | `rg -n 'to[=:] ?"/stock"' src --glob '!*.test.*'` の出力 11 行（記録の 4 画面の 8 行、サイドバー 2 行、在庫変動履歴 1 行。packet 2.8） | 記録の 4 画面の結果（→「在庫を見る」）と上の戻る導線（→ `/products`）、サイドバー 2 項目（→ `/products`）、在庫変動履歴の fallback（→ `/products`） | なし | `test_result_panel_view_stock_links_per_saved_item`、`test_ui12_d1_products_exclusive_active`、`test_ui06c_d9_back_pins_products` |
| `/products` への link | `rg -n 'to="/products"' src --glob '!*.test.*'` の出力 3 行（`src/features/products/PriceRevisionPage.tsx:125` の 0 件の導線「商品一覧を開く」、`src/features/products/ProductImportPage.tsx:26` の見出しの「商品一覧へ戻る」、同 `:97` の完了の「商品一覧を確認」）、`src/features/products/lib/return-to.ts`、商品フォームの戻る（`src/routes/products/new.tsx:27`・`src/routes/products/$code.edit.tsx:29` の `navigate`）と not found の案内（`src/features/products/ProductFormPage.tsx:222` の「商品一覧へ戻って、もう一度選択してください。」） | 77 の label（「商品と在庫を開く」）、一括インポートの 2 site の画面名（「商品と在庫へ戻る」、[60](../../../function-design/60-ui-product-import.md) UI-01c-D11・§60.5 の result、`docs/SCREEN_DESIGN.md` の商品一括インポート画面。今の build の完了の label「商品一覧を確認」は 60 の文に合わせる）、商品フォームの not found の案内の画面名（「商品と在庫へ戻って…」）、`return-to.ts` の作る側と受け口に `status`・`selected`。正本の label の site: [67](../../../function-design/67-ui-plu-export.md) §67.7 の `confirmed` の操作「商品と在庫へ戻る」（今の build の `src/features/plu-export` に戻る導線は無く、作るときはこの label） | 許す範囲（`/products` とその search、UI-01b-D2）は変えない。作る側を router の href に替えない（数字だけの値の引用符つきの直列化） | `test_ui01b_d2_return_to_round_trips_status_selected_numeric_code`、`test_ui14_empty_link_label`、`test_old_screen_name_labels_absent` |
| 旧い画面名の可視の文言（`/products` への link の外） | `rg -n '商品一覧|在庫照会|商品検索・一覧|在庫少一覧' src --glob '!*.test.*'` の出力 53 行（comment を含む）。行頭の comment を除くと 27 行で、そのうち `src/features/products/search.ts:133` は行末の comment なので、画面に出る文言は 26 行 | 上の 2 行の site に加え、廃番の確認の dialog の「商品一覧の通常表示から外れます」（`src/features/products/components/DiscontinueConfirmDialog.tsx:47`、[02](../../../design-system/02-component-catalog.md) ⑧ の構造の例〈02:533〉）、在庫少の基準の画面の「ホームと在庫照会の在庫少の判定」（`src/features/threshold-settings/ThresholdSettingsPage.tsx:229`、[69](../../../function-design/69-ui-threshold-settings.md) の区画の説明〈69:153〉と L3-1〈69:185〉）を「商品と在庫」に | 統合で書き直す page とナビの題（`ProductListPage.tsx:270`・`:299`、`StockInquiryPage.tsx:101`、`src/config/navigation.ts` の 6 行）と「在庫照会へ戻る」の 10 行は上の行と E-02 の行で扱う。`PriceRevisionPage.tsx:90` の「商品一覧を取得できませんでした」は一括価格改定の自分の候補の一覧の名で、画面名でない。comment は runtime が code とともに直す | `test_old_screen_name_labels_absent`、既存の文言の assert（`src/features/threshold-settings/ThresholdSettingsPage.test.tsx:309`、廃番の確認の文は `src/features/products/components/ProductForm.test.tsx:166`）を改める |
| `returnTo` の数字だけの商品コード（GA5） | 66 UI-06c-D9 の `stockInquirySearchSchema`、`/products` の search schema（`src/features/products/search.ts`）、商品修正の受け口（`src/features/products/lib/return-to.ts` の typed parse-back） | `/products` の `q`・`selected` を string で往復。商品修正の `returnTo` は手組みと typed parse-back のまま | — | `test_products_route_numeric_code_round_trip`、`test_ui01b_d2_return_to_round_trips_status_selected_numeric_code` |
| 商品を探す検索欄の列（catalog ⑨） | 商品一覧・在庫照会の `SearchBar`、一括価格改定の `PriceRevisionFilters`、在庫少の画面の絞り込み `filterAndSortLowStockList` | 在庫少の絞り込みに `maker_code` | 入出庫履歴の記録の検索（記録を探す別の問い）、棚卸しの対象確認欄（D-118 の (b)） | `test_ui06a_d8_low_stock_single_list_maker_code_excludes_discontinued` |
| 取引 4 画面の候補・明細の列（catalog ⑮） | 入庫・返品交換・手動販売・廃棄破損の候補と明細（packet 段 1 の S-07〜S-10） | 入庫の候補に現在庫、明細に部門・現在庫 | 棚卸しの候補（並走の b1・b2、D-118 の (b)） | `test_transaction_candidate_and_line_columns` |

## Negative Paths

- missing input: 引数なしの `/products`（全商品）、`productCode` なしの入庫（今のまま）
- invalid input: `status=foo`・`selected` 101 文字・`perPage=7` は既定へ（§50.4、zod の catch）。`status=stockout` は不正値でなく互換の入力で、`low_stock` として読む（下の Boundary Checks の status/policy enum と Compatibility Checks）
- duplicate/ambiguous input: `productCode` が部分一致で複数の商品に当たる値（完全一致で 1 つだけ）、同じ `productCode` での再 mount
- unknown reference: `selected` が一覧に無い（既存の clear、UI-06a）、`productCode` が `not_found`、`/stock?selected=` の不明な商品
- dependency missing: `listLowStock` の失敗（契約 H）、`listDepartments` の失敗（UI-06a-D2）、`getStockDetail` の失敗（詳細の inline）
- permission/write failure: 該当なし（読取りの画面。入庫の保存の失敗は今の UI-02 の契約のまま）
- dry-run side effect: 該当なし

## Boundary Checks

- threshold: 在庫少の基準ちょうど（`stock_quantity = 基準`）は在庫少（BIZ の `<=`。frontend は集合を使うだけ）
- null/default: `status` なし = `all`、`supplier_name` null は「—」、`maker_code` null の行は検索の対象外で並びは今のまま
- empty/non-empty: 商品 0 件の DB、在庫少の集合 0 件（「すべて」は全行が在庫あり／在庫切れ）、「在庫切れ・在庫少」の 0 件
- min/max: `page` の範囲外、`perPage` 200、`selected`・`productCode` 100 文字、`returnTo` 500 字（受け口ごとに着地が違う。[66](../../../function-design/66-ui-stock-movements.md) §66.3・UI-06c-D9）。在庫変動履歴の `returnTo` は 500 字を受け、501 字は schema で `undefined` になり既定の戻り先〈`/products?q=<code>&selected=<code>&discontinued=all`〉に着く（来た一覧の条件は失うが同じ商品に着く、`test_ui06c_d9_back_pins_products`）。業務記録詳細の `returnTo`（入れ子の `detailReturnTo`）は 501 字で `undefined` になり、詳細の「前の画面へ戻る」は詳細の既定 `/inventory/records` に着く（在庫変動履歴と商品は失う。D-138 で変えない今の挙動、`test_ui06c_d9_record_detail_over_limit_returns_to_records`）
- status/policy enum: `status` は 2 値、旧値 `stockout` は `low_stock`
- wire type: command・DTO は変えない（`search_products`・`list_low_stock`・`get_stock_detail`）
- internal type: `StockStatus` に「集合が読めない」状態を足す（契約 H）
- producer/consumer: URL の producer は統合の画面・記録の 4 画面の結果・在庫変動履歴・`/stock` の redirect、consumer は `/products` の route schema と入庫の route schema
- round-trip token: `returnTo`（入れ子を含む、66 の `detailReturnTo`）、商品修正の `returnTo`（手組みと typed parse-back、51 UI-01b-D2）、`productCode`
- precision/range: 該当なし
- cross-language parse: router の `parseSearch`（JSON.parse）で数字だけの商品コードが number にならない

route / search の往復は href だけでなく、受け側の parse の後の型と行の選択の復元まで確かめる（`test_products_route_numeric_code_round_trip`、`test_ui06c_d9_back_pins_products`）。

## Compatibility Checks

- old schema/input: `/stock` の旧 URL・旧 `returnTo`（`test_stock_redirects_to_products_with_mapping`、`test_ui06c_d9_back_pins_products` の `/stock` の fallback）、旧 `status=stockout`（`/stock` の寄せでも、直接の `/products?status=stockout` でも `low_stock` として読む。[50](../../../function-design/50-ui-product-list.md) §50.4 の `status` の行、`test_stock_redirects_to_products_with_mapping`）。限定: 統合の前の在庫照会で選んだ表示件数 100・200 はローカルの値で URL に無く、寄せた先で再現しない（`page` は `perPage=50` で写す。50 UI-01a-D19）
- new schema/input: `/products` の `status`・`selected`、入庫の `productCode`
- output order: 「すべて」は URL の並べ替え（既定は商品コード昇順）、「在庫切れ・在庫少」は取引先名 → 在庫数 → 商品名
- optional field behavior: `status=low_stock` の間も `sort`・`dir`・`perPage`・`discontinued` を URL に残す（`page` は状態の変更で 1 に戻る）

## Data Safety Checks

- source-derived data: test の商品は合成（`HZ-0047` 型の既存の例）。店の商品名・JAN・金額を使わない
- generated outputs: `src/routeTree.gen.ts` は `npm run generate:routes` の出力
- secrets: 該当なし
- local-only files: 該当なし
- synthetic sample boundaries: 数字だけの商品コード（JSON の数として読める `1234567890123` の型）は合成

## Main Wiring / Integration Checks

- helper connected to main path: `/products` の route が統合の page を描き、`/stock` の route が redirect する（route の生成物と到達の test、`src/config/navigation.test.ts`）
- output reaches manifest/report: 該当なし
- effective config reaches runtime: navigation の 2 項目の `to`・`search`・`activeMatch`（UI-12-D1）
- CLI arg reaches implementation: 該当なし

## Mutation-style Adequacy Questions

- If a mock value is changed so it differs from the design-doc expected value: `listLowStock` の mock の集合を変えて、状態の欄が集合に従う（`test_contract_h_membership_decides_low_in_all`）。
- If invalidate/refetch changes the value before versus after the operation: 入庫の保存の後に「在庫を見る」で開いた行の在庫と状態が保存の後の値（D-052 の invalidation、`test_d052_low_stock_set_key_invalidated`）。
- If a key branch is inverted: `status` の分岐（`all` で `listLowStock` の結果だけを出す）→ 実際の一覧の結果を見る hook / page の `test_ui01a_d1_default_lists_all_active_products`（`searchProducts` が `listLowStock` の集合の外の商品を含む 3 件、`listLowStock` が 1 件を返す fixture で、一覧が 3 件であることを見る。test は後続の runtime の lane が書く）。
- If a threshold comparison changes: frontend は閾値を持たない（集合の有無だけを見る）。BIZ の `<=` は既存の Rust の test。
- If a guard is removed: `productCode` を URL から外す guard → `test_ui02_d18_product_code_prefills_once`。逆に撤去する guard（検索前ガード）が残る → `test_selected_without_q_opens_detail`。廃番の除外 → `test_ui06a_d8_*`・`test_ui02_d18_unknown_or_discontinued_not_added`。
- If an output field is omitted: 詳細の link の `returnTo`・`productCode` → `test_ui01a_d17_detail_links`。
- If output order changes: 列の順 → `test_ui01a_d16_columns_by_status`、並び → `test_ui06a_d8_*`。
- If dry-run performs a side effect: 該当なし。
- If a JSON number crosses JavaScript safe integer range: 該当なし（商品コードは string）。
- If a state token is round-tripped through browser/client code: `returnTo`・`selected` → `test_products_route_numeric_code_round_trip`・`test_ui06c_d9_back_pins_products`。

## Residual Test Gaps

- 道具 6 つ・「すべて」の 8 列が 16px で横に収まるか、状態の見分け、行を開いて直す 2 click の受け止めは自動 test で確かめられない。runtime の Windows native L3 で owner が確かめる（D-138 の Revisit）。
- 商品ごとに違う最低限の数は表さない（D-138 の Guarantee range。packet の Open Questions の Q-3）。
