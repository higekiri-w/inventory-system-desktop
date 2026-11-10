# Plan Packet: ㉑ フィルタ Label 上置き + 見出し 2 段の runtime 反映

2026-09-15 起票。起点は `0414392338b7e7816803f13e379e69f689e21ef2`（origin/main）。design ⑳（[archive packet](2026-09-10-filter-label-top-design.md)、PR #51 merge）が catalog ⑨ / ① / ⑤ と mockup-g で確定した規範を runtime へ反映する。Scope の正本は ⑳ packet の S6 申し送りで、本 packet は現行 main で file:line を再実測した（Sonnet Explore 2026-09-15、Coordinator が主要 site を直接読んで確認）。実装は別 run とし、Coordinator 裁定と独立 Plan Review 通過後に発注する。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: c5b1ea2b275d74791fa0d6c76e1f7be0a52f4a65
- Amendments: 69c6011cc410d14b16e2793c089f286cb960fd18, 25f705127a910a8346ae5d7a12179ece19504bec, 67be6db06eb6cb0a07e0a2083ad35f0570a74804, 7a3bfc1cf30c63f1c4c05e22057938c147985ca4, 2cf4691d416ae5139d0392feb0b9b4ae2ecd569a
- Coordinator: Fable 5.1
- Writer: Codex
- Plan Reviewer: Sonnet + Opus（独立 fresh context、並列。裁定は Coordinator が直列）
- Final Reviewer: Sonnet + Opus（独立 fresh context）+ Codex review 1 run
- Final Review Minimum: 2
- Human Gate: ready,merge,manual

manual = owner Windows native L3 の抜き取り（AC-L3-1〜4）。github mode の有効化根拠は [PR #59 の専用記録](https://github.com/kosei-w90607/inventory-system-desktop/pull/59#issuecomment-5664736990)。本 lane は github mode で closure record と manual（L3）record を初めて使う dogfood target（Plans.md 2026-09-14 申し送り）。Fable 指揮の分業 lane であり D-087 の一貫担当例外は適用しない。

遷移記録（append-only）:
- kickoff → spec-check → plan-draft → plan-gate（`a2e01334`）: Risk R3 を記録、Design Readiness が catalog ⑨ / ① / ⑤ + mockup-g を実装十分と引用（⑳ で design 完了済み、spec-check → plan-draft の許容 skip）、packet + Matrix を同 commit に置く。
- plan-gate round 1（Sonnet: P1 3 / P2 2、Opus: P2 6 / P3 7、2026-09-15）→ in-place 是正（`b771eb61`、Phase は plan-gate のまま）。設計判断の差し戻しなし。round 2 = closure（Sonnet 可 / Opus 新規 P2-A）→ 是正 `3ee233a8` + `41a9e7e7` → round 3 = closure（Opus 新規 P2-F）→ 天井到達の disposition `c5b1ea2b`（同型一括是正、Review Response 参照）。
- plan-gate → plan-approved → implementing（本 commit、state-only）: 独立 Plan Reviewer（Sonnet + Opus）の round 1 findings は全 closed、round 2 / 3 の新規 P2 は Coordinator の同型一括是正で closed、P1/P2 = 0。Plan Commit = `c5b1ea2b`（plan-first `a2e01334` を先頭とする是正済みの確定版）。実装は Codex 発注書 49 で本 commit を HEAD_SHA として開始する。

## Owner Effort Budget

- 介入回数上限: 4（L3 抜き取り 1 round + Ready + merge + 予備 1。視覚系 lane の実績〈⑭ L3 2 round / ⑲ 2 round〉を踏まえ既定 3 から +1）
- 実働時間上限: 40分
- relay 往復上限: 4（既定 2 から改訂。Gated Amendment 2 で 3〈owner L3 round 1 の是正、owner 承認 = 発注書 54 の起動〉、Gated Amendment 4 で 4〈fresh broad Opus の P2-1 / P3-1 是正、owner 承認 = 発注書 55 の起動〉。視覚系 lane の先例〈⑲ relay 3〉+ 1）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
operator workflow の UI 契約（一覧 / 記録画面の toolbar 配置、SegmentedControl の accessible name の出どころ、`PageHeader` の actions 持ち 15 page の配置）を変える。DB / Tauri command / DTO / route / search state は変更しないが、`PageHeader` と `DepartmentFilter` と `SegmentedControl` は共有 component で波及が page 横断（`PageHeader` 28 page、`DepartmentFilter` 5 site、SegmentedControl 3 site + tab 型 2 site 不変）になるため R2 に落とさない。

## Goal

Goal Invariant:

### 最小完了条件

- 一覧 / 記録画面の toolbar で、すべてのフィルタ入力（検索 / 部門 / 並び替え / 表示件数 / 廃番表示 / PLU表示 / 並び順 / 取引先 / 未入力のみ表示）の可視 label が入力の上に同じ見た目（`text-sm` weight 400 muted）で並び、Checkbox だけは行の縦中央に来る（catalog ⑨ 使用トークン、mockup-g state-1 提案 a / state-2 提案）。
- `PageHeader` の actions 持ち page と、右要素を持つ section 見出し 2 箇所（差異のある商品 / 棚卸し進捗）が「見出し行 + 説明行」の 2 段になり、説明がボタンの横で折り返さない（catalog ① 構造 block / セクション見出し variation、mockup-g state-3 / state-4）。
- section 見出し h2 の token が `text-xl font-semibold` に揃う（catalog ① variation）。

### 失敗定義

- accessible name が変わる、または消える（`getByRole("combobox", { name: "表示件数" })` 等の既存 query が解決しなくなる、SegmentedControl の group name が可視 label と一致しない）。
- URL / search state、フィルタの候補ソース（DSR-10）、検索の確定経路（Enter / debounce / IME）に差が出る。
- `PageHeader` の actions 無し page（13 page）の DOM が変わる。tab / mode 切替の SegmentedControl（`src/components/sales/TabsHeader.tsx` は class 定数 + `<Link>` で component 非使用、`monthly-sales/components/ModeTabs.tsx`、`plu-export/PluExportPage.tsx:695` の書出しモード）に可視 Label が付く。
- `src-tauri/**`、`docs/function-design/**`、`01-decision-rules.md` に diff が出る。

### 非目的

- 廃番表示 3 択 / PLU表示 5 択の Tabs / Select 化（DSR-02 drift。本 packet の D-RT4 で「据え置き + Backlog 保留」を裁定）。
- 商品追加検索（ProductAddSuggest wrapper）5 箇所の `space-y-2` → `grid gap-1`（toolbar 外、⑮ の wrapper。描画差は 4px で「壊さない優先」）。
- ホーム画面の h2（`text-lg font-medium`、action group 見出し）の token 変更（後続の「ホーム mockup-c」lane の footprint）。
- 04-backbone 原則 7 の drift（検索ボタン併記）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## 起票時実測（2026-09-15、origin/main `04143923`）

⑳ S6 の file:line を現行で再実測した結果。差分は「S6 との差」列。

| # | site | 現行 | S6 との差 |
|---|---|---|---|
| 1 | `src/components/patterns/DepartmentFilter.tsx:61-64` | `div.flex.items-center.gap-2` + `<label htmlFor={triggerId} id={labelId}>`、`labelId` は `:58,62` で未参照 | 一致 |
| 1a | `DepartmentFilter` 呼び出し | `StocktakePage.tsx:747` / `DailySalesPage.tsx:84` / `ProductListPage.tsx:119` / `StockInquiryPage.tsx:116` / `PriceRevisionFilters.tsx:92` の **5 site**（全部 `@/components/patterns/DepartmentFilter`） | S6「4 サイト」は日次売上を数え落とし。component 1 箇所の改修で 5 site が揃う |
| 2 | `src/features/products/ProductListPage.tsx:155` 2 段目 wrapper | `flex flex-wrap items-center gap-3`（1 段目 `:106` は既に `items-end gap-3`） | 一致 |
| 2a | 同 `:156-159` 並び替え / `:187-190` 表示件数 | `div.flex.items-center.gap-2` + raw `<label className="text-sm text-muted-foreground">` | 一致 |
| 2b | 同 SegmentedControl `:135-142`（`ariaLabel="廃番表示"`）/ `:143-150`（`"PLU表示"`）/ `:179-186`（`"並び順"`） | 可視 label の prop なし | 一致 |
| 3 | `src/features/stock-inquiry/StockInquiryPage.tsx:132-135` 表示件数 | `div.flex.items-center.gap-2` + raw label。wrapper `:103` は既に `items-end gap-3` | +3 行 |
| 4 | `src/features/products/components/PriceRevisionFilters.tsx:62-91` 取引先 trigger | `div.flex.items-center.gap-2` + `<label id="price-revision-supplier-label">` + `<Button aria-haspopup="dialog" aria-labelledby>`（⑲ GA2 の群化 comment `:60-61`）。wrapper `:51` は既に `items-end gap-3` | ⑲ 後の形。`grid gap-1` 化は未反映 |
| 4a | 同 `:102-111` 廃番を含む / `:143-155` 取引先未設定 Checkbox | `<label htmlFor><Checkbox/>文言</label>` の label 内包 | 2 件目の開始行は `:143`（S6 `:145`） |
| 4b | 同 `:112-119` 表示件数 | `div.flex.items-center.gap-2`（`:112`）+ raw label（`:113-`、`id="price-revision-per-page-label"` は repo 内で未参照） | wrapper は `:112` |
| 5 | `src/features/stocktake/StocktakePage.tsx:746` toolbar | `flex flex-wrap items-center gap-4 rounded-lg border bg-card p-4`、閉じは `:798` | 一致 |
| 5a | 同 `:756-770` Checkbox | `div.flex.items-center.gap-2` + `<Checkbox id>` + sibling `<Label htmlFor>` | 一致（唯一の sibling 型） |
| 5b | 同 `:771-774` 表示件数 | `div.flex.items-center.gap-2` + `<Label className="text-muted-foreground">`（`Label` component、既定 `font-medium`） | S6 未記載。D1 の `font-normal` 打ち消しが必要 |
| 5c | 同 `:381-410` `StocktakeProgressHeader` | `div.space-y-2` > `div.flex.flex-wrap.items-center.justify-between.gap-3` > (`div` > h2 `text-xl font-semibold` + p) + Badge、`<Progress>` は `:407` | 範囲 -4/+3 行 |
| 6 | `src/features/integrity-check/IntegrityCheckPage.tsx:232-260` 表示件数 | `div.flex.items-center.gap-2` + `<label id="integrity-check-per-page-label">`（id は未参照）+ Select | Select 閉じは `:260` |
| 6a | 同 `:324-333` 差異のある商品 | `section.space-y-4` > `div.flex.flex-wrap.items-end.justify-between.gap-3` > (`div` > h2 `text-xl font-semibold` + p) + Button | -1 行 |
| 7 | `src/components/ui/segmented-control.tsx:28-35` props / `:46` root | `ariaLabel` 必須、可視 label の prop なし。root は `div[role="group"][aria-label]`（labelable でない）。使用 site は `rg -n '<SegmentedControl' src --glob '!*.test.tsx'` = `ProductListPage.tsx:135,143,179`（toolbar）/ `ModeTabs.tsx:19`（mode）/ `PluExportPage.tsx:695`「書出しモード」（Card 内の mode 切替、Plan Review round 1 で検出）の 5 site。`TabsHeader.tsx` は class 定数 + `<Link>` で component 非使用 | S6 は PluExport を数えていない |
| 8 | `src/components/patterns/SearchBar.tsx:72`（commit）/ `:178`（live） | `<Label className="shrink-0 text-muted-foreground">` / `<Label className="text-muted-foreground">`、`font-normal` なし | 一致 |
| 9 | `src/components/patterns/PageHeader.tsx:20-28` JSDoc / `:30` inline / `:31-44` actions 分岐 | `header.flex.flex-wrap.items-start.justify-between.gap-3` > `div.min-w-0.flex-1.space-y-1`（h1 + subtitle + description）+ `div.shrink-0`。props は既に `description?` あり（`:15`） | 一致 |
| 10 | section h2 `text-lg` | `rg -n '<h2[^>]*text-lg' src --glob '!*.test.tsx' \| rg -v 'font-medium'` = 27 箇所（単一行 literal `<h2 className="text-lg font-semibold">` が 25 + `MonthlySalesPage.tsx:157` / `DailySalesPage.tsx:165` の `id` 付き 2）。`HomePage.tsx:90,95,100` は `text-lg font-medium` で除外 | S6「A/B 6 + 右要素なし 24」と規模一致（HomePage 3 を除く） |
| 11 | `docs/design-system/02-component-catalog.md:28` ① canonical | `PageHeader{title, subtitle?, actions?}` の 3 variant（`description?` 欠落） | 一致 |
| 11a | 同 `:640` ⑨ D2 block comment / ① 「runtime 反映は後続 lane」 | 「runtime 反映は後続 lane」の marker が ⑨ block comment と ① 段落末に残る | 本 lane で解消 |
| 12 | 既存 test | 構造変更で落ちる assert = `StocktakePage.test.tsx:1171`（`closest(".flex.flex-wrap.items-center.gap-4")`）/ `IntegrityCheckPage.test.tsx:549`（`closest(".flex.items-center.gap-2")`、bg-card 不在の round 2 回帰）/ `PriceRevisionFilters.test.tsx:68`（`label.closest(".flex.items-center.gap-2")`、⑲ GA2 群化回帰）/ `PageHeader.test.tsx:53-59,70-71,87-92`（header の flex class、`subtitle.parentElement` の `min-w-0 flex-1 space-y-1`、⑮ SC1）。`StocktakePage.test.tsx:1185,1261` の `el.className === "flex flex-wrap items-center gap-4"` は「枠なし旧 frame の不在」negative assert で、現行も変更後も一致要素が無く PASS（触らない）。repo に snapshot test は 0 件。各一覧 test は `getByRole("combobox", { name: "表示件数" })` で accessible name 依存 | Plan Review round 1 で訂正 |

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **D-RT1 label wrapper は raw `<label>` + `div.grid.gap-1` を canonical とし、`Label` component を使う既存 site（SearchBar 2 / Stocktake 表示件数 1）は component を残して `font-normal` を足す**（catalog ⑨ 使用トークンの両形を許す規範どおり。置換すると SearchBar の test / a11y 契約〈live 型は可視 label のみ〉に触るため最小差分を選ぶ）。
- **D-RT2 `SegmentedControl` の `showLabel?: boolean`**（Plan Review round 1 Opus P3-1 で `label?: string` から変更: catalog ⑨ は「`ariaLabel` の文言を上置き label として可視化」であり、別文字列を渡せる prop は同一性を型で保証せず `SegmentedControlOption.label` とも名前が衝突する）: `showLabel` のとき `div.grid.gap-1` で包み、`<span id={labelId} className="text-sm text-muted-foreground">{ariaLabel}</span>` を上置きし、`div[role="group"]` に `aria-labelledby={labelId}` を付け **`aria-label` は出力しない**（accessible name の出どころを 1 つにする。⑨ live 型の「aria-label を追加しない」と同じ理由）。未指定時は現行どおり `aria-label={ariaLabel}`。`ariaLabel` prop は必須のまま（catalog ⑤「`ariaLabel` は常に必須」）。toolbar 3 site は `showLabel` を付け、mode 切替 2 site（ModeTabs / PluExport 書出しモード）は付けない。`labelId` は `useId()`。
- **D-RT3 `PageHeader` actions 分岐は 2 段に改め、actions 無し分岐は触らない**: `<header className="space-y-1">` > `<div className="flex flex-wrap items-start justify-between gap-3">` > `<h1 className="min-w-0 flex-1 text-2xl font-semibold">` + `<div className="shrink-0">{actions}</div>`、その下に `subtitle !== undefined` / `description !== undefined` の `<p className="text-sm text-muted-foreground">` を全幅で置く（catalog ① 構造 block と literal 一致）。actions 無しの 13 page は DOM 不変（失敗定義）。JSDoc `:20-28` の (c) 行と「副題・説明は左グループ内で折り返す」、inline `:30` を 2 段の記述へ更新する。描画差が出るのは商品 CSV 取込み（説明 154 字）のみ、subtitle 持ち 5 page は説明が 4px 下がる（⑳ D8 実測）。
- **D-RT4 廃番表示 3 択 / PLU表示 5 択は SegmentedControl のまま**（DSR-02「3 つ以上は Tabs」の drift）。理由: 両者は「絞り込み条件」で tab（表示モード）ではなく、Select 化は 1 click 増で owner の「揃えたい」に反する。Tabs 化は route/search の view 切替と混同を招く。drift は catalog ⑨ D1 但し書きに既に記録済みで、Backlog「保留」へ 1 行起票（Coordinator、plan-first commit）し、DSR-02 側の例外明文化は design 判断として据え置く。
- **D-RT5 section h2 token sweep は `src/features` の `text-lg` を持つ h2 全部（`id` 付きの月次 / 日次 2 件を含む 27 箇所）**（右要素の有無を問わない。catalog ① variation は「page 内の sub-section 見出し」全体の token を `text-xl` と定義済みで、A/B だけ変えると同 page 内で h2 の大きさが混在する）。除外 = `HomePage.tsx` の `text-lg font-medium`（action group 見出しで、後続 home lane の footprint）。L3 は入出庫記録詳細 1 page を抜き取り（AC-L3-4）。
- **D-RT6 形態 C 2 箇所の 2 段化は既存 `space-y-*` の内側に `div.space-y-1` を新設して見出し行 + 説明 `<p>` を包む**（⑳ Final Review Opus P3-9）: IntegrityCheck は `section.space-y-4` > `div.space-y-1` > (`div.flex.flex-wrap.items-start.justify-between.gap-3` > h2 `min-w-0 flex-1` + Button `shrink-0`) + p。Stocktake は `div.space-y-2` > `div.space-y-1` > (row > h2 + Badge) + p、`<Progress>` は `space-y-2` 直下のまま。~~Gated Amendment 2: 右要素が h2 より低い Badge のときは `self-center`~~（**GA3 で撤回**: owner の指摘は toolbar の Checkbox で、Badge ではなかった。Badge と catalog ① variation は現行のまま）
- **D-RT7 棚卸し Checkbox は `PriceRevisionFilters` と同型の label 内包へ**: `<label htmlFor="stocktake-uncounted-only" className="flex items-center gap-2 self-center text-sm"><Checkbox id=… /> 未入力のみ表示</label>`（`PriceRevisionFilters.tsx:102-111` の class を写す。`self-center` は toolbar `items-end` 内で行の縦中央に置く ⑳ owner 回答 (b)）。`PriceRevisionFilters` の 2 Checkbox にも `self-center` を足す。**Gated Amendment 3（owner L3 round 1、2026-09-15「未入力のみ表示のチェックが浮いてる。枠的にそこが中心なのは分かるが他と比べて位置はおかしく見える」）: `self-center` を撤回。上置き label で行が高くなった結果、行の縦中央は label 行と入力行の間になり浮く。label class を `flex h-9 items-center gap-2 self-end text-sm` にし、入力（SelectTrigger の既定 `h-9`）と同じ高さの箱を行の下端に揃えて、その中で縦中央に置く = 他の入力と同じ帯に並ぶ。3 site（Stocktake 1 + PriceRevisionFilters 2）を揃える。catalog ⑨ D1 の Checkbox 句と mockup-g state-2 の `align-self:center` も同期する。** **Gated Amendment 4（fresh broad Opus P3-1）: `PriceRevisionFilters` の「取引先未設定の商品も含める」label は toolbar 行の外（`div.space-y-3` 直下、`:141-146`）にあり `self-end` は効かず `h-9` だけが残る。baseline の `flex items-center gap-2 text-sm` へ戻す。`h-9 items-center self-end` は toolbar 行内の 2 site（Stocktake「未入力のみ表示」+ PriceRevisionFilters「廃番を含む」）。**
- **D-RT8 toolbar wrapper の `items-end`**: 上置き label を持つ行は `flex flex-wrap items-end gap-3`（既存の上置き page と同じ）。対象 = `ProductListPage.tsx:155`（`items-center` → `items-end`）と `StocktakePage.tsx:746`（`items-center gap-4` → `items-end gap-3`）。**Gated Amendment 1（Final Review Opus P2-1）: `DailySalesPage.tsx:82`（`DateNavigator` + `DepartmentFilter`、`items-center justify-between gap-4`）も対象。census が DepartmentFilter 5 site 目を取りこぼしていた。`items-end` にして `DateNavigator` と下辺を揃える（`gap-4` / `justify-between` は不変）。** 他は既に `items-end`。

## Scope

- **S1 `DepartmentFilter.tsx`**: `:61-64` を `div.grid.gap-1` + `<label className="text-sm text-muted-foreground" htmlFor={triggerId}>` にし、未参照の `labelId`（`:58,62`）を削除。5 site 一括。
- **S2 `ProductListPage.tsx`**: `:155` を `items-end`。`:156-159` 並び替え / `:187-190` 表示件数の wrapper を `grid gap-1`。SegmentedControl 3 site（廃番表示 / PLU表示 / 並び順）に `showLabel`。
- **S2a（Gated Amendment 1）`DailySalesPage.tsx:82`**: toolbar wrapper `items-center` → `items-end`。`DailySalesPage.test.tsx` に wrapper の assert 1 本。
- **S3 `StockInquiryPage.tsx:132-135`**: 表示件数 wrapper を `grid gap-1`。
- **S4 `PriceRevisionFilters.tsx`**: 取引先 trigger wrapper `:62`（GA2 comment `:60-61` は「同じ wrapper に置いて群化」の趣旨を保ち、`grid gap-1` に更新）/ 表示件数 wrapper `:112` を `grid gap-1`、未参照の `id="price-revision-per-page-label"`（`:114`）を削除（`DepartmentFilter` の `labelId` と同型の死に id、Plan Review round 1 Opus P3-5）。Checkbox 2 site（`:102` / `:143`）に `self-center`（**GA3: `h-9 items-center self-end` へ**。**GA4: `:143` は toolbar 外のため baseline `flex items-center gap-2 text-sm` へ戻し、`h-9 items-center self-end` は `:102` のみ。test の for ループは「廃番を含む」1 件へ**）。
- **S5 `StocktakePage.tsx`**: toolbar `:746` を `items-end gap-3`。Checkbox `:756-770` を D-RT7 の label 内包 + `self-center`（**GA3: `h-9 items-center self-end` へ**）。表示件数 `:771-774` を `grid gap-1` + `Label` に `font-normal`。`StocktakeProgressHeader` `:385-410` を D-RT6 の 2 段（row は `items-start`、h2 に `min-w-0 flex-1`、Badge に `shrink-0`）。**Gated Amendment 1（Opus P3-7）: Badge の 2 分岐（warning / success）は `tone` と icon だけが違うため 1 要素に畳む（`shrink-0` の複製を増やさない。既存 `it.each([0,1])` が oracle）。** **GA4（fresh broad Opus P2-1）: その `it.each([0,1])` に icon の三項（未入力 > 0 = `svg.lucide-triangle-alert`、0 = `svg.lucide-circle-check`）を検知する assert を 1 行足す（先例 `PriceRevisionFilters.test.tsx:192` の `svg.lucide-chevron-down`）。** ~~Gated Amendment 2: 同 Badge の `className` を `shrink-0 self-center` に。~~（**GA3 で撤回**: GA2 は Coordinator が owner の L3 指摘〈Checkbox〉を Badge と誤読したもの。Badge は現行のまま）
- **S6 `IntegrityCheckPage.tsx`**: 表示件数 wrapper `:232` を `grid gap-1`、未参照の `id="integrity-check-per-page-label"`（`:234`）を削除。差異のある商品 `:324-333` を D-RT6 の 2 段（`items-end` → `items-start`、h2 `min-w-0 flex-1`、Button `shrink-0`）。
- **S7 `segmented-control.tsx`**: D-RT2 の `showLabel?` prop。
- **S8 `SearchBar.tsx:72` / `:178`**: `Label` に `font-normal`。
- **S9 `PageHeader.tsx`**: D-RT3。JSDoc / inline comment も同時更新。**Gated Amendment 1（Opus P3-4 / Sonnet P3）: file 冒頭 `:3` と JSDoc `:21` の「3 variant」→「4 variant」、一覧に `(d) title + description` を追加（catalog ① canonical 行の 4 variant と同期）。**
- **S10 h2 token sweep**: D-RT5。`src/features` の `text-lg` を持つ h2 27 箇所（`id` 付き `MonthlySalesPage.tsx:157` / `DailySalesPage.tsx:165` を含む）を `text-xl` に（HomePage の `font-medium` と非 h2 は除外）。
- **S11 test**（既存 assert の更新は意図を保つ。Plan Review round 1 Opus P2-1 / P2-2 / P2-5）:
  - `DepartmentFilter.test.tsx`: wrapper が `grid gap-1`、`labelId` 撤去後も `getByRole("combobox", { name: "部門" })` が解決、`aria-labelledby` 属性なし。
  - `segmented-control.test.tsx`: `showLabel` あり = `getByRole("group", { name: ariaLabel })` 解決 + `getByText(ariaLabel)` が `span` + group に `aria-label` 属性なし / `showLabel` なし = 現行 DOM（`aria-label={ariaLabel}`）。
  - `ProductListPage.test.tsx`: 3 group が name で解決 + 同文の可視 text。
  - `StocktakePage.test.tsx:1171`: `closest(".flex.flex-wrap.items-center.gap-4")` → `closest(".flex.flex-wrap.items-end.gap-3")`（filter-row containment の意図維持）。**`:1185,1261` の `el.className === "flex flex-wrap items-center gap-4"` は触らない**（枠なし旧 frame の不在 negative assert。新 class へ書き換えると変更後の root と一致して自己矛盾）。追加: Checkbox が `label` 内包 + `self-center`、`getByLabelText("未入力のみ表示")` 解決、`StocktakeProgressHeader` の説明 `<p>` が h2 row の sibling + `<Progress>` の親が `space-y-2`。
  - `IntegrityCheckPage.test.tsx:549`: `closest(".flex.items-center.gap-2")` → `closest(".grid.gap-1")`（bg-card 不在の round 2 回帰は維持）。追加: 差異のある商品の説明 `<p>` が row の sibling、`<h2 id="integrity-difference-heading">` に `min-w-0 flex-1`。
  - `PriceRevisionFilters.test.tsx:68`: `label.closest(".flex.items-center.gap-2")` → `closest(".grid.gap-1")`（⑲ GA2 群化回帰 = 同じ wrapper に label と trigger、の意図維持）。
  - `PageHeader.test.tsx:53-59,70-71,87-92`: ⑮ SC1「actions と副題と説明を同じ見出しグループに表示する」を D-RT3 の 2 段へ読み替え = header が `space-y-1`、見出し行（h1 の親）が `flex flex-wrap items-start justify-between gap-3`、h1 に `min-w-0 flex-1`、subtitle / description `<p>` の親が `header`（見出し行の sibling）、actions の親が `shrink-0`。追加: actions 無し 3 variant（title のみ / title + subtitle / title + subtitle + description。`BackupRestorePage.tsx:337` / `PluExportPage.tsx:363` が actions 無しで description を渡す実在形）の `container.innerHTML` を `toMatchInlineSnapshot` で固定（repo 初の inline snapshot。失敗定義「actions 無し 13 page の DOM 不変」の oracle。**inline snapshot は local の vitest で populate し、生成された literal を同 commit に含める**: `CI=true` の hosted では未 populate の inline snapshot は書き込まれず FAIL する）。
- **S12 docs**: catalog ① `:28` に `description?`（`PageHeader{title, subtitle?, description?, actions?}` の 4 variant）/ ⑨ `:640` block comment と ① の「runtime 反映は後続 lane」marker を撤去し「runtime 反映済み（本 PR）」へ / ⑨ `:647`「4 サイトが揃う」→「5 サイトが揃う」（実測 #1a）/ ⑨ `:650` 使用トークン段落の「sales TabsHeader の日次/月次、monthly ModeTabs 等」→「monthly ModeTabs、PLU書出しの書出しモード 等」（TabsHeader は SegmentedControl 非使用、closure round 2 Opus P3-E）/ ~~Gated Amendment 2: ① variation に Badge の `self-center` 1 句~~（GA3 で撤回）/ **Gated Amendment 3: ⑨ 使用トークン段落の Checkbox 句「toolbar 内では `self-center` で行の縦中央に置く（上置き label を持たないため `items-end` の下辺揃えに加わらない。owner 確認 2026-09-11）」→「toolbar 内では `h-9 items-center self-end`（入力と同じ高さの箱を行の下端に揃え、その中で縦中央。上置き label を持たないため行の中央では浮く。owner L3 2026-09-15）」/ mockup-g state-2 の `.toolbar-row .check{align-self:center}` → `{align-self:flex-end;height:36px;display:flex;align-items:center}`**（**GA4（Opus P3-2）: selector を `.proposed .toolbar-row .check` に絞る〈現行 panel の `.check` に効かせない〉。state-2 提案 h3「Checkbox は縦中央」→「Checkbox は入力と同じ帯」**）/ ⑤ アクセシビリティ段落 `:323` の「`role="group"` + `aria-label` で群を識別」の直後に「（フィルタ toolbar 内で可視 Label を出す場合は `aria-labelledby` で span に紐付け、`aria-label` は出さない）」を 1 句 / 更新履歴 1 行（PR 番号は Draft PR 作成後に Writer が埋める）。**Gated Amendment 1（Opus P3-1 / P3-2）: `:54`（① component gap の解消）と `:639`（⑨ block comment）の裸「本 PR」→「PR #63」。`:54` の「左 group の `<div className="min-w-0 flex-1 space-y-1">` にまとめた」「説明を左列内で折り返し、actions を右上に留めた」は ⑮ 時点の経緯として過去形に改め、末尾に「PR #63 で見出し行 + 説明行の 2 段へ移行（`PageHeader.tsx:31-45`）」を 1 句。① 構造 block と variation は触らない。**（**GA4: 重複のため「runtime 反映済み（PR #63、`PageHeader.tsx:31-45`）」へ畳む**）
- **S13（Coordinator、plan-first commit）**: Plans.md「次の行動」+ Wave Registry（wave 10、stacked train）/ Backlog の本 lane 行を「着手中」へ + DSR-02 drift の保留行 / 単位の拡張の店回答（2026-09-15）と blocker 解除の記録。Writer は触らない。

## Non-scope

- `src-tauri/**`、`docs/function-design/**`、`docs/design-system/01-decision-rules.md`、`docs/SCREEN_DESIGN.md`
- tab / mode 切替: `src/components/sales/TabsHeader.tsx`（class 定数 + `<Link>`、component 非使用）、`src/features/monthly-sales/components/ModeTabs.tsx`、`src/features/plu-export/PluExportPage.tsx:695` の「書出しモード」（Card 内の Diff / Full 切替 = mode 切替で toolbar のフィルタではない。`showLabel` を付けない）
- `HomePage.tsx` の h2、ProductAddSuggest wrapper 5 箇所、`ProductForm.tsx:226` / `BackupRestorePage.tsx:415,594` / `ProductImportPreview.tsx:132` の toolbar 外 Checkbox
- mockup-g の変更（確定済み参照。`:has()` は衛生 batch 4 で是正済み。**GA3 で state-2 の Checkbox CSS 1 行のみ例外**）
- 表示小修正 batch 2（stacked 後続 lane）の file 行: `StockInquiryPage.tsx:100`（PageHeader）/ `:225-257`、catalog ⑥ / ⑬

## Acceptance Criteria

rg oracle は出力空 = 0 件。baseline は起票時実測（origin/main `04143923`）。

- **AC1** `rg -c 'className="grid gap-1"' src/components/patterns/DepartmentFilter.tsx` = 1（baseline 0）/ `rg -c labelId src/components/patterns/DepartmentFilter.tsx` = 0（baseline 2）
- **AC2** `rg -n -U 'className="flex items-center gap-2">\n\s*<(label|Label)' src/components/patterns/DepartmentFilter.tsx src/features/products/ProductListPage.tsx src/features/stock-inquiry/StockInquiryPage.tsx src/features/products/components/PriceRevisionFilters.tsx src/features/stocktake/StocktakePage.tsx src/features/integrity-check/IntegrityCheckPage.tsx | rg -c 'className="flex items-center gap-2">'` = 0（baseline 8 = 実測表 #1 / #2a × 2 / #3 / #4 / #4b / #5b / #6 と 1:1。Plan Review round 1: 旧 `-B3` 窓は複数行 label の 2 site を取りこぼし baseline 4）/ `rg -c 'price-revision-per-page-label|integrity-check-per-page-label' src` = 0（baseline 2、死に id 撤去）/ `rg -n -B2 'id="price-revision-supplier-label"' src/features/products/components/PriceRevisionFilters.tsx | rg -c 'grid gap-1'` = 1（baseline 0）
- **AC3** `rg -c 'aria-labelledby' src/components/ui/segmented-control.tsx` ≥ 1（baseline 0）/ `rg -c 'showLabel' src/features/products/ProductListPage.tsx` = 3（baseline 0）/ `rg -c 'showLabel' src/features/monthly-sales/components/ModeTabs.tsx src/features/plu-export/PluExportPage.tsx src/components/sales/TabsHeader.tsx` = 0（mode 切替に付けない。各 file 0 件）
- **AC4** `rg -c 'font-normal' src/components/patterns/SearchBar.tsx` = 2（baseline 0）/ `rg -n -B1 '^\s*表示件数\s*$' src/features/stocktake/StocktakePage.tsx | rg -c 'font-normal'` = 1（baseline 0）
- **AC5** `rg -c '左グループ内で折り返す|左列内で折り返し' src/components/patterns/PageHeader.tsx` = 0（baseline 2）/ `rg -c 'min-w-0 flex-1 space-y-1' src/components/patterns/PageHeader.tsx` = 0（baseline 1）/ `rg -c 'min-w-0 flex-1 text-2xl' src/components/patterns/PageHeader.tsx` = 1（baseline 0）/ **GA1** `rg -c '3 variant' src/components/patterns/PageHeader.tsx` = 0（`57d4d4b3` 時点 2）+ `rg -c '4 variant' src/components/patterns/PageHeader.tsx` = 2（時点 0）
- **AC6** `rg -c 'items-center gap-4 rounded-lg' src/features/stocktake/StocktakePage.tsx` = 0（baseline 1）/ `rg -c 'items-end gap-3 rounded-lg' src/features/stocktake/StocktakePage.tsx` = 1（baseline 0）/ **GA3** `rg -c 'self-center' src/features/stocktake/StocktakePage.tsx src/features/products/components/PriceRevisionFilters.tsx` = 0（各 file 0。`836a04c8` 時点 1 / 2）+ `rg -c 'h-9 items-center gap-2 self-end' src/features/stocktake/StocktakePage.tsx` = 1 + `rg -c 'h-9 items-center gap-2 self-end' src/features/products/components/PriceRevisionFilters.tsx` = 1（時点 0 / 0。**GA4**: `eebab51b` 時点 2、toolbar 外の `:143` を戻す）+ **GA4** `rg -A1 'htmlFor="price-revision-include-unassigned"' src/features/products/components/PriceRevisionFilters.tsx | rg -c 'flex items-center gap-2 text-sm'` = 1（`eebab51b` 時点 0）/ `rg -c 'flex flex-wrap items-center gap-3' src/features/products/ProductListPage.tsx` = 0（baseline 1、2 段目 wrapper）/ **GA1** `rg -c 'items-end justify-between gap-4' src/features/daily-sales/DailySalesPage.tsx` = 1（baseline 0）+ `rg -c 'items-center justify-between gap-4' src/features/daily-sales/DailySalesPage.tsx` = 0（baseline 1）
- **AC7** `awk '/function StocktakeProgressHeader/,/^}/' src/features/stocktake/StocktakePage.tsx | rg -c 'items-start justify-between'` = 1（baseline 0）/ 同 awk で `rg -c 'space-y-1'` = 1（baseline 0）/ `rg -U -o '<h2[^>]*id="integrity-difference-heading"[^>]*>' src/features/integrity-check/IntegrityCheckPage.tsx | rg -c 'min-w-0 flex-1'` = 1（baseline 0。section 側に付いても通らない。closure round 2 Opus P2-A: D-RT6 後の h2 行は 105 文字で prettier〈printWidth 100〉が属性を折り返すため、単一行前提の pattern は false red になる）/ `rg -c 'items-end justify-between' src/features/integrity-check/IntegrityCheckPage.tsx` = 0（baseline 1）/ **GA1** `awk '/function StocktakeProgressHeader/,/^}/' src/features/stocktake/StocktakePage.tsx | rg -c 'shrink-0'` = 1（`57d4d4b3` 時点 2、Badge 1 要素化）+ **GA4** `rg -c 'lucide-triangle-alert' src/features/stocktake/StocktakePage.test.tsx` = 1 + `rg -c 'lucide-circle-check' src/features/stocktake/StocktakePage.test.tsx` = 1（`eebab51b` 時点 0 / 0）/ ~~GA2 の Badge `self-center` oracle~~（GA3 で撤回: `awk '/function StocktakeProgressHeader/,/^}/' src/features/stocktake/StocktakePage.tsx | rg -c 'self-center'` = 0 不変）
- **AC8** `rg -n '<h2[^>]*text-lg' src --glob '!*.test.tsx' | rg -v 'font-medium' | wc -l` = 0（baseline 27。属性順に依存しない pattern、HomePage の `font-medium` 3 件は自動除外）/ `rg -c 'text-lg font-medium' src/features/home/HomePage.tsx` = 3（不変）
- **AC9** `rg -c 'subtitle\?, description\?, actions\?' docs/design-system/02-component-catalog.md` = 1（baseline 0）/ `rg -c 'runtime 反映は後続 lane' docs/design-system/02-component-catalog.md` = 0（baseline 2）/ `rg -c '4 サイトが揃う' docs/design-system/02-component-catalog.md` = 0（baseline 1）/ `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c 'sales TabsHeader の日次/月次'` = 0（baseline 1。file 全体では ⑤ `:272` 使いどころの例示にも同 literal があり 2。⑤ は本 lane 非対象、closure round 3 Opus P2-F）/ `awk '/^## ⑤/,/^## ⑥/' docs/design-system/02-component-catalog.md | rg -c 'aria-labelledby'` ≥ 1（baseline 0）/ **GA3** `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c 'self-center'` = 0（`836a04c8` 時点 1）+ `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c 'h-9 items-center self-end'` = 1（時点 0）+ `rg -c 'align-self:center' docs/design-system/reference/mockup-g-filter-toolbar.html` = 0（時点 1）+ `rg -c 'align-self:flex-end' docs/design-system/reference/mockup-g-filter-toolbar.html` = 1（時点 0）/ `awk '/^## 更新履歴/,0' docs/design-system/02-component-catalog.md | rg -c 'runtime 反映'` ≥ 1 / **GA1** `rg -c '本 PR）' docs/design-system/02-component-catalog.md` = 1（`57d4d4b3` 時点 3。①`:54` と ⑨`:639` の 2 箇所を `PR #63` へ）+ `rg -c 'PageHeader.tsx:31-43' docs/design-system/02-component-catalog.md` = 0（時点 1）/ **GA4** `rg -c '^\.proposed \.toolbar-row \.check' docs/design-system/reference/mockup-g-filter-toolbar.html` = 1 + `rg -c '^\.toolbar-row \.check' docs/design-system/reference/mockup-g-filter-toolbar.html` = 0 + `rg -c 'Checkbox は縦中央' docs/design-system/reference/mockup-g-filter-toolbar.html` = 0（`eebab51b` 時点 0 / 1 / 1）+ `rg -c 'PageHeader.tsx:31-45' docs/design-system/02-component-catalog.md` = 1（不変。① `:54` 末尾の重複文を畳む）+ `rg -c 'PR #63 で見出し行 \+ 説明行の 2 段へ移行' docs/design-system/02-component-catalog.md` = 0（時点 1）+ `rg -c 'actions があっても副題・説明をタイトルと同じグループに保つ' src/components/patterns/PageHeader.tsx` = 0（時点 1）+ `rg -c '\(d\) title \+ description → .<header className="space-y-1">. \+ .<h1>. \+ .<p>.' src/components/patterns/PageHeader.tsx` = 1（時点 0。`.` は backtick）
- **AC10**（負の oracle）`git diff --name-only origin/main..HEAD -- src-tauri docs/function-design docs/design-system/01-decision-rules.md docs/SCREEN_DESIGN.md src/components/sales src/features/monthly-sales/components/ModeTabs.tsx src/features/plu-export src/features/home | wc -l` = 0
- **AC11** 対象 test（`DepartmentFilter` / `segmented-control` / `ProductListPage` / `StocktakePage` / `PageHeader` / `IntegrityCheckPage` / `StockInquiryPage` / `PriceRevisionFilters` / `SearchBar`）が vitest で PASS、`npm run typecheck` / `npm run lint` / `npm run format:check` PASS、最終 `bash scripts/local-ci.sh full` PASS（fresh worktree では `npm run generate:routes` を先に実行）
- **AC12** `bash scripts/doc-consistency-check.sh --target plan` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS
- **AC-L3-1** 商品一覧: 1 段目・2 段目とも label が入力の上、廃番表示 / PLU表示 / 並び順の SegmentedControl に可視 label、行の下辺が揃う（mockup-g state-1 提案 a と同じ見え方）。**GA1（Opus P3-5）: 2 段目で「並び替え」Select と「並び順」SegmentedControl の label が隣接して読めるか。違和感があれば文言案（例「昇順 / 降順」）を Backlog へ（本 lane は文言不変）。** 併せて日次売上の toolbar（`DateNavigator` と部門フィルタ）の下辺が揃う（S2a）
- **AC-L3-2** 棚卸し（`src/features/stocktake/StocktakePage.tsx` toolbar / `StocktakeProgressHeader`）: 「未入力のみ表示」Checkbox が入力と同じ帯（`h-9 items-center self-end`、GA3）、表示件数の label が上置き。棚卸し進捗の見出し行（h2 + 未入力 Badge）の下に「入力済み a / 全 b」の説明行。**L3 round 1（2026-09-15、`2e6e1d7c`）: 進捗の見出し行 + 説明行は PASS、toolbar の「未入力のみ表示」Checkbox が行の中央に浮いて見える = FAIL（Coordinator は一度 Badge と誤読し GA2 を作ったが owner の補足で訂正、GA3）→ Checkbox を `h-9 items-center self-end`。round 2 はこの 1 画面のみ再確認（Checkbox が部門 Select / 表示件数 Select と同じ帯に並ぶ）**
- **AC-L3-3** 商品 CSV 取込み: 説明文が「取込む」ボタンの下まで全幅で伸び、ボタンは右上に留まる。入庫（`ReceivingPage.tsx`、subtitle 持ち）は説明が見出し行の下、見た目は 4px 下がるのみ。同 page の section 見出し 3 つ（h2）が 20px で揃う（D-RT5 の入力 page 型の代表）
- **AC-L3-4** 入出庫記録詳細 1 page（`src/features/inventory-records/ReceivingRecordDetailPage.tsx`）: 「明細」等の h2 が 20px（`text-xl`、他 page の section 見出しと同じ大きさ）

## Design Sources

- Requirements / spec: owner 2026-09-08「検索ツールの場所の表記は揃えたい」/ 2026-09-11 Human Gate 回答 (a)(b)(c)（[⑳ packet](2026-09-10-filter-label-top-design.md) Workflow State）
- Architecture: 該当なし（UI 層のみ）
- Function / command / DTO: 該当なし（各画面 doc は toolbar の label 配置と h2 token を規定しない、⑳ Non-scope 実測）
- DB: 該当なし
- Screen / UI: `docs/design-system/02-component-catalog.md` ①（構造 block / セクション見出し variation）/ ⑤（アクセシビリティ段落）/ ⑨（使用トークン / DepartmentFilter 内部 block / アクセシビリティ）、`docs/design-system/reference/mockup-g-filter-toolbar.html` state-1〜4
- Decision log / ADR: DSR-01（主動線 1 個、不変）/ DSR-02（drift 記録、据え置き）/ DSR-10（候補ソース、不変）/ DSR-23（native select 禁止、不変）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | なし | not applicable |
| Command / DTO / generated binding / wire shape | なし | not applicable |
| DB / transaction / audit / rollback / migration | なし | not applicable |
| Screen / UI / route state / Japanese wording | catalog ① / ⑤ / ⑨ + mockup-g | existing sufficient（⑳ で確定。本 PR は ① canonical 行の `description?` と「後続 lane」marker の撤去のみ） |
| CSV / TSV / report / import / export format | なし | not applicable |
| Durable decision / ADR | DSR-02 drift の据え置き | Backlog 保留行（plan-first commit）。DSR 改訂は intentionally deferred |

## Registration / Generation Obligations

新規 command / route / doc / REQ なし。`SegmentedControl` の `showLabel?` prop は TypeScript の optional で bindings 非依存。generate 系の再実行義務なし（fresh worktree の `generate:routes` は vitest 前提であって成果物差分なし）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-FILTER-LABEL-RT-1 | catalog ⑨ 使用トークン / DepartmentFilter 内部 block | D-RT1 / D-RT8 | raw label + grid を canonical、`Label` 使用 site は `font-normal` 打ち消し。全置換は SearchBar 契約に触る | S1〜S6 / S8 | `DepartmentFilter.test.tsx` / 各 page test の accessible name |
| SPEC-FILTER-LABEL-RT-1 | catalog ⑤ アクセシビリティ | D-RT2 | `showLabel` で `ariaLabel` を span に可視化 + `aria-labelledby`。`aria-label` 併記は name の出どころ二重化、`label?: string` は文言の divergence を許す | S7 / S2 / S12 | `segmented-control.test.tsx` / `ProductListPage.test.tsx` |
| SPEC-FILTER-LABEL-RT-1 | catalog ⑨ D1 Checkbox 句 / mockup-g state-2 | D-RT7 | label 内包 + `h-9 items-center self-end`（GA3）。sibling 型は 1 site のみで揃える側 | S4 / S5 | `StocktakePage.test.tsx` `getByLabelText` |
| SPEC-FILTER-LABEL-RT-1 | catalog ① 構造 block | D-RT3 | actions 分岐のみ 2 段、actions 無し 13 page は不変（壊さない優先） | S9 | `PageHeader.test.tsx` |
| SPEC-FILTER-LABEL-RT-1 | catalog ① セクション見出し variation | D-RT5 / D-RT6 | 形態 C 2 箇所を 2 段、h2 token sweep（HomePage 除外） | S5 / S6 / S10 | `IntegrityCheckPage.test.tsx` / `StocktakePage.test.tsx` / AC8 |
| SPEC-FILTER-LABEL-RT-1 | DSR-02 | D-RT4 | 3 択 / 5 択は据え置き + Backlog 保留 | S13 | AC3 の tab 型 0 件 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes。catalog ① / ⑤ / ⑨ と mockup-g が確定文で、本 packet は file:line と裁定のみ
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-RT4（DSR-02 drift 据え置き）は Backlog 保留行へ。DSR 改訂は要らない（catalog ⑨ D1 但し書きが既に記録）
- Assumptions and constraints: `aria-labelledby` が jsdom / WebView2 で group の accessible name を与える（Contract Probe）
- Deferred design gaps, risk, and follow-up target: DSR-02 drift（Backlog 保留）/ ProductAddSuggest wrapper（Non-scope）/ HomePage h2（home lane）
- Test Design Matrix can cite design decision IDs or source doc sections: yes（D-RT1〜8）
- Absolute guarantee / escape hatch self-check completed: 「actions 無し page は DOM 不変」（inline snapshot 3 本）「mode 切替に `showLabel` を付けない」（AC3 / AC10）「accessible name 不変」（既存 page test）を失敗定義と S11 で機械検査

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable（UI 層のみ、CMD / BIZ 非接触） | — |
| Fact check / design decision split | 事実 = 起票時実測表（S6 との差 4 件: 5 site / +3 行 / -1 行 / `:143`）。判断 = D-RT1〜8 | 本 packet |
| Lifecycle / retry | not applicable（状態なし） | — |
| Operator workflow | label の位置と Checkbox の高さが変わる。操作手順・URL・候補は不変 | AC-L3-1 / 2 |
| Replacement path | not applicable | — |
| Data safety / evidence | not applicable（データ非接触） | — |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | 4 画面の抜き取り（商品一覧 / 棚卸し / 商品 CSV 取込み + 入庫 / 入庫記録詳細） | AC-L3-1〜4、github mode の manual record |
| 環境・再現性 | 新設の環境依存なし。fresh worktree の `generate:routes` は既知 | — |

## Design Readiness

- Existing design docs are sufficient because: ⑳ が catalog ① / ⑤ / ⑨ を確定文にし、mockup-g state-1〜4 を owner が実機 3 画面と並べて確認済み（2026-09-11）。本 packet は runtime の file:line と実装形（D-RT1〜8）を決めるだけ
- Source docs updated in this PR: catalog ① canonical 行の `description?`、①/⑨ の「後続 lane」marker 撤去、更新履歴
- Design gaps intentionally deferred: DSR-02 drift の是正（Backlog 保留）
- Durable decisions discovered in this plan and promoted to source docs: なし

Minimum design checks:

- Layer ownership: UI のみ
- Backend function design: 非接触
- Command / DTO / data contract: 非接触
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 文言不変（label 文字列は現行のまま）
- Error, empty, retry, and recovery behavior: 不変
- Testability and traceability IDs: SPEC-FILTER-LABEL-RT-1、REQ 追加なし

## Contract Probe

- `div[role="group"][aria-labelledby]` が testing-library の `getByRole("group", { name })` で span の text を name に採る: 前提は dom-accessibility-api の accname 実装（aria-labelledby は role を問わず最優先）。Writer が `segmented-control.test.tsx` に「`showLabel` → `getByRole("group", { name: ariaLabel })` 解決 + `aria-label` 属性なし」を最初に書き、RED → GREEN で確認する（実装 run の最初の test を probe とする。既存 `ProductListPage.test.tsx:769,827` の `aria-labelledby` 利用実績が傍証）
- WebView2 の実描画: mockup-g で owner 確認済み。runtime 差は L3 で確認（AC-L3-1〜4）

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| ⑨ 使用トークン「すべてのフィルタ入力は `div.grid.gap-1` + 上置き label（weight 400）」+ D-RT8 `items-end` | S1〜S6 / S8 | AC1 / AC2 / AC4 / AC6 + 各 page test の accessible name | AC-L3-1 / 2 |
| ⑨ DepartmentFilter 内部 block（`htmlFor` → SelectTrigger id、`labelId` 不要） | S1 | `DepartmentFilter.test.tsx` | — |
| ⑨ D1 Checkbox 句「label 内包 + `h-9 items-center self-end`」（GA3） | S4 / S5 | `StocktakePage.test.tsx` `getByLabelText("未入力のみ表示")` + AC6 | AC-L3-2 |
| ⑨ アクセシビリティ（live 型は可視 label のみ、aria-label なし） | S8（class のみ） | `SearchBar.test.tsx` 既存（不変） | — |
| ⑨ commit 型 SearchBar（`SearchBar.tsx:72`、横並び label のまま） | S8（`font-normal` のみ、配置は据え置き） | — | non-scope: runtime 呼び出し 0（`rg -n '<SearchBar' src` = 6 site 全部 live 型、59 §59.1「採用箇所なし」）のため横並びを据え置く。撤去は別判断（GA1、Opus P3-8） |
| ⑤ アクセシビリティ「toolbar 内は上置き Label、tab / mode は Label なし、`ariaLabel` 常に必須」（site census: toolbar 3 / mode 2〈ModeTabs / PluExport〉/ TabsHeader は component 非使用） | S7 / S2 / S12 | `segmented-control.test.tsx` + AC3 | AC-L3-1 |
| ⑤ 使いどころ「二択」（3 択 / 5 択は DSR-02 drift） | D-RT4（据え置き） | — | non-scope、Backlog 保留 |
| ① 構造 block（header `space-y-1` > 見出し行 > h1 `min-w-0 flex-1` + `shrink-0`、説明は全幅、`subtitle !== undefined`） | S9 | `PageHeader.test.tsx` + AC5 | AC-L3-3 |
| ① 「主動線が無い画面は h1 のみ」（actions 無し分岐不変） | S9（非接触） | `PageHeader.test.tsx` の inline snapshot 3 本（S11 新設） | — |
| ① セクション見出し variation（`text-xl font-semibold`、2 段、h2 `min-w-0 flex-1`、右要素 `shrink-0`） | S5 / S6 / S10 | AC7 / AC8 + `IntegrityCheckPage.test.tsx` / `StocktakePage.test.tsx` | AC-L3-2 / 4 |
| ① canonical props（`description?` を含む） | S12 | AC9 | — |
| DSR-01 主動線 1 個 | 非接触 | — | — |
| DSR-10 候補は master 全件 | 非接触 | `DepartmentFilter.test.tsx` 既存 | — |
| DSR-23 native select 禁止 | 非接触 | — | — |
| 既存 test の accessible name 契約（`combobox` name「表示件数」等） | S1〜S6 | 既存 page test が無変更で PASS | — |

隣接契約 sweep: catalog ⑨ の「入力の確定経路」「フィルタ候補のソース」「状態（disabled / focus）」は非接触、① の「詳細ルートの戻る導線」は actions 持ち page に含まれ D-RT3 の 2 段が当たる（説明なし page は DOM 不変の一部として AC5 / `PageHeader.test.tsx`）。

## Test Plan

Test Design Matrix: [2026-09-15-filter-label-top-runtime](test-matrices/2026-09-15-filter-label-top-runtime.md)。

- targeted tests: S11 の test file を実装と同 commit で更新（RED → GREEN、`segmented-control.test.tsx` を最初に）
- negative tests: mode 切替に `showLabel` なし（AC3）、actions 無し PageHeader の DOM 不変（`PageHeader.test.tsx` inline snapshot 3 本）、`aria-label` 属性が `showLabel` 時に無い
- compatibility checks: 既存 page test の `getByRole("combobox", { name })` が無変更で PASS
- data safety checks: not applicable
- main wiring/integration checks: `ProductListPage.test.tsx` で 3 group が name で解決 + 可視 text（`showLabel` が実際に渡っている）
- Human Gate に L3 を含むため、Writer 完了時に `cargo check --release` を実行する（Rust 非接触だが手順どおり）

## Boundary / Wire Contract

not applicable（JSON / CSV / DTO / bindings / route state 非接触。`SegmentedControl` の `showLabel?` は component props で wire ではない）。

## Review Focus

- D-RT2: `aria-label` を出力しない選択が ⑤「`ariaLabel` 常に必須」と両立するか（prop 必須・属性は `showLabel` で切替。S12 で ⑤ に 1 句）
- D-RT3: actions 無し 13 page の DOM 不変が test で担保されるか
- D-RT5: h2 sweep 27 箇所の除外（HomePage）と、`text-lg` を持つ非 h2（`p`）を触らないこと
- S11 の既存 assert 更新（Stocktake `:1171` / IntegrityCheck `:549` / PriceRevisionFilters `:68` / PageHeader）が各回帰の意図（filter-row containment / bg-card 不在 / GA2 群化 / ⑮ SC1）を保つか。`:1185,1261` を触らないこと
- ⑲ GA2 の群化 comment（`PriceRevisionFilters.tsx:60-61`）が `grid gap-1` 後も真か

## Spec Contract

Contract ID: SPEC-FILTER-LABEL-RT-1

- ⑳ が確定した catalog ⑨ / ⑤ / ① の規範（上置き label、Checkbox は入力と同じ帯（`h-9 items-center self-end`、GA3）、SegmentedControl 可視 label、PageHeader / section 見出しの 2 段、h2 token）が runtime へ反映され、accessible name・search state・候補ソース・文言は不変で、catalog の「runtime 反映は後続 lane」marker が消える

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-FILTER-LABEL-RT-1 | S1 | `DepartmentFilter.test.tsx` | 5 site 一括 | AC1 |
| SPEC-FILTER-LABEL-RT-1 | S2 / S3 / S4 / S6 | 各 page test（accessible name） | wrapper / `items-end` | AC2 / AC6 |
| SPEC-FILTER-LABEL-RT-1 | S7 + S2 | `segmented-control.test.tsx` / `ProductListPage.test.tsx` | D-RT2 | AC3 |
| SPEC-FILTER-LABEL-RT-1 | S5 | `StocktakePage.test.tsx` | Checkbox 内包 / 進捗 2 段 | AC6 / AC7 |
| SPEC-FILTER-LABEL-RT-1 | S8 | `SearchBar.test.tsx`（不変） | class のみ | AC4 |
| SPEC-FILTER-LABEL-RT-1 | S9 | `PageHeader.test.tsx` | D-RT3 | AC5 |
| SPEC-FILTER-LABEL-RT-1 | S10 | typecheck / rg | D-RT5 | AC8 |
| SPEC-FILTER-LABEL-RT-1 | S12 | docs review | literal 同期 | AC9 / AC12 |
| SPEC-FILTER-LABEL-RT-1 | 全体 | owner L3 | 実描画 | AC-L3-1〜4 |

## Data Safety

- 実データ・secret の混入なし（UI 層のみ、fixture は既存 test の合成データ）
- local-only paths: `.local/codex-orders/**`、scratchpad の draft worktree
- synthetic-only paths: `src/**/*.test.tsx` の fixture

## Implementation Results

[PR #63](https://github.com/higekiri-w/inventory-system-desktop/pull/63) で実装し squash merge 済み（`bb1862a5`、2026-09-16）。Draft PR head の遷移: `57d4d4b3`（初回 broad）→ GA1 是正 `2e6e1d7c` → owner L3 round 1 → GA3 是正 `eebab51b` → L3 round 2 → fresh broad → GA4 是正 `24f100b2` → L3 round 3 → Ready → hosted CI pass → merge。owner 介入 3 / 予算 4（L3 ×3。Ready / merge は 2026-09-16 の包括指示）。relay 4 / 上限 4（発注書 49 / 52 / 54 / 55）。専用 record（`inventory-workflow-v1`、PR #63 comment 5677564499）に review pass（broad: Sonnet `#issuecomment-5681732910` / Opus `#issuecomment-5681723093`）と manual pass（L3 round 1〜3、`#issuecomment-5682176786`）を記録。

## Review Response

- Findings Freeze: frozen after Broad Audit（2026-09-15、Sonnet + Opus + Codex review 50 の 3 pass 完了時点）; post-freeze exceptions: none.

### Plan Review round 1（2026-09-15、plan-gate、Sonnet + Opus 並列、裁定 Coordinator）

- Sonnet P1-1 / Opus P2-2（`StocktakePage.test.tsx:1185,1261` の literal 更新は自己矛盾）= accept → S11 を「触らない」+ `:1171` の `closest` 更新へ。Matrix F7 同期
- Sonnet P1-2 / Opus P2-4（AC2 の `-B3` 窓が複数行 label の 2 site を見ない、baseline 4）= accept → Opus 案の `-U` 隣接 pattern（baseline 8）へ置換
- Sonnet P1-3 / Opus P2-3（AC8 が `id` 付き h2 2 件を見ない、baseline 25）= accept → `<h2[^>]*text-lg` + `font-medium` 除外（baseline 27）へ。S10 / D-RT5 / 実測 #10 同期
- Sonnet P2-1 / Opus P2-6（PluExport `:696` の SegmentedControl が未監査、`src/features/sales` 不在、TabsHeader は component 非使用）= accept → Non-scope / AC3 / AC10 / 実測 #7 / Ledger ⑤ 行を訂正
- Sonnet P2-2（AC3 の path 不在）= accept（上と同じ是正）
- Opus P2-1（壊れる既存 test は Stocktake `:1171` / IntegrityCheck `:549` / PriceRevisionFilters `:68` / PageHeader 4 箇所）= accept → S11 を file:line + 意図保持方針で書き直し、実測 #12 訂正
- Opus P2-5（snapshot test が repo に無い）= accept → S11 に inline snapshot 2 本の新設、Matrix C5/F5 同期
- Opus P3-1（`label?: string` → `showLabel?: boolean`）= accept（diff と divergence が小さい）→ D-RT2 / S2 / S7 / AC3 / Trace / Matrix
- Opus P3-2（catalog `:647`「4 サイト」）= accept → S12 / AC9
- Opus P3-3（⑤ `:323` の `aria-label` 記述）= accept → S12 に 1 句 / AC9
- Opus P3-4（S4 の行番号 `:113` → `:112`）= accept
- Opus P3-5（`id="price-revision-per-page-label"` / `id="integrity-check-per-page-label"` が未参照。`rg -n 'price-revision-per-page-label|integrity-check-per-page-label' src` は定義行の 2 hit のみで `htmlFor` / `aria-labelledby` からの参照なし）= accept → S4 / S6 で撤去、AC2
- Opus P3-6（AC7 第 3 oracle が section にも当たる）= accept → `<h2 id=` 限定
- Opus P3-7（AC-L3-3 に入庫の h2 3 つ）= accept
- Sonnet 残る不確実性「SC4a 重複 test」= 本 lane 非起因の既存重複、非対応（Backlog 起票もしない。実害なし）
- Opus 残る不確実性「AC4 の `-B1` 窓と prettier 折返し」= S5 は `font-normal` 1 語追加のみで 1 行に収まる。Writer が 3 語書きにしたら AC4 が false red になるため、発注書に「`font-normal` のみ追加」を明記
- round 2 = closure（Sonnet + Opus、`b771eb61` の diff 限定）

### Plan Review round 2（closure、2026-09-15）

- Sonnet: round 1 の全 finding（上の裁定一覧と 1:1）closed、新規 P1/P2 なし → Findings Freeze 可
- Opus: round 1 の全 finding closed、**新規 P2-A**（P3-6 の是正先 AC7 第 3 oracle が単一行前提で、D-RT6 後の h2 行 105 文字 > printWidth 100 の折返しで false red）= accept → `rg -U -o` のタグ抽出へ差し替え（baseline 0 を Coordinator が再実行で確認）/ P3-B（inline snapshot を description 付き 3 variant に）= accept / P3-C（実測 #7 の行番号が `ariaLabel` 行で +1）= accept → `135,143,179` / `19` / `695` に統一 / P3-D（`CI=true` で未 populate の inline snapshot が FAIL）= accept → S11 と発注書に populate の 1 句 / P3-E（catalog ⑨ `:650` の「sales TabsHeader」が census と矛盾）= accept → S12 / AC9 に追加
- round 3 = closure（Opus、`3ee233a8` + `41a9e7e7` の diff 限定。round 天井 3 の最終）

### Plan Review round 3（closure、Opus、2026-09-15）→ 天井到達の disposition

- round 2 の P2-A / P3-B / P3-C / P3-D = closed。P3-E = partially closed
- **新規 P2-F**: AC9 に足した `rg -c 'sales TabsHeader の日次/月次'` の baseline を 1 と書いたが file 全体では 2（⑤ `:272` 使いどころの例示 + ⑨ `:650`）。S12 は ⑨ のみ直すため実装成功時に false red
- disposition（DEV_WORKFLOW Review Rules「round 数 3 を天井」: 同型指摘の一括是正）: round 1〜3 の P2 はすべて「Coordinator が oracle の baseline / 参照を rg せずに書いた」同型のため、次 round を開始せず、Coordinator が packet の全 AC oracle を現物で一括再実測して固定した（本 commit。結果: AC1 0/2、AC2 8/2、AC3 0/0/0、AC4 0/0、AC5 2/1/0、AC6 1/0/0/0/1、AC7 0/0/0/1、AC8 27/3、AC9 0/2/1/⑨限定 1/⑤ aria 0/更新履歴 2、AC10 pathspec 実在、いずれも記載どおり）。P2-F は AC9 を ⑨ 節限定の awk oracle（baseline 1）へ差し替えて閉じる。⑤ `:272` の例示（TabsHeader は class 定数で同じ二択切替 pattern を実装しており「例」としては誤りでない）は本 lane 非対象のまま
- P3-F（round 1 記録の「inline snapshot 2 本」）= 履歴表記のため放置（round 2 節に経緯あり）
- Plan Gate 判定: P1/P2 = 0（P2-F は上の是正で closed、独立再確認は天井のため行わない。Writer は実装 run で AC9 の command をそのまま実行するので、false red が残れば fail-closed で検出される）

### Final Review round 1（Broad Audit、2026-09-15、対象 head `57d4d4b3` = Draft PR #63）

- pass A Sonnet: P1 0 / P2 0 / P3 1（PageHeader「3 variant」pre-existing）。AC1〜AC12 自走一致、mutant 10/10 kill。Freeze 可
- pass B Opus: P1 0 / P2 1 / P3 8。AC 自走一致、mutant 8/8 kill、merge-tree（㉒ `56f1662c`）の衝突は `docs/Plans.md` のみ。Freeze 可
- pass C Codex（review 50、[PR review](https://github.com/higekiri-w/inventory-system-desktop/pull/63#pullrequestreview-5207788126)）: P1 / P2 / P3 0。AC 全 PASS、mutant 10/10、commands 78。Freeze 可
- 裁定（Coordinator、Gated Amendment 1 = 本 commit + 登録 commit、是正は Codex 発注書 52）:
  - Opus P2-1（`DailySalesPage.tsx:82` toolbar が `items-center`、D-RT8 census 漏れ）= accept → S2a / D-RT8 / AC6 / AC-L3-1
  - Opus P3-1（catalog 裸「本 PR」2 箇所）/ P3-2（`:54` の旧構造の現在形記述 + `:31-43`）= accept → S12 / AC9
  - Opus P3-4 / Sonnet P3（PageHeader「3 variant」）= accept（本 PR が catalog を 4 variant にしたため同 PR で同期）→ S9 / AC5
  - Opus P3-7（Badge 2 分岐の重複）= accept（ponytail、`it.each` が oracle）→ S5 / AC7
  - Opus P3-8（commit 型 SearchBar の据え置き理由）= accept → Ledger 1 行
  - Opus P3-5（「並び替え」「並び順」の隣接）= L3 観点へ → AC-L3-1
  - Opus P3-3（`59-ui-shared-patterns.md` §59.1 の PageHeader props / DepartmentFilter 採用画面が未同期）= Backlog 起票（closeout。function-design は AC10 の負 oracle で本 lane 非対象）
  - Opus P3-6（`StocktakePage.test.tsx` SC4a 重複 + 空集合化した negative assert）= Backlog 起票（closeout）
- Gated Amendment 1 = `69c6011c`（content）+ 登録 commit。是正後 = closure（Sonnet + Opus、diff 限定。Codex は review 50 の findings 0 のため closure 不要）→ manual（owner L3）→ Ready

### Gated Amendment 2（2026-09-15、owner L3 round 1）

- L3 round 1（head `2e6e1d7c`、この change での介入 1 回目 / 予算 4）: AC-L3-1 PASS / AC-L3-3 PASS / AC-L3-4 PASS / 日次売上 PASS / **AC-L3-2 の棚卸し進捗 Badge が FAIL**（owner 原文「未入力が浮いてる。枠的にそこが中心なのは分かるんだけど他と比べて考えると位置はおかしく見える」、screenshot は owner 端末）
- 原因: D-RT6 の見出し行は `items-start`（catalog ① variation どおり）で、右要素が Button（h-9、h2 より高い）なら違和感が無いが、Badge（h2 より低い）は上端に付いて浮く。mockup-g に棚卸し進捗は無く owner 未視認だった（⑳ Final Review Opus P3-9 の懸念どおり）
- 是正: Badge に `self-center`（1 語）+ catalog ① variation に「h2 より低い右要素は `self-center`」の 1 句。row の `items-start` と他 site は不変。Codex 発注書 54 → closure（Sonnet 1 pass、diff 限定）→ L3 round 2（棚卸し 1 画面、介入 2 回目）
- Findings Freeze 後の変更だが runtime（実描画）で証明された owner 指摘のため blocker として扱う（Review Rules ②）

### Gated Amendment 3（2026-09-15、owner L3 round 1 の訂正）

- owner の補足「一応だけど未入力のみっていうチェックボックスの位置だよ」で、GA2 が Coordinator の誤読（Badge）だったと判明。GA2 は撤回（Badge / catalog ① variation は現行のまま）、AC6 の GA2 補正も戻す
- 正しい指摘: toolbar の「未入力のみ表示」Checkbox が `self-center` で行の縦中央に置かれ、上置き label で高くなった行の中では label 行と入力行の間に浮く（screenshot は owner 端末、2026-09-15）
- 是正: 3 site の Checkbox label を `flex h-9 items-center gap-2 self-end text-sm`（入力と同じ高さの箱を下端に揃え、その中で縦中央）。catalog ⑨ D1 句と mockup-g state-2 を同期。Codex 発注書 54（書き直し）→ closure（Sonnet 1 pass、diff 限定）→ L3 round 2（棚卸し 1 画面。一括価格改定は同型のため抜き取り任意）
- 教訓: L3 所感は対象 component を owner に確かめてから裁定する（screenshot に 2 つの候補が写っていた）

### Gated Amendment 4（2026-09-15、Amendments 変更後の fresh broad の裁定）

GA3 で Amendments が変わったため helper 規則により Final Review を fresh broad で取り直した（対象 head `eebab51b`、Sonnet + Opus、独立 fresh context）。Sonnet = P1 0 / P2 0 / P3 1（packet 内 `self-center` 残存、docs のみ）= pass 可。Opus = P1 0 / P2 1 / P3 8 = pass 可。P2 があるため本 GA4 で裁定し、是正発注 55（relay 4 往復目、owner 承認 = 起動）→ closure（Sonnet 1 pass、diff 限定）→ broad 2 本を record → Ready。manual（owner L3 round 2 PASS）は再取得しない（是正は test / class 戻し / docs で、L3 対象 4 画面の見え方を変えない。一括価格改定「取引先未設定」は baseline へ戻る）。

- **Opus P2-1（GA1 で 1 要素に畳んだ棚卸し進捗 Badge の icon 三項〈warning / success〉を検知する test が無い）= accept** → S5 に GA4 の 1 行、AC7 に oracle 2 本
- **Opus P3-1（`PriceRevisionFilters.tsx:141-146`「取引先未設定の商品も含める」は toolbar 行の外で `self-end` 無効、`h-9` だけが効く）= accept** → D-RT7 / S4 に追記、baseline の class へ戻す。AC6 の期待値 2 → 1 + 戻し oracle 1 本。test の for ループは 1 件へ
- **Opus P3-2（mockup-g `:53` `.toolbar-row .check` が現行 panel にも効き before / after の比較にならない。GA3 closure の P3 と同件）= accept** → S12 に追記、`.proposed` に scope。h3 の文言も同期。AC9 に oracle 3 本
- Opus P3-3（Badge / Button base に `shrink-0` があり、明示 class と test の assert が恒真）= Backlog（closeout で起票。挙動影響なし）
- Sonnet P3 / Opus P3-4・P3-5（packet Trace / Ledger / AC-L3-2 と Matrix C3 / F6 / Adjacent の `self-center` 残存）= Coordinator 修正（本 commit で `h-9 items-center self-end` へ同期）
- Opus P3-6（Plans.md lane 1 行が「Draft PR 未作成、介入 0/4」のまま）= Coordinator 修正（本 commit）
- Opus P3-7（`PageHeader.tsx` JSDoc の (d) 行だけ体裁が違い、直後の「actions があっても副題・説明をタイトルと同じグループに保つ」が 2 段化前の文）= accept → (d) を (a)〜(c) と同じ体裁に、直後の 1 文を「actions は見出し行の右上に置き、副題・説明はその下の全幅の説明行に置く。」だけにする。AC9 に oracle 2 本
- Opus P3-8（catalog ① `:54` 末尾で「runtime 反映済み（PR #63）」と GA1 で足した「PR #63 で見出し行 + 説明行の 2 段へ移行（`PageHeader.tsx:31-45`）」が重複）= accept → 後者を畳み「**runtime 反映済み（PR #63、`PageHeader.tsx:31-45`）**。」の 1 文に。AC9 に oracle 1 本（`PageHeader.tsx:31-45` = 1 不変）
- Owner Effort Budget: relay 3 → 4

### Final Review closure（GA4 是正、head 24f100b2、2026-09-16）

Sonnet closure = findings 0、GA4 oracle 11 本一致、mutant 2 本 kill（`#issuecomment-5681732910`）。Opus 完了判定 = P1 0 / P2 0 / P3 4（docs 文言同期のみ。P3-1 Spec Contract の「Checkbox 縦中央」/ P3-2 Matrix `:124` の同語 / P3-3 mockup-g `:73` `:79` の「行の縦中央」/ P3-4 S12 GA1 節の注記、`#issuecomment-5681723093`）。P3-1 / P3-2 / P3-4 は本 closeout で archive 側に同期、P3-3 は同 PR で mockup-g を修正、fresh broad Opus P3-3（`shrink-0` 恒真 assert）は Backlog へ。

## 後続

- `shrink-0` 恒真 assert（`Badge` / `Button` の base に既存、`StocktakeProgressHeader` の明示 class と test の `toHaveClass("shrink-0")` が恒真。fresh broad Opus P3-3、PR #63）は [Backlog「記録目的」](../../backlog.md#記録目的受容済みリスクrevisit-条件付き)を参照。
- 一括価格改定の取引先紐付けを既定 off + 文言明示（owner 決定 2026-09-16）は [Backlog「次に動く lane」](../2026-10-19-backlog-closed.md#次に動く-lane順番固定)を参照。
- destructive Alert の soft 塗り + 三角 icon（owner 所感 2026-09-15、L3 で確定）は [Backlog「やると決めたもの」](../2026-10-19-backlog-closed.md#やると決めたもの順番未定)を参照。
