# Test Design Matrix: Z004 の取込みを取り消す入口

Plan Packet: [2026-10-23-z004-rollback-entry](../2026-10-23-z004-rollback-entry.md)。行の記号は RB（BIZ。RB7 は 32 の表の順 4 の文と variant）・W（CMD と登録。W3 は `CmdError` の取込みID）・U（画面）・T（既存の test の移し方）。test 名の REQ は REQ-401（Z004 の取込みと取消）・REQ-206 / REQ-207（記録の詳細）。

## Risk

Risk: R3

## Contracts Under Test

- 32 §15.6b BIZ-03-D2: preview は取消の本体を同じ TX で当てて明示の rollback で戻し、何も書かない。手順 5 の明示の rollback は手順 4 の成否によらず当たる（`tx.finish()`。Gated Amendment 2）。名指す商品は同じ状態の取消と同じ。
- 32 §15.6b の手順 1〜3: 無い ID・取消済みの拒否、取り込み直す精算回数、記録の詳細の差異・ロス原価・参考の差が変わることがある完了した棚卸し（35 の完了の後の比較の範囲に入る棚卸し。範囲の判定による候補）。
- 32 の冒頭の D-118 の節の表の順 4: 拒否の文が同じ精算の active な取込みの取込みID を名指し、error の構造（`BizError::SettlementConflict` の `csv_import_id` → `CmdError.related_csv_import_id`、kind は `import_error` のまま）でも同じ ID を持つ（preview と確定の両方の経路）。ほかの拒否と error は None。
- 41 の `preview_rollback_csv_import`: 薄い command、error の kind、登録（Specta と runtime の 2 面。`scripts/check-command-drift.sh`）。
- 65 §65.5a TRACE-D13: CSV取込みの詳細の取消の入口（active だけ）、成功の後の表示と再取得、失敗の再試行。取消の結果は表示している取込みのものだけ（別の取込みの詳細へ移ると作り直す。Gated Amendment 2）。
- 55 §55.7・§55.11 UI-07-D18（Gated Amendment 2）: 取消の応答を待つ間は、取込みの結果の画面（Z004 タブ）と CSV取込みの詳細の両方で route の移動とリロード等の page の離脱を止め（共有の hook の `useBlocker`・`enableBeforeUnload`。native の window の close は 55 §55.7 の「止める範囲」のとおり保証しない。Gated Amendment 4）、Z004 タブではタブの切替えも止める（`TabsTrigger` の disabled）。失敗すれば解く。
- 55 §55.11 UI-07-D18 の順 4 の拒否からの導線: 取込みの画面の error の Alert（`ErrorState`）が、`related_csv_import_id` があるときだけ名指した取込みの詳細への link を出す。
- 55 §55.11 UI-07-D18: 2 つの入口に共通の確認（項目・要再確認〈因果を言わない文、操作の無い一覧〉・取り込み直し・完了した棚卸し〈言い切らない文〉・押せる条件・cache を使い回さない）、取消の mutation を 1 つに、Z004 タブの導線と `tab`（初期選択だけ）。
- UI_TECH_STACK の UI-STATE-D2: `tab` の値の集合は `src/features/csv-import/types.ts` の tuple と schema が持ち、route は import する。

## Failure Modes

- preview が取消を COMMIT する、TX の外で操作ログを書く、写し・印・範囲・`is_voided`・取込みの状態を変えたまま残す。手順 4 の失敗で TX を明示に閉じずに返る（drop が ROLLBACK の失敗を捨て、共有の接続に未完了の TX が残りうる）、または閉じ方が手順 4 の真因を覆う。
- preview が取消と違う商品を名指す（規則を写して食い違う、求め直す商品の集合が違う）。
- 取消済みの取込みに preview が成功し、確認が「取り消す」を押させる。無い ID で preview が internal になる。
- 比較の範囲に入る完了した棚卸しを取りこぼす（完了の後に入れた遅い精算、精算回数がちょうど完了の時点の精算の n の精算）か、範囲に入らない棚卸しを名指す（精算日がちょうど完了日の、n より後の精算）。範囲に入るが記録の詳細の数が変わらない棚卸しを落とす（差を求めて絞る実装）、または確認が差異・ロス原価・参考の差の変化を言い切るか、変わりうる「差異」「ロス原価」を名指さない。
- 確認が「取り消すと…になります」と因果を言い切る（取り消す前から要再確認の商品も一覧に入る）。確認の一覧に入れ直しの操作や文が出る。
- 順 4 の拒否の文が取込みID を名指さない、または別の取込みの ID（固定の値、DB の最新の取込み）を名指す。error の構造に ID が無い、文と構造の ID が違う、順 4 以外の error に ID が入る。拒否の Alert に名指した取込みの詳細への link が無い、別の番号（精算回数）で link を作る、ID の無い error に link を出す。
- command が Specta と runtime の登録のどちらかに無い。
- 詳細が取消済みにも入口を出す、一覧の行に取消を出す、取消の後に詳細と一覧が古いまま。
- 確認が読取りの間・失敗の後に「取り消す」を押させる、閉じて開き直しても古い preview を出す。
- 2 つの入口で確認の中身や Toast・再取得が違う（mutation を 2 つ持つ）。
- 取消の応答を待つ間に route の移動・リロード等の page の離脱・タブの切替えで離れられる（離れた後の失敗は別の画面の Toast だけになる）、止めが片方の入口にしか効かない、失敗の後も止めが残る。
- 詳細で別の取込みへ移った後に、前の取込みの取消の結果（取消の card と Z004 の導線）が残る。
- `tab=z004` で Z004 タブが開かない、不正な値で画面が壊れる、導線の link の search が違う。route が値の集合を feature と別に持つ（UI-STATE-D2）。
- 要再確認を色だけで示す。

## Test Matrix

- Before citing an existing test as regression coverage, use `rg` or an equivalent repository search to verify that the cited test exists.
- helper と mock の実装を読み、実際に通る境界と置換される境界を確認して Test Type / coverage を選ぶ。helper 名だけで実 router / integration と分類しない。
- `Would fail if...` は壊れる振舞いを観測できる入力・経路と結びつける。状態 reset なら初回 mount に加え同値再選択等の別経路を確認し、対象契約が行使されるものを選ぶ。
- oracle の anchor は定義文にしか現れない literal を選び、`rg -c` で対象 file 内 1 件を file ごとに確かめる（複数 file を 1 本の `rg` で数えない）。不変の guard にも感度の実測行を置き、文言を改訂した後は「新しい文言が exact で 1 件 + 旧い文言が 0 件」の対にする。
- oracle は検証対象と独立の正本から転記し、mutation は production 側だけを変える。
- 結果を空にする注入で、期待が空集合の case だけが kill を主張していないかを確かめ、各組合せに非空の期待を 1 件置く。

BIZ の test は実の SQLite（`src-tauri/src/db/test_support.rs` の in-memory の DB と b1 の fixture）を通り、置換する境界は無い。画面の test は command を mock し、router は実の TanStack Router（今の `CsvImportRecordDetailPage.test.tsx`・`CsvImportPage.test.tsx` の render の形）。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| RB1 BIZ-03-D2（同じ商品を名指す） | preview と取消の名指しが食い違う | integration（実 DB） | `test_preview_rollback_csv_import_req401_matches_rollback_recheck_targets` | 同じ fixture（RB5 の A1 の形に、取り消す前から要再確認の商品 1 つを足す。非空の期待）で preview の `recheck_targets`（商品コード・範囲・理由）と、続けて取り消した `RollbackResult.recheck_targets` が等しくない、または取り消す前から要再確認の商品が両方に無い。preview の側だけを変える mutation（手順 4 で要再確認の一覧を求め直しの前に読む）で red。求め直す商品を狭める mutation は preview と取消が同じ関数を呼ぶので両方に同じく効き、この行では見えない（RB5 が止める） |
| RB2 BIZ-03-D2（何も書かない） | preview が COMMIT する・書いたまま残る | integration | `test_preview_rollback_csv_import_req401_writes_nothing` | preview の前後で、`products` の `stock_quantity`・`stock_recheck`・範囲の 2 列、`sale_records` と `inventory_movements` の `is_voided`、`csv_imports.status`、`operation_logs` の行数、`sqlite_sequence` のどれかが違う。`tx.finish()` を `tx.commit()` に替える mutation と、preview に操作ログの書込みを足す mutation の両方で red（成功の経路）。手順 4 の失敗の経路（`apply_rollback` の失敗でも `tx.finish()` で閉じ、閉じる失敗を先に、閉じられたら手順 4 の失敗を返す）は注入の道具が無く、この行では見えない（Residual Test Gaps の 1 行目と AC1 の `apply_rollback(&tx, csv_import_id)?` の数。Gated Amendment 2） |
| RB3 手順 1 | 無い ID・取消済みで通る | unit（実 DB） | `test_preview_rollback_csv_import_req401_rejects_missing_and_rolled_back` | 無い ID が `BizError::NotFound`（「CSV取込み記録が見つかりません: ID」を含む）でない、取消済みの ID が `BizError::ValidationFailed`（「この取込みは取り消し済みです」）でない、または拒否の後に DB が変わる |
| RB4 手順 2 | 精算回数を取り違える | unit | `test_preview_rollback_csv_import_req401_names_settlement_no` | 精算回数 0 と 101 の 2 つの取込みで、それぞれの preview の `settlement_no` が 0 と 101 でない（0 を「無い」として落とす、別の取込みの番号を返す） |
| RB5 SPEC-STK-CNT-D3 の A1（取り消すと要再確認になる） | 取消の前に要再確認でなかった商品を名指さない | integration | `test_preview_rollback_csv_import_req401_recheck_after_losing_count_day_settlement` | 精算 101（日付 D、商品 B だけ）・102（D+1、商品 A）、A の既定の計数（日付 D）の DB で、取消の前の A が要再確認でなく、101 の preview の `recheck_targets` が A だけ（範囲 = 計数 − 1 〜 計数、理由 `undetermined_settlement`）でない。期待は非空（空の結果を返す mutation で red）。求め直す商品を「この取込みの商品だけ」に狭める mutation（計数を持つ商品を落とす）でも red（A は 101 に行が無く、計数を持つ商品としてだけ求め直される） |
| RB6 手順 3（35 の範囲） | 比較の範囲に入る完了した棚卸しを取りこぼす・範囲の外を名指す・差で絞る | integration | `test_preview_rollback_csv_import_req401_affected_completed_stocktakes` | (a) 完了の前に取り込んだ取込み（id ≤ `cutoff_import_id`）の preview が、その棚卸しを名指さない。(b) 完了の後に取り込んだ、精算日が完了日より前の取込みの preview が名指さない（35 の「精算日 < 完了日」の項）。(c) 完了の後に取り込んだ、精算日が完了日以後で精算回数が完了の時点の精算の n より大きい取込みの preview が名指す。(d) 進行中の棚卸しを名指す。(e) 範囲に入るが記録の詳細の差異・ロス原価・参考の差のどれも変わらない取込み（完了の前に取り込んだ精算 101・102 が明細の商品を売っておらず、全ての明細の商品を fresh に数えて完了した後の 101）の preview が、その棚卸しを名指さない（差を求めて絞る実装で red）。(f)（Gated Amendment 2）精算回数の側だけで決まる等値境界: 精算回数がちょうど完了の時点の精算の n（`cutoff_import_id` の取込みの精算）で、精算日がちょうど完了日（`completed_at` の日付の部分。日付の側は偽）の取込みの preview が、その棚卸しを名指さない（`settlement_kept` の `settlement.settlement_no <= no` を `<` にする mutant で red）。(g)（同）日付の側だけで決まる等値境界: 完了の後に取り込んだ、精算回数が n より大きく（精算回数の側は偽）、精算日がちょうど完了日の取込みの preview が、その棚卸しを名指す（`settlement.settlement_date.as_str() < date` を `<=` にする mutant で red。`completed_at` に 0 時でない時刻〈例 `T18:00:00`〉を置き、`completed_stocktakes_comparing_settlement` の `get(..10)` を外して日時の全体と比べる mutant も red）。(f)(g) は今の (a)〜(e) の期待を変えないため、同じ test の中で別の DB（`setup_test_db` をもう 1 つ）に組んでよい（例: 精算 201〈精算日 D〉を取り込み、`completed_at` が D の 18 時・`cutoff_import_id` が 201 の取込みの完了した棚卸しを置き、完了の後に精算 202〈精算日 D〉を取り込む。201 の preview は名指し、202 の preview は名指さない）。修正前: (a)〜(e) は今の fixture の 2 つの境界〈精算回数 102 = n の取込みは精算日 04-02 < 完了日で日付の側も真、(c) は精算日 04-06 > 完了日 04-05〉で OR の片側だけが判定を決める等値の case が無く、2 つの比較の mutant が通る（Final Review の P3）。(f)(g) を足すと red。各 case に非空か空の期待を 1 つずつ置く。名指す集合を範囲のままにする理由: 記録の詳細の明細の差異・ロス原価は完了の時点の精算の n 以下の精算だけを使い（b1 の `stocktake_service.rs` の 720 行の `ledger_at_cutoff`、`after` なし）、範囲〈精算回数 ≤ n、または精算日 < 完了日。同じ file の 571〜574 行〉はそれを含むので、範囲で名指せば差異・ロス原価が変わる棚卸しを取りこぼさない |
| RB7 32 の表の順 4（取込みID を名指す。文と error の構造） | 文と構造が取込みID を名指さない・別の取込みを名指す | unit（実 DB）。既存の 3 test の期待を当て直す（T5） | preview の経路: `parse_tests.rs` の `test_parse_req401_rejects_other_version_of_same_settlement_and_allows_after_rollback`。確定の経路: `commit_tests.rs` の `test_commit_req401_rejects_shared_or_newly_matched_jan_and_rechecks_settlement_from_cache` と `csv_import_cmd.rs` の `test_csv_cmd_req401_commit_keeps_token_on_rejection_and_removes_on_success` | 拒否の文が「同じ精算と思われる別の資料が既に取り込まれています（精算 {n}、取込みID: {id}）」で、{id} が各 test の中で先に確定した同じ精算の取込みの `csv_import_id`（parse は `first.csv_import_id`、commit は v2 の確定の結果の ID、cmd は `insert_import` の返す ID）と等しくない。期待は test の中の変数から組み、固定の数を書かない。fixture: b1 の 3 test は名指す取込みが DB の最初の取込み（ID 1）なので、3 test とも名指す取込みの前に別の取込みを 1 件入れ、名指す ID を 1 以外にする（例: 別の精算回数の売上 0 の取込みの行、cmd では `insert_import(…, "rolled_back", …)` のように順 2〜5 の比べる先にならない取込み。既存の在庫・`write_surface` の期待を変えない形を選ぶ）。各 test で、名指す ID が 1 でないことを assert する。cmd の test は、名指す取込みの後にも順 2〜5 の比べる先にならない取込み（`insert_import(…, "rolled_back", …)`）を 1 件足し、名指す ID が 1 でも DB の最大の id でもないことを assert する（parse・commit の 2 test は名指す取込みが DB の最新のままなので、最新の ID を名指す mutant はこの 1 test が止める）。error の構造: BIZ の 2 test は `Err(BizError::SettlementConflict { message, csv_import_id })` で、`csv_import_id` が文の {id} と同じ期待の変数と等しい。cmd の test は `CmdError.kind` が `import_error` のままで、`related_csv_import_id` が `Some(期待の ID)`。production の側で ID を固定値 `1` にする mutant で 3 test とも red（b1 の今の fixture のままでは名指す ID が 1 なので、この mutant でも 3 test とも GREEN になる）。DB の最新の取込みの ID（`SELECT MAX(id) FROM csv_imports` や確定の経路の `last_insert_rowid()`）を名指す mutant で cmd の test が red。文の ID と構造の ID を別の値から取る mutant（構造だけ固定値）で 3 test とも red |
| W1 41 | error の kind が違う | unit（CMD） | `test_preview_rollback_csv_import_cmd_req401_maps_error_kinds` | 無い ID が kind `not_found`、取消済みが kind `validation` でない。成功で `RollbackPreview` の `settlement_no` が BIZ の値と違う |
| W2 41 §17.9 | command が Specta か runtime の登録に無い | static（既存の `scripts/check-command-drift.sh`。`scripts/doc-consistency-check.sh` の `check_command_registry_drift` も同じ script を呼ぶ） | `bash scripts/check-command-drift.sh`（AC2）。新しい test は書かない | `#[tauri::command]` の宣言（D）、runtime の登録（`run` の `tauri::generate_handler!`、H）、Specta の登録（`export_bindings_to` の `collect_commands!`、S）、bindings の wire 名（`__TAURI_INVOKE("preview_rollback_csv_import"`、T）の 4 つの集合が一致しない（H か S から外すと `[ERROR] D - H` か `[ERROR] D - S` で exit 1。S から外して bindings を再生成すると T からも消え、AC2 の `rg -c '^export type RollbackPreview ='` も 0 になる） |
| W3 40 §5.3・41 §17.4（順 4 の拒否の取込みID） | 変換で ID が落ちる・kind が変わる・ほかの error に ID が入る | unit（CMD。`cmd/mod.rs` の tests。b1 の `test_cmd_error_req205_stocktake_guard_carries_recovery` と同じ形） | `test_cmd_error_req401_settlement_conflict_carries_related_csv_import_id` | `BizError::SettlementConflict { message, csv_import_id: 42 }` の変換が kind `import_error`・message そのまま・`related_csv_import_id == Some(42)` でない。ほかの全ての variant（`ImportError`・`ValidationFailed`・`NotFound`・`StocktakeGuard` ほか）の変換で `related_csv_import_id` が None でない。変換の arm で `None` を返す mutant と、`ImportError` にも ID を入れる mutant で red |
| U1 TRACE-D13（表示条件） | 取消済みに入口が出る・一覧の行に出る | component（RTL） | `CsvImportRecordDetailPage.test.tsx` の `REQ-207: 成功・部分成功の詳細に「この取込みを取り消す」を出し、取消済みには出さない` | `completed`・`completed_partial` の 2 つの mock で button が無い、`rolled_back` の mock で button がある |
| U2 UI-07-D18（読取りの間） | 読み終わる前に押せる | component | `RollbackConfirmDialog.test.tsx` の `REQ-401: 取消の前の読取りの間は「取り消す」を押せず、確かめている文を出す` | preview の Promise を解決しない間に「取り消す」が `disabled` でない、または「取り消した後のようすを確かめています…」が無い |
| U3 UI-07-D18（中身） | 名指しが欠ける・色だけ・因果や差の変化を言い切る・変わりうる列を名指さない・記録の詳細と違う語で名指す・確認の中に入れ直しの操作 | component | `RollbackConfirmDialog.test.tsx` の `REQ-401: 対象の項目・精算回数・要再確認の一覧・取り込み直し・完了した棚卸しを出す` | 非空の preview（要再確認 2 商品〈取消で要再確認になる商品と、取り消す前から要再確認の商品を混ぜた mock〉、完了した棚卸し 1 つ、精算回数 0）で、「精算 0」・2 商品の商品名・範囲・理由の文字と icon（`aria-hidden` の svg と文字の両方）・取り込み直しの文・完了の日付の文のどれかが無い。要再確認の見出しの文が「取り消した後、次の商品は数え直しが必要です（今すでに必要な商品を含みます）」の exact で 1 件でない、または旧い「取り消すと、次の商品は数え直しが必要になります」が出る。完了した棚卸しの文が「完了した棚卸し（{完了の日付}）の評価額は変わりません。棚卸しの記録の詳細の「差異」「ロス原価」と「完了の後に入った記録による差」が変わることがあります。」（日付は mock の値。節の名は b1 の `c667d52c` の `StocktakeRecordDetailPage.tsx` の 265 行の aria-label と同じ文字）の exact で 1 件でない、旧い文に共通の「「完了の後に入った記録による差（参考）」」（round 1 の差異・ロス原価を名指さない文と round 2 の文の両方に入り、記録の詳細の見出しと一字一句は合わない語）が出る、または言い切りの「…が変わります。」が出る（mock は記録の詳細の数が変わらない棚卸しでも同じ文）。確認の一覧に「操作」の列見出し・「棚の数を確かめて入れ直してください。」・「棚卸しの画面で入れ直す」の link のどれかがある（`RecheckTargetList` を `renderAction` なしで使う形）。空の preview（要再確認 0、棚卸し 0）で「取り消しても、数え直しが必要になる商品はありません」が無い、または棚卸しの文が出る |
| U4 UI-07-D18（失敗と取消済み） | 失敗の後に押せる・読めた後の読み直しの失敗で古い preview のまま押せる | component | `RollbackConfirmDialog.test.tsx` の `REQ-401: 読取りの失敗と取消済みでは「取り消す」を押せない` と `REQ-401: 読めた後の読み直しが失敗したら、前の読取りを使わず「取り消す」を押せない` | preview の reject（kind `internal`）で「取り消した後のようすを読み込めませんでした」が無いか「取り消す」が押せる。kind `validation`（「この取込みは取り消し済みです」）で BIZ の文が出ないか押せる。開いたまま 1 回目を解決 → 読み直し（`refetchQueries`）を kind `internal` で失敗させて、失敗の文が無い、1 回目の値（1 回目の要再確認の商品名）が出ている、または「取り消す」が押せる（失敗でも `data` は残るので、`isFetching` だけで採用を決める実装が red） |
| U5 TRACE-D13（成功） | 取消の後が古い・結果が出ない | integration（RTL + QueryClient） | `CsvImportRecordDetailPage.test.tsx` の `REQ-401: 詳細から取り消すと同じ ID で取消し、C9 で再取得し、取消の結果と Z004 の導線を出す` | `rollbackCsvImport` が詳細の ID で 1 回呼ばれない、Toast が `duration: 8000` でない、`src/test/invalidation-oracle.ts` の売上 CSV rollback の key の集合（D-052 C9 の独立の oracle）が invalidate されない、取消の結果の要再確認の一覧（mock の非空の `recheck_targets`）か「商品別CSV取込み（Z004）を開く」（href に `tab=z004`）が無い、再取得の後に「この取込みを取り消す」が残る |
| U6 §55.9（失敗） | 失敗で state が壊れる・別の ID で再試行 | integration | `CsvImportRecordDetailPage.test.tsx` の `REQ-401: 取消の失敗は Toast で、同じ ID のまま再試行できる` | 1 回目の reject の後に Toast「取り消しに失敗しました。もう一度お試しください」が無い、2 回目の `rollbackCsvImport` の引数が 1 回目と違う、詳細が取消済みの表示になる |
| U7 UI-07-D18（同じ確認） | 結果の画面だけ古い確認 | component | `ResultStep.test.tsx` の `REQ-401: 結果の画面の「取り消す」は共通の確認で、結果の取込みID で読む` | 「取り消す」で `previewRollbackCsvImport` が `result.csv_import_id` で呼ばれない、確認に精算回数と要再確認の一覧が出ない、今の確認の文（「この取込みによる在庫数も元に戻ります。」）が残る |
| U8 UI-07-D18（導線と `tab`）・UI-STATE-D2 | Z004 タブが開かない・link が違う・route が値の集合を別に持つ | integration（実 router。route は feature の `csvImportSearchSchema` を import した `validateSearch` を通る）+ 同一性（`Route.options.validateSearch`）+ static（AC3 の 4 つの `rg`） | `CsvImportPage.test.tsx` の `REQ-401: tab=z004 で Z004 タブを開き、不正な tab は既定、取り込んだ記録の link は種別 CSV取込み` と AC3 の `rg` | `/csv-import?tab=z004` の render で Z004 タブが選ばれていない、`tab=zzz` で日報取込みでない、「取り込んだ記録を見る（取り消すときはここから）」の href が `/inventory/records?recordType=csv_import` でない。`expect(Route.options.validateSearch).toBe(csvImportSearchSchema)`（`Route` は `src/routes/csv-import/index.tsx`、schema は `src/features/csv-import/types.ts` から import）が通らない（route の中で同じ値を受ける別の schema〈`z.union([z.literal("daily-report"), z.literal("z004")])` など〉を作る mutant で red。振舞いの 3 つの期待はこの mutant でも通るので、同一性の検査が要る）。static: `validateSearch: csvImportSearchSchema` が無い、route に feature の types の import が無い、route に `z004` の文字がある、または `src/features/csv-import/types.ts` に `CSV_IMPORT_TAB_VALUES` の定義が無い（UI-STATE-D2） |
| U9 UI-07-D18（mutation を 1 つに） | 取消の処理を 2 つ持つ | static（AC3 の `rg`）+ 今の flow の test | AC3 の `rg -l 'commands.rollbackCsvImport' …` と `useCsvImportFlow.test.tsx` の今の取消の test（T3） | `rg` の出力が `useRollbackCsvImport.ts` の 1 行でない。flow の取消の test の期待（成功で idle と取消の結果、失敗で result のまま）が変わる |
| U10 UI-07-D18（cache を使い回さない） | 開き直して古い preview・読み直しの間に押せる | component | `RollbackConfirmDialog.test.tsx` の `REQ-401: 確認を開くたびに取消の前の読取りをやり直す` | 開く → 1 回目を解決 → 閉じる → 開くで、2 回目の Promise を解決しない間に「取り消す」が `disabled` でない、または 1 回目の値（1 回目の要再確認の商品名）が出ている。2 回目を解決した後に `previewRollbackCsvImport` の呼出しが 2 回でない、または 2 回目の mock の値〈要再確認の商品を替えた〉が出ない |
| U11 UI-07-D18（順 4 の拒否からの導線） | 拒否の Alert から名指した取込みへ進めない・別の番号で開く・link の取込みID を文から読む・ID の無い error に link | integration（実 router。`CsvImportPage` を render し、`parseAndValidateCsv` の mock が error を返す） | `CsvImportPage.test.tsx` の `REQ-401: 同じ精算の別の版の拒否の Alert から、名指した取込みの詳細へ進める` | 同じ test の中の 6 つの case。link がある側の照合は Alert（`getByRole("alert")`）の中に限り（`within`）、link が無い側の照合は Alert の中の link と、画面全体の href が `/csv-import/records/` で始まる link の両方を見る（Z004 タブの上の U8 の「取り込んだ記録を見る（取り消すときはここから）」は `/inventory/records` へ進むので、どちらにも当たらない）。(1) mock の error（kind `import_error`、message「同じ精算と思われる別の資料が既に取り込まれています（精算 101、取込みID: 7）」、`related_csv_import_id: 7`。取込みID と精算回数を別の値にする）で、Alert の中に「取込みID 7 の記録を見る（取り消すときはここから）」の link が無い、href の pathname が `/csv-import/records/7` でない、または search の `returnTo` が `/csv-import?tab=z004` でない。(2) 文の取込みID と field を別の値にした mock（message「同じ精算と思われる別の資料が既に取り込まれています（精算 101、取込みID: 9）」、`related_csv_import_id: 7`）で、Alert の中の link の文が「取込みID 7 の記録を見る（取り消すときはここから）」でない、href の pathname が `/csv-import/records/7` でない、または `/csv-import/records/9` への link がある（文から取込みID を読む実装と、field で link を出すかを決めて id だけ文から読む混成の実装が red）。(3) wire と同じ null の mock（(1) と同じ順 4 の文、`related_csv_import_id: null`。b1 の `c667d52c` の `src-tauri/src/cmd/mod.rs` の 71〜84 行の `CmdError` は `skip_serializing_if` を持たず、None は `null` で届く）で、Alert の中に link がある、または href が `/csv-import/records/` で始まる link がある（`!== undefined` や `"related_csv_import_id" in` で出す実装、順 4 の文の頭で出す実装、文の取込みID で出す実装が red）。(4) field の無い mock（順 3 の実際の文。b1 の `c667d52c` の `src-tauri/src/biz/csv_import_service/parse.rs` の順 3 の preview の文 `format!("このファイルは取込み済みです（取込みID: {}、取込み日: {}）", …)` の形で取込みID 5。field の無い形は `c667d52c` の `src/lib/invoke.ts` の 86〜92 行の `toCmdError` の internal への正規化と、既存の mock の形）で、(3) と同じ条件（`!== null` で出して `/csv-import/records/undefined` を作る実装と、文の取込みID で出す実装が red）。(5) 順 4 でない文に field がある mock（(4) と同じ順 3 の文で取込みID 5、`related_csv_import_id: 7`。W3 の契約では来ない入力で、出す条件が field だけであることを固める）で、Alert の中に「取込みID 7 の記録を見る（取り消すときはここから）」の link が無い、href の pathname が `/csv-import/records/7` でない、または `/csv-import/records/5` への link がある（field と順 4 の文の両方を条件にする積の実装が red）。(6) 取込みID も数字も含まない文に field がある mock（kind `import_error`、message「合成の拒否の文」、`related_csv_import_id: 7`。W3 の契約では来ない入力で、出す条件が文の語に依らないことを固める）で、Alert の中に「取込みID 7 の記録を見る（取り消すときはここから）」の link が無い、または href の pathname が `/csv-import/records/7` でない（field と「取込みID」の語の両方を条件にする積の実装が red。(1)〜(5) の文はどれも「取込みID」を含むので、この実装は (1)〜(5) を通る）。どの case でも Alert の見出し「プレビューが利用できません」と「最初に戻る」が変わる |
| U12 UI-07-D18・§55.7（取消の応答を待つ間の route の移動とリロード等の離脱の止め。共有の hook。Gated Amendment 2） | 取消の応答を待つ間に離れられる・失敗の後も止めが残る | unit（hook。`useCsvImportFlow.test.tsx` の今の `vi.mock("@tanstack/react-router", () => ({ useBlocker: vi.fn() }))` の形で、`useCsvImportFlow` が呼ぶ共有の hook の options を読む） | `useCsvImportFlow.test.tsx` の `REQ-401 / UI-07-D18: 取消の応答を待つ間は useBlocker と beforeunload で画面を離れず、失敗すれば離れられる` | result の state で `rollback(id)` を呼び、`rollbackCsvImport` の Promise を解決しない間に、最後の render が登録した全ての `useBlocker` の options（importing の止めと取消の止めの 2 つ）のうち、`shouldBlockFn()` が true のものが無い、または `enableBeforeUnload()`（関数なら呼んだ値）が true のものが無い。Promise を error で解決し失敗の Toast が出た後に、どちらかが true のまま残る（state は result のまま）。修正前は取消の止めが無く、応答を待つ間に全て false で red。mutant: hook の `useBlocker` を外す・`shouldBlockFn` を常に false・`enableBeforeUnload` を外す（mock の options に無い）は応答待ちの側で、`shouldBlockFn` と `enableBeforeUnload` を常に true は失敗の後の側で red。詳細の入口のリロード等の離脱の止めは同じ hook なので、この行と AC3 の `useBlocker(` の数（hook に 1）で見る |
| U13 TRACE-D13・§55.7（詳細の入口で止まる。Gated Amendment 2） | 止めが Z004 の flow にしか無く、詳細から離れられる | integration（実 router。`routeTree` と `createMemoryHistory` で `/csv-import/records/41` を開く。今の `CsvImportRecordDetailPage.test.tsx` の 165 行の形） | `CsvImportRecordDetailPage.test.tsx` の `REQ-401: 取消の応答を待つ間は詳細の画面から移れず、失敗すれば移れる` | 確認の「取り消す」を押し、`rollbackCsvImport` の Promise を解決しない間に「前の画面へ戻る」の link を押すと、`router.state.location.pathname` が `/csv-import/records/41` でない（見出し「CSV取込み #41」が消える）。Promise を error で解決し失敗の Toast が出た後に同じ link を押しても pathname が戻り先に変わらない。memory history の止めは push と replace だけに効く（`@tanstack/history` の `isPushOrReplace`。`history.back()` は止まらない）ので、link の押下か `router.navigate` で試す。修正前は止めが無く red。mutant: 止めを `useCsvImportFlow` だけに置く（共有の hook に置かない）で red |
| U14 UI-07-D18（Z004 タブの切替えを止める。Gated Amendment 2） | 取消の応答を待つ間にタブを切り替えて flow ごと消える | integration（実 router。`CsvImportPage.test.tsx` の今の `useCsvImportFlow` の mock〈`flow()`〉で `isRollingBack` を渡す） | `CsvImportPage.test.tsx` の `REQ-401: 取消の応答を待つ間はタブを切り替えられない` | `/csv-import?tab=z004` を `flow({ state: result, isRollingBack: true })` で render して、2 つのタブ（`getByRole("tab")`）のどちらかが `toBeDisabled()` でない、または「日報取込み」を押すと「商品別CSV取込み（Z004）」の `aria-selected` が `"true"` でなくなる（日報の mock の中身が出る）。`isRollingBack: false` の同じ render で「日報取込み」を押しても日報取込みに切り替わらない。修正前は disabled が無く red。mutant: disabled を外すは応答待ちの側、常に disabled は応答を待たない側で red |
| U15 TRACE-D13（取消の結果は表示している取込みのもの。Gated Amendment 2） | 別の取込みの詳細へ移っても前の取消の結果が残る | integration（実 router。`routeTree` と `createMemoryHistory`。route の param の変化で同じ route の component に新しい `importId` が届く形） | `CsvImportRecordDetailPage.test.tsx` の `REQ-401: 取り消した後に別の取込みの詳細へ移ると、前の取込みの取消の結果を出さない` | `/csv-import/records/41` で取消を成功させ（取消の結果の「数え直しが必要な商品」と「商品別CSV取込み（Z004）を開く」が出る。非空の `recheck_targets`）、`router.navigate({ to: "/csv-import/records/$importId", params: { importId: "42" } })` で有効な取込み 42 の詳細へ移ると、見出しが「CSV取込み #42」になった後に、「取込みを取り消しました」・「数え直しが必要な商品」・「商品別CSV取込み（Z004）を開く」のどれかが出る、または「この取込みを取り消す」が押せない・「取り消し中…」になっている。修正前は state が残って red。mutant: `key={importId}` を外すで red |

## 既存の test の移し方

着手時に下の command を逐語で回し、出力の各行を分類に当てた表を PR の本文に貼る（b1 の merge の後の版で回す。b1 の実装で test の名前が変わるので、本 packet は分類の規則だけを置く）。

| # | command | 分類 |
|---|---|---|
| T1 | `rg -n 'fn test_.*rollback' src-tauri/src/biz/csv_import_service/tests/rollback_tests.rs src-tauri/src/cmd/csv_import_cmd.rs` | 移す（期待を変えない）: 取消の本体を切り出すだけなので、b1 の取消の test は全部そのまま PASS する |
| T2 | `rg -n "it\\(|test\\(" src/features/csv-import/components/ResultStep.test.tsx` | 確認の中身を見る case は期待を変える（契約の変更: 55 UI-07-D18。今の確認の文と dl の assert を共通の確認の assert に。代わりに同じ失敗を止める行は U7）。結果の表示・要再確認の一覧（入れ直しの文と「棚卸しの画面で入れ直す」の link。b1 の `c667d52c` の 143〜148 行。`RecheckTargetList` に `renderAction` を渡す形の回帰）・「売上レポートを見る」・「ホームに戻る」の case は移す |
| T3 | `rg -n "it\\(|test\\(" src/features/csv-import/hooks/useCsvImportFlow.test.tsx` | 取消の case は移す（期待を変えない。mutation の置き場が共有の hook に移るだけで、成功の dispatch・`rollbackResult`・失敗の Toast と state は同じ）。mock の import 先だけを直す |
| T4 | `rg -n "it\\(|test\\(" src/features/inventory-records/CsvImportRecordDetailPage.test.tsx` | 移す（期待を変えない）: 今の表示・戻り導線（DSR-18）・取消済みの Alert の case。mock の command に `previewRollbackCsvImport`・`rollbackCsvImport` を足す |
| T5 | `rg -n '既に取り込まれています（精算' src-tauri/src/biz/csv_import_service/tests src-tauri/src/cmd/csv_import_cmd.rs` と、順 4 を通る照合を拾う `rg -n -e 'BizError::ImportError' -e 'fn import_error' src-tauri/src/biz/csv_import_service/tests` | 期待を変える（契約の変更: 32 の表の順 4 の文に取込みID を足し、`BizError::SettlementConflict` で返す。2 本目の出力は、順 4 で拒否する入力の行だけを `SettlementConflict` の照合へ移し、順 1〜3・5 の行〈`c667d52c` では `commit_tests.rs` の 725 行の順 5、`cross_feature_tests.rs` の 259 行の順 3 ほか〉は移す〈期待を変えない〉。b1 の `c667d52c` では `parse_tests.rs` の 664 行・`commit_tests.rs` の 682 行・`csv_import_cmd.rs` の 341 行の 3 件）。3 test とも名指す取込みの前に別の取込みを 1 件足す（fixture の変更。理由と形は RB7）。同じ失敗（ID を名指さない・別の ID）を止める行は RB7 |
| T6 | `rg -n 'fn test_complete_stocktake_cmd_req205_completion_blocked_wire' src-tauri/src/cmd/stocktake_cmd.rs`（Gated Amendment 1） | 期待を変える（契約の変更: W3。`CmdError` の wire に `related_csv_import_id` が全部の kind で出て、順 4 の拒否のほかは `null`）。期待 JSON に `"related_csv_import_id": null` の 1 key を足すだけで、test の名前・REQ-205 の comment・照合の強さ（JSON 全体の完全一致）は変えない。部分の照合に弱めない。同じ失敗（W3 の key が wire から消える・`null` でない値が出る）を止める行は W3 |
| T7 | `rg -n 'mockUseBlocker' src/features/csv-import/hooks/useCsvImportFlow.test.tsx`（Gated Amendment 2） | 選び方を変える（期待は変えない）: 今の `REQ-401: commit 中だけ useBlocker と beforeunload を有効化する` は `mockUseBlocker.mock.calls` の最後の呼出しを読む（2026-10-27 の本 worktree の 336 行）。共有の hook が `useCsvImportFlow` の importing の止めの後に `useBlocker` を呼ぶので、最後の呼出しは取消の止めになり、importing の間に false を返して red になる。最後の render の全ての `useBlocker` の options を読み、importing の間に `shouldBlockFn()`・`enableBeforeUnload()` が true のものがあることを見る形にする（U12 と同じ読み方）。止めの意味〈どれかの blocker が true なら止まる。`@tanstack/history` の `blockers` を順に見て 1 つでも true なら止める〉と同じで、importing の止めを外すと red のまま（弱めない）。同じ失敗を止める行は U12 |
| T8 | `rg -n 'T16' src/hooks/unsaved-changes-guard-sweep.test.ts`（Gated Amendment 3） | 期待を変える（契約の変更）: T16 の期待の集合（`useBlocker` を直接使う production の file の完全一致）に `src/features/csv-import/hooks/useRollbackCsvImport.ts` を 1 行足し、題名の「既存2 hook」を UI_TECH_STACK UI-USW-D1 と同じ持ち主の言い方にする。照合の形（src の全走査と完全一致）は変えない。置き換えた正本の節は UI-USW-D1 の例外の言い方。同じ失敗（5 つ目の `useBlocker` が黙って増える）を止めるのはこの T16 のまま |

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 確認（preview） | 閉じている、読取りなし | 開く → 読取り中（押せない） | 中身を出し、押せる | 取消の C9（`inventoryRecords.root()`）が key を含む | 開き直すたびに読む（`gcTime: 0`） | 閉じて開くと読み直す | 読取りは保存しない | 文を出し押せない | 閉じて開く | U2・U3・U4・U10 |
| 詳細の取消 | 「この取込みを取り消す」（active） | 取消中（button に spinner、押せない）。route の移動とリロード等の page の離脱を止める（共有の hook の `useBlocker`。成功の Toast と C9 の再取得、または失敗の Toast が終わるまで。React Query v5 は options の `onSuccess` / `onError` を待ってから pending を外す） | Toast、取消の結果、Z004 の導線 | C9 の key 全部 | 詳細が取消済み、入口が消える | 一覧から開き直しても取消済み。別の取込みの詳細へ移ると、前の取込みの取消の結果を出さない（`key={importId}` で作り直す） | DB に残る（`rolled_back`） | Toast、state を変えない、止めを解く | 同じ画面で同じ ID | U1・U5・U6・U13・U15 |
| 結果の画面の取消 | result | 取消中。route の移動・リロード等の page の離脱（共有の hook）とタブの切替え（`TabsTrigger` の disabled）を止める | idle と取消の結果（b1） | C9 | b1 のまま | b1 のまま | b1 のまま | Toast、result のまま、止めを解く | 同じ画面で同じ ID | U7・U12・U14・T3 |
| `tab` の search | 欠落 = 日報取込み | — | `z004` で Z004 タブ | — | — | 再 mount 時は search のとおり。同じ mount の間に戻る・進むで search が変わっても、タブの状態を保つ（`defaultValue` は初期選択だけ。55 §55.11） | URL のまま | 不正な値は既定 | — | U8 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| destructive の確認 dialog（DSR-20、`AlertDialogAction variant="destructive"`） | `rg -l 'AlertDialogAction' src -g '!*.test.*'` の 11: `src/features/stocktake/StocktakePage.tsx`・`src/features/backup-restore/BackupRestorePage.tsx`・`src/features/daily-report-import/DailyReportImportPage.tsx`・`src/features/integrity-check/IntegrityCheckPage.tsx`・`src/features/csv-import/components/AdditionalImportConfirmDialog.tsx`・`src/features/csv-import/components/ResultStep.tsx`・`src/components/patterns/UnsavedChangesDialog.tsx`・`src/components/ui/alert-dialog.tsx`・`src/features/products/import/ProductImportPreview.tsx`・`src/features/products/components/DiscontinueConfirmDialog.tsx`・`src/features/products/components/PluBulkTargetConfirmDialog.tsx` | `ResultStep.tsx` の取消の確認を共通の確認へ | 日報の取消（在庫を動かさず要再確認が出ない。読取りの要る中身が違う）、ほかは別の操作 | U2・U7 |
| 取消の成功の Toast の表示時間 8 秒 | `rg -n 'duration: 8000' src -g '!*.test.*'` の 2: `src/features/daily-report-import/hooks/useDailyReportImportFlow.ts`・`src/features/csv-import/hooks/useCsvImportFlow.ts` | Z004 の分を共有の hook へ移す（詳細も同じ） | 日報の分は日報の取消のまま | U5 |
| 取消の再取得（D-052 C9） | `rg -n 'csvImportRollback\(' src -g '!*.test.*'` の 1: `src/features/csv-import/hooks/useCsvImportFlow.ts` | 共有の hook へ移す | — | U5 |
| 詳細の戻り導線 `returnTo`（DSR-15・DSR-18） | `src/features/inventory-records/CsvImportRecordDetailPage.tsx` の `returnToLinkProps` | 変えない（取消の後も同じ戻り先） | — | T4 |
| `CmdError` に kind 別の構造の field を足す（先例: b1 の `stocktake_recovery`。`stocktake_guard` だけ Some） | `c667d52c` の `src-tauri/src/cmd/mod.rs` の `From<BizError>` と、`CmdError` を literal で作る 23 か所（`git grep -c -e 'error_id: None' -e 'error_id: Some'`、6 file）、TS の `stocktake_recovery` を書く 42 file（`git grep -l 'stocktake_recovery' -- 'src/**'`）、`CmdError` の wire 全体を照合する既存の test（`rg -n 'serde_json::to_value\(&(error|cmd_err|err)' src-tauri/src` の完全一致は `src-tauri/src/cmd/stocktake_cmd.rs` の `test_complete_stocktake_cmd_req205_completion_blocked_wire` の 1 本、b1 Matrix W4。Gated Amendment 1 で足した） | `related_csv_import_id`（`SettlementConflict` だけ Some）。Rust の 23 か所に None。W4 の test の期待 JSON に `"related_csv_import_id": null`（T6） | TS は任意の field（`?:`）にして 42 file を変えない（先例の必須の field と違う理由は packet の「起票時に起草役が決めたこと」の 8） | W3、AC2 |
| 応答を待つ間の離脱の止め（`useBlocker` + `enableBeforeUnload`。Gated Amendment 2） | `rg -n 'useBlocker\(' src -g '!*.test.*'` の 3（Gated Amendment 3: production の path の集合を test の中に固定する型〈`src/hooks/unsaved-changes-guard-sweep.test.ts` の T16〉もあるので、`rg -ln 'useBlocker' src --glob '*.test.*'` で test も数える）: `src/features/csv-import/hooks/useCsvImportFlow.ts`（importing）・`src/features/daily-report-import/hooks/useDailyReportImportFlow.ts`（importing。PR #172 の head `f0d6d1b0` では取消の応答を待つ間も）・`src/hooks/useUnsavedChangesWarning.ts`（未保存の form） | `src/features/csv-import/hooks/useRollbackCsvImport.ts`（Z004 の取消。2 つの入口に共通） | `useCsvImportFlow` の importing の止めは今のまま（取込みの応答）。日報取込みタブの取消の止めは #172 の所有（UI-07-D15 (6)）。`useUnsavedChangesWarning` は別の操作 | U12・U13、T7 |
| タブの切替えでの離脱（Radix の `TabsContent` は選ばれていないタブを unmount する〈`node_modules/@radix-ui/react-tabs/dist/index.mjs` の 145〜157 行の `Presence` の `present: forceMount \|\| isSelected`〉。router の遷移でないので blocker が効かない。Gated Amendment 2） | `rg -l 'TabsTrigger' src -g '!*.test.*' -g '!src/components/ui/**'` の 1: `src/features/csv-import/CsvImportPage.tsx` | Z004 タブの取消の応答を待つ間の `TabsTrigger` の disabled | 取込みの応答を待つ間（両タブの importing）と、日報取込みタブの取消の応答を待つ間のタブの切替えは、本 lane の差し戻しの範囲の外（同じ形の穴で、Coordinator へ報告。本 lane では足さない） | U14 |
| route の param が変わっても page の state が残る（同じ route の component が再 mount されない。Gated Amendment 2） | `rg -n 'remountDeps' src` は 0 件。param を持つ route の 8 file（`src/routes/csv-import.records.$importId.tsx`・`stocktake.records.$stocktakeId.tsx`・`stock/$code.movements.tsx`・`products/$code.edit.tsx`・`inventory/disposal/records/$recordId.tsx`・`inventory/manual-sale.records.$recordId.tsx`・`inventory/receiving.records.$recordId.tsx`・`inventory/return.records.$recordId.tsx`）は、どれも `key` を渡さない | `src/features/inventory-records/CsvImportRecordDetailPage.tsx`（`key={importId}`） | ほかの 7 route の page（`ProductFormPage.tsx` の `useState` ほか）は本 lane の差し戻しの範囲の外（同型の深掘りの候補として Coordinator へ報告） | U15 |
| TX を明示に閉じる（rollback が成功の経路。Gated Amendment 2） | `rg -n '\.rollback\(\)\|\.finish\(\)' src-tauri/src -g '!**/tests/**'` の production の 1: `src-tauri/src/biz/csv_import_service/rollback.rs` の preview（ほかは `src-tauri/src/db/plu_slot_repo.rs` の `#[cfg(test)]` の中の 1 と、tracing の builder の `.finish()` 2） | preview の手順 5 を `tx.finish()` に | COMMIT が成功の経路の書込み（失敗は drop の rollback）は repo の今の形で、32 §15.6b の手順 5 の「drop に頼らない」は preview だけの契約 | RB2、AC1 |
| route の search の検証（zod の `.catch(undefined)`）と、有限の値の集合の所有（UI-STATE-D2: feature の `types.ts` / `search.ts` が tuple と schema を持ち、route は import する） | `src/routes/csv-import.records.$importId.tsx` の `searchSchema`、`src/features/inventory-records/types.ts` の `inventoryRecordsSearchSchema`、`src/features/daily-sales/types.ts` の `dailySalesSearchSchema`（`src/routes/reports/daily.tsx` が import）、`src/features/monthly-sales/types.ts` の `monthlySalesSearchSchema`、`src/features/products/search.ts` の `productListSearchSchema` | `src/features/csv-import/types.ts` に tuple と schema、`src/routes/csv-import/index.tsx` は import | — | U8、AC3 |

## Negative Paths

- missing input: 無い取込みID の preview（RB3・W1）。
- invalid input: 不正な `tab`（U8）。
- duplicate/ambiguous input: 取消済みの取込みの preview（RB3・U4）と、取消済みへの重ねた取消（冪等の Ok、T1）。
- unknown reference: 完了した棚卸しが無い DB（RB6 の空の期待）。取込みID を構造で持たない import_error（wire の `null` と field の無い形。順 4 の文や、文に取込みID がある順 3 の文でも。U11 の (3)・(4)、W3 の None）。
- dependency missing: preview の読取りの失敗（U4）。
- permission/write failure: 取消の失敗（U6。失敗の後に止めが解ける U12・U13）。preview の手順 4 の失敗と戻しの失敗は code 読み（下の Residual Test Gaps）。
- dry-run side effect: preview が書く（RB2）。

## Boundary Checks

- threshold: 無い。
- null/default: `tab` の欠落は日報取込み（U8）。`affected_stocktakes` が空なら文を出さない（U3）。
- empty/non-empty: `recheck_targets` の空と非空（U3・RB5）。
- min/max: 精算回数 0（RB4・U3）。
- status/policy enum: 取込みの状態 3 つ（`completed`・`completed_partial`・`rolled_back`）で入口の有無（U1）。
- wire type: `RollbackPreview`・`AffectedStocktake`、`CmdError.related_csv_import_id`（任意。AC2・W3）。
- internal type: 切り出した取消の本体の関数（T1・RB1）。
- producer/consumer: BIZ → CMD → bindings → 確認（W1・U3）。順 4 の拒否: BIZ → CMD → `ErrorState`（RB7・W3・U11）。
- round-trip token: `tab` の search（U8）。
- precision/range: 精算回数は b1 の範囲の内で number（RB4）。
- cross-language parse: 無い。

## Compatibility Checks

- old schema/input: 今の `RollbackResult`・`ImportResult`・`CsvImportRecordDetail` の形は変えない（AC2 の bindings の再生成で今の型の diff が無い）。`CmdError` は任意の field を 1 つ足すだけ（kind の集合と今の field は変えない。TS の今の literal は変えずに型が通る〈AC4 の typecheck〉）。
- new schema/input: `RollbackPreview`・`AffectedStocktake`、`CmdError.related_csv_import_id`。
- output order: `affected_stocktakes` は完了の日時の昇順（RB6）。`recheck_targets` の順は b1 の取消と同じ（RB1）。
- optional field behavior: 新しい型は nullable の field を持たない。`CmdError.related_csv_import_id` は任意（無い・null は link を出さない。U11 の (3) が null、(4) が無い形）。

## Data Safety Checks

- source-derived data: 無い（実データを使わない）。
- generated outputs: `src/lib/bindings.ts`・`docs/function-design/90-traceability.md`（再生成だけ）。
- secrets: 無い。
- local-only files: L3 の DB と fixture は repo の外。
- synthetic sample boundaries: Rust・TS の test の値と b1 の L3 の fixture は合成。

## Main Wiring / Integration Checks

- helper connected to main path: 切り出した取消の本体を取消と preview の両方が呼ぶ（RB1・T1）。
- shared hook reaches both entries: 取消の応答を待つ間の止めは共有の hook の 1 か所で、取込みの結果の画面（U12）と CSV取込みの詳細（U13）の両方に効く（Gated Amendment 2）。
- output reaches manifest/report: 該当なし。
- effective config reaches runtime: 該当なし。
- CLI arg reaches implementation: 該当なし。command の登録（W2）。

## Mutation-style Adequacy Questions

mutation は対象経路の観測結果を変えるものを選び、既存の適用条件に従って実注入で red を確認する。effect 等が変化を打ち消す場合は別経路で単独に効くか調べる。観測不能な mutant の kill を固定 AC にせず、承認済み AC の訂正は Coordinator へ返す。

- If a mock value is changed so it differs from the design-doc expected value, which assertion proves the implementation used the correct source and not the mock's accidental constant? → RB7 は名指す取込みの ID を 1 以外にした fixture で、ID を固定値 1 にする mutant を red にし、cmd の test は名指す ID を最大でもない値にして最新の ID を名指す mutant を red にする。U11 は取込みID と精算回数を別の値にした mock で link の id の取り違えを見る。link の id を文から取る実装は、文の取込みID と field を別の値にした (2)・(5) が red にする。出す条件を文から取る実装は、文の取込みID で出す形を (3)・(4)、順 4 の文の頭で出す形を (3)、field と順 4 の文の積を (5)、field と「取込みID」の語の積を (6) が red にする。field の有無の判定を誤る実装は、`!== undefined`・`"related_csv_import_id" in` を (3)、`!== null` を (4) が red にする。U8 は `Route.options.validateSearch` の同一性で、route の中の同じ振舞いの別 schema を red にする。U3 は preview の mock の値（精算回数 0、商品名）がそのまま画面に出ることを見る。U7 は `result.csv_import_id` と違う ID の mock を置き、呼出しの引数を見る。
- If invalidate/refetch changes the value before versus after the operation, which test proves the lifecycle order and preserved snapshot are correct? → U5（取消の前は入口があり、C9 の再取得の後に取消済み）。U15（別の取込みの詳細へ移った後に、前の取込みの取消の結果を持ち越さない）。
- If a key branch is inverted, which test fails? → U1（`rolled_back` の判定）、U4（押せる条件）。
- If a threshold comparison changes, which test fails? → RB6 の (f)（「精算回数 ≤ n」の等値。`<=`→`<`）と (g)（「精算日 < 完了日」の等値。`<`→`<=`、日付の部分で比べること）。どちらも OR のもう片側が偽の fixture。
- If a guard is removed, which test fails? → RB3（取消済みの拒否）、U2（読取りの間の disabled）、U10（開き直した読取りの間の disabled）、U12・U13（取消の応答を待つ間の離脱の止め）、U14（タブの disabled）。
- If an output field is omitted, which test fails? → RB4・RB6・RB7（順 4 の文と構造の取込みID）・W3（`related_csv_import_id`）・U3・U11。
- If output order changes, which test fails? → RB6（完了の日時の昇順）。
- If dry-run performs a side effect, which test fails? → RB2。
- If a JSON number crosses JavaScript safe integer range, which test fails? → 該当なし（精算回数は b1 の入力の境で範囲の内。b1 の Z5）。
- If a state token is round-tripped through browser/client code, which test fails? → U8（`tab`）。

## Residual Test Gaps

- preview の失敗の経路は注入の手段が無い（`apply_rollback` を途中で失敗させる道具も、SQLite の ROLLBACK を失敗させる道具も足さない）。Final Review の code 読み（Gated Amendment 2）: 成功の経路も手順 4 の失敗の経路も `tx.finish()` で明示に閉じる（drop に頼らない）。閉じる失敗は握りつぶさず `BizError::DatabaseError` で先に返し、閉じられたときは手順 4 の失敗をそのまま返す（SQLite が自動で戻した TX では `finish` が autocommit を見て何もしないので真因を覆わない。無条件の `tx.rollback()` は「no transaction is active」で真因を覆う）。TX の外で何も書かない。静的には AC1 の `tx\.finish\(\)` が 1 と `apply_rollback\(&tx, csv_import_id\)\?` が 1（preview に `?` の早期に返る形が戻ると 2）。
- 取消の応答を待つ間の止めの browser の戻る・進む（popstate）と、リロード等の page の離脱での `beforeunload` の効き方は自動 test にしない（test の memory history は push と replace だけを止め、`beforeunload` を張らない）。U12 は `enableBeforeUnload` に渡す値、U13 は push の止めで見る。native の window の close（×・Alt+F4）は止める契約でない（55 §55.7 の「止める範囲」、UI-USW-D4。Gated Amendment 4）ので test の対象にしない。
- preview と取消の間に別の記録が入る競合（残りの危険の 1 つ目）は自動 test にしない（単店舗の 1 台の PC で、取消の結果の一覧が正であることは U5 で見る）。
- preview の待ち時間（商品・計数の多い DB）は測らない（見直しの契機は 32 §15.6b）。
