# Plan Packet: ⑳ フィルタ入力の Label 上置き統一 + セクション見出しの規範化（design-first、docs-only）

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

If a state-only commit materializes multiple phases, list the complete adjacent forward sequence and the pre-existing evidence for every intermediate transition in an append-only review/evidence record. Recording compression never permits a gate skip.

- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: d869b123
- Amendments: 1cafa865 e046f389 4c4f28bd 8422f845
- Coordinator: Fable 5.1
- Writer: Codex
- Plan Reviewer: Sonnet + Opus
- Final Reviewer: Sonnet + Opus + Codex
- Reviewed Content HEAD: 17fc8b66
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required（docs-only のため Ready 後の hosted final は owner `workflow_dispatch`。Ready 案内に明記する）
- Human Gate: owner mockup-g 確認（culling 2 件、Coordinator 既定を mockup の両案で確認する形: (1) フィルタ toolbar 内の SegmentedControl〈廃番表示 / PLU表示 / 並び順〉に上置き Label を付ける〈既定 = 全部付ける。state-1 で並び順の Label あり / なしを両方提示〉(3) Checkbox は label 内包の横並び維持〈既定〉。旧 (2) セクション見出しの component 化は class 統一と描画結果が同じで mockup では見分けられないため Coordinator 決定〈class / 文型 / token 統一のみ〉へ格下げ、D4 に記録）+ Ready 承認 → **回答（owner 2026-09-11、設問番号と衝突しないよう (a)(b)(c) で表記）**: (a) state-1 = 提案 a〈toolbar 内 SegmentedControl 全部に上置き Label、設問 (1)〉/ (b) state-2 = 提案を採り、「未入力のみ表示」Checkbox は行の縦中央〈`self-center`、設問 (3)〉/ (c) state-3・4 = 形態 C は見出し行 + 説明行の 2 段（説明をボタンの横で折り返さない）、`PageHeader` (c) にも同じ配置を適用〈owner 選択。実測: `PageHeader` 28 page 中 actions 持ち 15、うちボタンで説明が折り返すのは商品 CSV 取込み 1 page のみ〉。Gated Amendment 2 / 3 で確定文化。**owner Ready 承認 2026-09-11**（「Ready しよか」、merge まで Coordinator 代行）

## Owner Effort Budget

- 介入回数上限: 4（docs-only design。⑱ の実績 = mockup 2 round）
- 実働時間上限: 20分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

§5.5を使わないchangeは両方`none`のままにする。使う場合はtarget branch / PRへorder commitを混ぜず、artifact pathと専用remote order branch refを宣言する。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
docs-only（design-system canonical docs + mockup）。runtime 契約は変えない。後続 runtime lane の前提になる規範を書くため Plan Packet は必要だが、Test Matrix は rg oracle 中心の軽量版。

## Goal

Goal Invariant:

### 最小完了条件

- catalog ⑨ に「フィルタ入力（検索 / Select / date / number / `DepartmentFilter` / 並び替え / 表示件数 / toolbar 内の SegmentedControl）は可視 label を `grid gap-1` で上置きし、toolbar は `items-end`」が**全フィルタ入力の規範**として書かれ、例外（Checkbox 横並び / tab・mode 切替の SegmentedControl）が明文化されている
- catalog ① に「セクション見出し（h2 + 右要素 + 説明）」の variation が新設され、**見出し行 + 説明行の 2 段**（見出し行 `flex flex-wrap items-start justify-between gap-3`、見出し `min-w-0 flex-1`、右要素 `shrink-0`、説明は行の下に全幅）であること、`PageHeader` (c) も同型（D8）であること、「1 ページ 1 h1 は不変」が書かれている（Gated Amendment 2 / 3 で折返し契約から改稿）
- 適用範囲（どの画面のどの入力 / どの見出しが後続 runtime lane の対象か）が本 packet に file:line で記録され、runtime lane 申し送りになっている
- owner が mockup-g で現行 / 提案を並べて確認し、culling 2 件に回答している

### 失敗定義

- 規範が live SearchBar だけの記述のまま（⑭ の現状）で終わる
- catalog に page の file:line が書かれ、runtime lane 後に即 stale になる
- 1 ページ 1 h1 の規則が崩れる（セクション見出しを `PageHeader` で描く指示になる）
- `src/**` に diff が出る

### 非目的

- runtime 変更（後続 runtime lane。`DepartmentFilter` 1 component の改修で 4 サイトが揃う）
- backbone 原則 7「live + 検索ボタン併記」と catalog ⑨「ボタンなし」の drift 解消（backbone 自身が batch 1〜2 の宿題と記述済み、別件）
- 新規 DSR の起草（Label 配置は catalog の構造 / token 規約で足りる、DSR は「なぜ」を要する判断規則）
- ⑲ 取引先ピッカー runtime lane の trigger の Label 配置（⑲ は現行配置維持、runtime lane で本規範に揃える）

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（2026-09-10、worktree base `bb081e3`、Explore 報告を Coordinator が rg で再確認）

### Backlog の前提訂正

- Backlog `Plans.md:167` の「操作ログ / 取引先管理 等の他フィルタ入力を sweep」は前提が古い。**`OperationLogsPage` / `InventoryRecordsPage` / `StockMovementsPage` は既に全入力 `grid gap-1` 上置き**（`OperationLogsPage.tsx:349-425` / `InventoryRecordsPage.tsx:161-296` / `StockMovementsPage.tsx:135-207`）。取引先管理はフィルタ入力自体が無い（⑲ が検索を新設）。commit 型 `SearchBar` は採用箇所 0（catalog `:606` literal「現在の採用箇所なし・機能残置」）
- 実際の未統一面 = (a) `DepartmentFilter`（shared、`flex items-center gap-2` 横並び `DepartmentFilter.tsx:61-64`、4 サイト）(b) `<label>` + `Select` の横並び対（並び替え / 表示件数 / 取引先）(c) `StocktakePage` の toolbar（`items-center gap-4` `:746`、Checkbox / 表示件数）(d) `IntegrityCheckPage` 表示件数（`:232-251`）

### フィルタ入力の現状（page × input × 配置）

| page | input | component | Label | 配置 | file:line |
|---|---|---|---|---|---|
| 商品一覧（toolbar `flex flex-wrap items-end gap-3` `:106`） | 検索 | SearchBar live | 商品を検索 | 上置き | `ProductListPage.tsx:111-118` |
| 〃 | 部門 | DepartmentFilter | 部門 | 横並び | `:119-129` |
| 〃（同 1 段目） | 廃番表示 / PLU表示 | SegmentedControl（3 択 / 5 択、**両方に「すべて」が隣接**） | `ariaLabel` のみ | 可視 Label なし | `:135-142` / `:143-150` |
| 〃（2 段目 `flex flex-wrap items-center gap-3` `:155`） | 並び替え | `<label>`+Select | 並び替え | 横並び | `:156-159` |
| 〃 | 並び順 | SegmentedControl（2 択） | `ariaLabel` のみ | 可視 Label なし | `:179-186` |
| 〃 | 表示件数 | `<label>`+Select | 表示件数 | 横並び | `:187-190` |
| 在庫照会（`items-end` `:103`） | 検索 / 部門 / 表示件数 | SearchBar live / DepartmentFilter / `<label>`+Select | — | 上置き / 横並び / 横並び | `StockInquiryPage.tsx:104-131` |
| 一括価格改定（`items-end` `:47`） | 検索 / 取引先 / 部門 / 廃番を含む / 表示件数 / 取引先未設定を含める | SearchBar live / `<label>`+Select〈⑲ で trigger 化〉/ DepartmentFilter / Checkbox / `<label>`+Select / Checkbox | — | 上置き / 横並び / 横並び / 横並び(checkbox) / 横並び / 横並び(checkbox) | `PriceRevisionFilters.tsx:48-153` |
| 入出庫履歴 | 記録種別 / 期間 / 検索 / 記録ID / 部門 / 状態 / 表示件数 | Select / date / SearchBar / number / Select / Select / Select | — | **全部上置き** | `InventoryRecordsPage.tsx:161-296` |
| 操作ログ | 期間 / 種別 / 表示件数 | date / Select / Select | — | **全部上置き** | `OperationLogsPage.tsx:349-425` |
| 在庫変動 | 期間 / 種別 / 表示件数 | date / Select / Select | — | **全部上置き** | `StockMovementsPage.tsx:135-207` |
| 棚卸し（`items-center gap-4` `:746`） | 部門 / 未入力のみ表示 / 表示件数 | DepartmentFilter / Checkbox+Label / Label+Select | — | 横並び / 横並び(checkbox) / 横並び | `StocktakePage.tsx:747-789` |
| 整合性検証 | 表示件数 | `<label>`+Select（`flex items-center gap-2`） | 表示件数 | 横並び | `IntegrityCheckPage.tsx:232-251` |
| 入庫 / 手動販売 / 返品交換 / 廃棄 | 商品追加の検索（⑮ `ProductAddSuggest`、フィルタではない） | Label + Input | 商品追加 | 上置き（`space-y-2`） | `ReceivingPage.tsx:437-452` / `ManualSalePage.tsx:472-487` / `ReturnExchangePage.tsx:682-697` / `DisposalPage.tsx:371-386` |
| 棚卸し | 商品を検索・スキャン（カウント入力、フィルタではない） | Label + Input | — | 上置き（`space-y-2`） | `StocktakePage.tsx:561-568` |

### 規範の現在地

- catalog ⑨ `:636`「live 型は `div.grid.gap-1` 配下に可視 `Label` を上置きし `Input` を続ける。呼び出し側 toolbar は `items-end`」— **live SearchBar 限定**。`DepartmentFilter` の call-site block（`:625-633`、`:618-623` は live SearchBar）は横並びのまま。⑨ に page の file:line 引用は 0（`awk '/^## ⑨/,/^## ⑩/' | rg -c "\.tsx:[0-9]+"` = 0、この状態を維持する）
- catalog ⑤ SegmentedControl `:266-` に Label 規定なし
- memory / Backlog の「DSR-01 `:21`」は誤引用（DSR-01 は 1 画面 1 primary の規則で Label 配置の規定なし）
- 04-backbone 原則 7 `:23`「live + 検索ボタン併記」は catalog ⑨「ボタンなし」と drift（backbone 自身が「batch 1〜2 で改める」と記述、本 lane 非目的）

### 手書き見出し（PageHeader を通らない h2）の現状

Backlog `Plans.md:163` の行番号は drift 済み（内容で再特定）。**いずれも sub-section の h2 であり page title ではない**（各 page は別途 `PageHeader` の h1 を持つ）。`PageHeader` は h1 を描くため、これらを `PageHeader` に寄せる案は「1 ページ 1 h1」（catalog `:60`）に反する → 却下。

| site | 現行 | 形態 |
|---|---|---|
| `StocktakePage.tsx:385-407`（`StocktakeProgressHeader` 関数、Backlog「StocktakePage:386」） | `flex flex-wrap items-center justify-between gap-3` + 左 div〈h2 `text-xl`「棚卸し中…」+ p 進捗〉+ 右 Badge / action | **C**（説明が行の内側 + 右要素） |
| `MonthlySalesPage.tsx:155-163`（旧 `:82`） | `<section aria-labelledby>` + div〈h2 `text-lg` + p〉、flex 行なし | **B**（action なし） |
| `DailySalesPage.tsx:163-171`（旧 `:82`） | 同上 | **B** |
| `DisposalPage.tsx:659-667` | `flex flex-wrap items-center justify-between gap-2`〈h2 `text-lg`「直近の廃棄・破損」+ Button「すべての履歴を見る」〉、**p は行の外（下の sibling）** | **A**（説明が行の外） |
| `ReturnExchangePage.tsx:940-948` | 同型「直近の返品・交換」 | **A** |
| `ManualSalePage.tsx:706-714`（旧 `:699`） | 同型「直近の手動販売出庫」 | **A** |
| `ReceivingPage.tsx:658-666`（`:667` が行の外の `<p>`） | 同型「直近の入庫」 | **A** |
| `IntegrityCheckPage.tsx:325-334`（旧 `:300`） | `flex flex-wrap items-end justify-between gap-3` + 左 div〈h2 `text-xl`「差異のある商品」+ p〉+ Button | **C** |
| `IntegrityCheckPage.tsx:438`（旧 `:461`） | `AlertDialogTitle` | **見出しではない → 対象外** |

**3 形態の census（Plan Review round 1 Opus P1-2 で訂正）**: A 4 / B 2 / C 2。長い説明で右要素が次行左へ落ちる折返しバグ（⑮ Gated Amendment 2 と同型）を持ちうるのは **C の 2 箇所のみ**（説明が左 group の内側にあり右要素と同じ flex 行にいる）。A は説明が行の外にあるため折返し問題は無く、その配置は catalog ③ `**「直近の○○」系4画面の統一**`（base `:197`、⑮ で反映済み）で固定されている。B は flex 行を持たない。よって「8 箇所同型」は誤りで、component 化の根拠（rule of three）は C 2 箇所には成立しない。残る統一対象は **文型（h2 + 任意 p + 任意右要素）と token**: h2 token は `00-foundations.md:81` / catalog `:243` / `04-backbone.md:17` で **20px = `text-xl`** であり、A 4 箇所 + B 2 箇所の `text-lg`（18px = h3 token）は off-token、C 2 箇所の `text-xl` が正。catalog `:243`（④ フォームセクション）は h2 token のみで構造規定なし。

このほか右要素を持たない手書き `<h2` は 24 箇所（`HomePage.tsx:90,95,100` の 3 / 記録詳細 6 page の 13 / 4 記録画面の「○○を保存しました」「○○内容」8 = `ReceivingPage.tsx:324,377` / `DisposalPage.tsx:307,351` / `ManualSalePage.tsx:351,416` / `ReturnExchangePage.tsx:442,510`）、いずれも折返し問題の対象外（round 1 Sonnet P3 / round 2 Opus P2-8 で census 訂正）。`src/**` の `<h2` は計 36、うち off-token `text-lg` は 30（A 4 + B 2 + 上記 24）。`PluExportPage.tsx:584` は `text-xl` で対象外、`SidebarArea.tsx:19` は nav ラベル（`text-xs`）で対象外、`FormSection.tsx` は ④ canonical component で対象外。文型・token 統一の対象には含まれるため、runtime lane 申し送りで「h2 token の sweep 候補 24」として列挙するに留める。

## 設計判断（Coordinator adjudication、Plan Review / Human Gate で覆せる）

- **D1 規範の置き場 = catalog ⑨ の使用トークン段落を「すべてのフィルタ入力」に拡張**（DSR 新設なし）。draft literal:
  > **使用トークン**: commit 型は wrapper `min-w-[18rem] flex-1` + 要素間 `space-2`（8px）+ ラベル `text-muted-foreground`。**すべてのフィルタ入力**（検索 / Select / date / number / `DepartmentFilter` / 並び替え / 表示件数 / フィルタ toolbar 内の SegmentedControl）は `div.grid.gap-1` 配下に可視 label を上置きし、入力要素を続ける。可視 label は `<label className="text-sm text-muted-foreground" htmlFor={…}>`（weight 400、既存の上置きサイト〈入出庫履歴 / 操作ログ / 在庫変動〉と同じ見た目）。`Label` component を使う場合は `className="text-sm font-normal text-muted-foreground"` を渡して component 既定の `font-medium` を打ち消す（`cn()` の twMerge は同じ font-weight group の後勝ちで置換する。class を足すだけでは消えない）。呼び出し側 toolbar は `flex flex-wrap items-end gap-3` で入力欄の下辺を揃える（⑭ で live 型 SearchBar に導入した形を全入力へ拡張。owner 2026-09-08「検索ツールの場所の表記は揃えたい」）。例外 2 種: **Checkbox は label 内包の横並び**（`<label className="flex items-center gap-2 text-sm">` に `Checkbox` + 文言、文言は muted にしない、checkbox の慣行）を維持し、toolbar 内では `self-center` で行の縦中央に置く（上置き label を持たないため `items-end` の下辺揃えに加わらない。owner 確認 2026-09-11）。**tab / mode 切替として使う SegmentedControl（sales TabsHeader の日次/月次、monthly ModeTabs 等、⑤ 参照）は可視 Label を持たない**（`ariaLabel` 必須）。判定軸は選択肢の数ではなく文脈であり、フィルタ toolbar 内の SegmentedControl（廃番表示 / PLU表示 / 並び順）は `ariaLabel` の文言を上置き label として可視化する（廃番表示と PLU表示は隣接してどちらも「すべて」を持ち、label なしでは識別できない。並び順も同じ列にあるため揃える。owner 確認 2026-09-11 で確定。廃番表示 3 択 / PLU表示 5 択が SegmentedControl なのは DSR-02 drift〈2 択以外は Tabs〉であり、Tabs / Select 化しても同じ文脈軸を適用する）。SegmentedControl は `role="group"` の button 群で labelable 要素を持たないため、この label は `<span id={…} className="text-sm text-muted-foreground">` + `aria-labelledby` で紐付け、`htmlFor` は使わない。live 型 SearchBar は `Input`（`max-w-md` 維持）を続ける。`id` 未指定時は `useId()` でラベルとの対応を一意にし、`label` 未指定時は「商品を検索」。フィルタは `w-[11rem]`（商品一覧）等の固定幅を呼び出し側で指定。

  （上の blockquote が転記の全文。現行 `:636` の commit 型文・`max-w-md`・`useId()`・既定文言・固定幅の各句を残した拡張であり、置換ではない。⑨ **アクセシビリティ** 段落 `:647` の「live 型は可視 `<Label>`」は `<label>` 表記へ揃える〈文言・a11y 規定は不変〉）
- **D2 `DepartmentFilter` の構造 block（⑨ `:625-633`、call-site の props 説明 `:625-626` と `<DepartmentFilter …/>` 呼び出し例）は維持し、その直後に internal 構造 block を追加する**（置換しない。`:618-623` は live 型 SearchBar の block で触らない）。追加 block の draft:
  ```tsx
  // DepartmentFilter 内部（上置き Label、runtime 反映は後続 lane）: label は raw label 要素、htmlFor は SelectTrigger の id を指す
  <div className="grid gap-1">
    <label className="text-sm text-muted-foreground" htmlFor={triggerId}>部門</label>
    <Select …>
      <SelectTrigger id={triggerId} className={widthClass}>…</SelectTrigger>
      …
    </Select>
  </div>
  ```
  `id` は `SelectTrigger`（`<button>`、labelable）に付ける（現行 `DepartmentFilter.tsx:57-64` と同じ配線、`labelId` は未参照のため削除可）。挿入位置は `:633`（`/>`）の直後、`:634` の閉じ fence の**前**（同一 ```tsx fence 内）。component 1 箇所の改修で 4 サイトが揃うことを注記に書く（file:line は書かない）
- **D3 catalog ⑨ に page の file:line を書かない**（stale 化防止）。適用範囲の実測表は本 packet に置き、runtime lane packet が引き継ぐ。catalog には「一覧 / 記録画面の toolbar 全部」と書く
- **D4 セクション見出し = catalog ① の variation として新設**（「**バリエーション: セクション見出し（h2）**」、`:56` 使用トークン段落の直前）。draft literal（Plan Review round 1 Opus P1-1 / P1-2 で訂正）:
  > **バリエーション: セクション見出し（h2）**: page 内の sub-section（「直近の○○」テーブル / 公式部門集計 / 差異のある商品 / 棚卸し進捗 等）の見出しは h2（**`text-xl font-semibold`、④ / 00-foundations の h2 20px と同一 token**）+ 任意の右要素（Button / Badge）+ 任意の説明 `<p className="text-sm text-muted-foreground">` で構成する。配置は **見出し行 + 説明行** の 2 段: 見出し行は `flex flex-wrap items-start justify-between gap-3` に h2（`min-w-0 flex-1`。長い見出しは行内で折り返し、右要素を次行へ落とさない）と右要素（`shrink-0`）を置き、説明は見出し行の**下**に全幅で置く（外側 `space-y-1`）。説明を見出しと同じ flex 行の左 group に置いて右要素の横で折り返す形（旧 差異のある商品 / 棚卸し進捗）は採らない。`PageHeader` (c) も同じ 2 段配置（① 構造）であり、page 見出しと sub-section 見出しの規範は 1 本。「直近の○○」系 4 画面（③ テーブルの **「直近の○○」系4画面の統一** 段落の文型）は既にこの配置であり不変。**1 ページ 1 h1 は不変**であり、sub-section を `PageHeader` で描かない。Dialog の `AlertDialogTitle` は対象外。適用は class / 文型 / token の統一とし、component 化（`SectionHeader{title, description?, actions?}`）は現時点で採用しない（3 箇所目の独自要件が出た時点で再検討）

  （**Gated Amendment 2**、owner 2026-09-11 回答 (c)「ボタンによって説明が折り返すのはしっくりこない」: 旧 literal の折返し契約〈左 group `min-w-0 flex-1` で説明を折り返し右要素を右上に保つ〉を撤回し、見出し行 + 説明行の 2 段へ。⑮ Gated Amendment 2 の折返し契約は「右ボタンが次行左へ落ちる」ことへの対処だったが、owner はボタンの横で説明が折り返すことも望まない。2 段配置は両方を満たし、形態 A と同型になるため規範が 1 本になる）
  - Coordinator 決定 = **class / 文型 / token 統一のみ**（round 2 Opus P2-6 で Human Gate から格下げ。class 統一と component 化は描画結果が同じで mockup では見分けられず、owner に問う意味が無い）。理由: 右要素を持つ見出しは形態 C の 2 箇所で rule of three 未達、A 4 箇所へ component を当てると説明が行内へ移り「直近の○○」系 4 画面の見た目を変える（owner「壊さない優先」）。literal 内の同 doc 行番号参照は自分の挿入でずれるため見出し文字列で参照する（round 2 Opus P1-2）
- **D5 SegmentedControl の可視 Label の判定軸 = 文脈（フィルタ toolbar 内か、tab / mode 切替か）。フィルタ toolbar 内（廃番表示 / PLU表示 / 並び順）は上置き Label、tab / mode 切替（sales TabsHeader 日次/月次、monthly ModeTabs）は Label なし、を Coordinator 既定とし、Human Gate (1) で owner 確定（2026-09-11、提案 a）**（round 1 で「自己記述」を反転、round 2 Opus P2-7 で軸を選択肢数から文脈へ）。理由: `src/features/products/search.ts:15-26` の両 option set が「すべて」を持ち `ProductListPage.tsx:135-150` で隣接描画されるため label なしでは識別できない。「2 択は無し」にすると同じ列で並び順だけ Label が無く owner の「揃えたい」に反し、新規 SegmentedControl ごとに数を数える規則になる。tab 型に上置き Label を付けるのは誤りなので全面適用もしない。runtime lane で `SegmentedControl` に `label?` prop（`grid gap-1` + raw label）
- **D6 mockup-g（`docs/design-system/reference/mockup-g-filter-toolbar.html`）を新設**: state-1 = 商品一覧の **2 段 toolbar 全体**（原則 6 の枠 + 段間 `space-y-3`。1 段目 検索 / 部門 / 廃番表示 / PLU表示〈両 SegmentedControl を隣接させ「すべて」の重複を見せる〉、2 段目 並び替え / 並び順 / 表示件数）の現行〈横並び・Label なし〉/ 提案 a〈上置き、toolbar 内 SegmentedControl 全部に Label〉/ 提案 b〈上置き、並び順のみ Label なし〉の 3 列（Human Gate (1) を両案で提示。**Gated Amendment 2**: 回答後は h3 を「提案 a（確定）」「提案 b（不採用）」に改め、列は比較記録として残す）、state-2 = 棚卸し toolbar（部門 / 未入力のみ表示〈label 内包へ揃え、**Gated Amendment 2** で `align-self:center` の縦中央〉/ 表示件数）現行 / 提案、state-3 = 形態 C「差異のある商品」に長い説明 + Button で現行（`items-end`、右要素が落ちる）/ 提案（**Gated Amendment 2**: 見出し行 + 説明行の 2 段、説明は全幅。旧「左列で折返し」提案を置換）並置 + 形態 A「直近の入庫」は同配置で不変の注記、**state-4（Gated Amendment 2 で新設）= `PageHeader` (c) を商品 CSV 取込み（説明 154 字 + 右ボタン）で現行（左 group 内で折返し）/ 提案（見出し行 + 説明行）の並置**。CSS token は `mockup-d-lists.html` を流用、外部 CDN / JS なし。toolbar の枠は 04-backbone 原則 6 / `ListShell.tsx:207` の `rounded-lg border bg-card p-4` に合わせ `background:var(--card)`（白）とし、`var(--accent)` の灰色塗りにしない（**Gated Amendment 1**、Final Review Opus P2: 枠色の変更提案に見えると gate 回答が 1 往復増える）
- **D7 ⑲ との境界と wave 登録**: ⑲ は `PriceRevisionFilters` の取引先を trigger 化するが Label 配置は現行維持（⑲ Non-scope）。本規範の runtime lane が trigger の wrapper も `grid gap-1` へ揃える。両 lane が `02-component-catalog.md` を編集する（行範囲は互いに素、更新履歴表末尾のみ textual conflict）ため、DEV_WORKFLOW「同じ source document を編集する lane は同居させず」の例外を owner 2026-09-10「2 本まで並走可」を根拠に適用し、`Plans.md` Wave Registry に **wave 9** として両 lane を登録した（round 1 Opus P2）。merge train = 先に Ready になった方、後続が origin/main 単段 merge で両側保持
- **D8 `PageHeader` (c) の配置 = 見出し行 + 説明行（Gated Amendment 2、owner 2026-09-11 回答 (c) の適用範囲 = page 見出しにも）**: ① 構造 block と「バリエーション: 説明セクション」「component gap の解消」の記述を、外側 `<header className="space-y-1">` → 見出し行 `<div className="flex flex-wrap items-start justify-between gap-3">`（h1 `min-w-0 flex-1` + `shrink-0` の actions。**Gated Amendment 3**: 見出しに `min-w-0 flex-1` を付け、長い見出しでも右要素が次行左へ落ちない ⑮ の保証を 2 段配置でも無条件に保つ。構造 block の `{subtitle && …}` は runtime `PageHeader.tsx` と同じ `subtitle !== undefined && …` に揃え、段落末に「runtime 反映は後続 lane（現行 `PageHeader.tsx` は左 group 内折返し）」を 1 句置く）→ `subtitle` / `description` の `<p className="text-sm text-muted-foreground">` を見出し行の下に全幅、へ改める。canonical path（`src/components/patterns/PageHeader.tsx`）/ props（`title, subtitle?, description?, actions?`）/ `items-start`（h1 と actions の上端揃え）は不変。理由: 実測（2026-09-11、`rg -l "<PageHeader" src --glob "!*.test.tsx"`）で `PageHeader` 28 page 中 actions 持ち 15、うち subtitle / description なし 9 page は完全不変、subtitle 22〜34 字の 5 page（入庫 / 廃棄 / 手動販売 / 返品交換 / 取引先管理）は見出し行高が Button `h-9` 律速になり説明が 4px 下がるのみで reflow なし（form lock 中は actions が `null` で完全不変。Final Review round 3 Opus 実測で訂正）。変わるのは商品 CSV 取込み（説明 154 字）のみで、説明が actions の下まで伸びて折り返しが減る。owner は「折り返す page だけ個別に直す」より規範 1 本を選択（説明の長さで配置が変わる規範は保守者が迷う）。runtime 反映は `PageHeader.tsx` の actions 分岐 1 箇所（S6 申し送り）。mockup-g state-4 で現行 / 提案を並置し owner が見て確認済みの前提を runtime lane の L3 で再確認する

## Scope

- **S1 catalog ⑨ 改訂**（`02-component-catalog.md:602-660`）: 使用トークン段落を D1 literal へ / `DepartmentFilter` の call-site block `:625-633` の直後に D2 の internal 構造 block を**追加** / 使いどころに「toolbar の全フィルタ入力」を 1 句 / **状態** bullet に「Checkbox / SegmentedControl の例外」を残さず D1 の 1 段落に集約（重複記述しない）
- **S2 catalog ① 改訂**（`:24-56`）: D4 variation 新設（`:56` 使用トークンの直前）/ ③ の `**「直近の○○」系4画面の統一**` 段落（base `:197`、見出し文字列で位置決め）末尾に「見出しの token と右要素の扱いは ① セクション見出し variation に従う（説明は行の下のまま）」1 句 / **Gated Amendment 2（D8）**: ① 構造 block（`:33-45`）を見出し行 + 説明行の 2 段へ、「バリエーション: 説明セクション」と「component gap の解消」の説明位置の記述（左 group 内 / `space-y-1` グループ）を D8 に合わせて改稿。canonical path / props / `items-start` の不変を明記し、見出し行の class literal は variation と同一にする
- **S3 catalog ⑤ SegmentedControl**（`:266-`）: **使いどころ「アプリ内の二択切替」は不変**（**Gated Amendment 1**: 旧案の「二択・少数選択肢（3〜5）」拡張は DSR-02 `:33`「3 つ以上または内容が異質なら Tabs」と ⑤ Don't「3 つ以上の切替に流用しない」に正面衝突するため撤回。廃番表示 3 択 / PLU表示 5 択が SegmentedControl なのは既存の DSR-02 drift であり、本 lane の Label 規範は Tabs / Select 化しても同じ文脈軸で適用できる → drift の是正は runtime lane の申し送り + Backlog）。**アクセシビリティ** 段落（base `:322`）末尾に「tab / mode 切替として使う場合は可視 Label を持たない、フィルタ toolbar 内で使う場合は ⑨ の上置き Label 規範に従う（`ariaLabel` は常に必須）」を 1 文として追記（**状態** bullet 列には置かない。Final Review Opus P3）。Human Gate (1) の結果で確定文にする。回答前は draft literal のまま
- **S4 mockup-g 新設 + `reference/README.md` 一覧表に 1 行登録 + `:25`「mockup-c / mockup-d / mockup-e / mockup-f は同じ CSS（token）系統」の列挙に mockup-g を追加**（D6）/ **Gated Amendment 2**: state-1 h3 改題（確定 / 不採用）、state-2 提案の Checkbox に `align-self:center`、state-3 提案を 2 段配置へ置換（lead 文も更新）、state-4 新設（D6）
- **S5 更新履歴**: `02-component-catalog.md` 更新履歴表に 1 行（PR 番号は Draft PR 作成後に Writer が埋める、無ければ「本 PR」）
- **S6 runtime lane 申し送り（本 packet に記録、Writer は書かない）**: `src/components/patterns/DepartmentFilter.tsx:61-64` → `grid gap-1`（4 サイト一括、`labelId` 削除可）/ `src/features/products/ProductListPage.tsx:156-159`（並び替え）`:187-190`（表示件数）+ 2 段目 wrapper `:155` を `items-end` / `src/features/stock-inquiry/StockInquiryPage.tsx:129-131` / `src/features/products/components/PriceRevisionFilters.tsx`（表示件数 `:115-119`、取引先 trigger〈⑲ 後〉）/ `src/features/stocktake/StocktakePage.tsx:746-789`（toolbar `items-center gap-4` → `items-end gap-3`、表示件数上置き、**Checkbox `:756-770` は div + sibling Label から label 内包へ揃える**〈フィルタ toolbar 内の Checkbox は `PriceRevisionFilters.tsx:103,145` の 2 サイトが label 内包、棚卸しの 1 サイトのみ div + sibling。toolbar 外の `IntegrityCheckPage.tsx:399` / `PriceRevisionPage.tsx:65` は label 内包、`ProductForm.tsx:226` / `BackupRestorePage.tsx:415,594` は div + sibling、`ProductImportPreview.tsx:132` は `aria-label` のみで、いずれも toolbar 外のため本規範の対象外〉）/ `src/features/integrity-check/IntegrityCheckPage.tsx:232-251` / SegmentedControl に `label?` prop（labelable 要素が無いため `<span id>` + `aria-labelledby`、`htmlFor` 不可。Final Review Opus P3）、`ProductListPage.tsx:135-150,179-186` の 3 箇所に付与（Human Gate (1) 確定 = 全部付ける）/ **棚卸し Checkbox は label 内包 + `self-center`**（Gated Amendment 2、owner 回答 (b)）。**廃番表示 3 択 / PLU表示 5 択の SegmentedControl は DSR-02 drift**（2 択以外は Tabs）— Tabs / Select 化の要否は runtime lane packet で判断し、Backlog へ起票（closeout 時。**併せて起票**: mockup-g の `:has()` selector 使用〈参照 mockup 中 g のみ、動作影響なし、Final Review round 3 Opus P3-11〉/ catalog ① `:28` canonical 行の props に `description?` が欠ける〈⑮ 起源の pre-existing、runtime lane の S7 で 1 語追加、Final Review round 4 Opus N-1〉）/ **`src/components/patterns/SearchBar.tsx:72`（commit 型）`:178`（live 型）の `<Label className>` に `font-normal` を足す**（D1 の weight 400 canonical、live 4 サイト一括）/ 商品追加検索 5 箇所の `space-y-2` → `grid gap-1`（任意、⑮ ProductAddSuggest wrapper）/ セクション見出し: 形態 C 2 箇所（`IntegrityCheckPage.tsx:325-334` / `StocktakePage.tsx:385-407`）を見出し行 + 説明行の 2 段へ（説明 `<p>` を flex 行の外に出し外側 `space-y-1`、見出しに `min-w-0 flex-1`、Gated Amendment 2 D4 / 3。**`StocktakePage.tsx:385-407` の `StocktakeProgressHeader` は既存 `space-y-2` の内側に `space-y-1` を新設して h2 行 + p を包み、行は `items-center` → `items-start`、`<Progress>` は `space-y-2` 直下に残す**〈Final Review round 3 Opus P3-9。棚卸し進捗は mockup-g に無く owner 未視認のため runtime lane L3 の抜き取りに追加〉）/ **`PageHeader.tsx` は JSX（actions 分岐）と同時に JSDoc `:21-27` と inline comment `:30` の旧契約記述（「副題・説明は左グループ内で折り返す」）も更新する**/ **`src/components/patterns/PageHeader.tsx` の actions 分岐 1 箇所を 2 段配置へ**（Gated Amendment 2 D8。actions 持ち 15 page に波及するが描画差は商品 CSV 取込みのみ。runtime lane の L3 は商品 CSV 取込み + 短い説明の page 2 つの抜き取り）+ 形態 A / B 6 箇所の h2 `text-lg` → `text-xl`（+ 右要素なしの h2 24 箇所の token sweep 候補）。component 化は採用しない（D4 Coordinator 決定）

## Non-scope

- `src/**` / `src-tauri/**` 一切
- 04-backbone 原則 7 の drift（検索ボタン併記）
- DSR 新設、`01-decision-rules.md` の変更（本 lane は catalog + mockup のみ）
- `docs/SCREEN_DESIGN.md`（Label 配置の記述なし、Explore 実測）
- `docs/function-design/*`（各画面の doc は toolbar の Label 配置を規定していない。runtime lane で必要なら同期）
- `Plans.md` の Backlog 前提訂正は Coordinator が plan-first commit で行う（Writer は触らない）
- ⑲ lane の file（`01-decision-rules.md:25` / catalog `:578-598` picker 小節 / 51 / 61 / 77 / 78）

## Acceptance Criteria

docs-only のため rg oracle（出力空 = 0 件）。baseline は起票時実測。

- **AC1** `rg -c "すべてのフィルタ入力" docs/design-system/02-component-catalog.md` ≥ 1（baseline 0）
- **AC12**（Gated Amendment 1）`rg -c "少数選択肢" docs/design-system/02-component-catalog.md` = 0（`0d4ede74` 時点 1、⑤ 使いどころを二択に戻す）/ `rg -c "Human Gate \(3\) で確認" docs/design-system/02-component-catalog.md` = 0（`0d4ede74` 時点 0 → GA1 で ≥ 1 → **Gated Amendment 2 で反転**、`240b2c46` 時点 1。確定文化でマーカー撤去）/ `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c "aria-labelledby"` ≥ 1（`0d4ede74` 時点 0、SegmentedControl の label 紐付け）/ `awk '/^## ⑤/,/^## ⑥/' docs/design-system/02-component-catalog.md | rg -c "^\*\*アクセシビリティ\*\*.*可視 Label を持たない"` = 1（`0d4ede74` 時点 0）/ `awk '/^## ⑤/,/^## ⑥/' docs/design-system/02-component-catalog.md | rg -c "^- tab / mode 切替"` = 0（`0d4ede74` 時点 1、状態 bullet 列から撤去）/ `rg -n "^\.toolbar\{" docs/design-system/reference/mockup-g-filter-toolbar.html | rg -c "background:var\(--card\)"` = 1（`0d4ede74` 時点 0 = `var(--accent)`）
- **AC2** `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c 'className="grid gap-1"'` ≥ 1（baseline 0、D2 の DepartmentFilter block）
- **AC3** `rg -c "label 内包の横並び" docs/design-system/02-component-catalog.md` ≥ 1（baseline 0）/ `rg -c "可視 Label を持たない" docs/design-system/02-component-catalog.md` ≥ 2（baseline 0、⑨ D1 + ⑤ S3）/ `rg -c "フィルタ toolbar 内の SegmentedControl" docs/design-system/02-component-catalog.md` ≥ 1（baseline 0、D5 文脈軸）/ `rg -c 'text-sm text-muted-foreground" htmlFor' docs/design-system/02-component-catalog.md` ≥ 1（baseline 0、D1 の label canonical class + D2 block）/ `rg -c "font-normal" docs/design-system/02-component-catalog.md` ≥ 1（baseline 0、D1 の `Label` 打ち消し）/ `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c "commit 型は wrapper"` = 1（baseline 1、拡張で消えていない）
- **AC4** `rg -c "バリエーション: セクション見出し" docs/design-system/02-component-catalog.md` = 1（baseline 0）/ `rg -c "SectionHeader" docs/design-system/02-component-catalog.md` ≥ 1（baseline 0）/ `awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "min-w-0 flex-1"` = 3（baseline 2 = `:34` code block + `:52` 本文。round 1 で ≥ 3 → Gated Amendment 2 で ≤ 1 → **Gated Amendment 3 で = 3**: 構造 block の h1 + variation の h2 + `:54` の ⑮ 経緯記述）
- **AC11**（token、負の oracle）`awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "text-lg"` = 0（baseline 0、維持）/ `awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "text-xl font-semibold"` ≥ 1（① 節限定 baseline 0。file 全体では `:226` ④ FormSection block が既に 1 件あるため awk 必須）/ `rg -c "見出し行 \+ 説明行" docs/design-system/02-component-catalog.md` ≥ 2（baseline 0。**Gated Amendment 2** で旧 oracle「説明を見出し行の下に置く形」≥ 1 を置換。variation + ① (c) 構造の記述の 2 箇所）/ `awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c ":197"` = 0（baseline 0、literal 内に同 doc 行番号を書かない）
- **AC5**（負の oracle、D3）`awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c "\.tsx:[0-9]+"` = 0（baseline 0、維持）
- **AC6**（負の oracle）`rg -c "1 ページ 1 個の h1" docs/design-system/02-component-catalog.md` = 1（baseline 1、不変）/ `rg -n "sub-section を .PageHeader. で描かない|PageHeader で描かない" docs/design-system/02-component-catalog.md | wc -l` ≥ 1
- **AC7** `fd -g "mockup-g-filter-toolbar.html" docs/design-system/reference | wc -l` = 1（baseline 0）/ `rg -c "mockup-g" docs/design-system/reference/README.md` ≥ 1（baseline 0）/ `rg -c "<script|https?://" docs/design-system/reference/mockup-g-filter-toolbar.html` = 0
- **AC8** `awk '/^## 更新履歴/,0' docs/design-system/02-component-catalog.md | rg -c "セクション見出し"` ≥ 1（baseline 0）
- **AC9** `git diff --name-only origin/main..HEAD -- src src-tauri docs/design-system/01-decision-rules.md docs/function-design | wc -l` = 0
- **AC10** `bash scripts/doc-consistency-check.sh` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS / `bash scripts/tests/reading-order-drift.test.sh` PASS / `npm run format:check` PASS（docs は `.prettierignore` 除外、既定どおり）
- **AC13**（Gated Amendment 2、Human Gate 回答の確定文化。baseline は `240b2c46` 時点）`awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c "Human Gate"` = 0（時点 1、(1)(3) マーカー撤去）/ `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c "DSR-02 drift"` ≥ 1（時点 0、Final Review round 2 Opus N-1 の但し書き）/ `awk '/^## ⑨/,/^## ⑩/' docs/design-system/02-component-catalog.md | rg -c "self-center"` ≥ 1（時点 0、Checkbox 縦中央）/ `awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "折返し契約"` = 0（時点 1）/ `awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "説明行"` ≥ 2（時点 0、(c) 構造 + variation）/ `rg -c "align-self:center" docs/design-system/reference/mockup-g-filter-toolbar.html` ≥ 1（時点 0、state-2 提案の Checkbox）/ `rg -c 'id="state-4"' docs/design-system/reference/mockup-g-filter-toolbar.html` = 1（時点 0）/ `rg -c "不採用" docs/design-system/reference/mockup-g-filter-toolbar.html` ≥ 1（時点 0、state-1 提案 b の h3）/ `rg -c "説明行" docs/design-system/reference/mockup-g-filter-toolbar.html` ≥ 1（時点 0、state-3 提案 h3）
- **AC14**（Gated Amendment 3、Final Review round 3 Opus P2-3 / P2-4 / P2-5 / P3-10 の是正 oracle。baseline は `25d4e456` 時点）`awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "runtime 反映は後続 lane"` ≥ 1（時点 0、① が docs-only であることの明示）/ `awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "subtitle !== undefined"` ≥ 1（時点 0、runtime と同じ条件式）/ `awk '/^## ①/,/^## ②/' docs/design-system/02-component-catalog.md | rg -c "Human Gate"` = 0（時点 0、維持）/ `rg -c "owner 確定（2026-09-11）" docs/design-system/reference/README.md` ≥ 1（時点 0、mockup-f 行の先例に倣う）/ `rg -c "PageHeader" docs/design-system/reference/README.md` ≥ 1（時点 0、mockup-g 行の内容欄に state-4）/ `rg -c "Human Gate で確認" docs/design-system/reference/README.md` = 0（時点 1）/ `rg -c "heading-title" docs/design-system/reference/mockup-g-filter-toolbar.html` ≥ 3（時点 0、CSS 定義 `.heading-title{min-width:0;flex:1}` + state-3 / state-4 提案 panel の h2 / h1）
- **AC-HumanGate** owner が mockup-g を実機 3 画面（商品一覧 / 棚卸し / 整合性検証の差異一覧）と並べ、culling (1)(3) に回答。回答は Gated Amendment で catalog の draft literal を確定文へ → **回答済み（owner 2026-09-11、Workflow State の Human Gate 欄に原文要旨）**。確定文化は Gated Amendment 2 + Codex 是正発注 41、oracle は AC13

## Design Sources

- Requirements / spec: owner 2026-09-08「検索ツールの場所の表記は揃えたい、今回は商品を検索だけ」（Plans.md `:167`）、⑮ Writer 実測の手書き header 9 箇所（Plans.md `:163`）
- Architecture: 該当なし
- Function / command / DTO: 該当なし
- DB: 該当なし
- Screen / UI: `docs/design-system/02-component-catalog.md` ① / ⑤ / ⑨ / `:197`、`04-backbone.md` 原則 6 / 7、`reference/mockup-d-lists.html`（token 流用元）、`PageHeader.tsx:31-43`（現行の左 group 内折返し実装。runtime lane で 2 段へ）
- Decision log / ADR: 追加なし

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 該当なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 該当なし | existing sufficient |
| DB / transaction / audit / rollback / migration | 該当なし | existing sufficient |
| Screen / UI / route state / Japanese wording | catalog ① / ⑤ / ⑨、mockup-g | updated in this PR |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | 該当なし（catalog の構造規約で足りる） | intentionally deferred（DSR 化しない） |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| mockup-g（reference doc 新設） | `docs/design-system/reference/README.md` 一覧表へ登録（S4）。他の生成義務は該当なし |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| owner 2026-09-08 揃えたい | catalog ⑨ `:636` | D1 / D2 / D3 | 全入力へ拡張、file:line は書かない。却下 = DSR 新設 | S1 | AC1〜3 / AC5 |
| ⑮ PageHeader (c) 折返し契約 | catalog ① `:48-52` | D4 / D8 | 3 形態 A4 / B2 / C2、Gated Amendment 2 で C 2 箇所と `PageHeader` (c) を見出し行 + 説明行の 2 段へ（折返し契約は撤回）、1 h1 不変。却下 = PageHeader 流用 / component 化（rule of three 未達） | S2 | AC4 / AC6 / AC11 |
| SegmentedControl の Label | catalog ⑤ / ⑨ | D5 | 判定軸 = 文脈。フィルタ toolbar 内は上置き Label（隣接する「すべて」の識別 + 同列で揃える）、tab / mode 切替は Label なし。却下 = 選択肢数で分ける（並び順だけ不揃い）/ 全面 Label（tab に不適） | S1 / S3 | AC3 |
| mockup 運用 | reference/README | D6 | 静的 HTML、token 流用 | S4 | AC7 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: S1 / S2 後は yes（規範 + 例外 + 根拠が catalog にある）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D1 / D4 / D5 → catalog
- Assumptions and constraints: Checkbox 横並びは慣行（owner 回答 (b) で横並び + 行の縦中央）
- Deferred design gaps, risk, and follow-up target: backbone 原則 7 drift（既存宿題）、runtime lane（S6）
- Test Design Matrix can cite design decision IDs or source doc sections: yes
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 例外 2 種（Checkbox / SegmentedControl）を D1 に明文化、他に例外なし

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable | — |
| Fact check / design decision split | Backlog の前提（操作ログ等未統一）は事実誤り → 本 packet で訂正、Plans.md も plan-first commit で訂正 | Plans.md |
| Lifecycle / retry | not applicable | — |
| Operator workflow | Label 上置きで toolbar が 1 行分高くなる（既に入出庫履歴 / 操作ログで実装済みの形、owner 既視） | AC-HumanGate |
| Replacement path | not applicable | — |
| Data safety / evidence | not applicable | — |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | mockup-g を実機と並べる | AC-HumanGate |
| 環境・再現性 | not applicable | — |

## Design Readiness

- Existing design docs are sufficient because: 規範の形は ⑭ / ⑮ で実装・記述済み（live SearchBar / PageHeader (c)）。本 lane はその適用範囲を広げる記述
- Source docs updated in this PR: catalog ① / ⑤ / ⑨ / 更新履歴、reference/README、mockup-g
- Design gaps intentionally deferred: backbone 原則 7、DSR 化
- Durable decisions discovered in this plan and promoted to source docs: D1 / D4 / D5

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): docs のみ
- Backend function design: 該当なし
- Command / DTO / data contract: 該当なし
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: Label 文言は現行のまま（部門 / 並び替え / 表示件数 / 取引先）
- Error, empty, retry, and recovery behavior: 該当なし
- Testability and traceability IDs: AC1〜AC11 + AC-HumanGate

## Contract Probe

N/A。外部前提なし（docs-only）。

## Contract Coverage Ledger

R2 のため簡略。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D1 全フィルタ入力の上置き規範 + 例外 | S1 | AC1 / AC3 | AC-HumanGate |
| D2 DepartmentFilter 構造 block | S1 | AC2 | non-scope（runtime lane） |
| D3 file:line を書かない | S1 | AC5 | — |
| D4 セクション見出し variation | S2 | AC4 / AC6 / AC11 | AC-HumanGate（state-3 / state-4 で 2 段配置の見た目を確認、回答済み。component 化は Coordinator 決定で不採用、gate 項目なし） |
| D5 SegmentedControl 例外 | S3 | AC3 | AC-HumanGate (1) |
| D6 mockup-g | S4 | AC7 | AC-HumanGate |

## Test Plan

Test Design Matrix: [test-matrices/2026-09-10-filter-label-top-design.md](test-matrices/2026-09-10-filter-label-top-design.md)

- targeted tests: AC1〜AC11 の rg / awk / fd oracle
- negative tests: AC5 / AC6 / AC9
- compatibility checks: 既存 ⑨ の live 型 SearchBar 記述（`max-w-md` / `useId()` / 既定 label）が残る（`rg -c "useId" catalog` 不変）
- data safety checks: 該当なし
- main wiring/integration checks: 該当なし

## Boundary / Wire Contract

該当なし（docs-only）。

## Review Focus

- D1 の段落が既存の live 型 SearchBar 記述を**置換でなく拡張**していること（`max-w-md` / `useId()` / 既定「商品を検索」が残る）
- catalog ⑨ に page の file:line が入っていないこと（AC5）
- ① variation が「1 ページ 1 h1」と矛盾しないこと、`PageHeader` の (c) 契約の literal（`items-start` / `min-w-0 flex-1` / `shrink-0`）と一致していること
- Human Gate 回答（(a)(b)(c)）が確定文化され、⑨ / ① に `Human Gate` の draft マーカーが残っていないこと（AC13。Gated Amendment 2 で「未回答のまま」から反転）
- mockup-g が `mockup-d-lists.html` の token を流用し、新規 CSS 系統・外部 CDN・JS を持たないこと
- 更新履歴 1 行のみで、他 section の文言を触っていないこと

## Spec Contract

R2 のため簡略。Contract ID: SPEC-FILTERLABEL-D-1 — catalog ⑨ / ① / ⑤ に上置き規範・例外・セクション見出し variation が draft literal どおり転記され、mockup-g が登録される。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-FILTERLABEL-D-1 | S1 | AC1 / AC2 / AC3 / AC5 | 拡張であり置換でない | rg |
| SPEC-FILTERLABEL-D-1 | S2 | AC4 / AC6 | 1 h1 不変 | rg |
| SPEC-FILTERLABEL-D-1 | S3 | AC3 | draft literal | rg |
| SPEC-FILTERLABEL-D-1 | S4 | AC7 | token 流用 | fd / rg |
| SPEC-FILTERLABEL-D-1 | S5 | AC8 | 1 行のみ | awk |

## Data Safety

- what must not be committed: なし
- local-only paths: `.local/codex-orders/**`、`.local/preview/**`（Human Gate 用の写し）
- synthetic-only paths: 該当なし

## Writer Instructions

- Codex `model_reasoning_effort=medium`。docs-only、編集してよい file = `02-component-catalog.md` / `reference/README.md` / `reference/mockup-g-filter-toolbar.html`（新規）の 3 file のみ。他に diff が出たら push せず停止
- 着手前: `git status --short` 空 → `git checkout --detach <遷移 commit SHA>` → `rev-parse --short HEAD` を報告。packet / Matrix / Plans.md / PR body は編集しない
- 各 S の転記元は本 packet「設計判断」の draft literal（blockquote 内の全文）。**再導出・言い換えしない**。blockquote の外の括弧書きは Coordinator の注記であり転記しない。Human Gate 2 件は draft literal のまま（確定文にしない）
- 行番号は base `bb081e3` 時点の値。各 S は見出し文字列 / 引用文字列で位置決めし、先行編集（S2 の ① 挿入で ⑤ / ⑨ / `:197` 以降がずれる）の shift を行番号で追わない
- docs は `.prettierignore` 除外のため `format:check` は既定どおり実行して PASS を報告すればよい（前提不一致ではない）
- mockup-g は単一 HTML + inline CSS、`mockup-d-lists.html` の `:root` 変数と `.tbl` / `.field` 系 class を複製。外部 CDN / JS なし
- 完了後に worktree を自分で detach する。`git add` は明示 path、`HEAD:branch` push 禁止
- ponytail block（verbatim）:

```
### 実装原則（ponytail、full）
書く前に順に問う: (1) そもそも要るか（推測需要は書かず 1 行で理由）(2) この codebase に既に helper / util / pattern があるか（再実装しない）(3) 標準ライブラリで済むか (4) platform の素の機能で済むか (5) 導入済み依存で済むか (6) 1 行で済むか (7) それでも要るなら動く最小 code。
規則: 実装 1 つの interface / 製品 1 つの factory / 変わらない値の config を作らない。将来用の scaffold を作らない。追加より削除、賢さより退屈さ。
例外: 正しさ・データ安全・既存 test の契約・packet の AC を削る方向には使わない。
```

## Implementation Results

Codex Writer（medium、2026-09-10〜11、発注書 37 → 是正 38 / 41 / 43）: catalog ⑨ の使用トークン段落を「すべてのフィルタ入力」の上置き Label 規範へ拡張（Checkbox は label 内包 + 縦中央、tab / mode 切替の SegmentedControl は Label なし、toolbar 内は `<span id>` + `aria-labelledby`）、`DepartmentFilter` の internal 構造 block を追加、① にセクション見出し variation（見出し行 + 説明行の 2 段、見出し `min-w-0 flex-1`）を新設し `PageHeader` (c) も同型へ改稿（runtime 反映は後続 lane）、⑤ アクセシビリティに 1 文、mockup-g（state-1〜4）と reference README を追加。owner Human Gate 3 件は mockup-g で回答・確定文化。PR: https://github.com/higekiri-w/inventory-system-desktop/pull/51

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Fill after review.
If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
- Findings Freeze: frozen after Plan Review round 3（2026-09-10、Sonnet approve-with-P2 + Opus approve-with-P2、P1 = 0、P2 は本 commit で全件是正）; post-freeze exceptions: none.

### Plan Review round 1（2026-09-10、plan-first `bde0478`、Sonnet + Opus 独立）

- Opus: reject（P1 2 / P2 7 / P3 4）。Sonnet: reject（P1 2 / P2 3 / P3 3、うち AC4 baseline と `StocktakeProgressHeader` は Opus と重複）
- Coordinator 実証: h2 token = 20px（`00-foundations.md:81` / catalog `:243`）→ `text-xl` / 「直近の○○」4 箇所は `<p>` が flex 行の外（`ReceivingPage.tsx:658-667` 等）→ 折返しバグ非該当、3 形態 A4 / B2 / C2 / `search.ts:15-26` で廃番表示・PLU表示とも「すべて」を持ち `ProductListPage.tsx:135-150` で隣接 → D5 反転 / AC4 baseline 2 / `DepartmentFilter` block は `:625-633` / `label.tsx:11` `font-medium`
- 裁定: **accept 19**（Opus P1-1 / P1-2 / P2 全 7 / P3 全 4、Sonnet P1-2〈4 箇所の h2 行番号〉/ P2-1 商品追加 / P2-3 ProductListPage 行番号 / P3 全 3）/ no-action 0。Opus P2「⑲ が存在しない」は branch 視点の artifact（⑲ は PR #50、別 branch）だが Wave Registry 未登録の指摘は正当 → wave 9 を両 branch に登録
- 是正 = `58602ce` + `2a73a0a` + `471cac1`（Human Gate 文言 / 実測表 2 つ / 3 形態 census / D1 / D2 / D4 / D5 / D6 / D7 / S1〜S4 / S6 / AC3 / AC4 / AC11 / Matrix / Plans.md wave 9）。PR body の `:461` は `:438` へ更新。round 2 = Sonnet + Opus 再注入

### Plan Review round 2（2026-09-10、`471cac1`、Sonnet + Opus 独立）

- Opus: reject（P1 2 / P2 9 / P3 7）。Sonnet: approve-with-P2（P2 2 / P3 4、Design Intent Trace の stale 2 行は Opus P2-3 と重複）
- Coordinator 実証: `label.tsx:11` `font-medium` は twMerge で `text-sm text-muted-foreground` を足しても残る（同 group の `font-normal` が必要）/ D4 literal の `:197` は S2 の挿入で自分がずらす / `:636` の commit 型文が D1 で消える / Design Intent Trace `:219-220` と Matrix C1・Adjacent Pattern Audit が round 1 以前の文言のまま / `<h2` census は 36 中 off-token 30、右要素なし 24（16 は誤り）
- 裁定: **accept 24**（Opus P1-1 / P1-2 / P2-1〜9 / P3-1〜7、Sonnet P2 2 / P3 4）/ no-action 0。P2-6（Human Gate (2) 格下げ）と P2-7（D5 の判定軸を選択肢数 → 文脈）は Coordinator 判断として採用（理由は D4 / D5 本文）
- 是正 = `7d848eb` + `658881e` + `83ec131`（Human Gate 2 件化 / Goal / 実測表 2 行 + census / D1 全文 blockquote + `font-normal` + `:647` / D2 挿入位置 / D4 見出し参照 + component 化不採用 / D5 文脈軸 / D6 state-1 3 列 / S3 / S6〈SearchBar `font-normal`・Stocktake `items-center`・Checkbox census〉/ AC3 / AC11 / AC-HumanGate / Design Intent Trace / Writer Instructions 2 項 / Matrix C1・C8・Adjacent Pattern Audit / Plans.md `:461` → `:438`）。round 3 = closure（Sonnet + Opus、diff 限定）

### Plan Review round 3 closure（2026-09-10、`83ec131`、Sonnet + Opus 独立）

- Sonnet: approve-with-P2（P2 3 = Review Focus「3 件」/ Ledger D4 行の `AC-HumanGate (2)` / PR #51 body 上部の旧文言、P3 1 = AC 範囲の略記）。Opus: approve-with-P2（closure 1〜9 全 closed、D1 literal は `:636` の 5 文すべて生存で脱落 0、h2 census 36 / 30 / 5 / 1 を機械照合。P2 2 = D4 literal の「本節の」は ① に挿入されるが参照先は ③ `:197` / 未 commit の是正 5 箇所、P3 2 = 見出し文字列の空白ゆれ / round 2 記録の SHA）。両報告の全文は PR #51 body「Plan Review round 3」節に転記（`gh pr view 51 --json body`）
- 裁定: accept 8 / no-action 0。すべて本 commit で是正（「③ テーブルの」/ S2・census の見出し文字列を実見出し `系4画面` に統一 / Review Focus・Ledger・Testability・Test Plan・Matrix の 2 件化と AC1〜AC11 / `ProductImportPreview.tsx:132` は `aria-label` のみ / round 2 記録の SHA / PR body 上部）
- **Findings Freeze**（round 3 / 天井 3）。次 = Plan Commit 記入 + state-only 遷移 `plan-draft->plan-gate->plan-approved->implementing` → Codex 発注書 37

### Final Review round 1（2026-09-10 夜、content `0d4ede74`、Sonnet + Opus 独立）

- Codex 37（session `01a08ac1-1f5d-7d72-804b-941506465c53`、medium、commands 43 / 手戻り 3、commit `0d4ede74`、3 file のみ、AC1〜AC11 全 PASS、docs gate 全 PASS、`local-ci.sh full` は Writer 側の `npm ci` 手順違いで未走 → Coordinator 側で Ready 前に 1 run）。全文報告は `/tmp/codex20-report.md`（Coordinator 保管）
- Sonnet: approve（AC1〜AC11 独立再実行で全 PASS、転記 byte 一致、mockup `:root` byte 一致、state-1 3 列、docs gate PASS、仮想 mutant で AC3 が落ちることを確認。P3 = Implementation Results 未記入〈Coordinator が state 遷移で記入〉）
- Opus: reject（P1 1 / P2 1 / P3 4）。**P1 = S3 の使いどころ拡張「二択・少数選択肢（3〜5）」が DSR-02 `:33` と ⑤ Don't `:330` に正面衝突**（Coordinator の packet 起因、Writer 非過失）。P2 = mockup `.toolbar` が `var(--accent)` 灰色塗りで backbone 原則 6 の `bg-card` と不一致。P3 = Checkbox 句に Human Gate (3) の明記なし / 新 bullet が状態列に混在 / SegmentedControl は labelable 要素を持たず `htmlFor` 不可（`aria-labelledby`）/ Matrix F8 の stale 文言。転記忠実性は byte 一致（D1 2199B / D4 1426B）、⑧ picker 小節の footprint 非侵犯を確認。両報告の全文は PR #51 body「Final Review round 1」節に転記（`gh pr view 51 --json body`）
- Coordinator 実証: DSR-02 `:33`「2 択の切替は SegmentedControl、3 つ以上は Tabs」/ ⑤ `:330` Don't / mockup `:38` `.toolbar{…background:var(--accent)…}` / `ProductListPage.tsx:135,143` の 3 択・5 択 SegmentedControl は既存 drift → **accept 6 / no-action 0**
- **Gated Amendment 1**（本 commit）: S3 = 使いどころ不変 + 文を **アクセシビリティ** 段落へ / D1 literal に Checkbox 句の「Human Gate (3) で確認」と SegmentedControl の `<span id>` + `aria-labelledby` 1 文 / D6 mockup `.toolbar` = `var(--card)` / S6 申し送り（`aria-labelledby`、DSR-02 drift の Backlog）/ AC12（oracle 6 本）/ Matrix F8。Codex 是正発注 38 → closure（Sonnet + Opus、Opus 必須）+ Codex review 39 → owner Human Gate（mockup-g の写しを `.local/preview/` に置く）

### Final Review round 2 closure + Human Gate 回答（2026-09-11、content `240b2c46`、Sonnet + Opus 独立）

- Codex 38（是正 `240b2c46`、catalog + mockup-g、AC1〜AC12 全 PASS、F3 は blockquote と byte 一致、commands 27 / 手戻り 0）。全文報告は `.local/codex-orders/log-38-fix-pr51-r1.txt` 末尾
- Sonnet: approve（AC1〜AC12 独立再実行で全 PASS〈AC12 は 6 oracle 個別〉、round 1 findings 6/6 解消、D1 2463B / D4 1426B が packet と byte 一致、mockup `:root` 631B が mockup-d と一致、footprint 5 file のみ・⑧ 小節非接触、仮想 mutant 3 本で AC12 oracle が落ちることを確認。`format:check` は node_modules 無しで未実行 → Coordinator の L1 full で充足）
- Opus: approve（P1 0 / P2 0 / P3 5 = Freeze 後の新規 follow-up）。round 1 6/6 解消・新矛盾なし。N-1 = ⑨ の廃番表示 / PLU表示 名指しが DSR-02 drift の追認に見える / N-2 = ⑤ 側 1 文に Human Gate マーカーなし / N-3 = ⑨ a11y「可視 `<label>`」の raw 誤読 / N-4 = Checkbox 例外の toolbar 内外境界 / N-5 = Human Gate (3) の対案 panel なし。canonical 突合（DSR-23 / ① 1 h1 / PageHeader (c) literal 逐語一致 / D1 が置換でなく拡張）は DSR-02（N-1）以外すべて一致
- Coordinator 裁定: N-1 accept（本 Gated Amendment 2 に但し書き同乗）/ N-5 accept（(3) は既定の追認として owner に提示）/ N-2・N-3・N-4 no-action（回答で確定文化 / D1 両形許容 / 段落冒頭で限定）
- L1 full（Coordinator、worktree `/tmp/rev-20`、HEAD `240b2c46`、tree CLEAN）: RESULT=PASS、`MERGE_EVIDENCE_VALID=true`、非 0 は `npm-audit` WARN_ONLY のみ。log `.local/codex-orders/reports/l1-full-pr51-240b2c46.log`
- **Human Gate 回答（owner 2026-09-11、mockup-g の写しを `.local/preview/` で確認）**: (1) 提案 a / (2) 棚卸しは提案 + Checkbox は行の縦中央 / (3) 形態 C は見出し行 + 説明行の 2 段（「差異ある商品と補正を確定のボタン配置を同じ行にするのって可能？説明文がここで折り返すのがあんましっくりこない」）。「他の画面も」の範囲を Coordinator が実測（`PageHeader` 28 / actions 15 / 折返し 1 page）して規範 1 本（`PageHeader` (c) にも適用）を提案、owner「それでやってみよう」
- **Gated Amendment 2**（本 commit）: Human Gate 欄に回答 / D1 literal を確定文（Checkbox `self-center`、SegmentedControl 全部 Label + DSR-02 drift 但し書き〈N-1〉、マーカー撤去）/ D4 literal を見出し行 + 説明行へ改稿（折返し契約撤回）/ D5 確定 / D6 state-1 改題・state-2 `align-self:center`・state-3 置換・state-4 新設 / **D8 新設**（`PageHeader` (c) の 2 段配置）/ S2・S4・S6 追記 / AC4・AC11・AC12 の oracle 反転 / **AC13**（9 oracle）/ Matrix F4・F8 改訂・F9 新設・仮想 mutant 2 本追加。Codex 是正発注 41 → closure round 3（Sonnet + Opus、Opus 必須）+ Codex review 39 の findings 反映 → state-only → Ready（docs-only は owner dispatch）
- **Gated Amendment 2 訂正（2026-09-11、Codex 41 fail-closed 停止起源）**: D1 literal に残した「owner 2026-09-11 Human Gate (1) / (2)」の語が AC13「⑨ 節の `Human Gate` = 0」と矛盾（Codex 41 が編集前に停止、commands 23）。canonical に gate 用語を残す理由が無いため literal 側を「owner 確認 2026-09-11」へ改め、AC13 は不変。発注書 41 の起点を本訂正後の tip に更新して再発注

### Final Review round 3 closure（2026-09-11、content `25d4e456`、Sonnet + Opus 独立）

- Codex 41（是正 r2、起点 `9cd547b9`、`25d4e456`、commands 30 / 手戻り 2。2 回の fail-closed 停止は Coordinator 起因: ① D1 literal の gate 用語 × AC13 → 訂正 `4c4f28bd` / ② 停止残 worktree と local branch の起点不一致 → Coordinator が付け替え）
- L1 full（Coordinator、`/tmp/rev-20`、HEAD `25d4e456`、CLEAN）: RESULT=PASS、`MERGE_EVIDENCE_VALID=true`、log `.local/codex-orders/reports/l1-full-pr51-25d4e456.log`
- Sonnet: approve（AC1〜AC13 全 PASS、D1 2,791B / D4 1,441B byte 一致、D8 意味一致、Human Gate 3 回答の反映確認、state-4 が `ProductImportPage.tsx` の実文言、`:root` 631B 一致、footprint 5 file・⑧ 非接触、仮想 mutant 2 本で AC13 落下）。P2 = README `:21` の「Human Gate で確認」残存 / packet Review Focus の反転漏れ、P3 = Goal の折返し契約残存 / Matrix C4・F4 の期待値未更新 / README の state-4 未言及
- Opus: reject（P1 1 / P2 4 / P3 5。**canonical の設計は妥当**、P1 は packet 側）。P1 = Goal `:54` が撤回済みの折返し契約を完了条件に残し AC13 と正面衝突 / P2 = packet 4 箇所の旧前提（Review Focus / Design Sources / Design Intent Trace / Ledger）、README `:21` stale（確定表現 + state-4 欠落）、catalog ① に「runtime 未反映」の明示がない、**見出し行の h1 / h2 に `min-w-0 flex-1` が無く長い見出しで右要素が次行左へ落ちる保証欠落**（⑮ の bug 形状。最悪ケース `StocktakeProgressHeader` で余裕約 35px）/ P3 = D8 実測の内訳不正確（説明なし 9 + subtitle 5 + CSV 1）、subtitle 5 page で説明が 4px 下がる、Human Gate 欄の設問 / 回答番号衝突、S6 の `StocktakeProgressHeader` 適用と JSDoc 更新漏れ、構造 block の `&&` と runtime `!== undefined` の差、mockup の `:has()` 新規使用。A(i)〜(iv) の設計妥当性はすべて成立、N-1〜N-5 closed
- Coordinator 裁定: **packet 側（P1 / P2-2 / P3-6・7・8・9）= 本 commit で是正**（Goal / Review Focus / Design Sources / Design Intent Trace / Ledger / D8 実測 / 回答 (a)(b)(c) 改番 / S6 補強）/ **canonical 側（P2-3 README / P2-4 runtime marker / P2-5 `min-w-0 flex-1` / P3-10 `!== undefined`）= Gated Amendment 3 + Codex 是正発注 43**（P2-5 は描画結果が owner の見た mockup と不変なので Human Gate 再訪不要、D4 / D8 literal に同乗）/ P3-11（`:has()`）= Backlog（動作影響なし）

### Gated Amendment 3（2026-09-11、Final Review round 3 起源）

- 是正（本 commit）: Goal を 2 段配置へ / D4 literal の h2 と D8 の h1 に `min-w-0 flex-1` / D8 に `subtitle !== undefined` と「runtime 反映は後続 lane」の 1 句 / D8 実測内訳の訂正 / Human Gate 回答を (a)(b)(c) に改番（S6 / D8 / 起票時実測の参照も同期）/ S6 に `StocktakeProgressHeader` の適用形と `PageHeader.tsx` JSDoc 更新を追記 / Review Focus・Design Sources・Design Intent Trace・Ledger の旧前提を掃除 / AC4 を `= 3` へ再反転 / **AC14**（7 oracle）/ Matrix C4・F4 行を同期
- 再発注: 発注書 43（README `:21` / catalog ① 構造 block・variation・runtime marker / mockup `.heading-title` / 更新履歴）→ closure round 4（Sonnet + Opus、AC4 / AC13 / AC14 + D1 / D4 byte に限定）→ state-only → Ready

### Final Review round 4 closure（2026-09-11、content `17fc8b66`、Sonnet + Opus 独立）

- Codex 43（是正 r3、起点 `01d4a5fb`、`17fc8b66`、commands 24 / 手戻り 0。F1〜F6 完了、AC4 `1 / 1 / 3`、AC13 9/9、AC14 7/7、D1 2,791B / D4 1,543B byte 一致）
- L1 full（Coordinator、`/tmp/rev-20`、HEAD `17fc8b66`、CLEAN）: RESULT=PASS、`MERGE_EVIDENCE_VALID=true`、log `.local/codex-orders/reports/l1-full-pr51-17fc8b66.log`
- Sonnet: approve（新規なし。AC4 / AC13 / AC14 独立再実行で全 PASS、round 3 の canonical 側 findings 5 件すべて閉塞、`.heading-title` 3 箇所・現行 panel 不変、footprint 3 file・⑤⑧⑨ 非接触、仮想 mutant で AC4 落下）
- Opus: approve（P1 0 / P2 0 / P3 6）。round 3 の 11 findings 解消（P3-11 は Backlog 経路のみ未固定）。B(i) `flex-1` = basis 0 + `min-w-0` で右要素が落ちない保証が無条件に成立、Human Gate 再訪不要の裁定も成立。N-1 = catalog `:28` canonical 行の props に `description?` 欠落（⑮ 起源）/ N-2・N-3 = Matrix `:14` `:63` `:18` `:46` の旧前提 / N-4 = 構造 block の h1 に理由句なし / N-5 = packet D4 理由文の旧語 / N-6 = `:has()` の Backlog 起票経路
- Coordinator 裁定: N-1 = runtime lane S7 で 1 語追加（S6 の Backlog 併記に記録、round 5 を呼ばない）/ N-2・N-3・N-5 = 本 commit で是正（Matrix 4 行 + D4 理由文 + `Plans.md` の lane 記述）/ N-4 = no-action（AC4 `= 3` が機械的に守る）/ N-6 = S6 の Backlog 併記に追加
- state-only（本 commit）: `implementing->local-verified->independent-review->human-confirm`、Reviewed Content HEAD `17fc8b66`。local-verified = Coordinator の L1 full、independent-review = Final Review round 1〜4（Sonnet + Opus）+ Codex 39（Freeze 可）。Human Gate 3 件は回答済み、残 = owner の Ready 承認 → `human-confirm->ready-hosted-final` → hosted final は owner `workflow_dispatch`
