# Plan Packet: wave 運用（pipeline + wave 編成）の DEV_WORKFLOW amendment（D-055 候補）

## Workflow State

- Phase: ready-hosted-final
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: b7c29c2
- Amendments: 20d0dad 4a8a899
- Coordinator: Fable
- Writer: Codex（実装。packet 設計は Fable = design board 例外の適用: workflow design-only change、owner 明示指示 = 2026-07-27 wave 運用決定・引き継ぎ書）
- Plan Reviewer: Sonnet（独立 fresh context subagent。rally round 1〜3 実施、round 3 で P1/P2 = 0。owner 判断で Codex 追加 round 可）
- Final Reviewer: Codex review-only subagents（独立 Double Audit、closure 済み）
- Reviewed Content HEAD: 38fe86acf14f847dca93c8b0bda2efefe40f8dba
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: merge のみ pending（owner が後処理を Coordinator へ委任）。plan 承認 + wave 1 選定 = 介入 1/3、Ready 承認 + 後処理委任 = 介入 2/3（いずれも 2026-07-27）

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
workflow gate change（Workflow State の packet selection rule、Owner Effort Budget の介入計上、merge gate 前後の運用、Subagent Budget を改訂し、PK4/PK5 checker の最小改訂を含む）。Risk Tiers の「merge gate changes」に該当し、Double Audit と hosted final が必要。

## Goal

Goal Invariant:

### 最小完了条件

- 監査是正の残 13 単位（順10〜22）を、owner 介入 1 回あたり複数単位分進められる wave 運用の契約が DEV_WORKFLOW に正本化され、wave 1（2 lane pilot）を fail-closed 停止なしに開始できる状態になる。

### 失敗定義

- wave 運用を導入した結果、per-unit の検査の深さ（plan-first、mutation 独立再実測、oracle 独立性、Contract Audit、L3 準備義務）のいずれかが削られる、または複数 active packet の fail-closed 保護が registry 外でも無効化される。

### 非目的

- 検査 gate の削減・簡略化（順9 WER の教訓: mutation 全 red でも P1 は独立 diff review だけが捕捉した。深さは per-lane で全て維持する）
- 複数是正単位の 1 packet への統合（lane = 1 単位 = 1 packet は不変）
- wave 1 の lane 選定そのもの（本 packet 承認後に owner が編成案から選定する）
- 監査残単位の実装着手

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。

## Scope

- `docs/DEV_WORKFLOW.md`: 下記 D-055 契約（D1〜D7）の正本化。対象節 = `Workflow State`（packet selection rule）、`Subagent Budget`、`Owner Effort Budget`、`Draft PR Checkpoint`、`Post-Merge Closeout`、新設の `Wave Operation` 節（他節から link）
- `docs/decision-log.md`: D-055 の新設（決定・理由・棄却代替案・revisit 条件）
- `Plans.md`: `Wave Registry` 節の新設（現 wave の lane 一覧の置き場所。中身の lane 登録は wave 1 編成時）
- `docs/PROJECT_HANDOFF.md`: workflow 変更の同期 1 行
- `docs/AGENT_OPERATING_MANUAL.md`: §4 router 表への wave 運用 1 行（必要最小限）
- `scripts/doc-consistency-check.sh`: PK4 の最小改訂 — 複数 active packet の無条件 ERROR（一意性検査）を「全 active packet が `Plans.md`『次の行動』節内から link されていること」の per-packet 検査へ置換。link のない packet は従来どおり ERROR（fail-closed 維持）
- `scripts/check-workflow-git.sh`: PK5 の最小改訂 — packet に `Rebase Map:` 行がある場合、ancestry 検証を Map 最新の rebase 後 plan-first SHA に対して行う（`Plan Commit` field は不変のまま）
- `scripts/tests/doc-consistency-plan-packet.test.sh`: 常設 regression の追随 — test #11（複数 active packet の旧文言 assert）を新意味論へ書換え（全 link ありの正例 = PASS / link なし 1 packet 混在の負例 = ERROR。T-PK4a/b をこの常設 fixture として実装）
- `scripts/tests/workflow-git-checks.test.sh`: `Rebase Map` の正例・負例 fixture 追加（T-PK5 を常設化。既存の Plan Commit rewrite 検出・Amendments append-only fixture と共存し、それらを弱めない）

## Non-scope

- 監査残単位（順10〜22）の packet 起票・実装（wave 1 編成は本 PR merge 後の別作業）
- Scope に列挙した checker / test file を超える wave 対応機械 check の新設（wave 単位の新 PK check、registry 陳腐化の機械検出等は pilot で摩擦を実測してから判断。D-039 の「checker」語彙は不変）
- Workflow State の phase enum・遷移表・STATECAP・state-backtrack 契約の変更（per-lane で全て不変）
- CI routing（`ci.md`）の変更（hosted 1 change 1 final run は per-lane で不変）

## Acceptance Criteria

- `rg -n "Wave Registry" Plans.md docs/DEV_WORKFLOW.md` が両 file で hit する（registry の定義と置き場所が接続されている）
- `rg -n "D-055" docs/decision-log.md docs/DEV_WORKFLOW.md` が hit し、Matrix の anchor phrase A1〜A8 が全て baseline-red → 実装後 green で実測されている
- `bash scripts/doc-consistency-check.sh` PASS（active plan があるため `--target plan` も PASS）
- synthetic 複数 active packet（全て次の行動節内 link あり）で `bash scripts/doc-consistency-check.sh --target plan` が PASS、link のない packet を 1 つ混ぜると `ERROR` になることを実測し PR body に記録（Matrix T-PK4a/b）
- synthetic rebase 状況で `Rebase Map:` 行ありの packet が `bash scripts/check-workflow-git.sh` の ancestry 検査を通過し、Map なし・旧 SHA 非 ancestor では fail することを実測し PR body に記録（Matrix T-PK5。多段 rebase chain 正例 + gated Amendment SHA 込み rebase の正例・負例を含む — Amendment 1）
- PK4 の link 判定が code fence / comment 内の見かけ上の link を有効 link と誤認せず `bash scripts/doc-consistency-check.sh --target plan` が `ERROR` になる（fail-open 防止）ことを負例 fixture で実測し PR body に記録（Matrix T-PK4c — Amendment 1）
- T-PK4a/b と T-PK5 は ad-hoc 実測に留めず、`scripts/tests/doc-consistency-plan-packet.test.sh`（test #11 書換え含む）と `scripts/tests/workflow-git-checks.test.sh` の常設 fixture として実装し、`run_required` の `doc-consistency-plan-packet-tests` / `workflow-git-checks-tests` が green であることを含む `bash scripts/local-ci.sh full` CLEAN を L1 evidence とする
- Matrix X1/X1b/X1c/X2〜X8 の実 mutation 注入で、対応する `rg` assertion が exit 1 へ反転することを clean tree で実測し、注入 → red → 復元 → green の記録を PR body に残す（X 追加時の range 追随漏れを Amendment 2 で是正）
- 旧文言 grep evidence: `rg -n 'single active packet' docs/ Plans.md AGENTS.md .agents/ .claude/` の live hit（archive 配下の歴史記述を除く）が 0、または読み替え注記の同一 PR 追記で解消済みであることを PR body に記録

## Design Sources

- Requirements / spec: なし（workflow 変更、製品仕様非接触）
- Architecture: なし
- Function / command / DTO: なし
- DB: なし
- Screen / UI: なし
- Decision log / ADR: D-034（Workflow State / Subagent Budget）、D-035（state/evidence 分離）、D-038（Owner Effort Budget / Findings Freeze / STATECAP cap）、D-039（Plan Commit ancestry）、D-050（WER consolidation）。owner 決定の一次記録 = agent memory `project-wave-operation-pilot`（2026-07-27。本 packet が repository 正本化の実施物）
- 設計入力（順10 拡張 scope 精査、2026-07-27 実施）: 残単位の正本 = `docs/research/audit-2026-07/report.md` 優先度付き是正リスト（順10〜22 の 13 単位が残、順1〜9 は PR #14〜#26 で消化済み）。干渉 pair の実読確認 = 順13×順14（`src/lib/bindings.ts` 生成物共有）、順15×順17（`OperationLogsPage.tsx`）、順15×順18（`navigation.ts`）、順15×順21（`ManualSaleRecordDetailPage.tsx`、findings p7/p1 の証拠行で確認）、順10×順18（`53-ui-home.md`）。UI_TECH_STACK.md の弱い重複（順13/14/16/18/19）は着手時に実編集要否を個別確認

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | なし | 該当なし |
| Command / DTO / generated binding / wire shape | なし | 該当なし |
| DB / transaction / audit / rollback / migration | なし | 該当なし |
| Screen / UI / route state / Japanese wording | なし | 該当なし |
| CSV / TSV / report / import / export format | なし | 該当なし |
| Durable decision / ADR | `docs/decision-log.md` D-055 | updated in this PR |

## Registration / Generation Obligations

該当なし（新規 command / route / 画面 / function-design doc なし。新設するのは DEV_WORKFLOW 内の節と decision-log entry で、doc-consistency-check の既存対象内。`Wave Operation` 節は DEV_WORKFLOW 内部の節追加であり親文書の索引義務は Source Index 表の変更なしで満たされる — 節 link は同 file 内の相互参照で行う）

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-WAVE | DEV_WORKFLOW `Wave Operation`（新設） | D-055-D1 | lane/wave 定義と編成入口条件。棄却: 複数単位の 1 packet 統合（レビュー独立性と Findings Freeze の単位が崩れる） | DEV_WORKFLOW 新節 | Matrix X4 / anchor A1,A2 |
| SPEC-WF-WAVE | DEV_WORKFLOW `Workflow State` packet selection rule | D-055-D2 | Wave Registry による意図された複数 active の区別。棄却: fail-closed の全面撤廃（誤 resume 保護を失う）/ checker 非改訂での運用（Contract Probe 2 で不成立を実証） | DEV_WORKFLOW / Plans.md / `doc-consistency-check.sh` PK4 | Matrix X1,X1b,X1c / anchor A3,A4,A4b,A4c / T-PK4a,b |
| SPEC-WF-WAVE | DEV_WORKFLOW `Owner Effort Budget` | D-055-D3 | wave batch 承認の介入計上。棄却: per-change 予算の緩和（3 回/change 構造は owner 決定で維持） | DEV_WORKFLOW | Matrix X3 / anchor A5 |
| SPEC-WF-WAVE | DEV_WORKFLOW `Draft PR Checkpoint` / `Post-Merge Closeout` | D-055-D4 | merge train（並列 Draft・直列 merge・train 先頭のみ ready-hosted-final・Rebase Map で PK5 再充足）。棄却: 全 lane 同時 Ready（rebase ごとに hosted run が増え 1 change 1 final run と衝突）/ `Plan Commit` field の書換（PK5 immutability 違反） | DEV_WORKFLOW / `check-workflow-git.sh` PK5 | Matrix X2,X7 / anchor A6,A7,A11a,A11b / T-PK5,T-PK5b,T-PK5c |
| SPEC-WF-WAVE | DEV_WORKFLOW `Review Rules` 参照 | D-055-D5 | レビュー並列化と裁定直列。棄却: 裁定も並列化（Coordinator の一貫裁定が崩れ相互修正案方式と不整合） | DEV_WORKFLOW 新節から参照 | anchor A8 |
| SPEC-WF-WAVE | DEV_WORKFLOW `Subagent Budget` | D-055-D6 | 全 lane 合算の同時上限。棄却: 無制限（裁定品質と機材保護） | DEV_WORKFLOW | Matrix X5 |
| SPEC-WF-WAVE | decision-log D-055 | D-055-D7 | 2 lane pilot 条項と rollback 条件。棄却: 即 3+ lane 本格化（owner 決定は pilot first） | decision-log | Matrix X6 |

## Design Intent Audit

- Source docs can answer what/why without chat history: D-055 本文に決定・理由・棄却代替案・revisit 条件を記録し、DEV_WORKFLOW 各節は D-055 を参照する
- Plan-only durable decisions found and promoted: 本 packet の D1〜D7 は全て DEV_WORKFLOW / decision-log へ正本化される（packet は証跡のみ）
- Assumptions and constraints: Codex 発注予算は無制限（owner 決定）。owner 介入予算は per-change 3 回を維持。lane 実装 Writer は Codex、レビュー一次は発注書駆動（Opus 5 / Codex）、裁定は Coordinator
- Deferred design gaps: wave 対応の機械 check（PK 系）は pilot 後に判断。3 lane 化は pilot WER 後の owner 判断
- Test Design Matrix cites decision IDs: X1〜X6 が D1〜D7 に対応
- Absolute guarantee / escape hatch self-check: 「registry 外複数 active は従来どおり fail-closed」の例外は Wave Registry 経由のみ。conflict-free rebase の Phase 維持は patch-id 同値の機械証明がある場合のみで、証明できなければ既存規則（content change → implementing 戻り）に落ちる。Rebase Map も同じく patch-id 同値の証明がある conflict-free rebase 専用で、証明なしの Map 追記で PK5 を通す escape hatch にしない（T-PK5 の負例で機械確認）

## Impact Review Lenses

not applicable — field investigation / 実機 / POS / CSV 形式変更を含まない workflow docs 変更のため。lens で見るべき運用リスク（rebase 失敗、budget 超過、fail-closed 誤発火）は D-055-D7 の pilot 条項と Test Matrix で扱う。

## Design Readiness

- Existing design docs are sufficient because: 変更対象は DEV_WORKFLOW 自身。現契約の全文実読（2026-07-27）に基づき、改訂対象文の現行文言を Matrix の anchor に固定済み
- Source docs updated in this PR: DEV_WORKFLOW / decision-log / Plans.md（registry 節）/ PROJECT_HANDOFF / AGENT_OPERATING_MANUAL（1 行）
- Design gaps intentionally deferred: 機械 check 化、3 lane 化条件の精緻化（pilot WER 入力待ち）
- Durable decisions discovered in this plan and promoted: D-055 全体

Minimum design checks: 製品 code 非接触のため layer ownership / DTO / persistence / operator UI / error recovery は該当なし。Testability = docs anchor + checker（下記 Matrix）。

## Contract Probe

- `git patch-id` で conflict-free rebase の内容不変を機械判定できる（D4 の前提）: scratch repo で lane branch の rebase 前後に `git diff main...HEAD | git patch-id --stable` を比較（2026-07-27 実測）-> conflict-free では patch-id 完全一致、競合時は rebase が非 0 exit で停止し検出可。この whole-diff 比較は **Writer evidence 側の command**（PR body 記録）であり、PK5 の機械検査は mapped pair ごとの単一 commit patch-id 同値で行う（証明の 2 層化 = Amendment 1 で契約精緻化）
- `scripts/doc-consistency-check.sh --target plan` が複数 active packet を正しく検査する（D2 の前提）: 本 packet の複製を synthetic 2 つ目の packet として一時配置し実行（2026-07-27 実測、検証後撤去・非 commit）-> **PK4 は active packet が複数の時点で無条件 ERROR**（一意性検査が per-packet link 検査より先に発火し、`Plans.md` の内容は参照されない。Coordinator が checker 実コードの分岐で確認）。当初の probe 記録は出力の中身を確認せず「per-packet link 要求」と誤結論しており、Plan Review round 1 P1-1 の実証で是正した。したがって D2 は checker 変更なしでは機能せず、PK4 の最小改訂（一意性 ERROR → 全 active packet の『次の行動』節内 link 必須、link なしは fail-closed 維持）を Scope に編入する。Wave Registry は『次の行動』節内に置き各 lane の packet link を含める（改訂後 PK4 の検査対象）

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D-055-D1 lane/wave 定義・編成入口条件（footprint 互いに素、生成 file 再生成 lane は wave に 1 つ、同一 source doc 編集 lane の同居禁止） | DEV_WORKFLOW `Wave Operation` 新節 | anchor A1/A2 baseline-red→green、X4 mutation red | non-scope（実運用は wave 1 pilot で dogfood） |
| D-055-D2 Wave Registry と packet selection rule 改訂（registry 記載の複数 active は許可、fail-closed 3 経路〈registry 外 / 不一致 / 陳腐化〉維持、PK4 最小改訂 = 全 active packet の次の行動節内 link 必須） | DEV_WORKFLOW `Workflow State` + Plans.md `Wave Registry` 節 + `scripts/doc-consistency-check.sh` PK4 | anchor A3/A4/A4b/A4c、X1/X1b/X1c mutation red、T-PK4a/b（`doc-consistency-plan-packet.test.sh` test #11 書換えの常設 fixture） | non-scope |
| D-055-D3 wave batch 承認（per-change 介入 ≤3 不変、batch は各 lane に 1 回計上、wave summary 依頼形式） | DEV_WORKFLOW `Owner Effort Budget` | anchor A5、X3 mutation red | non-scope |
| D-055-D4 merge train（Draft 並列・merge 直列・train 先頭のみ ready-hosted-final〈先頭は owner 指定・Coordinator が到達順で提案〉・conflict-free rebase は patch-id 証明で Phase 維持 + L1 full 再実行・Rebase Map で PK5 再充足〈plan-first + 各 Amendment SHA、実効 SHA 検証、証明 2 層 = Amendment 1〉・conflict は implementing 戻り・rebase は Codex） | DEV_WORKFLOW `Draft PR Checkpoint` / `Post-Merge Closeout` + `scripts/check-workflow-git.sh` PK5 | anchor A6/A7/A11a/A11b、X2/X7 mutation red、T-PK5（`workflow-git-checks.test.sh` の常設 fixture、多段 chain + Amendments 込み含む、既存 rewrite 検出・Amendments fixture と共存）、Contract Probe 1 | non-scope |
| D-055-D5 レビュー並列・裁定直列（per-lane 独立 reviewer、mutation 再実測・Findings Freeze・Double Audit 不変） | DEV_WORKFLOW `Wave Operation` 新節（Review Rules 参照） | anchor A8 | non-scope |
| D-055-D6 subagent 合算同時上限 4（per-lane 上限は D-034 表のまま） | DEV_WORKFLOW `Subagent Budget` | X5 mutation red | non-scope |
| D-055-D7 pilot 条項（wave 1 = 2 lane、WER 必須、rollback 条件 = fail-closed / budget 超過の同時多発で単線へ戻す） | decision-log D-055 | X6 mutation red | non-scope |

## Test Plan

Test Design Matrix: [test-matrices/2026-07-27-wave-operation-dev-workflow-amendment.md](test-matrices/2026-07-27-wave-operation-dev-workflow-amendment.md)

- targeted tests: anchor phrase A 系の baseline-red 固定（実装前に `rg` で 0 hit を実測）→ 実装後 green。checker 変更は常設 regression（`scripts/tests/` の 2 suite、`local-ci.sh full` の `run_required`）の fixture 追随・追加で守る
- negative tests: X1〜X6 の実 mutation 注入（clean tree、実装後）で anchor 検査 or checker が red
- compatibility checks: `bash scripts/doc-consistency-check.sh`（full + `--target plan`）、`bash scripts/check-workflow-git.sh`
- data safety checks: 実 POS / 店舗 data 非接触。commit 対象は docs + Scope に列挙した `scripts/` の checker / test file のみ
- main wiring/integration checks: 旧文言（単一 active packet 前提）の repo-wide sweep evidence を PR body に記録

## Boundary / Wire Contract

該当なし（JSON / CSV / DTO / bindings / DB 非接触。docs + checker script の workflow 変更）。

## Review Focus

- D2: Wave Registry が fail-closed 保護を弱めていないか（registry 外の複数 active、registry と packet の不一致、registry 自体の陳腐化の 3 経路で従来どおり停止するか）
- D4: merge train が既存の merge gate（三点一致）・hosted 1 change 1 final run・pre-push Ready guard・STATECAP と矛盾なく接続するか。特に「train 先頭のみ ready-hosted-final」で後続 lane の hosted run が増えないこと、conflict-free rebase の patch-id 証明が escape hatch にならないこと
- D3: batch 承認が「介入予算の実質緩和」に化けていないか（per-lane 計上の定義が曖昧だと予算が形骸化する）
- 検査の深さ per-lane 維持が全 D で明文化されているか（Goal Invariant 失敗定義との突合）
- 既存文言との drift: Workflow State / Review Rules / Contract Audit 節の「change 単位」前提の記述で、wave 化により読み替えが必要な箇所の列挙漏れ

## Spec Contract

Contract ID: SPEC-WF-WAVE-2026-07-27

- D1: wave = file footprint が互いに素な 2〜3 lane の集合。lane = 1 是正単位 = 1 Plan Packet = 1 branch = 1 Draft PR（既存 change 概念の別名であり、per-lane の workflow 契約は一切変更しない）
- D2: `Plans.md` `Wave Registry` に列挙された lane の packet 群のみ、複数 active packet として正当。registry は `Plans.md` の『次の行動』節内に置き、各 lane の packet link を含める。PK4 は最小改訂し、複数 active packet の無条件 ERROR を「全 active packet の『次の行動』節内 link 必須」の per-packet 検査へ置換する（link のない packet は従来どおり ERROR）。resume は registry の lane 単位で packet を選択。fail-closed は 3 経路とも維持し、それぞれを規範文として実装する: ① registry に列挙されていない複数 active packet ② registry と実在 packet の不一致（packet 欠落・branch/PR 不一致）③ registry の陳腐化の疑い（registry が現 wave を反映していない兆候）— いずれも停止して owner 報告
- D3: owner 承認は wave 単位で batch 可能。batch 1 セッションで進めた各 lane に介入 1 回を計上し、per-lane 予算（既定 3 回）は不変。依頼は lane ごとの `介入 N/M + 完了 1 文` を束ねた wave summary 形式。wave summary 内の lane ごとの承認・却下は独立（一部 lane のみの承認が可能）。計上は session 単位でなく decision point 単位: 同一 lane の複数 decision point が 1 session で進んだ場合はその数だけ介入を計上し、batch を予算の実質緩和に使わない
- D4: Draft PR までは lane 並列。ready-hosted-final への遷移は merge train 先頭の lane のみ。train 順序（先頭の選定）は owner が batch Ready 承認時に指定し、既定案として Coordinator が human-confirm 到達順の順序を提案する。先頭 merge 後、次 lane は Codex が rebase し、patch-id 同値を証明できる conflict-free rebase は Phase 維持 + rebase 後 HEAD での L1 full 再実行 + PR body 更新で merge gate を再充足。rebase 後、Writer は packet の append-only 記録に `Rebase Map: <旧SHA> -> <新SHA>` を **plan-first commit と各 gated Amendment SHA のそれぞれについて**追記し、PK5 の ancestry / descendant 検証は Map 適用後の実効 SHA で行う（`Plan Commit` field と `Amendments` 行の原 SHA 列は不変のまま。Map 追記は patch-id 同値の証明がある conflict-free rebase 専用 — Amendment 1）。patch-id 同値の証明は 2 層とする: **機械検査（PK5）**は mapped pair ごとの単一 commit patch-id 同値 + chain 整合を検証し、旧 object を local で解決できない場合は fail-closed。**lane 全体の内容同値**（rebase 前後の `git diff <base>...<head> | git patch-id --stable` 一致）は Writer evidence として PR body に記録する。conflict が出たら content change として implementing へ戻る。owner の train 承認 1 回で train 全 lane の Ready 遷移実行を Coordinator へ委任できる
- D5: Plan Reviewer / Final Reviewer は lane ごとに独立 fresh context。一次レビューは並列可、裁定は Coordinator 直列。mutation 独立再実測・oracle 独立性・Findings Freeze・Double Audit・L3 準備義務は per-lane 不変
- D6: subagent は per-lane 上限（D-034 表）に加え、全 lane 合算同時 4 を上限とする
- D7: wave 1 は 2 lane pilot。完了時 WER で摩擦を実測記録し、3 lane 化は owner 判断。複数 lane で fail-closed 停止 / budget 超過が同時発生したら wave を中断し単線運用へ戻す。複数 lane の L3 を 1 session に束ねるかは wave 1 dogfood で決めて WER に記録する（L3 fixture 準備義務は per-lane 維持）

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-WAVE D1 | DEV_WORKFLOW 新節執筆 | A1/A2, X4 | 編成入口条件の抜け | Matrix 実測記録 |
| SPEC-WF-WAVE D2 | Workflow State 改訂 + Plans.md registry + PK4 最小改訂 | A3/A4/A4b/A4c, X1/X1b/X1c, T-PK4a/b, Probe 2 | fail-closed 3 経路 | Matrix 実測記録 |
| SPEC-WF-WAVE D3 | Owner Effort Budget 改訂 | A5, X3 | 予算形骸化 | Matrix 実測記録 |
| SPEC-WF-WAVE D4 | Draft PR Checkpoint / Post-Merge Closeout 改訂 + PK5 Rebase Map 対応 | A6/A7/A11a/A11b, X2/X7, T-PK5/T-PK5b/T-PK5c, Probe 1 | 三点一致 / hosted 1 run / PK5 整合 | Matrix 実測記録 |
| SPEC-WF-WAVE D5 | 新節（Review Rules 参照） | A8 | 深さ維持の明文化 | Matrix 実測記録 |
| SPEC-WF-WAVE D6 | Subagent Budget 改訂 | X5 | 上限の実効性 | Matrix 実測記録 |
| SPEC-WF-WAVE D7 | decision-log D-055 | X6 | rollback 条件の実行可能性 | Matrix 実測記録 |

## Data Safety

- 実 POS CSV / 店舗 data / DB file / backup / log / secret は commit しない（本 change は docs + Scope に列挙した checker / test script のみ）
- local-only paths: `.local/ci-evidence/`（L1 証跡、非 commit）
- synthetic-only paths: Contract Probe 2 の synthetic packet は検証後に撤去し commit しない

## Implementation Results

D-055 と gated Amendment 1 を docs、PK4 / PK5 checker、常設 fixture に反映し、anchor / guard、mutation、L1 full、独立 Double Audit を完了した。Draft PR: https://github.com/higekiri-w/inventory-system-public/pull/27

Exact-HEAD SHA と test count は D-035/D-038 に従い PR body を正本とする。

## Review Response

- Findings Freeze: 2026-07-27 final Double Audit closure 時点; post-freeze exceptions: none.

**Plan Review round 1（2026-07-27、独立 Plan Reviewer = Sonnet fresh context）**

- 結果: P1×3 / P2×1 / P3×3、全件 accept（相互修正案方式、Coordinator が P1 を実コードで再現確認の上裁定）
- P1-1: PK4 は複数 active packet で無条件 ERROR となり Wave Registry が機能しない（reviewer が synthetic 複製で実証、Coordinator が checker の一意性分岐を実読確認）。当初 Contract Probe 2 の「per-packet link 要求」は出力内容を確認しない誤結論だった → Probe 2 記録を訂正し、PK4 最小改訂を Scope へ編入
- P1-2: D2 の fail-closed 3 経路のうち anchor/mutation があるのは 1 経路のみ → A4b/A4c + X1b/X1c を Matrix へ追加
- P1-3: conflict-free rebase は plan-first commit の SHA を変え PK5 ancestry を機械的に破る → D4 に `Rebase Map` 契約（`Plan Commit` field 不変の append-only 記録、patch-id 同値証明専用）+ `check-workflow-git.sh` PK5 最小改訂を Scope へ編入、anchor A11 + X7 + T-PK5 を追加
- P2-1: batch 承認の粒度が未定義で予算形骸化リスク → D3 に「lane ごとの承認・却下は独立」「decision point 単位で計上」を明文化、anchor A5b + X8 を追加
- P3-1: M-N2 guard が file 全域対象で誤爆リスク → 遷移表の行形へ regex を絞り、新規表 cell で `→` を使わない実装注記を追加
- P3-2: train 先頭の選定規則が未記載 → D4 に owner 指定 + Coordinator 到達順提案を明文化
- P3-3: 複数 lane の L3 同席方式が未記載 → D7 に wave 1 dogfood で決定・WER 記録を明文化

**Plan Review round 2（2026-07-27、同 reviewer による closure 確認）**

- 結果: round 1 の 7 件は全件解消確認。新規 P1×1 = PK4/PK5 の常設 regression suite（`scripts/tests/doc-consistency-plan-packet.test.sh` test #11 が旧・無条件 ERROR 文言を assert / `workflow-git-checks.test.sh` の PK5 rewrite 検出網）が Scope から漏れており、実装すると `local-ci.sh full` の `run_required` gate が red になる — Coordinator が `local-ci.sh` の登録行と test #11 fixture を実読して再現性を確認、accept
- 是正: 両 test file を Scope へ編入（test #11 の新意味論書換え + Rebase Map fixture 追加、既存 rewrite 検出・Amendments fixture と共存）、T-PK4/T-PK5 を ad-hoc 実測から常設 fixture 化へ変更、AC に `local-ci.sh full` CLEAN（両 run_required green 含む）を明記

**Plan Review round 3（2026-07-27、同 reviewer による closure 確認）**

- 結果: round 2 P1 の解消確認（test #11 の既存 primitive で新 fixture が実装可能なことまで実 file で検証）、**P1/P2 = 0、plan-gate 通過可**。新規 P3×2 = Scope 拡大後の数値 echo 陳腐化（Non-scope「2 点」/ Data Safety「2 file」）→ 同日修正（数値でなく Scope 列挙への参照に置換、数値 echo の rg sweep 済み）

**遷移記録（2026-07-27、state-only）**

- owner が D-055 plan を承認し wave 1 lane = 順17 × 順22 を選定（この change での介入 1 回目/予算 3 回）。既存 evidence（plan-first commit `b7c29c2` が全実装 commit に先行 = 実装 commit 未着手 / 独立 Plan Reviewer round 3 で P1/P2 = 0）により `plan-gate -> plan-approved -> implementing` を本 state-only commit で一括実体化。実装 Writer = Codex（発注書は Coordinator が提示、owner relay で起動）

**gated Amendment 1（2026-07-27、Codex fail-closed 停止の Coordinator 裁定）**

- Codex が実装 commit `dd3bc22` 到達後の自己 Double Audit で P2 相当 4 件を検出し fail-closed 停止（push / state 遷移 / Draft PR 未実施の正しい停止）。Coordinator が 4 件全てを実コード・Matrix 実読で再現確認し accept:
  1. Amendments ancestry: PK5 の Amendments 検査が「HEAD の祖先」を要求するため、gated Amendment を持つ lane は正当な conflict-free rebase + Rebase Map でも必ず fail → **Rebase Map の対応対象を plan-first commit + 各 Amendment SHA へ拡張**し、検証を Map 適用後の実効 SHA で行う契約に精緻化（`Amendments` 行の原 SHA 列は不変）
  2. patch-id 証明範囲: 実装は plan-first 単一 commit の patch-id chain のみで、Contract Probe 1 の whole-diff 同値と証明対象が乖離 → **証明の 2 層化**（機械検査 = mapped pair 単一 commit patch-id + fail-closed / lane 全体同値 = Writer evidence として PR body）を契約化
  3. Matrix M-A11 oracle 欠陥（Coordinator の authoring ミス）: combined `rg` が checker 側 hit で X7 を検出不能 → M-A11a（DEV_WORKFLOW）/ M-A11b（checker）へ分割、X7 は M-A11a で検出
  4. M-N5 期待表記誤り（期待 1 → 正 0/UNCHANGED 出力）+ PK4 link 判定の fail-open 防止負例（T-PK4c）+ 多段 rebase chain / Amendments 込み fixture の追加
- 本 Amendment は packet / Matrix の契約精緻化のみ。実装への反映（checker / test / DEV_WORKFLOW 文言の追随）は Codex 再開 scope

**gated Amendment 2（2026-07-27、Coordinator 独立再実測による X7 oracle 欠陥の是正）**

- human-confirm 到達後、Coordinator が独立 fresh context（worktree 隔離）で X1〜X8 全 10 mutation を Matrix どおり実注入再実測。結果 9/10 red 実証、**X7 のみ FAIL**: DEV_WORKFLOW の Rebase Map 定義文を削除しても Draft PR Checkpoint 節の cross-reference hit（`Rebase Map` は Wave Operation に従う）が残るため、汎用 literal の M-A11a が反転しない。Writer の「X7 red 実証」記録は Matrix の注入内容どおりには再現不能（PR body の該当 evidence は訂正対象）
- Findings Freeze 後の新規発見だが、実再実測の failure で立証されているため blocker として accept。是正 = M-A11a assertion を定義文 literal（`Rebase Map: <旧 SHA> -> <新 SHA>`）へ特定化（X7 の注入内容は不変、oracle 側の弁別性を回復）。DEV_WORKFLOW / checker / test の実装は無変更（oracle 定義のみの是正）
- 併せて AC の mutation range「X1〜X6」が X7/X8 追加時に未追随だった stale 表記を是正
- 手順 = state-backtrack `human-confirm -> implementing`（`adc5bc6`）→ 本 Amendment content commit → Amendments 行追記 → Coordinator が新 M-A11a で X7 を再実測 → L1 full 再実行 → 再 walk（evidence 充足後、Ready 承認時に adjacent forward 一括実体化。forward state-only は cap 3 の残 1 枠に収める）

**遷移記録（2026-07-27、state-only、再 walk 一括実体化）**

- owner が Ready 化を承認し後処理（Ready / hosted final / merge / closeout）を Coordinator へ委任（この change での介入 2 回目 / 予算 3 回）。既存 evidence により `implementing -> local-verified -> independent-review -> human-confirm -> ready-hosted-final` を本 state-only commit（forward state-only 3/3）で一括実体化:
  - implementing -> local-verified: content candidate = Reviewed Content HEAD 記載 SHA、L1 full PASS / CLEAN / MERGE_EVIDENCE_VALID（evidence は PR body と `.local/ci-evidence/`）
  - local-verified -> independent-review -> human-confirm: Double Audit closure（P1/P2 = 0、ac12745 時点）+ Amendment 2 の delta（Matrix oracle 是正 + Plans.md 同期のみ、実装無変更）は Coordinator の閉包確認 = X7 含む全 10 mutation の独立再実測 red で閉じる（Findings Freeze 下の closure confirmation）
  - human-confirm -> ready-hosted-final: owner Ready 承認（本記録冒頭）。本 commit 後の resulting HEAD で L1 full を再実行し PR body を全面 refresh する

**Final Double Audit と遷移記録（2026-07-27、state-only）**

- 独立 reviewer 2 pass で D1〜D7、gated Amendment 1、PK4 / PK5 と常設 fixture を突合した。code fence / comment / code span の pseudo-link、Rebase Map root 間流入、chain 負例の感度に関する findings は受理して実装・回帰 fixture を追加し、closure で P1/P2 = 0 を確認した。
- packet / Matrix の元 Scope・Spec Contract を gated Amendment 1 より前の履歴として保持する指摘は、append-only 契約と正本優先順位に従い非採用とした。追加 amendment を要する契約変更はない。
- L1 full、anchor / guard、全 mutation の独立再実測、Draft PR 作成が完了したため、`local-verified -> independent-review -> human-confirm` を一括実体化する。Ready 化と hosted final は owner の Human Gate 待ち。
