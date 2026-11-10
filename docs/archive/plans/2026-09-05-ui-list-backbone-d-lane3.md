# Plan Packet: UI 一覧の背骨 D — Lane 3（ページ送りの横展開: perPage Select 全画面 + 件数文言の自然文化 + 上部帯の太字撤去 + 履歴系 backend 上限 100→200）

Lane 2（PR #32、squash `7e0ccf1`、2026-09-04）で共有部品化した `Pagination` / `PaginationSummary` / `ListShell`（pilot: 商品一覧のみ）を土台に、ページ送り関連の残 3 項目を全一覧画面へ横展開する。`ListShell` 化そのもの（識別列固定・現在行 3 点を含む Lane 4、`--border-strong` sweep を含む Lane 5）は本 lane の対象外。owner 選定（2026-09-05、[Plans.md ④](../../Plans.md) E9/E10/E11 直回答）に基づき、scope を「件数文言の自然文化」「perPage 共有定数 1 本 + 各画面 Select」「履歴系 backend 上限 200」の 3 点に限定する。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

If a state-only commit materializes multiple phases, list the complete adjacent forward sequence and the pre-existing evidence for every intermediate transition in an append-only review/evidence record. Recording compression never permits a gate skip.

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: fa15866
- Amendments: f56a5f7 cc7b0e8 ab3d12a
- Coordinator: Fable 5.1（main session、conductor）
- Writer: Codex（発注書 relay）
- Plan Reviewer: 独立 Sonnet subagent（fresh context）
- Final Reviewer: Sonnet subagent（Fable が P1/P2/P3 裁定）
- Reviewed Content HEAD: eada5fb
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: owner Windows native L3（8 画面の perPage Select 動作 + 入出庫履歴・在庫変動履歴の 200 選択 + 件数文言の新形 + 上部帯の非太字、AC-L3-1〜4）

2026-09-05: PR #34 closeout。hosted final = run 33928546779（head `1881429`、success）→ squash merge `1d44ba2`。`ready-hosted-final -> merge -> archive` を本 closeout commit（packet + Test Matrix の archive 移動、Plans.md ④ の完了表記）に同乗。実績: 介入 4/3（起票 1 / L3 run 1 FAIL / L3 run 2 FAIL / L3 run 3 PASS + 承認 = 超過 1）、relay 2/2（Codex 実装 + 是正）、Gated Amendment 3 本、state-backtrack 2 本、forward state-only 3/3、mutation 累計 25 体 survivor 0。Lane 4 / Lane 5 への申し送りは Plans.md ④ ledger の R2-1〜5 / R3-1〜4 / R5-1〜5 と backlog（mockup-d-lists.html:110 の旧定数名、D-023 の実装側未充足）。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。
介入 3 回の内訳: 1 回目 = 起票選定（2026-09-05、E9/E10/E11 直回答、消費済み）。2 回目 = Windows native L3（AC-L3-1〜4）。3 回目 = 承認 + merge（Coordinator 代行）。

## Consultation Relay

§5.5を使わないchangeは両方`none`のままにする。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
route/search state の変更（入出庫履歴・在庫変動履歴・在庫照会・操作ログ・一括価格改定・整合性チェックの 6 画面へ perPage 状態を新設）+ operator workflow の見た目変更（8 画面共通の件数文言差し替え、上部帯 `font-semibold` 撤去）+ backend バリデーション契約変更（`inventory_service::list.rs:21` `MAX_PER_PAGE` 100→200、対応 BIZ 関数 4 本の受理上限が動く）。DB スキーマ変更・POS CSV・PLU TSV 形式の変更はない。DEV_WORKFLOW Risk Tiers の R3「route/search state, operator workflow, backend validation contract」に該当し、Plan Packet + Test Design Matrix + targeted gates + Windows native L3 を必須とする。

## Goal

Goal Invariant:

### 最小完了条件

- 8 一覧画面（商品一覧 / 棚卸し / 在庫照会 / 入出庫履歴 / 在庫変動履歴 / 操作ログ / 一括価格改定 / 整合性チェック）すべてに perPage `Select`（選択肢 50 / 100 / 200、共有定数 1 本）が表示され、選択に応じて一覧の表示件数が変わる
- 件数文言が新形「全 {n} 件のうち {from}〜{to} 件を表示（{p} / {t} ページ）」で全 8 画面（上部帯 + 下部 pager）に統一される
- 上部帯 `PaginationSummary` から `font-semibold` が外れる
- 入出庫履歴・在庫変動履歴で perPage 200 を選んでも `ValidationFailed` にならない（`MAX_PER_PAGE` 100→200）

### 失敗定義

- 8 画面のいずれかで Select が出ない、選択肢が共有定数と一致しない、または既定 perPage が owner 裁定値と異なる
- 旧文言「{n} 件中 {from}〜{to} 件目 · {p} / {t} ページ」が src / test / catalog ⑩ / DSR-22 のいずれかに残る
- 入出庫履歴・在庫変動履歴で 200 選択時に `ValidationFailed` が発生する、または `MAX_PER_PAGE` 引き上げにより既存の上限超過 test（101 系）が無意味化したまま放置される

### 非目的

- 残り 7 画面の `ListShell` 化（toolbar 2 段・sticky 帯・skeleton 統一）
- 識別列固定 + 表の出っ張り解消（Lane 4、owner 裁定 E12）
- native `<select>`/`<input>` の `--control-surface` token 化 + `--border-strong` sweep（Lane 5、owner 裁定 D8/E13）
- 棚卸し A'+器・完了画面の帯 contrast 是正（棚卸し lane、owner 裁定 G20）
- ページ送りの表上下両配置（catalog ⑩ で open のまま、本 lane では判断しない）
- perPage 選択肢の 40 刻み化（catalog ⑩ で不採用裁定済み、蒸し返さない）

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（2026-09-05、HEAD `6fc4aba`（main、PR #33 merge 後）、すべて本 packet 起草者が rg で再確認）

- **8 画面の現 per_page 値・定数名・file:line・Select 有無**（Explore 前提と突合し全件一致、line 番号のみ現行 HEAD で更新）:
  - 商品一覧: `src/features/products/search.ts:36` `PRODUCT_PER_PAGE_OPTIONS = [50, 100, 200]`、既定 100（`normalizePerPage` fallback 50、URL 明示があれば優先）。Select 実装済み（`ProductListPage.tsx:189-206`、id `product-per-page`、label 「表示件数」）
  - 棚卸し: `StocktakePage.tsx:124` `useState(50)`、`:69` で `PRODUCT_PER_PAGE_OPTIONS` を import。Select 実装済み（label `:752` 「表示件数」、`SelectItem` map `:769` `PRODUCT_PER_PAGE_OPTIONS.map`、id `stocktake-per-page`、perPage 変更で `page: 1` に reset）
  - 在庫照会: `src/features/stock-inquiry/hooks/useStockInquiry.ts:63` `per_page: 50`（`commands.searchProducts` 呼出し、backend は `product_repo.rs:774` `per_page.min(PAGINATION_MAX_PER_PAGE=200)` で 200 まで clamp 済み）。`StockInquiryPage.tsx:200` は `perPage={50}` 固定値を `PaginationSummary`/`Pagination` へ渡すのみ。Select 未実装
  - 入出庫履歴: `src/features/inventory-records/InventoryRecordsPage.tsx:44` `const PER_PAGE = 20`、`:100` `per_page: PER_PAGE`（`list_inventory_records` 呼出し、backend は `inventory_service/list.rs:21` `MAX_PER_PAGE=100` で 101 以上が `ValidationFailed`）。Select 未実装
  - 在庫変動履歴: `src/features/stock-movements/types.ts:52` `MOVEMENTS_PER_PAGE = 20`、`hooks/useStockMovements.ts:49` `per_page: MOVEMENTS_PER_PAGE`（`list_movements` 呼出し、同じ `MAX_PER_PAGE=100`）。Select 未実装
  - 操作ログ: `src/features/operation-logs/OperationLogsPage.tsx:33` `const PER_PAGE = 20`、`:272` `per_page: PER_PAGE`（`commands.listLogs` → `system_service::list_operation_logs` → `system_repo.rs:124` `per_page.min(PAGINATION_MAX_PER_PAGE=200)` で 200 まで clamp 済み、`MAX_PER_PAGE` とは別系統）。Select 未実装
  - 一括価格改定: `src/features/products/priceRevisionSearch.ts:6` で `PRODUCT_PER_PAGE_OPTIONS` を再利用した型 `PriceRevisionPerPage = (typeof PRODUCT_PER_PAGE_OPTIONS)[number]` を定義、`:27-30` は zod `perPage` field の `.refine()` で同定数に対する値検証（型定義と検証ロジックは別行）。Select 実装は未配線（`PriceRevisionPage.tsx` に `<Select>` JSX なし、`PriceRevisionFilters.tsx:102` は `PRODUCT_PER_PAGE_OPTIONS.map` を含むが要素種別は独立確認要）。URL search（`src/routes/products/price-revision.tsx:10` `validateSearch: priceRevisionSearchSchema`）を持つ。backend は `product_repo.rs:774` 経由で 200 まで clamp 済み
  - 整合性チェック: `src/features/integrity-check/IntegrityCheckPage.tsx:45` `const PER_PAGE = 100`、`:97` `mismatches.slice((page-1)*PER_PAGE, page*PER_PAGE)` — **client-side slice**。`:82` の `per_page: 1` は無関係な別クエリ（`commands.listLogs` で直近の整合性チェック実行有無を確認するだけの呼出しで、一覧の pagination とは無関係）。整合性チェックの一覧は `run_integrity_check` の結果を全件メモリに保持して分割表示するため、backend `MAX_PER_PAGE` の影響を受けない。Select 未実装
  - 上記のうち **backend `MAX_PER_PAGE`（`list.rs:21`）の影響を受けるのは入出庫履歴・在庫変動履歴の 2 画面のみ**。在庫照会・操作ログ・一括価格改定・棚卸し・商品一覧は `PAGINATION_MAX_PER_PAGE=200`（`constants.rs:6`）系の silent clamp を経由し、選択肢上限 200 が既に通る。整合性チェックはどちらの系統にも属さない
- **旧文言 `件中` / `件目` の hit 数**（`rg -c` 実測、HEAD `6fc4aba`）:
  - src 側の真の pagination pin: `src/components/patterns/Pagination.tsx:43`（`rangeText` 実装、`Pagination` 下部・`PaginationSummary` 上部の両方がこの 1 関数を共有）
  - test 側 6 file 12 箇所: `Pagination.test.tsx:36,41,46,63`（SC3a/SC3c、上限・端数・0件境界の 3 ケース + summary 1 ケース）/ `ListShell.test.tsx:106` / `OperationLogsPage.test.tsx:257,269,396,408`（4 箇所同一文言）/ `IntegrityCheckPage.test.tsx:410` / `StockInquiryPage.test.tsx:491` / `ProductListPage.test.tsx:630`
  - docs 側で pin している箇所（S11 対象、archived plans と reference 分析 doc は D-050 非遡及のため対象外）:
    - `docs/design-system/02-component-catalog.md` ⑩ ページネーション: `rg -n "件中|件目" docs/design-system/02-component-catalog.md` = **7 hit**（Plan Review round 1 P1-1、当初申告の 3 箇所は過小、訂正済み）。うち S11 対象 5 箇所 = 構造例 2 箇所（`:613`,`:619`）+ 文言節 1 箇所（`:643`）+ 必須構成 2「上下の件数・現在位置」（`:905`）+ Don't 節（`:920`）。**`:943`,`:944` は変更履歴表の行**（Lane 2 完了時点の canonical を記述した過去表記、DSR-22 `:460` と同じ D-050 非遡及対象）で編集しない
    - `docs/design-system/01-decision-rules.md` DSR-22（`:427` 見出し）: ルール文中 2 箇所（`:429`）。同ファイル `:460` は 2026-09-03 時点の変更履歴（当時の canonical を記述した過去表記）で、D-050 の非遡及対象として編集しない
    - `docs/function-design/74-ui-operation-logs.md:263`: 「範囲付き統一形」の pin 1 箇所
    - mockup 4 file: `mockup-d-lists.html`（3 箇所）/ `mockup-d-history.html`（3 箇所）/ `mockup-d-forms-b.html`（1 箇所）/ `mockup-c-products.html`（1 箇所）
    - **偽陽性として除外**（同じ「件中」「件目」文字列だが pagination と無関係、S11 で触らない）: `docs/function-design/36-biz-integrity-check.md:92`（operation_log summary 文言「{checked_count}件中{mismatch_count}件の不整合」）/ `docs/function-design/53-ui-home.md:88`（レビュー指摘の個数を指す「11 件中」）/ `docs/function-design/73-ui-stocktake.md:114`（「1 件目はスキャンできても」の運用描写）/ `docs/function-design/75-ui-integrity-check.md:195`（fixture 設計の「100件目、101件目」）/ `docs/design-system/02-component-catalog.md:943,944`（変更履歴表、D-050 非遡及）
  - **食い違い**: 発注時の想定「DSR-22 / 04-backbone / review-checklist / mockup 注記の旧文言」のうち `docs/design-system/04-backbone.md` と `docs/quality/review-checklist.md` は `rg -c "件中|件目"` = 0（実測、両 file とも hit なし）。この 2 file は S11 の対象から除外する
- **Pagination.test.tsx / ListShell.test.tsx / 各 Page test の文言 oracle の位置**: 上記「旧文言 hit 数」節に列挙した test 側 6 file 12 箇所と同一
- **`.text-base.font-semibold` selector の破損リスク**（実装実読で判明、発注時未記載）: `ListShell.test.tsx:96,116` と `ProductListPage.test.tsx:628,654` が `container.querySelector(".text-base.font-semibold")` で上部 `PaginationSummary` を一意特定している。`font-semibold` を外すと 4 箇所とも selector が何も一致しなくなる。下部 `Pagination` は `text-sm text-muted-foreground`（`font-semibold` を含んだことがない）なので `.text-base` 単独でも上部と衝突しないことを確認済み（S2 の完了条件に採用）
- **`MAX_PER_PAGE` の Rust test の位置と現 boundary test の値**: `src-tauri/src/biz/inventory_service/list.rs:321` `test_list_receivings_req201_per_page_exceeds_max`（`:326` `per_page: 101`）/ `list.rs:503` `test_list_inventory_records_req206_rejects_invalid_page_params`（`:515` `per_page: 101`）/ `src-tauri/src/cmd/inventory_cmd.rs:111` `test_list_movements_req303_per_page_exceeds_max`（`:120` `per_page: 101`）。3 test とも `MAX_PER_PAGE` を 200 へ上げると `per_page: 101` が上限内に収まり `unwrap_err()` が `Ok` を返して panic する（テストの意図が無意味化）。境界値を `201` へ更新しないと即 fail する
- **`MAX_PER_PAGE` を共有する関数**（発注時「2 系統ある事実」より詳細）: `list.rs:21` の `MAX_PER_PAGE` は `validate_page_params`（`list_receivings` / `list_returns` / `list_disposals` / `list_inventory_records` が共有）と `list_movements` 内の直接比較（`:204`）の**両方**から参照される。100→200 の変更は本 lane スコープ外の `list_receivings`/`list_returns`/`list_disposals`（入庫・返品・廃棄の「直近」ミニ一覧、`per_page: 10` 固定で UI から可変にならない）にも同時適用されるが、これらの画面は per_page を UI から変更できないため機能影響はない。設計 doc は「共通型」として書かれているため 3 箇所とも 200 へ揃える（下記 Design Sources）
- **`MAX_PER_PAGE` を設計 doc に明記する 10 箇所**（Plan Review round 1 P1-2/P1-3、当初申告の 3 箇所は過小。`rg -Fn` で全箇所を個別に実在確認済み、旧表記 → 新表記の対で記録する）:
  - `docs/function-design/21-io-inventory-repo.md:7` §10.1「per_page: u32（デフォルト50、上限100。100超はBIZ層でバリデーションエラー）」→「上限200」— `ListQuery` 共通型（入庫/返品/廃棄/入出庫履歴ハブが共有）
  - `docs/function-design/44-cmd-inventory.md:161`「`per_page: u32` — 上限100（BIZ層でバリデーション）」→「上限200」— CMD-02 入庫コマンド群の個別記述
  - `docs/function-design/44-cmd-inventory.md:1056` §23.10「ページパラメータのバリデーション（page >= 1, 1 <= per_page <= 100）」→「1 <= per_page <= 200」— BIZ 層 list ラッパー全般（`list_movements` を含む）
  - `docs/function-design/65-inventory-record-traceability.md:86` §65.4.2「一覧は `page` 1 始まり、`per_page` 上限 100。」→「`per_page` 上限 200。」— `list_inventory_records` 専用
  - `docs/function-design/61-ui-receiving.md:105`「`listReceivings` は `perPage=10` を初期値とし、CMD/BIZ の上限100を UI から超えない。」→「上限200」— UI から可変にならないため機能影響なし（Non-scope、S10 で doc 値のみ同期）
  - `docs/function-design/63-ui-return-exchange.md:118`「`listReturns` は `perPage=10` を初期値とし、CMD/BIZ の上限100を UI から超えない。」→「上限200」— 同上
  - `docs/function-design/64-ui-disposal.md:106`「`listDisposals` は `perPage=10` を初期値とし、CMD/BIZ の上限100を UI から超えない。」→「上限200」— 同上
  - `docs/function-design/66-ui-stock-movements.md:85`「`per_page` は 20 固定。CMD/BIZ の上限 100 を踏まえ、UI から上限超過を送らない。」→「`per_page` は既定 50 + `Select` で 50/100/200 を選択可能（S6）。CMD/BIZ の上限 200 を踏まえ、UI から上限超過を送らない。」— **`per_page` 20 固定の記述自体が S6 の実装で成立しなくなる唯一の箇所**（他 7 箇所は上限値のみの更新）
  - `docs/function-design/62-ui-manual-sale.md:143`「`per_page` は 5 固定で、既存 `listInventoryRecords` の上限 100 内に収める。」→「上限200」— `ManualSalePage.tsx` の recent list（`per_page: 5` 固定、UI から可変にならない）、Non-scope
  - `docs/function-design/10-common-rules.md:91` policy 段落「Inventory movement / record BIZ lists keep the existing `MAX_PER_PAGE = 100` reject contract, and sales import history lists also keep their existing 100 reject behavior.」→「Inventory movement / record BIZ lists now reject above `MAX_PER_PAGE = 200`（D-081 で 100 から引き上げ、reject 方式〈clamp ではなくエラー〉は維持）。Sales import history lists keep their existing 100 reject behavior（本 lane 非対象）.」— 文中 "sales import history" 節は変更しない（S10 で明記）
- **`docs/decision-log.md` D-031（`:220-227`）が本 lane の変更を明示的に例外化している**（Coordinator 発見、Plan Review round 1 最重要指摘）: D-031 の `Exception` 行「`inventory_service`'s `MAX_PER_PAGE = 100` reject contract stays as-is for inventory movement / record BIZ lists. Changing it to clamp would add no operator value and would risk breaking existing tests and UI-06c / traceability contracts that already verify the reject behavior.」は本 packet の S10 と正面から矛盾する。本 lane はこの例外を部分改訂するため、新規 decision-log entry（D-081）を S10 の作業として追加し、D-031 側にも追記を残す（下記 Design Intent Trace L3-D4 / S10 参照）。`rg -n "Amended|Superseded" docs/decision-log.md` で確認した既存書式は `- Superseded in part by: D-0xx は〜を変更する。…は不変。` の 1 行付記のみ（`Amended:` という field 名は repo に前例なし）で、D-081 側も同じ書式（`Decision` / `Status` / `Why` / `Impact` / `Alternatives considered`、D-080 に倣う）で書く
- **REQ 番号**（Design Intent Trace 用、既存 test から実在確認）: 入出庫履歴 = REQ-206（`InventoryRecordsPage.test.tsx:108`）/ 在庫変動履歴 = REQ-303（`StockMovementsPage.test.tsx:82`）/ 在庫照会 = REQ-301（`StockInquiryPage.test.tsx:3`）/ 操作ログ = REQ-902・UI-11c（`OperationLogsPage.test.tsx:77`）/ 一括価格改定 = REQ-105・UI-14（`PriceRevisionPage.test.tsx:142`）/ 整合性チェック = REQ-904・UI-13（`IntegrityCheckPage.test.tsx:104`）/ 商品一覧 = UI-01a / 棚卸し = REQ-205・UI-10。既存 REQ トークンの再利用のみで新規 REQ は追加しないため `generate_traceability` 再生成は不要（S11 で再確認する）

## Scope

- **S1 共有定数の移設**: `src/components/patterns/list-per-page.ts`（新設）に `LIST_PER_PAGE_OPTIONS = [50, 100, 200] as const` を定義（L3-D2）。`src/features/products/search.ts:36` の `PRODUCT_PER_PAGE_OPTIONS` 定義を削除し、`search.ts` / `priceRevisionSearch.ts` / `PriceRevisionFilters.tsx` / `ProductListPage.tsx` / `StocktakePage.tsx` / `search.test.ts` の import を新 module へ切り替える。型 `ProductPerPage`（`search.ts:42`）/ `PriceRevisionPerPage`（`priceRevisionSearch.ts:6`）は `(typeof LIST_PER_PAGE_OPTIONS)[number]` を参照するローカル型エイリアスとして残し、型名自体は変更しない（L3-D2、既存 zod schema の型注釈を壊さない）。完了条件: `rg -n "PRODUCT_PER_PAGE_OPTIONS" src` が 0 件、`rg -n "LIST_PER_PAGE_OPTIONS" src` が 8 箇所以上（旧箇所数と同数以上）
- **S2 `PaginationSummary` の weight 変更**: `Pagination.tsx:99` の className から `font-semibold` を削除（`text-base text-foreground tabular-nums` を維持）。`ListShell.test.tsx:96,116` と `ProductListPage.test.tsx:628,654` の selector を `.text-base.font-semibold` → `.text-base` へ更新し、`ListShell.test.tsx:105` の `expect(summaryTokens).toContain("font-semibold")` を削除する（`toContain("text-base")` は残す）。完了条件: `rg -n "font-semibold" src/components/patterns/Pagination.tsx` が 0 件、`rg -n "text-base.font-semibold" src --glob '*.test.tsx'` が 0 件
- **S3 `rangeText` 文言差し替え**: `Pagination.tsx:33-44` の `rangeText` を新形「全 {n} 件のうち {from}〜{to} 件を表示（{p} / {t} ページ）」へ（`totalCount === 0` は「0 件」のまま）。`Pagination.test.tsx:36,41,46,63` の期待文言を独立転記で更新（production 定数からの導出禁止）。完了条件: `rg -Fn "件中" src/components/patterns/Pagination.tsx` が 0 件、`rg -Fn "全 {" src/components/patterns/Pagination.tsx` は該当なし（テンプレートリテラルなので実際の記法は `` `全 ${...} 件のうち` `` 系、リテラル完全一致は実装後に再確認）
- **S4 在庫照会 Select**: `StockInquiryPage.tsx` にローカル state `perPage`（既定 50、`useState`）を追加し `useStockInquiry.ts:63` の `per_page: 50` 固定値を差し替える。Select は商品一覧と同じ `Select`/`SelectTrigger`/`SelectContent`/`SelectItem` 構成、label 文言「表示件数」、id `stock-inquiry-per-page`。perPage 変更時は表示中の page を 1 に戻す（ローカル state、URL 非対応 — L3-D3）。完了条件: `StockInquiryPage.test.tsx` に Select 変更で `searchProducts` mock の `per_page` が変わることを assert する新規 test
- **S5 入出庫履歴 Select**: `InventoryRecordsPage.tsx:44` の `const PER_PAGE = 20` をローカル state（既定 50、owner 裁定）に置き換え、`:100` の `per_page: PER_PAGE` を state 参照に変更。Select 追加（id `inventory-records-per-page`）。perPage 変更で page を 1 に戻す（ローカル state）。完了条件: 200 選択時に `list_inventory_records` 呼出しの `per_page: 200` が `ValidationFailed` にならないこと（S10 の backend 変更と対）を統合 test で確認
- **S6 在庫変動履歴 Select**: `stock-movements/types.ts:52` の `MOVEMENTS_PER_PAGE = 20` 定数参照を `StockMovementsPage.tsx` 側のローカル state（既定 50）に置き換え、`useStockMovements.ts:49` を state 経由に変更。Select 追加（id `stock-movements-per-page`）。完了条件: S5 と同様、200 選択時に `ValidationFailed` が出ないことを確認
- **S7 操作ログ Select**: `OperationLogsPage.tsx:33` の `const PER_PAGE = 20` をローカル state（既定 50）に置き換え、`:272` を state 参照に変更。Select 追加（id `operation-logs-per-page`）。backend は `system_repo.rs` 経由で 200 まで clamp 済みのため backend 変更は不要
- **S8 一括価格改定 Select**: `PriceRevisionPage.tsx` に `Select` JSX を追加（`priceRevisionSearch.ts` の URL search state `perPage` は既存、UI 未配線分のみ実装）。既定値は owner 決定（2026-09-05）どおり 50。現状の fallback（`priceRevisionSearch.ts:90` の `: 50`）と一致するため値の変更はなく、Select の配線のみ。backend は 200 まで clamp 済みのため backend 変更不要
- **S9 整合性チェック Select**: `IntegrityCheckPage.tsx:45` の `const PER_PAGE = 100`（既定 100、現状維持）をローカル state に置き換え、Select 追加（id `integrity-check-per-page`）。**client-side slice** のため、他画面と異なり「request per_page」ではなく「`visibleMismatches` の件数上限」がテスト対象になる（L3-D6）。完了条件: Select で 50 を選ぶと 1 ページの表示行数が 50 以下になることを assert する新規 test（mock request の `per_page` assert は該当なし）
- **S10 backend `MAX_PER_PAGE` 200 + decision-log D-081**: `src-tauri/src/biz/inventory_service/list.rs:21` の `MAX_PER_PAGE: u32 = 100` を `200` へ変更。既存 3 test の境界値を `101` → `201` へ更新（`list.rs:326`,`list.rs:515`,`src-tauri/src/cmd/inventory_cmd.rs:120`）。新規に `per_page: 200` が `Ok` を返すことを確認する boundary test を `list_inventory_records` と `list_movements` それぞれに追加する。設計 doc **10 箇所**の「上限100」系表記を「上限200」系へ更新（起票時実測の「`MAX_PER_PAGE` を設計 doc に明記する 10 箇所」節の対 anchor どおり）: `21-io-inventory-repo.md:7` / `44-cmd-inventory.md:161,1056` / `65-inventory-record-traceability.md:86` / `61-ui-receiving.md:105` / `63-ui-return-exchange.md:118` / `64-ui-disposal.md:106` / `66-ui-stock-movements.md:85`（`per_page` 固定記述も S6 実装に合わせて改訂）/ `62-ui-manual-sale.md:143` / `10-common-rules.md:91`。`constants.rs:6` の `PAGINATION_MAX_PER_PAGE = 200` は変更しない（既に 200、2 系統併存は `10-common-rules.md:91` の policy 段落が既に記述）。**decision-log 更新**: `docs/decision-log.md` に新規 `## D-081: inventory_service MAX_PER_PAGE を 100 → 200 へ引き上げ、reject 方式は維持（D-031 例外の部分改訂）（2026-09-05）` entry を追加する。書式は D-080（`:650-657`）に倣い `Decision` / `Status: accepted` / `Why`（owner 直回答 E11 = 入出庫履歴・在庫変動履歴で 200 を選ぶ operator value が生じた。clamp へは変えず reject の値だけ動かす。D-031 の懸念「既存 test / UI-06c / traceability 契約を壊す」は本 lane の boundary test 更新〈101→201〉と design doc 10 箇所の同期で解消） / `Impact`（`list.rs:21` の値変更、影響する 5 関数、既存 test 3 本の境界値更新 + 新規 2 本、design doc 10 箇所） / `Alternatives considered`（clamp 化は不採用〈reject という既存契約の性質を変え、UI-06c/traceability の「上限超過はエラー」契約と食い違う〉、100 据え置きは不採用〈owner E11 の要望を満たせない〉）を書く。`docs/decision-log.md:220-227` の D-031 末尾に `- Superseded in part by: D-081（2026-09-05）changes inventory_service's MAX_PER_PAGE reject contract from 100 to 200; the reject mechanism itself is unchanged.` の 1 行を追記する（既存の `Superseded in part by:` 書式に合わせる、`Amended:` という field 名は repo に前例がないため使わない）
- **S11 docs 同期**: 起票時実測の「旧文言 hit 数」節に列挙した非除外・非 archived の doc をすべて新文言へ更新: `02-component-catalog.md`（構造例 2 箇所 `:613,619` + 文言節 `:643` + 必須構成 2「上下の件数・現在位置」`:905` + Don't 節 `:920`、`:943,944` は変更履歴表のため編集しない）/ `01-decision-rules.md` DSR-22（`:429` 2 箇所）/ `74-ui-operation-logs.md:263`/ mockup 4 file（`mockup-d-lists.html` 3 / `mockup-d-history.html` 3 / `mockup-d-forms-b.html` 1 / `mockup-c-products.html` 1）。`04-backbone.md` と `review-checklist.md` は実測 0 hit のため対象外（起票時実測で記録済み）。**catalog ⑩ の個別項目**（Plan Review round 1 P2-1、round 2 で `:612` `:638` 追加）: 構造例 JSX の `:612`（`font-semibold` の class 記述）と `:638`（`PRODUCT_PER_PAGE_OPTIONS.map`）も同時更新する。`:606`「perPage 切替（50 / 100 / 200）は呼び出し側ページの `Select` が担う」文中と `:651`「perPage 規約」節の `PRODUCT_PER_PAGE_OPTIONS` を `LIST_PER_PAGE_OPTIONS` へ改称（S1 と対）/ `:645`「使用トークン」節の上部 `PaginationSummary` class 記述から `font-semibold` を削除（S2 と対）。REQ token を新規追加しないため `generate_traceability` 再生成は不要（実装後に `rg -n "REQ-" src --glob '*.test.tsx'` の差分で再確認）
- **S12 Plans.md ④ + Contract Coverage Ledger 記入**: 本 packet の active link を反映（本 commit で同時実施）。Contract Coverage Ledger は下記節を参照

### Gated Amendment 1（Writer 環境準備時の指摘起源、2026-09-05、Coordinator 起票）

Writer（Codex）が実装着手前の突合で、`docs/function-design/66-ui-stock-movements.md:62`（UI-06c-D2「perPage は 20 固定で、表示件数選択は実利用で必要になったら…」camelCase 表記）が S6 の変更対象から漏れていることを検出し、A) gated amendment で契約を追加 / B) S6 取り下げ の 2 択で停止した。Coordinator 裁定 = **A**。同型の漏れを S4〜S9 の対象画面 UI doc 全 6 file で sweep した結果、在庫変動履歴以外にも 3 file で「固定 perPage」の durable 契約が実装と矛盾したまま残ることが判明したため、まとめて追加する。契約の追加であって owner 決定（既定値・選択肢・文言）の変更ではない。`Amendments` 欄は次の state-only 遷移 commit で本 amendment commit の SHA を記入する（Lane 2 先例 `646fa1c`）。

- **A1-a（在庫変動履歴、S6 の doc 同期）**: `66-ui-stock-movements.md:62` UI-06c-D2 の「perPage は 20 固定で、表示件数選択は実利用で必要になったら追加する」を「perPage は既定 50 + `Select`（50 / 100 / 200、owner 直回答 E10、2026-09-05）。search params は従来どおり `dateFrom` / `dateTo` / `type` / `page` に限定し perPage はローカル state（L3-D3）」へ。`:103` の sample code `per_page: 20,` を `per_page: perPage,`（state）へ。`:85` は既存 S10 対象
- **A1-b（操作ログ、S7 の doc 同期）**: `74-ui-operation-logs.md:58` UI-11c-D8「pagination は `per_page=20` 固定（増減 UI なし）」→「既定 50 + `Select`（50 / 100 / 200）、URL search state は UI-11c-D1 の 4 キー限定を維持し perPage はローカル state（L3-D3）」。`:189` sample `per_page: 20,` → `per_page: perPage,`。`:262`「既定 `per_page = 20`（増減 UI なし。…）」→「既定 `per_page = 50`（`Select` で 50 / 100 / 200）」。`:578` 再利用表「`Pagination`固定20件」→「`Pagination` 既定 50 + `Select`」
- **A1-c（整合性チェック、S9 の doc 同期）**: `75-ui-integrity-check.md:101`「client-side pagingは100件固定」→「client-side paging は既定 100、`Select` で 50 / 100 / 200（L3-D6）」、`:102`「`perPage=100`」→「`perPage={perPage}`（state、既定 100）」、`:183`「client-side 100件固定で再利用」→「client-side 既定 100 + `Select` で再利用」。`:118` の `per_page=1`（直近チェック確認クエリ）は無関係のため触らない
- **A1-d（在庫照会、S4 の doc 同期）**: `58-ui-stock-inquiry.md:147` flow 図と `:240` sample の `per_page: 50` → `per_page: perPage`（state、既定 50）、`:454`「`perPage={50}`」→「`perPage={perPage}`」。`:80` の「デフォルト 50、上限 200」は既定値と一致するため据え置き
- **入出庫履歴・一括価格改定**: 入出庫履歴の UI 契約は `65-inventory-record-traceability.md`（`:86` は S10 対象）と `74-ui-operation-logs.md:262` の言及のみで固定 perPage の独立契約なし。一括価格改定は `77-ui-bulk-price-revision.md:69` が既に「50 / 100 / 200、既定 50」で S8 と一致。いずれも追加対象なし
- **AC 追加**: AC10c（下記）。Contract Coverage Ledger に「UI 契約 4 file の固定 perPage 記述」行を追加。Test Matrix に SC8c を追加
- **起票時実測の訂正**: 「旧文言 hit 数」節の `66-ui-stock-movements.md` は `:85` のみを対象としていたが、`:62`（camelCase）と `:103`（sample）も対象。sweep は snake_case の `per_page` 表記だけで行っており camelCase を取りこぼした（同型の取りこぼしを防ぐため AC10c の anchor は camelCase / snake_case / 「件固定」の 3 表記を個別に持つ）

### Gated Amendment 2（owner L3 run 1 FAIL 起源、2026-09-05、Coordinator 起票）

owner Windows native L3 run 1（Reviewed Content HEAD `cae3d13`）で操作ログ画面が FAIL。owner disposition = **A（3 件すべて本 PR で修正）**。Coordinator が code で機序を裏取りし、同型の漏れを sweep した上で Scope に追加する。owner 決定（既定値・選択肢・文言）の変更は含まない。

- **A2-a（viewport、8 画面 sweep）**: 表示件数変更 handler は `page` を 1 に戻すが scroll を先頭へ戻さないため、長い一覧では 1 ページ目のデータが画面途中から見える。DSR-17 分類①「同一画面内の状態遷移 → event-driven で `scrollPageToTop()` を呼ぶ」に該当。実測: 8 画面とも perPage handler から `scrollPageToTop()`（`src/lib/page-scroll.ts:3`）を呼んでいない。是正 = 8 画面すべての perPage 変更 handler（URL search state 画面は navigate 直後、local state 画面は setState 直後）で `scrollPageToTop()` を呼ぶ。ページ送り（前後 page）時の scroll は既存挙動のまま（本 lane 対象外、L3 run 2 で owner が問題視すれば別 item）
- **A2-b（枠線、操作ログのみ）**: 操作ログの native `<input type="date">` ×2（`OperationLogsPage.tsx:349,363`、class `h-9 rounded-md border px-3`）と native `<select>`（`:377`、`h-9 min-w-52 rounded-md border px-3`）が `--border` を使い、隣の shadcn `Select`（`border-input` = `--border-strong`、`bg-control-surface`）と濃淡差が出る。是正 = 3 要素を `border-input bg-control-surface` へ。これは Plans ④ D8（native 入力欄の token 化 sweep、Lane 5）の操作ログ分の前倒しで、入出庫履歴（既に `border-input`）/ 入庫 / 廃棄の native 入力欄は Lane 5 のまま
- **A2-c（操作種別 registry、5 種別）**: backend が現行機能として出力する `product_price_revise` が `operation-type-labels.ts` に未登録のため「その他（product_price_revise）」fallback で表示される（`74-ui-operation-logs.md:172`「拡張ルール: 新 operation_type を導入する PR は同じ PR で registry へ追加」の drift）。sweep（`src-tauri/src` の `operation_type: "…"` 出力 − registry key、test fixture `test_op` / `z_unknown` は除外）で未登録は 5 種別: `product_price_revise` / `product_bulk_plu_target` / `plu_register_snapshot_import` / `supplier_rename` / `supplier_merge`。是正 = 5 entry を registry と `74-ui-operation-logs.md:146` 付近の registry 表へ追加。ラベル（Coordinator 案、owner は L3 run 2 の表示で確認）:
  - `product_price_revise` → 商品管理 / 一括価格改定
  - `product_bulk_plu_target` → 商品管理 / PLU 対象一括切替
  - `plu_register_snapshot_import` → PLU書出し / PLU 登録状態の取込み
  - `supplier_rename` → 取引先管理 / 取引先の改名（新カテゴリ「取引先管理」を 商品管理 の直後に置く）
  - `supplier_merge` → 取引先管理 / 取引先の統合
- **AC 追加**: AC13（registry 5 key の存在 + 74 doc 表の同期）、AC14（8 画面の perPage handler で `scrollPageToTop` 呼出し）、AC15（操作ログ native 3 要素の token）、AC-L3-5（下記）。Test Matrix に SC9a〜c を追加。mutation 再実測に X15（`scrollPageToTop` 呼出し撤去）/ X16（registry から `product_price_revise` を外す）を追加
- **Workflow**: `human-confirm -> implementing` を state-backtrack で戻し、Writer（Codex）が test 先行で是正 → L1 full → Final Review round 2 + mutation 追加分の再実測 → 遷移（S12 の Plans.md ④ 更新 content commit に同乗、state-only cap 温存）→ owner L3 run 2（canonical 先頭から AC-L3-1〜5）
- **Owner Effort Budget**: 介入は 起票 1 / L3 run 1 = 2 で、L3 run 2 が 3/3。承認は L3 run 2 の PASS 報告と同一 message で受け取り 4 回目を発生させない（Coordinator 責務としてここに記録）

### Gated Amendment 3（owner L3 run 2 FAIL 起源、2026-09-05、Coordinator 起票）

owner Windows native L3 run 2（Reviewed Content HEAD `16c10be`、介入 3/3 消費）で操作ログの AC-L3-5 (a) が再発: 表示件数 50 の 2 ページ目から 100 に変更すると、データは 1 ページ目に戻るが viewport が一覧の途中に残る（owner screenshot あり、非公開）。他画面は途中 scroll 状態での変更が実機で試せず未観測。棚卸し / 一括価格改定 / 整合性チェックは未確認のまま停止。データの登録・更新は無し（owner 確認）。

**機序（Coordinator 委譲の code 実読 + 決定論的再現 test で確定、実装実読の三者整合ではなく再現 test を oracle とする）**:
1. perPage handler（`OperationLogsPage.tsx:404-410`）は `setPerPage` → `update({}, true)`（page reset の `navigate()`、非同期開始）→ `scrollPageToTop()`（`page-scroll.ts:3-11`、`main.scrollTo({top:0, behavior:"smooth"})`）を同期で呼ぶ
2. TanStack Router 内蔵の `setupScrollRestoration`（`router-core/src/router.ts:1131`）が router 構築時に `onRendered` を **最初に** subscribe し、`createAppRouter()`（`src/lib/app-router.ts:63-106`）の独自 `onRendered`（`applyMainNavScroll` → 復元 cache → 遅延再適用）はその後に subscribe される。subscriber は登録順に走る
3. 内蔵の `onRendered`（`router-core/src/scroll-restoration.ts:264-309`）は遷移先 href の cache に `main` の entry があれば `element.scrollTop = scrollY` を同期で直接代入する。page reset 後の href（page param なし）は「以前 1 ページ目を閲覧していた時の位置」を `onBeforeLoad` の snapshot（`:227-231`）で保持している
4. `markMainNavScroll` は Sidebar 経由でしか立たない（`main-nav-scroll.ts`、`SidebarLink.tsx` / `SidebarHeader.tsx` のみ）ため `applyMainNavScroll` は無反応、遅延再適用の `main.scrollTop >= savedScrollTop` 判定も早期 return → 内蔵復元の直接代入が後勝ちし、`scrollPageToTop()` の smooth scroll は中断される
5. 再現: 本番同一の `createAppRouter` + 実 routeTree を mount し、page 1 で 640 → 次のページ → page 2 で 900 → 表示件数 100 を選択 → `main.scrollTop` が `640`（期待 0）で決定論的に fail（3 回連続）。happy-dom の smooth scroll 模擬は実 browser の割込み挙動を再現しないため、`behavior:"smooth"` の `scrollTo` だけ no-op に spy して測定した（実機 WebView2 で「smooth 中の直接代入が目標未到達で止まる」ことは Chromium 一般挙動からの推定、実機では未観測 = 残る未確定事項）
6. 既存 SC9a は `scrollPageToTop` mock の呼出し回数のみを assert し、復元との相互作用と最終位置を検査していなかった（oracle の欠陥）

**是正（Coordinator 裁定 = 委譲報告の案 1 を改良）**: `scrollPageToTop()` 自身が「次の render は強制的に先頭」の flag を立てる形にし、`app-router.ts` の独自 `onRendered`（内蔵復元の後に走る）が flag を消費して `main.scrollTop = 0` を後勝ちで適用する。呼出し元 24 箇所（15 画面）は無改修で恩恵を受け、将来の一覧画面も同型 bug を再発しない。flag は有効期限付き（例: 1 秒）とし、navigate を伴わない呼出し（保存後の DSR-03 型）で立った flag が後続の正規の「戻り」復元（DSR-17 ②）を壊さないようにする。復元 cache の href 運用を pop 限定にする案 2 は、push で実現している「一覧→詳細→戻り」（`app-router.test.tsx` T5/T10/T11）との判別が未検証で本 lane では過大として不採用。
- **A3-a（`src/lib/page-scroll.ts` + `src/lib/app-router.ts`）**: `scrollPageToTop()` で flag を set（期限付き）+ 既存の `scrollTo`。`createAppRouter` の `onRendered` 冒頭（`applyMainNavScroll` と同じ位置）で flag を消費し、有効なら `main.scrollTop = 0` を直接代入し、その render では復元 cache の遅延再適用を skip する。`markMainNavScroll` の既存挙動は不変
- **A3-b（再現 test の tracked 化）**: `src/features/operation-logs/OperationLogsPage.scroll-restoration.test.tsx` を新規作成（本番 `createAppRouter` + 実 routeTree、上記手順、`behavior:"smooth"` の `scrollTo` は no-op spy）。是正前 fail / 是正後 pass。加えて `app-router.test.tsx` に「flag 期限切れ後の戻り復元は従来どおり」「flag 有効中の render は先頭」の 2 case（T5/T10/T11 は不変）
- **A3-c（SC9a の oracle 強化）**: 既存 SC9a は呼出し回数 assert のまま残し、最終位置は A3-b の test が担うことを Matrix に明記（SC9a 単独では本 bug を検出できなかった事実を記録）
- **Writer**: Codex の週次リセット前残量温存（owner 決定 2026-09-05）のため、本 amendment の Writer は **Claude Sonnet 5 subagent（worktree isolation、checkout 必須、packet 編集禁止）** とする。Execution Mode は `fable-window` のまま、独立性（Writer ≠ Final Reviewer、fresh context）は維持
- **AC 追加**: AC16（下記）、AC-L3-6。Matrix に SC10a〜c。mutation に X18（`onRendered` の flag 消費を外す）/ X19（flag の期限判定を外す）
- **Owner Effort Budget（超過、Coordinator 責務）**: 介入 3/3 消費済みのため、DEV_WORKFLOW「ceiling を超えそうな時は最小十分な完了経路へ戻す」に従い、L3 run 3 は **操作ログの AC-L3-5 (a) 再現手順 1 点 + 未確認 3 画面（棚卸し / 一括価格改定 / 整合性チェック）の AC-L3-1 のみ** に絞り、PASS 報告と承認を同一 message で受ける（4 回目 = 超過 1 回として記録）。他画面の再確認は run 1 / run 2 の PASS を history として保持する
- **Workflow**: `human-confirm -> implementing` を state-backtrack（前回 backtrack `e06a847` との間に content commit `d9c83a7` / `16c10be` / `b411c76` / `4c85b3c` / `338cfe0` が実在）→ Sonnet Writer 是正 → Final Review round 3 + mutation → 遷移は content commit 同乗 → L3 run 3

## Non-scope

- 残り 7 画面の `ListShell` 化（toolbar 2 段・sticky 帯・skeleton 統一。owner 裁定、Lane 3〜5 振り分けの前提）
- 識別列固定 + 表の出っ張り解消（Lane 4、owner 裁定 E12「1 item として扱う」だが本 lane には含めない）
- native `<select>`/`<input>` の `--control-surface` #fafaf9 token 化 sweep + outline button/Badge/SegmentedControl の `--border-strong` sweep（Lane 5、owner 裁定 D8/E13）
- 棚卸し A'+器（帯の枠あり）の実装、完了画面の帯 contrast 是正（棚卸し lane、owner 裁定 G20）
- ページ送りの表上下両配置（catalog ⑩ で open のまま判断しない）
- perPage 選択肢の 40 刻み化（catalog ⑩ で不採用裁定済み）
- `list_receivings` / `list_returns` / `list_disposals` の per_page 選択肢拡張（UI から可変にならないため、backend 上限変更の副次的影響を受けるのみで新規 test は追加しない）

## Acceptance Criteria

- AC1: `LIST_PER_PAGE_OPTIONS` が `src/components/patterns/list-per-page.ts` に定義され `[50, 100, 200]` と一致する — vitest（新規 test、production 定数の import ではなく独立転記比較）
- AC2: `PRODUCT_PER_PAGE_OPTIONS` の残存が 0 件 — `rg -n "PRODUCT_PER_PAGE_OPTIONS" src` = 0
- AC3: `Pagination.tsx` の `PaginationSummary` から `font-semibold` が外れている — `rg -n "font-semibold" src/components/patterns/Pagination.tsx` = 0
- AC4: 旧文言「件中」「件目」が pagination 実装・test・対象 docs から 0 件（偽陽性 4 箇所を除く）— `rg -Fn "件中" src/components/patterns` = 0 かつ `rg -Fn "件中" src --glob '*.test.tsx'` = 0（IntegrityCheckPage/StockInquiryPage/OperationLogsPage/ProductListPage/ListShell/Pagination の各 test）
- AC5: 新文言「全 {n} 件のうち」が `Pagination.tsx` に存在する — `rg -Fn "件のうち" src/components/patterns/Pagination.tsx` ≥ 1
- AC6: `list_inventory_records`（`InventoryRecordsPage`）と `list_movements`（`StockMovementsPage`）で `per_page: 200` が `Ok` を返す — `cargo test test_list_inventory_records` / `cargo test test_list_movements` 新規 boundary test 名で確認
- AC7: `MAX_PER_PAGE` 超過境界が `201` に更新されている — `cargo test test_list_receivings_req201_per_page_exceeds_max` / `test_list_inventory_records_req206_rejects_invalid_page_params` / `test_list_movements_req303_per_page_exceeds_max` が pass
- AC8: 6 画面（在庫照会/入出庫履歴/在庫変動履歴/操作ログ/一括価格改定/整合性チェック）それぞれに `Select` が新規追加される — 各 Page.test.tsx に Select 変更 test が存在（vitest test name で確認）
- AC9: 入出庫履歴・在庫変動履歴で perPage 200 選択時に `ValidationFailed` が発生しない — 対応 Page.test.tsx の統合 test（mock backend 経由）
- AC10: 設計 doc 10 箇所それぞれで旧表記が 0、新表記が 1 件以上（対 anchor、file ごとに個別 `rg -Fn` で検査）:
  - `rg -Fn "上限100" docs/function-design/21-io-inventory-repo.md` = 0 かつ `rg -Fn "上限200" docs/function-design/21-io-inventory-repo.md` ≥ 1
  - `rg -Fn "上限100" docs/function-design/44-cmd-inventory.md` = 0（`:161`）かつ `rg -Fn "1 <= per_page <= 100" docs/function-design/44-cmd-inventory.md` = 0（`:1056`）かつ `rg -Fn "上限200" docs/function-design/44-cmd-inventory.md` ≥ 1 かつ `rg -Fn "1 <= per_page <= 200" docs/function-design/44-cmd-inventory.md` ≥ 1
  - `rg -n "per_page.*上限.*100" docs/function-design/65-inventory-record-traceability.md` = 0（`:86`）かつ `rg -n "per_page.*上限.*200" docs/function-design/65-inventory-record-traceability.md` ≥ 1
  - `rg -Fn "上限100" docs/function-design/61-ui-receiving.md docs/function-design/63-ui-return-exchange.md docs/function-design/64-ui-disposal.md` = 0（各 `:105`,`:118`,`:106`）かつ `rg -Fn "上限200" docs/function-design/61-ui-receiving.md docs/function-design/63-ui-return-exchange.md docs/function-design/64-ui-disposal.md` 各 1 件以上
  - `rg -n "上限 100" docs/function-design/66-ui-stock-movements.md` = 0（`:85`）かつ `rg -n "上限 200" docs/function-design/66-ui-stock-movements.md` ≥ 1 かつ `rg -n "per_page.*20 固定" docs/function-design/66-ui-stock-movements.md` = 0（起票時 = 1 hit `:85` の「`per_page` は 20 固定」。バッククォート込みのため literal ではなく regex で検査、Plan Review round 2 P1 是正）かつ `rg -n "既定 50" docs/function-design/66-ui-stock-movements.md` ≥ 1
  - `rg -n "上限 100" docs/function-design/62-ui-manual-sale.md` = 0（`:143`）かつ `rg -n "上限 200" docs/function-design/62-ui-manual-sale.md` ≥ 1
  - `rg -n "MAX_PER_PAGE = 100" docs/function-design/10-common-rules.md` = 0（`:91`）かつ `rg -n "MAX_PER_PAGE = 200" docs/function-design/10-common-rules.md` ≥ 1
- AC10b: `docs/decision-log.md` に D-081 entry が存在し D-031 に付記がある — `rg -Fn "## D-081" docs/decision-log.md` = 1 件、`rg -Fn "Superseded in part by: D-081" docs/decision-log.md` = 1 件
- AC10c（Gated Amendment 1）: UI 契約 4 file の固定 perPage 記述が 0、新記述が 1 件以上（表記ごとに個別 anchor、起票時 hit 数を併記）:
  - `rg -Fn "perPage は 20 固定" docs/function-design/66-ui-stock-movements.md` = 0（起票時 1、`:62`）かつ `rg -Fn "per_page: 20," docs/function-design/66-ui-stock-movements.md` = 0（起票時 1、`:103`）かつ `rg -n "既定 50" docs/function-design/66-ui-stock-movements.md` ≥ 2（`:62` + `:85`）
  - `rg -n "per_page=20.*固定" docs/function-design/74-ui-operation-logs.md` = 0（起票時 1、`:58`）かつ `rg -Fn "per_page: 20," docs/function-design/74-ui-operation-logs.md` = 0（起票時 1、`:189`）かつ `rg -Fn "per_page = 20" docs/function-design/74-ui-operation-logs.md` = 0（起票時 1、`:262`）かつ `rg -Fn "固定20件" docs/function-design/74-ui-operation-logs.md` = 0（起票時 1、`:578`）かつ `rg -n "既定 50" docs/function-design/74-ui-operation-logs.md` ≥ 1
  - `rg -Fn "100件固定" docs/function-design/75-ui-integrity-check.md` = 0（起票時 2、`:101` `:183`）かつ `rg -Fn "perPage=100" docs/function-design/75-ui-integrity-check.md` = 0（起票時 1、`:102`）かつ `rg -n "既定 100" docs/function-design/75-ui-integrity-check.md` ≥ 1
  - `rg -Fn "per_page: 50" docs/function-design/58-ui-stock-inquiry.md` = 0（起票時 3、`:147` `:240` + 1）かつ `rg -Fn "perPage={50}" docs/function-design/58-ui-stock-inquiry.md` = 0（起票時 1、`:454`）かつ `rg -n "既定 50" docs/function-design/58-ui-stock-inquiry.md` ≥ 1（`:80` の「デフォルト 50」は据え置きで可）
- AC11: catalog ⑩・DSR-22 の文言 pin が新形に統一される — `rg -Fn "件のうち" docs/design-system/02-component-catalog.md docs/design-system/01-decision-rules.md` 各 1 件以上、`rg -Fn "件中" docs/design-system/02-component-catalog.md docs/design-system/01-decision-rules.md` が偽陽性（`:943,944` 変更履歴表 + DSR-22 `:460`）を除き 0
- AC11b: catalog ⑩ の `PRODUCT_PER_PAGE_OPTIONS` 残存が 0、`font-semibold` が ⑩ 節内（`:602-665`）で 0 — `rg -Fn "PRODUCT_PER_PAGE_OPTIONS" docs/design-system/02-component-catalog.md` = 0 かつ `rg -Fn "LIST_PER_PAGE_OPTIONS" docs/design-system/02-component-catalog.md` ≥ 1 かつ `rg -n "font-semibold" docs/design-system/02-component-catalog.md` が ⑩ 節（`:602-665`）内で 0
- AC12: `generate_traceability` 再生成が不要であることの確認 — 実装後 `git diff --stat src/**/*.test.tsx` の REQ token 追加差分が 0（新規 REQ を追加していない）
- AC-L3-1（owner Windows native L3）: 8 画面で `Select`（`LIST_PER_PAGE_OPTIONS`）が出て 50/100/200 が選べる
- AC-L3-2（owner Windows native L3）: 入出庫履歴・在庫変動履歴で 200 を選んで `ValidationFailed` にならない
- AC-L3-3（owner Windows native L3）: `Pagination.tsx` の件数文言が新形で表示される
- AC-L3-4（owner Windows native L3）: 上部の `PaginationSummary` 帯が太字（`font-semibold`）でない
- AC13（Gated Amendment 2）: `rg -Fn "product_price_revise" src/features/operation-logs/operation-type-labels.ts` = 1、同様に `product_bulk_plu_target` / `plu_register_snapshot_import` / `supplier_rename` / `supplier_merge` が各 1（起票時 0）。`rg -Fn "product_price_revise" docs/function-design/74-ui-operation-logs.md` ≥ 1（起票時 0）。新規 test が 5 key のラベル文字列を独立転記で完全一致比較する
- AC14（Gated Amendment 2）: 8 画面の perPage 変更 handler で `scrollPageToTop` が呼ばれる — `rg -ln "scrollPageToTop" src/features/products/ProductListPage.tsx src/features/stocktake/StocktakePage.tsx src/features/stock-inquiry/StockInquiryPage.tsx src/features/inventory-records/InventoryRecordsPage.tsx src/features/stock-movements/StockMovementsPage.tsx src/features/operation-logs/OperationLogsPage.tsx src/features/products/PriceRevisionPage.tsx src/features/integrity-check/IntegrityCheckPage.tsx` = 8 file（起票時 0）。各 Page test に「表示件数変更で `scrollPageToTop` mock が 1 回呼ばれる」を追加（handler 経由の呼出しを assert し、ページ送りでは呼ばれないことも 1 case 置く）
- AC15（Gated Amendment 2）: `rg -n 'className="h-9 rounded-md border px-3"|className="h-9 min-w-52 rounded-md border px-3"' src/features/operation-logs/OperationLogsPage.tsx` = 0（起票時 3）かつ `rg -c "border-input bg-control-surface" src/features/operation-logs/OperationLogsPage.tsx` ≥ 3
- AC-L3-5（owner Windows native L3、Gated Amendment 2）: 操作ログで (a) 表示件数を変えると画面が先頭から見える (b) 期間・種別の入力欄と表示件数 Select の枠の濃さが揃っている (c) 一括価格改定のログが「一括価格改定」で表示され「その他（`product_price_revise`）」でない（`OperationLogsPage.tsx` の種別 badge を目視）。他 7 画面でも表示件数変更後に先頭表示になる
- AC16（Gated Amendment 3）: `src/features/operation-logs/OperationLogsPage.scroll-restoration.test.tsx` が存在し（`rg -Fn "createAppRouter" src/features/operation-logs/OperationLogsPage.scroll-restoration.test.tsx` ≥ 1）、`npx vitest run src/features/operation-logs/OperationLogsPage.scroll-restoration.test.tsx` が pass。`rg -n "scrollTop = 0|scrollTop=0" src/lib/app-router.ts` ≥ 1（起票時 0）。`app-router.test.tsx` の既存 T5 / T10 / T11 が不変で pass し、flag 期限切れ / 有効中の 2 case が追加されている
- AC-L3-6（owner Windows native L3 run 3、最小経路）: 操作ログ（`OperationLogsPage.tsx`）で表示件数 50 の 2 ページ目まで scroll した状態から 100 を選ぶと、1 ページ目のデータが**先頭から**表示される（run 2 の再現手順そのもの、`main.scrollTop` = 0 相当を目視）。あわせて棚卸し / 一括価格改定 / 整合性チェックで AC-L3-1（Select と切替）を確認

## Design Sources

List the source design docs this plan relies on. Plan Packets are not durable design source of truth.

- Requirements / spec: `docs/spec/requirements.md`（REQ-105/206/301/303/902/904 は既存）
- Architecture: `docs/ARCHITECTURE.md`（UI → CMD → BIZ → IO 一方向、変更なし）
- Function / command / DTO: `docs/function-design/21-io-inventory-repo.md` §10.1 / `44-cmd-inventory.md` §23.10（+ CMD-02 個別記述 `:161`） / `65-inventory-record-traceability.md` §65.4.2 / `61-ui-receiving.md` / `63-ui-return-exchange.md` / `64-ui-disposal.md` / `66-ui-stock-movements.md` / `62-ui-manual-sale.md` / `10-common-rules.md`（10 箇所、起票時実測節参照） / `59-ui-shared-patterns.md`
- DB: 変更なし
- Screen / UI: `docs/design-system/02-component-catalog.md` ⑩ ページネーション / `docs/design-system/01-decision-rules.md` DSR-22
- Decision log / ADR: `docs/decision-log.md` D-031（`:220-227`、既存の例外裁定）を本 lane で部分改訂する新規 D-081 entry を追加（S10）。L3-D1〜D3/D5/D6 は packet 止まりの新規決定

## Required Design Artifacts

Use `docs/DEV_WORKFLOW.md` Design artifact selection to decide what must exist before implementation.

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 設計 doc 10 箇所（`21-io-inventory-repo.md` §10.1 / `44-cmd-inventory.md` §23.10・`:161` / `65-inventory-record-traceability.md` §65.4.2 / `61-ui-receiving.md` / `63-ui-return-exchange.md` / `64-ui-disposal.md` / `66-ui-stock-movements.md` / `62-ui-manual-sale.md` / `10-common-rules.md`） | updated in this PR（S10） |
| Command / DTO / generated binding / wire shape | `ListQuery` / `MovementQuery` の `per_page` 上限（型は変更なし） | existing sufficient（値のみ変更、DTO shape 不変） |
| DB / transaction / audit / rollback / migration | — | 該当なし |
| Screen / UI / route state / Japanese wording | `02-component-catalog.md` ⑩ / `01-decision-rules.md` DSR-22 | updated in this PR（S11） |
| CSV / TSV / report / import / export format | — | 該当なし |
| durable decision / ADR | `docs/decision-log.md` D-031 の部分改訂（新規 D-081 entry + D-031 への付記） + 本 packet の L3-D1〜D3/D5/D6（Design Intent Trace 参照） | new in this PR（S10） |

## Registration / Generation Obligations

新規追加物に付随する登録・生成義務の checklist。該当なしなら `該当なし` と 1 行残す（節の削除はしない）。

| 新規追加物 | 登録・生成義務 |
|---|---|
| Tauri command（frontend から呼ぶ） | 該当なし（既存 command の per_page 値のみ変更、新規 command なし） |
| function-design doc 新設 | 該当なし（既存 doc 10 箇所の値更新のみ、新設なし） |
| source / workflow doc 新設・改名 | 該当なし |
| AGENT_OPERATING_MANUAL §5.5 consultation relay 使用 | 該当なし（§5.5 不使用） |
| REQ coverage 追加（設計書・テスト追加） | 該当なし（既存 REQ-105/206/301/303/902/904 の再利用のみ、新規 REQ 追加なし。実装後に `rg -n "REQ-" src --glob '*.test.tsx'` の差分で新規 REQ token が無いことを再確認） |
| route 新設 | 該当なし |
| operator 画面新設 | 該当なし（既存画面への Select 追加のみ） |
| durable decision（decision-log 新規 entry） | D-031（`docs/decision-log.md:220-227`）の `Exception` 記述を部分改訂するため、新規 `## D-081` entry を追加し D-031 側に `Superseded in part by: D-081` を付記する（S10、D-080 書式踏襲） |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| — | `02-component-catalog.md` ⑩ / `01-decision-rules.md` DSR-22 | L3-D1（2026-09-05） | 件数文言を「{n} 件中 {from}〜{to} 件目 · {p} / {t} ページ」から「全 {n} 件のうち {from}〜{to} 件を表示（{p} / {t} ページ）」へ。owner 直回答 E9「文言は自然文型で確定」。代替案（現行維持）は owner が明示的に棄却 | `Pagination.tsx` `rangeText` | `Pagination.test.tsx` SC3a/SC3c |
| — | `02-component-catalog.md` ⑩「perPage 規約」 | L3-D2（2026-09-05） | 共有定数を `features/products` から `components/patterns` へ移設。理由: Lane 2 D-9 が確立した「patterns は features を import しない」逆依存禁止の原則を、定数の定義側にも適用する（現状は逆に features 側の定数を patterns 以外の 6 画面が import する歪な構成だった）。型エイリアス名は変更コスト対効果が低いため据え置き | `src/components/patterns/list-per-page.ts`（新設） | `search.test.ts` の定数 import 元更新 |
| — | Plans.md ④ owner 反応 ledger E9〜E11 | L3-D3（2026-09-05） | perPage の永続化先は画面ごとに分岐: URL search state を既に持つ画面（商品一覧・一括価格改定）はそれを踏襲、それ以外（在庫照会・入出庫履歴・在庫変動履歴・操作ログ・整合性チェック）はローカル state のみとし再訪でリセットする。理由: 6 画面に新規 URL search schema を追加するのは本 lane の scope（perPage 横展開のみ）を超える設計変更になる。Coordinator 確定（2026-09-05） | 5 画面の `useState` | 各 Page.test.tsx |
| — | 設計 doc 10 箇所（起票時実測節参照） | **L3-D4 → D-081 へ昇格**（2026-09-05、Plan Review round 1 P1-3 指摘により decision-log 昇格が確定） | `MAX_PER_PAGE`（list.rs:21）は `list_receivings`/`list_returns`/`list_disposals`/`list_inventory_records`/`list_movements` の5関数で共有される単一定数のため、100→200 の変更は入出庫履歴・在庫変動履歴以外にも及ぶ。UI から可変にならない 3 関数（receivings/returns/disposals）は機能影響なしと確認済み。`docs/decision-log.md` D-031 の `Exception`（「`MAX_PER_PAGE = 100` reject contract stays as-is … would add no operator value」）を部分改訂するため、L3-D4 は packet 止まりの決定ではなく decision-log D-081 として昇格する（他の L3-D と異なり Design Intent Audit の「昇格不要」対象から除外） | `list.rs:21` + `docs/decision-log.md`（D-081 新規 + D-031 付記） | `list.rs` 既存 3 test の境界値更新 + 新規 200 OK test 2 本 + `rg -Fn "## D-081" docs/decision-log.md` |
| — | `ListShell.test.tsx` / `ProductListPage.test.tsx` | L3-D5（2026-09-05） | `font-semibold` 撤去に伴い `.text-base.font-semibold` selector が機能しなくなるため `.text-base` 単独へ切替。下部 `Pagination` は `text-sm` のため衝突しないことを確認済み | `ListShell.test.tsx:96,116` / `ProductListPage.test.tsx:628,654` | 同上（selector 更新自体が contract） |
| — | `IntegrityCheckPage.tsx` | L3-D6（2026-09-05） | 整合性チェックは `run_integrity_check` 全件取得後の client-side slice のため、他画面のような「request per_page」オラクルが成立しない。Select の効果は「1 ページの表示行数上限」で検証する | `IntegrityCheckPage.tsx:45,97` | `IntegrityCheckPage.test.tsx` 新規 test |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 本 packet の「起票時実測」節と Design Intent Trace が値・理由の一次情報。実装後は S11 で catalog ⑩ / DSR-22 に文言 pin を反映し、packet 依存を解消する
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: L3-D1（文言）は S11 で catalog ⑩・DSR-22 へ反映する。**L3-D4 は decision-log D-081 へ昇格**（既存 D-031 の `Exception` 裁定を部分改訂するため packet 止まりでは足りないと Plan Review round 1 で指摘・確定。S10 で design doc 10 箇所へも反映）。L3-D2/D3/D5/D6 は実装詳細のため packet 止まりで良い（source doc への昇格不要、Coordinator 裁定）
- Assumptions and constraints: 画面別既定 perPage は owner 決定（2026-09-05）で全 8 画面確定済み。URL param 対象画面の判定（L3-D3）は Coordinator 確定（2026-09-05）、Plan Review は妥当性を検査する
- Deferred design gaps, risk, and follow-up target: `list_receivings`/`list_returns`/`list_disposals` の 200 boundary test は本 lane で追加しない（UI 非公開のため）。Lane 4/5 の contract audit で拾う候補として記録
- Test Design Matrix can cite design decision IDs or source doc sections: Yes（[Test Matrix](test-matrices/2026-09-05-ui-list-backbone-d-lane3.md) 各行に L3-D 番号か DSR-22/catalog ⑩ 節番号を付す）
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 例外なし。全 8 画面が同一 `LIST_PER_PAGE_OPTIONS` を参照し、backend 上限緩和は既存 5 関数すべてに一律適用される（部分適用の抜け道なし）

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable — UI ↔ BIZ の既存境界を変更しない | — |
| Fact check / design decision split | 適用: 起票時実測で「発注時の想定と食い違う点」を複数発見（04-backbone/review-checklist の 0 hit、`.text-base.font-semibold` selector 破損、IntegrityCheck の client-side slice） | 本 packet の「起票時実測」節 |
| Lifecycle / retry | not applicable | — |
| Operator workflow | 適用: 8 画面の一覧操作に Select が増える。owner L3 で操作性を確認（AC-L3-1） | AC-L3-1〜4 |
| Replacement path | not applicable | — |
| Data safety / evidence | not applicable — DB 書込みなし | — |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | 適用: owner Windows native L3（AC-L3-1〜4） | Human Gate |
| 環境・再現性 | not applicable — toolchain / CI runner 変更なし | — |

## Design Readiness

- Existing design docs are sufficient because: catalog ⑩・DSR-22 は Lane 1a/2 で正本化済みで、本 lane は既定値・上限値の数値更新と全画面展開のみ
- Source docs updated in this PR: `02-component-catalog.md` ⑩ / `01-decision-rules.md` DSR-22 / `21-io-inventory-repo.md` §10.1 / `44-cmd-inventory.md` §23.10 / `65-inventory-record-traceability.md` §65.4.2
- Design gaps intentionally deferred: なし（既定 perPage は 8 画面すべて owner 決定済み）
- Durable decisions discovered in this plan and promoted to source docs: L3-D1（文言）/ L3-D4（`MAX_PER_PAGE` 共有範囲の明記）

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 維持。UI 層の state 追加のみ、CMD/BIZ/IO の呼出し方向は変更しない
- Backend function design: `list.rs` の `MAX_PER_PAGE` 定数値変更のみ、シグネチャ不変
- Command / DTO / data contract: `ListQuery`/`MovementQuery` の shape 不変、許容値域のみ変更
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 件数文言統一、Select ラベル「表示件数」で統一
- Error, empty, retry, and recovery behavior: `ValidationFailed` の発火境界が 101→201 に移動。0 件時の表示（「0 件」）は変更しない
- Testability and traceability IDs: 既存 REQ-105/206/301/303/902/904 を再利用、新規 REQ 追加なし

## Contract Probe

Required for R3/R4 plans that rely on an unverified external premise. If not applicable, state N/A and the reason.

- IntegrityCheckPage の Select が client-side slice にのみ作用し backend request を発火しないという前提: `IntegrityCheckPage.tsx:76-99` の実読（起票時実測）で `mismatches` が `run_integrity_check` の単発結果を `useState` 保持していることを確認済み → 実装時に再度 `IntegrityCheckPage.tsx` を読んで前提が崩れていないか確認するのみで、追加の実験は不要（N/A、静的コード確認で足りる）
- `.text-base` 単独 selector が `ListShell`/`ProductListPage` の DOM で他要素と衝突しないという前提: 起票時実測で下部 `Pagination` が `text-sm` であることを確認済み。実装時に `screen.debug()` 等で他の `text-base` 要素混入がないか確認する（Writer completion 条件、Contract Probe 自体は N/A）

## Contract Coverage Ledger

Required for R3/R4. Include every contract or design decision in the touched source-doc sections; a missing row is a Plan Gate blocker.

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| L3-D1 件数文言 | `Pagination.tsx` `rangeText` | `Pagination.test.tsx` SC3a/SC3c（更新） | AC-L3-3 |
| L3-D2 共有定数移設 | `list-per-page.ts` + 6 file の import 更新 | `search.test.ts`（更新）+ 新規 `list-per-page.test.ts` | non-scope（L3 対象外、機械的契約） |
| L3-D3 perPage 永続化先の分岐 | 5 画面の `useState` | 各 Page.test.tsx | AC-L3-1 |
| L3-D4 → D-081 `MAX_PER_PAGE` 100→200 引き上げ | `list.rs:21` + `docs/decision-log.md`（D-081 新規 + D-031 付記） | `list.rs` 3 test 更新 + 新規 2 test + `rg -Fn "## D-081" docs/decision-log.md` | AC-L3-2 / AC10b |
| UI 契約 4 file の固定 perPage 記述（Gated Amendment 1: `66-ui-stock-movements.md:62,103` / `74-ui-operation-logs.md:58,189,262,578` / `75-ui-integrity-check.md:101,102,183` / `58-ui-stock-inquiry.md:147,240,454`） | S4 / S6 / S7 / S9 の doc 同期 | AC10c の表記別 anchor（camelCase / snake_case / 件固定） | AC-L3-1 / AC10c |
| L3-D5 `.text-base` selector | `ListShell.test.tsx` / `ProductListPage.test.tsx` | 同左（selector 更新） | non-scope（内部 test 契約） |
| L3-D6 整合性チェック client-side slice | `IntegrityCheckPage.tsx` | `IntegrityCheckPage.test.tsx` 新規 | AC-L3-1 |
| DSR-22 件数文言 pin | `01-decision-rules.md:429` | 該当なし（docs review） | non-scope |
| catalog ⑩ 件数文言 pin + perPage 規約 + 定数名 + weight | `02-component-catalog.md:613,619,643,651,905,920,645` | 該当なし（docs review） | non-scope |
| 21-io-inventory-repo.md §10.1 上限値 | `docs/function-design/21-io-inventory-repo.md:7` | 該当なし（docs review） | non-scope |
| 44-cmd-inventory.md 上限値（§23.10 + CMD-02 個別記述） | `docs/function-design/44-cmd-inventory.md:161,1056` | 該当なし（docs review） | non-scope |
| 65-inventory-record-traceability.md §65.4.2 上限値 | `docs/function-design/65-inventory-record-traceability.md:86` | 該当なし（docs review） | non-scope |
| 61/63/64-ui-*.md の上限値記述（receivings/returns/disposals） | `61-ui-receiving.md:105` / `63-ui-return-exchange.md:118` / `64-ui-disposal.md:106` | 該当なし（docs review） | non-scope |
| 66-ui-stock-movements.md の上限値 + per_page 固定記述 | `docs/function-design/66-ui-stock-movements.md:85` | `StockMovementsPage.test.tsx` 新規（S6 と対） | AC8 |
| 62-ui-manual-sale.md の上限値記述 | `docs/function-design/62-ui-manual-sale.md:143` | 該当なし（docs review） | non-scope |
| 10-common-rules.md の pagination policy 段落 | `docs/function-design/10-common-rules.md:91` | 該当なし（docs review） | non-scope |
| `list_receivings`/`list_returns`/`list_disposals` の上限緩和副作用 | `list.rs:21`（共有定数） | 追加テストなし（UI 非公開のため） | non-scope（Lane 4/5 audit 候補として記録） |

## Test Plan

For R3/R4, include or link a Test Design Matrix: [test-matrices/2026-09-05-ui-list-backbone-d-lane3.md](test-matrices/2026-09-05-ui-list-backbone-d-lane3.md)
- If the Human Gate includes L3, Writer completion includes `cargo check --release` before the owner native build; this is not a CI gate.

- targeted tests: 8 画面の Select 動作 test（新規 6 + 既存 2 画面の import 更新確認）、`Pagination.test.tsx` SC3a/SC3c 更新、`list.rs`/`inventory_cmd.rs` の boundary test 更新 + 新規
- negative tests: perPage 201 相当（既存 3 test の境界値更新）、`ValidationFailed` を維持する経路の確認
- compatibility checks: `PRODUCT_PER_PAGE_OPTIONS` 依存箇所の網羅的置換確認（`rg` 0 hit）
- data safety checks: 該当なし（DB 書込みなし）
- main wiring/integration checks: 各画面の Select → state → backend request/client slice の配線 test

## Boundary / Wire Contract

Required when the change touches JSON API, browser state, CSV, config, manifest, cache schema, Tauri command DTOs, generated bindings, report output, or DB-backed compatibility.

- producer: frontend の `per_page` パラメータ（`ListQuery`/`MovementQuery` DTO 経由）
- consumer: `inventory_service::list.rs` の `validate_page_params` / `list_movements` 内チェック
- wire type: `u32`（既存、変更なし）
- internal type: `u32`（既存、変更なし）
- precision/range: 許容範囲が `1..=100` から `1..=200` へ拡大（`MAX_PER_PAGE` 定数値のみ変更、DTO shape 不変）
- round-trip path: frontend Select → state → `commands.list*` invoke → BIZ validate → repo query → `PaginatedResult` 返却
- invalid input: `per_page > 200` は引き続き `BizError::ValidationFailed("ページパラメータが不正です")`（境界値のみ移動、エラー経路は不変）
- compatibility: 既存 `per_page <= 100` のリクエストは影響なし。201 以上を送っていた既存 test はすべて更新対象として本 packet に列挙済み

## Review Focus

- `rangeText` の文言差し替えが `Pagination`（下部）と `PaginationSummary`（上部）の両方に一貫して効くこと（共有関数のため二重実装なきこと）
- `.text-base` 単独 selector が意図せぬ要素に一致していないこと（L3-D5）
- `MAX_PER_PAGE` の 100→200 変更が影響する 5 関数すべてで既存 test が矛盾なく更新されていること（`list_receivings`/`list_returns`/`list_disposals` を含む）
- 6 画面の perPage 状態が「URL param か local state か」の裁定（L3-D3）どおりに実装されているか
- IntegrityCheckPage の Select が client-side slice の件数のみに作用し、余計な backend request を発火していないこと

## Spec Contract

Required for R3/R4.
Use at least one data row. Put concrete test names in the Test column when a regression test exists; use review/evidence labels only for plan-only checks.

Contract ID: SPEC-UILB-D3

- 8 一覧画面が共有 `LIST_PER_PAGE_OPTIONS`（50/100/200）を参照する `Select` を持ち、件数文言が新形に統一され、入出庫履歴・在庫変動履歴で perPage 200 が `ValidationFailed` を起こさない

## Trace Matrix

Required for R3/R4.

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-UILB-D3 | S1 | `search.test.ts`（更新）+ 新規 `list-per-page.test.ts` | 共有定数の 1 本化 | vitest |
| SPEC-UILB-D3 | S2/S3 | `Pagination.test.tsx` SC3a/SC3c、`ListShell.test.tsx`、`ProductListPage.test.tsx` | 文言・weight の全画面共通反映 | vitest |
| SPEC-UILB-D3 | S4〜S9 | 各 Page.test.tsx 新規 test | Select 配線・既定値 | vitest |
| SPEC-UILB-D3 | S10 | `list.rs`/`inventory_cmd.rs` 更新 3 + 新規 2 | 200 OK / 201 NG boundary | cargo test |
| SPEC-UILB-D3 | S11 | docs review（自動テストなし） | 文言 pin・上限値 doc 一致 | `rg` 完全一致 |

## Data Safety

Required for R3/R4.

- what must not be committed: なし（本 lane に synthetic/local-only データ生成はない）
- local-only paths: 該当なし
- synthetic-only paths: 該当なし

## Implementation Results

Fill after implementation.

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- S1〜S11 を実装し、共有 perPage 選択肢、6 画面の Select 配線、件数文言・weight、client-side slice、inventory BIZ の 200 OK / 201 NG 境界、Gated Amendment 1 を含む設計 doc を同期した。
- S13（Gated Amendment 2）を実装し、8 画面の表示件数変更後の先頭 scroll、操作ログ native 入力欄の control token、現行 operation type 5 種の registry と設計 doc・traceability を同期した。
- Draft PR: [#34](https://github.com/higekiri-w/inventory-system-desktop/pull/34)

## Review Response

Plan Review（独立 Sonnet subagent、fresh context、read-only、Coordinator が P1 を全件実測で裏取り）:
- round 1（対象 `fa15866`）: P1×3（catalog ⑩ の旧文言 hit 数 3 → 7 の過小申告 / AC10 の literal anchor が対象 doc の実表記と不一致 / `MAX_PER_PAGE` を記述する design doc が 3 → 10 箇所）+ P2×2 + P3×1 → 全件 accept。Coordinator が実測中に decision-log D-031 の例外条項（inventory_service 上限 100 据え置き）との衝突を発見し、D-081 新設を S10 に追加。是正 commit `8d8cad3`
- round 2（対象 `8d8cad3`）: round 1 の P1 3 件 fixed、P2-1 partial（catalog ⑩ `:612` `:638` の列挙漏れ）、新規 P1×1（`66-ui-stock-movements.md` の anchor がバッククォート込みで literal 不一致 = vacuous oracle）→ accept。是正 commit `d8c3afd`
- round 3（対象 `d8c3afd`）: 両是正を実測で確認、新規 P1 なし、P2×1（`既定 50` anchor の将来衝突の素地、記録のみ）→ **approve**（round 3/3）

2026-09-05: Plan Gate 収束（round 3/3、是正 commit = round 1 `8d8cad3` / round 2 `d8c3afd`）。`plan-draft -> plan-gate -> plan-approved -> implementing` を本 state-only commit で圧縮遷移（forward state-only 1/3）: plan-gate の証跡 = plan-first commit `fa15866`（Plans.md ④ active link 同乗）と上記 round 1〜3、plan-approved の証跡 = round 3 approve（最終 P1/P2 = 0、P2 記録のみ 1）、Plan Commit = plan-first commit `fa15866`。

2026-09-05: Writer（Codex）が worktree isolation で content commit 3 本（`b649257` / `b703f28` / `cae3d13`）を積み Draft PR #34 を作成（frontend gate / cargo test / `cargo check --release` / doc gate / check-workflow-git / L1 full RESULT=PASS / traceability `--check` clean。exact SHA と evidence は PR body を正とする。mutation 自己検証は発注書どおり不実施）。実装中の停止 2 回: (1) `66-ui-stock-movements.md:62` の固定 perPage 契約漏れ → Gated Amendment 1（`f56a5f7`）、(2) 在庫照会 test の無名 `combobox` query が新 Select と多重一致 → Coordinator が `DepartmentFilter` の `<label htmlFor>`「部門」を実読確認し、accessible name 付与のみ（expect 不変）を許可。変更 3 test は Writer 報告に列挙。

Final Review round 1（独立 Sonnet subagent、fresh context、隔離 worktree、対象 `f56a5f7..cae3d13`、AC を実行で再実測）: P1×1 / P2×0 / P3×1。
- P1（accept、手順違反）: PR body の `Reviewed Content HEAD` を Writer が Draft PR 作成時に `cae3d13` で先行記入（packet 側は `pending` のまま）。DEV_WORKFLOW `:84,102` どおり本 field は independent-review → human-confirm の state-only 遷移 commit で Coordinator が設定する。是正 = 本遷移 commit で packet 側を `cae3d13` に設定し、PR body の値はその時点で正となる（merge gate の三点一致には含まれない field、`:119,420`）。Writer 発注書に「PR body の Reviewed Content HEAD は pending で置く」を次回から明記する
- P3（記録のみ、backlog）: `docs/design-system/reference/mockup-d-lists.html:110` の `PRODUCT_PER_PAGE_OPTIONS` 表記が旧名のまま（S11 の mockup 対象は文言のみで定数名は scope 外）。D-080 で mockup は reference-only のため、次に mockup を触る lane で同期する
- 上記以外はすべて clean: owner 決定（文言 / 既定値 8 画面 / 選択肢 / `font-semibold` / `MAX_PER_PAGE` reject 維持）一致、Scope 逸脱なし、Writer による packet 改稿は Implementation Results 3 行のみ、test oracle は独立転記（Reviewer が mutation 2 体で非 vacuous を実証）、L3-D3 の URL / local state 分岐、page reset、query key の perPage 包含、D-081 / D-031 書式、docs 10 + 13 箇所の anchor、PR body 必須項目

Coordinator mutation 独立再実測（Sonnet 委譲、隔離 worktree `cae3d13` clean tree、`npm ci --ignore-scripts` 実 directory、Rust は target 共有）: X1 文言 1 文字 / X2 0 件文言 / X3 選択肢 `[50,100,300]` / X4 `font-semibold` 復活 / X5〜X10 6 画面の既定値改変 / X11 page reset 撤去 / X12 `MAX_PER_PAGE` 100 / X13 `MAX_PER_PAGE` 201 / X14 整合性チェックの固定 100 slice = **14 体全 kill、survivor 0**。各 mutant 後の `git status --short` 空を確認、teardown 完了。Rust の実 path は `src-tauri/src/biz/inventory_service/list.rs`（packet の `src-tauri/src/inventory_service/list.rs` 表記は module path 略記、機能影響なし）。

Review-only skipped because: Final Review を独立 Sonnet subagent（fresh context）が担い、Coordinator が mutation を別 context で独立再実測したため、R3 review-only sub-agent の役割は充足済み。
- Findings Freeze: frozen at Final Review round 1（P1/P2 = 0 after disposition）; post-freeze exceptions: none.

2026-09-05: `implementing -> local-verified -> independent-review -> human-confirm` を本 state-only commit で圧縮遷移（forward state-only 2/3）: local-verified の証跡 = Writer content commit の gate 群 + L1 full RESULT=PASS（PR body）、independent-review の証跡 = 上記 Final Review round 1（P1 disposition 後 P1/P2 = 0）+ mutation 14/14 kill、Reviewed Content HEAD = `cae3d13`、Amendments = `f56a5f7`。次 = owner Windows native L3（AC-L3-1〜4、介入 2/3）。

2026-09-05: owner Windows native L3 run 1（Reviewed Content HEAD `cae3d13`、介入 2/3 消費）= 操作ログ FAIL（表示件数変更後の viewport が途中位置 / native 入力欄と Select の枠の濃淡差 / 一括価格改定ログが「その他（product_price_revise）」fallback）。owner disposition = A（3 件すべて本 PR で修正）→ Gated Amendment 2（`cc7b0e8`）。`human-confirm -> implementing` を本 state-backtrack commit で 1 段戻す（Reviewed Content HEAD = pending へ、Amendments に `cc7b0e8` 追加）。次 = Writer 是正（S13 = A2-a〜c）→ Final Review round 2 + mutation X15/X16 → 遷移は S12 content commit に同乗 → owner L3 run 2（AC-L3-1〜5、介入 3/3、承認同乗）。

2026-09-05: Writer（Codex）是正 commit `d9c83a7`（A2-a〜c、test 先行: 実装前 9 件 fail → 実装後 pass、全 gate PASS、L1 full RESULT=PASS、evidence は PR body）。Final Review round 2（同 Reviewer、隔離 worktree、差分 `e06a847..d9c83a7` 21 file）= P1/P2/P3 = 0、approve（A2-a 8 画面の呼出し位置とページ送り不変 / A2-b 3 要素 / A2-c ラベルとカテゴリ順・74 doc 同期を実読、AC13〜15 + AC8/9/10c/12 を再実行）。Coordinator mutation 独立再実測（Sonnet 委譲、`d9c83a7`）: X15a〜h（8 画面の `scrollPageToTop` 撤去）/ X15i（ページ送りへの追加 = 対照 case の非 vacuous 確認）/ X16a（`product_price_revise` 削除）/ X16b（label 改変）= kill、**survivor 2** = X16c（「取引先管理」entry を配列末尾へ = カテゴリ順の契約に test なし）/ X17（native select の `border-input` 撤去 = SC9c が docs review のみ）。Coordinator 裁定 = P2 × 2、test 強化のみで kill する（production 不変）。

2026-09-05: Writer test 強化 commit `16c10be`（`OperationLogsPage.test.tsx` のみ: カテゴリ順 test に supplier 2 種別を含めて optgroup 配列を独立転記 literal で assert / SC9c を vitest 化し 3 要素の `border-input` `bg-control-surface` を assert、Writer 自己確認 2 体 fail）。Coordinator 独立再実測（`16c10be`）: X16c / X17 / X17b（開始日 input の `bg-control-surface` 撤去）= **3 体 kill、survivor 0**。期待値が production 定数の import で導出されていないことを実読確認。Lane 3 全体の mutation = 17 体、最終 survivor 0。

2026-09-05: `implementing -> local-verified -> independent-review -> human-confirm` を S12（Plans.md ④ 同期）の content commit に同乗させて遷移（state-only cap 温存、forward state-only は 2/3 のまま）: local-verified の証跡 = `d9c83a7` / `16c10be` の gate 群 + L1 full RESULT=PASS（PR body）、independent-review の証跡 = Final Review round 2 approve + mutation 17/17 kill、Reviewed Content HEAD = `16c10be`。次 = owner Windows native L3 run 2（AC-L3-1〜5、介入 3/3、PASS と承認を同一 message で）。

2026-09-05: owner Windows native L3 run 2（Reviewed Content HEAD `16c10be`、head `b411c76`、介入 3/3 消費）= 在庫照会 / 入出庫履歴 / 在庫変動履歴 PASS、操作ログの一括価格改定 badge PASS、**操作ログ AC-L3-5 (a) FAIL**（50 件 2 ページ目 → 100 件で viewport が途中に残る）で停止。棚卸し / 一括価格改定 / 整合性チェックは未確認。owner 所感の原文は `4c85b3c` / `338cfe0` で tracked 化（Plans ④ R2-1〜5）。機序は Gated Amendment 3（`ab3d12a`）に記録、再現 test で確定。`human-confirm -> implementing` を本 state-backtrack commit で 1 段戻す（Reviewed Content HEAD = pending、Amendments に `ab3d12a`）。次 = Sonnet Writer 是正（A3-a〜c）→ Final Review round 3 + mutation X18/X19 → 遷移は content commit 同乗 → owner L3 run 3（AC-L3-6 最小経路、介入 4/3 = 超過 1、PASS と承認を同一 message で）。

2026-09-05: Writer（Claude Sonnet 5 subagent、worktree isolation）是正 commit `0e9c60f`（A3-a: `page-scroll.ts` に TTL 1000 ms の flag、`app-router.ts` の独自 `onRendered` で consume → `main.scrollTop = 0` + 遅延再適用 skip / A3-b: `OperationLogsPage.scroll-restoration.test.tsx` 本番 `createAppRouter` + 実 routeTree、是正前 `expected 640 to be +0` で fail → 是正後 pass / SC10b 2 case）。全 gate PASS、L1 full RESULT=PASS、traceability `--check` OK（`UI-11c` 紐付け）。Final Review round 3（同 Reviewer、隔離 worktree、subscribe 順を router-core 実読、是正前 fail を再現、X18/X19 kill 確認）= P1×0 / P2×1 / P3×0 → approve。
- P2（accept、是正）: navigate を伴わない `scrollPageToTop()`（保存後の DSR-03 型、24 箇所中 14）で立った flag が TTL 内の別画面遷移でその画面の正規の「戻り」復元（DSR-17 ②）を潰す。Coordinator 裁定 = flag を立てた時の `pathname` に束縛（不一致は clear）。Writer 是正 commit `eada5fb`（`consumeForceScrollTop(currentPathname)`、`app-router.ts` は `latestLocation.pathname` を渡す、SC10b 2 case を同一 pathname に作り直し + 「pathname 不一致では別画面の戻り復元が守られる」case 追加、`createMemoryHistory` が `window.location` を同期しないため `onBeforeLoad` で `pushState` する test helper）。Reviewer 最終確認（差分限定、`latestLocation` の更新順を router-core `:2330,2397` で実読、X23 を再注入して新 case のみ fail）= approve、P3×1（guard assertion の値の clarity、記録のみ）
- Coordinator mutation 独立再実測（Sonnet 委譲）: `0e9c60f` で X18〜X22 = 5/5 kill、`eada5fb` で X18〜X25（X23 pathname 判定除去 / X24 pathname 未記録 / X25 固定 `/` を渡す）= **8/8 kill、survivor 0**。Lane 3 累計 25 体、最終 survivor 0。SC9a 単独では X20 を検出できない（`vi.mock` で本体を通らない）ことを実測で記録（Matrix SC10c）

2026-09-05: `implementing -> local-verified -> independent-review -> human-confirm` を S12（Plans.md ④ 同期）の content commit に同乗させて遷移（forward state-only 2/3 のまま）: local-verified の証跡 = `0e9c60f` / `eada5fb` の gate 群 + L1 full RESULT=PASS（PR body）、independent-review の証跡 = Final Review round 3 approve + 最終確認 approve + mutation 8/8 kill、Reviewed Content HEAD = `eada5fb`。次 = owner Windows native L3 run 3（AC-L3-6 最小経路: 操作ログの再現手順 1 点 + 棚卸し / 一括価格改定 / 整合性チェックの AC-L3-1、介入 4/3 = 超過 1、PASS と承認を同一 message で）。

2026-09-05: owner Windows native L3 run 3（HEAD `fa561e3`、内容 `eada5fb`、介入 4/3 = 超過 1）= 棚卸し / 一括価格改定 / 整合性チェックの AC-L3-1 PASS、**操作ログ AC-L3-6「なおった」= PASS**、所感の原文は `83a34e0`（R3-1〜4）で tracked 化。owner 承認「治ったなら承認しちゃおう、レーン3承認」（`5d3f98e` で原文保存、同 message で Lane 5 所感 R5-1〜5）。`human-confirm -> ready-hosted-final` を本 state-only commit で遷移（forward state-only 3/3、Ready 化と hosted final は Coordinator 代行）。Human Gate 完了。
