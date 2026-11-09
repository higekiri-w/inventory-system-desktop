# Plan Packet: 並走の摩擦を削る（ハーネス、R3）

2026-09-28 起草。出典は wave 14（2026-09-28）で起きた並走の摩擦の観測（下の「現状の事実」）と、owner 決定 2026-09-28「規則とハーネスの改善を最優先にする（全体の開発の動きに関係するから）」、同日の起票承認（並走の摩擦だけを先に 1 本の lane で起こす。PR4・PR5 の残りはその後）。owner の考え方（要旨）: 規則は失敗を測る方法としてはよいが、それで首が回らなくなるのは避けたい。Coordinator の言い換え: 規則が止めるときは、起きている失敗を 1 文で言えること。言えないのに止める規則は緩めてよい。安全の境界（古い証拠の拒否、独立 review、owner の L3、承認）は残す。

本 lane は wave に属さない単独の lane（ハーネス刷新 PR4・PR5 の前に置く）。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R3
- Plan Commit: 13403d0f523168ec08f2fde92cbc1469246f8a19
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（fresh context、Coordinator が指定する worktree で作業）
- Plan Reviewer: fresh Opus 5.5 subagent（fork でない）+ Codex（GPT-6 Astra、effort high。`.local/codex-orders/MODEL-SELECTION.md` の「merge gate・helper・classifier・hook の合否を変える変更」の行）。互いに独立で Writer と別 context
- Final Reviewer: Fable 5.1（Claude 側、R3）+ Codex（GPT-6 Astra、effort high、同じ行）。互いに独立で Writer・Plan Reviewer と別 context（Double Audit）。後の reviewer に先の結果を見せない
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/harness-parallel-friction

`Branch` は helper・checker が評価しない追加行（D7 の dogfood。template への追加は S9）。

manual なし: 製品の runtime・画面・配布物を変えず、Windows native L3 の対象画面が無い。helper・checker の振舞いは合成 fixture の自動 test で確かめる（L3 Eligibility の (1) に当たらない）。r4 なし: data・DB・破壊的 git 操作を含まず、変更は revert で戻せる。Final Review Minimum 2: `scripts/pr-gate.py`・`scripts/check-workflow-git.sh`・`scripts/doc-consistency-check.sh`・`scripts/tests/**` は `scripts/ci/classify-changes.sh:55` の実行制御（full）、`docs/DEV_WORKFLOW.md` ほかは `:59` の policy docs で、どちらも workflow=true になり helper は Minimum 2 を要求する（`scripts/pr-gate.py:250-252`）。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（2026-09-28、起草役 = Opus 5.5 subagent、本 commit、plan-first）: Risk R3（下記 Risk）。spec-check で、現行の正本（`docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」「Helperの境界」、`docs/DEV_WORKFLOW.md` の Workflow State の packet 選択規則・Wave Operation・Post-Merge Closeout）が「先行 lane の closeout を後続 lane の同期より先に済ませる」「Plans.md の lane ごとの link から packet を選ぶ」を定めており、既存の設計のままでは足りないと確かめた（skip は使わない）。design の出力は本 packet の Spec Contract（D1〜D9）と G2 の比較に置き、正本（merge-evidence・DEV_WORKFLOW・decision-log D-097）への昇格は実装の S7・S8・S12 で行う（workflow の正本は本 lane が書き換える文書そのもの。ハーネス刷新 PR3 と同じ形）。G2 は推奨案（B）で設計を閉じ、owner の示した案（A）と違うため Plan Gate で owner に諮る（owner が A か C を選べば design へ戻る）。Plan Review へ。
- owner の判断の反映（2026-09-29、Coordinator、Phase は plan-gate のまま）: owner が G2 に B（closeout を後回しにして wave ごとに R0 の PR 1 本にまとめる）を選び、PK5 を PR の差分が触る packet に絞る S3 を本 lane に入れることを承認した（この change での介入 2 回目、2 つを同じ 1 回で得た）。起票時に owner へ示した A は採らない。設計は本 packet の推奨のまま閉じ、Plan Review round 1 へ出す。あわせて origin/main（`df0488eb`、小口のまとめ・色と強調の closeout まで）を取り込んだ。
- 訂正（2026-09-29、裁定 r1 の F4）: 上の行の「介入 2 回目、2 つを同じ 1 回で得た」は、decision point 単位の計上（`docs/DEV_WORKFLOW.md:267`）では G2 = B と S3 の承認の 2 回（介入 2 回目・3 回目）に当たる。Owner Effort Budget の表を消費 3 に直した。
- Plan Review round 1 の裁定 r1 の反映（2026-09-29、起草役 = Opus 5.5 subagent、Phase は plan-gate のまま）: fresh Opus 5.5・Codex（GPT-6 Astra）とも reject（P1 0）で、Coordinator が全件を accept した（F1〜F14 と Codex の注記）。S1・S3・S4・S8・S12、AC、Test Plan、G2 の費用、Contract Probe 5、Owner Effort Budget を直した。Plan Review round 2 へ。
- Plan Review round 2 の裁定 r2 の反映（2026-09-29、起草役 = Opus 5.5 subagent、Phase は plan-gate のまま）: fresh Opus 5.5 は approve（P3 7）、Codex（GPT-6 Astra）は reject（P1 1 / P2 1 / P3 1）で、Coordinator が全件を accept した（G1〜G10）。closeout の判定に同じ PR の中の同名 archive への移送の対応を必須にし（G1、test と MU11 を追加）、Ordinary Operation の Draft の次条件・hosted の起点の前提、S7・S8・S9・S10、AC6・AC7・AC8・AC11、relay の消費、G2 の費用、Matrix の走査を直した。Plan Review round 3（上限）へ。
- Plan Review round 3（上限）の結果と裁定 r3 の一括是正（2026-09-29、起草役 = Opus 5.5 subagent、Phase は plan-gate のまま）: fresh Opus 5.5 は approve（P3 6）、Codex（GPT-6 Astra）は reject（P1 0 / P2 1 / P3 1）で、Coordinator が全件を accept した（H1〜H8）。Ordinary Operation の base 同期の後の record の順序を merge-evidence の manual の再利用の順序に合わせ（H1）、S2 に複数 page の fixture と test・MU12 を足し（H2）、守る境界を「差分が `--packet` 以外の active packet を触る PR の拒否」に絞って `--packet` に他の lane の packet を名指しする経路を Residual Test Gaps に置き（H3）、S8 に `docs/DEV_WORKFLOW.md:420` を足し（H4）、closeout の test を 2 packet にし（H5）、S5 で `strip_inline_code_spans` を消し（H6）、引用を直し（H7）、S3 の fail-closed の MU13 を足した（H8）。round 4 は回さず、Coordinator の現物の確認と owner の plan-approved へ。
- plan-gate → plan-approved（2026-09-29、Coordinator、本 commit）: 上限の Plan Review round 3（対象 `1a2c5fe7`）は Claude 側 = approve（P3 6）、Codex 側 = reject（P1 0 / P2 1 / P3 1）。round 天井に従い追加の round は回さず、裁定 r3 の一括是正（`13403d0f`）で全件を反映した。Coordinator が一括是正を現物で確かめた（MU1〜MU13 と追加の test 4 本が packet と Matrix にそろう、H4 の文言、`strip_inline_code_spans` の削除の項目、`doc-consistency-check.sh --target plan` と `check-workflow-git.sh` が exit 0）。owner 承認（2026-09-29「承認するよ」、この change での介入 4 回目、介入の上限 5 → 6 を同じ 1 回で承認）のもと plan-approved。Plan Commit = `13403d0f`。
- plan-approved → implementing（2026-09-29、Coordinator、state-only）: Writer（Opus 5.5 subagent）へ実装を発注する。Writer の開始 HEAD は本 commit。
- implementing → archive（2026-09-29、Coordinator の closeout〈Opus 5.5 subagent〉、本 commit）: PR #123 を squash merge（`294ba366`、2026-09-29、helper 経由）。本 lane は「本 lane 自身の merge」のとおり旧い helper（merge 前の origin/main の `scripts/pr-gate.py`）で record・Ready・merge した。packet と Matrix を archive へ移送し、Implementation Results と Review Response を記録。S11 で closeout まで残した本 lane の行を `docs/Plans.md` の `## 次の行動` から消し、Fable broad の P3-3 に沿って backlog の Wave Registry の link の項目を閉じ、closure 2 の P3・起票時に見つけた checker の脆さ・manual の再利用の観測を backlog に置いた。本 closeout は D-097 の新しい規則と helper のもとで出す最初の closeout。

## Owner Effort Budget

- 介入回数上限: 6（owner 承認 2026-09-29、この change での介入 4 回目。plan-approved の承認が見込みに入っていなかったため、その承認と同じ 1 回で 5 から改めた）。その前は 5（既定 3 から。理由: G2 の推奨が owner の示した案と違い、Plan Gate で owner の判断点が増える）。closeout 時点の実績: 6 / 6（起票承認、G2 = B、S3 の承認、plan-approved と上限 6、Ready、merge）
- 実働時間上限: 15分（文書・script・test の変更で manual は無い。owner の作業は判断点の回答と Ready・merge の指示に限られる見込み）
- relay 往復上限: 5（Plan Review の Codex が最大 3 round、Final Review の Codex broad 1、base 同期の後の Codex closure 1）。closeout 時点の実績: 4 / 5（Plan Review の Codex 3、Final Review broad の Codex 1）。closure は Claude 側で 2 本行い、Codex closure の予備は使わなかった
- Plan Review round 天井: 3（既定 3）

| 種別 | 上限 | 消費（2026-09-29 時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 4: 起票承認 2026-09-28（並走の摩擦だけを先に 1 本の lane で起こす）、G2 = B 2026-09-29、S3 の承認 2026-09-29（G2 と S3 は別の decision point、`docs/DEV_WORKFLOW.md:267`）、plan-approved と上限 6 の承認 2026-09-29 | 2: Ready 1、merge 1 | 0 | 6 = 4 + 2 + 0 |
| relay | 5 | 3: Plan Review round 1・round 2・round 3 の Codex | 2: Final Review の Codex broad 1、base 同期の後の Codex closure 1 | 0 | 5 = 3 + 2 + 0 |

介入の予備は 0 になった。round 天井の disposition などで追加の owner 判断が要れば、その判断と介入の上限の改定を同じ 1 回で owner に求める。

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。Ready の依頼は「この change での介入 5 回目 / 予算 6 回」、merge の指示の依頼は「6 回目 / 予算 6 回」と書く。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3「merge gate changes」に当たる。本 lane は merge の可否を決める 3 つの判定を変える: helper の packet の判定（`scripts/pr-gate.py:223-252`）、PK4 の Plans.md の登録の検査（`scripts/doc-consistency-check.sh:1364-1391`）、PK5 の Plan Commit の祖先の検査の対象（`scripts/check-workflow-git.sh` の `main()`）。どれも「変更前に red だった入力が変更後に green になる」向きの変更を含む（他の lane の merge 済み packet が残る head、Plans.md に lane の link が無い head）。kickoff の問い「この変更でどれかの required gate の green / red が変わるか」の答えは「変わる」で、R3・Minimum 2。R4 には当たらない（data・破壊的操作なし）。

Review-only skipped because: R3 の review-only sub-agent は、Final Review の Double Audit の broad（Fable 5.1 と Codex の 2 本、どちらも Contract Audit を正本から行う）が兼ねる（`docs/DEV_WORKFLOW.md` Review Rules・Draft PR Checkpoint）。

## 現状の事実（wave 14、2026-09-28）

Coordinator の観測（1〜6）と、起票時の実測（7）。

1. main に merge 済みの lane の packet が `docs/plans/` に残ると、main を取り込んだ別の lane の PR で helper が `packet absent or multiple active packets`（`scripts/pr-gate.py:231`）を返し、record も Ready もできなかった。PR #118 が #114 の packet を持ち込んだ。そのため lane ごとに「merge → closeout の PR（Ready・CI・merge）→ 他の lane が追いつく」が直列になり、1 lane につき PR を 2 本ずつ通した（#114→#119、#117→#120、#118→closeout）。
2. R0 / R1 の PR（closeout を含む）も、head の `docs/plans/` に packet が 1 つでもあると `R2+ active packet requires --packet`（`:244`）で止まる。
3. helper は `docs/Plans.md` の `## 次の行動` に packet への link を求める（`:232-234`、`packet not registered in Plans`）。PK4（`scripts/doc-consistency-check.sh:1364-1391`）も同じ link を求める。各 lane がこの節に 1 行ずつ書き足すため、main の取込みのたびに Plans.md がほぼ必ず衝突した（#118・#116・#117 で計 4 回）。衝突を解いた版では manual を再利用できない（merge-evidence の規則）ため、#118 は L3 をやり直した。
4. strict（最新の main との整合）は MG-D1 のまま。1 本の merge ごとに他の PR が追いつく必要は残る。
5. ほかの衝突源（decision-log の末尾への追記、`90-traceability.md` の再生成）は本 lane の対象外（Non-scope に判断を書いた）。
6. 事故: `docs/plans/` が空になる merge で、git の directory rename の推測が別の lane の packet を `docs/archive/plans/` へ動かしかけた。`git -c merge.directoryRenames=false merge` で避けた（Contract Probe 4 で再現）。
7. 起票時の実測: merge 済みの packet が main に残ると、PK5（`scripts/check-workflow-git.sh`）も red になる。squash merge の後は packet の Plan Commit が main の祖先でないため。base `b0f3b68b` で exit 1（Contract Probe 3）。PK5 は pre-push（`scripts/pre-push.sh:188`）・local-ci（`scripts/local-ci.sh:197`）・hosted の docs job（`.github/workflows/ci.yml:312`）で走るので、helper を直しても、後続 lane は先行 lane の closeout を待たないと Merge gate が red になる。

## Goal

Goal Invariant:

### 最小完了条件

- 並走する 2 lane の片方が merge された後、もう片方は、先の lane の closeout の PR（Ready・CI・merge）を待たずに main を取り込み、helper の `status` / `capture` / `record` / `ready` / `merge` と hosted の PK4・PK5 を通して merge できる。
- merge 済みで closeout 前の packet が main に残っていても、他の lane の R2+ の PR も、無関係な R0 / R1 の PR（closeout を含む）も、その packet を理由に止まらない。
- lane の起票と closeout が `docs/Plans.md` の `## 次の行動` へ lane ごとの行を足さない。そのため main の取込みで Plans.md が衝突しない。dashboard は `docs/plans/` を指す 1 行で active な作業を示し、lane の branch・Phase・PR は packet と helper status から追える。
- 守る境界として次の入力を拒否する（現行で拒否していたものは引き続き、差分に基づくものは新たに）: 差分が `--packet` 以外の active packet を触る R2+ の PR（`--packet` と並んで他の lane の packet を編集・移送・削除する PR、差分が 2 個以上の packet を触る PR、`--packet` と差分の packet の不一致を含む）、差分が active packet を 1 つも触らない R2+ の PR（`--packet` を付けた PR。packet を付けずに R0 と申告する PR の Risk の正しさは機械では決めない、Residual Test Gaps）、active packet を編集・削除する R0 / R1 の PR（closeout の移送を除く）、自分の packet の Plan Commit の祖先・不変性の破れ、古い head/base の record。helper は `--packet` がどの lane の packet かを知らないので、自分の packet を持たない stacked の branch が merge 済みの他の lane の packet を `--packet` に名指しして書き換える PR は、この境界の外に残る（現行の helper でも通り、退行ではない。Residual Test Gaps と Impact Review の「Plan Commit の固定」）。

### 失敗定義

- 上の待ちのどれかが残る（先行 lane の closeout を待たないと、後続 lane の helper・PK4・PK5 のどれかが red になる）。
- 判定を緩めた結果、差分が `--packet` 以外の active packet を触る PR（他の lane の packet を並んで書き換える PR）、R2+ の作業を R0 と称して active packet を編集する PR、git の directory rename の推測で他の lane の packet を archive へ動かした PR が通る。
- 自分の packet の Plan Commit の祖先・不変性（PK5）、承認済み packet の Risk・Final Review Minimum・Human Gate の照合、record の head/base の照合、Double Audit の下限のどれかが弱まる。
- lane の branch・Phase・PR を tracked の証跡から辿れなくなる。

### 非目的

- manual の再利用の条件を緩めること（衝突を解いて追いついた版の manual の再利用は、本 lane を入れた後の実際の衝突を見て owner が改めて決める）。
- strict（MG-D1）の撤去。1 本の merge ごとに他の PR が追いつく必要は残る。本 lane は追いつきのたびに起きる衝突と待ちを減らす。
- `docs/decision-log.md` の末尾への追記と `docs/function-design/90-traceability.md` の再生成による衝突。
- PR4 の残り（Findings Freeze の撤去・Owner Effort Budget・Wave Operation の他の改訂・PR2 / PR3 の follow-up）と PR5 の残り（classifier の穴・`.claude/agents`・local-ci の symlink 検出・pre-push の merge 済み PR の検出）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、本 lane の merge 後に並走する 2 lane（X・Y）を起票から closeout まで通す操作列を書く。各行の「旧:」が本 lane の前に起きていた待ちで、本 lane が消す所を示す。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| main に本 lane が入っている。X・Y の起票の承認がある | X・Y の起草役がそれぞれ plan-first commit に packet と Matrix だけを置く（packet の前文に wave と lane、Workflow State に `Branch` 行） | Plans.md に lane の行を足さないので、起票が Plans.md の衝突源にならない（旧: 各 lane が `## 次の行動` に 1 行足し、main の取込みのたびに衝突。wave 14 で計 4 回） | `bash scripts/doc-consistency-check.sh` の PK4 が、`## 次の行動` の `docs/plans/` を指す行で green（D6） | なし |
| X・Y（manual・R4 の要る lane を含む）が実装・Draft PR・Final Review を進める | 通常どおり（manual の要る lane は L3 の結果、R4 の要る lane は R4 の承認も現在の head で record する） | 変化なし | helper status で CI 以外の要件（review・manual・r4）が揃ったことを確かめる。CI の未実行・未完了（`CI run absent; …`・`latest CI is pending/failed`、`scripts/pr-gate.py:367`・`:371`。status は CI の blocker も返す、`:384-389`）は Draft の前進条件にしない。helper `ready` は CI を見ない（`:465-468`）。CI の成功は Ready の後に確かめる（merge-evidence `:122`・`:123`） | なし |
| X が Ready・CI 成功・owner の merge 指示 | helper `merge` で X を merge | main に X の packet が Phase implementing のまま残る | merge 成功 | なし |
| Y は Draft、X の closeout はまだ | Y を Draft のまま（Ready なら Draft に戻して）`git -c merge.directoryRenames=false merge origin/main` を 1 回 | X の closeout を待たずに取り込める（旧: X の closeout の PR の Ready・CI・merge を待つ = 待ち 1）。Plans.md は X も Y も触っていないので衝突しない。directory rename の推測で Y の packet が archive へ動かない | 単段 merge。`bash scripts/check-workflow-git.sh` が Y の packet だけを PK5 の対象にして green（X の packet は Y の差分が触らない、D4） | 外部前提が 2 つ（どちらも外れたときは全 packet か X の packet も検査して red の側に倒れる。安全は弱まらない）。起点の経路: pre-push・local-ci は `WORKFLOW_BASE_SHA` 未設定で `origin/main`（S4 (a2)）、hosted の PR の docs job は `WORKFLOW_BASE_SHA` = `github.event.pull_request.base.sha`（`.github/workflows/ci.yml:311`、S4 (a1)）。(1) hosted の PR run の `pull_request.base.sha` が、Y が取り込んだ main 以上であること（外部前提。古い main だと X の packet も差分に出て PK5 が red。確かめ方: 本 lane の merge 後に main を単段 merge で取り込んだ最初の PR の docs job の step log に出る `WORKFLOW_BASE_SHA` と、取り込んだ main の SHA を read-only で突き合わせる）。(2) hosted で `origin/main` を使うのは workflow_dispatch の run だけ（Contract Probe 5、外部前提。解決できなければ全 packet を検査する） |
| Y は main を取り込んだ新しい head | `python3 scripts/pr-gate.py status / capture --pr Y --packet Y の packet` の後、merge-evidence の manual の再利用の順序（`docs/agent-guidance/merge-evidence.md:134`・`:136`）で record を取り直す。manual の要る lane は、旧 head の manual pass を含む fresh capture から、manual の再利用（単段 merge の機械条件〈`:132`〉と owner の `reuse_approval` を満たすとき）か再実施の結果を先に record し、fresh capture をしてから closure を record する。R4 の要る lane は新しい head で R4 の承認を取り直す（manual の再利用では代用しない、`:134`）。manual も R4 も要らない lane は capture の後に closure を record する | helper が Y の差分が触る active packet 1 つで判定し、record できる（旧: head の `docs/plans/` に X と Y の 2 つがあり `packet absent or multiple active packets` で record も Ready もできない = 待ち 2） | 現在版の record で closure pass、manual の要る lane は manual pass、R4 の要る lane は R4 pass | なし。順序の根拠: 逆にして closure を先に record すると旧 manual が pending に戻り、Ready の判定は `manual incomplete`（`scripts/pr-gate.py:322`）、その後の再利用も `server manual pass required`（`:446`）で拒まれる（Plan Review round 3 の Codex が既存の `RecordLifecycle` fixture で再現、merge-evidence `:136`）。本 lane はこの順序を変えない |
| Y の record が揃う | owner の Ready 判断 → helper `ready` → hosted CI → owner の merge 指示 → helper `merge` | Y が merge される。hosted の PK4・PK5 も X の packet で red にならない | Merge gate 成功 | なし |
| X・Y とも merge 済み、どちらも closeout 前 | 1 本の R0 の closeout の PR で X と Y の packet と Matrix を `docs/archive/plans/` へ移し（Phase archive）、`docs/Plans.md` の `## 直近の完了` を更新する | lane ごとの closeout の PR 2 本が 1 本になる（旧: #114→#119、#117→#120、#118→closeout の 1 lane 1 本）。その間も他の lane は止まらない | helper `--risk R0` が、`docs/plans/` を離れる各 packet について、同じ PR の差分の中で同名の `docs/archive/plans/<名前>.md` への移送が起きていて、head の archive の Phase が archive であることを確かめて通す（D2） | GitHub が移送を `renamed` でなく削除と追加で返す場合も、`removed`（active）と `added` / `modified`（archive）の組を移送と認める（Contract Probe 1・2） |
| X の merge 後・closeout 前に、無関係な R0 の docs 修正 PR が出る | helper `--risk R0 --manual not-required` | 通る（旧: head の `docs/plans/` に X の packet があり `R2+ active packet requires --packet` で止まる = 待ち 3） | 差分が active packet を触らない | なし |
| 誤り: Y の PR が X の packet を書き換える、R0 の PR が active packet を編集する、R0 の PR が active packet を削除するだけ（同名の古い archive が無変更で残る）、同期の衝突の解き方で Y の packet が archive へ動く | helper `status` | 拒否（R2+ は差分の packet が 2 つ、R0 は active packet の編集、削除だけの R0 は同じ PR の中の archive への移送が無い、archive へ動いた Y の packet は `--packet` と差分が一致しない） | 是正して取り直す | なし |

この列で、wave 14 の 3 つの待ち（closeout の PR の直列・record 不可・R0 の停止）と Plans.md の衝突が消える。strict による追いつき（merge のたびに他の Ready の PR が同期・CI をやり直す）は残る（非目的）。closeout の PR の merge も strict の追いつきを 1 回起こすが、wave ごとに 1 本にまとめる（D5）。

## Scope

行番号は origin/main `df0488eb` を取り込んだ本 branch のもの（script と workflow の文書は起点 `b0f3b68b` から変わっていない。`docs/Plans.md` は変わった）。各 file の変更は該当行・節に限る。

- S1 `scripts/pr-gate.py` の `Gate.requirements()`（D1〜D3）
  - `:223-228` の head の `docs` / `docs/plans` の一覧の取得をやめ、`:214-215` で取得済みの PR の差分（`/pulls/N/files` の各 entry の `status`・`filename`・`previous_filename`）から、差分が触る active packet を決める。
  - 差分の切り詰めの検出（F2）
    - 全 page の entry の数が 0 か 3000 以上なら exit 非 0 で止める（現行 `:216` と同じ message `PR diff unavailable or truncated`・exit 1）。GitHub の files API は 1 PR あたり最大 3000 files を返す（公式: https://docs.github.com/en/rest/pulls/pulls#list-pull-requests-files ）。
    - 判定は entry の数で行い、path の数に依らない。現行の `len(paths) < 6000`（`:216`）は各 entry を 2 path に展開する `:215` に依るため、`previous_filename` の扱いを書き直すと意味が変わる。classifier へ渡す path（`:215`・`:219`）は現行のまま。
  - active packet の path（F12 (a)）: full path が `docs/plans/\d{4}-\d\d-\d\d-[^/]*\.md` に fullmatch するもの（`docs/plans/` 直下だけ。下の階層は拾わない）。現行 `:228` は basename に `\d{4}-\d\d-\d\d-.*\.md` を当てていたので、full path に移しても同じ集合になる。
  - 触る packet の分類（D1、F12 (b)）
    - head にある packet: `status` が `removed` 以外の entry の `filename` が active packet の path。
    - 差分で `docs/plans/` を離れる packet: `status` が `renamed` の entry の `previous_filename`、または `status` が `removed` の entry の `filename` が active packet の path。これ以外の `previous_filename`（`copied` の元の file は残り、変わらない）は触る packet に数えない。
    - `status` が `added` / `modified` / `renamed` / `removed` / `changed` / `copied` / `unchanged` 以外なら入力エラー（exit 2）。
  - R2+（`--packet` あり）: 触る packet（head にある packet と離れる packet の和）がちょうど `{--packet}` で、かつ `--packet` が head にある packet でなければ拒否する（exit 1。message は `PR diff must touch exactly the --packet active packet`）。`--packet` と並んで他の lane の packet を編集・移送・削除する差分は「2 つ以上」として拒否される。helper は `--packet` がどの lane の packet かを知らないので、`--packet` 自体に他の lane の packet を名指しし、その packet だけを書き換える差分は通る（現行の helper でも通る。Residual Test Gaps）。離れる packet は `--packet`（head にある）と一致しえないので、R2+ の経路は下の移送の検査と archive の取得を行わず、この集合の比較だけで決める。
  - R0 / R1（`--packet` なし）: head にある packet を 1 つでも触れば `R2+ active packet requires --packet` で拒否する。そのうえで、離れる packet がすべて下の closeout の移送なら通す。
  - 離れる packet の扱い（R0 / R1 の経路、D2、F6、裁定 r2 の G1）: 離れる packet `docs/plans/<名前>.md` ごとに、次の順で確かめる。
    - (1) 移送の対応: **同じ PR の差分の中で**、同名の `docs/archive/plans/<名前>.md` への移送が起きていること。認める形は 2 つ: `status` が `renamed` で `previous_filename` = `docs/plans/<名前>.md`・`filename` = `docs/archive/plans/<名前>.md` の entry、または `removed` の `docs/plans/<名前>.md` と、`added` か `modified` の `docs/archive/plans/<名前>.md` の entry の組。どちらも無ければ exit 1（message は `packet leaving docs/plans must move to docs/archive/plans in this PR`）。既存の同名の archive が無変更のまま active packet が消える差分（base に active A・B と古い archive A があり、PR は active A の `removed` だけ）は、archive 側の entry が差分に無いのでここで拒否される。head の archive の存在や Phase は、移送の証明に使わない。
    - (2) head の `docs/archive/plans/<名前>.md` を `self.contents(path, head)` で取得する。取得できない（404 を含む。現行の transport では `command failed`）なら exit 2（現行の `contents()` の扱い、`:207-210`・`:41`）。(1) を満たした後なので、ここに来るのは差分と head の内容が食い違う異常だけ。
    - (3) 取得できた内容の Workflow State の Phase が `archive` なら closeout の移送として通す。`archive` 以外なら「進行中の packet の移送」として拒否する（exit 1。directory rename の推測で動いた packet の形）。
  - `:232-234` の `docs/Plans.md` の `## 次の行動` の link の照合を削除する（D3。dashboard の検査は PK4 の 1 か所に置く）。
  - 変えないもの: `:229-230` の `--packet` の path の形式、`:235-242` の approved snapshot との Risk・Final Review Minimum・Human Gate の照合、`:245-247` の R0 / R1 の `--risk` / `--manual` の要求と `CI execution change requires R3 packet`、`:250-252` の Double Audit の下限、`:217-222` の main 側の classifier、record・status・capture・ready・merge の他の全処理。
- S2 `scripts/tests/pr-gate.test.py`
  - `FAKE_GH` の `/pulls/7/files` の entry に `status` と `previous_filename` を持たせる（既定の entry は `status: modified`）。
  - 複数 page の fixture（裁定 r3 の H2）: `FAKE_GH` の `/pulls/7/files` は現行で常に 1 page（`[s['files']]`、`scripts/tests/pr-gate.test.py:212`）を返し、helper は `--paginate --slurp`（`scripts/pr-gate.py:63`）の全 page を受ける（`:214`）。`state['file_pages']`（page の list）があればそれをそのまま返し、無ければ現行どおり `[s['files']]` を返すようにする。
  - 新しい class `PacketScope`（CLI 経由、fake gh の process を通す）に Test Design Matrix の G1 の行の test を置く: `test_other_lane_packet_in_head_r2_passes`、`test_two_active_packets_in_diff_rejected`、`test_packet_argument_mismatch_rejected`、`test_diff_across_pages_rejected`（1 ページ目に自分の packet、2 ページ目に他の lane の packet → exit 1。裁定 r3 の H2）、`test_r0_closeout_archive_move_passes`（1 本の R0 PR で 2 packet を移す。1 件は `renamed`、もう 1 件は `removed` と `added`。裁定 r3 の H5）、`test_r0_closeout_reported_as_remove_and_add_passes`、`test_r0_edit_of_active_packet_rejected`、`test_r2_archiving_other_lane_packet_rejected`、`test_r0_packet_deletion_without_archive_rejected`（archive 側の entry が差分に無く、archive も head に無い → S1 (1) で exit 1）、`test_r0_removal_with_preexisting_archive_rejected`（裁定 r2 の G1 の反例。Matrix の同名の行の fixture）、`test_r0_archive_move_without_phase_archive_rejected`、`test_rename_inside_plans_counts_both_paths`、`test_packet_scope_does_not_list_docs_or_read_plans`、`test_unknown_file_status_is_input_error`、`test_truncated_diff_rejected`（3000 entry で exit 1 と `PR diff unavailable or truncated`、2999 entry ではこの理由で止まらない）。
  - 既存の test の fixture を新しい契約に合わせる（期待する合否は変えない）
    - `configure_packet` と `test_registered_amendment_order_cannot_roll_back_manual`: packet を `state['files']` に `added` で載せ、`state['packets']` と `docs/Plans.md` の fixture を使わない。
    - `state['files']` を上書きする 3 本（`test_packet_double_audit_cli`、`test_workflow_minimum_one_rejected`、`test_codex_only_r3_ui_minimum_one_accepted`〈`scripts/tests/pr-gate.test.py:423-428`〉）: 上書きする file に加えて packet を差分に載せる。
  - exit 1 を期待する test は拒否の理由（stderr の message）も assert する（F7。fixture の載せ忘れが別の理由の exit 1 で green のまま残らないように）
    - S2 が fixture を直す test の exit 1 の呼出し: `test_workflow_minimum_one_rejected`（`required Double Audit minimum is 2`、`scripts/pr-gate.py:252`）、`test_r4_minimum_one_rejected` と `test_r4_approval_gate_cannot_be_omitted`（`R4 gates missing`、`:184`）、`test_latest_amendment_snapshot_owns_gate_conditions` の `:376`（`approved packet condition changed`、`:242`）、`test_packet_double_audit_cli` の exit 1 の各呼出し（理由の文は実装時に現行の出力から取る）。`test_unamended_gate_condition_changes_are_rejected` は既に理由を assert している（`:310`）。
    - `PacketScope` の exit 1 の test も理由を assert する（Matrix の各行の message）。
  - 退役する契約の test を置き換える: `test_absent_plans_directory_is_normal_r0`（head の一覧を読まない R0 の経路）は `test_packet_scope_does_not_list_docs_or_read_plans` へ、`test_parent_or_present_directory_http_failure_is_error`（一覧の HTTP 失敗）は「closeout の移送で archive の内容の取得が HTTP 失敗なら exit 2」へ、`test_multiple_packet_and_missing_manual`（head の 2 packet）は `test_two_active_packets_in_diff_rejected` へ置き換える。置き換えは、退役する「head の一覧」という入力の代わりに同じ失敗（曖昧な結び付け・HTTP 失敗の握り潰し）を新しい入力で拒否することを確かめるもので、弱めない。
- S3 `scripts/check-workflow-git.sh`（D4）
  - PK5 の `check_plan_commit_ancestry`（`main()` の loop、`:150`）を、branch の差分が触る packet だけに適用する。
  - 起点: `WORKFLOW_BASE_SHA` が設定されていて HEAD と違えばそれ（hosted の PR）。そうでなければ `origin/main`（env 未設定の pre-push・local-ci と、env が HEAD と同じ hosted の dispatch）。
  - 差分（F12 (c)）: `git diff --no-renames --name-only "<起点>"...HEAD -- docs/plans/` に出る path のうち、HEAD にある `docs/plans/` 直下の `*.md`（現行の loop と同じ `find -maxdepth 1`、`:151`）を対象にする。`--no-renames` で rename を削除と追加に分け、rename 検出の設定に依らない。
  - fail-closed: 起点が解決できない、または `git merge-base --all "<起点>" HEAD` が 1 つでない（0 か 2 以上）なら、現行どおり全 packet を対象にする。
  - Evidence Mode の値と Phase の enum の検査（`main()` の loop 内）と shallow の検査は、現行どおり全 packet・全履歴に掛ける。
  - 冒頭の comment の「検査内容」に対象の範囲を 1 行足す。
- S4 `scripts/tests/workflow-git-checks.test.sh`
  - 合成 repo に、squash 相当で Plan Commit が祖先でない merge 済み packet（main 側）と、自分の packet（branch 側）を置く。
  - (a1) `WORKFLOW_BASE_SHA` = main（hosted の PR の経路）で、merge 済み packet は対象外になり exit 0。
  - (a2) `WORKFLOW_BASE_SHA` を未設定にし、`git update-ref refs/remotes/origin/main <main>` を置く（pre-push・local-ci の経路）。merge 済み packet は対象外になり exit 0。
  - (b) branch が merge 済み packet を書き換えると対象になり exit 1。
  - (c) 自分の packet の Plan Commit が祖先でないと exit 1。
  - (d) 起点が解決できない（`origin/main` も `WORKFLOW_BASE_SHA` も無い）と全 packet を検査して exit 1。
  - (e) `WORKFLOW_BASE_SHA` が HEAD と同じ（dispatch）なら `origin/main` を起点にし、自分の packet を検査する。
  - (f) 起点と HEAD の merge-base が 2 つある（criss-cross merge）と全 packet を検査して exit 1。
  - 既存の「squash 相当の負例」（`scripts/tests/workflow-git-checks.test.sh:128-152`）は起点なしのまま残す。
- S5 `scripts/doc-consistency-check.sh` の PK4 の Plans.md の検査（`:1364-1391`、D6）: active packet が 1 つ以上あるとき、`## 次の行動` の本文（`strip_fenced_code_and_html_comments` の後）に文字列 `docs/plans/` が 1 回以上あることを求める。packet ごとの link の要求はやめる（link があっても拒まない）。error の文は `PK4: docs/Plans.md の '## 次の行動' に active packet の一覧（docs/plans/）を指す行がありません`。この検査は inline code を除かないので、link の抽出の前処理だった `strip_inline_code_spans`（`scripts/doc-consistency-check.sh:939`、呼出し元は `:1378` だけ）を関数ごと削除する（裁定 r3 の H6。死んだ code を残さない）。
- S6 `scripts/tests/doc-consistency-plan-packet.test.sh`: `write_plans_md_linking` を、`docs/plans/` を指す行を書く helper に変える（正例の fixture の意味を保つ）。section 11・11c・12・28 を新しい契約へ置き換える: packet ごとの link が無くても pointer があれば PK4 OK、pointer が無ければ ERROR、pointer が code fence か HTML comment の中だけなら ERROR、`### Wave Registry` の小見出しの下の pointer も検出する。inline code の中の `docs/plans/` は pointer として数える（文字列の要求で、link の要求ではないため）。
- S7 `docs/agent-guidance/merge-evidence.md`
  - `## base同期だけでheadが変わる場合` の冒頭の文（`:130`）: 「まず先行PRのcloseoutを完了し、対象をDraftへ戻してからorigin/mainを取り込む」を、「先行 PR の closeout を待たない。merge 済みで closeout 前の packet は、後続 PR の差分が触らない限り helper と PK5 の判定に入らない。対象を Draft へ戻してから `git -c merge.directoryRenames=false merge origin/main` で取り込む」にする（base 同期の正本に単段 merge の command を置く。裁定 r2 の G7）。同じ段落の以降の文（GitHub の Update branch の扱い以降）は変えない。
  - `## Helperの境界`: `:142` の段落（親 docs の一覧で packet の不在を確かめる）を、「R2+ は PR の差分が触る active packet がちょうど `--packet` の 1 つであること、R0 / R1 は差分が active packet を触らないこと（closeout の移送だけを許す。移送は、同じ PR の差分の中で同名の `docs/archive/plans/` へ移し〈`renamed`、または `removed` と `added` / `modified` の組〉、archive の Phase が archive のもの）を確かめる。触る packet は差分の `filename` と `previous_filename` で決め、head の `docs/plans/` の一覧と Plans.md は読まない」の段落に替える。`:144` の「当該PR headのpacket・Plansの登録と照合する」を「当該 PR の差分が触る active packet と照合する」にする。
  - `## closeoutとActions停止時` に 1 文足す: 「merge 済みの lane の closeout は wave ごとに 1 本の R0 PR にまとめてよく、wave を閉じる前（次の wave の起票の前）に完了する。単独の lane は 1 lane の wave とみなす」。
  - `## 実行手順` の `:160` の「`PACKET`は登録された単一packet」を「`PACKET`はその PR の差分が触る単一の active packet」にする。
- S8 `docs/DEV_WORKFLOW.md`
  - Workflow State の packet 選択規則（`:110`）
    - 前半（Plans.md の current-work の link、`Wave Registry` の lane、registry の 3 種の不一致）を次に替える: 「lane の作業の再開は、依頼が名指しする lane の packet（`docs/plans/` の dated packet。packet の `Branch` 行と前文の wave・lane で特定する）から始める。active であることだけで選ばない。依頼が 1 つの packet を特定できない、packet の `Branch` と branch / PR が一致しない、PR が merge 済み（closeout 待ち）のときは、推測で選ばず停止して owner に報告する。merge 済みの packet の closeout はこの規則の対象でなく、Post-Merge Closeout に従って packet を名指しして行う」。
    - 後半の「再開時は、行動する前に packet・helper status・専用 record・CI を読み…」は残す。
  - Plan Commit ancestry の段落（`:114`、F9）: 「PK5 の祖先・不変性の検査は、branch の差分が触る packet に掛ける。起点は `WORKFLOW_BASE_SHA` が HEAD と違えばそれ、そうでなければ `origin/main`。起点が解決できないか merge-base が 1 つでなければ全 packet に掛ける（fail-closed）。merge 済みで closeout 前の packet は、差分が触らない限り対象外」の趣旨の 1 文を足す（段落は英語なので英語で書き、AC4 が引く語 `packets the branch diff touches` を含める）。他の文は変えない。
  - Wave Operation の冒頭の文（`:229`、F3）: 「lane登録と独立性を維持し、」を「lane の独立性を維持し（lane の一覧は `docs/plans/` の packet が持ち、`Plans.md` に lane ごとの行を置かない）、」にする。同じ文の残り（実装後の状態は PR から導く、base 同期は MG-D6 に従う）は変えない。
  - Wave Operation `:235`: 「現 wave と lane の task、branch、packet、Draft PR、Phase、owner 介入状況、merge train 順序は Plans.md の Wave Registry に置く」を、「lane の一覧は `docs/plans/` の packet（前文に wave と lane、Workflow State の `Branch` 行に branch）、Draft PR・Phase・現在地は packet と helper status が持つ。lane ごとに `Plans.md` へ行を足さない。merge train 順序は owner が batch Ready 承認時に指定する（`:237`）」にする。
  - 単段 merge の command（D8、裁定 r2 の G7）: `git -c merge.directoryRenames=false merge origin/main` を、Wave Operation `:238`（「`origin/main` の単段 merge で行い」の所）と Stacked train `:246`（「最新 `origin/main` を 1 回だけ merge する」の所）の両方に明記する。`:238` は Stacked train を参照するが、`:246` から `:238` への参照は無いため、どちらにも同じ command を置く（merge-evidence `:130` は S7）。
  - Draft PR Checkpoint
    - `:385`: 「Record pending manual checks in the PR body and `Plans.md`.」を「Record pending manual checks in the PR body.」にする。
    - `:390`（F3）: 「If the user explicitly asks not to create a PR, leave the branch local and record the next publish step in `Plans.md`.」を、「If the user explicitly asks not to create a PR, leave the branch local and record the next publish step in the change's Plan Packet (`## Implementation Results`); only a no-packet R0/R1 change records it in `Plans.md`.」にする。
    - この 2 点で、lane 固有の予定・状態を `Plans.md` へ書き足す経路を残さない。lane でなく全体の事項の書込み（例: 役割が決まらないときの blocker の記録、`docs/AGENT_OPERATING_MANUAL.md:50`）は残る。「lane は `Plans.md` を一切編集しない」とは定めない。
  - Post-Merge Closeout
    - `:419`: 「merge 済み lane を個別に archive し、Wave Registry の lane 状態を同期してから train の次 lane を進める」を「merge 済み lane の closeout は wave ごとにまとめてよく、train の次 lane はその closeout を待たない」にする。後半（全 lane の closeout 後の WER）は残す。
    - `:420`（裁定 r3 の H4）: 「Update `docs/PROJECT_HANDOFF.md` when its navigation targets change; live progress belongs only in `Plans.md`.」を「Update `docs/PROJECT_HANDOFF.md` when its navigation targets change. Project-level live progress belongs in `Plans.md`; a lane's state lives in its Plan Packet and helper status, not in `Plans.md`.」にする（lane の agent がこの文に従って lane の進捗を `Plans.md` へ書き足し、D6 が消した衝突源に戻るのを防ぐ）。
    - `:426`: 「先行closeoutを後続PRのbase同期より先に完了する。」の 1 文を消す（同じ行の他の文は残す）。
- S9 `docs/templates/plan-packet.md`: `## Workflow State` の箇条の末尾に `- Branch: <agent/...>` の 1 行と、「`Branch` は helper・checker が評価しない追加行。lane の branch を packet から辿るために書く」の 1 文を足す。あわせて `# Plan Packet` の見出しの直後（`## Workflow State` の前、`docs/templates/plan-packet.md:1-3`）に「前文に wave と lane を書く（単独の lane は「wave に属さない単独の lane」と書く）」の 1 行を足す（D7、裁定 r2 の G9）。
- S10 入口の語（D7）: `.agents/skills/inventory-workflow-start/SKILL.md:12` の「`Plans.md` の対象リンクから packet の」を「`docs/plans/` の対象 packet（`Plans.md` が指す一覧から、依頼が名指しするもの）の」に、`.claude/commands/plan-rally.md:7` の「`Plans.md`から対象のactive Plan Packetを一意に特定する」を「`docs/plans/` から依頼が名指しする active Plan Packet を一意に特定する」に、`docs/AGENT_OPERATING_MANUAL.md:86` の表の右列の「+ Plans.md の「Wave Registry」への link」を「+ `docs/plans/` の packet」にする。
- S11 `docs/Plans.md`（D6）
  - `## 次の行動` の先頭の段落の後に 1 行を置く: 「active な lane の Plan Packet は `docs/plans/` の dated packet が正本（lane の branch は各 packet の `Branch` 行、現在地は helper status）。lane の起票・closeout はこの節へ lane ごとの行を足さない（D-097）」。
  - 本 lane の起票で足した本 lane の行は、本 lane の closeout まで残す（下の「本 lane 自身の merge」）。他の lane の既存の行は触らない（各 lane の closeout で消える）。
  - `### Wave Registry` の「形式」の行（`:85`）を、「完了済み wave の記録だけを置く。進行中の lane は `docs/plans/` の packet が持つ（D-097）」にする。完了済み wave の記録は変えない。
- S12 `docs/decision-log.md` の末尾に `## D-097` を追記する（番号は本 packet で予約する。D-094 は並走の色と強調の lane、D-095 は小口のまとめが予約して使わない、D-096 は使用済み）。書く内容:
  - 並走の摩擦を削る判定の変更（D1〜D8 の要旨）と、G2 の採った案（B）と棄却案（A・C）。
  - 部分的に置き換える既存の決定: D-055 の「Wave Registry だけを複数 active packet の入口とする」、D-039 の PK5 の対象範囲、merge-evidence の「先行 PR の closeout を先に」。
  - D-055 の却下理由への答え（F14）: D-055 は「registry を設けず複数 active packet を全面許可する案」を fail-closed 保護を失うとして却下した（`docs/decision-log.md:424`）。本 lane では、その保護を次の 2 つが代わりに担う。
    - 読み手の選択: 依頼の名指し・packet の `Branch` 行・不一致や特定不能での停止（D7）。
    - 機械の判定: PR と packet の結び付けを差分で決める helper（D1・D2）と、差分が触る packet の PK5（D4）。
    - registry が止めていた「どの packet か分からない」状態は、停止の条件として残る。
  - B の費用（G2 の比較の「受け入れる費用」、F13）: wave ごとの closeout の PR 1 本と追いつき、PK を厳しくする lane が先に closeout を要すること、まとめた closeout による D8 への依存。
  - Revisit trigger: 本 lane の後の実際の衝突で manual の再利用の条件を owner が見直すとき。
- 登録（本 plan-first commit に同乗）: `docs/Plans.md` の `## 次の行動` に本 packet と Matrix の link を 1 行足す（現行の PK4 と helper が要求するため。本 lane の実装の前の規則に従う）。

対象を使う呼出し側・隣接 test を確認した: helper の呼出し側は `docs/agent-guidance/merge-evidence.md` の実行手順と `.local/` の運用だけで、`scripts/local-ci.sh:197`・`scripts/pre-push.sh:188`・`.github/workflows/ci.yml:312` は `check-workflow-git.sh` を引数なしで呼び（ci は `WORKFLOW_BASE_SHA` を渡す）、呼び方は変えない。`scripts/tests/local-ci.test.sh:39` と `scripts/tests/pre-push.test.sh:54` は `check-workflow-git.sh` を偽物に差し替えるので影響しない。`scripts/tests/pr-gate.test.py:353` は `check-workflow-git.sh` を `origin` の無い合成 repo で呼ぶため、起点なしの全 packet の検査のまま（S3 の fail-closed の経路）。`scripts/tests/doc-consistency-plan-packet.test.sh:848` は Phase の配列の parity だけを見る。予期しない拡張は既存の改訂経路へ戻し、「関連 file 全般」を許可範囲にしない。

### 本 lane 自身の merge（自分の変更で自分の gate を緩めない）

- 本 lane の PR の helper の操作（status・capture・record・ready・merge）は origin/main の helper で行う: `git show origin/main:scripts/pr-gate.py > "$TMPDIR/pr-gate-main.py"` を本 lane の worktree で `python3 "$TMPDIR/pr-gate-main.py" …` として使う（Gate は cwd の git で head を確かめるため、script の置き場所に依らない）。本 lane の branch の helper で自分の PR を判定しない。
- そのため本 lane は旧い規則で merge する: capture の前に、head の `docs/plans/` の active packet が本 packet 1 つだけであること（並走の小口のまとめ〈#118〉と色と強調〈#116〉の closeout が main に入り、それを本 branch へ単段 merge してあること）と、`## 次の行動` に本 packet の link があること（S11 で残す）を満たす。これが旧い規則で待つ最後の回になる。
- hosted CI は PR の head の script（新しい PK4・PK5）で走る。これは既存の設計どおりで、変更の妥当性は Double Audit が見る。

## Non-scope

- manual の再利用の条件（merge-evidence の MG-D6 の機械条件と `reuse_geometry`）、strict（MG-D1）、Merge gate・classifier（`scripts/ci/**`）・`.github/workflows/**`・`.github/merge-gate-ruleset.json`。
- `docs/decision-log.md` の追記と `90-traceability.md` の再生成による衝突: backlog に残す（本 lane の closeout で 1 項目を起こす。起こさない判断もその時の観測で決める。decision-log は lane が予約した番号の順に末尾へ足すため衝突は機械的に解け、traceability は再生成で解けるので、本 lane の 3 点ほど待ちを生んでいない）。
- `AGENTS.md` の `Session Start` の R2+ の行の「`Plans.md` から対象 packet を特定し」: `Plans.md` の pointer 行から `docs/plans/` を辿れるので意味は保たれる。drift test（`scripts/tests/reading-order-drift.test.sh`）の対象の行で、語を変える利益が小さいため変えない。
- `docs/DEV_WORKFLOW.md:68`（訂正の sweep の対象の「its `Plans.md` entry」）: entry がある場合の sweep として残せ、lane の行が無ければ該当が無いだけで誤りにならない。書込みの義務を作らない。PR4 の文面の整理に回す。`:390`（PR を作らない場合の Plans.md への記録）は書込みの義務なので S8 で直す。
- `docs/templates/test-design-matrix.md`、PR template、`docs/AGENT_OPERATING_MANUAL.md` の `:86` 以外（§5.3 の Plans.md cleanup prompt を含む。closeout の文面は Post-Merge Closeout を参照しており変えなくてよい）。
- helper の処理の中の、他の lane の Test Design Matrix（`docs/plans/test-matrices/`）の書き換えの検出: 本 lane は packet の判定だけを変える（現行も Matrix は見ていない）。Residual Test Gaps に記録する。
- `.local/checklists/kickoff.md` の [43]（packet を置く commit に Plans.md の link を同乗）と closeout の checklist: tracked 外。本 lane の merge 後に Coordinator が直す。
- `docs/Plans.md` の他の lane の既存の行と `## 直近の完了`: 本 lane の実装では触らない（S11 の 3 点だけ）。
- `scripts/check-workflow-git.sh` が Phase を packet 全体の行頭 `- Phase:` から読む（`## Workflow State` の節に限らない。Plan Commit・Amendments は最初の 1 行）脆さ: 起票時に本 packet の本文の箇条が同じ形で始まり `invalid tracked Phase` になった（2026-09-28、本文を直して回避）。S3 はこの読み方を変えない。closeout で backlog の候補にする。

## Acceptance Criteria

baseline は、origin/main `df0488eb` を取り込んだ後の本 branch の plan 側の HEAD（本 packet・Matrix・`docs/Plans.md` の登録行だけが main との差分）で、同じ command を逐語で 2026-09-29 に実行した実測（F8。`b0f3b68b` の時期の baseline は取り直した。`bash scripts/local-ci.sh full` だけは Plan Review round 1 の Codex の実測を引く）。AC の数は機能の代理にせず、test は全 PASS を求める（本数は runner の出力を参照）。

- AC1（G1 の判定、D1・D2）: `python3 scripts/tests/pr-gate.test.py -v -k PacketScope` が exit 0 で、S2 の `PacketScope` の各 test（`test_truncated_diff_rejected` を含む）が `ok`。baseline: `Ran 0 tests` / `NO TESTS RAN`、exit 5。
- AC2（helper の既存の契約、D9）: `python3 scripts/tests/pr-gate.test.py` が exit 0。baseline: exit 0（`OK`）。
- AC3（一覧と Plans.md の照合の撤去、D3）: `rg -n 'packet absent or multiple active packets|packet not registered in Plans|contents/docs\?ref|contents/docs/plans\?ref' scripts/pr-gate.py` が 0 行（exit 1）。baseline: 4 行（`:224`・`:227`・`:231`・`:234`）。
- AC4（PK5 の対象、D4、F1。`scripts/check-workflow-git.sh`）
  - `bash scripts/tests/workflow-git-checks.test.sh` が exit 0 で、S4 の (a1)・(a2)・(b)〜(f) を含む。(a2) が env 未設定で `origin/main` を起点にする経路（pre-push・local-ci）を持つ。baseline: exit 0（S4 の場面は未実装）。
  - merge 済みの packet が main に残る場面の red は、現行の checker では既存の「PK5: squash 相当の負例」（`scripts/tests/workflow-git-checks.test.sh:128-152`）が持つ。実装後は S4 (a1)・(a2) が同じ形の fixture で exit 0 を求める。現在の main には merge 済みの packet が無く、実 repo の command ではこの場面を再現できない（F8: `WORKFLOW_BASE_SHA=$(git rev-parse origin/main) bash scripts/check-workflow-git.sh` と env 未設定の `bash scripts/check-workflow-git.sh` は、どちらも baseline で exit 0）。
  - 正本: `rg -n 'packets the branch diff touches' docs/DEV_WORKFLOW.md` が 1 行（S8 の `:114` の文。F9）。baseline: 一致なし（exit 1）。
- AC5（PK4 の pointer、D6）: `bash scripts/tests/doc-consistency-plan-packet.test.sh` が exit 0（S6 の置き換えを含む）。baseline: exit 0（packet ごとの link の契約）。実物: `awk '/^## 次の行動/,/^## 直近の完了/' docs/Plans.md | rg -c 'docs/plans/'` が 1 以上。baseline: 一致なし（exit 1）。本 lane の行は closeout まで残す: 同じ出力への `rg -c '\]\(plans/2026-09-28-harness-parallel-friction\.md\)'` が 1（baseline: 1）。
- AC6（closeout を先に済ませる規則の撤去、D5）: `rg -n '先行closeoutを後続PRのbase同期より先に完了する|まず先行PRのcloseoutを完了し|lane 状態を同期してから' docs/DEV_WORKFLOW.md docs/agent-guidance/merge-evidence.md` が 0 行（exit 1）。baseline: 3 行（DEV_WORKFLOW `:419`・`:426`、merge-evidence `:130`）。新しい規則
  - base同期の節（S7 の `:130`）: `rg -n 'closeout を待たない' docs/agent-guidance/merge-evidence.md` が 1 行。baseline: 一致なし（exit 1）。base同期の節の新しい文は「差分が触らない」で `差分が触る` に一致しないため、この式を当てる（裁定 r2 の G5）。
  - Helperの境界と実行手順（S7 の `:142`・`:144`・`:160`）: `rg -n '差分が触る' docs/agent-guidance/merge-evidence.md` が 2 行以上。baseline: 一致なし（exit 1）。
- AC7（directory rename の手順、D8、裁定 r2 の G7）: `rg -c 'directoryRenames=false' docs/DEV_WORKFLOW.md` が 2（Wave Operation `:238` と Stacked train `:246`）、`rg -c 'directoryRenames=false' docs/agent-guidance/merge-evidence.md` が 1（base同期の節 `:130`）。baseline: どちらも一致なし（exit 1）。
- AC8（Plans.md の lane の登録・書込みを前提にした規則の撤去、D6・D7、F3）
  - `rg -n 'Plansの登録と照合|packet不在は親docs一覧で確認する|linked from the current-work section|owner 介入状況、merge train 順序は|各 lane に是正単位、branch、active packet link' docs/DEV_WORKFLOW.md docs/agent-guidance/merge-evidence.md docs/Plans.md` が 0 行（exit 1）。baseline: 5 行（Plans `:85`、merge-evidence `:142`・`:144`、DEV_WORKFLOW `:110`・`:235`）。
  - `rg -n 'record the next publish step in .Plans\.md.|lane登録と独立性を維持し' docs/DEV_WORKFLOW.md` が 0 行（exit 1）。baseline: 2 行（`:229`・`:390`）。
  - 入口: `rg -n 'Plans\.md. の対象リンク|Plans\.md.から対象のactive|Plans\.md\]\(Plans\.md\)「Wave Registry」' .agents/skills/inventory-workflow-start/SKILL.md .claude/commands/plan-rally.md docs/AGENT_OPERATING_MANUAL.md` が 0 行（exit 1）。baseline: 3 行（SKILL `:12`、plan-rally `:7`、MANUAL `:86`）。
  - `rg -n 'Record pending manual checks in the PR body and' docs/DEV_WORKFLOW.md` が 0 行（exit 1。baseline: 1 行 `:385`）。
  - Post-Merge Closeout の `:420`（S8、裁定 r3 の H4）: `rg -n 'live progress belongs only in' docs/DEV_WORKFLOW.md` が 0 行（exit 1。baseline: 1 行 `:420`）、`rg -n "a lane's state lives in its Plan Packet" docs/DEV_WORKFLOW.md` が 1 行（baseline: 一致なし、exit 1。2026-09-29 に起草役が実測）。
  - `rg -n '^- Branch:' docs/templates/plan-packet.md` が 1 行（baseline: 一致なし、exit 1）。
  - `rg -n '前文に wave と lane' docs/templates/plan-packet.md` が 1 行（S9 の前文の行、裁定 r2 の G9。baseline: 一致なし、exit 1）。
- AC9（durable decision）: `rg -c '^## D-097' docs/decision-log.md` が 1。baseline: 一致なし（exit 1）。
- AC10（mutation、Test Plan の MU1〜MU13）: 実装を commit した後、`$TMPDIR` の写し（`copy="$TMPDIR/friction-mut"; mkdir -p "$copy"; git archive HEAD | tar -x -C "$copy"; git -C "$copy" init -q; git -C "$copy" add -A`。改変ごとに作り直し、終わったら消す。本 repo の index・設定は触らない）で各 mutation を入れ、対応する test が red（exit 非 0）になり、改変なしの写しでは green になる。
- AC11（検査の全体、F1。すべて exit 0）
  - `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/doc-consistency-check.sh`、`bash scripts/doc-consistency-check.sh --target plan`、`git diff --check origin/main...HEAD`（commit 済みの branch の差分を見る。未 stage の変更だけを見る `git diff --check` は clean な tree で必ず exit 0 になり検査にならない。裁定 r2 の G6）、`bash scripts/local-ci.sh full` が exit 0。
  - `bash scripts/check-workflow-git.sh` を `WORKFLOW_BASE_SHA` 未設定・`origin/main` ありの実 repo（pre-push・local-ci と同じ経路）で実行して exit 0。merge 済みの packet が残る場面を実 repo で持てないときも、同じ経路は AC4 の S4 (a2) が持つ。
  - baseline: run-workflow-tests exit 0、doc-consistency exit 0（ERROR 0・WARN 1）、`--target plan` exit 0（ERROR 0・WARN 1）、check-workflow-git exit 0、`git diff --check origin/main...HEAD` exit 0（2026-09-29、裁定 r2 の反映の前の plan 側の HEAD で起草役が実測）、local-ci full exit 0（Plan Review round 1・round 2 の Codex の実測。起草役は再実行していない）。
- AC12（範囲、S 全体）: `git diff --name-status origin/main...HEAD` の変更 file が S1〜S12 の file と本 packet・Matrix に限られる。`scripts/ci/**`・`.github/**`・`scripts/local-ci.sh`・`scripts/pre-push.sh`・`AGENTS.md`・`CLAUDE.md` に本 lane 由来の差分が無い。

## Design Sources

- Requirements / spec: 該当なし（製品要件を変えない）。
- Architecture / Function / DB / Screen: 該当なし。
- Workflow 正本（本 lane が書き換える文書と、参照だけする文書）: `docs/agent-guidance/merge-evidence.md`（MG-D1 strict・MG-D5〜D9、「base同期だけでheadが変わる場合」「Helperの境界」「closeoutとActions停止時」）、`docs/DEV_WORKFLOW.md`（Workflow State の packet 選択規則と PK5 の段落、Wave Operation、Draft PR Checkpoint、Post-Merge Closeout）、`docs/templates/plan-packet.md`、`docs/Plans.md`。
- Decision log / ADR: D-034（Session Start と Workflow State）、D-039（PK5）、D-055（Wave Operation と Wave Registry）、D-085 / MG-D1〜D12（helper 経由の merge と strict）、D-090（Ordinary Operation）。本 lane は D-097 を足す。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | existing sufficient（製品コードを変えない） |
| Command / DTO / generated binding / wire shape | なし | existing sufficient |
| DB / transaction / audit / rollback / migration | なし | existing sufficient |
| Screen / UI / route state / Japanese wording | なし | existing sufficient |
| CSV / TSV / report / import / export format | なし | existing sufficient |
| Workflow gate（helper・PK4・PK5 の判定、closeout の時期、packet の選択） | `docs/agent-guidance/merge-evidence.md`、`docs/DEV_WORKFLOW.md`、`docs/templates/plan-packet.md` | updated in this PR（S7〜S9） |
| Durable decision / ADR | `docs/decision-log.md` D-097 | updated in this PR（S12） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| decision-log の追記 | D-097（本 packet で予約）。D-094 の後に並走の lane が merge した場合は番号の順に並べる |
| workflow の正本の文の変更（S7〜S10） | 親文書の索引・link の実在は `bash scripts/doc-consistency-check.sh` の link 検査（R3）で確かめる。文書の新設・改名・削除は無い |
| test の追加・置き換え（S2・S4・S6） | `scripts/tests/run-workflow-tests.sh` が既に 3 本を実行している（新しい test file は無い） |

Tauri command・function-design doc・REQ・route・operator 画面: 該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-PARALLEL-FRICTION | merge-evidence「Helperの境界」 | D1 | PR に結び付く packet は、その PR が作り・変える packet で決まる。head の一覧は main から取り込んだ他の lane の packet を含むため、PR と無関係な理由で止まる（観測 1）。却下: head の一覧のうち merge 済みを除く（merge 済みの判定に PR の API か packet の追加行が要り、誤りの向きが緩む側になる） | S1 | AC1、AC3 |
| SPEC-WF-PARALLEL-FRICTION | merge-evidence「Helperの境界」「closeoutとActions停止時」 | D2 | R0 / R1 の経路は「差分が active packet を触らない」で決め、closeout の移送だけを許す。移送は、同じ PR の差分の中で同名の archive へ移していて（`renamed`、または `removed` と `added` / `modified` の組）、head の archive の Phase が archive のときだけ認め、directory rename の推測で動いた進行中の packet（Phase は implementing 等）を拒む。却下: rename の metadata だけで判定する（GitHub が削除と追加で返す場合を取り逃す）、head の同名の archive の存在と Phase だけで判定する（古い同名の archive が残る repo で、active packet を削除するだけの PR を通す。裁定 r2 の G1）、移送を一律に許す（観測 6 の事故を通す） | S1 | AC1 |
| SPEC-WF-PARALLEL-FRICTION | merge-evidence「Helperの境界」 | D3 | dashboard の検査は PK4（hosted の docs job で走る）の 1 か所に置き、helper は PR と packet の結び付けだけを持つ（MG-D2・D4 の二重実装の回避）。却下: helper が pointer 行を照合する（同じ検査を 2 か所に持つ） | S1 | AC3 |
| SPEC-WF-PARALLEL-FRICTION | DEV_WORKFLOW「Plan Commit ancestry」、`check-workflow-git.sh` | D4 | squash merge 後の packet の Plan Commit は main の祖先でないため、merge 済みの packet が残る head は PK5 が必ず red になる（2026-09-28 に実測）。PK5 は自分の plan-first の祖先を確かめる検査なので、差分が触る packet に限っても保護は変わらない。起点が無ければ全 packet（現行）に倒す。却下: merge 済みの packet を Phase の値で除く（Phase は merge で変わらない）、PK5 を hosted だけに限る（local の gate で red が残る） | S3 | AC4 |
| SPEC-WF-PARALLEL-FRICTION | merge-evidence「base同期」「closeoutとActions停止時」、DEV_WORKFLOW Post-Merge Closeout | D5 | 下の「G2 の比較」。closeout を後回しにしてまとめる（推奨 B）。却下: lane の PR の最後の commit に含める（A）、lane ごとに A と B を選ぶ（C） | S7、S8 | AC6 |
| SPEC-WF-PARALLEL-FRICTION | Plans.md、DEV_WORKFLOW Artifact Map・Wave Operation、PK4 | D6 | 共有の節へ lane ごとに行を足す運用が取込みのたびの衝突を生む（観測 3）。active な作業の一覧は `docs/plans/` 自体が持ち、Plans.md はそこを指す 1 行を持つ。pointer は markdown link にしない（link 検査 R3 が `-f` で file の実在を見るため directory を指せない。`scripts/doc-consistency-check.sh:1716`）。却下: PK4 の Plans.md の検査を丸ごと消す（dashboard が active な作業を指さなくなっても検出できない）、Wave Registry に lane の行を残す（同じ衝突が残る） | S5、S11 | AC5、AC8 |
| SPEC-WF-PARALLEL-FRICTION | DEV_WORKFLOW Workflow State の packet 選択規則、template | D7 | Plans.md の lane の行が持っていた branch を packet の `Branch` 行へ移し、packet は依頼の名指しで選ぶ。merge 済み（closeout 待ち）の packet は lane の作業として再開しない（closeout は Post-Merge Closeout に従い packet を名指しして行う）。却下: `Branch` を必須 field にする（10 field の契約と helper・PK4・既存 packet の変更が要る。追加行で足りる） | S8、S9、S10 | AC8 |
| SPEC-WF-PARALLEL-FRICTION | DEV_WORKFLOW Wave Operation の単段 merge | D8 | `docs/plans/` が空になる merge で git の directory rename の推測が他の lane の packet を archive へ動かしかけた（観測 6）。Contract Probe 4 で再現し、`merge.directoryRenames=false` で起きないことを確かめた。D2 と D1 が起きた場合も検出する。却下: `docs/plans/README.md` で directory を空にしない（`check-workflow-git.sh` の Phase の検査と `--target plan` の既定の対象に入り、`test-matrices/` にも別に要る） | S8 | AC7 |
| SPEC-WF-PARALLEL-FRICTION | merge-evidence 全体 | D9 | 守る境界を変えない: record の head/base、approved snapshot の 3 条件、Double Audit の下限、main 側の classifier、R0 / R1 の `--risk` / `--manual`、CI 実行 code の R0 / R1 の拒否、manual の再利用の条件、strict | S1（変えないもの） | AC2、AC11 |

## G2 の比較（設計判断。Plan Review で覆せる形）

owner に示した案は A（closeout を lane の PR の最後の commit に含める）。Coordinator の見立て（G1 で merge 済みの packet が他の lane を止めなくなるので、closeout を後回しにしてもまとめても止まらない）は、helper については成り立つが、PK5 については成り立たない: 2026-09-28 に base `b0f3b68b` で `bash scripts/check-workflow-git.sh` が exit 1（merge 済みの小口のまとめの packet の Plan Commit が squash merge で祖先でなくなる）。そのため B と C は D4 を前提にする。

| 観点 | A: lane の PR の最後の commit に closeout を含める | B: closeout を後回しにし、wave ごとにまとめる（推奨） | C: 組み合わせ（manual の無い lane は A、manual のある lane は B） |
|---|---|---|---|
| 他の lane を止めるか | 止めない（main に merge 済みの packet が残らない） | 止めない（D1・D2・D4） | 止めない |
| review・L3 の後に head が変わるか | 変わる。Review Response は Final Review の後にしか書けないので、closeout の commit は review の後になり、closure が 1 回増える（Codex の relay も 1 増える）。manual の lane は manual の record が旧 head のものになり、再利用は base 同期の単段 merge だけに限られる（非目的で緩めない）ため L3 のやり直しになる。closeout の commit を L3 の前に置けば避けられるが、L3 の FAIL の是正でまた書き直す | 変わらない | manual の無い lane は A と同じ closure の増加、manual のある lane は B と同じ |
| helper・Workflow State・PK5 の変更 | 自分の packet を head の `docs/archive/plans/` から読む経路、merge 前に Phase archive を受ける変更（遷移表の implementing → archive の条件「helper で merge 済み」の書き換え）、PK5 が archive へ移した自分の packet の Plan Commit を検査する経路（無いと最終 head で祖先の検査が抜ける）が要る | D1・D2・D4 だけ。遷移表・自分の packet の読み方・PK5 の自分の packet の検査は変わらない | A と B の両方の経路と、その選び分けの fail-closed |
| Plans.md の衝突 | lane の PR が `## 直近の完了` へ書き足すため、取込みのたびに衝突する（G3 の目的に反する）。書き足しを lane の PR から外すと、dashboard の更新だけを B と同じくまとめて後で行う形になる | lane の PR は Plans.md に lane 固有の行を足さない。lane の完了を書き足すのはまとめた closeout の PR だけで、lane とは衝突しない | manual の無い lane は A と同じ |
| PR の本数 | lane ごとに 1 本 | lane ごとに 1 本 + wave ごとに closeout 1 本 | 間 |
| strict の追いつき | 増えない | closeout の merge で他の Ready の PR の追いつきが wave ごとに 1 回増える | 間 |
| 記録の正確さ | archive へ移す時点では merge の SHA・最終の CI が分からない | merge 後に書くので揃う | 間 |

推奨: B。

- 理由
  - (1) 安全の境界を担う code の変更が最も少ない（自分の packet の読み方・遷移表・PK5 の自分の packet の検査を変えない）。
  - (2) review と L3 の後に head を変えないので、owner が緩めないと決めた manual の再利用の条件のもとでも L3 のやり直しが起きない。
  - (3) lane が Plans.md に lane 固有の行を足さないので G3 の目的と両立する。
- 受け入れる費用
  - wave ごとに 1 本の closeout の PR と、その merge による main の更新 1 回（同期する PR が残っていれば、その各 PR に追いつきが起きる）。
  - merge 済みの packet が wave を閉じるまで `docs/plans/` に残る（D7 の選択規則で lane の作業の再開の対象から外し、helper status で merge 済みと分かる）。
  - その間に PK1〜PK4・PK6 を厳しくする lane が入ると、残っている merge 済みの packet で red になりうる。D1・D2 は他の lane の packet の修正を拒むので、PK を厳しくする lane は、先に merge 済みの packet の closeout を済ませる必要がある（F13）。merge 済みの packet の link 先の doc を動かす（改名・削除する）lane も同じで、link 検査（`scripts/doc-consistency-check.sh` の R3 は `docs/` 配下の全 `.md` を走査する、`:1682`）が merge 済みの packet で red になり、その packet の修正は D1・D2 が拒むので、先に closeout が要る（裁定 r2 の G9）。
  - まとめた closeout は `docs/plans/` を空にしやすく、並走する lane の同期が D8（`merge.directoryRenames=false`）に頼る度合いが強まる。D8 を守らずに archive へ動いた packet は D1・D2 が拒むが、是正の手間が生じる（F13）。

closeout の期限とまとめ方（B）: merge 済みの lane の closeout は wave ごとに 1 本の R0 PR にまとめ、wave を閉じる前（次の wave の起票の前）に完了する。単独の lane は 1 lane の wave とみなす（merge の後の最初の closeout の機会）。早めに出すのは妨げない。closeout の PR は他の lane の PR の同期を待たせない。この期限を機械で守らせる検査は無く、merge 済みの packet が `docs/plans/` にたまりうる。Wave Operation の観測項目とし、本 lane の WER（`docs/DEV_WORKFLOW.md:454`）と以後の wave の WER で、wave を閉じる時点に merge 済みの packet が残っていないか、期限を越えて残った packet が他の lane の待ち（上の PK・link の変更）を生まなかったかを見る（裁定 r2 の G9）。

merge 済みで closeout 前の packet が main に残る間の扱い（B）:

- Phase の値: `implementing` のまま（実装後の状態は PR の native state が持つ、MG-D5。merge 済みかは helper status が `merged` を返す）。closeout の commit で `archive` にする（遷移表は変えない）。
- PK 検査: PK1〜PK4・PK6 は他の active packet と同じく掛かる（形式は merge 時点で満たしている）。PK5 の祖先の検査は、その packet を差分が触る PR だけに掛かる（D4）。closeout の PR は packet を `docs/plans/` から出すので PK5 の対象外。
- Plans.md: 何もしない（lane の行が無い）。`## 次の行動` の pointer 行が `docs/plans/` を指し、merge 済みかどうかは packet の `Branch` 行から PR を引いて helper status で分かる。

owner に諮る判断点: 推奨 B は owner の示した A と違うため、Plan Gate で owner の選択（A / B / C）を得る。**owner は 2026-09-29 に B を選んだ（決着）。**A か C を選べば、Scope（S1 に自分の packet の archive からの読み方、S3 に archive の自分の packet の PK5、S8 に遷移表の変更）と Owner Effort Budget（closure の増加）を改めて design へ戻る。

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 規則は S7〜S11 の正本に、判断と棄却案は D-097 に置く。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D1〜D8 を merge-evidence・DEV_WORKFLOW・template と D-097 に置く（実装の S7〜S12）。G2 の比較の表は D-097 に要約を置く。
- Assumptions and constraints: hosted の workflow_dispatch の run の docs job で `origin/main` が解決できる（Contract Probe 5、外部前提。解決できなくても fail-closed。PR の run は `WORKFLOW_BASE_SHA` を起点にする）。hosted の PR run の起点 `WORKFLOW_BASE_SHA` = `github.event.pull_request.base.sha`（`.github/workflows/ci.yml:311`）が、後続 lane が単段 merge で取り込んだ main 以上である（外部前提、裁定 r2 の G4。外れると先行 lane の merge 済みの packet も差分に出て PK5 が red になり、安全は弱まらない。確かめ方は Ordinary Operation の 4 行目のとおり、本 lane の merge 後に main を取り込んだ最初の PR の docs job の step log の `WORKFLOW_BASE_SHA` と取り込んだ main の突き合わせ〈read-only〉）。GitHub の PR の files API が `status` と `renamed` の `previous_filename` を返し（Probe 1）、1 PR あたり最大 3000 files を返す（S1 の切り詰めの検出の前提、公式資料）。本 lane の PR は旧い helper で merge する（「本 lane 自身の merge」）。
- Deferred design gaps, risk, and follow-up target: 他の lane の Test Design Matrix の書き換えの検出（Residual Test Gaps）、decision-log と traceability の衝突（Non-scope、closeout で backlog の判断）、衝突を解いた版の manual の再利用（owner が本 lane の後の観測で決める）。
- Test Design Matrix can cite design decision IDs or source doc sections: D1〜D9 を Matrix が引く。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「merge 済みの packet は他の PR を止めない」の例外 = その packet を差分が触る PR（他の lane の packet の書き換え、closeout）。前者は、`--packet` と並んで触れば D1、R0 / R1 で触れば D2 で拒否する（`--packet` 自体にその packet を名指しする経路は helper では拒めず、Residual Test Gaps に置く）。後者は同じ PR の差分の中で同名の archive へ移し、head の archive の Phase が archive の移送だけを通す（既存の archive が無変更のまま active packet が消える差分は拒否）。「PK5 は差分が触る packet だけ」の例外 = 起点が解決できないときと merge-base が 1 つでないとき（全 packet に倒す）と、`WORKFLOW_BASE_SHA` が HEAD と同じ dispatch（`origin/main` を起点にする）。main 上の dispatch では差分が空になり PK5 の祖先の検査は掛からないが、main の packet は各 PR の head で検査済み（現行でも main の dispatch は merge 済みの packet で red になっており、検査として働いていなかった）。

## Impact Review Lenses

安全の境界が弱まらないことを、owner の指定した 4 点を中心に論じる。現場調査・実機・POS・CSV の lens は該当なし（1 行ずつ理由を書く）。

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 該当なし: 製品の adapter と core を変えない | — |
| Fact check / design decision split | 事実 = wave 14 の観測 1〜6、PK5 の red（2026-09-28 実測）、GitHub の files API の返し方（PR #118・#119・#120・#116 を実測）、git の directory rename の挙動（probe で再現）。判断 = D1〜D8 と G2 の推奨 | Contract Probe、D-097 |
| Lifecycle / retry | lane の packet の状態 = 起票 → 実装 → merge 済み（closeout 待ち）→ archive。merge 済みの間の Phase・PK・Plans.md の扱いを G2 の節に書いた。closeout の PR が失敗・中断しても他の lane は止まらない | Matrix の State Lifecycle |
| Operator workflow | 該当なし（店の operator の操作を変えない）。開発の操作列は Ordinary Operation | — |
| Replacement path | helper の判定の入力を head の一覧から PR の差分へ替える。GitHub の API の形が変わった場合、未知の `status` は exit 2 で止まる | S1 |
| Data safety / evidence | 古い証拠の拒否: record の head/base の照合、capture と server の record の一致、closure の要求、approved snapshot の Risk・Minimum・Human Gate の照合はどれも変えない（D9、AC2）。判定の入力が差分に替わっても record は現在の head/base に結び付いたまま | AC2、AC11 |
| Reporting / accounting semantics | 該当なし | — |
| Manual verification | 該当なし: 画面が無い。helper・checker は合成 fixture で確かめる | — |
| 環境・再現性 | PK5 の起点は環境ごとに違う: hosted の PR の run は `WORKFLOW_BASE_SHA` = PR の base（`.github/workflows/ci.yml:311`、S4 (a1)）、pre-push・local-ci は local の `origin/main`（S4 (a2)）、hosted の workflow_dispatch の run は checkout（fetch-depth 0）の `origin/main`（Contract Probe 5、外部前提）。`merge.directoryRenames=false` は repo の設定（`.git/config`）でなく command に付け、環境に依らない | S3、S8 |

owner が指定した 4 点:

- 古い証拠の拒否: 上の Data safety の行。helper の判定は PR の差分（GitHub が PR の現在の head/base から返す）で行い、record の対象版の照合は変えない。
- Plan Commit の固定: 自分の packet は PR の差分に必ず出る（plan-first commit が branch にあり、merge-base 以降の差分に入る）ので、PK5 の祖先・不変性の検査は変わらずに掛かる。approved snapshot の照合も変えない。自分の packet を差分に出さずに R2+ の作業をする経路は、`--packet` に自分の packet を名指しすれば差分の packet と一致せず拒否される。自分の packet を持たない stacked の branch が、merge 済みの他の lane の packet を `--packet` に名指しして書き換える経路は、helper では拒めない（helper は `--packet` がどの lane の packet かを知らない。現行の helper でも、その packet が head の唯一の packet なら通るので退行ではない。Plan Review round 3 の Opus が合成 repo で確認）。stacked でなければ、その packet の Plan Commit は squash merge のため祖先でなく PK5 が red にする。stacked で残る境界は、その packet の Plan Commit の書換えの検出（PK5、`scripts/check-workflow-git.sh:91-103`）、承認済みの Risk・Final Review Minimum・Human Gate の変更の拒否（`scripts/pr-gate.py:235-242`）、独立 review、owner の Ready・merge の承認（Residual Test Gaps）。
- 他の lane の packet の改変の検出: 差分が `--packet` 以外の active packet を触る PR を拒否する点で強くなる。現行は head の packet の数だけを見ていたため、他の lane の packet を書き換えても数が変わらなければ検出しなかった。新しい判定は、R2+ では差分が `--packet` 以外の active packet を触った時点で（D1）、R0 / R1 では active packet を触った時点で（D2、closeout の移送を除く）拒否する。PK5 も、差分が触る merge 済みの packet の祖先の破れで red になる。`--packet` 自体に他の lane の packet を名指しする経路は上の「Plan Commit の固定」のとおり helper の判定の外に残る。
- R0 の経路の乱用: 現行の R0 の拒否（head に packet があれば止める）は PR と無関係な理由で掛かっていた（main に packet が無い時期は R0 の申告を何も止めていない）。新しい判定は、R0 の PR が active packet を編集・削除すること自体を拒み、closeout の移送は、同じ PR の差分の中で同名の `docs/archive/plans/` へ移し（`renamed`、または `removed` と `added` / `modified` の組）、head の archive の Phase が archive のときだけ通す。同名の古い archive が既にあっても、差分に archive 側の entry が無い削除は拒否する（裁定 r2 の G1）。R0 の申告で CI 実行 code を変える PR の拒否（`CI execution change requires R3 packet`）と main 側の classifier は変えない。業務上の Risk の申告の正しさは現行どおり owner と model が判断する。packet を触らない runtime の変更を R0 と申告する PR は、現行と同じく機械では拒めない（classifier では BIZ の変更が `workflow=false` になり、CI 実行 code の拒否に当たらない。`src-tauri/src/biz/csv_import_service/commit.rs` を `bash scripts/ci/classify-changes.sh --files-from-stdin` に通して `rust=true`・`workflow=false`、2026-09-29 実測）。本 lane が拒む「R2+ で差分の packet が 0」は `--packet` を付けた PR に限る。

## Design Readiness

- Existing design docs are sufficient because: 足りない（spec-check の結果）。design の出力は本 packet の Spec Contract と G2 の比較にあり、正本への昇格は S7〜S12。
- Source docs updated in this PR: merge-evidence、DEV_WORKFLOW、plan-packet template、入口の 3 file の語、Plans.md、decision-log D-097。
- Design gaps intentionally deferred: Non-scope の各項目。
- Durable decisions discovered in this plan and promoted to source docs: D-097。

Minimum design checks for business-app work: 製品コードを変えないため、layer・function・DTO・永続化・画面・error の各項目は該当なし。testability は Matrix。

## Contract Probe

- 1「GitHub の PR の files API は、closeout の移送を `renamed` と `previous_filename` で返し、main を取り込んだ lane の PR の差分は自分の packet だけを含む」: `gh api -X GET 'repos/kosei-w90607/inventory-system-desktop/pulls/N/files?per_page=100'`（2026-09-28、read-only）を #120・#119（closeout: `renamed docs/archive/plans/… ← docs/plans/…`）、#118（#114 の packet が main にある時期に merge した lane: 差分の packet は `added docs/plans/2026-09-27-small-fixes-batch.md` だけ）、#116（open の lane: 自分の packet の `added` だけ）で読んだ → 成立。
- 2「GitHub が移送を `renamed` でなく削除と追加で返す場合がある」: 類似度の閾値による（未実測）→ 設計は依存しない。D2 は `removed` の `filename` も離れる packet として扱い、同じ PR の `added` / `modified` の同名の archive との組を移送と認めたうえで、head の archive の Phase で判定する（`test_r0_closeout_reported_as_remove_and_add_passes`）。
- 3「merge 済みの packet が main に残ると PK5 が red になる」: base `b0f3b68b` の worktree で `bash scripts/check-workflow-git.sh` → exit 1、`docs/plans/2026-09-27-small-fixes-batch.md の Plan Commit '06f12ca8…' は現在の HEAD の祖先ではありません`。`git merge-base --is-ancestor 06f12ca8… b0f3b68b` も exit 1 → 成立（Coordinator の観測に無かった 4 つ目の待ち。G2 の比較の前提）。
- 4「`docs/plans/` が空になる main の closeout を lane が取り込むと、git が lane の packet を archive へ動かす候補にする」: `$TMPDIR` の合成 repo（git 2.53.0。main が最後の packet を archive へ移し、lane が別の packet を足す）で `git merge main` → `CONFLICT (file location)`（lane が足した `docs/plans/2026-01-02-b.md` を、main で rename された directory の中の追加とみなし `docs/archive/plans/2026-01-02-b.md` への移動を示す）、作業 tree では lane の packet が `docs/archive/plans/` にある。`git -c merge.directoryRenames=false merge main` は衝突なしで lane の packet が `docs/plans/` に残る → 成立（D8）。
- 5「hosted の workflow_dispatch の run の docs job（`actions/checkout@v6`、`fetch-depth: 0`）で `origin/main` が解決できる」（F11）
  - 関係する run: hosted の PR の run では `WORKFLOW_BASE_SHA` = PR の base で HEAD と違い、S3 は `origin/main` を使わない（`.github/workflows/ci.yml:311`）。`origin/main` を使うのは、env が `github.sha` = HEAD になる workflow_dispatch の run だけ。PR の run の log では確かめられない。
  - 裏付け: checkout v6 の全履歴の fetch は `refs/heads/*` を `refs/remotes/origin/*` へ取る（https://github.com/actions/checkout/blob/v6/src/ref-helper.ts#L65 、Plan Review round 1 の Codex の確認）。
  - 閉じ方: 外部前提として残す。本 lane は確かめるための dispatch の run を起こさない（hosted の run を 1 本増やすだけで、PR の merge の経路に使わない）。解決できなければ S3 は全 packet の検査に倒れ（red の向き）、安全は弱まらない。後に dispatch を recovery で使った run が出たら、その log で確かめる。
- 6「現行の helper の判定」: `scripts/pr-gate.py:223-234`（head の一覧、`packet absent or multiple active packets`、`packet not registered in Plans`）と `:243-244`（`R2+ active packet requires --packet`）を読んだ → 成立。
- 7「link 検査は directory を指す link を拒む」: `scripts/doc-consistency-check.sh:1716`（`[ ! -f "$resolved_path" ]`）→ 成立。D6 の pointer は文字列 `docs/plans/` にする。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D1 R2+ は差分が触る active packet がちょうど `--packet`、切り詰めの疑いのある差分（entry 0 か 3000 以上）は exit 1 | S1 | AC1（`PacketScope` の R2+ の行、`test_diff_across_pages_rejected`、`test_truncated_diff_rejected`）、AC10 の MU1・MU2・MU4・MU10・MU12 | `--packet` に他の lane の packet を名指しする経路（Residual Test Gaps） |
| D2 R0 / R1 は active packet を触らない、closeout の移送は同じ PR の中の同名の archive への移送で Phase archive のものだけ | S1 | AC1（R0 の行と `test_r0_removal_with_preexisting_archive_rejected`）、AC10 の MU3・MU5・MU11 | — |
| D3 helper は head の一覧と Plans.md を読まない | S1 | AC1（`test_packet_scope_does_not_list_docs_or_read_plans`）、AC3 | — |
| D4 PK5 の祖先の検査は差分が触る packet、起点なし・merge-base が 1 つでなければ全 packet | S3 | AC4（S4 (a1)・(a2)・(b)〜(f)）、AC10 の MU6・MU7・MU9・MU13 | hosted の dispatch の `origin/main`（Probe 5、外部前提） |
| D5 closeout は wave ごとにまとめ、後続 lane は待たない | S7、S8 | AC6 | owner の G2 の判断（Plan Gate） |
| D6 Plans.md は `docs/plans/` を指す 1 行、PK4 はその行を求める | S5、S6、S11 | AC5、AC10 の MU8 | — |
| D7 packet の `Branch` 行と依頼の名指しによる選択、入口の語 | S8、S9、S10 | AC8 | — |
| D8 単段 merge は `merge.directoryRenames=false` | S8 | AC7 | — |
| D9 守る境界（record の head/base、approved snapshot、Double Audit、classifier、R0 / R1 の要求、manual の再利用、strict） | S1 の変えないもの | AC2（既存の test 全体） | 非目的 |
| 隣接: PK4 の Workflow State の形式の検査（10 field・enum・Findings Freeze） | 変えない | AC5（既存の section 1〜10・14〜27） | — |
| 隣接: PK5 の Plan Commit の書換え・Amendments の prefix（D-039・MG-D5） | 変えない（対象の packet に掛かる） | AC4（既存の場面）、AC2（`test_registered_amendment_order_cannot_roll_back_manual`） | — |
| 隣接: closeout の PR は R0 で自身の closeout を要求しない（MG-D9） | 変えない | AC1（`test_r0_closeout_archive_move_passes`） | — |
| 隣接: Plan Packet の Plans.md の登録（本 plan-first commit） | 本 packet の登録行 | `--target plan` | 本 lane の closeout で消す |

adjacent-contract sweep: S7〜S11 が触る節で上表に無い契約は、merge-evidence の RecordV1・capture / record の順序・Actions 停止時の扱い、DEV_WORKFLOW の遷移表・fail-closed・Evidence Ownership・Stacked train の base 付け替え（どれも変えない）。

## Test Plan

Test Design Matrix: [2026-09-28-harness-parallel-friction.md](test-matrices/2026-09-28-harness-parallel-friction.md)

- targeted tests: `python3 scripts/tests/pr-gate.test.py`（`-k PacketScope` を含む）、`bash scripts/tests/workflow-git-checks.test.sh`、`bash scripts/tests/doc-consistency-plan-packet.test.sh`、`bash scripts/doc-consistency-check.sh`、`--target plan`、`bash scripts/check-workflow-git.sh`。
- negative tests: Matrix の拒否の行（2 packet、2 ページ目にある他の lane の packet、`--packet` の不一致、R0 の編集・削除、同名の古い archive が無変更のままの削除、Phase archive でない移送、他の lane の packet の移送、未知の status、起点なしの PK5、pointer 無し）。
- mutation（AC10、各 mutation は実注入して red を確かめる。構造の推論だけで済ませない）:
  - MU1: S1 の触る packet を head の一覧（旧 `:224-228`）に戻す → `test_other_lane_packet_in_head_r2_passes` が red。
  - MU2: R2+ の条件を「`--packet` が触る packet に含まれる」（部分集合）に緩める → `test_two_active_packets_in_diff_rejected` と `test_r2_archiving_other_lane_packet_rejected` が red。
  - MU3: R0 / R1 の経路から触る packet の検査を外す → `test_r0_edit_of_active_packet_rejected` が red。
  - MU4: 触る packet の計算から `previous_filename` を外す → `test_r2_archiving_other_lane_packet_rejected` と `test_rename_inside_plans_counts_both_paths` が red。
  - MU5: 離れる packet の Phase archive の検査を外す → `test_r0_archive_move_without_phase_archive_rejected` が red。
  - MU6: S3 の対象を全 packet に戻す → S4 (a1)・(a2)（merge 済み packet が対象外で exit 0）が red。
  - MU7: S3 の対象を常に空にする → S4 (b)・(c) が red。
  - MU8: S5 の pointer の検査を外す → S6 の「pointer が無ければ ERROR」が red。
  - MU9（F1）: `WORKFLOW_BASE_SHA` が未設定のとき `origin/main` を使わない（env が HEAD と同じときだけ `origin/main`、env 無しは全 packet）→ S4 (a2) が red。
  - MU10（F2）: 切り詰めの検出を path の数に戻す（`len(paths) < 6000`、path は `previous_filename` があるときだけ足す）→ `test_truncated_diff_rejected` が red（3000 entry が 3000 path になり受理される）。
  - MU11（裁定 r2 の G1）: S1 (1) の移送の対応の検査を外し、離れる packet を head の archive の Phase だけで判定する → `test_r0_removal_with_preexisting_archive_rejected` が red（古い archive A の Phase archive で受理される）。
  - MU12（裁定 r3 の H2）: S1 の触る packet を 1 ページ目（`pages[0]`）の entry だけから集める → `test_diff_across_pages_rejected` が red（2 ページ目の他の lane の packet を見落として受理される）。
  - MU13（裁定 r3 の H8）: S3 の fail-closed の fallback（起点が解決できない・merge-base が 1 つでないときの全 packet）を空集合にする → S4 (d)・(f) が red（merge 済み packet を検査せず exit 0 になる）。
- compatibility checks: 既存の helper の test 全体（AC2）、PK4 の既存の section（AC5）、PK5 の既存の場面（AC4）。
- data safety checks: 変更は script・test・文書だけで、実データ・secret を含まない（Data Safety）。
- main wiring/integration checks: `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/local-ci.sh full`、実装の PR の hosted CI（PR の run は `WORKFLOW_BASE_SHA` の経路。dispatch の経路は Probe 5 の外部前提）。

## Boundary / Wire Contract

- producer: GitHub REST の `GET /repos/{owner}/{repo}/pulls/{n}/files`（`--paginate --slurp`）と `GET /contents/{path}?ref=` 。
- consumer: `scripts/pr-gate.py` の `Gate.requirements()`。
- wire type: file entry の `status`（`added` / `removed` / `modified` / `renamed` / `copied` / `changed` / `unchanged`）、`filename`、`previous_filename`（renamed のとき。他の status で来ても触る packet には数えない）。contents は base64 の本文。
- internal type: 触る packet の path の集合 2 つ（head にある、離れる）。
- precision/range: 差分の entry の数が 0 か 3000 以上（GitHub の上限）なら exit 1（`PR diff unavailable or truncated`）。path の数（旧 `:216` の 6000 未満）では判定しない。
- round-trip path: なし（read-only の判定）。
- invalid input: 未知の `status`、`removed` / `renamed` の entry の欠損 field は exit 2。`docs/plans/` を離れる packet に、同じ PR の差分の中の同名の `docs/archive/plans/` への移送（`renamed`、または `removed` と `added` / `modified` の組）が無ければ exit 1（既存の archive が無変更の削除を含む）。移送があって archive の内容の取得に失敗（404 を含む）すれば exit 2。取得できた archive の内容の Phase が `archive` でなければ exit 1。
- compatibility: CLI の引数・exit code・record の wire（RecordV1）・capture の形は変えない。capture の `requirements` の中身も同じ key。

## Review Focus

- Plan Review の冒頭で `Ordinary Operation` の操作列が目的を達成できるかを `成立 / 具体的な反例あり / 外部前提が未確認` で答える（Probe 5 は外部前提）。
- G2 の比較: 推奨 B の理由と棄却した A・C の評価が現物と合うか。特に A の「review の後の head の変化と manual のやり直し」と「Plans.md の `## 直近の完了` の衝突」の評価、Coordinator の見立てに PK5 が抜けていたという指摘（Probe 3）。
- D1・D2 の判定が、他の lane の packet の改変、R0 の乱用、directory rename の事故、GitHub の rename の返し方の揺れのどれかを通さないか。R2+ の lane の PR が自分の packet を差分に出さない場面が現実にあるか（あれば fail-closed で止まることの影響）。
- D4 の起点の決め方（`WORKFLOW_BASE_SHA` → `origin/main` → 全 packet）が、pre-push・local-ci・hosted の PR・hosted の dispatch（PR の branch と main）のどれでも、自分の packet を検査から外さないか。
- D6 の pointer を文字列の検査にしたこと（link にしない理由、inline code を数える理由）と、PK4 の検査を丸ごと消す案との比較。
- S2 の既存の test の置き換えが、退役する入力の代わりに同じ失敗を新しい入力で拒むものになっているか（弱めていないか）。
- 「本 lane 自身の merge」で旧い helper を使う手順が成り立つか（Gate が cwd の git を使うこと）。

## Spec Contract

Contract ID: SPEC-WF-PARALLEL-FRICTION

- D1: R2+ の PR は、差分（`removed` 以外の entry の `filename`、`renamed` の `previous_filename`、`removed` の `filename`）が触る `docs/plans/` 直下の dated packet（full path が `docs/plans/\d{4}-\d\d-\d\d-[^/]*\.md`）がちょうど `--packet` の 1 つで、それが head にあるときだけ helper の判定を通る。差分の entry が 0 か 3000 以上（切り詰めの疑い）なら判定せず exit 1。
- D2: R0 / R1 の PR は、差分が head にある active packet を触らないときだけ通る。差分で `docs/plans/` を離れる packet は、その PR の差分の中で同名の `docs/archive/plans/<同じ名前>` への移送（`renamed` で `previous_filename` = active・`filename` = archive、または `removed` の active と `added` / `modified` の archive の組）が起きていなければ拒む（exit 1。既存の archive が無変更のまま active packet が消える差分を含む）。移送が起きていれば head の archive を取得し、Phase が `archive` のとき closeout の移送として通し、`archive` でなければ拒む（exit 1）。archive が取得できなければ exit 2。
- D3: helper は head の `docs` / `docs/plans` の一覧と `docs/Plans.md` を読まない。
- D4: PK5 の Plan Commit / Amendments の祖先・不変性の検査は、起点（`WORKFLOW_BASE_SHA` が HEAD と違えばそれ、そうでなければ `origin/main`）と HEAD の merge-base からの差分（`--no-renames`）が触る、HEAD にある packet に掛ける。起点が解決できないか merge-base が 1 つでなければ全 packet に掛ける。Evidence Mode・Phase・shallow の検査は全体に掛ける。
- D5: merge 済みの lane の closeout は wave ごとに 1 本の R0 PR にまとめてよく、wave を閉じる前に完了する。後続 lane の base 同期は先行 lane の closeout を待たない。merge 済みで closeout 前の packet は Phase implementing のまま `docs/plans/` に残る。
- D6: `docs/Plans.md` の `## 次の行動` は `docs/plans/` を指す 1 行を持ち、lane の起票・closeout はこの節へ lane ごとの行を足さない。PK4 は active packet があるとき、その節（code fence と HTML comment を除く）に `docs/plans/` の文字列を求める。lane 固有の予定・状態は packet に書く。lane でなく全体の事項（例: 役割が決まらないときの blocker、`docs/AGENT_OPERATING_MANUAL.md:50`）の Plans.md への書込みは残る。
- D7: packet は Workflow State に helper・checker が評価しない `Branch` 行を持ち、前文に wave と lane を書く。lane の作業の再開は依頼が名指しする packet から始め、特定できない・branch が一致しない・merge 済みのときは停止する。merge 済みの packet の closeout はこの停止に当たらず、Post-Merge Closeout に従う。
- D8: lane の base 同期の単段 merge は `git -c merge.directoryRenames=false merge origin/main` で行う。
- D9: record の head/base の照合、approved snapshot の Risk・Final Review Minimum・Human Gate の照合、Double Audit の下限、main 側の classifier、R0 / R1 の `--risk` / `--manual` の要求と CI 実行 code の拒否、manual の再利用の条件、strict は変えない。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-PARALLEL-FRICTION-D1 | S1、S2 | AC1、AC10（MU1・MU2・MU4・MU10・MU12） | 他の lane の packet の改変 | test 出力と mutation の exit |
| SPEC-WF-PARALLEL-FRICTION-D2 | S1、S2 | AC1、AC10（MU3・MU5・MU11） | R0 の乱用、directory rename | test 出力と mutation の exit |
| SPEC-WF-PARALLEL-FRICTION-D3 | S1 | AC1、AC3 | 二重実装 | rg 出力 |
| SPEC-WF-PARALLEL-FRICTION-D4 | S3、S4 | AC4、AC10（MU6・MU7・MU9・MU13） | 起点の決め方 | test 出力と実物の exit |
| SPEC-WF-PARALLEL-FRICTION-D5 | S7、S8 | AC6 | G2 の比較 | rg 出力、owner の判断 |
| SPEC-WF-PARALLEL-FRICTION-D6 | S5、S6、S11 | AC5、AC10（MU8） | pointer を文字列にしたこと | test 出力と rg 出力 |
| SPEC-WF-PARALLEL-FRICTION-D7 | S8、S9、S10 | AC8 | 選択規則の fail-closed | rg 出力 |
| SPEC-WF-PARALLEL-FRICTION-D8 | S8 | AC7 | 手順の所在 | rg 出力、Contract Probe 4 |
| SPEC-WF-PARALLEL-FRICTION-D9 | S1 の変えないもの | AC2、AC11 | 境界の弱化 | test 出力 |

## Data Safety

- 変更は tracked の script・test・文書だけ。実 POS / 店舗データ、DB、backup、log、receipt、secret、`.env*` を読まず、commit しない。
- local-only: `.local/`（Codex の発注書、helper の capture）、`$TMPDIR`（AC10 の mutation の写し、Contract Probe 4 の合成 repo）。
- synthetic-only: helper・checker の test の fixture（合成の path・SHA・packet の本文）。GitHub の実 PR は read-only の GET でだけ読み、fixture に写さない。

## Implementation Results

Fill after implementation.

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- 実装: [PR #123](https://github.com/higekiri-w/inventory-system-desktop/pull/123)。S1〜S12 を Writer（Opus 5.5 subagent、発注 133）が 4 commit で実装した（helper、PK5、PK4、文書と D-097）。AC1〜AC12 を満たし、MU1〜MU13 を実注入して red を確かめた。S10 の `.claude/commands/plan-rally.md` も sandbox に拒まれず編集できた。
- 文言の細部で packet の逐語から離れた所: D-097 は closeout での移送で link が切れないよう、packet を link でなく名前で参照した。DEV_WORKFLOW の S8 の文から行番号の参照を省いた。Scope・AC は変えていない。
- commit 3（PK4）だけを checkout すると、実 repo の self-pass の test が Plans.md の pointer 行（commit 4）の前なので red になる。最終 head では green。
- 実装の要旨: helper は PR に結び付く packet を head の `docs/plans/` の一覧でなく PR の差分が触る active packet で決め（R2+ はちょうど `--packet` の 1 つ、R0 / R1 は active packet を触らず、closeout の移送は同じ PR の中の同名の archive への移送で Phase archive のものだけ）、Plans.md の link の照合をやめた。PK5 は branch の差分が触る packet に絞り、起点が解決できない・merge-base が 1 つでない・差分の取得に失敗したときは全 packet に倒す。PK4 は `## 次の行動` に `docs/plans/` を指す行を求める。merge-evidence・DEV_WORKFLOW・template・入口の語を、closeout を待たない base 同期（`merge.directoryRenames=false` の単段 merge）と wave ごとにまとめる closeout へ書き換え、D-097 を置いた。
- review の是正: Final Review broad の P2 1 件・P3 4 件と closure 1 の P3 1 件（ASCII でない名前の packet が PK5 から外れる）を Writer と Coordinator が Ready の前に直した（Review Response）。Fable broad の P3-3 は本 closeout で backlog の項目を閉じ、closure 2 の P3 は backlog に置いた。
- CI・manual・merge: hosted CI は全ジョブ pass。manual は not-required（製品の画面が無い）。Ready・merge は旧い helper 経由（2026-09-29）。

## Review Response

Fill after review.
- Final Review broad（2026-09-29、対象 `77443223`、互いに独立の 2 本）: Claude 側 Fable 5.1 = approve（P1 / P2 0、P3 3）、Codex 側 GPT-6 Astra（発注 134）= reject（P1 0 / P2 1 / P3 2）。両者とも MU1〜MU13 の red と正常な経路の成立を確かめた。Coordinator の裁定（すべて accept）:
  - Codex P2-F1（`scripts/check-workflow-git.sh:149` で `git diff` が失敗すると検査の対象が空になり、祖先に無い Plan Commit が exit 0 で通る。旧実装は exit 1）: Writer が、差分の取得に失敗したら全 packet の検査へ戻す（fail-closed）形に直し、回帰 test と mutation で固定する。
  - Codex P3-F2（承認依頼の例の回数が上限 6 に合わない）: Coordinator が直した（本 commit）。
  - Codex P3-F3（PK4 の test から、混在する fence の負例が落ち、その判定を壊す mutation が green になる）: Writer が pointer の形の負例を戻す。
  - Fable P3-1（`pr-gate.py` の集合 `archived` が `added` / `modified` の全 file を集め、名前が誤読を招く）: Writer が名前か絞り込みを直す（判定の結果は変えない）。
  - Fable P3-2（`PacketScope` が `CLI` の fixture を属性で借りる理由が書かれていない）: Writer が 1 行の comment を足す。
  - Fable P3-3（`docs/backlog.md:141` の Wave Registry の link の項目が旧前提の語のまま）: closeout で直す。
  - 是正の後の closure は Claude 側の fresh reviewer で行う（Final Review Minimum 2 は broad の 2 本で満たしている）。
- closure（2026-09-29、対象 `c6d6506d`、Claude 側 fresh Opus 5.5）: approve。前回の 5 件はすべて閉じた（F1 は修正を戻すと D4 (a3) が red）。新しい P3 1 件（N1）: `scripts/check-workflow-git.sh:149`・`:170` の `git diff --name-only` は既定の `core.quotePath` の下で、ASCII でない文字や `"` を含む path を quote して出すため、そういう名前の packet が PK5 の対象から黙って外れる（合成で再現）。今ある packet はすべて ASCII の名前で、helper も ASCII 以外の `--packet` を受け付けないので実害は小さいが、検査が黙って通る穴なので、Coordinator は Ready の前に直すと裁定した（Writer、回帰 test 1 件）。直した後の closure も Claude 側で行う。
- Findings Freeze: not yet frozen; post-freeze exceptions: none.

- Closeout（2026-09-29）: Plan Review は round 3（上限）まで回し plan-approved（遷移記録のとおり）。round 2 の Codex の P1（古い同名の archive があると active packet の単純な削除が通る、closeout の判定の抜け道）は、差分の中の移送の対応を必須にして塞いだ。closure 1 の N1 を `-z` の NUL 区切りで直した後の closure 2（`4a9b5c8b`、fresh Fable 5.1）= approve（新しい P1 / P2 0、P3 1）。その P3（`scripts/check-workflow-git.sh` の `wait "$!"` は、loop と `wait` の間に別の background や process substitution が無いことに依存する）は対応不要とし、将来そこを触るときの注意として backlog に置いた。専用 record は review pass（broad 2 + closure 2）、manual not-required。hosted CI 全ジョブ pass の後、owner の指示で helper 経由の squash merge（PR #123、`294ba366`、2026-09-29）。衝突を解いた版の manual の再利用は緩めず、本 lane を入れた後の実際の衝突を見て owner が改めて決める（backlog の観測の項目）。
