> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: [02-component-catalog.md](../design-system/02-component-catalog.md)（パターン契約の正典）、[2026-06-12-design-system-pr-b.md](../archive/plans/2026-06-12-design-system-pr-b.md)（Plan Packet、D-B1〜D-B6 の判断根拠）

## 59. UI-patterns: 共通 UI パターン部品（src/components/patterns/）

### 本書のテンプレ判定（UI 層関数設計書の 2 段階）

[52-ui-shared-layout.md](52-ui-shared-layout.md) が確立した二段判定に従う:

- CMD 呼び出し: なし（全 component が props 駆動の pure presentational、useQuery / invoke 非内蔵）
- 入力バリデーション: なし（SearchBar は文字列をそのまま `onSearchChange` へ渡し、検証は呼び出し側の責務）
- 画面内部 state 駆動のフロー分岐: なし（SearchBar の draft / debounce は入力 UI の内部機構であり、業務データや CMD 呼び出しとは結合しない）

→ **業務ロジックなし（純構造）** と判定。主要 3 項目（コンポーネント構成 / React State / 備考）で記述する。

### 59.1 コンポーネント構成

| ファイル | 契約（props） | 採用画面 | catalog |
|---|---|---|---|
| `src/components/patterns/PageHeader.tsx` | `{title, subtitle?, actions?}` | 8 画面（ProductFormPage は通常 + edit error の 2 site） | ① |
| `src/components/patterns/SummaryCard.tsx` | `{title, isLoading, isError, onRetry, loadingSkeleton?, children}` | HomePage 3 カード（独立/共有 query × per-card retry） | ② |
| `src/components/patterns/FormSection.tsx` | `{title, description?, children}`（description 未指定時 `<p>` 非描画） | ProductForm 4 セクション | ④ |
| `src/components/patterns/EmptyState.tsx` | `{icon?, title, description?, action?}` | 件数は断定せず `rg -n --glob '!**/*.test.tsx' '<EmptyState' src` で実測する（round 1 P2-4、D-050 準拠）。filter-empty reset の対象/除外分類は [02-component-catalog.md](../design-system/02-component-catalog.md) ⑥ が正本 | ⑥ |
| `src/components/patterns/SearchBar.tsx` | `{value, onSearchChange, label?, id?, placeholder?, ariaLabel?, debounceMs?, showSubmitButton?, type?, wrapperClassName?, inputClassName?}` | 商品一覧・在庫照会・入出庫履歴（いずれも live 型、`debounceMs=200`）。commit 型は現在の採用箇所なし（機能残置、撤去は別判断、2026-08-03 owner L3 是正） | ⑨ |
| `src/components/patterns/DepartmentFilter.tsx` | `{options, selected, onChange, disabled?, allLabel?, idPrefix?}`（allLabel 既定「すべての部門」。幅は部品が `w-[11rem]` に固定し、幅の prop を持たない） | daily / products / price-revision（`PriceRevisionFilters`） / stock / stocktake | ⑨ |
| `src/components/patterns/PageShell.tsx` | `{className?, children}`（`cn(className, "space-y-6 p-6")`、base class 後勝ちで `space-y-6`/`p-6` を保証、`min-h-screen` は持たない） | `src/features/**/*Page.tsx` 全 29 file（`rg -l "<PageShell" src/features --glob '!*.test.tsx'`、Lane 2） | 04 原則 6 |
| `src/components/patterns/ListShell.tsx` | `{toolbar?, toolbarSecondary?, pagination?, topSummary?, stickyHeader?, identityColumns?, isLoading?, skeleton?, children}` | 商品一覧 pilot 1 画面（`rg -l "<ListShell" src/features --glob '!*.test.tsx'`、他 7 画面は Lane 3〜5、Lane 2） | ⑯ |
| `src/components/patterns/Pagination.tsx` | `Pagination`: `{page, perPage, totalCount, onPageChange}`（下部 pager）。`PaginationSummary`: `{page, perPage, totalCount}`（上部 text-only） | `Pagination` は旧 7 caller（IntegrityCheck / InventoryRecords / OperationLogs / PriceRevision / StockInquiry / StockMovements / Stocktake、`rg -l "<Pagination\b" src/features --glob '!*.test.tsx'`）+ 商品一覧は `ListShell` 経由（直接 JSX 不使用）。旧 `ProductPagination`（features/products 配下）から Lane 2 で移設 | ⑩ |

各 component の DOM 構造・トークン・Do/Don't の正典は [02-component-catalog.md](../design-system/02-component-catalog.md) の該当パターン。本書は props 契約と採用箇所の対応表を担い、二重記述しない。

### 59.2 React State

- **PageHeader / SummaryCard / FormSection / EmptyState / DepartmentFilter**: 内部 state なし（完全 props 駆動）
- **SearchBar**: `draft`（入力中文字列のローカル保持）+ debounce timer（live 型のみ）。`value` prop 変更で `draft` を同期。Enter keydown は両モードとも `event.isComposing` を最優先除外（IME 変換確定 Enter の誤発火防止）。commit 型は Enter / 検索ボタンで `onSearchChange(draft.trim())`、live 型は onChange を `debounceMs` で遅延 + Enter で即 flush（trim なし）

### 59.3 備考

- **SummaryCard の 3 パターン規約**: 回復導線の置き方（per-card retry / page-level Alert）は catalog ② の 3 パターン規約に従う。daily/monthly `SummaryCardsBar` はパターン 3 の canonical variant として patterns/ 統合の対象外（構造非互換、PR-B D-B1）
- **空状態の 2 系統**: 0 件成功 = `EmptyState`（pure テーブル内 or ページ分岐内）/ 取得失敗 = ページ側 Alert 差し替え。`EmptySearchPlaceholder` と shortcuts `emptyMessage` は semantic 相違（空結果でない）のため適用除外（catalog ⑥）
- **test の責務分離**: `src/components/patterns/*.test.tsx` は各 component 単体の DOM 規約を、feature 側の characterization test（B0 系）は「画面が component を正しく差し込む結線」を担う。両者は責務が異なるため重複ではなく、片方を理由なく削除しない
- **型の共有**: `DepartmentOption` は `patterns/DepartmentFilter.tsx` が唯一の定義を持つ（SPEC-UIBB-6、2026-08-03 batch B で統一済み）。feature 側ローカル定義（`useProductList.ts` / `daily-sales/types.ts` / `stock-inquiry/types.ts` の 3 箇所）は削除し、re-export へ置き換えた。方式はファイル別（round 1 P1-5 対応）: `useProductList.ts` は同一モジュール内で `DepartmentOption[]` を使用するため、`export type { X } from` 単文ではローカル binding が入らず成立しない。`import type { DepartmentOption } from "@/components/patterns/DepartmentFilter";` + `export type { DepartmentOption };` の 2 文とする。`daily-sales/types.ts` / `stock-inquiry/types.ts` はモジュール内使用がないため `export type { DepartmentOption } from "@/components/patterns/DepartmentFilter";` の直接 re-export 1 文とする。consumer の import path はいずれも不変
- **FilePicker の適用除外**（2026-08-03 batch B、round 2 P2-4 で責務帰属を精密化）: `src/components/FilePicker.tsx` は `@tauri-apps/plugin-dialog` / `@tauri-apps/plugin-fs` の副作用を持つため、本節 §59.4 の純表示部品規約の対象外とする。`patterns/` へは移動せず `components/` 直下に配置したまま、visual（DOM 構造・トークン・visual Do/Don't）は [02-component-catalog.md](../design-system/02-component-catalog.md) ⑭ が、behavior / API 契約（props 既定値・入口 2 経路・`onError` フォールバック・禁止規定）は [UI_TECH_STACK.md](../UI_TECH_STACK.md) §6.5.4 が正典

### 59.4 非目的

| やらないこと | 理由 | 責務を持つモジュール |
|------------|------|-----------------|
| CMD 呼び出し・データ取得 | patterns/ は純表示部品。query 管理は画面側 | 各 feature の hooks |
| 入力値の業務バリデーション | SearchBar は文字列を素通しする | 呼び出し側ページ / BIZ 層 |
| daily/monthly 集計バーの統合 | タイトル Skeleton / sub 行 / Tooltip の構造非互換、prop 肥大化回避 | `SummaryCardsBar`（各 feature 内） |

### 更新履歴

| 日付 | PR | 内容 |
|------|-----|------|
| 2026-06-13 | PR-B | 新設（B7）。6 component の契約と採用箇所、3 パターン規約・空状態 2 系統・test 責務分離を記録 |
| 2026-08-03 | ui-polish-batch-b（本 PR） | §59.1 DepartmentFilter 採用箇所を stocktake 追加の 4 画面へ、EmptyState 採用箇所を実測 19 画面/component へ sync。§59.3 `DepartmentOption` re-export 統一を実施済みの記述へ更新（feature 側ローカル定義 3 file 削除）。§59.3 に FilePicker（`components/FilePicker.tsx`）の §59.4 対象外注記を追加（02-component-catalog.md ⑭ が実装規約を管理） |
| 2026-08-03 | ui-polish-batch-b round 1 是正（本 PR） | §59.1 EmptyState / DepartmentFilter の採用箇所を件数断定（19 画面/component、4 画面）から検索式参照へ是正（round 1 P2-4、D-050 準拠）。§59.3 re-export 方式をファイル別（`useProductList.ts` = import type + export type の 2 文、他 2 file = 直接 re-export 1 文）へ明記（round 1 P1-5 対応） |
| 2026-08-03 | ui-polish-batch-b round 2 是正（本 PR） | round 2 P2-4 対応: §59.3 FilePicker 適用除外注記の帰属を「visual（DOM 構造・トークン・visual Do/Don't）は catalog ⑭、behavior / API 契約は UI_TECH_STACK §6.5.4」へ精密化（旧「方針・経緯の正典」という曖昧表現を是正） |
| 2026-08-03 | ui-polish-batch-b owner L3 是正（本 PR） | §59.1 SearchBar 採用箇所を「商品一覧・在庫照会ともに live 型（`debounceMs=200`）」へ統一。commit 型は現在の採用箇所なし（機能残置、撤去は別判断）と注記（owner L3 判断 2026-08-03、gated design amendment、50 UI-01a-D9 / 02 ⑨ と同時是正） |
| 2026-09-28 | small-fixes-batch（plan 側で先に訂正） | §59.1 DepartmentFilter の props から幅の prop を外し（幅は部品が `w-[11rem]` に固定、02 ⑨）、採用画面に price-revision（`PriceRevisionFilters`）を足して 5 site にする |
