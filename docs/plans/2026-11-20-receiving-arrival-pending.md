# Plan Packet: 届く前に入れた入庫を「まだ届いていない」で保存し、届いた日に在庫へ入れる（導入の前の関門 16）

導入の前の関門 16〜18（[Plans](../Plans.md#導入の前の関門)）の最初の lane。関門 17・18 は別の lane にし、順と理由は [backlog](../backlog.md) の 3 つの項目に書いた（下の「分け方」）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行の「DB」（`receiving_records` に列を 1 つ足す migration）、「Tauri command の DTO」（`create_receiving` の要求に 1 field、`get_receiving_record` の結果に 2 field、新しい command 1 つ、入出庫の記録の一覧の状態の値 1 つ）、「状態を持つ画面・operator の業務の流れ」（入庫の画面の保存、入庫の記録の詳細の新しい操作、入出庫履歴の状態の絞り込み）に当たる。R4 の行には当たらないと判断した: migration は `ALTER TABLE … ADD COLUMN` だけで、表を作り直さず、行を消さず、既存の行の値を変えない（足す列の既定が今の意味〈届いた〉と同じ）。D-118 の v9・vZ・振替の vT が R4 だったのは `inventory_movements` を作り直したため（[振替の packet](2026-10-29-stock-transfer.md) の Reason）で、本 lane は作り直さない。書き換える既存の値は「届いた」を記録するときの未着の入庫の日付だけで、それは入力の日の仮の値（入力の時刻は `created_at` に残る）。owner は TD-763（2026-11-17）で「R4 見込み」と述べたので、owner が R4 を選べば `## Recovery` と `## Data Safety` を足す（下の「owner に諮ること」の Q3）。workflow の file（`.github/`・`scripts/`）に触れない。

## 分け方（関門 16〜18）

- 3 つを別の lane にし、設計だけの lane は置かない。各 lane の計画の commit に、その lane の正本の直し（ADR・decision-log・function-design・DB 設計）を入れる（plan-first）。
- 理由（触る表）: 16 は `receiving_records`（と返品の範囲の答えしだいで `return_records`）、17 は `stocktakes`、18 は 4 つの記録の header と、手動販売の売上の行を元の記録へ結ぶ列（今の `sale_records` は手動販売の記録を指さない。`src-tauri/src/biz/inventory_service/manual_sale.rs:265` の `insert_sale_record` は `csv_import_id: None` で書き、手動販売の ID を持たない）。重なる表が無い。
- 理由（在庫の求め直しへの効き方）: 16 は求め直しの規則（ADR SPEC-STK-CNT-D3 の判定表）を変えない（未着の入庫は movement を作らず、届いた日に作る movement は今の入出庫の窓で正しく判定される。下の Design Readiness）。17 は求め直しを変えず、完了の評価だけを読み替える。18 は求め直しの入力（有効な movement）を変える。互いの結果に依らない。
- 理由（migration）: 3 つとも `ADD COLUMN` で足り、表を作り直さないので、版をまとめても作り直しの回数は減らない。1 つの版にまとめると、先に入る lane がまだ実装しない lane の列を入れることになり、後のための足場になる（AGENTS.md「最小の実装」）。版は lane ごとに分け、22 §21 の規則（先に merge する側が小さい番号）に従う。
- 理由（L3・review の負担）: 画面が重ならない（16 は入庫の画面と入庫の記録の詳細、17 は棚卸しの完了と記録の詳細、18 は 4 つの記録の詳細）。1 本にすると review の Contract Ledger が 3 つの別の契約を抱え、L3 も 3 つの流れを 1 回で見ることになる。
- 順: 16 → 18 → 17。16 を先にするのは、今 D-118 の窓が黙ってずれる唯一の受け入れた穴の解消で（ADR の Consequences の「入出庫を事象の前に入力した場合」）、18 の取消が未着の入庫も扱えるように状態を先に決めるため。17 を最後にするのは、評価を「基準日の後の記録を外す」形で読み替えるので、16 の未着（movement が無い）と 18 の取消（無効の movement）の意味が決まってから test に入れられるため。17 は年末にだけ効くので、最後でも導入に間に合う。

## Goal

Goal Invariant:

### 最小完了条件

- 品が届く前に入庫を入れるとき、店の人が入庫の画面で「まだ届いていない」に印を付けて保存できる。その記録は在庫を動かさない。
- 品が届いたら、その入庫の記録の詳細で「届いた」を押して届いた日を選ぶと、その日付で在庫が増える。その後の棚卸しの「入れ直す」・数の保存に対して、届いた日で窓が決まる（ADR SPEC-STK-CNT-D3 のまま）。
- 届く前に「入れ直す」をしても、届いた後に在庫が実物より少なくならない（ADR の Consequences の例: 棚 5、未着の入庫 +3、入れ直して 5、届いて実物 8 → 写しも 8）。
- 未着の入庫が残っていることが、入庫の画面と入出庫履歴で分かり、そこから「届いた」まで進める（D-127）。

### 失敗定義

- 未着で保存した入庫が在庫・在庫変動の履歴・要再確認に効く。
- 「届いた」の後に、届いた日でなく入力の日で窓が決まる、または同じ記録で movement が 2 回できる。
- 印を付けない普通の入庫の振舞い（保存で在庫が増える、窓、冪等の再送）が変わる。
- 未着の入庫が画面のどこにも出ず、届いた後に押し忘れても気づけない。

### 非目的

- 返品・交換の未着（下の Q2 の答えの前は入れない）。
- 届いた数が入れた数と違う・一部だけ届いたときの扱い（今は入庫の取消〈関門 18〉と入れ直しで直す。分けて届けるのは別の入庫として入れる）。
- 届く予定の日の管理・知らせ（予定の日は持たない）。
- 棚卸しの基準日（関門 17）と記録の取消（関門 18）。
- 印を付けた後に外す（未着をやめて今の日付で在庫に入れる）操作。「届いた」で今日を選べば同じになる。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 確かめた事実（2026-11-20、`2e058901`）

- 入庫の日付は既に「品が店に届いた日」で、ADR D3 の入出庫の窓はこの日付に依る（[31](../function-design/31-biz-inventory-service.md) の D-118 の節、[61](../function-design/61-ui-receiving.md) の冒頭の注）。届く前に入れると、保存した後の fresh な計数は、その movement を id で「入る」とみなす（`src-tauri/src/biz/stock_ledger.rs:144` の `count.fresh && movement.id <= count.seen_movement_id` → `Included`）。これが ADR の Consequences の穴。
- 入庫の日付に今日より後の制限は無い（BIZ: `src-tauri/src/biz/inventory_service/common.rs:68-83` の `validate_business_date` は形と実在だけ。UI: `src/features/receiving/ReceivingPage.tsx:391-399` の `type="date"` に `max` が無い）。ただし先の日付を入れても、上の id の規則で fresh な計数に「入る」とみなされる。
- 入庫の保存は 1 TX で header・明細・`apply_stock_change`（movement の INSERT と求め直し）・操作ログ `receiving_create` を書く（`src-tauri/src/biz/inventory_service/receiving.rs:60` の `create_receiving`、`:225` の `apply_stock_change`、`:254-257` の操作ログ）。マスタの原価は書き換えない（差は結果の `cost_diffs` で示すだけ。同 `:273-278`）。
- 入出庫の記録の一覧の状態は、入庫は常に `'active'`（`src-tauri/src/db/disposal_repo.rs:414-417` の既定。CSV 取込みと棚卸しだけ `status_expr` を持つ、同 `:348-365`）。絞り込みの値は `all`・`active`・`canceled`・`in_progress`（同 `:262`）。
- `receiving_records` に状態の列は無い（[transaction](../db-design/transaction-tables.md) §4-5）。
- migration の最新は v13（`src-tauri/src/db/migration.rs:107`）。PR #294（Plan Commit 済み、未 merge）が v14（`ej_records`）と 22 §22・MNT-03-D23 を足す（`git show origin/agent/sd-bulk-ej-runtime:docs/plans/2026-11-20-sd-bulk-ej-runtime-a1.md` の S1）。本 lane は v15・22 §23・MNT-03-D24 の見込みで、先に merge した側に合わせて取り直す（22 §21）。PR #291 は migration に触れない（#294 の packet の「確かめた事実」）。
- decision-log の最新は D-160。本 lane は D-161 を予約する（`docs/DEV_WORKFLOW.md` Review Rules の連番の registry）。

## Scope

前提: 下の「owner に諮ること」の Q1 が B、Q2 が (a) のとき。ほかを選んだら Scope を書き直して計画の review の前に出す。

- S1 正本（計画の commit に入れる。owner の答えの後）: ADR [2026-10-16-stocktake-count-placement](../adr/2026-10-16-stocktake-count-placement.md) の Consequences の「入出庫を事象の前に入力した場合」を、未着の入庫で閉じたことと残る穴（下の Design Readiness）に置き換え、横断の表の入庫の行に「未着の入庫は届いた日に movement を作る」を足す。decision-log に D-161。[31](../function-design/31-biz-inventory-service.md) §12.3 に未着の保存、新しい §12.3a に `arrive_receiving`、§12.8 に fingerprint の項。[44](../function-design/44-cmd-inventory.md) の CMD-02 に command。[61](../function-design/61-ui-receiving.md) に UI-02-D20（未着の印と保存の結果）・UI-02-D21（入庫の画面の未着の知らせ）、§61.4・§61.9。[65](../function-design/65-inventory-record-traceability.md) §65.4.1・§65.5・§65.8.1 に状態「未着」と入庫の記録の詳細の「届いた」。[transaction](../db-design/transaction-tables.md) §4-5 に列。[22](../function-design/22-mnt-migration.md) §23（MNT-03-D24）。`docs/DB_DESIGN.md` は表を足さないので触れない。
- S2 migration v15（`src-tauri/src/db/schema_v15.rs`、`MigrationKind::Sql`、1 TX）: `ALTER TABLE receiving_records ADD COLUMN arrival_pending INTEGER NOT NULL DEFAULT 0 CHECK (arrival_pending IN (0, 1))`。`migration.rs` に登録する。既存の行は 0（届いた）。
- S3 BIZ（`src-tauri/src/biz/inventory_service/receiving.rs`）: `ReceivingCreateRequest` に `arrival_pending: bool`。真なら header と明細だけを書き（`apply_stock_change` を呼ばない、`stock_warnings` は空）、操作ログ `receiving_create` の detail に `arrival_pending: true`。fingerprint（31 §12.8）に `arrival_pending` を入れる（同じ key で印だけ違う再送を conflict にする）。新しい `arrive_receiving(conn, record_id, arrival_date, today)`: 1 TX で、記録が無ければ NotFound、未着でなければ ValidationFailed（「この入庫は {日付} に届いた記録が済んでいます」）、`arrival_date` が `validate_business_date` を通り、入力の日（`DATE(created_at)`）以上・今日以下でなければ ValidationFailed。通れば `receiving_date` を届いた日に、`arrival_pending` を 0 にし、明細ごとに `apply_stock_change(…, arrival_date, …)`、操作ログ `receiving_arrive`（detail に `record_type: receiving_record`・`record_id`・入力の日・届いた日）。結果は `stock_warnings`（create と同じ形）。
- S4 IO（`src-tauri/src/db/receiving_repo.rs`）: header の INSERT に列、未着を読む・届いた日を書く関数、`get_receiving_record` の行に `arrival_pending` と、未着の記録の明細の商品について入力の後に保存した計数（商品コード・商品名・日付。`stock_counts.counted_at` が header の `created_at` 以後）。入出庫の記録の一覧（`src-tauri/src/db/disposal_repo.rs` の入庫の spec）に `status_expr` `CASE WHEN arrival_pending = 1 THEN 'pending' ELSE 'active' END`、絞り込みの値に `pending`。
- S5 CMD（`src-tauri/src/cmd/receiving_cmd.rs`）: `arrive_receiving` を足し（今日は CMD が渡す。棚卸しの完了と同じ形）、`lib.rs` の handler と `src-tauri/dev-tools/src/bin/generate_bindings.rs` に登録し、`src/lib/bindings.ts` を作り直す。
- S6 UI 入庫の画面（`src/features/receiving/`）: 日付の欄の近くに checkbox「まだ届いていない（届いたら入庫の記録で『届いた』を押します）」（既定は外す）。印を付けて保存した結果に「まだ届いていない入庫として保存しました。在庫は『届いた』を押したときに増えます」と記録の詳細への link。画面の上に、未着の入庫があれば「まだ届いていない入庫が {n} 件あります」と、入出庫履歴を種別「入庫」・状態「未着」で開く link（D-127。0 件なら出さない）。recent list の行に「未着」の badge（色だけにしない）。
- S7 UI 入庫の記録の詳細（`src/features/inventory-records/ReceivingRecordDetailPage.tsx`）: 未着なら状態「未着」と「届いた」の button。押すと届いた日の欄（既定は今日、入力の日〜今日）の確認の dialog。入力の後に計数のある商品があり、選んだ日がその計数の日より後なら、dialog に「{商品名} は {日付} に数を入れています。品がその前に届いていたなら、届いた日をその日以前にしてください」（UI-02-D20 の判断。下の Q1 の B の一部）。成功で Toast と再取得（D-052 に入庫の届いたの entry を足す。`src/lib/invalidation-contract.ts`・`docs/decision-log.md` の D-052 の C 番号）、失敗で同じ記録のまま再試行できる。
- S8 UI 入出庫履歴（`src/features/inventory-records/InventoryRecordsPage.tsx`）: 状態の選択肢に「未着」、行の badge。操作ログの種類の表示（`src/features/operation-logs/operation-type-labels.ts`）に `receiving_arrive`「入庫の到着」と、関連記録の link（[74](../function-design/74-ui-operation-logs.md) の許可リストの `receiving_record` は今のまま当たる）。
- 呼出し側・隣接: `create_receiving` の呼出し（frontend の `receiving-request.ts`、BIZ・CMD の test、`src-tauri/src/seed_demo.rs` は SQL で INSERT するので列の既定で足りる）を `rg "ReceivingCreateRequest|create_receiving"` で全部直す。整合性の検査（36）・棚卸し（35）・求め直し（`stock_ledger.rs`）は変えない（未着の記録は movement を持たないので入力に現れない）。
- Non-scope: 返品・交換（Q2）、取消（関門 18）、基準日（関門 17）、届く予定の日。

## Acceptance Criteria

- Test Plan の行の test が全て PASS し、`cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`、`cargo run --bin generate_traceability -- --check`、`bash scripts/doc-consistency-check.sh` が ERROR なしで終わる。
- `cargo run --bin generate_bindings` の後の `src/lib/bindings.ts` の差分が、`ReceivingCreateRequest.arrival_pending`、`get_receiving_record` の結果の 2 field、`arriveReceiving` とその型、一覧の状態の値 `pending` だけ。
- 普通の入庫（印なし）の既存の test が変更なしで PASS する（`rg` で既存の `create_receiving` の test を挙げ、期待値を変えていないことを差分で示す）。
- `## L3` の項目が owner の PASS。

## Design Readiness

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D1・D2・D3（入出庫の movement の窓と鍵）と横断の表・Consequences、[31](../function-design/31-biz-inventory-service.md) §12.2・§12.3・§12.8 と D-118 の節、[61](../function-design/61-ui-receiving.md) UI-02-D10・D19、[65](../function-design/65-inventory-record-traceability.md) §65.4.1・§65.5・§65.8.1、[transaction](../db-design/transaction-tables.md) §4-5、[22](../function-design/22-mnt-migration.md) §21、D-052。
- 必要な設計成果物: updated in this PR（S1。計画の commit に入れる。owner の Q1・Q2 の答えの後）。
- plan にしかない durable な判断の昇格先: 「未着の入庫は movement を作らず、届いた日を記録したときにその日付で作る。ADR D3 の判定表は変えない」を D-161 と ADR の横断の表・Consequences へ。
- 窓が正しいことの導き（判定表は変えない）: 「届いた」で作る movement の id は、それより前に保存した全ての計数の `seen_movement_id` より大きいので、fresh な計数（今日 T）に対しては日付で決まる（`stock_ledger.rs:144-151`）。届いた日 E < T なら「入る」（品は数えた時に棚にあった）、E = T なら「決まらない」で要再確認、E > T は届いた日が今日以下なので、計数が届いた日より前の日のとき（「入らない」= 計数に足す）。既定の計数（日付 D）も E と D で決まる。つまり届いた日が正しければ窓は正しい。
- 前提・制約と延期した design gap: (1) 届いた日を誤って選ぶと窓がずれる（例: 届いたのは計数より前なのに、押したのが後で今日を選ぶと二重に足す）。dialog の知らせ（S7）で減らすが消えない。ADR の Consequences の受け入れた穴の列挙を、この形に置き換える。(2) 「届いた」を押し忘れると在庫が実物より少ないまま。入庫の画面の知らせ（S6）と入出庫履歴の「未着」で見え、棚の「入れ直す」をしても未着の入庫は在庫に入らない（届いた後に押せば足される。E が計数の日より後なら）。押し忘れたまま計数の後に押して届いた日を計数より前にすると「入る」で正しい。(3) 返品・交換は Q2 の答えまで延期。(4) 一部だけ届く・数が違うは非目的（関門 18 の取消の後に、取消して入れ直す）。
- 絶対保証の例外と抜け道: 「未着の記録は在庫に効かない」は movement を作らないことで構造上成り立つ。抜け道は、未着のまま `apply_stock_change` を呼ぶ経路で、BIZ の分岐の test（T1）と mutation の注入（T9）で止める。「同じ記録で movement が 2 回できない」は `arrive_receiving` の TX の中の未着の検査で成り立ち、並行の 2 回目は同じ TX の UPDATE の条件（`WHERE arrival_pending = 1` の変更行数が 1）で止める。
- 判定: not ready。理由: owner の Q1（店の人の操作）と Q2（返品の範囲）が未決で、S1 の正本の直しはその答えに依る。答えの後に S1 を計画の commit に入れ、計画の review に出す。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| SPEC-STK-CNT-D3 入出庫の窓（変えない） | ADR D3 の判定表 | S3（届いた日で movement を作る）。`stock_ledger.rs` は変えない | T3・T4 | L3-2 |
| ADR 横断の表の入庫の行・Consequences の「事象の前に入力」 | ADR | S1 | T3 | — |
| D-161（予約。未着の入庫） | decision-log | S1 | T1〜T4 | — |
| SPEC-STK-CNT-D2 求め直しの原理（入力が変わる TX は求め直す） | ADR D2、31 §12.2 | S3（届いたの TX で `apply_stock_change`） | T2 | — |
| 31 §12.3 create_receiving（未着の分岐） | 31 | S3 | T1・T7 | L3-1 |
| 31 §12.3a arrive_receiving（新設） | 31 | S3・S5 | T2・T5 | L3-2 |
| 31 §12.8 fingerprint | 31 | S3 | T7 | — |
| CMD-02 の command | 44 | S5 | T5（CMD）、bindings の差分 | — |
| UI-02-D10 保存の結果 / UI-02-D19 lock（変えない） | 61 | S6 | T10 | L3-1 |
| UI-02-D20 未着の印と結果・UI-02-D21 未着の知らせ（新設） | 61 | S6 | T10 | L3-1 |
| §65.4.1 状態・§65.5 詳細・§65.8.1 hub | 65 | S4・S7・S8 | T6・T11 | L3-2 |
| receiving_records の列 | transaction §4-5 | S2・S4 | T8 | — |
| 22 §23 / MNT-03-D24 migration v15 | 22（§21 の番号の規則） | S2 | T8 と既存の版の連番の test | — |
| D-052 invalidation（届いたの entry） | decision-log D-052、UI_TECH_STACK §2.5 | S7 | 既存の invalidation の oracle の test に entry を足す | — |
| 74 操作ログの種類と関連記録 | 74 | S8 | T12 | — |
| 36 整合性・35 棚卸しの完了・73（非対象） | 36・35・73 | 変えない（未着は movement を持たない） | T3 で完了の拒否の理由が増えないこと | 非対象 |

隣の契約の sweep: 返品・交換（63・31 §12.4）は Q2 で非対象。振替（79）・手動販売（62）・廃棄（64）は日付の意味が同じだが店の答え（TD-762）は入庫・返品だけで、非対象。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| T1 未着の保存 | 未着でも在庫・movement・要再確認が動く | Rust unit（BIZ） | `create_receiving_pending_writes_no_movement` | 未着でも `apply_stock_change` を呼ぶ（movement が 1 行以上、`stock_quantity` が変わる、`stock_warnings` が空でない） |
| T2 届いた | 届いた日でなく入力の日で movement を書く・header が未着のまま | Rust unit | `arrive_receiving_writes_movements_on_arrival_date` | movement の `business_date` が入力の日、`receiving_date` が変わらない、`arrival_pending` が 1 のまま、操作ログ `receiving_arrive` が無い |
| T3 ADR の例（状態の遷移: 保存 → 入れ直す → 届いた） | 届く前の入れ直しで、届いた後に在庫が少ない | Rust integration（BIZ と `stock_ledger`） | `pending_receiving_then_fresh_count_then_arrival_next_day` | 棚 5・未着 +3・入れ直して 5・翌日に届いた → 写しが 8 でない、または要再確認になる |
| T4 同じ日の届いた | 計数と同じ日の届いたが黙って入る | Rust integration | `arrival_same_day_as_fresh_count_is_recheck` | 届いた日 = 計数の今日のとき要再確認の理由 2 にならない |
| T5 negative（再試行・失敗） | 2 回目の届いた・範囲の外の日付・無い記録・届いた記録で movement ができる | Rust unit | `arrive_receiving_rejects_twice_out_of_range_missing` | 2 回目で movement が増える、入力の日より前・今日より後が通る、無い ID が NotFound でない、失敗の後に DB が変わる |
| T6 一覧の状態 | 未着が「有効」に混ざる・絞り込みが効かない | Rust unit（IO） | `list_inventory_records_receiving_pending_status` | 未着の記録の status が `active`、`status=pending` で 0 件、届いた後に `pending` のまま |
| T7 冪等 | 同じ key で印だけ違う再送を replay とみなす | Rust unit | `create_receiving_pending_flag_in_fingerprint` | 印だけ違う再送が `idempotent_replay` で通る |
| T8 migration v15 | 列・CHECK・既定が無い、既存の行が未着になる | Rust unit（migration） | `migration_v15_adds_arrival_pending` | 列が無い、`arrival_pending=2` が通る、v14 の DB の既存の行が 0 でない |
| T9 mutation の注入 | 構造の推論だけで済ませる | 手で注入（記録は PR の comment） | T1・T3 | 未着の分岐を外す注入で T1・T3 が red にならない |
| T10 入庫の画面 | 印・結果の文・未着の知らせ・badge が出ない | vitest | `ReceivingPage.test.tsx` の未着の describe | 印が要求に載らない、結果の文が出ない、未着 0 件で知らせが出る・1 件で出ない、badge が色だけ |
| T11 入庫の記録の詳細 | 「届いた」が届いた記録に出る、日付の範囲・知らせ・再試行・再取得が欠ける | vitest | `OtherRecordDetailPages.test.tsx` の入庫の未着の describe | 届いた記録に button が出る、範囲の外の日付で送る、計数の後の日付で知らせが出ない、失敗の後に再試行できない、成功で invalidate が呼ばれない |
| T12 操作ログ | 新しい種類が「不明」になる | vitest | `OperationLogsPage.test.tsx` の種類の表 | `receiving_arrive` の表示が無い |

## L3

- 入庫の画面 / 開発用の Windows の native build で、商品を 1 つ入れ「まだ届いていない」に印を付けて保存 / 結果に「まだ届いていない入庫として保存しました…」が出て、商品と在庫の画面でその商品の在庫が変わっていない。入庫の画面の上に「まだ届いていない入庫が 1 件あります」が出る（L3-1）。
- 入庫の記録の詳細 / L3-1 の知らせの link → 入出庫履歴（状態「未着」）→ 行 → 「届いた」→ 今日のまま確定 / 状態が「有効」になり、在庫が入れた数だけ増え、在庫変動の履歴にその日付の入庫の行が出て、入庫の画面の知らせが消える（L3-2）。
- fixture: 合成の商品 1 つ（開発の DB。店の実データを使わない）。Eligibility: native の画面の操作と表示の確かめで、新しい tool・故障注入は要らない。

## owner に諮ること（未決）

- Q1 届く前に入れる入庫の店の人の操作（TD-763 の「届いた日を持たせる」の形）:
  - A: 日付の欄に届く予定の日（先の日付）を入れる。判定表の fresh な計数の id の規則を「業務日が今日以下のときだけ」に直す（migration なし、画面の追加なし）。予定の日がずれると窓が黙ってずれ、届いた日が分からないときに入れられない。
  - B: 「まだ届いていない」に印を付けて保存し、届いたら入庫の記録で「届いた」を押して届いた日を選ぶ（本 packet の Scope）。普段の入庫は今のまま（印は既定で外す）。手間が増えるのは届く前に入れたときだけで、押し忘れは画面の知らせで見える。届いた日の選び違いは dialog の知らせで減らす（S7 の知らせは本起票の AI の調べから足した案で、B を選ぶとこれも含む）。
  - C: 今のまま、届いてから入れる運用にする（code なし）。店の約束に頼る形で、TD-763 と合わない。
  - 推奨: B（candidate）。理由: 予定の日を当てなくてよく、ずれは要再確認か画面の知らせで見える（「黙ってずれなければよい」TD-267）。
- Q2 返品の範囲（関門 16 は「入庫・返品」）: アプリの返品・交換は客からの返品と交換（[63](../function-design/63-ui-return-exchange.md)）で、交換は渡す品（在庫減）を同じ記録に持つ。店の言う「戻る前に入れる返品」が、客からの返品か、問屋へ返す品か（アプリに問屋への返品の流れは無い）、交換を含むかは確かめていない。
  - (a) 本 lane は入庫だけにし、返品は店の答えの後に同じ形（未着の印と「届いた」）で足す lane を起こす。
  - (b) 本 lane で返品・交換の種別「返品」（戻る明細だけ）にも同じ印を足す。
  - (c) 交換も含め、記録の全部を届いた日まで止める（渡す品も止まる）。
  - 推奨: (a)（precondition-dependent。店に「品が戻る前に入れる返品はどんな場面か〈客からの返品・問屋への返品・交換〉」を確かめてから決める）。(a) なら関門 16 は返品の lane が済むまで残る。
- Q3 Risk: TD-763 は R4 見込み。Risk Tiers で R3 と判断した（上の Reason）。owner が R4 を選べば `## Recovery`（v15 の前の backup に戻す、22 §16 の「回復」）と `## Data Safety` を足す。推奨: R3（confirmed は owner）。
