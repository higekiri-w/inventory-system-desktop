# 80. UI-09c: レシートの控え

> **状態**: D-121、実装済み（Plan Packet `docs/plans/2026-11-24-ej-receipts-a2.md`、backlog の「SD の一括取込みと EJ の控えの runtime（A）」の (A-2)）。決定（UI-09c-D1〜D5）の正本は [SCREEN_DESIGN](../SCREEN_DESIGN.md) の「レシートの控え画面」で、本書は画面の状態・search と選んだ行の持ち方・query の key・字形の指定を持つ。

> 対応仕様: REQ-908 / UI-09c（D-121、BIZ-08-D9）
>
> 入力ドキュメント: `docs/SCREEN_DESIGN.md` の「レシートの控え画面」、`docs/function-design/37-biz-daily-report-import-service.md` §37.11（閲覧）、`docs/function-design/45-cmd-daily-report-import.md` §45.6c、`docs/function-design/24-io-csv-import-repo.md` §14.25、`docs/function-design/55-ui-csv-import.md` UI-07-D17・§55.7、`docs/UI_TECH_STACK.md` §2.5（D-052）・UI-USW-D1・UI-USW-D5

レジが SD に書く電子ジャーナル（EJ）の記録を、紙のレシートの控えとして PC で見る画面。読むだけで、在庫・売上の集計を変えない（ADR SPEC-STK-CNT-D11）。書くのは手で選ぶ EJ の取込み（CMD-12 `importEjFiles`、insert-only）だけで、日報取込みの画面と同じ共有の hook を使う。

## 80.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: 記録のある日を日付で、記録を番号で探し、その日の控えを番号の順の表で出し、選んだ控えの文を記録のまま等幅で出す。欠けを区間で知らせ、その場で EJ のファイルを選んで埋められるようにする。

**シグネチャ**: frontend の境界は `commands.listEjRecords(date)`、`commands.findEjRecordsByNo(recordNo)`、`commands.getEjRecord(id)`、`commands.importEjFiles(files)`（45 §45.6c）。page は `EjReceiptsPage({ search, onSearchChange })`。

**処理ステップ**: 画面の状態は §80.3、表示は §80.4、key と invalidation は §80.5、番号で探す流れは §80.6、欠けと手で選ぶ EJ は §80.7。

## 80.1 Component / Route 構成

| 置き場 | 役割 |
|---|---|
| `src/routes/reports/receipts.tsx` | route `/reports/receipts`。`validateSearch` に feature の schema を渡し、`onSearchChange(next, { replace })` を `navigate` にする（`src/routes/reports/daily.tsx` と同じ形） |
| `src/features/ej-receipts/types.ts` | search の schema `ejReceiptsSearchSchema`（§80.2） |
| `src/features/ej-receipts/EjReceiptsPage.tsx` | 画面（`PageShell`）。表・控えの文・番号の欄・欠けの Alert・補助のリンク |
| `src/features/ej-receipts/components/EjDateNavigator.tsx` | 前の日・次の日と日付の入力。UI-09a の `DateNavigator`（56 §56.7）の形を写し、±1 日でなく応答の `prev_date`・`next_date` へ飛ぶ（無ければ disabled）。日付の欄は打っている間は日を移さず、Enter か欄から出たときに移す（カレンダーの窓からの選択も同じ。無効・空なら今の日へ戻す）。前の日・次の日は、欄に focus があるときだけ欄の focus を外さず（欄の外から押すとボタンに focus が移る）、表示中の日から 1 回だけ移る（打ちかけの下書きは捨てる。打ちかけの日へは移らない）。UI-09a の部品は変えない |
| `src/features/ej-receipts/hooks/useEjRecords.ts` | query の hook 3 つ（§80.5） |
| `src/features/ej-receipts/lib/receipt-format.ts` | 種類の label・金額・時刻の純関数と、控えの文の字形の指定（§80.4） |
| `src/features/daily-report-import/hooks/useImportEjFiles.ts` | 手で選ぶ EJ の取込み（55 UI-07-D17 と共有。§80.7） |
| `src/config/navigation.ts` | 「毎日の業務」の末尾、`ui-09b` の後に `ui-09c`「レシートの控え」（icon `Receipt`。[52](52-ui-shared-layout.md) §52.3・§52.4） |

## 80.2 search

- `date`: `z.iso.date().optional().catch(undefined)`（形と暦日の両方。`2026-02-30`・`2026-13-01`・`2025-02-29` を捨て、`2024-02-29` を通す）。無いか捨てたときは記録のある最も新しい日（`listEjRecords(null)`）。
- `no`: ASCII の数字 1〜6 桁の文字列だけ（`z.string().regex(/^[0-9]{1,6}$/).optional().catch(undefined)`）。Router が数に読んだ値（`?no=12`・`?no=1e2`・`?no=100000`）は捨てる。アプリに URL を打つ欄は無く、番号の URL を作るのはアプリの中の link（55 UI-07-D17 の食い違いの番号）だけで、`record_no` は 6 桁の文字列なので文字列のまま届く（`?no=000123` は `"000123"`、link の `search={{ no: "100000" }}` は `?no=%22100000%22` で `"100000"`）。
- `date` と捨てた `no` の組（`?date=2026-11-20&no=12`）は、番号なしでその日を出す（`date` が無いときだけ最新の日）。

## 80.3 画面の状態と選んだ行の持ち方

| 状態 | 表示 |
|---|---|
| 読込み中（初回） | 「読み込んでいます…」。空の文を出さない |
| 記録が 1 件も無い（`date: None`） | 「レシートの控えはまだありません。売上データ取込みで SD を読むと、ここに出ます。」と「売上データ取込みへ」（`/csv-import?tab=daily-report`）。日の移動と番号の欄は出さない。下の補助のリンクは出す |
| その日の記録が無い（`records` が空） | 「この日の控えはありません」。日の移動と番号の欄は出す（`prev_date`・`next_date` で移れる） |
| 読込みの失敗（command のエラー） | destructive の Alert に「レシートの控えを読めませんでした」と原因（`describeError`）と「再試行」（その query を取り直す。UI-09a の失敗の出し方と同じ） |
| 1000 件を超えた日（`truncated`） | 表の上に「この日の控えが多いため、先頭 1000 件だけを出しています」 |
| 日を移る間 | 前の日の表を出したまま（`placeholderData: keepPreviousData`。日の移動と番号の欄を unmount しない） |

- 選んだ行は画面の local の状態（ID と日の組）で、URL に持たない。表示中の日と同じときだけ使う。控えの文は選んだ ID で `getEjRecord` から読むので、番号で見つけた控えが一覧の 1,000 件の外でも出る。
- `no` が search に来たら（同じ画面の link からも）その番号を探し、1 件ならその日に移って行を選び、search を `{ date: <その日>, no: undefined }` に `replace` で置き換える（戻る操作で同じ探しをもう一度しない）。0 件・2 件以上・形の誤りは search を変えずに入力の下に出す（§80.6）。
- 表の行は上下の矢印で選び直し、Enter と Space で選ぶ（roving tabindex。選んだ行か先頭の行だけが Tab で止まる）。

## 80.4 表示

種類の label（UI-09c-D2。色だけに頼らず文字で示す）:

| kind | mode | label |
|---|---|---|
| `sale` | `return` | 返品 |
| `sale` | それ以外 | 取引 |
| `cancelled` | どれでも | 取引中止 |
| `cash_movement` | どれでも | 入金・出金 |
| `receipt` | どれでも | 領収書 |
| `settlement` | どれでも | 精算 |
| `inspection` | どれでも | 点検 |
| `settings` | どれでも | 設定 |
| `training` | どれでも | 練習 |
| `unclassified` | どれでも | その他 |

- 時刻は `printed_at` の `HH:MM`、番号は 6 桁のまま、合計は `1,234円`（`total_amount` が null なら「—」、0 は「0円」）。
- 控えの文は `<pre>` に `body_text` をそのまま入れ、`white-space: pre`（折り返さず横に scroll、行末の空白を保つ）。trim・tab の展開・折り返しをしない。
- 字形は `EJ_RECEIPT_FONT_FAMILY`（`"BIZ UDGothic", "MS Gothic", monospace`）を `src/features/ej-receipts/lib/receipt-format.ts` の 1 か所に置く。1 行 24 バイト（半角 1・全角 2）の並びを崩さない Windows の和文の等幅を先頭にする（Tailwind の `font-mono` は和文の等幅を持たない）。L3-1 の結果（2026-11-26、開発の Windows の native build、PR #305 の head `ac452463`、repo 外の回答台帳 TD-918）: 先頭の BIZ UDGothic がローカル ファイルから当たり、右端の桁がそろって折り返さず、字の大きさも読めた（PASS）。BIZ UDGothic の無い PC で代わりの字形が当たる場合は未観測。

## 80.5 Command / query の key / D-052

| hook | key（`src/lib/query-keys.ts`） | command |
|---|---|---|
| `useEjRecordDay(date)` | `queryKeys.ejRecords.day(date)` = `["ej-records", "day", { date }]`（`date` は null で最新の日） | `listEjRecords` |
| `useEjRecordsByNo(recordNo)` | `queryKeys.ejRecords.byNo(recordNo)` | `findEjRecordsByNo`（`recordNo` が null の間は走らない） |
| `useEjRecordDetail(id)` | `queryKeys.ejRecords.detail(id)` | `getEjRecord`（`id` が null の間は走らない） |

3 つとも `queryKeys.ejRecords.root()`（`["ej-records"]`）の下に置く。取込みの成功は D-052 の C26（SD の一括取込み、55 UI-07-D17）と C27（手で選ぶ EJ、§80.7）がこの prefix を無効にする。今の契約は [UI_TECH_STACK](../UI_TECH_STACK.md) §2.5。C10（日報の個別の取込みと取消）は `ej_records` を書かないので、この prefix を含めない。閲覧の 3 command は `#[tauri::command(async)]`（45 §45.4 の「実行の thread」）。

## 80.6 番号で探す（UI-09c-D3）

- 番号の欄で Enter を押すと探す。同じ番号で Enter を押すと取り直し、その応答で件数か文を出す（一時の読取りの失敗から同じ入力でやり直せる）。IME の変換中の Enter（`KeyboardEvent.isComposing`）は送らない。送る前に既存の `src/components/patterns/normalizeComposedDigits.ts` の `normalizeComposedDigits` で全角の数字を半角にする（IME をオンのまま打った番号で「形が正しくない」と返さない。D-127。[51](51-ui-product-form.md) UI-01b-D16 と同じ部品で、写しを作らない）。BIZ の形の検査は今どおり ASCII の数字だけを受け、0 埋めは BIZ がする（37 §37.11）。
- 結果は 3 つ: 1 件ならその日に移って行を選ぶ（§80.3）。2 件以上（別の 4 桁の欄。今のレジ 1 台の店では起きない）なら移らず、入力の下に「同じ番号の控えが N 件あります」と各件の日付のボタン（`EjRecordRef` の `date`・`id` の昇順。押すとその日に移ってその `id` の行を選ぶ）。0 件なら「その番号の控えはありません」。形の誤りは BIZ の文（「記録の番号は 6 桁までの数字で入れてください」）を入力の下に出す。

## 80.7 欠けと手で選ぶ EJ（UI-09c-D5）

- 欠け（`gaps`、全期間）があれば、一覧の上に warning の Alert（icon + 見出し「控えが足りない所があります」+ 区間ごとに「記録 n〜m の控えがありません。」、1 つの番号は「記録 n の控えがありません。」）。無ければ出さない。
- 画面の下に補助のリンク「EJ のファイルを選んで取り込む」を常に置く（記録が 1 件も無い日・欠けの無い日も。最大の番号の後ろの控えは欠けに出ないため）。押すと共有の hook `useImportEjFiles` が native の file dialog で複数を選び、20MB の検査をして `importEjFiles` を呼ぶ。待つ間（選択の確定から応答まで）は hook が route の移動・リロード・native の window の close（`ej-import`）を止め（55 §55.7、UI_TECH_STACK UI-USW-D5）、リンクと日付の欄・前の日・次の日を押せない（待ちが終わってから打ち直して移る）。待つ間に番号で 1 件見つかったら、待ちが終わってからその日へ移る（§80.6。検索を消さない）。
- 成功で D-052 の C27 を当て、日の一覧を取り直す（埋まれば Alert が消える）。toast は `inserted(result, "選んだファイルから")`（追加ありは「選んだファイルから控え N 件を取り込みました」、0 件は「選んだファイルから新しい控えはありませんでした」）。Alert の下に、`SdBulkImport.tsx` の `EjNotes`（読めないファイルの名・食い違いの番号・保留。55 UI-07-D17 と同じ文、repo 外の回答台帳 TD-310）を出す。日報取込みの画面には成功の toast を出さない（その画面の結果の行が名指す）。
- 読込みと command の失敗は Alert の下に文で出し、もう一度押せる。最後に成功した結果は再試行が失敗しても残る。

## 80.8 Non-scope

金額・商品名で探すこと（D-121 の Revisit）、控えの印刷・書出し、EJ を在庫・売上に使うこと、取り込み済みの EJ を `XZ_BKUP` へ移す操作、売上レポートの `TabsHeader` に 3 つ目のタブを足すこと（Plan Packet の非目的）。

## 80.9 Test Focus

`src/features/ej-receipts/EjReceiptsPage.test.tsx`（読込み中・空・失敗と再試行・1000 件超・日の移動・番号で探す 3 つの結果・IME と `?no=`・控えの文・欠けの Alert・手で選ぶ EJ）、`src/features/ej-receipts/lib/receipt-format.test.ts`（種類の label の全行と金額）、`src/routes/-ej-receipts-route.test.tsx`（実の Router の search）、`src/config/navigation.test.ts`（到達）、`src/features/daily-report-import/components/SdBulkImport.links.test.tsx`（一括の結果からの link）、`src/lib/invalidation-contract.meta.test.ts`（C26・C27 の独立 oracle）。

## 80.10 変更履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-11-25 | ej-receipts-a2（実装、PR #305） | 新設。UI-09c の画面の状態、search と選んだ行の持ち方、種類の label の表、番号で探す 3 つの結果、欠けと手で選ぶ EJ、query の key と D-052 の C26・C27、字形の指定（L3-1 の結果は未確認）を書いた（[Plan Packet](../plans/2026-11-24-ej-receipts-a2.md)）。 |
| 2026-11-25 | ej-receipts-a2（Final の closure round 2 の是正、PR #305） | §80.3 の選んだ行の持ち方を、ID と日の組を表示中の日と同じときだけ使う形に置き換えた。§80.6 に同じ番号の Enter での取り直し、§80.7 に取込みの待ちの間に見つけた 1 件へ待ちの後に移ることを書いた。 |
| 2026-11-26 | ej-receipts-a2（L3 の観察の是正、PR #305） | §80.1 の `EjDateNavigator` に、日付の欄は打っている間は日を移さず Enter か欄から出たときに移すことを書いた（WebView2 で日の 2 桁目の打ちかけが捨てられた）。 |
| 2026-11-26 | ej-receipts-a2（Ready の前の docs、PR #305） | §80.4 に L3-1 の結果（BIZ UDGothic が当たった、TD-918）を書いた。 |
| 2026-11-26 | ej-receipts-a2（Final の closure round 6 の是正、PR #305） | §80.1 に前の日・次の日は表示中の日から 1 回だけ移り打ちかけの下書きを捨てること、§80.7 に取込みの待ちの間は日付の欄と前の日・次の日も押せないことを書いた。 |
| 2026-11-27 | ej-receipts-a2（Final の closure round 7 の是正、PR #305） | §80.1 の前の日・次の日が欄の focus を外さないのを、欄に focus があるときだけにした（番号の欄から押した後の Enter が番号を探していた）。 |
