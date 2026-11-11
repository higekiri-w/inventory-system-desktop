# Plan Packet: 月次の画面に、月の中の取り込んだ日と取り込んでいない日を出す（導入の前の関門 12 の前提 B）

wave に属さない単独の lane。導入の前の関門の 12（[Plans](../Plans.md#導入の前の関門)）の前提 B。owner の決定 2026-11-02（repo 外の回答台帳 TD-578 の O3 = a。PR #229 の packet `docs/plans/2026-11-02-daily-monthly-acceptance.md` の O3 に記録）で、B は自分の R3 の lane になり、関門 12 の前提になった。出発点は PR #229 の branch の `docs/backlog.md` の項目「月次の画面に月の中の取り込んだ日を出す」と、main の `docs/backlog.md` の「UI-09a・09b 将来設計」の取込み済み日数（SALES2-D3 で defer、`get_monthly_sales` DTO 拡張を伴う R3）。本 commit は Plan Packet の起票と plan-first の設計正本の修正（57・34・24・SCREEN_DESIGN・backlog、Scope の「設計正本の変更の一覧」）だけで、実装は計画の固定の後の別の run。

owner の決定が固定した受入の観点（TD-578 の O3。原文の写しを PR #229 の packet から引く。弱めない）:

> B: 月次の画面で、月の中のどの日を取り込んだか（取り込んでいない日はどれか）が分かり、月の公式部門集計にどの日が入っているかを使い手が画面で確かめられる（57 UI-09b-D10 の distinct report date）。

あわせて、本 lane は自分の merge の commit で、受入の手順書 `docs/daily-monthly-acceptance-runbook.md` の「前提の lane の後に足す」の部分（月の取込み済み日の範囲の行）を自分の画面の名で書き足す（PR #229 の 57 UI-09b-D9 と backlog の B の項目が書く義務）。

## Workflow State

- Risk: R3
- Plan Commit: 1d179b893203ab41ffbba4f11fae9597012f7385
- Amendments: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「Tauri command の DTO・状態を持つ画面」に当たる: command `get_monthly_sales` の出力 `MonthlySalesReport` に field を足して bindings を作り直し（下の「現状の調べ」の DTO の行）、月次売上の画面（query・refetch・invalidate の状態を持つ）に表示を足す。R4 にしない理由: 読み取りだけで、DB の列・migration・削除・backup・restore・店の実データに触れない（L3 の入力物は tracked の合成の fixture だけ）。

## Goal

Goal Invariant:

### 最小完了条件

- 月次売上の画面で、日報（Z001/Z002/Z005）を取り込んだ月を開くと、その月の取り込んだ日（取消していない取込みのある日）と取り込んでいない日が、日付の文字で読める。今月の「取り込んでいない日」は前日までで、今日と今日より後の日を並べない（今日を取り込めば今日は「取り込んだ日」に並ぶ。owner の決定 TD-593）。
- 取り込んだ日は、月の公式部門集計（`公式部門集計（レジ日報由来）`）に入っている日と同じ集合で、同じ日に 2 回以上取り込んだ日も 1 日と数える（57 UI-09b-D10 の distinct report date）。
- 取り込んだ日を押して日次売上でその日を見た後、`月次` の tab を押すと、日次で表示中の日の月の月次売上に戻る（月を選び直さない）。
- 取込みの取消・追加の取込み・backup の復元の後、月次の画面を開き直すか戻ると、日の一覧が公式部門集計と同じ時点の値に変わる。
- 受入の手順書の「月: 取込み済み日の範囲」の行が、本 lane の画面の名で書き足され、「前提の lane の後に足す」の文がその行から無くなる（PR #229 の merge の後）。

### 失敗定義

- 取り込んだ日の一覧と公式部門集計が別の日の集合を表す（取消した取込みの日が一覧に残る、Z004・手動販売だけの日が一覧に入る、同じ日の 2 回の取込みが 2 日と数えられる）。
- 取り込んでいない日が色・アイコンだけで示され、文字で読めない。または「営業日」「日平均」と読める語で出る（57 §57.10 Q-1）。
- 今月の今日（締めの前）や、まだ来ていない日が「取り込んでいない日」に並び、取込み忘れと誤読させる（直し方の link も出る）。または今日を取り込んだのに今日が「取り込んだ日」に並ばない（O2、TD-593 で改めた）。
- 取消・取込みの後に月次の画面へ戻っても、日の一覧だけが古いまま残る。
- 取り込んだ日を押して日次売上へ進んだ後、`月次` の tab で戻ると別の月（今月）が開き、使い手が月を選び直す（D-127）。
- 受入の手順書の行が書き足されないまま merge する、または書き足した行が今の画面に無い語・見出しを指す。

### 非目的

- 休業日（定休日・臨時休業）の登録や「営業日」の暦。アプリは店を開けた日を知らない（57 §57.10 Q-1）。
- 月次への Z001 / Z002 の展開（backlog の「UI-09a・09b 将来設計」の残り。D-096）。
- 取込みの履歴一覧と、完了の画面を離れた後の取消の入口（前提 A の lane）。
- 月次の CSV 出力・印刷（`export_sales_csv` は `items` だけを読む。変えない）。
- 日次売上の画面（56）の表示を変える。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 現状の調べ（2026-11-03、main `b5e8fa4c`）

- 画面: `src/features/monthly-sales/MonthlySalesPage.tsx` の `OfficialDepartmentTotalsSection` は見出し `公式部門集計（レジ日報由来）`、説明 `日報取込み済み日の Z005 部門別売上合計です。`（:161）、`rows === null` の warning `この月のレジ日報は未取込みです。`（:168）、空配列の `公式部門集計の行はありません。`（:174）、部門の表だけを出す。月の中のどの日を取り込んだかは出さない（PR #229 の packet の表の行「未取込み日の識別（月の中の日） | できない（B の後）」と同じ）。
- DTO（変更が要る根拠）: `src-tauri/src/biz/sales_service.rs:141-147` の `MonthlySalesReport` は `month`・`mode`・`items`・`prev_month_comparison`・`official_department_totals` だけで、日付の列を持たない。`official_department_totals` の行 `OfficialMonthlyDepartmentTotal`（:166-173）も `department_id`・`label`・`amount`・`quantity`・`count` だけ。`src/lib/bindings.ts:1124-1130` も同じ。IO の `get_monthly_official_department_totals`（`src-tauri/src/db/sales_repo.rs:1552-1600`）は `COUNT(*)` で親の有無を見て、部門ごとに `SUM` するだけで、日付を返さない。
- 既にある別の command（D-147 の「既にある helper」の検討）: `list_daily_report_imports`（`src-tauri/src/cmd/daily_report_import_cmd.rs:315`、bindings の `listDailyReportImports(page, perPage, dateFrom, dateTo)`）は日付の範囲で取込みの行を返すが、使わない。理由: (1) 取込みごとの行で、取消した行も返す（CMD が `status: None` を渡し、IO `list_daily_report_imports`〈`sales_repo.rs:964`〉は status で絞らない）。completed で絞り日付で distinct にする規則（UI-09b-D10）を UI に置くことになり、`UI -> CMD -> BIZ -> IO` の「業務規則は BIZ」に反する。(2) `per_page` の上限 100 の paging で、月の全部を 1 回で得る保証が無い。(3) 公式部門集計と別の query・別の cache key になり、invalidate の時差で 2 つが食い違う時間ができる（失敗定義の 1 つめ）。同じ BIZ の 1 回の呼出し（CMD が DB の mutex を持つ間）で両方を作れば食い違わない。
- 日付の型: `daily_report_imports.report_date` は `TEXT NOT NULL` の `YYYY-MM-DD`、`status` は `CHECK(status IN ('completed','rolled_back'))`、`idx_daily_report_imports_report_date` がある（`src-tauri/src/db/schema_v4.rs:10-16`・`:58`）。
- invalidate: 日報の取込み・取消は `invalidationContract.dailyReportImport()`（`src/lib/invalidation-contract.ts:127-131`）が `monthlySalesRoot()` を含み、`useDailyReportImportFlow.ts:177`・`:198` が呼ぶ。復元の後は `BackupRestorePage.tsx:339` の `queryClient.clear()`。新しい field は同じ query に乗るので、invalidate の集合は変えない。
- 今日の日付: `src/features/daily-sales/lib/date-nav.ts` の `useTodayDate`（`toLocaleDateString("sv-SE")` の local の `YYYY-MM-DD`、画面に戻ると取り直す）がある。曜日は `src/features/daily-sales/components/DailyReportSheet.tsx:26` が `new Date(year, month - 1, day).getDay()` の local で引く（UTC の parse をしない形）。
- 月次の画面の test は `TabsHeader` を mock して router を使わない（`MonthlySalesPage.test.tsx:21-23`）。日の link を足すと router の文脈が要る。`src/test/render-with-router.tsx` の `renderWithRouter` がある。
- `MonthlySalesReport` を作る TS の test の値は 4 か所（`rg -n "official_department_totals" src --glob '!src/lib/bindings.ts'` の出力のうち `MonthlySalesPage.test.tsx:72`・`:106` と `useMonthlySalesReport.test.tsx:71`・`:110`。ほかに `:124` と `:282` は spread で `buildReport()` を上書きする）。必須の field を足すと型の上で全部に足す。
- Rust の test の `get_monthly_sales` の呼出しは `sales_service.rs` の test module にある（`rg -n "get_monthly_sales\(" src-tauri/src/biz/sales_service.rs`。`export_sales_csv` の月次は `report.items.len()` だけを読む、`:466-510`）。
- 新しい語と既存の test の matcher（`rg -F '<語>' src --glob '*.test.*'`、2026-11-03）: `取り込んだ日`・`取り込んでいない日`・`今日より後の日`・`店を開けなかった日` は 0 件。TD-593 の後の文 `今日と、今日より後の日は数えていません。` も 0 件（`rg -n -F '今日と、今日より後の日' src --glob '*.test.*'`、`rg -n -F '今日と' src --glob '*.test.*'` とも 0 件、2026-11-03）。`日分` は `src/features/home/HomePage.test.tsx:184`・`:186`・`:225` の `前日分が未取込みです`（別の画面の完全一致の matcher で、衝突しない）。`売上データ取込みを開く` は `src/features/daily-sales/components/ExportBar.test.tsx:119`（日次の ExportBar の link。同じ語を同じ行き先に使う意図の再利用で、月次の test とは render が別）。`取込み済み日` は `MonthlySalesPage.test.tsx:135`（O5 で説明文を置き換えるなら直す既存の assert）。月次の test の `getAllByRole("row")`（`MonthlySalesPage.test.tsx:180`）は表の行を数えるので、日の一覧は表の行（`<tr>`）で作らない。
- 「営業日」「日平均」は今の `src/features/monthly-sales` の test でない source に 0 件（`rg -n "営業日|日平均" src/features/monthly-sales --glob '!*.test.*'` が exit 1、2026-11-03。計画の review round 1 の P3 で、test の否定の assert に当たらないよう test を外した）。
- 日次から月次への戻り（計画の review round 1 の P2）: 日次売上の `TabsHeader` の `月次` の link（`src/components/sales/TabsHeader.tsx:31` の `<Link to="/reports/monthly">`）は search を渡さず、`MonthlySalesPage.tsx:47-49` は search の `month` が無ければ今月を開く。日次の 2026-03-23 から `月次` を押すと 2026-03 でなく今月が開く。`TabsHeader` は日次・月次の 2 画面が使う共通の部品（`DailySalesPage.tsx:82`・`MonthlySalesPage.tsx:80`、56・57 の Q-3）で、日次は表示中の日を `const date = search.date ?? today`（`DailySalesPage.tsx:47`）で持つ。月次の search の `month` は `monthlySalesSearchSchema`（`src/features/monthly-sales/types.ts:68`）が受ける。`TabsHeader` の test は `src/components/sales/TabsHeader.test.tsx`（実の router で描く `renderAt`）、日次の page の test は `TabsHeader` を mock する（`DailySalesPage.test.tsx:18-20`）。

## owner の判断事項

owner の答え（2026-11-03、repo 外の回答台帳 TD-588）: O1〜O5 とも a。O2 は同じ日に TD-593 で改めた（今月の「取り込んでいない日」に今日も数えない。O2 の下の段）。答えは 57 UI-09b-D11 に書いた。下の選択肢と【推奨】は packet の起票の AI が出したもので（O4 は D-127 の当てはめ。出所も UI-09b-D11 に残した）、下は問いと選択肢の記録。Human Gate の必須の条件にはしない。

- **O1 表示の形**: (a) 公式部門集計の節の中、説明の下・部門の表の上に 2 行の文の一覧: `取り込んだ日（{N}日分）` の後に `23日（月）、24日（火）` の形で日を並べ、各日を日次売上のその日（`/reports/daily?date=YYYY-MM-DD`）への link にする。`取り込んでいない日（{M}日分）` の後に同じ形で日を並べる（link にしない。0 日なら `なし`）【推奨】／(a') (a) から日の link を除く／(b) 月の暦（7 列、各升に日と `取込み済み`／`未取込み` の文字）／(c) 日数と範囲だけ（例 `2日分`）。理由: (a) は文字だけで読め（色に頼らない）、新しい部品の種類が要らず、日を押せば日次の `部門別集計（Z005）` を開けて「月の公式部門集計にどの日が入っているか」を日ごとに確かめられる（受入の手順書の手順 7 の比べ方と同じ）。数を `{N}日分` にするのは、`12日` と書くと「12 日目」と読めるため。(b) は曜日の並びが見やすいが、表の部品と test の `row` の数え方に当たり、code が増える。(c) は「どの日か」が分からず、owner の観点を満たさない。変わる所: (a') なら link の test と L3-3 が無くなる。(b) なら UI-09b-D11 の形・component・L3 を作り直す。
- **O2 今日より後の日**: (a) 数えない。今月は今日（PC の日付）までの日だけを「取り込んでいない日」に並べ、今月だけ文を 1 つ添える。来月より先の月は「取り込んでいない日」を空にする【推奨】／(b) 月の全部の日を数える。理由: (b) は今月にまだ来ていない日を取込み忘れと並べてしまう。今日の日付は日次の画面と同じ `useTodayDate`（PC の local の日付）。変わる所: (b) なら U2・U3 の期待と C3 の文が無くなる。
  - **TD-593 による改め（owner の決定 2026-11-03、repo 外の回答台帳 TD-593、TD-588 の O2 を改める）**: 今月の「取り込んでいない日」に今日も数えない（前日まで。ホームの `前日分が未取込みです` と同じ基準）。今日を取り込めば今日は「取り込んだ日」に並ぶ。今月だけ添える文は `今日と、今日より後の日は数えていません。`。理由: 日報は営業の終わりの締めで取り込む運用（`docs/project-memory.md:157`）で、今日を数えると締めの前は毎日今日が並び、直し方の link も出る（計画の review の closure round 2〈発注 776〉の指摘）。
- **O3 店を開けなかった日**: (a) 「取り込んでいない日」に入れたまま、その行に `店を開けなかった日も含みます。` を 1 文添える【推奨】／(b) 文を添えない／(c) 休業日を登録できるようにする（本 lane の外。休業日の正本が要る）。理由: アプリは休業日を知らない（Q-1 で営業日の扱いを持たない）。定休日のある店なら毎週その曜日が並び、文が無いと取込み忘れと読める。店の事実（定休日・臨時休業の有無）は repo の中で未確認（`rg -n "定休|休業|営業日" docs/project-memory.md docs/Plans.md` が 0 件）。曜日を出す（O1 の (a)）と、定休日の並びは曜日で見分けられる。変わる所: (b) なら C1 の文の assert を外す。
- **O4 取り込んでいない日の直し方**: (a) 「取り込んでいない日」が 1 日以上あるとき、その行の後に link `売上データ取込みを開く`（`/csv-import?tab=daily-report`。日次の印刷の押せない理由〈56 UI-09a-D17〉と同じ語）を出す【推奨】／(b) 出さない。出所: AGENTS.md の「使い手に押しつけない」（D-127: 起きたその画面で直し方を名指す）を本 draft が当てたもので、owner の観点の原文には無い。SD の一覧の窓（今日から 30 日前まで、37 §37.9）より前の日は SD の一覧に出ないが、同じ画面の「ファイルを選んで取り込む」（55 UI-07-D16）で取り込めるので、行き先は 1 つでよい。変わる所: (b) なら C1 の link の assert と L3-4 が無くなる。
- **O5 説明文**: (a) `日報取込み済み日の Z005 部門別売上合計です。` を `「取り込んだ日」に並ぶ日の Z005 部門別売上の合計です。` に置き換える【推奨】／(b) 今のまま。理由: 日を画面に出すと、説明は一覧を指せば足り、「日報取込み済み日」という別の言い方が残ると 2 つの語が同じものを指す（継ぎ足さない、D-127）。変わる所: (a) なら既存の assert `MonthlySalesPage.test.tsx:135` を新しい文に直す（契約の変更による直し。弱めない）。PR #229 の 57 UI-09b-D9 と受入の手順書の今の説明文の引用は、本 lane の同期の後の書き足しで直す（Scope の「#229 の後」）。

## Scope

本 commit で変えた正本（plan-first、Design Phase Rules。計画の review round 1 の是正で 56 の `TabsHeader`・`useTodayDate` の行と 57 UI-09b-D11 の戻り方を足し、closure round 2 の是正で 56 §56.5 と O2 の改め（TD-593）を足した。O1〜O5 は owner の答え〈2026-11-03、repo 外の回答台帳 TD-588〉: O1〜O5 とも a の形。新しい契約には「未実装」と書き、実装の PR で外す）:

- `docs/function-design/57-ui-monthly-sales.md`、`docs/function-design/34-biz-sales-service.md`、`docs/function-design/24-io-csv-import-repo.md`、`docs/SCREEN_DESIGN.md` の月次売上レポート画面の節、`docs/function-design/56-ui-daily-sales.md` の `TabsHeader` と `useTodayDate` の記述（§56.1・§56.5・§56.13。A の lane の段には触れない）。
- `docs/backlog.md`: main の「UI-09a・09b 将来設計」の取込み済み日数の部分を本 lane へ移した旨に置き換えた（月次への Z001 / Z002 の展開の部分は残す）。B の項目は #229 の branch にしか無いので、#229 の後に直す（下）。

設計正本の変更の一覧（本 commit で入れたもの）:

| 正本 | 節 | 変更 |
|---|---|---|
| 57 | `UI-09b-D10` | 「将来「取込み済み日数」を表示する場合は」を今の契約に置き換える: 取り込んだ日の一覧と日数は `official_report_dates`（distinct report date、completed だけ）から作り、import の件数を使わない。 |
| 57 | 新設 `UI-09b-D11`（`D11` は 57 で未使用。`rg -n "UI-09b-D1[1-9]" docs src src-tauri` が 0 件、#229 の branch の 57 も 0 件） | 取り込んだ日と取り込んでいない日の表示（O1〜O5 の答えの形・置き場所・文言・link・今日より後の日・空の月）。`official_department_totals === null` の月は今の warning だけで、日の一覧を出さない。空配列の月は日の一覧と `公式部門集計の行はありません。` の両方を出す。 |
| 57 | REQ-401 第2スライス表示詳細 | 日の一覧の置き場所の 1 行（UI-09b-D11 を引く）。O5 = a なら説明文の語。 |
| 57 | §57.2 bindings 由来の型 | `MonthlySalesReport` の snippet を今の形（`official_department_totals` と新しい `official_report_dates`）に直す（今の snippet は `official_department_totals` を欠く drift）。ファイル構成の表に新しい純関数・component の行。 |
| 57 | §57.3 データフロー・§57.5 hook | DTO の行に `official_report_dates`、page が `useTodayDate` と `compute-report-days` で日の一覧にすること。hook の `derived` に入れず、別の query を足さないこと。 |
| 57 | §57.6・§57.9 | 新しい純関数 `compute-report-days`（下の UI の項）と test の段。 |
| 57 | §57.10 Q-1 | 「取り込んだ日（N日分）」は営業日数ではなく、営業日数・日平均の語を使わない規則のままであることを 1 文。 |
| 57 | §57.12 | 日の書き方 `23日（月）` と数の書き方 `2日分` の 2 行。 |
| 57 | UI-09b-D11 の「日次から月次への戻り」（計画の review round 1 の P2 の是正） | 日次売上の `TabsHeader` の `月次` の link に、日次で表示中の日の月を search `month` で渡す。月次から日次へは今のまま。§57.2 の「UI-09b-D11 で足す file」に `TabsHeader.tsx`・`DailySalesPage.tsx`・`date-nav.ts` の行。§57.6 の `compute-report-days` の返り値を `isCurrentMonth` にする（round 1 の P3）。 |
| 56 | §56.1 の `TabsHeader` の行と `date-nav.ts` の行、§56.5 の `#### useTodayDate`、§56.13 の `useTodayDate` の非目的の行 | `TabsHeader` の行に、日次が表示中の日の月を `month` で渡すこと（57 UI-09b-D11）。§56.1 の `date-nav.ts` の行と §56.5 の見出しの「daily-sales 内閉鎖」を「daily-sales に置き、月次売上も読む」に直し、§56.5 の「将来 UI-09b / UI-06a 等で today/yesterday hooks を多用するようになったら共通化」と §56.13 の行の「将来 UI-09b / UI-06a 等で多用化する時に共通化」を、使い手 2 つでは移さず 3 つめで `src/lib/dates/` へ、に置き換える（`date-nav.ts` の冒頭の comment と同じ判断）。 |
| 57 | 更新履歴 | 本 lane の起票の行。 |
| 34 | `MonthlySalesReport構造体` | `official_report_dates: Vec<String>` を足す（`YYYY-MM-DD`、昇順、重複なし、completed の取込みのある日だけ。日報の取込みが無い月は空）。 |
| 34 | §19.4 手順 6・7 | 手順 6 の「将来coverage countを返す場合は…」を、`sales_repo::get_monthly_official_report_dates` で日を得る手順に置き換え、`official_report_dates` が空 ⇔ `official_department_totals` が `None` の不変条件（同じ conn・同じ範囲・同じ status の条件）と、`sale_records` の日を入れないこと（UI-09b-D8）を書く。手順 7 の返却の列に足す。 |
| 34 | §19.4 の設計判断（新しい段）と更新履歴 | 「取り込んだ日を月次の DTO で返す」: `list_daily_report_imports` を使わない理由、IO の関数を分ける理由、「取り込んでいない日」を BIZ で返さない理由。 |
| 24 | 新設 §14.22a `get_monthly_official_report_dates` | `SELECT DISTINCT report_date FROM daily_report_imports WHERE status = 'completed' AND report_date BETWEEN ?1 AND ?2 ORDER BY report_date ASC`。0 件は `Ok(vec![])`。§14.22 手順 5 の「将来、取込み済み営業日数を返す場合は」を §14.22a を引く文に置き換える（「営業日数」の語も外す）。更新履歴の行。 |
| SCREEN_DESIGN | 月次売上レポート画面 | 取り込んだ日・取り込んでいない日の 1 行（57 UI-09b-D11 を引く）。 |

実装（計画の固定の後）:

- IO（Rust）: `src-tauri/src/db/sales_repo.rs` に `pub fn get_monthly_official_report_dates(conn: &DbConnection, date_from: &str, date_to: &str) -> Result<Vec<String>, DbError>`（24 §14.22a。`get_monthly_official_department_totals` の直後）。既存の `get_monthly_official_department_totals` は変えない（その `None` の契約と test〈`test_get_monthly_official_department_totals_none_req502` ほか〉を保つ。2 つを 1 つの関数にまとめる案は、既存の IO の契約と test を変える範囲の拡大になるので採らない）。
- BIZ（Rust）: `src-tauri/src/biz/sales_service.rs` の `MonthlySalesReport` に `pub official_report_dates: Vec<String>`、`get_monthly_sales` で同じ `date_from`・`date_to` で呼んで入れる（mode に依らない）。
- CMD: `src-tauri/src/cmd/sales_cmd.rs` の `get_monthly_sales` は素通しのまま（変えない）。command の追加・登録の変更は無い（`src-tauri/src/lib.rs` は変えない）。
- 生成物: `src/lib/bindings.ts`（`cd src-tauri && cargo run --bin generate_bindings`。差分は `MonthlySalesReport` の 1 field の見込み）、`docs/function-design/90-traceability.md`（`cargo run --bin generate_traceability`。手で直さない）。
- UI（TS）:
  - `src/features/monthly-sales/lib/compute-report-days.ts`（新）: `computeReportDays(month: string, importedDates: readonly string[], today: string)` → `{ imported: ReportDay[]; notImported: ReportDay[]; isCurrentMonth: boolean }`（`ReportDay = { date: "YYYY-MM-DD"; label: "23日（月）" }`）。月の日を 1 日から月末まで作り（`compute-period-label` と同じ `new Date(year, monthNum, 0).getDate()`）、`importedDates` のうちその月の日を取り込んだ日に、残りのうち `today` より前の日（`date < today`、前日まで）だけを取り込んでいない日にする（今日と今日より後の日は除く。O2、TD-593）。取り込んだ日は今日・今日より後の日でも `importedDates` のとおり並べる。`isCurrentMonth` は `month` が `today` の月と同じときだけ true（`今日と、今日より後の日は数えていません。` を出す条件。来月より先の月は false、57 §57.6）。不正な月は空の結果（throw しない、G-9）。曜日は `new Date(year, month - 1, day).getDay()` の local（UTC の parse をしない）。distinct・completed は BIZ が保証するので、ここでは重複を作らないことだけを守る（BIZ の規則を写さない）。
  - `src/features/monthly-sales/components/OfficialReportDays.tsx`（新）: O1〜O4 の答えの形で 2 行を描く。一覧は `<ul>`（`aria-label` は `取り込んだ日`・`取り込んでいない日`）で、表の行にしない。link は TanStack Router の `<Link to="/reports/daily" search={{ date }}>` と `<Link to="/csv-import" search={{ tab: "daily-report" }}>`。色・アイコンだけで状態を示さない（見出しの文字が状態を持つ）。
  - `src/features/monthly-sales/MonthlySalesPage.tsx`: `OfficialDepartmentTotalsSection` に `reportDates` と `month`・`today` を渡し、`rows !== null` のときだけ `OfficialReportDays` を説明の下・表（か空の文）の上に置く。O5 = a なら説明文を置き換える。`useTodayDate` を `@/features/daily-sales/lib/date-nav` から読む（使い手が日次・月次の 2 つでは `src/lib/dates/` へ移さない）。
  - `src/features/daily-sales/lib/date-nav.ts`: 冒頭の comment の「将来 UI-09b / UI-06a で多用化されたら src/lib/dates/ に共通化（Backlog）」を、今の判断（`useTodayDate` の使い手は日次売上と月次売上〈UI-09b-D11〉の 2 つで、2 つでは移さない。3 つめの使い手ができたら `src/lib/dates/` へ移す）に置き換える。呼出し側に別の comment を足さない（継ぎ足さない、D-127）。code は変えない。
  - `src/components/sales/TabsHeader.tsx`（56・57 の Q-3 の共通部品）: 任意の prop `month?: string`（`YYYY-MM`）を足し、渡されたときだけ `月次` の link に search `{ month }` を付ける（57 UI-09b-D11 の「日次から月次への戻り」）。渡されないときと `日次` の link は今のまま（月次から日次へは今のまま、active の判定〈`activeOptions.includeSearch: false`〉も変えない）。
  - `src/features/daily-sales/DailySalesPage.tsx`: `<TabsHeader month={date.slice(0, 7)} />`（`date` は `search.date ?? today` の表示中の日。日付の入力・前日・翌日・link のどれで日が変わっても同じ式で追う）。月次の `mode`・並べ替えは渡さない（月次の既定に戻る。今と同じ）。
  - test（戻り）: `src/components/sales/TabsHeader.test.tsx` に T1、`src/features/daily-sales/DailySalesPage.test.tsx` に D1〜D3（今の `TabsHeader` の mock を、受け取った `month` を文字で出す mock に替える。既存の assert は変えない）。
  - test: `src/features/monthly-sales/lib/compute-report-days.test.ts`（新）、`src/features/monthly-sales/components/OfficialReportDays.test.tsx`（新、`renderWithRouter`）、`src/features/monthly-sales/MonthlySalesPage.test.tsx`（`buildReport` と `:106` の値に `official_report_dates`、公式部門集計のある test は `renderWithRouter` で包む。`TabsHeader` の mock は残してよい）、`src/features/monthly-sales/hooks/useMonthlySalesReport.test.tsx`（`:71`・`:110` の値に `official_report_dates: []`。振舞いの assert は変えない）。
- 直す既存 test（理由: DTO と契約が変わった所だけ。弱めない）: 上の TS の fixture の 4 か所（型の上で必須）。O5 = a なら `MonthlySalesPage.test.tsx:135` の説明文の assert を新しい文に（同じ強さの完全一致）。Rust の既存 test は field を足しても通る見込みで、assert は変えない（`test_monthly_sales_req502_prev_month_none` に `official_report_dates.is_empty()` を足すのは新しい assert で、弱めではない）。
- #229 の後（PR #229 の merge の後に `git -c merge.directoryRenames=false merge origin/main` で同期してから。それまでは手を付けない）:
  - `docs/daily-monthly-acceptance-runbook.md`: 合格基準の表の「月: 取込み済み日の範囲」の行の「前提の lane の後に足す（B の後に確かめる観点）: …B の lane が、自分の merge の commit でこの部分を自分の画面の名で書き足す」を、本 lane の画面の名で書いた観測できる条件に置き換え、手順の列の「7、前提の lane の後に足す」を「6・7」にする。置き換えの形（O1〜O5 とも a、TD-588。語は実装の画面の語に合わせる）: `月次売上` の `公式部門集計（レジ日報由来）` の `取り込んだ日（N日分）` に、その月に取り込んだ日（比べる日と手順 5 の日）が全部並び、各日を押すと `日次売上` のその日が開く。手順 1 で残した日は `取り込んでいない日` に並ぶ。手順 6 の復元の後は、後で取り込んだ日が `取り込んでいない日` に戻る。取り込んだ日の無い月は `この月のレジ日報は未取込みです。` だけが出る（今の版で見る部分を保つ）。手順 7 の本文に、`取り込んだ日` の一覧を見て和を取る日を選ぶ 1 文を足す。「判定」の節の規則（前提の行が残る版は未了）は A の行のために残す（A の lane の行は触らない）。
  - `docs/function-design/57-ui-monthly-sales.md` の UI-09b-D9 の受入の段の「取込み済み日の範囲」の項: 「今の画面は月の中のどの日を取り込んだかを出さない（…将来の形）。…R3 の lane を受入の前提にし…その lane は…書き足す」を、UI-09b-D11 の画面で確かめる文に置き換える（前提が満ちたので、前提の文と書き足しの義務の文を消す）。「その行が残る版…未了」の規則は A のために残るかを同期の時の文で確かめて決める（A の義務は 56 D13 が持つので、57 D9 から消してよい見込み）。
  - `docs/backlog.md`: #229 が足す B の項目「月次の画面に月の中の取り込んだ日を出す」に、本 lane で済んだこと（PR 番号）を書く。「日報画面の Excel 印刷・バインダー代替受入判定」の「前提は…（A）と…（B）」の文の B を済みにする。
  - `docs/Plans.md` の関門 12 の「前提は 2 つの R3 の lane（O3）…両方が merge されるまで全体の判定は未了」の B の部分に済みを書く（Plans は完了の行を足さない規則〈D-145 (12)〉なので、前提の文の B を外す形にする）。

Non-scope: `src/lib/invalidation-contract.ts`・`src/test/invalidation-oracle.ts`（集合は変えない）、`src-tauri/src/lib.rs`、`docs/function-design/42-cmd-sales-stocktake.md`（signature と手順が変わらない。返り値の型の中身は 34 が持つ）、55・37・45 と 56 の UI-09a-D13 ほか取込み・受入の段（A の lane と日報の取込みの画面）、`export_sales_csv`、日次売上の画面の表示（触るのは `TabsHeader` に月を渡す 1 か所だけ）。56 は §56.1 の `TabsHeader`・`date-nav.ts` の行、§56.5 の `#### useTodayDate`、§56.13 の `useTodayDate` の非目的の行だけを本 commit で直した（下の表）。

他の lane との重なり（2026-11-03、`gh pr diff N --name-only` の目視）:

- PR #229（R4、未 merge）: runbook・57 UI-09b-D9・backlog の B の項目・Plans の関門 12 を足す。本 lane はそれらを #229 の後にだけ触る（上の「#229 の後」）。それ以外の本 lane の変更は #229 に依らない。#229 が merge される前に本 lane の Ready は頼まない（受入の手順書の行の義務を満たせないため）。plan-first で変えた正本と #229 の重なり（2026-11-03、`git merge-file` で main を base に試した）: `docs/backlog.md` は衝突なし。`docs/function-design/57-ui-monthly-sales.md` は UI-09b-D10・D11 と #229 の D9 の段は衝突なしで、末尾の更新履歴の表だけが衝突する（両方の行を日付の順に残して解く）。
- 前提 A の lane（並行して起票中。本 packet は A の作業を計画しない）: 重なりの見込みは、`docs/daily-monthly-acceptance-runbook.md`（A は「日: 修正・再取込み後の表示」の行と手順 5、B は「月: 取込み済み日の範囲」の行と手順 7。行・節の所有を両方の packet に書く）、`docs/function-design/57-ui-monthly-sales.md` の UI-09b-D9（B は「取込み済み日の範囲」の項だけ）、`docs/function-design/56-ui-daily-sales.md`（B は §56.1 の `TabsHeader`・`date-nav.ts` の行、§56.5 の `#### useTodayDate`、§56.13 の `useTodayDate` の行だけ。A は取込み・受入の段）、`src/features/daily-sales/DailySalesPage.tsx` と `DailySalesPage.test.tsx`（B は `TabsHeader` に月を渡す 1 か所と、その mock・D1〜D3 だけ。A が日次売上の画面に取消の入口を置くなら別の所）、`docs/backlog.md`（各自の項目）、`docs/function-design/24-io-csv-import-repo.md`（A が §14.20 `list_daily_report_imports` を触るなら、B の §14.22a と節が別。末尾の更新履歴は merge 順で両方を残す）、`src-tauri/src/db/sales_repo.rs`（A が `list_daily_report_imports` を触るなら別の関数。test module の末尾の追記は両方を残す）、`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`（生成物。手で解かず、取り込んだ後に再生成する、D-098）。A が `invalidationContract.dailyReportImport()` の集合を変えても、`monthlySalesRoot()` が残る限り本 lane の表示は追う（P3 の test が見張る）。
- PR #228: `docs/function-design/24-io-csv-import-repo.md`（§14.19 の前後と更新履歴）・`src-tauri/src/db/sales_repo.rs`（`latest_completed_daily_report_date` の前後）・`src/lib/bindings.ts`・`90-traceability.md`。hunk は本 lane の場所（§14.22・`get_monthly_official_department_totals` の後）と別の見込み。生成物は再生成で解く。
- PR #227・#216・#194: `src/lib/bindings.ts`・`90-traceability.md`（生成物）だけが重なる。

## Acceptance Criteria

- AC1（IO・BIZ）: `get_monthly_sales` の `official_report_dates` が、その月の completed の取込みのある日を `YYYY-MM-DD` の昇順・重複なしで持ち、取消した取込みだけの日・月の外の日・Z004 / 手動販売だけの日を持たない。`official_report_dates.is_empty()` と `official_department_totals.is_none()` が常に同じ（取消の前後を含む）。mode に依らない。下の I1〜I3・B1〜B4 が PASS。
- AC2（wire）: `cd src-tauri && cargo run --bin generate_bindings` の後の `git diff src/lib/bindings.ts` が `MonthlySalesReport` の `official_report_dates: string[]` の 1 field（と生成の comment）だけ。command の数と名前は変わらない。
- AC3（画面）: O1〜O5 の答え（TD-588: 全部 a、O2 は TD-593 の改め）のとおりに、公式部門集計のある月で取り込んだ日と取り込んでいない日が文字で出て、日の数が `{N}日分` で distinct の数と同じ。公式部門集計の無い月は今の warning だけ。下の U1〜U6・C1〜C3・P1〜P5 が PASS。新しい文字列に「営業日」「日平均」が無い（`rg -n "営業日|日平均" src/features/monthly-sales --glob '!*.test.*'` が 0 件）。日次売上の画面から `月次` の tab を押すと、日次で表示中の日の月の月次売上が開く（T1・D1〜D3）。
- AC4（状態）: 取込みの invalidate（`monthlySalesRoot()`）の後の refetch で、日の一覧が新しい値に変わる。取得の失敗では日の一覧を出さず、再試行の成功で出る（P3・P4）。
- AC5（受入の手順書、#229 の後）: `rg -n "前提の lane の後に足す" docs/daily-monthly-acceptance-runbook.md` の当たりに「月: 取込み済み日の範囲」の行が無く、その行が本 lane の画面の語（O1 の答えの見出しの語と `月次売上`・`公式部門集計（レジ日報由来）`）で書かれている。`rg -n "B の lane|前提 B|B の後" docs/daily-monthly-acceptance-runbook.md docs/function-design/57-ui-monthly-sales.md` の残りの当たりが、済んだことを書く文か A と共通の判定の規則だけ（報告に並べる）。runbook の書いた語が `rg -F` で `src/features/monthly-sales` の非 test の source に在る。
- AC6: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-03-monthly-imported-days.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が通る。Rust・frontend・bindings・traceability の gate は `docs/DEV_WORKFLOW.md` Verification Gates のとおり。`## L3` があるので Writer は L3 の前に `cargo check --release`。

## Design Readiness

- 引用する設計正本（節まで）: 57 UI-09b-D8（series の分離）・D9（導入前受入）・D10（distinct report date）・REQ-401 第2スライス表示詳細・§57.2・§57.3・§57.5・§57.6・§57.8（失敗 4 状態）・§57.9・§57.10 Q-1・§57.12、34 `MonthlySalesReport構造体`・§19.4、24 §14.22、42 `get_monthly_sales`（変えない）、56 UI-09a-D17（`売上データ取込みを開く` の語と行き先）、55 UI-07-D16（予備の経路）、37 §37.9（SD の窓）、D-052 と `src/lib/invalidation-contract.ts`、D-127（使い手に押しつけない・継ぎ足さない）、D-147（最小の実装）。
- 必要な設計成果物: updated in this PR（plan-first: 57 UI-09b-D10・D11〈新〉・§57.2/3/5/6/9/10/12、34 §19.4 と構造体と設計判断の段、24 §14.22a〈新〉と §14.22 手順 5、SCREEN_DESIGN の月次の節、56 §56.1 の `TabsHeader`・`date-nav.ts` の行、§56.5 の `#### useTodayDate`、§56.13 の `useTodayDate` の行。Scope の「設計正本の変更の一覧」）。decision-log の新しい D は置かない（横断の判断でなく、57 の局所の判断 UI-09b-D11 と 34・24 の契約で足りる）。
- plan にしかない durable な判断の昇格先（本 commit で昇格した）: O1〜O5 の既定と出所 → 57 UI-09b-D11。`list_daily_report_imports` を使わない理由・IO の関数を分ける理由・「取り込んでいない日」を BIZ で返さない理由 → 34 §19.4 の設計判断の段。UI に置く計算（暦の補集合と今日より後の除外）と BIZ に置く規則（completed・distinct）の境 → 57 UI-09b-D11 の「層の境」。日次から月次への戻り方と棄却した案 → 57 UI-09b-D11 の「日次から月次への戻り」と 56 §56.1 の `TabsHeader` の行。今日より後の文の条件 → 57 §57.6 の `isCurrentMonth`。packet にだけ残る判断は無い。
- 前提・制約と延期した design gap: 休業日はアプリが知らない（O3）。今日は PC の local の日付（日次の画面と同じ前提）。月次への Z001 / Z002 の展開は延期のまま（backlog）。前提 A の画面は本 lane の外。
- 絶対保証の例外と抜け道: 「取り込んだ日と公式部門集計は同じ日の集合」は、同じ BIZ の呼出しの中で同じ conn・同じ範囲・同じ `status = 'completed'` の条件で 2 つの query を読むことで保つ（CMD が DB の mutex を持つ間に書込みは入らない、`sales_cmd.rs:69-74`）。抜け道: 部門の行が 0 の親だけの日は、日には入り部門の合計には何も足さない（空配列の月と同じ扱いで、矛盾ではない。B4 で固定）。「今日より後の日を並べない」は PC の日付が正しい前提で、PC の日付が狂えば並び方も狂う（日次の画面と同じ前提）。
- 層の境（計画の review で確かめたい所）: completed・distinct の規則は IO/BIZ に置き、UI は BIZ が返した日と暦から補集合を作るだけ（`compute-period-label` が月末を UI で出すのと同じ種類の表示の派生）。BIZ に「取り込んでいない日」も返させる案は、今日の日付を BIZ に持ち込み test に時計の差し替えが要るので採らない（推奨）。
- 計画の review の固定の問いへの先回りの答え:
  - (1) 普通の一日: 成立（O1〜O5 とも a、TD-588。O2 は TD-593 の改め）。初期状態: 2026-03 に取込みの無い DB → 日報取込みで 23 日・24 日（2 回）を取り込む → 月次売上の 2026-03 で `取り込んだ日（2日分）` に `23日（月）、24日（火）`、`取り込んでいない日（29日分）` → 23 日を押すと日次の 2026-03-23 → `月次` の tab で 2026-03 の月次売上に戻り、同じ一覧（日次の `TabsHeader` が表示中の日の月を search `month` で渡す。日次で前日・翌日・日付の入力で別の月の日へ動いた後なら、その月に戻る。T1・D1〜D3）。翌日の起動: 同じ DB を読むので同じ表示（今月なら、前の日を取り込まないまま日が変わったとき、その前日の分だけ「取り込んでいない日」が 1 日伸びる。締めの前の今日は並ばない〈TD-593〉。`useTodayDate` は画面に戻ると取り直す）。再試行: 取得の失敗の Alert の `再試行` で一覧が出る（P4）。危険な結果の側（取消した日が残る）は B1・P3 で別に見る。
  - (2) 登録と生成: command は足さない（`lib.rs` の `collect_commands`・`invoke_handler` は変えない）。bindings の再生成と traceability の再生成は Scope にある。route は足さない（既存の `/reports/daily` の `date` と `/csv-import` の `tab` を使う。`src/features/daily-sales/types.ts` の `dailySalesSearchSchema`、`src/features/csv-import/types.ts` の `CSV_IMPORT_TAB_VALUES`）。REQ は REQ-502（月次）で、新しい REQ は足さない。function-design は 57・34・24 の節の更新で、新しい文書は無い。
  - (3) 外の前提: specta の `Vec<String>` → `string[]`（既存の `Vec<T>` の field と同じ。生成の diff で確かめる、AC2）。WebView2 の `<ul>` の折り返しと日本語の括弧の読みやすさは L3 で見る。曜日の local の計算は Windows の TZ が JST の前提で、UTC の parse を避ける形にして TZ に依らないようにする（U5）。
  - (4) データの安全: 店の実データを使わない。test と L3 の入力物は tracked の合成の fixture（`tests/fixtures/daily-report-z001/`、`tests/fixtures/daily-report/`。README に合成と書いてある）だけ。受入の手順書の書き足しは画面の語だけで、店の値・file 名・日付の実例を書かない（#229 の AC6 の規則と同じ）。
  - (5) 前提の出所: 受入の観点は TD-578 の原文の写し（PR #229 の packet の O3）。O1〜O5 の選択肢と推奨は packet の起票の AI が出し、owner が項目ごとに選んで答えた（2026-11-03、repo 外の回答台帳 TD-588: 全部 a）。O2 の今日の扱いは、closure round 2 の指摘を受けて owner が TD-593 で改めた（AI の推奨の「今日までの日」を「前日まで」に）。O4 の推奨の出所は D-127 の当てはめ。店の休業日の事実は未確認。code の事実は上の「現状の調べ」の file:行（main `b5e8fa4c`）。
- この変更で不要になる記述・条件: (i) 57 UI-09b-D10・34 §19.4 手順 6・24 §14.22 手順 5 の「将来…を表示する／返す場合は」の仮定の文（今の契約に置き換える。24 の「営業日数」の語も Q-1 と食い違うので外す）。(ii) main の backlog「UI-09a・09b 将来設計」の取込み済み日数の部分（本 lane へ移す）。(iii) #229 の 57 UI-09b-D9 の「今の画面は…出さない…前提にし…書き足す」と、受入の手順書の「月: 取込み済み日の範囲」の行の「前提の lane の後に足す」の部分（#229 の後に置き換える）。(iv) O5 = a なら説明文の「日報取込み済み日」の言い方。(v) 57 §57.2 の `MonthlySalesReport` の snippet の drift（`official_department_totals` を欠く）。(vi) `date-nav.ts` の冒頭の comment と 56 §56.5・§56.13 の「将来 UI-09b / UI-06a で多用化されたら共通化」、56 §56.1・§56.5 と 57 の冒頭の表の「内閉鎖」（`useTodayDate` を月次売上も読むので、今の形に直す）（今の判断に置き換える。呼出し側に別の comment を足さない）。残すと決めたもの: `get_monthly_official_department_totals` の `COUNT(*)` の親の有無の判定（新しい日の query と意味が重なるが、IO の既存の契約と test を変えずに済み、BIZ の不変条件の test〈B1〉が 2 つの食い違いを見張る）。
- 判定: ready。(1) owner の答え（2026-11-03、repo 外の回答台帳 TD-588）: O1〜O5 とも a。O2 は TD-593 で改めた（今日も数えない）。(2) Scope の「設計正本の変更の一覧」の全部の行を本 commit で正本に入れた（57・34・24・SCREEN_DESIGN・backlog）。ほかに未解決の設計の問いは無い。#229 の後の書き足し（runbook・57 UI-09b-D9・backlog の B・Plans）は設計の問いでなく、#229 の merge を待つ実装の順の制約。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| UI-09b-D10（distinct report date） | 57 UI-09b-D10、34 §19.4 手順 6、24 §14.22a | `sales_repo::get_monthly_official_report_dates`、`sales_service::get_monthly_sales` | I1・B1・U1 | L3-2（24 日の 2 回が 1 日） |
| 34 `MonthlySalesReport.official_report_dates`（新） | 34 構造体・§19.4 手順 7 | `sales_service.rs`、bindings | B1〜B4、AC2 の diff | 非対象 |
| 34 §19.4 の不変条件（日が空 ⇔ 部門の集計が None） | 34 §19.4 手順 6 | `get_monthly_sales` | B1（取消の前後）・B4 | 非対象 |
| 24 §14.22a（completed・範囲・昇順・重複なし） | 24 §14.22a | `sales_repo.rs` | I1・I2・I3 | 非対象 |
| 24 §14.22（変えない） | 24 §14.22 | 変えない | 既存の `test_get_monthly_official_department_totals_*` が PASS | 非対象 |
| UI-09b-D8（series の分離） | 57 UI-09b-D8 | IO の query が `daily_report_imports` だけを読む | B2（Z004 だけの日が入らない） | 非対象 |
| UI-09b-D11（新。表示の形・置き場所・文言・link） | 57 UI-09b-D11、SCREEN_DESIGN の月次の節 | `compute-report-days.ts`、`OfficialReportDays.tsx`、`MonthlySalesPage.tsx` | U1〜U6・C1〜C3・P1・P2・P5 | L3-1〜L3-4 |
| REQ-401 第2スライス表示詳細（null の note・空配列の文） | 57 REQ-401 第2スライス表示詳細 | `MonthlySalesPage.tsx`（今の 2 つの表示を保つ） | P1・P2、既存の `test_monthly_sales_page_no_official_note_req502`・`SC22 / REQ-502` の test | L3-1 |
| §57.8 失敗 4 状態（API の失敗は Alert だけ） | 57 §57.8 | `MonthlySalesPage.tsx`（Alert の枝に日の一覧を出さない） | P4 | 非対象 |
| §57.10 Q-1（営業日・日平均の語を使わない） | 57 §57.10 | 文言 | AC3 の `rg` | L3-2 |
| §57.12 表記（日の書き方） | 57 §57.12（UI-09b-D11 で `{D}日（{曜}）` を足す） | `compute-report-days.ts` | U5 | L3-2 |
| D-052（invalidate の集合） | `src/lib/invalidation-contract.ts`・D-052 | 変えない（同じ query に乗る） | 既存の `invalidation-contract.meta.test.ts`、P3 | 非対象 |
| UI-09b-D11 の日次から月次への戻り（Q-3 の共通部品 `TabsHeader`） | 57 UI-09b-D11、56 §56.1 の `TabsHeader` の行 | `TabsHeader.tsx`（`month` の prop）、`DailySalesPage.tsx`（表示中の日の月を渡す） | T1・D1〜D3、既存の `TabsHeader.test.tsx` の active の 2 test | L3-3 |
| UI-09a-D17 の link の語と行き先（O4 で再利用） | 56 UI-09a-D17 | `OfficialReportDays.tsx` | C1 | L3-4 |
| UI-09b-D9 の受入の行（#229 の後） | 57 UI-09b-D9（#229）、受入の手順書の合格基準の表 | runbook・57・backlog・Plans の書き足し | AC5 の `rg` | 非対象（受入は関門 12 で別に走る） |
| CMD-09 `get_monthly_sales`（変えない） | 42 `get_monthly_sales` | 変えない | 既存の `sales_cmd.rs` の test が PASS | 非対象 |

隣の契約の sweep（除外の理由）: `export_sales_csv` の月次（`items` だけを読む。変えない）、`list_daily_report_imports`（使わない。理由は「現状の調べ」）、日次売上の画面と `useTodayDate`（`useTodayDate` は読むだけで振舞いを変えない。日次売上は `TabsHeader` に月を渡す 1 か所だけ）、`TabsHeader` の active の判定（`activeOptions.includeSearch: false` を変えない。既存の `TabsHeader.test.tsx` の 2 つの active の test が PASS すること）、`MonthNavigator`・`ModeTabs`・sort の URL state（触らない。日次から戻ったときの `mode`・並べ替えは月次の既定に戻る、今と同じ）、前月比（`prev_month_comparison` は商品別の series で、日の一覧と関係しない）。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| 24 §14.22a・UI-09b-D10 | 同じ日の 2 回の取込みが 2 つ並ぶ、取消した日が残る、月の外の日が入る | unit（Rust、IO） | I1 `test_get_monthly_official_report_dates_req502_distinct_completed_in_range`（2026-03-01・03-23・03-24 を 2 つ・03-31 を completed、03-25 を rolled_back だけ、02-28・04-01 を completed → `["2026-03-01","2026-03-23","2026-03-24","2026-03-31"]`） | `DISTINCT` を外す、`status` の条件を外す、`BETWEEN` の端を誤る。昇順は 24 §14.22a の SQL の `ORDER BY report_date ASC` の文で保証し、この test の red で見張らない（`ORDER BY` を外しても `idx_daily_report_imports_report_date` の範囲 scan で昇順のまま返り、red にならない。計画の review round 1 の reviewer が sqlite3 3.54.1 で確かめた） |
| 24 §14.22a | 取込みの無い月で error か `None` を返す | unit（IO） | I2 `test_get_monthly_official_report_dates_req502_empty_month` | 0 件を error にする |
| 24 §14.22a | 同じ日に completed と rolled_back がある日が落ちる | unit（IO） | I3 `test_get_monthly_official_report_dates_req502_same_day_rollback_and_completed` | 日の単位で rolled_back を優先する |
| 34 §19.4 の不変条件 | 日と部門の集計が食い違う（取消の後に日だけ残る） | unit（Rust、BIZ） | B1 `test_get_monthly_sales_req502_report_dates_follow_official_totals`（1 日を取り込む → 日 1・集計 Some → その取込みを `rollback_daily_report_import` で取り消す → 日 0・集計 None） | BIZ が別の範囲・別の status で日を読む、取消の後も日を返す |
| UI-09b-D8 | Z004 / 手動販売だけの日が取り込んだ日に入る | unit（BIZ） | B2 `test_get_monthly_sales_req502_report_dates_exclude_sale_record_only_days`（`seed_sale` の 03-10 と日報の 03-23 → `["2026-03-23"]`） | 日を `sale_records` から読む、2 つの series を合わせる |
| 34 §19.4（mode に依らない） | 部門別の mode で日が空になる | unit（BIZ） | B3 `test_get_monthly_sales_req502_report_dates_same_for_both_modes` | mode の枝の中で日を読む |
| 34 §19.4 の抜け道 | 部門の行が 0 の親の日が落ちる、または集計が None になる | unit（BIZ） | B4 `test_get_monthly_sales_req502_report_dates_with_empty_department_lines`（`seed_daily_report` だけ → 日 1・集計 `Some([])`） | 日を部門の行の join から読む |
| UI-09b-D11（暦の補集合） | 取り込んだ日と取り込んでいない日の分け方を誤る | unit（TS） | U1 `compute-report-days.test.ts`: 2026-03・`["2026-03-23","2026-03-24"]`・today 2026-11-03 → 取り込んだ日 2・取り込んでいない日 29（1〜22・25〜31）・`isCurrentMonth` false | 補集合を誤る、月末を 30 に固定する |
| UI-09b-D11（O2） | 今月のまだ来ていない日が並ぶ | unit（TS） | U2: 2026-11・`["2026-11-01"]`・today 2026-11-03 → 取り込んでいない日は 2 日だけ（今日の 3 日と 4〜30 日は入らない）、`isCurrentMonth` true | `today` の比べ方を `<` でなく `<=` にする（今日が入る）、除外を外す（mutation: `<` を `<=` にすると red、除外の条件を消すと red） |
| UI-09b-D11（O2、今日を取り込んだ日） | 今日を取り込んだのに「取り込んだ日」に並ばない | unit（TS） | U2b: 2026-11・`["2026-11-01","2026-11-03"]`・today 2026-11-03 → 取り込んだ日は 1・3 日、取り込んでいない日は 2 日だけ | 今日での切り捨てを取り込んだ日にも当てる |
| UI-09b-D11（O2） | 来月より先の月に全部の日が並ぶ | unit（TS） | U3: 2026-12・`[]`・today 2026-11-03 → 両方空、`isCurrentMonth` false（文の条件は今月だけ） | 月の比べ方を誤る、`isCurrentMonth` を「日を除いたか」で作る |
| UI-09b-D11（月の長さ） | 閏年・2 月の日数を誤る | unit（TS） | U4: 2028-02 は 29 日、2026-02 は 28 日、2026-04 は 30 日 | `new Date(year, monthNum, 0)` を使わずに日数を決める |
| §57.12（曜日） | UTC の parse で曜日が 1 日ずれる | unit（TS） | U5: `2026-03-21` → `21日（土）`、`2026-03-23` → `23日（月）`、`2026-03-01` → `1日（日）` | `new Date("YYYY-MM-DD")` と `getUTCDay` の組に替える、曜日の表をずらす（mutation: 表の先頭を `月` にすると red） |
| G-9 | 不正な月で throw する、月の外の日を入れる | unit（TS） | U6: `2026-13` → 空・throw しない。`["2026-02-28"]` を 2026-03 に渡しても取り込んだ日に入らない | 範囲の外の日を数える、検査を外す |
| UI-09b-D11（表示と文言） | 状態が文字で読めない、数が distinct と違う、link の先が違う | component（`renderWithRouter`） | C1 `OfficialReportDays.test.tsx`: `取り込んだ日（2日分）`・`取り込んでいない日（29日分）` の文、`getByRole("list", { name: "取り込んだ日" })` の項目の文字、`23日（月）` の link の href が `/reports/daily?date=2026-03-23`、`売上データ取込みを開く` の href が `/csv-import?tab=daily-report`、O3 の文（過去の月 2026-03・today 2026-11-03 なので 31 日まで全部が対象。今月の境は U2・U2b・C3） | 数を import の件数にする、link の search を落とす、見出しの文字を色だけにする |
| UI-09b-D11（全部取り込んだ月） | 取り込んでいない日が 0 のときに空の見出しや直し方の link が出る | component | C2: 全部の日を取り込んだ 2026-02 → `取り込んでいない日（0日分）` と `なし`、`売上データ取込みを開く` が無い | 0 の枝を外す |
| UI-09b-D11（O2 の文） | 今月以外にも文が出る、今月に出ない、今日を数えないことが読めない | component | C3: 今月（2026-11・`["2026-11-01"]`・today 2026-11-03）で `今日と、今日より後の日は数えていません。` が在り、`取り込んでいない日（1日分）` に `2日（月）` だけが並ぶ。過去の月と来月より先の月（2026-12）で文が無い | `isCurrentMonth` を見ない、日を除いたかで文を出す、今日を「取り込んでいない日」に入れる |
| UI-09b-D11（日次から月次への戻り） | `月次` の link が月を運ばず今月が開く、`month` の無いときに空の search を付ける | component（実の router） | T1 `TabsHeader.test.tsx`: `month="2026-03"` を渡すと `月次` の link の href が `/reports/monthly?month=2026-03`、渡さないと `/reports/monthly`（今の href）。`日次` の link の href は両方で `/reports/daily` | `月次` の link に search を渡さない、`month` が無いときも search を付ける |
| UI-09b-D11（日次の表示中の日の月） | 日次で日を動かした後も前の月を渡す、今日の月を渡す | component（日次の page） | D1 `DailySalesPage.test.tsx`: `search.date` `2026-03-23` → mock の `TabsHeader` が受けた `month` が `2026-03`。D2: `2026-03-31` で `翌日` を押す → `onSearchChange` の updater の結果の search で rerender → `2026-04`。日付の入力で `2026-05-10` にした結果の search で rerender → `2026-05`。D3: `search.date` 無し → 今日（`useTodayDate`）の月 | `month` を `today` から作る、`search.date` の初回の値で固定する（mutation: `month={today.slice(0, 7)}` にすると D1・D2 が red） |
| REQ-401 第2スライス（null） | 未取込みの月に日の一覧が出て warning と重なる | component（page） | P1: 既存の `test_monthly_sales_page_no_official_note_req502` に `queryByRole("list", { name: "取り込んだ日" })` が null を足す | null の月にも一覧を描く |
| REQ-401 第2スライス（空配列） | 部門の行の無い月で日の一覧が消える | component（page） | P2: 既存の `SC22 / REQ-502` の test（`official_department_totals: []`）に `official_report_dates: ["2026-05-02"]` と一覧の在ることを足す | 一覧を表の枝の中だけに置く |
| D-052・State（invalidate → refetch） | 取込み・取消の後も日の一覧が古い | component（page） | P3: 初回の resolve で `["2026-05-02"]` → `queryClient.invalidateQueries({ queryKey: queryKeys.monthlySalesRoot() })` → 2 回目の resolve で `["2026-05-02","2026-05-03"]` → `取り込んだ日（2日分）` | 一覧を初回の値で memo して固定する、別の query key に置く |
| §57.8・State（failure → retry） | 取得の失敗で古い一覧や空の一覧が出る、再試行で戻らない | component（page） | P4: 1 回目 error → Alert だけで一覧が無い → `再試行` → 成功で一覧 | error の枝に一覧を描く |
| State（revisit・月の切替え） | 月を変えても前の月の日が残る | component（page） | P5: 2026-05 の一覧 → `onSearchChange` で 2026-04 の search で rerender → 2026-04 の resolve の日だけ | 一覧の month を search でなく固定値から作る |
| UI-09b-D11（O2、page の今日の結線、TD-593） | page が `useTodayDate` の今日でなく別の値を `computeReportDays` に渡し、今月の今日と今日より後の日が並ぶ | component（page、Date だけの偽の時計） | P6 `MonthlySalesPage.test.tsx`: `vi.useFakeTimers({ toFake: ["Date"] })`・`vi.setSystemTime(new Date(2026, 4, 3, 12))`、月 2026-05・`official_department_totals: []`・`official_report_dates: ["2026-05-01"]` → `取り込んでいない日（1日分）`、一覧の項目は `2日（土）` だけ、`今日と、今日より後の日は数えていません。` が在る | page の today を固定の値にする、`computeReportDays` に今日でない日（年末）を渡す |

状態の遷移: initial（Skeleton、一覧なし）→ success（一覧）→ invalidate → refetch（新しい一覧、P3）／failure（Alert だけ）→ retry → success（P4）／revisit（月の切替え、P5）。restart（翌日の起動）は同じ DB と `useTodayDate` の取り直しで、U2 の today を変えた計算と同じ（新しい test は足さない）。

mutation の注入（Contract Audit で実装に注入して red を確かめる）: IO の `DISTINCT` を外す（I1。`ORDER BY` を外す mutation は red にならないので使わない）、`TabsHeader` の `月次` の link から search を外す（T1）、日次で `month` を今日から作る（D1・D2）、`isCurrentMonth` を「日を除いたか」に替える（U3・C3）、今日の境の `<` を `<=` にする（U2・C3）、`status = 'completed'` を外す（I1・B1）、BIZ で日を `sale_records` から読む（B2）、UI の今日と今日より後の除外を消す（U2）、曜日の計算を UTC にする（U5、TZ が JST の環境では red にならないことがあるので、表のずらしの mutation を併せて使う）、P3 の一覧を初回の値で固定する、page の today を固定の値（`"2000-01-01"`）にする・`computeReportDays` に年末を渡す（P6）。

## L3

前提: 開発用の Windows、本 lane の版の build。2026-02・2026-03 に日報の取込みが無い開発用の DB（前の L3 で 2026-03 を取り込んだ DB なら、起動の前に Coordinator が L3 の依頼で空の DB で起動する手順を渡す）。入力物は tracked の合成の fixture（CP932・CRLF。README に合成と書いてある）: `tests/fixtures/daily-report-z001/` の 2026-03-23 の束（A）と 2026-03-24 の 2 束（B1・B2）。取込みは `売上データ取込み` の `日報取込み` の tab の「ファイルを選んで取り込む」（55 UI-07-D16）で、A → B1 → B2 の順（B2 は「追加で取り込む」）。O1〜O5 とも a（TD-588）の形で書く。

- L3-1 / 月次売上（`月次` の tab）/ 月の入力で 2026-02 を開く / `公式部門集計（レジ日報由来）` の下に今の `この月のレジ日報は未取込みです。` だけが出て、`取り込んだ日` の一覧が出ない。
- L3-2 / 月次売上 / A・B1・B2 を取り込んだ後に 2026-03 を開く / `取り込んだ日（2日分）` に `23日（月）、24日（火）`、`取り込んでいない日（29日分）` に `1日（日）` から `22日（日）` と `25日（水）` から `31日（火）` が日本語の文字で並び、`店を開けなかった日も含みます。` の文がある。色だけで分けていない。行が画面の幅で折り返しても日が切れず読める（Windows の既定の字の大きさ）。「営業日」の語が無い。
- L3-3 / 同じ画面 / `23日（月）` を押す / 日次売上の 2026-03-23 が開き、`部門別集計（Z005）` が出る。その画面の `月次` の tab を押すと、月を選び直さずに 2026-03 の月次売上が開き、同じ一覧が出る。日次で `翌日` を押して 2026-03-24 にしてから `月次` を押しても 2026-03 が開く。
- L3-4 / 同じ画面 / `売上データ取込みを開く` を押す / `売上データ取込み` の画面の `日報取込み` の tab が開く。
- L3-5 / 月次売上 / L3-1 の後に、2026-02 の 1 日から 28 日までの日報を全部取り込んだ後に 2026-02 を開く（28 日分の合成の束は tracked の fixture に無いので、Coordinator が L3 の依頼で合成の束か取り込んだ後の開発用の DB を渡す） / `取り込んだ日（28日分）` の後に `取り込んでいない日（0日分）` と `なし` が出て、その下の `店を開けなかった日も含みます。` の文と並べて読んでも意味が取りやすい（`なし` の直後の文が浮いて見えない）。`売上データ取込みを開く` が無い。Windows の既定の字の大きさで、`なし` と文が本文の一覧より小さい補足の字に見える（発注 781 の裁定 2、PR #236 の comment issuecomment-6608079577）。

今月の今日と今日より後の日の扱い（O2、TD-593）は今月の日付の fixture が要るので L3 に置かず、U2・U2b・U3・C3 と page の結線の P6 で見る。L3-1〜L3-5 は過去の月（2026-02・2026-03）なので、TD-593 で期待は変わらない。取消・復元の後の追従は自動 test（B1・P3）と、関門 12 の受入の手順書の手順 5・6 で見る。
