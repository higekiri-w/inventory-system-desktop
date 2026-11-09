# Workflow Effectiveness Review: filesystem failure observability（PR #24）

## Workflow Used

- Project Profile: R3 / hosted CI required / dual-vendor-no-fable
- Plan Packet: [packet](2026-07-24-filesystem-failure-observability.md)
- Test Design Matrix: [matrix](test-matrices/2026-07-24-filesystem-failure-observability.md)
- review-only sub-agent: skipped。owner 起動の Sonnet 5 fresh context を
  Plan Reviewer / Final Reviewer に固定した
- external review: Plan Review 一次、Final Review 一次 + 独立再実測
- human approval: Plan / Ready / merge の3接点
- gates: targeted Rust tests、X1-X7 mutation、doc consistency、traceability、
  L1 full CLEAN、[exact-HEAD hosted final](https://github.com/higekiri-w/inventory-system-public/actions/runs/30032542718)

## What Worked

- Plan Review の P2 が production / test 共用 generic helper の具体 signature を
  実装前に要求し、failure-injection test だけ通る別経路を設計段階で防いだ。
- 現 HEAD の `mnt/` sweep を監査時の旧行番号より優先したため、restore だけでなく
  backup metadata と diagnostic top-level / entry failure まで S1-S8 として分類できた。
- Test Matrix が「伝搬」と「WARN後継続」を別 oracle にし、X1-X7 を実際に注入して
  silent discard への退行を全件検出した。Final Reviewer の X3 / X6 再注入でも
  同じ防御が独立に機能した。
- D-035 の三点SHA比較と `MERGE_EVIDENCE_VALID=true` が、owner infra 変更を含む
  DIRTY tree を merge evidence に誤用させず、Ready 後の exact-HEAD hosted finalへ
  同じ候補を渡した。

## What Did Not Work

- 最初の実装後セルフ監査で D1/D3/D4/D5 の failure-injection test 不足と
  D2 oracle の弱さが判明した。`1d97550` で是正できたが、Matrix row ごとの
  implementation / test 突合を実装直後に一度で終えられていなかった。
- Final Review は source design の test 名から `req700` infix が抜けた転記ずれと、
  D4 non-Unicode test bullet の欠落を P3 として検出した。candidate safety には
  影響しないが、Plan 時点の cited-test existence 確認が source doc の exact name
  まで閉じていなかった。
- L1 CLEAN のため owner infra 3 files を手動 stash / pop する必要があり、
  product change と環境整備が同じ worktree にある運用コストが出た。

## Issues Caught Before Implementation

- Plan Review P2-1: entry failure injection の production / test 同一路を
  concrete helper signature で固定。
- Plan Review P3-1: MNT-02 retention parse fallback を filesystem Result 是正と
  混在させず、別 R2/M 相当の変更へ分離。

## Issues Caught by Tests

- backup entry error 後の余分な create / cleanup、副作用を B1 が検出。
- create / list metadata の `size_bytes=0` fallback を B2 / B3 が検出。
- diagnostic の NotFound / other IO 分離と個別 WARN 後継続を D1-D5 が検出。
- restore cleanup failure が元の `Recovered` を置換する退行を R1 が検出。
- X1-X7 の実 mutation により、構造上の説明だけでなく test の実感度を確認した。

## Issues Caught by Review-only Sub-agent

| Finding | Classification | Result |
|---|---|---|
| repo-local review-only sub-agent | skipped | owner 指定の fresh-context Final Reviewer が Contract Audit と独立再実測を担当 |

## Issues Caught by External Review

- Plan Review: P2×1 / P3×1。いずれも gated amendment `c0fd2f0` に反映。
- Final Review: P1=0 / P2=0 / P3=2、Ready 可。P3×2 は Findings Freeze 後の
  evidence-quality follow-up として post-merge closeout で source design に反映。
- Final Reviewer は S1-S8 sweep、J1-J7、helper 経路、X3 / X6 mutation、
  oracle 独立性、全 gate を独立再実測した。

## Escaped / Late Findings

- `70-mnt-diagnostic-log.md` の test 名転記ずれと D4 bullet 欠落が Final Review
  まで残った。runtime / test implementation の欠陥ではなく、source design の
  evidence-quality drift だったため merge 後に解消した。
- create metadata error 後に VACUUM output が残り、retry で backup が増える可能性は
  既知の残余 risk。今回の Goal Invariant は不正確な成功を返さないことを優先し、
  cleanup / dedup semantics の再設計は行わなかった。

## Test Adequacy

Strong tests:

- public production function と failure-injection test が同じ helper / file-ops pathを通る。
- WARN の存在だけでなく、後続 eligible entry の処理と主 error / variant 維持を同時に
  assert している。
- production 定数や helper を oracle へ import せず、literal path / kind /
  operation type を独立転記している。

Weak or missing tests:

- non-Unicode filename の実 filesystem 再現は platform 差があり、
  対応 platform の cfg-aware test と helper-level contract に留まる。
- metadata failure 後の残置 backup を次回 retry でどう扱うかは、現契約上の
  residual behavior であり本変更の test 対象外。

Mutation-style observations:

- X3 / X6 は Final Reviewer が再注入しても red となり、Writer の mutation receipt
  だけに依存しなかった。
- D2 は「後続削除だけなら green」の survivor を避けるため WARN oracle を強化した。

## Signal / Noise

- independent review findings total: 4
- accepted: 4
- rejected: 0
- deferred: Final Review P3×2をpost-merge closeoutまで延期し、今回解消
- question: 0

## Cost / Friction

- useful cost: Plan Review の same-path helper 固定、セルフ監査是正、X1-X7 mutation、
  Final Reviewer の独立 mutation / gate 再実測。
- excessive friction: owner infra 3 filesをL1直前だけ手動stashし、完了後に戻す操作。
- confusing steps: なし。state-only / PR body / hosted evidence の所有先は一致した。
- review rounds (broad audit / closure確認の内訳): Plan Review一次、
  Final Review一次 + independent remeasurement
- state-only commits / 総commit数: 3 / 10

## Recommended Workflow Adjustment

Keep:

- Plan Gate で production / test 共用 helper signature まで具体化する。
- Matrix の mutation を clean committed baseline へ実注入する。
- exact-HEAD L1 / hosted / live PR HEAD の三点比較。

Change:

- source design が test 名を列挙する場合、Final Review の cited-test existence checkで
  `rg` の実名と bullet を一対一比較する。既存 `DEV_WORKFLOW.md` の
  「Cited test existence」を適用すれば足り、新しい一般ルールは増やさない。
- product change と owner infra maintenance は commit と検証を分離する。

Follow-up:

- MNT-02 `log_retention_days` parse fallback は別 R2/M 相当の是正単位で扱う。
- backup metadata failure 後の残置 file / retry 重複が実運用上問題になる場合は、
  MNT-01 の cleanup / dedup contract を先に設計する。

## Retired / Consolidated Rules

- none。今回の late P3 は既存の cited-test existence 規律の適用不足であり、
  Markdown test-name checkerを追加すると低頻度の evidence drift に対して
  rule / maintenance costが増える。Final Review と source doc 修正へ統合する方が
  net rule growthを避けられる。

## Applied / Deferred Workflow Changes

Applied:

- Final Review P3×2を `70-mnt-diagnostic-log.md` の exact test identifiers と
  D4 bulletへ反映。
- packet / Matrix / WER / `Plans.md` / `docs/PROJECT_HANDOFF.md` を
  post-merge closeoutで同期。

Deferred:

- MNT-02 retention parse fallback → 別 R2/M 相当の是正単位。
- backup metadata failure 後の残置 file / retry contract → 実害または運用要望が
  確認された時点で MNT-01 Design Phase。

Not applied:

- Markdown 内の test-name exact-match 機械 gate。既存 review check と今回の
  post-merge correctionで十分であり、機械 enforcement の便益が未実証。
