# Plan Packet: merge-evidence の B (ii) の Scope の文を、同期の手が持ち込む中身で見る形に置き換える

wave に属さない単独の lane。`docs/agent-guidance/merge-evidence.md` の「manualの使い回し」の B の (ii) の 1 箇条と `docs/decision-log.md` だけを変える。helper（`scripts/pr-gate.py`）・test・classifier・CI・製品の runtime は変えない。

出所は owner の決定（原文は repo 外の回答台帳。経緯と review の裁定は PR #265 の comment）:

- TD-743（2026-11-16）・TD-746（2026-11-17）: PR #194・#216 の B の closure で、同期の手が packet の Scope の予定 path に無いとして reviewer が割れ、owner がその回限りで Scope の中と裁定した。TD-743 は main が後から足した test に lane の hook を止める mock を足した手、TD-746 は L3 の版の lane の変更（予定 path に無かった backlog を含む）と main の変更を重ねた手。
- TD-747（2026-11-17）: 規則の文を直す lane を起こす。
- TD-750（2026-11-17）: 方向の承認（candidate）。L3 の版の lane の変更と main 側の変更を重ねる手と、main 側の test・docs を lane へ機械的に合わせる手は Scope の外と数えない。それを超える中身は今までどおり Scope と照らす。(ii) の置き換えで、条件を継ぎ足さない。
- TD-751（2026-11-17）: L3 の後の lane の commit も同じ線で見る。
- TD-753・TD-755（2026-11-17）: 文と packet を短く組み直す。

decision-log の番号: D-155 を予約する（下の「調べ」）。

安全の境界: (ii) の Scope の照らしは、`docs/DEV_WORKFLOW.md` Workflow State（D-098）の「古い証拠の拒否」「独立 review」の運用に関わる。本 lane はその不合格の条件を緩める向きに改訂し、通常の packet・PR の経路で行う。変えないもの: 集合の判定、(i)、両側を落とさない・test を消さない弱めない、Scope の外の新しい中身は Gated Amendment まで不合格、B の closure の本数・見る範囲・承認と証跡、Plan Commit の固定、R4 の `r4` の承認、fail-closed、A・C、review の closure の機械の記録。

## Workflow State

- Risk: R3
- Plan Commit: ffebcfef4f50a865950ccb69edfaa2f794f991c1
- Amendments: none

## Risk

Risk: R3

Reason:
gate の文書 `docs/agent-guidance/merge-evidence.md` を変える（classifier が `workflow=true`。下の「調べ」）。R4 にしない理由: 破壊的な操作・実データ・secret・不可逆な git の操作を伴わず、helper の code・RecordV1・GitHub への書込みの経路も変えない。

## Goal

Goal Invariant: B の closure の (ii) が、下の「規則の形」の新しい文で、TD-743・TD-746 の形の手を Scope と照らさず、それ以外の lane の手を今までどおり Scope と照らす。

### 最小完了条件

- merge-evidence の (ii) が新しい文に置き換わり、Test Plan の R1〜R7 の判定が期待のとおりになる。
- decision-log に D-155 があり、D-154 に `Superseded in part by: D-155` の 1 行がある。

### 失敗定義

- R1〜R7 のどれかが期待と違う判定に読める。
- (ii) の外の文（(i)、集合、本数、承認、fail-closed）が変わる、または規則の文が merge-evidence の外に言い直される。
- 手の目的（同期の辻褄合わせか）を closure に判定させる文になる（D-154 の Alternatives で TD-723 がやめた形）。

### 非目的

- helper の code・案内の文・test（helper は B を名指す 1 行だけを出す。D-154）。
- A・C・集合・本数・見る範囲・承認と証跡。
- 新しい中身を Scope と照らす読み（予定 path でか振舞いでか）。今の文から引き継ぎ、closure が割れたら owner の裁定（TD-750 の承認は照らさない手の線だけ）。
- `docs/templates/subagent-review-packet.md` と `.local` の発注書の雛形。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 調べ（2026-11-17、`origin/main` = `0cf0a26186029cea09c1dab9d0fc4506a3197c5e`）

- decision-log の番号: `git grep -n "^## D-15" origin/main -- docs/decision-log.md` は D-150（:1335）・D-151（:1346）・D-152（:1357）・D-154（:1368）の見出し（exit 0）。`git grep -n -E "D-15[5-9]" origin/main -- docs` は当たり無し（exit 1）。D-153 は PR #250（`agent/window-close-guard`）が予約済み（`docs/plans/2026-11-14-sync-docs-conflict.md:16`。手元の `origin/agent/*` の ref でも D-155〜D-159 は 0 件。ref は本 run で fetch していない）。よって D-155。
- classifier（`printf '%s\n' <path> | bash scripts/ci/classify-changes.sh --files-from-stdin | grep '=true'`）: merge-evidence は `docs`・`workflow`、decision-log と本 packet は `docs`。
- helper は B の基準を持たない: `rg -n -i "scope" scripts/pr-gate.py` は 5 行（:316・:358 の計画の節の名の列、:444 の `workflow_scope`、:794・:795 の `record` の引数）。B を名指すのは `rg -n "manualの使い回し|見るものと合否" scripts/pr-gate.py` の :64（`B_RULE`）と :790 だけ。
- 規則の言い直しは無い: `rg -n "両側.*落とさ|承認済みの Scope|Scope の外の変更|Scope の中にある" --glob '!docs/templates/subagent-review-packet.md' --glob '!.local/**' --glob '!docs/plans/2026-11-14-sync-docs-conflict.md' --glob '!docs/archive/**' .` は `docs/agent-guidance/merge-evidence.md:165` の 1 行だけ。
- TD-743 の手の現物: `src/features/daily-report-import/DailyReportImportRecordsPage.test.tsx` の 26〜32 行は、`useAutoBackupCheck: () => undefined` の stub と `vi.fn()` 群で hook を止め、comment は「この画面の test の外なので hook を止める（隣の route の test と同じ）」。

## 規則の形

新しい (ii) の文（今の (ii) の箇条を丸ごと置き換える。前の 2 文は今と同じ）:

> - (ii) lane の手の普通の正しさ。手で解いた path は両側（lane 側と main 側）の変更を落とさない。test を消す・skip する・弱めることをしない。lane の手のうち、L3 の版の lane の変更（Scope の予定 path に無いものを含む）と main 側の変更を重ねる直しと、main 側の test・docs の件数・名・参照を L3 の版の lane に合わせる直し（その test が確かめない hook・module を止める mock を含む）は Scope と照らさない。それ以外（新しい assertion、test が期待する振舞いや文の契約を変える直し〈mock の返り値によるものを含む〉、削除）は承認済みの Scope（packet の `## Scope` 節。packet の無い R0〜R2 の lane は PR body の 5 行の「範囲」の行）と照らし、外なら Gated Amendment が通るまで不合格。どの commit で入れた手かは問わず、最後の結果で見る（TD-750・TD-751、D-155）。

文が言わないこと（closure の読みの補い）:

- 「lane の手」は merge-evidence の「lane の手と集合」の定義のまま（手で解いた path と、C に当たらない lane の commit）。C で機械が受ける commit は入らない。
- L3 の版と Scope の食い違い（予定 path に無い lane の変更が L3 の版にある）は (ii) の合否に数えない。closure が気づいたら packet の docs の follow-up（P3）として報告し、契約に及ぶ食い違いは中身に応じて (i) か通常の finding として扱う。D-155 の Decision に 1 句で置く。

## Scope

- `docs/agent-guidance/merge-evidence.md`: B の (ii) の箇条（今の 165 行）を新しい文に置き換える。ほかの文は変えない。
- `docs/decision-log.md`:
  - 末尾に D-155（同期の手と L3 の後の lane の commit のうち、L3 の版の lane の変更と main 側の変更を重ねる直しと、main 側の test・docs を L3 の版の lane に合わせる直しは B の (ii) で Scope と照らさない。2026-11-17）。Status: owner の決定 TD-743・746・747・750・751・753・755、本 packet で起票（TD-750 は candidate で、計画の承認の確認を経て書く）。Context: TD-743・TD-746 で closure が割れ、owner がその回限りで裁定した。Decision: 基準の文は merge-evidence の (ii) が持ち、D-155 は要旨と参照（D-146・D-154 と同じ置き方）。L3 の版と Scope の食い違いの扱い（上の「文が言わないこと」）。Why: L3 の版の lane の変更は同期の前からあり L3 の版の review の範囲、main 側の件数・名・参照を合わせる直しは新しい振舞いを持ち込まない、置き場所で線を引くと TD-723 に反する。Alternatives: 回ごとに owner が裁定する（TD-747）、同期のたびに Gated Amendment を出す、Scope の照らしを外す、手の目的を closure に判定させる（TD-723）、例外を継ぎ足す（D-127）、helper に照らさせる（中身は機械で判定できない）。Guarantee range: 当てはめは closure と owner の判断で機械の保証でない。線の上で割れうる。照らす読み（予定 path か振舞いか）は今の文から引き継ぎ、割れたら owner の裁定。Compatibility: D-154 (4) が merge-evidence に置く B の基準のうち (ii) の Scope の照らしを置き換える。ほかの D と集合・本数・helper は変えない。D-098 の安全の境界の運用を通常の経路で緩める向きに改訂した。Revisit: 照らさなかった手が merge の後で lane の振舞いと合わないと分かったとき、closure がまた割れたとき。
  - D-154 に `Superseded in part by: D-155（2026-11-17。(4) が merge-evidence に置く B の基準のうち (ii) の Scope の照らし）` の 1 行だけを足す（D-144 の行と同じ形）。
- 本 packet。

Non-scope: `scripts/**`、`docs/DEV_WORKFLOW.md`、`AGENTS.md`、`docs/AGENT_OPERATING_MANUAL.md`、`docs/templates/**`、merge 済みの packet、`docs/backlog.md`・`docs/Plans.md`（`rg -n "TD-74[3-9]|TD-750" docs/backlog.md docs/Plans.md` は当たり無し、exit 1）。

この変更で不要になるもの: TD-743・TD-746 のような、(ii) の Scope の割れへの owner のその回限りの裁定。ほかに消す記述は無い（「調べ」の `rg`）。

## Acceptance Criteria

- AC1: `git diff origin/main -- docs/agent-guidance/merge-evidence.md` の差分が (ii) の箇条の 1 行だけで、その行が「規則の形」の文と同じ。
- AC2（削除の検査）: `rg -n "の中にある。Scope の外の変更は今の経路" docs/agent-guidance docs/decision-log.md` が 0 件（exit 1）。新しい文の固有の句「は Scope と照らさない。それ以外」が merge-evidence に 1 件。
- AC3: `docs/decision-log.md` の差分が D-155 の節と D-154 の 1 行だけ。`rg -n "は Scope と照らさない。それ以外" docs scripts --glob '!**/plans/**' --glob '!**/archive/**'` の当たりが merge-evidence の 1 行だけ（D-155 は文を言い直さない）。隔離した copy（`git archive` の `9064774` を展開し、merge-evidence の (ii) を新しい文に置き換えた版）で、この `rg` は `docs/agent-guidance/merge-evidence.md:165:` の 1 行（exit 0）、AC2 の旧い句の `rg` は exit 1 だった。
- AC4: `git diff --stat origin/main -- scripts .github AGENTS.md docs/DEV_WORKFLOW.md` が空。
- AC5: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-17-b-scope-sync-hands.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が exit 0。
- AC6: 計画の review と Final の review の報告（PR の comment）が、Test Plan の R1〜R7 を `docs/agent-guidance/merge-evidence.md` の新しい文に当てた判定を書き、期待から外れた行が 0。

## Design Readiness

- 引用する設計正本: merge-evidence の「lane の手と集合」と「manualの使い回し」の B、decision-log D-154・D-144・D-146・D-098、`docs/DEV_WORKFLOW.md` の Workflow State、`AGENTS.md` の「継ぎ足さない」。
- 必要な設計成果物: (ii) の文と D-155 を updated in this PR（計画の固定の後）。
- plan にしかない durable な判断の昇格先: 「文が言わないこと」→ D-155。
- 前提・制約と延期した design gap: 当てはめは closure と owner の判断で、機械で判定しない（helper を変えない）。
- 絶対保証の例外と抜け道: 新しい振舞いを件数・mock の形に見せかける手は、文が「期待する振舞いや文の契約を変える直し〈mock の返り値によるもの〉」を照らす側に置いて塞ぐ。(ii) を抜けても L3 の項目を変える変更は (i) で止まる。
- 判定: ready（TD-750 の candidate は計画の承認で確かめる）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MG-D13 B (ii) | merge-evidence「manualの使い回し」の B の (ii) | (ii) の箇条の置き換え | 無し（文の判定。R1〜R7、AC1・AC2） | 非対象 |
| MG-D13 B のほか・集合 | merge-evidence「manualの使い回し」の B、「lane の手と集合」 | 変えない | AC1・AC4 | 非対象 |
| D-154 | decision-log D-154 (4) | `Superseded in part by: D-155` の 1 行 | AC3 | 非対象 |
| D-155 | decision-log（新規） | 本 lane の決定 | AC3・AC5 | 非対象 |
| D-146 | decision-log D-146（(i)） | 変えない | AC1 | 非対象 |
| D-098 | DEV_WORKFLOW Workflow State の安全の境界 | 緩める向きの改訂。変えないものは前文 | R3・R4・R7・AC1 | 非対象 |
| D-127 | AGENTS.md「継ぎ足さない」 | (ii) の置き換えで、例外を足さない | R6 | 非対象 |

隣の契約（「reviewのclosure」、C・docs の closure、DEV_WORKFLOW の L3 の表、`AGENTS.md` の Ready の承認）は (ii) を引かないので触らない（「調べ」の `rg`）。

## Test Plan

review の項目は、計画の review と Final の review が新しい文に当てて判定する（文の判定に機械の test を置かない。helper は判定しない、D-154）。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| (ii) TD-743 | main 側の test に、その test が確かめない hook を止める mock（stub と `vi.fn()` 群）を足した手が照らされて不合格になる | review | R1 TD-743 の形は照らさない | mock を照らさない側から外す、または mock の返り値を lane と同じに求める |
| (ii) TD-746 | L3 の版の lane の変更（予定 path に無い backlog）と main の変更を重ねた手が不合格になる | review | R2 TD-746 の形は照らさない | 重ねる直しを予定 path の中に限る |
| (ii) 反例 | 同期の merge の中で、Scope に無い新しい機能の test や仕様の文を足した手が合格する | review | R3 Scope の外の新しい中身は不合格 | 同期の merge の中の手を全部照らさない |
| (ii) 線の両側 | 不合格のはずの (a)(b)(e)(f) が合格、または合格のはずの (c)(d) が不合格。(a) mock の返り値で test が確かめる振舞いを変えた手（Scope の外）。(b) Scope に無い振舞いの新しい assertion。(c) Scope の中の新しい test を足した手。(d) main 側の sweep の test の件数に L3 の版の lane の画面を入れた手。(e) main 側の function-design の文の契約を lane の契約に書き換えた手（Scope の外）。(f) main 側の test の期待する振舞い（保存で確認の dialog が出ない）を lane の振舞い（出る）へ反転させた手（Scope の外） | review | R4 線の両側 | 照らす側から mock の返り値・期待する振舞い・文の契約を落とす、件数の直しを照らす、新しい中身を照らさずに一律に不合格にする |
| (ii) lane の commit | 同じ結果でも置き場所で判定が変わる。(a) 同期の後の 1 親の commit で main 側の test に hook を止める mock を足す → 照らさない。(b) Scope の外の新しい振舞いを足す commit → 不合格。(c) 同期の merge で main 側の文を落とし、直す commit で戻した版（最後の結果は正しく重ねた merge と同じ）→ 照らさない | review | R5 置き場所を問わず最後の結果 | 対象を手で解いた path に限る、lane の commit を照らしから外す、親の数で分ける |
| (ii) 変えないもの | (ii) の外の文が変わる、例外が継ぎ足される、手の目的を判定させる | review・doc | R6 置き換えの形 | 差分が (ii) の外に及ぶ（AC1）、旧い文が残る（AC2） |
| (ii) 削除 | L3 の後の lane の commit が Scope の外の main 側の docs の文を消す変更が合格する | review | R7 削除も照らす | 照らす側から削除を落とす |
| doc の形 | packet・docs の形の ERROR | CLI | D1 doc-consistency（全体と `--target plan`）、D2 check-workflow-git | link 切れ・節の欠け・Workflow State の形の誤り |

L3: 置かない。operator の画面・app の build を変えず、規則の文だけを変える（DEV_WORKFLOW L3 Eligibility の (1) を満たさない）。

## Owner Effort Budget

- owner の手は、計画の承認の確認（TD-750 の candidate を confirmed にする）と、実装の後の Ready の承認の 2 回の見込み（merge-evidence は gate の文書で、R0 の常設の委任に当たらない）。
- L3 の後の lane の commit も同じ線で見ることは owner が決めた（TD-751）。
