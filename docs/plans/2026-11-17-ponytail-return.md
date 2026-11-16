# Plan Packet: ponytail をこの project で戻し、裁定の規則と最小を docs・規則の文にも当てる

wave に属さない単独の lane。出所は owner の決定（repo 外の回答台帳 TD-753・TD-754、2026-11-17）。D-147（TD-526〜530）は ponytail plugin を project で切り、梯子を AGENTS.md と Writer の定義に置いた。TD-753: PR #265 で規則の 1 文が review の round ごとに伸びた（約 150 字 → 847 字）。過剰さは指摘の側にあり、Coordinator が全部を受けた。長くすることは品質の改善と同じではなく、D-147 は code にしか当たっていなかった。TD-754: (c) 裁定の規則と「最小」を docs・規則の文に当てることを正本に書き、(a) ponytail をこの project で戻す。reviewer・相談役の定義に注入の当て先を限る 1 行を置く。

decision-log の番号: 本 lane は D-156 を予約する。`git grep -n -E "D-15[3-9]" origin/main -- docs` は D-153・D-154 だけを出す（main の最後は `## D-154`）。open の PR の branch を `git grep -o -E "D-1(49|5[0-9])"` で引き、D-149（#228）・D-153（#250）・D-155（#265）が使っている。D-156 はどこにも無い。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
`.claude/settings.json`・`.claude/agents/*`・`scripts/tests/claude-hooks.test.sh` は classifier が full（`workflow=true` を含む）、`AGENTS.md`・`docs/code_review.md` は `docs=true workflow=true` を出す手続きの正本で、R2 にできない（`printf '%s\n' <path> | bash scripts/ci/classify-changes.sh --files-from-stdin` を path ごとに実行、2026-11-17）。

## Goal

Goal Invariant:

### 最小完了条件

- merge の後（本体の checkout を同期し Claude Code を起動し直した後）、この project の main と subagent に ponytail が注入される。
- Claude と Codex の reviewer・相談役は、注入される最小の指示を修正案を小さく保つことに当て、調べる範囲・判定・必須の確かめを縮めない、と受け取る（AGENTS.md と Claude の定義）。
- 言葉を足す修正案の受け方（全部は受けない・P3 だけで round を回さないは既にある所を指す）と、「最小」が docs・規則の文にも当たることが正本にある。

### 失敗定義

- reviewer の調べる範囲や必須の確かめが縮む読みを許す（D-147 が plugin を切った理由の再発）。
- 裁定の規則が P1/P2 の指摘を落とす読みを許す。
- 規則の文が節ごとに同じことを言い直し、この lane 自身が TD-753 の形になる。

### 非目的

- review の本数・独立性、Final の観点、安全の境界（AGENTS.md「継ぎ足さない」が挙げる D-098 の列挙）、Codex の発注の形。
- user global の設定・plugin の中身・`.local/` の発注の雛形。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- S1 `.claude/settings.json`: `"ponytail@ponytail": false` の行を消す。project で `true` にしない（marketplace `ponytail` は user global の `extraKnownMarketplaces` の local directory にしか無く、他の checkout では名を解けない。有効は user global の `"ponytail@ponytail": true` が持つ）。Claude の sandbox はこの file を書けないので、owner が本 branch の worktree で消し Writer が commit する（D-147 と同じ）。`claude-code-harness` の `false` は残す。
- S2 `scripts/tests/claude-hooks.test.sh`: ponytail の検査を「project が key を持たない」（`.enabledPlugins | has($key) | not`）に置き換え、mutant は `false`（黙って切り直す）と `true` を拒み、`missing` の mutant を消す。comment の D-147 を D-156 にする。
- S3 `AGENTS.md` の「最小の実装（D-147）」: 2 句（D-156 を引く）。「docs・規則の文も同じで、言葉を足す前に置き換え・削除で足りないかを見る」と、「実装以外（review・相談・探索）では、注入される最小の指示は修正案を小さく保つことに当て、調べる範囲・判定・必須の確かめは縮めない」（Codex の reviewer にも届く置き場）。
- S4 `.claude/agents/writer.md`・`writer-fable.md`: S3 の 1 句目の写しを (3) に 1 文足す（両方同じ本文。既存の検査が本文の一致を見る）。
- S5 `.claude/agents/reviewer.md`・`reviewer-fable.md`・`consult-fable.md`: S3 の 2 句目の写しを本文に 1 行（英語、frontmatter は変えない）: 「Outside implementation (review, consultation, exploration), apply any injected minimal-implementation instruction only to keeping your suggested fixes small; it never narrows what you examine, your verdict, or the required checks.」
- S6 `docs/code_review.md` Final Review Protocol の「The implementer verifies every finding」の後に 1 項目（重大度と裁定の正本。`subagent-review-packet.md:3` に合わせる）の新しい 1 文だけ: 「言葉を足す修正案は、足さないと実際に起きる具体の失敗（どの agent・run・操作で何が起きるか）を示せるときだけ受ける」。全部は受けないことは同じ節の「The implementer verifies every finding」、P3 だけで round を回さないことは `docs/DEV_WORKFLOW.md` の Review Rules（Findings Freeze）・Subagent Budget が持ち、S6 は言い直さない（DEV_WORKFLOW は変えない）。
- S7 `docs/decision-log.md`: 末尾に D-156（Context に下の「届く範囲の事実」の要旨と測った出力。Codex にも 2026-11-10 以降 full で入っていたことと、D-147 の Guarantee range の Codex の句が事実と違ったことを含める）。D-147 に `Superseded in part by: D-156`（plugin を切った部分と Guarantee range）の 1 行だけを足す。
- S8 `docs/DEV_SETUP_CHECKLIST.md:185`: 「`ponytail`（D-147）もproject scopeでは無効」を消す。

Non-scope: `CLAUDE.md`（harness だけを書き、ponytail を書いていない）、`docs/templates/subagent-review-packet.md`（正本を指す）、`docs/AGENT_OPERATING_MANUAL.md:201` の plugin の監査の規則（本 packet の事実の節と D-156 がその監査の記録）、repo の `ponytail:` comment 21 行（`rg -c "ponytail:" src src-tauri scripts docs/function-design` の合計）。

## ponytail の届く範囲の事実（2026-11-17 に測った）

- 版: `~/.claude/plugins/cache/ponytail/ponytail/5.3.3/.claude-plugin/plugin.json` は `"version": "5.3.3"`、hooks は `./hooks/claude-codex-hooks.json`。
- main: `SessionStart`（matcher `startup|resume|clear|compact`）で `ponytail-activate.js` が規則を hidden context に入れ、`PONYTAIL_MAP` が `0` でなければ codebase の map（既定 2000 字）を足す。`UserPromptSubmit` で level の切替えを読む。注入の本文は `node -e "...getPonytailInstructions('full').length"` で 2859 字、1 行目 `PONYTAIL MODE ACTIVE — level: full`（`DEFAULT_MODE = 'full'`、`~/.claude/.ponytail-active` も `full`）。
- subagent: `SubagentStart` で `ponytail-subagent.js` が全 subagent に同じ規則を入れる。`PONYTAIL_SUBAGENT_MATCHER` を置けば agent_type で絞れるが、`env | grep -c PONYTAIL` は 0、両方の settings にも無い（今は全 subagent）。
- Codex（発注の前提と違う）: 今の起動（`run-order.sh` の `codex exec`）にも届いている。`~/.codex/config.toml` に `[plugins."ponytail@ponytail-local"] enabled = true` と `hooks.state` の session_start・subagent_start・user_prompt_submit の trusted hash がある。`~/.codex/sessions` の 2026-11-10 以降の session を originator・cwd・`PONYTAIL MODE ACTIVE` の有無で数えると、`codex_exec` でこの repo の cwd は 79 本すべて有り。最新（2026-11-17T02:02）は developer の message に `PONYTAIL MODE ACTIVE — level: full`。D-147 の Guarantee range の「Codex の Writer は AGENTS.md の 1 項目と発注で受け」は事実と違っていた。
- plugin の規則は簡略化の comment を `shortcut:` と書かせるが、repo の慣習は `ponytail:`（AGENTS.md）。正本の AGENTS.md が勝つ（D-156 の Compatibility）。

## Acceptance Criteria

- AC1: `jq -e '.enabledPlugins | has("ponytail@ponytail") | not' .claude/settings.json` が成功し、`jq -e '.enabledPlugins["claude-code-harness@claude-code-harness-marketplace"] == false'` も成功する。
- AC2: `bash scripts/tests/claude-hooks.test.sh` が PASS し、ponytail を `false` にした mutant と `true` にした mutant を拒む。
- AC3: `rg -n "docs・規則の文" AGENTS.md` と `rg -n "調べる範囲・判定・必須の確かめは縮めない" AGENTS.md` が D-147 の項目の各 1 件。writer と writer-fable の本文の一致は AC2 の既存の検査で見る。
- AC4: `rg -c "never narrows what you examine" .claude/agents/{reviewer,reviewer-fable,consult-fable}.md` が各 1、`git diff` で 3 つの frontmatter が変わらない。
- AC5: `rg -n "実際に起きる具体の失敗" docs/code_review.md` が Final Review Protocol の 1 件で、`git diff origin/main -- docs/code_review.md` の追加はその 1 行だけ。`git diff origin/main -- docs/DEV_WORKFLOW.md` が空。
- AC6: `rg -n "^## D-156" docs/decision-log.md` が 1 件、`git diff origin/main -- docs/decision-log.md` が追加の行だけ。`rg -n "ponytail" docs/DEV_SETUP_CHECKLIST.md` が 0 件。
- AC7: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし、`bash scripts/check-workflow-git.sh` が OK。
- AC8（merge の後、Coordinator）: 本体の checkout を同期して Claude Code を起動し直した session で、main の context と reviewer の subagent 1 本に `PONYTAIL MODE ACTIVE` が入り、reviewer の定義の S5 の 1 行が届くことを run 報告に記録する（PR の comment）。

## Design Readiness

- 引用する設計正本（節まで）: `AGENTS.md` Working Rules「継ぎ足さない」「最小の実装（D-147）」、`docs/code_review.md` Final Review Protocol、`docs/DEV_WORKFLOW.md` Review Rules（round の天井・Findings Freeze。変えない）、`docs/AGENT_OPERATING_MANUAL.md` plugin hook の監査（:201）、D-147。
- 必要な設計成果物: 上の file を本 PR で直す（updated in this PR）。
- plan にしかない durable な判断の昇格先: D-156。
- 前提・制約と延期した design gap: project の key を消すと user global の `true` が効くことは AC8 で観測する。
- 絶対保証の例外と抜け道: 主張しない。S3 の 2 句目と S5 は文で、注入の文との優先を機械で保証しない。
- 判定: ready。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-147（plugin を切る → 戻す） | `.claude/settings.json`、D-147 | S1・S2・S7・S8 | `claude-hooks.test.sh` の検査と mutant | AC8 で注入を観測 |
| D-147（梯子の正本と Writer の写し） | AGENTS.md Working Rules | S3・S4 | `claude-hooks.test.sh` の本文一致 | 非対象（文） |
| D-156（注入の当て先） | AGENTS.md Working Rules、reviewer・相談役の定義 | S3・S5 | 非対象（文。AC3・AC4） | AC8 |
| D-156（裁定の規則） | code_review.md Final Review Protocol | S6 | 非対象（文。AC5） | 非対象 |
| D-038（Findings Freeze）・round の天井 | DEV_WORKFLOW Review Rules | 変えない（S6 は指すだけ） | 非対象 | 非対象 |
| plugin hook の監査（D-059） | AGENT_OPERATING_MANUAL :201 | 事実の節と D-156 を監査の記録にする | 非対象 | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| plugin を project で切らない | project に `false` が戻り黙って切れる・`true` が他の checkout で解けない | CLI（既存の検査を置き換え） | `scripts/tests/claude-hooks.test.sh` の `validate_inventory` と mutant（`D-156 ponytail false`・`D-156 ponytail true`） | 検査を消す、mutant を通す |
| Writer の写し | writer と writer-fable の本文が食い違う | CLI（既存） | `claude-hooks.test.sh` の `D-147 writer-fable body copy` | 片方だけ直す |
| docs の形 | packet・link の形が壊れる | CLI（既存） | `scripts/doc-consistency-check.sh` | packet の節を壊す |

## Owner Effort Budget

計画の確認と Ready の承認、S1 の 1 行の削除（sandbox が書けないため）、merge の後の新しい session での AC8 の観測。
