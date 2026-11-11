> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)
> **入力ドキュメント**: ARCHITECTURE.md（タスク仕様）、[ui-task-specs.md §UI-09a/09b](../architecture/ui-task-specs.md)（タスク要求）、[SCREEN_DESIGN.md §日次売上レポート画面](../SCREEN_DESIGN.md)（レイアウト判断）、[2026-05-17-phase-2-ui-09a.md](../archive/plans/2026-05-17-phase-2-ui-09a.md)（実装プラン archive、本書の判断根拠）

## 56. UI-09a: 日次売上レポート画面

> **2026-06-30 REQ-401 redesign note**: 日次売上画面は、Z001/Z002/Z005由来の公式日報集計と、Z004/手動販売由来の商品別売上明細を分けて表示する。日報取込み済みでも商品別明細が空の場合があるため、「商品別がない=売上がない」と扱わない。

### 本書のテンプレ判定（業務ロジックあり版、共通 6 項目）

UI 層関数設計書の 2 段階テンプレ（業務ロジック有無で使い分け、`memory/frontend-function-design-granularity.md`）に従い、UI-09a は**業務ロジックあり版**と判定する。

**判定根拠**:

- CMD 呼び出し: `commands.getDailySales(date)` ×2（当日 + 前日 useQuery、部分障害許容）+ `commands.exportSalesCsv("daily", date)` ×1（useMutation、Blob ダウンロード）
- 入力バリデーション: URL search params の zod 4 validateSearch + `<input type="date">` ネイティブバリデーション
- 画面内部 state 駆動のフロー分岐: 単価派生計算 + 部門小計行挿入 + 前日比カード部分障害 fallback + 列ソート（5 列対応、`unit_price=null` 末尾配置） + CSV エクスポート Blob memory 安全性

→ **業務ロジックあり版**。共通 6 項目（コンポーネント構成 / React State / CMD 呼び出し / 利用者操作フロー / エラー表示 / ローディング表示）+ ショートカット / 備考 / 非目的 + 更新履歴の 10 章構成。

UI-00 ホーム同型の「簡潔版 = useState + useQuery + 純関数」を採用（useReducer は不要、状態数 6 未満 + 並行/cancel なし、`memory/feedback-recommend-with-explicit-basis.md`）。React 19 React Compiler で memoize 自動化、`useMemo` / `useCallback` 明示なし。

---

### 56.1 概要

`/reports/daily` route 配下に配置する読み取り専用ダッシュボード。SP-501-01〜07 + REQ-501 に直接対応する。SCREEN_DESIGN.md の主要な画面間遷移「売上データ取込み完了 → 売上レポート」を成立させる。

**REQ-401再設計後の表示領域**:

| 領域 | データソース | 表示意味 |
|---|---|---|
| 日報サマリ | `DailySalesReport.official_daily_report`（daily_report_imports / daily_report_*_lines） | レジ日報の公式集計。総売上、純売上、支払集計、部門別集計 |
| 日報サマリの日計（Z001） | `DailySalesReport.official_daily_report.summary_imports`（daily_report_imports / daily_report_summary_lines） | Z001 の全行をレジの並びのまま取込みごとに表示する。合算しない（UI-09a-D16） |
| 商品別明細 | `DailySalesReport.items`（sale_records） | Z004商品別CSVまたは手動販売出庫から得た商品別売上 |
| 商品別部門小計 | `DailySalesReport.department_subtotals`（sale_records集計） | 商品別明細が存在する範囲の部門小計 |

`UI-09a-D12`: 日報サマリと商品別明細を1つの表に混ぜない。Z001/Z002/Z005は商品コードを持たないため、商品名・JAN付き一覧を生成できない。商品別一覧はPLU/Z004または手動販売出庫の範囲だけを表す。

`UI-09a-D13`（導入前受入）: 現行Excelファイル群はZ001/Z002/Z005を毎日ほぼそのまま貼り付けて上書きするため、Excel側に日別履歴を残さない。印刷・バインダーは各営業日の日報を残す現行唯一の手段で、紙は店の控えと店の外の提出先へ渡す用を兼ね、提出先は紙を全部求める（2026-09-29 の店主の答え、repo 外の回答台帳 TD-059・TD-060。旧版の「外部提出帳票ではなく」を直した）。紙はやめず、アプリが 1 日 1 枚の日報の紙を出して手で写す作業を無くす（UI-09a-D17、[D-120](../decision-log.md)。2026-09 の「印刷機能は受入結果から必要性を決める」を置き換えた）。go-live前の受入（実際の1日分と横並びで、公式集計、過去日到達、未取込み日の識別、修正・再取込み後の表示、backup/restore後の再現）は、運用開始（go-live）の前に行う。運用開始の後の紙の照合（UI-09a-D19 の (2)）とは別で、この受入が先。

受入の場と合格基準（2026-11-02 の起票。[packet](../plans/2026-11-02-daily-monthly-acceptance.md) の O1〜O6 への owner の答え、repo 外の回答台帳 TD-578。手順は [受入の手順書](../daily-monthly-acceptance-runbook.md)）:

- 場と時期: 開発用の Windows で、店へ入れる版の MSI を、開発の data folder を退避した後の空の DB に入れ、店の PC への初期投入（[initial-load-runbook.md](../initial-load-runbook.md) の流れ 3）の前に、管理者が行う（店の PC では行わない）。比べる日は運用開始の前の実際の日で、訪店の時にレジの SD の `CASIO` の folder を、管理者の USB メモリの root でない日付の付いた folder の中へ丸ごと写した写し（`<日付の folder>\CASIO\SR500_550_4000\XZ` の形。開発用の Windows へ写すときも同じ形）を「SD から読む」→「場所を選ぶ」で読み（[29](29-io-daily-report-parser.md) IO-09 の `resolve_register_sd_root`。窓は [37](37-biz-daily-report-import-service.md) §37.9 のとおり開発用の Windows の今日から 30 日前まで）、その日の綴りの紙（今の Excel の紙）と並べる。綴りの紙は、紙かそのコピーを店から借りて開発用の Windows の前で並べ、返す（写真は手段にしない。借りた日と返した日を管理者の記録に書く。repo 外の回答台帳 TD-580）。写しと受入の data folder は店の実データで、repo の checkout（ignored の `.local/` を含む）・cloud の同期の外の、drive の直下でない folder に置き（[71](71-mnt-backup.md) MNT-01-D10）、AI の agent への発注・報告・handoff に path・値を書かず、agent に読ませる作業をしない（agent は開発用の Windows の file を読めるので、場所でなく渡し方で守る）。消す時期は owner が決める（71 §71.13 の別の PC に店のデータを置く間の扱い）。店の DB には受入の記録を残さない。導入の日に店の PC で初めて SD を読むと運用開始の前の日が「取り込めます」で並ぶ扱いは、この受入でなく初期投入の側で別に扱う。
- 公式集計: 選んだ 1 日（窓の中に同じ日に 2 回以上精算した日があればその日も）で、日次売上の画面の `日計（Z001）`・`支払集計（Z002）`・`部門別集計（Z005）` の行の名称・個数/件数・金額が、綴りの紙のその日の同じ帳票の枠の行と全部同じ（値が 0 の行を含む）。2 回以上の日の綴りの紙が何を載せているかは店の事実で未確認で、決まるまでは 1 回の日で比べ、2 回以上の日は並べて違いを控える（合否に数えない）。2 回以上の日を取り込んで並べるのは、月次の判定（下の backup/restore 後の再現と UI-09b-D9 の修正後の値）を済ませた後にする。月次は同じ日の 2 回分を足すが（UI-09b-D10）、その日の紙が 2 回分を載せるかが未確認なので、月次の判定は、紙を借りた 1 回精算の 2 日（比べる日と、下の修正・再取込み後の表示で取り込み直す日）だけを取り込んだ状態で行う。
- 過去日到達: 日付の入力と前日・翌日で比べる日へ着く。
- 未取込み日の識別: 取り込んでいない日に `この日付のレジ日報は未取込みです。` が出る（月次は UI-09b-D9）。
- 修正・再取込み後の表示: 取込みを取り消すとその日の値が取込みの前へ戻り、同じ束を取り込み直すと元の値に戻る（取り込み直した後、日次売上の画面のその日の 3 つの表が、借りたその日の綴りの紙と枠・行ごとに同じ。その日は比べる日と同じ月の 1 回精算の日から選び、紙を借りる。受入で値を比べる相手は借りた紙だけで、画面の値を写し・写真・書き写しで運ばない。repo 外の回答台帳 TD-650）。今の版で取り消せるのは取込みの完了の画面を離れる前だけで、離れた後の取消の入口（[backlog](../backlog.md) の「日報取込み標準手順の残設計」の履歴一覧と取消の導線）は無い。入口を作る R3 の lane を受入の前提にし（TD-578 の O3）、その lane の後に、誤りに気づいたとき離れた後でも取り消して正しい束を取り込み直せることを確かめる。その lane は、自分の merge の commit で、受入の手順書の「前提の lane の後に足す」の行を自分の画面の名で書き足す。
- backup/restore 後の再現: 受入の data folder の中で、取込みの後に作った backup → 比べる月の、まだ取り込んでいない日の束を 1 つ取り込み、その日の値が変わるのを見る → その backup を復元（同じ日のうちに）→ 日次・月次の値が backup を作った時の値と同じ。backup を作った時の値は金額を書かずに次で確かめる: 比べた日の日次は借りた綴りの紙と同じ、後で取り込んだ日は未取込みの note に戻る、月次の公式部門集計はその月に取り込んだ日（後で取り込んだ日を除く。比べる日と、修正・再取込み後の表示で取り込み直した日）の借りた綴りの紙の部門別集計の和（紙から電卓で出す）と同じ。取消を使わないので、この項目は取消の入口の lane に依らない（離れた後の取消は上の修正・再取込み後の表示の項目が確かめる）。
- 合わない項目が 1 つでもあれば運用を始めない（Plans の導入の前の関門の 12）。「前提の lane の後に足す」のままの行が残る版、または前提の lane（ここの取消の入口と UI-09b-D9 の月の中の取り込んだ日）が merge される前の版では、全体の判定は未了で、初期投入の流れ 3 へ進まない（他の項目が全部合っても）。全体の合格は、「前提の lane の後に足す」のままの行が無く、行う行が全部 PASS のときだけ。月の帳票との照合（UI-09b-D9）の行は、O2 (ii) の店の事実が未確認の間は「行わない（O2 (ii) 未確認）」、当たらないと決まったら「行わない（当たらない）」と書き、合否に数えない。合わなかった項目を控え、直す lane を起こす。紙の切替えの前の照合（UI-09a-D19 の (2)）はこの受入と別に残す（TD-560）。

#### REQ-401 第2スライス表示詳細

- `DailySalesPage` は `SummaryCardsBar` の下に「レジ日報（公式）」セクションを表示する。
- `official_daily_report === null` の場合は「この日付のレジ日報は未取込みです。」の軽量 note を表示し、画面全体 error や大きな EmptyState にはしない。
- `official_daily_report` がある場合は、総売上 / 純売上、支払集計、部門別集計を表示する。支払行の `amount` は nullable のため「未取得」を許容し、部門行の `amount` は必須金額として表示する。
- `official_daily_report.source_import_count` が1以上なら、公式セクションに `N回の取込みを合算` と表示する（N が 2 以上の日の文は UI-09a-D16 のとおり直す）。複数parentのgross/netまたは明細optional値がNULL安全側へ伝播した場合も「未取得」を維持し、0として見せない。
- `official_daily_report.warnings` が非空の場合は、公式日報セクション内に warning トーンの注記（アイコン + テキスト）を表示する。注記の見出しは `日報の確認が必要です` にする（部門未対応と支払集計〈UI-09a-D16 の支払集計の段〉の warning を同じ注記に並べるため。D-120。前の `日報の部門確認が必要です` を置き換えた）。上部 Alert 帯は取得失敗やデータ安全系状態に限定し、部門未対応 warning とは混ぜない。
- official があり `items.length === 0` の場合、商品別明細セクションは「商品別明細は未取込み」と表示する。「売上なし」と誤読される文言は使わない。

`UI-09a-D15`: 同日の複数active importは日報サマリ、商品別明細ともBIZ-05の加算済み結果を表示する（支払集計を足さない日は UI-09a-D16 の支払集計の段、D-120）。UIで最新1回へ絞らず、公式日報seriesと商品別seriesを互いに足さない。rollback後のrefetchでは残存active importのaggregateへ直ちに収束する。日計（Z001）の行は UI-09a-D16 のとおり取込みごとに並べ、合算しない。

#### 日計（Z001）の表示

`UI-09a-D16`（Z001 の全行、[D-096](../decision-log.md#d-096-日計z001は同日の取込みごとに並べ合算しない2026-09-27)）: 公式セクションに Z001 の全行の表を置く。BIZ-05 の `official_daily_report.summary_imports`（[34 §19.2](34-biz-sales-service.md#192-型定義)）をそのまま描き、UI で合算・並べ替え・行の絞り込みをしない。

- 置き場所: `official_daily_report` がある日だけ、総売上 / 純売上の metric の下、支払集計・部門別集計の 2 表の上に、幅いっぱいで置く。未取込みの日（`official_daily_report === null`）は既存の note だけで、日計の表も見出しも出さない。
- 分岐: 1 回の日と 2 回以上の日は `official_daily_report.source_import_count`（= `summary_imports` の件数、34 §19.2）で分ける。表と文で同じ値を使う。
- 文字列（完全一致。下の `{N}` は `toLocaleString("ja-JP")` の数、`{日時}` は取込み日時。括弧は全角、`/` は半角、「—」は U+2014）:

| 使う場所 | 要素 | 文字列 |
|---|---|---|
| Z001 の表の見出し | h3（既存 2 表の見出しと同じ段） | `日計（Z001）` |
| 既存の支払の表の見出し | h3 | `支払集計（Z002）` |
| 既存の部門の表の見出し | h3 | `部門別集計（Z005）` |
| Z001 の表の列見出し | th | `名称` / `個数/件数` / `金額` |
| 1 回の日の文 | 既存の `p` | `{N}回の取込みを合算`（既存のまま。例 `1回の取込みを合算`） |
| 2 回以上の日の文 | 既存の `p` の文を置き換える | `総売上・純売上・支払集計・部門別集計は{N}回の取込みを合算しています（注記で足していないと示した表を除く）。日計（Z001）は取込みごとに表示します。`（括弧の部分は D-120 で足した。下の支払集計の段） |
| 2 回以上の日の取込みごとの見出し | h4（`日計（Z001）` の h3 の下） | `{N}回目の取込み（取込み日時 {日時}）`。`{日時}` は `YYYY-MM-DD HH:mm`（例 `1回目の取込み（取込み日時 2026-03-21 18:05）`） |
| 行の無い取込み | `p` | `この取込みの日計（Z001）の行はありません。` |
| 値の無い欄 | td | `—` |

- 見出し: `日計（Z001）` の h3 を表の直前に置く。既存 2 表の見出しは上の表の文字列に変える（語は変えず出どころを添えるだけ。Z002 / Z005 の行・合算・列は変えない）。runtime lane の L3 は、これらの文字列で店主が読み違えないかの確認に限り、文字列を変えるなら Gated Amendment で本表を直す。
- 列: 名称 / 個数/件数 / 金額 の 3 列（既存 2 表と同じ並び、数と金額は右寄せ）。見出しの「個数/件数」はレジの帳票の見出しに合わせる。
- 行: 各取込みの `lines` を返された順（`sort_order` の順）に全行出す。値が 0 の行も出す。名称は保存された `label` をそのまま出し、アプリで言い換えない。
- 値: 個数/件数の欄は `quantity` があればそれを、無ければ `count` を出す（Z001 の行はどちらか一方だけを持つ。総売の行は `quantity` だけ）。数は `toLocaleString("ja-JP")` で単位の文字を付けない。金額は既存と同じ `¥` + `toLocaleString("ja-JP")`。両方 NULL の個数/件数と、NULL の金額は `—` とする。`—` は「その行にその値が無い」を表す（既存の支払集計でも欠けた件数は `—`）。合算で欠けた金額の「未取得」（metric と支払集計の金額）とは別の表示である。負の数はそのまま負数で出し、金額は `¥-1,234`、件数は `-1` とする（同じ section の支払集計・部門別集計の金額の書き方〈`DailySalesPage.tsx` の `formatMoney`〉と、商品別明細の返品行の負数表示と同じ。前日比のカードの `-¥…` とは揃えない、owner 決定 2026-10-06）。
- 1 回の取込みの日（`source_import_count` が 1）: 表 1 つ。取込みごとの見出しは付けない。文は既存のまま。
- 同じ日に 2 回以上取り込んだ日（`source_import_count` が 2 以上）: `日計（Z001）` の h3 の下に、取込みごとの表を返された順（取込みの古い順）に並べ、各表の直前に取込みごとの h4 を置く。`{N}` はその日に表示中の取込みの中での古い順の番号で、レジの精算回数ではない（取り消した取込みは数えず、残りで振り直す）。`{日時}` は `imported_at` の秒を落として `T` を空白にした形（見出しとして読みやすさを優先する。秒までの取込み日時は、同じ日に追加で取り込むときの確認の dialog の既存分の表〈UI-07-D13〉で見える）。合算した表は出さない。文は上の表の 2 回以上の日の文にする。
- 行が 0 件の取込み: 取込みの経路では作られない（完了した取込みは総売か純売の行を必ず持つ）が、`lines` が空なら、その取込みの位置（2 回以上の日は h4 の下、1 回の日は h3 の下）に表の代わりに上の表の文を出し、取込み自体は落とさない。
- 変えないもの: 総売上 / 純売上の metric、部門別集計の合算と行、未取込みの note、部門未対応の warning の文、取得失敗の上部 Alert と再試行、商品別明細との分離（UI-09a-D12）、UI-09a-D15 の合算表示、前日の取得（`SummaryCardsBar` の前日比は日計を使わない）、日次 CSV 出力の列。
- 支払集計の合算は、日報の紙と同じ規則（[34](34-biz-sales-service.md) SALES-SHEET-D2）で足す（D-120）。今と同じ行と値になる日の範囲と、範囲の外の日の値は 34 §19.3 手順 3 が持つ。精算の間で名称か並びが変わった日は、位置と名称ごとの行に分かれる（D2 の 2）。そのうえ 1 回の精算の中に同じ名称の行が 2 行以上ある日は、足さずに取込みの古い順に全行を並べる（D2 の 3。表の行と列は変えない。取込みの区切りは画面の表に出さず、3 の日の文が紙〈`日報を印刷`〉で取込みごとに見られると案内する）。2 の日と 3 の日は、BIZ-05 の `warnings` のその日の文（34 §19.3 手順 3。3 の日は 3 の文だけ）を既存の warning の注記として出し、使い手に黙って通さない。この日にも事実と合うように、上の表の 2 回以上の日の文は足していない表を除くと書き、warning の注記の見出しは部門に限らない語にする（REQ-401 第2スライス表示詳細）。同じ `payment_key` の行が 2 つ返りうるので、支払集計の表の行の key は `${payment_key}-${index}`（index は返された配列の位置。部門別集計の行の key と同じ形）にする。wire の `payment_key` の意味は変えない。

#### 日報の紙（印刷、D-120）

`UI-09a-D17`（1 日 1 枚の紙）: 日次売上の `ExportBar` の印刷ボタンで、その日のレジの日報を 1 枚の紙に印刷する。内容は BIZ-05 の `get_daily_report_sheet`（[34 §19.3a](34-biz-sales-service.md)、SALES-SHEET-D1〜D6）をそのまま描き、UI で足し算・並べ替え・行の絞り込みをしない。今の Excel の横長の並び（4 つの帳票の枠を横に並べ、縮小して 1 枚に入れる形）は写さない（owner の決定、repo 外の回答台帳 TD-296 の 15）。

- 印刷ボタン: 文字は `日報を印刷`。紙の内容は日次売上の画面を開いたときに日付ごとに読み（`commands.getDailyReportSheet(date)`、query key は `queryKeys` に足す `["daily-sales", "sheet", { date }]`。既存の `queryKeys.dailySalesRoot()` の prefix の invalidate〈日報・Z004 の取込みと取消の成功〉に乗る）。押したときの順は次に固定する（SALES-SHEET-D6）: (1) `await refetch()` で読み直す、(2) 読み直しが失敗したか `import_count` が 0 なら印刷せず、下の表のその状態の文を出して終わる、(3) 読み直した値と、押した時刻の `印刷 {YYYY-MM-DD HH:mm}` で紙の内容を描き直し、その描画が DOM に反映された後に、(4) `window.print()` を呼ぶ。読み直しの解決の前と描画の前には印刷しない。押せない状態と文（ボタンの下に文で出す。Tooltip だけにしない）:

| 状態 | ボタン | ボタンの下の文（完全一致） |
|---|---|---|
| 読込み中 | disabled（`aria-disabled`） | なし |
| `import_count` が 0 | disabled（`aria-disabled`） | `この日のレジ日報は未取込みのため、印刷できません。` と、その場から取り込めるリンク `売上データ取込みを開く`（`/csv-import`。既存の売上データ取込みの画面） |
| 読込みの失敗 | disabled（`aria-disabled`） | 既存の CmdError の日本語の文（[UI_TECH_STACK §6.4](../UI_TECH_STACK.md)）と、既存と同じ `再試行` ボタン（紙の内容の query の `refetch()`。§56.8 の当日の取得失敗の `再試行` と同じ形）。日付を選び直しても読み直す |
| `import_count` が 1 以上 | 押せる | なし。ただし `plu_import.skipped_count` が 1 以上なら、紙の上の知らせ（下の表）と同じ `数えていない PLU の行が {N} 行あります` を画面にも出し、理由と直し方を見られるリンク `取込みの記録を開く`（入出庫履歴 `/inventory/records` を search `recordType: "csv_import"`・`status: "active"`・`dateFrom` と `dateTo` にその日〈`YYYY-MM-DD`〉で開く。Z004 の取込みの記録はこの一覧にあり、`csv_import` の `dateFrom`/`dateTo` は `csv_imports.settlement_date` で絞り、`active` は取消していない取込みで、`plu_import` が数える取込み〈24 §14.21b〉と同じ集合になる。一覧の行から、その取込みの記録〈行ごとの誤りの理由〉へ進める）を添える。印刷は止めない（日報の部分は正しく、PLU の部分に数えていない行があることを紙と画面の両方で示す。owner の設計の原則〈repo 外の回答台帳 TD-310〉: 黙って過ぎさせない）。`label_changed_sources` が空でないときも、紙の上の知らせ（下の表の帳票の 2 行。どちらの文かも同じ規則）と同じ文を、帳票ごとに 1 行、`{帳票の見出し}：{文}`（見出しは下の紙の並びの語。例 `支払集計（Z002）：この日の精算の間で…`。`：` は全角）の形で画面に出す（印刷の前に分かる。Z002 は同じ日に公式セクションの warning の注記にも支払集計の文が出る〈UI-09a-D16 の支払集計の段〉。文の正本は紙の知らせが下の表、注記が 34 §19.3 手順 3 で、別々に持つ） |

- 紙の並び（既定。owner が実機の印刷で並べた案から選ぶまでこの形。D-120 の Revisit）: A4 縦、余白は上下左右 10 mm。上に題 `売上日報`、日付 `{YYYY}年{M}月{D}日（{曜}）`（`report_date`。曜は `月`〜`日` の 1 字）、取込みが 2 回以上の日は `{N}回の取込みを合算`（`unsummed_sources` が空でない日は `{N}回の取込みを合算（取込みごとに載せた帳票を除く）`）、右上に `印刷 {YYYY-MM-DD HH:mm}`（印刷した時刻。後から取込みを直した紙と見分ける）。その下を 2 段に分け、左の段に `日計（Z001）`、その下に `部門別集計（Z005）`、右の段に `支払集計（Z002）` を置く。2 段の下に幅いっぱいで `PLU の売上（Z004）` を置く。帳票の見出しは日次売上の画面の見出し（UI-09a-D16）と同じ語。
- 表: Z001・Z002・Z005 の表の列は `名称` / `個数/件数` / `金額`。PLU の表の列は `商品名` / `数量` / `金額`。数と金額は右寄せ。行は BIZ-05 が返した順に全部出す。値の書き方は日次売上の画面の日計（UI-09a-D16）と同じ（個数/件数の欄は `quantity` があればそれ、無ければ `count`、`toLocaleString("ja-JP")`、無い値は `—`）。ただし金額に `¥` を付けない（列の見出しで分かり、桁の多い列を狭くする。負の数は `-1,234`）。名称が空の行の名称は `（名前なし）`。`unsummed_sources` にある帳票（SALES-SHEET-D2 の 3）は、行の `import_seq` ごとに表を分けて `import_seq` の順に並べ、各表の見出しの行（下の「次の紙への続け方」）の帳票の見出しの後に `　{N}回目の取込み`（`{N}` は `import_seq`）を足す（列と値の書き方は同じ）。PLU の数量は商品別明細と同じ表示の規則（[共通規則](10-common-rules.md) SPEC-UNIT-D3）。
- 紙の上の知らせ（完全一致。該当する帳票の最初の表の見出しの行の直下〈表の本体の最初の行。列をつないだ 1 つの欄〉に 1 行。表を出さないときは、見出しの行と同じ文字列の 1 行の下に出す）:

| 条件 | 文 |
|---|---|
| その帳票が `label_changed_sources` にあり、`unsummed_sources` に無い（SALES-SHEET-D2 の 2） | `この日の精算の間で、帳票の行の名称か並びが変わっています。同じ名称の行が 2 行あれば、別々に足しています。` |
| その帳票が `unsummed_sources` にある（SALES-SHEET-D2 の 3。上の行の文は出さない） | `この日の精算の間で、帳票の行の名称か並びが変わり、1 回の精算の中に同じ名称の行が 2 行以上あります。別の項目を足さないよう、この帳票は足さずに取込みごとに載せています。` |
| `plu_import.import_count` が 0 | `この日の PLU の売上は取り込まれていません。`（表を出さない） |
| `plu_import.import_count` が 1 以上で `plu_rows` が空 | `この日に売れた PLU の商品はありません。`（表を出さない） |
| `plu_import.skipped_count` が 1 以上 | `数えていない PLU の行が {N} 行あります（入出庫履歴の CSV取込みの記録で見られます）。` |

- 文字の大きさの下限: 紙の上の全部の文字を 9 pt 以上にする（見出しは 11 pt 以上の太字）。収まらないときは文字を小さくせず、次の紙へ続ける。「1 日 1 枚」は精算ごとに紙を分けないことを指し、行が多い日の 2 枚目を禁じない（店主の答え「PLU が入って縦に印刷されると文字が小さすぎて見えない」、TD-279）。
- 次の紙への続け方（既定）: 各表の `thead` を 2 行にし、1 行目（見出しの行。11 pt 以上の太字）に `{帳票の見出し}　{YYYY}年{M}月{D}日（{曜}）`、2 行目に列の見出しを置く（帳票の見出しを別の要素に置かず、この 1 行目が見出しを兼ねる。`　` は全角の空白）。表が紙をまたぐと、この 2 行が次の紙の上に繰り返し出る（`thead` の `display: table-header-group`。CSS だけで作り、紙の高さを測る処理を持たない）。これが `（続き）` の代わりで、2 枚目以降の紙にも日付と帳票の名前が出る（`（続き）` の語は出さない。1 枚目と 2 枚目で見出しの行を変えるには紙の高さを測る処理が要るため）。行は途中で割らない（`tr { break-inside: avoid }`）。2 段（左に日計と部門、右に支払）が 1 枚目に収まらない日は、左の段の続きは次の紙の左の段へ、右の段の続きは次の紙の右の段へ続け、2 段が両方終わった後に PLU の表を始める。題・日付・印刷の時刻の行は 1 枚目の上だけに出す。
- 1 枚に収まるかの見積り（未実測、起草役の計算。前提: 本文 9 pt で行の高さ 1.4 倍〈約 4.4 mm〉、見出し 11 pt、罫線の太さと表の間の余白は数えない）: 印刷できる高さは 277 mm（A4 縦 297 mm − 上下の余白 10 mm ずつ）。上の題と日付の行で約 13 mm、左の段（日計 28 行・部門 21 行と、表ごとの見出しの 2 行）で約 238 mm、右の段（支払 38 行と見出しの 2 行）で約 179 mm。左の段の後に残る約 27 mm に PLU の見出しの 2 行と 3 行ほどが入る。1 行が 5 mm（罫線と上下の余白を足した高さ）なら、左の段だけで約 266 mm になり、題の行と合わせて 1 枚目を越える。PLU の行のある日は 2 枚目へ続く見込みが高く、runtime の lane の L3 で実機の印刷で確かめる（packet の Contract Probe の P5）。
- 色: 黒の文字と細い罫線だけにし、色を使わない（色で伝える意味が無い。負の数も色でなく `-` で示す）。
- 画面での見え方: 紙の内容は画面には出さず、印刷のときだけ出す（`@media print`、UI-09a-D18）。印刷の前の見た目は印刷の dialog のプレビューで見る。

`UI-09a-D18`（印刷の手段）: WebView の印刷（`window.print()`）を使い、`@page { size: A4 portrait; margin: 10mm }` と `@media print` で紙の内容だけを出す（アプリの枠・サイドバー・画面の他の部分を印刷しない）。`日報を印刷` を押さずに印刷の操作（Ctrl+P 等。どの画面でも）をしたときは、白紙の代わりに `この画面の印刷には対応していません。日報は日次売上の画面の「日報を印刷」から印刷してください。` の 1 行を紙に出す（アプリの root の外に置き、画面には出さず印刷のときだけ出す。紙の内容を描いている間は描かず、日報の紙と同じ印刷に出さない。D-127）。印刷先・枚数・用紙は OS の印刷の dialog で選ぶ（アプリにプリンタの設定の画面を作らない）。Windows の WebView2 でこの手段が dialog を出し、A4 縦と余白の指定が効くことは未確認の前提で、runtime の lane が実装の後の L3（packet の Matrix の L3-1。Contract Probe の P1）で確かめる（code が小さく、Windows の build と owner の操作を L3 の 1 回にまとめる。2026-10-29、runtime の lane の Plan Review round 1）。効かなければ、Rust 側の CMD が同じ紙の内容を 1 つの HTML の file にしてアプリの data の folder の固定の file（毎回上書き。どの印刷かは紙の中の印刷の時刻で見分ける）に書き、導入済みの `tauri-plugin-opener` の Rust の API `open_path`（`with` を渡さない）で、その PC で `.html` に関連付けられた既定のアプリで開いて、そこで印刷する（新しい依存も capability も足さない。frontend の `opener:default` は local の file を開く許可を持たないので、frontend からは開かない）。`open_path` はブラウザを指定する API ではなく、file の関連付けの先のアプリで開く（関連付けがエディタなら HTML のソースが開く）ので、この道を採る条件は、店の PC の `.html` の関連付けが印刷できるブラウザであることを runtime の lane が Contract Probe で確かめること（確かめられなければこの道を採らない）。HTML に入れる値（名称・商品名・数）は全部 HTML escape する。書込みか `open_path` が失敗したら CMD は error を返し、UI は成功と扱わず既存の CmdError の文を出す。この道を採るときは、[42](42-cmd-sales-stocktake.md) に command（名前・引数・返り値・§22.8 の登録）を足す Gated Amendment を出す（今は wire を決めない）。棄却案: PDF を作る（PDF を作る依存を足す）、Excel の file を出す（店が開いて印刷する手順が残り、手で写す作業を無くす目的に合わない）、OS の印刷の API を Rust から呼ぶ（OS ごとの code が要る）。

`UI-09a-D19`（紙の切替えの時期）: 新しい紙へ切り替える日までは、店は今の Excel への貼付けと印刷をそのまま続ける（アプリの紙は比べるときだけ印刷する）。切り替える前に、(1) runtime の lane の L3 で owner が紙の並びを実機の印刷で比べて決め、(2) 店のアプリの運用が始まった後の実際の 1 日（同じ日に 2 回以上精算した日があればその日も）で、アプリの紙と今の Excel の紙を横に並べて値が同じことを確かめる（今の Excel の紙はレジスターツールの日報を貼ったものなので、紙の値がレジスターツールの日報と行ごとに同じかという D-120 の Guarantee range の未確認の前提も、ここで確かめる。D-142 の (5)。運用開始の前の UI-09a-D13 の導入前受入とは別。順は D13 の受入 → 運用開始 → この確かめ）。切り替える日は月の初めか元日にする（店主の答え: 並びを変えるなら月初めや元旦からのように時期を選ぶ、TD-279）。既定は (2) の後の最初の月の初めで、それが 12 月（年末の棚卸しの月）にかかるなら元日。どちらにするかは owner が店と決める。切り替えた日から、店は毎日の取込みの後に `日報を印刷` で印刷して綴じ、Excel への貼付けをやめる。前の綴りはそのまま残す。切り替えた後に紙に不都合が見つかれば、翌日から Excel の貼付けに戻せる（アプリの側は何も変えない。紙はいつでも過去の日の分を印刷し直せる）。

**ファイル構成（18 file = lib 7 + hooks 2 + components 6 + page 1 + types 1 + route 1）**:

| ファイル | 責務 | 行数目安 |
|---|---|---|
| `src/routes/reports/daily.tsx` | TanStack Router file route + zod 直接渡し + `<DailySalesPage />` mount + `SearchParams` 型 export | 25-35 |
| `src/features/daily-sales/DailySalesPage.tsx` | 最上位レイアウト | 110-140 |
| `src/features/daily-sales/types.ts` | `SortColumn` (5 列、`unit_price` 含む) / `SortDirection` / `GroupedSection` / `SalesLineSummary` / `DepartmentOption` | 40-60 |
| `src/features/daily-sales/lib/sort-items.ts` | `sortDailyItems` 純関数（5 列対応、`unit_price` で `calculateEffectiveUnitPrice` 計算 + null 末尾） | 40-55 |
| `src/features/daily-sales/lib/group-items.ts` | `groupItemsByDepartment` 純関数 | 40-60 |
| `src/features/daily-sales/lib/filter-items.ts` | `filterItemsByDepartment` 純関数 | 15-25 |
| `src/features/daily-sales/lib/compute-summary.ts` | `computeSalesLineSummary` 純関数（BIZ-05 で source 別集計未提供のため UI 派生） | 25-40 |
| `src/features/daily-sales/lib/calculate-unit-price.ts` | `calculateEffectiveUnitPrice` 純関数（user Option 1.5 採用、`Math.round(abs(amount)/abs(quantity))`） | 20-30 |
| `src/features/daily-sales/lib/date-nav.ts` | `addDays` / `formatJpDate` / `useTodayDate()` hook（daily-sales 内閉鎖、UI-00 `useYesterdayDate` 同型） | 50-75 |
| `src/features/daily-sales/lib/test-fixtures.ts` | `makeMockItem(overrides: Partial<DailySaleItem>): DailySaleItem` factory | 20-30 |
| `src/features/daily-sales/hooks/useDailySalesReport.ts` | 2 useQuery + 派生 5 純関数 orchestration + `derived.departmentOptions` 生成 | 100-140 |
| `src/features/daily-sales/hooks/useExportDailySalesCsv.ts` | useMutation + Blob ダウンロード + `setTimeout(100)` で `revokeObjectURL` + Sonner id-based dedup（UI-09b PR #66 で `useExportFile` 共通化 + 本 hook は wrapper 化 = 20-30 行に縮退） | 80-110 |
| `src/features/daily-sales/components/DateNavigator.tsx` | 前日 button + `<input type="date">` + 翌日 button | 60-80 |
| `src/components/sales/TabsHeader.tsx` | 日次（`/reports/daily`）+ 月次（`/reports/monthly`）切替（router-driven `<Link>` 視覚表現、UI-09b PR #66 で共通化 + daily/monthly 共有） | 45-60 |
| `src/components/patterns/DepartmentFilter.tsx` | shadcn Select 単一選択、hook 側 `derived.departmentOptions` を pure に描画（PR-B で 3 feature のローカル実装を統合、allLabel 既定「すべての部門」、[59-ui-shared-patterns.md](59-ui-shared-patterns.md)） | 50-70 |
| `src/features/daily-sales/components/SummaryCardsBar.tsx` | 4 カード（売上合計 / 販売点数 / 売上明細数 + Tooltip / 前日比） | 90-130 |
| `src/features/daily-sales/components/ProductTable.tsx` | shadcn Table 6 列 + 列ソート（5 列対応）+ 部門小計行 + 手動バッジ + 単価列 null = 「—」placeholder | 140-180 |
| `src/features/daily-sales/components/ExportBar.tsx` | CSV 出力 button（active）+ `日報を印刷`（押せない理由をボタンの下に文で出す、押したときに読み直してから紙を描いて印刷する。印刷する間だけ `DailyReportSheet` を描く。UI-09a-D17） | 140-170 |
| `src/features/daily-sales/components/DailyReportSheet.tsx` | 日報の紙（`document.body` の直下へ portal、画面では隠し印刷のときだけ出す。UI-09a-D17・D18） | 200-230 |
| `src/features/daily-sales/components/PrintUnsupportedNotice.tsx` | Ctrl+P の案内の 1 行（`document.body` の直下へ portal、印刷のときだけ出す。紙を描く間は描かない。`RootLayout` に置く。UI-09a-D18） | 30-45 |
| `src/features/daily-sales/hooks/useDailyReportSheet.ts` | `commands.getDailyReportSheet(date)` の useQuery（key `queryKeys.dailyReportSheet(date)`） | 15-25 |

合計 **21 file**。frontend code 1000-1300 行 + 関数設計 500-600 行 + test 270-340 行。

---

### 56.2 ファイル構成（命名規約 + 型 export）

**snake_case 維持 (specta-typescript デフォルト)**:

- Rust 側 BIZ-05 DTO（`DailySaleItem` / `DailySalesReport` / `DeptSubtotal` / `GrandTotal`）は specta-typescript のデフォルト設定で **snake_case のまま** `src/lib/bindings.ts` 経由で TypeScript 側に flow する（serde rename 等は未付与）。camelCase 変換が必要になったら別 PR で specta 設定を検討
- `DailySaleItem` の 8 field（`product_code` / `name` / `department_name` / `department_id` / `stock_unit` / `quantity` / `amount` / `source`）は snake_case のまま生成され、`source` は `DailySaleSource`（`"auto" | "manual"`）、`stock_unit` は `ProductStockUnit` の literal union で型検査される（[共通規則](10-common-rules.md) SPEC-UNIT-D3・D11、§56.10 の「単位と数量」）

**`SearchParams` 型 export**（`src/routes/reports/daily.tsx`）:

```ts
export type SearchParams = z.output<typeof searchSchema>;
```

- `z.output` を使うのは zod 4 で `.optional()` + `.catch()` の挙動が input/output で差異あるため、TanStack Router の `Route.useSearch()` の型推論と整合させる
- 各 component / hook で `SearchParams` を import せず、`Route.useSearch()` 経由で型推論を受ける（DRY、本 repo 初の validateSearch 採用パターン）

**source enum 化（D-061 (e)）**:

- `source: DailySaleSource` generated literal union を使用し、`computeSalesLineSummary` は `switch` + `never` で全 variant を網羅する
- 第 3 値（例: `"adjustment"`）を追加した場合、Rust enum と frontend 分岐の双方が未追随なら typecheck で検出される

---

### 56.3 データフロー

```
URL search params (?date=...&dept=...&sortBy=...&sortDir=...)
  ↓ TanStack Router validateSearch（zod 4 直接渡し）
  ↓ Route.useSearch() = SearchParams
DailySalesPage
  ↓ useDailySalesReport({ date, dept, sortBy, sortDir })
    ↓ today useQuery: commands.getDailySales(date)
    ↓ yesterday useQuery: commands.getDailySales(yesterday(date))
    ↓ 派生 5 純関数 orchestration:
      ├ filterItemsByDepartment(items, dept)
      ├ sortDailyItems(filtered, sortBy, sortDir)
      ├ groupItemsByDepartment(sorted)
      ├ computeSalesLineSummary(items)
      └ derived.departmentOptions = unique by department_id, sort by id
  ↓ 戻り値:
    { today, yesterday, derived: { grouped, summary, departmentOptions, yesterdayDate } }
    today / yesterday は独立した UseQueryResult。loading / error / 部分失敗はpageで判定
SummaryCardsBar / ProductTable / DepartmentFilter / DateNavigator / TabsHeader / ExportBar
  ↓ useExportDailySalesCsv() → commands.exportSalesCsv("daily", date) → Blob ダウンロード
```

**派生計算の責務**:

- フィルタ / ソート / グルーピング / 集計 / 単価計算 はすべて hook 内で実行、components は pure（props を受け取って描画するだけ）
- `departmentOptions` 生成も hook 側責務（DepartmentFilter は pure component）

---

### 56.4 URL state 設計（zod 4 直接渡し、sortBy 5 列対応）

```ts
// src/routes/reports/daily.tsx
import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { DailySalesPage } from "@/features/daily-sales/DailySalesPage";

const searchSchema = z.object({
  date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/).optional().catch(undefined),
  dept: z.coerce.number().int().positive().optional().catch(undefined),
  sortBy: z.enum(["product_code", "name", "quantity", "unit_price", "amount"]).optional().catch(undefined),
  sortDir: z.enum(["asc", "desc"]).optional().catch(undefined),
});

export type SearchParams = z.output<typeof searchSchema>;

export const Route = createFileRoute("/reports/daily")({
  validateSearch: searchSchema,
  component: DailySalesPage,
});
```

**デフォルト値とフォールバック**:

| param | 未指定時 | 不正値時 |
|---|---|---|
| `date` | 当日（`useTodayDate()` から取得） | undefined fallback → 当日 |
| `dept` | `null` = すべて | undefined fallback → すべて |
| `sortBy` | `null` = ソートなし（BIZ 層の `department_id` 昇順を維持） | undefined fallback → ソートなし |
| `sortDir` | `"asc"`（sortBy 指定時のみ意味あり） | undefined fallback |

**不正値の挙動**:

- `?date=invalid` / `?dept=abc` / `?sortBy=garbage` / `?sortBy=department`（enum 5 値外で一見有効）→ `.optional().catch(undefined)` で各 field 単位に undefined fallback
- schema 全体は throw しない、画面は当日 + 全部門 + ソートなしで描画
- F5 耐性: URL を bookmark / 再読込しても同じ状態が復元される

**search params 部分更新**（TanStack Router 関数形式で他 param 温存）:

```ts
const search = Route.useSearch();  // 型推論 = SearchParams
const navigate = Route.useNavigate();

const handleDateChange = (newDate: string) => {
  navigate({ search: (prev) => ({ ...prev, date: newDate }) });
};

const handleSortChange = (column: SortColumn) => {
  navigate({
    search: (prev) => ({
      ...prev,
      sortBy: column,
      sortDir: prev.sortBy === column && prev.sortDir === "asc" ? "desc" : "asc",
    }),
  });
};
```

本 repo 初の `validateSearch` 採用（`rg -n "validateSearch" src/` → 0 件、PR #65 で確立）。

**関連 memory**:

- `feedback-desktop-app-url-design.md`: URL 状態化原則（テスト容易・F5 耐性・queryKey 独立・コード分割）
- `zod-4`: v4 では `z.coerce.number()` のまま、`z.enum()` の breaking change は string literal 配列のみ受理（本 plan 準拠済）

---

### 56.5 hook 設計

#### useDailySalesReport

```ts
type Args = {
  date: string;          // YYYY-MM-DD
  dept: number | null;   // null = すべて
  sortBy: SortColumn | null;
  sortDir: SortDirection;
};

type Result = {
  todaySales: DailySalesReport | undefined;
  yesterdaySales: DailySalesReport | undefined;
  grouped: GroupedSection[];        // 部門小計挿入済み
  summary: SalesLineSummary;        // 売上明細数 = items.length + 自動/手動内訳
  departmentOptions: DepartmentOption[];
  isLoading: boolean;
  error: Error | null;              // 当日 fail = 画面全体 Alert
  partialError: { yesterday: Error | null };  // 部分障害（前日比カードのみ）
};

export function useDailySalesReport(args: Args): Result;
```

**実装方針**:

- `today useQuery`: `queryKeys.dailySales(args.date)` / `staleTime: 5 * 60_000`（UI-00 と統一）/ `gcTime: 10 * 60_000`
- `yesterday useQuery`: `queryKeys.dailySales(addDays(args.date, -1))` / 同 staleTime、**`enabled: true` 常時**（UI-00 部分障害許容パターン `53-ui-home.md §53.4 D-3` と整合）
- 本 query を stale 化する mutation 集合は [D-052](../decision-log.md) と `src/lib/invalidation-contract.ts` を正本とし、本書へ producer 一覧を複製しない。`dailySalesRoot()` は日付別 key の prefix factory とする。
- 派生計算は `useQuery` 完了後に同期実行（5 純関数を順次適用、React Compiler 自動 memoize）
- `partialError.yesterday` を SummaryCardsBar に渡し、前日比カードのみ「比較データなし」描画（他 3 カード + テーブルは正常）

**staleTime 統合の根拠**:

- UI-00 が `queryKeys.dailySales(yesterday)` を 5min staleTime で購読、本 hook も同 queryKey + 同 staleTime で重複しても問題なし
- 動的 staleTime（当日 30sec / 過去日 24h）は同 queryKey 異 staleTime の anti-pattern、却下（§9 代替案）

#### useExportDailySalesCsv

```ts
type Args = {
  date: string;
};

type Result = {
  exportCsv: (args: Args) => void;
  isExporting: boolean;
};

export function useExportDailySalesCsv(): Result;
```

**実装方針**:

- UI-09b PR #66 で `useExportFile({ reportType, target })` 共通化、`useExportDailySalesCsv` は `exportFile({ reportType: "daily", target: date })` wrapper に縮退（80-110 行 → 20-30 行）
- queryKey なし（副作用なし、cache invalidate しない）
- `onSuccess`: base64 → Uint8Array → Blob → `URL.createObjectURL` → `<a download>` click → `setTimeout(() => URL.revokeObjectURL(url), 100)`（共通化先 `useExportFile` に集約）
- `onSuccess` Sonner: `toast.success("日次売上 を保存しました（${n} 件）", { id: "export-daily-success" })` ← id-based dedup（連続クリックでも重複トースト 0、PR #66 で `-csv-` セグメント削除 + reportType ラベル化）
- `onError` Sonner: `toast.error("出力に失敗しました: ${message}", { id: "export-daily-error" })`

**Blob memory 安全性の 3 罠**（実装時に必ず踏襲）:

1. **revokeObjectURL タイミング**: `setTimeout(() => URL.revokeObjectURL(url), 100)` で Chromium download 中断回避（即時 revoke すると download が中断するケースあり）
2. **IPC payload size**: `bytes_base64` は 1 日 sale_records 最大想定 1000 件 × ~200 byte = ~200KB、Tauri 2 IPC payload 上限（WebView2 256MB 公称、現実 50MB degrade）内、安全マージン十分
3. **base64 改行混入**: Rust 側 `cmd::sales_cmd::export_sales_csv` で `base64::engine::general_purpose::STANDARD.encode` 使用 = 改行なし pure base64 → UI 側 `atob(bytes_base64)` 直 decode 安全。`MIME` engine のみ改行混入リスクあり、本実装の `STANDARD` engine は安全（`.replace(/\s/g, '')` 前処理不要）

#### useTodayDate（daily-sales 内閉鎖）

- `src/features/daily-sales/lib/date-nav.ts` 内に `useTodayDate(): string` を `useYesterdayDate.ts` と同型で実装（`toLocaleDateString("sv-SE")` + Visibility API listener）
- UI-00 の `useYesterdayDate` との実装重複は 5-10 行レベルで許容（DRY 違反軽微）
- 将来 UI-09b / UI-06a 等で today/yesterday hooks を多用するようになったら `src/lib/dates/` に共通化（Backlog 候補）

---

### 56.6 純関数（テスト対象 6 個）

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `sortDailyItems` | `(items: DailySaleItem[], by: SortColumn \| null, dir: SortDirection) => DailySaleItem[]` | 5 列対応。`unit_price` で `calculateEffectiveUnitPrice` を計算してから数値比較。`null` 行は末尾配置（asc/desc 共通）。同値タイブレークは入力順保持（安定ソート） |
| `groupItemsByDepartment` | `(items: DailySaleItem[]) => GroupedSection[]` | 部門ごとに小計行を挿入。部門順は `department_id` 昇順固定（BIZ-05 順を尊重）。各 section は `{ departmentId, departmentName, items, subtotalQuantity, subtotalAmount }` |
| `filterItemsByDepartment` | `(items: DailySaleItem[], deptId: number \| null) => DailySaleItem[]` | `deptId === null` で恒等関数（items そのまま）。該当行なしの場合は空配列を返す |
| `computeSalesLineSummary` | `(items: DailySaleItem[]) => SalesLineSummary` | `{ total: items.length, autoCount: items.filter(i => i.source === "auto").length, manualCount: items.filter(i => i.source === "manual").length }`。BIZ-05 で source 別集計未提供のため UI 派生（将来 BIZ 拡張で削除可能） |
| `calculateEffectiveUnitPrice` | `(item: DailySaleItem) => number \| null` | `item.quantity === 0` → `null`、それ以外 → `Math.round(Math.abs(item.amount) / Math.abs(item.quantity))`。返品行（quantity<0, amount<0）も絶対値で「単価の大きさ」として正数表示（user Option 1.5 採用） |
| `addDays` | `(date: string, days: number) => string` | YYYY-MM-DD 形式の日付に N 日加算。月またぎ / 年またぎ / 閏年 2/29 を正しく処理（`new Date(date)` + `setDate(getDate()+days)` + `toLocaleDateString("sv-SE")`） |

**`calculateEffectiveUnitPrice` 仕様**（user Option 1.5 確定、§56.10 業務ルール参照）:

```ts
// src/features/daily-sales/lib/calculate-unit-price.ts

/**
 * 売上明細の実績単価を計算する純関数。
 * 商品マスタの販売単価ではなく、売上記録の金額 ÷ 数量で求めた派生値。
 * 返品行（quantity<0 + amount<0）では絶対値で「単価の大きさ」として表示。
 * quantity=0 の場合は null を返し、表示側で「—」placeholder。
 * 端数は四捨五入で整数円表示（Math.round）。
 */
export function calculateEffectiveUnitPrice(item: DailySaleItem): number | null {
  if (item.quantity === 0) return null;
  return Math.round(Math.abs(item.amount) / Math.abs(item.quantity));
}
```

**`computeSalesLineSummary` 根拠**（BIZ-05 で source 別集計未提供のため UI 派生）:

- BIZ-05 `DailySalesReport` の `GrandTotal` は `{ count_points, length_cm, amount }` のみ、source 別 count を返さない（数量は種類ごとの 2 つ。[共通規則](10-common-rules.md) SPEC-UNIT-D3・D11、§56.10 の「単位と数量」）
- UI で `items` 配列を再走査して `auto / manual` カウントを派生計算
- 将来 BIZ-05 を拡張して `GrandTotal.auto_count` / `manual_count` を追加すれば本関数は削除可能（Backlog 候補）
- `source`（bindings の `DailySaleSource`、`"auto"` / `"manual"`）を `switch` で分け（`src/features/daily-sales/lib/compute-summary.ts` の `computeSalesLineSummary`）、`default` の `never` で網羅を型で確かめる（値が増えれば compile error）

---

### 56.7 UI コンポーネント

**UI component 利用一覧**:

| component | 利用箇所 |
|---|---|
| `Card` | SummaryCardsBar（4 カード） |
| `Button` | DateNavigator（前日/翌日）、ExportBar（CSV出力/印刷） |
| `Select` | DepartmentFilter |
| `SegmentedControl` visual primitive | TabsHeader（日次 / 月次 切替、router-driven `<Link>` 視覚表現、UI-09b PR #66 で `src/components/sales/` 共通化）。二択切替の見た目は `src/components/ui/segmented-control.tsx` の list/item/active/inactive class を使い、active border は押しボタン状に見えない `border-stone-300` とする |
| `Table` | ProductTable（6 列） |
| `SortableHeader<T extends string>` | `src/components/sales/SortableHeader.tsx` を共通ownerとし、ProductTableの5列（`product_code` / `name` / `quantity` / `unit_price` / `amount`）で利用（UI-TABLE-D1） |
| `Tooltip` | SummaryCardsBar（売上明細数）。日次の ExportBar の印刷は Tooltip を使わず、押せない理由をボタンの下に文で出す（UI-09a-D17） |
| `Badge` | ProductTable（手動行の②分類の stone の「手動」バッジ、`variant="secondary"`） |
| `Alert` | DailySalesPage（当日 fail 時の画面全体 error） |
| `Skeleton` | DailySalesPage / ProductTable（loading 時） |

**SummaryCardsBar overflow 対策（PR #70 / PR-3）**:

- `Card` + `CardContent` に `min-w-0`、value div に `truncate` を付与する。monthly と同じく、grid item の `min-width:auto` と長い金額・期間ラベルによるカード溢れを止めるための CSS 対策。

**商品コード readability（2026-06-07 follow-up）**:

- H-6 Windows native 5 画面通し確認で「商品コードは小さい」と feedback があったため、日次売上 `ProductTable` の商品コードセルは `font-mono text-sm font-medium` とする。旧 `text-xs` は最小級で、全体 WebView 表示スケール導入後も単独で読みにくさが残るため使わない。

**DatePicker 非採用根拠**:

- shadcn Date Picker は dayjs 依存 + 7 追加 component
- `<input type="date">` ネイティブで十分（OS タイムゾーン / 日本語 / キーボード操作 / accessibility 全て揃う）
- WSL2 / Windows native 両環境で動作確認済（UI-00 で実証）

**部門小計行の表現**:

```tsx
<tr className="bg-stone-100 dark:bg-stone-800 font-medium">
  <td colSpan={3}>{departmentName}</td>
  <td className="text-right">{subtotalQuantity}</td>
  <td />{/* 単価列は空 */}
  <td className="text-right">¥{subtotalAmount.toLocaleString("ja-JP")}</td>
</tr>
```

- grey 帯 = stone palette warm tones（[../design-system/00-foundations.md](../design-system/00-foundations.md)「カラーパレット」ウォーム系採用の論拠）
- 単価列は空（小計に単価概念なし）

**単価列 null = 「—」placeholder**:

```tsx
<td className="text-right">{unitPrice === null ? "—" : `¥${unitPrice.toLocaleString("ja-JP")}`}</td>
```

- `quantity=0` 行は表示上「—」、ソート時末尾配置
- em-dash "—" を使用（hyphen "-" ではない、視覚的に「データなし」を明確化）

**Sonner dedup pattern**:

```ts
toast.success("日次売上 を保存しました（23 件）", { id: "export-daily-success" });
toast.error("出力に失敗しました: ロックされたファイル", { id: "export-daily-error" });
```

**PR #66 改名注記**: `export-daily-csv-success`/`export-daily-csv-error` は本 PR (UI-09a) 当時の id、UI-09b PR #66 で `useExportFile` 共通化に伴い `-csv-` セグメント削除 + reportType (`daily` / `monthly_by_product` / `monthly_by_department`) ラベル化に rename。同 id 内 dedup 動作は維持、内部 dedup key の実態整合性を優先（memory `feedback-naming-must-match-reality.md` 系の判断軸、外部公開 API ではないため後方互換 wrapper 不要）。

- `id` を固定すると、同 id の既存トーストが置換される（dismiss + 再表示ではない）
- 連続クリック時の重複トースト発火を防止（PR #56 P2-B horizontal pattern を継承）

---

### 56.8 エラー処理

**3 階層のエラー扱い**:

| エラー源 | 影響範囲 | 表示 |
|---|---|---|
| 当日 useQuery fail | 画面全体 | `<Alert variant="destructive">` + 「再試行」ボタン、サマリ + テーブル + エクスポート全て描画されない |
| 前日 useQuery fail | 前日比カードのみ | 「比較データなし」+ カード内 inline icon、他 3 カード + テーブル正常 |
| export useMutation fail | トースト | Sonner error トースト（id-based dedup）、画面状態は不変 |

**当日 fail = 画面全体 Alert** の根拠:

- 主要表示データが取得できない以上、サマリ・テーブル・エクスポート全て無意味
- UI-00 ホーム同型（部分障害許容パターン `53-ui-home.md §53.4`）

**前日 fail = 前日比カードのみ部分障害** の根拠:

- 前日データは「補助情報」、欠損しても当日業務には影響しない
- 当日 0 円のケース（前日比 = 比較不可）と同じ表現を再利用

**export fail = Sonner エラー** の根拠:

- ファイルロック / ディスク満杯 / permission denied 等は一過性、画面状態を壊さず通知のみ
- 失敗後も同 button 連打で再試行可能（id-based dedup）

**CmdError 分類**:

- BIZ-05 fail（query 内部エラー）: 「データ取得に失敗しました」
- IO-05 fail（CSV エクスポート、ファイルダイアログキャンセル）: 「保存先が指定されませんでした」（user キャンセル）/ 「ファイル書き込みに失敗しました」（IO エラー）

---

### 56.9 テスト戦略

**Vitest 純関数 only（本 PR scope）**:

`memory/feedback-vitest-react19-setup-pattern.md` §1-8 全踏襲（pretest hook 設計 / vitest.config 最小構成 / eslint+globals 連携 / tsconfig types 拡張 / setup.ts / array-type rule / mock cast pattern / table-driven + focused test 併用）。

| ファイル | ケース数 | 内容 |
|---|---|---|
| `lib/sort-items.test.ts` | 6-8 | 空 / 1 件 / 列別（5 列: product_code / name / quantity / unit_price / amount）/ 同値タイブレーク / null guard（`unit_price=null` 末尾配置） |
| `lib/group-items.test.ts` | 4-6 | 空 / 1 部門 / 複数部門 / 部門小計位置 / department_id 昇順 |
| `lib/filter-items.test.ts` | 4 | null 恒等 / 該当あり / 該当なし / 複数該当 |
| `lib/compute-summary.test.ts` | 4-5 | 空 / 全 auto / 全 manual / 混在 / 未知 source 防御 |
| `lib/calculate-unit-price.test.ts` | 5 | ① 2376/4=594 ② -594/-1=594（返品行）③ 2002/2=1001 ④ 1000/3=333（四捨五入）⑤ quantity=0=null |
| `lib/date-nav.test.ts` | 5-7 | 前日 / 翌日 / 月またぎ / 年またぎ / 閏年 2/29 / 不正 date |

合計 **30-37 ケース**、`makeMockItem` factory 利用で `DailySaleItem` 7 field 全必須を保証（`bindings.ts` との型整合）。

**factory helper**:

```ts
// src/features/daily-sales/lib/test-fixtures.ts
import type { DailySaleItem } from "@/lib/bindings";

export function makeMockItem(overrides: Partial<DailySaleItem> = {}): DailySaleItem {
  return {
    product_code: "P001",
    name: "商品A",
    department_name: "毛糸",
    department_id: 1,
    quantity: 1,
    amount: 100,
    source: "auto",
    ...overrides,
  };
}
```

**hooks/components test は本 PR スコープ外**:

- TanStack Query Provider mock / Router mock は本 PR では足さなかった（その後に整い、今は `QueryClientProvider` を使う test が多数ある）
- `memory/feedback-pr-merge-gate-scope-discipline.md` 準拠、PR size を新規 19 + 更新 11 に抑える
- E2E（Playwright）は Phase 2 完了時 8-9 判定で対応

---

### 56.10 業務ルール

#### 売上明細数（user Option 1.5 採用）

- カード名 = 「売上明細数」（旧表記「取引件数」から変更、`memory/feedback-naming-must-match-reality.md` 準拠）
- 値 = `items.length`（sale_records の行数ベース）+ 内訳「自動 N / 手動 M」
- Tooltip = 「売上レコード行数ベース。レシート単位の取引件数は後続仕様で定義。」
- 後続 BIZ 拡張で receipt_id / POS 取引キー仕様確定後、レシート単位の「取引件数」を追加可能（Backlog 「REQ-501 取引件数の集計単位詳細仕様」）

#### 単価派生（user Option 1.5 採用）

- 計算式 = `Math.round(Math.abs(item.amount) / Math.abs(item.quantity))`
- 商品マスタの販売単価ではなく、売上記録の金額 ÷ 数量で求めた**実績単価**
- 返品行（quantity<0 + amount<0）も絶対値で正数表示（「単価の大きさ」として読みやすい）
- `quantity=0` → `null` → 「—」placeholder + ソート時末尾配置
- 厳密な業務意味（販売単価 / 値引前単価 / レシート単価）が必要になった場合は BIZ-05 DTO 拡張 or sale_records への `unit_price` 系カラム追加で別 PR 検討（Backlog 「REQ-501 単価列の意味精査」）
- 単位と数量（D-113、owner の決定 TD-203。[共通規則](10-common-rules.md) SPEC-UNIT-D3）: 単価は `|金額| × 基準数量 ÷ |数量|` を整数の演算で四捨五入し、長さの商品は 1 m あたり（`¥700/m`）。明細の数量は `formatStockDisplay(quantity, stock_unit)`（`1.3 m`）。部門小計・合計・販売点数のカードは `count_points`（個数の種類の和、`点`）と `length_cm`（長さの種類の和、m）の 2 本で出す（`5 点・1.8 m`。`N 点` の `N` は 3 桁区切りで `1,234 点`。owner の決定 TD-206・TD-207。レジの点数との照合は「レジ日報（公式）」の節の Z001 の表が受け持つ）。画面で部門小計を足す所（`group-items.ts`）も同じ 2 本に足し（SPEC-UNIT-D3 の集計の規則、TS の twin）、数量の並べ替えは明細を `(count_points, length_cm)` に当てた値の順。

#### 前日比符号

- 計算式 = `(today.amount - yesterday.amount) / yesterday.amount * 100`
- 前日 0 円の場合 = 除算回避し「比較不可」表示
- 正の値 = green up 矢印、負の値 = red down 矢印（mockup 整合）

#### 部門小計のソート時挙動

- 列ヘッダソート時、部門小計行 + 部門グルーピングは維持し「部門内のみソート」
- `groupItemsByDepartment` 内で部門ごとに `sortDailyItems` 適用
- 部門順は `department_id` 昇順固定（BIZ-05 順、user の業務優先度順）

#### 返品行（quantity<0）の表示

- `is_voided=0` のため返品も対象（取消し済み売上は除外）
- quantity と amount を負数表示（赤字 / 括弧書きではなく単純な負数）
- 単価列のみ `abs(amount) / abs(quantity)` で正数表示（user Option 1.5 採用、§56.10 単価派生）

---

### 56.11 ショートカット

- グローバル ショートカット（UI-shortcuts `54-ui-shortcuts.md` で定義済）+ 画面固有ショートカット:

| キー | 動作 |
|---|---|
| `Ctrl + ←` | 前日（DateNavigator 同等） |
| `Ctrl + →` | 翌日 |
| `Ctrl + S` | CSV 出力 |
| `Ctrl + /` | ショートカット一覧（グローバル） |

- 実装は本 PR scope 外（Phase 2 完了時に画面固有ショートカット統合）、SHORTCUTS 定数の `daily-sales` グループとして将来追加

---

### 56.12 表記揺れ（3 系統許容）

- **サイドバー label** = `src/config/navigation.ts` の `ui-09a` の「日次売上」（利用者の縦移動メニュー、簡潔さ優先）
- **page h1** = 「日次売上」（PR-1 で mockup の「売上レポート」から分離。日次/月次は TabsHeader で同居するため、h1 が「売上レポート」だと L3 デモで日次か月次か判別不能だった = U4。navigation label と揃える）
- **ウィンドウタイトル** = `src/components/layout/RootLayout.tsx` の `deriveTitle` 経由で「在庫管理システム - 日次売上」（navigation.title 由来、OS タスクバー識別性）

3 系統は読み手・用途が異なる（縦メニュー / 画面見出し / OS タスクバー）。PR-1 以降は label・h1 とも「日次売上」で一致、ウィンドウタイトルのみアプリ名 prefix 付き。

---

### 56.13 非目的（IO/BIZ 同型表）

| やらないこと | 理由 | 責務を持つモジュール |
|---|---|---|
| プリンタの設定の画面 | 印刷先・枚数・用紙は OS の印刷の dialog で選ぶ（UI-09a-D18） | OS |
| 月次タブの本実装 | UI-09b（8-5 PR）scope | UI-09b 着手時 |
| 税率別合計の表示 | モックアップ + SP-501 明示なし | BIZ-05 DTO 拡張後の別 PR（Backlog 「税率別合計の必要性確認」） |
| BIZ-05 への source 別 count 追加 | UI 派生で軽量に実装可能 | 必要になったら BIZ 拡張で削除（`computeSalesLineSummary` を破棄可能に設計） |
| DatePicker（shadcn Date Picker）採用 | `<input type="date">` で十分、依存 7 component + dayjs 不要 | 日付範囲選択が要件化された時点で別 PR |
| hooks/components の Vitest test | 本 PR の時点では Query Provider mock / Router mock が無かった（その後に整い、今は `QueryClientProvider` を使う test がある） | 済み |
| E2E（Playwright）テスト | Phase 2 完了時 8-9 判定で対応 | 8-9 判定後 |
| useTodayDate の共通 `src/lib/dates/` 移送 | UI-00 既存 `useYesterdayDate.ts` への影響、本 PR scope 外 | 将来 UI-09b / UI-06a 等で多用化する時に共通化 |

---

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-05-17 | PR #65 | 新規作成（Phase 2 8-3 UI-09a 日次売上レポート画面、業務ロジックあり版テンプレ、URL state + 2 useQuery + 派生 5 純関数 + 単価派生 + 主動線 CTA 配線パターン初適用） |
| 2026-05-22 | PR-1 (tone/nav fix) | page h1 を「売上レポート」→「日次売上」に分離（U4、§56.12）。TabsHeader の `activeOptions.includeSearch:false` で search params 付き URL の active 維持（B1） |
| 2026-06-06 | PR #70 | SummaryCardsBar カード溢れ対策（§56.7）: Card + CardContent に `min-w-0` + value div `truncate`。月次版と同方針 |
| 2026-06-07 | display-scale follow-up | H-6 feedback 対応として `ProductTable` の商品コードセルを `font-mono text-sm font-medium` に更新し、`ProductTable.test.tsx` で `text-xs` 回帰を防止 |
| 2026-06-08 | selection-tone follow-up | TabsHeader の二択切替 visual を `SegmentedControl` primitive に寄せ、monthly ModeTabs と同じ shared segmented control 仕様を参照する形に更新 |
| 2026-07-29 | 監査是正 順21a plan-first | UI-TABLE-D1として日次5 sortable列のheader implementation ownerを`src/components/sales/SortableHeader.tsx`へ正本化。列集合・sort callback・ARIA・表示は不変 |
| 2026-08-16 | PR #79 | SPEC-SDI-D6: `source_import_count` の「N回の取込みを合算」表示、NULL安全表示、同日複数active importの加算済み表示契約を正本化。 |
| 2026-09-27 | daily-report-z-display（design） | UI-09a-D16: 公式セクションに日計（Z001）の全行の表を足し、同日複数取込みの日は取込みごとに並べる。既存 2 表の見出しに出どころ（Z002 / Z005）を添える（[Plan Packet](../archive/plans/2026-09-27-daily-report-z-display.md)、D-096）。 |
| 2026-10-06 | z001-display（runtime、起票） | UI-09a-D16 の「値」に負の数の書き方（金額 `¥-1,234`、件数 `-1`、同じ section の既存の 2 表と同じ）を補った（owner 決定 2026-10-06、[Plan Packet](../archive/plans/2026-10-06-z001-display.md)）。 |
| 2026-10-17 | 日報の紙と意味づけ（design） | UI-09a-D17〜D19（proposed・未実装、D-120）: 1 日 1 枚の日報の紙（印刷ボタン、並びの既定、文字の大きさの下限 9 pt、色を使わない）、印刷の手段（WebView の印刷、効かなければ `.html` の既定のアプリで開く）、紙の切替えの時期。UI-09a-D13 の紙の用途の記述を直し、支払集計の合算を紙と同じ規則（34 SALES-SHEET-D2）にすると書き、足さない日にも事実と合うよう、2 回以上の日の文・warning の注記の見出し・紙の題を直した。名称か並びの変わった日（D2 の 2）も支払集計の warning を注記に出す。 |
| 2026-10-08 | 単位の拡張 design lane | §56.10 の単価派生に単位の拡張の後（proposed・未実装、D-113 / SPEC-UNIT-D3、owner の決定 TD-203）を追加: 長さの商品の数量は m、単価は 1 m あたり、小計・合計は点と m の 2 本（TD-206・TD-207）。 |
| 2026-10-29 | 日報の紙の runtime（Plan Review round 1 の是正） | UI-09a-D18 の未確認の前提（WebView2 の印刷）を確かめる時を「実装の前」から「実装の後の L3（L3-1）」に直した。UI-09a-D19 の (2) が、紙の値がレジスターツールの日報と行ごとに同じかの確かめ（D-120 の Guarantee range の未確認の前提）を兼ねることを書いた（D-142 の (5)）。 |
| 2026-10-31 | 日報の紙の runtime（実装、D-120） | UI-09a-D16〜D18 の proposed の注記を外し、§56.1 のファイル構成に紙の component と hook を足した。§56.7 の Tooltip の行と §56.13 の印刷ボタンの行を実装に合わせた。 |
| 2026-11-01 | 日報の紙の runtime（Final Review round 1 の是正） | UI-09a-D18 に、`日報を印刷` を押さずに印刷したときの案内の 1 行と、紙を描く間は出さないことを足した（D-127）。§56.1 のファイル構成に案内の component を足した。 |
| 2026-11-02 | 日報・月次の画面の導入前受入（起票） | UI-09a-D13 に受入の場と時期・合格基準・合わないときの扱いを足した（owner の決定 TD-570 で関門 12、packet の O1〜O6 への答え TD-578）。場は開発用の Windows と店の SD の写し。完了の画面を離れた後の取消の入口を受入の前提と書いた。 |
| 2026-11-03 | 日報・月次の画面の導入前受入（計画の review round 1 の是正） | UI-09a-D13 の受入の段で、backup/restore 後の再現の手順と合格基準を「backup → まだ取り込んでいない日を 1 つ取り込む → 同じ日のうちに復元 → backup を作った時の値と同じ」に直した（「復元の前と同じ」は手順の順と食い違った。backup の前の取込みの取消は今の版でできないので、取消を使わない形にした）。綴りの紙を借りて並べ返すこと（TD-580）、USB メモリの写しの置き場、agent の境界を渡し方で書くこと、前提の lane が受入の手順書の行を書き足す義務と、前提の行が残る版では全体の判定が未了であることを足した。 |
| 2026-11-03 | 日報・月次の画面の導入前受入（計画の review の closure round 2 の是正） | UI-09a-D13 の判定の段に、全体の合格は前提の行が残らず行う項目が全部合ったときだけで、月の帳票との照合は行わない間は「行わない（理由）」と書いて合否に数えないことを足した（UI-09b-D9・packet と同じ読み）。写しの folder の形（`<日付の folder>\CASIO\SR500_550_4000\XZ`）と、backup/restore 後の再現で backup を作った時の値を金額を書かずに確かめる参照と、足す日を比べる月から選ぶことを書いた。 |
| 2026-11-03 | 日報・月次の画面の導入前受入（実装） | UI-09a-D13 の受入の段に受入の手順書（`docs/daily-monthly-acceptance-runbook.md`）への link を足した。修正・再取込み後の表示に、取り込み直した後の参照（日次売上の画面のその日の値が、取り込み直す前の確認の画面と完了の画面の値と同じ）を足した（計画の review の closure round 3 の P3）。 |
| 2026-11-11 | 日報・月次の画面の導入前受入（PR #229 の Gated Amendment 1、owner の決定 TD-650） | UI-09a-D13 の受入で値を比べる相手を借りた綴りの紙にそろえた: 修正・再取込み後の表示は、取り込み直す前の確認の画面と完了の画面の値から、借りたその日の紙（3 つの表を枠・行ごと）に、backup/restore 後の再現の月次は、日次の部門別集計の和から、借りた紙の部門別集計の和（紙から電卓で出す）に替えた（画面の値を覚えて運ぶ比べ方は、確認の画面に件数・数量・日計の全行が無く、合計の変わらない誤りを見逃すので採らない。製品の契約 UI-09b-D10 は変えない）。公式集計に、2 回以上精算した日を取り込んで並べるのは月次の判定の後、と順を足した（その日の紙が 2 回分を載せるかが未確認のため）。 |
