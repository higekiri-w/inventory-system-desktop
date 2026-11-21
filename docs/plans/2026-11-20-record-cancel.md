# Plan Packet: 打ち間違えた入出庫の記録を取り消す（導入の前の関門 18）

導入の前の関門 16〜18（[Plans](../Plans.md#導入の前の関門)）の 2 本目の lane（順は 16 → 18 → 17。関門 16 の packet〈branch `agent/gates-16-18-design`、`99964d72`〉の「分け方」）。出所は [backlog](../backlog.md) の「入出庫の記録の取消の runtime」、関門 16 の起票の申し送り（同じ branch の backlog の同じ項目）、owner の決定（repo 外の回答台帳 TD-833〈2026-11-19、取消を導入の前に。振替は起票で決める〉・TD-846〈2026-11-20、関門 16 は店の答え待ちの間に 18 を先に起票〉・TD-848〈2026-11-20、起票の Q1〜Q4 の答え。下の「owner の決定」〉）。起点は `ee93eb5c`（origin/main、2026-11-20）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason: `docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行の「DB」（5 つの header に 2 列、`sale_records` に 1 列と index の migration）、「Tauri command の DTO」（新しい command 1 つ、詳細 5 つと直近の一覧 3 つの DTO に field）、「状態を持つ画面・operator の業務の流れ」（記録の詳細の取消の操作と取消済みの表示）に当たる。R4 の行には当たらないと判断した: 取消は行を消さない論理の無効化（`is_voided`。Z004 の取消と同じ形）で、header・明細・movement・売上の行は残り、migration は `ADD COLUMN` と index だけで表を作り直さず既存の値を変えない。本番の DB は無い（[project-memory](../project-memory.md)、ADR SPEC-STK-CNT-D10）。owner は TD-833 で「R3〜R4 見込み」とし、TD-848 の Q3 で R3 に異議なしとした。workflow の file（`.github/`・`scripts/`）に触れない。

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

決めた形: B。理由: H5 が無く棚卸しを止めず、Z004 の取消と規則が 1 つで、求め直し・整合性・売上の集計・完了の後の差を変えずに済む（D-147: 既にある関数と規則）。振替も同じ形で、向きを入れ替えた振替で戻す今の案内（同じ日の計数で要再確認を出す。A と同じ害）を置き換える。変える決まった設計: 65 TRACE-D4・D5・D6・D14、§65.0・§65.4.1・§65.6.1・§65.6.2・§65.7.2・§65.8.2・§65.10・§65.11、ADR の横断の表の入出庫の行・D7・Consequences の H5、transaction の §4-5 の完成形の段と §11a-11b、D-143 (5)、79 §79.6・§79.8（計画の commit で直した。下の S1）。失うもの: 商品別の在庫変動の履歴で取消が行として見えない（TD-848 の Q4 で受けた）。

## Goal

Goal Invariant:

### 最小完了条件

- 店の人が、入庫・返品交換・手動販売・廃棄・詰め替え・切り分け（振替）の記録の詳細で「この記録を取り消す」を押し、理由を入れて確定すると、その記録で動いた在庫が元に戻り（その記録が無かった値。振替は減らした商品と増やした商品の全部）、手動販売はその日の日次売上・月次の売上から外れる。
- 取り消した記録は消えず、入出庫履歴・記録の詳細・作業画面の直近の一覧で「取消済み」と分かり、詳細で取消の日時と理由が読める。操作ログに残る。
- 取消は要再確認を新しく作らず、完了した棚卸しの評価額を変えない。
- 打ち間違いは、取り消した後に作成の画面で入れ直して直せる。

### 失敗定義

- 取り消した記録の movement・手動販売の売上の行が在庫・売上・要再確認に効き続ける、または別の記録の movement・売上の行が無効になる。振替の取消で一部の明細の商品だけが戻る。
- 取消で要再確認が新しく出る、完了した棚卸しの `total_cost`・評価の列が変わる。
- 取消済みの記録をもう一度取り消せる、取消の途中の失敗で header だけ・movement だけが変わる。
- 取消で header・明細・movement・売上の行が物理削除される。
- 取り消した記録が「有効」のまま見える（一覧・詳細・直近の一覧）。

### 非目的

- 明細ごとの取消、取消の取消（やり直し）、訂正の command（TRACE-D5。取り消してから入れ直す）。作成の画面への取り消した内容の写し（TD-848 の Q2 で採らない）。
- 在庫変動の履歴に取り消した記録の行を出すこと（TD-848 の Q4 で採らない）。
- CSV取込み（§65.5a）・棚卸しの取消、届いた日（関門 16）、棚卸しの基準日（関門 17）。
- migration の前に保存した手動販売の売上の行の埋め戻し（本番の DB は無い）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 確かめた事実（2026-11-20、`ee93eb5c`。振替の項は 2026-11-21、`113e3689`）

- 求め直しの入力は `is_voided=0` の movement だけ: `src-tauri/src/db/inventory_repo.rs:279-285`（`list_ledger_movements_for_product`）を `stock_ledger.rs:314-321` の `read_ledger` が読み、在庫・要再確認・完了の後の差・整合性の全てがここを通る（`stocktake_service.rs:13`・`:748`、`stock_ledger.rs:355-358` の `derive_stock`）。窓の判定は `stock_ledger.rs:136-152`。
- Z004 の取消は `csv_import_service/rollback.rs:20-45` の `apply_rollback`: `sales_repo::void_sale_records_by_import`（`sales_repo.rs:539`）と `void_movements_by_reference`（`:558-587`。reference で `is_voided=0` の movement を全部選んで商品コードと数量を返し、`is_voided=1` にする）と求め直し。`void_movements_by_reference` は `ref_type: &str` を受ける汎用の関数。
- 手動販売の売上の行は手動販売を指さない: `src-tauri/src/biz/inventory_service/manual_sale.rs:265-279` の `insert_sale_record` は `csv_import_id: None`・`source: "manual"` だけ。`sale_records` に手動販売の ID の列は無い（[POS](../db-design/pos-tables.md) §13、`ee93eb5c` の版）。
- 売上の読み手は `is_voided=0` で絞る: 日次 `sales_repo.rs:1237`、月次 `:1673`・`:1708`、商品の最終販売日 `product_repo.rs:1284`。商品の最終入庫日 `product_repo.rs:1278-1281` は取消を見ない（Scope の S4 で直す）。
- 記録の詳細の `status` は 5 つとも固定の `"active"`（`receiving_repo.rs:287`、`return_repo.rs:335`、`manual_sale_repo.rs:220`、`disposal_repo.rs:676`、`transfer_repo.rs:224`）。関連 movement は `is_voided = 0` で読む（`receiving_repo.rs:261`、`return_repo.rs:308`、`manual_sale_repo.rs:195`、`disposal_repo.rs:653`、`transfer_repo.rs:194-203`）。
- 入出庫履歴の一覧の状態は `RecordSpec.status_expr` で種類ごとに出し、5 つの記録は `None`（= `'active'`。`disposal_repo.rs:289-341`・`:374-385`・`:414-417`）。絞り込みは既に `canceled` を受ける（`disposal_repo.rs:262`、`list.rs:98-103`）。画面の状態の選択肢と「取消済み」の文字は既にある（`src/features/inventory-records/types.ts:14-19`・`:89-97`）。
- 直近の一覧: 入庫・廃棄・返品交換は `listReceivings`・`listDisposals`・`listReturns`（`ReceivingPage.tsx:151`、`DisposalPage.tsx:156`、`ReturnExchangePage.tsx:226`）、手動販売と振替は `listInventoryRecords`（`ManualSalePage.tsx:169`、`StockTransferPage.tsx:144-158`）。手動販売の行は状態の badge を出す（`ManualSalePage.tsx:776`）が、振替の表は日付・記録ID・代表商品・記録日時・操作だけで状態を出さない（`StockTransferPage.tsx:700-733`）。
- 振替の記録の作成（`src-tauri/src/biz/inventory_service/transfer.rs`）: 1 つの記録は減らす明細と増やす明細を 1 つ以上ずつ持ち（`:118`）、同じ商品は 1 回だけ（`:149`）。明細ごとに `apply_stock_change(…, MovementType::Transfer, ReferenceType::TransferRecord, record_id, None)` を呼び（`:192-215`）、1 回の呼出しが movement を 1 つ INSERT する（`common.rs:109`）。よって明細が N（≥ 2）の振替は、`reference_type='transfer_record'` の movement を N 持ち、商品は N で全部違う（[transaction](../db-design/transaction-tables.md) §11a-11b の業務シナリオ例は元 1・先 1 の 2 つ）。`void_movements_by_reference(tx, "transfer_record", record_id)` はその N を全部返すので、無効にして求め直す商品は N（減らす商品と増やす商品の全部）。
- 振替の詳細の画面は取消の代わりに「取消はできません」の Alert と、向きを入れ替えた振替で戻す案内 `TRANSFER_CORRECTION_GUIDE` を出す（`src/features/inventory-records/TransferRecordDetailPage.tsx:45-47`・`:171-174`。期待は `TransferRecordDetailPage.test.tsx:118-121`）。設計は [79](../function-design/79-ui-stock-transfer.md) §79.6・§79.8、D-143 (5)、65 TRACE-D14。`transfer_records` は 5 列（`schema_v13.rs:52-58`）。
- `ReferenceType`（`src-tauri/src/db/inventory_repo.rs:58`）は `Serialize` と `specta::Type` だけで `Deserialize` を持たない（command の入力にできない）。作業画面の直近の一覧は `receivings`・`returns`・`disposals` の key（`src/lib/query-keys.ts:106-121`、`staleTime` 30 秒）で、`inventoryRecords.root()` の下に無い。取消の待ちの止めは `useBlocker` と `useWindowCloseGuard`（種類の union は `src/lib/window-close-guard.ts:12-17`）。
- 返品・交換の作成の画面は、成功の応答で冪等キーを新しくし（`src/features/return-exchange/ReturnExchangePage.tsx:268`）、結果を出している間は保存を押せない（`:283-285` の `isFormLocked`）。「続けて返品・交換」とリセットも新しいキーにする（`:388-404`）。取消は別の route の詳細で行い、作業画面を離れると state が消えて次の mount で新しいキーになる（`:219`）ので、取り消した返品の作成を同じキーで再送する経路は無い（計画の review の #10 を見送った根拠）。
- 操作ログの関連記録の link は `record_type` で決まり、4 つの記録と振替の route を持つ（`src/features/operation-logs/OperationLogsPage.tsx:59-64`）。種類の名前は `operation-type-labels.ts`。
- `inventory_movements` の CHECK は `is_voided` に条件を持たない（`schema_v13.rs:22-37`）。trigger は無い（`rg TRIGGER src-tauri/src/db` は test の 1 つだけ）。
- SQLite の `ADD COLUMN` で、別の列を見る CHECK と `REFERENCES` を付けた列が足せ、既存の行・違反の値の拒否が期待どおり（python 3 の sqlite3、SQLite 3.46.1 で試した。2026-11-20。app の bundled〈`libsqlite3-sys` 0.28.0、`src-tauri/Cargo.lock:2137-2138`〉の SQLite 3.45.0 でも、計画の review〈Codex、2026-11-21〉が隔離の build で同じ DDL を実測した。実装の T9 が app の SQLite で確かめる）。SQLite の `trim` は半角空白だけを除く（全角空白 1 字の理由は CHECK を通る。同じ review の実測）。
- migration の最新は v13（`src-tauri/src/db/migration.rs:106-110`）。PR #294（Plan Commit 済み）が v14・22 §22・MNT-03-D23 を足す。関門 16 の packet も次の版・§23・MNT-03-D24・D-161 を見込む（D-161 は main で別の決定が使った）。本 lane は v15・§23・MNT-03-D24 の見込みで、先に merge した側に合わせて取り直す（22 §21・§23）。decision-log は D-163（発注で D-162 は backup の docs の lane が使う）。
- 本番の DB は無い（ADR SPEC-STK-CNT-D10、Plans の関門は導入の前）。

## Scope

取り消せる記録は入庫・返品交換・手動販売・廃棄・振替の 5 つ（TD-848 の Q1）。

- S1 正本（計画の commit に入れた）: decision-log D-163 と、D-143 の `Superseded in part by: D-163` の行（D-143 の本文は書き換えない）。ADR [2026-10-16-stocktake-count-placement](../adr/2026-10-16-stocktake-count-placement.md) の横断の表の入出庫の行・SPEC-STK-CNT-D7・Consequences の H5。[65](../function-design/65-inventory-record-traceability.md) の冒頭の D-118 の段・§65.0・TRACE-D4〜D6・TRACE-D14・§65.4.1・§65.5b（新設）・§65.6.1・§65.6.2・§65.7.2・§65.8.2・§65.10・§65.11・§65.12。[transaction](../db-design/transaction-tables.md) §4-5 の「取消の列」と §11a-11b の取消の 2 列。[POS](../db-design/pos-tables.md) §13 の `manual_sale_id`。[22](../function-design/22-mnt-migration.md) §23（MNT-03-D24）。[21](../function-design/21-io-inventory-repo.md) §10.5a の `status`。[31](../function-design/31-biz-inventory-service.md) §12.5 の手順 7・§12.6a・§12.6c（新設）。[44](../function-design/44-cmd-inventory.md) §23.7b（新設）。[79](../function-design/79-ui-stock-transfer.md) §79.5・§79.6・§79.8〜§79.10。`docs/DB_DESIGN.md` の D-6 と index の一覧、[tracking](../db-design/tracking-system-tables.md) の `business_date` の行と `is_voided` の理由（逆向きの movement・「`is_voided` は CSV取込みの取消だけ」を D-163 に置き換えた）、`docs/SCREEN_DESIGN.md` の入出庫履歴の利用者配慮（「訂正済み」を外した）、`docs/UI_TECH_STACK.md` UI-USW-D5 の種類 `inventory-record-cancel` と close の確認の文。実装の PR で「proposed・未実装」の注を外す。
- S2 migration v15（`src-tauri/src/db/schema_v15.rs`、`MigrationKind::Sql`、1 TX）: 22 §23 の手順のとおり（5 つの header）。`migration.rs` に登録する。22 の §3.1 の schema の一覧に 1 行。
- S3 BIZ（`src-tauri/src/biz/inventory_service/` に `cancel.rs` を足し `mod.rs` で公開）: 31 §12.6c の `cancel_inventory_record` と `RecordCancelResult`。取り消せる 5 つの `ReferenceType` は 1 か所の match。`sales_repo::void_movements_by_reference`・`recompute_stock`・`negative_stock_warning_message`・`system_repo::insert_operation_log` を使い、新しい求め直しを書かない。`recompute_stock` の上限の error は TX 全体を戻す（31 §12.6c の手順 5）。理由の検査は `str::trim`（全角空白・tab を含む）。31 §12.5 の手動販売の保存で `manual_sale_id` を書く（`NewSaleRecord` に field。Z004 の確定・seed・test の呼出しは `rg "NewSaleRecord \{"` で全部直し、None を渡す）。
- S4 IO: 5 つの header の取消の列を書く関数（`receiving_repo`・`return_repo`・`manual_sale_repo`・`disposal_repo`・`transfer_repo` の各 1 つ。または表の名前を BIZ の match から渡す 1 つの関数。実装で小さい方）、手動販売の結んだ売上の行の数と無効化（`sales_repo`）。詳細 5 つの `status`・`canceled_at`・`cancel_reason`、直近の一覧 3 つの `canceled_at`、入出庫履歴の一覧の 5 つの `status_expr`（`CASE WHEN {header_alias}.canceled_at IS NULL THEN 'active' ELSE 'canceled' END`）、商品の最終入庫日から取消済みを外す（`product_repo.rs:1278-1281`）。`transfer_repo.rs:80` の `TransferRecordDetail.status` の doc comment「今は常に `"active"`（取消は持たない）」を取消の状態の説明に直す（bindings の comment も変わる）。
- S5 CMD（`src-tauri/src/cmd/inventory_cmd.rs` か新しい `record_cancel_cmd.rs`、44 §23.7b）: `cancel_inventory_record` を `#[tauri::command(async)]` で足し（44 §23.7b の「実行の thread」、UI-USW-D5 の 8。`crate::cmd::assert_commands_run_off_main_thread` で固定）、`lib.rs` の `collect_commands!`・`generate_handler!` に登録し、`src/lib/bindings.ts` を作り直す。`ReferenceType`（`inventory_repo.rs:58`）に `serde::Deserialize` を足す（44 §23.7b の「受け方」）。
- S6 UI 記録の詳細（`src/features/inventory-records/` の `ReceivingRecordDetailPage.tsx`・`ReturnRecordDetailPage.tsx`・`ManualSaleRecordDetailPage.tsx`・`DisposalRecordDetailPage.tsx`・`TransferRecordDetailPage.tsx`）: 65 §65.5b のとおり、取消の button・確認の dialog（理由の欄、種類ごとの文）・成功と失敗・取消済みの表示。5 つで同じ部品（dialog と取消済みの Alert と mutation の hook）を 1 つ作って使う。dialog の入れ直しの文に記録の業務日付を出す。hook は送信中に確定を押せなくし、`useBlocker` と `useWindowCloseGuard("inventory-record-cancel", …)` を持つ（`window-close-guard.ts` の union と RootLayout の確認の窓の文に種類を足す。hook は UI-USW-D1 の `useBlocker` の直接使用の許可対象〈65 §65.5b〉なので、`src/hooks/unsaved-changes-guard-sweep.test.ts` の T16 の期待の集合にその 1 file を足す。許可対象の正本の変更が理由で、既存 test の弱めではない。T18 の全件の検査は変えない）。失敗・結果不明の後の振舞い（同じ詳細の中で dialog と理由を保つ取り直し、`canceled` の負の在庫の知らせ、取り直しの失敗の後の「状態をもう一度確かめる」）は 65 §65.5b。振替の詳細の「取消はできません」の Alert と `TRANSFER_CORRECTION_GUIDE` を外す（79 §79.8。`TransferRecordDetailPage.test.tsx:118-121` の期待は 79 の変更に合わせて取消の test に置き換える）。invalidation は D-052 の C25（`src/lib/invalidation-contract.ts` と decision-log D-052 の C 番号の範囲の文、oracle の test。振替の詳細の query key は `inventoryRecords.root()` の下。作業画面の直近の一覧の `receivings.root()`・`returns.root()`・`disposals.root()` は別の key なので、種類を問わず 3 つとも集合に入れる）。`src/features/inventory-records/types.ts:94-95` の `canceled`（選択肢の表と同じ文字を返す重複）と `corrected`（訂正を持たない）の分岐を消し、`docs/backlog.md` の記録状態 Badge の tone の項の「訂正済み」を外す。
- S7 UI 直近の一覧と一覧: 入庫・返品交換・廃棄の画面の直近の一覧の行に「取消済み」（文字）。振替の画面の直近の一覧の行にも一覧の `status` から「取消済み」を出す（79 §79.5。`StockTransferPage.tsx:700-733` に状態が無い）。手動販売の直近の一覧と入出庫履歴は今の状態の表示で「取消済み」が出ることを test で確かめる（変える所があれば直す）。
- S8 操作ログ: `operation-type-labels.ts` に `inventory_record_cancel`「入出庫記録の取消」（区分「入出庫」）、[74](../function-design/74-ui-operation-logs.md) の種類の表。
- 呼出し側・隣接: `RecordSpec` の 5 つの行、`get_*_record` の BIZ（`list.rs`、`transfer.rs` の `get_transfer_record`）、詳細の DTO を使う frontend の test の fixture（`rg "status: \"active\"" src --glob '*.test.*'`）を直す。求め直し（`stock_ledger.rs`）・整合性（36）・棚卸し（35）・売上の集計（`sales_repo` の読取り）・振替の作成（`create_transfer`）は変えない。
- REQ-208（`docs/spec/requirements.md:27`・`:52`）を実装の PR で `deferred` から外し、task の列に振替の詳細 `UI-16b` を足し、traceability を再生成する（`90-traceability.md` は生成物なので、計画の commit では変えない。`requirements-coverage.md` は原典の ID だけの表で、Design Phase 補足の REQ-208 の行は持たない〈REQ-206・207・209・210 と同じ〉）。取消の主な Rust の test（T1〜T5・T13）の名を `test_cancel_…_req208` にし（生成器は `fn test_` で始まる名だけを拾う）、90 の REQ-208 の行に載せる。
- Non-scope: 作成の画面への写し（TD-848 の Q2 の (b)）・訂正の command（Q2 の (c)）、履歴の取消の行（Q4 の (b)）、届いた日（関門 16。関門 16 の「届いた」は取消済みの入庫を拒む必要がある。後から merge する lane が足す）、基準日（関門 17）。

## Acceptance Criteria

- Test Plan の行の test が全て PASS し、`cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`、`cargo run --bin generate_traceability -- --check`、`bash scripts/doc-consistency-check.sh` が ERROR なしで終わる。
- `cargo run --bin generate_bindings` の後の `src/lib/bindings.ts` の差分が、`cancelInventoryRecord`・`RecordCancelResult`、詳細 5 つ（振替の `TransferRecordDetail` を含む）の `canceled_at`・`cancel_reason`、直近の一覧 3 つの `canceled_at`、`TransferRecordDetail.status` の doc comment（S4）だけ。
- 生成した `docs/function-design/90-traceability.md` の REQ-208 の行が `required` で、T1〜T5・T13 の test を載せる。
- 振替の取消で、その記録の全部の明細の商品（減らす商品と増やす商品）の在庫がその記録を保存しなかった値に戻り、その記録の movement が全部無効になり、同じ商品の別の記録の movement は残る（T13 の `test_cancel_transfer_voids_every_line_and_recomputes_all_products_req208`）。
- 取消の前の作成・一覧・詳細の既存の test が、`status`・新しい field の fixture の追加の他は期待値を変えずに PASS する（差分で示す）。例外は振替の詳細の向きを入れ替える案内の期待（`TransferRecordDetailPage.test.tsx:118-121`）で、79 §79.8 の変更を理由に取消の test へ置き換える（`/取消|訂正/` は新しい「取り消す」に当たらないので、訂正の button が無い検査と取消の button がある検査に分ける）。`src/hooks/unsaved-changes-guard-sweep.test.ts` の T16 の期待の集合に取消の hook の 1 file を足すのは、UI-USW-D1 の許可対象の変更（65 §65.5b）が理由で、既存 test の弱めではない（T18 は変えない）。`src-tauri/tests/design_compliance_test.rs` の doc → module の表に新しい module（`cancel.rs`・`schema_v15.rs`・CMD を新しい file にしたらその file）を足すのは期待値の変更に数えない。
- `## L3` の項目が owner の PASS。

## Design Readiness

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D2・D3・D6・D7 と横断の表、D-163、D-143 (2)・(5)、[65](../function-design/65-inventory-record-traceability.md) TRACE-D4〜D6・TRACE-D14・§65.4.1・§65.5・§65.5a・§65.5b・§65.7.2・§65.8.2、[31](../function-design/31-biz-inventory-service.md) §12.5・§12.6a・§12.6b・§12.6c、[44](../function-design/44-cmd-inventory.md) §23.7b、[transaction](../db-design/transaction-tables.md) §4-5・§11a-11b、[POS](../db-design/pos-tables.md) §13、[22](../function-design/22-mnt-migration.md) §21・§23、[79](../function-design/79-ui-stock-transfer.md) §79.5・§79.6・§79.8、D-052、[74](../function-design/74-ui-operation-logs.md)、[UI_TECH_STACK](../UI_TECH_STACK.md) UI-USW-D5。
- 必要な設計成果物: updated in this PR（S1。振替の分〈D-143 の Superseded in part の行・79・TRACE-D14・transaction §11a-11b〉を含め、計画の commit に入れた）。
- plan にしかない durable な判断の昇格先: 取消の形・列・command の数・訂正を持たないこと・振替を入れることは D-163 と 65・ADR・79 へ上げた。
- 「取消は要再確認の理由を新しく作らない」の導き: 要再確認の理由は (1) 決まらない精算の行、(2) 決まらない入出庫の movement、(3) 計数の衝突（ADR D3）。取消は計数と精算の一覧を変えず（効く計数は計数だけで決まる。`stock_ledger.rs:216` の `effective_counts`）、movement の集合から行を除くだけなので、(1)(3) の有無は変わらず (2) は増えない。保証は「取消の後の写し（在庫・範囲・理由）が、同じ計数・精算から対象の記録の movement だけを除いて求め直した結果に一致する」で、範囲が狭くなることは保証しない（反例: 範囲 10〜15 → 10〜20。理由は前後とも `ConflictingCounts`。計画の review〈Codex、2026-11-21〉の実測で、T4 の (d)）。振替は商品ごとに 1 つの movement を除くだけで、商品ごとに同じ導きが当たる。抜け道は「無効にしていない別の記録の movement を無効にする」経路で、T1・T3・T13 の他の記録の movement が残ることと T8 の mutation で止める。
- 「その記録が無かった値」の導き: 効く計数 C に対し、取り消した movement が「入る」なら C の値に元から含まれず在庫は変わらない（棚に無かった品は数えていない）、「入らない」なら和から引かれ、「決まらない」なら範囲の計算から外れる。いずれも、その記録を保存しなかった DB の求め直しと同じ値（T4）。振替は減らす商品と増やす商品のそれぞれで同じ（T13）。
- 前提・制約と延期した design gap: (1) 商品別の在庫変動の履歴に取消は行として出ず、後の行の変動後在庫は保存した時点の写しのまま（Z004 の取消と同じ。TD-848 の Q4）。(2) 取消の取消は無い。誤って取り消したら作成の画面で入れ直す（日付の案内とその理由は 65 §65.5b・79 §79.6、数は T4 の (e)。TD-848 の Q2）。入れ直した記録は新しい id と保存の時刻を持つので、同じ日の fresh な計数より後なら「決まらない」で要再確認になりうる（入れ直した事実は計数の後なので正しい側）。(3) vC の前に保存した手動販売は取り消せない（本番の DB は無い。開発・試験 DB だけ）。(4) 振替で増やした商品を売った後に振替を取り消すと、その商品の在庫が負になりうる。止めずに `stock_warnings` で知らせる（作成と同じ INV-3、31 §12.6c の手順 6）。
- 絶対保証の例外と抜け道: 「取消は物理削除しない」は BIZ が UPDATE だけを使うことで成り立ち、T1・T13 の行数の検査で止める。「同じ記録を 2 回取り消せない」は TX の中の `canceled_at` の検査と `WHERE canceled_at IS NULL` の変更行数で成り立つ（T5）。
- 判定: ready（Q1〜Q4 は TD-848 で決まり、振替の分の正本は計画の commit に入れた）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-163 取消の形（movement の無効化と求め直し） | decision-log、ADR D7・横断の表、DB_DESIGN D-6、tracking の `business_date`・`is_voided` | S3 | T1〜T4・T13 | L3-1 |
| ADR SPEC-STK-CNT-D2・D3（変えない） | ADR | `stock_ledger.rs` は変えない | T3・T4 | L3-1 |
| ADR SPEC-STK-CNT-D6 完了の評価（変えない） | ADR、35 | 変えない | T6 | — |
| TRACE-D4 物理削除しない・理由必須 | 65 | S3・S4 | T1・T5・T13 | L3-1 |
| TRACE-D5 訂正は取消と入れ直し（TD-848 の Q2）、入れ直しの日付の案内 | 65 §65.5b、79 §79.6・§79.8 | command を足さない、S6 の dialog の文 | T4 の (e)・T10 | L3-1（dialog の文）・L3-4 |
| TRACE-D6 movement の無効化 | 65 §65.6.2 | S3 | T1・T3・T13 | — |
| TRACE-D14 振替の取消（D-143 (5) を置き換える。TD-848 の Q1） | 65、79 §79.6・§79.8、decision-log D-143・D-163、transaction §11a-11b | S2〜S4・S6・S7 | T13・T10・T12 | L3-3 |
| §65.4.1 状態の正規化（5 つの記録の `canceled`） | 65 | S4 | T7 | L3-1・L3-3 |
| §65.5b 詳細の取消の UI（送信中・失敗と結果不明の取り直し・待ちの止め） | 65、UI_TECH_STACK UI-USW-D5（`inventory-record-cancel`） | S6 | T10 | L3-1〜L3-5 |
| §65.7.2 記録ごとの無効にするもの | 65 | S3 | T1・T2・T3・T13 | L3-2・L3-3 |
| §65.8.2 履歴に無効の行を出さない（変えない。TD-848 の Q4） | 65、66 | 変えない | T1（`list_movements` に出ない） | L3-1 |
| 31 §12.5 手動販売の `manual_sale_id` | 31 | S3 | T2 | — |
| 31 §12.6a 詳細の状態と取消の field | 31、21 §10.5a | S4 | T7 | L3-1 |
| 31 §12.6c cancel_inventory_record | 31 | S3 | T1〜T5・T13 | L3-1〜L3-3 |
| 44 §23.7b command・DTO・`ReferenceType` の受け方 | 44 | S5 | T11、bindings の差分 | — |
| transaction §4-5・§11a-11b 取消の列・POS §13 `manual_sale_id` | db-design | S2・S4 | T9 | — |
| 22 §23 / MNT-03-D24 migration v15 | 22（§21 の番号の規則） | S2 | T9 と既存の連番の test | — |
| D-052 C25 invalidation（作業画面の直近の一覧の 3 つの root を含む） | decision-log D-052、UI_TECH_STACK §2.5、65 §65.5b | S6 | 既存の invalidation の oracle の test に entry を足す、T12 | — |
| 74 操作ログの種類 | 74 | S8 | T12 | — |
| 56・57 売上（変えない） | 56・57 | 変えない（`is_voided=0` を読む） | T2 | L3-2 |
| 36 整合性（変えない） | 36 | 変えない | T3 で整合性の検査が差を出さない | 非対象 |

隣の契約の sweep: 振替（79・D-143）は上の TRACE-D14 の行。振替の作成（31 §12.6b）は変えない。CSV取込みの取消（§65.5a・32）は変えない（`void_movements_by_reference` は読むだけで共有する）。商品の最終入庫日（20）は S4。関門 16 の「届いた」と取消の組合せは後から merge する lane が足す（未着で取り消した入庫に「届いた」を出さない）。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| T1 入庫の取消（状態の遷移: 保存 → 取消） | movement が残る・別の記録の movement も消える・行が消える | Rust unit（BIZ） | `test_cancel_receiving_voids_own_movements_and_recomputes_req208` | 取消の後に在庫が保存の前の値でない、その記録の movement が `is_voided=0`、同じ商品の別の入庫の movement が `is_voided=1`、header・明細・movement の行数が減る、`canceled_at`・`cancel_reason` が無い、操作ログ `inventory_record_cancel` が無い、`list_movements` に取り消した行が出る |
| T2 手動販売の取消と売上 | 売上が日次・月次に残る・別の手動販売の売上も消える | Rust unit（BIZ と sales の読取り） | `test_cancel_manual_sale_voids_linked_sale_records_req208` | 取消の後の日次売上・月次の売上にその行が残る、同じ日・同じ品・同じ額の別の手動販売の行が消える（合成の 2 件 1,000 円から片方を取り消して 500 円が残らない）、保存で `manual_sale_id` が書かれない |
| T3 返品・交換と廃棄 | movement の無い明細の扱い・交換の 2 方向 | Rust unit | `test_cancel_return_and_disposal_void_only_saved_movements_req208` | `register_processed` で movement の無い返品の取消が失敗する・在庫を動かす、交換の戻りと渡しの片方だけ戻る、廃棄の取消で在庫が戻らない、取消の後に整合性の検査（36）が差を出す |
| T4 要再確認の理由を作らない（H5 の回帰）と入れ直しの日付 | 取消で要再確認の理由が増える・写しが「記録が無かった値」でない・範囲が狭まる向きを誤って期待する | Rust integration（BIZ と `stock_ledger`） | `test_cancel_never_adds_recheck_and_matches_record_absent_req208` | (a) 入庫の後に同じ日の fresh な計数 → 取消で要再確認になる・在庫が計数の値でない、(b) 既定の計数と同じ日付の入庫（要再確認）→ 取消で要再確認が解けない、(c) 計数の後の日付の入庫 → 取消で在庫がその数だけ戻らない、(d) 翌日付の出庫 −5 を保存 → 同じ日に fresh な計数 10 → 既定の計数 20 を順に保存 → その出庫を取り消す: 取消の前の範囲が 10〜15 でない、取消の後の範囲が 10〜20 でない（狭まる向きを期待しない）、理由が前後とも `ConflictingCounts` だけでない、(e) 11/18 の入庫 +3 → 11/19 の既定の計数 10 → 11/21 にその入庫を取り消して入れ直す: 取消の後の在庫が 10 でない、11/18 で入れ直した後の在庫が 10 でない、11/21（作成の画面の既定の今日）で入れ直した後の在庫が 13 でない（dialog が日付を名指す理由の固定）。(d) の数は計画の review〈Codex、2026-11-21〉の実測、(e) の数は導き（未実測。実装の test が確かめる）。(a)〜(d) の各場合で取消の後の写し（在庫・範囲・理由）が、その記録を保存しなかった DB の求め直しと同じでない |
| T5 negative（再取消・無い・空の理由・種類の外・結びの無い手動販売・上限・失敗で DB 不変） | 2 回目で何かが変わる・拒否の後に DB が変わる・書込みの後の失敗で途中まで残る | Rust unit | `test_cancel_inventory_record_rejects_and_leaves_db_unchanged_req208` | 2 回目の取消が通る・`canceled_at` が変わる（振替でも）、無い ID が NotFound でない、空・半角空白・全角空白（U+3000）・tab だけの理由が通る、`csv_import`・`stocktake` が通る、`manual_sale_id` の無い手動販売の取消で movement が無効になる、上限の例（在庫が上限の商品を手動販売で 1 売り、別の入庫で上限に戻してから、その販売を取り消す。header・売上・movement を更新した後の求め直しで上限を 1 超える）が拒まれない、どの拒否の後も header・売上の行・movement・商品の写し（`stock_quantity`・範囲）・操作ログが取消の前と同じでない |
| T6 完了した棚卸し（凍結した評価と、求め直る差異・参考の差を分ける） | 評価の列が変わる・参考の差の有無を誤って期待する | Rust integration | `cancel_after_stocktake_completion_keeps_valuation` | 両方の場合で、取消の前後に `total_cost`・`valuation_quantity`・`valuation_cost_price` が変わる。(a) 計数に含まれた入庫（入庫 +3 → その入庫を見た fresh な計数 10 → 完了 → 入庫を取り消す）: 求め直した数が 10 でない、参考の差の行が出る、明細の帳簿との差が 7 → 10 に求め直らない（差異・ロス原価は求め直る側）、(b) 計数の後で完了の日までの日付の入庫（計数 10 → 入庫 +3 → 完了〈評価数量 13〉→ 入庫を取り消す）: 参考の差の行が出ない、行の完了の時点の数が 10 でない。(a) の数は計画の review〈Codex、2026-11-21〉の実測、(b) の数は導き（未実測） |
| T7 一覧と詳細の状態 | 取消済みが「有効」に混ざる | Rust unit（IO） | `inventory_records_status_canceled_for_five_record_types` | 取り消した 5 つの記録（振替を含む）の一覧の status が `active`、`status=canceled` で出ない・`active` で出る、詳細の `status`・`canceled_at`・`cancel_reason` が違う、直近の一覧の `canceled_at` が無い、商品の最終入庫日が取り消した入庫の日付 |
| T8 mutation の注入 | 構造の推論だけで済ませる | 手で注入（記録は PR の comment） | T1・T2・T4・T13 | 無効化を外す・求め直しを外す・`manual_sale_id` で絞らない・求め直しを返った商品の先頭の 1 つだけにする・取り消せる種類の match から `TransferRecord` を外す注入で、T1・T2・T4・T13 のどれも red にならない |
| T9 migration v15 | 列・CHECK・FK・index が無い、既存の行が取消済みになる | Rust unit（migration） | `migration_v15_adds_cancel_columns_and_manual_sale_link` | 5 つの header（`transfer_records` を含む）のどれかに列が無い、index が無い、v14 の DB の既存の行の新しい列が NULL でない、`canceled_at` だけ・半角空白だけの理由・Z004 の行への `manual_sale_id`・無い手動販売の ID が通る（全角空白・tab は DB の `trim` が除かないので T5 の BIZ で見る） |
| T10 記録の詳細の取消 | 取消済みに button が出る・理由なしで送る・二重に送る・失敗や応答の消失で取消済みと負の在庫が分からない・待ちの間に離れられる | vitest | `RecordCancel.test.tsx`（5 つの詳細の describe） | 有効でない記録に button が出る、空の理由で送れる、送信中に「取り消す」がもう一度押せる、dialog の入れ直しの文に記録の業務日付（例: `2026-11-18`）が出ない、手動販売で売上の文が出ない、成功で C25 の invalidate が呼ばれない・取消済みの Alert に理由が出ない、失敗・結果不明の後に詳細を取り直さない、取り直しが `canceled` で C25 が呼ばれない・在庫 −1 の movement の商品に負の在庫の文が出ない・在庫 0 の商品に出る・movement の無い返品の明細（`register_processed`）の在庫 −1 の商品に出る・長さの商品の −30 cm が `-0.3 m` で出ない、取り直しが `active` で dialog と理由が残らない、取消と取り直しの 2 回の失敗の後に dialog と理由が消える・詳細が error の表示に替わる・「状態をもう一度確かめる」が出ない・その前に「取り消す」が押せる・「状態をもう一度確かめる」が取消を送る・その読取りが `active` を返した後に同じ理由で「取り消す」が押せない、送信中に `useBlocker` と `useWindowCloseGuard("inventory-record-cancel")` が登録されない・応答の後に解かれない、取消済みの badge が色だけ、振替の詳細に向きを入れ替える案内か「取消はできません」が出る |
| T11 command の wire | DTO の field と種類の受け方が違う | Rust unit（CMD） | `cancel_inventory_record_cmd_maps_errors` | 種類の外（`csv_import`・`stocktake`）が `validation` でない、無い ID が `not_found` でない、snake_case の 7 つの値（`"transfer_record"` 等）が `ReferenceType` に変換できない、定義に無い文字列（`"receiving"`）の変換が error にならない、`cancel_inventory_record` が main thread の外で動く定義でない（`assert_commands_run_off_main_thread`） |
| T12 直近の一覧と操作ログ | 取消済みが直近の一覧で有効に見える・取消の前の cache が残る・種類が「その他」 | vitest | `ReceivingPage.test.tsx`・`DisposalPage.test.tsx`・`ReturnExchangePage.test.tsx`・`StockTransferPage.test.tsx` の直近の一覧の describe、`RecordCancel.test.tsx` の作業画面へ戻る describe、`OperationLogsPage.test.tsx` の種類の表 | 取り消した行に「取消済み」が出ない（振替の画面は一覧の `status` が `canceled` の行）、入庫・返品交換・廃棄の作業画面で直近の一覧を読み込んだ（cache がある）後に詳細で取り消して作業画面へ戻ると、その行が「有効」のまま出る、`inventory_record_cancel` が「その他」で出る |
| T13 振替の取消（減らす商品と増やす商品の両方） | 片側だけ戻る・別の振替の movement も消える・行が消える | Rust unit（BIZ） | `test_cancel_transfer_voids_every_line_and_recomputes_all_products_req208` | 減らす 1・増やす 2 の振替（明細 3、movement 3）を取り消した後に、3 つの商品のどれかの在庫が振替の前の値でない、その記録の movement のどれかが `is_voided=0`、同じ商品の別の振替・入庫の movement が `is_voided=1`、`transfer_records`・`transfer_items`・movement の行数が減る、増やした商品を売った後の取消で `stock_warnings` にその商品が出ない、操作ログの無効にした movement の数が 3 でない |

## L3

- 入庫の記録の詳細 / 開発用の Windows の native build で、合成の商品 1 つに入庫 +3 を保存し、入出庫履歴 → その行 → 「この記録を取り消す」→ 理由「打ち間違い」→ 確定 / 詳細が「取消済み」と取消の日時・理由を出し、商品と在庫の画面でその商品の在庫が入庫の前の値、在庫変動の履歴にその入庫の行が無く、入出庫履歴の状態「取消済み」でその記録が出て、入庫の画面の直近の一覧の行に「取消済み」が出る（L3-1）。
- 手動販売の記録の詳細 / 合成の商品を今日の日付で手動販売（金額 500 円）→ 日次売上で今日の手動販売の行と合計を見る → 記録の詳細で取り消す / 日次売上の今日の手動販売の行がその分（500 円）だけ減り（同じ日・同じ商品の別の手動販売が無ければ行が消える）、合計が 500 円減り、在庫が販売の前の値に戻る（L3-2）。
- 詰め替え・切り分けの記録の詳細 / 合成の商品 2 つ（元・先）で、元を 2 減らし先を 4 増やす詰め替え・切り分けを保存 → その記録の詳細で取り消す / 詳細に向きを入れ替える案内が無く「取消済み」と取消の日時・理由が出て、商品と在庫の画面で元と先の両方の在庫が保存の前の値、詰め替え・切り分けの画面の直近の一覧の行に「取消済み」が出る（L3-3）。
- 過去の日付で入れ直す / 合成の商品に昨日の日付で入庫 +3 を保存 → 今日の日付でその商品を「数を保存」（計数）→ その入庫の詳細で「この記録を取り消す」を押す → dialog の入れ直しの文が昨日の日付を名指すのを見る → 理由を入れて確定 → 入庫の画面で日付を昨日に変えて +3 を入れ直す / dialog に昨日の日付が出て、入れ直した後の商品と在庫の画面の在庫が計数の値のまま（L3-4）。
- 取消の後に閉じる / L3-1 の取消の確定の後（「取消済み」が出た後）に窓の × を押す / 確認の窓を出さずにアプリが閉じる（待ちの止めが解けている。待ちの間の確認の窓は応答が速く手では出しにくいので T10 で見る）（L3-5）。
- fixture: 合成の商品 4 つ（開発の DB。店の実データを使わない）。Eligibility: native の画面の操作と表示の確かめで、新しい tool・故障注入は要らない。

## owner の決定（repo 外の回答台帳 TD-848、2026-11-20）

起票の「owner に諮ること」への答え。全て起票の推奨のとおり。

- Q1 振替（詰め替え・切り分け）を同じ取消に入れるか: A（入れる）。Scope の S1〜S8 と T13・L3-3 に入れた。選ばなかった B（今の向きを入れ替えた振替で戻す）は、戻す記録が同じ日付の別の記録になり、同じ日の fresh な計数があると要再確認を出す。
- Q2 打ち間違いの直し方: (a)（取り消してから作成の画面で入れ直す。今の Scope のままで足りる）。Revisit: 明細の多い記録の入れ直しが負担と分かったら (b)（取消の後に作成の画面を取り消した内容で埋めて開く。DB と command を変えずに足せる）。(c) 訂正の command は採らない（D-163 の Alternatives）。
- Q3 Risk: R3（異議なし。`## Recovery`・`## Data Safety` は足さない）。
- Q4 在庫変動の履歴（商品ごと）に取り消した記録の行を出すか: (a)（出さない。今の Scope と §65.8.2 のままで足りる）。Revisit: 見せる要望が出たら (b)（`list_movements` の結果に field 1 つと 66 の表示。DB を変えずに足せる。Z004 の取消の行も同じく出る）。

## 計画の review に問う点

- 取消の形 B の「要再確認を作らない」「記録が無かった値」の導き（Design Readiness）に反例があるか。特に fresh な計数の `seen_movement_id` の規則と、既定の計数・精算の窓の組合せ。
- 完了した棚卸しの参考の差（ADR D6 の比較の範囲）に、取り消した movement が期待どおりに出るか（`stocktake_service.rs:748` の読み方）。
- 手動販売の結びの無い記録を拒む形（31 §12.6c の手順 3）が、本番の DB が無い前提で足りるか。
- 冪等キーを持たない取消（再取消は拒否）で、応答を失ったときの画面の振舞い（65 §65.5b の失敗）が足りるか。
- 1 つの command にした形（`ReferenceType` を受ける）が、44 §23.3 の error の写像と bindings で問題ないか。
- 振替の取消で、増やした商品を売った後の負の在庫（Design Readiness の (4)）を止めずに知らせる形で足りるか。
- この変更で不要になる記述・条件は無いか（65 の `corrected` の残り、`types.ts:95` の「訂正済み」、振替の詳細の `TRANSFER_CORRECTION_GUIDE` とその test の期待）。
