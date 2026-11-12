# Plan Packet: 日報の取込みを、完了の画面を離れた後も「日報の取込みの記録」から取り消せるようにする

wave に属さない単独の lane。導入の前の関門 12 の前提 A（owner の決定 2026-11-02、repo 外の回答台帳 TD-578 の O3 = a。PR #229 の packet `docs/plans/2026-11-02-daily-monthly-acceptance.md` の「owner の答え」の O3 に記録）。出発点は `docs/backlog.md` の項目「日報取込み標準手順の残設計」のうち「日報取込みの履歴一覧と、完了画面を離れた後の取消の導線（command `list_daily_report_imports` は登録済みで UI 未使用）」と、BIZ-08-D4 の文と SD の一覧の「同じ精算を取込み済み」の行がこの導線を前提にするという記述。本 commit は Plan Packet の起票と plan-first の設計正本の修正（下の Scope の「本 commit で変えた正本」。どれも proposed・未実装の注記つき）だけで、実装は計画の固定の後の別の run。先例は Z004 の取込みの記録の詳細から取り消す入口（PR #197、2026-10-28 merge、`docs/plans/2026-10-23-z004-rollback-entry.md`、55 UI-07-D18）で、その形（タブの上の導線・共有の確認・共有の取消の mutation と止め）を日報へ移す（D-147: 既にある pattern を先に使う）。

## Workflow State

- Risk: R3
- Plan Commit: b5136eb28ba47fda49ba91f5ea15f286b5b64fb5
- Amendments: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「Tauri command の DTO・route と search の状態・状態を持つ画面・operator の業務の流れ」に当たる: command `list_daily_report_imports` の出力の型を置き換えて bindings を作り直し、新しい route（`/csv-import/daily-reports`、search `page`）と、取消の応答を待つ間に画面の離脱を止める状態を持つ画面を足し、日報の取消の業務の流れの入口を 1 つから 2 つにする。R4 にしない理由: 取消そのもの（37 §37.5、`status='rolled_back'` の論理の取消で行を消さず、同じ束を取り込み直せる）は変えず、migration・削除・backup・restore・店の実データに触れない。

## Goal

Goal Invariant:

### 最小完了条件

- 日報を取り込んだ後に誤りに気づいたとき、完了の画面を離れた後でも、`売上データ取込み` の `日報取込み` タブの上の導線から「日報の取込みの記録」の画面を開き、その取込みの行の `取り消す` で、完了の画面と同じ確認を経て取り消せる（owner の受入の観点、TD-578 の O3）。
- 取り消すと、その日の日次売上・月次売上の公式の値が取込みの前へ戻り（取消済みの取込みは集計に入らない）、正しい束を「SD から読む」で取り込み直すと正しい値になる（同上）。
- BIZ-08-D4 の拒否の文（「前の取込みを取り消してから読み直してください」）が案内する操作（前の取込みを取り消す）が、拒否の出た同じタブの上の導線から使い手にできる（同上）。
- 実装の merge の commit で、受入の手順書 `docs/daily-monthly-acceptance-runbook.md` の「日: 修正・再取込み後の表示」の行の「前提の lane の後に足す」の部分を、この lane の画面の名で書き足す（同上、56 UI-09a-D13・backlog の A の項目の義務）。

### 失敗定義

- 完了の画面を離れた後に、アプリの中で日報の取込みを取り消す操作が無い、または取消済みの取込みに `取り消す` が出る。
- 一覧の行の `取り消す` が、その行と別の取込みを取り消す（日付から ID を推し量る、行と ID の取り違え。UI-07-D14）、または同じ日の他の取込みまで取り消す。
- 確認を経ずに取り消せる、または確認が取込みID・対象日・ファイル名・総売上・純売上のどれかを示さない（UI-07-D14）。
- 取り消した後に、日次売上・月次売上・一覧のどれかが古い値のまま残る（D-052 C10 の再取得が当たらない）。
- 取消の応答を待つ間に route の移動・リロード等の page の離脱で画面を離れられる（離れた後に失敗すると、知らせが別の画面の Toast だけになり同じ画面で再試行できない。55 §55.7、D-127）。取消が失敗したとき、同じ行の同じ取込みID で再試行できない。
- 一覧の読込みの失敗が空の一覧に見える（「記録はまだありません」と読ませる）。
- 取込みの状態を色だけで示す。
- 完了の画面の今の取消（取込みの直後の `取り消す`、取消の後の `idle`、タブの切替えの止め）の振舞いが変わる。

### 非目的

- 取消の本体（37 §37.5 の論理の取消・冪等・operation log）と、Z004 の取消（55 UI-07-D18、65 §65.5a）を変えること。
- 日報の取込みを入出庫履歴（65 の hub）に入れること（日報は在庫を動かさず、入出庫の記録でない。37 §37.5 の重要）。
- 取込みの記録の詳細の画面（Z001・Z002・Z005 の行を取込みごとに見る画面。55 冒頭の Z001 / Z002 / Z005 の表示の後続候補の案 C）。
- 一覧の対象日・状態による絞込み（O5。CMD の `date_from`・`date_to` は渡さず `null`）。
- 日次売上・月次売上の画面に取消の導線を置くこと（O1 の (b)）。
- BIZ-08-D4 の拒否の error に取込みID を構造で持たせること（Z004 の `related_csv_import_id` に当たるもの。O1 の理由）。
- 取込み済みを `XZ_BKUP` へ移す (b)（PR #228）、月次の画面の取り込んだ日（関門 12 の前提 B）、店舗マニュアルへの反映（backlog の同じ項目の残り）。
- native の window の close を止めること（UI_TECH_STACK UI-USW-D4 の別 backlog のまま）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 現状の調べ（2026-11-03、main `b5e8fa4c`）

- 取消を呼ぶのは完了の画面だけ: `rg -l 'commands.rollbackDailyReportImport' src --glob '!*.test.*' --glob '!src/lib/bindings.ts'` → `src/features/daily-report-import/hooks/useDailyReportImportFlow.ts`（1 file。`rollbackMutation`、成功で `dispatch({ type: "rollback_succeeded" })`・Toast `日報取込みを取り消しました`〈8 秒〉・`invalidationContract.dailyReportImport()`、失敗で Toast `取り消しに失敗しました。もう一度お試しください`）。取込みの流れの状態は同じ hook の `useReducer` で、`CsvImportPage` を離れると消える。確認の dialog は `DailyReportImportPage.tsx` の `DailyReportResultStep` の中の `AlertDialog`（title `日報取込みを取り消しますか？`、取込み ID・対象日・ファイル名・総売上・純売上）。
- 一覧の command は UI から呼ばれない: `rg -n "listDailyReportImports" src --glob '!src/lib/bindings.ts'` → 0 件（exit 1）。bindings は `listDailyReportImports(page, perPage, dateFrom, dateTo)` → `PaginatedResult<DailyReportImport>`。`DailyReportImport` は `src-tauri/src/db/sales_repo.rs` の repo の行の型（`bundle_hash`・`source_files_json`〈内部の保存の JSON の文字列〉・`source_adapter`・`note` を含む）で、BIZ は `pub type DailyReportImportRow = DailyReportImport;`（`biz/daily_report_import_service/mod.rs`）の別名で返す。`rg -c 'source_files_json' src/lib/bindings.ts` → 1。
- ファイル名の取り出しは BIZ にある: `biz/daily_report_import_service/parse.rs` の `fn source_filenames(source_files_json)`（各要素の `filename` だけを読み、読めなければ `BizError::ImportError("既存日報のファイル情報を読み取れません")`）。同じ日の追加の確認の `SameDateDailyReportImportSummary.source_filenames` がこれを使う。
- 一覧の query key と再取得は用意済み: `src/lib/query-keys.ts` の `dailyReportImports(page, perPage)`（`["daily-report-imports", "list", { page, perPage }]`）と `dailyReportImportLists()`。`invalidationContract.dailyReportImport()`（D-052 C10）は `dailyReportImportLists()`・`dailySalesRoot()`・`monthlySalesRoot()` を含み、`src/test/invalidation-oracle.ts` も同じ。取消と取込みの後に一覧・日次・月次が再取得される（集合を変えずに足りる）。
- 取消の後の集計: 日次・月次の公式の読取りは `status = 'completed'` の取込みだけを数える（`sales_repo.rs` の `WHERE i.report_date = ?1 AND i.status = 'completed'` 等）。SD の一覧の状態（`Imported`・`SameSettlementImported`）も `completed` の取込みだけで決まり（`find_*` の `status = 'completed'`）、取り消すと同じ束は `取り込めます` に戻る。
- 「同じ精算を取込み済み」の行（`DailyReportImportPage.tsx` の `CANDIDATE_STATUS.SameSettlementImported`）には `確認する` が無く（`candidate.status === "NotImported"` の行だけ）、直し方の文も無い。全件が `Imported`・`SameSettlementImported` なら一覧の上は `新しい精算はありません`（55 UI-07-D15 (3) の (c)）。
- 先例の形: `src/features/csv-import/hooks/useRollbackCsvImport.ts`（mutation・Toast・再取得・`useBlocker` を 1 か所に置き、`onSuccess` を呼ぶ側から受ける）、`CsvImportPage.tsx` の Z004 タブの上の `取り込んだ記録を見る（取り消すときはここから）`。
- `useBlocker` の直接使用は `src/hooks/unsaved-changes-guard-sweep.test.ts` の T16 が 4 file の完全一致で固定し（`useCsvImportFlow.ts`・`useRollbackCsvImport.ts`・`useDailyReportImportFlow.ts`・`useUnsavedChangesWarning.ts`）、T17 が `src/features/**/*Page.tsx` の全部を APPLIED / EXCLUDED のどちらかに分類させる（新しい Page は足さないと red）。例外の持ち主は 55 §55.7（UI_TECH_STACK UI-USW-D1）。
- command の登録: `bash scripts/check-command-drift.sh` → `D=76 H=76 S=76 T=76`。本 lane は command を足さない。
- L3 の合成の SD の tree: `biz/daily_report_import_service/tests.rs` の `#[ignore]` の `write_l3_register_sd_tree`（env `INVENTORY_L3_SD_TREE_OUT`、helper `sd_root_at`・`put_bundle`・`bundle(date, settlement_no, amount)`）が先例（PR #172 の L3）。

## owner の判断事項

正本で一意に決まらない製品の問い。owner の答え（2026-11-03、repo 外の回答台帳 TD-587）: O1〜O5 とも a。答えは 55 UI-07-D19 に書いた。下の推奨は起草役（AI）が出したもので、O5 の前提（誤りに気づくのは取り込んで間もない時が多い）は店の事実を確かめていない推測。下は問いと選択肢の記録（「変わる所」は答えが a でなかった場合の影響）。店主に新しく聞く事項は無い。

- **O1 入口の置き場**: (a) `日報取込み` タブの上に `取り込んだ日報の記録を見る（取り消すときはここから）` を置き、新しい画面「日報の取込みの記録」を開く【推奨】／(b) (a) に加え、日次売上の公式の節にその日の記録への導線を置く／(c) 日報取込みタブの中に一覧を直接置く。理由: D4 の拒否は日報取込みタブの preview・error に出るので、同じタブの上の導線は拒否の出た画面から届き（D-127）、Z004 タブと同じ形で覚える場所が 1 つで済む（04-backbone の同じ操作は同じ顔）。(b) は誤りに日次売上で気づく場面に近いが、レポートの画面に破壊的な操作の入口を足し、要件の外（D-147）。(c) はタブの flow の状態（`useReducer`）と同じ所に一覧を持ち、取込みの途中の状態と混ざる。D4 の error に取込みID を構造で持たせない理由も同じ（同じタブの上に導線があり、wire を変えずに済む。UI は message の文字列で分岐しない、UI-07-D15 (5)）。変わる所: (b) なら 56 に決定と導線、56 の test・L3 が増える。(c) なら UI-07-D19 の置き場と route が変わる。
- **O2 取消の操作の置き場**: (a) 一覧の行に `取り消す` を置き、完了の画面と同じ確認の dialog を経る【推奨】／(b) 行から取込みの記録の詳細の画面へ進み、そこで取り消す（Z004 の 65 §65.5a の形）。理由: 日報の取込みには詳細の画面が無く、確認の dialog が UI-07-D14 の確認の項目（ID・対象日・ファイル名・総売上・純売上）を全部示すので、詳細の画面を足しても同じ情報の 2 つ目の置き場になる（D-147）。65 §65.8.1 の「行内に取消/訂正ボタンは置かず」は入出庫履歴の hub の規則で、日報はその hub に入らない。変わる所: (b) なら新しい route・Page・その test が 1 組ずつ増え、L3 の到達が 1 段増える。
- **O3 SD の一覧の「同じ精算を取込み済み」の行**: (a) 行の下に `取り込み直すときは、前の取込みを取り消してから読み直してください。` と、記録の画面への link `日報の取込みの記録を開く` を置く【推奨】（closure の後に TD-592 でその行の `精算 {n}` を足した。下の項）／(b) 変えない（タブの上の導線だけ）。理由: この行は BIZ-08-D4 と同じ状況（同じ対象日・同じ精算回数で bytes の違う束）で、今は直し方を名指さない（backlog の同じ項目が「この取消の導線を前提にする」と書く。D-127）。状態の enum で出すので message の文字列に依らない。変わる所: (b) なら Scope の `DailyReportImportPage.tsx` の SD の一覧の変更、55 UI-07-D15 (3) の追記、U9、L3-2 の (i) を外す。
- **O4 一覧に出す取込み**: (a) 取り消した取込みも状態の label（`取込み済み`／`取消済み`、icon つき）で出す【推奨】／(b) 取り込んだままの取込みだけを出す。理由: 取り消せたことを同じ画面で確かめられ、取り消した後も行が消えないので、ページの中の位置がずれず別の行を押し違えない。37 §37.6 の既定も全状態。変わる所: (b) なら CMD-12 の `status` を公開する（45 §45.6 の「status filter は第1スライスでは公開しない」の改訂）か BIZ が `completed` を渡す必要があり、U1 の取消済みの行の case と L3-3・L3-5 の観測が変わる。
- **O5 探し方**: (a) 新しい順（対象日・取込み日時・ID の降順）のページ送りだけ（1 ページ 20 件）【推奨】／(b) 対象日で絞る入力を足す。理由: 誤りに気づくのは取り込んで間もない時が多く（店主の事実は未確認。推測）、新しい順の 1 ページ目に載る。絞込みは route の search・入力の検証・空の結果の文が増える。広げる条件（`ponytail:` comment に書く）: 1 ページ目に目当ての日が無いという所感が出たとき。変わる所: (b) なら search に `date`、CMD に `date_from`・`date_to` を渡し、U7 に絞込みの case が増える。
- **精算回数の列**（計画の review round 1 の P3。出所は AI の review）: owner の決定（2026-11-03、repo 外の回答台帳 TD-590）: (a) 足す。`DailyReportImportRow.settlement_no` と一覧の「精算回数」の列（`null` は `記録なし`）、repo の行の型と 4 つの SELECT に列（24 §14.2 の「足さない」を置き換えた）。理由: 同じ日に取り込んだままの取込みが 2 つ以上ある日（D-071、UI-07-D13）に、BIZ-08-D4 の「前の取込み」（同じ対象日・同じ精算回数）を行から見分け、別の精算を取り消して公式の値をいったん下げることを避ける。退けた (b): 足さずに残りの危険と L3 の見分けの観測で扱う。
- **SD の一覧の「同じ精算を取込み済み」の行の精算回数**（出所は計画の review の closure 775 の P3〈AI の review〉。列だけでは D4 の場面の画面の情報と結び付かず、記録の一覧のどの行が「前の取込み」かを使い手が決められなかった）: owner の決定（2026-11-03、repo 外の回答台帳 TD-592）: その行にも精算回数を出す。`DailyReportSdCandidate.settlement_no: Option<i64>`（37 §37.9）と、その行の状態の label に並べた「精算 {n}」（数の書き方は記録の一覧の列と同じ）。`null` は出さない（この状態では必ず数。型の上だけの場合）。ほかの状態の行には出さない（起草役の判断: TD-592 の目的は「前の取込み」を記録の一覧の同じ精算回数の行と結ぶことで、その状態の行だけが要る。毎日の行の見た目を変えない。Coordinator が覆せる）。

## 起票時に起草役が決めたこと

設計の値を変えない技術の判断。Coordinator が覆せる。1〜6 は本 commit で正本へ置いた（1 は 37 §37.2・§37.6 と 45 §45.6、2〜6 は 55 §55.12 と §55.1・§55.7）。7 は test の fn の comment だけ。

1. **一覧の wire の型を置き換える**: `biz/daily_report_import_service/mod.rs` の `pub type DailyReportImportRow = DailyReportImport;` を、`#[derive(Debug, Clone, serde::Serialize, specta::Type)] pub struct DailyReportImportRow { id, report_date, source_filenames: Vec<String>, settlement_no: Option<i64>, gross_amount, net_amount, status, imported_at, rolled_back_at }` に置き換える（名前は今の別名のままなので CMD-12 の `daily_report_import_cmd.rs` の signature は変わらない）。`settlement_no` は owner の決定（2026-11-03、repo 外の回答台帳 TD-590。出所は計画の review round 1 の P3〈AI の review〉）で、repo の `DailyReportImport` に `settlement_no: Option<i64>` を足し、`row_to_daily_report_import` を使う 4 つの SELECT（`find_daily_report_import_by_id`・`find_blocking_daily_report_by_bundle_hash`・`find_daily_report_imports_by_report_date`・`list_daily_report_imports`。`rg -n 'row_to_daily_report_import' src-tauri/src/db/sales_repo.rs` → 定義 1 と呼出し 4、2026-11-03）の列の末尾に `settlement_no` を足して写しの末尾で読む（24 §14.2。共有の写しが列の位置で読むので、1 つだけに足すと他の 3 つがずれる。#228 の同期の後に同じ rg で呼出しを数え直す）。`list.rs` が repo の `DailyReportImport` を受けて写し、`source_filenames` は `parse.rs` の `source_filenames` を共有に移して使う（同じ規則を 2 つ持たない。読めない JSON は今と同じ `ImportError` で一覧全体を失敗にし、空の名前にして黙って通さない）。`status` は今の文字列のまま（DB の CHECK `status IN ('completed','rolled_back')`、schema_v4）。棄却: (a) UI が `source_files_json` を JSON として読む（BIZ の内部の保存の形〈37 §37.4 手順 6 の `StoredSourceFile`〉に UI が縛られる）、(b) 今の型に `source_filenames` を足して `source_files_json`・`bundle_hash` 等を残す（使い道の無い内部の値を wire に残す。D-127 継ぎ足さない。今の consumer は 0）。repo の `DailyReportImport` が wire に出なくなれば `specta::Type` の derive を外す（bindings から `DailyReportImport` の型が消える）。
2. **取消の mutation を 1 つの hook にする**: `src/features/daily-report-import/hooks/useRollbackDailyReportImport.ts`（`useRollbackCsvImport` と同じ形）に、`commands.rollbackDailyReportImport`・成功の Toast（文と 8 秒は今のまま）・`invalidationContract.dailyReportImport()`・失敗の Toast・`useBlocker({ shouldBlockFn: () => mutation.isPending, enableBeforeUnload: () => mutation.isPending })` を置き、`onSuccess` を呼ぶ側から受ける。`useDailyReportImportFlow` は `onSuccess` で `rollback_succeeded` を dispatch するだけにし、取込み中（`importing`）の `useBlocker` は flow に残す。`isBlocking`（取込み中か取消の応答を待つ間。親のタブの切替えの止め、UI-07-D15 (6)）は今と同じ値を返す。
3. **確認の dialog を 1 つの部品にする**: `src/features/daily-report-import/components/DailyReportRollbackConfirmDialog.tsx`。中身（title・本文・dl の 5 項目・`キャンセル`／`取り消す`）は今の `DailyReportResultStep` の dialog の文字どおりに移し、完了の画面と一覧の行が使う。入力は `{ id, reportDate, filenames, grossAmount, netAmount }`（完了の画面は `DailyReportImportResult` と preview の file 名から、一覧は行から渡す）。
4. **route と search**: `src/routes/csv-import.daily-reports.tsx`（`/csv-import/daily-reports`、`/csv-import` の layout route の下。`/csv-import/records/$importId` と同じ家族）。search は `page`（1 以上の整数、欠落・不正は 1）だけで、Zod schema は feature の `src/features/daily-report-import/types.ts` が持ち route は import するだけ（UI-STATE-D2。先例 `csvImportSearchSchema`）。戻る導線は固定の `売上データ取込みへ戻る`（`/csv-import`。既定のタブが日報取込み）で、`returnTo` は持たない（入口が 1 つ。O1）。
5. **一覧の読取り**: `useQuery`、key は既存の `queryKeys.dailyReportImports(page, perPage)`（`dateFrom`・`dateTo` は `null`。O5）。1 ページ 20 件の定数に `ponytail:` comment（O5 の広げる条件）。読込み中、空（`日報の取込みの記録はまだありません。` は `total_count === 0` のときだけ）、範囲外のページ（`total_count` が 1 以上で行が 0 なら `このページには記録がありません。` と search `page` を 1 にする `1 ページ目を開く`。先例 `InventoryRecordsPage.tsx` は行 0 を一律に空の文にするので写さない）、失敗（Alert に CMD の message〈`describeError`〉と `再試行`。37 §37.6 の読めない `source_files_json` のように再試行で直らない失敗でも原因の文が読める）、ページ送り（`src/components/patterns/Pagination.tsx`）。
6. **一覧の行**: 列は 対象日・精算回数・取込みID・ファイル名（全部）・総売上・純売上・取込み日時・状態。精算回数は数をそのまま（桁区切りなし、先頭の 0 を付けない）、`null` は `記録なし`（TD-590。同じ日の取込みのうち D4 の「前の取込み」〈同じ対象日・同じ精算回数〉を行から見分ける）。状態は日本語の label と icon（`取込み済み`／`取消済み`。色だけで示さない）。`取り消す` は `status === "completed"` の行だけに出し、取消の応答を待つ間はどの行の `取り消す` も押せない（`取り消し中…` と spinner は押した行）。成功の後は C10 の再取得で行が `取消済み` に替わり、`取り消す` が消える。失敗は Toast で、行と ID はそのまま（同じ行で再試行）。画面の上の説明: `誤って取り込んだ日報の取込みを取り消せます。取り消しても在庫数は変わりません。取り込み直すときは、売上データ取込みの日報取込みで読み直してください。`
7. **L3 の合成の SD の tree**: `tests.rs` に `#[ignore]` の `write_l3_daily_report_rollback_sd_tree`（env `INVENTORY_L3_SD_TREE_OUT`、今の `write_l3_register_sd_tree` の helper を使う。既存の fn は他の lane の L3 が使いうるので変えない）。昨日の日付で束を 3 つ置く: 精算回数 `0101` の 2 つを bytes と金額を変えて（X: `bundle(昨日, "0101", "15000")` を suffix `A`、Y: `bundle(昨日, "0101", "12000")` を suffix `B`）、別の精算 Z: `bundle(昨日, "0102", "9000")` を suffix `C`。X を取り込むと Y が `同じ精算を取込み済み`（BIZ-08-D4 の状況）になり、Z と X を取り込んだ日は同じ日に取り込んだままの取込みが 2 つになる（精算回数の列で見分ける、TD-590）。

## Scope

本 commit で変えた正本（plan-first。新しい決定と節は proposed・未実装の注記つきで、実装の PR が注記を外して実装の記述に直す）:

- `docs/function-design/55-ui-csv-import.md`: §55.12（UI-07-D19）の新設（O1〜O5 の既定・理由・棄却案、画面・行の取消・共有の確認と hook・読込みの状態・O3 の行の案内・受入）。§55.0 の日報取込みの利用者フローの手順 6、UI-07-D15 (3) の「同じ精算を取込み済み」の行の案内、§55.1 の module の表の 4 行、§55.7 の進行中の処理のガードの hook の名指しと解除の行の日報の注記、§55.9 の rollback 失敗の UX、更新履歴（起票と計画の review round 1 の是正の 2 行）。round 1 の是正で §55.12 の一覧に精算回数の列と `null` の表示（TD-590）、読込みの状態（空の文の条件・範囲外のページ・失敗の Alert の message）を足した。
- `docs/function-design/37-biz-daily-report-import-service.md`: §37.2 の `DailyReportImportRow`（`settlement_no` を含む）と `DailyReportImport` の扱いの文・`settlement_no`、§37.6 の出力の型と「出力」の段（`settlement_no` の写し）、更新履歴。
- `docs/function-design/45-cmd-daily-report-import.md`: §45.3 の DTO の所有の文、§45.6 の出力の型と段（精算回数）、§45.6a の候補の `settlement_no`（TD-592）、§45.8 の再生成の文、更新履歴。
- closure round 2 の是正（TD-592）: 37 §37.9 の `DailyReportSdCandidate` の `settlement_no`、55 UI-07-D15 (3) と §55.12 の O3 の「精算 {n}」、37・45・55 の更新履歴の行。
- `docs/function-design/24-io-csv-import-repo.md`: §14.2 の `DailyReportImport` の構造体に `settlement_no` の行と、注記（wire に出さないこと、TD-590 で `settlement_no` を足し「足さない」を置き換えたこと、4 つの SELECT に列を足すこと）だけ（並走の B の lane が 24 §14.22a を編集中なので、更新履歴の行は実装の PR で足す）。
- `docs/SCREEN_DESIGN.md`: 売上データ取込み画面の節のレイアウト判断に 1 行（行を対象日と精算回数で見分けることを含む。並走の B の lane が月次の節を編集中なので、この節だけ）。
- `docs/architecture/ui-task-specs.md`: UI-07 の CMD 呼び出しの日報完了取消しの行と、一覧の行。

本 commit で変えない正本: 57・34（並走の B の lane）、56・`docs/Plans.md`・受入の手順書（#229 の後、下の「#229 への依存」）、`docs/backlog.md`（実装の PR で「この変更で不要になる記述」を直す）、UI_TECH_STACK（UI-USW-D1 は例外の hook を 55 §55.7 に委ねる言い方なので列挙を足さない）、decision-log（新しい D は無い。D-052 C10 の集合は変えない）。

実装（計画の固定の後）。順は S1（Rust）→ S2（生成物）→ S3（画面）→ S4（docs）。S4 の受入の手順書の行だけは PR #229 の merge の後（下の「#229 への依存」）。

- S1 BIZ（Rust）:
  - `src-tauri/src/biz/daily_report_import_service/mod.rs`: `DailyReportImportRow` の struct（起草役の判断 1）、`source_filenames` の置き場（`parse.rs` から移す。可視性は `pub(super)`）。
  - `src-tauri/src/biz/daily_report_import_service/mod.rs`（続き）: wire の `DailyReportSdCandidate` に `pub settlement_no: Option<i64>`（TD-592）。
  - `src-tauri/src/biz/daily_report_import_service/scan.rs`: 候補の wire に、内部の `Candidate.settlement_no` と同じ値を入れる（今は内部の `Candidate.settlement_no`〈`scan.rs` の struct `Candidate`〉だけが持ち、`parse_daily_report_bundle` の `parsed.settlement_no` から作る〈`read_group` の最後の `Ok(Candidate { settlement_no: parsed.settlement_no, … })`〉。手順 5 まで読めなかった束の `read_group` の中の `fallback` は `None`。2026-11-03 の `rg -c 'settlement_no' src-tauri/src/biz/daily_report_import_service/scan.rs` → 10 行で、置き場は struct の field・`scan_with_roots` の並べ替え〈手順 8〉・`fallback` の `None`・手順 7 の照合・`Ok(Candidate…)` の 5 か所。wire の `DailyReportSdCandidate { … }` の literal は `scan.rs` の 2 か所〈`fallback` と `Ok(Candidate…)`〉だけ）。値の出所を 2 つにしない（wire は内部の field と同じ式から作る）。状態の判定・並びは変えない。
  - `src-tauri/src/biz/daily_report_import_service/list.rs`: repo の行を `DailyReportImportRow` に写す。入力ガードは今のまま。
  - `src-tauri/src/biz/daily_report_import_service/parse.rs`: `source_filenames` の呼出しを共有の関数へ（振舞いは変えない）。
  - `src-tauri/src/biz/daily_report_import_service/tests.rs`: 既存の `test_daily_report_req401_list_validation_and_result` を新しい型に合わせ（assert を弱めない）、下の Test Plan の B1〜B5 と L3 の tree の fn を足す。
  - `src-tauri/src/db/sales_repo.rs`: `DailyReportImport` に `settlement_no: Option<i64>` を足し、`row_to_daily_report_import` を使う 4 つの SELECT の列の末尾に `settlement_no` を足して写しの末尾（位置 11）で読む（判断 1、24 §14.2）。`specta::Type` を外す（wire に出なくなったときだけ）。WHERE・ORDER BY は変えない。repo の既存の list の test（`list_daily_report_imports` の page・日付の範囲・入力ガード）は assert を弱めずに通す。
  - `src-tauri/src/cmd/daily_report_import_cmd.rs`: 変えない見込み（型の名前が同じ）。compile で import の調整が要れば最小で直す。
- S2 生成物: `src/lib/bindings.ts`（`cd src-tauri && cargo run --bin generate_bindings`。差分は `listDailyReportImports` の戻りの型名と `DailyReportImportRow` の型、`DailyReportImport` の型が消える、`DailyReportSdCandidate` に `settlement_no: number | null` が足される見込み）、`docs/function-design/90-traceability.md`（`cargo run --bin generate_traceability`）。手で直さない。
- S3 画面（TS）:
  - 新規 `src/features/daily-report-import/hooks/useRollbackDailyReportImport.ts`（判断 2）。
  - 新規 `src/features/daily-report-import/components/DailyReportRollbackConfirmDialog.tsx`（判断 3）。
  - 新規 `src/features/daily-report-import/DailyReportImportRecordsPage.tsx`（判断 5・6。PageShell・PageHeader の題 `日報の取込みの記録`）。
  - 新規 `src/routes/csv-import.daily-reports.tsx`（判断 4。`src/routeTree.gen.ts` は ignored の生成物で commit しない）。
  - `src/features/daily-report-import/types.ts`: search の tuple なしの Zod schema と型（判断 4）。
  - `src/features/daily-report-import/hooks/useDailyReportImportFlow.ts`: `rollbackMutation` の本体を共有の hook へ移す（判断 2）。
  - `src/features/daily-report-import/DailyReportImportPage.tsx`: `DailyReportResultStep` の dialog を共有の部品に置き換える。O3 = (a) の `SameSettlementImported` の行の案内と link と、状態の label に並べた `精算 {n}`（TD-592。`settlement_no` が数のときだけ、ほかの状態の行には出さない）。
  - `src/features/csv-import/CsvImportPage.tsx`: 日報取込みの `TabsContent` の上に `取り込んだ日報の記録を見る（取り消すときはここから）`（`/csv-import/daily-reports`。Z004 タブの link と同じ形）。
  - test: 新規 `src/features/daily-report-import/DailyReportImportRecordsPage.test.tsx`、新規 `src/features/daily-report-import/hooks/useRollbackDailyReportImport.test.tsx`、`DailyReportImportPage.test.tsx`（完了の画面の確認の既存の assert が共有の部品で通る）、`DailyReportImportPage.sd.test.tsx`（O3）、`hooks/useDailyReportImportFlow.test.tsx`（取消の成功で `idle`・失敗で `result` の既存の case が通る）、`src/features/csv-import/CsvImportPage.daily-tabs.test.tsx`（導線と、取消の応答を待つ間のタブの止めの既存の case）、`src/hooks/unsaved-changes-guard-sweep.test.ts`（T16 の期待の集合に `src/features/daily-report-import/hooks/useRollbackDailyReportImport.ts` を 1 行、T17 の EXCLUDED_PAGES に `DailyReportImportRecordsPage` を 1 行。完全一致は弱めない）。
  - `src/lib/query-keys.ts`・`src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts` は変えない（既存の key と C10 の集合で足りる。上の「現状の調べ」）。
- S4 docs（実装の PR）: 本 commit で変えた正本（上）の proposed・未実装の注記と「今の実装は…」の注記を外して実装の記述に直し（55・37・45・24・`docs/SCREEN_DESIGN.md`・`docs/architecture/ui-task-specs.md`）、24 の更新履歴に行を足し、`docs/backlog.md` の「この変更で不要になる記述」を直す。実装で古くなる次の 3 か所も同じ PR で直す（計画の review round 1 の P3）: (i) 55 §55.1 の module の表の `useDailyReportImportFlow.ts` の行（「`useMutation × 5`（…commit・rollback）+ useBlocker」と「commit/rollback 成功時は D-052-C10」）を、`useMutation × 4`（SD の scan・SD からの preview・手で選んだ preview・commit）+ `useRollbackDailyReportImport` + 取込み中だけの `useBlocker`、commit の成功で C10（取消の C10 は共有の hook）に、(ii) 55 UI-07-D15 (6) の「取消の応答を待つ間（`rollbackMutation.isPending`）」を共有の hook `useRollbackDailyReportImport` の pending に、(iii) `docs/diagrams/current-system.md` の route 対応表に「日報の取込みの記録 | `/csv-import/daily-reports` | 日報取込みタブの上の導線、SD の一覧の「同じ精算を取込み済み」の行の link」の 1 行を足し、表の後の「日報専用の詳細routeもなく、日報の取込み結果/履歴は取込み画面内で扱う」を「日報専用の詳細routeは無い。日報の取込みの履歴と取消は `/csv-import/daily-reports`、取込みの結果は取込み画面内で扱う」に置き換える。#229 の merge の後: `docs/daily-monthly-acceptance-runbook.md`・`docs/function-design/56-ui-daily-sales.md`・`docs/function-design/57-ui-monthly-sales.md` の UI-09b-D9 の 1 文（TD-667）・`docs/Plans.md`・`docs/backlog.md` の A の文（下の「#229 への依存」）。

Non-scope: `rollback.rs`・`commit.rs`・`scan.rs` の振舞い、`src/lib/invoke.ts`・`CmdError` の形、Z004 の画面と hook、`src/config/navigation.ts`（Sidebar に新しい項目を足さない。入口はタブの上の導線）、日次・月次の画面の code。

型・JSON・関数の形を変える所:

| 対象 | 作る所 | 読む所 | test の fixture / 呼び方 |
|---|---|---|---|
| `DailyReportImportRow`（wire） | `list.rs` | `DailyReportImportRecordsPage.tsx` | `tests.rs` の B1〜B4、記録の画面の test の `listDailyReportImports` の resolve 値 |
| `source_filenames`（共有の関数） | `mod.rs` | `list.rs`・`parse.rs` | B1・B2、既存の同じ日の追加の確認の test |
| `DailyReportImport.settlement_no`（repo の行、TD-590） | `sales_repo.rs` の `row_to_daily_report_import` と 4 つの SELECT | `list.rs`（ほかの 3 つの呼出し〈`rollback.rs`・`parse.rs`・`commit.rs` の照合〉は読まない） | B4、`sales_repo.rs` の既存の list・find の test |
| `DailyReportSdCandidate.settlement_no`（wire、TD-592） | `scan.rs` の候補の組立て（内部の `Candidate.settlement_no` と同じ値） | `DailyReportImportPage.tsx` の `SameSettlementImported` の行 | B5、`DailyReportImportPage.sd.test.tsx` の scan の resolve 値（既存の mock の候補にも field が要る。TS では必須の field になる見込みなので、候補を literal で書く test の mock に足す。2026-11-03 の `rg -c 'candidate_key:' src --glob '*.test.*'` → `DailyReportImportPage.sd.test.tsx` 1・`reducer.test.ts` 2。実装の時に同じ rg で全部拾い直す） |
| `useRollbackDailyReportImport({ onSuccess })` | 新しい hook | `useDailyReportImportFlow.ts`・`DailyReportImportRecordsPage.tsx` | hook の test、flow の test、記録の画面の test |
| `DailyReportRollbackConfirmDialog` の props | 新しい部品 | `DailyReportResultStep`・記録の画面の行 | `DailyReportImportPage.test.tsx`・記録の画面の test |
| `/csv-import/daily-reports` の search `page` | `types.ts` の schema | route・記録の画面 | 記録の画面の test（U7） |

他の lane との重なり（2026-11-03、`git diff --name-only origin/main...origin/<branch>`）:

- PR #228（`agent/xz-bkup-move`、取込み済みを `XZ_BKUP` へ移す (b)、実装の commit がある）が `DailyReportImportPage.tsx`（SD の一覧の文と片付けの失敗の文）・`daily_report_import_service/mod.rs`・`tests.rs`・`daily_report_import_cmd.rs`・`sales_repo.rs`・`src/lib/bindings.ts`・37・45・55・backlog を変える。本 lane が触る行は別（一覧の型・`source_filenames`・完了の画面の dialog・SD の一覧の `SameSettlementImported` の行の下・55 の新しい節）。後に merge する側が `origin/main` を単段で取り込み、bindings と traceability は取り込んだ後に再生成する（D-098）。#228 は SD の読取りの中で取り込んだ束の file を `XZ_BKUP` へ移すが、scan は `XZ` と `XZ_BKUP` の両方を読み（UI-07-D12）、取り消した取込みの束は移さない（#228 の AC2）ので、取り消した後に同じ束を SD から読み直せる前提は #228 の後も成り立つ見込み（#228 の merge の後の同期で U・L3 の前提を確かめ直す。未確認）。#228 は `scan.rs` と 37 §37.9 も変える（BIZ-08-D11 の片付け）ので、同期の後に候補の wire の literal（`rg -n 'DailyReportSdCandidate \{' src-tauri/src`）と `settlement_no` の置き場を数え直し、TD-592 の field を同じ式で足す。
- BIZ-08-D8 の runtime（backlog の「SD の一括取込みと EJ の控えの runtime（A）」、D-121、37 §37.10。日報の同じ日の追加の確認を精算回数の照合で置き換える）: 本 lane より先に merge されると、L3-1・L3-4 の「同じ日の追加の確認で `追加で取り込む`」が出なくなる（精算回数の違う Z と X、Z と Y は確認なしで入る）。同期の後の再確認の対象に (A) を含め、(A) が入っていれば L3-1・L3-4 の手順の文から追加の確認の段を外す（計画の review の closure 775 の Residual。起票の時点〈2026-11-03〉で (A) の open PR は無い: `gh pr list --state open` の目視）。
- PR #229（`agent/daily-monthly-acceptance`、docs だけ）: 下の「#229 への依存」。
- PR #227・#216・#194: 触る file は重ならない（`git diff --name-only` の目視）。

### #229 への依存

受入の手順書 `docs/daily-monthly-acceptance-runbook.md` と、56 UI-09a-D13・`docs/Plans.md` の関門 12・backlog の A の項目の「前提の lane」の文は、PR #229 にだけあり main に無い。実装のうちこれらの書き足しは、#229 が merge された後に本 lane の branch が `origin/main` を単段で取り込んで（`git -c merge.directoryRenames=false merge origin/main`）から、本 lane の merge の commit に入れる（#229 の packet の O3 と手順書の合格基準の表の義務）。#229 が本 lane より後になるなら、本 lane は書き足しの義務を持ったまま merge を待つ（Coordinator が順を決める）。ほかの Scope は #229 に依らない。

書き足す内容（#229 の版の文で書いた。同期の後の版で文が変わっていれば当て直す）:

- 手順書の手順 5 の下位に 1〜4 を足す: **完了の画面を離れた後に取り消す。** 手順 5 で取り込み直した日について、`日次売上` の画面へ移ってから `売上データ取込み` の画面へ戻り、`日報取込み` の tab の上の `取り込んだ日報の記録を見る（取り消すときはここから）` → `日報の取込みの記録` の画面でその日の `取込み済み` の行の `取り消す` → 確認の dialog の `取り消す` → `日報取込みを取り消しました` が出て、その行が `取消済み` になる → `日次売上` でその日に `この日付のレジ日報は未取込みです。` → `日報取込み` の tab で一覧を読み直し、その日の行が `取り込めます` に戻ったのを見て同じ束を取り込み直し、その日の 3 つの表を借りた手順 5 の日の紙と並べる（手順 3 と同じ比べ方）。取消の直後の月次は受入で見ない（owner の決定 TD-667。月次の戻りは取消の後の再取得の対象に月次を含む既存の自動 test〈D-052 C10、U3〉と本 lane の L3-3 が持ち、受入の月次は手順 6 の 4 の 1 回の紙との比べのまま）。BIZ-08-D4 の拒否の文が案内する操作（前の取込みを取り消す）はこの手順と同じ操作で、店の SD の写しで D4 を起こす手順は置かない（同じ精算で bytes の違う束は店の写しに通常無い。D4 そのものの表示は本 lane の自動 test と L3-2 が確かめた）。
- 合格基準の表の「日: 修正・再取込み後の表示」の行: 手順の列に足した手順の番号を足し、「前提の lane の後に足す（A の後に確かめる観点）: …A の lane が、自分の merge の commit でこの部分を自分の画面の名で書き足す」を、上の手順の画面と観測（`日報の取込みの記録` の `取り消す`、行の `取消済み`、日次の未取込みの note、取り込み直した値）で置き換える。「前提の lane の後に足す」の語はこの行から無くす（月の取込み済み日の範囲の行〈B〉には残る）。
- 判定の節: 規則（「前提の lane の後に足す」のままの行が残る版、または A・B の lane が merge される前の版では未了）は変えない（B が残る）。
- 56 UI-09a-D13 の受入の段: 「今の版で取り消せるのは取込みの完了の画面を離れる前だけで、離れた後の取消の入口…は無い。入口を作る R3 の lane を受入の前提にし…」を、離れた後は `日報の取込みの記録`（55 UI-07-D19）から取り消せることと、受入の手順書のその手順を引く文に置き換える（継ぎ足さない）。
- 57 UI-09b-D9 の修正後の値の段の最後の文（完了の画面を離れた後の取消で月次の値が取込みの前へ戻ることは、取消の入口の lane の後に受入の手順書の「前提の lane の後に足す」の行が確かめる）を、それは受入では見ず、既存の自動 test（取消の後の再取得の対象に月次を含む）と本 lane の L3-3 が持つ、と書く文に置き換える（owner の決定 TD-667）。57 の変更履歴に 1 行足す。
- `docs/Plans.md` の関門 12 の前提の文: A を前提から外し、B だけを残す（完了の行は足さない。D-145 (12)）。
- backlog の「日報取込み標準手順の残設計」の、#229 が足した A の文（受入の観点と書き足しの義務）を、済み（本 lane の PR）と書く 1 文に置き換える。「日報画面の Excel 印刷・バインダー代替受入判定」の「前提は…の取消の導線（A）と…（B）」から A を外す。

### この変更で不要になる記述・条件

- `useDailyReportImportFlow.ts` の `rollbackMutation` の本体（Toast・再取得・失敗の Toast）と、`blocking` の中の取消の pending の扱い（共有の hook の `useBlocker` へ移す。`isBlocking` の値は同じ）、comment「§55.6 の rollback の行「result 画面は維持」」。
- `DailyReportResultStep` の中の確認の `AlertDialog` の中身（共有の部品へ移す）。
- BIZ の `pub type DailyReportImportRow = DailyReportImport;` の別名と、wire の `DailyReportImport` の型（`source_files_json`・`bundle_hash`・`source_adapter`・`note` を wire に出していた）。repo の `DailyReportImport` の `specta::Type`（wire に出なくなれば）。
- 正本の文（本 commit で直した）: 45 §45.3 の `DailyReportImport` を wire の DTO とする文と §45.6 の戻りの型、24 の「`DailyReportImport`（上、`specta::Type` の行の DTO）には `settlement_no` を足さない（list の wire を変えない…）」（list の wire は BIZ の行の型になり、TD-590 で `settlement_no` を足す）。
- 正本の文（実装の PR で直す。S4）: 55 §55.1 の `useDailyReportImportFlow.ts` の行の `useMutation × 5`・rollback・「commit/rollback 成功時は D-052-C10」、55 UI-07-D15 (6) の `rollbackMutation.isPending`、`docs/diagrams/current-system.md` の「日報専用の詳細routeもなく、日報の取込み結果/履歴は取込み画面内で扱う」と route 対応表、55・37・45・24・SCREEN_DESIGN・ui-task-specs の proposed・未実装と「今の実装は…」の注記、backlog の「日報取込み標準手順の残設計」の履歴一覧と取消の導線の文と、BIZ-08-D4 の文と SD の一覧の行が「この取消の導線を前提にする」の文（済みにする）。#229 の後は上の「#229 への依存」の文。
- 確かめて不要でないもの: 完了の画面の `取り消す`（取込みの直後の最短の取消として残す）、取消の本体の冪等の Ok（2 つの入口の間で古い画面から押しても失敗にしない）、flow の取込み中の `useBlocker`、C10 の key の集合、`docs/plans/2026-10-17-sd-direct-read-runtime.md` の Residual Risk の記述（merge 済みの packet は書き換えない）。

## Acceptance Criteria

- AC1（一覧と SD の候補の wire）: B1〜B5 が PASS。`rg -n -A8 '^export type DailyReportSdCandidate = ' src/lib/bindings.ts` に `settlement_no: number | null` がある（baseline 無し、2026-11-03 の bindings.ts の 667〜673 行）。`rg -c 'source_files_json' src/lib/bindings.ts` が 0（baseline 1、2026-11-03 `b5e8fa4c`）。`rg -n '^export type DailyReportImportRow = ' src/lib/bindings.ts` が 1 件で、`source_filenames: string[]` と `settlement_no: number | null` を持つ。`sales_repo.rs` の `row_to_daily_report_import` を使う SELECT の全部が `settlement_no` を列に持つ（`rg -n 'row_to_daily_report_import' src-tauri/src/db/sales_repo.rs` の呼出しの数と、それらの SELECT の `rolled_back_at, note, settlement_no` の数が同じ。baseline は呼出し 4・列 0）。
- AC2（入口）: `rg -n "listDailyReportImports" src --glob '!src/lib/bindings.ts' --glob '!*.test.*'` が `DailyReportImportRecordsPage.tsx` に 1 件以上（baseline 0 件・exit 1）。`rg -l 'commands.rollbackDailyReportImport' src --glob '!*.test.*' --glob '!src/lib/bindings.ts'` が `src/features/daily-report-import/hooks/useRollbackDailyReportImport.ts` の 1 file だけ（baseline は `useDailyReportImportFlow.ts`）。U1〜U8 が PASS。
- AC3（完了の画面の今の取消）: `DailyReportImportPage.test.tsx`・`DailyReportImportPage.sd.test.tsx`・`hooks/useDailyReportImportFlow.test.tsx`・`CsvImportPage.daily-tabs.test.tsx` の既存の取消・タブの止めの test が assert を弱めずに PASS（U10）。
- AC4（sweep）: `src/hooks/unsaved-changes-guard-sweep.test.ts` の T16・T17 が、期待の集合に各 1 行を足しただけで PASS（完全一致のまま）。
- AC5（登録）: command を足さない。`bash scripts/check-command-drift.sh` の 4 つの値が一致する（baseline `D=76 H=76 S=76 T=76`。#228 の同期の後は値が変わりうるが、一致が契約）。
- AC6（gate）: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-03-daily-report-import-rollback-entry.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が通る。Rust・frontend・bindings・traceability は `docs/DEV_WORKFLOW.md` Verification Gates のとおり。L3 の前に Writer が `cd src-tauri && cargo check --release` を通す（Implementation Rules）。
- AC7（受入の手順書、#229 の merge の後）: 同期の後の版で、`rg -n "前提の lane の後に足す" docs/daily-monthly-acceptance-runbook.md` の当たりに「日: 修正・再取込み後の表示」の行が無く（B の行と判定の節には残る）、その行と足した手順が `日報の取込みの記録` と `取り込んだ日報の記録を見る（取り消すときはここから）` を名指す。手順書が名指す画面の文字列は `rg -nF "<文字列>" src --glob '!*.test.*' --glob '!src/lib/bindings.ts'` で各 1 件以上（報告に並べる。#229 の AC2 と同じ照合）。`rg -n "離れた後の取消の入口" docs/function-design/56-ui-daily-sales.md docs/Plans.md docs/backlog.md` に、入口が無いと読ませる生きた文が残らない。

## Design Readiness

- 引用する設計正本（節まで）: 55 §55.0 の日報取込みの利用者フロー（手順 6）・UI-07-D14（両タブ共通の per-import 取消契約）・UI-07-D15 (3)・(5)・(6)・§55.7（`useBlocker` による importing 中の常時 block、取消の応答を待つ間の block、解除のタイミング）・§55.9（rollback 失敗の UX）・UI-07-D18（Z004 の先例）。37 §37.3 手順 8（BIZ-08-D4）・§37.5・§37.6・§37.9。45 §45.3（DTO の所有）・§45.5・§45.6・§45.8。24 §14.2（`DailyReportImport` の構造体の注記）・§14.20。D-052 C10。UI_TECH_STACK UI-STATE-D2・UI-USW-D1・UI-USW-D4。SCREEN_DESIGN の売上データ取込み画面。design-system DSR-20（戻せない操作の確認）と色だけで示さない規則。56 UI-09a-D13（#229 の版）。
- 必要な設計成果物: updated in this PR（Scope の「本 commit で変えた正本」: 55 §55.12 UI-07-D19 の新設と §55.0・UI-07-D15 (3)・§55.1・§55.7・§55.9、37 §37.2・§37.6、45 §45.3・§45.6・§45.8、24 §14.2 の注記、SCREEN_DESIGN の売上データ取込み画面、ui-task-specs の UI-07。どれも proposed・未実装の注記つき）。実装の PR で注記を外す（S4）。deferred: 56 UI-09a-D13・57 UI-09b-D9 の 1 文（TD-667）・`docs/Plans.md`・受入の手順書の書き足し（#229 の後）、backlog の文（実装の PR）。decision-log は要らない（画面の局所の判断で UI-07-D19 に置き、D-052 C10 の集合は変えない）。UI_TECH_STACK は要らない（UI-USW-D1 は例外の hook を 55 §55.7 に委ねる）。
- plan にしかない durable な判断の昇格先: O1〜O5 の答え（owner の答え〈2026-11-03、repo 外の回答台帳 TD-587〉: O1〜O5 とも a）と起草役の判断 1〜6 は本 commit で 55 UI-07-D19（判断 1 は 37 §37.2・§37.6 と 45 §45.6）へ置いた。精算回数の列（owner の決定 TD-590）は 55 UI-07-D19・37 §37.2・§37.6・45 §45.6・24 §14.2 へ置いた。packet にだけある判断は判断 7（L3 の tree。test の fn の comment だけ）。
- 前提・制約と延期した design gap: 一覧の絞込み（O5）、取込みの記録の詳細の画面（O2 の (b)）、日次売上からの導線（O1 の (b)）は延期。#228 の後も取り消した束を SD から読み直せることは #228 の AC2 と UI-07-D12 からの見込みで、同期の後に確かめる（未確認）。誤りに気づく時期が取り込んで間もない（O5 の理由）は店の事実として未確認の推測。
- 絶対保証（cannot happen / always happens）の例外と抜け道: 「行の `取り消す` はその行の取込みだけを取り消す」は、行の `id` をそのまま `rollbackDailyReportImport` に渡し（U2 の mutation の注入）、BIZ が指定の ID だけを `completed` から更新する（37 §37.5、既存の `test_daily_report_req401_rollback_keeps_same_date_sibling`）ことで保つ。「取消の応答を待つ間に画面を離れない」は route の移動とリロード等の page の離脱だけで、native の window の close は保証しない（UI-USW-D4。取消は単一の TX なので失うのは結果の知らせだけ）。「取消済みに `取り消す` が出ない」は一覧の読取りの時点の状態で、古い一覧から押しても BIZ の冪等の Ok で別の取込みは変わらない。
- 判定: ready。理由: 範囲の要件（REQ-401 の日報取込みの取消、関門 12 の前提 A の owner の観点 TD-578）ごとに、正本の節（55 UI-07-D19・UI-07-D14・§55.7・§55.9、37 §37.5・§37.6、45 §45.6）が本 commit の版にある。O1〜O5 は owner の答え（2026-11-03、repo 外の回答台帳 TD-587）: O1〜O5 とも a で、UI-07-D19 と本 packet に書いた（未解決の設計の問いは無い）。#229 の書き足しは依存として延期を明示し、実装の着手を止めない。Test Plan の表は UI-07-D19 と 37 §37.6 から導ける。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| UI-07-D19（新、入口・画面・行の操作・読込みの状態） | 55 §55.12（本 commit で新設、proposed・未実装） | `CsvImportPage.tsx`・`DailyReportImportRecordsPage.tsx`・route・`types.ts` | U1〜U8 | L3-1・L3-3・L3-5 |
| UI-07-D19 の精算回数の列（TD-590） | 55 §55.12、37 §37.2・§37.6、24 §14.2 | `sales_repo.rs` の行の型と 4 つの SELECT、`list.rs`、記録の画面の列 | B4、U1（`null` の `記録なし`・同じ日の 2 行） | L3-3（同じ日の 2 行を精算回数で見分ける） |
| UI-07-D14（正確な ID・確認の項目・import 単位の行・成功の再取得・失敗の再試行） | 55 UI-07-D14 | 確認の部品・共有の hook・記録の画面 | U2・U3・U4、既存の完了の画面の test | L3-3 |
| 37 §37.5（指定 ID だけ・冪等・在庫不変） | 37 §37.5 | 変えない（呼ぶ入口が増えるだけ） | 既存の `test_daily_report_req401_rollback_idempotent_and_no_stock_change`・`test_daily_report_req401_rollback_keeps_same_date_sibling`・`test_daily_report_rollback_req401_logs_exact_selected_id_and_survives_log_failure` | L3-3（在庫数の文、同じ日の他の取込みが残る） |
| 37 §37.6・45 §45.6・§45.8（一覧の出力の型） | 37 §37.6、45 §45.3・§45.6・§45.8、24 §14.2 | `mod.rs`・`list.rs`・`parse.rs`・bindings | B1〜B4、既存の list の test | 非対象（画面は U1） |
| D-052 C10（取込み・取消の再取得） | decision-log D-052、`src/lib/invalidation-contract.ts` | 共有の hook（集合は変えない） | U3（oracle と一致）、既存の `invalidation-contract.meta.test.ts` | L3-3・L3-4（日次・月次の値） |
| 55 §55.7・UI-USW-D1（取消の応答を待つ間の止めと例外の hook） | 55 §55.7、UI_TECH_STACK UI-USW-D1・D4 | 共有の hook の `useBlocker`、flow の importing の `useBlocker` | U5、T16 | 非対象（native close は UI-USW-D4） |
| UI-07-D15 (6)（日報タブの取込み・取消の間のタブの止め） | 55 UI-07-D15 (6)・§55.7 | flow の `isBlocking`（値は同じ） | 既存の `CsvImportPage.daily-tabs.test.tsx` | 非対象 |
| §55.9（取消の失敗は同じ画面・同じ ID） | 55 §55.9 | 共有の hook・記録の画面 | U4 | 非対象 |
| BIZ-08-D4（拒否の文が案内する操作） | 37 §37.3 手順 8・§37.4 手順 4a | 文は変えない。同じタブの上の導線（O1） | U8（error の状態でも導線が出る） | L3-2 |
| UI-07-D15 (3)（SD の一覧の状態の label、O3 の案内、「同じ精算を取込み済み」の行の「精算 {n}」〈TD-592〉） | 55 UI-07-D15 (3)・§55.12 の O3、37 §37.9、45 §45.6a | `scan.rs`・`mod.rs` の候補の wire、`DailyReportImportPage.tsx` の `SameSettlementImported` の行 | B5、U9 | L3-2（Y の行の `精算 101`）・L3-3（同じ数の行を選ぶ） |
| UI-STATE-D2（search の値の集合は feature の types） | UI_TECH_STACK UI-STATE-D2 | `types.ts`・route | U7 | 非対象 |
| UI-USW-D3（Page の分類の sweep） | UI_TECH_STACK UI-USW-D3 | T17 の EXCLUDED_PAGES（未保存の入力を持たない一覧の画面） | T17 | 非対象 |
| DSR-20・色だけで示さない | design-system 00-foundations、SCREEN_DESIGN | 確認の部品・状態の label | U1・U2 | L3-6 |
| UI-09a-D13 の受入の段の義務（#229） | 56 UI-09a-D13（#229 の版）、受入の手順書 | S4（#229 の後） | 非対象（docs） | AC7 |

隣の契約の sweep（除外の理由）: UI-07-D13（同じ日の追加の確認）は取消の後に同じ日の active な取込みが変わるだけで振舞いを変えない（既存の test が PASS）。UI-07-D16（手で選ぶ経路）は D4 の error の表示の場所（U8）だけで変えない。UI-07-D18 と 65 §65.5a（Z004）は触らない。56・57 の画面の code は変えず、取消の後の値は C10 の再取得で替わる（L3-3・L3-4 で目視）。§14.16〜§14.18 の呼出し（`rollback.rs`・`parse.rs`・`commit.rs`）は repo の行に足す `settlement_no` を読まない（B4 の外。既存の test が PASS）。UI-USW-D4（native close）は別 backlog のまま。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| 37 §37.6 | 一覧が file 名を返さない・内部の JSON を返す・取消済みを落とす・並びが違う | unit（Rust） | B1 `test_daily_report_req401_list_rows_carry_source_filenames_and_status`（SD の束〈`sd_relative_path`・`copy_path` を持つ要素〉と手で選んだ束を同じ日（同じ取込み日時の 2 行を含む）と別の日に取り込み、1 つを取り消す → 行が `source_filenames` に 3 つの file 名、取消済みの行は `status = "rolled_back"` と `rolled_back_at`、並びは対象日・取込み日時・ID の降順） | `source_filenames` を `filename` 以外の key から作る、`completed` だけに絞る、並びを変える、同じ日時の行の ID の順を逆にする |
| 37 §37.6 | 読めない `source_files_json` を空の名前にして黙って通す | unit | B2 `test_daily_report_req401_list_rejects_unreadable_source_files_json`（`NewDailyReportImport` で壊れた JSON の行を直接 insert → `ImportError`） | 読めない行を空の列にする・行を飛ばす |
| 37 §37.6（入力ガード） | page・per_page の境界の検査が外れる | unit | B3 既存の `test_daily_report_req401_list_validation_and_result` を新しい型で（`total_count`・`page: 0` の拒否は今の assert のまま、行の `source_filenames` を足して見る） | ガードを外す、型の写しで件数を落とす |
| 37 §37.6・24 §14.2（TD-590） | 精算回数が行に出ない・別の行の値になる・NULL を 0 にする | unit（Rust） | B4 `test_daily_report_req401_list_rows_carry_settlement_no`（同じ対象日に精算回数 `0101` と `0102` の束を取り込み、精算回数の無い行〈`NewDailyReportImport.settlement_no: None`〉を別の日に直接 insert → 行の `settlement_no` が `Some(101)`・`Some(102)`・`None`、`find_daily_report_import_by_id` で読んだ行も同じ値〈4 つの SELECT の位置のずれを止める〉） | 一覧の SELECT にだけ列を足して他の 3 つの位置がずれる、写しで `settlement_no` を落とす、NULL を 0 に写す |
| UI-07-D19 | 一覧の列・状態の label・`取り消す` の出し分けが違う、同じ日の取込みを見分けられない | component | U1 `DailyReportImportRecordsPage.test.tsx`: `取込み済み` と `取消済み` の行（label と icon の両方）、`取り消す` は `completed` の行だけ、ファイル名は全部。同じ対象日の 2 行の精算回数の列が `101`・`102`、`settlement_no: null` の行は `記録なし`（空の欄・`0` でない） | 状態を色だけで示す、`rolled_back` の行に `取り消す` を出す、精算回数の列を落とす、`null` を空か 0 で描く |
| UI-07-D14 | 確認を経ずに取り消す、別の ID を取り消す、確認の項目が欠ける | component | U2: 行の `取り消す` → dialog に取込みID・対象日・ファイル名・総売上・純売上と本文、`キャンセル` で command を呼ばない、`取り消す` で `rollbackDailyReportImport` がその行の `id` で 1 回（mutation の注入: 同じ日の別の行の id を渡すと red） | dialog を外す、行の取り違え、日付から ID を引く |
| D-052 C10・UI-07-D14 | 取り消した後に一覧・日次・月次が古いまま | component | U3: 成功で Toast `日報取込みを取り消しました`（duration 8000）、invalidate の集合が `d052InvalidationOracle.dailyReportImport()` と完全一致、再取得の後に行が `取消済み` で `取り消す` が消える（mutation の注入: invalidate を外すと red） | 再取得を外す、別の contract を使う |
| §55.9 | 失敗で行・ID を失う、再試行できない | component | U4: 失敗で Toast `取り消しに失敗しました。もう一度お試しください`、行は `取込み済み` のまま、もう一度押すと同じ `id` で呼ぶ（retry） | 失敗で一覧を消す・再読込みで別の行になる |
| 55 §55.7 | 取消の応答を待つ間に画面を離れる・他の行を押せる | unit（hook）＋component | U5 `useRollbackDailyReportImport.test.tsx`: pending の間 `useBlocker` の `shouldBlockFn`・`enableBeforeUnload` が true、成功・失敗の後に false（mutation の注入: 条件を `false` にすると red）。記録の画面で pending の間は全部の行の `取り消す` が disabled、押した行が `取り消し中…` | 止めを外す、解除しない |
| UI-07-D19（読込みの状態） | 失敗が空に見える、失敗の原因が読めない、再試行できない | component | U6: 読込み中の表示、`total_count: 0` で `日報の取込みの記録はまだありません。`、失敗（`import_error`・message `既存日報のファイル情報を読み取れません`）で Alert にその message（`describeError` の結果と完全一致）と `再試行` → 再取得で行が出る | 失敗を空として描く、Alert に固定の文だけを出して CMD の message を落とす |
| UI-STATE-D2・O5・UI-07-D19 | ページの search が壊れる、範囲外のページを「記録が無い」と読ませる | component | U7: `page` 欠落・`0`・文字で 1 ページ目、`listDailyReportImports(page, 20, null, null)`、次のページで `page=2` を呼ぶ、revisit（同じ search で開き直す）で同じページ。範囲外: `page=9` で `items: []`・`total_count: 3` → `このページには記録がありません。` と `1 ページ目を開く` が出て `日報の取込みの記録はまだありません。` は出ない、押すと `listDailyReportImports(1, 20, null, null)`（mutation の注入: 空の文の条件を `items.length === 0` にすると red） | `perPage` の取り違え、search を検証しない、行 0 を一律に空の文にする |
| UI-07-D19・BIZ-08-D4 | 拒否の出た画面から取消の入口へ行けない | component | U8 `CsvImportPage.daily-tabs.test.tsx`: 日報取込みタブの上に `取り込んだ日報の記録を見る（取り消すときはここから）`（href `/csv-import/daily-reports`）、D4 の `import_error`（37 の固定の文）の error の状態でも出ている | 導線を idle の時だけに出す、Z004 タブに置く |
| 37 §37.9（TD-592） | SD の候補に精算回数が載らない・別の値になる | unit（Rust） | B5 `test_req401_scan_candidate_carries_settlement_no`（既存の `test_req401_scan_*` と同じ形の名前）（合成の SD の tree に、同じ日の精算回数 `0101` の束を 2 つ〈bytes 違い〉と `0102` の束、Z002 の無い組を置き、`0101` の 1 つを取り込んでから scan → `SameSettlementImported` の候補の `settlement_no` が `Some(101)`、`NotImported` の `0102` が `Some(102)`、`Incomplete` が `None`） | wire に field を入れない、`fallback` に読めない値を入れる、内部の `Candidate.settlement_no` と別の式で作る |
| UI-07-D15 (3)・O3・TD-592 | 「同じ精算を取込み済み」の行が直し方と「前の取込み」の精算回数を名指さない | component | U9 `DailyReportImportPage.sd.test.tsx`: `SameSettlementImported`・`settlement_no: 101` の行に `精算 101`（完全一致の文）、その下に案内の文と `日報の取込みの記録を開く`。`settlement_no: null` の `SameSettlementImported` の行は案内と link だけで `精算` の文を出さない。`Imported`・`NotImported`（`settlement_no` が数）の行には `精算` の文も案内も出ない | 状態でなく message で出し分ける、全部の行に出す、精算回数を落とす、`null` を `精算 null` や `精算 0` で描く、ほかの状態の行にも数を出す |
| UI-07-D14・UI-07-D15 (6) | 完了の画面の今の取消が変わる | component | U10 既存: `DailyReportImportPage.test.tsx` の確認の dialog、`useDailyReportImportFlow.test.tsx` の成功で `idle`・失敗で `result`、`CsvImportPage.daily-tabs.test.tsx` の取消の応答を待つ間のタブの disabled | 共有の部品・hook に移すときに文・遷移・タブの止めを落とす |
| UI-USW-D1・D3 | 例外の hook・Page の分類の漏れ | unit | T16・T17（`unsaved-changes-guard-sweep.test.ts`） | 新しい hook・Page を集合に足さない、または `useBlocker` を Page で直接使う |

状態の遷移（記録の画面）: initial（読込み中）→ success（行）／empty（`total_count === 0`）／範囲外のページ（1 ページ目へ）／failure（CMD の message）→ retry → 行の `取り消す` → 確認（cancel で戻る）→ pending（全部の行の `取り消す` が disabled、離脱を止める）→ success（Toast・invalidate・refetch で `取消済み`）／failure（Toast、行と ID を保持、retry）→ revisit（同じ search で開き直すと同じページ、U7）→ restart（アプリの再起動の後も DB から同じ行。L3-5）。完了の画面の遷移は今のまま（U10）。

新しい表示の文字列と既存の test の matcher の衝突（`rg -F -c '<語>' src --glob '*.test.*'`、2026-11-03）: `取り込んだ日報の記録を見る`・`日報の取込みの記録`・`日報の取込みの記録はまだありません`・`売上データ取込みへ戻る` は 0 件。`取消済み` は `CsvImportRecordDetailPage.test.tsx` 11・`InventoryRecordsPage.test.tsx` 4・`RollbackConfirmDialog.test.tsx` 1・`src/features/products/search.test.ts` 1（どれも別の画面で、記録の画面を描かない）。`取込み済み` は `DailyReportImportPage.test.tsx` 5・`DailyReportImportPage.sd.test.tsx` 3・`MonthlySalesPage.test.tsx` 1・`CsvImportPage.test.tsx` 1（SD の一覧の label と同じ語。記録の画面は別の route なので同じ描画に並ばない。`CsvImportPage.test.tsx` の 1 件は Z004 の文 `このファイルは取込み済みです（…）` の中で、日報タブの描画と別）。`取り込んだ記録を見る`（Z004 の導線）は `CsvImportPage.test.tsx` 1 で、新しい文 `取り込んだ日報の記録を見る` はこの部分文字列を含まない。`取込み日時` は `DailyReportImportPage.test.tsx` 3・`DailySalesPage.test.tsx` 12（列の見出しにする。別の画面）。O3 の文 `取り込み直すときは、前の取込みを取り消してから読み直してください。` は 37 の D4 の文の末尾と同じ文字を含む: `rg -n '取り消してから' src --glob '*.test.*'` は 0 件（exit 1、2026-11-03。D4 の文を見る test は `src-tauri` の `D4_TEXT` だけ）なので今の matcher とは当たらない。U8・U9 で D4 の error と O3 の行が同じ描画に出る case は無い（D4 の error は preview・error の状態、O3 の行は `sd_list` の状態）が、新しい test は部分一致でなく完全な文で照合する。round 1 の是正で足した文: `このページには記録がありません`・`1 ページ目を開く` は `rg -F -c '<語>' src` で 0 件（exit 1）。`記録なし` は test に `StockHistoryTable.test.tsx` 2・`MovementTable.test.tsx` 2（入出庫の画面の別の文で、記録の画面を描かない）。closure round 2 の是正で足した `精算 {n}`: `rg -F -c '精算 ' src/features/daily-report-import` は 0 件（exit 1）。`精算 101` は test に `CsvImportRecordDetailPage.test.tsx` 1・`CsvImportPage.test.tsx` 1・`StockHistoryTable.test.tsx` 1（Z004 の画面と入出庫の画面で、日報取込みタブの SD の一覧を描かない。`CsvImportPage.test.tsx` の 1 件は Z004 タブの文で、U9 の照合は `DailyReportImportPage.sd.test.tsx` の中の行に限る）。

## L3

前提: 開発用の Windows で、PR の head の native build。DB は空（Coordinator が先例の差し替えの手順で用意し、owner は起動と終了だけ）。入力は合成の SD の tree だけ（起草役の判断 7。Coordinator が `cargo test -- --ignored write_l3_daily_report_rollback_sd_tree` を `INVENTORY_L3_SD_TREE_OUT` つきで回して作り、Windows の path を省略せずに渡す。L3 の日の前日の日付で、精算回数 `0101` の束 X〈総売上 15000 の合成〉と Y〈12000 の合成〉、精算回数 `0102` の束 Z〈9000 の合成〉）。店の実データは使わない。下の順に続けて行う。

- L3-1 / `売上データ取込み` → `日報取込み` タブ / タブの上に `取り込んだ日報の記録を見る（取り消すときはここから）` が読める。`SD から読む` → `場所を選ぶ` で合成の tree を選び、同じ日の 3 行（X・Y・Z）が `取り込めます`。Z を `確認する` → `取り込む`、完了の画面の総売上・純売上を控え、`日次売上を見る` でその日がその値（1 回の取込み）。`売上データ取込み` に戻って `SD から読む` → X を `確認する` → 同じ日の追加の確認で `追加で取り込む` → `日次売上` のその日が 2 回の取込みの合算。
- L3-2 / `日報取込み` タブ / (i) `SD から読む` → Y の行が `同じ精算を取込み済み` と `精算 101` で（ほかの行には `精算` の数が出ない）、その下に `取り込み直すときは、前の取込みを取り消してから読み直してください。` と `日報の取込みの記録を開く`（押すと記録の画面）。(ii) 予備の `ファイルを選んで取り込む` で Y の 3 file を選んで `確認する` → BIZ-08-D4 の文が出て、タブの上の導線が見えている。
- L3-3 / `日次売上` へ移ってから Sidebar の `売上データ取込み` → 導線 → `日報の取込みの記録` / 同じ対象日の 2 行が、精算回数の列（`101`・`102`）で見分けられ、各行に取込みID・ファイル名 3 つ・総売上・純売上・取込み日時と `取込み済み`（文字と icon）。L3-2 (i) で Y の行に出た `精算 {n}` と同じ数の精算回数の行（同じ対象日）を「前の取込み」として選び、その行で `取り消す` → 確認の dialog に同じ項目と「同じ日の他の取込みは残ります。取消しても在庫数は変わりません」。`キャンセル` で何も変わらない。もう一度 `取り消す` → `取り消す` → `日報取込みを取り消しました` が読める間出て、その行が `取消済み` になり `取り消す` が消え、精算回数 `102` の行は `取込み済み` のまま。`日次売上` のその日が L3-1 で控えた Z だけの値（1 回の取込み）に戻り、`月次売上` の公式部門集計から X の分が外れる。
- L3-4 / `日報取込み` タブ / `SD から読む` → Y が `取り込めます` → `確認する` → 同じ日の追加の確認で `追加で取り込む` → `日次売上` のその日が Z と Y の 2 回の取込みの合算（Y の完了の画面の値を足した値）、`月次売上` の公式部門集計がそれに替わる。
- L3-5 / アプリを終了して起動し直す → `日報の取込みの記録` / X が `取消済み`、Y と Z が `取込み済み` で、上から新しい順（対象日・取込み日時の降順）。
- L3-6 / `日報の取込みの記録` と確認の dialog / 文字が既定の表示の大きさで読め、状態を色だけで区別していない。取消の応答を待つ間に Sidebar を押しても画面が移らない（応答は短く、見えなくても FAIL にしない。止めの振舞いは U5 が確かめる）。
