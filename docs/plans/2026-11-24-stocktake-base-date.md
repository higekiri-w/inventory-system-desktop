# Plan Packet: 棚卸しに基準日を持たせ、基準日より後の入出庫と精算を年末の評価額から外す（導入の前の関門 17）

起票: 2026-11-24（起点 `1edcedc3`）。出所は [backlog](../backlog.md) の「棚卸しの基準日の runtime」と [Plans](../Plans.md#導入の前の関門) の関門 17（owner の決定 2026-11-19、repo 外の回答台帳 TD-832）。decision-log の番号は D-167（予約済み）。O2 は (a) に決まり（TD-894）、O1 は向きが E（問屋の締め日から翌年の仕入れを自動で分ける、TD-898）に決まった。O1 の残りの事項（Design Readiness の O1-1〜O1-5 と店への問い）の答えの前は計画の review へ進めない。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「DB・migration」（`stocktakes` に NULL を許す列を 1 つ足す）、「Tauri command の DTO」（`complete_stocktake` の入力と記録の詳細の結果）、「状態を持つ画面・operator の業務の流れ」（完了の確認の窓）に当たる。R4 には当たらないと判断した: migration は `ADD COLUMN` だけで表を作り直さず既存の値を変えない（先例は v15 の R3〈[record-cancel](2026-11-20-record-cancel.md)〉。R4 にした v9・v11 は行のある表の作り直し）。変わる数字はこの版より後に完了する棚卸しの評価額だけで、完了済みの評価額は書き換えない（SPEC-STK-VAL-D6）。本番の DB は無い（[project-memory](../project-memory.md)）。O1 の E を同じ lane に入れても（O1-1）、足すのは新しい表 1 つ（`CREATE TABLE`。既存の表に触れない）と `ADD COLUMN` だけで、既存の行の値を変えないので R3 のまま。計画の review で表の作り直しが要ると分かれば R4 へ上げる。

## Goal

Goal Invariant:

### 最小完了条件

- 棚卸しの完了のときに基準日（店は大晦日）を選ぶと、評価額（仕入原価の総額）が基準日の終わりの時点の在庫で求まる。基準日より後の日付の入庫・返品交換・手動販売・廃棄・振替と、精算日が基準日より後の Z004 の売上は、完了の前に入っていても評価額を動かさない（完了が年明けにずれても、年明けの入荷と売上で総額が変わらない）。
- 棚卸しの結果と記録の詳細に基準日が出て、完了の後の参考の差は基準日の時点の在庫と比べる。

### 失敗定義

- 基準日より後の記録が評価額を動かす、または基準日以前の記録が評価から落ちる。
- 基準日を選ばないまま、または未来の日付で完了できる。
- 基準日までの精算を取り込んでいないことを知らせずに完了できる。
- 完了が在庫を動かす、完了済みの評価額を書き換える、今の在庫（写し）・要再確認・差異の値が今と変わる。
- この版より前に完了した棚卸し（基準日が NULL）の記録の詳細の値が変わる。

### 非目的

- 翌年の仕入れを問屋の締め日で分けて評価から外すこと（O1 の E。設計は Design Readiness。同じ lane に入れるかは O1-1 で、入れるなら答えの後に下書きを Scope 以下へ移す）。
- 完了した棚卸しの基準日の変更、完了の取消。
- 入出庫履歴の一覧の棚卸しの業務日付（今は完了日）と「前回の棚卸し」の表示を基準日にすること。
- 開始のときに基準日を決める入力、基準日の既定値（下の Design Readiness の「入力の形」）。

## Scope

基準日の時点の在庫の定義（S1 で ADR D6 と 35 に書く。導き方は Design Readiness）: 商品の効く計数 C（値 V、日付 T）について、V に「C に入っていない記録のうち日付が基準日以前のもの」を足し、「C に入っている記録のうち日付が基準日より後で T 以前のもの」を引く。初期在庫は足しも引きもしない（どの計数よりも前の申告。ADR D3）。計数の無い商品は、初期在庫と、日付が基準日以前の記録の和。日付は D3 の窓と同じ日付（入出庫は業務日、Z004 は精算日。`LedgerMovementRow.business_date`）。取消済みの記録（`is_voided`、D-163）は求め直しの入力に無いので、どちらにも入らない。

- S1 設計正本（実装より前の commit）: decision-log に D-167（末尾の追記）。ADR [stocktake-count-placement](../adr/2026-10-16-stocktake-count-placement.md) の SPEC-STK-CNT-D6（評価数量・完了の条件 1・完了の後の差・確認の文）・D9（列）・Consequences（O2 の答えの穴）。[35](../function-design/35-biz-stocktake-service.md) の「完了」・§20.5・§20.6a・§20.8 の INV-3 と SPEC-STK-VAL-D3 の quantity の定義。[42](../function-design/42-cmd-sales-stocktake.md) の `complete_stocktake` の入力と `get_stocktake_record` の結果、[40](../function-design/40-cmd-product.md) の `completion_blocked`（商品の無い `SettlementPendingHint`）、[20](../function-design/20-io-product-repo.md) §2.11 の `complete_stocktake` と §2.11a、[tracking](../db-design/tracking-system-tables.md) の「棚卸し」の節と stocktakes の列、[22](../function-design/22-mnt-migration.md) の §3.1 の一覧と新しい節（§24・MNT-03-D25 の見込み）、[73](../function-design/73-ui-stocktake.md) の冒頭の表の「完了」「完了の結果」・UI-10-D4・§73.7・§73.8・§73.12・§73.13、[65](../function-design/65-inventory-record-traceability.md) の D-118 の節の棚卸し詳細の項、`docs/architecture/cmd-task-specs.md` の `complete_stocktake` の行。
- S2 migration（`src-tauri/src/db/schema_v<N>.rs` を足し、`db/mod.rs` の `mod` の行と `migration.rs` の `use` と `migrations()` に登録。`MigrationKind::Sql`、v15 と同じ形）: `ALTER TABLE stocktakes ADD COLUMN base_date TEXT`（NULL を許す。既存の行は NULL のまま。止まる検査なし、作り直しの起点でない）。
- S3 BIZ（`src-tauri/src/biz/stocktake_service.rs`、`stock_ledger.rs`）: `CompleteStocktakeRequest` に `base_date`。形と暦の検査は `validate_count_date` の検査を使い、`today` より後は TX の前に `ValidationFailed`。完了の条件 1（`settlement_pending` の (i)〜(iii)）を、基準日を「日付が基準日の既定の計数」と同じに扱って当てる（商品に依らないので target を作らず、`targets` が空でも hint があれば拒む）。評価数量を `max(基準日の時点の在庫, 0)` にする。基準日の時点の在庫は `stock_ledger.rs` の窓の判定（`judge`）を使う 1 つの関数で求め、完了と `after_completion_diff` の両方が呼ぶ（今の在庫を求める `derive_ledger`・`recompute_stock` の結果は変えない）。`after_completion_diff` は今の範囲（`ledger_at_cutoff(…, Some(完了日))`）の入力に同じ関数を当てて `valuation_quantity` と比べ、要再確認の範囲の両端も同じ定義で求める。`base_date` が NULL の完了済みの行は今のまま（完了の時点の在庫）。`completed_stocktakes_comparing_settlement` は変えない（範囲による候補のまま。計数に入っている基準日より後の精算を取り消すと基準日の時点の在庫が変わるので、基準日で絞らない）。差異・ロス原価・操作ログの `difference_count` は変えない。
- S4 IO（`src-tauri/src/db/stocktake_repo.rs`）: `complete_stocktake` が `base_date` を書き、`get_stocktake_record_detail` がカーソルと一緒に `base_date` を読む（wire に出る `Stocktake` は変えない）。
- S5 CMD と生成物（`src-tauri/src/cmd/stocktake_cmd.rs`、`src/lib/bindings.ts`）: `complete_stocktake(stocktake_id, base_date)`。`StocktakeRecordDetail` に `base_date`、`AfterCompletionDiffItem` の `stock_at_completion`・`_low`・`_high` を `stock_at_base_date`…に改める（中身が基準日の時点の在庫になるため）。command は足さない（`lib.rs` は変えない）。
- S6 UI 完了（`src/features/stocktake/StocktakePage.tsx`、`hooks/useCompleteStocktake.ts`）: 完了の確認の窓に基準日の入力（「0 にする」の日付の入力と同じ `type="date"`、今日より後は選べない。既定値なしで、選ぶまで「確定する」を押せない）と「基準日より後の入出庫と精算は評価額に入りません」の文。最後の精算の問いの文を基準日の言い方に直す。結果の画面に基準日。商品の無い hint の拒否を「取り込んでいない精算がある」の欄に直し方の文だけで出す。
- S7 UI 記録の詳細（`src/features/inventory-records/StocktakeRecordDetailPage.tsx`）: 基準日の行、参考の差の列と説明の文を「基準日の時点の在庫」に。
- S8 docs の片付け（実装の PR に入れる）: `docs/Plans.md` の関門 17 の行を外す。`docs/backlog.md` の「棚卸しの基準日の runtime」を済みの形に（TD-765 の B）。`docs/ACCEPTANCE_SCENARIO.md` の Step 4 の確定の手順に基準日を選ぶ操作を足す。
- 呼出し側・隣接 test: `CompleteStocktakeRequest { … }` を作る所（`rg 'CompleteStocktakeRequest \{' src-tauri` で `stocktake_cmd.rs`・`stocktake_service.rs` の test・`inventory_service/cancel.rs` の test・`csv_import_service/tests/cross_feature_tests.rs`・`commit_tests.rs`）に `base_date` を足す（既存の test は基準日を完了日にする。完了の時点に完了日より後の日付の記録を持つ test〈例: `test_get_stocktake_record_req206_after_completion_same_range_as_valuation`〉で期待が変わるものは、D6 の改訂を理由に名指して直す。弱めない）。frontend は `StocktakePage.test.tsx`・`StocktakePage.suggest.test.tsx`・`StocktakeRecordDetailPage.test.tsx`。
- 生成物: `src/lib/bindings.ts`、`docs/function-design/90-traceability.md`（どちらも同期の後に作り直し、手で解かない）。

migration の版: 実装の時点の `origin/main` の最大 + 1（22 §21 の規則。2026-11-26 の同期の時点の main の最大は v15〈`ls src-tauri/src/db/schema_v*.rs`〉で、PR #307〈PLU の runtime〉が v16 を取る見込み。先に merge する側が小さい番号を取り、後の側が file 名・`mod` の行・登録・22 の節と ID を取り直す）。PR #307 も migration を足すので、2 本のうち後に merge する側は、先の側の merge と同期の後に L3 をする（同期で `migrations()` の登録の列を手で解くと lane の手が runtime に及び、L3 のやり直しになる。[merge-evidence](../agent-guidance/merge-evidence.md) の MG-D13）。

並走の lane と重なる file の所有（Wave Operation。ほかの file は重ならない）:

| file | 本 lane が持つ行・節 | 重なる lane |
|---|---|---|
| `src-tauri/src/db/migration.rs`・`db/mod.rs`・`schema_v<N>.rs` | 本 lane の `mod`・`use`・`migrations()` の 1 件と自分の schema の file | PLU の runtime（上の「migration の版」） |
| `docs/function-design/22-mnt-migration.md` | §3.1 の自分の 1 行と自分の節 | PLU の runtime |
| `docs/decision-log.md` | 末尾の D-167（merge 順で両方を残す） | D-164・D-166・D-168・D-169 の lane |
| `src/lib/bindings.ts`・`docs/function-design/90-traceability.md` | 生成物（取り込んだ後に作り直す） | PLU の runtime・関門 19・PR #304 |
| `src-tauri/src/biz/csv_import_service/tests/commit_tests.rs`・`cross_feature_tests.rs` | `CompleteStocktakeRequest` の `base_date` の行だけ | PR #304（JAN-8 の integration test を足す） |
| `docs/Plans.md`・`docs/backlog.md` | 関門 17 の行と「棚卸しの基準日の runtime」の項目 | 関門 14・19 の行、ほかの項目 |

Non-scope: 今の在庫の求め直し（`derive_ledger`・`recompute_stock`・整合性の検査）、計数の保存、Z004 の確定と取消、入出庫の記録の取消（D-163）、入出庫履歴の一覧の SQL、D-052 の invalidation（query key を足さない）、操作ログの文。

## Acceptance Criteria

- AC1: `cd src-tauri && cargo test --lib` で Test Plan の T1〜T3 が PASS（基準日より後の記録を外し、基準日当日以前を残し、基準日より後の計数からさかのぼり、初期在庫・取消済み・届く前に入れた記録を引かない）。
- AC2: 基準日の形の誤り・未来の日付・基準日までの精算の未取込みは、何も書かずに `ValidationFailed` か `completion_blocked` で拒む（T4・T5。header・明細・在庫が変わらない）。
- AC3: 完了の後の参考の差が基準日の時点の在庫と比べ、基準日が NULL の完了済みの行は今の値のまま（`get_stocktake_record` の T6・T7）。
- AC4: migration が空の DB と行のある直前の版の DB に当たり、既存の行の値を変えない（T8）。`migrations()` の連番の既存の test が PASS。
- AC5: 確認の窓は基準日を選ぶまで確定できず、送る要求に基準日が入り、結果と記録の詳細に基準日が出る（T9 と、`npm test` の T10・T11）。
- AC6: 既存の test は、`base_date` の引数・field の追加と S5 の field 名の変更の他は期待値を変えずに全部 PASS し、期待が変わるのは完了の時点に基準日より後の日付の記録を持つ test だけで、PR の差分で名指す（`cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`）。`stock_ledger.rs` の既存の test は 1 行も変えない。
- AC7: `cargo run --bin generate_bindings` の後の `src/lib/bindings.ts` の差分が S5 の 3 点だけ。`cargo run --bin generate_traceability -- --check` と `bash scripts/doc-consistency-check.sh`・`--target plan` が ERROR なし。
- AC8: Test Plan の mutation（M1〜M5）を 1 つずつ注入すると名指しの test が `cargo test --lib -- <test 名>` で exit 101（red）になり、戻すと exit 0。
- AC9: 「完了の時点の在庫」で評価する旧い文が、`docs/archive`・merge 済みの packet の外の設計正本に残らない（`rg -n 'max\((その時点の|写しの|完了の時点の写しの)在庫' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が、基準日が NULL の行の説明だけを返す）。

## Design Readiness

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D3（窓と判定表・A1）・D6・D7・D9・Consequences、35 の「完了」・§20.5・§20.5a（SPEC-STK-VAL-D1〜D6）・§20.6a、65 §65.5b と D-163 (6)、tracking の「棚卸し」と §16-17、22 §21・§23、73 UI-10-D4・§73.7、project-memory の Store Premises Facts の棚卸しの行。
- 必要な設計成果物: S1 の正本を updated in this PR。新しい決定は D-167。
- plan にしかない durable な判断の昇格先: 基準日の時点の在庫の定義と完了の条件は ADR D6 と 35、入力の形は 73 UI-10-D4、O1・O2 の答えは D-167 と ADR の Consequences。
- 導き方（起票で手で導いた。code では未実測。計画の review の中心の問い）: 今の在庫は V + Σ（C に入っていない記録）。基準日の終わりで切ると、C より後で基準日以前の記録は残り、C より前で基準日より後の記録（C が基準日より後のときの、間の記録）は戻す。要再確認の範囲は、決まらない記録を入っている・入っていないのそれぞれとして同じ定義を当てた最小と最大（完了は要再確認 0 を求めるので、範囲が出るのは完了の後の差だけ）。
- 入力の形（Writer の設計。owner が変えたければ計画の review の前に直す）: 基準日は完了の確認の窓で選ぶ。棚卸しは基準日より前に始まり、基準日が効くのは完了の TX の評価だけなので、開始に手順を足さない。既定値を置かないのは、年明けに完了するときに既定の「今日」のまま確定すると、この lane が防ぐ失敗（年明けの記録が評価に入る）がそのまま起き、確定は取り消せないため。
- 届いた日との関係: 関門 16 は列を足さずに閉じた（D-170。入庫の日付の欄が「届いた日」）。基準日の判定は D3 の窓と同じ日付（`LedgerMovementRow.business_date`。入庫は届いた日）だけを読む。届く前に入れた入庫（計数に入っていて日付が計数の日付より後）は、今の在庫と同じく足しも引きもしない。届いた後に在庫が実物より少なくなる穴（ADR の Consequences、D-170 が残した）は基準日の時点の在庫にも同じ形で残る。
- 絶対保証の例外と抜け道: 「基準日より後の記録は評価に入らない」は記録の日付を信じる（日付を基準日以前にして入れた記録は入る）。初期在庫は登録日が基準日より後でも入る（棚卸し中に見つけて登録した品を評価に入れるため。年明けに初めて入った品を初期在庫つきで完了の前に登録すると評価に入る）。精算の境目は O2 の (a)。
- 延期した design gap: 商品の無い `SettlementPendingHint` の wire と画面の形（40・73。S1 で決める）。

### owner に決めてもらう事項

**O2 基準日と精算の境目（決定: (a)、owner TD-894）**: Z004 は精算の中の売上の日時を持たないので、「精算日が基準日より後の精算は基準日より後」とみなす。基準日に精算が無く次の精算が基準日の売上を含むときは、その分だけ評価額が多く出る（ADR の A1 と同じ形の穴）。完了の確認の文で「基準日の締めをその日のうちに取り込んだか」を確かめ、ADR の Consequences の受け入れた穴の列挙に足す（S1）。採らなかった案（D-167 の Alternatives）: 基準日に精算が無いとき次の最初の精算を基準日以前とみなす（休みの年に黙って少なく出る）、完了の窓で休みか締めの翌日かを選ばせる（記憶に頼る入力。TD-261 と逆向き）。

**O1 翌年の仕入れ（向きは決定: E、owner TD-898）**: 問屋の締め日から翌年の仕入れを自動で分け、年末の評価額から外す。問屋と、問屋と取引するメーカーの対応表は店が作る（TD-900）。翌年の分を棚卸しで数えないことは前から既知（TD-053）。

店の事実（repo 外の回答台帳 TD-916・917・926 の要旨）: 締め日は 20 日と月末、12 月も変わらない。見分けはどの問屋も伝票の日付（届いた日でない）。

| 問屋の形 | 年内の境（この日付までの伝票が年内） | 未確かめ |
|---|---|---|
| 20 日締め（例: 内藤。21 日〜翌月 20 日の分の伝票が 21 日以降に来て同月末払い） | 12/20 | — |
| エトワール海渡（12/1 の発注から月末までの 1 か月分を翌月に請求する同社の後払い） | 11/30 | 店の答えは「発注の日」で、伝票の日付と同じとみなしてよいか（店への問い 1） |
| 月末締め（上の他） | 12/31（基準日まで全部年内） | 答えはエトワール海渡についてだけ（店への問い 2） |
| 現金・前払い（締め日なし） | 全部年内（領収書の日付。棚卸しで数える） | — |

E の設計（Writer が決めたこと）:

- 年内の境は問屋ごとに月日を 1 つ持つ（NULL = 基準日まで全部年内）。締めの種類を型で持たない（上の 4 つの形がどれも 1 つの月日で表せ、判定が比べ 1 回で済む）。境は基準日の年の月日として読む。
- 翌年の分: 取り消していない入庫の明細のうち、届いた日が基準日以前で、伝票の日付（O1-3）がその明細の問屋（O1-2）の境より後のもの。届いた日が基準日より後の入庫は基準日で既に外れるので二重に引かない。
- 評価数量 = `max(基準日の時点の在庫 − 翌年の分の数量の和, 0)`（O1-5）。評価原価・§20.5a の関数・差異・ロス原価は変えない。翌年の分の数量は完了の TX で求めて明細に残し、記録の詳細に出す（完了の後に入庫を取り消しても、凍結した評価額は変えない。SPEC-STK-VAL-D6）。
- 問屋の決まらない入庫（メーカーか対応が無い）は引かず、届いた日がいちばん早い境の翌日から基準日までのものを、完了の確認の窓に件数と商品で名指す（黙って評価に入れない。AGENTS.md「使い手に押しつけない」）。

owner に決めてもらう事項（推奨は confirmed / candidate / precondition-dependent）:

| 事項 | 選択肢 | 推奨 |
|---|---|---|
| O1-1 lane の分け方 | (a) この lane は基準日と O2 だけ、E は次の lane（D-075 (6) の見直しの項目と合わせて起票し、この lane の `base_date` の上に足す）。(b) E もこの lane に入れる（下の下書きを Scope 以下へ移す。同じ版の migration） | candidate: (a)。基準日だけで年明けの入出庫が評価から外れ、E は新しい表・取引先の管理の画面・入庫の画面・評価に及び、review と L3 が倍になる。precondition: 次の年末の棚卸し（2026-12-31）に E が要るなら (b) か、(a) の 2 本を続けて入れる（導入の時期は Plans の関門が持ち、ここでは決めない） |
| O1-2 問屋の置き場 | (a) 新しい表 `wholesalers`（名前・年内の境）と、メーカーから問屋への対応 `suppliers.wholesaler_id`（`ADD COLUMN`）。明細の問屋は商品のメーカー（`products.supplier_id`）から引き、入庫の画面に問屋の操作を足さない。(b) (a) に加え、入庫の 1 回ごとに `receiving_records.wholesaler_id` を持ち、入庫の画面で選ぶ（既定は見出しのメーカーの対応）。(c) `suppliers` に問屋も入れて種別の列で分ける | precondition-dependent: 1 つのメーカーの品をいつも同じ問屋から仕入れるなら (a)（TD-900 の対応表がそのまま効き、入庫の手間が増えない。緊急の別の問屋からの仕入れ〈backlog の D-075 (6) の見直し〉だけ外れる）。メーカーが問屋で分かれるか、入庫ごとに問屋を残したい（店主の答え Q-104 の a、TD-761）なら (b)。(c) は D-075 (6) の「メーカー/ブランド」の意味を変え、メーカーと問屋が同じ一覧に混ざるので採らない。どれも D-075 (6) の「問屋は持たない」を改める |
| O1-3 伝票の日付 | (a) 入庫に「伝票の日付」の欄を足す（`receiving_records.slip_date`、NULL を許す。画面の既定は届いた日で、違うときだけ直す。NULL の行は届いた日で読む）。(b) 届いた日で代える（列を足さない） | candidate: (a)。(b) は伝票の日付が境以前で届いたのが境の後の分（20 日締めで 12/20 以前の伝票が 12/21 以降に届く等）を翌年とみなし、評価額が少なく出る（伝票の日付は届いた日以前なので逆には外れない）。(a) も既定のまま直さなければ (b) と同じ誤差になる |
| O1-4 対応表の置き場 | (a) アプリの取引先の管理の画面（UI-15、[78](../function-design/78-ui-supplier-management.md)）で、問屋の追加と境、メーカーごとの問屋を入れる。(b) 店が作った表（Excel）を取り込む | candidate: (a)。取込みの形式・検査・誤りの直し方を足さずに済み、間違いをその画面で直せる。店が既に表を持っていて行が多いなら (b) を後から足せる |
| O1-5 引き算が在庫を超えるとき | (a) 0 で止め、当たった商品を完了の確認の窓に名指す。(b) 0 で止めるだけ（前の案 B と同じ近似） | candidate: (a)。翌年の分の品が年内に売れた分だけ評価額が少なく出る近似は同じで、(a) は黙らない。答えを ADR の Consequences の受け入れた穴に足す |

店への問い（owner が店に聞く。台帳の未回答の問いに置く）: 1「エトワール海渡の伝票の日付は、発注の日と同じですか（11 月の終わりに頼んで 12 月に届いた品の伝票は 11 月の日付ですか）」。2「月末締めの他の問屋は、12 月 31 日の伝票まで年内の仕入れですか」。3「1 つのメーカーの品は、いつも同じ問屋から仕入れますか」（O1-2 の前提）。

E の下書き（O1-1 が (b) なら答えの形で Scope 以下へ移す。(a) なら次の lane の起票の入力）:

- S: 設計正本（decision-log の D-075 (6) の改訂、master-tables の `suppliers` と新しい `wholesalers` の節、transaction-tables の `receiving_records`、ADR D6 と Consequences、35、61・78、[project-memory](../project-memory.md) の Store Premises Facts に締めの事実）。migration（`CREATE TABLE wholesalers` と、`suppliers`・`receiving_records`〈O1-3 の (a)〉・`stocktake_items`〈翌年の分の数量〉への `ADD COLUMN` だけ）。BIZ・IO・CMD（問屋の作成・改名・境、メーカーへの割当て、入庫の伝票の日付、完了の評価の引き算と名指し、`list_suppliers` の DTO）。UI（UI-15 の問屋、UI-02 の伝票の日付、完了の確認の窓の名指し、記録の詳細の翌年の分）。
- AC・T: 境の当日は年内・翌日は翌年、境が NULL は全部年内、取消済み・届いた日が基準日より後は引かない、伝票の日付が NULL は届いた日、0 で止めて名指す、問屋の決まらない入庫の名指し、完了の後の取消で凍結の値が変わらない。mutation: 境の比べを `>=` にする、取消済みも引く、届いた日を見ずに引く。
- L3: 取引先の管理で問屋と境（20 日締め）を入れてメーカーに割り当て、伝票の日付 12/21 の入庫を入れて完了すると、総額がその分を外し、確認の窓と記録の詳細に翌年の分が出る。

- 判定: not ready（O1-1〜O1-5 と店への問い 1〜3 の答えの後に、計画の review の前提として ready）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| SPEC-STK-CNT-D6 評価数量 | ADR D6、35「完了」・§20.5 | S3 | T1〜T3 | L3-2 |
| SPEC-STK-CNT-D6 完了の条件 1 | ADR D6、35 の `settlement_pending` の表 | S3 | T5 | L3-1 |
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
| 入力の検査 | 形の誤り・未来の基準日で完了する | unit | T4 `test_complete_stocktake_req205_rejects_invalid_or_future_base_date` | 検査が無い・TX の中で書いた後に拒む |
| 完了の条件 1 | 基準日までの精算が無い・欠けているのに完了する | unit | T5 `test_complete_stocktake_req205_base_date_settlement_pending` | 基準日を条件に当てない（M4） |
| 完了の後の差 | 完了の後に入れた基準日より後の記録が差に出る／基準日以前の記録が出ない／範囲がずれる | unit | T6 `test_get_stocktake_record_req206_after_completion_diff_at_base_date` | 差が基準日を見ない（M5） |
| VAL-D6 | 基準日が NULL の完了済みの行の詳細が変わる | unit | T7 `test_get_stocktake_record_req206_null_base_date_keeps_completion_values` | NULL を今日や完了日とみなす |
| D9・MNT-03 | 既存の行が変わる・空の DB で当たらない | unit | T8 `migration_v<N>_adds_stocktake_base_date` | 列の定義が違う・値を埋める |
| CMD-10 の wire | 基準日が wire に無い・拒否の wire が変わる | unit | T9 `test_complete_stocktake_cmd_req205_base_date_wire`（既存の `…_completion_blocked_wire` は引数を足すだけ） | 引数名・hint の形が設計と違う |
| UI-10-D4 | 選ばずに確定できる・要求に入らない・未来を選べる・商品の無い hint が出ない | component | T10 `StocktakePage.test.tsx` の基準日の test | 既定値を置く・`max` を外す |
| 65 棚卸し詳細 | 基準日と「基準日の時点の在庫」が出ない | component | T11 `StocktakeRecordDetailPage.test.tsx` の基準日の test | 表示を足さない |

状態の遷移（T10）: 窓を開く（未選択で確定不可）→ 選ぶ → 送信中（押せない）→ 成功（結果に基準日）／`completion_blocked`（欄に理由、窓を開き直すと選び直し）／`validation`（文を出して一覧を読み直す。今の振舞い）→ 再試行。

mutation（実装の run で 1 つずつ注入して red を確かめ、戻す）: M1 評価数量を写しの在庫に戻す → T1・T2。M2 日付の比べを `>=` にする → T1。M3 計数の日付より後の「入っている」記録と初期在庫も引く → T3。M4 基準日の条件 1 を外す → T5。M5 `after_completion_diff` が基準日を渡さない → T6。

## L3

fixture は合成の Z004（layout A、CP932・CRLF、精算回数つき）を 2 本（精算日が基準日より前と基準日。基準日は L3 の日の前の日にするので、日付は L3 の日に合わせて作る）。商品と計数と入庫は画面から入れる。

- L3-1 棚卸しの画面 / 商品を数えて保存し、基準日より後の日付の入庫を入れ、基準日の精算を取り込む前に、基準日を選んで確定する / 「取り込んでいない精算がある」に直し方の文が出て、取り込んだ後は確定できる。
- L3-2 棚卸しの画面 / L3-1 の続きで「棚卸しを確定する」を押す / 確認の窓は基準日を選ぶまで「確定する」を押せず、今日より後は選べない。選んで確定すると、結果に基準日が出て、総額が後の入庫を含まない額になっている。
- L3-3 棚卸しの記録の詳細 / 完了の後に、基準日以前の日付の入庫と基準日より後の日付の入庫を 1 つずつ入れる / 基準日が出て、参考の差に前者だけが出る。
