# Plan Packet: 打ち間違えた入出庫の記録を取り消す（導入の前の関門 18）

導入の前の関門 16〜18（[Plans](../Plans.md#導入の前の関門)）の 2 本目の lane（順は 16 → 18 → 17。関門 16 の packet〈branch `agent/gates-16-18-design`、`99964d72`〉の「分け方」）。出所は [backlog](../backlog.md) の「入出庫の記録の取消の runtime」、関門 16 の起票の申し送り（同じ branch の backlog の同じ項目）、owner の決定（repo 外の回答台帳 TD-833〈2026-11-19、取消を導入の前に。振替は起票で決める〉・TD-846〈2026-11-20、関門 16 は店の答え待ちの間に 18 を先に起票〉）。起点は `ee93eb5c`（origin/main、2026-11-20）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason: `docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行の「DB」（4 つの header に 2 列、`sale_records` に 1 列と index の migration）、「Tauri command の DTO」（新しい command 1 つ、詳細 4 つと直近の一覧 3 つの DTO に field）、「状態を持つ画面・operator の業務の流れ」（記録の詳細の取消の操作と取消済みの表示）に当たる。R4 の行には当たらないと判断した: 取消は行を消さない論理の無効化（`is_voided`。Z004 の取消と同じ形）で、header・明細・movement・売上の行は残り、migration は `ADD COLUMN` と index だけで表を作り直さず既存の値を変えない。本番の DB は無い（[project-memory](../project-memory.md)、ADR SPEC-STK-CNT-D10）。owner は TD-833 で「R3〜R4 見込み」としたので、R4 を選べば `## Recovery` と `## Data Safety` を足す（下の Q3）。workflow の file（`.github/`・`scripts/`）に触れない。

## 取消の形の比べ（決めた形: 元の movement を無効にして求め直す。D-163）

| 観点 | A: 65 の完成形（逆向きの movement を足す。TRACE-D4・D6、§65.6.2） | B: 元の movement を `is_voided=1` にして求め直す（ADR D7 の Z004 の取消と同じ） |
|---|---|---|
| 在庫の求め直し（D-118） | 元と逆の 2 行が和で打ち消す。逆の行は新しい id で元の日付を持つ | 元の行が和と判定から外れる。在庫は「その記録が無かった」値（`stock_ledger.rs:136-152` の `judge` と `read_ledger` は `is_voided=0` だけを読む。`inventory_repo.rs:279-285`） |
| 要再確認（H5） | 出る: 逆の行は fresh な計数の `seen_movement_id` より後の id で、日付が計数の今日と同じなら「決まらない」（`stock_ledger.rs:144-151`）。既定の計数と同じ日付なら元と逆の両方が決まらない | 出ない: 無効の行は判定に入らず、効く計数・精算は変わらない。決まらない movement が減るだけ（理由 2 が解けることはあっても増えない） |
| 棚卸しの完了（D6） | 完了の条件 2（要再確認 0）を H5 が止めうる。評価額は変えない | 条件 2 を止めない。評価額は変えない。記録の詳細の参考の差は `read_ledger` を通るので無効の行を除いて出る（`stocktake_service.rs:748`） |
| 売上（56・57） | 手動販売の売上の行は別に打ち消す（負の行か無効化）。どちらも元の手動販売との結びが要る | 結んだ行を `is_voided=1`。日次・月次は `is_voided=0` だけを読むので変更なし（`sales_repo.rs:1237`・`:1673`・`:1708`）。結ぶ列 `manual_sale_id` は両案で要る（今は `manual_sale.rs:268` が `csv_import_id: None` だけを渡す） |
| 原価合計・ロス原価 | 記録ごとの詳細の値（`list.rs:130`・`:212`）。記録をまたぐ集計は無い（`rg` で `loss_cost`・`total_cost` の読み手を見た）。両案で同じ: 取消済みの記録の詳細は値を出したまま「取消済み」 | 同左 |
| 操作ログ・監査（REQ-208） | header・明細・元と逆の movement が残る。履歴に「取消」の行が見える | header・明細・無効の movement・売上の行が残る（物理削除しない）。商品別の在庫変動の履歴は Z004 の取消と同じく無効の行を出さない（`inventory_repo.rs:420-429`）。取消の日時・理由は header、操作ログ `inventory_record_cancel` |
| 足すもの | `inventory_movements` に 2 列、逆の行の作成と表示の区別、header の列 | header の 2 列、Z004 の取消の関数（`sales_repo.rs:558` の `void_movements_by_reference`）の再利用 |

決めた形: B。理由: H5 が無く棚卸しを止めず、Z004 の取消と規則が 1 つで、求め直し・整合性・売上の集計・完了の後の差を変えずに済む（D-147: 既にある関数と規則）。変える決まった設計: 65 TRACE-D4・D5・D6、§65.0・§65.4.1・§65.6.1・§65.6.2・§65.7.2・§65.8.2・§65.10・§65.11、ADR の横断の表の入出庫の行・D7・Consequences の H5、transaction の §4-5 の完成形の段（計画の commit で直した。下の S1）。失うもの: 商品別の在庫変動の履歴で取消が行として見えない（下の Q4）。

## Goal

Goal Invariant:

### 最小完了条件

- 店の人が、入庫・返品交換・手動販売・廃棄の記録の詳細で「この記録を取り消す」を押し、理由を入れて確定すると、その記録で動いた在庫が元に戻り（その記録が無かった値）、手動販売はその日の日次売上・月次の売上から外れる。
- 取り消した記録は消えず、入出庫履歴・記録の詳細・作業画面の直近の一覧で「取消済み」と分かり、詳細で取消の日時と理由が読める。操作ログに残る。
- 取消は要再確認を新しく作らず、完了した棚卸しの評価額を変えない。
- 打ち間違いは、取り消した後に作成の画面で入れ直して直せる。

### 失敗定義

- 取り消した記録の movement・手動販売の売上の行が在庫・売上・要再確認に効き続ける、または別の記録の movement・売上の行が無効になる。
- 取消で要再確認が新しく出る、完了した棚卸しの `total_cost`・評価の列が変わる。
- 取消済みの記録をもう一度取り消せる、取消の途中の失敗で header だけ・movement だけが変わる。
- 取消で header・明細・movement・売上の行が物理削除される。
- 取り消した記録が「有効」のまま見える（一覧・詳細・直近の一覧）。

### 非目的

- 振替の取消（下の Q1 の答えの前は入れない）。
- 明細ごとの取消、取消の取消（やり直し）、訂正の command（TRACE-D5。取り消してから入れ直す）。作成の画面への取り消した内容の写し（Q2）。
- 在庫変動の履歴に取り消した記録の行を出すこと（Q4）。
- CSV取込み（§65.5a）・棚卸しの取消、届いた日（関門 16）、棚卸しの基準日（関門 17）。
- migration の前に保存した手動販売の売上の行の埋め戻し（本番の DB は無い）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 確かめた事実（2026-11-20、`ee93eb5c`）

- 求め直しの入力は `is_voided=0` の movement だけ: `src-tauri/src/db/inventory_repo.rs:279-285`（`list_ledger_movements_for_product`）を `stock_ledger.rs:314-321` の `read_ledger` が読み、在庫・要再確認・完了の後の差・整合性の全てがここを通る（`stocktake_service.rs:13`・`:748`、`stock_ledger.rs:355-358` の `derive_stock`）。窓の判定は `stock_ledger.rs:136-152`。
- Z004 の取消は `csv_import_service/rollback.rs:20-45` の `apply_rollback`: `sales_repo::void_sale_records_by_import`（`sales_repo.rs:539`）と `void_movements_by_reference`（`:558`。reference で選んで `is_voided=1`、商品コードと数量を返す）と求め直し。`void_movements_by_reference` は `ref_type: &str` を受ける汎用の関数。
- 手動販売の売上の行は手動販売を指さない: `src-tauri/src/biz/inventory_service/manual_sale.rs:265-279` の `insert_sale_record` は `csv_import_id: None`・`source: "manual"` だけ。`sale_records` に手動販売の ID の列は無い（[POS](../db-design/pos-tables.md) §13、`ee93eb5c` の版）。
- 売上の読み手は `is_voided=0` で絞る: 日次 `sales_repo.rs:1237`、月次 `:1673`・`:1708`、商品の最終販売日 `product_repo.rs:1284`。商品の最終入庫日 `product_repo.rs:1278-1281` は取消を見ない（Scope の S4 で直す）。
- 記録の詳細の `status` は 4 つとも固定の `"active"`（`receiving_repo.rs:287`、`return_repo.rs:335`、`manual_sale_repo.rs:220`、`disposal_repo.rs:676`）。関連 movement は `is_voided = 0` で読む（`receiving_repo.rs:261`、`return_repo.rs:308`、`manual_sale_repo.rs:195`、`disposal_repo.rs:653`）。
- 入出庫履歴の一覧の状態は `RecordSpec.status_expr` で種類ごとに出し、4 つの記録は `None`（= `'active'`。`disposal_repo.rs:289-341`・`:414-417`）。絞り込みは既に `canceled` を受ける（`disposal_repo.rs:262`、`list.rs:98-103`）。画面の状態の選択肢と「取消済み」の文字は既にある（`src/features/inventory-records/types.ts:14-19`・`:89-97`）。
- 直近の一覧: 入庫・廃棄・返品交換は `listReceivings`・`listDisposals`・`listReturns`（`ReceivingPage.tsx:151`、`DisposalPage.tsx:156`、`ReturnExchangePage.tsx:226`）、手動販売と振替は `listInventoryRecords`（`ManualSalePage.tsx:169`、`StockTransferPage.tsx:151`）。
- 操作ログの関連記録の link は `record_type` で決まり、4 つの記録と振替の route を持つ（`src/features/operation-logs/OperationLogsPage.tsx:59-64`）。種類の名前は `operation-type-labels.ts`。
- `inventory_movements` の CHECK は `is_voided` に条件を持たない（`schema_v13.rs:22-37`）。trigger は無い（`rg TRIGGER src-tauri/src/db` は test の 1 つだけ）。
- SQLite の `ADD COLUMN` で、別の列を見る CHECK と `REFERENCES` を付けた列が足せ、既存の行・違反の値の拒否が期待どおり（python 3 の sqlite3、SQLite 3.46.1 で試した。2026-11-20。app は rusqlite 0.31 の bundled〈`libsqlite3-sys` 0.28.0、`src-tauri/Cargo.lock:2137-2138`〉で版は未実測。実装の T9 が app の SQLite で確かめる）。
- migration の最新は v13（`src-tauri/src/db/migration.rs:106-110`）。PR #294（Plan Commit 済み）が v14・22 §22・MNT-03-D23 を足す。関門 16 の packet も次の版・§23・MNT-03-D24・D-161 を見込む（D-161 は main で別の決定が使った）。本 lane は v15・§23・MNT-03-D24 の見込みで、先に merge した側に合わせて取り直す（22 §21・§23）。decision-log は D-163（発注で D-162 は backup の docs の lane が使う）。
- 本番の DB は無い（ADR SPEC-STK-CNT-D10、Plans の関門は導入の前）。

## Scope

前提: 下の Q1 が B（振替を入れない）。A を選んだら S1〜S8 に振替を 5 つ目として足す（下の Q1 の「A のときに足すもの」）Gated の書き直しを計画の review の前に出す。

- S1 正本（計画の commit に入れた）: decision-log D-163。ADR [2026-10-16-stocktake-count-placement](../adr/2026-10-16-stocktake-count-placement.md) の横断の表の入出庫の行・SPEC-STK-CNT-D7・Consequences の H5。[65](../function-design/65-inventory-record-traceability.md) の冒頭の D-118 の段・§65.0・TRACE-D4〜D6・§65.4.1・§65.5b（新設）・§65.6.1・§65.6.2・§65.7.2・§65.8.2・§65.10・§65.11・§65.12。[transaction](../db-design/transaction-tables.md) §4-5 の「取消の列」と §11a-11b の「取消の列を持たない理由」。[POS](../db-design/pos-tables.md) §13 の `manual_sale_id`。[22](../function-design/22-mnt-migration.md) §23（MNT-03-D24）。[31](../function-design/31-biz-inventory-service.md) §12.5 の手順 7・§12.6a・§12.6c（新設）。[44](../function-design/44-cmd-inventory.md) §23.7b（新設）。`docs/DB_DESIGN.md` の index の一覧。実装の PR で「proposed・未実装」の注を外す。
- S2 migration v15（`src-tauri/src/db/schema_v15.rs`、`MigrationKind::Sql`、1 TX）: 22 §23 の手順のとおり。`migration.rs` に登録する。22 の §3.1 の schema の一覧に 1 行。
- S3 BIZ（`src-tauri/src/biz/inventory_service/` に `cancel.rs` を足し `mod.rs` で公開）: 31 §12.6c の `cancel_inventory_record` と `RecordCancelResult`。`sales_repo::void_movements_by_reference`・`recompute_stock`・`negative_stock_warning_message`・`system_repo::insert_operation_log` を使い、新しい求め直しを書かない。31 §12.5 の手動販売の保存で `manual_sale_id` を書く（`NewSaleRecord` に field。Z004 の確定・seed・test の呼出しは `rg "NewSaleRecord \{"` で全部直し、None を渡す）。
- S4 IO: 4 つの header の取消の列を書く関数（`receiving_repo`・`return_repo`・`manual_sale_repo`・`disposal_repo` の各 1 つ。または表の名前を BIZ の match から渡す 1 つの関数。実装で小さい方）、手動販売の結んだ売上の行の数と無効化（`sales_repo`）。詳細 4 つの `status`・`canceled_at`・`cancel_reason`、直近の一覧 3 つの `canceled_at`、入出庫履歴の一覧の 4 つの `status_expr`（`CASE WHEN {header_alias}.canceled_at IS NULL THEN 'active' ELSE 'canceled' END`）、商品の最終入庫日から取消済みを外す（`product_repo.rs:1278-1281`）。
- S5 CMD（`src-tauri/src/cmd/inventory_cmd.rs` か新しい `record_cancel_cmd.rs`、44 §23.7b）: `cancel_inventory_record` を足し、`lib.rs` の `collect_commands!`・`generate_handler!` に登録し、`src/lib/bindings.ts` を作り直す。
- S6 UI 記録の詳細（`src/features/inventory-records/` の `ReceivingRecordDetailPage.tsx`・`ReturnRecordDetailPage.tsx`・`ManualSaleRecordDetailPage.tsx`・`DisposalRecordDetailPage.tsx`）: 65 §65.5b のとおり、取消の button・確認の dialog（理由の欄、種類ごとの文）・成功と失敗・取消済みの表示。4 つで同じ部品（dialog と取消済みの Alert と mutation の hook）を 1 つ作って使う。invalidation は D-052 の C25（`src/lib/invalidation-contract.ts` と decision-log D-052 の C 番号の範囲の文、oracle の test）。
- S7 UI 直近の一覧と一覧: 入庫・返品交換・廃棄の画面の直近の一覧の行に「取消済み」（文字）。手動販売の直近の一覧と入出庫履歴は今の状態の表示で「取消済み」が出ることを test で確かめる（変える所があれば直す）。
- S8 操作ログ: `operation-type-labels.ts` に `inventory_record_cancel`「入出庫記録の取消」（区分「入出庫」）、[74](../function-design/74-ui-operation-logs.md) の種類の表。
- 呼出し側・隣接: `RecordSpec` の 4 つの行、`get_*_record` の BIZ（`list.rs`）、詳細の DTO を使う frontend の test の fixture（`rg "status: \"active\"" src --glob '*.test.*'`）を直す。求め直し（`stock_ledger.rs`）・整合性（36）・棚卸し（35）・売上の集計（`sales_repo` の読取り）は変えない。
- REQ-208 の coverage（`docs/spec/requirements.md:27`・`:52`、`requirements-coverage.md`）を実装の PR で `deferred` から外し、traceability を再生成する。
- Non-scope: 振替（Q1）、作成の画面への写し（Q2）、履歴の取消の行（Q4）、届いた日（関門 16。関門 16 の「届いた」は取消済みの入庫を拒む必要がある。後から merge する lane が足す）、基準日（関門 17）。

## Acceptance Criteria

- Test Plan の行の test が全て PASS し、`cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`、`cargo run --bin generate_traceability -- --check`、`bash scripts/doc-consistency-check.sh` が ERROR なしで終わる。
- `cargo run --bin generate_bindings` の後の `src/lib/bindings.ts` の差分が、`cancelInventoryRecord`・`RecordCancelResult`、詳細 4 つの `canceled_at`・`cancel_reason`、直近の一覧 3 つの `canceled_at` だけ。
- 取消の前の作成・一覧・詳細の既存の test が、`status`・新しい field の fixture の追加の他は期待値を変えずに PASS する（差分で示す）。
- `## L3` の項目が owner の PASS。

## Design Readiness

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D2・D3・D6・D7 と横断の表、D-163、[65](../function-design/65-inventory-record-traceability.md) TRACE-D4〜D6・§65.4.1・§65.5・§65.5a・§65.5b・§65.7.2・§65.8.2、[31](../function-design/31-biz-inventory-service.md) §12.5・§12.6a・§12.6c、[44](../function-design/44-cmd-inventory.md) §23.7b、[transaction](../db-design/transaction-tables.md) §4-5、[POS](../db-design/pos-tables.md) §13、[22](../function-design/22-mnt-migration.md) §21・§23、D-052、[74](../function-design/74-ui-operation-logs.md)。
- 必要な設計成果物: updated in this PR（S1。計画の commit に入れた）。Q1 が A なら振替の分（D-143 (5)・79 §79.6・TRACE-D14・transaction §11a-11b）を Gated の書き直しで足す。
- plan にしかない durable な判断の昇格先: 取消の形・列・command の数・訂正を持たないことは D-163 と 65・ADR へ上げた。
- 「取消は要再確認を新しく作らない」の導き: 要再確認の理由は (1) 決まらない精算の行、(2) 決まらない入出庫の movement、(3) 計数の衝突（ADR D3）。取消は計数と精算の一覧を変えず（効く計数は計数だけで決まる。`stock_ledger.rs:216` の `effective_counts`）、movement の集合から行を除くだけなので、(1)(3) は変わらず (2) は増えない。範囲（lo〜hi）も決まらない movement が減る向きにしか変わらない。抜け道は「無効にしていない別の記録の movement を無効にする」経路で、T1・T3 の他の記録の movement が残ることと T8 の mutation で止める。
- 「その記録が無かった値」の導き: 効く計数 C に対し、取り消した movement が「入る」なら C の値に元から含まれず在庫は変わらない（棚に無かった品は数えていない）、「入らない」なら和から引かれ、「決まらない」なら写しから外れ範囲が縮む。いずれも、その記録を保存しなかった DB の求め直しと同じ値（T4）。
- 前提・制約と延期した design gap: (1) 商品別の在庫変動の履歴に取消は行として出ず、後の行の変動後在庫は保存した時点の写しのまま（Z004 の取消と同じ。Q4）。(2) 取消の取消は無い。誤って取り消したら作成の画面で入れ直す（dialog の文で知らせる。入れ直した記録は新しい id と保存の時刻を持つので、同じ日の fresh な計数より後なら「決まらない」で要再確認になりうる。入れ直した事実は計数の後なので正しい側）。(3) vC の前に保存した手動販売は取り消せない（本番の DB は無い。開発・試験 DB だけ）。(4) 振替は Q1。
- 絶対保証の例外と抜け道: 「取消は物理削除しない」は BIZ が UPDATE だけを使うことで成り立ち、T1 の行数の検査で止める。「同じ記録を 2 回取り消せない」は TX の中の `canceled_at` の検査と `WHERE canceled_at IS NULL` の変更行数で成り立つ（T5）。
- 判定: ready（Q1 が B のとき）。Q1 が A なら振替の分の正本を足してから計画の review に出す。Q2・Q4 の答えが既定と違えば Gated の書き直し。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-163 取消の形（movement の無効化と求め直し） | decision-log、ADR D7・横断の表 | S3 | T1〜T4 | L3-1 |
| ADR SPEC-STK-CNT-D2・D3（変えない） | ADR | `stock_ledger.rs` は変えない | T3・T4 | L3-1 |
| ADR SPEC-STK-CNT-D6 完了の評価（変えない） | ADR、35 | 変えない | T6 | — |
| TRACE-D4 物理削除しない・理由必須 | 65 | S3・S4 | T1・T5 | L3-1 |
| TRACE-D5 訂正は取消と入れ直し | 65 | command を足さない | — | L3-1（dialog の文） |
| TRACE-D6 movement の無効化 | 65 §65.6.2 | S3 | T1・T3 | — |
| §65.4.1 状態の正規化（4 つの記録の `canceled`） | 65 | S4 | T7 | L3-1 |
| §65.5b 詳細の取消の UI | 65 | S6 | T10 | L3-1・L3-2 |
| §65.7.2 記録ごとの無効にするもの | 65 | S3 | T1・T2・T3 | L3-2 |
| §65.8.2 履歴に無効の行を出さない（変えない） | 65、66 | 変えない | T1（`list_movements` に出ない） | L3-1 |
| 31 §12.5 手動販売の `manual_sale_id` | 31 | S3 | T2 | — |
| 31 §12.6a 詳細の状態と取消の field | 31 | S4 | T7 | L3-1 |
| 31 §12.6c cancel_inventory_record | 31 | S3 | T1〜T5 | L3-1・L3-2 |
| 44 §23.7b command・DTO | 44 | S5 | T11、bindings の差分 | — |
| transaction §4-5 取消の列・POS §13 `manual_sale_id` | db-design | S2・S4 | T9 | — |
| 22 §23 / MNT-03-D24 migration v15 | 22（§21 の番号の規則） | S2 | T9 と既存の連番の test | — |
| D-052 C25 invalidation | decision-log D-052、UI_TECH_STACK §2.5 | S6 | 既存の invalidation の oracle の test に entry を足す | — |
| 74 操作ログの種類 | 74 | S8 | T12 | — |
| 56・57 売上（変えない） | 56・57 | 変えない（`is_voided=0` を読む） | T2 | L3-2 |
| 36 整合性（変えない） | 36 | 変えない | T3 で整合性の検査が差を出さない | 非対象 |

隣の契約の sweep: 振替（79・D-143）は Q1。CSV取込みの取消（§65.5a・32）は変えない（`void_movements_by_reference` は読むだけで共有する）。商品の最終入庫日（20）は S4。関門 16 の「届いた」と取消の組合せは後から merge する lane が足す（未着で取り消した入庫に「届いた」を出さない）。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| T1 入庫の取消（状態の遷移: 保存 → 取消） | movement が残る・別の記録の movement も消える・行が消える | Rust unit（BIZ） | `cancel_receiving_voids_own_movements_and_recomputes` | 取消の後に在庫が保存の前の値でない、その記録の movement が `is_voided=0`、同じ商品の別の入庫の movement が `is_voided=1`、header・明細・movement の行数が減る、`canceled_at`・`cancel_reason` が無い、操作ログ `inventory_record_cancel` が無い、`list_movements` に取り消した行が出る |
| T2 手動販売の取消と売上 | 売上が日次・月次に残る・別の手動販売の売上も消える | Rust unit（BIZ と sales の読取り） | `cancel_manual_sale_voids_linked_sale_records` | 取消の後の日次売上・月次の売上にその行が残る、同じ日・同じ品・同じ額の別の手動販売の行が消える、保存で `manual_sale_id` が書かれない |
| T3 返品・交換と廃棄 | movement の無い明細の扱い・交換の 2 方向 | Rust unit | `cancel_return_and_disposal_void_only_saved_movements` | `register_processed` で movement の無い返品の取消が失敗する・在庫を動かす、交換の戻りと渡しの片方だけ戻る、廃棄の取消で在庫が戻らない、取消の後に整合性の検査（36）が差を出す |
| T4 要再確認を作らない（H5 の回帰） | 取消で要再確認が出る・在庫が「記録が無かった値」でない | Rust integration（BIZ と `stock_ledger`） | `cancel_never_adds_recheck_and_matches_record_absent` | (a) 入庫の後に同じ日の fresh な計数 → 取消で要再確認になる・在庫が計数の値でない、(b) 既定の計数と同じ日付の入庫（要再確認）→ 取消で要再確認が解けない、(c) 計数の後の日付の入庫 → 取消で在庫がその数だけ戻らない。各場合で取消の後の写しが、その記録を保存しなかった DB の求め直しと同じでない |
| T5 negative（再取消・無い・空の理由・種類の外・結びの無い手動販売・失敗で DB 不変） | 2 回目で何かが変わる・拒否の後に DB が変わる | Rust unit | `cancel_inventory_record_rejects_and_leaves_db_unchanged` | 2 回目の取消が通る・`canceled_at` が変わる、無い ID が NotFound でない、空白だけの理由が通る、`csv_import`・`stocktake`・`transfer_record` が通る、`manual_sale_id` の無い手動販売の取消で movement が無効になる、どの拒否の後も header・movement・売上の行が前と同じでない |
| T6 完了した棚卸し | 評価額が変わる・参考の差に出ない | Rust integration | `cancel_after_stocktake_completion_keeps_valuation` | 完了の後に取り消すと `total_cost`・`valuation_quantity`・`valuation_cost_price` が変わる、記録の詳細の参考の差に取り消した数が出ない |
| T7 一覧と詳細の状態 | 取消済みが「有効」に混ざる | Rust unit（IO） | `inventory_records_status_canceled_for_four_record_types` | 取り消した 4 つの記録の一覧の status が `active`、`status=canceled` で出ない・`active` で出る、詳細の `status`・`canceled_at`・`cancel_reason` が違う、直近の一覧の `canceled_at` が無い、商品の最終入庫日が取り消した入庫の日付 |
| T8 mutation の注入 | 構造の推論だけで済ませる | 手で注入（記録は PR の comment） | T1・T2・T4 | 無効化を外す・求め直しを外す・`manual_sale_id` で絞らない注入で、T1・T2・T4 が red にならない |
| T9 migration v15 | 列・CHECK・FK・index が無い、既存の行が取消済みになる | Rust unit（migration） | `migration_v15_adds_cancel_columns_and_manual_sale_link` | 列か index が無い、v14 の DB の既存の行の新しい列が NULL でない、`canceled_at` だけ・空白の理由・Z004 の行への `manual_sale_id`・無い手動販売の ID が通る |
| T10 記録の詳細の取消 | 取消済みに button が出る・理由なしで送る・失敗で再試行できない・再取得しない | vitest | `RecordCancel.test.tsx`（4 つの詳細の describe） | 有効でない記録に button が出る、空の理由で送れる、手動販売で売上の文が出ない、成功で C25 の invalidate が呼ばれない・取消済みの Alert に理由が出ない、失敗の後に dialog と理由が残らない、取消済みの badge が色だけ |
| T11 command の wire | DTO の field と種類の受け方が違う | Rust unit（CMD） | `cancel_inventory_record_cmd_maps_errors` | 種類の外が `validation` でない、無い ID が `not_found` でない |
| T12 直近の一覧と操作ログ | 取消済みが直近の一覧で有効に見える・種類が「その他」 | vitest | `ReceivingPage.test.tsx`・`DisposalPage.test.tsx`・`ReturnExchangePage.test.tsx` の直近の一覧の describe、`OperationLogsPage.test.tsx` の種類の表 | 取り消した行に「取消済み」が出ない、`inventory_record_cancel` が「その他」で出る |

## L3

- 入庫の記録の詳細 / 開発用の Windows の native build で、合成の商品 1 つに入庫 +3 を保存し、入出庫履歴 → その行 → 「この記録を取り消す」→ 理由「打ち間違い」→ 確定 / 詳細が「取消済み」と取消の日時・理由を出し、商品と在庫の画面でその商品の在庫が入庫の前の値、在庫変動の履歴にその入庫の行が無く、入出庫履歴の状態「取消済み」でその記録が出て、入庫の画面の直近の一覧の行に「取消済み」が出る（L3-1）。
- 手動販売の記録の詳細 / 合成の商品を今日の日付で手動販売（金額 500 円）→ 日次売上で今日の手動販売の行と合計を見る → 記録の詳細で取り消す / 日次売上の今日の手動販売の行が消え合計が 500 円減り、在庫が販売の前の値に戻る（L3-2）。
- fixture: 合成の商品 1 つ（開発の DB。店の実データを使わない）。Eligibility: native の画面の操作と表示の確かめで、新しい tool・故障注入は要らない。

## owner に諮ること（未決）

- Q1 振替（詰め替え・切り分け）を同じ取消に入れるか（TD-833 で未決）:
  - A: 入れる。振替の記録の詳細にも「この記録を取り消す」を置き、`transfer_records` に同じ 2 列を足す。今の「向きを入れ替えた振替で戻す」案内（[79](../function-design/79-ui-stock-transfer.md) §79.6）と D-143 (5) を改める。足すもの: migration の 1 表、BIZ の match の 1 行、`RecordSpec` の 1 行、振替の詳細の画面、test 1 行。
  - B: 入れない。今の戻し方のまま（戻す振替は同じ日付の別の記録で、同じ日の fresh な計数があると要再確認になる。2 つの記録が残る）。
  - 推奨: A（candidate。出所: 本起票の AI の調べ）。理由: 足す手間は小さく（上の 5 つ）、今の戻し方は打ち間違いを直すたびに記録が 2 つ残り、同じ日の計数で要再確認を出す（取消の形の比べの A と同じ害）。
- Q2 打ち間違いの直し方（TRACE-D5 の訂正）:
  - (a) 取り消してから作成の画面で入れ直す（本 packet の Scope。取消の成功の後に作成の画面への link を出すかは実装で小さく足せる）。
  - (b) (a) に加え、取消の後に作成の画面を取り消した内容で埋めて開く（明細の多い入庫で入れ直しの手間を減らす。画面ごとに作業が増える）。
  - (c) 訂正の command（取消と入れ直しを 1 つの操作）。
  - 推奨: (a)（candidate）。(b) は DB と command を変えないので導入の後に足せる。明細の多い記録の打ち間違いの頻度は未実測。
- Q3 Risk: TD-833 は R3〜R4 見込み。Risk Tiers で R3 と判断した（上の Reason）。owner が R4 を選べば `## Recovery`（v15 の前の backup に戻す、22 §16 の「回復」）と `## Data Safety` を足す。推奨: R3（confirmed は owner）。
- Q4 在庫変動の履歴（商品ごと）に取り消した記録の行を出すか（65 TRACE-D6 の「通常の業務取消は履歴を見せる」を変える。出所: 本起票の AI の調べ）:
  - (a) 出さない（Z004 の取消と同じ。記録の詳細・入出庫履歴の「取消済み」と操作ログで追う）。
  - (b) 「取消」の印を付けて出す（[44](../function-design/44-cmd-inventory.md) の `list_movements` の結果に field 1 つ、[66](../function-design/66-ui-stock-movements.md) の表示。Z004 の取消の行も同じく出る）。
  - 推奨: (a)（candidate）。理由: 在庫の数は「その記録が無かった」値で、履歴もそれに合う。(b) は後から足せる（DB を変えない）。

## 計画の review に問う点

- 取消の形 B の「要再確認を作らない」「記録が無かった値」の導き（Design Readiness）に反例があるか。特に fresh な計数の `seen_movement_id` の規則と、既定の計数・精算の窓の組合せ。
- 完了した棚卸しの参考の差（ADR D6 の比較の範囲）に、取り消した movement が期待どおりに出るか（`stocktake_service.rs:748` の読み方）。
- 手動販売の結びの無い記録を拒む形（31 §12.6c の手順 3）が、本番の DB が無い前提で足りるか。
- 冪等キーを持たない取消（再取消は拒否）で、応答を失ったときの画面の振舞い（65 §65.5b の失敗）が足りるか。
- 1 つの command にした形（`ReferenceType` を受ける）が、44 §23.3 の error の写像と bindings で問題ないか。
- この変更で不要になる記述・条件は無いか（65 の `corrected` の残り、TRACE-D14 の取消の文、`types.ts:95` の「訂正済み」）。
