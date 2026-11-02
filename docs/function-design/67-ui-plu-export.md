# UI-08: PLU書出し

> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **対応REQ**: REQ-402
> **Design Phase**: 2026-08-18 D-072 / REQ-907 PLU slot 永続割当・Z004 占有読込みへ更新（SPEC-PLS-D2、D5、D7）。2026-10-16 D-117: 占有は毎日の締めの SD の読取りで自動で照合し、Z004 を選ぶ step・占有の件数の要約・clear 行の解除待ち・`over_limit_warning` を外す（runtime は後続の lane。占有の自動の照合は SD を直接読む runtime に依り、それまで code は今の FilePicker の step を残す）

## 67.1 目的

UI-08 は、商品マスタから CASIO PCツール用 PLUタブ区切りテキストを作成し、利用者が保存したファイルを PCツール / SDカード / レジへ手動投入できるようにする operator-facing flow である。

アプリが保証するのは「対象商品からCP932 PLUファイルを生成し、利用者が保存済みとして扱う対象を記録する」までである。CV17の受理、SDカード書出し、SR-S4000への反映は外部手順で、アプリはその場では確かめられない。レジに入ったコードと名称は、次の締めで SD を読んだときの Z004 の照合に現れる（[33 §16.3](33-biz-plu-export-service.md)）が、単価・課税方式・部門リンクの反映は Z004 に無い（D-117 (7)、D-011 の前提の訂正）。店主は PLU を変えるとき（新商品・値段の変更）にこの画面と CV17 を開き、溜まった未反映をまとめて 1 回で書き出す（D-117 (6)）。

## 67.2 関数要求

**関数要求**: UI-08は、PLUファイルの作成、native save dialogでの保存、保存済み確認、差分対象プレビューを1画面で扱う。ファイル生成時点ではDBを更新せず、利用者が保存済み扱いを明示した後にだけ `confirm_plu_export_saved` を呼ぶ。

## 67.3 シグネチャ

```
function PluExportPage(): JSX.Element
```

主要入力はURL route `/products/plu-export` とコマンド結果である。search paramsでmodeを永続化せず、画面内stateでDiff/Fullを切り替える。

## 67.4 処理ステップ

1. `commands.listPluDirty()` で未反映商品を取得し、その件数を Diff 対象件数として表示する
2. 利用者がDiffまたはFullを選ぶ
3. `getPluSlotSummary` を読み、`snapshot_at` があれば「レジの登録状況: {日時}（売上データ取込みで SD を読んだとき）」を above the fold に 1 行で出す。日時は `snapshot_at`（DB の時差なしの `YYYY-MM-DDTHH:MM:SS`）を `Date` に通さず、共有 `formatDateTime`（`src/lib/date-time.ts`）で `YYYY-MM-DD HH:mm:ss` にして出す。`snapshot_at` が無ければ、書出しの前に warning の Alert で「レジの登録状況がまだ読まれていません。売上データ取込みでレジの SD を一度読んでください。」と売上データ取込み（`/csv-import`）への導線を出す（UI-08-D11）。画面を開いたとき（prepare の前）に、今日の日付と `snapshot_at` の日付の差（暦日）が UI の定数 `PLU_SNAPSHOT_STALE_DAYS = 3` を超えていれば、その行を warning の見た目（icon + 文言。色だけにしない）にし、「最後にレジの登録状況を読んでから日がたっています。売上データ取込みでレジの SD を読んでから書き出してください。」を添える（書出しは拒まない。wire・BIZ・DB は変えない。精算回数が戻って照合が止まったときもこれで見える、33 §16.3）。占有の件数は出さない
4. `commands.preparePluExport({ mode })` を呼び、`bytes_base64`、memory No. 付き prepared rows、`target_product_codes`、`excluded`（要修正一覧）を受け取る。成功後は slot 状態が変わり得るため D-052-C18 の SSOT helper を適用する。`excluded` が空でなければ理由付き一覧を表示し、商品マスタ修正へ誘導する
5. native save dialogで保存先を選び、CP932 PLUファイルバイト列を書き込む
6. 保存キャンセルまたは保存失敗では `confirm_plu_export_saved` を呼ばず、未反映を残す
7. 保存成功後は `target_product_codes`、保存先、件数、文字コード、保存日時を復帰用 `localStorage` に保存する。PLUファイル本文 (`bytes_base64`) は保存しない
8. 画面再表示時に保存済み未確認の復帰状態があれば、ページ上部に `保存済みで未確認のPLU書出しがあります` を表示し、同じ exact product_code set で未反映解除できる導線を出す。保存日時は保存形の `savedAt`（`toISOString()` の UTC）を読むときだけローカル時刻へ直し、`YYYY-MM-DD HH:mm:ss` で出す（解釈できない値はそのまま出す）。保存する形は変えない
9. 利用者が保存済み扱いを確認した場合だけ `commands.confirmPluExportSaved({ product_codes, prepared_rows })` を呼ぶ。復帰状態がある場合は復帰状態の `targetProductCodes` / `preparedRows` を使い、現在の差分一覧から再計算しない
10. confirm成功後に D-052-C14 の SSOT helper を適用し、復帰状態を削除して未反映解除結果を表示する

## 67.5 Design Decisions

| Root | Decision ID | Decision | Why |
|---|---|---|---|
| REQ-402 / D-072 / D-027 | UI-08-D1 | PLUファイル生成とPLU未反映解除を分離する。`prepare_plu_export` は slot 予約を永続化するが商品を反映済みにせず、`confirm_plu_export_saved` が reserved→active と `plu_dirty=false` を同時確定する（D-117 (6) で 2 段を残す）。 | 生成成功だけでは外部投入成功を意味しない一方、再 prepare でも同じ memory No. を使う必要がある。CV17 の取込みに失敗しても、外すまでは同じ商品が差分に残る（D-027）。 |
| REQ-402 / adapter boundary | UI-08-D2 | UI文言では「アプリで確認できるのはPLUファイル保存まで」と明示し、レジ反映済みとは書かない。 | D-011/D-023。アプリはその場では反映を確かめられず、次の締めの Z004 の照合で分かるのもコードと名称だけ（単価・課税方式・部門リンクは分からない。D-117 (7)）。誤って反映確認済みに見せると運用リスクになる。 |
| REQ-402 / CV17 1.1.1 | UI-08-D3 | UI-08 は CV17 1.1.1 `スキャニングPLU(商品)` adapter profile の `.txt` タブ区切りテキストを保存する。ヘッダは11列、memory No. は通常PLU の最大登録数 216 の次の 217 始まり、スキャニングコードは13桁JAN必須とし、product_code fallbackは禁止する。工場出荷時配分（SR-S4000 取説確認済み: PLU総枠 5,000 = 通常 216 + スキャニング 4,784）により217始まり。開始は CV17 の設定で変えられるが、店は 217 のまま（D-117 (7)）。 | 2026-07-03 field gate で、同形状の `.txt` は `CV17 TXT import -> PC tool SD settings write -> SR-S4000 設定読み -> barcode/register behavior confirmation` の流れで反映できることを確認した。CV17 import成功だけでは完了扱いにせず、実際のgateはレジ側バーコード確認までとする。 |
| REQ-402 / recovery | UI-08-D4 | 保存キャンセル・保存失敗では `plu_dirty` と予約を残し、再保存または再生成を案内する。CV17取込み失敗以降は Diff / Full の書出し直しだけで回復する（前に保存した file は取り込まない）。 | 書き出し直しは memory No. が永続しているので同じ枠を使い、保存の後の照合で枠が変わっていれば新しい枠を取る。前に保存した file は、その間にレジ側で変わった枠を上書きしうる（33 §16.8）。`plu_dirty` は confirm まで残る（D-027）。 |
| REQ-402 / safety | UI-08-D5 | Diff / Full とも、書出しの前に画面内 Alert（既存の「既存PLUバックアップ確認」の Alert を言い換えたもの。新しい Alert は作らない）で手順を案内する: 「レジスターツールで読む前に、レジで SD 設定書込みをし、その同じ SD をレジスターツールで読んでください。レジでの設定読込みまでを、次の締めの前に終えてください。」（CV17 の取説: レジで設定書込みをした SD を読まないと、レジ側で設定した内容が消える。事実の一覧 P-044・P-046）。実バックアップファイルはrepoに入れない。217〜5000 の **4,784 slot** は固定件数上限との比較ではなく、snapshot と永続状態で空き・管理範囲を判定する。件数の上限の警告は出さない（`over_limit_warning` は D-117 (5) で撤去）。 | CV17 はレジの設定を保持せず、読んだ SD の設定にファイルの行を重ねてレジへ書く。古い SD を読むとレジ側で変えた設定を失うので、Diff でも同じ手順が要る。次の締めの前に読込みまで終えると、次の締めの照合がその反映を写す。 |
| REQ-402 / D-028 | UI-08-D8 | PLU書出し対象は三分バケットで表示する。対象外（`plu_target=0`）は差分一覧・書出しに現れない。要修正（`plu_target=1` かつ JAN不備・同一JAN価格不一致）は prepare 結果の `excluded` として理由付き一覧で表示し、商品マスタ修正へ誘導する。生成はブロックしない。 | JANなし商品が1件あるだけでPLU書出し全体が止まる構造（Full恒久失敗・Diff恒久残留・通知汚染）を解消する。decision-log D-028。 |
| REQ-402 / D-072 | UI-08-D9 | Diff / Full とも CV17 へ投入できる。Full は対象の商品の app 管理 slot 全体の再同期、Diff は未反映の対象と説明する。前に保存した file は取り込まない（UI-08-D4）。 | `plu_slots` により memory No. が固定され、部分 import の暫定リスクを解消した。旧 file にはこの保証がない。 |
| REQ-402 / status visibility | UI-08-D6 | PLUファイル保存、保存失敗、キャンセル、未反映解除結果はページ上部の注意文直下に表示し、状態遷移後はページ先頭へスクロールする。ページ全体は既存業務画面と同じ内側余白を持つ。confirm失敗は成功色の保存済み表示に混ぜず、失敗タイトルと再試行導線を持つ別 Alert にする。 | 保存後の `この書出しを未反映から外す` は次操作の要であり、下部に出すと利用者が見落とす。confirm失敗が成功表示に混ざると未反映解除に失敗した事実を見落とす。UI-02/03/04/05 の保存結果 visibility follow-up と DSR-01 の共通レイアウト継承に合わせる。 |
| REQ-402 / D-028 L3 | UI-08-D10 | 注意情報はページ上部圏に集約する: 要修正一覧（excluded）は状態表示 section の直後・コンテンツ 2 カラムより前に置き、見出しに warning アイコン + 説明 1 行を添える。CV17 回復手順文言は保存完了の成功 Alert に埋めず、warning トーンの独立 Alert（タイトル `PCツールに取り込めなかった場合の回復手順`）とし、restored pending Alert 内にも同文を強調表示する。 | PR #124 L3 の owner 指摘（要修正一覧が最下部でスクロールしないと見えない / 事故防止文言が埋もれている）。安全側の案内は視線が最初に通る場所 + 強調（色のみ不可）で初めて効く。memory feedback-operator-ui-critical-notes-placement、DSR-03（データ安全系 = 上部 Alert 帯）整合。 |
| REQ-402 / recovery | UI-08-D7 | PLUファイル保存後、未反映解除前に画面遷移・アプリ終了・PC再起動が起きても復帰できるよう、保存済み未確認状態を軽量 `localStorage` に残す。保存するのは `version`、mode、保存先、保存日時、推奨ファイル名、件数、文字コード、exact product_code / memory_no set だけで、ファイル本文や実ファイル内容は保存しない。confirm成功または `破棄して再書出し` で削除する。 | PCツール、SDカード、レジ操作はアプリ外の時間が長く、画面を開きっぱなしにする前提は実運用に合わない。履歴/監査ではないためDBテーブルは作らず、未完了作業の復帰に必要な最小状態だけを保持する。 |
| REQ-907 / D-117 | UI-08-D11 | 占有は毎日の締めの売上データ取込みで SD を読んだときに自動で照合する（[33 §16.3](33-biz-plu-export-service.md)）。この画面は Z004 を選ぶ step と占有の件数の要約を持たず、最終照合日時を 1 行で出す。一度も照合していなければ、書出しより先に売上データ取込みへの導線を出す。Diff 件数は `listPluDirty` 件数とし、0 のときだけ Diff を無効化する。 | 店は締めで毎日 SD を PC に差して読む（D-111）ので、店主の操作を 1 つ増やさずに、既存登録を空きと推測しない gate を保てる（SPEC-PLS-D2、D5、D7）。件数の表示を要する反例は無く、競合は商品を未反映に戻す規則が受け持つ（D-117 (5)）。旧形（FilePicker の step、件数の要約、解除待ちの説明）は D-117 で置き換えた。 |

## 67.6 Route / Components

想定routeは `/products/plu-export`。商品管理エリアの独立画面として扱い、商品一覧のsearch paramsでmode切替しない。

主要component:

- `PluExportPage`: page orchestration、query/mutation、save dialog、状態分岐
- `PluDirtySummary`: 差分件数、最終書出し目安、0件空状態
- `PluRegisterSnapshotPanel`: 最終照合日時の 1 行と、未照合のときの売上データ取込みへの導線（above the fold）
- `PluExportModePanel`: Diff / Full の二択、対象件数、SD 設定書込みの手順の案内（UI-08-D5、Diff / Full とも）
- `PluDirtyProductTable`: 差分対象商品一覧（`plu_target=1` の商品のみが対象。対象外商品はここに現れない）。商品コード、JANコード、商品名、売価、在庫を表示し、JAN未登録は `未登録` と出す。長さの商品の在庫は m で出す（[共通規則](10-common-rules.md) SPEC-UNIT-D3 の「表示する所」の表の 9 行目）
- `PluExcludedTable`: prepare 結果の要修正一覧（D-028）。商品コードと理由（JAN未登録 / 13桁でない / チェックディジット不正 / 同一JAN内の価格・税率不一致）を表示し、商品マスタ編集への導線を出す
- `PluExportResultPanel`: ページ上部の状態表示。保存結果、confirm導線、外部手順、未反映解除結果を注意文直下に出す

## 67.7 State Machine

| State | Meaning | Allowed actions |
|---|---|---|
| `idle` | 差分一覧を表示中 | mode変更、prepare開始 |
| `preparing` | `prepare_plu_export` 実行中 | 操作disabled |
| `save_dialog` | 保存先選択・保存処理中 | 操作disabled |
| `saved` | PLUファイル保存済み、まだ未反映解除していない | 再保存、Diff再生成、書出し済み確認 |
| `restored_pending` | 画面再表示時に保存済み未確認状態を復元した | 書出し済み確認、破棄して再書出し |
| `confirming_exported` | `confirm_plu_export_saved` 実行中 | 操作disabled |
| `confirmed` | app-side exported state更新済み | 差分一覧再取得、商品と在庫へ戻る（D-138 の後。proposed・未実装、今の build に戻る導線は無い）、PCツール手順確認 |
| `error` | prepare/save/confirm 失敗 | 再試行、戻る |

保存キャンセルは `idle` または `saved` に戻す。保存キャンセルと保存失敗は `confirm_plu_export_saved` を呼ばない。

## 67.8 Command Contract

| Command | Input | Output | UI handling |
|---|---|---|---|
| `commands.listPluDirty()` | none | `ProductResponse[]` | 差分対象一覧（`plu_target=1` かつ `plu_dirty=1`、D-028）。0件は空状態でエラーではない |
| `commands.getPluSlotSummary()` | なし | `snapshot_at` | 最終照合日時を above the fold に表示。null なら売上データ取込みへの導線 |
| `commands.preparePluExport({ mode })` | `"diff"` / `"full"` | `bytes_base64`, `suggested_filename`, `content_type`, `encoding`, `count`, `target_product_codes`, `prepared_rows[memory_no]`, `excluded` | 未照合は `register_snapshot_required`。`no_free_slot` を含む要修正は生成から除外。slot 予約・external の採用は永続化し、成功後は D-052-C18 の SSOT helper を適用 |
| `commands.confirmPluExportSaved({ product_codes, prepared_rows })` | prepare結果の `target_product_codes` / `prepared_rows` | `updated_count`, `confirmed_at` | 保存済み確認。成功後は D-052-C14 の SSOT helper を適用 |

`target_product_codes` はprepare時のexact setである。confirm時にUIが現在の差分一覧から再計算しない。

**enum 契約化（D-061）**: `preparePluExport({ mode })` は bindings 由来の `ExportMode` generated union を利用する。値・wire 表現は不変。

### 67.8.1 Browser Recovery Contract

保存済み未確認状態は `localStorage` key `inventory:plu-export:pending:v1` に置く。これは履歴・監査ではなく、画面離脱やアプリ再起動後に同じ保存済みPLUファイルを未反映解除するための復帰状態である。

```json
{
  "version": 1,
  "mode": "diff",
  "savedAt": "2026-07-01T12:00:00.000Z",
  "savedPath": "PLU_20260701.txt",
  "suggestedFilename": "PLU_20260701.txt",
  "count": 1,
  "encoding": "CP932",
  "targetProductCodes": ["BT0001"],
  "preparedRows": [{ "memory_no": 217, "target_product_codes": ["BT0001"] }]
}
```

- 保存しないもの: `bytes_base64`、PLUファイル本文、JAN、商品名、価格、実PLUファイル、PCツール/SDカード/レジ側の結果
- 読込み時にJSON不正、schema不一致、対象コード空なら削除して通常の `idle` として扱う。D-117 の前の形（`overLimitWarning` を持つ）は schema 不一致として消え、通常の画面に戻る（memory No. は永続なので、書き出し直せば同じ番号になる）
- 復帰状態の confirm は `targetProductCodes` を使い、画面上の現在の差分一覧や再prepare結果から対象を再計算しない
- confirm成功時と `破棄して再書出し` 実行時に削除する。confirm失敗時は保持し、再試行できるようにする

## 67.9 UI / Wording

- ページタイトル: `PLU書出し`
- ページ説明（① 説明セクション使用パターン、[02-component-catalog.md](../design-system/02-component-catalog.md) ①）: レジに登録する商品のファイルを作るページです。レジの登録状況は、毎日の売上データ取込みでレジの SD を読むと自動で読み込まれます。『差分を書き出す』（未反映の商品だけ）か『全件を書き出す』を選んで保存し、レジスターツールで取り込んでレジへ反映した後、『この書出しを未反映から外す』を押して確定します。
- Diff mode label: `差分を書き出す`
- Full mode label: `全件を書き出す`
- confirm button: `この書出しを未反映から外す`
- result note: `アプリで確認できるのはPLUファイル保存までです。PCツールへの取込み、SDカード書出し、レジ読込みは手動で確認してください。`
- snapshot status: `レジの登録状況: {YYYY-MM-DD HH:mm:ss}（売上データ取込みで SD を読んだとき）`
- snapshot stale（暦日の差が 3 を超える）: `最後にレジの登録状況を読んでから日がたっています。売上データ取込みでレジの SD を読んでから書き出してください。`
- SD 設定書込みの案内（Diff / Full とも、UI-08-D5）: `レジスターツールで読む前に、レジで SD 設定書込みをし、その同じ SD をレジスターツールで読んでください。レジでの設定読込みまでを、次の締めの前に終えてください。`
- snapshot required: `レジの登録状況がまだ読まれていません。売上データ取込みでレジの SD を一度読んでください。` / action `売上データ取込みへ`
- failure note before confirm: `PCツールに取り込めなかった場合は、差分または全件を書き出し直してください。前に保存したファイルは取り込まないでください（保存の後にレジの登録状況が変わっていると、レジの登録を上書きします）。`（UI-08-D10: 独立 warning Alert と restored pending Alert の両方に出る 1 つの文字列）
- excluded list lead: `これらの商品は今回のPLUファイルに含めていません。商品マスタでJANコード・売価・税率を修正すると、次回の書出しから含まれます。`（UI-08-D10 L3 P3: 修正対象を除外理由の全種（JAN不備 / 同一JANの売価・税率不一致）と一致させ、後半の行動文を強調表示して muted 一色の見落としを防ぐ）
- format validation note: `JANコードが未登録または不正な商品は、スキャニングPLU書出しに含められません。商品マスタで13桁JANを確認してください。`
- excluded list title: `書出しに含めなかった商品（要修正）`
- excluded reasons: `JAN未登録` / `JANが13桁ではありません` / `JANのチェックディジットが不正です` / `同じJANの商品で売価または税率が一致していません`
- import note: `Diff / Full ともレジへ投入できます。`
- restored pending title: `保存済みで未確認のPLU書出しがあります`
- discard button: `破棄して再書出し`

保存、保存失敗、キャンセル、未反映解除結果はページ上部の注意文直下に出す。保存後確認ボタンを差分一覧や書出し設定の下に置かない。保存後は `この書出しを未反映から外す` を主動線とし、再書出しボタンは副導線として表示する。要修正一覧（excluded）は状態表示の直後・コンテンツ 2 カラムより前に置き、見出しに warning アイコンを添える（UI-08-D10。色のみで符号化しない）。

状態は色だけで示さない。`未反映`, `保存済み`, `未反映から外しました`, `レジ反映は未確認` の日本語ラベルを主情報にする。

## 67.10 Query Invalidation

- prepare成功: D-052-C18 の SSOT helper で slot summary を invalidate する。`plu_dirty` は変わらないが、予約・external採用で slot status が変わり得る
- 売上データ取込みの SD の読取り（CMD-12 `scan_register_sd`）成功: 占有の照合で `snapshot_at` と、missing・conflict の商品の `plu_dirty` が変わり得るので slot summary と未反映一覧を stale 化する。D-052 の C17（Z004 を選ぶ snapshot の取込み）はこの規則に置き換わり、runtime の lane が D-052 の Contract 行・production SSOT・独立 oracle を同じ PR で直す（D-117 の Compatibility）
- save cancel / save failure: invalidateしない
- 商品 update / 廃番 toggle 成功: D-052-C2、confirm成功: D-052-C14 を適用し、いずれも slot summary を stale 化する。具体的な query key 集合と除外判断は本書へ複製しない。

## 67.11 Error / Recovery

- Diff対象0件: `listPluDirty` が 0 なら prepareを呼ばず、空状態で「PLU書出しが必要な商品はありません。」を表示する（何が無いかを言う、DSR-11。2026-10-24 に今の build の文へ合わせた、D-138 の design lane の X-06）。Full書出しは選べる
- prepare validation error: Alertで表示し、差分一覧へ戻れる
- 未照合: `register_snapshot_required` を表示し、売上データ取込みへの導線へフォーカスを移す
- JANなし / 13桁以外 / チェックディジット不正 / 同一JAN価格不一致 / `no_free_slot`: prepareは失敗せず、該当商品を `excluded`（要修正一覧）として理由付きで表示する。既存 slot は維持し、今回のファイルには含めない
- save cancel: dirtyは残る。トーストだけで済ませず、画面上に再保存導線を残す
- save failure: Alert + 再保存 / 再生成
- confirm failure: destructive Alert + `もう一度未反映から外す`。confirmが失敗しても保存済みPLUファイル自体は残るが、`PLUファイルを保存しました` の成功 Alert には混ぜない
- restored pending: 保存後に画面遷移、アプリ終了、PC再起動があっても、次回 `/products/plu-export` 表示時にページ上部へ復帰Alertを出す。利用者は `この書出しを未反映から外す` または `破棄して再書出し` を選べる
- invalid recovery state: `localStorage` のJSON不正やschema不一致は削除し、エラー表示ではなく通常の画面に戻す
- save cancel / save failure / save success / confirm success / confirm failure はページ先頭へスクロールし、上部の状態表示を利用者がすぐ読めるようにする

## 67.12 Windows Native L3 / Manual Gate

UI-08 implementation PRでは Windows native L3 と外部手順確認を必須にする。

- navigation: sidebarから `/products/plu-export` へ入れる
- Diff 0件 / Diffあり / Full の表示差
- native save dialogで推奨ファイル名 `PLU_{YYYYMMDD}.txt` が出る
- 保存キャンセル時に未反映が消えない
- 締めの売上データ取込みで SD を読んだ後、この画面の最終照合日時がその読取りの時刻になる。未照合の DB では売上データ取込みへの導線が出て、書出しが拒まれる
- 保存後、confirm前なら同じ memory No. で Diff / Full を再書出しでき、どちらも CV17 へ投入できる
- 保存後確認ボタンと保存失敗時の再保存導線がページ上部に見える
- 保存後、画面遷移またはアプリ再起動相当の再表示でも `保存済みで未確認のPLU書出しがあります` がページ上部に出て、保存先・件数・文字コードを確認できる
- 復帰Alertから `この書出しを未反映から外す` と `破棄して再書出し` の両方を選べる
- confirm後に未反映件数が減り、ホーム通知も更新される
- 画面文言が「レジ反映済み」と誤読されない
- CV17 1.1.1へ保存済み `.txt` を投入し、受理可否、列差異、エラー文言を匿名化して記録する
- CV17 import が通っても、SD-card書出し / SR-S4000設定読込 / register scan-call が通るまで外部manual gateは未通過とする

**適用範囲（D-073、2026-08-19）**: CV17 / SR-S4000 を伴う Windows native L3 は、(1) PLU file 形状（IO-04 出力の header / field / 値）、(2) レジ向け field 値（固定列・課税方式・部門リンク等）、(3) UI-08 の書出し・confirm・復帰の operator flow、のいずれかに触れる PR に必須とする。いずれにも触れない PR（商品一覧の PLU 移行表示・一括対象化・CSV `PLU対象` 列等）は human visual confirmation で足り、レジ実機と CV17 を要さない。

## 67.13 Non-scope

- レジ反映の自動確認
- PLU export history table
- PLUファイル本文のbrowser storage保存
- Z004商品別売上の再評価
- SALES daily report implementation
- 実PLU/実CSV/レジバックアップファイルのrepo保存
