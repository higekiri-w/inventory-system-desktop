# Test Design Matrix: 棚卸しの数の置き場と在庫数の求め直しの runtime (b1)

Plan Packet: [2026-10-22-stocktake-count-runtime-b1](../2026-10-22-stocktake-count-runtime-b1.md)。期待値は設計正本（ADR SPEC-STK-CNT-D1〜D12、22 §20、24・30・31・32・35・36・40・41・42・51・55・65・73・75 の D-118 の節、共通規則）から導き、実装や mock から写さない。test の名前は REQ を含む（`docs/DOC_STYLE_GUIDE.md` §1）。横断の約束の # は packet の「横断の約束」（Z1〜Z20）。

## Risk

Risk: R4

## Contracts Under Test

- vZ（22 §20）: 止まる検査・全行と id の写し・`sqlite_sequence`・Z004 と phase を結ぶ CHECK・列の追加と CHECK（packet の横断の約束の Z17）、作り直しの後の失敗が全部戻ること（M8・M9。round 1 の是正）。
- round 1 の是正で足した契約: 要再確認の対象の範囲と単位、完了の拒否の直し方（packet の横断の約束の Z18）、preview の cache の照合（同 Z19）、計数の欄の文脈（同 Z20）、共有 JAN の経路（update は在庫連動だけ、JAN は一括 import の上書き）、L3 の fixture の生成（F1）。
- round 2 の是正で足した契約: 商品一括 import の判定の状態（同じ CSV の先の新規の行を当てる）と確定の TX の拒否の形（行を名指して確定全体を戻す）と在庫連動を始める拒否の順（30）、`zero_uncounted_items` の明細の参照（35）、開始と完了の操作ログの detail と summary の差異の件数（35）、要再確認の一覧の key と再取得の経路と entry（UI_TECH_STACK の D-118 の段落）、demo の seed の reset（E1）、STK-1・STK-2 と対照の trace を日付と精算回数で書き直したこと（X1〜X3）。
- round 3 の是正で足した契約: 32 の順 2 は取り消した取込みとも machine_no を比べる、fresh な計数の n は n の active な取込みがあるときだけ使う（ADR D3。Codex の closure の Gated Amendment で取込みのカーソルの形に置き換えた）、完了の後の比較の範囲（完了の時点にあった記録〈計数・movement・精算のカーソル以下〉に完了の後に入れた完了の前の記録を足す。ADR D6・35。計数を除くことは下の closure の是正で明記した）と `cutoff_movement_id`、商品フォームの「今の数を入れる」の導線が離脱の確認を通ることと `/stocktake/` の search の `product`・`returnTo`（51・73）、開始の操作ログの detail の key。
- Plan Review closure の是正で足した契約: 32 の順 2 で取り消した取込みとだけ合わないときは拒否の文でその取込み（取込みID・精算回数・取込み日・レジ番号）を名指し、machine_no の違う active な取込みがあるときは今の文（Z2・Z22。24 の照合先は active を優先）、完了の後の比較の範囲から完了の後に保存した計数を除く（R2 の (d)。ADR D6・35）、完了の (iii) の n は n の active な取込みがあるときだけ使う（K2 の列。正本は round 3 のまま。Codex の closure の Gated Amendment で取込みのカーソルの形に置き換え、K2 の列を書き直した）。
- Codex の Plan Review closure（P1）の Gated Amendment（2026-10-23）で足した・置き換えた契約: fresh な計数の判定の n と完了の時点の精算の n は、取込みのカーソル（`stock_counts.seen_import_id`・`stocktakes.cutoff_import_id`。取り消したものを含む `MAX(csv_imports.id)`）以下の取込みID を持つ今 active な取込みの最大の精算回数（ADR D3・D6。round 3 の「n の active な取込みがあるときだけ」と `cutoff_settlement_no` を置き換えた。L13・C2・K1・K2・R2 の (e)・R6・Z24・J7・M1・M6）、取り込み直した取込みは保存の後の取込みとして判定の n の算出に入らず、取り込み直した精算は D3 の表のとおり k ≤ n なら入り、判定の n だった精算を取り込み直したときに日付で判定する（ADR D7。owner の決定 TD-400。Gated Amendment の 2 本目で D3 の表に合わせた。L13・K2・Z24）。
- 精算の窓（ADR D3 の「精算 k」の行、A1、理由 1）と求め直す商品（ADR の横断の表）、初期在庫の見分け方（ADR D9）。
- 計数の保存（既定・fresh・0 にする）の検査と保存と求め直し、後入力の確認、要再確認の一覧（ADR D5、35）。
- 完了の拒否の 3 つの理由・評価額・カーソル・総額の範囲、記録の詳細と完了の後の差（ADR D6、35、65）。
- Z004 の preview の拒否と知らせ、確定の取り直し・引き当ての食い違い・`stock_linked`・整数の数量の換算・共有 JAN の行・全て 0 の file、取消と取り込み直し（ADR D4・D7、32、24）。
- 共有 JAN の不変条件・在庫連動を始める検査・共有 JAN の一覧（ADR D4・D8・D10、30、36）。
- wire（`stocktake_guard` と回復型、新設 4 と外す command、生成の enum）と画面（73・55・51・65・75、UI_TECH_STACK の再取得）。
- 撤去（停止・旧本体・時点証拠・probe）と、STK-1・STK-2 の診断の新しい経路での PASS。

## Failure Modes

- 5 つの入口の部分的な解除、停止の文言や旧本体・時点証拠の code の残り。
- 精算の窓の判定の誤り（`d_k = D` を入るとする、A1 を当てない、`k ≤ n` の fresh の比べ方の誤り、欠けで「最初の 1 本」を誤る）、理由 1 が今の `pos_stock_sync` を見る。
- 求め直す商品の漏れ（計数を持つが取込みに無い商品、連動の off の商品の取込みの行）。
- 計数の保存が未来の日付・形の違う日付・上限の外・別の商品の明細を通す、fresh のカーソルを TX の外で読む。
- 完了が理由を無視して進む・在庫を動かす・総額の範囲の外を保存する。完了の後に評価額が変わる。
- Z004 の拒否の漏れ（精算回数の無い file・同じ精算の別の版・順の食い違い）、preview と違う商品へ確定が入れる、長さの商品が 100 分の 1、共有 JAN の行が在庫を動かす、`stock_linked` が今の設定を写す。
- 取消が補償の movement を作る・計数を消す・求め直さない。取消で比べる先が無くなった後に、別のレジの Z004 が通る（round 3 の是正）。fresh な計数と完了の判定の n が、保存・完了の後の取込み（取消の後に同じ番号を使う取り込み直し・初期化の後の新しい系列）の精算回数から求まる（Codex の closure の Gated Amendment）。
- 完了の評価に入った記録（未来の日付の精算と入出庫）が完了の後の比較から外れ、後の記録が無いのに差が出る（round 3 の是正）。
- 共有 JAN の不変条件の経路の漏れ（どちらの商品を変える場合か）、在庫連動を始める検査が「決まらない」売上を通す。
- vZ が行・id・列・index を欠く、`sqlite_sequence` を戻す、止まる DB で何かを変える。
- wire の number が安全な整数を越える値を運ぶ、未知の code を画面が「そのまま続行」にする。
- 既存の test を契約の置き換えの根拠なしに消す・弱める。

## Test Matrix

- Before citing an existing test as regression coverage, use `rg` or an equivalent repository search to verify that the cited test exists.
- helper と mock の実装を読み、実際に通る境界と置換される境界を確認して Test Type / coverage を選ぶ。helper 名だけで実 router / integration と分類しない。
- `Would fail if...` は壊れる振舞いを観測できる入力・経路と結びつける。
- oracle の anchor は定義文にしか現れない literal を選び、`rg -c` で対象 file 内 1 件を file ごとに確かめる（複数 file を 1 本の `rg` で数えない）。文言を改訂した後は「新しい文言が exact で 1 件 + 旧い文言が 0 件」の対にする。
- oracle は検証対象と独立の正本から転記し、mutation は production 側だけを変える。
- 結果を空にする注入で、期待が空集合の case だけが kill を主張していないかを確かめ、各組合せに非空の期待を 1 件置く。

### M: migration vZ（22 §20、横断の約束の Z17）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| M1 | 空の DB で vZ が当たり、22 §20 の手順 5 の 8 つの列（`csv_imports.machine_no`・`csv_imports.settlement_no`、`sale_records.stock_linked`、`stocktake_items.valuation_quantity`、`stocktakes.cutoff_import_id`・`stocktakes.cutoff_count_id`・`stocktakes.cutoff_movement_id`、`stock_counts.seen_import_id`）と Z004 と phase を結ぶ CHECK と 3 つの index がある（round 3 の是正で列を名指し、`cutoff_movement_id` を足した。Codex の closure の Gated Amendment で `cutoff_settlement_no` を `cutoff_import_id` に置き換え、`seen_import_id` を足した） | 列・CHECK・index の欠け | schema | `test_vz_req903_fresh_db_has_columns_checks_and_indexes` | 手順 3〜5 のどれかを落とす、index を作り直さない |
| M2 | 直前の版の DB（入庫・返品交換・手動販売・廃棄・初期在庫と demo の seed の形の `sale_auto`〈`reference_type` NULL〉の movement、末尾の行を消した履歴）で全列と id が同じ、`sqlite_sequence` が前の値 | 行・値の欠け、id の使い回し | schema / regression | `test_vz_req903_rebuild_keeps_rows_ids_and_sequence` | 写しの列の漏れ、手順 4 の `sqlite_sequence` の戻しを落とす（次の INSERT の id が前の最大 + 1 になる） |
| M3 | 止まる検査（`stocktakes` の行、`csv_imports` の行、`reference_type = 'csv_import'` の movement、`phase = 1` の movement。1 つずつ）で schema・値・版が何も変わらず `MigrationFailed`（`作り直し` と表名） | 止まる DB を変える、原因の文が無い | schema / negative | `test_vz_req903_stops_without_changes_on_stocktake_or_z004_rows` | 条件の 1 つを落とす（その DB で vZ が通り版が進む）、`csv_imports` の行の検査を手順 5 の列の追加の後へ動かす（列の CHECK の失敗〈`CHECK constraint failed`〉で止まり、message に `作り直し` と表名が出ない）、message に表名が無い。検査を書込みの後へ動かしても TX が戻れば最終の状態は同じなので、最終の状態の比較ではなく message で見分ける（round 1 の是正） |
| M4 | vZ の後、`csv_import`・`phase = 0`、`receiving_record`・`phase = 1`・`settlement_no` 5、`reference_type` NULL・`phase = 1`・`settlement_no` 5 の INSERT が CHECK で拒まれ、`csv_import`・`phase = 1`・`settlement_no` 0 と 7 は入る（Contract Probe P3。NULL・`phase = 1` の行だけが `=` と `IS` の違いを見分ける。round 1 の是正） | CHECK の漏れ・`=` で比べる誤り | schema / negative | `test_vz_req401_reference_type_and_phase_check` | CHECK を `=` で書く（NULL で通る）、CHECK を落とす |
| M5 | `csv_imports` の `machine_no` が NULL・空、`settlement_no` が NULL・負の INSERT が CHECK で拒まれ、`('0001', 42)` は入る（Contract Probe P1） | 列の CHECK の漏れ | schema / negative | `test_vz_req401_csv_imports_settlement_key_checks` | 列の CHECK を落とす、`machine_no <> ''` を落とす |
| M6 | `sale_records` の `stock_linked` の既定 0・2 の拒否、`stocktake_items` の `valuation_quantity`・`stocktakes` の 3 つのカーソル（`cutoff_import_id`・`cutoff_count_id`・`cutoff_movement_id`）が NULL を許す。`stock_counts.seen_import_id` は fresh の NULL と 1 以上を受け、default の行の値と 0 を拒み、直前の版で fresh と default の行がある `stock_counts` にも足せて既存の行は NULL（Codex の closure の Gated Amendment。packet の Contract Probe P4） | 既定・CHECK の誤り | schema | `test_vz_req205_stock_linked_default_and_completion_columns` | 既定を 1 にする、CHECK を落とす、カーソルを NOT NULL にする、`seen_import_id` の CHECK を落とす（default の行の値か 0 が入る）、`count_kind` の条件を落とす（default の行の値が入る）、`seen_import_id` を NOT NULL にする（行のある表で列の追加が失敗する） |
| M7 | 再実行で重複適用しない。最新の版までの migrate の後に §18 の trigger があれば残る（§18 が先に入った版で） | 二重適用・trigger の消失 | schema | `test_vz_req903_idempotent_and_keeps_jan_alias_triggers_when_present` | 版の記録を落とす（2 回目の migrate で vZ が再び走り、手順 5 の `ADD COLUMN` が `duplicate column name` で失敗する）。trigger の項は §18 が vZ より先に merge した版でだけ観測でき、§18 の無い版では trigger が無いので kill を主張しない（§18 は `products` と別名の表の規則で、vZ の作り直しは `inventory_movements` だけなので、当たらないことの確かめ。round 2 の是正） |
| M8 | 止まる検査を通った後、手順 3〜5（`DROP`・改名・列の追加）を終えてからの COMMIT の失敗（`migration_tx::fail_operations(&[FailurePoint::Commit])`。v9 の `test_migration_req903_vl_commit_failure_rolls_back_everything` と同じ注入）で、`sqlite_master` の `inventory_movements`・`csv_imports`・`sale_records`・`stocktakes`・`stocktake_items` の SQL、全行、`sqlite_sequence`、版が vZ の前のまま、`conn.is_autocommit()` が true（TX が終わっている）。失敗を外せば同じ DB で当たる（round 1 の是正） | 作り直しの後の失敗で部分的に残る | schema / negative | `test_vz_req903_commit_failure_after_rebuild_rolls_back_everything` | 共有の helper を使わず COMMIT する、ROLLBACK を落とす、`sqlite_sequence` を戻す所を TX の外に置く |
| M9 | 手順 6 の確かめの失敗（`foreign_keys` を OFF にして、存在しない `csv_import_id` を指す `sale_records` の行を入れておく。手順 1 は `csv_imports` の行だけを見るので通り、手順 3〜5 の後に全体の `PRAGMA foreign_key_check` が `sale_records` を返す）で `MigrationFailed`（`作り直し` と `foreign_key_check` と表名）、M8 と同じ 5 項目が vZ の前のまま（round 1 の是正） | 確かめの後の失敗で部分的に残る | schema / negative | `test_vz_req903_verification_failure_after_rebuild_rolls_back_everything` | 手順 6 の確かめを落とす、確かめの失敗で ROLLBACK しない、確かめを `inventory_movements` だけに絞る |

### L: 精算の窓と求め直し（ADR D3・D2・D9、31）

計数の日付 D・今日 T・精算 k の精算日 d_k の組は ADR D3 の判定表から決め、期待は表の文から写す。

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| L1 | 既定の計数（日付 D）に対する精算: `d_k < D` 入る、`d_k = D` 決まらない（D の全ての精算）、`d_k > D` は D に精算があれば入らない（A1） | 境界の取り違え | unit | `test_stock_ledger_req205_default_count_settlement_window` | `=` を入るにする、D の 2 本目の精算を入るにする、A1 を当てない |
| L2 | 既定の計数で D に精算が無いとき、`d_k > D` の最初の 1 本だけ決まらない、残りは入らない | 「最初の 1 本」の誤り | unit | `test_stock_ledger_req205_default_count_first_settlement_after_quiet_day` | 全部を決まらないにする、全部を入らないにする |
| L3 | fresh な計数（今日 T、保存の時点の n）: `k ≤ n` 入る。`k > n` のうち `d_k < T` 入る、`d_k = T` 決まらない、`d_k > T` は T に精算（n を含む）があれば入らない、無ければ最初の 1 本だけ決まらない | fresh の比べ方の誤り | unit | `test_stock_ledger_req205_fresh_count_settlement_cursor` | `k < n` にする、n を見ずに日付だけで決める |
| L4 | fresh で n が NULL（取込みが無かった）なら全ての精算を日付で判定する | NULL を 0 とみなす | unit | `test_stock_ledger_req205_fresh_count_without_imports_uses_dates` | NULL を 0 に倒す（精算 0 を入るにする） |
| L5 | 理由 1: 決まらない精算の取込みにその商品の有効な Z004 の movement がある → `UndeterminedSettlement`、範囲は適用する・しないの最小と最大 | 理由 1 の漏れ・範囲の誤り | unit | `test_stock_ledger_req205_undetermined_settlement_movement_recheck` | 理由を足さない、範囲の端を取り違える |
| L6 | 理由 1: movement の無い行でも、取込みのときの `stock_linked = 1` で個数 ≠ 0 か金額 ≠ 0 の行があれば要再確認。`stock_linked = 0` の行と、個数・金額とも 0 の行は理由にならない。今の `pos_stock_sync` を見ない | 今の設定を見る・0 の行を拾う | unit | `test_stock_ledger_req205_undetermined_settlement_uses_stock_linked_at_import` | 今の `pos_stock_sync` を見る、金額 ≠ 0 の条件を落とす |
| L7 | 取り消した取込み（`rolled_back`）は精算の一覧に入らない | 取消の漏れ | unit | `test_stock_ledger_req205_rolled_back_import_excluded_from_settlements` | active の条件を落とす |
| L8 | 初期在庫の印は `reference_type` NULL で `movement_type` が `receiving`。`sale_auto`・`reference_type` NULL（demo の seed）は業務日で窓を持つ | seed の売上を初期在庫とみなす | unit | `test_stock_ledger_req205_seed_sale_without_reference_is_windowed` | `movement_type` の条件を落とす（計数の後の seed の売上が引かれない） |
| L9 | 計数の無い商品では `phase = 1` の movement を含む全ての有効な movement の和（誤りで止まらない） | (a) の止まりが残る | unit | `test_stock_ledger_req205_sum_includes_settlement_movements_without_counts` | `phase != 0` の分岐を残す |
| L10 | 理由の順と重複なし（`undetermined_settlement`・`same_date_movement`・`conflicting_counts`） | 順・重複の誤り | unit | `test_stock_ledger_req205_recheck_reason_order_and_dedup` | 理由を push する順を変える、同じ理由を 2 回足す |
| L11 | 欠けがあると過剰に決まらない側へ倒れ、欠けの後追いの取込みで決まる（D の精算の有無と「最初の 1 本」は取り込み済みの精算で決める） | 欠けで過小になる | unit | `test_stock_ledger_req205_gap_errs_on_undetermined_side_and_resolves_after_backfill` | 欠けた精算を推測で埋める |
| L12 | Z004 の確定・取消で求め直す商品 = 計数を持つ全ての商品 + その取込みの売上か movement を持つ全ての商品（連動の on / off を問わない） | 求め直しの漏れ | integration | `test_stock_ledger_req205_req401_recompute_targets_for_settlement_change` | 計数を持つが取込みに無い商品を省く、連動 off の商品を省く。観測: (a) 計数を持ち取込みに行の無い商品と (b) 計数が無く連動 off の行だけを持つ商品の写しを、確定と取消の前にわざと記録から食い違わせ（`UPDATE products SET stock_quantity`。J6 と同じ形）、確定と取消の後に両方の写しが `derive_stock` と一致することを見る（求め直さないと食い違いが残る。値の変わらない fixture では省いても green なので、食い違いで観測する。round 2 の是正） |
| L13 | fresh な計数の判定の n は、保存の時点の取込みのカーソル c（`seen_import_id`）以下の取込みID を持つ、今 active な取込みの最大の精算回数（ADR D3。完了の (ii)(iii) も同じ。Codex の closure の Gated Amendment で round 3 の「n の active な取込みがあるときだけ使う」を置き換えた）。列 (a)（Codex の反例）: 精算 1（精算日 T−1）= I1 → 商品 A の fresh な計数 5（c = I1）→ I1 を取消 → 精算 1（T+1、A × −1）= I2 → 写し 4・範囲 4〜5・`undetermined_settlement`（判定の n は NULL、I2 は T より後の最初の 1 本で決まらない）。列 (b)（取消と取り込み直し）: 精算 99（T、A × −1。数える前の売上）= I1・100（T、A の行なし。誤った版）= I2 → A の fresh な計数 5（c = I2）→ 対照: 写し 5・要再確認なし → 100 を取消 → 写し 5・要再確認なし（判定の n = 99 で 99 は入る）→ 正しい版の 100（T、A × −1）を I3 で取り込み直す → 写し 4・範囲 4〜5（I3 は保存の後の取込みで精算日 T。過剰側。owner の決定 TD-400）→ A を入れ直す 5（c = I3）→ 写し 5・要再確認なし。列 (c)（判定の n でない精算の取り込み直し。Amendment の review の Codex の反例。Gated Amendment の 2 本目）: 精算 99（T、A × −1。数える前の売上）= I1・100（T、A の行なし）= I2 → A の fresh な計数 5（c = I2）→ 対照: 写し 5・要再確認なし → 99 を取消 → 写し 5・要再確認なし（判定の n = 100）→ 正しい版の 99（T、A × −1）を I3 で取り込み直す → 写し 5・要再確認なし（I3 は判定の n の算出に入らないが、k = 99 ≤ n = 100 で入る） | 番号を証拠に使う（番号の active な取込みの有無か保存した `settlement_no` で n を決める）、取り消した取込みを判定の n に含める | unit | `test_stock_ledger_req205_fresh_settlement_cursor_uses_import_ids` | 番号の active な取込みの有無で n を決める（round 3 の規則。(a) で写し 5・要再確認なし、(b) の取消の後に写し 4・範囲 4〜5、取り込み直しの後に写し 5・要再確認なし）、判定の n に取り消した取込みを含める（(a) で n = 1 になり写し 5）、c を見ずに今の active な最大を使う（(a) で写し 5、(b) の取り込み直しの後に I3 が入り写し 5・要再確認なし）、`k ≤ c`（精算回数を取込みID と比べる。fixture の精算回数 99・100 と取込みID 1〜3 を離しておくので、(b) の取消の後に 99 が決まらず要再確認になる）、判定の n が NULL のときに 0 とみなす（精算 0 を入るにする。L4 と同じ）、取り込み直した取込み（c より大きい取込みID）を k ≤ n に関わらず日付で判定する（D7 の初めの文の一般化。(c) の取り込み直しの後に写し 4・範囲 4〜5） |

### C: 計数の保存と読取り（ADR D5、35）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| C1 | 既定の計数の保存: `stock_counts` に `default` の行、写しを求め直す、結果（id・種類・在庫・範囲・理由・`later_count_exists`） | 保存・求め直しの漏れ | integration | `test_save_stock_count_req205_default_saves_and_recomputes` | `recompute_stock` を呼ばない、結果の field を落とす |
| C2 | fresh な計数: `business_date` は今日（違えば `ValidationFailed`）、`settlement_no` は同じ TX の最大の精算回数（無ければ NULL。ADR D1 の鍵の n）、`seen_movement_id` は `MAX(id)`（無ければ 0）、`seen_import_id` は同じ TX の `MAX(csv_imports.id)`（取り消したものを含む。無ければ NULL。Codex の closure の Gated Amendment）。列: 精算 100（T）= I1 → 99（T、A × −1）= I2 → A の fresh な計数（c = I2）→ 100 を取消で、判定の n = 99（99 は入り、A は要再確認にならない）。最大の id の取込みが取り消されている DB でも c はその id | カーソルの誤り | integration | `test_save_stock_count_req205_fresh_stores_cursor_in_same_tx` | 今日以外を通す、NULL を 0 にする（取込みの無い DB で `settlement_no` が 0、`seen_import_id` が 0 なら CHECK で落ちる）、`MAX(id)` でない値を書く（movement のある DB で `seen_movement_id` が最大の id と違う）、`seen_import_id` を n の取込みID（I1）で書く（列で判定の n が NULL になり、99 が決まらず A が要再確認になる）、`seen_import_id` を取り消した取込みを除いた最大で書く（最大の id の取込みが取り消されている DB で値が違う）。カーソルを TX の外で読む mutant は 1 本の接続の test では値が同じで観測できないので、Final Review の code 読みの項目へ移した（Residual Test Gaps。round 2 の是正） |
| C3 | 未来の日付・形の違う日付（`2026-4-7`）・実在しない日付（`2026-02-30`）は何も書かずに拒否 | 日付の検査の漏れ（循環の前提が崩れる） | negative | `test_save_stock_count_req205_rejects_future_or_invalid_dates` | `validate_business_date` か今日との比べを落とす |
| C4 | 値の範囲: 0 は保存できる、負と `1000000000` は何も書かずに `ValidationFailed`、`999999999` は保存できる | 上限の漏れ | negative / boundary | `test_save_stock_count_req205_count_value_range` | 上限を落とす、0 を拒む |
| C5 | `stocktake_item_id` が進行中の棚卸しの明細でない（存在しない id・完了した棚卸しの明細）、明細の商品と `product_code` が違う → 書く前に `ValidationFailed`（「この明細は別の商品です」）、`stock_counts` の行数が変わらない | 明細の取り違え | negative | `test_save_stock_count_req205_rejects_mismatched_stocktake_item` | 一致の検査を落とす（別の商品の明細に計数が入る）。INSERT の後に検査する: 存在しない id（`999999`）の case で、検査が先なら `ValidationFailed`、INSERT が先なら `stock_counts.stocktake_item_id` の外部キー（`REFERENCES stocktake_items(id)`、`foreign_keys` は ON）の失敗で `DatabaseError` になり、kind で見分ける（同じ文で TX が戻る case は最終の状態が同じなので順を見分けない。round 2 の是正） |
| C6 | 明細の `actual_count`・`counted_at` は効く計数に揃う（fresh が先の計数を置き換える。後入力の古い日付の既定の計数は効かない） | 写しの誤り | integration | `test_save_stock_count_req205_item_copy_follows_effective_count` | 最後に保存した値を写す |
| C7 | `preview_stock_count_placement` は ADR D3 の「計数どうし」で、その日付の計数より後の計数があるとき `later_count_exists = true`（保存は拒まない） | 後入力の確認の誤り | unit | `test_preview_stock_count_placement_req205_later_count_exists` | 日付だけで比べる（fresh の後の同じ日付の既定を後とする） |
| C8 | `zero_uncounted_items`: 選んだ商品に値 0 の既定の計数を 1 TX で、商品ごとに求め直す。各計数は進行中の棚卸しの当該明細を参照する（`stocktake_item_id`）。日付は選べ、未来は拒否。選んだ商品のどれかが明細を持たなければ何も書かずに `ValidationFailed`（「この商品は棚卸しの対象にありません: {商品コード}」）。0 にした後に他の理由が無ければ完了できる（round 2 の是正） | 部分的な保存、明細を参照しない計数 | integration / negative | `test_zero_uncounted_items_req205_one_tx_per_request` | `stocktake_item_id` を入れない（0 にした後の `complete_stocktake` が `uncounted` で拒まれる）、商品ごとに commit する（明細を持つ商品 A と明細の無い商品 B の順の依頼で、拒否の後に A の計数が残る）、明細の無い商品を飛ばして続ける（拒否にならない）、未来の日付を通す |
| C9 | `list_recheck_products`: 写しが要再確認の商品、進行中の棚卸しに明細があれば `active_count` と `stocktake_item_id`、無ければ `count_now`、理由つき。各 target が `stock_unit`・写しの `stock`・範囲の両端を持つ（長さの商品で cm の値。round 1 の是正） | 回復先の誤り、範囲が届かない | unit | `test_list_recheck_products_req205_actions_reasons_and_range` | action を取り違える、架空の id を入れる、範囲を null のまま返す、単位を落とす |
| C10 | 明細の読取り（`get_stocktake_items`・`find_stocktake_item`）: 効く計数の値・種類・日付、写しの在庫と範囲、理由、差異（効く計数 − 入っている記録だけの帳簿。計数が無ければ null）、`stock_unit`。進捗の計数済み = 明細を参照する計数がある | 差異・計数済みの定義の誤り | integration | `test_get_stocktake_items_req205_effective_count_and_difference` | 差異を `actual_count - system_stock` にする、計数済みを `actual_count` の NULL で数える |
| C11 | `preview_stock_count_placement` は棚卸しの明細の無い商品でも、商品名・単位・写しの在庫と範囲・理由・効く計数（無ければ null）を返す（「今の数を入れる」の欄。42・35。round 1 の是正） | 計数の欄の文脈が届かない | unit | `test_preview_stock_count_placement_req205_returns_count_context_without_stocktake` | 明細から読む（棚卸しが無いと返らない）、範囲を落とす、計数の無い商品で 0 を入れる |

### K: 開始・完了（ADR D6、35、§20.5a）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| K1 | 完了: 明細ごとに `valuation_quantity = max(写し, 0)`、評価原価、§20.5a の総額、カーソル（`cutoff_import_id` = `MAX(csv_imports.id)`〈取り消したものを含む。無ければ NULL。Codex の closure の Gated Amendment で最大の精算回数を置き換えた〉・最大の計数の id・`MAX(inventory_movements.id)`〈無ければ 0。round 3 の是正〉）、header を completed。movement を作らず写しを変えない | 在庫を動かす・カーソルの漏れ | integration | `test_complete_stocktake_req205_freezes_valuation_without_movements` | movement を書く（`inventory_movements` の行数が変わる）、カーソルを書かない。`recompute_stock` を呼ぶ: 計数の保存の後に 1 商品の写しをわざと記録から食い違わせ（`UPDATE products SET stock_quantity`、`stock_recheck` は 0 のまま。J6 と同じ形）、完了の後もその写しと `valuation_quantity`（= `max(食い違わせた写し, 0)`）が残ることを見る（求め直すと写しが記録の値へ戻る。写しが正しい fixture では値が変わらず観測できない。round 2 の是正） |
| K2 | `settlement_pending` の (i) 精算日が X 以上の精算が無い、(ii) 取り込まれていない k が隣の精算の日付で X を挟む、(iii) 既定の計数（と判定の n が NULL の fresh な計数。L13）で X より前の精算が無い → `StocktakeGuard`（`completion_blocked`、明細ごとの target と `block_reason`）、何も書かない。`settlement_pending` に `SettlementPendingHint`（(ii) の欠けた精算回数の区間、(i) の X の最大、(iii) の X の最小。当たらない所は null。round 1 の是正）。列（closure の是正を Codex の closure の Gated Amendment で書き直した）: 精算 100（精算日 T、商品 A の行なし）= I1 → A の fresh な計数（今日 T、c = I1）→ 99（T、A の行なし）= I2 を後追いで取り込む → (iii) は出ない（判定の n = 100）→ 100 を取り消す → 判定の n は NULL（I2 は保存の後の取込み）なので (iii)（T より前の精算が無い）で `settlement_pending`、hint の (iii) の日付は T（(i) は 99 で満たし、A は 99 に行が無いので要再確認にならない）→ 100 を I3 で取り込み直しても (iii) が残る（I3 も保存の後）→ A を入れ直す（c = I3）と `settlement_pending` が消え、完了できる | 未取込みを見落とす、直し方を名指せない、取消の後も n で比べる | negative | `test_complete_stocktake_req205_settlement_pending_three_conditions_and_hint`、`test_complete_stocktake_req205_settlement_pending_fresh_cursor_uses_import_ids`（列） | (i)〜(iii) のどれかを落とす、fresh の n の扱いを誤る、hint の区間・日付の最大と最小を取り違える、完了の判定が c を見ない（列で取消の後に今の active な最大 99 を n にして (iii) を見ず、完了が通る）、番号の active な取込みの有無で n を決める（round 3 の規則。取り込み直しの後に `settlement_pending` が消える）、保存した `settlement_no` を n にする（取消の後も n = 100 として (iii) を見ず、完了が通る） |
| K3 | 明細の商品の写しが要再確認 → `recheck` | 要再確認を見落とす | negative | `test_complete_stocktake_req205_blocks_on_recheck` | `stock_recheck` を見ない |
| K4 | 未計数 → `uncounted`。完了の時点で廃番かつ写し 0 の明細は止めない。理由は全部集めてから返す | 例外の誤り・最初の理由で返す | negative | `test_complete_stocktake_req205_uncounted_except_discontinued_zero` | 廃番 0 を止める、廃番で在庫 ≠ 0 を通す、最初の理由だけを返す |
| K5 | 総額が安全な整数の範囲の外（原価と実数が上限の個数の単位の商品 10 個）→ `ValidationFailed`（「合計金額が大きすぎます」）、何も書かない。9 個は保存できる | 範囲の漏れ | negative / boundary | `test_complete_stocktake_req205_total_cost_safe_integer_range` | `check_safe_integer` を落とす |
| K6 | §20.5a の評価の golden（基準数量・四捨五入・負の原価・i128 の溢れ）を新しい完了で保つ | 評価の誤り | unit | 既存の `test_complete_req205_total_cost_*` を新しい完了へ移す（T2） | 評価の関数を差し替える |
| K7 | 開始: 進行中が無ければ全ての対象の商品の明細（自動の 0 の入力なし）、進行中があれば `StocktakeInProgress`、対象 0 件は今の文言 | 自動の 0 が残る | integration | `test_start_stocktake_req205_creates_items_without_auto_fill` と既存の開始の test（T2） | 廃番 0 を自動で埋める |
| K8 | 開始・完了の操作ログ（35 の「操作ログ」: `stocktake_start` の detail は `stocktake_id`・`item_count` の 2 つちょうど（round 3 の是正で開始の test に key の assert を足す）、`stocktake_complete` の summary の差異の件数と detail の `difference_count` は差異が null でなく 0 でない明細の数、detail は `stocktake_id`・`total_cost`・`total_items`・`difference_count`。round 2 の是正）と整合性の結果を含む完了の結果 | ログ・結果の欠け、外した key の残り、差異の件数の取り違え | integration | `test_start_stocktake_req205_operation_log`（移し、`detail_json` の key が `stocktake_id`・`item_count` の 2 つちょうどの assert を足す。今の test は `operation_type` と summary の件数だけを読み、`auto_filled_count` を残した mutant を見分けない）、`test_complete_req205_operation_log`（期待を変える。差異のある明細 1 つ〈計数 8・帳簿 10〉と無い明細 1 つの fixture で summary が「差異: 1件」、detail の key が上の 4 つちょうど）、`test_complete_stocktake_req205_includes_integrity_result`（移す）。T2 | ログを落とす、detail に `adjusted_count`・`force_fill_used`・`auto_filled_count` を残す、差異の件数を明細の数か 0 で書く |

### R: 記録の詳細と完了の後の差（ADR D6、35、65）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| R1 | 完了した棚卸しの明細は、カーソルの時点の効く計数と差異、評価数量・評価原価、差異が負で評価原価があれば行のロス原価（`cost_line_centi` の 1/100 円の十進の文字列。長さは基準数量 100） | カーソルの後の計数を使う、基準数量を落とす | integration | `test_get_stocktake_record_req206_items_at_cutoff_with_loss_cost` | id ≤ `cutoff_count_id` を落とす、基準数量で割らない |
| R2 | 完了の後の差: 範囲は完了の時点にあった記録（カーソル以下。日付によらない）に、完了の後に入れた完了の前の入出庫・初期在庫・精算の記録を足したもの（計数はカーソル以下だけ。35。round 3 と closure の是正）。完了日以前の日付で完了の後に入れた入出庫が差になり、完了日より後の日付で完了の後に入れた入出庫は入らない。精算は精算回数 ≤ 完了の時点の精算の n（`cutoff_import_id` 以下の今 active な取込みの最大の精算回数。精算日によらない）と精算日 < 完了日。確定の差と要再確認の差を分け、確定の差の合計は円の十進の文字列。同じ範囲の確かめ（round 3 の是正）: (a) 今日 T、精算 100（T、売上 0）→ 商品 A の fresh な計数 5 → 精算 101（T+1、A × 1。知らせつきで取り込む）→ A の写し 4 で完了（評価数量 4、完了の時点の精算の n は 101）→ 記録を足さずに詳細を開くと A の差は無く、差の一覧は空、確定の差の合計は ¥0。(b) 完了の前に業務日 T+1 の入庫（A × 2）を入れて完了 → すぐ詳細で差 0。(c) 完了の後に業務日 T+1 の入庫を入れても差は出ない。(d) (a) の後に A の既定の計数 3（日付 T = 完了日。fresh な計数 5 と重なる）を保存しても A は差の一覧（確定・要再確認のどちら）にも出ない（計数は `cutoff_count_id` 以下だけ。closure の是正）。(e) 完了の後に埋めた欠け（Codex の closure の Gated Amendment）: 商品 C の既定の計数 5（日付 D）、精算 4（D−1）・5（D）・6（D+1）・8（D+3）を取り込み（どれも C の行なし。7 は欠けで、隣の 6・8 の精算日が D を挟まないので完了の (ii) に当たらない）、D+3 に完了（評価数量 5）→ 精算 7（精算日 D+3、C × 1）を後追い → C に確定の差 −1（7 は完了の時点の精算の n = 8 以下で範囲に入り、D に精算があるので A1 で計数に入らない） | 差の取り違え、評価と比較の範囲の食い違い | integration | `test_get_stocktake_record_req206_after_completion_diff_split`、`test_get_stocktake_record_req206_after_completion_same_range_as_valuation`（(a)〜(d)）、`test_get_stocktake_record_req206_after_completion_includes_backfilled_settlement`（(e)） | 完了の後に入れた完了日より後の記録を入れる、要再確認の差を合計に入れる、精算を精算日で切る（(a) で 101 が外れ差 +1）、入出庫を業務日だけで切る（(b) で入庫が外れ差 −2）、`cutoff_movement_id` を見ずに全ての movement を入れる（(c) で差 +2）、完了の後に保存した計数を業務日で範囲に入れる（(d) で fresh な計数 5 と衝突し、A が要再確認の差に出る。日付を T より前にすると fresh な計数が後で効かず、観測できない）、範囲の精算を `cutoff_import_id` 以下の取込みだけにする（(e) で 7 が外れ、差が出ない。7 の精算日を完了日と同じにするのは「精算日 < 完了日」の項で入らないようにするため） |
| R3 | 記録の詳細は補正の movement の節を持たない。入出庫の記録の一覧の棚卸しの件数は明細の数 | movement 起点が残る | integration | `test_list_inventory_records_req206_stocktake_count_is_item_count` | 件数を movement から数える |
| R4 | 見つからない・状態の enum（今の契約） | — | regression | 既存の `test_get_stocktake_record_req206_not_found_message`・`test_get_stocktake_record_req206_status_enum_and_fail_fast` を移す（T2） | — |
| R5 | 完了の時点の在庫が負でも `max(…,0)` で評価し、負の数量を評価の関数に渡さない | 負の評価 | negative | `test_get_stocktake_record_req206_after_completion_diff_clamps_negative` | `max` を落とす（`ValidationFailed` か負の差） |
| R6 | 完了の後の差の範囲の精算は完了の時点の精算の n で決め、取消の後に同じ番号を使う取込みを数えない（ADR D6。Codex の closure の Gated Amendment）。列: 商品 A の既定の計数 5（日付 T）、精算 1（T−1）= I1・2（T、A の行なし）= I2 で完了（`cutoff_import_id` = I2、評価数量 5）→ I1・I2 を取消 → 初期化の後の精算 1（T+1、A × 1）= I3・2（T+2、A × 1）= I4 → 記録の詳細で A は差の一覧（確定・要再確認のどちら）にも出ない（完了の時点の精算の n は NULL で、範囲の精算は精算日 < T の active な取込みだけで空。完了の時点の在庫 5） | 番号の戻った新しい系列の精算が完了の時点の在庫に入り、偽の差が出る | integration | `test_get_stocktake_record_req206_after_completion_ignores_reused_settlement_numbers` | 完了のカーソルを最大の精算回数にする（`cutoff_settlement_no` = 2 の形。I3・I4 が範囲に入り、A が要再確認の差〈完了の時点の在庫 3〜4〉に出る）、完了の時点の精算の n に取り消した取込みを含める（同じ） |

### Z: Z004 の preview・確定・取消（ADR D4・D7、32、24）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| Z1 | 順 1: メタが無い（従来の形）・精算回数が数字でない・`9007199254740992` → preview も確定も拒否（32 の文） | 拒否の漏れ・範囲 | negative / boundary | `test_parse_req401_rejects_missing_or_out_of_range_settlement_no` | 範囲の検査を落とす（`9007199254740991` は受ける） |
| Z2 | 順 2: Z004 の取込み（取り消したものを含む）と machine_no が違う → 拒否（round 3 の是正で比べる先を広げた。取消の後の列は Z22）。レジ `0001` の active な取込みと、同じレジの取り消した取込みがある DB で、レジ `0002` の file は今の文（「レジの番号が前の精算と合いません。レジの入替・初期化を確かめてください」）で拒み、取り消した取込みを名指さない（active を優先する。closure の是正） | 別のレジを通す、active な取込みと合わないのに取り消した取込みを原因として名指す | negative | `test_parse_req401_rejects_other_machine_no` | machine_no を比べない、照合先で active を優先しない（取り消した取込みを名指す文になる） |
| Z3 | 順 3: 同じ hash（今の重複の拒否） | — | regression | 既存の重複の test を移す（T2） | — |
| Z4 | 順 4: 同じ精算の active な取込みで hash が違う → 拒否。取り消した取込みは比べない（取り込み直せる） | 二重計上・取り込み直し不能 | negative / integration | `test_parse_req401_rejects_other_version_of_same_settlement_and_allows_after_rollback` | `rolled_back` を比べる先に入れる |
| Z5 | 順 5: 精算回数が小さいのに精算日が後、大きいのに前 → 拒否（両方向） | 順の食い違いを通す | negative | `test_parse_req401_rejects_settlement_order_mismatch_both_directions` | 片方向だけ見る |
| Z6 | 知らせ: 欠けを連続の区間にまとめ昇順（1 つなら `from = to`）、区間の数は欠けた番号の数に依らない（精算 1 と `9007199254740991` の間でも区間 1）。精算日が今日より後は知らせで拒否しない | 区間の誤り・番号を並べる | unit / boundary | `test_parse_req401_settlement_gap_notices_and_future_date` | 番号ごとに並べる（大きな番号で止まる）、未来の日付を拒む |
| Z7 | 同じ精算日の別の精算は確認なしに確定まで進む（`additional_import_confirmed` が無い） | 同日の確認が残る | integration | `test_commit_req401_same_date_other_settlement_needs_no_confirmation` | 同日の snapshot の照合を残す |
| Z8 | 確定: `csv_imports` に精算の鍵、売上に `stock_linked`、在庫連動の行の movement は `business_date = settlement_date`・`phase = 1`・`settlement_no`、求め直す商品を求め直し、結果に `recheck_targets`（単位・写しの在庫・範囲つき。round 1 の是正）と知らせ。写しと `derive_stock` が一致。同じ商品の複数の行の movement の `stock_after` は全部同じ写しの値 | 鍵・求め直しの漏れ、範囲が届かない | integration | `test_commit_req401_writes_settlement_key_and_recomputes` | `settlement_no` を渡さない（CHECK で落ちる）、求め直さない |
| Z9 | 正常な行が全て 0 の file は売上と movement を作らず取込みとして確定し、精算の並び・欠けに入る | 空の拒否が残る | integration | `test_commit_req401_zero_rows_file_is_imported_as_settlement` | 空の対象の拒否を残す |
| Z10 | 共有 JAN の行: 売上は先頭の商品、どの候補の movement も作らない、`stock_linked = 0`、行の error にしない | 候補の在庫を動かす | integration | `test_commit_req401_shared_jan_row_records_sale_only` | 先頭の商品の在庫を動かす |
| Z11 | 在庫連動でない商品の行は売上だけ（`stock_linked = 0`） | movement を作る | integration | 既存の `test_commit_req401_pos_stock_sync_false` を移す（T2） | 連動の分岐を落とす |
| Z12 | 長さの商品の整数の数量は × 100 で売上と movement に入る。× 100 の後の絶対値が `9999999` を越える行は preview の行の error。小数の数量は今の行の error | 100 分の 1・溢れ | unit / boundary | `test_parse_req401_length_unit_quantity_converted_to_cm` | × 100 を落とす、上限の検査を落とす |
| Z13 | 確定の取り直しで preview の行と違う商品に当たる・商品が無くなる行があれば何も書かずに `ImportError`（「商品の登録が読み込みの後に変わりました…」）。例: preview で JAN J → 商品 A、その後の一括 import の上書きで A の JAN を K、B の JAN を J にする（cache の `normalized_jan` J の引当先が B に変わる。商品コード A と今の JAN K からは求まらない。round 1 の是正） | 黙って別の商品へ入れる | negative | `test_commit_req401_rejects_when_original_jan_now_matches_other_product` | 取り直しを cache の `product_code` で引く（元の JAN を見ない）、取り直しの結果を比べない |
| Z14 | 取消: 売上と movement を無効にし、求め直す商品を求め直し、補償の movement を作らず、計数を消さない。結果に `recheck_targets`。取消済みは冪等の Ok。取消で比べる先が無くなった後の取込みは Z22・Z23・L13（round 3 の是正） | 補償・計数の消去 | integration | `test_rollback_req401_voids_and_recomputes_without_compensation` | 補償の movement を書く、計数を消す、求め直さない |
| Z15 | 確定の TX で順 1〜5 を取り直す（preview の後に別の取込みが確定した場合） | TOCTOU | negative | `test_commit_req401_rechecks_rules_in_tx` | 確定で検査を省く |
| Z16 | 部分的な行の error・返品の符号・負の在庫の警告・在庫の上限で TX 全体が戻る（今の契約） | — | regression | 既存の `test_commit_req401_partial_with_errors`・`test_commit_req401_sign_flip_inv1`・`test_commit_req401_negative_stock_warning`・`test_commit_req401_stock_limit_rolls_back_whole_tx`・`test_preview_req401_pos_quantity_limit_is_row_error` を移す（T2） | — |
| Z21 | preview の cache の照合の残りの 3 通り（32 の「preview の cache の契約」の (4)）: (a) preview で単独だった JAN が確定の前に共有 JAN になった（一括 import の上書きで別の商品に同じ JAN）、(b) preview で `unmatched_product` だった行の JAN に確定の前に商品ができた、(c) 確定の TX の順 1〜5 は cache の `settlement`（machine_no・精算回数・精算日）で取り直す（preview の後に同じ精算の別の版が確定していれば順 4 で拒否）。(a)(b) は何も書かずに `ImportError`、token は残る。対照: preview の後に変わったのが在庫連動の設定だけなら拒否せず、`stock_linked` は確定の時点の値（round 1 の是正） | 共有 JAN・未登録の行が黙って別の扱いになる、精算の識別を file から読み直す | negative / integration | `test_commit_req401_rejects_shared_or_newly_matched_jan_and_rechecks_settlement_from_cache` | `shared_jan` を比べない、error 行を照合しない、cache に `settlement` を持たない（確定で順 4 を見ない）、連動の設定の変化を拒否にする |
| Z22 | 取消の後の別のレジ（Codex の反例の列。round 3 の是正）: レジ A（machine_no `0001`）の精算 100（精算日 T−1）を確定 → 商品 A の fresh な計数 5（今日 T、n = 100）→ 精算 100 を取消（active な取込みが 0 件）→ レジ B（`0002`）の精算 1（精算日 T+1、A × 1）の preview と確定が順 2 で拒否（取り消した取込みとだけ合わないので、文が原因の取込みを名指す: 「取り消した取込みとレジの番号が合いません（取込みID: {精算 100 の取込みの ID}、精算 100、取込み日: {その `imported_at`}、レジ: 0001。この file のレジ: 0002）…」。今の文〈「レジの番号が前の精算と合いません…」〉は出ない。何も書かない、token は残る。closure の是正）。A の写しは 5 のまま、要再確認なし。対照: 同じレジ A の精算 100 の別の版（hash 違い）は順 2 を通り、取消の後に取り込める（D7。順 4 は active だけと比べる） | 取消で比べる先が消え、別のレジの精算 1 が通る（店のレジは 1 台の前提。通っても判定の n は取込みのカーソルで求めるので `k ≤ n` で黙っては入らない。Codex の closure の Gated Amendment で書き直した） | negative / integration | `test_parse_req401_rejects_other_machine_after_all_imports_rolled_back` | 順 2 の比べる先を active に限る（B の精算 1 が通る）、順 2 を版ごとに比べて D7 の取り込み直しを拒む、取り消した取込みとだけ合わないのに今の文で拒む（原因を名指さない）、名指す取込みの ID・精算回数・取込み日・レジを取り違える |
| Z23 | 取消の後の同じレジの番号の戻り（round 3 の是正）: レジ A の精算 100（T−1）を確定 → 商品 A の fresh な計数 5（今日 T、n = 100）→ 精算 100 を取消 → レジ A の精算 1（精算日 T+1、A × 1）は比べる active な取込みが無いので順 5 を通って確定する → 判定の n は NULL（c 以下の active な取込みが無い）で日付で判定: T に active な精算が無く、精算 1 は T より後の最初の 1 本で決まらない → A は要再確認（理由 `undetermined_settlement`、範囲 4〜5、写し 4）、結果の `recheck_targets` に A。対照: 取消の前（精算 100 が active）なら同じ file は順 5 で拒否 | 番号の戻った系列の精算 1 を k ≤ n で計数に入れて黙ってずれる | integration | `test_commit_req401_settlement_no_reset_after_rollback_falls_back_to_dates` | 保存した `settlement_no` を判定の n にする（番号を証拠に使う。A の写し 5・要再確認なし）、取消の後も順 5 で取り消した取込みと比べる（精算 1 が拒否になる。取り消した誤った版の精算日で正しい精算を拒みうる形で、32 は採らない） |
| Z24 | 取消の後の同じ番号の精算（Codex の closure の P1 の反例。Gated Amendment）: レジ A の精算 1（T−1）を確定 → 商品 A の fresh な計数 5（今日 T、進行中の棚卸しの明細を参照）→ 精算 1 を取消 → レジ A の精算 1（T+1、A × 1）の preview と確定が順 1〜5 を通る（順 4 は取り消した取込みを比べず、順 5 は比べる active な取込みが無い）→ 結果の `recheck_targets` に A（理由 `undetermined_settlement`、範囲 4〜5、写し 4）。続けて A の明細を持つ棚卸しの完了は `recheck` と `settlement_pending`（(iii)）で拒まれる | 番号の active な取込みがあるだけで、計数の後の売上を数える前の売上として計数に入れて黙ってずれる | integration | `test_commit_req401_reused_settlement_no_after_rollback_falls_back_to_dates` | 番号の active な取込みの有無で n を決める（round 3 の規則。写し 5、`recheck_targets` に A が無く、完了が評価数量 5 で通る）、確定の TX で求め直しの対象から計数を持つ商品を落とす（L12 と同じ観測） |

### X: STK-1・STK-2 と cross feature の model

trace の新しい形（round 2 の是正）: 今の `temporal_trace`（`cross_feature_tests.rs`）は全ての段が同じ日付（`DATE` = `2026-09-16`）で、旧本体の `force_fill: true` と「完了が写しを計数で上書きする」前提を持つ。新しい完了は締めの精算の取込みを待ち（35 の `settlement_pending` の (i)〜(iii)）、同じ日の売り買いの後の既定の計数は要再確認になる（ADR D3）ので、trace を日付と精算回数で書き直す。D = `2026-09-16` を今日 T とし、World の初めに精算 1（精算日 D−1、正常な行が全て 0）を取り込む（(iii) と fresh の n のため）。段: `Start`（開始）、`Count`（SKU0 の既定の計数、日付 D、値は物理の数）、`Recount`（SKU0 の fresh な計数、今日 D）、`Move(op)`（業務日 D の入出庫）、`PosSale`（物理の数 −2、次の精算に載る）、`Close`（次の精算回数・精算日 D の Z004 を、未取込みの売上の行で取り込む。無ければ全て 0 の file）、`Complete`（XFA-NOSYNC を `zero_uncounted_items` で 0 にしてから完了。`completion_blocked` なら `block_reason` の集合を trace の結果に積んで続ける）。oracle: (a) 未取込みの売上が無い時点で、SKU0 が要再確認なら範囲が物理の数を含み、そうでなければ写しが物理の数と同じ（黙ってずれない）、(b) 完了が通ったら写しが完了の前と同じで物理の数と同じ、(c) trace ごとの `completion_blocked` の列が下の期待と同じ。

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| X1 | STK-1: 数えた後の入出庫が完了で打ち消されない（新しい経路で PASS）。trace は 4 つの `op`（入庫・手動販売・廃棄・返品）ごとの `[Start, Count, Move(op), Close, Complete, Recount, Complete]` と `[Start, Count, PosSale, Close, Complete, Recount, Complete]`。1 回目の完了は `[recheck]` で止まり（同じ日の記録は決まらない）、入れ直しの後に通る | 確定が数えた後の記録を打ち消す | integration / regression | `test_cross_feature_req205_count_then_movement_is_kept`（`diagnostic_cross_feature_req205_count_then_movement` を改名し `#[ignore]` と `XFA_TEMPORAL_FAIL` を外す） | 完了が写しを計数の値に上書きする（oracle (b)）、同じ日の記録を要再確認にしない（1 回目の完了が通り、oracle (c)）、範囲が物理の数を含まない（oracle (a)） |
| X2 | STK-2: 完了の後に届いた過去の精算が二重に減らない（新しい経路で PASS）。fixture: 精算 1（D−3）・3（D−1）を全て 0 で取り込み、精算 2（精算日 D−2、SKU0 の 2 個の売上。物理の数は数える前に 8）を保留した World。trace `[Start, Count(8), Close, Complete, 精算 2 を取り込む]`。完了は止まらず（欠けた 2 の隣の精算日 D−3・D−1 が計数の日付 D を挟まない。35 の (ii)）、精算 2 は計数の日付より前なので計数に入り、取り込んだ後も写しは 8 | 後着の Z004 の二重減算 | integration / regression | `test_cross_feature_req205_req401_late_import_not_double_counted`（`diagnostic_cross_feature_req205_req401_late_import` を改名し `#[ignore]` と `XFA_LATE_IMPORT_FAIL` を外す） | 計数に入っている精算を窓で除かない（写しが 6）、完了の後の取込みで完了の評価額を書き換える |
| X3 | cross feature の model（XFA-D1〜D3）の 3 本を新しい経路で保つ。`test_cross_feature_req205_controls` の 4 つの対照は上の形で書き直し、どれも oracle を破らない: `[Start, Count, Close, Complete]`（止まらない）、`[Start, Count, Move(ManualSale), Recount, Close, Complete]`（止まらない）、`[Start, PosSale, Count, Close, Complete, Recount, Complete]`（1 回目は `[recheck]`）、`[Start, Count, PosSale, Complete, Close, Complete, Recount, Complete]`（1 回目は `[settlement_pending]`、2 回目は `[recheck]`） | 対照が新しい契約の正しい列で失敗する | integration | `test_cross_feature_req201_req202_req203_req204_req401_model`・`test_cross_feature_req904_req203_model_detects_consistent_corruption` を移す、`test_cross_feature_req205_controls` は期待を変える（T2） | 対照の列で oracle (a)〜(c) のどれかが破れる（窓の判定・完了の拒否の取り違え） |

### J: 共有 JAN と在庫連動を始める検査（ADR D4・D8、30）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| J1 | create: 在庫連動の商品と同じ JAN の別の商品がある、または連動の新しい商品が別の商品と JAN を共有する → `ValidationFailedAt`（field `pos_stock_sync` か `jan_code`、30 の文） | create の漏れ | negative | `test_create_product_req101_rejects_shared_jan_with_stock_linked` | create で検査しない |
| J2 | update（在庫連動を始める場合だけ。update は JAN を変えない: `ProductUpdateRequest` に `jan_code` が無い）: 連動しない B と C が同じ JAN を持つ（今の不変条件で許される組）とき、C を在庫連動にする保存を拒否（field `pos_stock_sync`、30 の文）し、C の在庫連動を付けないほかの欄の保存は通る（round 1 の是正で JAN の向きの 2 通りを J3 へ移した） | 在庫連動を始める側の漏れ、連動しない保存の過剰な拒否 | negative | `test_update_product_req102_rejects_stock_link_on_shared_jan` | update の検査を落とす、自分の JAN だけを見て他の商品を見ない、在庫連動の変化を見ずに共有 JAN だけで拒む |
| J3 | 商品一括 import（JAN が変わる経路は上書きの行だけ）: preview で、上書きの行で連動の A の JAN を非連動の C と同じ値に、上書きの行で非連動の C の JAN を A と同じ値に、新規の行で連動の商品が既存の商品と JAN を共有、同じ CSV の新規の行どうしで同じ JAN（どちらも在庫連動。後の行だけが error 行）、の 4 通りで当該行だけ error 行（30 の文）、他の正常行は続く（round 1 の是正で J2 から 2 通りを移し、round 2 の是正で 4 通り目を足した）。確定（round 2 の是正。30 の D-118 の節）: preview で正常だった行が確定の TX で当たる 2 通り — (a) preview の後に別の経路で DB が変わった（preview の後に連動の商品 X を同じ JAN で登録する）、(b) 上書きを選んだ 2 行が同じ新しい JAN になり片方が在庫連動 — で、何も書かずに（正常な他の新規の行も登録しない）`validation`（field なし）、message は最初に当たった行の商品コードと preview の行の error と同じ文と「その行をスキップにするか、ファイルを選び直して」を含む。画面: 確定の拒否で Alert に BIZ の message をそのまま出し（解析しない）、「戻る」で preview と上書きの選択に戻り、ファイルを選び直せる | preview だけの検証、どちらかの商品の向きの漏れ、CSV の中の重なりの漏れ、確定で当たった行を黙って飛ばす・部分的に確定する、拒否の後に preview と選択を失う | integration / negative / component | BIZ: `test_import_req104_shared_jan_row_error_both_directions_and_commit_recheck`。画面: `ProductImportPage.test.tsx` の `J3 commit rejection shows the named row and keeps the preview` | commit で検査しない（(a)(b) で確定が通る）、上書きの行で自分の側だけを見る、preview が先の新規の行を当てない（4 通り目で 2 行とも正常）、当たった行だけを飛ばして他を確定する（(a) で正常な新規の行が登録される）、message が行を名指さない、画面が `validation` で idle に戻る（選択を失う） |
| J4 | 在庫連動を始める（false → true）: 連動なしの Z004 の売上（`stock_linked = 0`、個数 ≠ 0 か金額 ≠ 0）が全て効く計数に「入る」ときだけ許す。「入らない」「決まらない」・衝突・計数が無く売上がある → `StocktakeGuard`（`recount_required`、`active_count` / `count_now`）。J2 の共有 JAN の組にも当たる商品では J2 の `validation` を先に返す（数えても直らない拒否を先に。30。round 2 の是正） | 決まらない売上を通す、拒否の順の取り違え | negative / integration | `test_update_product_req102_start_stock_link_requires_recount` | 「決まらない」を通す、計数の無いときを通す、共有 JAN と両方に当たる商品で `recount_required` を返す |
| J5 | 一括 import の上書きで連動を始める行も同じ検査で行の error。共有 JAN にも当たる行は共有 JAN の文を先に返す。確定の TX で当たれば J3 と同じ形で確定全体を戻す（preview の後に連動なしの売上を取り込んだ場合。round 2 の是正） | import の漏れ、拒否の順の取り違え | negative | `test_import_req104_start_stock_link_row_error` | import の上書きで検査しない、共有 JAN と両方に当たる行で `recount_required` の文を先に返す、確定で検査しない |
| J6 | true → false は記録も確認も要らない。連動の切替の TX でその商品を求め直す（値は変わらない） | 切替で写しがずれる | integration | `test_update_product_req102_toggle_stock_link_recomputes` | 求め直しを呼ばない（呼ばなくても値は同じなので、写しをわざと食い違わせた fixture で確かめる） |
| J7 | 在庫連動を始める検査の判定の n も取込みのカーソルで求める（ADR D3・D8。Codex の closure の Gated Amendment）。列: 在庫連動でない商品 C の fresh な計数 5（精算 1〈T−1〉= I1 の後。c = I1）→ I1 を取消 → 精算 1（T+1、C × 1、`stock_linked = 0`）= I2 → C を在庫連動にする保存が `StocktakeGuard`（`recount_required`）で拒まれる | 計数の後の連動なしの売上を「入る」として連動を始め、写しがその売上を引かない | negative / integration | `test_update_product_req102_start_stock_link_judges_settlements_by_import_cursor` | 番号の active な取込みの有無で n を決める（round 3 の規則。I2 の売上が `k ≤ n` で「入る」になり、連動の開始が通る） |

### I: 整合性（ADR D10、36）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| I1 | `shared_jan_conflicts`: 連動の商品と同じ JAN の別の商品の組を JAN の昇順・商品コードの昇順で返す。`fix_integrity` の対象にしない。写しの比較の結果を変えない | 組の漏れ・自動の修正 | integration | `test_run_integrity_check_req904_lists_shared_jan_conflicts` | 非連動どうしの組を出す、連動の商品だけの組を落とす、補正する |
| I2 | 精算の窓を持つ `derive_stock` と写しの比較: Z004 の確定・取消・計数の後に差異なし | 整合性の誤検知 | integration | `test_run_integrity_check_req904_no_mismatch_after_settlements_and_counts` | 整合性の検査が精算の窓と違う規則で求める |

### W: CMD と wire（40・41・42、D-061）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| W1 | `StocktakeGuard` → kind `stocktake_guard` と `stocktake_recovery` が Some。既存の kind では None | 回復型の漏れ | unit | `test_cmd_error_req205_stocktake_guard_carries_recovery` | 変換で recovery を落とす、既存の kind に Some を入れる |
| W2 | 新設 4 の command が `collect_commands` と test の一覧に登録され、`update_count` が無い | 登録の漏れ | unit | `test_stocktake_cmd_req205_new_commands_registered` | 登録を落とす |
| W3 | BIZ は時計を読まず、引数の `today` で日付を判定する（CMD が PC の今日を渡す） | BIZ が時計を読む | unit（BIZ） | `test_save_stock_count_req205_uses_today_argument_not_clock`（`stocktake_service`。round 2 の是正で CMD の test から BIZ の test へ移した） | BIZ の中で `chrono::Local` を読む: `today` = `2000-01-05` を渡し、fresh な計数（`business_date` `2000-01-05`）が保存でき、既定の計数（`business_date` `2000-01-06`）が未来として拒まれることを見る（時計を読むと、前者は今日でないとして、後者は過去として扱われ、どちらも逆になる）。CMD が PC の今日を渡すことは test で時計を差し替えられないので Final Review の code 読みの項目（Residual Test Gaps） |
| W4 | `complete_stocktake(stocktake_id)`（`force_fill` なし）、`completion_blocked` の wire が完全一致 | 旧い引数が残る | unit | `test_complete_stocktake_cmd_req205_completion_blocked_wire` | `force_fill` を残す |
| W5 | `commit_csv_import(preview_token)`、成功で token を消し、拒否・失敗では残す | token の消し方 | unit | `test_csv_cmd_req401_commit_keeps_token_on_rejection_and_removes_on_success` | message の文字列で消す分岐を残す |
| W6 | 新しい有限の値（Z14）の wire の round-trip | 生成型と Rust の食い違い | unit | `finite_enum_contract_tests.rs` に足す行（`test_response_only_enum_wire_serialize_req303_req501` の一覧） | enum の rename を誤る |
| W7 | TS: `CMD_ERROR_KIND` と `describeError` が `stocktake_guard` を扱い、未知の code を続行にしない | TS の twin の漏れ | unit | `describe-error.test.ts` の `stocktake_guard` の case | `CMD_ERROR_KIND` に足さない（`satisfies` で typecheck が落ちる） |
| W8 | `CountRecoveryTarget` の参照なしは null（架空の id を使わない）、`recount_reasons` は要再確認の対象のときだけ | 架空の id | unit | `test_cmd_error_req205_recovery_targets_use_null_for_missing_refs` | 0 を入れる |

### U: 画面（73・55・51・65・75、UI_TECH_STACK）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| U1 | 「数を保存」: 保存の日付（既定は今日）を出し、保存で `saveStockCount`（`fresh: false`）、保存済みの feedback、検索欄へ focus | 日付の非表示・focus | component | `StocktakePage.test.tsx` の `U1 default count saves with today's date and returns focus` | 日付を出さない、focus を戻さない |
| U2 | 「前の日の分を入れる」: 今日より後を選べない、`previewStockCountPlacement` の `later_count_exists` で「後の数が既にあり、そちらが効きます」を保存の前に出す（拒まない） | 確認の漏れ | component | `U2 back-dated count warns when a later count exists` | 確認を出さない、保存を拒む |
| U3 | 商品の「入れ直す」は前に保存した数があるときだけ、`fresh: true`・日付は今日だけ | fresh の誤送信 | component | `U3 recount sends fresh true with today only` | 前の数が無いのに出す、日付を選ばせる |
| U4 | m の商品の実数は小数 2 桁までを cm に直して送り、小数 3 桁・負・上限の外は入力の error | m の換算の誤り | component | `U4 length product count converts meters to cm` | 浮動小数で掛ける（`0.29` → 28） |
| U5 | 日付の変わり目（時計を差し替える）: 前の保存から日付が変わったら確認 | 確認の漏れ | component | `U5 date change since last save asks before saving` | 日付の比べを落とす |
| U6 | 要再確認の一覧（棚卸しの有無にかかわらず。各行は商品・範囲〈`CountRecoveryTarget` の範囲を単位どおり。長さは m〉・理由）と行の「入れ直す」（`stocktake_item_id` を送る）、保存の後に一覧から消える（範囲の表示は round 1 の是正） | 一覧・再取得の漏れ、範囲が出ない | component | `U6 recheck list shows range and recount resolves the row` | 保存の後に invalidate しない、範囲を出さない、長さを cm で出す |
| U7 | 完了の確認の文に H2 の問い、`completion_blocked` で `block_reason` ごとの一覧と「取り込んでいない精算がある」の直し方の文（`SettlementPendingHint` の `missing`・`on_or_after`・`before` から 73 の文。round 1 の是正）、「0 にする」と「残りをまとめて 0 にする」（確認を挟む、日付を選べる） | 理由の混同、直し方が出ない | component | `U7 completion blocked lists reasons, settlement hint and zero actions` | 理由を 1 つの一覧に混ぜる、確認を挟まない、hint を出さない |
| U8 | 完了の結果: 評価額と明細の数、在庫を動かさない旨の文 | 旧い補正の一覧 | component | `U8 completion result shows valuation without adjustments` | `adjusted_items` の一覧を出す |
| U9 | 「今の数を入れる」の入口（`/stocktake/` の search の `product` と `returnTo`。73）: `product` の商品の欄を開き、fresh な計数を保存した後に `returnTo` へ戻る。`returnTo` は `/products/{product と同じ商品コード}/edit` だけを受け、外れれば（`/settings`・別の商品の `/products/{他の商品コード}/edit`・無い）保存の後も `/stocktake/` に留まる（round 3 の是正） | 戻り先の誤り | component / route | `U9 count_now entry saves fresh and returns`、`U9 returnTo outside the product's edit form stays on stocktake` | `returnTo` を捨てる、許す先を検証しない（`/settings` へ移る）、別の商品の form へ戻る、保存の失敗でも戻る |
| U10 | Z004 の preview の拒否（BIZ の文の Alert、「ファイルを選び直す」）、知らせ、同じ日の追加の確認の dialog を出さない | 確認の残り | component | `PreviewStep.test.tsx` の `U10 z004 preview shows notices without additional confirm` | dialog を出す |
| U11 | 取込みの結果と取消の結果の要再確認の一覧（商品・範囲〈単位どおり〉・理由。mock の範囲は 55 の例の「4〜5」と違う値にする）と棚卸しの画面への導線（範囲は round 1 の是正） | 導線の漏れ、範囲が出ない | component | `ResultStep.test.tsx` の `U11 import result lists recheck targets with range and recount link` | 一覧を出さない、範囲を出さない |
| U12 | 商品フォーム: 共有 JAN の field 付きの拒否を欄の近くに、`recount_required` で文と「今の数を入れる」の導線（`active_count`・`count_now` のどちらも `/stocktake/` へ `product` と `returnTo`〈`/products/{商品コード}/edit`〉）、入力を保ち toast・遷移をしない。導線は離脱の確認を通る（round 3 の是正。51）: 拒否の後も在庫連動の checkbox を変えたままなので dirty で、導線の遷移が block される（「破棄して移動」で進み、「編集を続ける」で留まって入力を保つ） | 回復の導線の漏れ、guard の迂回 | component | `ProductFormPage.test.tsx` の `U12 stock link start rejection shows recovery link`、`ProductFormPage.unsaved-guard.test.tsx` の `U12 recovery link passes through the unsaved-changes guard` | 遷移する、message を解析する、導線が guard を迂回する（遷移の `ignoreBlocker`、遷移の前に baseline を提出値へ揃える）、search の key を誤る |
| U13 | 棚卸しの記録の詳細: 明細の数・差異・評価数量・ロス原価（BIZ の文字列）、完了の後の差（確定と要再確認を分け合計）、「在庫変動」の節が無い | 旧い補正の表示 | component | `StocktakeRecordDetailPage.test.tsx` の `U13 record detail shows counts and after-completion diff` | movements の節を残す、ロス原価を画面で掛ける |
| U14 | 整合性の画面の共有 JAN の一覧（文・JAN ごとの商品コード、補正の操作を持たない、差異なしと別の欄） | 一覧の漏れ | component | `IntegrityCheckPage.test.tsx` の `U14 shared jan conflicts are listed separately` | 差異の一覧に混ぜる |
| U15 | UI_TECH_STACK の D-118 の段落の経路と entry（round 2 の是正）: D-052 の SSOT の 12 の entry（`stocktakeStart`・`stocktakeCountUpdate`・`stocktakeComplete`・`csvImportCommit`・`csvImportRollback`・`receiving`・`returnExchange(false)`・`manualSale`・`disposal`・`productUpdate`・`productImport`・`integrityFix`）が要再確認の一覧の key `queryKeys.stocktake.recheckList()` を持ち、`stocktakeCountUpdate`（計数の保存と「0 にする」）が写しの consumer（商品一覧・在庫少・在庫照会・在庫変動・商品フォーム・棚卸しの明細）を持つ。ほかの entry（`productCreate`・`returnExchange(true)`・`dailyReportImport` ほか）は要再確認の一覧を持たない | 古い写し・古い回復先が残る | unit / component | `src/lib/invalidation-contract.meta.test.ts` の今の case（contract と、UI_TECH_STACK の段落から独立に写した oracle の 22 entry の完全一致）、今の oracle の完全一致の case（`ReceivingPage.test.tsx`・`ReturnExchangePage.test.tsx`・`ManualSalePage.test.tsx`・`DisposalPage.test.tsx`・`ProductFormPage.test.tsx`・`ProductImportPage.test.tsx`・`IntegrityCheckPage.test.tsx`・`useCsvImportFlow.test.tsx`）、`StocktakePage.test.tsx` の `U15 count save and zero use the stocktakeCountUpdate entry`（保存と「0 にする」の実呼出しの集合が oracle の `stocktakeCountUpdate()` と完全一致） | contract か oracle の片方だけに key を足す・落とす（meta test）、hook が contract を通らずに直に invalidate する（`invalidation-contract.static.test.ts`）、「0 にする」が別の entry を使う。contract と oracle の両方で同じ entry を落とす誤りは test で観測できないので、oracle が UI_TECH_STACK の段落の 12 と一致するかを Final Review の code 読みの項目にした（Residual Test Gaps） |
| U16 | 停止の案内が無く、開始・数の保存・完了・Z004 の「取り込む」が押せる | 停止の UI の残り | component | `U16 stocktake and z004 writes are enabled`（棚卸しと Z004 の 2 file） | 停止の prop や Alert を残す |

### E: demo の seed の reset（round 2 の是正）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| E1 | `seed_demo::delete_all`（`--reset`）が `stock_counts` を先に消し、計数のある DB でも reset が通る（`stock_counts` は `products` と `stocktake_items` を外部キーで参照する） | 計数のある開発 DB の reset が外部キーの失敗で止まる | integration | `src-tauri/tests/seed_test.rs` の `seed_reset_req205_clears_stock_counts`（seed の後に棚卸しの明細を参照する計数の行を入れ、`run_seed(&mut conn, true)` が通り、`stock_counts` が 0 行） | `delete_all` の表に `stock_counts` を足さない、`stocktake_items` か `products` より後に消す |

### F: L3 の fixture の生成（round 1 の是正）

| ID | Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|---|
| F1 | 固定の T（`2026-01-15`）で packet の Test Plan の「L3 の fixture の表」の 9 file を一時の folder に作り、Z004 の 7 つを `parse_z004` で読み戻すと、マシンNo. `0001`・精算回数・精算日（T からの相対）・行（JAN・数量・金額）が表と同じで、商品の CSV の 2 つを `parse_product_csv` で読み戻すと P1〜P6 の行（と上書きの行）が表と同じ。env の `INVENTORY_L3_DAY` と `INVENTORY_L3_OUT` で書く `#[ignore]` の test は同じ関数と同じ読み戻しを使う | L3 の当日に file が取り込めない、表と違う中身で台本の期待が外れる | unit | `test_l3_fixtures_req401_req104_generated_files_parse_back`（`src-tauri/src/io/l3_fixtures.rs`。手元の `#[ignore]` は `write_b1_l3_fixtures`） | 精算回数のメタ行の形を誤る（`parse_z004` の `settlement_metadata` が None）、精算日を T からずらし損ねる、CP932 で書かない、JAN の check digit を誤る |

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 計数の保存（既定・fresh） | 数量欄は空、日付は今日 | 保存中は商品の切替・離脱を抑止 | 写しと明細の写しが変わり feedback | 写しの consumer（U15） | 一覧・要再確認の一覧 | 同じ商品をもう一度検索すると前の数が読取りで出る | 再起動の後も DB の計数が効く（UI の draft を復活させない） | 拒否は文と field、入力を保つ | 入力を直して保存 | C1〜C8・U1〜U6・U15 |
| 要再確認 | 写しの印 0 | — | 入れ直しで解ける | 保存・取込み・取消の後 | 一覧 | 取込みの結果と棚卸しの画面の両方 | DB の写しが正 | — | 入れ直し | L5・L6・C9・U6・U11 |
| 完了 | 進行中 | 完了中は全操作を無効 | 評価額の固定、結果の表示 | 進行中・前回の要約・記録の一覧 | 前回の要約（完了の前の snapshot を結果に渡す今の UI-10-D5 を保つ） | 記録の詳細で完了の後の差 | 完了した記録は変わらない | `completion_blocked` は理由の一覧、範囲の外は文 | 理由を片付けて再び完了 | K1〜K5・R1〜R5・U7・U8 |
| Z004 の取込み | ファイル選択 | preview → 確定中は離脱を抑止 | 結果と要再確認の一覧 | 写しの consumer・取込みの一覧 | 結果の後の一覧 | 取込みの記録の詳細 | token は 30 分で消える | 拒否の文、token を残す | ファイルを選び直す | Z1〜Z16・Z21〜Z23・U10・U11 |
| Z004 の取消 | 取込みの記録 | 取消中 | 結果と要再確認の一覧 | 同上 | 同上 | 取り込み直し（同じ精算回数の file。fresh な計数にとって保存の後の取込みで判定の n の算出に入らず、判定の n だった精算なら日付で判定する。L13・K2・Z24） | — | 失敗の文 | — | Z14・U11 |
| 在庫連動を始める | 連動 off の商品 | 保存中 | 連動 on | 商品の consumer | — | 回復先へは離脱の確認を通って移り（未保存の入力を破棄）、`returnTo` で戻った商品フォームは DB の値で開く（round 3 の是正） | — | `recount_required` と導線 | 数を入れて保存し直す | J4〜J6・U12・U9 |
| 商品一括 import の確定（round 2 の是正） | preview（error 行・上書きの選択） | 確定中は操作を無効（UI-01c-D12） | 結果の件数 | 商品の consumer（`productImport`） | 一覧 | — | — | 確定の TX で当たった行は行を名指した Alert、何も書かない | 「戻る」で preview と選択に戻り、その行をスキップにするかファイルを選び直す | J3・J5 |
| migration vZ | 直前の版の DB | 起動中 | 版 vZ | — | — | — | 止まった DB は作り直し | `MigrationFailed`（作り直しの文） | 作り直しの後に起動 | M1〜M7 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| 停止の入口（SPEC-STOP-D1〜D4） | `rg -l '一時停止中' src src-tauri/src --glob '!src/lib/bindings.ts'` の 10 file | 全部を撤去 | — | AC6・U16 |
| 旧本体の呼出し | `rg -c 'legacy_(commit_csv_import\|rollback_csv_import\|start_stocktake\|update_count\|complete_stocktake)\(' src-tauri/src` の 7 file | 全部を新しい経路か撤去へ（T2） | — | AC6・AC10 |
| IME の確定の Enter の guard（UI-10-D2 の追記 2） | `rg -n 'isComposing' src/features/stocktake src/features/receiving` | 新しい数量欄・日付の欄・入れ直しの欄 | — | U1 と既存の T23 を移す |
| 数量の空欄の FieldError（ST-C5-D1） | `StocktakePage.test.tsx` の T-C5-1〜4 | 「数を保存」「入れ直す」の両方 | — | T5 の移す行 |
| m の入力（SPEC-UNIT-D3 の `parseLengthInput` の系） | `rg -n 'parseRequired|parseLength|unit-quantity' src/features/receiving src/features/disposal src/lib` | 棚卸しの実数の欄 | — | U4 |
| query invalidation（D-052 の SSOT。round 2 の是正で source を直した） | `src/lib/invalidation-contract.ts` の全 22 entry、`src/lib/query-keys.ts`、独立の oracle `src/test/invalidation-oracle.ts`、`src/lib/invalidation-contract.meta.test.ts`（contract と oracle の完全一致）、`src/lib/invalidation-contract.static.test.ts`（hook からの直の `invalidateQueries` を拒む。許すのは `src/features/stocktake/stocktake-error-invalidation.ts` ほかの防御の helper だけ） | 写しを書く経路と、要再確認の一覧の回復先を変える開始と完了の 12 entry に `queryKeys.stocktake.recheckList()`、`stocktakeCountUpdate` に写しの consumer（UI_TECH_STACK の D-118 の段落） | `productCreate`（新しい商品は計数を持たず要再確認にならない）、`returnExchange(true)`（movement を作らない）、`dailyReportImport`・`thresholdSave`・PLU・価格改定・取引先の entry（写しも棚卸しも変えない）。`stocktakeComplete` の今の写しの consumer は外さない（値を変えない再取得で、誤った表示を生まない） | U15 |
| 一括 import の確定の TX の再検証（BIZ-01-D8 の単位の再検証、D-122 の J3・J4・J7。round 2 の是正） | `src-tauri/src/biz/product_service.rs` の `commit_import` の `import_row_commit_error`、30 §4.9.2 の判定表 | 共有 JAN の不変条件と在庫連動を始める検査（行を名指して確定全体を戻す同じ形） | 当たった行だけを飛ばす形（結果の型と画面を作り直すことになる。30 の D-118 の節） | J3・J5 |
| `CmdErrorKind` の網羅（`CMD_ERROR_KIND` の `satisfies`、`describeError`、`BackupRestorePage` の kind の分岐） | `rg -n 'restore_durability_unknown' src src-tauri/src --glob '!src/lib/bindings.ts'` | `invoke.ts`・`describe-error.ts` | `BackupRestorePage` は restore の kind だけを分ける（`stocktake_guard` を受けない画面なので足さない） | W7 |
| route search の `returnTo`（UI-01b-D2 ほか） | `rg -n 'returnTo' src/routes` | `/stocktake/` の search（許す先は `/products/{product}/edit` だけ。round 3 の是正） | — | U9 |
| 離脱の確認（UI-USW-D1・D2。round 3 の是正） | `rg -n 'useUnsavedChangesWarning' src/features` | 商品フォームの「今の数を入れる」の導線（今の guard をそのまま通す） | 棚卸しの画面（UI-USW-D3 (c) の即時の DB 保存で guard の対象外のまま） | U12 |
| env で folder を受ける手元の `#[ignore]` の test（round 1 の是正） | `rg -n '#\[ignore' src-tauri/src`（`ej_parser.rs` の `real_ej_structure_probe`） | `io/l3_fixtures.rs` の `write_b1_l3_fixtures` | 読み戻しの確かめは通常の test（F1）にも置き、CI で生成の誤りを止める（手元の test だけに頼らない） | F1・AC12 |
| 生成の enum（D-061） | `src-tauri/src/finite_enum_contract_tests.rs` の一覧 | 新しい 5 つ | — | W6 |
| 写しを求め直す書く経路（(a) の Y2） | `rg -n 'recompute_stock\(' src-tauri/src --glob '!*test*'` | 計数の保存・0 にする・Z004 の確定と取消・連動の切替 | 完了（在庫を動かさない） | L12・C1・Z8・Z14・J6 |

## Negative Paths

- missing input: 精算回数・マシンNo. の無い Z004（Z1）、計数の値の空欄（U1 の FieldError）。
- invalid input: 形の違う日付・実在しない日付・未来の日付（C3）、範囲の外の計数（C4）、範囲の外の精算回数（Z1）、× 100 で溢れる長さの行（Z12）。
- duplicate/ambiguous input: 同じ hash（Z3）、同じ精算の別の版（Z4）、共有 JAN の行（Z10）、重なる計数の値の違い（L10 の衝突）、取消の後の別のレジと番号の戻り（Z22・Z23。round 3 の是正）、取消の後の同じ番号の精算（Z24・L13・J7・R6。Codex の closure の Gated Amendment）。
- unknown reference: 別の商品の明細・進行中でない明細・存在しない明細（C5）、棚卸しの明細の無い商品の「0 にする」（C8）、preview の後に消えた商品（Z13）。
- stale preview: 商品一括 import の preview の後の DB の変化と、上書きを選んだ行どうしの JAN の重なり（J3・J5。round 2 の是正）。
- dependency missing: 未取込みの精算（K2）、計数の無い商品の在庫連動の開始（J4）。
- permission/write failure: 在庫の上限・評価額の範囲で TX 全体が戻る（K5・Z16）、vZ の確かめの失敗（M3）。
- dry-run side effect: preview（`parse_and_validate`）と `preview_stock_count_placement`・`list_recheck_products`・`derive_stock` は書かない（Z6・C7・C9 で DB の行数が変わらないことを確かめる）。

## Boundary Checks

- threshold: 計数 `999999999` と `1000000000`（C4）、精算回数 `9007199254740991` と `9007199254740992`（Z1）、長さの行 `99999`（× 100 で `9999900`）と `100000`（Z12）、評価額の総額 9 個と 10 個（K5）。
- null/default: fresh の `settlement_no` が NULL（L4・C2）と、判定の n が NULL（c が NULL、または c 以下の active な取込みが無い。L13・K2・Z24）、`seen_import_id`・`cutoff_import_id` の NULL（C2・K1・M6・R6）、`seen_movement_id` 0、`stock_linked` の既定 0（M6）、カーソル NULL（K1）と movement のカーソル 0（K1）。
- empty/non-empty: 精算の取込みが無い DB（L4・K2 の (i)(iii)）、正常な行が全て 0 の file（Z9）、要再確認が 0 の一覧。
- min/max: 欠けの区間の端（Z6）、範囲の最小と最大（L5）。
- status/policy enum: `count_kind`・`block_reason`・`action`・`code`・`recount_reasons`（W6・W8）。
- wire type: 数は number（安全な整数）、読取りの金額は十進の文字列（R1・R2）。
- internal type: `RecheckReason` と `RecountReason` の順（L10・W6）。
- producer/consumer: CMD の今日（W3）、画面の日付の欄の ISO（U2）。
- round-trip token: preview token（W5）。
- precision/range: m の入力の 2 桁（U4）、行のロス原価の基準数量（R1）。
- cross-language parse: `CmdErrorKind` の TS の twin（W7）。

## Compatibility Checks

- old schema/input: 直前の版の DB（M2）、従来の形の Z004（Z1。今の tracked の fixture `tests/fixtures/z004/normal-small.csv` ほかを拒否の入力に使う）。
- new schema/input: layout A の精算の識別メタを持つ Z004（test の helper `make_z004_bytes` を精算回数つきに）。
- output order: 欠けの区間の昇順、共有 JAN の一覧の JAN と商品コードの昇順、`recount_reasons` の順。
- optional field behavior: `stocktake_recovery` は既存の kind で null、`after_completion` は完了していなければ null。

## Data Safety Checks

- source-derived data: 無い（実データを使わない）。
- generated outputs: `src/lib/bindings.ts`、`docs/function-design/90-traceability.md`。
- secrets: 無い。
- local-only files: owner の L3 用の DB（L3-1 の前と後に消さずに移し、L3-1 は Coordinator が main の build の demo の seed で作った合成の DB、L3-2 以降は空の DB で行う。packet の Test Plan の「L3 の DB の用意と差し替え」と Data Safety。round 2 の是正）。
- synthetic sample boundaries: `tests/fixtures/z004/` の合成の Z004（CP932・CRLF、マシンNo. `0001`）と商品の CSV、Rust・TS の fixture。

## Main Wiring / Integration Checks

- helper connected to main path: 計数の保存・完了・Z004 の確定と取消が `recompute_stock` を通る（C1・Z8・Z14）。CMD が BIZ を呼ぶ（W2〜W5）。
- output reaches manifest/report: 取込みの結果の `recheck_targets` と知らせが画面に出る（U10・U11）。
- effective config reaches runtime: 在庫連動の設定が確定の TX で `stock_linked` に写る（Z8・L6）。
- CLI arg reaches implementation: 該当なし。

## Mutation-style Adequacy Questions

- If a mock value is changed so it differs from the design-doc expected value, which assertion proves the implementation used the correct source and not the mock's accidental constant? → 画面の test の mock の在庫・範囲・理由は設計正本の例（「4〜5」・理由の文）と違う値にし、表示がその mock の値を出すことを見る（U6・U11）。BIZ の test は fixture の記録から期待を手で計算し、`derive_stock` の結果と比べる（L・C・K）。
- If invalidate/refetch changes the value before versus after the operation, which test proves the lifecycle order and preserved snapshot are correct? → U15、完了の結果の前回の要約（UI-10-D5 の snapshot を保つ既存の T19 を移す）。
- If a key branch is inverted, which test fails? → A1（L1）、`k ≤ n`（L3）、`stock_linked`（L6）、廃番 0 の例外（K4）。
- If a threshold comparison changes, which test fails? → C4・Z1・Z12・K5。
- If a guard is removed, which test fails? → C3・C5・C8・Z13・J2・J3（一括 import の確定の再検証）・M3・Z22（取消の後の順 2）・U12（離脱の確認）・L13・Z24・J7・R6（取込みのカーソル）。
- If an output field is omitted, which test fails? → C1（結果の field）、Z8（`recheck_targets`）、W1（`stocktake_recovery`）。
- If output order changes, which test fails? → Z6・I1・L10。
- If dry-run performs a side effect, which test fails? → Z6・C7・C9 の行数の確かめ。
- If a JSON number crosses JavaScript safe integer range, which test fails? → Z1（精算回数）、K5（総額）、R2（読取りの金額は文字列）。
- If a state token is round-tripped through browser/client code, which test fails? → W5（preview token）、U9（`returnTo`）。

## 既存の test の移し方

着手時に下の command を逐語で回し、表と違う所があれば表を直してから進める（2026-10-22 の HEAD `c98fe7dd` で測った）。分類は 3 つ: **移す**（契約が新しい経路に残る。呼び先と fixture を新しい経路へ替え、値の期待〈`assert` の右辺・件数・message〉を変えない）、**期待を変える**（設計正本の契約の変更。正本の節を書く）、**契約ごと外す**（ADR・owner の決定で契約が置き換わった。置き換えた節と、同じ失敗を止める Matrix の行を書く）。

### T1 最新の版の期待

round 2 の是正で、`init_database`・`migration::migrate` で最新の版まで当てて `(VB_VERSION, VB_VERSION)` か `(10, 10)` を期待する test を拾えるよう command を広げた（round 1 の command は `lib.rs` の 2 行と `migration.rs` の 2 行だけを拾った。広げた command は round 1 の 2 つの式も含む）。2026-10-22 の HEAD `e16b9eda`（code は main `c98fe7dd` と同じ）で測った:

```
$ rg -n '\(VB_VERSION, VB_VERSION\)|\(10, 10\)|version_and_products\(&conn\), \(crate::db::VB_VERSION, 0\)' src-tauri/src
src-tauri/src/db/schema_v9.rs:471:        assert_eq!(version(&conn), (10, 10));
src-tauri/src/lib.rs:1215:        assert_eq!(version_and_products(&conn), (crate::db::VB_VERSION, 0));
src-tauri/src/lib.rs:1249:        assert_eq!(version_and_products(&conn), (crate::db::VB_VERSION, 0));
src-tauri/src/db/schema_v10.rs:390:        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
src-tauri/src/db/schema_v10.rs:441:        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
src-tauri/src/db/schema_v10.rs:569:        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
src-tauri/src/db/migration.rs:910:        assert_eq!((max_version, version_count), (10, 10));
src-tauri/src/db/migration.rs:1548:        assert_eq!(max_version_and_count(&conn), (10, 10));
src-tauri/src/db/migration.rs:1687:            assert_eq!(max_version_and_count(&conn), (10, 10));
exit=0
```

| hit（test） | 分類 | 理由 / 直し方 |
|---|---|---|
| `lib.rs` の `test_startup_req903_rebuild_moves_app_data_and_legacy_db_then_empty_vb`・`test_startup_req903_rebuild_with_committed_manifest_writes_no_restore_log` | 期待を変える | 22 §20: 最新の版が vZ になる。「作り直しの後の空の DB は最新の版まで当たる」の意味なので最新の版 `app_max_version()` と比べる（vB の定数のままにしない。下の注） |
| `migration.rs` の `test_migration_req903_v8_fresh_database_and_empty_tables`・`test_migration_req903_v8_commit_and_fk_restore_failures`（1548・1687） | 期待を変える | 同上（最新の版の期待を `app_max_version()` へ） |
| `migration.rs` の `test_migration_req907_v5_seeds_all_plu_slots`（910。round 2 で足した） | 期待を変える | `init_database` の後の最新の版の期待を `app_max_version()` へ（PLU の枠の期待は変えない） |
| `schema_v9.rs` の `test_migration_req903_vl_fresh_database_has_keys_counts_and_copy_columns`（471。round 2 で足した） | 期待を変える | `init_database` の後の最新の版の期待を `app_max_version()` へ。列の期待（`inventory_movements` の 13 列、`products` の写しの 3 列）は vZ が変えない表と列なので変えない。`stock_counts` は vZ が `seen_import_id` を足すので、v9 の 9 列を先頭にこの順で持つ形にする（足した列は M1 が見る。Codex の closure の Gated Amendment で直した） |
| `schema_v9.rs` の `test_migration_req904_vl_fixes_copy_and_logs_trigger_migration`（Codex の closure の Gated Amendment で足した） | 移す | v9 の DB で最新の `list_stock_counts_for_product` を呼んでいた（vZ の `seen_import_id` を読むので v9 の DB に当てられない）。v9 の DB に計数が無いこと（`stock_counts` が 0 行。計数の保存の経路は b1）を確かめてから、精算の一覧と同じく計数を空にして求め直す。値の期待（写しと求め直しが同じ）は変えない |
| `schema_v10.rs` の `test_migration_req903_vb_fresh_database_renames_cost_columns`（390。round 2 で足した） | 期待を変える | `setup_latest`（`init_database`）の後の最新の版の期待を `app_max_version()` へ。原価の 6 列の期待は vZ が触らないので変えない |
| `schema_v10.rs` の `test_migration_req903_vb_rebuild_keeps_columns_indexes_and_fk`（441。round 2 で足した） | 移す | `migrate()` の後に全 object の SQL（`schema_objects`）を vB の前の写しと比べるので、vZ まで当てると `inventory_movements` の CHECK と 4 つの表の列で外れる。22 §19 の先例（vB を足したときに同じ型の test を版の上限を取る migrate へ替えた）どおり `migration::migrate_up_to(&conn, VB_VERSION)` に替え、期待は変えない |
| `schema_v10.rs` の `test_migration_req903_vb_verify_or_commit_failure_rolls_back`（569。round 2 で足した） | 移す | 末尾の「失敗を外せば同じ DB で当たる」の `migrate()` を `migrate_up_to(&conn, VB_VERSION)` に替え、`(VB_VERSION, VB_VERSION)` の期待は変えない（vB の失敗の注入と巻き戻しの期待はそのまま） |

上の command の形（`(10, 10)` の組）に当たらず、実装の 1 段目で red になった最新の版の期待（Codex の closure の Gated Amendment で足した。実装の 1 段目の HEAD `1df2906d` で測った）:

```
$ rg -n 'assert_eq!\((v1|v|version|max_version), 10\)' src-tauri/src
src-tauri/src/db/schema_v2.rs:378:        assert_eq!(max_version, 10);
src-tauri/src/db/schema_v2.rs:586:        assert_eq!(v, 10);
src-tauri/src/db/migration.rs:769:        assert_eq!(v1, 10);
src-tauri/src/db/migration.rs:858:        assert_eq!(version, 10);
src-tauri/src/db/migration.rs:941:        assert_eq!(max_version, 10);
exit=0
```

| hit（test） | 分類 | 理由 / 直し方 |
|---|---|---|
| `migration.rs` の `test_migration_req903_is_idempotent`（769）・`test_migration_req401_v4_creates_daily_report_tables_and_indexes`（858）・`test_migration_req907_upgrades_v4_database_and_seeds_all_plu_slots`（941） | 期待を変える | `init_database` の後の最新の版の期待を `app_max_version()` へ（22 §20: 最新の版が vZ になる。日報の表・PLU の枠の期待は変えない）。`test_migration_req903_is_idempotent` は同じ test の 2 回目の版を 1 回目の版と比べ（「変わってはいけない」の意味のまま）、`schema_versions` の件数 10 を `app_max_version()`（v1 から最新の版まで 1 件ずつ）にする |
| `schema_v2.rs` の `test_v2_req903_applied_on_fresh_db`（378）・`test_v2_req903_idempotent_rerun`（586） | 期待を変える | 同上（`idempotency_key` の列の期待は変えない）。`test_v2_req903_idempotent_rerun` の 2 回目の版と `schema_versions` の件数 10 も `app_max_version()` にする |

注（Codex の closure の Gated Amendment で直し方を替えた）: 最新の版を期待する test は、版の定数（`VZ_VERSION`）でなく `app_max_version()`（`src-tauri/src/db/migration.rs`。`migrations()` の最大の版）と比べる。理由: これらの test の意味は「最新の版まで当たる」で、版の定数にすると版を足すたびに直す（並走の PR #172 の §17 の migration も版を足し、同じ形を採った）。先例は時点証拠の schema の test（`schema_time_evidence.rs`。S4 で撤去）の `app_max_version()`。vZ が当たること自体は M1〜M9 が `migrate_up_to(&conn, VZ_VERSION)` と版の定数で見る。

`migrate_up_to` で版を指定する他の test と、`app_max_version()` を引く test（`migration.rs` 5・`test_support.rs` 3。`schema_time_evidence.rs` の 3 は撤去）は変えない。

### T2 旧本体・停止の test

```
$ rg -c 'legacy_(commit_csv_import|rollback_csv_import|start_stocktake|update_count|complete_stocktake)\(' src-tauri/src
src-tauri/src/biz/csv_import_service/commit.rs:1
src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs:15
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:17
src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs:10
src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs:5
src-tauri/src/biz/csv_import_service/rollback.rs:1
src-tauri/src/biz/stocktake_service.rs:37
exit=0
$ rg -c '#\[test\]' src-tauri/src/biz/csv_import_service/tests/commit_tests.rs src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs src-tauri/src/biz/csv_import_service/tests/parse_tests.rs src-tauri/src/biz/csv_import_service/tests/list_tests.rs src-tauri/src/biz/stocktake_service.rs src-tauri/src/cmd/stocktake_cmd.rs src-tauri/src/cmd/csv_import_cmd.rs
src-tauri/src/cmd/csv_import_cmd.rs:4
src-tauri/src/cmd/stocktake_cmd.rs:10
src-tauri/src/biz/stocktake_service.rs:40
src-tauri/src/biz/csv_import_service/tests/list_tests.rs:6
src-tauri/src/biz/csv_import_service/tests/parse_tests.rs:22
src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs:7
src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs:8
src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs:5
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:15
exit=0
```

`commit_tests.rs`（15）:

| test | 分類 | 理由 / 代わりの行 |
|---|---|---|
| `test_commit_req401_normal_flow`、`test_commit_req401_toctou_check`、`test_commit_req401_pos_stock_sync_false`、`test_commit_req401_negative_stock_warning`、`test_commit_req401_partial_with_errors`、`test_commit_req401_sign_flip_inv1`、`test_preview_req401_pos_quantity_limit_is_row_error`、`test_commit_req401_stock_limit_rolls_back_whole_tx` | 移す | 売上・行の error・状態・同じ hash の再検査・連動 off・符号・上限は 32 の新契約に残る。fixture の Z004 に精算回数を足す（Z3・Z11・Z16） |
| `test_commit_req401_same_day_additional_import_is_insert_only`、`test_commit_req401_additional_import_requires_confirmation`、`test_commit_req401_same_date_snapshot_removal_requires_repreview`、`test_commit_req401_same_date_snapshot_replace_requires_repreview`、`test_commit_req401_same_date_snapshot_order_requires_exact_match`、`test_commit_req401_additional_confirmation_without_duplicate`、`test_commit_req401_settlement_date_toctou` | 契約ごと外す | ADR D4 の「同じ精算日の別の精算は確認なしに別の取込み」（D-071 の Z004 の側の置換え）。`test_commit_req401_settlement_date_toctou` も同じ日の snapshot の照合（preview の後に別の経路で同じ日の取込みが入る。test の comment は SPEC-SDI-D4）。同じ精算の二重は順 4、確定の TX の取り直しは Z4・Z7・Z15 |

`rollback_tests.rs`（8）:

| test | 分類 | 理由 / 代わりの行 |
|---|---|---|
| `test_rollback_req401_normal`、`test_rollback_req401_idempotent`（どちらも assert の単位で分ける。下の「assert の単位で分けた test」）、`test_rollback_req401_not_found`、`test_rollback_req401_same_day_is_per_import_with_negative_return`、`test_rollback_req401_selected_negative_return_reverses_only_return_contribution`、`test_import_rollback_req401_logs_exact_selected_id_and_preserves_commit_on_log_failure`、`test_rollback_req401_stock_limit_rolls_back_whole_tx` | 移す | 取込みごとの取消・冪等・操作ログ・上限は残る。在庫の期待は求め直しの値（計数の無い fixture では今と同じ和） |
| `test_rollback_req401_stock_corrections` | 契約ごと外す | ADR D7「補償の movement は作らない」、`StockCorrection` を外す。Z14 |

`cross_feature_tests.rs`（5）: `test_cross_feature_req201_req202_req203_req204_req401_model`・`test_cross_feature_req904_req203_model_detects_consistent_corruption` は移す（X3。fixture の Z004 に精算回数を足し、`commit_bytes` の同じ日の追加の確認の分岐〈`duplicate_check`〉を外す。model の期待は変えない）。`test_cross_feature_req205_controls` は期待を変える（round 2 の是正で「移す」から直した。共有の `temporal_trace` が旧本体の `force_fill: true` と「完了が写しを上書きする」前提を持ち、新しい完了は締めの精算を待ち同じ日の記録で要再確認にするので、4 つの対照を X の節の「trace の新しい形」で書き直す。「oracle を破らない」の期待は保つ。ADR D3・D6、35）。診断 2 本は `#[ignore]` を外して改名し、新しい経路で PASS する回帰 test にする（X1・X2。trace は同じ形で書き直す。ADR D12・停止 ADR SPEC-STOP-D3 の「診断 2 本は新しい経路で PASS する回帰 test に変える」）。

`legacy_stop_tests.rs`（7、file ごと撤去）: 停止の assert（5 つの入口が停止 error・DB の全表が不変）は契約ごと外す（停止 ADR を superseded にする。SPEC-STOP-D6 の解除）。F1〜F7 の入力は新しい経路の test の入力に移す: F1（STK-1）→ X1、F2（STK-2）→ X2、F3（旧い形の明細と取込み）→ M3（vZ の止まり。v9 で既に止まる形）、F4（全て 0 の行）→ Z9、F5（共有 JAN）→ Z10、F6（逆順）→ Z5、F7（欠けた日）→ Z6・L11。

`stocktake_service.rs`（40）:

完了を通す test（下の表の `test_complete_req205_already_completed`・`total_cost_` の 5 本・`test_complete_stocktake_req205_includes_integrity_result`・`test_complete_req205_negative_cost_rolls_back`・`test_complete_req205_operation_log`）は、共有の helper `start_and_count_all`（今は `UPDATE stocktake_items SET actual_count` で入れる）を新しい経路の fixture に替える: 明細ごとに `save_stock_count`（既定、日付 D、`stocktake_item_id` つき）で計数を保存し、35 の完了の (i)(iii) を満たす正常な行が全て 0 の Z004 を 2 本（精算日 D−1 と D）取り込む。新しい完了は精算の取込みが無いと `settlement_pending` で止まり、明細を参照する計数が無いと `uncounted` で止まるので、helper を替えないと値の期待を比べる前に止まる（round 2 の是正。値の期待は変えない）。

| test | 分類 | 理由 / 代わりの行 |
|---|---|---|
| `test_get_active_stocktake_req205_*`（2）、`test_find_stocktake_item_req205_*`（2）、`test_get_last_completed_stocktake_req205_*`（2）、`test_get_progress_req205_normal`、`test_get_progress_req205_not_found`、`test_get_stocktake_record_req206_not_found_message`、`test_get_stocktake_record_req206_status_enum_and_fail_fast` | 移す | 読取りの契約は残る（進捗の計数済みの定義は C10。fixture は計数を入れる） |
| `test_start_stocktake_req205_normal`（assert の単位で分ける。下の表）、`test_start_stocktake_req205_already_in_progress`、`test_start_stocktake_req205_no_eligible`、`test_start_stocktake_req205_operation_log`（今の summary の件数の assert は変えず、`detail_json` の key が `stocktake_id`・`item_count` の 2 つちょうどの assert を足す。round 3 の是正） | 移す | 開始の契約は残る（K7・K8） |
| `test_start_stocktake_req205_auto_fill_discontinued` | 契約ごと外す | tracking「自動の 0 の入力を持たない」。廃番かつ写し 0 は完了を止めない（K4） |
| `test_update_count_req205_normal`・`_negative`・`_zero_is_valid`・`_item_not_found`・`_not_in_progress`・`_dynamic_difference` | 契約ごと外す | `update_count` は 42 の新契約で外す。同じ失敗は C1・C4・C5・C10 が止める |
| `test_complete_req205_not_found`、`test_complete_req205_already_completed`、`test_complete_req205_total_cost_multiple_products`、`test_complete_req205_total_cost_overflow`、`test_complete_req205_total_cost_length_product_per_meter`、`test_complete_req205_total_cost_rounds_half_up_at_yen`、`test_complete_req205_total_cost_yen_overflow`、`test_complete_stocktake_req205_includes_integrity_result` | 移す | 評価額（§20.5a）・状態・整合性の結果は残る。評価数量は `max(写し, 0)` で、fixture の計数を保存した後の写しが今の `actual_count` と同じなので値は同じ（K1・K6・K8）。`length_product_per_meter` の detail の `total_cost` の assert は 35 の「操作ログ」に残る key |
| `test_complete_req205_normal_all_counted` | 契約ごと外す（round 2 の是正で「移す」から直した） | assert の 3 行が全部 `adjusted_items`（外す出力）を見る。ADR D6「完了は在庫を動かさず補正の一覧を返さない」。同じ失敗（差のある明細を取り違える）は R1（記録の詳細の明細の差異。差異は 35 の「効く計数 − 帳簿」なので、この fixture では −2）と C10 が止める |
| `test_complete_req205_operation_log` | 期待を変える（round 2 の是正） | 35 の「操作ログ」: detail の `adjusted_count` を `difference_count` に、summary の差異の件数を差異のある明細の数にした。`stocktake_id`・`total_cost` の assert は残る（K8） |
| `test_complete_req205_negative_cost_rolls_back` | 期待を変える（round 2 の是正） | 「在庫は確定前のまま」の `[10, 10]` は、計数 7 の保存が写しを 7 に求め直す（ADR D5）ので、完了の前に読んだ写し `[7, 7]` を期待にする（意味〈拒否で写しが変わらない〉は保つ）。error の kind と商品コード・header・明細の評価原価の assert は変えない |
| `test_complete_req205_force_fill_true`、`test_complete_req205_force_fill_false_uncounted`、`test_complete_req205_force_fill_sets_actual_to_system_stock`、`test_complete_req205_force_fill_negative_stock_clamped_to_zero` | 契約ごと外す | owner の決定 TD-263 の Q7 = b（force_fill を持たない）、ADR D6。未計数は K4、0 にするは C8 |
| `test_complete_req205_creates_movements`、`test_complete_req205_no_difference`、`test_complete_req205_stock_after_equals_actual` | 契約ごと外す | ADR D6「完了は在庫を動かさない」。K1 が「movement 0・写し不変」を止める |
| `test_get_stocktake_record_req207_movement_source` | 契約ごと外す | 65 の新契約（記録の詳細は補正の movement の節を持たない）。R3 |

`stocktake_cmd.rs`（10）: `test_update_count_req205_negative_stops_before_validation`・`test_update_count_req205_zero_stops`・`test_start_and_complete_stocktake_req205_stop` は契約ごと外す（停止 ADR の撤去。W2〜W4 が新しい command の wire を止める）。残り 7（ページの検証 2・find 2・last completed・active・record の not found）は移す。

`csv_import_cmd.rs`（4）: `test_get_csv_import_record_cmd_req206_not_found_kind` は移す。`test_csv_cmd_req401_parse_cache_preserves_ordered_same_date_ids` は契約ごと外す（同じ日の取込みの一覧を Z004 で使わない。ADR D4）。`test_csv_cmd_req401_commit_stop_keeps_preview_token` は契約ごと外す（assert が停止の error と `additional_import_confirmed` の 3 通りの入力だけを見る。停止 ADR を superseded に。token を残す振舞いは W5 が止める）。`test_csv_cmd_req401_rollback_stop` は期待を変える（T10。停止の assert を外し、取消の CMD が新しい経路を通ることを確かめる形）。

`parse_tests.rs`（22）・`list_tests.rs`（6）: 旧本体を呼ばない。parse の test は fixture に精算回数を足す（従来の形の拒否を確かめる test は従来の形のまま Z1 へ）。`test_parse_and_validate_req401_future_settlement_date`（「未来」の拒否を確かめる）は期待を変える（32: 精算日が今日より後は拒否せず知らせ。Z6）。`test_parse_and_validate_req401_same_date_snapshot_is_ordered`（T10）と `test_parse_and_validate_req401_no_duplicate`（assert が `duplicate_check` の 2 つだけ）は契約ごと外す（ADR D4。`PreviewData.duplicate_check` を外す。同じ失敗は Z3〈同じ hash の拒否〉・Z6〈知らせ〉・Z7〈同じ日の別の精算は確認なし〉が止める）。`test_parse_and_validate_req401_normal_matching` は assert の単位で分ける（下の表）。`test_parse_and_validate_req401_no_valid_data`（正常な行が全て 0 の file を「取込み対象のデータがありません」の `ImportError` で拒むことを見る）は期待を変える（packet の S5「正常な行が全て 0 の file を拒まない」と本 Matrix の Z9、32 の確定の節「今の空の対象の拒否をやめ…精算の識別メタだけを持つ取込みとして確定する」: preview は通り、`matched_rows` が空で、行の error も無い。Codex の closure の Gated Amendment で足した。実装の 1 段目で red になった）。

#### assert の単位で分けた test（round 2 の是正）

「移す」の test のうち、b1 が外す出力（`adjusted_items`・`stock_corrections`・`auto_filled_count`・`MatchedRow.pos_stock_sync`・`PreviewData.duplicate_check`、操作ログの detail の `adjusted_count`・`force_fill_used`）を見る assert を持つ test を、2026-10-22 の HEAD で外す名前を `rg` で引いて洗った（`rg -n 'adjusted_items|auto_filled_count|corrected_count|stock_corrections|adjusted_count|force_fill_used|duplicate_check|DuplicateStatus|pos_stock_sync' src-tauri/src/biz/stocktake_service.rs src-tauri/src/biz/csv_import_service/tests src-tauri/src/cmd/stocktake_cmd.rs src-tauri/src/cmd/csv_import_cmd.rs` の hit を test ごとに読み、入力の literal〈`force_fill: false` ほか〉は T12 の機械的な追随に回した）。外す assert だけの test は上の表で「契約ごと外す」にした（`test_complete_req205_normal_all_counted`・`test_parse_and_validate_req401_no_duplicate`）。残る assert を持つ test は次のとおり分ける。

| test | 移す assert（期待を変えない） | 外す assert と代わりの行 |
|---|---|---|
| `stocktake_service.rs` の `test_start_stocktake_req205_normal` | `stocktake_id > 0`、`item_count` 2、DB の明細の行数 2 | `auto_filled_count` 0（`StartStocktakeResult` から外す。tracking「自動の 0 の入力を持たない」。K7 が「自動の 0 が無い」を止める） |
| `rollback_tests.rs` の `test_rollback_req401_normal` | 取込みの後の在庫 7、`success`、`voided_sale_count` 1、`voided_movement_count` 1、取消の後の在庫 10（計数の無い fixture なので求め直しの値は今の和と同じ） | `stock_corrections` の 4 行（件数・商品コード・前後の在庫。ADR D7「補償の movement を作らない」で `StockCorrection` を外す。Z14 が「補償の movement を作らず求め直す」と結果の `recheck_targets` を止める） |
| `rollback_tests.rs` の `test_rollback_req401_idempotent` | 1 回目の `success`・`voided_sale_count` 1、2 回目の `success`・`voided_sale_count` 0・`voided_movement_count` 0、在庫 10 | `rb2.stock_corrections.is_empty()`（同上。Z14 の取消済みの冪等の Ok） |
| `parse_tests.rs` の `test_parse_and_validate_req401_normal_matching` | `matched_rows` の件数 2・商品コード・数量 3、`matched_summary` の件数 2・金額 1100、`error_summary` の件数 0 | `matched_rows[i].pos_stock_sync` の 2 行（32 の「preview の cache の契約」で `MatchedRow` は `pos_stock_sync` を持たず、確定の TX で取り直す。Z21 の対照〈preview の後の連動の設定の変化は拒否せず `stock_linked` は確定の時点の値〉と Z8・L6 が止める）、`duplicate_check.status` が `NoDuplicate`（ADR D4。Z3・Z7） |

`disposal_repo.rs` の `test_list_inventory_records_req206_stocktake_item_count_diff_only`: 期待を変える（65: 棚卸しの件数は明細の数。R3）。同じ file の `test_list_inventory_records_req206_in_progress_filter_exclusion`（進行中の棚卸しが商品・部門の絞り込みに当たらないことを見る）も期待を変える（65 の D-118 の節「棚卸しの件数は、その棚卸しの明細の数（入庫・廃棄の記録の件数と同じく明細の数。movement からは数えない）」と packet の S2「`list_inventory_records` の棚卸しの `RecordSpec` の件数と絞り込みを、補正の movement から明細へ」: 進行中の棚卸しも明細の商品と部門で絞り込みに当たるので、keyword と部門の絞り込みの件数を 0 から 1 にし、test 名を `test_list_inventory_records_req206_in_progress_filter_matches_items` に改める。65 の §65.4 の旧い「差異 movement が未発生のため hit しない」の文は S9 で直す。Codex の closure の Gated Amendment で足した。実装の 1 段目で red になった）。

### T3 時点証拠の test

```
$ rg -c '#\[test\]' src-tauri/src/db/schema_time_evidence.rs src-tauri/src/biz/stocktake_service/time_evidence.rs src-tauri/src/biz/stocktake_service/time_evidence_tests.rs src-tauri/src/biz/csv_import_service/time_evidence.rs src-tauri/src/biz/csv_import_service/tests/time_evidence_tests.rs src-tauri/src/db/inventory_repo.rs src-tauri/src/db/stocktake_repo.rs src-tauri/src/db/sales_repo.rs
src-tauri/src/db/sales_repo.rs:53
src-tauri/src/db/stocktake_repo.rs:45
src-tauri/src/db/inventory_repo.rs:23
src-tauri/src/biz/csv_import_service/tests/time_evidence_tests.rs:14
src-tauri/src/biz/stocktake_service/time_evidence_tests.rs:19
src-tauri/src/db/schema_time_evidence.rs:8
exit=0
```

- **契約ごと外す**: 時点証拠 ADR（SPEC-STK-TIME-D1〜D10）は D-118 で superseded で、D-109 で `#[cfg(test)]` に置いた code は「runtime の lane が外す」（ADR D12、35・32・21 の D-118 の節）。file ごとの 3 つ（`schema_time_evidence.rs` 8、`stocktake_service/time_evidence_tests.rs` 19、`csv_import_service/tests/time_evidence_tests.rs` 14）と、`inventory_repo.rs`・`stocktake_repo.rs`・`sales_repo.rs` の `mod time_evidence` の中の test（3 file の `#[test]` の数は file 全体の数で、そのうち `mod time_evidence` の中の分だけを外す。着手時に `rg -n 'mod time_evidence' -A` で範囲を確かめる）。同じ失敗を止める行は無い（契約そのものが無くなった）。

### T4 停止の画面の test

```
$ rg -c 'writesSuspended|一時停止中|WRITES_SUSPENDED|IMPORT_SUSPENDED' src
src/features/stocktake/StocktakePage.tsx:8
src/features/stocktake/StocktakePage.suggest.test.tsx:1
src/features/stocktake/StocktakePage.test.tsx:2
src/features/csv-import/CsvImportPage.test.tsx:1
src/features/csv-import/CsvImportPage.tsx:3
src/features/csv-import/components/PreviewStep.tsx:2
exit=0
```

- 停止の案内と無効化を確かめる test は契約ごと外す（SPEC-STOP-D4 の撤去）。U16 が「押せる・案内が無い」を止める。flow の test が渡していた「停止 off」の prop は外す（値の期待は変えない）。

### T5 棚卸しの画面の test

```
$ rg -c '^\s*(it|test)\(' src/features/stocktake/StocktakePage.test.tsx src/features/stocktake/StocktakePage.suggest.test.tsx src/features/stocktake/lib/stocktake-formatters.test.ts
src/features/stocktake/lib/stocktake-formatters.test.ts:10
src/features/stocktake/StocktakePage.suggest.test.tsx:4
src/features/stocktake/StocktakePage.test.tsx:56
exit=0
```

- 移す: UI-10-D1〜D13 の振舞い（開始・継続の表示、検索・HID・候補・名前の検索の fallback・IME の guard・focus・廃番 badge・絞り込み・ページ送り・`stocktake_in_progress`・`stocktake_not_in_progress` の扱い・前回の要約の snapshot）と数量の空欄の FieldError。保存を「呼ばない」ことを見る test（`T-C5-1`〜`T-C5-3`・`T5`・`T8`・`T23`）は mock の名前を `saveStockCount` に替えるだけで期待は変わらない。
- 期待を変える（42 の `SaveStockCountRequest`。round 2 の是正で「移す」から分けた）: 保存の呼出しの引数を見る `T4`・`T6`・`T17`・`T-C5-4`（`toHaveBeenCalledWith(501, 8)` ほかの `(stocktake_item_id, actual_count)` を `{ product_code, count_value, fresh: false, business_date: 今日, stocktake_item_id }` に。数量と明細の値は同じ。`T6` の「数えた商品を確認なしに保存し直す」は既定の計数の 2 つ目になり、同じ日付で値が違えば BIZ が要再確認〈理由 3〉にする。L10・C10 が止め、打ち直しの画面の経路は U3 の「入れ直す」）。
- 期待を変える（Gated Amendment の 2 本目。Coordinator の裁定）: `T6` の欄の文の assert を「入力済みの数を上書きできます」から「数を直すときは「入れ直す」を押してください。同じ日付で「数を保存」から違う数を入れると、数え直しが必要になります」に（73 の D-118 の節。同じ日付の違う値は要再確認〈理由 3〉になり、上書きにならない。新しい文を assert し、確認なしで保存する期待は変えない。文は「入れ直す」の button と同じ条件で出るので、fixture の明細に効く計数〈`count_value` 5〉を足す。明細の実数がある商品は効く計数も持つ）。
- assert の単位で分ける（round 2 の是正）: `T11 result shows total_cost, adjusted_items, and last comparison` は `棚卸し結果`・`仕入原価総額`・`¥2,500`・前回の要約の 2 行を移し、`adjusted_items` の商品名（`赤い糸`）の assert を外す（U8 が「補正の一覧を出さない」を止める）。`SC9 / REQ-205: difference %s retains sign and tone; uncounted stays neutral` は進行中の一覧の行の符号と色の assert を移し（差異は 35 の読取りの `difference`）、完了の結果の行の assert を外す（同じく U8）。
- 契約ごと外す（round 2 の是正で名指した）: `T21 result shows signed difference via formatListDifference (+N / -N, UI-10-D10)`（完了の結果の `adjusted_items` の符号だけを見る。U8）、`test_stocktake_page_req205_suspended_in_progress_disables_writes_and_keeps_browsing`（停止。T4 の節と U16）。
- 契約ごと外す: force_fill の確認と再送（T9・T10・T20）。owner の決定 TD-263、ADR D6。U7 が完了の拒否の一覧と 0 にするを止める。
- `stocktake-formatters.test.ts` の差異の計算（`update_count` の `current_difference` と同じ式）は、差異を BIZ が返す（35 の新契約の「読取り」）ので画面の計算をやめ、その test は契約ごと外す（C10・U1 が止める）。表示の formatter（日時・数量）の test は移す。

### T6 Z004 タブの test

```
$ rg -c '^\s*(it|test)\(' src/features/csv-import/CsvImportPage.test.tsx src/features/csv-import/components/PreviewStep.test.tsx src/features/csv-import/components/ResultStep.test.tsx src/features/csv-import/hooks/useCsvImportFlow.test.tsx src/features/csv-import/reducer.test.ts
src/features/csv-import/reducer.test.ts:6
src/features/csv-import/hooks/useCsvImportFlow.test.tsx:9
src/features/csv-import/components/ResultStep.test.tsx:1
src/features/csv-import/CsvImportPage.test.tsx:2
src/features/csv-import/components/PreviewStep.test.tsx:4
exit=0
```

- 期待を変える: `reducer.test.ts`・`useCsvImportFlow.test.tsx` の `additionalImportConfirmed` の payload と commit の引数（41: Z004 は使わない。`commit invalidates the exact independent oracle set` の `toHaveBeenCalledWith("preview-token", false)` は `("preview-token")` に）。
- 契約ごと外す（round 2 の是正で名指した。ADR D4: Z004 は同じ日の追加の確認を持たない。U10 が「dialog を出さない」を止める）: `useCsvImportFlow.test.tsx` の `test_import_command_req401_additional_confirmation_reaches_biz`、`PreviewStep.test.tsx` の `test_additional_import_req401_alert_dialog_lists_all_cancel_then_confirms_once`・`test_additional_import_req401_importing_disables_confirmation`。
- 移す: 取込みの流れ・error の表示・離脱の抑止。`PreviewStep.test.tsx` の `AdditionalImportConfirmDialog.tsx?raw` の日時の書式の test は、dialog を日報取込みタブが使い続けるので移す（置き場所を日報取込みの test へ移すかは着手時に決める）。
- 契約ごと外す: 停止の案内の test（T4 と同じ。`CsvImportPage.test.tsx` の `test_csv_import_page_req401_suspended_z004_tab_shows_notice_and_blocks_confirm`、`PreviewStep.test.tsx` の `test_preview_step_req401_suspended_shows_preview_and_disables_import`）。

### T7 `CmdError` の構築と mock

```
$ rg -c 'CmdError \{' src-tauri/src
src-tauri/src/cmd/csv_import_cmd.rs:4
src-tauri/src/cmd/daily_report_import_cmd.rs:5
src-tauri/src/cmd/disposal_cmd.rs:1
src-tauri/src/cmd/inventory_cmd.rs:1
src-tauri/src/cmd/manual_sale_cmd.rs:1
src-tauri/src/cmd/mod.rs:12
src-tauri/src/cmd/plu_export_cmd.rs:1
src-tauri/src/cmd/product_cmd.rs:1
src-tauri/src/cmd/receiving_cmd.rs:1
src-tauri/src/cmd/return_cmd.rs:1
src-tauri/src/cmd/settings_cmd.rs:4
exit=0
$ rg -l 'error_id: null' src --glob '*.ts*' | wc -l
34
exit=0
```

- 移す（機械的な追随）: Rust の struct の literal 22 か所（packet の S7 の数え方: `rg -U -c 'CmdError \{\s*\n\s*[a-z_]+:' src-tauri/src` の和 21 と `cmd/mod.rs` の `Self {` の 1。上の `rg -c 'CmdError \{'` の 32 は定義・`impl`・戻り型・doc comment を含むので literal の数ではない。round 2 の是正で数え方を S7 に揃えた）に `stocktake_recovery: None`、TS の `CmdError` の型の mock に `stocktake_recovery: null`（値の期待は変えない）。TS の 34 file のうち型が `CmdError` でない literal は変えない（着手時に typecheck の error で数え直す）。

### T8 棚卸しの記録の詳細の画面の test

```
$ rg -c '^\s*(it|test)\(' src/features/inventory-records/StocktakeRecordDetailPage.test.tsx
7
exit=0
```

- 期待を変える（65 の新契約）: 補正の movement の節・`adjustment_quantity` と画面のロス原価の計算を確かめる行は、明細の数・差異・BIZ のロス原価の文字列・完了の後の差へ（U13）。到達・戻り・not found の test は移す。

### T9 生成の enum と kind の一覧

```
$ rg -n 'restore_durability_unknown|RestoreDurabilityUnknown' src src-tauri/src --glob '!src/lib/bindings.ts' | wc -l
16
exit=0
```

- 移す: `cmd/mod.rs` の kind の test と `finite_enum_contract_tests.rs` の一覧に `stocktake_guard` と新しい 5 つの enum を足す（W1・W6）。`BackupRestorePage` の kind の分岐は変えない（Adjacent Pattern Audit の除外）。

### T10 vZ の CHECK・`NewCsvImport`・内部型の固定で red になる既存の test（round 1 の是正）

```
$ rg -n "INSERT INTO csv_imports|NewCsvImport \{|'csv_import'" src-tauri/src
src-tauri/src/cmd/csv_import_cmd.rs:337:            &crate::db::sales_repo::NewCsvImport {
src-tauri/src/cmd/csv_import_cmd.rs:350:            &crate::db::sales_repo::NewCsvImport {
src-tauri/src/cmd/csv_import_cmd.rs:474:                    &crate::db::sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/commit.rs:124:        &NewCsvImport {
src-tauri/src/biz/stock_ledger.rs:509:             VALUES ('L12', 'sale_auto', -1, 0, 'csv_import', 1, '2026-05-20T10:00:00', ?1, 1, 3)",
src-tauri/src/biz/csv_import_service/tests/time_evidence_tests.rs:68:        "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount,
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:135:                 WHERE reference_type = 'csv_import' AND reference_id = ?1 AND is_voided = 0",
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:154:        &sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:198:        &sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:358:        &sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:405:        &sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:445:        &sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:464:        &sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:500:            &sales_repo::NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/parse_tests.rs:294:        &NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/parse_tests.rs:340:        &NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/parse_tests.rs:353:        &NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/list_tests.rs:17:            &NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/list_tests.rs:67:        &NewCsvImport {
src-tauri/src/biz/csv_import_service/tests/list_tests.rs:149:         VALUES ('CSV-DETAIL-P1', 'sale_auto', -1, 9, 'csv_import', ?1, NULL, 0, '2026-08-03T12:00:00', '2026-04-01', 0)",
src-tauri/src/db/schema_v1.rs:185:    reference_type TEXT CHECK(reference_type IN ('csv_import','manual_sale','receiving_record','return_record','disposal_record','stocktake') OR reference_type IS NULL),
src-tauri/src/db/schema_v9.rs:42:    reference_type TEXT CHECK(reference_type IN ('csv_import','manual_sale','receiving_record','return_record','disposal_record','stocktake') OR reference_type IS NULL),
src-tauri/src/db/schema_v9.rs:199:             WHERE reference_type IN ('csv_import', 'stocktake') OR movement_type = 'stocktake')",
src-tauri/src/db/schema_v9.rs:696:            "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount,
src-tauri/src/db/sales_repo.rs:101:pub struct NewCsvImport {
src-tauri/src/db/sales_repo.rs:258:        "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount, skipped_count, status, imported_at)
src-tauri/src/db/sales_repo.rs:525:         WHERE reference_type = 'csv_import'
src-tauri/src/db/sales_repo.rs:1708:            "INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount, skipped_count, status, imported_at)
src-tauri/src/db/sales_repo.rs:1794:        let record = NewCsvImport {
src-tauri/src/db/sales_repo.rs:2030:                 VALUES ('DETAIL-MOVE', 'sale_auto', -1, 9, 'csv_import', ?1, NULL, ?2, ?3, '2026-04-01', 0)",
src-tauri/src/db/sales_repo.rs:2141:             VALUES ('VM-P1', 'sale_auto', -3, 7, 'csv_import', ?1, 0, '2026-03-21T00:00:00', '2026-04-01', 0)",
src-tauri/src/db/sales_repo.rs:2153:                "SELECT is_voided FROM inventory_movements WHERE reference_type = 'csv_import' AND reference_id = ?1",
src-tauri/src/db/sales_repo.rs:2170:             VALUES ('INV7-P1', 'sale_auto', -5, 5, 'csv_import', 999, 0, '2026-03-21T00:00:00', '2026-04-01', 0)",
src-tauri/src/db/test_support.rs:134:         INSERT INTO csv_imports (id, filename, settlement_date, file_hash, total_items, total_amount,
src-tauri/src/db/test_support.rs:144:         ('LG-MEAS','sale_auto',-1,3,'csv_import',1,'2026-03-20T20:00:00', '2026-04-01', 0);",
src-tauri/src/db/disposal_repo.rs:763:            "INSERT INTO csv_imports
exit=0
$ rg -n 'fn test_' src-tauri/tests/import_internal_contract_test.rs
113:fn test_import_internal_contract_req401_is_minimal() {
196:fn test_wire_contract_req401_i_w1_i_w2_i_w3_i_w5_generated_binding_is_atomic() {
250:fn test_sales_design_req502_future_coverage_counts_distinct_report_dates() {
300:fn test_active_sales_import_vocabulary_sweep_i_g1() {
324:fn test_sweep_dir_for_tokens_skips_known_generated_paths() {
exit=0
$ rg -n 'phase_one_movement_is_error_until_b' src-tauri/src
src-tauri/src/biz/stock_ledger.rs:501:    fn test_stock_ledger_req401_phase_one_movement_is_error_until_b() {
exit=0
$ rg -n 'key_movement\(.*Some\(|settlement_no: Some\(' src-tauri/src --glob '!src-tauri/src/io/**'
src-tauri/src/db/inventory_repo.rs:788:        let id = insert_movement(&conn, &key_movement("KEY-1", "2026-05-04", Some(0))).unwrap();
exit=0
```

最後の command は round 2 の是正で足した（`NewMovement` を `reference_type` NULL と精算回数で組む fixture は、上の `'csv_import'` の検索に当たらないが vZ の CHECK `(reference_type IS 'csv_import') = (phase = 1)` で落ちる）。

| hit（file の symbol） | 分類 | 理由 / 代わりの行 |
|---|---|---|
| `src-tauri/src/biz/csv_import_service/commit.rs` の `execute_commit`、`src-tauri/src/db/sales_repo.rs` の `NewCsvImport` の定義と `insert_csv_import` | production（S2・S5） | — |
| `src-tauri/src/db/schema_v1.rs`・`schema_v9.rs` の CHECK と止まる検査、`sales_repo.rs` の `get_csv_import_record_detail` の `reference_type = 'csv_import'` | 変えない | 版の履歴の SQL と、読取りの絞り込み（vZ の CHECK に当たらない） |
| `schema_v9.rs` の `test_migration_req205_vl_stops_on_stocktakes_or_csv_imports_without_changes` | 変えない | v8 の DB に v9 だけを当てる test で、vZ を当てない |
| `csv_import_cmd.rs` の `test_csv_cmd_req401_parse_cache_preserves_ordered_same_date_ids` | 契約ごと外す | T2 と同じ（ADR D4。同じ日の取込みの一覧を Z004 で使わない。W5） |
| `csv_import_cmd.rs` の `test_csv_cmd_req401_rollback_stop` | 期待を変える | 停止の assert を外し（停止 ADR を superseded に）、取消の CMD が新しい経路を通ることを確かめる形にする。fixture の `NewCsvImport` に精算の鍵を足す（W5 と同じ file） |
| `commit_tests.rs` の `test_commit_req401_toctou_check` | 移す | 同じ hash の再検査は順 3 で残る。fixture の `NewCsvImport` に精算の鍵を足す（値の期待は変えない） |
| `commit_tests.rs` の `test_commit_req401_same_day_additional_import_is_insert_only`・`additional_import_requires_confirmation`・`settlement_date_toctou`・`same_date_snapshot_removal_requires_repreview`・`same_date_snapshot_replace_requires_repreview`・`same_date_snapshot_order_requires_exact_match` | 契約ごと外す | T2 と同じ（ADR D4） |
| `parse_tests.rs` の `test_parse_and_validate_req401_file_hash_blocking` | 移す | 順 3（同じ hash の拒否）は残る。fixture に精算の鍵 |
| `parse_tests.rs` の `test_parse_and_validate_req401_same_date_snapshot_is_ordered` | 契約ごと外す | ADR D4（同じ日の snapshot を Z004 で持たない。Z7） |
| `list_tests.rs` の `test_list_csv_imports_req401_normal`・`seed_detail_import`（helper）・`test_get_csv_import_record_req207_movement_source` | 移す | 一覧と詳細の読取りの契約は変わらない。`NewCsvImport` に精算の鍵、`csv_import` の movement を `phase = 1`・`settlement_no` つきに（値の期待は変えない） |
| `inventory_repo.rs` の `test_insert_movement_req201_writes_key_and_phase_from_settlement_no`（helper `key_movement` の `Some(0)` の case。round 2 の是正） | 移す | `phase` を `settlement_no` から決める契約（21）は残る。vZ の CHECK で `reference_type` が NULL の `phase = 1` の行は入らないので、`Some` の case の fixture を `reference_type` `csv_import`（と `reference_id`）にする。`business_date`・`phase`・`settlement_no` の期待は変えない（M4 が CHECK を止める） |
| `stock_ledger.rs` の `test_stock_ledger_req401_phase_one_movement_is_error_until_b` | 契約ごと外す | ADR D12 の (a) の「`phase = 1` の行は誤りで止まる」を b1 で精算の窓に置き換える（31）。L9 が「計数の無い商品で `phase = 1` を含む和」を、L1〜L5 が精算の窓を止める |
| `disposal_repo.rs` の `seed_csv_import`（helper） | 移す | helper の INSERT に精算の鍵を足す。helper を呼ぶ test の期待は変えない |
| `sales_repo.rs` の `test_insert_sale_record_req401_auto_with_csv_import`・`seed_csv_import`（helper）・`test_get_csv_import_record_detail_req207_movements_filter`・`test_void_movements_req401_by_reference`・`test_void_movements_req401_no_movement_type_filter` | 移す | raw の INSERT に精算の鍵、`csv_import` の movement を `phase = 1`・`settlement_no` つきに（値の期待は変えない） |
| `test_support.rs` の `seed_legacy_fixture`、`time_evidence_tests.rs` の `import` | 契約ごと外す | T3（時点証拠の撤去） |
| `src-tauri/tests/import_internal_contract_test.rs` の `test_import_internal_contract_req401_is_minimal` | 期待を変える | 32 の「preview の cache の契約」で BIZ-03-D1 を改めた: `MatchedRow` は `line_no`・`normalized_jan`・`product_code`・`shared_jan`・`quantity`・`amount`、`CommitRequest` は `cached_data`・`today`。Rust と 32 の §15.2 の型の節の両方を同じ並びにする（Z13・Z21） |
| 同じ file の `test_wire_contract_req401_i_w1_i_w2_i_w3_i_w5_generated_binding_is_atomic` | 変えない | 固定の語（`AdditionalImportConfirmationRequired`・`additionalImportConfirmed`・`same_date_imports`）は日報の取込みの型（`DailyReportDuplicateStatus`・`commitDailyReportImport`・`DailyReportDuplicateCheck`）が持ち続ける。bindings の再生成の後に通ることを確かめ、通らなければ期待を変えるに移し、理由を PR に書く |
| 同じ file の `test_sales_design_req502_future_coverage_counts_distinct_report_dates`・`test_active_sales_import_vocabulary_sweep_i_g1`・`test_sweep_dir_for_tokens_skips_known_generated_paths` | 変えない | b1 は固定の語を足さない・戻さない（新しい code に `Overwrite` 系の語を書かない） |

### T11 棚卸しの repo の test（round 1 の是正）

```
$ rg -n -A2 '#\[test\]' src-tauri/src/db/stocktake_repo.rs | rg -c 'fn '
45
exit=0
```

| test（`src-tauri/src/db/stocktake_repo.rs`） | 分類 | 理由 / 代わりの行 |
|---|---|---|
| `test_find_active_stocktake_req205_*`（4）、`test_insert_stocktake_item_req205_*`（3）、`test_find_stocktake_item_by_code_req205_*`（5）、`test_find_last_completed_stocktake_req205_*`（2）、`test_insert_stocktake_req205`、`test_find_stocktake_by_id_req205_*`（2）、`test_find_eligible_products_req205_*`（4）、`test_list_stocktake_items_req205_basic`・`_includes_is_discontinued`・`_dept_filter`（3）、`test_get_stocktake_record_detail_req206_not_found`（1） | 移す（25。下の 2 行と合わせて 29） | 読取り・開始の契約は残る（明細の型に field が増えても、今の field の期待は変えない） |
| `test_update_item_count_req205`・`_not_found`（2） | 移す | `update_stocktake_item_count` は計数の保存で明細の表示用の写し（`actual_count`・`counted_at`）を書くのに使い続ける（35） |
| `test_find_item_with_parent_status_req205_found`・`_not_found`（2） | 移す | 計数の保存の明細の確かめ（進行中の棚卸しの明細か）に使い続ける（35。C5） |
| `test_list_stocktake_items_req205_counted_filter`・`_dept_and_counted_combined`・`_dept_and_uncounted_combined`（3）、`test_get_stocktake_progress_req205`、`test_count_uncounted_items_req205`、`test_list_uncounted_items_req205` | 期待を変える（6） | 35 の新契約の「計数済み」は明細を参照する計数があること（`actual_count` の NULL でない）。fixture に計数の行を入れる（C10・K4・C8） |
| `test_get_stocktake_record_detail_req206_header_and_item_count`（1。Codex の closure の Gated Amendment で「移す」から直した。実装の 1 段目で red になった） | 期待を変える | 記録の詳細の明細は補正の movement からでなく明細から読む（65 の D-118 の節「棚卸しの記録の詳細は補正の movement の一覧を持たない」、packet の S2「記録の詳細は明細と header から読む（補正の movement から読まない）」）ので、`items.len()` は補正の movement の 1 でなく明細の数 2（`item_count` と同じ）。fixture の補正の movement は、詳細が movement を読まないことを示すために残す。header と `item_count` の期待は変えない |
| `test_complete_stocktake_req205`・`_already_completed`（2）、`test_update_item_valuation_req205` | 期待を変える（3） | 完了の TX がカーソルと `valuation_quantity` も書く（35・tracking。K1） |
| `test_get_items_for_complete_req205_normal`・`_partial`・`_excludes_null`（3） | 契約ごと外す | force_fill の後に `actual_count` の NULL が無い全明細を読む旧本体の手順（35 の新契約の完了は計数から評価数量を求め、この読取りを使わない。owner の決定 TD-263）。K1・K4 |
| `test_get_stocktake_record_detail_req207_correction_items_live_basis`・`_movements_filter`（2） | 契約ごと外す | 65 の新契約（記録の詳細は補正の movement を持たない）。R1・R3 |
| `test_find_saved_request_req205_item_recount_and_both`（`mod time_evidence` の中、1） | 契約ごと外す | T3 |

### T12 形の変わる型・引数を作る所（機械的な追随。round 1 の是正）

```
$ rg -c 'NewSaleRecord \{' src-tauri/src
src-tauri/src/cmd/sales_cmd.rs:1
src-tauri/src/db/sales_repo.rs:9
src-tauri/src/biz/sales_service.rs:1
src-tauri/src/biz/csv_import_service/commit.rs:1
src-tauri/src/biz/inventory_service/manual_sale.rs:1
exit=0
$ rg -c 'CommitRequest \{' src-tauri/src
src-tauri/src/cmd/csv_import_cmd.rs:1
src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs:4
src-tauri/src/biz/csv_import_service/tests/commit_tests.rs:17
src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs:2
src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs:1
src-tauri/src/biz/csv_import_service/mod.rs:1
exit=0
$ rg -c 'CachedPreview \{' src-tauri/src
src-tauri/src/cmd/csv_import_cmd.rs:3
src-tauri/src/biz/csv_import_service/test_support.rs:2
src-tauri/src/biz/csv_import_service/mod.rs:1
src-tauri/src/biz/csv_import_service/tests/legacy_stop_tests.rs:1
exit=0
$ rg -c 'MatchedRow \{' src-tauri/src
src-tauri/src/biz/csv_import_service/parse.rs:2
src-tauri/src/biz/csv_import_service/mod.rs:1
exit=0
$ rg -c 'IntegrityResult \{' src-tauri/src
src-tauri/src/biz/integrity_service.rs:2
exit=0
$ rg -l 'mismatch_count' src --glob '*.test.*'
src/features/stocktake/StocktakePage.test.tsx
src/features/integrity-check/IntegrityCheckPage.test.tsx
exit=0
$ rg -c 'apply_stock_change\(' src-tauri/src
src-tauri/src/biz/csv_import_service/commit.rs:1
src-tauri/src/biz/inventory_service/common.rs:7
src-tauri/src/biz/inventory_service/receiving.rs:1
src-tauri/src/biz/inventory_service/disposal.rs:1
src-tauri/src/biz/inventory_service/manual_sale.rs:1
src-tauri/src/biz/inventory_service/returns.rs:1
exit=0
```

- 移す（値の期待は変えない）: `NewSaleRecord` の literal に `stock_linked`（手動の売上ほかは false、Z004 の確定だけが取込みのときの値）、`CommitRequest` の literal から `additional_import_confirmed` を外し `today` を足す（旧本体の test は T2 の分類のとおり移すか外す）、`CachedPreview` の literal に `settlement`（`active_same_date_import_ids` を外す）、`MatchedRow` の literal に `normalized_jan`・`shared_jan`（`pos_stock_sync` を外す）、`IntegrityResult` の literal と TS の mock（`StocktakePage.test.tsx`・`IntegrityCheckPage.test.tsx`）に `shared_jan_conflicts: []`、`apply_stock_change` の呼出しから `settlement_no` を外す（`common.rs` の定義と test、入出庫 4 業務の 4 か所。Z004 の確定の 1 か所は S5 で消える）。
- 着手時に上の command を逐語で回し、出力と表が違えば表を直してから進める。

## Residual Test Gaps

- Windows の実機での HID・IME・native の file dialog の CP932 の取込みは L3（packet の Test Plan の L3-3・L3-6・L3-8・L3-10）。
- 移行の前からある共有 JAN の組の画面の一覧は、DB の手での改変が要るので L3 に入れず、U14 と画面の写し。
- 日付の変わり目（U5）は時計を差し替える自動 test と画面の写しで、Windows の実機の時計では見ない。
- 計数の衝突（理由 3）の画面の表示は L3 の台本に入れず、C10・U6 と画面の写し。
- Final Review の code 読みの項目（自動 test で観測できない mutant。round 2 の是正で「Would fail if」から移した）: (1) `save_stock_count` の fresh なカーソル（最大の精算回数と `MAX(inventory_movements.id)` と `MAX(csv_imports.id)`）を INSERT と同じ TX の中で読み、`complete_stocktake` の `cutoff_import_id` を完了の TX の中で読む（C2・K1。1 本の接続の test では TX の外で読んでも値が同じ。取込みのカーソルは Codex の closure の Gated Amendment で足した）。(2) `save_stock_count` の明細の検査が INSERT より前（C5。存在しない明細の case は外部キーで見分けるが、明細の商品の違いの case は同じ文で TX が戻り最終の状態が同じ）。(3) 棚卸し・Z004・商品の CMD が PC の今日を `chrono::Local` から 1 か所で読み BIZ へ渡し、BIZ の計数・完了・Z004 の日付の判定が `chrono::Local` を読まない（W3。BIZ の側は W3 の test で見る）。(4) D-052 の oracle（`src/test/invalidation-oracle.ts`）で要再確認の一覧の key を持つ entry が UI_TECH_STACK の D-118 の段落の 12 と同じ（U15。contract と oracle の両方で同じ entry を落とす誤りは meta test で見つからない）。(5) `stock_counts` を UPDATE・DELETE する関数が無い（packet の Design Readiness の `rg`）。
- 店の PC の件数（数千点）での Z004 の確定と完了の時間は未実測（ADR の Consequences の「1 商品の求め直しは合成データで 0.01 ms 未満」は repo の外の測定）。実装の後に合成の 3000 商品の fixture で測るかは S5 で決める（R4 の L3 の合否に入れない）。
