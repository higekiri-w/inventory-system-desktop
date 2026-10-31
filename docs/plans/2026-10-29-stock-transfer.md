# Plan Packet: 在庫の振替（詰め替え・切り分け。導入の前の関門 9）

wave に属さない単独の lane（PR #216、branch `agent/stock-transfer`）。owner の決定（repo 外の回答台帳 TD-463、2026-10-27）で、在庫の振替（仕入れた品を別の商品にして売るときの在庫の動き。反物 → 端切れ、大袋 → 小袋）を初回の導入の前に入れる（導入の前の関門の confirmed の 9、[Plans](../../Plans.md)・[backlog](../backlog.md) の「在庫の振替（詰め替え・切り分け）」）。店主の答えは TD-358（問い Q-092 に「よくある」）。b1 の起票の「`inventory_movements` の 2 回目の作り直し〈vZ〉に予約として足さない」（TD-378）は Q-092 を未回答と扱った時点の判断で、TD-463 で覆った。新しい決定は D-143。

起票し直し: 2026-10-27 に旧形式で起票した packet（`Phase` の行と Test Design Matrix を持つ版）を、owner の決定（2026-10-29）で新形式（`docs/DEV_WORKFLOW.md` Plan Packet Rules）の本 packet に置き換えた。設計の中身（D-143 と各正本の plan-first の置換え）は変えず、計画の成果物の形だけを変えた。旧い packet と Matrix は本 packet の起票の commit で外した（履歴は git に残る）。

## Workflow State

- Risk: R4
- Plan Commit: 4dd2ca2415ceeac81b60b4253eaa0cbf0344b9de
- Amendments: none

## Risk

Risk: R4

Reason:
migration vT（[22 §21](../function-design/22-mnt-migration.md)）が行のある `inventory_movements`（在庫の記録の原本。D-051・D-118）を作り直す: 新しい表へ全行を写して `DROP TABLE` し、名前を変え、`sqlite_sequence` を戻す。`docs/DEV_WORKFLOW.md` Risk Tiers の R4 の「破壊的なデータの lifecycle」に当たる（Coordinator の裁定、2026-10-27）。先例: 同じ形で `inventory_movements` を作り直した D-118 の (a) の v9（`docs/archive/plans/2026-10-18-stocktake-count-runtime-a.md` の Risk: R4）と b1 の vZ（`docs/archive/plans/2026-10-22-stocktake-count-runtime-b1.md` の Risk: R4）。vT が行の値を変えないこと、止める条件を持たないこと、店の DB が空の状態で vT を通ることは R4 を下げる理由にしない（写し方の誤り〈列の取り違え・id の振り直し〉は在庫の記録を黙って壊し、行数と id の集合の確かめだけでは列の取り違えを見つけられない。下の Data Safety と Recovery）。あわせて R3 の行の「Tauri command の DTO」（2 つの command、6 つの型、2 つの enum の値）と「operator の業務の流れ」（新しい作成画面と記録の詳細、2 つの一覧の種別）に当たる。workflow の file（`.github/`・`scripts/`）には触れない（classifier の `workflow=true` にならない）。

## Goal

Goal Invariant:

### 最小完了条件

- 入出庫の「詰め替え・切り分け」の画面で、日付と「減らす商品」「増やす商品」を 1 つ以上ずつ（数量はその商品の単位で、長さは m）入れて保存すると、1 つの記録として、減らす商品の在庫が減り、増やす商品の在庫が増える。両方が同じ TX で動き、片方だけが動いた状態は残らない（D-143 (1)(2)）。
- その記録は入出庫履歴（種別「詰め替え・切り分け」）と記録の詳細（減らした商品・増やした商品・関連在庫変動）で後から追え、在庫変動履歴の各行は種別「詰め替え・切り分け」と元記録の link を持ち、操作ログに残る（TRACE-D14）。
- 在庫の求め直し（D-118）は振替の movement を入出庫の movement と同じ規則で扱う: 同じ日付の既定の計数がある商品は要再確認になり、計数より前の日付の振替は計数に入っている（ADR SPEC-STK-CNT-D3）。
- 振替は商品の原価を変えない（D-143 (4)）。

### 失敗定義

- 保存が片方の商品だけを動かす、または途中の失敗で記録・明細・movement・操作ログのどれかが残る。
- 減らす商品の movement が正、増やす商品の movement が負、業務日がヘッダの日付と違う、phase が 0 でない、`reference_type`・`reference_id` が記録を指さない。
- 振替の movement が初期在庫とみなされる、または計数との窓の判定が入出庫と違う。
- 同じ商品を減らして増やす記録が保存できる。長さの商品の m の入力が cm に直らない、または数量の上限・0・負を通す。
- 商品の原価が振替で変わる。
- vT が値のある DB で止まる、行の値・id・`sqlite_sequence` を変える、または既存の種類の行を拒む。
- 入出庫履歴・在庫変動履歴・操作ログで振替の記録が見つからない、または詳細へ進めない。

### 非目的

- 取消・訂正（REQ-208 の取消の設計に後で入れる）。
- 元と先の換算の比の記憶と先の数の自動計算、先の原価を元から求めること（owner の判断事項 (ii) の B・C。D-143 の Revisit）。
- 振替だけの一覧画面。商品と在庫の画面から振替を始める導線。
- `stocktake` の種類を DB の CHECK と enum から外すこと（下の Scope の「この変更で不要になるもの」の確かめて残すもの）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

本 lane が変える file と目的。手法は 1 lane・1 PR（plan-first で設計正本を起票の commit に置き、計画の固定の後に S1〜S5 の段で runtime まで）。design lane（docs だけの PR）と runtime lane に分けない: 振替は既存の契約の組合せで新しい原理を持たず（在庫の求め直しは入出庫の movement と同じ規則で `biz::stock_ledger` を変えない、記録は廃棄と同じヘッダと明細の形で入出庫履歴の表〈`list_inventory_records` の `RecordSpec`〉に 1 行足すだけ、migration は vZ〈22 §20〉と同じ全行を写す形）、正本の置換えを独立に review する価値が分けたときの owner の手間（Ready と merge が 2 回ずつ）を上回らない。段は検証の単位で切る: S1〜S3 は 1 つの Rust の検証の単位（S3 の wire の変更は S4 の画面と bindings で揃う）、S4 は frontend の検証の単位、S5 は docs と生成物。既存の test の移し方は下の Test Plan の T1〜T3。

### 起票の commit で変えた正本（plan-first）

- [decision-log](../decision-log.md) D-143（新設）。
- [ADR](../adr/2026-10-16-stocktake-count-placement.md) の横断の約束の表の入出庫の行に振替。
- [DB_DESIGN](../DB_DESIGN.md) のテーブル一覧（11a・11b）・正本の一覧・CHECK 制約方針・インデックス方針・operation_type の一覧。
- [tracking](../db-design/tracking-system-tables.md) §14 の 2 つの CHECK の説明と movement_type の値の表（`transfer` と、`stocktake` を残す理由）。
- [transaction](../db-design/transaction-tables.md) §11a-11b（新設）。
- [10](../function-design/10-common-rules.md) の「入力の上限と安全な整数の範囲」(a) の 1 明細の数量と在庫の行。
- [21](../function-design/21-io-inventory-repo.md) §10.5a（新設）。
- [22](../function-design/22-mnt-migration.md) §3.1 のモジュール構成と §21（新設、MNT-03-D22）。
- [31](../function-design/31-biz-inventory-service.md) の D-118 の節の振替の項、§12.1、§12.6b（新設）、§12.6a の read 関数。
- [44](../function-design/44-cmd-inventory.md) §23.1、§23.3、§23.7 の `list_inventory_records` の種別、§23.7a（新設、CMD-13）。
- [52](../function-design/52-ui-shared-layout.md) の route 表の UI-16。
- [65](../function-design/65-inventory-record-traceability.md) の冒頭、§65.1、§65.2 TRACE-D14、§65.3、§65.12。
- [66](../function-design/66-ui-stock-movements.md) の Filters。
- [74](../function-design/74-ui-operation-logs.md) の UI-11c-D7、operation_type の registry、関連記録の許可リストと route 表。
- [79](../function-design/79-ui-stock-transfer.md)（新設、UI-16・UI-16b）。
- [FUNCTION_DESIGN](../FUNCTION_DESIGN.md) の索引の UI-16。
- [SCREEN_DESIGN](../SCREEN_DESIGN.md) の Phase 3 の画面の一覧と UI-16 の段落。

### S1 migration と IO

本 packet の計画の節（Scope・Acceptance Criteria・Contract Ledger・Recovery・Data Safety）は vT の版の番号を数字で書かない。番号は N（vT を書く head の base〈`origin/main`〉の `migrations()` の最大 + 1。22 §21）で、file 名だけが N を含み、関数名・定数・test 名は番号を含まない。merge の順で N が変わっても計画の節は変わらない。

- `src-tauri/src/db/schema_v{N}.rs`（新設）: `apply_stock_transfer`（22 §21 の手順。版の定数 `VT_VERSION`。vZ の `schema_v11.rs` と同じ helper〈`commit_transaction`・`rollback_after_error`〉）。
- `src-tauri/src/db/migration.rs`: `migrations()` に版 N で登録（description `振替の種類と振替の記録の表（D-143）`）。先に merge する側が小さい番号を取る（22 §21。#172 の packet と 22 §17 の「先に merge した側が若い番号」と同じ規則。`migrate` は版の欠けを確かめず今の版より大きいものだけを当てるので、main に欠けを残すと後から足した小さい版が当たらない）。この file に Test Plan の M7 の test（`migrations()` の版が 1 から欠けの無い昇順の連番）を書く。M7 があるので、branch の上の vT は N で書き、base が進んで N が変わったら file 名・`mod` の行・登録の版を新しい N に取り直す（runtime の変更。下の Recovery の「版の番号の取り直し」）。
- `src-tauri/src/db/mod.rs`: `pub mod schema_v{N}; pub mod transfer_repo;`。
- `src-tauri/src/db/transfer_repo.rs`（新設）: 21 §10.5a の 4 関数と `TransferDirection`・`NewTransferRecord`・`NewTransferItem`・`TransferRecordDetail`・`TransferRecordDetailItem`。
- `src-tauri/src/db/inventory_repo.rs`: `MovementType::Transfer`・`ReferenceType::TransferRecord` と `as_str`・`parse_*` の各 1 arm、as_str の test。
- `src-tauri/src/db/disposal_repo.rs`: `list_inventory_records` の `SPECS` に `transfer_record` の 1 行。
- `src-tauri/tests/design_compliance_test.rs`: `SKIP_DOCS` に `79-ui-stock-transfer.md`（`64-ui-disposal.md`・`78-ui-supplier-management.md` と同じ UI の設計書の扱い。関数の定義を持たない）、`build_doc_to_modules_map` の 21 の行に `db::transfer_repo`、31 の行に `biz::inventory_service::transfer`、44 の行に `cmd::transfer_cmd`。

### S2 BIZ

- `src-tauri/src/biz/inventory_service/transfer.rs`（新設）: `create_transfer`・`get_transfer_record` と 3 つの型（31 §12.6b）。
- `src-tauri/src/biz/inventory_service/mod.rs`: module と再 export。
- `src-tauri/src/biz/inventory_service/list.rs`: `list_inventory_records` の許可の `matches!` に `transfer_record`、`resolve_movement_source` の `TransferRecord` の arm（label `詰め替え・切り分け`・route `/inventory/transfer/records`）。
- `src-tauri/src/biz/inventory_service/common.rs`: `apply_stock_change` の doc comment の「入出庫 4 業務だけが呼ぶ」を振替を含めた 5 つに（振舞いは変えない）。

### S3 CMD と wire

- `src-tauri/src/cmd/transfer_cmd.rs`（新設）: 44 §23.7a の 2 command（`#[tauri::command]` と `#[specta::specta]` の対）。
- `src-tauri/src/cmd/mod.rs`: `pub mod transfer_cmd;`。
- `src-tauri/src/lib.rs`: `collect_commands!` と `generate_handler!` に 2 つずつ。
- `src-tauri/src/finite_enum_contract_tests.rs`: `MovementType::Transfer`・`ReferenceType::TransferRecord`・`TransferDirection` の 2 値の wire と DB の CHECK の parity。
- `src/lib/bindings.ts`: `cargo run --bin generate_bindings` で再生成（手で書かない）。

### S4 画面

- `src/routes/inventory/transfer.tsx`・`src/routes/inventory/transfer/index.tsx`・`src/routes/inventory/transfer/records/$recordId.tsx`（新設）と `src/routeTree.gen.ts`（`npm run generate:routes`）。
- `src/features/stock-transfer/StockTransferPage.tsx`・`lib/transfer-request.ts` と test（新設、79 §79.2〜§79.7）。
- `src/features/inventory-records/TransferRecordDetailPage.tsx` と test（新設、79 §79.8）。
- `src/features/inventory-records/types.ts`: `INVENTORY_RECORD_TYPE_OPTIONS` に `{ value: "transfer_record", label: "詰め替え・切り分け" }`（「廃棄・破損」の次）。
- `src/features/stock-movements/types.ts`: `MOVEMENT_TYPE_OPTIONS` に `{ value: "transfer", label: "詰め替え・切り分け" }`（「廃棄・破損」の次）。`lib/movement-formatters.ts` の `movementTypeLabels` に `transfer`（`Record<MovementType, string>` なので bindings の再生成の後は足さないと型の error）。
- `src/features/operation-logs/operation-type-labels.ts`: `transfer_create: { category: "入出庫", label: "詰め替え・切り分け記録" }`。`OperationLogsPage.tsx` の record_type → route の表に `transfer_record`。
- `src/config/navigation.ts`: 入出庫の `ui-05` の次に `ui-16`（`to` + `status: "active"`。79 §79.7）。
- `src/lib/invalidation-contract.ts`: `stockTransfer`（C23）。`src/test/invalidation-oracle.ts`・`src/test/invalidation-contract-static-fixtures.ts` に独立に写した C23。
- 既存の test の追随（Test Plan の T2）: `src/features/products/search.test.ts`、`src/features/stock-movements/StockMovementsPage.test.tsx`、`src/features/stock-movements/lib/movement-formatters.test.ts`、`src/features/operation-logs/operation-type-labels.test.ts`、`src/features/operation-logs/OperationLogsPage.test.tsx`、`src/features/inventory-records/InventoryRecordsPage.test.tsx`、`src/config/navigation.test.ts`（REQ-210 入りの UI-16 の到達の test）、`src/lib/invalidation-contract.meta.test.ts`、`src/hooks/unsaved-changes-guard-sweep.test.ts`（未保存の離脱ガードを持つ画面の一覧に `StockTransferPage`）。

### S5 docs・生成物

- `docs/spec/requirements.md` に REQ-210（「仕入れた品を別の商品に詰め替え・切り分けた在庫の動きを記録できること」。UI-16, UI-16b, BIZ-02、coverage `required`）、`docs/spec/requirements-coverage.md` の同じ行、79 の `> 対応仕様:` の行に REQ-210、`docs/function-design/90-traceability.md` を `cargo run --bin generate_traceability` で再生成（手で書かない）。起票の commit で足さない理由: 起票の commit は cargo を回さず、要求の一覧か `> 対応仕様:` の行を変えると 90 の再生成が要り、再生成しないと traceability の required gate が red になる。
- `docs/UI_TECH_STACK.md` の D-052 の entry の数（「22 mutation entry / 25 success handler」と C の範囲）を `rg -c "invalidateByContract\(" src/features --glob '!**/*.test.*'` の実測で直し、decision-log D-052 の Contract 行に C23 を 1 文で足す（D-052 の Revisit の規則）。
- `docs/ARCHITECTURE.md`・`docs/architecture/ui-task-specs.md`・`docs/diagrams/current-system.md` の画面・command の一覧に UI-16・UI-16b・CMD-13 の行。
- 起票の commit で置いた「proposed・未実装」の D-143 の分を現行の記述に直す（下の「この変更で不要になるもの」）。正本（22・DB_DESIGN・tracking・transaction）が書く vT の版の番号と file 名は、S5 の時点の N に合わせる（N が変われば同じ番号に直す）。
- 生成物（bindings・routes・traceability）の検査を S5 の後に 1 回通す。

### この変更で不要になるもの・確かめて残すもの

- backlog の項目「在庫の振替（詰め替え・切り分け）」（#214 で main に入った）と Plans の関門の 9: 本 lane の merge で済む。本 lane では backlog・Plans を変えない（merge の後に外す）。
- 正本の「proposed・未実装」の D-143 の分: 起票の commit で置いた注記（AC9 の command で数える）は S5 で現行の記述に直す。31 の「入出庫 4 業務」「4 つの `create_*`」、10 の「4 つの create」「5 経路」、74 の許可リストの括弧書き、66 の「b2 で変えていない」の文も同じ。
- code: 振替のための回り道の code は今は無い（廃棄 + 入庫の 2 回の操作は店の手作業で、アプリの code ではない）。外す code は無い。
- 確かめて残すもの: `movement_type` の `stocktake` と `reference_type` の `stocktake`。DB に `movement_type='stocktake'` の行は v9 の後に無い（22 §19 の手順 1 が止める）が、値は在庫変動履歴の種別の絞り込み（`list.rs` の `list_movements` の `Some(MovementType::Stocktake)`）と計数の行の元記録の種類（`resolve_movement_source(&Some(ReferenceType::Stocktake), …)`）に使い、DB の CHECK と enum の値の集合を同じに保つ（D-061・D-064。`finite_enum_contract_tests.rs` の parity）。vT で CHECK から外すと enum との parity が崩れ、enum から外すと b2 の wire が変わる。
- 確かめて残すもの: 廃棄の `list_disposals`（廃棄の作成画面の「最近の記録」が使う）。振替は使わないが、廃棄の画面を変えない。

### 並走の lane と重なる file（節の所有）

下の「予定 path の一覧」と open な PR の差分（`git diff --name-only --no-renames origin/main...origin/<branch>`）を `comm -12` で突き合わせた（2026-10-29、open は #212・#211・#201・#194・#190・#172。#190 とは 0）。本 lane は先頭に来たときに `origin/main` を単段で取り込む（`git -c merge.directoryRenames=false merge origin/main`）。生成物（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`・`src/routeTree.gen.ts`）は手で解かず、取り込んだ後に再生成する（D-098）。

| file | 本 lane の節 | 重なる PR | 扱い |
|---|---|---|---|
| `docs/decision-log.md` | D-143（D-140 と D-144 の間）、D-052 の Contract 行への 1 文（S5） | #194・#201・#211（D-142）・#212（D-141）の追記と各行 | 追記は番号順の位置で両方を残す。D-052 の行は本 lane だけ |
| `docs/SCREEN_DESIGN.md` | Phase 3 の画面の一覧と UI-16 の段落 | #172（SD 直読みの節）・#212（商品フォーム） | 節が違う。同期で当て直す |
| `docs/DB_DESIGN.md` | テーブル一覧 11a・11b、CHECK、index、operation_type | #172（`daily_report_imports` の列） | 節が違う |
| `docs/function-design/22-mnt-migration.md` | §3.1 の 1 行と末尾の §21 | #172（§3.1 の vS の file の行と §17 の vS） | §3.1 は隣の行で、同期で両方を残す。版の番号は 22 §21 の規則 |
| `docs/function-design/10-common-rules.md` | 「入力の上限と安全な整数の範囲」(a) の 2 行 | #211（日報の紙の節） | 節が違う |
| `docs/UI_TECH_STACK.md` | D-052 の entry の数（S5） | #212 | 同期の後に実測で直す |
| `docs/ARCHITECTURE.md` | UI-16 の行（S5） | #172 | 行が違う |
| `docs/function-design/90-traceability.md`・`src/lib/bindings.ts` | 再生成（S3・S5） | #172 | 手で解かず、同期の後に再生成 |
| `src-tauri/src/db/migration.rs`・`src-tauri/src/db/mod.rs` | vT の登録と 2 つの `mod` | #172（vS の登録） | 版の番号は 22 §21 の規則（先に merge した側が若い番号）。登録の行は両方を残す |
| `src-tauri/src/lib.rs`・`src-tauri/src/cmd/mod.rs` | 2 command の登録と `mod` | #172・#201（bin を外す。`lib.rs` だけ） | 登録の行は両方を残す |
| `src-tauri/tests/design_compliance_test.rs` | 79 の登録と 3 module | #172・#201 | 行が違う |

### 予定 path の一覧（`comm -12` の入力）

`{N}` は上の S1 の N。突き合わせでは、その時の N の数字に置き換えて入れる。

```text
docs/ARCHITECTURE.md
docs/DB_DESIGN.md
docs/FUNCTION_DESIGN.md
docs/SCREEN_DESIGN.md
docs/UI_TECH_STACK.md
docs/adr/2026-10-16-stocktake-count-placement.md
docs/architecture/ui-task-specs.md
docs/db-design/tracking-system-tables.md
docs/db-design/transaction-tables.md
docs/decision-log.md
docs/diagrams/current-system.md
docs/function-design/10-common-rules.md
docs/function-design/21-io-inventory-repo.md
docs/function-design/22-mnt-migration.md
docs/function-design/31-biz-inventory-service.md
docs/function-design/44-cmd-inventory.md
docs/function-design/52-ui-shared-layout.md
docs/function-design/65-inventory-record-traceability.md
docs/function-design/66-ui-stock-movements.md
docs/function-design/74-ui-operation-logs.md
docs/function-design/79-ui-stock-transfer.md
docs/function-design/90-traceability.md
docs/plans/2026-10-29-stock-transfer.md
docs/spec/requirements-coverage.md
docs/spec/requirements.md
src-tauri/src/biz/inventory_service/common.rs
src-tauri/src/biz/inventory_service/list.rs
src-tauri/src/biz/inventory_service/mod.rs
src-tauri/src/biz/inventory_service/transfer.rs
src-tauri/src/cmd/mod.rs
src-tauri/src/cmd/transfer_cmd.rs
src-tauri/src/db/disposal_repo.rs
src-tauri/src/db/inventory_repo.rs
src-tauri/src/db/migration.rs
src-tauri/src/db/mod.rs
src-tauri/src/db/schema_v{N}.rs
src-tauri/src/db/transfer_repo.rs
src-tauri/src/finite_enum_contract_tests.rs
src-tauri/src/lib.rs
src-tauri/tests/design_compliance_test.rs
src/config/navigation.test.ts
src/config/navigation.ts
src/features/inventory-records/InventoryRecordsPage.test.tsx
src/features/inventory-records/TransferRecordDetailPage.test.tsx
src/features/inventory-records/TransferRecordDetailPage.tsx
src/features/inventory-records/types.ts
src/features/operation-logs/OperationLogsPage.test.tsx
src/features/operation-logs/OperationLogsPage.tsx
src/features/operation-logs/operation-type-labels.test.ts
src/features/operation-logs/operation-type-labels.ts
src/features/products/search.test.ts
src/features/stock-movements/StockMovementsPage.test.tsx
src/features/stock-movements/lib/movement-formatters.test.ts
src/features/stock-movements/lib/movement-formatters.ts
src/features/stock-movements/types.ts
src/features/stock-transfer/StockTransferPage.test.tsx
src/features/stock-transfer/StockTransferPage.tsx
src/features/stock-transfer/lib/transfer-request.test.ts
src/features/stock-transfer/lib/transfer-request.ts
src/hooks/unsaved-changes-guard-sweep.test.ts
src/lib/bindings.ts
src/lib/invalidation-contract.meta.test.ts
src/lib/invalidation-contract.ts
src/routeTree.gen.ts
src/routes/inventory/transfer.tsx
src/routes/inventory/transfer/index.tsx
src/routes/inventory/transfer/records/$recordId.tsx
src/test/invalidation-contract-static-fixtures.ts
src/test/invalidation-oracle.ts
```

### Non-scope

- 取消・訂正、換算の比、先の原価の算出と提案、振替だけの一覧画面、商品の画面からの入口（上の Goal の非目的と 79 §79.9）。
- 廃棄の画面と `list_disposals` の変更。
- `stocktake` の種類を外すこと。
- 在庫の求め直し（`biz::stock_ledger`）・整合性の検査（36）・棚卸し（35・73）の変更（振替の movement は今の規則でそのまま扱われる。変えないことを Test Plan の L の行で確かめる）。
- backlog・Plans の更新（merge の後）。

## Acceptance Criteria

AC の baseline は 2026-10-27 に起票の worktree で測った（command は逐語）。実装の後の期待は「完了時」。

- AC1（migration）: Test Plan の M1〜M7 の Rust の test が PASS。M1〜M7 の test の名前は番号を含まない接頭辞 `vt_stock_transfer_` で始め（M7 は `migration.rs` に置く）、`cd src-tauri && cargo test vt_stock_transfer_` が全て PASS で、その出力の `test result:` の行の `passed` の和が 7 以上（M1〜M7 の 1 本ずつ。該当 0 本、または 7 本に満たないなら、exit 0 でも FAIL）。各 test の名前は接頭辞の後に Test Plan の行の印 `m1_`〜`m7_` を付け（`vt_stock_transfer_m1_…`〜`vt_stock_transfer_m7_…`。印は Test Plan の行の ID で、版の番号ではない）、`cd src-tauri && cargo test vt_stock_transfer_ -- --list` の出力に `vt_stock_transfer_m1_`〜`vt_stock_transfer_m7_` の 7 つの印がそれぞれ 1 本以上ある（どれか 1 つの印が欠ければ、passed の和が 7 以上でも FAIL）。空の DB と、入庫・返品交換・手動販売・廃棄・初期在庫・Z004（`phase = 1`）の movement と計数・棚卸しを持つ直前の版の DB の両方で vT が成功し、各 movement の全列と id と `sqlite_sequence` が vT の前と同じ。vT の後に `transfer`・`transfer_record` の行が入り、一覧に無い値が CHECK で拒まれる。
- AC2（BIZ の振舞い）: Test Plan の B1〜B14 の test が PASS。1 つの `create_transfer` で、減らす明細は `-quantity`・増やす明細は `+quantity` の movement（`movement_type='transfer'`、`reference_type='transfer_record'`、`reference_id=record_id`、`business_date=transfer_date`、`phase=0`、`settlement_no` NULL）を明細ごとに 1 つ作り、両方の商品の写しを求め直す。検証の失敗（31 §12.6b の手順 0・1・2・5 の各文）と TX の中の失敗（在庫の上限越え）で `transfer_records`・`transfer_items`・`inventory_movements`・`operation_logs` の行数が呼ぶ前と同じ。同じ key・同じ内容は `idempotent_replay: true` で行が増えず、同じ key・違う内容は `IdempotencyConflict`。`products.cost_price_centi` が呼ぶ前と同じ。
- AC3（D-118 との関係）: Test Plan の L1〜L4 の test が PASS: 振替の日付と同じ日付の既定の計数を持つ商品は要再確認（理由 2）、計数より前の日付の振替は計数に入り、後の日付は入らない、fresh な計数の保存の前に保存した振替は入る。振替の movement は初期在庫とみなされない（`INITIAL_STOCK_SQL` の式に当たらない）。`biz::stock_ledger` の差分が無い（`git diff origin/main -- src-tauri/src/biz/stock_ledger.rs` が空）。
- AC4（wire）: 完了時に `rg -n '^export type MovementType =' src/lib/bindings.ts` の行に `"transfer"`、`rg -n '^export type ReferenceType =' src/lib/bindings.ts` の行に `"transfer_record"` があり、`rg -n -e 'createTransfer:' -e 'getTransferRecord:' -e '^export type TransferDirection =' src/lib/bindings.ts` が 3 行・exit 0（baseline: 出力 0 行・exit 1）。baseline: `rg -n -e "'transfer'" -e '"transfer"' -e 'transfer_record' src-tauri/src src --glob '!src/lib/bindings.ts' | wc -l` → `0`（2026-10-29 に `1c925027` の worktree で再測。`wc -l` を外した `rg` だけでは出力 0 行・exit 1）。`cd src-tauri && cargo run --bin generate_bindings` の後に `git diff --exit-code src/lib/bindings.ts` が 0。
- AC5（記録の追跡）: Test Plan の R1〜R5 の test が PASS: `list_inventory_records` が `record_type: "transfer_record"` で振替の記録だけを返し、`"all"` に含め、商品の絞り込みが減らす明細と増やす明細の両方で当たり、件数は明細の数。未対応の種別は今のまま `validation`。`resolve_movement_source` が `詰め替え・切り分け #{id}` と `/inventory/transfer/records/{id}` を返す。`get_transfer_record` が減らす明細を先に返し、無い id は `not_found`。
- AC6（作成画面）: Test Plan の U1〜U11 の frontend の test が PASS（79 §79.10 の UI-16-D1〜D11）。
- AC7（記録の詳細と種別）: Test Plan の U12〜U16 の test が PASS（UI-16b、入出庫履歴と在庫変動履歴の種別の選択肢、操作ログの label と関連記録の link）。
- AC8（到達と invalidation）: `src/config/navigation.test.ts` の UI-16 の到達の test と、`src/lib/invalidation-contract.meta.test.ts` の C23 の独立の oracle との完全一致が PASS。
- AC9（正本の現行化）: `rg -n 'proposed・未実装' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | rg -c -e 'D-143' -e '振替' -e 'transfer' -e '詰め替え'` が、完了時に出力 0 行・exit 1（一致する行が 0）。baseline（起票の commit の後、2026-10-29 に `1c925027` の worktree で再測）: 出力 `38`・exit 0。`docs/spec/requirements.md` に REQ-210 の行があり、`cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0。
- AC10（gate）: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-10-29-stock-transfer.md` の本 lane が足した ERROR が 0、`bash scripts/check-workflow-git.sh` が exit 0、Rust（`cargo fmt --check`・`cargo clippy --all-targets --all-features -- -D warnings`・`cargo test`）と frontend（`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`）が PASS。L3 の前に Writer が `cargo check --release` を通す。
- AC11（L3）: owner が下の `## L3` の L3-1〜L3-7 を PASS とし、helper の manual の record（`python3 scripts/pr-gate.py status --pr <番号>` の manual）が L3 を頼んだ head で PASS になる。
- AC12（R4）: 独立 2 本の Final Review（Double Audit）が同じ head で helper の broad の record に載り（`python3 scripts/pr-gate.py status --pr <番号>` の Final Review が 2 本）、finding の裁定の後に owner の `r4` の承認が同じ helper の record に載る。L3 の前に Coordinator が vT の前の snapshot（下の Recovery の「写し方」）の写しで vT の予行（下の Recovery の当て方）を済ませ、予行の後の写しに vT が当たったこと（版が N、新設の 2 表、広げた CHECK。下の Recovery の (0)）を確かめた上で、行数・id の最大・`sqlite_sequence` が vT の前と同じで、`inventory_movements` の全列の比べ（vT の前の snapshot を `ATTACH` した両向きの `EXCEPT`）が両方 0 行。

## Design Readiness

- 引用する設計正本（節まで）: D-143、ADR SPEC-STK-CNT-D1〜D3・D9 と横断の約束の表、tracking §14、transaction §11a-11b、10 SPEC-UNIT-D2・D3・D11 と「入力の上限と安全な整数の範囲」(a)、21 §10.5a、22 §16・§20・§21、31 §12.2・§12.6・§12.6a・§12.6b・§12.8、44 §23.3・§23.7・§23.7a、52 の route 表、64（同じ形の作成画面）、65 TRACE-D1・D2・D7・D11・D14、66 の Filters、74 UI-11c-D7 と registry、79。wire の型（`TransferCreateRequest`・`TransferItemInput`・`TransferCreateResult`・`TransferRecordDetail`・`TransferRecordDetailItem`・`TransferDirection`）の正本は 44 §23.7a と 21 §10.5a。
- 必要な設計成果物: BIZ・IO・CMD・migration・table・operator の業務の流れ・日本語の語・decision の行が全て当たり、全て updated in this PR（起票の commit）。D-052 の Contract 行と UI_TECH_STACK の数は S5（実測が要る）。
- plan にしかない durable な判断の昇格先: 起票の時に起草役が決めた技術の判断（movement の種類は 1 つ〈`transfer`〉で向きは quantity の符号、記録はヘッダ `transfer_records` と明細 `transfer_items`〈向き `out`/`in`、UNIQUE(記録, 商品)〉の 2 表、一覧の command を作らず `list_inventory_records` を `record_type: "transfer_record"` で読む、検証の順は廃棄と同じで振替だけの検査は「減らす・増やすを 1 つ以上ずつ」〈明細の形の段〉と「同じ商品の 2 回目」〈明細ごとの段〉、画面の語は全て「詰め替え・切り分け」で「振替」は設計の中の名前、C23 は廃棄の C7 から `disposals.root()` を除いた集合、migration の版は先に merge する側が小さい番号を取り、branch の上では M7 が通る base の最大 + 1 で書く）は D-143 と 21・22・31・79 に置いた。Coordinator が覆せる。REQ-210 を S5 で足すのは配布の手順で、正本の判断ではない（Scope の S5）。
- 前提・制約と延期した design gap: 取消・訂正は REQ-208（deferred）の設計に入れる（65 §65.7.2）。換算の比と先の原価は D-143 の Revisit。店主に新しく聞く必要がある事項は無い（Q-092 の答え〈TD-358〉で足りる。換算の比を毎回同じにしているかは D-143 の Revisit の契機で、聞かなくても既定の形が成り立つ）。起票の commit（docs だけ）はどの required gate の green / red も変えない（新設の 79 は `> 対応仕様:` の行に REQ の ID を持たず、FUNCTION_DESIGN の索引の UI-16 はどの REQ の対応タスクにも無いので、`generate_traceability -- --check` の出力は変わらない）。ただし `design_compliance_test` は未登録の設計書を assert で止め、classifier は docs だけの変更で Rust の job を走らせないので、S1 が 79 を `SKIP_DOCS` に足すまでの間だけ、手元の `cargo test --test design_compliance_test` が 79 を名指して落ちる（先例: 取引先管理の 78 の設計書の commit〈`e28bdb8a`〉と登録の commit〈`8cdd1c38`〉が分かれていた）。
- owner の判断事項（製品の振舞いで正本から一意に決まらなかった問い。owner の方針〈TD-401〉で推奨を既定にして書き進めた。どれも L3 の依頼に束ねて owner に見せ、答えが既定と違えば名指した所を直す Gated Amendment にする）:
  - (i) 画面の置き場と語: A（推奨・既定）サイドバーの入出庫の「廃棄・破損」の次に独立の画面「詰め替え・切り分け」（`/inventory/transfer`）で、画面・一覧・在庫変動履歴・操作ログの語は全て「詰め替え・切り分け」。B 廃棄・破損の画面の中に切替を置く。C 商品と在庫の画面の行の詳細から始める。推奨の理由: 入出庫のほかの作成画面と同じ置き場と操作で、ロスの画面に別の目的の操作を混ぜない。C は元と先の 2 つの商品を選ぶ所が要り、商品の画面の道具が増える（D-138 の「失うもの」）。答えで変わる所: B・C なら 79 UI-16-D1・§79.5・§79.7、52 の UI-16 の行、`src/config/navigation.ts` と route の file、L3-1。語だけを変えるなら 79・65・66・74 の語と S4 の label の表。
  - (ii) 原価の移し方: A（推奨・既定）動かさない（振替は商品の原価を変えず、明細に原価を持たない）。B 保存の後に元の原価から求めた先の 1 単位の原価を参考に出し、先の商品の原価を直す提案をする（入庫の原価差分の承諾〈REQ-209〉と同じ形）。C B の値で先の原価を自動で書く。推奨の理由: 原価は商品の値で、棚卸しの評価は商品の原価を使う（D-143 の Why）。先が複数のときの配り方の規則が要らず、利用者の見ない所で原価が変わる経路を作らない。答えで変わる所: B なら 31 §12.6b に原価の読取りと提案の値、79 に提案の dialog、明細に元の原価の写し（`transfer_items` の列）、Test Plan に丸めの行、価格の履歴の書込み。C なら加えて価格の履歴の自動の書込みと配り方の規則（D-143 の再決定）。
  - (iii) 1 つの記録の明細の数: A（推奨・既定）減らす商品と増やす商品の両方を複数行にできる。B 元 1・先 1 に限る。C 元 1・先は複数。推奨の理由: DB と BIZ の手間は A も B も同じで、1 本の反物を 2 種の端切れに分ける・複数の商品から 1 つを作る使い方を 1 回で記録できる。答えで変わる所: B・C なら 31 §12.6b の手順 1 に行の数の検査、79 UI-16-D3 と §79.5、Test Plan の B3 と U3。
  - (iv) 商品の追加欄の検索の範囲（廃番の商品を含めるか）: A（推奨・既定）64 と同じく廃番を含めない（`is_discontinued: false`）。B 廃番を含める（減らす商品の欄だけ、または両方）。推奨の理由: ほかの入出庫の作成画面と同じ検索で、売らなくなった商品を増やす先に選ぶ誤りを作らない。弱み: 仕入れをやめて残りを切る反物（廃番）を減らす商品に選べず、0 件の案内（「商品登録へ」）が重複の登録を誘いうる。答えで変わる所: B なら 79 UI-16-D3 の検索の範囲の文と `StockTransferPage` の検索の条件（`queryOverrides`）、Test Plan の U3 に廃番の行、L3-2 に廃番の商品。BIZ（31 §12.6b）は廃番を見ないので変わらない。出所: AI の review（PR #216 の計画の review round 1、Opus の P3）。owner の確認の前に Human Gate の必須にしない（AGENTS.md）。
- 絶対保証の例外と抜け道: 「片方だけが動いた状態は残らない」は、明細ごとの `apply_stock_change` が同じ TX の中で、どの失敗も `?` で TX 全体を戻す（rusqlite の Transaction の drop で ROLLBACK）ことに依る。例外は COMMIT の後の失敗だけで、COMMIT は 1 回。「vT はどの DB でも止まらない」は、CHECK に値を足すだけで既存の行の値の集合が新しい集合に含まれることに依る。例外は 22 §21 の手順 5 の確かめの失敗（作り直しの前後の行数と id の集合の不一致と、DB 全体の `PRAGMA foreign_key_check` の違反。FK は vT の写しと関わりの無い表の違反でも止まる）で、そのときは vT の変更を残さずに止まり（同じ起動で先に COMMIT した migration の分は残る）、作り直しを指示しない文を出す。FK の違反で止まる見込みは低い: v11 を通った DB は vZ（22 §20 の手順 6）で同じ DB 全体の確かめを一度通っており、その後のアプリの書込みは `foreign_keys` を ON にした接続（`db/mod.rs` の `configure_database`）で行う。境界の 4 観点: 安全な整数の範囲は明細の数量が `MAX_LINE_QUANTITY`、在庫が `recompute_stock` の上限で、どちらも既存の定数と 1 か所（金額は無く、明細の数に上限を置かないが和を取る値が無い）。0 は画面と BIZ と DB の CHECK（`quantity > 0`）で拒み、在庫 0 は既存の formatter、在庫の負は stock_warnings で名指す。失敗の時点と順序は DB を見ない検証（手順 0〜2）→ 冪等の再送（4）→ DB の検証（5）→ TX（6〜10）。1 つの記録に個数と長さの明細が混ざるので、数量は明細ごとにその商品の単位で持ち、足し合わせる表示を作らない。
- 残りの危険: 版の番号（#172 より先に本 lane が merge すると版 12 を取り、#172 が 13 に取り直す〈22 §21〉）。取り直しを忘れて main に欠けを作ると後から足した小さい版が当たらない DB ができるが、これは Test Plan の M7（`migrations()` の版が 1 から欠けの無い連番）がどちらの merge 順でもその head の CI で止める（merge の前の helper の確かめには無いので、CI の M7 が唯一の機械の止め。Final Review も M7 を読む、Test Plan の R4 の段落）。M7 が止めないのは、別の build で版を記録した開発・試験 DB（下の Recovery の「版の番号の取り直し」）。誤った振替の戻し方（取消が無いので逆向きの振替で戻すと記録が 2 つ残り、同じ日付に計数のある商品は要再確認になる。過剰側）。逆向きの振替は元の記録と同じ日付・同じ数量で保存する（79 §79.6・§79.8）: 反例（Codex、round 1）は 10/27 の振替 → 10/28 の計数 → 10/29 に今日の日付で逆向き、で、元の日付なら逆向きは計数に入って在庫が計数の値のまま戻り、今日の日付なら計数の後の動きとして在庫に足され、実物を数えた計数の値と食い違う（Codex の例では 700 cm が 1000 cm になる。隔離の実験で `stock_ledger::judge` が前者を `Included`・後者を `NotIncluded` と判定）。元と先の数の釣り合いを確かめないので、入力の誤り（3 m を 30 m）は在庫の誤りとして残り、次の計数で直る（ほかの入出庫と同じ）。
- 判定: ready。正本だけで「何を作るか・なぜ・何を退けたか」が読める（D-143 の Decision・Why・Alternatives、79 の決定表）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-143 (1)(2) | decision-log D-143、31 §12.6b | S2 `create_transfer` | B1・B2 | L3-3 |
| D-143 (3) | D-143、10 SPEC-UNIT-D2・D3、79 UI-16-D4 | S2 の数量の検証、S4 の m の入力 | B5・B6・U5 | L3-2 |
| D-143 (4) | D-143、31 §12.6b の「原価・金額」 | S2（原価を読まず書かない） | B13 | — |
| D-143 (5) | D-143、79 §79.6・§79.8 | S4 の案内 | U12 | L3-4 |
| D-143 (6) / MNT-03-D22 | 22 §21、tracking §14、transaction §11a-11b | S1 `schema_v{N}.rs`、`migration.rs` の登録 | M1〜M7 | — |
| D-143 (7) / UI-16-D1 | 79、52 | S4 route と navigation | U1、C-NAV | L3-1 |
| ADR 横断の約束の表（振替の行） | ADR、31 の D-118 の節の振替の項 | S2（`apply_stock_change` を明細ごとに） | L1〜L4 | L3-5 |
| ADR SPEC-STK-CNT-D9（初期在庫の印） | ADR D9、21 の D-118 の節 | 変えない（`INITIAL_STOCK_SQL`） | L4 | — |
| 10 (a) 1 明細の数量 | 10「入力の上限と安全な整数の範囲」 | S2 | B5 | — |
| 31 §12.6b 手順 0〜10 | 31 | S2 | B3〜B12 | — |
| 31 §12.8 fingerprint | 31 | S2 | B11 | — |
| 21 §10.5a | 21 | S1 `transfer_repo` | R3・R4 | — |
| 44 §23.7a / CMD-13 | 44 | S3 | B14、C-REG | — |
| 44 §23.7 `list_inventory_records` の種別 | 44、21 §10.5a | S1 の `SPECS`、S2 の許可 | R1・R2・R4 | L3-6 |
| 65 TRACE-D14 | 65 | S2 `resolve_movement_source`、S4 の詳細と選択肢 | R5・U12〜U14 | L3-4・L3-6・L3-7 |
| 65 TRACE-D11 | 65、79 UI-16-D11 | S4 | U11 | L3-4 |
| 66 Filters の `transfer` | 66 | S4 | U14 | L3-7 |
| 74 UI-11c-D7・registry | 74 | S2 の detail_json、S4 の label と route | U15・U16 | L3-7 |
| 79 UI-16-D2〜D10 | 79 | S4 | U2〜U10 | L3-2・L3-3 |
| D-052 C23 | 79 UI-16-D10、decision-log D-052（S5） | S4 `invalidation-contract.ts` | U10 | — |
| D-061 / D-064（enum と CHECK の parity） | DB_DESIGN の CHECK 制約方針 | S3 `finite_enum_contract_tests.rs` | E1 | — |
| C-REG（登録と生成） | 本 packet の Scope の S1（`design_compliance_test`）・S3（`lib.rs` の登録と bindings）・S4（routes・navigation）・S5（REQ-210 と traceability） | S1・S3・S4・S5 | bindings・routes・traceability の `--check` | — |
| C-NAV（到達導線） | 79 §79.7、52 | S4 `navigation.ts` | `navigation.test.ts` | L3-1 |

adjacent-contract sweep: 31 §12.2（`apply_stock_change` の引数と前提）は変えずに呼ぶ（行を立てない）。31 §12.6（廃棄）・64・`list_disposals` は変えない（Non-scope）。35・36・73（棚卸し・整合性）は振替の movement を種類で分けないので変えない（L1〜L4 が確かめる）。66 の `StockHistoryTable`・`MovementTable` は種別の label の表を共有するので S4 の label の 1 行で両方に出る（U13）。

## Test Plan

期待値は設計正本（D-143、ADR SPEC-STK-CNT-D1〜D3・D9 と横断の約束の表、tracking §14、transaction §11a-11b、10 SPEC-UNIT-D2・D3 と「入力の上限と安全な整数の範囲」(a)、21 §10.5a、22 §21、31 §12.6b・§12.8、44 §23.7・§23.7a、65 TRACE-D11・D14、66 の Filters、74 UI-11c-D7、79）から写し、実装から写さない。test の名前は実装の Writer が決め（M1〜M7 だけは AC1 の形 `vt_stock_transfer_m{1〜7}_` で始める）、行の ID（Contract の列の先頭）を test の名前か comment に入れる。fixture は合成の商品と値だけ。Would fail if... の列は実装か assertion に注入する mutation で、Writer は注入して red になることを実装の報告に書く。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| M1: 22 §21 手順 3・4 | 列の取り違え・id の振り直し | schema（Rust） | vT 空の DB と直前の版の DB で成功し、全 movement の全列と id が同じ | 写す列の並びを 1 つずらす、`id` を写さない |
| M2: 22 §21 手順 4 | `sqlite_sequence` を戻さない | schema | 末尾の行を消した fixture で vT の後の次の INSERT の id が前の `sqlite_sequence` + 1 | 手順 4 の `sqlite_sequence` の書き戻しを外す |
| M3: 22 §21 手順 3・4 | CHECK・index の欠け | schema | vT の後に `transfer`・`transfer_record` の行が入り、`convert` が拒まれ、3 つの index と 2 つの表の index がある | CHECK に値を足さない、index を作り直さない |
| M4: transaction §11a-11b | 表の CHECK の欠け | schema | `transfer_items` の `direction='x'`・`quantity=0`・同じ記録の同じ商品の 2 行目、`transfer_records` の空の key と形の違う日付が拒まれる | CHECK か UNIQUE を書き落とす |
| M5: 22 §21 手順 5 | 確かめの失敗で中途半端に残る、画面が作り直しを促す | schema | 手順 5 の確かめが失敗する状態（test の中で作る。行数と id の集合の不一致、ほかの表の FK の違反の 2 つ）で schema・値・版が何も変わらず `MigrationFailed`。message のうち詳細の部分（末尾の `: inventory_movements（…）` の括弧の中）を除いた文の全体が、22 §21 手順 5 の vT の文の同じ部分と一字一句一致する（そのため `DB は変わっていません`・`前の版のアプリで開いて`・`開発者に知らせて`・`作り直し` を含まない。文は、同じ起動で先の migration が COMMIT した場合にも正しい形。22 §21 の「止まった後の進め方」） | 確かめを外す、ROLLBACK しない、§20 の手順 6 の文（`REDO_GUIDE`）を使う、旧い文に戻す（`DB は変わっていません`・`前の版のアプリで開いて` の文、知らせる先の `開発者`） |
| M6: 22 §21 手順 1 | 値のある DB で止まる | schema | 入庫・返品交換・手動販売・廃棄・初期在庫・Z004（`phase = 1`）の movement と計数・棚卸しを持つ DB で成功し、写しが vT の前と同じ | 止める検査を足す、値を変える |
| M7: 22 §21 MNT-03-D22 | 版の欠け（番号の取り直しの忘れ） | unit（`migration.rs`） | `migrations()` の版の列が `1..=migrations().len()` と等しい（1 から欠けの無い昇順の連番。重複も落ちる） | N − 1 を飛ばして N を登録する |
| B1: 31 §12.6b 手順 8 | 符号・参照・業務日の誤り | integration（Rust、in-memory DB） | 元 A（個）3・先 B（個）6 で、A に -3・B に +6 の `transfer` の movement、`reference_type='transfer_record'`、`reference_id=record_id`、`business_date=transfer_date`、`phase=0`、写しが A -3・B +6 | 符号を逆にする、業務日に今日を入れる、参照を外す |
| B2: D-143 (1)、31 §12.6b | 複数行の明細の扱い | integration | 元 2 行・先 1 行の記録で 3 つの movement と 3 つの明細、`stock_after` が各商品の求め直しの値 | 最初の行だけ処理する |
| B3: 31 §12.6b 手順 1 | 片側 0 行が通る | integration | 減らす明細だけ・増やす明細だけ・空の items が `ValidationFailed`（各文）で、4 つの表の行数が変わらない | 手順 1 の検査を外す |
| B4: 31 §12.6b 手順 5 | 同じ商品の 2 回目が通る | integration | 同じ商品を減らす側と増やす側に入れる、同じ側に 2 回入れる、のそれぞれが `明細{n}: 同じ商品が2回入っています（{code}）` で何も書かない | 重複の検査を外す（DB の UNIQUE が先に当たると別の error になる） |
| B5: 10 (a)、31 §12.6b 手順 5 | 上限と 0・負 | integration | 数量 `9999999` は通り、`10000000`・0・-1 は各文で止まる（減らす側と増やす側の両方で） | 片側だけに上限を掛ける |
| B6: SPEC-UNIT-D2 | 長さの商品の数量の単位 | integration | 長さの商品 300（cm）を減らすと写しが 300 減る（m への換算を BIZ でしない） | BIZ で 100 倍・100 分の 1 にする |
| B7: 31 §12.6b 手順 2 | 日付の形と暦 | integration | `2026-4-7`・`2026-02-30` が各文で止まる | `validate_business_date` を呼ばない |
| B8: 31 §12.6b 手順 5 | 無い商品 | integration | 無い商品コードで `NotFound`、何も書かない | TX の中で見つけて一部を書く |
| B9: 31 §12.6b 手順 8・D-118 | TX の中の失敗で一部が残る | integration | 増やす商品の在庫が上限を越える記録（先に元の movement を書く順）で `在庫数が大きすぎます`、4 つの表の行数と元の商品の写しが呼ぶ前と同じ | TX を明細ごとに分ける |
| B10: 31 §12.6b 手順 0 | 冪等キーの検査 | integration | 空・空白だけ・256 文字の key が各文で止まる | key の検査を外す |
| B11: 31 §12.6b 手順 3・4、§12.8 | 冪等の再送と衝突 | integration | 同じ key・同じ内容（明細の順を入れ替えたものを含む）で `idempotent_replay: true` と同じ `record_id`、行が増えない。同じ key・違う内容で `IdempotencyConflict` | fingerprint に向きを入れない（向きだけ違う内容が replay になる） |
| B12: 31 §12.6b 手順 8・9 | 警告と操作ログ | integration | 元の在庫が負になると `stock_warnings` に商品コードの昇順・重複なしで入る。操作ログが `transfer_create` の 1 行で detail_json に `record_type: "transfer_record"` と `record_id` | 警告を落とす、detail に 2 field を書かない |
| B13: D-143 (4) | 原価が変わる | integration | 保存の前後で元と先の `products.cost_price_centi` と `price_history` の行数が同じ | 先の原価を書く |
| B14: 44 §23.7a | CMD の error の写像 | unit（CMD） | `ValidationFailed`→`validation`、`NotFound`→`not_found`、`IdempotencyConflict`→`idempotency_conflict` | 写像を落とす |
| L1: ADR D3 の理由 2 | 同じ日付の計数で要再確認にならない | integration | 先に B に日付 D の既定の計数 5、日付 D の振替（B を減らす 1）で B が要再確認（範囲 4〜5）、A は要再確認でない | 振替の movement を求め直しで落とす |
| L2: ADR D3 の判定表（入出庫の行） | 計数より前・後の日付の扱い | integration | 計数の日付 D、振替の日付 D-1 は計数に入り（写し = 計数値）、D+1 は入らない（写し = 計数値 + 振替） | 業務日でなく保存の時刻で判定する |
| L3: ADR D3（fresh な計数） | fresh な計数との前後 | integration | 振替の後に「入れ直す」（fresh）を保存すると、同じ日付の振替は入る（要再確認でない） | movement の id のカーソルの比べを振替だけ外す |
| L4: ADR D9 | 初期在庫とみなされる | integration | 振替の movement を持つ商品の求め直しで、振替の movement が `INITIAL_STOCK_SQL` に当たらない（初期在庫の印が偽）。`biz::stock_ledger` の diff が空（AC3） | `reference_type` を NULL で書く |
| R1: 21 §10.5a、44 §23.7 | 種別で出ない | integration | `list_inventory_records` の `record_type: "transfer_record"` で振替だけ、`"all"` に含まれる、件数（`item_count`）は明細の数、`detail_route` が `/inventory/transfer/records/{id}` | `SPECS` に行を足さない、許可の `matches!` に足さない |
| R2: 65 TRACE-D14 | 増やす側の商品で当たらない | integration | 商品の絞り込みに先の商品のコードで当たる、部門の絞り込みも同じ | 明細の片側だけを検索の母集団にする |
| R3: 21 §10.5a | 詳細の形 | integration | `get_transfer_record` が減らす明細を先、明細の id の順、単位つき、関連 movement 2 つ（`source` が補われる） | 並びを id だけにする、source を補わない |
| R4: 21 §10.5a、44 §23.7 | 無い id・未対応の種別 | integration | 無い id で `not_found`、`record_type: "unknown_type"` は今のまま `validation` | NotFound の変換を落とす |
| R5: 65 TRACE-D2・D14 | 元記録の link | unit | `resolve_movement_source(TransferRecord, 7)` が label `詰め替え・切り分け #7`・route `/inventory/transfer/records/7` | arm を足さない |
| E1: D-061・D-064 | wire と CHECK の値の集合のずれ | unit（`finite_enum_contract_tests.rs`） | `MovementType::Transfer`・`ReferenceType::TransferRecord`・`TransferDirection` の 2 値が wire の文字列と DB の CHECK の値と一致 | CHECK か enum の片方だけに値を足す |
| U1: 79 UI-16-D1、C-NAV | 到達できない | integration（frontend、router） | サイドバーの入出庫の廃棄・破損の次に「詰め替え・切り分け」、`/inventory/transfer` で題と active | navigation に足さない、順を違える |
| U2: 79 UI-16-D2 | ad hoc invoke | unit | `commands.createTransfer`・`commands.getTransferRecord` の mock だけを通る | `invoke` を直に呼ぶ |
| U3: 79 UI-16-D3 | 表の扱い | component | 2 つの表に Enter で足せ、同じ表の再追加で数量 +1、もう片方の表にある商品は足されず案内が出る、payload の `direction` が表に対応し items の順が減らす → 増やす | 片方の表の追加を他方に入れる、案内を出さない |
| U4: 79 UI-16-D4 | 片側 0 行で保存できる | component | どちらかの表が 0 行で保存ボタンが disabled、理由の文が出る | disabled の条件を片側だけにする |
| U5: 79 UI-16-D4 | 数量の検証と単位 | unit（`transfer-request.ts`） | 0・負・形の誤り・上限越えを止め、`m` の商品の `1.5` を `150` で送る、個数の商品の小数を止める | 換算の純関数を通さない |
| U6: 79 UI-16-D6 | key の扱い | component | 失敗の後の同内容の再試行で key を再利用し、成功・リセット・編集の後は新しい key | 再試行のたびに key を作る |
| U7: 79 UI-16-D7 | 保存中の検索の結果 | component | 保存中は 2 つの追加欄が disabled、保存 event と競合して完了した検索の結果を反映しない | lock を片方の欄だけで読む |
| U8: 79 UI-16-D8 | 保存結果の欠け | component | record_id、減らした・増やした一覧（数量と単位）、warning、replay が読め、ページ先頭へスクロールする | 一覧の片方を出さない |
| U9: 79 UI-16-D9 | recent list | component | `listInventoryRecords` を `record_type: "transfer_record"`・page 1・per_page 10 で呼び、空・失敗・成功を出す、「すべての履歴を見る」が `/inventory/records?recordType=transfer_record` | 種別を渡さない |
| U10: 79 UI-16-D10、D-052 | invalidation の過不足 | integration（`invalidation-contract.meta.test.ts`） | 保存成功の実呼出しの集合が独立に写した C23 の oracle と完全一致（順は問わない、重複を検出） | `disposals.root()` を含める、`stocktake.recheckList()` を落とす |
| U11: 79 UI-16-D11、TRACE-D11 | 戻り先 | integration（router） | 保存結果と recent list の「詳細を見る」が `?returnTo=%2Finventory%2Ftransfer` を付けて詳細へ進む | returnTo を付けない |
| U12: 79 §79.8 | 詳細の欠け | component | 詳細に 2 つの表（単位つき）、関連在庫変動、取消の代わりの案内（79 §79.8 の文。「この記録と同じ日付・同じ数量で」を含む）、戻り先の fallback `/inventory/records` | 片方の表を出さない、案内を出さない、案内から日付・数量の条件を落とす |
| U13: 66、65 TRACE-D14 | 種別の label | unit（`movement-formatters.test.ts`） | `formatMovementType("transfer")` が「詰め替え・切り分け」 | label を足さない（型の error にもなる） |
| U14: 66 Filters、65 TRACE-D14 | 選択肢の欠け・順 | unit（`search.test.ts`、`StockMovementsPage.test.tsx`） | `MOVEMENT_TYPE_OPTIONS` と `INVENTORY_RECORD_TYPE_OPTIONS` の廃棄・破損の次に「詰め替え・切り分け」、選ぶと `type=transfer` / `recordType=transfer_record` | 選択肢を足さない、順を違える |
| U15: 74 registry | 操作ログの label | unit（`operation-type-labels.test.ts`） | `transfer_create` が入出庫の「詰め替え・切り分け記録」 | label を足さない |
| U16: 74 UI-11c-D7 | 関連記録の link | component（`OperationLogsPage.test.tsx`） | detail_json の `record_type: "transfer_record"`・`record_id: 7` で `/inventory/transfer/records/7` へ進む link | 許可リストに足さない |
| T1: 既存の movement・記録の一覧・在庫変動履歴・migration の test（`rg -n -e 'fn test_list_inventory_records' -e 'fn test_list_movements' -e 'fn test_v11' src-tauri/src` で数える。2026-10-29 に `1c925027` で 40 行） | 振替を足して既存の種類の振舞いが変わる | 既存（Rust） | 既存の test を期待を変えずに PASS。最新の版を期待する test は `app_max_version()` と比べる形（22 §21 の「テスト」） | 既存の種類の値・順・件数を変える |
| T2: `search.test.ts` の 2 つの `toEqual`、`StockMovementsPage.test.tsx` の `REMAINING_MOVEMENT_TYPE_OPTIONS`、`operation-type-labels.test.ts`、`invalidation-contract.meta.test.ts` の entry の一覧、`unsaved-changes-guard-sweep.test.ts` の画面の一覧、`navigation.test.ts` | 振替の 1 項目の足し漏れ・ほかの項目の順の変化 | 既存（frontend） | 振替の 1 項目を正本の位置に足す（ほかの項目と順は変えない） | 項目を足さない、ほかの項目の順を変える |
| T3: `inventory_repo.rs` の `as_str` の test、`finite_enum_contract_tests.rs` | 新しい値の parse・as_str の欠け | 既存（Rust） | 新しい値の 1 行を足す | arm を足さない |

oracle の anchor: U14 の期待は 66 の Filters と 65 TRACE-D14 の語（「詰め替え・切り分け」）から写す。U10 の oracle は 79 UI-16-D10 の 8 つの key の列から写し、production の `invalidation-contract.ts` を import しない。M1 の期待の列は 22 §21 の手順 3 の「今の 13 列」を `PRAGMA table_info` の結果と比べ、vT の前の版の表から写す。

状態の遷移（State Lifecycle）:

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 作成画面の form | 日付は今日、2 つの表は空、保存は disabled | 保存中は入力と 2 つの追加欄が disabled | 保存結果、form は操作不可 | C23 の query | 最近の記録と在庫の画面の query | 「続けて」で空の form と新しい key | アプリの再起動で空の form | Alert、入力と key を保持 | 同内容は同じ key、編集の後は新しい key | U3〜U9 |
| 振替の記録 | 無い | 1 つの TX | 記録・明細・movement・操作ログ | — | 詳細と一覧が読む | 詳細を開き直すと同じ内容 | 再起動の後も同じ | 何も残らない | 冪等の再送は同じ記録 | B1〜B12、R3 |
| 商品の写し（在庫・要再確認） | 振替の前の値 | — | 求め直した値 | 商品の画面・在庫変動履歴の query | — | — | — | 呼ぶ前の値 | — | B1・B9・L1〜L4 |

移す pattern（Adjacent Pattern Audit）:

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| 作成画面の商品追加（Enter・候補・フォーカス戻し・async lock、UI-05-D5・D6・D15・D16） | `src/features/disposal/DisposalPage.tsx`・`receiving`・`manual-sale`・`return-exchange` の各 page | `StockTransferPage` の 2 つの追加欄（lock は 1 つを共有） | 種別・理由・原価の列は持たない（D-143 (4)） | U3・U7 |
| 冪等キー（UI-05-D9） | 4 つの作成画面 | `StockTransferPage` | — | U6 |
| recent list と returnTo（UI-05-D12・D17、DSR-18） | 4 つの作成画面と記録の詳細 6 画面 | `StockTransferPage`・`TransferRecordDetailPage` | 一覧は `listInventoryRecords` で読む（種類ごとの一覧の command を作らない） | U9・U11・U12 |
| 未保存の離脱ガード（UI_TECH_STACK §6.11） | `unsaved-changes-guard-sweep.test.ts` の一覧 | `StockTransferPage` | — | T2 |
| 種別の label の表（`Record<MovementType, string>`） | `movement-formatters.ts`（`MovementTable` と `StockHistoryTable` が共有） | 1 行 | — | U13 |
| 記録の種別の一覧（`RecordSpec`） | `disposal_repo.rs` の `SPECS` 6 行 | 1 行 | — | R1・R2 |
| 操作ログの関連記録の許可リスト | `OperationLogsPage.tsx` の表、74 | 1 行 | — | U16 |
| `inventory_movements` の作り直し（vZ） | `schema_v11.rs` | `schema_v{N}.rs` | 止める検査は持たない（22 §21 手順 1） | M1〜M6 |

negative と境界:

- missing input: 片側 0 行（B3・U4）、空の key（B10）。invalid input: 数量 0・負・上限越え・形の誤り（B5・U5）、日付の形と暦（B7）、`direction` の一覧外（serde の拒否、E1）。duplicate / ambiguous input: 同じ商品の 2 回目（B4・U3）、同じ key の違う内容（B11）。unknown reference: 無い商品（B8）、無い記録の id（R4）、未対応の種別（R4）。write failure: TX の中の失敗（B9）、vT の確かめの失敗（M5）。外部の依存と dry-run は無い。
- 境界: 数量 `9999999` / `10000000` と最小 1（B5）、在庫の上限（B9）、日付の既定は今日（U3）、`settlement_no` は NULL（B1）、片側 0 行と空の items（B3）、`direction` の 2 値（E1）、詳細の `status` は `active`（R3）、`TransferDirection` と `MovementType`・`ReferenceType` の union の広がり（AC4）、DB の `direction` TEXT と CHECK（M4）、producer / consumer（`list_inventory_records` → 入出庫履歴、`list_movements` → 在庫変動履歴。R1・U14）、round-trip の token（`/inventory/records?recordType=transfer_record`、`returnTo`。U9・U11・U14）、m の小数 2 桁 → cm（U5）、`"transfer"` の label が TS の `Record` の全 variant を網羅（U13）。
- 互換: vT の前の版の DB の全行（M1・M6）、`transfer` の行（M3）、既存の選択肢の順は変えず廃棄・破損の次に足す（U14）。新しい optional の field は無い。

実行: targeted tests は Rust の `cargo test vt_stock_transfer_` と `cargo test vt_stock_transfer_ -- --list`（M1〜M7。AC1 の本数と 7 つの印の確かめ）・`cargo test db::migration`・`cargo test transfer`・`cargo test list_inventory_records`・`cargo test finite_enum`、frontend の `npx vitest run src/features/stock-transfer src/features/inventory-records src/features/stock-movements src/features/operation-logs src/config src/lib/invalidation-contract.meta.test.ts src/features/products/search.test.ts src/hooks/unsaved-changes-guard-sweep.test.ts`。main の配線は作成画面 → 保存 → 記録の詳細 → 入出庫履歴 → 在庫変動履歴の link の通り（U11・U12・R5）と、生成物の再生成（bindings・routes・traceability）。L3 があるので、Writer の完了に owner の native build の前の `cargo check --release` を含める（CI の gate ではない）。

R4: Final Review は独立 2 本の Contract Audit（Double Audit。`docs/DEV_WORKFLOW.md` Contract Audit (R3/R4)）を同じ head で行い、どちらも vT の写し方（M1〜M6）、版の欠けの止め（M7。実際の `migrations()` の登録の列と番号の取り直し）と片方だけが動かない TX（B9）を設計正本から直接読んで確かめ、互いの結果を見ずに下の Recovery の手順が 22 §16・§21 の「回復」と食い違わないかを確かめる。`r4` の承認は、2 本の Final Review の finding を裁定し、L3 が PASS した後に、下の Recovery の手順を添えて owner に頼む。

## L3

L3 は lane の runtime の変更が止まった後に 1 回だけ頼む（`docs/DEV_WORKFLOW.md` L3）。L3 の build の vT は、最初から L3 を頼む head の base の最大 + 1（N。22 §21）で書いてある。L3 の後に base が進んで（並走の lane の migration が先に merge して）N が変われば、番号の取り直しは runtime の変更で、取り直す前の build の L3 は取り直しになり、その build で vT を記録した開発・試験 DB は下の Recovery の「版の番号の取り直し」の扱いになる。各段の PASS / FAIL だけを返してもらう 1 本の台本（作成画面 → 保存結果 → 記録の詳細 → 入出庫履歴 → 在庫変動履歴 → 操作ログ）で、画面の目視を同じ 1 回に含める。owner の判断事項 (i)〜(iii)（Design Readiness）を同じ依頼に束ねる。

起点の DB: owner の PC の今の開発・試験 DB をそのまま使う（vT は値のある DB でも止まらない。作り直しは要らない。ただし vT を含む build を初めて起動する前に下の Recovery の「写し方」で vT の前の snapshot を作り、vT の番号と違う番号の build でその DB を開く前にはその snapshot に戻す）。Coordinator が L3 の依頼と同時に、合成の商品 CSV（CP932）を渡し、owner は商品の一括インポートで入れる（`docs/DEV_WORKFLOW.md` L3 の取込み fixture の規則）。CSV の行: L3-2〜L3-4 の個数の商品 1〈`L3-TR-01`〉と長さの商品 1〈`L3-TR-M1`、初期在庫 10 m〉（既に DB にあれば、preview の重複の行を「スキップ」のまま取り込み、使い回してよい）と、L3-5 だけに使う個数の商品 2〈`L3-TR-5A-{T}`（減らす側、初期在庫 10）・`L3-TR-5B-{T}`（増やす側、初期在庫 0）。`{T}` は Coordinator が CSV を作った日時 `YYYYMMDDHHMM` で、実行ごと〈L3 の取り直しを含む〉に新しくする〉。L3-5 の 2 商品は実行ごとに未使用の商品コードなので、既存の計数・業務の movement（初期在庫の外）が無い起点になる。owner は取込みの preview で 2 商品が新規登録候補の行（重複の行でない）であることを見る。重複の行なら取り込まず、Coordinator が `{T}` を新しくした CSV を渡し直す。L3-5 の計数は owner が棚卸しの画面で `L3-TR-5A-{T}` に今日の日付で「数を保存」する（既定の計数。下の L3-5 の到達経路）。L3 の前に下の Recovery の vT の予行を済ませる。

| L3 | 到達経路 | 入力物 | 観測可能な合格基準 | 依存する既知 backlog |
|---|---|---|---|---|
| L3-1 | サイドバー → 入出庫 → 詰め替え・切り分け | — | 廃棄・破損の次に「詰め替え・切り分け」があり、題と説明が読める | — |
| L3-2 | 同じ画面 | `L3-TR-M1`（減らす、1.5 m）、`L3-TR-01`（増やす、3） | 2 つの表に Enter で足せ、長さの商品の数量を m で入れられ、現在庫が単位つきで読める。増やす商品の欄に `L3-TR-M1` を入れると行が足されず案内が出る | — |
| L3-3 | 「詰め替え・切り分けを保存」 | L3-2 の入力 | 保存中は入力が止まり、保存結果に記録の番号、減らした 1.5 m・増やした 3 個、行ごとの「在庫を見る」が読める | — |
| L3-4 | 保存結果の「詳細を見る」→「前の画面へ戻る」 | — | 詳細に減らした商品と増やした商品の 2 つの表と関連在庫変動、取消の代わりの案内。戻ると作成画面 | — |
| L3-5 | (1) サイドバー → 棚卸し。進行中の棚卸しが無ければ「棚卸しを開始する」で始める。検索欄に `L3-TR-5A-{T}` を入れ、計数 5 を「数を保存」（既定の計数。「今の数を入れる」〈fresh〉は使わない）。(2) 詰め替え・切り分けで、日付は今日のまま、`L3-TR-5A-{T}` を減らす商品（1）・`L3-TR-5B-{T}` を増やす商品（1）にして 1 回だけ保存（この 2 商品をほかの段で使わない）。(3) サイドバー → 棚卸しで画面を開き直す | 起点: 上の CSV で入れたばかりの `L3-TR-5A-{T}`・`L3-TR-5B-{T}`（計数と業務の movement が無く、movement は 5A の初期在庫の 1 つだけ。5B は初期在庫 0 なので movement が無い）。`L3-TR-5A-{T}`（計数 5、減らす 1）、`L3-TR-5B-{T}`（増やす 1） | native の画面で、上部の「数え直しが必要な商品」の一覧の `L3-TR-5A-{T}` の行の商品名・範囲（4〜5 個、単位つき）・理由が、切れ・重なり・横の scroll なしに読める。範囲 4〜5 はこの起点で決まる: 求め直し（`biz::stock_ledger` の `derive_ledger`）は効く計数を土台にし、初期在庫の movement（一括インポートの初期在庫は `receiving`・`reference_type` NULL〈`biz/product_service.rs` の `commit_import`〉で `INITIAL_STOCK_SQL` に当たる）は窓を持たず計数に入り、同じ業務日の業務の movement は「決まらない」で負なら下限・正なら上限に足す。起点の計数は 5 の 1 つ、同じ日の movement は振替の −1 の 1 つだけなので、下限 4・上限 5（round 2 の Codex の `derive_ledger` の隔離の実行の表の「計数 5 → 同日の振替 −1 → 4〜5」の行。既存の同日の計数 9 があると 4〜9、同日の振替 −1 が 2 つあると 3〜5 になり、未使用の商品コードの起点はこの 2 つを除く）。一覧に出ること自体は自動 test（L1）が確かめ、ここでは Windows の WebView での表示と、詰め替え・切り分けの保存から棚卸しの画面へ移る操作を見る | — |
| L3-6 | 入出庫履歴で種別「詰め替え・切り分け」 | — | L3-3・L3-5 の記録が出て、行から詳細へ進める | — |
| L3-7 | `L3-TR-M1` の在庫変動履歴、操作ログ | — | 種別「詰め替え・切り分け」・元記録の link の行、種別の絞り込みの選択肢。操作ログに「詰め替え・切り分け記録」と関連記録の link | — |

## Recovery

前例の b1 の packet（`docs/archive/plans/2026-10-22-stocktake-count-runtime-b1.md` の Risk の Rollback / recovery notes）と [22](../function-design/22-mnt-migration.md) §16 の「回復」・§21 の「止まった後の進め方」と「回復」に合わせる。

- 写し方（vT の前の snapshot。下の予行・置き換え・Data Safety が全部これを使う）: vT を含む build を初めて起動する前に、vT を知らない今の版のアプリでその DB を開き、バックアップ画面の「今すぐバックアップを作成」を押す（owner の 1 操作。今の版のアプリは vT を書く head の base〈`origin/main`〉の build。22 §16 の「回復」の手順 1 と同じ backup）。今の code の `create_backup`（`src-tauri/src/mnt/backup.rs`）は、`VACUUM INTO` で WAL の確定済みの変更を取り込んだ 1 つの file を正式名へ直に書き、大きさを読み、操作ログを書くだけで、`quick_check` も検査の後の公開もしない（71 §71.4 の手順 4a・4b〈D-114〉は未実装）。そのため snapshot の確かめは次の Coordinator の `integrity_check` が担う。Coordinator はその file（画面の「現在の保存先」の `inventory_backup_{YYYYMMDD}_{HHMMSS}.db`）を消さずに repo の外の日付の付いた folder へ写し、写しを読取りだけで開いて、版（`schema_versions` の `MAX(version)`）が base の build の `migrations()` の最大（N − 1。vT の前の版）と等しく、`PRAGMA integrity_check` が `ok` であることを確かめる（22 §16 の「回復」の手順 4 と同じ確かめ。command は [migration-recovery-runbook](../migration-recovery-runbook.md) の手順 4）。版が N − 1 より小さい、または `integrity_check` が `ok` でなければ、base の build で snapshot を作り直し、合うまで vT を含む build を起動しない。版が N − 1 より大きい（別の lane の build が base に無い版を記録した DB）なら、この DB で snapshot を作らず、先にその lane の build で開く前に作った backup に戻してから作り直す（22 §21 の「版の番号の取り直しと開発・試験 DB」）。snapshot を作ってから vT を含む build を起動するまでの間は、今の版のアプリで記録を入れない（入れた記録は snapshot に無く、戻すと失われる）。DB の file（`.db`）だけを写す形は使わない: WAL に残った確定済みの行が写しから欠け、写しの `integrity_check` も `ok` になるので、予行でも気づけない（round 1 の Codex の合成 DB の再現）。
- vT が途中で止まったとき: vT は 1 つの TX で、手順 5 の確かめの失敗と TX の中のどの失敗も、表・値・版を全部戻し、版を記録しない（22 §21 の手順 5・6、MNT-03-D1 の共有の helper）。vT の変更は残らない。`migrate` は migration ごとに COMMIT するので、同じ起動で vT より前の migration が COMMIT していれば、その版と変更は残る。vT がその起動の最初の migration なら、版が変わらないので vT の前の版のアプリでそのまま開ける（同じ起動で先に別の migration が COMMIT していれば開けないので、上の snapshot に戻す。22 §21 の「止まった後の進め方」）。起動の画面は今の `DatabaseInit` の文の後に、22 §21 の手順 5 の vT だけの message（この更新の変更は戻した・先に済んだ別の更新は残る・起動し直しても止まるときは管理者に知らせる〈前置きの `DatabaseInit` の文と同じ知らせる先〉・前の版で開けないときは管理者が更新の前の backup に戻す。作り直しを指示しない。文の正本は 22 §21 の手順 5）を添えて出す。直し方は code の修正で、DB は作り直さなくてよい。
- vT の後に戻したいとき: vT を COMMIT した DB は vT を知らない旧版のアプリで開けない（`SchemaNewerThanApp`、22 §16 の「回復」）。アプリの中の復元（71 §71.7）では旧版へ戻せないので、管理者がアプリの外で、DB を上の snapshot に置き換え（[migration-recovery-runbook](../migration-recovery-runbook.md) の手順。22 §16 の「回復」の手順 2〜7 が正本で、今の DB の file は消さずに移す）、戻す先の build で開く。snapshot の後に入れた記録（vT の後の振替を含む）は戻らない。置き換えの理由は 3 つで、証拠は理由ごとに分ける。どの理由でも、対象の DB の path、回復元（上の snapshot の写しの path）、戻らなくなる記録（snapshot の後に入れた記録。操作ログの `created_at` が snapshot の時刻より後の行で数える）を名指す。
  - (1) 写しの誤り（finding による）: DEV_WORKFLOW Review Rules の不可逆 finding の 4 項目を書いた finding があるときだけ行う。`actual harm path` = vT の前からある行（snapshot にある id）が今の DB で違う。`affected candidate` = 対象の DB と回復元の組。`non-destructive revalidation` = 置き換えの前に、アプリを閉じた今の DB を `sqlite3 -readonly` で開いて回復元を `ATTACH ... AS before` し、before → 今の DB の向きだけの差分 `SELECT COUNT(*) FROM (SELECT <比べる列> FROM before.inventory_movements EXCEPT SELECT <比べる列> FROM inventory_movements)` が 1 以上（逆の向きは vT の後に足した行が必ず出るので使わない）。比べる列は 13 列から、vT の前からある行でも正当に変わる列を外した 12 列（`id`・`product_code`・`movement_type`・`quantity`・`stock_after`・`reference_type`・`reference_id`・`note`・`created_at`・`business_date`・`phase`・`settlement_no`）。外す列は `is_voided` だけ: 今の code で前からある行を書き換えるのは商品別売上 CSV の取込みの取消（`db/sales_repo.rs` の `void_movements_by_reference`、`csv_import` の行の `is_voided` を 1 に）だけで、`stock_after` を書く `update_movement_stock_after`（`db/inventory_repo.rs`）は同じ TX で足した行だけを書き、行を消す経路は無い（S1 の前に `rg -n -i 'update\s+inventory_movements|delete\s+from\s+inventory_movements' src-tauri/src` で、test と migration の外に別の経路が足されていないかを確かめ直す）。`blocker reason` = 在庫の記録の原本が vT で黙って変わり、使い続けると誤りが広がる。
  - (2) 版の番号の取り直し: 下の項。証拠は build と schema の非互換（版の比べ）: 読取りだけで DB の `schema_versions` の `MAX(version)` と `sqlite_master` の `transfer_records` の有無（vT を当てたか）を読み、vT を当てた DB の `MAX(version)`（その DB が vT を記録した番号）が、開く build の vT の番号と違うこと（違う番号の build で開くと `SchemaNewerThanApp` で開けないか、同じ番号の別の migration が記録済みで当たらない）。差分の行は要らない。
  - (3) 機能を戻す（finding によらない owner の判断）: owner が vT を含まない build へ戻すと決めたときだけ行う。証拠は (2) と同じ版の比べ（DB の版が戻す先の build の `migrations()` の最大より大きい）。正常な vT は行を変えないので (1) の差分は 0 で、それを求めない。
  開発・試験 DB は作り直してもよい（D-123 (1)）。
- 版の番号の取り直し（L3 の後に vT の番号が変わるとき）: vT の番号と違う番号の build（main で取り直した後の build・#172 の L3 の build）でその DB を開く前に、上の snapshot に戻す（22 §21 の「版の番号の取り直しと開発・試験 DB」。戻さずに開くと、vT を版 N で記録した DB は N より小さい版までの build で開けず、後から版 N を取った別の migration は記録済みなので当たらない）。逆向きも同じ: 別の lane の build が base に無い版を記録した DB は、その版が base の最大より大きいので、上の「写し方」の版の確かめで止め、先にその lane の build で開く前に作った backup に戻す。番号の取り直しは runtime の変更なので L3 の取り直しになる（`docs/DEV_WORKFLOW.md` L3）。戻した後の DB は作り直さずに使い続けられる（vT の前の DB なので、取り直した build で vT が当たり直す）。
- merge の後に機能を戻すとき: revert の PR で code を戻す。vT は版を記録しているので、revert した版のアプリは vT の後の DB を開けない（上の項と同じく DB を戻すか作り直す）。そのため戻すより修正の PR で直すのを既定にする（b1 の packet と同じ判断）。
- vT の予行（L3 の前、Coordinator）: 上の snapshot をもう 1 つ写し（snapshot 自身は変えない）、その写しに vT を含む build を当てる。当て方は 1 つに決める: Coordinator の WSL の作業 checkout（L3 を頼む head）で、起動と窓を閉じる操作だけを owner が sandbox の外の WSL の terminal で行い（Coordinator の Bash の tool からは起動しない: その sandbox は unix socket を作れず〈2026-10-31 の観測〉、GUI の窓を出せない。Coordinator は写しを置く・止める・読む・比べるを受け持ち、起動の command は owner に貼る形で渡す。owner の手は予行の 1 回で、L3 の依頼と別の回）、debug build を `npm run tauri dev`（`package.json` の `tauri` script = Tauri CLI。WSL での GUI の起動は `docs/DEV_SETUP_CHECKLIST.md` §4.5 の `cargo tauri dev` → GUI ウィンドウの記録）で 1 回起動し、Linux の app_data_dir（`~/.local/share/com.inventory-system.desktop/inventory.db`。identifier `com.inventory-system.desktop` は `src-tauri/tauri.conf.json`、path は `src-tauri/src/lib.rs` の setup の comment と `prepare_database_with_init` の `app_data.join("inventory.db")`）に置いた写しに当てる。起動の前に、その folder に既に `inventory.db`（と `-wal`・`-shm`）があれば消さずに日付の付いた別の folder へ移す。写しを置いた後、起動の前に、その写し（snapshot 自身でない方）の自動バックアップを止める: `sqlite3 <写し> "UPDATE app_settings SET value='0' WHERE key='backup_enabled'; SELECT changes();"` が `1` を出し、続く `sqlite3 -readonly <写し> "SELECT value FROM app_settings WHERE key='backup_enabled'"` が `0` を出す。どちらかが違えば起動しない。止める理由: debug の起動は DB の準備の後に必ず `run_startup_auto_backup`（`src-tauri/src/lib.rs`）を呼び、画面の `useAutoBackupCheck` も同じ `check_auto_backup`（`src-tauri/src/mnt/backup.rs`）を呼ぶ。保存先は `resolve_backup_dir` が `app_settings` の `backup_path` から決め（空なら `{app_data}/backups`）、owner の Windows の DB の写しで `backup_path` が Windows の path なら、Linux では相対 path になって cwd（`src-tauri/`）の下に vT の後の DB の写しを書きうる。`check_auto_backup` は値が文字列 `'1'`（`schema_v1.rs` の既定）のときだけ動くので、`'0'` にすれば保存先の値に依らず 2 つの経路とも何も書かない（`backup_path` を書き換える形は、書き換え先を選ぶ段が増え、その先に vT の後の DB の写しが残る）。同じ値を `'0'` にする先例は 74 §74.15.2。予行の写しは捨てる DB なので元の値に戻さない。`app_settings` は下の比べ（(0) の版・2 表・CHECK、(a) の `inventory_movements`、(b) の行数・`MAX(id)`・`sqlite_sequence` の `inventory_movements` の値）に入らないので、この変更で比べの結果は変わらない。画面が出たら何も記録せずに閉じる。owner の Windows の data folder の file は置き換えない。debug build の起動は `src/lib/bindings.ts` を書き出すので、起動の後に `git status --short` が空であること（bindings が再生成済みの head であり、作業 checkout に DB の写しが無いこと）も確かめる。アプリの終わりの証拠は、owner が窓を閉じた後に `npm run tauri dev` の command が終わって prompt に戻ったこと（owner の知らせ。tauri の CLI は、窓を閉じてアプリが終わると `beforeDevCommand` を止めて終わる〈tauri-cli 2.11.1 の `dev.rs` の `on_app_exit`〉。止め方は窓を閉じる 1 つにし、Ctrl+C で止めない）。知らせが来なければ読まずに待つ。process の名前で探す command〈`pgrep`・`pidof`〉は使わない（Coordinator の Bash は command ごとに別の PID 名前空間の sandbox で動き、`pgrep -f` は自分を包む shell に当たってアプリが無くても exit 0、`pidof` は別の call のアプリが見えず動いていても exit 1 になる〈2026-10-31 に両方を再現〉）。`git status --short` の確かめは、その知らせの後に Coordinator が行う。アプリは終了で DB の接続を閉じない（`src-tauri/src/lib.rs` の `app.manage(AppState { db: … })` が接続を持ち、tauri の `App::run` は managed state を drop せずに process を終える）ので、閉じた後には毎回 `inventory.db-wal`・`inventory.db-shm` が残り、vT の確定済みの変更が WAL にだけあることがある。そのため予行の後の写しは、Linux の app_data_dir の `inventory.db`・`inventory.db-wal`・`inventory.db-shm` の 3 つを同じ folder に置いたまま読取りだけで開く（`sqlite3 -readonly ~/.local/share/com.inventory-system.desktop/inventory.db`。`-readonly` でも同じ folder の WAL の確定済みの frame を読む。`ATTACH` した DB も読取りだけになる。3 つを分けて写さず、`.db` だけを写して開かない: 上の「写し方」と同じく WAL にだけある変更が欠ける）。開いた後に `ATTACH '<vT の前の snapshot>' AS before` をして、次を確かめてから、owner に L3 を頼む（M1・M2 の自動 test の、実の開発・試験 DB での裏づけ）: (0) vT が当たったこと（当たっていない写しでも (a)(b) は全部通るので先に確かめる）: `SELECT MAX(version) FROM schema_versions` が N（その build の `migrations()` の最大）、`before` の同じ値が N − 1（`schema_versions` は `version`・`applied_at` だけで description を持たないので、vT の中身は次の 2 つで確かめる）、`sqlite_master` に `transfer_records`・`transfer_items` の表があり、`SELECT sql FROM sqlite_master WHERE name = 'inventory_movements'` が `'transfer'` と `'transfer_record'` を含む。どれかが違えば (a)(b) に進まず、予行が当たっていない finding にする。(a) 修飾の無い表名は開いた予行の後の写し（`main`）を指す。両向きの差分 `SELECT COUNT(*) FROM (SELECT * FROM inventory_movements EXCEPT SELECT * FROM before.inventory_movements)` と、`SELECT COUNT(*) FROM (SELECT * FROM before.inventory_movements EXCEPT SELECT * FROM inventory_movements)` が両方 `0`（全列の比べ。`id` が主キーなので行の重複は無く、集合の比べで行ごとの一致になる。列の取り違えは両向きの差分の行に出る）、(b) `inventory_movements` の行数と `MAX(id)`、`sqlite_sequence` の `inventory_movements` の値（行が無ければ 0 と読む。22 §21 の手順 2 と同じ）が両方の DB で同じ。どれかが違えば L3 を頼まず、差分の行を finding にする。

## Data Safety

- 実の店の DB・POS の資料・商品の実値を commit しない。test と L3 の fixture は合成の商品コード（`L3-TR-*` ほか）と値だけ。
- L3 の DB は owner の PC の開発・試験 DB（repo の外）。L3 の商品 CSV は Coordinator が repo の外で渡す。
- 店の運用の細部と発言の原文は書かず、owner の決定は repo 外の回答台帳の ID で引く。
- 開発・試験 DB: vT は値のある開発・試験 DB（owner の PC の L3 用の DB を含む）にもそのまま当たり、`inventory_movements` を作り直す。vT を含む build を owner の PC で初めて起動する前に、上の Recovery の「写し方」で vT の前の snapshot（今の版のアプリ〈base の build〉の「今すぐバックアップを作成」の 1 file。WAL の確定済みの変更を含む）を作り、Coordinator が repo の外へ写して版と `integrity_check` を確かめる（DB の file だけを写す形は使わない）。写し方の誤りが見つかったとき・版の番号を取り直したとき・owner が機能を戻すと決めたときに戻す元になる（上の Recovery）。予行の写しは Coordinator の WSL の repo の外（Linux の app_data_dir と日付の付いた folder）だけに置き、commit しない。予行の起動の自動バックアップは止める（起動の前に予行の写しの `app_settings` の `backup_enabled` を `'0'` にする。上の Recovery の予行）: 止めないと、写した DB の `backup_path` によって vT の後の DB の写しが作業 checkout の中に書かれうる。owner の Windows の data folder の file は予行で置き換えない。D-123 の作り直しは要らないが、作り直す場合も古い file は消さずに移す（22 §16 の作り直しの手順）。
- 導入の後に main へ入る場合: 本 lane は導入の前の関門で、初回の導入の版に入る見込み（店の DB は空から作られ、vT は行の無い `inventory_movements` を作り直す）。本 lane が導入の後に main へ入ることになったら、店の DB は値のある本番の DB になるので、その版の配布の前に、(1) 店の PC の DB の backup が確かめ済みであること（[71](../function-design/71-mnt-backup.md) の起動の時の backup と確かめ）、(2) 店の DB の snapshot（上の Recovery の「写し方」）での vT の予行（上の Recovery の予行と同じ手順と全列の比べ）、(3) 上の Recovery の手順を店の PC で管理者が行えること、を Ready の前の条件に足す Gated Amendment を出す（`r4` の承認の範囲を本番の DB に広げるので、owner に改めて諮る）。

## Implementation Results

この節は計画の節ではない（実装の後の記録）。Scope・AC・Contract Ledger・Recovery・Data Safety の文は変えていない。

### commit

- S1: `4574ecf6` feat(inventory): 振替の migration と記録の repository を足す
- S2: `c9764872` feat(inventory): 振替の作成と記録の読取りの BIZ を足す
- S3: `2d98d818` feat(inventory): 振替の command と wire を足す
- S4: `c5efe375` feat(inventory): 詰め替え・切り分けの作成画面と記録の詳細を足す
- S5: この節を足した commit（docs: 振替の正本を現行の記述に直し、REQ-210 を足す）

vT の版は v13（file `src-tauri/src/db/schema_v13.rs`、`migration.rs` の登録の版 13）。base の `origin/main`（`25d106a9`）の `migrations()` の最大は 12（#172 の vS）で、22 §21 の N と同じ。正本（22・DB_DESIGN・tracking・transaction・21）の版の番号と file 名も v13 / `schema_v13.rs` で揃っている。

### Scope の文と違えた点と理由

- `src-tauri/src/db/mod.rs`: Scope の S1 は `pub mod schema_v{N};` だが、既存の `schema_v*` に揃えて `mod schema_v13;`（非公開）にした。`migration.rs` だけが使い、外へ出す理由が無い（Coordinator が受理）。
- `src-tauri/tests/architecture_test.rs`: D-083 の allowlist に `TransferRecordDetail` の re-export の 1 行を足した（予定 path の一覧に無い。CMD が BIZ の再 export で詳細の型を返すため。Coordinator が受理）。
- `src-tauri/src/biz/inventory_service/transfer.rs`: B1 と R1 の test の名前に `req210` を入れた（`test_create_transfer_req210_d143_b1_…`・`test_list_inventory_records_req210_d143_r1_…`）。`generate_traceability` は Rust の test を `fn test_…` の名前の `_reqNNN` で REQ に結ぶので、入れないと 90 の REQ-210 の Rust 側が「—」になる。Test Plan の行の ID（`d143_b1`・`d143_r1`）は保った。
- `src/lib/query-keys.ts`（予定 path の一覧に無い）: `inventoryRecords.transferDetail(recordId)` を足した。記録の詳細の query の key（ほかの記録の詳細と同じ形で、C23 の `inventoryRecords.root()` に含まれる）。
- `src/features/stock-transfer/StockTransferNavigation.test.tsx`（新設、予定 path の一覧に無い）: U1 の「router の上で active」の test。`navigation.test.ts` は `.ts` で JSX を書けないので、実 router と `SidebarArea` を使う test を別 file にした（`navigation.test.ts` には配置と値の test を足した）。
- `src/test/page-root-pageshell-sweep.test.ts`（T2 の一覧に無い）: PageShell を使う page の数の期待を 28 file / 43 箇所から 30 file / 47 箇所に直した（`StockTransferPage` の 1 と `TransferRecordDetailPage` の 3）。
- 保存結果の行ごとの「在庫を見る」（79 UI-16-D8、64 UI-05-D11 と同じ先）: 64 の先 `/products?q=<商品コード>&selected=<商品コード>` は D-138 の未実装で、今の `/products` の search の schema に `selected` が無い。今ある `q` だけを渡す `/products?q=<商品コード>` にした（D-138 の runtime で `selected` を足す）。
- `docs/spec/requirements-coverage.md`: Scope の S5 は「同じ行」を足すとするが、足していない。この ledger は owner 保管原典の 42 ID だけを持ち、「上記 42 件との差集合は常に空」（同 file の完全性契約）で、Design Phase 補足 ID（REQ-106・107・206〜209）も載っていない。REQ-210 も同じ扱いにした（Coordinator の裁定待ち）。
- `src/routeTree.gen.ts` は `.gitignore` の対象で追跡されない。AC の `git diff --exit-code src/routeTree.gen.ts` は常に 0 になり、生成の確かめは `npm run typecheck`（`pretypecheck` が `generate:routes` を回す）と `npm run build` が担う。

### この変更で不要になった記述

`rg -n 'proposed・未実装' docs --glob '!docs/archive/**' --glob '!docs/plans/**' | rg -c -e 'D-143' -e '振替' -e 'transfer' -e '詰め替え'` が出力 0 行・exit 1（AC9）。31 の「入出庫 4 業務」「4 つの `create_*`」（振替を含む 5 つに）、10 の「4 つの create」（ほかの 4 つ）と「5 経路」（6 経路）、74 の許可リストの括弧書き、66 の「振替の runtime が足す」、79・65・tracking・DB_DESIGN・SCREEN_DESIGN の「要求の ID は runtime の lane が足す」を現行の記述に直した。65・79 の変更履歴の起票の行の「proposed・未実装」は「runtime の前の設計」と言い換えた（AC9 が変更履歴を除かないため）。
