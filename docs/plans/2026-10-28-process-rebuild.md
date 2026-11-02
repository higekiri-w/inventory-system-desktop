# Plan Packet: 開発の手続きの作り直し（D-145）

wave に属さない単独の lane。開発の手続き（packet の形・Workflow State・review と L3 の取り直し・closeout・Plans・CI の重複・owner の手）を、守りたいことの芯から組み直す workflow の lane。helper（`scripts/pr-gate.py`）・検査（`scripts/doc-consistency-check.sh`・`scripts/check-workflow-git.sh`）・hosted CI（`.github/workflows/ci.yml`）・その test と、手続きの正本の文（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/agent-guidance/merge-evidence.md`・`docs/ci.md`・template ほか）を 1 本の PR で変える。製品の runtime（`src/**`・`src-tauri/**`）は変えない。

出所: owner の決定（repo 外の回答台帳 TD-474〜478〈2026-10-27〉、TD-479〜502〈2026-10-28〉）。所感 TD-475（小さな見直しでも大規模になる今の体制はおかしい）・TD-478（芯から組み直す）・TD-489（ゼロからすっきりした形へ、見直せるだけ見直す）を受け、Coordinator が作らせた設計案（repo の外の `.local/reports/process-slimming-2026-10-27/rebuild-proposal-v2.md`、以下「v2」）の §9 の 5 問に owner が答えた: Q1 = A（TD-498）、Q2 = C（TD-499）、Q3 = A（TD-500）、Q4 = A（TD-501）、Q5 = B（TD-502）。そのほか TD-481（同じ head の CI を 1 回に、優先度は下）・TD-483（表示だけの修正は R2）・TD-484（R3 の Final は 2 本を既定にし helper に守らせる）・TD-487（Plans は関門・ブロッカー・製品の未決判断だけ、完了は git log、進捗は handoff）。

規模と手法: 起票（本 commit）は本 packet・Test Design Matrix・`docs/decision-log.md` の D-145（plan-first）。正本の文の書き換えと helper・検査・CI・test の変更は plan-approved の後の実装の段で、#217 の merge の後に `origin/main` を単段で取り込んでから始める（下の「#217・#204 との順」）。正本の文を本 commit で書き換えない理由は下の Design Readiness。

この PR 自身は、merge されるまで今の規則（`docs/DEV_WORKFLOW.md` の Workflow State・Plan Packet Rules）で進める（v2 の K12）。機械の側は 2 つに分かれる: helper（`scripts/pr-gate.py` の status・capture・record・ready・merge）は PR の base の版で走る（D-099）。bash の検査（`doc-consistency-check.sh` の PK1・PK4・PK5 と、本 lane が足す H2・H13）は pre-push と hosted の docs job で branch の版で走り、PK4 は branch の `scripts/pr-gate.py` の `parse_packet` を import する。よって実装の commit の後は、この PR の packet を新しい検査が見る。本 lane が自分の検査を緩めて自分を通すことの手当て: (1) 本 packet は最後まで今の形（Workflow State の 11 行と今の必須の節）に保ち、新しい検査と今の検査の両方を満たす。(2) 最終 head で、base の版の検査（`scripts/` を base の版に差し替えた一時の worktree）でも本 packet を回す（AC9）。D-145 の新しい規則をこの PR に先取りで当てない。例外は owner の 1 回限りの決定 TD-502（下の Plan Reviewer の行）だけで、座組表の運用の本数を変えるもので helper の強制は変えない。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: aa14364173dda14d22d87b0714f4f683ad39e535
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は本 run の起草役。実装は plan-approved の後の別の run で `subagent_type: writer`）
- Plan Reviewer: round 1 は fresh Opus 5.5（`subagent_type: reviewer`、fork でない）+ Codex の 2 本（座組表どおり）。round 2 以降は、前の round で P1/P2 を出した側だけに出す（両方が出していれば両方。repo 外の回答台帳 TD-502、この PR に限る owner の決定）。round の天井は 3（`docs/DEV_WORKFLOW.md` Review Rules）
- Final Reviewer: Claude 側 1 本（R3 なので Fable 5〈当面、TD-410〉）+ Codex。座組表どおり。gate を変える変更なので、Codex が使えない間も Fable 5 + fresh Opus に代えず Codex を待つ（TD-499 の C）
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/process-rebuild

manual なし: helper・検査・CI と文書の変更で、Windows native で確かめる物が無い。Final Review Minimum 2: `scripts/pr-gate.py`・`scripts/**`・`.github/workflows/ci.yml`・`docs/agent-guidance/**` を変え classifier が `workflow=true` を出す workflow gate の変更（`docs/DEV_WORKFLOW.md` Workflow State の `Final Review Minimum`、Contract Audit の Double Audit）。

遷移の記録:

1. kickoff → spec-check（2026-10-28、起草役）: 範囲を v2 の §8.1 の段 2（§6 の H1〜H9・H11〜H14、§7 の tracked の文書、decision-log の新しい D、#204 の目的の H13 での畳み込み）とし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-28、起草役、同じ commit）: 手続きの正本の多くの節を置き換える設計が要る（下の Design Readiness）。
3. design → plan-draft（2026-10-28、起草役、同じ commit）: 設計の判断を D-145 に置いた。各正本の書き換えの中身は下の Scope の「正本の文の最終形」に節ごとに定め、文そのものは実装の段で書く（理由は Design Readiness）。未解決の設計の問いは無い（決めきれなかった点は Review Focus に挙げ、Plan Review に問う）。
4. plan-draft → plan-gate（2026-10-28、Coordinator）: packet と Matrix は起票の commit `2efd9426` で commit 済み。`doc-consistency-check.sh --target plan` は全チェック通過。
5. plan-gate のまま是正（2026-10-28、round 1 の reject を受けて）: round 1 の是正（Review Response 参照）。
6. plan-gate のまま是正（2026-10-28、round 2 の reject を受けて）: round 2 の是正（Review Response 参照）。次の round 3 が天井。
7. plan-gate のまま是正（2026-10-28、round 3〈上限〉の reject を受けて）: round 3 の是正（Review Response 参照）。
8. plan-gate のまま是正（2026-10-28、closure の P2 を受けて）: closure の是正（Review Response 参照）。
9. plan-gate → plan-approved → implementing（2026-10-28、Coordinator）: Plan Review は round 1〜3（上限）と、上限の後の是正の確かめ（owner の決定 TD-506）で、独立の reviewer の P1/P2 = 0（Codex 側 Astra の closure が `6c0ce73d` で approve、Claude 側 fresh Opus 5.5 の closure が `aa143641` で approve）。Plan Commit = `aa143641`（計画の最後の commit で、すべての実装の commit より前）。closure の P3 2 件（D-145 (4) に形式の見分けの範囲と止める条件を足す、「強める」の対象から `unknown` を外し migration・依存の 2 つにする）は実装の段で直す。

## Owner Effort Budget

今の規則（PK1 の必須の節）で起票するので置く。D-145 (11) で Owner Effort Budget を消す決定（TD-501）は、この PR の merge の後の packet から効く。

- 介入回数上限: 8（既定 6 から上げる。理由: 起票の前に設計の入力として 5 つの decision point〈TD-498〜502〉を 1 回の問いで得たため）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定。round 2 以降の本数は Workflow State の Plan Reviewer の行の TD-502 の形）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 8 | 5（TD-498〜502） | 2（Ready 1・merge 1） | 1（Plan Review の裁定が owner に上がる 1） | 8 = 5 + 2 + 1 |

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
merge の関門（helper の review・manual の受理条件、Final Review Minimum と Human Gate の決め方、packet の検査、hosted CI の `Merge gate` の出し方）を変える。`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「merge gate changes」。D-098 が挙げる安全の境界（古い証拠の拒否・独立 review・Plan Commit の固定・fail-closed）のいくつかの形を変える改訂で、AGENTS.md Working Rules の「安全の境界の改訂は通常の packet・PR の経路で行う」に従い、owner の決定（TD-474〜502）で起票する。項目ごとの扱いは下の「安全の境界ごとの扱い」。R4 にしない理由: 破壊的な操作・店の実データ・secret・不可逆な git の操作を伴わない（既存の packet・archive を消さず、動かさない）。

required gate の green / red: 変わる。(1) Draft の run も `Merge gate` を出す（H9）。(2) pre-push と hosted の docs job に、計画の承認の前の実装を止める ERROR（H2）と、生きた docs に `path:行` を新しく足す ERROR（H13）が加わる。(3) PK1・PK4・PK5 の対象が active な packet（起点の tree に無い dated packet）に絞られ（H4）、必須の節が変わる（H14）。

## Goal

Goal Invariant:

### 最小完了条件

- R3 の lane で、人（owner・Coordinator・Writer）が書き写す・書き直すのは判断（計画・finding・裁定・PASS/FAIL・承認）だけになり、状態（Phase・担当・本数・Human Gate・完了の一覧・lane の現在地）を手で書かずに、今と同じ安全の境界で merge まで進む。具体的には: packet に Phase・担当 4 行・Branch・Final Review Minimum・Human Gate・Owner Effort Budget を書かない、`Plan Commit` を手で写さない、merge の後に closeout の PR を作らない、L3 の後の docs だけの commit で L3 を取り直さない、同じ head の CI を 2 回回さない。
- R2 の lane（表示だけの修正と、gate・未知の path に触れない開発の道具・docs）は packet と計画の review なしに、PR body の 5 行と差分の review 1 本で merge まで進む。R2 の行き先は 1 つ: classifier が `workflow=true` か `unknown=true` を出す変更は R2 にできず R3 として起票する。`rust`・`frontend` に触れる R2（契約に触れない表示・開発の道具）は packet なしでよいが、migration（`src-tauri/src/db/migration.rs`・`src-tauri/src/db/migration_tx.rs`・`src-tauri/src/db/schema_v*.rs`）と依存（`package.json`・`package-lock.json`・`.npmrc`）に触れる変更は R2 にできず R3 として起票する。
- 走っている lane（起票の時点の open の PR 13 本）の packet を書き換えずに、新しい helper・検査がそれらを受ける。

### 失敗定義

- 計画の承認の前の実装、古い head の証拠、Writer 自身の review、R4 の承認の使い回し、壊れた packet のどれかを、新しい helper・検査が今より通しやすくする（緩めると packet に書いた H6・H8・H14〈新しい形の R3 の Data Safety〉の 3 か所を除く）。
- 走っている lane の旧い packet を新しい helper・検査が拒み、lane が止まる。
- 文書と helper が食い違う期間ができる（文だけ先に merge する等）。
- 既存の拒否の test を消す・skip する・弱める。

### 非目的

- 製品の振舞い・画面・DB の変更。
- repo の外の運用の道具（`.local/checklists/`・`.local/codex-orders/templates/`・`new-order.sh`・`env.md`・`MODEL-SELECTION.md`・MEMORY の 1 行目）の作り直し。v2 §8.1 の段 0 で、Coordinator が PR なしで行う。
- owner の個人の `~/.claude/CLAUDE.md`、Codex の sandbox の設定（TD-500。settings の変更は owner）。
- GitHub の ruleset（required check の名 `Merge gate` は変えない）。
- 既存の archive の packet・WER・decision-log の過去の D の本文の書き換え（Superseded の行の追加だけ）。
- D 番号・migration の版の自動の採番、PR の件名の型の検査、owner の承認の自動の記録（v2 §6 の「入れないもの」）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、作り直しの後の通常列を置く（D-145 の流れ）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| R2 の表示だけの修正の branch | Writer が実装 → PR body に目的・範囲・非目的・AC・Risk の 5 行（Risk の行は `Risk: R2（目視: 要）` か `Risk: R2（目視: 不要）` の固定の書式）→ `pr-gate.py capture --risk R2 --manual required`（operator 向けの画面に触れるとき。目視を manual として `record --kind manual --evidence <目視の記録の pointer>` で記録する）か `--manual not-required`（画面に触れないとき）→ 差分の review 1 本 → record | packet・計画の review・Plan Commit なしで Draft まで進む。画面の目視を飛ばすと helper の status が `manual incomplete` で止まる | review が P1/P2 0、画面なら manual の pass | R2N1・R2N2・R2N3 |
| R3 の lane を起票 | packet（最小形）を書いて commit → Plan Review round 1 の 2 本 → 是正 → `scripts/plan-approve.sh <packet> <reviewer の読んだ SHA>` | Plan Commit を手で写さない。承認の前に code を push すると pre-push が止める | Plan Review が P1/P2 0 | PA1・PH4 |
| R3 の lane を実装・review | 実装 → Draft → capture → Final broad 2 本（Risk から helper が 2 を導く）→ 是正 → closure | packet に本数・Human Gate を書かない | helper status の blocker が manual と CI だけ | MN1 |
| L3 の後に P3 の docs の是正を積む | lane の docs だけの commit → push → capture → `record --kind review --review-stage docs --evidence <P3 だけと裁定した pointer>` | L3 も review も取り直さない（C・H6） | `## L3` 節と計画の節が変わらない | C1・RC1 |
| 先行 PR の merge の後 | `git -c merge.directoryRenames=false merge origin/main`（#217 の A・B のまま）。計画の承認の前に同期したときも、`plan-approve.sh` は lane の commit だけを見て、間の merge は #217 の同期だけの条件で確かめる | 同期で L3 も計画の review も取り直さない | #217 の条件 | #217・PA5 |
| Ready・merge | owner が名指しの PR の束を 1 回で承認 → Coordinator が helper `ready`・`merge`。同じ head の Draft の run の `Merge gate` を merge の証拠に使う | closeout の PR を作らない。Plans に完了の行を足さない | helper の `merge` が通る | CI1・CI2 |
| 次の session の再開 | `git fetch origin` → `git log --oneline -20 origin/main` → `gh pr list --state open` → 名指しの PR ごとに `python3 scripts/pr-gate.py status --pr N` と、PR の active な packet があれば `--packet <path>`、無ければ PR body の Risk の行（`Risk: R<0|1|2>（目視: 要|不要）`）から `--risk R0|R1|R2` と `--manual required|not-required` をその PR の値で渡す（helper が base の版と違えば、helper が出す base の版の command で引く）→ handoff | 完了は git log、lane の現在地は helper から読む。merge 済みの packet を active と取り違えない（checker の active の定義） | — | AR2・AR4 |
| Codex の枠が無い | R3 の Final は Fable 5 + fresh Opus の 2 本。R4 と gate を変える変更は Codex を待つ | R3 の lane が止まらない | — | TD-499 |

## Scope

新しく書く引用は D-125・`docs/DOC_STYLE_GUIDE.md` §1 の形（path と symbol・節名・決定 ID。行番号で引かない）。owner の決定は「repo 外の回答台帳 TD-nnn」で引く。

### 予定の file

| file | 変更 | 段 |
|---|---|---|
| `docs/plans/2026-10-28-process-rebuild.md`・`docs/plans/test-matrices/2026-10-28-process-rebuild.md` | 本 packet と Matrix | 本 commit |
| `docs/decision-log.md` | 末尾に D-145。置き換わる D（D-034・D-038・D-055・D-097・D-101・D-107〈PK4 の対象を active な packet に絞る所〉・D-124〈#217 の D-144 の後の残り〉）の末尾に `Superseded in part by: D-145` の行 | D-145 は本 commit、Superseded の行は実装 |
| `scripts/plan-approve.sh`（新設） | H1 | 実装 |
| `scripts/pr-gate.py` | H2・H3・H4・H5・H6・H7・H8・H9（`ci`）・H11、`mutate` の merge の後の案内 `docs closeout PR` を外す | 実装 |
| `scripts/check-workflow-git.sh` | H2（Phase の検査を外し、先走りの検査を足す） | 実装 |
| `scripts/doc-consistency-check.sh` | H2（PK4 の Phase）・H4（PK1・PK4 の対象、Plans の pointer の検査を外す）・H13・H14 | 実装 |
| `.github/workflows/ci.yml` | H9（`merge_gate` の name 式、trigger の `ready_for_review`）。docs job の `doc-consistency-check.sh` の step に `WORKFLOW_BASE_SHA`（PR の base の SHA）を渡す行（今は PK5 の step だけが持つ。H2・H4・H13 の起点） | 実装 |
| `scripts/pre-push.sh` | H2・H13 の検査に起点を渡す（今の `WORKFLOW_BASE_SHA` の渡し方に合わせる。要るときだけ） | 実装 |
| `scripts/tests/pr-gate.test.py`・`workflow-git-checks.test.sh`・`doc-consistency-plan-packet.test.sh`・`ci-workflow.test.sh`・`reading-order-drift.test.sh`・`run-workflow-tests.sh`、新設の `plan-approve.test.sh` | Matrix の行。Phase・移送・Owner Effort Budget・`Draft (no merge evidence)` を前提にした case を新しい契約に合わせて書き換える | 実装 |
| 正本の文（下の「正本の文の最終形」の表の file） | 書き換え | 実装 |

### 正本の文の最終形

行数は見込みで未実測（起票の時点の行数は `wc -l AGENTS.md CLAUDE.md docs/DEV_WORKFLOW.md docs/AGENT_OPERATING_MANUAL.md docs/agent-guidance/merge-evidence.md docs/ci.md docs/code_review.md docs/templates/plan-packet.md docs/templates/test-design-matrix.md docs/templates/subagent-review-packet.md .github/pull_request_template.md docs/Plans.md` → 76・35・364・217・173・100・107・203・111・33・45・85。`docs/Plans.md` は `Plans.md` の symlink の実体）。各 file は #217 の merge の後の版から書き換える。

| file | 残す・置く中身 | 消す中身 |
|---|---|---|
| `AGENTS.md` | Session Start の表を 5 行（質問・調査 / R0〜R2 / R3〜R4 の計画と実装・再開 / review / Ready・merge）にし（Ready・merge の行は残し、読む資料は `docs/ci.md` と `docs/agent-guidance/merge-evidence.md`。closeout だけを消す）、先頭に「`git fetch origin` の後、`git log --oneline -20 origin/main` で完了を、`gh pr list --state open` と helper の status（名指しの PR ごとに明示の引数で）で lane の現在地を読む。handoff・memory・前の調べの値は行動の根拠にする前に正本で引き直す」（v2 §2.6）。Working Rules・Decision and Approval Boundaries（束ねた Ready/merge の承認の委任を R0〜R4 の全 PR に広げる。Codex の起動を Coordinator の委任に足す〈TD-500〉）・Workspace Access・Memory Model（現在地は handoff、Plans は関門・ブロッカー・製品の決まっていない判断）・Safety・npm ガードは文のまま | Ready・merge・closeout の行の closeout、`Plans.md` を現在地とする文 |
| `CLAUDE.md` | import・言語・報告の形・コピペ物の path・安全境界の検証を subagent に出さない・座組表への pointer | 重複した入口の説明（AGENTS と同じ文） |
| `docs/DEV_WORKFLOW.md` | Risk の線（R2 に表示だけの修正〈TD-483〉、状態を持つ画面は R3）、Risk ごとの最小の流れの表（v2 §2.2。R2 は Q1 の A）、R3/R4 の packet の最小形（v2 §2.5）、計画の固定（`plan-approve.sh`・PK5・先走りの検査）、L3（`## L3` 節・L3 Eligibility・fixture の実 encoding・取り直しの表〈v2 §2.3〉）、review の規則（Findings Freeze・round の上限 3・不可逆 finding の 4 項目・Contract Audit の項目・計画の review の発注の固定の問い〈普通の一日・登録と生成・外の前提・データの安全・前提の出所〉・closure の固定の問い〈是正で足した契約文と見送った経路を全部挙げる〉）、検証の command、Design Phase の短い規則、Wave の one-writer・path の所有・単段の同期（旧形式の packet は rename・切り直しをしない、の 1 文を含む）、Subagent Budget、Commit / PR Messages | Phase の enum・遷移の表・遷移の記録、Workflow State の担当 4 行・Final Review Minimum・Human Gate の手書き、Owner Effort Budget の節、Post-Merge Closeout の節、Draft PR Checkpoint の PR body の全文の直し直しと介入の数、`Draft (no merge evidence)` を待つ Ready の手順、Artifact Map の archive と test-matrices の行 |
| `docs/AGENT_OPERATING_MANUAL.md` | 役割・独立性・座組表（model は agent 定義の名を指す。effort の値は agent 定義の frontmatter を正とする）、§3.3 に TD-499（R3 の Final は Codex が使えない間 Fable 5 + fresh Opus、R4 と gate を変える変更は Codex を待つ）、§3.5、発注書の必須項目、相談役は任意 | §5.3（Plans cleanup）、相談役の必須、Owner Effort Budget への言及 |
| `docs/agent-guidance/merge-evidence.md` | helper の契約。計画の固定（H1）、Minimum・Human Gate の導出（H3）、L3 の使い回し C と docs の closure（H5・H6）、GA の closure（H7）、R2 の packet なしの道（H8）、同じ head の CI（H9）、`--report`（H11）を足す。MG-D3 と MG-D9 に Superseded の注記 | closeout の節、Phase archive の移送 |
| `docs/ci.md` | Draft の run も `Merge gate` を出し、`ready_for_review` で再実行しない。Final Trigger Selection を同じ head の 1 run に縮める | Draft と Ready の 2 名の段、Ready の後の Draft の赤の rerun の手順 |
| `docs/code_review.md` | closure の固定の問いを 1 項 | — |
| `docs/templates/plan-packet.md` | 下の「新しい packet の書式」の正例と同じ形 | Workflow State の 11 行の雛形（3 行に）、Owner Effort Budget、Ordinary Operation・Registration・Impact Review Lenses・Boundary/Wire・Contract Probe の節（発注の固定の問いへ移す）、R3 の Data Safety（発注の固定の問いへ。R4 は残す）、Implementation Results・Review Response（PR の comment へ） |
| `docs/templates/test-design-matrix.md` | 削除せず、旧い packet が引く参照として残し、冒頭に「新しい packet は packet の `## Test Plan` の表に書く」 | — |
| `docs/templates/subagent-review-packet.md` | 計画の review と closure の発注の固定の問い。reviewer は読んだ SHA を `REVIEWED_HEAD=` の 1 行で報告する（H11） | Review Response を読ませない除外の指示は、新形式の packet では要らない（packet に無い）。旧形式の packet の review では今の除外の文を残す（移行中の lane の round 2 以降の独立性のため） |
| `.github/pull_request_template.md` | 要約・範囲・検証の結果（報告を指す）・残り。R0〜R2（packet なしの道）は目的・範囲・非目的・AC・Risk の 5 行で、Risk の行は `Risk: R<0|1|2>（目視: 要|不要）` の固定の書式（例 `Risk: R1（目視: 不要）`。helper の `--risk`・`--manual` の値と 1 対 1 で、helper が照合する） | Plan Packet・Source docs・helper status の link・介入 N/M・Dashboard / archive follow-up |
| `docs/Plans.md` | 導入の前の関門・ブロッカー・製品の決まっていない判断・参照（TD-487） | 現在のフェーズ・次の行動・直近の完了・Wave Registry |
| `docs/DEV_WORKFLOW.md` の検証の節（追記） | review の copy は `git archive` でなく `git worktree add` で作る（起点が無いと PK1・PK4・PK5 が active を決められず ERROR になる） | — |
| `docs/PROJECT_HANDOFF.md`・`docs/project-profile.md`・`docs/DEV_SETUP_CHECKLIST.md`（`Draft (no merge evidence)` の行）・`.agents/skills/inventory-workflow-start/SKILL.md`・`.agents/skills/inventory-implementation/SKILL.md`・`.agents/skills/test-design/SKILL.md`・`docs/backlog.md`（Wave Registry の項目と、`Draft (no merge evidence)`・`ready_for_review` を引く項目） | 消える節・語への参照を新しい形に直す（drift の sweep。下の AC4・AC7。対象の行は実装の段に AC4・AC7 の `rg` で全部挙げる） | — |

### 新しい packet の書式（1 通り）

新しい packet は次の正例の形だけにする（v2 §2.5 の先頭の裸の `Risk:`・`Plan Commit:` の行の形は採らない）。Workflow State は `- Key: value` の 3 行で、helper・PK4・PK5 はこの節だけを読む。`## Risk` の `Risk: Rn` の行は PK1 と helper が Workflow State の Risk と一致を確かめる今の検査のまま残す（機械が照合する鏡で、食い違えば止まる）。見出しの名は今の template と同じ英語の名を使い、新しい名を作らない（旧い packet との対応を少なくする）。下の正例は字下げの code block にし、本 packet の検査がそれを本 packet の節と取り違えないようにした。

    # Plan Packet: <件名>（PR の件名と同じ）

    ## Workflow State

    - Risk: R3
    - Plan Commit: pending
    - Amendments: none

    ## Risk

    Risk: R3

    Reason:
    <理由 1 文。出所つき>

    ## Goal

    Goal Invariant:

    ### 最小完了条件
    ### 失敗定義
    ### 非目的

    ## Scope
    ## Acceptance Criteria
    ## Design Readiness
    ## Contract Ledger
    （表: 契約 ID / 設計正本の節 / 実装（Scope） / 自動 test / L3・非対象）
    ## Test Plan
    （表: Contract / Failure Mode / Test Type / Test Name / Would fail if...。runtime の R3 は状態の遷移・negative・mutation の注入を含む）
    ## L3            ← manual があるときだけ（節があること = manual）
    ## Recovery      ← R4 だけ
    ## Data Safety   ← R4 だけ

形式の見分け: 形式は機械が決める: packet を最初に足した commit（起点..HEAD の範囲〈起点は下の active な packet の定義と同じ〉で `git log --diff-filter=A --format=%H <起点>..HEAD -- <packet>` の最も古いもの。範囲を起点の後に限るので、昔 `docs/plans/` にあった path を足し直しても昔の add を拾わない）の親の tree に `scripts/plan-approve.sh` が無ければ旧形式、あれば新形式。旧形式は Workflow State に `Phase` の行を持ち（今の規則のまま）、新形式は持たない。形式と `Phase` の行の有無が食い違えば PK4 と helper が拒む（file 名の日付を古くしても、`Phase` の行を残しても、新形式の規則を逃れられない）。最初に足した commit が引けないとき、その commit に親が無いとき（root の commit・shallow の境の commit）、`git rev-parse --is-shallow-repository` が true のときは、判定できないとして ERROR（fail-closed。shallow の境の commit は全部の file の add に見え、親の tree が引けず旧形式に落ちる〈fail-open〉のを止める）。rename と切り直し: rename は `--follow` で辿らない（rename の後の path の最初の add で判定する）。走っている旧形式の lane が本 lane の merge の後に packet を rename するか、新しい main へ cherry-pick で切り直すと、新形式と判定され `Phase` の行で拒まれる。旧形式の lane は packet を rename せず、切り直さずに単段の同期で進める（DEV_WORKFLOW の Wave Operation に 1 文で書く）。`Phase` の値は評価しない。新形式の packet は R3 と R4 だけで、Risk R2 の新形式の packet は PK4 と helper が拒む（R2 は packet なしの道、H8）。

| 項目 | 新形式 | 旧形式（受ける形） |
|---|---|---|
| Workflow State | `Risk`・`Plan Commit`・`Amendments` の 3 行。`Final Review Minimum` は上げるときだけ 1 行 | 今の 11 行。`Final Review Minimum`・`Human Gate` は書いた値をそのまま使う（下の H3） |
| PK1 の必須の節 | 全部: Risk・Goal・Scope・Acceptance Criteria。R3+: Design Readiness・Contract Ledger・Test Plan。R4: Recovery・Data Safety | 今の PK1 の必須の節（Owner Effort Budget・Contract Probe・Data Safety・Test Design Matrix の link を含む）のまま。非遡及 |
| 契約の一覧 | `## Contract Ledger` | `## Contract Ledger`、または `## Spec Contract` と `## Trace Matrix` の組（今と同じ） |
| テストの表 | `## Test Plan` の中の表 | `## Test Plan` と Test Design Matrix の link |
| manual | `## L3` 節があること | Workflow State の `Human Gate` に `manual` |
| L3 の比較対象（H5・H6） | `## L3` 節 | `## L3` 節があればそれ。無ければ `## Test Plan` 節の全体と、packet が link する Test Design Matrix の file の全体（#211 は Matrix の `Manual` 節に L3 の基準を置く）。`## L3` も `## Test Plan` も無い、または Matrix の link が 0 本・2 本以上・file が無いときは、比べられないとして使い回さない |

同じ入力を全部の読み手に通す（AC8）: 正例（新形式）は本 branch の一時の worktree に置いて全部の読み手に通す。旧形式の lane の packet は、各 lane の branch の head の一時の worktree（各 lane の履歴のまま）に実装後の `scripts/` を重ね、全部の読み手（PK1・PK4・PK5・H2・`parse_packet`・`requirements`）をそこで回す。

### active な packet の定義（1 か所）

active な packet = `docs/plans/` 直下の dated packet（`docs/plans/YYYY-MM-DD-*.md`）のうち、比べる起点の tree に同じ path が無いもの（この branch が足した packet）。起点は、helper では PR の base の SHA、push 前と hosted の検査では `WORKFLOW_BASE_SHA` と HEAD の merge-base。`WORKFLOW_BASE_SHA` が空か HEAD と同じとき（`workflow_dispatch` の run は今の式で HEAD になる）は `origin/main` と HEAD の merge-base を使い、それも HEAD と同じなら lane の commit が無いので、active な packet は 0 件として通す（main の checkout と、最初の commit の前の新しい branch。今の `check-workflow-git.sh` の挙動と同じ）。ERROR は起点を解決できないときと、merge-base が一意でないときに限る。helper の `requirements`（`--packet` と packet なしの道の判定）、PK1・PK4・PK5、H2、`--target plan` の引数なしの対象は、全部この 1 つの定義を使う（本 packet で「active な packet」と書くときは全部この意味。PR の base が本 lane の merge の前の PR は、base の版の helper の今の取り方で判定される〈D-099〉）。

- merge 済みの packet（起点の tree にある packet）は active でない。squash merge のため、その `Plan Commit` は main の祖先にならず（起票の時点で `git merge-base --is-ancestor 7895aca82b11016d6a22c69ef30257e65d640db3 origin/main` → exit 1。`docs/plans/2026-10-27-docs-small-batch-3.md` の Plan Commit。`docs/plans/2026-10-23-stocktake-count-runtime-b2.md` の `33831dd3…` も exit 1）、今の取り方（差分が触る packet）では、後の PR がその packet を 1 行直すだけで PK5 が落ちる。
- merge 済みの packet は R0/R1 の道で直してよい（typo・link の直し）。ただし Workflow State の節の行を変える差分は ERROR にする（merge の時点の記録を書き換えない）。
- link の検査（R3）は、merge 済みの packet の link 切れを WARN にする（文書の改名・削除のたびに merge 済みの packet を直させない。直すのは任意で、上の R0/R1 の道で行う）。
- 起点が取れない・merge-base が一意でないときは、active な packet を決められないとして ERROR で止め、理由を名指す（merge 済みの packet が残るので「全 packet を検査する」は必ず落ち、fail-closed の代わりにならない）。
- 旧い closeout の PR（#215 の型）が packet を `docs/archive/plans/` へ移すのは受ける（H4）。移動（`docs/plans/` 側の削除と archive 側の追加）は Workflow State の行の変更として扱わない（Matrix AR5）。
- review のための copy は `git worktree add` で作る。`git archive` の copy には起点が無く、上のとおり ERROR になる（その ERROR は「起点が取れない」を名指す）。

### 計画の節（H6・H7 が比べる集合、1 か所）

計画の節 = `## Scope`・`## Acceptance Criteria`・`## Contract Ledger`（旧形式で `## Contract Ledger` が無ければ `## Spec Contract` と `## Trace Matrix`）。Risk が R4 なら `## Recovery` と `## Data Safety` も加える（旧形式の R4 は、在れば `## Data Safety` と rollback・recovery を書いた節〈見出しに `Recovery` か `Rollback` を含む節〉を加える）。比べるのは節の本文の byte。どれかの節が無い・重複して読めないときは「変わった」と扱う（fail-closed）。H6 と H7 はこの定義だけを引く。

### helper・検査・CI の変更（v2 §6 の番号）

- H1 `scripts/plan-approve.sh <packet> <SHA>` と `--amend <SHA>`: SHA が 40 桁の小文字 hex で branch の HEAD の祖先、packet の `Plan Commit` が `pending`（`--amend` では SHA が Plan Commit の子孫で `Amendments` に無い）を確かめ、さらに次の 2 つを確かめる（reviewer が読んだ版の後に、手元で実装や計画の変更を積んでから承認を書く道を止める）。(a) lane の commit だけを見る: `git log --first-parent --no-merges --name-only <SHA>..HEAD` の path が空か、その packet だけ。間にある merge の commit は、#217 の同期だけの条件の関数を満たすこと（base 同期だけ。#217 の merge の後に実装し、その関数を使う）。(b) `<SHA>` の版と HEAD の版で、その packet の本文が、承認の記録の行（Workflow State の `Plan Commit`・`Amendments`）を除いて byte で同じ。(c) 書き込む前に、対象の packet の index と作業 tree が HEAD の版と一致する（`git diff --quiet -- <packet>` と `git diff --cached --quiet -- <packet>` がどちらも exit 0）。一致しなければ何も変えずに拒む（未 commit の Scope の変更が承認の commit に入る道を止める）。満たせばその行だけを書き換えて packet だけの commit を作る。push はしない。
- H2 Phase を外す: `parse_packet` は Phase を必須にせず、旧い packet の Phase 行は受けて評価しない。`snapshot`・`nonci`・`status` の `phase == implementing` を `plan_commit != pending` に置き換える。`check-workflow-git.sh` の Phase の 8 値の検査を外し、新しい検査を足す: active な packet（上の定義）のどれかが `Plan Commit: pending` で、起点からの差分に classifier（branch の版の `scripts/ci/classify-changes.sh`）の `rust`・`frontend`・`generated`・`env`・`unknown` のどれかが true、または `scripts/**`・`.github/**` の path があれば ERROR。起点が取れなければ active を決められないとして ERROR（上の定義）。v2 §6 H2 の 4 種に `env` と workflow の実行 code の path を足した（workflow の lane の helper の先走りを止めるため。`workflow=true` 全体は docs も含むので使わない）。
- H3 `requirements` で Minimum と Human Gate を決める。旧形式の packet（`Phase` の行がある）は、書いた `Final Review Minimum` と `Human Gate` をそのまま使う（今の helper と同じ。R4・`workflow=true` で 2 を求める今の検査も残す。D-145 (15) の非遡及。TD-484 の 2 本の既定は新形式の packet の話）。新形式の packet は導く: Minimum は R3 = 2、R4 = 2、classifier `workflow=true` なら 2。packet に `Final Review Minimum` の行があれば導いた値との大きい方（下げる向きは受けない）。Human Gate は ready,merge を常に、r4 は Risk が R4 のとき、manual は `## L3` 節があるとき。どちらの形式でも、Workflow State に `Human Gate` の行があり `r4`・`manual` を含めば、導いた値に足す（書いた承認を落とさない。R3 + 明示の r4 は r4 を要る）。`approved packet condition changed` は、新形式では Risk と `## L3` 節の有無と（あれば）`Final Review Minimum`・`Human Gate` の行で照合し、旧形式は今の 3 field で照合する。
- H4 archive への移送をなくす: R0/R1 の道の `moved packet is not Phase archive` の検査を外す（`docs/archive/plans/` への移動は受け、削除だけ・archive の外への移動は今のまま拒む。v2 §6 H4 は移送の分岐ごと消す案で、本 packet は #215 の型を通しつつ `docs/plans/` の packet の黙った削除を止める形に絞った）。PK1・PK4・PK5 の対象を active な packet（上の定義）にし、merge 済みの packet の Workflow State の行の変更を ERROR に、merge 済みの packet の link 切れを WARN にする。PK4 の `docs/Plans.md` の `## 次の行動` の pointer の検査を外す。`--target plan PATH` の明示の検査は今のまま。
- H5 L3 の使い回し C: #217 が足す同期だけの条件の関数の隣に、「manual の版から今の head までの lane の commit（1 親）の path が、classifier で `docs` だけ（`workflow=false`）、`--no-renames` で列挙、packet の L3 の比較対象（上の「新しい packet の書式」の表の最後の行。旧形式で `## L3` が無ければ `## Test Plan` 節の全体）が byte で同じ」の条件を足し、#217 の A・B と組み合わせて manual を引き継ぐ。比較対象が取れなければ使い回さない。R4 は引き継がない。
- H6 `--review-stage docs`: H5 と同じ条件（L3 の比較対象を含む）で、旧 record の review が pass、broad の Plan 契約が今の packet と同じ、計画の節（上の定義）が byte で同じ、`--evidence` に P3 だけと裁定した pointer があるときだけ、`{head, base, sync_from}` の closure を書く（#217 の `--review-stage sync` と同じ形）。`workflow=true` の docs は受けない。Risk が R4 の packet は受けない（R4 の P3 の是正も人の closure に回す）。比較元と比較先の packet がどちらも manual なし（`## L3` の無い新形式の R3、または旧形式で `Human Gate` に manual が無いもの）なら、L3 の比較だけを対象外にする（manual の要否が版の間で変わったときは受けない）。
- H7 GA の後: broad の版と今の版で計画の節（上の定義。H6 と同じ集合）が byte で同じなら、`FRESH_BROAD` の代わりに closure を求める。`## Contract Ledger` だけが変わっても `FRESH_BROAD`。節が読めなければ今のまま `FRESH_BROAD`。
- H8 packet なしの道（R0〜R2）: `--risk R2` を新しく受け、Minimum 1・`plan_commit` null・`--manual` 必須（operator 向けの画面に触れるなら `required` で、目視を manual として記録する）。R0・R1 は今のとおり Minimum 0。packet なしの道（R0〜R2）で機械が拒むのは、migration（`src-tauri/src/db/migration.rs`・`src-tauri/src/db/migration_tx.rs`・`src-tauri/src/db/schema_v*.rs`）・依存（`package.json`・`package-lock.json`・`.npmrc`）の path、classifier の `unknown=true`、workflow の実行 code（classifier の `workflow=true` かつ `rust=true`。今の helper の「`workflow`かつ`rust`」の検査と同じもの）。`workflow=true` でも docs だけの変更（`AGENTS.md`・`CLAUDE.md`・`.agents/*`・`docs/templates/*`・`docs/code_review.md`・PR template の typo・link・意味を変えない直し）は、今のとおり R0/R1 で受ける（`docs/DEV_WORKFLOW.md` Risk Tiers の R0 の行のまま）。R2 では `workflow=true` を受けない（gate の文書の意味を変える変更は R3。R2 の行き先の線のまま）。拒むときは理由に path と「R3 の packet で起票する」を出す。それ以外の `rust`・`frontend` に触れる R2（契約に触れない表示・開発の道具）は packet なしで受ける。状態を持つ画面・契約に触れる変更を R2 にしないのは人の Risk の判断で、差分の review の発注の固定の問いに「Risk の判定は正しいか（状態を持つ画面・migration・依存の追加・契約の変更でないか）」を置く。PR body の Risk の行は `Risk: R<0|1|2>（目視: 要|不要）` の固定の書式（template に書く）で、helper は PR の JSON の body からこの行を 1 行だけ読み、Risk と目視の要否が `--risk`・`--manual` と食い違えば拒む（行が無い・2 行ある・書式が違うときも拒む）。diff が active な packet を足していれば packet なしの道を拒む（`--packet` の道へ）。
- H9 同じ head の CI を 1 回に: `merge_gate` の name を常に `Merge gate` にし、trigger から `ready_for_review` を外す。helper `ci()` は同じ head の最新の run が `pull_request`（Draft を含む）か `workflow_dispatch` で成功し `Merge gate` が 1 つなら受ける（今の照合は全部残す）。Ready の後の push を pre-push が拒む挙動は今のまま。
- H11 `record --report FILE`: FILE の `REVIEWED_HEAD=<40hex>` の 1 行を読み `--reviewed-head` に使う。capture の head との照合は今のまま。
- H12（`scripts/lanes.sh`）は作らない（YAGNI。再開は Coordinator が `gh pr list --state open` と、名指しの PR ごとに helper の status を明示の引数で引く。merge 済みの packet を active と取り違える型は checker と helper の active の定義が防ぐ）。
- H13 生きた docs に `path:行` を新しく足したら ERROR（起点からの差分の `+` 行だけ。既存の行は対象外）。D-125 (2)・(3) と `docs/DOC_STYLE_GUIDE.md` §1「code と文書の箇所の引き方」の適用除外をそのまま反映する: 対象は設計の正本・workflow の正本・runbook（`docs/**`、`AGENTS.md`、`CLAUDE.md`）。対象外は、`docs/archive/**`・`docs/plans/**`（dated の packet と Matrix）・`docs/research/**` と dated の調査・提案・L3 の記録・`docs/decision-log.md`（append-only の本文）、`docs/backlog.md` の行のうち同じ行に 7〜40 桁の hex の commit を添えたもの。起点は PK5 と同じ（hosted は docs job の step に渡す `WORKFLOW_BASE_SHA`、push 前は `scripts/pre-push.sh` が渡すか `origin/main`）。起点が取れなければ WARN と理由。#204 の検査（D-125 の目的）を、この縮めた形で置き換える。
- H14 PK1 の必須の節を上の「新しい packet の書式」の表にする（新形式と旧形式で分ける）。R3 の Data Safety の必須は新形式で外し、計画の review の発注の固定の問い（データの安全）へ移す。R4 は Data Safety と Recovery を必須に残す。旧形式は今の必須の節のまま。
- H10（`.local/codex-orders/new-order.sh`）は repo の外で本 PR の範囲の外（Non-scope）。

### 安全の境界ごとの扱い

D-098 が DEV_WORKFLOW Workflow State で挙げる境界、AGENTS の Safety・npm ガード、既存テストを弱めない規則について、項目ごとに「緩める / 同じ強さを別の形で守る / 強める / 変えない」を書く。

| 境界 | 本 PR の変更 | 扱い | 根拠 |
|---|---|---|---|
| 古い証拠の拒否 | H5（docs だけの lane の commit の後の L3 の使い回し）、H9（Draft の run を merge の証拠に）、H11（報告から reviewed head） | 同じ強さを別の形で守る。runtime・生成物・未知の path・L3 節の変更では使い回さない。CI は同じ head の run だけ。reviewed head は capture の head と照合 | Matrix C2・C3・CI2・RP1 |
| 独立 review | H3（Minimum の導出）は新形式の packet で強める（R3 は 2 本、TD-484）。旧形式の packet は書いた値のままで、走っている lane の本数は上がりも下がりもしない。H7（GA の後の closure）は同じ強さを別の形で守る（計画の節〈Scope・AC・Contract Ledger、R4 は Recovery・Data Safety も〉のどれかが変われば fresh broad）。H6（docs だけの commit の機械の closure）は **緩める**: lane が書いた P3 の docs の文が人の review を経ずに入る。範囲を TD-476 (2) に限り、gate の文書・計画の節の変更・R4・裁定の pointer の無いものは受けない | owner の決定 TD-476 (2)。Matrix RC2・RC4・GA2・GA4・MN2 |
| Plan Commit の固定 | H1（script が書く）は同じ強さ（PK5 と helper の照合は今のまま）。H8（R2 の packet なしの道）は **緩める**: R2 から Plan Commit の固定と計画の review を外す。機械が packet なしの道で止めるのは、R0〜R2 で migration・依存の path・`unknown=true`・workflow の実行 code（`workflow`かつ`rust`）、加えて R2 では `workflow=true`。`workflow=true` の docs だけの R0/R1（typo・link）は今のとおり受ける。R0/R1 で migration・依存・`unknown` を止めるのは今より**強める**（今の helper は R0/R1 で `workflow`かつ`rust` だけを止める）。ほかの runtime（`src/**`・`src-tauri/**`）の変更を R2 にするかは人の Risk の判断で、差分の review の固定の問い（Risk の判定は正しいか）で確かめる | owner の決定 TD-498（Q1 = A）。Matrix R2N2・R2N4 |
| 不可逆 finding の 4 項目 | 文を DEV_WORKFLOW の review の規則に残す | 変えない | — |
| fail-closed | H2 は強める（承認の前の実装を機械が止める。今は止めていない〈`check-workflow-git.sh` の冒頭の説明と `pr-gate.py` の `snapshot` を読んだ。pre-push と CI の全経路は未確認〉）。H4・H14 は同じ強さを別の形で守る（active な packet は今と同じに検査する。active を決める起点が取れなければ ERROR）。merge 済みの packet は今は archive へ移して検査の外に出しているので、検査の外に置くのは今と同じで、加えて Workflow State の行の変更を ERROR にする。壊れた packet・enum の外の Risk は今のまま止める | Matrix PH4・PH6・AR2・AR4・AR5 |
| owner の Human Gate・R4 の承認 | H3 で新形式は導く（manual は `## L3` 節、r4 は R4）。どちらの形式でも Workflow State に書いた `r4`・`manual` は落とさない（R3 + 明示の r4 は r4 を要る）。束ねた Ready/merge の承認は各 PR の helper `merge` を省かない | 同じ強さを別の形で守る | Matrix MN3・MN4・MN5 |
| R3/R4 の Data Safety の節（PK1 の機械の必須） | H14 で新形式の R3 から外し、計画の review の発注の固定の問い（データの安全）へ移す。R4 は必須に残す | R3 は **緩める**（機械の検査が発注の問いに替わる。店の実データを commit しない規則は AGENTS の Safety と `scripts/check-env-safety.sh`・public-sanitization の test が今のまま守る）。R4 は変えない | D-145 (10)。Matrix PK1N |
| bash の検査が branch の版で走ること | 本 PR の実装の commit の後、この PR の packet を新しい PK1・PK4・PK5 が見る | 手当て: 本 packet を今の形に保ち、最終 head で base の版の検査でも回す（AC9） | 冒頭の段落 |
| Decision and Approval Boundaries の承認境界 | 束ねた承認の委任を R0〜R4 に広げる（今は Wave の train の委任）。Codex の起動を Coordinator の委任に足す（TD-500。sandbox の設定は owner） | 委任の範囲を広げる（承認の要る操作は変えない） | owner の決定 TD-500、v2 §2.2 の「束ね」 |
| AGENTS の Safety・npm ガード | 文のまま | 変えない | — |
| 既存テストを弱めない | Phase・移送・Owner Effort Budget・`Draft (no merge evidence)` を前提にした case は新しい契約に合わせて書き換え、書き換えた case と理由を PR body に挙げる。拒否の case（古い head・broad の不足・base の版の helper・R4 の使い回し・UI merge の道・PK5）は消さない | 変えない | Matrix KEEP |

### この変更で不要になる記述・条件（D-127）

上の「正本の文の最終形」の「消す中身」の列と、helper の `parse_packet` の Phase の enum・`Final Review Minimum`・`Human Gate` の必須、`requirements` の `Phase archive` の検査、`check-workflow-git.sh` の Phase の検査、PK4 の Plans の pointer の検査、`ci.yml` の name 式の Draft の分岐、`pr-gate.py` の `mutate` が merge の後に返す案内 `docs closeout PR` と `requirements` の closeout を前提にした comment。消す語（`Owner Effort Budget`・`介入 N 回目`・`Post-Merge Closeout`・`直近の完了`・`Wave Registry`）が起票の時点で現れる生きた file（`rg -lF '<語>' --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' .`）:

| 語 | file |
|---|---|
| Owner Effort Budget | `docs/templates/plan-packet.md`・`scripts/doc-consistency-check.sh`・`scripts/tests/doc-consistency-plan-packet.test.sh`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/DEV_WORKFLOW.md`（ほかに `.agents/skills/inventory-workflow-start/SKILL.md` が節への link を持つ） |
| Post-Merge Closeout | `AGENTS.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/DEV_WORKFLOW.md` |
| 直近の完了 | `docs/backlog.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/Plans.md`・`docs/DEV_WORKFLOW.md` |
| Wave Registry | `scripts/tests/doc-consistency-plan-packet.test.sh`・`docs/DEV_WORKFLOW.md`・`docs/Plans.md`・`docs/backlog.md`・`docs/research/2026-07-28-codex-sol-week-playbook.md`（research は時点の調べで直さない） |
| 介入 N | `docs/AGENT_OPERATING_MANUAL.md`・`docs/research/2026-07-28-codex-sol-week-playbook.md`（同）・`docs/DEV_WORKFLOW.md`・`scripts/tests/doc-consistency-plan-packet.test.sh`・`docs/templates/plan-packet.md` |

残すもの: 不可逆 finding の 4 項目、Findings Freeze と round の上限 3、L3 Eligibility、fixture の実 encoding、Contract Audit の各項目（発注の問いとして）、one-writer と depth 1、単段の同期、relay の tag、`--pr-reviews N`、`--reviewed-head` の照合、base の版の helper、回答台帳。どれも今も防ぐ失敗が言える（v2 §2.4・§3）。

### 他の lane との所有と順

起票の時点（起点 `bbea3314`）の open の PR 13 本の変更 file（`gh pr diff <N> --name-only`）と本 lane の予定 path を突き合わせた。

| file | 相手 | 相手の所有 | 本 lane の所有 |
|---|---|---|---|
| `scripts/pr-gate.py`・`scripts/tests/pr-gate.test.py` | #217（helper の実装は plan-approved の後で、起票の時点の差分には出ない。#217 の packet の Scope の表） | 同期だけの条件・code に触れない条件・manual の A/B・`--review-stage sync`・確かめ直し（#217 の H1〜H7） | H2〜H9・H11。#217 の merge の後に、その上で書く |
| `docs/DEV_WORKFLOW.md`・`docs/agent-guidance/merge-evidence.md`・`docs/ci.md` | #217 | MG-D13 と L3 の箇条・`MG-D13` の引用 | #217 の merge の後の版から全体を書き換える（#217 の文は最終形の L3 と同期の節に残す） |
| `docs/decision-log.md` | #217（D-144）・#216（D-143）・#212（D-141）・#211（D-142）・#209（D-140）・#201（D-137）・#194（D-130 の改訂ほか） | 末尾への D の追記 | 末尾の D-145 と、置き換わる D の末尾の Superseded の行。末尾の追記は merge 順で両方を残す |
| `scripts/doc-consistency-check.sh`・`scripts/tests/doc-consistency-plan-packet.test.sh` | #201 | R0・R1 の走査に `src-tauri/dev-tools/src` を足す（`check_stale_parent_doc_references`・`check_code_doc_references`）と test の冒頭の comment | PK1・PK4 と新しい検査。関数が違うので、後から merge する側が base 同期で取り込む |
| `docs/Plans.md` | #215・#201・#190 | 各 lane の完了・関門の行 | 節の構造（現在のフェーズ・次の行動・直近の完了・Wave Registry を消す）。先に merge された相手の「直近の完了」の行は、本 lane の同期で節ごと消える（完了は git log に残る、TD-487）。関門・ブロッカーの行の中身は相手の版を保つ |
| `docs/backlog.md` | #215・#212・#211・#201・#190・#172 | 各項目の行 | `Wave Registry` の項目と、`Draft (no merge evidence)`・`ready_for_review` を引く項目（AC4 の `rg` が当てる行。起点の版で 2 項目）だけ |
| `docs/archive/harness-context/2026-09-30-Plans-completed.md` | #215 | 完了の行の移送 | 触らない（以後の追記をやめるだけ） |

- #217（`agent/l3-reuse-after-sync`、Plan Review round 2 中）と直列: 本 lane の helper の実装は #217 の merge の後、`origin/main` を単段で取り込んだ head の上で行う。本 lane の H5・H6 は #217 の同期だけの条件の関数と `--review-stage sync` の形を前提にする。起票（本 commit）は #217 と並行。
- #204（`agent/docs-file-line-check`、plan-approved、実装の前）は v2 §8.2 のとおり本 lane に畳む: D-125 の目的（行番号の引用を新しく作らない）は H13 の縮めた検査で保ち、#204 の packet の範囲（`.github/workflows/ci.yml` の 2 か所・新しい test・`docs/ci.md`・`docs/quality/review-checklist.md`・`docs/DOC_STYLE_GUIDE.md` の節）は本 lane では引き継がない。#204 を閉じる操作は本 lane でしない（Coordinator が owner の承認の範囲で行う）。
- #215（closeout）は今の形のまま merge してよい（AR1 で新しい helper も移送を受ける）。独立の closeout の PR はこれで最後になる。本 lane の merge の後の新しい helper で裁かれる packet なしの PR（R0/R1。#215 が本 lane より後に merge するときを含む）は、PR body に `Risk: R1（目視: 不要）` の形の行を足す（PR body の編集だけで、commit は要らない。H8 の照合）。
- 本 PR の後の同期: 本 PR は `scripts/pr-gate.py` ほかを変え、classifier が full（`rust=true`）を出すので、本 PR の merge の後に同期する lane は #217 の条件では「code に触れる同期」（B: 同期の closure と owner の 1 行）になる。順の方針: L3 待ちの lane（#209・#197・#210）は、できれば本 PR より先に merge する。先に merge できない lane は B で L3 を使い回す（L3 を取り直さない）。
- ほかの走っている lane（#216・#212・#211・#210・#209・#201・#197・#194・#190・#172）の packet は書き換えない。新しい helper・検査は旧形式の packet を今の規則（書いた Final Review Minimum・Human Gate、今の PK1 の必須の節）で読み、Phase の値は評価しない（PH3・MN2・MN3）。起票の時点で `Final Review Minimum: 1` の R3 の lane（#212・#211・#210・#209・#197・#172。`git grep -h -e '^- Risk:' -e '^- Final Review Minimum:' -e '^- Human Gate:' origin/agent/<branch> -- <packet>` を各 branch で回した。どれも `Human Gate: ready,merge,manual`。R4 の #216・#194・#190 は 2 と `r4`、#201 は R3 で 2。AC8 で実装の段に確かめ直す）も、同期の後に 2 本へ上がらない。H5〜H9 は、各 lane が base 同期して PR の base に新しい helper が入った後から効く（D-099）。

### 1 PR に収めるか

v2 §8.3 の分ける案（2a 機械と merge-evidence・ci.md → 2b 文）を見た。1 PR に収める。理由: owner が Q5 で 2 本に分ける案（C）を選ばず B を選んだ（TD-502）。機械と文を同じ PR に入れれば規則と helper が食い違う期間が無い（v2 §8.1）。2a を先にすると、その間は文が今の規則を求め続ける。分ける条件: Plan Review が round 3 に届き、残る findings が文と機械の片方に集まるとき、Coordinator が `docs/DEV_WORKFLOW.md` Review Rules の round の天井の disposition（owner escalation）で 2a・2b への分割を owner に諮る。

## Non-scope

- 製品の runtime・画面・DB・bindings・traceability の生成物。
- repo の外の運用（`.local/**`、v2 §8.1 の段 0）と owner の個人の設定。
- GitHub の ruleset と `.github/policies/**`（required check の名は変えない）。
- 走っている lane の packet の書き換え、archive の packet・WER の書き換え。
- #204 の packet の範囲（上の「他の lane との所有と順」）。
- `scripts/ci/classify-changes.sh` の分類の変更。

## Acceptance Criteria

- AC1（helper）: `PYTHONDONTWRITEBYTECODE=1 python3 scripts/tests/pr-gate.test.py` が exit 0 で、Matrix の PH1〜PH3・PH7・MN1〜MN5・AR1・AR4・C1〜C5・RC1〜RC4・GA1〜GA4・R2N1〜R2N5・RC1b・CI2・RP1・RP2 と KEEP の helper の case が通る。
- AC2（計画の固定と先走りの検査）: `bash scripts/tests/plan-approve.test.sh` と `bash scripts/tests/workflow-git-checks.test.sh` が exit 0 で、Matrix の PA1〜PA5・PH4〜PH6・PH8・PH9・PH10・AR5 が通る。
- AC3（packet の検査）: `bash scripts/tests/doc-consistency-plan-packet.test.sh` が exit 0 で、Matrix の AR2・AR3・AR6・FL1〜FL3・PK1N・PH7 が通る。
- AC4（CI）: `bash scripts/tests/ci-workflow.test.sh` が exit 0 で Matrix の CI1・CI4 が通る。`rg -n "Draft \(no merge evidence\)|ready_for_review" .github docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' --glob '!docs/research/**'` が 0 件（`docs/research/**` は時点の調べで直さない。`docs/DEV_SETUP_CHECKLIST.md` と `docs/backlog.md` の当たりは Scope の drift の sweep の行で直す）。`ci-workflow.test.sh` は docs job の doc check の step が `WORKFLOW_BASE_SHA` を持つことも確かめる（CI4）。
- AC5（欠番）: `scripts/lanes.sh` は作らない（H12）。番号を詰めない（ほかの AC の参照を変えないため）。
- AC6（回帰）: `bash scripts/tests/run-workflow-tests.sh` が exit 0。`bash scripts/check-workflow-git.sh` が exit 0。Draft の hosted の run（classifier `workflow=true` で full）が緑。
- AC7（文書）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-28-process-rebuild.md` が ERROR 0。`rg -n -e 'Owner Effort Budget' -e '介入 N 回目' -e 'Post-Merge Closeout' -e '直近の完了' -e 'Wave Registry' AGENTS.md CLAUDE.md docs .github .agents --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' --glob '!docs/research/**'` が 0 件（本 packet と旧い packet は `docs/plans/**` の除外で外れる。`scripts/tests/**` は旧い packet の受理の fixture に語が残るので対象にしない）。
- AC8（同じ入力を全部の読み手に）: (a) 新形式: 実装の後の branch の一時の worktree に、上の「新しい packet の書式」の正例を埋めた合成の packet（R3・manual なし、R3・`## L3` あり、R4 の 3 本。`plan-approve.sh` のある branch で足す）を 1 本ずつ置いて commit し、`bash scripts/doc-consistency-check.sh --target plan <packet>`（PK1・PK4）が ERROR 0、`WORKFLOW_BASE_SHA=<置く前の commit> bash scripts/check-workflow-git.sh`（PK5・H2）が exit 0、helper の `parse_packet` と合成の PR の JSON での `requirements` の結果（Minimum・manual・r4）が H3 の期待と一致する。負の例: 新形式の R3 で `## Contract Ledger` を欠く packet は PK1 が ERROR、新形式（`plan-approve.sh` のある親から足した）なのに `Phase` の行を持つ packet（file 名の日付を古くしたものを含む）と新形式の R2 の packet は PK4 が ERROR。(b) 旧形式: 最終 head の時点で open の PR のうち、旧形式の packet を持つもの全部（起票の時点では #216・#212・#211・#210・#209・#201・#197・#194・#190・#172。最終 head で `gh pr list --state open` から引き直す）ごとに、その branch の head の一時の worktree（`git worktree add <tmp> origin/<branch>`）に実装後の本 branch の `scripts/` を重ね（`git -C <tmp> checkout <本 branch の head> -- scripts`）、`WORKFLOW_BASE_SHA=$(git merge-base origin/main origin/<branch>) bash scripts/check-workflow-git.sh`（PK5・H2、lane の履歴で）が exit 0、`bash scripts/doc-consistency-check.sh --target plan <その lane の packet>` が ERROR 0、同じ worktree で helper の `parse_packet` と合成の PR の JSON での `requirements` が旧形式と判定し、Minimum・manual・r4 が packet に書いた値と一致する（全部の読み手を各 lane の履歴の worktree で確かめる。packet を本 branch に写すと最初の add の commit が本 branch の commit になり形式の判定が変わるので、写す形は使わない）。一時の worktree は確かめた後に消す。command と出力を PR body に貼る。
- AC9（自分の検査で自分を緩めない）: 最終 head で `git worktree add <tmp> HEAD` → `git -C <tmp> checkout <PR の base の SHA> -- scripts` → `bash <tmp>/scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-28-process-rebuild.md`（`<tmp>` を cwd にして）の ERROR は、base の版の PK4 が出す `PK4: docs/Plans.md の '## 次の行動' に active packet の一覧（docs/plans/）を指す行がありません` の 1 件だけ（本 PR が `## 次の行動` と、その検査を消すので期待どおり）で、それ以外の ERROR は 0。同じ worktree で `bash scripts/check-workflow-git.sh` が exit 0。一時の worktree は確かめた後に消す（`git worktree remove <tmp>`）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/DEV_WORKFLOW.md` の `## Risk Tiers`・`## Plan Packet Rules`・`## Workflow State`・`## Wave Operation`・`## Owner Effort Budget`・`## Verification Gates`・`## Review Rules`・`## Contract Audit (R3/R4)`・`## Draft PR Checkpoint`・`## Post-Merge Closeout`。`docs/AGENT_OPERATING_MANUAL.md` の `## 座組`・`### 3.3 Capacity-degraded（役割担当の一時不能）`・`### 5.3 Plans.md cleanup prompt`。`docs/agent-guidance/merge-evidence.md` の `## 設計判断`（MG-D3・MG-D9）と #217 の MG-D13。`docs/ci.md` の `## Hosted Trigger Model`・`### Final Trigger Selection`。`AGENTS.md` の `## Session Start`・`## Decision and Approval Boundaries`。`docs/decision-log.md` の D-034・D-038・D-039・D-055・D-062・D-085・D-097・D-098・D-099・D-101・D-107・D-124・D-125・D-127 と #217 の D-144。
- 必要な設計成果物: workflow gate change → `docs/decision-log.md` の D-145 = updated in this PR（本 commit）。各正本の書き換え = updated in this PR（実装の段）。中身は上の「正本の文の最終形」と H1〜H14。
- 本 commit で正本の文を書き換えない理由: 書き換える文の多くは、同じ branch の検査（PK1・PK4・`reading-order-drift.test.sh`・`doc-consistency-plan-packet.test.sh`）が形を確かめている。文だけを先に変えると、branch の pre-push と hosted の docs・workflow の job が古い検査で落ちるか、検査を先に変えると plan-first の commit に実装が入る。文と検査と helper を同じ実装の段で変え、Plan Review には上の表と H1〜H14 の粒度で計画を読ませる（v2 §8.3 の「文の変更を計画の commit に入れる」案は取らない）。
- plan にしかない durable な判断の昇格先: 作り直しの決定は D-145。各 H の受理条件の細部は実装の段で merge-evidence に置く（helper の契約の正本）。
- 前提・制約と延期した design gap: (1) #217 の merge が先（H5・H6 の前提）。#217 の形が round 2 以降で変われば、本 lane は Gated Amendment で H5・H6 を合わせる。(2) H9 は MG-D3 の「Draft はマージ根拠となる check を発行しない」を置き換える（下の Contract Probe）。(3) 計画の review の発注の固定の問いが、packet の節と同じだけ拾うかは測れない（v2 §10.1 の 3）。作り直しの後の 3 lane で、問いが拾った finding の数を handoff に残す（Coordinator、repo の外）。
- 絶対保証の自己点検: 「計画の承認の前の実装を機械が止める」（H2）の例外は、(1) branch の版の classifier を使うので、同じ branch で classifier を書き換えると逃れうる（`scripts/**` の path の検査が別に止める）、(2) `--no-verify` の push（hosted の docs job の同じ検査が止める）。「旧い packet を拒まない」の例外は、旧い packet の Plan Commit・Amendments・Risk の形が壊れているとき（今と同じに止まる、fail-closed）。
- 判定: ready（決めきれなかった点は Review Focus に挙げる）。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| source / workflow doc 新設・改名・削除 | 親文書の目次・索引の更新 | 新しい文書は作らない。`docs/DEV_WORKFLOW.md` の Source Index・Artifact Map を最終形に合わせる |
| scripts の新設 | `scripts/tests/run-workflow-tests.sh` に新しい test を登録 | `plan-approve.test.sh` |
| Tauri command・function-design・REQ・route・画面 | — | 該当なし |

## Impact Review Lenses

not applicable: 実機・外部の tool・POS・CSV・operator の発見から始まる作業でない。owner の決定と repo の履歴の実測（v2 §4.1 の頻度の command）から始まり、GitHub Actions の挙動は下の Contract Probe で確かめた。

## Boundary / Wire Contract

- producer: `scripts/pr-gate.py` の `record`（RecordV1 の JSON）、`scripts/plan-approve.sh`（packet の `Plan Commit`・`Amendments` の行）、reviewer の報告（`REVIEWED_HEAD=` の行）。
- consumer: 同じ helper（PR の base の版、D-099）、`check-workflow-git.sh`（PK5・先走りの検査）、`doc-consistency-check.sh`（PK1・PK4）。
- wire type: RecordV1 は `version` 1 のまま。closure の形は #217 の `{head, base, audit}` と `{head, base, sync_from}` だけ（H6 は `sync_from` の形を使い、新しい形を足さない）。packet の Workflow State は `- Key: value` の行（Plan Commit・Amendments・Risk の 3 つを読む。旧い行は受けて評価しない）。
- internal type: 同じ dict。
- precision/range: SHA は 40 桁の小文字 hex。
- round-trip path: `plan-approve.sh` → packet → helper `parse_packet` → `capture` → `record`。
- invalid input: 短い SHA・祖先でない SHA（H1）、`REVIEWED_HEAD=` の欠け・重複（H11）。
- compatibility: 旧い packet（PH3）と今の record の形を受ける。新しい形の packet（Phase の無い）を今の main の helper は `packet fields missing/empty/duplicate` で拒むが、helper は PR の base の版だけが動き、本 lane の merge の後の base でしか新しい形の packet を判定しないので混ざらない。本 lane の merge の前に新しい形の packet を起票しない（本 packet は今の形）。

## Test Plan

Test Design Matrix: [test-matrices/2026-10-28-process-rebuild.md](test-matrices/2026-10-28-process-rebuild.md)

- targeted tests: AC1〜AC5 の command。
- negative tests: Matrix の PA2・PH1・PH4・PH6・MN2・MN4・AR1・C2〜C4・RC2・GA2・R2N2・CI2・RP2・FL2。
- compatibility checks: PH3・MN2・MN3・MN4・AR1・AC8。
- data safety checks: 合成の git repo・fake gh・合成の packet だけ。店のデータを使わない。
- main wiring/integration checks: Matrix の Main Wiring の 4 行。
- workflow: `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/check-workflow-git.sh`、`bash scripts/doc-consistency-check.sh`、`--target plan`。Draft の hosted（full）。

## Review Focus

- H2 の先走りの検査の path の集合（v2 の 4 種に `env` と `scripts/**`・`.github/**` を足した）が、plan-first の commit（workflow の lane は正本の文を含みうる）を止めず、実装を全部止めるか。
- H4 で R0/R1 の移送の分岐を消さずに `Phase archive` の検査だけを外した形（v2 §6 H4 との違い）が妥当か。
- H6 の緩める範囲（docs だけ・`workflow=false`・計画の節が同じ・裁定の pointer）が TD-476 (2) に収まるか。gate の文書の P3 の是正も人の closure に回す形でよいか。
- H9 で MG-D3 を置き換えて、Draft の run の `Merge gate` を merge の証拠にしても、exact head の強さが同じか（下の Contract Probe）。
- H13 の起点が無いときを WARN にした形（v2 §6 H13 は ERROR の条件だけを書く）が、既存の 231 か所（`git grep -cE '[A-Za-z0-9_./-]+\.(md|rs|ts|tsx|py|sh):[0-9]+' origin/main -- docs ':!docs/archive'` の合計、起点 `bbea3314` で 231）を一斉に止めずに新しい引用を止めるか。
- 旧形式の packet の Minimum・Human Gate を書いた値のまま使う形で、走っている lane の条件が上がりも下がりもしないか。新形式で書いた `r4`・`manual` を落とさないか。
- active な packet の定義（起点の tree に無い dated packet）が、helper・PK1・PK4・PK5・H2 で 1 つにそろい、merge 済みの packet を直す R0 の PR と、走っている lane の packet の検査の両方を成り立たせるか。
- 新しい packet の書式の正例と旧形式の対応の表で、同じ packet を PK1・PK4・PK5・helper が同じに読むか（AC8）。
- 本 commit で正本の文を書き換えない判断（Design Readiness）。
- この変更で不要になる記述・条件は他に無いか（D-127）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| D-145 (1) 人は判断だけを書く | D-145 | 正本の文の最終形 | AC7 | 非対象 |
| D-145 (2) Plan Commit を script で固定 | DEV_WORKFLOW Plan Commit ancestry（PK5） | H1 | PA1〜PA5 | 非対象 |
| D-145 (3) Phase を外し、承認の前の実装を止める | DEV_WORKFLOW Workflow State（fail-closed） | H2 | PH1〜PH6 | 非対象 |
| D-145 (4) Minimum と Human Gate の導出 | DEV_WORKFLOW Workflow State（`Final Review Minimum`・`Human Gate`）、TD-484 | H3 | MN1〜MN5・PH7 | 非対象 |
| D-145 (5) archive へ移さない、closeout をなくす | DEV_WORKFLOW Post-Merge Closeout、merge-evidence MG-D9、D-101 | H4 | AR1〜AR6 | 非対象 |
| D-145 (6) L3 の使い回し C と docs の closure | merge-evidence MG-D13（#217）、DEV_WORKFLOW Wave Operation の L3 の箇条 | H5・H6 | C1〜C5・RC1〜RC4 | 非対象 |
| D-145 (7) GA の後の closure | merge-evidence の `FRESH_BROAD` の文、DEV_WORKFLOW Review Rules（Findings Freeze） | H7 | GA1〜GA4 | 非対象 |
| D-145 (8) R2 の packet なしの道 | DEV_WORKFLOW Risk Tiers・Plan Packet Rules、TD-498 | H8 | R2N1〜R2N4 | 非対象 |
| D-145 (9) 同じ head の CI を 1 回 | docs/ci.md Hosted Trigger Model・Final Trigger Selection、merge-evidence MG-D3 | H9 | CI1〜CI4 | 非対象 |
| D-145 (10) reviewed head を報告から | merge-evidence の実行手順 | H11 | RP1・RP2 | 非対象 |
| D-145 (5) active な packet の定義（helper・PK1・PK4・PK5・H2 で 1 つ） | DEV_WORKFLOW Plan Commit ancestry（PK5 の対象）、D-107 | H2・H4 と「active な packet の定義」 | AR4〜AR6・PH6 | 非対象 |
| D-145 (10) lane の現在地（`lanes.sh` は作らず、`gh pr list` と helper の status で引く） | AGENTS Session Start | 文 | AC7 | 非対象 |
| D-145 (10) 新しい `path:行` を足さない | D-125 | H13 | FL1〜FL3 | 非対象 |
| D-145 (10) packet の最小形 | DEV_WORKFLOW Plan Packet Rules、template | H14 と「新しい packet の書式」 | PK1N・AC8 | 非対象 |
| D-145 (11) Owner Effort Budget を消す | DEV_WORKFLOW Owner Effort Budget、D-038、TD-501 | 文と PK1 の必須の節 | PK1N・AC7 | 非対象 |
| D-145 (12) Plans は関門・ブロッカー・製品の決まっていない判断だけ | DEV_WORKFLOW Artifact Map、D-101、TD-487 | `docs/Plans.md` と PK4 | AR3・AC7 | 非対象 |
| D-145 (13) Codex が使えない間の Final | MANUAL §3.3、TD-499 | 文 | AC7 | 非対象 |
| D-145 (14) Codex の起動と束ねた承認の委任 | AGENTS Decision and Approval Boundaries、TD-500 | 文 | AC7 | 非対象 |
| 隣接（変えない）: base の版の helper が判定 | merge-evidence、D-099 | 変えない | KEEP | 非対象 |
| 隣接（変えない）: 古い head/base の record を拒む・broad の不足を拒む・R4 を使い回さない | merge-evidence の状態と非CI記録 | 変えない | KEEP | 非対象 |
| 隣接（変えない）: 不可逆 finding の 4 項目・Findings Freeze・round の上限 3 | DEV_WORKFLOW Review Rules | 文を残す | AC7（消えていないことを `rg` で確かめる） | 非対象 |

## Contract Probe

- Draft と Ready で選ぶ job が同じか（H9 の前提、v2 §10.2）: `rg -n 'draft|event_name' .github/workflows/ci.yml` → `draft` を参照するのは `merge_gate` の name 式の 1 か所だけで、ほかは `changes` の step の `workflow_dispatch` の分岐（dispatch は常に full、`docs/ci.md` CI-TRIGGER-D1）。`rg -n 'draft' scripts/ci/classify-changes.sh scripts/ci/check-required-jobs.sh` → 0 件。よって `pull_request` の event では Draft と Ready で同じ job を選ぶ。
- MG-D3 の理由（H9 で置き換える物）: `docs/agent-guidance/merge-evidence.md` の `## 設計判断` の MG-D3 は「docs-onlyにも軽いCIを実行する。Draftはマージ根拠となるcheckを発行しない」で、棄却した案の列は paths-ignore と skip token と全 docs の full。Draft の check を根拠にしない理由は列に書かれていない。helper は Ready・merge の時点で同じ head の run を照合する（`pr-gate.py` の `ci`）ので、Draft の run の `Merge gate` を受けても別の head の証拠は通らない（Matrix CI2）。
- 計画の承認の前の実装を今の機械が止めていないこと（H2 の「強める」の根拠）: `scripts/check-workflow-git.sh` の冒頭の説明は Phase の 8 値と PK5（Plan Commit が pending の packet は対象外）だけを挙げる。`pr-gate.py` の `snapshot` は capture で `phase == implementing` を求めるが、push は止めない。pre-push と hosted の全経路は未確認。
- 走っている lane の旧い packet の形: 起票の時点の open の PR の packet は全部 Workflow State の 11 行の形（#217・#204 の packet を `git show origin/<branch>:<packet>` で読み、10 本の lane の `Risk`・`Final Review Minimum`・`Human Gate` の行を `git grep -h` で読んだ〈上の「他の lane との所有と順」〉。全文の受理は AC8 で実装の段に確かめる）。
- merge 済みの packet の Plan Commit は main の祖先でない（active な packet の定義の前提）: `git merge-base --is-ancestor 7895aca82b11016d6a22c69ef30257e65d640db3 origin/main` → exit 1、`git merge-base --is-ancestor 33831dd3c01266b0b458c0d8c8d189ad9895672c origin/main` → exit 1（`docs/plans/` に残る #214・#193 の packet の Plan Commit。`origin/main` = `bbea3314`）。

## Data Safety

- commit しないもの: 回答台帳の原文（TD の番号で引く）、record の中身、店のデータ、`.local/**`。
- local だけ: `.local/pr-gate/` の capture（今のまま ignored）、v2 の設計案。
- 合成だけ: test の git repo・fake gh の state・合成の packet。

## Implementation Results

実装は 3 つの commit（機械と test → 正本の文 → drift の sweep と Superseded の行）。SHA と test の本数は写さない（Evidence Ownership）。PR と CI の出力が正本。

### H ごとの実装

- H1: `scripts/plan-approve.sh`（新設）。SHA の形と祖先、`pending`（`--amend` では Plan Commit の子孫かつ重複なし）、(a) SHA の後の lane の commit が packet だけで間の merge が #217 の `Gate.sync_merge` を満たす、(b) 記録の行を除いて packet の本文が同じ、(c) packet に未 stage・stage 済みの変更が無い、を確かめて packet だけの commit を作る。test は `scripts/tests/plan-approve.test.sh`（PA1〜PA5）。
- H2: `parse_packet` から Phase の enum と必須を外し、`snapshot`・`nonci`・`status` は `plan_commit` で計画の前後を決める。`check-workflow-git.sh` は Phase の検査を外し、先走りの検査と、merge 済みの packet の Workflow State の書換えの ERROR を足した。active な packet の定義は `pr-gate.py` の `plan_origin`・`active_packets` の 1 か所。
- H3: `parse_packet(text, form)` と `packet_form`（形式）・`gate_conditions`（承認の後の照合）。
- H4: helper の active を差分の added・renamed・copied の dated packet にし、`Phase archive` の検査を外した。PK1・PK4 の対象を active にし、Plans の pointer の検査を消した。merge 済みの packet の link 切れは WARN。
- H5・H6・H7: `sync_merges(docs=True)`・`l3_basis`・`l3_change`・`plan_sections`・`plan_change`・`plan_same`、`--review-stage docs`。機械の closure（sync・docs）は Gated Amendment の後を受けない（GA は監査の closure）。
- H8: `--risk R2`、`body_risk`（PR body の Risk の行）、migration・依存・`unknown`・workflow かつ rust の拒否、R2 の `workflow=true` の拒否。
- H9: `ci.yml` の aggregate の名を `Merge gate` に固定し `ready_for_review` を外した。docs job の doc check に `WORKFLOW_BASE_SHA`。helper の `ci` は変えない（Draft の run を既に受ける作り）。
- H11: `record --report`（`report_head`）。H13: doc check の FL。H14: PK1 の新形式の必須の節。

### mutant

clean な tree に 1 つずつ注入して戻した 12 個が全部、対応する test を red にした: PH4 の pending の判定の反転、MN2 の max を min に、C2 の L3 の比較を外す、RC2 で workflow の docs を通す、GA2 で読めない節を通す、AR1 の削除の拒否を外す、RP1 で報告の head を落とす、PA3 の Amendments を前に足す、PA4 の本文の比較を外す、R2N5 の body の照合を外す、MN3 の L3 節の検出を外す、PH7 の形式の照合を外す。

### 実装で決めた点（計画の文から離れた所と段取り）

- shallow の判定: packet の文の「`git rev-parse --is-shallow-repository` が true なら ERROR」は採らず、packet を最初に足した commit が shallow の境か（graph の親と raw の parent 行を比べる。MG-D4 と同じ考え方）で判定する。owner の作業 repo は無関係な ref の shallow の印を持ち、文のとおりでは local の検査が全部 ERROR になるため。親の無い commit と shallow の境は ERROR のまま（test で固定）。
- `ci.yml` の workflow job の checkout に `fetch-depth: 0` を足した（計画の ci.yml の変更の外）。suite の実 repo の self-pass が active な packet の起点を要り、深さ 1 の checkout では起点が取れないため。ci-workflow.test で固定した。
- `--target plan` で名指しした active でない packet（merge 済み・archive）は、Phase の行の有無で形式を決め、git の履歴との照合をしない。
- `--review-stage docs` の `--evidence` は 1 本を必須にし、RecordV1 を変えないため record には載せない。
- 計画の節と L3 の比較対象の byte の比較は、節の末尾の空行を除く（後ろに節を足すだけで前の節が変わったと出さないため）。
- D-145 の文の closure の P3 2 件は直した。本 packet の「安全の境界ごとの扱い」の表に同じ「migration・依存・unknown を強める」の語が残るが、Scope を変えないため直していない。
- 段取り: pre-push は変えない（起点は `origin/main` に落ちる）。引数なしの `--target plan` で active な packet が 0 件なら今の exit 2 のまま。計画の固定の commit の件名は「docs(plans): Plan Commit を <SHA の先頭 12 桁> に固定する」。

### AC8 の結果

- (a) 新形式: 本 branch の一時の worktree で、R3・manual なし、R3・`## L3` あり、R4 の合成の packet が PK1・PK4（`--target plan`）で ERROR なし、`check-workflow-git.sh` で exit 0、helper の `parse_packet` と合成の PR の `requirements` で Minimum 2 と manual・r4 が H3 の期待どおり。負の例（R3 で Contract Ledger の欠け、新形式で Phase の行、file 名の日付を古くした Phase の行、新形式の R2）は PK1・PK4 と helper が拒んだ。
- (b) 旧形式: 最終 head の時点の open の PR のうち packet を持つ #216・#212・#211・#210・#209・#204・#201・#197・#194・#190・#172 の全部で、各 branch の head の一時の worktree に本 branch の `scripts/` を重ね、`check-workflow-git.sh` が exit 0、`--target plan` が ERROR なし、helper が旧形式と判定し、Minimum・manual・r4 が packet に書いた値と一致した（#215 は active な packet を持たない）。一時の worktree は消した。

### 既知の制限

- 旧形式の R4 で復旧の見出し（`Recovery` か `Rollback` を含む節）を持たない #194・#190 は、計画の節が Scope・Acceptance Criteria・Contract Ledger・Data Safety だけになる（#216 は `Rollback / Recovery（R4）` の節を含む）。見出しの無い復旧の文がこれらの節の外にあれば、H7 はそれが変わっても fresh broad でなく closure を求める（closure も独立 review なので review を飛ばす経路にはならない。H6 は R4 を受けない）。規則は packet の文のとおりで変えない（Coordinator の裁定）。

## Review Response

### Plan Review round 1（起点 `190fed4d26c1e5ec871c30f693ae8b4b2f4f6dd1`、判定 reject）

Codex（P1 1・P2 3）と fresh Opus 5.5（P2 5・P3 8）の findings を Coordinator が裁定し、下の表の全件を受理した（受理しなかったもの: なし）。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| X1 | Codex #1 | P1 | 導く `r4` を「Risk が R4、または Workflow State の `Human Gate` に `r4` がある」にし、書いた `manual` も落とさない。R3 + 明示の r4 を拒む case を足す | packet H3・安全の境界の表（Human Gate の行）、D-145 Decision (4)、Matrix MN3・MN4・MN5 |
| X2 | Codex #2 | P2 | H5・H6 の L3 の比較対象を、旧形式で `## L3` が無ければ `## Test Plan` 節の全体にし、取れなければ使い回さない。旧形式の基準の変更を拒む case を足す（#210 の型） | packet「新しい packet の書式」の表の最後の行・H5・H6、Matrix C5・RC3 |
| X3 | Codex #3、Opus F6 | P2 / P3 | 新しい packet の書式を Workflow State 内の `- Key: value` の 3 行に統一し、正例 1 本と旧形式との対応の表を置く（見出しは今の英語の名を使い、旧い節名の対応を少なくした）。同じ入力を PK1・PK4・PK5・helper に通す検証を AC8 にした | packet「新しい packet の書式」・H14・template の行・AC8・Review Focus、D-145 Decision (10)、Matrix FMT1・PK1N |
| X4 | Codex #4 | P2 | H13・FL1 に D-125 (2)・(3) と `docs/DOC_STYLE_GUIDE.md` §1 の適用除外（commit を添えた backlog の行、dated の記録、decision-log の本文、dated の packet と Matrix）を反映し、正例を足す | packet H13、Matrix FL1・FL3 |
| X5 | Opus F1 | P2 | 「active な packet」を「起点の tree に無い dated packet」と 1 か所で定義し、helper・PK1・PK4・PK5・H2 をそろえる。merge 済みの packet は R0/R1 の道で直せる（Workflow State の行の変更は ERROR）、link 切れは WARN、起点が取れなければ ERROR | packet「active な packet の定義」・H2・H4・安全の境界の表（fail-closed）・Contract Ledger・Contract Probe、D-145 Decision (5)、Matrix AR4・AR5・AR6・PH6 |
| X6 | Opus F2 | P2 | 旧形式の packet は書いた `Final Review Minimum`・`Human Gate` をそのまま使い、新形式だけで導く（非遡及。TD-484 は新しい既定） | packet H3・安全の境界の表（独立 review）・他の lane との所有と順・Review Focus、D-145 Decision (4)、Matrix MN2 |
| X7 | Opus F3 | P2 | AC4 の `rg` から `docs/research/**` を除外し、`docs/DEV_SETUP_CHECKLIST.md` と `docs/backlog.md` の該当の行を drift の sweep の行に足す | packet AC4・正本の文の最終形の最後の行 |
| X8 | Opus F4 | P2 | H7 の比べる節を H6 と同じ「計画の節」（Scope・Acceptance Criteria・Contract Ledger）にし、定義を 1 か所に置く | packet「計画の節」・H6・H7・安全の境界の表（独立 review）、D-145 Decision (7)、Matrix GA3 |
| X9 | Opus F5 | P2 | R2 の行き先を 1 文で決める: `workflow=true` か `unknown=true` の変更は R3 として起票、`rust`・`frontend` の R2 は packet なし | packet Goal・H8、D-145 Decision (8)、Matrix R2N2 |
| X10 | Opus F7 | P3 | `lanes.sh` は PR ごとに base の版の helper で status を取り、packet の無い PR は PR body から `--risk`・`--manual` を補い、exit 2 は理由を出して飛ばす | packet H12、Matrix LN2 |
| X11 | Opus F8 | P3 | `ci.yml` の docs job の doc check の step に `WORKFLOW_BASE_SHA` を渡す行を Scope に足し、test で確かめる | packet 予定の file（`ci.yml` の行）・H13・AC4、Matrix CI4 |
| X12 | Opus F9 | P3 | `plan-approve.sh` に「reviewer の読んだ SHA から HEAD までの差分が空か packet だけ」の検査を足す | packet H1、Matrix PA4 |
| X13 | Opus F10 | P3 | R2 の画面の目視を `--manual required` と目視の evidence の manual の記録に結び付ける | packet Ordinary Operation の R2 の行・H8、Matrix R2N3 |
| X14 | Opus F11 | P3 | Superseded の列に D-107 を足す | packet 予定の file（decision-log の行）、D-145 Compatibility |
| X15 | Opus F12 | P3 | PK1 の `Data Safety` の必須を、新形式の R3 では発注の固定の問いへ移し（緩める）、R4 は残すと決めて安全の境界の表に載せる | packet 安全の境界の表・H14・template の行、Matrix PK1N |
| X16 | Opus F13 | P3 | 冒頭の「今の規則で裁く」を「helper は base の版、bash の検査は branch の版」と書き直し、本 packet を今の形に保つことと base の版の検査でも回すこと（AC9）を手当てにした | packet 冒頭の段落・安全の境界の表・AC9 |

### Plan Review round 2（起点 `2c03b64ba6b4dc59ae5cb41da161bc5fa91b85be`、判定 reject）

TD-502 の形で両方に出した（round 1 で両方が P1/P2 を出したため）。Codex（P1 2・P2 4・P3 1）と fresh Opus 5.5（P2 7・P3 9）の findings を Coordinator が裁定し、下の表の全件を受理した（受理しなかったもの: なし）。方針は D-127 と owner の軸（TD-489・TD-490）: 新しい仕組みの継ぎ目に当たる指摘は、継ぎ足しより仕組みを消せないかを先に見た（Y11 で `scripts/lanes.sh` を消した）。次の round 3 が天井。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| Y1 | Codex #1・Opus F4 | P1 / P2 | `plan-approve.sh` は reviewer の読んだ SHA と HEAD の packet の本文を、承認の記録の行（`Plan Commit`・`Amendments`）を除いて byte で比べる | packet H1、Matrix PA4 |
| Y2 | Codex #2 | P1 | 旧形式の L3 の比較対象に、`## Test Plan` に加えて packet が link する Matrix の file の全体を含め、link が特定できなければ使い回さない（#211 の型） | packet「新しい packet の書式」の表の最後の行、Matrix C5・RC3 |
| Y3 | Opus F5 | P2 | `plan-approve.sh` の承認の後の変更の検査は lane の commit だけ（`git log --first-parent --no-merges <SHA>..HEAD`）を見て、間の merge は #217 の同期だけの条件の関数で確かめる | packet H1・Ordinary Operation の同期の行、Matrix PA5 |
| Y4 | Opus F1 | P2 | AC9 は base の版の PK4 が出す `## 次の行動` の pointer の ERROR 1 件だけを名指しで期待どおりとし、それ以外の ERROR 0 を求める | packet AC9 |
| Y5 | Codex #4・Opus F2 | P2 | AC8 の旧形式の互換の検査は、各 lane の branch の head の一時の worktree に新しい `scripts/` を重ねて PK5・H2 を回す。本 branch に写す形は PK1・PK4・`parse_packet`・`requirements` だけ | packet AC8・「新しい packet の書式」の後の段落 |
| Y6 | Opus F3 | P2 | 旧形式として受けるのは file 名の日付が 2026-10-28 以前の packet だけ。それより後の日付で `Phase` のある packet は拒む | packet「新しい packet の書式」の形式の見分け・AC8、D-145 Decision (4)、Matrix PH7 |
| Y7 | Opus F6 | P2 | R4 では `## Recovery` と `## Data Safety` も計画の節に入れ、H6 は R4 を拒む | packet「計画の節」・H6・安全の境界の表（独立 review）、D-145 Decision (7)、Matrix RC4・GA4 |
| Y8 | Opus F7 | P2 | R2 の packet なしの道は migration（`src-tauri/src/db/migration.rs`・`migration_tx.rs`・`schema_v*.rs`）と `package.json`・`package-lock.json`・`.npmrc` を含む差分を拒む。安全の境界の表の R2 の文を実態（機械が止める範囲と人の判断の範囲）に合わせ、差分の review の固定の問いに Risk の判定を足す | packet Goal・H8・安全の境界の表（Plan Commit の固定）、D-145 Decision (8)・Guarantee range、Matrix R2N4 |
| Y9 | Codex #5 | P2 | RC2 の見出しを packet の「計画の節」の定義（英語の見出し）にそろえた | Matrix RC2 |
| Y10 | Codex #6 | P2 | 旧形式の Owner Effort Budget を optional の列挙から外す（旧形式では今までどおり必須） | Matrix Compatibility Checks |
| Y11 | Codex #3・Opus F9 の lanes の部分 | P2 | `scripts/lanes.sh` とその test を Scope から消す（YAGNI）。再開は `gh pr list` と helper の status を明示の引数で引く。R2 の PR body の Risk の行を `Risk: R2（目視: 要）` / `Risk: R2（目視: 不要）` の固定の書式にし、template と H8 に書く | packet 予定の file・正本の文の最終形（AGENTS・PR template の行）・Ordinary Operation の再開の行・H8・H12・AC5・Contract Ledger、D-145 Decision (5)、Matrix の LN の行・Negative Paths・Compatibility・Mutation・Residual |
| Y12 | Opus F8 | P3 | 旧形式の packet の review では、review の template の Review Response の除外の文を残す | packet 正本の文の最終形（subagent-review-packet の行） |
| Y13 | Opus F9 | P3 | active の定義の言い回しを「起点の tree に無い dated packet」にそろえ、base が古い PR は base の版の helper の取り方に従うと注記した | packet Risk の節・「active な packet の定義」 |
| Y14 | Opus F10 | P3 | 所有の表の `docs/backlog.md` に AC4 が当てる 2 項目を足した | packet 他の lane との所有と順 |
| Y15 | Opus F11 | P3 | 新形式で Risk R2 の packet は作らず、あれば拒む | packet 形式の見分け、D-145 Decision (4)、Matrix PH7 |
| Y16 | Opus F12・Codex #7 | P3 | PH4 の判定の反転の検出の期待を PH4 と、足した PH8（承認済みかつ code の差分なら通る）に限った | Matrix PH8・Mutation-style Adequacy Questions |
| Y17 | Opus F13 | P3 | AR5 に #215 型の archive への移動が通る case を足した | packet「active な packet の定義」、Matrix AR5 |
| Y18 | Opus F14 | P3 | 本 PR の merge の後の同期は #217 の B になること、L3 待ちの lane（#209・#197・#210）はできれば本 PR より先に merge する順の方針を書いた | packet 他の lane との所有と順 |
| Y19 | Opus F15 | P3 | `pr-gate.py` の `mutate` の merge の後の案内 `docs closeout PR` を不要になる記述と Scope に足した | packet 予定の file・この変更で不要になる記述 |
| Y20 | Opus F16 | P3 | review の copy は `git archive` でなく worktree にすると、DEV_WORKFLOW の検証の節と active の定義に書いた | packet 正本の文の最終形・「active な packet の定義」 |

### Plan Review round 3（上限。起点 `d1c71155e1280901ee457977e5d8937dce7ad81a`、判定 reject）

Codex（P2 3）と fresh Opus 5.5（P2 2・P3 3）の findings を Coordinator が裁定し、下の表の全件を受理した（受理しなかったもの: なし）。上限の到達: Coordinator の disposition = 一括是正（`docs/DEV_WORKFLOW.md` Review Rules の round の天井）。次の round は開かない。是正の確かめ（この是正で plan-approved に進めるか）は owner の判断を待つ。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| Z1 | Opus F1 | P2 | 旧形式の見分けを file 名の日付の境から機械の判定に変えた: packet を最初に足した commit の親の tree に `scripts/plan-approve.sh` が無ければ旧形式。形式と `Phase` の行の有無が食い違えば拒む。古い日付を付けて新形式の規則を逃れる負例を足し、AC8 (b) の対象を最終 head の時点で open の PR 全部にした | packet「新しい packet の書式」の形式の見分け・AC8、D-145 Decision (4)、Matrix PH7 |
| Z2 | Opus F2 | P2 | active な packet の起点に「`WORKFLOW_BASE_SHA` が空か HEAD と同じなら `origin/main` との merge-base、それも HEAD なら ERROR」を足した（今の `check-workflow-git.sh` の落とし方を引き継ぐ） | packet「active な packet の定義」、Matrix PH9 |
| Z3 | Codex #1 | P2 | `plan-approve.sh` は書き込む前に対象の packet の index と作業 tree が HEAD と一致することを求め、不一致なら何も変えずに拒む | packet H1、Matrix PA4 |
| Z4 | Codex #2 | P2 | H6 は、比較元・比較先とも manual なしなら L3 の比較だけを対象外にする（manual の要否が変わったときは受けない） | packet H6、Matrix RC1 |
| Z5 | Codex #3 | P2 | 再開の手順の packet なしの分岐を、PR body の Risk の行から `--risk R0|R1|R2` と manual の要否をその PR の値で渡す形にした。R0/R1 の Risk の行の書式も同じ `Risk: R<n>（目視: 要|不要）` に決めた | packet Ordinary Operation の再開の行・PR template の行・H8 |
| Z6 | Opus F3 | P3 | H8 の path の拒否（migration・依存・`workflow`・`unknown`）を packet なしの道の全部（R0〜R2）に当てた | packet H8、D-145 Decision (8)、Matrix R2N4 |
| Z7 | Opus F4 | P3 | AGENTS の Session Start は 5 行（Ready・merge の行を残し、closeout だけ消す）と 1 通りに決めた | packet 正本の文の最終形（AGENTS の行） |
| Z8 | Opus F5 | P3 | helper の packet なしの道で、PR body の Risk の行の Risk・目視の要否と `--risk`・`--manual` が食い違えば拒む | packet H8、Matrix R2N5 |

### 是正の確かめ（closure、起点 `6c0ce73da8a818348554d03d25240c9a4c0bb3c9`）

round 3（上限）の一括是正の確かめ。fresh Opus 5.5 は reject（P2 1・P3 6）、Codex は approve（P3 1）。Coordinator が全件を受理し、下の処置にした（受理しなかったもの: なし）。N1 は round 3 の Z6（Opus r3 F3 の修正案）の行き過ぎを戻すもので、今の軽量化の方針（TD-475）に逆らって R0/R1 の workflow の docs を強めない。

| ID | 出所 | 重大度 | 処置 | 直した節 |
|---|---|---|---|---|
| W1 | Opus N1 | P2 | packet なしの道（R0〜R2）で機械が拒むのを migration・依存・`unknown=true`・workflow の実行 code（`workflow`かつ`rust`）に戻した。`workflow=true` の docs だけの R0/R1 は今のとおり受け、R2 では `workflow=true` を受けない（X9 の線）。R0/R1 で migration・依存・`unknown` を止めるのは「強める」と安全の境界の表と D-145 に書いた | packet H8・安全の境界の表（Plan Commit の固定）、D-145 Decision (8)・Guarantee range、Matrix R2N4 |
| W2 | Opus N2 | P3 | 形式の見分けの「最初に足した commit」を起点..HEAD の範囲に限った | packet 形式の見分け、Matrix PH7 |
| W3 | Opus N3 | P3 | rename は `--follow` で辿らず、旧形式の lane は packet を rename・切り直ししないと決め、DEV_WORKFLOW の Wave Operation に書くことにした | packet 形式の見分け、Matrix PH10 |
| W4 | Opus N4 | P3 | 最初の commit に親が無い（root・shallow の境）か shallow の repository なら ERROR | packet 形式の見分け、Matrix PH7 |
| W5 | Opus N5 | P3 | `origin/main` との merge-base も HEAD と同じなら active 0 件として通す（今の挙動）。ERROR は起点を解決できないときと merge-base が一意でないときだけ | packet active な packet の定義、Matrix PH9 |
| W6 | Opus N6 | P3 | AC1 に RC1b・R2N5、AC2 に PH9（と W3 の PH10）を足した | packet AC1・AC2 |
| W7 | Opus N7 | P3 | merge の後の新しい helper で裁かれる packet なしの PR は、PR body に `Risk: R1（目視: 不要）` の行を足す（commit は要らない）と書いた | packet 他の lane との所有と順（#215 の行） |
| W8 | Codex #1 | P3 | AC8 の「packet を本 branch に写す」検証をやめ、各 lane の worktree で全部の読み手を確かめる形に統一した | packet AC8・「新しい packet の書式」の後の段落 |

### Final Review broad（監査した版 `69738dbd40c1cbcfdfca6c5c96118c30245c66d4`。Codex 側 reject〈P1 1・P2 3・P3 1〉、Claude 側 Fable 5 approve〈P3 5〉）

Coordinator の裁定に従い、次を受理して直した（C = Codex 側の #、F = Fable 側の #）。Scope・AC は変えない。C1 は H2・H7 の「計画の固定」の実装の穴を塞ぐものとして扱う。

| finding | 裁定 | 反映先 |
|---|---|---|
| C1（P1）承認の後に計画の節を変えても止まらない。旧形式の `Phase: plan-gate` を承認済みと扱う | 受理 | `pr-gate.py` の `plan_texts`・`plan_drift`・`approval_state`、`requirements`（status・capture・record・ready・merge の共通の判定）、`parse_packet`（旧形式の承認の前の Phase は `pending` と扱う）、`check-workflow-git.sh`（同じ判定で ERROR と次の一手）。test は helper の `test_plan_change_after_approval_rejected`・`test_old_pre_gate_phase_is_not_approved` と checker の C1（先に red を確かめてから直した）。Matrix に PH11 を足した。DEV_WORKFLOW の計画の固定と merge-evidence の状態の節に 1 項 |
| C2（P2）`plan-approve.sh` が merge の一覧の取得の失敗を「merge なし」と扱う | 受理 | 終了値を確かめ、書く前に拒む。test は plan-approve の C2 |
| C3（P2）= F2（P3）`packet_form` が `ls-tree` の失敗を旧形式に落とす | 受理 | 成功した空の出力だけを「不在」、失敗は判定不能の ERROR。test は `test_unreadable_parent_tree_is_an_error` |
| C4（P2）GA の後、計画の節が同じでも機械の closure の拒否が fresh broad を案内する | 受理 | `sync_closure` が監査の closure を案内する。test は `test_rc_machine_closure_after_amendment_needs_audit`（節が変われば fresh broad のまま） |
| C5（P3）live な旧規則（R2+ の承認済み packet、Plans を現在地とする文） | 受理 | MANUAL §5.6・code_review・project-profile・DOC_STYLE_GUIDE・ARCHITECTURE・merge-evidence の R2+ の文 |
| F1（P3）新形式の Final Review Minimum の「上げるときだけ 1 行」に上げる先が無い | 受理（文だけ） | DEV_WORKFLOW の Workflow State と plan-packet の template の文。code は変えない |
| F3（P3）origin/main に落ちたときの ERROR が古い origin/main を名指さない | 受理 | `check-workflow-git.sh` と doc check の FL に「origin/main が古ければ git fetch origin」 |
| F4（P3）docs の closure の裁定の pointer が record に残らない | 受理（文だけ） | merge-evidence の docs の closure の段に、pointer の実体は PR の comment に置く文 |
| F5（P3）archive への移送で archive 側の中身を照合しない | 見送り（backlog） | `docs/backlog.md` の保留に 1 項目。archive は証拠でなく安全の境界は壊れない |

C1 の判定は、書き換えた既存の test 2 つにも当たった: workflow-git の D-102 D4 の case は承認済みの packet に `## Scope` を足していたので `## Notes` に移した。Workflow State が 2 つの packet は承認の判定で曖昧として止まるようになったので、その case は「止まる、2 つ目の Evidence Mode は読まない」に書き換えた。

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。
