# Plan Packet: 座組で Fable 5.2 を難所のピンチヒッターにし、Ready の承認の範囲と docs の直しの手続きを AGENTS.md に置く

wave に属さない単独の lane。座組表（`docs/AGENT_OPERATING_MANUAL.md` の `## 座組` と §3.3）、Claude 側の subagent の定義（`.claude/agents/`）、`AGENTS.md` の Decision and Approval Boundaries、`docs/DEV_WORKFLOW.md` の Risk ごとの最小の流れの 1 セル、`docs/decision-log.md`、Windows の clone の path（`docs/DEV_SETUP_CHECKLIST.md`・`docs/msi-distribution-runbook.md`）を変える。製品の runtime（`src/**`・`src-tauri/**`）・helper（`scripts/**`）・CI・classifier は変えない。

出所: owner の決定（どれも repo 外の回答台帳、2026-11-10）。TD-620（座組は B。Fable 5.2 は難所のピンチヒッター役。Final の Claude 側と相談役を Fable 5.2 に、難所の lane の Writer を Fable 5.2 に、その lane の Final の Claude 側は fresh Opus 5.5 + Codex。TD-410 の「当面 Fable 5」を改める）、TD-627（Fable の Writer を出す条件と範囲）、TD-628（Ready の承認の範囲）、TD-630（docs の直しの手続き）、TD-603・TD-629・TD-631（Windows の user のフォルダは `C:\Users\Alice`、L3 用の clone は今の名前から新しく作る、場所は `C:\Users\Alice\projects\inventory-system-desktop`）。TD-632（Fable が Writer の lane で Codex が使えない間の Final）は owner の問いに Coordinator が答えた案で、PR の承認のときに owner が確かめる。

decision-log の番号: 本 lane は D-152 を予約する（main は D-151 まで。open な 8 本の branch が持つ D は D-143・D-148・D-149。`git grep -h -o -E '## D-1[4-9][0-9]' origin/agent/<8 本> -- docs/decision-log.md docs/plans` で確かめた、2026-11-10）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
classifier が `AGENTS.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/DEV_WORKFLOW.md` で `workflow=true`、`.claude/agents/*.md` で `workflow=true` と全 gate を出し（`echo <path> | bash scripts/ci/classify-changes.sh --files-from-stdin`、2026-11-10）、R2 にできない（`docs/DEV_WORKFLOW.md` Risk Tiers）。`AGENTS.md` の Decision and Approval Boundaries は安全の境界（D-098 の「Decision and Approval Boundaries の承認境界」）で、改訂は通常の packet・PR の経路で行う（AGENTS.md Working Rules の「継ぎ足さない」）。R4 にしない理由: 破壊的なデータの操作・backup の復元・実データ・secret・不可逆な git の操作を伴わない。

## Goal

Goal Invariant:

### 最小完了条件

- merge の後の本体の checkout で、Coordinator が `subagent_type: reviewer-fable`・`consult-fable`・`writer-fable` を起動すると Fable 5.2（`claude-fable-5-2`）・effort high で動き、座組表がどの段でどれを使うか（Fable の Writer を出す条件、Fable が書いた lane の Final の Claude 側）を一意に示す。
- `AGENTS.md` だけを読んだ Coordinator が、Ready の承認で merge まで進めてよい範囲と切れる条件、docs の follow-up の PR の扱い、lane の外の R0 の常設の委任を、owner に聞き直さずに判断できる。
- Windows の L3 用の clone を新しく作る owner が、`docs/DEV_SETUP_CHECKLIST.md` §4.6 の command をそのまま流して `C:\Users\Alice\projects\inventory-system-desktop` に clone を作り、feature branch を取り込める。

### 失敗定義

- 旧い agent の名（`reviewer-fable5`・`consult-fable5`）か旧い model の ID（`claude-fable-5`）を指す生きた指示が tracked に残り、起動が失敗するか Fable 5 で動く。
- Fable が Writer をした lane の Final か closure の Claude 側に Fable が入る規則の読みが残る（TD-627 の「Final は fresh Opus 5.5 + Codex」に反する）。
- 常設の委任が R0 の外（R1 以上、目視の要る R0、lane の中の変更）へ読めるか、main への直接の push を許すと読める。
- Ready の承認が head の変わった後の merge にも効くと読める、または R4 の `r4` の承認を兼ねると読める。
- Writer の D-147 の写しが `writer.md` と `writer-fable.md` で食い違う。

### 非目的

- user の `~/.claude/agents/fable-escalation.md`（`model: fable` のまま、TD-622）。
- `.codex/**` の旧い path と Windows の Codex Desktop の項目（別の R3 の lane、TD-607〜615）。
- merge 済みの packet（`docs/plans/` の既存の file）・`docs/archive/**`・`docs/research/**`・`docs/backlog.md` と `docs/decision-log.md` の過去の記録の中の「Fable 5」「Fable 5.1」（その時点の事実で、遡って直さない）。
- `docs/TOOLING_SKILL_COMMANDS.md` の冒頭の旧 path（調べた時点の対象を書いた記録）。
- 手元の `.local/codex-orders/`・`.local/checklists/` の agent の名（merge の後に手元で直す。下の AC10）。
- 店の PC の古い clone の作り直し（次の訪店の項目、TD-629）。
- helper・classifier・CI の変更、Final Review Minimum の導き方の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- S1 `.claude/agents/`（agent の名から model の版を外し、次の model の更改で名前と参照の sweep を要らなくする）:
  - `reviewer-fable5.md` を `reviewer-fable.md` に `git mv` し、frontmatter を `name: reviewer-fable`・`model: claude-fable-5-2`・`effort: high`、description の「on Fable 5」を「on Fable」にする。`disallowedTools` と本文は変えない。
  - `consult-fable5.md` を `consult-fable.md` に `git mv` し、同じく `name: consult-fable`・`model: claude-fable-5-2`・`effort: high`、description の「on Fable 5」を「on Fable」にする。`disallowedTools` と本文は変えない。
  - `writer-fable.md` を足す。frontmatter は `name: writer-fable`、description「Writer on Fable for the hard lanes the 座組表 names (pinch hitter). Implements a Plan Packet's Scope in the worktree the Coordinator names. The order (発注書) is the only instruction.」、`model: claude-fable-5-2`、`effort: high`。本文は `writer.md` の本文と同じ（code の所有を委ねる文〈Prompting Claude Fable 5.2「Say who owns the code」の例文〉を入れない。範囲は packet の Scope のまま、TD-627・D-147）。
  - `writer.md` と `writer-fable.md` の `## Minimal implementation (D-147)` の 1 文目を「This section is the intended copy of the D-147 item in `AGENTS.md` Working Rules; change it in `AGENTS.md`, `writer.md` and `writer-fable.md` in the same PR.」にする（写しが 2 つになるため）。`writer.md` の frontmatter は変えない。
  - 書けないとき: sandbox の Bash から `.claude/agents/` は読取り専用（2026-11-10、`touch` が `Read-only file system`）。Edit・Write の tool でも書けなければ、Writer は編集の前に止まり、変更の全文を報告に書く。Coordinator が owner の `!` の script（`git mv` と file の中身）に組み直す。
- S2 `docs/AGENT_OPERATING_MANUAL.md`:
  - `## 座組` の Writer 行: 担当の列に、難所の lane は Fable 5.2 の subagent（ピンチヒッター）を出すこと、出す条件（どれか 1 つ: (1) 起票の時点で R4・データの安全〈migration・restore・backup・削除〉・戻しにくい設計 (2) 同じことを 2 回ミスした〈是正の後に同じ指摘が再び出る・同じ test や gate が再び落ちる・同じ AC を 2 回満たせない。計画の段にも当てる〉 (3) 原因の見当がつかないバグ (4) Coordinator が要ると見たとき、理由を 1 行残す）、交代した段の残りは Fable に任せきり次の lane は Opus から、範囲は packet の Scope のまま（発注に code の所有を委ねる文を入れない、D-147）、起動の `subagent_type: writer-fable`（`.claude/agents/writer-fable.md`）を足す。Codex 停止中の列は同じ文を指す。effort の列に Fable 5.2 = high（TD-620）を足す。
  - Final Reviewer 行: 「Fable 5」を「Fable 5.2」に（3 か所）、起動を `subagent_type: reviewer-fable`（`.claude/agents/reviewer-fable.md`）に。Fable 5.2 が Writer をした lane は closure を含め Claude 側を fresh Opus 5.5 にする句を足す（TD-627）。Codex 停止中の列の R3 の 2 本を「Fable 5.2 + fresh Opus 5.5」にし、Fable 5.2 が Writer をした lane はこの代わりを使わず Codex の枠を pending にする句を足す（TD-632 の案）。
  - 相談役 行: 「Fable 5（当面。owner 2026-10-24、TD-410）」を「Fable 5.2（owner 2026-11-10、TD-620）」に、起動を `subagent_type: consult-fable`（`.claude/agents/consult-fable.md`、`model: claude-fable-5-2`）に。alias の文を公式の今の記述（`ANTHROPIC_DEFAULT_FABLE_MODEL` が無ければ、`fable` は Claude apps gateway の session で Fable 5、それ以外で Fable 5.2 に解決する。公式 model-config「Model aliases」、2026-11-10 に確かめた）に置き換える。advisor の句の model を Fable 5.2（`claude-fable-5-2`）に。
  - 表の下の確認日の行: 確認日を 2026-11-10 にし、決定の列に 2026-11-10（Fable 5.2 をピンチヒッターに。TD-620・627）を足す。
  - 根拠の行: Fable 5 prompting guide の link を [Prompting Claude Fable 5.2](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5-2#calibrate-effort)「Calibrate effort」（`high` から始め、`xhigh` は伸びを測れた最難所。2026-11-10 に確かめた）に置き換える。
  - §3.3 の「Codex が使えない間の Final Review」の項: 「Fable 5 と fresh Opus 5.5」を「Fable 5.2 と fresh Opus 5.5」にし、Fable 5.2 が Writer をした lane を除く句を足す。
  - 変えない所: §3 の独立性の項目、Plan Reviewer 行、Coordinator 行、Human Gate 行、Fable の消費を 1〜2 週ごとに `/usage` で見る項（TD-627 の「消費がひどければ見直す」はこの項の運用に含まれる）。
- S3 `AGENTS.md` の Decision and Approval Boundaries の「owner へ送り、中継を省かない問い合わせ」の項目: 「owner は名指しの複数の PR（R0〜R4）の Ready と merge を 1 回の承認で…D-145 (14)）。」の前後を、次の意味の文に置き換える（文面は Writer、意味はこの箇条書きのとおり）:
  - Ready の承認は、承認した head のままの merge までを含む。承認を求めるときに、merge で入るものと入らないものを添える（TD-616）。head が変われば承認は切れ、取り直す。R4 の `r4` の承認は別に要る（TD-628）。
  - owner は名指しの複数の PR（R0〜R4）の Ready と merge を 1 回の承認で任せられる（今の文。各 PR の helper の `merge` は省かない）。
  - lane の中で分かっている docs の直しは、Ready の前に同じ PR に入れる。Ready の承認を求めるときに名指した follow-up の docs の PR は、その承認に含める（Risk の review は省かない。R2 は差分の review 1 本）（TD-630 (1)(2)）。
  - lane の外で見つけた R0 のうち目視の要らないもの（PR body の `Risk: R0（目視: 不要）`）は常設の委任で、Coordinator がすぐ PR を作り、hosted の `Merge gate` が通れば helper の `ready`・`merge` まで進め、後で owner に報告する。溜めてまとめない。main への直接の push はせず、main の保護を外さない（TD-630 (3)）。
  - 今の文の「現行の明示承認と有効な委任（…owner の承認後の helper `ready` / `merge` の実行）」の括弧に、lane の外の R0 の常設の委任を足す。「委任の範囲の正本はこの項目」は変えない。出所の ID に D-152 を足す。
- S4 `docs/DEV_WORKFLOW.md` の「Risk ごとの最小の流れ」の表の R0・R1 の行の owner の列: 「Ready・merge の承認」に「（lane の外で見つけた目視の要らない R0 は AGENTS.md の常設の委任）」を足す。他の行・節は変えない（Draft・Ready・merge の節の「owner の承認（名指しの PR の束でよい）の後、Coordinator が helper `ready` → helper `merge`」と承認依頼の節の「委任の範囲は AGENTS.md」は今のままで S3 と合う）。
- S5 `docs/decision-log.md`: 末尾に D-152（Status・Context・Decision・Why・Alternatives・Compatibility・Revisit。D-145 (14) を広げ、TD-410 の「当面 Fable 5」を改める。D-147 の写しが 2 つになる）。
- S6 Windows の clone と WSL の path（TD-603・TD-629・TD-631。旧環境の path の残り）:
  - `docs/DEV_SETUP_CHECKLIST.md` §4.6: clone を `C:\Users\Alice\projects\inventory-system-desktop` にし、取り込みの手順の前に初回の clone の command（`git clone https://github.com/higekiri-w/inventory-system-desktop.git C:\Users\Alice\projects\inventory-system-desktop`）を足す。旧 private repo の origin を張り替える段落は消す（新しく今の名前から clone するので当たらない。D-127）。
  - 同 §4.5 の作業ディレクトリの行（`/home/{user}/inventory-system-public`）を `/home/alice/projects/inventory-system-desktop` に、§3.3 の sanitize 後の path（`-home-kosei-projects-inventory-system-public`）を `-home-alice-projects-inventory-system-desktop` にする（例の文も合わせる）。
  - `docs/msi-distribution-runbook.md` 流れ 1 の clone の path を `C:\Users\Alice\projects\inventory-system-desktop` にする。
- S7 本 packet。

呼出し側・隣接の確認（2026-11-10）: `git grep -n -E 'reviewer-fable5|consult-fable5|claude-fable-5|Fable 5|fable-escalation' -- ':!docs/archive' ':!docs/research'` の生きた指示は `.claude/agents/` の 2 file と `docs/AGENT_OPERATING_MANUAL.md` の 54・72・73・76・80 行だけ（他は merge 済みの packet・backlog・decision-log の過去の記録と、`scripts/tests/doc-consistency-plan-packet.test.sh` の fixture の文字列「Fable 5（fixture coordinator）」で、agent を指さない）。`.claude/agents/` を読む test は `scripts/tests/claude-hooks.test.sh`（frontmatter の `hooks`・`permissionMode`・`mcpServers` の禁止）と `scripts/tests/classify-changes.test.sh`（`.claude/agents/reviewer.md` の分類）で、名前は見ない。`C:\Users\Owner\projects\inventory-system-public` は `docs/DEV_SETUP_CHECKLIST.md` の 252・255 行と `docs/msi-distribution-runbook.md` の 55 行だけ（`src/lib/extractFilename.test.ts` の `C:\Users\Owner\data.csv` は test の入力で path の案内でない）。

Non-scope: 上の非目的のとおり。`docs/agent-guidance/merge-evidence.md` の「Ready/mergeはownerの明示指示を前提にする」は変えない（常設の委任は owner が前もって出した明示の指示で、範囲の正本は AGENTS.md）。

## Acceptance Criteria

- AC1: `.claude/agents/` に `reviewer-fable.md`・`consult-fable.md`・`writer-fable.md` があり、3 つとも frontmatter に `model: claude-fable-5-2` と `effort: high` がある。`reviewer-fable5.md`・`consult-fable5.md` が無い。`git diff --find-renames origin/main -- .claude/agents` が 2 つを rename として示す。
- AC2: `diff <(sed '1,/^---$/d' .claude/agents/writer.md) <(sed '1,/^---$/d' .claude/agents/writer-fable.md)` が空（本文が同じ）。`writer.md` の frontmatter（最初の `---` の間）が origin/main と同じ。`rg -n -i "codebase is yours|is yours" .claude/agents` が 0 件。
- AC3: `rg -n -e 'reviewer-fable5|consult-fable5' -e 'claude-fable-5([^-]|$)' -g '!docs/archive/**' -g '!docs/research/**' -g '!docs/plans/**' -g '!docs/backlog.md' -g '!docs/decision-log.md' .` が 0 件（`.claude/agents/` と `AGENTS.md`・`docs/AGENT_OPERATING_MANUAL.md` を含む）。`docs/decision-log.md` の当たりは D-152 の中の旧名の記録だけ。
- AC4: `docs/AGENT_OPERATING_MANUAL.md` の `## 座組` と §3.3 に「Fable 5」（後ろに `.` の付かない形）が残らない（`rg -n 'Fable 5([^.]|$)' docs/AGENT_OPERATING_MANUAL.md` が 0 件）。Writer 行が出す条件 (1)〜(4)・任せきり・次の lane は Opus から・範囲は Scope のまま・`writer-fable` を持ち、Final Reviewer 行と §3.3 が Fable 5.2 が Writer をした lane の除外（Final と closure の Claude 側は fresh Opus 5.5、Codex 停止中は代えずに pending）を持つ。
- AC5: `AGENTS.md` の Decision and Approval Boundaries が S3 の 4 点（Ready の承認は同じ head の merge まで・head が変われば切れる・`r4` は別 / follow-up の docs の PR は承認に含め Risk の review は省かない / lane の外の目視の要らない R0 の常設の委任・main への直接の push はしない / 委任の範囲の正本はこの項目）を持ち、Working Rules・Safety・npm 供給網ガードの節は origin/main と同じ（`git diff origin/main -- AGENTS.md` の hunk が Decision and Approval Boundaries の中だけ）。
- AC6: `docs/DEV_WORKFLOW.md` の差分が Risk ごとの最小の流れの表の R0・R1 の行の 1 セルだけ。
- AC7: `docs/decision-log.md` の末尾に D-152 があり、Compatibility に D-145 (14) を広げること・TD-410 を改めること・D-147 の写しが 2 つになることを書く。
- AC8: `rg -n 'inventory-system-public|C:\\Users\\Owner\\projects|-home-kosei-' docs/DEV_SETUP_CHECKLIST.md docs/msi-distribution-runbook.md` が 0 件。§4.6 に初回の clone の command がある。`rg -n 'C:\\Users\\Alice\\projects\\inventory-system-desktop' docs/DEV_SETUP_CHECKLIST.md docs/msi-distribution-runbook.md` が両方に当たる。
- AC9: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR なし。`bash scripts/tests/claude-hooks.test.sh` と `bash scripts/tests/classify-changes.test.sh` が PASS（既存の test を消さず弱めない）。hosted の `Merge gate` が成功。
- AC10（merge の後）: 本体の checkout を main に進め、`subagent_type: consult-fable` を 1 回起動して、実効の model（`claude-fable-5-2`）と effort（high）を `/tasks` か run の metadata で確かめ、結果を handoff に書く。取れなければ未実測と書く。手元の `.local/codex-orders/MODEL-SELECTION.md`・`.local/codex-orders/templates/`・`.local/checklists/README.md` の旧い agent の名を直す（tracked でない）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/AGENT_OPERATING_MANUAL.md` §3（独立性）・§3.3・`## 座組`。`AGENTS.md` の Decision and Approval Boundaries と Working Rules の「継ぎ足さない」「最小の実装（D-147）」。`docs/DEV_WORKFLOW.md` の Risk Tiers・Draft・Ready・merge・承認依頼。`docs/decision-log.md` D-098・D-145 (14)・D-147。
- 必要な設計成果物: 座組表・AGENTS.md・D-152 を本 PR で書く（updated in this PR）。merge-evidence・helper は existing sufficient（Ready・merge は今も owner の明示の指示を前提にし、常設の委任はその指示の 1 つ）。
- plan にしかない durable な判断の昇格先: TD-620・627・628・630・632 を D-152 と座組表と AGENTS.md へ。agent の名から版を外す判断を D-152 へ。
- 前提・制約と延期した design gap:
  - P1: Claude Code が `model: claude-fable-5-2` の frontmatter を受け、Fable 5.2 で subagent を動かす（公式 sub-agents「Choose a model」は full model ID を受ける。今の `consult-fable5` が `claude-fable-5` で動いた先例がある）。merge の前の worktree の定義は今の session の project の dir の外なので効かず、AC10 で merge の後に確かめる。
  - P2: `.claude/agents/` を agent の tool で書けるか（S1 の「書けないとき」）。
  - TD-632 は owner の確かめの前の案。owner が別の形を選べば、計画の固定の前なら packet を直し、固定の後なら Gated Amendment にする。
- 絶対保証の例外と抜け道: 常設の委任は「lane の外で見つけた」「R0」「目視: 不要」の 3 つを全部満たす PR だけ。R0 の判定は人の判断で、helper は PR body の Risk の行と `--risk`・`--manual` の照合だけを確かめる（Risk の取り違えは機械では止まらない）。取り違えの抜け道は、R0 の意味（意味を変えない docs の直し）が `docs/DEV_WORKFLOW.md` Risk Tiers に固定され、helper の packet なしの道が classifier の `unknown=true`・workflow の実行 code（`workflow=true` かつ `rust=true`）・migration・依存に触れる R0/R1 を止めること（D-145 (8)）で狭める。owner への後の報告は委任の記録として残す。
- 判定: ready。変更は座組表の 4 行と §3.3 の 1 項、AGENTS.md の 1 項目、DEV_WORKFLOW の 1 セル、D-152、agent の定義 3 つ、path の 3 か所に留まる。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| 座組（担当・model・effort） | MANUAL `## 座組`、SPEC-WF-HARNESS2-D1 | S1・S2 | `claude-hooks.test.sh`（frontmatter の禁止 field）、AC1〜AC4 の rg・diff | AC10（merge の後の実効の確かめ） |
| §3 の独立性（Writer ≠ Final Reviewer、fresh context） | MANUAL §3 | 変えない。Fable が書いた lane の Final を fresh Opus 5.5 にする句は独立性を強める側 | 非対象（文書） | 非対象 |
| Codex 停止中の Final（TD-499） | MANUAL §3.3、D-145 (13) | S2（Fable 5.2 に、Fable が書いた lane を除く） | AC4 の rg | TD-632 は owner の確かめ |
| Writer の D-147 の写し | AGENTS.md Working Rules「最小の実装（D-147）」、D-147 | S1（写しを 2 つにし、1 文目で同じ PR で直すことを示す） | AC2 の diff | 非対象 |
| 委任の範囲（Ready・merge・docs・R0） | AGENTS.md Decision and Approval Boundaries、D-145 (14)、D-098 | S3・S4・S5 | 非対象（文書。AC5・AC6 の diff の範囲） | 非対象 |
| merge は helper 経由・直接 UI merge の禁止 | AGENTS.md Working Rules、merge-evidence | 変えない（常設の委任も helper の `ready`・`merge` を通る） | 非対象 | 非対象 |
| Windows の L3 の clone | DEV_SETUP_CHECKLIST §4.6、msi-distribution-runbook 流れ 1 | S6 | AC8 の rg | 次の Windows の L3 で owner が clone を作る |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| agent の定義の frontmatter の安全 | 新しい定義に `hooks`・`permissionMode`・`mcpServers` が入り、project の hook の無効が崩れる | 既存の shell test | `scripts/tests/claude-hooks.test.sh` | `writer-fable.md` の frontmatter に禁止の field を足す |
| `.claude/agents/` の分類 | 新しい定義が full の gate から外れる | 既存の shell test | `scripts/tests/classify-changes.test.sh` | classifier の `.claude/agents/*` の分類を外す（今の mutant の case） |
| 旧い名と ID の sweep | 旧い agent の名・`claude-fable-5` の生きた指示が残る | CLI（rg） | AC3・AC4 | MANUAL か `.claude/agents/` に旧い名か ID を 1 つ残す |
| Writer の本文の写し | Fable の Writer だけ D-147 の梯子が違う・code の所有の文が入る | CLI（diff・rg） | AC2 | `writer-fable.md` の本文の 1 行を変える、ownership の文を足す |
| docs の検査 | link 切れ・packet の形の誤り | 既存の検査 | `bash scripts/doc-consistency-check.sh`・`--target plan` | packet の節を欠く、MANUAL の link を壊す |
| 実効の model と effort | 定義が効かず session の model・effort で動く | 手での確かめ（merge の後） | AC10 | `model:` の ID を誤る、定義が読まれない |
</content>
</invoke>
