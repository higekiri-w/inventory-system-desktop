---
name: inventory-implementation
description: Implement inventory-system code or documentation changes against its approved scope and source contracts. Reviews alone use the review skills.
---

# Inventory Implementation

[AGENTS.md の Session Start](../../../AGENTS.md#session-start) で対象と現在地を確認する。R0〜R2 は packet なしの道（PR body の 5 行）。R3/R4 は対象 packet の完全な Workflow State と Scope / AC / Contract Ledger / Test Plan を読み、`Plan Commit` が SHA（計画を固定済み）であることを確認する（[Workflow State](../../../docs/DEV_WORKFLOW.md#workflow-state)。`pending` のまま実装を push すると先走りの検査が止める）。不正・曖昧・未承認の状態を自分で承認済みにしない。

## Work

- `git status --short --branch` で既存変更を確認し、必要な設計正本と対象のコード・テストを読む。UI / command / DB / workflow のうち関係する境界だけを辿る。
- 設計が不足していれば [Design Phase](../../../docs/DEV_WORKFLOW.md#design-phase-rules) へ戻る。業務規則は BIZ、CMD は薄く保つ。
- 振舞いの変更は可能な範囲で失敗再現 → 最小修正 → 対象テストの順に進め、意味のある REQ / spec ID を付ける。既存テストを通すために仕様を弱めず、テストの保護はAGENTSのWorking Rulesに従う。
- DTO / command の変更は bindings、route の変更は route tree、REQ の変更は traceability の生成義務を確認する。該当する正本を同じ変更で同期する。
- scope 内で判明した失敗を解消し、許可された完了条件まで進める。追加の仕様判断・破壊的操作・未解決 Human Gate は owner へ返す。

## Verify and hand off

検証の種類とタイミングは [Verification Gates](../../../docs/DEV_WORKFLOW.md#verification-gates) と [CI](../../../docs/ci.md) が所有する。実装中は対象テストと `local-ci.sh changed`。merge 前の feedback は push ごとの Draft の hosted の run で、手元で全 gate を回したいときだけ `local-ci.sh full`（任意）。

Verify の後に [Draft・Ready・merge](../../../docs/DEV_WORKFLOW.md#draftreadymerge) で Draft PR を開き、helper の capture の後に [Review Rules](../../../docs/DEV_WORKFLOW.md#review-rules) の Final Review（本数は helper が導く。R4 / workflow gate change は Double Audit。R2 は差分の review 1 本）を Draft で受ける。指摘は現物で確認し、成果物と検証、残る owner 確認を引き渡す。Ready / merge は明示承認に従う。

最終報告には結果、必要な検証根拠、未完了事項を含める。正本にない工程や、無関係な清掃を完了条件に増やさない。
