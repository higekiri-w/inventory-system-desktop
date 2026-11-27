# Plan Packet: 画面の振舞いは使い手の操作から自然に期待する結果を正とし、review の指摘で設計を変える直しは owner が選ぶ（D-175）

wave に属さない単独の lane（PR #314）。出所は owner の原則と決定（repo 外の回答台帳 TD-936・TD-937・TD-939・TD-942・TD-943・TD-944・TD-946）。最初の実装 `a315cb69` は R2 で入れ、差分の review（PR #314 の issuecomment-7219754724、Codex 1171）で 3 件を受けた: 規則の文書なので R3、第 2 文が保存日数 0 を拒む仕様に戻す直しまで owner の選択待ちにする読み、decision-log の末尾の空行。owner は R3 で進めると決めた（TD-945）。`a315cb69` は revert の commit で打ち消し、本 packet の計画の固定の後に入れ直す。計画の review round 1（Codex 1175 と Coordinator）の後、owner は第 2 文を「設計で決めた振舞いに戻す直しはそのまま入れ、設計を変える直しは owner が選ぶ」形にすると決めた（TD-946）。

## Workflow State

- Risk: R3
- Plan Commit: 9c9018b2f50b14ed760567c4ff4458079db40752
- Amendments: none

## Risk

Risk: R3

Reason:
`AGENTS.md` は classifier が `docs=true workflow=true` を出す手続きの正本で、R2 にできない（`printf '%s\n' AGENTS.md | bash scripts/ci/classify-changes.sh --files-from-stdin`、2026-11-27。`docs/DEV_WORKFLOW.md` の Risk Tiers）。

## Goal

Goal Invariant:

### 最小完了条件

- `AGENTS.md` の「使い手に押しつけない（D-127）」の項目に、画面の振舞いの正（使い手の操作から自然に期待する結果）と、review の指摘で使い手に見える振舞いが変わるときの受け方（設計で決めた振舞いに戻す直しはそのまま入れ、設計を変える直しは owner が選ぶ）がある。
- `docs/decision-log.md` の末尾に D-175 があり、その理由と出所を引ける。

### 失敗定義

- 設計で決めた振舞いに戻す直し（例: 保存日数 0 を拒む仕様に戻す、番号を打った後の Enter を番号の検索に戻す）まで owner の選択待ちになる読みを許す。
- reviewer の指摘で設計を変える直しを owner に見せないまま入れる読みを許す（PR #305 の round 8 の再発）。

### 非目的

- review の本数・独立性・round の天井、`docs/code_review.md` の重大度と裁定、reviewer の発注の雛形。
- `docs/backlog.md` などからの参照（最小）。モデルごとの書き分け（`CLAUDE.md`・`docs/agent-guidance/`）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- S1 `AGENTS.md` Working Rules「使い手に押しつけない（D-127）」: 項目の末尾（「その場で直せるようにする。」の後）に次の 2 文を同じ行で足す。新しい項目は立てない。
  > 画面の振舞いは、使い手がした操作から自然に期待する結果が出るものを正とする（ボタンは名のとおりに動き、番号を打って Enter を押せばその番号を探す）。review の指摘で使い手に見える振舞いが変わるときは、設計で決めた振舞いに戻す直しはそのまま入れ、設計を変える直しは、操作・今の結果・直した後の結果を並べて owner が選ぶ（D-175）。
- S2 `docs/decision-log.md`: 末尾に次の D-175 を足す（番号は `a315cb69` の D-175 を引き継ぐ。この clone の `origin/main` `e95e8946` の最後の見出しは `## D-170`）。file の末尾は D-175 の Revisit の行と改行 1 つで終え、空行を足さない。

  ```markdown
  ## D-175: 画面の振舞いは使い手の操作から自然に期待する結果を正とし、review の指摘で設計を変える直しは owner が選ぶ（2026-11-27）

  - Status: accepted（owner の原則と決定、repo 外の回答台帳 TD-936・TD-937・TD-939・TD-942・TD-943・TD-944・TD-945・TD-946）
  - Context: PR #305（EJ の控えの画面）の Final の closure round 7 で、reviewer が「番号の欄に打って未送信のまま『前の日』を押した後の Enter で番号の検索が走る」を回帰とし、前の版の「Enter でボタンがもう一度押されて 1 日進む」を正とした。Coordinator は、直すと Enter で 1 日進むことを owner に見せないまま、round 7 の指摘を受けて是正 `3c732b7f` で test に固定し、round 8 で approve した。Windows の L3 で owner が「番号を打って Enter したらその番号へ移るのが正、行動に自然と意図した結果が伴うのが正」と示し、その是正を戻した。owner は機能面で何を採るかの基準をこれまで明示していなかったと述べた。待ちの間・失敗の後の止め方と文は UI の刷新の入力に束ねた（TD-943、`docs/backlog.md` の UI の刷新の入力の (j)）。
  - Decision: AGENTS.md の「使い手に押しつけない（D-127）」の項目に 2 文を足す。画面の振舞いは使い手がした操作から自然に期待する結果が出るものを正とする（ボタンは名のとおりに動く、番号を打って Enter を押せばその番号を探す）。review の指摘で使い手に見える振舞いが変わるときは、設計で決めた振舞いに戻す直しはそのまま入れ、設計を変える直しは、操作・今の結果・直した後の結果を並べて owner が選ぶ。共通の決まりなのでモデルごとに書き分けない（モデル固有の補助は `CLAUDE.md` と `docs/agent-guidance/`）。
  - Alternatives: 新しい項目を立てる（D-127 と同じ向きで、継ぎ足しになる）; 除外なしの形（Codex 1171。保存日数 0 を拒む仕様に戻す直しまで選択待ちになる）; 安全要件だけを除く形（設計どおりに戻す直しまで選択待ちになる。計画の review round 1 の 2 本の P2）; review を Claude 側だけにする（owner は採らない、TD-942。見つける力は残し、受け口を締める）; Coordinator の手元の checklist だけに置く（reviewer の発注に効かない）。
  - Revisit: この原則で決めた振舞いを店の L3 や実運用で owner が覆したとき。
  ```

Non-scope: `CLAUDE.md`・`docs/agent-guidance/`（モデル固有の補助で、共通の決まりを写さない）、`docs/code_review.md`・`docs/templates/subagent-review-packet.md`（reviewer は AGENTS.md の Session Start で本文を読む）、`docs/backlog.md`・`Plans.md`・`docs/PROJECT_HANDOFF.md`（参照先は変わらない）。

## Acceptance Criteria

- AC1: `git diff origin/main -- AGENTS.md` が「使い手に押しつけない（D-127）」の 1 行の置換え（削除 1・追加 1）だけで、追加の行が S1 の 2 文を含む。
- AC2: `rg -n "^## D-175" docs/decision-log.md` が 1 件で file の最後の見出し、`git diff origin/main -- docs/decision-log.md` が追加の行だけ、`tail -c 1 docs/decision-log.md | od -c` が `\n` 1 つで、最後の行が空でない。
- AC3: `git diff --check origin/main...HEAD` が出力なし。
- AC4: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-11-27-natural-result-principle.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が OK。

## Design Readiness

- 引用する設計正本（節まで）: `AGENTS.md` Working Rules「使い手に押しつけない（D-127）」「継ぎ足さない（D-127）」「owner の所感は原文で扱う（D-127）」、`docs/DEV_WORKFLOW.md` Risk Tiers、D-127。
- 必要な設計成果物: S1・S2 を本 PR で直す（updated in this PR）。
- plan にしかない durable な判断の昇格先: D-175。
- 前提・制約と延期した design gap: 「設計で決めた振舞い」は個々の review で設計正本から引く（列挙しない）。待ちの間・失敗の後の止め方と文は UI の刷新の入力（TD-943、`docs/backlog.md` の (j)）で扱い、本 lane では扱わない。
- 絶対保証の例外と抜け道: 主張しない。文の規則で、機械で検査しない。
- owner に決めてもらう事項: 無い。文は owner が TD-944 で「それでよさそう」と示し、第 2 文は計画の review round 1 の後に owner が TD-946 で決めた。
- L3: 無し（docs だけ）。
- 判定: ready。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-127（使い手に押しつけない） | AGENTS.md Working Rules | S1 | 非対象（文。AC1） | 非対象（docs だけ） |
| D-175 | `docs/decision-log.md` | S2 | 非対象（文。AC2） | 非対象 |
| D-127（owner の所感は原文で扱う・AI の review から出た要件） | AGENTS.md Working Rules | 変えない（S1 は同じ向き） | 非対象 | 非対象 |
| Risk Tiers | `docs/DEV_WORKFLOW.md` | 変えない（本 packet の Risk） | 非対象 | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| docs の形 | packet・link・decision-log の形が壊れる | CLI（既存） | `scripts/doc-consistency-check.sh`（全体と `--target plan`） | packet の節や decision-log の見出しを壊す |
| 計画の先の実装 | 計画の固定の前に S1・S2 が入る | git（手順。`scripts/check-workflow-git.sh` は 2 file を先走りの対象にしないので根拠にしない） | 固定の前: `git diff origin/main...HEAD -- AGENTS.md docs/decision-log.md` が空。固定の後: `git diff origin/main...<Plan Commit> -- AGENTS.md docs/decision-log.md` が空で、`git log --first-parent --oneline <Plan Commit>..HEAD -- AGENTS.md docs/decision-log.md` に入れ直しの commit がある | `Plan Commit: pending` のまま 2 file を変える、または 2 file の変更が Plan Commit より前に残る（打ち消し済みの `a315cb69`・`314297ad` は差分が空なので通る） |
| 空白 | 末尾の空白・空行 | CLI（既存） | `git diff --check origin/main...HEAD` | 行末の空白や末尾の空行を足す |
