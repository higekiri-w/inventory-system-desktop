# Plan Packet: 普通の一日の操作列・Writer 停止時の規則・問い合わせの行き先（workflow の軽量化 1 段目の文書部分、R3）

2026-09-22 起草。出典は [Backlog](../../backlog.md#やると決めたもの順番未定)「workflow の軽量化 3 段」（㉗ PR #85 の dogfood 所見起源、owner disposition 2026-09-22）と、`docs/Plans.md`「次の行動」1（owner 決定 2026-09-22）。1 段目「発注の構造化と発注前検査」のうち、script・schema を伴わない文書だけの部分を先に入れる。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Evidence Mode: github
- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 74085e4d4441ed594f944447087355ac716861ff
- Amendments: none
- Coordinator: Fable 5.1
- Writer: Sonnet subagent（worktree run）
- Plan Reviewer: Opus（独立 fresh context）
- Final Reviewer: Codex + Opus（互いに独立、Plan Reviewer とも別の fresh context、Double Audit）
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: 製品 runtime・画面・配布物への変化がない文書変更で、Windows native の確認対象がない。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（本 commit、plan-first）: 対象は workflow 文書 3 本と decision-log。規則の所有先は既存正本で決まっている（Plan Review は `docs/DEV_WORKFLOW.md` Review Rules、Writer 発注は `docs/AGENT_OPERATING_MANUAL.md` §5.6、owner 負担は `docs/DEV_WORKFLOW.md` Owner Effort Budget）。owner の設計判断を要する未決の論点は起票時点でなし。PR #88 の dogfood 所見（計画と実装を同じ vendor が見て packet の前提誤りが Plan Gate を通過した）に対し、本 packet の Plan Reviewer（Opus）は Writer（Sonnet）と別 model だが同じ vendor であり、Plan Gate の時点では所見の条件を解消しない。別 vendor の目は Final Review の Codex が担い、Plan Gate では同 vendor・別 model の残余 risk を Coordinator の判断で受容して進め、owner へは 2026-09-22 の進捗報告で提示した（Codex への差替えは owner の指示があれば行う。owner relay を Plan Review に使わない判断。`docs/DEV_WORKFLOW.md` Review Rules の vendor 条項は Writer が Codex の packet が対象で、本 packet には literal に掛からない）
- plan-gate → plan-approved → implementing（本 commit、state-only）: Plan Review round 1（Opus、P1 0 / P2 5 / P3 4、操作列は `成立`）→ 全件採用し是正 `649b1f5b` → round 2 closure（同 reviewer、P1/P2 = 0、P3 2）→ P3 を反映。Plan Commit = `74085e4d`（plan-first `2c7139c6` → 是正を含む確定版）。実装は Sonnet subagent の worktree run で本 commit を起点にする
- implementing → archive（closeout、本 commit）: PR #90 squash merge `89f0bc2d`（2026-09-22）。packet / Matrix を `docs/archive/plans/` へ移送し、Implementation Results / Review Response を記録した。broad 後の是正で実装とずれていた記述を closeout で訂正: Spec Contract D4 (a) と AC4 の「複数の行に当てはまる場合の優先の 1 文」を `docs/DEV_WORKFLOW.md` の実装（3 文）に合わせ、Matrix の Negative Paths の duplicate/ambiguous input を同じ 3 文に合わせ、Mutation-style Adequacy Questions の「除くと不成立になる → なる」という言い過ぎを pass A（A-2）の判定表に沿って訂正した（いずれも closure P3-2）。closure P3-3（非遡及の句が Plan Packet Rules 側〈`:69`〉だけにあり Review Rules 側〈`:391`〉に carve-out が無い）は Matrix の Residual Test Gaps に記録し、節を持たない旧 packet では発火せず実害が無いこと・policy 文書の変更を要することから見送った

## Owner Effort Budget

- 介入回数上限: 3（内訳の見込み: Codex の Final Review relay 1、Ready 1、merge 1）
- 実働時間上限: 15分（既定 30 分から引下げ。文書だけの変更で、owner の作業は Codex relay 1 回と Ready / merge の判断に限られる見込みのため）
- relay 往復上限: 2
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers「迷ったら workflow gate に触れる場合は R3」と `docs/project-profile.md` High-risk Changes を当てた。本変更は機械 checker・phase・review 本数・Human Gate を変えないが、(a) Plan Review が合否を判断するときに答える問いを 1 つ足し、(b) Writer 停止時に owner への中継を省ける条件を定める。(a) は Plan Gate の判断内容、(b) は承認の行き先に触れるため、file の種類ではなく影響で R3 とする。`scripts/ci/classify-changes.sh` は `docs/DEV_WORKFLOW.md` / `docs/AGENT_OPERATING_MANUAL.md` / `docs/templates/*` を workflow 分類にし、helper は Final Review Minimum 2 を要求する。

## Goal

Goal Invariant:

### 最小完了条件

- 次の R2+ 起票（最初の適用先は ㉘ runtime の最初の lane）で、Coordinator が template の節に普通の一日の操作列を書け、Plan Reviewer が既存の Plan Review の中で「その操作列で目的を達成できるか」を最初に答える。
- Writer が編集前に止まったとき、Coordinator が「発注だけの誤りか、正本の変更が要るか」を文書で判定でき、前者は owner へ中継せずに直せる。
- owner へ届く問い合わせが、owner にしか決められないもの（目的・受容リスク・店の事実・Human Gate）に絞られる行き先が 1 つの表で分かる。

### 失敗定義

- 新しい phase・review 本数・常設 gate・承認 commit が増える。
- 発注の訂正という名目で Scope / AC / 権限 / review 数 / finding の採否が変えられる、または実行 mode 上 owner が持つ裁定・承認が Coordinator へ移る。
- 注意書きが増えるだけで、template の節・Plan Review の問い・判定できる規則文のどれにも届かない。

### 非目的

- 発注 schema（WriterOrderV1）、発注前検査の script、packet の機械可読部分、classifier / checker の変更（1 段目の残り = 別 change）。
- 作業群（Goal Group）単位の owner 負担の通算、M1〜M8 の測定規則（外部設計相談の案にあるが `docs/Plans.md` の owner 決定の 3 点に含まれない）。
- 2 段目（Amendment の影響別扱い）、3 段目（重複文書の統合）。
- 既存の active / archive packet への遡及適用。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

本 packet 自身は現行 template（本節なし）で Plan Review を受ける。下表は Plan Reviewer への参考情報であり、本変更が定める規則を本 PR 自身の review の省略や合否に先取りして使わない。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 設計を含む R2+ を起票する | Coordinator が template の `Ordinary Operation` に操作列を書き、Plan Review を発注する | reviewer の報告の冒頭に `成立 / 具体的な反例あり / 外部前提が未確認` のどれかが出る | `成立`、または反例・未確認の前提を packet へ反映して既存の Plan Review の採否を通る | reviewer が冒頭で答えるかは次の起票で観測（未実測） |
| plan-approved、Writer へ発注済み | Writer が発注と正本の不一致を見つけ、編集前に不一致箇所・正本の参照・現在 HEAD を返す | Coordinator が正本は一意で発注だけが誤りと判定し、発注を作り直して §5.6 の最終照合を行う | Scope / AC / commit 条件 / 権限 / phase / Risk / review 数が不変。owner への中継なしで Writer が再開する | なし |
| 同上 | Writer が返した不一致が、正本の意味を変えないと解消しない（例: PR #85 の「合成モデルは変更しない」とモデル是正要求の衝突） | 発注の訂正では進めず、既存の独立確認と Gated Amendment へ戻る | 実行 mode 上の裁定者が Amendment を採用する | なし |
| 作業中に owner へ聞きたいことが出る | Coordinator が行き先の表で分類する | 技術的前提は agent の調査・probe へ、店の事実と Human Gate だけが owner へ届く | 表の行に当てはまる | 表に当てはまらない問い合わせの頻度は未実測 |

## Scope

本 packet と Matrix の `D1`〜`D5` は、子 ID `SPEC-WF-LIGHT1A-D1`〜`SPEC-WF-LIGHT1A-D5` の略記である（`docs/decision-log.md` の `D-090` とは別の番号体系）。

- S1 `docs/templates/plan-packet.md`: `## Goal` の直後、`## Scope` の直前に `## Ordinary Operation` を追加する（Spec Contract D1）。既存の節・field・文言は削除も変更もしない。
- S2 `docs/DEV_WORKFLOW.md`:
  - Review Rules に、Plan Review の最初に普通の一日の操作列の成立を答える規則を 1 項目追加する（D2）。
  - Plan Packet Rules に、`Ordinary Operation` 節の位置づけを 1 項目で示し Review Rules の規則へ link する（D2）。
  - Owner Effort Budget に `### 問い合わせの行き先` の表を追加する（D4）。
- S3 `docs/AGENT_OPERATING_MANUAL.md` §5.6: `**Writer が編集前に止まったとき**` で始まる規則文を追加する（D3）。行き先の表は複製せず `docs/DEV_WORKFLOW.md` の該当節へ link する。
- S4 `docs/decision-log.md`: `## D-090` を末尾へ追加する（D5）。
- S5 本 packet・Matrix・`docs/Plans.md`: 計画、現在地、次の行動を同期する（Coordinator が担当。Writer は編集しない）。

対象を使う側の確認（起票時、main `8fa12a5c`）: template を読む script は `scripts/tests/doc-consistency-plan-packet.test.sh`（`介入 N` / `予算 M` の文字列の存在だけを assert）と `scripts/tests/classify-changes.test.sh`（path 名だけ）。`scripts/doc-consistency-check.sh` は template の節構成を検査しない。どちらも節の追加で結果が変わらない。

## Non-scope

- 上記以外の tracked file。特に `scripts/**`、`.github/**`、`src/**`、`src-tauri/**`、`AGENTS.md`、`CLAUDE.md`、Skills、`docs/ci.md`、`docs/agent-guidance/**`。
- `docs/AGENT_OPERATING_MANUAL.md` §5.4（低制約 profile）と `docs/templates/subagent-review-packet.md`: reviewer への読む順序・観点の強制を足さない。Plan Review の発注に何を参照させるかは Review Rules の 1 項目だけが所有する。
- `docs/templates/test-design-matrix.md`。
- `Ordinary Operation` の有無を検査する checker rule（PK 系）の新設。
- 非目的に挙げた項目すべて。

## Acceptance Criteria

baseline は main `8fa12a5c` の worktree で同じ command を実行した実測。

- AC1: `rg -n '^## (Goal|Ordinary Operation|Scope)$' docs/templates/plan-packet.md` が `Goal` → `Ordinary Operation` → `Scope` の順に 3 行を出す（baseline: `48:## Goal` / `66:## Scope` の 2 行）。節は D1 の 5 列の表、適用対象、`not applicable` の扱い、「文書の完了」と「通常運用の達成」を分ける指示を含む。`git diff origin/main --numstat -- docs/templates/plan-packet.md` の削除行数が 0。
- AC2: `rg -n '外部前提が未確認' docs/DEV_WORKFLOW.md` が Review Rules 内の 1 項目に一致する（baseline: 一致なし、exit 1）。その項目は D2 の全要素を含み、新しい phase・review 本数・常設 gate・承認 commit を足さないと明記する。Plan Packet Rules の項目はこの規則への link を持つ。
- AC3: `rg -n '編集前に止まったとき' docs/AGENT_OPERATING_MANUAL.md` が §5.6 内に一致する（baseline: 一致なし、exit 1）。規則文は D3 の全要素を含み、§5.6 の既存文「食い違いは Coordinator が正本と発注書を訂正し」との関係（D3 の 8）を示し、PR #85 の衝突を「発注の訂正として通してはならない例」として挙げる。
- AC4: `rg -n '問い合わせの行き先' docs --glob '!docs/archive/**' --glob '!docs/Plans.md' --glob '!docs/plans/**'` が `docs/DEV_WORKFLOW.md`（見出しと表）と、`docs/AGENT_OPERATING_MANUAL.md` / `docs/decision-log.md`（link または言及のみ）に一致する（baseline: 一致なし、exit 1）。`rg -n '^### 問い合わせの行き先$' docs/DEV_WORKFLOW.md` が 1 行に一致し、参照側の link はすべて `DEV_WORKFLOW.md#問い合わせの行き先` を指す（doc check は anchor を検査しないため、この突合で確認する）。表は D4 の 7 行と、複数の行に当てはまり owner 側の行を含む場合の優先・裁定権の所在が争点の場合の扱い・owner 側でない行だけに当てはまる場合の扱いを分けて示す記述を持ち（closeout で訂正、closure P3-2）、`docs/DEV_WORKFLOW.md` 以外に表の複製がない。
- AC5: gate・役割・権限が不変。`git diff origin/main --stat -- scripts .github src src-tauri AGENTS.md CLAUDE.md docs/ci.md` が空。template の `Workflow State` field・`Human Gate`・`Final Review Minimum` の行に差分なし（AC1 の削除 0 行で観測）。
- AC6: `rg -n '^## D-090' docs/decision-log.md` が 1 行に一致する（baseline: 一致なし、exit 1）。entry は Status / Decision / Why / Compatibility を持つ（D5）。
- AC7: `git diff --check`、`bash scripts/doc-consistency-check.sh`、`bash scripts/doc-consistency-check.sh --target plan`、`bash scripts/local-ci.sh changed` が成功する。Test Plan の事例照合 T1〜T7 が成立する。未解決の P1 / P2 なし。

## Design Sources

- Requirements / spec: 該当なし（製品要件を変えない）。
- Architecture: 該当なし。
- Function / command / DTO: 該当なし。
- DB: 該当なし。
- Screen / UI: 該当なし。
- Workflow: `AGENTS.md` Decision and Approval Boundaries。`docs/DEV_WORKFLOW.md` Plan Packet Rules / Design Phase Rules / Owner Effort Budget / Review Rules。`docs/AGENT_OPERATING_MANUAL.md` §3 / §3.2（codex-only の自己裁定の禁止、D-087）/ §5.4 / §5.6。`docs/agent-guidance/merge-evidence.md`（Gated Amendment と broad の関係）。
- Decision log / ADR: D-038（Owner Effort Budget、Findings Freeze）、D-055（decision point 計上）、D-062（実測・Plan Reviewer の vendor 独立）、D-084（codex-only の裁定は owner）、D-087。
- 根拠となる実績: `docs/Plans.md`「直近の完了」の PR #85 / #88 / #75 の dogfood 所見。§5.6 の現行の発注手順の出典は PR #82（`docs/archive/plans/2026-09-17-writer-dispatch-docs.md`）。外部設計相談（2026-09-22、owner 実施）の回答は local-only で公開 repository に置かないため、必要な内容は本 packet の Spec Contract に書き下した。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 該当なし | existing sufficient |
| Command / DTO / generated binding / wire shape | 該当なし | existing sufficient |
| DB / transaction / audit / rollback / migration | 該当なし | existing sufficient |
| Screen / UI / route state / Japanese wording | 該当なし | existing sufficient |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Workflow 規則（本変更の対象そのもの） | `docs/DEV_WORKFLOW.md` / `docs/AGENT_OPERATING_MANUAL.md` §5.6 / `docs/templates/plan-packet.md` | updated in this PR |
| Durable decision / ADR | `docs/decision-log.md` D-090 | updated in this PR |

## Registration / Generation Obligations

- source / workflow doc の新設・改名・削除なし。既存文書への節・項目の追加のみ。`docs/DEV_WORKFLOW.md` に目次はなく、新設の `### 問い合わせの行き先` は S3 / S4 からの link 先として anchor を使う。`scripts/doc-consistency-check.sh` の link 検査は anchor 部を捨てて file の実在だけを見るため、anchor の一致は AC4 の command と review で確認する。
- D-n の採番: main `8fa12a5c` の最新は D-089。並走・stacked の lane はなく D-090 を使う。merge 前に main が進んで D-090 が埋まった場合は Review Rules「連番契約 registry の採番を後続 lane が確定する」に従い採番し直す。
- Tauri command / function-design doc / REQ / route / operator 画面: 該当なし。bindings / route tree / traceability の再生成は対象外。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-LIGHT1A | `docs/templates/plan-packet.md` Ordinary Operation | D1 | PR #85 は「誤った判定を通さないか」だけを 3 round 問い、「普通の一日で目的を達成できるか」を最初に問わなかった。却下: 新しい design review phase の新設（gate と owner 負担が増える） | S1 | T1 / T7 |
| SPEC-WF-LIGHT1A | `docs/DEV_WORKFLOW.md` Review Rules / Plan Packet Rules | D2 | 既存の Plan Review の同じ run・同じ採否に含めれば費用が増えない。却下: §5.4 へ読む順序を足す（低制約 profile の趣旨に反する） | S2 | T1 / T5 |
| SPEC-WF-LIGHT1A | `docs/AGENT_OPERATING_MANUAL.md` §5.6 | D3 | Writer の編集前停止は PR #85 / #75 / #70 / #71 等で大半が Coordinator の発注誤りだった。owner を伝言役にしない。却下: Writer が自分で発注を読み替える（fail-closed を失う） | S3 | T2 / T3 / T4 |
| SPEC-WF-LIGHT1A | `docs/DEV_WORKFLOW.md` Owner Effort Budget | D4 | 上限値だけでは owner 負担が減らず、問い合わせの行き先を変える必要がある。却下: 上限値の引上げ | S2 | T3 / T4 / T6 |
| SPEC-WF-LIGHT1A | `docs/decision-log.md` D-090 | D5 | 規則の理由と両立条件を chat 履歴なしで辿れるようにする | S4 | T6 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 規則は S1〜S3 の正本、理由と両立条件は D-090 に置く。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D-090。packet だけに残る durable な決定はない。
- Assumptions and constraints: 外部設計相談の案は下書きであり、owner 承認済みの規則ではない。採るのは `docs/Plans.md` の owner 決定に含まれる 3 点だけ。案の「Coordinator は訂正後に発注前検査を行う」は、発注前検査の script が未導入のため §5.6 作成・訂正の手順 4 の最終照合へ読み替える。
- Deferred design gaps, risk, and follow-up target: 1 段目の残り（schema・script・発注前検査）、作業群通算の予算、測定規則は backlog「workflow の軽量化 3 段」に残る。
- Test Design Matrix can cite design decision IDs or source doc sections: D1〜D5 を引用する。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「owner への中継を要しない」が唯一の escape hatch。成立条件（正本が一意、Scope / AC / commit 条件 / 権限 / phase / Risk / review 数が不変、finding の採否を含まない）と、適用しない場合（正本が曖昧、正本の意味を変える、実行 mode 上 owner が持つ裁定）を D3 に明記した。codex-only（D-084）では Coordinator の起草役が Codex、裁定者が owner であり、D3 は裁定を移さない。D-087 とも両立する（Astra 主担当でも owner の採用・裁定・Human Gate は維持）。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable: 製品 code なし | なし |
| Fact check / design decision split | 適用。効果（停止・往復・owner 負担の減少）は未実測で、規則文に効果の数値を書かない | closeout の dogfood 所見 |
| Lifecycle / retry | 適用。Writer の停止 → 発注の作り直し → 再開、の再開時に §5.6 手順 3 の「再開」の扱いを使う | D3 |
| Operator workflow | not applicable: 店舗 operator の操作は変わらない。ここでの「利用者」は Coordinator / Writer / reviewer / owner | なし |
| Replacement path | not applicable | なし |
| Data safety / evidence | 適用。外部設計相談の原文は local-only のまま、packet・PR へ転記しない | Data Safety |
| Reporting / accounting semantics | not applicable | なし |
| Manual verification | not applicable: 実機確認の対象なし | なし |
| 環境・再現性 | not applicable: 新しい環境依存なし | なし |

## Design Readiness

- Existing design docs are sufficient because: 規則の所有先（Review Rules / §5.6 / Owner Effort Budget / template）は既存正本で決まっており、本変更はその所有先へ規則を足す。
- Source docs updated in this PR: S1〜S4。
- Design gaps intentionally deferred: 非目的の項目。
- Durable decisions discovered in this plan and promoted to source docs: D-090。

Minimum design checks for business-app work: 製品の layer / command / DB / operator workflow / error 挙動に変更なし。Testability は Test Plan の事例照合と AC の command で担保する。

## Contract Probe

- 前提「template への節の追加で既存の checker / test の結果が変わらない」: main `8fa12a5c` で `rg -n 'templates/plan-packet' scripts` → `scripts/tests/doc-consistency-plan-packet.test.sh:824-825`（`介入 N` / `予算 M` の存在 assert）と `scripts/tests/classify-changes.test.sh:46`（path 名）のみ。`rg -n 'Ordinary|required_heading' scripts/doc-consistency-check.sh` → 一致なし。節の追加は結果を変えない。実装後は AC7 の gate で再確認する。
- 未確認の外部前提（library / OS / hardware）: N/A。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D1 template の Ordinary Operation 節 | `docs/templates/plan-packet.md` | AC1 の command、doc check | 節の有無を検査する checker は non-scope |
| D2 Plan Review の最初の問い | `docs/DEV_WORKFLOW.md` Review Rules / Plan Packet Rules | AC2 の command。link 先 file の実在は doc check、anchor は review | reviewer が実際に冒頭で答えるかは次の起票で観測 |
| D3 Writer 停止時の規則 | `docs/AGENT_OPERATING_MANUAL.md` §5.6 | AC3 の command | 事例照合 T2〜T4 は review で確認 |
| D4 問い合わせの行き先 | `docs/DEV_WORKFLOW.md` Owner Effort Budget | AC4 の command（見出しと参照側の anchor 文字列の突合を含む）| なし |
| D5 D-090 | `docs/decision-log.md` | AC6 の command | なし |
| 隣接契約: AGENTS Decision and Approval Boundaries（owner 承認の範囲） | 変更しない | AC5 | D3 / D4 が弱めないことを Review Focus で確認 |
| 隣接契約: §3.2 codex-only の自己裁定の禁止、D-087 | 変更しない | AC5（file は同じだが §3.2 に差分なし: `git diff origin/main -- docs/AGENT_OPERATING_MANUAL.md` の hunk が §5.6 のみ） | なし |
| 隣接契約: Plan Packet Rules の旧前提 sweep、§5.6 手順 1〜4 | 変更しない（D3 から参照） | 同上 | なし |
| 隣接契約: §5.6 の既存文「食い違いは Coordinator が正本と発注書を訂正し…」「正本の変更が必要なら既存の改訂・承認経路を使う」 | 既存文は変更しない。D3 の 8 が関係を示し、正本の訂正を Gated Amendment の経路へ向ける | AC3 | 2 つの文が D3 の 5 を迂回する根拠として読めないことを Review Focus で確認 |
| 隣接契約: Review Rules「連番契約 registry の採番を後続 lane が確定する」 | 変更しない。S4 の D-090 採番が行使する | AC6 | Registration / Generation Obligations の採番の項 |
| 隣接契約: Owner Effort Budget の上限値・上限到達時の手順・decision point 計上 | 変更しない | `git diff origin/main -- docs/DEV_WORKFLOW.md` に既存行の削除・変更なし | 作業群通算は non-scope |
| 隣接契約: Findings Freeze、Plan Review round 天井、Writer ≠ Plan Reviewer | 変更しない | 同上 | なし |

## Test Plan

[Test Design Matrix](test-matrices/2026-09-22-workflow-ordinary-operation-and-stop-routing.md)。自動 test は追加しない（規則文の意味は機械で判定できず、checker の新設は非目的）。文書の意味は事例照合 T1〜T7 で確認し、既存の docs / workflow gate を実行する。packet と実装は別 commit（plan-first）。Writer は S1〜S4 を 1 つ以上の commit で実装し、packet・Matrix・`docs/Plans.md` を編集しない。

- targeted tests: AC1〜AC6 の command。
- negative tests: T3（正本の変更が要る不一致を発注の訂正で通さない）、T4（codex-only で owner の裁定を移さない）、T5（新しい gate が増えていない）。
- compatibility checks: T7（設計を含まない小さな R2 は `not applicable` で足りる。既存 packet へ遡及しない）。
- data safety checks: 外部設計相談の原文・owner 発言の原文を tracked file へ転記しない。
- main wiring/integration checks: T6（S3 / S4 から行き先の表への link、Plan Packet Rules から Review Rules への link。file の実在は doc check、anchor の一致は AC4 の command と review）。

## Boundary / Wire Contract

N/A: JSON / CSV / config / DTO / generated binding / DB の変更なし。`Ordinary Operation` の表は人が読む構成で、機械 parse 用の schema にしない。

## Review Focus

- §5.6 の既存文（Coordinator が正本と発注書を訂正する）を根拠に、D3 の 5（Gated Amendment へ戻る）を迂回できる読み方が残っていないか。
- D3 の「owner への中継を要しない」条件が、Scope / AC / 権限 / review 数 / finding の採否の変更や、実行 mode 上 owner が持つ裁定（§3.2 codex-only、D-087）を通す抜け道になっていないか。
- D2 が新しい phase・review 本数・常設 gate・承認 commit を作っていないか。§5.4 の低制約 profile と矛盾しないか。
- D4 の表が `AGENTS.md` Decision and Approval Boundaries と Owner Effort Budget の既存規則を弱めていないか。行の間で行き先が二重に読める問い合わせがないか。
- 規則が 1 か所だけに置かれ、link で参照されているか（表や規則文の複製がないか）。
- 普通の一日の観点: 次の ㉘ 起票で Coordinator がこの template と規則だけで操作列を書き、reviewer へ渡せるか。

## Spec Contract

Contract ID: SPEC-WF-LIGHT1A

- D1（template `## Ordinary Operation`）: 位置は `## Goal` の直後。表の列は `初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照`。設計を含む変更（operator の操作、data / command 契約、業務の状態遷移を決める・変える packet）は表を 1 つ置き、必要なら翌日の起動・再試行まで含める。workflow の変更は、発注 → 停止と訂正 → review → owner 判断の通常列を使う。どちらでもない変更は `not applicable` と理由を 1 行で書き、節は削除しない。範囲を限定した packet（design-only 等）を閉じる場合も、製品の目的が未達なら明記し、「この文書を完了できる」と「通常運用を達成できる」を分けて書く。
- D2（Plan Review の最初の問い）: Plan Review の発注は、対象 packet の適用版・Goal・`Ordinary Operation`・Contract Probe・対象差分を参照させ、Scope や期待結果を別の文へ書き直さない。reviewer は報告の冒頭で、操作列が目的を達成できるかを `成立 / 具体的な反例あり / 外部前提が未確認` のどれかで答える。不成立なら最短の入力・状態遷移と、期待した結果・到達した結果の差を示す。「危険な結果を出さないか」と「正常な条件で目的を達成できるか」は別々に問う。この確認は既存の Plan Review の同じ run・同じ採否判断に含め、新しい phase・review 本数・常設 gate・単独の承認 commit を足さない。`not applicable` の packet では操作列の成立を問わないが、reviewer は `not applicable` とした理由の妥当性を確認し、D1 の適用対象（設計を含む変更・workflow の変更）に当たる packet が `not applicable` としていれば finding にする。
- D3（Writer が編集前に止まったとき、§5.6）:
  1. Writer は、発注と適用版の正本が一致しない場合に編集を始めず、不一致箇所・正本の参照・現在 HEAD を Coordinator へ返す。
  2. 正本が一意で発注だけが誤っている場合は、Coordinator が発注を作り直す。この訂正で Scope・AC・commit 条件・権限・phase・Risk・review 数を変えない。
  3. Coordinator は訂正後に §5.6 作成・訂正の手順 4 の最終照合（旧前提 sweep を含む）を行う。
  4. 2 と 3 を満たす発注の訂正は owner への中継を要しない。
  5. 正本が曖昧な場合はこの経路を使わない。正本の意味を変える場合は既存の独立確認と Gated Amendment へ戻る。
  6. finding の採否と権限の追加を発注の訂正として扱わない。
  7. 実行 mode により owner が持つ裁定・承認（§3.2 codex-only、D-087）をこの規則で代行しない。
  8. §5.6 の既存文「食い違いは Coordinator が正本と発注書を訂正し、Writer が独断で条件を外さない」のうち正本の訂正は 5 の経路で行い、Coordinator が単独で行えるのは 2 の発注の訂正だけである、と同じ規則文の中で関係を示す。既存文「正本の変更が必要なら既存の改訂・承認経路を使う」と重複する文を別に足さず、5 はその文を指す。
  9. 例: PR #85 の「合成モデルは変更しない」という発注とモデル是正の要求の衝突は、正本の変更が必要な例であり、発注の訂正として通してはならない。
- D4（`### 問い合わせの行き先`、Owner Effort Budget 内）: 次の 7 行の表。列は `問い合わせの種類 | 最初の行き先 | owner への中継を省ける条件`。
  1. 発注書だけの誤記・転記の矛盾・古い着手条件 → Coordinator。条件 = D3 の 2〜3 を満たす。
  2. 環境の不備・既知 command の出力先の確認 → Coordinator / 担当者。条件 = 許可済みの環境・範囲で解消でき、追加の費用・権限・正本の変更を伴わない。
  3. 技術的な前提が不明 → agent の調査・Contract Probe。owner に技術的な正しさの承認を求めない。結果が製品の振舞い・受容リスクの選択になる場合は 6 へ。
  4. 正本の意味の変更・影響の分類の争い → 独立 reviewer による事実確認、その後は実行 mode 上の裁定者。独立 review は owner の採用権を代替せず、裁定権が owner にある場合は中継を省かない。
  5. 店の事実・実機でしか確認できない挙動 → owner。具体的な質問か、短い PASS / FAIL で答えられる形にする。省略不可。
  6. 目的・製品の振舞い・受容リスク・予算・優先順位・範囲の変更・不可逆な操作 → owner。省略不可。
  7. Windows L3・R4・Ready・merge → owner。現行の明示承認と有効な委任の範囲だけを使う。
  表の前後に、(a) 問い合わせが表の複数行に当てはまり、その中に owner 側の行（5〜7）が含まれる場合は owner 側の行を優先すること・裁定権の所在そのものが争点の場合は owner へ送ること・owner 側でない行だけに当てはまる場合は各行の条件に従うこと（3 文、closeout で訂正、closure P3-2。旧記述は「複数の行に当てはまる場合、または裁定権の所在が争点の場合は owner 側の行を優先」という 1 文で、実装〈`docs/DEV_WORKFLOW.md:326`〉と字面が一致していなかった）、(b) この表は `AGENTS.md` Decision and Approval Boundaries と本節の上限・計上規則を変更しないこと、(c) owner に証跡の編集や発注訂正の伝言を依頼しないことを書く。
- D5（D-090）: Status = accepted（owner 決定 2026-09-22）。Decision = D1〜D4 の要旨。Why = PR #85 の Plan Review rally と発注訂正の停止の実績。Compatibility = phase・review 本数・Human Gate・Plan Commit / Amendments・独立性・Double Audit・D-084 / D-087 の owner の裁定を維持、checker は変更しない、既存 packet へ遡及しない、1 段目の残りと 2・3 段目は別 change。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-LIGHT1A-D1 | S1 | AC1、T1、T7 | 普通の一日の観点 | PR の diff、runner 出力 |
| SPEC-WF-LIGHT1A-D2 | S2 | AC2、T1、T5 | 新しい gate を作っていないか | 同上 |
| SPEC-WF-LIGHT1A-D3 | S3 | AC3、T2、T3、T4 | 抜け道になっていないか | 同上 |
| SPEC-WF-LIGHT1A-D4 | S2 | AC4、T3、T4、T6 | 既存の承認境界を弱めていないか | 同上 |
| SPEC-WF-LIGHT1A-D5 | S4 | AC6、T6 | 複製がないか | 同上 |

## Data Safety

- commit しないもの: 外部設計相談の回答（`.local/consultations/**`）の原文、owner 発言の原文、vendor の session ID、実店舗のデータ。
- local-only paths: `.local/**`（発注書、review 報告の控え、gate の出力）。
- synthetic-only paths: 該当なし（fixture を追加しない）。

## Implementation Results

[PR #90](https://github.com/higekiri-w/inventory-system-desktop/pull/90) で実装し squash merge 済み（`89f0bc2d`、2026-09-22）。`docs/templates/plan-packet.md` に `## Ordinary Operation` 節、`docs/DEV_WORKFLOW.md` の Review Rules・Plan Packet Rules・Owner Effort Budget に規則項目と `### 問い合わせの行き先` の表、`docs/AGENT_OPERATING_MANUAL.md` §5.6 に「Writer が編集前に止まったとき」の規則文、`docs/decision-log.md` に D-090 を追加した。4 file とも既存行の削除は無く追加のみ。Writer 実装後、Coordinator の diff 確認で packet 内だけの略記 `D1`〜`D4` が正本へ漏れた箇所・Writer 向けの指示文が規則文へ残った箇所・D-090 の Why の文意・表の注記の Markdown 描画の 5 点を是正した。Final Review broad の finding のうち 3 点（注記 (a) の複数行一致時の適用範囲〈A-1 / B-5〉、Plan Packet Rules の項目の出典を template へ明示〈B-2〉、非遡及の 1 句〈B-3〉）を採用して是正した。

## Review Response

Plan Review（Opus、独立 fresh context）: round 1 = 操作列は `成立`、P1 0 / P2 5 / P3 4 → 全件採用して是正 → round 2 closure = P1/P2 なし、P3 2 → 反映（Plan Commit）。Plan Reviewer は Writer（Sonnet）と別 model だが同じ vendor で、PR #88 の dogfood 所見の条件を Plan Gate 時点では解消しないことを遷移記録に明記した。

Final Review broad 2 本（互いに独立、Plan Reviewer とも別の fresh context、Double Audit）: pass A（Codex、`#pullrequestreview-5272080909`）P1 0 / P2 0 / P3 2、pass B（Opus、`#issuecomment-5767767062`）P1 0 / P2 0 / P3 5。finding 7 件の裁定（Coordinator、`#issuecomment-5768012695` に記録）: 採用 4（A-1 / B-5 / B-2 / B-3）→ 是正して反映、closeout 1（A-2、Matrix の Mutation-style の言い過ぎ）→ 本 closeout で訂正、follow-up 2（B-1、B-4）→ `docs/backlog.md`「workflow の軽量化 3 段」へ起票。

Final Review closure（Opus、別 context、`#issuecomment-5768012695`）: P1 0 / P2 0、新規 P3 3 件。うち 1 件（broad の裁定が PR に未記録）は PR body へ記録して解消、残り 2 件（closure P3-2: Spec Contract D4 (a) と Matrix の (a) 関連記述の実装への訂正、closure P3-3: 非遡及の句の carve-out 有無）は本 closeout で処理した。合格、hosted CI pass、helper Ready → merge。

- Findings Freeze: frozen after Broad Audit（broad 2 本の完了後）; post-freeze exceptions: none.
