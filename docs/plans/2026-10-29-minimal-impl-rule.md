# Plan Packet: 最小の実装の梯子を正本に置き、ponytail plugin を project で外す

wave に属さない単独の lane。手続きの正本の文（`AGENTS.md`・`docs/code_review.md`・`docs/decision-log.md`）、Writer の agent の定義（`.claude/agents/writer.md`）、project の設定（`.claude/settings.json`）とその検査（`scripts/tests/claude-hooks.test.sh`）を変える。製品の runtime（`src/**`・`src-tauri/**`）と helper・CI は変えない。

出所: owner の判断（repo 外の回答台帳 TD-526〜530、2026-10-29）。ponytail plugin（user global の `/home/kosei/.claude/settings.json` の `enabledPlugins` で有効。SessionStart で main に、SubagentStart で全 subagent に「最小の実装」の指示を入れる。本文は frontmatter を除いて約 5,700 字）をこの project で切る（TD-528）。owner「でも強度と効果は保ちたい」を受け、置き換えと切る操作を同じ merge に入れる（TD-529・TD-530）。AGENTS.md への 1 行は単独の R3 にせず手続きの lane に束ねる（TD-527）。本 lane がその束ねる先であることは、owner が「今から手続きやろう、これでワークフロー周りはおしまいかな」（2026-10-29、置き換えの lane を今起こすか他を先にするかの問いへの答え）で起こした lane であることによる。plugin の注入が reviewer・相談役にも入ることは指示の棚卸し（repo 外の `.local/reports/instructions-reorg-2026-10-27/inventory.md` の §1.2・§5.2）が挙げ、計画の review round 1 の reviewer の run でも `PONYTAIL MODE ACTIVE — level: full` の注入を観測した。

decision-log の番号: 本 lane は D-147 を予約する（main は D-146 まで。open の PR が足す D は D-143 まで）。

## Workflow State

- Risk: R3
- Plan Commit: 731afe462d5395cd1449117e5f997d17e6c3e9a9
- Amendments: none

## Risk

Risk: R3

Reason:
`AGENTS.md`・`docs/code_review.md`・`.claude/agents/*`・`.claude/settings.json`・`scripts/tests/*` は classifier が `workflow=true` を出す手続きの正本と検査で、R2 にできない（`docs/DEV_WORKFLOW.md` Risk Tiers）。

## Goal

Goal Invariant:

### 最小完了条件

- merge の後（本体の checkout を同期し、Claude Code を起動し直した後）、この project の session では ponytail plugin が main にも subagent にも注入されない。
- その状態で、Claude の Writer は起動のたびに agent の定義で最小の実装の手順（下の Scope の writer.md の (1)〜(6)）を受け取る。
- Codex を含む全部の agent が毎回読む `AGENTS.md` に、最小の実装の規則が 1 項目ある（正本）。
- 全部の reviewer が読む `docs/code_review.md` に、作り込みすぎ（`/ponytail-review` が探していたもの）を見る観点と、その重大度がある。

### 失敗定義

- 梯子が「削ってはいけないもの」（入力の検証・データを失わないための error 処理・安全の境界・アクセシビリティ・明示の要求）を落とし、正しさや安全を削る読みを許す。
- 梯子が、承認済みの要件（packet・PR body）が求める依存・抽象・設定まで禁じ、発注とぶつかって不要な停止や要求の欠落を招く。
- 「最小」を差分の行数で読ませ、AGENTS.md Safety の「小ささは確かめやすさで測る」と食い違う。
- Writer の定義が test の数に上限を置く読みを許し、packet の Test Plan や AGENTS.md の「意味のあるテスト」と食い違う。
- code_review.md の観点が、作り込みすぎだけを理由に P1/P2 の blocking を出させ、正しさの review を押しのける。
- reviewer・相談役の定義に Writer 向けの手順を入れる。
- 置き換えと切る操作の間に、どちらも効かない版ができる。

### 非目的

- 効き目の測定の仕組み（D-147 の Revisit）。
- Coordinator の `.local/` の発注の雛形（repo の外。Writer の雛形の梯子は Codex の Writer への補いとして残すかを repo の外で決める）。
- user global の設定（`/home/kosei/.claude/settings.json`）と他の project。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `AGENTS.md` の Working Rules: 「継ぎ足さない」の後に 1 項目（日本語、正本）。文の案: 「最小の実装（D-147）: 実装では、触る code と呼出し側を読み切ってから、承認済みの要件と品質・安全の規則を満たす最初の手段を、要るか → 既にある helper・pattern → 標準ライブラリ・標準機能 → 入れてある依存 → 新しい code、の順で採る。既にある手段で満たせるなら、依存・抽象・設定・後のための足場を足さない（小ささは Safety のとおり確かめやすさで測る）。入力の検証・データを守る error 処理・安全の境界・明示の要求は削らない。意図した簡略化の限界と広げる条件は `ponytail:` comment に書く。」
- `.claude/agents/writer.md` の本文（frontmatter は変えない）: 今の 3 行の後に、Writer の起動のたびに入れる最小の実装の節（英語、今の本文に合わせ、言い切りの形）。AGENTS.md の項目の意図した写しで、強さのために system prompt に置く。中身: (1) 先に問題を読み切る（触る code・呼出し側・流れ）。梯子は理解の後で、読む量を減らす理由にしない。(2) 梯子 — 上から最初に成り立つものを採る: 要るか（推測の要求なら作らない）→ この repo に既にある helper・型・pattern → 標準ライブラリ・標準機能 → 入れてある依存 → 新しい code。同じ大きさの候補なら境界条件に正しい方を採る。(3) 承認済みの要件を既にある手段で満たせるなら、依存・実装が 1 つの interface・変えない値の設定・後のための足場を足さない。(4) bug は根本で: 共有の関数を直す前に呼出し側を全部 `rg` し、1 か所で直す。(5) 削ってはいけないもの: 信頼の境界の入力の検証、データを失わないための error 処理、安全の境界、アクセシビリティの基本、明示の要求。(6) 意図した簡略化で限界があるものは `ponytail:` comment に限界と広げる条件を書く（repo の慣習）。test の数と形は packet の Test Plan と AGENTS.md に従い、この節では決めない。
- `docs/code_review.md` の Blocking Review Focus の「保守者として読めるか」の後に 1 項目: 作り込みすぎ（既にある helper・標準機能の再実装、使わない柔軟さ、実装が 1 つの抽象、承認済みの要件に無い依存・推測の機能）。finding は削る所と置き換えを名指し、同じ振舞いと安全が保てる根拠を添える。重大度は P3。作り込みすぎだけを理由に P2 にしない（保守者が読めない具体〈どこが・なぜ〉を示せるときは既存の P2 の規則による）。
- `.claude/settings.json`: `enabledPlugins` に `"ponytail@ponytail": false` を足す（TD-530。Claude の sandbox はこの file を書けないので owner が本 branch の worktree で足し、Claude が commit する）。
- `scripts/tests/claude-hooks.test.sh` の `validate_inventory`: `HARNESS_KEY` と同じ形で `"ponytail@ponytail"` が `false` であることを確かめ、`true` にした mutant を拒む行を足す（今の harness の key の mutant と同じ形）。
- `docs/decision-log.md` の末尾に D-147。Status accepted（owner の判断 TD-526〜530）。Context: plugin の注入の範囲（main・全 subagent。reviewer・相談役にも「lazy」の指示が入る）と owner の「強度と効果は保ちたい」。Decision: 規則の正本は AGENTS.md の 1 項目。writer.md の節は Writer の起動のたびに system prompt に入れて強さを保つ意図した写しで、直すときは同じ PR で両方を直す。review の観点は code_review.md。plugin は同じ merge で project の設定で切り、効くのは本体の checkout を同期し Claude Code を起動し直した後。Why: agent の定義の本文は起動のたびに system prompt に入る、AGENTS.md は Codex も読み非 fork の subagent にも載る（context として。system prompt より弱い）、reviewer・相談役には Writer 向けの手順を入れない（AGENTS.md の共通の項目は全 agent に届くが、実装のときに当てるもので reviewer の調べる範囲・判定・必須の検証を縮めない）。Alternatives: plugin のまま、今すぐ切る（置き換えまで効かない版ができる）、`PONYTAIL_SUBAGENT_MATCHER=^writer$` で Writer だけに注入する（main への注入が残り、設定が env と user global に散り、Codex に届かない）、merge の後に別の lane で切る（R3 がもう 1 本要る）、`.claude/settings.local.json` で手元だけ切る（他の checkout に効かない）、CLAUDE.md にも書く（`@AGENTS.md` で取り込むので重複）。Compatibility: D-127 を変えない（「最小」は D-127 の確かめやすさの意味で読む）。Guarantee range: Codex の Writer は AGENTS.md の 1 項目と発注で受け、Claude の Writer より弱い。main は AGENTS.md と global の `~/.claude/CLAUDE.md` の行だけ。Revisit: 切った後の数本の lane の Final で作り込みすぎの指摘を見る。比べる基準として、切る前の lane（#220 など）の差分を同じ観点で数えておく。

Non-scope: `.claude/agents/reviewer.md`・`reviewer-fable5.md`・`consult-fable5.md`（入れない）、`CLAUDE.md`（`@AGENTS.md` で取り込むので書かない）、`docs/templates/subagent-review-packet.md`（code_review.md を指すので複製しない）。repo の中の `ponytail:` comment は名前のまま残す（plugin 名に頼らず読める。`rg -n "ponytail:" src src-tauri scripts` で 7 行）。

## Acceptance Criteria

- AC1: `AGENTS.md` の Working Rules に D-147 を引く最小の実装の項目が 1 つ（`rg -c "D-147" AGENTS.md` が 1）で、「承認済みの要件」「確かめやすさ」「削らない」を含む。`CLAUDE.md` は変わらない。
- AC2: `.claude/agents/writer.md` の frontmatter が変わらず（`git diff` で `---` の間が同じ）、本文に Scope の (1)〜(6) がある。test の数を決める文が無い。`rg -n "ponytail|minimal" .claude/agents/reviewer.md .claude/agents/reviewer-fable5.md .claude/agents/consult-fable5.md` が 0 件。
- AC3: `docs/code_review.md` の Blocking Review Focus に作り込みすぎの項目があり、重大度 P3 と「作り込みすぎだけを理由に P2 にしない」を書く。
- AC4: `jq -e '.enabledPlugins["ponytail@ponytail"] == false' .claude/settings.json` が成功し、`claude-code-harness@…` の `false` も残る。
- AC5: `bash scripts/tests/claude-hooks.test.sh` が PASS し、`ponytail@ponytail` を `true` にした mutant（と key を消した mutant）を拒む。
- AC6: `rg -n "^## D-147" docs/decision-log.md` が 1 件。D-146 以前の本文は変わらない（`git diff origin/main -- docs/decision-log.md` が追加の行だけ）。
- AC7: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし。
- AC8（merge の後、Coordinator）: 本体の checkout を同期して Claude Code を起動し直した session で、subagent を 1 本起動し、`PONYTAIL MODE ACTIVE` の注入が無く、Writer の定義の節と AGENTS.md の D-147 の項目が届くことを run 報告に記録する（PR の comment）。

## Design Readiness

- 引用する設計正本（節まで）: `AGENTS.md` Working Rules（「継ぎ足さない」「意味のあるテスト」）と Safety（「小さく、確認可能な変更」）と npm 供給網ガード（依存の追加の条件）、`docs/code_review.md` Blocking Review Focus・Finding Severity、`docs/AGENT_OPERATING_MANUAL.md` 座組（Writer の `subagent_type: writer`、settings は本体の checkout を同期し再起動してから頼る）、`docs/decision-log.md`「Status の書き方」。
- 必要な設計成果物: 上の file を本 PR で直す（updated in this PR）。
- plan にしかない durable な判断の昇格先: D-147。
- 前提・制約と延期した design gap: agent の定義の本文が起動のたびに system prompt に入ること（公式 sub-agents「Write subagent files」）と、非 fork の subagent が CLAUDE.md と `@AGENTS.md` を載せること（公式 memory。計画の review round 1 の reviewer の run で観測）は確か。project の `enabledPlugins` の `false` が user global の `true` に勝つこと（公式 settings の優先順）は AC8 で実測する。効き目の同等性は測らない（Revisit）。
- 絶対保証の例外と抜け道: 主張しない。Codex の Writer と main は Claude の Writer より弱く受ける（D-147 の Guarantee range）。
- 判定: ready。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-147（全 agent の規則、正本） | AGENTS.md Working Rules | AGENTS.md の 1 項目 | 非対象（文） | 非対象 |
| D-147（Writer の写し） | `.claude/agents/writer.md` の本文、座組の Writer | writer.md の節 | `claude-hooks.test.sh`（定義の形） | AC8 で配送を観測 |
| D-147（review の観点） | code_review.md Blocking Review Focus・Finding Severity | code_review.md の 1 項目 | 非対象（文） | 非対象 |
| D-147（plugin を切る） | `.claude/settings.json` の `enabledPlugins` | settings.json の 1 行、検査の 1 行 | `claude-hooks.test.sh` の `validate_inventory` と mutant | AC8 で注入が無いことを観測 |
| D-127（継ぎ足さない）・Safety（確かめやすさ）・npm 供給網ガード | AGENTS.md | 新しい項目を実装の手段の選び方に絞り、承認済みの要件の依存を禁じない | 非対象 | 非対象 |
| decision-log の追記の規則 | decision-log「Status の書き方」 | D-147 を末尾に足す | 非対象 | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| plugin を project で切る | settings の `ponytail@ponytail` が `true` に戻る・消えて user global の `true` が効く | CLI（既存の検査に足す） | `scripts/tests/claude-hooks.test.sh` の `validate_inventory` と mutant（`D-147 ponytail true`・`D-147 ponytail missing`） | 検査の行を消す、mutant を `true` にしても通す |
| agent の定義の形 | 本文の追記で frontmatter が壊れる・hooks を持つ | CLI（既存） | `scripts/tests/claude-hooks.test.sh` | writer.md の frontmatter を壊す、hooks の key を足す |
| docs の形 | link・見出し・packet の形が壊れる | CLI（既存） | `scripts/doc-consistency-check.sh`（PK1・PK4 ほか） | packet の節や Workflow State を壊す |
