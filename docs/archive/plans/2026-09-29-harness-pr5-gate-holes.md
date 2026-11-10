# Plan Packet: ハーネス刷新 PR5 gate の穴を塞ぐ（R3）

2026-09-29 起草。出典は owner 決定 2026-09-24（ハーネス刷新は 5 本。PR5 = classifier / helper の穴・`.claude/agents`）、2026-09-28（規則とハーネスの改善を最優先。止める規則は起きている失敗を 1 文で言えること〈Coordinator の言い換え〉。安全の境界は残す）、2026-09-29（PR4・PR5 の起票承認。PR5 に local-ci の `node_modules` symlink 検出と、pre-push の merge 済み PR 検出・未 stage の WARN を入れる）。背景の事実は 2026-09-24 のハーネス監査（§0-8 helper の自己検査の穴、§3.Q V2・V9 classifier の穴と helper の実行場所、§8-9）と 2026-09-28 の教訓の棚卸し（local-only の報告。要点は下の「現状の事実」に書き下す）。

本 lane は wave に属さない単独の lane（ハーネス刷新の最後の 2 本の片方。PR4〈手続きの軽量化〉と並走し、file の所有は下の「PR4 との file の所有」で分ける）。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R3
- Plan Commit: 0c6489ab9c8bd3a50e0dbed5513b1d610100530c
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（fresh context、Coordinator が指定する worktree で作業）
- Plan Reviewer: fresh Opus 5.5 subagent（fork でない）+ Codex（GPT-6 Astra。merge gate・helper・classifier・hook の合否を変える変更）。互いに独立で Writer と別 context
- Final Reviewer: Fable 5.1（Claude 側、R3）+ Codex（GPT-6 Astra）。互いに独立で Writer・Plan Reviewer と別 context（Double Audit）。後の reviewer に先の結果を見せない
- Final Review Minimum: 2
- Human Gate: ready,merge
- Branch: agent/harness-pr5-gate-holes

manual なし: 製品の runtime・画面・配布物を変えず、Windows native L3 の対象画面が無い。helper・classifier・hook・local-ci の振舞いは合成 fixture の自動 test で確かめる。r4 なし: data・DB・破壊的 git 操作を含まず、変更は revert で戻せる。Final Review Minimum 2: `scripts/**`・`scripts/tests/**`・`.claude/settings.json` は `scripts/ci/classify-changes.sh:55` の実行制御（full）、`docs/agent-guidance/**`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md` は `:59` の policy docs で、どちらも workflow=true になり helper は Minimum 2 を要求する（`scripts/pr-gate.py:270-272`）。owner の作業が要る場面（`.claude/agents/**`・`.claude/settings.json` の書込みの代行と、merge 後の本体の checkout の同期）は下の「判断点」の G1 に書く。

遷移記録（append-only）:

- kickoff → spec-check → plan-draft → plan-gate（2026-09-29、起草役 = Opus 5.5 subagent、本 commit、plan-first）: Risk R3（下記 Risk）。spec-check で、既存の正本（`docs/agent-guidance/merge-evidence.md` の MG-D4 の分類表・「Helperの境界」・「実行手順」、`docs/ci.md` の Classifier / Pre-push Contract、`docs/AGENT_OPERATING_MANUAL.md` の `## 座組`）が本 lane の判定の置き場所として足りると確かめた（新しい設計文書は作らない）。設計判断（D1〜D10）は本 packet の Spec Contract に置き、正本への反映は実装の S9〜S12 で行う（workflow の正本は本 lane が書き換える文書そのもの。PR2・PR3・並走の摩擦の lane と同じ形）。design phase は使わない（skip の条件は Design Readiness）。Plan Review へ。
- plan-gate: Plan Review round 1（fresh Opus 5.5 reject P2 2 / P3 5、Codex GPT-6 Astra reject P1 1 / P2 5）→ 全件採用し是正（相談役 Fable 5.1 の起草、2026-09-29）
- plan-gate（round 2）: Plan Review round 2 の是正（相談役 Fable 5.1 の起草、2026-09-30。内訳は Review Response）
- plan-gate（round 3）: Plan Review round 3（上限、対象 3dbdf369。fresh Opus 5.5 reject P2 1 / P3 7、Codex GPT-6 Astra reject P2 1）→ round 天井の disposition「同型指摘の一括是正」（相談役 Fable 5.1 の起草、2026-09-30。内訳は Review Response）。Coordinator が現物で確かめて plan-approved を owner に諮り、reviewer の再確認は Final Review に回す
- plan-gate → plan-approved（2026-09-30、Coordinator、本 commit）: 上限の Plan Review round 3（対象 `3dbdf369`）の後、round 天井の disposition「同型指摘の一括是正」（`0c6489ab`）を Coordinator が現物で確かめた（撤回した前提の sweep 0 件、新しい文の件数、`doc-consistency-check.sh --target plan` exit 0〈WARN 1 = PK3〉、`check-workflow-git.sh` exit 0、helper の `parse_packet` が通る）。独立 reviewer の再確認は Final Review（Fable 5.1 + Codex）で行う。owner 承認（2026-09-30、この change での介入 2・3 回目 = plan-approved と G1 の確認）: G1 = (a)（`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=1` は本 lane で入れる、`disableSkillShellExecution` は backlog の follow-up）、残すリスク = A。Plan Commit = `0c6489ab`（承認した計画の最後の commit）
- plan-approved → implementing（2026-09-30、Coordinator、state-only）: Writer（Opus 5.5 subagent）へ実装を発注する。Writer の開始 HEAD は本 commit。

## Owner Effort Budget

- 介入回数上限: 7（既定 3 から。理由: `.claude/agents/**` と `.claude/settings.json` は本体の checkout が sandbox から書けず、merge 後に owner が sandbox の外で本体を同期し Claude Code を再起動する 1 回が要る〈G1〉。plan-approved に owner の判断点が 1 つある。予備 2 = Writer が worktree の `.claude/agents/**` を書けないときの owner の代行 1〈G1 (a)〉と、Ready の後に PR4 が merge されて base 同期の後に再 Ready を諮る 1）
- 実働時間上限: 15分（owner の作業は判断点の回答、Ready・merge の指示、merge 後の本体の同期 1 回）
- relay 往復上限: 6（既定 2 から。理由: Plan Review の Codex 3 round、Final Review の Codex broad 1、Final の是正の後の Codex closure 1。予備 1 = PR4 が先に merge されて base 同期した後の Codex closure。是正の closure と base 同期の closure は両方起こり得る）
- Plan Review round 天井: 3（既定 3）

| 種別 | 上限 | 消費（2026-09-30 の plan-approved 時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 7 | 3: 起票承認と範囲の判断（2026-09-29、同じ問いの 1 回）、plan-approved（2026-09-30）、G1 の確認（2026-09-30。plan-approved と別の decision point、`docs/DEV_WORKFLOW.md:267`） | 3: Ready 1、merge 1、merge 後の本体の同期と Claude Code の再起動 1 | 1: G1 (a) の owner の代行 1。PR4 の merge の後の再 Ready が要るときは、その問い合わせと同じ 1 回で上限の改定を諮る | 7 = 3 + 3 + 1 |
| relay | 6 | 3: Plan Review の Codex round 1・round 2・round 3 | 2: Final Review の Codex broad 1、Final の是正の後の Codex closure 1 | 1: PR4 の merge の後の base 同期の Codex closure | 6 = 3 + 2 + 1 |

改定（2026-09-30、owner の承認）: 介入の上限 7 → 9、relay の上限 6 → 8。

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

相談役への相談は[AGENT_OPERATING_MANUAL](../../AGENT_OPERATING_MANUAL.md)の座組に従う。本節の2欄は`none`のままにする。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3「merge gate changes」に当たる。本 lane は次の合否を変える: classifier の分類（helper の Final Review Minimum と R0 / R1 の経路の可否、hosted の job の選択。`scripts/ci/classify-changes.sh`）、helper が走ってよい条件と review の record の受理（`scripts/pr-gate.py`）、pre-push の push の可否（`scripts/pre-push.sh`）、local-ci full の合否（`scripts/local-ci.sh`）。kickoff の問い「この変更でどれかの required gate の green / red が変わるか」の答えは「変わる」で、R3・Minimum 2。どれも「変更前に green だった入力が red になる」向き（締める側）で、緩める向きの変更は無い（D10）。R4 には当たらない（data・破壊的操作なし）。

Review-only skipped because: R3 の review-only sub-agent は、Final Review の Double Audit の broad（Fable 5.1 と Codex の 2 本、どちらも Contract Audit を正本から行う）が兼ねる（`docs/DEV_WORKFLOW.md` Review Rules・Draft PR Checkpoint）。

## 現状の事実（main `7ac96d9e`、2026-09-29 に起草役が確かめた）

1. classifier の穴: `scripts/ci/classify-changes.sh:59` の policy docs の一覧に `.claude/agents/*` と `docs/quality/review-checklist.md` が無い。どちらも `:67` の `*.md` / `docs/*` で一般 docs（docs=true・workflow=false）になる（AC1 の baseline）。`.claude/agents/*.md` は subagent の model・effort・tools・`hooks`・`permissionMode` を決める設定で、review の座組を実質的に変えられるのに、helper の Minimum 2 も workflow 回帰も掛からず、R0 / R1 の経路でも通る（`scripts/pr-gate.py:259` は `workflow` と `rust` の両方が true のときだけ拒む）。監査 V2。
2. `.gitignore:116` が `.claude/agents` を除外している（`git check-ignore -v .claude/agents/reviewer.md` → `.gitignore:116:.claude/agents`）。`:100-107` の comment のとおり、Claude Code の sandbox の protected paths が本体の checkout の `.claude/agents` を mount で塞ぎ dir でなくする（見え方は session で違う）ための雑音対策で、tracked の定義を置けない原因にもなっている。
3. `scripts/local-ci.sh:224` は full のとき無条件に `npm ci` を実行する。worktree の `node_modules` が本体の `node_modules` への symlink だと、`npm ci` が symlink 先（本体）の中身を消す。事故は 3 回（2026-09 に 2 回と 09-25。09-25 は並走中の reviewer も止まった）。教訓の棚卸しの優先度は高。
4. `scripts/pre-push.sh:98` は `gh pr list --head <branch> --state open` だけを見る。merge 済みの PR の branch へ追加で push すると素通りし、commit が PR に載らず孤児になる（2 回）。
5. closeout の `git mv` の後の編集を add し忘れ、rename だけの commit を push した事故が 5 回ある（`git status` の ` M`・`RM`）。正当な WIP の push もあるので止めずに WARN にとどめる（教訓の棚卸しの判断）。
6. helper の自己検査の穴: `scripts/pr-gate.py:219-221` は review 要件の判定に base 側の classifier を GitHub から取得するが、helper 本体は実行した checkout の file。`pr-gate.py` を変える PR は、変更後の helper で自分を検査できる。PR1（#97）と並走の摩擦の lane（#123）は「origin/main の helper の写しを使う」手順を発注書と packet に固定して回避した（`docs/archive/plans/2026-09-24-harness-legacy-and-execution-mode-removal.md` の D9）。手順を忘れれば通る。監査 §0-8・V9。helper は repo root を `git rev-parse --show-toplevel`（`:196`）で決め、自分の file 位置を使わない（`rg -c '__file__' scripts/pr-gate.py` → 0 件、exit 1）。
7. broad の record の順序: `pr-gate.py record --review-stage broad`（`:450-454`）は capture の head を broad の head にする。監査した head を受け取らないため、是正を push した後の head の capture で broad を record すると、監査していない是正を broad が覆ったことになり closure が要らなくなる。PR #113 で起き、owner に 1 回余分に諮った。closure（`:455-459`）も同じ形。
8. closure の record は audit 1 つ（`:458-459`、RecordV1 の `Closure.audit`）。同じ head に 2 本の closure run がある場合にどちらを record するかが文書に無い（PR2・PR3 の Final Review の follow-up (1)、`docs/backlog.md` の該当項目。file の所有に合わせて PR4 から本 lane へ移す〈Coordinator 裁定〉）。
9. 座組の注記: `docs/AGENT_OPERATING_MANUAL.md:68`（Writer 行）は「Agent tool から直接起動する subagent は session の値（high）を継承する。Workflow から起動する subagent は effort を指定できる。tracked の定義 file（`.claude/agents/**`）は `.gitignore:116` の除外の解除と classifier の穴と一緒に PR5」、`:69`（Plan Reviewer 行）は「実効値の注意は Writer 行と同じ」。座組表の Writer・Plan Reviewer の effort medium は、今は Agent tool の起動では効かない。
10. subagent の入れ子: `docs/DEV_WORKFLOW.md:254` の「Max delegation depth is 1」は文書だけで、Claude Code の既定は 3 層（公式、Contract Probe P1）。
11. 本体の checkout（Coordinator の main session の cwd）の HEAD は `253eef06` で、origin/main（`7ac96d9e`）より古い。project の subagent 定義と `.claude/settings.json` は session の cwd から読まれる（公式、P1・P3）ので、tracked の定義を足しても本体の checkout を同期するまで Coordinator の session には効かない。本体の `.claude/settings.json` は sandbox から書けず（`test -w` が偽）、`.claude/agents` は上の 2 の mount で塞がれる。

## Goal

Goal Invariant:

### 最小完了条件

- `.claude/agents/**` と `docs/quality/review-checklist.md` を変える PR に、hosted が workflow 回帰を、helper が packet 付きなら Minimum 2 を要求する（`scripts/pr-gate.py:270-272`）。`.claude/agents/**` を変える PR は R0 / R1 の経路を通れない（`:259`）。`docs/quality/review-checklist.md` だけを R1 で申告した packet 無しの経路は、他の policy docs と同じく残る（Residual Test Gaps）。
- helper は、自分の file が PR の base の `scripts/pr-gate.py` と違うときに何もせず止まり、base の版を使う command を示す。この停止は D2 を載せた helper（本 lane の merge 後の main から分岐した branch と、main を取り込んだ branch）で、手順を忘れても効く。本 lane の merge より前に分岐した lane の旧い helper は止まらず、移行手順で塞ぐ（「本 lane 自身の検査と merge」の 7、Residual Test Gaps）。
- review の record は監査した head を受け取り、capture の head と違えば止まる。是正の push の後に capture の head で broad を record して closure を飛ばす経路は、operator が監査した head（reviewer の報告の commit か、発注した review packet の `対象差分と内容commit` 欄）を渡す限り塞がる。現在の HEAD を渡せば通る（reviewed head は入力で、写し元の規律は merge-evidence の文に依る。Matrix の Residual Test Gaps）。
- local-ci full は `node_modules` が symlink のとき、どの gate よりも前に非 0 で止まり、symlink 先を変えない。
- pre-push は merge 済みの PR の branch への push を止め、「main から新しい branch」を示す。push する file に未 stage の変更が残れば WARN を出して push を続ける。
- Coordinator が Agent tool で起動する Writer・reviewer が、tracked の定義（`.claude/agents/{writer,reviewer}.md`）から座組表の effort（medium）で動き、subagent の入れ子は設定（`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=1`）で止まる（どちらも本体の checkout の同期と Claude Code の再起動の後）。座組表の注記が実態と合う。

### 失敗定義

- 上のどれかが残る（例: `.claude/agents/*.md` の変更が R1 で通る、`pr-gate.py` を変えた PR が自分の helper で record できる、監査していない head の broad が record できる、symlink の `node_modules` のまま `npm ci` が走る、merge 済みの branch への push が通る）。
- 新しい停止が正当な作業を止める: `local-ci changed` や symlink でない worktree の full、open の PR の branch や PR の無い branch への push、未 stage の変更を残した WIP の push、`pr-gate.py` を変えない PR の helper の操作（base と同じ版の helper）。
- 既存の安全の境界が弱まる: 古い証拠の拒否（record の head/base の照合、capture と server の一致、closure の要求）、独立 review の本数（Double Audit の下限、main 側の classifier）、Human Gate（approved snapshot の照合、Ready・merge の owner 指示）、fail-closed の既定（照会失敗・未知の値で止まる）。

### 非目的

- PR4 の範囲（下の「PR4 との file の所有」の PR4 の行）。
- strict（MG-D1）の撤去、衝突を解いた版の manual の再利用の条件（owner 2026-09-28「今回は緩めない」）。
- 過去の decision-log の本文と archive の書換え。
- 生成器（`src-tauri/src/bin/generate_*`、bindings の export）の classifier の扱いの変更（D10 で入れないと決める）。
- helper が base の版へ自動で切り替えて実行すること（D2 の棄却案）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、本 lane の merge 後に 1 本の lane（X）を worktree の作成から merge まで通す操作列を書く。各行の「旧:」は本 lane の前の振舞い。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 本 lane が main にあり、owner が本体の checkout を sandbox の外で origin/main に同期し、Claude Code の main session を再起動した（G1） | Coordinator が Agent tool で `subagent_type: writer`（または `reviewer`）を起動する | subagent が effort medium で動き、`/tasks` の行に model と effort が出る（旧: session の high を継承）。subagent が Agent tool を持たない（depth 1） | Coordinator が run 報告に実効の model・effort を記録する | P3（本体の同期と再起動までは効かない。公式 sub-agents「Write subagent files」: watcher は session 開始時に存在した dir だけを見る）。再起動後の最初の run で `/tasks` を確かめる（AC7） |
| X の worktree を origin/main から作り、`node_modules` を本体への symlink にした | `bash scripts/local-ci.sh changed` | 従来どおり（changed は `npm ci` を実行しない） | exit 0 | なし |
| 同上 | `bash scripts/local-ci.sh full` | どの gate よりも前に `ERROR=node_modules is a symlink …` で exit 非 0。本体の `node_modules` は変わらない（旧: `npm ci` が本体を空にした） | symlink を外して `npm ci --ignore-scripts` で実 dir を作り、full をやり直す | なし |
| X の Draft PR（open） | commit して `git push origin <branch>`。push する file の 1 つに未 stage の編集が残っている | `[pre-push] WARN: unstaged changes in pushed files: …` を出し、push は続く（旧: 何も出ない） | 意図した commit なら続ける。add し忘れなら add・commit して push し直す | なし |
| X の broad 監査が head H1 で終わり、是正が要る | capture（H1）→ `record --kind review --review-stage broad --reviewed-head H1 …` → 是正を push（H2）→ capture（H2）→ closure の監査 → `record --review-stage closure --reviewed-head H2 …` | broad が H1、closure が H2 に結び付く | 現在版の closure pass | なし |
| 同上で順序を誤る（是正か base 同期の push が broad の record より先に来た） | head を変える push（H2）の後に capture（H2）→ `record --review-stage broad --reviewed-head H1` | `reviewed head differs from capture head` で exit 1。何も書かない（旧: H2 の broad として記録され closure が要らなくなった） | H2 で broad をやり直す（H1 の broad は record できない） | なし |
| 同じ head に closure run が 2 本（例: Claude 側 2 本） | 後に完了判定を出した run を `--pass-model` / `--run-ref` にし、両方の run の証跡を `--evidence` に並べて 1 回 record する | 1 つの closure audit に両方の証跡が残る | 現在版の closure pass | なし（record の wire は変えない） |
| X が `pr-gate.py` を変えない。X の branch の `pr-gate.py` が PR の base と同じ | `python3 scripts/pr-gate.py` の status / capture / record / ready / merge（`--pr N --packet …`） | 従来どおり | 各操作の従来の条件 | P5（base の `pr-gate.py` を取得できる） |
| main の `pr-gate.py` が X の分岐の後に変わった、または X 自身が `pr-gate.py` を変える | 同上を X の checkout の helper で実行 | `helper differs from base …` と、`git fetch origin && git show <base SHA>:scripts/pr-gate.py > "${TMPDIR:-/tmp}/pr-gate-base.py"` で base の版を使う command を示して exit 1。何も書かない（旧: 変更後の helper で自分を検査できた） | base の版の写しで実行し直す（`pr-gate.py` を変えない PR は origin/main を単段 merge してもよい） | P5 |
| X が merge 済み。同じ branch に追加の是正を commit した | `git push origin <branch>` | `PR #N for <branch> is already merged; create a new branch from main` で push が止まる（旧: 素通りして commit が孤児になった） | origin/main から新しい branch を切り、新しい PR にする | P6 |
| PR が `.claude/agents/writer.md` を変える | helper `status --risk R1 --manual not-required` | `CI execution change requires R3 packet` で止まる（旧: R1 で review 無しに通った）。packet 付きなら Minimum 2 | R3 の packet で進める | なし |
| PR が `.agents/skills/foo/.claude-plugin/plugin.json`（skill folder を plugin にする manifest）を足す | helper `status --risk R1 --manual not-required`、hosted の docs job の hook test | helper は `CI execution change requires R3 packet` で止まる（旧: docs＋workflow の policy で R1 を通った）。hook test は manifest の存在で red | R3 の packet と hook test の変更（どちらも full）で進める | なし |
| PR が `docs/quality/review-checklist.md` だけを変える | hosted CI、helper | docs と workflow 回帰の job が走り、packet 付きなら Minimum 2（旧: docs だけ） | 従来の条件 | なし |

この列で、現状の事実 1・3〜7・9・10 の穴が塞がる。11 は本体の同期と再起動（G1）の後に効く。

### 本 lane 自身の検査と merge（自分の変更で自分の gate を緩めない）

本 lane は `pr-gate.py` を変えるため、先例（PR1 の D9、並走の摩擦の lane）と同じく origin/main の helper で自分を検査する。

1. Coordinator が本 lane の worktree で `git fetch origin` の後、`git show origin/main:scripts/pr-gate.py > "$TMPDIR/pr-gate-main.py"` を作り、`git hash-object "$TMPDIR/pr-gate-main.py"` と `git rev-parse origin/main:scripts/pr-gate.py` の一致を確かめる。
2. cwd を本 lane の worktree にしたまま `python3 "$TMPDIR/pr-gate-main.py" status|capture|record|ready|merge --pr N --packet docs/plans/2026-09-29-harness-pr5-gate-holes.md` を使う（helper は cwd の git で root と HEAD を決める。現状の事実 6）。本 lane の新しい helper で自分の PR を record・Ready・merge しない。
3. 旧い helper には `--reviewed-head` が無い。本 lane の broad は head を変える push（是正、origin/main の単段 merge による base 同期）の前に、監査した head の capture で record し、closure は是正の後の capture で record する（D8 の順序を手で守る。base 同期が broad の record より先に来ると、その head の broad は record できず新しい head で broad をやり直す）。
4. base が進んだら（origin/main の単段 merge の後）1 をやり直す。`ready` と `merge` の直前には、base が進んでいなくても 1 の blob 照合をやり直す。
5. origin/main の `scripts/doc-consistency-check.sh` と `scripts/check-workflow-git.sh` も `$TMPDIR` に取り出し、本 lane の worktree の root で実行する（checker は既定 mode と `--target plan` の両方、PK5 は `WORKFLOW_BASE_SHA` 付き）。hosted CI は PR の head の script（新しい classifier・pre-push 等）で走り、その妥当性は Double Audit が見る。
6. 本 lane の新しい helper の `status` も、Phase が implementing・PR が Draft・broad の record がある状態で本 lane の worktree で実行し、D2 の自己照合で exit 1（`helper differs from base`）になることを確かめる（AC2 の実物の確認。本 lane は `pr-gate.py` を変えるので必ず違う）。
7. 本 lane の merge の後、D2 の自動停止は D2 を載せた helper にだけ効く（本 lane の merge 後の main から分岐した lane と、main を取り込んだ lane。`pr-gate.py` を変える後続の PR は手順を忘れても自分の helper が止まる）。本 lane の merge より前に分岐した lane の helper は旧い版で、自己照合も `--reviewed-head` も無い（PR4 の `scripts/pr-gate.py` は origin/main と同じ blob `95c5f91c`、merge-base `7ac96d9e`。open の PR は他に #81 だけ）。その lane は本 lane の merge の後、helper の最初の操作の前に origin/main を単段 merge する（strict で元々必要）か、`git show origin/main:scripts/pr-gate.py > "$TMPDIR/pr-gate-base.py"` の写しを使う。取り込むまでのその lane の record は `--reviewed-head` 無しで通る。この移行は手順で、忘れると抜ける（機械的な停止は無い。Residual Test Gaps と D-099）。Coordinator は本 lane の merge の直後に `gh pr list --state open --json number,headRefName` で open の PR を列挙し、各 lane の次の発注書にこの手順を書く。本 lane の closeout（R0）は main の新しい helper で行う。

## Scope

行番号は main `7ac96d9e` のもの。各 file の変更は該当行・節に限る。

- S1 `scripts/ci/classify-changes.sh`（D1）
  - `:55` の実行制御（full）の一覧に `.claude/agents/*` と `*/.claude-plugin/*`（skill folder を plugin にする manifest の dir。`.agents/skills/foo/.claude-plugin/plugin.json` と `.claude/skills/foo/.claude-plugin/plugin.json` の両方に当たる。公式 skills「Skill folder as a plugin」、2026-09-30 確認）を足す。plugin の中身になる path（`hooks/hooks.json`・`agents/*.md`・`.mcp.json`・`scripts/*`）は full にしない: manifest が無ければ読まれず（公式 plugins/loading の `@skills-dir`）、path の列挙は hooks.json が指す `${CLAUDE_PLUGIN_ROOT}/scripts/*` のように漏れる。manifest が tracked に無いことは S11 の hook test が求める。
  - `:59` の policy docs（docs＋workflow）の一覧に `docs/quality/review-checklist.md` を足す。
  - 他の分類・出力 key・fallback は変えない。`src-tauri/src/bin/generate_*` は変えない（D10）。
- S2 `scripts/tests/classify-changes.test.sh`（D1）
  - `.claude/agents/reviewer.md`・`.claude/agents/sub/writer.md`（階層の下も同じ分類）・`.agents/skills/example/.claude-plugin/plugin.json`・`.claude/skills/example/.claude-plugin/plugin.json` が、既存の実行制御の loop（`:92-99`）と同じ形で rust / rust_drift / frontend / docs / env / generated / traceability / workflow = true・unknown = false。`.agents/skills/example/hooks/hooks.json`（plugin の中身。manifest 無しでは読まれない）は既存の policy の loop（`:46-52`）に足し、docs = true・workflow = true・rust = false のまま（中身を full にしない判断の固定）。
  - `docs/quality/review-checklist.md` が既存の policy の loop（`:46-52`）と同じ形で docs = true・workflow = true・rust = false・frontend = false。
  - 無関係の docs（`docs/backlog.md`、`docs/quality/other.md`）が docs = true・workflow = false（新しい負例）。
- S3 `scripts/pr-gate.py` の自己照合（D2）
  - `Gate.requirements()` の classifier の取得（`:220`）と同じ base（`pr['base']['sha']`）で `scripts/pr-gate.py` を contents API から取得し、base64 を復号した bytes と、実行中の file（`Path(__file__)`）の bytes を比べる。違えば GateError（exit 1）で `helper differs from base <base SHA>; run: git fetch origin && git show <base SHA>:scripts/pr-gate.py > "${TMPDIR:-/tmp}/pr-gate-base.py" && python3 "${TMPDIR:-/tmp}/pr-gate-base.py" …` を出す（message は literal。f-string なら `${{TMPDIR:-/tmp}}` と escape する。`$TMPDIR` が無い shell では `/tmp` に落ち、base SHA が local に無くても fetch で取れる）。
  - 置く場所は `requirements()` の先頭（PR の差分の取得より前）。status・capture・record・ready・merge はすべて `snapshot()` → `requirements()` を通るので、1 か所で全操作に掛かる。
  - 取得の失敗は既存の `contents()` のとおり exit 2。比較は bytes の完全一致（改行・末尾の空白の正規化をしない）。
- S4 `scripts/pr-gate.py` の `--reviewed-head`（D8）
  - `main()` に `--reviewed-head` を足す。`record --kind review`（broad・closure の両方）で必須（無ければ exit 2、既存の `review needs stage/model/run-ref/evidence` と同じ扱い）。full SHA の形を `sha()` で確かめる。
  - `snap['head']` と違えば GateError（exit 1）で `reviewed head differs from capture head; record the broad before pushing a fix, or audit the current head`。書込みの前に判定する。
  - manual / r4 の record には要求しない。RecordV1 の wire は変えない（reviewed head は capture の head と同じなので record に新しい field を足さない）。
- S5 `scripts/tests/pr-gate.test.py`（D2・D8）
  - CLI の fixture の `contents` に `scripts/pr-gate.py` = 実物（`ROOT/scripts/pr-gate.py` の text）を足す（既存の test が自己照合を通るように）。`args()` の既定に `reviewed_head=H` を足し、既存の CLI の review の record に `--reviewed-head <head>` を足す。
  - 新しい class `HelperVersion`: base の `scripts/pr-gate.py` が実物の末尾に改行を 1 byte 足した fixture（`text + '\n'`。`x` を足す fixture では `rstrip` 等の正規化の mutation が green のままになるため、不一致の fixture はすべてこの形）で、status・capture・record・ready・merge のそれぞれが exit 1、message に `helper differs from base` と base SHA、fake gh の呼出しに `pr ready` / `pr merge` / comment の POST・PATCH が無い。同じ bytes なら status の blockers が従来どおり。base の取得が HTTP 失敗なら exit 2。base と head で本文が違う test `test_compares_base_not_head`（`test_rules_keep_explicit_desired_defaults`〈`:482-500`〉の形: `pr.base.sha` を head と別の `B` にし、`snapshots[B]['scripts/pr-gate.py']` に base の本文、`contents['scripts/pr-gate.py']` に head 側の本文を置く。fake の `gh` は ref の snapshot に無い path を `contents` から返す〈`:226`〉ので、head 側は `contents` になる）: base = 実物・head = 実物 + 末尾の改行 → `status` の blockers が空。base = 実物 + 末尾の改行・head = 実物 → exit 1、message に `helper differs from base` と `B`。この test は `status` だけを叩く（`capture` は base の commit の実在を `git cat-file` で求める〈`pr-gate.py:296-298`〉ので、合成の `B` では通らない）。
  - 新しい class `ReviewedHead`: broad・closure の record で `--reviewed-head` が capture の head と違えば exit 1 で comment を書かない（fake の `comments` が不変）。無ければ exit 2。一致すれば従来どおり record される（`test_packet_double_audit_cli` の形）。
- S6 `scripts/local-ci.sh`（D5）
  - 分類の log（`:187-191`）の直後、最初の gate（`:193` の docs）の前に、`MODE == full` かつ `-L "$REPO_ROOT/node_modules"` なら `log "ERROR=node_modules is a symlink; npm ci would empty its target. Run: unlink node_modules (no trailing slash), then npm ci --ignore-scripts"` と `finish FAIL 1`。`rm -rf node_modules/`（末尾 /）は symlink 先の中身を消すので案内しない。
  - changed では検査しない（`npm ci` を実行しないため。worktree で symlink を使う正当な運用を止めない）。`:224` の `npm ci` は変えない。
- S7 `scripts/tests/local-ci.test.sh`（D5）
  - fixture の repo に `target/marker` を持つ dir と、それを指す `node_modules` の symlink を置き、PATH の先頭に呼出しを記録する stub の `npm`・`cargo` を置く。`local-ci.sh full` が exit 非 0、evidence に `ERROR=node_modules is a symlink`・`RESULT=FAIL`、`GATE=` の行が 1 つも無い、stub の呼出し記録が空、`target/marker` が残る。
  - 同じ symlink で `local-ci.sh changed` は exit 0 で ERROR が無い。
  - `node_modules` を実 dir にし、fixture の repo に空の `src-tauri/` を作り（full は `--all`〈`local-ci.sh:177`〉で rust=true になり、`run_required rust-fmt "$REPO_ROOT/src-tauri"`〈`:208`〉が `cd "$workdir"`〈`:110`〉で落ちて `frontend-install` に届かないため）、full の他の gate を stub（`scripts/tests/run-workflow-tests.sh`・`scripts/check-env-safety.sh`・`npm`・`cargo`）で通すと、ERROR が無く `GATE=frontend-install` と `npm ci` の呼出しがある。
  - `:13` の `grep -Fq 'run_required frontend-install "$REPO_ROOT" npm ci'` は残す。
- S8 `scripts/pre-push.sh`（D6・D7）
  - D6: `:98` の open の照会が空のとき、同じ branch で `gh pr list --head "$branch" --state merged --json number --jq 'if length == 0 then empty else .[0].number end'` を照会する。番号が返れば `[pre-push] PR #<N> for <branch> is already merged; create a new branch from main with a new name (a reused name stays blocked).` を出して `fail_gate merged-pr`。番号が空（open も merged も無い）なら通す。照会の失敗は `fail_gate ready-state-lookup`（既存と同じ fail-closed）。open の PR があれば merged を照会しない（既存の open PR を止めないための安全弁。open の PR は push の後にしか作れないので、merge 済みの名前を再利用した最初の push は必ず止まる。名前の再利用は避ける）。
  - D7: Ready の確認の後、分類の前に、push する各 ref のうち `local_oid` が HEAD と同じものについて、push 範囲（分類と同じ起点: remote_oid、新しい ref なら origin/main（無ければ main）との merge-base）の変更 file と `git diff --name-only`（index と作業 tree の差 = 未 stage）の共通部分を求め、空でなければ `[pre-push] WARN: unstaged changes in pushed files: <path…>` を stderr に出す。exit code・`quality-check.log` の outcome は変えない。起点が決まらない ref と、push 範囲の diff を取れない ref（remote_oid が local に無い等で `git diff` が非 0）は WARN を出さず続ける（`|| true`。`set -euo pipefail`〈`:4`〉の下で `$(…)` の失敗が hook を落とし、`record_outcome` の無い停止になるのを防ぐ）。その ref の分類は既存どおり classifier の full fallback（`classify-changes.sh:224-228`）に任せる。
- S9 `scripts/tests/pre-push.test.sh`（D6・D7）
  - fake の `gh`（`:56-82`）に `--state` の解析、`FAKE_MERGED_HEAD`（その branch の merged の照会に `[{"number":42}]` を返す）、`FAKE_MERGED_EXIT`（merged の照会だけ非 0）、呼出しの記録（`printf 'gh %s\n' "$*" >> "$CALL_LOG"`。`CALL_LOG` は `run_hook` が既に渡す `:130`）を足す。既定は merged も `[]`。
  - merged の branch（`FAKE_MERGED_HEAD=feature`）→ exit 非 0、log の最後が `FAIL merged-pr`、stderr に `already merged` と `new branch`。open の Draft の PR と merged の PR が同じ branch にある → exit 0（open を優先。calls.log に `--state merged` が無い）。PR 無し（`run_hook ""`。open も merged も `[]`）→ exit 0、log の最後が `PASS `、`FAIL merged-pr` が無く、calls.log に `--state merged` がある（照会が実行されたことの証拠）。merged の照会だけ失敗（`FAKE_MERGED_EXIT=4`）→ exit 非 0・`FAIL ready-state-lookup`。
  - push 範囲の file（`src/example.ts`）に未 stage の変更 → exit 0、stderr に `WARN: unstaged changes in pushed files` と path。push 範囲の外の file の未 stage の変更 → WARN が出ない。変更なし → WARN が出ない。remote_oid が local に無い SHA の push（`run_hook` に remote_oid の引数を足す。既定は `$base_sha`）→ exit 0、WARN が無い、log の最後が `PASS `（分類は classifier の full fallback。hook が `git diff` の失敗で落ちない証拠）。
- S10 `.gitignore` と `.claude/agents/**`（D3）
  - `.gitignore:116` の `.claude/agents` を消し、`:100-107` の comment に「`.claude/agents` は tracked の subagent 定義を置くので除外しない（本体の checkout の sandbox の mount は同期で実 dir になれば消える）」の 1 文を足す。他の行は変えない。
  - `.claude/agents/writer.md`: frontmatter `name: writer`、`description`（Plan Packet の Scope を worktree で実装する Writer。発注書が唯一の指示）、`model: opus`、`effort: medium`。body は 3 行以内（AGENTS.md の `Session Start` から読むこと、発注書が指示の正本であること、報告は発注書の様式）。規則を複製しない。
  - `.claude/agents/reviewer.md`: frontmatter `name: reviewer`、`description`（Plan Review・Final Review・closure の review-only の reviewer。Writer と別 context）、`model: opus`、`effort: medium`、`disallowedTools: Edit, Write, NotebookEdit`。body は 3 行以内（`docs/code_review.md` の出力の形、`docs/templates/subagent-review-packet.md` に従うこと）。
  - Fable 5.1 の reviewer（Final Review の Claude 側、相談役）は定義を作らない（座組の effort は high で、session の値の継承と同じ。Coordinator が Agent tool の `model` で指定する）。
- S11 `.claude/settings.json` と `scripts/tests/claude-hooks.test.sh`（D4）
  - `.claude/settings.json` に `"env": {"CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH": "1"}` を足す。他の key は変えない。
  - `scripts/tests/claude-hooks.test.sh`（D3・D4 と D-059 の zero inventory の延長）。冒頭（`validate_source_binding` の前）に `command -v ruby >/dev/null || fail "ruby (yaml) is required"` を置く（YAML は Ruby で読む。`scripts/tests/ci-workflow.test.sh:33-38`・`run-workflow-tests.sh:8` と同じ tool。hosted の docs job は ruby を apt しないが runner image に Ruby 3.2.3 が同梱〈2026-09-30 確認〉。無ければ green にせず止める）。次の関数を足す:

    ```bash
    frontmatter_lacks_hooks() {
        awk 'NR == 1 && $0 != "---" { exit } NR > 1 && $0 == "---" { exit } NR > 1 { print }' "$1" |
            ruby -ryaml -e 'doc = YAML.safe_load(STDIN.read, permitted_classes: [Date, Time]); exit(doc.is_a?(Hash) && doc.key?("hooks") ? 1 : 0)' 2>/dev/null
    }
    ```

    先頭行が `---` のときだけ frontmatter とし（公式 skills:348・sub-agents:337 と同じ規則）、top-level に `hooks` キーがあれば 1、無ければ 0、YAML として読めなければ 1（fail-closed。`hooks:`・`'hooks':`・`"hooks":`・`hooks :`・flow mapping・escape 付きを拒み、入れ子の `hooks`・date 値・frontmatter 無しを通す）。`validate_inventory`（`:13-26`）に、`jq -e '.env.CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH == "1"'` と、次の 2 つを足す（`:23` の hook_dir と同じ形。dir が無ければ通す）:

    ```bash
    local file
    while IFS= read -r -d '' file; do
        frontmatter_lacks_hooks "$file" || return 1
    done < <(
        [[ -d "$root/.claude/agents" ]] && find "$root/.claude/agents" -name '*.md' -print0
        [[ -d "$root/.claude/commands" ]] && find "$root/.claude/commands" -name '*.md' -print0
        [[ -d "$root/.agents/skills" ]] && find "$root/.agents/skills" -name SKILL.md -print0
        true
    )
    if find "$root/.agents" "$root/.claude" -path "$root/.claude/worktrees" -prune -o -name .claude-plugin -print 2>/dev/null | grep -q .; then
        return 1
    fi
    ```

    前者は hook を登録できる frontmatter 3 種（agent・command file・skill。公式 hooks「Hooks in skills and agents」、skills「Command files は `name`・`paths` 以外同じ frontmatter」、2026-09-30 確認）が `hooks` を持たないことの監査、後者は skill folder を plugin にする manifest（`.claude-plugin/`）が tracked に無いことの監査で、どちらも `CLAUDE.md` の「tracked project hook inventoryは空」を守る（`.claude/skills/*` は symlink で `find` は辿らず、実体は `.agents/skills` で見る。`.claude/worktrees` は除く）。`validate_audit_wiring`（`:68-81`）の classify の検査 path に `.claude/agents/reviewer.md` と `.agents/skills/example/.claude-plugin/plugin.json` を足す。既存の負例の作り方（fixture の copy を壊して red を確かめる形。`make_fixture`〈`:130-158`〉は `.claude/agents`・`.agents` を写さないので、負例は copy に dir ごと作る）で、正例 1 と負例 6 を足す: 正例 = `.claude/agents/writer.md`（clean な frontmatter）と `.agents/skills/example/SKILL.md`（`metadata:` に date 値）を置いた copy が green。負例 = env を消した fixture、classifier から `.claude/agents/*` を外した fixture、`.claude/agents/writer.md` の top-level に `'hooks':`（引用符付き）を置いた fixture、`.claude/commands/plan-rally.md` の先頭に `---`・`"hooks":`・`---` の frontmatter を足した fixture、`.agents/skills/example/SKILL.md` の frontmatter に `hooks:` を置いた fixture、`.agents/skills/example/.claude-plugin/plugin.json` を置いた fixture（frontmatter は clean）。
- S12 正本の文書（D1〜D9）
  - `docs/agent-guidance/merge-evidence.md`
    - `:53` の実行制御の行に `.claude/agents/**` と `**/.claude-plugin/**`、`:54` の policy の行に `docs/quality/review-checklist.md` を足す。
    - `:58` の段落（helper は current main 側の分類を使う）に 1 文: 「helper は自分の file が PR の base の `scripts/pr-gate.py` と同じときだけ動き、違えば base の版を使う command を示して止まる（D2。D2 を載せた版以降の helper に効く）」。
    - `## Helperの境界`（見出し `:140`。record の説明は `statusはread-only` で始まる同節の 146 行目の段落）の record の説明に `--reviewed-head`（review で必須、capture の head と一致）を足す。
    - `## 実行手順`（`:158-171`）: `:160` の「captureの返すpathを使い、SHAを手転記しない」の後に「reviewed headだけは、reviewerの報告が監査したcommitを書いていればそれを、無ければそのreviewを発注したreview packetの`対象差分と内容commit`欄（`docs/templates/subagent-review-packet.md:15`）のcommitを写す。captureの出力や現在のHEADから取らない（監査していないheadを記録する誤りをhelperがcaptureのheadとの照合で止める）」を足す。例の record（`:164-168`）に `--reviewed-head "$REVIEWED_HEAD"` を足す。`:171` の後に 2 文: 「broad は head を変える push（是正・base 同期）の前に、監査した head の capture で record する」「同じ head に closure run が 2 本あるときは、後に完了判定を出した run を model・run_ref にし、両方の証跡 pointer を evidence に並べて 1 回 record する」。
  - `docs/ci.md`
    - `## Local Commands`（`:49-61`）に 1 文: 「full は `node_modules` が symlink なら gate の前に失敗する（`npm ci` が symlink 先を空にするため）。changed は検査しない」。
    - `## Pre-push Contract`（`:67-73`）に 1 文: 「push 先の branch の PR が open に無く merged にあれば push を拒否し、main から新しい branch を案内する。push する file に未 stage の変更があれば WARN を出して push を続ける」。
    - `## Stale Green Prevention`（`:75-79`）の正規経路の文に「helper は PR の base と同じ版でだけ動く」を足す。
  - `docs/AGENT_OPERATING_MANUAL.md` の `## 座組` の表（`:65-72`。表の外と表の下にある箇条書きは触らない = PR4 の所有と PR4 の AC20 の不変条件）
    - Writer 行（`:68`）の effort 欄を置き換える: 「medium（owner 2026-09-23。Codex が Writer のときは Plan Reviewer 行の Codex の値）。起動: Agent tool では `subagent_type: writer`（`.claude/agents/writer.md`、effort medium）で起動する。定義を使わない起動は session の値（high）を継承する。subagent の入れ子は `.claude/settings.json` の `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=1` で止める（`docs/DEV_WORKFLOW.md` Subagent Budget の depth 1）。定義と設定は本体の checkout を同期し Claude Code を再起動してから効く（公式 [sub-agents](https://code.claude.com/docs/en/sub-agents)「Write subagent files」・「Supported frontmatter fields」の `effort`）。実効値を run 報告に記録する（`/tasks`）」。`.gitignore:116` と PR5 の句を消す。
    - Plan Reviewer 行（`:69`）の「Opus = medium（実効値の注意は Writer 行と同じ）」を「Opus = medium（`subagent_type: reviewer`、`.claude/agents/reviewer.md`、effort medium、編集の tool なし。起動と実効の注意は Writer 行と同じ）」に置き換える。Codex の句は変えない。
    - Final Reviewer 行（`:70`）の effort 欄「Opus = medium」を「Opus = medium（Plan Reviewer 行と同じ `reviewer` の定義）」に、「Fable 5.1 = high（owner 2026-09-25）」を「Fable 5.1 = high（owner 2026-09-25。定義を使わず session の値を継承）」に置き換える。
  - `docs/decision-log.md` の末尾に `## D-099` を追記する（番号の予約は下の G2）。書く内容: D1〜D10 の要旨。D2 の採った案と棄却案、および D2 の保証の範囲（D2 を載せた helper 以降。本 lane の merge より前に分岐した lane は手順で移行し、忘れると抜ける。旧 helper を classifier の出力 key の追加で fail-closed にする案は、ci.md Classifier Contract の 9 key を変えるので採らない）。D8 の保証の範囲（operator が監査した head を渡すときだけ。現在の HEAD を渡せば通る）。D10 の入れない判断と残るリスク。D1 で `.claude/agents/**` と `**/.claude-plugin/**`（skill folder を plugin にする manifest。公式 skills「Skill folder as a plugin」・plugins/loading の `@skills-dir`、2026-09-30 確認）を full にした理由と、plugin の中身（`hooks/hooks.json`・`agents/*.md`・`.mcp.json`）は full にせず manifest の不在を hook test で求める理由（列挙は漏れる）。skill・command file・agent の frontmatter の `hooks`（公式 hooks「Hooks in skills and agents」、skills「Command files」、2026-09-30 確認）は分類を変えず（skill 文書の編集がすべて R3 になる費用）、hook test が frontmatter を YAML として読んで top-level の `hooks` を拒む（D-059 の zero inventory の延長。検査を外す PR は hook test が full なので R3）。残るリスク: skill の body の `!` command（invoke 時に shell を実行。公式 skills、2026-09-30 確認。今は 0 件。塞ぐなら `.claude/settings.json` の `disableSkillShellExecution: true` を別 lane で）、`.claude/CLAUDE.md`・`.claude/output-styles/*`・`:59` の pattern の外の `docs/**/CLAUDE.md` が一般 docs のまま残ること（Matrix の Residual Test Gaps、今は tracked に無い）。

対象を使う呼出し側・隣接 test の確認: classifier の consumer は `scripts/pre-push.sh`・`scripts/local-ci.sh`・hosted の `changes` job・`scripts/pr-gate.py:220-221`（どれも出力 key で読み、path の一覧を持たない）と `scripts/tests/claude-hooks.test.sh:78-81`。helper の consumer は merge-evidence の手順と Coordinator の checklist（tracked 外）。`scripts/tests/run-workflow-tests.sh` は既存の test file だけを実行し、本 lane は新しい test file を作らないので登録の変更は無い。

## PR4 との file の所有

| file / 範囲 | 所有 |
|---|---|
| `docs/DEV_WORKFLOW.md`、`docs/templates/**`、`docs/code_review.md`、`docs/quality/review-checklist.md`、`scripts/doc-consistency-check.sh`、`scripts/check-workflow-git.sh`、`scripts/tests/doc-consistency-plan-packet.test.sh`・`workflow-git-checks.test.sh`・`reading-order-drift.test.sh`、`.agents/skills/**`、`AGENTS.md`（follow-up (5) の導線だけ）、`docs/AGENT_OPERATING_MANUAL.md`（`## 座組` の表を除く） | PR4 |
| `scripts/ci/classify-changes.sh`、`scripts/pr-gate.py`、`scripts/local-ci.sh`、`scripts/pre-push.sh` とそれぞれの test、`.gitignore`、`.claude/agents/**`、`.claude/settings.json`、`docs/agent-guidance/merge-evidence.md`、`docs/ci.md`、`docs/AGENT_OPERATING_MANUAL.md` の `## 座組` の表 | PR5 |
| `docs/decision-log.md` の末尾追記、`scripts/tests/run-workflow-tests.sh` への登録 | 両方（merge 順で両方を残す） |
| `docs/Plans.md`、`docs/backlog.md`、`docs/archive/**` | どちらも実装 PR で触らない（closeout） |

- 表に無く本 lane が触る file: `scripts/tests/claude-hooks.test.sh`（S11。settings の監査と classify の検査を持つ test で、settings と classifier を所有する本 lane に置く。PR4 の表にも無い）。判断点 G2 で Coordinator に確かめる。
- merge 順の影響: 本 lane が先に merge されると、PR4 が触る `docs/quality/review-checklist.md` が workflow=true になる。PR4 は `docs/DEV_WORKFLOW.md` 等で元から workflow=true・Minimum 2 なので要件は変わらない。PR4 の branch は本 lane の merge の前に分岐しており（merge-base `7ac96d9e`、PR4 の `scripts/pr-gate.py` は origin/main と同じ blob `95c5f91c`）、PR4 の helper は旧い版で D2 では止まらない。本 lane が先に merge されたら、PR4 は helper の最初の操作の前に origin/main を単段 merge する（strict で元々必要）か base の版の写しを使う。取り込むまでの PR4 の record は `--reviewed-head` 無しで通る（Residual Test Gaps）。PR4 が先なら本 lane への影響は無い（PR4 は `pr-gate.py` を触らない）。
- `docs/decision-log.md` は両方が末尾に足す。後に merge する側が単段 merge で両方の節を残す（番号は G2）。

## Non-scope

- PR4 の範囲すべて（上の表の PR4 の行）。`docs/DEV_WORKFLOW.md:254` の depth 1 の文は変えない（設定で機械化しても文の意味は変わらない）。
- strict の撤去、衝突を解いた版の manual の再利用の条件（owner 2026-09-28）。
- 過去の decision-log の本文と archive の書換え。
- 生成器（`src-tauri/src/bin/generate_*`、`lib.rs` の bindings の export）の classifier の扱い（D10、入れない）。
- RecordV1 の wire の変更（closure の audit を複数にする案。D9 の棄却案）。
- helper の自動の切替え（D2 の棄却案）。
- Fable 5.1・Codex 用の agent 定義（S10 の注記）。`.claude/agents` の `hooks`・`mcpServers`・`permissionMode` の利用。
- reviewer の報告の形（`docs/code_review.md` `## Output Shape`〈`:91`〉）と review packet の template（`docs/templates/subagent-review-packet.md`）に監査した commit の欄を足すこと: どちらも PR4 の所有。本 lane は merge-evidence の文で写し元を定めるだけにし、欄の追加は PR4 への申し送りにする。
- tracked 外の資料（`.local/` の checklist・発注書の雛形の symlink の許可の削除、個人 memory の教訓 100 の削除）: 本 lane の merge 後に Coordinator が直す。教訓 100（「subagent の effort 指定は Workflow の `agent()` で」）が要らなくなったことは、merge 後の AC7 の確認で決める。
- `docs/backlog.md` の follow-up (1) の項目を閉じること: closeout で行う（本 lane の実装 PR は backlog を触らない）。

## Acceptance Criteria

baseline は main `7ac96d9e`（本 branch の plan 側の HEAD と script・文書が同じ）で、同じ command を逐語で 2026-09-29 に起草役が実行した実測。test は全 PASS を求め、本数を AC にしない。

- AC1（classifier、D1、S1・S2）: 下の 4 つがすべて成り立つ（`classify-changes.sh --files-from-stdin` の出力と test の exit 0）。
  - `printf '%s\n' .claude/agents/reviewer.md | bash scripts/ci/classify-changes.sh --files-from-stdin` の出力が `rust=true`・`workflow=true`・`unknown=false`。baseline: `rust=false … docs=true … workflow=false unknown=false`。
  - `printf '%s\n' .agents/skills/example/.claude-plugin/plugin.json | bash scripts/ci/classify-changes.sh --files-from-stdin` の出力が `rust=true`・`workflow=true`・`unknown=false`。baseline: `rust=false … docs=true … workflow=true unknown=false`（policy。2026-09-30 実測）。
  - `printf '%s\n' docs/quality/review-checklist.md | bash scripts/ci/classify-changes.sh --files-from-stdin` の出力が `docs=true`・`workflow=true`・`rust=false`・`frontend=false`。baseline: `workflow=false`。
  - `printf '%s\n' docs/backlog.md | bash scripts/ci/classify-changes.sh --files-from-stdin` の出力が `workflow=false`（不変）。baseline: `workflow=false`。
  - `bash scripts/tests/classify-changes.test.sh` が exit 0（S2 の場面を含む）。baseline: exit 0（`PASS: classify-changes`、S2 の場面は未実装）。
- AC2（helper の自己照合、D2、S3・S5）: 下の 2 つ（test の exit 0 と実物の exit 1）。
  - `python3 scripts/tests/pr-gate.test.py -v -k HelperVersion` が exit 0 で各 test が `ok`。baseline: `NO TESTS RAN`、exit 5。
  - 実物: 本 lane の新しい helper の `status` を本 lane の worktree で実行して exit 1・stderr に `helper differs from base`（「本 lane 自身の検査」の 6）。`python3 "$TMPDIR/pr-gate-main.py" status …` は従来どおり動く。
- AC3（reviewed head、D8、S4・S5）: `python3 scripts/tests/pr-gate.test.py -v -k ReviewedHead` が exit 0。baseline: `NO TESTS RAN`、exit 5。`rg -n -- '--reviewed-head' scripts/pr-gate.py docs/agent-guidance/merge-evidence.md` が 3 行以上（helper の引数・Helperの境界・実行手順の例）。baseline: 一致なし（exit 1）。
- AC4（helper の既存の契約、D11）: `python3 scripts/tests/pr-gate.test.py` が exit 0。baseline: exit 0（`OK`）。
- AC5（local-ci、D5、S6・S7）: `bash scripts/tests/local-ci.test.sh` が exit 0（S7 の 3 場面を含む）。baseline: exit 0（`PASS: local-ci`、S7 の場面は未実装）。`rg -n 'node_modules' scripts/local-ci.sh` が 1 行以上。baseline: 一致なし（exit 1）。
- AC6（pre-push、D6・D7、S8・S9）: `bash scripts/tests/pre-push.test.sh` が exit 0（S9 の場面を含む）。baseline: exit 0（`PASS: pre-push`）。`rg -n -- '--state merged' scripts/pre-push.sh` が 1 行。baseline: 一致なし（exit 1）。
- AC7（agent 定義と depth、D3・D4、S10・S11）: 下の 4 つ（`git check-ignore` の exit 1、`jq` の出力 `1`、test の exit 0、merge 後の run 報告）。
  - `git check-ignore -v .claude/agents/reviewer.md` が exit 1（除外されない）。baseline: `.gitignore:116:.claude/agents` と対象 path（tab 区切り）、exit 0。
  - `git ls-files .claude/agents` が `.claude/agents/reviewer.md` と `.claude/agents/writer.md` の 2 行。`rg -n '^effort: medium$' .claude/agents` が 2 行、`rg -n '^disallowedTools: Edit, Write, NotebookEdit$' .claude/agents/reviewer.md` が 1 行。baseline: `git ls-files` は出力なし、`rg` は対象 path が無く exit 2。
  - `jq -r '.env.CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH' .claude/settings.json` が `1`。baseline: `null`。`rg -n -F 'key?("hooks")' scripts/tests/claude-hooks.test.sh` が 1 行（frontmatter の `hooks` の監査）、`rg -n -F '.claude-plugin' scripts/tests/claude-hooks.test.sh` が 1 行以上（manifest の不在の監査）。baseline: どちらも一致なし（exit 1）。`bash scripts/tests/claude-hooks.test.sh` が exit 0（S11 の正例 1 と負例 6 を含む）。baseline: exit 0。
  - merge 後（本体の同期と Claude Code の再起動〈G1〉の後）: Coordinator が最初に `subagent_type: writer` か `reviewer` で起動した run の `/tasks` の行に model と effort medium が出ること、その subagent が Agent tool を持たないことを run 報告に記録する（P3 の外部前提の確認。合否は Ready・merge の条件にしない。出なければ follow-up）。
- AC8（正本の文書、S12）: 下の各 `rg` の出力が期待の行数。
  - `rg -n '\.claude/agents' docs/agent-guidance/merge-evidence.md` が 1 行以上、`rg -n 'review-checklist' docs/agent-guidance/merge-evidence.md` が 1 行以上。baseline: どちらも一致なし（exit 1）。
  - `rg -n 'helper differs from base|base の版' docs/agent-guidance/merge-evidence.md docs/ci.md` が 1 行以上。baseline: 一致なし（exit 1）。
  - `rg -n '同じ head に closure run が 2 本' docs/agent-guidance/merge-evidence.md` が 1 行。baseline: 一致なし（exit 1）。
  - `rg -n 'symlink' docs/ci.md` が 1 行以上、`rg -n 'merged' docs/ci.md` が 1 行以上。baseline: どちらも一致なし（exit 1）。
  - `rg -n 'session の値（high）を継承する。Workflow から起動する|gitignore:116' docs/AGENT_OPERATING_MANUAL.md` が 0 行（exit 1）。baseline: 1 行（`:68`）。`rg -n 'subagent_type: writer' docs/AGENT_OPERATING_MANUAL.md` が 1 行、`rg -n 'CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH' docs/AGENT_OPERATING_MANUAL.md` が 1 行。baseline: どちらも一致なし。
  - `rg -c '^## D-099' docs/decision-log.md` が 1（番号は G2 で変わりうる）。baseline: 一致なし（exit 1）。
- AC9（mutation、Test Plan の MU1〜MU18）: 実装を commit した後、`$TMPDIR` の写し（`copy="$TMPDIR/pr5-mut"; mkdir -p "$copy"; git archive HEAD | tar -x -C "$copy"; git -C "$copy" init -q; git -C "$copy" add -A`。改変ごとに作り直し、終わったら消す。本 repo の index・設定は触らない）で各 mutation を入れ、対応する test が red（exit 非 0）になり、改変なしの写しでは green になる。
- AC10（検査の全体。すべて exit 0）: `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/doc-consistency-check.sh`、`bash scripts/doc-consistency-check.sh --target plan`、`bash scripts/check-workflow-git.sh`、`git diff --check origin/main...HEAD`、`bash scripts/local-ci.sh changed`、`bash scripts/check-env-safety.sh`（`.gitignore` を変えるため）。「本 lane 自身の検査」の 5（origin/main の checker と PK5 の cross-run）も exit 0。baseline（plan 側の HEAD の前、main と同じ内容で 2026-09-29 に起草役が実測）: run-workflow-tests exit 0、doc-consistency exit 0（既定・`--target plan` とも ERROR 0、WARN 1 = PK3 の review-only skip の警告〈P:62〉、PK4 OK。2026-09-30 実測）、check-workflow-git exit 0、`git diff --check origin/main...HEAD` exit 0。local-ci changed・check-env-safety は未実測（実装時に Writer が実測する）。`bash scripts/local-ci.sh full` は Writer の worktree の `node_modules` が実 dir のときだけ実行する（symlink なら本 lane の S6 がまさに止める）。
- AC11（範囲）: `git diff --name-status origin/main...HEAD` の変更 file が S1〜S12 の file と本 packet・Matrix に限られる。`docs/DEV_WORKFLOW.md`・`docs/templates/**`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`scripts/doc-consistency-check.sh`・`scripts/check-workflow-git.sh`・`.agents/**`・`AGENTS.md`・`docs/Plans.md`・`docs/backlog.md`・`src-tauri/**` に本 lane 由来の差分が無い。

## 判断点

owner に諮る判断点（G1）と、Coordinator に確かめる判断点（G2・G3）。各判断点は Plan Review で覆せる。

- G1（owner、plan-approved の依頼と同じ 1 回で諮る。merge 後の作業を含む）: `.claude/agents/**` と `.claude/settings.json` の書込みと、本体の checkout の同期・Claude Code の再起動。
  - 確認済みの事実: 本 lane の worktree（`.claude/worktrees/harness-pr5`）では、sandbox の Bash で `mkdir .claude/agents` が成功し（直後に `rmdir`、2026-09-29 の probe）、`.claude/settings.json` は `test -w` が真。本体の checkout では `.claude/settings.json` は `test -w` が偽、`.claude/agents` は sandbox の mount で塞がれ dir でない。公式の protected paths は `.claude` を保護し `.claude/worktrees` を除く（P2）。公式 sub-agents「Write subagent files」: watcher は session 開始時に存在した dir だけを見るので、`.claude/agents` を初めて置いた後は再起動が要る（2026-09-29 WebFetch）。Edit / Write の tool で worktree の `.claude/agents/**` を書けるかは未確認。
  - 選択肢: (a) Writer が worktree で書く。順に試す: Edit / Write の tool → worktree の sandbox の Bash（heredoc。`mkdir .claude/agents` が成功した probe がある）→ それも書けなければ Writer は内容を報告に書き、owner が sandbox の外（自分の terminal）でその file を作る（予備の介入 1）。merge 後、owner が本体の checkout を sandbox の外で origin/main に同期し、Claude Code の main session を再起動する（同期と再起動で介入 1）。再起動後の最初の run で Coordinator が `/tasks` を確かめる（AC7）。(b) 定義を tracked にせず owner の `~/.claude/agents/` に置く（review が掛からない。classifier の穴を塞ぐ意味が薄れる）。(c) Coordinator の main session を本体でなく origin/main の worktree から起動する運用に変える（session の起動のたびに owner の操作が要る）。
  - 推奨: (a)（candidate。Writer の最初の書込みの結果で、owner の代行が要るかが決まる）。承認すると、Writer・reviewer の subagent が座組表の effort（medium、owner 2026-09-23。`CLAUDE.md` は #117 で座組表を正本にした）で初めて実効になり、入れ子が設定で止まる。
  - G1 と同じ依頼で owner に示す残すリスク（判断点でなく確認）: 本 lane の後も機械で止まらないのは次の 4 つ。(1) skill の body の `!` command（invoke 時に shell を実行。公式 skills、2026-09-30 確認）は `.agents/**` の policy（rust=false）のままで、R1 の申告なら packet 無しに `pr-gate.py:259` を通る。今は 0 件。skill・command file・agent の frontmatter の `hooks` と skill folder の plugin manifest（`.claude-plugin/`）は hook test（full）が拒むので、この経路だけが残る。(2) `.claude/CLAUDE.md`・`.claude/output-styles/*`・`docs/**/CLAUDE.md`（`:59` の pattern の外）は一般 docs（今は tracked に無い）。(3) D8 は operator が監査した head を渡すときだけ塞ぐ（現在の HEAD を写せば通る。手順は merge-evidence の文）。(4) 本 lane の merge より前に分岐した lane の旧い helper は自己照合を持たず、移行は手順（「本 lane 自身の検査と merge」の 7）。選択肢: A（推奨）= 本 lane はこのまま、(1) は `.claude/settings.json` に `disableSkillShellExecution: true` を足す follow-up を backlog に置く（settings は full で hook test が監査する。skill の `!` を使わない今は運用に影響しない）。B = (1) を今すぐ塞ぐため `.agents/**`・`.claude/skills/**` を full にする（skill 文書の編集がすべて R3 packet と Minimum 2 になる。採らない）。C = (1) の settings の 1 key を本 lane の S11 に入れる（小さいが findings の外で、Final Review の対象が増える）。
- G2（Coordinator、PR4 の packet と照合）: decision-log の番号と、表に無い file（`scripts/tests/claude-hooks.test.sh`）。本 packet は D-099 を予約する（PR4 が D-098 を予約する前提）。PR4 の packet と食い違えば、Coordinator が裁定し、後に merge する側が packet の gated amendment で採番し直す（`docs/DEV_WORKFLOW.md` Review Rules の連番の規則）。
- G3（Coordinator、Plan Review で確かめる）: D2 の方式（自己照合で止める）と D10（生成器を入れない）。どちらも技術判断で、下の Design Intent Trace に理由と棄却案を書いた。

## Design Sources

- Requirements / spec: 該当なし（製品要件を変えない）。
- Architecture / Function / DB / Screen: 該当なし。
- Workflow 正本（本 lane が書き換える文書と、参照だけする文書）: `docs/agent-guidance/merge-evidence.md`（MG-D4 の分類表、MG-D8 helper の境界、RecordV1、「実行手順」）、`docs/ci.md`（Classifier / Pre-push Contract、Local Commands、Stale Green Prevention）、`docs/AGENT_OPERATING_MANUAL.md` の `## 座組`、`docs/DEV_WORKFLOW.md`（Risk Tiers、Subagent Budget の depth 1、Verification Gates。参照だけ）、`docs/project-profile.md` High-risk Changes（参照だけ）。
- 公式資料（2026-09-29 に WebFetch で確かめた。Contract Probe P1〜P4）: [Create custom subagents](https://code.claude.com/docs/en/sub-agents)（frontmatter の `effort`・`model`・`disallowedTools`、scope、「Let subagents spawn their own subagents」）、[Environment variables](https://code.claude.com/docs/en/env-vars)（`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH`）、[Settings](https://code.claude.com/docs/en/settings)（`env` の key と trust）、[Permission modes](https://code.claude.com/docs/en/permission-modes)（Protected paths）。
- Decision log / ADR: D-030（npm ガード、変えない）、D-034（Subagent Budget の depth 1）、D-059（hook inventory、`claude-hooks.test.sh`）、D-085 / MG-D1〜D12（helper と classifier）、D-092（座組表）、D-097（並走の摩擦。helper の差分の判定は変えない）。本 lane は D-099（予約、G2）を足す。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | existing sufficient（製品コードを変えない） |
| Command / DTO / generated binding / wire shape | なし | existing sufficient |
| DB / transaction / audit / rollback / migration | なし | existing sufficient |
| Screen / UI / route state / Japanese wording | なし | existing sufficient |
| CSV / TSV / report / import / export format | なし | existing sufficient |
| Workflow gate（classifier、helper、pre-push、local-ci、subagent 定義・設定） | `docs/agent-guidance/merge-evidence.md`、`docs/ci.md`、`docs/AGENT_OPERATING_MANUAL.md` の `## 座組` | updated in this PR（S12） |
| Durable decision / ADR | `docs/decision-log.md` D-099（予約） | updated in this PR（S12） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| decision-log の追記 | D-099（予約、G2）。PR4 の D 番号と突き合わせる |
| workflow の正本の文の変更（S12） | link の実在は `bash scripts/doc-consistency-check.sh` の link 検査で確かめる。文書の新設・改名・削除は無い |
| test の追加（S2・S5・S7・S9・S11） | 既存の test file に足す。`scripts/tests/run-workflow-tests.sh` の登録の変更は無い |
| `.claude/agents/**` の新設 | `.gitignore:116` の除外の解除（S10）。classifier の full（S1）。座組表からの参照（S12） |

Tauri command・function-design doc・REQ・route・operator 画面: 該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-HARNESS5 | merge-evidence MG-D4 の分類表 | D1 | `.claude/agents/*.md` は subagent の model・effort・tools に加え `hooks`（command を実行する）と `permissionMode` を持てる設定で、`.claude/settings.json`・`.claude/hooks/*` と同じ「実行制御」に置く。full にすると R0 / R1 の経路（`pr-gate.py:259`）で拒まれ、R3 の packet と Minimum 2 が要る。`docs/quality/review-checklist.md` は review の観点の正本（`docs/DEV_WORKFLOW.md` Review Rules・`AGENTS.md` の初回レビューが参照）で、`docs/code_review.md` と同じ policy docs に置く。却下: `.claude/agents/*` を policy docs（`:59`）に置く（監査の当初案。workflow 回帰と packet 付きの Minimum 2 は掛かるが、R1 の申告で packet 無しに reviewer の定義を変えられる経路が残る）。`**/.claude-plugin/**` は skill folder を plugin にする manifest で、agents・hooks・MCP を束ねて本体に読ませる入口（公式 skills「Skill folder as a plugin」・plugins/loading `@skills-dir`、2026-09-30 確認）なので実行制御に置く。中身の path は full にしない（manifest 無しでは読まれず、列挙は漏れる。manifest の不在は S11 が求める）。費用: agent 定義を変える PR は hosted で Rust・frontend の job も走る（変更は稀） | S1 | AC1、S2 |
| SPEC-WF-HARNESS5 | merge-evidence「Helperの境界」、`:58` | D2 | helper が PR の base の `pr-gate.py` と自分の bytes を比べ、違えば止まる。base の classifier を使う既存の設計（`:58`・`:62`「PR が自分用に弱めた policy を信頼しない」）を helper 本体に広げる。変更後の helper が照合そのものを消せば通るが、その変更は Double Audit が読む差分に出る。守るのは「手順を忘れて変更後の helper を使う」事故で、D2 を載せた helper 以降に効く（本 lane の merge より前に分岐した lane の旧い helper は止まらず、手順で移行する。「本 lane 自身の検査と merge」の 7）。悪意の helper の実行は守らない（hook と同じく local の実行は信頼の外）。却下: (i) 自動で base の版を取得して exec する（摩擦は無いが、GitHub から取った code を暗黙に実行する経路が増え、どの版が動いたかが見えにくい）、(ii) 実行場所の強制（wrapper script。wrapper も PR の checkout にあり同じ問題）、(iii) hosted CI での検査（helper は local で動き、CI はどの helper が動いたかを知らない）、(iv) `pr-gate.py` を変える PR だけ照合する（D2 以降も、main が helper を更新した後に古い branch が古い helper で record でき、D8 のような新しい検査が効かない）、(v) 旧 helper も止めるために classifier の出力 key を足す（旧 helper は key の集合の不一致で exit 2 になる `pr-gate.py:224` が、ci.md Classifier Contract の 9 key と MG-D4 の契約を変え、所有の外の hosted job の出力も変わる） | S3 | AC2 |
| SPEC-WF-HARNESS5 | 座組表、公式 sub-agents の frontmatter | D3 | 座組表の effort（Writer・Opus reviewer = medium）を Agent tool の起動で効かせる手段は frontmatter の `effort`（公式、P1）。`model: opus` は main が Opus 5.5 なら main と同じ model に解決する（P4）。reviewer は編集の tool を外す（Bash は test と mutation の実注入に要るので残す）。定義の body は規則を複製しない。定義・command file・skill の frontmatter の `hooks` と skill folder の plugin manifest は hook test が拒む（D-059 の zero inventory の延長。frontmatter は YAML として読み、引用符付き・flow mapping のキーも拾う）。却下: Workflow の `agent()` だけで指定する（Agent tool の通常の起動に効かない）、Fable の定義も作る（Fable は high で session の継承と同じ） | S10、S12 | AC7 |
| SPEC-WF-HARNESS5 | DEV_WORKFLOW Subagent Budget、公式 env-vars | D4 | depth 1 の規則を設定で機械化する（公式に `1` で入れ子を止めると明記、P1）。project の settings に置き、hook test で固定する。却下: 各定義の `disallowedTools: Agent` だけにする（定義を使わない起動に効かない） | S11 | AC7 |
| SPEC-WF-HARNESS5 | ci.md Local Commands | D5 | full だけが `npm ci` を実行するので、full の最初（gate の前）で止める。途中の gate を走らせてから止めると時間を捨てる。changed は止めない（worktree の symlink は changed の正当な使い方）。却下: `npm ci` の直前に置く（Rust の gate の後まで待つ）、symlink を自動で外す（本体の checkout を壊す操作に近づく） | S6 | AC5 |
| SPEC-WF-HARNESS5 | ci.md Pre-push Contract | D6 | open の PR が無く merged の PR がある branch への push は、PR に載らない commit を作るだけ。open があれば merged を照会しない（既存の open PR を止めない安全弁）。merged の照会は branch を削除した後も PR を返す（read-only probe #104、2026-09-29）ので、merge 済みの名前を再利用した最初の push は必ず止まる。名前の再利用は避け、message で新しい名前を案内する。照会の失敗は既存どおり止める。却下: closed（merge されていない）も止める（閉じた PR の branch を作り直す正当な使い方がある）、merged の `headRefOid` の祖先判定で再利用を許す（複雑さに見合う頻度が無い） | S8 | AC6 |
| SPEC-WF-HARNESS5 | ci.md Pre-push Contract | D7 | 未 stage の変更は正当な WIP でも起きるので止めず WARN。push する file と HEAD の ref に限って雑音を減らす。却下: 止める（WIP の push を止める）、全 ref に出す（HEAD でない ref の作業 tree は無関係） | S8 | AC6 |
| SPEC-WF-HARNESS5 | merge-evidence「Helperの境界」「実行手順」 | D8 | review の record に監査した head を必須にし、capture の head と一致を求める。broad・closure の両方に掛ける（closure も是正の後の capture で同じ誤りが起きる）。record の wire は変えない。却下: broad だけ（closure の同じ穴が残る）、record に reviewed head の field を足す（capture の head と同じ値の複製） | S4 | AC3 |
| SPEC-WF-HARNESS5 | merge-evidence「実行手順」、RecordV1 | D9 | 同じ head の 2 本の closure は、後に完了判定を出した run を 1 つの audit にし、両方の証跡を evidence に並べる（record は後の書込みで closure を置き換える現行の振舞いに合う）。却下: `Closure.audit` を配列にする（RecordV1 の wire・既存の record・fixture の変更が要り、closure の必要数は 1 のまま） | S12 | AC8 |
| SPEC-WF-HARNESS5 | ci.md Classifier Contract、教訓 52 | D10 | 生成器を workflow に入れない。`src-tauri/src/bin/generate_traceability.rs` は T4 の baseline（`FE_UNREFERENCED_BASELINE`）を持ち、UI の lane が REQ の無い frontend test を足すたびに更新する（#35・#40）。full にすると UI の lane が Minimum 2 と全 job を負う。bindings の export の本体は `src-tauri/src/lib.rs`（#88 の変更の主体）で、製品 code と分けられない。どちらも今は `src-tauri/*` で Rust・generated・traceability の全 gate が走る。gate を緩める生成器の変更の Risk は kickoff の「required gate の green / red が変わるか」（教訓 52）と Final Review が判定する。残るリスク: 生成器の判定を緩める変更を R1 と申告すれば packet 無しで通る（Residual Test Gaps）。却下: `generate_*.rs` を full にする（上の費用）、baseline を別 file に分けて判定の code だけ full にする（`src-tauri/**` の変更で本 lane の file の所有を越える。必要なら別 lane） | なし | なし（判断の記録は D-099） |
| SPEC-WF-HARNESS5 | merge-evidence 全体 | D11 | 変えないもの: record の head/base の照合、capture と server の一致、closure の要求、approved snapshot の照合、Double Audit の下限、main 側の classifier の使用、R0 / R1 の `--risk` / `--manual` の要求と CI 実行 code の拒否、manual の再利用の条件、strict、PR の差分による packet の判定（D-097）、pre-push の Ready の拒否と緊急 bypass の token | 全体 | AC4、AC10 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 規則は S12 の正本に、判断と棄却案は D-099 に置く。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: D1〜D10 を merge-evidence・ci.md・座組表と D-099 に置く（S12）。
- Assumptions and constraints: GitHub の contents API が PR の base の `pr-gate.py` を返す（classifier の取得と同じ経路、P5）。`gh pr list --state merged --head` が merge 済みの PR を返す（P6、read-only で確認済み）。project の subagent 定義と settings の `env` は本体の checkout の同期の後に効く（P3、外部前提）。
- Deferred design gaps, risk, and follow-up target: 生成器の classifier（D10、Residual Test Gaps）、agent 定義の実効値の確認（AC7 の merge 後の項目）、tracked 外の資料の更新（Non-scope）。
- Test Design Matrix can cite design decision IDs or source doc sections: D1〜D11 を Matrix が引く。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「helper は base と同じ版でだけ動く」の例外 = 照合を消した helper を実行する場合（D2 の守らない範囲、差分に出る）。「merge 済みの branch への push を止める」の例外 = open の PR がある場合（D6）と緊急 bypass の token（既存）。「symlink の `node_modules` で止める」は full だけ（changed は例外）。「record は監査した head を要る」は review だけ（manual / r4 は対象外）。「record は監査した head を要る」の例外 = operator が現在の HEAD を `--reviewed-head` に渡す場合（入力の規律は merge-evidence の文。D8 の守らない範囲）。「hook inventory は空」の例外 = skill の body の `!` command（frontmatter の外。Residual）。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 該当なし: 製品の adapter と core を変えない | — |
| Fact check / design decision split | 事実 = 現状の事実 1〜11（起草役が現物で確認）、公式資料（P1〜P4）、probe（P2・P6）。判断 = D1〜D10 と G1〜G3 | Contract Probe、D-099 |
| Lifecycle / retry | review の record の順序（broad → 是正の push → closure）を D8 で機械化。止まった後の戻り方を各 message に書く（base の版の command、新しい branch、実 dir の作り方、broad のやり直し） | Ordinary Operation |
| Operator workflow | 該当なし（店の operator の操作を変えない）。開発の操作列は Ordinary Operation | — |
| Replacement path | helper の実行を「checkout の file」から「base と同じ版」に替える。並走 lane は、D2 を載せた helper なら止まった時点で、旧い helper（PR4）なら本 lane の merge の直後に手順で、単段 merge か写しに切り替える（上の「PR4 との file の所有」） | S3、S12 |
| Data safety / evidence | 古い証拠の拒否は強まる（D8 が監査していない head の broad を拒む）。record の wire・capture の形は変えない。helper の自己照合は何も書かずに止まる | AC3、AC4 |
| Reporting / accounting semantics | 該当なし | — |
| Manual verification | 該当なし: 画面が無い。agent 定義の実効値は merge 後の run 報告で確かめる（AC7、Ready・merge の条件にしない） | AC7 |
| 環境・再現性 | agent 定義・depth の設定は本体の checkout の同期に依る（G1、P3）。local-ci の symlink の検査は `-L` だけで OS に依らない。pre-push の merged の照会は `gh` に依り、無ければ既存どおり止まる | G1、S6、S8 |

安全の境界が弱まらないこと:

- 古い証拠の拒否: record の head/base の照合、capture と server の一致、closure の要求は変えない（D11）。D8 は監査した head と capture の head の一致を足すので強まる。
- 独立 review の本数: Double Audit の下限と main 側の classifier は変えない。D1 で `.claude/agents/**` と review-checklist に Minimum 2 が掛かる範囲が広がる。D2 で、変更後の helper が自分の review の要件を読み替える経路が塞がる。
- Human Gate: approved snapshot の照合、Ready・merge の owner 指示は変えない。helper が止まる場面が増えるだけで、Ready・merge の条件を緩めない。
- fail-closed の既定: 新しい照会（base の `pr-gate.py`、merged の PR）の失敗は止まる側（exit 2、`fail_gate ready-state-lookup`）。D7 の WARN は push を止めないが、既存の gate の判定にも影響しない。

新しい停止が正当な作業を止めないこと:

- WIP の push: D7 は WARN だけで exit 0（S9 の test）。
- symlink でない worktree・changed: D5 は full かつ symlink のときだけ止まる（S7 の test）。
- open の PR の branch・PR の無い branch への push: D6 は open が空で merged があるときだけ止まる（S9 の test）。
- `pr-gate.py` を変えない PR の helper: base と同じ版なら従来どおり。D2 を載せた helper の lane は、main の helper が分岐の後に変わると止まり、単段 merge か写しで進む（1 command）。旧い helper の lane は止まらない（Residual Test Gaps）。
- agent 定義: 定義を使わない起動（general-purpose・Fable）は従来どおり。

## Design Readiness

- Existing design docs are sufficient because: 分類・helper・pre-push・local-ci の契約の置き場所（merge-evidence・ci.md・座組表）が既にあり、本 lane はその中の行・文を足し替える。新しい概念・wire は作らない（spec-check → plan-draft の skip）。
- Source docs updated in this PR: merge-evidence、ci.md、座組表、decision-log D-099。
- Design gaps intentionally deferred: D10 の生成器、Non-scope の各項目。
- Durable decisions discovered in this plan and promoted to source docs: D-099。

Minimum design checks for business-app work: 製品コードを変えないため、layer・function・DTO・永続化・画面・error の各項目は該当なし。testability は Matrix。

## Contract Probe

- P1「subagent の frontmatter の `effort` が session の effort を上書きし、`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=1` が入れ子を止める」: 公式 [sub-agents](https://code.claude.com/docs/en/sub-agents) の frontmatter の表（`effort`: 「Effort level when this subagent is active. Overrides the session effort level. Default: inherits from session」）と「Let subagents spawn their own subagents」（既定は 3 層、`settings.json` の `env` で変える、「Set `1` to turn nesting off」）、[env-vars](https://code.claude.com/docs/en/env-vars)（v2.1.217 以降、正の整数だけ受理）を 2026-09-29 に WebFetch → 成立。local の Claude Code は v2.1.284（`claude --version`）。
- P2「worktree の `.claude/agents/**` と `.claude/settings.json` を書ける」: 公式 [permission-modes](https://code.claude.com/docs/en/permission-modes) の Protected paths は `.claude` を保護し「except for `.claude/worktrees`」（2026-09-29 WebFetch）。本 lane の worktree で sandbox の Bash の `mkdir .claude/agents` が成功（直後に `rmdir`、差分なし）、`test -w .claude/settings.json` が真。本体の checkout では `test -w` が偽 → worktree の Bash では成立。Edit / Write の tool の経路は未確認（Writer が最初の書込みで確かめ、書けなければ sandbox の Bash、それも書けなければ G1 の (a) の代行）。
- P3「project の subagent 定義と settings の `env` は session の cwd の checkout から読まれる」: 公式 sub-agents の scope の表（`.claude/agents/` は「Current project」、cwd から上へ走査）と「Write subagent files」（watcher は session 開始時に存在した dir だけを見る。初めて置いた後は再起動）、settings（`.claude/settings.json` は clone の中で session を始めたときに読む。`env` の多くは folder の trust の後に効く）→ 成立（2026-09-29 WebFetch）。本体の checkout の HEAD は `253eef06`（origin/main より古い）で、`.claude/agents` は sandbox の mount で塞がれ dir でない → merge 後の本体の同期と Claude Code の再起動（G1）まで Coordinator の session には効かない。外部前提として AC7 の merge 後の項目で確かめる。
- P4「`model: opus` は Opus 5.5 に解決する」: 公式 sub-agents「Choose a model」（main の model が同じ family なら alias は main の model に解決する）→ 成立（main session が Opus 5.5 である限り）。
- P5「helper は cwd の git で動き、base の file を contents API で取れる」: `scripts/pr-gate.py:196`（`git rev-parse --show-toplevel`）、`rg -c '__file__' scripts/pr-gate.py` → 0 件（exit 1）、`:207-210` の `contents()` と `:220` の classifier の取得 → 成立。自己照合の `Path(__file__)` は新しく足す唯一の file 位置の参照で、写しを `$TMPDIR` から実行しても自分の bytes を読む。
- P6「`gh pr list --head <branch> --state merged` が merge 済みの PR を返す」: `gh pr list --head agent/harness-parallel-friction --state merged --json number,isDraft,state` → `[{"isDraft":false,"number":123,"state":"MERGED"}]`、同じ branch の `--state open` の照会は空（2026-09-29、read-only）→ 成立。
- P7「生成器は UI の lane が日常的に触る」: `git log -- src-tauri/src/bin/generate_traceability.rs` → #35・#40（UI の lane）が `FE_UNREFERENCED_BASELINE` を更新。`git show --stat 7d2002a0`（#88）→ `src-tauri/src/bin/generate_bindings.rs` 8 行と `src-tauri/src/lib.rs` 325 行 → 成立（D10 の前提）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D1 `.claude/agents/*` は full、`docs/quality/review-checklist.md` は policy、無関係の docs は不変 | S1 | AC1（S2）、AC7（S11 の classify の負例）、MU1・MU2 | — |
| D2 helper は base と同じ版でだけ動く、違えば何も書かず exit 1 | S3 | AC2（`HelperVersion`）、MU3・MU4・MU14 | 照合を消した helper の実行、本 lane の merge より前に分岐した lane の旧い helper（守らない範囲。手順で移行） |
| D3 writer / reviewer の定義（effort medium、reviewer は編集の tool なし）、除外の解除、frontmatter の `hooks` と plugin manifest の zero inventory | S10、S11、S12 | AC7（`claude-hooks.test.sh` の負例）、MU16・MU17 | 実効値は merge 後の run 報告（P3）。skill の body の `!` command は non-scope（Residual） |
| D4 depth 1 の設定 | S11 | AC7（`claude-hooks.test.sh` の負例）、MU5 | — |
| D5 local-ci full は symlink の `node_modules` で gate の前に止まる、changed は止めない | S6 | AC5（S7）、MU6・MU7 | — |
| D6 pre-push は merged の branch への push を止め、open を優先、照会失敗で止める | S8 | AC6（S9）、MU8・MU9・MU13 | — |
| D7 未 stage の変更の WARN、exit 0 | S8 | AC6（S9）、MU10・MU15 | — |
| D8 review の record は `--reviewed-head` を要り、capture の head と一致 | S4 | AC3（`ReviewedHead`）、MU11・MU12 | — |
| D9 同じ head の 2 本の closure の record の仕方 | S12 | AC8 | 文書の規則（helper は検査しない） |
| D10 生成器を入れない | なし | なし | non-scope（D-099 に記録） |
| D11 変えない境界 | 全体 | AC4、AC10 | — |
| 隣接: MG-D4 の分類表と classifier の一致 | S1、S12 | AC1、AC8 | — |
| 隣接: 座組表の effort と定義の一致 | S10、S12 | AC7・AC8 の rg | 表と定義の自動照合はしない（Residual Test Gaps） |

adjacent-contract sweep: S12 が触る節で上表に無い契約は、merge-evidence の RecordV1 の他の field・manual の再利用・base 同期・Actions 停止時、ci.md の Hosted Trigger・Cache Policy・Required Check、座組表の Coordinator・相談役・Human Gate の行（どれも変えない）。

## Test Plan

Test Design Matrix: [2026-09-29-harness-pr5-gate-holes.md](test-matrices/2026-09-29-harness-pr5-gate-holes.md)

- targeted tests: `bash scripts/tests/classify-changes.test.sh`、`python3 scripts/tests/pr-gate.test.py`（`-k HelperVersion`・`-k ReviewedHead` を含む）、`bash scripts/tests/local-ci.test.sh`、`bash scripts/tests/pre-push.test.sh`、`bash scripts/tests/claude-hooks.test.sh`。
- negative tests: Matrix の拒否の行（base と違う helper、監査した head の不一致・欠落、symlink の `node_modules` の full、merged の branch、merged の照会失敗、settings の env の欠落、classifier から `.claude/agents` を外した fixture）。
- mutation（AC9、各 mutation は実注入して red を確かめる。構造の推論だけで済ませない）:
  - MU1: S1 の `.claude/agents/*` を実行制御の行から policy の行へ移す → S2 の `.claude/agents` の `rust=true` の assert が red。
  - MU2: S1 の `docs/quality/review-checklist.md` を消す → S2 の review-checklist の `workflow=true` の assert が red。
  - MU3: S3 の比較を常に一致とする（照合を外す）→ `HelperVersion` の不一致の test が red。
  - MU4: S3 の照合を ready・merge だけに掛ける（status・capture・record では照合しない）→ `HelperVersion` の record の test が red（comment が書かれる）。
  - MU5: `.claude/settings.json` の env を消す → `claude-hooks.test.sh` が red。
  - MU6: S6 の検査を消す → S7 の symlink の場面が red（`npm ci` の stub が呼ばれ、ERROR が無い）。
  - MU7: S6 の条件から `MODE == full` を外す → S7 の changed の場面が red。
  - MU8: S8 の merged の照会を消す → S9 の merged の場面が red（exit 0）。
  - MU9: S8 の merged の照会の失敗を無視する（`|| true`）→ S9 の照会失敗の場面が red。
  - MU10: S8 の WARN を `fail_gate` にする → S9 の未 stage の場面が red（exit 非 0）。
  - MU11: S4 の一致の判定を外す → `ReviewedHead` の不一致の test が red。
  - MU12: S4 の判定を broad だけに掛ける → `ReviewedHead` の closure の不一致の test が red。
  - MU13: S8 の merged の判定を、open が空なら番号の有無に関わらず `fail_gate merged-pr` にする → S9 の PR 無し（`run_hook ""`）の場面が red（exit 非 0、`FAIL merged-pr`）。
  - MU14: S3 の base の `scripts/pr-gate.py` の取得の ref を head にする → `HelperVersion.test_compares_base_not_head` が red（順〈base = 実物・head = 実物 + 末尾の改行〉が exit 1、逆〈base = 実物 + 末尾の改行・head = 実物〉が exit 0）。ref に依らない既存の fixture（`contents` だけ）ではこの mutation は green のまま。
  - MU15: S8 の D7 の `git diff` から `|| true` を外す → S9 の remote_oid が local に無い場面が red（hook が `set -e` で落ち、exit 非 0、log に行が無い）。
  - MU16: S11 の `frontmatter_lacks_hooks` の呼出しを消す、または `^hooks:` の grep に戻す → `claude-hooks.test.sh` の `'hooks':`（agent）・`"hooks":`（command）の負例が red（`mutant was accepted`）。
  - MU17: S11 の `.claude-plugin` の検査を消す → `.agents/skills/example/.claude-plugin/plugin.json` の負例が red。
  - MU18: S1 の `*/.claude-plugin/*` を消す → S2 の `.claude-plugin` の `rust=true` の assert が red。
- compatibility checks: 既存の helper の test 全体（AC4）、classifier・local-ci・pre-push・hook の既存の場面（AC1・AC5・AC6・AC7）。
- data safety checks: 変更は script・test・設定・文書だけで、実データ・secret を含まない（Data Safety）。
- main wiring/integration checks: `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/local-ci.sh changed`、実装の PR の hosted CI、「本 lane 自身の検査」の 5・6。

## Boundary / Wire Contract

- producer: GitHub REST の `GET /repos/{owner}/{repo}/contents/scripts/pr-gate.py?ref=<base SHA>`（base64）、`gh pr list --state merged`（JSON）、CLI の `--reviewed-head`。
- consumer: `scripts/pr-gate.py` の `Gate.requirements()`・`Gate.record()`、`scripts/pre-push.sh`。
- wire type: contents の base64 の本文、`[{"number": N}]`、40 桁の小文字 hex の SHA。
- internal type: bytes の比較、PR 番号、SHA の文字列比較。
- precision/range: bytes の完全一致（正規化なし）。
- round-trip path: なし（record の wire・capture の形は変えない）。
- invalid input: base の取得失敗は exit 2、不一致は exit 1。`--reviewed-head` の欠落・SHA でない値は exit 2、不一致は exit 1。merged の照会の失敗は push を止める。
- compatibility: CLI は `--reviewed-head` が review の record で必須になる（旧い呼び方は exit 2 で止まる。merge-evidence の実行手順を同時に直す）。RecordV1・capture・exit code の意味は変えない。`.claude/settings.json` は key を足すだけ。

## Review Focus

- Plan Review の冒頭で `Ordinary Operation` の操作列が目的を達成できるかを `成立 / 具体的な反例あり / 外部前提が未確認` で答える（P3 は外部前提）。
- D2 の方式: 自己照合で止める案と、棄却した自動の切替え・wrapper・CI での検査・`pr-gate.py` を変える PR だけの照合の比較。並走の lane（PR4）への影響と、base の SHA が古い場合の振舞い（message の command が base の版を示すので進める）。
- D1 で `.claude/agents/**` を policy でなく full にした理由と費用。
- D10 で生成器を入れない判断と、残るリスクの置き方。
- D8 を closure にも掛けることが、正当な record の順序（merge-evidence の実行手順、manual の再利用の順序）を止めないか。
- D5・D6・D7 の新しい停止が、正当な作業（WIP の push、changed、symlink でない worktree、open の PR）を止めないか。
- G1 の owner の作業（本体の同期）が避けられないか、より軽い経路が無いか。
- 「本 lane 自身の検査と merge」が成り立つか（旧い helper に `--reviewed-head` が無い期間の順序を含む）。

## Spec Contract

Contract ID: SPEC-WF-HARNESS5

- D1: classifier は `.claude/agents/*`（下の階層を含む）と `*/.claude-plugin/*` を実行制御（全 area = true、unknown = false）に、`docs/quality/review-checklist.md` を policy docs（docs = true・workflow = true、rust = false・frontend = false）に分類する。他の path の分類は変えない。
- D2: helper は status・capture・record・ready・merge のすべてで、実行中の file の bytes が PR の base の `scripts/pr-gate.py` と完全に一致するときだけ先へ進む。違えば何も書かず exit 1 で base の版を使う command を示す。base の取得の失敗は exit 2。
- D3: `.claude/agents/writer.md` と `.claude/agents/reviewer.md` を tracked に置き、どちらも `model: opus`・`effort: medium`、reviewer は `disallowedTools: Edit, Write, NotebookEdit`。`.gitignore` は `.claude/agents` を除外しない。座組表は起動の仕方と実効値の確かめ方を書く。hook test は `.claude/agents/**/*.md`・`.claude/commands/**/*.md`・`.agents/skills/**/SKILL.md` の frontmatter を YAML として読み、top-level の `hooks` キー（引用符付きを含む）と、`.agents/**`・`.claude/**` の `.claude-plugin/` dir を拒む。
- D4: `.claude/settings.json` は `env.CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH = "1"` を持ち、hook test がそれを求める。
- D5: `local-ci.sh full` は `node_modules` が symlink なら、どの gate よりも前に ERROR を log して非 0 で終わる。changed は検査しない。
- D6: pre-push は push 先の branch に open の PR が無く merged の PR があれば push を拒否し（`FAIL merged-pr`）、main から新しい branch を案内する。open があれば merged を照会しない。照会の失敗は拒否する。
- D7: pre-push は HEAD の ref の push 範囲の file に未 stage の変更があれば stderr に WARN を出し、判定と exit code を変えない。push 範囲の diff を取れない ref は WARN を出さず続ける。
- D8: helper の review の record は `--reviewed-head` を必須にし、capture の head と一致しなければ何も書かず exit 1。
- D9: 同じ head の 2 本の closure run は、後に完了判定を出した run を model・run_ref にし、両方の証跡を evidence に並べて 1 回 record する（文書の規則）。
- D10: 生成器（`src-tauri/src/bin/generate_*`、bindings の export）は classifier の分類を変えない。
- D11: record の head/base の照合、capture と server の一致、closure の要求、approved snapshot の照合、Double Audit の下限、main 側の classifier、R0 / R1 の経路の条件、manual の再利用の条件、strict、PR の差分による packet の判定、pre-push の Ready の拒否と bypass の token は変えない。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-HARNESS5-D1 | S1、S2 | AC1、AC9（MU1・MU2・MU18） | full にした理由 | test 出力と mutation の exit |
| SPEC-WF-HARNESS5-D2 | S3、S5 | AC2、AC9（MU3・MU4・MU14） | 自己照合の方式 | test 出力、本 lane の新 helper の status の exit 1 |
| SPEC-WF-HARNESS5-D3 | S10、S11、S12 | AC7、AC8、AC9（MU16・MU17） | 定義の中身と座組表 | `git ls-files`・rg、merge 後の `/tasks` |
| SPEC-WF-HARNESS5-D4 | S11 | AC7、AC9（MU5） | 設定の効く条件 | jq・test 出力 |
| SPEC-WF-HARNESS5-D5 | S6、S7 | AC5、AC9（MU6・MU7） | 止める位置 | test 出力 |
| SPEC-WF-HARNESS5-D6 | S8、S9 | AC6、AC9（MU8・MU9・MU13） | open の優先 | test 出力 |
| SPEC-WF-HARNESS5-D7 | S8、S9 | AC6、AC9（MU10・MU15） | WARN の範囲 | test 出力 |
| SPEC-WF-HARNESS5-D8 | S4、S5 | AC3、AC9（MU11・MU12） | closure にも掛けること | test 出力 |
| SPEC-WF-HARNESS5-D9 | S12 | AC8 | wire を変えないこと | rg 出力 |
| SPEC-WF-HARNESS5-D10 | なし | なし | 残るリスク | D-099 |
| SPEC-WF-HARNESS5-D11 | 全体 | AC4、AC10 | 境界の弱化 | test 出力 |

## Data Safety

- 変更は tracked の script・test・設定・文書だけ。実 POS / 店舗データ、DB、backup、log、receipt、secret、`.env*` を読まず、commit しない。
- local-only: `.local/`（発注書、helper の capture、local-ci の evidence）、`$TMPDIR`（AC9 の mutation の写し、helper・checker の写し）。
- synthetic-only: helper・classifier・pre-push・local-ci・hook の test の fixture（合成の path・SHA・packet・`node_modules` の dir）。GitHub の実 PR は read-only の GET でだけ読み、fixture に写さない。
- local-ci の test は fixture の中の dir だけを symlink の先にし、本 repo の `node_modules` を触らない。

## Implementation Results

Fill after implementation.

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- 実装: [PR #127](https://github.com/higekiri-w/inventory-system-desktop/pull/127)。S1〜S12 を Writer（Opus 5.5 subagent、発注 154）が 6 commit で実装した（classifier、helper の自己照合と `--reviewed-head`、local-ci、pre-push、settings の env と hook test、正本の文書と D-099）。AC1〜AC11（AC7 の merge 後の項目を除く）を満たし、MU1〜MU18 を実注入して red を確かめた。
- G1 (a) の経路: `.claude/agents/{writer,reviewer}.md` と `.claude/settings.json` は Writer が tool で書けた。`.gitignore` の編集は auto mode の classifier に拒否され（Self-Modification）、Writer は迂回せず、owner が sandbox の外で `.gitignore` の 2 か所と定義 2 本を commit・push した（予備の介入 1 を使用）。
- 実装の判断: pre-push の push 範囲の起点を関数 `push_range_base` に移し、D7 と分類で共用した（分類の振舞いは変えない）。AC2 の実物（新しい helper の `status` の exit 1）は broad の record の前に確かめた。record の後の再確認は「本 lane 自身の検査と merge」の 6 として Coordinator が行う。
- base 同期: origin/main を単段 merge で取り込んだ（衝突なし。main 側は `docs/backlog.md`・`docs/project-memory.md` だけ）。
- 実装の要旨（closeout、2026-09-30）: classifier は `.claude/agents/**` と skill folder の plugin manifest を実行制御、`docs/quality/review-checklist.md` を policy docs に置く。helper は PR の base と同じ版でだけ動き、review の record に監査した head（`--reviewed-head`）を求める。local-ci full は `node_modules` が symlink なら gate の前に止まり、pre-push は merge 済みの PR の branch への push を止め、push する file の未 stage の変更に WARN を出す。writer・reviewer の subagent 定義を tracked に置き、`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=1` で入れ子を止め、hook test が frontmatter の `hooks` と plugin manifest を拒む（D-099）。
- review・CI・merge: Final Review broad の受理 P2 3 件を Writer が是正し、closure は Codex GPT-6.1 Sol が approve。Ready の後の hosted CI の red（`claude-hooks.test.sh` の ruby が `Date` を require しない。手元は gem の psych が隠した）を Coordinator が `ruby --disable-gems -ryaml -rdate` に直し、その後の closure も GPT-6.1 Sol が approve。manual は not-required。merge は PR5 → PR4 の順（owner 2026-09-30 の train 委任）で、helper 経由の squash merge（2026-09-30）。
- AC7 の merge 後の項目（2026-09-30、G1 = owner の本体の同期と Claude Code の再起動の後、最初の `subagent_type: reviewer` の run）: subagent の tool に `Agent` / `Task` が無く（depth 1 が効いた）、Edit / Write / NotebookEdit も無く（定義の `disallowedTools` が効いた）、実効 model は Opus 5.5 だった。effort は次の `subagent_type: writer` の run（本 closeout）の `/tasks` の行が `Opus 5.5 (1M context) (medium)` で、session の high を継承していない（owner が `/tasks` で確認）。

## Review Response

Fill after review.
If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.

Plan Review round 1（2026-09-29、対象 `ae965767`）: fresh Opus 5.5 = reject（P1 0 / P2 2 / P3 5、O1〜O7）、Codex GPT-6 Astra = reject（P1 1 / P2 5、C1〜C6）。Coordinator が現物で裏取りし全件を採用した。是正の文面は相談役 Fable 5.1 の起草（反例 A・C を採り、B は採らない）を起草役が反映した。行番号は本是正の commit のもの。

- O1（P2、skill の frontmatter の `hooks` が policy のまま R1 で通る）: 採用。分類は変えず、Matrix の Adjacent Pattern Audit の実行制御の行（M:86）の除外理由を事実に直し、Matrix の Residual Test Gaps（M:152）、S12 の D-099 の書く内容（P:200）、G1 の残すリスクの項（P:269）に書いた。
- O2（P2、「PR 無し」の test が Draft PR を入力する）: 採用。S9 に `run_hook ""`（open も merged も `[]`）の場面と fake `gh` の呼出しの記録を足し（P:175-176）、Matrix の D6 の行（M:59）を差し替え、MU13（P:416）を足した（Ledger P:386・Trace P:468・AC9 P:257・Matrix M:137）。
- O3（P3、branch 名の再利用）: 採用（小さい方）。S8 の D6（P:172）の説明と message を「新しい名前で切り直す」に直し、Design Intent Trace の D6 の行（P:313）に、merged の PR の head の祖先判定で再利用を許す案の棄却を書いた。
- O4（P3、`--reviewed-head` の値の出所）: 採用。S12 の merge-evidence の実行手順（P:191）に「reviewed head は reviewer の報告の監査 commit の行から写し、capture・現在の HEAD から取らない」を足した。
- O5（P3、symlink の ERROR 文言）: 採用。S6（P:164）の message を、symlink を末尾の / 無しで外してから `npm ci --ignore-scripts` で実 dir を作る案内にした。
- O6（P3、同期の後の再起動と代行の予備）: 採用。Owner Effort Budget（P:34・P:41）、G1（P:265-269）、Ordinary Operation の 1 行目（P:112）に再起動と代行の予備を入れた（C3・C5 と同じ是正）。
- O7（P3、`CLAUDE.md` の effort 節と座組表の medium の食い違い）: 採用。前提が origin/main に無かったため、判断点にせず G1 の承認文に 1 文（相談役の反例 A）。G1 の推奨（P:268）に「座組表の medium〈owner 2026-09-23〉が初めて実効になる。`CLAUDE.md` は #117 で座組表を正本にした」を入れた。
- C1（P1、PR5 の前に分岐した lane の旧い helper は自己照合で止まらない）: 採用。D2 の自動停止の保証を「D2 を載せた helper 以降」に限り、移行手順と「忘れると抜ける」事実を書いた: 最小完了条件（P:84）、本 lane 自身の検査と merge の 7（P:137）、PR4 との file の所有の merge 順（P:214）、D2 の行（P:309、旧 helper を classifier の出力 key で止める案は棄却案 (v)）、Replacement path（P:337）、新しい停止の項（P:355）、Ledger の D2（P:382）、D-099（P:200）、Matrix の State Lifecycle（M:76）と Residual Test Gaps（M:151）。
- C2（P2、review-checklist の Minimum 2 の保証が Goal と設計で違う）: 採用。最小完了条件（P:83）を「packet 付きなら Minimum 2」に揃え、packet 無しの経路を Matrix の Residual Test Gaps（M:154）に書いた。
- C3（P2、定義の初回の読込みに session の再起動が要る）: 採用。最小完了条件（P:88）、Ordinary Operation の 1 行目（P:112）と列の結び（P:125）、G1（P:265-269）、AC7 の merge 後の項目（P:249）、Contract Probe P3（P:371）に「本体の同期 → Claude Code の再起動 → 最初の run で確認」を入れた。
- C4（P2、「PR 無し」の予定 test が Draft PR を入力する）: 採用。O2 と同じ是正。
- C5（P2、介入の予算に代行と並走の再 Ready が入っていない）: 採用。介入の上限を 7（消費 1 + 見込み 4 + 予備 2、予備 = G1 の owner の代行 1・PR4 の merge の後の再 Ready 1）にした（P:34・P:41）。
- C6（P2、S12 が PR4 所有の、座組の表の下にある箇条書きを編集する）: 採用。depth と同期の注記を PR5 が所有する座組表の Writer 行の effort 欄に置き、表の外と表の下にある箇条書きを触らないと明記した（P:196-199）。PR4 の packet の編集は不要（相談役の反例 C: PR4 の AC20 が `## 座組` 節全体の不変を求める）。
- 行番号の正誤（相談役の起草 §1 末尾）: S12 の `## Helperの境界` の参照に見出し `:140` を足した（P:190）。record の説明の段落は起票時の 146 行目で正しく、起草の「record の段落は 144 行目」は `追加候補は` で始まる別の段落なので採らず、146 行目の段落を文で指した（`sed -n 146p docs/agent-guidance/merge-evidence.md` が `statusはread-only…recordは…`）。AC7 の baseline を「`rg` は対象 path が無く exit 2」に直した（P:247）。

Plan Review round 2（2026-09-30、対象 `38811a2a`）: Codex GPT-6 Astra = reject（P1 0 / P2 1、C1）、fresh Opus 5.5 = reject（P1 0 / P2 2 / P3 5、O1〜O7）。Coordinator が現物で裏取りし全件を採用した。是正の文面は相談役 Fable 5.1 の起草を起草役が反映した。行番号は本是正の commit のもの。

- C1・O1（P2、D2 の test が head から取得する誤実装を検出できない）: 採用。S5 に base と head で本文が違う `test_compares_base_not_head`（P:162）と MU14（P:419）を足し、Matrix の D2 の行（M:50・M:51）と Adjacent（M:87）をその test に結び付けた。Ledger（P:384）・Trace（P:469）に MU14。
- O2（P2、reviewed head の写し元の欄が報告に無い）: 採用。S12 の実行手順の文（P:192）を「報告が監査した commit を書いていればそれ、無ければ発注した review packet の `対象差分と内容commit` 欄」に直し、欄の追加を PR4 への申し送りとして Non-scope（P:227）に書いた。
- O3（P3、relay の消費）: 採用。表（P:40・P:43）を 2026-09-30 時点の消費 2・見込み 3 にした。
- O4（P3、`.claude/CLAUDE.md` 等が一般 docs に落ちる）: 採用（分類は変えない）。Matrix の Adjacent Pattern Audit（M:90）と Residual Test Gaps（M:157）に 1 行、D-099（P:201）に 1 文。
- O5（P3、agent 定義の frontmatter の `hooks:`）: 採用。S11（P:186）に `^hooks:` の検査と負例、AC7（P:250）、MU16（P:421）、Spec Contract D3（P:454）、Ledger（P:385）・Trace（P:470）、Matrix の D3（M:13・M:54）。
- O6（P3、D7 が diff を取れない ref で hook を落とす）: 採用。S8（P:174）に「WARN を出さず続ける」、S9（P:178）に場面、MU15（P:420）、Spec Contract D7（P:458）、Ledger（P:389）・Trace（P:474）、Matrix の D7（M:65）と Negative Paths（M:98）。
- O7（P3、行番号と事実）: 採用。`CALL_LOG` の行（P:176）を `:130` に、Minimum 0 の行（M:159）を `pr-gate.py:268` に直し、本体の `.claude/agents` の見え方（P:67・P:373）を「mount で塞がれ dir でない」とだけ書いた。

Plan Review round 3（2026-09-30、対象 `3dbdf369`。round 天井）: Codex GPT-6 Astra = reject（P1 0 / P2 1、F1）、fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 7、#1〜#8）。round 4 は無い（`docs/DEV_WORKFLOW.md` Review Rules の round 天井）。Coordinator が現物で裏取りし全件を採用、disposition は同型指摘の一括是正。是正の文面は相談役 Fable 5.1 の起草（反例 A〜D を採る）を起草役が反映し、Coordinator が現物で確かめて plan-approved を owner に諮る。独立 reviewer の再確認は Final Review。行番号は本是正の commit のもの。

- R3-F1（Codex P2、`'hooks':` が `^hooks:` の grep をすり抜ける）: 採用。S11（P:188）の検査を Ruby の `YAML.safe_load` で frontmatter を読む形（`scripts/tests/ci-workflow.test.sh` と同じ tool。解析不能は fail-closed）に替え、引用符付きの負例を足した。AC7（P:279）、MU16（P:450）、Spec Contract D3（P:485）、Matrix の D3（M:13・M:57・M:58・M:59）。
- R3-O1（Opus P2、skill folder の `.claude-plugin/` と `.claude/commands` の frontmatter `hooks`）: 採用。S1 の full に `*/.claude-plugin/*`（P:147）、S2 に例（P:151）、merge-evidence の分類表（P:217）、AC1（P:265）、D1（P:339・P:483）、Matrix（M:11・M:48・M:94）。`.claude/commands` と skill の frontmatter の `hooks` は分類を変えず、S11 の同じ検査を command file と `SKILL.md` にも回して機械で拒む（相談役の反例 B。費用が無いので残すリスクにしない）。plugin の中身は full にせず、manifest の不在を S11 が求める（反例 C）。G1 の残すリスク（P:300）と D-099（P:229）を書き直した。
- R3-O2（P3、D8 は監査した head を渡すときだけ塞ぐ）: 採用。最小完了条件（P:87）、Design Intent Audit の例外（P:358）、D-099（P:229）、Matrix の Residual（M:162）。
- R3-O3（P3、base 同期の push も head を変える）: 採用。Ordinary Operation（P:119）、本 lane 自身の検査の 3（P:136）、S12 の文（P:220）。
- R3-O4（P3、message の command が `$TMPDIR` 前提）: 採用。S3（P:155）と Ordinary Operation（P:122）を `git fetch origin && … "${TMPDIR:-/tmp}/pr-gate-base.py"` にした。
- R3-O5（P3、不一致の fixture の形）: 採用。S5（P:164）と Matrix（M:41・M:52・M:54・M:153）を「実物 + 末尾の改行」に固定した。
- R3-O6（P3、relay の予備）: 採用。上限 6、消費 3・見込み 2・予備 1（P:38・P:44）。
- R3-O7（P3、G1 (a) の順序）: 採用。tool → worktree の sandbox の Bash → owner の代行（P:298・P:401）。
- R3-O8（P3、S7 の fixture と AC10 の baseline）: 採用。S7 に `src-tauri/`（P:172）、AC10 の baseline に PK3 WARN 1（P:289）。
- 相談役の反例 D（skill の body の `!` command が残すリスクの記述に無い）: 採用（記述だけ）。G1（P:300）、D-099（P:229）、Design Intent Audit（P:358）、Matrix の Residual（M:161）。塞ぐ手段（settings の `disableSkillShellExecution`）は本 lane に入れず follow-up 候補。

Final Review broad（2026-09-30、対象 `c3129f59`）: Fable 5.1 = approve（P1 0 / P2 0 / P3 3、F-1〜F-3）、Codex GPT-6 Astra（発注 156）= reject（P1 0 / P2 2 / P3 1、A1〜A3）。数えない観察の Codex GPT-6.1 Sol（発注 157、PR へ投稿しない。owner 2026-09-30）= reject（P1 0 / P2 3 / P3 1、S1〜S4。S1・S2・S4 は A1〜A3 と同じ）。broad の 2 本は是正の push の前に `c3129f59` で record した（origin/main の helper の写し、本 lane 自身の検査と merge の 3）。Coordinator が現物で裏取りし、P2 は全件採用した。是正は Writer（Opus 5.5 subagent、発注 159）。

- A1・S2（P2、`scripts/tests/claude-hooks.test.sh:43` が `.claude/skills` の下の実 file を監査しない）: 採用。`.claude/skills` の `SKILL.md` を frontmatter の監査に入れ（symlink は `find` の既定で辿らない）、実 dir の `hooks` 付き skill の負例を足した（`bd18c65a`）。D3（P:487）の監査対象の列挙と S11（P:192-197）の literal code はこの追加の前の形のまま。実装が D3 の目的（hook を登録できる frontmatter を持たせない）を広げる向きなので、packet は書き換えず本項で結ぶ。
- A2・S1（P2、`:15` の awk が CRLF の frontmatter を「frontmatter なし」として通す）: 採用。行末の CR を落としてから区切りを判定し、CRLF の agent 定義の `hooks` の負例を足した（`bd18c65a`）。
- S3（P2、`:47` の `find … | grep -q .` が `pipefail` の下で SIGPIPE の 141 を「manifest なし」と読む。観察の Sol だけが検出）: 採用。Coordinator が `.claude-plugin` 200 個で 3/3 再現（1 個なら 3/3 拒否）。`find … -print -quit` の出力が空でないかで判定し、200 個の負例を足した（`bd18c65a`、修正前 5/5 red）。数えない run の finding だが、D3 の zero inventory が数で破れる実害の筋なので採る。
- A3・S4（P3、helper の復帰 command の末尾が literal の `…`）: 見送り。S3 が message を literal と決めており、直すには Amendment が要る。安全側に止まり、operator が引数を補えば動く。closeout で backlog へ。
- F-1（P3、docs job が runner image の ruby に頼る）: 見送り。`.github/**` は本 lane の所有の外。fail-closed。closeout で backlog へ。
- F-2（P3、frontmatter の `permissionMode`・`mcpServers` を拒まない）: 見送り。範囲の拡大。full（R3・Minimum 2）には乗る。closeout で backlog へ。
- F-3（P3、`--reviewed-head` の短縮 SHA の error に引数名が無い）: 採用（文書だけ）。merge-evidence の実行手順（`:160`）に「40 桁の full SHA で写す（短縮 SHA は helper が拒む）」を足した（`92f04dd0`）。helper は変えない。
- 是正の検証（Writer の報告、`92f04dd0`）: 受理 P2 3 件は修正前 red → 修正後 green。MU5・MU16・MU17 を再注入して red。AC7・AC10 exit 0、origin/main の checker と PK5 の cross-run exit 0、`bash scripts/local-ci.sh full` PASS（`MERGE_EVIDENCE_VALID=true`）。変更は `claude-hooks.test.sh` と merge-evidence の 2 file だけ。

- Ready 後の hosted CI（2026-09-30、run `36632701759`、head `e3103e2b`）: docs job と workflow job の `claude-hooks.test.sh` の live 検査が `live Claude hook contract is not the D-059 zero-hook inventory` で red。Draft の間は hosted が走らず、初めての実行だった。原因（Coordinator が再現）: `frontmatter_lacks_hooks` の `YAML.safe_load(…, permitted_classes: [Date, Time])` が `Date` を require していない。手元の rbenv の ruby は gem の psych 5.3.1 が `date` を読み込むので通り、runner の標準の psych 5.0.1 では `NameError`（stderr を捨てる）→「hooks あり」と判定された。`ruby --disable-gems` で手元でも同じ失敗を再現。Findings Freeze ② の実行時の失敗で証明された P2 として、Coordinator が `ruby --disable-gems -ryaml -rdate` に直した（1 行）。修正前（`-rdate` なし）は手元で同じ文面の red、修正後 green。`--disable-gems` で手元と hosted が同じ標準ライブラリで動くので、`-rdate` を消すと手元でも red になる（固定）。PR を Draft に戻し、Codex の closure で確かめる。Fable の F-1（docs job の ruby）の見送りは変えない（runner image に ruby があり、失敗の原因ではなかった）。

- Findings Freeze: frozen after Final Review broad（Double Audit の 2 本、2026-09-30）; post-freeze exceptions: none.

- Closeout（2026-09-30）: 是正の後の closure は Codex GPT-6.1 Sol = approve（取り直しの 163。161 は発注書の矛盾で数えなかった）、Ready の後の hosted CI の red を直した後の closure も GPT-6.1 Sol = approve（164）。見送った P3 の A3・S4（helper の復帰 command の末尾が literal の `…`）、F-1（hosted の docs job が runner image の ruby に頼る）、F-2（frontmatter の `permissionMode`・`mcpServers` を拒まない）と、G1 で後回しにした `disableSkillShellExecution` の follow-up は `docs/backlog.md` へ置いた。backlog の PR2・PR3 の follow-up (1)（同じ head の 2 本の closure の record）は本 lane の D9 で閉じた。
