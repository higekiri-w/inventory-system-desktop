# 判断ルール集（DSR-01〜24）

> **親文書**: [README.md](README.md)
> **責務**: 実装時の迷いを一意に解消するルール集。「どちらを使うか」「いつ使うか」を DSR 番号で参照できる。

---

## 読み方

各 DSR は次の 4 部構成で書く。

- **ルール**: 守るべき内容を 1-2 文の断定形で示す
- **Why**: なぜそうするか。利用者前提（非 IT・高齢・赤黄色覚困難）と、[00-foundations.md](00-foundations.md) の色の役割表・強調の段階、[03-philosophy.md](03-philosophy.md) の出典に接地させる
- **判定フロー / 具体例**: 迷ったときに辿るフロー、または実装上の実例
- **関連**: 対応する [02-component-catalog.md](02-component-catalog.md) のパターン、および `docs/quality/review-checklist.md` カテゴリ 9 の対応項目

---

## 話題別の索引

DSR は番号を変えずに、下の話題の順に並べる。番号は追加された順で、話題とは関係しない。

- **配置と一覧**: DSR-04 ステータスバッジ: 状態列 vs セル内 badge / DSR-09 Form セクション分割の基準 / DSR-12 truncate と情報密度のバランス / DSR-13 表示スケールの設計方針 / DSR-16 同型情報のグループ化と囲みの階層 / DSR-22 一覧の器・現在行・UI 部品枠のコントラスト
- **入力**: DSR-02 Tabs vs SegmentedControl 判定フロー / DSR-05 read-only vs disabled の使い分け / DSR-06 必須表示の統一パターン / DSR-10 フィルタ候補のソース選定 / DSR-14 ファイル選択はネイティブダイアログ（path-based）を優先する / DSR-23 プルダウンは shadcn `Select` に統一する / DSR-24 追加導線を伴う master 参照は picker dialog に統一する
- **知らせ方**: DSR-03 Toast vs Alert 使い分け基準 / DSR-07 確認ダイアログを出す境界 / DSR-11 空状態・エラー文言・Tooltip の表示基準 / DSR-19 作成・保存成功の feedback 規約 / DSR-20 destructive 確認 dialog の配置・dismiss 規約
- **移動と戻り**: DSR-15 returnTo 等のリダイレクト系 param は検証してから使う / DSR-17 画面遷移と scroll の 3+1 分類 / DSR-18 詳細画面の戻り導線契約
- **色と状態**: DSR-01 主動線 CTA（1 画面 1 primary） / DSR-08 semantic 色のみで意味を伝えない / DSR-21 現在地と選択状態の色分離

---

## DSR-04 ステータスバッジ: 状態列 vs セル内 badge

**ルール**: 一覧で状態が主情報なら独立した「状態列」を置く。行の識別が主で状態が従なら、商品名セル内に `Badge` を置き、行全体を `text-muted-foreground` で減衰させる。

**Why**: IBM Carbon のデータテーブル設計と japanese-webdesign の「一度に把握したい」期待値の両立。状態列はスキャンしやすいが、列を増やすと商品名が読みにくくなる。商品名の可読性（非 IT 利用者の一次情報）を壊してまで状態列を足さない。

**判定フロー / 具体例**:

```
状態は一覧の主情報か？
├─ Yes（在庫照会の在庫状態等）  → 状態列を追加（StockStatusBadge）
└─ No（廃番フラグ等の従的情報） → 商品名セル内 Badge + 行 text-muted-foreground
```

実例: 在庫照会は「状態」列に `在庫切れ` / `在庫少` バッジ（`ProductListTable.tsx`）。商品一覧の廃番は商品名セル内の `廃番` Badge + 行減衰（`ProductTable.tsx`、PR #95 で状態列方針から変更）。

**関連**: パターン③テーブル / ⑬ステータスバッジ。review-checklist カテゴリ 9 対応（テーブル密度・幅・truncate が主要値の理解を壊していないか）。

---

## DSR-09 Form セクション分割の基準

**ルール**: フォームは意味境界でセクション分割し、各セクションに見出し + 1 行説明 + `Separator` を付ける。セクションが 4 つ以上になったら構成を再考する。

**Why**: IBM Carbon の Progressive disclosure と refactoring-ui のヒエラルキー。長いフォームを 1 枚で見せると、高齢の利用者がどこまで入力したか見失う。意味のまとまり（識別 / 分類 / 価格 / 在庫）で区切ると、入力の見当がつく。セクションが増えすぎる場合は、その画面に項目を詰めすぎていないか（GOV.UK「Do less」）を疑う。

**判定フロー / 具体例**: 商品フォームは「商品の識別 / 分類と取引先 / 価格 / 在庫」の 4 セクション。各セクションは `<h2>` 見出し + `text-muted-foreground` の 1 行説明 + `Separator`。5 セクション以上になりそうなら、項目の要否や画面分割を先に検討する。

**関連**: パターン④フォームセクション。review-checklist カテゴリ 9 対応（共通レイアウト・typography 継承）。

---

## DSR-12 truncate と情報密度のバランス

**ルール**: `min-w-0` + `truncate` は意図した列にのみ当てる。truncate した主要値には、全文を確認できる手段（折り返し・展開・別表示）を残す。

**Why**: japanese-webdesign の「情報密度 = 信頼」と inventory-operator-ui の「主要値を回復手段なしに隠さない」の両立。業務データは密に見せたいが、商品名や金額を黙って切り詰めると、利用者は欠けた情報に気づけない。truncate は意図的に、かつ回復手段とセットで使う。

**判定フロー / 具体例**: カードの金額は `min-w-0` + `truncate` でカードの溢れを防ぐ（`SummaryCardsBar.tsx`）。一方で商品名列は `min-w-[14rem]` + `whitespace-normal` で折り返す（`ProductTable.tsx`）。行インライン展開の詳細セルは `whitespace-normal` で切らずに全文を見せる（パターン⑫）。

**関連**: パターン③テーブル / ②サマリカード / ⑫行インライン展開。review-checklist カテゴリ 9 対応（テーブル・カード・チップの密度・幅・truncate が主要値の理解を壊していないか）。

---

## DSR-13 表示スケールの設計方針

**ルール**: 「文字が小さい」への対処として、個別の font-size 増を初手にしない。全体の見やすさは display-scale option（PR #77 導入済み、WebView zoom）で扱う。

**Why**: inventory-operator-ui のルール。1 箇所だけ文字を大きくすると、画面間で文字サイズがばらつき、Fluent 2「Coherent」が崩れる。視認性は機能要件であり、capability・永続化・L3 検証を伴う全画面横断の設計で解く。局所対応はその場しのぎになり、別画面で同じ不満が再発する。

**判定フロー / 具体例**:

```
「小さくて読みにくい」指摘が来た
├─ 全体的な読みにくさ → display-scale option（Sidebar footer の 3 段階 zoom: standard=1 / large=1.15 / extra_large=1.3）
└─ 特定セルの最小級 text-xs → 通常 table text へ上げる（局所だが「縮めすぎを戻す」方向のみ）
```

font-size を全体一律に底上げする再設計は本ルールの scope 外（将来検討、参照のみ）。

**関連**: パターン⑬ステータスバッジ末尾「小さい文字・密な表への対応」。review-checklist カテゴリ 9 対応（非 IT・高齢利用者が通常距離で主要テキストを読める設計か / 状態を変える control へ戻れるか）。

---

## DSR-16 同型情報のグループ化と囲みの階層

**ルール**: 同型レコード（同じ形の複数件）を並べて示すときは、以下の判断フローで表示形式を選ぶ。囲み（`border` / カード）は意味階層ごとに 1 つまでとし、内部の反復は余白・行区切り（`border-b`）で示す。薄い `border` を単独のグループ信号にしない。

```
複数件をどう見せるか？
├─ 比較が目的（同じ項目を横並びで見比べる） → 列を揃えた表 / structured list
│    （per-card の dl 反復を避け、共通の列見出しまたは揃った grid 行にする）
├─ レコードごとに固有の操作がある（行削除・個別編集ボタン等） → 一意見出し付きの summary card
│    （カード反復は許容するが、カード自体を唯一の識別信号にせず見出しで区別する）
└─ 単一レコードの確認（1 件だけを確認する） → key/value の definition list（原則適合）
```

**Why**: NN/g の *Common Region*（近接した要素を共有の境界で囲むと 1 グループとして知覚される強い原理）は、境界を多用すると clutter を生み、かえって比較を妨げる。GOV.UK Design System の *Summary list* パターンは、少量の関連する key-value を per-card の反復でなくカード 1 枚の中で列を揃えて示し、同型レコードの比較を可能にする。囲みを比較目的で反復すると、各レコードが独立した「箱」に見え、利用者は箱をまたいで値を目で追う負荷を負う。非 IT・高齢の operator ほどこの負荷の影響を受けやすい（2026-08-29 owner L3-lite 実測: D13 項目完全性は PASS でも「磨かれて見やすい」が FAIL になった直接原因）。

**判定フロー / 具体例**: 追加確認 dialog（`AdditionalImportConfirmDialog.tsx`）の既存分 3 件は「比較が目的」（同じ日の取込みを見比べて重複か判断する）に該当するため、per-card の `dl` 反復を廃し、列の揃った structured list へ再構成した（gated Amendment 3）。一方、rollback 確認 summary（`ResultStep.tsx` / `DailyReportResultStep`）は「単一レコードの確認」（今から取り消す 1 件だけを確認する）に該当し、既存の definition list のままで原則適合と判定した（囲み階層が 2 重以上でないかだけ点検する）。

**強調の段階との関係**: `00-foundations.md` の強調の段階の段 1（役割の薄い地）は線を引かずその役割の領域だけに使い、ただのまとまりは段 0 の stone で示す。段 2 の線は状態を知らせる行・badge・Alert と Home の入口 card に限る。これにより、薄い `border` を単独のグループ信号にしない本 DSR と両立する。

**関連**: パターン⑧Dialog/確認（比較用 variant）。review-checklist カテゴリ 9 対応（同型情報の表示形式が DSR-16 の判断フローに適合するか）。

---

## DSR-22 一覧の器・現在行・UI 部品枠のコントラスト

**ルール**: `ListShell`（一覧の器、`src/components/patterns/ListShell.tsx`、Lane 2 実装・商品一覧 pilot 採用）は pagination を持つ一覧画面全体に適用する。`ListShell` 採用画面では `topSummary` prop（既定 `false`、`src/components/patterns/ListShell.tsx` の `ListShell` の引数）で上部表示を明示的に opt-in し、非 `ListShell` 画面では対応する `PaginationSummary` を直接描画することで、pagination を持つ一覧画面すべてに適用する。opt-in 後の上部の件数・現在位置 text は totalCount > 0 のとき常に表示する（table header の sticky 化は引き続き `ListShell` の `stickyHeader` prop 採用画面のみ）。上部は `PaginationSummary` で範囲付き統一形「全 {n} 件のうち {from}〜{to} 件を表示（{p} / {t} ページ）」の text 表示のみを持ち、pager ボタンは置かない。下部は `totalPages > 1`、または `page > totalPages`（範囲外ページからの回復導線）のときに、件数 + pager フル装備の `Pagination`（`src/components/patterns/Pagination.tsx`）を描画する（Lane 2 で旧 `ProductPagination` から移設・実装済み、Lane 4 で `totalPages <= 1` gating と上部ロールアウトを実装済み、`page > totalPages` 回復例外は PR #40 Codex P1 是正で棚卸し一覧の呼び出し側 gate へ同期済み）。識別列固定は、横スクロール（overflow）が実際に生じるときにだけ opt-in で発動し、固定対象は画面分類ごとに次のとおり pin する（本 mapping が唯一の正本、⑯・04 原則 6・mockup では重複記載しない。Lane 4 Gated Amendment 1（2026-09-06、owner 案 X）で商品一覧のみ専用の scroll 箱（`data-list-scroll-container`）方式へ更新: 表自身が縦横 scroll 箱になり、main（縦、toolbar 分の page scroll）と箱（横に常時・箱の viewport 基準高さを内容が超えたときのみ縦）が並行して scroll する（Lane 4 Gated Amendment 4）。識別列の `sticky left-*` は箱を基準に機能する（`identityColumns` prop、S9 は不変）。他画面は同じ mapping・同じ機構のまま opt-in 未実施）:

| 分類 | 画面 | 固定列 |
|---|---|---|
| 商品系 | 商品一覧 / 在庫照会 / 一括価格改定 / 整合性チェック / 日次・月次 ranking | 商品コード + 商品名 |
| 棚卸し | 棚卸しカウント一覧 | 商品コード + 商品名 |
| 入出庫履歴 | 入出庫履歴（`src/features/inventory-records/InventoryRecordsPage.tsx` の表頭の〈記録日時〉/〈代表商品〉、複数商品の横断一覧。採用後の列順は表頭（`TableHeader`）= 種別 / 業務日付 / 代表商品 / 明細数 / 状態 / 記録日時 / 操作 で非隣接・非先頭のため、左固定にあたり 記録日時 → 代表商品 を先頭 2 列へ並べ替える（header 配列一致 test の更新を含む、Lane 3〜5）（記録ID列は本節の (b) 決定により一覧から除外。本 PR（⑮）で記録ID列の撤去を反映済み。記録IDフィルタは維持する）） | 記録日時 + 代表商品 |
| 在庫変動履歴 | 在庫変動履歴（`src/features/stock-movements/StockMovementsPage.tsx` の `PageHeader` と商品の見出し、単一商品ページ — 商品は見出しに表示され列は持たない、`src/features/stock-movements/components/MovementTable.tsx` の表頭の「日時」「種別」） | 日時 + 種別 |
| 操作ログ | 操作ログ（`src/features/operation-logs/OperationLogsPage.tsx` の表頭の「日時」「種別」） | 日時 + 種別 |
| 管理系 | 取引先管理（`src/features/suppliers/components/SupplierUsageTable.tsx` の表頭の「取引先名」、取引先コード列は存在しない） | 取引先名 |
| 管理系 | backup 一覧（`src/features/backup-restore/BackupRestorePage.tsx` の表頭の「日時」「ファイル名」） | 作成日時（`created_at`）+ ファイル名（`file_name`） |

履歴系（入出庫履歴 / 在庫変動履歴 / 操作ログ）の固定列は、旧分析 doc の実装未確認の「仮置き」判断（`docs/design-system/reference/2026-08-23-current-design-analysis.md` の 1-8「識別列の有無は画面依存」）を実コード確認で更新したものであり、Lane 2 の L3（PR #32 run 3「history は固定列を含む現実装を維持」、run 4〜7 で異議なし、2026-09-04）で実利用者確認済みとして**最終確定**した。

**記録IDの一覧表示可否（owner culling）**: 入出庫履歴一覧（`src/features/inventory-records/InventoryRecordsPage.tsx` の表頭）の記録ID列は種別ごとの連番で全体では一意でなく、記録詳細ページも記録IDの数値を本文表示しない（[UI 磨き batch 3 design](../archive/plans/2026-09-06-ui-polish-batch3-design.md) 起票時実測）。(a) 種別込み複合表示 / (b) 一覧の表示列から外す / (c) 現状維持 の3案を owner culling に諮った結果、**(b) 一覧の表示列から外す**で確定した（2026-09-06、原文回答は同 packet「Review Response」節）。フィルタ入力欄（`65-inventory-record-traceability.md` §65.4.1 の記録ID exact match）は一覧列と独立した UI 要素のため維持する。反映先: `docs/function-design/65-inventory-record-traceability.md` §65.8.1 結果列挙（本 PR で反映済み）。上記固定列 mapping 表（記録日時 + 代表商品）は本決定の影響を受けない。

現在行（入力中の行 / 開いている行 / 選択行）は `00-foundations.md` の色の役割表の進行中の役割に属する。選択行は操作対象として開いている 1 行に限り、checkbox / radio で複数選ぶ行は選択状態（DSR-21 の stone と部品の checked）で、現在行に含めない。現在行は左 4px のバー + 淡い背景 + badge/文言の 3 点で示す（色だけに頼らない、DSR-08）。詳細を開いた行（02 ⑫ の行インライン展開、操作ログの詳細の開閉）は開いている行に当たり、3 点目は直下に開いた詳細と「詳細を閉じる」の類の文言が担い、badge を足さない（00 の迷いやすい場面、D-094）。作業中の囲み（役割色の 2px 枠）は強調の段階 3 で、1 画面に 1 か所とする。バーと作業中の囲みの枠は進行中の枠（`--ongoing`）、地は進行中の地（`--ongoing-soft`）で引く（D-094）。バーと作業中の囲みの枠は、隣接背景に対し 3:1 以上の色で引く（操作枠と同じ要件）。UI 部品の枠は、interactive な操作枠（入力・outline ボタン・select・segmented・focus ring）を隣接背景（ページ背景）に対し 3:1 以上（WCAG 2.2 SC 1.4.11 / 2.4.13）にする — 対象は文字ラベルではなく塗り・枠自体が識別子になる component（focus ring 等）で、`secondary` button のような文字ラベルで識別される部品は塗り・枠を装飾として扱い、この 3:1 要件の対象に含めない。**badge（状態/分類/強調）は 3:1 の対象外**とする（非 interactive のため WCAG 1.4.11 の対象外）。代わりに次を要求する: 文字は WCAG 1.4.3 の通常テキスト基準 4.5:1 以上、**badge は枠線（tone 固有色または `--border`）を必ず持つ**（soft 背景単独・枠なしは不可）、**非中立の①状態 badge（warning/success/destructive tone）はさらに icon を必須にする**（中立 tone の①状態 badge は icon 任意、canonical `在庫あり`=`src/features/stock-inquiry/components/StockStatusBadge.tsx` の `StockStatusBadge` の中立の Badge が icon なしで準拠する先例。②分類・③強調も識別に必要な場合のみ icon 可、廃番等は枠線ルールで担保されるため icon 任意）。枠線の強度は 3:1 を要求しない（tone 固有色または `--border` でよい）（2026-09-05 owner 決定、v2 mockup 視認。旧文は Badge/outline chip も 3:1 対象としていたが、owner が「`--border-strong` の枠はくどい」と明示的に却下した）。構造線（行区切り・表枠・card 枠）は 3:1 の対象外だが `--border`（`#cdc8c4`、対 `--background` 1.59:1、Lane 2 で `#e7e5e4`〈≈1.20:1〉から濃化済み）を用いる。token は `--border`（構造線）/ `--border-strong`（操作枠、`#8a8480`、対 `--background` 3.53:1、`--input` が参照）/ `--control-surface`（操作面、`#fafaf9`、対 `--card` #f5f5f4 1.02:1、入力欄・Select、Gated Amendment 7 S46）の 3 種と、現在行のバーと地の `--ongoing` / `--ongoing-soft`（00 の進行中の家族）。globals.css に実装済み、値は [00-foundations.md](00-foundations.md) カラーパレット表を正本とし DS3 の突合対象に含める。

**Why**: 視野狭窄では横スクロール中の行追従と画面端の見落としが増えるため、識別列を固定し件数・現在位置を上でも示す（Q5 原則③④、`ui-design-rules-qa-v2.md` 6-9「近接の要因」「要素の見切れ」、原田秀司『UIデザインの教科書［新版］』翔泳社、2020）。ソート機能を持つ一覧は基準列を明示する（Q5 原則①、5-4）。左 rail への 3 大操作常設（Q5 原則②、5-4）は、本アプリでは sidebar（22 項目、`src/config/navigation.ts` の `status: "active"` 実測値、`src/components/layout/RootLayout.tsx` の `<aside>`）が既に左を占有しているため採らず、横 toolbar 2 段（04-backbone 原則 6）で代替する（Q5-② 適用外注記）。ページングは現在地・件数を保ち利用者に高い「コントロール感」を与えるため、探索・比較・編集が主目的の一覧では無限スクロールでなく本則とする（Q15、6-8）。上部 pager のボタンを任意にとどめるのは、同じ操作ボタンが上下 2 組あると選択肢が増え判断コストが上がるため、初心者ほど操作体系はシンプルな方がよいという原則（Q12 §1「初めてインターフェースを触るユーザー（初心者）にとっては、操作体系はシンプルなほうが使いやすい（インタラクションコストが少なくて済むため）」、同書）を安全側に適用したものである。高齢者・IT に不慣れな利用者への直接の配慮規定は同書に記載がなく（Q12 §2/§3）、operator profile の根拠は本アプリでは WCAG / Laws of UX 側から引く（Q12 caution）。現在行の 3 点表示と一覧の器の統一は、Common Region によるグルーピングと選択負荷の低減にも資する（Miller's Law / Law of Common Region、Jon Yablonski『UXデザインの法則』第 2 版、相島雅樹・磯谷拓也 訳、オライリー・ジャパン、2025-01。DSR-19/20/21 が同著を Why で引用した先例と同型の引用範囲に限定し、理論書の主張をそのまま規則として転記しない）。Badge/outline chip の枠線必須化は owner Human Gate 所感（2026-09-03、`mockup-d-lists.html` の廃番 Badge が黄み白の soft 背景で白地に埋もれ見づらい）に基づく — soft 背景単独は非テキスト部品の識別信号として機能しないため、境界（枠線、tone 固有色または `--border`）を必須にし、文字は WCAG 1.4.3 の 4.5:1 を別途維持する。2026-09-05 owner 決定により、UI 部品枠 3:1 の対象を interactive な操作枠に限定し、badge は対象外とした。WCAG 1.4.11 はテキストで識別される component の境界コントラストを要求しない（badge は文字ラベルで意味を識別する）。2026-09-03 の owner 所感（廃番 Badge が soft 背景のみ・icon なしで白地に埋もれ見づらい）が禁止したい組み合わせ（soft 背景単独・icon なし）は、本節の icon 必須・soft 背景単独不可のルールで数値要件なしに引き続き維持される。DSR-16「同型情報のグループ化と囲みの階層」とは主題が異なる — DSR-16 は複数レコードの表示形式（表 / summary card / definition list のどれを選ぶか）の選定、DSR-22 は選んだ表示形式の器そのもの（件数・sticky・識別列固定・枠のコントラスト）と現在行の指し方を扱う。DSR-21「現在地と選択状態の色分離」とも異なる — DSR-21 の「現在地」は主ナビゲーションの active link（「今どの画面にいるか」、画面単位）を指すのに対し、DSR-22 の「現在行」は一覧内で編集中/選択中の 1 レコード（「今どのレコードを扱っているか」、レコード単位）を指し、対象階層で区別する。

**判定フロー / 具体例**: 商品一覧・在庫照会・一括価格改定・棚卸し・整合性チェック等の pagination を持つ一覧は本則に従う（適用範囲・固定列 mapping は上記ルール表を参照）。perPage の既定値は画面ごとに変えてよいが、共有定数（`LIST_PER_PAGE_OPTIONS` 等）は 1 本のまま維持する（裁定案・owner 了承 2026-09-03: 棚卸しは未入力を潰し切る全走査が主動線のため既定 50、商品一覧は 1 件探索が主動線のため既定 100）。廃番 Badge の是正例: `secondary` pill のまま `--border` 枠線を追加し、既存の灰色系 secondary Badge も同根の可読性課題として Lane 3〜5 の sweep 対象に含める。

**低視力 L3**: 操作者画面の L3 checklist に (a) Windows forced-colors（ハイコントラスト）で状態・枠・focus が消えない (b) DPI 125% / 150% で崩れない（rem / em 基準、px 直書きを避ける）(c) 実利用者（緑内障等）の Windows native 1 セッションを代表画面で、を含める。行内の icon のみボタンは見た目 16px のまま当たり判定 24×24 以上（padding）にする（WCAG 2.2 2.5.8 target size）。一覧の器・現在行に限らず低視力を前提にした全画面の L3 判定に適用する。

**関連**: パターン③テーブル / ⑩ページネーション / ⑬ステータスバッジ / ⑯一覧の器（ListShell）。DSR-08「semantic 色のみで意味を伝えない」/ DSR-16「同型情報のグループ化と囲みの階層」（表示形式の選定は DSR-16、器は DSR-22）/ DSR-21「現在地と選択状態の色分離」（画面レベルの現在地は DSR-21、レコードレベルの現在行は DSR-22）。review-checklist カテゴリ 9 対応（一覧の器・現在行・UI 部品枠のコントラスト、Badge/outline chip の枠線必須化、低視力 L3 が DSR-22 に沿っているか）。

---

## DSR-02 Tabs vs SegmentedControl 判定フロー

**ルール**: 2 択の切替は SegmentedControl を使い、3 つ以上または内容が異質なら Tabs を使う。route 駆動の 2 択は SegmentedControl + `<Link>`、ローカルの view mode 切替は SegmentedControl + button group にする。

**Why**: 二択切替を画面ごとに別の見た目で組むと、Fluent 2「Coherent」（画面間で配置が一貫）が崩れ、利用者が同じ操作を別物と認識する。SegmentedControl は押しボタン状の濃い外枠を避けた中庸の active で統一し、視覚言語のばらつきを防ぐ。

**判定フロー / 具体例**:

```
切替対象は 2 択か？
├─ Yes → route 駆動（URL が変わる）か？
│        ├─ Yes（例: 日次/月次） → SegmentedControl + <Link>
│        └─ No（例: 商品別/部門別の view mode） → SegmentedControl + button group（aria-pressed）
└─ No（3 つ以上 or 内容が異質） → Tabs
```

実例: 売上データ取込み画面（`CsvImportPage.tsx`、PR #125）は「日報取込み / 商品別CSV取込み（Z004）」の 2 択だが、CMD（CMD-12 vs CMD-07）・保存先テーブル（`daily_report_imports` 系 vs `csv_imports` 系）・結果表示が全く異なるため「内容が異質」に該当し Tabs を採用した（UI-07-D9: 日報と Z004 を同一テーブル・同一結果として表示しない）。既定タブは業務上の主動線（`defaultValue="daily-report"`）にする。

**関連**: パターン⑤SegmentedControl。review-checklist カテゴリ 9 対応（共通レイアウト継承・active state が色以外でも判別可能か）。

---

## DSR-05 read-only vs disabled の使い分け

**ルール**: 値を見せて編集だけ不能にするなら `readOnly` + `bg-muted` を使う。操作そのものを不能化するなら `disabled` を使う。

**Why**: 両者は利用者への意味が違う。`readOnly` は「これは確定値で、見て確認するもの」、`disabled` は「いまは操作できない」。混同すると、利用者が「なぜ触れないのか」を誤解する。Fluent 2「Relevant」（予測通り動く）の観点で、見た目と挙動を一致させる。

**判定フロー / 具体例**:

```
値を見せたいか？
├─ Yes（確認用に表示、編集不可） → readOnly + bg-muted
└─ No（操作経路ごと閉じる）       → disabled
```

実例: 編集時の商品コード・JAN・現在庫は `readOnly` + `bg-muted`（値は見せる）。編集時の長さの単位の商品の数量単位 select は `disabled`（選択操作自体を止める。D-119）。

**関連**: パターン④フォームセクション。review-checklist カテゴリ 9 対応（主要テキスト・状態が読める設計か）。

---

## DSR-06 必須表示の統一パターン

**ルール**: 必須項目は色だけで示さず、ラベルに「（必須）」と明記する。

**Why**: WCAG 1.4.1（色のみで情報を伝えない）。赤色のアスタリスクだけで必須を示すと、赤を識別しにくい利用者には伝わらない。日本語テキストの「（必須）」なら誰でも読める。Polaris の語彙統一の方針で、全フォームで同じ表記に揃える。

**判定フロー / 具体例**: 必須フィールドのラベルは「商品名（必須）」「部門（必須）」「売価（必須）」のように書く。任意項目には何も付けない（または明示が要るなら「任意」）。アスタリスクや色の赤化は単独で必須シグナルにしない。

**関連**: パターン④フォームセクション。review-checklist カテゴリ 9 対応（業務ステータスが色だけで符号化されていないか）。

---

## DSR-10 フィルタ候補のソース選定

**ルール**: select / filter の候補が master 全件由来か filtered result 由来かを明示的に決める。filtered result から候補を作り、現在の選択値が候補から消える縮退は禁止する。

**Why**: inventory-operator-ui のルール。絞込み結果から候補を再生成すると、選択中の値だけが候補に残り、他の値へ直接切り替えられなくなる。利用者は「フィルタを外す」操作を知らないことが多く、行き止まりになる。master 全件を候補にすれば、いつでも他候補へ移れる。

**判定フロー / 具体例**:

```
filter 候補はどこから作るか？
├─ master 全件（推奨） → listDepartments の全件を SelectItem に展開
└─ filtered result      → 選択値が候補から消えないことを証明できなければ採用しない
```

実例: 部門フィルタは `listDepartments` の master 全件を候補にする（`DepartmentFilter.tsx`）。現在の検索結果に含まれる部門だけを候補にはしない。

**関連**: パターン⑨検索 + フィルタ。review-checklist カテゴリ 9 対応（Select / filter の候補を現在の filtered result から派生していないか）。

---

## DSR-14 ファイル選択はネイティブダイアログ（path-based）を優先する

**ルール**: ファイル選択は `@tauri-apps/plugin-dialog` の `open()` でパスを取得し、`@tauri-apps/plugin-fs` の `readFile()` で読み込む path-based 方式を優先する。HTML `<input type="file">` の新設は禁止する（既存画面も含めて全面禁止、[UI_TECH_STACK.md §6.5.4](../UI_TECH_STACK.md#654-ネイティブダイアログ) 参照）。かつて暫定例外だった plain input（Z004 取込み / 商品一括インポート / レシート画像）は D-054（順9 PR #26）の共通 FilePicker への移行で解消済み（2026-08-03 round 2 是正、plain `<input type="file">` は src 内 0 件）。

**Why**: WebView2 では HTML file input がネイティブダイアログ起動後に DOM 変化まで再描画されず白画面になるバグがある（JS 例外なし・console 無出力。PR #125 の Windows native L3 実機検証で検出、経緯は [UI_TECH_STACK.md §6.5.4](../UI_TECH_STACK.md)）。WSL2 / 机上レビューでは再現しないため、実装規約として予防するしかない。複数ファイル選択の画面は再現リスクが高く、優先移行対象とする。

**判定フロー / 具体例**: 日報取込み（`useDailyReportImportFlow.ts` の `chooseFiles`）が移行第一号。`open({ multiple: true, filters: [{ name: "CSV", extensions: ["csv", "CSV"] }] })` でパス取得 → `readFile(path)` でバイト読込み。capability は `dialog:allow-open` + `fs:allow-read-file`。`open()` のキャンセルは `null` を返すので state 据え置きで安全に扱える。選択エラーの表示は DSR-03 のインライン 1 スロットに従う。共通 `FilePicker` パターン部品（`src/components/FilePicker.tsx`）は D-054（順9 PR #26）で登録済み。visual / 実装規約は [02-component-catalog.md ⑭](02-component-catalog.md) が、behavior / API 契約は [UI_TECH_STACK.md §6.5.4](../UI_TECH_STACK.md#654-ネイティブダイアログ) が正典。

**関連**: パターン⑥空状態・エラー・ローディング（インライン選択エラー）。review-checklist カテゴリ 9 対応（操作の起点が予測通り動くか）。

---

## DSR-23 プルダウンは shadcn `Select` に統一する

**ルール**: 業務選択肢を表す native `<select>` を使わない。プルダウンは shadcn `Select`（`SelectTrigger`/`SelectContent`/`SelectItem`）に統一する。`SelectItem value=""` は Radix の禁止制約に当たるため、空値 sentinel は非空文字列（`"all"`/`"none"` 等、業務意味に応じて既存ファイル慣習に合わせる）へ変換するか、`SelectValue placeholder` による無 sentinel 方式を使う。

**Why**: owner 決定（R5-3、2026-09-05）。同一画面内に見た目が異なる 2 種類のプルダウン（native と shadcn）が混在していたため統一する。shadcn `Select` は開閉・キーボード操作が一貫し、部門フィルタ等の既存 canonical 実装と揃う。

**具体例**: 空値 sentinel は `SelectItem value="all"`（「すべて」の意味）または `SelectItem value="none"`（「指定なし」「取引先なし」等、明示的な無選択の意味）へ変換する。数値 ID を値に持つ場合は `SelectItem value={String(id)}` へ文字列化し、`onValueChange` で `Number(value)` に復元する。共有 `Select` は空文字の `onValueChange` を無視する（Radix bubble select の echo 対策、`SelectItem value=""` 禁止と対）。

**関連**: パターン⑨検索 + フィルタ。`DepartmentFilter.tsx` が canonical 実装例。review-checklist カテゴリ 9 対応。DSR-24（追加導線 + 選択を伴う候補は picker dialog、本 DSR の例外）。

---

## DSR-24 追加導線を伴う master 参照は picker dialog に統一する

**ルール**: 複数候補から 1 件を選ぶ master 参照（取引先等）のうち、選択に加えて「新規追加」の導線を伴うものは、shadcn `Select`（DSR-23）ではなく picker dialog（検索 input + scroll 一覧 + 枠外固定の追加/閉じるボタン）に統一する。候補が追加導線を持たない選択（部門等）は DSR-23 の `Select` のまま変更しない。

**適用条件**: (1) 呼び出し画面が候補から 1 件を選ぶ操作を持つこと、(2) その候補データが「漸進的に増える complete master data」であること（例: 取引先 = `listSuppliers()` / `createSupplier()` の組）、(3) その画面が候補への追加導線（inline パネル・別 Select 横のボタン等）を持つこと。3 条件のいずれかを満たさない候補は対象外（例: `DepartmentFilter` の部門は追加導線を持たない固定 master のため (3) を満たさず対象外）。追加導線を持たない画面への適用可否は本 DSR の対象外とし、個別の design 判断に委ねる。

**Why**: DSR-23 は見た目の統一を目的にプルダウンを Select へ統一したが、追加導線が伴う候補では Select 自体に行を増やす手段が無く、画面ごとに inline パネル（`ProductForm.tsx`）・Select 横のボタン（`PriceRevisionFilters.tsx`）とバラバラな実装が並立してきた（`onCreated` の戻り値契約が file ごとに異なる — Supplier を返す / 返さない）。owner L3 所感（2026-09-08、`docs/Plans.md` Backlog entry）「追加である以上オレンジでは」「絞り込み列に登録操作が混ざる」は、Select 横に primary 色のボタンを並べる現行実装が、絞り込みの主動線と登録操作を同一視覚面で混同させていることを指す。dialog に閉じ込めることで、画面本体は選択操作だけに単純化され、追加操作は dialog を開いた後だけに現れる別の文脈になる。

**判定フロー / 具体例**: DSR-01（1 画面 1 primary）は画面本体の主動線を変えず、dialog を開いている間だけ dialog 内の「追加の確定」ボタン 1 個を primary（操作の塗り）にすることで維持する（dialog を閉じれば画面には primary が戻らない）。適用例: 一括価格改定 filter・商品登録/修正 form・入庫記録（UI-02）（いずれも候補から 1 件を選ぶ操作を持つ）。非適用例: 部門フィルタ（`DepartmentFilter`、追加導線なし、Select のまま）。取引先管理は選択操作を持たないため DSR-24 非適用。一覧の見た目（検索 input + scroll 箱）だけ picker dialog と揃える（78 SPEC-SUP-D2、SPD-D6）。入庫記録は UI-02-D3 の defer を解除して適用（owner 判断 2026-09-10、取引先 80 件規模で Select からの選択負荷が高い）。

**関連**: パターン⑧ Dialog / 確認（picker dialog 小節）、DSR-23（プルダウンは Select 統一、本 DSR はその例外条項）、DSR-01（1 画面 1 primary）。

---

## DSR-03 Toast vs Alert 使い分け基準

**ルール**: 一過性の操作結果は toast、画面に居座る回復導線つきの状態は inline Alert にする。Alert は置き場所で役割を分ける: **画面上部の Alert 帯はデータ安全系（取込み済み重複・同日追加確認・取得失敗など「進めると危ない / 進めない」状態）の専用スロット**とし、入力検証エラー（選択ミス・形式不備）は**発生源直近のインライン 1 スロット**（毎試行置換・成功でクリア）に置く。toast は両者と併用してよい（即時フィードバック）。

**Why**: ux-principles #1（System status visibility）と #9（エラーは原因 + 解決策）に従う。「保存しました」のような結果は流れて消えてよいが、取得失敗のように利用者が次の手を打つ必要がある状態は、画面に残して回復導線を併置しないと、高齢の利用者がメッセージを見落として手が止まる。さらに、上部 Alert 帯に入力検証エラーまで流し込むと「危ない状態」の警告が日常の入力ミスに埋もれ、データ安全系の警告の重みが失われる（operator 向け注意情報は視線が最初に通る場所に集約する原則と同根）。入力検証エラーは発生源の直近に出す方が、非 IT の利用者がどの操作をやり直せばよいか迷わない。

**判定フロー / 具体例**:

```
利用者の追加操作が要るか？
├─ No（完了通知だけ） → toast（Sonner、自動消去）
└─ Yes（回復・再試行が要る） → inline Alert + 回復ボタン
         ├─ データ安全系（重複ブロック・同日追加確認・取得失敗）
         │    → 画面上部の Alert 帯（destructive / warning トーン）
         └─ 入力検証（選択ミス・形式不備）
              → 発生源直近のインライン 1 スロット（毎試行置換・成功でクリア）+ toast 併用可
```

実例: 商品保存成功は `toast.success`。商品一覧の取得失敗は `Alert variant="destructive"`。サマリカードの取得失敗は Alert + 再試行 — 置き場所はパターン②の 3 パターン規約に従う（独立 query = カード内 / 同一系 query が束ねるカード群 = page-level Alert。daily/monthly `SummaryCardsBar` が後者の canonical variant）。

3 階層の実例（日報取込み）: 上部 Alert 帯は `AlreadyImported`（destructive）/ `AdditionalImportConfirmationRequired`（warningトーン）のデータ安全系専用。ファイル選択の検証エラーは選択ボタン直下の `SelectionErrorMessage`（`role="alert"`、パターン⑥のインライン1スロット）で毎回の選択試行で置換・成功で消去し、`toast.error` を併用する（`DailyReportImportPage.tsx`）。

**状態遷移後の可視性**: 保存・確定・取消などの状態遷移直後は `scrollPageToTop()`（`src/lib/page-scroll.ts`）を呼び、ページ上部の結果表示を利用者に確実に見せる（UI-08-D6「下部に出すと利用者が見落とす」。入庫 / 返品・交換 / 手動販売 / 廃棄 / PLU 書出しの 5 画面で使用済みの共有 util）。

**関連**: パターン⑥空状態・エラー / ⑦Toast / ⑧Dialog。review-checklist カテゴリ 9 対応（状態を変える control に recovery path が残っているか）。

---

## DSR-07 確認ダイアログを出す境界

**ルール**: 破壊的・不可逆な操作、または誤実行が会計・在庫の重複計上を生む高影響操作の直前に確認ダイアログを挟む。復帰・再表示など影響の小さい可逆操作は確認なしで直接実行する。

**Why**: ux-principles #5（Error prevention）と GOV.UK「Do less」の両立。重要操作には確認が要るが、可逆操作にまで確認を挟むと、毎日使う業務で操作が重くなり、利用者が確認を読まず反射的に押すようになる。確認を「本当に効くところ」に絞ることで、確認の重みを保つ。

**判定フロー / 具体例**:

```
その操作は破壊的・不可逆、または誤実行で会計・在庫を重複計上するか？
├─ Yes（廃番化・同日別ファイルの追加取込み等） → 確認ダイアログ（AlertDialog）
└─ No（表示に戻す・再表示等）                  → 直接実行
```

実例: 「廃番にする」は確認あり（`DiscontinueConfirmDialog`）。売上の同日別ファイル追加はactive import全件を示す確認あり（文言正本は function-design 55）。「表示に戻す」は直接実行。

**関連**: パターン⑧Dialog/確認。review-checklist カテゴリ 9 対応（状態を変える control に recovery path が残っているか）。

---

## DSR-11 空状態・エラー文言・Tooltip の表示基準

**ルール**: 空状態は「何が無いか」+「次の一手」を文言で示す。意味を Tooltip だけに閉じ込めない。

**Why**: ux-principles #9（エラーは原因 + 解決策）と WCAG（hover/focus 依存の情報を必須化しない）。「該当する商品がありません」だけでは利用者は止まる。次にどうすればよいか（検索条件を変える / 登録する）まで書く。Tooltip は hover / focus でしか出ず、高齢の利用者や touch 操作では届きにくいため、Tooltip は補足に限る。

**判定フロー / 具体例**: 空状態は見出し（"商品が見つかりません"）+ 説明（"検索条件を変更するか、新しい商品を登録してください"）+ 必要なら Primary アクションで構成する。Tooltip に入れた補足（例: 売上明細数の算出根拠）は、無くても画面の主要操作が成立する内容に限る。

**関連**: パターン⑥空状態・エラー・ローディング / ⑬ステータスバッジ。review-checklist カテゴリ 9 対応（表示文言が業務上の意味と一致しているか）。

---

## DSR-19 作成・保存成功の feedback 規約

**ルール**: 作成・保存の成功後に同じ作業文脈へ戻る flow は、成功 toast を最低保証とする。保存後に利用者が確認または継続操作すべき内容がある場合は、toast に加えて result panel を残し、専用 result step または結果画面へ全面遷移する flow は、その持続的な結果表示自体を完了 feedback としてよい。

**Why**: Peak-End rule の観点では、操作の終端を明示することが一連の体験の評価を安定させる。Zeigarnik 効果の観点でも、完了した操作を未完了のまま記憶に残させず、作業の区切りを明確にする必要がある。参照: Jon Yablonski『UXデザインの法則 ―最高のプロダクトとサービスを支える心理学』第 2 版、相島雅樹・磯谷拓也 訳、オライリー・ジャパン、2025-01（原著 *Laws of UX*）。

**判定フロー / 具体例**:

```
成功後の表示はどこか？
├─ 同じ入力・一覧の作業文脈へ戻る
│    ├─ 完了だけを伝えればよい → toast
│    └─ 確認事項・次の操作がある → toast + result panel
└─ 専用 result step / 結果画面へ全面遷移する
     → 持続的な結果表示（同内容の toast 併用は任意）
```

- **適合先例**: 起票時に実測した 5 方式のうち、toast のみ（取引先改名・統合、PLU 一括、設定保存、帳票出力等）、toast + result panel（入庫・返品交換・手動販売・廃棄の保存）、専用 result step のみ（売上 CSV / 日報取込み確定）、結果画面への全面置換（棚卸し確定）は、上記の表示先と継続作業の差に沿った適合形である。
- **R3 是正対象**: 同じ作業文脈へ戻るのに完了 feedback がない取引先追加と価格改定の行確定は、本則に適合しない。後続 R3 で成功 toast を追加する。
- **duration 階層**: 単純な完了通知は Toaster 全体既定の `3000ms`。商品名・商品コードを含み、成功と同時に読んで確認する重要情報付きの通知は `5000ms`。取消結果、または非 IT operator 向けに結果・次の手の読了時間を確保する通知は `8000ms` とする。時間延長だけでは回復導線にならないため、操作が必要な状態は DSR-03 に従い Alert に残す。
- **toast id**: 同じ操作が連打・再試行・連続 callback で重なり得る場合は、操作種別と必要な対象範囲を表す安定 id を付け、同一通知を置換する。別商品・別帳票など利用者が個別に識別すべき成功まで同じ id で潰さない。単発で重複経路がない通知には id を必須としない。

本規約は DSR-03 を supersede しない。DSR-03 が toast / Alert / Dialog の役割と回復導線の置き場所を決め、本規約はそのうち作成・保存成功の最低 feedback、持続的結果表示との組合せ、duration、重複抑止を refine する。

**関連**: DSR-03「Toast vs Alert 使い分け基準」 / パターン⑦Toast。review-checklist カテゴリ 9 対応（作成・保存成功が適切な feedback で閉じ、重要度に応じた表示時間と重複抑止になっているか）。

---

## DSR-20 destructive 確認 dialog の配置・dismiss 規約

**ルール**: destructive 確認の実行 Action は `variant="destructive"` に統一し、2 ボタンの DOM 順を Cancel → Action とする。Esc・外側クリックは Cancel と同じ経路へブリッジするのを本則とし、明示選択なしの dismiss が実害につながる場合、または未保存内容の扱いを明示選択させる場合だけ、明示 prop で硬化する。

**Why**: Von Restorff 効果により、危険な選択肢を通常操作から視覚的・位置的・言語的に区別すると、確認場面で選択肢を取り違えにくい。未保存編集の確認で Esc を無効にし、利用者自身に「編集を続ける」か「破棄して移動」かを選ばせる friction は、Zeigarnik 効果を意図的に活用して未完了作業を曖昧に閉じない先例である。参照: Jon Yablonski『UXデザインの法則 ―最高のプロダクトとサービスを支える心理学』第 2 版、相島雅樹・磯谷拓也 訳、オライリー・ジャパン、2025-01（原著 *Laws of UX*）。

**配置と dismiss の契約**:

- **variant**: destructive Action は必ず `variant="destructive"` とし、title / description で対象・影響・復帰可否を示し、Action label に実行内容を明記する。色だけを危険性のシグナルにしない。
- **2 ボタン**: DOM 順は Cancel → Action。`sm` 以上では Cancel を左、Action を右に置く。narrow 幅では footer の `flex-col-reverse` により視覚上 Action が上、Cancel が下になるが、これは action の到達性と既存 primitive の一貫性を保つ意図された挙動であり、DOM 順を反転しない。
- **3 ボタン**: 取引先統合 stage 2 を先例とし、DOM 順を「前段階へ戻る secondary」→ Cancel → destructive Action とする。`sm` 以上は同じ左から右、narrow 幅は `flex-col-reverse` により Action → Cancel → secondary の上から下になる。
- **dismiss 本則**: Esc・外側クリック・close control は `onOpenChange` を通じて Cancel と同じ処理へブリッジし、明示 Cancel button と同じ後状態を作る。
- **硬化条件**: (a) 選択を確定せず閉じると保存結果が曖昧になり、再実行による在庫二重計上等の実害へ直結する場合（`CostDiffDialog` 先例）、または (b) 未保存内容を破棄するか継続するかを明示選択させる必要がある場合（`UnsavedChangesDialog` 先例）に限り、Esc・外側クリックを無効化する。単に重要そう、誤操作が心配という理由だけでは硬化しない。
- **硬化手段**: content primitive の `onEscapeKeyDown` / `onPointerDownOutside` で `preventDefault()` し、close button を持つ Dialog では `showCloseButton={false}` を明示する 1 系統を正とする。`onOpenChange` を配線しないことで偶然閉じなくする暗黙硬化は禁止する。確定・Cancel button など許可した close 経路は parent state へ明示的に接続する。
- **AlertDialog 系の制約**: `AlertDialogContent`（`radix-ui` 経由の `@radix-ui/react-alert-dialog`）は `AlertDialogContentProps extends Omit<DialogContentProps, 'onPointerDownOutside' | 'onInteractOutside'>`（@radix-ui/react-alert-dialog 1.1.15 の `node_modules/@radix-ui/react-alert-dialog/dist/index.d.mts:23`）のため、そもそも `onPointerDownOutside` / `onInteractOutside` を受け付けない。AlertDialog の外側クリックは prop 無しで常に非 dismiss（primitive 既定、PR #25 gated amendment `2433199` で実証済み）であり、AlertDialog 系で使える明示硬化手段は `onEscapeKeyDown` の `preventDefault()` のみである（通常の `Dialog` と異なる）。
- **Cancel 文言**: 既定は `キャンセル`。保存中の内容を保持する、または戻り先を具体的に示す方が判断しやすい場合に限り、`編集を続ける` や `残す取引先を選び直す` のような結果・遷移先を表す文言を使ってよい。`戻る` / `やめる` のように後状態が判別できない語へ置き換えない。前段階へ戻る secondary action は Cancel と別の操作として扱う。

DSR-07 は確認 dialog を出すかどうかの境界を決め、DSR-20 は出すと決めた destructive dialog 内の強調・配置・dismiss を決める。DSR-08 との関係では、`variant="destructive"` は色だけの警告ではなく、配置、具体的な Action label、title / description と組み合わせた複合強調である。dialog 内に同型レコードを複数示す場合の構造は DSR-16 に従い、危険性の強調を理由に囲みを反復しない。

**関連**: DSR-07「確認ダイアログを出す境界」 / DSR-08「semantic 色のみで意味を伝えない」 / DSR-16「同型情報のグループ化と囲みの階層」 / パターン⑧Dialog/確認。review-checklist カテゴリ 9 対応（variant、DOM / visual 順、dismiss 本則と硬化条件、Cancel 文言が一貫しているか）。

---

## DSR-15 returnTo 等のリダイレクト系 param は検証してから使う

**ルール**: `returnTo` など遷移先を運ぶ search param は、そのまま `<Link to>` / href に渡さない。「`/` 始まり」かつ「`new URL(value, base)` で解決した origin がアプリの base origin と一致する」ことを検証し、不合格ならアプリ内の既定ルートへフォールバックする。通過した値は `pathname + search` だけを返し、hash は落とす（app が hash を使わないため）。

**Why**: 任意文字列を遷移先として使うと、外部 URL / protocol-relative URL への想定外遷移（open-redirect 型）が起きうる。デスクトップアプリでも業務動線が壊れ、利用者が迷子になる。PR #114-#115 の入出庫 4 詳細ページで `normalizeReturnTo` として確立した規約を全 returnTo 系 param に適用する。旧判定（prefix 文字列一致のみ）は `/\host`（バックスラッシュ）や tab 入り値（`/\t/evil.example`）を `/` 始まりのまま通し、ブラウザの URL 正規化がこれらを別 origin へ解決するため、origin 判定へ強化した（returnTo 衛生 R3、起票時実測、2026-09-17）。

**判定フロー / 具体例**: 戻り link を描画する側は `returnToLinkProps(value, options?)`（`src/lib/return-to.ts`）を使い、`to`(pathname) / `search`(object) へ分解した結果を受け取る。内部の判定は共有 guard `parseReturnTo` が担い、`value` が `/` 始まりで、かつ `new URL(value, "http://inventory.local")` の `origin` が base と一致し、解決後の pathname が `//` 始まりでないときだけ合格とする。合格しない場合（`options.pathname` の不一致を含む）は `null` を返し、呼出側が遷移先ごとの既定ルートへ切り替える（DSR-18）。`/\evil.example` や `/\t/evil.example` は `/` 始まりだが origin が `http://evil.example` に解決されるため拒否する。`/ok#frag` は `/ok`（hash なし）として合格する。`/..//evil.example` のように解決後の pathname が `//` 始まりになる値も拒否し、検証済みの `pathname + search` を再解析しない。`normalizeReturnTo(value, fallback)` は同じ guard を共有する primitive で、文字列 1 本（`pathname + search`）が必要な呼出側のために残す。共通 guard は値ごとの URL 解析を 1 回にし、不正値でも throw しない。新規に returnTo を受ける route を作るときは同じ検証を必ず入れる。

**関連**: パターン①ページヘッダ（詳細ルートの戻る導線）。review-checklist カテゴリ 9 対応（状態を変える control へ戻れるか / 導線が行き止まりにならないか）。

---

## DSR-17 画面遷移と scroll の 3+1 分類

**ルール**: 画面遷移時の scroll は、①同一画面内の状態遷移、②route 遷移を伴う一覧→詳細→戻り、③操作完了に伴う Home への programmatic navigate の 3 分類に、④主ナビゲーション操作による route 遷移を加えた 3+1 分類で決める。分類④は遷移先を常に先頭表示する。発火契機は sidebar 等の主ナビゲーション操作であり、route component の mount ではない。したがって、全画面へ無条件に適用する mount 一律 scroll の禁止と両立する（分類④の起源: owner 所感、PR #17 comment 5463874988）。横 overflow が実発生する一覧（現状 商品一覧のみ）は、表自身を縦横の scroll 箱にしてよい。商品一覧では `<main>` が縦方向（toolbar 分の page scroll を含む）、表自身の箱が横方向（常時）と箱の viewport 基準高さを内容が超えたときの縦方向を担う（Lane 4 Gated Amendment 4）。scroll 復元・分類④（主ナビゲーション遷移の先頭表示）・perPage 変更時の先頭 scroll は main（縦）と箱（横・条件付き縦）の両方を対象にする。

**Why（app 契約の背景）**: 本アプリは persistent な `<main>`（`src/components/layout/RootLayout.tsx`、RootLayout 構成の正本は [52-ui-shared-layout.md §52.1](../function-design/52-ui-shared-layout.md#521-コンポーネント構成)）が route content の唯一の scroll container である（sidebar の Radix `ScrollArea` viewport は chrome の scroll container であり、これとは別枠 — (j) 参照）。route 遷移で `<main>` は unmount されないため、scroll 位置を明示的に扱わないと stale scroll が全画面へ持ち越される。一方、mount 一律の先頭 scroll は一覧→詳細→戻りの位置を失わせることが PR #15 Amendment 2 で実証され、revert 済みである。操作結果の可視性、戻り導線の連続性、主ナビゲーションの予測可能な初期表示を両立するには、mount ではなく遷移の契機ごとに発火条件を分ける必要がある。例外: 横 overflow が実発生する一覧では表自身が縦横 scroll 箱になり、main（縦、toolbar 分の page scroll）と箱（横・条件付き縦）が並行して route content の scroll container になる（Lane 4 Gated Amendment 1、2026-09-06、実装済み。対象画面は商品一覧のみ、詳細は (c)/(j) 参照）。**Lane 4 Gated Amendment 4（2026-09-07、owner 決定 (c) の変形、実装済み）**: 商品一覧では `<main>` が縦方向（toolbar 分の page scroll）、箱が横方向（常時）と箱の viewport 基準高さ（`max-h-[calc(100vh-6.75rem)]`）を内容が超えたときの縦方向を担う。詳細は [UI 一覧の背骨 D — Lane 4 packet](../archive/plans/2026-09-05-ui-list-backbone-d-lane4.md) Lane 4 Gated Amendment 4 参照。

**Why（library 観測事実、TanStack Router 1.168.23）**: 現行 app は `scrollRestoration` 未設定である。`@tanstack/react-router` 1.168.23 の型定義 JSDoc は既定 key を `location.href` と説明するが、実装既定は `location.state.__TSR_key || location.href` である。`__TSR_key` は history entry ごとに新規発行されるため、DSR-18 が維持する `<Link>` の push 戻りでは同じ href へ戻っても既存 cache key と一致せず、既定のままでは位置を復元できない。また cache miss 時の先頭 scroll は `window.scrollTo` と `scrollToTopSelectors` に委ねられ、既定 selector は `['window']` なので route content の唯一の scroll container `<main>` には効かない。cache は sessionStorage（`tsr-scroll-restoration-v1_3`）へ `pagehide` 時に保存され、library は `window.history.scrollRestoration = "manual"` を設定する。document capture listener（`document.addEventListener("scroll", ..., true)`）は `<main>` に限らずあらゆる scrolling element の scroll を拾い、`onBeforeLoad` 時にその位置を positional CSS selector（または `data-scroll-restoration-id` 属性 selector）で cache へ書き込む。`data-scroll-restoration-id` は selector を安定化するだけで cache 対象からの除外にはならず、1.168.15 の router option（`scrollRestoration | getScrollRestorationKey | scrollRestorationBehavior | scrollToTopSelectors`、`router-core/dist/esm/router.d.ts:309-334`）に per-element の除外手段は存在しない。sidebar の `ScrollArea` viewport がこの機構で cache・復元された実例が (j) の起源。これらは版数に依存する観測事実であり、下記の app 契約とは分離し、router 更新時と後続 R3 Contract Probe で再検証する。`@tanstack/router-core` 1.171.3（`@tanstack/react-router` 1.170.5）で再検証（`scroll-restoration.js` の差は `var`→`const/let` の 6 行、`router.d.ts` の option と `router.js` の `resetNextScroll`・`subscribe`/`emit` は同一。版による行の移動は `router.js:415` → `:427`、`:118` → `:119`、`:146-160` → `:147-161`）。以後の版上げでも、観測事実（既定 key・storage key・4 option・`resetNextScroll`・subscribe の順序・document capture listener）のいずれかに差が出たときだけ (f) の実機検証へ戻る。

**判定フロー / 具体例**:

```
scroll を伴う遷移はどれか？
├─ ① 同一画面内の状態遷移（保存・確定・取消）
│    → event-driven で scrollPageToTop() を呼ぶ
│      （DSR-03「状態遷移後の可視性」/ UI-08-D6 を参照）
├─ ② route 遷移を伴う一覧→詳細→戻り
│    → 戻り時の scroll 位置を復元する（先頭 scroll ではない）
├─ ③ 操作完了に伴う Home への programmatic navigate
│    → one-shot の in-memory flag を消費した時だけ Home を先頭表示で開始する
│      （UI-11b-D11 / UI-11b-D12 型。通常の Home 到達では scroll しない）
└─ ④ sidebar 等の主ナビゲーション操作による route 遷移
     → 復元 cache の有無にかかわらず遷移先を先頭表示する
       （発火契機は navigation 操作。route component の mount ではない）
```

分類①の結果表示契約は DSR-03「状態遷移後の可視性」と UI-08-D6 を正本とし、本節では重複定義しない。分類③の現行 producer は復元成功 flow だけであり、将来、操作完了後に Home へ遷移する同型 producer を追加する場合も、通常到達と区別できる one-shot flag の消費時だけ発火させる。

**分類②の実装方式契約（app 契約）**:

- **(a) push 戻りを維持する**: 戻り導線は DSR-18 の `returnTo` を使う `<Link>` push 遷移のままとし、`history.back()` へ変更しない。`returnTo` が遷移元 href を再現し、href key の scroll 復元がその位置を再現する相互補完関係とする。
- **(b) href key を明示する**: TanStack Router の `scrollRestoration` 導入時は `getScrollRestorationKey` を上書きし、`location.href` を key にする。1.168.23 の実装既定 `__TSR_key` に依存しない。
- **(c) container を安定識別する**: route content の唯一の scroll container `<main>` に `data-scroll-restoration-id` を付与する。CSS 階層や class の位置に依存した cache key を使わない。例外: Lane 4 Gated Amendment 1 の対象画面（商品一覧、2026-09-06、実装済み）では、main 自身の `data-scroll-restoration-id="main"`（`src/components/layout/RootLayout.tsx` の `<main>`、不変）に加えて専用 scroll 箱にも `data-scroll-restoration-id="products-list"` を付与し、箱を安定識別する（Lane 4 Gated Amendment 4）。
- **(d) `<main>` を先頭 scroll 対象にする**: `scrollToTopSelectors` へ `<main>` の selector を設定し、復元 miss 時と分類④の先頭 scroll が `window` ではなく実際の scroll container に作用するようにする。例外: Lane 4 Gated Amendment 1 の対象画面（商品一覧、2026-09-06、実装済み）では、`scrollToTopSelectors` に専用 scroll 箱の selector（`[data-scroll-restoration-id="products-list"]`）も加えた 2-selector 配列にする。
- **(e) event-driven 経路を併存させる**: 既存の `scrollPageToTop()` は分類①・③の正本実装として残し、router の復元機構へ吸収しない。
- **(f) R3 で実機前提を検証する**: 後続 R3 は是正を仮適用した Contract Probe と Windows native L3 の両方で、WebView2 の sessionStorage 挙動、cache hit / miss、`scrollPageToTop()` の smooth scroll と router の位置決めの干渉に加え、async content の遅延描画で復元値が clamp された場合に (i) の遅延再適用で保存位置へ到達すること、および route 遷移で再 mount される component の mount focus が分類②の復元を上書きしないことを検証する。いずれかが fail した場合は、本方式を固定せず DSR-17 の方式選定へ戻る revisit trigger とする。(i) の利用者入力・次 navigation による解除は T12、mount focus の `preventScroll` 契約は T15、WebView2 での実効性は Windows native L3 で検証する。
- **(g) 分類④を復元より優先する**: href key は URL が同じなら push / back / 主ナビゲーションの別を表現しないため、同一 href の cache hit では主ナビ発火時は分類④が復元より優先（常に遷移先先頭）する。後続 R3 の router-core 1.168.15 実読 spike では、次の 3 候補はいずれも分類④の単独機構として不成立と確定した。
  - 主ナビ navigation の `resetScroll`: `router.resetNextScroll` へ写される scroll 処理全体の binary gate であり、既定の `true` でも cache hit の復元が先に実行され、先頭 scroll は cache miss 時だけの fallback になる（@tanstack/router-core 1.168.15 の `router-core/dist/esm/router.js:415`、`scroll-restoration.js:118-171`）。
  - `scrollRestoration` の function 化: 対象 location で `false` を返すと早期 return して scroll を一切変更せず、persistent な `<main>` の stale scroll が残るため先頭表示にならない（@tanstack/router-core 1.168.15 の `scroll-restoration.js:122`）。ただし (h) の分類③ route を router の位置決めから除外する機構には使える。
  - 主ナビ目印 + restoration 用の一意 key: `getScrollRestorationKey` は離脱元の保存（`onBeforeLoad`）と遷移先の復元の双方に使われるため、主ナビ entry を一意 key にすると returnTo push 戻りの href key と保存 key が一致せず分類②を壊す。また getKey は setup 時に closure へ捕捉され、遷移ごとに動的差し替えできない（@tanstack/router-core 1.168.15 の `scroll-restoration.js:73,106,114`）。

  採用機構は app 層の遷移先 href 付き one-shot flag + `router.subscribe("onRendered")` とする。router constructor 内の `setupScrollRestoration` が先に購読し、公開 `subscribe` / `emit` は `Set` の挿入順で同期実行されるため、app handler は復元後に `<main>` を instant で先頭へ上書きする（@tanstack/router-core 1.168.15 の `router.js:118,146-160`）。`onRendered` は React の `useLayoutEffect` から paint 前に emit される（@tanstack/react-router 1.168.23 の `react-router/dist/esm/Match.js:112-125`）。同一 href の再クリックでは emit されないため、handler は flag を必ず先に消費し、search を含む完全 href が現 location と一致する場合だけ scroll する。cache hit が残る同一 href への主ナビ再訪と、active 項目再クリック後の無関係な戻り復元を必須検証にする。
- **(h) 分類③の negative 契約を機構分離する**: UI-11b-D12 の negative 契約は smooth scroll（`scrollPageToTop()` 経路）の専有契約であり、router の遷移時位置決めとは機構分離する。`scrollRestoration` の cache miss fallback を UI-11b-D12 の one-shot 発火と解釈してはならない。後続 R3 の必須 Acceptance Criteria に既存 `HomePage.test.tsx` の negative test が regression しないことを置き、必要なら `scrollRestoration` の function 化による分類③対象 route の適用除外を機構候補に含める。
- **(i) async content による初回復元 clamp を 1 回だけ再適用する**: router の復元後、app 層の `onRendered` handler は `getElementScrollRestorationEntry` で `<main>` の保存値を取得し、保存 `scrollY > 0` かつ実 `scrollTop < scrollY`、すなわち初回復元が clamp された場合だけ遅延再適用を armed する。`<main>` subtree の childList を `MutationObserver` で監視し、`scrollHeight >= scrollY + clientHeight` へ到達したとき `scrollTop = scrollY` を 1 回適用して解除する。適用前の `wheel` / `pointerdown` / `keydown`、または次の `onBeforeLoad` でも解除し、armed は常に高々 1 本とする。分類④ flag が現 location と一致した場合は (g) の先頭 scroll だけを行い、遅延再適用を起動しない。例外: Lane 4 Gated Amendment 1 の対象画面（商品一覧、2026-09-06、実装済み）では、main（縦）に加えて専用 scroll 箱（横・条件付き縦）の保存値も対象に、それぞれ独立に、同じ「clamp されていたら 1 回だけ再適用」を行う（Lane 4 Gated Amendment 4）——armed の判定と適用完了は軸ごとに独立させるが、解除（利用者入力・次 `onBeforeLoad`）は両軸一括で行う（round 3 是正、Opus P3）。
- **(j) 復元対象を `<main>` に限定する**: document capture listener は `<main>` 以外の scrolling element（sidebar の Radix `ScrollArea` viewport 等）も positional selector で cache し、`onRendered` は cache の全 selector を無条件に復元するため、`data-scroll-restoration-id` の有無に関わらず sidebar 等の chrome scroll container が route 遷移で復元されてしまう（PR #28 L3 round 2〜3 実観測、`scrollTop 100 → 0` / `0 → 100`）。app 層は router 生成直後（起動時 sweep）と各 `onBeforeLoad`（直前に離脱した location の snapshot に対して）の両方で、`[data-scroll-restoration-id="main"]` 以外の selector entry を `scrollRestorationCache` から削除する。sidebar の scroll 位置は業務上の route state ではなく chrome の一過性 UI 状態であり、保持・復元する対象にしない。副作用として、`<main>` を一度も scroll していない画面への遷移は cache miss のまま先頭表示になる（(d) の fallback が想定どおり機能する）。例外: Lane 4 Gated Amendment 1 の対象画面（商品一覧、2026-09-06、実装済み）では、allowlist に `[data-scroll-restoration-id="products-list"]` も加え、`main`/`products-list` の 2 selector を削除対象から除外する。

**禁止**: route component の mount を契機に無条件で `scrollPageToTop()` を呼んではならない。詳細から戻った際の位置喪失を招くためであり、PR #15 Amendment 2 の revert を再導入しない。また、route 遷移で再 mount される component の mount 契機 `focus()` は `preventScroll: true` を必須とする。focus の native scroll が分類②の復元を上書きするためである。

**関連**: DSR-03「状態遷移後の可視性」 / DSR-18「詳細画面の戻り導線契約」 / UI-08-D6 / UI-11b-D11 / UI-11b-D12。review-checklist カテゴリ 9 対応（操作結果の初期可視性、一覧へ戻る位置の連続性、主ナビゲーション遷移の先頭表示が 3+1 分類に沿って両立しているか）。

---

## DSR-18 詳細画面の戻り導線契約

**ルール**: 「前の画面へ戻る」ラベルの導線は、遷移元の URL（pathname と search state）へ戻す。業務記録詳細 route へ遷移するすべての link は遷移元 URL を `returnTo` として送り、`returnTo` が欠落または不正な場合だけ遷移先ごとの既定 hub へフォールバックする。

**Why**: 入出庫履歴の絞り込み、作業画面の recent list、保存結果、操作ログの調査結果から詳細を確認したあと、無条件に別の hub へ戻すと、ラベルと実挙動が一致せず、検索条件や page を組み直す必要がある。遷移元を本則にしつつ、欠落・不正時は従来の既定 hub を安全側の escape hatch として残すことで、調査の連続性と既存 deep link の互換性を両立する。

**判定フロー / 具体例**:

```
業務記録詳細へ遷移する link か？
├─ Yes → 送信側は現在の pathname + search state を returnTo に直列化して送る
│         （router の href、`useRouterState({ select: (s) => s.location.href })`。
│          文字列を手組みしない — 数字だけの検索語が number 化して型が失われる、GA5 系）
│         在庫変動履歴（StockMovementsPage.tsx の detailReturnTo）は href ではなく、
│         route が検証した search を正規化した object を router と同じ直列化
│         （`defaultStringifySearch`）で組む（66 UI-06c-D9）。
│         受け側が typed parse-back で router の parseSearch を通らない
│         products/lib/return-to.ts の buildProductListReturnTo は手組みを存置する。
│         └─ 詳細側の「前の画面へ戻る」で共通 helper `returnToLinkProps` を通す
│              ├─ DSR-15 の origin 判定を通る → `to`(pathname) / `search`(object) へ
│              │   分解して戻る（文字列 `to` に query を埋め込まない）
│              ├─ `options.pathname` を渡した導線は、解決した pathname が一致しない
│              │   場合も不正値として扱う（例: 在庫変動履歴の「商品と在庫へ戻る」は
│              │   `/products` に pin する。D-138 の後、proposed・未実装。今の build は
│              │   「在庫照会へ戻る」で `/stock` に pin する）
│              └─ 欠落・不正・pin 不一致 → helper は `null` を返し、呼出側が書いた
│                  遷移先ごとの既定 hub へ戻る
└─ No  → その導線固有の契約に従う
```

共通 helper（`normalizeReturnTo` / `returnToLinkProps`、`src/lib/return-to.ts`）は DSR-15 の origin 検証を最低基準とする。これは DSR-15 を supersede せず extend し、DSR-15 が別 PR の宿題としていた共通 util 抽出を、後続実装の必須契約にするものである。`normalizeReturnTo` はフォールバック先を引数で受ける。`returnToLinkProps` はフォールバック先を受けず、不合格を `null` で返す（戻り値の型は `{ to, search } | null`）。空の `to` を「値なし」の印にすると、呼出側が分岐を書き忘れても型検査を通り、押しても遷移しない戻り link になるため、分岐を型で強制する（2026-09-25）。`returnToLinkProps` は `to` に query を埋め込まず `search` object へ分解するため、戻り link は `<Link {...(returnToLinkProps(returnTo) ?? { to: "/inventory/records", search: {} })}>` のように既定 hub を呼出側で併記した形にする（`<Link to={backHref}>` のような文字列 `to` へ query を直書きしない、returnTo 衛生 R3、2026-09-17）。

商品一覧の typed parse-back に使う `src/features/products/lib/return-to.ts` は、この共通 helper へ緩和しない。`pathname` が `/products` または `/products/` の場合だけ許可する exact-allowlist は、復元対象の search 型まで限定する用途で prefix 検証を強化した上位互換として存置する。typed 復元が必要な導線は同様に許可範囲を強化してよいが、共通 helper の最低基準を下回ってはならない。

**関連**: DSR-15「returnTo 等のリダイレクト系 param は検証してから使う」 / DSR-17 分類②「一覧→詳細→戻り」。review-checklist カテゴリ 9 対応（戻りラベル、遷移元 URL、search state、fallback、検証強度が同じ契約を示しているか）。

---

## DSR-01 主動線 CTA（1 画面 1 primary）

**ルール**: 1 画面の主動線（Primary button）は 1 個に絞る。Primary は操作の役割色（`--primary`、現行値は `00-foundations.md` の色の役割表）の塗り 1 個だけにし、それ以外の CTA は 3 段で降格する — 補助アクション（画面の主目的に付随し、別の入力面を開く等の二次操作）は `secondary`（`--secondary` stone-200 塗り + `--border-strong` 枠、色相を付けない中間段）、それ以外（一覧へ戻る等の離脱系操作）は `outline` / `ghost` に降格する。「1 画面 1 primary」は操作の塗りだけを数え、dialog の中で戻せない操作を実行する危険の塗りのボタンは DSR-20 に従う（主動線の数に入れない）。0 primary の画面は昇格を検討し、入口のホームは最重要導線 1 つだけを操作の強調（Home の入口 card、`00-foundations.md` の迷いやすい場面）にする。

**Why**: refactoring-ui §1「Not everything can be important」のとおり、すべてを強調するとヒエラルキーが崩れる。非 IT の利用者は「いま何を押せばよいか」を即断したい。Primary が複数あると、どれが本筋の操作か判断に迷う。GOV.UK「Do less」の精神で、1 画面の本筋操作を 1 つに定める。

**判定フロー / 具体例**: 商品一覧の主動線は「商品登録」1 個（Primary）。一覧へ戻る・修正などは outline。フォーム画面の主動線は「登録する / 保存する」1 個で、「一覧へ戻る」は outline に降格する。取引先 inline パネルの runtime 是正対象は解消済み（PR #50、DSR-24 の picker 化で inline パネルを撤去）。`button.tsx` の `secondary` は `border border-border-strong` を持つ（owner L3 AC-L3-2 を受け、card 地から操作枠を識別できる強さへ是正）。

**関連**: パターン①ページヘッダ / ④フォームセクション。review-checklist カテゴリ 9 対応（既存画面の共通レイアウト継承・別アプリ化防止）。

---

## DSR-08 semantic 色のみで意味を伝えない

**ルール**: 色は `00-foundations.md` の色の役割表の役割に沿って `success` / `warning` / `destructive` 系のセマンティックトークンで当て、`emerald-` / `rose-` などの生 Tailwind 色 class を `src/features/**` に直書きしない。色は二次シグナルとし、意味は日本語テキストか記号が一次で担う（`00-foundations.md` の色の役割表の注記「どの状態も文字か記号を添え、色だけに頼らない」）。アイコンを必須にするのは非中立の①状態 badge（DSR-22、catalog ⑬）と非中立の〈warning・destructive〉Alert（catalog ⑥。destructive は部品が三角 icon を描く）で、入力欄のエラー文（catalog ④ の `FieldError`）・増減の ± のような役割色の文字だけの表示は、文言・記号・下線のどれかを添えればアイコンを要しない（`00-foundations.md` の迷いやすい場面）。

**Why**: WCAG 1.4.1 と inventory-operator-ui の中核ルール（色相だけで業務状態を符号化しない）。赤黄を識別しにくい利用者でも、テキストと記号（状態 badge と非中立の Alert ではアイコン形状も）で意味が読める必要がある。生 Tailwind 色を直書きするとトークン体系から外れ、`00-foundations.md` のパレットと不整合になる。palette 外色の直書きは eslint `no-restricted-syntax`（PR-C 導入）が `src/features/**` + `src/components/patterns/**` で機械検出する。badge の tone family は `00-foundations.md` の色の役割表の役割で分ける: 完了（緑）= 終わったことを伝えるプラスの報告、注意・確認（琥珀）= 手を止めて確かめてほしい注意、危険・失敗（赤）= 警告（owner 回答 2026-09-06、catalog ⑬ 参照。この割り当ては変えない）。利用者の作業が続いている領域と行には進行中の役割を加える（`--ongoing` の家族、00 のカラーパレット表）。ランキング 1 位は③強調の琥珀 pill（地・文字・`--warning` の枠）と行の地に「1 位」の文字を添えて示す（色だけに頼らない。catalog ⑬、D-094）。

**判定フロー / 具体例**: 在庫状態は `Badge` + `lucide` アイコン（`CircleAlert` / `TriangleAlert`）+ 日本語ラベル（`在庫切れ` / `在庫少`）で示す。比較のプラス / マイナスも記号 + テキストを併記する。増減数値の色は補助シグナルとして重ねる: + は `text-success-strong`、− は `text-destructive-strong`、0 は `text-muted-foreground`（記号 + 文言併記は維持する）。

> **是正済み（PR-C）**: 旧逸脱（`StockStatusBadge.tsx` / `ProductListTable.tsx` / `StockDetailContent.tsx` / `SummaryCardsBar.tsx` の `rose-` / `amber-` / `emerald-` 直書き）は semantic shade token（`00-foundations.md`）へ移行済み。rose→red / emerald→green は意図的色補正として L3 承認。再混入は eslint palette 外色 ban と doc-consistency DS3（token HEX 整合）が機械防止する。

**関連**: パターン⑬ステータスバッジ / ⑥空状態・エラー / ⑦Toast。review-checklist カテゴリ 9 対応（業務ステータスが色だけで符号化されていないか / 色トークン継承）。

---

## DSR-21 現在地と選択状態の色分離

**ルール**: 「今どこにいるか」を示す現在地（主ナビゲーションの active link、`aria-current="page"`）は、stone 系 selection tone に操作の役割色の細いバー 1 点（左端バー、`00-foundations.md` の強調の段階 2。現行は `border-l-primary` 系 token）を重ねて有彩色で示す。「何を絞っているか / どのモードか」を示す選択状態（filter chip の on、SegmentedControl の active）は無彩色 stone のままとし、有彩色を使わない。hover は両者とも無彩色。アクセントは背景色ではなく細いバーに限定し、`amber-` 生 class は使わない（DSR-08）。現在地（画面単位）と現在行（レコード単位、DSR-22）は色で区別せず、置き場所（ナビか一覧の行か）と文言で区別する。

**Why**: 『UIデザインの教科書［新版］』（原田秀司、翔泳社、2020）5-3 は「現在地は有彩色、hover は無彩色」で現在地と一時状態を区別する。`00-foundations.md`「4色エリアモデルの扱い」の、ナビの現在地だけを操作の役割色の細いバーで示す規則はこれと一致する。以前の Primary（amber-700）は warning 系（PLU 通知・在庫少）と同系色で、背景全面に使うと警告と現在地の区別が崩れた。操作の色を注意・確認から分けた後も（D-094）、背景は stone に残しバーだけ有彩色にし、選択状態（stone）と現在地（有彩色のバー）の区別を保つ（`02-component-catalog.md` ⑤ の「選択状態の背景色と役割色を分離する」）。

**判定フロー / 具体例**: 対象が「画面の位置」を表すなら現在地 → アクセントあり（例: SidebarLink の active）。対象が「絞り込み・表示モード」を表すなら選択状態 → stone のみ（例: 在庫照会の状態 chip、商品別 / 部門別の切替、日次 / 月次 tab）。迷う場合は「他画面へ移動しても残る状態か」で判定し、移動で消える状態は選択状態とする。

**関連**: パターン SegmentedControl（`02-component-catalog.md` 実装ルール）、DSR-08（色は二次シグナル）。review-checklist カテゴリ 9 対応（現在地と選択状態の色分離）。

---

## 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-10-07 | npm の audit の lane（Issue #135） | DSR-17: library 観測事実を @tanstack/router-core 1.171.3（@tanstack/react-router 1.170.5）で再検証した記録と、以後の版上げで (f) の実機検証へ戻る客観条件を足す。app 契約 (a)〜(g) は不変。 |
| 2026-09-28 | 色と強調の反映（decision-log D-094） | DSR-08: icon を必須にする Alert を非中立の Alert に、ランキング 1 位を③強調の琥珀 pill と行の地に（順位と太字の試しは L3 で採らなかった）、進行中の token を 00 の家族に。DSR-21 の Why を操作の色の分離後の形へ。DSR-22: 現在行のバーと地を進行中の token に、詳細を開いた行の出典を D-094 に、token の列挙から旧現在行背景を外した。DSR-24 の「現行は amber」を外した |
| 2026-09-25 | 画面遷移の戻り先を型で守る（R3） | DSR-15 / DSR-18: `returnToLinkProps` からフォールバック引数と空の `to` の印を外し、不合格を `null` で返して呼出側の分岐を型で強制する。在庫変動履歴の `detailReturnTo` を手組みの存置例外から外し、正規化した search を router と同じ直列化（`defaultStringifySearch`）で組む（66 UI-06c-D9）。手組みの存置は商品一覧の `buildProductListReturnTo` だけになる。 |
| 2026-09-24 | デザインの決まりの組み直し | `## 話題別の索引` を置き、DSR を番号と見出しを変えずに話題の順（配置と一覧 / 入力 / 知らせ方 / 移動と戻り / 色と状態）へ並べ直した。00 の色の役割表・強調の段階と矛盾する記述を直した: DSR-01（操作の塗りだけを数える、0 primary の画面の昇格）、DSR-08（tone family を役割表へ向け進行中を加える、ランキングは順位と太字の試し）、DSR-16（強調の段階との関係）、DSR-21（現在地は操作の役割色の細いバー、現在地と現在行は置き場所と文言で区別）、DSR-22（現在行は進行中の役割、選択行は開いている 1 行に限る、04 原則 14 の参照を原則 6 へ）。DSR-24 の primary の色の語を現行の説明へ。読み方の Why の接地先を 00 と 03 へ |
| 2026-09-17 | PR #78 | DSR-15「判定フロー / 具体例」の主語を、production caller が 0 になった `normalizeReturnTo` から、戻り link を描画する側が使う `returnToLinkProps` へ同期。`normalizeReturnTo` は共有 guard `parseReturnTo` を使う primitive として残す旨を明記。 |
| 2026-09-17 | returnTo 衛生 GA2 | 解決後の pathname が `//` 始まりの値を拒否し、共通 helper の URL 解析を 1 回にして throw を防ぐ契約と、送信側の手組みを存置する 2 site の例外を明記。 |
| 2026-09-17 | returnTo 衛生（R3） | DSR-15 の判定を prefix 一致から origin 一致へ強化（`/\host` / tab 入り値を拒否、hash を落とす）。DSR-18 の共通 helper を `returnToLinkProps`（`to`/`search` 分解、`options.pathname` pin）へ拡張し、送信側は router の href を直列化する本則を明記。 |
| 2026-09-10 | PR #50 | DSR-24 の取引先ピッカー runtime 反映により、DSR-01 の inline パネル是正対象を解消。 |
| 2026-09-10 | PR #49 | DSR-24 を新設し、DSR-23 から例外条項への相互参照を追加。title を「DSR-01〜24」へ同期。owner Human Gate round 1（2026-09-10）に基づき、UI-02-D3 の defer を解除して入庫記録を適用例へ追加。 |
| 2026-09-08 | PR #45 | owner L3 AC-L3-2 で secondary の `--border` 枠が card 地に溶け込むと判定され、操作枠を `--border-strong` へ変更。塗り・文字色・hover は維持。 |
| 2026-09-08 | PR #45 | UI 規約 runtime 反映に合わせ、verbatim 重複の DSR-23 第2ブロックを除去。第1ブロックの本文は維持。 |
| 2026-09-06 | UI 一覧の背骨 D — Lane 4（Gated Amendment 1、実装済み） | DSR-17 の Why・(c)・(j) に、商品一覧限定で `<main>` の代わりに専用 scroll 箱が唯一の scroll container になる例外を注記（PR #40 owner L3 run 1 FAIL・案 X、Plan Review round 1 P1-2）。実装（app-router.ts 側の resolver 導入、Writer commit d6e545c）を含めて実装済み |
| 2026-09-06 | UI 磨き batch 3 design | DSR-22 識別列マッピング表に記録ID表示方針の owner culling 結果（(b) 一覧の表示列から外す、確定 2026-09-06）を注記として追加 |
| 2026-09-05 | UI 一覧の背骨 D — Lane 4 | DSR-22 の上部/下部 pagination 発動条件を改訂: `topSummary`/直接描画による明示 opt-in + `totalCount > 0` で上部常時表示、下部は `totalPages > 1` のときだけ描画（pager ボタンは下部のみ）。識別列固定は商品一覧のみ `identityColumns` prop 活性化で実装済みと明記（他画面は opt-in 未実施のまま） |
| 2026-09-05 | 本 PR | UI 規約補強 design batch。DSR-01 に 3 段 CTA 階層（`secondary` 中間段、`--secondary` stone-200 塗り + `--border` 枠）を追記し `ProductForm.tsx` 二重 primary 候補を runtime 是正対象に記録。DSR-08 に増減数値の色規則（+ success-strong / − destructive-strong / 0 muted-foreground）を追記。DSR-22 の枠 3:1 要件を interactive な操作枠へ narrow 化し、badge（状態/分類/強調）を 3:1 対象外・枠線必須（tone 固有色または `--border`）・非中立の①状態は icon 必須へ改訂（owner v2 mockup 決定）。新規 DSR-23 は起草しない（lane ⑧ が登録） + Human Gate 回答反映（2026-09-06）: DSR-08 Why に tone family の感情区分（緑=報告/琥珀=注意/赤=警告）を追記 |
| 2026-09-05 | ⑧ native select 統一 | DSR-23「プルダウンは shadcn Select に統一する」を新設。title を「DSR-01〜23」に更新（owner 決定 R5-3 の執行） |
| 2026-09-03 | UI 一覧の背骨 D — Lane 2 | `--border-strong` / `--row-current` / `--border` 濃化 / `--input` 統一を globals.css に実装、token 候補値の HEX を canonical（本節）へ正式登録。件数文言は `Pagination` / `PaginationSummary`（`src/components/patterns/Pagination.tsx`）が範囲付き統一形を描画する実装済み表記へ更新。identityColumns 予約 prop と両立 probe（sticky × 識別列固定 × DSR-17）の申し送りを追記 |
| 2026-09-03 | 本 PR | Human Gate + Codex review 是正。件数文言を当時の canonical（`{totalCount} 件中 {page} / {totalPages} ページ`）へ揃え範囲付き統一形を後続 lane での移行対象と明記、上部表示・識別列固定の発動条件と画面→固定列 mapping 表を pin（⑯・04 原則 14 の重複記載を解消）、Badge/outline chip を UI 部品枠 3:1 の対象に拡張（owner Human Gate 所感、廃番 Badge 可読性）、token 候補値の HEX を canonical から撤去し packet / reference 分析 doc へ移動 |
| 2026-09-03 | 本 PR | DSR-22「一覧の器・現在行・UI 部品枠のコントラスト」を新設。title を「DSR-01〜22」に更新。DSR-16（同型情報のグループ化）・DSR-21（現在地と選択状態の色分離）と主題重複しないことを明記（旧 Lane 1 branch の DSR-16 提案を再採番して承継）。Final Review round 2 是正で「低視力 L3」節（forced-colors / DPI 125〜150% / 当たり判定 24×24 / rem・em）を追加し、04-backbone 原則 16 の dangling pointer を解消 |
| 2026-09-02 | 本 PR | DSR-20 に AlertDialog 系の制約（Radix `AlertDialogContentProps` が `onPointerDownOutside` / `onInteractOutside` を型 Omit するため、硬化手段は `onEscapeKeyDown` のみが対象になる）を追記。Plans.md backlog（PR #25 gated amendment `2433199` 起源）の同乗解消。 |
| 2026-09-02 | PR #29 | DSR-17 に (j)「復元対象を main に限定する」を新設。document capture listener が sidebar 等の chrome scroll container も cache・復元してしまう実挙動（PR #28 L3 実観測）に対し、app 層 allowlist prune で main 以外の cache entry を router 生成時と各 onBeforeLoad で削除する契約を追加。Why / (c) の「唯一の scroll container」表現を route content 限定へ訂正。 |
| 2026-09-02 | wave 8 lane 1 | DSR-21「現在地と選択状態の色分離」を新設。主ナビゲーションの現在地は stone 系 selection tone に Primary 左端バーを重ね、filter chip / SegmentedControl の選択状態は stone のまま維持する。 |
| 2026-08-30 | PR #22 | DSR-19「作成・保存成功の feedback 規約」と DSR-20「destructive 確認 dialog の配置・dismiss 規約」を新設。toast 最低保証・3/5/8 秒階層・id 適用基準、および destructive variant・配置・dismiss / 硬化・Cancel 文言を横断契約化。 |
| 2026-08-30 | PR #21 | DSR-17 を 3+1 分類へ拡張。分類④「主ナビゲーションは遷移先先頭」を追加し、分類②の push 戻り + href key + `<main>` 復元、分類間の優先順位、R3 Probe / L3 義務を確定。 |
| 2026-08-30 | PR #20 | DSR-18 新設: 詳細画面の戻り先を遷移元 URL とし、業務記録詳細 link の `returnTo` 送信義務、遷移先ごとの fallback、DSR-15 を extend する共通 helper 方針を確定。 |
| 2026-08-29 | scroll-policy-design 裁定 | DSR-17 新設: 画面遷移と scroll を 3 分類し、同一画面内は event-driven、詳細戻りは位置復元、操作完了後の Home は one-shot flag 消費時だけ先頭表示とする。mount 一律 scroll を禁止。 |
| 2026-08-29 | PR #15（gated Amendment 3） | DSR-16 新設: 同型情報のグループ化と囲みの階層。owner L3-lite 可読性 FAIL（per-card dl 反復の clutter / 比較不能）を受け、NN/g Common Region と GOV.UK summary list を根拠に判断フローを正本化。 |
| 2026-08-16 | PR #79 | SPEC-SDI-D5: DSR-03の同日追加AlertとDSR-07の高影響な重複計上防止境界を正本化。 |
| 2026-09-08 | PR #46 | 入出庫履歴の記録ID列撤去を反映済みへ更新し、一覧の参照行を実装へ同期。(b) 決定とフィルタ維持の契約は不変。 |
