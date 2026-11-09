# Plan Packet: デザインの決まり runtime lane A（色と強調）

2026-09-27 起票。起点は `e7c22f8f`（origin/main）。デザインの決まりの組み直し（[D-091](../../decision-log.md#d-091-デザインの決まりを見る人の受け取り方から組み直す2026-09-24)、PR #98）が `docs/design-system/00-foundations.md` に置いた色の役割・強調の段階・迷いやすい場面を、token・横断部品・全画面へ反映する。範囲の正本は [backlog](../../backlog.md) の「デザインの決まり runtime lane A（色と強調）」の項目で、本 packet は現行 main の site を `rg` で数え直し、移すか除外かを決めた。実装は plan-approved の後に Writer（別 context）が行う。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R3
- Plan Commit: dbd5dd5412070fdcb32d382fd31fe4971e9afb67
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（worktree `.claude/worktrees/design-color-emphasis`、branch `agent/design-color-emphasis`）
- Plan Reviewer: Opus 5.5（fresh subagent、Writer と別 context）+ Codex（GPT-5.6 Sol か GPT-6 Astra、owner が起動時に指定）
- Final Reviewer: Fable 5.1（fresh subagent。design lane と R3 以上の Claude 側）+ Codex（GPT-5.6 Sol か GPT-6 Astra）。互いに独立で、後の reviewer に先の結果を見せない
- Final Review Minimum: 1
- Human Gate: ready,merge,manual

manual = owner が before / after（と試し）を実機で並べて見る L3（AC-L3-1〜15）。Final Review Minimum は規則どおり 1（R4 でなく、予定 file に `scripts/ci/classify-changes.sh` が workflow と判定する path が無い）。R3 で operator が読む状態の見え方を全画面で変えるため、Contract Audit の 2 本目の推奨（DEV_WORKFLOW Contract Audit「operator-visible state lifecycle」）に従い、Final Reviewer 2 本を運用で回す。

遷移記録（append-only）:
- kickoff → spec-check → plan-draft → plan-gate（本 commit、plan-first、2026-09-27、起草役）: Risk R3 を記録。Design Readiness が D-091 と `docs/design-system/00-foundations.md`（色の役割・強調の段階・迷いやすい場面）・`01-decision-rules.md`（DSR-08 / DSR-21 / DSR-22）・`02-component-catalog.md`（⑥ ⑫ ⑬）を実装に足りると引用するため、spec-check → plan-draft の許容 skip を使う（正本で決まっていない 2 点〈card の面色・muted の文字色の値〉は Non-scope に置き、design へ戻さない）。packet と Test Design Matrix を同じ commit に置く。decision-log の番号は **D-094** を予約する（D-092 / D-093 は並走のハーネス PR2 / PR3 が予約済み）。
- owner 回答の反映（Phase は plan-gate のまま、2026-09-27、起草役、本 commit）: owner の回答（Coordinator の中継）を反映した。card の面色 = 別 lane（Non-scope のまま理由を差し替え）、muted の文字色 = `--muted-foreground` を濃くする（Scope に S19 と D-CE16 を足し、候補 3 つを L3 の試しにする）、返品・交換の選択カード = 操作の色へ追従のまま、Home の入口 card の試しの地 = #E6F0F0、D-CE8〜10 は事前確認なしで L3 の試しで採否（Coordinator 判断）。介入の消費は 2 になった。
- Plan Review round 1: 両 reviewer とも reject、plan-gate のまま是正（裁定 r1、2026-09-27、起草役、本 commit）。Plan Commit は pending のまま。裁定の B1-1〜B1-12 を Scope / AC / L3 / Owner Effort Budget / Matrix へ反映した（中身は裁定書、Review Response には書かない）。
- 予算の承認（2026-09-27、Coordinator の記録）: owner が介入 19・実働 75 分・relay 6 を承認（この change での介入 4 回目）。plan-gate のまま Plan Review round 2 へ進む。
- Plan Review round 2: 両 reviewer とも reject、plan-gate のまま是正（裁定 r2、2026-09-27、起草役、本 commit）。Plan Commit は pending のまま。裁定の B2-1〜B2-11 を反映し、予算は判断点を数え直して改定案を owner 承認待ちで書いた（中身は裁定書、Review Response には書かない）。
- 予算の再承認（2026-09-27、Coordinator の記録）: owner が介入 24・relay 7・実働 75 分を承認（この change での介入 9 回目）。plan-gate のまま Plan Review round 3（上限）へ進む。
- 上限の Plan Review round 3 の結果（2026-09-28、起草役、本 commit）: 両 reviewer とも reject、一括是正（裁定 r3）。round 4 は出さず、plan-gate のまま裁定の B3-1〜B3-11 を反映した。Plan Commit は pending のまま。介入の上限は 25 に改め owner 承認待ち（plan-approved の承認と同じ 1 回で求める）。
- plan-gate → plan-approved（2026-09-28、Coordinator、本 commit）: Plan Review round 3（上限、対象 `b3844379`）は両 reviewer とも reject（P1 0）。round 天井に従い追加の round は回さず、裁定 r3 の一括是正（`dbd5dd54`）で全件を反映し、Coordinator が予算表・AC12 の baseline・L3 の DB の前提を現物で確かめた。owner 承認（2026-09-28「返事二つOKだよ」、この change での介入 11 回目、介入の上限 25）のもと plan-approved。Plan Commit = 本 commit の親（`dbd5dd54`）。
- plan-approved → implementing（2026-09-28、Coordinator、state-only）: Writer（Opus 5.5 subagent の worktree run）へ実装を発注する。Writer の開始 HEAD は本 commit。
- L3 round 1 の答えの反映（2026-09-28、Coordinator、S18、Phase は implementing のまま）: owner の L3 round 1（2026-09-28、PR head `212d8f19`・試し `25c7c5bc`・M1 `5adf4682`）の答えを記録する。**PASS** は AC-L3-1 / 7 / 8 / 11 / 12 / 13 / 15。**採る**は 9（詳細を開いた行、D-CE7）、10（取込みの手順の表示、D-CE8）、4（Home の入口 card、D-CE12、試しの `bg-ongoing-soft`）、5（日報の取込み済みの badge、試しの危険・失敗）。**採らない**は 3（最新・上書き件数、D-CE3、琥珀 pill のまま。D-094 に恒久の例外として記録）、6（Home の前日分の知らせ、危険・失敗のまま）。muted の文字色（D-CE16）は **M2 #6b6560**。owner の懸念「新しいデザイン（card の面色・書体）で疑似的なちゃぶ台返しになるのでは」を受け、D-094 の Revisit に「card の面色・書体の lane の L3 で muted を見直す」を書く（owner の選んだ値は変えない）。**2（ランキング 1 位、D-CE13）** は試し（順位と太字だけ）を採らない。owner の新方向「1 位を分かりやすく明示、なんなら 3 位まで」は分岐表の 2 通りの外なので、owner 決定 2026-09-28（この change での介入 21 回目）により、本 lane は分岐表の「採らなかったとき」（1 位の行に琥珀の pill と地、`--rank-top-*` を残す）で閉じ、2 位・3 位の見せ方は別 lane として backlog に起こす（Gated Amendment にしない）。同じ回で介入の上限を 25 → 26 に改めた。L3 の所感の別件（入庫記録の単位の並び、stone 以外の中立色）は Writer が backlog へ起こす。採否の反映は S15 / S16 / S17 で Writer に発注する。
- implementing → archive（2026-09-28、Coordinator の closeout、本 commit）: PR #116 を squash merge（`e188366e`、2026-09-28、helper 経由）。packet と Matrix を archive へ移送し、Implementation Results と Review Response を記録。closure 2 の P3（backlog の旧 04 の (a) が #118 で解消した幅の混在を未了のまま書いている）を backlog で直した。Plans.md と backlog を同期。

## Owner Effort Budget

- 介入回数上限: 26（owner 承認 2026-09-28、この change での介入 21 回目。L3 round 1 の 2 の新方向を別 lane に分ける判断と同じ 1 回で、その判断 1 回分を足した）。その前は 25（owner 承認 2026-09-28、この change での介入 11 回目。その前の値は 24〈owner 承認 2026-09-27（再改定、この change での介入 9 回目）〉、その前は 19〈owner 承認 2026-09-27「介入19・75分・relay 6で承認」〉。裁定 r3 の B3-6 で、予算の再改定の承認と Plan Review round 3 の Codex の起動を消費済みにし、round 3 後の一括是正の plan-approved の承認 1 回を見込みに足した。合計 25 は 24 を 1 超えるため、plan-approved の承認と同じ 1 回〈この change での介入 11 回目〉で承認を得た。下の表）。closeout 時の消費 25 / 上限 26 = 下の表の消費 21、Final Review の Codex の起動 1、L3 round 2 の合否 1、Ready 1、merge 1。Codex closure の予備 1 は使わなかった（closure は Claude 側で 2 本）
- 実働時間上限: 75分（owner 承認 2026-09-27「介入19・75分・relay 6で承認」。既定は 30 分。L3 round 1 は build 4 本〈main / PR head / 試し / muted の M1〉を並べて 15 項目を見る。今回の数え直しで変えない）
- relay 往復上限: 7（owner 承認 2026-09-27（再改定、この change での介入 9 回目）。以前の承認済みの値は 6〈同上〉。内訳 = Plan Review の Codex 3〈round 1・round 2・round 3 とも消費済み〉+ Final Review の Codex 1 + Final Review の指摘後の Codex closure 1〈予備〉+ manual〈L3 の結果の受け渡し〉2〈round 1・round 2〉。2026-09-28 の数え直し〈裁定 r3 の B3-6〉で 消費 3 / 見込み 3 / 予備 1 = 7、承認済みの 7 の内。L3 round 1 の受け渡しで 消費 4 / 見込み 2 / 予備 1 = 7）。closeout 時の消費 6 / 上限 7 = Plan Review の Codex 3、L3 round 1 の受け渡し 1、Final Review の Codex 1、L3 round 2 の受け渡し 1。closure は Claude 側で 2 本行い、Codex closure の予備は使わなかった

介入の判断点（decision point 単位、2026-09-28 の裁定 r3 による数え直し）:

| 判断点 | 回数 | 状態 |
|---|---|---|
| 起票承認（owner 2026-09-27「B 色と強調 + E 小口」） | 1 | 消費済み |
| owner の色の判断: card の面色を別 lane に分ける | 1 | 消費済み |
| owner の色の判断: muted の文字色を濃くする | 1 | 消費済み |
| owner の色の判断: 返品・交換の選択カードは操作の色へ追従 | 1 | 消費済み |
| owner の色の判断: 入口 card の試しの地は #E6F0F0 | 1 | 消費済み |
| Plan Review round 1 の Codex の起動 | 1 | 消費済み |
| 予算の改定の承認（裁定 r1 の B1-8、19・75 分・6） | 1 | 消費済み |
| Plan Review round 2 の Codex の起動 | 1 | 消費済み |
| 予算の再改定の承認（裁定 r2 の B2-4、24・75 分・7） | 1 | 消費済み |
| Plan Review round 3 の Codex の起動（天井） | 1 | 消費済み |
| round 3 後の一括是正の plan-approved の承認（介入の上限 25 の承認を同じ 1 回で求める。裁定 r3 の B3-6） | 1 | 消費済み |
| L3 round 1 の試しの採否（D-CE7 / D-CE8 / D-CE12 / D-CE13 / D-CE16、日報の取込み済みの badge、Home の前日分の知らせ、最新と上書き件数） | 8 | 消費済み |
| L3 round 1 の試し以外の合否（AC-L3-1 / 7 / 8 / 11〜13 / 15） | 1 | 消費済み |
| L3 round 1 の 2（ランキング 1 位）の新方向の扱い（owner 決定 2026-09-28、上限 26 の承認を同じ 1 回で得た） | 1 | 消費済み |
| Final Review の Codex の起動 | 1 | 見込み |
| Final Review の指摘後の Codex closure の起動 | 1 | 予備 |
| L3 round 2（採否を反映した最終版）の合否 | 1 | 見込み |
| Ready | 1 | 見込み |
| merge | 1 | 見込み |
| 合計 | 26 | 消費 21 / 見込み 4 / 予備 1 |

Plan Review round 2（Codex）の試算は 22 回（round 1 の表 19 に色の判断の分割 +3）。裁定 r2 の表はそれに Codex closure の予備 1 と、再改定の承認 1 を足した 24。裁定 r3 の表は round 3 後の一括是正の plan-approved の承認 1 を足した 25（round 3 は上限のため、以後の Plan Review の起動は無い）。L3 round 1 の後、2 の新方向の扱い 1 を足して 26（owner 承認 2026-09-28、この change での介入 21 回目）。
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
operator が状態を読み分ける見た目（押す・進行中・危険・失敗・注意の色と強調）を全画面で変える。共有部品（`alert.tsx` の `destructive` は 53 site、`badge.tsx`、`progress.tsx`、`--primary` / `--ring` を読む `button`・`checkbox`・ナビの現在地・focus ring）への変更が画面横断に波及し、DEV_WORKFLOW Risk Tiers の「operator workflow」と、R2 / R3 で迷えば stable contract（DSR の見え方の契約）に触れる方を採る規則に当たる。DB / Tauri command / DTO / route / search state / CSV は変えないため R4 ではない。

## Goal

Goal Invariant:

### 最小完了条件

- 操作（押すボタン・リンク・ナビの現在地・checkbox の checked・focus ring）が D-091 の操作の色（#1D5C63）になり、注意・確認の琥珀と別の色で見える。
- 進行中（現在行・待ちの spinner・作業の進み具合の棒、owner が採った試しでは詳細を開いた行と取込みの手順のいまのステップ）が新しい進行中の token で示される。
- 危険・失敗の Alert が薄い地 + 線 + 三角 icon + 文言（段 2）になり、53 site すべてで icon が付く。
- 00 の「迷いやすい場面」23 行と色の役割表の移行列に「runtime lane A 待ち」「lane A の L3 で試し」が残らず、試し 8 点は owner の L3 の採否どおりの形（実装と 00 / 01 / 02 / 04 / review-checklist）にそろう。
- 説明文の muted の文字（`--muted-foreground`）が `--card`・`--background`・進行中の地のどれの上でも AA（4.5:1）以上になり、値は owner が L3 で候補 3 つから選んだもの（D-CE16）。
- 進行中の地（#E6F0F0）の上に載るすべての文字が AA（4.5:1）以上、icon は 3:1 以上になる（Contract Probe の「進行中の地の上の文字色」の表。在庫少のセルは開いた行では `text-warning-strong`、D-CE7）。
- 枠の無い操作部品（Button の塗り・ghost・link・destructive、Accordion の開閉、ScrollArea）と、focus の前後で枠の色が変わらない部品（checked の Checkbox、返品・交換の選択カード、押された Toggle〈在庫照会の在庫状態の chip〉）の focus 表示が不透明な ring になり、合成後の色で対 background・対 card 3:1 以上になる（D-CE17、DSR-22）。
- 旧 token（`--row-current`、試しを採った場合の `--rank-top-*`、`--warning-emphasis` と操作の兼用）が src と 00 から消え、DS3（00 ↔ `globals.css` の HEX 突合）が通る。

### 失敗定義

- 今の赤・琥珀・緑の割り当て（在庫切れ・取消済み・マイナスの増減・取得失敗の赤、注意の琥珀、完了の緑、owner 決定 2026-09-06）が変わる。
- 状態の意味が色だけで伝わる箇所が増える（文言・icon・記号・位置のどれかを失う）。accessible name・role・文言が変わる。
- main に途中の状態（操作の色だけ新しく、部品や画面が旧 token のまま等）が置かれる。
- `src-tauri/**`、`src/lib/bindings.ts`、`src/routeTree.gen.ts`、`package.json` / `package-lock.json` に diff が出る。
- 正本に無い色の値・token の役割を実装が先に決める（例: card の面色を変える、muted の文字色を D-CE16 の候補以外の値にする）。

### 非目的

- 書体の変更（runtime lane B）。棚卸し画面 D1 の実装と作業中の囲み（段 3）の新設。
- card の面色を白へ寄せること（owner 決定 2026-09-27: 別 lane）。
- ボタンの高さ（44px）・badge の文字の太さの変更（badge の太さは AC-L3-3 で見るだけ）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

operator の操作手順・data 契約・状態遷移は変えないが、operator が普通の一日で状態を読み分けられるかが目的のため、表を置く。各行は L3 の到達手順にも使う。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 最後の商品別 CSV（Z004）取込みの精算日が前日より前（Z004 の取込み履歴を持つ L3 の DB〈合成、AC-L3 の前提〉。本番 DB は Z004 の取込みが 0 件のため知らせが出ない。知らせの条件は `useHomeSummary.ts` の `lastImportSettlementDate < yesterday`） | Home を開く | 前日分の未取込みの知らせ（危険・失敗の薄い地 + 線 + 三角 icon + 文言）と、「売上データ取込み」の入口 card（操作の線）が目に入り、押す先が 1 つに決まる | 入口 card を押す。日報を取り込んでも Z004 の取込み履歴は変わらず、Z004 の確定の停止中は知らせは消えない | 知らせを注意・確認へ揃える試し、入口 card の地を操作の仲間へ揃える試しは L3（AC-L3-4 / 6、見た目の比較だけ。fixture は Coordinator が用意） |
| 売上データ取込みの日報取込みタブ、ファイル未選択 | 日報ファイルを選び、プレビューを見て取り込む | 取込み中は待ちの spinner が進行中の色で示される。取込み済みの日なら危険・失敗の Alert で止まる | 結果の画面が出る | 取込み済みの badge を危険・失敗へ揃える試しは L3（AC-L3-5）。spinner は一瞬のため色は自動 test（AC5、S16）で確かめる |
| 日報取込みの結果の画面 | 結果を読む | 完了の badge（緑、icon + 文言）と、押すボタン（操作の塗り）を言い分けられる | 次の画面へ移る | AC-L3-7 |
| 商品別CSV取込み（Z004）タブ | ファイルを選び、プレビューを見る | 手順の表示のいまのステップが分かる。取込みの確定は停止中（[停止 ADR](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md) SPEC-STOP-D4）のため、「3 結果」と取込み中の表示（移動制限の知らせ）は画面では出ない | —（停止の解除は ㉘ の ⑤） | いまのステップの試しは L3（AC-L3-10、ステップ 1〜2）。ステップ 3 と移動制限の知らせは自動 test だけで確かめる |
| 在庫照会の一覧 | 行を押して詳細を開き、もう一度押して閉じる | 開いた行と詳細が一体に見え、どれを開いているか分かる | 別の行を開く・閉じる | 詳細を開いた行を進行中にする試しは L3（AC-L3-9）。行内の muted の文字は現行の値では進行中の地の上で 4.13:1、D-CE16 の 3 候補で 4.66〜6.57:1（Contract Probe） |
| 入庫記録の入力 | 取引先を選ぶ dialog で行を選ぶ | 選んでいる行が左端のバー + 進行中の地 + 「選択中」で分かる | 閉じて入力を続ける | AC-L3-8 |
| 月次売上の商品別 | ランキングと前月比を見る | 1 位が琥珀の pill「1 位」と行の地で分かる（L3 round 1 で試し〈順位と太字〉を採らなかった側、owner 2026-09-28）。前月比は記号と文字色で増減が分かる | — | AC-L3-2 / 11 |
| どの画面でも取得に失敗 | 画面を開く | 危険・失敗の Alert（薄い地 + 線 + 三角 icon + 原因と次の一手）と再試行 | 再試行を押す | AC-L3-12 |

本 lane で完了できるのは見た目の移行までで、operator の業務の目的（取込み・照会・記録）は現行のまま達成できる。muted の文字の AA 未達は本 lane で閉じる（D-CE16）。card の面色は別 lane に残る。

## 起票時実測（2026-09-27、`e7c22f8f`）

site 数は下の command をこの worktree で実行した出力。「非 test」は `--glob '!*.test.*'`。分類の全行は [Matrix](test-matrices/2026-09-27-design-color-emphasis.md) の Adjacent Pattern Audit。

| # | 対象 | command | 出力 | 扱い |
|---|---|---|---|---|
| 1 | `primary` の非 test 行 | `rg -n 'primary' src --glob '!*.test.*' \| wc -l` | 35 | token の値の変更で追従する site と、進行中へ移す site に分ける（Matrix） |
| 2 | `--ring` を読む site | `rg -n -e '-ring\b' src --glob '!*.test.*' \| wc -l` | 19（16 file） | すべて `ring-ring` / `border-ring` / `outline-ring` / token 定義で、`--ring` の値の変更で追従。site の編集なし |
| 3 | `warning-emphasis` | `rg -n 'warning-emphasis' src --glob '!*.test.*' \| wc -l` | 5 | 注意・確認の文字（在庫少のセル・件数・取込みの警告文）は不変。token 定義 2 行は値不変 |
| 4 | `rank-top` | `rg -n 'rank-top' src \| wc -l` | 8（test 0） | ランキング 1 位の試しを採れば全削除 |
| 5 | `row-current` | `rg -n 'row-current' src \| wc -l` | 9（非 test 3・test 6） | 現在行を進行中の token へ移し全削除 |
| 6 | `toast.info` | `rg -n 'toast\.info' src \| wc -l` | 0 | 変更なし（00「お知らせ一般」の行どおり） |
| 7 | `variant="destructive"` | `rg -n 'variant="destructive"' src --glob '!*.test.*' \| wc -l` | 63 = Alert 53 + `AlertDialogAction` 9 + `Button` 1（`MergeSupplierDialog.tsx:193`、dialog 内） | Alert 53 は `alert.tsx` の変更で移る（うち `ImportingStep` 1 は注意・確認へ）。dialog の実行ボタン 10 は不変（DSR-20） |
| 8 | 明示の icon を持つ destructive Alert | `rg -U -c '<Alert variant="destructive"[^>]*>\s*<AlertTriangle' src --glob '!*.test.*'` の合計 | 6（`BackupRestorePage.tsx:324,364`、`CostDiffDialog.tsx:141`、`HomePage.tsx:78`、`IntegrityCheckPage.tsx:211`、`ProductImportPreview.tsx:83`） | icon を部品が描くため 6 site の明示の icon を外す（D-CE2） |
| 9 | `data-[state=selected]` / `has-aria-expanded` | `rg -n 'has-aria-expanded\|data-\[state=selected\]' src --glob '!*.test.*'` | `table.tsx:48` の既定 1 行 + `ProductListTable.tsx:96` のコメント 1 行 | `table.tsx` の既定は stone のまま（選択欄を開いた入力行に当たるため）。詳細を開いた行は site 側で明示の class を当てる（D-CE7） |
| 10 | `<Badge variant="default"` | `rg -n '<Badge variant="default"' src --glob '!*.test.*' \| wc -l` | 2（`BackupRestorePage.tsx:520` 最新、`ProductImportPreview.tsx:76` 上書き件数） | `--warning-emphasis` の塗りへ移す（D-091）。variant 無指定の `<Badge>` は `ProductRankingTable.tsx:80` だけ |
| 11 | 待ちの spinner | `rg -n 'animate-spin text-primary' src \| wc -l` | 5（`DailyReportImportPage.tsx:98,298`、`ImportingStep.tsx:22`、`ParseStep.tsx:25`、`IntegrityCheckPage.tsx:490`） | 進行中へ |
| 12 | 進み具合の棒 | `rg -n '<Progress' src --glob '!*.test.*'` | 6 行 = runtime 3 site + `progress.tsx` の定義・コメント 3（`:3` コメント、`:17` Root、`:23` Indicator）。runtime 3 site（`StocktakePage.tsx:430` 棚卸しの進み、`DepartmentTable.tsx:95` 部門比率、`IntegrityCheckPage.tsx:495` 確認中の不定の棒〈`before:bg-warning`〉） | 作業の進み 2 site は進行中、比率 1 site はふつう・補足（00「進み具合の棒」の行） |
| 13 | 00 に無い token の drift | `rg -n 'warning-foreground\|info-soft\|border-info\|text-info-strong' src \| wc -l` | 2（`IntegrityCheckPage.tsx:272`、`PluExportPage.tsx:366`） | どちらも class が生成されず見た目に効いていない。class を外す（D-CE11、見た目不変） |
| 14 | 未保存の案内の赤い文字 | `rg -B1 '未保存の(入庫\|手動販売\|廃棄・破損\|返品・交換)内容があります' src --glob '!*.test.*' \| rg -c 'text-destructive'` | 4 | 注意・確認の文字へ（D-CE10） |
| 15 | docs の移行の印 | `rg -n 'lane A (待ち\|の L3 で試し\|の L3 まで\|の merge 前\|の merge 後\|の L3 で決ま)\|runtime lane A で(追加\|移す)' docs/design-system docs/quality/review-checklist.md docs/UI_TECH_STACK.md docs/SCREEN_DESIGN.md --glob '!reference/**' \| wc -l` | 36（00 26 / README 4 / review-checklist 2 / 01・02・04・UI_TECH_STACK 各 1） | すべて解消 |
| 16 | muted の文字色の読み手 | `rg -n 'muted-foreground' src --glob '!*.test.*' \| wc -l`（file 数は `rg -l` で 75） | 276 行（`text-muted-foreground` 265、`:text-muted-foreground` 4、`placeholder:` 2、`border-muted-foreground/30` 2、`text-muted-foreground/40` 1、`hover:` 1、`dark:` 1 と token 定義 2 は `--muted-foreground` と `--color-muted-foreground`。内訳は `rg -o --no-filename '[a-z:-]*muted-foreground[/0-9]*' src --glob '!*.test.*' \| sort \| uniq -c`）。test 27 行 | すべて `globals.css` の値の変更で追従（S19、site の編集なし）。同じ役割で token を読まない `EmptyState.tsx:32` の `text-stone-500` は除外（Non-scope） |
| 17 | ネイティブの radio・checkbox（2026-09-28 追記、裁定 r3 B3-9） | `rg -n 'type="(radio\|checkbox)"' src --glob '!*.test.*'` | 3（`ReturnExchangePage.tsx:567,590` の radio、`StockUnitField.tsx:79` の checkbox）。`accent-color` の指定が無く、checked はブラウザ既定の色 | `accent-primary` を足し、00「checkbox・radio の checked = 操作」へ揃える（S21） |

起票時に見つけた、backlog の項目に無い隣接 site（Matrix で移す・除外を決めた）: `ReturnExchangePage.tsx:151` の登録方法の選択カード（`border-primary bg-primary/5`）、`FilePicker.tsx:134` の drag over（同）、`input.tsx` の文字選択（`selection:bg-primary`）、`formatErrorRow.ts:22` の「フォーマット異常」badge（`destructive` の塗り）、`PriceRevisionTable.tsx:104` の「入力中」（現在行を badge 1 点で示す）。

## 設計判断（Coordinator adjudication、Plan Review で覆せる）

- **D-CE1 token**: 進行中の家族を既存の家族（warning / success / destructive）と同じ形で足す。`--ongoing`（#2F7F86、D-091 の進行中の枠。バー・棒・spinner・段 3 の枠）、`--ongoing-soft`（#E6F0F0、地）、`--ongoing-border`（#7FB0B4、badge 大の要素の 1px 線）、`--ongoing-strong`（#123E43、文字）。`@theme inline` に `--color-ongoing*` を足す。`--primary` と `--ring` を #1D5C63（D-091 の操作の塗り）へ、`--primary-foreground` は #fafaf9 のまま。`--warning-emphasis` の値は不変（#b45309）で、役割は注意・確認だけになる。名前は Plan Review で覆せる。値は D-091 の候補値で、L3 で確定したら 00 のパレット表・セマンティックカラー表へ登録し、D-094 に記録する（D-091「runtime lane A が実測と L3 で値を確定し…登録する」）。
- **D-CE2 destructive Alert**: `alert.tsx` の `destructive` を `warning` と対称の 4 点にする: `bg-destructive-soft border-destructive text-destructive-strong [&>svg]:text-destructive *:data-[slot=alert-description]:text-destructive-strong/90`。三角 icon（lucide の `TriangleAlert`、`aria-hidden`）は `variant="destructive"` のとき部品が最初の子として描き、明示の icon を持つ 6 site（実測 #8）から外す。backlog の「destructive Alert の soft 塗り + 三角 icon」を本 lane に束ねる（00「危険・失敗の Alert」の行が「束ねるかを lane A の Plan で決め」とする。狙いの形は 00 が段 2 と決めており、別 PR にすると Alert だけ規則とずれたまま main に残る）。部品が描く理由 = DSR-08 の「非中立の Alert は icon 必須」を 53 site の個別編集でなく 1 か所で守り、今後の site の書き忘れを構造で防ぐ。`warning` は現行どおり site が icon を書く（変えない）。
- **D-CE3 badge**: `badge.tsx` の `default` variant（`bg-primary` の塗り）と `defaultVariants` を削る。最新・上書き件数（実測 #10）は `className="border-warning bg-warning-emphasis text-primary-foreground"`（D-091 の移し先、対比 4.81:1）で琥珀 pill を保つ。`link` variant の `text-primary` は `--primary` の値で追従する。
- **D-CE4 進み具合の棒**: `progress.tsx` の棒を `bg-ongoing` にし、`indicatorClassName?: string` を 1 つ足す（棒の class を `cn` で上書き）。部門比率（`DepartmentTable.tsx:95`）は `indicatorClassName="bg-muted-foreground"`（ふつう・補足、stone の既存 token）。確認中の不定の棒（`IntegrityCheckPage.tsx:497`）は `before:bg-ongoing`。
- **D-CE5 待ちの spinner**: 実測 #11 の 5 site を `text-ongoing` へ。ボタン内の保存中の spinner は元の役割のまま（00「保存中のボタン」の行、変更なし）。
- **D-CE6 現在行**: `SupplierPickerDialog.tsx:137` を `border-l-4 border-l-ongoing bg-ongoing-soft` へ。バーは `--ongoing`（D-091: 進行中の線 #7FB0B4 は 2.29:1 で現在行のバーに使えない）。
- **D-CE7 詳細を開いた行（試し、owner 了承 2026-09-25）**: 在庫照会の選択行と展開行、操作ログの開いた行と詳細の行を、左端のバー（`border-l-4 border-l-ongoing`）+ 進行中の地（`bg-ongoing-soft`）で一体に見せる。閉じた行は `border-l-4 border-l-transparent`（`SupplierPickerDialog` と同じく列がずれない）。`table.tsx` の既定（`data-[state=selected]:bg-muted`・`has-aria-expanded:bg-muted/50`）は stone のまま残す（`has-aria-expanded` は選択欄を開いた入力行にも当たるため進行中にしない）。site 側の上書きは既定と同じ variant で書く（在庫照会の選択行は `data-[state=selected]:bg-ongoing-soft`、操作ログの開いた行は `has-aria-expanded:bg-ongoing-soft`。既定の variant 付き class は素の `bg-*` より詳細度が高く、素の class では上書きできない。`cn` の tailwind-merge が同じ variant の class を後勝ちで 1 つにする: Contract Probe）。hover で色が動かないよう、開いた行と詳細の行（展開行）の両方に `hover:bg-ongoing-soft` を足す（詳細の行は `data-state` も `has-aria-expanded` も持たず、付け忘れると hover で詳細の行だけ stone になる）。3 点目は開いた詳細と「詳細を閉じる」の類の文言が担い、badge を足さない。開いた行の在庫少のセルは `text-warning-emphasis`（#b45309、進行中の地の上で 4.33:1、AA 未達）から `text-warning-strong`（#78350f、7.81:1）へ替える（同じ注意・確認の役割の家族。閉じた行は `text-warning-emphasis` のまま〈対 `--background` 4.81:1〉）。在庫切れのセル `text-destructive`（5.57:1）と本文（15.06:1）はそのまま。`table.tsx:27` の `TableBody` の `[&_tr:last-child]:border-0` を `[&_tr:last-child]:border-b-0` に限る（最後のレコードを開くと詳細の行が `tbody` の最後の行になり、`border-0` が左のバーを消す。`TableRow` は `border-b` 以外の枠を持たないため、既存の 30 file の `TableBody` の見た目は変わらない）。
- **D-CE8 取込みの手順の表示（試し、Coordinator の既定 2026-09-25）**: 00「取込みの手順の表示」の行どおり、いまのステップの番号を `border-ongoing-border bg-ongoing-soft text-ongoing-strong` の丸 + 太字、名前を `font-semibold text-foreground`。済んだステップと先のステップは同じ段 0（`border-muted-foreground/30 text-muted-foreground`）で、位置と `aria-current="step"` で分ける。結果の画面でもいまのステップは「結果」のまま進行中にする（00 の行は結果の画面を例外にしていない）。採らなかったときは現行の class のまま（いまのステップ = `border-primary bg-primary text-primary-foreground` の塗り、済み = `border-primary bg-primary/10 text-primary`。`--primary` の値で操作の色になる）の 1 通りに固定し、押すボタンでない塗り（段 4）の例外を D-094 に記録する。Z004 の確定が停止中（SPEC-STOP-D4）のため画面ではステップ 1〜2 しか出ず、ステップ 3 は自動 test で固定し、結果の画面での受け取られ方は停止の解除（㉘ の ⑤）の lane の L3 で見る。
- **D-CE9 前月比のセル**: 00「役割色の文字だけの表示」の行（増減の ±）を当てる。薄い地の chip をやめ、+1.0% 以上は `text-success-strong`、−1.0% 以下は `text-destructive-strong`、それ以外と「—」は `text-muted-foreground`。記号と % の文言は不変。chip をやめるのにあわせ、文字を `text-xs` から `text-sm`（14px）へ（00 のタイポグラフィ: badge 以外の 12px は触る lane が 14px へ寄せる）。段 1（薄い地だけ）は 00 が「その役割の領域だけ」に限るため、セルの地に使わない。`docs/function-design/57-ui-monthly-sales.md` の該当行と `docs/SCREEN_DESIGN.md:373` を同期する。
- **D-CE10 失敗ではない赤の知らせ**: 取込み中の移動制限（`ImportingStep.tsx:29`）は失敗でも戻せない操作でもなく、待ってほしい知らせのため注意・確認（`variant="warning"` + `AlertTriangle`、catalog ⑥ の warning の形）。Z004 の確定が停止中のため画面には出ず、自動 test で固定する。入力 4 画面の未保存の案内（実測 #14）は「商品登録へ進むと入力が残らない」ことを確かめてほしい文のため、注意・確認の文字だけの表示 `text-warning-emphasis`（在庫少のセルと同じ token。文言が意味を担う）。未保存の案内は AC-L3-13 で owner が見る。
- **D-CE11 drift の掃除**: `IntegrityCheckPage.tsx:272` の `text-warning-foreground` を外す（`warning` variant が icon を `text-warning` にする）。`PluExportPage.tsx:366` の `border-info bg-info-soft text-info-strong` を外す（class が生成されず、現状も既定の Alert の見た目。役割はふつう・補足のお知らせ一般のまま、見た目不変）。
- **D-CE12 Home の入口 card**: PR head は owner の現行の決定を保つ形（`border-primary bg-warning-soft`、icon `text-primary`。`--primary` の値で線と icon が操作の新しい色になる。site の編集なし）。試しは地を `bg-ongoing-soft`（#E6F0F0、進行中の地を操作の仲間の薄い地として流用。owner 決定 2026-09-27）。
- **D-CE13 ランキング 1 位（試し、owner 了承 2026-09-24）**: PR head は試しの答え（00 の役割と段の列）。1 位は badge と行の地をやめ、`<span className="text-sm font-semibold text-foreground">1 位</span>`（2 位以下の `text-sm text-muted-foreground` は不変）。`--rank-top-*` 3 token と `@theme` の 3 行を削る。
- **D-CE14 試しの扱いと順序**: 試し 8 点（D-CE7 / D-CE8 / D-CE12 / D-CE13 / D-CE16 と、日報の取込み済みの badge・Home の前日分の未取込み・最新と上書き件数）は、採る・採らないの両方の到達形を下の「L3 の分岐」に書いて本 packet で先に承認を受ける。L3 の採否は Scope 内の分岐の選択で、packet の契約を変えないため Gated Amendment にしない。分岐表に書いた 2 通り（muted は 3 候補）以外の形を owner が求めたときは Gated Amendment の対象にする。before（main）は見比べるための基準で、`--primary` の値を変えた後は同じ見た目を作れないため選ぶ対象にしない（選ぶのは after か試し）。順序は 実装 → Writer の検証 → L3 round 1（採否）→ 採否の反映 → Final Review broad → L3 round 2（最終版の確認）→ Ready → merge。broad の後に見た目を変えると broad の取り直しになるため、L3 round 1 を broad の前に置く。
- **D-CE15 隣接 site**: `ReturnExchangePage.tsx:151` の選択カードと `FilePicker.tsx:134` の drag over は `--primary` の値で操作の色へ追従させ、site は編集しない（選択カードは owner 決定 2026-09-27: 操作の色へ追従のまま。見え方は AC-L3-1 で見る）。`input.tsx` の文字選択は操作として追従。`formatErrorRow.ts:22`（危険の塗りの badge、00 は危険の塗りを dialog の実行ボタンだけに限る）と `PriceRevisionTable.tsx:104`（現在行を badge 1 点で示す）は、役割の選択（②分類か①状態か、現在行の 3 点へ足すか）が正本に無いため除外し、Writer が backlog へ 1 行ずつ起票する。
- **D-CE16 muted の文字色（試し、owner 決定 2026-09-27「`--muted-foreground` を濃くする」）**: 値は正本で決まっていないため、ふつう・補足（stone、D-091 の色の役割）の中で次の 3 候補を L3 で並べ、owner がどれか 1 つを選ぶ（採らない選択肢は無い）。比は WCAG 2.x の相対輝度で、本 worktree の `$TMPDIR/cr.py`（`L = 0.2126R + 0.7152G + 0.0722B`、sRGB の線形化、比 = `(L1 + 0.05) / (L2 + 0.05)`）により計算した（2026-09-27）。

  | 候補 | HEX | 対 `--card` #f5f5f4 | 対 `--background` #fafaf9 | 対 進行中の地 #E6F0F0 | 対 `--list-head` #e7e5e4（参考） | 対 `--foreground` #1c1917（本文との差、参考） |
  |---|---|---|---|---|---|---|
  | 現行 | #78716c（stone-500） | 4.40:1 | 4.59:1 | 4.13:1 | 3.82:1 | 3.65:1 |
  | M1（最小の濃さ） | #6f6964 | 4.96:1 | 5.18:1 | 4.66:1 | 4.31:1 | 3.23:1 |
  | M2（一覧の見出しの帯でも AA） | #6b6560 | 5.27:1 | 5.50:1 | 4.95:1 | 4.58:1 | 3.04:1 |
  | M3（Tailwind の stone-600） | #57534e | 6.99:1 | 7.30:1 | 6.57:1 | 6.08:1 | 2.29:1 |

  3 候補とも必須の 3 つの地（card・background・進行中の地）で 4.5:1 以上。M1 は一覧の見出しの帯（`--list-head`）の上では 4.5:1 に届かない。濃くするほど本文（`--foreground`）との差が縮み、ラベルと値の濃さの差（00「ラベルと値」）が弱まる。PR head は M2 で作り、L3 round 1 で M1・M3 と並べる。token の名前と class（`text-muted-foreground` 265 行ほか）は変えず、`globals.css` の値だけを変える。
- **D-CE17 枠の無い部品の focus 表示（DSR-22 の操作枠 3:1、裁定 r1 B1-4）**: focus ring の `focus-visible:ring-ring/50`（3px、50% の透過）は、合成後の色で見ると新しい `--ring` でも対 background 2.35:1・対 card 2.33:1 で 3:1 に届かない（現行の琥珀でも 2.08:1・2.05:1）。枠を持つ部品は focus で 1px の枠が不透明な `border-ring`（7.28:1 / 6.97:1）になるため足りるが、枠の無い部品は透過の ring だけが目印になる。対象を次の部品に限り、`focus-visible:ring-ring/50` を不透明な `focus-visible:ring-ring` にそろえる: `src/components/ui/button.tsx`（base。`outline` 以外の variant は枠が無い。`destructive` の `focus-visible:ring-destructive/20`〈合成後 1.41:1〉も外し、focus は操作の色にそろえる）、`src/components/ui/accordion.tsx`（開閉の trigger）、`src/components/ui/scroll-area.tsx`（viewport）、`src/components/ui/checkbox.tsx`（checked のとき枠と地が `--primary` で `--ring` と同じ値になり、focus で枠の色が変わらないため、目印は透過の ring だけになる。裁定 r2 B2-3）、`src/features/return-exchange/ReturnExchangePage.tsx:147` の登録方法の選択カードの `focus-within:ring-ring/50`（「レジ戻し済み」を選ぶと枠が `border-primary` で checkbox と同じ形。同じ扱いにそろえる 1 通りに決めた）、`src/components/ui/toggle.tsx:8`（在庫照会の在庫状態の chip。`StatusChips.tsx` は `variant="outline"` で枠を持つが、押された chip は `src/components/ui/selection-tone.ts:9` の `data-[state=on]:border-stone-400` が `focus-visible:border-ring` より後に出て〈Tailwind v4 の出力順、詳細度同じ。Plan Review r3〈Opus〉が compile で実測〉focus で枠が変わらず、目印は透過の ring だけになる。裁定 r3 B3-3）。site 数（非 test、`rg -U -c '<Button\b[^>]*variant="(ghost|link|outline|secondary|destructive|default)"' ` の variant ごとの合計と `rg -c '<Button\b'` の差から）: `<Button` 187 = 無指定（塗り）41・`ghost` 9・`link` 5・`outline` 126・`destructive` 1・`default` 2・`secondary` 0・動的な `variant={…}` 3（`PluExportPage.tsx:729`、`alert-dialog.tsx:138,156`。`rg -U -c '<Button\b[^>]*variant=\{' src --glob '!*.test.*'` の合計）。ほかに `AlertDialogAction` / `AlertDialogCancel` が `Button` を使う。`<AccordionTrigger` 1（`ErrorRowsTable.tsx:43`）、`<ScrollArea` 2（`Sidebar.tsx:15`、`ShortcutsDialog.tsx:39`）、`<Checkbox` 9（`rg -c '<Checkbox\b' src --glob '!*.test.*'` の合計。7 file）、選択カード 2（`ReturnExchangePage.tsx:565,588`）、`<ToggleGroupItem` 1（`StatusChips.tsx:34`、`rg -c '<ToggleGroupItem\b' src --glob '!*.test.*'`。`<Toggle` の直接の使用は 0）。checkbox の focus の比（`$TMPDIR/blend.py` と `$TMPDIR/cr.py`、2026-09-27）: checked の枠と地 #1D5C63 は focus の前後で変わらない（1.00:1）。透過の ring は対 background 2.35:1・対 card 2.33:1（現行の琥珀 2.08:1。Plan Review r2〈Codex〉の Chrome 実測は 2.354:1・2.321:1）、不透明にすると 7.28:1・6.97:1。未 checked は focus で 1px の枠が `border-ring`（7.28:1、focus 前の `--border-strong` は 3.53:1）。不透明な ring は checked の地と同じ色でつながり、3px 大きく見える形になる。`dialog.tsx` の閉じるボタンと `DateNavigator` / `MonthNavigator` は既に不透明な ring、`segmented-control.tsx` は focus で `border-border-strong`（3.53:1）。塗りのボタンでは不透明な ring が塗りと同じ色でつながり、3px 太く見える形になる（AC-L3-1 で見る）。

## Scope

1 PR。commit は backlog の進め方どおり、(1) 新 token を足す → (2) 部品を移す → (3) 画面を移す → (4) `--primary` / `--ring` の値を変え旧 token を削る → (5) docs、の順に分けてよい（途中の状態は branch 内だけ）。

- **S1 `src/styles/globals.css`**: D-CE1 の 4 token と `@theme inline` の 4 行を足す。`--primary` / `--ring` を #1D5C63 へ（commit (4)）。`--row-current` と `--color-row-current` を削る。ランキングの試しを採れば `--rank-top-*` 3 行と `@theme` の 3 行を削る（L3 の分岐）。
- **S2 `src/components/ui/alert.tsx`**: D-CE2。`destructive` の class と、`variant === "destructive"` のとき最初の子に `TriangleAlert` を描く。
- **S3 destructive Alert の明示の icon を外す 6 site**: 実測 #8 の 6 site から `AlertTriangle` の子を外す（使わなくなった import も外す）。
- **S4 `src/components/ui/badge.tsx`**: D-CE3。`default` variant と `defaultVariants` を削る。
- **S5 ③強調の 2 site**: `BackupRestorePage.tsx:520`、`ProductImportPreview.tsx:76` を D-CE3 の class へ（L3 の分岐で試しを採れば stone の pill と太字へ）。
- **S6 `src/components/ui/progress.tsx`**: D-CE4。棒を `bg-ongoing`、`indicatorClassName?` を足す。`DepartmentTable.tsx:95` に `indicatorClassName="bg-muted-foreground"`、`IntegrityCheckPage.tsx:497` を `before:bg-ongoing`。
- **S7 待ちの spinner 5 site**: D-CE5。
- **S8 現在行**: `SupplierPickerDialog.tsx:137` を D-CE6。
- **S9 詳細を開いた行**: D-CE7。`src/features/stock-inquiry/components/ProductListTable.tsx`（選択行と展開行、開いた行の在庫少のセルを `text-warning-strong`）、`src/features/operation-logs/OperationLogsPage.tsx`（開いた行と詳細の行）、`src/components/ui/table.tsx:27`（`TableBody` の末尾の規則を `border-b-0` に限る）。L3 で採らなければ現行（stone、在庫少のセルは `text-warning-emphasis`）へ戻す。`table.tsx:27` の変更は採否に関わらず入れる（stone の形では左のバーが無く見た目は変わらない）。
- **S10 取込みの手順の表示**: `src/features/csv-import/components/StepIndicator.tsx` を D-CE8。L3 で採らなければ現行の class のまま（`--primary` の値で操作の色になる）にし、D-094 に例外を記録する（L3 の分岐）。
- **S11 ランキング 1 位**: `src/features/monthly-sales/components/ProductRankingTable.tsx` を D-CE13。
- **S12 前月比のセル**: `src/features/monthly-sales/components/comparison-cell.tsx` を D-CE9。
- **S13 失敗ではない赤の知らせ**: `ImportingStep.tsx`（Alert を `warning` + `AlertTriangle`）、入力 4 画面の未保存の案内（`ReceivingPage.tsx`・`ManualSalePage.tsx`・`DisposalPage.tsx`・`ReturnExchangePage.tsx` の `text-destructive` を `text-warning-emphasis`）。
- **S14 drift**: D-CE11 の 2 site。
- **S15 L3 の分岐の反映**: owner の L3 round 1 の答えに従い、下の「L3 の分岐」表の採る / 採らない側の edit を行う（Home の入口 card は `src/features/home/components/ActionButton.tsx`、日報の badge は `DailyReportImportPage.tsx:163-166`、Home の前日分の知らせは `HomePage.tsx:78`、ほか S5 / S9 / S10 / S11）。
- **S16 test**（既存 assert の更新は意図を保つ。class だけでなく文言・role・icon を assert する）:
  - `src/components/ui/alert.test.tsx`: destructive が soft の 4 点の class と `svg` 1 つ（部品が描く）を持つ、`warning` と `default` は部品が icon を描かない。既存の「destructive は warning の class を持たない」は保つ。
  - `src/components/ui/badge.test.tsx:64`: `bg-primary` の既定の塗りの assert を、variant 無指定で `bg-primary` を出さない assert へ。`:13-26` の対照 test（outline は `border-border-strong` を持ち、比べる相手は持たない）は、比べる相手を `variant="secondary"` に替えて意図を保つ。`badge.tsx:38` の既定引数 `variant = "default"` も削る。
  - `data-variant="default"` の assert（`BackupRestorePage.test.tsx:581`、`ProductImportPreview.test.tsx:95`、`ProductRankingTable.test.tsx:101`）は、③強調の class（`bg-warning-emphasis` と `border-warning`。ランキングは採った側では試しの答えの `font-semibold`、採らなかった側では rank-top の地・文字・枠。下の `ProductRankingTable.test.tsx` の項目）の assert へ替える。
  - `src/components/ui/alert.test.tsx:34-42` の `it.each(["default", "destructive"])` の `bg-card` の assert は、`default` だけに残し、`destructive` は soft の 4 点の class の assert へ分ける（warning の class を持たない対照は両方に残す）。
  - `src/components/ui/button.test.tsx` に「base と `destructive` が `focus-visible:ring-ring` を持ち、`focus-visible:ring-ring/50` と `focus-visible:ring-destructive/20` を持たない」、accordion・scroll-area・checkbox（checked の状態で render して class を確かめる）と返品・交換の選択カードの class の assert（D-CE17）。`src/features/stock-inquiry/components/StatusChips.test.tsx` に、押された chip（`data-state="on"`）が `focus-visible:ring-ring` を持ち `ring-ring/50` を持たない assert（裁定 r3 B3-3）。描画（checked + focus の外側の ring の比）は jsdom で測れないため Contract Probe と AC-L3-15 で確かめる。
  - `table.tsx` の `TableBody` が `[&_tr:last-child]:border-b-0` を持ち `[&_tr:last-child]:border-0` を持たない class の assert。描画（最後のレコードを開いても左のバー 4px が残る）は jsdom で測れないため Contract Probe と AC-L3-9 で確かめる。
  - `src/features/daily-report-import/DailyReportImportPage.test.tsx` に、解析中（`role="status"`、「日報ファイルを解析中…」）の spinner の `svg` が `text-ongoing` を持ち `text-primary` を持たない assert（D-CE5。取込み中の spinner は合成 fixture では一瞬で L3 で判定できないため、自動 test と AC5 で確かめる。裁定 r3 B3-5）。
  - `src/styles/globals.test.ts:20,28`: `--row-current` の literal を撤去し、`--ongoing*` 4 token と `@theme` の 4 行、`--primary` / `--ring` の値を literal で固定。
  - `src/features/suppliers/components/SupplierPickerDialog.test.tsx:61,65,66`: `bg-row-current` / `border-l-primary` を `bg-ongoing-soft` / `border-l-ongoing` へ（「選択中」の文言の assert は保つ）。
  - `src/features/stock-inquiry/components/ProductListTable.test.tsx`・`src/features/operation-logs/OperationLogsPage.test.tsx`: 開いた行と詳細の行が進行中の class を持ち、閉じると外れる（D-CE7）。選択欄を開いた入力行が進行中にならないことは `table.tsx` の既定が不変であることで保つ。
  - `ProductListTable.test.tsx` に、在庫少の行を選ぶとそのセルが `text-warning-strong`、選択を外すと `text-warning-emphasis` に戻る assert（B1-1）。
  - `src/features/monthly-sales/components/ProductRankingTable.test.tsx`: 採った側は 1 位の「1 位」の文言が `font-semibold` で、行に地の class が無い。採らなかった側でも S4 で `default` variant を削るため `:101` の `data-variant="default"` は必ず red になる。`:101` を琥珀 pill の地・文字・枠（`bg-rank-top-badge-bg`・`text-rank-top-badge-text`・`border-warning`）と、sort 後も 1 位の行に追従する assert に置き換える。
  - StepIndicator の test を足す（`aria-current="step"` のステップだけが進行中の class と太字、済んだ・先のステップは muted）。新しい test file は REQ か UI の ID（例 `UI-07`）を describe に含める（traceability T4 は ID の無い FE test file の数を baseline と比べる）。
  - 前月比のセル・移動制限の Alert・未保存の案内・③強調の badge・Home の入口 card・日報の badge・Home の Alert の既存 test のうち class を assert するものを、新しい形へ（`ActionButton.test.tsx:43,44,80`、`DailyReportImportPage.test.tsx`、`BackupRestorePage.test.tsx`、`IntegrityCheckPage.test.tsx`、`DepartmentTable.test.tsx` ほか。対象は `rg -l 'primary|rank-top|row-current|bg-warning\b|bg-muted|destructive-soft|success-soft' src --glob '*.test.*'` の 17 file から Writer が確かめる。2026-09-27 に同じ command の `| wc -l` で 17）。
- **S17 docs**（L3 の分岐の答えを反映した最終形で書く）:
  - `docs/design-system/00-foundations.md`: 冒頭の「現行と狙い」、色の役割表の「現行の実装」「移行」列、「迷いやすい場面」表の現行の実装・移行列（23 行すべて「済」か、試しを採らなかった行は owner の答えの形）、「強調の段階」の③強調の琥珀 pill の文、カラーパレット表（`--row-current` の行を削り、`--ongoing` 4 行を HEX つきで足す）、セマンティックカラー表（Primary の HEX を #1D5C63、Warning Emphasis の「現行は操作の色と同じ値」を削る、試しを採れば Rank Top 3 行を削る）、「候補の色の値…D-091 に置く」の注記を登録済みへ。「迷いやすい場面」表の前置き（`:63-68` の試しの書き方 2 通りと移行中の例外 2 点）を L3 の答えへ（試しを採らなかった例外は D-094 を指す）。
  - `docs/design-system/00-foundations.md` の迷いやすい場面「役割色の文字だけの表示」の行: 進行中の地の上に載る注意・確認の文字は `-strong`（在庫少のセル、D-CE7）と注記する。
  - `docs/design-system/01-decision-rules.md`: DSR-08（icon を必須にする Alert を「非中立の〈warning・destructive〉Alert」へ、ランキングの文を L3 の答えへ、Why の `:487` の「token は runtime lane A で追加する」の句）、DSR-21 の Why の「runtime lane A までの現状」の文（`:501`）、DSR-22 の現在行の「現行は primary のバーと `--row-current`」の文と、同じ段落の token の列挙の `--row-current`（`:136`）、詳細を開いた行の文。
  - `docs/design-system/02-component-catalog.md`: ① `:61` の Primary（`amber-700`）、⑤ `:322,328` の amber の文、⑥ destructive の段落（`:415` の「Home は destructive のまま `AlertTriangle` icon を追加」、`:419` の「本 packet では変更しない」、`:450` の使用トークン）と「destructive の Alert は部品が三角 icon を描き、site は icon を書かない」の 1 文、⑧ `:591` の取引先 picker の現在行の語彙（`--row-current` + 左 4px primary バー）、⑫ の見本の `:816`（展開行の `bg-muted hover:bg-muted`）・`:827` の使用トークン・`:830-831` の状態（展開行の `bg-muted` と `hover:bg-muted`。D-CE7 の採否の答えの形へ）、⑬ の ③強調の note（`:894`）と 1 位、⑯ `:1015` の `--row-current`、更新履歴 1 行。
  - `docs/design-system/04-backbone.md`: 原則 2 と旧番号の対応表の旧 3 の行の「Alert」を「非中立の Alert」へ、原則 4 のランキングの文を L3 の答えへ（`:26`）。原則 10（`:44`）の「現行は primary のバーと `--row-current` で、runtime lane A で進行中の token へ移す」、旧番号の対応表の旧原則 15 の行（`:71`）の同じ句、DSR-21 の Why の行（`:78`）の「runtime lane A までの現状」、badge の太さの行（`:115`）の「runtime lane A の L3 の before / after で」の句を、lane A 反映後の最終形へ（現在行は進行中の token、太さは AC-L3-3 の owner の答え）。
  - `docs/design-system/README.md`: 目次の表の `:33`、「移行中の読み方」（`:37`）「移行中の作り方」（`:41` ほか）を lane A 反映後の形へ（残る移行は runtime lane B だけ。lane B の移行の文は残す）。既存 docs との責務境界の表の `:66`（「画面ごとの色の記述は runtime lane A が画面と同時に直す」）を反映済みの形へ。
  - `docs/quality/review-checklist.md`: カテゴリ 9 の③強調とランキングの行、「runtime lane A の merge 前は…」の句。
  - `docs/UI_TECH_STACK.md:46`: token の家族に進行中を足し「進行中は runtime lane A で追加」を削る。
  - `docs/UI_TECH_STACK.md` §5.4 の「フォーカスリング明示」（`:469`）の系統①: 枠を持つ部品（input 等）は `focus-visible:ring-ring/50` 系のまま、「枠の無い部品と、focus の前後で枠の色が変わらない部品は不透明な `focus-visible:ring-ring`（DSR-22 の 3:1）」を足す（D-CE17、裁定 r3 B3-2）。
  - `docs/SCREEN_DESIGN.md`: `:110`（入口 card）、`:228`（「手動」は黄色でなく②分類の stone の pill）、`:368`（1 位）、`:373`（前月比）。
  - `docs/function-design/56-ui-daily-sales.md:326`: 「黄色「手動」バッジ」を②分類の stone の pill（`variant="secondary"`）へ（`docs/SCREEN_DESIGN.md:228` と同じ drift、実装は `daily-sales/components/ProductTable.tsx:132`）。
  - `docs/function-design/57-ui-monthly-sales.md`: `:397-399`、`:404`、`:456-458`（前月比と 1 位の class）。
  - `docs/design-system/00-foundations.md` のカラーパレット表の `--muted-foreground` の行: owner が選んだ HEX と、対 `--background`・対 `--card`・対 進行中の地の実測比へ（DS3 が `globals.css` と突合する）。
  - `docs/decision-log.md`: **D-094** を追加（D-CE1 の token 名と値の確定、D-CE16 の muted の候補 3 つと owner の選んだ値・比、試し 8 点の採否、試しを採らなかった場合の恒久の例外〈Home の入口 card の地・③強調の琥珀 pill、D-091 の求め〉、D-CE2 / D-CE9 / D-CE10 の役割の割当て、D-CE17 の focus 表示〈枠の無い部品と focus の前後で枠の色が変わらない部品は不透明な ring〉）。
  - `docs/backlog.md`: D-CE15 の除外 2 件と、`aria-invalid` の入力欄の focus（枠が赤のまま ring `ring-destructive/20`、合成後 1.41:1、既存。本 lane の対象外）の 1 件を起票する。「card の面色を白へ寄せる」の項目は残し「lane A から分けた（owner 決定 2026-09-27）」の 1 句を足す。「muted の文字色が通常サイズの説明文で AA に届かない箇所」は本 lane で閉じる（完了印は closeout）。lane A の項目と「destructive Alert の soft 塗り + 三角 icon」の完了印も closeout で付ける。
- **S18（Coordinator、plan-first commit と owner 回答の反映 commit）**: 本 packet、Matrix、`docs/Plans.md` の登録 1 行。Writer は触らない。
- **S19 muted の文字色**: `src/styles/globals.css` の `--muted-foreground` を D-CE16 の候補の値へ（PR head は M2、L3 の答えで確定）。`src/styles/globals.test.ts` に選んだ値の literal を足す。site の class は変えない。
- **S20 枠の無い部品の focus 表示**: D-CE17。`button.tsx`・`accordion.tsx`・`scroll-area.tsx`・`checkbox.tsx` の `focus-visible:ring-ring/50` を `focus-visible:ring-ring` へ、`ReturnExchangePage.tsx:147` の `focus-within:ring-ring/50` を `focus-within:ring-ring` へ、`button.tsx` の `destructive` の `focus-visible:ring-destructive/20` を外す。`toggle.tsx:8` の `focus-visible:ring-ring/50` を `focus-visible:ring-ring` へ（裁定 r3 B3-3）。`ReturnExchangePage.tsx:625`（枠を持つ textarea）は変えない。
- **S21 ネイティブの radio・checkbox の checked**（裁定 r3 B3-9）: `ReturnExchangePage.tsx:567,590` の radio（`className="mt-1"`）と `StockUnitField.tsx:79` の checkbox（className 無し）に `accent-primary` を足し、checked を操作の色にする（00「checkbox・radio の checked」の行。`--color-primary` から生成）。比は #1D5C63 対 選択カードの地（`bg-primary/5` を #fafaf9 に合成 = #eff2f1）6.75:1、対 `bg-warning-soft` #fffbeb 7.33:1、対 `--background` 7.28:1（相対輝度、2026-09-28）。

### L3 の分岐

PR head（L3 round 1 で見せる after）は 00 の「役割」「段」の列の答えで作る。試しの版（L3 round 1 の 3 本目の build。4 本目は muted の M1 だけを変えた比較 build）は、Writer が PR head の上に 1 commit で作り、PR の branch へは入れない（採った分だけ S15 で PR へ入れる）。

| 試し | PR head（after） | 試しの版 | 採ったとき | 採らなかったとき |
|---|---|---|---|---|
| ランキング 1 位（D-CE13） | 順位と太字 | —（PR head が試し） | S11 のまま。`--rank-top-*` を削る | 琥珀 pill と行の地へ戻す（`--rank-top-*` を残す）。00 の色の役割表と迷いやすい場面、04 原則 4、DSR-08 のランキングの文、review-checklist カテゴリ 9 の badge の行を現行へ（00 の注記どおり） |
| 詳細を開いた行（D-CE7） | 進行中の地とバー | —（PR head が試し） | S9 のまま | stone（`bg-muted`）へ戻し、在庫少のセルは `text-warning-emphasis` のまま（`table.tsx:27` の変更は残す）。00 の表と色の役割表、02 ⑫ の狙いの 1 文、DSR-22 の詳細を開いた行の文をふつう・補足の段 0 へ、DSR-22 と 04 原則 10 の「開いている行」を「入力や編集のために開いている行」と書き分ける（00 の注記どおり） |
| 取込みの手順の表示（D-CE8） | いまのステップを進行中の段 2 | —（PR head が試し） | S10 のまま | 現行の class のまま（いまのステップ = `border-primary bg-primary text-primary-foreground`、済み = `border-primary bg-primary/10 text-primary`、先 = muted。`--primary` の値で操作の色になる）。00 の「取込みの手順の表示」の行をこの形へ直し、押すボタンでない塗り（段 4）の例外を D-094 に記録する |
| Home の入口 card（D-CE12） | 操作の新しい線 + 注意の薄い地 | 操作の線 + `bg-ongoing-soft` | `ActionButton.tsx` の地を `bg-ongoing-soft` へ。00・02・04 の該当文を試しの答えへ | そのまま。D-094 に恒久の例外（操作の要素に注意の薄い地）として記録 |
| 日報の取込み済みの badge | 注意・確認（現行） | 取込み済みの分岐だけ `tone="destructive"`（Alert と同じ危険・失敗）。同日追加確認の分岐は注意・確認のまま | `DailyReportImportPage.tsx` の badge を危険・失敗へ。00 の表と注記を直す | そのまま |
| Home の前日分の未取込み | 危険・失敗（現行、D-CE2 の soft の形） | `variant="warning"` + `AlertTriangle` | `HomePage.tsx:78` を注意・確認へ。00 の色の役割表と表を直す | そのまま |
| 最新・上書き件数（D-CE3） | 琥珀 pill（`--warning-emphasis` の塗り） | stone の pill と太字（`variant="secondary"` + `font-semibold`） | S5 を stone の pill と太字へ。00・02 ⑬・04 原則 4・review-checklist を直す | そのまま。D-094 に恒久の例外（押すボタンでない badge の塗り）として記録 |
| muted の文字色（D-CE16） | M2 #6b6560 | M3 #57534e（試しの版）と M1 #6f6964（追加の比較 build） | owner が選んだ 1 つを `globals.css`・`globals.test.ts`・00 のパレット表・D-094 へ | —（採らない選択肢は無い。どれか 1 つを採る） |

## Non-scope

- `src-tauri/**`、`src/lib/bindings.ts`、`src/routeTree.gen.ts`、`package.json`、`package-lock.json`、`docs/design-system/reference/**`（mockup、正本でない）、`.agents/**`。
- **card の面色を白へ寄せる**（backlog の項目）: owner 決定 2026-09-27: 別 lane。backlog の項目は残す。
- `EmptyState` の説明文の `text-stone-500`（`src/components/patterns/EmptyState.tsx:32`、catalog ⑥ が stone の生の class を指定）: `--muted-foreground` を読まないため S19 で変わらない。空状態は主に `--background` の上（4.59:1、AA）に置かれる。token へ寄せるかは catalog ⑥ の改訂になるため除外し、Writer が backlog へ 1 行起票する。
- 作業中の囲み（段 3、2px 枠）の新設と、取込み中の領域の進行中の地（段 1）: backlog の lane A の対象一覧に無く、使う画面（棚卸し D1）は後続 lane。 backlog の lane A の L3 項目「進行中の囲みが『まだ終わっていない』と受け取られるか」は、棚卸し D1 の lane の L3 へ移す（本 lane の L3 は現在行と詳細を開いた行で進行中の地の受け取り方を見る）。
- `table.tsx` の既定（`data-[state=selected]:bg-muted`・`has-aria-expanded:bg-muted/50`）: stone のまま（D-CE7）。
- `formatErrorRow.ts:22` の危険の塗りの badge、`PriceRevisionTable.tsx:104` の「入力中」: D-CE15 で除外し backlog へ。
- ボタンの高さ、badge の文字の太さ（500）、書体: 変えない（L3 で見るだけ、または別 lane）。
- Toast（Sonner）の色: 変えない（`toast.success` / `toast.error` は役割どおり）。

## Acceptance Criteria

baseline は起票時実測（`e7c22f8f`）。「分岐」と書いた AC は L3 の答えで期待値が決まり、PR body に採った側を書く。

- **AC1 token**: `rg -c -e '--ongoing(-soft|-border|-strong)?:' src/styles/globals.css` = 4（baseline 0 = 出力なし）/ `rg -n -e '--primary: #' -e '--ring: #' src/styles/globals.css` の 2 行とも `#1d5c63`（大文字小文字不問。baseline `#b45309` の 2 行、`:82`・`:84`）/ runtime の参照: `rg -n 'row-current' src --glob '!*.test.*' | wc -l` = 0（baseline 3）/ 分岐: 1 位の試しを採れば `rg -n 'rank-top' src --glob '!*.test.*' | wc -l` = 0（baseline 8）、採らなければ 8。test の中の旧 token（不在を確かめる `not.toContain` 等）は数えない。
- **AC2 00 と DS3**: `bash scripts/doc-consistency-check.sh` で DS3 が OK、ERROR 0 / ``rg -c '`--ongoing' docs/design-system/00-foundations.md`` ≥ 4（baseline 0）/ `rg -n 'row-current' docs/design-system/00-foundations.md | wc -l` = 0（baseline 2）。
- **AC3 destructive Alert**: `rg -c 'bg-card text-destructive' src/components/ui/alert.tsx` = 0（baseline 1）/ `rg -c 'bg-destructive-soft border-destructive text-destructive-strong' src/components/ui/alert.tsx` = 1（baseline 0）/ `rg -U -c '<Alert variant="destructive"[^>]*>\s*<AlertTriangle' src --glob '!*.test.*'` の合計 = 0（baseline 6）/ `rg -c '<Alert variant="destructive"' src --glob '!*.test.*'` の合計 = 52（baseline 53、`ImportingStep` の 1 が注意・確認へ。分岐: Home の前日分の試しを採れば 51）/ `alert.test.tsx` の destructive の test が PASS。
- **AC4 badge**: `rg -n '<Badge variant="default"' src --glob '!*.test.*' | wc -l` = 0（baseline 2）/ `rg -c 'defaultVariants' src/components/ui/badge.tsx` = 0（baseline 1）/ `rg -c 'bg-primary' src/components/ui/badge.tsx` = 0（baseline 1）/ 分岐: 最新・上書き件数の試しを採らなければ `rg -c 'bg-warning-emphasis' src/features/backup-restore/BackupRestorePage.tsx src/features/products/import/ProductImportPreview.tsx` が各 1（baseline 0）、採れば各 0。
- **AC5 spinner と棒**: `rg -n 'animate-spin text-primary' src | wc -l` = 0（baseline 5）/ `rg -c 'bg-warning' src/components/ui/progress.tsx` = 0（baseline 1）/ `rg -n 'before:bg-warning' src | wc -l` = 0（baseline 1）/ `rg -c 'indicatorClassName' src/features/monthly-sales/components/DepartmentTable.tsx` = 1（baseline 0）。
- **AC6 現在行**: `rg -c 'border-l-ongoing bg-ongoing-soft' src/features/suppliers/components/SupplierPickerDialog.tsx` = 1（baseline 0）。
- **AC7 詳細を開いた行（分岐）**: 数えるのは class の文字列（二重引用符の中）だけで、comment 行は数えない。採れば `rg -o '"[^"]*bg-ongoing-soft[^"]*"' src/features/stock-inquiry/components/ProductListTable.tsx | wc -l` ≥ 2 と `rg -o '"[^"]*bg-ongoing-soft[^"]*"' src/features/operation-logs/OperationLogsPage.tsx | wc -l` ≥ 1（baseline 0 / 0）、`rg -o '"[^"]*bg-muted[^"]*"' src/features/stock-inquiry/components/ProductListTable.tsx | wc -l` = 0（baseline 1）、`rg -o '"[^"]*text-warning-strong[^"]*"' src/features/stock-inquiry/components/ProductListTable.tsx | wc -l` ≥ 1（baseline 0）。採らなければ 4 つとも baseline のまま。どちらでも `rg -c 'has-aria-expanded:bg-muted/50 data-\[state=selected\]:bg-muted' src/components/ui/table.tsx` = 1（不変）と `rg -c 'last-child\]:border-b-0' src/components/ui/table.tsx` = 1（baseline 0）・`rg -c 'last-child\]:border-0' src/components/ui/table.tsx` = 0（baseline 1）。
- **AC8 取込みの手順の表示（分岐）**: 数えるのは class の文字列だけ。採れば `rg -o '"[^"]*primary[^"]*"' src/features/csv-import/components/StepIndicator.tsx | wc -l` = 0（baseline 2）と `rg -o '"[^"]*bg-ongoing-soft[^"]*"' src/features/csv-import/components/StepIndicator.tsx | wc -l` = 1（baseline 0）。採らなければ同じ 2 式が 2 と 0（どちらも baseline のまま）で、D-094 に例外の記録がある。どちらの側でも StepIndicator の test がその側の class を assert する。
- **AC9 前月比**: `rg -c 'success-soft|destructive-soft' src/features/monthly-sales/components/comparison-cell.tsx` = 0（baseline 2）/ `rg -c 'text-success-strong|text-destructive-strong' src/features/monthly-sales/components/comparison-cell.tsx` = 2（baseline 0）/ `rg -c 'bg-success-soft text-success|bg-destructive-soft text-destructive' docs/function-design/57-ui-monthly-sales.md` = 0（baseline 2）。
- **AC10 失敗ではない赤**: `rg -c 'variant="destructive"' src/features/csv-import/components/ImportingStep.tsx` = 0（baseline 1）/ `rg -c 'variant="warning"' src/features/csv-import/components/ImportingStep.tsx` = 1（baseline 0）/ `rg -B1 '未保存の(入庫|手動販売|廃棄・破損|返品・交換)内容があります' src --glob '!*.test.*' | rg -c 'text-destructive'` = 0（baseline 4）/ 同 `| rg -c 'text-warning-emphasis'` = 4（baseline 0）。
- **AC11 drift**: `rg -n 'warning-foreground|info-soft|border-info|text-info-strong' src | wc -l` = 0（baseline 2）。
- **AC12 docs の移行の印**: 起票時実測 #15 の command の出力 = 0 行（baseline 36）/ 更新履歴の行を除く `runtime lane A` と `row-current`（`--row-current` を含む）: `rg -n 'runtime lane A|row-current' docs/design-system --glob '!**/reference/**' | rg -v '^[^:]+:[0-9]+:\| 20[0-9]{2}-[0-9]{2}-[0-9]{2} \|' | wc -l` = 0（baseline 40 = 00 25 / 04 5 / README 4 / 01 3 / 02 3、2026-09-28 に `b3844379` で実行。更新履歴の表の行〈`| 2026-09-03 |` 等で始まる行〉と `reference/` を除く。README の runtime lane B の移行の文は `runtime lane A` を含まない形で残る。裁定 r3 B3-1。旧式の `rg -n 'row-current' docs/design-system/01-decision-rules.md docs/design-system/02-component-catalog.md | wc -l`〈baseline 4〉は 01 の更新履歴 `:527` を数えるため、この式に置き換えた）/ 00 の「迷いやすい場面」表の移行列の 23 行が「済」か owner の答えの形（Final Review が表を直読みして確かめる）。
- **AC13 merge 直前の sweep**（backlog の AC）: merge 直前に `origin/main` を 1 回 merge し、Matrix の Adjacent Pattern Audit の「最終 sweep の式」9 本を再実行して、全 hit が同 Audit の分類（追従 / 移した / 除外）のどれかに当たることを PR body に記録する。分類に無い hit が 1 つでもあれば merge しない。
- **AC14 検証**: 対象 test（S16 の file）が vitest で PASS、`npm run typecheck` / `npm run lint` / `npm run format:check` PASS、`cd src-tauri && cargo run --bin generate_traceability -- --check` PASS（T4 の数が不変）、最終 `bash scripts/local-ci.sh full` PASS（fresh worktree では `npm run generate:routes` を先に実行）。L3 の前に Writer が `cargo check --release` を実行する（DEV_WORKFLOW Implementation Rules）。
- **AC15**: `bash scripts/doc-consistency-check.sh --target plan` ERROR 0 / `bash scripts/check-workflow-git.sh` PASS。
- **AC16 負の oracle**: `git diff --name-only origin/main...HEAD -- src-tauri src/lib/bindings.ts src/routeTree.gen.ts package.json package-lock.json docs/design-system/reference .agents | wc -l` = 0。
- **AC17 muted の文字色**: `rg -n -e '--muted-foreground: #' src/styles/globals.css` の値が D-CE16 の M1 / M2 / M3 のどれか（baseline `#78716c`、`:75`）/ 差分検査: `diff <(git grep -c 'muted-foreground' origin/main -- src ':!*.test.*' | sed 's/^origin\/main://') <(git grep -c 'muted-foreground' HEAD -- src ':!*.test.*' | sed 's/^HEAD://')` の出力に現れる file が `DepartmentTable.tsx`（S6 の `bg-muted-foreground` で増える）・`comparison-cell.tsx`（S12 で 0 から増える）・`StepIndicator.tsx`（S10）だけで、他の file の件数は減らない（baseline: 出力なし・exit 0、`git grep -c` の対象 75 file。3 file の baseline は 1 / 0 / 3）/ DS3 が OK（00 の `--muted-foreground` の行の HEX が `globals.css` と一致、AC2）/ `globals.test.ts` の muted の literal の test が PASS。
- **AC18 focus 表示（D-CE17）**: `rg -c 'ring-ring/50' src/components/ui/button.tsx src/components/ui/accordion.tsx src/components/ui/scroll-area.tsx src/components/ui/checkbox.tsx src/components/ui/toggle.tsx` が各 0（baseline 各 1。toggle は裁定 r3 B3-3 で追加）/ `rg -c 'focus-within:ring-ring/50' src/features/return-exchange/ReturnExchangePage.tsx` = 0（baseline 1）・`rg -c 'focus-visible:ring-ring/50' src/features/return-exchange/ReturnExchangePage.tsx` = 1（不変、textarea）/ checkbox の test に checked + focus の class の assert（`data-state="checked"` で `focus-visible:ring-ring` を持ち `ring-ring/50` を持たない）/ `rg -c 'focus-visible:ring-destructive/20' src/components/ui/button.tsx` = 0（baseline 1）/ 合成後の比は Contract Probe の式で対 background 7.28:1・対 card 6.97:1（不透明のため `--ring` の比と同じ）/ `StatusChips.test.tsx` の押された chip（`data-state="on"`）の focus の assert が PASS（`focus-visible:ring-ring` を持ち `ring-ring/50` を持たない。chip の外側の ring の比は透過で対 background 2.35:1・対 card 2.33:1、不透明で 7.28:1・6.97:1）/ `button.test.tsx` の focus の assert が PASS / 正本: `rg -c '枠の無い部品と、focus の前後で枠の色が変わらない部品.*不透明な' docs/UI_TECH_STACK.md` = 1（baseline 0。§5.4 系統①、裁定 r3 B3-2）。
- **AC19 ネイティブの radio・checkbox（S21）**: `rg -c 'accent-primary' src/features/return-exchange/ReturnExchangePage.tsx` = 2・`rg -c 'accent-primary' src/features/products/components/StockUnitField.tsx` = 1（baseline どちらも出力なし = 0、2026-09-28）/ `rg -n 'type="(radio|checkbox)"' src --glob '!*.test.*' | wc -l` = 3（不変。増えていれば分類を足す）。
- **AC-L3**（画面 / 到達手順 / 観測可能な合格基準。結果は github mode の `manual` record に残す。round 1 は build 4 本〈before = main、after = PR head、試し = 試しの版、muted の M1 の比較 build〉を並べる。round 2 は採否を反映した PR head だけで、round 1 から変わった画面を見る）:
  - **AC-L3 の前提（DB の共有、裁定 r3 B3-4）**: 4 build は `src-tauri/tauri.conf.json` の identifier が同じ（`com.kosei.inventory`）で、app_data の `inventory.db` を共有する。Coordinator は AC-L3-3 / 5 / 6 / 7 の前提（最後の Z004 取込みの精算日が前日より前、取込み済みの日と未取込みの日の日報に合う取込みの状態）を 1 つで満たす合成 DB（以下「L3 の DB」）を用意して hash（`sha256sum`）を取り、元の `inventory.db`（と `-wal`・`-shm`）を退避する。各 build の起動前に、同じ hash の L3 の DB の写しを app_data の `inventory.db` の位置へ置き直し（`-wal`・`-shm` は消す）、写しの hash と build 名を manual record に書く。AC-L3-7 の前に、対象日の日報が未取込みであることを取込み画面で確かめる。AC-L3-3 で手動販売を保存したら（AC-L3-7 で取り込んだ後も）、次の build の前に置き直す。L3 の後は、退避した元の DB を app_data へ戻す。
  - **AC-L3-1 全体の並べ比べ**: Home（`/`）と入庫記録（`/inventory/receiving`）/ 起動して左のナビから開く / after でナビの現在地のバー・主要ボタンの塗り・focus ring（Tab で移る）が同じ操作の色で、在庫少の琥珀・PLU 通知バーの琥珀と別の色に見える。
  - **AC-L3-2 ランキング 1 位**: 月次売上（`/reports/monthly`）の商品別 / ナビ →「月次売上」→ 商品別 / 1 位が色なしで順位と太字だけで見分けられる（採否）。round 1 で試しを採らなかったため（owner 2026-09-28）、round 2 では 1 位の行が琥珀の pill「1 位」と行の地で見分けられることを PASS / FAIL で見る。
  - **AC-L3-3 最新と手動**: バックアップ（`/settings/backup`、最新）と日次売上（`/reports/daily`、手動）/ ナビから各画面。日次売上の既定の表示日は今日（`DailySalesPage.tsx:46`）で、「手動」badge はその日に手動販売があるときだけ出る（`daily-sales/components/ProductTable.tsx:132` の `item.source === "manual"`）。L3 の DB で手動販売出庫（`/inventory/manual-sale`）を 1 件保存してから今日の日次売上を開く（または手動販売のある日を L3 の DB に入れておき、日付の移動で開く。保存したら次の build の前に L3 の DB を置き直す）/ 「最新」と②分類の「手動」を言い分けられる。琥珀 pill（after）と stone の pill + 太字（試し）を比べる（採否）。あわせて badge の文字の太さ（現行 500）が読み分けに足りるかを owner が言う（変えるなら別 lane）。
  - **AC-L3-4 Home の入口 card の 3 状態**: Home（`/`）/ 起動直後 / before（琥珀の線 + 注意の薄い地）・after（操作の新しい線 + 注意の薄い地）・試し（操作の線 + 操作の仲間の薄い地）を並べ、最重要の入口 1 つとして目に留まる形を owner が選ぶ（採否。before は見比べるだけで、選ぶのは after か試し）。
  - **AC-L3-5 日報の取込み済み**: 売上データ取込み（`/csv-import`）の日報取込みタブ / 取込み済みの日の日報を選ぶ（CP932 の合成 fixture は Coordinator が用意） / Alert（危険・失敗）と badge（after = 注意・確認、試し = 危険・失敗。同日追加確認の badge は注意・確認のまま）の役割を見て選ぶ（採否）。
  - **AC-L3-6 Home の未取込みの知らせ**: Home（`/`）/ L3 の DB（最後の Z004 取込みの精算日が前日より前）で起動（日報の取込みでは知らせは消えない）/ after（危険・失敗の soft の地 + 三角 icon）と試し（注意・確認）を比べる（採否）。
  - **AC-L3-7 操作と完了の言い分け**: 日報取込みの結果の画面（`/csv-import`、`DailyReportImportPage.tsx` の結果）/ 未取込みの日の日報を取り込む / 完了の緑の badge（icon + 文言「成功」）と操作の塗りのボタンを文字と icon で言い分けられる（D-091 の比 1.52:1 のため色では区別できない前提）。見るのは結果の画面の言い分けだけで、取込み中の spinner は合成 fixture では一瞬しか出ないため、進行中の色は AC5 と spinner の class の assert（S16 の `DailyReportImportPage.test.tsx`）で確かめる（裁定 r3 B3-5）。
  - **AC-L3-8 現在行**: 入庫記録（`/inventory/receiving`）の取引先を選ぶ dialog / 取引先の欄を押す / 選んでいる行が左端のバー + 進行中の地 + 「選択中」で分かる。
  - **AC-L3-9 詳細を開いた行**: 在庫照会（`/stock`）と操作ログ（`/settings/logs`）/ 在庫照会は行を押して詳細を開く・もう一度押して閉じる・別の行を開く・一覧の最後の行を開く。操作ログは行の「詳細を表示」ボタンで開き「詳細を閉じる」で閉じる（行そのものには開閉の操作が無い、`OperationLogsPage.tsx:540-558`）・最後の行を開く / 開いた行と詳細が一体に見え、最後の行でも左のバーが詳細の行まで続く。在庫少の行を開いたとき在庫数が読める。緑寄りの色に違和感が無いか、3 点目に badge を足さない形で分かるか、行内の薄い文字が読めるかを owner が言う（採否）。
  - **AC-L3-10 取込みの手順の表示**: 売上データ取込み（`/csv-import`）の商品別CSV取込み（Z004）タブ / ファイルを選ぶ → プレビュー（確定は停止中のためステップ 3 は出ない）/ いまのステップが進行中の段 2 で目に留まり、済んだステップ（プレビュー時の 1）と先のステップ（3）を見分けられる（採否。before は見比べるだけで、選ぶのは after か、採らない側〈現行の class〉。結果の画面の見え方は StepIndicator の自動 test だけで確かめ、停止の解除〈㉘ の ⑤〉の lane の L3 で見る）。
  - **AC-L3-11 前月比**: 月次売上（`/reports/monthly`）/ 部門別と商品別 / 地の無い文字色 + 記号で増減が読める（D-CE9）。
  - **AC-L3-12 危険・失敗の Alert の見本**: 日報取込みの二重取込みの Alert（AC-L3-5 と同じ画面、`/csv-import`）/ 同上 / 薄い地 + 線 + 三角 icon + 文言で「止まる」と受け取れる。あわせて日報取込みの画面で、危険の Alert（二重取込み）と注意（同日追加確認）の Alert を、三角 icon のまま文言で言い分けられる（どちらも三角 icon、owner 決定 2026-09-06 / 09-15。裁定 r3 B3-10）。
  - **AC-L3-13 失敗ではない知らせ**: 入庫記録（`/inventory/receiving`）/ 明細を 1 行入れた後、商品コード欄に存在しないコードを入れて「追加」を押し（Enter でもよい）、「該当する商品がありません」を出す（案内は `searchMessage === "該当する商品がありません"` かつ明細 1 行以上のときだけ出る。`ReceivingPage.tsx:490-501`、手動販売・廃棄・返品交換も同じ）/ 「未保存の入庫内容があります」が琥珀の文字で、失敗の赤と取り違えない（取込み中の移動制限の知らせ〈`ImportingStep.tsx`〉は Z004 の確定が停止中で画面に出ないため、自動 test だけで確かめる）。
  - **AC-L3-14 muted の文字色**: 在庫照会（`/stock`、部門・取引先の列と詳細を開いた行 = 進行中の地の上）、日報取込みの結果の画面（`/csv-import`、`DailyReportImportPage.tsx` の結果の card の「取込み ID」「対象日」等のラベル = `--card` の上）、入庫記録（`/inventory/receiving`、説明文 = `--background` の上）/ ナビから各画面（日報の結果は AC-L3-7 の続き）/ M2（PR head）・M3（試しの版）・M1（追加の比較 build）を並べ、説明文が通常距離で読め、見出しや値より薄い補足と分かる 1 つを owner が選ぶ（採否。どれか 1 つを採る）。
  - **AC-L3-15 返品・交換の選択カードと低視力の条件**: 返品・交換（`/inventory/return`）の登録方法の選択カード（D-CE15、owner 決定 2026-09-27 の確認先）/ 選択を切り替え、Tab で focus を当てる / 「レジ戻し済み」を選んだカードは枠と地が操作の色（`border-primary bg-primary/5`、`--primary` に追従）、「レジ未処理」を選んだカードは現行の注意・確認（`border-warning-border bg-warning-soft`、変えない）のままで、どちらも選択状態と分かる。選んだ radio の点が操作の色（`accent-primary`、S21）で見える。focus の外側の ring が不透明な操作の色で見える（D-CE17）。あわせて代表画面（Home `/`、在庫照会 `/stock`、日報取込み `/csv-import`）で DSR-22 の低視力 L3 の (a) Windows の forced-colors（ハイコントラスト）で状態・枠・focus が消えない、(b) DPI 125% / 150% で崩れない、を見る（(c) の実利用者の 1 セッションは本 lane の L3 に含めない。Residual）。

## Design Sources

- Requirements / spec: owner 2026-09-24 のデザインの見直しの判断と、ランキング 1 位の試しの了承（D-091 Status）、owner 2026-09-25 の詳細を開いた行の試しの了承、runtime lane A を 1 PR で出すことの了承（2026-09-24、backlog）、owner 所感 2026-09-15（destructive Alert の soft 塗り + 三角 icon、backlog）
- Architecture: 該当なし（UI 層のみ）
- Function / command / DTO: `docs/function-design/57-ui-monthly-sales.md` §57.7 / §57.10（前月比と 1 位の表示。S17 で同期）
- DB: 該当なし
- Screen / UI: `docs/design-system/00-foundations.md`（色の役割・強調の段階・迷いやすい場面・カラーパレット・セマンティックカラー）、`01-decision-rules.md` DSR-08 / DSR-20 / DSR-21 / DSR-22、`02-component-catalog.md` ⑥ / ⑫ / ⑬、`04-backbone.md` 原則 2 / 4 / 10、`docs/SCREEN_DESIGN.md`、`.agents/skills/inventory-operator-ui/SKILL.md`
- Decision log / ADR: [D-091](../../decision-log.md#d-091-デザインの決まりを見る人の受け取り方から組み直す2026-09-24)（候補値・コントラスト・試し・移し先）、D-094（本 lane で予約）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | not applicable |
| Command / DTO / generated binding / wire shape | なし | not applicable |
| DB / transaction / audit / rollback / migration | なし | not applicable |
| Screen / UI / route state / Japanese wording | 00 / 01 / 02 / 04 / README / review-checklist / UI_TECH_STACK / SCREEN_DESIGN / 57 | existing sufficient（D-091 と 00 が狙いを決め済み）。本 PR は現行の実装・移行の列と、試しの採否の反映を更新する |
| CSV / TSV / report / import / export format | なし | not applicable |
| Durable decision / ADR | D-094 | updated in this PR（token の確定値・試しの採否・恒久の例外） |

## Registration / Generation Obligations

新規 command / route / 画面 / function-design doc / REQ なし。`progress.tsx` の `indicatorClassName?` は TypeScript の optional prop で bindings に依らない。`docs/function-design/57-ui-monthly-sales.md` の編集は REQ 参照を増減しないため 90-traceability の再生成は要らない（`generate_traceability -- --check` で確かめる、AC14）。新しい FE test file は REQ か UI の ID を含める（T4）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-COLOR-EMPHASIS-RT-1 | 00 色の役割（操作）/ 迷いやすい場面「focus ring」「checkbox の checked」「現在地と現在行」「役割色の文字だけの表示」のリンク | D-091、D-CE1 | 操作を琥珀から分け 1 色 1 役割にする。site ごとの置換は採らず token の値で追従させる（壊さない） | S1 | `globals.test.ts`、`SidebarLink.test.tsx:149`（`border-l-primary` 不変） |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 色の役割（進行中）/ 迷いやすい場面「待ちの spinner」「進み具合の棒」「現在地と現在行」 | D-CE1 / D-CE4 / D-CE5 / D-CE6 | 進行中の家族を既存の家族と同じ形で足す。進行中の線は 3:1 未満のためバーと棒は `--ongoing` | S1 / S6 / S7 / S8 | `globals.test.ts`、`SupplierPickerDialog.test.tsx`、`DepartmentTable.test.tsx` |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 迷いやすい場面「危険・失敗の Alert」、DSR-08、02 ⑥ | D-CE2 | 部品が icon を描き 53 site の書き忘れを防ぐ。53 site の個別追加（差分が大きく、今後の site で漏れる）は採らない | S2 / S3 | `alert.test.tsx` |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 強調の段階（段 4 は押すボタンだけ）、D-091 の③強調の移し先 | D-CE3 | badge の既定の塗りを無くし、琥珀 pill は `--warning-emphasis` で保つ（`--warning` は 3.05:1 で不可） | S4 / S5 | `badge.test.tsx`、`BackupRestorePage.test.tsx` |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 迷いやすい場面「詳細を開いた行」、02 ⑫、DSR-22 | D-CE7 | 既定を変えず site で上書き（選択欄を開いた入力行を進行中にしない） | S9 | `ProductListTable.test.tsx`、`OperationLogsPage.test.tsx` |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 迷いやすい場面「取込みの手順の表示」 | D-CE8 | 塗りは押すボタンだけのため段 2 へ | S10 | StepIndicator の新 test |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 迷いやすい場面「ランキング 1 位」 | D-CE13 | 色でなく順位と太字（owner 了承の試し） | S11 | `ProductRankingTable.test.tsx` |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 迷いやすい場面「役割色の文字だけの表示」、DSR-08、57 §57.7 | D-CE9 | 段 1 は領域だけ。chip に枠と icon を足す案（段 2 の badge 化）は強調が増えるため採らない | S12 | 前月比の test（`DepartmentTable.test.tsx` / `ProductRankingTable.test.tsx`） |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 色の役割（注意・確認 / 危険・失敗の使う場面） | D-CE10 | 失敗でない知らせに危険の赤を使わない。ふつう・補足（Alert の既定）は「待ってほしい」が伝わりにくいため採らない | S13 | 移動制限・未保存の案内の test |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 色の役割表の注記（00 に無い token を使わない）、review-checklist カテゴリ 9 | D-CE11 | 生成されない class を外す。info 家族を作る案は D-091 が撤回済み | S14 | AC11 |
| SPEC-COLOR-EMPHASIS-RT-1 | 00 カラーパレット「サブテキスト」、ラベルと値 | D-CE16 | owner 決定 2026-09-27 で token の値を濃くする。site ごとに文字を濃くする案は 276 行に及び、ラベルと値の役割を site ごとに割るため採らない | S19 | `globals.test.ts`、DS3 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes。狙いは 00 の色の役割表・強調の段階・迷いやすい場面、値とコントラストと試しは D-091。本 packet は site の census と実装の形（D-CE1〜15）だけ
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: token 名（D-CE1）、destructive Alert の icon を部品が描くこと（D-CE2）、前月比・失敗でない知らせの役割（D-CE9 / D-CE10）、枠の無い部品と focus の前後で枠の色が変わらない部品の不透明な focus ring（D-CE17）は S17 で 00 / 02 ⑥ / UI_TECH_STACK §5.4 と D-094 へ上げる
- Assumptions and constraints: tailwind-merge が独自の色 token の class を同じ variant 同士で 1 つにする（Contract Probe で確認済み）。WebView2 の実描画は L3
- Deferred design gaps, risk, and follow-up target: card の面色（owner 決定 2026-09-27: 別 lane、backlog）。muted の文字色は D-CE16 で本 lane が閉じる。作業中の囲みと取込み中の領域の地（棚卸し D1 の lane）。D-CE15 の除外 2 件（backlog）
- Test Design Matrix can cite design decision IDs or source doc sections: yes（D-CE1〜15、00 の迷いやすい場面の行名）
- Absolute guarantee / escape hatch self-check completed: 「危険・失敗の Alert は全 site で icon が付く」は部品が描くことで担保し、例外（明示の icon の二重）は S3 と AC3 で 0 にする。「今の赤・琥珀・緑の割り当てを変えない」は、変える要素が試しの採否（owner の L3）と D-CE10 の 2 か所（失敗でない赤を琥珀へ）だけで、後者は AC-L3-13 で owner が見る

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable（UI の見た目のみ、CMD / BIZ 非接触） | — |
| Fact check / design decision split | 事実 = 起票時実測（site 数、コントラスト比、tailwind-merge の挙動）。判断 = D-CE1〜15 | 本 packet、Matrix |
| Lifecycle / retry | 行の開閉・選択・手順の進み・取込み中の状態で見た目が切り替わる | Matrix の State Lifecycle Matrix |
| Operator workflow | 操作手順・文言・URL は不変。状態の読み分けが変わる | Ordinary Operation、AC-L3 |
| Replacement path | not applicable | — |
| Data safety / evidence | not applicable（データ非接触）。L3 の fixture は合成 | Data Safety |
| Reporting / accounting semantics | 前月比の見た目だけが変わり、値と閾値（±1.0%）は不変 | AC9 |
| Manual verification | 15 項目を before / after / 試しで並べる | AC-L3-1〜15、github mode の manual record |
| 環境・再現性 | 新設の環境依存なし。L3 の build 4 本の手順は Coordinator が owner の環境に合わせて用意する | — |

## Design Readiness

- Existing design docs are sufficient because: D-091 と 00 が 6 役割・段 0〜4・迷いやすい 23 場面の答えと候補値を決め、02 ⑥ / ⑫ / ⑬ と DSR-08 / 21 / 22 が部品の形を持つ。backlog が「Plan で役割と段を決める」とした 3 点（取込みの手順の表示・前月比のセル・失敗ではない赤の知らせ）は、00 の既存の行（取込みの手順の表示・役割色の文字だけの表示・色の役割表の使う場面）を当てて決まる（D-CE8 / D-CE9 / D-CE10）
- Source docs updated in this PR: S17
- Design gaps intentionally deferred: card の面色（owner 決定 2026-09-27: 別 lane）
- Durable decisions discovered in this plan and promoted to source docs: D-CE1 / D-CE2 / D-CE9 / D-CE10 / D-CE16 / D-CE17 を D-094 と 00 / 02 / UI_TECH_STACK §5.4 へ

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI のみ
- Backend function design: 非接触
- Command / DTO / data contract: 非接触
- Persistence / transaction / audit impact: なし
- Operator workflow / Japanese UI wording: 文言不変（ランキング 1 位の「1 位」、「選択中」、Alert の文言はそのまま）
- Error, empty, retry, and recovery behavior: 取得失敗の Alert は見た目だけ変わり、再試行の導線は不変
- Testability and traceability IDs: SPEC-COLOR-EMPHASIS-RT-1。REQ の追加なし

## Contract Probe

- tailwind-merge 3.5.0 が独自の色 token の class を同じ variant 同士で後勝ちにする: 本 worktree で `node -e` により `twMerge('border-b hover:bg-muted/50 has-aria-expanded:bg-muted/50 data-[state=selected]:bg-muted','data-[state=selected]:bg-ongoing-soft')` → `border-b hover:bg-muted/50 has-aria-expanded:bg-muted/50 data-[state=selected]:bg-ongoing-soft`、`twMerge('bg-primary text-primary-foreground','border-warning bg-warning-emphasis')` → `text-primary-foreground border-warning bg-warning-emphasis`、`twMerge('relative bg-card text-card-foreground','bg-destructive-soft text-destructive-strong')` → `relative bg-destructive-soft text-destructive-strong`（2026-09-27、version は `node_modules/tailwind-merge/package.json` の 3.5.0）。variant の違う class（素の `bg-ongoing-soft` と `data-[state=selected]:bg-muted`）は merge されず、CSS の詳細度で variant 付きが勝つため、D-CE7 は同じ variant で書く
- コントラスト（WCAG 2.x の相対輝度、本 worktree の `$TMPDIR/cr.py` で計算、2026-09-27）: #1D5C63 対 #fafaf9 = 7.28:1、対 #f5f5f4 = 6.97:1 / #2F7F86 対 #fafaf9 = 4.47:1、対 #E6F0F0 = 4.02:1、対 #e7e5e4（棒の溝）= 3.72:1 / #123E43 対 #E6F0F0 = 10.07:1 / #7FB0B4 対 #fafaf9 = 2.29:1 / #78716c（muted）対 #E6F0F0 = 4.13:1、対 #f5f5f4 = 4.40:1 / #1c1917 対 #E6F0F0 = 15.06:1 / #7f1d1d 対 #fef2f2 = 9.16:1 / #b91c1c 対 #fef2f2 = 5.91:1 / #b45309 対 #f5f5f4 = 4.60:1、対 #fafaf9 = 4.81:1 / #1D5C63 対 #15803d = 1.52:1。含意: 現行の muted の文字は進行中の地の上で AA（4.5:1）に届かない（D-CE16 で濃くする）
- muted の文字色の候補（同じ `$TMPDIR/cr.py`、2026-09-27）: 表は D-CE16。計算に使った command は `python3 "$TMPDIR/cr.py" '#6f6964:#f5f5f4' '#6f6964:#fafaf9' '#6f6964:#E6F0F0' '#6f6964:#e7e5e4' '#6f6964:#1c1917'`（M2 #6b6560・M3 #57534e も同じ組で実行）
- 進行中の地（#E6F0F0）の上の文字色（同じ `$TMPDIR/cr.py`、2026-09-27。command は `python3 "$TMPDIR/cr.py" '#1c1917:#E6F0F0' '#78716c:#E6F0F0' '#b45309:#E6F0F0' '#78350f:#E6F0F0' '#b91c1c:#E6F0F0' '#123E43:#E6F0F0' '#1D5C63:#E6F0F0'`。載る site は開いた行・詳細の行〈`ProductListTable.tsx`、`StockDetailContent.tsx`、`OperationLogsPage.tsx` の `Detail`〉、現在行〈`SupplierPickerDialog.tsx`〉、いまのステップ〈`StepIndicator.tsx`〉、試しの入口 card〈`ActionButton.tsx`〉で、`rg -o 'text-[a-z-]+' <file>` で文字色を数えた）:

  | 文字色 | HEX | 載る site | 対 #E6F0F0 | 判定 |
  |---|---|---|---|---|
  | 本文 `text-foreground`（既定） | #1c1917 | 全 site の本文・outline badge の文字・ghost ボタンの文字 | 15.06:1 | AA |
  | `text-muted-foreground` | 現行 #78716c / M1 / M2 / M3 | 在庫照会の部門・取引先の列、詳細のラベルと商品コード、入口 card の説明 | 4.13 / 4.66 / 4.95 / 6.57 | 現行は未達、D-CE16 の 3 候補で AA |
  | `text-warning-emphasis`（在庫少のセル） | #b45309 | `ProductListTable.tsx:38` | 4.33:1 | **未達** → 開いた行では `text-warning-strong` |
  | `text-warning-strong` | #78350f | 開いた行の在庫少のセル（D-CE7） | 7.81:1 | AA |
  | `text-destructive`（在庫切れのセル、詳細の取得失敗の文） | #b91c1c | `ProductListTable.tsx:39`、`StockDetailContent.tsx:61` | 5.57:1 | AA |
  | `text-ongoing-strong` | #123E43 | いまのステップの番号 | 10.07:1 | AA |
  | `text-primary`（icon） | #1D5C63 | 試しの入口 card の icon | 6.55:1 | 非テキスト 3:1 を満たす |

  badge（soft の地）・outline ボタン（`bg-background`）・`pre`（`bg-muted`）は自分の地を持つため進行中の地の上の文字に数えない。
- focus ring の合成後の色（本 worktree の `$TMPDIR/blend.py`、合成色 = `α × ring + (1 − α) × 地` を sRGB で計算し相対輝度で比、2026-09-27）: `#1D5C63` 50% を #fafaf9 に合成 = #8cabae、対 #fafaf9 2.35:1 / #f5f5f4 に合成 = #89a8ac、対 #f5f5f4 2.33:1 / 現行の `#b45309` 50% = 2.08:1・2.05:1 / `#b91c1c` 20%（destructive の ring）を #fafaf9 に合成 = 1.41:1 / 不透明な `#1D5C63` = 7.28:1・6.97:1。Plan Review（Codex）も 2.35:1・2.32:1 と実測（card の値は丸めの差）
- checked の checkbox と「レジ戻し済み」の選択カードの focus（同じ `$TMPDIR/blend.py` / `$TMPDIR/cr.py`、2026-09-27。command は `python3 "$TMPDIR/blend.py" '#1D5C63:0.5:#fafaf9' '#1D5C63:0.5:#f5f5f4' '#1D5C63:1:#fafaf9' '#1D5C63:1:#f5f5f4'` と `python3 "$TMPDIR/cr.py" '#1D5C63:#1D5C63' '#8a8480:#fafaf9'`）: checked の枠と地 #1D5C63 は focus の前後で同じ（1.00:1）ため、枠の変化は目印にならない。外側の ring は透過で対 background 2.35:1・対 card 2.33:1（Plan Review r2〈Codex〉の Chrome 実測 2.354:1・2.321:1）、不透明で 7.28:1・6.97:1。未 checked は focus 前の枠 `--border-strong` 3.53:1 から `border-ring` 7.28:1 へ変わる
- `TableBody` の末尾の規則: Plan Review（Codex）が生成した Tailwind CSS を Chrome で描画し、最後のレコードを開くと親の行の左の枠は 4px、詳細の行は 0px（`[&_tr:last-child]:border-0` が勝つ）、`border-b-0` に限ると詳細の行も 4px と実測した（reviewer の実測。Writer が実装後に同じ probe か L3 の AC-L3-9 で確かめる）
- `tr` の左の枠が表に描かれる: Tailwind の preflight が `table` を `border-collapse: collapse` にし、`SupplierPickerDialog` の現在行（`border-l-4`）が既に描かれている（DSR-22 の現在行の L3 で確認済み）。WebView2 での在庫照会・操作ログの行は AC-L3-9 で確かめる
- WebView2 の実描画と色の受け取り方: L3（AC-L3-1〜15）

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| 00 色の役割「操作」（押すボタン・リンク・ナビの現在地・checked・focus ring） | S1（token の値。`button` / `badge` の `link` / `checkbox` / `SidebarLink` / `ring-ring` 系は追従） | `globals.test.ts`（値の literal）、`SidebarLink.test.tsx:149,166`（`border-l-primary` 不変） | AC-L3-1 |
| 00 色の役割「進行中」（現在行・spinner・棒・詳細を開いた行・手順のいまのステップ） | S1 / S6〜S10 | `SupplierPickerDialog.test.tsx`、`DepartmentTable.test.tsx`、`ProductListTable.test.tsx`、`OperationLogsPage.test.tsx`、StepIndicator の新 test | AC-L3-8〜10 |
| 00 色の役割「注意・確認」「完了」（済） | 非接触 | 既存 | AC-L3-7 |
| 00 色の役割「危険・失敗」の Alert（段 2、icon） | S2 / S3 | `alert.test.tsx` | AC-L3-12 |
| 00 色の役割表の注記（00 に無い token を使わない） | S14 | AC11 | — |
| 00 強調の段階（段 4 は押すボタンだけ、badge の塗りは③強調だけ） | S4 / S5 / S10 | `badge.test.tsx`、StepIndicator の新 test | AC-L3-3 / 10 |
| 00 迷いやすい場面 23 行（focus ring / 操作枠 / 保存中のボタン / 待ちの spinner / 進み具合の棒 / checked / 複数選択の行 / Home の入口 card / 画面上の確定・削除 / dialog の実行ボタン / 日報の取込み済み / 未取込みの知らせ / 危険・失敗の Alert / 減衰 / 注意の Alert と badge の線 / 完了の知らせ / 現在地と現在行 / 詳細を開いた行 / 取込みの手順の表示 / ランキング 1 位 / 最新・上書き件数 / お知らせ一般 / 役割色の文字だけの表示） | 行ごとの移し先は Matrix の Adjacent Pattern Audit。「済」の 9 行（操作枠・保存中のボタン・複数選択の行・画面上の確定・削除・dialog の実行ボタン・減衰・注意の Alert と badge の線・完了の知らせ・お知らせ一般）は非接触 | AC12（移行列）、各 test | AC-L3 の各項目 |
| 00 カラーパレット・セマンティックカラー（HEX を `globals.css` と一致） | S1 / S17 | DS3（AC2） | — |
| 00 サブテキスト `--muted-foreground`（ふつう・補足の muted の文字。WCAG 1.4.3 の 4.5:1、owner 決定 2026-09-27 で濃くする） | S19（D-CE16） | `globals.test.ts` の literal、DS3（AC17） | AC-L3-14 |
| DSR-08（非中立の Alert と①状態 badge は icon 必須、増減の ± の色） | S2 / S12 / S17 | `alert.test.tsx`、前月比の test | AC-L3-11 |
| DSR-20（dialog の実行ボタンは危険の塗り） | 非接触（10 site 不変） | 既存 | — |
| DSR-21（現在地は操作の細いバー、選択状態は stone） | S1（追従）、D-CE15（選択カードは owner 決定 2026-09-27、追従） | `SidebarLink.test.tsx` | AC-L3-1 / 15 |
| DSR-22（現在行 3 点、詳細を開いた行、操作枠 3:1） | S8 / S9 | `SupplierPickerDialog.test.tsx`、`ProductListTable.test.tsx` | AC-L3-8 / 9 |
| DSR-22（操作枠 3:1 を focus にも。枠の無い部品の focus は不透明な ring） | S20（D-CE17）/ S17（UI_TECH_STACK §5.4） | `button.test.tsx`、`StatusChips.test.tsx`（AC18） | AC-L3-1 / 15 |
| 00 迷いやすい場面「checkbox・radio の checked」（ネイティブの radio・checkbox） | S21 | AC19（`rg`） | AC-L3-15 |
| DSR-22 の低視力 L3（forced-colors、DPI 125% / 150%） | 非接触（見え方の確認） | — | AC-L3-15 |
| WCAG 1.4.3（進行中の地の上の文字 4.5:1） | S9（在庫少のセル）/ S19 | `ProductListTable.test.tsx`（選択・解除） | AC-L3-9 / 14 |
| 02 ⑥（エラーは destructive Alert、AlertTitle + AlertDescription の 2 段） | S2 / S17 | `alert.test.tsx` | AC-L3-12 |
| 02 ⑫（展開行を選択行と一体に見せ、hover で色を動かさない） | S9 | `ProductListTable.test.tsx` | AC-L3-9 |
| 02 ⑬（badge 3 種、③強調の枠 `--warning`） | S4 / S5 / S11 | `badge.test.tsx` | AC-L3-2 / 3 |
| D-091 の③強調の移し先（`--warning-emphasis`、4.81:1） | S5 | `BackupRestorePage.test.tsx` の class | AC-L3-3 |
| D-091 の試しと採らなかったときの処理（恒久の例外の記録） | S15 / S17（D-094） | — | AC-L3-2〜6 / 9 / 10 |
| 57 §57.7 / §57.10（前月比の閾値 ±1.0% と表示、1 位の強調） | S11 / S12 / S17 | 前月比・1 位の test | AC-L3-2 / 11 |
| 今の赤・琥珀・緑の割り当て（owner 決定 2026-09-06）を変えない | 非接触（在庫状態の badge・セル・件数、取消済み、増減） | 既存の `StockStatusBadge` / `ProductListTable` / `SummaryCards` の test が無変更で PASS | — |

隣接契約 sweep: 00 の「ラベルと値」「タイポグラフィ」「スペーシング」「アイコンサイズ」「書体」は非接触（三角 icon は 16px の Alert の既定 `[&>svg]:size-4` に従う）。02 ⑦ Toast、⑭ FilePicker（drag over は `--primary` で追従、D-CE15）、⑯ ListShell（`--row-current` の記述だけ S17）。

## Test Plan

Test Design Matrix: [2026-09-27-design-color-emphasis](test-matrices/2026-09-27-design-color-emphasis.md)。

- targeted tests: S16。部品（alert / badge / progress / globals）を先に RED → GREEN、画面は class に加えて文言・role・icon を assert する
- negative tests: destructive Alert の icon が 1 つだけ（二重にならない）、`warning` と `default` は部品が icon を描かない、`table.tsx` の既定が stone のまま、badge の variant 無指定で `bg-primary` を出さない
- compatibility checks: 在庫状態・取消済み・増減・注意の既存 test が無変更で PASS（赤・琥珀・緑の割り当て不変）
- data safety checks: not applicable
- main wiring/integration checks: 画面の test で、部品の変更が実画面に届く（例: `BackupRestorePage` の常に destructive のまま残る Alert〈`BackupRestorePage.tsx:324`〉が soft の class と icon 1 つを持つ。Home の前日分の知らせは試しの採否で variant が変わるため対象にしない）
- Human Gate に L3 を含むため、Writer 完了時に `cargo check --release` を実行する（Rust 非接触だが手順どおり）

## Boundary / Wire Contract

not applicable（JSON / CSV / DTO / bindings / route state 非接触。`indicatorClassName?` は component の props で wire ではない）。

## Review Focus

- D-CE2: 部品が icon を描く形が catalog ⑥ と DSR-08 と両立するか。6 site の二重 icon が残らないか。`role="alert"` の読み上げに icon が混ざらないか（`aria-hidden`）
- D-CE7: 同じ variant での上書きが tailwind-merge と CSS の詳細度の両方で効くか。選択欄を開いた入力行（廃棄・返品交換）が進行中にならないか
- D-CE9 / D-CE10: 役割の割当てが 00 の既存の行から一意に導けるか（新しい方針を足していないか）
- L3 の分岐: 採らなかった側の docs の直し（00 の注記の列挙）が漏れなく Scope にあるか
- 今の赤・琥珀・緑の割り当てが変わる site が D-CE10 と試し以外に無いか（開いた行の在庫少のセルは琥珀の家族の中で `-strong` へ替えるだけ）
- D-CE17: 不透明な ring を塗りのボタン・checked の checkbox に当てたときの見え方、対象を 6 部品（button・accordion・scroll-area・checkbox・返品・交換の選択カード・toggle）に限った census に漏れが無いか
- D-CE16: 3 候補の比が packet の計算式どおりか。濃くした muted と本文の差（ラベルと値の濃さの差）が L3 で見られるか

## Spec Contract

Contract ID: SPEC-COLOR-EMPHASIS-RT-1

| Contract | Test |
|---|---|
| 操作・進行中・危険・失敗の Alert・③強調・ランキング・前月比・失敗でない知らせが、00 の色の役割と段（試しは owner の L3 の答え）どおりに token・部品・画面へ反映され、今の赤・琥珀・緑の割り当てと文言・role・accessible name は不変で、00 / 01 / 02 / 04 / README / review-checklist から lane A の移行の印が消える。muted の文字色は owner が選んだ D-CE16 の候補で、card・background・進行中の地の上で 4.5:1 以上になる | `alert.test.tsx`、`badge.test.tsx`、`globals.test.ts`、`SupplierPickerDialog.test.tsx`、`ProductListTable.test.tsx`、`OperationLogsPage.test.tsx`、`ProductRankingTable.test.tsx`、StepIndicator の新 test、AC1〜AC13 |

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-COLOR-EMPHASIS-RT-1 | S1 / S4 / S5 | `globals.test.ts`、`badge.test.tsx` | D-CE1 / D-CE3 | AC1 / AC2 / AC4 |
| SPEC-COLOR-EMPHASIS-RT-1 | S2 / S3 | `alert.test.tsx` | D-CE2 | AC3 |
| SPEC-COLOR-EMPHASIS-RT-1 | S6〜S10 | 各 test | D-CE4〜D-CE8 | AC5〜AC8 |
| SPEC-COLOR-EMPHASIS-RT-1 | S11〜S14 | 各 test | D-CE9〜D-CE11、D-CE13 | AC9〜AC11 |
| SPEC-COLOR-EMPHASIS-RT-1 | S15 / S17 | AC12 / AC13 | L3 の分岐 | AC-L3、PR body |
| SPEC-COLOR-EMPHASIS-RT-1 | S19 | `globals.test.ts`、DS3 | D-CE16 | AC17、AC-L3-14 |

## Data Safety

- 実店舗の DB・日報・CSV・バックアップを commit しない。L3 の fixture（取込み済みの日の日報、数秒かかる大きさの商品別売上 CSV）は Coordinator が合成し、repo の外（`.local/`）に置く。取込みの fixture は実 encoding（CP932）にそろえる（DEV_WORKFLOW Human Visual Confirmation）
- local-only: L3 の build と screenshot（owner の手元）

## Implementation Results

Fill after implementation.

- 実装: [PR #116](https://github.com/higekiri-w/inventory-system-desktop/pull/116)。S1〜S17・S19〜S21 を Writer（Opus 5.5 subagent）が実装し、L3 round 1 の採否（遷移記録の最後の行）を S15〜S17 で反映した。
- ランキング 1 位（D-CE13）は分岐表の「採らなかったとき」で閉じた。「設計判断」の D-CE13、「L3 の分岐」の表の PR head の列、「Design Intent Trace」の SPEC-COLOR-EMPHASIS-RT-1（ランキング 1 位）の行にある「順位と太字」は計画時の試しの記述で、最終形は D-094・00・02 ⑬・04 原則 4 のとおり琥珀の pill と行の地である。
- Registration / Generation Obligations の「90-traceability の再生成は要らない」は計画時の見込みで、実装では新しい test の REQ 参照数が変わったため再生成した（`docs(traceability)` の commit）。
- 実装の要旨: 操作の色（`--primary` / `--ring` = #1D5C63）と進行中の家族（`--ongoing` / `-soft` / `-border` / `-strong`）を token に足し、`--row-current` を削った。危険・失敗の Alert は薄い地 + 線 + 文字の段 2 にし、三角 icon を部品が描く。badge の既定の塗りを削り、最新・上書き件数は `--warning-emphasis` の琥珀 pill へ移した。進み具合の棒・待ちの spinner・現在行・詳細を開いた行・取込みの手順のいまのステップを進行中の token へ、前月比のセルを地の無い役割色の文字 + 記号へ、失敗ではない知らせ（取込み中の移動制限・入力 4 画面の未保存の案内）を注意・確認へ移した。枠の無い部品と focus の前後で枠の色が変わらない部品の focus ring を不透明にし、ネイティブの radio・checkbox の checked を操作の色にした。muted の文字色は owner が選んだ M2 #6b6560。00 / 01 / 02 / 04 / README / review-checklist / UI_TECH_STACK / SCREEN_DESIGN / 56 / 57 を最終形へ直し、D-094 を記録した。
- 試しの採否（L3 round 1、遷移記録のとおり）: 詳細を開いた行・取込みの手順の表示・Home の入口 card（進行中の地）・日報の取込み済みの badge（危険・失敗）を採り、最新・上書き件数（琥珀 pill のまま）・Home の前日分の知らせ（危険・失敗のまま）・ランキング 1 位（琥珀の pill と行の地のまま）は採らなかった。採らなかった側の恒久の例外は D-094 に記録した。
- review の是正: Final Review broad の P2 2 件・P3 5 件（Review Response）を Writer（発注 121）と Coordinator の packet の同期で直した。
- CI・manual・merge: hosted CI は全ジョブ pass。manual は owner の Windows native L3 の round 1（採否）と round 2（round 1 から変わった 4 画面が PASS）。Ready・merge は helper 経由（2026-09-28）。

## Review Response

Fill after review.

- Final Review broad（2026-09-28、対象 `4420ca04`、互いに独立の 2 本）: Claude 側 Fable 5.1 = approve（P1 / P2 0、P3 4）、Codex 側 GPT-6 Astra（発注 120）= reject（P1 0 / P2 2 / P3 1）。Coordinator の裁定:
  - Codex P2-1（同日追加確認の注意の Alert 2 site〈日報取込み・CSV 取込みのプレビュー〉に三角 icon が無い。DSR-08 の「非中立の Alert は icon 必須」と AC-L3-12 に反する）: accept。Writer が 2 site に icon を足し、test で固定する。
  - Codex P2-2 と Fable P3-2（Ordinary Operation の 1 位の行が試しの文のまま）: accept。Coordinator が Ordinary Operation の行と AC-L3-2 を採った側へ直し、計画時の記述の読み方を Implementation Results に書いた。
  - Codex P3 と Fable P3-1（02 ⑧ の picker footer に「amber」が残る）: accept。Writer が「操作の塗り（`--primary`）」へ直す。
  - Fable P3-3（`comparison-cell.tsx` の同じ値の定数 2 つ）: accept。Writer が 1 つに畳む。
  - Fable P3-4（90-traceability の再生成の記述の食い違い）: accept。Implementation Results に事実を書いた。
  - 是正の後の closure は Claude 側の fresh reviewer で行う（Final Review Minimum 1）。
- Findings Freeze: not yet frozen; post-freeze exceptions: none.

- Closeout（2026-09-28）: Plan Review は round 3（上限）まで回し plan-approved（遷移記録のとおり）。Final Review broad の是正の後、closure 1（`4b08d6be`、fresh Fable 5.1）= approve（P 0）。小口のまとめ #118・#121 を含む main の取込みの後の closure 2（`5c23d983`、fresh Fable 5.1）= approve（新しい P1 / P2 0、P3 1）。その P3（`docs/backlog.md` の「旧 04 の未了の見た目の作業」の (a) が #118 で解消した幅の混在を未了のまま書いている）は本 closeout で backlog を直した。manual は owner の Windows native L3 の round 2（`5c23d983`、2026-09-28 23:42）で、round 1 から変わった 4 画面（AC-L3-2・4・5・12）がすべて PASS。専用 record は review pass（broad 2 + closure 2）、manual pass。hosted CI は全ジョブ pass、[PR #116](https://github.com/higekiri-w/inventory-system-desktop/pull/116) を squash merge（`e188366e`）。L3 round 2 の owner の所感 2 件（ランキングは 3 位まで強調したい、Alert を上に出すなら同じ意味の badge は要るのか）は、後のデザインの刷新で扱う（owner 判断）として backlog に置いた。
