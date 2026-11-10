# Plan Packet: 入出庫履歴 6 種対称化 実装（65 slice 4d）— PR B

Design Phase は PR #13（squash `6c688fe`、2026-08-29 merge）で完了済み。owner 裁定 2026-08-27/28（`/inventory/records` の 6 種対称化、in_progress 棚卸しの hub 表示、状態 filter 4 値化、検索母集団差の注記常設）は 65 §65.10 slice 4d に正本化済み。本 packet はその実装 PR B。専用一覧 `/csv-import/records` / `/stocktake/records` は完成形契約のまま runway 残置（Plans.md backlog「入出庫履歴の完成形 runway 復帰」）。

## Workflow State

- Phase: ready-hosted-final
- Risk: R3
- Execution Mode: fable-window
- Plan Commit: 2941807
- Amendments: 3460133
- Coordinator: Claude Fable 5 (main session)
- Writer: Codex (GPT-5.6、発注書駆動)
- Plan Reviewer: Claude Sonnet 5 (independent fresh context)
- Final Reviewer: Claude Sonnet 5 (independent fresh context) + Coordinator mutation 独立再実測
- Reviewed Content HEAD: 6b9fb32
- Final Exact-HEAD Evidence: PR body
- Hosted CI Requirement: required
- Human Gate: none（owner Windows native L3 全項目 PASS + Ready 承認済み 2026-08-29。残る owner 操作は Ready 化と merge のみ）

Phase 遷移記録（kickoff → spec-check → plan-draft → plan-gate、本 plan-first commit に同乗）: task scope と R3 判定は本 packet に記録。spec-check では in-scope source docs（65 / 21-io / 44-cmd）が PR #13 で改訂済み・実装十分と判定し、spec-check → plan-draft の許可された skip（Design Readiness が既存 docs 充足を引用）を適用。packet + Test Design Matrix を同 commit で commit し plan-gate に至る。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 45分
- relay 往復上限: 5（PR A 実績 = relay 4 往復 / 予算 2 の超過教訓。実装 PR は是正往復を見込んで設定）
- Plan Review round 天井: 3

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
Risk Tiers R3 行のうち route/search state（`/inventory/records` の状態 filter 4 値化・種別 filter 6 種化 = search state の値域拡張）、operator workflow（hub 一覧の表示・絞り込みの実挙動変更、status filter の no-op → 実効化）に該当。SQL 契約変更（UNION ALL 6 種化 + 外側 WHERE 新設）を伴う。DB schema・CSV format・merge gate は変更しない。Tauri command DTO / generated bindings は型不変（record_type / status は string のまま、AC4 で差分ゼロを確認）。

## Goal

Goal Invariant:

### 最小完了条件

- `/inventory/records` で CSV取込み・棚卸しを含む 6 種の業務記録が横断検索・表示でき（65 §65.10 slice 4d）、状態 filter 4 値「すべて / 有効 / 取消済み / 進行中」が WHERE に実効し、進行中棚卸しは「進行中」badge + 代表商品・明細数の両列「-」で表示される。

### 失敗定義

- 6 種のいずれかが hub 一覧に出ない、または既存 4 種の表示・検索・遷移に回帰が出る
- status filter が no-op のまま、または IO 早期リターン gate の 4 値拡張漏れにより canceled / in_progress 検索が常に 0 件になる
- stocktake の item_count が差異件数（`reference_type='stocktake' AND is_voided=0` の COUNT）にならない（stocktake_items 全件数化、cross-type id 衝突、is_voided 行混入による誤カウント）
- 単日検索（date_from = date_to = 完了日）で stocktake 行が消える（DATE() ラップ欠落）
- in_progress 棚卸しが「明細なし」等で誤読される表示になる、または filter 部注記が 65 §65.8.1 の確定文言と不一致
- `csvImportCommit` / `stocktakeStart` / `stocktakeComplete` 成功後に hub 一覧が stale のまま表示され続ける（D-052 拡張漏れ）

### 非目的

- 専用一覧 `/csv-import/records`・`/stocktake/records` と `listCsvImportRecords` / `listStocktakeRecords` の実装（完成形契約のまま runway 残置）
- CSV 出力・印刷（65 §65.10 slice 6）、取消/訂正機能と `corrected` status（slice 5）
- bindings の型変更（`InventoryRecordQuery` / `InventoryRecordSummary` の record_type / status は string のまま）
- page / per_page 境界の変更（UNION 後 LIMIT/OFFSET、種別数増加の影響なし — 触らない）
- 既存 6 詳細画面・既存 stocktake / csv-import command の変更

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

1. IO `src-tauri/src/db/disposal_repo.rs`: `RecordSpec` へ 5 field 追加 — `status_expr` / `item_count_expr` / `date_expr` / `created_at_col` / `filter_item_extra_where`（既定値は現行 4 種の挙動と同一。既存 `item_table` / `item_fk_col` は流用し、stocktake のみ `inventory_movements` / `reference_id` へ上書き — 21-io §10.5 / 設計判断 D-f）。`SPECS` へ csv_import / stocktake の 2 エントリ追加（csv: status_expr = `rolled_back→'canceled'` 他 `'active'`、date_expr = `settlement_date`、created_at_col = `imported_at`、item_count = 明細行数〈is_voided 行含む — D-d〉。stocktake: status_expr = `completed→'active'` / `in_progress→'in_progress'`、date_expr = `DATE(COALESCE(completed_at, started_at))`、created_at_col = `started_at`、`filter_item_extra_where` = reference_type / is_voided 絞り込みを dept EXISTS / keyword EXISTS / representative_item の 3 テンプレートへ適用、`item_count_expr` = 同条件を式内に埋め込んだ自己完結サブクエリ）
2. IO 同 file: `query.status` の WHERE 実反映 — `UNION ALL` 後の外側 derived table（`FROM (...) records`、count_sql `L450` / data_sql `L457-460` に実在確認済み）へ正規化済み `status` alias の等値条件を 1 回だけ適用（count / data 双方）。literal `'active' AS status`（L432）は per-spec の `status_expr` へ置換
3. IO 同 file: status 早期リターン gate（L255-262 `!matches!(query.status.as_deref(), None | Some("all") | Some("active"))`）を 4 値 allowlist（+ canceled / in_progress）へ拡張（拡張漏れだと canceled / in_progress が BIZ 通過後も空ページ短絡し新設 WHERE に到達しない — 21-io §10.5）
4. BIZ `src-tauri/src/biz/inventory_service/list.rs`: record_type allowlist（L81-92）へ csv_import / stocktake 追加、status allowlist（L93-95）を 4 値へ拡張（44-cmd §23.10: 公開シグネチャ不変、内部 allowlist のみ拡張）
5. CMD 層はコード変更なし（薄い委譲）。bindings は型不変のため差分ゼロ（AC4 で機械確認）
6. Rust test: 既存拒否 test `test_list_inventory_records_req206_rejects_unknown_record_type`（list.rs L518-537）の oracle を `csv_import` から実在しない token（`unknown_type`）へ差し替え（test 名・意図は不変 — 設計判断 D-e、既存 test 削除・無効化禁止に抵触しない意味追随）+ 新規 test 一式（Test Design Matrix T1-T16・T24）
7. D-052 invalidation 拡張（Revisit 条項に従い同一 PR で SSOT / oracle / 再実測を同期）: `csvImportCommit` / `stocktakeStart` へ `queryKeys.inventoryRecords.root()` を追加、`stocktakeComplete` は `stocktakeDetailRoot()` を `root()` へ置換（root() は prefix で stocktakeDetail を包含。6 種化により commit / start / complete が hub 一覧の行集合・status・item_count を変えるため。`csvImportRollback` の root() 先例と同型の最小集合化）。`productCreate` / `productImport` は変更なし（進行中棚卸しへの明細自動追加は hub の in_progress 行では両列「-」表示のため読取り集合と交差しない — 導出は PR body に記録）。`docs/decision-log.md` D-052 Contract 行は既存記述の訂正 + 追記（Plan Review round 1 P2-1: ①既存文「棚卸し詳細 prefix（`inventoryRecords.stocktakeDetailRoot()`）は C1 / C3 / C11 で stale 化する」を C1 / C3 の直接参照 + C11 は root() の prefix 包含経由へ訂正 ②既存文「C15 棚卸し開始 / C16 棚卸し明細個数更新は列レベル導出で非追加」を「C15 は slice 4d で root() 追加（hub 一覧に in_progress 行が出現するため）、C16 は非追加のまま」へ訂正 ③C8 csvImportCommit / C15 への root() 追加の 1 文追記。追記のみでは既存 2 文が実装後に文字通り誤りとして残存する）+ 独立転記 oracle test（`src/test/invalidation-oracle.ts`）追随
8. FE `src/features/inventory-records/types.ts`: `INVENTORY_RECORD_TYPE_OPTIONS` へ csv_import「CSV取込み」/ stocktake「棚卸し」追加（label は `resolve_movement_source`（list.rs L225-226）と一致）、`INVENTORY_RECORD_STATUS_OPTIONS` を 4 値へ（すべて / 有効 / 取消済み / 進行中）。`formatRecordStatus` はコード変更不要（options 配列を先に引く既存実装〈types.ts L84-87〉のため 4 値化で `in_progress` / `canceled` とも自動解決される — 重複 if 分岐を追加しない。Plan Review round 1 P3-1）
9. FE `src/features/inventory-records/InventoryRecordsPage.tsx`: filter 部に 65 §65.8.1 確定文言の 1 行注記を常設（「商品・部門での絞り込みは、CSV取込みでは取込み明細、棚卸しでは差異のあった商品が対象です。」）、stocktake × in_progress は代表商品・明細数の両列「-」表示、stocktake × active（completed）× item_count 0 は代表商品「差異なし」（明細数は `0` のまま。汎用 fallback「明細なし」を使わない）
10. FE test `InventoryRecordsPage.test.tsx`: 既存 options oracle test「REQ-206: 記録種別フィルターで4種の業務記録を選べる」（L67-98、単一 it() が種別 / 状態両配列を検証）を 6 種 + 4 値へ拡張し test 名の「4種」も「6種」へ改名 + csv / stocktake の href / SPA 遷移 test + 注記 / 「-」 / 「差異なし」表示 test（Matrix T17-T22）。既存 disposal 遷移 test は不変
11. REQ-206 / REQ-207 token 付き test 追加 → `cargo run --bin generate_traceability` で 90 再生成（R1 direct-impl drift generators の教訓）
12. commit 分割: backend → frontend → 生成物再生成の順（発注書に明記）

## Non-scope

- 専用一覧 route / command（`listCsvImportRecords` / `listStocktakeRecords`）— runway 残置
- CSV 出力・印刷（slice 6）、取消/訂正（slice 5、`corrected` status 含む）
- `csv_imports` / `stocktakes` / `inventory_movements` の schema 変更（なし）
- 既存 stocktake / csv-import / daily-report-import command 群の wire 変更（なし）
- 詳細画面 6 種の変更（なし）
- page / per_page・ORDER BY 機構の変更（date_expr は SELECT / WHERE の 2 箇所のみ、外側 ORDER BY は inner SELECT の `business_date` alias を継承するため独立変更不要）
- daily report import の hub 合流（`daily_report_imports` は別 table で `csv_imports` を書かない — 実査済み、hub 非対象）

## Acceptance Criteria

- AC1（変更前 canary）: main 時点で hub が 4 種のみである実出力（`rg -c "csv_import" src-tauri/src/db/disposal_repo.rs` = 0〈exit 1〉）を PR body に収録する
- AC2: `cd src-tauri && cargo test` green（Matrix T1-T16・T24 の新規 test + 既存 suite を含む）
- AC3: `npm test` green（Matrix T17-T23 を含む）
- AC4: `cargo run --bin generate_bindings` 再実行後 `git diff --exit-code src/lib/bindings.ts` = 0（型不変 = 差分ゼロ）
- AC5: 拒否 test の oracle 差し替え（`rg -c "unknown_type" src-tauri/src/biz/inventory_service/list.rs` ≥ 1 かつ 当該 test 内 `csv_import` oracle 0）で test 名不変のまま green
- AC6: `cargo test test_list_inventory_records_req206_filter_canceled` green（T5 — csv rolled_back のみ hit・他 5 種除外の非空期待）
- AC7: `cargo test test_list_inventory_records_req206_cross_type_id_collision` green（T8 — 同一数値 id seed + item_count 数値検証）
- AC8: `cargo test test_list_inventory_records_req206_stocktake_single_day_boundary` green（T11 — date_from = date_to = 完了日で該当行が出る）
- AC9: `src/features/inventory-records/InventoryRecordsPage.test.tsx` の注記 exact 文言 test（T18、§65.8.1 確定文言の独立転記 oracle）が `npm test` で green
- AC10: `src/test/invalidation-oracle.ts` の D-052 oracle test（T23）が csvImportCommit / stocktakeStart / stocktakeComplete の新集合と順序非依存・重複検出付き完全一致（stocktakeComplete の stocktakeDetailRoot() → root() 置換、productCreate / productImport の非変更を含む集合一致）で `npm test` green
- AC11: `cargo run --bin generate_traceability` 再生成で 90 が clean（REQ-206/207 の新規 test 行が反映され、再実行で diff なし）
- AC12: `bash scripts/local-ci.sh full` CLEAN（L1、exact-HEAD evidence は PR body 所管）
- AC13: Human Gate に L3 を含むため Writer 完了条件に `cargo check --release`（CI gate ではない）

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-206（業務記録の一覧・詳細追跡、対応 UI に UI-07b / UI-10b）/ REQ-207（movement → 元記録の相互参照）
- Architecture: `docs/ARCHITECTURE.md`（UI → CMD → BIZ → IO 一方向）
- Function / command / DTO: [65-inventory-record-traceability.md](../../function-design/65-inventory-record-traceability.md) §65.4.1（L78-80: 検索母集団・状態 filter 4 値・外側 derived table 適用・in_progress 構造的非 hit）/ §65.6.1（L124-132: status 正規化写像表）/ §65.8.1（L206-216: hub UI 完成形 — 注記確定文言・「進行中」badge・両列「-」・「差異なし」）/ §65.10 slice 4d（L268-274）/ §65.12 変更履歴（L306）、[21-io-inventory-repo.md](../../function-design/21-io-inventory-repo.md) §10.5 `list_inventory_records`（L254〜: record_type 6 種・status 4 値と外側適用・date_expr・created_at_col・空ページ gate 4 値）、[44-cmd-inventory.md](../../function-design/44-cmd-inventory.md) §23.7（L462〜: 6 種・4 値）/ §23.10（L994〜: BIZ wrapper シグネチャ不変）
- DB: `src-tauri/src/db/schema_v1.rs` csv_imports status CHECK 3 値（L140）/ stocktakes status CHECK 2 値（L212）— schema 変更なし。`docs/db-design/` の該当 table 定義
- Screen / UI: 65 §65.8.1、`docs/SCREEN_DESIGN.md`（6 種明記済み・改訂不要）、`docs/UI_TECH_STACK.md` §2.5（D-052 導出原則）、`docs/design-system/README.md` 一式
- Decision log / ADR: D-052（invalidation SSOT + Revisit 条項）、D-050 / D-035 / D-038（Evidence Ownership）。owner 裁定 2026-08-27/28 は 65 §65.10 slice 4d に正本化済み

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status |
|---|---|---|
| Backend function / command / repository / validation / error | 21-io §10.5 / 44-cmd §23.7・§23.10 / 65 §65.4.1・§65.6.1 | existing sufficient（PR #13 で改訂済み） |
| Command / DTO / generated binding / wire shape | 44-cmd §23.7 + 本 packet `Boundary / Wire Contract`（型不変・値域拡張のみ） | existing sufficient |
| DB / transaction / audit / rollback / migration | schema_v1.rs CHECK 値（read-only、schema 変更なし） | existing sufficient |
| Screen / UI / route state / Japanese wording | 65 §65.8.1（確定文言・badge・「-」・「差異なし」） | existing sufficient（PR #13 で改訂済み） |
| CSV / TSV / report / import / export format | — | 触らない |
| Durable decision / ADR | decision-log D-052 Contract 行 | updated in this PR（既存記述の訂正 + 追記 — Scope 7） |

## Registration / Generation Obligations

| 新規追加物 | 登録・生成義務 | 対応 |
|---|---|---|
| Tauri command | — | 該当なし（新規 command なし。既存 `list_inventory_records` の内部拡張のみ、bindings 差分ゼロを AC4 で機械確認） |
| function-design doc 新設 | — | 該当なし（既存 doc は PR #13 改訂済み） |
| source / workflow doc 新設・改名 | — | 該当なし |
| Consultation Relay | — | 不使用 |
| REQ coverage 追加 | `generate_traceability` で 90 再生成 | Scope 11 / AC11 |
| route 新設 | — | 該当なし（既存 hub の拡張。csv / stocktake 詳細 route は slice 4b / 4c 実装済み） |
| operator 画面新設 | — | 該当なし（既存画面の拡張、navigation entry 既存）。csv / stocktake 行 → 詳細への到達導線は T22（userEvent.click SPA 遷移）で担保 |

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-206（6 種横断検索） | 65 §65.10 slice 4d / 21-io §10.5 | 設計判断 D-f（PR #13） | `RecordSpec` 拡張で単一 UNION ALL 維持。種別別 SQL 分岐は抽象の放棄でコスト過大。追加 field 5 つ・既存 field 流用（filter_item_table 新設は死重で不採用） | disposal_repo.rs Scope 1 | T1 / T14 |
| REQ-206（status 正規化 + filter 実効） | 65 §65.4.1 / §65.6.1 / 21-io §10.5 | D-a | IO 層 SQL CASE 式 + 外側 derived table WHERE 1 回適用。従来 no-op の query.status を実反映。gate 4 値拡張が対 | Scope 1-3 | T2 / T3 / T4 / T5 / T6 |
| REQ-206（stocktake item_count = 差異件数） | 65 §65.10 slice 4d / §65.8.1 | D-b | movements 母集団統一（stocktake_items は全対象商品を持ち母集団問題）。多態 FK 衝突対策で item_count_expr は自己完結 SQL | Scope 1 | T7 / T8 |
| REQ-206（csv item_count = is_voided 行を含む明細行数） | 65 §65.10 slice 4d / 21-io §10.5 | D-d | TRACE-D6: is_voided は取込み無効化用途で履歴事実は見せる（取消済みは status badge が伝える）。voided 除外案は履歴保持方針に反し不採用 | Scope 1（csv は既定 template のまま filter 句を追加しない） | T25 |
| REQ-206（business_date） | 65 §65.4.1 / 21-io §10.5 | D-c | csv=settlement_date（精算日 = 業務確定日）、stocktake=DATE(COALESCE(completed_at, started_at))。datetime と date-only の BINARY 比較は境界日を取りこぼす（sqlite3 実測済み — PR #13 rally r3 P1） | Scope 1 | T9 / T10 / T11 |
| REQ-206（in_progress の filter 意味論） | 65 §65.4.1（構造的非 hit の既知の制約） | D-i | 差異 movement 0 件のため EXISTS 常 false は filter 意味論の正しい帰結。例外ロジック不採用 | Scope 1 | T13 |
| REQ-206（拒否 test の意味追随） | — | D-e | csv_import が正当種別化するため oracle を unknown_type へ差し替え。test 削除・無効化禁止に抵触しない | Scope 6 | T15 / AC5 |
| REQ-206（hub UI 表示） | 65 §65.8.1 | D-h | 注記常設 + in_progress 両列「-」（0 と算定前の区別）+ completed×差異 0 の「差異なし」（「明細なし」誤読防止） | Scope 8-9 | T17-T21 |
| REQ-207（csv / stocktake 行 → 詳細遷移） | 65 §65.3 / slice 4b・4c 実装済み route | — | 詳細 route は実装済み。hub 行からの href / SPA 遷移のみ追加 | Scope 9-10 | T22 |
| D-052（hub 一覧の invalidation） | UI_TECH_STACK §2.5 / decision-log D-052 | D-052 Revisit | 6 種化で csvImportCommit / stocktakeStart / stocktakeComplete が hub 行集合を変える。root() 追加（rollback 先例と同型）。stocktakeComplete は prefix 包含により stocktakeDetailRoot() を root() へ置換。productCreate / productImport は非変更（in_progress 行は両列「-」で読取り集合と交差しない） | Scope 7 | T23 / AC10 |

## Design Intent Audit

- Source docs can answer what/why without chat history: yes — 65 slice 4d が owner 裁定・6 種契約・表示規定を、21-io §10.5 / 44-cmd §23.7・§23.10 が層別実装契約を記載（PR #13 で確定済み）
- Plan-only durable decisions promoted: D-052 consumer 集合拡張は decision-log Contract 行へ同一 PR で追記（Scope 7）。それ以外の設計判断はすべて PR #13 で source docs 正本化済み
- Assumptions / constraints: schema CHECK 値（csv 3 値 / stocktake 2 値）、各ヘッダ table の AUTOINCREMENT 個別採番による id 数値衝突の常態（多態 FK）、既存 4 種は新 field 既定値で挙動不変、SQLite CASE / COALESCE / DATE() は標準機能
- Deferred design gaps: 専用一覧 runway、slice 5（取消/訂正・corrected）、slice 6（CSV 出力・印刷）
- Test Design Matrix cites decision IDs: yes
- Absolute guarantee self-check: 「in_progress 棚卸しは差異 movement 0 件」は補正 movement を `complete_stocktake` のみが書く既存保証に依存（PR #9 packet Contract Probe で `stocktake` reference の producer 単一性を検証済み、以後の変更なし）。例外なし

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | app-core の一覧表示のみ。POS / CV17 等 adapter 事実に触れない | — |
| Fact check / design decision split | D-052 gap は Coordinator が invalidation-contract.ts L82-122 実読で確認（csvImportCommit / stocktakeStart / stocktakeComplete に inventoryRecords.root() 不在）。daily report import の hub 非対象は `INSERT INTO csv_imports` が sales_repo.rs のみ + daily_report_imports 別 table を rg で確認。実装 anchor（gate L255-262 / literal L432 / 外側 derived table L450・L457-460 / SPECS 4 エントリ）は 2026-08-29 実査で pre-image 確認済み | 本 packet + PR body |
| Lifecycle / retry | hub 一覧 query の invalidate / refetch（9 mutation）と in_progress → active の状態遷移表示を State Lifecycle Matrix でカバー | Test Matrix |
| Operator workflow | filter 4 値の実効化（従来 no-op）、注記常設、「-」/「差異なし」の誤読防止 | Matrix T17-T21 + L3 |
| Replacement path | not applicable（外部システム非接触） | — |
| Data safety / evidence | synthetic fixture のみ。実 POS / 実棚卸しデータ非 commit | Data Safety 節 |
| Reporting / accounting semantics | 表示のみで集計意味論不変。差異件数は既存詳細画面の語義（補正 movement 正）と一致 | — |
| Manual verification | L3 視認 1 系統（6 種混在 + 各 filter + in_progress / 差異 0 表示） | Human Gate |
| 環境・再現性 | 新設の環境依存なし | — |

## Design Readiness

- Existing design docs are sufficient because: PR #13（squash `6c688fe`）が 65 / 21-io / 44-cmd / 73-ui / 55-ui の 5 doc 11 節へ slice 4d 契約（status 正規化・差異件数・検索母集団・date_expr・created_at_col・gate 4 値・hub UI 表示規定・確定文言）を確定済み。Final Review で「改訂済み source docs だけで実装可能」を確認済み
- Source docs updated in this PR: `docs/decision-log.md` D-052 Contract 行の既存記述訂正 + 追記のみ（Revisit 条項の同一 PR 同期 — Scope 7）
- Design gaps intentionally deferred: 専用一覧 runway / slice 5 / slice 6
- Durable decisions discovered and promoted: D-052 consumer 集合拡張（csvImportCommit / stocktakeStart / stocktakeComplete、6 種化起因）

Minimum design checks:

- Layer ownership: UI（options / 注記 / 表示分岐）→ CMD（変更なし）→ BIZ（allowlist 拡張のみ）→ IO（SQL 契約の本体）
- Backend function design: 21-io §10.5 / 44-cmd §23.7・§23.10
- Command / DTO / data contract: Boundary / Wire Contract 節（型不変・値域拡張）
- Persistence / transaction / audit impact: read-only 一覧、TX 不要、操作ログ記録なし
- Operator workflow / Japanese wording: §65.8.1 確定文言・「進行中」「取消済み」label（既存 formatRecordStatus と一致）・色だけに依存しない badge
- Error / empty / retry / recovery: 未知 record_type / status の BIZ 拒否（日本語文言）、空結果、in_progress の「-」表示
- Testability / traceability: REQ-206 / REQ-207 token、90 再生成

## Contract Probe

N/A — 新規の未検証外部前提なし。依存する前提はすべて検証済み: stocktake datetime vs date-only の DATE() 境界挙動は PR #13 design rally r3 で sqlite3 実測済み、外側 derived table・status gate・SPECS 構造は 2026-08-29 の repo 実査（Impact Review Lenses Fact check 行)で pre-image 確認済み。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| 65 §65.10 slice 4d: record_type 6 種横断（all = 6 種） | SPECS 2 エントリ追加 | T1（6 種 seed 混在一覧 + business_date 降順） | L3 視認 |
| 65 §65.6.1 写像表: csv completed / completed_partial → active、rolled_back → canceled | csv status_expr | T2（completed_partial は completed と区別した個別 seed — CASE 分岐見落とし mutation 検出用） | — |
| 65 §65.6.1 写像表: stocktake completed → active、in_progress → in_progress | stocktake status_expr | T2 | — |
| 65 §65.4.1: status filter 外側 derived table へ 1 回適用（count / data 双方） | 外側 WHERE 新設 | T3（filter=active で rolled_back / in_progress 除外 + total_count 整合） | — |
| 65 §65.4.1: filter=in_progress で stocktake のみ | 同上 | T4（非空期待） | — |
| 65 §65.4.1: filter=canceled で csv rolled_back のみ・他 5 種除外 | 同上 | T5（非空期待） | — |
| 21-io §10.5: status 早期リターン gate 4 値拡張 | gate L255-262 | T6（canceled / in_progress 指定で空ページ短絡しない — T4 / T5 と独立に gate 通過を検証） | — |
| 65 slice 4d / D-b: stocktake item_count = 差異件数（reference_type='stocktake' AND is_voided=0） | item_count_expr 自己完結 SQL | T7（is_voided 行 seed 込みの数値検証） | — |
| D-f 多態 FK: cross-type id 衝突の非誤ヒット（dept / keyword / representative_item / item_count の 4 箇所） | filter_item_extra_where 3 テンプレート + item_count_expr | T8（collision fixture: 同一数値 id の disposal_record と stocktake を意図的 seed、非空期待 + item_count 数値検証） | — |
| 65 slice 4d / D-d: csv の item_count / representative_item は is_voided 行を含む（filter 句なし = 既定 template 継続、gated Amendment 1） | csv SPECS エントリ（item_count_expr = None / filter_item_extra_where = ""） | T25（is_voided=1 明細を含む rolled_back csv の item_count 数値 + representative_item 実商品名 assert） | — |
| D-c: csv business_date = settlement_date | csv date_expr | T9（settlement_date で検索 hit / imported_at 日では非 hit の対 oracle） | — |
| D-c: stocktake business_date = COALESCE fallback（in_progress は started_at） | stocktake date_expr | T10 | — |
| D-c: DATE() ラップの単日検索境界 | 同上 | T11（date_from = date_to = 完了日で該当行が出る。ラップ欠落だと date_to 境界で消える） | — |
| 21-io §10.5 / 65 slice 4d: created_at_col（記録日時）csv=`imported_at` / stocktake=`started_at`（両 table に created_at 列なし — L433 ハードコードの置換） | created_at_col field + SELECT 置換 | T24（csv / stocktake seed で記録日時値が imported_at / started_at と一致し business_date と異なることを assert） | — |
| 65 §65.4.1 / D-i: in_progress × dept / keyword filter の構造的非 hit（既知の制約） | filter テンプレート | T13（filter 指定で非 hit + filter 無指定で表示の対 oracle） | — |
| 既存 4 種 regression: 新 field 既定値継承で挙動同一 | RecordSpec 既定値 | T14（既存 4 種の一覧・filter・件数が変更前後で同一）+ 既存 suite green（AC2） | — |
| D-e: 拒否 test oracle 差し替え（意味追随、test 名不変） | list.rs 拒否 test | T15 / AC5 | — |
| 44-cmd §23.10: BIZ 公開シグネチャ不変・allowlist のみ拡張 | list.rs allowlist | T16（6 種受理 / 未知種別・未知 status 拒否）+ AC4（bindings 差分ゼロ） | — |
| rally r5 P3: filter 母集団と item_count 母集団の同一集合（重複条件の意図的保持を comment 明示） | disposal_repo.rs comment + test | T12（同一 fixture で filter hit 集合と item_count が同一母集団由来である regression） | — |
| 65 §65.8.1: 種別 options 6 種 + 状態 options 4 値（label は resolve_movement_source と一致） | types.ts | T17（options 配列 oracle 独立転記、test 名 6 種化） | — |
| 65 §65.8.1: 母集団差注記の確定文言常設 | InventoryRecordsPage.tsx filter 部 | T18（exact 文言 oracle 独立転記） | L3 視認 |
| 65 §65.8.1: in_progress は代表商品・明細数とも「-」+「進行中」badge（色だけに依存しない） | 同 Page | T19（対象商品を持つ in_progress seed で両列「-」+ label text assert） | L3 視認 |
| 65 §65.8.1: completed × 差異 0 件は代表商品「差異なし」・明細数 `0`（「明細なし」不使用） | 同 Page | T20（全数一致完了 seed の独立 test） | L3 視認 |
| formatRecordStatus: in_progress → 進行中（STATUS_OPTIONS 4 値化経由 — options 先行検索の既存実装で自動解決、if 分岐追加なし） | types.ts STATUS_OPTIONS | T21 | — |
| REQ-207: csv / stocktake 行 → 詳細への SPA 遷移 | 同 Page（既存 href 機構） | T22（userEvent.click → 遷移後 render assert、href 単独不可） | L3 視認 |
| D-052 拡張: csvImportCommit / stocktakeStart へ root() 追加、stocktakeComplete は stocktakeDetailRoot() → root() 置換 | invalidation-contract.ts + oracle | T23 / AC10 | — |
| D-052 Revisit: decision-log Contract 行の既存記述訂正 + 追記（Scope 7 ①②③） | docs/decision-log.md | 実装 review で diff 確認（doc 行） | — |
| D-052: productCreate / productImport 非変更の導出記録 | PR body | — | 導出根拠を PR body 記録 |
| REQ token → 90 再生成 | generate_traceability | AC11 | — |
| 既存 flow 回帰なし（既存 4 種 hub・6 詳細画面・stocktake / csv-import flow） | — | 既存 suite green（AC2 / AC3） | — |

隣接契約 sweep 実施記録: 65 §65.4.1 の他 filter 行（記録 ID / 日付範囲 / 部門の既存機構）は per-spec 化の対象外で既存挙動維持（T14 が固定）。21-io §10.5 の page / per_page は「触るな」を Non-scope に明記。44-cmd §23.7 の他 command 群は無改変。65 §65.8.1 の結果列構成（記録種別 / 記録 ID / 業務日付 / 代表商品 / 明細数 / 状態 / 記録日時 / 詳細ボタン）は既存実装の列構成のまま（新列追加なし、値の種別別意味のみ拡張）。73-ui UI-10-D5 / §73.14 / 55-ui §55.3 の runway 注記は PR #13 で改訂済み・本 PR 非接触。daily report import は csv_imports 非書込みで hub 非対象（Non-scope 明記）。

## Test Plan

Test Design Matrix: [test-matrices/2026-08-29-records-six-symmetry-impl.md](test-matrices/2026-08-29-records-six-symmetry-impl.md)

- targeted tests: IO SQL 契約（正規化・差異件数・date_expr・collision・WHERE 実効）、BIZ allowlist、FE options / 注記 / 表示分岐 / SPA 遷移、D-052 oracle
- negative tests: 未知 record_type / status 拒否、in_progress × filter 非 hit、単日境界、is_voided 混入
- compatibility checks: 既存 4 種 regression（T14）、bindings 差分ゼロ（AC4）、既存 disposal 遷移 test 不変
- data safety checks: synthetic fixture のみ
- main wiring/integration checks: hub 一覧 → 詳細 SPA 遷移（T22）、D-052 invalidation（T23）、90 再生成（AC11）
- Human Gate に L3 を含むため、Writer 完了条件に `cargo check --release` を含める（AC13、CI gate ではない）

## Boundary / Wire Contract

- producer: `cmd::list_inventory_records`（既存、コード変更なし）
- consumer: `InventoryRecordsPage`（`queryKeys.inventoryRecords.list(query)` の useQuery）
- wire type: `InventoryRecordQuery` / `InventoryRecordSummary`（record_type / status は string のまま型不変 — 44-cmd §23.10。bindings 差分ゼロを AC4 で機械確認）
- internal type: `RecordSpec`（IO 内部、5 field 追加）— wire に露出しない
- precision/range: item_count（差異件数 COUNT 含む）・id は i64、JS safe integer 内（既存と同等）
- round-trip path: DB → UNION ALL SQL → IO DTO → BIZ → bindings → UI 表示のみ（書込みなし）
- invalid input: 未知 record_type / status → `BizError::ValidationFailed`（利用者向け日本語文言、既存経路）
- compatibility: 値域拡張のみ — record_type 実値 4 → 6 種、status 実値は従来 `'active'` 固定 → 3 値（active / canceled / in_progress）、query.status 許容 4 値。既存 4 種の行の wire 値は不変。FE `formatRecordStatus` は未知値 fallback 実装済みのため旧 FE × 新 backend の中間状態でも表示が壊れない

## Review Focus

- RecordSpec 既定値の regression（既存 4 種の挙動・SQL 出力が変更前と同一か — T14 の弁別性）
- item_count_expr（自己完結 SQL）と filter_item_extra_where（3 テンプレート適用）の対応関係 — 片側だけの適用は cross-type 衝突・差異件数誤カウントを残す（PR #13 rally r4 P1 の型）
- gate 4 値拡張漏れ（canceled / in_progress 検索が常に 0 件になる型 — T6 が T4 / T5 と独立に gate を検証しているか）
- 空集合 oracle 衝突の回避（T4 / T5 / T8 / T11 の主 oracle が非空期待か — 順22 X2 の教訓)
- 注記文言・「差異なし」・options label の独立転記（production 定数 / SSOT を import しない — D-052 既存 gate + 順6 教訓）
- D-052 拡張の欠落 / 過剰両方向（stocktakeComplete の root() 置換で stocktakeDetailRoot() 包含が失われないか、productCreate / productImport 非変更の導出、csvImportCommit の root() 追加が dailyReportImport へ波及しない根拠）
- 単日境界 T11 の fixture が完了日時を date_to と同日の遅い時刻（例 18:00）にしているか（早い時刻では DATE() 欠落でも通る）

## Spec Contract

Contract ID: SPEC-65-4D-SIX-SYMMETRY-2026-08-29

- `/inventory/records` は 6 種（receiving_record / return_record / manual_sale / disposal_record / csv_import / stocktake）を横断検索し、`all` は 6 種横断とする
- status は IO 層 SQL CASE で active / canceled / in_progress へ正規化し、query.status 4 値（all / active / canceled / in_progress）を UNION ALL 後の外側 derived table へ 1 回だけ WHERE 適用する（count / data 双方）
- stocktake の item_count は `reference_type='stocktake' AND is_voided=0` の差異件数、business_date は `DATE(COALESCE(completed_at, started_at))`。csv の item_count は is_voided 行を含む明細行数、business_date は `settlement_date`
- 商品 / 部門 filter の母集団は csv = sale_records、stocktake = 差異 movements（in_progress は構造的に非 hit — 既知の制約）
- in_progress 行は「進行中」badge + 代表商品・明細数の両列「-」。completed × 差異 0 件は代表商品「差異なし」・明細数 `0`（「明細なし」不使用）
- filter 部に「商品・部門での絞り込みは、CSV取込みでは取込み明細、棚卸しでは差異のあった商品が対象です。」を常時表示する
- `csvImportCommit` / `stocktakeStart` / `stocktakeComplete` 成功時、hub 一覧 query は D-052 SSOT 経由で invalidate される

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| REQ-206（6 種横断） | Scope 1 / 4 | T1 / T14 | RecordSpec 既定値 regression | PR body + Matrix |
| REQ-206（正規化 + filter 実効） | Scope 1-4 | T2-T6 | gate 拡張漏れ・外側 WHERE | Matrix |
| REQ-206（差異件数 + 衝突） | Scope 1 | T7 / T8 | expr / extra_where 対応関係 | Matrix |
| REQ-206（business_date） | Scope 1 | T9-T11 | 単日境界 fixture の時刻設計 | Matrix |
| REQ-206（記録日時 created_at_col） | Scope 1 | T24 | date_expr との取り違え弁別 | Matrix |
| REQ-206（hub UI 表示） | Scope 8-10 | T17-T21 / AC9 | 文言独立転記 | PR body |
| REQ-207（詳細遷移） | Scope 9-10 | T22 | click SPA 遷移証明 | Matrix |
| D-i（構造的非 hit） | Scope 1 | T13 | 対 oracle | Matrix |
| D-052 拡張 | Scope 7 | T23 / AC10 | 欠落 / 過剰両方向 | PR body + decision-log diff |

## Data Safety

- 実 POS データ・実棚卸しデータ・実 DB file は commit しない
- test fixture・L3 用データは synthetic のみ（既存 test 慣行に従う）
- L3 手順（backup → synthetic 6 種 seed〈in_progress 棚卸し・差異 0 完了棚卸し・rolled_back csv を含む〉→ hub 表示 / filter 4 値 / 注記 / 遷移確認 → restore 復元）は Ready 依頼と同時に PR body へ記載する（L3 fixture prep の教訓）
- `.local/ci-evidence/` はローカルのみ

## Implementation Results

- IO / BIZ は6種横断、3値status正規化、4値filter、棚卸し差異母集団、種別別の日付・記録日時写像を実装し、既存4種の既定挙動を維持した。
- hub UI は6種・4値の選択肢、検索母集団差の常設注記、進行中棚卸しの両列「-」、完了差異0件の「差異なし」、CSV取込み / 棚卸し詳細へのSPA遷移を実装した。
- D-052 SSOT / 独立oracle / decision-log を6種化のconsumer集合へ同期し、REQ-206 / REQ-207 traceabilityを再生成した。wire型・schema・公開シグネチャは不変。
- Draft PR: https://github.com/higekiri-w/inventory-system-desktop/pull/14

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

Pre-PR review-only pass は Contract Audit とtargeted検証を完了し、P1 / P2 / P3 はいずれもなし。正式な Final Review と Coordinator mutation 独立再実測は Draft 後に実施する。
- Findings Freeze: not yet frozen; post-freeze exceptions: none.

Phase 遷移記録（本 final content commit に同乗）: `implementing -> local-verified`。Writer の L1 full は PASS / CLEAN、追加gateとmutation X1-X11は復元後green、pre-PR review-only passはblockerなし。exact-HEAD evidenceはD-035/D-038どおりPR bodyを正本とし、tracked fileへSHA・test件数を転記しない。残りは正式なFinal Review + Coordinator mutation独立再実測、Windows native L3、Ready承認、hosted final、merge。

### Plan Review rally 記録（2026-08-29、append-only）

- round 1（Claude Sonnet 5 独立 fresh context、対象 = plan-first commit `1f4ba47`）: P1×1（created_at_col〈記録日時〉契約の Ledger 行・専用 test 欠落 — disposal_repo.rs L433 の `{header_alias}.created_at` ハードコードは csv/stocktake に created_at 列が無いため置換必須なのに、値の取り違えを検出する assert が Matrix に無い）/ P2×1（decision-log D-052 Contract 行の既存 2 文〈stocktakeDetailRoot の C1/C3/C11 stale 化・C15 非追加〉が本 change の root() 置換・C15 root() 追加と矛盾したまま残存するリスク — 追記のみでは不足、訂正の義務化が必要）/ P3×1（formatRecordStatus への in_progress 追加記述は options 先行検索の既存実装〈types.ts L84-87〉と重複し、不要な if 分岐追加を誘発し得る）。Coordinator が全 3 件の引用 file:line（disposal_repo.rs L433 / schema_v1.rs L140・L212 / decision-log.md D-052 Contract 行 / types.ts L83-91）を実読で独立裏取りし全件 accept。是正 = Ledger created_at_col 行 + Matrix T24・X11 追加、Scope 7 を「既存記述の訂正 + 追記」へ改訂、Scope 8 の formatRecordStatus 記述を「コード変更不要」へ是正、旧前提の packet / Matrix 全節 rg sweep 済み。観点 b（oracle 妥当性）/ c（D-052 導出 — 欠落・過剰・dailyReportImport 非対象・呼出し site 実効性）/ d / e / f / g / h は指摘なし
- round 2 closure（別 Sonnet fresh context、対象 = 是正 commit `2941807`）: 3/3 CLOSED（T24 fixture の弁別性〈created_at_col と date_expr の取り違え検出〉、Scope 7 訂正指示と decision-log D-052 実文言の verbatim 突合、types.ts L83-91 実装との整合をそれぞれ実読確認）、旧前提 sweep hit 0、delta 起因の新規 findings 0。**P1/P2 残 0**
- owner Plan Gate 承認（2026-08-29、介入 1/3）
- plan-gate → plan-approved → implementing の materialize evidence: 上記 P1/P2=0、plan-first commit `1f4ba47` + rally 是正 `2941807` = Plan Commit が全実装 commit に先行（実装 commit 未作成）、Writer は Codex（発注書駆動）

### review-only 一次記録（2026-08-29、append-only）

- round 1（Claude Sonnet 5 独立 fresh context、対象 = `d771c88..95a8022`）: **P1 = 0** / P2×1（Scope 1 の D-d 契約〈csv の item_count / representative_item は is_voided 行を含む〉が Design Intent Trace・Contract Coverage Ledger・Test Design Matrix のいずれにも対応行を持たず、rolled_back csv の item_count / representative_item を assert する test も不在 — is_voided filter を csv 側へ誤追加する mutant が無検出で survive する negative-space gap）/ P3×1（decision-log D-052 Contract 行で C11 の root() 置換が訂正文①と追記文③の 2 箇所で重複記述 — 機能上の誤りなし）。観点 1（SQL 生成・injection 面・既存 4 種 regression）/ 2（設計契約突合）/ 4（D-052 実変更・oracle 独立転記・prefix 包含の技術的裏付け）/ 5（scope 突合 — packet 外 hunk は search.test.ts +4 行のみで options 第二 oracle への必然的随伴と適合判定）/ 6（層境界・bindings 差分ゼロ）/ 7（state-only 95a8022 の allowlist 適合）/ 8（test 品質・REQ token・90 再生成)は指摘なし
- Coordinator 裁定: P2-1 は rg + 実読（実測: `rg -n "item_count" src-tauri/src/db/disposal_repo.rs` → assert 行の実測出力は L1160〈disposal 既存 test〉/ L1516・L1564〈stocktake T7 / T8〉のみで csv 種別の assert は 0 件）で独立実証のうえ **accept** — gated Amendment 1（Ledger D-d 行 + Design Intent Trace D-d 行 + Matrix T25 / X12 追補）+ Codex relay round 2 で test 追加。P3-1 **accept** — 同 relay で重複 1 文の dedup。code fix を要するため implementing へ backtrack（本 commit）
- round 2 Writer 是正: T25 を追加し、X12 実注入で RED → 復元後 GREEN を確認。D-052 C11 記述を意味不変で dedup し、REQ-206 traceability を再生成した。L1 full は PASS / CLEAN。`implementing -> local-verified` に再遷移し、正式な Final Review と Coordinator mutation 独立再実測は引き続き未実施。

### Final Review 記録（2026-08-29、append-only）

- Sonnet 独立 fresh context の Contract Audit（対象 = Reviewed Content HEAD `6b9fb32`）: Contract Coverage Ledger **27/27 行適合**（不適合 0、65 §65.4.1 / 21-io §10.5 の通読による negative-space 検出 0）。AC 独立再実測 = AC2 / AC3 / AC4 / AC5 / AC11 を reviewer 自身が実行し全 PASS（bindings・traceability は再生成後 diff なし + tree clean）、AC1 / AC12 は PR #14 body との整合確認。round 2 delta 適合（T25 fixture は Matrix どおり / decision-log C11 dedup は意味不変 / gated Amendment 1 と Amendments 行整合）。anti-tautology（独立転記 oracle の production 定数非 import）/ Non-scope 遵守（packet 外は search.test.ts +4 の必然的随伴のみ）/ state-only 3 commit の hunk audit / PK5 ancestry（`git merge-base --is-ancestor` で Plan Commit `2941807` が初回実装 commit `64f4013` に先行）すべて適合。**P1/P2/P3 = 0**
- Coordinator mutation 独立再実測: **X1〜X12 の 12/12 全 kill、survivor 0**（clean tree HEAD `6b9fb32`、注入形は Matrix から独立導出・Writer 記録非参照。各注入 → 対応 test red → git restore → `git status --porcelain` 0 検分、最終 green は targeted Rust 21/21・FE 19/19）。X4 は T7 に加え T12〈母集団整合 regression〉も red の二重防御、X11 / X12 は T24 / T25 が単独 red となり Plan Review / review-only で追補した 2 契約の弁別性を実証
- local-verified → independent-review → human-confirm の materialize evidence: 上記 Contract Audit P1/P2 = 0 + mutation 全 kill。残 Human Gate = owner Windows native L3 目視 + Ready 承認 + merge

### owner L3 PASS と ready-hosted-final 遷移（2026-08-29、append-only）

- owner Windows native L3（content = Reviewed Content HEAD `6b9fb32`）: **全項目 PASS** — 6 種・4 状態の履歴表示 / 進行中棚卸しの両列「-」/ 差異 0 棚卸しの「差異なし」+ 明細数 0 / CSV取込み・棚卸し詳細への遷移 / 基準バックアップへの復元 / L3 用 synthetic データの消失確認（介入 2/3）。PR #14 を阻害する不具合なし
- owner 所感の UI 改善候補 5 件（成功通知の視認性 / 復元後の先頭スクロール / 取消確認の整形 / 戻り先の検索維持 / 状態表現の統一）は**別スコープ候補**と owner 判定 — Post-Merge Closeout で Plans.md backlog（表示磨き batch 候補）へ起票する
- human-confirm → ready-hosted-final の evidence: Human Gate 全項目解決（上記 L3 PASS）+ owner Ready 承認（2026-08-29、介入 3/3。L3 報告と同一 message で同時付与）。遷移は Draft のまま本 state-only commit で作成し、その exact HEAD で L1 full → PR body 全面 refresh → owner Ready 化 / hosted 三点一致の順（DEV_WORKFLOW 遷移規則どおり）
