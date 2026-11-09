# UI-13 WER 起源の workflow docs 正本化（登録・生成義務 checklist / Contract Probe 手順 / materialize 局所検査）

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 1fbeb32
- Amendments: none
- Coordinator: Fable
- Writer: Fable（docs 3 file の軽微修正につき Coordinator 直接実装。自己承認回避のため Plan / Final review は独立 context に出す）
- Plan Reviewer: independent Sonnet review context
- Final Reviewer: workflow gate change につき Double Audit — 同一のフル Contract Audit（Ledger 突合 / negative-space / drift-fix / 回帰・迂回）を相互に独立な context で 2 回冗長実施（DEV_WORKFLOW Review Rules の Double audit 定義どおり。レンズ分割はしない — 冗長性が見逃しを拾う設計、PR #159 miss #13 前例）+ Fable 裁定
- Reviewed Content HEAD: ee2acc7534250472cf46d42fcff1a6a66f5f1280
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required（workflow 契約 docs の変更。docs-only につき PR event は filter され、Ready 後の owner-directed `workflow_dispatch` で final を取得する）
- Human Gate: none（owner 事前指示 2026-07-15「このまま1の workflow docs PR もおねがい」= plan scope・Ready・merge の事前承認 = 介入 1 回目。round 1 P1 による失効条項該当を owner に確認し「維持して続行、トークン節約しつつ、必要な場合はサブエージェント使いながら進めて」で事前承認維持を再確認 = 介入 2 回目 / 予算 2（調整後）。以降の scope 逸脱・新規 P1/P2 は再び owner に戻す）

Plan Gate record（append-only）:

- ready-hosted-final → merge → archive（2026-07-15、Post-Merge Closeout）: PR #7 squash merge（`2be498b`）。hosted final = workflow_dispatch run success + 三点 SHA 一致（run URL / headSha は PR body）。本 commit で packet / matrix を `docs/archive/plans/` へ移動、WER は同ディレクトリ
- human-confirm → ready-hosted-final（2026-07-15、state-only 遷移 commit）: Human Gate は owner 事前承認（介入 1 回目）+ round 2 P1-b の owner 再確認（介入 2 回目 / 予算 2）で消化済み。Ready と hosted `workflow_dispatch`（docs-only につき PR event filter、explicit dispatch 経路）は事前承認の範囲内として Coordinator が実行する
- implementing → local-verified → independent-review → human-confirm（2026-07-15、本承認記録 content commit に相乗り、全遷移の証跡は commit 前に存在）: ① local-verified 証跡 = content candidate `ee2acc7` での L1 full PASS / CLEAN / MERGE_EVIDENCE_VALID=true（evidence は PR body 記載）、② independent-review 証跡 = Double Audit（同一フル Contract Audit を独立 context ×2 で冗長実施）。pass A: P1/P2/P3 = 0、Ledger 4 行全 pass、negative-space / drift / 回帰・迂回 / scope 逸脱すべて clean。pass B: P1/P2 = 0 / P3 = 1（packet 内 3 箇所の「8 項目」表記が round 3 の 9 token 化に未同期）、Ledger 4 行全 pass、③ 裁定 = Fable、P3 accept・処方どおり本 commit で訂正（Test Plan / Review Focus / Spec Contract の表記のみ、実装 diff への影響なし）。Human Gate は owner 事前承認消化済みにつき human-confirm の残項目なし
- plan-gate → plan-approved → implementing（2026-07-15、単一 state-only commit で materialize、canonical compression）: plan review rally 4 round で収束（round 4: P1/P2/P3 = 0、「Plan Gate 可」）。Plan Commit = 1fbeb32（plan-first、実装 commit なしを ancestry で確認）。owner 事前承認は round 2 P1-b の owner 再確認で維持済み
- plan review round 4（同 independent context、commit 1fbeb32 の diff に対して）: P1 = 0 / P2 = 0 / P3 = 0、新規指摘なし、Plan Gate 可
- plan review round 3（同 independent context）: P1 = 0 / P2 = 1（Matrix 行 2 が AC ⑨ 追随漏れ、Coordinator の round 2 反映時の同期漏れ）→ accept、処方どおり Matrix を AC と 1:1 再同期。⑤ rebut は reviewer が memory 原本と M2 checker 実体で裏取りし異論なしと確定
- plan review round 2 P1-b の消化（2026-07-15）: owner 確認済み「維持して続行」（介入 2 回目 / 予算 2）。事前承認は維持、予算調整は Owner Effort Budget 調整記録参照
- plan review round 2（同 independent context、round 1 反映後の commit c357d9e に対して）: P1 = 2 / P2 = 1、裁定 = Fable。P1-a（Double Audit のレンズ分割が正本の冗長設計に反する）→ accept、フル Contract Audit ×2 の冗長実施に修正。P1-b（Human Gate 失効条項を Writer 兼任 Coordinator が自己裁定で狭く再解釈）→ accept、owner に「round 1 P1 発生済みだが事前承認維持でよいか」を確認する（この確認が介入 2 回目になるため予算 1→2 の調整承認も同時に依頼）。P2（AC token の 1:1 対応不備）→ partial accept: ⑨「到達導線」token を追加し token → 表行対応を明記。⑤「必須セクション」の削除処方は rebut — memory checklist の実在義務（新設 function-design doc は compliance checker の必須セクション充足が必要）であり template 表行に文言として存在する
- plan review round 1（independent Sonnet context）: P1 = 1（Risk R2 自己判定が DEV_WORKFLOW Risk Tiers「workflow gate に触れるなら R3」と矛盾）→ accept、R3 へ昇格し Ledger / Matrix / Spec Contract / Trace Matrix / Double Audit を追加。P3 = 1（AC の checklist 8 項目が単一 prose 条件）→ accept、個別 rg token に分割。Assumption「checker は template 節構成を token 検査しない」は reviewer が script 実体で裏取り済み（PK 検査は dated active plan のみ、M2 は function-design のみ）。R3 昇格は scope 逸脱でなく検証重量の増加のため、owner 事前承認は維持（P1/P2 失効条項は plan 内容の欠陥を想定したもので、本件は workflow 重量の是正）

## Owner Effort Budget

- 介入回数上限: 2
- 実働時間上限: 15分
- relay 往復上限: 0

調整記録（append-only）: 2026-07-15 介入上限 1→2（owner 承認、介入 2 回目 / 予算 2）。理由: plan review round 1 P1（Risk 昇格）が Human Gate 失効条項に文言上該当し、Writer 兼任 Coordinator の自己裁定で維持を確定させず owner 確認に戻したため（round 2 P1-b の処方）。owner 指示: トークン節約 + 必要時 subagent 活用。

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
workflow gate change（`docs/DEV_WORKFLOW.md` の Plan Packet Rules / Workflow State という人間手続き gate の正本、および全 R2+ plan を拘束する template の変更）。DEV_WORKFLOW の Risk Tiers「workflow gate に触れるなら R3」の明文規則および PR #2 / PR #4 の前例（docs 中心の workflow 変更を R3 + Double Audit で処理）に従う。機械 gate（checker / CI / git 検査）のロジックには触れず runtime contract の変更もないが、それは R2 の根拠にならない（plan review round 1 P1 で是正）。

## Goal

Goal Invariant:

### 最小完了条件

- 次に R2+ の Plan Packet を書く作業者が、template を開いた時点で登録・生成義務 checklist（UI-13 Amendment 1〜4 の failure class 対策）を参照でき、Contract Probe の「是正仮適用で end-to-end」手順と forward materialize commit 直後の `check-workflow-git.sh` 局所実行が `docs/DEV_WORKFLOW.md` の正本に存在する。

### 失敗定義

- checklist が Claude 私有 memory にしか存在しない状態が続く。
- 規範文言の追加が既存 gate の挙動・enum・checker 期待 token を変えてしまう。

### 非目的

- checker / hook / CI への機械強制の実装（D-046-7 のとおり docs 宣言だけでは不十分な失敗クラスだが、機械化は発動データを見て別 PR で判断する）。
- 既存 packet・archive の遡及修正。
- Plan Packet template の checklist 以外の構造変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

- `docs/templates/plan-packet.md`: `## Registration / Generation Obligations` 節を Required Design Artifacts の直後に新設（新規 command / 新設 doc / 新規 REQ / 新規 route / 新規 operator 画面の登録・生成義務チェック表。該当行は Scope と、R3/R4 では Contract Coverage Ledger に反映してから Plan Gate に出す。operator 到達導線は Ledger 標準行）。Contract Probe 節に「登録漏れ是正を含む probe は是正を仮適用した状態で end-to-end に回す」1 文を追記
- `docs/DEV_WORKFLOW.md`: Contract Probe 規定（Plan Packet Rules 内)に同旨の 1 文を追記。Workflow State の state-only 遷移規則に「forward materialize commit を作成したら直後に `bash scripts/check-workflow-git.sh` を実行し、STATECAP 超過を commit 時点で検出する」1 文を追記

## Non-scope

- `scripts/doc-consistency-check.sh` / `scripts/check-workflow-git.sh` / CI workflow / hook の変更
- 既存 enum・checker 期待 token・phase 遷移表の変更
- memory 側 checklist の削除（template 昇格後も Claude 向け運用注記として残す）

## Acceptance Criteria

- `rg "Registration / Generation Obligations" docs/templates/plan-packet.md` が節 header を返す
- template の checklist（表 5 行・義務 9 token）が個別に検査できる（各 1 hit 以上、token → 表行の対応を 1:1 明記）: command 新規行 = ① `rg "collect_commands"` ② `rg "specta::specta"` ③ `rg "generate_bindings"` / doc 新設行 = ④ `rg "design_compliance_test"` ⑤ `rg "必須セクション"`（compliance checker の必須セクション充足義務） / REQ coverage 行 = ⑥ `rg "generate_traceability"` / route 新設行 = ⑦ `rg "generate:routes"` / operator 画面行 = ⑧ `rg "navigation"`（有効化 + 到達テスト） ⑨ `rg "到達導線"`（Ledger 標準行の文言）— いずれも対象は `docs/templates/plan-packet.md`
- `rg "是正を仮適用" docs/DEV_WORKFLOW.md docs/templates/plan-packet.md` が両ファイルに hit する
- `rg "check-workflow-git.sh" docs/DEV_WORKFLOW.md` が state-only 遷移規則の文脈で hit する（exit code 0）
- `bash scripts/doc-consistency-check.sh` ERROR 0
- `bash scripts/local-ci.sh full` PASS / CLEAN（gate 挙動が変わっていないことの回帰確認を兼ねる）

## Design Sources

- Requirements / spec: [UI-13 WER](2026-07-15-ui13-integrity-check-workflow-effectiveness-review.md) Recommended Workflow Adjustment（Change 3 点）
- Architecture: `docs/DEV_WORKFLOW.md`（Plan Packet Rules / Workflow State）
- Function / command / DTO: 変更なし
- DB: 変更なし
- Screen / UI: 変更なし
- Decision log / ADR: D-046（docs 宣言 + interface 強制の系譜）、D-038

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 変更なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 変更なし | existing sufficient |
| DB / transaction / audit / rollback / migration | 変更なし | existing sufficient |
| Screen / UI / route state / Japanese wording | 変更なし | existing sufficient |
| CSV / TSV / report / import / export format | 該当なし | — |
| Durable decision / ADR | UI-13 WER（判断根拠は記録済み、新規 D 不要） | existing sufficient |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-UI13WER | UI-13 WER Change 1 | SPEC-WF-UI13WER-D1 | checklist は template 常設（毎 plan で強制視認）。rejected: memory のみ（Claude 以外の作業者・Codex に届かない）、DEV_WORKFLOW 本文のみ（plan 作成時に開かれない） | `templates/plan-packet.md` | AC の rg token 検査 |
| SPEC-WF-UI13WER | UI-13 WER Change 2 | SPEC-WF-UI13WER-D2 | probe 手順は既存 Contract Probe 規定の拡張 1 文（新概念を作らない）。rejected: 独立節の新設（D-038 系の「既存レンズ拡張」方針に反する） | `DEV_WORKFLOW.md` + template Contract Probe 節 | AC の rg token 検査 |
| SPEC-WF-UI13WER | UI-13 WER Change 3 | SPEC-WF-UI13WER-D3 | materialize 直後の局所検査は手順文言で先行導入し、機械強制（hook 等）は発動データ待ち。rejected: 即 hook 化（sandbox 書込み制約 + D-039 slice 2 の hook 統合 backlog と重複） | `DEV_WORKFLOW.md` Workflow State | AC の rg token 検査 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: WER + 本変更後の DEV_WORKFLOW / template で成立
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: なし（WER 記録済み判断の正本化のみ）
- Assumptions and constraints: checker は template の節構成を token 検査していない（PK 検査対象は Workflow State / Matrix リンク / Findings Freeze 等）ため、節追加は既存 gate を壊さない — L1 full で回帰確認する
- Deferred design gaps, risk, and follow-up target: 機械強制化（hook / checker WARN）は発動実績を見て別 PR
- Test Design Matrix can cite design decision IDs or source doc sections: SPEC-WF-UI13WER-D1〜D3 を cite（Matrix 参照）

## Impact Review Lenses

not applicable — field investigation / 実機 / 外部ツール / POS / フォーマット変更のいずれも起点でない。UI-13 WER の Recommended Workflow Adjustment からの計画的正本化。

## Design Readiness

- Existing design docs are sufficient because: 変更対象は workflow 規範 docs そのもので、根拠は UI-13 WER に記録済み
- Source docs updated in this PR: `DEV_WORKFLOW.md` / `templates/plan-packet.md`
- Design gaps intentionally deferred: 機械強制化
- Durable decisions discovered in this plan and promoted to source docs: なし

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 変更なし
- Backend function design: 変更なし
- Command / DTO / data contract: 変更なし
- Persistence / transaction / audit impact: 変更なし
- Operator workflow / Japanese UI wording: 変更なし
- Error, empty, retry, and recovery behavior: 変更なし
- Testability and traceability IDs: AC の rg token で検査

## Contract Probe

N/A — 外部前提なし（変更対象は自 repo の docs のみ。checker が template 節構成に依存しない事実は L1 full の回帰で確認する）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-WF-UI13WER-D1（checklist template 常設） | `templates/plan-packet.md` 新節 | AC token 検査 ①〜⑨ | non-scope for L3（docs-only） |
| SPEC-WF-UI13WER-D2（probe 是正仮適用 end-to-end） | `DEV_WORKFLOW.md` Contract Probe 規定 + template Contract Probe 節 | AC `rg "是正を仮適用"` 両ファイル hit | non-scope for L3 |
| SPEC-WF-UI13WER-D3（materialize 直後の局所検査） | `DEV_WORKFLOW.md` Workflow State state-only 規則 | AC `rg "check-workflow-git.sh"` | non-scope for L3 |
| 回帰: 既存 gate 挙動・enum・checker token 不変 | 追記のみ（削除・変更なし） | `doc-consistency-check.sh` ERROR 0 + `local-ci.sh full` PASS | non-scope for L3 |

## Test Plan

Test Design Matrix: [test-matrices/2026-07-15-ui13-wer-workflow-docs.md](test-matrices/2026-07-15-ui13-wer-workflow-docs.md)

- targeted tests: AC の rg token 検査（checklist 5 行・9 token + 2 文追記）
- negative tests: なし（docs-only）
- compatibility checks: `doc-consistency-check.sh` ERROR 0、`local-ci.sh full` PASS（checker / gate の回帰）
- data safety checks: 実データなし
- main wiring/integration checks: なし

## Boundary / Wire Contract

該当なし（wire 変更なし）。

## Review Focus

- 追記文言が既存 gate の挙動・enum・checker 期待 token を変えないこと（宣言の追加に留まること）
- checklist（5 行・9 token）が UI-13 Amendment 1〜4 + memory checklist と過不足なく対応すること
- Contract Probe 追記が既存規定の拡張として読めること（新概念化していないこと）

## Spec Contract

Contract ID: SPEC-WF-UI13WER

- template を開いた R2+ plan 作成者が登録・生成義務 checklist（5 行・9 token）を視認でき、該当時に Scope / Ledger への反映を要求される
- Contract Probe の「是正を仮適用した状態で end-to-end」手順が DEV_WORKFLOW 正本と template の両方に存在する
- forward materialize commit 直後の `check-workflow-git.sh` 局所実行が DEV_WORKFLOW の state-only 遷移規則に存在する
- 追記は宣言のみで、既存 gate の挙動・enum・checker 期待 token を変更しない

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-UI13WER | template 節新設 → DEV_WORKFLOW 2 文追記 → 検証 | AC token 検査（Matrix 記載） | 宣言のみ / gate 不変 / 8 項目過不足 | L1 full PASS + hosted dispatch green + 三点 SHA 一致 |

## Data Safety

- 実店舗データ・秘匿情報の混入なし（docs-only）

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- PR: https://github.com/higekiri-w/inventory-system-public/pull/7
- 実装: `templates/plan-packet.md` へ Registration / Generation Obligations 節（義務表 5 行 + Ledger 到達導線標準行）と Contract Probe 是正仮適用 1 文、`DEV_WORKFLOW.md` へ probe 規定 1 bullet + materialize 局所検査 1 bullet。全て追加のみ（削除・変更行 0）
- 新規則の self-dogfood: 本 change 自身の plan-approved materialize 直後に `check-workflow-git.sh` を局所実行し PASS を確認

## Review Response

Fill after review.
- Findings Freeze: not yet frozen; post-freeze exceptions: none.
