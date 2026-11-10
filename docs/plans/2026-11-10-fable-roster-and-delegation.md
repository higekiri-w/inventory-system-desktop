# Plan Packet: 座組で Fable 5.2 を難所のピンチヒッターにし、Ready の承認の範囲と docs の直しの手続きを AGENTS.md に置く

wave に属さない単独の lane。座組表（`docs/AGENT_OPERATING_MANUAL.md` の `## 座組` と §3.3）、Claude 側の subagent の定義（`.claude/agents/`）とその検査（`scripts/tests/claude-hooks.test.sh`）、`AGENTS.md` の Decision and Approval Boundaries、`docs/DEV_WORKFLOW.md` の Risk ごとの最小の流れの 1 セルと承認依頼の 1 項目、`docs/decision-log.md`、旧環境の path（`docs/DEV_SETUP_CHECKLIST.md`・`docs/msi-distribution-runbook.md`）を変える。製品の runtime（`src/**`・`src-tauri/**`）・helper（`scripts/pr-gate.py`）・CI の job・classifier は変えない。

出所: owner の決定（どれも repo 外の回答台帳、2026-11-10）。TD-620（座組は B。Fable 5.2 は難所のピンチヒッター役。Final の Claude 側と相談役を Fable 5.2 に、難所の lane の Writer を Fable 5.2 に、その lane の Final の Claude 側は fresh Opus 5.5 + Codex。TD-410 の「当面 Fable 5」を改める）、TD-627（Fable の Writer を出す条件と範囲）、TD-628（Ready の承認の範囲）と TD-633（同期だけで head が変わったときは承認を保つ）、TD-630（docs の直しの手続き）と TD-634（規則の文書の R0 は常設の委任から除く）、TD-616（承認を求めるときに merge で入るものを添える）、TD-603・TD-629・TD-631（Windows の user のフォルダは `C:\Users\Alice`、L3 用の clone は今の名前から新しく作る、場所は `C:\Users\Alice\projects\inventory-system-desktop`）。TD-633・TD-634 は計画の review round 1 の Opus の P2 から owner に諮って決めた。TD-632（Fable が Writer の lane で Codex が使えない間の Final）は owner の問いに Coordinator が答えた案で、PR の承認のときに owner が確かめる。

decision-log の番号: 本 lane は D-152 を予約する（main は D-151 まで。open な 8 本の branch が持つ D は D-143・D-148・D-149。`git grep -h -o -E '## D-1[4-9][0-9]' origin/agent/<8 本> -- docs/decision-log.md docs/plans` で確かめた、2026-11-10）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
classifier が `AGENTS.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/DEV_WORKFLOW.md` で `workflow=true`、`.claude/agents/*.md`・`scripts/tests/**` で `workflow=true` と全 gate を出し（`echo <path> | bash scripts/ci/classify-changes.sh --files-from-stdin`、2026-11-10）、R2 にできない（`docs/DEV_WORKFLOW.md` Risk Tiers）。`AGENTS.md` の Decision and Approval Boundaries は安全の境界（D-098 の「Decision and Approval Boundaries の承認境界」）で、改訂は通常の packet・PR の経路で行う（AGENTS.md Working Rules の「継ぎ足さない」）。workflow gate change なので Double Audit（独立 2 本、MANUAL §3）に当たり、helper が導く Final の 2 本で満たす。R4 にしない理由: 破壊的なデータの操作・backup の復元・実データ・secret・不可逆な git の操作を伴わない。

## Goal

Goal Invariant:

### 最小完了条件

- merge の後の本体の checkout で、Coordinator が `subagent_type: reviewer-fable`・`consult-fable`・`writer-fable` を起動すると Fable 5.2（`claude-fable-5-2`）・effort high で動き、座組表がどの段でどれを使うか（Fable の Writer を出す条件と記録、Fable が書いた lane の Final と closure の Claude 側）を一意に示す。
- `AGENTS.md` だけを読んだ Coordinator が、Ready の承認で merge まで進めてよい範囲と切れる条件（同期だけで head が変わったときを含む）、docs の follow-up の PR の扱い、lane の外の R0 の常設の委任とその外を、owner に聞き直さずに判断できる。
- Windows の L3 用の clone を新しく作る owner が、`docs/DEV_SETUP_CHECKLIST.md` §4.6 の command をそのまま流して `C:\Users\Alice\projects\inventory-system-desktop` に clone を作り、feature branch を取り込める。

### 失敗定義

- 旧い agent の名（`reviewer-fable5`・`consult-fable5`）か旧い model の ID（`claude-fable-5`）を指す生きた指示が tracked に残り、起動が失敗するか Fable 5 で動く。agent の file 名と frontmatter の `name:` が食い違う。
- Fable が Writer をした lane の Final か closure の Claude 側に Fable が入る規則の読みが残る、または session が替わった後にその lane かどうかを正本（PR）から読めない（TD-627 の「Final は fresh Opus 5.5 + Codex」に反する）。
- 常設の委任が R0 の外（R1 以上、目視の要る R0、lane の中の変更、classifier が `workflow=true` を出す規則の文書）へ読めるか、main への直接の push を許すと読める。
- Ready の承認が、helper の同期だけの条件と code に触れない条件の外で head が変わった後の merge にも効くと読める、または R4 の `r4` の承認を兼ねると読める。逆に、同期だけで head が変わった束の後続の PR の承認が切れると読める（TD-633 に反する）。
- Writer の D-147 の写しが `writer.md` と `writer-fable.md` で食い違い、検査が通る。

### 非目的

- user の `~/.claude/agents/fable-escalation.md`（`model: fable` のまま、TD-622）。
- `.codex/**` の旧い path と Windows の Codex Desktop の項目（別の R3 の lane、TD-607〜615）。
- merge 済みの packet（`docs/plans/` の既存の file）・`docs/archive/**`・`docs/research/**`・`docs/backlog.md` と `docs/decision-log.md` の過去の記録の中の「Fable 5」「Fable 5.1」（その時点の事実で、遡って直さない）。座組表の確認日の行の 2026-10-24（TD-410）の項目も履歴として残す。
- `docs/TOOLING_SKILL_COMMANDS.md` の冒頭の旧 path（調べた時点の対象を書いた記録）。`docs/DEV_SETUP_CHECKLIST.md` §3.1 の済みの項目（private repo の作成の記録）の文。
- 手元の `.local/codex-orders/`・`.local/checklists/` の agent の名（merge の後に手元で直す。下の AC10）。
- 店の PC の古い clone の作り直し（次の訪店の項目、TD-629）。
- helper・classifier・CI の job の変更、Final Review Minimum の導き方の変更。helper に owner の承認や Writer の model を機械で照合させること。
- この環境に ruby を入れること（`claude-hooks.test.sh` が要る。hosted の CI で走る）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- S1 `.claude/agents/`（agent の名から model の版を外し、次の model の更改で名前と参照の sweep を要らなくする）:
  - `reviewer-fable5.md` を `reviewer-fable.md` に `git mv` し、frontmatter を `name: reviewer-fable`・`model: claude-fable-5-2`・`effort: high`、description の「on Fable 5」を「on Fable」にする。`disallowedTools` と本文は変えない。
  - `consult-fable5.md` を `consult-fable.md` に `git mv` し、同じく `name: consult-fable`・`model: claude-fable-5-2`・`effort: high`、description の「on Fable 5」を「on Fable」にする。`disallowedTools` と本文は変えない。
  - `writer-fable.md` を足す。frontmatter は `name: writer-fable`、description「Writer on Fable for the hard lanes the 座組表 names (pinch hitter). Implements a Plan Packet's Scope in the worktree the Coordinator names. The order (発注書) is the only instruction.」、`model: claude-fable-5-2`、`effort: high`。本文は `writer.md` の本文と同じ（code の所有を委ねる文〈Prompting Claude Fable 5.2「Say who owns the code」の例文〉を入れない。範囲は packet の Scope のまま、TD-627・D-147）。
  - `writer.md` と `writer-fable.md` の `## Minimal implementation (D-147)` の 1 文目を「This section is the intended copy of the D-147 item in `AGENTS.md` Working Rules; change it in `AGENTS.md`, `writer.md` and `writer-fable.md` in the same PR.」にする（写しが 2 つになるため）。`writer.md` の frontmatter は変えない。
  - 書けないとき: sandbox の Bash から `.claude/agents/` は読取り専用（2026-11-10、`touch` が `Read-only file system`）。Edit・Write の tool でも書けなければ、Writer は編集の前に止まり、変更の全文を報告に書く。Coordinator が owner の `!` の script（`git mv` と file の中身）に組み直し、中身を owner に示してから流してもらう。
- S2 `docs/AGENT_OPERATING_MANUAL.md`:
  - `## 座組` の Writer 行: 担当の列に、難所の lane は Fable 5.2 の subagent（ピンチヒッター）を出すこと、出す条件（どれか 1 つ: (1) 起票の時点で R4・データの安全〈migration・restore・backup・削除〉・戻しにくい設計 (2) 同じことを 2 回ミスした〈是正の後に同じ指摘が再び出る・同じ test や gate が再び落ちる・同じ AC を 2 回満たせない。計画の段にも当てる〉 (3) 原因の見当がつかないバグ (4) Coordinator が要ると見たとき）、交代した段の残りは Fable に任せきり次の lane は Opus から、範囲は packet の Scope のまま（発注に code の所有を委ねる文を入れない、D-147）、起動の `subagent_type: writer-fable`（`.claude/agents/writer-fable.md`）を足す。記録: `writer-fable` を出したら、Coordinator が段（計画の是正・実装・是正）と当たった条件（(4) は理由）を PR の comment に 1 行残す。lane のどの段でも `writer-fable` が commit を書いたら、その lane は「Fable 5.2 が Writer をした lane」で、Final と closure の発注はこの記録から Claude 側を選ぶ。Codex 停止中の列は同じ文を指す。effort の列に Fable 5.2 = high（TD-620）を足す。
  - Final Reviewer 行: 「Fable 5」を「Fable 5.2」に（3 か所）、起動を `subagent_type: reviewer-fable`（`.claude/agents/reviewer-fable.md`）に。Fable 5.2 が Writer をした lane は closure を含め Claude 側を fresh Opus 5.5 にする句を足す（TD-627）。Codex 停止中の列の R3 の 2 本を「Fable 5.2 + fresh Opus 5.5」にし、Fable 5.2 が Writer をした lane はこの代わりを使わず Codex の枠を pending にする句を足す（TD-632 の案）。
  - 相談役 行: 「Fable 5（当面。owner 2026-10-24、TD-410）」を「Fable 5.2（owner 2026-11-10、TD-620）」に、起動を `subagent_type: consult-fable`（`.claude/agents/consult-fable.md`、`model: claude-fable-5-2`）に。alias の文を公式の今の記述（`ANTHROPIC_DEFAULT_FABLE_MODEL` が無ければ、`fable` は Claude apps gateway の session で Fable 5、それ以外で Fable 5.2 に解決する。公式 model-config「Model aliases」、2026-11-10 に確かめた）に置き換え、Fable 5.2 は Claude Code v2.1.321 以上で、Fable 5.2 を出すよう設定していない gateway は `claude-fable-5-2` を拒むことを 1 句足す（同「Work with Fable」、計画の review round 1 の Opus の P3）。advisor の句の model を Fable 5.2（`claude-fable-5-2`）に。
  - 表の下の確認日の行: 確認日を 2026-11-10 にし、決定の列に 2026-11-10（Fable 5.2 をピンチヒッターに。TD-620・627）を足す。2026-10-24（TD-410）の項目は履歴として残す。
  - 根拠の行: Fable 5 prompting guide の link を [Prompting Claude Fable 5.2](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5-2#calibrate-effort)「Calibrate effort」（`high` から始め、`xhigh` は伸びを測れた最難所。2026-11-10 に確かめた）に置き換える。
  - §3.3 の「Codex が使えない間の Final Review」の項: 「Fable 5 と fresh Opus 5.5」を「Fable 5.2 と fresh Opus 5.5」にし、Fable 5.2 が Writer をした lane を除く句を足す。
  - 変えない所: §3 の独立性の項目、Plan Reviewer 行、Coordinator 行、Human Gate 行、Fable の消費を 1〜2 週ごとに `/usage` で見る項（TD-627 の「消費がひどければ見直す」はこの項の運用に含まれる）。
- S3 `AGENTS.md` の Decision and Approval Boundaries の「owner へ送り、中継を省かない問い合わせ」の項目: 「owner は名指しの複数の PR（R0〜R4）の Ready と merge を 1 回の承認で…D-145 (14)）。」の前後を、次の意味の文に置き換える（文面は Writer、意味はこの箇条書きのとおり）:
  - Ready の承認は、承認した head のままの merge までを含み、merge の後は Coordinator が手元の記録（handoff）まで続ける。head が変われば承認は切れ、取り直す。ただし helper が同期だけの条件と code に触れない条件を満たすと判定した base の同期（`docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」の A と同じ条件）で head が変わったときは承認を保つ（TD-633。束ねた承認で先の PR の merge が main を進め、strict の保護で後続の PR を同期する場合）。code に触れる同期と lane の差分の変化では切れる。R4 の `r4` の承認は別に要る（TD-628）。
  - owner は名指しの複数の PR（R0〜R4）の Ready と merge を 1 回の承認で任せられる（今の文。各 PR の helper の `merge` は省かない）。
  - lane の中で分かっている docs の直しは、Ready の前に同じ PR に入れる。Ready の承認を求めるときに内容を名指した follow-up の docs の PR は、その承認に含める。承認はその名指した内容に結び、内容が名指しから外れたら切れる。Risk ごとの review は省かない（R2 は差分の review 1 本）（TD-630 (1)(2)）。
  - lane の外で見つけた R0 のうち、目視の要らないもの（PR body の `Risk: R0（目視: 不要）`。目視の要る R0 は owner の目視を待つので委任しても終わらない）で、classifier が `workflow=true` を出す path（規則の文書）に触れないものは常設の委任で、Coordinator がすぐ PR を作り、hosted の `Merge gate` が通れば helper の `ready`・`merge` まで進め、次の owner への返答と handoff で報告する。溜めてまとめない。main への直接の push はせず、main の保護を外さない（TD-630 (3)・TD-634）。規則の文書に触れる R0 は今までどおり Ready の承認を求める。
  - 今の文の「現行の明示承認と有効な委任（…owner の承認後の helper `ready` / `merge` の実行）」の括弧に、lane の外の R0 の常設の委任を足す。「委任の範囲の正本はこの項目」は変えない。出所の ID に D-152 を足す。
- S4 `docs/DEV_WORKFLOW.md`:
  - 「Risk ごとの最小の流れ」の表の R0・R1 の行の owner の列: 「Ready・merge の承認」に「（lane の外で見つけた目視の要らない R0 のうち規則の文書に触れないものは AGENTS.md の常設の委任）」を足す。
  - 「承認依頼」の 1 つ目の項目: 「承認すると利用者から見て何が完了するか」の文に、Ready・merge の承認では merge すると入るもの（アプリの動き・画面・データ・build・手元の道具に何が変わるか）と入らないものを添えることを足す（TD-616。AGENTS.md には書かず、承認の求め方はこの節に置く。計画の review round 1 の Opus の観察）。
  - 他の行・節は変えない（Draft・Ready・merge の節の「owner の承認（名指しの PR の束でよい）の後、Coordinator が helper `ready` → helper `merge`」と承認依頼の節の「委任の範囲は AGENTS.md」は今のままで S3 と合う）。
- S5 `docs/decision-log.md`: 末尾に D-152（Status・Context・Decision・Why・Alternatives・Compatibility・Revisit）。Compatibility: D-145 (14) を広げる（同期だけの head の変化で承認を保つ）、TD-410 の「当面 Fable 5」を改める、D-147 の「両方を直す」が写し 2 つを含む 3 か所になる、D-145 (13) の「Fable 5」は当時の記録。Revisit: `/usage` の Fable の消費（1〜2 週ごと、TD-627）、次の Fable の model、Fable が書いた lane の Claude 側の Final の effort（`reviewer` の定義の medium のままでよいか。owner は model だけを決めた。計画の review round 1 の Opus の P3）。
- S6 旧環境の path（TD-603・TD-629・TD-631）:
  - `docs/DEV_SETUP_CHECKLIST.md` §4.6: clone を `C:\Users\Alice\projects\inventory-system-desktop` にし、取り込みの手順の前に初回の clone の command（`git clone https://github.com/higekiri-w/inventory-system-desktop.git C:\Users\Alice\projects\inventory-system-desktop`）を足す。旧 private repo の origin を張り替える段落は消す（新しく今の名前から clone するので当たらない。D-127）。
  - 同 §4.5 の作業ディレクトリの行（`/home/{user}/inventory-system-public`）を `/home/alice/projects/inventory-system-desktop` にし、同じ行の「CLAUDE.md 『やってはいけないこと』参照」（今の CLAUDE.md にその節は無い）を CLAUDE.md の「再開とmemory」（repo 固有の成果物は作業 checkout に保持する）への参照にする。
  - 同 §3.1 の clone の command block（`git clone git@github.com:{username}/inventory-system.git`・`cd inventory-system`）を、今の repo を `/home/alice/projects/inventory-system-desktop` に clone する command にする（済みの項目の文は非目的のとおり残す）。
  - 同 §3.3 の sanitize 後の path（`-home-kosei-projects-inventory-system-public`）を `-home-alice-projects-inventory-system-desktop` にする（例の文と検証の command も合わせる）。
  - `docs/msi-distribution-runbook.md` 流れ 1 の clone の path を `C:\Users\Alice\projects\inventory-system-desktop` にする。
- S7 `scripts/tests/claude-hooks.test.sh`: `validate_contract` が通る root で、`.claude/agents/writer.md` と `.claude/agents/writer-fable.md` が両方あれば、frontmatter を除いた本文が同じことを確かめる検査を足し、本文の 1 行だけが違う mutant を `expect_rejected` で拒むことを確かめる（失敗定義 5 つ目を merge の後も守る。計画の review round 1 の Opus の P3）。fixture（`make_fixture`）に `writer-fable.md` が無ければ検査は当たらない形でよく、mutant は fixture に 2 本を置いて作る。D-147 の comment（`the ponytail plugin stays off … (AGENTS.md and writer.md carry the rule instead)`）を `writer.md` と `writer-fable.md` に合わせる。既存の検査と mutant は消さず弱めない。
- S8 本 packet。

呼出し側・隣接の確認（2026-11-10）: `git grep -n -E 'reviewer-fable5|consult-fable5|claude-fable-5|Fable 5|fable-escalation' -- ':!docs/archive' ':!docs/research'` の生きた指示は `.claude/agents/` の 2 file と `docs/AGENT_OPERATING_MANUAL.md` の 54・72・73・76・80 行だけ（他は merge 済みの packet・backlog・decision-log の過去の記録と、`scripts/tests/doc-consistency-plan-packet.test.sh` の fixture の文字列「Fable 5（fixture coordinator）」で、agent を指さない。計画の review の 2 本も hidden の dir を含めて同じ結論）。`.claude/agents/` を読む test は `scripts/tests/claude-hooks.test.sh`（frontmatter の `hooks`・`permissionMode`・`mcpServers` の禁止）と `scripts/tests/classify-changes.test.sh`（`.claude/agents/reviewer.md` の分類）で、名前は見ない。`C:\Users\Owner\projects\inventory-system-public` は `docs/DEV_SETUP_CHECKLIST.md` の §4.6 の 2 か所と `docs/msi-distribution-runbook.md` の流れ 1 だけ（`src/lib/extractFilename.test.ts` の `C:\Users\Owner\data.csv` は test の入力で path の案内でない）。

Non-scope: 上の非目的のとおり。`docs/agent-guidance/merge-evidence.md` の承認の文（成功の定義の「ownerのReady/merge判断…は維持する」、状態の表の「ownerのReady判断待ち」「ownerのmerge指示があればhelperでmerge」、R3/R4 の例の「record/Ready/mergeはその操作のowner指示を得てから実行する」）は変えない。Ready の承認が merge を含むこと（TD-628）は「merge の指示」を Ready の承認と同時に出したものとして、常設の委任は owner が前もって出した指示として読め、委任の範囲の正本は AGENTS.md に置く（merge-evidence に写さない）。

## Acceptance Criteria

- AC1: `.claude/agents/` に `reviewer-fable.md`・`consult-fable.md`・`writer-fable.md` があり、3 つとも frontmatter に `model: claude-fable-5-2` と `effort: high` があり、`name:` が file 名の stem と同じ。`reviewer-fable5.md`・`consult-fable5.md` が無い。`git diff --name-status -M40% origin/main -- .claude/agents` が 2 つを `R` で示す（既定の 50% では reviewer が 48% で add/delete になる。計画の review round 1 の Codex の実測）。
- AC2: `diff <(sed '1,/^---$/d' .claude/agents/writer.md) <(sed '1,/^---$/d' .claude/agents/writer-fable.md)` が空（本文が同じ）。`writer.md` の frontmatter（最初の `---` の間）が origin/main と同じ。`git grep -n -i -E 'codebase is yours|is yours' -- .claude/agents` が 0 件。
- AC3: `git grep -n -E 'reviewer-fable5|consult-fable5|claude-fable-5([^-]|$)' -- ':!docs/archive' ':!docs/research' ':!docs/plans' ':!docs/backlog.md' ':!docs/decision-log.md'` が 0 件（tracked だけを、`.claude/`・`.agents/`・`.codex/` を含めて探す）。`docs/decision-log.md` の当たりは過去の記録と D-152 の中の旧名の記録だけ。
- AC4: `git grep -n -E 'Fable 5([^.]|$)' -- docs/AGENT_OPERATING_MANUAL.md` の当たりが、alias の文（`Claude apps gateway` を含む行）と確認日の行（2026-10-24 の TD-410 の履歴）だけ。Writer 行が出す条件 (1)〜(4)・任せきり・次の lane は Opus から・範囲は Scope のまま・`writer-fable`・PR の comment の記録と「どの段でも commit を書いたら」の判定を持ち、Final Reviewer 行と §3.3 が Fable 5.2 が Writer をした lane の除外（Final と closure の Claude 側は fresh Opus 5.5、Codex 停止中は代えずに pending）を持つ。
- AC5: `AGENTS.md` の Decision and Approval Boundaries が S3 の点（Ready の承認は同じ head の merge まで・同期だけと code に触れない条件の同期では保つ・それ以外の head の変化で切れる・`r4` は別 / follow-up の docs の PR は名指した内容に結び Risk の review は省かない / lane の外の目視の要らない R0 で規則の文書に触れないものの常設の委任・main への直接の push はしない / 委任の範囲の正本はこの項目）を持つ。`git diff -U0 origin/main -- AGENTS.md | grep '^@@'` の行の範囲が、`grep -n '^## ' AGENTS.md` で引いた Decision and Approval Boundaries の節の行の範囲の中だけ。
- AC6: `git diff -U0 origin/main -- docs/DEV_WORKFLOW.md | grep '^@@'` の hunk が、Risk ごとの最小の流れの表の R0・R1 の行と、承認依頼の節の 1 つ目の項目の 2 か所だけ。
- AC7: `docs/decision-log.md` の末尾に D-152 があり、Compatibility と Revisit に S5 の項目を書く。
- AC8: `git grep -n -E 'inventory-system-public|C:\\Users\\Owner\\projects|-home-kosei-|\{username\}/inventory-system\.git|やってはいけないこと' -- docs/DEV_SETUP_CHECKLIST.md docs/msi-distribution-runbook.md` が 0 件。§4.6 に初回の clone の command がある。`git grep -n -F 'C:\Users\Alice\projects\inventory-system-desktop' -- docs/DEV_SETUP_CHECKLIST.md docs/msi-distribution-runbook.md` が両方に当たる。
- AC9: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR なし。`bash scripts/tests/classify-changes.test.sh` が PASS。`bash scripts/tests/claude-hooks.test.sh` が PASS（この環境は ruby が無く `FAIL: ruby (yaml) is required` で止まるので、hosted の CI の結果で見る。ruby のある環境が使えれば手元でも流す）。hosted の `Merge gate` が成功。既存の test を消さず弱めない。
- AC10: S7 の検査が、`writer-fable.md` の本文の 1 行を変えた mutant を拒む（test の中の `expect_rejected`）。実装の報告に、test の外で同じ mutation を入れた tree で `claude-hooks.test.sh` が red になることを書く（ruby が無ければ hosted か、検査の関数を ruby に依らない形にしたなら手元で）。
- AC11（merge の後、main の同期の直後で、次の発注の前）: 本体の checkout を main に進め、`claude --version` が 2.1.321 以上であることと、`subagent_type: consult-fable` を 1 回起動して実効の model（`claude-fable-5-2`）と effort（high）を `/tasks` か run の metadata で確かめ、結果を handoff に書く。取れなければ未実測と書き、Goal の 1 つ目を検証済みとしない。手元の `.local/codex-orders/MODEL-SELECTION.md`・`.local/codex-orders/templates/`・`.local/checklists/README.md` の旧い agent の名を直す（tracked でない。直すまで旧い名で発注しない）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/AGENT_OPERATING_MANUAL.md` §3（独立性）・§3.3・`## 座組`。`AGENTS.md` の Decision and Approval Boundaries と Working Rules の「継ぎ足さない」「最小の実装（D-147）」。`docs/DEV_WORKFLOW.md` の Risk Tiers・Draft・Ready・merge・承認依頼。`docs/agent-guidance/merge-evidence.md` の「base同期だけでheadが変わる場合」（MG-D13 の A）と承認の文。`docs/decision-log.md` D-098・D-145 (13)(14)・D-147。
- 必要な設計成果物: 座組表・AGENTS.md・DEV_WORKFLOW の 2 か所・D-152 を本 PR で書く（updated in this PR）。merge-evidence・helper は existing sufficient（Non-scope の理由）。
- plan にしかない durable な判断の昇格先: TD-620・627・628・630・632・633・634 を D-152 と座組表と AGENTS.md へ。agent の名から版を外す判断と、Fable が Writer をした lane の記録の置き場（PR の comment）を D-152 と座組表へ。
- 前提・制約と延期した design gap:
  - P1: Claude Code が `model: claude-fable-5-2` の frontmatter を受け、Fable 5.2 で subagent を動かす（公式 sub-agents「Choose a model」は full model ID を受ける。Agent tool の呼出しの `model` は alias だけで、`fable` は gateway の session で Fable 5 になるので、定義の frontmatter で full ID を固定する。Fable 5.2 は v2.1.321 以上、この環境は 2.1.322〈計画の review round 1 の Opus の実測〉）。今の session の project の dir は本体の checkout なので、worktree の定義は効かず、AC11 で merge の後に確かめる。
  - P2: `.claude/agents/` を agent の tool で書けるか（S1 の「書けないとき」）。
  - TD-632 は owner の確かめの前の案。owner が別の形を選べば、計画の固定の前なら packet を直し、固定の後なら Gated Amendment にする。
  - 同期の後に承認を保つ判定（TD-633）と Fable の lane の判定は、helper が機械で照合しない（helper は owner の承認も Writer の model も見ない）。Coordinator が helper の判定（同期だけ・code に触れない）と PR の comment の記録を見て行う。
- 絶対保証の例外と抜け道: 常設の委任は「lane の外で見つけた」「R0」「目視: 不要」「規則の文書（`workflow=true`）に触れない」の 4 つを全部満たす PR だけ。R0 の判定は人の判断で、helper は PR body の Risk の行と `--risk`・`--manual` の照合だけを確かめる（Risk の取り違えは機械では止まらない）。取り違えの抜け道は、R0 の意味（意味を変えない docs の直し）が `docs/DEV_WORKFLOW.md` Risk Tiers に固定されていること、helper の packet なしの道が classifier の `unknown=true`・workflow の実行 code（`workflow=true` かつ `rust=true`）・migration・依存に触れる R0/R1 を止めること（D-145 (8)）、`workflow=true` の判定は classifier の出力で機械に確かめられることで狭める。owner への後の報告は委任の記録として残す。
- 判定: ready。変更は座組表の 4 行と §3.3 の 1 項、AGENTS.md の 1 項目、DEV_WORKFLOW の 2 か所、D-152、agent の定義 3 つ、claude-hooks の検査 1 つ、path の 5 か所に留まる。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| 座組（担当・model・effort） | MANUAL `## 座組`、SPEC-WF-HARNESS2-D1 | S1・S2 | `claude-hooks.test.sh`（frontmatter の禁止 field）、AC1〜AC4 の git grep・diff | AC11（merge の後の実効の確かめ） |
| Fable の Writer の記録と判定 | MANUAL `## 座組` の Writer 行・Final Reviewer 行 | S2 | 非対象（文書。AC4） | 非対象 |
| §3 の独立性（Writer ≠ Final Reviewer、fresh context） | MANUAL §3 | 変えない。Fable が書いた lane の Final を fresh Opus 5.5 にする句は独立性を強める側 | 非対象（文書） | 非対象 |
| Codex 停止中の Final（TD-499） | MANUAL §3.3、D-145 (13) | S2（Fable 5.2 に、Fable が書いた lane を除く） | AC4 の git grep | TD-632 は owner の確かめ |
| Writer の D-147 の写し | AGENTS.md Working Rules「最小の実装（D-147）」、D-147 | S1（写しを 2 つにし、1 文目で同じ PR で直すことを示す）・S7 | S7 の検査と mutant、AC2 の diff | 非対象 |
| 委任の範囲（Ready・merge・同期・docs・R0） | AGENTS.md Decision and Approval Boundaries、D-145 (14)、D-098、merge-evidence の MG-D13 の A | S3・S4・S5 | 非対象（文書。AC5・AC6 の hunk の範囲） | 非対象 |
| 承認の求め方（merge で入るもの） | DEV_WORKFLOW 承認依頼、TD-616 | S4 | 非対象（文書。AC6） | 非対象 |
| merge は helper 経由・直接 UI merge の禁止 | AGENTS.md Working Rules、merge-evidence | 変えない（常設の委任も helper の `ready`・`merge` を通る） | 非対象 | 非対象 |
| 旧環境の path（Windows の clone・WSL の checkout・memory） | DEV_SETUP_CHECKLIST §3.1・§3.3・§4.5・§4.6、msi-distribution-runbook 流れ 1 | S6 | AC8 の git grep | 次の Windows の L3 で owner が clone を作る |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| agent の定義の frontmatter の安全 | 新しい定義に `hooks`・`permissionMode`・`mcpServers` が入り、project の hook の無効が崩れる | 既存の shell test（hosted） | `scripts/tests/claude-hooks.test.sh` | `writer-fable.md` の frontmatter に禁止の field を足す |
| Writer の本文の写し | Fable の Writer だけ D-147 の梯子が違う、後の PR が片方だけを直す | shell test の検査と mutant（S7）・CLI（diff） | `scripts/tests/claude-hooks.test.sh` の S7 の検査、AC2 | `writer-fable.md` の本文の 1 行を変える、ownership の文を足す |
| `.claude/agents/` の分類 | 新しい定義が full の gate から外れる | 既存の shell test | `scripts/tests/classify-changes.test.sh` | classifier の `.claude/agents/*` の分類を外す（今の mutant の case） |
| 旧い名と ID の sweep、名前と file 名 | 旧い agent の名・`claude-fable-5` の生きた指示が残る、`name:` が file 名と違う | CLI（git grep） | AC1・AC3・AC4 | MANUAL か `.claude/agents/` に旧い名か ID を 1 つ残す、rename の後の file に `name: reviewer-fable5` を残す |
| 旧環境の path | 旧い path・旧い repo の名・死んだ参照が残る | CLI（git grep） | AC8 | §4.6・§3.1・§4.5・§3.3・runbook のどれかに旧い形を残す |
| docs の検査 | link 切れ・packet の形の誤り | 既存の検査 | `bash scripts/doc-consistency-check.sh`・`--target plan` | packet の節を欠く、MANUAL の link を壊す |
| 実効の model と effort | 定義が効かず session の model・effort で動く | 手での確かめ（merge の後） | AC11 | `model:` の ID を誤る、定義が読まれない、Claude Code の版が下限より古い |
</content>
</invoke>
