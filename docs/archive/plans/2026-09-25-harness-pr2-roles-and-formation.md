# Plan Packet: 座組の新設と退役した役割規則の削除（ハーネス刷新 PR2、R3）

2026-09-25 起草。出典は harness 監査（2026-09-24、fresh Opus 5.5、local-only の報告。要点は本 packet に書き下した）の §3.F / 3.G / 3.H / 3.K / 3.N / 3.O、§4、§5、§7 の PR2 行と、owner 決定 2026-09-23（Opus = Opus 5.5 を Coordinator / Writer / review へ全面解禁、Sonnet 5 / Opus 5 は座組から退役、Fable 5.1 は難所の相談役、Codex 稼働時の Plan Review = fresh Opus + Codex〈GPT-6 Astra〉、Final Review = fresh Opus + Codex〈GPT-6 Sol 既定・難所は Astra〉、Codex 停止中は Plan Review を Opus のみで進め Final Review だけ待つ、effort は Coordinator = high・Writer / reviewer = medium）・2026-09-24（規則は環境が変わるたびに変える。安全境界は維持する。push と Draft PR の作成は Coordinator に委任。P1/P2 の差し戻しで是正を書く前に Fable へ反例探しを頼む）・2026-09-25（Ready から merge・closeout までは owner の承認の後 Coordinator が helper で代行。Final Review の Claude 側は R3 以上と design lane で Fable 5.1、closure の Claude 側の既定は Fable 5.1。Codex の合否は是正の後も外さない。Plan Review の Codex は owner の指定で Sol にできる。Writer・レビューの Opus の medium は定義 file を置く PR5 まで実効しないと書いて進める）。Codex の effort は owner 決定 2026-09-07 / 08（Astra は既定 medium、難所 high）と 2026-09-14（Sol は既定 high）。PR3「入口と重複」と並走する（owner 2026-09-25「PR2とPR3の並列はやろうかな」）。owner 決定 2026-09-25（範囲の縮小）: PR2 は (1) `## 座組` の新設、(2) 独立性の規則の追加、(3) 退役した仕組み（Execution Mode 3 値の定義、D-056 の Opus read-only 専任、希少 slot の投入条件、§5.5 相談窓口役、§5.7 変則 provenance 監査、Astra 主担当、Subagent Budget の数値上限、D-062(c) の `codex-only` 句）を手順として指示する文の削除、(4) 退役に伴う参照の付け替えと D-092、だけにする。残る節は書き直さず、退役した仕組みへの言及だけを除く。Fable 5.1 の effort は high（Final Review の Claude 側・closure・相談役、owner 2026-09-25）。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

- Phase: archive
- Risk: R3
- Plan Commit: 6b51e223c3bf8e14a27136acb33a8223b8f75eac
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（worktree `.claude/worktrees/harness`、branch `agent/harness-pr2-roles`、fork でない、effort medium。Agent tool から直接起動すると session の high を継承するため、実効値を run 報告に記録する）
- Plan Reviewer: Opus 5.5（fork でない fresh subagent、Writer と別 context）+ Codex（GPT-6 Sol、effort high。owner 指定 2026-09-25）
- Final Reviewer: Fable 5.1（fresh subagent）+ Codex（GPT-6 Sol 既定、難所は Astra）。互いに独立な Double Audit で、各 reviewer に他の reviewer の結果を見せない（S1 の独立性の規則を本 PR 自身にも適用する）。run の報告に実効 model を記録し、Fable から別 model へ切り替わった run は Fable の本数に数えず取り直す
- Final Review Minimum: 2
- Human Gate: ready,merge

manual なし: 製品の runtime・画面・配布物は変わらない。Final Review Minimum 2 は classifier が方針 docs（`docs/AGENT_OPERATING_MANUAL.md`・`docs/DEV_WORKFLOW.md`・`docs/agent-guidance/**`・`docs/templates/**`）を `workflow=true` にし、helper が 2 を要求するため（`docs/project-profile.md` の Risk と、監査 §0-7）。

decision-log の番号予約: 本 PR は **D-092** を使う。D-091 は PR #98（デザインの決まり）で merge 済み（base `e7c22f8f` の decision-log にある）、D-093 は並走 PR3 が使う（DEV_WORKFLOW Review Rules「連番契約 registry の採番を後続 lane が確定する」の起草時予約）。

座組と現行規則の不一致（append-only の記録）: 本 packet の座組は上の owner 決定による。現行の `docs/AGENT_OPERATING_MANUAL.md:38`（D-056: 高自律・低制約適性 slot = Opus は read-only の Reviewer / Explorer 専任）・`:37` / §3.1（希少 slot の投入条件）・§3.4 の slot 表（Sonnet 5 / Opus 5）とは衝突する。owner 決定 2026-09-24 により owner の現行決定を優先し、その規則を書き換えるのが本 PR である。

遷移記録（append-only）:

- kickoff → spec-check → plan-draft → plan-gate（本 commit、plan-first、2026-09-25、Coordinator 補助の起草役）: Risk R2（下記 Risk）。書き換える正本は workflow 文書そのもので、規則の中身は owner 決定と監査で決まっており、owner の設計判断を要する未決の論点は無い（spec-check → plan-draft の唯一の skip。Design Readiness 参照）。`docs/Plans.md` の wave 13 に登録した。次は Plan Review（fresh Opus + Codex〈GPT-6 Astra〉）。
- plan-gate（2026-09-25、Coordinator の指示で是正）: Plan Review round 1 は両 reviewer とも reject。plan-gate のまま packet を是正した。findings と裁定の詳細は round 2 の完了後に Review Response へ記録する。
- plan-gate（2026-09-27、round 2 の是正）: Plan Review round 2 は両 reviewer とも reject、plan-gate のまま是正。owner 決定で PR2 の範囲を縮小（MANUAL の残す節は書き直さず、上の (1)〜(4) だけにする）。
- Risk の経緯（2026-09-27 に追記）: 起票時（`c0e4a14b`）の Risk は R2 で、Plan Review round 1 の是正（`9d444c3f`、2026-09-25）で R3 に改めた（review の規則を書き換える workflow gate の変更。理由は下記 Risk）。上の起票行の「Risk R2」はその時点の値。D-091 は PR #98 で merge 済み。
- plan-gate（2026-09-27、round 3 の一括是正）: Plan Review round 3（上限）は両 reviewer とも reject。round 4 は回さず、同型の一括是正（裁定 r3）。予算の改定は owner 承認待ち。
- plan-gate → plan-approved（2026-09-27、Coordinator、本 commit）: Plan Review round 3（上限、対象 `a76a1514`）は Claude 側（fresh Opus 5.5）・Codex 側（GPT-6 Astra、high）とも reject（P2 = helper が同じ head/base の broad だけを数えるのに Claude 側 broad の取り直しが無い、AC8 の期待値が `#座組` の link の追加と矛盾、S4 の残す文が AC2 / AC7 の禁止語に当たる、ほか P3）。round 天井に達したため追加の round は回さず、同型の一括是正（`6b51e223`、裁定 r3）で全件を反映し、Coordinator が現物で確かめた。owner 承認（2026-09-27）のもと予算を介入 10・relay 5 に改めた。Plan Commit = 本 commit の親（round 3 の一括是正後の承認版 `6b51e223`）。
- plan-approved → implementing（2026-09-27、Coordinator、本 commit）: plan-approved の承認版（`10190352` の親）のまま Writer（Opus 5.5 subagent）に実装を発注する。packet の契約は変えない。
- implementing → archive（2026-09-28、Coordinator、closeout、本 commit）: PR #113 を squash merge（`7c51572c`、2026-09-28）。packet と Matrix を archive へ移送し、Implementation Results と Review Response を記録。Plans.md と backlog を同期。

## Owner Effort Budget

- 介入回数上限: 10（owner 承認 2026-09-27 で 4 から改定。closeout 時の消費 12 で上限を 2 超過 = plan-approved までの 6、Final Review の Codex relay 2〈broad 107・closure 111〉、helper record の指示 1〈見込みに無かった〉、record の形の判断 1〈broad の record 前に是正を push した Coordinator の順序の誤りで生じた〉、Ready 1、merge 1〈owner 指示 2026-09-28〉。超過は merge 後の closeout で判明し、事前に上限の改定を諮っていない。plan-approved 時点の消費 6 = 起票承認 2026-09-25 1、範囲縮小の owner 決定 2026-09-25 1、Plan Review round 1〜3 の Codex relay 3、本改定と plan-approved の承認 1。残る見込み = Final Review の Codex relay 1、必要なら Codex closure 1、Ready 1、merge 1）
- 実働時間上限: 15分（文書だけの変更で、owner の作業は Codex の起動 1 行と Ready・merge の判断に限られる見込み）
- relay 往復上限: 5（owner 承認 2026-09-27 で 4 から改定。closeout 時の消費 5 = Plan Review round 1〜3 の Codex 3、Final Review の Codex broad 107・closure 111 の 2。plan-approved 時点の消費 3 = Plan Review round 1〜3 の Codex。残り = Final Review の Codex broad 1、必要なら是正の後の Codex closure 1。理由: round が天井の 3 まで回り、是正の後の取り直しにも Codex の合否を含めるため〈owner 決定 2026-09-25〉）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

本 PR は §5.5 相談窓口役を削除し、template の §5.5 の使用指示 2 行（`docs/templates/plan-packet.md:34`・`:125`）を相談役への参照に替える（S7）。template の本節の見出しと 2 欄は PR4 が消すまで残るため、値だけ置く。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
reviewer の充当（Final Review の Claude 側を R3 以上で Fable 5.1 にする、Codex の合否を是正後の取り直しでも外さない）、Double Audit の独立性、D-062(c) の Plan Reviewer の規則、Human Gate の委任の参照、Subagent Budget の数値上限の撤去を書き換える。どれも merge の可否を左右する review の規則で、`docs/DEV_WORKFLOW.md` Risk Tiers の「If uncertain between R2 and R3, choose R3 when the change touches ... workflow gate」に当たる（D-090 の lane と PR1 も同じ判定で R3）。変更は文書だけで、`scripts/**`・`.github/**`・classifier・helper は変えず、checker / pre-push / helper / hosted CI の判定式は変わらない。方針 docs に触るので helper は Final Review Minimum 2 を要求する（R3 でも本数は同じ）。MANUAL の文字列を検査する test（`scripts/tests/doc-consistency-plan-packet.test.sh:842-843` の `one-shot irreversible` / `task-shape`、`scripts/tests/claude-hooks.test.sh:30-54` の禁止句）は、要求を満たす文面にして test 側は変えない。R4 には当たらない（data・secret・不可逆操作なし）。

Review-only skipped because: R3 の review-only sub-agent は、Final Review の Double Audit の broad（Fable 5.1 と Codex の 2 本、どちらも Contract Audit を正本から行う）が兼ねる（`docs/DEV_WORKFLOW.md` Review Rules・Draft PR Checkpoint、`docs/code_review.md` の `## Review-only Sub-agent Protocol`）。

## Goal

Goal Invariant:

### 最小完了条件

- 本 PR の merge 後、Coordinator が R2+ packet を起票するとき、`docs/AGENT_OPERATING_MANUAL.md#座組` の表 1 つを読めば、Coordinator / Writer / Plan Reviewer / Final Reviewer / 相談役 / Human Gate の担当・effort と、Codex 停止中の進め方が決まる。その表は owner の現行決定（2026-09-07 / 08・14・23 / 24 / 25）と一致し、本 PR の所有 file の範囲では、座組・model・effort の値を定める記述はこの表だけになる（CLAUDE.md の effort 節は PR3 がこの anchor への参照に替える。Non-scope）。
- 同じ round で 2 本以上の review を回すとき（Plan Review・Final Review とも。Double Audit を含む）に、各 reviewer に他の reviewer の結果を見せない規則が、MANUAL §3 の独立性の項と review packet の両方に置かれ、次の Plan Review・Final Review の発注でそのまま使える。
- 退役した仕組み（Execution Mode 3 値の定義、D-056 の Opus read-only 専任、希少 slot の投入条件、§5.5 相談窓口役、§5.7 変則 provenance 監査、Astra 主担当、Subagent Budget の数値上限、D-062(c) の `codex-only` 句）を手順として指示する文が、MANUAL・DEV_WORKFLOW・agent-guidance・review packet・plan-packet template の §5.5 使用指示に残らず、decision-log D-092 がその退役と残すものを列挙する。MANUAL の残す節は書き直さず、退役した仕組みへの言及だけを除く。

### 失敗定義

- 残すべき独立性・安全境界（Writer ≠ Plan Reviewer、Writer ≠ Final Reviewer、Plan Reviewer と Final Reviewer は別の fresh context、自己承認禁止、fresh context、Double Audit、Human Gate は owner、depth 1、one-writer、load-bearing 判断の正本直読み、hook 0 本と plugin 無効化、advisory hook の推定禁止と hook が review を強制しないこと、one-shot irreversible の task-shape と不可逆 mutation の直前の owner 判断、Writer が編集前に止まったときの番号付き手順 1〜3、発注直前の照合〈必須 command の出力先が編集禁止範囲に入らない〉）のどれかが文書から消える、または弱まる。
- 座組表が owner 決定と食い違う、または座組・effort の規則や effort の値が 2 か所以上に残る（本 PR の所有 file の範囲で）。
- 本 PR が PR3 / PR4 / PR5 の所有 file を編集する（所有表の例外 = `docs/templates/plan-packet.md:34`・`:125` を除く）、または PR3 と同じ行を取り合う。
- MANUAL の編集で既存の test（`doc-consistency-plan-packet.test.sh:842-843`、`claude-hooks.test.sh`）や link 検査が落ちる、または既存の見出しが変わって DEV_WORKFLOW `:68`・`:286` と `.agents/skills/inventory-workflow-start/SKILL.md:13` の anchor が古くなる。
- 本 PR 自身の Final Review で、reviewer が他の reviewer の結果を読んで独立性が崩れる（PR #97・#94 の再発）。

### 非目的

- 入口と重複の整理（AGENTS / CLAUDE / `.claude/**` / `.agents/**` / code_review / review-checklist / pr-review-prompt / PR template / project-profile）: PR3。CLAUDE.md の effort 節を `docs/AGENT_OPERATING_MANUAL.md#座組` への参照に替えるのも PR3。
- 手続きの軽量化（plan-packet template〈`:34`・`:125` を除く〉、Owner Effort Budget の数値、Review Rules の 3 分類、review-only と Final の統合、Findings Freeze、Wave Operation、Post-Merge Closeout、WER）: PR4。
- classifier の穴（`.claude/agents/**`）、helper の自己検査の穴、`.claude/agents/{reviewer,writer}.md` の新設による subagent ごとの effort 指定（`.gitignore:116` の除外の解除と一緒）、depth 1 の機械化（settings の env `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH`）: PR5。
- MANUAL の全面的な縮約（残す節の書き直し、節番号の振り直し）と、`docs/agent-guidance/` の `shared.md`・profiles の README への統合: 別 PR（closeout で backlog に 1 行）。
- 新しい gate・checker・test の追加。独立性の規則は文書の規則として置き、機械の検査にしない。
- 過去の decision-log 本文と archive の書き換え。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

workflow の変更なので、発注 → 停止と訂正 → review → owner 判断の通常列を、本 PR の merge 後の文書で書く。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 本 PR merge 後、新しい R2+ の依頼 | Coordinator（`#座組` の Coordinator 行）が `#座組` の表で Workflow State の 4 役を書き、plan-first commit を push する | 座組を owner に尋ねずに決められる。規範の文では、model 名は packet の field の値と `#座組` の表にだけ現れる（MANUAL `:15` と DEV_WORKFLOW `:83` も同じ意味の文にする） | Plan Review を発注 | Codex の稼働状況は Coordinator がその時点で確かめる |
| plan-gate、Codex 稼働中 | fresh Opus（fork でない）と Codex（既定 Astra。owner の指定で Sol）に Plan Review を発注する。Writer が Codex の packet では Writer と別 vendor の Plan Reviewer（fresh Opus）を必ず含める。各 reviewer に他の reviewer の結果を見せない（MANUAL §3 の独立性の項）。Plan Review の Codex 発注は PR 番号を前提にせず、SHA を固定した隔離 worktree で読ませる | 2 本の review が独立に返る。冒頭で操作列の 3 値 | P1/P2 = 0 で plan-approved | なし |
| plan-gate、Codex 停止中 | fresh Opus だけで Plan Review を進める（`#座組` の Codex 停止中の列） | 停止を理由に Phase を止めない | P1/P2 = 0 で plan-approved | なし |
| Plan Review・Final Review が P1/P2 で差し戻す、または Gated Amendment を書く | 是正を書く前に Fable 5.1 へ反例探しを頼む（相談役。Writer・reviewer には数えない）。Fable が使えないときは省くか fresh Opus で代え、遷移を止めない | 是正案の穴を先に潰し、round を減らす | 是正 commit → 次 round | Fable の価値は実測で見直す（`未実測`）。相談した run は closure に数えない |
| 実装済み、Verify 済み | Writer（`#座組` の Writer 行。effort の実効値を run 報告に記録）の実装と Verify の後、Coordinator が Draft PR を作る（DEV_WORKFLOW Draft PR Checkpoint の時点。owner 委任 2026-09-24 は作成の実行の委任で、早期作成の依頼ではない）。Claude 側（R3 以上と design lane は Fable 5.1、それ以外は fresh Opus）と Codex（Sol 既定）に Final Review を発注する。各 reviewer の発注では、他の reviewer の結果が置かれた場所すべて（例: packet の Review Response の段落、PR body、PR の comment / review、Coordinator が保存した local の報告 file）を読ませず、発注の本文にも他の reviewer の結果とそれ由来の観点を書かない。すべてそろうまで Coordinator は転記しない | Double Audit が 2 本とも独立に数えられる | 同じ HEAD の両方の broad を helper で record し（helper は同じ head/base の broad だけを Final Review Minimum に数える）、その後に是正で HEAD が変わったら closure（Claude 側の既定は Fable 5.1、fresh context）を record | 相手の結果を読んだと申告した run と、発注と実効 model が違う（宣言なしに切り替わった）run は数えず取り直す |
| Final Review、Codex 停止中 | Claude 側の broad を済ませ、Codex の枠を pending にして結果を待つ（Claude の fresh reviewer で代えない。代替は Codex の別 model だけで、Claude での代替は owner の明示決定だけ）。Claude 側の findings の是正は進めてよく、Codex には是正後の HEAD への初回 broad として（先の findings を見せず）発注する。Codex の broad が返ったら、Claude 側も同じ HEAD で broad を取り直す（helper は同じ head/base の broad だけを Final Review Minimum に数える。`scripts/pr-gate.py` の `record` は broad の head/base が変わると audits を作り直す）。取り直しの発注にも Codex の結果と先の findings を見せない。Draft のまま次の lane を進める | Ready 以降だけが止まる | Codex の broad が返り、同じ HEAD で Claude 側の broad を取り直す | なし |
| Final Review が P1/P2 で差し戻し、是正（GA を含む）を入れた | Codex に修正案と確認条件を頼んでよいが、それは一案で、採るかは Coordinator が現物で裏取りして決める（採否の記録は Review Response）。是正の後の取り直し（broad または closure）にも Codex の合否判定を含め、Final Reviewer の欄から Codex を外す GA をしない（owner 決定 2026-09-25）。closure・是正の発注は前回 findings から始めてよい（`docs/code_review.md` の `## Verification Rules` の closure の文） | 修正の後も別 vendor の合否が残る | Codex を含む取り直しが通る | Codex 停止中は上の行と同じく Ready 以降だけを止める |
| review 通過 | owner の Ready・merge 承認 → Coordinator が helper で `ready` / `merge`・closeout を代行（owner 2026-09-25。委任の範囲は `AGENTS.md` Decision and Approval Boundaries） | owner は判断だけをする | merge → closeout PR | なし |

## Scope

S1〜S7 が本 PR の実装範囲。行番号は base `e7c22f8f`（本 branch の merge-base。起票時の `85b18a04` から本 PR の所有 file は変わっていない: `git diff --stat 85b18a04 e7c22f8f -- docs/AGENT_OPERATING_MANUAL.md docs/DEV_WORKFLOW.md docs/agent-guidance docs/templates` が空、decision-log は D-091 の追記だけ）のもの。

- **S1 `docs/AGENT_OPERATING_MANUAL.md` は書き直さず、次の (1)〜(4) だけを行う**（owner 2026-09-25）。既存の見出しは変えない（例外は §5.4 の見出しの括弧書き 1 か所。`rg -o 'AGENT_OPERATING_MANUAL\.md#[^) ]+' --hidden -g '!.git/**' .` で §5.4 の anchor を指す参照は 0 件）。節番号は振り直さない（DEV_WORKFLOW `:280` の「§3.5」、D-062 本文の「§3.3」が古くなるため）。書き直さない節: §1（`:3`・`:11` 以外）、§2（`:15`・`:22` 以外）、§3 の既存の独立性の項目 `:32-36`、§3.3 の既存 4 項目、§3.5（`:107` 以外）、§4（`:119`・`:124` 以外）、§5.1〜§5.3、§5.4（下の表の 4 行以外）、§5.6（`:233`・`:250` 以外）、§6（`:274` 以外）、§6.1。
  - (1) **`## 座組` の新設**: `### 3.5` の後・`## 4.` の前に H2 で置く（見出しは必ずこの文字列。PR3 の CLAUDE.md が `#座組` を参照する）。表 1 つ。行 = Coordinator / Writer / Plan Reviewer / Final Reviewer / 相談役 / Human Gate、列 = Codex 稼働時の担当 / Codex 停止中 / effort。effort の値と実効値の注意は effort 列の cell にだけ置き、決定日を添える（tracked の正本の中で effort の値はこの列だけ）。中身は Ordinary Operation と上の owner 決定のとおり:
    - Coordinator = Opus 5.5 main session。effort: high（owner 2026-09-23）。
    - Writer = Opus 5.5 subagent〈Codex も可〉。effort: medium（owner 2026-09-23。Codex が Writer のときは Codex の値）。同じ cell に実効値の注意「Agent tool から直接起動する subagent は session の値（high）を継承する。Workflow から起動する subagent は effort を指定できる。tracked の定義 file（`.claude/agents/**`）は `.gitignore:116` の除外の解除と classifier の穴と一緒に PR5。実効値を run 報告に記録する」（公式 sub-agents「Supported frontmatter fields」の `effort`）。
    - Plan Reviewer = fresh Opus 5.5 + Codex（既定 Astra。owner の指定で Sol）。Codex 停止中 = fresh Opus だけで進める。effort: Opus = medium（実効値の注意は Writer の cell と同じ）、Codex は model ごと（Astra = 既定 medium・難所 high〈owner 2026-09-07 / 08〉、Sol = 既定 high〈owner 2026-09-14〉）。
    - Final Reviewer = Claude 側 1 本 + Codex（Sol 既定、難所は Astra）。Claude 側は R3 以上と design lane で Fable 5.1、それ以外は fresh Opus 5.5。closure〈base 同期を含む〉の Claude 側の既定は Fable 5.1（owner 2026-09-25）。担当の cell に「Codex の本務はレビューとしての合否判定で、是正の後の取り直しでも外さない。修正案は一案で採否は Coordinator」（owner 決定 2026-09-25）と「Fable が使えないときは §3.3 に従い理由を 1 行残して fresh Opus で代えてよい。発注と実効 model が違う（宣言なしに切り替わった）run は数えず取り直す」を置く。Codex 停止中 = Claude 側を済ませ、Codex の枠は §3.3 に従い pending（Ready 以降だけが止まる）。effort: Fable 5.1 = high（owner 2026-09-25）、Opus = medium、Codex は Plan Reviewer の行と同じ。
    - 相談役 = Fable 5.1〈Plan Review・Final Review の P1/P2 の差し戻しと、Gated Amendment を書くたびに、是正を書く前に反例探し。Writer・reviewer の数に入れない。使えないときは省くか fresh Opus で代え、遷移を止めない。相談した run は closure に数えず、closure は fresh context で行う〉。advisor は座組の必須にせず、使う場合の組合せと費用の注意を同じ行の cell に 1 行で置く。effort: high（owner 2026-09-25）。
    - Human Gate = owner（判断は owner。委任の範囲は `AGENTS.md` の Decision and Approval Boundaries）。
  - 表の下の注記は 5 行以内で、effort の値を書かない（値は表の cell だけ）: (1) 確認日と owner 決定の日付。(2)「モデル更改時は owner 決定を受けてこの表だけを書き換える。座組・effort を他の文書へ複製しない」と、削除する §3.4 `:103` 後半の「owner をモデル間の伝書鳩にしない（発注は Plan Packet / PR body / review packet という repository 証跡経由で渡す）」の 1 文。(3) Fable の出番の理由と見直し方を 1 文: owner 2026-09-25「週制限のなかで Fable 持て余す、指揮を Opus にしてる分」を受け、Fable は context を絞った subagent の発注で使い advisor は常用せず、1〜2 週ごとに `/usage` の Fable の消費で出番を増減する。(4) 根拠の URL（公式 Opus 5.5 prompting guide「Calibrate effort」、effort「Recommended effort levels for Claude Opus 5.5」、Fable 5.1 prompting guide）と「名前の同じ effort を model 間で同じ思考量と見なさない」。
  - (2) **独立性の規則の追加**: §3 の既存の独立性の項目（`:32-36` の 5 項目: Writer ≠ Plan Reviewer、Writer ≠ Final Reviewer、Final Reviewer は Coordinator・Writer と fresh context で自己承認禁止、R4・workflow gate の Double Audit、Human Gate は owner）は変えず、同じ list に次を足す。
    - (a) fork は独立に数えない。第三者性の要る review は fork でない fresh な subagent を使う（owner 2026-09-23。根拠: Claude Code 公式 sub-agents「How forks differ from other subagents」= fork は会話の全履歴を継承する）。
    - (b) **同じ round で 2 本以上の review を回すとき（Plan Review・Final Review とも。Double Audit を含む）は、同じ round の各 reviewer に、他の reviewer の結果が置かれた場所すべて（例: packet の Review Response の段落、PR body、PR の comment / review、Coordinator が保存した local の報告 file）を読ませず、その発注の本文にも他の reviewer の結果とそれ由来の観点を書かない（発注の順や同時発注に依らず、どの reviewer の発注にも入れる）。closure・是正の発注は前回 findings から始めてよい（`docs/code_review.md` の `## Verification Rules` の closure の文。MANUAL の文では行番号で参照しない）。すべての結果がそろうまで Coordinator は結果を packet・PR body に転記しない。読んだと申告した run は独立に数えず取り直す。Codex の broad が round をまたいで pending のときは、Claude 側の findings の是正を進めてよく、Codex には是正後の HEAD への初回 broad として（先の findings を見せず）発注する。Codex の broad が返ったら、Claude 側も同じ HEAD で broad を取り直す（helper は同じ head/base の broad だけを Final Review Minimum に数える。`scripts/pr-gate.py` の `record`）。取り直しの発注にも Codex の結果と先の findings を見せない**（2026-09-25 の dogfood 所見: PR #97・#94 で 2 回取り直した）。
    - (c) Writer が Codex の packet は、Writer と別 vendor の Plan Reviewer を含める（担当は `## 座組`。D-062 の原則「縛られる側に gate 文と検査器を書かせない」。現行 DEV_WORKFLOW `:349` の移入）。
    - (d) load-bearing な判断（Plan Gate、Final、裁定）は担当者が正本を直接読み、subagent の要約は claim として扱う（DEV_WORKFLOW Subagent Budget の該当行への参照で足りる）。
    - (e) Plan Reviewer と Final Reviewer は別の fresh context（同じ model・vendor でよい。DEV_WORKFLOW Wave Operation `:237` の同規則）。
    - §3.3 の既存 4 項目（`:83-86`）は保ち、次を足す。「Plan Review の Codex 分は上の 2 つ目の項目（pending の役割を要する遷移を進めない。base の `:84`）の例外とし、`## 座組` の表の Codex 停止中の列に従う。Final Review の Codex の枠は 2 つ目の項目のとおり pending にする」（ここに model 名を書かない。MANUAL の文では行番号で参照しない）。「Codex の Final Review の枠は Codex の別 model（`## 座組` の Final Reviewer 行の Codex の候補）でのみ代替できる。Claude での代替は owner の明示決定（decision-log に記録）だけで、Coordinator の指名では不可。Codex の結果が得られるまで枠は pending」。
  - (3) **削除する節**（退役した仕組みを手順として指示する範囲。行番号は base `e7c22f8f`）:

    | 範囲 | 中身 | 退役の出典 |
    |---|---|---|
    | §3 `:37` | 希少・高コストな model slot を通常実装の Writer に充てない項 | 希少 slot の投入条件（D-038 の slot 規範） |
    | §3 `:38` | 高自律・低制約適性 slot の read-only 専任と §5.5 の兼任 | D-056、D-058 |
    | §3.1 `:42-52` | 希少・最高能力 slot の投入条件と design board 例外 | 希少 slot の投入条件 |
    | §3.2 `:54-77` | Execution Mode 3 値と `codex-only` の運用形、`#### Astraを主担当にする場合（D-087）` | D-034 の Execution Mode（欄は PR #97 で撤去済み）、D-084、D-087 |
    | §3.4 `:88-103` | Model slot 対応表・旧称の解読・profile の対応（`:103` 後半の 1 文は `## 座組` の注記 (2) へ移す） | D-038(1)（slot 表 → `## 座組`） |
    | §5.5 `:185-211` | 相談窓口役 | D-058 |
    | §5.7 `:254-268` | 変則 provenance packet の監査採用手順 | D-065 のうち §5.7 |

  - (4) **削除に伴い直す行**（行の他の部分は変えない）:

    | 行 | 現行の句 | 直し方 |
    |---|---|---|
    | `:3` | availability mode / role mapping、役割割当だけでは表せない限定的な実行形態 | 本書が役割・独立性・座組・役割担当の一時不能（§3.3）・task-shape（§3.5）・追加 prompt の正本である、の意味に替える |
    | `:11` | Evidence Mode の段落（PR1 で退役済み） | 段落を削除する |
    | `:15` | モデル名は Plan Packet `Workflow State` の値としてのみ現れる | 「規範の文では、model 名は Plan Packet `Workflow State` の field の値と `## 座組` の表にだけ現れる」（DEV_WORKFLOW `:83` と同じ意味） |
    | `:22` | Final Reviewer の「independent-review phase の担当」 | 「実装後の Final Review（broad audit）の担当」（撤去済みの Phase 名。backlog の残存 2 か所のうち MANUAL 側） |
    | `:28` 後半 | model slot 名と現行実体の対応は §3.4 の表だけに置き | 担当・model・effort は `## 座組` の表だけに置き、他文書へ複製しない |
    | `:30` | Execution Mode（§3.2）に関わらず | 「常に」適用する |
    | `:81` 冒頭 | Execution Mode は vendor 単位の可用性を扱う。 | この 1 文を除く（続く役割担当の一時不能の文は保つ） |
    | `:107` | この task-shape 軸は Risk と、vendor 可用性を示す Execution Mode（§3.2）の双方に直交する。Execution Mode の enum は変更しない。 | 「この task-shape 軸は Risk に直交する。」だけ残す（`task-shape` の語 = `doc-consistency-plan-packet.test.sh:842-843`） |
    | `:119` | shared contract + §3.4 で対応する slot-neutral profile | agent-guidance index（`agent-guidance/README.md` への link）の参照だけにする |
    | `:124` | PR review 依頼の行の `pr-review-prompt.md` の link | link を外し `code_review.md` だけにする（PR3 の S7 の着手条件 `rg -n 'pr-review-prompt' docs/AGENT_OPERATING_MANUAL.md` = 0 が本 PR で成立し、PR3 は MANUAL を触らない） |
    | `:173` | §5.4 の見出しの括弧書き「（高自律・低制約適性 slot 向け）」 | 「（read-only の Reviewer / Explorer 向け）」 |
    | `:175` 冒頭 | §3 の高自律・低制約適性 slot への発注書は | 「read-only の Reviewer / Explorer への発注書は」。括弧内の理由は保つ |
    | `:181` | item 5 の「§5.5 相談窓口役では…例外を適用する」 | その句だけ除く（既定 0 と上限明記の必須は保つ） |
    | `:183` | 従来型発注書（手順込み）は他 slot 向けに従来どおり使用する | Writer への発注など他の発注には従来型発注書（手順込み、§5.6）を使う、の意味に |
    | `:233` | 対象変更・Evidence Mode・正本の条件 | 「Evidence Mode・」を除く |
    | `:250` | item 7（§3.2 codex-only、D-087） | item 7 を削除し、8・9 を 7・8 に繰り上げる（本文の「5 の経路」「2 の発注の作り直し」の項番は変わらない） |
    | `:274` | subagent 数の上限は Subagent Budget が正本。並列機能がこれを超えられる場合でも budget を守る | 「同時に動かす subagent の数に上限は置かず Coordinator が決める（DEV_WORKFLOW Subagent Budget）。depth 1 と one-writer は守る」 |

    `:215` 冒頭の「§5.4 の read-only Reviewer / Explorer 専用の低制約 profile ではなく」は、§5.4 を残すので変えない。
- **S2 `docs/DEV_WORKFLOW.md` の 6 か所だけを直す**（他の行は PR4 の所有で触らない。MANUAL の見出しを変えないので `:68` と `:286` の anchor は触らない）。
  - `:83` の Workflow State の役割 field の括弧内だけ: 「role definitions in AGENT_OPERATING_MANUAL; concrete model names appear only as values here, never in normative rules」を、役割の定義と座組表は `AGENT_OPERATING_MANUAL.md#座組`（link。link 先はこの文字列のままにする。AC8）にあり、規範の文では具体的な model 名はこの field の値と座組表にだけ現れる、の意味に替える（行の他の部分は変えない）。
  - `:215` Astra 主担当の行を削除する。
  - `:252-268` Subagent Budget: 見出し `## Subagent Budget` は保つ（`.claude/commands/plan-rally.md:8` が名前で参照する。PR3 の file）。`:254-261` の Risk 別の同時数の前置きと表、`:263` の wave 合算 4 を削除し、「同時に動かす subagent の数に上限は置かず、Coordinator が作業ごとに決める（owner 2026-09-23）」の 1 行に替える。`:264-268` の 5 項目（depth 1・one-writer rule・出力契約・load-bearing・P3-only）は残す。
  - `:280` one-shot irreversible の行: 後半の `This task-shape choice is separate from the vendor-oriented Execution Mode.` の 1 文だけを削る（link と「§3.5」は変えない）。
  - `:289`（問い合わせの行き先の行 4）: 「実行 mode 上の裁定者」を「Coordinator（finding の採否）。owner の裁定権に当たる場合は owner（`AGENTS.md` Decision and Approval Boundaries）」に替える（座組表に裁定者の行は無いので表を参照しない。行の他の部分は変えない）。
  - `:349` D-062(c) の行（Writer が Codex の packet の Plan Reviewer の vendor 規則、`codex-only`、§3.3 への参照）を、「Writer が Codex の packet の Plan Reviewer は `AGENT_OPERATING_MANUAL.md`（link）§3 の独立性の項（Writer と別 vendor の Plan Reviewer を含める。担当は同書 `## 座組`）に従い、役割担当が一時的に使えないときは同書 §3.3 に従う（D-062）」の 1 行に替える。
- **S3 `docs/agent-guidance/` の退役した言及と effort の値だけを直す**（file の構成と `merge-evidence.md`・`evals/`・`profiles/` の本文は変えない）。
  - `README.md:8` の「適用条件が成立してから github mode を使い、bootstrap は legacy 規定を維持」の句を削る。他の行（`:5`・`:6` の shared / profiles の link を含む）は変えない。
  - `shared.md:9` の段落（Evidence Mode・legacy・三点一致）を削る。他の段落は変えない。
  - `model-notes.md`: 見出し `## Effort の選定` は保つ（`evals/context-routing-fixture.md:11` が anchor で参照する）。節の本文は effort の値を持たず、Claude 側・Codex 側とも `../AGENT_OPERATING_MANUAL.md#座組`（link） を参照する 1〜2 文にする（`:7` の owner 方針 2026-09-14 と `:9` の値は座組表の effort 列が引き継ぐ）。`:9` の `[CLAUDE.md](../../../CLAUDE.md#sonnet--opus-の-effort)` の link は `../AGENT_OPERATING_MANUAL.md#座組` へ付け替える（PR3 が CLAUDE.md の effort 節を座組表への参照に替えるので、そのままでは PR3 の merge で anchor が切れる。退役に伴う参照の付け替え = owner 決定の (4)）。`GPT-5.6 Sol` の見出しと文を GPT-6 Sol に改め、model 固有の傾向（Sol は Astra と同じ effort の選び方へ機械的に揃えない、Astra 向けの傾向を自動継承しない等）だけを残す。model-notes 全体で `medium` / `high` / `xhigh` の値の語を書かない（AC12）。公式確認日を更新する。
- **S4 `docs/templates/subagent-review-packet.md` を改訂する**（本 PR の所有。PR3 は編集しない）。review の出力の規範は `docs/code_review.md` 1 か所に置き（PR3 の S6 が統合する）、本 template は出力の規範を持たない。発注の型（入力の 5 点）は MANUAL §5.4 に残し（owner 2026-09-25。§5.4 は書き直さない）、本 template に複製しない。
  - `## Role`: read-only 宣言に「subagent を起動しない（depth 1）」を文として足す。`:7` の「根拠、推定severity、確信度を示し、」の句を除く（出力の要求は `## Output` の参照に寄せる）。reviewer の姿勢として「軽微さや不確実さだけで黙って落とさない」（`:7`）と「P2 を残したまま pass にしない」（現行 `:32`）の 2 句を `## Role` に残す。read-only・claim の扱いは残す。`確信度` の語は置かない（確信度の要求は PR3 の S6 が `code_review.md` の `## Output Shape` へ足す。本 PR は足さない）。
  - `## Target` に 1 項目足す: 「読ませないもの（同じ round で 2 本以上の review を回すときの各 reviewer。Plan Review・Final Review とも、Double Audit を含む）: 他の reviewer の結果が置かれた場所すべて（例: packet の Review Response の段落、PR body、PR の comment / review、Coordinator が保存した local の報告 file）。その発注の本文にも他の reviewer の結果とそれ由来の観点を書かない（closure・是正の発注は前回 findings から始めてよい）。`../AGENT_OPERATING_MANUAL.md`（link）§3 の独立性の項」。
  - `## Output` の本文（`:32` の finding の書式・確信度の要求）を、`../code_review.md` の `## Output Shape` と `## Finding Severity` への参照 1 文に替える（「P2 を残したまま pass にしない」は上の `## Role` へ移す）。
  - `:26` の `legacyのstate-onlyはfile名とzero-context hunkの両方を確認する。` の文と `github modeは` の前置きを削り、専用 record / head / base・必要 broad / closure / manual / R4・実効 rules / CI を確認する中身は残す。文末の「旧三点一致を要求しない」は「廃止済みの SHA 照合を要求しない」に置き換える（意味は保ち、AC2・AC7 の禁止語〈`legacy`・`state-only`・`三点一致`〉に当たらない。実装後の AC2・AC7 で確かめる）。
- **S5 `docs/decision-log.md` に D-092 を末尾へ追記する**（既存 entry の本文は変えない）。見出し `## D-092: Opus 5.5 主軸の座組を AGENT_OPERATING_MANUAL の座組表 1 か所に置き、Execution Mode 時代の役割規則を退役させる（2026-09-25）`、欄は D-090 と同じ Status / Decision / Why / Compatibility。Compatibility に次を列挙する。
  - 全面 superseded: D-056（Opus read-only 専任）、D-058（相談窓口役）、D-079（UI 座組）、D-084（codex-only）、D-087（Astra 主担当）。
  - 部分的に superseded: D-034（Execution Mode 3 値〈PR #97 で撤去済み〉、Subagent Budget の数値上限）、D-038(1)（slot 表 → 座組表）、D-062(c)（vendor 規則は MANUAL §3 の独立性の項 (c) へ。`codex-only` の句は消え、capacity pending は §3.3 の一般則が持つ）、D-065 のうち §5.7 変則 provenance 監査。
  - PR #97（PR1）で撤去済みの記録（PR1 の Non-scope が PR2 の D へ送ったもの）: D-035（state-only・三点一致）、D-038(8)（STATECAP）、D-046-3（backtrack）、D-049（execpolicy の 2 mirror 維持）、D-055 / D-074 のうち Rebase Map、D-085 の MG-D10 / MG-D11（移行）。
  - 維持: D-059（hook 0 本）、D-062(a)(b) と設計原則、D-039（PK5）、D-085 / D-086（merge gate）、D-090（Ordinary Operation・Writer 停止時）、独立性・Double Audit・Human Gate、MANUAL の残す節（§3.3・§3.5・§5.1〜§5.4・§5.6・§6）。
  - Why: owner 決定 2026-09-23 / 24 / 25（環境が変わった: Opus 5.5 と Fable 5.1 の水準、Codex の役割が review 中心へ、Claude 側の枠が潤沢）と、PR #97・#94 の独立性の崩れ、#96 の GA6 を Claude 側の reviewer 2 本だけで締めて merge し、最終形が Codex の合否を通らなかったこと（owner 2026-09-25「レビューとしての通すか否かが第三者の目として入るからそこはしっかり責務として持ちたい。あくまで一案でそこを採用するかはこっち次第」）。
- **S6 `docs/Plans.md` と `docs/backlog.md`**: `Plans.md` は本 commit の登録行と、closeout での完了の反映だけ。`backlog.md` は実装 PR では編集せず、closeout で (i)「撤去済みの Phase 名 `independent-review` が 2 か所に残る」の entry を template 側（`docs/templates/plan-packet.md:196`、PR4 の所有）だけに狭め、(ii)「MANUAL の全面的な縮約（残す節の書き直し・節番号の振り直し）と agent-guidance の統合」の 1 行を足す。
- **S7 `docs/templates/plan-packet.md` の 2 行だけ**（template の他の行は PR4 の所有で触らない）。
  - `:34`（Consultation Relay 節の §5.5 の使用指示）を「相談役への相談は `../AGENT_OPERATING_MANUAL.md`（anchor なしの link）の座組に従う。本節の 2 欄は `none` のままにする」の意味の 1 行に替える。
  - `:125`（Registration 表の「AGENT_OPERATING_MANUAL §5.5 consultation relay 使用」の行）を、左欄・右欄とも §5.5 の使用指示を持たない行（相談役への相談は登録義務なし、`Consultation Relay` の 2 欄は `none`）に替える。変更後の template に `§5.5` の語は残らない（`rg -n '§5\.5' docs/templates/plan-packet.md` が 0 行、baseline 2 行）。

### PR3 / PR4 / PR5 との file の所有

| file / 範囲 | 所有 | 本 PR の扱い |
|---|---|---|
| `docs/AGENT_OPERATING_MANUAL.md` 全体 | PR2 | S1。見出し `## 座組` を必ず置く（両 PR で固定した anchor = `docs/AGENT_OPERATING_MANUAL.md#座組`）。既存の見出しは変えない |
| `docs/DEV_WORKFLOW.md` の `:83`（括弧内だけ）・`:215`・`:252-268`・`:280`（後半 1 文だけ）・`:289`（語だけ）・`:349` | PR2 | S2 |
| `docs/DEV_WORKFLOW.md` のその他の行 | PR4（PR3 は DEV_WORKFLOW を編集しない前提。問い合わせの行き先の表の AGENTS への統合で DEV_WORKFLOW 側を消すのは PR4） | 触らない |
| `docs/agent-guidance/README.md`・`shared.md`・`model-notes.md` | PR2 | S3 |
| `docs/agent-guidance/merge-evidence.md`・`evals/`・`profiles/` | どの PR も本改訂では触らない（merge-evidence は PR1 で縮約済み） | 触らない |
| `docs/templates/subagent-review-packet.md` | **PR2**（PR3 の「review 系 template の統合」のうちこの file は PR2 が持つ） | S4 |
| `docs/templates/pr-review-prompt.md`、`docs/code_review.md`、`docs/quality/review-checklist.md` | PR3 | 触らない。pr-review-prompt は PR3 が `code_review.md` へ統合して削除するので、subagent-review-packet への移入は要らない（PR4 の作業にもならない）。確信度の要求の `code_review.md` への追加も PR3（S6） |
| `docs/decision-log.md` | PR2 は D-092 だけ、PR3 は D-093 だけ（いずれも末尾追記。D-091 は PR #98 で merge 済み） | S5。末尾の追記どうしの衝突は merge 順で両方を残す |
| `docs/templates/plan-packet.md` の `:34`・`:125`（§5.5 の使用指示） | PR2 | S7 |
| `docs/templates/plan-packet.md` のその他の行（Consultation Relay 節の見出しと 2 欄、`:196` の `independent-review` を含む）、`docs/templates/test-design-matrix.md` | PR4 | 触らない |
| `AGENTS.md`、`CLAUDE.md`、`.claude/**`（settings を除く）、`.agents/**`、`.github/pull_request_template.md`、`docs/project-profile.md` | PR3 | 触らない |
| `scripts/**`、`.claude/agents/**`、`.github/workflows/**`、`docs/archive/**` | PR5 / どの PR も本 PR では触らない | 触らない |
| `docs/Plans.md` | 各 PR が登録行と closeout だけ | 登録行（本 commit）と closeout |

本 PR の merge で古くなる他 PR 所有の参照（本 PR では直さず、所有 PR が直す）:

- `CLAUDE.md` の `## Sonnet / Opus の effort` 節と `.claude/commands/plan-rally.md:23` の `consultation relay` → PR3。
- MANUAL `## 座組` の Human Gate 行が参照する委任の範囲の文（push・Draft PR 作成と、owner の承認後の helper `ready` / `merge` の実行）→ PR3 が `AGENTS.md` Decision and Approval Boundaries に書く。
- review の出力の確信度の要求 → PR3 の S6 が `code_review.md` の `## Output Shape` に足す（本 PR の S4 は review packet から出力の規範を外し、code_review を参照する）。
- `docs/templates/plan-packet.md` の Consultation Relay 節の見出しと 2 欄、`:196` → PR4（§5.5 の使用指示の `:34`・`:125` は本 PR の S7）。
- `docs/backlog.md:127` の「AGENT_OPERATING_MANUAL §5.7 候補」（新しい runbook の置き場の候補番号）→ closeout で entry の文を見直す（backlog は実装 PR で編集しない）。

古くならない参照: `.agents/skills/inventory-workflow-start/SKILL.md:13` の §3 の anchor と同じ行の §3.3 の見出し語、DEV_WORKFLOW `:68`・`:286` の §5.6 の anchor、`:280` の「§3.5」は、本 PR が既存の見出しと節番号を変えないので有効のまま（PR2・PR3 とも触らない）。`AGENTS.md:21` の profile の文も、profiles を変えないので有効のまま。

## Non-scope

- 上の所有表で PR3 / PR4 / PR5 のものとした file と行すべて。
- PR3 の merge までの並存: `CLAUDE.md` の `## Sonnet / Opus の effort` 節（high / xhigh）が残り、MANUAL `## 座組` の Human Gate 行が参照する委任の文（AGENTS）と review の出力の確信度（code_review）は PR3 で入る。本 PR で `CLAUDE.md` を触ると PR3 と同じ節を取り合うので触らない。PR2 → PR3 は続けて merge する。
- merge の順: PR2 merge → PR2 の closeout → PR3 に main を同期 → PR3 の Final Review → Ready。helper は PR head の active packet が 1 つであることを要求する（`scripts/pr-gate.py:231`）。
- `docs/DEV_WORKFLOW.md` Review Rules の `:348`（R3 の review-only 既定）・`:350-351`（review-only skip・R4）・3 分類・Findings Freeze、Owner Effort Budget（`:270-281` のうち `:280` の後半 1 文以外）、問い合わせの行き先（`:282-296` のうち `:289` の語以外。`:286` の anchor は見出しを変えないので触らない）、Wave Operation（`:228` 以降。`:237` の「Plan Reviewer と Final Reviewer は lane ごとの独立 fresh context」を含む）: PR4。
- MANUAL の全面的な縮約（残す節の書き直し、節番号の振り直し、§5.2・§5.3 の整理）と、`docs/agent-guidance/` の `shared.md`・profiles の README への統合: 別 PR（closeout で backlog に 1 行、S6）。
- depth 1 の機械化（settings の env `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH`）: PR5。本 PR は文書の規則（MANUAL・DEV_WORKFLOW Subagent Budget・review packet の `## Role`）。
- `docs/agent-guidance/evals/*` の本文。eval fixture は keep（監査 F15、低優先）。
- `docs/research/2026-07-28-codex-sol-week-playbook.md` の D-056 への言及（研究記録で、規範ではない）。
- `docs/archive/**`、decision-log の既存 entry の本文（追記型）。
- 監査 K13 / K14（保守者可読性、店の事実の照合）の追加: PR3（`docs/code_review.md`）。
- 数値の token 削減量の測定（監査 §10、`未実測`）。

## Acceptance Criteria

baseline は本 branch の HEAD（base `e7c22f8f` に本 packet・Matrix・`Plans.md` の登録行を足しただけ）で同じ command を実行した出力（2026-09-27 に実測）。

- **AC1** `rg -c '^## 座組$' docs/AGENT_OPERATING_MANUAL.md` → `1`（baseline: 一致なし、exit 1）。
- **AC2** 退役した仕組みが本 PR の所有 file に残らない: `rg -n 'fable-window|dual-vendor-no-fable|codex-only|Execution Mode|相談窓口|高自律|低制約適性|希少|D-056|D-058|D-079|D-084|D-087|Consultation Relay|independent-review|Sonnet|GPT-5\.6|Sol 5\.6|Opus 5（|Fable 5（|Evidence Mode|legacy|三点一致|state-only|vendor 可用性' docs/AGENT_OPERATING_MANUAL.md docs/agent-guidance/README.md docs/agent-guidance/shared.md docs/agent-guidance/model-notes.md docs/templates/subagent-review-packet.md | wc -l` → `0`（baseline `42`）。`rg -c 'pr-review-prompt' docs/AGENT_OPERATING_MANUAL.md` → 一致なし（baseline `1`）。
- **AC3** DEV_WORKFLOW: `rg -n 'Astra主担当|Writer が Codex（発注書駆動の実装者）|Max concurrent sub-agents|全 lane 合算の同時 subagent 上限|vendor-oriented Execution Mode|実行 mode 上の裁定者' docs/DEV_WORKFLOW.md | wc -l` → `0`（baseline `6`）。残すもの: `rg -n 'Max delegation depth is 1|One-writer rule|Sub-agent output contract|Load-bearing decisions|P3-only findings' docs/DEV_WORKFLOW.md | wc -l` → `5`（baseline `5`）。`rg -c '^## Subagent Budget$' docs/DEV_WORKFLOW.md` → `1`（baseline `1`）。`:83` の旧文: `rg -c 'never in normative rules' docs/DEV_WORKFLOW.md` → 一致なし（baseline `1`）。
- **AC4** 独立性の規則: `rg -c '他の reviewer' docs/AGENT_OPERATING_MANUAL.md` と `rg -c '他の reviewer' docs/templates/subagent-review-packet.md` がそれぞれ `1` 以上（baseline: どちらも一致なし）。`rg -c '別の fresh context' docs/AGENT_OPERATING_MANUAL.md` → `1` 以上（baseline: 一致なし）。`rg -c 'Coordinator の指名では不可' docs/AGENT_OPERATING_MANUAL.md` → `1`（baseline: 一致なし）。文面が S1 (2) の要素（(b) の対象 = 同じ round で 2 本以上の review を回すときの各 reviewer〈Plan Review・Final Review とも。発注の順に依らない〉、読ませない対象 = 他の reviewer の結果が置かれた場所すべてと例示 4 種〈local の報告 file を含む〉、その発注の本文に他の reviewer の結果とそれ由来の観点を書かないこと〈closure・是正の発注は除く〉、転記の順序、読んだ run の取り直し、Codex の broad が round をまたいで pending のときの進め方〈Codex の broad が返った後、Claude 側も同じ HEAD で broad を取り直すことを含む〉、(c) の Writer が Codex の場合、(e) の Plan Reviewer と Final Reviewer の別 context、§3.3 の Codex の枠の代替と Plan Review の Codex 分の例外）を含むことを review で確認する。
- **AC5** `rg -c '^## D-092: ' docs/decision-log.md` → `1`（baseline: 一致なし）。`awk '/^## D-092: /,0' docs/decision-log.md | rg -o 'D-0(56|58|79|84|87)' | sort -u | wc -l` → `5`。追記だけ: `git diff origin/main...HEAD -- docs/decision-log.md | rg -c '^-[^-]'` → 一致なし。
- **AC6** agent-guidance: `find docs/agent-guidance -mindepth 1 | sort` → baseline と同じ 11 行（`README.md`、`evals` と 2 file、`merge-evidence.md`、`model-notes.md`、`profiles` と 3 file、`shared.md`）。`git diff --stat origin/main...HEAD -- docs/agent-guidance/merge-evidence.md docs/agent-guidance/evals docs/agent-guidance/profiles` → 空。`rg -c '^## Effort の選定$' docs/agent-guidance/model-notes.md` → `1`（baseline `1`）。
- **AC7** review packet: `rg -n 'legacy|state-only|三点一致' docs/templates/subagent-review-packet.md | wc -l` → `0`（baseline `1`）。出力の規範を持たない: `rg -n '確信度|P1/P2/P3' docs/templates/subagent-review-packet.md | wc -l` → `0`（baseline `2`: `:7`、`:32`）、`rg -c 'code_review\.md' docs/templates/subagent-review-packet.md` → `1` 以上（baseline `1`）、`## Output` の参照替え: `awk '/^## Output/,0' docs/templates/subagent-review-packet.md | rg -c 'code_review'` → `1` 以上（baseline: 一致なし、exit 1。`:3` の既存の参照だけでは満たさない）。`:26` の置換: `rg -c '廃止済みの SHA 照合を要求しない' docs/templates/subagent-review-packet.md` → `1`（baseline: 一致なし）。reviewer の姿勢の 2 句を残す（句ごとに数え、行の割り方に依らない）: `rg -c '黙って落とさない' docs/templates/subagent-review-packet.md` と `rg -c 'を残したまま' docs/templates/subagent-review-packet.md` がそれぞれ `1` 以上（baseline: どちらも `1`）。read-only 宣言の depth 1: `rg -c 'depth 1' docs/templates/subagent-review-packet.md` → `1` 以上（baseline: 一致なし）。
- **AC8** MANUAL を指す anchor が実在する: (i) §5.6 の既存 2 件の保持: `rg -o 'AGENT_OPERATING_MANUAL\.md#[^) ]+' docs -g '!docs/archive/**' -g '!docs/decision-log.md' -g '!docs/plans/**' | rg -v '#座組$'` → 2 行で、どちらも `#56-従来型-writer-発注書の共通出力契約`（baseline 3 行: `docs/DEV_WORKFLOW.md:68`・`:286` の同じ anchor と `:215` の `#astraを主担当にする場合d-087`。`:215` は S2 で消える）。(ii) 新しい `#座組` の参照: `rg -c 'AGENT_OPERATING_MANUAL\.md#座組' docs/DEV_WORKFLOW.md` と `rg -c 'AGENT_OPERATING_MANUAL\.md#座組' docs/agent-guidance/model-notes.md` がそれぞれ `1` 以上（baseline: どちらも一致なし。S2 `:83`・S3 の link）。(iii) 参照先の見出し: `rg -c '^## 座組$' docs/AGENT_OPERATING_MANUAL.md` → `1`（AC1 と同じ）。(i)〜(iii) は S2・S3 を base の写しに仮に当てて 2 行・1・1 になることを起票時に確かめた（2026-09-27、`$TMPDIR` の写し。tracked file は変えていない）。残る anchor は AC10 で保つ §5.6 の見出しから GitHub の slug 規則で作られるものと一致する（link 検査は anchor を見ないので review で確認する）。`.agents/skills/inventory-workflow-start/SKILL.md:13` の §3 の anchor も AC10 で保つ見出しを指す。
- **AC9** 他 PR の所有 file に触れない: `git diff --stat origin/main...HEAD -- AGENTS.md CLAUDE.md .claude .agents .github .codex scripts docs/code_review.md docs/quality docs/project-profile.md docs/ci.md docs/templates/pr-review-prompt.md docs/templates/test-design-matrix.md docs/templates/adr.md docs/templates/workflow-effectiveness-review.md docs/archive` → 空。`git diff -U0 origin/main...HEAD -- docs/templates/plan-packet.md` の hunk が `:34`・`:125` の中だけにある。`git diff -U0 origin/main...HEAD -- docs/DEV_WORKFLOW.md` の hunk が S2 の 6 か所（旧 `:83`、`:215`、`:252-268`、`:280`、`:289`、`:349`）の中だけにある（hunk の位置は review で確認）。
- **AC10** 既存の見出しを変えない: `rg -c '^## 3\. Role Assignment（役割割当の制約とAvailability）$|^### 3\.3 Capacity-degraded（役割担当の一時不能）$|^### 3\.5 Task Shape: one-shot irreversible$|^### 5\.6 従来型 Writer 発注書の共通出力契約$|^## 6\. ハーネス間の既知の非対称（重要な注意）$|^### 6\.1 Claude project hook の所有境界$' docs/AGENT_OPERATING_MANUAL.md` → `6`（baseline `6`）。削除する節の見出しが消える: `rg -c '^### 3\.[124] |^### 5\.[57] |^#### ' docs/AGENT_OPERATING_MANUAL.md` → 一致なし（baseline `6`: §3.1・§3.2・§3.4・§5.5・§5.7 と `#### Astraを主担当にする場合（D-087）`）。残す §5.1〜§5.4 の見出し: `rg -c '^### 5\.[1-4] ' docs/AGENT_OPERATING_MANUAL.md` → `4`（baseline `4`）。
- **AC11** 検査: `bash scripts/doc-consistency-check.sh`、`bash scripts/doc-consistency-check.sh --target plan`、`bash scripts/check-workflow-git.sh`、`bash scripts/tests/run-workflow-tests.sh`（`claude-hooks.test.sh`・`doc-consistency-plan-packet.test.sh:842-843`・`reading-order-drift.test.sh` を含む）、`git diff --check` がすべて exit 0。
- **AC12** 残す境界の文字列と effort の所在: `rg -c '不可逆 mutation の直前 gate に同席し' docs/AGENT_OPERATING_MANUAL.md` → `1`（baseline `1`。§3.5 `:110` を変えない）。`awk '/^### 5\.6/,/^## 6\./' docs/AGENT_OPERATING_MANUAL.md | rg -c '^[123]\. '` → `6`（baseline `9`: §5.6 の作成・訂正の手順 1〜3 と止まったときの 1〜3、および削除する §5.7 の 1〜3）。`rg -c 'Writer が編集前に止まったとき' docs/AGENT_OPERATING_MANUAL.md` → `1`（baseline `1`）。`rg -c '^7\. 実行 mode' docs/AGENT_OPERATING_MANUAL.md` → 一致なし（baseline `1`）。`rg -c '値としてのみ現れる' docs/AGENT_OPERATING_MANUAL.md` → 一致なし（baseline `1`）。`rg -n '\b(medium|high|xhigh)\b' docs/agent-guidance/model-notes.md | wc -l` → `0`（baseline `2`: `:7`、`:9`）。`rg -n 'CLAUDE\.md#' docs/agent-guidance/model-notes.md | wc -l` → `0`（baseline `1`: `:9` の `CLAUDE.md#sonnet--opus-の-effort`）。

## Design Sources

- Workflow 正本（本 PR が書き換えるもの自身）: `docs/AGENT_OPERATING_MANUAL.md`、`docs/DEV_WORKFLOW.md` の Workflow State（`:83`）/ Implementation Rules / Subagent Budget / Owner Effort Budget / Review Rules、`docs/templates/subagent-review-packet.md`、`docs/templates/plan-packet.md:34`・`:125`、`docs/agent-guidance/{README,shared,model-notes}.md`。
- 参照だけする正本: `docs/code_review.md` の `## Output Shape`・`## Finding Severity`・`## Verification Rules`（closure の読み方）・`## Review-only Sub-agent Protocol`（R3 の review-only skip）。PR3 の所有で行が動くので、節名で参照する。
- Decision log: D-034、D-038、D-056、D-058、D-059、D-062、D-065、D-079、D-084、D-087、D-090。
- 監査: `.local/reports/harness-audit-2026-09-24.md`（local-only）§3.F / 3.G / 3.H / 3.K / 3.N / 3.O、§4、§5、§7 PR2 行、§8、§9 Q2（5 本で決定済み）。
- PR1 archive packet: `docs/archive/plans/2026-09-24-harness-legacy-and-execution-mode-removal.md`（Non-scope で PR2 へ送ったもの = MANUAL の §3.1 / §3.2 / §3.3 / §3.5 / §5.5 と DEV_WORKFLOW の旧 `:312,381` = 現 `:280,:349`、decision-log の superseded の列挙）。
- 公式資料（取得版 `.local/reports/official-guides/`、URL が正本）: https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-opus-5-5 「Calibrate effort」、https://platform.claude.com/docs/en/build-with-claude/effort 「Recommended effort levels for Claude Opus 5.5」、https://code.claude.com/docs/en/sub-agents 「How forks differ from other subagents」「Supported frontmatter fields」（`effort`）「Let subagents spawn their own subagents」。
- 公式資料の追加（owner 2026-09-25「Fable5.1のガイドも盛り込んでよさそう」、取得版は同じ directory）: https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5-1 （座組の相談役と、Fable を reviewer・相談役に置く発注書の書き方に関わる節: 「Consider all effort levels」「Ask for user-facing progress updates」「Batch independent tool calls in agent loops」「Finish the whole task」「Keep changes and tests to what the task asks for」「Let the lead agent keep working while subagents run」）、https://code.claude.com/docs/en/advisor （advisor tool、experimental。main が Opus 5.5 のとき advisor は Fable か Opus 5 以降、有効化は owner の `/advisor`・`advisorModel`・`--advisor`、呼ぶ時機は Claude が決める。Max では Fable の使用は週の上限の 50% まで plan 内で、超えると usage credits）。`#座組` の相談役の行と `model-notes.md` は、Opus 5.5 guide と同じく Fable 5.1 guide と照合する。advisor は座組の必須にせず、使う場合の組合せと費用の注意を相談役の行の注記に 1 行で置く。

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | なし | existing sufficient（製品コードを変えない） |
| Command / DTO / generated binding / wire shape | なし | existing sufficient |
| DB / transaction / audit / rollback / migration | なし | existing sufficient |
| Screen / UI / route state / Japanese wording | なし | existing sufficient |
| CSV / TSV / report / import / export format | なし | existing sufficient |
| Workflow 正本（役割・独立性・座組） | `docs/AGENT_OPERATING_MANUAL.md`、`docs/DEV_WORKFLOW.md` の S2 の 6 か所、`docs/agent-guidance/{README,shared,model-notes}.md`、`docs/templates/subagent-review-packet.md`、`docs/templates/plan-packet.md:34`・`:125` | updated in this PR（S1〜S4、S7） |
| Durable decision / ADR | `docs/decision-log.md` D-092 | updated in this PR（S5） |

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| MANUAL の節の削除（§3.1・§3.2・§3.4・§5.5・§5.7） | 削除した節を節番号・anchor で指す参照を `rg -n '§3\.[124]|§5\.[57]|#astraを主担当' --hidden -g '!.git/**' -g '!docs/archive/**' -g '!docs/decision-log.md' -g '!docs/plans/**' .` で洗い（base `e7c22f8f` の当たり: MANUAL 内、DEV_WORKFLOW `:215`・`:349`、template `:34`・`:125`、`docs/backlog.md:127`。他の当たりは別文書の同じ番号の節）、本 PR の所有分を S1〜S2・S7 で直す。backlog は closeout で見直す |
| MANUAL の見出し | 既存の見出しは変えず（例外 = §5.4 の括弧書き。§5.4 の anchor を指す参照は 0 件）、`## 座組` を足すだけ。DEV_WORKFLOW `:68`・`:286` と `.agents/skills/inventory-workflow-start/SKILL.md:13` の anchor は変わらない（AC8・AC10） |
| MANUAL の文字列を検査する test | `doc-consistency-plan-packet.test.sh:842-843`（`one-shot irreversible` / `task-shape`）と `claude-hooks.test.sh:39-46`（禁止句）を満たす文面を保つ。test は変えない |

Tauri command・REQ・route・画面は該当なし。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| SPEC-WF-HARNESS2 | MANUAL §3 / §3.4、DEV_WORKFLOW `:83`、model-notes、owner 決定 2026-09-07 / 08・14・23 / 24 / 25 | D1 | 座組と effort の値を表 1 か所に置き、モデル更改時の書換えを 1 か所にする。却下: model-notes・CLAUDE.md にも値を残す（2 か所が食い違う） | S1 `## 座組`、S2 `:83`、S3 model-notes | AC1、AC12、T-F2 |
| SPEC-WF-HARNESS2 | MANUAL §3 の独立性、DEV_WORKFLOW `:237`・`:349`、PR #97・#94 の dogfood 所見 | D2 | 同じ round の各 reviewer に他の reviewer の結果を見せない規則を、置き場所を列挙して文書の規則として置く。却下: 機械の検査にする（非目的） | S1 (2)（§3 の独立性の項と §3.3）、S4 `## Target` | AC4、T-I1 |
| SPEC-WF-HARNESS2 | MANUAL §3.1 / §3.2 / §3.4 / §5.5 / §5.7、DEV_WORKFLOW `:215`・`:252-268`・`:289`・`:349`、template `:34`・`:125` | D3 | Execution Mode 時代の役割規則を退役させ、D-092 に記録する。却下: 注記付きで残す（読み手が旧手順を実行する） | S1、S2、S3、S4、S5、S7 | AC2、AC3、AC5、AC7 |
| SPEC-WF-HARNESS2 | MANUAL の §2 / §3 / §3.3 / §3.5 / §5.4 / §5.6 / §6 / §6.1、DEV_WORKFLOW Subagent Budget、owner 決定 2026-09-25（範囲の縮小） | D4 | 残す節は書き直さず、退役した仕組みへの言及だけを除く。見出しと節番号を保ち anchor を切らない（Contract Coverage Ledger）。却下: 本 PR で MANUAL 全体を縮約する（owner 2026-09-25 に外した。別 PR） | S1、S2 | AC3、AC10、AC12、T-S1〜T-S5 |
| SPEC-WF-HARNESS2 | `docs/templates/subagent-review-packet.md`、`docs/code_review.md`、MANUAL §5.4 | D5 | review の出力の規範は code_review 1 か所。review packet は出力の規範を持たず、reviewer の姿勢の 2 句と発注の入力欄だけを持つ。発注の型は MANUAL §5.4 に残す。却下: review packet に出力形式を残す（2 か所の規範）、§5.4 を review packet へ移す（残す節の書き直しになる） | S4 | AC7、T-P1 |
| SPEC-WF-HARNESS2 | owner 決定 2026-09-25、D-062 の原則 | D6 | Final Review の Claude 側を R3 以上と design lane で Fable 5.1、closure の既定を Fable 5.1 にし、Codex の合否を是正後も外さない。Codex の枠の代替は Codex の別 model だけ。実効 model を記録する。却下: Fable を相談役だけに置く（owner 決定と違う） | S1 `## 座組`、S1 (2) の §3.3 | T-F2 |
| SPEC-WF-HARNESS2 | 所有表、PR3 packet、PR4 / PR5 の範囲 | D7 | 並走 PR と file・行を取り合わない。例外は template の 2 行だけで、本 PR の §5.5 の削除が原因 | 所有表、Non-scope | AC9、T-B1 |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 座組・独立性・役割担当の一時不能は MANUAL に、退役と維持の列挙と理由は D-092 に置く。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: 「座組と effort の値は `#座組` の表だけ」「同じ round の各 reviewer に他の reviewer の結果を見せない」「Final Review の Claude 側は R3 以上で Fable 5.1」「Codex の Final Review の枠は Codex の別 model でのみ代替」「review packet は出力の規範を持たない」を MANUAL・review packet・D-092 に置く。
- Assumptions and constraints: PR3 が `CLAUDE.md` の effort 節、AGENTS の委任の文、code_review の確信度を続けて入れる（Non-scope の並存）。Writer・レビューの medium は PR5 の定義 file まで実効しない（座組表の effort 列の cell で正直に書く）。MANUAL の節番号に欠番が残る（縮約は別 PR）。
- Deferred design gaps, risk, and follow-up target: subagent ごとの effort を定義 file で固定すること（PR5）、plan-packet template の Consultation Relay 節と `independent-review`（PR4）、DEV_WORKFLOW の他の節（PR4）、MANUAL の全面的な縮約と agent-guidance の統合（別 PR、closeout で backlog）。
- Test Design Matrix can cite design decision IDs or source doc sections: D1〜D7 と失敗定義の境界を Matrix が引く。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 所有の例外（template `:34`・`:125`）を所有表と AC9 に書いた。Codex 停止中の Final Review は Ready 以降だけを止め、Codex を外す例外は owner の明示決定に限る。

## Impact Review Lenses

not applicable: 実機調査・外部 tool・POS・CSV・operator workflow を起点にしない workflow 文書の変更で、製品の設計前提を変えない。

## Design Readiness

- Existing design docs are sufficient because: 規則の中身は owner 決定（2026-09-07 / 08・14・23 / 24 / 25）・2026-09-25 の dogfood 所見・監査で決まっており、本 PR はそれを workflow 正本へ書く。owner の判断を要する未決の論点は無い。
- Source docs updated in this PR: S1〜S5、S7 の file。
- Design gaps intentionally deferred: subagent ごとの effort を定義 file で固定すること（PR5）、plan-packet template の Consultation Relay 節と `independent-review` の残り（PR4）、MANUAL の全面的な縮約（別 PR）。
- Durable decisions discovered in this plan and promoted to source docs: 独立性の規則「同じ round の各 reviewer に他の reviewer の結果を見せない」（S1 (2) (b)、S4）と D-092。

## Contract Probe

Scope の前提 3 つを起票時に確かめた。

- P1 link 検査は `docs/` 配下（`docs/archive/**` を含む）の Markdown だけを見て、link 先 file の実在を確かめる: `docs/agent-guidance/README.md` を一時的に退避して `bash scripts/doc-consistency-check.sh` → `AGENT_OPERATING_MANUAL.md:90` と `:119` の 2 件が `リンク先 'agent-guidance/README.md' が存在しません` の ERROR（`8bd2bc9a`、退避を戻して `git status --porcelain` 空を確認）。検査対象は `find "$ALL_DOCS" -name "*.md"`（`scripts/doc-consistency-check.sh` の Markdown link 検査）で、root の `AGENTS.md` は含まれない。
- P2 link 検査は anchor を落として file だけを見る: 同じ関数が `link_path=$(echo "$link_path" | sed 's/#.*//')` で anchor を除く（`scripts/doc-consistency-check.sh:1710-1711`）。したがって anchor の食い違いは機械では落ちず、AC8 の review で見る。
- P3 subagent の effort は定義 file で指定できる: 公式 sub-agents「Supported frontmatter fields」の `effort`（「Overrides the session effort level. Default: inherits from session」、取得版 `claude-code-sub-agents.md:311`）。監査 §10 が「公式資料では未確認」とした点はこれで確認済み。定義 file の新設は PR5。

## Contract Coverage Ledger

失敗定義の「残す境界」を 1 行ずつ、MANUAL（または DEV_WORKFLOW）の節 → 本 PR の後の置き場所 → 検査 / review で写す。行番号は base `e7c22f8f`。

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| Writer ≠ Plan Reviewer、Writer ≠ Final Reviewer（MANUAL §3 `:32-33`） | §3 の既存項目（変えない） | なし（文書の規則） | review: T-I1 |
| Plan Reviewer と Final Reviewer は別の fresh context（DEV_WORKFLOW Wave Operation `:237`） | §3 に足す (e)（同じ model・vendor でよい） | AC4（`別の fresh context`） | review: T-I1 |
| 自己承認禁止・fresh context（MANUAL §3 `:34`） | §3 の既存項目、(a) fork を独立に数えない | なし | review: T-I1 |
| Double Audit（MANUAL §3 `:35`、DEV_WORKFLOW Contract Audit） | §3 の既存項目と (b) 同じ round の各 reviewer に他の reviewer の結果を見せない（Plan Review を含む。Codex の broad が round をまたぐ場合と、その後の Claude 側の broad の取り直しを含む） | AC4（`他の reviewer` の語） | review: T-I1（読ませない場所の列挙） |
| D-062(c) の vendor 独立性（DEV_WORKFLOW `:349`） | §3 に足す (c)（Writer が Codex の packet は別 vendor の Plan Reviewer を含める）、DEV_WORKFLOW `:349` の参照 1 行 | AC3（`Writer が Codex（発注書駆動の実装者）` = 0） | review: T-I1 |
| 役割担当の一時不能の pending と、代替が決まらないときの Plans.md の blocker への記録（MANUAL §3.3 `:83-86`、D-062(c) の capacity pending の句の受け皿） | §3.3 の既存 4 項目（変えない）と、足す 2 文（Codex 停止中は `## 座組` の列、Codex の Final Review の枠の代替） | AC4（`Coordinator の指名では不可`） | review: T-F2 |
| Human Gate は owner（MANUAL §2 の Human Gate 行、§3 `:36`） | §2・§3 の既存の行（変えない）、`## 座組` の Human Gate 行（委任の範囲は `AGENTS.md` の参照） | なし | review: T-F2。委任の文は PR3 |
| depth 1（DEV_WORKFLOW Subagent Budget `:264`） | DEV_WORKFLOW `## Subagent Budget`（残す）、MANUAL `:274` の文、review packet `## Role` の read-only 宣言 | AC3 後半（5 項目 = 5）、AC7（`depth 1`） | 機械化（settings の env `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH`）は PR5。本 PR は文書の規則 |
| one-writer（DEV_WORKFLOW Subagent Budget `:265`、MANUAL §2 の Writer 行） | DEV_WORKFLOW `## Subagent Budget`、§2 の Writer 行（変えない）、MANUAL `:274` の文 | AC3 後半 | — |
| load-bearing 判断の正本直読み（DEV_WORKFLOW Subagent Budget `:267`） | DEV_WORKFLOW `## Subagent Budget`、§3 に足す (d) | AC3 後半 | — |
| hook 0 本と plugin 無効化（MANUAL §6.1） | §6.1（変えない） | `claude-hooks.test.sh`（T-S2） | — |
| advisory hook の推定禁止、hook は review を強制しない（MANUAL §6.1 `:285`・`:286`） | §6.1（変えない） | なし | review: T-S5 |
| one-shot irreversible の task-shape（MANUAL §3.5） | §3.5（見出しと `:107` の `task-shape` の語を保ち、Execution Mode の句だけ除く）、DEV_WORKFLOW `:280` の link と「§3.5」 | `doc-consistency-plan-packet.test.sh:842-843`（T-S1） | — |
| 不可逆 mutation の直前の owner 判断（MANUAL §3.5 `:110`） | §3.5 `:110`（変えない） | AC12（`不可逆 mutation の直前 gate に同席し` = 1、T-S4） | — |
| Writer が編集前に止まったときの手順 1〜3（MANUAL §5.6 `:242-246`、DEV_WORKFLOW `:286` が「2〜3」を指す） | §5.6（item 7 の削除と 8・9 の繰り上げだけ。1〜3 は変えない）、DEV_WORKFLOW `:286` の anchor（変えない） | AC12（awk の番号 = 6、見出し語 = 1、`^7\. 実行 mode` = 0、T-S5）、AC10（§5.6 の見出し） | review: AC8 の anchor |
| 発注直前の照合（必須 command の出力先が編集禁止範囲に入らない。MANUAL §5.6 作成・訂正の手順 4 `:234`） | §5.6 `:234`（変えない） | なし | review: T-S5 |
| 座組・effort の値の所在（D1） | `## 座組`、MANUAL `:15`・`:28`、DEV_WORKFLOW `:83`、model-notes（`:9` の CLAUDE.md への link を `#座組` へ付け替え） | AC1、AC3（`:83`）、AC8 (ii)（DEV_WORKFLOW・model-notes の `#座組` の参照）、AC12（`値としてのみ現れる` = 0、model-notes の値 = 0、`CLAUDE\.md#` = 0） | review: T-F2 |
| review packet は出力の規範を持たず、reviewer の姿勢の 2 句を残す（D5。現行 `:7`・`:32`） | S4 の `## Role`・`## Output` | AC7 | 確信度の code_review への追加は PR3 |

adjacent-contract sweep: S1〜S4、S7 が触る節で上表に無い契約は、MANUAL §5.1〜§5.4 の実機調査・backfill・Plans cleanup・発注の型（§5.4 は見出しの括弧書きと `:175`・`:181`・`:183` の句以外を変えない、review で確認）と、DEV_WORKFLOW の S2 以外の行（変えない、AC9）。

## Test Plan

Test Design Matrix: [2026-09-25-harness-pr2-roles-and-formation.md](test-matrices/2026-09-25-harness-pr2-roles-and-formation.md)。

- targeted tests: AC1〜AC8、AC10、AC12 の `rg` / `find` / `git diff`。
- negative tests: Matrix の Mutation-style（`one-shot irreversible` を消す → `doc-consistency-plan-packet.test.sh` が red、禁止句を入れる → `claude-hooks.test.sh` が red、`README.md` を消す → link 検査が ERROR、直前の owner 判断の文を消す → AC12 が 0 になる）。
- compatibility checks: active な他 packet は並走 lane の branch にだけある（main には無い）ので、本 branch の `--target plan` は本 packet だけを見る。PR3 の起草 branch と file が重ならない（AC9）。
- data safety checks: 実データ・secret を含まない文書変更（Data Safety）。
- main wiring/integration checks: `bash scripts/tests/run-workflow-tests.sh`（policy docs で workflow=true のとき hosted でも走る suite）。

## Boundary / Wire Contract

not applicable: JSON・browser state・CSV・config・DTO・bindings・DB を変えない。

## Review Focus

- Plan Review の冒頭で、Ordinary Operation の各行が本 PR の merge 後の文書で成立するかを `成立 / 具体的な反例あり / 外部前提が未確認` で答える。
- 座組表の中身が owner 決定（2026-09-07 / 08・14・23 / 24 / 25）と一致するか。特に Codex 停止中の Plan Review と Final Review の非対称、Fable を相談役としては reviewer の数に入れないことと Final Review の Claude 側に置くことの書き分け、effort の値が表の cell にだけあること、Human Gate の委任の参照先。
- S1 (2) (b) の独立性の規則が PR #97・#94 の崩れ方（packet の Review Response と PR body を読ませた）を実際に防ぐか。読ませない場所の列挙（PR の comment / review、local の報告 file、各 reviewer の発注の本文）が過不足ないか。
- 削る規則のうち、実は安全境界や独立性を担っていたものが無いか（§3.1 の希少 slot、§3 の Opus read-only、§5.5、§5.7、Subagent Budget の数値、D-062(c)）。S1 の「削除する節」と「削除に伴い直す行」の 2 表に過不足が無いか、残す節に書き直しが紛れていないか。Contract Coverage Ledger の各行が残す節か足す項目で拾えているか。
- Risk R3 の判定と、R3 の review-only を Final Review の Double Audit の broad が兼ねる扱い。
- 所有表が PR3 の起草（`.claude/worktrees/harness-closeout`、branch `agent/harness-pr3-entry`）と重ならないか。特に `docs/templates/subagent-review-packet.md`（PR2）と pr-review-prompt・code_review（PR3）、DEV_WORKFLOW を PR3 が触らない前提、template の 2 行の例外。
- 本 PR の Writer（Opus）が Opus を縛る規則を書くこと（D-062 の原則「縛られる側に gate 文と検査器を書かせない」）: Final Review に Codex を含めることで別 vendor の目を置く。これで足りるか。

## Spec Contract

Contract ID: SPEC-WF-HARNESS2

- D1: 座組（担当・model・effort・Codex 停止中の進め方）の tracked の正本は `docs/AGENT_OPERATING_MANUAL.md` の `## 座組` の表 1 つで、effort の値も担当ごとにこの表の effort 列の cell だけに置く。`docs/agent-guidance/model-notes.md` は値を持たず表を参照し、`docs/DEV_WORKFLOW.md:83` と MANUAL `:15` は、規範の文では model 名が Workflow State の値と座組表にだけ現れるとする。
- D2: 独立性は MANUAL §3 の既存 5 項目に、(a) fork を独立に数えない、(b) 同じ round で 2 本以上の review を回すときの各 reviewer に他の reviewer の結果が置かれた場所すべてを読ませず発注の本文にも書かない（Codex の broad が返った後は Claude 側も同じ HEAD で broad を取り直す）、(c) Writer が Codex の場合の別 vendor の Plan Reviewer、(d) load-bearing 判断の正本直読み、(e) Plan Reviewer と Final Reviewer は別の fresh context、を足す。§3.3 は Codex の Final Review の枠を Codex の別 model でのみ代替し、Claude での代替は owner の明示決定に限る。review packet の `## Target` も同じ読ませないものを持つ。
- D3: 退役した仕組み（Execution Mode、D-056、希少 slot、§5.5、§5.7、Astra 主担当、Subagent Budget の数値、D-062(c) の `codex-only` 句）を手順として指示する文が本 PR の所有範囲に残らず、D-092 が退役・部分退役・PR1 撤去済み・維持を列挙する。
- D4: 失敗定義の残す境界は、Contract Coverage Ledger の各行の置き場所に残る。MANUAL の残す節は書き直さず、既存の見出しと節番号を保つ。
- D5: review の出力の規範は `docs/code_review.md`。`docs/templates/subagent-review-packet.md` は出力の規範を持たず、reviewer の姿勢の 2 句（黙って落とさない、P2 を残したまま pass にしない）と発注の入力欄（`## Role` の read-only 宣言と depth 1、`## Target`）を持つ。発注の型は MANUAL §5.4 に残す。
- D6: Final Review の Claude 側は R3 以上と design lane で Fable 5.1、closure の既定も Fable 5.1。Codex の合否は是正後の取り直しでも外さない。run の報告に実効 model を記録し、発注と実効 model が違う run は数えない。
- D7: 本 PR は PR3 / PR4 / PR5 の所有 file と行に触れない（例外: template `:34`・`:125`）。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-WF-HARNESS2-D1 | S1、S2、S3 | AC1、AC3、AC12、T-F1、T-F2 | 座組表と owner 決定の一致、effort の所在 | rg 出力と表の突合 |
| SPEC-WF-HARNESS2-D2 | S1、S4 | AC4、T-I1 | 読ませない場所の過不足 | rg 出力と文面 |
| SPEC-WF-HARNESS2-D3 | S1〜S5、S7 | AC2、AC3、AC5、AC7、T-R1、T-L1 | 退役の取りこぼし | rg 出力 |
| SPEC-WF-HARNESS2-D4 | S1、S2 | AC3、AC10、AC12、T-S1〜T-S5、T-G2 | Ledger の各行、残す節の書き直しの有無 | test 出力・rg 出力 |
| SPEC-WF-HARNESS2-D5 | S4 | AC7、T-P1 | 出力の規範の二重化 | rg 出力 |
| SPEC-WF-HARNESS2-D6 | S1 | T-F2 | Fable の二つの役の書き分け、Codex の枠の代替 | 表の文面 |
| SPEC-WF-HARNESS2-D7 | 所有表、Non-scope | AC6、AC9、AC11、T-B1、T-G1 | 他 PR との重なり | git diff の stat・hunk |

## Data Safety

not applicable（製品データを扱わない）: 変更は tracked の workflow 文書だけで、実 POS / 店舗データ、DB、backup、log、receipt、secret、`.env*` を読まず、commit しない。

- local-only: `.local/`（監査報告、公式資料の取得版、Codex 発注書）。tracked に写さず、URL と節名だけを書く。
- synthetic-only: 該当なし（fixture を作らない。mutation は Writer の作業 worktree で行い、確認後に戻す）。

## Implementation Results

Fill after implementation.

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- Writer（Opus 5.5 subagent）が S1〜S5・S7 を実装した（S6 の Plans.md 登録行は起票 commit、backlog は本 closeout）。MANUAL に `## 座組` の表と独立性の項 (a)〜(e)・§3.3 の 2 文を足し、§3.1・§3.2・§3.4・§5.5・§5.7 を削除、DEV_WORKFLOW の 6 か所・agent-guidance 3 file・review packet・plan-packet template の 2 行を直し、decision-log に D-092 を追記した。AC1〜AC12 と検査は PASS。
- Final Review の是正 1 件（Codex の P3 と Coordinator の指摘 C-1〈P2〉が同じ点）: owner 方針 2026-09-14「総 token で効率を判断」が PR2 + PR3 で tracked から消えるため、MANUAL `## 座組` の表の下の注記に「effort の選び方」（owner 2026-09-14）を置き、model-notes の `## Effort の選定` からその注記を指すようにした。S1 は注記を (1)〜(4) の 4 項目と列挙し、S3 は方針を「座組表の effort 列が引き継ぐ」と書くが、実物は注記 (5) が方針を持つ（列は値だけを持つため）。packet 本文の書換えは broad を無効化するため行わず、ここに記録する。
- CI: Merge gate success（https://github.com/higekiri-w/inventory-system-desktop/actions/runs/36332090301）。Ready・merge は owner 指示（2026-09-28）で helper 経由、[PR #113](https://github.com/higekiri-w/inventory-system-desktop/pull/113)。

## Review Response

Fill after review. Final Review の結果は、Double Audit の 2 本がそろうまで本節と PR body に転記しない（S1 (2) (b)）。
- Findings Freeze: frozen after Broad Audit（両方とも approve）; post-freeze exceptions: none（broad の Codex P3 と同じ点の Coordinator の指摘 C-1 を是正、Fable の P3 は follow-up として backlog へ）.

### Plan Review（2026-09-25〜27、Claude 側 = fresh Opus 5.5、Codex 側 = GPT-6〈round 3 は Astra、high〉、互いに独立）

- round 1〜3 とも両 reviewer が reject。round 1・2 は plan-gate のまま是正（round 2 の後に owner 決定で範囲を縮小）、round 3（上限、`a76a1514`）は同型の一括是正（`6b51e223`、裁定 r3）で全件を反映し、owner 承認のもと予算を改めて plan-approved。経過は遷移記録。

### Final Review（2026-09-27〜28、互いに独立な Double Audit）

- broad（head `24ca59ba`、base `e7c22f8f`）Claude 側（Fable 5.1、fresh）: approve、P1/P2 = 0、P3 3 → 3 件とも PR4 への follow-up（backlog）。① closure の record は audit 1 つで、2 本の closure run のどちらを record するかが文書で決まらない（`scripts/pr-gate.py:436-439`）。② MANUAL の「design lane」が tracked の正本で未定義。③ 読ませない場所の例示に是正 commit の件名が無い。
- broad Codex 側（GPT-6 Astra、発注 107）: approve、P1/P2 = 0、P3 1 = owner 方針 2026-09-14「総 token で効率を判断」が PR2 + PR3 で tracked から消える（Coordinator の追加指摘 C-1〈P2〉と同じ）→ 採用し是正 `f30f3ea7`（Implementation Results）。
- closure（head `f30f3ea7`）Codex 側（GPT-5.6 Sol、発注 111）: approve、P1/P2/P3 = 0、AC1〜AC12 PASS。
- closure Claude 側（Fable 5.1）: approve、P1/P2 = 0、P3 1 = S1 / S3 の記述と注記 (5) の食い違い → 記録で処理（Implementation Results の 2 項目め）。
- helper record: broad を受けた後、record の前に head が `f30f3ea7` へ進んだため、「broad `24ca59ba` + closure `f30f3ea7`」の形では記録できなかった（capture は PR の現 head だけを取る）。owner 決定（2026-09-28）で、現 head `f30f3ea7` に broad の audit 2 本（Codex = 107 + 111、Fable = broad + closure）として記録し、evidence に実際の SHA と run を書いた。教訓: broad の record は是正の push より前に行う。
