# Plan Packet: ㉖ 在庫変動履歴からの戻りで在庫照会の検索条件と商品選択を保持する（NAV-1、R3）

2026-09-16 起草。出典は [監査 NAV-1](../../research/2026-09-16-diagram-audit.md#nav-1-在庫変動履歴からの戻りで商品選択が失われる)（P2 / confirmed）と Backlog「やると決めたもの（順番未定）」の NAV-1 行。owner 2026-09-16「次何やるかふたつとって早速始めよう」で Coordinator が wave 12 の lane 1 に選定（lane 2 = ㉗ `docs/plans/2026-09-16-stocktake-count-baseline.md`、file footprint 互いに素）。file:line は origin/main `c6167c4d` で実測（Coordinator 2026-09-16）。実装は別 run（Codex 発注書 60）とし、独立 Plan Review 通過後に発注する。Test Design Matrix: `docs/plans/test-matrices/2026-09-16-stock-movements-return-selected.md`。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: bbec3bf518d834cbd03e594cfae05faf75cb5a45
- Amendments: b0f9fb50cfa45e62fd07412c181c2a6fee8f1f65, df865a4ebfeea8ad32f174c30ae9d6e03c0a9a37, f87702bcc7064a47e5fb944bfa62c2e90fb6e082, 1d1b6748dc0aad6ea2dd2ce411dcb90d297db97f, 7479a7ef6fc55bdd9b2e731ceb1d750ceccb8e7f
- Coordinator: Fable 5.1
- Writer: Codex（発注書 60 run 1〜3。実装 `3495e85e` / docs `081b1303` は Codex）+ Sonnet subagent（GA4: relay 3/3 到達後の残作業 = 90-traceability 再生成 / full / Draft PR のみ。worktree 分離、Plan Reviewer とは別 fresh context）
- Plan Reviewer: Sonnet（独立 fresh context）
- Final Reviewer: Sonnet + Opus（独立 fresh context）
- Final Review Minimum: 2
- Human Gate: ready,merge,manual

manual = owner Windows native L3 1 往復（在庫照会で検索 → 商品行を展開 → 「在庫変動履歴」→ 「在庫照会へ戻る」で同じ検索条件と選択行が戻る。続けて業務記録詳細の在庫変動履歴 link 経由（`returnTo` なしの入口）→ 「在庫照会へ戻る」でその商品が検索・選択された在庫照会になる。目視と PASS/FAIL のみ。fixture 不要 = demo seed の任意の商品）。

遷移記録（append-only）:
- kickoff → spec-check → design → plan-draft → plan-gate（本 commit）: Risk R3（route/search state。在庫変動履歴 route の search param を 1 つ追加し、在庫照会側に `returnTo` の producer を 1 site 追加する。R2/R3 で迷う場合は R3 の規則）。Design Phase = 66 に UI-06c-D9、58 に UI-06a-D7 を新設する design 判断を本 packet の D-D1〜D-D4 で先行し（実装 run で S5 / S6 として source docs へ書く）、DSR-18 本文と `src/lib/return-to.ts` は不変。Test Design Matrix を同 commit で置く。
- plan-gate → plan-approved → implementing（本 commit、state-only）: Plan Review round 1（Sonnet、P1/P2 = 0、P3 2）→ in-place 是正 `bbec3bf5`（T2' の表記、Writer を Sonnet subagent へ）。P3 のみのため reviewer 再投入なし（Subagent Budget）。Plan Commit = `bbec3bf5`（plan-first `93d2227c` → 是正を含む確定版）。実装は Sonnet subagent の worktree run で本 commit を HEAD_SHA として開始する。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 15分
- relay 往復上限: 4（既定 2 から改訂。GA2 / GA3 / 改訂 3 で 3 消費〈Coordinator 起因〉。GA5: L3 round 1 で見つかった実装欠陥の修正 run を Codex 発注書 61 として 4 往復目に置く、理由 = owner「Codex に回したほうが質良い」と、欠陥が Coordinator の設計指示〈文字列 fallback〉起因であること）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
在庫変動履歴 route `/stock/$code/movements` の search schema に `returnTo` を追加し（route/search state）、在庫照会の詳細（`StockDetailContent`）から在庫変動履歴へ遷移する link を `returnTo` の producer にする。在庫変動履歴の「在庫照会へ戻る」は `returnTo` を共通 helper `normalizeReturnTo` で検証して使い、欠落・不正時は商品コード検索付き `/stock` へ fallback する。route 定義 file、DTO、BIZ、`return-to.ts`、在庫照会の hook（`useStockInquiry`）は不変。UI route/search behavior に触れるため Risk Tiers の規則で R3 とし、Test Design Matrix / Spec Contract / Trace Matrix / Data Safety を置く。R4 要素（データ lifecycle・実データ）はない。

## Goal

Goal Invariant: 在庫照会で検索して商品を選び、在庫変動履歴を見てから「在庫照会へ戻る」と、同じ検索条件（`q` / `dept` / `status` / `page`）と同じ商品の展開状態に戻る。在庫変動履歴へ直接（業務記録詳細の在庫変動履歴 link 経由、`returnTo` なしの入口）来た場合も、「在庫照会へ戻る」でその商品が検索・選択された在庫照会に着地する。

### 最小完了条件

- (1) 在庫照会の詳細（インライン展開・フォールバックカードの両経路）から在庫変動履歴へ遷移すると、URL に `returnTo=<現在の /stock URL（search state 込み）>` が付く
- (2) 在庫変動履歴の「在庫照会へ戻る」は `returnTo` があればそこへ戻り、欠落・不正（`/` 始まりでない、`//` 始まり）なら `/stock?q=<商品コード>&selected=<商品コード>` へ戻る
- (3) 在庫変動履歴の filter 変更・page 送り・「絞り込みを解除」で `returnTo` が失われない。在庫変動履歴 → 業務記録詳細 → 「前の画面へ戻る」で在庫変動履歴へ戻ったときも `returnTo` が残る
- (4) 66 / 58 の source docs が上記を UI-06c-D9 / UI-06a-D7 として保持する

### 失敗定義

在庫照会 hook（`useStockInquiry.ts`）の検索前ガード（§58.4「検索前に `selected` が残る → clear」）の撤去・変更、`src/routes/**` / `src/lib/return-to.ts` / `src/features/inventory-records/**` の変更、DSR-18 本文（`01-decision-rules.md`）の変更、「在庫照会へ戻る」の label 変更、業務記録詳細 6 画面の「在庫変動履歴」link への `returnTo` 追加（別 scenario、Non-scope）。

### 非目的

在庫照会に戻ったときの scroll 位置（DSR-20 系）、在庫変動履歴の filter / pagination / 表示、業務記録詳細から在庫変動履歴へ来た場合に「業務記録詳細へ戻す」導線（label が「在庫照会へ戻る」のままなので fallback で在庫照会へ戻す）、廃番商品が在庫照会の検索対象外で fallback 後に選択が解除される件（既存の `is_discontinued: false` 契約、§58.3）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（origin/main `c6167c4d`、Coordinator 計測 2026-09-16）

- `src/features/stock-movements/StockMovementsPage.tsx`: `:90` `<Link to="/stock" search={{ selected: productCode }}>` = 「在庫照会へ戻る」（`rg -c 'search=\{\{ selected: productCode \}\}'` = 1、`rg -c '在庫照会へ戻る'` = 1）/ `:73-82` `returnToParams` → `detailReturnTo`（業務記録詳細へ送る `returnTo`、`dateFrom` / `dateTo` / `type` / `page` のみ直列化）/ `:249` `<MovementTable ... returnTo={detailReturnTo} />` / `:51-57` `updateSearch` は `...prev` を spread（未知 key を落とさない）/ `:65-72` `resetFilters` も `...prev` を spread。`rg -c 'returnTo'` = 8、`rg -c 'normalizeReturnTo'` = 0、`useRouterState` 未使用
- `src/features/stock-movements/types.ts:30-43` `stockMovementsSearchSchema` = `dateFrom` / `dateTo` / `type` / `page` の 4 key（`rg -c 'returnTo'` = 0）/ `:47-60` `normalizeStockMovementsSearch` は 4 key だけを返す（`returnTo` は page 側で `search.returnTo` を直接読む。normalize の返却型は不変）
- `src/routes/stock/$code.movements.tsx:14-17` は `stockMovementsSearchSchema` を `validateSearch` に渡すだけ（schema は types.ts が単一所有者、route file 非接触）
- `src/features/stock-inquiry/components/StockDetailContent.tsx:35-44` `ActiveCta` = `<Link to="/stock/$code/movements" params={{ code: productCode }}>`（`returnTo` なし。`rg -c 'useRouterState|returnTo'` = 0）。`:96` で「在庫変動履歴」に使用。`StockDetailContent` は `ProductListTable.tsx:4`（インライン展開）と `StockDetailCard.tsx:21`（フォールバック）の 2 経路で共用、いずれも `/stock` route 内でのみ描画
- `src/features/stock-inquiry/components/StockDetailContent.test.tsx:33-48` 「REQ-301: StockDetailContent shows active movement history link」= href `"/stock/BT0002/movements"` を固定（`rg -c '"/stock/BT0002/movements"'` = 1、`rg -c 'returnTo'` = 0）。`renderWithRouter`（`src/test/render-with-router.tsx`、実 TanStack Router + memory history、`initialPath` 既定 `/`）
- `src/features/stock-movements/StockMovementsPage.test.tsx`: `renderWithRouter` 経由（`:26-31` `renderWithClient`、実 router）。`:87-123` は `detailReturnTo` の href を `"/inventory/disposal/records/7?returnTo=%2Fstock%2FBT0002%2Fmovements%3FdateFrom%3D2026-06-01%26dateTo%3D2026-06-30%26type%3Ddisposal%26page%3D2"` で固定（実 router の search 直列化 = `%3F` `%3D` `%26`）。`rg -c '在庫照会へ戻る'` = 0、`rg -c 'returnTo'` = 1、`rg -c 'selected'` = 0
- 受け側 `src/features/stock-inquiry/hooks/useStockInquiry.ts:158-171`: `isAllEmpty && selected !== null → navigate({ selected: undefined })`（§58.4、`useStockInquiry.test.tsx:768` 「検索前（status=all + q 空）に selected 付き URL → clear + detail 走らせない」）。`q` があれば list を取得し、`selected` が list に含まれれば展開する（`:733` 「list 成功時に selected が現 list に不在なら clear」）。本 lane 非接触
- 商品コード検索: `src-tauri/src/db/product_repo.rs:780-781` `keyword` は LIKE `%kw%`、`:1845` `test_search_products_req103_keyword_product_code` が商品コード一致を固定。`searchProducts` は `is_discontinued: false`（`useStockInquiry.ts:58`）
- 共通 helper `src/lib/return-to.ts` `normalizeReturnTo(value, fallback)`: `/` 始まりかつ `//` 始まりでない値だけ通す。consumer の同型 = `DisposalRecordDetailPage.tsx:46` `backHref = normalizeReturnTo(returnTo, "/inventory/records")` → `:80` `<Link to={backHref}>`、test `DisposalRecordDetailPage.test.tsx:120-141`（`it.each` で query 付き / 外部 URL / `//` の 3 ケース、実 router で href を固定）
- 在庫変動履歴への入口: `StockDetailContent.tsx:39`（在庫照会、本 lane の producer）+ 業務記録詳細 6 画面（`CsvImportRecordDetailPage.tsx:224` / `ManualSaleRecordDetailPage.tsx:179` / `ReceivingRecordDetailPage.tsx:168` / `DisposalRecordDetailPage.tsx:175` / `StocktakeRecordDetailPage.tsx:209` / `ReturnRecordDetailPage.tsx:212`、いずれも `params` のみ、Non-scope）
- docs: `66-ui-stock-movements.md` §66.1 主動線 4「『在庫照会へ戻る』で `/stock?selected=$code` に戻る」/ §66.2 決定表 UI-06c-D1〜D8 / §66.3 search state 4 key / §66.4 Route search 4 key / §66.5 Header actions「`在庫照会へ戻る` → `/stock?selected=$code`」/ §66.7 Tests。`rg -c '/stock\?selected=\$code'` = 2（GA2 で regex を旧 URL に限定、値は同じ）、`rg -c 'returnTo'` = 0、`rg -c 'UI-06c-D9'` = 0（docs / src 全体でも 0）。66 に変更履歴表はない
- docs: `58-ui-stock-inquiry.md:538` 「『在庫変動履歴』は UI-06c で active link 化し、`/stock/$productCode/movements` へ遷移する」/ `:536` 「商品修正」は `returnTo` を渡さない（不変）/ §58.10 業務ルールに `#### UI-06a-Dn:` block（D6 まで使用済み、`:602`）/ `:671` 変更履歴表。`rg -c 'UI-06a-D7'` = 0（docs / src 全体でも 0）
- 対象 test 3 file（`StockMovementsPage` / `StockDetailContent` / `useStockInquiry`）は全 PASS（本数は PR body / CI 出力を正とする）

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **D-D1 在庫照会 → 在庫変動履歴の link を `returnTo` producer にする（UI-06a-D7）**: `StockDetailContent` の `ActiveCta` に `useRouterState({ select: (state) => state.location.href })` の現在 href を `search={{ returnTo }}` で渡す（`DisposalPage.tsx:134` と同型、新しい state を作らない）。決定: 「在庫変動履歴」link は現在の `/stock` URL（search state 込み）を `returnTo` として送る。理由: 監査 NAV-1 のとおり、`/stock?selected=<code>` だけでは受け側の検索前ガード（§58.4）が `selected` を解除し、利用者が検索と選択をやり直す。DSR-18 の判定フロー「業務記録詳細へ遷移する link か？ No → その導線固有の契約」に該当するため、DSR-18 本文は変えずに 58 / 66 の局所契約として置く。捨てた案: 受け側の検索前ガードを撤去（bookmark / F5 の detail 空振り防止〈Codex Round 1 P2-2〉を壊す）/ `/stock?q=<code>&selected=<code>` を常に送る（元の検索条件を捨てる）
- **D-D2 「在庫照会へ戻る」は `normalizeReturnTo(search.returnTo, fallback)`（UI-06c-D9）**: fallback は `<Link to="/stock" search={{ q: code, selected: code }}>`（`Link` の `search` props、GA5）で組む。label は「在庫照会へ戻る」のまま（`returnTo` も fallback も在庫照会に着地するため、DSR-18 の「前の画面へ戻る」label は使わない）。理由: 直接アクセス・業務記録詳細からの入口では在庫照会の元条件が存在しないため、商品コード検索（LIKE、`product_repo.rs:1845`）で対象商品を 1 件以上含む list を出し、`selected` で展開する。捨てた案: fallback を `/stock`（商品を見失う、現行と同じ不便）/ fallback を `/stock?selected=<code>`（現行のまま、NAV-1 未解消）
- **D-D3 業務記録詳細 → 在庫変動履歴 → 「前の画面へ戻る」の往復で `returnTo` を保つ**: `StockMovementsPage.tsx:73-82` の `returnToParams` に `returnTo` があれば `set("returnTo", search.returnTo)` を加え、`detailReturnTo` に入れ子で載せる。理由: 在庫変動履歴の元記録 link から業務記録詳細へ行って戻ると在庫変動履歴の URL は `detailReturnTo` で再現されるため、そこに `returnTo` が無いと「在庫照会へ戻る」が fallback へ落ちる。`updateSearch` / `resetFilters` は `...prev` を spread 済みで追加変更なし。捨てた案: 入れ子を避けて sessionStorage に持つ（URL 以外の state を増やす、F5 で消える）
- **D-D4 route file / hook / helper / DSR-18 は不変**: `stockMovementsSearchSchema`（types.ts が単一所有者）に `returnTo: z.string().max(500).optional().catch(undefined)` を足すだけで route file は変わらない（`stocktake.records.$stocktakeId.tsx:7` と同じ形）。`normalizeStockMovementsSearch` の返却型は不変（`returnTo` は page が `search.returnTo` を直接読む）。decision-log は追加しない（既存 DSR-18 / TRACE-D11 の考え方を局所契約へ適用するだけで durable な新判断ではない）

## Scope

- **S1 `src/features/stock-movements/types.ts`**: `stockMovementsSearchSchema` に `returnTo: z.string().max(500).optional().catch(undefined)` を追加（D-D4）。`StockMovementsSearch` 型は `z.output` 由来で自動追従
- **S2 `src/features/stock-movements/StockMovementsPage.tsx`**: `:90` の `Link` を、`returnTo` が有効（`normalizeReturnTo` を通る）なら `<Link to={returnTo}>`、欠落・不正なら **`<Link to="/stock" search={{ q: productCode, selected: productCode }}>`** へ（D-D2。**GA5**: fallback を文字列 `/stock?q=…&selected=…` で組むと TanStack Router の `parseSearch` が値を `JSON.parse` するため、数字だけの商品コードが number になり `z.string()` で落ちて検索前に戻る〈L3 round 1 (b) FAIL〉。`search` props なら `stringifySearch` が JSON に見える文字列を引用符付きで書き、round trip で string に戻る）/ `:73-82` `returnToParams` に `returnTo` を追加（D-D3）
- **S3 `src/features/stock-inquiry/components/StockDetailContent.tsx`**: `ActiveCta` の `Link` に `search={{ returnTo }}` を追加。`returnTo` は `useRouterState({ select: (state) => state.location.href })`（D-D1）
- **S4 tests**: `StockDetailContent.test.tsx:33-48` を `initialPath` 付き（例 `/stock?q=BT&selected=BT0002`）で render し、href = `/stock/BT0002/movements?returnTo=%2Fstock%3Fq%3DBT%26selected%3DBT0002` を固定（Matrix T1）/ `StockMovementsPage.test.tsx` に「在庫照会へ戻る」の href を `it.each`（T2' 有効 `returnTo` / T2 欠落 / T3 外部 URL / T4 `//`）で固定、`detailReturnTo` に `returnTo` が入れ子で載る case（T5）、filter 変更後も `returnTo` が残る case（T6、`onSearchChange` の updater 結果で確認）
- **S5 `docs/function-design/66-ui-stock-movements.md`**: §66.1 主動線 4 を「『在庫照会へ戻る』で `returnTo`（在庫照会からの遷移元 URL）へ戻る。欠落・不正時は `/stock?q=$code&selected=$code`」へ / §66.2 決定表に UI-06c-D9 行（決定 = D-D2 + D-D3、理由 / 棄却案 = 上記）/ §66.3 型と fallback 規則に `returnTo?: string`（`max(500)`、不正値は `undefined`）/ §66.4 Route search に `returnTo` / §66.5 Header actions の遷移先を書き換え / §66.7 Tests に「REQ-303 / UI-06c-D9: `returnTo` の正規化と fallback、`detailReturnTo` への入れ子」bullet
- **S6 `docs/function-design/58-ui-stock-inquiry.md`**: `:538` の bullet を「『在庫変動履歴』は … へ遷移し、現在の `/stock` URL（search state 込み）を `returnTo` として送る（UI-06a-D7）」へ / §58.10 に `#### UI-06a-D7: 在庫変動履歴への `returnTo` 送信（2026-09-16、NAV-1）` block（決定 / Why / Rejected = D-D1）/ `:671` 変更履歴に 1 行
- **S7 `docs/function-design/90-traceability.md`（GA3）**: `cargo run --bin generate_traceability` の再生成差分（手動編集なし、commit 3 として単独）
- **S8（Coordinator、plan-first commit）**: Plans.md / backlog.md の登録、Test Design Matrix

## Non-scope

- `src/routes/**`（route file は schema を import するだけ）、`src/lib/return-to.ts`、`src/features/stock-inquiry/hooks/**`、`src/features/stock-inquiry/StockInquiryPage.tsx`、`src/features/inventory-records/**`（業務記録詳細 6 画面の「在庫変動履歴」link は `params` のみのまま。fallback で在庫照会へ戻る）
- `docs/design-system/01-decision-rules.md`（DSR-18）、`65-inventory-record-traceability.md`（TRACE-D11）、decision-log
- `src-tauri/**`、`MovementTable` / `useStockMovements`
- 在庫変動履歴の filter・pagination・表示・label

## Acceptance Criteria

rg oracle は出力空 = 0 件。baseline は起票時実測（origin/main `c6167c4d`）。追加系は `≥` で書く（import 行の書き方で count が揺れるため固定値にしない）。

- **AC1** `rg -c 'returnTo' src/features/stock-movements/types.ts` ≥ 1（baseline 0）/ `rg -c 'normalizeReturnTo' src/features/stock-movements/StockMovementsPage.tsx` ≥ 2（baseline 0、import + 呼び出し）/ `rg -c 'search=\{\{ selected: productCode \}\}' src/features/stock-movements/StockMovementsPage.tsx` = 0（baseline 1）/ `rg -c 'returnToParams.set\("returnTo"' src/features/stock-movements/StockMovementsPage.tsx` = 1（baseline 0）
- **AC2** `rg -c 'useRouterState' src/features/stock-inquiry/components/StockDetailContent.tsx` ≥ 2（baseline 0、import + 呼び出し）/ `rg -c 'search=\{\{ returnTo \}\}' src/features/stock-inquiry/components/StockDetailContent.tsx` = 1（baseline 0）
- **AC3** `rg -c '"/stock/BT0002/movements"' src/features/stock-inquiry/components/StockDetailContent.test.tsx` = 0（baseline 1）/ `rg -c 'returnTo' src/features/stock-inquiry/components/StockDetailContent.test.tsx` ≥ 1（baseline 0）/ `rg -c '在庫照会へ戻る' src/features/stock-movements/StockMovementsPage.test.tsx` ≥ 1（baseline 0）/ `rg -c 'returnTo' src/features/stock-movements/StockMovementsPage.test.tsx` ≥ 4（baseline 1）
- **AC4** `rg -c '/stock\?selected=\$code' docs/function-design/66-ui-stock-movements.md` = 0（baseline 2 = §66.1 主動線 4 + §66.5 Header。**GA2 で旧 URL に限定**: 旧 regex `selected=\$code` は S5 の新 fallback `/stock?q=$code&selected=$code` にも一致し、S5 と両立しなかった）/ `rg -c 'UI-06c-D9' docs/function-design/66-ui-stock-movements.md` ≥ 3（baseline 0: 決定表 + §66.5 + §66.7）/ `rg -c 'returnTo' docs/function-design/66-ui-stock-movements.md` ≥ 4（baseline 0）/ `rg -c 'UI-06a-D7' docs/function-design/58-ui-stock-inquiry.md` ≥ 3（baseline 0: `:538` bullet + §58.10 見出し + 変更履歴）
- **AC5**（負の oracle）`git diff --name-only origin/main..HEAD -- src/routes src/lib src/features/stock-inquiry/hooks src/features/stock-inquiry/StockInquiryPage.tsx src/features/inventory-records src-tauri docs/design-system docs/function-design/65-inventory-record-traceability.md docs/decision-log.md | wc -l` = 0
- **AC6** mutant: (1) S3 の `search={{ returnTo }}` を外す → T1 FAIL（`?returnTo=` 不在）。(2) S2 の `normalizeReturnTo(...)` を `search.returnTo ?? fallback` に置換 → T3 / T4（外部 URL・`//`）FAIL。(3) S2 の fallback の `search` から `q` を外す → T2（欠落 case）FAIL。(5)（GA5）fallback を文字列 `/stock?q=${code}&selected=${code}` に戻す → T2-num（数字だけの商品コード）FAIL。(4) S2 の `returnToParams.set("returnTo", ...)` を外す → T5 FAIL。5 本とも実装後に kill を実測して報告する（(1)〜(4) は run 2 で kill 済み、修正 run では (3) と (5) を再実測）
- **AC7** 対象 test 3 file（`StockMovementsPage` / `StockDetailContent` / `useStockInquiry`）全 PASS（本数は PR body / CI 出力を正とする）、`npm run typecheck` / `lint` / `format:check` PASS、最終 `bash scripts/local-ci.sh full` PASS（traceability 検査は S7 の再生成後に通る、GA3）。既存 `StockDetailContent.test.tsx:33-48` は S3 で href が変わるため必ず FAIL する。修正前後の red / green を報告する
- **AC8** `bash scripts/doc-consistency-check.sh --target plan` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS
- **AC-L3-1** Windows native: 在庫照会で任意の keyword を検索 → 商品行を展開 → 「在庫変動履歴」→ 「在庫照会へ戻る」→ 同じ keyword と展開行が戻る。続けて **入出庫履歴 → 任意の記録の「詳細を見る」→ 詳細の「在庫変動履歴」**（`returnTo` なしの入口。Tauri には address bar が無いため「直打ち」は不可、GA5 で訂正）→ 「在庫照会へ戻る」→ その商品コードで検索され展開された在庫照会。**数字だけの商品コード**で行う。PASS/FAIL のみ

## Design Sources

- Requirements / spec: REQ-303（商品ごとの在庫変動履歴）、REQ-301（在庫照会）、REQ-207（在庫変動履歴と業務記録の相互参照）
- Architecture: 該当なし
- Function / command / DTO: 66 §66.1 / §66.2 UI-06c-D1・D2（search params の限定、本 lane で `returnTo` を追加）/ §66.3 / §66.4 / §66.5 / §66.7。58 §58.4（`selected` ライフサイクル、検索前 clear、不変）/ §58.7 `StockDetailContent`（`:531-538`）/ §58.10 UI-06a-D6 まで
- DB: 該当なし
- Screen / UI: `docs/design-system/01-decision-rules.md` DSR-15（redirect 系 param の検証）/ DSR-18（戻り導線契約、共通 helper の最低基準。不変）
- Decision log / ADR: なし（D-D4）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | not applicable |
| Command / DTO / generated binding / wire shape | なし | not applicable |
| DB / transaction / audit / rollback / migration | なし | not applicable |
| Screen / UI / route state / Japanese wording | 66 §66.1 / §66.2 UI-06c-D9 / §66.3 / §66.4 / §66.5 / §66.7、58 §58.7 / §58.10 UI-06a-D7 / 変更履歴 | updated in this PR（S5 / S6） |
| CSV / TSV / report / import / export format | なし | not applicable |
| Durable decision / ADR | DSR-15 / DSR-18（既存の helper と検証規範） | existing sufficient |

## Registration / Generation Obligations

command / route / doc 新設なし、bindings 非接触、`generate:routes` 不要（route file 非接触、search schema は `types.ts` の import）。**該当あり（GA3）**: S4 で REQ 番号付き test（REQ-301 / REQ-303 / REQ-207）を追加・反転するため `docs/function-design/90-traceability.md`（AUTO-GENERATED）の coverage 行が変わる → 実装 run で `cargo run --bin generate_traceability` を実行し再生成差分を commit する（生成 file の再生成は本 wave では ㉖ だけ。㉗ は docs-only）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-301 / REQ-303 | 58 §58.7 / §58.10 | UI-06a-D7（D-D1） | 在庫照会の元条件を `returnTo` で運ぶ。捨てた案: 受け側ガード撤去 / 商品コード検索固定 | S3 / S6 | T1 |
| REQ-303 | 66 §66.2 / §66.5 | UI-06c-D9（D-D2） | `normalizeReturnTo` + 商品コード検索 fallback。捨てた案: `/stock` / 現行維持 | S1 / S2 / S5 | T2' / T2 / T3 / T4 |
| REQ-303 / REQ-207 | 66 §66.2 / §66.3 | UI-06c-D9（D-D3） | 業務記録詳細往復で `returnTo` を入れ子で保つ。捨てた案: sessionStorage | S2 | T5 / T6 |
| — | DSR-15 / DSR-18 | D-D4 | route file / helper / DSR-18 本文は不変、decision-log 追加なし | — | AC5 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（66 UI-06c-D9 と 58 UI-06a-D7 に決定・理由・棄却案・NAV-1 起源を置く）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-D1 → 58 UI-06a-D7（S6）、D-D2 / D-D3 → 66 UI-06c-D9（S5）
- Assumptions and constraints: `returnTo` の検証は既存 helper（`/` 始まりかつ `//` 始まりでない）。TanStack Router の `Link to={string with query}` が query を保った href を出すことは `DisposalRecordDetailPage.tsx:80` + `DisposalRecordDetailPage.test.tsx:120-141`（実 router）で実証済み。`useRouterState` の `location.href` は `DisposalPage.tsx:134` で実用済み
- Deferred design gaps, risk, and follow-up target: 業務記録詳細 6 画面から在庫変動履歴へ来た場合は fallback（商品コード検索）で在庫照会へ戻る。「元の業務記録詳細へ戻す」導線は label 変更を伴う別 scenario で Non-scope。廃番商品は fallback 後に list に出ず `selected` が解除される（§58.3 の `is_discontinued: false` 契約、既存）
- Test Design Matrix can cite design decision IDs or source doc sections: yes
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: `returnTo` 欠落・不正は必ず fallback（T2〜T4）。既存 deep link `/stock/<code>/movements`（`returnTo` なし）は fallback で従来より良い着地になり互換。受け側の検索前ガードは不変で、`returnTo` に `q` が無い URL（在庫照会で検索前に手打ちした URL）へ戻った場合は従来どおり `selected` が解除される（AC5 で hook 非接触を機械検査）

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable（UI 内の route/search 契約のみ） | — |
| Fact check / design decision split | 事実 = 起票時実測（producer 1 site、入口 7 site、受け側ガード、helper、実 router test の直列化形）。判断 = D-D1〜D-D4 | 本 packet |
| Lifecycle / retry | `returnTo` は URL のみに持つ。F5 で保持、filter / page / reset で保持（`...prev` spread）、業務記録詳細往復で入れ子保持。欠落・不正は fallback | Matrix State Lifecycle |
| Operator workflow | 検索 → 展開 → 履歴 → 戻る、の往復で条件と選択が残る。業務記録詳細の在庫変動履歴 link 経由（`returnTo` なしの入口）でも商品を見失わない | AC-L3-1 |
| Replacement path | not applicable | — |
| Data safety / evidence | 表示・遷移のみ。L3 は demo seed | Data Safety |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | 実 router 配線（route 解決 + 実 navigation）は unit test（memory history）で href まで、実遷移は L3 | AC-L3-1 |
| 環境・再現性 | URL: `/stock/<code>/movements?returnTo=...` は既存 route + 追加 search param。`max(500)` は業務記録詳細 route と同値 | — |

## Design Readiness

- Existing design docs are sufficient because: DSR-15 / DSR-18 が `returnTo` の検証・fallback の最低基準と共通 helper を定め、consumer 側の同型実装（業務記録詳細 6 画面）と実 router test が存在する。66 / 58 は「`/stock?selected=$code` へ戻る」「`returnTo` なしの遷移」を局所契約として固定しており、その 2 箇所の改訂（UI-06c-D9 / UI-06a-D7）が本 lane の design 出力
- Source docs updated in this PR: 66 §66.1 / §66.2 / §66.3 / §66.4 / §66.5 / §66.7（S5）、58 §58.7 / §58.10 / 変更履歴（S6）。実装 run で更新。本 plan-first commit は packet / Matrix / Plans.md / backlog.md のみ
- Design gaps intentionally deferred: 業務記録詳細からの入口で元の記録詳細へ戻す導線（Non-scope）
- Durable decisions discovered in this plan and promoted to source docs: UI-06c-D9（66）、UI-06a-D7（58）

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI のみ（search param 1 つ + link 2 箇所）
- Backend function design: 非接触
- Command / DTO / data contract: 非接触。URL search param `returnTo` を追加（Boundary / Wire Contract）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: label「在庫照会へ戻る」は不変
- Error, empty, retry, and recovery behavior: `returnTo` 欠落・不正は商品コード検索 fallback。廃番商品は既存契約どおり list 外
- Testability and traceability IDs: REQ-301 / REQ-303 / REQ-207、UI-06c-D9 / UI-06a-D7、REQ 新設なし

## Contract Probe

- TanStack Router `Link to={文字列（query 付き）}` が query を保った href を出す: 既存 consumer `DisposalRecordDetailPage.tsx:80` と実 router test `DisposalRecordDetailPage.test.tsx:120-141`（`"/stock/DP-001/movements?type=disposal&page=2"` がそのまま href になる）で実証済み -> 新規 probe 不要
- `search={{ returnTo }}` の直列化形: `StockMovementsPage.test.tsx:117-121`（実 router、`%3F` `%3D` `%26`）で実証済み -> T1 の期待値はこの形で書く

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| UI-06a-D7（在庫変動履歴 link の `returnTo` 送信、両描画経路） | S3 / S6 | T1（`StockDetailContent` 直接 render。`StockDetailCard` は同 component を包むだけで既存 `it.each` が両者を render） | AC-L3-1 |
| UI-06c-D9（`returnTo` 正規化、有効値へ戻る） | S1 / S2 / S5 | T2' 有効 case | AC-L3-1 |
| UI-06c-D9（欠落・不正は `/stock?q=<code>&selected=<code>` fallback） | S2 / S5 | T2 / T3 / T4 | AC-L3-1（業務記録詳細の在庫変動履歴 link 経由） |
| UI-06c-D9（`detailReturnTo` への入れ子） | S2 / S5 | T5 | — |
| UI-06c-D2（filter / reset で `returnTo` を落とさない） | 既存 `...prev` spread | T6 | — |
| UI-06c-D1 / D2（既存 search params 4 key の挙動不変） | 非接触 | 既存 `StockMovementsPage.test.tsx:87` `:147` `:313` | — |
| §58.4 受け側ガード不変 | 非接触 | 既存 `useStockInquiry.test.tsx:733` `:768` | AC5 |
| 業務記録詳細 6 画面の link 不変 | 非接触 | AC5（負の oracle） | — |

## Test Plan

Matrix: `docs/plans/test-matrices/2026-09-16-stock-movements-return-selected.md`。

- targeted tests: S4 を実装と同 commit で更新（AC3 / AC7、`StockDetailContent.test.tsx:33-48` の red → green）
- negative tests: mutant 4 本（AC6）。`returnTo` 欠落・外部 URL・`//` の fallback（T2〜T4）
- compatibility checks: 既存 `StockMovementsPage.test.tsx:87-123`（`returnTo` なしの `detailReturnTo` は不変）、AC5
- data safety checks: Data Safety 参照
- main wiring/integration checks: 両 test file は `renderWithRouter`（実 TanStack Router + memory history、`src/test/render-with-router.tsx`）で描画し、href は実 router の直列化で決まる。route matching と実 navigation は helper の対象外なので、在庫照会に戻った後の展開復元は L3（AC-L3-1）が担う

## Boundary / Wire Contract

browser state（`returnTo` search param）を扱うため記入する。

- producer: `StockDetailContent` の「在庫変動履歴」`Link`（本 lane で追加）。業務記録詳細 6 画面は producer にしない（fallback）
- consumer: `/stock/$code/movements` route → `StockMovementsPage`（`normalizeReturnTo`、本 lane で追加）
- wire type: URL search param `returnTo`（string、URL-encoded href、`max(500)`）
- internal type: `string | undefined`（`StockMovementsSearch.returnTo`）
- precision/range: `max(500)` 超過・非 string は zod `.catch(undefined)` で `undefined` → fallback
- round-trip path: `/stock?q=..&status=..&dept=..&page=..&selected=<code>` → `/stock/<code>/movements?returnTo=<encoded>` →（元記録 link）`/inventory/<kind>/records/<id>?returnTo=<encoded /stock/<code>/movements?...&returnTo=...>` → 「前の画面へ戻る」→ 在庫変動履歴（`returnTo` 保持）→ 「在庫照会へ戻る」→ 元の `/stock` URL
- invalid input: 欠落・`/` 始まりでない・`//` 始まりは `/stock?q=<code>&selected=<code>` へ、`Link` の `search` props（`{ q, selected }`）で組む（GA5）
- compatibility: additive（`returnTo` なしの既存 deep link は fallback、既存 4 key は不変）

## Review Focus

- T1 の `initialPath` と期待 href の直列化（`%3F` `%3D` `%26`）が実 router の出力と一致するか。D-D3 の入れ子 `returnTo` が `max(500)` に収まるか（典型 URL で 200 字弱、Matrix Boundary で概算）。fallback を `Link` の `search` props（`{ q, selected }`、GA5）で組む変換の漏れ。`useRouterState` を `ActiveCta` の内側で呼ぶことで `StockDetailContent` が router context 外で描画されないか（現行の 2 経路とも `/stock` route 内、`renderWithRouter` の test も router 内）。66 / 58 の同期漏れ（AC4）

## Spec Contract

Contract ID: SPEC-UI06C-D9-R1

- 在庫照会の詳細（インライン展開・フォールバックカード）の「在庫変動履歴」link は、現在の `/stock` URL（search state 込み）を `returnTo` として `/stock/$code/movements` へ送る（Test: T1）
- 在庫変動履歴の「在庫照会へ戻る」は `returnTo` が `/` 始まりかつ `//` 始まりでないときその URL へ戻る（Test: T2'）
- `returnTo` が欠落・不正のとき `/stock?q=<商品コード>&selected=<商品コード>` へ戻る（Test: T2 / T3 / T4）
- 在庫変動履歴から業務記録詳細へ送る `returnTo` は、在庫変動履歴自身が受け取った `returnTo` を入れ子で含む（Test: T5）
- filter 変更・reset・page 送りで `returnTo` を落とさない（Test: T6）
- 受け側 `useStockInquiry` の検索前ガードと `selected` ライフサイクルは不変（Test: 既存 `useStockInquiry.test.tsx:733` `:768`、evidence: AC5）

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-UI06C-D9-R1（producer） | S3 / S6 | T1 | 直列化形、両描画経路 | AC2 / AC3 / AC4 |
| SPEC-UI06C-D9-R1（正規化 / fallback） | S1 / S2 / S5 | T2 / T2' / T3 / T4 | `Link` の `search` props（GA5）、helper 経由 | AC1 / AC3 / AC4 |
| SPEC-UI06C-D9-R1（入れ子） | S2 | T5 | `max(500)` | AC1 |
| SPEC-UI06C-D9-R1（filter で保持） | 既存 spread | T6 | — | Matrix |
| SPEC-UI06C-D9-R1（受け側不変） | — | 既存 hook test | 非接触 | AC5 |

## Data Safety

- 実店舗データ・実商品名・実 JAN を test / packet / PR に含めない（test は既存 synthetic fixture `BT0002` 等）
- L3 は開発 DB（demo seed）。本番 DB / backup を使わない
- local-only / synthetic-only path の追加なし

## Implementation Results

[PR #75](https://github.com/higekiri-w/inventory-system-desktop/pull/75) で実装し squash merge 済み（`6f7928ed`、2026-09-17）。Writer = Codex 発注書 60 run 1〜3（run 1 = AC4 の削除 oracle の regex が新記述にも一致して fail-closed、run 2 = 実装 `3495e85e` / docs `081b1303` 完了後に `local-ci.sh full` の traceability 検査で停止、run 3 = 発注書の再開版に残っていた着手条件で停止）+ Sonnet subagent（GA4、90-traceability 再生成 `9e8d02be`）。GA5 修正 = Codex 発注書 61（実装 `60865ddf` / 90 再生成 `8192c2db`）、66 の記述同期 = Sonnet（`4b48cd38`）。

Final Review round 1（head `9e8d02be`）= Sonnet pass A / Opus pass B とも P1/P2 0。owner L3 round 1 = (a) PASS / (b) FAIL（数字だけの商品コードで文字列 fallback が `parseSearch` により number 化し検索前状態へ戻る）→ Gated Amendment 5。Final Review round 2（head `8192c2db`）= Sonnet / Opus とも P2 1（66 の fallback 記述が GA5 前のまま drift）→ `4b48cd38` で是正 → closure pass。owner L3 round 2 = (a)(b) PASS、round 2 の結果を `4b48cd38` の確認として owner が承認。

介入 3 / 予算 3、Codex relay 4 / 上限 4（GA5 で改訂）。証跡の所在: mutant 証跡 `.local/codex-orders/evidence-60-run2-081b1303/`（ignored、本体 checkout のみ）、full の log は各 run の報告に記録。test 本数・exact HEAD は転記しない（D-035/D-038 Evidence Ownership）。

## Review Response

Fill after review.
If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
- Findings Freeze: frozen after round 2 broad（head `8192c2db`）; post-freeze exceptions: none.

### Plan Review round 1（2026-09-16、plan-gate、Sonnet、裁定 Coordinator）

- reviewer 実施: plan-first commit `93d2227c` の差分が docs 4 file のみ / AC1〜AC5 の baseline command を逐語実行し全件一致 / file:line を実物と突合し一致 / D-D1〜D-D4 と DSR-18 の判定フロー（No 分岐）・§58.4 ガードの整合を hook 実物で確認 / 隔離 probe test（未追跡、検証後削除）で `renderWithRouter` + `useRouterState` が `initialPath` を返し、`search={{ returnTo }}` の href が packet の期待値 `/stock/BT0002/movements?returnTo=%2Fstock%3Fq%3DBT%26selected%3DBT0002` と一致することを実測 / `routeTree.gen.ts` が search schema を複製しないことを確認（`generate:routes` 不要は正）/ 対象 3 test file 全 PASS
- P3（Design Intent Trace の UI-06c-D9〈D-D2〉行の Test target に `T2'` が無い）= accept → 本 commit で `T2' / T2 / T3 / T4` へ
- P3（Scope S4 の it.each 記述が Matrix の 4 ID 表記とずれる）= accept → 本 commit で `T2' / T2 / T3 / T4` の表記へ
- 同 commit で Writer を Codex → Sonnet subagent へ変更（owner 不在の「早速始めよう」指示に合わせ Codex relay を挟まない。Plan Reviewer とは別 fresh context、独立性制約は維持）。P3 のみのため reviewer 再投入なし（Subagent Budget）
- 判定: P1/P2 = 0、**Plan Gate 通過可**。Plan Commit = 本 commit（plan-first `93d2227c` → 是正を含む確定版）

### Gated Amendment 1（2026-09-16、Writer を Codex へ戻す）

- 経緯: Plan Review round 1 の是正 commit `bbec3bf5` で Writer を Codex → Sonnet subagent に変え、遷移 `907beb5f` の後に Sonnet subagent の実装 run を起動した。owner が「Codex に回したほうが質良い」と指示したため run を停止（commit 0、mutant (3) 注入中の作業 tree は `git stash`「sonnet-writer-partial-㉖-2026-09-16」へ退避、worktree は clean に戻した）
- 是正: Writer field を Codex（発注書 60、owner 起動）へ。Scope / 設計判断 / AC / Matrix は不変。発注書 60 の HEAD_SHA は本 GA の登録 commit にする
- Owner Effort Budget: relay 0/2（本 GA は Codex run 前）。介入 1/3（本指示を decision point として計上）
- 教訓: owner 不在でも Writer の既定は Codex（発注書を用意して起動待ちにする）。Sonnet への切替は relay 上限到達か owner 指示のときだけ

### Gated Amendment 2（2026-09-16、Codex 発注書 60 run 1 の fail-closed 停止）

- run 1（HEAD `6edd7da6`）: Writer は AC1〜AC5 の baseline を逐語実行して全件一致を確認した上で、S5 の新 fallback 記述 `/stock?q=$code&selected=$code` が AC4 の regex `selected=\$code` に一致して「= 0」と両立しないことを検出し、file 編集前に停止（正しい挙動）。commit 0、worktree 除去済み、stash 非参照
- 原因: Coordinator の AC 設計誤り（削除 oracle の regex を旧 URL 全体に限定していなかった）
- 是正: AC4 の regex を `/stock\?selected=\$code`（旧 URL 限定）へ。baseline は同じ 2。Scope / 設計判断 / 他の AC / Matrix は不変
- Owner Effort Budget: relay 1/2 を消費（Coordinator 起因）。発注書 60 改訂 2 は本 GA の登録 commit を HEAD_SHA にする
- 教訓: 削除 oracle は「消す文字列そのもの」ではなく「新しい記述に一致しない形」で書く。起票時に新記述の例文へ同じ regex を当てて 0 件になることを確認する

### Gated Amendment 3（2026-09-16、Codex 発注書 60 run 2 の AC7 停止）

- run 2（HEAD `078fbb00`）: Writer は S1〜S6 を実装（`3495e85e` 実装 + test、`081b1303` docs）、AC1〜AC6 / AC8 PASS、red → green、mutant 4/4 kill まで完了し、`local-ci.sh full` の traceability 検査（`90-traceability.md` の再生成差分）で停止。packet が「生成義務なし」で 90 を Scope 外にしていたため正本不一致として変更せず停止（正しい挙動）。2 commit は branch に保持、push / PR 未実施、worktree 除去済み、証跡 `.local/codex-orders/evidence-60-run2-081b1303/`
- 原因: Coordinator の起票誤り。REQ 番号付き test の追加・反転は `90-traceability.md`（AUTO-GENERATED）の coverage 行を変える（feedback memory「REQ 追加時は 90 再生成を scope に」の同型。新規 REQ でなくても test の増減で変わる）
- 是正: Registration / Generation Obligations を「該当あり」に、S7（90 の再生成 commit）を追加、AC7 に注記。Scope S1〜S6 / 設計判断 / AC oracle / Matrix は不変
- Owner Effort Budget: relay 2/3（上限 2 → 3、Coordinator 起因）。発注書 60 改訂 3 は本 GA の登録 commit を HEAD_SHA にし、残作業 = 再生成 commit + full + Draft PR
- 教訓: 起票時に「追加・反転する test に REQ 番号があるか」を確認し、あれば 90 の再生成を Scope と生成義務に置く

### Gated Amendment 4（2026-09-17、Codex 発注書 60 run 3 の着手条件停止、relay 3/3）

- run 3（HEAD `e9e4e627`）: 発注書改訂 3 に「着手前に AC1〜AC5 の起票時 baseline と一致を確認、不一致なら停止」が残っており、実装済み HEAD では一致しないため Writer は編集前に停止（正しい挙動）。実装後の AC1〜AC5 は PASS を報告。commit 0、worktree 除去済み
- 原因: Coordinator の発注書改訂誤り（実装前の着手条件を実装後の run に残した）。packet の記述誤りではない
- 是正: relay 3/3 到達につき、残作業（S7 = `cargo run --bin generate_traceability` の再生成 commit / `local-ci.sh full` / doc check / Draft PR）を Sonnet subagent に移し Writer field へ併記（㉕ GA3 と同経路）。Codex の実装 2 commit は不変。Scope / AC / Matrix は不変
- Owner Effort Budget: relay 3/3。介入は増えない（Draft PR 後の L3 が 2 回目）
- 教訓: 発注書を「再開版」に改訂するときは、着手条件・手順・検証の各節を実装後の状態に合わせて全部書き直す（前提の一文を残さない）

### Gated Amendment 5（2026-09-17、owner L3 round 1 の (b) FAIL）

- L3 round 1（PR #75 head `9e8d02be`、介入 2/3）: (a) 検索 → 展開 → 履歴 → 戻る = **PASS**。(b) 入出庫履歴 → 廃棄・破損詳細 → 在庫変動履歴 → 「在庫照会へ戻る」= **FAIL**（在庫照会が検索前の空状態。商品コード `2099000000019`〈数字のみ〉）
- 原因: S2 の Coordinator 指示どおり fallback を文字列 `/stock?q=${code}&selected=${code}` で組んだため、TanStack Router の `parseSearch`（`@tanstack/router-core` `searchParams.js`、各値を `JSON.parse`）が数字だけの商品コードを number にし、route の `z.string().catch(undefined)` で `q` / `selected` とも `undefined` → 受け側の検索前ガードで空状態。アプリ自身の navigate は `stringifySearch` が JSON に見える文字列を引用符付きで書くため無事。unit test（T2〜T4）は `BT0002` のみで検出できず、Opus pass B の probe も英字コードだった
- 是正: S2 を `search` props で組む形へ、AC6 に mutant (5)、Matrix に T2-num（数字だけの商品コードで href が引用符付き〈`%22…%22`〉になり、`parseSearch` で string に戻る）を追加、AC-L3-1 (b) を記録詳細経由に訂正（Tauri に address bar は無い）。Scope の file 集合と Spec Contract は不変
- Owner Effort Budget: relay 上限 3 → 4（修正 run = Codex 発注書 61）。介入 2/3（L3 round 2 で 3/3）
- broad 2 本（Sonnet / Opus、head `9e8d02be`）は本 GA で Amendments が変わるため、修正後 head で取り直す（㉔ の教訓）
- 教訓: route/search state を文字列で組まない（`Link` の `search` props か `stringifySearch` を通す）。fallback の test は数字だけの商品コードを含める

### Final Review round 1（2026-09-17、head `9e8d02be`）

- pass A（Sonnet、Contract Audit）: P1/P2 = 0（P3 2、いずれも evidence quality）。AC1〜AC5 逐語再実行一致、mutant 4 本再注入で kill、Workflow State / Amendments 整合を確認（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5700325983)）
- pass B（Opus）: P1/P2 = 0（P3 6、いずれも merge blocker ではない）。mutant 独立 6 本 kill、隔離 probe で `returnTo` 3 段往復・不正値拒否を実証。findings: diagrams 現況化・`return-to.ts` の `/\evil` 通過・入れ子長さ概算・PR body log 所在は closeout/backlog へ（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5700326393)）
- 裁定: 両 pass とも P1/P2 = 0 のため review = pass 可。P3 は closeout（本 commit）と backlog（returnTo 衛生 lane）へ振り分け

### owner L3 round 1（2026-09-17、head `9e8d02be`、介入 2/3）

- (a) PASS / (b) FAIL（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5700876468)）。原因・是正は Gated Amendment 5 に記録

### Final Review round 2（2026-09-17、head `8192c2db`、GA5 後の新しい broad）

- pass A（Sonnet、Contract Audit）: P1 0 / P2 1（66 の fallback 記述が GA5 前のまま drift）/ P3 3。mutant (5) 再現で T2-num のみ FAIL を確認、`@tanstack/router-core` ソースで root cause を裏取り。裁定: P2 は 66 の docs-only 再同期 commit（`4b48cd38`）で解消、P3 は closeout へ（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5701266797)）
- pass B（Opus）: P1 0 / P2 1（同件、docs SSOT drift）/ P3 5。実 routeTree probe で fallback 分岐 15 パターンを検証、Adjacent Pattern audit で他 site への波及なしを確認。P3 5 件はすべて backlog（returnTo 衛生 lane）へ（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5701297781)）
- 裁定: P2（両 pass 共通、66 の記述 drift）を accept し `4b48cd38` で是正。実装・test は不変のため review = pass 見込み、closure で確認

### owner L3 round 2（2026-09-17、head `8192c2db`、介入 3/3）

- (a)(b) とも PASS（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5701173911)）。GA5 の修正で round 1 (b) FAIL が解消

### Final Review closure（2026-09-17、head `4b48cd38`）

- Sonnet、fresh context、read-only: `4b48cd38` は 66 のみ（+4/−3）、round 2 の P2 = closed。round 1 / round 2 / L3 round 1〜2 の全 finding を通しで確認し未解消の merge blocker なし。新規 P3（PR body の HEAD 表記 / packet 旧 prose / backlog 実ファイリング未確認）はすべて closeout へ（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5701397542)）
- 判定: **closure pass 可**

### manual（owner L3）の確認（2026-09-17）

- L3 round 2 の結果を `4b48cd38` の確認として owner が承認。manual = pass として record（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/75#issuecomment-5701443037)）

### Closeout（本 commit、Findings 解消）

- Final Review round 1 pass B P3 3 件、round 2 pass A/B P3 合計 8 件（うち diagrams 現況化 / packet 旧 prose sweep / backlog 5 件起票は本 commit と後続 backlog entry で対応）、closure P3 3 件（PR body 表記は github 上の記録として現状保持、packet 旧 prose は本 commit で sweep、backlog 実ファイリングは本 commit で実施）を解消
