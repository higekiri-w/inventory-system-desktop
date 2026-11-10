# Plan Packet: ⑲ 取引先ピッカー統合 dialog の runtime 反映（`SupplierPickerDialog` 新設 / `CreateSupplierDialog` 統合 / 呼び出し元 3 + 取引先管理検索）

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

If a state-only commit materializes multiple phases, list the complete adjacent forward sequence and the pre-existing evidence for every intermediate transition in an append-only review/evidence record. Recording compression never permits a gate skip.

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: b49c7b5b
- Amendments: 5aaed903 2d00ba8e 010b647d 2c76a4ac 083c61b6 c1bd8906 dbbe73a2
- Coordinator: Fable 5.1
- Writer: Codex
- Plan Reviewer: Sonnet + Opus
- Final Reviewer: Sonnet + Opus + Codex
- Reviewed Content HEAD: 5db76334
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: owner Windows native L3（AC-L3-1〜6、dialog 重ね (A) の実機確認を含む）+ Ready 承認 → **L3 run 1（2026-09-11、content `0769d9ee`）**: AC-L3-1 / 2 / 3 / 5 PASS（5 の「Tab が内側から漏れない」は owner に不明瞭だったが PASS）、AC-L3-4 = 検索 PASS、0 件文言と箱内 scroll は件数準備の負担から**自動 test で担保する選択**（owner 提示の二択、Coordinator 推奨 = `SPEC-SUP-D2` test と `closest(".overflow-auto")` assertion が正本）、AC-L3-6 = 是正要（trigger に dialog を想起させる icon が無く「ボタンなのか何なのか分からない」/ 固定帯は下線のみで見やすさに欠ける・左端に帯・「選択中」badge はしつこい）。追加観測: 長い取引先名は「めちゃはみ出してる」（#13 実証）/ 商品修正の取得中フラッシュは見えない（no-action）/ 削除済み id の帯は再現不要（自動 test C14 が正本）。→ Gated Amendment 4 → 是正 → L3 run 2 は AC-L3-1（はみ出し）/ AC-L3-6（icon・帯）のみ → **L3 run 2（2026-09-11、content `be71a857`）: AC-L3-1 / AC-L3-6 PASS「良くなったね」**。観察 3 点 = (1) 帯と先頭行のクリーム色が同じ「変えるなら色」→ Coordinator 案〈帯は白地の箱、クリーム + 左バーを外す〉を owner 承認 / (2) ▼ が Select より濃い「揃えられるなら揃えて」→ `text-muted-foreground` / (3) badge 撤去は「大丈夫に見える」→ no-action。→ Gated Amendment 5 → 是正 → L3 run 3 は AC-L3-6（帯が行と区別できるか、▼ の濃さ）のみ → **L3 run 3（2026-09-11、content `5db76334`）: AC-L3-6 PASS**「L3 PASS かな、気になったら言うわ、バックログはしなくていい」。**owner Ready 承認 2026-09-11**「Ready していいよ」

## Owner Effort Budget

- 介入回数上限: 5（視覚系 + 実機 dialog 重ね確認。⑭ / ⑮ の実績 L3 2〜3 round を見込む）
- 実働時間上限: 60分
- relay 往復上限: 3
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
operator workflow（取引先の選択・追加の手順）と UI 契約（`CreateSupplierDialog` の `onCreated` 引数、3 画面の取引先入力 widget）を変える。DB / Tauri command / DTO / route は変更しないが、本アプリ初の dialog-in-dialog を導入し、その挙動は Windows WebView2 実機でしか確定できない（canonical 自身が「実機未検証の期待値」と明記）。

## Goal

Goal Invariant:

### 最小完了条件

- 一括価格改定の絞り込み・商品登録/修正 form・入庫記録の 3 画面で、取引先を `Select` ではなく取引先ピッカー dialog（名前検索 + scroll 一覧 + 「現在の選択」固定帯 + footer「新しい取引先を追加」primary / 「閉じる」）から選べる。ピッカー内の「新しい取引先を追加」で既存 `CreateSupplierDialog` が重なって開き、追加成功後は新規取引先が自動選択され両 dialog が閉じる（A 案）
- 取引先管理画面に名前検索 input と scroll 一覧の箱がある（client-side filter、footer なし）
- `CreateSupplierDialog` の実装が 1 つ（`onCreated(supplier)` 契約）になり、`ProductForm` の inline 常設パネル（3 つ目の実装）が撤去されている
- owner の Windows native L3 で dialog 重ね (A) の ESC / 外クリック / focus trap / 初期 focus / 復帰先が canonical の期待どおりに動く

### 失敗定義

- 取引先を選ぶ・追加する既存の操作が 3 画面のどれかでできなくなる（取引先未指定で保存できる契約 UI-01b-D8 / UI-02-D3 を含む）
- 一括価格改定の「取引先未設定の商品も含める」toggle の既定 on（SPEC-PRV-D3）が失われる、または dialog 内へ移動する
- dialog 重ね (A) が WebView2 で破綻し（ESC で両方閉じる / 内側 dialog の外クリックが picker まで閉じる / focus が内側から漏れる）、(B) fallback へ切替えずに Ready 化する
- `CreateSupplierDialog` が 2 実装のまま残る、または `ProductForm` の inline パネルが残る

### 非目的

- 取引先管理の並び替え / paging / bulk rename（78 §78.12 Deferred のまま）
- backend `list_suppliers` の並び順変更（`ORDER BY id ASC` のまま。picker 側で client-side name 昇順にする）
- 取引先一覧 query key の統一（`productForm.suppliers` / `priceRevision.suppliers` / `queryKeys.suppliers` の `withUsage()` の 3 key 併存は既存のまま。追加後の再取得は各 host の既存 key で行う）
- フィルタ入力 Label 上置きの全画面統一（⑳ design lane）。本 lane の trigger の Label 配置は各 host の現行配置を維持する
- (B) fallback（単一 dialog 2-stage）の実装。(A) が L3 で破綻した場合のみ Gated Amendment で切替える

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（2026-09-10、worktree base `bb081e3`、Explore 報告を Coordinator が rg で再確認）

### canonical（⑱ PR #49 で確定、本 lane は転記・実装のみ）

- `docs/design-system/01-decision-rules.md:469-479` DSR-24（適用 = 一括価格改定 filter / 商品登録・修正 form / 入庫記録。非適用 = `DepartmentFilter`、取引先管理〈見た目のみ流用〉）
- `docs/design-system/02-component-catalog.md:578-598` picker dialog 小節（構成 `:584` / 動作 `:586` / 重ね契約 (A)(B) `:588-590` / 状態 `:592` / a11y `:594`）。`:582` canonical 行は「後続実装、`src/features/suppliers/components/**` 配下を想定」
- `docs/design-system/reference/mockup-f-supplier-picker.html` 7 状態（state-1 filter 文脈 / state-2 入力文脈 / state-3 取引先管理 / state-4 A 案 / state-6b 二重 scrim / state-7 入庫記録）。背景の trigger は `<span class="btn outline">取引先を選択</span>`（文言は placeholder、現在値表示の規定なし → 設計判断 D5）
- `docs/function-design/77-ui-bulk-price-revision.md:27,30,94`（SPEC-PRV-D3 / D6、『すべての取引先』行は一覧先頭、toggle は filter 列に残置）
- `docs/function-design/51-ui-product-form.md:21,36,149-156,210`（UI-01b-D7 / D21。`:151` canonical `CreateSupplierDialog` = `src/features/products/components/CreateSupplierDialog.tsx` と明記、inline 常設パネルは撤去指示）
- `docs/function-design/78-ui-supplier-management.md:28,55-59,146-158`（SPEC-SUP-D2 名前検索 + scroll 一覧、client-side filter、全件取得維持）
- `docs/function-design/61-ui-receiving.md:23,155`（UI-02-D3 defer 解除、検索 / 追加 / 自動選択で閉じる）
- `docs/design-system/01-decision-rules.md:25` DSR-01 具体例に `ProductForm.tsx:343` / `:481` の stale citation（実 tree は `:387` / `:496`。inline パネル撤去で段落自体が解消される）

### runtime 現状

- `CreateSupplierDialog` 3 実装:
  - `src/features/products/components/CreateSupplierDialog.tsx`（104 行、`onCreated: (supplier: Supplier) => Promise<void>` `:20-28`、`commands.createSupplier` `:42`、呼び出し = `PriceRevisionFilters.tsx:184-191`、import は相対 `./CreateSupplierDialog` `:22`）
  - `src/features/suppliers/components/CreateSupplierDialog.tsx`（115 行、`onCreated: () => Promise<void>` `:20-28`、`<form>` 包み `:69-111`〈Enter 確定 + IME `isComposing` guard `:86-88`〉、`describeError` 使用、input id `supplier-management-new-name` `:77-79`、呼び出し = `SupplierManagementPage.tsx:68-74`。products 版の strict superset）
  - `src/features/products/components/ProductForm.tsx:301-391` inline パネル（`showSupplierInput` state `:98`、toggle button `:322-332`〈`variant="secondary"`〉、`createSupplier` + `listSuppliers` を直接呼び `setSupplierOptions` `:357-386`、確定ボタン「追加する」`:387`）
- 取引先を選ぶ 3 サイト（すべて shadcn `Select`）:
  - `PriceRevisionFilters.tsx:60-83`（`value="all"` sentinel、`suppliersQuery` prop、key `queryKeys.priceRevision.suppliers()`、隣に「新しい取引先を追加」button）。toggle `:143-156`、取得失敗 `:158-170`（`<p role="alert">` + 再試行 link）、部門 alert `:171-183`
  - `ProductForm.tsx:303-321`（`value="none"` sentinel「取引先なし」、`suppliers` prop `:43` を local `supplierOptions` state に写す、`supplierWarning` 時 disabled `:305`）
  - `ReceivingPage.tsx:396-415`（`value="none"` sentinel「指定なし」、`supplierQuery` `:133-141` = `queryKeys.productForm.suppliers()` を ProductForm と共用、`isFormLocked || supplierQuery.isLoading` で disabled）
- `SupplierManagementPage.tsx`（86 行）: 検索 input なし。`useSuppliersWithUsage()`（backend `ORDER BY s.name ASC`、`product_repo.rs:556`）→ `SupplierUsageTable`。`Plus` は lucide-react `:1,28`
- `commands.listSuppliers()` は `ORDER BY id ASC`（`product_repo.rs:420`）。name 昇順は取引先管理の `list_suppliers_with_usage` のみ
- `src/components/ui/dialog.tsx`: `radix-ui` 統合 package、`DialogContent` は `...props` 透過（`onOpenAutoFocus` / `onInteractOutside` / `onEscapeKeyDown` を呼び出し側から渡せる）、`DialogOverlay` = `bg-black/50 z-50` `:34`
- dialog-in-dialog の先例・test 先例なし（`rg` 0 件）。`MergeSupplierDialog.tsx` が単一 dialog 2-stage の先例（(B) fallback の型）
- `ListShell.tsx:51` `STICKY_TABLE_CLASSES` / `:223` `max-h-[calc(100vh-6.75rem)] overflow-auto` が商品一覧の scroll 箱
- `ProductForm.test.tsx:691-758` が inline パネルの test（`data-variant="secondary"` assertion `:695-698` を含む）。`ProductForm.test.tsx` に `QueryClientProvider` なし（0 件）→ ProductForm は queryClient を持たない。`ProductForm.tsx:97,104` の `setSupplierOptions` は `suppliers` prop を local state に**加算 merge** する effect（親の stale prop が local 追加分を消さない）
- Lane 5 の control 面 token test: `ProductForm.test.tsx:763-800`（SC4e: 部門 / 取引先 / 税率が `bg-control-surface`、not `bg-background`）、`ReceivingPage.test.tsx:1027-1032`（SC4b: 取引先 select 同様、assertion `:1031-1032`）。`Button` `outline` variant は `bg-background`（`button.tsx:16`）、`Input` / `SelectTrigger` は `bg-control-surface`（`input.tsx:11` / `select.tsx:53`）
- `PriceRevisionFilters.test.tsx`（92 行）は全体が ⑭ GA2 の群化 test 2 本（`:57` / `:80`）、header comment `:3-11` に「REQ/UI ID を付けない代わりに `FE_UNREFERENCED_BASELINE` を更新した」経緯。toggle「取引先未設定の商品も含める」の test は無い（`includeUnassigned` は fixture `:36` のみ）
- `src-tauri/src/bin/generate_traceability.rs:54` `FE_UNREFERENCED_BASELINE = 26`（[T4]、ID 未参照の FE test file 数を増減両方向で ERROR）
- bindings: `Supplier { id, name, created_at }`（`bindings.ts:1645-1649`）、`createSupplier(name) => Supplier` `:28`、`listSuppliers() => Supplier[]` `:26`
- test file 数 170（`fd -e test.tsx -e test.ts . src`）

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **D1 `CreateSupplierDialog` の canonical 位置 = `src/features/suppliers/components/CreateSupplierDialog.tsx`**（51 `:151` の literal「`src/features/products/components/…`」を改める）。理由: 取引先 master を所有する feature は suppliers。`SupplierPickerDialog` を suppliers 配下に置く（catalog `:582`）以上、picker → products/CreateSupplierDialog、products/ProductForm → suppliers/Picker の相互依存を避ける唯一の配置。suppliers 版の本体（`<form>` 包み = Enter 確定、`describeError`）を残し、`onCreated` の型だけ products 版の `(supplier: Supplier) => Promise<void>` に揃える。products 版は削除。却下案 = products 版を canonical にして suppliers 版を削る（feature 依存が双方向になる）
- **D2 `SupplierPickerDialog` の props 契約**（`src/features/suppliers/components/SupplierPickerDialog.tsx`）:
  - `open: boolean` / `onOpenChange(open)` — 通常の Radix 制御
  - `suppliers: Supplier[]` / `isLoading: boolean` / `isError: boolean` / `onRetry(): void` — host の query 状態をそのまま渡す（picker は fetch しない。3 host の query key が異なるため、picker が key を知る必要をなくす）
  - `leadingLabel: string` — 先頭行の文言（filter = 「すべての取引先」、ProductForm = 「取引先なし」、入庫 = 「指定なし」）
  - `selected: number | null` — 現在選択（`null` = 先頭行）
  - `onSelect(id: number | null): void` — 行クリックで呼び、picker は自分で閉じる
  - `onCreated(supplier: Supplier): Promise<void>` — host が一覧を再取得する。picker 内部の handler = `await onCreated(s)` → `onSelect(s.id)` → 両 dialog を閉じる（A 案）
  - `triggerLabel` は持たない。trigger は host 側（D5）
- **D3 一覧の並び = picker 内で `name.localeCompare(other, "ja")` 昇順**（backend 不変）。理由: 80 件の目視走査は名前順が前提、取引先管理（name ASC）と揃える。1 行で済む
- **D4 検索 input = `Label` + `Input type="search"`（`grid gap-1`）を picker 内に直接置く。`SearchBar` は使わない**。理由: `SearchBar` の debounce / Enter flush / IME guard は URL state・server 再取得のための機構で、取得済み 80 件の in-memory filter には不要。test も同期で書ける。filter = `name.includes(query.trim())`（大小文字正規化なし、日本語名主体）。0 件 = `EmptyState`（絞り込み解除 action なし、catalog `:592`）
- **D5 host 側 trigger = `Button variant="outline"` + `className="w-full justify-between bg-control-surface"`、文言 = 現在の選択（取引先名 / `leadingLabel`）、`aria-haspopup="dialog"`、accessible name は `aria-labelledby="<label id> <trigger id>"`（host の「取引先」`<Label id>` + trigger 自身の id）で「取引先 + 現在値」にする**。mockup の `取引先を選択` は placeholder 文言（背景の非対話 `<span>`、`mockup-f:69,85,109`）であり、閉じた状態で現在値が見えないのは `Select` からの後退になるため現在値を出す。icon は付けない（owner「飾りより情報」）。`bg-control-surface` は Lane 5 SC4e / SC4b（`ProductForm.test.tsx:763-800`〈assertion `:792-800`〉/ `ReceivingPage.test.tsx:1027-1032`〈assertion `:1031-1032`〉）が取引先欄に要求する control 面 token で、`cn()` の twMerge が `outline` の `bg-background` を置換する。`<Label htmlFor>` + `<button>` の名前計算は HTML-AAM で非標準のため `aria-labelledby` を使い、現在値の assertion は role name でなく `toHaveTextContent` で行う（Plan Review round 1 Opus P1-3 / P2-4）。**現行 host の Label は `htmlFor` のみで `id` を持たない**（`ProductForm.tsx:302` `<Label htmlFor="supplier-id">` / `ReceivingPage.tsx:396` `<Label htmlFor="receiving-supplier">` / `PriceRevisionFilters.tsx:61` は素の `<label htmlFor="price-revision-supplier">`）ため、`aria-labelledby` 用に `id` を**新規付与**する（round 2 Opus P2-2。落とすと `getByLabelText("取引先")` が解決せず SC4e / SC4b が静かに壊れる。Contract Probe P3）。disabled 条件は各 host の現行 `Select` の条件を引き継ぐ（`supplierWarning !== null` / `isFormLocked || isLoading`）。**owner L3 で見た目を確認する（AC-L3-6）**。本項は catalog `:584` への規範追加（literal 同期ではない）として S7 で source 化する。**Gated Amendment 3（Final Review round 1 Opus P1-1 / P2-4）**: (a) 現在値の解決は picker が export する 1 関数 `supplierCurrentLabel(suppliers, selected, leadingLabel)` に集約し、`selected === null` なら `leadingLabel`、一覧に該当 id が無ければ **「取引先を確認できません」**（取得失敗で `data` が無い / 削除済み id。未指定文言で断定しない）を返す。picker の固定帯と 3 host の trigger はこの関数だけを使う（literal は src に 1 箇所）。trigger の disabled 条件は不変で `isError` でも押せる（picker 内の Alert + 再試行に到達させる。host に alert を戻さない = S3 の一本化を維持）。(b) trigger 幅は block 配置の `ProductForm` / `ReceivingPage` が `w-full`、filter 行の `PriceRevisionFilters` は旧 `SelectTrigger` と同じ **`w-48`**（`flex flex-wrap items-end gap-3` の中で選択名の長さに幅が追従しないよう固定）。catalog `:584` に未解決時文言の 1 句を足す（S7）。**Gated Amendment 4（owner L3 run 1、2026-09-11）**: (c) trigger の右端に lucide `ChevronDown`（`className="size-4 opacity-50"`、`aria-hidden`、`select.tsx:60` の `SelectTrigger` と同じ見え方。**Gated Amendment 5**: `text-muted-foreground` を足し、`SelectTrigger` の `[&_svg]:text-muted-foreground` と同じ薄さにする〈owner L3 run 2「ちょっと濃く見える、揃えられるなら揃えて」、Final Review round 3 Opus #19〉）を置き「押すと選択肢が開く」控えを示す。owner「パッと見『指定なし』としか書いてなくてボタンなのか何なのか分からない、dialog が開くことを想起させるアイコンが要る」により旧「icon は付けない」を撤回（AC-L3-6 の想定どおり）。(d) trigger の文言は `<span className="truncate">` で包む（3 host 一律。Final Review round 2 Opus #13、owner L3「めちゃはみ出してる」で実証）。accessible name は `aria-labelledby` のままで変わらない
- **D6 scroll 箱 = `<div className="max-h-[50vh] overflow-auto rounded-md border">` で `<table>` を包むだけ**（picker / 取引先管理とも同じ class 文字列。**Gated Amendment 3（Opus P2-3）**: 取引先管理は `SupplierUsageTable` の root が既に `rounded-md border` を持つため、外側は `max-h-[50vh] overflow-auto` のみとし枠と角丸を二重にしない〈owner「線少なく角丸」〉。picker 側は table に枠が無いので現行どおり。取引先管理は viewport 基準の `max-h-[calc(100vh-…)]` でも可、Writer が実画面で選ぶ）。`ListShell.tsx:51` `STICKY_TABLE_CLASSES` は `stickyHeader` 時のみ適用される thead 固定用（`:199`）で、列見出しが `sr-only` の picker には効かないため **export も流用もしない**（Plan Review round 1 Opus P2-5）。`ListShell.tsx:223` の viewport `max-h` も別 div のもの。新規 class 系統を作らない
- **D7 「現在の選択」固定帯 = scroll 箱の外、検索 input の下**（catalog `:584` literal）。小見出し「現在の選択」（`text-xs text-muted-foreground`）+ 名前 + `Badge`「選択中」、背景 `bg-row-current`（token 名は `00-foundations` / 既存 DSR-22 実装の `--row-current` 利用箇所を Writer が `rg` で確認して同じ class を使う）、下辺 `border-b border-border-strong`。**Gated Amendment 4（owner L3 run 1、2026-09-11）**: 帯 = `rounded-md border border-border-strong border-l-4 border-l-primary bg-row-current px-4 py-3`（全周の枠 + 左 4px primary バー + 背景）+ 小見出し + 名前、**`Badge`「選択中」は帯に置かない**（一覧側の現在行にだけ残す）。owner「下線だけで上左右がクリーム色の枠だけなのは見やすさに欠ける」「帯の左端に帯を付けるか」「選択中バッジを帯にも付けるのはしつこい」。未解決時は帯にも「取引先を確認できません」（`supplierCurrentLabel`、Final Review round 2 Opus #15 は badge 撤去で解消）。検索 input / scroll 箱との上下 gap は現行の `DialogContent` の並びを保つ。**Gated Amendment 5（owner L3 run 2、2026-09-11）**: 帯 = **白地の箱** `rounded-md border border-border-strong bg-background px-4 py-3`（全周枠 + 角丸のみ。`bg-row-current` と `border-l-4 border-l-primary` を帯から外す）+ 小見出し + 取引先名 `font-medium`。理由: クリーム背景 + 左バーは DSR-22「一覧の中で今選ばれている行」の語彙で、帯が同じ語彙を借りると既定状態で帯と先頭行が同じ見た目で 2 段続く（Final Review round 3 Opus #21、owner「上と下で選択中のクリーム色も一緒だから、変えるなら色」）。箱 = 要約 / クリーム + バー = 現在行、と語彙を 1 対 1 にする。class 順は Prettier の並べ替えに任せ `prettier-ignore` を外す（AC16 の oracle は順序非依存、Opus #18）
- **D8 ProductForm の追加後再取得 = 既存 `:371-376` の `commands.listSuppliers()` → `setSupplierOptions` を `onCreated` に移す**（ProductForm は queryClient を持たず test にも Provider が無い。invalidate 化は非目的）。ReceivingPage = `supplierQuery.refetch()`、PriceRevisionFilters = 既存 `:187-190` と同じ、取引先管理 = `suppliersQuery.refetch()`（引数は無視）
- **D9 DSR-01 `:25` の「runtime 是正対象（2026-09-05 起票時実測）: `ProductForm.tsx:343`…」文は、inline パネル撤去で対象が消えるため、citation 訂正ではなく「解消済み（本 PR、DSR-24 の picker 化で inline パネル撤去）」に書き換える**（Backlog の「`:343`→`:387`・`:481`→`:496` 同乗」は訂正でなく解消として消化）
- **D10 dialog 重ね (A) の自動 test は jsdom で書く**（内側 open 中の ESC が内側だけ閉じる / 内側 close 後の focus が検索 input）。jsdom で Radix `DismissableLayer` の stack が再現しない場合は当該 test を「WebView2 L3 のみ」に落とし、Implementation Results に理由を残す（Contract Probe P1）

## Scope

- **S1 `SupplierPickerDialog` 新設**（`src/features/suppliers/components/SupplierPickerDialog.tsx` + `SupplierPickerDialog.test.tsx`）: D2 props、構成 = `DialogTitle`「取引先を選択」+ `DialogDescription`（1 文、例「名前で検索して選ぶか、新しい取引先を追加します。」）+ 検索（D4）+ 固定帯（D7）+ scroll 一覧（D6、`<table>` で列見出し「選択」は `sr-only`、行 = ✓〈lucide `Check`〉+ `Badge`「選択中」+ 名前、現在行に DSR-22 の 3 点〈左 4px primary バー + `bg-row-current` + ✓ / badge〉、先頭行 `leadingLabel`、以降 D3 順）+ footer（`shrink-0`、左 `Button` primary〈lucide `Plus` + 「新しい取引先を追加」〉、右 `Button variant="outline"`「閉じる」）。状態 = `isLoading` → `ListSkeleton`、`isError` → `Alert variant="destructive"` + 再試行（`onRetry`）、検索 0 件 → `EmptyState`。動作 = 行クリック → `onSelect` + 閉じる / 外クリック・Esc → 閉じる（選択不変）/ 追加 → 内側に `CreateSupplierDialog`（S2）を開く、成功 → `onCreated` → `onSelect(id)` → 両方閉じる。a11y = open 時 focus は検索 input（Radix 既定の最初の focusable で足りなければ `onOpenAutoFocus` で明示）、内側 close 後の focus は検索 input（`CreateSupplierDialog` の `onCloseAutoFocus` で指定）、閉じたら trigger へ戻る（Radix 既定）
- **S2 `CreateSupplierDialog` 統合**（D1）: `src/features/suppliers/components/CreateSupplierDialog.tsx` の `onCreated` を `(supplier: Supplier) => Promise<void>` に変更し `:46` で `await onCreated(supplier)`。`src/features/products/components/CreateSupplierDialog.tsx` を削除する。**`CreateSupplierDialog` を直接 import・描画するのは `SupplierPickerDialog.tsx`（picker 内で nest、相対 `./CreateSupplierDialog`）と `SupplierManagementPage.tsx` の 2 箇所のみ**。`PriceRevisionFilters.tsx:22` の import と `:184-191` の `<CreateSupplierDialog>` 描画は S3 で picker に置き換わるため**撤去**する（`ProductForm` / `ReceivingPage` も同様に host からは import しない。**Gated Amendment 1**: 旧文「import を suppliers 版へ付け替える」は S3 と矛盾し `noUnusedLocals` で typecheck が落ちる — Codex 36 の fail-closed 停止で発見）。`SupplierManagementPage.tsx:68-74` は callback 引数を無視するだけで動作不変。suppliers 版の input id `supplier-management-new-name`（`:77-79`）は 4 host で使われるため `create-supplier-name` に改名（同時 mount は無いので重複 id ではないが命名の実態合わせ）
- **S3 `PriceRevisionFilters.tsx` の取引先 `Select` → trigger + picker**（`:60-83` の `Select` と隣接「新しい取引先を追加」button を撤去、D5 trigger に置換、`leadingLabel="すべての取引先"`、`selected = normalized.supplier ?? null`、`onSelect(id) → onPatch({ supplier: id })`〈`null` は既存どおり `all` 意味〉、`onCreated` = `await suppliersQuery.refetch()` **のみ**（既存 `:187-190` の `onPatch({ supplier: supplier.id })` は移さない。選択は picker の `onSelect(s.id)` が行うため、移すと二重発火し Mutation #3 が kill できない。Opus P2-3）。`:143-156` toggle は不変（filter 列に残す、SPEC-PRV-D3）。`:158-170` の取得失敗 `<p role="alert">` は **撤去**し picker の `isError` / `onRetry` に一本化する（Coordinator 決定、Opus P2-5。部門の alert `:171-183` は不変。既存 test はこの alert を assert していない〈`rg "取引先一覧を取得できませんでした" src` = `PriceRevisionFilters.tsx:160` のみ〉）。**`PriceRevisionFilters.test.tsx` は全体が GA2（⑭ Gated Amendment 2、owner L3 起源）の「取引先 label / Select / 追加ボタンの群化」test 2 本（`:57` wrapper 共有 / `:80` DOM 順序）で、両方の subject が消える → trigger 版に書き換える（Label + trigger が 1 wrapper、DOM 順序 = Label → trigger → 部門 → … → 表示件数）+ header comment `:3-11` を同 commit で更新**。加えて新規 test「取引先選択時に『取引先未設定の商品も含める』が filter 列に既定 on で現れる」を追加（SPEC-PRV-D3 の regression、既存 test には無い。Sonnet P1-2）。GA2 の群化契約は Ledger に行を立てる）。**Gated Amendment 2（Codex 36b fail-closed 停止起源）: page 側 test `src/features/products/PriceRevisionPage.test.tsx` も S3 の対象**。旧 Select 前提の 6 test を picker 経由へ書き換える（Q3 規則で test 名 / comment も更新、削除・skip 不可）: `:166-204` の取引先選択（`findByRole("option", { name: "取引先A" })` → trigger click → picker 内の行 click）/ `:523-575` の「新しい取引先を追加」page flow（画面上の button → picker footer の primary → 内側 dialog、`toast` / `listSuppliers` 再取得 / filter 選択状態の assertion は維持）/ `:677-690` SC6（`data-slot="select-trigger"` + `suppliersQuery` loading 中 disabled → trigger が loading 中 `disabled`、`data-slot` assertion は撤去、Lane 5 の disabled 契約は in-place で維持）。この file は `REQ-105` / `UI-14` で referenced（[T4] 影響なし）
- **S4 `ProductForm.tsx` の取引先 `Select` + inline パネル → trigger + picker**（`:303-391` を D5 trigger + `SupplierPickerDialog`〈`leadingLabel="取引先なし"`〉に置換、`showSupplierInput` / `supplierName` / `supplierCreateError` / `isCreatingSupplier` state と `:322-332` toggle button を撤去、`onCreated` = D8。`supplierWarning !== null` の disabled は trigger に引き継ぐ）。`ProductForm.test.tsx:691-758` の inline パネル test を picker 経由の flow（空白名拒否は `CreateSupplierDialog` 側の既存 test に委ね、ProductForm 側は「追加成功 → 自動選択 → 両 dialog close」を検証）に書き換え、`data-variant="secondary"` assertion `:695-698` を撤去。**`ProductForm.test.tsx:763-800`（Lane 5 SC4e: `getByLabelText("取引先")` が `bg-control-surface` を持ち `bg-background` を持たない）は trigger に対して in-place で成立させる**（D5、`getByLabelText` は `aria-labelledby` でも解決する）。`ProductForm.test.tsx:18-24` は `@/lib/bindings` のみ mock のため、統合後の dialog から `sonner` の `toast.success` が届く（既存 SupplierManagementPage.test と同じ扱い、必要なら mock）
- **S5 `ReceivingPage.tsx` の取引先 `Select` → trigger + picker**（`:396-415`、`leadingLabel="指定なし"`、`selected = values.supplierId`、`onSelect(id) → updateValues(supplierId: id)`、`onCreated → supplierQuery.refetch()`、disabled = `isFormLocked || supplierQuery.isLoading`）。`ReceivingPage.test.tsx:237,254,276` の Select 依存 assertion を trigger + picker 経由に書き換え。**`:1027-1032`（Lane 5 SC4b: 取引先欄の `bg-control-surface` / not `bg-background`、assertion は `:1031-1032`）は trigger に対して in-place で成立させる**（D5）。`onCreated` は `async () => { await supplierQuery.refetch(); }` の形にする（`Promise<QueryObserverResult>` を `Promise<void>` に渡すため）
- **S6 取引先管理の名前検索 + scroll 箱**（`SupplierManagementPage.tsx`）: `PageHeader` の下に `Label`「取引先名で検索」+ `Input type="search"`（`grid gap-1`、D4 と同じ filter）、`SupplierUsageTable` を D6 の scroll 箱で包む。data が非空で filter 結果 0 件 → `EmptyState`「該当する取引先はありません」（既存の「取引先はまだ登録されていません」は data 空のときのみ）。検索 input は data が非空のときだけ描画し、`isLoading` / `isError` / data 空の 3 分岐（`:41-59`）は不変。`SupplierManagementPage.test.tsx` に「検索で一覧が絞られる / 0 件文言」test 追加
- **S7 docs 同期**: `02-component-catalog.md:582` canonical 行を実装済み path へ（「後続実装、…想定」を消す）/ `01-decision-rules.md:25` D9 / `51-ui-product-form.md:151` canonical path を D1 へ + 変更履歴 / `61` `77` `78` の変更履歴に runtime 反映 1 行 / `docs/design-system/README.md`・`UI_TECH_STACK.md` に picker の「後続実装」文言があれば同期（Writer が `rg -n "後続実装|runtime lane" docs/design-system docs/UI_TECH_STACK.md` で実測、無ければ変更なしと報告）

## Non-scope

- 78 §78.12 Deferred（並び替え / paging / bulk rename / 単独削除）
- backend（`src-tauri/**`）一切。`list_suppliers` の並び順、DTO、command 追加なし
- query key の統一・cross-screen cache invalidation（既存 3 key のまま）
- `DepartmentFilter` / 部門 `Select` の picker 化（DSR-24 非適用）
- フィルタ入力 Label 上置きの sweep（⑳ design lane）。3 host の Label 配置は現行維持
- (B) fallback の実装（L3 破綻時のみ Gated Amendment）
- `MergeSupplierDialog` / `RenameSupplierRow` の変更
- `docs/decision-log.md`（durable 判断の追加なし。D1 の path 変更は 51 の記述更新で足りる）
- mockup-f の更新（trigger 文言の差 D5 は mockup を直さず本 packet と catalog `:584` への 1 句追記で記録）
- ⑳ lane（`agent/filter-label-top-design`、docs-only）との衝突: `02-component-catalog.md` の更新履歴表末尾でのみ textual conflict が出る想定（⑳ は DSR-01 を触らない。Plans.md wave 9 行と同じ範囲）。merge 順は先に Ready になった方、後続が origin/main を単段 merge し両側保持で解消

## Acceptance Criteria

R3 のため各 AC に機械 oracle を付す。`rg` の出力空 = 0 件。

- **AC1** `fd -g "SupplierPickerDialog.tsx" src/features/suppliers/components | wc -l` = 1、`fd -g "SupplierPickerDialog.test.tsx" src/features/suppliers/components | wc -l` = 1（baseline 0 / 0）
- **AC2** `fd -g "CreateSupplierDialog.tsx" src/features/products | wc -l` = 0（baseline 1）/ `rg -c "onCreated: \(supplier: Supplier\) => Promise<void>" src/features/suppliers/components/CreateSupplierDialog.tsx` = 1（baseline 0）/ `rg -c "CreateSupplierDialog" src/features/suppliers/components/SupplierPickerDialog.tsx` ≥ 2（import + JSX、baseline = file 不在。**Gated Amendment 1** で検査先を `PriceRevisionFilters.tsx` から picker へ変更）/ `rg -c "CreateSupplierDialog" src/features/products/components/PriceRevisionFilters.tsx` = 0（baseline 2 = import `:22` + JSX `:184`。host からの参照撤去、`ProductForm.tsx` / `ReceivingPage.tsx` は baseline 0 のまま 0）/ `rg -c "CreateSupplierDialog" src/features/suppliers/SupplierManagementPage.tsx` = 2（baseline 2、不変）/ `rg -c 'id="create-supplier-name"' src/features/suppliers/components/CreateSupplierDialog.tsx` = 1（baseline 0）
- **AC3** `rg -n "showSupplierInput|new-supplier-name|isCreatingSupplier|supplierCreateError" src/features/products/components/ProductForm.tsx | wc -l` = 0（baseline 9。`setSupplierOptions` は D8 の再取得と `:97,104` の prop merge で残るため pattern に含めない〈Opus P1-1〉）/ `rg -c "commands.listSuppliers" src/features/products/components/ProductForm.tsx` ≥ 1（D8、baseline 1）/ `rg -c "SupplierPickerDialog" src/features/products/components/ProductForm.tsx` ≥ 2（import + JSX、baseline 0）
- **AC4** `rg -n 'SelectItem value="all"' src/features/products/components/PriceRevisionFilters.tsx | wc -l` = 0（baseline 1）/ `rg -c "SupplierPickerDialog" src/features/products/components/PriceRevisionFilters.tsx` ≥ 2 / `rg -n "取引先未設定の商品も含める" src/features/products/components/PriceRevisionFilters.tsx | wc -l` = 1（不変、toggle 残置）
- **AC5** `rg -n 'SelectItem value="none">指定なし' src/features/receiving/ReceivingPage.tsx | wc -l` = 0（baseline 1）/ `rg -c "SupplierPickerDialog" src/features/receiving/ReceivingPage.tsx` ≥ 2
- **AC6** `rg -n "取引先名で検索" src/features/suppliers/SupplierManagementPage.tsx | wc -l` ≥ 1（baseline 0）/ `rg -n "該当する取引先はありません" src/features/suppliers/SupplierManagementPage.tsx | wc -l` = 1
- **AC7** picker 構成の literal: `rg -c "DialogTitle" src/features/suppliers/components/SupplierPickerDialog.tsx` ≥ 1、同 `DialogDescription` ≥ 1、`sr-only` ≥ 1、`現在の選択` ≥ 1、`選択中` ≥ 1、`新しい取引先を追加` = 1、`閉じる` ≥ 1、`localeCompare` = 1、`rg -n "＋" src/features/suppliers/components/SupplierPickerDialog.tsx | wc -l` = 0、`rg -c "Plus" …SupplierPickerDialog.tsx` ≥ 1、`rg -c "SearchBar" …SupplierPickerDialog.tsx` = 0（D4）
- **AC8** A 案の自動 test: `rg -n "auto-selects the created supplier and closes both dialogs" src/features/suppliers/components/SupplierPickerDialog.test.tsx | wc -l` = 1、`npx vitest run src/features/suppliers` green
- **AC9** trigger（D5）: `rg -n 'aria-haspopup="dialog"' src/features/products/components/PriceRevisionFilters.tsx src/features/products/components/ProductForm.tsx src/features/receiving/ReceivingPage.tsx | wc -l` = 3（baseline 0）
- **AC10** docs 同期: `rg -n "SupplierPickerDialog.*後続実装" docs/design-system/02-component-catalog.md | wc -l` = 0（baseline 1 = `:582`。無関係な `:514` `DiscontinueConfirmDialog` の「後続実装」は触らない）/ `rg -n "ProductForm.tsx:343|ProductForm.tsx:481" docs/design-system/01-decision-rules.md | wc -l` = 0（baseline 1 行）/ `rg -n "src/features/suppliers/components/CreateSupplierDialog.tsx" docs/function-design/51-ui-product-form.md | wc -l` ≥ 1（baseline 0）/ `rg -n "products/components/CreateSupplierDialog" docs --glob '!docs/archive/**' --glob '!docs/plans/**' | wc -l` = 0（baseline 1）/ 変更履歴表に本 PR 行が各 1: `awk '/^## .*変更履歴|^## 更新履歴/,0' <doc> | rg -c "取引先ピッカー"` が **01 = 1（baseline 0）/ 02 = 1（0）/ 51 = 2（1）/ 61 = 1（0）/ 77 = 2（1）/ 78 = 2（1）**（見出しは 01・02 `## 更新履歴`、51 `## 7.9 変更履歴`、61 `## 61.10 変更履歴`、77 `## 77.10 変更履歴`、78 `## 78.13 変更履歴`）
- **AC11** 品質 gate: `npm run lint` / `npm run typecheck` / `npx vitest run` 全 green / `bash scripts/local-ci.sh full` `RESULT=PASS`（1 worktree 1 run）/ `bash scripts/doc-consistency-check.sh` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS。[T4] の事前 oracle: `rg -cP '\b(REQ-[0-9]{3}\b|UI-[0-9]{2}[a-z]?\b)' src/features/suppliers/components/SupplierPickerDialog.test.tsx` ≥ 1（baseline: file 不在）/ `rg -cP '\b(REQ-[0-9]{3}\b|UI-[0-9]{2}[a-z]?\b)' src/features/products/components/PriceRevisionFilters.test.tsx` = 0（baseline 0、増やさない）
- **AC12** plan-first commit は docs のみ: `git diff --name-only origin/main..<Plan Commit> -- src src-tauri | wc -l` = 0
- **AC13**（Gated Amendment 3、Final Review round 1 の是正 oracle。baseline は `bbf0edc5` 時点）: `rg -c "取引先を確認できません" src --glob '!*.test.tsx'` = 1（時点 0、literal は `supplierCurrentLabel` の 1 箇所のみ）/ `rg -l "supplierCurrentLabel" src --glob '!*.test.tsx' | wc -l` = 4（時点 0、picker + 3 host）/ `rg -c "取引先を確認できません" src/features/suppliers/components/SupplierPickerDialog.test.tsx` ≥ 1 かつ `src/features/products/components/PriceRevisionFilters.test.tsx` ≥ 1（時点 0 / 0、未解決時の test。後者は `isError` × `data` undefined × supplier 選択済みの経路）/ `rg -B3 "event.stopPropagation" src/features/suppliers/components/SupplierPickerDialog.tsx | rg -c "portal"` ≥ 1（時点 0、防壁の理由 comment）/ `rg -c "expect\(onSubmit\).not.toHaveBeenCalled" src/features/products/components/ProductForm.test.tsx` ≥ 1（時点 0）/ `rg -c 'className="max-h-\[50vh\] overflow-auto"' src/features/suppliers/SupplierManagementPage.tsx` = 1 かつ同 file の `rounded-md border` = 0（時点 0 / 1）/ `rg -c "w-48" src/features/products/components/PriceRevisionFilters.tsx` ≥ 1 かつ同 file の `w-full justify-between` = 0（時点 0 / 1）/ `rg -c '"default"' src/features/products/components/ProductForm.test.tsx` ≥ 1（時点 0、「追加する」の variant assertion を反転して復元）/ `rg -c "L3" src/features/products/components/PriceRevisionFilters.tsx` ≥ 1 かつ `rg -c "baseline|BASELINE" src/features/products/components/PriceRevisionFilters.test.tsx` ≥ 1（時点 0 / 0、GA2 rationale の provenance 復元）/ `rg -c "取引先select" src/features/products/PriceRevisionPage.test.tsx` = 0（時点 1）/ `rg -c "border-l-primary" src/features/suppliers/components/SupplierPickerDialog.test.tsx` ≥ 1（時点 0）/ `rg -c "parentElement" src/features/suppliers/SupplierManagementPage.test.tsx` = 0（時点 1、`closest` へ）/ `rg -c "取引先を確認できません" docs/design-system/02-component-catalog.md` ≥ 1（時点 0、`:584` の 1 句）
- **AC-L3-1**（一括価格改定 `PriceRevisionFilters.tsx`、WebView2）: trigger に現在値「すべての取引先」→ 開く → 検索で絞る → 行クリックで閉じ trigger に取引先名、toggle が既定 on で現れ位置は filter 列
- **AC-L3-2**（商品登録 form `ProductForm.tsx`）: trigger「取引先なし」→ 開く → 「新しい取引先を追加」→ 内側 dialog が**手前に重なって**開く（二重 scrim）→ 名前入力 → 追加 → 両 dialog が閉じ trigger に新規名。inline パネルが無い。picker 内の「現在の選択」固定帯が一覧の行と高さ・背景・境界で区別でき「流れない帯」と分かる（catalog `:584`、Opus P2-6）
- **AC-L3-3**（入庫記録 `ReceivingPage.tsx`）: AC-L3-2 と同じ flow、「指定なし」に戻せる、取引先未指定で保存できる
- **AC-L3-4**（取引先管理 `SupplierManagementPage.tsx`）: 検索 input で一覧が絞られる、0 件文言、scroll 箱内で一覧が縦 scroll、追加ボタン・名前変更・統合は従来どおり（run 1: 検索 PASS。0 件文言と箱内 scroll は owner 選択で自動 test 担保 = `SPEC-SUP-D2` + `closest(".overflow-auto")`、実機再確認なし）
- **AC-L3-5**（dialog 重ね (A) の実機契約、catalog `:589`）: 内側 open 中の **ESC は内側だけ**閉じる / 内側の**外クリックは内側だけ**閉じる / Tab が内側から漏れない / 内側 close 後の focus は picker の検索 input / picker close 後の focus は trigger。1 つでも破綻 → state-backtrack + Gated Amendment で (B) へ
- **AC-L3-6**（trigger の見た目、D5、`aria-haspopup="dialog"` の outline `Button`）: outline button に現在値、icon なし、Label「取引先」との対応が自然。owner 所感で「Select らしさが要る」なら Gated Amendment（chevron 追加）→ **run 1 で発動、GA4 で `ChevronDown` 追加 + 固定帯を全周枠 + 左バー + badge なしへ。run 2 で trigger と帯を再確認**
- **AC15**（Gated Amendment 4、owner L3 run 1 の是正 oracle。baseline は `0769d9ee` 時点）: `rg -l "ChevronDown" src/features/products/components/PriceRevisionFilters.tsx src/features/products/components/ProductForm.tsx src/features/receiving/ReceivingPage.tsx | wc -l` = 3（時点 0）/ `rg -c 'className="truncate"' <同 3 file>` 各 ≥ 1（時点 0）/ `rg -c "border-l-4 border-l-primary" src/features/suppliers/components/SupplierPickerDialog.tsx` = 2（時点 1、帯 + 現在行）/ `rg -c "<Badge" src/features/suppliers/components/SupplierPickerDialog.tsx` = 1（時点 2、帯の badge 撤去）/ `rg -c "border-b border-border-strong" src/features/suppliers/components/SupplierPickerDialog.tsx` = 0（時点 1）/ `rg -c "lucide-chevron-down" src/features/products/components/PriceRevisionFilters.test.tsx` ≥ 1（時点 0、C16 test）/ `awk '/^## ⑧/,/^## ⑨/' docs/design-system/02-component-catalog.md | rg -c "ChevronDown"` ≥ 1（時点 0）/ `awk '/^## ⑧/,/^## ⑨/' docs/design-system/02-component-catalog.md | rg -c "帯に badge は置かない"` ≥ 1（時点 0）/ `awk '/^## ⑧/,/^## ⑨/' docs/design-system/02-component-catalog.md | rg -c "icon は付けない|icon なし"` = 0（時点 = Writer が実測して報告）。**GA5 で反転**: 第 3 oracle `border-l-4 border-l-primary` は = 1（帯から外す。順序非依存の AC16 が正本）
- **AC16**（Gated Amendment 5、owner L3 run 2 の是正 oracle。baseline は `be71a857` 時点）: `rg -c "border-l-4" src/features/suppliers/components/SupplierPickerDialog.tsx` = 2 かつ `rg -c "border-l-primary"` 同 = 1（時点 3 / 2。`border-l-4` は現在行の primary バーと非現在行の `border-l-transparent`〈位置合わせ〉の 2 行が正。Codex 45 が「期待 1 に対し実測 2」で push 保留 → Coordinator の baseline 誤りを訂正）/ `rg -c "bg-row-current" src/features/suppliers/components/SupplierPickerDialog.tsx` = 1（時点 2、現在行のみ）/ `rg -c "bg-background" src/features/suppliers/components/SupplierPickerDialog.tsx` ≥ 1（時点 0、帯）/ `rg -c "prettier-ignore" src/features/suppliers/components/SupplierPickerDialog.tsx` = 0（時点 1）/ `rg -c "font-medium" src/features/suppliers/components/SupplierPickerDialog.tsx` ≥ 1（時点 0、帯の取引先名）/ `rg -c "ChevronDown.*text-muted-foreground|text-muted-foreground.*ChevronDown" <3 host file>` 各 ≥ 1（時点 0。1 行 JSX。複数行なら `rg -A2 "<ChevronDown"` で確認して報告）/ `rg -c "not.toHaveClass\(\"bg-row-current\"\)" src/features/suppliers/components/SupplierPickerDialog.test.tsx` ≥ 1（時点 0、C17 test）/ `awk '/^## ⑧/,/^## ⑨/' docs/design-system/02-component-catalog.md | rg -c "白地の箱"` ≥ 1（時点 0）/ 同 `rg -c "帯も同じ|帯にも同じ"` ≥ 1（時点 0、未解決時文言 Opus #20）/ 同 `rg -c "text-muted-foreground"` ≥ 1（時点 0、chevron 色 Opus #19）/ 同 `rg -c "高さ・背景・境界で差別化"` = 0（時点 1、Opus #17）

## Design Sources

- Requirements / spec: `docs/function-design/77-ui-bulk-price-revision.md`（SPEC-PRV-D3 / D6、REQ-105 / 106）、`docs/function-design/51-ui-product-form.md`（UI-01b-D7 / D8 / D21）、`docs/function-design/61-ui-receiving.md`（UI-02-D3）、`docs/function-design/78-ui-supplier-management.md`（SPEC-SUP-D2 / D9）
- Architecture: `docs/ARCHITECTURE.md`（UI 層のみ）
- Function / command / DTO: 変更なし（`list_suppliers` / `create_supplier` 既存）
- DB: 変更なし
- Screen / UI: `docs/design-system/01-decision-rules.md`（DSR-24 / DSR-22 / DSR-01）、`docs/design-system/02-component-catalog.md`（⑧ picker dialog 小節、⑥ EmptyState、⑨、⑯ ListShell）、`docs/design-system/reference/mockup-f-supplier-picker.html`
- Decision log / ADR: 追加なし

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 変更なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 変更なし | existing sufficient |
| DB / transaction / audit / rollback / migration | 変更なし | existing sufficient |
| Screen / UI / route state / Japanese wording | DSR-24 / catalog picker 小節 / mockup-f / 51 / 61 / 77 / 78 | updated in this PR（D1 = 51 `:151` の canonical path 規範を書き換え、D5 = catalog `:584` に trigger 現在値表示の規範を追加。他は canonical 行・stale citation・変更履歴の同期） |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | 該当なし（D1 は 51 の path 記述更新） | existing sufficient |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| 該当なし | Tauri command / route / function-design doc / REQ の新設なし。`bindings.ts` / routeTree / 90-traceability の再生成は不要（L1 full の生成系検査 3 種は差分なしで PASS する想定） |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| DSR-24 | 01-decision-rules `:469-479` | DSR-24 | 追加導線を伴う master 参照は picker dialog。却下 = Select 横 primary 追加ボタン | S1 / S3 / S4 / S5 | `SupplierPickerDialog.test.tsx`、各 host test |
| catalog ⑧ picker dialog | 02-component-catalog `:578-598` | D2 / D3 / D4 / D6 / D7 | 構成・動作・状態・a11y の literal 転記。SearchBar 不使用は in-memory filter のため | S1 | `SupplierPickerDialog.test.tsx` |
| catalog ⑧ 重ね契約 (A) | 02-component-catalog `:588-590` | D10 | Radix nest、二重 scrim 許容。(B) は L3 破綻時のみ | S1 + S2 | jsdom test（P1）+ AC-L3-5 |
| UI-01b-D7 / D21 | 51 `:21,36,149-156` | D1 / D8 | canonical `CreateSupplierDialog` 1 実装、inline パネル撤去 | S2 / S4 | `ProductForm.test.tsx` |
| UI-01b-D8 | 51 | — | 取引先未指定で保存可能（不変） | S4 | 既存 test 維持 |
| SPEC-PRV-D3 / D6（REQ-105 / 106） | 77 `:27,30,94` | — | toggle は filter 列残置、『すべての取引先』先頭 | S3 | `PriceRevisionFilters.test.tsx` |
| UI-02-D3 | 61 `:23,155` | — | 入庫記録も picker、未指定保存可 | S5 | `ReceivingPage.test.tsx` |
| SPEC-SUP-D2 / D9 | 78 `:28,55-59,146-158` | — | 名前検索 + scroll 箱、footer なし、全件取得 | S6 | `SupplierManagementPage.test.tsx` |
| DSR-22 現在行 3 点 | 01-decision-rules DSR-22 | D7 | 色だけに頼らない現在行 | S1 | `SupplierPickerDialog.test.tsx`（✓ + badge 文言） |
| DSR-01 | 01-decision-rules `:25` | D9 | 二重 primary 疑いの対象が消える | S7 | AC10 |
| trigger 表示 | mockup-f 背景（規定なし） | D5 | 現在値を出す。却下 = 固定文言「取引先を選択」（閉じた状態で現在値不明） | S3 / S4 / S5 | AC9 + AC-L3-6 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（DSR-24 / catalog picker 小節 / 51 / 61 / 77 / 78 が ⑱ で確定済み）。D5 trigger の現在値表示のみ canonical に規定が無く、S7 で catalog `:584` に 1 句追記して source 化する
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D1（canonical path）→ 51 `:151`、D5 → catalog `:584`
- Assumptions and constraints: Radix `DismissableLayer` の nested ESC / outside-click 挙動（P1）、WebView2 での focus trap（AC-L3-5）
- Deferred design gaps, risk, and follow-up target: query key 併存（非目的、Backlog 既存なし → closeout で起票判断）、(B) fallback
- Test Design Matrix can cite design decision IDs or source doc sections: yes
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 取引先未指定での保存（UI-01b-D8 / UI-02-D3）は trigger + picker でも `null` 選択で維持。既存 `Select` の disabled 条件は trigger が継承

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable（UI 層のみ） | — |
| Fact check / design decision split | not applicable | — |
| Lifecycle / retry | 取得失敗の再試行を picker 内 Alert に一本化。host 側 alert（`PriceRevisionFilters.tsx:158-170`）は撤去（Coordinator 決定、S3 / round 1 Opus P2-5） | Matrix C8 / F5 行 + Review Focus |
| Operator workflow | 取引先選択が 1 click（Select 展開）→ 2 click（trigger → 行）に増える代わりに検索が付く。owner は ⑱ で「80 社の切替は検索の方が速い」と判断済み | AC-L3-1 |
| Replacement path | (A) 破綻時は (B) 2-stage へ Gated Amendment | AC-L3-5 |
| Data safety / evidence | not applicable（DB 書込みは既存 `create_supplier` のみ） | — |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | dialog 重ね (A) は WebView2 実機のみが oracle | AC-L3-5 |
| 環境・再現性 | 新設の環境依存なし | — |

## Design Readiness

- Existing design docs are sufficient because: ⑱ が DSR-24 / catalog / mockup / 4 function-design を確定済み。本 packet の D1〜D10 は実装上の配置・props・test 戦略で、設計の再導出ではない
- Source docs updated in this PR: S7（catalog `:582` `:584`、DSR-01 `:25`、51 `:151` + 変更履歴、61 / 77 / 78 変更履歴）
- Design gaps intentionally deferred: (B) fallback、query key 統一、取引先管理の sort / paging
- Durable decisions discovered in this plan and promoted to source docs: D1 / D5

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI のみ。CMD 以下不変
- Backend function design: 不変
- Command / DTO / data contract: 不変（`Supplier` DTO をそのまま使う）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 「取引先を選択」「現在の選択」「選択中」「新しい取引先を追加」「閉じる」「取引先名で検索」「該当する取引先はありません」（catalog / mockup / 78 の literal）
- Error, empty, retry, and recovery behavior: 取得失敗 = picker 内 Alert + 再試行、検索 0 件 = EmptyState、追加失敗 = `CreateSupplierDialog` 既存のエラー表示（不変）
- Testability and traceability IDs: SPEC-SUPPICK-RT-1、AC1〜AC12、AC-L3-1〜6

## Contract Probe

- **P1** Radix `Dialog` を nest したとき ESC / 外クリックが最前面だけを閉じ、内側 close 後に focus が picker 内へ戻る（catalog `:589` の「想定」）: Writer が S1 の test で jsdom 上で実演（`user.keyboard("{Escape}")` で内側のみ閉じる、`document.activeElement` が検索 input）→ 結果を Implementation Results に記録。jsdom で再現不能なら「L3 のみ」に落とし理由を残す（D10）。**実機 oracle は AC-L3-5**
- **P2** `DialogContent` が `onCloseAutoFocus` / `onOpenAutoFocus` を透過する（`dialog.tsx:59` `...props`）: 起票時に Coordinator が確認済み（透過）→ 実装で利用可
- **P3** testing-library の `getByLabelText("取引先")` が 2-id の `aria-labelledby="<label id> <trigger id>"` を持つ `<button>` を解決する（DTL `getLabels` は id ごとに label entry を作り、複数 label 時は結合文字列と各 label 単体の両方で matcher を回す = 解決する見込み。round 2 Opus の読解、`node_modules` 不在で未実行）: Writer が S4 着手時に SC4e `ProductForm.test.tsx:794` を trigger 化した状態で 1 回実行して先に確認する。解決しない場合は SC4e / SC4b の matcher を `/取引先/` へ緩めることを最終報告で提案し、Coordinator が Gated Amendment で裁定する

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| DSR-24 picker 統一（3 host） | S3 / S4 / S5 | AC4 / AC5 / AC9 + 各 host test | AC-L3-1〜3 |
| catalog 構成（検索 / 固定帯 / 一覧 / footer） | S1 | AC7 + `SupplierPickerDialog.test.tsx` | AC-L3-1 |
| catalog 動作（行クリック閉じる / Esc・外クリック / A 案） | S1 | AC8 + test | AC-L3-2 / 5 |
| catalog 重ね契約 (A) | S1 + S2 | P1 jsdom test | AC-L3-5（正本） |
| catalog 状態（Skeleton / Alert / EmptyState） | S1 | test 3 本 | non-scope（L3 は 0 件のみ AC-L3-4） |
| catalog a11y（Title / Description / 初期 focus / 復帰） | S1 | test（`document.activeElement`） | AC-L3-5 |
| DSR-22 現在行 3 点 | S1 | test（✓ + 「選択中」文言） | AC-L3-1 |
| UI-01b-D7 / D21（canonical 1 実装、inline 撤去） | S2 / S4 | AC2 / AC3 + `ProductForm.test.tsx` | AC-L3-2 |
| UI-01b-D8 / UI-02-D3（未指定で保存可） | S4 / S5 | 既存 test 維持 | AC-L3-3 |
| SPEC-PRV-D3 / D6（toggle 残置 / 先頭行） | S3 | AC4 + `PriceRevisionFilters.test.tsx` | AC-L3-1 |
| SPEC-SUP-D2 / D9（検索 + scroll 箱） | S6 | AC6 + `SupplierManagementPage.test.tsx` | AC-L3-4 |
| D3 name 昇順 | S1 | test（順序 assertion） | non-scope |
| D5 trigger 現在値 + `aria-labelledby` | S3 / S4 / S5 | AC9 + host test（`toHaveTextContent`） | AC-L3-6 |
| Lane 5 SC4e / SC4b control 面 token（取引先欄 = `bg-control-surface`、not `bg-background`） | S4 / S5（trigger className） | `ProductForm.test.tsx:763-800` / `ReceivingPage.test.tsx:1027-1032` in-place | non-scope |
| catalog `:584` footer の配分（左 primary / 右 outline） | S1 | C1 test（`data-variant`） | AC-L3-2 |
| ⑭ GA2 取引先 label / 操作の群化（1 wrapper + DOM 順序） | S3 | `PriceRevisionFilters.test.tsx` 2 本を trigger 版へ書き換え | AC-L3-1 |
| Lane 5 SC6（取引先欄は `suppliersQuery` loading 中 disabled）+ page 側の取引先選択 / 追加 flow / toggle（REQ-105 / UI-14） | S3 | `PriceRevisionPage.test.tsx:166-204,523-575,677-690` を picker 経由へ in-place 書き換え（Gated Amendment 2） | AC-L3-1 |
| 取得失敗の再試行を picker に一本化 | S3 | `PriceRevisionFilters.test.tsx`（host alert 不在 + picker Alert） | non-scope |
| D6 scroll 箱（plain div、STICKY 不使用） | S1 / S6 | C1 test（帯・footer が scroll 箱の外） | AC-L3-2 / 4 |
| D9 DSR-01 解消 / S7 docs | S7 | AC10 | non-scope |

## Test Plan

Test Design Matrix: [test-matrices/2026-09-10-supplier-picker-runtime.md](test-matrices/2026-09-10-supplier-picker-runtime.md)
- If the Human Gate includes L3, Writer completion includes `cargo check --release` before the owner native build; this is not a CI gate（frontend のみだが慣行維持）

- targeted tests: `SupplierPickerDialog.test.tsx`（新設、契約 10 項目、test 名に governing ID）/ `PriceRevisionFilters.test.tsx`（GA2 2 本の trigger 版 + 新規「toggle が filter 列に既定 on」）/ `ProductForm.test.tsx`（inline → picker flow、SC4e in-place）/ `ReceivingPage.test.tsx`（Select → picker、SC4b in-place）/ `SupplierManagementPage.test.tsx`（検索 + 0 件）
- negative tests: 空白名は `CreateSupplierDialog` 側で拒否（既存）/ 取得失敗時に picker の一覧が出ず Alert + 再試行 / 検索 0 件で EmptyState、検索クリアで復帰 / Esc・外クリックで `onSelect` が呼ばれない
- compatibility checks: 取引先未指定のまま保存できる（ProductForm / Receiving の既存 test、`ProductFormPage.test.tsx:127`「allows no-supplier create when supplier options fail」= `supplierWarning` で trigger が disabled のまま保存できること）/ toggle 既定 on（`PriceRevisionPage.test.tsx:166-204` を picker 経由に書き換えて維持）/ `SupplierManagementPage` の追加 flow が引数変更後も動く / `ReceivingPage.suggest.test.tsx` / `ProductFormPage.test.tsx` の部門・候補 listbox の `option` は無関係で不変
- data safety checks: 該当なし
- main wiring/integration checks: 該当なし（route / DTO 不変）

## Boundary / Wire Contract

該当なし。Tauri command / DTO / bindings / DB / CSV / route state を変更しない。`CreateSupplierDialog` の `onCreated` 引数変更は TS 内部契約（compile で検出）。

## Review Focus

- picker が fetch せず host の query 状態を受ける構造（D2）が 3 host で守られ、追加後の再取得が各 host の既存 key で行われること
- `CreateSupplierDialog` の canonical 1 本化後に、直接 import が `SupplierPickerDialog.tsx`（nest）と `SupplierManagementPage.tsx` の 2 箇所だけで、3 host（`PriceRevisionFilters` / `ProductForm` / `ReceivingPage`）から `CreateSupplierDialog` の参照が消えていること（AC2、Gated Amendment 1）
- `ProductForm` から inline パネル関連 state / handler が完全に消え、`supplierWarning` disabled が trigger に移っていること
- picker footer が scroll 箱の外で固定（`shrink-0`）、固定帯が scroll 箱の外にあること（catalog `:584` literal）
- 内側 dialog の overlay が既定のまま（二重 scrim、`bg-black/50` を消していない）
- `＋` 全角文字が無く lucide `Plus` を使っていること
- 各 host test で `Select` 依存の assertion（`getByRole("combobox")` 等）が picker 経由（`getByRole("button", { name: /取引先/ })` → dialog 内 `getByRole("row"|"button")`）へ置換され、旧挙動を語る test 名 / comment が残っていないこと
- S7 の docs 変更が canonical 行 / stale citation / 変更履歴の同期と、規範追加 2 点（D1 = 51 `:151` canonical path、D5 = catalog `:584` trigger 現在値 1 句）に限られ、DSR-24 本文や picker 小節の他の設計文を書き換えていないこと
- `PriceRevisionFilters.tsx` の host 側 alert 撤去後も部門 alert `:171-183` が残り、picker の `isError` 分岐で再試行が `suppliersQuery.refetch` を呼ぶこと
- [T4]（`generate_traceability.rs:557` の regex `\b(REQ-[0-9]{3}\b|UI-[0-9]{2}[a-z]?\b)`、file 単位 presence）の観点で、`SupplierPickerDialog.test.tsx` が `UI-01b` / `UI-02` を含む test 名を最低 1 本持ち、`PriceRevisionFilters.test.tsx` に `REQ-NNN` / `UI-NN` が**増えていない**こと（AC11 の 2 oracle）。`DSR-24` / `SPEC-*` は [T4] 上は無効
- picker の `EmptyState` に action / 再検索 button を渡していないこと（catalog `:592`「⑥ の絞り込み解除 action は置かない」）

## Spec Contract

Contract ID: SPEC-SUPPICK-RT-1

- ⑱ が確定した DSR-24 / catalog picker dialog 小節 / 51 / 61 / 77 / 78 の契約が runtime（`src/features/suppliers` / `products` / `receiving`）へ反映され、`CreateSupplierDialog` が `onCreated(supplier)` 契約の 1 実装になり、canonical docs の「後続実装」表記と DSR-01 の stale citation が是正される

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-SUPPICK-RT-1 | S1 | `SupplierPickerDialog.test.tsx` | 構成 / 動作 / 状態 / a11y / 重ね (A) | vitest + AC7 / AC8 |
| SPEC-SUPPICK-RT-1 | S2 | `SupplierManagementPage.test.tsx`（追加 flow）+ typecheck | canonical 1 実装 | AC2 |
| SPEC-SUPPICK-RT-1 | S3 | `PriceRevisionFilters.test.tsx` | toggle 残置 / 先頭行 / trigger | AC4 / AC9 |
| SPEC-SUPPICK-RT-1 | S4 | `ProductForm.test.tsx` | inline 撤去 / 未指定保存 / A 案 | AC3 / AC9 |
| SPEC-SUPPICK-RT-1 | S5 | `ReceivingPage.test.tsx` | 指定なし / 未指定保存 | AC5 / AC9 |
| SPEC-SUPPICK-RT-1 | S6 | `SupplierManagementPage.test.tsx` | 検索絞り込み / 0 件 | AC6 |
| SPEC-SUPPICK-RT-1 | S7 | docs review | literal 同期のみ | AC10 |
| SPEC-SUPPICK-RT-1 | 全体 | owner L3 | dialog 重ね (A) 実機 | AC-L3-1〜6 |

## Data Safety

- what must not be committed: なし
- local-only paths: `.local/codex-orders/**`（発注書・log）
- synthetic-only paths: 該当なし

## Writer Instructions

- Codex `model_reasoning_effort=medium`（難所 = S1 の dialog 重ね test〈P1〉と S4 の ProductForm 置換。Coordinator 判断で high 昇格可）。commands 実行数と手戻り回数を PR body に記録する
- 着手前: `git status --short` 空を確認 → `git checkout --detach <遷移 commit SHA>` → `git rev-parse --short HEAD` を報告に書く。packet の Scope / Spec / AC / Matrix 契約行 / Workflow State / Ledger / 結果記入は一切編集しない（字面に疑義があれば最終報告で指摘）
- worktree: `npm ci --ignore-scripts` → `npm run generate:routes`。push 前に `npm run format:check` + lint + typecheck + targeted tests。`bash scripts/local-ci.sh full` は **1 worktree 1 run（同時走行禁止。回数上限ではない）**。`git add` は明示パスのみ（`-A` / `.` 禁止）。`HEAD:branch` 形式の push 禁止。完了後に worktree を自分で detach する。commit subject は conventional prefix、body は日本語可
- 各 S の設計意図（WHY）は Scope / 設計判断 D1〜D10 に書き込み済み。Writer は再導出せずそのまま従う。canonical の文言（catalog `:584-594`）を UI literal の正本とする
- 既存 test の assertion を書き換えるときは、旧挙動を語る test 名 / comment も同じ commit で直す（⑮ Q3 の一般規則）。既存 test の削除・skip は不可。inline パネル固有の test は picker flow の test に**置換**する（削除ではない）
- **[T4] baseline の維持（round 2 Opus P1-1 で機構を訂正）**: `src-tauri/src/bin/generate_traceability.rs:557` は regex `\b(REQ-[0-9]{3}\b|UI-[0-9]{2}[a-z]?\b)` の **file 単位 presence** で「ID 未参照 FE test file」を数え、`:54` の `FE_UNREFERENCED_BASELINE = 26` と増減両方向で突合する（`src-tauri/**` は Non-scope、baseline は変えない）。`DSR-24` / `SPEC-*` はこの regex に一致せず [T4] 上は無効。したがって (a) 新規 `SupplierPickerDialog.test.tsx` は **`UI-01b` または `UI-02` を含む test 名を最低 1 本**持つこと（Matrix の `UI-01b-D21:` 行で足りる。他の test 名の `DSR-24:` 接頭辞は可読性のためで [T4] には寄与しない）→ 26 のまま。(b) `PriceRevisionFilters.test.tsx` は「REQ/UI ID を付けない」前提で baseline に計上されている（header `:9-11`）ため、GA2 書き換え・新規 toggle test・header comment のいずれにも **`REQ-NNN` / `UI-NN` を一切書かない**（`SPEC-PRV-D3:` / `SPEC-PRV-D6:` は可）→ 書くと 26 → 25 で ERROR。(c) `ProductForm.test.tsx` / `ReceivingPage.test.tsx` / `SupplierManagementPage.test.tsx` は既に referenced で影響なし
- 内側 `CreateSupplierDialog.tsx:46-53` は `await onCreated()` の後に `toast.success` → `setName("")` → `onOpenChange(false)` → `setPending(false)` を実行する。A 案では `onCreated` 中に picker が閉じ内側が unmount されるため、jsdom で act 警告が出ることがある。出た場合は既存 `SupplierManagementPage.test.tsx` の `waitFor` 方針で吸収し、`CreateSupplierDialog` 側の順序は変えない（round 2 Opus P3-4）
- `rg -c <pattern> <file>` は 0 件のとき **出力なし・exit 1** になる（literal `0` は出ない）。oracle 表では空出力を 0 として記録する。scripted に使うなら `|| echo 0`
- 統合後の `CreateSupplierDialog` は `sonner` の `toast.success` を呼ぶ。`ProductForm.test.tsx` / `ReceivingPage.test.tsx` で toast 由来の警告や act 警告が出たら既存 `SupplierManagementPage.test.tsx` と同じ mock 方針に揃える
- ponytail block（実装原則、以下を verbatim で発注書に注入）:

```
### 実装原則（ponytail、full）
書く前に順に問う: (1) そもそも要るか（推測需要は書かず 1 行で理由）(2) この codebase に既に helper / util / pattern があるか（再実装しない）(3) 標準ライブラリで済むか (4) platform の素の機能で済むか（<input type="date">、CSS、DB 制約）(5) 導入済み依存で済むか（数行のために新規依存を足さない）(6) 1 行で済むか (7) それでも要るなら動く最小 code。
規則: 実装 1 つの interface / 製品 1 つの factory / 変わらない値の config を作らない。将来用の scaffold を作らない。追加より削除、賢さより退屈さ。file 数は最少、動く最短 diff（ただし問題を理解してから。正しい場所の小さな変更 > 間違った場所の最小変更）。同サイズの選択肢は edge case に正しい方。意図的に角を落とした箇所（上限のある近似・O(n²)・global lock 等）は `ponytail:` comment を残す。
例外: 正しさ・データ安全・既存 test の契約・packet の AC を削る方向には使わない。
```

- 具体的な適用: picker の scroll 箱は `ListShell.tsx` の class を流用し新規 class 系統を作らない（D6）。検索は `Input` 直置き（D4）。picker は fetch も queryClient も持たない（D2）。`CreateSupplierDialog` は suppliers 版の本体を残し型だけ変える（D1）。trigger は `Button variant="outline"` 1 個で、新規 component（`SupplierTrigger` 等）を作らない（D5、3 host で同じ 5 行程度なら重複を許容し、rule of three を超えて共通化したくなったら最終報告で提案するに留める）
- Maintainability review lens（owner 決定 2026-09-07）: 命名 / 理由 comment / 退屈な構造 > 賢い圧縮。D3 の並び替え・D8 の local 再取得・S3 で host 側 alert を残す / 消す判断には短い理由 comment を残す

## Implementation Results

Codex Writer（medium、2026-09-10〜11、発注書 36 → 36b → 36c で完走、是正 42）: 取引先の選択と追加を `SupplierPickerDialog` に統合し、`CreateSupplierDialog` を suppliers 配下の 1 本に集約、3 host（一括価格改定 filter / 商品登録 form / 入庫記録）を trigger + picker 経由に置換、取引先管理に検索 input と scroll 箱を追加。Final Review round 1 の是正で現在値の解決を `supplierCurrentLabel` 1 関数に集約（未解決時は「取引先を確認できません」）、picker の submit 防壁に理由 comment と test、取引先管理の外枠撤去、filter 行の trigger 幅固定。S7 で catalog ⑧ 小節 / DSR-01 / 4 function-design を同期。PR: https://github.com/higekiri-w/inventory-system-desktop/pull/50

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Fill after review.
If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
- Findings Freeze: frozen after Plan Review round 3（2026-09-10、Sonnet approve + Opus approve、P1/P2 = 0）; post-freeze exceptions: none.

### Plan Review round 1（2026-09-10、plan-first `808d8d6`、Sonnet + Opus 独立）

- Opus: reject（P1 4 / P2 8 / P3 7）。Sonnet: approve-with-P2（P1 2 / P2 1 / P3 3、うち Sonnet P1-1 = Opus P1-4〈AC10 `:514`〉、Sonnet P2-1 = Opus P2-7〈AC2 相対 import〉の 2 件が重複）。両報告の全文は PR #50 body「Plan Review round 1」節に転記（`gh pr view 50 --json body`）
- Coordinator 実証: P1-1 `setSupplierOptions` は `ProductForm.tsx:97,104` の prop merge でも使用 → AC3 pattern から除外 / P1-2 `generate_traceability.rs:54` `FE_UNREFERENCED_BASELINE = 26` [T4] → test 名に ID 付与で baseline 不変 / P1-3 `ProductForm.test.tsx:792-800` SC4e・`ReceivingPage.test.tsx:1027-1030` SC4b が `bg-control-surface` を assert → D5 に token を組み込み in-place 成立 / P1-4 = Sonnet P1-1 `:514` の無関係「後続実装」→ AC10 oracle を scope
- 裁定: **accept 13**（Opus P1-1〜4 / P2-1〜8、Sonnet P1-2〈Matrix C9 の不在 test〉、P3 全件）/ no-action 0。Opus P2-8 の「D5 は Human Gate 不要」に同意（mockup-f の trigger は背景の非対話 placeholder）
- 是正 = `bcecd31` + `36ab65f`（D5 / D6 / S2 / S3 / S4 / S5 / S6 / AC2 / AC3 / AC10 / AC-L3-2 / Required Design Artifacts / Review Focus / Ledger 5 行 / Writer Instructions 3 項 / 起票時実測 / Test Plan / Matrix）。round 2 = Sonnet + Opus 再注入（round 1 是正の巻き込み確認）

### Plan Review round 2（2026-09-10、`36ab65f`、Sonnet + Opus 独立）

- Opus: reject（P1 1 / P2 3 / P3 4）。Sonnet: approve-with-P2（P2 3 / P3 1）。closure 13 件中 12 件 closed、#2（[T4]）のみ not closed。両報告の全文は PR #50 body「Plan Review round 2」節に転記（`gh pr view 50 --json body`）
- Coordinator 実証: [T4] の regex は `generate_traceability.rs:557` `\b(REQ-[0-9]{3}\b|UI-[0-9]{2}[a-z]?\b)` の file 単位 presence → `DSR-24` / `SPEC-*` は無効、`PriceRevisionFilters.test.tsx` は unreferenced として baseline 26 に計上 / 3 host の Label は `htmlFor` のみで `id` なし / SC4b の assertion は `ReceivingPage.test.tsx:1031-1032` / Impact lens「Writer 判断」が S3 の決定と矛盾（rg sweep 漏れ）
- 裁定: **accept 12**（Opus P1-1 / P2-1〜3 / P3-1〜4、Sonnet P2-1〜3 / P3-1）/ no-action 0
- 是正 = 本 commit（Writer Instructions の [T4] 機構訂正 + act 警告注記 / AC11 に [T4] oracle 2 行 / D5 に Label id 新規付与 + Contract Probe P3 / S5 `:1027-1032` + `async` 形 / Ledger footer 配分行 / Impact lens / Non-scope 衝突範囲 / Review Focus 3 項 / `:158-170` `:171-183` / Matrix〈C1 `data-variant`、C8/F5 host alert 行、SC4b 範囲〉/ round 1 記録の重複 ID と「3 項」）。round 3 = closure（Sonnet + Opus、diff 限定）

### Plan Review round 3 closure（2026-09-10、`f13eedc`、Sonnet + Opus 独立）

- Sonnet: approve（blocking なし。AC oracle・引用・Ledger ⇄ Matrix ⇄ AC・Plans.md・PR body すべて一致）。Opus: approve（closure 1〜7 全 closed、[T4] 26 / 170 を独立再現、Matrix test 名の [T4] 適合を PCRE2 で実測、P3 2 = SC4e の assertion 範囲表記の非対称 / 01-decision-rules 更新履歴の merge-train 注記）。両報告の全文は PR #50 body「Plan Review round 3」節に転記（`gh pr view 50 --json body`）
- 裁定: accept 1（P3-1、本 commit で `:763-800`〈assertion `:792-800`〉に統一）/ no-action 1（P3-2、Non-scope の記述は Plans.md wave 9 行と同じ範囲で正しく、01-decision-rules の更新履歴は ⑳ が触らない）
- **Findings Freeze**（round 3 / 天井 3）。次 = Plan Commit 記入 + state-only 遷移 `plan-draft->plan-gate->plan-approved->implementing` → Codex 発注書 36

### Gated Amendment 1（2026-09-10、Codex 36 fail-closed 停止起源）

- Codex 36（session `01a08abc-b569-7f33-a67b-7e72323fc108`、commands 24 / 手戻り 0、commit 0、`/tmp/codex-19` detach 済み）が実装前に停止: S2 / AC2 が `PriceRevisionFilters.tsx` に `CreateSupplierDialog` の直接 import を要求する一方、S3 が同 file の dialog 描画を picker に置き換えるため、import が未使用になり `tsconfig.json` `noUnusedLocals: true` と両立しない。Writer の指摘は正当（Plan Review 3 round が見逃した契約不整合。Writer が自力で packet を書き換えず停止したのは発注書の規律どおり）
- 是正（本 commit、post-Freeze の契約変更 = Gated Amendment）: S2 の import 付け替え文を「host から撤去、直接 import は `SupplierPickerDialog.tsx`（nest）と `SupplierManagementPage.tsx` の 2 箇所」へ / AC2 第 3 oracle の検査先を picker へ + host 3 file の参照 0 を負 oracle に追加 / Review Focus の「4 呼び出し元がすべて import」を訂正。Scope の他項・D1〜D10・Matrix は不変
- 教訓: import 契約は「誰が描画するか」と一体で決まる。発注前に Coordinator が「各 file が何を import するか」を 1 表にして S 間の整合を機械的に確認する（memory へ）

### Gated Amendment 2（2026-09-10 夜、Codex 36b fail-closed 停止起源）

- Codex 36b（session `01a08ac6-0ea4-7d90-8e07-08827b9982b1`、commands 37 / 手戻り 1、commit 0、`/tmp/codex-19` は `6d827707` で detach 済み・**未 commit の S1〜S6 途中差分 12 file を保持**）が S7 手前で停止: 編集許可範囲外の `src/features/products/PriceRevisionPage.test.tsx` に旧 Select 前提の test 6 本（`role="option"` 選択 / 画面上の「新しい取引先を追加」button / `data-slot="select-trigger"`）があり 6 fail。加えて Writer 自身の新規 `SupplierManagementPage.test` の scroll 箱 selector が 1 階層浅く 1 fail（Writer 側で直す）。D10 の dialog 重ね test は jsdom で成立（内側 Esc / 検索 input focus / trigger focus）、L3 降格なし
- 是正（本 commit）: S3 に `PriceRevisionPage.test.tsx` の 3 区画（`:166-204` / `:523-575` / `:677-690` SC6）を picker 経由の in-place 書き換えとして追加 / Test Plan の互換確認に `ProductFormPage.test.tsx:127` と無関係 file の除外を明記 / Ledger に SC6 + page flow 行 / Matrix に 3 行 + C7 の GA1 前文言（Writer 指摘）を訂正。Coordinator 側の Adjacent Pattern Audit が component test のみで page 統合 test を数えなかったのが原因（`rg -l "取引先" src --glob '*.test.tsx'` で全 test file を当てる sweep を発注前に行うべきだった）
- 再発注: 発注書 36（起点 = 本 Amendment 記録後の tip）、`/tmp/codex-19` の途中差分を引き継ぐ

### Final Review round 1（2026-09-11、content `bbf0edc5`、Sonnet + Opus 独立 + Codex 40）

- Codex 36c（実装 `000ca0cf` + docs `288f4fea` `bbf0edc5`、L1 full PASS、vitest 1,498 pass、AC 43 行 PASS、commands 73 / 手戻り 3。全文 `.local/codex-orders/reports/36c-impl-pr50.md`）
- Sonnet: approve（P3 2 = 対応不要）。AC1〜AC12 全 PASS、Matrix mutant 12/12 killed〈各 mutant を picker test + host test の組で実行〉、fs scan 契約逸脱なし、Review Focus 11/11 PASS。全文 `.local/codex-orders/reports/final-review-r1-pr50-sonnet.md`
- Opus: reject（P1 1 / P2 5 / P3 6）。P1 = 一覧未解決時（fetch 失敗 × 選択済み）に trigger / 固定帯が「すべての取引先」等の未指定文言で断定表示（`PriceRevisionFilters.tsx:77-78` / picker `:52` / `ProductForm.tsx:313-314` / `ReceivingPage.tsx:403`。S3 の host alert 撤去で error 表示ゼロ）。P2 = picker `onSubmit` stopPropagation が唯一の防壁で comment / test なし / 取引先管理 scroll 箱の二重枠 / filter 行 trigger `w-full` の幅追従（D5 literal 起因）/ 「追加する」variant assertion の無 replacement 削除 / GA2 rationale の圧縮。P3 = SC4i test 名「取引先select」/ `border-l-primary` 未 assert / import 順 / `tr` onClick 二重経路 / `parentElement` 3 連鎖 / `onOpenAutoFocus` の query reset。D1〜D10 写像・canonical ⑧ 小節・mockup-f 7 状態・S7 docs は上記以外すべて一致。全文 `.local/codex-orders/reports/final-review-r1-pr50-opus.md`
- Codex 40（review、medium、commands 44、PR #50 review comment `5170190713`）: 新規 findings なし。mutant 12 本を独立注入し全件 kill、AC 44 行独立再測定で全一致、[T4] 26 維持、`merge-tree` 衝突 = `docs/Plans.md` のみ。Findings Freeze 可（既存 accept 済み P1 / P2 の修正完了は意味しない）
- Coordinator 実証: P1 = `find(...)?.name ?? "すべての取引先"` を code で確認 / P2-2 = `CreateSupplierDialog` は `preventDefault` のみ、`ProductForm` の form は無条件に `onSubmit()` / P2-3 = 外 `rounded-md border` × `SupplierUsageTable:15` の同 class / P2-5 = `secondary` assertion 2 行削除を diff で確認 → **accept: P1-1 / P2-2〜6 / P3-7・8・9・11、no-action: P3-10・12**（canonical 非違反、記録のみ）

### Gated Amendment 3（2026-09-11、Final Review round 1 Opus reject 起源）

- 是正（本 commit）: D5 に (a) 現在値解決の 1 関数化 `supplierCurrentLabel` + 未解決時「取引先を確認できません」〈trigger は isError でも有効、host alert は戻さない〉(b) filter host のみ `w-48` / D6 に取引先管理の外枠撤去 / **AC13**（oracle 15 本）/ Matrix C14・C15 + mutant 13・14。Writer は packet を編集しない
- 再発注: 発注書 42（起点 = 本 Amendment 記録後の tip）。Freeze 後のため closure round 2 は AC13 + mutant 13・14 + P2〜P3 の closure 確認に限定

### Final Review round 2 closure（2026-09-11、content `0769d9ee`、Sonnet + Opus 独立）

- Codex 42（是正、起点 `95dbcbfe`、`d929028c` + `0769d9ee`、commands 57 / 手戻り 1、AC13 15 oracle 全 PASS、mutant 13・14 kill、L1 full PASS〈END_HEAD_SHA 一致〉）。申し送り = Matrix Negative Paths の旧 `leadingLabel` fallback 文言 → 本 commit で訂正
- Sonnet: approve（新規 findings なし。AC13 15/15、mutant 13・14 kill を独立再現、accept 済み #1〜#9・#11 すべて解消、no-action #10・#12 は無変更、vitest 171 file / 1,500 test green、[T4] 26 維持、footprint 11 file = 発注書 42 の許可範囲）
- Opus: approve（新規 P3 3 + 観察 1、いずれも Freeze 後の follow-up）。#13 = filter host の `w-48` trigger は `Button` に `truncate` が無く長い取引先名が枠外へはみ出す（旧 `SelectTrigger` は `line-clamp-1`。GA3 (b) の literal 側の穴）/ #14 = `PriceRevisionFilters.tsx` の `@/features` import だけ相対 import の後ろのまま（#9 の名指し外）/ #15 = catalog `:584` の追記は起動ボタンのみで、固定帯の未解決時表示（「取引先を確認できません」+「選択中」badge）が canonical 未規定 / 観察 = 商品修正で取引先設定済みの商品を開くと suppliers 取得中に trigger が「取引先を確認できません」を一瞬表示しうる（`useProductFormOptions` が `data ?? []`、`ProductForm` は `isLoading={false}` 固定）。#1 (iii) は 3 host 中 2（`ProductForm` は取得失敗時 trigger disabled + host Alert で代替、GA3 (a) の「disabled 条件は不変」に準拠）
- Coordinator 裁定: #1 (iii) = closure 済み（`ProductForm` の host Alert が error 表示を担う設計、GA3 literal 準拠）/ **#13・#14・#15 = accept、owner L3 後の是正 round に同乗**（L3 で見える差分と 1 往復にまとめる。#13 = 3 host の trigger 中身を `<span className="truncate">` で包む / #14 = 1 行移動 / #15 = catalog `:584` に「固定帯も同じ」1 句）/ **観察 = owner L3 AC-L3-6 で「商品修正を取引先設定済みの商品で開いた瞬間」を観測項目に追加**、見えたら同 round で `ProductForm` に `isLoading` を配線
- state-only（本 commit）: `implementing->local-verified->independent-review->human-confirm`、Reviewed Content HEAD `0769d9ee`。local-verified の証拠 = Codex 42 の L1 full（Writer 報告）、independent-review = Sonnet + Opus round 1〜2 + Codex 40（Freeze 可）。次 = owner L3（AC-L3-1〜6 + 追加観測 3 点）

### Gated Amendment 4（2026-09-11、owner L3 run 1 起源）

- owner L3 run 1（content `0769d9ee`）: AC-L3-1 / 2 / 3 / 5 PASS、AC-L3-4 は検索 PASS + 0 件文言・箱内 scroll を自動 test 担保に切替（owner 選択）、AC-L3-6 は是正要。原文要旨: 「現在の選択セクションが微妙に見える。左端に帯を付けるか？選択中バッジを帯にも付けるのはしつこくないか？下線だけで上左右はクリーム色の枠だけなのは見やすさに欠ける」「取引先のやつパッと見『指定なし』としか書いてなくてボタンなのか何なのか分からない、dialog が開くことを想起させるアイコンを用意すべき」「（長い取引先名は）めちゃはみ出してる」「（取得中フラッシュは）見えない」
- 是正（本 commit）: D5 に (c) `ChevronDown` + (d) `truncate` / D7 に帯の全周枠 + 左バー + badge 撤去 / Human Gate 欄に run 1 記録 / AC-L3-4・6 追記 / **AC15**（oracle 9 本）/ Matrix C16 + mutant 15・16。Final Review round 2 Opus #13（truncate）/ #14（import 順）/ #15（帯の未解決時 = badge 撤去で解消）を同乗。取得中フラッシュ（Opus 観察）は owner が見えず no-action
- 再発注: 発注書 44（起点 = state-backtrack 後の tip）→ closure round 3（Sonnet + Opus、AC15 + mutant 15・16 + 回帰に限定）→ state-only → owner L3 run 2（AC-L3-1 はみ出し / AC-L3-6 icon・帯のみ）→ Ready

### Final Review round 3 closure（2026-09-11、content `be71a857`、Sonnet + Opus 独立）

- Codex 44（是正 r2、起点 `ccedd98c`、`ded1a787` + origin/main 単段 merge `be71a857`〈⑳ 完了分、runtime 差分 0〉、commands 74 / 手戻り 2。AC15 9/9、mutant 15・16 kill、L1 full PASS〈END_HEAD_SHA 一致〉。申し送り = 帯の class 順で `prettier-ignore` / merge 直後の初回 full が CI 自己テストの時刻逆行で FAIL → 別 worktree で PASS）
- Sonnet: approve（新規なし。AC15 9/9 独立再現、mutant 15・16 kill〈16 は帯 test 3 本〉、owner 所感の反映 7 項目 PASS、vitest 171 file / 1,501 test green、[T4] 維持、footprint 7 file、merge 混入なし）
- Opus: approve（新規 P3 5、Freeze 後の follow-up）。#17 = catalog `:584` 帯の句「高さ・背景・境界で差別化」が背景・左バー共有と矛盾 / #18 = `prettier-ignore` の理由 comment が実態（AC15 oracle の class 隣接）とずれ、repo 初の ignore / #19 = chevron の色が `SelectTrigger`（`text-muted-foreground`）と違い `currentColor` / #20 = 帯の未解決時表示が catalog 未規定（#15 残余）/ #21 = filter 既定状態（未選択）で帯と先頭行「すべての取引先」が同じ背景 + 左バーで 2 段続く。A(i) `ChevronDown` は妥当（代案 `Search` / `Ellipsis` は劣る）、(ii) `truncate` で `min-w-0` 不要、(iv) badge 撤去後の accessible な意味は十分。mockup-f との差分 8 点を列挙（Backlog 用）
- Coordinator 裁定: #17 / #18 / #20 = accept、**owner L3 run 2 の是正 round に同乗**（#18 = AC15 oracle を順序非依存〈`border-l-4` / `border-l-primary` 各 rg〉に改め `prettier-ignore` を外す）/ #19・#21 = **run 2 で owner が判定**（#21 の事前案 = 帯の背景を外す / 帯の左バーを外す / 先頭行が現在行のとき帯を出さない、の 3 択）/ C16 test が filter host のみなのは AC15 の `rg -l` oracle で足りると判断 / mockup-f 差分 8 点は closeout の Backlog へ
- state-only（本 commit）: `implementing->local-verified->independent-review->human-confirm`、Reviewed Content HEAD `be71a857`。local-verified = Codex 44 の L1 full、independent-review = round 3 closure。次 = owner L3 run 2（AC-L3-1 はみ出し / AC-L3-6 icon・帯 + 観察 = 既定状態の帯と先頭行、chevron の濃さ、badge 撤去で現在の選択が分かりにくくないか）

### Gated Amendment 5（2026-09-11、owner L3 run 2 起源）

- owner L3 run 2（content `be71a857`）: AC-L3-1 / AC-L3-6 PASS「良くなったね」。観察 (1) 帯と先頭行のクリーム色が同じ「その質問か。変えるなら色かな。どう調べれば解決法が出るかも分からん」→ Coordinator: クリーム + 左バーは DSR-22 の現在行の語彙で帯が借りると 2 段に見える、帯は白地の箱 + 名前 `font-medium` に → owner 画像付きで承認 / (2)「ちょっと濃くは見える、揃えられるなら揃えてもいい」→ `text-muted-foreground` / (3)「大丈夫に見える、現在の選択と書いてあってリストにチェックと badge まである」→ no-action
- 是正（`083c61b6` + 本 commit）: D7 に白地の箱 / D5 に chevron 色 / Human Gate 欄に run 2 記録 / AC15 第 3 oracle 反転 + **AC16**（oracle 12 本）/ Matrix C17 + mutant 17（本 commit）。Final Review round 3 Opus #17（catalog 帯の句の矛盾）/ #18（`prettier-ignore` 撤去）/ #19 / #20（帯の未解決時 1 句）/ #21 を同乗
- 再発注: 発注書 45（起点 = 本 commit 後の tip）→ closure round 4（Sonnet + Opus、AC16 + mutant 17 + 回帰に限定）→ state-only → owner L3 run 3（AC-L3-6 帯と ▼ のみ）→ Ready

### closeout 圧縮記録（2026-09-11、STATECAP 上限のため branch で記録できなかった遷移）

- branch の forward state-only は `0b8377ec`（plan-draft→implementing）/ `01306c0b` / `86e94596`（各 →human-confirm、backtrack `63ebd94d` / `0fecd48d` を挟む）で上限（全体 3 / post-implementation 2）に達し、round 4 後の `implementing->local-verified->independent-review->human-confirm` を記録した `16354ef3` は STATECAP 超過で branch から下ろした（tip を `5db76334` に戻す、他 SHA 不変）。先例 PR #40 / #41 と同じく、以降の遷移を本 closeout commit で圧縮記録する
- **`implementing->local-verified->independent-review->human-confirm->ready-hosted-final->merge->archive`**（Reviewed Content HEAD `5db76334`）。中間遷移の証拠: local-verified = L1 full RESULT=PASS、END_HEAD_SHA=`5db76334`、CLEAN、MERGE_EVIDENCE_VALID=true（Coordinator、`.local/codex-orders/reports/l1-full-pr50-5db76334.log`。Writer Codex 45 の L1 は `83fa4a23` で PASS）/ independent-review = Final Review round 4 closure Sonnet approve + Opus approve（P2 1 = 帯の枠と背景が Input と同 token → owner L3 run 3 で判定、P3 2 no-action）/ human-confirm = owner L3 run 3 AC-L3-6 PASS（2026-09-11）+ Ready 承認「Ready していいよ」/ ready-hosted-final = hosted final `pull_request` run `34524148605` success（`5db76334`）/ merge = PR #50 squash `2a36b086`（2026-09-11、subject は Coordinator が日本語で付け直し）/ archive = 本 commit（packet / Matrix を `docs/archive/plans/` へ）
- 申し送り: mockup-f を運用後の確定形（▼ trigger / 白地の帯 / badge なし）に同期する件は owner 判断で Backlog に載せない（必要になった lane で拾う）。一括価格改定 toolbar の Checkbox 縦中央は「フィルタ Label 上置き + 見出し 2 段の runtime lane」の申し送りへ
