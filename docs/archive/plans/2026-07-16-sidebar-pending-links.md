# サイドバー pending 2 項目の解消（UI-01b 商品登録 active化 + UI-06b 在庫少一覧 deep-link化）

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 330628d
- Amendments: none
- Coordinator: Fable
- Writer: Codex（発注 relay、owner がコピペ実行）
- Plan Reviewer: independent Sonnet review context
- Final Reviewer: independent Sonnet review context + Fable 裁定（P1/P2/P3）
- Reviewed Content HEAD: 4b489ab
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（L3 全 PASS・Ready 承認済み 2026-07-16、詳細証跡は PR body。残る owner 操作は Ready 化クリックと merge のみ）

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
operator workflow（サイドバー導線）の変更。`route`新設や CMD/BIZ/DB 変更はないが、`NavItem` 型への `search` field 追加と `SidebarLink` の active 判定ロジック変更は既存 20 nav 項目全体の表示契約に触れる横断変更であり、誤実装は「今どこにいるか分からない」という非 IT operator 向け致命的回帰になり得る。

## Goal

Goal Invariant:

### 最小完了条件

- operator がサイドバーから「商品登録」（`/products/new`）と「在庫少一覧」（`/stock` の在庫少フィルタ）の両方に到達できる。
- サイドバー上の pending（未実装・灰色 disabled）項目が 0 件になる。
- `/stock` に在庫少フィルタで在庫少一覧経由の場合は「在庫少一覧」のみ active、それ以外の状態で在庫照会に来た場合は「在庫照会」のみ active になり、両方同時点灯・両方非点灯にならない。

### 失敗定義

- 「商品登録」または「在庫少一覧」のいずれかにサイドバーから到達できない。
- 「在庫照会」「在庫少一覧」の active 表示が同時点灯または同時非点灯になり、operator の現在地表示が曖昧になる。
- 新規 `/stock/low` 画面を作ってしまう（D-047 の判断を上書きする）。

### 非目的

- `/stock/low` 独立画面の新設（D-047 で廃止済み判断）。
- 在庫照会画面（UI-06a）自体の機能変更（検索・フィルタ・詳細カードのロジックは無変更）。
- UI-06c（在庫変動履歴）の変更。
- 商品登録・在庫照会以外の nav 項目（既存 active 17 項目）の挙動変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `src/config/navigation.ts`:
  - `NavItem` 型に optional `search?: Record<string, string>` field と optional `activeMatch?: { searchKey: string; is?: string; isNot?: string }` field を追加（UI-12-D1）
  - `ui-01b-new`（商品登録）: `to: null` → `to: "/products/new"`、`status: "pending"` → `status: "active"`
  - `ui-06b`（在庫少一覧）: `to: null` → `to: "/stock"`、`status: "pending"` → `status: "active"`、`search: { status: "low_stock" }` + `activeMatch: { searchKey: "status", is: "low_stock" }` を追加
  - `ui-06a`（在庫照会）: `activeMatch: { searchKey: "status", isNot: "low_stock" }` を追加（排他判定のため。`to`/`status`/既存挙動は無変更）
  - L49 ヘッダコメント「4 エリア × 19 項目」→ 20 項目に是正（UI-13 追加時の更新漏れ。`52-ui-shared-layout.md` 側は本 PR で 20 項目へ是正済み、コード側コメントのみ未反映だった）
- `src/routes/stock/index.tsx`: 冒頭コメント「sibling route として UI-06b = /stock/low、UI-06c = /stock/$code/movements を予約。」を D-047 に合わせて是正する。UI-06b は独立画面ではなく本 route への deep-link のため「予約」ではない。UI-06c の予約記述は変更しない（例: 「sibling route として UI-06c = /stock/$code/movements を予約。UI-06b 在庫少一覧は独立画面を作らず本 route への `status=low_stock` deep-link とする（D-047）。」）
- `src/components/layout/SidebarLink.tsx`:
  - `search` field を持つ nav 項目は `<Link>` の `search` prop に渡して deep-link 遷移させる
  - `activeMatch` を持つ nav 項目のみ、`useRouterState` selector（§52.5 pathname 取得と同一パターン）で現在 pathname + search を読み、「`pathname === item.to` かつ predicate 成立」で active を明示判定する（UI-12-D1）。predicate は `is` 指定時は現在値との等値、`isNot` 指定時は非等値（現在値が `undefined`（= URL に key がない）の場合も `isNot` は成立扱い）。`activeMatch` を持たない項目は既存 `<Link activeOptions={{ exact: true, includeSearch: false }}>` 経路を無変更で維持する
  - **contract**: `SidebarLink` コンポーネント内に特定 route 文字列（`"/stock"` 等）をハードコードしない。比較対象の pathname は常に `item.to` から取得する（汎用性維持、将来同種の nav ペアが増えても component 変更不要）
- `src/config/navigation.test.ts`: REQ 番号入り到達テストを ui-11c/ui-13 パターン踏襲で追加（REQ-101: 商品登録、REQ-302: 在庫少一覧）+ 全 nav 項目 pending 0 件の回帰テスト
- `src/components/layout/SidebarLink.test.tsx`: 排他 active の正逆両方向テスト（`/stock?status=low_stock` で在庫少一覧のみ active / `/stock`（search なし）と `/stock?status=stockout` で在庫照会のみ active）+ 複合 search param テスト（`/stock?status=low_stock&q=毛糸` でも在庫少一覧のみ active、`searchKey` 以外の param が判定を壊さないこと）
- `src/components/layout/Sidebar.test.tsx`: 既存 snapshot・構造テストの追随（pending 表示 0 件化に伴う既存 assertion 更新のみ、新規業務ロジック追加なし）
- 生成物再生成: `src/lib/bindings.ts` / routes は無変更のため再生成不要。`cargo run --bin generate_traceability` は REQ-101/REQ-302 の test 追加に伴う coverage 数値変動を確認するため実装 PR で 1 度実行し、diff が REQ-101/REQ-302 の test 数変動のみであることを確認する（AUTO-GENERATED、手動編集禁止は継続）
- docs 側（本 Design Phase PR で先に完了）: `docs/function-design/52-ui-shared-layout.md`、`docs/function-design/58-ui-stock-inquiry.md`、`docs/SCREEN_DESIGN.md`、`docs/ARCHITECTURE.md`、`docs/decision-log.md`（D-047 追加）

## Non-scope

- `/stock/low` 独立画面・専用 route ファイル
- UI-06a（在庫照会）の検索・フィルタ・詳細カードのロジック変更
- UI-06c（在庫変動履歴）
- 新規 Tauri command、BIZ/DB 変更
- 既存 17 active nav 項目の挙動変更

## Acceptance Criteria

- `npm test` で `navigation.test.ts` が green。テスト名に `req101` を含む「商品登録が `/products/new` で active」テストと、`req302` を含む「在庫少一覧が `/stock` + `search.status=low_stock` で active」テストが存在する（evidence: テスト名 + `npm test` 結果）。
- `navigation.test.ts` に、全 nav 項目の `status` が `"pending"` を含まないことを assert する回帰テストが存在する（evidence: テスト名 + assertion）。
- `SidebarLink.test.tsx` に、`/stock?status=low_stock` では「在庫少一覧」のみ `aria-current`/active class 相当を持ち「在庫照会」は持たない、逆に `/stock`（search なし、または `status` が `low_stock` 以外）では「在庫照会」のみ active で「在庫少一覧」は持たないことを検証するテストが両方向存在する（evidence: テスト名 + assertion）。
- `rg "status: \"pending\"" src/config/navigation.ts` が 0 件（exit code 1）。
- hosted CI green + 三点 SHA 一致（live PR HEAD = local full evidence HEAD = hosted `headSha`、evidence: PR body）。
- Human Gate: Windows native L3 で「商品登録」「在庫少一覧」のサイドバークリック到達、および `/stock` を在庫照会・在庫少一覧の両経路から開いたときの active 表示の一意性を owner が目視確認する。

## Design Sources

List the source design docs this plan relies on. Plan Packets are not durable design source of truth.

- Requirements / spec: `docs/spec/requirements.md` REQ-101（商品を新規登録できること）、REQ-302（在庫切れ・在庫少の商品を一覧表示できること）
- Architecture: `docs/ARCHITECTURE.md`（UI-06b 行、第3段階/第10段階の依存関係記述）
- Function / command / DTO: `docs/function-design/52-ui-shared-layout.md` §52.3/§52.4/§52.6（navigation 定義・SidebarLink 仕様）、`docs/function-design/58-ui-stock-inquiry.md` §58.4/§58.10（既存 `status=low_stock` フィルタ contract、本 PR が deep-link 先として再利用する既存契約）
- DB: 該当なし（Rust/DB 変更なし）
- Screen / UI: `docs/SCREEN_DESIGN.md`（在庫照会画面節、§4/§5）
- Decision log / ADR: D-047（本 PR で新設、`/stock/low` 独立画面の廃止）、UI-12-D1（本 PR で新設、排他 active 判定契約）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 該当なし | existing sufficient（変更なし） |
| Command / DTO / generated binding / wire shape | 該当なし | existing sufficient（変更なし） |
| DB / transaction / audit / rollback / migration | 該当なし | existing sufficient（変更なし） |
| Screen / UI / route state / Japanese wording | `52-ui-shared-layout.md` / `58-ui-stock-inquiry.md` / `SCREEN_DESIGN.md` / `ARCHITECTURE.md` | updated in this PR（本 Design Phase） |
| CSV / TSV / report / import / export format | 該当なし | — |
| Durable decision / ADR | D-047 / UI-12-D1 | updated in this PR（本 Design Phase、`decision-log.md` に新設） |

## Registration / Generation Obligations

新規追加物に付随する登録・生成義務の checklist（UI-13 Amendment 1〜4 の failure class「plan 段階の列挙漏れ」対策）。該当する行の義務を Scope に明記し、R3/R4 では Contract Coverage Ledger にも契約行として反映してから Plan Gate に出す。該当なしなら `該当なし` と 1 行残す（節の削除はしない）。

| 新規追加物 | 登録・生成義務 |
|---|---|
| Tauri command（frontend から呼ぶ） | 該当なし（新規 command なし） |
| function-design doc 新設 | 該当なし（既存 52-ui-shared-layout.md / 58-ui-stock-inquiry.md の改訂のみ、新設 doc なし） |
| REQ coverage 追加（設計書・テスト追加） | 該当（REQ-101 / REQ-302 は既存 `required` coverage、新規テスト追加に伴い `cargo run --bin generate_traceability` で `90-traceability.md` 再生成し diff が該当 REQ の test 数変動のみであることを確認する） |
| route 新設 | 該当なし（`/products/new` / `/stock` とも既存 route、新規 route ファイルなし） |
| operator 画面新設 | 該当（新設ではなく既存 pending entry の active 化 2 件。義務内容は同一 — `src/config/navigation.ts` の entry 有効化（`to` + `status: "active"`）+ `navigation.test.ts` に REQ 番号入り到達テスト。「operator が画面に到達できる」到達導線契約を Contract Coverage Ledger の標準行として立てる） |

L1 full の生成系検査は bindings / frontend routes / traceability の 3 種。bindings / frontend routes は本変更で対象外（新規 command・route なし）。traceability 再生成のみ probe 対象。

## Design Intent Trace

Use spec/requirement IDs as the root. Use child decision IDs such as `UI-01a-D1`, `BIZ-08-D2`, or `SPEC-WF-...-D1` when a design choice needs rationale.

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-101 | `51-ui-product-form.md`（画面本体、変更なし）、`52-ui-shared-layout.md` §52.3/§52.4 | — | 既存実装済み route を nav 有効化するのみ。新規判断なし | `src/config/navigation.ts` | `navigation.test.ts` REQ-101 到達テスト |
| REQ-302 | `58-ui-stock-inquiry.md` §58.4/§58.10（既存 `status=low_stock` フィルタ contract）、`52-ui-shared-layout.md` §52.3/§52.4/§52.6 | D-047 | `/stock/low` 独立画面は REQ-302 に対し UI-00 サマリ + UI-06a フィルタチップで既 covered のため投資対効果なし。rejected: (a) `/stock/low` 新画面実装（重複画面）、(b) サイドバー項目削除（REQ-302 の常設到達導線消失） | `src/config/navigation.ts`（`search` field 追加） | `navigation.test.ts` REQ-302 到達テスト |
| — | `52-ui-shared-layout.md` §52.6（SidebarLink 仕様） | UI-12-D1 | `/stock` を指す nav 項目が 2 つになるため active 表示は排他にする。`NavItem.activeMatch?: { searchKey: string; is?: string; isNot?: string }` を新設し、ui-06b は `{ searchKey: "status", is: "low_stock" }`、ui-06a は `{ searchKey: "status", isNot: "low_stock" }` を宣言。`SidebarLink` は `activeMatch` を持つ項目のみ router の現在 pathname + search を明示比較（`includeSearch` 意味論には依存しない）、`activeMatch` を持たない項目は既存 `<Link activeOptions>` を無変更維持。component 内へのルート文字列ハードコード禁止（`item.to` 経由で比較）。rejected: 両方点灯許容（operator の現在地が曖昧）、`includeSearch` 依存（library の exact/fuzzy 意味論がバージョン依存で曖昧）、component 内 `"/stock"` 直書き分岐（汎用性がなく将来の同種ペア追加のたびに component 修正が要る） | `src/config/navigation.ts`（`activeMatch` 追加）、`src/components/layout/SidebarLink.tsx`（判定実装） | `SidebarLink.test.tsx` 排他 active 正逆両方向テスト + 複合 search param テスト |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: Yes — `decision-log.md` D-047 / UI-12-D1、`52-ui-shared-layout.md` §52.3/§52.4/§52.6、`58-ui-stock-inquiry.md` §58.1 に本 Design Phase PR で反映する
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-047（`/stock/low` 独立画面廃止）、UI-12-D1（排他 active 判定）を `decision-log.md` に新設
- Assumptions and constraints: `/stock` route の `validateSearch` zod schema（`status: z.enum(["all","stockout","low_stock"])`）は既存契約のまま変更しない前提。SidebarLink は RootLayout 配下で mount されるため route 固有の `Route.useSearch()` ではなく router 全体の generic search state を読む必要がある（§52.5 windowTitle 機構の `useRouterState({ select })` パターンを踏襲、Contract Probe 参照）
- Deferred design gaps, risk, and follow-up target: なし（本変更は既存 `/stock` フィルタ契約の再利用のみで、新規 BIZ/DB/CMD 判断を要しない）
- Test Design Matrix can cite design decision IDs or source doc sections: Yes — `docs/plans/test-matrices/2026-07-16-sidebar-pending-links.md` が D-047 / UI-12-D1 / REQ-101 / REQ-302 を Contracts Under Test に列挙する

## Impact Review Lenses

Fill this when the task starts from field investigation, real-device confirmation, external tool behavior, POS/register integration, CSV/TSV/report format changes, operator workflow discoveries, or a finding that may change source design assumptions. Otherwise write `not applicable` and why.

not applicable — 本変更は既存 route の nav 有効化 + 既存フィルタ契約への deep-link 追加であり、field investigation、実機確認、外部ツール、POS/レジ連携、CSV/TSV/report format、新規 operator workflow discovery のいずれにも該当しない。

## Design Readiness

State whether the design is ready for implementation.

- Existing design docs are sufficient because: `/products/new` route と `/stock` route の `status=low_stock` フィルタは既に実装済み・テスト済みの契約であり、本変更は nav 層（`navigation.ts` / `SidebarLink.tsx`）のみで完結する
- Source docs updated in this PR: `52-ui-shared-layout.md`、`58-ui-stock-inquiry.md`、`SCREEN_DESIGN.md`、`ARCHITECTURE.md`、`decision-log.md`（D-047 新設）
- Design gaps intentionally deferred: なし
- Durable decisions discovered in this plan and promoted to source docs: D-047、UI-12-D1

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 変更は UI 層のみ（`navigation.ts` / `SidebarLink.tsx`）、CMD/BIZ/IO/MNT 無変更
- Backend function design: 該当なし
- Command / DTO / data contract: 該当なし（既存 `/stock` route の `validateSearch` zod schema を無変更のまま再利用）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 既存ラベル「商品登録」「在庫少一覧」「在庫照会」を維持、新規文言なし
- Error, empty, retry, and recovery behavior: 既存 `/stock` / `/products/new` の契約をそのまま再利用、新規エラーパスなし
- Testability and traceability IDs: REQ-101 / REQ-302 は既存 `required` coverage、UI-12-D1 / D-047 を新設 decision ID として追跡

## Contract Probe

Required for R3/R4 plans that rely on an unverified external premise (external library behavior, OS/hardware behavior, etc.). Record the minimal experiment and its result as one line per premise. If not applicable, state N/A and the reason in one line instead of deleting the section.

登録漏れ是正を含む probe は、是正を仮適用した状態で end-to-end に実行する — 未登録状態のままの probe は、登録後に初めて顕在化する義務（specta 属性欠落等）を検出できない（UI-13 Amendment 1 の教訓）。

- TanStack 宣言的 `<Link search={...}>` によるナビゲーションが実運用で機能するか: `src/features/disposal/DisposalPage.tsx:622`（`<Link to="/inventory/records" search={{ recordType: "disposal_record" }}>`、一覧 route へのフィルタ付き deep-link という本変更と同型の既存実装、Windows native L3 済み）と `src/features/products/ProductListPage.tsx:76`（`<Link to="/products/new" search={{ returnTo }}>`）の 2 箇所で実証済み -> 既存実装パターンの直接転用であり追加 probe 不要（`src/routes/products/new.tsx` の `returnTo` は route 側で search を読む consumer 実装であり、`<Link search>` の producer 側実例ではないため引用を差し替え）
- RootLayout（route 非依存）配下で router の現在 search state を汎用的に読めるか: `§52.5` ウィンドウタイトル機構が既に `useRouterState({ select: (s) => s.location.pathname })` で pathname を汎用取得しているのと同一 selector API（`s.location.search`）を使うため、実装時にそのまま横展開すれば動作する -> 既存実装パターンの直接転用であり追加 probe 不要。`includeSearch` 意味論に依存しない設計判断（UI-12-D1）そのものが、この premise を機構レベルで回避する設計

## Contract Coverage Ledger

Required for R3/R4. Include every contract or design decision in the touched source-doc sections; a missing row is a Plan Gate blocker. Re-verify every row against real implementation at independent-review.

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| REQ-101 到達導線（商品登録） | `navigation.ts`（`to`/`status`） | `navigation.test.ts` REQ-101 到達テスト | L3（サイドバー→到達クリック確認、画面本体は既存 L3 済みにつき軽量） |
| REQ-302 到達導線（在庫少一覧 deep-link） | `navigation.ts`（`search` field）、`SidebarLink.tsx`（search 付与ナビゲーション） | `navigation.test.ts` REQ-302 到達テスト | L3（サイドバー→在庫少フィルタ反映確認） |
| UI-12-D1（排他 active 判定） | `navigation.ts`（`activeMatch` field: ui-06b `is: "low_stock"` / ui-06a `isNot: "low_stock"`）、`SidebarLink.tsx`（`activeMatch` 保有項目のみ router search state 明示比較、route 文字列ハードコード禁止） | `SidebarLink.test.tsx` 正逆両方向 + 複合 search param（`status=low_stock&q=毛糸` 等） | L3（`/stock` を両経路で開き active 表示の一意性を目視） |
| D-047（pending 0 化 / `/stock/low` 非新設） | `navigation.ts` 全 20 項目 `status` 監査 | `navigation.test.ts` pending 0 件回帰テスト | non-scope（自動テストのみで十分、新規画面がないため L3 不要） |

## Test Plan

For R3/R4, include or link a Test Design Matrix.

- targeted tests: `navigation.test.ts`（REQ-101/REQ-302 到達 + pending 0 件）、`SidebarLink.test.tsx`（排他 active 正逆）
- negative tests: `/stock`（search なし・または `status≠low_stock`）で「在庫少一覧」が active にならないことの確認、`/stock?status=low_stock` で「在庫照会」が active にならないことの確認
- compatibility checks: 既存 17 active nav 項目（`search` field を持たない）の active/pending 判定が本変更前後で不変であることを `Sidebar.test.tsx` で確認
- data safety checks: 該当なし（DB/永続化なし）
- main wiring/integration checks: `Sidebar.tsx` → `SidebarArea.tsx` → `SidebarLink.tsx` の既存配線を変更しないため新規 wiring チェックなし。`RootLayout.tsx` からの router search 取得のみ追加配線

Test Design Matrix: [docs/plans/test-matrices/2026-07-16-sidebar-pending-links.md](test-matrices/2026-07-16-sidebar-pending-links.md)

## Boundary / Wire Contract

Required when the change touches JSON API, browser state, CSV, config, manifest, cache schema, Tauri command DTOs, generated bindings, report output, or DB-backed compatibility.

- producer: `SidebarLink.tsx` の「在庫少一覧」nav 項目クリック（`<Link to="/stock" search={{ status: "low_stock" }}>`）
- consumer: `src/routes/stock/index.tsx` の既存 `validateSearch`（zod 4、`status: z.enum(["all","stockout","low_stock"]).optional().catch(undefined)`）
- wire type: URL query string `?status=low_stock`
- internal type: 既存 `ListChipFilter = "all" | "stockout" | "low_stock"`（`58-ui-stock-inquiry.md` §58.2 types.ts、無変更）
- precision/range: 列挙値のみ、数値精度は関与しない
- round-trip path: サイドバークリック → `navigate({ to: "/stock", search: { status: "low_stock" } })` → URL → 既存 `validateSearch` → `StatusChips` が `low_stock` 選択状態を表示
- invalid input: 既存 zod `.catch(undefined)` で不正値は `"all"` へ fallback（本変更で新規に導入する分岐ではない、既存契約の再利用）
- compatibility: `/stock` の `validateSearch` schema 自体は無変更。既存 `low_stock` enum member を deep-link の to 先として再利用するのみ

## Review Focus

- `SidebarLink.tsx` の排他 active 判定が `includeSearch` 意味論に依存せず実装されているか（UI-12-D1 の設計意図どおりか）
- `navigation.ts` の pending 項目が本当に 0 件になっているか（既存 17 active 項目に副作用がないか）
- `search` field を持つ nav 項目のクリックが実際に `/stock` の既存 `status=low_stock` フィルタと接続するか（`58-ui-stock-inquiry.md` 契約からの drift がないか）

## Spec Contract

Required for R3/R4.
Use at least one data row. Put concrete test names in the Test column when a regression test exists; use review/evidence labels only for plan-only checks.

Contract ID: D-047 / UI-12-D1

- サイドバー「商品登録」「在庫少一覧」が pending から active になり、operator が両方に到達できる
- `/stock` を指す 2 nav 項目（在庫照会・在庫少一覧）は search state に応じて排他的に active 表示される
- `/stock/low` 独立画面は作らない（D-047）

## Trace Matrix

Required for R3/R4.

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| REQ-101 | `navigation.ts` ui-01b-new active化 | `navigation.test.ts` REQ-101 到達テスト | 到達導線が実 route と一致するか | テスト名 + `npm test` 結果 |
| REQ-302 | `navigation.ts` ui-06b search 追加 | `navigation.test.ts` REQ-302 到達テスト | `58-ui-stock-inquiry.md` 既存 `status=low_stock` 契約と整合するか | テスト名 + 58-ui doc 対照 |
| UI-12-D1 | `SidebarLink.tsx` 排他 active 実装 | `SidebarLink.test.tsx` 正逆両方向 | `includeSearch` 依存を排除できているか | テスト名 + コード diff |
| D-047 | pending 0 化 | `navigation.test.ts` pending 0 件回帰テスト | 既存 active 項目に副作用がないか | テスト名 + assertion |

## Data Safety

Required for R3/R4.

- 実データ・PII・機微情報を含まない（UI nav 定義のみ、DB/ファイル I/O なし）
- local-only paths: 該当なし
- synthetic-only paths: 該当なし（テストは navigation 定数と route mock のみで完結）

## Implementation Results

Fill after implementation.

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

実装は Codex 発注 relay 1 往復で完了。Scope どおり `navigation.ts`（2 項目 active 化 + `search`/`activeMatch` field）、`SidebarLink.tsx`（`ActiveMatchSidebarLink` 分岐、`useLinkProps` で SPA 遷移維持、route 文字列ハードコードなし）、テスト 3 file（REQ-101/REQ-302 到達・排他 active 正逆・stockout/複合 param・pending 0 回帰、実 navigation 配線経由）、`stock/index.tsx` コメント是正、traceability 再生成。レビュー起因の実装手戻りなし。L1/L3/hosted CI 証跡は PR body 参照: https://github.com/higekiri-w/inventory-system-public/pull/9

## Review Response

Fill after review.
If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.

independent Final Review（Sonnet 独立 context、Writer / Plan Reviewer と別）+ Contract Audit 実施済み。Contract Coverage Ledger 全 4 行 verified（実装・テスト・source docs 突合、AC 機械検証 pass）。findings は P1 = 0 / P2 = 0 / P3 = 2 で、Coordinator 裁定は両方 accept・non-blocker:

- P3-1: `navigation.ts` の `"low_stock"` が `search.status` と `activeMatch.is` に二重記述（SSOT 定数なし）。`navigation.test.ts` の `toMatchObject` が両 field を固定しており回帰は CI 検出可能。恒久対処（`LOW_STOCK_FILTER_VALUE` 定数化）は follow-up として Plans.md backlog へ。
- P3-2: sidebar リンクの focus-visible styling 不足は本 PR 以前からの既存 gap（回帰ではない）。a11y polish として Plans.md backlog へ。

- Findings Freeze: frozen after Final Review; post-freeze exceptions: none.

## Workflow Narrative（append-only）

- 2026-07-16 state-only遷移 plan-draft->plan-gate->plan-approved->implementing（compression 規則、STATECAP forward 1/3）。各遷移の既存証跡: plan-draft->plan-gate = packet + Test Design Matrix commit 済み（plan-first `330628d` + Plan Gate 是正 `08fb05b`、`doc-consistency-check.sh --target plan` green）/ plan-gate->plan-approved = 独立 Plan Reviewer（Sonnet 独立 context、Writer と別）が一巡目 P1: 0 / P2: 5 / P3: 3 を報告、Coordinator 全件 accept・是正適用（`08fb05b`）後の targeted 再レビューで 8/8 resolved・新規 P1/P2 なし・P1/P2 残 = 0、`Plan Commit` = `330628d`（実装 commit は本遷移時点で 0 件、先行性成立）/ plan-approved->implementing = 実装（Codex 発注 relay）開始のための entry。Plan Reviewer の P3 未満メモ（`search` と `activeMatch` の `"low_stock"` 値重複が将来ズレるリスク）は Review Focus の既存観点で実装レビュー時に確認する。
- 2026-07-16 implementing->local-verified（本 content commit に riding）。実装は Codex 発注 relay（content candidate `13aa31f`、変更 file は packet Scope と一致、packet / Workflow State 不可侵遵守）。L1 `local-ci.sh full` CLEAN evidence は candidate SHA とともに PR body に記録（Evidence Ownership、tracked doc には転記しない）。本 commit は受け入れ検分で発見した残存 drift 2 箇所（52 §52.3 冒頭の「19 ナビ表示 = 21 route」表記 / SCREEN_DESIGN 2026-05-08 注記の項目数転記）の是正を含む。
- 2026-07-16 state-only遷移 local-verified->independent-review->human-confirm（compression 規則、post-implementation state-only 1/2）。各遷移の既存証跡: local-verified->independent-review = independent Final Reviewer（Sonnet 独立 context）engage 済み / independent-review->human-confirm = Final Review + R3 Contract Audit 完了（Ledger 4/4 verified、P1/P2 = 0、P3 = 2 は Review Response の裁定どおり non-blocker・backlog 化）。`Reviewed Content HEAD` = `4b489ab`（実装 `13aa31f` + 受け入れ検分 docs 是正を含む audited content commit）。残る Human Gate = owner L3（サイドバー到達 2 件 + active 表示一意性の目視）。
- 2026-07-16 state-only遷移 human-confirm->ready-hosted-final（post-implementation state-only 2/2、forward STATECAP 3/3 到達 = 以後の遷移は content commit riding のみ）。証跡: owner L3 全 PASS（商品登録 到達・active / 在庫照会 単独 active / 在庫少一覧 low_stock 反映・単独 active / 往復切替、Windows native、介入 2 回目/予算 3 回の束ね承認で Ready 承認込み）。本遷移後、この exact HEAD で L1 full を実行し PR body を全面更新してから owner が Ready をトリガーする。
- 2026-07-16 ready-hosted-final->merge->archive（Post-Merge Closeout）: Public PR #9 squash merge（`70dc4d1`）。hosted CI green + 三点 SHA 一致（live PR HEAD = local full evidence HEAD = hosted `headSha`）は PR body に証跡。本 commit で packet / matrix を `docs/archive/plans/` へ移動、WER は同ディレクトリの `2026-07-16-sidebar-pending-links-workflow-effectiveness-review.md`。
