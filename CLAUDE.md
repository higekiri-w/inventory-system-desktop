# 在庫管理システム

@AGENTS.md

Tauri / React / SQLite の単店舗向けアプリ。日本語で応答し、識別子は英語、commit/PR本文は原則日本語にする。

## 共通の入口

上で import した `AGENTS.md` の `Session Start` が唯一の読書ルートで、ここに順序や gate を複製しない。`.claude/skills` の既存 symlink は `.agents/skills/` と同じ Skill 本文を共有する。

tracked project hook inventoryは空で、`claude-code-harness`はproject scopeで無効。計画の固定（`scripts/plan-approve.sh`・PK5・先走りの検査）をClaude固有hookで再実装しない。

## 長い作業の報告

長いtool作業では開始前と節目に、現在の結果・不確実性・次の行動を短く共有する。独立したtool callはまとめ、最終応答は単独で結果・根拠・未完了事項が分かる形にする。

モデル固有補助は [Anthropicの当該モデル向けガイド](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices) と実際の観測で選ぶ。Astraの特性をClaudeへ転記しない。

owner に渡すコピペ物（command・URL の一覧・checklist）は返答の本文に inline で書き、path は省略しない完全な形で書く（WSL は `/home/kosei/...`、Windows は `C:\Users\Owner\...`、Windows から WSL の file は `\\wsl.localhost\Ubuntu-22.04\home\kosei\...`。D-127）。

## 座組と effort

座組と effort の既定値は [座組表](docs/AGENT_OPERATING_MANUAL.md#座組) が正本で、ここに値を書かない。`.claude/settings.json` の `effortLevel` は main session の値。project設定を読まない `--safe-mode` 等の別セッションでは effort を明示指定し、要求値と取得できた実行metadataを記録する。

安全境界（safe wrapper・path traversal・repo の外の参照・execpolicy）の敵対的な検証は Claude の subagent に発注しない。発注の分野そのものが dual-use の安全の判定に掛かり、model が切り替わって止まる（PR #15 で語の言い換えでも再現した）。この分野の review は Coordinator が source を直接読む inline の review か、Codex への発注で行う。Coordinator の inline の review は fresh context でないので Final Review の本数に数えない。この分野の lane の Claude 側の枠の扱いは、共通の正本で決めるまで、その lane の起票で owner に諮る（D-127。backlog に項目がある）。

## 再開とmemory

停止時は `Ctrl+C`、別terminalから `claude --resume <session-id>`、再発時は `--fork-session` を検討する。transcript破損が疑われるAPI 400等では新規sessionでAGENTSの該当ルート、対象の現物、直近の引継ぎから再開する。

完了は git log、lane の現在地は helper の status、関門・ブロッカーは `Plans.md`、安定した事実は `docs/project-memory.md`、判断は `docs/decision-log.md` とdesign doc（AGENTS の Memory Model）。個人auto-memoryは補助であり、書込みや読取りをgateにしない。repo固有の成果物は作業checkoutに保持し、`~/.claude/`等の個人領域へ移さない。
