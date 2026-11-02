---
name: inventory-workflow-start
description: Route inventory-system development starts and resumes to the applicable workflow. Use when choosing the next development step; standalone questions and reviews use their own route.
---

# Inventory Workflow Start

[AGENTS.md の Session Start](../../../AGENTS.md#session-start) に従い、依頼に合う入口と次の行動を選ぶ。ここに読書順序・phase 表・gate を複製しない。

- 新規作業か同じ作業の再開かを識別し、[Risk Tiers](../../../docs/DEV_WORKFLOW.md#risk-tiers) で影響を分類する。無関係な active packet を選ばない。
- R0〜R2 は対象の仕様・コード・テストと該当する blocker を確認する。Plan Packet、Workflow State、Plan Commit は要求せず、PR body の 5 行（Risk の行は `Risk: R<0|1|2>（目視: 要|不要）`）で進める。
- R3/R4 の再開は、依頼が名指しする PR の helper の status と、その active な packet の完全な Workflow State を読む。`Plan Commit` が `pending` なら計画の review の前後、SHA なら実装の後（[Workflow State](../../../docs/DEV_WORKFLOW.md#workflow-state) の active な packet・計画の固定・fail-closed）。通過済みの工程を繰り返さない。
- 新しい R3/R4 は範囲と設計の充足を確認する。役割と可用性は [Agent Operating Manual](../../../docs/AGENT_OPERATING_MANUAL.md#3-role-assignment役割割当の制約とavailability)。利用不能時は同書の Capacity-degraded に従う。
- 実装前の設計不足は Design Phase で解消し、計画を固定する（`scripts/plan-approve.sh`）前は実装へ渡さない。具体的な承認待ちと、その前に完了できる作業を区別する。

## Handoff

- 実装: `inventory-implementation`。この repo では generic `implementation` を重ねて実行しない。
- operator UI: `inventory-operator-ui`。製品の表示・操作契約を優先する。
- review: `inventory-code-review`。初回監査と既存指摘の closure を区別する。

進捗は必要な現在地・次の行動・未解決判断を短く共有する。承認依頼は [承認依頼](../../../docs/DEV_WORKFLOW.md#承認依頼)、完了は [Done Definition](../../../docs/DEV_WORKFLOW.md#done-definition) に従う。固定の kickoff/final テンプレートを全依頼に出力しない。
