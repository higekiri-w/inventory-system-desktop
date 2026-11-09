# Plan Packet: ㉔ ホーム画面を mockup-c 案へ寄せる（R2）

2026-09-16 起草。owner 決定（2026-09-11「これ好き、採用したい」、同日確定「補助文言は状態の説明であって導線ではない」「前日分未取込みの alert はこのまま」）。出典は Backlog「次に動く lane」先頭「ホーム画面を mockup-c 案へ寄せる」。canonical = `docs/design-system/reference/mockup-c-home.html`（お手本）。wave 11 の lane 1 で、lane 2 ㉕（`docs/plans/2026-09-16-disposal-result-detail-link.md`、廃棄・破損）と file footprint が互いに素。file:line は origin/main `9d6799ff` で実測（Coordinator 2026-09-16）。実装は別 run（Codex）とし、独立 Plan Review 通過後に発注する。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R2
- Execution Mode: fable-window
- Plan Commit: d5ef2635a77054c2002e95585074a34cdd876f75
- Amendments: a9e158cfb2a457e4062c76291986661bdb2615a9, 755facc8beeeccc05dc5edbbe3c6c7f79acd7f52, d9e0b1d8dc4e1b36b90e72e94d061721b0d1a2a4
- Coordinator: Fable 5.1
- Writer: Codex（実装 `807cc249` / docs `232c94da`）+ Sonnet（GA3 の隣接 test 是正・検証・Draft PR。subagent、worktree 分離、Plan Reviewer とは別 fresh context）
- Plan Reviewer: Sonnet（独立 fresh context）
- Final Reviewer: Sonnet + Opus（独立 fresh context。Opus はデザインレビュー観点を含む）
- Final Review Minimum: 2
- Human Gate: ready,merge,manual

manual = owner Windows native L3 1 画面（ホーム: 11 の入口 card が icon + 題名 + 1 行説明で並び、「売上データ取込み」だけが強調され、上段 summary card 3 枚の補助文言が状態の説明になっている（GA4 で 4 枚 → 3 枚）。前日分未取込みの alert は従来どおり。目視と PASS/FAIL のみ）。

遷移記録（append-only）:
- kickoff → spec-check → plan-draft → plan-gate（本 commit）: Risk R2。Design Readiness は SCREEN_DESIGN §ホーム画面「利用者配慮」（全ボタンにタイトル＋説明文 / 売上データ取込みの強調）と 53 §53.1 / §53.5、mockup-c-home.html を十分と引用し、本 lane で SCREEN_DESIGN / 53 / 52 §52.3 / decision-log を実装と同 PR で同期する（S6）。Test Design Matrix は R2 で AC が rg oracle + 既存 test の更新 + 小 test 1 file で閉じるため付けない。
- plan-gate → plan-approved → implementing（本 commit、state-only）: Plan Review round 1（Sonnet、P1/P2 = 0、P3 2）→ in-place 是正 `d5ef2635`（AC6 の mutant (1) kill 経路、`navigation.test.ts` の inventory 追加）。P3 のみのため reviewer 再投入なし（Subagent Budget）。Plan Commit = `d5ef2635`（plan-first `f4b19e21` → 是正を含む確定版）。実装は Codex 発注書 58 で本 commit を HEAD_SHA として開始する。

## Owner Effort Budget

- 介入回数上限: 3（L3 round 1 = 1 回目、2026-09-16。GA4 の是正後に round 2 = 2 回目）
- 実働時間上限: 20分
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

Reason:
ホームの表示のみを変える UI 変更（入口 card の layout・説明文・強調、summary card の補助文言と 1 枚追加）。route / search state / DTO / BIZ / query は不変で、4 useQuery（`useHomeSummary`）の key・引数・派生値も不変（4 枚目の card は既存派生値 `derived.pluDirtyCount` を表示するだけ）。`src/config/navigation.ts` への optional field 追加は sidebar の契約（label / to / icon / status / search / activeMatch）を変えない。R3 に上げる契約変更はない。

## Goal

Goal Invariant: ホームの入口 card が「何をする画面か」を icon + 題名 + 1 行説明で伝え、「売上データ取込み」だけが primary 強調され、上段 summary card の補助文言が状態の説明（導線ではない）になる。前日分未取込みの alert と PLU 通知バーは不変。

### 最小完了条件

- (1) 3 区画 11 card すべてが icon（24px）+ 題名 + 1 行説明（mockup-c の文言、下表）で表示される。
- (2) `ui-07`（売上データ取込み）の card だけが primary 強調（枠 `border-primary` + 背景 `bg-warning-soft`、icon `text-primary`）。
- (3) summary card が 4 枚: 昨日の売上（既存）/ 在庫切れ（補助文言 = 状態説明の新文言、D-H4）/ 在庫少「基準を下回る商品」/ PLU 未反映「レジ反映待ち」。
- (4) SCREEN_DESIGN §ホーム画面、53 §53.1 / §53.5、52 §52.3 の該当記述が新表示に同期し、decision-log に D-089「ホームの補助文言は状態の説明であって導線ではない」。
- (5) test が新表示を固定する（S5）。

### 失敗定義

前日分未取込み alert（`HomePage.tsx:79-86`、destructive + 「最後の取込み精算日」）の変更、`PluNotificationBar` の変更、`HomePage.tsx` の diff（負の oracle AC4）、navigation の `label` / `to` / `icon` / `status` / `search` / `activeMatch` の変更、sidebar への影響、pending 機構（Tooltip + `aria-disabled`）の撤去、`useHomeSummary` の query 変更。

### 非目的

他画面（商品一覧 / 在庫照会など mockup-c / d 系）の採用、destructive Alert の soft 塗り（別 lane、Backlog）、`SummaryCard` pattern への prop 追加、見出し h2 の様式変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 起票時実測（origin/main `9d6799ff`、Coordinator 計測 2026-09-16）

- `src/config/navigation.ts`: `NavItem` に `description` なし（`rg -c 'description'` = 0）。22 項目、ホームが使う 11 = `ui-07` `ui-09a` `ui-06a` `ui-01a` / `ui-02` `ui-03` `ui-04` `ui-05` / `ui-10` `ui-11b` `ui-11a`
- `src/features/home/components/ActionButton.tsx`: `flex-col items-center`（`:45`）の縦積み、`size` prop（`"lg" | "md"`）、`Icon` は `item.icon` を `h-6 w-6` で表示済み。pending 分岐は Tooltip「後続フェーズで着手予定」（`:63`）
- `QuickActionGrid.tsx` / `InventoryActionGrid.tsx`: `grid-cols-2`。`MiscActionRow.tsx`: `grid-cols-3` + `size="md"` × 3
- `SummaryCards.tsx`: `<SummaryCard` × 3（昨日の売上 / 在庫切れ / 在庫少）、`md:grid-cols-3`。在庫切れ・在庫少に補助文言なし。`derived.pluDirtyCount` は `useHomeSummary.ts:62,76` / `types.ts:30` に既存
- `HomePage.tsx:79-86` 前日分未取込み alert、`:88` `<SummaryCards>`、`:90-103` 3 区画（h2 `text-lg font-medium`）
- test: `HomePage.test.tsx` 5 本（`:20-23` で 3 grid と `PluNotificationBar` を mock、`:181` `getAllByText("1 件")` = 2 を固定）/ `SummaryCards.test.tsx` 8 本（B0 characterization、`makeBaseSummary` に `pluDirty` fixture あり）/ `ActionButton` の専用 test なし / `SidebarLink.test.tsx` 17 本（navigation 型を共有） / `src/config/navigation.test.ts` 7 本（`toMatchObject` の部分一致で `description` 追加に非感応、無変更 PASS 対象）
- docs: `SCREEN_DESIGN.md:90`「サマリ3枚」= 1、`:95`「青枠で強調」= 1、`:94`「全ボタンにタイトル＋説明文」（元意図、未実装）/ `53-ui-home.md:33`「3 カード束ね」= 1、§53.5 表に pluDirty の card 行なし / `52-ui-shared-layout.md:110-119` `NavItem` 型 table に `description` なし / catalog ②「ホームの在庫切れ・在庫少 2 カードは共有 lowStock query」（不変） / decision-log 最終 D-088
- mockup-c-home.html: `.action` = `grid-template-columns:24px 1fr`、icon 24、題名 16px/600、説明 14px muted、`min-height:72px`、`.action.primary` = 枠 primary + 背景 primary-soft（amber-100、本 repo に同名 token なし）。summary `.card .sub` = 「37 点」「すぐ確認」「基準を下回る商品」「レジ反映待ち」

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **GA4 追記（owner L3 round 1、2026-09-16）**: **D-H3 撤回** = PLU 未反映 card は PLU 通知バー（件数 + 「PLU 書出しへ」）と情報が重なるため外し、summary は 3 枚（owner「外しで決まり」）。**D-H6** = icon `mt-0.5` + card `py-3.5`（owner「題名が icon より上にずれて見える」、mockup と同値）。**D-H7** = 在庫切れ・在庫少の件数は 1 件以上のときだけ数字に色（在庫切れ `text-destructive` / 在庫少 `text-warning-emphasis`）、0 件は無色。SCREEN_DESIGN:98「赤 / 黄色は危険度を補助する強調」の実装で、label と補助文言が一次情報、色は二次（DSR-08）。owner「一旦見て判断」。**D-H8** = 入口 card の面は現状維持（白の token が無く、`--card` stone-100 は owner が「野暮ったい」と感じている。`--card` を白へ寄せる design lane を backlog 起票して summary と入口 card を一度に扱う）。「当日の売上を記録します」は mockup のまま。summary の icon は見送り（backlog）

- **D-H1 説明文の置き場 = `NavItem.description?: string`**（`src/config/navigation.ts`）。ActionButton は `navItemId` だけを受ける現行（53 D-2「navigation が SSOT」）の延長で、文言の正本を 1 箇所に保つ。sidebar は `description` を読まない。捨てた案: home 内の id → 文言 map（SSOT が 2 つになる）/ ActionButton に `description` prop（呼び出し 11 箇所に散る）
- **D-H2 card の layout と強調**: ActionButton を左 icon 24（`h-6 w-6`）+ 右「題名（`font-semibold`）+ 説明（`text-sm text-muted-foreground`）」の 2 列（`grid-cols-[1.5rem_1fr]` 相当）、`text-left`、`min-h-[4.5rem]`、`w-full` に変える。3 区画とも同じ card・同じ `grid-cols-2`（mockup-c 準拠、`size` prop は撤去し `MiscActionRow` の `grid-cols-3` → `grid-cols-2`）。強調は `variant?: "default" | "primary"` prop で、`primary` は `border-primary bg-warning-soft` + icon `text-primary`（mockup の primary-soft は amber-100 で同名 token が無いため、既存 `--warning-soft`（amber-50）を使い新 token を作らない。枠は mockup どおり primary）。pending 分岐も同 layout のまま Tooltip + `aria-disabled` を維持。捨てた案: 新 token `--primary-soft` の追加（1 用途のための token 追加）/ `Button variant` の新設（`button.tsx` を触る）
- **D-H3 summary card 4 枚目「PLU 未反映」**: `pluDirty` query（独立 query、catalog ② パターン 1 = per-card retry）を `SummaryCard` で表示し、件数 = `derived.pluDirtyCount`、補助文言「レジ反映待ち」。`PluNotificationBar` は不変（bar + card の併存は mockup-c どおり。bar は「PLU 書出しへ」の導線、card は件数の状態表示で役割が違う）。grid は `md:grid-cols-4`。捨てた案: 3 枚のまま（mockup-c と骨格が違い、owner 確定の「レジ反映待ち」を置く場所がない）
- **D-H4 在庫切れの補助文言**: 既定「在庫 0 の商品」（「基準を下回る商品」と同じ「〜の商品」型）。候補: 「在庫 0 の商品」「在庫がない商品」「残数 0 の商品」。owner が別を選べば文言だけ差し替える（Plan Gate 前に回答があればそれを採る）。「すぐ確認」は導線に見えるため採用しない（owner 2026-09-11）
- **D-H5 docs 同期**: SCREEN_DESIGN §ホーム画面（「サマリ3枚」→ 4 枚〈PLU 未反映を追加〉、「青枠で強調」→「primary 枠 + warning soft 背景で強調」、「補助文言は状態の説明であって導線ではない」を利用者配慮に 1 行）/ 53 §53.1（`SummaryCards` 4 カード、`ActionButton` の引数に `variant`、`size` 撤去、`NavItem.description` 参照）と §53.5 表（pluDirty 失敗時は bar 非表示 + toast に加え card が「取得失敗 / 再試行」）と更新履歴 / 52 §52.3 の `NavItem` 型 table に `description?: string` 1 行 + 更新履歴 / decision-log D-089（書式は D-088 と同じ `Status / Decision / Why / Compatibility`）。catalog ② は不変（在庫切れ・在庫少の共有 query 例示は変わらない）

## Scope

- **S1 `src/config/navigation.ts`**: `NavItem` に `description?: string` を追加し、ホームが使う 11 項目に mockup-c の文言を入れる（下表）。他 11 項目は付けない
- **S2 `src/features/home/components/ActionButton.tsx`**: D-H2 の layout、`description` 表示、`variant` prop、`size` prop 撤去。pending 分岐は同 layout で Tooltip 維持
- **S3 `QuickActionGrid.tsx`**（`ui-07` に `variant="primary"`）/ **`MiscActionRow.tsx`**（`size="md"` 撤去、`grid-cols-3` → `grid-cols-2`）。`InventoryActionGrid.tsx` は無変更
- **S4 `src/features/home/components/SummaryCards.tsx`**: 在庫切れ・在庫少に補助文言（`text-sm text-muted-foreground`、昨日の売上の「N 点」と同構造）、PLU 未反映 card 追加（D-H3）、`md:grid-cols-3` → `md:grid-cols-4`。file header comment の「3 サマリカード束ね」も 4 へ
- **S5 test**: `SummaryCards.test.tsx`（4 card、補助文言 3 種、pluDirty error 時の card 内「再試行」= 独立 query なので 1 本）/ `HomePage.test.tsx:181` の `getAllByText("1 件")` 件数を実 fixture に合わせて更新（PLU fixture 1 件なら 3）/ 新規 `ActionButton.test.tsx`（description 表示、`variant="primary"` の class、pending 時の Tooltip + `aria-disabled` 維持、`description` 未定義 item でも描画）
- **S6 docs**: `docs/SCREEN_DESIGN.md` §ホーム画面 / `docs/function-design/53-ui-home.md` §53.1・§53.5・更新履歴 / `docs/function-design/52-ui-shared-layout.md` §52.3・更新履歴 / `docs/decision-log.md` D-089
- **GA4（owner L3 round 1 所感、2026-09-16）**:
  - **S2-b `ActionButton.tsx`**: icon を 2px 下げ（`mt-0.5`、mockup の `margin-top:2px`）て題名の行と中心を揃える。card の上下余白を 14px（`py-3.5`、`buttonVariants` の `py-2` を上書き）にして 72px の箱の中で内容を上下均等にする（D-H6）
  - **S4-b `SummaryCards.tsx`**: PLU 未反映 card を外し summary 3 枚に戻す（`md:grid-cols-3`、header comment も 3 枚へ。D-H3 撤回）。在庫切れ・在庫少の件数（`text-2xl font-semibold` の div）に、件数 ≥ 1 のときだけ色を付ける: 在庫切れ `text-destructive`、在庫少 `text-warning-emphasis`。0 件は無色（D-H7）
  - **S5-b test**: `SummaryCards.test.tsx` を 3 枚に戻し（PLU card の test 2 本を削除、B0 の card 数 assert を 3 へ）、件数の色の test を追加（在庫切れ 2 件 → `text-destructive` あり / 在庫少 1 件 → `text-warning-emphasis` あり / 両方 0 件 → どちらの class も無し）。`HomePage.test.tsx:181` の `getAllByText("1 件")` を 2 へ戻す。`ActionButton.test.tsx` に `py-3.5` / icon `mt-0.5` の class assert を 1 本
  - **S6-b docs**: `SCREEN_DESIGN.md` §ホーム画面「サマリ4枚（…PLU未反映件数）」→「サマリ3枚（昨日の売上、在庫切れ件数、在庫少件数）」、`:98` の「赤 / 黄色は危険度を補助する強調として使う」の末尾に「（件数が 1 以上のときだけ数字に色を付け、0 件は無色。2026-09-16 実装）」。`53-ui-home.md` §53.1 `SummaryCards.tsx` 行を「3 カード束ね（昨日売上 / 在庫切れ / 在庫少）」へ、§53.5 表の pluDirty 行を GA4 前（bar 非表示 + toast のみ）へ戻し、§53.6 の PLU 未反映カード行を削除、§53.1 か §53.5 に在庫切れ・在庫少の件数色（D-H7）を 1 行、更新履歴に GA4 の 1 行。D-089 / 52 は不変
- **S5-b（GA3）** `src/features/backup-restore/BackupRestorePage.flow.test.tsx`: 単一要素検索 `getByText(/昨日の売上/)` 4 箇所（`:223` `:266` `:298` `:333`）を summary title に限定した `/^昨日の売上 \(/` へ。`:286` の `getAllByText` は不変。説明文「今日・昨日の売上明細と集計を確認します」（S1 の表）は変えない。theory: 復元 flow 後にホームへ戻る test が summary title を探す意図で、新しい入口 card の説明文が同じ語を含むため単一要素検索が複数一致した（regression ではなく test matcher の過広）
- **S7（Coordinator、plan-first commit）**: Plans.md / backlog.md の登録

説明文（mockup-c-home.html から転記、S1 の正本）:

| id | 題名 | 説明 |
|---|---|---|
| ui-07 | 売上データ取込み | レジの日報CSVを読み込み、当日の売上を記録します |
| ui-09a | 日次売上 | 今日・昨日の売上明細と集計を確認します |
| ui-06a | 在庫照会 | 商品の在庫数・在庫切れ / 在庫少を調べます |
| ui-01a | 商品検索・一覧 | 登録済み商品を探す・売価や在庫を確認します |
| ui-02 | 入庫記録 | 仕入れた商品が届いたときに記録します |
| ui-03 | 返品・交換 | お客様からの返品・交換を記録します |
| ui-04 | 手動販売出庫 | レジを通さず売った商品の在庫を減らします |
| ui-05 | 廃棄・破損 | 傷んだ・破損した商品を在庫から除きます |
| ui-10 | 棚卸し | 実在庫を数えてシステム在庫と突き合わせます |
| ui-11b | バックアップ・復元 | データの控えを取る・控えから戻します |
| ui-11a | 在庫少の基準 | 「在庫少」と判定する数量の基準を設定します |

## Non-scope

- `src/features/home/HomePage.tsx`（alert・見出し・区画順序）
- `src/features/home/components/PluNotificationBar.tsx`
- `src/features/home/hooks/**`、`src/features/home/lib/**`、`src/features/home/types.ts`
- `src/components/patterns/SummaryCard.tsx`、`src/components/ui/**`、`src/components/layout/**`
- `docs/design-system/**`（catalog / foundations / reference は不変）
- 他画面の feature（GA3 の S5-b `BackupRestorePage.flow.test.tsx` の matcher 限定 4 箇所だけ例外）

## Acceptance Criteria

rg oracle は出力空 = 0 件。baseline は起票時実測（origin/main `9d6799ff`）。

- **AC1** `rg -c 'description:' src/config/navigation.ts` = 11（baseline 0）/ `git diff origin/main..HEAD -- src/config/navigation.ts | rg '^[-+]' | rg -v '^(\+\+\+|---)' | rg -v 'description' | wc -l` = 0（`description` 行以外の diff なし = label / to / icon / status 不変）
- **AC2** `rg -c 'description' src/features/home/components/ActionButton.tsx` ≥ 1（baseline 0）/ GA4: `rg -c 'py-3\.5' 同` = 1 / `rg -c 'mt-0\.5' 同` = 1 / `rg -c 'flex-col items-center' 同` = 0（baseline 1）/ `rg -c 'size="md"' src/features/home/components/MiscActionRow.tsx` = 0（baseline 3）/ `rg -c 'variant="primary"' src/features/home/components/QuickActionGrid.tsx` = 1（baseline 0）/ `rg -c '後続フェーズで着手予定' src/features/home/components/ActionButton.tsx` = 1（baseline 1、pending 機構維持）
- **AC3**（GA4 で改訂）`rg -c '<SummaryCard$|<SummaryCard ' src/features/home/components/SummaryCards.tsx` = 3（GA4 前 4）/ `rg -c 'レジ反映待ち' 同` = 0（GA4 前 1）/ `rg -c '基準を下回る商品' 同` = 1 / `rg -c 'text-destructive' 同` = 1（GA4 前 0）/ `rg -c 'text-warning-emphasis' 同` = 1（GA4 前 0）/ `rg -c 'md:grid-cols-3' 同` = 1 / D-H4 の確定文言 `rg -c '在庫 0 の商品' 同` = 1（baseline 0。owner が別文言を選んだ場合はその文言で読み替え、packet に記録）/ `rg -c 'すぐ確認' src/features/home` = 0（baseline 0、負）
- **AC4**（負の oracle）`git diff --name-only origin/main..HEAD -- src/features/home/HomePage.tsx src/features/home/components/PluNotificationBar.tsx src/features/home/components/InventoryActionGrid.tsx src/features/home/hooks src/features/home/lib src/features/home/types.ts src/components docs/design-system | wc -l` = 0
- **AC5** docs（GA4 で改訂、**GA5 で式を section / 本文に限定**）: `sed -n 86,100p docs/SCREEN_DESIGN.md | rg -c 'サマリ3枚'` = 1 / `sed -n 86,100p docs/SCREEN_DESIGN.md | rg -c 'サマリ4枚|PLU未反映件数'` = 0（§ホーム画面のみ。`:209` `:350` の他画面の「サマリ4枚」は無関係）/ `rg -c '0 件は無色' docs/SCREEN_DESIGN.md` = 1 / `awk '/^### 更新履歴/{exit} {print}' docs/function-design/53-ui-home.md | rg -c 'PLU 未反映'` = 0（本文のみ。更新履歴は append-only で残る）/ `rg -c '青枠' docs/SCREEN_DESIGN.md` = 0（baseline 1）/ `rg -c '3 カード束ね' docs/function-design/53-ui-home.md` = 1（GA4 の S6-b で 3 枚へ戻したため。GA3 以前の「= 0」は撤回）/ `rg -c 'description' docs/function-design/52-ui-shared-layout.md` ≥ 1（baseline 0）/ `rg -c '^## D-089' docs/decision-log.md` = 1（baseline 0）
- **AC6** test: `SummaryCards.test.tsx` ≥ 10 本（baseline 8）/ `ActionButton.test.tsx` 新規 ≥ 3 本 / `HomePage.test.tsx` 5 本 PASS / `SidebarLink.test.tsx` 17 本 PASS（無変更）/ `src/config/navigation.test.ts` 7 本 PASS（無変更）。mutant: (1) `QuickActionGrid` の `variant="primary"` を外す → AC2 の rg oracle（`variant="primary"` count = 0）で kill。test-based kill は対象外（`ActionButton.test` は ActionButton に直接 `variant` を渡す単体 test で、呼び出し忘れは検出しない。Plan Review round 1 P3）。(2) `navigation.ts` の `ui-02` の `description` を消す → AC1 = 10 で FAIL。(3)（GA4 で撤回、後継は GA4 の mutant 2 本 = 在庫切れの色分岐を外す → 色 test FAIL / `py-3.5` を外す → ActionButton test FAIL、Sonnet run で kill 実測）。3 本とも実装後に kill を実測して報告する
- **AC7** `npm run typecheck` / `lint` / `format:check` PASS、最終 `bash scripts/local-ci.sh full` PASS（GA3: run 1 の full は `BackupRestorePage.flow.test.tsx` 4 failed / 1518 passed。S5-b 後に full を再実行し PASS を報告。`rg -c 'getByText\(/昨日の売上/\)' src/features/backup-restore/BackupRestorePage.flow.test.tsx` = 0〈baseline 4〉/ `rg -c '\^昨日の売上 \\\(' 同` = 4〈baseline 0〉）
- **AC8** `bash scripts/doc-consistency-check.sh --target plan` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS
- **AC-L3-2**（GA4、round 2）ホーム目視: (e) 入口 card の icon と題名が揃って見え、文字が上に寄っていない (f) summary が 3 枚で PLU 未反映 card が無い (g) 在庫切れ・在庫少の件数が 1 以上なら数字に色、0 なら無色（在庫少の基準を上げ下げして両方を見る。DB 編集不要）。所感で「色がうるさい」なら D-H7 を撤回し無色へ戻す
- **AC-L3-1** ホーム（Windows native）目視: (a) 11 card が icon + 題名 + 説明で 2 列に並ぶ、(b) 売上データ取込みだけ枠と背景が強調、(c) summary 4 枚と補助文言（在庫切れ = D-H4 の文言、在庫少 = 基準を下回る商品、PLU 未反映 = レジ反映待ち）（GA4 で撤回、後継は AC-L3-2 (f)）、(d) 前日分未取込み alert と PLU 通知バーが従来どおり。PASS/FAIL のみ。fixture 不要（demo seed の件数表示で足りる。件数 0 でも card は出る）

## Design Sources

List the source design docs this plan relies on. Plan Packets are not durable design source of truth.

- Requirements / spec: REQ-301 / REQ-302（在庫アラート表示）、SP-102-07（PLU 通知）、`docs/architecture/ui-task-specs.md` §UI-00（状態管理に PLU 未反映件数を含む）
- Architecture: 該当なし
- Function / command / DTO: 53 §53.1（component 構成）/ §53.2（派生値 `pluDirtyCount`、不変）/ §53.5（部分障害許容）、52 §52.3（`NavItem` 型）
- DB: 該当なし
- Screen / UI: `docs/SCREEN_DESIGN.md` §ホーム画面（レイアウト判断 / 利用者配慮「全ボタンにタイトル＋説明文」「売上データ取込みの強調」）、`docs/design-system/reference/mockup-c-home.html`（お手本）、`docs/design-system/00-foundations.md`（`--primary` / `--warning-soft` token）、catalog ②（サマリカードのパターン 1 / 2）
- Decision log / ADR: `docs/decision-log.md` D-089（S6 で追加）

## Required Design Artifacts

Use `docs/DEV_WORKFLOW.md` Design artifact selection to decide what must exist before implementation.

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | not applicable |
| Command / DTO / generated binding / wire shape | なし | not applicable |
| DB / transaction / audit / rollback / migration | なし | not applicable |
| Screen / UI / route state / Japanese wording | SCREEN_DESIGN §ホーム画面、53 §53.1 / §53.5、52 §52.3 | updated in this PR（S6） |
| CSV / TSV / report / import / export format | なし | not applicable |
| Durable decision / ADR | D-089 | updated in this PR（S6） |

## Registration / Generation Obligations

該当なし（command / route / doc 新設なし、REQ 追加なし、bindings 非接触、`generate:routes` 不要）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| UI-00（入口） | SCREEN_DESIGN §ホーム画面 利用者配慮 / 53 §53.1 | D-H1 / D-H2 | 説明文は navigation SSOT、layout は mockup-c。捨てた案: home 内 map / prop 散在 / 新 token | S1 / S2 / S3 | `ActionButton.test.tsx` |
| REQ-301 / 302 / SP-102-07 | SCREEN_DESIGN §ホーム画面 レイアウト判断 / 53 §53.5 / catalog ② パターン 1 | D-H3 | 4 枚目 PLU 未反映 card。捨てた案: 3 枚のまま | S4 | `SummaryCards.test.tsx` |
| REQ-301 | SCREEN_DESIGN §ホーム画面 利用者配慮 | D-H4 / D-089 | 補助文言は状態の説明で導線ではない。「すぐ確認」不採用 | S4 / S6 | `SummaryCards.test.tsx` |
| — | 53 / 52 / SCREEN_DESIGN / decision-log | D-H5 | docs を同 PR で同期 | S6 | — |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（SCREEN_DESIGN §ホーム画面の「全ボタンにタイトル＋説明文」が元意図で、本 lane はその実装。D-089 と 53 / 52 を同 PR で更新）
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-089（decision-log、S6）。D-H1〜D-H3 は 53 §53.1 / 52 §52.3 / SCREEN_DESIGN へ
- Assumptions and constraints: `derived.pluDirtyCount` は既存派生値（53 §53.2）で query 追加なし。token は既存 `--primary` / `--warning-soft` のみ
- Deferred design gaps, risk, and follow-up target: mockup-c の primary-soft（amber-100）と本 lane の `--warning-soft`（amber-50）は 1 段違う。L3 で owner が薄いと感じたら token 追加を別 lane で判断
- Test Design Matrix can cite design decision IDs or source doc sections: yes（D-H1〜D-H4。R2 のため専用 Matrix は付けず、Test Plan に理由を記す）
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: alert / bar / HomePage / hooks / navigation の非 description 行の不変を AC1 / AC4（負の oracle）で機械検査

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable | — |
| Fact check / design decision split | 事実 = 起票時実測（card 数、`size` prop、`description` 不在、docs の記述）。判断 = D-H1〜D-H5 | 本 packet |
| Lifecycle / retry | PLU 未反映 card は `pluDirty` の loading / error / data を独立判定（catalog ② パターン 1）。error 時は bar 非表示 + toast（既存）に加え card 内「再試行」 | S4 / S5 |
| Operator workflow | 入口 card の説明で「何をする画面か」が読める。強調は 1 つだけ（DSR-01 系の主動線 1 つ） | AC-L3-1 |
| Replacement path | not applicable | — |
| Data safety / evidence | 表示のみ、データ変更なし | — |
| Reporting / accounting semantics | not applicable | — |
| Manual verification | 1 画面の目視 | AC-L3-1 |
| 環境・再現性 | URL state 不変 | — |

## Design Readiness

State whether the design is ready for implementation.

- Existing design docs are sufficient because: SCREEN_DESIGN §ホーム画面の利用者配慮（タイトル＋説明文、売上データ取込みの強調）と mockup-c-home.html が目標の見た目を定め、53 §53.1 / §53.5 が component と部分障害の契約を持つ。本 lane は新 component・新 token・新 query を作らず、既存の `ActionButton` / `SummaryCard` / `NavItem` の範囲で mockup-c に寄せる。docs は表示の差分（card 数、強調の色、補助文言の決定）を S6 で同期する
- Source docs updated in this PR: SCREEN_DESIGN §ホーム画面、53 §53.1 / §53.5、52 §52.3、decision-log D-089（S6。いずれも実装 run で更新。本 plan-first commit は packet / Plans.md / backlog.md のみ）
- Design gaps intentionally deferred: primary-soft token の要否（L3 で判断）
- Durable decisions discovered in this plan and promoted to source docs: D-089

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI のみ
- Backend function design: 非接触
- Command / DTO / data contract: 非接触
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 説明文 11 + 補助文言 3（D-H4 は owner 確認）
- Error, empty, retry, and recovery behavior: PLU 未反映 card の error / loading を `SummaryCard` の既存 3 状態で扱う
- Testability and traceability IDs: UI-00 / REQ-301 / REQ-302 / SP-102-07、REQ 新設なし

## Contract Probe

N/A: 外部前提なし（library / OS 挙動に依存しない。Tailwind の任意値 class `grid-cols-[1.5rem_1fr]` / `min-h-[4.5rem]` は既存 build 設定で通常どおり生成される。R2）。

## Contract Coverage Ledger

R2 のため任意だが、docs 同期の網羅性を独立 review で確認できるよう置く。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D-H1 / D-H2（入口 card の説明と強調） | S1 / S2 / S3 | `ActionButton.test.tsx` | AC-L3-1 (a)(b) |
| D-H3（PLU 未反映 card、パターン 1） | S4 | `SummaryCards.test.tsx` | AC-L3-1 (c) |
| D-H4 / D-089（補助文言は状態の説明） | S4 / S6 | `SummaryCards.test.tsx` | AC-L3-1 (c) |
| 前日分未取込み alert / PLU 通知バー不変 | 非接触 | AC4（負の oracle）+ `HomePage.test.tsx` 既存 | AC-L3-1 (d) |
| navigation の label / to / icon / status 不変 | S1（description 行のみ） | AC1 + `SidebarLink.test.tsx` 既存 17 本 | — |

## Test Plan

For R3/R4, include or link a Test Design Matrix.

Test Design Matrix は付けない（R2、AC が rg oracle + 既存 test の更新 + 小 test 1 file で閉じるため）。

- targeted tests: S5 を実装と同 commit で更新・追加（AC6）
- negative tests: mutant 3 本（AC6）。`description` 未定義の item を `ActionButton` に渡しても描画が崩れない（`ActionButton.test.tsx`）
- compatibility checks: navigation の非 description 行 diff 0（AC1）、sidebar test 無変更 PASS
- data safety checks: not applicable
- main wiring/integration checks: `HomePage.test.tsx`（実 hook 経由）で summary 4 枚と alert が共存すること（`:181` の件数更新）
- state lifecycle（PLU 未反映 card）: loading = Skeleton / error = card 内 Alert + 再試行（bar は非表示 + toast のまま）/ data = 「N 件」+「レジ反映待ち」/ refetch = 再試行ボタンで `pluDirty.refetch`

## Boundary / Wire Contract

not applicable（DTO / wire / route / search 非接触）。

## Review Focus

- D-H1 の `NavItem.description` が sidebar 側に副作用を持たないか（`SidebarLink` は読まない）。D-H3 の bar + card 併存が冗長に見えないか（mockup-c どおりだが design review 観点）。D-H2 の色 token の選択（枠 primary + 背景 warning-soft）が foundations の用途表と矛盾しないか。`HomePage.test.tsx:181` の件数更新が fixture と一致しているか

## Implementation Results

[PR #70](https://github.com/higekiri-w/inventory-system-desktop/pull/70) で実装し squash merge 済み（`41c2e3e2`、2026-09-16）。Writer = Codex 発注書 58（実装 `807cc249` / docs `232c94da`、full で他 feature の test matcher 衝突により停止 → GA3）+ Sonnet subagent（GA3 の test 是正 `61c558fb`、GA4 の是正 `12969410` / `26658963`）。Final Review round 1 @ `61c558fb` = Sonnet pass A（P3 3）/ Opus pass B（P2 2 → SCREEN_DESIGN 同期 `e52d0e0d` + L3 観察項目）/ Sonnet closure。owner L3 round 1 の所感 → GA4（icon / 題名の揃え、PLU 未反映 card 撤去、件数の状態色、card 面は token lane へ）→ GA5（AC5 の oracle）→ origin/main 単段 merge `4c5fa0bb` → Opus closure（P2 2 = docs 記述、本 closeout で是正）→ helper 要求で broad 2 本を現 head で取り直し（Opus + Sonnet、pass）。owner L3 round 2 PASS（「色も一旦これでいい」）。介入 2 / 予算 3、Codex relay 1 / 上限 2

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none.

### Plan Review round 1（2026-09-16、plan-gate、Sonnet、裁定 Coordinator）

- P1/P2 = 0。起票時実測は全件現物と一致（reviewer が rg で再計測）
- P3-1（AC6 mutant (1) の kill 経路として挙げた test が S5 に無い）= accept → mutant (1) は AC2 の rg oracle で kill と明記、test-based は対象外
- P3-2（`src/config/navigation.test.ts` 7 本が test inventory に無い）= accept → 起票時実測と AC6 の PASS 対象に追加（`toMatchObject` の部分一致で `description` 追加に非感応）
- reviewer 確認済み: D-H1（`SidebarLink` は `description` を読まない）、D-H2（`--warning-soft` の用途欄外流用は既存 pattern）、D-H3（catalog ② パターン 1 と整合、53 §53.4 の bar 条件不変）、`HomePage.test.tsx:181` は fixture 実測で 3 件に一意、`MiscActionRow` の 2 列折返しは mockup-c と同構造、L3 Eligibility 充足
- 判定: Plan Gate 通過可（P3 のみ、reviewer 再投入なし）

### Gated Amendment 3（2026-09-16、Codex 発注書 58 run 1 の AC7 停止）

- run 1（HEAD `2cc4c331`）: Writer は S1〜S6 を実装（`807cc249` runtime + test、`232c94da` docs）、AC1〜AC6 / AC8 PASS、red 8 → green 71、mutant 3/3 kill。AC7 の `local-ci.sh full` で `src/features/backup-restore/BackupRestorePage.flow.test.tsx` 4 本が失敗（`getByText(/昨日の売上/)` が summary title と新説明文「今日・昨日の売上明細と集計を確認します」の 2 要素に一致）。他 feature は Non-scope のため停止（正しい挙動）。commit は local branch に保持、push / Draft PR 未実施、worktree 除去済み。Coordinator が 2 commit を push（`232c94da` = origin tip）
- 原因: Coordinator の隣接 test の洗い出し不足（説明文の語が他 feature の正規表現 matcher と衝突する可能性を見ていなかった）
- 是正: Scope に S5-b（matcher を `/^昨日の売上 \(/` に限定、4 箇所）を追加、Non-scope に例外を明記、AC7 に再実行と rg oracle を追記（本 commit）。説明文・runtime・docs は不変
- 残作業（S5-b + AC7 の full 再実行 + AC1〜AC8 の再確認 + Draft PR）は Sonnet subagent の Writer run で行う（Codex relay は 1/2 のまま温存。Writer field に併記）。Codex の実装 commit は変更しない
- 教訓: 新しい表示文言を入れる lane では、その語を `rg` で全 test に当てて regex matcher の衝突を起票時に洗う

### Gated Amendment 4（2026-09-16、owner L3 round 1 の所感）

- round 1（head `e52d0e0d`）の所感: 題名が icon より上にずれて見える → 修正 / PLU 通知バーと PLU 未反映 card が重なる → card を外す（決定）/ 説明文は最小幅で 2 行になるが枠内 → 変更なし / 入口 card の面は背景と別にしたいが summary の色は野暮ったい → token の design lane へ（D-H8）/ 在庫切れ・在庫少は 1 件以上のとき数字に色 → 一旦入れて見る（D-H7）/ summary の icon → 見送り（backlog）
- 是正: Scope に S2-b / S4-b / S5-b / S6-b、設計判断に D-H3 撤回・D-H6〜D-H8、AC2 / AC3 / AC5 の改訂と AC-L3-2 を追加（本 commit）。Writer = Sonnet subagent（GA3 と同じ）。是正後は closure audit（Opus、design lens）→ owner L3 round 2（介入 2/3）
- backlog 起票（closeout で）: `--card` を白へ寄せる design lane（summary と入口 card の面を mockup どおり白に、foundations の判断更新）/ summary card の icon / 59:21・SCREEN_DESIGN:73 の旧記述 / `size-6 h-6 w-6` / mockup-d の旧前提

### Gated Amendment 5（2026-09-16、GA4 実装 run の AC5 報告）

- Writer（Sonnet、head `26658963`）は S2-b / S4-b / S5-b / S6-b を実装し AC1〜AC4 / AC6〜AC8 PASS、GA4 mutant 2/2 kill、full PASS。AC5 の 3 式が現物と食い違うと報告（読み替えず実測を報告 = 正しい挙動）: (1) `サマリ4枚` は他画面 `:209` `:350` に無関係な出現 (2) `PLU 未反映` は 53 の更新履歴（append-only）に残る (3) `3 カード束ね` は GA4 の S6-b で戻したため 1 が正
- 原因: Coordinator の oracle 式が section を限定しておらず、GA4 で S6-b を改訂した際に AC5 の該当式を更新していなかった
- 是正: AC5 の 3 式を section / 本文に限定し、期待値を GA4 の Scope に合わせる（本 commit）。実装・docs は不変
- 教訓: docs の rg oracle は section 範囲（`sed -n` / `awk`）で限定し、GA で Scope を変えたら同 commit で AC の式を rg で sweep する

### Final Review（2026-09-16）

- **round 1 pass A**（Sonnet、broad、head `61c558fb`）: pass（P1/P2 = 0、P3 3）。対象 6 test file 全 PASS、typecheck PASS を実測。P3-1（59-ui-shared-patterns.md:21「HomePage 3 カード」が当時の 4 カード実装に未同期）= accept、closeout で同期（GA4 で 3 カードへ戻ったため本 closeout では変更不要と確認済み）。P3-2（`ActionButton.tsx` の `size-6 h-6 w-6` 重複）= 記録のみ。P3-3（`reference/mockup-d-home-sales-admin.html` の旧前提記述）= 記録のみ。D-H1〜D-H4 の確認済み事項を含む（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/70#issuecomment-5694384991)）
- **round 1 pass B**（Opus、broad、デザイン / operator UI 観点、head `61c558fb`）: P2 2 / P3 9（P1 なし）。P2-1（SCREEN_DESIGN.md:91,93 の「大きなボタン」「小さく」が `size` 撤去後の実装と矛盾）= accept、同 PR で是正 → closure `e52d0e0d`。P2-2（`--warning-soft` が PLU 通知バーと主動線 card の両方の塗りで意味が紛れ得る）= accept、code 不変で L3 観察項目へ。P3 群（card 内文字の上寄り / 面の差 / 「当日」表記 / grid 幅 / PLU バー併存 / 色相 1 系統 / `size-6 h-6 w-6` 重複 / hover 塗り置換 / test の navigation 書換え）は L3 観察項目または記録のみ（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/70#issuecomment-5694448225)）
- **closure**（Sonnet、head `e52d0e0d`）: closure pass（新規 P1/P2 = 0）。差分は SCREEN_DESIGN.md 2 行の docs 1 commit のみ。review = pass、manual は owner 待ち（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/70#issuecomment-5694480219)）
- owner L3 round 1（head `e52d0e0d`）の所感 → GA4（本 packet 上記）→ GA5 → origin/main 単段 merge `4c5fa0bb`
- **closure**（Opus、design lens、head `4c5fa0bb`）: closure pass。D-H6 / D-H7 / D-H3 撤回を実装で確認、AC oracle 全件と AC4 負 oracle を再実測して一致。**新規 P2 2 件**: P2-1（packet の `manual =` 行・AC6 mutant (3)・AC-L3-1 (c) に GA4 前の「summary 4 枚 / PLU card」が残存）= accept、**本 closeout で是正**（本 commit で上記の通り是正済み）。P2-2（Plans.md lane 1 行が「Draft PR 未作成、介入 0/3」のまま）= accept、**本 closeout で完了行へ**（下記 Plans.md 更新で対応）。**P3**: D-H8 の backlog 文言に「primary card の塗り = `--warning-soft` 流用の是非（mockup-c は `--primary-soft` amber-100 で別 token）」を追記 / `--warning-emphasis` と `--primary` が同 HEX で PLU バー表示中は amber が 3 役という観察条件 / `SummaryCards.test.tsx` の `pluDirty` override 引数が死に配線、次に触る lane で削除（3 件とも本 closeout の backlog.md「記録目的」で記録）。review = pass（broad 2 @ `61c558fb` + closure @ `e52d0e0d` + closure @ `4c5fa0bb`）、manual = pass（owner L3 round 2 @ `4c5fa0bb`）（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/70#issuecomment-5695336945)）
- **broad 取り直し 2 本目**（Sonnet、head `4c5fa0bb`、GA4/GA5 反映後の再監査）: pass（P1/P2/P3 = 0）。helper の要求で Amendments 増加後の現 head で broad を取り直し。AC1/AC2/AC3/AC5（GA5 版）の oracle を逐語再実測して全件一致、vitest 16 file / 99 tests 全 PASS、typecheck PASS（[comment](https://github.com/higekiri-w/inventory-system-desktop/pull/70#issuecomment-5695391365)）
- manual = owner L3 round 1（`e52d0e0d`、所感 → GA4）/ round 2（`4c5fa0bb`、PASS「色も一旦これでいい」）
