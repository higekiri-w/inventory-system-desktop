# Plan Packet: 取引先管理 実装 PR — UI-15 `/settings/suppliers`（一覧 + usage 件数 + 追加 + インライン改名 + 統合 dialog 2 段階）+ migration v6 + `rename_supplier` / `merge_suppliers` / `list_suppliers_with_usage` + D-052 C21/C22（SPEC-SUP 実装、design 正本 = PR #3）

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

If a state-only commit materializes multiple phases, list the complete adjacent forward sequence and the pre-existing evidence for every intermediate transition in an append-only review/evidence record. Recording compression never permits a gate skip.

- Phase: archive
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 0a2f278
- Amendments: 102bce7 c26052a d888e9c a41fc93
- Coordinator: Fable
- Writer: Codex
- Plan Reviewer: Sonnet
- Final Reviewer: Sonnet
- Reviewed Content HEAD: f8dbd67
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: Plan Gate 承認済み（2026-08-25、介入 1/3）→ Windows native L3 4 項目 PASS（2026-08-26、介入 2/3。synthetic fixture 投入 → 確認 → baseline 復元まで完了、実測記録は PR #4 body 正本。UX 所感 2 件は post-freeze P3 として backlog 起票）→ Ready 承認済み（2026-08-26、介入 3/3）→ hosted final（Rust / TS / bindings を含む non-doc change のため CI-TRIGGER-D1 の Ready / `synchronize` 経路で自動 run）→ 三点一致 → merge

## Owner Effort Budget

- 介入回数上限: 3（Plan Gate 承認 + L3 PASS/FAIL + Ready 承認）
- 実働時間上限: 30分
- relay 往復上限: 2（第 1 発注 = 実装本体、第 2 発注 = Final Review 是正 delta がある場合のみ）
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

§5.5を使わないchangeは両方`none`のままにする。

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
(a) migration v6 で DB schema を変更する（`suppliers.updated_at` 追加 + backfill）、(b) Tauri command 3 件（`rename_supplier` / `merge_suppliers` / `list_suppliers_with_usage`）と BIZ 所有 DTO 2 種を新設し `bindings.ts` を再生成する、(c) operator 画面 UI-15 を新設し route / navigation を追加する、(d) `merge_suppliers` は products / receiving_records の一括付替え + suppliers 行 DELETE という不可逆な operator workflow の入口になる、(e) D-052 invalidation registry に C21 / C22 を実体化する。DB / command wire / UI route / operator workflow のいずれも R3 条件（`docs/DEV_WORKFLOW.md` Risk Tiers）に該当する。

## Goal

Goal Invariant: 店主がサイドバー「システム管理」の「取引先管理」から UI-15（`/settings/suppliers`）に到達し、取引先を name 昇順一覧（関連商品数 / 入庫記録数付き）で見渡し、追加・インライン改名・2 段階確認付き統合（products + receiving_records の付替え → source DELETE の 1 TX）を安全に行える画面を、PR #3 で正本化済みの設計契約（SPEC-SUP-D1〜D10 / REQ-106 / REQ-107）どおりに main に入れる。

### 最小完了条件

- サイドバー「システム管理」末尾に「取引先管理」が表示され、`/settings/suppliers` に到達できる（REQ-107 到達テスト PASS）。
- 一覧が name 昇順で全件表示され、各行に `関連商品数` / `入庫記録数` が `N件`（0 件も `0件`）で出る。
- `新しい取引先を追加` で追加でき、行の `名前を変更` でインライン改名（trim / 空文字 field error / 同名衝突は統合案内文言 / 同値 no-op 成功）でき、改名成功時に BIZ が `updated_at` と operation_log `supplier_rename` を更新する。
- 行の `統合` から「残す側の選択 → 影響件数 + 不可逆性の最終確認」の 2 段階を経て統合でき、1 TX で products / receiving_records が残す側へ付け替わり source 行が消え、operation_log `supplier_merge` が残り、完了通知の 2 件数が `SupplierMergeResult` と一致する。
- 統合・改名の成功後、UI-01b / UI-14 / 受入系画面の取引先表示が stale にならない（D-052 C21 / C22）。

### 失敗定義

- merge の付替えが products のみで receiving_records が欠落する（DELETE 時 FK 違反、SPEC-SUP-D4 の破れ。Matrix の mutation 予約が防御）。
- 統合の 2 段階確認・影響件数文言・不可逆文言のいずれかが省略される（SPEC-SUP-D6 の破れ）。
- migration v6 の backfill が欠け既存行の `updated_at` が NULL のまま、または再実行で v6 が重複適用される。
- 既存 `list_suppliers` / `create_supplier` の wire が変わる、`bindings.ts` の diff が新規 3 command + 2 DTO 追加以外を含む（SPEC-SUP-D8 の破れ）。
- C21 / C22 の invalidate 集合が書込み列の consumer を取りこぼす、または consumer ゼロの key を含む（D-052 導出規則違反）。
- 登録・生成義務（`collect_commands!` / `generate_handler!` / bindings / route 生成 / navigation entry + 到達テスト / 90-traceability 再生成 / REQ-107 昇格）の漏れが L1 以降で顕在化する。

### 非目的

- 取引先の単独削除・問屋チャネル・約 80 社の事前一括投入（78 doc §78.12 Deferred 維持）。
- 検索・任意並び替え・paging・bulk rename・統合の自動 undo（同上）。
- UI-01a 商品検索への取引先 filter 露出（owner 裁定 2026-08-25 で backlog、Plans.md 起票済み）。
- 既存 `CreateSupplierDialog`（features/products）/ `ProductForm` インライン追加 UI の挙動変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- migration `src-tauri/src/db/schema_v6.rs` 新設（SPEC-SUP-D5 / 22-mnt §14 / SPEC-SUPI-D1）: `pub(crate) fn apply_v6_supplier_updated_at(conn: &Connection, version: i64)` を v5（`apply_v5_plu_slots`）と同じ自己完結 pattern で実装 — BEGIN → `ALTER TABLE suppliers ADD COLUMN updated_at TEXT`（NULLABLE）→ `UPDATE suppliers SET updated_at = created_at` で backfill → 同一 TX 内で既存行の `updated_at = created_at` 一致を検証 SELECT で確認 → schema_versions に v6 を記録 → COMMIT（失敗時 rollback）。`db/mod.rs` に `mod schema_v6` 宣言、`migration.rs` `migrations()` に v6 entry（description `suppliers.updated_at 追加`、kind `MigrationKind::Custom(schema_v6::apply_v6_supplier_updated_at)`）を追加。
- docs `docs/function-design/22-mnt-migration.md` §14: MIGRATIONS 登録順段落の kind 表記 `MigrationKind::Sql(schema_v6::get_v6_supplier_updated_at_schema())` を `MigrationKind::Custom(schema_v6::apply_v6_supplier_updated_at)` へ 1 箇所是正（SPEC-SUPI-D1。処理ステップ 1〜4 と §14 見出し・登録順 v1→…→v6 は無改変）。
- IO `src-tauri/src/db/product_repo.rs`（20-io §rename_supplier / §merge_suppliers / §list_suppliers_with_usage を正本として実装）: `rename_supplier(conn, supplier_id, name, updated_at) -> Result<Supplier, DbError>`（UPDATE 0 件は not-found 相当）/ `merge_suppliers(conn, source_id, target_id) -> Result<(i64, i64), DbError>`（products UPDATE → receiving_records UPDATE → source DELETE の順、IO 自身は commit しない）/ `list_suppliers_with_usage(conn) -> Result<Vec<SupplierUsageRow>, DbError>`（`SupplierUsageRow` は IO 内部型。2 参照表の同時 JOIN による件数水増しを避け、独立集約または相関 subquery。`ORDER BY suppliers.name ASC` 全件）。既存 `list_suppliers` / `find_or_create_supplier` / `find_supplier_by_id` は無改変。
- BIZ `src-tauri/src/biz/product_service.rs`（30-biz §4.7.4 / §4.7.5 / §4.7.6 を正本として実装）: `rename_supplier`（trim / 空文字 `ValidationFailed` / 不存在 `NotFound` / 他行と同名 `ValidationFailed` / 同値 no-op は `updated_at` と operation_log を更新せず成功返却 / 実改名時は `chrono::Local` `%Y-%m-%dT%H:%M:%S` の現在時刻で IO を呼び `insert_operation_log(operation_type = "supplier_rename")` を同一 TX 内で記録）/ `merge_suppliers`（`source_id == target_id` は `ValidationFailed`、source / target 不存在は `NotFound`、`conn.transaction()` の 1 TX 内で IO `merge_suppliers` + `insert_operation_log(operation_type = "supplier_merge")`〈source/target の ID・名称 + 付替え 2 件数〉、成功時 commit / 失敗時全 rollback。統合後の再実行は source 不在の `NotFound` で黙って成功しない）/ `list_suppliers_with_usage`（IO 結果を wire DTO へ変換）。BIZ 所有 DTO 2 種を `product_service.rs` に新設（SPEC-PRVA-D5 準拠、`PriceRevisionResult` と同じ `#[derive(Debug, serde::Serialize, specta::Type)]` 様式）: `SupplierMergeResult { products_updated: i64, receiving_records_updated: i64 }` / `SupplierWithUsage { id: i64, name: String, product_count: i64, receiving_record_count: i64 }`。
- CMD `src-tauri/src/cmd/product_cmd.rs`: `rename_supplier(state, supplier_id, name)` / `merge_suppliers(state, source_id, target_id)` / `list_suppliers_with_usage(state)` を既存 `create_supplier` と同じ 3 行 pattern（DB lock → BIZ → `map_err(CmdError::from)`）で追加。DTO は qualified path 参照（CMD 側に再定義しない）。`src-tauri/src/lib.rs` の `collect_commands!` と `generate_handler!` に 3 command を登録。error kind は既存 `From<BizError>` 変換のまま（validation / not_found / duplicate。40-cmd の割当と一致、新規変換なし）。
- 生成物: `cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成し commit（diff = 新規 3 command + `SupplierMergeResult` / `SupplierWithUsage` の追加のみ、既存 export 不変 = SPEC-SUP-D8 / SPEC-SUPI-D7）。`cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成。`npm run generate:routes` は `routeTree.gen.ts`（gitignore）再生成のみで commit 対象なし。
- route `src/routes/settings/suppliers.tsx`: `createFileRoute("/settings/suppliers")` + `SupplierManagementPage`（既存 `settings/` 配下 route と同型。URL search state なし = 78 doc §78.3、`validateSearch` なし）。
- UI `src/features/suppliers/`（78 doc §78.3 の tree どおり新設）: `SupplierManagementPage.tsx`、`components/CreateSupplierDialog.tsx`（SPEC-SUPI-D4、features/products 版は無改変）/ `SupplierUsageTable.tsx` / `RenameSupplierRow.tsx` / `MergeSupplierDialog.tsx`、`hooks/useSuppliersWithUsage.ts` / `useRenameSupplier.ts` / `useMergeSuppliers.ts`。画面契約・日本語文言・失敗境界は 78 doc §78.3〜§78.8 を正本とする（一覧列 `取引先名` / `関連商品数` / `入庫記録数` / `操作`、`N件` 表示、追加 dialog、インライン改名の Enter / Escape / pending 行単位無効化、統合 dialog 段階 1「残す側を選んでください」/ 段階 2「◯件の商品 / ◯件の入庫記録が付け替わります」+「この操作は元に戻せません。…」+ `統合する` / `残す取引先を選び直す` / `キャンセル`、Loading = `ListSkeleton`、Empty = `取引先はまだ登録されていません`、一覧失敗 = `取引先を読み込めませんでした` + `再試行`、not-found 時は一覧再取得導線）。共有 `PageHeader` / `EmptyState` / `ListSkeleton` と `ui/`（dialog / select / table / input / label / alert / button / badge）を流用する。
- query key `src/lib/query-keys.ts`: `suppliers` namespace 新設（`root: () => ["suppliers"]` / `withUsage: () => ["suppliers", "with-usage"]`、SPEC-SUPI-D3。literal key 直書き禁止）。
- D-052 C21 / C22 実体化（SPEC-SUP-D7 / SPEC-SUPI-D2）: `src/lib/invalidation-contract.ts` に `supplierRename()` = C21 / `supplierMerge()` = C22（key 集合は SPEC-SUPI-D2 の 8 key）、`src/test/invalidation-oracle.ts` に独立転記、`invalidation-contract.meta.test.ts` の entry 件数 20 → 22、両 file のヘッダコメント「C1〜C20」→「C1〜C22」、`invalidation-contract.static.test.ts` の ALLOWED 2 list は無改変（直接 `invalidateQueries` を使わない）。`docs/decision-log.md` D-052 Contract 行に C21 / C22 と件数（entry 20 → 22 / handler 数は実装後の `rg -c "invalidateByContract\(" src/features --glob '!**/*.test.*'` 実測値で更新）、`docs/UI_TECH_STACK.md` §2.5 の同件数を同期。成功 handler は `useRenameSupplier` / `useMergeSuppliers` が `invalidateByContract` 経由で呼ぶ。追加（create）は 78 doc §78.9 どおり自画面一覧の再取得のみで D-052 entry を増やさない。
- docs 正本同期（rally round 1 P1-1）: `docs/function-design/78-ui-supplier-management.md` §78.9 を実装済みの確定記述へ改訂 — C21 / C22 は D-052 登録済みであること、consumer 全数導出により C21 / C22 は同一の 8 key 集合に確定したこと（SPEC-SUPI-D2）、集合の正本は decision-log D-052 Contract 行と `src/lib/invalidation-contract.ts` であることを記し、旧列挙行（「…UI-15 新設 key の 3 系統」「上記 3 系統 + products 系 root」の非対称記述）は残さない。「追加成功は自画面一覧の再取得のみで D-052 entry を追加しない」文は維持。あわせて `docs/decision-log.md` D-078 の Invalidation reservation 段落直後に「実装 PR の consumer 全数導出（SPEC-SUPI-D2）により、C21 も C22 と同一の 8 key 集合に確定した（集合の正本は D-052 Contract 行と `src/lib/invalidation-contract.ts`）。」を追記する（予約記述自体は履歴として書き換えない）。
- navigation `src/config/navigation.ts`: システム管理エリア末尾（整合性検証の次、52-ui §52.4 の順序固定どおり）に `id: "ui-15"` / label・title `取引先管理` / `to: "/settings/suppliers"` / `status: "active"` / icon `Building2`（SPEC-SUPI-D5）を追加。`src/config/navigation.test.ts` に `test_navigation_req107_ui15_active_at_settings_suppliers`（ui-11c パターン）。
- docs `docs/function-design/52-ui-shared-layout.md`: 各項目アイコン表へ `取引先管理` / `Building2` の 1 行追加（SPEC-SUPI-D5、設計 gap の同 PR 是正）。§52.4 の「2026-08-25 時点」段落の「後続実装 PR が追加する」旨を実装済みの記述へ同期（22 項目 / pending 0 の実態は不変）。
- docs `docs/function-design/78-ui-supplier-management.md`: §78.6 相当へ「共通離脱ガード（UI_TECH_STACK §6.11 `useUnsavedChangesWarning`）は UI-USW-D3 (c)〈行単位の即時 DB 保存 + dialog 完結〉により適用しない」1 文追記（SPEC-SUPI-D6、PR #95 gated amendment 1 と同型の事前消化）。
- sweep test `src/hooks/unsaved-changes-guard-sweep.test.ts`: `EXCLUDED_PAGES` に `SupplierManagementPage`（`src/features/suppliers/SupplierManagementPage.tsx`）1 entry 追加（assertion 不変の manifest 追従、SPEC-SUPI-D6）。
- requirements `docs/spec/requirements.md`: REQ-107 の coverage を `deferred` → `required` へ昇格し、補足節の「REQ-107 は取引先管理の実装 PR で `required` へ昇格する」文と「REQ-107 / REQ-206〜208 は実装着手まで `coverage=deferred`」文を昇格後の実態へ追随是正（file 内 `REQ-107` 全 hit を rg で sweep）。90-traceability は再生成で追随。
- テスト: Rust = archived Matrix 予約名 R-1〜R-5 を確定（migration v6 / rename 4 件 / merge 4 件 / operation_log / usage 件数。配置は各層の既存 `#[cfg(test)]` 慣行）。RTL = `SupplierManagementPage.test.tsx` + hooks test + `navigation.test.ts`（Matrix 参照）。新設 FE test file は describe / it 名に `REQ-107` または `UI-15` を 1 箇所以上含め、`generate_traceability` の T4 FE baseline（`FE_UNREFERENCED_BASELINE = 22`）を変えない。既存 test の扱い（gated amendment 1 で契約を明確化、PR #95 WER (2) の事前明記を継承）: **既存 test case（fn / it / assertion / manifest）の改変**は次の 4 例外のみ — (i) `unsaved-changes-guard-sweep.test.ts` `EXCLUDED_PAGES` 1 entry、(ii) `invalidation-contract.meta.test.ts` の件数 literal 20 → 22 + oracle file への C21/C22 独立転記、(iii) 既存 Rust struct literal への機械的 field 追従が必要になった場合の assertion 不変の追従（現時点の実測では該当なし。発生時は Writer が報告に明記）、(iv) migration v6 追加に伴う既存 migration 系 test の version / 件数期待 literal 5 → 6 と対応する説明文言の機械的追従（gated amendment 2 + 3。2026-08-25 repo 全体 sweep による全数 = `migration.rs` 7 箇所〈502 / 512 / 518 / 551 / 587 / 651 / 667 行〉+ `schema_v2.rs` 5 箇所〈372 / 378 / 586 / 596 / 601 行〉、他 file は tests / mnt / biz / scripts/tests 含め 0 hit。test fn・構造・fixture helper・検証意味は不変とし、追従箇所を Writer が報告に全数列挙する）。**既存 test file への新規 test fn の追加**は改変に当たらず、本 packet が名指しする追加先に限り許容する — `src/config/navigation.test.ts`（R-6 到達テスト）と、R-1〜R-5 の配置先となる既存 Rust file（`product_repo.rs` / `product_service.rs` / migration 系）の `#[cfg(test)]`。いずれも既存 case・assertion は不変のまま（既存 fixture helper の流用は可、helper 本体の変更は不可）。
- Writer 完了条件に `cd src-tauri && cargo check --release`（Human Gate が L3 を含むため。CI gate ではない）。

## Non-scope

- 取引先の単独削除・問屋チャネル・約 80 社の事前一括投入・検索・並び替え・paging・bulk rename・統合 undo（78 doc §78.12）。
- UI-01a への取引先 filter 露出（backlog）。
- 既存 supplier 3 関数（`list_suppliers` / `find_or_create_supplier` / `find_supplier_by_id`）と `create_supplier` command の wire / 挙動変更、features/products の `CreateSupplierDialog` / `ProductForm` の変更。
- `Supplier` model / wire への `updated_at` 露出（SPEC-SUPI-D7。DB 列のみ、UI で表示しない）。
- suppliers への index 追加、E3（UI_TECH_STACK §2.5）の定義変更。

## Acceptance Criteria

- AC-1: `cd src-tauri && cargo test` PASS、予約 test 名 R-1〜R-5 の 11 fn（Ledger 参照。R-1 ×1 / R-2 ×4 / R-3 ×4 / R-4 ×1 / R-5 ×1、rally round 1 P2-1 で実カウント是正）が `rg -c "fn <name>"` で各 1 hit。
- AC-2: `cd src-tauri && cargo run --bin generate_bindings` 実行後に `git diff --stat src/lib/bindings.ts` が空（commit 済みと一致）、diff 内容は新規 3 command（`renameSupplier` / `mergeSuppliers` / `listSuppliersWithUsage`）+ `SupplierMergeResult` / `SupplierWithUsage` の追加のみ。`rg -c "renameSupplier|mergeSuppliers|listSuppliersWithUsage" src/lib/bindings.ts` ≥ 3、既存 `Supplier` 型の行は無変更。
- AC-3: `cd src-tauri && cargo test --test design_compliance_test` PASS（20-io / 30-biz の新設 fn シグネチャと実装の一致を含む）。
- AC-4: `npm run generate:routes && npm run typecheck && npm run lint && npm run format:check && npm test` PASS。`rg -c '"ui-15"' src/config/navigation.ts` = 1、`rg -c "test_navigation_req107_ui15_active_at_settings_suppliers" src/config/navigation.test.ts` = 1、`rg -c 'status: "pending"' src/config/navigation.ts` = 0。
- AC-5: `rg -c "supplierRename" src/lib/invalidation-contract.ts src/test/invalidation-oracle.ts src/features/suppliers/hooks/useRenameSupplier.ts` 各 ≥ 1、`supplierMerge` も同様（hook は `useMergeSuppliers.ts`）、`rg -c "toHaveLength\(22\)" src/lib/invalidation-contract.meta.test.ts` = 1、`rg -c "C21|C22" docs/decision-log.md docs/UI_TECH_STACK.md` 各 ≥ 1。
- AC-6: `rg -c 'coverage=required' docs/spec/requirements.md` の REQ-107 行反映 + `rg -n 'REQ-107' docs/spec/requirements.md` の全 hit に `deferred` 残存なし。`cd src-tauri && cargo run --bin generate_traceability -- --check` exit 0（REQ-107 は昇格後 T3 対象になり、本 PR の test 付与で WARN も出ない）。
- AC-7: `rg -c "Building2" src/config/navigation.ts docs/function-design/52-ui-shared-layout.md` 各 ≥ 1、`rg -c "MigrationKind::Custom\(schema_v6::apply_v6_supplier_updated_at\)" src-tauri/src/db/migration.rs docs/function-design/22-mnt-migration.md` 各 = 1（`rg -c "get_v6_supplier_updated_at_schema" docs/ src-tauri/ --glob '!docs/plans/**' --glob '!docs/archive/**'` = 0。glob は本 packet / Matrix / archive が oracle 記述として同 token を逐語引用することによる自己参照 hit の除外 — gated amendment 4、design PR round 1 P1-2 と同 class）。
- AC-8: `bash scripts/doc-consistency-check.sh` exit 0 + `bash scripts/doc-consistency-check.sh --target plan` 全チェック通過。
- AC-9: `bash scripts/local-ci.sh full` RESULT=PASS / END_TREE_STATE=CLEAN（content candidate と Ready exact-HEAD の 2 回、evidence は PR body。evidence log は先頭 `HEAD_SHA` と末尾 `END_HEAD_SHA` / `RESULT` / `MERGE_EVIDENCE_VALID` で読む）。
- AC-10: human visual confirmation（Windows native L3）の結果が PR body の `Human Gate` 欄に `L3: PASS` または `L3: FAIL` で記録されている。
- AC-11: C21/C22 の正本同期（rally round 1 P1-1）— `rg -F 'UI-15 新設 key の 3 系統' docs/function-design/78-ui-supplier-management.md` = 0 hit かつ `rg -F '上記 3 系統 + products 系 root' docs/function-design/78-ui-supplier-management.md` = 0 hit、`rg -F -c '同一の 8 key 集合に確定' docs/function-design/78-ui-supplier-management.md docs/decision-log.md` 各 ≥ 1。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-106 / REQ-107
- Architecture: `docs/ARCHITECTURE.md`（UI -> CMD -> BIZ -> IO 一方向、CMD 薄層）
- Function / command / DTO: `docs/function-design/78-ui-supplier-management.md`（SPEC-SUP-D1〜D10 正本）、`20-io-product-repo.md` §rename_supplier / §merge_suppliers / §list_suppliers_with_usage、`30-biz-product-service.md` §4.7.4〜§4.7.6、`40-cmd-product.md` の同 3 command 節
- DB: `docs/db-design/master-tables.md` §3 suppliers（updated_at / rename / merge 契約）、`transaction-tables.md`（receiving_records cross-reference）、`docs/function-design/22-mnt-migration.md` §14 migration v6
- Screen / UI: `docs/SCREEN_DESIGN.md` #21、`52-ui-shared-layout.md` §52.3 / §52.4、`docs/UI_TECH_STACK.md` §2.5（D-052）/ §6.11（UI-USW-D3）、`docs/design-system/`（04-backbone / 02-component-catalog）
- Decision log / ADR: D-078（owner 裁定 + C21/C22 予約）/ D-052（invalidation registry）/ D-075（suppliers = メーカー/ブランド意味論）、archived design packet `docs/archive/plans/2026-08-25-supplier-management-design.md` + Matrix「実装 PR への予約」R-1〜R-10

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 20-io / 30-biz / 40-cmd の supplier 3 関数節（PR #3 正本化済み） | existing sufficient |
| Command / DTO / generated binding / wire shape | 40-cmd + 本 packet Boundary / Wire Contract、`bindings.ts` 再生成 | existing sufficient + 生成物 updated in this PR |
| DB / transaction / audit / rollback / migration | master-tables §3 / 22-mnt §14（kind 表記 1 箇所是正 = SPEC-SUPI-D1） | updated in this PR（22-mnt kind 是正のみ） |
| Screen / UI / route state / Japanese wording | 78 doc（離脱ガード非適用 1 文追記 = SPEC-SUPI-D6、§78.9 確定記述への改訂 = SPEC-SUPI-D2 / P1-1）、52-ui（アイコン表 1 行 + §52.4 時点表記同期 = SPEC-SUPI-D5） | updated in this PR |
| CSV / TSV / report / import / export format | 該当なし | existing sufficient |
| Durable decision / ADR | decision-log D-052 Contract 行（C21/C22 + 件数）、D-078 追記（8 key 確定）、UI_TECH_STACK §2.5 件数 | updated in this PR |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 |
|---|---|
| Tauri command 3 件 | `#[tauri::command]` / `#[specta::specta]` 対 + `src-tauri/src/lib.rs` の `collect_commands!` と `generate_handler!` へ登録 + `cargo run --bin generate_bindings` で `bindings.ts` 再生成 + commit |
| migration v6 | `schema_v6.rs` 新設 + `db/mod.rs` mod 宣言 + `migration.rs` `migrations()` 登録 + 22-mnt §14 kind 表記是正 |
| route 新設（`/settings/suppliers`） | `src/routes/settings/suppliers.tsx` 追加 + `npm run generate:routes`（`routeTree.gen.ts` は gitignore、commit 不要） |
| operator 画面新設（UI-15） | `navigation.ts` システム管理末尾に entry + `navigation.test.ts` REQ-107 到達テスト + 52-ui アイコン表 1 行 |
| D-052 mutation entry 新設（C21 / C22） | `invalidation-contract.ts` + oracle 独立転記 + meta 件数 22 + ヘッダコメント C22 + decision-log D-052 Contract 行 + UI_TECH_STACK §2.5 件数。番号は D-078 予約どおり C21 / C22 |
| query key namespace 新設（`suppliers`） | `src/lib/query-keys.ts`（literal 直書き禁止） |
| REQ coverage 変更（REQ-107 昇格） | requirements.md 改訂 + `cargo run --bin generate_traceability` で 90-traceability 再生成（手動編集禁止） |
| 新規 Page 追加 | `unsaved-changes-guard-sweep.test.ts` の manifest 分類（`EXCLUDED_PAGES`）+ 78 doc 非適用 1 文 |

## Design Decisions（packet-local、SPEC-SUPI-D1〜D7）

### SPEC-SUPI-D1: migration v6 の kind は `MigrationKind::Custom`（22-mnt §14 の kind 表記を是正）

- 22-mnt §14 の処理ステップ 3「既存行の `updated_at` が `created_at` と一致することを同一 transaction 内で検証する」は、SQL batch を実行するだけの `MigrationKind::Sql`（`migration.rs` `apply_sql_migration`）では実装できない。ADD COLUMN + backfill + TX 内検証の既存先例は v3（`plu_target`）/ v5（plu_slots）でいずれも `Custom`。よって kind は `MigrationKind::Custom(schema_v6::apply_v6_supplier_updated_at)` とし、22-mnt §14 の kind 表記 1 箇所を同 PR で是正する（処理ステップ・見出し・登録順 v1→…→v6・description は無改変）。
- 却下: `MigrationKind::Sql` のまま検証を R-1 test だけに置く（設計正本の処理ステップ 3 が runtime 検証を要求しており、backfill 失敗時に rollback できない）/ 処理ステップ 3 を削る改訂（検証は backfill の実質的安全網で削る理由がない）。
- 転記先: 22-mnt §14（本 PR）。

### SPEC-SUPI-D2: D-052 C21 / C22 の invalidate 集合（consumer 全数導出）

- C21（rename、書込み列 = `suppliers.name` / `suppliers.updated_at`）= C22（merge、書込み列 = `products.supplier_id` / `receiving_records.supplier_id` + suppliers 行 DELETE）= 次の 8 key（順序非依存・重複なし）:
  `queryKeys.productForm.root()` / `queryKeys.priceRevision.root()` / `queryKeys.suppliers.root()` / `queryKeys.productList.root()` / `queryKeys.lowStock(false)` / `queryKeys.stockInquiryRoot()` / `queryKeys.receivings.root()` / `queryKeys.inventoryRecords.root()`。
- 導出（UI_TECH_STACK §2.5 の table.column 規則、Contract Probe P-2 の rg 実測）: `suppliers.name` は `LEFT JOIN suppliers` で `find_by_product_code`（productForm.product）/ `search_products`（productList.search / priceRevision.search）/ `get_stock_detail`（stockInquiry.detail）/ `list_low_stock_products`（lowStock）/ `list_receiving_records`（receivings.recent）/ `get_receiving_record_detail`（inventoryRecords.receivingDetail）の各 query result に `supplier_name` として載る。取引先候補は `productForm.suppliers()` / `priceRevision.suppliers()`（root の prefix 包含で invalidate）、UI-15 一覧は `suppliers.withUsage`。E3（name / 部門 / 単位 / 価格の閉集合）は `supplier_name` を含まないため除外根拠にならない。product / supplier の code 単位は改名・統合時に列挙不能のため root prefix とし、collateral は E2 で許容（`productForm.root()` が supplier query を巻き込む個別事例の precedent どおり）。`stockMovements` / `stocktake` / 売上系は supplier 列を読む query が 0 hit のため含めない（含めると過剰 invalidation 違反）。
- D-078 との整合: D-078 の「3 系統を全て含む」（productForm.suppliers / priceRevision.suppliers / UI-15 新設 key）は下限であり、上記 root 集合が prefix でこれを包含する。C22 の「products 系 root」= `productList.root()` を含む。C21 と C22 が同一集合に収束するのは、rename も表示中の `supplier_name` を全 consumer で変えるため（entry は operation が異なるため C21 / C22 の 2 entry として登録する）。
- 却下: D-078 の 3 系統 + products root だけに限定（receivings / inventoryRecords / stockInquiry / lowStock の `supplier_name` が stale のまま残り導出規則違反）/ E3 に取引先名を追加する定義変更（E3 の閉集合は PR #95 rally で再確認済みの durable 契約で、改名は低頻度・refetch 代償僅少のため正しく invalidate する側が適切）。
- 転記先: decision-log D-052 Contract 行 + UI_TECH_STACK §2.5 件数 + 78 doc §78.9 改訂 + D-078 追記（いずれも本 PR、Scope の docs 正本同期 bullet と AC-11。rally round 1 P1-1 = 予約時の非対称列挙を正本に残すと実装 SSOT と食い違うため、確定集合へ同期する）。

### SPEC-SUPI-D3: UI-15 の query key namespace

- `queryKeys.suppliers = { root: () => ["suppliers"], withUsage: () => ["suppliers", "with-usage"] }` を新設する。既存 key に `"suppliers"` を先頭要素とするものはない（2026-08-25 実測）。
- 却下: `productForm.suppliers()` の流用（UI-15 は usage 付き一覧で query が別、feature 所有も別）/ `supplierManagement` などの長い namespace 名（既存の feature 短名慣行に合わせる）。

### SPEC-SUPI-D4: `CreateSupplierDialog` は features/suppliers に新設

- 78 doc §78.3 の tree どおり `src/features/suppliers/components/CreateSupplierDialog.tsx` を新設し、成功後は UI-15 の usage 付き一覧のみ再取得する（§78.5、D-052 entry を増やさない）。features/products 版（open / onOpenChange / onCreated props）は無改変。
- 却下: features/products 版の cross-feature import（feature 境界を跨ぐ依存を作る）/ `components/patterns` への共通昇格（成功時挙動が画面文脈で異なり、共通化は 78 doc の tree 契約外。要望が続けば別 PR で判断）。

### SPEC-SUPI-D5: navigation icon = `Building2` + 52-ui アイコン表の gap 是正

- 52-ui の各項目アイコン表に「取引先管理」の割当がない（PR #3 の設計 gap、2026-08-25 実測）。lucide-react `Building2`（会社・法人の慣用アイコン、既存 22 項目と非重複）を割当て、52-ui のアイコン表へ 1 行追加する。
- 却下: `Handshake` / `Truck`（問屋・配送の含意が D-075 のメーカー/ブランド意味論とずれる）。

### SPEC-SUPI-D6: 離脱ガード分類 = `EXCLUDED_PAGES`

- `SupplierManagementPage` は UI-USW-D3 (c)（行単位の即時 DB 保存）に該当する: インライン改名は行内確定で即 DB 保存、追加・統合は dialog 完結で蓄積未保存が生じない。`EXCLUDED_PAGES` へ 1 entry 追加し、78 doc に非適用の 1 文を転記する（PR #95 gated amendment 1 の同型を plan 時点で消化）。
- 却下: `APPLIED_PAGES` + `useUnsavedChangesWarning` 配線（78 doc 契約外の UX 変更）。

### SPEC-SUPI-D7: `Supplier` model / wire 無改変

- `suppliers.updated_at` は DB 列のみで Rust `Supplier` model / wire に露出しない（UI に表示要求がなく、R-9 の「bindings diff = 新規 3 command + 2 DTO のみ」凍結と整合）。改名後の値の検証は R-2 の Rust test が DB を直接 SELECT して行う。
- 却下: `Supplier` へ `updated_at` field 追加（既存 wire の破壊的変更、UI 消費者ゼロ）。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-107 | 22-mnt §14 / master-tables §3 | SPEC-SUP-D5 / SPEC-SUPI-D1 | nullable + backfill（SQLite ALTER TABLE 制約）、TX 内検証のため Custom kind | schema_v6.rs / migration.rs / 22-mnt | R-1 |
| REQ-107 | 30-biz §4.7.4 / 40-cmd | SPEC-SUP-D3 / SPEC-SUPI-D7 | trim / 空文字 / 同名衝突 / 同値 no-op / updated_at + operation_log。wire は Supplier 不変 | product_service / product_repo / product_cmd | R-2 |
| REQ-107 | 30-biz §4.7.5 / 20-io / master-tables | SPEC-SUP-D4 | 1 TX で 2 UPDATE → DELETE、片側付替え経路なし、再実行は not_found | 同上 | R-3 / R-4（mutation 予約） |
| REQ-107 | 20-io §list_suppliers_with_usage / 30-biz §4.7.6 | SPEC-SUP-D8 | 独立集約で件数水増し防止、name 昇順全件、DTO は BIZ 所有 | product_repo / product_service | R-5（非空期待必須） |
| REQ-106 / REQ-107 | 78 doc §78.3〜§78.8 | SPEC-SUP-D2 / D6 / SPEC-SUPI-D4 | 一覧 + 追加 + インライン改名 + 統合 2 段階、失敗境界の分離、日本語文言 | features/suppliers/* | R-7（RTL） |
| REQ-107 | 78 doc §78.9 / D-078 | SPEC-SUP-D7 / SPEC-SUPI-D2 / D3 | consumer 全数導出の 8 key、C21/C22 の 2 entry、追加は entry 外 | invalidation-contract / query-keys / hooks | R-7 oracle + meta 22 |
| REQ-107 | 52-ui §52.3 / §52.4 / SCREEN_DESIGN #21 | SPEC-SUP-D9 / SPEC-SUPI-D5 | システム管理末尾 + 到達テスト + アイコン gap 是正 | navigation.ts / route / 52-ui | R-6 |
| REQ-107 | requirements.md 補足 | — | coverage=required 昇格 + 補足文 sweep + 90-traceability 再生成 | requirements.md / 90-traceability | AC-6 |
| REQ-107 | UI_TECH_STACK §6.11 | SPEC-SUPI-D6 | (c) 行単位即時保存で EXCLUDED、78 doc に非適用 1 文 | sweep test manifest / 78 doc | Matrix（T17 PASS） |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: yes（SPEC-SUP-D1〜D10 は 78 doc + 各層 doc + D-078 に正本化済み。本 packet の SPEC-SUPI-D1〜D7 は実装配置・是正・集合確定で、durable なもの〈D1 の kind 是正 / D2 の C21/C22 集合 / D5 のアイコン〉は同 PR で source docs へ転記する）。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: SPEC-SUPI-D1（22-mnt）/ D2（decision-log D-052 + D-078 追記 + UI_TECH_STACK + 78 §78.9 改訂）/ D5（52-ui）/ D6（78 doc 1 文）。D3 / D4 / D7 は既存正本の範囲内の実装配置で転記不要（Ledger の test で契約化）。
- Assumptions and constraints: suppliers への FK 参照は products / receiving_records の 2 テーブルで全数（design packet Contract Probe P-1 の rg 実測を Probe P-1 で再確認）/ SQLite `ALTER TABLE ADD COLUMN` は NOT NULL + 非定数 DEFAULT 不可（nullable + backfill）/ 単一 operator・単一 window で merge TX 外の並行書込みなし / suppliers.name は UNIQUE（既存 schema）。
- Deferred design gaps, risk, and follow-up target: 削除・問屋・80 社一括投入・検索・並び替え・undo（78 doc §78.12）/ UI-01a 取引先 filter（backlog）/ CreateSupplierDialog の共通昇格（要望発生時）。
- Test Design Matrix can cite design decision IDs or source doc sections: yes（SPEC-SUP-D1〜D10 / SPEC-SUPI-D1〜D7 / D-052 / UI-USW-D3）。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 絶対保証 = merge は参照 0 化後 DELETE（例外 = TX 外並行書込みだが単一 operator 前提で並行系なし、source==target は入口拒否）/ 既存 supplier wire 凍結（bindings diff 検分で拘束）/ 統合の 2 段階省略経路なし（RTL が段階 1 を経ない実行 API の不在を assert）。escape hatch なし。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | not applicable — POS / CSV adapter に接点なし、app-core master data のみ | — |
| Fact check / design decision split | 事実 = suppliers.name の consumer 全数（rg 実測、Probe P-2）/ FK 参照 2 テーブル（Probe P-1）/ MigrationKind の実装制約（Probe P-3）。判断 = SPEC-SUPI-D1〜D7 | 本 packet |
| Lifecycle / retry | merge 成功後の再実行 = source 不在 `NotFound`（黙って成功しない）/ rename 同値 no-op = 成功 / 一覧・追加・改名・統合の失敗境界分離と入力保持（78 §78.8） | Matrix State Lifecycle |
| Operator workflow | 統合は不可逆 → 2 段階 + 影響件数 + 不可逆文言。誤統合時の復元はバックアップ復元のみ（78 doc に正本化済み、簡易 undo なし） | L3 checklist |
| Replacement path | not applicable — 外部システム非依存 | — |
| Data safety / evidence | L3 は owner local DB の backup 控え → synthetic fixture 投入 → 確認 → backup 復元。実取引先名・実 DB を commit / screenshot しない | Data Safety |
| Reporting / accounting semantics | merge 後、過去の入庫記録の取引先表示は残す側の名称になる（同一実体の重複解消として履歴の意味は保存、78 doc 正本化済み） | — |
| Manual verification | operator 画面新設 → human visual confirmation（Windows native L3 4 項目、78 §78.10） | Test Plan |
| 環境・再現性 | migration v6 は既存 DB / 新規 DB の両方で適用される。L3 fixture は Ready 依頼と同時に owner へ渡す（repo 外） | Test Plan |

## Design Readiness

- Existing design docs are sufficient because: PR #3 が UI-15 の画面契約（78 doc）、IO / BIZ / CMD 契約（20-io / 30-biz / 40-cmd）、DB / migration 契約（master-tables / 22-mnt §14）、到達導線（52-ui / SCREEN_DESIGN）、要求正本（REQ-106 / REQ-107）、invalidation 予約（D-078 C21/C22）を正本化済み。residual gap は 3 件で本 PR が解消する: (a) 22-mnt §14 の kind 表記と処理ステップ 3 の不整合（SPEC-SUPI-D1）、(b) 52-ui アイコン表の取引先管理欠落（SPEC-SUPI-D5）、(c) C21/C22 の consumer 全数（D-078 は suppliers 系 3 系統の下限のみ列挙、SPEC-SUPI-D2 で確定）。
- Source docs updated in this PR: 22-mnt §14 kind 1 箇所 / 52-ui アイコン表 1 行 + §52.4 時点表記 / 78 doc 離脱ガード 1 文 + §78.9 確定記述への改訂 / decision-log D-052 Contract 行 + D-078 追記 / UI_TECH_STACK §2.5 件数 / requirements.md REQ-107 昇格 + 補足 sweep / 90-traceability（再生成）。
- Design gaps intentionally deferred: Design Intent Audit 参照。
- Durable decisions discovered in this plan and promoted to source docs: SPEC-SUPI-D1 / D2 / D5 / D6。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): UI-15 → product_cmd 3 件（+ 既存 `create_supplier` / `list_suppliers` は UI-15 では追加 dialog のみ `create_supplier` を流用）→ product_service（validation / TX / operation_log / DTO 変換）→ product_repo（SQL のみ）。CMD は 3 行 pattern を維持。
- Backend function design: 20-io / 30-biz §4.7.4〜§4.7.6（PR #3 正本化済み、本 PR は実装のみ）。
- Command / DTO / data contract: 40-cmd + Boundary / Wire Contract 節。DTO 2 種は BIZ 所有・CMD qualified path 参照（SPEC-PRVA-D5 準拠）。
- Persistence / transaction / audit impact: migration v6（nullable + backfill + TX 内検証）、merge = 1 TX（2 UPDATE + DELETE + operation_log）、rename = 実改名時のみ updated_at + operation_log。
- Operator workflow / Japanese UI wording: 78 doc §78.3〜§78.8 の文言を正とし、RTL は exact text で assert（Matrix）。
- Error, empty, retry, and recovery behavior: 一覧 / 追加 / 行単位改名 / 統合の 4 失敗境界を分離、入力・選択保持、not-found は一覧再取得導線（78 §78.8）。
- Testability and traceability IDs: REQ-106 / REQ-107 / SPEC-SUP-D1〜D10 / SPEC-SUPI-D1〜D7。FE test file は REQ-107 / UI-15 token を含め T4 baseline 不変。

## Contract Probe

- P-1（FK 参照全数の再確認）: `rg -n 'REFERENCES suppliers' src-tauri/src/db/schema_v*.rs` = 3 hit（schema_v1 products / schema_v1 receiving_records 初版 / schema_v2 receiving_records_new 再構築）で、suppliers を参照するのは products / receiving_records の 2 テーブル（2026-08-25 再実測、design packet P-1 と一致）。
- P-2（suppliers.name / supplier_id の consumer 全数）: `rg -n 'JOIN suppliers' src-tauri/src/db/ src-tauri/src/biz/` = receiving_repo 2 箇所（`list_receiving_records` / `get_receiving_record_detail`）+ product_repo 4 箇所（`find_by_product_code` / `search_products` / `get_stock_detail` / `list_low_stock_products`）で全数（2026-08-25 実測）。frontend の `supplier_name` 表示は `ReceivingPage` / `ReceivingRecordDetailPage` / `ProductForm`（supplierName）。stockMovements / stocktake / 売上系 repo に suppliers 参照は 0 hit。→ SPEC-SUPI-D2 の 8 key 集合の導出根拠。
- P-3（MigrationKind の実装制約）: `migration.rs` の `apply_sql_migration` は SQL batch 実行 + schema_versions 記録のみで任意検証を挟めない。v2 / v3 / v5 の Custom は fn 内で BEGIN → 変更 → 検証 SELECT → schema_versions → COMMIT を自己完結する（`schema_v5::apply_v5_plu_slots` 実読、2026-08-25）。→ SPEC-SUPI-D1 の根拠。
- P-4（参照 component の実在、PR #95 WER (3) 規律）: `PageHeader` / `EmptyState` / `ListSkeleton` は `src/components/patterns/` に、dialog / select / table / input / label / alert / button / badge は `src/components/ui/` に実在（2026-08-25 eza 実測）。未実装の参照 component なし。
- P-5（採番・登録面の現況）: `invalidation-contract.ts` ヘッダ = C1〜C20 / meta test `toHaveLength(20)` / D-052 Contract 行 = 20 entry・23 handler（decision-log 392 行）→ C21/C22 で 22 entry。`design_compliance_test.rs` SKIP_DOCS に `78-ui-supplier-management.md` 登録済み（PR #3）→ 本 PR での SKIP_DOCS 追加なし。`FE_UNREFERENCED_BASELINE = 22`（generate_traceability.rs:43）。navigation は 21 項目 / pending 0。`SPEC-SUPI` prefix は docs/（archive 含む）で 0 hit（2026-08-25 rg 実測）。
- P-6（既存 CreateSupplierDialog の形）: `src/features/products/components/CreateSupplierDialog.tsx` は open / onOpenChange / onCreated props + trim / 空文字 field error + `commands.createSupplier` 呼出し（実読）。UI-15 版の実装参考にできるが import はしない（SPEC-SUPI-D4）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-SUP-D5 / SPEC-SUPI-D1 migration v6（nullable 追加 + backfill + TX 内検証 + 重複適用なし） | schema_v6.rs / migration.rs / db/mod.rs / 22-mnt §14 | `test_migration_v6_adds_updated_at_with_backfill`（R-1: 既存行 = created_at、新規行 NULL 許容、再実行で v6 非重複適用） | — |
| SPEC-SUP-D3 rename trim / 空文字 | product_service::rename_supplier | `test_rename_supplier_trims_and_rejects_empty`（R-2） | L3 item 3 |
| SPEC-SUP-D3 同名衝突 = validation | 同上 | `test_rename_supplier_conflict_returns_validation`（R-2） | L3 item 3 |
| SPEC-SUP-D3 同値 no-op = 成功 + updated_at / log 非更新 | 同上 | `test_rename_supplier_same_name_noop`（R-2） | — |
| SPEC-SUP-D3 / D5 実改名時の updated_at + operation_log `supplier_rename` | 同上 + product_repo::rename_supplier | `test_rename_supplier_updates_updated_at_and_logs`（R-2） | — |
| SPEC-SUP-D4 merge = 2 UPDATE → DELETE の 1 TX | product_service::merge_suppliers + product_repo::merge_suppliers | `test_merge_suppliers_repoints_products_and_receiving_records_then_deletes`（R-3。**mutation 予約: receiving_records 側 UPDATE を欠落させた mutant が red**） | L3 item 4 |
| SPEC-SUP-D4 source == target 拒否 | 同上 | `test_merge_suppliers_rejects_same_id`（R-3） | — |
| SPEC-SUP-D4 不存在 = NotFound（再実行含む） | 同上 | `test_merge_suppliers_not_found`（R-3） | — |
| SPEC-SUP-D4 途中失敗の全 rollback | 同上 | `test_merge_suppliers_single_tx_rollback_on_failure`（R-3。receiving_records 側 UPDATE を一時 trigger 等で失敗させ、products 付替え・DELETE・log が残らない） | — |
| SPEC-SUP-D4 operation_log `supplier_merge`（ID・名称・2 件数） | 同上 | `test_merge_suppliers_writes_operation_log`（R-4） | — |
| SPEC-SUP-D8 usage 件数（COUNT 独立集約・0 件明示・name 昇順） | product_repo::list_suppliers_with_usage + product_service | `test_list_suppliers_with_usage_counts`（R-5。**非空期待 case 必須**: 商品 2 + 入庫 1 の取引先と 0 件取引先を併置し件数 exact + 昇順を assert） | L3 item 1 |
| SPEC-SUP-D8 wire = 3 command + 2 DTO のみ追加、既存凍結 | product_cmd / lib.rs / bindings.ts | AC-2 の bindings diff 検分 + 既存 supplier test（`test_list_suppliers_req101_empty` / `test_find_or_create_supplier_req101_*`）無改変 PASS | — |
| SPEC-SUP-D2 一覧表示（列 / N件 / 0件 / name 昇順）+ 追加導線 | SupplierManagementPage / SupplierUsageTable | RTL `REQ-107 一覧は name 昇順で取引先名・関連商品数・入庫記録数を N件 表示し 0 件も 0件 と明示する` | L3 item 1 |
| SPEC-SUP-D2 / REQ-106 追加 dialog（trim / 空文字 / 成功で一覧再取得のみ） | CreateSupplierDialog（suppliers 版） | RTL `新しい取引先を追加は trim して createSupplier を呼び成功後に usage 一覧だけを再取得する` / `取引先名が空白のみなら createSupplier を呼ばず field error を出す` | L3 item 2 |
| SPEC-SUP-D3 インライン改名 UI（Enter / Escape / pending 行単位 / 同名文言 / 失敗保持） | RenameSupplierRow / useRenameSupplier | RTL `名前を変更で行が入力欄になり保存で renameSupplier を呼び Escape で現在名に戻す` / `他行と同名の改名は統合案内の validation 文言を表示し入力を保持する` / `改名失敗時は行の編集状態と入力を保持し再試行を出す` / `改名 pending 中は同じ行だけ無効化する` | L3 item 3 |
| SPEC-SUP-D6 統合 dialog 2 段階（段階 1 選択必須 / 段階 2 件数 + 不可逆文言 / 戻る導線） | MergeSupplierDialog | RTL `統合は残す側を選ぶ段階 1 を経ないと実行できない` / `段階 2 は source の usage 件数で「◯件の商品 / ◯件の入庫記録が付け替わります」と「元に戻せません」を表示する` | L3 item 4 |
| SPEC-SUP-D4 / D6 統合実行（1 request / pending 中閉鎖・二重送信なし / 完了通知 = SupplierMergeResult 件数 / 失敗保持） | MergeSupplierDialog / useMergeSuppliers | RTL `統合するは mergeSuppliers を 1 回だけ呼び成功で dialog を閉じ結果件数と一致する完了通知を出す` / `統合失敗時は段階 2 と選択・件数表示を保持し統合できませんでしたと再試行を出す` | L3 item 4 |
| SPEC-SUP-D7 / SPEC-SUPI-D2 C21 / C22 の 8 key 集合 | invalidation-contract / oracle / hooks | RTL `改名成功後に D-052-C21 の独立 oracle 集合を invalidate する` / `統合成功後に D-052-C22 の独立 oracle 集合を invalidate する` + meta 22 + static test PASS | — |
| SPEC-SUP-D1 / D10 単独削除 action 不在 | SupplierManagementPage | RTL の操作列 assert（`名前を変更` / `統合` のみ、削除 button 不在）+ `rg -i 'delete_supplier' src/ src-tauri/src/` 0 hit | — |
| 78 §78.8 Loading / Empty / 一覧失敗 / not-found 回復 | SupplierManagementPage | RTL `一覧取得失敗は取引先を読み込めませんでしたと再試行を出し 0 件と誤認させない` / `0 件は取引先はまだ登録されていませんと追加導線を出す` / `not-found 失敗時は一覧が古い可能性の文言と再取得 action を出す` | L3 item 1 |
| SPEC-SUP-D9 到達導線 | navigation.ts / route / 52-ui | `test_navigation_req107_ui15_active_at_settings_suppliers`（R-6）+ `test_navigation_all_items_no_pending_status` PASS + AC-7 の icon rg | L3 item 1 |
| SPEC-SUPI-D6 離脱ガード分類 | sweep test manifest / 78 doc | 既存 T17 PASS + `rg -c "SupplierManagementPage" src/hooks/unsaved-changes-guard-sweep.test.ts` = 1 + `rg -c "UI-USW-D3" docs/function-design/78-ui-supplier-management.md` ≥ 1 | — |
| REQ-107 昇格 + traceability | requirements.md / 90-traceability | AC-6（deferred 残存 0 + `--check` exit 0） | — |
| 登録: bindings / routes / 90-traceability 再生成 | 生成物 | AC-2 / AC-4 / AC-6 | — |

## Test Plan

Test Design Matrix: [test-matrices/2026-08-25-supplier-management-impl.md](test-matrices/2026-08-25-supplier-management-impl.md)。archived design Matrix「実装 PR への予約」R-1〜R-10 を本 Matrix が確定・継承する。
Human Gate が L3 を含むため、Writer 完了条件に `cd src-tauri && cargo check --release` を含める（CI gate ではない）。

- targeted tests: Rust R-1〜R-5 の 11 fn、RTL（`SupplierManagementPage.test.tsx` / hooks test / `navigation.test.ts`）、`invalidation-contract.meta.test.ts`（22）/ `.static.test.ts`、`design_compliance_test`、`generate_traceability -- --check`、`generate_bindings` diff。
- negative tests: 空文字 / 空白のみ / 同名衝突 / source==target / 不存在 ID / merge 再実行 / 途中失敗 rollback / 一覧・追加・改名・統合の各 reject で入力保持。
- compatibility checks: 既存 supplier 3 関数の既存 test 無改変 PASS、`bindings.ts` の既存 export 不変、`test_navigation_all_items_no_pending_status` PASS、migration v1〜v5 済み DB と新規 DB の両方で v6 適用（R-1）。
- data safety checks: fixture は synthetic のみ。L3 は owner local DB の backup 控え → synthetic 投入 → 確認 → backup 復元。DB / screenshot / fixture は commit しない。
- main wiring/integration checks: route file → `routeTree.gen.ts`（生成）→ サイドバー entry → `SupplierManagementPage` mount、hooks が `commands.listSuppliersWithUsage` / `renameSupplier` / `mergeSuppliers` を bindings 経由で呼ぶ、`invalidateByContract(..., supplierRename())` / `supplierMerge()` の配線、`migrations()` に v6。

Human visual confirmation checklist（Windows native L3、owner は目視と PASS/FAIL のみ。evidence 整形は agent。fixture = Coordinator が Ready 依頼と同時に渡す synthetic 手順書〈backup 控え → UI-01b/UI-02 で synthetic 取引先 3 件・商品 2 件・入庫 1 件を投入 → 確認 → backup 復元〉）:

1. 到達と一覧: サイドバー「システム管理」末尾に「取引先管理」が表示され、押すと `/settings/suppliers` が開き、title・説明文・name 昇順一覧（取引先名 / 関連商品数 / 入庫記録数 / 操作）が見え、0 件の取引先も `0件` と表示される。
2. 追加: 「新しい取引先を追加」→ dialog で日本語名を入力（IME 確定の Enter で誤送信しない）→ 追加した行が一覧に見える。空白のみでは追加されず field error が出る。
3. インライン改名: 「名前を変更」→ 行内入力欄 → 保存で新名称が一覧に反映、Escape で元に戻る。他行と同名にすると「同じ名前の取引先があります。重複している場合は「統合」を使ってください。」が出て入力が保持される。
4. 統合: 消す側の行の「統合」→ 段階 1 で残す側を選択（未選択では進めない）→ 段階 2 で「◯件の商品 / ◯件の入庫記録が付け替わります」の件数が段階 1 の行表示と一致し、「この操作は元に戻せません。…」の不可逆文言が見える → 「統合する」→ 完了通知の 2 件数一致・一覧から source 消失・target の件数増加。事後に UI-01b の取引先 select と UI-14 の取引先 filter で旧名が消え新名/残存側だけになっている。確認後、backup を復元して synthetic データを消す。

## Boundary / Wire Contract

- producer: CMD `rename_supplier` / `merge_suppliers` / `list_suppliers_with_usage`（新設、product_cmd）、既存 `create_supplier` / `list_suppliers`（無改変、追加 dialog が `create_supplier` を流用）
- consumer: UI-15（本 PR）のみ。既存画面は新 command を呼ばない
- wire type: `rename_supplier(supplier_id: i64, name: String) -> Result<Supplier, CmdError>` / `merge_suppliers(source_id: i64, target_id: i64) -> Result<SupplierMergeResult, CmdError>`（`SupplierMergeResult { products_updated: i64, receiving_records_updated: i64 }`）/ `list_suppliers_with_usage() -> Result<Vec<SupplierWithUsage>, CmdError>`（`SupplierWithUsage { id: i64, name: String, product_count: i64, receiving_record_count: i64 }`）。snake_case JSON（specta 生成）
- internal type: IO `SupplierUsageRow`（Supplier + 2 件数、非公開）→ BIZ が wire DTO へ変換。`Supplier` model は `{ id, name, created_at }` のまま不変（SPEC-SUPI-D7、updated_at は DB 列のみ）
- precision/range: 件数は i64（SQLite COUNT の自然型）。name は既存 `create_supplier` と同一の trim / 空文字拒否 / UNIQUE
- round-trip path: UI 入力 → trim（BIZ）→ DB → C21/C22 invalidate → usage 付き一覧再取得で表示。統合は段階 2 の件数（一覧 row の usage 値）→ 実行 → `SupplierMergeResult` の実付替え件数で完了通知
- invalid input: 空文字 → `validation` / 他行と同名 → `validation`（統合案内は UI 文言）/ 不存在 ID・merge 再実行 → `not_found` / source==target → `validation`。kind は既存 `From<BizError>` 変換のまま（40-cmd の割当と一致）
- compatibility: 既存 `list_suppliers` / `create_supplier` の wire 無改変。`bindings.ts` diff は新規 3 command + 2 DTO の追加のみ。migration v6 は旧 DB（v5 適用済み）と新規 DB の両方で成立し、`updated_at` NULL は「改名前」として全経路で許容

## Review Focus

- merge の 2 UPDATE → DELETE が 1 TX 内で完結し、receiving_records 側 UPDATE 欠落 mutant が R-3 で red になるか（片側付替え = failure 定義筆頭）。rollback test が products 付替え・DELETE・operation_log の全不残存を assert しているか。
- C21 / C22 の 8 key 集合が SPEC-SUPI-D2 の導出と一致し、oracle が独立転記か。過剰側（consumer ゼロ key）と欠落側（P-2 の consumer 取りこぼし）の両方向。
- migration v6 が nullable + backfill + TX 内検証 + 再実行非重複で、22-mnt §14 の kind 是正が rg で機械確認できるか（AC-7）。
- 統合 dialog の 2 段階・件数文言・不可逆文言が 78 doc の exact 文言で RTL assert されているか（文言表 presence oracle 規律）。
- 既存 supplier wire の凍結（bindings diff / 既存 test 無改変）と、`Supplier` model 不変（SPEC-SUPI-D7）。
- usage 件数の 2 参照表独立集約（同時 JOIN の件数水増しがないこと、R-5 の非空期待 case）。
- D-062: packet / Matrix 内の数値主張は契約値・実測（コマンド併記）・`未実測` tag のいずれかか。

## Spec Contract

Contract ID: SPEC-SUPI

- SPEC-SUP-D1〜D10（78 doc ほか source docs 正本化済み、実装対象）と SPEC-SUPI-D1〜D7（本 packet の Design Decisions 節）を正とする。

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| SPEC-SUP-D5 / SPEC-SUPI-D1 | schema_v6 + migration.rs + 22-mnt 是正 | R-1 | migration 方式 / kind 整合 | cargo test + AC-7 rg |
| SPEC-SUP-D3 | BIZ rename + IO + operation_log | R-2（4 fn） | validation / no-op / updated_at | cargo test |
| SPEC-SUP-D4 | BIZ merge 1 TX + IO 2 UPDATE → DELETE | R-3（4 fn）+ R-4 | 参照全数 / rollback / mutation 予約 | cargo test + mutant 実注入 |
| SPEC-SUP-D8 | IO usage 集約 + BIZ DTO + CMD 3 件 + bindings | R-5 + AC-2 | 件数水増し / wire 凍結 | cargo test + bindings diff |
| SPEC-SUP-D2 / D6 | features/suppliers UI 一式 | RTL（Ledger） | 2 段階 / 文言 / 失敗保持 | npm test + L3 |
| SPEC-SUP-D7 / SPEC-SUPI-D2 / D3 | C21/C22 + query keys + hooks + 78 §78.9 改訂 + D-078 追記 | RTL oracle + meta 22 + AC-11 rg | 集合の両方向 / 正本同期 | npm test + AC-11 |
| SPEC-SUP-D9 / SPEC-SUPI-D5 | navigation + route + 52-ui icon | R-6 | 表順 / pending 0 / icon | npm test + AC-7 |
| SPEC-SUPI-D6 | sweep manifest + 78 doc 1 文 | T17 PASS + rg | 分類根拠 | npm test |
| REQ-107 | requirements 昇格 + 90-traceability | AC-6 | deferred 残存 0 | --check exit 0 |

## Data Safety

- 実店舗の取引先名・商品名・価格・入庫実績を test fixture / PR / screenshot に commit しない（synthetic 名のみ、例:「テスト取引先A」）。
- local-only paths: owner の local DB とその backup（L3 用、repo 外）、L3 手順書の synthetic 投入データ（repo 外）。
- synthetic-only paths: `src-tauri/src/**` の `#[cfg(test)]` fixture、`src/features/suppliers/**/*.test.tsx` / `src/config/navigation.test.ts` の mock。

## Implementation Results

- migration v6、IO / BIZ / CMD の取引先改名・統合・usage 付き一覧、generated bindings、UI-15 route / navigation / 追加・インライン改名・2 段階統合、D-052 C21/C22、REQ-107 昇格と source docs 同期を実装した。
- Rust / frontend / docs / generated drift / release check の自動検証と、補助 Codex review-only の指摘是正・closure confirmation を完了した。Windows native L3 と packet 指定の独立 Sonnet Final Review は Draft PR 後の gate として残す。
- Draft PR: https://github.com/higekiri-w/inventory-system-desktop/pull/4

## Review Response

If R3 review-only sub-agent is skipped, record an explicit line beginning with `Review-only skipped because:` and the reason.
- Plan Review round 1（Sonnet、独立 fresh context、2026-08-25、packet `0a2f278`）: P1 1 / P2 1 / P3 1、verdict fail。全件 Coordinator が rg で実証再現のうえ accept して是正: P1-1 C21/C22 の 8 key 対称集合（SPEC-SUPI-D2）が durable 正本 D-078（decision-log 635 行）と 78 doc §78.9 の非対称記述（C21 = 3 系統 / C22 = +products root）と矛盾したまま Scope 外だった → Scope に docs 正本同期 bullet（78 §78.9 確定記述への改訂 + D-078 追記）+ AC-11（新旧文言 presence oracle）+ Required Design Artifacts / Design Readiness / Trace Matrix / SPEC-SUPI-D2 転記先を同期 / P2-1 AC-1 の Rust 予約 test 数「10 fn」は Ledger 実カウント 11 fn と不一致（D-062）→ 11 へ是正（Test Plan 側も同期）/ P3-1 Contract Probe P-2 の `list_recent_receiving_records` は不存在 fn 名（正 = `list_receiving_records`、receiving_repo.rs:118）→ SPEC-SUPI-D2 導出文と P-2 の 2 箇所を是正。
- Plan Review round 2（Sonnet、独立 fresh context、2026-08-25、packet `d4a54c3`）: round 1 是正 3 件は全 CLOSED（AC-11 oracle の自己参照なし・11 fn 実カウント一致・fn 名実在を独立再現）。fresh 全文レビューで新規 P1 / P2 / P3 = 0、verdict pass。SPEC-SUPI-D2 の 8 key を実コード側から独立再検算（`searchProducts` の非 cache 命令的呼出しと query cache 消費を区別し、欠落・過剰の両方向で問題なし）、SPEC-SUPI-D1 の v5 pattern 一致・R-1〜R-10 全数継承・D-062 編成適合・採番衝突なしを確認。rally 収束（2 round / 天井 3、P1+P2: 2 → 0）。
- Final Review（Sonnet、独立 fresh context、worktree 隔離、content candidate `f8dbd67`、2026-08-25〜26）: P1 0 / P2 0 / P3 1 = PASS。Scope 突合 = 変更 file 全数が packet Scope / gated amendments に対応（packet 外 0 / 未実装 0、packet への実装 hunk は Implementation Results 節のみ）。Contract Coverage Ledger 全行の内容一致監査、AC-1〜AC-8 + AC-11 の全再実測 PASS、凍結境界（bindings diff = 新規 3 command + 2 DTO のみ・既存 test 変更は申告どおり migration 系 literal 12 箇所と許容例外のみ・static test / SKIP_DOCS diff 0）、Matrix 予約の 7 mutant 全件を clean worktree で実注入し全 red → revert 後全緑・終了時 worktree clean、D-052 は production 8 key が SPEC-SUPI-D2 と一致・oracle literal 独立転記・entry / handler 数の docs 記載と実測一致、review-only P2 是正 3 件の実在確認、negative-space audit で未実装契約なし。P3-1 = 78 doc §78.4 の `SupplierWithUsage` field 名表記（camelCase）が実 wire（snake_case、40-cmd 記載と一致）と不一致 — design PR #3 由来の pre-existing・本 packet Scope 外・機能影響なしのため同 PR 是正せず backlog 起票（closeout で Plans.md へ）。
- Findings Freeze: frozen after Final Review（2026-08-26）; post-freeze exceptions: none.

### 遷移記録（2026-08-26、state-only 遷移 implementing -> local-verified -> independent-review -> human-confirm）

- implementing -> local-verified の evidence: content candidate `f8dbd67`（Codex Writer 第 1 発注、gated amendment 1〜4 反映済み、relay 1/2）で L1 `local-ci.sh full` RESULT=PASS / END_TREE_STATE=CLEAN / MERGE_EVIDENCE_VALID=true（evidence = `.local/ci-evidence/`、実 run は先頭 `HEAD_SHA` と末尾 `END_HEAD_SHA` / `RESULT` / `MERGE_EVIDENCE_VALID` で判読。exact SHA は PR #4 body 正本）。Coordinator は scope file 一覧・plan-first ancestry・L1 境界行・C21/C22 集合・正本同期 oracle・navigation entry を独立再実測して一致確認。
- local-verified -> independent-review の evidence: 独立 Sonnet Final Reviewer（fresh context、worktree 隔離）が Contract Audit を実施（Review Response の Final Review 行が正）。
- independent-review -> human-confirm の evidence: findings 裁定完了・P1/P2 = 0（P3 1 件は backlog 起票で処理）。`Reviewed Content HEAD` = `f8dbd67` と `Amendments` 行（gated amendment 1〜4 の SHA）を本 state-only commit で設定。隣接 3 遷移を 1 commit で圧縮記録（forward state-only 2 本目 / cap 3、post-implementation 1 本目 / cap 2）。

### 遷移記録（2026-08-26、state-only 遷移 human-confirm -> ready-hosted-final）

- Windows native L3（介入 2/3）: checklist L3-1〜L3-4 を owner が全件 PASS（到達と一覧・`N件`/`0件` 表示・削除 action 不在 / 追加 + IME Enter + 空白 reject / インライン改名 + Escape + 同名衝突文言と入力保持 / 統合 2 段階 + 件数文言一致 + 不可逆文言 + 完了通知 + UI-01b・UI-14 への invalidation 波及）。synthetic fixture は baseline backup へ復元し消失確認済み。実測記録・復元証跡は PR #4 body 正本。UX 所感 2 件（改名ボタンの double-click 貫通リスク / 改名時のみ完了通知がない非対称性）は Findings Freeze 後 P3 = follow-up として closeout で Plans.md backlog へ起票する（78 doc は改名時通知を規定せず設計適合、L3 非阻害）。
- owner Ready 承認（2026-08-26、介入 3 回目 / 予算 3 回）: 本 state-only commit で `human-confirm -> ready-hosted-final` を materialize（forward state-only 3 本目 / cap 3、post-implementation 2 本目 / cap 2）。resulting exact HEAD で L1 full を再実行し、PR body を更新してから owner が Draft を解除する。Ready event の同一 HEAD hosted final を待ち、三点一致前は merge しない。

### merge / closeout（2026-08-26）

- hosted run success で PR HEAD = L1 evidence SHA = hosted headSha の三点一致成立（run URL / headSha は PR #4 body 正本）。owner merge（merge commit `c2cdda8`、PR #4 @ inventory-system-desktop。squash ではなく merge commit で branch の 12 commit を保存）。
- Post-Merge Closeout: packet / Matrix を `docs/archive/plans/` へ移動、Phase を archive とし `Plans.md` を同期。Final Review P3-1（78 §78.4 field 名表記）と L3 owner 所感 2 件（改名ボタンの double-click 貫通リスク / 改名成功時の完了通知なし非対称）を Plans.md backlog へ起票。
- 実績 = 介入 3/3（Plan Gate / L3 / Ready）・relay 1/2（発注 1 本 + fail-closed amendment 回答 4 件は同一 lane）・forward state-only 3/3・gated amendment 4 本（全 true positive、Codex fail-closed 起源）。remote branch `agent/supplier-management-impl` の削除と local branch 掃除は owner 端末操作として残置。

### 遷移記録（2026-08-25、state-only 遷移 plan-draft -> plan-gate -> plan-approved -> implementing）

- plan-draft -> plan-gate の evidence: packet と Test Design Matrix を plan-first commit `0a2f278` で commit 済み、`doc-consistency-check.sh --target plan` exit 0（WARN 3 は file 名をカラム名と誤認する既知 class の偽陽性）。
- plan-gate -> plan-approved の evidence: 独立 Sonnet Plan Reviewer rally 2 round（round 1 P1 1 / P2 1 / P3 1 → 全件実証再現のうえ是正 `d4a54c3`、round 2 fresh context で round 1 全 CLOSED + 新規 0 / verdict pass）、owner Plan Gate 承認（2026-08-25、介入 1 回目 / 予算 3 回）、Plan Commit = plan-first commit `0a2f278`（本 branch の全実装 commit に先行する）。
- plan-approved -> implementing の evidence: Codex Writer への発注直前遷移（Coordinator が発注前に遷移を完了する運用）。隣接 3 遷移を 1 state-only commit で圧縮記録（DEV_WORKFLOW 圧縮規則の canonical 例と同型、forward state-only 1 本目 / cap 3）。

### gated amendment 1（2026-08-25、Codex Writer fail-closed 起源、true positive）

- 事象: Codex が発注書の起動条件確認後、実装開始前に停止・申告。packet の「既存 test は 3 例外を除き無改変」と、AC-4 / Scope が要求する `navigation.test.ts` への R-6 到達テスト追加が矛盾する（同 file は 3 例外に含まれない）。
- 検分: Coordinator が rg で再現 — packet 95 行の凍結文と 90 / 111 行の追加要求が矛盾し、さらに同 95 行自身が R-1〜R-5 の配置先を「各層の既存 `#[cfg(test)]` 慣行」と規定するため、Rust 既存 file（product_repo.rs 等）への test 追加も同 class の矛盾（Codex 指摘の navigation.test.ts より広い）。Matrix 40 行の「許容する既存 file 変更は 3 例外のみ」も file 単位表現で同矛盾。
- 裁定: Codex 提案（navigation.test.ts を第 4 例外に追加）は範囲不足のため一般化して採用 — 凍結契約を「既存 test case の改変（3 例外のみ）」と「既存 test file への新規 test fn の追加（packet 名指しの追加先に限り許容、既存 case・assertion 不変）」に区別する形で packet / Matrix の 2 箇所を是正。実装内容・test 予約・AC は不変（契約表現の明確化のみ）。
- 却下: 第 4 例外としての個別追加（R-1〜R-5 の Rust file が漏れ、同じ fail-closed が再発する）/ 凍結文の削除（既存 case 改変の防御が消える）。
- amendment commit SHA は Workflow State `Amendments` に後続 commit で記録（PR #86 / #95 と同型）。

### gated amendment 2（2026-08-25、Codex Writer fail-closed 起源、true positive）

- 事象: Codex が v6 実装中に既存 `test_migration_req903_is_idempotent` の FAIL（left: 6 / right: 5）を検出して停止・申告。既存 migration test が最新 version = 5 と migration 件数 = 5 を literal で固定しており、v6 登録後は「既存 test case の改変は例外のみ」（gated amendment 1 後契約）と両立しない。
- 検分: Coordinator が rg で再現 — `migration.rs` の v5 固定 assertion は申告どおり 7 箇所（502 / 512 / 518 / 551 / 587 / 651 / 667 行）で全数一致。version 進行に伴う期待 literal の機械的追従であり、検証意味（idempotency / 登録順 / 件数整合）は不変。
- 裁定: Codex の最小 amendment 案をそのまま採用 — 例外 (iv) として「既存 migration test の version / 件数期待 literal 5 → 6 と対応説明文言の機械的追従」を packet / Matrix に追加。例外 (iii)（struct literal 追従）と同 class の事前明記漏れで、PR #95 WER (2) の教訓の適用範囲を version literal まで広げるもの。
- 却下: migration test の新規複製（旧 test を残し v6 用を別 fn で追加 — 同一検証の二重化で v7 以降も直らない）/ 期待値の動的導出（`migrations().len()` 参照は test oracle の SSOT 共有で mutation 感度を失う、feedback: Test oracle must not share SSOT）。
- amendment commit SHA は `Amendments` に後続 commit で記録。

### gated amendment 3（2026-08-25、Codex Writer fail-closed 起源、true positive）

- 事象: Codex が `cargo test` で `schema_v2.rs` の既存 test 2 件（`test_v2_req903_applied_on_fresh_db` / `test_v2_req903_idempotent_rerun`）の FAIL を検出して停止・申告。amendment 2 の例外 (iv) が `migration.rs` に file 限定されており、schema_v2.rs 側の同 class（version 固定 literal 5 箇所）を許容しない。
- 検分: Coordinator が申告 5 箇所（372 / 378 / 586 / 596 / 601 行）を実読で全数一致確認。あわせて amendment 2 時点で怠った repo 全体 sweep（drift-fix sweep 規律）を実施 — schema_v1 / v3 / v4 / v5・test_support・src-tauri/tests・mnt・biz・scripts/tests に version 固定 literal は 0 hit で、本 class の全数は migration.rs 7 + schema_v2.rs 5 の 12 箇所で閉じる。
- 裁定: 例外 (iv) を「migration 系 test の version / 件数期待 literal 追従」へ一般化し、sweep で確定した全数を列挙（file 限定の逐次拡張を止め、class ごと閉じる）。Codex の最小案と同内容 + 全数保証。
- 反省記録（WER 候補）: amendment 2 で Codex 報告の file だけに例外を絞り、同型の repo 全体 sweep を怠ったことが本 amendment の fail-closed 再往復を生んだ（往復実績は本節の gated amendment 記録が evidence）。既存 test 凍結の例外設計は「報告箇所」でなく「class の全数 sweep」で確定する（drift-fix sweep の Plan 段階適用）。
- relay 会計の訂正: fail-closed への amendment 回答は同一発注 lane 内であり新規 relay に計上しない（PR #95 実績「1 発注 + amendment 回答 = relay 1/2」の先例に整合）。本 change の relay 実績は 1/2 のまま。
- amendment commit SHA は `Amendments` に後続 commit で記録。

### gated amendment 4（2026-08-25、Codex Writer fail-closed 起源、true positive）

- 事象: Codex が AC-7 の negative oracle `rg -c "get_v6_supplier_updated_at_schema" docs/ src-tauri/` = 0 を実行し、packet 2 hit + Matrix 1 hit（いずれも oracle 記述の逐語引用）で PASS 不能と申告。source docs / 実装側の残存は 0。
- 検分: Coordinator が同コマンドを再実行し hit が docs/plans/ の 2 file のみであること、`--glob '!docs/plans/**' --glob '!docs/archive/**'` 付与で 0 hit（exit 1）になることを実証。あわせて packet / Matrix の他の negative oracle を sweep — docs/ を無 glob で対象にするのは本コマンドと Matrix の鏡写し行のみで、他（AC-6 / AC-11 / delete_supplier 不在）は file・src 限定で自己参照汚染なし。
- 裁定: Codex 提案を採用し、closeout での archive 移動後も成立するよう `!docs/archive/**` も併せて付与（design packet AC-1 / AC-8 と同型）。Scope・実装契約・期待値は不変。design PR rally round 1 P1-2 と同 class の再発であり、負値 oracle 起草時の自己参照 self-check（design PR WER 候補 (1)）を本 packet の AC-7 に適用し損ねた Coordinator の見落とし。
- 却下: packet / Matrix 側の記述を非一致表記（token 分割）に書き換える（oracle の可読性を損ない、glob 除外という確立済みの標準手段がある）。
- amendment commit SHA は `Amendments` に後続 commit で記録。
