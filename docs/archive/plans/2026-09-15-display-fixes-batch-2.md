# Plan Packet: ㉒ 表示小修正 batch 2（在庫少の基準 見出し / Alert title 太字 / 在庫照会 副題 + 件数 / Badge「在庫あり」/ 基準の error 文言 / 明細数列 撤去）

2026-09-15 起票。wave 10 の lane 2（stacked train、D-074）。base は ㉑ の plan-first commit `a2e01334afed8535239644515a845359529a803e`（`agent/filter-label-top-runtime`）で、実装は ㉑ の実装 HEAD の上に stack し、㉑ merge 後に `origin/main` 単段 merge で base を付け替える。出典は Backlog「表示小修正 batch 2 候補」（owner 2026-09-11 所感 + L8-4 owner 決定 2026-09-15）。file:line は現行 main `04143923` で再実測（Sonnet Explore 2026-09-15、Coordinator が主要 site を直接確認）。実装は別 run とし、独立 Plan Review 通過後に発注する。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: 68fa591531e2249e13578cdff70ae3785b66757a
- Amendments: c35992b0f169c097319e9e4a75c41679bc5d3b0f, 479289cce653dcb4394e8e2756309fc61001c2db, fde49a358de2a36645de0f302cd30a050b5005d2, e1296981cab7cf45b215ba187a4df36dbdcce098
- Coordinator: Fable 5.1
- Writer: Codex
- Plan Reviewer: Sonnet（独立 fresh context）
- Final Reviewer: Sonnet + Opus（独立 fresh context。closure は Opus を含める）
- Final Review Minimum: 1
- Human Gate: ready,merge,manual

manual = owner Windows native L3 の抜き取り 3 画面（AC-L3-1〜3、10 分以内）。文言・weight の変更は rg oracle で閉じるが、Alert title の太さと「全 N 件」の見え方は owner の目が oracle（視覚系 UI change の運用教訓（`.claude/rules/review-workflow.md`） ではなく owner 方針 2026-09-05）。Fable 指揮の分業 lane で D-087 の一貫担当例外は適用しない。

遷移記録（append-only）:
- kickoff → spec-check → plan-draft → plan-gate（`b21703ad`）: Risk R2、Design Readiness が既存 function-design 58 / 65 / 69 + catalog ⑥ / ⑬ + mockup-d を十分と引用（文言・token の同期のみ）。Test Matrix は R2 任意だが test 更新が 5 file に及ぶため付ける。
- plan-gate → plan-approved（本 commit、state-only）: Plan Review round 1（Sonnet、P2 1）→ 是正 `3175c56f` + `0fad049e` → round 2 closure（新規 P2 1）→ 是正 `68fa5915` → round 3 closure = Findings Freeze 可（P1/P2 = 0、round 天井 3 で終了）。Plan Commit = `68fa5915`（plan-first `b21703ad` を含む是正済みの確定版）。implementing への遷移は ㉑ の実装 HEAD へ stack した時点で Coordinator が別 commit で記録する。
- plan-approved → implementing（本 commit、state-only）: ㉑ の実装 HEAD `2e6e1d7c`（Final Review round 1 是正後、closure 進行中）を単段 merge `c8087a1e` で取り込み stack した（衝突は `docs/Plans.md` のみ、両 lane の行を保持して解消）。stack base = `2e6e1d7c`。実装は Codex 発注書 51 で本 commit を HEAD_SHA として開始する。㉑ が closure で更に変わった場合は同じ単段 merge で追随する。

## Owner Effort Budget

- 介入回数上限: 3（L3 抜き取り 1 + Ready + merge）
- 実働時間上限: 20分
- relay 往復上限: 4（GA3 で 3 → 4。理由: Final Review round 1 の是正 1 往復。owner 承認 = 発注書 56 の起動〈55 は ㉑ GA4 の是正〉。GA2 時点の改訂理由: 実装 run 2 回が Coordinator の packet 誤り〈65 `:98` の節取り違え / 58 `:114` の括弧なし「通常」見落とし〉で fail-closed 停止し、Writer の実装 7 commit は保持されているが push 前。3 往復目は是正 1 箇所 + push + Draft PR のみ。owner 承認 = 発注書 53 の起動）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
利用者可視の文言・weight・列の変更で保守性と見え方に影響するが、runtime 契約（DTO / route / search state / DB / command）は変えない。`AlertTitle` は共通 component で 81 箇所に効くが class 1 語の変更で構造不変。「全 N 件」は既存 `items.length` の表示で backend 非接触（`list_low_stock` は Vec 全件、client filter 経路、58 §58.10 UI-06a-D1 の Rejected〈`list_low_stock` 経路への pagination 拡張、`:596`〉を維持）。`item_count` は DTO に残し列だけ外す。R3 に上げる契約変更はない。

## Goal

Goal Invariant:

### 最小完了条件

- 在庫少の基準: 区画見出しをなくし説明文のみにして h1「在庫少の基準」と二重に出ない（GA3、owner 判断 (c)）。入力 error は「1〜99999の整数を入力してください」の 1 文に揃う。
- Alert: icon の真横の title（`AlertTitle`）が app 全体で太字（600）になる。
- 在庫照会: 副題 1 行が付き、在庫状態 Badge が「在庫あり」になり（GA3、owner 判断 (b)）、在庫切れ / 在庫少の絞り込みで「全 N 件」が出る。
- 入出庫履歴: 一覧から「明細数」列が消え、他の列と詳細導線は不変。

### 失敗定義

- `item_count` の DTO / backend、`ManualSalePage` の「明細数」、`PaginationSummary` / `Pagination` の `status === "all"` 契約、`THRESHOLD_ERROR_MESSAGES` の判定分岐（空 / 非整数 / 1 未満 / 99999 超で保存拒否）に差が出る。
- 「通常」が docs / test に残る、または「正常」以外の別語になる。
- ㉑ の file 行（`StockInquiryPage.tsx` 表示件数 block、catalog ① / ⑤ / ⑨）を書き換える。

### 非目的

- 在庫少 / 在庫切れ絞り込みへの pagination（58 §58.10 UI-06a-D1 Rejected 維持）。「全 N 件のうち a〜b 件」の範囲表現は成立しない（client filter は slice しない）ため採らない。
- 記録状態 Badge の tone / 中立 Badge の見栄え（Backlog 保留）。
- `ManualSalePage.tsx:360,743` の「明細数」（別画面、65 の対象外）。
- ホーム mockup-c 採用（後続 lane）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## 起票時実測（2026-09-15、origin/main `04143923`）

| # | 項目 | runtime | docs 同期 | test |
|---|---|---|---|---|
| B1 | 在庫少の基準 見出し重複 | `src/features/threshold-settings/ThresholdSettingsPage.tsx:186-189` PageHeader `title="在庫少の基準"` + `:227-230` FormSection `title="在庫少の基準"` | `docs/function-design/69-ui-threshold-settings.md:152`（§69.9「FormSection 見出し」）。`:55` UI-11a-D6 は h1 / ナビの決定で不変 | `ThresholdSettingsPage.test.tsx:304` は h1 のみ |
| B2 | AlertTitle weight | `src/components/ui/alert.tsx:45` `"col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight"`。`<AlertTitle` は 35 file 81 箇所 | catalog ⑥「Alert warning variant」節 `:398-412` に weight の記述なし（新規 1 文） | `src/components/ui/alert.test.tsx`（title 描画のみ） |
| B3 | 在庫照会 副題 | `src/features/stock-inquiry/StockInquiryPage.tsx:100` `<PageHeader title="在庫照会" />` | `docs/design-system/reference/mockup-d-history.html:172` の文「商品ごとの在庫数と状態を確認し、その場で入出庫へ進みます」。`docs/function-design/58-ui-stock-inquiry.md` §58.7 に PageHeader の記述なし（新規 1 行） | なし |
| B4 | Badge「通常」→「正常」 | `src/features/stock-inquiry/components/StockStatusBadge.tsx:36`（唯一の描画元） | catalog ⑬ `:868`（code）/ `:885`（tone family 表、anchor `StockStatusBadge.tsx:42` は stale）/ `:905` / `:911`。58 `:114`（§58.2 file 構成表「在庫切れ / 在庫少 / 通常」、括弧なし。**Gated Amendment 2**、Codex fail-closed 停止 2）/ `:516` / `:565` / `:581` / `:585` / `:646`。74 は別文脈で対象外 | `ProductListTable.test.tsx:85,87` |
| B5 | 絞り込み時の件数 | `StockInquiryPage.tsx:225-231` PaginationSummary / `:249-257` Pagination とも `statusValue === "all"` gate。`useStockInquiry.ts:76-79` は `list_low_stock` 全件を `filterAndSortLowStockList` に通し `totalCount: null, source: "low_stock"`、slice なし | mockup-d-history `:175`「在庫少 12 件・在庫切れ 3 件」（`.cnt-plain` 16px / 600 / tabular-nums）+ `:177`「件数のみ上部に太字表示」。58 §58.7 / §58.10 に件数行の記述なし（新規） | `StockInquiryPage.test.tsx` に low_stock 表示の既存 case あり（件数 assert なし） |
| B6 | 基準の error 文言 | `src/features/threshold-settings/lib/extract-thresholds.ts:9-13` `THRESHOLD_ERROR_MESSAGES`（required / integer / max）、判定 `:18,22,27,31`。`threshold-form-schema.ts:9,14` re-export | 69 §69.7 `:131-134` 表 4 行 | `ThresholdSettingsPage.test.tsx:96,109,122,135` |
| B7 | 明細数列（L8-4） | `src/features/inventory-records/InventoryRecordsPage.tsx:354` TableHead / `:377` TableCell（`isInProgressStocktake ? "-" : record.item_count`）。`:366` の代表商品分岐は `item_count === 0` を使う（残す） | 65 `:212`（§65.8.1 列構成文）/ `:213`（算出仕様）/ `:271`（母集団注記、DTO 仕様として残す）。**`:98` は §65.5 詳細表示の表（記録詳細 page の項目、別契約）で対象外**（Gated Amendment 1、Codex fail-closed 停止 2026-09-15） | `InventoryRecordsPage.test.tsx:164`（進行中棚卸し「-」）/ `:922`（columnheader 7 列）/ `:927` 付近 `toHaveLength(7)` |

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **D-B1 区画見出しはなし、説明文のみ**（**GA3 是正**、owner 判断 (c)。理解しやすさのため見出しを外し、`FormSection` の DOM から h2 を除いた形〈説明 `<p>` + `Separator` + 入力〉を画面内に持つ。`FormSection` component と catalog ④ 本体は変更しない）。
- **D-B2 `AlertTitle` は `font-medium` → `font-semibold`**（owner 2026-09-11 原文）。catalog ⑥ の Alert warning variant 節に「`AlertTitle` は `font-semibold`（600）で icon の真横の 1 行を本文（`AlertDescription`）より強く出す」を 1 文追記。variant を問わず共通。
- **D-B3 副題は `subtitle` prop で mockup-d-history `:172` の文をそのまま**（`description` は複数文の操作説明用、catalog ①）。58 §58.7 に「PageHeader: title「在庫照会」+ subtitle『商品ごとの在庫数と状態を確認し、その場で入出庫へ進みます』」を 1 行。
- **D-B4 「在庫あり」**（**GA3 是正**、owner 判断 (b)。「正常」は `status === "all"` で閾値を見ずに `stock_quantity > 0` を ok とし、閾値割れ商品でも「正常」と断言してしまうため不採用、「在庫ありは基準以上を保証しない」と明記して採用）。catalog ⑬ の 4 箇所と 58 の 5 箇所、test 2 箇所を同 commit で更新。catalog `:885` の stale anchor `StockStatusBadge.tsx:42` は現行行（`:35-37`）へ直す（file:line を書かない規範は ⑨ D3 の ⑨ 限定で、⑬ は既に anchor を持つため最小差分で更新）。
- **D-B5 件数は「全 N 件」1 本（Backlog 原文の「全 N 件のうち a〜b 件を表示」は client filter が slice しないため範囲が成立せず、mockup-d `:177`「件数のみ太字表示」の形に落とす）、`status !== "all"` かつ `items.length > 0` のときだけ**、`PaginationSummary` の上の位置（`:225` の直前）に `<p className="text-base font-semibold tabular-nums">全 {data.items.length} 件</p>`（mockup `.cnt-plain` の 16px / 600 / tabular-nums を token で写す。`small` の注記「ページ送りはこの絞り込みでは行いません」は付けない = 情報より説明が増える）。`PaginationSummary` / `Pagination` の gate は不変。`items.length === 0` は EmptyState が出るため件数を出さない。N は `filterAndSortLowStockList` 後の件数（検索 / 部門で絞った後）で、mockup の「在庫少 12 件・在庫切れ 3 件」の 2 値表示は status が片方ずつしか選べないため 1 値。
- **D-B6 error 文言は 1 本「1〜99999の整数を入力してください」**（全角「〜」、半角数字。owner「範囲が一目で分かる 1 本」）。`THRESHOLD_ERROR_MESSAGES` の 3 key は残してよい（値を同一文にする）。判定分岐（空 / 非整数 / 1 未満 / 99999 超 → 保存拒否）は不変で、文言だけ揃える。上部「保存できませんでした」Alert は backend 失敗用で対象外（owner 2026-09-11 合意）。69 §69.7 表の文言列 4 行を同一文へ。
- **D-B7 明細数列は撤去、`item_count` は DTO に残す**（owner 決定 2026-09-15 (a)。backend 非接触、代表商品分岐 `:366` が `item_count` を使う）。`:212` の列構成から「明細数」を外す（**`:98` の §65.5 詳細表示表は触らない**、Gated Amendment 1）、`:213` は「`item_count` は一覧に表示しない（L8-4、owner 2026-09-15）。算出仕様は DTO として維持」へ改める、`:271` は DTO 注記として残す。手動販売出庫で困れば『代表商品 ほか N 件』型で戻す（Backlog 記録目的）。

## Scope

- **S1 `ThresholdSettingsPage.tsx:228`**: `FormSection` を h2 なしの `<section>`（説明 `<p>` + `Separator` + 入力）へ置換（GA3、owner 判断 (c)）。69 `:152` を「（なし、説明文のみ）」に。test を「h2 なし + 説明文あり」の assert へ。
- **S2 `alert.tsx:45`**: `font-medium` → `font-semibold`。catalog ⑥ に D-B2 の 1 文。`alert.test.tsx` に `AlertTitle` の class assert（`font-semibold` を持つ）。
- **S3 `StockInquiryPage.tsx:100`**: `subtitle` 追加。58 §58.7 に 1 行。`StockInquiryPage.test.tsx` に `getByText` 1 本。
- **S4 `StockStatusBadge.tsx:36`**: 「在庫あり」（GA3、owner 判断 (b)）。catalog ⑬ 4 箇所 + anchor、58 6 箇所（`:114` の「在庫切れ / 在庫少 / 通常」→「在庫切れ / 在庫少 / 在庫あり」を含む。**Gated Amendment 2**）、`ProductListTable.test.tsx:85,87`。
- **S5 `StockInquiryPage.tsx`**: D-B5 の件数行。58 §58.7（表示）と §58.10（業務ルール: 絞り込み時は件数のみ、pagination なし）に各 1 行。`StockInquiryPage.test.tsx` に「low_stock 3 件 → 『全 3 件』」「all → 件数行なし」「low_stock 0 件 → EmptyState のみ」の 3 本。
- **S6 `extract-thresholds.ts:9-13`**: 3 値を同一文へ。69 `:131-134`。`ThresholdSettingsPage.test.tsx:96,109,122,135` の期待文言。
- **S7 `InventoryRecordsPage.tsx:354,377`**: 列削除。65 `:212,213`（**Gated Amendment 1: `:98` は §65.5 詳細表示の表で対象外、削除しない**）。`InventoryRecordsPage.test.tsx:164`（明細数の assert を外し test 名を「代表商品を-で表示」へ）/ `:922`（配列から「明細数」除去）/ `:927` 付近（`toHaveLength(6)`）。
- **S8 docs**: catalog 更新履歴 1 行（⑥ / ⑬）。58 / 65 / 69 の更新履歴表に各 1 行。
- **S9（Coordinator、plan-first commit）**: Plans.md Wave Registry lane 2 に packet link / Phase。Writer は触らない。

## Non-scope

- `src-tauri/**`（`item_count` / `list_low_stock` 不変）
- `src/features/manual-sale/**`、`src/features/home/**`
- ㉑ の file 行（`StockInquiryPage.tsx:132-135`、catalog ① / ⑤ / ⑨、`PageHeader.tsx`）
- `PaginationSummary` / `Pagination` component、58 UI-06a-D1 の Rejected 判断
- 記録状態 Badge の tone、中立 Badge の見栄え統一（Backlog 保留）
- 一覧列以外の「明細数」表示（`rg -n '明細数' src --glob '!*.test.tsx'` から `InventoryRecordsPage` / `ManualSalePage` / `daily-sales`〈別語「売上明細数」〉を除いた 8 hit = 記録詳細 `*RecordDetailPage.tsx` 5 file〈CsvImport / ManualSale / Receiving / Disposal / Return。`StocktakeRecordDetailPage` には無く 65 §65.5〈6 種別 yes〉との既存差異、本 lane 対象外〉+ 入庫 / 廃棄 / 返品交換 の保存結果パネル 3 file〈`ReceivingPage.tsx:331` / `DisposalPage.tsx:316` / `ReturnExchangePage.tsx:451`〉。いずれも 65 §65.5 / 各画面 doc の別契約で、L8-4 は一覧列のみ）
- mockup-d-history の変更（`:177` の「件数のみ太字表示」と一致するため不要）

## Acceptance Criteria

rg oracle は出力空 = 0 件。baseline は起票時実測（origin/main `04143923`）。diff の base は stack 元（㉑ branch の実装 HEAD、実装発注書に SHA を明記）。

- **AC1**（**GA3 補正**: owner 判断 (c) 区画見出しなし）`rg -c 'title="基準値"' src/features/threshold-settings/ThresholdSettingsPage.tsx` = 0（`0e7783f5` 時点 1）/ `rg -c '<FormSection' src/features/threshold-settings/ThresholdSettingsPage.tsx` = 0（時点 1。import も外す）/ `rg -c 'title="在庫少の基準"' src/features/threshold-settings/ThresholdSettingsPage.tsx` = 1（baseline 2、h1 のみ）/ `rg -c '\| FormSection 見出し \| （なし' docs/function-design/69-ui-threshold-settings.md` = 1（時点 0）+ `rg -c '\| FormSection 見出し \| 基準値 \|' docs/function-design/69-ui-threshold-settings.md` = 0（時点 1）
- **AC2** `rg -c 'font-semibold' src/components/ui/alert.tsx` = 1（baseline 0）/ `rg -c 'font-medium' src/components/ui/alert.tsx` = 0（baseline 1）/ `awk '/^## ⑥/,/^## ⑦/' docs/design-system/02-component-catalog.md | rg -c 'font-semibold'` ≥ 1（baseline 0。**GA3 補正**: 文は `## ⑥` 直下〈使いどころ段落の前〉へ）+ `awk '/^## ⑥/,/使いどころ/' docs/design-system/02-component-catalog.md | rg -c 'variant を問わず'` = 1（`0e7783f5` 時点 0）
- **AC3** `rg -c 'subtitle="商品ごとの在庫数と状態を確認し、その場で入出庫へ進みます"' src/features/stock-inquiry/StockInquiryPage.tsx` = 1（baseline 0）/ `rg -c '商品ごとの在庫数と状態を確認し' docs/function-design/58-ui-stock-inquiry.md` ≥ 1（baseline 0）
- **AC4** `rg -c '通常' src/features/stock-inquiry/components/StockStatusBadge.tsx` = 0（baseline 1）/ `rg -c '在庫あり' src/features/stock-inquiry/components/StockStatusBadge.tsx` = 1（baseline 0。**GA3 補正**: owner 判断 (b)「正常」→「在庫あり」）+ `rg -c '正常' src/features/stock-inquiry/components/StockStatusBadge.tsx` = 0（`0e7783f5` 時点 1）+ `awk '/^## ⑬/,/^## ⑭/' docs/design-system/02-component-catalog.md | rg -c '正常'` = 0（時点 4）+ `rg -c '「正常」' docs/function-design/58-ui-stock-inquiry.md` = 0（時点 6、更新履歴行を含む）+ `rg -c '基準以上を保証しない' docs/function-design/58-ui-stock-inquiry.md` ≥ 1（時点 0）+ `rg -c '"正常"' src/features/stock-inquiry/components/ProductListTable.test.tsx` = 0（時点 2）/ `awk '/^## ⑬/,/^## ⑭/' docs/design-system/02-component-catalog.md | rg -c '通常'` = 0（baseline 4）/ `rg -c '「通常」' docs/function-design/58-ui-stock-inquiry.md` = 0（baseline 5）/ **GA2** `rg -c '在庫少 / 通常' docs/function-design/58-ui-stock-inquiry.md` = 0（baseline 1、`:114`）+ `rg -c '在庫少 / 正常' docs/function-design/58-ui-stock-inquiry.md` = 0（**GA3 補正**、`0e7783f5` 時点 1）+ `rg -c '在庫少 / 在庫あり' docs/function-design/58-ui-stock-inquiry.md` = 1（時点 0）/ `rg -c '"通常"' src/features/stock-inquiry/components/ProductListTable.test.tsx` = 0（baseline 2）/ `rg -c 'StockStatusBadge.tsx:42' docs/design-system/02-component-catalog.md` = 0（baseline 1）
- **AC5** `rg -c 'statusValue !== "all"' src/features/stock-inquiry/StockInquiryPage.tsx` = 1（baseline 0）/ `rg -c '全 \{data.items.length\} 件' src/features/stock-inquiry/StockInquiryPage.tsx` = 1（baseline 0）/ `rg -c '\{statusValue === "all" &&' src/features/stock-inquiry/StockInquiryPage.tsx` = 2（baseline 2 = `:229` / `:246` の gate。`:79` の `isFilterDefault` は pattern 外。Plan Review round 1 Sonnet P2: 旧 `rg -c 'statusValue === "all"'` は baseline 3 で「= 2」が必ず FAIL した）
- **AC6** `rg -c '1〜99999の整数を入力してください' src/features/threshold-settings/lib/extract-thresholds.ts` ≥ 1（baseline 0）/ `rg -c '1以上の整数を入力してください|99999以下で入力してください' src/features/threshold-settings docs/function-design/69-ui-threshold-settings.md` = 0（baseline 8）/ `rg -c '^  required:|^  integer:|^  max:' src/features/threshold-settings/lib/extract-thresholds.ts` = 3 または `THRESHOLD_ERROR_MESSAGE` 単一定数（判定分岐は test 4 本で不変を確認）
- **AC7** `rg -c '明細数' src/features/inventory-records/InventoryRecordsPage.tsx` = 0（baseline 1）/ `rg -c 'item_count' src/features/inventory-records/InventoryRecordsPage.tsx` ≥ 1（baseline 2、代表商品分岐が残る）/ `rg -c '明細数' src/features/manual-sale/ManualSalePage.tsx` = 2（不変）/ `rg -c '記録種別、業務日付、代表商品、明細数' docs/function-design/65-inventory-record-traceability.md` = 0（baseline 1）/ `rg -c '^\| 明細数 \|' docs/function-design/65-inventory-record-traceability.md` = 1（baseline 1、**不変**。§65.5 詳細表示の行、Gated Amendment 1 で「= 0」から反転）/ `rg -c '"明細数"' src/features/inventory-records/InventoryRecordsPage.test.tsx` = 0（baseline 1）
- **AC8**（負の oracle）`git diff --name-only <stack base>..HEAD -- src-tauri src/features/manual-sale src/features/home src/components/patterns/PageHeader.tsx src/components/ui/segmented-control.tsx | wc -l` = 0
- **AC9** 対象 test（`ThresholdSettingsPage` / `alert` / `StockInquiryPage` / `ProductListTable` / `InventoryRecordsPage`）PASS、`npm run typecheck` / `npm run lint` / `npm run format:check` PASS、最終 `bash scripts/local-ci.sh full` PASS（fresh worktree は `npm run generate:routes` 先行）
- **AC10** `bash scripts/doc-consistency-check.sh --target plan` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS
- **AC-L3-1** 在庫少の基準（`ThresholdSettingsPage.tsx`）: h1「在庫少の基準」の下に区画見出しはなく、説明文「保存すると…」だけが入力欄の上にある（GA3、owner 判断 (c)）。空欄で保存 → 欄の下に「1〜99999の整数を入力してください」1 文
- **AC-L3-2** 在庫照会（`StockInquiryPage.tsx`）: 副題 1 行、Badge「在庫あり」（GA3、owner 判断 (b)）、サイドバー「在庫少一覧」から開いて上部に「全 N 件」（太字）。チップ「在庫少」⇄「すべて」の切替で件数行（太字 16px）と PaginationSummary（14px muted）が入れ替わる見え方が気にならないか（所感、GA3）
- **AC-L3-3** 入出庫履歴（`InventoryRecordsPage.tsx`）: 列が 6 本（明細数なし）、詳細を見る は不変。一括価格改定の「ご注意」Alert の title（`AlertTitle`）が本文より太い。日報取込みで取込み済みの日報を選んだときの Alert title「この日報は取込み済みです。二重取込みはできません。」が 1 行に収まるか（切れていれば Backlog、GA3）

## Design Sources

- Requirements / spec: owner 2026-09-11 所感（Backlog「表示小修正 batch 2 候補」）、L8-4 owner 決定 2026-09-15 (a)
- Architecture: 該当なし
- Function / command / DTO: `docs/function-design/58-ui-stock-inquiry.md` §58.7 / §58.10 / §58.12、`65-inventory-record-traceability.md` §65.8.1、`69-ui-threshold-settings.md` §69.7 / §69.9
- DB: 該当なし
- Screen / UI: catalog ⑥（Alert warning variant）/ ⑬（ステータスバッジ）/ ①（subtitle の用途）、`reference/mockup-d-history.html:172-177`
- Decision log / ADR: 58 §58.10 UI-06a-D1 Rejected（`list_low_stock` 経路への pagination 拡張、維持）、D-047（在庫少一覧 = 在庫照会の絞り込み）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | なし（`item_count` / `list_low_stock` 不変） | not applicable |
| Command / DTO / generated binding / wire shape | なし | not applicable |
| DB / transaction / audit / rollback / migration | なし | not applicable |
| Screen / UI / route state / Japanese wording | 58 / 65 / 69 + catalog ⑥ / ⑬ | updated in this PR（文言・列・weight の同期） |
| CSV / TSV / report / import / export format | なし | not applicable |
| Durable decision / ADR | L8-4 は owner 決定済み（Plans.md）。65 に決定日を記す | updated in this PR |

## Registration / Generation Obligations

なし（command / route / doc 新設なし、REQ 追加なし、bindings 非接触）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-DISP-B2-1 | 69 §69.9 | D-B1 | 見出しなし、説明文のみ（GA3、owner 判断 (c)） | S1 | `ThresholdSettingsPage.test.tsx` |
| SPEC-DISP-B2-1 | catalog ⑥ | D-B2 | class 1 語、app 全体 | S2 | `alert.test.tsx` |
| SPEC-DISP-B2-1 | catalog ① subtitle / mockup-d `:172` | D-B3 | `subtitle`（1 行）、`description` は複数文用 | S3 | `StockInquiryPage.test.tsx` |
| SPEC-DISP-B2-1 | catalog ⑬ / 58 §58.10 | D-B4 | 「在庫あり」（GA3、owner 判断 (b)）、描画元 1 箇所 | S4 | `ProductListTable.test.tsx` |
| SPEC-DISP-B2-1 | mockup-d `:175-177` / 58 §58.10 UI-06a-D1 | D-B5 | 件数のみ、範囲・pagination なし | S5 | `StockInquiryPage.test.tsx` |
| SPEC-DISP-B2-1 | 69 §69.7 | D-B6 | 文言 1 本、判定分岐不変 | S6 | `ThresholdSettingsPage.test.tsx` |
| SPEC-DISP-B2-1 | 65 §65.8.1 | D-B7 | 列撤去、DTO 維持 | S7 | `InventoryRecordsPage.test.tsx` |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history: yes（各 doc の該当行を同 PR で更新、owner 決定は 65 / 69 の更新履歴に日付付きで残す）
- Plan-only durable decisions promoted: D-B7 の「戻す条件（代表商品 ほか N 件）」は Backlog 記録目的へ（Coordinator、closeout）
- Assumptions and constraints: `filterAndSortLowStockList` が slice しない（`useStockInquiry.ts:77` 実測）
- Deferred design gaps: 記録状態 Badge の tone（保留）
- Test Design Matrix can cite decision IDs: yes（D-B1〜7）
- Absolute guarantee / escape hatch self-check: `status === "all"` の gate 不変を AC5 で機械検査、`item_count` DTO 不変を AC7 / AC8 で検査

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable | — |
| Fact check / design decision split | 事実 = 実測表（81 箇所 / 描画元 1 / slice なし / 74 は別文脈）。判断 = D-B1〜7 | 本 packet |
| Lifecycle / retry | not applicable | — |
| Operator workflow | 見出し・文言・列が変わるが操作手順は不変。error 文言は 1 本になるため「なぜ拒否されたか」は範囲で読み取る | AC-L3-1 |
| Replacement path | not applicable | — |
| Data safety / evidence | not applicable | — |
| Reporting / accounting semantics | not applicable（`item_count` の算出は不変） | — |
| Manual verification | 3 画面の抜き取り | AC-L3-1〜3 |
| 環境・再現性 | なし | — |

## Design Readiness

- Existing design docs are sufficient because: 変更はすべて既存 doc の文言・列・weight の同期で、新しい振舞いは「全 N 件」のみ。その形は mockup-d-history `:175-177` と 58 §58.10 UI-06a-D1（`:596`） が既に定めている
- Source docs updated in this PR: 58 / 65 / 69 / catalog ⑥ ⑬
- Design gaps intentionally deferred: なし
- Durable decisions discovered: なし

Minimum design checks:

- Layer ownership: UI のみ
- Backend function design: 非接触
- Command / DTO / data contract: 非接触（`item_count` 残置）
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 区画見出しなし（説明文のみ）「在庫あり」「全 N 件」「1〜99999の整数を入力してください」の 4 文言を doc と同期
- Error, empty, retry, and recovery behavior: error 文言 1 本化のみ、判定不変
- Testability and traceability IDs: SPEC-DISP-B2-1、REQ 追加なし

## Contract Probe

N/A: 外部前提なし（library / OS 挙動に依存しない。`items.length` と class 変更のみ）。

## Contract Coverage Ledger

R2 のため任意だが、docs 同期の網羅性を独立 review で確認できるよう置く。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| 69 §69.9 FormSection 見出し | S1 | `ThresholdSettingsPage.test.tsx` h2 | AC-L3-1 |
| 69 §69.7 入力検証（4 rule → 保存拒否） | S6（文言のみ） | `ThresholdSettingsPage.test.tsx:96,109,122,135` | AC-L3-1 |
| catalog ⑥ Alert warning variant（構造不変、weight 追記） | S2 | `alert.test.tsx` | AC-L3-3 |
| catalog ① subtitle（1 行の短い副題） | S3 | `StockInquiryPage.test.tsx` | AC-L3-2 |
| catalog ⑬ 中立 Badge（stone 無彩色、文言） | S4 | `ProductListTable.test.tsx` | AC-L3-2 |
| 58 §58.10 status = all の「通常」表示契約 | S4（文言） | `ProductListTable.test.tsx` | — |
| 58 §58.7 / mockup-d `:177` 絞り込み時は件数のみ、pagination なし | S5 | `StockInquiryPage.test.tsx` 3 本 | AC-L3-2 |
| 58 §58.10 UI-06a-D1 Rejected（`list_low_stock` 経路への pagination 拡張、`:596`、Revisit = 100 件超の運用） | S5（非接触） | AC5（gate 不変） | — |
| 65 §65.8.1 一覧列構成 / `item_count` 算出 | S7 | `InventoryRecordsPage.test.tsx` | AC-L3-3 |
| 65 §65.5 詳細表示「明細数 yes × 6」（記録詳細 page の項目） | 非接触（GA1） | AC7 の `^\| 明細数 \|` = 1 不変 | — |
| 65 TRACE-D6 母集団注記（`:271`） | 非接触 | — | — |

## Test Plan

Test Design Matrix: [2026-09-15-display-fixes-batch-2](test-matrices/2026-09-15-display-fixes-batch-2.md)。

- targeted tests: S1〜S7 の test を実装と同 commit で更新
- negative tests: `status === "all"` で件数行が出ない / low_stock 0 件で件数行が出ない / 判定 4 rule が保存拒否を続ける
- compatibility checks: `item_count` を持つ既存 fixture が無変更で通る
- data safety checks: not applicable
- main wiring/integration checks: `AlertTitle` の class 変更が 1 箇所で 81 箇所に効く（`alert.test.tsx`）

## Boundary / Wire Contract

not applicable（wire 非接触）。

## Review Focus

- D-B5: 「全 N 件」の N が検索 / 部門で絞った後の件数であることが 58 の記述と test で明確か。`items.length === 0` の扱い。Backlog 原文「全 N 件のうち a〜b 件」から件数 1 本へ落とした根拠（client filter は slice しない、mockup-d `:177`）
- D-B6: 文言 1 本化で 69 §69.7 の 4 rule 表が「rule は 4 つ、文言は 1 つ」と読めるか
- D-B7: 65 `:213` の改稿で `item_count` の算出仕様（DTO）が消えないか
- ㉑ との境界: `StockInquiryPage.tsx` の表示件数 block（㉑）と `:100` / `:225` 付近（㉒）が別 hunk か
- 74-ui-operation-logs の「通常」を触らないこと（別文脈）

## Implementation Results

[PR #64](https://github.com/kosei-w90607/inventory-system-desktop/pull/64) で実装し squash merge 済み（`f2ef9e52`、2026-09-16）。Draft PR head の遷移: `8d0ae264`（Final Review round 1）→ GA3 是正 `1f5d6193`（発注書 56）→ Sonnet closure（Findings Freeze）→ owner L3 round 1 → base 同期 `beb95bd2`（㉑ squash 後の main 取込み）→ 独立監査 2 本（Sonnet / Opus）→ owner L3 round 2 → Ready → hosted CI pass → merge。owner 介入 2 / 予算 3（L3 ×2）。relay 4 / 上限 4（発注書 51 ×2 run / 53 / 56）。専用 record に review pass（broad: Sonnet `#issuecomment-5683251444` / Opus `#issuecomment-5683311573`）と manual pass（L3 round 1 + 2、`#issuecomment-5683220516`）を記録。

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none.

### Plan Review round 1（2026-09-15、plan-gate、Sonnet、裁定 Coordinator）

- P2-1（AC5 第 3 oracle の baseline が 3、`:79` `isFilterDefault` を含む。`rg -n 'statusValue === "all"' src/features/stock-inquiry/StockInquiryPage.tsx` = `:79` / `:229` / `:246`）= accept → gate 文脈 `\{statusValue === "all" &&` に限定（baseline 2）
- 残る不確実性「DSR `:596` の参照ズレ」= accept → 正しくは 58 §58.10 UI-06a-D1 Rejected（`:596`）。packet / Matrix の参照を訂正
- 残る不確実性「Adjacent Pattern Audit に記録詳細 7 page の明細数 field が無い」= accept → Non-scope と Matrix に 1 行（65 §65.5 の別契約）
- 残る不確実性「D-B5 が Backlog 原文の字面から離れる根拠が本文に無い」= accept → D-B5 と Review Focus に 1 句
- round 2 = closure（Sonnet、`3175c56f` + `0fad049e` の diff 限定）

### Plan Review round 2（closure、Sonnet）

- P2-1 / DSR 参照 / D-B5 = closed
- 「記録詳細 7 page」= **not closed、新規 P2**: 「7」は round 1 reviewer の文言を Coordinator が現物で数えずに転記した（実測は record-detail 5 + 保存結果パネル 3 = 8 hit）→ accept、Non-scope と Matrix の該当行を実測の内訳へ訂正（本 commit）
- round 3 = closure（Sonnet、本 commit の diff 限定。round 天井 3 の最終）

### Gated Amendment 1（2026-09-15、Codex 発注書 51 の fail-closed 停止）

- Writer が実装前に停止: D-B7 / S7 / AC7 が 65 `:98` の「明細数」行の削除を指定していたが、`:98` は §65.5 詳細表示の表（記録詳細 page の項目）で、Non-scope / Matrix が「別契約として維持」と書く行と矛盾していた。Coordinator の起票時実測が Explore 報告の「`:98`（対応マトリクス）」を §65.8.1 の一覧列と取り違えた（一覧列の記述は `:212` / `:213`）
- 是正: 実測表 B7 / D-B7 / S7 / AC7 / Ledger から `:98` の削除を外し、AC7 の oracle を「= 1 不変」へ反転。Scope の他項目・AC は不変
- 教訓: 起票時実測の doc 行番号は「その行が属する節」まで自分で読む（[[feedback-verify-own-corrective-claims]] と同型）

### Gated Amendment 2（2026-09-15、Codex 発注書 51 再実行の fail-closed 停止）

- Writer が 6 commit + lint 是正 1 commit を作った後、58 `:114`（§58.2 file 構成表）に括弧なしの在庫状態「通常」が残ることを検出して push 前に停止。packet の S4 と AC4 は `「通常」`（括弧付き）だけを数えており、Goal（「通常」が docs に残らない）と oracle が一致していなかった
- 是正: S4 / 実測表 B4 / AC4 に `:114` を追加（oracle は「在庫少 / 通常」→「在庫少 / 正常」）。`:192` `:447` `:607` の「通常の EmptyState」は一般語で対象外（`rg -n '通常' docs/function-design/58-ui-stock-inquiry.md` を Coordinator が実読）
- Owner Effort Budget: relay 往復上限 2 → 3（理由は同欄）。Writer の停止（発注書 51 の run 1 = 65 `:98`、run 2 = 58 `:114`。報告は `.local/codex-orders/last-51-display-fixes-batch-2-impl.md` と `report-51-…md`）はいずれも正しく、原因は Coordinator の起票品質
- 教訓: 文言置換の oracle は括弧付き literal だけでなく、その語が別の区切りで現れる形（` / 通常`、`通常）` 等）を rg で全数確認してから固定する

### Gated Amendment 3（2026-09-15、Final Review round 1 の裁定。owner 判断 2 件を含む）

Final Review round 1: pass A Sonnet = P1 0 / P2 0 / P3 2、pass B Opus = P1 0 / P2 1 / P3 10（mutant 9 本中 3 survive は挙動同値 or oracle 不在）。Findings Freeze 不可 → 本 GA3 で裁定し是正発注 56（GA3 補正で 55 から振り直し）→ closure（Sonnet）→ Freeze。

- **Opus P2-1（「正常」は `status === "all"` で閾値を見ずに `stock_quantity > 0` を ok にするため、閾値割れ商品でも「正常」と断言する）= owner 判断 (b)「在庫あり」**（2026-09-15「在庫ありは基準以上を保証しない」と明記）。D-B4 を「在庫あり」へ改訂。是正: `StockStatusBadge.tsx:36`「正常」→「在庫あり」、catalog ⑬ `:870` / `:887` / `:907` / `:913`、58 `:114` / `:519` / `:568` / `:584`（契約 H に「『在庫あり』は基準以上を保証しない〈すべて表示では閾値判定を行わない〉」を 1 文）/ `:588` / `:651`、`ProductListTable.test.tsx:85,87`。AC4 の「正常」を「在庫あり」に読み替え（`rg -c '正常' StockStatusBadge.tsx` = 0、`rg -c '在庫あり'` = 1、58 / catalog ⑬ / test の「正常」0、58 に「基準以上を保証しない」≥ 1）
- **Opus P3「基準値」= owner 判断 (c) 区画見出しを外して説明文だけ残す**（理解しやすさ）。D-B1 を改訂。是正: `ThresholdSettingsPage.tsx:227-230` の `FormSection` を `<section className="space-y-3"><p className="text-sm text-muted-foreground">保存すると…</p><Separator />…</section>` 相当（FormSection の DOM から h2 を除いた形。`FormSection` component と catalog ④ は触らない）に置換、69 §69.9 `:152` を「FormSection 見出し | （なし、説明文のみ）」へ、test の h2「基準値」assert を「h2 なし + 説明文あり」へ。AC1 を `rg -c 'title="基準値"'` = 0 / `rg -c '<FormSection' ThresholdSettingsPage.tsx` = 0 / 69 `\| FormSection 見出し \| （なし` = 1 に反転
- Opus P3（「全 N 件」の class に oracle なし、mutant survive）= accept → `StockInquiryPage.test.tsx` に `toHaveClass("text-base", "font-semibold", "tabular-nums")` 1 行
- Sonnet P3-1 / Opus P3（恒真 guard `data.items.length > 0`）= accept → guard 削除、`{statusValue !== "all" && (…)}`（AC5 の `statusValue !== "all"` = 1 不変）
- Opus P3（catalog `:884` `:886` の stale anchor `:34` / `:25`）= accept → `:29` / `:20`
- Opus P3（65 `:271` の「代表商品・明細数をともに『-』」）= accept → 「代表商品を『-』とする（明細数は一覧に表示しない、L8-4）」
- Opus P3（58 `:596` の新 bullet が UI-06a-D1 block 内で日付・ID なし）= accept → `#### UI-06a-D6: 絞り込み時の件数表示（2026-09-15、SPEC-DISP-B2-1 / D-B5）` として D1 block の後ろへ独立
- Opus P3（catalog ⑥ の AlertTitle 文が warning 節の内側）= accept → `## ⑥` 直下（`:336` の使いどころ段落の前）へ移動。AC2 第 3 oracle を `awk '/^## ⑥/,/^## ⑦/' … | rg -c 'font-semibold'` ≥ 1 に変更
- Sonnet P3-2 / Opus P3（`THRESHOLD_ERROR_MESSAGES` 3 key 同一文、空欄分岐が挙動同値）= accept → 定数に「3 key は 69 §69.7 の 4 rule 対応を保つ別名、値は同一（owner 2026-09-11）」の comment 1 行。分岐は残す
- Opus P3（Matrix Boundary Checks の `out_of_stock`）= Coordinator 修正（本 commit、正しくは `stockout`）
- Opus P3（PaginationSummary 14px muted と「全 N 件」16px 太字がチップ切替で跳ねる）= L3 観点へ（AC-L3-2 に「チップ切替時の件数行の見え方」）。揃えるなら別 lane（⑩ canonical 波及）
- Opus P3（`DailyReportImportPage.tsx:146` の 2 文 title が `line-clamp-1` で切れ得る）= L3 観点へ（AC-L3-3 に日報取込みの Alert）+ Backlog（title 1 文化）
- Owner Effort Budget: relay 3 → 4
- **GA3 補正（Coordinator、Amendments 追加）**: 上記裁定を AC1 / AC2 / AC4 / AC-L3-1〜3 の行に同期し、是正発注番号を 56 へ（55 は ㉑ GA4 の是正に先着）。裁定の内容は変えない

### Final Review（現在候補 beb95bd2 の完了判定、2026-09-16）

- Sonnet: P1 0 / P2 0 / P3 0（findings 0、pass 可）
- Opus: P1 0 / P2 1 / P3 8 → Coordinator 裁定で P2-1 を P3 へ格下げ（69 §69.4 の画面構成文が GA3 是正〈`FormSection` 撤去〉に追随せず「PageHeader + FormSection + 保存ボタン」のまま残る。挙動非影響の docs drift、runtime は既に h2 なしで実装済み）+ 新規 P3 3 件
- closeout（本 PR）で是正した項目:
  - P3-1（P2-1 格下げ分）: 69 `:75` / `:82` の画面構成文を GA3 是正後の実装（`<section>`、見出しなし）へ同期
  - P3-2: 69 §69.9 の行名「FormSection 見出し」「FormSection 説明」が `FormSection` 不使用後も残存 → 「区画 見出し」「区画 説明」へ
  - P3-3: catalog ⑩ 使用トークンに在庫照会の絞り込み時件数行（`text-base font-semibold`）が `PaginationSummary`（`text-sm muted`）と非対称である旨の記録がない → 例外 1 文を追記
  - P3-4: packet / Matrix に GA3 で撤回された「基準値」「正常」の literal が Goal / 設計判断 / Scope / Design Intent Trace / Minimum design checks / タイトル行 / Matrix C1・C5 に残存 → GA3 裁定（区画見出しなし / 「在庫あり」）へ書き換え
- Freeze 後の変更は本 closeout の docs 同期のみ、runtime / gate 契約は不変

## 後続

- 日報取込み「取込み済み」Alert の title を 1 文に（Final Review Opus P3、2 文が `line-clamp-1` で切れ得る）は [Backlog「保留」](../../backlog.md#見た目ux) を参照。
- destructive Alert の soft 塗り + 三角 icon（owner 所感 2026-09-15、L3 で確定。既登録）は [Backlog「やると決めたもの」](../2026-10-19-backlog-closed.md#やると決めたもの順番未定) を参照。
- 在庫照会の絞り込み時の件数行の token 非対称（`text-base font-semibold` vs `PaginationSummary` の `text-sm muted`）は catalog ⑩ 使用トークンの例外注記と [Backlog「記録目的」](../../backlog.md#記録目的受容済みリスクrevisit-条件付き) を参照。
