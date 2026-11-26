# Plan Packet: EJ の控えの画面（(A-2)、関門 19）

wave に属さない単独の lane。出所は [backlog](../backlog.md) の「SD の一括取込みと EJ の控えの runtime（A）」の (A-2) の段と D-121、owner の決定（[Plans の導入の前の関門](../../Plans.md#導入の前の関門) の関門 19。repo 外の回答台帳 TD-838: 導入の前、UI の刷新の前に、画面に要る構成要素を絞った形。TD-296: test の EJ は合成の fixture だけ）、(A-1) の [packet](2026-11-20-sd-bulk-ej-runtime-a1.md) と PR #294 の申し送り。起点は `1edcedc3`（origin/main、2026-11-24）。計画の review と実装は次の session。

## Workflow State

- Risk: R3
- Plan Commit: a51545519dbd9990794867234802574805e7e9a0
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R3

Reason:
新しい route と search の状態（`/reports/receipts` の `date`・`no`）、状態を持つ画面、Tauri command 3 本の DTO と生成 bindings、D-052 の invalidation の entry（DEV_WORKFLOW の Risk Tiers の R3）。R4 でない理由: 読むだけの command で、書くのは (A-1) の `import_ej_files`（insert-only）の再利用だけ。migration・削除・復元に触れない。

## Goal

Goal Invariant:

### 最小完了条件

- 店の人がサイドバーの「レシートの控え」を開くと、記録のある最も新しい日の控えが番号の順の表で出て、行を選ぶとその控えの文が紙のレシートと見比べられる等幅の並びで出る（SCREEN_DESIGN UI-09c-D1・D2・D4）。
- 前の日・次の日（記録のある隣の日）と日付の入力で日を移れ、番号を入れて Enter で、その控えの日に移って行が選ばれる（UI-09c-D2・D3）。
- 控えに欠けがあれば一覧の上の Alert が区間で名指しし、その場の「EJ のファイルを選んで取り込む」で埋めると Alert が消える（UI-09c-D5）。
- 売上データ取込みの一括の結果から「レシートの控えを見る」と、食い違いで前の控えを残した番号の控えへ移れる（55 UI-07-D17 の (A-2) の分）。

### 失敗定義

- 控えの文の行末の空白が消える・折り返す・半角と全角の混じる行の桁がずれる（店主が紙と見比べられない）。
- 不正な `date`・`no` の search で画面が error になる、または別の日・別の番号の控えを出す。
- 取込みの後に画面が古い一覧・古い欠けのまま残る（invalidation の欠け）、または控えを読まない画面の query まで無効にする（D-052 の過剰）。
- 実の EJ・POS file・DB を fixture にする。

### 非目的

- 金額・商品名で探すこと（D-121 の Revisit）、控えの印刷・書出し、EJ を在庫・売上に使うこと（ADR SPEC-STK-CNT-D11）。
- 取込み済みの EJ を `XZ_BKUP` へ移す操作（backlog の「(b) の lane」）、(B) の Z004 の分、EJ の小数の数量（IO-08.5）。
- 保留の文と保留だけで押せる「まとめて取り込む」の見直し（UI の刷新へ送った。owner の判断 TD-858）。
- 売上レポートの `TabsHeader`（日次・月次）に 3 つ目のタブを足すこと（到達は navigation の 1 項目）。

## Scope

確かめた事実（2026-11-24、`1edcedc3`）: 閲覧の型と 5 つの repo の関数・3 つの BIZ の関数・3 つの command は未実装（`rg -n 'list_ej_records|find_ej_records_by_no|get_ej_record|EjRecordSummary' src-tauri/src src/lib/bindings.ts` が 0 件）。`ej_repo.rs` の mode・kind の enum は `Serialize` だけで、DB の TEXT から読む match が無い。`queryKeys` に `ejRecords` は無い。手で選ぶ EJ の取込みは `useDailyReportImportFlow.ts` の `ejImportMutation`・`chooseEjFiles`・`ejPending`（`:215-310`）の中にだけあり、成功の後の invalidation は無い。`design_compliance_test.rs` の 24 の行は `db::ej_repo`、37 の行は `::ej` を既に持つ。

- S1 repo（`src-tauri/src/db/ej_repo.rs`、24 §14.25 の閲覧の分）: `EjRecordSummary`・`EjRecordDetail`・`EjRecordRef` と、`list_ej_records_on_date`（最大 1001 件）・`find_ej_adjacent_dates`・`find_latest_ej_date`・`find_ej_records_by_no`・`get_ej_record_detail`。SQL は 24 §14.25 の SQL 方針のまま。mode・kind は明示の match で読み、外れた値は `DbError`（先例 `sales_repo.rs` の `row_to_csv_import`）。型は `biz/mod.rs` で re-export（今の `NewEjRecord` 等と同じ）。enum と CHECK の parity は (A-1) の `finite_enum_contract_tests.rs` の ej の 2 enum に読む側の match を足す。
- S2 BIZ（`src-tauri/src/biz/daily_report_import_service/ej.rs`、37 §37.11 の「処理ステップ（閲覧）」）: `EjRecordDay` と `list_ej_records`・`find_ej_records_by_no`・`get_ej_record`。日付は既存の `crate::biz::inventory_service::validate_business_date`（10 バイトの ASCII の形を先に見てから chrono の暦日。`2026-1-1` を拒む。先例 `sales_service.rs`）で確かめ、その error を §37.11 の文「日付の形が正しくありません」に替える。翌日は `chrono::NaiveDate` で求める（§37.11 手順 1 (iii) と同じ crate）。`gaps` は今の `list_ej_number_gaps`。新しい file を作らない（37 の design_compliance の行は `::ej` を持つ）。
- S3 CMD（`src-tauri/src/cmd/daily_report_import_cmd.rs`、45 §45.6c）: 3 本を `#[tauri::command(async)]`（関数は sync のまま。§45.4 の「実行の thread」と同じ形）で足す。取込みの `import_ej_files` が DB の Mutex を持つ間に同じ画面の閲覧が main thread で Mutex を待つと、× と Alt+F4 の確かめの窓が止まるため（UI_TECH_STACK UI-USW-D5 の 8）。§45.6c・§45.4 の「CMD-12 のほかの command」の文・UI-USW-D5 の 8 にこの理由を書く。閲覧が重なって async の worker が尽きると、「閉じる」は取込みが終わるまで遅れる（UI-USW-D5 の 8 の末尾と同じ）。`BizError` は §45.7 の `From<BizError>` で変える（`NotFound`・`ValidationFailed` の kind は今の表のまま）。`lib.rs` の `collect_commands!`・`generate_handler!` に登録し、`src/lib/bindings.ts` を再生成する。
- S4 route と navigation: `src/routes/reports/receipts.tsx`（`src/routes/reports/daily.tsx` と同じ `validateSearch` + zod の形）、`src/routeTree.gen.ts` の再生成、`src/config/navigation.ts` の「毎日の業務」の `ui-09b` の後に `ui-09c`「レシートの控え」（icon は lucide の既存の import の中から選び、新しい依存を足さない）と `navigation.test.ts` の到達の test。search の schema（feature の `types.ts`、route は import する）の `date` は `z.iso.date()`（入っている zod 4.6.7。形と暦日の両方。`2026-02-30`・`2026-13-01`・`2025-02-29` を捨て `2024-02-29` を通す）の `.optional().catch(undefined)`。`no` は ASCII の数字 1〜6 桁の文字列だけを受ける（`z.string().regex(/^[0-9]{1,6}$/).optional().catch(undefined)`）。Router が数に読んだ値（`?no=12`・`?no=1e2`・`?no=100000`）は捨てて既定（最新の日）。アプリに URL を打つ欄は無く、番号の URL を作るのはアプリの中の link（S9）だけで、`record_no` は 6 桁の文字列なので、先頭が 0 なら生のまま・そうでなければ引用符つきで渡り、どちらも文字列のまま届く（`?no=000123`・`?no=%22100000%22` は `"000123"`・`"100000"`、`?no=12`・`?no=1e2`・`?no=100000` は undefined。実測）。
- S5 画面（新規 `src/features/ej-receipts/`）: `EjReceiptsPage.tsx`（`PageShell`）、query の hook 3 つ（日の一覧・番号で探す・控えの文。key は S6）、表は `src/components/ui/table.tsx`、Alert は `src/components/ui/alert.tsx`、日の移動は `DateNavigator`（`src/features/daily-sales/components/DateNavigator.tsx`）の形（`Button` 2 つ + `<input type="date">` + `formatJpDate`）を写した feature の中の部品（隣の日は応答の `prev_date`・`next_date` に飛び、無ければ disabled。UI-09a の ±1 日の部品は変えない）。種類の label（kind と mode から。返品はモードが戻の取引）と金額（`1,234円`・「—」・「0円」）は feature の `lib/` の純関数。控えの文は `<pre>`（`white-space: pre`、折り返さず横に scroll、行末の空白を保つ）で、字形は Windows の和文の等幅を先頭にした font の指定を feature の中の 1 か所に置く（候補 `"BIZ UDGothic", "MS Gothic", monospace`。決めるのは L3-1。`font-mono` は和文の等幅を持たない）。表の行は上下と Enter で選べる（UI-09c の利用者配慮）。番号の入力は IME の変換中の Enter を送らず（`isComposing`）、既存の `src/components/patterns/normalizeComposedDigits.ts` の `normalizeComposedDigits` で全角の数字を半角にしてから送る（IME をオンのまま打った番号で「形が正しくない」と返さない。D-127。複製しない、51 UI-01b-D16。BIZ の形の検査は今どおり ASCII だけを受ける。80 に書く）。選んだ控えの本文は ID で `get_ej_record` から読む（番号で見つけた控えが一覧の 1,000 件の外でも出る）。手で選ぶ EJ の結果（S7 の hook の `result`）は、toast は `inserted(result, "選んだファイルから")` と、Alert の下の `EjNotes`（読めない file の名・食い違いの番号。`SdBulkImport.tsx` の `EjNotes` と `inserted` を export して共有し、写しを作らない）で出す。読込みの失敗は画面に原因と再試行（共有の error の形。`src/features/daily-sales/DailySalesPage.tsx` の失敗の出し方と同じ）。
- S6 query の key と D-052: `src/lib/query-keys.ts` に `ejRecords.root()`（`["ej-records"]`）と下の 3 つの子の key。`src/lib/invalidation-contract.ts` に 2 つの entry を足す: C26 `sdBulkImport`（一括取込みの成功。C10 `dailyReportImport` の 3 key + `ejRecords.root()`）と C27 `ejImport`（手で選ぶ EJ の成功。`ejRecords.root()` だけ）。C10 は日報の個別の取込みと取消で `ej_records` を書かないので変えない（過剰を足さない）。`useDailyReportImportFlow.ts:203` の一括の成功を C26 に替える。独立の oracle `src/test/invalidation-oracle.ts`・`invalidation-contract.meta.test.ts`（2026-11-24 の `1edcedc3` で 25 entry → 27。meta test の `toHaveLength` は 26 → 28〈`returnExchange` を 2 回数える〉）。C26・C27 の今の契約は `docs/UI_TECH_STACK.md` §2.5 の D-052 の段に置き（entry と handler の数を実測で書き直す）、§45.6c の手順 4 と `import_ej_files` の段・55 UI-07-D17 の生きた旧い文（「成功の後は日報の取込みと同じ invalidation（`invalidationContract.dailyReportImport`）」等）は §2.5 の C26・C27 を指す文に置き換える。decision-log の D-052 は変えない（冒頭の「Status の書き方」）。
- S7 手で選ぶ EJ の共有: `useDailyReportImportFlow.ts` の `ejImportMutation`・`chooseEjFiles`・`ejPending`・`ejSelectionError` と、その待ちの止め（`useBlocker` と `useWindowCloseGuard("ej-import")`）を、新規 `src/features/daily-report-import/hooks/useImportEjFiles.ts` に移し、日報の flow と S5 の画面の両方が使う（同じ file dialog・20MB の検査・読込みの失敗の文・待ちの解除を 2 つに書かない。D-147）。成功で C27 を当てる。hook は `reset()` を返し、日報の flow の `scanSd`・`bulkImport` が今の `setEjResult(null)`・`setEjSelectionError(null)`・`ejImportMutation.reset()` の代わりに呼ぶ。成功の toast は控えの画面の側で出し（S5）、日報の画面の動きは変えない。flow の `busy` は hook の待ちを今どおり含める（タブの切替えの止め、55 §55.7）。新しい hook は `useBlocker` を直に呼ぶので、`src/hooks/unsaved-changes-guard-sweep.test.ts` の許可の列と 55 §55.7・UI_TECH_STACK UI-USW-D1 の例外の名指しに足す。同じ test の T17 のため、`EjReceiptsPage.tsx` を `EXCLUDED_PAGES` に足す（未保存の編集を持たず、取込み中の止めは共有の hook が持つ）。
- S8 close の文と UI-07-D17 の入口の見直し（PR #294 の計画の closure round 2 の裁定 3）: `ej-import` は 2 つの画面から出るので、`src/components/layout/WindowCloseDialog.tsx` の `ej-import` の説明と UI_TECH_STACK UI-USW-D5 の 4 の表の行を、どちらの画面でも当たる文にする。文: 「取込みが終わるまでお待ちください。終わると閉じられます。いま閉じると、取り込めたかどうかをこの画面で確かめられなくなります。閉じたときは、次にアプリを開いてから「レシートの控え」の「EJ のファイルを選んで取り込む」で同じファイルをもう一度選んでください。取り込み済みの控えは二重に入りません。」Alert の有無を条件にしない（最大の番号の後ろの控えが入っていないことは `list_ej_number_gaps` の欠けに出ない）。UI-USW-D5 の 4 の表の下の段落の `ej-import` の文（「結果の文が残っていれば同じ file を選び直す」）も同じ筋に直す。UI-07-D17 の「EJ のファイルを選んで取り込む」の出す条件（`gaps` か `unreadable_files` が空でない）は変えず、欠けだけが残る日（一括が押せない）の入口は UI-09c-D5 が持つ（UI-07-D17 の「(A-1) には控えの画面の入口が無く」の文を、UI-09c-D5 を指す文にする）。`sd-bulk-import` の文は変えない（round 1 で確かめた）。
- S9 UI-07-D17 の link（`src/features/daily-report-import/components/SdBulkImport.tsx`）: 結果の下の「レシートの控えを見る」（`/reports/receipts`）と、`saved` が true の食い違いの番号ごとの「前の控えは「レシートの控え」で見られます」とその番号の link（`/reports/receipts?no=<record_no>`。型つきの `Link search={{ no: record_no }}`〈文字列〉に限る。開いた画面が UI-09c-D3 の番号で探す流れで日に移り行を選ぶ）。`saved` が false の番号には出さない（今のまま）。
- S10 正本と登録: 新規 `docs/function-design/80-ui-ej-receipts.md`（UI-09c の画面の文書。SCREEN_DESIGN の UI-09c-D1〜D5 を引き、画面の状態〈読込み中・空・その日の記録なし・失敗と再試行・1000 件超〉、種類の label の表、番号で探す 3 つの結果、search と選んだ行の持ち方〈選んだ行は local の状態。`no` が来たら探し、見つかれば search を `{ date: <その日>, no: undefined }` に replace で置き換える〉、key と D-052 の C26・C27 を書く）と `docs/FUNCTION_DESIGN.md` の索引に 1 行。`src-tauri/tests/design_compliance_test.rs` の `SKIP_DOCS` に `80-ui-ej-receipts.md`。SCREEN_DESIGN の「レシートの控え画面」の見出しと画面の一覧の 12b の行・24 §14.25・37 §37.11・45 §45.6c・55 UI-07-D17・`docs/db-design/pos-tables.md`・`docs/architecture/ui-task-specs.md` の UI-09c の「proposed・未実装」「(A-2)、未実装」の印を外す（(B) の分の印は残す）。SCREEN_DESIGN にない「その日の記録が無い」日の文は 80 に足す（案「この日の控えはありません」）。SCREEN_DESIGN UI-09c-D1 の置き場の文（「「売上」の群に…`/reports/monthly` の後」）を「「毎日の業務」の末尾、`ui-09b` の後」に、UI-09c-D3 に「選んだ控えの本文は ID で `get_ej_record` から読み、一覧の 1,000 件の外でも出る」の 1 文を足し、UI-09c-D5 の欠けが無いときの非表示の条件からリンクを外す（S8・U9 の常に置く形に合わせる）。52 §52.3・§52.4 の navigation の表と、navigation の項目の数（2026-11-24 に `rg -c 'id: "ui-' src/config/navigation.ts` で 23。足した後 24）を実測で書き直す: `navigation.ts` の comment（今「22 項目」）、52 の 30 行目と §52.4、`docs/design-system/01-decision-rules.md` の「22 項目」。REQ は開発拡張の次の番号（REQ-908 の見込み。merge の直前に `docs/spec/requirements.md` の最大を引き直す。DEV_WORKFLOW Review Rules の「連番の registry の採番は後続の lane が確定する」）を `docs/spec/requirements.md`・`docs/spec/requirements-coverage.md` に足し、test の ID に付け、`docs/function-design/90-traceability.md` を `cargo run --bin generate_traceability` で作り直す。
- S11 同じ PR の docs の片付け（TD-765 の B）: `Plans.md` の関門 19 の行を外す。`docs/backlog.md` の (A) の項目を済みの形（(A-1) は PR #294、(A-2) はこの PR）に書き換える。backlog の (A) を後の runtime として引く文（`:45`・`:102`・`:112`・`:201`）は、済みの後も正しいかを読み、古ければ直す。
- 新しい D: 要らない見込み（決定は D-121・D-052・SCREEN_DESIGN UI-09c にある）。計画の review で要ると決まれば D-169（予約済み）。migration は無い（`ej_records` と `idx_ej_records_printed_at` は v14 にある）。

並走の lane と重なる file の所有（DEV_WORKFLOW の Wave Operation。どちらが先に merge しても後の側が単段の同期の後に L3 をする。D-052 の C の番号と件数も後に merge する側が振り直す）:

| file | この lane が持つ行・節 | 他の lane |
|---|---|---|
| `src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts`・`invalidation-contract.meta.test.ts` | `sdBulkImport`・`ejImport` の 2 entry と meta test の名の列・件数 | PLU の runtime: C17・C19 の置換えと SD の読取りの成功の entry（件数は後の側が同期の後に実測で合わせる） |
| `docs/UI_TECH_STACK.md` | §2.5 の D-052 の段の C26・C27 と数、UI-USW-D1 の例外の名指し、UI-USW-D5 の 4 の表の `ej-import` の行と表の下の段落の `ej-import` の文、8 の閲覧の 3 command | PLU の runtime: §2.5 の C17・C19。E-1: L3 の smoke の節（重ならない節） |
| `src/lib/bindings.ts` | 3 command と閲覧の型 | PLU の runtime（生成物。手で解かず、取り込んだ後に再生成） |
| `src-tauri/src/lib.rs` | `collect_commands!`・`generate_handler!` の 3 行 | PLU の runtime: `import_plu_register_snapshot` の撤去 |
| `src-tauri/src/cmd/daily_report_import_cmd.rs` | 3 command の関数と test | PLU の runtime: scan の wire（§45.6a、`plu_snapshot_skipped_older`） |
| `docs/function-design/45-cmd-daily-report-import.md` | §45.6c の閲覧の 3 command と手順 4 の key、§45.4 の「ほかの command」の文 | PLU の runtime: §45.6a 手順 2a |
| `docs/function-design/55-ui-csv-import.md` | UI-07-D17 と §55.7 の hook の名指し | PLU の runtime: UI-07-D15 (8) |
| `src-tauri/src/biz/daily_report_import_service/` | `ej.rs` の閲覧の 3 関数と `tests.rs` の閲覧の test | PLU の runtime: `scan.rs` の §37.9 手順 10 |
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.11 の閲覧の段 | PLU の runtime: §37.9 手順 10 |
| `docs/function-design/90-traceability.md`・`docs/spec/requirements.md` | REQ の行と再生成 | 全 lane（生成物は再生成。REQ の番号は後の側が引き直す） |
| `Plans.md`・`docs/backlog.md` | 関門 19 の行、(A) の項目 | 棚卸し: 関門 17。PLU: PLU の項目。E-1: E-1 の項目。Z004: 店内コードの項目 |

Non-scope: `ej_records` の schema と migration、EJ の取込みの規則（BIZ-08-D9 の手順 1〜5）、IO-08・IO-09、`DateNavigator` と `TabsHeader`、`seed_demo.rs`、`l3_fixtures.rs`（(A-1) の生成の folder をそのまま使う）。

## Acceptance Criteria

- AC1: Test Plan の各行の test が `cd src-tauri && cargo test` と `npm run test` で PASS し、Linux の hosted の `Merge gate` が通る。
- AC2: Test Plan の mutation（M1〜M8）を 1 つずつ入れ、名指しの test が red（`cargo test <name>` の `FAILED`、`npx vitest run <file>` の `FAIL`）になり、戻すと PASS することを PR の comment に残す。
- AC3: `bash scripts/doc-consistency-check.sh`（Command registry drift を含む）と `--target plan` が ERROR なし、`cd src-tauri && cargo run --bin generate_traceability -- --check` が ERROR 0、`bash scripts/check-workflow-git.sh` が OK、`git diff origin/main --stat -- tests/fixtures` が空。
- AC4: UI-09c と閲覧の 3 command・5 つの repo の関数・3 つの BIZ の関数の未実装の印が残らない: `rg -nF -e '(A-2)、未実装' -e '(A-2) が足す（未実装）' -e 'UI-09c、proposed' -e '(A-2) で、未実装' -e '(A-2) で、どちらも未実装' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` の出力が各文書の更新履歴の表の行（`| 20` で始まる行）だけ。(B) の未実装の印は残る。
- AC5: `## L3` の項目の PASS（owner）。Writer は L3 の build の前に `cargo check --release`。

## Design Readiness

- 引用する設計正本（節まで）: SCREEN_DESIGN の「レシートの控え画面」（UI-09c-D1〜D5）、24 §14.25、37 §37.11（閲覧の段・手順 5・BIZ-08-D9）、45 §45.6c（手順 4 と `import_ej_files` の段）・§45.7、55 UI-07-D17・§55.7、UI_TECH_STACK §2.5（D-052）・UI-USW-D1・UI-USW-D5、decision-log D-052・D-121、pos-tables の「SD の一括取込みと EJ の控えの契約」。
- 必要な設計成果物: UI-09c の function-design の文書は無い → `80-ui-ej-receipts.md` を updated in this PR（SCREEN_DESIGN が「runtime の lane が作る」と決めている）。ほかは existing sufficient で、印を外す・C26/C27 と close の文を足すのが updated in this PR。
- plan にしかない durable な判断の昇格先: C26・C27 の分け方（C10 を変えない理由）は UI_TECH_STACK §2.5 へ。`ej-import` の文と UI-07-D17 の欠けだけの日の入口は UI-USW-D5 の 4 の表と 55 UI-07-D17 へ。その日の記録が無い日の文・選んだ行の持ち方・font の指定は 80 へ。
- 前提・制約と延期した design gap: 字形と大きさは SCREEN_DESIGN UI-09c-D4 が L3 で決めると定めている（L3-1 の結果を 80 に書く）。`find_ej_records_by_no` の 2 件以上は今のレジ 1 台の店では起きない（UI-09c-D3）ので、automated test だけで確かめる。番号の一周（999999 → 000000）は D-121・BIZ-08-D9 の Revisit のまま。
- 絶対保証の例外と抜け道: 「控えの文を記録のまま出す」は保存した `body_text` を変えずに出すことに依る（37 §37.11 の `EjRecordDetail.body_text`）。表示の側で trim・折り返し・tab の展開をしない。前の日・次の日は `printed_at`（レジの時計）で決め、レジの時計が戻った日は番号の順と日の順が食い違いうる（BIZ-08-D9 の「日付は `printed_at` で引き、アプリの時計と比べない」）。
- 判定: ready

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| 24 §14.25 閲覧の 5 関数 | 24 §14.25 | S1 | R1〜R5 | 非対象（repo） |
| BIZ-08-D9 閲覧 | 37 §37.11 処理ステップ（閲覧） | S2 | B1〜B4 | L3-2 |
| CMD-12-D2 閲覧の 3 command | 45 §45.6c・§45.7・§45.8 | S3 | K1・K2、AC3 の Command registry drift | 非対象（wire） |
| UI-09c-D1 route・search・navigation | SCREEN_DESIGN UI-09c-D1、52 §52.3・§52.4 | S4 | U1・U2 | L3-1（サイドバーから開く） |
| UI-09c-D2 日付で探す | SCREEN_DESIGN UI-09c-D2、80 | S5 | U3〜U5 | L3-3（前の日） |
| UI-09c-D3 番号で探す | SCREEN_DESIGN UI-09c-D3、80 | S5 | U6・U7 | L3-2 |
| UI-09c-D4 控えの文 | SCREEN_DESIGN UI-09c-D4、80 | S5 | U8 | L3-1 |
| UI-09c-D5 欠けと手で選ぶ EJ | SCREEN_DESIGN UI-09c-D5、45 §45.6c | S5・S7 | U9・U10 | L3-3 |
| 空・失敗・1000 件超 | SCREEN_DESIGN UI-09c の空の状態、80 | S5 | U11 | 非対象（自動） |
| D-052 C26・C27 | UI_TECH_STACK §2.5 | S6 | I1・I2 | 非対象（自動） |
| UI-USW-D1・D5 `ej-import` | UI_TECH_STACK UI-USW-D1・D5、55 §55.7 | S7・S8 | G1・G2 | L3-3 |
| UI-07-D17 の (A-2) の分 | 55 UI-07-D17 | S8・S9 | L1・L2 | 非対象（自動） |
| REQ（開発拡張） | `docs/spec/requirements.md` | S10 | AC3 の traceability | 非対象（docs） |
| BIZ-08-D9 取込み・D-121 の取込みの規則 | 37 §37.11 手順 1〜5 | 変えない | (A-1) の E1〜E10 | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| R1 日の一覧 | 前の日・翌日の 00:00 の記録が入る／1001 件目を落とさない | Rust | `ej_repo` の test: `D 23:59` は入り翌日 `00:00` は入らない、1001 件を入れて 1001 件返る | 範囲を `<=`・`LIMIT 1000` にする（M1） |
| R2 隣の日と最新 | 記録の無い日に飛ぶ・端で値を返す | Rust | 3 日に記録、間に空の日 → 前後が記録のある日、端は None、空の表で最新は None。その日の `00:00` の記録は前の日にしない | 前後を ±1 日で求める・前の日の `<` を `<=` にする |
| R3 番号で探す | 別の 4 桁の欄の同じ番号を落とす | Rust | 2 つの欄に同じ番号 → 2 件、`(date, id)` の昇順 | `machine_prefix` で絞る |
| R4 控えの文（negative） | 無い id で error、行末の空白を削る | Rust | 無い id → `None`、行末の空白と `"\n"` のつなぎを含む文が完全一致 | `trim_end` を入れる |
| R5 mode・kind を読む（negative） | 外れた TEXT を既定の値にする | Rust（mutation） | 生 SQL で CHECK の外の値を入れられないので、match の関数に直接外れた文字列 → `DbError`。enum の全 variant が `as_str` と往復する | match に `_ =>` の既定を足す（M2） |
| B1 日の既定と形（negative） | `None` で最新の日に行かない、`2026-02-30`・`2026-1-1` を受ける | Rust | `None` → 最新の日、記録 0 件 → `date: None`・空、`2026-02-30`・`2026-1-1`・`""` → `ValidationFailed("日付の形が正しくありません")` | 暦日を見ず形だけを見る（M3） |
| B2 1000 件 | 超えたのに `truncated` が false・1001 件返す | Rust | 1001 件 → 1000 件と `truncated = true`、1000 件 → false | `> 1000` を `>= 1000` にする |
| B3 番号の形（negative） | `1234567`・`12a`・空を受ける、0 埋めしない | Rust | 3 つ → `ValidationFailed("記録の番号は 6 桁までの数字で入れてください")`、`12` → `000012` で当たる、全角数字は拒む | 0 埋めを外す（M4） |
| B4 無い id と欠け | 無い id で空の detail、欠けを出さない | Rust | `get_ej_record(999)` → `NotFound`、000101・000104 の日 → `gaps` に 000102〜000103 | `gaps` を空のまま返す |
| K1 CMD の kind と thread | `NotFound`・`ValidationFailed` が internal になる、閲覧が main thread で動く | Rust | 3 command の kind（`not_found`・`validation`）と §45.7 の文。既存の `test_daily_report_cmd_req401_commit_and_rollback_run_off_main_thread` の名の列に 3 本を足し（`crate::cmd::assert_commands_run_off_main_thread`）、`assert_eq!(commands.len(), 4)` と comment を 7 に直す | `From<BizError>` を通さない、`(async)` を外す |
| K2 登録 | bindings と handler の片方だけ | CI | Command registry drift（AC3） | `generate_handler!` だけに足す |
| U1 search（negative） | 不正な値で error・別の日を出す | Vitest | 実の Router に URL を渡す: `date=2026-13-01`・`date=2026-02-30`・`date=abc`・`no=12`・`no=1e2`・`no=100000`（生の数）・`no=1234567`・`no=a` → 既定（最新の日、番号なし）。`no=000123` と、link で作った `no: "100000"` は受ける | schema に `.catch` が無い、`no` の regex が無い |
| U2 navigation | 到達できない・置き場が違う | Vitest | `ui-09c` が「毎日の業務」で `ui-09b` の次、`to: "/reports/receipts"`・active | 項目を足さない・順を変える |
| U3 初期・読込み中・成功 | 既定の日でない・読込み中に空の文 | Vitest | 読込み中は空の文を出さない、成功で最新の日の表（時刻・番号・種類・合計） | 読込み中に空の状態を出す |
| U4 隣の日へ（状態の遷移） | 記録の無い日に行く・端で押せる | Vitest | 前の日・次の日で `prev_date`・`next_date` へ移る、None なら disabled、日付の入力で移り、その日の記録が無ければ 80 の文。日付の欄は change では移らず、Enter か欄から出たときに 1 回だけ移り、空なら今の日へ戻す。打ちかけのまま mouse で前の日を押すと表示中の日から 1 回だけ移る。欄で移った後に前後のボタンで移ると、前に打った日を打ち直して Enter で移れる | ±1 日で移る・change のたびに移る・打ちかけの日と隣の日へ 2 回移る |
| U5 種類と金額 | 色だけ・返品を取引と出す | Vitest（純関数） | kind × mode の表の全行（返品は `sale` + `return`）、`null` → 「—」、0 → 「0円」、`1234` → 「1,234円」 | 返品の分岐を消す（M5） |
| U6 番号で探す 3 つ | 2 件で勝手に移る・0 件で黙る | Vitest | 1 件 → その日に移り行を選ぶ（本文は ID で `get_ej_record` から読み、一覧の 1,000 件の外の ID でも出る）、2 件 → 移らず「同じ番号の控えが 2 件あります」と日付の link（押すとその id の行）、0 件 → 「その番号の控えはありません」、形の誤り → BIZ の文。件数は今回の応答で判定（C27 で無効にした cache の 1 件で移らず、応答 2 件で候補、取り直しの失敗で文） | 2 件で先頭へ移る・cache の値で判定する |
| U7 IME と `?no=` | 変換の Enter で送る・link の番号で探さない | Vitest | `isComposing` の Enter は送らない、`１０２` は `102` で送る、`?no=000123` で開くと探して行を選び、search が `{ date: <その日> }` に replace で替わり、その後に次の日へ移れる。実の Router で戻ると `?no=` の前の画面 | `isComposing` を見ない（M6）・route の `replace` を落とす |
| U8 控えの文 | 行末の空白・並びが崩れる | Vitest | 行末の空白を含む文が `<pre>` の text と完全一致、`white-space: pre`、80 §80.4 の字形の順（独立の期待値）、選んだ行だけ `tabindex=0`。選んだ控えは表示中の日に結ぶ（search の日を外から替える・前の表の行を移る間に押す → 移った後に文を出さない） | 文を `trim` する・`<p>` で出す・選択を id だけで持つ |
| U9 欠けの Alert | 欠けを出さない・無いのに出す | Vitest | `gaps` 2 区間 → icon + 見出し + 「記録 n〜m の控えがありません。」2 行、`role` と文字で示す。空なら Alert なし、下の補助の link は常にある | Alert の条件を外す |
| U10 手で選ぶ EJ（状態の遷移） | 成功の後に Alert が古いまま・待ちに離れられる | Vitest | 選んで成功 → C27 で日の一覧を取り直し Alert が消える、追加ありと追加 0 件の両方で toast、読めない file と conflict を `EjNotes` で名指し、待つ間は `useBlocker` と close の集合に `ej-import`、日付の欄と前の日・次の日は押せず、応答の後に解ける（解けた後に打って Enter で移る）。失敗 → error を出し再試行できる（日の一覧が失敗の間もリンクの近くに出す）。hook の test で 20MB 超は `importEjFiles` を呼ばず文を出す | 成功で invalidate しない（M7）・一覧が成功のときだけ結果を描く |
| U11 空・失敗・1000 件 | 空の文が無い・失敗で黙る | Vitest | 0 件 → SCREEN_DESIGN の文と「売上データ取込みへ」、空の DB でも下の「EJ のファイルを選んで取り込む」が出る、失敗 → 原因と再試行で取り直す、`truncated` → 「この日の控えが多いため、先頭 1000 件だけを出しています」 | 失敗を空として出す |
| I1 D-052 の独立 oracle | entry の欠け・過剰 | Vitest | meta test の 27 entry と oracle の完全一致、C26 = C10 + `ejRecords.root()`、C27 = `ejRecords.root()` | C10 に `ejRecords` を足す・C26 を C10 のままにする（M8） |
| I2 handler の呼出し | 一括の成功が C10 のまま | Vitest | 一括の成功で C26（既存の `DailyReportImportPage.sd.test.tsx:570` あたりの一括の期待を C26 の独立の期待に更新する）、手で選ぶ EJ の成功で C27、日報の個別の取込みと取消は C10 のまま | `useDailyReportImportFlow.ts` の一括を戻す |
| G1 共有の hook の止め | 日報のタブの止めが外れる | Vitest | 既存の `useDailyReportImportFlow.test.tsx` の手で選ぶ EJ の待ちの test が全部 PASS（期待を変えない）、sweep test の許可の列に新しい hook、T17 の `EXCLUDED_PAGES` に `EjReceiptsPage.tsx`、日報の画面に成功の toast が出ない | hook の `busy` を flow から外す |
| G2 close の文 | 片方の画面で当たらない・完了を言い切る | Vitest | `WindowCloseDialog.test.tsx` の `ej-import` が S8 の文（Alert の有無を条件にしない）、「日報取込み」タブと「取り込めています」を含まない | 旧い文に戻す |
| L1 一括の結果の link | 「レシートの控えを見る」が無い | Vitest | `SdBulkImport.test.tsx`: 結果の下に link（`/reports/receipts`）、押すと控えの画面が開く | link を消す |
| L2 食い違いの link | `saved: false` に link を出す | Vitest | `saved: true` の番号だけに `?no=<番号>` の link と文、false は今の文だけ | 条件を外す |

mutation（実装の run で 1 つずつ注入して red を確かめ、戻す）: M1 R1 の `LIMIT 1001` を 1000 → R1。M2 R5 の match に既定の枝 → R5。M3 B1 の暦日の検査を外し形だけにする → B1。M4 B3 の 0 埋めを外す → B3。M5 返品の label の分岐を消す → U5。M6 `isComposing` を見ない → U7。M7 手で選ぶ EJ の成功の invalidation を外す → U10・I2。M8 本番の側の C26 の定義から `ejRecords.root()` だけを外す → I1・I2（呼出し先の取り違えは I2 が拾う）。

## L3

場は開発用の Windows の native build、空の DB。入力物は (A-1) の L3 の生成（`src-tauri/src/io/l3_fixtures.rs` の `write_sd_bulk_l3_fixtures`、`INVENTORY_L3_DAY` は L3 の日の前日以前）の `sd-a` と `ej-manual`（CP932・CRLF、合成）。準備は「売上データ取込み」→ 日報取込みタブ →「場所を選ぶ」で `sd-a` →「まとめて取り込む」（T−1 の 000101〜000103 と T の 000106〜000107 が入り、000104〜000105 が欠け）。

- L3-1 レシートの控え / サイドバーの「レシートの控え」→ T の日の 000106 の行を選ぶ / 控えの文の半角カナの品名と金額の行・全角の「合  計」等の行の右端の桁がそろい、行が折り返さず、紙のレシートと同じ 24 桁の並びに見え、字の大きさが読める（字形の候補で並びが崩れれば別の候補で取り直す）。決めた字形が実際に当たったこと（候補の先頭が無い PC では代わりの字形）を 80 に書く。
- L3-2 レシートの控え / 番号の欄に IME をオンのまま「102」と打ち、変換の確定の Enter と探す Enter を押す / 変換の確定の Enter では探さず、探す Enter で T−1 に移って 000102 の行が選ばれる（全角で入っても同じ）。
- L3-3 レシートの控え / T の日から「前の日」で T−1 へ移り、Alert を見て「EJ のファイルを選んで取り込む」で `ej-manual` の file を選ぶ / 選ぶ前は Alert「控えが足りない所があります」と「記録 000104〜000105 の控えがありません。」が出て、取り込んだ後は Alert が消え 000104・000105 の行が T−1 に出る。
