# Plan Packet: 棚卸しに基準日を持たせ、基準日より後の入出庫と精算を年末の評価額から外す（導入の前の関門 17）

起票: 2026-11-24（起点 `1edcedc3`）。出所は [backlog](../backlog.md) の「棚卸しの基準日の runtime」と [Plans](../Plans.md#導入の前の関門) の関門 17（owner の決定 2026-11-19、repo 外の回答台帳 TD-832）。decision-log の番号は D-167（予約済み）。O2 は (a) に決まり（TD-894）、O1 は向きが E（問屋の締め日から翌年の仕入れを自動で分ける、TD-898）、lane の分け方が O1-1 = (a)（TD-929、2026-11-27）に決まった: この lane は基準日と O2 だけで、E は次の lane。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「DB・migration」（`stocktakes` と `stocktake_items` に NULL を許す列を 1 つずつ足す）、「Tauri command の DTO」（`complete_stocktake` の入力と記録の詳細の結果）、「状態を持つ画面・operator の業務の流れ」（完了の確認の窓）に当たる。R4 には当たらないと判断した: migration は `ADD COLUMN` だけで表を作り直さず既存の値を変えない（先例は v15 の R3〈[record-cancel](2026-11-20-record-cancel.md)〉。R4 にした v9・v11 は行のある表の作り直し）。変わる数字はこの版より後に完了する棚卸しの評価額だけで、完了済みの評価額は書き換えない（SPEC-STK-VAL-D6）。本番の DB は無い（[project-memory](../project-memory.md)）。計画の review で表の作り直しが要ると分かれば R4 へ上げる。

## Goal

Goal Invariant:

### 最小完了条件

- 棚卸しの完了のときに基準日（店は大晦日）を選ぶと、評価額（仕入原価の総額）が基準日の終わりの時点の在庫で求まる。基準日より後の日付の入庫・返品交換・手動販売・廃棄・振替と、精算日が基準日より後の Z004 の売上は、完了の前に入っていても評価額を動かさない（完了が年明けにずれても、年明けの入荷と売上で総額が変わらない）。
- 棚卸しの結果と記録の詳細に基準日が出て、完了の後の参考の差は基準日の時点の在庫と比べる。

### 失敗定義

- 基準日より後の記録が評価額を動かす、または基準日以前の記録が評価から落ちる。
- 基準日を選ばないまま、未来の日付、開始日の前年の 12/31 より前の日付、または 12/31 以外の日を確認なしで完了できる（TD-935）。
- 登録日が基準日より後の初期在庫の品を、店が選ばないまま完了できる（TD-934）。
- 基準日までの精算を取り込んでいないことを知らせずに完了できる。
- 完了が在庫を動かす、完了済みの評価額を書き換える、今の在庫（写し）・要再確認・差異の値が今と変わる。
- この版より前に完了した棚卸し（基準日が NULL）の記録の詳細の値が変わる。

### 非目的

- 翌年の仕入れを問屋の締め日で分けて評価から外すこと（O1 の E は次の lane。[backlog](../backlog.md) の「取引先の意味（D-075 (6)「suppliers = メーカー/ブランド、問屋は持たない」）の見直し」）。
- 完了した棚卸しの基準日の変更、完了の取消。
- 入出庫履歴の一覧の棚卸しの業務日付（今は完了日）と「前回の棚卸し」の表示を基準日にすること。
- 開始のときに基準日を決める入力、基準日の既定値（下の Design Readiness の「入力の形」）。

## Scope

基準日の時点の在庫の定義（S1 で ADR D6 と 35 に書く。導き方は Design Readiness）: 商品の効く計数 C（値 V、日付 T）について、V に「C に入っていない記録のうち日付が基準日以前のもの」を足し、「C に入っている記録のうち日付が基準日より後で T 以前のもの」を引く。初期在庫は足しも引きもしない（どの計数よりも前の申告。ADR D3）。計数の無い商品は、初期在庫と、日付が基準日以前の記録の和。ただし初期在庫の登録日（その movement の業務日）が基準日より後の品は店の選択に従う（TD-934）: 「基準日にもう店にあった」は上のまま、「基準日の後に届いた」は初期在庫を基準日より後の記録として扱う（計数のある品は V から引き、計数の無い品は足さない）。日付は D3 の窓と同じ日付（入出庫は業務日、Z004 は精算日。`LedgerMovementRow.business_date`）。取消済みの記録（`is_voided`、D-163）は求め直しの入力に無いので、どちらにも入らない。

- S1 設計正本（実装より前の commit）: decision-log に D-167（末尾の追記）。ADR [stocktake-count-placement](../adr/2026-10-16-stocktake-count-placement.md) の SPEC-STK-CNT-D6（評価数量・完了の条件 1・初期在庫の確かめ・完了の後の差・確認の文〈「その日の最後の精算を取り込みましたか」を、精算日で問う「基準日の最後の精算を取り込みましたか」に〉。条件 1 の末尾の「大晦日に数えて締めや取込みが翌日なら、完了もその取込みの後」は基準日の検査への参照に置き換える）・D9（列）・Consequences（O2 の答えの穴。「店主の手間」の「締めが翌日なら完了も翌日」も基準日の検査への参照に置き換える）。[35](../function-design/35-biz-stocktake-service.md) の「完了」・§20.5・§20.6a・§20.8 の INV-3 と SPEC-STK-VAL-D3 の quantity の定義。[42](../function-design/42-cmd-sales-stocktake.md) の `complete_stocktake` の入力と `get_stocktake_record` の結果、[40](../function-design/40-cmd-product.md) の `completion_blocked`（商品の無い `SettlementPendingHint` と理由 `initial_stock_after_base_date`）、[20](../function-design/20-io-product-repo.md) §2.11 の `complete_stocktake` と §2.11a、[tracking](../db-design/tracking-system-tables.md) の「棚卸し」の節と stocktakes・stocktake_items の列、[22](../function-design/22-mnt-migration.md) の §3.1 の一覧と新しい節（§24・MNT-03-D25 の見込み）、[73](../function-design/73-ui-stocktake.md) の冒頭の表の「完了」「完了の結果」・UI-10-D4・§73.7・§73.8・§73.12・§73.13、[65](../function-design/65-inventory-record-traceability.md) の D-118 の節の棚卸し詳細の項、`docs/architecture/cmd-task-specs.md` の `complete_stocktake` の行。
- S2 migration（`src-tauri/src/db/schema_v<N>.rs` を足し、`db/mod.rs` の `mod` の行と `migration.rs` の `use` と `migrations()` に登録。`MigrationKind::Sql`、v15 と同じ形）: `ALTER TABLE stocktakes ADD COLUMN base_date TEXT` と `ALTER TABLE stocktake_items ADD COLUMN initial_stock_included INTEGER CHECK (initial_stock_included IN (0, 1))`（どちらも NULL を許し、既存の行は NULL のまま。NULL は「当たらない」、1 は入れる、0 は外す。止まる検査なし、作り直しの起点でない）。
- S3 BIZ（`src-tauri/src/biz/stocktake_service.rs`、`stock_ledger.rs`）: `CompleteStocktakeRequest` に `base_date` と `initial_stock_choices`（商品コード → 入れる・外す）。TX の前に `ValidationFailed`: 形と暦（`validate_count_date` の検査）、`today` より後、棚卸しの開始日の前年の 12/31 より前（TD-935。開始日から求めるので年 2 回目の棚卸しも通る）。完了の条件 1（`settlement_pending` の (i)〜(iii)）を、基準日を「日付が基準日の既定の計数」と同じに扱って当て、(ii) を、効く計数の日付と基準日の間の区間（両端を含む）と欠けの両隣の日付 [d_a, d_b] が重なるときも拒む形に広げる（間の欠けの売上が足し引きから落ちるため）。この区間の検査は fresh の n で切らない（k ≤ n の欠けも拒む。計数が基準日より後のとき、n 以下の欠けの売上は数えた数に入っているのに、取り込まれていないので基準日まで引き戻されない）。基準日の判定は商品に依らないので target を作らず、拒む条件と `has_pending` を商品の理由と基準日の判定の OR にする（`targets` が空でも hint があれば拒む）。初期在庫の登録日が基準日より後の品（計数の有無に依らない）に選択が無ければ、その品を理由 `initial_stock_after_base_date` の target で名指して拒み（TD-934）、選択は完了の TX で明細の `initial_stock_included` に書く（当たらない品の選択は使わず書かない）。評価数量を `max(基準日の時点の在庫, 0)` にする。基準日の時点の在庫は `stock_ledger.rs` の窓の判定（`judge`）を使う 1 つの関数で求め、完了と `after_completion_diff` の両方が呼ぶ（今の在庫を求める `derive_ledger`・`recompute_stock` の結果は変えない）。`after_completion_diff` は今の範囲（`ledger_at_cutoff(…, Some(完了日))`）の入力に同じ関数を当てて `valuation_quantity` と比べ、要再確認の範囲の両端も同じ定義で求める。`base_date` が NULL の完了済みの行は今のまま（完了の時点の在庫）。`completed_stocktakes_comparing_settlement` は変えない（範囲による候補のまま。計数に入っている基準日より後の精算を取り消すと基準日の時点の在庫が変わるので、基準日で絞らない）。差異・ロス原価・操作ログの `difference_count` は変えない。
- S4 IO（`src-tauri/src/db/stocktake_repo.rs`）: `complete_stocktake` が `base_date` と明細の `initial_stock_included` を書き、`get_stocktake_record_detail` がカーソルと一緒にそれらを読む（wire に出る `Stocktake` は変えない）。
- S5 CMD と生成物（`src-tauri/src/cmd/stocktake_cmd.rs`、`src/lib/bindings.ts`）: `complete_stocktake(stocktake_id, base_date, initial_stock_choices)`。`CompletionBlockReason` に `initial_stock_after_base_date`。`StocktakeRecordDetail` に `base_date`、明細に `initial_stock_included`、`AfterCompletionDiffItem` の `stock_at_completion`・`_low`・`_high` を `stock_at_base_date`…に改める（中身が基準日の時点の在庫になるため）。command は足さない（`lib.rs` は変えない）。
- S6 UI 完了（`src/features/stocktake/StocktakePage.tsx`、`hooks/useCompleteStocktake.ts`）: 完了の確認の窓に基準日の入力（「0 にする」の日付の入力と同じ `type="date"`、`min` は開始日の前年の 12/31、`max` は今日、既定値なし）と「基準日より後の入出庫と精算は評価額に入りません」の文。未選択と範囲の外の値（手で打った未来の日付を含む）では「確定する」を押せず、12/31 以外の日はその日で評価するときだけの明示の確認（check box）を入れるまで押せない（TD-935）。最後の精算の問いの文（`StocktakePage.tsx` の「今日の最後の精算（締め）を取り込みましたか」）を、精算日で問う「基準日の最後の精算を取り込みましたか」に直す（年明けに取り込んでも、基準日の精算がそろっていれば「はい」と答えられる。D-175）。結果の画面に基準日。`completion_blocked` は、`settlement_pending` があれば `targets` が空でも「取り込んでいない精算がある」の欄と panel を残して直し方の文を出す（今の `pending.length > 0` の条件を直す）。`handleZero` は残りの `targets` が空でも `settlement_pending` があれば panel を片づけない（最後の未計数の品を「0 にする」と直し方の文が消えるのを止める）。`initial_stock_after_base_date` の品は品ごとに「基準日にもう店にあった（評価に入れる）」「基準日の後に届いた（外す）」を選ばせ、全部選ぶまで確定できない（TD-934）。12/31 以外の確認と品ごとの選択は「棚卸し ID と基準日」の組に結び、同じ組の再試行では使い、基準日を変えたら外して答え直させる（前の組の選択を送らない）。
- S7 UI 記録の詳細（`src/features/inventory-records/StocktakeRecordDetailPage.tsx`）: 基準日の行、参考の差の列と説明の文を「基準日の時点の在庫」に（`base_date` が NULL の行は今の「完了の時点の在庫」の見出し）。選択のある明細に TD-934 の選択を出す。
- S8 docs の片付け（実装の PR に入れる）: `docs/Plans.md` の関門 17 の行を外す。`docs/backlog.md` の「棚卸しの基準日の runtime」を済みの形に（TD-765 の B）。`docs/ACCEPTANCE_SCENARIO.md` の Step 4 の確定の手順に基準日を選ぶ操作を足す。
- 呼出し側・隣接 test: `CompleteStocktakeRequest { … }` を作る所（`rg 'CompleteStocktakeRequest \{' src-tauri` で `stocktake_cmd.rs`・`stocktake_service.rs` の test・`inventory_service/cancel.rs` の test・`csv_import_service/tests/cross_feature_tests.rs`・`commit_tests.rs`）に `base_date` と空の `initial_stock_choices` を足す（既存の test は基準日を完了日にする。期待が変わる test と残す検査は AC6）。frontend は `StocktakePage.test.tsx`・`StocktakePage.suggest.test.tsx`・`StocktakeRecordDetailPage.test.tsx`。
- 生成物: `src/lib/bindings.ts`、`docs/function-design/90-traceability.md`（どちらも同期の後に作り直し、手で解かない）。

merge の順と migration の版: この lane は PR #307（PLU の runtime）の後に merge する（migration の版・22 の節・ID は #307 の後の番号を取る。版は 22 §21 の規則で実装の時点の `origin/main` の最大 + 1）。L3 は #307 の merge と同期の後にする（同期で `migrations()` の登録の列を手で解くと lane の手が runtime に及び、L3 のやり直しになる。[merge-evidence](../agent-guidance/merge-evidence.md) の MG-D13）。

並走の lane と重なる file の所有（Wave Operation。ほかの file は重ならない）:

| file | 本 lane が持つ行・節 | 重なる lane |
|---|---|---|
| `src-tauri/src/db/migration.rs`・`db/mod.rs`・`schema_v<N>.rs` | 本 lane の `mod`・`use`・`migrations()` の 1 件と自分の schema の file | PLU の runtime（上の「migration の版」） |
| `src-tauri/src/db/mod.rs`・`src-tauri/src/lib.rs`（起動の準備） | `db/mod.rs` の `mod` の 1 行だけ（`lib.rs` は変えない） | PR #311（更新・migration の前の保全。#307 の後に実装）。#311 が先に main に入れば、同期で入る main 側の変更が code に触れるので MG-D13 の B（closure と owner の承認）で L3 を使い回し、lane の手が docs と frontend の test の外に及べば L3 をやり直す |
| `docs/function-design/22-mnt-migration.md` | §3.1 の自分の 1 行と自分の節 | PLU の runtime・PR #311 |
| `docs/decision-log.md` | 末尾の D-167（merge 順で両方を残す） | D-164・D-166・D-168・D-169 の lane |
| `src/lib/bindings.ts`・`docs/function-design/90-traceability.md` | 生成物（取り込んだ後に作り直す） | PLU の runtime・関門 19・PR #304 |
| `src-tauri/src/biz/csv_import_service/tests/commit_tests.rs`・`cross_feature_tests.rs` | `CompleteStocktakeRequest` の `base_date` の行だけ | PR #304（JAN-8 の integration test を足す） |
| `docs/Plans.md`・`docs/backlog.md` | 関門 17 の行と「棚卸しの基準日の runtime」の項目 | 関門 14・19 の行、ほかの項目 |

Non-scope: 今の在庫の求め直し（`derive_ledger`・`recompute_stock`・整合性の検査）、計数の保存、Z004 の確定と取消、入出庫の記録の取消（D-163）、入出庫履歴の一覧の SQL、D-052 の invalidation（query key を足さない）、操作ログの文。

## Acceptance Criteria

- AC1: `cd src-tauri && cargo test --lib` で Test Plan の T1〜T3・T12 が PASS（基準日より後の記録を外し、基準日当日以前を残し、基準日より後の計数からさかのぼり、初期在庫・取消済み・届く前に入れた記録を引かない。登録日が基準日より後の初期在庫は選択どおり）。
- AC2: 基準日の形の誤り・未来の日付・開始日の前年の 12/31 より前・基準日までの精算の未取込み（計数と基準日の間の欠けを含む）・登録日が基準日より後の初期在庫の選択の欠けは、何も書かずに `ValidationFailed` か `completion_blocked` で拒む（T4・T5・T12。header・明細・在庫が変わらない）。
- AC3: 完了の後の参考の差が基準日の時点の在庫と比べ、基準日が NULL の完了済みの行は今の値のまま（`get_stocktake_record` の T6・T7）。
- AC4: migration が空の DB と行のある直前の版の DB に当たり、既存の行の値を変えない（T8）。`migrations()` の連番の既存の test が PASS。
- AC5: 確認の窓は基準日を選ぶまで（12/31 以外は明示の確認まで）確定できず、送る要求に基準日と初期在庫の選択が入り、結果と記録の詳細に基準日と選択が出る（T9 と、`npm test` の T10・T11）。
- AC6: 既存の test は、S3・S5 の引数・field の追加と field 名の変更の他は期待値を変えずに全部 PASS する。D6 が期待を変える test は次だけで（`rg -n 'complete\(|CompleteStocktakeRequest' src-tauri` の呼出し側を読んで確かめた分）、PR の差分で名指す: `test_complete_stocktake_req205_freezes_valuation_without_movements`（K1。評価数量 99 → 8〈記録から〉、写し 99・movement の件数・カーソルは同じ。後半の取込みの無い DB の完了は基準日の (i) で拒む期待に変える）、`test_get_stocktake_record_req206_after_completion_includes_backfilled_settlement`（基準日 = 完了日では欠け #7 で完了の前に拒む）、`test_get_stocktake_record_req206_after_completion_same_range_as_valuation`（基準日の (iii) で拒むので前の日の精算を足す。評価数量は 4・7 → 5・5、差が空のままは同じ）、`test_complete_stocktake_req205_settlement_pending_three_conditions_and_hint` の最後の K2-F（欠け #2 の両隣が基準日を挟むので拒む）、`test_complete_stocktake_req205_settlement_pending_fresh_cursor_uses_import_ids` の最後の完了（基準日の (iii)。前の日の精算を足して通す）、`test_complete_stocktake_cmd_req205_completion_blocked_wire`（fixture に精算が無く `settlement_pending` が null でなくなるので、精算を足して null の wire を保つ）。`start_stocktake` の実時計の `started_at` で始めて 2026 年の固定日（`D` ほか）で完了する fixture（`start_and_count_all` とそれを使う test の全部）は、`started_at` を完了日の年に揃える（揃えないと 2027-01-01 以降の実行で下限が 2026-12-31 以降になり、二重確定の test は初回の `unwrap()` で落ちる。期待値は変えない）。カーソルが NULL・NULL・0 の読みと、完了の後に埋めた欠けの差（-3）の検査は、`base_date = NULL` の旧記録の fixture で残す（`cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`）。`stock_ledger.rs` の既存の test は 1 行も変えない。
- AC7: `cargo run --bin generate_bindings` の後の `src/lib/bindings.ts` の差分が S5 の点だけ。`cargo run --bin generate_traceability -- --check` と `bash scripts/doc-consistency-check.sh`・`--target plan` が ERROR なし。
- AC8: Test Plan の mutation（M1〜M9）を 1 つずつ注入すると名指しの test が `cargo test --lib -- <test 名>` で exit 101（red）になり、戻すと exit 0。
- AC9: 「完了の時点の在庫」で評価する旧い文が、`docs/archive`・merge 済みの packet の外の設計正本に残らない（`rg -n 'max\((その時点の|写しの|完了の時点の写しの)在庫' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が、基準日が NULL の行の説明だけを返す）。

## Design Readiness

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D3（窓と判定表・A1）・D6・D7・D9・Consequences、35 の「完了」・§20.5・§20.5a（SPEC-STK-VAL-D1〜D6）・§20.6a、65 §65.5b と D-163 (6)、tracking の「棚卸し」と §16-17、22 §21・§23、73 UI-10-D4・§73.7、project-memory の Store Premises Facts の棚卸しの行。
- 必要な設計成果物: S1 の正本を updated in this PR。新しい決定は D-167。
- plan にしかない durable な判断の昇格先: 基準日の時点の在庫の定義と完了の条件は ADR D6 と 35、入力の形は 73 UI-10-D4、O1（向きと O1-1）・O2・TD-934・TD-935 の答えは D-167 と ADR の Consequences。
- 導き方（起票で手で導いた。code では未実測。計画の review の中心の問い）: 今の在庫は V + Σ（C に入っていない記録）。基準日の終わりで切ると、C より後で基準日以前の記録は残り、C より前で基準日より後の記録（C が基準日より後のときの、間の記録）は戻す。要再確認の範囲は、決まらない記録を入っている・入っていないのそれぞれとして同じ定義を当てた最小と最大（完了は要再確認 0 を求めるので、範囲が出るのは完了の後の差だけ）。
- 入力の形（決定: owner TD-935、2026-11-27）: 基準日は完了の確認の窓で選び、開始に手順を足さない（基準日が効くのは完了の TX の評価だけ）。既定値は置かない（年明けに「今日」のまま確定すると年明けの記録が評価に入り、確定は取り消せない）。下限は開始日の前年の 12/31（BIZ でも拒む）。12/31 以外の日は、その日で評価するときだけの明示の確認を入れて初めて確定できる（年 2 回目の棚卸しも通せる）。
- 届いた日との関係: 関門 16 は列を足さずに閉じた（D-170。入庫の日付の欄が「届いた日」）。基準日の判定は D3 の窓と同じ日付（`LedgerMovementRow.business_date`。入庫は届いた日）だけを読む。届く前に入れた入庫（計数に入っていて日付が計数の日付より後）は、今の在庫と同じく足しも引きもしない。届いた後に在庫が実物より少なくなる穴（ADR の Consequences、D-170 が残した）は基準日の時点の在庫にも同じ形で残る。
- 絶対保証の例外と抜け道: 「基準日より後の記録は評価に入らない」は記録の日付を信じる（日付を基準日以前にして入れた記録は入る）。登録日が基準日より後の初期在庫は、店が品ごとに「基準日にもう店にあった」か「基準日の後に届いた」を選ぶまで確定できない（TD-934。選んだ値は明細に残り、選び違えはそのまま評価に入る）。精算の境目は O2 の (a)。
- 延期した design gap: 商品の無い `SettlementPendingHint` と `initial_stock_after_base_date` の wire と画面の形（40・73。S1 で決める）。

### owner に決めてもらう事項

**O2 基準日と精算の境目（決定: (a)、owner TD-894）**: Z004 は精算の中の売上の日時を持たないので、「精算日が基準日より後の精算は基準日より後」とみなす。基準日に精算が無く次の精算が基準日の売上を含むときは、その分だけ評価額が多く出る（ADR の A1 と同じ形の穴）。完了の確認の文で、精算日で問う「基準日の最後の精算を取り込みましたか」を確かめ（取込みが年明けでも、基準日の最後の精算がそろっていれば「はい」）、ADR の Consequences の受け入れた穴の列挙に足す（S1）。採らなかった案（D-167 の Alternatives）: 基準日に精算が無いとき次の最初の精算を基準日以前とみなす（休みの年に黙って少なく出る）、完了の窓で休みか締めの翌日かを選ばせる（記憶に頼る入力。TD-261 と逆向き）。

**O1 翌年の仕入れ（決定: 向きは E〈owner TD-898〉、O1-1 = (a)〈TD-929〉）**: この lane は基準日と O2 だけ。E（問屋の締め日から翌年の仕入れを分ける）は次の lane で、この lane の `base_date` の上に足す（O1-2〜O1-5 と店への問いはその lane。[backlog](../backlog.md) の「取引先の意味（D-075 (6)「suppliers = メーカー/ブランド、問屋は持たない」）の見直し」）。

- 判定: ready（owner の未決は無い。延期した design gap は S1 で決める）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| SPEC-STK-CNT-D6 評価数量 | ADR D6、35「完了」・§20.5 | S3 | T1〜T3 | L3-2 |
| SPEC-STK-CNT-D6 完了の条件 1 | ADR D6、35 の `settlement_pending` の表 | S3 | T5 | L3-1 |
| SPEC-STK-CNT-D6 初期在庫の確かめ（TD-934） | ADR D6、35「完了」、40、73、tracking | S2〜S7 | T12・T9・T10・T11 | L3-4 |
| 基準日の下限と 12/31 以外の確認（TD-935） | 73 UI-10-D4、35「完了」 | S3・S6 | T4・T10 | L3-5 |
| SPEC-STK-CNT-D6 完了の後の差 | ADR D6、35「完了」・§20.6a、65 | S3・S7 | T6・T7・T11 | L3-3 |
| SPEC-STK-VAL-D3（quantity） | 35 §20.5a | S3 | T1 | 非対象（unit） |
| SPEC-STK-VAL-D6 非遡及 | 35 §20.5a | 変えない | T7 | 非対象 |
| SPEC-STK-CNT-D3 窓・D7 取消（D-163 (6)） | ADR D3・D7 | 変えない（`judge` を使う） | T3、`stock_ledger.rs` の既存の test | 非対象 |
| SPEC-STK-CNT-D9 保存の形 | ADR D9、tracking §16-17、20 §2.11 | S2・S4 | T8 | 非対象 |
| MNT-03 の新しい節（D25 の見込み） | 22 | S2 | T8、連番の既存の test | 非対象 |
| CMD-10 `complete_stocktake`・`get_stocktake_record` | 42、cmd-task-specs | S5 | T9 | L3-2 |
| `completion_blocked` の hint | 40 | S3・S6 | T5・T9・T10 | L3-1 |
| UI-10-D4・§73.7 | 73 | S6 | T10 | L3-2 |
| D-167 | decision-log | S1 | — | 非対象（docs） |
| D-052 invalidation | UI_TECH_STACK | 変えない | 既存の oracle | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| D6 評価数量 | 基準日より後の入出庫・精算が評価に入る／基準日当日が落ちる | unit | T1 `test_complete_stocktake_req205_base_date_excludes_later_records` | 引き算が無い（M1）、境目が `>=`（M2） |
| D6 評価数量 | 基準日より後の計数（既定・fresh）で、間の記録を戻さない | unit | T2 `test_complete_stocktake_req205_base_date_back_calculates_through_later_count` | 計数に入っている記録を見ない |
| D3・D7 | 初期在庫・取消済みの記録・届く前に入れた記録（計数に入っていて日付が計数より後）を引く | unit | T3 `test_complete_stocktake_req205_base_date_keeps_initial_voided_and_prearrival` | 日付だけで全部を引く（M3） |
| 入力の検査 | 形の誤り・未来・開始日の前年の 12/31 より前の基準日で完了する（12/31 ちょうどと、年 2 回目の棚卸しの 12/31 より後の日は通る） | unit | T4 `test_complete_stocktake_req205_rejects_invalid_or_future_base_date` | 検査が無い・下限が無いか 1 日ずれる（M8）・TX の中で書いた後に拒む |
| 完了の条件 1 | 基準日までの精算が無い・欠けている・計数と基準日の間に欠けがあるのに完了する | unit | T5 `test_complete_stocktake_req205_base_date_settlement_pending` | 基準日を条件に当てない（M4）・区間の重なりを見ない・区間の検査を n で切る |
| 初期在庫の確かめ | 登録日が基準日より後の初期在庫の品を、選ばずに完了する・選択どおりに評価しない・選択を残さない・保存した選択が完了の後の差に効かない | unit | T12 `test_complete_stocktake_req205_initial_stock_after_base_date_requires_choice` | 検査が計数のある品だけ（M7）か計数の無い品だけ・「外す」で V から引かない・明細に書かない・差の計算に保存した選択を渡さない（M9） |
| 完了の後の差 | 完了の後に入れた基準日より後の記録が差に出る／基準日以前の記録が出ない／範囲がずれる | unit | T6 `test_get_stocktake_record_req206_after_completion_diff_at_base_date` | 差が基準日を見ない（M5） |
| VAL-D6 | 基準日が NULL の完了済みの行の詳細が変わる | unit | T7 `test_get_stocktake_record_req206_null_base_date_keeps_completion_values` | NULL を今日や完了日とみなす（M6） |
| D9・MNT-03 | 既存の行が変わる・空の DB で当たらない | unit | T8 `migration_v<N>_adds_stocktake_base_date` | 列の定義が違う・値を埋める |
| CMD-10 の wire | 基準日が wire に無い・拒否の wire が変わる | unit | T9 `test_complete_stocktake_cmd_req205_base_date_wire`（既存の `…_completion_blocked_wire` は AC6 の精算の fixture で `settlement_pending: null` の完全一致を保つ） | 引数名・hint の形が設計と違う |
| UI-10-D4 | 選ばずに確定できる・要求に入らない・未来を選べるか打てば押せる・下限より前を選べる・12/31 以外を確認なしで押せる・商品の無い hint（`targets` が空）が出ない・最後の未計数の品を「0 にする」と hint が消える・初期在庫の品を選ばずに押せる・基準日を変えても前の日の確認と選択が残る | component | T10 `StocktakePage.test.tsx` の基準日の test | 既定値を置く・`min`・`max` を外す・確認を見ない・`pending.length > 0` のまま・`handleZero` が `targets` だけで panel を消す・確認と選択を基準日に結ばない |
| 65 棚卸し詳細 | 基準日と「基準日の時点の在庫」と初期在庫の選択が出ない・`base_date` が NULL の行に「基準日の時点の在庫」の見出しか説明の文が出る | component | T11 `StocktakeRecordDetailPage.test.tsx` の基準日の test | 表示を足さない・見出しと説明の文を NULL で切り替えない（NULL の側は両方を「完了の時点」の文で照合する） |

入力（名指しの例）:

- T5: 計数 10（12/28、既定）、基準日 12/31、精算 #100（12/27）・#101（12/29）・#103（12/30）・#104（12/31）、欠けた #102（12/30）でこの商品が 3 個売れる。今の (ii) は計数日も基準日も挟まないが、区間 12/28〜12/31 と #102 の両隣 [12/29, 12/30] が重なるので #102 を名指して何も書かずに拒み、#102 を取り込んだ後は評価数量 7 で通る。ほかに 3 つの組: 商品の理由を全部満たしたまま基準日の条件だけが落ちる、`targets` が空で hint だけで拒む、基準日までを取り込み済みで通る。もう 1 つ、計数が基準日より後の組: 基準日 12/31、取り込み済みの #103（12/30）・#104（12/31）・#105（1/1）・#107（1/2）・#108（1/3）、欠けた #106（1/2）でこの商品が 3 個売れる、1/3 に fresh で 10 を数えて n = 107。k = 106 ≤ n でも区間 12/31〜1/3 と [1/1, 1/2] が重なるので #106 を名指して拒み（n で切ると評価数量 10 で通る）、取り込んだ後は 13 で通る。
- T6: 基準日 12/31、1/2 の計数 10 で完了し、完了の後に 1/2 の入庫 +4（計数の後に登録して「決まらない」）を入れる。基準日の時点の在庫の範囲は 6〜10（下が 6、上が 10）で、参考の差の上下もその範囲から求め、確定の差の合計に入れない。今の範囲の足し方を流用して 10〜14 を出す実装で red。
- T7: fixture に、完了日より後の日付でカーソルに入った記録（完了の前に登録した翌日の入庫 +4）を持つ旧記録を置く。NULL は今の経路（完了の時点の在庫）を通るので差は 0 のまま。NULL を完了日とみなして新しい関数を当てる（M6）と差 −4 が出て red。
- T10: 未来の値を手で打つと「確定する」を押せない。12/31 以外の日で確認を入れ、初期在庫の品を選んだ後に基準日を別の日に変えると、確認と選択が外れて「確定する」を押せず、答え直すと送る要求は新しい日の選択だけを持つ（同じ日のままの再試行では答えを使う）。未計数 1 件と商品の無い `settlement_pending` で拒まれた後、その品を「0 にする」と未計数の行だけが消え、「取り込んでいない精算がある」と直し方の文は残る。
- T11: `base_date` が NULL の行は、見出し「完了の時点の在庫」と説明の文（今の「完了の日より前の記録による差」）を一緒に照合し、どちらにも「基準日」が出ない。
- T12: 計数のある品と計数の無い品の両方で、選択が無ければ品を名指して何も書かずに拒み、「入れる」「外す」でそれぞれの評価数量になり、明細の `initial_stock_included` に残る。続けて、基準日 12/31・初期在庫 5・1/2 の計数 8 の品を「外す」で確定し（評価数量 3）、何も足さずに `get_stocktake_record` を読み直すと、明細の選択は「外す」で、参考の差は空（保存した選択を使わないと 8 と比べて +5 が出る）。

状態の遷移（T10）: 窓を開く（未選択で確定不可）→ 選ぶ → 送信中（押せない）→ 成功（結果に基準日）／`completion_blocked`（欄に理由、窓を開き直すと選び直し）／`validation`（文を出して一覧を読み直す。今の振舞い）→ 再試行。

mutation（実装の run で 1 つずつ注入して red を確かめ、戻す）: M1 評価数量を写しの在庫に戻す → T1・T2。M2 日付の比べを `>=` にする → T1。M3 計数の日付より後の「入っている」記録と初期在庫も引く → T3。M4 基準日の条件 1 を外す → T5。M5 `after_completion_diff` が基準日を渡さない → T6。M6 `base_date` が NULL の行を完了日とみなす → T7。M7 初期在庫の検査を計数のある品だけにする → T12。M8 下限を開始日の年の 12/31 にするか 1 日ずらす → T4。M9 `after_completion_diff` に保存した `initial_stock_included` を渡さない → T12。

## L3

DB は TD-892 の `park` で空の DB から始め、同期の後の head の build で作った DB だけを使う（前の試しの DB を使わない）。日付は L3 の日（今日）に合わせて作る: 基準日 = 昨日、計数日 = 一昨日。fixture は合成の Z004（layout A、CP932・CRLF、精算回数つき）を連番で 4 本で、2 つの確定に通しで使う: 1 本目 = 計数日の前日（既定の計数の (iii) を解く）、2 本目 = 計数日、3 本目 = 基準日（1 つ目の確定）、4 本目 = 今日（2 つ目の確定の計数日の (i) を解く）。どの精算も対象の商品を動かさない。商品と計数と入庫は画面から入れる。確認の窓では、基準日が 12/31 なら確認の check box は出ないのでそのまま「確定する」を押し、ほかの日なら check box を入れてから押す（以下「基準日を選んで押す」。元日の L3 は基準日が 12/31 になる）。

- L3-1 棚卸しの画面 / 商品 A を「初期在庫（必須）」0 で登録し（0 は初期在庫の記録を作らないので TD-934 の対象にならない）、「棚卸しを開始する」。A を「前の日の分を入れる」で計数日にして「数を保存」。1・2 本目だけを取り込み、「棚卸しを確定する」から基準日を選んで押す / 確定されず、「取り込んでいない精算がある」に直し方の文が出る（基準日の精算だけが無いので商品の名は出ない）。
- L3-2 棚卸しの画面 / 3 本目を取り込み、同じ棚卸しで「棚卸しを確定する」を押す / 確認の窓は基準日を選ぶまで「確定する」を押せず、今日より後は選べず、問いの文は「基準日の最後の精算を取り込みましたか」。基準日を選んで押すと確定し、結果に基準日が出て、総額が計数の数の額になっている。
- L3-3 棚卸しの記録の詳細 / 確定の後に、A に基準日当日の入庫 +2 と今日の入庫 +4 を登録する / 基準日が出て、「完了の後に入った記録による差」に +2 個の分だけが出る。
- L3-4 棚卸しの画面 / 商品 B を「初期在庫（必須）」5 で今日登録してから「棚卸しを開始する」。A と B を今日の日付で「数を保存」し、4 本目を取り込む。「棚卸しを確定する」から基準日を選んで押す / B が名指しされ、「基準日にもう店にあった」か「基準日の後に届いた」を選ぶまで「確定する」を押せない。選んで押すと確定し、記録の詳細に B の選択が出る（TD-934）。
- L3-5 確認の窓（L3-4 の確定の前に開いた窓） / 基準日に開始日の前年の 12/31 より前の日と、12/31 以外の日（元日なら今日）を選ぶ / 前者は選べない。後者は確認の check box を入れるまで「確定する」を押せず、入れた後に日付を変えると check box と B の選択が外れる（TD-935）。
