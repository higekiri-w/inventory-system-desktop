# Plan Packet: 価格改定支援 実装 B — UI-14 一括価格改定画面（route / navigation / 絞り込み + 取引先未設定含む / 新原価案導出 / 行単位確定 / 最近改定 badge）+ `ProductSearchQuery` 取引先 filter 拡張（issue #90、SPEC-PRV 実装 B）

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

If a state-only commit materializes multiple phases, list the complete adjacent forward sequence and the pre-existing evidence for every intermediate transition in an append-only review/evidence record. Recording compression never permits a gate skip.

- Phase: ready-hosted-final
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 325da8a
- Amendments: 119847e
- Coordinator: Fable
- Writer: Codex
- Plan Reviewer: Sonnet
- Final Reviewer: Sonnet
- Reviewed Content HEAD: cc60e26
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: Plan Gate 承認済み（2026-08-23、介入 1/3）→ human visual confirmation（Windows native L3 4 項目）PASS（item 4 の PLU 未反映件数は fixture 条件で観測不能だったため、実商品 1 件の売価改定 → ホーム N → N+1 → backup 復元の最小シナリオで追加確認 PASS）+ Ready 承認済み（2026-08-23、介入 3/3）→ hosted final（Rust / TS / bindings を含む non-doc change のため CI-TRIGGER-D1 の Ready / `synchronize` 経路で自動 run。予防的 `workflow_dispatch` はしない）→ 三点一致 → merge

## Owner Effort Budget

- 介入回数上限: 3（Plan Gate 承認 + L3 PASS/FAIL と Ready 承認 + 予備 1）
- 実働時間上限: 40分（L3 が 400 行級の反復操作を含むため実装 A より 10 分多く取る）
- relay 往復上限: 2（第 1 発注 = 実装本体、第 2 発注 = Final Review 是正 delta がある場合のみ）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

§5.5を使わないchangeは両方`none`のままにする。使う場合はtarget branch / PRへorder commitを混ぜず、artifact pathと専用remote order branch refを宣言する。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
(a) `ProductSearchQuery` に field を 2 件追加して `bindings.ts` を再生成する（既存 CMD `search_products` の wire contract 変更。UI-01a の呼出し元が無改変で型整合することが条件）、(b) operator 画面 UI-14 を新設し route / navigation を追加する（SPEC-PRV-D3〜D7、REQ-105 / REQ-106）、(c) `revise_product_price`（products 部分更新 + price_history + operation_logs の 1 TX、売価変更時の `plu_dirty` は POS 連携へ波及）を UI から初めて呼ぶ入口になる、(d) D-052 invalidation registry に C20 を新設する。DTO + operator workflow + DB 書込み入口のいずれも R3 条件（`docs/DEV_WORKFLOW.md` Risk Level）に該当する。

## Goal

Goal Invariant: 店主が値上げリスト（紙 / PDF）を手元に置き、UI-14 で取引先・部門・keyword で商品を絞り込み（取引先指定時は未設定商品も既定で含む）、行ごとに新売価を入れると新原価（案）が整数除算で提示され、行単位で `revise_product_price` に確定でき、数日またいでも DB の現価格と「最近改定」badge から再開できる画面を、実装 A の backend（`revise_product_price` / `create_supplier` / `list_price_history`）に `ProductSearchQuery` の取引先 filter 拡張だけを加えて main に入れる。

### 最小完了条件

- サイドバー「商品管理」に「一括価格改定」が表示され、`/products/price-revision` に到達できる（REQ-105 到達テスト PASS）。
- 取引先を選ぶと「取引先未設定の商品も含める」が既定 on で表示され、on では `supplier_id = X OR supplier_id IS NULL`、off では `supplier_id = X` の商品だけが一覧に出る。部門 / keyword / 廃番 / page / perPage は URL に保持され F5 で再現する。
- 行に新売価を入力すると新原価（案）が `floor(新売価 × 現原価 ÷ 現売価)` で埋まり（現売価 0 は現原価 fallback、現掛率は `—`）、「確定」で該当 1 商品だけが `commands.reviseProductPrice` に送られ、成功後は DB の現売価・現原価が再表示されて行入力が消え、本日改定した行に「最近改定」badge が出る。
- 取引先 filter の「新しい取引先を追加」で取引先を追加でき、追加後に filter で選択状態になる。取引先選択中は「未設定の商品にこの取引先を設定する」が既定 on で表示され、on の行確定で `assign_supplier_id` が渡る。
- 絞り込み枠と一覧の間に `画面を再読み込みすると、確定前に入力した新売価・新原価は失われます。1行ずつ確定してください。` が常時表示される。

### 失敗定義

- 取引先指定時に未設定商品が既定で漏れる / 「含める」off なのに未設定商品が混ざる / 取引先未指定なのに条件が付く（SPEC-PRV-D3 の破れ）。
- 新原価（案）が浮動小数や四捨五入で 1 円ずれる、現売価 0 で例外になる、掛率が小数 1 桁でない（SPEC-PRV-D4 の破れ）。
- 確定が複数行を送る / 失敗行の入力が消える / 成功後に旧価格が残る / 他行の表示が消える（SPEC-PRV-D5 / D7 の破れ）。
- 既存 `searchProducts` 呼出し元（UI-01a `search.ts` / `useProductList`）の test が退行する、`bindings.ts` の diff が追加以外を含む、`ProductBulkFilter` の母集団が変わる。
- 登録・生成義務（route file + `generate:routes` / `navigation.ts` entry / D-052 C20 の SSOT・oracle・件数・decision-log / `bindings.ts` / `90-traceability.md`）の漏れが L1 以降で顕在化する。

### 非目的

- 実装 C（入庫 cost_diffs、REQ-209）/ PDF 自動解析 / 新売価の自動提案 / 複数行の一括確定 / 確定前入力の draft 保存（77-ui §77.9）。
- 取引先の改名・統合・専用管理画面・約 80 社の事前投入（D-075）。
- `ProductBulkFilter` / `find_products_for_bulk_plu_target` への取引先 filter 追加、UI-01a への取引先 filter 追加（UI-01a は取引先 filter を持たず、一括 PLU 母集団も変えない）。
- 並べ替え UI（列ヘッダ sort）。URL `sort` は受理・正規化するが操作 UI は置かない（SPEC-PRVB-D5）。
- 検索結果 DTO への `last_price_changed_at` 付与（SPEC-PRVB-D2 で却下、L3 で遅延が問題なら別起票）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- IO `src-tauri/src/db/product_repo.rs`（SPEC-PRVB-D1、20-io §search_products を本 PR で改訂）:
  - `ProductSearchQuery` に `#[serde(default)] pub supplier_id: Option<i64>` と `#[serde(default)] pub include_unassigned: bool` を追加（既存 `plu` と同じ `#[serde(default)]` 慣行、TS 側は optional）。
  - WHERE 句: `supplier_id = Some(X)` かつ `include_unassigned = true` → `(p.supplier_id = ?N OR p.supplier_id IS NULL)`、`Some(X)` かつ `false` → `p.supplier_id = ?N`、`None` → 条件なし（`include_unassigned` は無視）。COUNT と SELECT は既存どおり同じ `conditions` を共有する。
  - `ProductBulkFilter` / `find_products_for_bulk_plu_target` / ORDER BY / paging は変えない。
  - Rust test 4 件（Ledger）を `product_repo.rs` の既存 `test_search_products_req103_*` と同じ fixture 様式で追加。
- docs: `docs/function-design/20-io-product-repo.md` `search_products` の `ProductSearchQuery` 構造体 bullet に 2 field を追加し、処理ステップ 2 の WHERE 句に supplier 条件を 1 行追加。`docs/function-design/40-cmd-product.md` `search_products コマンド` に「`supplier_id` / `include_unassigned` は UI-14 の取引先 filter（SPEC-PRV-D3）として IO-01 の WHERE へ渡す」を 1 文追記。`docs/function-design/30-biz-product-service.md` §4.6 は wrapper（追加の業務ロジックなし）で変更なし。`docs/function-design/77-ui-bulk-price-revision.md` は §77.3 の tree に `priceRevisionSearch.ts` / `lib/price-revision-math.ts` を追加、§77.9 の「UI-14 の実装と Windows native L3 は後続実装 PR B で扱う」bullet を削除、§77.10 に実装 B 行を追加（updated in this PR）。
- 生成物: `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成し commit（`ProductSearchQuery` 型に 2 field 追加、既存 CMD / 型は不変）。`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（RTL / navigation test の `REQ-105` / `REQ-106` / `UI-14` 参照を反映）。`npm run generate:routes` は `src/routeTree.gen.ts`（gitignore）を再生成するだけで commit 対象なし。
- route `src/routes/products/price-revision.tsx`: `createFileRoute("/products/price-revision")` + `validateSearch: priceRevisionSearchSchema` + `PriceRevisionPage`（`src/routes/products/index.tsx` と同型、eager import）。
- URL state `src/features/products/priceRevisionSearch.ts`（+ `.test.ts`）: 77-ui §77.4 の 8 param（`q` / `supplier` / `includeUnassigned` / `dept` / `discontinued` / `sort` / `page` / `perPage`）の zod schema・`normalizePriceRevisionSearch`・`buildPriceRevisionSearchQuery(search): ProductSearchQuery`・`updatePriceRevisionSearch(patch)`。`PRODUCT_PER_PAGE_OPTIONS` / `PRODUCT_SORT_OPTIONS` 等は `search.ts` から import して重複定義しない（SPEC-PRVB-D5）。
- 導出 `src/features/products/lib/price-revision-math.ts`（+ `.test.ts`）: `deriveProposedCost(newSelling, cost, selling)` / `formatMarkupRate(cost, selling)` / `isRevisedToday(changedAt, todayYmd)` の pure 関数（SPEC-PRVB-D3）。UI component は必ずこの module 経由で導出し、component 内に算式を書かない。
- hooks `src/features/products/hooks/usePriceRevisionList.ts`（+ `.test.tsx`）: `commands.searchProducts(buildPriceRevisionSearchQuery(search))` / `commands.listSuppliers()` / `commands.listDepartments()` を `useQuery`（`queryKeys.priceRevision.search(normalized)` / `.suppliers()` / `.departments()`）で取得し、page 内の各行について `commands.listPriceHistory(code, 1)` を `useQueries`（`queryKeys.priceRevision.history(code)`）で取得して `latestChangedAt` を行に結び付ける（SPEC-PRVB-D2）。`src/features/products/hooks/useReviseProductPrice.ts`: `useMutation` で `commands.reviseProductPrice(input)` を呼び、成功時に `invalidateByContract(queryClient, invalidationContract.productPriceRevise(productCode))`（D-052 C20、SPEC-PRVB-D6）。
- components `src/features/products/components/PriceRevisionFilters.tsx` / `PriceRevisionTable.tsx` / `CreateSupplierDialog.tsx` と page `src/features/products/PriceRevisionPage.tsx`（+ `PriceRevisionPage.test.tsx`）: 77-ui §77.3 / §77.6 / §77.7 の構成・列・文言・行状態（SPEC-PRVB-D4 / D7 / D8 / D9）。共有 `PageHeader` / `SearchBar` / `DepartmentFilter` / `EmptyState` / `ListSkeleton` / `ProductPagination` / `Badge` を流用する。
- 登録: `src/lib/query-keys.ts` に `priceRevision` namespace（`root` / `search` / `suppliers` / `departments` / `history`）、`src/lib/invalidation-contract.ts` に C20 `productPriceRevise(productCode)`、`src/test/invalidation-oracle.ts` に C20 の独立転記、`src/lib/invalidation-contract.meta.test.ts` の entry 件数 19 → 20（C19 追加時 PR #86 と同型の契約更新、既存 entry の assertion は不変）、`invalidation-contract.ts` L7 と `invalidation-oracle.ts` L8 のヘッダコメント「D-052-C1〜C19」を「C1〜C20」に更新、`docs/decision-log.md` D-052 の Contract 行に C20 と件数（19 → 20 entry / 22 → 23 handler）、`docs/UI_TECH_STACK.md` §2.5 の同件数と「PLU slot summary は …」文に C20 非該当の注記は不要（C20 は slot summary を含まない）。`src/config/navigation.ts` 商品管理に `ui-14`（label / title `一括価格改定`、`to: "/products/price-revision"`、`status: "active"`、52-ui §52.3 の表順）+ `src/config/navigation.test.ts` に `test_navigation_req105_ui14_active_at_products_price_revision`。
- テスト: Ledger の Rust 4 件 / RTL（`usePriceRevisionList.test.tsx` / `PriceRevisionPage.test.tsx` / `navigation.test.ts` = archived Matrix 予約名、`priceRevisionSearch.test.ts` / `price-revision-math.test.ts` = 本 packet で追加）/ `invalidation-contract.meta.test.ts` 件数更新。新原価案・掛率・本日判定の oracle は production 定数・関数を共有せず literal を独立転記する。既存 test は改変しない（例外 = meta test の件数 literal 1 箇所の契約更新 / gated amendment 1 の `src/hooks/unsaved-changes-guard-sweep.test.ts` `EXCLUDED_PAGES` への `PriceRevisionPage` 1 entry 追加 / Rust 既存 test の `ProductSearchQuery` struct literal への新 field 2 行追加〈assertion 不変〉）。新設する FE test file（`priceRevisionSearch.test.ts` / `price-revision-math.test.ts` / `usePriceRevisionList.test.tsx` / `PriceRevisionPage.test.tsx`）は describe / it 名に `REQ-105` または `UI-14` を 1 箇所以上含め、`generate_traceability` の T4 FE baseline（`FE_UNREFERENCED_BASELINE = 22`、REQ / UI ID 未参照 FE test file 数。増減どちらも `--check` ERROR）を変えない。
- Writer 完了条件に `cd src-tauri && cargo check --release`（Human Gate が L3 を含むため、CI gate ではない）。
- gated amendment 1（2026-08-23、Review Response 参照）: (i) `src/hooks/unsaved-changes-guard-sweep.test.ts` の `EXCLUDED_PAGES` に `PriceRevisionPage`（`src/features/products/PriceRevisionPage.tsx`）を 1 entry 追加し、`docs/function-design/77-ui-bulk-price-revision.md` §77.6「行確定と中断・再開」に「共通離脱ガード（UI_TECH_STACK §6.11 `useUnsavedChangesWarning`）は UI-USW-D3 (c)〈行単位の即時 DB 保存、棚卸しと同型〉により適用せず、常時文言で代替する」を 1 文追記（updated in this PR）。(ii) 共有 `src/components/patterns/ListSkeleton.tsx`（`docs/design-system/04-backbone.md` 原則 11 の共通 `ListSkeleton`、`src/components/ui/skeleton.tsx` を束ねる列構造維持の読込み表示）を新設し、`ListSkeleton.test.tsx`（描画と `aria-busy` / role の最小 assertion）と `docs/design-system/02-component-catalog.md` への 1 行（所在 / 用途 / props）を追加。(iii) Rust 既存 test の `ProductSearchQuery` struct literal（`product_repo.rs` / `product_service.rs` の fixture）に `supplier_id: None, include_unassigned: false` を加えるのは assertion 不変の機械的追従として許容。

## Non-scope

- `docs/function-design/61-ui-receiving.md` / `create_receiving` / `ReceivingCreateResult.cost_diffs`（実装 C）。
- `products` / `suppliers` / `price_history` の schema 変更、index 追加（`products.supplier_id` に index は無いが、初年度の商品数規模で `未実測` のまま追加しない）。
- `ProductBulkFilter` / `find_products_for_bulk_plu_target` / UI-01a の filter / `search.ts` の既存 schema（`buildProductSearchQuery` は Contract Probe 1 の結果次第で `include_unassigned` 既定値を明示する変更のみ許容）。
- `PriceHistorySection`（UI-01b）/ `ProductForm` の取引先 inline 追加（実装 A 済み、UI-14 の dialog は別 component）。
- 列ヘッダ sort UI、perPage 既定値の変更、`PAGINATION_MAX_PER_PAGE` の変更。
- operation_logs の UI-14 用 query / 表示（E1 除外）。

## Acceptance Criteria

- `cd src-tauri && cargo test` が PASS し、Ledger の Rust test 4 名が `src-tauri/src/db/product_repo.rs` に各 1 回存在する（`rg -c "fn <name>"` で各 1）。
- `cd src-tauri && cargo run --bin generate_bindings` 実行後に `git diff --stat src/lib/bindings.ts` が空（commit 済みと一致）で、`rg -c "supplier_id" src/lib/bindings.ts` が実装前より増え、`ProductSearchQuery` 型に `supplier_id` と `include_unassigned` の行が各 1 行ある（`rg -n "include_unassigned" src/lib/bindings.ts` が 1 以上）。`bindings.ts` の diff は `ProductSearchQuery` 型への追加のみ。
- `cd src-tauri && cargo test --test design_compliance_test` が PASS（fn シグネチャ不変、allowlist 追加なし）。
- `npm run generate:routes && npm run typecheck && npm run lint && npm run format:check && npm test` が PASS し、Matrix 記載の RTL / unit case が各 file に存在する。`src/features/products/search.ts` と `src/features/products/hooks/useProductList.ts` の既存 test は無改変で PASS。
- `rg -c '"ui-14"' src/config/navigation.ts` = 1、`rg -c "test_navigation_req105_ui14_active_at_products_price_revision" src/config/navigation.test.ts` = 1、`rg -c 'status: "pending"' src/config/navigation.ts` = 0（既存 `test_navigation_all_items_no_pending_status` を維持）。
- `rg -c "productPriceRevise" src/lib/invalidation-contract.ts src/test/invalidation-oracle.ts src/features/products/hooks/useReviseProductPrice.ts` が各 1 以上、`rg -c "toHaveLength\(20\)" src/lib/invalidation-contract.meta.test.ts` = 1、`rg -c "C20" docs/decision-log.md docs/UI_TECH_STACK.md` が各 1 以上。
- `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- `bash scripts/doc-consistency-check.sh` exit 0（既存 WARN を除く）、`bash scripts/doc-consistency-check.sh --target plan` 全チェック通過。
- `bash scripts/local-ci.sh full` RESULT=PASS / END_TREE_STATE=CLEAN（content candidate と Ready exact-HEAD の 2 回、evidence は PR body。evidence log は先頭 `HEAD_SHA` と末尾 `END_HEAD_SHA` / `RESULT` / `MERGE_EVIDENCE_VALID` で読む）。
- human visual confirmation（Windows native L3）の結果が PR body の `Human Gate` 欄に `L3: PASS` または `L3: FAIL` の文字列で記録されている（`gh pr view --json body` で確認）。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-105 / REQ-106、`docs/spec/requirements-coverage.md` SP-102-08
- Architecture: `docs/ARCHITECTURE.md`（UI -> CMD -> BIZ -> IO）
- Function / command / DTO: `docs/function-design/20-io-product-repo.md` `search_products`（`ProductSearchQuery`、本 PR で改訂）、`docs/function-design/30-biz-product-service.md` §4.4.1 `revise_product_price` / §4.6 / §4.7.2 / §4.7.3、`docs/function-design/40-cmd-product.md` `search_products` / `revise_product_price` / `create_supplier` / `list_price_history` / `list_suppliers`
- DB: `docs/db-design/master-tables.md` §3（suppliers / products.supplier_id の漸進補完）、`docs/db-design/tracking-system-tables.md` §15 price_history
- Screen / UI: `docs/function-design/77-ui-bulk-price-revision.md` §77.1〜§77.8（SPEC-PRV-D3〜D7）、`docs/function-design/52-ui-shared-layout.md` §52.3 UI-14 行、`docs/function-design/50-ui-product-list.md` §50.4（UI-01a の URL state 先例。query key 先例は `src/lib/query-keys.ts` の `productList` namespace）、`docs/UI_TECH_STACK.md` §2.5（D-052 導出原則 / E1〜E6）
- Decision log / ADR: `docs/decision-log.md` D-031（`PAGINATION_MAX_PER_PAGE = 200`）/ D-052（invalidation SSOT）/ D-075、archived design packet `docs/archive/plans/2026-08-22-price-revision-design.md` SPEC-PRV-D3 / D4 / D6 / D7、archived Matrix `docs/archive/plans/test-matrices/2026-08-22-price-revision-design.md`「実装 PR への予約 → 実装 B」、archived 実装 A packet `docs/archive/plans/2026-08-22-price-revision-impl-a.md` Boundary / Wire Contract（整数除算 overflow 境界の B への defer）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 20-io `search_products`（`ProductSearchQuery` 2 field + WHERE）、30-biz §4.6（wrapper、変更なし）、§4.4.1 / §4.7.2 / §4.7.3（実装 A 済み） | updated in this PR（20-io）+ existing sufficient（30-biz） |
| Command / DTO / generated binding / wire shape | 40-cmd `search_products` 1 文、`bindings.ts`（生成） | updated in this PR（40-cmd 1 文 + bindings 再生成） |
| DB / transaction / audit / rollback / migration | master-tables §3、tracking-system-tables §15、schema_v1.rs 既存（`products.supplier_id` 列あり、migration なし） | existing sufficient |
| Screen / UI / route state / Japanese wording | 77-ui §77.3〜§77.7、52-ui §52.3 UI-14 行、UI_TECH_STACK §6.11 UI-USW-D3、design-system 04-backbone 原則 11 / 02-component-catalog | existing sufficient + updated in this PR（77-ui §77.3 tree 2 file / §77.9 bullet 削除 / §77.10 行追加 / §77.6 離脱ガード非適用 1 文〈gated amendment 1〉、02-component-catalog に ListSkeleton 1 行〈gated amendment 1〉） |
| CSV / TSV / report / import / export format | 該当なし（PLU TSV 契約は不変、`plu_dirty` は既存経路） | existing sufficient |
| Durable decision / ADR | D-052 Contract 行（C20 追加）、SPEC-PRV-D3〜D7（source docs 転記済み）、SPEC-PRVB-D1〜D9（本 packet） | updated in this PR（decision-log D-052 / UI_TECH_STACK §2.5 件数）+ existing sufficient |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| Tauri command | 該当なし（新規 CMD なし。`search_products` の DTO field 追加のみ → `cd src-tauri && cargo run --bin generate_bindings` で `bindings.ts` 再生成 + commit） |
| function-design doc 新設 | 該当なし（77-ui は PR #93 で新設済み。本 PR は 20-io / 40-cmd / 77-ui の改訂のみ） |
| source / workflow doc 新設・改名 | 該当なし |
| AGENT_OPERATING_MANUAL §5.5 consultation relay 使用 | 該当なし |
| REQ coverage 追加（テスト追加） | RTL / navigation test が `REQ-105` / `REQ-106` / `UI-14` を参照するため `cd src-tauri && cargo run --bin generate_traceability` で `90-traceability.md` 再生成（手動編集禁止） |
| route 新設 | `src/routes/products/price-revision.tsx` 追加 + `npm run generate:routes`（`routeTree.gen.ts` は gitignore、commit 不要。`npm test` / `typecheck` の pre script が再生成する） |
| operator 画面新設 | `src/config/navigation.ts` 商品管理に `ui-14` entry（52-ui §52.3 の表順）+ `navigation.test.ts` 到達テスト。52-ui §52.3 の行は既存で変更なし |
| D-052 mutation entry 新設（C20） | `invalidation-contract.ts` + `src/test/invalidation-oracle.ts`（独立転記）+ `invalidation-contract.meta.test.ts` 件数 20 + `decision-log.md` D-052 Contract 行 + `UI_TECH_STACK.md` §2.5 件数。番号 C20 は本 packet が予約し、実装 C は C21 以降（または既存 C4 の流用）を使う |
| query key 新設 | `src/lib/query-keys.ts` `priceRevision` namespace（literal key 直書き禁止） |

## Design Decisions（packet-local、SPEC-PRVB-D1〜D9）

### SPEC-PRVB-D1: `ProductSearchQuery` 取引先 filter の契約

- `supplier_id: Option<i64>`（`#[serde(default)]`）と `include_unassigned: bool`（`#[serde(default)]`、既定 false）を追加する。`Some(X)` + `true` → `(p.supplier_id = ? OR p.supplier_id IS NULL)`、`Some(X)` + `false` → `p.supplier_id = ?`、`None` → 条件なし（flag は無視）。他条件とは AND、COUNT と SELECT で同一 WHERE。
- 理由: 77-ui §77.4 / §77.6 の抽出条件をそのまま IO に置き、BIZ §4.6 wrapper と CMD には判断を持たせない。`#[serde(default)]` は既存 `plu` と同じ慣行で、TS 側 optional となり UI-01a の `buildProductSearchQuery` を無改変で通す（Contract Probe 1）。
- 却下: 新 CMD `search_products_for_price_revision` の新設（同じ一覧取得の二重契約）/ `ProductBulkFilter` への同時追加（UI-01a に取引先 filter が無く、一括 PLU 母集団を変える理由がない）/ 未設定商品の抽出を UI 側で 2 回検索して結合（paging と total_count が壊れる）。
- 転記先: 20-io `search_products`（field bullet 2 行 + 処理ステップ 2 の 1 行）、40-cmd `search_products` 1 文（本 PR）。

### SPEC-PRVB-D2: 「最近改定」の導出方式

- page 内の各行について `commands.listPriceHistory(product_code, 1)` を `useQueries` で取得し、先頭要素の `changed_at` 先頭 10 文字（`YYYY-MM-DD`。`insert_price_history` は `chrono::Local` の `%Y-%m-%dT%H:%M:%S` で書く）と端末ローカル日付 `new Date().toLocaleDateString("sv-SE")`（`useYesterdayDate.ts` / `date-nav.ts` の既存慣行）の文字列一致で「本日」を判定する。`Date` parse はしない。履歴取得中 / 失敗 / 0 件の行は badge 非表示で、一覧本体は隠さない。
- 理由: 77-ui §77.5 / §77.6 は「直近 changed_at から導出、別 field を永続化しない」を契約にしており、実装 A の `list_price_history` がそのまま使える。1 page 最大 200 行 = 最大 200 回の local IPC で、体感遅延は `未実測`（L3 item 3 で実測し PR body に記録）。
- 却下: 検索結果 DTO に `last_price_changed_at` を付与する IO / DTO 拡張（B の backend 変更を filter のみに絞る。L3 で遅延が問題なら別起票）/ 確定した行だけ session 内で badge を出す（数日またぎの再開点にならない、D7 違反）。
- 転記先: なし（77-ui §77.5 の範囲内の実装方式。Ledger の hook test で契約化）。

### SPEC-PRVB-D3: 新原価（案）/ 現掛率 / 本日判定の pure module 隔離と算式

- `src/features/products/lib/price-revision-math.ts` に `deriveProposedCost(newSelling, cost, selling) = selling === 0 ? cost : Math.floor((newSelling * cost) / selling)`、`formatMarkupRate(cost, selling) = selling === 0 ? "—" : (Math.round((cost * 1000) / selling) / 10).toFixed(1)`、`isRevisedToday(changedAt, todayYmd) = changedAt?.slice(0, 10) === todayYmd` を置く。入力は 0 以上の整数（UI の validation 後）。
- 理由: 77-ui §77.6 の整数除算・小数 1 桁四捨五入・現売価 0 の `—` / fallback を component から切り離し、独立 oracle の unit test と mutation 注入を可能にする。`newSelling * cost` は円が 10^7 未満（`未実測` 前提、実装 A packet と同じ）なら 10^14 未満で JS safe integer（2^53 ≈ 9.007×10^15）内に収まり、`Math.floor` は非負整数の整数除算と一致する。実装 A から defer された overflow 境界は `9_999_999 × 9_999_999 ÷ 1 = 99_999_980_000_001` の exact 計算 test で消化する。
- 却下: `toFixed` だけで掛率を丸める（浮動小数の 2 進表現で x.x5 が切り捨てられ得る）/ BigInt（値域が不要、表示経路が複雑化）。
- 転記先: なし（77-ui §77.6 の算式の実装配置。Ledger の unit test で契約化）。

### SPEC-PRVB-D4: 行状態と確定の契約

- 行状態は `idle`（入力なし）/ `editing`（新売価または新原価案に入力あり）/ `pending`（送信中: 同行の入力と確定を disabled、他行は操作可）/ `error`（`確定できませんでした` + error 内容 + `再試行`、入力保持）。`確定` button は新売価が 0 以上の整数として入力済みのときだけ有効。負値 / 非整数 / 空は行内 field error で CMD を呼ばない。
- 新原価（案）は新売価の入力後に `deriveProposedCost` で初期化し、利用者が新原価案を手で編集していない間は新売価の変更に追従して再導出、手で編集した後は新売価を変えても上書きしない（行ごとの `costTouched` flag）。
- `PriceRevisionInput` は `{ product_code, new_selling_price, new_cost_price, assign_supplier_id }` で、`assign_supplier_id` は取引先 filter 選択中かつ「未設定の商品にこの取引先を設定する」on のとき選択中 supplier_id、それ以外 `null`。成功時は C20 を invalidate して一覧を再取得し、該当行の入力と状態を消し、同行の `listPriceHistory(code, 1)` も再取得して badge を更新する。`PriceRevisionResult.changed = false`（no-op）でも成功扱いで入力を消す。
- 理由: 77-ui §77.6「行確定と中断・再開」の契約を state で固定し、Writer と Final Reviewer の解釈を揃える。
- 却下: 新売価変更で常に新原価案を上書き（利用者の手入力が消える）/ no-op をエラー表示（実装 A SPEC-PRVA-D4 で no-op は正常）。
- 転記先: なし（77-ui §77.6 の範囲内。RTL で契約化）。

### SPEC-PRVB-D5: URL state の正規化

- zod schema は 77-ui §77.4 の 8 param。`supplier` 未指定なら `includeUnassigned` は URL から落とし query は `supplier_id: null, include_unassigned: false`、`supplier` 指定で `includeUnassigned` 欠落なら `true`（既定 on）。`sort` は `PRODUCT_SORT_OPTIONS` の値を受理し、無効値と欠落は `product_code` 昇順（操作 UI は置かない）。`dept` / `discontinued` / `q` / `page` / `perPage` は UI-01a `search.ts` と同じ正規化（`PRODUCT_PER_PAGE_OPTIONS` 50 / 100 / 200、無効は既定 50）。filter 変更時は `page = 1`。「未設定の商品にこの取引先を設定する」toggle は URL に載せず component state とし、`supplier` が変わるたび既定 on に戻す。
- 理由: 77-ui §77.4「無効な search 値は既定へ回復し、F5 / 再訪時に確定済みの filter 状態を再現する」「確定前の行入力は URL に載せない」を満たし、UI-01a の schema 定数を共有して上限を揃える。
- 却下: `search.ts` の `productListSearchSchema` を直接拡張（UI-01a の URL に UI-14 専用 param が漏れる）。
- 転記先: なし（77-ui §77.4 の範囲内。`priceRevisionSearch.test.ts` で契約化）。

### SPEC-PRVB-D6: D-052 C20 `productPriceRevise(productCode)` の key 集合

- `[queryKeys.productList.root(), queryKeys.productForm.product(productCode), queryKeys.pluDirty(), queryKeys.priceRevision.root()]`。
- 理由（UI_TECH_STACK §2.5 の table.column 導出）: `revise_product_price` が書く列は `products.selling_price` / `cost_price` / `updated_at` / 条件付き `plu_dirty` / 条件付き `supplier_id` と `price_history` 行。UI-01a 一覧（価格 / PLU 状態列）、UI-01b 商品 form（価格・取引先、再 mount で履歴セクションも再取得）、ホーム未反映件数（`plu_dirty`）、UI-14 自身（一覧 + 行履歴）が読む。`lowStock` / `stockInquiryRoot` / `stockMovements.root` は、価格列については商品 master 表示列の JOIN stale として E3 除外、取引先（`supplier_id`）については当該 query の consumer（`SummaryCards` / stock-inquiry / `StockMovementsPage`）が supplier を読まないため invalidate 対象に該当しない（E3 とは別の「書込み列を読む consumer なし」の理由）、`pluSlotSummary` は `get_plu_slot_summary` が plu_slots 状態と snapshot 設定だけを読み `plu_dirty` 非依存（Contract Probe 2）、operation_logs は E1 除外。
- 却下: C2 `productUpdate` の集合をそのまま流用（E3 で除外すべき 3 key を含み「過剰 invalidation は契約違反」に触れる）/ C20 を置かず `refetch()` のみ（UI-01a / UI-01b / ホームが stale のまま）。
- 転記先: decision-log D-052 Contract 行 + UI_TECH_STACK §2.5 件数（本 PR）。

### SPEC-PRVB-D7: 取引先 / 部門候補と「新しい取引先を追加」

- 候補は `useQuery`（`queryKeys.priceRevision.suppliers()` / `.departments()`）。`CreateSupplierDialog` は入力 name を trim、空白のみは field error で CMD を呼ばない、`commands.createSupplier(trimmed)` 成功後に suppliers query を `refetch()` し、返却 `Supplier.id` を URL `supplier` に set（filter 選択状態、`includeUnassigned` は既定 on）。失敗時は dialog を閉じず入力保持 + 再試行可能な Alert（77-ui §77.7）。取引先候補の取得失敗は filter 枠内の inline error + 再試行で、商品一覧は隠さない。
- 理由: 実装 A の UI-01b inline 追加（`ProductForm` が `createSupplier` → `listSuppliers` 再取得）と同じ往復順序を UI-14 でも保ち、D-052-S1 static test が拒否する直接 `invalidateQueries` を使わない。
- 転記先: なし（77-ui §77.6 / §77.7 の範囲内。RTL で契約化）。

### SPEC-PRVB-D8: perPage と 400 行級 L3 の関係

- `perPage` は 50 / 100 / 200（UI-01a と同じ `PRODUCT_PER_PAGE_OPTIONS`）で、IO の `PAGINATION_MAX_PER_PAGE = 200`（D-031）が上限。400 行級の値上げリストは perPage 200 × 2 page、または取引先 / 部門 / keyword で 1 page に収める運用とする。L3 checklist は perPage 200 で 2 page を往復し、page 遷移時に確定前の入力が消える（77-ui §77.6 の常時文言どおり）ことも確認する。
- 理由: 77-ui §77.4「UI-01a と同じ既存上限内で送る」を守り、上限変更や無限 scroll を持ち込まない。
- 却下: UI-14 だけ perPage 400 を許容（D-031 の上限と IO clamp に反する）。
- 転記先: なし（L3 checklist に反映）。

### SPEC-PRVB-D9: Empty / Error の導線

- filter なし（`q` 空、`supplier` / `dept` 未指定、`discontinued` false）で 0 件 → `EmptyState` に title「該当する商品がありません」（UI-01a `ProductListPage` の既存 `EmptyState` title と同文言、新文言を増やさない）+ 商品一覧（`/products`）への link（77-ui §77.7「商品一覧への導線」は文言を規定しないため、先例文言 + link で固定する）。filter ありで 0 件 → `条件に一致する商品がありません` + `絞り込みを解除`（全 param を既定へ戻し page 1）。一覧取得失敗 → ページ上部 Alert に日本語説明 + `再試行`（query `refetch`）。いずれも色だけで状態を表さない。
- 転記先: なし（77-ui §77.7 の範囲内。RTL で契約化）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-105 | 77-ui §77.4 / §77.6 絞り込み、20-io `search_products`、40-cmd `search_products` | SPEC-PRV-D3 / SPEC-PRVB-D1 / D5 | supplier 紐付けが疎な初年度に未設定商品を既定で含める。UI 2 回検索結合は却下 | product_repo / 20-io / 40-cmd / priceRevisionSearch / usePriceRevisionList / PriceRevisionFilters | Rust 4 test / `priceRevisionSearch.test.ts` / hook test / RTL toggle |
| REQ-105 | 77-ui §77.6 一覧列と価格入力 | SPEC-PRV-D4 / SPEC-PRVB-D3 / D4 | 掛率は導出のみ、新原価案は整数除算、新売価は手入力。浮動小数 / 自動提案は却下 | price-revision-math / PriceRevisionTable | `price-revision-math.test.ts` / RTL 導出 |
| REQ-105 | 77-ui §77.6 行確定、30-biz §4.4.1、40-cmd `revise_product_price` | SPEC-PRV-D5 / SPEC-PRVB-D4 / D6 | 行単位確定 + 部分失敗の行内復旧。一括確定は却下 | useReviseProductPrice / PriceRevisionTable / invalidation-contract C20 | RTL 確定 / 失敗 / C20 oracle |
| REQ-106 | 77-ui §77.6 取引先の漸進補完、30-biz §4.7.2、40-cmd `create_supplier` | SPEC-PRV-D6 / SPEC-PRVB-D7 | filter 内で取引先追加 + 未設定商品だけへ紐付け toggle。80 社一括投入は却下 | CreateSupplierDialog / PriceRevisionFilters / useReviseProductPrice | RTL 取引先追加 / assign_supplier_id |
| REQ-105 | 77-ui §77.6 中断・再開、§77.5 `listPriceHistory` | SPEC-PRV-D7 / SPEC-PRVB-D2 | DB + price_history からの復元。draft 保存 / DTO 拡張は却下 | usePriceRevisionList（useQueries）/ PriceRevisionTable badge | hook test / RTL badge / 文言 |
| REQ-105 | 52-ui §52.3、77-ui §77.8 navigation | — | 到達導線の登録義務 | navigation.ts / route file | `test_navigation_req105_ui14_active_at_products_price_revision` |
| REQ-105 | 77-ui §77.7、§77.4 perPage、D-031 | SPEC-PRVB-D8 / D9 | 既存上限内 / Empty・Error 導線 | PriceRevisionPage / ProductPagination | RTL Empty / Error + L3 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（SPEC-PRV-D3〜D7 は 77-ui に転記済み。`ProductSearchQuery` の取引先 filter だけが 20-io / 40-cmd に未記載で、本 PR で改訂する = Design Readiness 注意の解消）。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: SPEC-PRVB-D1（20-io / 40-cmd）、SPEC-PRVB-D6（decision-log D-052 / UI_TECH_STACK §2.5）を本 PR で転記。SPEC-PRVB-D2 / D3 / D4 / D5 / D7 / D8 / D9 は 77-ui 既存文言の範囲内の実装方式固定で転記不要（Ledger の test で契約化）。
- Assumptions and constraints: schema 変更なし / `products.supplier_id` 列は既存（schema_v1.rs）/ 円は 10^7 未満（`未実測` 前提、実装 A と同じ）/ `changed_at` は `chrono::Local` の `%Y-%m-%dT%H:%M:%S`（既存 `insert_price_history`）/ 1 page 最大 200 行の行別 `listPriceHistory` 呼出しの体感遅延は `未実測`（L3 で実測）/ `routeTree.gen.ts` は gitignore。
- Deferred design gaps, risk, and follow-up target: 実装 C（cost_diffs）/ `last_price_changed_at` の DTO 付与（L3 で遅延が問題なら別起票）/ 列ヘッダ sort UI / `products.supplier_id` index（規模 `未実測`）/ 取引先の改名・統合（別起票）。
- Test Design Matrix can cite design decision IDs or source doc sections: yes（SPEC-PRV-D3〜D7、SPEC-PRVB-D1〜D9、D-031、D-052、UI_TECH_STACK E1 / E3）。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 絶対保証 = 取引先指定時の未設定含む既定 on / 新原価案は整数除算のみ / 確定は 1 行 1 request / 失敗行の入力保持 / 既存 `searchProducts` 呼出し元の無改変整合。escape hatch なし。`bindings.ts` は `ProductSearchQuery` 型への field 追加のみ、既存 CMD の入出力不変。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | POS 連携は `plu_dirty` 既存経路のみ（UI から `revise_product_price` を呼ぶ入口が増えるだけ）。新 adapter なし | — |
| Fact check / design decision split | 事実 = 77-ui / 20-io / 40-cmd / 実装 A の bindings。判断 = SPEC-PRVB-D1〜D9 | 本 packet |
| Lifecycle / retry | 一覧取得失敗 = Alert + 再試行、行確定失敗 = 行内再試行 + 入力保持、取引先追加失敗 = dialog 保持 | Matrix State Lifecycle |
| Operator workflow | サイドバー → UI-14 → 取引先 / keyword 絞り込み → 旧売価突合 → 新売価入力 → 新原価案確認 → 確定 → 「最近改定」で再開点確認 | L3 checklist |
| Replacement path | 紙の棚卸しリストへの転記 → UI-14 行単位確定 | Goal |
| Data safety / evidence | 実店舗データは commit しない。L3 は owner の local DB に synthetic 400 行を投入して行い、screenshot / DB / fixture は commit しない | Data Safety |
| Reporting / accounting semantics | 原価更新は棚卸し評価（`valuation_cost_price` 凍結）に遡及しない（既存契約、実装 A と同じ） | — |
| Manual verification | UI-14 の human visual confirmation（Windows native L3、400 行級） | Test Plan |
| 環境・再現性 | L3 fixture（synthetic 商品 CSV 400 行級）は Ready 依頼と同時に owner へ渡す（repo 外） | Test Plan |

## Design Readiness

- Existing design docs are sufficient because: 77-ui に UI-14 の URL state / CMD・DTO 契約 / 表示と操作 / Loading・Empty・Error / テスト観点（SPEC-PRV-D3〜D7）が揃い、実装 A で `revise_product_price` / `create_supplier` / `list_price_history` / `Supplier` / `PriceHistoryEntry` が `bindings.ts` に生成済み、52-ui §52.3 に UI-14 の route / file / group が登録済み。唯一の gap = 20-io `ProductSearchQuery` / 40-cmd `search_products` に取引先 filter が未定義（Plans.md の Design Readiness 注意）で、本 PR の SPEC-PRVB-D1 amendment（updated in this PR）で解消する（owner 裁定 (a)）。
- Source docs updated in this PR: 20-io `search_products`（field 2 行 + WHERE 1 行）、40-cmd `search_products`（1 文）、77-ui §77.3 / §77.9 / §77.10、decision-log D-052 Contract 行、UI_TECH_STACK §2.5 件数、90-traceability（再生成）。
- Design gaps intentionally deferred: Design Intent Audit 参照。
- Durable decisions discovered in this plan and promoted to source docs: SPEC-PRVB-D1 / D6。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI-14 → product_cmd（`search_products` / `list_suppliers` / `list_departments` / `create_supplier` / `revise_product_price` / `list_price_history`）→ product_service → product_repo。取引先 filter の WHERE は IO、価格改定・履歴・supplier 紐付けの判断は BIZ（実装 A）、UI は導出表示（掛率 / 新原価案 / 本日判定）と入力 validation だけを持つ。
- Backend function design: 20-io `search_products`（本 PR 改訂）、30-biz §4.4.1 / §4.6 / §4.7.2 / §4.7.3。
- Command / DTO / data contract: 40-cmd `search_products`（`ProductSearchQuery` + 2 field）/ `revise_product_price`（`PriceRevisionInput` / `PriceRevisionResult`）/ `create_supplier` / `list_price_history` / `list_suppliers`。
- Persistence / transaction / audit impact: 本 PR の backend 変更は読取り WHERE のみ。書込みは実装 A の `revise_product_price`（3 テーブル 1 TX）を UI から呼ぶ。schema 変更なし。
- Operator workflow / Japanese UI wording: title `一括価格改定`、filter label `取引先` / `部門` / `廃番を含む` / `取引先未設定の商品も含める` / `未設定の商品にこの取引先を設定する` / `新しい取引先を追加`、列 `商品コード` / `JAN` / `メーカー品番` / `商品名` / `現売価` / `現原価` / `現掛率` / `新売価` / `新原価（案）` / `確定`、badge `最近改定`、行 error `確定できませんでした` + `再試行`、常時文言 `画面を再読み込みすると、確定前に入力した新売価・新原価は失われます。1行ずつ確定してください。`、Empty `条件に一致する商品がありません` + `絞り込みを解除`（filter あり）/ `該当する商品がありません` + 商品一覧への link（filter なし、SPEC-PRVB-D9）。色のみの状態表現禁止（inventory-operator-ui）。
- Error, empty, retry, and recovery behavior: 一覧 = Alert + 再試行、行 = 行内 error + 再試行 + 入力保持、取引先追加 = dialog 保持、候補取得失敗 = inline error（一覧は隠さない）、Empty = filter 有無で導線を分ける（SPEC-PRVB-D9）。
- Testability and traceability IDs: REQ-105 / REQ-106、SPEC-PRV-D3〜D7、SPEC-PRVB-D1〜D9、D-031、D-052-C20、UI-14。

## Contract Probe

- Probe 1: tauri-specta が `#[serde(default)]` 付き field を TS optional として生成するか: 既存 `ProductSearchQuery.plu`（`#[serde(default)] pub plu: Option<PluMigrationFilter>`）が `bindings.ts` で `plu?: PluMigrationFilter | null` と optional 生成済み → `supplier_id` は同型で N/A。`include_unassigned: bool` + `#[serde(default)]` は bool での先例が無い → Writer は再生成後に `rg -n "include_unassigned" src/lib/bindings.ts` の形を報告し、required（`include_unassigned: boolean`）で生成された場合のみ `search.ts` の `buildProductSearchQuery` に `include_unassigned: false` を明示追加して既存 test 無改変で `npm run typecheck` を通す（Scope の条件付き許容）。
- Probe 2: `get_plu_slot_summary` が `plu_dirty` を読むか: `src-tauri/src/biz/plu_export_service.rs` の `get_plu_slot_summary` は `system_repo::get_setting`（snapshot 設定）と plu_slots 状態の集計で `plu_dirty` を参照しない（Coordinator の rg、2026-08-23）→ C20 から `pluSlotSummary` を除外（SPEC-PRVB-D6）。Writer は実装時に `rg -n "plu_dirty" src-tauri/src/biz/plu_export_service.rs` の `get_plu_slot_summary` 範囲が 0 hit であることを報告し、hit があれば C20 に `pluSlotSummary` を加えて packet amendment を申告する。
- Probe 3: `design_compliance_test` は fn シグネチャのみ照合し struct field は検査しない（実装 A SPEC-PRVA-D5 で確認済み）→ `ProductSearchQuery` の field 追加で allowlist 追加なし。`search_products` のシグネチャは不変。
- Probe 4: `npm run generate:routes`（`tsr generate`）は `src/routeTree.gen.ts`（gitignore、`src/main.tsx` が import）を再生成し、`pretypecheck` / `prelint` / `prelint:fix` / `pretest` の pre script（`package.json`）で自動実行され、dev / build 時は `vite.config.ts` の `tanstackRouter` plugin（`@tanstack/router-plugin/vite`）が生成する → 新 route file を置けば CI でも生成される。commit 対象なし。
- Probe 5: `useQueries` で 1 page 最大 200 件の `listPriceHistory(code, 1)` を並列発行したときの体感遅延: `未実測`。L3 item 3 で perPage 200 の一覧描画から badge 出現までを owner が体感で PASS/FAIL し、PR body に記録する。遅延が許容外なら SPEC-PRVB-D2 の却下案（DTO 拡張）を別起票する。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-PRVB-D1 `Some(X)` + `include_unassigned=false` → 選択取引先のみ（NULL / 他取引先は除外、total_count 一致） | product_repo `search_products` | `test_search_products_req105_supplier_filter_matches_only_selected_supplier` | — |
| SPEC-PRVB-D1 `Some(X)` + `true` → 選択取引先 + NULL（他取引先は除外、total_count 一致） | 同上 | `test_search_products_req105_supplier_filter_includes_unassigned_when_flag_set` | — |
| SPEC-PRVB-D1 `None` → 条件なし（`include_unassigned=true` でも全件） | 同上 | `test_search_products_req105_no_supplier_filter_when_unspecified` | — |
| SPEC-PRVB-D1 supplier 条件と department / keyword の AND | 同上 | `test_search_products_req105_supplier_filter_combines_with_other_conditions` | — |
| SPEC-PRVB-D1 wire: `ProductSearchQuery` に 2 field、既存呼出し元無改変 | bindings.ts / search.ts | AC の `generate_bindings` diff 空 + `rg` + 既存 `search.test.ts` / `useProductList.test.tsx` 無改変 PASS + `npm run typecheck` | — |
| 20-io / 40-cmd amendment の転記 | 20-io `search_products` / 40-cmd `search_products` | `rg -c "include_unassigned" docs/function-design/20-io-product-repo.md docs/function-design/40-cmd-product.md` 各 ≥ 1 + `doc-consistency-check.sh` exit 0 | — |
| SPEC-PRV-D3 / SPEC-PRVB-D5 取引先指定時に `supplier_id = X` + `include_unassigned = true` を送る | usePriceRevisionList / priceRevisionSearch | hook test `取引先指定時は supplier_id と include_unassigned=true を searchProducts に渡す` | L3 item 2 |
| SPEC-PRVB-D5 `includeUnassigned=false` を送る / 取引先未指定なら `supplier_id: null, include_unassigned: false` | 同上 | hook test `includeUnassigned=false のとき include_unassigned=false を渡す` / `取引先未指定なら supplier_id null と include_unassigned false を渡し URL の includeUnassigned は無視する` | — |
| SPEC-PRV-D3 在庫ゼロ商品を含む（在庫条件を query に載せない） | 同上 | hook test `在庫数 0 の商品も一覧に含まれ query に在庫条件を載せない` | — |
| SPEC-PRVB-D2 page 内各行の `listPriceHistory(code, 1)` と `latestChangedAt` の結び付け | usePriceRevisionList | hook test `page 内の各行について listPriceHistory(code, 1) を呼び changed_at を行に結び付ける` | L3 item 3（体感） |
| SPEC-PRVB-D5 URL 正規化（無効値 → 既定 / supplier 指定で includeUnassigned 既定 true / sort 無効 → product_code 昇順 / filter 変更で page 1 / perPage 50・100・200） | priceRevisionSearch | `priceRevisionSearch.test.ts`: `無効な search 値は既定へ回復する` / `supplier 指定時は includeUnassigned 欠落を true にし未指定時は落とす` / `sort は無効値と欠落で product_code 昇順になる` / `filter patch は page を 1 に戻す` | — |
| SPEC-PRV-D4 / SPEC-PRVB-D3 新原価案 floor 導出 | price-revision-math | `price-revision-math.test.ts`: `deriveProposedCost は整数除算で切り捨てる`（独立 oracle: (1200, 700, 1000) → 840 / (1250, 333, 1000) → 416 / (999, 700, 1000) → 699 / (1001, 999, 1000) → 999〈`Math.round` なら 1000 になる区別 case〉） | — |
| SPEC-PRV-D4 現売価 0 → 現原価 fallback | 同上 | `deriveProposedCost は現売価 0 で現原価を返す`（(1200, 700, 0) → 700） | — |
| 境界（実装 A から defer）10^7 直下の積の exact 計算 | 同上 | `deriveProposedCost は 10^7 直下の値でも exact に計算する`（(9_999_999, 9_999_999, 1) → 99_999_980_000_001、literal 独立転記） | — |
| SPEC-PRV-D4 現掛率 % 小数 1 桁四捨五入 / 現売価 0 は `—` | 同上 | `formatMarkupRate は小数 1 桁に四捨五入する`（(700, 1000) → "70.0" / (333, 1000) → "33.3" / (1, 16) → "6.3" / (2, 3) → "66.7" / (23, 80) → "28.8"〈`toFixed(1)` 直接実装なら "28.7" になる判別 case〉）/ `formatMarkupRate は現売価 0 で — を返す` | — |
| SPEC-PRV-D7 / SPEC-PRVB-D2 本日判定 | 同上 | `isRevisedToday は changed_at の日付部分と today の一致で判定する`（("2026-08-23T09:15:00", "2026-08-23") → true / ("2026-08-22T23:59:59", "2026-08-23") → false / (undefined, …) → false） | — |
| SPEC-PRV-D3 「取引先未設定の商品も含める」既定 on / toggle off 可 | PriceRevisionFilters / Page | RTL `取引先を選ぶと「取引先未設定の商品も含める」が既定 on で表示され off にすると include_unassigned=false で再検索する` | L3 item 2 |
| SPEC-PRV-D6 「未設定の商品にこの取引先を設定する」は取引先選択中だけ表示・既定 on | 同上 | RTL `「未設定の商品にこの取引先を設定する」は取引先選択中だけ表示され既定 on で supplier 変更時に on へ戻る` | — |
| SPEC-PRV-D4 新売価入力 → 新原価案初期化、現売価 0 行は `—` + fallback、新売価は空から | PriceRevisionTable | RTL `新売価入力で新原価（案）が導出され現売価 0 の行は掛率「—」と現原価 fallback になり新売価は空から始まる` | L3 item 3 |
| SPEC-PRVB-D4 新原価案を手で編集後は新売価変更で上書きしない / 未編集なら追従 | 同上 | RTL `新原価（案）を手で編集した後は新売価変更で上書きせず未編集なら追従する` | — |
| SPEC-PRV-D5 / SPEC-PRVB-D4 確定は該当 1 商品だけ、`assign_supplier_id` は toggle と取引先に従う | useReviseProductPrice / Table | RTL `確定は該当行 1 商品だけを reviseProductPrice に送り assign_supplier_id は取引先選択 + toggle on のとき supplier_id、それ以外 null` | L3 item 3 |
| SPEC-PRVB-D4 / D6 成功後に C20 invalidate → 一覧再取得で新価格表示、行入力消去、同行履歴再取得 | 同上 | RTL `確定成功後に D-052-C20 の独立 oracle 集合を invalidate し再取得した新価格を表示して行入力を消す` | L3 item 3 / 4 |
| SPEC-PRV-D5 失敗行だけ error + 再試行 + 入力保持、他行不変 | 同上 | RTL `確定失敗時は該当行だけ「確定できませんでした」と再試行を出し入力を保持し他行は変わらない` | — |
| SPEC-PRVB-D4 pending 中は同行 disabled、他行操作可 | 同上 | RTL `確定の送信中は同じ行の入力と確定が無効化され他行は操作できる` | — |
| SPEC-PRVB-D4 負値 / 非整数 / 空は field error で CMD を呼ばない | 同上 | RTL `新売価が負値または非整数なら field error を出し reviseProductPrice を呼ばない` | — |
| SPEC-PRV-D7 本日 changed_at の行だけ icon + text `最近改定` | 同上 | RTL `本日の changed_at を持つ行だけ「最近改定」badge を icon + text で表示する`（`vi.setSystemTime` で today 固定、mock 履歴は literal 転記） | L3 item 3 |
| SPEC-PRV-D7 再読込で確定前入力が消える常時文言 | Page | RTL `再読み込みで確定前の入力が失われる旨の文言を常時表示する` | L3 item 4 |
| SPEC-PRV-D6 / SPEC-PRVB-D7 取引先追加 → suppliers 再取得 → filter 選択状態 | CreateSupplierDialog / Filters | RTL `新しい取引先を追加すると createSupplier 後に listSuppliers を再取得し追加した取引先が filter で選択状態になる` | L3 item 2 |
| SPEC-PRV-D6 空白のみは CMD を呼ばない / 失敗時 dialog 入力保持 | 同上 | RTL `取引先名が空白のみなら createSupplier を呼ばず field error を出し失敗時は入力を保持する` | — |
| SPEC-PRVB-D9 Empty（filter なし / あり）と一覧取得失敗 | Page | RTL `filter なしの 0 件は商品一覧への導線、filter ありの 0 件は「条件に一致する商品がありません」と「絞り込みを解除」を出す` / `一覧取得失敗で Alert と再試行を出し再試行で再取得する` | — |
| UI-14 到達導線（REQ-105） | navigation.ts / route | `test_navigation_req105_ui14_active_at_products_price_revision`（`to: "/products/price-revision"`, `status: "active"`）+ 既存 `test_navigation_all_items_no_pending_status` PASS | L3 item 1 |
| D-052 C20 登録（SSOT + oracle + 件数 + docs） | invalidation-contract / oracle / meta test / decision-log / UI_TECH_STACK | `invalidation-contract.meta.test.ts` 件数 20 PASS + `invalidation-contract.static.test.ts` PASS + AC の rg | — |
| 登録: bindings 再生成 / 90-traceability 再生成 / route 生成 | bindings.ts / 90-traceability / routeTree.gen.ts | AC の `generate_bindings` diff 空 + `generate_traceability -- --check` exit 0 + `npm run generate:routes` 後の typecheck | — |
| gated amendment 1 (i) UI-USW-D3 (c) 分類: `PriceRevisionPage` は離脱ガード非適用（EXCLUDED）、常時文言で代替 | unsaved-changes-guard-sweep.test.ts / 77-ui §77.6 | 既存 `T17` が `PriceRevisionPage` を `EXCLUDED_PAGES` 込みで PASS + `rg -c "PriceRevisionPage" src/hooks/unsaved-changes-guard-sweep.test.ts` = 1 + `rg -c "UI-USW-D3" docs/function-design/77-ui-bulk-price-revision.md` ≥ 1 | L3 item 3（page 移動で確定前入力が消える挙動の目視） |
| gated amendment 1 (ii) 共有 `ListSkeleton`（04-backbone 原則 11） | components/patterns/ListSkeleton.tsx / 02-component-catalog | `ListSkeleton.test.tsx`: `ListSkeleton は指定行数の skeleton 行を描画し読み込み中を示す` + `rg -c "ListSkeleton" docs/design-system/02-component-catalog.md` ≥ 1 | L3 item 1（Loading 表示の目視） |
| SPEC-PRVB-D8 perPage 200 × 2 page での 400 行級の操作性 | Page / ProductPagination | —（自動化不能） | L3 item 3（主動線の反復速度 + 体感遅延） |

## Test Plan

Test Design Matrix: `docs/plans/test-matrices/2026-08-23-price-revision-impl-b.md`。
Human Gate が L3 を含むため、Writer 完了条件に `cd src-tauri && cargo check --release` を含める（CI gate ではない）。

- targeted tests: Ledger の Rust 4 件（product_repo inline `#[cfg(test)]`）、`priceRevisionSearch.test.ts` / `price-revision-math.test.ts` / `usePriceRevisionList.test.tsx` / `PriceRevisionPage.test.tsx` / `navigation.test.ts`、`invalidation-contract.meta.test.ts`（件数 20）/ `.static.test.ts`、`design_compliance_test`、`generate_traceability -- --check`、`generate_bindings` diff。
- negative tests: 負値 / 非整数の新売価で CMD 未呼出し、空白のみの取引先名、確定 reject で入力保持、一覧 reject で Alert、`include_unassigned=false` で NULL 行が出ない、`None` で flag 無視。
- compatibility checks: 既存 `test_search_products_req103_*` / `_req907_*` / `_req105_keyword_matches_maker_code` と `search.test.ts` / `useProductList.test.tsx` / `ProductListPage.test.tsx` が無改変で PASS。`bindings.ts` の diff は `ProductSearchQuery` 型への追加のみ。`test_navigation_all_items_no_pending_status` PASS。
- data safety checks: fixture は synthetic のみ。L3 は owner の local DB に synthetic 商品 CSV（400 行級、repo 外）を取り込んで行い、DB / screenshot / fixture は commit しない。
- main wiring/integration checks: route file → `routeTree.gen.ts` → サイドバー entry → `PriceRevisionPage` mount、`usePriceRevisionList` が `buildPriceRevisionSearchQuery` の結果で `commands.searchProducts` を呼ぶ、`useReviseProductPrice` が `invalidateByContract(..., productPriceRevise(code))` を呼ぶ、`bindings.ts` の `ProductSearchQuery` に 2 field。

Human visual confirmation checklist（Windows native L3、owner は目視と PASS/FAIL のみ。evidence 整形は agent。fixture = Coordinator が Ready 依頼と同時に渡す synthetic 商品 CSV 400 行級〈取引先あり / 未設定 / 現売価 0 / 本日改定済みを含む〉を UI-01c で取り込む）:

1. サイドバー「商品管理」に「一括価格改定」が表示され、押すと `/products/price-revision` が開き、title と絞り込み枠、一覧、常時文言（再読み込みで入力が失われる旨）が見える。
2. 取引先 filter: 取引先を選ぶと「取引先未設定の商品も含める」が on で表示され、未設定商品が一覧に混ざる。off にすると未設定商品が消える。「新しい取引先を追加」で日本語名を入力（IME 確定の Enter で誤送信しない）→ 追加 → filter で選択状態になる。空白だけでは追加されず field error が出る。
3. 400 行級（perPage 200 × 2 page）: keyword（メーカー品番の一部）で絞り込み → 現売価を値上げリストの旧売価と目視突合 → 新売価入力 → 新原価（案）が自動で埋まる（現売価 0 の行は `—` と現原価のまま）→ 確定 → 行が新価格で再表示され「最近改定」badge が付く、を 10 行程度反復する。紙の棚卸しリストへ転記するより速い主動線になっているか、一覧描画から badge 出現までの体感遅延（`未実測` → PASS/FAIL と所感を PR body へ）、page 2 への移動で確定前の入力が消えること（常時文言どおり）を確認する。
4. 確定後の反映: UI-01a 商品一覧で同商品の売価・原価が新値、UI-01b 修正モードの価格履歴に新しい行、ホームの PLU 未反映件数が増える（売価を変えた場合）。UI-14 を再読み込みしても確定済み行は新価格のままで、確定前の入力は消える。

## Boundary / Wire Contract

- producer: CMD `search_products`（`ProductSearchQuery` 拡張、product_cmd）、既存 `revise_product_price` / `create_supplier` / `list_price_history` / `list_suppliers` / `list_departments`
- consumer: `search_products` = UI-01a（既存、新 field は送らない / optional）と UI-14（本 PR）。`revise_product_price` = UI-14（本 PR で初めて UI から呼ぶ）/ UI-02（実装 C）。`create_supplier` / `list_price_history` = UI-01b（実装 A）と UI-14 — いずれも `src/lib/bindings.ts` 経由
- wire type: `ProductSearchQuery { keyword: string | null, department_id: number | null, is_discontinued: boolean | null, plu?: PluMigrationFilter | null, supplier_id?: number | null, include_unassigned?: boolean, sort_key: SortKey, sort_order: SortOrder, page: number, per_page: number }`（optional 表記は Contract Probe 1 の生成結果で確定）/ `PriceRevisionInput { product_code: string, new_selling_price: number, new_cost_price: number, assign_supplier_id: number | null }` / `PriceRevisionResult { product_code: string, changed: boolean, plu_dirty_set: boolean, supplier_assigned: boolean }` / `Supplier { id: number, name: string, created_at: string }` / `PriceHistoryEntry { …, changed_at: string }`（実装 A どおり）
- internal type: Rust `Option<i64>` supplier id / `bool` flag / `i64` 円 / `String` 日時（`%Y-%m-%dT%H:%M:%S`、Local）
- precision/range: 円整数 ≥ 0。上限契約は source docs に無く 10^7 未満は `未実測` 前提（実装 A と同じ）。新原価案 `newSelling × cost` は 10^14 未満で JS safe integer 内、10^7 直下の exact 計算は `price-revision-math.test.ts` の境界 test で検証（実装 A から defer された overflow 境界の消化）。掛率は小数 1 桁表示のみ（wire に載せない）。perPage は 50 / 100 / 200、IO は `PAGINATION_MAX_PER_PAGE = 200` で clamp
- round-trip path: URL search → `normalizePriceRevisionSearch` → `buildPriceRevisionSearchQuery` → `searchProducts` → 一覧。行入力 → `PriceRevisionInput` → `reviseProductPrice` → C20 invalidate → `searchProducts` / `listPriceHistory(code, 1)` 再取得 → 行表示 + badge。取引先: dialog → `createSupplier` → suppliers `refetch()` → URL `supplier` set
- invalid input: 負値 / 非整数 / 空の新売価は UI field error で CMD 未呼出し。CMD 側は実装 A どおり負値 = `Validation`、不存在 product / supplier = `NotFound`（UI-14 は行内 error 表示）
- compatibility: `search_products` の既存入力は不変（新 field 省略 = 従来動作）。`bindings.ts` は `ProductSearchQuery` 型への追加のみ。`routeTree.gen.ts` は生成物で commit しない

## Review Focus

- SPEC-PRVB-D1 の WHERE が COUNT と SELECT の両方に同じ条件で入り、`None` で flag が無視されるか。Rust 4 test が mutation 実注入（`OR p.supplier_id IS NULL` 除去 / flag 反転 / `None` でも条件付与）で実際に落ちるか。
- `#[serde(default)]` の生成形（Contract Probe 1）と `search.ts` 無改変の整合。`bindings.ts` diff が `ProductSearchQuery` 型のみか。
- 導出が `price-revision-math.ts` に隔離され、component 内に算式の複製がないか。unit test の oracle が literal 独立転記か（production 関数を oracle 側で呼んでいないか）。境界 test（10^7 直下）が exact 比較か。
- 確定が 1 行 1 request で、`assign_supplier_id` が toggle + 取引先選択に従うか。成功後の C20 invalidate と行入力消去、失敗時の入力保持 + 他行不変。
- C20 の key 集合が SPEC-PRVB-D6 と一致し、oracle / meta 件数 / decision-log / UI_TECH_STACK が同期しているか。`pluSlotSummary` 除外の根拠（Probe 2）。
- navigation entry の位置（52-ui §52.3 の表順）と到達テスト、route file の `validateSearch`。
- RTL の oracle が mock 戻り値を共有せず独立転記か。既存 test の改変・skip がないか（meta test の件数 literal 以外）。
- 20-io / 40-cmd / 77-ui の改訂が SPEC-PRVB-D1 と一致し、77-ui §77.9 の「後続実装 PR B」bullet が残っていないか（文言表 presence oracle: 新文言 ≥ 1 / 旧文言 0）。
- D-062: packet / Matrix 内の数値主張は契約値（source doc 由来）か `未実測` tag か。

## Spec Contract

Contract ID: SPEC-PRVB

- SPEC-PRV-D3 / D4 / D5 / D6 / D7（77-ui 転記済み、実装対象）と SPEC-PRVB-D1〜D9（本 packet の Design Decisions 節）を正とする。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-PRVB-D1 | product_repo WHERE + 20-io / 40-cmd 改訂 + bindings 再生成 | `test_search_products_req105_supplier_filter_*` / `_no_supplier_filter_when_unspecified` + rg | COUNT / SELECT 同一、None で無視、wire optional | cargo test + mutation 注入 + bindings diff |
| SPEC-PRV-D3 / SPEC-PRVB-D5 | priceRevisionSearch + usePriceRevisionList + Filters | `priceRevisionSearch.test.ts` / hook test / RTL toggle | 既定 on / URL 正規化 | npm test + L3 item 2 |
| SPEC-PRV-D4 / SPEC-PRVB-D3 | price-revision-math + Table | `price-revision-math.test.ts` / RTL 導出 | 隔離 / 独立 oracle / 境界 | npm test + L3 item 3 |
| SPEC-PRV-D5 / SPEC-PRVB-D4 / D6 | useReviseProductPrice + C20 + Table 行状態 | RTL 確定 / 失敗 / pending / C20 oracle + meta 20 | 1 行 1 request / 入力保持 / key 集合 | npm test + L3 item 3・4 |
| SPEC-PRV-D6 / SPEC-PRVB-D7 | CreateSupplierDialog + toggle | RTL 取引先追加 / assign_supplier_id | trim / 再取得 / 選択状態 | npm test + L3 item 2 |
| SPEC-PRV-D7 / SPEC-PRVB-D2 | useQueries 履歴 + badge + 常時文言 | hook test / RTL badge / 文言 | 本日判定 / 非表示条件 | npm test + L3 item 3・4 |
| REQ-105 到達 | navigation.ts + route | `test_navigation_req105_ui14_active_at_products_price_revision` | 表順 / pending 0 | npm test + L3 item 1 |
| SPEC-PRVB-D8 / D9 | ProductPagination / EmptyState / Alert | RTL Empty / Error | 導線 | npm test + L3 item 3 |
| 登録義務 | generate_bindings / generate_traceability / generate:routes / C20 docs | AC rg + --check exit 0 | 同期 | rg 出力 |

## Data Safety

- 実店舗の商品名・価格・取引先名を test fixture / PR / screenshot に commit しない。
- local-only paths: owner の local DB（L3 用、repo 外）、L3 用 synthetic 商品 CSV（repo 外または `$TMPDIR`、commit しない）。
- synthetic-only paths: `src-tauri/src/**/tests`、`src/features/products/**/*.test.ts(x)`、`src/config/navigation.test.ts` の fixture。

## Implementation Results

- `ProductSearchQuery` の取引先 filter、UI-14 の URL state / 行単位価格確定 / 取引先漸進補完 / 履歴 badge、D-052 C20、navigation、共有 loading pattern を実装し、設計書・生成 bindings・traceability を同期した。
- review-only 監査で検出した browser 履歴経由の取引先変更時 state reset と Empty recovery oracle を是正し、契約値の paging test も補強した。
- Draft PR: https://github.com/higekiri-w/inventory-system-public/pull/95

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Fill after review.
If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
- Plan Review round 1（Sonnet、独立 context、2026-08-23、packet `325da8a`）: P1 0 / P2 3 / P3 3、verdict fail。全件 Coordinator が rg で裏取りのうえ accept して是正 `baba1ca`: P2-1 Contract Probe 4 の `predev` は存在しない script（`package.json` は `pretypecheck` / `prelint` / `prelint:fix` / `pretest` のみ、dev は `vite.config.ts` の `tanstackRouter` plugin）→ 文言訂正 / P2-2 Design Sources の 50-ui §50.5 は CMD / DTO 契約で URL state は §50.4 → 訂正 + query key 先例を `query-keys.ts` に明示 / P2-3 `generate_traceability` の T4 FE baseline（`FE_UNREFERENCED_BASELINE = 22`）に新設 FE test file が触れる → Scope に REQ-105 / UI-14 参照の義務を追記 / P3-1 SPEC-PRVB-D9 の「該当する商品がありません」が wording 表と Matrix assert から漏れ → 両方に追加 / P3-2 `invalidation-contract.ts` / `invalidation-oracle.ts` のヘッダコメント C1〜C19 更新を Scope に追記 / P3-3 Ledger の floor 導出 oracle に Matrix の (1001, 999, 1000) → 999 を転記。
- Plan Review round 2（Sonnet、fresh context、2026-08-23、packet `baba1ca`）: delta 検証 7 hunk / anchor 9/9 実在 / 矛盾 0。P1 0 / P2 1 / P3 2、verdict fail。全件 Coordinator が実測・rg で裏取りのうえ accept して是正: P2-1 掛率 oracle の「(1, 16) が `toFixed(1)` 直接実装を検出」は誤り（両実装とも "6.3"。node で brute force 2001×2000 のうち 792 組が不一致、(23, 80) = 28.75 は直接 "28.7" / 四捨五入 "28.8"）→ Ledger / Matrix の oracle に (23, 80) → "28.8" を追加し誤主張を削除、Final Reviewer の実注入 mutant に D3 掛率 `toFixed` 直接を追加（6 → 7）/ P3-1 round 1 P3-1 是正で入れた「商品がまだ登録されていません」は 77-ui にも既存 EmptyState 先例（UI-01a `ProductListPage` title「該当する商品がありません」）にも無い新文言 → 先例 title に統一し SPEC-PRVB-D9 に根拠を明記、Matrix の Adjacent Pattern Audit に EmptyState 行を追加 / P3-2 SPEC-PRVB-D6 の除外根拠で「取引先」を E3 に含めていた（E3 は name / 部門 / 単位 / 価格のみ）→ 価格は E3、取引先は「consumer が supplier を読まない」に分けて訂正。
- Plan Review round 3（Sonnet、fresh context、delta 検証、2026-08-23、packet `f4981ac`）: delta 9 hunk / anchor 9/9 実在、(23, 80) と (1, 16) の実測値と brute force 792 組を node で再現一致、regression sweep（Workflow State 13 field / 77-ui 契約語 26/26 / `doc-consistency-check.sh --target plan` exit 0・ERROR 0・WARN 2 = PK3 既知）。P1 0 / P2 0 / P3 1、verdict pass。P3-1 Matrix の Adjacent Pattern Audit「mutation 成功時 invalidation（D-052）」行に round 2 P3-2 の是正（価格 = E3 / 取引先 = consumer なし）が伝播していなかった → accept、同行を訂正。Plan rally 収束（round 3/3、天井内）。Plan Commit 候補 = plan-first commit `325da8a`（本 branch の全 content commit の祖先）。
- Final Review（Sonnet、fresh context、worktree 隔離、2026-08-23、content candidate `cc60e26`）: P1 0 / P2 0 / P3 0 = PASS。Scope 33 file が packet Scope / gated amendment 1 と一致（packet 外 0 / 未実装 0、`docs/plans/` hunk は Implementation Results のみ、既存 test 改変は許容 3 種のみ）、Ledger 36/36 行を実装 + test で監査（C20 oracle / 算式隔離 / `isRevisedToday` 文字列比較を確認）、AC 全項目を再実測（`cargo test` 886 PASS / `npm test` 144 file・1058 PASS / bindings diff 空 / traceability `--check` exit 0 / `--target plan` 全チェック通過 / `cargo check --release` PASS）、Matrix の 7 mutant + reviewer 選定 3 mutant（`includeUnassigned` 既定 false 化 / `costTouched` guard 除去 / toggle 無視）を clean worktree で実注入し全 kill・生存 0、終了時 worktree clean。所見: 掛率 `toFixed` 直接 mutant の (23, 80) 判別は `(cost / selling) * 100` の演算順でのみ再現（test は実際に捕捉、欠陥ではない）。
- Findings Freeze: frozen after Final Review（2026-08-23）; post-freeze exceptions: none.

### 遷移記録（2026-08-23、state-only 遷移 plan-draft -> plan-gate -> plan-approved -> implementing）

- plan-draft -> plan-gate の evidence: packet と Test Design Matrix を plan-first commit `325da8a` で commit 済み、`doc-consistency-check.sh --target plan` exit 0（WARN は未実装 test token の PK3 のみ）。
- plan-gate -> plan-approved の evidence: 独立 Sonnet Plan Reviewer 3 round（round 1 P1 0 / P2 3 / P3 3 → 是正 `baba1ca`、round 2 P1 0 / P2 1 / P3 2 → 是正 `f4981ac`、round 3 fresh delta 検証 P1 0 / P2 0 / P3 1 = pass → 是正 `4b0b332`、Review Response 参照）、owner 裁定 (a)（`ProductSearchQuery` amendment を本 packet に同梱）+ owner plan approval（2026-08-23、介入 1 回目 / 予算 3 回）、Plan Commit = plan-first commit `325da8a`（本 branch の全 content commit の祖先）。
- plan-approved -> implementing の evidence: 実装は Codex Writer 発注（cwd pin / Plan Commit 記入 / 本遷移を Coordinator が先行）で着手し、plan-first commit が全実装 commit に先行する。隣接 3 遷移を 1 state-only commit で圧縮記録（PR #84 / #93 / #94 と同型、DEV_WORKFLOW 圧縮規則、forward state-only 1 本目 / cap 3）。

### gated amendment 1（2026-08-23、Codex Writer fail-closed 起源、true positive）

- 事象: Writer が実装後の `npm test` で 1 件 FAIL（1051 PASS）を検出して停止・申告。`src/hooks/unsaved-changes-guard-sweep.test.ts` `T17` は `src/features/**/*Page.tsx` 全件を `APPLIED_PAGES` / `EXCLUDED_PAGES` の manifest と完全一致で照合するため、新設 `PriceRevisionPage.tsx` が未分類で落ちる。発注書は既存 test の変更を meta test 件数と navigation test に限定していた。あわせて Writer は 77-ui §77.7 の「共通 ListSkeleton」が未実装（`src/components/patterns` に無く、`ui/skeleton.tsx` primitive のみ）のため `src/components/patterns/ListSkeleton.tsx` を新設し、Rust 既存 test の `ProductSearchQuery` struct literal に新 field 2 行を追加していた（Coordinator が `git status` / `git diff` で検分）。
- 検分: `unsaved-changes-guard-sweep.test.ts:8-22`（APPLIED 6 画面）/ `:23-`（EXCLUDED に `StocktakePage` を含む）/ `:99-117`（T17 完全一致）、`docs/UI_TECH_STACK.md` §6.11 UI-USW-D3「除外 (c) 行単位の即時 DB 保存で蓄積未保存が生じない画面（棚卸し）」、77-ui §77.6（行単位確定 + 常時文言「画面を再読み込みすると…1行ずつ確定してください。」+ draft 保存テーブルなし）、`docs/design-system/04-backbone.md` L25 原則 11「読込みは共通 `ListSkeleton`」、`02-component-catalog.md` に ListSkeleton 未記載（rg 0 hit）。
- 裁定: (i) `PriceRevisionPage` を `EXCLUDED_PAGES` に追加（UI-USW-D3 (c)、棚卸しと同型の行単位即時保存。77-ui は常時文言を代替安全網として契約しており、離脱ガード追加は 77-ui の契約外で UX 変更になるため採らない）+ 77-ui §77.6 に非適用の 1 文を転記（UI-USW-D3「適用画面は各 function-design の該当節に明記」の裏返しとして非適用理由も明記）。(ii) `ListSkeleton` 新設は 04-backbone 原則 11 に適合するため採用、共有 pattern の慣行に合わせ test と 02-component-catalog 1 行を追加。(iii) Rust fixture の field 追加は struct literal の機械的追従で assertion 不変のため許容、Scope の「既存 test 無改変」の例外に明記。Final Reviewer は (i) の分類根拠と (iii) の assertion 不変を監査対象に含める。
- 却下: `PriceRevisionPage` を `APPLIED_PAGES` に入れて離脱ガードを配線（77-ui 契約外、Plan Gate 済み scope の UX 変更）/ sweep test から UI-14 を特例で除外する条件分岐（manifest 完全一致の設計意図を壊す）。
- amendment commit SHA は Workflow State `Amendments` に後続 commit で記録（PR #86 gated amendment 1 と同型）。

### 遷移記録（2026-08-23、state-only 遷移 implementing -> local-verified -> independent-review -> human-confirm）

- implementing -> local-verified: content candidate `cc60e26`（Codex Writer 第 1 発注、gated amendment 1 反映済み、relay 1/2）で L1 `local-ci.sh full` RESULT=PASS / END_TREE_STATE=CLEAN / MERGE_EVIDENCE_VALID=true（evidence path は PR #95 body）。Coordinator の packet commit（`119847e` / `b6a7c72`）は docs/plans のみ。exact-HEAD の L1 full は Ready 遷移 commit で再実施する。
- local-verified -> independent-review: 独立 Sonnet Final Reviewer（fresh context、worktree 隔離）が Contract Audit（Scope 突合、Ledger 36 行の実装 + test 監査、AC 再実測、Matrix 7 mutant + 追加 3 mutant の実注入、PR body / docs 検査。件数は Review Response の Final Review 行と reviewer 報告が正）を実施。
- independent-review -> human-confirm: Final Review P1 0 / P2 0 / P3 0 = PASS、是正 delta なし。`Reviewed Content HEAD` = `cc60e26`。隣接 3 遷移を 1 state-only commit で圧縮記録（post-implementation state-only 1 本目 / forward 合計 2 本目 / cap 3、PR #94 と同型）。

### 遷移記録（2026-08-23、state-only 遷移 human-confirm -> ready-hosted-final）

- Windows native L3（介入 2/3）: checklist 1〜3 を owner が全件 PASS（item 1 到達 / title / 絞り込み / 一覧 / 常時文言、item 2 取引先 filter の未設定含む on = 380 件 / off = 250 件・空白 reject・IME 確定 Enter 誤送信なし・追加後の選択状態、item 3 perPage 200 × 2 page・page 移動で確定前入力消失・現売価 0 の `—` + 現原価維持・10 行の期待値確定・全行「最近改定」・描画から badge 出現まで一瞬で速度良好）。item 4 は UI-01a 売価原価 / UI-01b 価格履歴 / UI-14 再読込後の確定値と badge / 確定前入力消失が PASS、ホームの PLU 未反映件数だけ fixture（380 件すべて `PLU対象=0`、ホーム件数は `plu_dirty = 1 AND plu_target = 1`）で観測不能 → owner 裁定により介入 3/3 で最小シナリオ（復元済み実 DB の PLU 反映済み実商品 1 件を UI-14 で売価改定 → ホーム N → N+1 → backup 復元）を追加実施し PASS。
- Data recovery: L3 前の控え（`inventory_backup_20260823_033106.db`）へ通常復元し、ホーム遷移 / synthetic 商品コード 0 件 / synthetic 取引先消失 / 残留なしを確認。追加シナリオ後も再復元済み。break-glass は未使用。
- 非再現観察: 新原価（案）が一度空欄に見えた事象は再試行で再現せず（215 が自動入力）、機能 FAIL とせず観察として記録。
- UX findings（全一覧画面を対象とする改善候補、本 PR の scope 外、closeout で Plans.md backlog へ記録）: 総件数と pagination の上部表示 / table header の sticky 化 / 長い一覧で商品識別列を見失わない設計 / 枠線・背景・操作領域のコントラスト強化 / 緑内障のある実利用者を前提にした Windows native 再評価。
- owner Ready 承認（2026-08-23、介入 3 回目 / 予算 3 回）: 本 state-only commit で `human-confirm -> ready-hosted-final` を materialize（forward state-only 3 本目 / cap 3、post-implementation 2 本目 / cap 2）。resulting exact HEAD で L1 full を再実行し、PR body を更新してから Draft を解除する。Ready event の同一 HEAD hosted final を待ち、三点一致前は merge しない。
