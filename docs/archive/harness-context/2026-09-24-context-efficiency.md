<!-- 2026-09-24 に docs/agent-guidance/ から移送した設計記録。本文を保存し、相対リンクだけ移送先に合わせて補正。 -->

# ハーネスの文脈効率

Status: accepted（2026-09-14、[PR #52](https://github.com/higekiri-w/inventory-system-desktop/pull/52)でmerge）。この入口を適用する。整備PR自身は変更前のworkflow gateで検証した。

## 目的と境界

SPEC-HARNESS-CONTEXT: Astra を主対象に、Sol と Claude Code でも、必要な契約を維持してタスク完了までの読み込みと重複作業を減らす。token 削減率は未実測。file bytes は入力負担の代理値であり、課金 token や cache hit の実測として扱わない。

この設計では Risk、Plan Gate、vendor 独立性、Workflow State、R4 承認、L3、Ready/merge、最終 HEAD の証拠を変更しない。既存の規範を短い入口から必要時に参照する。R2 Plan Gate廃止とreviewer固定の撤廃は対象外。証跡保存・STATECAPの後継は[merge-evidence](../../agent-guidance/merge-evidence.md)でEvidence Modeごとに定義し、Plan Commit保護を維持する。

## 契約

| ID | 採用する設計 | 理由・棄却案・再検討条件 |
|---|---|---|
| HC-D1 | AGENTS の Session Start が唯一の入口。全作業共通は root の安全・権限境界と依頼の識別。質問/調査、小変更、計画/実装、レビュー、closure、resume に応じて関係する正本を読む | 全作業への DEV_WORKFLOW/Plans/project-memory の全文必読を撤去。無関係な packet に依頼を紐づけない。関係資料の欠落を推測で埋める案は棄却 |
| HC-D2 | R2+ の着手/再開は現在の対象と該当 packet の完全な Workflow State を確認し、必要な Risk/Design/State/Review/CI 条件を読む。読み込み量の削減は gate の省略ではない | 日付が古いことだけで blocker を無効化しない。現在の依頼と対象に関係する未解決 gate は維持。malformed/missing/ambiguous packet は現行の fail-closed |
| HC-D3 | 初回レビューは影響する設計正本と差分を直接確認。closure は既存 findings、修正差分、影響する契約を起点にし、別領域へ影響する変更や新しい重大欠陥の根拠があれば範囲を拡張 | Writer の要約だけのレビュー、常時全 repo の再監査の両案を棄却。独立レビューの回数・要否は既存規定を維持 |
| HC-D4 | inventory-workflow-start と inventory-implementation は短い router として、依頼識別/既存権限/必要な参照先/完了を支える repo 固有情報だけを持つ。workflow の表と一律の報告型を再掲しない | generic implementation/TDD の重複起動を避ける。既存 Skill の名前、通常の自動選択、Claude 側 symlink は維持 |
| HC-D5 | 検証の正本は DEV_WORKFLOW と ci.md。Skill/Claude rule は独自の full 再実行や既存成果物の一律削除を要求しない | 必須チェックは省略しない。human-confirm state-only 後だけの追加 full は実装 Skill の重複規定として除去。失敗再現と重要な test oracle の検出力確認は維持 |
| HC-D6 | Plans は現行の task/未解決判断/次の行動/参照を持ち、長い未了 backlog は別文書へ、完了履歴は archive へ移す。project-memory は安定した事実、PROJECT_HANDOFF は案内にする | 移送前の原文を archive に保存し、未了項目の本文・未決判断・参照先を移送先と全件対応づけて照合。件数だけを保存の根拠にしない。現在の detached checkout にある別件 Plans 修正は取り込まず、merge 前の最新 main 差分で取りこぼしを確認する |
| HC-D7 | Codex と Claude で共通の契約を共有し、モデル固有補助は各 vendor の公式資料と観測に基づき分離する。Claude の rules/commands も必要な資料と影響範囲の gate へ参照する | Astra の特性を Claude/Sol の事実として転記しない。個人の応答キャラクターや供給網/データ保護を削除しない |
| HC-D8 | read-safe-file.sh の先頭引数としてのみ `--lines START:END <path>` を認識し、その他の option 風引数は既存どおり拒否したうえで、正整数の閉区間だけを表示できる。無指定の複数 path 読込みは互換維持。path 正規化/allowlist/secret拒否は部分読込みにも同じものを使う | 範囲確認には search-safe-files.sh の見出し行を利用。任意 shell/absolute path/外部 Skill を許可する案は棄却。空、逆順、option風、CR/LF、不正数値は非0で拒否 |
| HC-D9 | 読書順序テストを旧固定順序の文字列一致から新しい入口契約へ整合させる。safe wrapper の実入出力を synthetic fixture で検証し、モデルによる判断は既存 eval の固定ケースと追加の代表ケースで確認する | 文言だけのテストで判断品質を保証したとしない。自動テストと実モデル観測を区別する |
| HC-D10 | 起動先・権限・hooks の新規変更はこの slice では行わない。既知の agmsg Stop hook、global Skill 公開範囲、worktree の起動先は後続の具体的確認対象として列挙する | ignored/user-global 資産は共有 PR だけでは配布できない。現物と実効状態を確認してから別の狭い変更にする |
| HC-D11 | 採用前後の代表ケースでは親/子/再試行の取得可能な usage と読書量を区別して記録し、契約理解/完遂/不要停止を併記する。未取得の token は未実測と表示する | 削減目標値は未実測。低リスクの read-only 問合せと closure を優先して比較し、全モデル/全ケースの巨大な常設評価基盤は作らない |

## 適用と復旧

整備 PR は旧 gate のまま plan-first、非 Writer の Plan Review、local full、workflow change の Double Audit を通す。既存 packet の field や SHA をこの改訂のために書き換えない。merge 後に新しい Session Start を利用し、進行中 task も既に通過した gate の繰返しはせず現在の Workflow State から再開する。読取り範囲の判断に不足があれば該当する正本へ拡張する。

問題があれば該当する instruction/helper の変更を通常の修正 PR で戻す。履歴の破壊、force push、DB 復元は必要ない。

## 参照した資料

- [共通 workflow](../../DEV_WORKFLOW.md)、[役割](../../AGENT_OPERATING_MANUAL.md)、[CI](../../ci.md)
- [OpenAI: Rethinking skills and prompts for GPT-6 Astra](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)
- [OpenAI: prompting](https://developers.openai.com/api/docs/guides/latest-model#prompting-best-practices)
- [Anthropic: Claude Code best practices](https://code.claude.com/docs/en/best-practices)
- [Anthropic: Skills](https://code.claude.com/docs/en/skills)
- [Anthropic: model-specific prompting](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices)

公式資料確認日: 2026-09-13。モデル固有の助言は該当モデルだけに適用し、API 専用の仕組みを CLI 設定として扱わない。
