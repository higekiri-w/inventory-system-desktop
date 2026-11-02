# プロジェクト引き継ぎ案内

現在の作業を再開するときは [AGENTS.md](../AGENTS.md) `Session Start` の該当ルートを使う。この案内や過去ログの全文を毎回読む必要はない。

| 知りたいこと | 正本 |
|---|---|
| 完了・lane の現在地・関門 | 完了は `git log origin/main`、lane の現在地は `gh pr list --state open` と helper の status、導入の前の関門・ブロッカー・製品の未決判断は [Plans.md](Plans.md)（D-145） |
| 未了候補・保留・受容済みリスク | [Backlog](backlog.md) |
| 安定した業務・POSの前提 | [project-memory.md](project-memory.md) |
| 層と依存関係 | [ARCHITECTURE.md](ARCHITECTURE.md) |
| 振舞い・command・エラー契約 | [FUNCTION_DESIGN.md](FUNCTION_DESIGN.md) の対象サブ文書 |
| 永続化・migration | [DB_DESIGN.md](DB_DESIGN.md) |
| 更新の前の版へ戻す管理者の手順（migration v8 以降） | [migration-recovery-runbook.md](migration-recovery-runbook.md)（正本は [22](function-design/22-mnt-migration.md) §16） |
| アプリの MSI を作る・入れる・更新する・前の版へ戻す管理者の手順 | [msi-distribution-runbook.md](msi-distribution-runbook.md) |
| 店の PC で本物のデータを入れ始める管理者の手順（初期投入） | [initial-load-runbook.md](initial-load-runbook.md) |
| 現行のER・画面遷移・業務フローと設計点検 | [図面の入口](diagrams/current-system.md)、[図面監査](research/2026-09-16-diagram-audit.md) |
| 表示・操作・日本語文言 | [SCREEN_DESIGN.md](SCREEN_DESIGN.md)、[design-system](design-system/README.md) |
| 環境構築 | [DEV_SETUP_CHECKLIST.md](DEV_SETUP_CHECKLIST.md) |
| 計画・実装・レビュー・完了手続き | [DEV_WORKFLOW.md](DEV_WORKFLOW.md)、[CI](ci.md) |
| 実装後のPR状態・review/manual記録・merge保護 | [merge evidence](agent-guidance/merge-evidence.md) とhelper status |
| 決定理由 | [decision-log.md](decision-log.md)、[ADR](adr/README.md) |

再開時は対象の現物と必要な状態を確認し、既に完了した工程を繰り返さない。未解決判断と未実施検証を引き継ぐ。

旧引き継ぎ全文は [履歴snapshot](archive/harness-context/2026-09-14-PROJECT_HANDOFF.md) に保存した。そこにあるphaseや「次にやること」は記録当時の情報で、現在の指示ではない。この案内は参照先が変わるときに更新する。

## 履歴境界（public snapshot）

- public repository の履歴境界日は、Phase B で作成する parent を持たない初期 commit の author date とする
- 境界日以前を指す PR 番号、issue 番号、commit SHA は、owner が保管する private archive の証跡として読み替える。private archive の URL や repository 識別子は公開文書へ記載しない
- 日常開発は public repository だけを持つ書込み専用 clone で行い、archive remote、旧 object、replace ref を置かない
- 境界を越えた履歴参照が必要な場合は、public repository への push-capable remote を持たない履歴閲覧専用 clone を使う。private archive から旧 main ancestry を取得した後、`git replace --graft <public-init> <archive-main-head>` をローカルで再適用する
- replace ref は clone や通常 push では共有されない。新しい履歴閲覧 clone と履歴閲覧用途の Windows 同期 clone では、役割分離を確認してから同じ手順を再適用する

移行時の規範と更新手順は [PUBLIC_REPO_MIGRATION.md](PUBLIC_REPO_MIGRATION.md)。POSの過去の確認値は [移送前記録](archive/harness-context/2026-09-14-PROJECT_HANDOFF.md) に保存し、現在のadapter事実は [project-memory.md](project-memory.md) と [実機確認](plu-export-and-real-csv-verification.md) を参照する。
