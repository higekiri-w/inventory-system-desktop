# Plan Packet: 手続きの軽量化（ハーネス刷新 PR4、R3）

2026-09-29 起草。出典は harness 監査（2026-09-24、agent 側の規則・仕組みの棚卸し。§3.I〜3.M・§3.S・§6・§7 の PR4 行・§8・§9 と 2026-09-27 の追記）、運用の教訓の棚卸し（2026-09-28）の「PR4 に載せる 9 項目」、受理 P1/P2 の回帰テスト固定の試行（owner 2026-09-27「発注書で試行 → PR4 で明文化」）、backlog のハーネス刷新 PR2・PR3 の follow-up。監査の行番号は監査時の HEAD `3148347b` のもので、本 packet の Scope は起点 `7ac96d9e`（origin/main）で各項目を確かめ直した行番号を使う。PR1（#97）・PR2（#113）・PR3（#117）・並走の摩擦（#123）で済んだ項目は「済み」で外した（下の「範囲の候補の確認」）。

owner の決定: 2026-09-24 ハーネス刷新は 5 本（PR0 ∥ PR1 → (PR2 ∥ PR3) → (PR4 ∥ PR5)）、Owner Effort Budget の数値上限は「残して緩める」（値は PR4 で提案）、規則は環境が変わるたびに変える。2026-09-27 受理 P1/P2 の回帰テスト固定は試行の後に PR4 で明文化。2026-09-28 規則とハーネスの改善を最優先（考え方の要旨「規則は失敗を測る方法としてはよいが、それで首が回らなくなるのは避けたい」。Coordinator の言い換え: 止める規則は起きている失敗を 1 文で言えること。言えないのに止める規則は緩めてよい。安全の境界〈古い証拠の拒否・独立 review・owner の L3・承認〉は残す）。2026-09-29 順番は「分かった情報がチャットにだけ残る状態の解消 ＞ ハーネス PR4・PR5 ＞ 作業」、PR4・PR5 の起票承認。relay の節約を目的にしない（owner の方針）。

本 lane は wave に属さない単独の lane（PR5「gate の穴」と並走し、file の所有を下の表で分ける）。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R3
- Plan Commit: ac00162da132ea4405079ccb160fb49052677781
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（fresh context、Coordinator が指定する worktree で作業）
- Plan Reviewer: fresh Opus 5.5 subagent（fork でない）+ Codex（GPT-6 Astra。merge gate の合否を変える変更）。互いに独立で Writer と別 context
- Final Reviewer: Fable 5.1（Claude 側、R3）+ Codex（GPT-6 Astra。難所: merge gate の合否を変える）。互いに独立で Writer・Plan Reviewer と別 context（Double Audit）。後の reviewer に先の結果を見せない
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/harness-pr4-lightweight

`Branch` は helper・checker が評価しない追加行。

manual なし: 製品の runtime・画面・配布物を変えず、Windows native L3 の対象画面が無い。checker の振舞いは合成 fixture の自動 test で確かめる（L3 Eligibility の (1) に当たらない）。r4 なし: data・DB・破壊的 git 操作を含まず、変更は revert で戻せる。Final Review Minimum 2: `scripts/doc-consistency-check.sh`・`scripts/tests/**` は `scripts/ci/classify-changes.sh:55` の実行制御、`docs/DEV_WORKFLOW.md`・`docs/templates/*`・`docs/code_review.md`・`docs/AGENT_OPERATING_MANUAL.md`・`AGENTS.md`・`.agents/*`・`docs/project-profile.md`・`.github/pull_request_template.md` は `:59` の policy docs で、どちらも workflow=true になり helper は Minimum 2 を要求する（`scripts/pr-gate.py:271-272`）。

遷移記録（append-only）:

- kickoff → spec-check → design → plan-draft → plan-gate（2026-09-29、起草役 = Opus 5.5 subagent、本 commit、plan-first）: Risk R3（下記 Risk）。spec-check で、本 lane の設計の正本は本 lane が書き換える workflow 文書そのもの（`docs/DEV_WORKFLOW.md`・`docs/templates/*`・`docs/code_review.md`）で、今の正本のままでは足りない（Owner Effort Budget・Wave・review の規則を書き換える）と確かめ、製品の設計正本（function / DB / screen）は触らないと確かめた（skip は使わない）。design の出力は本 packet の Spec Contract（D1〜D15）・「規則の処置表」・設計判断 1〜3 に置き（同じ plan-first の変更）、正本と decision-log D-098 への昇格は実装の S1〜S16 で行う（PR2・PR3・並走の摩擦と同じ形）。owner に諮る判断点 G1（分け方）・G2（Owner Effort Budget の値）は推奨案（A・A）で設計を閉じ、Plan Gate で owner に諮る。owner が G1 = B を選べば plan-draft へ戻って packet を 2 つに分け、G2 で A 以外を選べば S1・S6 と AC10 の値を plan-approved の前に直す。G3（所有表の外の 3 file）は Coordinator の裁定。Plan Review へ。
- plan-gate（round 1）: Plan Review round 1 の是正（相談役 Fable 5.1 の起草、2026-09-29。内訳は Review Response）
- plan-gate（round 2）: Plan Review round 2 の是正（相談役 Fable 5.1 の起草、2026-09-30。内訳は Review Response）
- plan-gate（round 3）: Plan Review round 3 の P3 の是正（2026-09-30。内訳は Review Response）
- plan-gate → plan-approved（2026-09-30、Coordinator、本 commit）: 上限の Plan Review round 3（対象 `87cf8eca`）で独立 Plan Reviewer 2 本の P1/P2 = 0、P3 の是正（`ac00162d`）を Coordinator が現物で確かめた（`doc-consistency-check.sh --target plan` と `check-workflow-git.sh` が exit 0、helper の `parse_packet` が通る）。owner 承認（2026-09-30、この change での介入 2〜4 回目 = plan-approved・G1・G2 を同じ 1 回の問い合わせで得た）: G1 = A（1 本）、G2 = A（介入の既定 6・実働 30 分、relay の上限を外す、上限に届くときは次の判断と同じ 1 回で諮る）。Plan Commit = `ac00162d`（承認した計画の最後の commit）
- plan-approved → implementing（2026-09-30、Coordinator、state-only）: Writer（Opus 5.5 subagent）へ実装を発注する。Writer の開始 HEAD は本 commit。

## Owner Effort Budget

今の規則（改訂前、`docs/DEV_WORKFLOW.md:260-270`）で書く。本 lane の後の規則は S6 と G2。

- 介入回数上限: 7（既定 3 から。理由: owner の判断点 G1・G2 を Plan Gate で 1 回ずつ得るため。decision point 単位の計上〈`docs/DEV_WORKFLOW.md:267`〉で 起票承認 1 + G1 1 + G2 1 + plan-approved 1 + Ready 1 + merge 1 = 6、予備 1）
- 実働時間上限: 15分（文書・script・test の変更で manual は無い。owner の作業は判断点の回答と Ready・merge の指示に限られる見込み）
- relay 往復上限: 5（Plan Review の Codex が最大 3 round、Final Review の Codex broad 1、base 同期の後の Codex closure 1）
- Plan Review round 天井: 3（既定 3）

| 種別 | 上限 | 消費（2026-09-30 の plan-approved 時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 7 | 4: 起票承認（2026-09-29、PR4・PR5 の起票）、G1・G2・plan-approved（2026-09-30、同じ 1 回の問い合わせ） | 2: Ready 1、merge 1 | 1 | 7 = 4 + 2 + 1 |
| relay | 5 | 3: Plan Review の Codex round 1・round 2・round 3（各 1。2026-09-29・2026-09-30・2026-09-30） | 2: Final Review の Codex broad 1、base 同期の後の Codex closure 1 | 0 | 5 = 3 + 2 + 0 |

G1 と G2 は同じ 1 回の問い合わせで答えを得ても、decision point ごとに 2 回と数える。

改定（2026-09-30、owner「Aだな」。train〈PR5 → PR4〉の Ready の委任と同じ 1 回）: relay 上限 5 → 7。消費は Plan Review 3、Final broad の Codex 1（発注 155）、closure 2（160 は発注書が packet を含む差分の全文を読ませたため数えず取り直し、162）、見込みは PR5 の merge の後の base 同期の closure 1。介入は 5（従来の 4 + broad の record の owner の `!` 1）+ Ready・merge の 2 = 7 で上限内。

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3「merge gate changes」に当たる。本 lane は docs job（hosted の `.github/workflows/ci.yml` の docs job、pre-push、local-ci）で走る checker の合否を変える: PK1 の R3 必須節（`scripts/doc-consistency-check.sh:1015`）、PK1 の R4 の review-only skip の ERROR（`:1060-1068`）、PK4 の `- Findings Freeze:` 行の要求（`:1298-1305`）の撤去と、PK4 の `Plan Commit` の書式検査・PK1 の Contract Ledger のデータ行の検査の追加。kickoff の問い「この変更でどれかの required gate の green / red が変わるか」の答えは「変わる」（緩める向き 3 つ、厳しくする向き 2 つ = PK4 の書式と PK1 の Ledger のデータ行）で、R3・Minimum 2。WARN だけの検査（PK3 の skip と Trace Matrix、WER の Retired 節）は exit code に影響しないが同じ script の変更として同乗する（監査 §6）。R4 には当たらない（data・破壊的操作なし）。R3 の review-only sub-agent は Final Review の Double Audit の broad が兼ね、skip の記録は PR body に置く（本 lane の後は S7 で語自体が Final Review に統合される）。

## 範囲の候補の確認（2026-09-29、起点 `7ac96d9e`）

Coordinator が挙げた候補を今の main で確かめた結果。「済み」は外し、根拠を書く。

| 候補 | 出典 | 今の main | 扱い |
|---|---|---|---|
| A1 template の統合（Contract Ledger・Design Readiness・R2 の短い本体）と PK1 / PK3 | 監査 J4・J5、§7 PR4 | template に 4 表（`docs/templates/plan-packet.md:137`・`:199`・`:236`・`:245`）と Design 系 4 節（`:97`・`:108`・`:145`・`:172`）。PK1 `:1015`、PK3 `:1123-1146` | S1・S2・S3 |
| A2 Owner Effort Budget の改訂、教訓 40・49 | 監査 I1・§9 Q1、owner 2026-09-24、教訓 40・49 | `docs/DEV_WORKFLOW.md:260-270`（介入 ≤3・実働 ≤30 分・relay ≤2・hard stop）、template `:27-35`、PK1 `:1014` が節を要求 | S1・S6、判断点 G2 |
| A3 Design Phase・Contract Audit の縮約、Impact Review Lenses の重複 | 監査 J9・K9・J7 | Design Phase `:116-209`（94 行）、Contract Audit `:351-365`（11 項目）、Lenses の表が DEV_WORKFLOW `:172-181` と template `:158-168` の 2 か所 | S5・S7 |
| A4 Review Rules の 3 分類の削除と、backlog の「普通の一日の finding が 3 分類のどれにも割り当てられていない」 | 監査 K2、backlog `docs/backlog.md:61` | `:335`。test が語の存在を要求（`scripts/tests/doc-consistency-plan-packet.test.sh:770-772`） | S7・S3 |
| A5 review-only と Final Review の統合（PK1 の R4 skip 検査を含む） | 監査 K4 | DEV_WORKFLOW 8 行、code_review 3 行、PK1 `:1060-1068`、PK3 `:1148-1154`、template `:269`、PR template 2 行ほか（AC7 の baseline） | S2・S4・S7・S9・S10・S11・S12・S13・S15 |
| A6 Findings Freeze の PK4 行の要求の撤去（原則は残す） | 監査 K6、PR1 の D4 から移送 | PK4 `:1298-1305`、template `:270`、test の section 6 | S2・S3・S7 |
| A7 WER の撤去 | 監査 M7 | `check_new_wer_retired_rules`（`:1364-1411`、呼出し `:1929`・`:1984`）、DEV_WORKFLOW `:37`・`:419`・`:421`・`:454`、`.agents/skills/workflow-effectiveness-review/`、`docs/templates/workflow-effectiveness-review.md` | S2・S3・S4・S9・S11・S12 |
| A8 Wave / stacked の縮約と D-055 の改訂 | 監査 L1・L2、owner 2026-09-24 | `:227-249`。D-097（#123）で Plans.md の lane 登録・closeout の待ち・`directoryRenames=false` は済み。残り = lane 数 2〜3 と同じ source document の同居禁止（`:234`）、生成物 1 lane の上限（`:234`・`:245`）、文の重複 | S8 |
| A9 Draft PR / closeout の縮約 | 監査 M1・M2・M3・M6 | `:367-428`。一過性の dogfood 記述 `:392-394`、WER の行 | S9 |
| A10 問い合わせの行き先の表を AGENTS への link に | 監査 I6、PR2 / PR3 の packet | 表 `:272-286` が残る（AGENTS 側は PR3 で済み）。MANUAL §5.6 の link `docs/AGENT_OPERATING_MANUAL.md:185` がこの見出しを指す | S6・S13 |
| A11 template の Consultation Relay 節と 2 欄、`independent-review` の残り | backlog `:63`、PR2 packet | template `:37-42`・`:130`・`:201` | S1 |
| 監査 K11 review-checklist の件数上限と新規観点禁止 | 監査 K11・§8-7 | 済み（PR #117）: `docs/quality/review-checklist.md:9`「件数で落とさない」、`:11` closure は Findings Freeze へ | 外す |
| 監査 K13 保守者可読性・K14 店の事実の照合 | 監査 K13・K14 | 済み（PR #117）: `docs/code_review.md:11`・`:25` | 外す |
| 監査 J11 旧前提 sweep の縮約・§5.6 の縮約 | 監査 J11 | §5.6 は PR2（#113）が縮めた形で残る（`docs/AGENT_OPERATING_MANUAL.md:151-189`）。MANUAL の全面的な縮約は backlog の別 PR | 外す（Non-scope） |
| 監査 M9 squash の subject | 監査 M9、backlog | helper `merge` の実装（PR5 の所有 file） | 外す（Non-scope） |
| B1 受理 P1/P2 の回帰テスト固定 | 教訓 110、試行 | 試行の block は発注書だけ。test-design Skill は「prefer adding a regression test」（`.agents/skills/test-design/SKILL.md` の Rules） | S10・S12（文言は下の「試行の集計」） |
| B2 L3 の到達経路・入力物・既知 backlog と fixture の同時手渡し | 教訓 103 | 無し（`docs/DEV_WORKFLOW.md:329` は encoding だけ） | S1・S9 |
| B3 component の移動・統合・置換の import / 描画の表 | 教訓 29 | 無し | S1 |
| B4 Matrix の oracle の 4 行 | 教訓 64・65・66・68 | 無し（`docs/templates/test-design-matrix.md` に oracle の語が 0 件、AC16 の baseline） | S11 |
| B5 是正で新しく書く事実主張の裏取り | 教訓 70 | `:68` は旧前提の sweep だけ | S4 |
| B6 reviewer の scope 増減の提案を owner の記録まで照合 | 教訓 62 | 無し | S7 |
| B7 PK4 の `Plan Commit` の書式検査 | 教訓 42 | PK4 は pending と Phase の組だけ（`:1286-1291`） | S2・S3 |
| B8 `inventory-operator-ui` Skill の読む物に `01-decision-rules.md` | 教訓 73 | 無し | S12 |
| B9 Owner Effort Budget の改訂に伴う 40・49 | 教訓 40・49 | A2 と同じ | S6、G2 |
| C(2) MANUAL の「design lane」の定義 | backlog `:64` の (2) | 語は座組表（`docs/AGENT_OPERATING_MANUAL.md:70`、PR5 の所有）にだけあり、定義が無い | S13（表の外に定義を置く） |
| C(3) 読ませない場所の例示に是正 commit | backlog `:64` の (3) | MANUAL `:36`、subagent-review-packet `:19` に無い | S11・S13 |
| C(4) code_review の Output Shape の確信度の欄 | backlog `:64` の (4) | 本文は確信度を求める（`docs/code_review.md:93`）が例に欄が無い | S10 |
| C(5) AGENTS の行き先 (ii) から MANUAL §5.6 への導線 | backlog `:64` の (5) | `AGENTS.md:47` に link が無い | S14 |
| C(1) closure の record | backlog `:64` の (1) | `scripts/pr-gate.py:436-439`（PR5 の所有） | 外す（Coordinator 裁定で PR5） |
| D 試行の文言 | owner 2026-09-27 | 下の「試行の集計」 | S10 |

## 試行の集計（受理 P1/P2 の回帰テスト固定、2026-09-27 以降）

試行の報告の観測表は空だった。試行の期間（2026-09-27 の決定から本起票まで）に Final Review を終えて archive へ移した lane の Review Response と、merge commit の test の差分から集計した。Writer の報告は tracked に無いので、「修正前に red」を Writer が確かめたかは、Review Response に記録がある所だけを書き、ほかは `未実測`。

| lane / PR | 受理 P1/P2 数 | 固定した数（test / L3 / docs） | 固定できず止まった数と理由 | closure の round 数 | 同じ finding の再発 | 所感（手間・誤検知） |
|---|---|---|---|---|---|---|
| 色と強調 #116（runtime） | 2（Codex P2-1 icon の欠落、P2-2 Ordinary Operation の文） | test 1（P2-1: `git show e188366e` の `DailyReportImportPage.test.tsx`・`PreviewStep.test.tsx` に svg の数の assert）、docs 1（P2-2: Coordinator が packet を直した） | 0 | 2（是正の後の closure 1、main 取込みの後の closure 2） | 0（closure 1 は P 0、closure 2 は新しい P1 / P2 0） | 修正前の red の確認は `未実測` |
| 小口のまとめ #118（runtime） | 0（P3 3 件だけ） | — | — | 1 | 0 | 試行の対象の finding が出なかった |
| 並走の摩擦 #123（workflow の script） | 1（Codex P2-F1 `git diff` の失敗で PK5 が空の対象になる） | test 1（`scripts/tests/workflow-git-checks.test.sh` の D4 (a3)、`git show 294ba366` で確認）+ mutation | 0 | 2（closure 1 の P3 N1 の是正の後の closure 2） | 0 | closure 1 が「修正を戻すと D4 (a3) が red」を確かめた（Review Response の記録）。runtime でない script でも同じ固定が効いた |
| Z001 表示の設計 #114（docs の design lane、参考） | 1（closure 1 の Codex P2: packet の model の根拠の欠け） | docs 1 | 0 | 2 | 0 | docs の finding は drift sweep の側で、試行の test の対象外 |

集計: test で観測できる振舞いの受理 P1/P2 は 2 件（runtime 1、workflow の script 1）で、2 件とも test で固定し、固定できずに止まった件は 0、再発は 0。件数が少なく、手間や誤検知の傾向は言えない（`未実測`）。文言の決定: 試行の案を採り、対象を「runtime の振舞い」から「自動 test で観測できる振舞い（製品の runtime と workflow の script）」へ広げる（#123 の実例）。止まる条件（test を足せないなら修正せず理由を報告）は 0 件だったが、既存 test を弱めて green にする抜け道を塞ぐ文として残す。closure 側は試行の block のとおり「修正を戻すと red」を確かめ、固定の欠けは既存 finding の未 closure として扱い、新しい finding にしない。見直しの契機: 固定できずに止まる件が続いたとき（D-098 の Revisit）。

## Goal

Goal Invariant:

### 最小完了条件

- R3 の Plan Packet を新しい template で書くと、契約の追跡は 1 表（Contract Ledger）、設計の準備は 1 節（Design Readiness）で済み、Consultation Relay 節・`- Findings Freeze:` 行・`Review-only skipped because:` 行を書かなくても docs job の checker（PK1〜PK4）が通る。旧 template で書いた active packet（本 packet と、並走する PR5 の packet を含む）も通り続ける。
- `Plan Commit` に `pending` でも 40 桁の小文字 hex でもない値（末尾の空白・タブ・注記を含む）を書いた active packet は、`bash scripts/doc-consistency-check.sh --target plan <packet>` が ERROR にする。起草役はこれを commit の前に回す。pre-push・local-ci・hosted の docs job でも同じ ERROR が出るが、そこでは PK5（`scripts/pre-push.sh:186-188` が doc check `:207` より先に走る）が commit 済みの初回値を固定した後になる（`scripts/check-workflow-git.sh:93-104`）。
- `docs/DEV_WORKFLOW.md` から、失敗を 1 文で言えない手続き（3 分類、review-only と Final Review の二重の語、WER の要求、wave の lane 数と同じ source document の上限、一過性の dogfood 記述、Design Phase と Contract Audit の重複した列挙、問い合わせの行き先の重複した表）が消え、残す規則は 1 か所にだけある。Owner Effort Budget は数値の上限を残したうえで G2 の値と扱いになる。
- 受理した P1/P2 の固定（修正前に red の test）、L3 の fixture の列挙、component の import / 描画の表、Matrix の oracle の規律、事実主張の裏取り、scope 増減の照合、design lane の定義、読ませない場所の是正 commit、確信度の欄、AGENTS から MANUAL §5.6 への導線が、使う場面の正本に 1 行ずつある。

### 失敗定義

- 旧 template の active packet（本 packet・PR5 の packet）が新しい checker で ERROR になる、または新しい template の R3 packet が ERROR になる。
- 安全の境界のどれかが弱まる: 独立 review（Writer ≠ reviewer、fresh context、Double Audit の本数）、Plan Commit の固定（PK5 の祖先・不変性）、owner の Human Gate、R4 の必須（`r4` と Final Review Minimum 2）、古い証拠の拒否（helper の head/base の照合）、不可逆 finding の 4 項目（K3）。
- 縮約で、失敗を 1 文で言える規則が移し先なしに消える（処置表の keep / merge の行が移し先に無い）。
- PR5 の所有 file に差分が出る、または PR5 と同じ行を取り合う。

### 非目的

- PR5 の範囲（classifier・helper・`local-ci.sh`・`pre-push.sh`・`.gitignore`・`.claude/**`・merge-evidence・`docs/ci.md`・座組表）。helper（`scripts/pr-gate.py`）の判定は変えない。
- MANUAL の全面的な縮約・節番号の振り直し・agent-guidance の統合（backlog の別 PR）。
- backlog「workflow の軽量化 3 段」の 1 段目の残り（WriterOrderV1・発注前検査の script）と 2 段目（Amendment の影響別扱い）。前者は監査 §3.S が delete と判定した Codex Writer 時代の対策で、後者は Gated Amendment の helper の扱い（broad `scripts/pr-gate.py:315`・closure `:443` の Amendments 一致）に及び PR5 の file に触れるため。
- 衝突を解いた版の manual の再利用の条件（owner 2026-09-28「今回は緩めない」）。
- 過去の decision-log の本文と archive の書き換え。過去の packet・設計書に残る旧い語（review-only・Contract Coverage Ledger 等）は遡って直さず、DEV_WORKFLOW に旧称の 1 文を置いて読めるようにする。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、本 lane の merge 後の R3 lane の一日（起票 → Plan Review → 実装 → Final Review → closeout）を通す操作列を書く。各行の「軽くなる」が本 lane で変わる所、「残る」が変えない所。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| main に本 lane が入っている。R3 lane の起票の承認がある | 起草役が新しい template で packet を書く | 軽くなる: 契約の追跡は Contract Ledger の 1 表（旧 4 表）、設計の準備は Design Readiness の 1 節（旧 4 節）、Consultation Relay 節と `- Findings Freeze:` 行は無い。R2 なら R3/R4 の部分を書かない。残る: Goal Invariant、Ordinary Operation、Contract Probe、Data Safety、Matrix、Owner Effort Budget（G2 の値） | `bash scripts/doc-consistency-check.sh --target plan` が exit 0（PK1 が Contract Ledger を R3 の節として受ける） | なし |
| 起草役が `Plan Commit` に確定待ちの注記や短い SHA を書いた | commit の前に `bash scripts/doc-consistency-check.sh --target plan <packet>`（作業木を読む） | 軽くなる（事故が減る）: PK4 が書式で ERROR にし、commit する前に直せる。commit 済みで未 push なら、新しい commit を足さずにその commit を直す（PK5 は commit 履歴の最古の non-pending 値を初回値に固定する〈`scripts/check-workflow-git.sh:93-104`〉ので、新しい commit で直すと書き換えの ERROR になる。旧: 書式検査が無く、解決できる短い SHA は PK5 も通し、helper が record の時点で拒んだ。教訓 42） | `pending` か 40 桁の小文字 hex に直して exit 0、それから commit | なし（Contract Probe 3・4・10・11） |
| packet が plan-gate | Plan Review（fresh Opus + Codex、互いに独立） | 軽くなる: finding を 3 分類に振り分けない（P1/P2/P3 の重大度だけ）。reviewer の scope の追加・除外の提案は owner の記録（catalog の除外注記・owner の原文・前 lane の Review Response）と照合してから採否を決める。残る: 冒頭の 3 値の答え、round 天井 3、相互修正案、不可逆 finding の 4 項目 | P1/P2 = 0 と plan-approved の owner 承認 | なし |
| Plan Gate で owner の判断が要る | Coordinator が承認依頼を出す | 軽くなる（G2 = A のとき）: relay の往復は数えない。介入の上限（既定 6）に届く見込みなら、次の判断と上限の改定を同じ 1 回で諮る（止まって別の承認を足さない）。残る: `この change での介入 N 回目 / 予算 M 回` と完了 1 文、decision point 単位の計上、goal-drift signal での即停止 | owner の回答 | G2 の owner の選択 |
| plan-approved | Writer が実装し Draft PR | 変化なし（plan-first、Plan Commit の固定、PK5） | 対象の検証が通る | なし |
| Draft PR | Final Review（`Final Review Minimum` の本数の broad、fresh・互いに独立） | 軽くなる: 「review-only sub-agent」と「Final Review」を別物として数えない。skip の行を書かない。残る: Contract Audit（8 項目に縮めた一覧、source docs から）、Double Audit、Findings Freeze の原則（packet に行は書かない） | helper `status` で review の要件 | なし |
| broad で P2 を受理した | Writer が是正する | 新しい規則: 自動 test で観測できる振舞いの P1/P2 は、修正前に red の test を先に書き、修正後に green を確かめ、red→green の command を報告する。観測できないものは L3 の項目、docs は drift sweep。closure は固定の test が差分にあり修正を戻すと red になることを確かめる | closure pass | 試行の件数が少ない（上の集計、`未実測` の所感） |
| manual の要る lane が Ready を頼む | Coordinator が L3 を頼む | 新しい規則: L3 の項目ごとの到達経路・入力物・依存する既知 backlog を packet に並べ、受理される fixture を Ready の依頼と同時に渡す。L3 の各 round を介入 1 回と数える | owner の PASS / FAIL | なし |
| merge 済み | closeout（wave ごとにまとめてよい、D-097） | 軽くなる: WER を書かない（旧: R3/R4・workflow の変更ごとに WER か次の dogfood 先の要求）。残る: archive への移送、Plans.md の同期、R0 の closeout PR | helper `--risk R0` | なし |
| 並走する別の lane の packet が旧 template のまま main か branch にある | その lane の doc check | 通り続ける（旧形式は受理するが要求しない）。`- Findings Freeze:` 行や `Review-only skipped because:` 行があっても評価しない | exit 0 | なし（Contract Probe 1） |

この列で、R3 lane の起票に書く表が 4 つから 1 つ、節が 4 つから 1 つに減り、review の分類・skip 行・Freeze 行・WER の手間が消える。独立 review の本数、Plan Commit の固定、Human Gate、helper の照合は変わらない。

## Scope

行番号は起点 `7ac96d9e`。各 file の変更は該当の行・節に限る。文の新しい中身は「規則の処置表」と Spec Contract に従い、Writer が文面を書く（逐語の文は AC の anchor の語だけを固定する）。

- **S1 `docs/templates/plan-packet.md` の組み直し**（D1・D2・D6・D13）
  - 構成: 3 つの部分をこの順に置く。(i) 全 Risk の本体 = 前文、Workflow State、Owner Effort Budget、Risk、Goal、Ordinary Operation、Scope、Non-scope、Acceptance Criteria、Design Readiness、Registration / Generation Obligations、Impact Review Lenses、Boundary / Wire Contract、Test Plan、Review Focus。(ii) 「R3/R4 の部分」= Contract Ledger、Contract Probe、Data Safety（頭に 1 行でその旨を書く。R2 は書かなくてよい）。(iii) 文書の末尾 = Implementation Results、Review Response（R3/R4 の部分より後に置く。独立 review は `## Review Response` より前だけを読む〈`docs/AGENT_OPERATING_MANUAL.md:36`〉ので、その前に (i)(ii) の全節が入る）。Impact Review Lenses は Risk に依らず今の template `:156` のきっかけの文で書き、不該当の lens は 1 行の理由（`docs/DEV_WORKFLOW.md:183`「For applicable R2+ work, record the lenses」を保つ）。Boundary / Wire Contract は Risk に依らず、`docs/DEV_WORKFLOW.md:62` の対象形式（JSON、browser state、CSV、config、manifest、cache schema、Tauri command DTO、generated bindings）に触る変更では必須のまま（条件文は変えない）。Registration は全 Risk で残す（列挙漏れの対策で、該当なしの 1 行で足りる）。checker・helper・test は節の位置を見ない（PK1 は見出し行の grep `scripts/doc-consistency-check.sh:1050-1054`、`extract_markdown_section` `:830-839` と helper の `workflow_fields` `scripts/pr-gate.py:147-153` は見出しから次の見出しまでを位置に依らず切り出す。`write_packet` は Review Response を最後に出す `scripts/tests/doc-consistency-plan-packet.test.sh:415`）ので、並びの変更で S2・S3 に足すものは無い。
  - Contract Ledger（新設、旧 Spec Contract `:236-243`・Trace Matrix `:245-250`・Design Intent Trace `:137-143`・Contract Coverage Ledger `:199-206` を置き換える）: 列は `契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象`。説明は 3 行以内: R3/R4 は必須、触る正本の節の契約・設計判断 ID をすべて行にする（行の欠けは Plan Gate の blocker）、行を書く前に隣接する契約の sweep、Final Review で各行を実装と突き合わせる。設計の理由と棄却案は正本・decision-log に置き、Ledger には ID で引く。
  - Design Readiness（旧 Design Sources `:97-106`・Required Design Artifacts `:108-119`・Design Intent Audit `:145-152`・Design Readiness `:172-189` を 1 節に）: 箇条は 6 つ = 引用する設計正本（節まで）、必要な設計成果物（DEV_WORKFLOW の Design artifact selection の当たる行だけ、状態 = existing sufficient / updated in this PR / deferred）、plan にしかない durable な判断の昇格先、前提・制約と延期した gap の follow-up、絶対保証の例外の自己点検、判定（ready / not ready と理由）。旧 `Minimum design checks` の 7 行は DEV_WORKFLOW の設計の完了条件への参照 1 行にする。
  - Owner Effort Budget（`:27-35`）: G2 の値と扱い（推奨 A: `介入回数上限`・`実働時間上限`・`Plan Review round 天井`、relay の行を消す）。消費の表（種別・上限・消費・残りの見込み・予備・合計）を置く。`介入 N`・`予算 M` の語は残す（test の section 17 が要求）。
  - 消す: Consultation Relay 節（`:37-42`）、Registration 表の相談役の行（`:130`）、Review Response の `Review-only skipped because:` の案内と `- Findings Freeze:` 行（`:269-270`）、`independent-review`（`:201`、Ledger の説明に置き換わる）。Registration の本文 `:123` と表 `:133` の「Contract Coverage Ledger」は「Contract Ledger」に直す（template 内に旧名の残りを作らない。AC5）。
  - Scope 節（`:77-81`）に 1 行（B3、教訓 29）: component の移動・統合・置換を含む Scope は、統合後の各 file が何を import し何を描画するかの表を置き、import の AC をそこから導く（anchor の語 `import し何を描画`）。
  - 同期先（所有表の外、Coordinator の裁定で本 PR に入れる。2026-09-30）: `docs/design-system/reference/README.md:32`「背骨を改定したとき: mockup の該当箇所を同時に直す（batch packet の Required Design Artifacts に含める）」の記入先を「Design Readiness の必要な設計成果物」に直す（撤去する節名への生きた指示を残さない。AC5）。
  - Test Plan 節（`:208-217`）に 1 行（B2、教訓 103）: Human Gate に manual（L3）を含むときは、L3 の項目ごとに到達経路・必要な入力物（DB のデータ・import する file・CSV・scan）・依存する既知 backlog を並べ、受理される fixture を Ready の依頼と同時に渡す（anchor の語 `受理される fixture`。規範の本文は S9 の Human Visual Confirmation）。
  - Impact Review Lenses（`:154-170`）: lens の表の唯一の所在にする（J7）。DEV_WORKFLOW `:172-181` の表が持つ `Question to answer`・`Evidence home` の 2 列を template の表へ移し（template の今の表 `:158-168` は Lens 名・Applicability / finding・Follow-up artifact だけで問いを持たない）、列は `Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact`。「不該当の lens は 1 行の理由」（`:183`）の説明をここへ移す（anchor: `Which claims are observed facts` が template に 1 件、AC14）。
  - Review Response 節（`:266-268`）に 1 行（round 2 の是正）: 先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く。独立 review は `## Review Response` より前だけを読む（anchor の語 `先行 round の結果`）。
- **S2 `scripts/doc-consistency-check.sh`**（D2・D3・D4）
  - PK1（`:1009-1074`）: R3 必須節（`:1015`）から `Spec Contract`・`Trace Matrix` を外し、`## Contract Ledger` があるか、`## Spec Contract` と `## Trace Matrix` の両方があるときに通す（どちらでもなければ ERROR `PK1: … Contract Ledger（旧 template は Spec Contract と Trace Matrix）を欠いています`）。`## Contract Ledger` があるときはその表のデータ行（`trace_matrix_data_rows` の読み〈`:962-980`〉を節名を引数にして使い回す。header と区切り行と空の行は数えない）が 0 件なら ERROR `PK1: … Contract Ledger にデータ行がありません`（PK2 は表の行を見ない〈`extract_prose` `:852-861`〉ので、空の Ledger を止める検査が他に無い。旧い組の経路には足さない）。archive の明示 path の扱い（`:1039-1048`）は変えない。R4 の review-only skip の ERROR（`:1060-1068`）を消す。`Owner Effort Budget`・`Data Safety`・`Contract Probe`・Test Design Matrix の参照の要求は変えない。
  - PK3（`:1112-1170`）: データ行の抽出（`trace_matrix_data_rows`、`:962-980`）を、`## Contract Ledger` があればその表から、無ければ `## Trace Matrix` から読むようにし、WARN の文言の節名もそれに合わせる。placeholder と `test_` token の検査は同じ。review-only skip の WARN（`:1148-1154`）を消す。
  - PK4（`:1217-1331`）: R3 の `- Findings Freeze:` 行の要求（`:1298-1305`）を消す。`Plan Commit` の値は `- Plan Commit:` の直後の空白を除いた行の残り全体で読み、末尾の空白・タブを削らない（`extract_workflow_field … full` は `:948` で末尾を削るので使わない。helper は `scripts/pr-gate.py:147-155` で末尾を保ち `:72-73` の `fullmatch` で拒む）。その値が `pending` か `[0-9a-f]{40}` に完全一致しなければ ERROR `PK4: … Plan Commit は pending か 40 桁の小文字 hex の SHA（末尾の注記・空白・タブなし）。commit 済みで未 push なら、新しい commit ではなくその commit を直す（PK5 は commit 済みの初回値を固定する）`（archive の明示 path は従来どおり PK4 の対象外）。既存の「plan-approved 以降で pending」の ERROR（`:1286-1291`）は残す。
  - WER: `check_new_wer_retired_rules`（`:1363-1411`）と呼出し（`:1929`・`:1984`）を消す。
- **S3 `scripts/tests/doc-consistency-plan-packet.test.sh`**（D2・D3・D4・D7）
  - fixture の既定の `PKT_PLAN_COMMIT`（`:274`、`abc1234`）と section 14 の上書き値（`:675`、`ffffff1`。plan-approved / implementing / archive の 3 phase）を 40 桁の合成 SHA にする（section 14 の 8 phase の正例はそのまま残す）。既定値を 40 桁にすると section 25（`:938-949`）が偶然守っていた「archive の明示 path では短い SHA が通る」の網が消える（section 13〈`:647-664`〉は `PKT_INCLUDE_WS=0`〈`:652`〉で Plan Commit 行を持たず、この網ではない）ので、PR4-F12 で archive の明示 path に `abc1234` の fixture を置き exit 0 を確かめる。`write_packet` に新 template の形（`## Contract Ledger` と `## Design Readiness`、Spec Contract・Trace Matrix・Findings Freeze 行なし）を出す切替えと、Contract Ledger と Trace Matrix のそれぞれのデータ行を 0 件（header と区切り行だけ）にする切替えを足す（PR4-F11・PR4-F2b）。
  - section 6（`:536`、Findings Freeze 行の欠落を ERROR）を「欠落でも ERROR なし」に置き換える（PR4-F3）。section 16（`:709-757`、WER の Retired 節）を「WER の検査が走らない」1 件に置き換える（PR4-F10）。section 17（`:759-`）の 3 分類の assert（`:770-772`）を消し、K3 の 4 項目・`goal-drift signal`・`one-shot irreversible`・`介入 N`・`予算 M`・承認依頼の counter の assert は残す。PR4-F5・F7 は exit 1 に加えて PK4 の書式の文言（`Plan Commit は pending か 40 桁`）を `assert_contains` で固定する（`pending（注記）` は Phase plan-gate で書く: implementing だと既存の「pending のままです」が先に出て MU5 を exit code で捕まえられない）。
  - 新しい section（見出しは `# --- PR4-F<番号>`）: 下の Test Plan の PR4-F1〜PR4-F12（F2b・F4b を含む）。
- **S4 `docs/DEV_WORKFLOW.md` の Artifact Map・Risk Tiers・Plan Packet Rules**（D5・D1・D13）
  - Artifact Map: `:35` の Review packets の行を「Plan Review・Final Review の発注に使う」に、`:37` の WER の行を消す。
  - Risk Tiers: `:49` の R3 の Required workflow を「Plan Packet、Test Matrix、targeted gates、Final Review（本数は Workflow State の `Final Review Minimum`）」に、`:50` の R4 に「Final Review Minimum 2 と Human Gate `r4`」を書く。
  - Plan Packet Rules: `:60` を「R3/R4 は Contract Ledger・Data Safety・Contract Probe・Test Design Matrix（旧 template の Spec Contract と Trace Matrix の組は Contract Ledger の代わりに受理）」、`:61` を Design Readiness の 1 節に、`:63-64` の Contract Probe を 2 行に縮め（probe は CI の設定・script・`git log` など具体物を調べ叙述だけで済ませない、の文は残す）詳細は template へ（J8）。`:68` から「its `Plans.md` entry」を消し（並走の摩擦 packet が PR4 へ回した残り）、1 文足す（B5、教訓 70）: 是正・強化で新しく書く契約文・引用・数は `rg` か実読で確かめ、確かめた所を併記する。併記できない主張は書かない（anchor の語 `確かめた所を併記`）。
- **S5 `docs/DEV_WORKFLOW.md` の Workflow State の停止の段落と Design Phase Rules**（D7・D8）
  - `:106` の停止の列挙から「Owner Effort Budget の hard stop」を「Owner Effort Budget の goal-drift signal」に直し、1 文足す: 停止を足すときは防ぐ失敗を 1 文で書く。書けない停止は緩める候補にする。安全の境界（古い証拠の拒否、独立 review、Plan Commit の固定、owner の Human Gate、R4 の承認、不可逆 finding の 4 項目、Workflow State の fail-closed〈`:109`〉、`AGENTS.md` の Decision and Approval Boundaries の承認境界）はこの原則で緩めない（owner 2026-09-28、D-098。anchor の語 `防ぐ失敗を 1 文で`）。Source Index（`:9-25`）に Purpose が `Durable decisions`、Source が `decision-log.md` への markdown link の行を ADR index の行（`:24`）の前に足す（Design inputs `:127` が挙げる decision-log を、処置表の Design inputs の行の参照先で失わないため）。
  - Design Phase Rules（`:116-209`）: 処置表の「Design Phase」の行どおりに縮める。見出し `## Design Phase Rules` は残す（`.agents/skills/inventory-implementation/SKILL.md` の anchor）。Review Rules `:346` の link `#design-decision-ids`（該当する見出しが元から無い）を `#design-phase-rules` に直す。
- **S6 `docs/DEV_WORKFLOW.md` の Owner Effort Budget と問い合わせの行き先**（D6・D11）
  - `## Owner Effort Budget`（`:260-270`）を G2 の値と扱いで書き直す（見出しは残す: `.agents/skills/inventory-workflow-start/SKILL.md:22` と `:239` の anchor）。推奨 A の中身: 既定の上限は介入 6 回・実働 30 分（anchor の語 `介入 6 回`）、packet が理由を書いて変えてよい。数え方は decision point 単位で、L3 の各 round も 1 回（anchor の語 `L3 の round`）、wave の batch 承認も lane ごとに数える（`:267` の縮約）。relay（Coordinator が Codex 等を起動する往復）は owner の手間でないので上限を置かない。上限に届く見込みのときは、追加の証跡・script・儀式を足す前に Goal Invariant の最小完了経路へ戻り、それでも owner の判断が要るなら、その判断と上限の改定を同じ 1 回で諮る。goal-drift signal は即停止のまま（`:269` の文を保つ）。Plan Review round 天井は Review Rules の 1 か所に置き、ここは参照だけ（`:264` の重複を消す）。one-shot irreversible の行（`:270`）は残し、「one-shot irreversible の task shape では MANUAL §3.5 の停止（`docs/AGENT_OPERATING_MANUAL.md:60`: time-box・Owner Effort Budget・goal-drift signal に達したら mutation 前に停止）が本節の「同じ 1 回で諮る」より優先する」を 1 文足す（anchor の語 `§3.5 の停止が優先`）。goal-drift signal の文（`:269`）の「classify candidate-safety work separately from supporting evidence」は 3 分類の語なので「compare the minimum completion route with supporting evidence」の趣旨に書き換える（`candidate-safety` を残さない。AC10）。
  - `### 問い合わせの行き先`（`:272-286`）を消し、`## Owner Effort Budget` の末尾に「問い合わせの行き先と委任の範囲は `AGENTS.md` の Decision and Approval Boundaries」の 1 行を置く（DEV_WORKFLOW から `../AGENTS.md` への markdown link）。
- **S7 `docs/DEV_WORKFLOW.md` の Review Rules と Contract Audit**（D5・D7・D8）
  - 処置表の「Review Rules」「Contract Audit」の行どおり。3 分類（`:335`）を消し、K3 の 4 項目（`:336`）は語を変えずに残す。review-only の 4 行（`:338`・`:340`・`:341` と `:337` の一部）を 1 行にする: R2+ は `Final Review Minimum` の本数の Final Review（broad audit）を行い、skip しない。R4 は Minimum 2 と `r4`（PK4・helper が強制）。旧称 review-only sub-agent はこの Final Review を指し、旧 packet の `Review-only skipped because:` 行は評価しない（anchor: `旧称 review-only sub-agent`。DEV_WORKFLOW でこの行だけが `review-only` を含む）。Contract Audit の前文 `:353` の「It extends the review-only sub-agent packet」は「It extends the review packet（`templates/subagent-review-packet.md`）」に直す（`:353` は旧い AC7 の式の唯一の一致で、AC7 の baseline 誤りの元）。Findings Freeze（`:348`）は ①〜④ の原則を 3 行に縮め、packet への記録の要求を書かない。1 行足す（B6、教訓 62）: reviewer の scope の追加・除外の提案は、catalog の除外注記・owner の原文・前 lane の Review Response と照合してから採否を決め、照合先を 1 行残す（anchor の語 `owner の記録と照合`）。
  - Contract Audit（`:351-365`）を 8 項目にする（K9）: Contract Ledger の再検証（隣接 sweep `:362` を統合）、Double audit、State Lifecycle Matrix、Adjacent Pattern Audit、mutation（`:359`・`:360` を統合）、Negative-space audit、Drift-fix sweep（Draft PR の旧文言 grep `:374` を統合）、Manual verification boundary。PR body freshness（`:365`）は S9 で Draft PR Checkpoint へ移す。見出し `## Contract Audit (R3/R4)` は残す（subagent-review-packet の anchor）。
- **S8 `docs/DEV_WORKFLOW.md` の Wave Operation と Stacked train**（D9）: 処置表の「Wave」の行どおりに縮める。lane 数 2〜3 と同じ source document の同居禁止（`:234`）を消し、「起票時に予定 file（生成物を含む）を突合し、重なる file は両 packet に行・節の所有表を書く（末尾への追記は merge 順で両方残す）。生成物が衝突したら手で解かず、取り込んだ後に再生成する」に置き換える（owner 2026-09-24 の不適用の決定の正本化、D-098 が D-055 の該当部分を置き換える）。train の委任の文（`:239`、`AGENTS.md:46` が参照）は意味を変えずに残す。見出し `## Wave Operation`・`### Stacked train` は残す。
- **S9 `docs/DEV_WORKFLOW.md` の Flow の順序・Human Visual Confirmation・Draft PR Checkpoint・Post-Merge Closeout・Done Definition**（D5・D10・D13）
  - Human Visual Confirmation: `:326` の「review-only sub-agent approval」を「Final Review」に。`:329` の fixture の encoding の行に B2 の本文を足す（L3 の項目ごとの到達経路・入力物・既知 backlog の列挙と、受理される fixture の Ready 依頼との同時手渡し）。
  - Draft PR Checkpoint（`:367-394`）: 処置表の行どおり。`:376` の review-only の条件を消し（Final Review は Draft で行う）、`:374` を Contract Audit の Drift-fix sweep へ、PR body freshness を受け入れ、Workflow-change dogfood（`:392-394`）を消す。`Human Gate` の欄と `この change での介入 N 回目 / 予算 M 回` の文（test の section 17 が要求）は残す。
  - Post-Merge Closeout（`:396-428`）: `:419` の WER の文と `:421` を消す。helper の手順と重なる Before merge（`:402-408`）を Workflow State `:105` への参照と、この節にしか無い 3 行（残存リスクの記録、manual の結果は agent が記録、古い HEAD の green を再利用しない〈安全: helper の `ci()` `scripts/pr-gate.py:384-391` が PR HEAD と同じ `head_sha` の CI 成功を要求する。文も残す〉）に縮める。
  - Done Definition: `:454` を消す。
  - 順序（C4）: `:7` の Flow `5. Verify -> 6. Review -> 6.5 Draft PR`、`:381`（Draft PR を Verify と Review の後に開く文）、`:449`（Done Definition の同じ順序の文）は helper の手順（`:105`: Draft で対象検証と capture、Final Review Minimum 以上の broad audit）と逆。Verify → Draft PR → capture → Final Review の順に揃える（Flow は `5. Verify -> 6. Draft PR -> 7. Final Review -> …` の形、`:381`・`:449` は「after Verify」と、Draft で Final Review を受ける旨）。Verify と Review を `+` でつないだ旧い語を残さない（AC13）。
- **S10 `docs/code_review.md`**（D5・D12・D13）
  - `:9` の「Design Sources / Design Readiness」を「Design Readiness」に、`:50` の「Design Intent Trace」を「Contract Ledger（旧 packet は Design Intent Trace と Trace Matrix）」に。
  - `## Verification Rules`: 2 行足す。file の不在・重複・drift を主張する前に file type（symlink）を `git ls-files -s` か `eza -l` で確かめる（DEV_WORKFLOW `:347` を移す、K8）。closure の固定の確認: Closure of an accepted P1/P2 confirms its pin: the pinning test (or L3 item / sweep) is in the diff and reverting the fix turns it red. A missing pin keeps that finding open; it is not a new finding（anchor の語 `reverting the fix turns it red`）。
  - `## Same PR vs Follow-up` の Fix in the same PR に 1 行（D12）: Pin each accepted P1/P2 finding whose fix changes behavior an automated test can observe (product runtime or workflow scripts) with a test that fails on the pre-fix code and passes after the fix, and report the red→green commands. When no automated test can observe it (visual, Windows native, hardware), add an L3 checklist item instead; docs-only findings go through the drift-fix sweep. Do not weaken an existing test to get green; when no test can be added, stop and report why instead of fixing（anchor の語 `fails on the pre-fix code`）。
  - `## Review-only Sub-agent Protocol`（`:72-78`）を `## Final Review Protocol` にし、skip の 2 行を消す。R3/R4 の review 発注に subagent-review-packet を使う・read-only で findings だけ・finding を実装者が確かめる、の 3 行は残す。
  - `## Output Shape` の例（`:95-106`）の finding の行に確信度の欄を足す（C4、例 `- P2（確信度: 高）- path:line - …`、anchor の語 `確信度: `）。
- **S11 templates**（D4・D5・D13）
  - `docs/templates/subagent-review-packet.md`: 題を「Review Packet（Plan Review・Final Review）」に、`:14` を「Contract Ledger（旧 packet は Contract Coverage Ledger）/ Test Design Matrix」に、`:19` の読ませない場所の例に「是正 commit の件名・本文」を足す（C3）。
  - `docs/templates/test-design-matrix.md`: Test Matrix の注記（`:17-19`）に oracle の規律を 3 行（B4、教訓 64・65・66・68）: oracle の anchor は定義文にしか現れない literal を選び `rg -c` で対象 file 内 1 件を確かめ、file ごとに分ける（複数 file を 1 本の `rg` で数えない）。不変の guard にも感度の実測行を置く。文言を改訂した後は「新しい文言が exact で 1 件 + 旧い文言が 0 件」の対にする。oracle は検証対象と独立の正本から転記し、mutation は production 側だけを変える。結果を空にする注入で、期待が空集合の case だけが kill を主張していないかを確かめ、各組合せに非空の期待を 1 件置く。`:101` の旧い Workflow State の問い（tracked に PR HEAD を置く案）を消す。
  - `docs/templates/workflow-effectiveness-review.md` を削除する（link は DEV_WORKFLOW `:37` の 1 件だけ、Contract Probe 7）。
- **S12 `.agents/skills/**`**（D4・D5・D12・D13）
  - `.agents/skills/workflow-effectiveness-review/` を削除する（`.claude/skills` に symlink は無い、`ls -la .claude/skills` で確認）。
  - `test-design/SKILL.md`: Required Reading の Design Sources・Design Readiness・Design Intent Trace の 3 行を「Plan Packet の Design Readiness と Contract Ledger（旧 packet は Design Sources・Design Intent Trace）」に、「For accepted P1/P2 fixes, prefer adding a regression test.」を code_review の固定の規則への参照に。
  - `inventory-code-review/SKILL.md`: `:54` の Contract Coverage Ledger を Contract Ledger（旧名を併記）に、`:62` の「Review-only results」を「Final Review results」に。
  - `inventory-implementation/SKILL.md:22`「独立レビューを完了する。… Draft PR Checkpoint で…引き渡す」を Verify → Draft → capture → Final Review の順に直す（C4。`独立レビューを完了する` を残さない）。
  - `inventory-operator-ui/SKILL.md` の Rules に 1 行（B8、教訓 73）: 見やすさ・見た目を目的にする change は、`docs/design-system/01-decision-rules.md` の該当 DSR を先に確かめ、無ければ DSR を先に起こす。
- **S13 `docs/AGENT_OPERATING_MANUAL.md`（`## 座組` の表を除く）**（D5・D6・D11・D13）
  - §2 の Coordinator の行（`:17`）の「超過見込みまたは goal-drift signal で hard stop する」を DEV_WORKFLOW `Owner Effort Budget` に従う形に。
  - §3 の独立性の項（`:36`）の読ませない場所の例に「是正 commit の件名・本文」を足す（C3）。§3 の末尾（`:41` の後、座組表の外）に design lane の定義を 1 行（C2）: design lane は、製品の設計正本（`docs/function-design/`・`docs/db-design/`・`docs/SCREEN_DESIGN.md`・`docs/design-system/`・ADR）の変更を主目的とし製品コードを変えない lane を指す（anchor の語 `design lane は`）。
  - §4 の router（`:87`・`:89`）を「Design Phase → DEV_WORKFLOW Design Phase Rules と template の Impact Review Lenses」「Contract Audit / Final Review」に。§5.6 の `:185` の link を `../AGENTS.md` の Decision and Approval Boundaries に。§6 の `:193` の「review-only packet」を「review packet」に。
- **S14 `AGENTS.md`（follow-up (5) の導線だけ）**（D11）: Decision and Approval Boundaries の agent 側で解く問い合わせの項（`:47`）の「発注だけの誤り」に、MANUAL §5.6「Writer が編集前に止まったとき」への markdown link（path は `docs/AGENT_OPERATING_MANUAL.md`、anchor は `#56-従来型-writer-発注書の共通出力契約`。DEV_WORKFLOW `:68` と同じ anchor）を足す。他の行は変えない。
- **S15 所有表の外の 3 file（判断点 G3、Coordinator の裁定待ち）**（D5）: review-only の語の統合に必要。PR5 の所有表にも無い。
  - `.github/pull_request_template.md`: `## Review-only`（`:35-38`）を消す（`## Validation` の「必要review/manual/R4と残るHuman Gate」が同じ欄を持つ）。PR2 packet が PR4 に回した行。
  - `docs/project-profile.md:160`: 「R3/R4 use the Final Review (count per `Final Review Minimum`); R4 also requires human approval (`r4`).」に。PR3 packet が PR4 に回した行（当時 `:231`）。
  - `docs/DOC_STYLE_GUIDE.md:234`: PK3 の説明を「R3/R4 plan の Contract Ledger（旧 Trace Matrix）と Acceptance evidence を warning で見える化する」に。
- **S16 `docs/decision-log.md` の末尾に `## D-098`**（番号は本 packet で予約する。D-097 は並走の摩擦で使用済み。PR5 は D-099 以降を使う前提で、Coordinator が PR5 の起草役に伝える）: 手続きの軽量化。(1) 止める規則は防ぐ失敗を 1 文で言えること、言えない停止は緩める候補、安全の境界は緩めない（owner 2026-09-28）。(2) Contract Ledger と Design Readiness への統合（D-034 の template の該当部分を置き換える。旧形式は受理）。(3) review-only を Final Review に統合し PK1 の R4 skip 検査を撤去（R4 の review は PK4 と helper の Minimum 2 が強制）。(4) Findings Freeze の記録行の要求を撤去し原則は残す（D-038(2) の記録の部分を置き換える）。(5) 3 分類の撤去（D-046-2 の分類の部分を置き換える。分類が防ぐ失敗〈証跡の欠けが候補の安全と競合する、証跡だけで破壊的な修正をする〉は Goal Invariant の優先順位と K3 の 4 項目が守り、4 項目は残す）。(6) WER と Retired 節の検査の撤去（D-046 の該当部分。規則は環境が変わったとき・誤作動したときにその場で直す）。(7) Owner Effort Budget の G2 の値と扱い（D-038(6)・D-046-1 の値と hard stop の部分を置き換える）。(8) wave の lane 数と同じ source document の同居禁止の撤去（D-055 の該当部分。owner 2026-09-24）。(9) 受理 P1/P2 の回帰テスト固定（owner 2026-09-27 の試行の明文化、集計は本 packet）。(10) PK4 の `Plan Commit` の書式検査（値全体を末尾を削らずに `pending` か 40 桁の小文字 hex と照合。検出点は commit の前の `--target plan`。PK5 は commit 済みの初回値を固定するので、commit 済み未 push ならその commit を直す）。Revisit: 固定できずに止まる件が続いたとき、Owner Effort Budget の上限の改定が lane ごとに続くとき。
- review-checklist（`docs/quality/review-checklist.md`）: 変更なし。`:11` の Findings Freeze への参照は原則が残るので有効（確認のみ）。
- `scripts/tests/run-workflow-tests.sh`: 変更なし（新しい test file は無い）。

### 規則の処置表（DEV_WORKFLOW の縮約、失敗を 1 文で言えるかで判定）

keep = 移し先の節に残す、merge = 他の行・文書に統合、delete = 消す。安全の境界に当たる行は「安全」と書き、keep にする。

| 節 | 現行（起点の行） | 処置 | 理由（防ぐ失敗、または消す理由） |
|---|---|---|---|
| Design Phase | 適用条件 `:118` | keep | 正本が変わる R2+ の変更で設計を飛ばす |
| Design Phase | Design inputs の一覧 `:120-127` | merge → Source Index（`:9-25`）への参照 1 行 | 同じ正本の一覧が Source Index にある（要件・spec map `:18`、architecture `:19`、function `:20`、DB `:21`、screen `:22`、ADR `:24`）。decision-log（`:127`）だけ Source Index に無いので S5 で `Durable decisions` の行を足す。artifact selection の表 `:131-139` は要件・spec と ARCHITECTURE を挙げないので、表への merge では消える |
| Design Phase | artifact selection の表 `:129-139` | keep（最後の行の Design Sources を Design Readiness に） | 変更の種類ごとに更新すべき正本を取り違える |
| Design Phase | Design outputs `:141-149` | delete | 表と Design Readiness の重複。Design Intent Trace の行は Contract Ledger へ |
| Design Phase | Design decision IDs `:151-156` | keep（2 行） | 設計判断の理由が plan にだけ残り追えなくなる |
| Design Phase | intent audit `:158-166`・checklist `:189-201`・completion `:203-209` | merge → 「設計の完了条件」11 行以内 | 3 つの列挙が同じことを言う（J9）。中身は 1 行ずつ残す: 正本だけで何を・なぜ・何を棄却したか分かる、spec ID ごとの設計成果物、plan だけの判断の昇格、前提と延期 gap、完成形を先に書く、Matrix が正本から導ける、高い risk tier での一時評価と隣接 spec の比較、層・関数・command / data・永続化・operator の流れ・選択肢の出所、絶対保証の例外（WER 由来の教訓、安全）、引用 test の実在（WER 由来の教訓）、欠けた設計は実装前に正本を直すか design-only に |
| Design Phase | Impact Review Lenses の表 `:168-183` | merge → template（表は template だけ。`Question to answer`・`Evidence home` の 2 列を template の表へ移す。DEV_WORKFLOW は使う場面の 2 行と link） | 2 か所の表がずれる（J7）。問いの列を移さないと 8 lens の問いが消える |
| Design Phase | Backfill note `:185-187` | keep（1 行） | 触らない領域の設計まで遡って書かせる |
| Plan Packet Rules | Contract Probe `:63-64` | keep（2 行に縮める。probe は CI の設定・script・`git log` など具体物を調べ、叙述だけで済ませない、の文は残す。仮適用の詳細は template） | 未検証の外部前提を叙述だけで確かめたことにする（D-038(4)、UI-13 Amendment 1） |
| Review Rules | `:333`・`:334`・`:342` | keep | 正本より packet を信じる、finding を検証せずに採る |
| Review Rules | 3 分類 `:335` | delete | 分類が防ぐ失敗（証跡の欠けが候補の安全と競合する、証跡だけで破壊的な修正をする。D-046 の Why）は `:57`（`Goal Invariant > Acceptance Criteria > supporting evidence`）と `:336`（K3 の 4 項目）が守る。分類は Ordinary Operation の finding の行き先を持たず（backlog `:61`）、重大度 P1〜P3 で足りる |
| Review Rules | 不可逆 finding の 4 項目 `:336` | keep（安全） | 証跡の欠けだけを根拠に破壊的な修正をする |
| Review Rules | Impact Lenses を review へ渡す `:337` | merge → subagent-review-packet `:25`（既にある） | 重複 |
| Review Rules | review-only の行 `:338`・`:340`・`:341` | merge → Final Review の 1 行（安全: R4 の必須は PK4・helper） | Final Review と別物として数える二重の語（K4） |
| Review Rules | Writer が Codex の packet の Plan Reviewer `:339` | keep | 縛られる側に gate 文を書かせる（D-062） |
| Review Rules | Same-PR fixes `:343` | merge → code_review の Same PR vs Follow-up（既にある） | 重複 |
| Review Rules | 相互修正案 `:344`、round 天井 `:345` | keep（1 行ずつ。round 天井はここが唯一の所在） | rally が収束しない |
| Review Rules | 連番 registry `:346` | keep（link を直す） | 並走 lane の番号の二重割当 |
| Review Rules | symlink の確認 `:347` | merge → code_review Verification Rules | 一般的な証拠の規律（K8） |
| Review Rules | Findings Freeze `:348` | keep（原則を 3 行に） | 小出しの発見で review が収束しない（PR #164 の 9 round） |
| Review Rules | Plan Review の発注と 3 値 `:349` | keep（3 行に） | 操作列の成立を問わずに plan を通す（D-090） |
| Contract Audit | 11 項目 `:355-365` | 8 項目に（S7） | 列挙の重複（K9）。中身は消さない。PR body freshness は Draft PR へ |
| Wave | lane の定義 `:233` | keep（1 行） | 複数の是正を 1 packet に束ねて review が追えない |
| Wave | lane 数 2〜3・同じ source document・生成物 1 lane `:234` | delete → 予定 file の突合と所有表、生成物は取込み後に再生成（S8） | owner 2026-09-24 で不適用。wave 13・14 は所有表と merge 順で解いた（PR2・PR3・並走の摩擦の packet） |
| Wave | lane 一覧 `:235`、reviewer の独立 `:236`、Ready は先頭だけ `:237`、単段 merge `:238`、train 委任 `:239` | keep（1 行ずつ） | strict のもとで Ready を並べると CI を無駄にする、独立 review が崩れる、他 lane の packet を archive へ動かす（D-097）。委任は AGENTS が参照 |
| Stacked | `:243-249` | keep（3 行に） | 多段 merge で Plan Commit の祖先が崩れる（PR #86、D-074）、実装 file の衝突の解き方を再 review しない |
| Owner Effort Budget | `:262-270` | 改訂（S6、G2） | 下の G2 |
| 問い合わせの行き先 | `:272-286` | merge → AGENTS（S6・S14） | 2 か所の同じ境界（I6）。AGENTS が上位（`:285` の注記 (b)） |
| Draft PR | first pass の条件 `:371-377` | keep（review-only の行を消し、旧文言 grep は Drift-fix sweep へ） | Draft を開く前に Final Review を要求する矛盾 |
| Draft PR | Workflow-change dogfood `:392-394` | delete | 一過性の記述（M3） |
| Post-Merge | WER `:419`（後半）・`:421`、Done `:454` | delete | 最後の実施が 2026-08-12 で形だけ（M7） |
| Post-Merge | Before merge `:402-408` | merge → Workflow State `:105` への参照 + keep 3 行（残存リスクの記録 `:407`、manual の結果は agent が記録 `:408`、古い HEAD の green を再利用しない `:405`） | helper の手順と重複。`:405` は古い証拠の拒否そのもので安全（helper の `ci()` が PR HEAD と同じ `head_sha` の CI 成功を要求する `scripts/pr-gate.py:384-391`。`:403`・`:491` から呼ぶ）。文も残す |

## Non-scope

- 上の所有表で PR5 のものとした file と行すべて（`scripts/ci/classify-changes.sh`、`scripts/pr-gate.py`、`scripts/local-ci.sh`、`scripts/pre-push.sh` とそれぞれの test、`.gitignore`、`.claude/agents/**`、`.claude/settings.json`、`docs/agent-guidance/merge-evidence.md`、`docs/ci.md`、`docs/AGENT_OPERATING_MANUAL.md` の `## 座組` の表）。backlog `:64` の (1) closure の record も PR5。
- `docs/Plans.md`（D-097: `## 次の行動` の `docs/plans/` を指す pointer 行で足りる）、`docs/backlog.md`、`docs/archive/**`: 実装 PR では触らない。closeout で backlog の「普通の一日の finding の 3 分類への割当て」「`independent-review` の残り」「PR2・PR3 の follow-up (2)〜(5)」を閉じ、「workflow の軽量化 3 段」に本 PR の分を記録する。
- 過去の packet・設計書に残る旧い語（例: `docs/function-design/65-inventory-record-traceability.md:294` の review-only sub-agent、`docs/decision-log.md` の D-090 が指す `DEV_WORKFLOW.md#問い合わせの行き先`）: 書き換えない。DEV_WORKFLOW の旧称の 1 文で読め、link 検査は anchor を見ない（Contract Probe 6）。
- `.claude/commands/check.md:5` と `AGENTS.md:15`（「Codex の review-only 発注」）の「review-only」: 読み取り専用の review という一般の意味で、Final Review の旧称ではない（`.claude/**` は所有表の外、AGENTS は S14 の導線だけが本 PR の所有）。
- `scripts/tests/workflow-git-checks.test.sh:357` と `doc-consistency-plan-packet.test.sh:496` の `independent-review`: doc-consistency 側（section 3）は撤去済みの Phase を拒む負例の入力、workflow-git 側（T-G5）は旧 state-only の commit 件名を積んでも通ることを確かめる正例の入力で、どちらも生きた指示ではない。
- DEV_WORKFLOW の Implementation Rules・Verification Gates・Subagent Budget・Commit / PR Messages・Evidence Ownership（PR1・PR2 が整えた。監査の M5〈`cargo check --release` の 2 か所〉は template の 1 か所が既に正で、重複の実害が無い）。
- Amendments の書式検査（`none` か 40 桁の SHA の列）: 起きた失敗が無い（教訓は Plan Commit だけ）。helper は既に `sha()` で拒む（`scripts/pr-gate.py:176-178`）。
- 分け方で G1 = B を選んだ場合の非目的: 下の G1。

### PR5 との file の所有（PR5 の packet と同じ表）

| file / 範囲 | 所有 |
|---|---|
| `docs/DEV_WORKFLOW.md`、`docs/templates/**`、`docs/code_review.md`、`docs/quality/review-checklist.md`、`scripts/doc-consistency-check.sh`、`scripts/check-workflow-git.sh`、`scripts/tests/doc-consistency-plan-packet.test.sh`・`workflow-git-checks.test.sh`・`reading-order-drift.test.sh`、`.agents/skills/**`、`AGENTS.md`（follow-up (5) の導線だけ）、`docs/AGENT_OPERATING_MANUAL.md`（`## 座組` の表を除く） | PR4 |
| `scripts/ci/classify-changes.sh`、`scripts/pr-gate.py`、`scripts/local-ci.sh`、`scripts/pre-push.sh` とそれぞれの test、`.gitignore`、`.claude/agents/**`、`.claude/settings.json`、`docs/agent-guidance/merge-evidence.md`、`docs/ci.md`、`docs/AGENT_OPERATING_MANUAL.md` の `## 座組` の表 | PR5 |
| `docs/decision-log.md` の末尾追記、`scripts/tests/run-workflow-tests.sh` への登録 | 両方（merge 順で両方を残す） |
| `docs/Plans.md`、`docs/backlog.md`、`docs/archive/**` | どちらも実装 PR で触らない（closeout） |

本 packet が表の外に求める file（判断点 G3、Coordinator の裁定）: `.github/pull_request_template.md`、`docs/project-profile.md`、`docs/DOC_STYLE_GUIDE.md`（S15）と、`docs/design-system/reference/README.md`（S1 の同期先、Coordinator の裁定 2026-09-30）。どれも PR5 の表に無い。本 PR は `scripts/check-workflow-git.sh`・`workflow-git-checks.test.sh`・`reading-order-drift.test.sh`・`run-workflow-tests.sh` を所有するが変更しない。`docs/decision-log.md` は末尾に D-098 を足すだけで、PR5 が先に merge して D-099 を足していれば番号順に D-098 → D-099 の並びで両方を残す（PR5 が D-099 を使う前提は Coordinator が PR5 と合わせる）。

## Acceptance Criteria

baseline は起点 `7ac96d9e` の本 worktree（本 packet と Matrix を置く前の状態。AC3 だけは本 packet を置いた後）で、同じ command を逐語で 2026-09-29 に起草役が実行した出力。AC の数は機能の代理にせず、test は全 PASS を求める（本数は runner の出力を参照）。G2 で owner が推奨 A 以外を選んだ場合、AC10 の該当行を plan-approved の前に直す。

- AC1（checker の fixture test 全体）: `bash scripts/tests/doc-consistency-plan-packet.test.sh` が exit 0。baseline: exit 0（section 6・16・17 は旧い契約）。
- AC2（新しい fixture）: `rg -c '^# --- PR4-F' scripts/tests/doc-consistency-plan-packet.test.sh` が 13 以上で（PR4-F1〜PR4-F12 と PR4-F2b。F4b は F4 の section 内でよい）、AC1 の中で各 section が期待どおり（Test Plan の PR4-F1〜PR4-F12・F2b・F4b）。baseline: 一致なし（exit 1）。
- AC3（本 packet が新旧どちらの checker でも通る、`--target plan`）
  - `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-09-29-harness-pr4-lightweight.md` が exit 0。baseline: exit 0、末尾 `結果: 全チェック通過`（本 commit の packet と Matrix を置いた状態で起草役が実行）。
  - `git show origin/main:scripts/doc-consistency-check.sh > "$TMPDIR/dcc-main.sh" && bash "$TMPDIR/dcc-main.sh" --target plan docs/plans/2026-09-29-harness-pr4-lightweight.md` が exit 0（起点の checker。実装の後も本 packet が旧い規則を満たすことの確認）。baseline: exit 0、末尾 `結果: 全チェック通過`（同上）。
- AC4（checker の撤去と追加、`scripts/doc-consistency-check.sh`）
  - `rg -c 'check_new_wer_retired_rules' scripts/doc-consistency-check.sh` が一致なし（exit 1）。baseline: `3`。
  - `rg -c 'Findings Freeze' scripts/doc-consistency-check.sh` が一致なし（exit 1）。baseline: `2`。
  - `rg -c 'Review-only skipped because' scripts/doc-consistency-check.sh` が一致なし（exit 1）。baseline: `2`。
  - `rg -c 'Contract Ledger' scripts/doc-consistency-check.sh` が 1 以上。baseline: 一致なし（exit 1）。
- AC5（template の構成、`docs/templates/plan-packet.md`）
  - `rg -n '^## (Spec Contract|Trace Matrix|Design Intent Trace|Contract Coverage Ledger|Design Sources|Required Design Artifacts|Design Intent Audit|Consultation Relay)$' docs/templates/plan-packet.md` が一致なし（exit 1）。baseline: 8 行（`:37`・`:97`・`:108`・`:137`・`:145`・`:199`・`:236`・`:245`）。
  - `rg -n '^## (Contract Ledger|Design Readiness)$' docs/templates/plan-packet.md` が 2 行。baseline: 1 行（`:172` Design Readiness）。
  - `rg -n 'Review Order Artifact|Review Order Ref|independent-review|Findings Freeze:|Review-only skipped|relay 往復上限' docs/templates/plan-packet.md` が一致なし（exit 1、`relay 往復上限` は G2 = A のとき）。baseline: 6 行（`:31`・`:41`・`:42`・`:201`・`:269`・`:270`）。
  - `rg -c 'Contract Coverage Ledger' docs/templates/plan-packet.md` が一致なし（exit 1）。baseline: `3`（`:123`・`:133`・`:199`）。
  - `rg -c '受理される fixture' docs/templates/plan-packet.md` が 1、`rg -c 'import し何を描画' docs/templates/plan-packet.md` が 1。baseline: どちらも一致なし（exit 1）。
  - `rg -c '先行 round の結果' docs/templates/plan-packet.md` が `1`。baseline: 一致なし（exit 1）。
  - `rg -c 'Required Design Artifacts' docs/design-system/reference/README.md` が一致なし（exit 1）。baseline: `1`（`:32`）。
- AC6（WER の撤去）: `test ! -e docs/templates/workflow-effectiveness-review.md && test ! -e .agents/skills/workflow-effectiveness-review` が exit 0。baseline: exit 1。`rg -n 'complete Workflow Effectiveness Review|templates/workflow-effectiveness-review|WER を完了' docs/DEV_WORKFLOW.md` が一致なし（exit 1）。baseline: 4 行（`:37`・`:419`・`:421`・`:454`）。
- AC7（review-only の統合）: 次の file ごとの `rg -c -i 'review-only' <file>` が一致なし（exit 1）。baseline は file ごとに `docs/code_review.md` 3、`docs/templates/plan-packet.md` 1、`docs/templates/subagent-review-packet.md` 1、`docs/AGENT_OPERATING_MANUAL.md` 2、`docs/project-profile.md` 1、`docs/DOC_STYLE_GUIDE.md` 1、`.github/pull_request_template.md` 2、`.agents/skills/inventory-code-review/SKILL.md` 1（後ろの 3 file は G3 で Coordinator が認めたとき）。`docs/DEV_WORKFLOW.md` だけは `rg -c -i 'review-only' docs/DEV_WORKFLOW.md` が `1`（旧称の 1 文。baseline `8`）で、`rg -c '旧称 review-only sub-agent' docs/DEV_WORKFLOW.md` が `1`（baseline 一致なし）。
- AC8（Findings Freeze は原則だけ残る）: `rg -c 'Findings Freeze' docs/DEV_WORKFLOW.md` が 1 以上（baseline `2`）、`rg -c 'Findings Freeze' docs/templates/plan-packet.md` が一致なし（baseline は AC5 の `:270` の 1 行）。
- AC9（3 分類の撤去、K3 の保持）: `rg -c 'candidate safety|mutation authority|evidence quality' docs/DEV_WORKFLOW.md` と `rg -c 'candidate safety|mutation authority|evidence quality' scripts/tests/doc-consistency-plan-packet.test.sh` がどちらも一致なし（exit 1）。baseline: `1` と `3`。`rg -c 'actual harm path' docs/DEV_WORKFLOW.md` が `1`（baseline `1`）。
- AC10（Owner Effort Budget、G2 = A の場合）: `rg -n 'interventions ≤3|relay round-trips ≤2|hard stop, not a target' docs/DEV_WORKFLOW.md` が一致なし（baseline: 1 行 `:262`）。`rg -c '介入 6 回' docs/DEV_WORKFLOW.md` が `1`、`rg -c 'L3 の round' docs/DEV_WORKFLOW.md` が `1`（baseline: どちらも一致なし）。`rg -c 'goal-drift signal' docs/DEV_WORKFLOW.md` が 1 以上（baseline `2`）。`rg -c -i 'candidate-safety' docs/DEV_WORKFLOW.md` が一致なし（baseline `1`、`:269`）。`rg -c '§3.5 の停止が優先' docs/DEV_WORKFLOW.md` が `1`（baseline 一致なし）。
- AC11（問い合わせの行き先と導線）: `rg -n '^### 問い合わせの行き先' docs/DEV_WORKFLOW.md` が一致なし（baseline `:272`）。`rg -n 'DEV_WORKFLOW.md#問い合わせの行き先' docs/AGENT_OPERATING_MANUAL.md` が一致なし（baseline `:185`）。`rg -n 'AGENT_OPERATING_MANUAL.md#56-' AGENTS.md` が 1 行（baseline 一致なし）。
- AC12（Wave と D-055）: `rg -n '2〜3 lane|同じ source document を編集する lane は同居させず|生成 file を再生成する lane は 1 wave に 1 つまで' docs/DEV_WORKFLOW.md` が一致なし（baseline: 2 行 `:234`・`:245`）。`rg -c 'directoryRenames=false' docs/DEV_WORKFLOW.md` が 1 以上（baseline `2`）。`rg -n '全 lane の Ready 遷移実行を Coordinator に委任' docs/DEV_WORKFLOW.md` が 1 行（baseline `:239`、AGENTS が参照する train 委任）。
- AC13（Draft PR / closeout）: `rg -n 'inventory-records-other-details|Workflow-change dogfood' docs/DEV_WORKFLOW.md` が一致なし（baseline: 2 行 `:392`・`:394`）。`awk '/^## Draft PR Checkpoint/,/^## Post-Merge Closeout/' docs/DEV_WORKFLOW.md | rg -c 'PR body freshness'` が `1`（baseline: 一致なし、今は Contract Audit にある）。`rg -n 'Verify \+ Review' docs/DEV_WORKFLOW.md` が一致なし（baseline 2 行 `:381`・`:449`）。`rg -n '6\. Review -> 6\.5 Draft PR' docs/DEV_WORKFLOW.md` が一致なし（baseline `:7`）。`rg -n '独立レビューを完了する' .agents/skills/inventory-implementation/SKILL.md` が一致なし（baseline `:22`）。
- AC14（見出しと安全の規則の保持）: `rg -c -x -F -e '## Risk Tiers' -e '## Plan Packet Rules' -e '## Workflow State' -e '## Design Phase Rules' -e '## Implementation Rules' -e '## Wave Operation' -e '### Stacked train' -e '## Owner Effort Budget' -e '## Verification Gates' -e '## Review Rules' -e '## Contract Audit (R3/R4)' -e '## Draft PR Checkpoint' -e '## Post-Merge Closeout' -e '## Done Definition' docs/DEV_WORKFLOW.md` が `14`（baseline `14`、inbound の anchor）。次の file ごと・語ごとの `rg -c` が 1 以上（baseline は括弧内、語ごとに分ける）: `docs/DEV_WORKFLOW.md` の `Double Audit|Double audit`（`3`）、`State Lifecycle`（`1`）、`Adjacent Pattern`（`1`）、`Negative-space`（`1`）、`Drift-fix sweep`（`2`）、`Cited test existence|引用 test の実在`（`1`）、`escape hatch|絶対保証`（`1`）、`adjacent-contract sweep|隣接する契約の sweep`（`1`）。`docs/templates/plan-packet.md` の `adjacent-contract sweep|隣接する契約の sweep`（`1`）。Impact Review Lenses の問いの列の移送: `rg -c 'Which claims are observed facts' docs/templates/plan-packet.md` が `1`、`rg -c 'Question to answer' docs/templates/plan-packet.md` が `1`（baseline どちらも一致なし）。`rg -c 'Which claims are observed facts' docs/DEV_WORKFLOW.md` が一致なし（baseline `1`、`:175`）。
- AC15（受理 P1/P2 の固定）: `rg -c 'fails on the pre-fix code' docs/code_review.md` が `1`、`rg -c 'reverting the fix turns it red' docs/code_review.md` が `1`（baseline: どちらも一致なし）。`rg -c 'prefer adding a regression test' .agents/skills/test-design/SKILL.md` が一致なし（baseline `1`）。
- AC16（小さな項目、file ごとの anchor）: `rg -c '確かめた所を併記' docs/DEV_WORKFLOW.md` が `1`（baseline 一致なし）。`rg -n "its \`Plans.md\` entry" docs/DEV_WORKFLOW.md` が一致なし（baseline `:68`）。`rg -c 'owner の記録と照合' docs/DEV_WORKFLOW.md` が `1`（baseline 一致なし）。`rg -c '防ぐ失敗を 1 文で' docs/DEV_WORKFLOW.md` が `1`（baseline 一致なし）。`rg -c 'oracle' docs/templates/test-design-matrix.md` が 1 以上、`rg -c 'tracked Workflow State stores the current PR HEAD' docs/templates/test-design-matrix.md` が一致なし（baseline 一致なしと `1`）。`rg -c '01-decision-rules.md' .agents/skills/inventory-operator-ui/SKILL.md` が `1`（baseline 一致なし）。`rg -c 'design lane は' docs/AGENT_OPERATING_MANUAL.md` が `1`（baseline 一致なし）。`rg -c '是正 commit' docs/AGENT_OPERATING_MANUAL.md` と `rg -c '是正 commit' docs/templates/subagent-review-packet.md` がそれぞれ `1`（baseline 一致なし）。`rg -c '確信度: ' docs/code_review.md` が `1`（baseline 一致なし）。
- AC17（durable decision）: `rg -c '^## D-098' docs/decision-log.md` が `1`。baseline: 一致なし（exit 1）。
- AC18（mutation、Test Plan の MU1〜MU13）: 実装を commit した後、`$TMPDIR` の写し（`copy="$TMPDIR/pr4-mut"; mkdir -p "$copy"; git archive HEAD | tar -x -C "$copy"; git -C "$copy" init -q; git -C "$copy" add -A`。改変ごとに作り直し、終わったら消す。本 repo の index・設定は触らない）で各 mutation を入れ、対応する test が red（exit 非 0）になり、改変なしの写しでは green になる。
- AC19（検査の全体、すべて exit 0）: `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/doc-consistency-check.sh`、`bash scripts/doc-consistency-check.sh --target plan`、`bash scripts/check-workflow-git.sh`、`git diff --check origin/main...HEAD`、`bash scripts/local-ci.sh full`。baseline: run-workflow-tests exit 0（末尾 `OK`）、doc-consistency exit 0（`結果: 全チェック通過`）、check-workflow-git exit 0（`PK5 検査 OK`）、`git diff --check origin/main...HEAD` exit 0。`--target plan` は本 commit の実測（AC3）。`bash scripts/local-ci.sh full` は `未実測`（起草役は実行していない。Plan Review か Writer が実行する）。
- AC20（範囲、S 全体）: `git diff --name-status origin/main...HEAD` の変更 file が S1〜S16 の file と本 packet・Matrix に限られる。`git diff --stat origin/main...HEAD -- scripts/ci scripts/pr-gate.py scripts/local-ci.sh scripts/pre-push.sh .gitignore .claude docs/agent-guidance docs/ci.md docs/Plans.md docs/backlog.md docs/archive` が空。座組表が変わらない: `diff <(git show "$(git merge-base origin/main HEAD)":docs/AGENT_OPERATING_MANUAL.md | awk '/^## 座組/,/^## 4\./') <(awk '/^## 座組/,/^## 4\./' docs/AGENT_OPERATING_MANUAL.md)` が exit 0（baseline exit 0）。

## Design Sources

- Requirements / spec: 該当なし（製品要件を変えない）。
- Architecture / Function / DB / Screen: 該当なし。
- Workflow 正本（本 lane が書き換える文書と、参照だけする文書）: `docs/DEV_WORKFLOW.md`（Artifact Map、Risk Tiers、Plan Packet Rules、Workflow State の停止の段落、Design Phase Rules、Wave Operation、Owner Effort Budget、Human Visual Confirmation、Review Rules、Contract Audit、Draft PR Checkpoint、Post-Merge Closeout、Done Definition）、`docs/templates/plan-packet.md`・`test-design-matrix.md`・`subagent-review-packet.md`、`docs/code_review.md`、`docs/AGENT_OPERATING_MANUAL.md`（§2・§3・§4・§5.6・§6）、`AGENTS.md` の Decision and Approval Boundaries（参照と link 1 つ）、`scripts/pr-gate.py`（参照だけ: Plan Commit の書式 `:19`・`:171-176`、R4 と Minimum `:184`・`:271-272`）、`scripts/check-workflow-git.sh`（参照だけ: PK5 の Plan Commit の読み方 `:52-61`）。
- Decision log / ADR: D-034（Session Start・Workflow State・template）、D-038（Findings Freeze・Owner Effort Budget）、D-046（3 分類・WER の Retired 節・budget の承認インターフェース）、D-055（wave）、D-074（stacked）、D-090（Ordinary Operation）、D-093（問い合わせの行き先の所有）、D-097（並走の摩擦）。本 lane は D-098 を足す。
- 公式資料: Opus 5.5 prompting guide「Unattended agentic runs」（止めたくない停止と止めたい停止を具体的に名指しする指示に model がよく反応する）。本 lane の「止める規則は防ぐ失敗を 1 文で書く」は、同じ考え方を repository の規則の書き方へ当てたもの（owner 2026-09-24「公式 Opus 5.5 prompting guide と照合して進める」）。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | existing sufficient（製品コードを変えない） |
| Command / DTO / generated binding / wire shape | なし | existing sufficient |
| DB / transaction / audit / rollback / migration | なし | existing sufficient |
| Screen / UI / route state / Japanese wording | なし | existing sufficient |
| CSV / TSV / report / import / export format | なし | existing sufficient |
| Workflow gate（PK1・PK3・PK4・WER の検査、template、review の規則） | `docs/DEV_WORKFLOW.md`、`docs/templates/*`、`docs/code_review.md`、`scripts/doc-consistency-check.sh` | updated in this PR（S1〜S15） |
| Durable decision / ADR | `docs/decision-log.md` D-098 | updated in this PR（S16） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| decision-log の追記 | D-098（本 packet で予約）。PR5 が先に merge していれば番号順に並べる |
| workflow 文書・template・Skill の削除（WER の template と Skill） | 親文書の索引（DEV_WORKFLOW の Artifact Map `:37`）の行を同じ PR で消す。link の実在は `bash scripts/doc-consistency-check.sh` の link 検査で確かめる（inbound は `:37` の 1 件、Contract Probe 7） |
| template の節の改名（Contract Ledger・Design Readiness） | checker の PK1・PK3、code_review、Skill、subagent-review-packet の参照を同じ PR で直す（S2・S10・S11・S12） |
| test の追加・置き換え（S3） | `scripts/tests/run-workflow-tests.sh` が既に実行している file（新しい test file は無い） |

Tauri command・function-design doc・REQ・route・operator 画面: 該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-HARNESS4 | template、DEV_WORKFLOW Plan Packet Rules、PK1・PK3 | D1・D2 | 下の「設計判断 1」 | S1・S2 | AC2（PR4-F1・F2・F4・F4b・F9・F11）、AC5 |
| SPEC-WF-HARNESS4 | PK4 | D3 | Findings Freeze の記録行は helper の broad / closure の仕組みと重複し、行が無いことで防ぐ失敗を 1 文で言えない（監査 K6）。`Plan Commit` の書式は教訓 42 の事故（凍結後の branch の作り直し）を 1 文で言える。書式を helper と同じ 40 桁にする理由は「設計判断 3」 | S2 | AC2（PR4-F3・F5〜F7） |
| SPEC-WF-HARNESS4 | DEV_WORKFLOW Post-Merge・Done、WER の template・Skill | D4 | WER は 2026-08-12 を最後に回っておらず、要求は形だけ（監査 M7）。却下: WER を任意で残す（任意の儀式は誰も回さず、Retired 節の検査だけが残る） | S2・S3・S9・S11・S12 | AC6、PR4-F10 |
| SPEC-WF-HARNESS4 | Risk Tiers、Review Rules、code_review | D5 | 下の「設計判断 2」 | S4・S7・S10〜S15 | AC7 |
| SPEC-WF-HARNESS4 | Owner Effort Budget | D6 | 下の G2 | S1・S6 | AC10 |
| SPEC-WF-HARNESS4 | Review Rules、Workflow State の停止 | D7 | 3 分類が防ぐ失敗（D-046 の Why: 証跡の欠けが候補の安全と競合する）は `docs/DEV_WORKFLOW.md:57` と `:336` が守り、分類は Ordinary Operation の finding の行き先を持たなかった（backlog `:61`）。停止の原則は owner 2026-09-28。却下: 3 分類に「普通の一日の成立」を 4 つ目として足す（分類を増やしても裁定は P1〜P3 の重大度で行っている） | S5・S7 | AC9、AC16 |
| SPEC-WF-HARNESS4 | Design Phase、Contract Audit | D8 | 処置表。却下: 列挙を残し「重複しているが読む順を示す」注記を足す（行数は減らず、ずれの元が残る） | S5・S7 | AC14 |
| SPEC-WF-HARNESS4 | Wave Operation | D9 | owner 2026-09-24 の不適用を正本にする。wave 13・14 は所有表と merge 順で解けた。却下: lane 数を 4〜5 に上げる（数に失敗の根拠が無い）。生成物の 1 lane の上限を残す（traceability は再生成で解けると並走の摩擦 packet が確かめた） | S8 | AC12 |
| SPEC-WF-HARNESS4 | Draft PR、Post-Merge、Done | D10 | 一過性の記述と形だけの要求を消し、helper の手順と重なる所は参照にする | S9 | AC13 |
| SPEC-WF-HARNESS4 | AGENTS、MANUAL §5.6、DEV_WORKFLOW | D11 | 同じ境界が 2 か所（I6）。AGENTS が上位（D-093） | S6・S13・S14 | AC11 |
| SPEC-WF-HARNESS4 | code_review Same PR・Verification Rules | D12 | 上の「試行の集計」 | S10・S12 | AC15 |
| SPEC-WF-HARNESS4 | template・TDM・Skill・MANUAL・code_review | D13 | 教訓 29・62・64〜68・70・73・103、backlog (2)〜(4)。どれも起きた失敗を 1 文で言える（教訓の棚卸しの表） | S1・S4・S7・S10〜S13 | AC5、AC16 |
| SPEC-WF-HARNESS4 | 全体 | D14 | 安全の境界を変えない（Impact Review Lenses） | 変えないもの | AC1、AC9、AC14、AC19 |

### 設計判断 1: Contract Ledger の列（Plan Review で覆せる形）

- 採る: `契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象`。旧 4 表の情報のうち、ID・正本・実装・test・L3 の対応を 1 行で持つ。Trace Matrix の Review Focus 列は `## Review Focus` 節が、Evidence 列は test 列と PR の record が、Design Intent Trace の Why / rejected 列は設計正本と decision-log（`docs/DEV_WORKFLOW.md:156`「Source design docs carry the decision, why, rejected alternatives」）が持つ。
- 棄却: (a) 4 表を残す（同じ ID と test を 4 回書かせる、監査 J4）。(b) 2 表（契約の一覧 + 追跡表）にする（ID と test の列がまだ 2 回）。(c) Why 列を足す（plan にだけ理由が残る書き方を促し、code_review `:9` が drift とする形になる）。
- 旧形式の受理: PK1 は `## Contract Ledger` か、`## Spec Contract` と `## Trace Matrix` の組のどちらかを求める。組の片方だけは ERROR（今と同じ強さ）。旧 `## Contract Coverage Ledger` の名前は Ledger の別名として受けない: 今の R3 packet は Spec Contract と Trace Matrix を必ず持つ（PK1 が要求してきた）ので、組の経路で通る。

### 設計判断 2: review-only と Final Review の統合の形（Plan Review で覆せる形）

- 採る: 語の統合。R2+ の Final Review（broad audit）の本数は Workflow State の `Final Review Minimum`、skip の経路は無い。PK1 の R4 skip の ERROR と PK3 の skip の WARN は消す。
- 安全の根拠: R4 の review の必須は、PK4 の「R4 は Final Review Minimum 2」（`scripts/doc-consistency-check.sh:1271-1273`）と helper の `R4 gates missing`（`scripts/pr-gate.py:184`）・broad の本数（`:314`）が強制する。R4 の packet が `Review-only skipped because:` を書いても、helper は broad 2 本が record されるまで Ready にしない。したがって行の検査は重複で、消しても R4 の review は弱まらない（PR4-F8 が R4 で Minimum 1 の packet の ERROR を確かめる）。
- 棄却: (a) PK1 の R4 skip 検査を残す（概念の無くなった行を検査し続け、語の二重を残す）。(b) review-only を「Plan Gate 前の軽い review」として別に残す（今の座組では Plan Review が fresh Opus + Codex で、同じ役を持つ）。

### 設計判断 3: `Plan Commit` の書式（Plan Review で覆せる形）

- 採る: 値全体が `pending` か `[0-9a-f]{40}`。helper の `sha()`（`scripts/pr-gate.py:19` `SHA = re.compile(r'[0-9a-f]{40}\Z')`）と同じ集合にし、値は末尾の空白・タブを削らずに照合し（helper は `fullmatch`）、helper が record の時点で拒む値を commit の前に止める。
- 棄却: (a) 教訓 42 の案の 7〜40 桁（PK4 を通っても helper が `invalid full SHA` で拒み、早く止める目的に反する）。(b) 先頭の token だけを見る（`extract_workflow_field` の既定。注記付きの値を通すが、PK5 は値全体を SHA として解決するので `解決できない SHA` になる、`scripts/check-workflow-git.sh:52-61`）。
- 影響: archive の packet（短い SHA が 50 件以上ある、Contract Probe 5）は PK4 の対象外のまま。helper 導入後の packet はすべて 40 桁（同 Probe）。

### G1: 分け方（owner に諮る判断点）

範囲が広い（15 file 前後、DEV_WORKFLOW の約 4 割の書き換え）。1 本の review に重すぎるかを owner に諮る。

| 観点 | A: 1 本（推奨） | B: 2 本（PR4a 検査と template → PR4b 文の縮約） |
|---|---|---|
| 中身 | S1〜S16 | PR4a = S1・S2・S3・S7 の review-only と Findings Freeze の行・S10 の Final Review Protocol・S11 の subagent-review-packet と WER template・S12 の WER Skill と test-design の参照・S15・D-098 の (2)〜(4)(6)(10)。PR4b = 残り（S4〜S9 の文の縮約、Owner Effort Budget、Wave、問い合わせ、B・C・D の項目、D-099 以降の番号） |
| review の重さ | 重い。処置表と S ごとの AC で見る所を区切る | 各本は軽い。ただし両方が DEV_WORKFLOW・template・test file を触るので直列になる |
| 手間 | Plan Review 最大 3 round、Final Review 2 本、介入 6 | Plan Review と Final Review が 2 倍、介入は約 2 倍（起票・plan-approved・Ready・merge が 2 組） |
| 期間 | PR5 と並走で 1 回 | PR4a の merge を待って PR4b（owner の順番「ハーネス PR4・PR5 ＞ 作業」の間が延びる） |
| 失敗の risk | 縮約の review が検査の変更の review と注意を分け合う | 検査の変更は小さく見やすいが、PR4a の間は template と DEV_WORKFLOW の語がずれる（Contract Ledger が template にあり DEV_WORKFLOW の Plan Packet Rules は旧名） |

推奨: A。理由: 検査の変更（PK1・PK3・PK4・WER）は数十行で小さく、重さの大半は文の縮約で、それは処置表の各行を移し先の anchor（AC14）で機械的に確かめられる。B は同じ file を直列に 2 回通すので、review の手間の総量と待ちが増え、間の期間に語のずれが生じる。B へ戻す条件: Plan Review の reviewer が範囲の広さを理由に見落としの risk を具体的に挙げた場合は、B へ戻して plan-draft からやり直す（本 packet の B 列が非目的の分け方）。各 round の reviewer の G1 への意見は Review Response に書く。

B を選んだ場合の非目的: PR4a は文の縮約（Design Phase・Contract Audit・Wave・Draft・closeout・Owner Effort Budget・問い合わせ）と B・C・D の項目を含めない。PR4b は checker・test を変えない（test の section 17 の 3 分類の assert の撤去だけは PR4b に入る）。

### G2: Owner Effort Budget の値と扱い（owner に諮る判断点）

実績（起点の archive、`rg -n -m1 '^- 介入回数上限' docs/archive/plans/2026-09-2*.md`）: 2026-09-22 以降の 17 件のうち、既定 3 のまま閉じたのは 3 件（docs-restore・ordinary-operation・nav-return）。上限の中央値は 7。改定の形は 3 つに分かれる。(a) 別の判断と同じ 1 回で改定した lane: 並走の摩擦（`2026-09-28-harness-parallel-friction.md:43`、plan-approved と同じ 1 回で 5→6）、PR3（`2026-09-25-harness-pr3-entry-and-dedup.md:37`）・PR2（`2026-09-25-harness-pr2-roles-and-formation.md:39`、「本改定と plan-approved の承認 1」）、小口（`2026-09-27-small-fixes-batch.md:33`）、色と強調（`2026-09-27-design-color-emphasis.md:39`）、Z001 表示（`2026-09-27-daily-report-z-display.md:43`、Ready と同じ 1 回）、Z004（`2026-09-23-legacy-stocktake-z004-write-stop.md:36`）。(b) 引き上げの承認（と数え直しの確認）だけで 1〜2 回を使った lane: EJ（`2026-09-23-ej-parser-core.md:36`）、legacy 撤去（`2026-09-24-harness-legacy-and-execution-mode-removal.md:37`）、design-rules（`2026-09-24-design-rules-renewal.md:44`）、棚卸時刻 ADR（`2026-09-23-stocktake-time-evidence-adr-revision.md:53`）。評価基準（`2026-09-25-stocktake-valuation-basis.md:35`）は「予算の引上げ 1」を単独の介入として見込んだ。(c) 事前の改定なしに超過した lane: PR2（上限 10 に消費 12、`:39`「事前に上限の改定を諮っていない」）。relay の上限は 2 から 5〜7 に改められ、並走の摩擦では relay 4 は Coordinator が Codex を起動した往復で owner の手間に数えていない（`:45`）。

| 案 | 介入 | 実働 | relay | 上限に届くとき |
|---|---|---|---|---|
| A（推奨） | 既定 6 | 既定 30 分 | 上限を置かない | 次の判断と上限の改定を同じ 1 回で諮る（止まって別の承認を足さない）。goal-drift signal は即停止 |
| B | 既定 6 | 既定 30 分 | 既定 5（Plan Review 3 + broad 1 + closure 1） | hard stop のまま |
| C（監査 Q1 の推奨） | 上限を置かず、承認依頼の形式（介入 N 回目・完了 1 文）だけ | 置かない | 置かない | goal-drift signal だけ |

推奨: A。理由: (1) owner の「残して緩める」に沿って介入と実働の数値を残す。(2) 既定 6 は実績の平均でなく、判断点の積算による提案: 起票承認・plan-approved・Ready・merge の 4 に、Plan Gate の判断点 1〜2 を足した値。中央値 7 と近く、manual の lane は L3 の round ごとに 1 回足し、packet が理由を書いて上げる。(3) relay は Coordinator が Codex を起動する往復で owner の手間でなく、上限は owner の方針「relay の節約を目的にしない」と緊張する（教訓 49）。review の round は Plan Review の round 天井 3 が既に抑える。(4) hard stop を「同じ 1 回で諮る」に置き換える根拠は、(a) の 7 lane がすでにその形で運用し、(b) の 4 lane では引き上げの承認と数え直しの確認だけで owner の 1〜2 回を使い、(c) の PR2 では hard stop が超過を防がなかったこと。つまり hard stop は止める働きより承認を 1 回増やす働きが大きかった。B は relay の上限を残す（owner の「残す」に字義どおり沿うが教訓 49 の緊張が残る）。C は「残して緩める」に反する。教訓 40（L3 の round の計上）はどの案でも「L3 の各 round を 1 回」と正本に書く。

### G3: 所有表の外の 3 file（Coordinator の裁定）

S15 の 3 file は review-only の語の統合に要る。所有表に無く、PR5 の表にも無い。本 packet は Scope に入れ、Coordinator が認めない場合は S15 を Non-scope に移し、その 3 file の旧い語は DEV_WORKFLOW の旧称の 1 文で読めるものとして残す（AC7 の該当 3 file の行を外す）。

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 規則は S1〜S15 の正本に、判断と棄却案は D-098 に置く。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D1〜D13 を DEV_WORKFLOW・template・code_review・MANUAL・Skill と D-098 に置く。処置表の各行の結果は正本の文そのもので、表は本 packet の review の道具（D-098 に要約だけ置く）。
- Assumptions and constraints: 旧 template の active packet（本 packet と PR5 の packet）は Spec Contract と Trace Matrix を持つ（今の PK1 が要求している）。helper は変えない。PR5 の packet の `Plan Commit` は 40 桁か `pending`（helper が既に要求。PR5 が先に merge して packet が main に残っても、本 lane の PK4 の書式検査で red にならない。Coordinator が PR5 と合わせる）。
- Deferred design gaps, risk, and follow-up target: Amendments の書式検査（Non-scope）、MANUAL の全面的な縮約（backlog）、workflow の軽量化 3 段の 1 段目の残りと 2 段目（Non-scope、closeout で backlog の記述を直す）。
- Test Design Matrix can cite design decision IDs or source doc sections: D1〜D14 を Matrix が引く。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「旧 template の packet は通り続ける」の例外 = Spec Contract と Trace Matrix の片方だけを持つ packet（今も ERROR、変えない）と、`Plan Commit` が `pending` でも 40 桁でもない active packet（新たに ERROR。main の active packet は 0 件〈`docs/plans/` が無い〉で、helper 導入後の archive はすべて 40 桁、Contract Probe 5）。「安全の境界を弱めない」の例外 = なし（Impact Review Lenses の 6 点）。「R4 の review は必須」の例外 = なし（PK4 と helper）。

## Impact Review Lenses

安全の境界が弱まらないことを、指定の 6 点を中心に論じる。現場調査・実機・POS・CSV の lens は該当なし（1 行ずつ理由を書く）。

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 該当なし: 製品の adapter と core を変えない | — |
| Fact check / design decision split | 事実 = 起点の各行（範囲の候補の確認）、試行の集計、Owner Effort Budget の実績、helper と PK5 の Plan Commit の読み方（Contract Probe）。判断 = D1〜D13、設計判断 1〜3、G1〜G3 | Contract Probe、D-098 |
| Lifecycle / retry | packet の状態の検査（PK4 の Phase と Plan Commit の組）は変えず、書式の検査を足す。旧 template の packet が新しい checker の下で通り続ける（Ordinary Operation の最後の行） | Matrix の State Lifecycle |
| Operator workflow | 該当なし（店の operator の操作を変えない）。開発の操作列は Ordinary Operation | — |
| Replacement path | template の節の名前が変わっても、PK1 は旧い組を受けるので、並走 lane の packet を書き直す必要が無い。旧い組の受理を外すのは、旧 template の active packet が無くなった後の別の変更 | D-098 |
| Data safety / evidence | 古い証拠の拒否: helper（record の head/base の照合、capture と server の record の一致、approved snapshot の照合）を変えない（`scripts/pr-gate.py` は非目的） | AC20 |
| Reporting / accounting semantics | 該当なし | — |
| Manual verification | 該当なし: 画面が無い。checker は合成 fixture で確かめる | — |
| 環境・再現性 | 新しい環境依存なし。checker は同じ bash と `rg`（`require_linux_ripgrep`）で走る | — |

指定の 6 点:

- 独立 review: Writer ≠ Plan Reviewer・Final Reviewer、fresh context、同じ round の他の reviewer の結果を読ませない（MANUAL §3、本 lane は例示に是正 commit を足して強める）、Double Audit（Final Review Minimum 2、PK4 と helper）を変えない。review-only の語の統合は review の本数を減らさない（R3 の review-only は既に Final Review の broad が兼ねていた。並走の摩擦 packet の Risk の skip の記録）。
- Plan Commit の固定: PK5（`scripts/check-workflow-git.sh`）の祖先・不変性の検査は変えない。PK4 の書式検査を足し、commit の前の `--target plan` で誤った値を止める（強める）。commit 済みの値は PK5 が固定するので、その場合は同じ commit を直す。
- Human Gate: PK4 の Human Gate の検査（`ready,merge` と R4 の `r4`、`:1274-1278`）と helper の検査を変えない。owner の Ready・merge・L3・R4 の判断は変えない。Owner Effort Budget の改訂（G2 = A）は上限に届いたときの扱いを「止まって承認を足す」から「次の判断と改定を同じ 1 回で諮る」に変えるが、owner の判断を省かない（改定そのものが owner の判断）。goal-drift signal の即停止と one-shot irreversible の time-box の停止（MANUAL §3.5）は残す。
- R4 の必須: 設計判断 2。PK4 の R4 の Minimum 2 と `r4`、helper の `R4 gates missing` が強制し、PR4-F8 が確かめる。
- 古い証拠の拒否: helper を変えない（Data safety の行）。Findings Freeze の記録行の撤去は、helper の broad / closure の record（head/base つき）が同じ事実を持つため（監査 K6）。
- 不可逆 finding の 4 項目（K3）: DEV_WORKFLOW Review Rules に語を変えずに残し、test の section 17 の assert（`actual harm path` ほか 4 語）を残す（AC9・MU9）。

## Design Readiness

- Existing design docs are sufficient because: 足りない（spec-check の結果）。design の出力は本 packet の Spec Contract・処置表・設計判断にあり、正本への昇格は S1〜S16。
- Source docs updated in this PR: DEV_WORKFLOW、plan-packet・test-design-matrix・subagent-review-packet の template、code_review、MANUAL（座組表の外）、AGENTS（link 1 つ）、Skill 4 本、decision-log D-098、S15 の 3 file（G3）。
- Design gaps intentionally deferred: Non-scope の各項目。
- Durable decisions discovered in this plan and promoted to source docs: D-098。

Minimum design checks for business-app work: 製品コードを変えないため、layer・function・DTO・永続化・画面・error の各項目は該当なし。testability は Matrix。

## Contract Probe

- 1「PK1 は節の見出しの行一致で判定し、本文の語は見ない（旧い組と新しい Ledger のどちらでも通す実装にできる）」: `sed -n '1050,1054p' scripts/doc-consistency-check.sh`（`grep -qE "^#{2,}[[:space:]]+${section}([[:space:]].*)?$"`）を読んだ → 成立。
- 2「archive の packet は PK4 の対象外で、PK1 も archive では D-039 の節を要求しない」: `sed -n '1039,1048p;1225p' scripts/doc-consistency-check.sh` を読んだ → 成立。書式検査を PK4 に足しても archive は壊れない。
- 3「helper は Plan Commit に 40 桁の小文字 hex か `pending` だけを受ける」: `sed -n '19p;171-178p' scripts/pr-gate.py` → `SHA = re.compile(r'[0-9a-f]{40}\Z')`、`plan == 'pending'` か `sha(plan)` → 成立（設計判断 3）。
- 4「PK5 は `Plan Commit` の値全体（末尾空白は削る）を SHA として解決し、注記付きは解決できない。初回値は commit 履歴から固定する」: `sed -n '52,63p;93,104p' scripts/check-workflow-git.sh`（`:53` で末尾空白を削り、`:60` で解決、`:96-100` で `git log --follow -p` の最古の non-pending 値を初回値）→ 成立。commit 済みの値は push 前でも固定される。
- 5「main に active packet は無く、helper 導入後の archive の Plan Commit は 40 桁」: `ls docs/plans/` → `No such file or directory`。`rg -n '^- Plan Commit:' docs/archive/plans/2026-09-1*.md docs/archive/plans/2026-09-2*.md | grep -vE ':- Plan Commit: [0-9a-f]{40}$'` → `2026-09-10`〜`2026-09-13` の 3 件だけ（helper 導入前）。archive 全体では短い SHA が多数（PK4 の対象外）→ 成立。
- 6「link 検査は anchor を検査しない（`#問い合わせの行き先` の見出しを消しても decision-log の過去の link は red にならない）」: `sed -n '1646,1647p' scripts/doc-consistency-check.sh`（`path#anchor → path` で anchor を除去）→ 成立。
- 7「WER の template と Skill への markdown link は DEV_WORKFLOW の 1 件だけ」: `rg -n -o "\]\([^)]*(templates/workflow-effectiveness-review\.md|skills/workflow-effectiveness-review[^)]*)\)" docs .agents .github AGENTS.md CLAUDE.md --hidden` → `docs/DEV_WORKFLOW.md:37` の 1 件 → 成立。
- 8「test が DEV_WORKFLOW の 3 分類の語の存在を要求している（同じ PR で assert を外さないと AC1 が red）」: `sed -n '770,772p' scripts/tests/doc-consistency-plan-packet.test.sh` → 3 行の `assert_contains` → 成立。
- 9「PK2 と P2 の検査は本 packet の語で ERROR にならない（表の中の未解決の marker の語を避けた）」: 本 commit で `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-09-29-harness-pr4-lightweight.md` → exit 0、末尾 `結果: 全チェック通過`（AC3）→ 成立。
- 10「40 桁+末尾空白は checker の full 抽出と PK5 を通り、helper が拒む」: `sed -n '948p' scripts/doc-consistency-check.sh`（`s/[[:space:]]+$//`）、`sed -n '53p' scripts/check-workflow-git.sh`、`sed -n '72,73p;147,155p' scripts/pr-gate.py`（`fullmatch`、値の末尾を保つ）。実験（2026-09-30、起草役。同じ command で誰でも再実行できる）: `python3` で `importlib.util.spec_from_file_location` により `scripts/pr-gate.py` を module として読み込み（GitHub には触れない）、本 packet の本文の `- Plan Commit: pending` を 40 桁+空白 / 40 桁+タブ / 7 桁（Phase は `implementing`）に置き換えた文字列を `parse_packet` に渡す → 3 件とも `GateError: invalid full SHA`。40 桁（`implementing`）と `pending`（`plan-gate`）は通る → 成立（設計判断 3 の「末尾を削らない」の根拠）。
- 11「pre-push は docs の分類で doc-consistency を回し、PK5 が先」: `sed -n '186,188p;204,208p' scripts/pre-push.sh`、`sed -n '67p' scripts/ci/classify-changes.sh`（`docs/*` → docs=true）→ 成立。docs だけの branch でも push 前に ERROR は出るが PK5 の後。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D1 template の構成（Contract Ledger・Design Readiness・R2 の短い本体・Consultation Relay の撤去・B2・B3） | S1 | AC5、PR4-F1 | — |
| D2 PK1 の Ledger か旧い組、PK1 の R4 skip ERROR と PK3 の skip WARN の撤去、PK3 の Ledger の読み | S2・S3 | AC2（PR4-F1・F2・F2b・F4・F4b・F8・F9・F11）、AC4、MU1〜MU3・MU8・MU11・MU13 | — |
| D3 PK4 の Findings Freeze 行の撤去と Plan Commit の書式 | S2・S3 | AC2（PR4-F3・F5〜F7・F12）、AC4、MU4〜MU7・MU10・MU12 | — |
| D4 WER の撤去 | S2・S3・S4・S9・S11・S12 | AC6、PR4-F10 | — |
| D5 review-only の統合 | S4・S7・S9〜S15 | AC7、PR4-F8 | S15 は G3 |
| D6 Owner Effort Budget | S1・S6・S13 | AC10、AC1（section 17 の counter の assert） | G2 |
| D7 3 分類の撤去、停止の原則、scope 増減の照合 | S5・S7 | AC9、AC16 | — |
| D8 Design Phase・Contract Audit の縮約（処置表） | S5・S7 | AC14 | — |
| D9 Wave・stacked・D-055 | S8 | AC12 | — |
| D10 Draft PR・closeout・Done | S9 | AC13、AC6 | — |
| D11 問い合わせの行き先と導線 | S6・S13・S14 | AC11 | — |
| D12 受理 P1/P2 の固定 | S10・S12 | AC15 | — |
| D13 小さな項目（B5・B8・C2〜C4・TDM） | S4・S10・S11・S12・S13 | AC16 | — |
| D14 変えない安全の境界 | 変えないもの | AC1（section 17 の K3・goal-drift・one-shot の assert）、AC19、AC20、MU9 | — |
| D15 decision-log | S16 | AC17 | — |
| 隣接: PK4 の Workflow State の 10 field・enum・Human Gate・R4 の Minimum | 変えない | AC1（既存の section 2〜5・7・14・21〜27） | — |
| 隣接: PK4 の Plans.md の pointer（D-097） | 変えない | AC1（section 11〜12・28） | — |
| 隣接: PK5 の祖先・不変性 | 変えない | AC19（`bash scripts/tests/run-workflow-tests.sh` の `workflow-git-checks.test.sh`） | — |
| 隣接: PK6 の数値主張 | 変えない | AC1（section 20） | — |

adjacent-contract sweep: S2 が触る関数の隣で上表に無い契約は、PK2（placeholder・空 bullet）、D-046 の Goal Invariant 構造の WARN、M1〜M3・P1・P2 の検査、link 検査で、どれも変えない。template の節の改名が触る参照は S10〜S13 と S15 で全数を直す（AC7・AC16 の file ごとの `rg`）。

## Test Plan

Test Design Matrix: [2026-09-29-harness-pr4-lightweight.md](test-matrices/2026-09-29-harness-pr4-lightweight.md)

- targeted tests: `bash scripts/tests/doc-consistency-plan-packet.test.sh`（PR4-F1〜PR4-F12 と既存の section）、`bash scripts/doc-consistency-check.sh`、`--target plan`（本 packet を新旧の checker で、AC3）。
- 新しい fixture（S3、見出し `# --- PR4-F<番号>`）
  - PR4-F1: 新 template の R3 packet（`## Contract Ledger`・`## Design Readiness`、Spec Contract・Trace Matrix・Findings Freeze 行・Consultation Relay なし）→ exit 0、PK1・PK4 OK、`Trace Matrix table に data row がありません` の WARN が出ない。
  - PR4-F2: 旧 template の R3 packet（今の既定の fixture。Spec Contract・Trace Matrix・Findings Freeze 行あり）→ exit 0。
  - PR4-F2b: 旧 template の R3 packet で Trace Matrix が header と区切り行だけ（データ行 0 件）→ exit 0、PK3 の WARN `Trace Matrix table に data row がありません` が出る（旧い組にデータ行の ERROR を当てない互換の固定）。
  - PR4-F3: 旧 template で Findings Freeze 行だけが無い R3 packet → exit 0（section 6 の置き換え）。
  - PR4-F4: R3 で Contract Ledger も旧い組も無い → ERROR（`Contract Ledger` を含む PK1 の文言）。PR4-F4b: R3 で Spec Contract だけあり Trace Matrix と Ledger が無い → ERROR。
  - PR4-F5: `Plan Commit` が確定待ちの英字と括弧の注記（Phase plan-gate）→ exit 1、PK4 の書式の文言を assert。
  - PR4-F6: `pending`（Phase plan-gate）と 40 桁の小文字 hex（Phase implementing）→ どちらも exit 0。
  - PR4-F7: 7 桁の hex、39 桁、41 桁、大文字の 40 桁、`pending` に括弧の注記（Phase plan-gate）、40 桁に括弧の注記、40 桁+末尾空白、40 桁+末尾タブ → それぞれ exit 1、書式の文言を assert。
  - PR4-F8: R4 packet（Minimum 2、Human Gate に `r4`）に `Review-only skipped because:` 行 → PK1 の ERROR も PK3 の WARN も出ない。R4 で Minimum 1 → PK4 の ERROR（既存の検査が R4 の review を守ることの確認）。
  - PR4-F9: 新 template の R3 packet の Contract Ledger に存在しない `test_` token → PK3 の WARN（`Contract Ledger` の節名を含む）。Ledger に placeholder → WARN。
  - PR4-F10: 2026-07-15 以降の日付で Retired 節の無い WER を archive に置く → D-046 の WER の WARN が出ず、header `WER Retired` も出ない。
  - PR4-F11: 新 template の R3 packet で Contract Ledger の表がデータ行 0 件（header と区切り行だけ、または template の空行 `|  |  |  |  |`）→ PK1 の ERROR（`データ行がありません`）。
  - PR4-F12: archive の明示 path（`docs/archive/plans/…`）に `Plan Commit: abc1234` の packet → exit 0、PK4 の書式の文言が出ない。
- negative tests: PR4-F4・F4b・F5・F7・F11、PR4-F8 の後半。
- mutation（AC18、各 mutation は実注入して red を確かめる。構造の推論だけで済ませない）
  - MU1: PK1 の Ledger の経路を外す（R3 は Spec Contract と Trace Matrix だけ）→ PR4-F1 が red。
  - MU2: PK1 の Ledger か組の要求そのものを外す → PR4-F4 が red。
  - MU3: 旧い組の条件を AND から OR にする → PR4-F4b が red。
  - MU4: PK4 の書式検査を外す → PR4-F5 と PR4-F7 が red。
  - MU5: 書式検査を先頭の token（`extract_workflow_field` の既定）に当てる → PR4-F7 の注記付きの 2 件が red。
  - MU6: 正規表現を `[0-9a-f]{7,40}` に緩める → PR4-F7 の 7 桁・39 桁が red。
  - MU7: PK4 の Findings Freeze 行の要求を戻す → PR4-F1 と PR4-F3 が red。
  - MU8: PK3 のデータ行を Trace Matrix からだけ読む → PR4-F9 が red（WARN が出ない）、PR4-F1 が red（data row が無い WARN が出る）。
  - MU9: DEV_WORKFLOW の写しから `actual harm path` を消す → section 17 が red（K3 の保持の感度）。
  - MU10: PK4 の書式検査の前に末尾の空白を削る（`extract_workflow_field … full` に戻す）→ PR4-F7 の末尾空白・タブの 2 件が red。
  - MU11: PK1 の Ledger のデータ行の検査を外す → PR4-F11 が red。
  - MU12: PK4 の書式検査を `is_archived_plan_path` の skip の前に置く → PR4-F12 が red。
  - MU13: PK1 の Ledger のデータ行の ERROR を Trace Matrix の経路にも当てる → PR4-F2b が red（exit 1）。
- compatibility checks: 既存の section 1〜5・7〜15・18〜28（旧 template の既定の fixture が通り続ける）、AC3（本 packet が新旧の checker で通る）。
- data safety checks: 変更は script・test・文書だけで、実データ・secret を含まない（Data Safety）。
- main wiring/integration checks: `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/local-ci.sh full`、実装の PR の hosted CI の docs job（PR の head の checker が本 packet を検査する）。

## Boundary / Wire Contract

- producer: Plan Packet（markdown）。
- consumer: `scripts/doc-consistency-check.sh` の PK1・PK3・PK4。helper（`scripts/pr-gate.py`）は変えない。
- wire type: 節の見出しの行（`## Contract Ledger`、`## Spec Contract`、`## Trace Matrix`）、Workflow State の `- Plan Commit:` 行の値全体（末尾の空白・タブを含む）。
- internal type: 節の有無の真偽、`Plan Commit` の値の文字列。
- precision/range: `pending` か小文字 hex 40 桁の完全一致（helper の `fullmatch` と同じ集合。末尾の空白・タブ・注記は不一致）。
- round-trip path: なし（read-only の検査）。
- invalid input: 書式外の `Plan Commit` は PK4 の ERROR（exit 1）。Ledger も旧い組も無い R3 packet は PK1 の ERROR。
- compatibility: 旧 template の packet（Spec Contract と Trace Matrix、`- Findings Freeze:` 行、`Review-only skipped because:` 行、Consultation Relay 節、relay の上限の行）はすべて受理する。archive の明示 path の扱いは変えない。

## Review Focus

- Plan Review の冒頭で `Ordinary Operation` の操作列が目的を達成できるかを `成立 / 具体的な反例あり / 外部前提が未確認` で答える。
- 処置表: delete・merge の各行が、失敗を 1 文で言えない規則か重複だけか。安全の境界に当たる規則が delete に入っていないか。keep の行の移し先の anchor が AC14 で確かめられるか。
- 設計判断 1〜3 と G1〜G3: 採る案の理由と棄却案の評価が現物と合うか。特に設計判断 2 の「R4 の review は PK4 と helper が強制する」と、設計判断 3 の「helper と同じ 40 桁」。
- 旧形式の受理が、並走する PR5 の packet（旧 template）と本 packet を新しい checker の下で通し続けるか。PR5 が先に merge した場合と後の場合の両方。
- 試行の集計から決めた文言（対象を workflow の script へ広げたこと、止まる条件を残したこと）が集計と合うか。
- S15（所有表の外の 3 file）と D-098 の番号の予約が PR5 と衝突しないか。

## Spec Contract

Contract ID: SPEC-WF-HARNESS4

- D1: template は全 Risk の本体と R3/R4 の部分に分かれ、R3/R4 の契約の追跡は Contract Ledger（`契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象`）の 1 表、設計の準備は Design Readiness の 1 節。Consultation Relay 節、Findings Freeze 行、review-only skip の案内、`independent-review` の語は無い。Scope 節に component の import / 描画の表、Test Plan に L3 の fixture の列挙、Review Response に「先行 round の結果はこの節にだけ書く」の 1 行がある。
- D2: PK1 は R3/R4 に `## Contract Ledger` か `## Spec Contract` と `## Trace Matrix` の組を求め、どちらも無ければ ERROR。R4 の review-only skip の ERROR と PK3 の skip の WARN は無い。PK3 の Trace の WARN は Ledger があればその表、無ければ Trace Matrix を読む。`## Contract Ledger` があるときはデータ行 1 件以上を求める（0 件は ERROR）。
- D3: PK4 は R3 に `- Findings Freeze:` 行を求めない。active packet の `Plan Commit` の値全体（末尾の空白・タブを削らない）が `pending` か小文字 hex 40 桁に完全一致しなければ ERROR。ERROR の文は、commit 済みで未 push ならその commit を直すよう案内する。
- D4: WER の要求（DEV_WORKFLOW の Artifact Map・Post-Merge・Done）、`check_new_wer_retired_rules`、WER の template と Skill は無い。
- D5: review-only sub-agent は Final Review に統合され、R2+ の Final Review の本数は `Final Review Minimum`、skip の経路は無い。旧称の 1 文が DEV_WORKFLOW にある。
- D6: Owner Effort Budget は G2 の値と扱い（推奨 A: 介入 6 回・実働 30 分、relay の上限なし、上限に届くときは次の判断と改定を同じ 1 回、L3 の各 round を 1 回、goal-drift signal で即停止）。
- D7: Review Rules に 3 分類は無く、K3 の 4 項目は語を変えずにある。Findings Freeze の原則がある。scope 増減の提案は owner の記録と照合する。Workflow State の停止の段落に「防ぐ失敗を 1 文で」の原則と、それで緩めない安全の境界の列挙がある。
- D8: Design Phase は処置表のとおりに縮まり、Impact Review Lenses の表は template だけにある。Contract Audit は 8 項目。
- D9: Wave に lane 数と同じ source document の上限は無く、予定 file の突合と所有表、生成物の取込み後の再生成がある。train の委任、Ready は先頭だけ、単段 merge（`directoryRenames=false`）、stacked の規則は残る。
- D10: Draft PR に review-only の条件と一過性の dogfood 記述は無く、PR body freshness がある。
- D11: DEV_WORKFLOW に問い合わせの行き先の表は無く、AGENTS への link がある。MANUAL §5.6 は AGENTS を指し、AGENTS の agent 側の項は MANUAL §5.6 を指す。
- D12: code_review に受理 P1/P2 の固定（修正前に red の test、観測できなければ L3、docs は sweep、test を弱めない・足せなければ止まる）と、closure での固定の確認がある。test-design Skill はそれを参照する。
- D13: DEV_WORKFLOW `:68` の裏取りの文、TDM の oracle の規律、operator-ui Skill の DSR、MANUAL の design lane の定義と是正 commit、subagent-review-packet の是正 commit、code_review の確信度の欄がある。
- D14: 独立 review、Plan Commit の固定（PK5）、Human Gate、R4 の必須、古い証拠の拒否（helper）、K3 の 4 項目は変わらない。
- D15: decision-log に D-098 がある。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-HARNESS4-D1 | S1 | AC5、PR4-F1 | Ledger の列 | rg 出力と test 出力 |
| SPEC-WF-HARNESS4-D2 | S2・S3 | AC2、AC4、MU1〜MU3・MU8・MU11・MU13 | 旧形式の受理 | test 出力と mutation の exit |
| SPEC-WF-HARNESS4-D3 | S2・S3 | AC2、AC4、MU4〜MU7・MU10・MU12 | 40 桁の理由 | test 出力と mutation の exit |
| SPEC-WF-HARNESS4-D4 | S2・S3・S4・S9・S11・S12 | AC6、PR4-F10 | 形だけの要求 | rg 出力と test 出力 |
| SPEC-WF-HARNESS4-D5 | S4・S7・S9〜S15 | AC7、PR4-F8 | R4 の review の強制 | rg 出力と test 出力 |
| SPEC-WF-HARNESS4-D6 | S1・S6・S13 | AC10 | G2 | rg 出力、owner の判断 |
| SPEC-WF-HARNESS4-D7 | S5・S7 | AC9、AC16 | 処置表 | rg 出力 |
| SPEC-WF-HARNESS4-D8 | S5・S7 | AC14 | 処置表 | rg 出力 |
| SPEC-WF-HARNESS4-D9 | S8 | AC12 | D-055 の改訂 | rg 出力 |
| SPEC-WF-HARNESS4-D10 | S9 | AC13 | 処置表 | rg 出力 |
| SPEC-WF-HARNESS4-D11 | S6・S13・S14 | AC11 | 導線 | rg 出力 |
| SPEC-WF-HARNESS4-D12 | S10・S12 | AC15 | 試行の集計 | rg 出力 |
| SPEC-WF-HARNESS4-D13 | S4・S10〜S13 | AC16 | 小さな項目 | rg 出力 |
| SPEC-WF-HARNESS4-D14 | 変えないもの | AC1、AC19、AC20、MU9 | 安全の境界 | test 出力 |
| SPEC-WF-HARNESS4-D15 | S16 | AC17 | 番号の予約 | rg 出力 |

## Data Safety

- 変更は tracked の script・test・文書だけ。実 POS / 店舗データ、DB、backup、log、receipt、secret、`.env*` を読まず、commit しない。店の運用・私的な事情を書かない（本 lane では出てこない）。
- local-only: `.local/`（Codex の発注書、helper の capture）、`$TMPDIR`（AC3 の起点の checker の写し、AC18 の mutation の写し）。
- synthetic-only: checker の test の fixture（合成の packet の本文と SHA）。

## Implementation Results

Fill after implementation.

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- 実装: [PR #128](https://github.com/higekiri-w/inventory-system-desktop/pull/128)。S1〜S16 を Writer（Opus 5.5 subagent、発注 153）が 3 commit で実装した（checker と fixture test、文書の縮約と 2 つの削除、D-098）。AC1〜AC20 を満たし、MU1〜MU13 を実注入して red を確かめた。PR5 の所有 file には触れていない。
- Coordinator の裁定: G3 = 認める（2026-09-30。S15 の 3 file は PR5 の所有表にも無く衝突しない）。
- packet の逐語から離れた所: template の「R3/R4 の部分」は見出しを増やさず、`## Contract Ledger` の先頭の 1 行で示した。並走の PR5 の packet を新しい checker で確かめるとき、Matrix への link のため Matrix も一緒に一時的に置いた（発注書の手順の不足。Scope・AC は変えていない）。
- base 同期: origin/main を単段 merge で取り込んだ（衝突なし。main 側は `docs/backlog.md`・`docs/project-memory.md` だけ）。
- 実装の要旨（closeout、2026-09-30）: R3 の契約の追跡を Contract Ledger の 1 表、設計の準備を Design Readiness の 1 節にまとめ、旧 template の packet も通し続ける。止める規則は防ぐ失敗を 1 文で言えることを原則にし、3 分類・review-only の二重の語・Findings Freeze の記録行・WER・wave の lane 数と同じ source document の上限・問い合わせの行き先の重複した表を消した。Owner Effort Budget は介入 6 回・実働 30 分を既定にし、relay の上限を外し、上限に届くときは次の判断と同じ 1 回で諮る。PK4 は `Plan Commit` の書式（`pending` か 40 桁の小文字 hex）を commit の前に止める。受理した P1/P2 は修正前に red の test で固定する（D-098）。
- review・CI・merge: Final Review broad の受理 P2 1 件を Writer が是正し、closure は Codex GPT-6.1 Sol が approve。PR5（#127）の merge の後の base 同期の closure も GPT-6.1 Sol が approve。manual は not-required。merge は PR5 → PR4 の順（owner 2026-09-30 の train 委任）で、helper 経由の squash merge（2026-09-30）。

## Review Response

Fill after review.
- Plan Review round 1（2026-09-29、対象 `9037c760`、互いに独立の 2 本）: fresh Opus 5.5 = reject（P2 2 / P3 6）、Codex GPT-6 Astra = reject（P2 4 / P3 2）。Coordinator が全件を採用し、相談役 Fable 5.1 の起草で是正した。finding ごとの採否と是正の所在（行番号は `bcd066f5`）:
  - Opus #1（P2、`Plan Commit` の書式検査は PK5 が初回値を固定する前に気付けない）: 採用。検出点を commit の前の `--target plan` に直し、commit 済み未 push ならその commit を直す案内を足した（Goal `:124`、Ordinary Operation `:152`、S2 PK4 `:180`、S16 の (10) `:231`、設計判断 3 `:394`、Impact Review Lenses の「Plan Commit の固定」`:458`、Contract Probe 4・11 `:478`・`:485`）。
  - Opus #2（P2、Impact Review Lenses の問いの列が消える、Design inputs の行の理由が事実と違う）: 採用。S1 の Lenses `:176`、処置表の Design inputs `:242`・Lenses `:247`、AC14 `:323`。
  - Opus #3（P3、section 14 の短い SHA の上書き値、archive の短い SHA の網、MU5 の kill）: 採用。S3 `:183`・`:184`、Test Plan の PR4-F5・F7・F12 `:523`・`:525`・`:530`、Matrix の F5・F7・F12 の行。
  - Opus #4（P3、旧語の拾い漏れ: template の Registration の Contract Coverage Ledger、DEV_WORKFLOW の candidate-safety、`:353` の review-only）: 採用。S1 の消す `:173`、S6 `:194`、S7 `:197`、AC5 `:309`、AC10 `:319`。
  - Opus #5（P3、AC7 の baseline の誤り）: 採用。AC7 `:316` を旧称の文の固有 literal に直し baseline を訂正。
  - Opus #6（P3、空の Contract Ledger が WARN だけで通る）: 採用。根拠は PK2 が表の行を見ない（`extract_prose` `scripts/doc-consistency-check.sh:852-861`）ので空の Ledger を止める検査が無いこと。S2 PK1 `:178`、PR4-F11 `:529`、MU11 `:543`、Matrix の FM11。
  - Opus #7（P3、MANUAL §3.5 の停止と G2 = A の食い違い）: 採用。MANUAL の該当行は `:60`。S6 `:194` に「§3.5 の停止が優先」、AC10 `:319`。
  - Opus #8（P3、MANUAL の読ませない場所は `:36`）: 採用。範囲の候補の確認 `:98`、S13 `:224`。
  - Codex F1（P2、末尾空白つきの 40 桁が PK4 を通り helper で拒まれる）: 採用。S2 PK4 `:180`（末尾を削らずに照合）、PR4-F7 `:525`、MU10 `:542`、Contract Probe 10 `:484`、Boundary / Wire Contract `:553`・`:555`、Matrix。
  - Codex F2（P2、section 14 の上書き値）: 採用。S3 `:183`。
  - Codex F3（P2、R2 で Boundary / Wire Contract まで省ける）: 採用。S1 の構成 `:169`（Boundary / Wire Contract を前半へ、`docs/DEV_WORKFLOW.md:62` の条件を保つ）。
  - Codex F4（P2、Draft の前に review を置く古い順序が残る）: 採用。S9 の順序 `:205`、S12 の inventory-implementation `:220`、AC13 `:322`。
  - Codex F5（P3、G2 の根拠の一般化が archive と合わない）: 採用。G2 の実績 `:416` を (a)〜(c) に分け、理由 (2)・(4) `:424` を判断点の積算と事実に合わせた。G1 `:410` も round 1 の事実に合わせた。
  - Codex F6（P3、AC7 の baseline と式）: 採用。Opus #5 と同じ是正（AC7 `:316`）。
- Plan Review round 2（2026-09-30、対象 `bcd066f5`、互いに独立の 2 本）: fresh Opus 5.5 = reject（P2 1 / P3 7）、Codex GPT-6 Astra = reject（P2 1 / P3 1）。Coordinator が全件を採用し、相談役 Fable 5.1 の起草で是正した。finding ごとの採否と是正の所在（節名で示す）:
  - Codex F1（P2）と Opus #1（P2、template の節の並び。R3/R4 の部分を Review Response の後に置くと独立 review の読み方 `awk '/^## Review Response/{exit} {print}'` で消える / Impact Review Lenses を R3/R4 に限ると R2 の lens が消える）: 採用。`S1` の構成を 3 部（全 Risk の本体に Lenses を含む → R3/R4 の部分 = Contract Ledger・Contract Probe・Data Safety → 末尾に Implementation Results・Review Response）に直した。checker・helper・test は節の位置を見ないと現物で確かめ、`S2`・`S3` に追加なし。
  - Codex F2（P3、先行 review の結果・評価が前半に混在）: 採用。遷移記録・`G1`・Contract Probe 10 から結果と評価を外し、本節に集めた。遷移記録の round の行は key を `plan-gate（round 1）`・`plan-gate（round 2）` に分けた（helper の `parse_packet` の重複 key の拒否 `scripts/pr-gate.py:154` を避ける）。template の Review Response 節に「先行 round の結果はこの節にだけ書く」の 1 行を `S1` に足し、`AC5` に anchor。
  - Opus #2（P3、relay の消費が 0 のまま）: 採用。`Owner Effort Budget` の表を消費 2 / 残り 3 に。
  - Opus #3（P3、3 分類の処置の理由）: 採用。処置表、Design Intent Trace の `D7`、`S16` (5) を「防ぐ失敗は Goal Invariant の優先順位と K3 が守る」に。
  - Opus #4（P3、Source Index に decision-log が無い）: 採用。`S5` に Source Index の行の追加、処置表の Design inputs の行に参照。
  - Opus #5（P3、緩めない境界の列挙）: 採用。`S5` に Workflow State の fail-closed と AGENTS の承認境界。
  - Opus #6（P3、Before merge の安全の行）: 採用。処置表の行を「安全（helper の `ci()` が強制）」とし、残す行を 3 行に（`S9` も同じ）。
  - Opus #7（P3、旧い組の空の Trace Matrix の互換を固定する test が無い）: 採用。`PR4-F2b` と `MU13`（Test Plan、Ledger、Trace Matrix、Matrix）。
  - Opus #8（P3、事実の誤り 3 つ）: 採用。`S3` の section 13 の記述、`Risk` の「厳しくする向き 2 つ」、処置表の「11 項目」。
- Plan Review round 3（上限、2026-09-30、対象 `87cf8eca`、互いに独立の 2 本）: fresh Opus 5.5 = approve（P1 / P2 0、P3 5）、Codex GPT-6 Astra = approve（P1 / P2 0、P3 2）。Coordinator が P3 を全件採用し、Writer（Opus 5.5 subagent）が反映した。finding ごとの採否と是正の所在:
  - Opus #1（`scripts/pr-gate.py:295` は closure の Amendments 一致でない）: 採用。Goal の非目的の 3 つ目を broad `:315`・closure `:443` に。
  - Opus #2・Codex #1（`docs/design-system/reference/README.md:32` の Required Design Artifacts への記入指示が残る）: 採用。`S1` に同期先の bullet（記入先を Design Readiness の必要な設計成果物に）、`AC5` に一致なしの `rg`、所有表の後の「表の外に求める file」に追加。
  - Opus #3（DEV_WORKFLOW `:63-64` の Contract Probe の行き先が処置表に無い）: 採用。処置表に Plan Packet Rules の行（keep 2 行、具体物を調べる文を残す）、`S4` に同じ条件。
  - Opus #4（Final Reviewer 行の Codex の model の理由）: 採用。Workflow State の Final Reviewer 行に「難所: merge gate の合否を変える」。
  - Opus #5（AC20 の座組表の比較相手）: 採用。`AC20` を merge-base の版との比較に。
  - Codex #2（`workflow-git-checks.test.sh:357` の説明）: 採用。Non-scope の該当行を「doc-consistency 側は撤去済み Phase の拒否、workflow-git 側（T-G5）は旧 commit 件名の受理」に。
Final Review broad（2026-09-30、対象 `473d90cf`）: Fable 5.1 = approve（P1 0 / P2 0 / P3 2、F-1・F-2）、Codex GPT-6 Astra（発注 155）= reject（P1 0 / P2 1 / P3 1、A1・A2）。broad の 2 本は是正の push の前に `473d90cf` で record した。Coordinator が現物で裏取りした。是正は Writer（Opus 5.5 subagent、発注 158）。

- A1（P2、`scripts/doc-consistency-check.sh:1292` の `[[:space:]]` が UTF-8 の locale で U+3000 を落とし、helper〈`scripts/pr-gate.py:152` の `[ \t]*`〉が拒む `Plan Commit` を checker が通す。D3 の「commit の前に止める」が未達）: 採用。Coordinator が `C.UTF-8` で削れ `C` で削れないことを再現。前置で落とす文字を ASCII の空白とタブに限り（`e48400b1`）、`LC_ALL=C.UTF-8` で走る PR4-F7b（空白・タブは通り、U+3000 は PK4 の ERROR）を足した。修正前 red → 修正後 green（呼ぶ側の locale が C でも同じ）。
- A2（P3、D-098 の「4 表」が base の template の実物〈追跡表 3 つと Spec Contract の箇条書き〉と合わない）: 採用。D-098 の Why・Alternatives を「契約を書く 4 つの節」にした（`9b54ae46`）。本 packet の「4 表」（計画時の呼び方）は plan の記録なので書き換えない。
- F-1（P3、PK1・PK3 の Ledger の経路の見出しの判定が `###` の小見出しにも当たる）: 見送り。既存の節検出と同じ規約で、誤れば ERROR（fail-closed）。closeout で backlog へ。
- F-2（P3、DEV_WORKFLOW の Risk Tiers の R2 の行が Final Review に触れない）: 見送り。base から同じで本 PR 起因でない。closeout で backlog へ。
- 是正の検証（Writer の報告、`9b54ae46`）: AC1〜AC20 を逐語で再実行し全件期待どおり。`Plan Commit` の書式に関わる MU4・MU5・MU6・MU10・MU12 と新規の MU-F1（前置を `[[:space:]]` に戻す）を注入して red。MU12 を Test Plan の字面どおりに入れると PR4-F12 より前の section で red になるため、狭くした版で PR4-F12 の red を確かめた。`bash scripts/local-ci.sh full` PASS（`MERGE_EVIDENCE_VALID=true`）。Coordinator が test の見出し comment の空白の脱落（PR4-F8）を直した。

- Findings Freeze: frozen after Final Review broad（Double Audit の 2 本、2026-09-30）; post-freeze exceptions: none.

- Closeout（2026-09-30）: 是正の後の closure は Codex GPT-6.1 Sol = approve（取り直しの 162。160 は発注書の矛盾で数えなかった）、PR5（#127）の merge の後の base 同期の closure も GPT-6.1 Sol = approve（165）。見送った P3 の F-1（PK1・PK3 の Ledger の経路の見出しの判定が `###` の小見出しにも当たる）と F-2（Risk Tiers の R2 の行が Final Review に触れない）、closure 162 の観察（PK4 の field 検査が並列実行で 1 回だけ失敗し、逐次で PASS。原因は分かっていない）は `docs/backlog.md` の「workflow / test / lint / docs」へ置いた。
