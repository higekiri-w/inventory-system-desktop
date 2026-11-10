# Plan Packet: sidebar viewport の scroll restoration 除外（復元対象を `<main>` に限定）

DSR-17 分類②④の実装（PR #24、archived）は `<main>` の復元を主目的に構築されたが、`@tanstack/router-core` 1.168.15 の document capture listener は sidebar の Radix `ScrollArea` viewport も含め、あらゆる scrolling element を positional CSS selector で cache する。これが wave 8 lane 1（PR #28）の L3 round 2〜3 で owner が実観測した sidebar 跳び（`scrollTop 100 → 0` / `0 → 100`、comment 5502166679）の直接原因であり、`docs/Plans.md` Backlog に「sidebar viewport の scroll restoration 除外（R3 候補、優先）」として記録されていた。本 packet はその runtime 是正。

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 1d9c577
- Amendments: none
- Coordinator: Claude Fable 5.1（main session、conductor）
- Writer: Codex（GPT-5.6、発注書駆動、worktree isolation）
- Plan Reviewer: Claude Sonnet 5 subagent（independent fresh context）+ Fable 裁定
- Final Reviewer: Claude Sonnet 5 subagent（independent fresh context）+ Coordinator mutation 独立再実測 + Fable 裁定
- Reviewed Content HEAD: 8b02cf3
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: owner Windows native L3（sidebar 跳びの再現手順で PASS 確認）

Phase 遷移記録（kickoff → spec-check → design → plan-draft → plan-gate、本 plan-first commit に同乗）: task scope と R3 判定は本 packet に記録。spec-check では PR #28 L3 round 2〜3 の owner 実観測（sidebar `scrollTop` の `100 → 0` / `0 → 100`）と Plans.md backlog の第一候補是正案（`data-scroll-restoration-id` 付与）を前提としたが、design phase として起票時に `node_modules/@tanstack/router-core/dist/esm/scroll-restoration.js` 1.168.15 を実読した結果、当該候補は不成立と判明した（下記「起票時実測」節）。是正方式を app 層 allowlist prune へ確定し、packet + Test Design Matrix を同 commit で commit して plan-gate に至る。

2026-09-02: Plan Review round 1（Sonnet subagent fresh context、router-core `router.js` の constructor 購読順と `emit` の Set 挿入順・`event.fromLocation` 付与を実読で独立再現）= P1 0 / P2 2 / P3 2、全件 accept、是正 commit `d7e9e4a`（新規 test ID を SP1〜SP5 へ改名、SP4 の別 file + sessionStorage 事前破壊 + precondition assert 方式を明記）。round 2（同 reviewer、diff 実読 + 全文再読）= P1/P2 = 0 / P3 2 + 既存 gap 1、全件 accept、是正 commit `9ef97ba`。Plan Gate 収束（介入 1/3 = 起票選定）。`plan-gate -> plan-approved -> implementing` を本 state-only commit で圧縮遷移、Plan Commit = plan-first commit `1d9c577`。

2026-09-02: Writer（Codex）が worktree isolation で content commit + Implementation Results 記録 commit を積み Draft PR #29 を作成（frontend gate / `cargo check --release` / L1 full RESULT=PASS、exact SHA と evidence は PR body を正とする）。Writer の mutation 自己検証は Codex 安全審査で注入不可となり Coordinator 裁定でスキップ（relay 2/2 消費）。Final Review round 1 = P1/P2 = 0 / P3 2、Coordinator mutation 独立再実測 X1〜X5 全 kill（Review Response 節）。`implementing -> local-verified -> independent-review -> human-confirm` を本 state-only commit で圧縮遷移（post-impl state-only 1/2）、Reviewed Content HEAD = Writer 最終 commit。次 = owner Windows native L3（介入 2/3）。

2026-09-02: owner Windows native L3（PR #29 comment 5508055961、run identity = human-confirm 遷移 HEAD、`L3_SYNC=PASS`）= 手順 1〜5 全 PASS（sidebar 途中 scroll からの sidebar 遷移 / 一覧→詳細→戻り / 主ナビ先頭表示 / アプリ再起動後の起動時 sweep）。補足観察の「詳細 route で sidebar active 表示が一時的に外れる」は既存の exact pathname active 契約どおりで scroll 位置は不変、本 change の対象外。介入 2/3 消費。`human-confirm -> ready-hosted-final` を本 state-only commit で遷移（post-impl state-only 2/2）、Ready 化・hosted final 確認・merge は Coordinator 代行（介入 3/3）。

2026-09-02: Coordinator が PR #29 を Ready 化、hosted final success（exact HEAD = ready-hosted-final 遷移 commit `5dd8a08`、PR head・remote ref・run head の三点一致成立、content は `8b02cf3`）、squash merge `2964c5e`（2026-09-02、Ready と merge は owner 委任で Coordinator 代行）。実績 = 介入 2/3（起票選定 + L3、Ready・merge は Coordinator 代行）・relay 2/2・post-impl state-only 2/2。closeout で archive へ移動。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

§5.5 は使わない。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
Risk Tiers R3 行のうち route/search state（全 route 遷移の scroll restoration cache 内容を app 層で書き換える）、operator workflow（sidebar 主ナビゲーション操作時の誤遷移 loop という全画面横断の操作感バグの是正）に該当。DB / POS CSV / Tauri command DTO / bindings / merge gate は変更しない（frontend router 層のみ、AC6 で bindings 差分ゼロを機械確認）。

## 起票時実測（2026-09-02、HEAD `b4a8da2` から分岐）

前提の現況実測（すべて Coordinator 実読・実行、`node_modules/@tanstack/router-core` **1.168.15** / `@tanstack/react-router` **1.168.23**、`src/lib/app-router.ts` 実読基準）:

- `src/components/layout/Sidebar.tsx` は `@/components/ui/scroll-area.tsx` の `ScrollArea`（Radix `ScrollAreaPrimitive.Viewport`）を使い、`data-scroll-restoration-id` を持たない。`<main>`（`RootLayout.tsx`）だけが `data-scroll-restoration-id="main"` を持つ（`rg -n 'data-scroll-restoration-id' src/` で確認、hit は `RootLayout.tsx` の 1 箇所のみ）。
- `document.addEventListener("scroll", onScroll, true)`（`scroll-restoration.js:104`）は capture phase で document 配下の任意の scroll イベントを拾い、`trackedScrollEntries`（Map、key = イベント発生要素）へ記録する。
- `onBeforeLoad` の `snapshotCurrentScrollTargets(restoreKey)`（`:90-103`）が `trackedScrollEntries` の各要素を `cache.state[restoreKey][selector] = position` へ書く。selector は `target.getAttribute("data-scroll-restoration-id")` があれば `[data-scroll-restoration-id="<値>"]`、なければ `getCssSelector(target)`（親を辿る positional selector、`:47-55`）。
- `onRendered`（`:113-182`）は `cache.state[cacheKey]` の**全 selector を無条件に** `document.querySelector(selector)` + `element.scrollLeft/scrollTop = position` で復元する（`:142-148`）。復元対象は `<main>` に限定されず、`applyMainNavScroll`（`src/lib/app-router.ts:15-21`、app 層の分類④ one-shot）や `getElementScrollRestorationEntry(... id: "main" ...)`（同 `:45-48`、D-E 遅延再適用）はいずれも `<main>` だけを個別に扱う app 層の**追加**処理であり、library 本体の全 selector 復元を止める仕組みではない。
- **`data-scroll-restoration-id` 付与は除外にならない（Backlog 記載の第一候補の反証）**: 属性を付けると selector が `getCssSelector` 由来の positional から `[data-scroll-restoration-id="..."]` に変わるだけで（`:96-98`）、`elementEntries` への記録と `onRendered` の復元対象からは外れない。除外には selector の種類ではなく「そもそも `cache.state` に残さない／復元させない」処理が要る。
- router option を rg した結果（`node_modules/@tanstack/router-core/dist/esm/router.d.ts:309-334`）、公開 API は `scrollRestoration | getScrollRestorationKey | scrollRestorationBehavior | scrollToTopSelectors` の 4 つのみで、per-element の除外オプションは存在しない。`scrollRestoration` の function 化は location 単位の全体 opt-out（`<main>` の分類②復元も失う）であり不採用。`getScrollRestorationKey` は set/get 双方で使われる対称 key のため、sidebar だけを別 key にする方法もない。
- `scrollRestorationCache`（`scroll-restoration.js:37`、`export { ..., scrollRestorationCache, ... }` を `index.d.ts:43` で re-export）は `@tanstack/router-core` の named export として公開されており、`@tanstack/react-router` からは re-export されない（`app-router.ts` は既に同モジュールから `getElementScrollRestorationEntry` を import している先例あり）。`.state` は plain object、`.set(updater)` は `functionalUpdate(updater, state) || state` を代入する non-nullable updater。sessionStorage が使えない環境（`getSafeSessionStorage()` 失敗）では `createScrollRestorationCache()` が `null` を返すため、**`scrollRestorationCache` は null であり得る**（`:14`）。

## Goal

Goal Invariant:

### 最小完了条件

route 遷移後、sidebar（chrome の主ナビゲーション領域）の scroll 位置は復元されない — 短い window で sidebar を scroll した履歴があっても、遷移直後に sidebar が跳んでポインタ下の項目が入れ替わることがない。同時に、DSR-17 分類②④（`<main>` の位置復元・主ナビ先頭表示・D-E 遅延再適用）は無変更で動作する。

### 失敗定義

- sidebar が遷移後になお跳ぶ（`scrollTop` が navigation を契機に変化する）。
- `<main>` の一覧→詳細→戻り復元（分類②、既存 T10）が regression する。
- 主ナビ先頭表示（分類④、既存 T4/T5/T13）または clamp 遅延再適用（D-E、既存 T12）が regression する。
- `HomePage.test.tsx` の negative test（UI-11b-D12）や `page-scroll.test.ts` が regression する。
- `scrollRestorationCache` が `null` の環境（sessionStorage 不可）で prune 呼び出しが例外を投げる。

### 非目的

- sidebar viewport（`Sidebar.tsx` / `scroll-area.tsx`）への `data-scroll-restoration-id` 付与によるコード変更（起票時実測で除外にならないと確定した候補、Non-scope へ記録）。
- `@tanstack/router-core` へのパッチや fork。
- router option（`scrollRestoration` / `getScrollRestorationKey` / `scrollToTopSelectors`）の変更。
- `getAppScrollRestorationKey` / `applyMainNavScroll` / DSR-17 (i) 遅延再適用ロジックの変更（allowlist prune はこれらと独立に効く後付け層とする）。
- `docs/UI_TECH_STACK.md` L403 の DSR 列挙 stale（別 backlog）、`app-router.ts` top-level singleton 副作用（PR #24 Final Review P3-2 backlog）の解消。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

1. **`pruneScrollRestorationEntries` の新設**（`src/lib/app-router.ts` のみ、新規 module 不要）: `scrollRestorationCache.state` から `[data-scroll-restoration-id="main"]` 以外の selector entry を削除する pure helper。引数 `key?: string` — 指定時はその key のみ、未指定時は全 key を対象にする。`scrollRestorationCache` が `null`（sessionStorage 不可環境）のときは no-op（null-safe）。
2. **起動時 sweep**: `createAppRouter` 内、`createRouter(...)` 呼出し直後に `pruneScrollRestorationEntries()`（全 key）を呼ぶ。sessionStorage から `pagehide` で永続化された既存 cache（是正前の汚染分を含む）を router 生成時点で一括除染する。
3. **`onBeforeLoad` 購読への追加**: 既存の `appRouter.subscribe("onBeforeLoad", (event) => { cancelDelayedRestoration?.(); ... })`（`app-router.ts:36-38`）に `if (event.fromLocation) pruneScrollRestorationEntries(getAppScrollRestorationKey(event.fromLocation));` を追加する。library の `setupScrollRestoration` は router constructor 内で先に `onBeforeLoad` を購読し（`router.js:118`、DSR-17 (g) で既に確認済みの順序保証）、`snapshotCurrentScrollTargets(fromLocation key)` を実行してから app の handler が呼ばれるため、app の prune は「library が直前に書いた fromLocation の snapshot」に対して確実に後から効く。
4. **契約 test 追加**: Matrix SP1〜SP5。既存 DSR-17 test（`app-router.test.tsx` T1/T4/T5/T9/T10/T11/T12/T13、`HomePage.test.tsx`、`page-scroll.test.ts`）の削除・無効化なし。SP4（null cache）は別 test file（例: `src/lib/app-router.null-cache.test.tsx`）に置く — `app-router.test.tsx` file scope で `vi.mock("@tanstack/router-core")` すると同一 file 内の real-cache regression test（T10/T12/T13）が壊れ、かつ Contract Probe P1b で `vi.resetModules()` は externalize された node_modules を file 内で再評価しないと判明済みのため。方式: file 先頭、`@tanstack/router-core` / `./app-router` の最初の import 前（setup 後の top-level `await import(...)` を使う）で `window.sessionStorage` を access 時に throw させる（`Object.defineProperty(window, "sessionStorage", { get() { throw new Error("sessionStorage disabled"); } })`）ことで、その file の isolated module context（`vitest.config.ts` は `isolate` 既定 `true`）で module 評価される `createScrollRestorationCache()`（`scroll-restoration.js:12-19`）が `null` を返すようにする。SP4 はまず `scrollRestorationCache === null`（同一 file で import）を precondition として assert し、成立しない場合は silent pass にせず test を FAIL させる（isolation が実際に null 環境を作ったことの証明）。その上で `createAppRouter()` と `pruneScrollRestorationEntries()` / `pruneScrollRestorationEntries("/stock")` が例外を投げないことを確認する。Fallback（pre-approved、amendment 不要）: この vitest 設定で precondition が成立しない場合、Writer は helper signature を `pruneScrollRestorationEntries(key?: string, cache = scrollRestorationCache)` へ変更し cache を注入可能な最終引数にして、SP4 は `null` を明示的に渡す。採用した経路は Implementation Results に記録する。
5. **DSR-17 docs 是正**（`docs/design-system/01-decision-rules.md`）:
   - Why（app 契約の背景）文の「`<main>`（...）が唯一の scroll container である」を「`<main>`（...）が route content の唯一の scroll container である」へ修正（289 行目付近）。sidebar の Radix `ScrollArea` viewport は chrome の scroll container であり、PR #28 L3 で復元される事実が実証済みのため、無限定の「唯一」は不正確。
   - Why（library 観測事実）文中の同型表現「唯一の scroll container `<main>` には効かない」も同一修正（同一節内での自己矛盾を避けるため、起票時実測で判明した事実を同じ pass で反映する）。
   - (c) 「唯一の scroll container `<main>`」も同様に修正。
   - library 観測事実の段落へ、document capture listener があらゆる scrolling element を positional selector で cache すること、`data-scroll-restoration-id` は selector の安定化であって除外ではないこと、per-element の除外 option が 1.168.15 に存在しないこと（`router.d.ts:309-334`）を追記する。
   - 新規契約 **(j) 復元対象を `<main>` に限定する** を追加（文面は下記「DSR-17 (j) 規範文（提案）」節）。
   - changelog table へ 1 行追加（既存行の形式に合わせる、ファイル末尾）。

## Non-scope

- `Sidebar.tsx` / `scroll-area.tsx` のコード変更。sidebar への `data-scroll-restoration-id` 付与。
- library patch、router option 変更。
- `getAppScrollRestorationKey` / `applyMainNavScroll` / DSR-17 (i) 遅延再適用ロジックの変更。
- `docs/UI_TECH_STACK.md` L403 の stale DSR 列挙是正（既存 backlog、本 packet と独立）。
- `app-router.ts` top-level router singleton 副作用の解消（PR #24 Final Review P3-2 backlog、本 packet と独立）。
- Tauri command / DTO / bindings / DB の変更。

## DSR-17 docs 是正（提案 verbatim、Plan Reviewer は本節の文面を精査対象にする — Writer は本節をそのまま `docs/design-system/01-decision-rules.md` へ適用する）

**Why（app 契約の背景）置換後の全文**（既存 1 文目のみ変更、残りは無変更）:

> 本アプリは persistent な `<main>`（`src/components/layout/RootLayout.tsx`、RootLayout 構成の正本は [52-ui-shared-layout.md §52.1](../../function-design/52-ui-shared-layout.md#521-コンポーネント構成)）が route content の唯一の scroll container である（sidebar の Radix `ScrollArea` viewport は chrome の scroll container であり、これとは別枠 — (j) 参照）。route 遷移で `<main>` は unmount されないため、scroll 位置を明示的に扱わないと stale scroll が全画面へ持ち越される。一方、mount 一律の先頭 scroll は一覧→詳細→戻りの位置を失わせることが PR #15 Amendment 2 で実証され、revert 済みである。操作結果の可視性、戻り導線の連続性、主ナビゲーションの予測可能な初期表示を両立するには、mount ではなく遷移の契機ごとに発火条件を分ける必要がある。

**Why（library 観測事実、TanStack Router 1.168.23）置換後の全文**（「唯一の scroll container」表現の修正 + 末尾に document capture listener の観測事実を追記）:

> 現行 app は `scrollRestoration` 未設定である。`@tanstack/react-router` 1.168.23 の型定義 JSDoc は既定 key を `location.href` と説明するが、実装既定は `location.state.__TSR_key || location.href` である。`__TSR_key` は history entry ごとに新規発行されるため、DSR-18 が維持する `<Link>` の push 戻りでは同じ href へ戻っても既存 cache key と一致せず、既定のままでは位置を復元できない。また cache miss 時の先頭 scroll は `window.scrollTo` と `scrollToTopSelectors` に委ねられ、既定 selector は `['window']` なので route content の唯一の scroll container `<main>` には効かない。cache は sessionStorage（`tsr-scroll-restoration-v1_3`）へ `pagehide` 時に保存され、library は `window.history.scrollRestoration = "manual"` を設定する。document capture listener（`document.addEventListener("scroll", ..., true)`）は `<main>` に限らずあらゆる scrolling element の scroll を拾い、`onBeforeLoad` 時にその位置を positional CSS selector（または `data-scroll-restoration-id` 属性 selector）で cache へ書き込む。`data-scroll-restoration-id` は selector を安定化するだけで cache 対象からの除外にはならず、1.168.15 の router option（`scrollRestoration | getScrollRestorationKey | scrollRestorationBehavior | scrollToTopSelectors`、`router-core/dist/esm/router.d.ts:309-334`）に per-element の除外手段は存在しない。sidebar の `ScrollArea` viewport がこの機構で cache・復元された実例が (j) の起源。これらは版数に依存する観測事実であり、下記の app 契約とは分離し、router 更新時と後続 R3 Contract Probe で再検証する。

**(c) 置換後の全文**（「唯一の scroll container」表現の修正のみ）:

> **(c) container を安定識別する**: route content の唯一の scroll container `<main>` に `data-scroll-restoration-id` を付与する。CSS 階層や class の位置に依存した cache key を使わない。

**新規契約 (j)**（(i) の直後、`禁止` 段落の直前に挿入）:

> **(j) 復元対象を `<main>` に限定する**: document capture listener は `<main>` 以外の scrolling element（sidebar の Radix `ScrollArea` viewport 等）も positional selector で cache し、`onRendered` は cache の全 selector を無条件に復元するため、`data-scroll-restoration-id` の有無に関わらず sidebar 等の chrome scroll container が route 遷移で復元されてしまう（PR #28 L3 round 2〜3 実観測、`scrollTop 100 → 0` / `0 → 100`）。app 層は router 生成直後（起動時 sweep）と各 `onBeforeLoad`（直前に離脱した location の snapshot に対して）の両方で、`[data-scroll-restoration-id="main"]` 以外の selector entry を `scrollRestorationCache` から削除する。sidebar の scroll 位置は業務上の route state ではなく chrome の一過性 UI 状態であり、保持・復元する対象にしない。副作用として、`<main>` を一度も scroll していない画面への遷移は cache miss のまま先頭表示になる（(d) の fallback が想定どおり機能する）。

**changelog table 追加行**（ファイル末尾 `## 更新履歴` 表の最上段へ Writer が実装 commit で追加する。日付列は実装 commit の日付、PR 列は Draft PR 番号、内容列は次の文とする）:

内容列: DSR-17 に (j)「復元対象を main に限定する」を新設。document capture listener が sidebar 等の chrome scroll container も cache・復元してしまう実挙動（PR #28 L3 実観測）に対し、app 層 allowlist prune で main 以外の cache entry を router 生成時と各 onBeforeLoad で削除する契約を追加。Why / (c) の「唯一の scroll container」表現を route content 限定へ訂正。

## Acceptance Criteria

- AC-L3（owner Windows native）: 短い window で sidebar を scroll → (i) sidebar の `<Link>` 経由の遷移、(ii) `<main>` 内の in-page link 経由の遷移、(iii) 「前の画面へ戻る」経由の遷移、いずれでも sidebar の `scrollTop` が遷移前後で変化しない。
- AC1（分類②維持）: 一覧→詳細→「前の画面へ戻る」で `<main>` の scroll 位置が復元される（既存 T10、regression-free）。
- AC2（分類④・D-E 維持）: sidebar 主ナビ再訪で `<main>` が先頭表示される（既存 T4/T5/T13）、および clamp 検出時の遅延再適用（既存 T12）が regression-free。
- AC3（(h) 維持）: `HomePage.test.tsx` の negative test（UI-11b-D12）と `page-scroll.test.ts` が無変更で green。
- AC4（fallback-suppression の明示）: `<main>` を一度も scroll していない画面への遷移が引き続き先頭表示になる（miss fallback、既存 T10 の miss case で被覆済みであることを確認）。
- AC5: `test-matrices/2026-09-02-sidebar-scroll-restoration-exclusion.md` の SP1〜SP5 が green、必須 mutation 注入 X1〜X5 が全 kill（Coordinator 独立再実測 + Final Reviewer 独立再実測）。
- AC6: `src/lib/bindings.ts` の diff ゼロ。
- AC7: frontend gate（`npm run typecheck` / `npm run lint` / `npm run format:check` / `npm test` / `npm run build`）green + `cargo check --release` PASS。
- AC8: DSR-17 (j) 追記・Why 訂正・changelog 行が反映され、`bash scripts/doc-consistency-check.sh` clean。

## Design Sources

- Requirements / spec: DSR-17 分類②④（scroll 契約 root）
- Architecture: 変更なし（UI 層内、router 生成 module 内の追加処理のみ）
- Function / command / DTO: `docs/function-design/52-ui-shared-layout.md` §52.1（`<main>` / Sidebar の構成正本）
- DB: 変更なし
- Screen / UI: `docs/design-system/01-decision-rules.md` DSR-17（(a)〜(i) 既存契約、(j) 本 packet で追加）
- Decision log / ADR: 変更なし（本 packet の是正結果は DSR-17 (j) へ正本化、Scope 5）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 変更なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 変更なし（AC6 で差分ゼロ機械確認） | existing sufficient |
| DB / transaction / audit / rollback / migration | 変更なし | existing sufficient |
| Screen / UI / route state / Japanese wording | DSR-17 (a)〜(i) + 新規 (j) | updated in this PR |
| CSV / TSV / report / import / export format | 変更なし | existing sufficient |
| Durable decision / ADR | DSR-17 (j) 追記（本 packet の是正結果の正本化） | updated in this PR |

## Registration / Generation Obligations

新規 Tauri command / route / function-design doc / operator 画面の追加はない。

| 新規追加物 | 登録・生成義務 | 対応 |
|---|---|---|
| 新規 export `pruneScrollRestorationEntries`（`src/lib/app-router.ts` 内、新規 module ではない） | — | test 同梱（Matrix SP1〜SP5）。route / command / doc 新設なし |
| REQ coverage | 該当なし — `src/lib/app-router.test.tsx` の既存 DSR-17 test（T1/T4/T5/T9/T10/T11/T12/T13）は DSR-17 決定 ID（D-A〜D-F）を cite する形式で REQ token を使っていない（`90-traceability.md` に DSR-17 / `app-router` の hit なし、`rg -n 'REQ-' src/lib/app-router.test.tsx` hit 0 を確認済み）。新規 Matrix SP1〜SP5 も同じ decision ID 引用の慣行（D-G）に従うため、`generate_traceability` 再生成は不要と判断する | 不要（結論確定） |
| route / operator 画面 / Tauri command | — | 該当なし |

**docs 内「唯一の scroll container」表現の sweep 結果**（`rg -n '唯一' docs --glob '!docs/archive/**'` の全 hit を確認。「唯一」の一般的な用法は他多数あるため、scroll container を指す文脈に絞って抽出）:

| ファイル:行 | 文脈 | 判定 |
|---|---|---|
| `docs/design-system/01-decision-rules.md:292` | Why（app 契約の背景）「`<main>`（...）が唯一の scroll container である」 | 誤り。Scope 5 で修正（route content 限定へ） |
| `docs/design-system/01-decision-rules.md:294` | Why（library 観測事実）「唯一の scroll container `<main>` には効かない」 | 同一節内の自己矛盾回避のため Scope 5 で同時修正 |
| `docs/design-system/01-decision-rules.md:319` | (c) 「唯一の scroll container `<main>`」 | 同上、Scope 5 で修正 |

`docs/function-design/52-ui-shared-layout.md` §52.1・`docs/UI_TECH_STACK.md`・`docs/quality/review-checklist.md`・`docs/design-system/README.md` は `唯一` または `<main>` を scroll container の排他性主張として使う箇所なし（52-ui-shared-layout.md §52.1 はファイル責務の列挙表のみ、review-checklist はカテゴリ 9 の DSR-17 分類要約のみで排他性主張なし）。以上より **修正対象は `01-decision-rules.md` 内の 3 箇所のみ**、他 doc への波及なしと判定する。README / review-checklist の DSR-17 索引は (j) の追加だけで足り、既存索引項目の文面修正は不要（索引はサブ項目名を列挙する形式で「唯一」表現を含まない）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| DSR-17 分類②④（sidebar 非復元） | 01 DSR-17 (j)（新設） | D-G | Backlog 記載の第一候補（sidebar へ `data-scroll-restoration-id` 付与）は起票時実測で不成立と確定（selector 種別が変わるだけで除外にならない、`scroll-restoration.js:96-98`）。per-element 除外 option は 1.168.15 に存在しない（`router.d.ts:309-334`）。採用は app 層 allowlist prune（`scrollRestorationCache` を直接書き換え、`<main>` selector だけ残す） | Scope 1-3 | SP1 / SP2 / SP3 |
| DSR-17 (j) null-safety | 01 DSR-17 (j) | D-G | `scrollRestorationCache` は sessionStorage 不可環境で `null`（`scroll-restoration.js:14`）。prune helper は null-safe に実装し、null 環境で例外を投げない | Scope 1 | SP4 |
| DSR-17 (b)(c)(d)(g)(i) 既存契約の非破壊 | 01 DSR-17 (b)(c)(d)(g)(i) | D-A〜D-F（既存） | prune は `<main>` selector を allowlist するだけで既存の復元パス自体を変更しないため、分類②④・D-E は無変更で動作する想定。regression test で確認 | 変更なし | 既存 T1/T4/T5/T9/T10/T11/T12/T13 |
| 01-decision-rules.md 「唯一の scroll container」表現の訂正 | 01 DSR-17 Why / (c) | Scope 5 | 排他的な「唯一」表現は sidebar が chrome scroll container として復元される実証済み事実と矛盾する。「route content の唯一」へ限定 | Scope 5 | doc-consistency-check + diff |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: DSR-17 (a)〜(i) の既存契約 + Scope 5 の (j) 追記で成立。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-G（採用機構＝app 層 allowlist prune）と起票時実測の判明事実（`data-scroll-restoration-id` 付与は除外にならない、per-element 除外 option 不在）を Scope 5 で DSR-17 (j) へ promote する。
- Assumptions and constraints: 版数依存の library 挙動（document capture listener の全 selector 復元、`scrollRestorationCache` の named export）は起票時実測節に file:line 付きで記録し、router 更新時の (f) 再検証対象へ含める。
- Deferred design gaps, risk, and follow-up target: なし（sidebar 除外は本 packet で完結、他の chrome scroll container〈将来追加され得る〉への一般化は実需発生時に再判断）。
- Test Design Matrix can cite design decision IDs or source doc sections: DSR-17 (j) / D-G を cite。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: prune が全滅しても是正前の挙動（sidebar が復元される既知バグ）へ戻るだけで、`<main>` の復元自体は影響を受けない（allowlist prune は既存復元パスと独立な後付け層）。null 環境では no-op となり例外で app を落とさない。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable — frontend router 層のみ | — |
| Fact check / design decision split | 起票時実測（node_modules 実読、file:line 付き）は観測事実。allowlist prune の採用は D-G の design decision | 本 packet |
| Lifecycle / retry | cache は sessionStorage（`pagehide` 書出し）。起動時 sweep が是正前の汚染分（旧版で保存された sidebar entry）も一括除染する | Scope 2 |
| Operator workflow | sidebar 短時間 scroll 履歴からの主ナビ操作で誤遷移 loop になる既知バグ（PR #28 L3 round 2〜3 owner 実観測）の解消 | AC-L3 |
| Replacement path | not applicable（外部システム非接触） | — |
| Data safety / evidence | scroll 座標・href のみ。業務データ非接触 | — |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | sidebar 跳びの視認品質と WebView2 実機の sessionStorage 挙動は自動 test で判別しきれない部分がある — L3 必須 | Human Gate |
| 環境・再現性 | test 環境は happy-dom（`vitest.config.ts:11`）。`scrollRestorationCache` は named export の module singleton で、Contract Probe（下記）で happy-dom 上の挙動を実測済み。実機 WebView2 差異は L3 で確認 | Contract Probe + L3 |

## Design Readiness

- Existing design docs are sufficient because: DSR-17 (a)〜(i) が分類②④の契約を既に確定済み。sidebar 除外という新しい契約だけが本 packet の起票時実測まで選定されていなかった。
- Source docs updated in this PR: `docs/design-system/01-decision-rules.md` DSR-17 に (j) を新設 + Why/(c) の「唯一」表現訂正。
- Design gaps intentionally deferred: 他の chrome scroll container（将来追加され得る）への一般化は実需発生時に判断。
- Durable decisions discovered in this plan and promoted to source docs: D-G（allowlist prune 機構）。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 変更なし（UI 層内、CMD/BIZ/IO 非接触）。
- Backend function design: 変更なし。
- Command / DTO / data contract: 変更なし（AC6 で機械確認）。
- Persistence / transaction / audit impact: 変更なし（sessionStorage のみ、DB 非接触）。
- Operator workflow / Japanese UI wording: sidebar 跳びバグの解消（AC-L3）。文言変更なし。
- Error, empty, retry, and recovery behavior: `scrollRestorationCache` null 時は no-op（AC5 / SP4）。
- Testability and traceability IDs: DSR-17 決定 ID（D-G）を Matrix が cite（REQ token 不使用、既存慣行どおり）。

## Contract Probe

登録漏れ是正を含む probe は、是正を仮適用した状態で end-to-end に実行する — 未登録状態のままの probe は、登録後に初めて顕在化する義務を検出できない。本 packet の premise は「library の実挙動」であり登録漏れ型ではないため、この規律は非該当（is-N/A）。

- **P1（起動時 sweep の可視性）**: `scrollRestorationCache.set(...)` で `/stock` key に `[data-scroll-restoration-id="main"]` + positional selector（`"div:nth-child(2) > nav:nth-child(1)"`）の 2 entry を直接 seed → 同一 test 内で `createAppRouter()` を呼び、`scrollRestorationCache.state["/stock"]` が seed 直後の singleton と同一インスタンスであることを確認 → **PASS**（同一 module singleton を読む限り、起動時 sweep は seed した entry を確実に見える。実測は `src/lib/__probe_scroll_prune.test.tsx`〈削除済み、一時 probe〉の "P1" test）。
  - 追加実験 **P1b（sessionStorage 直接 seed + `vi.resetModules()` + dynamic import での module 再評価）**: `sessionStorage.setItem("tsr-scroll-restoration-v1_3", ...)` → `vi.resetModules()` → `await import("@tanstack/router-core")` で singleton が sessionStorage を再読込みするかを検証 → **FAIL（技術として不成立と判明）**: この test file 内で `@tanstack/router-core` が既に他 test 経由でロード済みの場合、vitest の `resetModules()` は外部 npm package（node_modules 配下、非 project source）の singleton まで再評価しない（vitest のデフォルト externalization により、`node_modules` 配下は module registry reset の対象外になる）。**結論**: SP1（Matrix）は `scrollRestorationCache.set(...)` による直接 seed 方式を採用する（sessionStorage 経由の起動時再現ではなく、`createAppRouter()` 呼出し時点で `scrollRestorationCache.state` に何が入っているかだけを検証すれば起動時 sweep の契約は十分に検証できる — 本番の module 初期化順序は「sessionStorage 読込みは module import 時に 1 回」で決定的であり、その後 `createAppRouter()` が読む対象は常に同一 singleton）。
- **P2（subscriber 順序 + `event.fromLocation`）**: `createAppRouter` で router を起動し、`<main>` と sidebar 相当の attribute-tagged 要素（`data-scroll-restoration-id="sidebar"`）を用意 → sidebar 側へ scroll event を dispatch → 別の app `onBeforeLoad` 購読を登録し、その中で `scrollRestorationCache.state[fromHref]?.['[data-scroll-restoration-id="sidebar"]']` を観測 → navigate 実行 → **PASS**: 観測値は `{ scrollX: 0, scrollY: 77 }`（seed 値どおり）— library の `snapshotCurrentScrollTargets` が app の `onBeforeLoad` handler より確実に先に entry を書き込んでいることを確認（Set 挿入順、library が router constructor 内で先に subscribe する既存事実〈DSR-17 (g) 節で確認済み〉と整合）。`router.navigate({ href })` は `onBeforeLoad` を `fromLocation` 付きで確実に emit する（`fromLocationSeen === true` を確認）。
- **P3（`data-scroll-restoration-id` 付与は除外にならないことの実証）**: P2 と同じ harness で、sidebar 要素に `data-scroll-restoration-id="sidebar"` を付与した状態のまま navigate → `scrollRestorationCache.state["/stock"]["[data-scroll-restoration-id=\"sidebar\"]"]` が `{ scrollX: 0, scrollY: 77 }` として cache に残ることを確認 → **PASS（除外にならないことの実証成功）**。Backlog 記載の第一候補が起票時実測（node_modules 静読）と実行時実測（本 probe）の両方で不成立と確定。
- **P4（`scrollRestorationCache.set` による削除の可視性）**: `.set(() => ({ "/a": { main: {...}, positional: {...} } }))` で 2 entry を seed → `.set((s) => { delete s["/a"]["positional > selector"]; return s; })` で削除 → `.state["/a"]` が main entry のみになることを確認 → **PASS**（`functionalUpdate` は updater の戻り値をそのまま新 state として代入するため、mutate-and-return パターンで削除が確実に反映される。`pruneScrollRestorationEntries` の実装方式として採用）。

実行コマンド: `npx vitest run src/lib/__probe_scroll_prune.test.tsx`（4 test 中 P1/P2+P3（1 test に統合）/P4 が PASS、P1b のみ上記の理由で FAIL — 実装方針の反映後に probe file は削除済み、`git status --short` で `src/` 配下の残置ゼロを確認済み）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| DSR-17 (j) 起動時 sweep（全 key の非 main entry を削除） | `app-router.ts` `createAppRouter` 内の `pruneScrollRestorationEntries()` 呼出し | SP1 | — |
| DSR-17 (j) `onBeforeLoad` prune（`fromLocation` key のみ） | `app-router.ts` 既存 `onBeforeLoad` 購読への追加行 | SP2 | — |
| DSR-17 (j) `<main>` entry の保存（allowlist） | `pruneScrollRestorationEntries` の allowlist 判定 | SP3 | AC-L3 |
| DSR-17 (j) null-safety | `pruneScrollRestorationEntries` の `scrollRestorationCache` null guard | SP4 | — |
| DSR-17 (b)(c)(d) 分類②既存契約の非破壊 | 変更なし（既存 T1/T9/T10） | SP5（regression 実行） | — |
| DSR-17 (g) 分類④既存契約の非破壊 | 変更なし（既存 T4/T5/T13） | SP5（regression 実行） | — |
| DSR-17 (i) clamp 遅延再適用の非破壊 | 変更なし（既存 T12） | SP5（regression 実行） | — |
| DSR-17 (h) 分類③負契約の非破壊 | 変更なし（`HomePage.test.tsx`） | SP5（regression 実行） | — |
| DSR-17 禁止（mount 一律 scroll 再導入禁止） | 全 Scope | 実装 diff review（route component への scroll 追加なし） | — |
| sidebar 跳びバグの解消（PR #28 L3 実観測） | Scope 1-3 全体 | — | AC-L3（Human Gate） |
| 01-decision-rules.md 「唯一」表現訂正 | Scope 5 | doc-consistency-check | — |

## Test Plan

Test Design Matrix: [test-matrices/2026-09-02-sidebar-scroll-restoration-exclusion.md](test-matrices/2026-09-02-sidebar-scroll-restoration-exclusion.md)

- targeted tests: 起動時 sweep（SP1）、`onBeforeLoad` prune の key 限定性（SP2）、`<main>` entry 保存の regression（SP3）、null-safety（SP4）
- negative tests: SP2 の他 key 非破壊（non-empty presence oracle）、SP4 の null cache 例外なし
- compatibility checks: 既存 `app-router.test.tsx`（T1/T4/T5/T9/T10/T11/T12/T13）/ `HomePage.test.tsx` / `page-scroll.test.ts` 無変更 green（SP5、AC1〜AC4）
- data safety checks: 業務データ非接触
- main wiring/integration checks: `createAppRouter` 内での sweep 配線（build green で担保）、`onBeforeLoad` 購読内の prune 配線（SP2）
- Human Gate に L3 を含めるため、Writer 完了条件に `cargo check --release` を含める（CI gate ではない — release build blind spot 対策）

## Boundary / Wire Contract

- producer: `@tanstack/router-core` の document capture listener（library 内部、app からは非操作）が `scrollRestorationCache.state` へ書き込む。
- consumer: `app-router.ts` の `pruneScrollRestorationEntries`（本 packet で新設）が同じ `scrollRestorationCache.state` を読み書きする。library の `onRendered` 復元ロジックも同じ state を読む（consumer 側は変更なし）。
- wire type: sessionStorage key `tsr-scroll-restoration-v1_3` の JSON 値（`Record<string, Record<string, {scrollX:number, scrollY:number}>>`）。
- internal type: `ScrollRestorationByKey`（router-core 型、非公開エクスポートだが `.state` の実行時形状として扱う）。
- precision/range: scroll 座標（number、NaN/Infinity は library 側で `Number.isFinite` ガード済み、既存契約で非接触）。
- round-trip path: `onBeforeLoad` snapshot → 本 packet の prune → `pagehide` persist（sessionStorage） → 次回 module import 時の読込み → 起動時 sweep（本 packet）。
- invalid input: cache 破損 / 欠落は library 内 safe parse（miss 扱い）。`pruneScrollRestorationEntries` は存在しない key / entry に対して no-op（例外を投げない）。
- compatibility: 導入前（sidebar entry が cache に残存する状態）に対し、導入後は sidebar entry が prune され続けるため、is-N/A の後方互換問題なし（cache は sessionStorage のみで永続化されアプリ再起動ごとに再構築されるため、旧汚染分も起動時 sweep で除染される）。

## Review Focus

- `scrollRestorationCache` の null-safety（sessionStorage 不可環境で `pruneScrollRestorationEntries` / `createAppRouter` が例外を投げないか — SP4 の弁別性）。
- SP4 の前提 assert（`scrollRestorationCache === null`）が実在し、silent pass になっていないか。
- key の等価性（`getAppScrollRestorationKey` が返す文字列と `event.fromLocation` から算出する prune 対象 key が一致しているか。href 完全一致でなく別粒度になっていないか）。
- subscriber 順序への依存（library の `onBeforeLoad`〈snapshot 書込み〉が app の `onBeforeLoad`〈prune〉より確実に先に実行される保証が、`router.subscribe` の Set 挿入順という既存の暗黙契約に依存している点 — 版数更新時に崩れ得るリスクとして DSR-17 (f) の再検証対象に含めるべきか）。
- oracle の独立性（Matrix の selector literal `'[data-scroll-restoration-id="main"]'` が `app-router.ts` の `MAIN_SCROLL_SELECTOR` 定数を import せず、独立転記になっているか）。
- 起動時 sweep と `onBeforeLoad` prune の役割分担が二重実装（同じ処理の重複）でなく補完関係（sweep = 起動時の一括除染、onBeforeLoad = 継続的な除染）として実装されているか。
- DSR-17 (j) 文面が起票時実測の事実と一致し、誇張・省略がないか。「唯一」表現の訂正が節内で自己矛盾なく反映されているか。

## Spec Contract

Contract ID: SPEC-DSR17-SIDEBAR-SCROLL-EXCLUSION-2026-09-02

- `scrollRestorationCache` から `[data-scroll-restoration-id="main"]` 以外の selector entry を router 生成直後（全 key）と各 `onBeforeLoad`（`fromLocation` key のみ）に削除する。
- `scrollRestorationCache` が `null` の環境では prune は no-op とし、例外を投げない。
- `<main>` の DSR-17 分類②④（href key 復元・主ナビ先頭表示・clamp 遅延再適用）は無変更で動作する。
- sidebar（chrome の主ナビゲーション領域）の scroll 位置は route 遷移で復元されない。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| DSR-17 (j) 起動時 sweep | Scope 1-2 | SP1 | null-safety・sweep 範囲 | Matrix |
| DSR-17 (j) onBeforeLoad prune | Scope 3 | SP2 | key 等価性・非破壊範囲 | Matrix |
| DSR-17 (j) main entry 保存 | Scope 1 | SP3 | allowlist 判定 | Matrix |
| DSR-17 (j) null-safety | Scope 1 | SP4 | 例外なし | Matrix |
| DSR-17 (a)〜(i) 既存契約非破壊 | 変更なし | SP5（regression） | 既存 test 無変更 green | PR body |
| DSR-17 (j) 追記・「唯一」訂正 | Scope 5 | AC8 | 追記の正確性 | doc check + diff |
| sidebar 跳びバグ解消 | Scope 1-3 | — | 実機視認 | L3 + PR body |

## Data Safety

- scroll 座標・href・sessionStorage key のみを扱う。業務データ・実店舗データに非接触。
- local-only: sessionStorage は WebView プロセス内のみ、DB 非接触。
- synthetic-only: test の seed データはすべて synthetic な href / selector 文字列。

## Implementation Results

- `pruneScrollRestorationEntries` を追加し、router 生成直後の全 key sweep と `onBeforeLoad` の遷移元 key prune を配線した。`<main>` entry は保持し、sidebar 等の non-main selector entry だけを削除する。
- SP1〜SP4 と既存 regression を実装・確認した。SP4 は sessionStorage access を事前に失敗させる precondition が成立し、実 `scrollRestorationCache === null` 経路を採用した（fallback 注入は不使用）。
- Mutation X1〜X5 は Coordinator 裁定により Writer 側未実施（安全審査により注入不可）。Coordinator + Final Reviewer の clean tree 独立再実測へ委ねる。
- DSR-17 Why / (c) の scroll container 表現を route content 限定へ訂正し、(j) の allowlist prune 契約と更新履歴を反映した。
- Draft PR: [#29](https://github.com/higekiri-w/inventory-system-desktop/pull/29)

## Review Response

2026-09-02 Final Review round 1（Sonnet subagent fresh context、Contract Audit を DSR-17 (j) / Scope 1〜5 / Matrix SP1〜SP5 から実施、router-core `router.js:118` と `scroll-restoration.js:104-108` の購読順を実読で再確認）= P1 0 / P2 0 / P3 2。文言表 presence oracle: 旧文言「が唯一の scroll container である」0 hit、新文言「route content の唯一の scroll container」3 hit（Why×2 + (c)）。`bindings.ts` / `Sidebar.tsx` / `scroll-area.tsx` / `package.json` は差分ゼロ。

- P3-1（記録のみ）: SP3 は main entry 保存だけを assert し、同 test 内で非 main entry の削除は assert しない。Matrix の SP3 Failure Mode は「main entry の誤削除」限定で、非 main 削除は SP1 / SP2 が担保するため Matrix 逸脱ではない。
- P3-2（記録のみ）: `cache` injectable 引数は SP4 fallback 用として条件付き pre-approved だったが、helper signature に恒常的に組み込まれている。Scope 1 は signature を規定しておらず、既定値が module singleton で呼出し側は既定値のみ使うため実害なし。

Coordinator mutation 独立再実測（隔離 worktree、clean tree = Reviewed Content HEAD、`app-router.test.tsx` + `app-router.null-cache.test.tsx` を mutant ごとに単独実行）: X1（allowlist 全 admit）→ SP1 / SP2 kill、X2（起動時 sweep 削除）→ SP1 kill、X3（`toLocation` 取り違え）→ SP2 kill、X4（allowlist 反転）→ SP3 + SP1 / SP2 + 既存 T5 / T10 / T11 / T12 / T13 kill、X5（`onBeforeLoad` prune 行削除）→ SP2 kill。**X1〜X5 全 kill、survivor なし**、Matrix 期待列と一致（X4 は超過 kill）。Writer 側の mutation 自己検証は安全審査で注入不可のため未実施（Implementation Results 記載）で、本独立再実測が唯一の kill 証跡。

- Findings Freeze: frozen at Final Review round 1（P1/P2 = 0）; post-freeze exceptions: none.
