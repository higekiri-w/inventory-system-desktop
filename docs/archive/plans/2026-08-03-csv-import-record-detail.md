# Plan Packet: CSV取込み詳細 route + get_csv_import_record（65 slice 4b）

PR #57 owner L3（2026-08-03）で確認された既存不具合の是正: 在庫変動履歴の「CSV取込み #n」元記録 link が 404（`/csv-import/records/$importId` 詳細 route 不在）。UI-06c-D7 の明示契約（未実装 route でも `source.route` を表示）どおりの既存 gap であり、PR #57 の回帰ではない。

## Workflow State

- Phase: ready-hosted-final
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: d25990c
- Amendments: 9b24376 989d352
- Coordinator: Claude Fable 5 (main session)
- Writer: Codex (GPT-5.6、発注書駆動)
- Plan Reviewer: Claude Sonnet 5 (independent fresh context)
- Final Reviewer: Claude Sonnet 5 (independent fresh context) + Coordinator mutation 独立再実測
- Reviewed Content HEAD: 00097c3
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（初回 L3 主要動線 PASS + 再 L3 accordion 発見性 PASS + Ready 承認済み 2026-08-03。残る owner 操作は merge のみ）

編成注記: 前セッション引き継ぎは「Codex Plan Review」としていたが、本 change の Writer は Codex であり、D-062 (c)「Writer が Codex の packet では Plan Reviewer が Writer と別 vendor でなければならない（codex-only でも免除されない）」により Plan Reviewer を Claude Sonnet 5 独立 context に是正した。引き継ぎ案は batch A（Writer = Sonnet / Reviewer = Codex 鏡像分担）の編成の持ち越しであり、本 packet の編成が D-062 適合形。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
新規 route/search state（`/csv-import/records/$importId` + `returnTo`）、新規 Tauri command DTO + generated bindings（`CsvImportRecordDetail`）、operator workflow（「押せるのに 404」の解消）、D-052 invalidation 契約の集合変更（C9）を含むため。DB schema・CSV format・merge gate は変更しない。

## Goal

Goal Invariant:

### 最小完了条件

- 在庫変動履歴の「CSV取込み #n」元記録 link から、404 ではなく CSV取込み詳細（ヘッダ、sale_records 明細、エラー行、関連 movements、rollback 状態。65 §65.5 CSV取込み列）が表示される。

### 失敗定義

- link が引き続き 404 になる、または詳細画面が 65 §65.5 CSV取込み列の必須項目（記録ID/業務日付/作成日時/状態、明細数、商品情報、数量/単位、金額、関連 movements、rollback 状態）を表示しない。
- 既存 CSV 取込み flow（parse/commit/rollback/list）または既存 4 記録詳細画面に回帰が出る。

### 非目的

- 一覧 route `/csv-import/records` と `listCsvImportRecords(query)`（後続スライス、65 §65.10 slice 4b に明記）
- 棚卸し詳細 route
- rollback CTA の詳細画面への配置（UI-07 取込み画面の責務のまま）
- 74-ui-operation-logs 許可リストへの `csv_import` 追加（`record_type` producer 0 件のため実データ影響なし。65 §65.8.3 の「一時的に除外」記述が追跡先）

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

1. IO: `db::sales_repo` へ `get_csv_import_record_detail` + `list_csv_import_error_rows` 追加（[24-io-csv-import-repo.md](../../function-design/24-io-csv-import-repo.md) §14.13a）
2. BIZ: `biz::csv_import_service::get_csv_import_record` 追加（[32-biz-csv-import-service.md](../../function-design/32-biz-csv-import-service.md) §15.6a）。movements の source 補完は `biz::inventory_service` の既存 `resolve_movement_source` を共有し、label/route 規則の複製を作らない。**共有には `inventory_service/mod.rs` へ `pub(crate) use list::resolve_movement_source;` の re-export 1 行追加が必要**（`mod list` は private のため sibling module から現状 unreachable。同 file の `pub(crate) use common::apply_stock_change`〈BIZ-03 CSV取込みから呼び出し可〉と同型の既存慣習）— Plan Review round 1 P1 是正
3. CMD: `cmd::csv_import_cmd::get_csv_import_record` 追加 + `#[tauri::command]` / `#[specta::specta]` 属性の対 + `lib.rs` の **2 箇所登録**: `export_specta_bindings()` 内 `collect_commands![...]`（bindings 生成用）と `.invoke_handler(tauri::generate_handler![...])`（実行時 dispatch 用）の両方（[41-cmd-pos.md](../../function-design/41-cmd-pos.md) §17.5 / §17.9。collect_commands のみでは AC4 が green のまま IPC 実呼出しが「command not found」になる — Plan Review round 1 P2 是正）
4. bindings: `cargo run --bin generate_bindings` 再生成（`getCsvImportRecord` / `CsvImportRecordDetail` / `CsvImportErrorType` union）
5. route（gated Amendment 1 で訂正）: `src/routes/csv-import.records.$importId.tsx` 新設 + **`csv-import.tsx` の layout route 化**（`<Outlet />` のみを render、既存 `receiving.tsx` と同型）+ **`src/routes/csv-import/index.tsx` 新設**（`CsvImportPage` を index route へ移設、既存 `receiving/index.tsx` と同型）+ `npm run generate:routes`。詳細 route は `CsvImportRoute` の子として layout 経由で描画される。既存 `/csv-import` 取込み画面は index route で従来どおり描画されることを runtime route test で固定する（T16）。`validateSearch` の `returnTo` pattern（`z.string().max(500).optional().catch(undefined)`）を既存 4 詳細 route と同形で踏襲
6. UI: `src/features/inventory-records/CsvImportRecordDetailPage.tsx` 新設。`ReceivingRecordDetailPage.tsx` を canonical 参照とし、同構造（useQuery + queryKeys + describeError + returnTo 戻り導線）。status は 65 §65.6.1 に従い正規化した日本語 label（completed / completed_partial / rolled_back の 3 値表示）、rolled_back 時は rollback 状態を明示表示
7. query key: `queryKeys.inventoryRecords.csvImportDetail(importId)` 追加（既存 `*Detail` 4 key と同形）
8. D-052 契約変更（C9）: `invalidation-contract.ts` の `csvImportRollback` へ **広域 prefix `queryKeys.inventoryRecords.root()` を追加**する（既存 receiving / return / manualSale / disposal mutation と同一 pattern。`csvImportDetail` が正当 consumer となるため prefix + child collateral は D-052 許容に適合し、zero-arg 署名を変えないため `invalidation-contract.meta.test.ts` / `invalidation-oracle.ts` / `useCsvImportFlow.ts` の署名追随が不要 — Plan Review round 1 P2 是正で importId 個別指定案を不採用）+ 独立転記 oracle test の追随 + production-only mutation 感度の再実測。`csvImportCommit` への追加要否も table.column 導出手順（UI_TECH_STACK §2.5）で導出し、採否と根拠を PR body に記録（rollback は `csv_imports.status` / `sale_records.is_voided` / `inventory_movements.is_voided` を確定し本 query が全て読むため追加必須。commit は既存 import 行を変更しないため過剰禁止原則との突合が必要）
9. tests: 実装と同時に作成、`REQ-206` / `REQ-207` token 付与、`cargo run --bin generate_traceability` で 90 再生成
10. design docs（本 plan-first commit に同乗済み）: 24 §14.13a / 32 §15.6a + 更新履歴 / 41 §17.5 + §17.9 / 65 §65.10 slice 4b + 変更履歴
11. （gated Amendment 2、owner L3 裁定 2026-08-03 による Scope 追加）取込み画面 `ErrorRowsTable.tsx` の accordion trigger 発見性是正: 閉時 trigger を「エラー詳細を見る（N件）」、開時を「エラー詳細を閉じる（N件）」の明示的操作文言へ変更し、展開 chevron を文言隣接位置へ移動（`justify-start` 上書き）。accordion 挙動・行全体 click・keyboard focus は維持。55 §55.5 の trigger 文言規定を同期し、T17 test（閉状態の操作文言可視 / click 展開と文言切替 / 再 click で閉じる）を追加

## Non-scope

- 一覧 route `/csv-import/records` + `listCsvImportRecords(query)`
- 棚卸し詳細 route `/stocktake/records/$stocktakeId`
- rollback CTA の詳細画面配置
- 74-ui-operation-logs 許可リストへの `csv_import` 追加
- 55-ui-csv-import（取込み画面）recent list からの詳細導線追加（後続判断）
- CSV 出力 / 印刷（65 §65.10 slice 6）・画像添付
- `csv_imports` / `sale_records` / `csv_import_errors` の schema 変更（なし）
- 既存 preview / commit / rollback / list コマンドの wire 変更（なし）

## Acceptance Criteria

- AC1（変更前 canary）: main 時点で `/csv-import/records/$importId` route が存在しない実出力（`rg -c "csv-import/records" src/routeTree.gen.ts` = 0 件、または dev 環境での 404 スクリーンショット）を PR body に収録する
- AC2: `cd src-tauri && cargo test` green（`get_csv_import_record` 系の IO/BIZ/CMD test を含む）
- AC3: `npm test` green（`CsvImportRecordDetailPage` test を含む）
- AC4: `bindings.ts` diff に `getCsvImportRecord` / `CsvImportRecordDetail` が追加され、`cargo run --bin generate_bindings` 再実行で clean diff
- AC5: `src/routeTree.gen.ts` に `/csv-import/records/$importId` が生成される
- AC6: 在庫変動履歴画面の「CSV取込み #n」link を `userEvent.click` で押下し、SPA 遷移後に詳細画面の内容が render されることを assert する test（`href` assert のみは不可 — batch A X3 survivor の教訓）
- AC7: 存在しない import_id で利用者向け日本語 error（「CSV取込み記録が見つかりません」系）が表示される test
- AC8: rolled_back 取込みの詳細で、正規化状態 label + movements 0 件の正常表示 + 明細の is_voided 表示を assert する test（Matrix T12。`npm test` で実行され green）
- AC9: invalidation 独立転記 oracle test が `csvImportRollback` の新集合と順序非依存・重複検出付き完全一致
- AC10: `bash scripts/local-ci.sh full` CLEAN（L1、exact-HEAD evidence は PR body 所管）

## Design Sources

- Requirements / spec: REQ-206（過去記録の検索・詳細表示）、REQ-207 / TRACE-D2（movement → 元記録の相互遷移）
- Architecture: `docs/ARCHITECTURE.md`（UI → CMD → BIZ → IO 一方向）
- Function / command / DTO: [65-inventory-record-traceability.md](../../function-design/65-inventory-record-traceability.md) §65.3 / §65.5 / §65.7.1 / §65.10 slice 4b、[24-io-csv-import-repo.md](../../function-design/24-io-csv-import-repo.md) §14.13a、[32-biz-csv-import-service.md](../../function-design/32-biz-csv-import-service.md) §15.6a、[41-cmd-pos.md](../../function-design/41-cmd-pos.md) §17.4 / §17.5 / §17.9、[31-biz-inventory-service.md](../../function-design/31-biz-inventory-service.md) §12.6a（read-only パターン正本）、[66-ui-stock-movements.md](../../function-design/66-ui-stock-movements.md) UI-06c-D7
- DB: `docs/DB_DESIGN.md` 12 / 12a / 13（csv_imports / csv_import_errors / sale_records。schema 変更なし）
- Screen / UI: 65 §65.5 / §65.8、`docs/UI_TECH_STACK.md` §2.5（D-052 導出原則）/ §5.4（focus 可視性）、`docs/design-system/README.md` 一式、[59-ui-shared-patterns.md](../../function-design/59-ui-shared-patterns.md)
- Decision log / ADR: D-052（invalidation SSOT。Revisit 条項 = 契約変更時は SSOT / oracle / mutation 再実測を同一 PR）、D-061（有限 IPC 値の generated enum）、D-062（Plan Reviewer 別 vendor）、D-023（POS adapter boundary — 本 change は app-core の import lifecycle 表示のみで adapter 事実に触れない）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 24 §14.13a / 32 §15.6a / 41 §17.5 | updated in this PR（plan-first commit 同乗） |
| Command / DTO / generated binding / wire shape | 41 §17.5 + 本 packet `Boundary / Wire Contract` | updated in this PR |
| DB / transaction / audit / rollback / migration | DB_DESIGN 12/12a/13 | existing sufficient（read-only、schema 変更なし） |
| Screen / UI / route state / Japanese wording | 65 §65.3 / §65.5 / §65.6.1 / §65.10 slice 4b | existing sufficient + slice 4b を updated in this PR |
| CSV / TSV / report / import / export format | — | 触らない（取込み format 不変） |
| Durable decision / ADR | D-052 / D-061 / D-062 既存適用 | existing sufficient（新規 durable decision なし） |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 | 対応 |
|---|---|---|
| Tauri command `get_csv_import_record` | `lib.rs` の 2 箇所登録（`collect_commands![...]` + `.invoke_handler(generate_handler![...])`）/ `#[tauri::command]` + `#[specta::specta]` 対 / `generate_bindings` 再生成 | Scope 3 / 4。Ledger 行あり（invoke_handler 行を独立行として立てる） |
| function-design doc 新設 | — | 該当なし（既存 4 doc への追記。`build_doc_to_modules_map()` の 24/32/41/65 entry は `db::sales_repo` / `biz::csv_import_service` / `cmd::csv_import_cmd` を登録済みで map 変更不要。§14.13a/§15.6a/§17.5 の新関数は実装までは info_unimplemented 扱い） |
| source / workflow doc 新設・改名 | — | 該当なし |
| Consultation Relay | — | 不使用 |
| REQ coverage 追加 | `generate_traceability` で 90 再生成 | Scope 9 |
| route 新設 | `npm run generate:routes` | Scope 5 |
| operator 画面新設 | navigation entry | 不要（詳細画面は sidebar 非搭載。既存 4 記録詳細と同輩）。到達導線契約は「movements link click → SPA 遷移」を Ledger 行 + AC6 で担保 |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-207 / TRACE-D2 | 65 §65.3（route 表 L61）/ §65.10 slice 4b | UI-06c-D7 後続 | 詳細 route 実装を採用。link 非活性化案は 65 完成形（route 表・§65.5・§65.7.1 が既定）からの後退で、backend `resolve_movement_source` が route 文字列を生成済みのため不採用 | route + `CsvImportRecordDetailPage` | AC1 / AC5 / AC6 |
| REQ-206 | 65 §65.5 CSV取込み列 / §65.6.1 状態正規化 | — | 表示項目は既存契約に従う。金額 yes / 原価 no / 種別 no の列定義どおり | `CsvImportRecordDetailPage` | AC3 / AC8 |
| REQ-207 | 66 UI-06c-D7 | — | link URL 表示契約は不変。遷移先実装のみ追加 | — | AC6 |
| D-052 C9 | UI_TECH_STACK §2.5 | D-052 | rollback は `csv_imports.status` / `sale_records.is_voided` / `inventory_movements.is_voided` を確定し、本 query が 3 table を読むため invalidate 追加必須。commit は既存 import 行を変更しないため過剰禁止原則で要導出 | `invalidation-contract.ts` + oracle test | AC9 |
| D-061 | 32 §15.6a / 24 §14.13a | D-061 | `error_type` は BIZ 所有 `CsvImportErrorType`（IO は raw TEXT を返し BIZ で変換 — 層方向維持）。`status` は既存 IO 所有 `CsvImportStatus`（`CsvImport` header 型が enum 変換済みの値を返すため BIZ での変換なし・素通し）— Plan Review round 1 P3 是正で分離記述 | DTO | AC4 |
| REQ-206 | 65 §65.5（returnTo 戻り） | TRACE-D11 同型 | movements から来た場合の検索条件保持は既存 4 詳細の `returnTo` pattern を踏襲 | route `validateSearch` | Matrix T13 |

## Design Intent Audit

- Source docs can answer what/why without chat history: yes — 65 slice 4b が起点（404 backlog）と非 scope 境界を、24/32/41 が層別設計を記載
- Plan-only durable decisions promoted: なし（wire DTO の BIZ 所有・ErrorRow 再利用・IO raw TEXT の層判断は 24/32 に記載済み）
- Assumptions / constraints: SQLite 単一接続、DB CHECK による error_type 4 値保証、詳細 route は親 layout（`<Outlet />`）経由で描画される layout + index 構造（receiving 系 4 種と同型。gated Amendment 1 で訂正、Contract Probe 参照）
- Deferred design gaps: 一覧 route / 棚卸し詳細 / 74 許可リスト追随（それぞれ 65 §65.10・§65.8.3 が追跡）
- Test Design Matrix cites decision IDs: yes（Matrix 参照）
- Absolute guarantee self-check: 「rolled_back 取込みは movements 0 件」は void_movements_by_reference の既存動作に依存 — 例外は「rollback 前に手動補正が同一 reference に movement を作る」経路だが、`csv_import` reference は BIZ-03 のみが書くため成立しない（32 §15.5 正本）。詳細画面は 0 件でも N 件でも表示が壊れない設計とし T12 で固定

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | 本 change は app-core の import lifecycle 表示のみ。Z004 layout / CV17 等 adapter 事実に触れない | — |
| Fact check / design decision split | 404 は owner L3 実測（PR #57、2026-08-03）。起点事実は Plans.md backlog 起票に記録済み | Plans.md |
| Lifecycle / retry | rollback 後の詳細表示（rolled_back label + movements 0 件 + voided 明細）を State Lifecycle Matrix でカバー | Test Matrix |
| Operator workflow | 「押せるのに 404」解消。movements → 詳細 → returnTo 戻りの実順序 | Matrix T10/T13 + L3 |
| Replacement path | not applicable（POS 外部システム非接触） | — |
| Data safety / evidence | synthetic fixture のみ。実売上データ非 commit | Data Safety 節 |
| Reporting / accounting semantics | 表示のみで集計意味論不変。金額は sale_records.amount の表示転記 | — |
| Manual verification | L3 視認 1 項目（synthetic 取込み後の link 遷移・表示確認） | Human Gate |
| 環境・再現性 | 新設の環境依存なし（既存 toolchain / 既存生成系のみ） | — |

## Design Readiness

- Existing design docs are sufficient because: 65 が route / 表示項目 / command 対 / 状態正規化の完成形を保持し、31 §12.6a が read-only 詳細取得の層パターンを確立済み
- Source docs updated in this PR: 24 §14.13a / 32 §15.6a / 41 §17.5・§17.9 / 65 §65.10 slice 4b（いずれも plan-first commit 同乗）
- Design gaps intentionally deferred: 一覧 route、棚卸し詳細、74 許可リスト追随
- Durable decisions discovered and promoted: なし

Minimum design checks:

- Layer ownership: UI（表示・returnTo）→ CMD（thin wrapper）→ BIZ（NotFound 変換・ErrorRow 写像・source 補完）→ IO（SQL・DTO core）
- Backend function design: 24 §14.13a / 32 §15.6a / 41 §17.5
- Command / DTO / data contract: Boundary / Wire Contract 節
- Persistence / transaction / audit impact: read-only、TX 不要、operation log 記録なし（31 §12.6a と同判断）
- Operator workflow / Japanese wording: 状態 label 正規化（65 §65.6.1）、error 文言は describeError 経由
- Error / empty / retry / recovery: NotFound・エラー行 0 件・movements 0 件・DB error の各経路を Matrix でカバー
- Testability / traceability: REQ-206 / REQ-207 token、90 再生成

## Contract Probe

- TanStack Router flat dot 記法の親 route 共存（**gated Amendment 1 で probe 結論を訂正**）: 当初 probe は「routeTree に独立 route として共存生成」と結論したが、これは routeTree.gen.ts の entry 共存のみを確認し**親子関係（`getParentRoute`）と親の `<Outlet />` 有無を見落とした誤り**。実際は `receiving.records.$recordId` は `InventoryReceivingRoute` の**子**として登録され（routeTree.gen.ts `getParentRoute: () => InventoryReceivingRoute`）、`receiving.tsx` は `<Outlet />` のみを render する layout route、作成画面は `receiving/index.tsx` が担う layout + index 構造。`csv-import.tsx` は `CsvImportPage` を直接描画して Outlet を持たないため、詳細 route は URL が変わっても描画されない — Writer（Codex）の T10 runtime test red が実装中に検出し fail-closed 停止（2026-08-03、true positive）。是正 = csv-import も layout + index 構造へ再構成（Scope 5 訂正）。probe を「是正仮適用の end-to-end（runtime 描画まで）」で回さず routeTree 静的確認で打ち切ったことが見落としの原因

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| 24 §14.13a: ヘッダ不存在 → DbError::NotFound | `sales_repo::get_csv_import_record_detail` | T1 | — |
| 24 §14.13a: sale_records 明細 JOIN + is_voided 込み全行 + ORDER BY id ASC | 同上 | T2 | — |
| 24 §14.13a: movements は `csv_import`/`is_voided=0` filter + source=None | 同上 | T3 | — |
| 24 §14.13a: error rows ORDER BY source_line_no ASC + 0 件は空 Vec | `sales_repo::list_csv_import_error_rows` | T4（0 件 / N 件両 case、非空期待 1 case 以上） | — |
| 32 §15.6a: NotFound 変換 + 「CSV取込み記録が見つかりません」文言 | `csv_import_service::get_csv_import_record` | T5 | — |
| 32 §15.6a: ErrorRow 写像（line_no/name mapping + error_type enum 変換 + 想定外値 fail-fast） | 同上 | T6 | — |
| 32 §15.6a: source 補完は `resolve_movement_source` 共有（label「CSV取込み」/ route `/csv-import/records/{id}`） | 同上 | T7 | — |
| 32 §15.6a: wire DTO へ file_hash 非搭載 | `CsvImportRecordDetail` 型 | T6（DTO field assert） | — |
| 41 §17.5: CMD thin wrapper + kind="not_found" 変換 | `csv_import_cmd::get_csv_import_record` | T8（production command 実呼び） | — |
| 41 §17.9: collect_commands 登録 + specta 対 + bindings 生成 | `lib.rs` `export_specta_bindings()` / `bindings.ts` | AC4（clean diff） | — |
| 41 §17.9: invoke_handler（`generate_handler!`）登録 — collect_commands と独立の登録点 | `lib.rs` `.invoke_handler(...)` | T10 / AC6（IPC 実呼出し経路）+ 実装 review で登録行 diff 確認 | L3 視認が実 IPC の最終確認 |
| 32 §15.6a: `resolve_movement_source` の pub(crate) re-export（`inventory_service/mod.rs` 1 行、apply_stock_change 前例と同型） | `biz/inventory_service/mod.rs` | T7（共有関数経由の label/route exact oracle） | — |
| 65 §65.3: route `/csv-import/records/$importId` | route file + generate:routes | AC5 | — |
| Amendment 1: `csv-import.tsx` layout 化 + `csv-import/index.tsx` 移設で既存取込み画面が従来どおり `/csv-import` で描画される | route files（layout + index） | T16（runtime route test） | L3 で取込み画面到達も一瞥確認 |
| Amendment 1: 詳細 route が layout `<Outlet />` 経由で実描画される | `csv-import.tsx` layout | T10（click 遷移後の詳細 render assert — 本 Amendment の検出元 test） | — |
| Amendment 2: ErrorRowsTable accordion trigger の明示的操作文言（閉「エラー詳細を見る（N件）」/ 開「エラー詳細を閉じる（N件）」）+ chevron 文言隣接 + 行全体 click・keyboard focus 維持 | `ErrorRowsTable.tsx` + 55 §55.5 同期 | T17 | 再 L3（collapsed / open の発見性限定） |
| 65 §65.5 CSV列: 表示項目（ID/日付/状態/明細数/商品情報/数量/金額/movements/rollback 状態） | `CsvImportRecordDetailPage` | T9 | L3 視認 |
| 65 §65.6.1: status 正規化 label 表示 | 同上 | T9 / T12 | L3 視認 |
| 65 §65.5 / TRACE-D11 同型: returnTo 検索条件保持 + 不正戻り先 fallback | route `validateSearch` + Page | T13 | — |
| 66 UI-06c-D7: movements link click → SPA 遷移で詳細 render（到達導線契約） | movements link（既存）+ 新 route | T10（userEvent.click、href assert 単独不可） | L3 視認 |
| D-052 C9: rollback invalidation に本 query 追加 + 独立転記 oracle 完全一致 | `invalidation-contract.ts` + oracle test | T14 | — |
| D-052: commit への追加要否の導出記録 | PR body | — | 導出根拠を PR body 記録 |
| query key 直書き禁止（D-4 / 順17 無例外化） | `queryKeys.inventoryRecords.csvImportDetail` | T15（literal sweep 既存 pattern） | — |
| 既存 flow 回帰なし（parse/commit/rollback/list + 既存 4 詳細） | — | 既存 test suite green（AC2/AC3） | — |

隣接契約 sweep 実施記録: 65 §65.5 の「詳細画面からは在庫照会の商品詳細へ遷移できる」は既存 4 詳細の商品 link pattern を指す — CSV 明細行にも product_code があるため同 pattern を T9 の表示項目に含める（商品 link の有無は canonical の ReceivingRecordDetailPage と同構造とする）。65 §65.9 出力（CSV/印刷）は slice 6 で非 scope。§65.5「取消/訂正 CTA は cancel/correct command 実装済みの場合だけ表示」— CSV は rollback CTA を置かない（非目的）ため表示条件に抵触しない。

## Test Plan

Test Design Matrix: [test-matrices/2026-08-03-csv-import-record-detail.md](test-matrices/2026-08-03-csv-import-record-detail.md)

- targeted tests: IO/BIZ/CMD unit + production command 実呼び、Page RTL、click SPA 遷移、invalidation oracle
- negative tests: NotFound、エラー行 0 件、movements 0 件、不正 returnTo fallback
- compatibility checks: 既存 CSV command 4 本の wire 不変（bindings diff が追加のみであること）、既存 4 詳細画面 test green
- data safety checks: synthetic fixture のみ、実 POS データ非使用
- main wiring/integration checks: collect_commands 登録 → bindings 生成 → frontend 呼出しの end-to-end（AC4/AC5/AC6）
- Human Gate に L3 を含むため、Writer 完了条件に `cargo check --release` を含める（CI gate ではない）

## Boundary / Wire Contract

- producer: `cmd::csv_import_cmd::get_csv_import_record`（Rust）
- consumer: `CsvImportRecordDetailPage`（`commands.getCsvImportRecord(importId)` 経由の useQuery）
- wire type: `CsvImportRecordDetail`（tauri-specta 生成。field は snake_case、`status: CsvImportStatus`（3 値 union）、`error_rows: ErrorRow[]`（`error_type: CsvImportErrorType` 4 値 union）、`items: CsvImportRecordDetailItem[]`、`movements: MovementRecord[]`（source 補完済み））
- internal type: IO `CsvImportRecordDetailCore`（header: CsvImport + items + movements）+ `CsvImportErrorRecord`（error_type raw TEXT）。BIZ が wire DTO を構成
- precision/range: 金額 i64（円整数）、id i64。JS safe integer 内（既存 CsvImport wire と同等）
- round-trip path: DB → IO DTO → BIZ wire DTO → bindings → UI 表示のみ（書込みなし）
- invalid input: 存在しない import_id → `CmdError.kind="not_found"` → describeError 経由の利用者向け日本語文言
- compatibility: 既存 command（parseAndValidateCsv / commitCsvImport / rollbackCsvImport / listCsvImports）の wire 不変。`ErrorRow` 型共有は既存 preview wire に影響しない（型追加参照のみ）。file_hash は wire 非搭載

## Review Focus

- D-052 C9 集合変更の導出の正確さ（欠落と過剰の両方向。commit 側の採否根拠を含む）
- ErrorRow 再利用の wire 互換と写像の正しさ（line_no/name の field 名差異）
- click SPA 遷移証明が href assert に退化していないか（batch A X3 survivor の再発防止）
- エラー行の空集合 oracle 衝突（0 件期待 case だけにならないこと — 順22 X2 の教訓）
- returnTo の movements 検索条件保持と不正値 fallback
- IO raw TEXT → BIZ enum 変換の fail-fast が握りつぶしになっていないか

## Spec Contract

Contract ID: SPEC-UI06C-CSV-DETAIL-2026-08-03

- movements の `csv_import` 元記録 link は `/csv-import/records/$importId` に SPA 遷移し、65 §65.5 CSV取込み列の項目を表示する（404 にしない）
- 詳細は read-only とし、rollback CTA・取消/訂正 CTA を置かない
- rolled_back 取込みは正規化 label で状態を明示し、movements 0 件・voided 明細を正常表示する
- rollback mutation 成功時、本詳細 query は D-052 SSOT 経由で invalidate される

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| REQ-207 / TRACE-D2 | Scope 5/6（route + Page） | T10 / AC6 | click SPA 遷移証明 | PR body + Matrix |
| REQ-206 | Scope 6（表示項目） | T9 / AC3 | §65.5 列適合 | PR body |
| REQ-206（NotFound） | Scope 2/3 | T5 / T8 / AC7 | 変換経路 | PR body |
| D-052 C9 | Scope 8 | T14 / AC9 | 導出正確さ | PR body（commit 側採否含む） |
| D-061 | Scope 1/2/4 | T6 / AC4 | enum 変換 | bindings diff |
| TRACE-D11 同型 | Scope 5 | T13 | returnTo fallback | Matrix |

## Data Safety

- 実 POS CSV、実売上・実原価データ、実 DB ファイルは commit しない
- test fixture・L3 用データは synthetic のみ（既存 synthetic Z004 系 fixture の慣行に従う）
- L3 手順は「synthetic Z004 の取込み → 在庫変動履歴 → link click」の再現手順を Ready 依頼と同時に PR body へ記載する（L3 fixture prep の教訓）
- `.local/ci-evidence/` はローカルのみ

## Implementation Results

IO / BIZ / CMD の `get_csv_import_record`、generated bindings、CSV取込み詳細 route / Page、rollback invalidation、T1-T16 を実装した。gated Amendment 1 に従って CSV取込み画面を layout + index route へ再構成し、詳細 link の runtime 遷移と既存 `/csv-import` 直接進入を両方固定した。生成物 3 種の drift 確認、L1 full、X1-X7 実注入を完了し、Draft PR [#58](https://github.com/higekiri-w/inventory-system-public/pull/58) を作成した。独立 Final Review と Windows native L3 は pending。

（2026-08-03 完了時追記）独立 Final Review = Ledger 22/22 適合・P1/P2=0・P3×1（JAN 列 pre-existing → Plans.md backlog）、Coordinator mutation 独立再実測 = X1-X7 全 red（X7 は注入形を変えて label/route 両面感度を確認）。owner 初回 L3 = 主要動線 PASS + P3（ErrorRowsTable accordion の発見性）→ owner 裁定の同 PR 是正を正規 backtrack + gated Amendment 2 で処理し、明示的操作文言 + 文言隣接 chevron + T17 + 55 §55.5 同期 + X8 相当 red を追加、Post-Freeze closure 6/6 PASS（findings 0）。再 L3（accordion 発見性限定）PASS + owner Ready 承認により ready-hosted-final へ遷移（本 content commit に同乗、STATECAP 3/3 消費済みのため）。exact-HEAD evidence と hosted run は PR body 所管。

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Fill after review.
- Findings Freeze: frozen after Broad Audit（2026-08-03、Final Review 完了時点で発効）; post-freeze exceptions: none.

### Final Review 記録（2026-08-03、append-only）

- Sonnet 独立 fresh context の Contract Audit: Contract Coverage Ledger 22 行中 22 行が実装・test と整合（不適合 0）。Negative-space audit / State Lifecycle / Adjacent Pattern 再検証 / anti-tautology（T7・T14 の独立転記 oracle、production 定数非 import）/ Non-scope 遵守（branch 差分全 file 説明可能）/ T10 の click→pathname→command→render 実観測 / 生成物 3 種の整合を検査済み。**P1/P2 = 0、P3×1**。
- P3-1（JAN 列）: §65.5 は全記録種別で「商品コード / JAN / 商品名 / 部門 = yes」と規定するが、既存 4 詳細を含む 5 詳細画面すべてで JAN 非表示（DTO にも field なし）。本 PR は canonical 踏襲の結果で新規 drift ではない（pre-existing systemic）。裁定 = backlog 起票（§65.5 の実態同期 or 5 画面横断の JAN 列追加を別 change で設計判断）。
- Coordinator mutation 独立再実測: X1〜X7 の 7/7 全 red 再現（clean tree、注入→該当 test red→復元）。X7 は Writer の注入形（label 改変）と意図的に変えて route prefix 側を改変しても red — oracle の label/route 両面感度を確認。survivor 0。
- implementing → local-verified の evidence: Writer L1 `local-ci.sh full` PASS（exact HEAD `e1b2158` / CLEAN / MERGE_EVIDENCE_VALID=true、PR #58 body 記録）。local-verified → independent-review → human-confirm の evidence: 上記 Final Review P1/P2=0 + mutation 独立再実測。

### Owner L3 結果と state-backtrack（2026-08-03、append-only）

- owner Windows native L3（PR HEAD `4061c55`）: 主要動線 PASS（synthetic 取込み→在庫変動履歴→「CSV取込み #2」→詳細表示→returnTo 戻り、詳細は PR #58 comment 5160238536）。
- L3 P3 finding: 取込み画面 `ErrorRowsTable.tsx` の accordion trigger「エラー N 件」が展開可能な操作部と認識できない（chevron が右端、owner 自身が当初見落とし）。owner 裁定 = **別 follow-up へ送らず本 PR で是正**（Post-Freeze の content correction、Ready 保留）。
- 正規 backtrack: human-confirm → implementing（L3 finding のコード修正のため、遷移表の correction 規則どおり最先影響 phase へ復帰）。是正対象は元 Scope 外（取込み画面 UI-07 側の既存 component）のため、owner 裁定に基づく Scope 追加を gated Amendment 2 として記録する。
- 再 L3 の範囲: 当該 accordion の collapsed / open の発見性に限定（owner 指定）。

### L3 P3 是正の再走 evidence（2026-08-03、append-only）

- 是正 content commit `00097c3`（ErrorRowsTable の明示的操作文言 + 文言隣接 chevron + T17 + 55 §55.5 同期。gated Amendment 2 は `989d352`）。X8 相当の mutation（開閉切替の欠落）を Coordinator が実注入し T17 の 2 test red → 復元を確認。
- Post-Freeze closure（Sonnet 独立 fresh context、3 体目）: 検査 6 項目全 PASS（owner 指定 4 点適合 / T17 の Would-fail 4 種検出可能・tautology なし / 55 同期・旧文言残存 0 / scope 漏れなし / PreviewStep 回帰リスクなし / Amendment 2 整合）。findings 0、P1/P2 = 0。
- `b29fefb` は T17 の REQ-401 token 追加に伴う 90-traceability 機械的再生成のみ（+1/-1、生成コマンド決定的出力）。Coordinator が diff 検分済み。audited content 実質は `00097c3` から不変のため Reviewed Content HEAD は `00097c3`。
- implementing → local-verified: L1 `local-ci.sh full` PASS（exact HEAD `b29fefb` / CLEAN / MERGE_EVIDENCE_VALID=true、evidence は PR body 所管）。local-verified → independent-review → human-confirm: 上記 closure P1/P2=0。
- STATECAP: 本遷移で forward state-only 3/3 消費。ready-hosted-final 遷移は Implementation Results 記入の content commit に同乗させる（前 change の運用教訓を最初から適用）。

### 再 L3 PASS と ready-hosted-final 遷移（2026-08-03、append-only）

- owner 再 L3（accordion の collapsed / open 発見性限定）: PASS（「テストＯＫ」、介入 3/3 の decision point = 再 L3 PASS + Ready 承認）。
- human-confirm → ready-hosted-final の evidence: Human Gate 全項目解決（初回 L3 主要動線 PASS + 再 L3 PASS）+ owner Ready 承認。遷移は本 Implementation Results 追記 content commit に同乗（STATECAP 3/3 消費済みの正規手段）。Ready 化・exact-HEAD L1 full・PR body refresh は遷移 commit 後に実施し、evidence は PR body 所管。

### Plan Review rally 記録（2026-08-03、append-only）

- round 1（Claude Sonnet 5 独立 fresh context、D-062 (c) 編成）: P1×1（`resolve_movement_source` が private `mod list` 経由で sibling module から unreachable — compile blocker かつ「複製 drift」誘発）/ P2×2（`lib.rs` 登録は collect_commands + invoke_handler の 2 箇所で Ledger は 1 箇所のみ / D-052 C9 の invalidate 粒度を特定しておらず cascading Ledger 漏れ）/ P3×1（D-061 Trace 行の status・error_type 層帰属の一括り記述が不正確）。Coordinator が P1 の module 可視性主張を `inventory_service/mod.rs` 実読で独立裏取りし（`pub(crate) use common::apply_stock_change` の同型前例も確認）、全 4 件 accept。是正 commit `d25990c`。
- round 2 closure（別 Sonnet fresh context）: 4/4 CLOSED（各是正の技術的正確さを実コード実読で検証、`useCsvImportFlow.ts` zero-arg 呼出しの署名不変主張も確認）、是正差分起因の新規 findings 0。P1/P2 残 0。
- plan-gate → plan-approved → implementing の materialize evidence: 上記 P1/P2=0、plan-first commit `4854e3a`（+ rally 是正 `d25990c`）が全実装 commit に先行、Writer は Codex（発注書駆動、実装 commit 未作成）。

### Writer fail-closed 停止（初回）と gated Amendment `1`（2026-08-03、append-only）

- Writer（Codex）が実装中の T10 runtime test red で packet の route 前提の誤りを検出し fail-closed 停止（true positive、Draft PR 未作成・未 commit tree 保持）。内容 = Contract Probe の「flat dot 記法は非 nesting 共存」結論が誤りで、詳細 route は `CsvImportRoute` の子として生成されるが `csv-import.tsx` に `<Outlet />` がなく描画されない。
- Coordinator 裁定 = Writer の修正案（layout + index 構造への再構成、receiving 系 4 種と同型）を accept。routeTree.gen.ts の `getParentRoute` / `receiving.tsx` の Outlet-only layout / `receiving/index.tsx` の index 構造を Coordinator が実読で独立裏取り済み。
- gated Amendment 1 = Scope 5 訂正・Assumptions 訂正・Contract Probe 結論訂正・Ledger 2 行追加・Matrix T16 追加 + Adjacent Pattern Audit 行追加。原 `Plan Commit` は不変、Amendment SHA は `Amendments` 行へ後続記録。
- probe 見落としの原因記録: 「是正仮適用の end-to-end」を runtime 描画まで回さず routeTree 静的確認で打ち切った。
