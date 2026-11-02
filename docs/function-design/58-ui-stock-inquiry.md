> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（UI-06a タスク仕様）、SCREEN_DESIGN.md（商品と在庫画面）、screen_mockups.html（モックアップ、historical reference）、20-io-product-repo.md（IO-01 get_stock_detail / list_low_stock / search_products）、44-cmd-inventory.md（CMD-06 get_stock_detail / list_low_stock）、40-cmd-product.md（CMD-01 search_products）、57-ui-monthly-sales.md（UI-09b 業務ロジックあり版テンプレ）、56-ui-daily-sales.md（UI-09a 2 useQuery 部分障害許容テンプレ）

## 58. UI-06a: 在庫照会画面

> **計数の置き場と在庫の求め直し（D-118、b2 で実装済み）**: 一覧と詳細の在庫は写し（`stock_quantity`）のまま。写しが要再確認（`stock_recheck`）の商品は、在庫の欄に範囲（`formatStockOrRange` の形「{low}〜{high} {単位}」。例「8〜9 個」）と「要再確認」の①状態の badge（文字と icon）を出す（[40](40-cmd-product.md) の商品の結果の列。部品は [66](66-ui-stock-movements.md) と同じ `StockWithRecheck`）。状態の badge（在庫あり・在庫少・在庫切れ）は写しで決まり、範囲の端では決めない。在庫少の判定と並べ替えは写しの値で行う（今のまま）。詳細（UI-06b）は、写しが要再確認なら在庫数の下に要再確認の案内を出す（b2 の Plan Review round 1 の是正、2026-10-23。出所は AI の review で、owner の確認の前の既定）: warning の Alert（icon と文字）に、題「数え直しが必要です」、本文に理由（[35](35-biz-stocktake-service.md) の `list_recheck_products` の当該の商品の `recount_reasons` を、[73](73-ui-stocktake.md) の要再確認の一覧と同じ文で）と「棚卸しの画面で、棚の数を数えて入れ直してください。」、link「棚卸しの画面で入れ直す」（73 の「今の数を入れる」の入口。search の `product` に商品コード、`returnTo` は付けない）。案内を出すかは写しの `stock_recheck` で決め、当該の商品が一覧に無い・理由が空・一覧の読取りが読込み中か失敗なら、理由を出さずに題・案内・link を出す（一覧の読取りで詳細を止めない）。部品は `StockRecheckNotice`（要再確認の一覧の query は 73 と同じ `useRecheckProducts`）で、[66](66-ui-stock-movements.md) の見出しも同じ部品。

> **商品と在庫の画面への統合（D-138。proposed・未実装）**: 本画面は商品検索・一覧と 1 つの画面「商品と在庫」（route `/products`）になる（[50](50-ui-product-list.md) UI-01a-D14、owner の決定 TD-424・TD-425）。画面・URL・道具・一覧の列・0 件の契約は 50 が持つ。本書は統合の画面で、在庫の状態（§58.10 の Q-1・契約 H）、在庫切れ・在庫少の一覧（§58.6 `filterAndSortLowStockList`、UI-06a-D4・D6・D8）、行の詳細（§58.7 `StockDetailContent`、UI-06a-D5・D7）、部門候補（UI-06a-D2）、範囲外 page（UI-06a-D3）、pagination（UI-06a-D1）の契約を持ち続ける。旧契約 I（「すべて」の検索駆動表示）は 50 UI-01a-D1 で置き換えた。route `/stock` は `/products` へ寄せる（UI-01a-D19）。今の build（統合の前、`/stock` の在庫照会）の記述として残すのは次の項目で、後続の runtime の lane（[backlog](../backlog.md) の「商品と在庫の画面の統合の runtime」）が統合の形に書き換える: 下のテンプレ判定の表、§58.2 の file の表、§58.3 のデータフロー、§58.4 の schema の例と fallback の bullet、§58.5 の hook の疑似コード、§58.7 の `StockInquiryPage` の項（題・副題と疑似コード）と `SearchBar` の項、§58.9 の test の表。それ以外（§58.4 の `selected` ライフサイクルを含む）は統合の後の規則で、今の build と違う所は「今の build は…」と書いた。

### 本書のテンプレ判定（業務ロジックあり版・簡潔版、共通 6 項目。今の build）

今の build（`/stock`）の判定。統合の後の query は一覧（`search_products`）・在庫少の集合（`list_low_stock(true)`、契約 H）・詳細・部門候補の 4 つで、URL state は [50](50-ui-product-list.md) §50.4 の key（D-138）。

| 観点 | 本画面の選択 | 根拠 |
|---|---|---|
| useQuery 数 | **3 useQuery 部分障害許容**（list query = `search_products` または `list_low_stock`、detail query = `get_stock_detail`、departmentOptions query = `listDepartments`） | `get_stock_detail` と `list_low_stock` / `search_products` は独立 CMD で BIZ 内一体化されていない（UI-09a 横展開、機械的横展開でなく BIZ DTO 設計に従う）。`listDepartments` は page/q/dept/status 非依存の独立 query（UI-06a-D2、round 1 P1-3） |
| URL state | TanStack Router `validateSearch`（zod 4 直接渡し、`.optional().catch(undefined)` で不正値吸収）、`q` / `dept` / `status` / `page` / `selected` の 5 key | desktop-app-ui-constraints.md「状態の URL 化」、`selected` URL 化で詳細カード展開状態の F5 耐性、`page` は UI-06a-D1 pagination 導入で追加 |
| 派生純関数 | 4 個（derive-stock-state / format-stock-display / format-last-date / filter-low-stock-list） | UI-09a/b 同型、業務ルールを純関数に閉じる。閾値判定は持たない（BIZ 責務） |
| factory | 1 種類（`makeMockProductWithRelations` + `makeMockStockDetail`、test-fixtures.ts に集約） | テスト DRY、DTO + UI 派生型を 1 file に集約 |
| ファイル分離 | `src/features/stock-inquiry/` 内閉鎖（types / lib / hooks / components）+ shadcn primitives は `src/components/ui/` | UI-09a/b 同型。`toggle` / `toggle-group` は新規 add（`collapsible` も add したが F1 行インライン展開化で現在未使用、primitive 残置） |
| Sonner dedup | id-based（`stock-inquiry-list-error` / `stock-inquiry-detail-error`、リトライ成功で `toast.dismiss`） | UI-00 PR #56 Round 2 P2-B 確立、連続失敗時の toast 山積み回避 |

---

### 58.1 概要

- **対応 REQ**: REQ-301（SP-301-01〜03、商品別在庫照会）+ REQ-302（SP-302-01〜04、在庫切れ/少一覧）。REQ-303（在庫変動履歴）は UI-06c `/stock/$code/movements` への active link で接続する
- **サイドバー「在庫切れ・在庫少」との関係（D-047 の先を D-138 で置換え）**: UI-06b は独立画面を作らず、サイドバーの「在庫切れ・在庫少」から統合の画面 `/products` への `search: { status: "low_stock" }` deep-link として提供する（今の build は項目名「在庫少一覧」で `/stock` への deep-link）。`status` の状態のチップ（§58.10 UI-06a-D8）をそのまま使い、REQ-302 の常設到達導線を担う。同じ route を指すサイドバー 2 項目（商品と在庫・在庫切れ・在庫少）の排他 active 判定は `docs/function-design/52-ui-shared-layout.md` §52.6 UI-12-D1 を参照
- **対応 task**: UI-06a（[ARCHITECTURE.md §UI-06a](../architecture/ui-task-specs.md)）
- **呼び出す CMD**:
  - 一覧: `search_products(query: ProductSearchQuery)`（status === "all"）。在庫少の集合: `list_low_stock(true)`（status === "all" の状態の欄、§58.10 契約 H）。「在庫切れ・在庫少」の一覧: `list_low_stock` の結果から廃番を除く（status === "low_stock"、§58.6 `filterAndSortLowStockList`）。今の build は status === "stockout" | "low_stock" で `list_low_stock(false)` を読み、「すべて」では読まない
  - detail query: `get_stock_detail(productCode: string)`（commit 1 で specta 化）
- **route**: `/products`（[50](50-ui-product-list.md) UI-01a-D14。`/stock` は寄せる、UI-01a-D19）。今の build は `/stock`（`src/routes/stock/index.tsx`、[52-ui-shared-layout.md](52-ui-shared-layout.md) UI-12 合意書）
- **主動線**: ホームの「商品と在庫」・「在庫切れ・在庫少」の card / サイドバーの同じ 2 項目から開き、一覧 → 行の直下の詳細 → 発注判断・入庫記録・商品修正・在庫変動履歴（D-138）。今の build はホーム・サイドバーの「在庫照会」から開き、検索欄 → 結果一覧 → 詳細の展開
- **失敗 4 状態**:
  1. list query API fail → ProductListTable 領域 Alert + toast（`stock-inquiry-list-error`）
  2. list query items 空配列 → 0 件の空状態。見出し・説明・action は [50](50-ui-product-list.md) §50.7 の Empty の順（UI-01a-D18。範囲外 page は UI-06a-D3 が先）。今の build は一律に「該当する商品がありません」
  3. detail query fail → 詳細カード内 inline エラー（部分障害許容、一覧表示は維持） + toast（`stock-inquiry-detail-error`）
  4. （旧契約 I の検索前の案内は D-138 で撤去した。初めて開いたら一覧を出す〈50 UI-01a-D1〉。今の build は status === "all" かつ `q` 空文字で search_products を呼ばず EmptySearchPlaceholder を出す）

### 58.2 ファイル構成（命名規約 + 型 export）

#### bindings 由来の型（specta 出力、commit 1 で生成、`src/lib/bindings.ts` 実出力で検証済）

```ts
// src/lib/bindings.ts
export type Product = {
  product_code: string;
  jan_code: string | null;
  name: string;
  department_id: number;
  supplier_id: number | null;
  selling_price: number;
  cost_price_centi: number;
  tax_rate: ProductTaxRate;
  maker_code: string | null;
  stock_quantity: number;
  stock_unit: ProductStockUnit; // 10 個の code（SPEC-UNIT-D1）
  is_discontinued: boolean;
  plu_dirty: boolean;
  plu_exported_at: string | null;
  pos_stock_sync: boolean;
  created_at: string;
  updated_at: string;
};

// Product フィールドは serde flatten でトップレベルに展開される（intersection 型）
export type ProductWithRelations = {
  department_name: string;
  supplier_name: string | null;
} & Product;

export type StockDetail = {
  product: ProductWithRelations;       // named field（flatten ではない）
  last_receiving_date: string | null;
  last_sale_date: string | null;
};

export type ProductSearchQuery = {
  keyword: string | null;
  department_id: number | null;
  is_discontinued: boolean | null;
  sort_key: SortKey;                   // "Name" | "ProductCode" | "StockQuantity" | "SellingPrice"（PascalCase）
  sort_order: SortOrder;               // "Asc" | "Desc"（PascalCase）
  page: number;                        // 1 始まり
  per_page: number;                    // 1 以上、デフォルト 50、上限 200。200 超は IO 層でクランプ
};

export type PaginatedResult<T> = {
  items: T[];
  total_count: number;
  page: number;
  per_page: number;
};
```

**型アクセスの重要差異**（bindings 実出力で確認、`feedback-design-doc-tech-premise-verify-from-output.md`）:
- `list_low_stock` 戻り値 = `ProductWithRelations[]`（配列、`PaginatedResult` ではない）。flatten で `item.stock_quantity` / `item.product_code` を直接参照
- `search_products` 戻り値 = `PaginatedResult<ProductWithRelations>`（`data.items` / `data.total_count`）
- `StockDetail.product` は named field のため `detail.product.stock_quantity` / `detail.product.department_name`（flatten される Product フィールドは `detail.product.*` 下に展開）
- `commands.searchProducts(query)` の callsite は `ProductSearchQuery` を**直接渡す**（`{ query: {...} }` で wrap しない。Tauri invoke 引数 `{ query }` は binding 内部で wrap 済）

#### features/stock-inquiry/

| 配置 | ファイル | 責務 | 規模 |
|---|---|---|---|
| 新規 | `src/routes/stock/index.tsx` | route + `validateSearch`（zod 4）+ navigate ラップ | 40-60 |
| 新規 | `src/features/stock-inquiry/StockInquiryPage.tsx` | 最上位 page、失敗 4 状態出し分け + 主動線 + sub 動線 | 100-140 |
| 新規 | `src/features/stock-inquiry/types.ts` | `StockStatus = "ok" \| "low" \| "stockout"` / `ListChipFilter = "all" \| "stockout" \| "low_stock"` / `StockInquiryListResult` 正規化型 | 30-45 |
| 新規 | `src/features/stock-inquiry/lib/derive-stock-state.ts` | 在庫状態派生（source 引数で色分け契約 H 準拠、閾値持たない） | 20-30 |
| 新規 | `src/features/stock-inquiry/lib/format-stock-display.ts` | 在庫数 + 単位表示（10 個の code を型で網羅） | 20-30 |
| 新規 | `src/features/stock-inquiry/lib/format-last-date.ts` | `string \| null → string`（None → 「—」、Q-2） | 15-25 |
| 新規 | `src/features/stock-inquiry/lib/filter-low-stock-list.ts` | `list_low_stock` 結果の q / dept sub-filter + stockout/low 分岐 | 30-45 |
| 新規 | `src/features/stock-inquiry/lib/test-fixtures.ts` | `makeMockProductWithRelations` + `makeMockStockDetail` factory | 30-45 |
| 新規 | `src/features/stock-inquiry/hooks/useStockInquiry.ts` | 3 useQuery 部分障害許容 + `StockInquiryListResult` 正規化 + 1 件自動展開 + selected 不在 clear useEffect | 100-140 |
| 新規 | `src/components/patterns/SearchBar.tsx`（実装当時は `stock-inquiry/components/SearchBar.tsx`、PR-B で統合） | 検索 input（placeholder + `autoFocus` + Enter 検索 + debounce 200ms = live 型、[59-ui-shared-patterns.md](59-ui-shared-patterns.md)） | 50-70 |
| 新規 | `src/features/stock-inquiry/components/StatusChips.tsx` | 3 チップ（shadcn `ToggleGroup`、件数バッジなし、Q-5） | 40-55 |
| 新規 | `src/components/patterns/DepartmentFilter.tsx`（実装当時は UI-06a 用ローカル実装、PR-B で 3 feature を統合） | shadcn Select 単一選択（`DepartmentOption` 型は patterns/ が定義、[59-ui-shared-patterns.md](59-ui-shared-patterns.md)） | 40-60 |
| 新規 | `src/features/stock-inquiry/components/ProductListTable.tsx` | shadcn Table（`source` prop 受取 → derive-stock-state 引き渡し、状態列で Badge + icon + 日本語ラベル表示。stockout red / low yellow / ok default は在庫数セルの補助シグナル）+ 行クリックで選択 + 選択行直下に colSpan インライン展開（`detailQuery` props → StockDetailContent 共用） | 90-150 |
| 新規 | `src/features/stock-inquiry/components/StockStatusBadge.tsx` | `StockStatus` を `Badge + lucide icon + 日本語ラベル` に変換（在庫切れ / 在庫少 / 在庫あり）。閾値判定は持たない | 25-40 |
| 新規 | `src/features/stock-inquiry/components/EmptySearchPlaceholder.tsx` | status=all + q 空文字時の centered muted text（契約 I） | 20-30 |
| ~~新規~~ 撤去 | ~~`src/features/stock-inquiry/components/TruncatedResultsAlert.tsx`~~ | `truncated` 時に shadcn Alert で絞り込み案内（契約 I）。**2026-08-03 batch B（UI-06a-D1）で撤去**: pagination 導入により全件へ到達可能になったため、打ち切り告知は不要（§58.10 契約 I 参照） | 25-35 |
| 新規 | `src/features/stock-inquiry/components/StockDetailContent.tsx` | 詳細の内側描画（在庫数/売価/原価/最終入庫日/最終販売日 + 商品修正/入庫記録/在庫変動履歴 active link、isLoading/isError/data 全状態内包）。行インライン展開とフォールバックカードで共用。2026-08-26 に商品修正/入庫記録を active link 化 | 90-130 |
| 新規 | `src/features/stock-inquiry/components/StockDetailCard.tsx` | list 失敗時フォールバックカード（`StockDetailContent` を Card で包み独立描画 = 部分障害許容）、collapsible 不使用（条件描画） | 20-30 |

**2026-08-03 batch B 追加（UI-06a-D1）**: pagination は新規 file を作らず、既存 canonical `src/components/patterns/Pagination.tsx`（旧 `src/features/products/components/ProductPagination.tsx`、UI 一覧の背骨 D Lane 2 で移設）（[02-component-catalog.md](../design-system/02-component-catalog.md) ⑩）を結線する。`StockInquiryPage` が `total_count` から最終ページを計算して渡す。

#### components/ui（shadcn primitives 新規 add、shadcn CLI 不使用 = npm install 凍結遵守）

| 配置 | ファイル | 責務 | 規模 |
|---|---|---|---|
| 新規 | `src/components/ui/collapsible.tsx` | Radix Collapsible wrapper（`accordion.tsx` パターン踏襲、`@radix-ui/react-collapsible` は `radix-ui: ^1.4.3` umbrella transitive 既存）。※ F1 で StockDetailCard がインライン展開化したため現在未使用（primitive 残置、UI-06c 等で再利用余地。UI-06b は D-047 により独立画面化しないため対象外） | 20-30 |
| 新規 | `src/components/ui/toggle.tsx` | Radix Toggle wrapper（StatusChips の ToggleGroup item 基盤、`@radix-ui/react-toggle` 既存） | 30-45 |
| 新規 | `src/components/ui/toggle-group.tsx` | Radix ToggleGroup wrapper（StatusChips の 3 チップ用、`@radix-ui/react-toggle-group` 既存） | 40-60 |

#### 更新枠（既存 file）

| 配置 | ファイル | 責務 |
|---|---|---|
| 更新 | `src/lib/query-keys.ts` | `stockInquiry.list(status, q, dept)` / `stockInquiry.detail(productCode)` helper + `stockInquiryRoot()` prefix helper 追加 |
| 更新 | `src/config/navigation.ts` | UI-06a entry の `to: null → "/stock"` / `status: "pending" → "active"` |
| 更新 | `src/features/csv-import/hooks/useCsvImportFlow.ts` | commit / rollback success は D-052-C8/C9 の SSOT helper を適用（在庫照会 consumer を含む） |

合計: 新規 18 + 更新 3 = **21 file** + Vitest 8 = **29 file 実体**。frontend code 1100-1500 行 + 関数設計 450-550 行 + test 350-450 行。

### 58.3 データフロー

```
URL search params (q, dept, status, page, selected)
  ↓ validateSearch + zod 4 fallback
useStockInquiry({ status, q, dept, page, perPage, selected, navigate })
  ↓ list useQuery（status により分岐）
  │   status === "all" + q あり → commands.searchProducts(ProductSearchQuery { page, per_page: perPage, ... }) → PaginatedResult<ProductWithRelations>
  │   status === "stockout" | "low_stock" → commands.listLowStock(false) → ProductWithRelations[]（page 非対象、既存挙動）
  │   ↓ StockInquiryListResult { items, totalCount, source } に正規化（UI-06a-D1: `truncated` は撤去済み）
  ├ detail useQuery（selected != null）
  │   commands.getStockDetail(selected) → StockDetail
  ├ departmentOptions useQuery（page / q / dept / status 非依存、独立、UI-06a-D2）
  │   commands.listDepartments() → Department[] → DepartmentOption[]（hook 内変換、round 2 P1-1）
  ↓ 派生純関数（derive-stock-state / filter-low-stock-list / format-*）
{ listQuery, detailQuery, departmentOptionsQuery, departmentOptions, isAllEmpty }
  ↓
StockInquiryPage（EmptySearchPlaceholder / Skeleton / Alert / EmptyState 出し分け、EmptyState は絞り込み非既定時に「絞り込みを解除」action、catalog ⑥）
  ↓ render
SearchBar + StatusChips + DepartmentFilter（options={departmentOptions} / selected / onChange / disabled={departmentOptionsQuery.isLoading}）+ ProductListTable（状態列 + 選択行直下に StockDetailContent インライン展開）+ StockDetailCard（list 失敗時フォールバック）+ Pagination（status === "all" のみ、UI-06a-D1）
```

**派生 4 純関数の責務分担**:
1. `derive-stock-state` → `(item, source) → StockStatus`（色分け契約 H、`source === "search" && stock_quantity > 0` のみ `"ok"`）
2. `format-stock-display` → `(quantity, unit) → string`（`"10 個"` / `"3 m"`）
3. `format-last-date` → `(value: string | null) → string`（None → 「—」、Q-2）
4. `filter-low-stock-list` → `(items, q, dept, status) → ProductWithRelations[]`（`list_low_stock` 結果の sub-filter + stockout/low 分岐）

### 58.4 URL state 設計（zod 4 直接渡し）

```ts
// src/routes/stock/index.tsx
import { z } from "zod";

const searchSchema = z.object({
  q: z.string().min(1).max(100).optional().catch(undefined),
  dept: z.coerce.number().int().positive().optional().catch(undefined),
  status: z.enum(["all", "stockout", "low_stock"]).optional().catch(undefined),
  page: z.coerce.number().int().positive().optional().catch(undefined),
  selected: z.string().min(1).max(100).optional().catch(undefined), // 上限は q と同じ 100（66 UI-06c-D9）
});
```

- `status` の有限集合と `LOW_STOCK_FILTER = "low_stock"` は `src/features/stock-inquiry/types.ts` を単一所有者とする。route schema、`ListChipFilter`、サイドバー deep-link の `search.status`、`activeMatch.is/isNot` は同じ tuple / 定数を参照し、`"low_stock"` を別々に複製しない（UI-STATE-D2）。
- `q` undefined → `""` fallback（page 内）
- `dept` undefined → `null` fallback（全部門）
- `status` undefined → `"all"` fallback
- `page` undefined または invalid（0 / 負数 / 小数 / 非数値）→ `1` fallback（zod `.catch(undefined)` で吸収後、page 内で `page ?? 1`。50 §50.4 と同型、UI-06a-D1）
- `selected` undefined → `null` fallback（詳細カード閉）
- 不正値（例: `?status=invalid`、`?dept=abc`）→ zod `.catch(undefined)` で吸収、ユーザに通知せず黙って fallback（UI-09a/b 同方針）
- `selected` URL 化で詳細カード展開状態の F5 耐性を担保（detail query のみ再 fetch）
- 条件の変更（[50](50-ui-product-list.md) §50.4 の条件の key: `q`・`dept`・`status`・`discontinued`・`sort`・`dir`・`perPage`）で `page=1` に戻す。page 移動だけは条件を維持する。pagination は「すべて」（`search_products` 経路）だけで、「在庫切れ・在庫少」（`list_low_stock` の client filter 経路）には当てない（UI-01a-D15、UI-06a-D1）。今の build は条件の key が `q`・`dept`・`status` だけ
- **範囲外 page**（UI-06a-D3、round 1 P1-2 対応）: `items` 空 かつ `total_count > 0` かつ `page > 1` のとき、通常の EmptyState（filter-empty reset action を含む）ではなく専用メッセージ「このページには表示する商品がありません」+「先頭ページに戻る」ボタン（`page=1` へ navigate、50 §50.4 の条件の key は維持）を表示する。74 §74.10（UI-11c-D8）と同型。この判定は EmptyState 系より優先する（§58.10 参照）

#### `selected` ライフサイクル（race condition 回避。統合の画面の規則、D-138、proposed・未実装）

- **条件の変更**（`q`・`dept`・`status`・`discontinued`・`sort`・`dir`・`perPage`、[50](50-ui-product-list.md) §50.4）= `page=1` に戻し、`selected` を URL から clear する（`navigate` で `selected: undefined`）。新しい一覧の非同期な含有判定を避けて race を回避する。選択は「今の一覧の条件に対する状態」として扱い、stale な詳細を残さない。結果 1 件なら自動展開が再発火するので、絞り込みの動線は崩れない。page の移動は条件を変えず、`selected` は下の「今の一覧に無ければ clear」に従う
- 自動展開 useEffect の発火条件は `selected == null` ガードに加え、条件（上の key）と `page` の組ごとに 1 度だけ（UI-06a-D5、`useRef` に自動展開済みの条件 key を保持）。選択中行の再クリックで `onSelect(null)` が呼ばれ展開が閉じた（R2-3）後も、同じ条件のままでは自動展開が再発火しない。条件が変わった場合は新しい条件 key として改めて 1 度だけ自動展開する
- **list 成功時に `selected` が今の一覧の items に不在**（stale/手打ち URL、CSV 取込み invalidation 後の該当外化）= `selected` を clear（`navigate` で `selected: undefined`、C-P2-1）。行インライン展開（§58.8）の描画先消失を防ぎ「選択は今の一覧の条件に対する状態」を一貫させる。`listQuery.isSuccess` ガードで loading 中の誤判定を回避。同一条件で自動展開済みの場合は再展開せず収束しない（UI-06a-D5 の設計上の帰結、"selected 不在 = 詳細が必ず消える" ではないが自動での再表示も保証しない）。限定: `q` は部分一致なので、外から `q=<商品コード>&selected=<商品コード>` で開いたとき（入出庫 4 画面の「在庫を見る」〈[61](61-ui-receiving.md) UI-02-D10〉、在庫変動履歴の fallback〈[66](66-ui-stock-movements.md) UI-06c-D9〉）、その商品コードを含む商品が表示件数を超えて該当の商品が 1 ページ目に無いと、`selected` は外れる。一覧は `q` で絞られたまま残り、pager で届く（データは失わない）
- **検索語なしの `selected`**（例 `/products?selected=<商品コード>`、検索語なしの一覧から在庫変動履歴へ行って戻ったとき）= clear しない。D-138 で検索前の案内（旧契約 I）がなくなり、引数なしでも一覧が出るので、統合の前の検索前ガード（status=all かつ `q` 空で `selected` を clear し、detail query の `enabled` も止める）は要らない。その商品が今の一覧に無ければ上の規則で clear する
- 今の build（`/stock`）との違い: 条件の key は `q`・`dept`・`status` だけで、検索前ガード（`isAllEmpty`、Codex 実装レビュー Round 1 P2-2）がある（§58.5 の疑似コード）

### 58.5 hook 設計

#### useStockInquiry（3 useQuery 部分障害許容 + 正規化型）

```ts
import type { Department, ProductWithRelations, StockDetail } from "@/lib/bindings";
import type { DepartmentOption } from "@/components/patterns/DepartmentFilter"; // stock-inquiry/types.ts が re-export（59 §59.3）

export type StockInquiryListResult = {
  items: ProductWithRelations[];
  totalCount: number | null;       // source="search" 時のみ数値、source="low_stock" 時 null
  source: "search" | "low_stock";
};
// UI-06a-D1（2026-08-03）: `truncated` field は撤去済み。pagination 導入により
// 全件へページ送りで到達できるため、打ち切り告知フラグを持つ理由がなくなった。

export function useStockInquiry(params: {
  status: ListChipFilter;
  q: string;
  dept: number | null;
  page: number;                    // 1 始まり。status !== "all" では無視（既存 client filter 経路は非対象）
  perPage: number;                 // ローカル state、既定 50
  selected: string | null;
  navigate: (search: Partial<StockInquirySearch>) => void;
}) {
  const isAllEmpty = params.status === "all" && !params.q.trim();

  const listQuery = useQuery({
    queryKey: queryKeys.stockInquiry.list(params.status, params.q, params.dept, params.page, params.perPage),
    queryFn: async (): Promise<StockInquiryListResult> => {
      if (params.status === "all") {
        const data = await unwrapResult(
          commands.searchProducts({
            keyword: params.q.trim() || null,
            department_id: params.dept ?? null,
            is_discontinued: false,
            sort_key: "ProductCode",
            sort_order: "Asc",
            page: params.page,
            per_page: params.perPage,
          }),
          { source: "commands", cmd: "search_products" },
        );
        return {
          items: data.items,
          totalCount: data.total_count,
          source: "search",
        };
      }
      const rows = await unwrapResult(commands.listLowStock(false), {
        source: "commands",
        cmd: "list_low_stock",
      });
      const filtered = filterAndSortLowStockList(rows, params.q, params.dept, params.status);
      return { items: filtered, totalCount: null, source: "low_stock" };
    },
    enabled: !isAllEmpty,           // status=all + q 空文字 → search_products 呼ばない（契約 I）
    staleTime: 10_000,              // 10 sec（UI_TECH_STACK.md UI-06a 既定値、CSV 取込み直後の即時反映）
    gcTime: 5 * 60_000,
    retry: 1,
  });

  // UI-06a-D2（DSR-10 準拠、round 1 P1-3 対応）: 部門候補は listDepartments() の master 全件から作る。
  // page / q / dept / status のいずれにも依存しない単一 query とし、filtered result からの派生を禁止する。
  // implementation target（round 2 P1-1 対応）: src/lib/query-keys.ts の `stockInquiry.departmentOptions`
  // は現状 `(status, q)` 引数付き（["stock-inquiry", "department-options", { status, q }]）。本 change で
  // 無引数・一定 key（["stock-inquiry", "department-options"] as const 相当）へ変更する。対象は
  // src/lib/query-keys.ts / useStockInquiry.ts / useStockInquiry.test.tsx の 3 file。
  const departmentOptionsQuery = useQuery({
    queryKey: queryKeys.stockInquiry.departmentOptions(),
    queryFn: () => unwrapResult(commands.listDepartments(), {
      source: "commands",
      cmd: "list_departments",
    }),
    staleTime: 5 * 60_000,
    gcTime: 10 * 60_000,
  });
  // Department[] → DepartmentOption[] の変換は hook 側の責務（画面側は options={departmentOptions} を
  // そのまま渡すだけにする、useProductList.ts の departmentOptions 派生と同型）。
  const departmentOptions: DepartmentOption[] = (departmentOptionsQuery.data ?? [])
    .map((department) => ({ id: department.id, name: department.name }))
    .sort((a, b) => a.id - b.id);

  const detailQuery = useQuery({
    queryKey: queryKeys.stockInquiry.detail(params.selected ?? ""),
    queryFn: () => unwrapResult(commands.getStockDetail(params.selected!), {
      source: "commands",
      cmd: "get_stock_detail",
    }),
    enabled: !isAllEmpty && params.selected != null && params.selected.length > 0,
    staleTime: 10_000,
    gcTime: 5 * 60_000,
    retry: 1,
  });

  // 結果 1 件で自動展開（Q-3 補強）。selected == null ガードに加え、条件 key（status/q/dept/page）
  // ごとの「消費済み」ガードを持つ（UI-06a-D5、Codex P1-1/P1-2 是正）。消費済みは自動展開の
  // 発火時、または現一覧に実在する selected が観測された時点でのみ成立する（Codex round 2 P2
  // 是正: 非 null というだけで消費済みにすると、前ページの stale selected がページ送り後も
  // 維持されたまま新条件の単一結果を自動展開できなくなる）。条件 key が変われば消費済みフラグ
  // をリセットする（再訪時に改めて 1 度だけ自動展開する）。
  const conditionKey = `${params.status}|${params.q}|${String(params.dept)}|${String(params.page)}`;
  const items = listQuery.data?.items;
  const selectedBelongsToList =
    params.selected != null && (items ?? []).some((item) => item.product_code === params.selected);
  const autoExpandKeyRef = useRef(conditionKey);
  const autoExpandConsumedRef = useRef(selectedBelongsToList);
  useEffect(() => {
    if (autoExpandKeyRef.current !== conditionKey) {
      autoExpandKeyRef.current = conditionKey;
      autoExpandConsumedRef.current = selectedBelongsToList;
    } else if (selectedBelongsToList) {
      autoExpandConsumedRef.current = true;
    }
    if (items && items.length === 1 && params.selected == null && !autoExpandConsumedRef.current) {
      autoExpandConsumedRef.current = true;
      params.navigate({ selected: items[0].product_code });
    }
  }, [items, params.selected, conditionKey, selectedBelongsToList]);

  // selected を「現 list 条件に対する状態」に保つための clear（§58.4）。2 ケース:
  // (a) 検索前（isAllEmpty）に selected が残る（手打ち/F5/bookmark URL）→ detail 空振り防止で clear
  //     （Codex 実装レビュー Round 1 P2-2、detail enabled の !isAllEmpty guard と二重防御）。
  // (b) list 成功時に selected が現 list に不在 → 行インライン展開（§58.8）の描画先消失を防ぐ（C-P2-1）。
  //     isSuccess ガードで loading 中の誤判定を回避。同一条件 key で自動展開済みの場合は上の
  //     消費済みガードが再発火を止めるため再展開せず、収束しない（UI-06a-D5 の設計上の帰結）。
  //     条件 key が変われば新条件として改めて自動展開しうる。
  useEffect(() => {
    if (isAllEmpty && params.selected != null) {
      params.navigate({ selected: undefined });
      return;
    }
    if (
      listQuery.isSuccess &&
      params.selected != null &&
      !(items ?? []).some((item) => item.product_code === params.selected)
    ) {
      params.navigate({ selected: undefined });
    }
  }, [isAllEmpty, listQuery.isSuccess, listQuery.data, params.selected]);

  return { listQuery, detailQuery, departmentOptionsQuery, departmentOptions, isAllEmpty };
}
```

- **正規化の契約**: 自動展開 / Pagination は常に `listQuery.data.items` / `.totalCount` を参照（生 `data.total_count` / `data.items` 直接参照禁止、type narrowing 維持）
- list query 失敗時も detail query は独立動作（部分障害許容）、逆も同様。`departmentOptionsQuery` も同じく独立: list query の成否に関わらず候補取得は継続する
- 部門選択肢は `listDepartments()` の master 全件から作る（UI-06a-D2、DSR-10）。§58.4 の条件の key（50 §50.4）と `page` のいずれを変更しても候補集合は不変で、選択中部門から別部門へ直接切り替えられる。旧実装（filtered result からの派生）は Windows native L3 feedback で「個別部門選択後に他部門へ切り替えられない」functional defect として修正済みだった経緯があり、pagination 導入でこの縮退が再発・増幅するリスクがあるため、本 change で master 全件由来へ是正する
- **候補取得失敗時の挙動**（round 2 P1-1 対応、catalog ⑨「部門候補の取得失敗は呼び出し側で別途文言表示」の具体化）: `departmentOptionsQuery.isError` を hook が公開し、`StockInquiryPage` が `DepartmentFilter` 直下に「部門候補の取得に失敗しました」（`role="alert"`、products 画面 `ProductListPage.tsx` の既存文言と同型）を表示する。`listQuery`（一覧表示）とは完全に独立し、候補取得が失敗しても一覧表示は継続する

#### CSV 取込み後の invalidation（`useCsvImportFlow.ts` への追加）

`commands.commitCsvImport` / `rollbackCsvImport` 成功時は D-052-C8/C9 の SSOT helper を適用する。`stockInquiryRoot()` は list / detail を一括無効化する prefix factory だが、mutation 集合の具体列挙は `src/lib/invalidation-contract.ts` だけに置く。

### 58.6 純関数（テスト対象 4 + factory 1）

#### derive-stock-state（[item, source] → StockStatus）

色分け契約 H（§58.10）準拠（D-138 の後の形。今の build は `source === "search" && item.stock_quantity > 0` を `"ok"` にする）:
- `item.stock_quantity <= 0` → `"stockout"`（赤）
- `item.stock_quantity > 0` かつ在庫少の集合（`listLowStock(true)` の商品コード）に入る → `"low"`（黄）
- それ以外 → `"ok"`（default 表示、色なし。基準を上回る）
- 在庫少の集合が読めないとき（「すべて」で `listLowStock` だけが失敗）、`stock_quantity > 0` の行は `"unknown"`（状態の欄に「—」、契約 H）
- frontend に閾値定数を持たない（Q-1 + Round 4 P2-3 で 2 重確定、drift 源排除）

#### format-stock-display（[quantity, unit] → string）

- `unit: ProductStockUnit` を受け、10 個の code を型で網羅する（fallback の `"—"` を置かず、単位を足すと型検査で止まる。[共通規則](10-common-rules.md) SPEC-UNIT-D9・D11）。
- 個数の単位 → `"10 個"`・`"3 玉"`（数量 + D1 の表示の語。数量は `toLocaleString("ja-JP")` で桁区切り: `1,234 個`）
- 長さの単位（`m`）→ cm の整数を m で表す（`130` → `1.3 m`、`127` → `1.27 m`、`1234` → `12.34 m`、`300` → `3 m`。SPEC-UNIT-D3。整数の演算で作り、`toLocaleString` に小数を渡さない）

#### formatStockUnitLabel（[unit] → string、Gated Amendment 6 S45）

- 単位列（入庫等）や数量 input に添える単位（廃棄 / 返品交換 / 手動販売）の unit code を日本語ラベルへ変換する
- SPEC-UNIT-D1 の表示の語を返す（`pcs` → `"個"`、`ball` → `"玉"`、`m` → `"m"`）

#### 数量を出すほかの画面と formatter（D-113）

- 数量を出す画面の全部（売上・在庫変動・整合性チェック・PLU 書出しを含む）と、単位を持たない wire に単位を渡す方法は SPEC-UNIT-D3 の「表示する所」の表（owner の決定 TD-203）。
- 同じ file の `formatQuantityTotal`（集計の `5 点・1.8 m`）・`formatUnitQuantityTotal`（月次の商品別の行）・`formatUnitPrice`（`¥700/m`）・`formatCentiYen`（行の金額 `832.50 円`）・`formatLengthOrPlain`（長さの商品だけ m、個数の商品と単位の分からない行は語を付けない数）は、SPEC-UNIT-D3・D6 の形を 1 か所で持つ。

#### format-last-date（[value: string | null] → string）

- `value === null` → `"—"`（None 表示、Q-2）
- `value !== null` → `value` をそのまま返す（`YYYY-MM-DD`、DB_DESIGN.md 日付書式規約）

#### filterAndSortLowStockList（[items, q, dept, status] → ProductWithRelations[]、旧 filterLowStockList、UI-06a-D4 でリネーム）

- `status === "low_stock"` → 在庫少の集合から廃番を除いた全件（在庫切れと在庫少を 1 つの一覧に。UI-06a-D8。今の build は `stockout` → `stock_quantity <= 0`、`low_stock` → `stock_quantity > 0` に分ける）
- 加えて `q` あり → `product_code` / `name` / `jan_code` / `maker_code` の部分一致で絞り込み（「すべて」の `search_products` の keyword と同じ列。UI-06a-D8。今の build は `maker_code` を見ない）
- 加えて `dept` あり → `department_id === dept` で絞り込み
- フィルタ後、`supplier_name` 昇順（`localeCompare`、`null` は最後）→ `stock_quantity` 昇順 → `name` 昇順（`localeCompare`）で安定ソートする（UI-06a-D4、`status === "all"` の `search_products` 経路には適用しない）
- `list_low_stock` 返り値は 100 件以下想定で frontend filter/sort で高速（Q-1 user 確定の frontend 派生範囲内）

#### makeMockProductWithRelations / makeMockStockDetail（factory、test-fixtures.ts）

- `makeMockProductWithRelations(overrides: Partial<ProductWithRelations>): ProductWithRelations`（DTO flatten field + department_name / supplier_name）
- `makeMockStockDetail(overrides: Partial<StockDetail>): StockDetail`（product / last_receiving_date / last_sale_date）

### 58.7 UI コンポーネント

#### StockInquiryPage（最上位、失敗 4 状態出し分け）

- PageHeader: title「在庫照会」+ subtitle「商品ごとの在庫数と状態を確認し、その場で入出庫へ進みます」。
- 在庫少 / 在庫切れで `items.length > 0` のとき、一覧の直前（PaginationSummary の上）に「全 N 件」を `text-base font-semibold tabular-nums` で表示し、0 件では件数行を出さず EmptyState を表示する。

```tsx
function StockInquiryPage() {
  const { q, dept, status, page, selected } = Route.useSearch();
  const navigate = Route.useNavigate();
  const qValue = q ?? "";
  const deptValue = dept ?? null;
  const statusValue = status ?? "all";
  const pageValue = page ?? 1;
  const selectedValue = selected ?? null;
  const [perPage, setPerPage] = useState(50);
  const { listQuery, detailQuery, departmentOptionsQuery, departmentOptions, isAllEmpty } = useStockInquiry({
    status: statusValue, q: qValue, dept: deptValue, page: pageValue, perPage, selected: selectedValue,
    navigate: (search) => navigate({ search: (prev) => ({ ...prev, ...search }) }),
  });
  // q / dept / status の変更は page も既定（1）へ戻す（UI-06a-D1、50 §50.4 慣行の踏襲）。
  // page 移動（Pagination の onPageChange）は他条件を維持したまま page だけ更新する。
  const isFilterDefault = !qValue && deptValue === null && statusValue === "all";
  const resetFilters = () =>
    navigate({ search: (p) => ({ ...p, q: undefined, dept: undefined, status: undefined, page: undefined, selected: undefined }) });

  return (
    <div className="...">
      <SearchBar value={qValue} onSearchChange={(v) => navigate({ search: (p) => ({ ...p, q: v || undefined, page: undefined, selected: undefined }) })} />
      <StatusChips value={statusValue} onChange={(s) => navigate({ search: (p) => ({ ...p, status: s, page: undefined, selected: undefined }) })} />
      <DepartmentFilter
        options={departmentOptions}
        selected={deptValue}
        onChange={(d) => navigate({ search: (p) => ({ ...p, dept: d ?? undefined, page: undefined, selected: undefined }) })}
        disabled={departmentOptionsQuery.isLoading}
      />
      {/* 候補取得失敗は listQuery とは独立に別途文言表示（catalog ⑨、round 2 P1-1） */}
      {departmentOptionsQuery.isError && (
        <p className="text-sm text-destructive" role="alert">部門候補の取得に失敗しました</p>
      )}
      {isAllEmpty ? (
        <EmptySearchPlaceholder />
      ) : listQuery.isLoading ? (
        <Skeleton />
      ) : listQuery.isError ? (
        <>
          <Alert variant="destructive">在庫一覧の取得に失敗しました</Alert>
          {/* list 失敗時は行インライン展開できないため、フォールバックカードで詳細を独立描画（部分障害許容、§58.8） */}
          {selectedValue && <StockDetailCard query={detailQuery} />}
        </>
      ) : listQuery.data!.items.length === 0 && listQuery.data!.totalCount !== null && listQuery.data!.totalCount > 0 && pageValue > 1 ? (
        // 範囲外 page（UI-06a-D3、74 §74.10 UI-11c-D8 と同型）。通常 EmptyState / reset action より優先判定（SPEC-UIBB-8）。
        <EmptyState
          title="このページには表示する商品がありません"
          action={
            <Button variant="outline" onClick={() => navigate({ search: (p) => ({ ...p, page: undefined }) })}>
              先頭ページに戻る
            </Button>
          }
        />
      ) : listQuery.data!.items.length === 0 ? (
        <EmptyState
          title="該当する商品がありません"
          // 絞り込みが既定値以外のときだけ reset action を出す（catalog ⑥ filter-empty reset action、SPEC-UIBB-1/2）。
          action={!isFilterDefault ? <Button variant="outline" onClick={resetFilters}>絞り込みを解除</Button> : undefined}
        />
      ) : (
        <>
          {/* list 成功時は選択行の直下に詳細をインライン展開する（detailQuery を渡す、§58.7 ProductListTable） */}
          <ProductListTable
            items={listQuery.data!.items}
            source={listQuery.data!.source}
            selected={selectedValue}
            detailQuery={detailQuery}
            onSelect={(code) => navigate({ search: (p) => ({ ...p, selected: code ?? undefined }) })}
          />
          {/* status === "all" のときだけ表示（UI-06a-D1、02-component-catalog.md ⑩ canonical を結線） */}
          {statusValue === "all" && listQuery.data!.totalCount !== null && (
            <Pagination
              page={pageValue}
              totalCount={listQuery.data!.totalCount}
              perPage={perPage}
              onPageChange={(next) => navigate({ search: (p) => ({ ...p, page: next }) })}
            />
          )}
        </>
      )}
      {/* 最下部固定カードは廃止。詳細は list 成功=選択行直下インライン展開 / list 失敗=フォールバックカードの
          2 経路で描画し、selected が現 list に不在なら useStockInquiry が clear する（§58.4 / §58.8、C-P2-1） */}
    </div>
  );
}
```

#### SearchBar（HID スキャナ前提、Q-3 補強）

- placeholder「商品コード・商品名・JANで検索」
- `autoFocus`（初期表示時に検索 input へ focus、HID スキャナがそのまま入力可能）
- `onChange` debounce 200ms で URL state `q` 更新、Enter 押下時は debounce flush して即時更新
- 専用「スキャン」ボタンは入れない（§58.13 非目的）

#### StatusChips（2 チップ、件数バッジなし、Q-5）

- shadcn `ToggleGroup`（`type="single"`）、「すべて」/「在庫切れ・在庫少」のラベルのみ（UI-06a-D8。今の build は「すべて」/「在庫切れ」/「在庫少」の 3 つ）
- 件数バッジは表示しない（検索条件・部門フィルタとの結合で意味論が壊れる、§58.13）
- 選択中 tone は shared stone selection tone（solid stone-700 ではない）+ 太字。stockout / low の rose / amber は在庫状態の補助色として残し、filter selection の背景色と意味色を衝突させない

#### DepartmentFilter（共通 `patterns/DepartmentFilter`、2026-08-03 batch B で sync）

- 共通 `src/components/patterns/DepartmentFilter.tsx`（shadcn `Select`、「すべての部門」+ `commands` 由来の部門一覧）を使う。`DepartmentOption` 型は `patterns/DepartmentFilter.tsx` が単一定義し、stock-inquiry 側は re-export を import する（[59-ui-shared-patterns.md](59-ui-shared-patterns.md) §59.3、PR-B で 3 feature 統合済み）
- 候補は `listDepartments()` master 全件（UI-06a-D2、DSR-10）で、§58.4 の条件の key（50 §50.4）と `page` に関わらず不変（今の build の key は `page` / `q` / `dept` / `status`）。個別部門選択中も他部門候補が縮退しない。`DepartmentFilter` 自体は pure component とし、候補取得は `useStockInquiry` の `departmentOptionsQuery` が担う
- **canonical props 結線**（round 2 P1-1 対応、59 §59.1 の `{options, selected, onChange, disabled?}` 準拠）: `StockInquiryPage` は `options={departmentOptions}`（hook が `Department[] → DepartmentOption[]` へ変換済みの配列）/ `selected={deptValue}`/ `onChange`（URL state `dept` 更新）/ `disabled={departmentOptionsQuery.isLoading}` を渡す。候補取得失敗時は `departmentOptionsQuery.isError` を見て `DepartmentFilter` 直下に「部門候補の取得に失敗しました」を別途表示し、`listQuery`（一覧表示）とは独立に一覧表示は継続する（§58.5）

#### ProductListTable（高視認性状態表示契約 H、選択行直下インライン展開）

- 列: 商品コード / 商品名 / 部門 / 取引先 / 状態 / 在庫数（`format-stock-display`、生地は単位付き）/ 売価（UI-06a-D4、7 列。今の build。統合の後の列は状態で分ける〈50 UI-01a-D16〉）
- 取引先列（`supplier_name`）は「すべて」と「在庫切れ・在庫少」のどちらの列の組にも出す（列の組は状態で分ける、50 UI-01a-D16）。値が `null` の場合は `—` を表示する。今の build は `source` で列構成を条件分岐しない単一 component で、3 つの view に同じ 7 列を出す
- `source` prop を `derive-stock-state(item, source)` に引き渡し、状態列で `StockStatusBadge` を表示する
  - `stockout` = `CircleAlertIcon` + 「在庫切れ」Badge
  - `low` = `TriangleAlertIcon` + 「在庫少」Badge
  - `ok` = muted 「在庫あり」Badge
- 在庫数セルの stockout red / low yellow / ok default は二次シグナルとして残す。意味そのものは状態列の日本語ラベルで伝える
- H-6 feedback 対応として、商品コードセルと詳細 header の商品コードは `font-mono text-sm font-medium` とする。旧 `text-xs` は最小級で、全体 WebView 表示スケール導入後も読みづらさが残るため使わない
- 行クリックで `onSelect(isSelected ? null : product_code)` 発火（UI-06a-D5、R2-3 展開行トグルクローズ） → `selected` URL state 更新（`code ?? undefined`） → **選択行の直下**に colSpan 展開行で `StockDetailContent` をインライン描画（`detailQuery` props を受け取る、collapsible 不使用 = 条件描画）。選択中行を再クリックすると `onSelect(null)` が呼ばれ展開が閉じる。展開行は `bg-muted` を明示固定し選択行と視覚的に一体化（New-1）。list 失敗時の独立描画はフォールバック `StockDetailCard` が担う（§58.8）
- インライン展開の `colSpan` は表の列数と同じにする（統合の後は「すべて」8・「在庫切れ・在庫少」7、50 UI-01a-D16。今の build は 7）。列数と違う `colSpan`（旧 6・5 を含む）は regression test で防ぐ

#### EmptySearchPlaceholder（契約 I）/ Pagination（UI-06a-D1）

- EmptySearchPlaceholder: 今の build だけの部品（status=all + q 空文字時、centered muted text「商品コード、商品名、または JAN コードで検索してください」）。D-138 で旧契約 I とともに撤去する（50 UI-01a-D1）
- ~~TruncatedResultsAlert~~: **2026-08-03 batch B（UI-06a-D1）で撤去**。pagination 導入により全件へページ送りで到達できるため、「他にも検索結果があります」の打ち切り告知は不要になった（打ち切り告知とページ送りの二重表現を避ける）
- Pagination: `status === "all"` のときだけ表示（在庫切れ・在庫少〈`status=low_stock`〉は `list_low_stock` の client filter 経路のため対象外、既存挙動）。02-component-catalog.md ⑩ の canonical をそのまま結線し、`totalCount` から最終ページを計算する

#### StockDetailContent / StockDetailCard（インライン展開 + フォールバック共用、在庫変動履歴 link）

- `StockDetailContent`: 詳細の内側描画（在庫数 / 売価 / 原価 / 最終入庫日（`format-last-date`）/ 最終販売日（`format-last-date`））+ isLoading/isError/data 全状態を内包。行インライン展開（list 成功時、`ProductListTable` の colSpan 展開行内、Card なし）と フォールバックカード（list 失敗時、`StockDetailCard` が Card で包む）で**共用**
- `StockDetailCard`: `StockDetailContent` を Card で包む薄いラッパ（list 失敗 + selected != null のフォールバック専用、collapsible 不使用）
- `detailQuery.isError` → `StockDetailContent` 内 inline エラー表示（部分障害許容、一覧は維持。両描画経路で共通）
- 「商品修正」は `/products/$code/edit` への active link とし、`params.code` に対象商品コードを、`returnTo` に統合の画面の search state を `buildProductListReturnTo` で組んだ値（`status`・`selected` を含む）を渡す（50 UI-01a-D17、[51](51-ui-product-form.md) UI-01b-D2。保存・戻るで来た一覧とその行へ戻る）。今の build は `returnTo` を渡さず、保存の後は商品一覧へ着く。
- 「入庫記録」は `/inventory/receiving` への active link とし、search の `productCode` に対象商品コードを渡す（61 UI-02-D18。その商品を明細に 1 行入れた状態で開く）。廃番の商品の詳細にはこの導線を出さない（入庫の商品追加欄は廃番を出さない）。今の build は事前入力しない。
- 「在庫変動履歴」は UI-06c で active link 化し、`/stock/$productCode/movements` へ遷移する。現在の画面の URL（`location.href`、search state 込み）を `returnTo` として送る（UI-06a-D7）。2 つの導線で作る側の規約が違う: 在庫変動履歴の `returnTo` は受け側（66 UI-06c-D9）が `returnToLinkProps` で解いて router の search object で戻すので、router の直列化のままの `location.href` でよい。商品修正の `returnTo` は受け口が router の parseSearch を通らない typed parse-back（51 UI-01b-D2）なので、手組みの `buildProductListReturnTo` で組む（router の直列化は数字だけの値を引用符つきで出し、受け口が引用符ごと受ける）。UI-06c の画面本体、filter、pagination、`MovementRecord.source` 元記録リンクは [66-ui-stock-movements.md](66-ui-stock-movements.md) を正とする。

### 58.8 エラー処理（失敗 4 状態の網羅）

**詳細（StockDetailContent）の描画経路（行インライン展開 + フォールバック + clear）**:
- **list 成功 + selected が items に含まれる** → `ProductListTable` が選択行直下に colSpan 展開行でインライン描画（`StockDetailContent` 直接、Card なし）
- **list 成功 + selected が items に不在** → `useStockInquiry` が `selected` を clear（§58.4、C-P2-1）。展開先・detail query とも自然消滅
- **list 失敗 + selected != null** → Alert 下のフォールバック `StockDetailCard`（`StockDetailContent` を Card で包み独立描画 = 部分障害許容、Codex Round 1 P2-1 の契約を構造変更後も維持）
- 3 経路とも `StockDetailContent`（isLoading/isError/data 全状態を内包）が描画を担い、detail 失敗時の inline エラーも各経路内で表示する

| 状態 | 描画 | 復旧手段 |
|---|---|---|
| **#1 list query API fail**（`listQuery.isError`） | ProductListTable 領域 `<Alert variant="destructive">` + toast（`id: "stock-inquiry-list-error"`） | 検索条件変更 or ページ再読込、リトライ成功で同 id `toast.dismiss` |
| **範囲外 page**（UI-06a-D3、`items` 空 + `total_count > 0` + `page > 1`、round 1 P1-2 対応） | `<EmptyState title="このページには表示する商品がありません" action={<Button variant="outline">先頭ページに戻る</Button>} />`。74 §74.10（UI-11c-D8）と同型。**#2 の filter-empty reset action より優先して判定する**（SPEC-UIBB-8） | 「先頭ページに戻る」押下で `page=1`、50 §50.4 の条件の key は維持 |
| **#2 list query items 空配列** | `<EmptyState>` の見出し・説明・action は 50 §50.7 の Empty の順（条件に合わせた説明、絞り込みが既定値以外なら「絞り込みを解除」、商品が 1 件も無い DB は「商品がありません」と「商品を登録する」。UI-01a-D18、catalog ⑥ filter-empty reset action、SPEC-UIBB-1/2）。今の build は一律に「該当する商品がありません」で、q / dept / status が既定値以外なら「絞り込みを解除」 | 条件の変更、または reset action（50 §50.7 の既定値への一括復帰） |
| **#3 detail query fail**（`detailQuery.isError`） | 詳細の inline エラー（一覧は維持、部分障害許容）。list 成功時は選択行直下のインライン展開行内、list 失敗時はフォールバック StockDetailCard 内（両系統とも `StockDetailContent` が isLoading/isError/data を内包）+ toast（`id: "stock-inquiry-detail-error"`） | 別商品選択 or リトライ、成功で同 id `toast.dismiss` |
| **#4（D-138 で撤去）** | 今の build だけ: status=all + q 空文字で `<EmptySearchPlaceholder />`（search_products 呼ばない、旧契約 I）。統合の後は初めて開いたら一覧（50 UI-01a-D1・D18） | — |
| **在庫少の集合の取得失敗**（統合の後の「すべて」） | 一覧は出し、`stock_quantity > 0` の行の状態の欄に「—」、一覧の上に「在庫少かどうかを読めませんでした。状態の欄は在庫切れだけを示しています」と「もう一度読む」（契約 H） | 「もう一度読む」で `listLowStock` を再取得 |

- `commands.getStockDetail` の `BizError::NotFound`（商品コード不正）は CmdError `kind: "not_found"` で frontend に伝播、`InvokeError` で型安全に処理（`src/lib/invoke.ts`）

### 58.9 テスト戦略（Vitest 純関数 + hook + RTL。ケース実数は D-038 Evidence Ownership により PR body / CI 出力を正とし本表に転記しない）

| file | 主内容 |
|---|---|
| `derive-stock-state.test.ts` | source=search + stock>0 → ok / source=search + stock<=0 → stockout / source=low_stock + stock>0 → low / source=low_stock + stock<=0 → stockout / stock=0 境界 |
| `format-stock-display.test.ts` | 10 個の code の表示の語（`"pcs"` → 「個」、`"m"` → 「m」）と長さの m の形、原価の formatter（`¥83.33`） |
| `format-last-date.test.ts` | null → 「—」/ `YYYY-MM-DD` そのまま / 空文字扱い |
| `filter-low-stock-list.test.ts` | stockout 分岐 / low_stock 分岐 / q 部分一致 / dept 絞り込み / 複合 / 空配列 / `filterAndSortLowStockList`: 取引先名昇順（null 最後）→ 在庫数昇順 → 商品名昇順ソート（UI-06a-D4） |
| `useStockInquiry.test.tsx` | search → PaginatedResult 正規化（source/totalCount、`truncated` は撤去済み） / low_stock → 配列正規化 / status=all+q空 で enabled=false / 1 件自動展開 / status 切替 → selected clear → 新 list 1 件で再展開 / detail 部分障害 / list 成功 + selected 不在 → clear（C-P2-1） / isAllEmpty + selected → clear + detail 非発火（Round 1 P2-2） / page が queryKey とクエリ引数に反映される（SPEC-UIBB-3/4） / SPEC-UIBB-9: `departmentOptionsQuery` が `listDepartments()` を呼び、page/q/dept/status 変更後も候補が不変で選択中部門から別部門へ直接切替できる（round 1 P1-3、DSR-10）。status 変更（all → low_stock → stockout）も候補不変であることを追加 assert（round 2 P2-3）。同一 `QueryClient` 上で page/q/dept/status を変えても `listDepartments` の call count = 1 に留まる（round 2 P2-3、query-key 安定性の mutant 検出）。`queryKeys.stockInquiry.departmentOptions()` が無引数で呼ぶたびに同一・一定の key を返す unit test（round 2 P1-1、無引数化の regression 防止） / 手動クローズ後、同一検索条件では自動展開が再発火しない・条件変化後は再度発火する対 test（UI-06a-D5） / ページ送りで前ページの stale selected が維持されたままでも新条件の単一結果が自動展開する（round 2 P2 是正、fix 前 FAIL） |
| `SearchBar.test.tsx` + `StockInquiryPage.test.tsx` | `autoFocus` 検証 / Enter で debounce flush + 即時 search / 結果 1 件で自動展開 useEffect → URL state `selected` 更新 / list 成功 + selected でインライン展開 / 行クリックで selected 更新 → 展開（stateful harness、C-P2-3）/ list 失敗 + detail 成功でフォールバックカード独立描画（部分障害許容、Codex Round 1 P2-1）/ search flow の在庫切れ label / low_stock flow の在庫少 label（RTL + user-event）/ SPEC-UIBB-1/2: 絞り込み非既定+0件で reset action 表示・押下で全条件+page 既定復帰 / SPEC-UIBB-4: q・dept・status 変更で page=1、page 移動は条件維持 / SPEC-UIBB-5: 51 件 synthetic で page 2 に到達、`TruncatedResultsAlert` 残存 0（rg 静的 sweep） / SPEC-UIBB-8: `items` 空 + `total_count > 0` + `page > 1` で範囲外 page 専用メッセージ + 「先頭ページに戻る」を表示し、filter-empty reset action より優先判定される（UI-06a-D3、round 1 P1-2） / SPEC-UIBB-9: 候補 query pending 中は `DepartmentFilter` trigger が disabled（`departmentOptionsQuery.isLoading`、round 3 P2-3） / SPEC-UIBB-9: `listDepartments` reject + list query 成功で `role="alert"`「部門候補の取得に失敗しました」と商品一覧が**同時に**表示される（一覧独立の結合退行検出、round 3 P2-3）。test harness は QueryClient retry を無効化して失敗状態を一意に確定させる |
| `ProductListTable.test.tsx` | 状態列の「在庫切れ」「在庫少」「在庫あり」text / 商品コード cell `text-sm` readability guard / 選択行直下インライン展開 / nextElementSibling colSpan=7 guard（旧下部固定・旧 5/6 列混入検出）/ 非選択時展開なし / detail 失敗 inline（C-P2-3） / 展開行 whitespace-normal guard（Round 1 P2-1） / 取引先列ヘッダ・null 表示・両 view 共通表示（UI-06a-D4） / 選択中行の再クリックで `onSelect(null)` 発火（展開行トグルクローズ、UI-06a-D5） |
| `StatusChips.test.tsx` | selected chip の `data-state="on"` / chip click の filter value 発火 / deselect 空文字無視（常に 1 つ選択維持） |

`vi.mock("@/lib/bindings")` で commands mock + TanStack Router test wrapper（memory `feedback-vitest-react19-setup-pattern.md` 踏襲）。状態表示のテストは Tailwind color class ではなく text / DOM state / table structure を assert する。

### 58.10 業務ルール

#### Q-1 在庫少判定（frontend は閾値を持たない）

- frontend 責務: BIZ が返した集合を表示・分類
- BIZ 責務: 何が「在庫少」かを判定（app_settings の `stock_low_threshold` / `stock_low_threshold_fabric` を読み `list_low_stock` で適用）
- frontend に持ってよい派生: `list_low_stock` 結果内で `stock_quantity <= 0` なら「在庫切れ」、`> 0` なら「在庫少」
- frontend に持ってはいけない派生: `stock_unit === "m" ? 500 : 3` のような閾値判定（drift 源）

#### 契約 H 在庫状態の高視認性表示（[design-system/00-foundations.md §業務ステータスの視認性](../design-system/00-foundations.md)）

> （D-138 で置換え）どの状態でも、stock_quantity <= 0 は「在庫切れ」、stock_quantity > 0 で BIZ 判定済みの在庫少の集合（`list_low_stock`）に入る商品は「在庫少」、それ以外は「在庫あり」と表示する。「すべて」では一覧（`search_products`）とは別に `list_low_stock(true)`（廃番を含む。「すべて」は廃番の表示を持つため）を読み、商品コードが集合に入るかで決める。「在庫あり」は在庫少の基準（[69](69-ui-threshold-settings.md) の 2 つの key、`stock_quantity <= 基準` が在庫少）を上回ることを示す。「在庫切れ・在庫少」の一覧は同じ集合から廃番を除く。frontend は閾値を保持しない（Q-1）。在庫少の集合の読込み中は、「すべて」の stock_quantity > 0 の行の状態の欄を読込み中の印（Skeleton）にする。在庫少の集合が読めないとき、「すべて」の stock_quantity > 0 の行は状態の欄に「—」を出し、一覧の上に読めなかったことと「もう一度読む」を出す（「在庫あり」と言い切らない）。意味は状態列の日本語ラベル + icon + Badge で伝え、赤 / amber は補助シグナルとして使う。在庫少の集合の query の key は D-052 の invalidation の集合に入れる: `src/lib/invalidation-contract.ts` で `queryKeys.lowStock(false)` を挙げる entry の全部（`rg -c 'queryKeys.lowStock\(false\)' src/lib/invalidation-contract.ts` の出力 16。入出庫 4 画面・CSV 取込み・商品の保存・閾値の保存・棚卸しの数の保存ほか）に足し、test 側の独立 oracle（`src/test/invalidation-oracle.ts`）も同じ PR で直す（`queryKeys.lowStock(true)` は prefix でも `lowStock(false)` に当たらない。足さないと在庫の数は新しく状態の欄だけ古い）。今の build は「すべて」で閾値判定を行わず、stock_quantity > 0 を「在庫あり」と出す（基準以上を保証しない）。
>
> Why: 入れた後に時々、限られた商品について在庫が最低限要る数に達したかを確かめる（店主の答え Q-102・TD-423）。入庫の結果から開いた 1 商品の行（[61](61-ui-receiving.md) UI-02-D10）が「すべて」の一覧なので、「すべて」の状態の欄が基準との比べを示さないと、この確かめに答えない。新しい設定・field は足さない（基準は今の 2 つの key）。

| 一覧 | stock_quantity <= 0 | stock_quantity > 0 で在庫少の集合に入る | stock_quantity > 0 で集合に入らない | 集合が読めない |
|---|---|---|---|---|
| 「すべて」（`search_products` + `list_low_stock(true)`） | `CircleAlertIcon` + 「在庫切れ」Badge + 在庫数 red 補助 | `TriangleAlertIcon` + 「在庫少」Badge + 在庫数 amber 補助 | 「在庫あり」Badge + 在庫数 default | 状態の欄「—」 + 一覧の上の知らせ |
| 「在庫切れ・在庫少」（`list_low_stock` から廃番を除く） | 同上 | 同上 | （出ない） | 一覧の読込みの失敗（§58.8 #1） |

#### 契約 I（D-138 で撤去）

> 「すべて」で検索語が空なら一覧を取らず案内だけを出す形は、50 UI-01a-D1（初めて開いたら全商品の一覧）で置き換えた。perPage は URL の `perPage`（50 / 100 / 200、既定 100、50 §50.4）。per_page 上限は search_products 既存契約どおり 200 で、200 超は IO 層でクランプされる。全件へは pagination（§58.4 / UI-06a-D1）でページ送りして到達する。今の build は案内だけを出し、perPage はローカル state（既定 50）。

#### UI-06a-D1: pagination 導入 + truncated alert 撤去（2026-08-03 batch B）

- **決定**: 在庫照会「すべて」（`status === "all"`）に `page` search param と `Pagination`（既存 canonical、02-component-catalog.md ⑩、旧 `ProductPagination`）を導入し、`total_count` から全ページへ到達できるようにする。これに伴い `TruncatedResultsAlert` component と `StockInquiryListResult.truncated` flag を撤去する。
- **Why**: pagination により全件へページ送りで到達できるようになるため、「他にも検索結果があります」という打ち切り告知（旧 契約 I）はページ送りと二重表現になる。50 §50.4（商品一覧）の既存 page 慣行をそのまま踏襲し、在庫照会だけの新しい UX を発明しない。
- **Rejected**: truncated alert を pagination と併存させる案（「打ち切り告知」+「ページ送り」が同じ問題を二重に説明することになり、利用者が両方を読む必要が生じる）。在庫少 / 在庫切れ（`list_low_stock` 経路）への pagination 拡張（既存 100 件以下想定の client filter で十分、対象外のまま据え置き）。
- **Revisit trigger**: `list_low_stock` の返却件数が 100 件を恒常的に超える運用が確認された場合、client filter 経路への pagination 適用を再検討する。

#### UI-06a-D6: 絞り込み時の件数表示（2026-09-15、SPEC-DISP-B2-1 / D-B5）

- 在庫切れ・在庫少の絞り込み時は検索・部門・状態の client filter 後の `items.length` を N とする「全 N 件」のみを表示し、pagination は追加しない（0 件は EmptyState のみ）。

#### UI-06a-D7: 在庫変動履歴への `returnTo` 送信（2026-09-16、NAV-1）

- **決定**: 「在庫変動履歴」link は現在の画面の URL（search state 込み。統合の後は `/products`、D-138。今の build は `/stock`）を `returnTo` として送る。`StockDetailContent` の `ActiveCta` に `useRouterState({ select: (state) => state.location.href })` の現在 href を `search={{ returnTo }}` で渡す（`src/features/disposal/DisposalPage.tsx` の `DisposalPage` の `returnTo` と同型、新しい state を作らない）。
- **Why**: 元の検索条件と開いていた行へ戻す。起点の監査 NAV-1 は、`/stock?selected=<code>` だけでは受け側の検索前ガード（統合の前の §58.4）が `selected` を解除し、利用者が検索と選択をやり直すことだった。DSR-18 の判定フロー「業務記録詳細へ遷移する link か？ No → その導線固有の契約」に該当するため、DSR-18 本文は変えずに 58 / 66 の局所契約として置く。
- **Rejected**: `/products?q=<code>&selected=<code>` を常に送る（元の検索条件を捨てる）。統合の前の Rejected「受け側の検索前ガードを撤去」は、D-138 でガードそのものが要らなくなった（引数なしでも一覧が出る、§58.4）ので外した。

#### UI-06a-D2: 部門候補を listDepartments master 全件へ是正（DSR-10、round 1 P1-3、2026-08-03 batch B）

- **決定**: 部門候補 query を検索結果由来（`searchProducts` 先頭 page からの派生）から `commands.listDepartments()` の master 全件へ切り替える。§58.4 の条件の key（50 §50.4）と `page` のいずれを変更しても候補集合は不変にする（今の build の key は `page` / `q` / `dept` / `status`）。
- **Why**: filtered result 由来の候補縮退は DSR-10 / 02-component-catalog.md ⑨「フィルタ候補のソース」で禁止されている。pagination 導入により「現在 page の商品に含まれる部門」への縮退リスクが増幅するため、同一 change で正本準拠に是正する。wire 変更なし（`listDepartments` は既存 command）。
- **Rejected**: 現状維持（dept 選択時だけ同じ q/status で dept を外した候補用 query を使う案）。個別部門選択後に他部門へ切り替えられない機能不全を過去 L3 で一度是正した実績があり、pagination 追加でさらに再発しやすくなるため据え置かない。

#### UI-06a-D3: 範囲外 page 回復（round 1 P1-2、2026-08-03 batch B）

- **決定**: `items` 空 かつ `total_count > 0` かつ `page > 1` のとき、通常の EmptyState / filter-empty reset action ではなく専用メッセージ「このページには表示する商品がありません」+「先頭ページに戻る」ボタン（`page=1` へ navigate、他条件維持）を表示する。判定は EmptyState 系より優先する。
- **Why**: 74 §74.10（UI-11c-D8）と同型の契約を踏襲し、在庫照会だけの新しい UX を発明しない。50 §50.4 / `Pagination.tsx`（旧 `ProductPagination.tsx`）に clamp 契約は存在しないため、範囲外 page を無言で先頭ページへ丸める（clamp）案は既存 canonical の慣行から逸脱する。
- **Rejected**: 範囲外 page を `page=1` へ自動 clamp する案（既存 50 / 旧 ProductPagination（現 Pagination）に clamp 契約が実在しないため新規発明になり、利用者に無断で条件を書き換える点でも UI-11c-D8 の明示回復導線パターンと不整合）。

#### UI-06a-D4: 在庫少・在庫切れ一覧の取引先列 + 取引先名優先ソート（2026-09-06）

- **決定**: `ProductListTable` に取引先列（`supplier_name`、null は `—`）を追加する。取引先列は「すべて」と「在庫切れ・在庫少」のどちらの列の組にも出る（D-138 で列の組を状態で分けた、50 UI-01a-D16。D-138 の前は単一 component で 3 つの view に無条件で出していた）。並び順は `status === "low_stock"`（`list_low_stock` 経路）でのみ `filterAndSortLowStockList` が取引先名昇順（null 最後）→ 在庫数昇順 → 商品名昇順にクライアント側でソートする。「すべて」（`search_products` 経路）の並び順とページは [50](50-ui-product-list.md) §50.4 に従う。
- **Why**: `list_low_stock` は既に pagination なしの全件クライアント保持（§58.5）のため、フィルタ後配列を並べ替えるだけで済むクライアント側ソートが SQL `ORDER BY` 変更より変更面積が小さい（新規 Rust test・IO 層契約変更・design doc 改訂が不要）。
- **Rejected**: `list_low_stock_products`（`product_repo.rs`）の SQL `ORDER BY` を変更する案（IO 層契約変更・新規 Rust test・`20-io-product-repo.md` 改訂を伴い変更面積が大きい上、「すべて」に影響させない制御が別途必要になる）。

#### UI-06a-D5: 展開行トグルクローズ + 自動展開の検索条件単位制限（R2-3、2026-09-06）

- **決定**: `ProductListTable` の行クリックを、選択中行の再クリックで `onSelect(null)` を呼ぶよう変更する（`selected` URL state が `undefined` になり展開が閉じる）。自動展開 `useEffect`（§58.4）に、§58.4 の条件の key（50 §50.4）と `page` から導出した key を `useRef` で保持するガードを追加し、同じ条件で一度自動展開したら再展開しない。条件が変われば新しい key として改めて 1 度だけ自動展開する。
- **Why**: 既存の自動展開は `selected === null` ガードのみのため、再クリックで `selected` を `null` に戻しても同じ検索条件で結果がまだ 1 件なら即座に再展開し、「閉じる」操作を打ち消してしまう。`useRef` は URL 一本化の既存方針（§58.4）に追加の state を持ち込まずに済む。
- **境界（意図的、defect ではない）**: `useRef` はコンポーネントインスタンスのメモリ上のため、full remount（サイドバー遷移で商品と在庫の画面を離れて戻る等。今の build は `/stock`）が起きると ref は初期化され、同じ検索条件でも自動展開が再度発火する。1 セッション内で条件を変えずに何度も開閉する操作を対象にした guard であり、画面を離れて戻る操作まで記憶する要件ではない。
- **conditionKey の構成要素**: §58.4 の条件の key（50 §50.4 の `q`・`dept`・`status`・`discontinued`・`sort`・`dir`・`perPage`。統合の画面では `perPage` も URL の key）と `page`（D-138）。`q` は `trim()` せず生値をそのまま key に使う。今の build は URL 由来の `status`/`q`/`dept`/`page` の 4 項目（`useStockInquiry.ts` の `conditionKey`）で、`perPage` は page ローカルの state で対象外。
- **Rejected**: `selected` の変化元（ユーザー操作 vs 自動展開）を別 state で追跡する案（state 数が増え、URL 一本化の既存方針に反する）。

#### UI-06a-D8: 状態のチップを 2 つにし、在庫切れ・在庫少を 1 つの一覧にする（D-138、2026-10-24）

- **決定**: 状態のチップは「すべて」「在庫切れ・在庫少」の 2 つ。「在庫切れ・在庫少」は在庫少の集合（廃番を除く）を 1 つの一覧に出し、切れか少かは状態の列で見分ける。列は取引先・メーカー品番・商品コード・商品名・状態・在庫数・売価（50 UI-01a-D16）、並びは UI-06a-D4。検索欄は「すべて」と同じ 4 つの列（商品コード・商品名・JAN・メーカー品番）に当たる。
- **Why**: 発注の準備は在庫切れと在庫少を 1 枚で、取引先ごとに見る（repo 外の回答台帳 QU-004・L-130）。発注書に書く番号はメーカー品番（L-174・TD-362）。同じ検索欄が状態によって当たる列を変えない（packet 段 1 の X-05、Fluent 2 *Coherent*）。
- **Rejected**: 3 つのチップを残す（在庫切れだけ・在庫少だけの一覧を行き来する操作が残る）。
- **失うもの**: 在庫切れだけ・在庫少だけの一覧（状態の列で見分ける）。

#### 廃番除外（D-138 で置換え）

- 「在庫切れ・在庫少」の一覧は廃番を除く（SP-302-04、`list_low_stock` の結果から `is_discontinued` を除く）。「すべて」は 50 の廃番の表示（表示中 / すべて / 廃番のみ、既定は表示中）に従う。在庫少の集合は廃番を含めて読む（契約 H）。今の build は `include_discontinued = false` 固定で UI トグルなし。

### 58.11 ショートカット

- グローバル `Ctrl+/`: ShortcutsDialog（UI-shortcuts で実装済、本画面では追加 hook 不要）
- 画面固有ショートカット: 検索欄 `autoFocus` + Enter 検索（Q-3 補強）。それ以外は本 Phase では未定義

### 58.12 表記揺れ + UI 表示フォーマット

数量は `toLocaleString("ja-JP")` による桁区切りで表示する（例: `1,234 個`）。

| 系統 | 表示 | 内部 |
|---|---|---|
| 在庫数（個物） | 「10 個」 | `stock_quantity` integer + `stock_unit = "pcs"` |
| 在庫数（生地） | 「3 m」 | `stock_quantity` integer（cm）+ `stock_unit = "m"` |
| 在庫数（unexpected unit） | 「—」 | fallback（Q-4） |
| 最終入庫日 / 最終販売日 None | 「—」（Q-2） | `string \| null`（null） |
| 在庫状態（在庫切れ） | `CircleAlertIcon` + 「在庫切れ」Badge（在庫数 red は補助） | `StockStatus = "stockout"` |
| 在庫状態（在庫少） | `TriangleAlertIcon` + 「在庫少」Badge（在庫数 amber は補助） | `StockStatus = "low"`（在庫少の集合に入る。契約 H） |
| 在庫状態（在庫あり） | 「在庫あり」Badge | `StockStatus = "ok"` |
| 金額（売価） | 「￥1,234」 | `Intl.NumberFormat`（今のまま） |
| 金額（原価、1/100 円） | 「¥83.33」・「¥1,000」 | `formatCostYen`（[共通規則](10-common-rules.md) SPEC-UNIT-D5。半角の `¥`） |
| 画面名「商品と在庫」と CMD `get_stock_detail` / `list_low_stock` の命名差異 | 画面表示「商品と在庫」（D-138。今の build は「在庫照会」） | CMD は実態（詳細取得 / 在庫少一覧）に即した命名（memory `feedback-naming-must-match-reality.md`） |

### 58.13 非目的（IO/BIZ 同型表）

| やらないこと | 理由 | 責務を持つモジュール |
|---|---|---|
| 専用スキャンボタン実装 | Phase 2 では専用スキャンボタンは実装しない。バーコードスキャナは HID キーボード入力として検索欄に入る前提で、検索欄 focus + Enter 検索まで対応する。専用スキャン UX / 連続スキャン検知は Phase 3 UI-01a/UI-02 の HW 連携時に再設計する | Phase 3 UI-01a / UI-02（HW 連携） |
| 状態チップ件数バッジ | Phase 2 では状態チップに件数バッジを表示しない。理由: 件数の意味が検索条件・部門フィルタ・在庫少閾値 contract と結合し、追加 query / count contract が必要になるため。在庫切れ・在庫少件数の常時把握はホームサマリ（UI-00）で提供し、UI-06a ではチップをフィルタ操作に限定する | Phase 4 UI-06c または count API 設計時に再評価（UI-06b は独立画面ではないため対象外、D-047） |
| 在庫変動履歴の本体実装 | UI-06c で `/stock/$code/movements` として実装。UI-06a は詳細カードから active link で接続する | 画面本体の正典は [66-ui-stock-movements.md](66-ui-stock-movements.md) |
| `list_movements` specta 化 | PR #112 で generated `commands.listMovements` / `MovementRecord.source` contract を追加済み。UI-06c はその consumer として実装する | 画面本体の正典は [66-ui-stock-movements.md](66-ui-stock-movements.md) |
| 詳細カード CSV 出力 / 印刷 | SCREEN_DESIGN.md に言及なし | scope 外 |
| `stock_low_threshold` 設定変更 UI | 閾値設定は別画面 | UI-11a 閾値設定画面 |
| 「在庫切れ・在庫少」の廃番の表示トグル UI | 発注の準備に廃番は要らない（SP-302-04）。「すべて」の廃番の表示は 50 が持つ（D-138） | 50 UI-01a |

### 更新履歴

| 日付 | PR | 内容 |
|------|-----|------|
| 2026-10-24 | 画面ごとの要素の見直し design lane | 商品検索・一覧との統合（proposed・未実装、D-138）。冒頭の注記、§58.1 の deep-link の先、旧契約 I の撤去、契約 H の置換え（「すべて」でも在庫少を示す）、UI-06a-D8（チップ 2 つ、検索欄の列）、廃番除外の置換え、詳細の導線の `returnTo`・`productCode`、§58.4 の `selected` の上限 100（66 UI-06c-D9 と code に合わせた）。Plan Review round 1 の是正: 冒頭の「今の build」の範囲を項目単位に、§58.1 の CMD・route・主動線、§58.4 の `selected` ライフサイクルを統合の後の規則に（検索前ガードの撤去、部分一致の限定）、§58.7 の商品修正の `returnTo` を `buildProductListReturnTo` に、契約 H の集合の key の D-052 の invalidation、UI-06a-D7 の Rejected、§58.12 の画面名。 |
| 2026-10-07 | 単位の拡張 design lane | §58.6 に単位の拡張の後の formatter の契約を追加（proposed・未実装、D-113 / SPEC-UNIT-D3・D9・D11）。 |
| 2026-09-16 | ㉖ | UI-06a-D7 追加: `StockDetailContent` の「在庫変動履歴」link が現在の `/stock` URL を `returnTo` として送る（監査 NAV-1、66 UI-06c-D9 と対） |
| 2026-09-15 | 表示小修正 batch 2 | 在庫照会の副題、絞り込み後の「全 N 件」、在庫状態 Badge「在庫あり」を反映。 |
| 2026-08-26 | stale 実装状況表記一括是正 | `StockDetailContent` の「商品修正」「入庫記録」を既存画面への active link に変更し、遷移先と `returnTo` 非付与の契約を現況化 |
| 2026-05-20 | #67 | 新規作成（UI-06a 在庫照会、REQ-301/302 統合 1 画面、2 useQuery 部分障害許容 + URL state 4 key + 派生 4 純関数 + StockInquiryListResult 正規化型 + 色分け契約 H + 検索駆動表示契約 I + collapsible/toggle/toggle-group 新規 add + CSV 取込み invalidation / Plan rally 6 round converged） |
| 2026-05-20 | #67 | Codex Round 1 反映: P2-1/P2-2 = StockDetailCard を一覧テーブル下部固定表示に変更（list query 分岐の外に独立描画、list 失敗時も詳細を表示 = 部分障害許容と整合）。P3-1 = q/dept 変更時も `selected` を clear（§58.4、新 list の非同期含有判定を避け race 回避）。RTL に list 失敗 + detail 成功ケース追加（§58.9） |
| 2026-05-21 | #67 | 初 Windows native L3 デモ起因 F1/F2 修正: F1 詳細を最下部固定カード → 選択行直下 colSpan インライン展開（`StockDetailContent` 抽出、collapsible 不使用化）+ list 失敗時フォールバックカード + selected 不在時 clear（§58.4/§58.7/§58.8、Codex CLI Round 3-4）。F2 状態チップ選択を solid stone-700 + 太字でコントラスト強化（§58.7）。`ProductListTable.test` 新規（C-P2-3）。collapsible.tsx は未使用化（primitive 残置） |
| 2026-05-21 | #67 | Codex CLI 実装レビュー Round 1 P2 対応: P2-1 展開行 td に `whitespace-normal align-top`（table primitive 既定 `whitespace-nowrap` を打ち消し、長い商品名 / CTA 群の折り返し）。P2-2 isAllEmpty + selected URL での detail 空振り防止 = detail query `enabled` に `!isAllEmpty` guard + clear useEffect に isAllEmpty 分岐（§58.4 / §58.5）。test 2 件追加（isAllEmpty clear + detail 非発火 / 展開行 whitespace-normal guard） |
| 2026-06-07 | #74 | 高視認性 follow-up: 状態列を追加し、在庫状態を `Badge + lucide icon + 日本語ラベル` で表示する契約へ更新。在庫数の赤 / amber は補助シグナルに格下げし、`StatusChips` active tone を中庸 stone tone に調整。`ProductListTable` 展開行は `colSpan=6` に更新 |
| 2026-06-07 | display-scale follow-up | H-6 feedback 対応として `ProductListTable` の商品コードセルと `StockDetailContent` の詳細 header 商品コードを `font-mono text-sm font-medium` に更新し、`ProductListTable.test.tsx` で `text-xs` 回帰を防止 |
| 2026-06-08 | display-scale follow-up | Windows native L3 feedback 対応として、個別部門選択中も DepartmentFilter が他部門候補を維持するよう `useStockInquiry` に dept 未指定の候補用 query を追加し、`useStockInquiry.test.tsx` で回帰を防止 |
| 2026-06-08 | selection-tone follow-up | `StatusChips` active tone を shared selection-tone 定数参照に移し、Sidebar / TabsHeader と同じ stone selection 言語へ同期 |
| 2026-06-27 | UI-06c | `StockDetailContent` の「在庫変動履歴」を disabled placeholder から `/stock/$code/movements` active link へ変更。UI-06c 画面本体と `listMovements` consumer contract は [66-ui-stock-movements.md](66-ui-stock-movements.md) に分離 |
| 2026-07-16 | sidebar pending links follow-up | サイドバー「在庫少一覧」（UI-06b）の独立画面 `/stock/low` 予約を廃止し、本画面 `status=low_stock` フィルタへの deep-link に統合（D-047）。既存フィルタ contract（§58.4/§58.10）・useStockInquiry 実装は無変更 |
| 2026-07-29 | wave 4 plan-first | UI-STATE-D2に従い、status tupleと`LOW_STOCK_FILTER`をfeature-local SSOTとしてroute / navigationへ供給する契約を追加。URL値・fallback・表示は不変 |
| 2026-08-03 | ui-polish-batch-b（本 PR） | UI-06a-D1 追加: `page` search param + 既存 canonical 旧 `ProductPagination`（現 `Pagination`、02 ⑩）を結線し、「すべて」全件へページ送りで到達できるようにする（対象は `status === "all"` のみ）。`TruncatedResultsAlert` component と `truncated` flag を撤去（§58.2/§58.3/§58.5/§58.7/§58.10）。§58.7 DepartmentFilter を PR-B で統合済みの共通 `patterns/DepartmentFilter` 使用へ表記更新。§58.13 の pagination UI 行（本 change で実装）と DepartmentFilter 共通化行（PR-B で完了済み）を削除。filter-empty 0 件時の「絞り込みを解除」reset action（catalog ⑥）を追加（§58.7/§58.8） |
| 2026-08-03 | ui-polish-batch-b round 1 是正（本 PR） | UI-06a-D2 追加: 部門候補 query を `listDepartments()` master 全件へ切替（DSR-10、round 1 P1-3）、`departmentOptionsQuery` を dept/status 非依存の単一 query へ再設計（§58.5/§58.7/§58.10）。UI-06a-D3 追加: 範囲外 page（`items` 空 + `total_count > 0` + `page > 1`）に専用回復導線「先頭ページに戻る」を新設し、filter-empty reset action より優先判定（74 UI-11c-D8 同型、round 1 P1-2、§58.4/§58.7/§58.8/§58.9/§58.10）。EmptyState 疑似コードの `message=` prop を実契約 `title=` へ是正（round 1 P2-5、§58.7/§58.8） |
| 2026-08-03 | ui-polish-batch-b round 2 是正（本 PR） | round 2 P1-1 対応: `useStockInquiry` の return に `departmentOptionsQuery` / `departmentOptions`（`Department[] → DepartmentOption[]` 変換済み）を追加し、`StockInquiryPage` の `DepartmentFilter` を canonical props（`options` / `selected` / `onChange` / `disabled`、59 §59.1）へ結線（§58.3/§58.5/§58.7）。候補取得失敗時は `departmentOptionsQuery.isError` で「部門候補の取得に失敗しました」を listQuery と独立に表示する挙動を明記（§58.5/§58.7）。`queryKeys.stockInquiry.departmentOptions()` の無引数化（現状 `(status, q)` 引数付き）を implementation target として明示、対象は src/lib/query-keys.ts / useStockInquiry.ts / useStockInquiry.test.tsx（§58.5）。§58.9 SPEC-UIBB-9 に status 変更後の候補不変 assert / 同一 QueryClient での `listDepartments` call count = 1 / `departmentOptions()` 無引数・一定 key の unit test を追加 |
| 2026-08-03 | ui-polish-batch-b round 3 是正（本 PR） | P2-1（FilePicker ripple、対象は 60/63/SCREEN_DESIGN/FUNCTION_DESIGN/UI_TECH_STACK、本 file 対象外）と合わせて、本 file の要約層を詳細（§58.5/§58.3）と同期: 冒頭テンプレ判定表・§58.2 hook 行・§58.5 見出しを「2 useQuery」→「3 useQuery」、URL state を「4 key」→「5 key（q/dept/status/page/selected）」、§58.3 データフロー返却値を実疑似コードと同じ `{ listQuery, detailQuery, departmentOptionsQuery, departmentOptions, isAllEmpty }` へ修正（round 3 P2-2）。§58.9 に SPEC-UIBB-9 の loading（候補 pending 中は DepartmentFilter trigger disabled）/ error（`listDepartments` reject + list 成功で alert と一覧が同時表示、QueryClient retry 無効化）2 test を追加（round 3 P2-3） |
| 2026-09-06 | ⑨ 在庫少一覧の取引先列 + 棚卸し廃番 badge + R2-3（本 PR） | UI-06a-D4 追加: `ProductListTable` に取引先列（両 view 共通、null は `—`）を追加し、`filterLowStockList` を `filterAndSortLowStockList` へリネームして取引先名昇順（null 最後）→ 在庫数昇順 → 商品名昇順のクライアント側ソートを追加（§58.6/§58.7/§58.9/§58.10）。展開行 `colSpan` を `6` → `7` へ更新。UI-06a-D5 追加（R2-3）: 選択中行の再クリックで `onSelect(null)` を呼び展開が閉じるトグルクローズと、自動展開を検索条件（status/q/dept/page）単位で 1 度に制限する `useRef` ガードを追加（§58.4/§58.7/§58.9/§58.10） |
