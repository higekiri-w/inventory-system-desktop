# Plan Packet: 入口と重複の整理（harness 改訂 PR3、R3）

2026-09-25 起草。出典は harness 監査（2026-09-24、fresh Opus 5.5、local-only の報告。要点は本 packet に書き下した）の §0・§3.A / 3.K / 3.N / 3.O / 3.P / 3.Q / 3.S・§4・§5・§7 の PR3 行・§8 と、PR1（[archive packet](2026-09-24-harness-legacy-and-execution-mode-removal.md)）が Non-scope で PR3 へ送った項目。owner 決定: 2026-09-24「規則は環境が変わるたびに変える（安全境界は維持）」、2026-09-23「Coordinator = high、Writer / reviewer = medium」（本体は PR2 の座組表）、2026-09-25「PR2とPR3の並列はやろうかな」（本 packet の起票承認）、2026-09-25（Final Review の Claude 側は R3 以上で Fable 5.1、Plan Review の Codex は owner の指定で Sol）。npm 供給網ガード（D-030）を AGENTS へ移すのは監査 §9 の技術判断。2026-09-25 の owner 決定で PR2 は MANUAL の全面的な縮約をやめ、既存の見出しを変えずに `## 座組` を H2 で新設する範囲に絞った。本 packet の PR2 への前提（「PR2 との境界と merge 順」）はその範囲に合わせてある。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Phase: archive
- Risk: R3
- Plan Commit: 920740eaab41228c1760f78942fe818df130c853
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session）
- Writer: Opus 5.5 subagent（fork でない fresh context、Coordinator が指定する worktree で作業）
- Plan Reviewer: fresh Opus 5.5 subagent + Codex（GPT-6 Sol、effort high。owner 指定 2026-09-25）。互いに独立で Writer と別 context。Codex が rate limit 中なら Opus だけで進め、Codex の結果は戻り次第加える
- Final Reviewer: Fable 5.1（fresh subagent）+ Codex。互いに独立で Writer・Plan Reviewer と別 context（Double Audit）。run の報告に実効 model を記録し、Fable から別 model へ切り替わった run は Fable の本数に数えず取り直す
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: 製品の runtime・画面・配布物は変わらない。r4 なし: data・DB・破壊的 git 操作を含まず、変更は revert で戻せる。

座組は owner 決定 2026-09-23（Opus 5.5 を Coordinator / Writer / review へ全面解禁、第三者性の要る review は fork でない fresh subagent、Codex 停止中は Final Review だけ待つ）と 2026-09-25（Final Review の Claude 側は R3 以上で Fable 5.1）に従う。`docs/AGENT_OPERATING_MANUAL.md` の D-056（Opus は read-only）との衝突は PR1 と同じく owner の現行決定を優先し、改訂は並走の PR2 が行う。

遷移記録（append-only）:

- kickoff → spec-check → plan-draft（2026-09-25、起草役 = Opus 5.5 subagent、Coordinator 補助）: Risk R3（下記 Risk）。設計の正本は本 PR が書き換える入口文書そのもので、規則の中身は owner 決定と監査で決まっており、owner の設計判断を要する未決の論点は無い（spec-check → plan-draft の唯一の skip。Design Readiness 参照）。
- plan-draft → plan-gate（2026-09-25、本 commit）: packet と Matrix を plan-first commit で確定し、`docs/Plans.md` の「次の行動」に登録した。Plan Review へ。
- plan-gate（2026-09-25、Coordinator の指示で是正）: Plan Review round 1 は両 reviewer とも reject。plan-gate のまま packet を是正した。findings と裁定の詳細は round 2 の完了後に Review Response へ記録する。
- plan-gate（2026-09-27、round 2 の是正）: Plan Review round 2 は両 reviewer とも reject、plan-gate のまま是正。owner 決定（2026-09-25）で PR2 の範囲が縮小されたので、PR2 への前提を合わせた。findings と裁定の詳細は Plan Review の完了後に Review Response へ記録する。
- plan-gate（2026-09-27、round 3 の一括是正）: Plan Review round 3（上限）は両 reviewer とも reject。round 4 は回さず、同型の一括是正（裁定 r3）。予算の改定は owner 承認待ち。
- plan-gate → plan-approved（2026-09-27、Coordinator、本 commit）: Plan Review round 3（上限、対象 `fb1cc6bf`）は Codex 側（GPT-6 Astra、high）が P1 1、Claude 側（fresh Opus 5.5）が同じ指摘を P2 1 として reject（`inventory-code-review` が併用を指示する `engineering-review` Skill に別の重大度・label の規範が残り、最小完了条件 3 が成立しない、ほか P3）。範囲の漏れで作り直しには当たらないため、round 天井に従い追加の round は回さず、同型の一括是正（`920740ea`、裁定 r3）で S5 に engineering-review を加えるなど全件を反映し、Coordinator が現物で確かめた。owner 承認（2026-09-27）のもと予算を介入 9・relay 5 に改めた。Plan Commit = 本 commit の親（round 3 の一括是正後の承認版 `920740ea`）。
- plan-approved → implementing（2026-09-27、Coordinator、本 commit）: plan-approved の承認版（`2a209954` の親）のまま Writer（Opus 5.5 subagent）に実装を発注する。packet の契約は変えない。
- implementing → archive（2026-09-28、Coordinator の closeout、本 commit）: PR #117 を squash merge（`dc2f14f1`、2026-09-28、helper 経由）。packet と Matrix を archive へ移送し、Implementation Results と Review Response を記録。Fable の P3-2 を本 packet で直した。Plans.md と backlog を同期。

## Owner Effort Budget

- 介入回数上限: 9（owner 承認 2026-09-27 で 4 から改定。消費 5 = 起票承認 2026-09-25 1、Plan Review round 1〜3 の Codex relay 3、本改定と plan-approved の承認 1。残る見込み = Final Review の Codex relay 1、必要なら Codex closure 1、Ready 1、merge 1）。closeout 時の消費 9 / 上限 9 = plan-approved までの 5、Final Review の Codex broad の起動 1、base 同期の後の Codex closure の起動 1、Ready 1、merge 1
- 実働時間上限: 15分（文書と test の変更で、owner の作業は Codex relay と Ready / merge の判断に限られる見込み）
- relay 往復上限: 5（owner 承認 2026-09-27 で 2 から改定。消費 3 = Plan Review round 1〜3 の Codex。残り = Final Review の Codex broad 1、必要なら是正の後の Codex closure 1。理由: round が天井の 3 まで回り、是正の後の取り直しにも Codex の合否を含めるため〈owner 決定 2026-09-25〉）。closeout 時の消費 5 / 上限 5 = Plan Review 3、Final Review broad 1、closure 1
- Plan Review round 天井: 3（既定 3。round 3 で到達し、round 4 は回さない）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
`docs/project-profile.md:97`（High-risk Changes の「Test/workflow gates that affect what may be merged」）に当たる。本 PR は workflow 回帰 suite が実行する 2 本の test の要求を変える: `scripts/tests/reading-order-drift.test.sh:157-165`（直接 UI merge / 残存リスクの文を要求する file を 4 から 2 へ減らし、Skill 5 本への `Evidence Mode` の要求を外す）と `scripts/tests/codex-safe-wrappers.test.sh:365-370`（T13 が `CLAUDE.md` に要求する memory namespace を外す）。どちらも「変更前に red だった入力が変更後に green になる」方向の変更で、監査 §6 も drift test の要求文言の変更を gate の変更に数えている。監査 §7 は PR3 を R2 と書いたが、それは PR1 が reading-order-drift の変更を PR3 へ移す前の見積りで、file の種類ではなく gate への影響で R3 とする（2026-09-22 の PR #88 で R2 起票 → Final Review で R3 へ引き上げた前例と同じ判定）。`scripts/ci/classify-changes.sh:55` は `scripts/tests/*` を実行制御 = full にし、`:59` は入口文書を policy docs にするため workflow=true になり、helper は Final Review Minimum 2 を要求する（R2 でも同じ本数）。helper（`scripts/pr-gate.py`）・checker・classifier・CI 定義は変えない。R4 には当たらない。

Review-only skipped because: R3 の review-only sub-agent は、Final Review の Double Audit の broad（Fable 5.1 と Codex の 2 本、どちらも Contract Audit を正本から行う）が兼ねる（`docs/DEV_WORKFLOW.md` Review Rules・Draft PR Checkpoint、`docs/code_review.md:73`）。

## Goal

Goal Invariant:

### 最小完了条件

- 本 PR の merge 後、Claude と Codex のどちらの session も、入口（`AGENTS.md` / `CLAUDE.md` / `.claude/rules` / `.claude/commands` / workflow Skill）から、廃止済みの仕組み（Evidence Mode の選択、legacy、state-only、三点一致、exact-HEAD 証拠、撤去済みの Phase 名）を手順として読まない。
- vendor 中立の `AGENTS.md` だけで、npm 供給網ガード（D-030）と、owner へ送る問い合わせ・agent 側で解く問い合わせの行き先（PR の label・thread 状態・close・issue comment を含む）が分かる。`CLAUDE.md` は冒頭の `@AGENTS.md` の import で `AGENTS.md` を Claude の session に読み込ませ、Claude 固有の補助だけを持ち、座組と effort は PR2 の座組表を参照する。
- review の依頼・出力の規範（依頼形式・重大度・出力形式）は `docs/code_review.md` の 1 箇所で決まり（併用する review Skill の `inventory-code-review` と `engineering-review` も重大度・出力形式を code_review に従わせ、別の重大度の label を定めない）、subagent 発注の入力欄は `docs/templates/subagent-review-packet.md`（PR2 の所有。規範を複製しない）が持ち、薄い wrapper（`pr-review` / `review-only-subagent` Skill、`pr-review-prompt` template、`.claude/rules` 2 本、`.claude/commands` 3 本）が消えても、残る参照がすべて実在の file を指す。
- 守るべき境界（直接 UI merge 禁止と残存リスクの文、Session Start の R2+ 経路と fail-closed、history-view の旧 path・旧 namespace の拒否、hook inventory 0 本）は、変更前に拒否していた入力を変更後も拒否する。

### 失敗定義

- 入口のどれかが廃止済みの仕組みを指示し続ける、または削除した file への参照（markdown link・Skill 名・command 名）が live な文書に残る。
- npm ガード・問い合わせの行き先・PR 操作の承認境界のどれかが、移す途中で消える・弱まる（`CLAUDE.md` から消えたのに `AGENTS.md` に無い）。
- test の要求を緩めた結果、`AGENTS.md` か `docs/DEV_WORKFLOW.md` から直接 UI merge 禁止の文が消えても通る、Session Start の R2+ 経路や fail-closed が消えても通る、`CLAUDE.md` に history-view の旧 namespace が入っても通る。
- PR2 と同じ節を書き換えて衝突させる、または PR2 の merge 前後で link が壊れる。

### 非目的

- 座組表・役割・独立性・effort の値そのもの（PR2）。
- DEV_WORKFLOW の節の書き換え、Plan Packet template、review-only と Final Review の統合、WER、Owner Effort Budget（PR4）。
- classifier・helper・checker・CI 定義の変更（PR5 / PR4）。
- 入口の読書経路そのものの再設計（表の構造と drift test の契約は保つ）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、発注 → 停止と訂正 → review → owner 判断の通常列を merge 後の入口の読み方で書く。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 本 PR の merge 後、新しい Claude session が R2+ 作業を再開する | `CLAUDE.md`（冒頭の `@AGENTS.md` の import で `AGENTS.md` が起動時に読み込まれる）→ `AGENTS.md` `Session Start` の R2+ 行 → 対象 packet | Evidence Mode の確認や legacy の照合を求められず、packet の完全な Workflow State と helper status・専用 record・CI で現在地を知る。AGENTS に移した npm ガードと問い合わせの行き先も同じ session に入る | R2+ 行と fail-closed の文が残り、drift test が green（AC6）。import の行が backtick の外の行頭に 1 回ある（AC11） | なし |
| Coordinator が subagent の effort を決める | `CLAUDE.md` の参照から `docs/AGENT_OPERATING_MANUAL.md#座組` を開く | 担当ごとの effort を座組表の effort 列から読む（`CLAUDE.md` に effort の値も古い high / xhigh 方針も無い） | PR2 が merge 済みで見出し `## 座組` が実在（AC11） | なし（見出しの文字列は PR2 の AC1 `^## 座組$` が固定する） |
| Codex が review-only 発注を受ける | 発注書が読む範囲（packet / Matrix / 差分 / 観点）を指定し、Codex は `AGENTS.md` `Session Start` の初回レビュー行を読む | 発注書が指定した範囲から始め、Session Start の全体の読書経路は踏まない。R3/R4 の Contract Audit が求める設計正本は発注の範囲に関わらず読み、具体的な欠陥の根拠があれば他の範囲外も読んで理由を報告に書く。重大度と出力形式は範囲に関わらず `docs/code_review.md` に従って返す | 初回レビュー行に、指定範囲から始める経路と、範囲に関わらず設計正本と code_review の重大度・出力形式を使う条件がある（AC4） | 実際の token 削減量は未実測（backlog の 2026-09-07 実測は旧経路のもの） |
| Codex の Writer が npm package を足す | `AGENTS.md` の npm 供給網ガード節を読む | 名指し・`--save-exact`・cooldown bypass の承認制を Claude と同じ規則で知る | 節が `AGENTS.md` にあり `CLAUDE.md` に無い（AC3） | なし |
| Writer が編集前に止まる、または店の事実が要る判断に当たる | `AGENTS.md` Decision and Approval Boundaries の問い合わせの行き先を引く | 発注だけの誤り・環境・技術的前提は Coordinator / agent 側で解き、店の事実・製品の振舞い・受容リスク・Ready / merge・PR の label や close は owner へ送る。判断は owner で、現行の明示承認と有効な委任の範囲では Coordinator が実行を代行できる。agent 側の事実確認の後の裁定は裁定権者が行い、owner にある場合は中継を省かない。owner に技術的な正しさの承認や発注訂正の伝言を頼まない | 行き先と PR 操作の行が `AGENTS.md` にある（AC5） | `docs/DEV_WORKFLOW.md` の旧表は PR4 が link に置き換えるまで並存（Non-scope） |
| reviewer が Final Review を行う | `docs/code_review.md` を読む | 保守者として読めるか（P2）と、店の事実を `docs/project-memory.md` の「現場の前提」と照合する観点を固定観点として受け取る。checklist の件数上限やカテゴリ外を理由に指摘を黙って落とさず、カテゴリ外の指摘も severity を付けて返す | code_review と review-checklist の該当行（AC8、AC9） | なし |
| PR を作る | `.github/pull_request_template.md` から本文を作る | `Evidence Mode` 行と legacy の記録指示の無い本文になる。helper は PR 本文を使わないため merge 手順は変わらない | template の該当 2 行が無い（AC1） | なし |

## Scope

行番号は base `e7c22f8f`（本 branch の merge-base。2026-09-27 に main を取り込んだ。前の base `85b18a04` から本 PR の対象 file と PR2 の所有 file〈MANUAL・DEV_WORKFLOW・templates・agent-guidance〉は変わっていないことを `git diff --stat 85b18a04 e7c22f8f` で確かめた）のもの。起票時の base `8bd2bc9a` から本 PR の対象 file で変わったのは `docs/quality/review-checklist.md` と `docs/backlog.md` だけで、backlog は行番号でなく entry の文で指す。各 file の変更は該当行・節に限り、他の節を書き換えない。

- S1 `AGENTS.md`
  - `Session Start` `:14`（R2+ 実装・再開）: 「Evidence Modeを確認する。legacyのhuman-confirm以降は…停止する。」の 2 文を消し、「実装後の現在地は helper status・専用 record・CI で確認する」の 1 文にする。`R2+`・`実装`・`再開`・`Workflow State`・`完全` の語と `:23` の `fail-closed` は残す（drift test の check_entry_contract）。
  - `:15`（初回レビュー）: 「発注書が読む範囲を指定した review（例: Codex の review-only 発注）は、その範囲から始め、Session Start の全体の読書経路は踏まない。R3/R4 の Contract Audit が求める設計正本は、発注の範囲に関わらず読む。具体的な欠陥の根拠があれば他の範囲外も読み、理由を報告に書く。重大度と出力形式は範囲に関わらず `docs/code_review.md`」を足す（backlog の「Codex review-only 発注向けの Session Start fast path」entry の最短経路。発注書 template 側の宣言行は `.local/` の運用で tracked の対象外）。
  - `:21`: 「未指定 profile は `frontier`」を消し、座組・effort は `docs/AGENT_OPERATING_MANUAL.md#座組`（markdown link）、Codex/OpenAI 固有の補助は `docs/agent-guidance/`、Claude 固有の補助は `CLAUDE.md`、の参照にする（profile の既定は `docs/agent-guidance/README.md:6` の「未指定は frontier」が同じ内容を持つ重複なので、入口に profile 名を書かない。PR2 が profile の file を畳むかどうかに依らない）。
  - `:27`: 「GitHub evidence modeの適用条件と専用recordは」を「merge は helper 経由で、専用 record と手順は」にし、`直接UI merge` と `残存リスク` を含む文は残す。
  - `## Decision and Approval Boundaries`（`:38-45`）: 問い合わせの行き先を 2 項目で足す。(i) owner へ送り中継を省かないもの = 店の事実・実機でしか確認できない挙動（具体的な質問か短い PASS / FAIL）、目的・製品の振舞い・受容リスク・予算・優先順位・範囲の変更・不可逆な操作、Windows L3・R4・Ready・merge、PR の label・thread 状態・close・issue comment といった外部書込み（`AGENTS.md:45` が別承認とする操作）。判断は owner。現行の明示承認と有効な委任（push・Draft PR 作成、owner の承認後の helper `ready` / `merge` と closeout の実行）の範囲では Coordinator が実行を代行できる（委任の範囲の文は、`docs/DEV_WORKFLOW.md` Wave Operation の train 委任〈`:240`、PR4 の所有で本 PR は変えない〉を除き、tracked の正本ではここだけに置く。MANUAL〈PR2 の所有〉の Human Gate の記述が委任に触れる場合もここを参照先にする）。複数に当たり owner 側を含む場合と、裁定権の所在が争点の場合は owner へ送る。(ii) agent 側で解くもの = 発注だけの誤り（正本が一意で発注が食い違う）・環境の不備（許可済みの環境・範囲で解消でき、追加の費用・権限・正本の変更を伴わない場合）・技術的な前提（調査・Contract Probe）・正本の意味の争いの事実確認。事実確認の後の裁定は裁定権者が行い、owner にある場合は中継を省かない。owner に技術的な正しさの承認を求めず、証跡の編集や発注訂正の伝言を頼まない。出典は `docs/DEV_WORKFLOW.md:282-296` の表と注記 (a)〜(c)、行 7 の穴は backlog の「問い合わせの行き先の表の行 7 に `AGENTS.md` が別の明示承認とする PR 操作が挙がっていない」entry。`:45` の文は残す。
  - 新しい節 `## npm 供給網ガード（D-030）` を `## Safety` の後に置き、`CLAUDE.md:37-40` の 4 項目を移す（「userの明示承認」は「owner の明示承認」とする）。
- S2 `CLAUDE.md`
  - 冒頭（H1 の直後、Claude 固有の指示より前）に `@AGENTS.md` を code span・code block の外の独立した 1 行で置く。Claude Code は `CLAUDE.md` があると `AGENTS.md` を既定では読まず、`CLAUDE.md` の `@path` は import として起動時に展開される（backtick の中は展開されない）。公式 memory 文書が、`CLAUDE.md` と `AGENTS.md` を併用する repo にこの import を勧めている（Contract Probe）。
  - `:7` の冒頭の文（`AGENTS.md` への markdown link と「`Session Start` が唯一の読書ルート」）を、import で読み込んだ `AGENTS.md` の `Session Start` が唯一の読書ルート、の意味の文にする（AGENTS を読めと言葉で指示する文を import と重ねない。公式文書も言葉の指示を import に置き換えるよう勧める）。同じ段落の「ここに順序やgateを複製しない」と Skill の案内は残す。
  - `:11`（Evidence Mode の定型文）を消す。
  - `## Fable 5.1 補助`（`:15`）の見出しを「長い作業の報告」にする（本文 `:17`・`:19` は残す）。
  - `## Sonnet / Opus の effort`（`:21-27`）を「座組と effort」の短い節にする: 既定値は座組表（`docs/AGENT_OPERATING_MANUAL.md#座組` への markdown link）が正本で、ここに値を書かない。`.claude/settings.json` の `effortLevel` は main session の値。project 設定を読まない別 session（`--safe-mode` 等）では effort を明示し、要求値と取得できた実行 metadata を記録する（`:25` の前半を残す）。Sonnet 5 / Opus 5 guide の link と「medium へ下げない」は消す。
  - `:33` の memory 格納 path の 1 文を消す（大文字 `Projects` の古い path。実在は小文字で、機械ごとの値なので入口に書かない。S13 の T13）。
  - `## npm供給網ガード（D-030）`（`:35-40`）を消す（S1 へ移す）。`:42` は npm ガードを含めて AGENTS を参照する文にする。
- S3 `.claude/rules/`: `commands.md` と `review-workflow.md` を削除（どちらも正本への参照と定型文だけ）。`implementation-quality.md:17` の定型文を消す。`test-quality.md` は定型文を持たないため変えない（監査 §7 の「2 本の定型文」は実物では 1 本）。
  - archive `docs/archive/plans/2026-09-15-display-fixes-batch-2.md:22` の「視覚系 UI change の運用教訓」の link（`.claude/rules/review-workflow.md` を指す）だけを code span（`` `.claude/rules/review-workflow.md` ``）に替える（削除する file を link で指し、link 検査は `docs/archive/**` も走査するため）。この file の他の行は変えない（Non-scope の archive 不可触の唯一の例外）。
- S4 `.claude/commands/`: `test.md` / `design-review.md` / `phase-complete.md` を削除。`check.md:7` の定型文を消す。`plan-rally.md` は `:8` の「Owner Effort Budget、Subagent Budget」を「Plan Review round 天井」に、`:18` の「Packetの予算上限へ達したら」を「Plan Review round 天井に達したら」に、`:23` の「role / consultation relay」を「role / 座組」にし、`:26` の定型文を消す（Subagent Budget の数値上限と相談窓口役〈MANUAL §5.5〉は PR2 が撤去する。`:23` の置換先の `座組` は PR2 が新設する `## 座組`）。
- S5 workflow Skill（`.agents/skills/**`。`.claude/skills` の symlink は sandbox で書込み不可のため触らず、`pr-review` / `review-only-subagent` には symlink が無いことを `ls -la .claude/skills` で確認済み）
  - `inventory-workflow-start/SKILL.md`: `:20` から `pr-review` を消す。`:24` の定型文を消す。`:13`（MANUAL の §3 の anchor と語「Capacity-degraded」）は変えない（PR2 は MANUAL の既存の見出しを変えないので、anchor も語も PR2 の前後で有効）。
  - `inventory-implementation/SKILL.md`: `:20` の「human-confirm直後だけの追加fullは要求しない。legacyのready-hosted-finalの…L1 fullを行う。」の 2 文を消す。`:26` の定型文を消す。
  - `inventory-code-review/SKILL.md`: `:9`（「汎用レビュー哲学（bug 優先・nit/blocking・scope 規律・実証防御・指摘応答）は `engineering-review` スキルを参照」）から `nit/blocking` を消し、「重大度・出力形式は `docs/code_review.md`」の 1 句を足す。`:60` の `exact-HEAD evidence` を「helper status と専用 record」にする。`:61`（legacy D-035 の手順）を消す。`## Output Format`（`:65`）は見出しを残し、本文 `:67-74` を「findings・重大度・指摘なしの返し方は `docs/code_review.md` の `## Output Shape` と `## Finding Severity` に従う（指摘なしは `No blocking findings.`）」の 1 文に替える（現行 `:73` の `重大な findings なし` と、code_review `:91` の `No blocking findings.` の二重の規範を code_review に一本化する）。`## GitHub PR Comment Posting` の冒頭（`:85`）に「投稿してよい範囲は `AGENTS.md` Decision and Approval Boundaries」の 1 文を足す。`:99` の定型文を消す。
  - `engineering-review/SKILL.md`（`inventory-code-review:9` が併用を指示する Skill。`.claude/skills/engineering-review` は同じ本文への symlink）: 自らの SSOT の主張を review 哲学に限り、重大度・出力形式を `docs/code_review.md` に従わせる。(a) `## 責務境界`: `:17` の「汎用レビュー哲学の SSOT」に「重大度・出力形式は含まない（正本は `docs/code_review.md` の `## Finding Severity` と `## Output Shape`）」を足す。`:21` の表の行から `nit/blocking` を消し、「重大度・出力形式 | `docs/code_review.md`」の行を足す。(b) `:36` の step 4「severity 順に findings を報告 — Blocking / P1 / nit の順」を「findings を `docs/code_review.md` の `## Finding Severity`（P1 / P2 / P3）の順に、`## Output Shape` の形で報告する」にする（`reviewer/comments.md` への参照は書き方の原文として残す）。(c) `## nit / blocking の明示ラベルと scope 規律`（`:76`）の見出しを「label と scope 規律」にし、`:80-82` の label 定義 3 行を 1 文に替える: 「重大度と merge を止めるかどうかは `docs/code_review.md` の P1 / P2 / P3 だけで決める。`nit:` / `optional:` の label を添える場合は P3 の finding の書き方の補助に限り、同じ PR で直すか follow-up にするかは `docs/code_review.md` の `## Same PR vs Follow-up` に従う」。`:84` の scope 規律と `:86` の原文ポインタは残す。変えないもの: `:3` の description と `:7` の題（どちらも SSOT を review 哲学として名乗る）、`:102` の原文ポインタ表の「（nit / blocking / 提案）」（Google の原文の内容紹介）、`references/eng-practices/**`（無改変の原文、CC BY 3.0）。
  - `pr-review/`・`review-only-subagent/` を削除（review の依頼・出力の規範は S6 で `docs/code_review.md` へ、subagent 発注の入力欄は `docs/templates/subagent-review-packet.md`〈PR2 の所有〉が持つ。規範を複製しない）。
- S6 `docs/code_review.md`
  - `:36`: Evidence Mode の句（「Evidence Modeを先に確認する。github modeは」と「実装後state-onlyとPR本文L1の三点一致はlegacyだけ。」）だけを消し、検査項目の列挙は残す: 「実装後の状態は helper status と MG-D5〜D8（現行 `:36` の `agent-guidance/merge-evidence.md` への markdown link を残す）の server record・CI で確認し、record の対象 head/base、broad/closure、manual/R4、実効 rules を検査する。reviewer は専用 record を編集しない」。
  - `## Verification Rules` の箇条（`:40-49`）に「data safety の review では `git status --short` を見て、実 POS / 店舗の成果物が ignored のままかを確かめる」の 1 行を足す（S10 で消す project-profile `## Review Commands / Tools` の `:184` の移し先）。
  - `docs/templates/pr-review-prompt.md` の固有部分を `## Output Shape` の前後へ移す: 外部 reviewer（Codex 等）への PR review 依頼で渡すもの（Repo / PR / Title / Branch / Commit、In scope / Non-scope、Critical Contracts、claim として扱う検証）、実装しない・findings first・style / 命名 / 将来の拡張 / 明示された non-scope で block しない、の 3 点と、必要な時だけ unresolved questions / verification gaps / merge-split judgment を付ける（`pr-review-prompt.md:56`）の 1 句。重大度・closure・出力形式は code_review に既にあるので書き足さない。
  - `## Output Shape` に、finding ごとに確信度を添える要求を足す（現行 `docs/templates/subagent-review-packet.md` の `## Output` の要求を移す。PR2 が review packet からそれを外して code_review を参照にする。確信度の移し先はここだけ）。
  - `## Finding Severity` の表の P3 の maintainability に「保守者が読めない変更は P2（Blocking Review Focus）」を添える（K13 と矛盾させない）。
  - K13: `## Blocking Review Focus` に「保守者として読めるか（命名、理由の comment、賢い圧縮より退屈な構造、関数の長さ）。読めない変更は P2（owner 2026-09-07）」の 1 行を足す。
  - K14: `## Source Order` に「店の事実を前提にする判断は、`docs/project-memory.md` の「Store Premises Facts（現場の前提）」と照合する。載っていない事実は owner へ 1 問にして送る」の 1 行を足す（Plan Review も code_review に従うため、Plan Review の観点を兼ねる）。
- S7 `docs/templates/pr-review-prompt.md` を削除する。**着手条件**: PR2 の closeout が origin/main に入り、`rg -n 'pr-review-prompt' docs/AGENT_OPERATING_MANUAL.md` が 0 件の origin/main を本 branch へ 1 回 merge した後（MANUAL `:124` が markdown link で参照しており、先に消すと link 検査が落ちる。MANUAL は PR2 の所有で、`:124` の link は PR2 が外す。本 PR は MANUAL を触らない。同期の時点を「helper と PK5 の前提」の closeout 後の同期と揃え、同期を 1 回にする）。条件を満たさない時点で Writer はこの削除だけを残して Coordinator へ返す。
- S8 `docs/quality/review-checklist.md` `## 運用ルール`（`:5-12`）: `:8`「下記9カテゴリと設計判断レンズのみを観点とする。カテゴリ外の指摘は…今回は対象外」を「9 カテゴリと設計判断レンズ、および `docs/code_review.md` の Blocking Review Focus を観点とし、カテゴリ外の指摘も severity を付けて返す」にする。`:9`「各カテゴリで最大3件」を消し、「見つけた指摘は件数で落とさない」にする。`:10-11`（GitHub 投稿と投稿境界）を「PR への指摘投稿と、それ以外の PR 操作の承認は `AGENTS.md` Decision and Approval Boundaries」の 1 行にする。`:12`「再レビュー: 新規観点の追加は禁止」を「再レビュー（closure）は `docs/DEV_WORKFLOW.md` Review Rules の Findings Freeze に従う」にする。`## 観点外`（`:109-113`）の説明文も同じ向きに替える（カテゴリ外の指摘は今回 severity を付けて返し、この節にはカテゴリの追加候補だけをメモする。見出しと `- （なし）` は残す）。`:3` の「観点の後出し追加を防ぐ」は Findings Freeze への参照と矛盾しない範囲で残す。9 カテゴリと設計判断レンズは変えない（DS2 / DS4 の checker が参照）。
- S9 `.github/pull_request_template.md`: `:18`（`Evidence Mode: github` 行）と `:32`（helper・直接 UI merge・legacy bootstrap の段落）を消す。`## Review-only`（`:38-41`）は PR4（review-only と Final Review の統合）まで残す。
- S10 `docs/project-profile.md` の縮約: `## Source of Truth`（`:15-38`）、`## Test Commands`（`:141-168`）、`## Review Commands / Tools`（`:170-185`）、`## Current Workflow State`（`:244-252`）、`## Open Questions`（`:254-258`）、`## Suggested Follow-up`（`:260-264`）を削除。`## Workflow Notes` の `:236`（Evidence Mode）と `:237`（hook の記述、正本は MANUAL と CLAUDE）を削除。`## Project Type` / `## Outputs / Artifacts` / `## Stable Contracts` / `## High-risk Changes` / `## Data Safety Boundary` / `## Risk Level Examples` / `## Test Design Focus` と Workflow Notes の他の行は変えない（`:231` の review-only は PR4）。削除する節の command は `docs/DEV_WORKFLOW.md` Verification Gates の表（`:309-321`）と `AGENTS.md:36`（docker）が持つことを確認済み（`generate:routes` は template の Registration 表）。`## Review Commands / Tools`（`:172-185`）の 14 項目の行き先: 読書経路・設計正本・checklist・code_review の案内（`:172-176`）は `AGENTS.md` `Session Start` の初回レビュー行（`:15`）と `inventory-code-review` の `## Start Here`、`$inventory-code-review` の案内（`:177`）は `inventory-workflow-start` の `## Handoff`（`:20`、S5 の後も `inventory-code-review` は残る）、command の 5 項目（`:178-182`）は Verification Gates の表、drift の `rg`（`:183`）と Windows native L3 の記録（`:185`）は `docs/code_review.md` `## Verification Rules`（`:42-43`・`:49`）が既に持つ。他の正本に無い data safety の `git status --short`（`:184`）だけを S6 で code_review の `## Verification Rules` へ 1 行で移す。
- S11 `docs/PROJECT_HANDOFF.md:17`: 「（legacyは旧PR本文）」を消す。
- S12 `docs/TOOLING_SKILL_COMMANDS.md`（削除に伴う参照の是正だけ）: §3 の参照元（`:57-60`）と使い方の例（`:63-66`）から削除した 3 command を消し `plan-rally.md` を足す。`/test` / `/design-review` / `/phase-complete` の小節（`:75-89`）を消す。§6 の `:163` を「レビュー時は `/check` を使う」に、`:165` の項目 4 を消す。他の節（監査 P12 の DEV_SETUP への吸収）は範囲外。
- S13 test（入口の文面を書き換える PR として要求先を合わせる）
  - `scripts/tests/reading-order-drift.test.sh:158`: 直接 UI merge / 残存リスクを要求する file を `AGENTS.md` と `docs/DEV_WORKFLOW.md` の 2 本にする。`:162-164`（Skill 5 本への `Evidence Mode` の要求）を消す。`:157` の SPEC コメントと `:165` の PASS 行は残す。
  - `scripts/tests/codex-safe-wrappers.test.sh:365-370`: T13 の `namespace_files` から `CLAUDE.md` を外す（S2 で namespace を書かなくなるため。配列が空になるなら loop ごと消す）。T12（`:345-363`）の旧 path・旧 namespace の拒否は `CLAUDE.md` を含めたまま残し、T12 の DEV_SETUP 側（`:371-373`）と T13 の DEV_SETUP 側（`:375-376`）も残す。
- S14 `docs/decision-log.md` の末尾に `## D-093` を追記する（番号は予約済み。D-091 は PR #98、D-092 は PR2）: 入口文書の単一所有。(1) npm 供給網ガード（D-030）の行動規則の所在を `CLAUDE.md` から `AGENTS.md` へ移す（D-030 の Impact の「CLAUDE.md の節」を部分的に置き換える。規則の中身は変えない）。`CLAUDE.md` は `@AGENTS.md` の import で `AGENTS.md` を Claude の session に読み込ませる（`CLAUDE.md` があると Claude Code は `AGENTS.md` を既定では読まないため）。(2) 問い合わせの行き先の所有を `AGENTS.md` Decision and Approval Boundaries にする（D-090 の SPEC-WF-LIGHT1A-D4 を部分的に置き換える。`docs/DEV_WORKFLOW.md` の表は PR4 が link に置き換える）。(3) review の依頼・出力の規範（依頼形式・重大度・出力形式・確信度）を `docs/code_review.md` に統合し、`pr-review` / `review-only-subagent` Skill と `pr-review-prompt` template を廃止する。併用する review Skill（`inventory-code-review`・`engineering-review`）は重大度・出力形式を code_review に従わせ、`engineering-review` の SSOT の主張は review 哲学に限る。subagent 発注の入力欄は `docs/templates/subagent-review-packet.md`（D-092）が持ち、規範を複製しない。`pr-review-prompt` は本 PR で code_review へ統合済みで、PR4 での移入は要らない。(4) Final Review の固定観点「保守者として読めるか」（owner 2026-09-07）と店の事実の照合を code_review へ置く。
- 登録（plan-first commit に同乗）: `docs/Plans.md` の「次の行動」に本 packet と Matrix の link を 1 行足す。

対象を使う呼出し側・隣接 test を確認した（下記「削除する file と参照元」）。予期しない拡張は既存の改訂経路へ戻し、「関連 file 全般」を許可範囲にしない。

### 削除する file と参照元（branch の HEAD で `rg` 実測）

command: `rg -n --hidden -g '!.git' -g '!docs/archive/**' -g '!docs/decision-log.md' -g '!docs/plans/**' 'pr-review-prompt|review-only-subagent|\bpr-review\b|rules/commands\.md|review-workflow\.md|commands/(test|design-review|phase-complete)\.md|/design-review|/phase-complete|`/test`' .`（17 行）

archive を含む link の検索（link 検査は `docs/archive/**` も走査する）: `rg -n '\]\([^)]*(pr-review-prompt\.md|review-only-subagent/|pr-review/|rules/commands\.md|review-workflow\.md|commands/(test|design-review|phase-complete)\.md)' docs`（2 行: MANUAL `:124`、archive `2026-09-15-display-fixes-batch-2.md:22`）

| 削除する file | 参照元（live） | 処置 |
| --- | --- | --- |
| `.agents/skills/pr-review/SKILL.md` | `.agents/skills/inventory-workflow-start/SKILL.md:20`、`scripts/tests/reading-order-drift.test.sh:162` | S5、S13 |
| `.agents/skills/review-only-subagent/SKILL.md` | `scripts/tests/reading-order-drift.test.sh:162` | S13 |
| `docs/templates/pr-review-prompt.md` | `docs/AGENT_OPERATING_MANUAL.md:124`（markdown link、PR2 の所有） | S7 の着手条件（PR2 が消す） |
| `.claude/rules/commands.md` | `scripts/tests/classify-changes.test.sh:46`（classifier へ渡す path の文字列で、file の実在を要求しない） | 変えない（AC2 の除外） |
| `.claude/rules/review-workflow.md` | `docs/archive/plans/2026-09-15-display-fixes-batch-2.md:22`（markdown link） | S3（link を code span に替える 1 行） |
| `.claude/commands/test.md`・`design-review.md`・`phase-complete.md` | `docs/TOOLING_SKILL_COMMANDS.md:58-60,64-66,75,79,83,163,165` | S12 |

`.claude/skills` に上の Skill の symlink は無い（`ls -la .claude/skills`）。`.agents/skills/*/agents/openai.yaml` を持つのは `inventory-implementation` / `inventory-operator-ui` / `inventory-workflow-start` だけで、削除する 2 本は持たない。`scripts/tests/claude-hooks.test.sh` は `CLAUDE.md` と `.claude/commands/plan-rally.md`（残す）だけを読む。`scripts/doc-consistency-check.sh` の link 検査（R3、`:1676-1720`）は anchor を外して file の実在だけを見る（`:1711`）。

### PR2 との境界と merge 順

- file の所有: `CLAUDE.md` と `.agents/skills/**`（S5。削除する 2 本と、`inventory-code-review`・`engineering-review` の重大度・出力形式の従属を含む）は本 PR、座組表（`docs/AGENT_OPERATING_MANUAL.md` の `## 座組`）と MANUAL 全体・`docs/templates/subagent-review-packet.md`・`docs/agent-guidance/**`（merge-evidence 以外）は PR2。`docs/DEV_WORKFLOW.md` は本 PR では触らない（PR2 と PR4 が節ごとに持つ）。両 PR が足す行は `docs/Plans.md` の「次の行動」（各 1 行）と `docs/decision-log.md` の末尾（D-092 と D-093）だけで、後の merge では両方を残す（decision-log は番号順に D-092 → D-093）。
- PR2 の範囲（owner 決定 2026-09-25）: PR2 は MANUAL の全面的な縮約をやめ、既存の見出しと節番号を変えず、`## 座組` を `### 3.5` の後・`## 4.` の前に H2 で新設し（PR2 の AC1 `^## 座組$`、slug `#座組`）、退役した仕組みを指示する文とそれに伴う行（MANUAL `:124` の link を含む）を直す。本 PR が依存する MANUAL の anchor は `#座組` だけで、既存の anchor（例: `.agents/skills/inventory-workflow-start/SKILL.md:13` の §3 の anchor）は PR2 の前後で有効のまま。
- anchor: `CLAUDE.md`（S2）と `AGENTS.md`（S1 `:21`）から座組表への link は `docs/AGENT_OPERATING_MANUAL.md#座組` に固定する。
- **merge 順 = PR2 を先に merge する**（本文で「MANUAL の座組」と書いて link を張らない案は採らない）。理由: (1) S7 の `pr-review-prompt.md` の削除は MANUAL `:124` の markdown link が消えていることを要し、link 検査（file の実在）が本 PR 単独では落ちる。link を張らない案を選んでもこの依存は残る。(2) link 検査は anchor を見ないため、link を張っても検査は PR2 の前後どちらでも通るが、PR2 より先に入ると `#座組` が実在しない見出しを指す期間ができる。PR2 先ならその期間が無い。(3) decision-log の番号順（D-092 → D-093）と追記位置が一致する。
- PR2 に求めること（Coordinator が PR2 の起草役へ伝える。PR2 の所有 file なので本 PR は編集しない）: MANUAL `:124`（pr-review-prompt の link を PR2 で外す。これで本 PR の S7 の着手条件 `rg -n 'pr-review-prompt' docs/AGENT_OPERATING_MANUAL.md` = 0 がそのまま成立し、本 PR は MANUAL を触らない）。`docs/agent-guidance/model-notes.md:9` の `CLAUDE.md#sonnet--opus-の-effort` への link を座組表へ向ける（本 PR がその見出しを消すため。anchor は検査されないので黙って死ぬ。AC11 の merge 直前の rg で確かめる）。`docs/templates/subagent-review-packet.md:26`、`docs/agent-guidance/README.md:8`、`docs/agent-guidance/shared.md:9` の legacy / Evidence Mode の文は PR2 が消す（PR1 の Non-scope が PR3 と書いたが、所有に合わせ PR2 へ移す）。`docs/templates/subagent-review-packet.md` の `## Output` の出力の規範（確信度を含む）を外して `docs/code_review.md` を参照にする（確信度は本 PR の S6 が code_review へ足す）。PR2 が MANUAL に置く文で `docs/code_review.md` を指す箇所は、`docs/code_review.md:38` のような行番号でなく節名（`## Verification Rules` の closure の文）で参照する（本 PR の S6 が code_review に行を足し、行番号は merge 後に別の行を指す。link 検査も AC11 も行番号のずれを拾わない）。
- 本 PR の Writer は PR2 の merge を待たずに S1〜S6・S8〜S14 を進め、S7 だけを着手条件の後に行う。PR2 の closeout が origin/main に入った後、本 branch へ origin/main を 1 回 merge する（DEV_WORKFLOW Wave Operation の単段 merge。`Plan Commit`・`Amendments`・Phase は変えない）。conflict は `docs/Plans.md` と `docs/decision-log.md` の追記だけのはずで、両方を残して解消する。同期の後に AC11 の merge 直前の条件を満たさなければ、Writer は Coordinator へ返す（PR2 の所有 file の是正は PR2 側で行う）。
- PR2 が止まった場合は、本 PR も merge を待つ（Plan Review・実装は進めてよい）。本 PR の範囲を削って PR2 抜きで merge する繰延はしない。

### helper と PK5 の前提（capture の前に満たす）

- `scripts/pr-gate.py:231` は PR head の `docs/plans/` に active packet がちょうど 1 つであることを要求する。base `e7c22f8f` の main には `docs/plans/` の packet が無く（`git ls-tree --name-only e7c22f8f docs/plans/` が空）、active な他 packet は並走 PR2 の branch にだけある。PR2 の packet は PR2 の merge から closeout まで main に残るので、本 PR の capture の前に PR2 の closeout が origin/main に入り、それを本 branch へ merge してあることを確認する。
- branch の HEAD で `bash scripts/check-workflow-git.sh` は exit 0（下記 AC12 の baseline。plan-approved で Plan Commit を設定した後は本 packet も PK5 の対象で、`check-workflow-git.sh` は exit 0）。

## Non-scope

- `docs/DEV_WORKFLOW.md` の全節（`### 問い合わせの行き先` の表を AGENTS への link に置き換えること、Review Rules の review-only・3 分類・Findings Freeze、Artifact Map、Implementation Rules を含む）: PR4（Review Rules の D-062(c) 行・Subagent Budget・`:214-215` は PR2）。PR4 までは同じ行き先が DEV_WORKFLOW の表と AGENTS に並存するが、表の注記 (b) が AGENTS を上位とするため矛盾しない。
- `docs/AGENT_OPERATING_MANUAL.md`、`docs/templates/subagent-review-packet.md`、`docs/agent-guidance/**`、decision-log の D-092: PR2。
- merge の順: PR2 merge → PR2 の closeout → PR3 に main を同期 → PR3 の Final Review → Ready。helper は PR head の active packet が 1 つであることを要求する（`scripts/pr-gate.py:231`）。
- `docs/templates/plan-packet.md`（`:196` の撤去済み Phase 名 `independent-review` を含む。監査 §7 は template 全体を PR4 の所有とし、backlog の「撤去済みの Phase 名 `independent-review` が 2 か所に残る」entry は PR2 で直すと書く。どちらにしても本 PR ではない）、`docs/templates/test-design-matrix.md:101`、`docs/templates/workflow-effectiveness-review.md:78`、`test-design` Skill の重複行、`workflow-effectiveness-review` Skill: PR4。
- `.github/pull_request_template.md` の `## Review-only`、`docs/code_review.md` の `## Review-only Sub-agent Protocol`、`docs/project-profile.md:231`: review-only と Final Review の統合として PR4。
- `.claude/settings.json`、`.claude/hooks`、`.claude/skills`（sandbox で書込み不可。hook inventory 0 本は変えない）、`.claude/agents/**` の新設（PR5）。
- classifier の穴（`.claude/agents/**` と `docs/quality/review-checklist.md` が policy docs に入らない、監査 V2）: PR5。本 PR の `review-checklist.md` の変更は classifier 上 docs だが、本 PR 全体が workflow=true なので Minimum 2 は掛かる。
- `docs/TOOLING_SKILL_COMMANDS.md` の S12 以外（監査 P12 の DEV_SETUP への吸収と file の削除）、`.codex/**`（`AGENTS.md` `## Workspace Access` と `.codex/README.md` の重複、監査 O4）、`docs/DEV_SETUP_CHECKLIST.md`。
- Draft PR #81（`chore/repo-path-lowercase`）の小文字化: 本 PR は `CLAUDE.md` から memory path の文を消し T13 の `CLAUDE.md` 要求を外すだけで、DEV_SETUP・`.codex/**` の path は #81 に残す。#81 を後で進める場合、`CLAUDE.md` と T13 の hunk は本 PR の形に合わせて解消する。
- `docs/backlog.md`: 実装 PR では編集しない。closeout で次の entry を close する（行番号は動くので entry の文で指す）: 「問い合わせの行き先の表の行 7 に `AGENTS.md` が別の明示承認とする PR 操作が挙がっていない」、「Codex review-only 発注向けの Session Start fast path」、「Final Review の固定観点「保守者として読めるか」の docs 化」、「店の事実を前提にする判断の前に `docs/project-memory.md` の「現場の前提」と照合する観点」。「workflow の軽量化 3 段」entry の 3 段目（重複文書の統合）に本 PR の分を記録し、`pr-review-prompt` は本 PR で code_review へ統合済みで PR4 の移入は要らないことを残す。
- S10 で消す `docs/project-profile.md` の `## Open Questions` の未決の問い 2 件は closeout で backlog へ移す: `:256`（既存の research ADR を `docs/adr/` へ物理的に移すか、ADR index からの link のままにするか）と `:257`（tracked の POS sample / reference data を synthetic-only にするかという data safety の問い）。
- `.codex/README.md:246` の Evidence Mode・legacy の句（`github mode`、`legacyのstate-only/三点一致`）: 本 PR の入口の範囲外（`.codex/**` は上の O4 と同じく対象外）。follow-up として closeout で backlog へ 1 行記録する。
- `.agents/skills/inventory-workflow-start/SKILL.md:13`（MANUAL の §3 の anchor と語「Capacity-degraded」）: PR2 が MANUAL の見出しを変えないので有効のまま。本 PR でも PR2 でも変えない。
- `docs/archive/**`（例外: S3 の `docs/archive/plans/2026-09-15-display-fixes-batch-2.md:22` の link を code span に替える 1 行だけ）と decision-log の既存 entry の本文。
- `.local/` の Codex 発注書と `run.sh`（tracked 外）: 削除する Skill・template を参照していないかは発注前に Coordinator が確認する。

## Acceptance Criteria

baseline は本 branch の HEAD（base に本 packet・Matrix・`Plans.md` の登録行を足しただけ）で同じ command を実行した実測。起票時は base `85b18a04` で 2026-09-25 に実測し、2026-09-27 に HEAD `7b41ecb1`（base `e7c22f8f`）で AC1〜AC3・AC11 の既存の command を再実行して値が同じことを確かめ、round 2 の是正で足した command（AC4 の 3 本目、AC5 の `closeout` と `裁定権者`、AC8 の後半、AC11 の import と anchor）を実測した。round 3 の是正で足した command（AC1 の `github mode`、AC2 の plan-rally、AC3 の D-093、AC5 の `label`・`issue comment` の 2 以上、AC6 の `残存リスク` の mutation、AC8 の見出し・検査項目・`git status --short`・両 Skill、AC11 の AGENTS の `#座組` と `frontier`）は 2026-09-27 に HEAD `fb1cc6bf`（base `e7c22f8f`）で実測した。

- AC1（定型文と legacy の撤去、D2）: `rg -n -i 'Evidence Mode|github mode|legacy|三点一致|state-only|exact-HEAD|human-confirm|ready-hosted-final' AGENTS.md CLAUDE.md .claude/rules .claude/commands .agents/skills/inventory-workflow-start .agents/skills/inventory-implementation .agents/skills/inventory-code-review docs/code_review.md docs/project-profile.md .github/pull_request_template.md docs/PROJECT_HANDOFF.md docs/quality/review-checklist.md` が 0 行（exit 1）。`-i` は `AGENTS.md:27` の「GitHub evidence mode」を拾うため。baseline（同じ command）: 17 file・23 行（AGENTS 2〈`:14`・`:27`〉、CLAUDE 1、rules 3、commands 5、workflow-start 1、implementation 2、code-review 3、code_review 1、project-profile 2、PR template 2、HANDOFF 1）。後半 2 語は AGENTS `:14` と inventory-implementation `:20` の 2 行で、前半 5 語と同じ行（`-i` なしでは 22 行）。`github mode`（退役した mode の語）の 15 行はどれも `Evidence Mode` と同じ行で、足しても 23 行のまま（round 3 で実測）。
- AC2（削除と参照、D3 / D4）: `git ls-files .agents/skills/pr-review .agents/skills/review-only-subagent docs/templates/pr-review-prompt.md .claude/rules/commands.md .claude/rules/review-workflow.md .claude/commands/test.md .claude/commands/design-review.md .claude/commands/phase-complete.md` が空。S7 の同期後の HEAD で、上の「削除する file と参照元」の command（`` `/test` `` を含む）の一致が `scripts/tests/classify-changes.test.sh:46` の 1 行だけになる（baseline 17 行。`docs/plans/**` と decision-log は command が除外する）。同じ HEAD で archive を含む link の検索（同じ節の 2 つ目の command）が 0 行（baseline 2 行）。残す `plan-rally` の是正（S4）: `rg -n 'Subagent Budget|consultation relay|予算上限' .claude/commands/plan-rally.md` が 0 行（exit 1。baseline 3 行: `:8`・`:18`・`:23`）。
- AC3（npm ガードの移設、D1）: `rg -c 'min-release-age-exclude|ignore-scripts=true|--save-exact|npm audit fix --force' AGENTS.md` が 4、同じ command の `CLAUDE.md` が 0（exit 1）。baseline: AGENTS 0、CLAUDE 4。`rg -n '^## npm 供給網ガード（D-030）' AGENTS.md` が 1 行。所在の移動の記録（S14）: `rg -c '^## D-093' docs/decision-log.md` が 1（baseline 一致なし、exit 1）。
- AC4（Session Start、D8 / D2）: `bash scripts/tests/reading-order-drift.test.sh` が exit 0 で `PASS: reading-order-drift` と `PASS: merge evidence entry routing` を出す（check_entry_contract が S1 の書き換え後の R2+ 行と fail-closed を受理する）。`AGENTS.md` の `## Session Start` 節の初回レビュー行が、指定範囲から始めることと、範囲に関わらず設計正本を読むことを持つ: `awk '/^## Session Start/,/^## Working Rules/' AGENTS.md | rg -n '初回レビュー.*範囲から始め'` が 1 行、同じ出力への `rg -n '初回レビュー.*発注の範囲に関わらず'` が 1 行、`rg -n '初回レビュー.*重大度と出力形式は範囲に関わらず'` が 1 行、`rg -c '範囲だけ'` が一致なし（baseline: 0 行、0 行、0 行〈exit 1〉、一致なし）。
- AC5（問い合わせの行き先、D1）: `awk '/^## Decision and Approval Boundaries/,/^## Workspace Access/' AGENTS.md` の出力に、`店の事実`、`受容リスク`、`Windows L3`、`label`、`issue comment`、`技術的な正しさ`、`伝言`、`委任`、`closeout`、`裁定権者` がそれぞれ 1 回以上ある（10 語の `rg -c` がすべて 1 以上）。ただし `label` と `issue comment` は既存の `:45` が持つため、S1 (i) の項目で増えたことを見る: 同じ出力への `rg -c 'label'` と `rg -c 'issue comment'` がそれぞれ 2 以上。baseline（同じ command を語ごとに実行）: `label` 1、`issue comment` 1（どちらも `:45`）、他の 8 語は一致なし。
- AC6（test の要求先、D7）: AC4 に加え、mutation で要求が残っていることを確かめる。写しの作り方（AC7 も同じ）: 変更を commit した後、`copy="$TMPDIR/pr3-mut"; mkdir -p "$copy"; git archive HEAD | tar -x -C "$copy"; git -C "$copy" init -q; git -C "$copy" add -A`（test は `SOURCE_ROOT` を自分の位置から決め、`git ls-files` を使うため写しに index が要る。本 repo の git 設定・index は触らない）。改変ごとに写しを作り直し、終わったら消す。写しの `AGENTS.md` から `直接UI merge` を含む行を消すと `FAIL: helper-only merge boundary missing: AGENTS.md`、写しの `docs/DEV_WORKFLOW.md` で同じ操作をすると同じく FAIL（どちらも exit 1）。`残存リスク` 側は別の診断になる: 写しの `docs/DEV_WORKFLOW.md` から `残存リスク` を含む行（`:107`）を消すと `FAIL: server enforcement limitation missing: docs/DEV_WORKFLOW.md`、写しの `AGENTS.md` では `直接UI merge` と同じ行（`:27`）にあるため語だけを置き換える（`sed -i 's/残存リスク/residual/'`）と `FAIL: server enforcement limitation missing: AGENTS.md`（どちらも exit 1）。写しの `AGENTS.md` の R2+ 行を消すと `entry contract is incomplete`。`CLAUDE.md` と PR template から同じ行を消しても FAIL しないこと、Skill の `Evidence Mode` の語が無くても FAIL しないことを確認する。
- AC7（T12 / T13、D7）: `bash scripts/tests/codex-safe-wrappers.test.sh` が exit 0。mutation: 写しの `CLAUDE.md` に `-home-kosei-Projects-inventory-system/` を 1 行足すと T12 の FAIL（`history-view encoded namespace`）で exit 1。
- AC8（review の統合と観点、D4 / D5）: `rg -n '保守者' docs/code_review.md` と `rg -n '現場の前提' docs/code_review.md` がそれぞれ 1 行以上、`rg -n 'non-scope' docs/code_review.md` が 1 行以上（pr-review-prompt から移した「明示された non-scope で block しない」）、`rg -n 'Branch|Commit' docs/code_review.md` が 1 行以上（依頼で渡す PR context）、`rg -n '確信度' docs/code_review.md` が 1 行以上（review packet から移す出力の要求）、`rg -n '^\| P3 \|.*保守者' docs/code_review.md` が 1 行（Severity 表の P3 の注記）、`rg -n 'unresolved questions' docs/code_review.md` が 1 行以上（pr-review-prompt `:56` から移す句）。Skill の出力形式: `rg -n '重大な findings なし' .agents/skills/inventory-code-review/SKILL.md` が 0 行（exit 1）、`awk '/^## Output Format/,/^## Review Biases/' .agents/skills/inventory-code-review/SKILL.md | rg -c 'code_review\.md'` が 1 以上。併用 Skill の重大度・出力形式（S5）: `rg -n '\*\*(nit:|Blocking|optional:)\*\*|Blocking / P1 / nit|^## nit / blocking' .agents/skills/engineering-review/SKILL.md` が 0 行（exit 1）、`rg -n 'nit/blocking' .agents/skills/engineering-review/SKILL.md .agents/skills/inventory-code-review/SKILL.md` が 0 行（exit 1）、`rg -c 'code_review\.md' .agents/skills/engineering-review/SKILL.md` が 3 以上（責務境界・step 4・label の節）、`awk '/^# Inventory Code Review/,/^## Review Goals/' .agents/skills/inventory-code-review/SKILL.md | rg -c 'code_review\.md'` が 1 以上（`:9` の 1 句）。code_review の節と検査項目（S6）: `rg -c '^## (Output Shape|Finding Severity)$' docs/code_review.md` が 2 のまま、`rg -n 'head/base.*broad/closure.*manual/R4.*実効 ?rules' docs/code_review.md` が 1 行のまま（`:36` の書換えで検査項目を落とさない）、`awk '/^## Verification Rules/,/^## Same PR vs Follow-up/' docs/code_review.md | rg -c 'git status --short'` が 1 以上。baseline（round 3 で足した分）: engineering-review の label 定義 5 行（`:36`・`:76`・`:80`・`:81`・`:82`）、`nit/blocking` 2 行（engineering-review `:21`、inventory-code-review `:9`）、engineering-review の `code_review\.md` は一致なし、inventory-code-review 冒頭の `code_review\.md` は一致なし、見出し 2、検査項目 1 行（`:36`）、`git status --short` は一致なし。round 2 までの baseline: `保守者` 0、`現場の前提` 0、`non-scope` 0、`Branch|Commit` 0、`確信度` 0、P3 の注記 0、`unresolved questions` 0（exit 1）、`重大な findings なし` 1 行（`:73`）、Output Format 節の `code_review\.md` は一致なし（exit 1）。
- AC9（review-checklist、D5）: `rg -n '最大3件|新規観点の追加は禁止' docs/quality/review-checklist.md` が 0 行（exit 1。baseline 2 行）、`rg -n '今回は対象外' docs/quality/review-checklist.md` が 0 行（baseline 1 行: `:8`）、`rg -n 'Findings Freeze' docs/quality/review-checklist.md` と `rg -n 'AGENTS.md' docs/quality/review-checklist.md` がそれぞれ 1 行以上。`rg -c '^### [1-9]\. ' docs/quality/review-checklist.md` が 9 のまま（baseline 9）。`bash scripts/doc-consistency-check.sh` の DS2 / DS4 が新しい WARN を出さない。
- AC10（project-profile、D6）: `rg -c '^## (Source of Truth|Test Commands|Review Commands / Tools|Current Workflow State|Open Questions|Suggested Follow-up)$' docs/project-profile.md` が 0（exit 1）、`rg -c '^## (Project Type|Outputs / Artifacts|Stable Contracts|High-risk Changes|Data Safety Boundary|Risk Level Examples|Test Design Focus|Workflow Notes)$' docs/project-profile.md` が 8。baseline: 前者 6、後者 8。`rg -n 'Test/workflow gates that affect what may be merged' docs/project-profile.md` が 1 行のまま。
- AC11（CLAUDE.md と PR2 の境界、D1 / D9）: `rg -c 'AGENT_OPERATING_MANUAL.md#座組' CLAUDE.md` が 1 以上、同じ command の `AGENTS.md` が 1 以上（S1 `:21`。baseline 一致なし、exit 1）、`rg -n 'frontier' AGENTS.md` が 0 行（exit 1。baseline 1 行: `:21`）、`rg -n 'xhigh|Sonnet 5|prompting-claude-opus-5|Projects-inventory|Evidence Mode' CLAUDE.md` が 0 行（exit 1。npm ガードの 4 語が 0 であることは AC3 が見る。`:42` の参照文が「npm」の語を含むのは可）、`rg -n 'hook' CLAUDE.md` が 1 行以上（hook inventory の記述は残す）。`rg -c '^@AGENTS\.md$' CLAUDE.md` が 1（import が backtick の外の行頭に 1 回。baseline 一致なし、exit 1）。merge 直前の HEAD（PR2 の closeout を含む origin/main の同期後）で `rg -n '^## 座組$' docs/AGENT_OPERATING_MANUAL.md` が 1 行、`rg -n 'pr-review-prompt' docs/AGENT_OPERATING_MANUAL.md` が 0 行、`rg -n 'CLAUDE\.md#' -g '!docs/archive/**' -g '!docs/plans/**' -g '!docs/decision-log.md' AGENTS.md docs .agents .claude .codex scripts` が 0 行（本 PR が消す `CLAUDE.md` の見出しを指す anchor が残らない。baseline 1 行: `docs/agent-guidance/model-notes.md:9`、PR2 が直す）。
- AC12（検査の全体）: `bash scripts/doc-consistency-check.sh`（link 検査 R3 を含む）と `bash scripts/doc-consistency-check.sh --target plan` が exit 0、`bash scripts/tests/run-workflow-tests.sh` が exit 0（claude-hooks・classify-changes を含む workflow suite）、`bash scripts/local-ci.sh full` が exit 0。`bash scripts/doc-consistency-check.sh --target plan docs/archive/plans/2026-09-15-display-fixes-batch-2.md`（変えた archive file を explicit path で。baseline exit 0、既存の PK6 WARN 1 件）が exit 0。`bash scripts/check-workflow-git.sh` が exit 0（baseline exit 0。Plan Commit の設定後は本 packet も PK5 の対象で、exit 0）。`git diff --check` が exit 0。
- AC13（範囲、S 全体）: `git diff --name-status origin/main...HEAD`（S7 の同期後）の変更 file が Scope の S1〜S14（S3 の archive `docs/archive/plans/2026-09-15-display-fixes-batch-2.md` の 1 行を含む）と Plans.md の登録行・本 packet・Matrix に限られる。`scripts/pr-gate.py`、`scripts/ci/**`、`scripts/doc-consistency-check.sh`、`scripts/check-workflow-git.sh`、`.github/workflows/**`、`docs/DEV_WORKFLOW.md`、`docs/AGENT_OPERATING_MANUAL.md`、`docs/templates/subagent-review-packet.md`、`docs/templates/plan-packet.md`、`.claude/settings.json` に本 PR 由来の差分が無い（PR2 の merge で入った差分は除く）。

## Design Sources

- Requirements / spec: 該当なし（製品要件を変えない）。
- Architecture / Function / DB / Screen: 該当なし。
- Workflow 正本（本 PR が書き換える入口と、参照だけする正本）: `AGENTS.md`、`CLAUDE.md`、`docs/DEV_WORKFLOW.md`（Workflow State・Review Rules・`### 問い合わせの行き先`、変更しない）、`docs/code_review.md`、`docs/quality/review-checklist.md`、`docs/project-profile.md`。
- Decision log / ADR: D-030（npm ガード。Impact の所在を D-093 で部分的に置き換える）、D-034（Session Start が唯一の入口、drift test）、D-059（hook inventory 0 本）、D-085（直接 UI merge 禁止・helper 経由）、D-090（Ordinary Operation と問い合わせの行き先。SPEC-WF-LIGHT1A-D4 の所有を D-093 で部分的に置き換える）。
- 監査記録（local-only、要点は本 packet に書き下した）: §3.A A1、§3.B B7、§3.I I6、§3.K K11〜K14、§3.N N1〜N6、§3.P P3 / P4 / P8 / P11、§4、§5、§7 PR3 行、§8 の 1・4・7・8。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | existing sufficient（製品コードを変えない） |
| Command / DTO / generated binding / wire shape | なし | existing sufficient |
| DB / transaction / audit / rollback / migration | なし | existing sufficient |
| Screen / UI / route state / Japanese wording | なし | existing sufficient |
| CSV / TSV / report / import / export format | なし | existing sufficient |
| Durable decision / ADR | `docs/decision-log.md` D-093 | updated in this PR（S14） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| source / workflow doc の削除（Skill 2 本、template 1 本、rules 2 本、commands 3 本） | 参照元の是正（上の「削除する file と参照元」、S3 の archive の 1 行、S5 / S12 / S13 と S7 の着手条件）。親文書の索引: `docs/DEV_WORKFLOW.md` Artifact Map の Workflow Skills 行（`:37`）は削除する Skill を列挙していない。MANUAL `:124` は PR2 |
| decision-log の追記 | D-093（予約済み番号）。PR2 の D-092 の後に置く |
| archive の file の変更（`docs/archive/plans/2026-09-15-display-fixes-batch-2.md:22`） | DEV_WORKFLOW Plan Packet Rules の「when changing archived plans, check the changed archive files by explicit path」に従い、`bash scripts/doc-consistency-check.sh --target plan docs/archive/plans/2026-09-15-display-fixes-batch-2.md` を実行する（AC12） |

Tauri command・function-design doc・REQ・route・operator 画面: 該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-HARNESS3 | AGENTS Safety / CLAUDE、D-030 | D1 | vendor 中立の入口に安全境界と承認の行き先を置く（Codex は CLAUDE.md を読まない）。Claude には `CLAUDE.md` の `@AGENTS.md` の import で同じ AGENTS を読み込ませる。却下: CLAUDE.md と AGENTS の両方に置く（重複、監査 §5）、CLAUDE.md に「AGENTS を読め」と言葉で書くだけにする（Claude が開くかは判断次第で、公式文書も import を勧める） | S1、S2、S14 | AC3、AC5、AC11 |
| SPEC-WF-HARNESS3 | 入口の定型文（PR #54 で配布） | D2 | 旧 mode が消えたので「新旧を混ぜない」ための定型文は意味を失った。却下: 定型文を github だけの文に書き換えて残す（13 箇所の重複が残る） | S1〜S5、S9、S11 | AC1 |
| SPEC-WF-HARNESS3 | `.claude/rules`・`.claude/commands`・workflow Skill | D3 | 正本へ従えと言うだけの wrapper は読む量を増やすだけ。却下: 全部残して定型文だけ消す（監査 N5 / N6 / P4） | S3、S4、S5、S12 | AC2 |
| SPEC-WF-HARNESS3 | `docs/code_review.md`、pr-review-prompt、subagent-review-packet | D4 | review の依頼・出力の規範（重大度・出力形式・確信度）の正本を code_review 1 つにし、subagent 発注の入力欄は review packet（PR2）が持つ。`inventory-code-review` Skill の出力形式も code_review を参照し、`inventory-code-review:9` が併用を指示する `engineering-review` Skill も報告順と label を code_review の P1 / P2 / P3 に従わせる。却下: review packet や Skill に出力形式を残す（2 つの規範を読むことになり、指摘なしの返し方が `重大な findings なし` と `No blocking findings.` に割れている。`engineering-review` の `Blocking / P1 / nit` の報告順と label 定義も別の重大度になる）、規範ごと review packet へ統合（PR2 の所有で、外部 reviewer への依頼も code_review が持つ） | S5、S6、S7 | AC2、AC8 |
| SPEC-WF-HARNESS3 | code_review、review-checklist、backlog の保守者観点と「現場の前提」の照合の entry | D5 | 件数上限と「新規観点禁止」は subagent-review-packet の「黙って落とさない」と Findings Freeze の例外と矛盾する（監査 §8-7）。保守者観点は owner 2026-09-07、店の事実の照合は 2026-09-19 の実発生 | S6、S8 | AC8、AC9 |
| SPEC-WF-HARNESS3 | project-profile | D6 | 他の正本と重複する節を消し、この repo 固有の High-risk / Data Safety / Stable Contracts / Test Focus を残す | S10 | AC10 |
| SPEC-WF-HARNESS3 | drift test（D-034 / D-085）、T12 / T13 | D7 | 入口の文面を書き換える PR が要求先を合わせる（PR1 の裁定）。直接 UI merge 禁止は境界（AGENTS）と手順（DEV_WORKFLOW）の 2 箇所で足りる。却下: 4 file の要求を残す（CLAUDE と PR template に重複を強制する） | S13 | AC4、AC6、AC7 |
| SPEC-WF-HARNESS3 | AGENTS Session Start、backlog の「Codex review-only 発注向けの Session Start fast path」entry、`docs/code_review.md:38` | D8 | 発注書が範囲を指定した review は指定範囲から始め、Session Start の全体は読まない。ただし R3/R4 の Contract Audit が求める設計正本は範囲に関わらず読み、欠陥の根拠があれば範囲外も読む。却下: 指定範囲を上限にする（Contract Audit を省ける）、Session Start に別の読書順を足す（D-034 の drift に当たる） | S1 | AC4 |
| SPEC-WF-HARNESS3 | wave 13（Plans.md）、PR2 packet | D9 | 並走 PR の所有を重ねず、依存は merge 順で解く | 「PR2 との境界と merge 順」 | AC11、AC13 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 規則は S1〜S13 の正本に置き、所有の移動と廃止の理由は D-093 に置く。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: 「npm ガードの所在は AGENTS」「問い合わせの行き先と委任の範囲の所有は AGENTS」「review の依頼・出力の規範は code_review、subagent 発注の入力欄は review packet」「保守者観点は P2」を D-093 と各正本に置く。
- Assumptions and constraints: PR2 が MANUAL の既存の見出しを変えずに `## 座組` を H2 で新設し、MANUAL `:124` の link を外し、review packet の出力の規範を外し、model-notes の `CLAUDE.md#` の anchor を直す（「PR2 との境界と merge 順」。PR2 の範囲は owner 決定 2026-09-25 で縮小済み）。PR2 の closeout が S7 と capture の前に main に入る。PR2 が止まれば本 PR も merge を待つ。
- Deferred design gaps, risk, and follow-up target: DEV_WORKFLOW の旧表の link 化（PR4）、review-only と Final Review の統合（PR4）、classifier の穴 V2（PR5）、TOOLING の吸収（未割当、closeout で backlog に記録）。
- Test Design Matrix can cite design decision IDs or source doc sections: D1〜D9 を Matrix が引く。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「直接 UI merge 禁止の要求は AGENTS と DEV_WORKFLOW の 2 file」の例外として CLAUDE と PR template を外す理由を D7 に書いた。T13 の `CLAUDE.md` 除外は T12 が旧 namespace を拒否し続けることを AC7 の mutation で確かめる。

## Impact Review Lenses

not applicable: 現場調査・実機・外部 tool・POS・CSV の変更を含まない。入口文書と review 規則の整理だけ。

## Design Readiness

- Existing design docs are sufficient because: 規則の中身は owner 決定（2026-09-23 / 2026-09-24 / 2026-09-07）と監査の判定で決まっており、本 PR は所在の移動・重複の削除・薄い wrapper の削除を行う。新しい規則は K13 / K14（どちらも owner 決定と実発生が出典）だけ。
- Source docs updated in this PR: S1〜S14。
- Design gaps intentionally deferred: Non-scope の各項目。
- Durable decisions discovered in this plan and promoted to source docs: D-093。

Minimum design checks for business-app work: 製品コードを変えないため、layer・function・DTO・永続化・画面・error の各項目は該当なし。testability は Matrix。

## Contract Probe

- 「link 検査は anchor を見ない」: `scripts/doc-consistency-check.sh:1710-1711`（`sed 's/#.*//'` で anchor を外してから実在を見る）を読んだ → 成立。したがって `#座組` の link は PR2 の前後どちらでも検査を通り、merge 順は検査ではなく意味の一貫性と S7 の依存で決まる。
- 「`classify-changes.test.sh:46` は削除する `.claude/rules/commands.md` の実在を要求しない」: `:39-46` は path の文字列を `--files-from-stdin` で classifier へ渡すだけ → 成立。
- 「T13 は `CLAUDE.md` に大文字の namespace を要求する」: `scripts/tests/codex-safe-wrappers.test.sh:361,365-370` を読み、base で exit 0 を確認した → 成立。したがって memory path の文を消すなら T13 の対象から外す必要がある（S13）。
- 「helper は packet がちょうど 1 つでないと拒否する」: `scripts/pr-gate.py:228-231` → 成立。起票時の base `8bd2bc9a` には EJ の packet が残っていた（`85b18a04` 以降の base では archive 済み。「helper と PK5 の前提」）。
- 「AC6 / AC7 の mutation は写しで行える」: base `8bd2bc9a` を AC6 の手順で `$TMPDIR` へ写し、改変なしで reading-order-drift と codex-safe-wrappers が exit 0、写しの `AGENTS.md` から `直接UI merge` の行を消すと `FAIL: helper-only merge boundary missing: AGENTS.md`、写しの `CLAUDE.md` に `-home-kosei-Projects-inventory-system/` を足すと `FAIL: T12 live B-group file still contains the history-view encoded namespace` を確認した（2026-09-25）→ 成立。
- 「Claude Code は `CLAUDE.md` があると `AGENTS.md` を既定では読まず、`CLAUDE.md` の `@AGENTS.md` は起動時に import として展開される。backtick の中は展開されない」: 公式 memory 文書（https://code.claude.com/docs/en/memory 、「When Claude Code reads AGENTS.md」「Import additional files」「Remove an earlier AGENTS.md workaround」「Share one file with other coding tools」。確認日 2026-09-27）を読んだ → 成立。同じ文書は、言葉で AGENTS を読めと指示する `CLAUDE.md` を import に置き換えるよう勧め、import の下に Claude 固有の指示を置く形を示す（S2）。
- 「claude-hooks test は削除する command を読まない」: `scripts/tests/claude-hooks.test.sh:28-37,141-155` は `.claude/commands/plan-rally.md` だけを写す → 成立。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| D1 npm ガードと問い合わせの行き先は AGENTS、CLAUDE は `@AGENTS.md` の import と Claude 固有だけ | S1、S2、S14 | AC3、AC5、AC11（rg） | DEV_WORKFLOW の旧表は PR4 |
| D2 入口に廃止済みの仕組みの指示が無い | S1〜S5、S9、S11 | AC1（rg） | subagent-review-packet・agent-guidance は PR2、template は PR4 |
| D3 薄い wrapper の削除と参照の是正 | S3、S4、S5、S12 | AC2、doc check の link 検査 | — |
| D4 review の依頼・出力の規範の code_review への統合（`inventory-code-review` Skill の出力形式と `engineering-review` Skill の報告順・label を含む） | S5、S6、S7 | AC2、AC8 | subagent 発注の入力欄は review packet（PR2 の所有） |
| D5 review 観点（保守者 P2 と Severity 表の注記、現場の前提、件数上限とカテゴリ外の除外の撤去） | S6、S8 | AC8、AC9、DS2 / DS4 | — |
| D6 project-profile の縮約 | S10 | AC10 | `:231` は PR4 |
| D7 drift test と T12 / T13 の要求先 | S13 | AC4、AC6、AC7（mutation） | — |
| D8 review の最短経路（指定範囲から始め、Contract Audit の設計正本は範囲に関わらず読む） | S1 | AC4 | 発注書側の宣言行は `.local/`（tracked 外） |
| D9 PR2 との境界・merge 順・helper の前提 | 「PR2 との境界と merge 順」「helper と PK5 の前提」 | AC11、AC12、AC13 | — |
| 隣接: Session Start の R2+ 経路と fail-closed（D-034） | S1 | reading-order-drift の check_entry_contract と既存の mutation（`:86-93`） | — |
| 隣接: canonical 読書順の再掲禁止（D-034） | 全 S | reading-order-drift の DRIFT_PATTERN | — |
| 隣接: hook inventory 0 本（D-059） | S2（CLAUDE の hook 行を残す）、S10（profile の重複を消す） | claude-hooks.test.sh | — |
| 隣接: PK1〜PK6 と Plans.md の登録 | 本 packet | `--target plan` | — |

adjacent-contract sweep: S1〜S13 が触る節で、上表に無い契約は AGENTS の `## Workspace Access` / `## Memory Model` / `## Safety` の既存項目（変えない）と review-checklist の 9 カテゴリ（変えない、AC9）。

## Test Plan

Test Design Matrix: [2026-09-25-harness-pr3-entry-and-dedup.md](test-matrices/2026-09-25-harness-pr3-entry-and-dedup.md)

- targeted tests: `bash scripts/tests/reading-order-drift.test.sh`、`bash scripts/tests/codex-safe-wrappers.test.sh`、`bash scripts/tests/claude-hooks.test.sh`、`bash scripts/tests/classify-changes.test.sh`、`bash scripts/doc-consistency-check.sh`、`--target plan`。
- negative tests: AC6 / AC7 の mutation（`$TMPDIR` の写しで行い、tracked file を変えない）。
- compatibility checks: AC13（PR2 の所有 file に差分が無い）、AC11（PR2 の見出しと link）。
- data safety checks: 変更は文書と test だけで、実データ・secret を含まない（Data Safety）。
- main wiring/integration checks: `bash scripts/tests/run-workflow-tests.sh`、`bash scripts/local-ci.sh full`。

## Boundary / Wire Contract

not applicable: JSON・browser state・CSV・config・DTO・bindings・DB を変えない。

## Review Focus

- Plan Review の冒頭で `Ordinary Operation` の各行が成立するかを 3 値で答える。
- Risk を R3 とした判定（監査 §7 の R2 と異なる）が妥当か。R2 でよいなら、どの gate の合否も変わらない根拠を示す。
- `pr-review-prompt.md` の削除を PR2 の merge に依存させた merge 順（PR2 先）が妥当か。範囲を縮小した PR2（owner 決定 2026-09-25）の packet と照合して、`## 座組` の見出し名、MANUAL `:124` の扱い、model-notes の `CLAUDE.md#` の anchor の扱いが一致し、PR3 が PR2 の消さない節・変えない見出しに依存していないか。
- S1 の問い合わせの行き先が、`docs/DEV_WORKFLOW.md:282-296` の 7 行と注記 (a)〜(c) を落とさず、PR 操作（backlog の「問い合わせの行き先の表の行 7 …」entry）と委任の範囲を足しているか。並存期間に DEV_WORKFLOW の表と食い違わないか。
- S13 の test の緩め方が、守る境界（AGENTS と DEV_WORKFLOW の直接 UI merge 禁止、R2+ 経路、fail-closed、旧 namespace の拒否）を弱めていないか。
- S10 で消す project-profile の節に、他の正本に無い固有の情報が無いか。
- S6 の K13（読めない変更を P2）が code_review の Severity 表（P3 に maintainability）と矛盾しない書き方か。

## Spec Contract

Contract ID: SPEC-WF-HARNESS3

- D1: npm 供給網ガード（D-030）の行動規則と、問い合わせの行き先（PR 操作と、判断は owner・委任の範囲では Coordinator が実行を代行できることを含む）の正本は `AGENTS.md`。`CLAUDE.md` は冒頭の `@AGENTS.md` の import で `AGENTS.md` を Claude の session に読み込ませ、Claude 固有の補助だけを持ち、座組と effort は `docs/AGENT_OPERATING_MANUAL.md#座組` を参照する。
- D2: 入口（AGENTS、CLAUDE、`.claude/rules`、`.claude/commands`、workflow Skill、code_review、project-profile、PR template、HANDOFF、review-checklist）は、Evidence Mode・legacy・state-only・三点一致・exact-HEAD 証拠・撤去済みの Phase 名を手順として持たない。
- D3: `pr-review` / `review-only-subagent` Skill、`.claude/rules/{commands,review-workflow}.md`、`.claude/commands/{test,design-review,phase-complete}.md` を削除し、live な参照を残さない。
- D4: review の依頼・出力の規範（依頼形式・重大度・出力形式・確信度）の正本は `docs/code_review.md`。subagent 発注の入力欄は `docs/templates/subagent-review-packet.md`（PR2 の所有）が持ち、`inventory-code-review` Skill と同じく規範を複製しない。`engineering-review` Skill は review 哲学の SSOT に留まり、報告順と `nit:` / `optional:` 等の label を code_review の P1 / P2 / P3 に従わせる。`docs/templates/pr-review-prompt.md` は削除する。
- D5: Final Review は保守者として読めるか（読めない変更は P2。Severity 表の P3 に注記）と、店の事実の `docs/project-memory.md` との照合を固定観点にする。review-checklist は件数やカテゴリ外を理由に指摘を落とさず、再レビューは Findings Freeze に従う。
- D6: project-profile は repo 固有の節（Project Type、Outputs、Stable Contracts、High-risk、Data Safety、Risk Level Examples、Test Design Focus、Workflow Notes）だけを持つ。
- D7: drift test は直接 UI merge 禁止と残存リスクの文を `AGENTS.md` と `docs/DEV_WORKFLOW.md` に要求し、Skill に `Evidence Mode` を要求しない。T13 は `CLAUDE.md` に memory namespace を要求せず、T12 は `CLAUDE.md` の旧 path・旧 namespace を拒否し続ける。
- D8: `AGENTS.md` `Session Start` の初回レビュー行は、発注書が読む範囲を指定した review を指定範囲から始めさせ、Session Start の全体の読書経路を踏ませない。R3/R4 の Contract Audit が求める設計正本は発注の範囲に関わらず読み、具体的な欠陥の根拠があれば他の範囲外も読んで理由を報告に書く。
- D9: PR2 と所有を重ねず、PR2 を先に merge する。capture の前に active packet が本 packet 1 つになっている。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-HARNESS3-D1 | S1、S2、S14 | AC3、AC5、AC11 | 行き先の欠落、PR 操作 | rg 出力 |
| SPEC-WF-HARNESS3-D2 | S1〜S5、S9、S11 | AC1 | 定型文の残存 | rg 出力 |
| SPEC-WF-HARNESS3-D3 | S3、S4、S5、S12 | AC2、doc check | 参照の残存 | rg・git ls-files・doc check |
| SPEC-WF-HARNESS3-D4 | S5、S6、S7 | AC2、AC8 | 固有部分の移し漏れ | rg 出力 |
| SPEC-WF-HARNESS3-D5 | S6、S8 | AC8、AC9 | Severity 表との整合 | rg・doc check |
| SPEC-WF-HARNESS3-D6 | S10 | AC10 | 固有情報の欠落 | rg 出力 |
| SPEC-WF-HARNESS3-D7 | S13 | AC4、AC6、AC7 | 境界の弱化 | test 出力と mutation の exit |
| SPEC-WF-HARNESS3-D8 | S1 | AC4 | drift の再掲 | test 出力 |
| SPEC-WF-HARNESS3-D9 | 境界と前提の節 | AC11、AC12、AC13 | merge 順 | diff の name-status・helper status |

## Data Safety

- 変更は tracked の文書と test script だけ。実 POS / 店舗データ、DB、backup、log、receipt、secret、`.env*` を読まず、commit しない。
- local-only: `.local/`（Codex 発注書、監査記録、helper の capture）、`$TMPDIR`（AC6 / AC7 の mutation 用の写し）。
- synthetic-only: 該当なし（fixture を作らない。mutation は tracked file の写しに対して行う）。

## Implementation Results

- 同期: PR2（#113）と closeout（#115）を含む origin/main を単段で merge した。競合は `docs/Plans.md` と `docs/decision-log.md` の追記だけで、両方を残し decision-log は D-092 → D-093 の順にした。PR2 側の前提（`## 座組` の H2、MANUAL の `pr-review-prompt` の link の除去、review packet の出力の規範の除去、`CLAUDE.md#` の anchor の除去、座組表の注記「effort の選び方」）が同期後の tree で成り立つことを確かめた。
- S7: 着手条件（MANUAL の `pr-review-prompt` が 0 件）の成立後に `docs/templates/pr-review-prompt.md` を削除した。live な参照は残らない（AC2 の検索は classifier test の path 文字列 1 行だけ、archive を含む link は 0 行）。
- 入口: npm 供給網ガード（D-030）と問い合わせの行き先（PR 操作と委任の範囲を含む）を `AGENTS.md` へ移し、`CLAUDE.md` は冒頭の `@AGENTS.md` の import と Claude 固有の補助だけにした。座組と effort は MANUAL の座組表を参照する。Session Start の R2+ 行から Evidence Mode・legacy の手順を消し、初回レビュー行に発注で範囲を指定した review の最短経路を足した。D-093 を記録した。
- review: 依頼・出力の規範（重大度・出力形式・確信度）を `docs/code_review.md` に統合し、保守者観点（P2）と「現場の前提」の照合を置いた。`inventory-code-review`・`engineering-review` の重大度・出力形式を code_review に従わせ、review-checklist の件数上限とカテゴリ外の除外をやめた。`pr-review` / `review-only-subagent` Skill、rules 2 本、commands 3 本、`pr-review-prompt` template を削除し参照を是正した。
- その他: PR template・HANDOFF から Evidence Mode の記述を消し、project-profile を repo 固有の節に絞った。drift test の直接 UI merge 禁止の要求先を `AGENTS.md` と `docs/DEV_WORKFLOW.md` に絞り、T13 から `CLAUDE.md` を外した（T12 の旧 path・旧 namespace の拒否は残す）。
- CI・merge: hosted CI は全ジョブ pass。Ready・merge は helper 経由、[PR #117](https://github.com/higekiri-w/inventory-system-desktop/pull/117)（2026-09-28）。

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Fill after review.
- Findings Freeze: not yet frozen; post-freeze exceptions: none.

- Closeout（2026-09-28）: Plan Review は round 3（上限）まで回し plan-approved（遷移記録のとおり）。Final Review broad（head `3961d79a`、互いに独立の 2 本）は Claude 側 Fable 5.1 = approve（P1 / P2 0、P3 4）、Codex 側 GPT-6 Astra（発注 116）= approve（P 0）。base 同期（origin/main `9d351d5b` の merge、head `f4ceaf26`）の後の closure は Codex GPT-5.6 Sol（発注 119）= approve（P 0）。PR3 の差分は保たれ、違いは `docs/Plans.md` と `docs/decision-log.md` の衝突を解いた箇所だけ。専用 record は review pass（broad 2 + closure 1）、manual not-required。Fable の P3 の disposition: P3-2（「`Plan Commit: pending` で PK5 の対象外」が Workflow State と食い違う）は本 closeout で「helper と PK5 の前提」と AC12 の 2 か所を直した。P3-1（`docs/code_review.md` の `## Output Shape` の例に確信度の欄が無い）と P3-3（`AGENTS.md` の行き先 (ii) から MANUAL §5.6「Writer が編集前に止まったとき」への導線が無い）はハーネス刷新 PR4 の入力（backlog）。P3-4（`docs/TOOLING_SKILL_COMMANDS.md:44` の「（`CLAUDE.md` で推奨）」の参照先が無い、既存）は backlog に 1 行。
