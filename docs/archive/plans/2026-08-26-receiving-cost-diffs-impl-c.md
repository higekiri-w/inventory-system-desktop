# Plan Packet — 実装 PR C: 入庫時原価差分検出 cost_diffs（SPEC-PRV 実装 C）

## Workflow State

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 47d01a2
- Amendments: none
- Coordinator: Claude Fable 5
- Writer: Codex（GPT-5.6 系、発注書駆動）
- Plan Reviewer: Claude Sonnet 5（独立 fresh context、D-062）
- Final Reviewer: Claude Sonnet 5（独立 fresh context、worktree 隔離）
- Reviewed Content HEAD: d55f8f2
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（Plan Gate 承認 = 介入 1/3、L3 全 PASS = 介入 2/3、Ready 承認 = 介入 3/3、いずれも 2026-08-26 完了。残る owner 操作は merge のみ）

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2（発注 lane 単位で計上。同一発注内の fail-closed amendment 回答は同一 lane = PR #95 / PR #4 先例）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
Tauri command DTO（`ReceivingCreateResult` への field 追加 + 新 wire 型 `CostDiff`）と generated bindings、および operator workflow（入庫保存完了時の原価差分ダイアログ）に触れるため。DB schema / migration / CSV には触れない。

## Goal

Goal Invariant:

### 最小完了条件

- 入庫保存が成功し明細原価がマスタ原価と不一致のとき、operator に原価差分ダイアログが提示され、「マスタ原価をこの実原価に更新する」を商品単位で実行できる（新売価は現売価据え置き）。REQ-209 の T3 WARN（no-test）が解消される。

### 失敗定義

- 差分検出が入庫保存の成否に影響する（検出起因で保存済み入庫が失われる／保存が失敗する）。
- 既存 consumer が `ReceivingCreateResult` の変更で壊れる（bindings 互換破壊）。
- ダイアログの更新実行が売価を書き換える（据え置き契約違反）。

### 非目的

- CostDiff 型の field 拡張（設計正本の 4 field を凍結維持）。
- 差分見送りの記録・履歴化。
- UI-14 / UI-01b など他画面の変更。
- 入庫以外の業務記録（返品・廃棄・手動販売）への差分検出展開。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `src-tauri/src/biz/inventory_service/receiving.rs`: §12.3 step 10 の COMMIT 後原価差分検出（完全一致比較、保存 TX 非影響、replay 時は step 1 で空配列）+ `CostDiff` struct（4 field、`serde::Serialize` + `specta::Type`）+ `ReceivingCreateResult.cost_diffs` field 追加 + 予約済み Rust test 4 本。
- `src-tauri/src/biz/inventory_service/mod.rs`: `CostDiff` の re-export 追加。
- `src/lib/bindings.ts`: `cargo run --bin generate_bindings` 再生成（追加のみ diff）。
- `src/features/receiving/`: 原価差分ダイアログ component 新設（`ui/dialog.tsx` 使用）+ `ReceivingPage.tsx` の保存成功時表示接続 + 行単位更新（既存 `useReviseProductPrice` hook 流用、SPEC-PRVC-D1/D2）。
- `src/features/receiving/ReceivingPage.test.tsx`: RTL 新規 test 追加（Matrix T5〜T9）。
- 既存 test file 3 本（`ReceivingPage.test.tsx` / `ReceivingPage.suggest.test.tsx` / `ReceivingPage.unsaved-guard.test.tsx`）の `createReceiving` mock result literal への `cost_diffs: []` 追従（凍結例外 class、SPEC-PRVC-D4）。
- `docs/function-design/90-traceability.md`: `cargo run --bin generate_traceability` 再生成（REQ-209 T3 WARN 解消）。

## Non-scope

- source design docs の改訂（正本 3 doc は PR #93 で確定済み・本 PR では無改変。61-ui §61.8 の既存 Non-scope 群も維持）。
- `docs/decision-log.md` D-052 の改訂（SPEC-PRVC-D1 の裁定理由を参照）。
- DB schema / migration / CSV / PLU / 帳票。
- `src/lib/invalidation-contract.ts` の変更（entry 追加なし）。
- UI-14 / UI-01a / UI-01b / UI-15 の変更。
- 入庫 route / navigation の変更（既存 active のまま）。

## Acceptance Criteria

- AC-1: `cd src-tauri && cargo test req209` で予約 4 test（`test_create_receiving_req209_detects_cost_diff` / `test_create_receiving_req209_no_diff_when_cost_matches` / `test_create_receiving_req209_empty_on_idempotent_replay` / `test_create_receiving_req209_cost_diff_detection_does_not_affect_save_tx`）が存在し PASS。
- AC-2: `cd src-tauri && cargo run --bin generate_bindings` 後の `git diff src/lib/bindings.ts` が追加のみ（削除行に既存の型定義・field 行を含まない。`CostDiff` 型 + `cost_diffs` field の追加に限る）。
- AC-3: `cd src-tauri && cargo run --bin generate_traceability -- --check` の出力に REQ-209 の T3 WARN が出現しない（実行前 2026-08-26 実測では `90-traceability.md` L39 が `no-test`）。
- AC-4: `npm test -- ReceivingPage` PASS（Matrix T5〜T9 の新規 test + 既存 test 全 green）。
- AC-5: 既存 test file の変更が凍結例外 class（`createReceiving` mock result literal への `cost_diffs: []` field 追加）に限られることを `git diff` で確認。対象は 3 file 全数（`rg -l "createReceiving" src/features/receiving --glob '*.test.tsx'` 実測 3 hit、2026-08-26）。assertion・既存 test 名・既存期待値の改変は 0。
- AC-6: `bash scripts/local-ci.sh changed` PASS（Writer 実行、evidence は file 直書き + RESULT 行）。
- AC-7: D-052 静的回帰 test（success-path 直接 `invalidateQueries` 拒否）が green 維持（`invalidation-contract` 契約 test 無改変で PASS）。
- AC-8: `cargo check --release` PASS（Human Gate に L3 を含むための Writer 完了条件、CI gate ではない）。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-209（coverage=required）
- Architecture: `docs/ARCHITECTURE.md`（UI → CMD → BIZ → IO、変更なし・準拠のみ）
- Function / command / DTO: `docs/function-design/31-biz-inventory-service.md` §12.3（ReceivingCreateResult / CostDiff / 処理ステップ 10）、`docs/function-design/44-cmd-inventory.md`（`create_receiving` 出力型、cost_diffs 追加のみ契約）
- DB: 変更なし（読取りのみ: `products.cost_price` / `products.name` / `products.selling_price`）
- Screen / UI: `docs/function-design/61-ui-receiving.md` §61.4 wire / §61.5 D15 / §61.6 原価差分更新失敗 / §61.9 UI-02-D15
- Decision log / ADR: D-052（C4 = 入庫保存 invalidation 維持、C20 = `revise_product_price` mutation entry 流用）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 31-biz §12.3 step 10 + CostDiff 構造体 | existing sufficient（PR #93 で正本化済み） |
| Command / DTO / generated binding / wire shape | 44-cmd `ReceivingCreateResult.cost_diffs` + 本 packet Boundary / Wire Contract | existing sufficient |
| DB / transaction / audit / rollback / migration | 変更なし（COMMIT 後の読取りのみ） | existing sufficient |
| Screen / UI / route state / Japanese wording | 61-ui §61.5 D15 / §61.6 / §61.9 UI-02-D15 | existing sufficient |
| CSV / TSV / report / import / export format | 該当なし | 該当なし |
| Durable decision / ADR | D-052（無改変流用、SPEC-PRVC-D1 裁定） | existing sufficient |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| 新規 Tauri command | 該当なし（`create_receiving` の出力型変更のみ、command 追加なし） |
| function-design doc 新設 | 該当なし |
| route 新設 / operator 画面新設 | 該当なし（既存 UI-02 画面への dialog 追加） |
| wire 型追加（`CostDiff`） | `serde::Serialize` + `specta::Type` derive + `cargo run --bin generate_bindings` で `bindings.ts` 再生成（AC-2） |
| REQ-209 への test 付与 | `cargo run --bin generate_traceability` で `90-traceability.md` 再生成（AC-3、AUTO-GENERATED・手動編集禁止のまま） |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-209 | 31-biz §12.3 step 10 | SPEC-PRV-D8 | COMMIT 後検出で保存成否と分離 | `receiving.rs` step 10 | Matrix T1〜T4 |
| REQ-209 | 44-cmd 出力型 | SPEC-PRV-D8 | field 追加のみで既存 consumer 互換 | struct + bindings | AC-2 / T5〜T6 |
| REQ-209 | 61-ui §61.5 D15 / §61.9 UI-02-D15 | UI-02-D15 | ダイアログ提示 + 商品単位更新 + 据え置き | dialog component + ReceivingPage | Matrix T5〜T9 |
| REQ-209 | 61-ui §61.6 原価差分更新失敗 | UI-02-D15 | 行単位失敗 + 再試行、保存成功表示維持 | dialog component | Matrix T8 |
| REQ-209 | D-052 C20 | SPEC-PRVC-D1 | mutation 単位契約の流用、新 entry 不採用 | `useReviseProductPrice` 流用 | AC-7 + T7（hook 経由呼出し） |

## packet-local 決定（SPEC-PRVC-D1〜D4）

- **SPEC-PRVC-D1（invalidation 裁定）**: Plans.md が委ねた「C23 以降 or 既存 C4 流用」は**「新規 C entry を追加しない」**で確定する。根拠: ①入庫保存自体の invalidation は既存 C4（実装済み・無改変）②ダイアログの更新実行は mutation `revise_product_price` であり、D-052 の entry は mutation 単位契約 — C20 `productPriceRevise` が既に存在し、既存 `useReviseProductPrice` hook（C20 適用済み、画面非依存を 2026-08-26 実読確認）を流用する ③ `invalidateByContract(` の call site は hook 内共有のため増えず（`rg -c` 実測 25 のまま）、D-052 Decision 行の実測数値・Revisit 条件（書込み table.column / 読取り列 / E1〜E6 の変化）のいずれにも該当しない。よって D-052 正本・`invalidation-contract.ts`・契約 test はすべて無改変。連番 registry の採番衝突も構造的に発生しない。
- **SPEC-PRVC-D2（現売価据え置きの実装契約）**: CostDiff は設計正本どおり 4 field 凍結のため現売価を含まない。「マスタ原価をこの実原価に更新する」実行時に `commands.getProduct(product_code)`（binding 実在を 2026-08-26 実測）で現在の `selling_price` を読み、`PriceRevisionInput { product_code, new_selling_price: 現売価, new_cost_price: received_cost_price, assign_supplier_id: null }` を送る。`assign_supplier_id: null` は「紐付け変更なし」（`product_service.rs` L399-448 実読: `supplier_assigned = existing.supplier_id.is_none() && input.assign_supplier_id.is_some()`）。`getProduct` 失敗は 61-ui §61.6「原価差分更新失敗」に包含し、該当行のみ失敗表示 + 再試行可能。
- **SPEC-PRVC-D3（component 配置）**: ダイアログは `src/features/receiving/` 配下の新規 component file とし、既存 `src/components/ui/dialog.tsx`（実在を 2026-08-26 実測）を使用。状態表示は色だけに依存させない（61-ui D15）。
- **SPEC-PRVC-D4（既存 test 凍結の例外 class）**: `ReceivingCreateResult` への field 追加は bindings 型変更として既存 frontend test の `createReceiving` mock result literal に `cost_diffs: []` の追従を要する。class 全数 sweep（rg 実測 2026-08-26）: 対象は `src/features/receiving/` の test 3 file 全数のみ。Rust 側は `ReceivingCreateResult` 構築 literal が production `receiving.rs` 3 箇所のみで、既存 Rust test に構築 literal はなく（`invariants.rs` は `size_of` の型参照のみ）追従不要。例外は field 追加に限り、assertion・test 名・既存期待値の改変は凍結どおり禁止。

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（31-biz §12.3 / 44-cmd / 61-ui D15 が PR #93 で正本化済み）。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: なし（SPEC-PRVC-D1〜D4 はいずれも既存正本の適用裁定・実装配置であり durable design ではない。D15 の実装詳細として archive に残る）。
- Assumptions and constraints: `revise_product_price` / `getProduct` / `ui/dialog.tsx` / `useReviseProductPrice` の実在（Contract Probe で実測済み）。
- Deferred design gaps, risk, and follow-up target: 見送り記録なし（設計どおり）。差分検出の対象は入庫のみ（61-ui §61.8 既存 Non-scope 維持）。
- Test Design Matrix can cite design decision IDs or source doc sections: yes（各行に SPEC-PRV-D8 / UI-02-D15 を付す）。
- Absolute guarantee / escape hatch self-check completed: 「差分検出は保存 transaction の成否に影響しない」の絶対保証は、検出読取り失敗時に warn log + 空配列で返す実装（エラー握りつぶし禁止 rule に従い `tracing::warn!` 必須）とし、T4 で保存成果物の完全性を検証する。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable（外部 adapter に非接触） | — |
| Fact check / design decision split | applicable — Contract Probe（本 packet 8 項目）で観測事実（binding / component / hook 実在、意味論、凍結例外 class 全数、採番状況）を確認済み。裁定（SPEC-PRVC-D1〜D4）は事実と分離して記録 | Contract Probe 節 / packet-local 決定節 |
| Lifecycle / retry | applicable: 更新失敗の行単位再試行 / 見送り後の次回入庫再提示（検出は毎回の保存で実行、記録なし） | Matrix T8 / T1 |
| Operator workflow | applicable: 入庫保存 → 差分確認 → 更新 or 見送り → result panel 継続の実 flow | Matrix T5〜T9 + L3 |
| Replacement path | not applicable | — |
| Data safety / evidence | synthetic fixture のみ使用（実店舗データ非使用） | Data Safety |
| Reporting / accounting semantics | not applicable（集計・帳票に非接触。products.cost_price の更新意味論は既存 BIZ-01 契約のまま） | — |
| Manual verification | ダイアログの native 表示・日本語文言・focus は L3 | Test Plan L3 |
| 環境・再現性 | not applicable（toolchain / runner 変更なし） | — |

## Design Readiness

- Existing design docs are sufficient because: 31-biz §12.3 step 10（検出契約・CostDiff 4 field・replay 空配列・保存非影響）、44-cmd（field 追加のみ・consumer 互換）、61-ui（D15 ダイアログ・§61.6 失敗時挙動・§61.9 test focus）が PR #93 で正本化済みで、実装に必要な行動契約がすべて特定済み。spec-check → plan-draft の設計 skip（Design Readiness 十分性引用）を適用する。
- Source docs updated in this PR: なし。
- Design gaps intentionally deferred: なし。
- Durable decisions discovered in this plan and promoted to source docs: なし。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): 検出は BIZ（step 10）、CMD は passthrough のまま、UI は表示と `reviseProductPrice` 呼出しのみ。
- Backend function design: 31-biz §12.3 step 10 に準拠（COMMIT 後・完全一致・非影響・replay 空）。
- Command / DTO / data contract: field 追加のみ、Boundary / Wire Contract 参照。
- Persistence / transaction / audit impact: なし（検出は読取りのみ、更新は既存 `revise_product_price` の 3 テーブル TX に委譲）。
- Operator workflow / Japanese UI wording: 61-ui D15 の文言「マスタ原価をこの実原価に更新する」を exact 使用。
- Error, empty, retry, and recovery behavior: 61-ui §61.6（行単位失敗・再試行・保存成功表示維持・見送り無記録）。
- Testability and traceability IDs: REQ-209 を全 test に付与、90-traceability 再生成。

## Contract Probe

（いずれも 2026-08-26 に本 repo で実測済み。是正仮適用を要する登録漏れなし）

- `commands.reviseProductPrice` binding 実在: `rg -n "reviseProductPrice" src/lib/bindings.ts` → L12 hit。
- `commands.getProduct` binding 実在: `rg -n "getProduct" src/lib/bindings.ts` → L42 hit（`ProductWithRelations` 返却）。
- 参照 component 実在（PR #95 WER 教訓）: `fd dialog src/components/ui` → `dialog.tsx` / `alert-dialog.tsx` 実在。
- `useReviseProductPrice` の C20 適用と画面非依存: 実読（onSuccess/onError callback 注入可能な汎用 hook、`invalidationContract.productPriceRevise(input.product_code)` 適用済み）。
- `assign_supplier_id: null` の意味論: `product_service.rs` L399-448 実読（null = 紐付け変更なし）。
- 既存 test 凍結例外 class の全数: Rust 構築 literal = production 3 箇所のみ / frontend mock 追従対象 = test 3 file（SPEC-PRVC-D4 の実測）。
- D-052 採番状況: decision-log 実測で C22 まで使用済み → 本 packet は採番なし（SPEC-PRVC-D1）。
- REQ-209 現状: `90-traceability.md` L39 = `no-test`（T3 WARN 対象）実測。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| 31-biz step 10: COMMIT 後検出・完全一致比較（±1 円で差分） | `receiving.rs` step 10 | T1 / T2 | — |
| 31-biz step 10: 検出は保存成否に非影響（検出失敗で rollback しない、warn log） | `receiving.rs` step 10 | T4（保存成果物完全性） | 検出読取り失敗の実注入は非現実的 → warn log 実装は review 確認 |
| 31-biz step 1: idempotent replay は `cost_diffs: []` | `receiving.rs` step 1 分岐 | T3 | — |
| 31-biz CostDiff 4 field（product_code / product_name / master_cost_price / received_cost_price） | struct 定義 + bindings | T1（field 値 exact）+ AC-2 | — |
| 44-cmd: field 追加のみ・既存 consumer 互換・新 consumer は保存成功時のみ読む | cmd passthrough + bindings | AC-2 + T6（replay 非表示） | — |
| 61-ui D15: 保存成功 + 非空でダイアログ表示、空 / replay で非表示 | ReceivingPage 接続 | T5 / T6 | L3-1 目視 |
| 61-ui D15: 行 = 商品名・商品コード・マスタ原価・実原価、日本語ラベル、色非依存 | dialog component | T5 | L3-1 目視 |
| 61-ui D15: 「マスタ原価をこの実原価に更新する」→ `reviseProductPrice`、新売価は現売価据え置き | SPEC-PRVC-D2 | T7（call args exact） | L3-2 |
| 61-ui §61.6: 行単位成功 / 失敗判別、失敗行のみ再試行、入庫保存成功表示は維持 | dialog component | T7 / T8 | L3-3 |
| 61-ui D15: 見送りは無記録で閉じ、次回入庫で差分が残れば再提示 | dialog component | T9 + T1（再検出は毎保存で実行） | L3-4 |
| 61-ui §61.7: 入庫保存の invalidation は C4 SSOT のまま無改変 | 無改変 | 既存 UI-02-D12 test 凍結 green | — |
| D-052 C20 流用・新 entry なし・直接 `invalidateQueries` 禁止 | `useReviseProductPrice` 流用 | AC-7 + T7 | — |
| REQ-209 traceability（T3 WARN 解消） | 90-traceability 再生成 | AC-3 | — |

隣接契約 sweep（触れる source-doc 節の非対象契約）: 31-biz §12.3 step 1〜9（冪等・validation・fingerprint・TX・operation_log）と 61-ui §61.5 の既存 D1〜D14 は無改変・既存 test 凍結で防御。61-ui D10 の result panel / スクロールは dialog と併存（dialog は result panel を置換しない）— Review Focus に含める。

## Test Plan

Test Design Matrix: [test-matrices/2026-08-26-receiving-cost-diffs-impl-c.md](test-matrices/2026-08-26-receiving-cost-diffs-impl-c.md)

- targeted tests: Rust 予約 4 本（Matrix T1〜T4）+ RTL 5 本（T5〜T9）。
- negative tests: T2（一致で空）/ T3（replay で空）/ T6（空・replay で非表示）/ T9（見送り無記録）。
- compatibility checks: AC-2（bindings 追加のみ diff）+ AC-5（既存 test 凍結例外 class 限定）+ AC-7（D-052 静的回帰）。
- data safety checks: synthetic fixture のみ（Data Safety 参照）。
- main wiring/integration checks: `bash scripts/local-ci.sh changed`（AC-6）→ L1 full は implementing → local-verified 遷移時。
- Human Gate L3（Windows native、L3 Eligibility 3 条件充足 = native 目視のみ・新規 tool なし・fault injection なし）:
  - L3-0 準備: バックアップ画面で実施前 backup を作成 → synthetic 商品 1 件登録（原価 500 円）。
  - L3-1: 入庫画面で該当商品を実原価 501 円で保存 → 差分ダイアログが表示され、商品名・商品コード・マスタ原価 500・実原価 501 が日本語ラベルで読める。
  - L3-2: 「マスタ原価をこの実原価に更新する」実行 → 成功表示。商品編集画面で原価 501 円・売価が元の値のままを確認。
  - L3-3: 同商品を実原価 502 円で再入庫 → ダイアログ再提示（見送り再提示の実確認は L3-4 と併合可）。
  - L3-4: ダイアログを見送りで閉じる → 入庫の保存結果表示は維持される。
  - L3-5 後始末: backup restore で実施前状態へ復元（または synthetic 商品の残置を owner 判断で許容）。
  - Writer 完了条件: `cargo check --release` PASS（AC-8）。

## Boundary / Wire Contract

- producer: `create_receiving` command（CMD passthrough、BIZ 所有 DTO）。
- consumer: 既存 = `ReceivingPage.tsx`（`cost_diffs` を新規に読む）。他の `createReceiving` consumer なし（rg 実測: production では ReceivingPage のみ）。既存 consumer 互換 = field 追加のみで担保。
- wire type: `ReceivingCreateResult` に `cost_diffs: CostDiff[]` 追加。`CostDiff = { product_code: string, product_name: string, master_cost_price: number, received_cost_price: number }`。
- internal type: Rust `Vec<CostDiff>`、`i64` 価格（円整数）。
- precision/range: 円整数（i64 ↔ TS number）。完全一致比較のため丸めなし。
- round-trip path: Rust struct → specta → `bindings.ts` → ReceivingPage。逆方向なし（出力専用）。
- invalid input: なし（出力 field 追加のみ。入力契約 `ReceivingCreateRequest` は無改変）。
- compatibility: 既存 field の名称・意味・順序に依存する契約は不変（44-cmd）。idempotent replay は常に空配列。

## Review Focus

- 検出 code が COMMIT 後（TX 外）に置かれているか、検出失敗が保存結果に伝搬しないか（warn log の実装品質 rule 準拠込み）。
- T7 の call args が独立転記 oracle と完全一致か（swap / 据え置き破り mutant への感度）。
- 既存 test 変更が SPEC-PRVC-D4 の class に限定されているか（AC-5）。
- dialog と result panel の併存（D10 スクロール挙動を壊さない）。
- 90-traceability 再生成が同 commit に含まれるか（R1 direct impl の drift 教訓）。

## Spec Contract

Contract ID: SPEC-PRVC

- SPEC-PRVC-C1: 入庫保存成功時、各明細の `cost_price` と `products.cost_price` の完全一致比較で不一致のみを `cost_diffs` に積む（±1 円も差分。検出は COMMIT 後、保存成否に非影響）。
- SPEC-PRVC-C2: idempotent replay の応答は `cost_diffs: []` 固定で、ダイアログを提示しない。
- SPEC-PRVC-C3: ダイアログの行更新は `revise_product_price` を現売価据え置き・入庫実原価で呼び、`assign_supplier_id: null`（紐付け変更なし）を送る。invalidation は C20 契約を hook 経由で適用する。
- SPEC-PRVC-C4: 見送りは無記録で、次回入庫時に差分が残っていれば再提示される。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| REQ-209 / SPEC-PRVC-C1 | BIZ step 10 実装 | T1 / T2 / T4 | TX 外配置・非影響 | cargo test 出力 |
| REQ-209 / SPEC-PRVC-C2 | replay 分岐 field 追加 | T3 / T6 | replay 経路に検出なし | cargo test + RTL 出力 |
| REQ-209 / SPEC-PRVC-C3 | dialog + hook 流用 | T7 / T8 / AC-7 | args exact・C20 経由 | RTL 出力 |
| REQ-209 / SPEC-PRVC-C4 | dialog close 挙動 | T9 | 無記録の確認 | RTL 出力 |
| REQ-209 | traceability 再生成 | AC-3 | 同 commit 内包 | generate_traceability 出力 |

## Data Safety

- 実店舗の商品・原価・POS データは commit しない（fixture は synthetic 値のみ: 例 原価 500 / 実原価 499・501 / 売価 1200）。
- local-only paths: `.local/ci-evidence/`（L1 evidence、commit 対象外）。
- synthetic-only paths: Rust test fixture / RTL mock / L3 手順の商品登録値。
- DB schema / migration 変更なし。L3 は backup → restore で原状回復可能。

## Implementation Results

（exact-HEAD SHA / test 件数は PR body 側に置く — D-035/D-038）

- Codex Writer が発注 1 本・実装 1 commit で完走（fail-closed 停止なし）。BIZ step 10 + CostDiff + ダイアログ component 新設 + ReceivingPage 接続 + 予約 test 全数 + bindings / 90-traceability 再生成。凍結例外 class の実需要は 2 file（`ReceivingPage.suggest.test.tsx` は `createReceiving` result literal 非保持のため追従不要 — Final Review が実読確認）。
- 記録是正（Final Review P2-1）: idempotent replay の result panel 文言「同じ内容の再送として処理済み」は 61-ui §61.6 の設計契約（2026-07-13 起源）が旧実装では Badge「再送結果」のみで未実装だったものを、本 commit で初実装した。Ledger / Matrix T6 の「既存の replay 表示が維持される」は「設計契約どおりの replay 表示が提示される（本 PR で契約充足）」と読み替える。設計契約自体は 61-ui 正本どおりで逸脱なし。
- Draft PR: https://github.com/higekiri-w/inventory-system-desktop/pull/5

## Review Response

- Findings Freeze: frozen after Broad Audit（Final Review Contract Audit 1 pass、2026-08-26）; post-freeze exceptions: none.
- Final Review（独立 Sonnet、worktree 隔離）: P1 0 / P2 1 / P3 1。AC-1〜AC-8 全再実測 PASS、mutation M1〜M7 独立注入で全件 red → revert 後全緑、oracle 独立性・既存 test 凍結・実装品質 rule 適合を確認。
- P2-1（replay 文言の監査証跡不正確）: 採用。Coordinator が旧 tree（e96c7b0 時点で exact 文言 0 hit / Badge のみ）と 61-ui §61.6 を独立再確認し、Implementation Results の記録是正で解消（実装は設計契約適合のため code 変更なし）。
- P3-1（ダイアログ行更新の単一 hook 共有による直列化）: 修正不要で確定。ダイアログ本文「更新する商品を1件ずつ確認してください」（`rg -F "1件ずつ" src/features/receiving/CostDiffDialog.tsx` → L79 hit、2026-08-26 実測）の設計意図どおりの直列処理であり contract violation ではない。記載のみ。

## 遷移・レビュー記録（append-only）

- 2026-08-26: kickoff → spec-check（task scoped: Plans.md「次の行動」実装 PR C entry、Risk R3 判定・記録）→ plan-draft（唯一許可の skip: Design Readiness が既存正本 3 doc の十分性を引用）→ plan-gate（packet + Test Design Matrix を plan-first commit として本 commit で同時 commit）。本 commit がこの隣接 3 遷移を materialize する（recording compression、各遷移の evidence は本 packet の該当節）。
- 2026-08-26 Plan Review round 1（独立 Sonnet fresh context）: P1 0 / P2 2 / P3 1。P2-1 = Matrix T7 の Would-fail-if に oracle 範囲外の直接 invalidate mutant が混在（AC-7 防御へ一本化）、P2-2 = Fact check lens の not applicable 表記が Contract Probe 実施と矛盾（applicable へ是正）、P3-1 = T2 の Would-fail-if に diff=0 入力で検出不能な tolerance 記述（T1/M3 担当へ是正）。3 件とも修正案を採用し plan-gate に留まったまま是正、round 2 は closure 確認。
- 2026-08-26 Plan Review round 2（独立 Sonnet fresh context、closure 確認）: 是正 3 件すべて適用確認 OK（AC-7 の防御先 = `invalidation-contract.static.test.ts` の直接 invalidate 拒否 static test の実在まで確認済み）、新規 findings なし。P1 0 / P2 0 / P3 0 で rally 収束（round 実績 2 / 天井 3）。owner Plan Gate 承認待ち。
- 2026-08-26 owner Plan Gate 承認（介入 1/3）。本 state-only commit で plan-gate → plan-approved → implementing の隣接 2 遷移を materialize（evidence: 独立 Plan Reviewer P1/P2 = 0 = round 2 記録、plan-first commit `47d01a2` は全実装 commit に先行、owner 承認 2026-08-26）。`Plan Commit` field を `47d01a2` で確定。以後の実装は Codex 発注（cwd = 本 repo、branch `agent/receiving-cost-diffs`）。
- 2026-08-26 Codex Writer 実装完了（content candidate = `d55f8f2`、Draft PR #5）。本 state-only commit で implementing → local-verified → independent-review → human-confirm の隣接 3 遷移を materialize。evidence: ① local-verified = L1 `local-ci.sh full` CLEAN（candidate `d55f8f2`、evidence log 末尾 `RESULT=PASS` / `MERGE_EVIDENCE_VALID=true` を Coordinator 実読、PR body に記録）② independent-review = 独立 Sonnet Final Reviewer が worktree 隔離で Contract Audit 実施（Review Response 節）③ human-confirm = findings 裁定完了 P1/P2 = 0（P2-1 は記録是正で解消・code 変更なし、P3-1 は修正不要確定）、`Reviewed Content HEAD` = `d55f8f2` 設定。
- 2026-08-26 owner Windows native L3 実施: L3-0〜L3-5 全件 PASS（差分ダイアログ表示・日本語ラベル・原価更新と売価据え置き・再提示・見送り時の保存表示維持・backup restore による原状復元まで確認）。UX 所見 3 件は非阻害 P3 として PR body に記録（介入 2/3）。owner Ready 承認 2026-08-26（介入 3/3、明示承認。Ready 遷移実行は Coordinator へ委任）。本 state-only commit で human-confirm → ready-hosted-final を materialize。以後: 本 commit を含む exact HEAD で L1 full → PR body 鮮度更新 → Ready 化 → hosted CI 三点一致確認 → owner merge。
- 2026-08-26 merge → archive: L1 full PASS（Ready exact HEAD）+ hosted run success で三点一致成立（SHA / URL は PR #5 body が正本）→ owner merge（merge commit、branch commits 保存）→ Post-Merge Closeout で packet / Matrix を archive へ移動、Plans.md / PROJECT_HANDOFF 同期、L3 UX 所見 3 件を backlog 起票。
