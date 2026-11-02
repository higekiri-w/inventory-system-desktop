# Plan Packet: MG-D13 B の「L3 の項目に影響なし」の意味を書き、同期の closure の止めの理由と test を直す

wave に属さない単独の lane。merge の関門の文（`docs/agent-guidance/merge-evidence.md` MG-D13 の B）と helper（`scripts/pr-gate.py`）・その test を変える。製品の runtime（`src/**`・`src-tauri/**`）は変えない。

出所: owner の裁定 TD-516（2026-10-28、repo 外の回答台帳。#197 の 2 回目の同期の B の closure で Codex は表示の値の変化を理由に reject、Fable は approve し、owner が a を選んだ）。PR #217 の Final Review〈Fable 5〉の P3 2 件（repo 外の報告 `.local/reports/final-review/pr217-broad-fable-b09ffa30.md` の #1・#2。Coordinator の裁定で follow-up にした）。

decision-log の番号: 本 lane は D-146 を予約する（起票の時点で open の PR が足す D は D-143 まで。`gh pr list --state open` の各 branch の `docs/decision-log.md` の差分の `## D-` で確かめた）。

## Workflow State

- Risk: R3
- Plan Commit: df41889a4ad2b5fb0edba66db9d60d4538ffeeb9
- Amendments: none

## Risk

Risk: R3

Reason:
merge の関門（MG-D13 の L3 の使い回しの判定）の意味と、その helper（`scripts/pr-gate.py`）を変える。classifier が `workflow=true` を出す変更は R2 にできない（`docs/DEV_WORKFLOW.md` Risk Tiers）。

## Goal

Goal Invariant:

### 最小完了条件

- B の closure を頼む人と判定する reviewer が、「L3 の項目に影響なし」を owner の裁定 TD-516 と同じ意味で読める文が merge-evidence にある。TD-516 の裁定の元になった #197 の件（合否は変わらず、項目が見る一覧の「最終カウント」の表示値だけが変わった同期）が、その文で「影響なし」になる。
- R0/R1 の lane で `--kind review`（stage を問わず、同期の種類を問わず）を試すと、helper が「R0/R1 の lane に review の record は無い」と本当の理由で、何も書かずに止まる。
- manual の要る lane で `--review-stage sync` の 1 record が manual の A の引継ぎと sync の closure を同時に付ける流れを、test が直接確かめる。

### 失敗定義

- 書き足した文が、L3 の項目の操作が通る処理（裏のロジック）を変え、合否や業務の結果を変え得る変更まで「影響なし」に読める（L3 の使い回しが緩む）。
- 書き足した文が、合否と業務の結果を変えない表層の変化（項目が見る値・合格基準が名指す値を含む）を「影響あり」に読ませ、TD-516 の裁定と食い違う。
- helper の変更で、R2〜R4 の `--kind review` の今の受理・拒否が 1 つでも変わる。

### 非目的

- B の機械の条件（同期だけの条件・code に触れない条件）・A・C・docs の closure の条件を変えない。
- B の closure の本数（2 本）と owner の承認を変えない。
- `.local/` の発注の雛形は repo の外なので本 PR で変えない（手続きの道具の slimming で直す）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `docs/agent-guidance/merge-evidence.md` の「manualの使い回し」の (B 承認) の文: 「L3の項目に影響なし」の直後に意味を書く。文: 「影響なしは、同期で取り込んだ変更が L3 の各項目の合否と、確かめた操作・業務の結果を変えないこと。変わるのが画面の表層（表示の値・文言・配置）だけなら、項目が見る値（合格基準が名指す値を含む）でも影響に数えない。項目の操作が通る処理（裏のロジック）を変え、合否や業務の結果を変え得る変更は影響ありとする（owner の裁定 TD-516、2026-10-28。D-146）。」
- `docs/decision-log.md`: 末尾に D-146（B の「L3 の項目に影響なし」の意味。Status accepted〈owner の裁定 TD-516〉、Context に #197 の 2 回目の同期の割れた判定、Decision に上の文の要旨と規則の文の置き場〈merge-evidence〉、Alternatives に「合格基準が観測する事柄の変化は全部影響あり」〈Codex の reject の読み〉と「L3 をやり直す」〈TD-516 の b〉、Relation に D-144 の B の判定の意味を足すもので D-144 の Decision・Guarantee range は変えない）。D-144 には行を足さない（`docs/decision-log.md` の「Status の書き方」は merge 後の追記を Status と Superseded の行に限り、D-146 は D-144 を置き換えない）。
- `scripts/pr-gate.py` の `record`: capture の照合の後、record を組む前（`carry_manual` の呼出しより前）に、`--kind review` で `req['minimum']` が 0 なら「R0/R1 の lane に review の record は無い（`--kind review` は review のある lane だけ）」の文で exit 2 にするガードを 1 つ置く。`--review-stage sync`・`docs` の入口の `require` と、その他の review の入口の `require`（`'review needs stage/model/run-ref/evidence'`）から `req['minimum'] and` を消す（ガードが先に止めるので重ねない）。引数の検査と文はそのまま。
- `scripts/tests/pr-gate.test.py` の `SyncRecords` に 2 test を足す（下の Test Plan）。

Non-scope: `docs/DEV_WORKFLOW.md` の L3 の表（B の行は merge-evidence を指すので文を複製しない）。`docs/templates/subagent-review-packet.md`（B の closure の問いの文を持たないことを `rg -n "影響なし|L3 の項目" docs/templates` で確かめた。出力 0 件）。

## Acceptance Criteria

- AC1: merge-evidence の (B 承認) の文に、TD-516 と D-146 を引く意味の文があり、「合格基準が名指す値を含む」表層の変化を影響に数えないことと、操作が通る処理の変更を影響ありとすることを両方書く。`rg -n "TD-516" docs/agent-guidance/merge-evidence.md docs/decision-log.md` が両 file で 1 件以上、`rg -n "^## D-146" docs/decision-log.md` が 1 件。
- AC2: `LOW`（R0、minimum 0、manual あり）の req で、`--kind review` の stage 4 種（`broad`・`closure`・`sync`・`docs`、各 stage の正しい形の引数）を、docs だけの同期と code に触れる同期の両方で試すと、どれも exit 2 の `GateError` で止まり、文に `R0/R1` を含み `takes only` と `manual pass not carried` を含まない。何も書かない（`writes == []`）。
- AC3: `REQ`（R3、manual・r4 あり）で docs だけの同期の後、`--review-stage sync` の 1 record が manual を `source_head`=旧 L3 の版で引き継ぎ、review の closure を `sync_from` の形で付け、r4 を pending にする。
- AC4: `python3 scripts/tests/pr-gate.test.py` が全 PASS。今の `test_sync_stage_*`・`test_rc2_gate_docs_plan_change_and_evidence` を含め、既存の test を消さず弱めない。
- AC5: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし。
- AC6: AC2 の test はガードを消す mutation で red、AC3 の test は `carry_manual` の条件を `args.kind not in ('manual', 'review')` にする mutation で red になることを実装の報告に書く。

## Design Readiness

- 引用する設計正本（節まで）: `docs/agent-guidance/merge-evidence.md` の「manualの使い回し」（MG-D13）、`docs/decision-log.md` D-144 と「Status の書き方」、`docs/DEV_WORKFLOW.md` L3 の表（B の行）。
- 必要な設計成果物: merge-evidence の文と decision-log の D-146 を本 PR で書く（updated in this PR）。helper のガードは止めの文と止める位置の変更で、受理・拒否の集合を変えない（R0/R1 の `--kind review` は今も全部拒否。existing sufficient）。
- plan にしかない durable な判断の昇格先: TD-516 の意味を merge-evidence（規則の文）と D-146（判断の記録）へ。
- 前提・制約と延期した design gap: 「表層」と「操作が通る処理」の線引きは項目ごとに B の closure の reviewer と owner が持つ（機械では判定しない。D-144 の Guarantee range のまま）。表層の変化は、合格基準が名指す値の変化でも影響に数えない（TD-516 の裁定の元の件がその形）。
- 絶対保証の例外と抜け道: 主張しない。B は今も reviewer 2 本と owner の承認で、機械の保証でない。
- 判定: ready。正本の該当節が 2 か所と新しい D 1 件で、意味の追記と止めの位置の変更に留まる。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| MG-D13（B の判定の意味） | merge-evidence「manualの使い回し」(B 承認) | merge-evidence の文、D-146 | 非対象（文の意味。B は reviewer と owner の判定で機械の検査が無い） | 非対象 |
| MG-D13（R0/R1 の `--kind review` の拒否） | merge-evidence「reviewのclosure」「docs の closure」と、R0/R1 は review の record を持たないこと（「manualの使い回し」の R0/R1 の manual の lane の文） | `pr-gate.py` の `record` の入口のガード、2 つの `require` から `req['minimum'] and` を消す | `test_review_kind_names_no_review_for_r0r1`（新規）、今の `test_sync_stage_*`・`test_rc2_gate_docs_plan_change_and_evidence` | 非対象 |
| MG-D13（A の引継ぎと sync の closure の 1 record） | merge-evidence「manualの使い回し」(A 自動)・「reviewのclosure」 | 変更なし（今の挙動を test で固める） | `test_sync_stage_carries_manual_and_closure`（新規） | 非対象 |
| D-127（止めたときに理由を名指す） | AGENTS.md Working Rules「使い手に押しつけない」 | 同上のガード | 同上（新規の 1 本目） | 非対象 |
| decision-log の追記の規則 | decision-log「Status の書き方」 | D-146 を足し、D-144 は変えない | 非対象（文書の形） | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| R0/R1 の `--kind review` を止める理由 | R0/R1 の lane で引数をいくら直しても `takes only --outcome pass`・`review needs stage/...`・`manual pass not carried` の文で止まり、原因（review が無い）へ辿れない | unit（`SyncRecords`、合成 repo。subTest で stage 4 種 × docs だけの同期・code に触れる同期） | `test_review_kind_names_no_review_for_r0r1` | ガードを消す（各 stage が元の文で止まる、code の同期では `carry_manual` の文が先に出る）。文から `R0/R1` を消す。何かを書いてから止める |
| R2〜R4 の `--kind review` の今の受理・拒否 | ガードと `req['minimum'] and` の削除で、R3/R4 の拒否（`--pass-model` 付きの sync、`docs` の `--evidence` 無し）や R2 の差分の review（broad）の受理が変わる | unit / CLI | 今の `test_sync_stage_*`・`test_rc2_gate_docs_plan_change_and_evidence`・`test_r2n1_r2n3_r2_route`（全 PASS のまま） | 引数の検査を消す、ガードの条件を `minimum` 以外へ広げる |
| manual の要る lane の sync の 1 record | `Gate.record` の分岐の並びが変わり、sync の closure だけが付いて manual の引継ぎが落ちる | unit（`SyncRecords`、`req=REQ`・`stage='sync'`） | `test_sync_stage_carries_manual_and_closure` | `carry_manual` の条件を `args.kind not in ('manual', 'review')` にする。r4 を引き継ぐ |
