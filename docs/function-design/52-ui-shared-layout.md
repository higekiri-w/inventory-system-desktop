> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（タスク仕様）、[ui-12-design-agreement.md](../archive/plans/2026-04-21-ui-12-design-agreement.md)（設計合意書、本書の判断根拠）

## 52. UI-12: 共通レイアウト

### 本書のテンプレ判定（UI 層関数設計書の 2 段階）

UI 層の関数設計書は業務ロジック有無で 2 段階のテンプレを使い分ける:

| 種別 | 判定基準 | 該当 UI | テンプレ |
|---|---|---|---|
| 業務ロジックあり | CMD 呼び出し / 入力バリデーション / 画面内部 state 駆動のフロー分岐 のいずれかを持つ | UI-00, UI-01a/b/c, UI-02〜10, UI-13 | 共通 6 項目（コンポーネント構成 / React State / CMD 呼び出し / 利用者操作フロー / エラー表示 / ローディング表示）+ 必要時バリデーション・処理ステップ |
| 業務ロジックなし（純構造） | 上記いずれも持たない（**ルーター主導の副作用は除外**） | UI-12 共通レイアウト | 主要 3 項目（コンポーネント構成 / React State / 備考）+ 特殊振る舞いだけ業務ロジックあり形式で独立節を切る |

**UI-12 の判定根拠**:

- CMD 呼び出し: なし
- 入力バリデーション: なし（入力受付なし）
- 画面内部 state 駆動のフロー分岐: なし（例外: 2026-06-07 follow-up で追加した表示サイズ state は UI 表示設定であり、業務データや CMD 呼び出しとは結合しない）
- ウィンドウタイトル動的更新機構（§52.5）はルーター主導の副作用 = 業務ロジック判定外。表示サイズ機構は WebView 表示設定の副作用として §52.6 備考に記録する

→ **業務ロジックなし** と判定。主要 3 項目で記述し、ウィンドウタイトル機構だけ §52.5 で業務ロジックあり形式の独立節として詳細化する。

---

### 52.1 コンポーネント構成

| ファイル | 責務 |
|---|---|
| `src/config/navigation.ts` | NavStatus / NavItem / NavArea 型定義 + navigation 定数（4 エリア × 22 項目）。アイコンは lucide-react を import |
| `src/components/layout/RootLayout.tsx` | 2 カラム grid + Outlet + Toaster + Devtools + ウィンドウタイトル更新 useEffect（§52.5）+ 自動バックアップ確認 hook の呼出し（UI-11b-D13。hook 本体と state は backup-restore feature が持つ）+ 印刷の案内の 1 行（[56](56-ui-daily-sales.md) UI-09a-D18。component は daily-sales feature が持つ） |
| `src/components/layout/Sidebar.tsx` | aside + SidebarHeader + min-h-0 ScrollArea + 4 エリアの map（SidebarArea を呼ぶ） + DisplayScaleControl |
| `src/components/layout/SidebarArea.tsx` | 1 エリア描画。h2 + アイコン + SidebarLink × N + Separator |
| `src/components/layout/SidebarLink.tsx` | active/pending status 分岐。`activeMatch` を持たない項目は `<Link>` + `activeOptions={{ exact: true, includeSearch: false }}`（search params 付き URL でも path 一致のみで active 判定、TanStack デフォルト `includeSearch:true` は search 完全一致を要求し active が外れる）+ shared stone selection tone。`activeMatch` を持つ項目（在庫照会・在庫少一覧、同じ `to="/stock"` を共有）は `includeSearch` 意味論に依存せず、router の現在 pathname + search state を明示比較して排他的に active 判定する（UI-12-D1、§52.6。route 文字列のコンポーネント内ハードコード禁止）。pending=`<span role="link" aria-disabled="true" tabIndex={-1}>` + sr-only "（未実装）"、cursor-not-allowed + opacity-60。focusable な link（active / inactive）には UI_TECH_STACK §5.4 系統①の focus ring `focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50` を付与する（pending は `tabIndex={-1}` で focus 対象外。2026-08-03、batch A packet 起源） |
| `src/components/layout/SidebarHeader.tsx` | 店名ロゴ + `<Link to="/">` + 末尾 Separator |
| `src/components/layout/DisplayScaleControl.tsx` | Sidebar footer の表示サイズ Select（標準 / 大きめ / 特大） |
| `src/components/layout/useDisplayScale.ts` | `localStorage` token 読み書き + Tauri WebView zoom 適用 |
| `src/lib/display-scale.ts` | `DisplayScaleValue` / storage key / zoom factor の SSOT |

**接続点**:

- `src/routes/__root.tsx` で `<RootLayout>` を mount。既存の暫定レイアウトを差し替え
- `notFoundComponent` から `min-h-screen` を削除（RootLayout の `h-screen` 枠内で 404 描画、サイドバーから戻れる 404 = Nielsen #3 整合）

---

### 52.2 React State

UI-12 は業務データの内部 state を持たない（純構造コンポーネント）。

唯一の例外として、ウィンドウタイトル機構（§52.5）が `useRouterState({ select: (s) => s.location.pathname })` で取得する pathname を派生値として読むが、これは TanStack Router 由来の参照値であり UI-12 内部 state ではない（Phase 2 で route `head()` 機能を導入する際は `useMatches()` 経由の最深 `match.head?.title` 参照に切り替える、詳細は §52.5）。

SidebarLink の active 判定も TanStack Router の `<Link activeProps>` で表現する（router 内部 state を読むのみ、UI-12 で state 管理しない）。

同じく例外として、RootLayout は自動バックアップの確認 hook `useAutoBackupCheck`（UI-11b-D13）を 1 回呼ぶ。interval・実行中の guard・連続失敗・復元の間の停止の state は hook と backup-restore feature が持ち、UI-12 は呼出しの置き場所だけを提供する。

2026-06-07 follow-up で追加した表示サイズ機構だけは UI 表示設定として `DisplayScaleValue = "standard" | "large" | "extra_large"` を React state で保持する。初期値は `localStorage["inventory.displayScale.v1"]` から読み、未知値は `"standard"` に fallback する。state 変更時に同 key へ保存し、`getCurrentWebview().setZoom(1 | 1.15 | 1.3)` で WebView zoom に反映する。Tauri 呼び出し失敗は `console.warn` のみで握り、画面描画は継続する。

---

### 52.3 ルーティング定義

全画面対応表（22 ナビ表示 + 2 ナビ非表示。UI-06b は UI-06a と `/stock` を共用するため route は 23）。この数と共用の関係は今の build の値で、D-138 の後（proposed・未実装）は「商品検索・一覧」のナビ表示が外れて 21 になり、UI-01a・UI-06a・UI-06b が `/products` を共用し、`/stock` の index は `/products` への寄せになる（UI-12-D1、[50](50-ui-product-list.md) UI-01a-D14・D19）。設計合意書 §2.1 を転記（D-047 反映済み）。各記録種別の記録詳細 route（`/inventory/receiving/records/$recordId` 等、入出庫履歴 6 種、いずれもナビ非表示）はこの表の集計対象外とし、[65-inventory-record-traceability.md](65-inventory-record-traceability.md) §65.3 を正本とする。

| UI-ID | 画面名 | URL パス | route ファイル | サイドバー4エリア | ナビ表示 | 備考 |
|---|---|---|---|---|---|---|
| UI-00 | ホーム | `/` | `src/routes/index.tsx` | 毎日の業務 | ○ | 現 demo (search_products)、Phase 2 (8-1) で置換 |
| UI-07 | 売上データ取込み | `/csv-import` | `src/routes/csv-import.tsx`（layout）+ `src/routes/csv-import/index.tsx`（index） | 毎日の業務 | ○ | サイドバー配置は日常業務の心理モデルを優先。nav 表示名は D-025 で「売上データ取込み」に変更 |
| UI-09a | 日次売上 | `/reports/daily` | `src/routes/reports/daily.tsx` | 毎日の業務 | ○ | 日次/月次は別 route（合意書 §7.3） |
| UI-06a | 在庫照会 | `/stock` | `src/routes/stock/index.tsx` | 毎日の業務 | ○ | REQ-301/302/303 統合画面。D-138 で UI-01a と 1 つの画面「商品と在庫」（`/products`、毎日の業務のこの位置）にし、`/stock` は `/products` へ寄せる（[50](50-ui-product-list.md) UI-01a-D14・D19、proposed・未実装） |
| UI-09b | 月次売上 | `/reports/monthly` | `src/routes/reports/monthly.tsx` | 毎日の業務 | ○ | |
| UI-01a | 商品検索・一覧 | `/products` | `src/routes/products/index.tsx` | 商品管理 | ○ | 商品管理の起点。D-138 で UI-06a と統合した「商品と在庫」の route になり、サイドバーの項目は毎日の業務の「商品と在庫」の 1 つ（商品管理のエリアから外す。proposed・未実装） |
| UI-01b (新規) | 商品登録 | `/products/new` | `src/routes/products/new.tsx` | 商品管理 | ○ | |
| UI-01b (編集) | 商品修正 | `/products/$code/edit` | `src/routes/products/$code.edit.tsx` | （ナビ非表示） | — | D-138 の後は商品と在庫の行の詳細の「商品修正」から遷移（今の build は一覧の「修正」or 在庫照会詳細から） |
| UI-01c | 一括インポート | `/products/import` | `src/routes/products/import.tsx` | 商品管理 | ○ | |
| UI-08 | PLU書出し | `/products/plu-export` | `src/routes/products/plu-export.tsx` | 商品管理 | ○ | 商品管理 route とサイドバー配置を一致させる |
| UI-14 | 一括価格改定 | `/products/price-revision` | `src/routes/products/price-revision.tsx` | 商品管理 | ○ | REQ-105/106。値上げリストを行単位で商品マスタへ反映 |
| UI-02 | 入庫記録 | `/inventory/receiving` | `src/routes/inventory/receiving.tsx` | 入出庫 | ○ | |
| UI-03 | 返品・交換 | `/inventory/return` | `src/routes/inventory/return.tsx` | 入出庫 | ○ | |
| UI-04 | 手動販売出庫 | `/inventory/manual-sale` | `src/routes/inventory/manual-sale.tsx` | 入出庫 | ○ | |
| UI-05 | 廃棄・破損 | `/inventory/disposal` | `src/routes/inventory/disposal.tsx` | 入出庫 | ○ | |
| UI-16 | 詰め替え・切り分け | `/inventory/transfer` | `src/routes/inventory/transfer.tsx`（詳細: `/inventory/transfer/records/$recordId`、UI-16b） | 入出庫 | ○ | 廃棄・破損の次。D-143（[79](79-ui-stock-transfer.md)） |
| UI-02b〜05b | 入出庫履歴 | `/inventory/records` | `src/routes/inventory/records.tsx` | 入出庫 | ○ | 入庫/返品・交換/手動販売/廃棄・破損/CSV取込み/棚卸しの追跡入口。記録詳細 route は[65-inventory-record-traceability.md](65-inventory-record-traceability.md)を正本とする |
| UI-06b | 在庫少一覧 | `/stock`（deep-link、専用 route なし） | `src/routes/stock/index.tsx`（UI-06a と共用） | 入出庫 | ○ | 独立画面は廃止し、UI-06a `status=low_stock` フィルタへの deep-link に統合（D-047、UI-12-D1）。D-138 で項目名を「在庫切れ・在庫少」にし、先を `/products?status=low_stock` にする（proposed・未実装） |
| UI-10 | 棚卸し | `/stocktake` | `src/routes/stocktake/index.tsx`（layout: `src/routes/stocktake.tsx`、詳細: `src/routes/stocktake.records.$stocktakeId.tsx`） | 入出庫 | ○ | 入出庫エリア末尾配置（年次作業、合意書 §7.4）。`/stocktake/records/$stocktakeId` はナビ非表示の記録詳細 |
| UI-06c | 在庫変動履歴 | `/stock/$code/movements` | `src/routes/stock/$code.movements.tsx` | （ナビ非表示） | — | $code 必須、商品と在庫の行の詳細と記録の詳細から遷移（D-138。今の build は在庫照会の詳細） |
| UI-11b | バックアップ・復元 | `/settings/backup` | `src/routes/settings/backup.tsx` | システム管理 | ○ | |
| UI-11c | 操作ログ | `/settings/logs` | `src/routes/settings/logs.tsx` | システム管理 | ○ | |
| UI-11a | 在庫少の基準 | `/settings/thresholds` | `src/routes/settings/thresholds.tsx` | システム管理 | ○ | nav 表示名は[69-ui-threshold-settings.md](69-ui-threshold-settings.md) UI-11a-D6（2026-07-06）で「在庫少の基準」に変更、「閾値」は画面に出さない |
| UI-13 | 整合性検証 | `/settings/integrity` | `src/routes/settings/integrity.tsx` | システム管理 | ○ | BIZ-07 連携、システム管理側に配置（合意書 §7.5） |
| UI-15 | 取引先管理 | `/settings/suppliers` | `src/routes/settings/suppliers.tsx` | システム管理 | ○ | REQ-106/107。追加・改名・重複統合を扱う |

**ナビに出さない route（動線は親画面から）**:

- `/products/$code/edit` ← `/products`（D-138 の後は行の詳細の「商品修正」。今の build は一覧の「修正」or `/stock` の詳細カード）から
- `/stock/$code/movements` ← `/products`（D-138 の後）の行の詳細。今の build は `/stock` 詳細カードから

**URL 設計の根拠**: 機能ドメインベース階層（`/products/*`、`/inventory/*`、`/reports/*`、`/stock/*`、`/csv-import/*`、`/settings/*`）。4 エリア分類（使用頻度）と機能ドメインが 1:1 でないため、**URL = 機能ドメイン** + **サイドバー = 使用頻度 4 エリア** で分離している。詳細は設計合意書 §2.3。

**Phase 1 (UI-12) で作るのは `__root.tsx` の差し替えのみ**。上記 route ファイルはすべて Phase 2 以降で各画面着手時に追加する。

---

### 52.4 navigation 定数と型定義

#### 型定義

```ts
import type { LucideIcon } from "lucide-react";

export type NavStatus = "active" | "pending";

export type NavItem = {
  id: string;            // "ui-01a" 等
  label: string;         // サイドバー表示用
  title: string;         // ウィンドウタイトル用（§52.5）。通常 label と同一、将来分離可
  to: string | null;     // Phase 1 は `/` のみ実 path、他 18 項目は null
  search?: Record<string, string>; // 同一 to を複数 nav 項目が共有する場合の deep-link search params（UI-12-D1、2026-07-16 追加。例: 在庫切れ・在庫少〈D-138。今の build は在庫少一覧〉の { status: "low_stock" }）
  activeMatch?: { searchKey: string; is?: string; isNot?: string }; // 排他 active 判定用 predicate（UI-12-D1、2026-07-16 追加）。is=現在値との等値、isNot=非等値（現在値 undefined も isNot 成立）。持たない項目は既存 <Link activeOptions> 経路を無変更維持
  description?: string; // ホーム入口 card の 1 行説明（UI-00、2026-09-16 追加）。sidebar は読まない
  icon: LucideIcon;
  status: NavStatus;
};

export type NavArea = {
  id: "daily" | "products" | "inventory" | "system";
  label: string;
  icon: LucideIcon;
  items: readonly NavItem[];
};

export const navigation: readonly NavArea[] = [...] as const;
```

#### 4 エリア × 22 項目

下の表は今の build の項目。D-138 の後（proposed・未実装）は、毎日の業務の「在庫照会」が「商品と在庫」に、入出庫の「在庫少一覧」が「在庫切れ・在庫少」になり、商品管理から「商品検索・一覧」が外れて 4 項目（計 21 項目）になる（UI-12-D1、[50](50-ui-product-list.md) UI-01a-D14）。

| エリア | エリアアイコン | 項目数 | 項目（順序固定） |
|---|---|---|---|
| 毎日の業務 | `Sun` | 5 | ホーム / 売上データ取込み / 日次売上 / 在庫照会 / 月次売上 |
| 商品管理 | `Package` | 5 | 商品検索・一覧 / 商品登録 / 一括インポート / PLU書出し / 一括価格改定 |
| 入出庫 | `ArrowLeftRight` | 7 | 入庫記録 / 返品・交換 / 手動販売出庫 / 廃棄・破損 / 入出庫履歴 / 在庫少一覧 / **棚卸し**（末尾、年次作業） |
| システム管理 | `Wrench` | 5 | バックアップ・復元 / 操作ログ / 在庫少の基準 / 整合性検証 / 取引先管理 |

#### 各項目アイコン（lucide-react ^1.8.0）

項目名は今の build の名（D-138 の後の名は上の「4 エリア × 22 項目」の表の前の注記）。

| 項目 | アイコン | 項目 | アイコン |
|---|---|---|---|
| ホーム | `Home` | 入庫記録 | `PackagePlus` |
| 売上データ取込み | `FileUp` | 返品・交換 | `RotateCcw` |
| 日次売上 | `BarChart3` | 手動販売出庫 | `Hand` |
| 在庫照会 | `Search` | 廃棄・破損 | `Trash2` |
| 月次売上 | `BarChartBig` | 入出庫履歴 | `ScrollText` |
| 商品検索・一覧 | `PackageSearch` | 在庫少一覧 | `AlertTriangle` |
| 商品登録 | `PackagePlus` | 棚卸し | `ClipboardList` |
| 一括インポート | `FileSpreadsheet` | バックアップ・復元 | `DatabaseBackup` |
| PLU書出し | `FileDown` | 操作ログ | `ScrollText` |
| 一括価格改定 | `CircleDollarSign` | 在庫少の基準 | `SlidersHorizontal` |
| 取引先管理 | `Building2` | 整合性検証 | `ShieldCheck` |

**アイコンスタイル**: `className="size-4 stroke-[1.5]"`（16px、線細め）。active 時は stone-700、inactive 時は stone-500（[../design-system/00-foundations.md](../design-system/00-foundations.md)「4色エリアモデルの扱い」準拠）。

**Phase 1 の status 配分**（2026-04-21 時点の初期状態、以降のフェーズで順次 active 化）:

- `to: "/"` + `status: "active"` = ホームのみ 1 項目
- `to: null` + `status: "pending"` = 残り 18 項目（route 未実装、Phase 2 以降で順次 active 化）

**2026-08-25 時点**: 現行実装のサイドバーは 22 項目で pending は 0 件。UI-15 は `navigation.ts` のシステム管理末尾へ active entry として登録済み。UI-06b は独立画面ではなく UI-06a への deep-link（D-047、詳細は §52.6）。

---

### 52.5 ウィンドウタイトル動的更新機構（特殊振る舞い、業務ロジックあり形式の独立節）

UI-12 内で唯一「処理 + エラーハンドリング + 書式規約」を持つ機構。業務ロジックあり形式で詳細化する。

**関数要求**: ルート遷移に応じて `document.title` を `在庫管理システム - <画面名>` 形式で更新し、OS タスクバー / Alt+Tab で画面把握性を高める

**React State**:

- `useRouterState({ select: (s) => s.location.pathname })` の戻り値（TanStack Router 提供、現在 pathname 文字列）
- 派生値: navigation 配列から `to === pathname` の項目を引いた `title`、該当なし or ホーム (`/`) は空文字

**処理ステップ**:

1. RootLayout 内で `const pathname = useRouterState({ select: (s) => s.location.pathname })` で現在 pathname を取得（`useMatches()` を直接使うと TanStack Router の generic 推論が壊れて型 unsafe になるため、`select` プロジェクションで pathname だけ取り出す）
2. `deriveTitle(pathname)` で navigation 配列から title 抽出（純関数、別途 export して unit test 可能に）
3. `useEffect(() => { document.title = formatted }, [formatted])` で副作用反映
4. ホーム (`pathname === "/"`) は "在庫管理システム" 単独表記、それ以外は "在庫管理システム - <title>"
5. 動的 title（`$code` 等）は Phase 2 以降の各 route `head()` で loader data 由来として実装。その段階で `useRouterState` から `useMatches()` + 最深 `match.head?.title` 参照に切り替える

**エラーハンドリング**:

- `useRouterState({ select })` が初期 mount 前に呼ばれるケース: TanStack Router 側で初期 location が `/` を返すため null safety 不要
- Tauri ネイティブ API 併用 (2026-04-21 実機確認で確定):
  - **WSL2 (Ubuntu) WebKitGTK では `document.title` が OS ウィンドウタイトルに rebind されない**ことを実機確認 (Phase 1 GUI 疎通時、タスクバーに `inventory-system-tauri-scaffold` の Tauri 既定タイトルが残っていた)
  - 対応: `@tauri-apps/api/window` の `getCurrentWindow().setTitle(formatted)` を useEffect 内で `document.title` 更新と併用する
  - **Tauri 2 capability 必須**: `src-tauri/capabilities/default.json` の `permissions` に `core:window:allow-set-title` を明示追加する。`core:default` には含まれず、未追加の場合 `setTitle()` 呼び出しは permission denied で失敗する
  - Tauri API 呼び出し失敗時: `.catch((e: unknown) => console.warn(...))` で握って `document.title` のみ更新を保証 (ブラウザ tab 表示には影響なし)
  - Phase 2 Windows native 移行後: Windows は `document.title` の OS ウィンドウタイトル反映が確実だが、Tauri `setTitle()` 併用は害がないため維持

**バリデーション**:

- title 文字列は trim 後の長さチェック（空文字なら "在庫管理システム" にフォールバック）
- 区切り文字: 半角ハイフン + 前後半角スペースで統一

**書式規約**:

- ホーム: `在庫管理システム`
- 通常画面: `在庫管理システム - <画面名>`
- 動的 title: `在庫管理システム - <画面名>: <パラメータ>`（例: `在庫管理システム - 商品修正: HZ-0047`）

**Phase 1 (UI-12) と Phase 2 の責務分担**:

- UI-12: タイトル反映機構の配線、`NavItem.title` フィールド追加、ホーム route フォールバック
- Phase 2 以降: 各 route の `head()` 宣言、loader data を使う動的 title 実装

---

### 52.6 備考

- **notFoundComponent の min-h-screen 削除**: `__root.tsx` の現行 `min-h-screen` を削除し、RootLayout の `h-screen` 枠内で 404 描画する。サイドバーから戻れる 404 になる（Nielsen ヒューリスティック #3）
- **SidebarLink の a11y**: pending リンクは `<span role="link" aria-disabled="true" tabIndex={-1}>` + `<span className="sr-only">（未実装）</span>`。スクリーンリーダ・Tab 巡回から除外。Phase 1 ではコード上で実装、可能なら NVDA / VoiceOver で目視確認
- **SidebarLink active 時の hover**: active 状態は shared stone selection tone (`bg-stone-300` + `border-stone-400` + `font-semibold`) で表現し、hover 時も同 tone を維持する。DSR-21 の現在地アクセント（`border-l-primary` 系）を重ね、active / inactive 双方の寸法が変わらないよう base 側に同幅の transparent 左 border を持たせる。inactive の hover 上書き (`hover:bg-stone-200/60`) で active 状態が灰色化する事故を防ぐ。共通の外枠寸法は `border border-transparent` を base に持たせる。base class と `activeProps.className` は TanStack が連結するだけで merge しないため、同一 CSS property（`border-l-*` の色）で衝突する class を base に置かない（2026-09-02 L3 で発見、`border-l-transparent` の共存で Primary バーが不可視になった）
- **SidebarLink の TanStack Router props 分離**: `<Link>` の `activeProps` は base `className` に**追加**されるだけで上書きしないため、`className` に `hover:bg-stone-200/60` を含めると active 時も stone hover が残って灰色化する。これを防ぐため `className` には共通 `baseClass` のみ、active/inactive それぞれの背景色 + hover は `activeProps.className` / `inactiveProps.className` に完全分離する (2026-04-21 実機確認で確定)
- **デスクトップ前提**: 固定幅 240px サイドバー（レスポンシブ非対象）、hover 状態を主要導線に使って良い（タッチ非対応）、URL は内部識別子（ブックマーク非想定）。詳細は `memory/desktop-app-ui-constraints.md` 参照
- **invoke 境界**: Phase 2 closeout で `invoke-fallback.ts` は撤去済み。`components/layout/**` と `config/**` は引き続き invoke 層に触らず、画面側は `commands.*` + `unwrapResult` を使う
- **Sonner Toaster の theme**: `next-themes` Provider 未マウントでも default `"system"` で動作。`theme` prop を渡さない運用を維持（`src/components/ui/sonner.tsx` 既存定義）
- **デザイン哲学**: stone 系パレット + アイコン + 区切り線でグループ表現（色分け廃止）。出典は [../design-system/00-foundations.md](../design-system/00-foundations.md)「4色エリアモデルの扱い」（ドクトリン文書）
- **表示サイズ option**: H-6 の商品コード readability follow-up として Sidebar footer に `表示サイズ` Select を追加する。選択肢は `標準` / `大きめ` / `特大`、保存は frontend-only `localStorage`、WebView 反映は `@tauri-apps/api/webview` の `setZoom`。Tauri capability は `core:webview:allow-set-webview-zoom`。UI-11a/b/c の設定画面や `app_settings` には本 PR では接続しない
- **表示サイズ control の到達性**: WebView zoom 後に Sidebar 全体が実質大きくなるため、`Sidebar` root と navigation `ScrollArea` は `min-h-0` を持ち、navigation 側を縮小・スクロール可能にする。`DisplayScaleControl` は `shrink-0` として Sidebar footer に残し、`大きめ` / `特大` にした後でも表示サイズを戻せることを L3 で確認する
- **UI-12-D1（2026-07-16、同じ route を指す 2 項目の排他 active 判定の機構規定）**: `/stock` を指す nav 項目が「在庫照会」（ui-06a）と「在庫少一覧」（ui-06b、D-047 で `/stock` への deep-link に統合）の 2 つになったため、active 表示は排他にする。D-138 の後は 2 つの項目が「商品と在庫」（ui-06a、`to: "/products"`）と「在庫切れ・在庫少」（ui-06b、`to: "/products"` + `search: { status: "low_stock" }`）になり、下の機構をそのまま使う（`item.to` が `/products` に変わるだけ。proposed・未実装）。下の bullet の項目名（在庫照会・在庫少一覧）と `/stock` の例は今の build の値。
  - **型**: `NavItem` に optional `activeMatch?: { searchKey: string; is?: string; isNot?: string }` を追加する。
  - **宣言**（`navigation.ts` 側）: ui-06b（在庫少一覧）は `activeMatch: { searchKey: "status", is: "low_stock" }`、ui-06a（在庫照会）は `activeMatch: { searchKey: "status", isNot: "low_stock" }` を宣言する。
  - **判定**（`SidebarLink.tsx` 側）: `activeMatch` を持つ項目のみ、`useRouterState` selector（§52.5 ウィンドウタイトル機構の pathname 取得と同じ selector パターン）で現在 pathname + search を読み、「`pathname === item.to` かつ predicate 成立」で active と判定する。predicate は `is` 指定時は `currentSearch[searchKey] === is`（等値）、`isNot` 指定時は `currentSearch[searchKey] !== isNot`（非等値。`currentSearch[searchKey]` が `undefined`（= URL に該当 key がない）の場合も `isNot` は成立扱い、`undefined !== "low_stock"` は真のため「在庫照会」は `status` 未指定時も active になる）。`activeMatch` を持たない残りの項目は既存 `<Link activeOptions={{ exact: true, includeSearch: false }}>` 経路を無変更で維持する。
  - **contract**: `SidebarLink` コンポーネント内に特定 route 文字列（`"/stock"` 等）をハードコードしない。比較対象の pathname は常に `item.to` から取得する。これにより将来同種の nav ペア（同一 `to` を search で書き分ける組）が増えても component 側の変更は不要になる。
  - **有限値の所有**: `status=low_stock` は在庫照会 feature の URL search contract であり、`src/features/stock-inquiry/types.ts` の `LOW_STOCK_FILTER` を `navigation.ts` の `search.status` と `activeMatch.is/isNot` が共有する。layout 側に同じ literal の第2正本を置かない。
  - TanStack `<Link>` の `activeOptions.includeSearch` には依存しない — その意味論は search 完全一致要求（exact）であり、`/stock?status=low_stock&q=foo` のような追加 search key が付いた場合の挙動がバージョン依存で曖昧なため。
  - 理由・棄却代替案は decision-log D-047 参照

---

### 52.7 非目的

| やらないこと | 理由 | 責務を持つモジュール |
|---|---|---|
| 各画面コンポーネントの実装 | UI-12 は枠のみ提供 | UI-00〜13 各タスク |
| ダークモード切替 UI | 現状判断で見送り | 将来判断 |
| 表示サイズの DB 永続化 / 設定画面統合 | UI-11a/b/c の既存契約外。本 PR は frontend-only の表示設定に留める | Phase 4 UI-11 系で再判断 |
| Storybook 配線 | 不採用で確定（D-068、2026-08-12） | 導入せずクローズ。判断記録は UI_TECH_STACK §7.1 / decision-log D-068 |
| Error Boundary 戦略 | UI_TECH_STACK §6.10 で規約化済み（2026-08-03、7-8a 消化） | §6.10 が正本。実装は route 基盤（`main.tsx` defaultErrorComponent + `__root.tsx` errorComponent） |
| エラー表示 / ローディング表示の規約化 | UI_TECH_STACK.md で別途規約化 | 7-8b 横断UI標準化 |
| screen_mockups.html の色分け箇所修正 | モックアップアーカイブ扱い | 触らない |

---

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-09-02 | wave 8 lane 1 是正 | base class の `border-l-transparent` が `activeProps.className` の `border-l-primary` と連結時に共存し Windows で Primary バーが不可視化する不具合を是正。base は `border-transparent`（左辺含む透明）+ `border-l-[3px]`（幅のみ）に留める契約へ更新 |
| 2026-09-02 | wave 8 lane 1 | SidebarLink の shared stone selection tone に DSR-21 の Primary 左端バーを重ね、base 側の同幅 transparent 左 border で active 切替時の layout shift を防ぐ契約を追加 |
| 2026-04-21 | 7-3 UI-12（本 PR） | 新規作成。設計合意書 [ui-12-design-agreement.md](../archive/plans/2026-04-21-ui-12-design-agreement.md) §2-§5 を関数設計書形式（業務ロジックなし版、主要 3 項目 + ウィンドウタイトル機構の独立節）で転記 + 実装と同 PR で統合 |
| 2026-06-07 | display-scale follow-up | Sidebar footer に表示サイズ Select を追加。`localStorage` token + Tauri WebView zoom (`core:webview:allow-set-webview-zoom`) で全画面表示を 3 段階にする。拡大後も control に戻れるよう Sidebar navigation を `min-h-0` ScrollArea 化。DB/settings 画面統合は Phase 4 UI-11 系へ defer |
| 2026-06-08 | selection-tone follow-up | SidebarLink active tone を amber 系から shared stone selection tone へ統一。amber は在庫少などの業務セマンティック色に残し、navigation selection とは分離 |
| 2026-07-16 | sidebar pending links follow-up | サイドバー最後の pending 2 項目を解消。「商品登録」（UI-01b）を既存 `/products/new` route へ active化。「在庫少一覧」（UI-06b）の独立画面 `/stock/low` 予約を廃止し、UI-06a `/stock` の `status=low_stock` フィルタへの deep-link に統合（D-047）。`NavItem` に optional `search` field を追加し、`/stock` を指す 2 nav 項目の排他 active 判定を新設（UI-12-D1） |
| 2026-09-16 | ㉔ mockup-c 採用 | `NavItem.description` をホーム入口 card の 1 行説明として追加。sidebar は参照せず、既存の navigation 契約を維持 |
