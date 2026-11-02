# Test Design Matrix: 棚卸しの数の置き場と在庫数の求め直しの runtime (b2)

Plan Packet: [2026-10-23-stocktake-count-runtime-b2](../2026-10-23-stocktake-count-runtime-b2.md)。期待値は設計正本（ADR SPEC-STK-CNT-D1・D2・D3・D9・D12、20・40・44・50・51・58・65・66 の D-118 の節、master の写しの列の CHECK、共通規則 SPEC-UNIT-D3、design-system の badge 3 種構成）から導き、実装や mock から写さない。test の名前は REQ を含む（`docs/DOC_STYLE_GUIDE.md` §1）。b1 の wire（`StockCountKind`・`stock_counts` の書き方）は b1 の Plan Commit `aba17e2e` の版の名前で書き、b1 の merge の後の同期で当て直す（packet の「b1 への依存」）。

## Risk

Risk: R3

## Contracts Under Test

- 商品の結果（`ProductWithRelations`。`get_product`・`search_products`・`get_stock_detail`・`list_low_stock`）が写しの 3 列（`stock_recheck`・`stock_range_low`・`stock_range_high`）を DB のとおりに返す（20・40、ADR D9）。
- `list_movements` が movement と計数を 1 つの履歴に混ぜ、ADR D1 の鍵の降順で返し、業務日と種別で絞り、混ぜた後にページを切り、計数の元記録を棚卸しの記録に結ぶ（44 の D-118 の b2 の節、65 の D-118 の節）。`MovementRecord` の形は変えない。
- 4 画面（商品一覧・在庫照会の一覧と詳細・商品フォームの修正・在庫変動の見出し）が、要再確認の商品で範囲（b1 の `formatStockOrRange` の形「{low}〜{high} {単位}」）と「要再確認」の①状態の badge（文字と icon）を出し、要再確認でない商品は今の 1 つの値のまま（ADR D3、50・51・58・66）。在庫少の判定・並べ替え・ホームの件数は写しのまま（50・58）。
- 在庫照会の詳細と在庫変動の見出しが、要再確認の商品で理由（b1 の `list_recheck_products` の当該の target）と棚卸しの画面への link の案内を出す（58・66、packet の owner の判断事項 (ii)）。
- 在庫変動の表が計数の行を 65 の文で出し、計数の数を行の `count.stock_unit` で描き（商品情報の取得が失敗しても単位が付く）、「日付」の列に業務日と保存の日時と精算回数を出す（44・66）。

## Failure Modes

- 4 つの SELECT のどれかが 3 列を落とす、列の位置がずれて `get_stock_detail` の最終入庫日・最終販売日が別の値になる。
- 計数が履歴に出ない、保存の時刻の順のまま並ぶ、fresh な計数を同じ日付の入出庫より前に置く、初期在庫を同じ日付の記録より後に置く、n が NULL の fresh な計数を精算回数 0 と同じに扱う。
- ページを切ってから並べる（ページをまたいで順が崩れる・行が欠ける・重なる）、`total_count` が movement だけの件数。
- 日付の絞り込みが movement は保存の時刻、計数は業務日（軸の食い違い）。種別「棚卸し」で計数が出ない、「入庫」などで計数が混ざる。取り消した Z004 の movement が出る。
- 計数の元記録が棚卸しの記録でなく架空の link、明細を参照しない計数に link が出る。
- 要再確認でない商品に範囲や badge が出る、要再確認を色だけで示す、範囲の端で在庫少・在庫切れが決まる、長さの商品の範囲を cm で出す。
- 業務記録の詳細の `MovementRecord` の形や「関連する在庫変動」の表が変わる。
- 商品情報の取得（`getStockDetail`）だけが失敗した部分障害で、長さの商品の計数 450 が「450」か「450 個」と出て、「4.5 m」と読めない。
- 要再確認の案内が出ない、別の商品の理由を出す、一覧の読取りの失敗で詳細が止まる、link が棚卸しの画面の入口に届かない。

## Test Matrix

- Before citing an existing test as regression coverage, use `rg` or an equivalent repository search to verify that the cited test exists.
- helper と mock の実装を読み、実際に通る境界と置換される境界を確認して Test Type / coverage を選ぶ。helper 名だけで実 router / integration と分類しない。
- `Would fail if...` は壊れる振舞いを観測できる入力・経路と結びつける。
- oracle の anchor は定義文にしか現れない literal を選び、`rg -c` で対象 file 内 1 件を file ごとに確かめる（複数 file を 1 本の `rg` で数えない）。文言を改訂した後は「新しい文言が exact で 1 件 + 旧い文言が 0 件」の対にする。
- oracle は検証対象と独立の正本から転記し、mutation は production 側だけを変える。
- 結果を空にする注入で、期待が空集合の case だけが kill を主張していないかを確かめ、各組合せに非空の期待を 1 件置く。

### P: 商品の結果の写し（20・40、ADR D9）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| P1 | 要再確認の商品（`stock_recheck` 1、範囲 7〜12。55・73 の例「4〜5」と違う値）と要再確認でない商品（0、範囲 NULL）を fixture に置き、`find_by_product_code`・`search_products`・`get_stock_detail`・`list_low_stock_products` の 4 つが同じ 3 列を返す。`get_stock_detail` は最終入庫日・最終販売日も fixture の値（列の位置のずれを見る） | 3 列の落ち、列の位置のずれ | integration（IO、実 SQLite） | `test_product_reads_req301_req303_carry_copy_recheck_and_range`（`src-tauri/src/db/product_repo.rs`） | 4 つの SELECT のどれかで 3 列を足さない（その関数の結果が既定値になる）、`row_to_product_with_relations` の列の番号を誤る、`get_stock_detail` の `last_receiving_date` の番号をずらし損ねる |
| P2 | `ProductWithRelations` を JSON にすると `stock_recheck`・`stock_range_low`・`stock_range_high` が flatten された商品の field と同じ階層に出る（wire の key の名前） | wire の key の誤り・入れ子 | unit | `test_product_with_relations_req303_serializes_copy_fields_flat`（同上） | field の名前を変える、`#[serde(flatten)]` の外に置かず入れ子にする |

### H: 在庫変動の履歴（44 の D-118 の b2 の節、65、ADR D1）

fixture は 1 商品（個）に、次を保存の順に置く（日付と保存の時刻は合成。movement の `created_at` と `business_date` を別の日にして、保存の時刻の順と鍵の順が違う形にする）: 初期在庫 +10（業務日 D2、`reference_type` NULL・`receiving`）、精算 101 の Z004 −1（精算日 D1 < D2、`phase` 1・`settlement_no` 101）、既定の計数 9（日付 D2）、入庫 +2（業務日 D2）、精算 102 の Z004 −1（精算日 D2）、fresh な計数 10（今日 D2、n = 102）、既定の計数 8（日付 D1。後から前の日の分を入れた）、demo の seed の形の `sale_auto` −1（業務日 D2、`reference_type` NULL）、精算 102 の取消（上の精算 102 の movement を `is_voided` 1 にする）と同じ精算 102 の取り込み直し（精算日 D2 の Z004 −1。`created_at` は fresh な計数 10 より後）。期待の順は ADR D1 の鍵から手で求め、実装や結果から写さない。

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| H1 | 上の fixture の結果の順（新しい順）が、fresh な計数 10 → 取り込み直した精算 102 → 鍵 `(D2,0,NULL,0)` の 3 行（`sale_auto` −1・入庫 +2・既定の計数 9。保存の時刻の新しい順）→ 初期在庫 +10 → 精算 101 → 既定の計数 8、の手で求めた並びと完全一致。各行の `kind`・`business_date`・`settlement_no`・`saved_at` と、`movement` / `count` の片方だけが値。別の商品で、取込みの無いときに保存した fresh な計数（n が NULL、今日 D2）と、その後に取り込んだ精算回数 0 の Z004（精算日 D2）は、Z004 が上（fresh な計数の鍵の 3 つ目は -1 で、精算 0 の 0 より古い） | 鍵の誤り、保存の時刻の順のまま | integration（BIZ、実 SQLite） | `test_list_movements_req303_req205_orders_counts_and_movements_by_ledger_key`（`src-tauri/src/biz/inventory_service/list.rs`） | `created_at` で並べる（既定の計数 8 が先頭近くへ来る）、fresh の `after_settlement` を 0 にする（fresh な計数 10 と取り込み直した精算 102 が同じ鍵 `(D2,1,102,0)` になり、`saved_at` の新しい取り込み直しが fresh な計数 10 より上へ。Plan Review round 1 の是正: 取り込み直しの無い fixture では `saved_at` の順で fresh が上に残り、この mutant が red にならなかった）、計数を末尾にまとめる、n が NULL を 0 とみなす（別の商品の fresh な計数が精算 0 の Z004 より上へ） |
| H2 | 鍵の同じ行（既定の計数と入庫が同じ業務日）は `saved_at` の順で、`saved_at` も同じなら movement が計数より前（逆の順で出すので、表示では計数が上）、最後は id | 全順序でない（実行ごとに順が変わる） | integration | `test_list_movements_req303_same_key_rows_break_ties_by_saved_at_kind_and_id`（同上） | 足しの段を落とす（同じ鍵の行の順が読取りの順に依る）、movement と計数の段を逆にする |
| H3 | 日付の絞り込みは `business_date` の両端を含み、movement と計数の両方に当たる: `created_at` が D3 で業務日が D1 の movement と、日付 D1 の計数は `date_from = date_to = D1` で出て、D3 では出ない | 保存の時刻で絞る、片方だけに当てる | integration | `test_list_movements_req303_date_filter_uses_business_date_for_both_kinds`（同上） | movement を `created_at` で絞る（D1 で movement が消え、D3 で出る）、計数に日付を当てない（D3 で計数が出る）、両端を含めない |
| H4 | 種別の絞り込み: None で movement と計数の全部、`stocktake` で `movement_type = 'stocktake'` の movement（fixture に 1 行足す）と計数、`receiving` で入庫の movement だけ（計数 0 行） | 計数の含め方の誤り | integration | `test_list_movements_req303_type_filter_includes_counts_only_for_all_and_stocktake`（同上） | `receiving` でも計数を混ぜる、`stocktake` で計数を出さない、None で計数を出さない |
| H5 | 混ぜた後にページを切る: H1 の fixture を per_page 3 で読み、page 1・2・3 を連結すると H1 の並びと同じで、どのページも `total_count` が H1 の行の数（取り消した movement を除く） | ページを切ってから並べる、件数が movement だけ | integration | `test_list_movements_req303_paginates_after_merging_and_counts_both_kinds`（同上） | movement と計数をそれぞれ `LIMIT` してから混ぜる（page 2 の行が欠ける・重なる）、`total_count` を movement の件数にする |
| H6 | `is_voided` 1 の movement は出ず、計数は全部出る | 取り消した記録が出る | integration | H1 の fixture の取り消した精算 102 の最初の movement が結果に無いことを H1 で見る | `is_voided = 0` を落とす |
| H7 | 棚卸しの明細を参照する計数は `stocktake_id` と `source`（label「棚卸し #{id}」、route `/stocktake/records/{id}`）を持ち、参照しない計数は両方 None。movement の `source` は今の `resolve_movement_source` のまま。fixture は棚卸しを 2 件置き、1 件目に明細を 2 つ、2 件目に明細を 1 つ作って、計数は 2 件目の棚卸しの明細を参照する（期待の `stocktake_id` は 2 件目の棚卸しの id で、参照する明細の id と違う。test は fixture を作った後に 2 つの id が違うことを前提として assert する） | 架空の link、link の漏れ | integration | `test_list_movements_req207_count_source_links_to_stocktake_record`（同上） | 明細の id を棚卸しの id として出す、参照しない計数に link を作る、label の文字を変える |
| H8 | 初期在庫の印は `reference_type IS NULL AND movement_type = 'receiving'` だけ: 同じ業務日の `sale_auto`（`reference_type` NULL）は鍵の 4 つ目が 0 で、同じ日付の初期在庫より新しい側に並ぶ（H1 の並びの中で見る） | demo の seed の売上を初期在庫として先頭に置く | integration | H1 の並びの `sale_auto` −1 と初期在庫 +10 の位置 | 印を `reference_type IS NULL` だけで決める（`sale_auto` が初期在庫と同じ -1 になり、表示では既定の計数 9 と入庫 +2 より下へ動く） |
| H9 | `date_from`・`date_to` が形・暦の外（`2026-4-7`・`2026-02-30`）なら `ValidationFailed`（31 の `validate_business_date` の文）で、何も読まない | 不正な日付を文字列の大小で比べる | unit（BIZ） | `test_list_movements_req303_rejects_invalid_date_filter`（同上） | 日付の検査を落とす（`2026-4-7` が通り、文字列の比べで別の範囲になる） |

### W: CMD と wire（44、D-061）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| W1 | `StockHistoryKind` の wire の round-trip（`"movement"`・`"count"`） | 生成型と Rust の食い違い | unit | `src-tauri/src/finite_enum_contract_tests.rs` の response の enum の一覧（今の `test_response_only_enum_wire_serialize_req303_req501`）に足す行 | enum の rename を誤る |
| W2 | CMD の `list_movements` が BIZ の `PaginatedResult<StockHistoryItem>` をそのまま返す（計数の行が CMD を通って出る）。test は `tauri::test::mock_builder` で `AppState` を manage し、command の関数 `list_movements(app.state::<AppState>(), query)` を呼ぶ（先例 `src-tauri/src/cmd/product_cmd.rs` の `test_preview_import_req104_empty_file_validation`。BIZ を直に呼ぶ今の `inventory_cmd` の test の形にしない） | CMD が型を変える・行を落とす | unit | `test_list_movements_req303_cmd_returns_history_with_counts`（`src-tauri/src/cmd/inventory_cmd.rs`） | CMD で movement だけを返す（計数 1 行と movement 1 行の fixture で、結果に `kind` が `count` の行が無い） |

### U: 画面（50・51・58・66、design-system）

画面の test の mock の範囲は 7〜12（例の「4〜5」と違う値）、要再確認でない商品の在庫は 3 にする。「要再確認」の文字は badge の中に 1 件、icon は `aria-hidden`。要再確認の一覧の mock（b1 の `listRecheckProducts`）は、先頭に別の商品の target（理由 `conflicting_counts`。b1 の `formatRecountReasons` で「同じ日に違う数が保存されています」）を置き、その後に当該の商品の target（理由 2 つ: `undetermined_settlement`・`same_date_movement`。b1 の label では同じ文「数えた日に売り買い（入出庫）がありました」に 1 つにまとまる）を置く（先頭の target を読む誤りが、当該の商品に無い文として出る並び。理由 1・2 は同じ文なので、別の商品の理由を 1・2 にすると誤りが見えない）。

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| U1 | 商品一覧の在庫の列: 要再確認の行に「7〜12 個」と「要再確認」の badge（icon つき）、要再確認でない行は「3 個」で badge なし。在庫の列の並べ替えの key は今のまま `stock_quantity` | 範囲・badge の漏れ、要再確認でない行に出る | component | `ProductTable.test.tsx` の `U1 recheck product shows range and recheck badge, others show single stock` | 範囲を出さない（在庫の列に共有の部品を使わず `formatStockDisplay(stock_quantity)` のまま）、要再確認でない行にも badge を出す、badge を色だけにする（文字が無い）。範囲の文字の有無は `formatStockOrRange` が範囲の非 null で決め、DB の CHECK で `stock_recheck` と同値なので、「`stock_recheck` を見ずに範囲の有無で決める」は等価な mutant として数えない |
| U2 | 在庫照会の一覧: 範囲と badge は U1 と同じ。状態の badge は写しで決まる（`list_low_stock` 由来で写し 1・範囲 0〜1 の商品は「在庫少」で「在庫切れ」にならない） | 範囲の端で状態が決まる | component | `ProductListTable.test.tsx` の `U2 recheck range does not change stock status derived from copy` | 状態を範囲の下端で決める（「在庫切れ」になる） |
| U3 | 在庫照会の詳細の「在庫数」: 範囲と badge と、その下に要再確認の案内（U10 の部品） | 詳細の漏れ | component | `StockDetailContent.test.tsx` の `U3 detail stock row shows range and recheck badge` | 詳細に部品を使わない、案内を置かない |
| U4 | 商品フォームの修正: 読取り専用の「現在庫」に「7〜12 個」、label の隣に badge（`getProduct` の mock の `stock_recheck` が true。`ProductFormPage` から `ProductForm` へ prop `stockRecheck` で渡る）。何も変えないと dirty にならない（離脱の確認が出ない）。登録の画面は今のまま | 修正の画面の漏れ、範囲の文字で dirty になる | component | `ProductFormPage.test.tsx` の `U4 edit form shows read-only stock range and recheck badge without dirty state`、`product-form-request.test.ts` の `U4 productToFormValues formats recheck range for read-only stock` | `productToFormValues` が範囲を作らない、baseline と値が違って dirty になる、`ProductFormPage` が `stockRecheck` を渡さない（page を通す test で badge が出ない） |
| U5 | 在庫変動の見出しの「現在庫」: 範囲と badge（長さの商品は「4〜4.5 m」）と、その下に要再確認の案内（U10 の部品） | 見出しの漏れ、cm で出す | component | `StockMovementsPage.test.tsx` の `U5 header current stock shows range and recheck badge in meters` | 見出しに部品を使わない、案内を置かない、長さの商品で cm の整数を出す |
| U6 | 計数の行: 既定は「数を確認（{数}、{日付}）」、fresh は「数を入れ直し（{数}、{日付}）」（{数} は `count.count_value` を `count.stock_unit` で `formatStockDisplay` の形にしたもの。個数の商品の fresh 8〈`settlement_no` 102 を持つ〉は「数を入れ直し（8 個、…）」で、その行に「（精算」が出ない、長さの商品の既定 450 は「数を確認（4.5 m、…）」）、「増減」と「変動後在庫」は「—」、`source` があれば link、無ければ「元記録なし」。部分障害（UI-06c-D3）: `getStockDetail` が error で、`listMovements` が長さの商品の既定の計数 450（`stock_unit` `m`）を返すと、商品情報の取得の失敗の表示と並んで「数を確認（4.5 m、…）」が出る | 文の誤り、計数に増減を出す、商品情報の取得の失敗で計数の単位が落ちる | component | `StockHistoryTable.test.tsx` の `U6 count rows show 65 wording with dash quantity and stocktake link`、`StockMovementsPage.test.tsx` の `U6 count row keeps meters from its stock_unit when product detail fails` | 種類の文を取り違える、{数} を movement の数量の形で出す（個数で「8」になる）、計数の値を「増減」に出す、link の有無を逆にする、計数の単位を `count.stock_unit` でなく見出しの商品（`product_code → stock_unit` の対応）から引く（部分障害の case で「450」になる） |
| U7 | 「日付」の列: 業務日（`YYYY-MM-DD`）、Z004 の行は「（精算 {n}）」、下に「保存 {YYYY-MM-DD HH:mm}」。movement の行の種別・増減・元記録は今の `MovementTable` と同じ書き方。fixture の fresh な計数は `settlement_no` を持ち（例 102）、その行の「日付」に「（精算」が出ない（精算の番号を付けるのは Z004 の movement の行だけ。44） | 保存の時刻だけを出す、共有の書き方の食い違い | component | `StockHistoryTable.test.tsx` の `U7 date column shows business date settlement number and saved time` | 「日付」に `created_at` を出す、精算回数を出さない、種別の label を別に持って食い違う、`kind` を見ずに行の `settlement_no` があれば「（精算 {n}）」を付ける（fresh な計数の行に出る） |
| U8 | 在庫の欄の共有の部品（`StockRecheckBadge.tsx`）が商品の写しを b1 の `formatStockOrRange` へ渡す: 個数の要再確認 7〜12 は「7〜12 個」と badge、長さの要再確認 400〜450 は「4〜4.5 m」と badge、負の端 -1〜0 は「-1〜0 個」と badge、要再確認でない写し 3 は「3 個」で badge なし（書式そのものは b1 の `stocktake-formatters.test.ts` の `formatStockOrRange` の case） | 写しの field の渡し違い | component | `StockRecheckBadge.test.tsx` の `U8 stock cell passes copy range to formatStockOrRange and shows recheck badge` | 両端を入れ替えて渡す（「12〜7 個」）、`stock` に範囲の端を渡す、`stock_unit` を渡し損ねて長さで cm の整数を出す、b2 で別の書式の関数を作って単位を両端に付ける（「7 個〜12 個」） |
| U9 | 在庫変動の画面は BIZ の順のまま描く。mock の items は業務日でも `saved_at` でも（降順でも昇順でも）並んでいない順にする（例: 業務日 D2・D3・D1 で `saved_at` S1・S3・S2 の 3 行。D1 < D2 < D3、S1 < S2 < S3）。描いた行の順が mock の順と同じことを見る。計数の保存の entry（`stocktakeCountUpdate`）の consumer に在庫変動の key があること（b1 への依存の 5） | 画面で並べ替える、計数の保存の後に履歴が古いまま | component / unit | `StockMovementsPage.test.tsx` の `U9 history rows render in server order`、b1 の `src/lib/invalidation-contract.meta.test.ts` の今の case（contract と oracle の完全一致。b2 は変えない） | 画面で `saved_at` や業務日で並べ直す（降順でも昇順でも、mock の順と変わる）。b1 の entry から在庫変動の key を落とす（b1 の meta test が止める） |
| U10 | 要再確認の案内（58・66、owner の判断事項 (ii)）: 写しが要再確認なら warning の Alert に題「数え直しが必要です」、mock の当該の target の理由 2 つを b1 の `formatRecountReasons` の文で（1 つにまとまって「数えた日に売り買い（入出庫）がありました」が 1 件）、「棚卸しの画面で、棚の数を数えて入れ直してください。」、link「棚卸しの画面で入れ直す」の href が `/stocktake/?product={商品コード}`（`returnTo` なし）。別の商品の target の文「同じ日に違う数が保存されています」は出ない。要再確認でない商品は何も出さない。一覧が読取りの失敗・当該の target が無いときは、理由を出さずに題・案内・link を出す | 案内の漏れ、別の商品の理由、読取りの失敗で詳細が止まる | component | `StockRecheckNotice.test.tsx` の `U10 recheck notice shows reasons from recheck list and stocktake link` | 一覧の先頭の target の理由を出す（「同じ日に違う数が保存されています」が出て、当該の文が出ない）、`stock_recheck` を見ずに一覧の有無で出す、読取りの失敗で何も出さない、href に `returnTo` を付ける、理由の label を b1 と別に持つ（文字が b1 の label と食い違う） |

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 商品の画面の在庫の欄（4 画面）と要再確認の案内（詳細と在庫変動の見出し） | 写し（要再確認なら範囲と badge と案内） | 読込みは今の Skeleton・loading | 範囲と badge か 1 つの値 | 写しを書く経路の後（b1 の D-052 の entry） | 一覧・詳細・商品フォーム | 入れ直して要再確認が解けると 1 つの値 | DB の写しが正 | 今の error の表示（案内は一覧の読取りの失敗で理由だけを落とす） | 再読込み | P1・U1〜U5・U10 |
| 在庫変動の履歴 | movement と計数を鍵の順 | Skeleton 3 行（今のまま） | 表 | 計数の保存・Z004 の確定と取消・入出庫の後（b1 の entry、U9） | 表と件数 | 絞り込みとページを変える（page は 1 に戻る、今のまま） | 同じ | 今の destructive Alert（UI-06c-D3） | 条件を変える | H1〜H9・U5〜U7・U9 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| 写しの在庫の表示（`formatStockDisplay(…stock_quantity…)`） | `rg -n 'stock_quantity' src/features src/components --glob '!*.test.*'` の表示の行: `ProductTable.tsx`・`ProductListTable.tsx`・`StockDetailContent.tsx`・`product-form-request.ts`・`StockMovementsPage.tsx`、入出庫の候補と行（`DisposalPage.tsx`・`ManualSalePage.tsx`・`ReturnExchangePage.tsx` の候補と `*-row-utils.ts` の `currentStockQuantity`。入庫の画面は在庫を出さない）、`PluExportPage.tsx` | 50・58・51・66 の 5 か所 | 入出庫の候補と行（owner の判断事項 (i)。B に決まった、TD-397）、PLU の書出し（在庫の表示でなく書き出す値の確認。D-117 の範囲） | U1〜U5 |
| 写しの範囲の書き方（「{数}（要再確認 {low}〜{high}）」） | `rg -n 'function formatStockCopy' src`（`IntegrityCheckPage.tsx`・`OperationLogsPage.tsx`） | 無し | 75・74 の診断の書き方（写しの 4 項目を並べる）で、商品の画面の範囲と badge と目的が違う。置き換えない | — |
| ①状態の badge（`StockStatusBadge` の outline + icon + tone） | `src/features/stock-inquiry/components/StockStatusBadge.tsx`、[02](../../design-system/02-component-catalog.md) の tone family の表 | `StockRecheckBadge`（warning） | — | U1〜U5、02 の表の行（S5） |
| 在庫変動の表（`MovementTable`） | `rg -n 'MovementTable' src --glob '!*.test.*'`（在庫変動の画面と業務記録の詳細 6 画面） | 在庫変動の画面は `StockHistoryTable` に替え、種別の label・増減・元記録の link の書き方を共有 | 業務記録の詳細 6 画面は `MovementRecord` のまま `MovementTable` を使う（b1 で棚卸しの記録の詳細の分は外れる） | U6・U7、T3 |
| 要再確認の範囲と理由の文と一覧の query、「今の数を入れる」の入口（b1 の 73・55 の要再確認の一覧、73 の `/stocktake/` の search） | b1 head `3e6a8568` の `src/features/stocktake/lib/stocktake-formatters.ts`（`formatStockOrRange`・`formatRecountReasons`）、`src/features/stocktake/components/RecheckTargetList.tsx`（inline の badge）、`src/features/stocktake/StocktakePage.tsx`（inline の `recheckQuery`） | `StockRecheckBadge` は `formatStockOrRange` を、`StockRecheckNotice` は `formatRecountReasons` と切り出した `useRecheckProducts` を import する | `RecheckTargetList` の inline の badge は同じ形のまま置き換えない（b1 の画面を b2 で変えない） | U8・U10 |
| source link の解決（`resolve_movement_source`） | `src-tauri/src/biz/inventory_service/list.rs` | 計数の元記録（棚卸しの行） | — | H7 |
| 初期在庫の印（ADR D9） | `rg -n "reference_type IS NULL" src-tauri/src/db/inventory_repo.rs`（b1 の後は `list_ledger_movements_for_product` の 1 か所） | 履歴の movement の読取り（同じ定数） | — | H8、Residual の code 読み |
| 生成の enum（D-061） | `src-tauri/src/finite_enum_contract_tests.rs` の一覧 | `StockHistoryKind` | — | W1 |

## Negative Paths

- missing input: 計数が無い商品（movement だけの履歴。H1 の fixture の別の商品で 1 行見る）、要再確認でない商品（U1〜U5 の範囲・badge なし、U10 の案内なし）、要再確認の一覧に当該の target が無い（U10）。
- invalid input: 形・暦の外の日付の絞り込み（H9）、ページの外（今の `test_list_movements_req303_per_page_exceeds_max`）。
- duplicate/ambiguous input: 鍵の同じ行（H2）、同じ値の既定の計数と fresh な計数（H1 の並びの中で別々の行として出る）。
- unknown reference: 棚卸しの明細を参照しない計数（H7）、`source` の無い movement（今の `test_list_movements_req303_null_reference_has_no_source`）。
- dependency missing: 写しが要再確認なのに範囲が null の形は DB の CHECK で作れない（master）。画面は型の上で両端の非 null を見て、範囲を作らない（U1 の分岐で見る）。
- permission/write failure: 該当なし（読取りだけ）。
- dry-run side effect: 読取りだけで何も書かない（`list_movements` と商品の読取り。H1 の前後で表の行数が変わらないことを H1 で確かめる）。

## Boundary Checks

- threshold: per_page 200（今の上限、`test_list_movements_accepts_per_page_200`）。
- null/default: `settlement_no` が null の movement と既定の計数、n が NULL の fresh な計数（H1 の別の商品: 鍵の 3 つ目が -1 で、同じ日付の精算 0 の Z004 より古い側）、`stocktake_id` null（H7）、範囲 null（U1）。
- empty/non-empty: movement も計数も無い商品（今の `test_list_movements_req303_empty`）、計数だけの商品。
- min/max: 範囲の端の負と 0（U8）。
- status/policy enum: `StockHistoryKind`（W1）、`StockCountKind`（U6、b1 の生成型）。
- wire type: `ProductWithRelations` の 3 列（P2）、`StockHistoryItem` の片方だけの値（H1）。
- internal type: 鍵の 4 つと足しの 3 つ（H1・H2）。
- producer/consumer: BIZ の順を画面が保つ（U9）。
- round-trip token: route の search（`dateFrom`・`dateTo`・`type`・`page`）の往復は今の test のまま（T2）。
- precision/range: 写しと計数は在庫の上限の内、精算回数は安全な整数（b1 の Z1・Z4・Z5）。
- cross-language parse: `StockHistoryKind` の TS の literal union（W1 と typecheck）。

## Compatibility Checks

- old schema/input: 計数の無い DB（b1 の前の形の記録だけ）で、履歴は movement だけを今の列で出す（H1 の別の商品）。
- new schema/input: `stock_counts` の行がある DB（H1）。
- output order: 履歴の並びは鍵の降順（H1・H2）。業務記録の詳細の `movements` の順は今のまま（T3）。
- optional field behavior: `movement` / `count` の片方だけが値、`settlement_no`・`stocktake_id`・`source` の null（H1・H7）。

## Data Safety Checks

- source-derived data: 無い（実データを使わない）。
- generated outputs: `src/lib/bindings.ts`、`docs/function-design/90-traceability.md`。
- secrets: 無い。
- local-only files: L3 の DB と、Coordinator が控える b1 の L3 の後の DB の写し（repo の外）。
- synthetic sample boundaries: Rust・TS の test の fixture。

## Main Wiring / Integration Checks

- helper connected to main path: CMD の `list_movements` が BIZ の合成を通る（W2）。4 画面が共有の部品を使う（U1〜U5）。詳細と在庫変動の見出しが案内の部品を使う（U3・U5・U10）。
- output reaches manifest/report: 該当なし。
- effective config reaches runtime: 該当なし。
- CLI arg reaches implementation: 該当なし。

## Mutation-style Adequacy Questions

- If a mock value is changed so it differs from the design-doc expected value, which assertion proves the implementation used the correct source and not the mock's accidental constant? → 画面の mock の範囲は 7〜12 で、設計正本の例「4〜5」「8〜9」と違う値にし、表示がその mock の値を出すことを見る（U1〜U5）。BIZ の test は fixture の記録から順を手で求めて比べる（H1）。
- If invalidate/refetch changes the value before versus after the operation, which test proves the lifecycle order and preserved snapshot are correct? → b2 は再取得を変えない。計数の保存の entry が在庫変動と写しの consumer を持つことは b1 の meta test（U9 の引用）。
- If a key branch is inverted, which test fails? → fresh の `after_settlement`（H1）、初期在庫の -1（H8）、種別の絞り込みの計数の含め方（H4）、`stock_recheck` の分岐（U1）。
- If a threshold comparison changes, which test fails? → 日付の範囲の両端（H3）、per_page 200（今の test）。
- If a guard is removed, which test fails? → `is_voided = 0`（H6）、日付の検査（H9）。
- If an output field is omitted, which test fails? → 3 列（P1・P2）、`settlement_no`・`saved_at`・`source`（H1・H7・U7）。
- If output order changes, which test fails? → H1・H2・H5・U9。
- If dry-run performs a side effect, which test fails? → H1 の行数の確かめ。
- If a JSON number crosses JavaScript safe integer range, which test fails? → 新しい数は在庫の上限と精算回数の範囲の内で、b2 は足し算をしない。b1 の Z1・Z4・Z5 の test が範囲を止める。
- If a state token is round-tripped through browser/client code, which test fails? → route の search の往復（T2 の今の case）。

## 既存の test の移し方

着手時（b1 の merge と同期の後）に下の command を逐語で回し、表と違う所があれば表を直してから進める（2026-10-23 の HEAD `eed07d37` で測った）。分類は 3 つ: **移す**（契約が残る。呼び先と fixture と結果の読み方〈`items[0].movement` 〉を新しい型へ替え、値の期待〈`assert` の右辺・件数・文言〉を変えない）、**期待を変える**（設計正本の契約の変更。正本の節を書く）、**契約ごと外す**（契約が置き換わった。置き換えた節と、同じ失敗を止める Matrix の行を書く）。

### T1 `list_movements` の Rust の test

`rg -n 'fn test_list_movements' src-tauri/src` → 10 行:

| test | 分類 | 理由・代わりの行 |
|---|---|---|
| `src-tauri/src/cmd/inventory_cmd.rs` の `test_list_movements_req303_empty` | 移す | 件数 0 のまま |
| 同 `test_list_movements_req303_per_page_exceeds_max` | 移す | ページの検査は今のまま |
| 同 `test_list_movements_req303_voided_excluded` | 移す | `items[0].movement` の `movement_type` で読む。件数 1 のまま |
| 同 `test_list_movements_req303_type_filter` | 移す | 計数の無い fixture なので件数 1 のまま。計数の含め方は H4 |
| 同 `test_list_movements_req303_date_to_end_of_day` | 契約ごと外す | `created_at` に `T23:59:59` を付ける絞り込み（44 §23.9 の現行の SQL）は、業務日の絞り込みに置き換えた（44 の D-118 の b2 の節）。この test の helper は `created_at` が `2026-03-15T10:00:00`、`business_date` が `2026-04-01` の行を入れるので、業務日で絞ると期待が逆になる。同じ失敗（日付の端の含め方の誤り）は H3 が止める |
| 同 `test_list_movements_req207_source_link_through_biz` | 移す | `items[0].movement.source` で読む |
| `src-tauri/src/biz/inventory_service/list.rs` の `test_list_movements_req303_valid_movement_type` | 移す | — |
| 同 `test_list_movements_accepts_per_page_200` | 移す | — |
| 同 `test_list_movements_req303_includes_source_link` | 移す | `movement.source` で読む |
| 同 `test_list_movements_req303_null_reference_has_no_source` | 移す | 同上 |

### T2 在庫変動の画面の test

`rg -n "^\s*(it|test)\(" src/features/stock-movements` → 20 行（`MovementTable.test.tsx` 4・`movement-formatters.test.ts` 2・`StockMovementsPage.test.tsx` 14）と、`src/features/inventory-records/ReturnToFlow.test.tsx` の `listMovements` の mock:

- `StockMovementsPage.test.tsx` の 14: 14 本とも移す（mock の `listMovements` の items を `StockHistoryItem`〈`kind: "movement"` と `movement`〉に替える。日時を assert する case は無い: `UI-06c-D5: m の商品の在庫変動は…` が見るのは増減・変動後在庫・現在庫の m の書き方で、期待を変えない）。
- `ReturnToFlow.test.tsx`: `listMovements` の mock を同じく替えて移す（href の期待は変えない）。
- `MovementTable.test.tsx` の 4・`movement-formatters.test.ts` の 2: 業務記録の詳細のための `MovementTable` と共有の書き方は変えないので、期待を変えずに残す（T3 と同じ）。

### T3 変えない型の test（`MovementRecord`・業務記録の詳細）

`rg -l 'movements:' src/features/inventory-records --glob '*.test.tsx'` の file と、Rust の業務記録の詳細の test（`rg -n 'movements' src-tauri/src/db/*_repo.rs` の test）は、`MovementRecord` の形を変えないので何も変えずに通る。変える必要が出たら、`MovementRecord` を変えない判断（packet の「起票時に起草役が決めたこと」の 3）が崩れているので、Coordinator に返す。`ProductWithRelations` の literal を持つ 7 file（`rg -l 'plu_memory_no:' src --glob '!src/lib/bindings.ts'`）は 3 列（`stock_recheck: false`・`stock_range_low: null`・`stock_range_high: null`）を足すだけで、値の期待を変えない。

## Residual Test Gaps

- 初期在庫の印の式が `inventory_repo` の 1 つの定数で、求め直しの読取りと履歴の読取りが共有していること（test は振舞い〈H8〉を見るが、2 か所に同じ式を書いても通る）: Final Review の code 読みの項目。
- Windows native の画面での見え方（badge の色と icon、表の密度、長さの商品の m）: L3（packet の Test Plan）。
- 1 商品の履歴が数千行のときの読取りの時間: 未実測。単店舗の見込みの内で、計測は Non-scope（packet の「残りの危険」）。
