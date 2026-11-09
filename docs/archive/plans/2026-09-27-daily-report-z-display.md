# Plan Packet: Z001 / Z002 / Z005 の取り込み情報を画面で見られるようにする（design-first、R3）

2026-09-27 起草。起源は owner の L3 所感（2026-09-04 run 3 原文「Z001とか002とか005とかってもっと情報あるじゃん、その辺の取り込み情報も見れないと困るくない？…無いのはまずい」、[原文](../../design-system/reference/2026-09-04-owner-l3-feedback-raw.md)「run 3 原文」）と、owner 2026-09-11 の再確認「やるべきこと」（[backlog](../../backlog.md) の該当項目と「run 3 原文による訂正」(a)）。owner の着手承認は 2026-09-27「Issue の範囲を避けて片っ端から並列で」（wave 14 の lane D）。

本 lane は design-first で、取り込んだ Z001 / Z002 / Z005 の情報を「どの画面で・どの項目を・どう見せるか」と、そのための BIZ / IO の読み出し契約と Tauri command の返り値（DTO）を設計正本に書く。runtime の実装は後続 lane（[後続 runtime lane への申し送り](#後続-runtime-lane-への申し送り)）が持つ。packet・Matrix・設計正本を plan-first change で更新し、runtime のコードは後続 lane が書く。owner 決定（2026-09-27、Plan Review round 2 の後）により、本 lane の Goal は Z001 の全行に絞った。Z002 / Z005 は既存の表示のまま見出しに出どころを添えるだけで、既存の Z002 の合算は backlog の独立した項目で扱う。

## Workflow State

Use the field definitions, enums, transition evidence, packet-selection rule, and fail-closed behavior from `docs/DEV_WORKFLOW.md` `Workflow State`. Keep exactly one `- Key: value` line per field.

実装後の状態はPR native state / 専用record / CIが所有し、trackedに書かない。

- Phase: archive
- Risk: R3
- Plan Commit: 2e9078ca2f5bdffbe6c80e3d3fc25a83b17cf198
- Amendments: none
- Coordinator: Opus 5.5（Claude Code main session、effort high）
- Writer: Opus 5.5 subagent（worktree `.claude/worktrees/daily-report-z-display`、branch `agent/daily-report-z-display`）
- Plan Reviewer: Opus 5.5（fresh subagent、Writer と別 context）+ Codex（GPT-5.6 Sol か GPT-6 Astra、owner が起動時に指定（GPT-6 Sol は owner 決定 2026-09-26 により DevDay まで GPT-5.6 Sol で代える。座組表〈AGENT_OPERATING_MANUAL `## 座組`〉の GPT-6 Sol に当たる））
- Final Reviewer: Fable 5.1（fresh subagent、design lane の Claude 側）+ Codex（GPT-5.6 Sol か GPT-6 Astra、owner が起動時に指定（GPT-6 Sol は owner 決定 2026-09-26 により DevDay まで GPT-5.6 Sol で代える。座組表〈AGENT_OPERATING_MANUAL `## 座組`〉の GPT-6 Sol に当たる））、互いに独立
- Final Review Minimum: 2
- Human Gate: ready,merge

Final Review Minimum の根拠: R3 で workflow gate を変えない（予定 file はすべて `docs/` 配下で、`scripts/ci/classify-changes.sh` の policy docs の列挙に当たらず `workflow=false`）ため、規則上は 1 を選べ helper も 2 を要求しない。本 lane は operator に見える表示の契約を決めるため、[Contract Audit](../../DEV_WORKFLOW.md#contract-audit-r3r4) が R3 に推奨する 2 本目の独立 audit に当たり、座組も Fable と Codex の 2 本を置く（Codex の合否を外さない owner 方針 2026-09-25）。Codex を待たない判断をするなら Plan Gate 前に 1 へ下げる。

Human Gate の根拠: 製品 runtime・画面は変わらない文書の lane で、Windows native で確かめる対象がない（`manual` なし）。owner の設計判断（[owner への質問](#owner-への質問)）は Human Gate の token ではなく、Owner Effort Budget の介入 1 回として受ける。見た目の確認（実機の before / after と L3）は後続 runtime lane の Human Gate に置く。

遷移記録（append-only）:

- kickoff → spec-check → design（2026-09-27、Writer 起草の plan-first commit）: Risk は R3（Tauri command の返り値と、毎日使う画面の表示契約を決める。`docs/project-profile.md` High-risk Changes の「Tauri command arguments/return types」「daily workflow screens」「BIZ service behavior for sales」に当たる）。設計正本の対象は [Design Sources](#design-sources) のとおりで、設計の更新が要る（Z001 の全行を読む経路・返り値・画面の置き場所がどの正本にも無い）。design に留める理由は 2 つ: (1) 画面の案の選択と同日複数取込みの見せ方が owner の判断待ち（[owner への質問](#owner-への質問)）、(2) 実 Z001 / Z002 の行の性質が未確認（[Contract Probe](#contract-probe) P1 / P2）。どちらも `design → plan-draft` の条件「no unresolved design questions」を満たさず、設計の出力も正本にまだ無い。
- design → plan-draft（2026-09-27、Writer〈Opus 5.5 subagent、worktree run〉）: owner の回答（2026-09-27、Q1 = A、Q2 = (a)、Q3 = (a)、[owner への質問](#owner-への質問)）と Coordinator の Contract Probe（P1 / P2、[Contract Probe](#contract-probe)）を受け、Scope S1〜S7 を設計正本へ書いた（56 UI-09a-D16、34 §19.2 / §19.3、24 §14.21 / §14.24、SCREEN_DESIGN §3、decision-log D-096、29 §29.4.1、backlog）。未解決の設計の質問は無い: 画面の案・同日複数取込み・見た目の確かめ方は owner が決め、P2 の未確認（実物の同日 2 回精算）は Q2 (a) で設計の前提から外れ、Z002 の見出しの語は既定「支払集計」を置いて runtime lane の L3 の文言の確認に回した（SPEC-DRZ-D3）。
- plan-draft → plan-gate（2026-09-27、Writer）: packet と [Test Design Matrix](test-matrices/2026-09-27-daily-report-z-display.md) を commit した（R3）。Plan Commit は pending のまま。Plan Review（fresh Opus と Codex）は Coordinator が発注する。
- Plan Review round 1（2026-09-27、fresh Opus 5.5 と Codex GPT-6 Astra、対象 `e38844cd`）: 両 reviewer とも reject。Coordinator の裁定 r1（全件 accept、Scope は広げない）に沿って Writer が packet・Matrix・24 / 34 / 56 / D-096 / backlog・Plans.md を是正し、plan-gate のまま round 2 へ。
- Plan Review round 2（2026-09-27、fresh Opus 5.5 と Codex GPT-5.6 Sol、対象 `b67b1f48`）: Claude 側は approve、Codex 側は reject。P1 は owner 決定（2026-09-27、Goal を Z001 に絞り Z002 の既存の合算は backlog の独立した項目）で解消し、Coordinator の裁定 r2 に沿って Writer が packet・Matrix・56 D16 の文字列の確定・backlog・biz-task-specs を是正した。予算の上限の改定は owner 承認待ち。plan-gate のまま round 3 へ。
- 予算の承認（2026-09-27、Coordinator の記録）: owner が介入 5・relay 4 を承認（この change での介入 4 回目）。plan-gate のまま Plan Review round 3（上限）へ進む。
- Plan Review round 3（上限、2026-09-27、fresh Opus 5.5 と Codex GPT-6 Astra、対象 `e1bb49df`）: Claude 側は approve、Codex 側は予算の不整合で reject。round 4 は回さず同型の一括是正とし、予算は owner 承認済み（介入 7・relay 5、この change での介入 5 回目）。Writer が予算の欄・Matrix R3 / R9 / R11・Scope S8 と Design Sources を是正した。Phase は plan-gate、Plan Commit は pending のまま（plan-approved の記録は Coordinator の別 commit）。
- plan-gate → plan-approved（2026-09-27、Coordinator、本 commit）: Plan Review round 3（上限、対象 `e1bb49df`）は Claude 側 approve（P3 4）、Codex 側は予算の不整合（P2）と R11（P3）で reject、設計の blocker は無し。round 天井に従い追加の round は回さず、同型の一括是正（`2e9078ca`）で全件を反映し Coordinator が確かめた。owner 承認（2026-09-27、介入 7・relay 5）のもと plan-approved。Plan Commit = 本 commit の親（`2e9078ca`）。
- plan-approved → implementing（2026-09-28、Coordinator、state-only）: 本 lane の成果物（設計正本の Scope S1〜S8）は plan-first の change に同乗済みで、implementing で書く runtime のコードは無い（[Implementation Results](#implementation-results)）。Draft PR を作り Final Review（Fable 5.1 と Codex、互いに独立、Minimum 2）へ進む。
- implementing → archive（2026-09-28、Coordinator の closeout、本 commit）: PR #114 を squash merge（`253eef06`、2026-09-28、helper 経由、owner 指示）。packet と Matrix を archive へ移送し、Implementation Results と Review Response を記録。Owner Effort Budget を closeout 時点の実数に直した。Plans.md と backlog を同期。

## Owner Effort Budget

- 介入回数上限: 8（owner 承認 2026-09-28、Ready の承認と同じ 1 回〈「Readyにしてあげていいよ」、介入 7 回目〉で 7 から改定。closeout 時の消費 8 / 上限 8 = relay 6 の承認までの 6、Ready 1、merge 1〈8 回目〉。既定 3 → 起票時 4 → 1 回目の改定 5〈介入 4 回目〉→ 2 回目の改定 7〈介入 5 回目〉→ 3 回目の改定 8〈介入 7 回目〉）
- 上限の理由: 既定 3 に対し、design-first の lane で owner にしか決められない画面の選択（Q1〜Q3）が 1 回要り、Plan Review round 2 の P1 を owner の Goal の決定で解消し、予算の改定の承認が 2 回要った。これに着手承認・Ready・merge を足して 7 になり、base 同期の closure の是正で relay の改定の承認 1 回が加わって 8 になった（8 への改定は Ready の承認と同じ 1 回）。
- 実働時間上限: 30分（既定。owner の作業は質問への回答、relay、Ready / merge の判断）
- relay 往復上限: 6（owner 承認 2026-09-28、この change での介入 6 回目。既定 2 → 4 → 5 → 6。base 同期の closure〈Codex 114〉が packet の Codex の model の根拠の欠け〈P2〉を指摘し、是正後の closure を取り直すため）。closeout 時の消費 6 / 上限 6
- relay の上限の理由: Plan Review が round 3（天井）まで進み、各 round で Codex の合否を取った（Codex の合否を外さない owner 方針 2026-09-25）。これに Final Review の Codex 1 と、Final Review の closure で Codex を取り直す場合の予備 1 を足して 5 になり、base 同期の closure の P2 の是正後に closure を取り直す 1 が加わって 6 になった。

| 種類 | 消費（2026-09-28 closeout 時点） | 見込み | 計 / 上限 |
|---|---|---|---|
| 介入 | 5: 着手承認「Issue の範囲を避けて片っ端から並列で」、Q1〜Q3 の回答（1 回の問いでまとめて回答）、Goal を Z001 に絞る決定（裁定 r2）、予算の改定の承認 2 回（介入 5・relay 4 の承認、介入 7・relay 5 の承認）、relay 6 の承認（2026-09-28）、Ready（上限 8 への改定を同じ 1 回で、2026-09-28）、merge（2026-09-28）で 8 | なし | 8 / 8 |
| relay 往復 | 6: Plan Review の Codex round 1（GPT-6 Astra）・round 2（GPT-5.6 Sol）・round 3（GPT-6 Astra）、Final Review broad（112、GPT-6 Astra）、base 同期の closure（114、GPT-5.6 Sol）、是正後の closure（115、GPT-5.6 Sol） | なし | 6 / 6 |

- 見込みを超える介入・relay が要ると分かった時点で、Goal Invariant の最小完了条件に戻り、Coordinator が owner に諮る（DEV_WORKFLOW `Owner Effort Budget` の hard stop）。
- Plan Review round 天井: 3（既定 3）

既定値と超過時の Coordinator 責務は `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R3

Reason:
本 lane の diff は設計文書だけだが、決める契約は BIZ-05 の日次売上の返り値（`DailySalesReport.official_daily_report` の Tauri command DTO と生成 bindings）、IO の読み出し、毎日使う日次売上画面の表示である。Risk は file の種類でなく影響で決める（先例: [㉗ 棚卸しと後着売上の時点証拠](2026-09-16-stocktake-count-baseline.md) は docs-only で R3）。後続の runtime lane も R3。

## Goal

Goal Invariant: 取り込んだ Z001 の全行を、日次売上で見られる設計にする（owner 決定 2026-09-27 で Z001 に絞った。Z002 / Z005 は既存の表示のまま、見出しに出どころを添えるだけ）。

### 最小完了条件

- 後続 runtime lane の実装者が、設計正本（56 / 34 / 24 / SCREEN_DESIGN / decision-log）だけを読んで、次を 1 つに決められる: Z001 のどの行を・どの画面のどこに・どの列と見出しで出すか、同じ日に複数回取り込んだ日にどう見せるか、そのために IO / BIZ がどの行をどの順で読み、Tauri command がどの形で返すか。
- 店主が Excel に貼って見ていた Z001 の全行（総売・純売以外の行、総売の個数・純売の件数を含む）が、日次売上の画面で見える設計になっている。Z002 / Z005 は既存の支払集計・部門別集計の表示のまま（行・合算・列を変えず、見出しに出どころ〈Z002 / Z005〉を添えるだけ）。既存の Z002 の合算（1 取込みの中でも `code=01` / `現金` や `code=03` / `クレジット` に当たる行が 2 行以上あると同じ `payment_key` になり 1 行に合算されうる、`daily_report_parser.rs:504-513`、`sales_repo.rs:1070-1082`、実ラベルでの発生は未確認）は既存の潜在バグとして backlog の独立した項目に置き、本 lane の Goal に含めない。
- 決めない事柄（印刷・紙の代わりになるかの受入・月次への展開・取込み履歴の導線）が Non-scope と backlog に行き先つきで残る。

### 失敗定義

- Z001 の総売・純売以外の行が、設計の上でも画面に出る経路を持たないまま閉じる。
- 設計正本だけでは、同日複数取込みの日の Z001 の見せ方、表の列・見出し・空の値の表示、読む順のどれかで答えが 2 つ以上になる、または無い。
- 足してよいか分からない行を黙って合算する設計にする（D-071 の加算の読み出しを、行の性質を確かめずに Z001 へ広げる）。
- 既存の表示（総売上 / 純売上の metric、支払集計、部門別集計、未取込みの note、`N回の取込みを合算`、部門未対応の warning）の契約を壊す、または商品別明細と混ぜる（UI-09a-D12）。
- 実店舗の CSV 本文・金額・ラベルの実値を tracked file に書く。

### 非目的

- 印刷の有無・紙面の中身（店の事実待ち、Issue #105 G1〜G3）。
- 日報画面が Excel + 印刷 + バインダーの代わりになるかの受入判定（導入後の実 1 日で行う）。
- runtime の実装（Rust / TypeScript / bindings の変更）。
- parser・DB schema の変更、Z006 / Z009 / Z011 の取込み。
- 月次画面への Z001 / Z002 の展開。
- Z002 / Z005 の表示の変更（見出しに出どころを添える以外）と、既存の Z002 の合算の是正（backlog の独立した項目）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 現状の事実（2026-09-27、`e7c22f8f` の現物）

Z001 / Z002 / Z005 は日報として取り込まれ、在庫は動かさない（D-025）。保存先と見える場所は次のとおり。

| 取り込んだもの | DB の保存先 | 読む経路と見える画面 | 見えないもの |
|---|---|---|---|
| Z001 の総売・純売の金額 | 親 `daily_report_imports.gross_amount` / `net_amount`（`src-tauri/src/biz/daily_report_import_service/parse.rs:65-74` が Z001 の `gross_sales` / `net_sales` 行の金額から導く） | `sales_repo::get_completed_daily_report_aggregate`（`src-tauri/src/db/sales_repo.rs:1013`）→ BIZ `OfficialDailyReportSummary`（`src-tauri/src/biz/sales_service.rs:77-85`）→ 日次売上の「レジ日報（公式）」の総売上 / 純売上（`src/features/daily-sales/DailySalesPage.tsx:200-201`）。取込みの確認と完了の画面にも出る（`src/features/daily-report-import/DailyReportImportPage.tsx:193-194`、`:331-332`） | — |
| Z001 の全行（総売・純売を含む、ラベル・個数/件数・金額・並び） | `daily_report_summary_lines`（`src-tauri/src/db/schema_v4.rs:22`、INSERT は `src-tauri/src/biz/daily_report_import_service/commit.rs:130`）。総売の第 3 列は `quantity`、それ以外は `count`（`src-tauri/src/io/daily_report_parser.rs:268-272`）。総売・純売以外の `line_key` は並び順から作る `summary_N`（同 `:494-502`、`:515-517`） | 本番の読み出し経路が無い。`FROM daily_report_summary_lines` を含む SELECT は test だけ（下の実測）。BIZ の DTO にも生成 bindings にも Z001 の行の型が無い | **総売・純売以外の全行と、総売の個数・純売の件数** |
| Z002 の全行（ラベル・件数・金額） | `daily_report_payment_lines` | 上と同じ aggregate が `payment_key` で合算（`sales_repo.rs:1046-1099`）→ 日次売上の「支払集計」表（`DailySalesPage.tsx:205-214`）。取込みの確認画面にも出る（`DailyReportImportPage.tsx:221`） | 支払集計に出る（本 lane は変えない）。ただし1 取込みの中でも `code=01` / `現金` や `code=03` / `クレジット` に当たる行が 2 行以上あると同じ `payment_key` になり、日次売上では 1 行に合算される（`daily_report_parser.rs:504-513`、`sales_repo.rs:1070-1082`。実ラベルでの発生は未確認）。また合算の鍵は Z002 の行の並びから作る `payment_N` を含み（同）、同日 2 回の取込みで行の集合や並びが違えば別の項目が合算される（実物の同日 2 回精算は未確認、[Contract Probe](#contract-probe) P2）。既存の潜在バグとして backlog の独立した項目「既存の支払集計（Z002）で異なる項目が 1 行に合算されうる」へ（owner 決定 2026-09-27） |
| Z005 の全行（部門名・個数・金額） | `daily_report_department_lines`（`count` は常に NULL、`daily_report_parser.rs:467-474`） | aggregate が部門で合算（`sales_repo.rs:1101-1197`）→ 日次売上の「部門別集計」表（`DailySalesPage.tsx:215-225`）。月次売上の公式部門集計（`sales_repo.rs:1216`） | 全行が見える |
| 取込みごとの識別（取込み日時・ファイル名） | 親 `imported_at` / `source_files_json`。精算回数・時刻などのプリアンブルは保存しない | 日次売上は `N回の取込みを合算` だけ（`DailySalesPage.tsx:182-184`）。一覧の command `list_daily_report_imports` は登録済み（`src-tauri/src/lib.rs:325`）だが UI からの呼び出しは無い（下の実測） | 同日の各取込みの日時・ファイル名（本 lane は Z001 の見出しに取込み日時だけを使う。一覧と後日の取消の導線は Non-scope） |

実測（2026-09-27、本 worktree）:

```
$ rg -n 'FROM daily_report_summary_lines' src-tauri/src
src-tauri/src/db/sales_repo.rs:2060:  (#[cfg(test)] は同 file の 1337 行目から)
src-tauri/src/db/sales_repo.rs:2565:
src-tauri/src/biz/daily_report_import_service/tests.rs:420:
$ rg -c 'summary_lines' src/lib/bindings.ts
(出力なし、exit=1)
$ rg -n 'listDailyReportImports' src
src/lib/bindings.ts:180: （生成物の定義だけ）
```

ほかに見つけた既存の食い違い（本 lane の Scope で直す）: `docs/function-design/24-io-csv-import-repo.md` §14.21 の「実装遷移義務」は、旧 symbol `get_latest_completed_daily_report` から `get_completed_daily_report_aggregate` への rename を後続 commit の義務として書くが、code は rename 済み（`sales_repo.rs:1013`）。§14.21 は本 lane が Z001 の読み出しを足す節なので、同じ改訂で古い義務の段落を消す。

## 設計案

### 画面の案（owner が選ぶ）

| 案 | 中身 | 良い点 | 気になる点 |
|---|---|---|---|
| **A（推奨、owner 採用 2026-09-27）**: 日次売上の「レジ日報（公式）」に Z001 の表を足す | 既存の section の中、総売上 / 純売上の metric の下に「日計（Z001）」の表を 1 つ足す。列は 名称 / 個数/件数 / 金額、行は Z001 の全行をレジの並びのまま（0 の行も出す）。既存の 2 表の見出しにも出どころを添える（「支払集計（Z002）」「部門別集計（Z005）」）。route・サイドバー・取込み画面は変えない | 壊さない（表を 1 つ足すだけで、既存の表示・導線・route はそのまま）。取込み完了 →「日次売上を見る」の既存の導線でそのまま届く。前日 / 翌日と日付入力で過去日を引ける。既存の部品（`OfficialLinesTable` と同じ形の表、DSR-16 の「比較が目的 → 列を揃えた表」）で作れる | 日次売上の縦が伸び、商品別明細が下へ下がる。Z001 の行数が多いと表が長い（匿名化要約ではデータ 28〜29 行、P1） |
| B: 新しい画面「レジ日報」 | `/reports/daily-report?date=` のような route を新設し、Z001 / Z002 / Z005 を紙の日報と同じ並びで全行出す。日次売上には今の要約を残し「レジ日報の全項目を見る」リンクを置く | 1 日 1 枚の紙に近い形で見られる。将来、印刷を作るならその土台になる | 画面が 1 つ増え、覚えることが増える。route・サイドバー・到達テスト（`navigation.test.ts`）の義務が増える。印刷の要否が決まる前に紙の形へ寄せることになる |
| C: 取込みの確認・完了画面と、取込みの記録に出す | 売上データ取込みの確認・完了画面に Z001 の全行を出し、取込みごとの詳細画面（記録ハブ）を新設する | 取込みごとに見るので合算の問題が起きない | 日付で過去日を引けない（取込みの記録からたどる）。日報の記録ハブと詳細画面は新設で、A より作るものが多い。「見る」画面が取込み画面になり、日次売上との二重の置き場所になる |

推奨は A。根拠: owner の好み「壊さない優先」「飾りより情報」に最も合い、画面・導線を増やさずに不足（Z001 の全行）だけを埋める。B と C が持つ利点（紙に近い 1 枚、取込みごとの確認）は、印刷の要否（Issue #105）と取込み履歴の導線（backlog「日報取込み標準手順の残設計」）が決まったときに、A を土台に別 lane で足せる（A はそれを塞がない）。

### 同じ日に複数回取り込んだ日の Z001（owner が選ぶ、既定案あり）

- 既定案（推奨）: **取込みごとに並べ、合算しない**。取込みが 2 回以上の日だけ、Z001 の表を取込みの古い順に並べ、各表の見出しに「1 回目の取込み（取込み日時 2026-03-21 18:05）」の形で取込み日時（`imported_at`、`YYYY-MM-DD HH:mm`）を添える。1 回の日は見出しを付けず表 1 つ。どの取込みも落とさない（D-071 の「同日の全 active import を読む」は守る）。
- 理由: Z001 の行が精算ごとの差分（足してよい値）か累計・回数かが未確認（P2）で、総売・純売以外の行の鍵は並び順から作る `summary_N`（`daily_report_parser.rs:494-502`）のため、合算すると別の意味の行を足すおそれがある。取込みごとに並べれば、行の性質によらず正しい。
- 別案: 1 つの表に合算する（支払集計・部門別集計と同じ見せ方）。P2 で Z001 の全行が足してよい値と確かめられた場合に限り選べる。owner は既定案 (a) を採った（2026-09-27）ため、この案は D-096 の見直す条件に置く。
- 取込みごとに並べる場合、総売上 / 純売上の metric・支払集計・部門別集計は今までどおり合算のまま（UI-09a-D15 を変えない）。同じ section の中で「合算」と「取込みごと」が混ざるため、`N回の取込みを合算` の文を `総売上・純売上・支払集計・部門別集計は{N}回の取込みを合算しています。日計（Z001）は取込みごとに表示します。` へ直す（文字列は UI-09a-D16 で完全一致として確定）。

### 契約の既定案（案 A と「取込みごとに並べる」を採る場合。Writer が正本へ書く）

- SPEC-DRZ-D1（画面、56 の UI-09a-D16 として正本化）: 「レジ日報（公式）」section の metric の下に「日計（Z001）」の表を置く。列は 名称 / 個数/件数 / 金額（見出しの「個数/件数」はレジの帳票の見出しに合わせる。単位の文字は付けない）。行は保存された `label` をそのまま、`sort_order` の順に全行（0 の行を含む）。値が NULL の欄は「—」（その行にその値が無いという意味。合算で欠けた「未取得」とは別）。金額は既存の `¥` 表記、数は `toLocaleString("ja-JP")`。
- SPEC-DRZ-D2（複数取込み）: 上の「取込みごとに並べる」。
- SPEC-DRZ-D3（既存表示の不変）: 総売上 / 純売上の metric、支払集計・部門別集計の合算、未取込みの note、部門未対応の warning、商品別明細との分離（UI-09a-D12）、UI-09a-D15 の合算表示は変えない。変えるのは既存 2 表の見出しに出どころ（Z002 / Z005）を添えることと、D2 の文だけ。Z002 の表の見出しの語は P1 で決める予定だったが、行の名前ごとの照合は未実施（生の CSV を開かない決まり）のため、語は「支払集計」のまま変えず、見出しは `支払集計（Z002）` で確定した（UI-09a-D16。Goal を Z001 に絞った owner 決定 2026-09-27 により、Z002 の行の中身に合わせた言い換えは本 lane で扱わない）。
- SPEC-DRZ-D4（BIZ-05 の返り値、34 に正本化）: `OfficialDailyReportSummary` に field `summary_imports` を足す。型は下のとおり。`line_key` は返さない（並び順から作る鍵は利用者に意味を持たない）。`summary_imports` の件数は `source_import_count` と等しい（completed の取込みは総売か純売の行を必ず持つ、`parse.rs:75-79`）。

```rust
struct OfficialDailySummaryImport {
    daily_report_import_id: i64,   // UI の key と取込みの識別
    imported_at: String,           // YYYY-MM-DDTHH:MM:SS
    lines: Vec<OfficialDailySummaryLine>, // sort_order ASC, id ASC
}

struct OfficialDailySummaryLine {
    label: String,
    quantity: Option<i64>,
    count: Option<i64>,
    amount: Option<i64>,
}
// OfficialDailyReportSummary { ...既存..., summary_imports: Vec<OfficialDailySummaryImport> } // imported_at ASC, id ASC
```

- SPEC-DRZ-D5（IO の読み出し、24 §14.21 に正本化）: `get_completed_daily_report_aggregate` が、同日の completed の親の Z001 行を 1 つの query で読み、親の `imported_at ASC, id ASC`、行の `sort_order ASC, id ASC` で返す。`rolled_back` の親の行は読まない。行は合算しない。新しい command・新しい repo 関数・schema 変更・index 追加はしない（`idx_daily_report_summary_lines_import_id` が既にある、`schema_v4.rs:60`）。
- SPEC-DRZ-D6（command、42 は変更不要）: `get_daily_sales` の名前・引数は変えず、返り値 `DailySalesReport` の中の field が増えるだけ。runtime lane で bindings を再生成する。呼ぶ側はホームの `useHomeSummary.ts:22` と日次売上の `useDailySalesReport.ts:53` `:65`（前日の取得を含む）。日次 CSV 出力は `get_daily_sales` を内部で呼ぶ（34 §19）が、CSV の列は変えない。
- SPEC-DRZ-D7（決定の記録、D-096）: D-071 の「同日の全 active import を加算して読む」のうち、Z001 の行は「全 active import を読むが、足してよいと確かめていない行は取込みごとに並べる」とする。理由・不採用案（合算・最新 1 回だけ・新しい画面）・見直す条件（P2 で全行が足せると分かり owner が 1 つの表を望む、印刷の設計が紙 1 枚の形を要する）を D-096 に置く。

### owner への質問

1 回の問いでまとめて聞く（介入 1 回）。

- Q1 画面の案: A 日次売上に表を足す（推奨）/ B 新しい画面「レジ日報」/ C 取込みの画面と記録に出す。
- Q2 同じ日に 2 回以上取り込んだ日の日計（Z001）: (a) 取込みごとに並べる（推奨）/ (b) 1 つの表に合算する（P2 で全行が足せると確かめた場合だけ）。
- Q3 見た目の確かめ方: (a) runtime lane の Draft で実機の before / after を並べて見る（推奨。案 A は既存の表と同じ形を足すだけなので、先に mockup を作る効果が小さい）/ (b) 先に mockup（静的な HTML）を作って見てから runtime lane を起こす。

owner の回答（2026-09-27、Coordinator が中継、介入 2 回目）:

- Q1 = **A**（日次売上の「レジ日報（公式）」の下に Z001 の表を 1 つ足す。既存 2 表の見出しに出どころ〈Z002 / Z005〉を添える）。
- Q2 = **(a)**（同じ日に 2 回以上取り込んだ日の Z001 は、取込みごとに古い順で並べ、取込み日時を見出しにする。合算しない）。
- Q3 = **(a)**（見た目は後続 runtime lane の Draft で実機の before / after を並べて見る。mockup は作らない）。
- 画面の変更の時期: owner は「画面の変更は店舗確認の後」を本 lane には掛けない（2026-09-27 確認）。本 lane は設計の文書だけ。

owner の決定（2026-09-27、Plan Review round 2 の裁定 r2、介入 3 回目）: 本 lane の Goal は「取り込んだ Z001 の全行を日次売上で見られる」に絞る。Z002 / Z005 は既存の表示のまま（見出しに出どころを添えるだけ）。既存の Z002 の合算（`code=01` / `現金`、`code=03` / `クレジット` が同じ `payment_key` になり 1 行に合算されうる）は既存の潜在バグとして backlog に独立した項目で起票し、Z002 の行の構成を確かめてから直す。

## Ordinary Operation

この文書を完了できること（本 lane）と、店主が画面で日報の全項目を見られること（通常運用の目的）は別である。後者は runtime lane の実装と L3 まで達成しない。紙（Excel + 印刷 + バインダー）をやめられるかは、さらに別の受入判定（導入後の実 1 日）で決まる。下表は案 A と Q2 (a) を採った場合の、runtime lane の完了後の店主の普通の一日である。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 閉店後、レジを精算し SD から CV17 へ取り込んだ。アプリはホーム | 「売上データ取込み」で Z001 / Z002 / Z005 の 3 つを選び、確認して取り込む | 完了画面に総売上 / 純売上が出る（既存のまま） | 完了画面の「日次売上を見る」を押す | なし（既存の導線） |
| 日次売上がその日の日付で開く | 「レジ日報（公式）」の section を見る | 総売上 / 純売上の下に「日計（Z001）」の表があり、Excel に貼っていた Z001 の項目がレジの帳票と同じ名前・同じ並びで、個数/件数と金額つきで全部見える。その下に既存の支払集計（Z002）・部門別集計（Z005）が見出しに出どころつきで今までどおり出る | Z001 の全行が表にあり、値の無い欄が「—」で、0 の行も出ている | Z001 の行数と各行の埋まる欄（P1）。行数が多いと表が長い。Z002 の既存の合算（`payment_key` の重複）は本 lane の Goal の外で、backlog の独立した項目 |
| 翌日以降、前の日の日報を見返したい | 日次売上で「前日」ボタンか日付入力で過去日を開く | その日の日計（Z001）・支払集計・部門別集計が出る。取り込んでいない日は既存の「この日付のレジ日報は未取込みです。」 | 取り込んだ日と取り込んでいない日を見分けられる | なし |
| 同じ日に精算を 2 回して、2 回とも取り込んだ | 日次売上でその日を開く | 総売上 / 純売上・支払集計・部門別集計は 2 回分の合算。日計（Z001）は「1 回目の取込み（取込み日時 …）」「2 回目の取込み（取込み日時 …）」の 2 つの表が古い順に並ぶ。section の文で「合算している表」と「取込みごとの表」の違いが分かる | 2 つの表のどちらがどの取込みか、取込み日時で分かる | Z001 の行が足せる値か（P2）。取込み日時は精算の時刻ではない（精算時刻は保存していない） |
| 取り込んだ直後に、間違ったファイルだったと気づいた | 取込みの完了画面で「取消」 | 取り消した取込みの日計は日次売上から消え、残った取込みだけが出る（既存の取消・再取得の挙動に Z001 が乗る） | 日次売上を開き直すと残った取込みだけ | 後日（完了画面を離れた後）の取消の導線は無い（Non-scope、backlog） |

## Scope

本 lane は設計文書だけを変える。Writer は owner の回答（Q1〜Q3）と Contract Probe P1 / P2 の結果を受けて、下の file だけを編集する。下の S1〜S8 は案 A・Q2 (a) の場合で、owner が別の案を選んだら design のまま Scope を書き直してから plan-draft へ進む。

- S1 `docs/function-design/56-ui-daily-sales.md`: §56.1 の表示領域の表に「日報サマリの日計（Z001）」の行を足し、UI-09a-D16（SPEC-DRZ-D1〜D3 の画面の契約: 置き場所・列・見出し・行の順・0 の行・「—」の意味・複数取込みの見出しと並び・`N回の取込みを合算` の文の直し・既存 2 表の見出しの出どころ）を「REQ-401 第2スライス表示詳細」の後に置く。UI-09a-D15 に「Z001 の行は UI-09a-D16 のとおり取込みごと」の 1 文を足す。§56.13 非目的の印刷の行は変えない。更新履歴に 1 行。
- S2 `docs/function-design/34-biz-sales-service.md`: §19 の `DailySalesReport` / `OfficialDailyReportSummary` の型に SPEC-DRZ-D4 の `summary_imports` と 2 つの型を足し、`get_daily_sales` の処理ステップと「設計判断 — 日報集計と商品別明細を分ける」に Z001 の行は合算しないことを 1 文ずつ足す。
- S3 `docs/function-design/24-io-csv-import-repo.md` §14.21: 関数要求・処理ステップに SPEC-DRZ-D5 の Z001 行の読み出し（1 query、並び、`rolled_back` を読まない、合算しない）を足す。古い「実装遷移義務」の段落と旧 symbol のコード片を消す（code は rename 済み）。§14.24 の D-071 の行に「Z001 の行は加算せず取込みごとに返す、D-096」の 1 句を足す（additive read が Z001 にも掛かると読めないように）。
- S4 `docs/SCREEN_DESIGN.md` §3「日次売上レポート画面」: レイアウト判断に「日報サマリに日計（Z001）の全行の表を置き、同日複数取込みの日は取込みごとに並べる（UI-09a-D16）」の 1 行。
- S5 `docs/decision-log.md`: D-096 を足す（SPEC-DRZ-D7）。
- S6 `docs/function-design/29-io-daily-report-parser.md` §29.4.1: P1 / P2 で分かった Z001 / Z002 の行の性質（行数の範囲、どの欄が埋まるか、精算ごとの差分か累計か）を、実値・実ラベルなしの匿名化した形で 1〜3 行足す。P1 / P2 が新しい事実を出さなければ変えない。
- S7 `docs/backlog.md`: 34 行目の項目に本 packet への link と状態（design lane 起票済み）を足す。後続 runtime lane を 1 行で起票する。本 lane で見つけた後続候補（月次への Z001 / Z002、日報取込みの履歴一覧と後日の取消の導線、Z002 の合算の鍵の妥当性〈P2 の結果次第〉）を 1 行ずつ起票する（既存の項目で扱えるものは既存へ寄せる）。
- S8 `docs/architecture/biz-task-specs.md`: BIZ-08 の「集計契約」の公式日報日次の 1 行に「（Z001 の行は全 active import を読むが加算せず、取込みごとに ID つきで返す、D-096）」を足す（24 §14.24 と同じ扱い、Plan Review round 2 の裁定 r2 D2-5 で追加）。

確認した呼出し側と隣接（編集しないが Scope の判断に使った）: `docs/function-design/42-cmd-sales-stocktake.md` の `get_daily_sales`（「DailySalesReport をそのまま返す」で型を持たないため変更不要、SPEC-DRZ-D6）、`docs/function-design/37-biz-daily-report-import-service.md` / `45-cmd-daily-report-import.md` / `55-ui-csv-import.md`（取込みの確認・完了は変えない）、`docs/function-design/57-ui-monthly-sales.md`（月次は Non-scope）、`docs/db-design/pos-tables.md` 12c（「表示・照合に使える行データとして保存」で読み出しを既に想定、変更不要）、`docs/DB_DESIGN.md`（schema 不変）、`docs/function-design/90-traceability.md`（下の Registration / Generation Obligations）。

## Non-scope

- 印刷（日報の印刷の要否と紙面の中身）。店の事実待ち（Issue #105 G1〜G3、2026-09-27 追記）。56 §56.13 の「印刷ボタンの本実装」の行も変えない。
- 日報画面が Excel + 印刷 + バインダーの代わりになるかの受入判定（backlog「日報画面の Excel 印刷・バインダー代替受入判定」、UI-09a-D13）。導入後の実 1 日で行う。
- runtime の実装（`sales_repo.rs` / `sales_service.rs` / bindings / `DailySalesPage.tsx` / test）。後続 runtime lane が持つ。
- parser の変更（Z001 の `line_key` の意味付けの拡張を含む）、DB schema・migration・index、Z006 / Z009 / Z011 の取込み（pos-tables B-2）。
- 月次画面への Z001 / Z002 の展開（合算の意味が要る。backlog へ）。
- 取込みの確認・完了画面への Z001 の全行（取込み前の確認は「正しいファイルか」を見る場所で、全項目は取込み後に日次売上で見る。案 C を採らない限り変えない）。
- 日報取込みの履歴一覧と、完了画面を離れた後の取消の導線（`list_daily_report_imports` は登録済みで UI 未使用。backlog「日報取込み標準手順の残設計」へ寄せる）。
- 日次 CSV 出力への Z001 の追加、run 3 のほかの項目（前日比の符号色、CSV 出力・印刷のボタンの位置）。
- layout A のプリアンブル（精算回数・日付・時刻）と Z001 の「レコード」列の表示（どちらも保存していない。backlog へ）。そのため日計の「N回目の取込み」は取込みの順で、精算の順ではない（UI-09a-D16）。
- 既存の支払集計（Z002）の `payment_key` の重複・並び順由来の鍵による合算。既存の潜在バグで、owner 決定 2026-09-27 により backlog の独立した項目「既存の支払集計（Z002）で異なる項目が 1 行に合算されうる」で、Z002 の行の構成を確かめてから直す。

## Acceptance Criteria

本 lane（設計文書の改訂）の AC。baseline は 2026-09-27 に本 worktree（`e7c22f8f`）で同じ command を実行した結果。

- AC1 画面の契約: `rg -n 'UI-09a-D16' docs/function-design/56-ui-daily-sales.md` が 1 件以上（baseline: 出力なし、exit=1）。D16 の本文だけで、置き場所・列・見出し・行の順・0 の行・「—」の意味・複数取込みの見出しと並びが 1 つに決まる（review で確認）。
- AC2 返り値の契約: `rg -n 'summary_imports' docs/function-design/34-biz-sales-service.md docs/function-design/24-io-csv-import-repo.md docs/function-design/56-ui-daily-sales.md` が 3 file すべてで 1 件以上（baseline: `rg -c summary_imports docs src src-tauri` は出力なし、exit=1）。
- AC3 古い義務の削除: `rg -c 'get_latest_completed_daily_report' docs/function-design/24-io-csv-import-repo.md` が出力なし・exit=1（baseline: `1`）。
- AC4 決定の記録: `rg -c '^## D-096' docs/decision-log.md` が `1`（baseline: 出力なし、exit=1）。D-096 は Decision / Why / Alternatives / Revisit を持つ。
- AC5 既存表示の不変: `git diff e7c22f8f -- docs/function-design/56-ui-daily-sales.md` の削除行（`-` で始まる行）に UI-09a-D12 / D13 と §56.13 の印刷の行が含まれない（review で diff を読む）。
- AC6 runtime を触らない: `git diff --name-only e7c22f8f -- src src-tauri tests` が空。
- AC7 Probe の記録: [Contract Probe](#contract-probe) の P1 / P2 が `結果:` つきで埋まり、実値・実ラベルを含まない（Data Safety、review で確認）。
- AC8 backlog: `rg -n 'plans/2026-09-27-daily-report-z-display' docs/backlog.md` が 1 件以上（baseline: 出力なし）。
- AC9 docs の検査: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が exit 0。
- AC10 traceability: `git diff --quiet e7c22f8f -- docs/function-design/90-traceability.md` が exit 0。REQ の参照を増減した場合は `cargo run --bin generate_traceability` で再生成し、その差分を同じ commit に含める（下の Registration / Generation Obligations）。
- AC11 IO と BIZ の型の層（Plan Review round 1 の裁定 r1 で追加）: `rg -c 'Option<OfficialDailyReportSummary>, DbError' docs/function-design/24-io-csv-import-repo.md` と `rg -c 'get_completed_daily_report_aggregate\(conn, date\) → Option<OfficialDailyReportSummary>' docs/function-design/34-biz-sales-service.md` がどちらも出力なし・exit=1（baseline: `e7c22f8f` でどちらも `1`、`git show e7c22f8f:<path> | rg -c '<同じ pattern>'` で実測）。IO の返り値は `OfficialDailyReportRow`（review で確認）。
- AC12 Rust の wire 契約 test の申し送り（同）: `rg -c 'import_internal_contract_test' docs/plans/test-matrices/2026-09-27-daily-report-z-display.md` が 1 以上（baseline: `e38844cd` の Matrix で出力なし、exit=1）。
- AC13 表示の文字列の確定（Plan Review round 2 の裁定 r2 で追加）: `rg -c '総売上・純売上・支払集計・部門別集計は\{N\}回の取込みを合算しています。日計（Z001）は取込みごとに表示します。' docs/function-design/56-ui-daily-sales.md` が 1 以上（baseline: `b67b1f48` で出力なし、exit=1）、かつ `rg -c '文言の確定は|の類を出す|言い換えは runtime lane' docs/function-design/56-ui-daily-sales.md` が出力なし・exit=1（baseline: `b67b1f48` で `2`）。どちらも `git show b67b1f48:<path> | rg -c '<同じ pattern>'` で実測。
- AC14 日次 CSV の不変の test 設計（同）: `rg -c 'R16' docs/plans/test-matrices/2026-09-27-daily-report-z-display.md` が 1 以上（baseline: `b67b1f48` で出力なし、exit=1）。

## Design Sources

- Requirements / spec: `docs/spec/requirements.md` REQ-401 / REQ-501、`docs/spec/requirements-coverage.md` SP-501
- Architecture: `docs/ARCHITECTURE.md`（`UI -> CMD -> BIZ -> IO/MNT`）、`docs/project-memory.md` Store Premises Facts（日報の Excel 貼付け・上書き・印刷・バインダー）
- Function / command / DTO: `docs/architecture/biz-task-specs.md` BIZ-08 の集計契約（S8）、`docs/function-design/56-ui-daily-sales.md` §56.1（UI-09a-D12 / D13 / D15）、`34-biz-sales-service.md` §19.3、`24-io-csv-import-repo.md` §14.21、`42-cmd-sales-stocktake.md` `get_daily_sales`、`29-io-daily-report-parser.md` §29.4、`37-biz-daily-report-import-service.md` §37.2
- DB: `docs/DB_DESIGN.md`、`docs/db-design/pos-tables.md` 12b〜12e と B-2
- Screen / UI: `docs/SCREEN_DESIGN.md` §3「日次売上レポート画面」、`docs/design-system/00-foundations.md`（ラベルと値、強調の段階）、`01-decision-rules.md` DSR-16、`04-backbone.md` 原則 2 / 3 / 5、`.agents/skills/inventory-operator-ui/SKILL.md`
- Decision log / ADR: D-023（POS adapter の境界）、D-025（日報と Z004 の分離）、D-071（同日複数精算の加算の読み出し）、D-091（デザインの決まり）。新規 D-096（予約、SPEC-DRZ-D7）

## Required Design Artifacts

| Area touched by upcoming work | Required source doc / artifact | Status: existing sufficient / updated in this PR / intentionally deferred |
|---|---|---|
| Backend function / command / repository / validation / error | 34 §19.3（BIZ-05）、24 §14.21（IO） | updated in this PR（Writer run、S2 / S3） |
| Command / DTO / generated binding / wire shape | 34 の型（DTO の所有元）、42 の `get_daily_sales` | 34 は updated in this PR、42 は existing sufficient（型を持たない pass-through）。bindings の再生成は runtime lane |
| DB / transaction / audit / rollback / migration | `DB_DESIGN.md`、pos-tables 12c | existing sufficient（schema・index 不変、読み出しだけ足す） |
| Screen / UI / route state / Japanese wording | 56 §56.1、SCREEN_DESIGN §3 | updated in this PR（S1 / S4）。route・search state は不変 |
| CSV / TSV / report / import / export format | 29 §29.4.1（匿名化 shape） | P1 / P2 が新しい adapter の事実を出したときだけ updated in this PR（S6）。CSV 出力は不変 |
| Durable decision / ADR | decision-log D-096 | updated in this PR（S5） |

## Registration / Generation Obligations

- 本 lane（文書だけ）: REQ の参照を増減しない限り `docs/function-design/90-traceability.md` の再生成は要らない（AC10）。function-design doc の新設・改名はない。
- 後続 runtime lane へ申し送る義務: Tauri command は新設しない（`get_daily_sales` の返り値の型が増える）。`cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成する。案 B を選んだ場合だけ、route 新設（`npm run generate:routes`）と `src/config/navigation.ts` の entry と `navigation.test.ts` の到達テストの義務が加わる。

## Design Intent Trace

| Spec / requirement ID | Source design doc section | Decision ID | Why / rejected alternatives | Implementation target | Test target |
|---|---|---|---|---|---|
| REQ-501 / SP-501 | 56 §56.1 | UI-09a-D16（SPEC-DRZ-D1 / D3） | Z001 の全行を既存の公式 section に足す。不採用: 新しい画面（案 B）、取込み画面（案 C）、0 の行を隠す（紙と並びが変わる）、ラベルをアプリで言い換える（adapter の事実を core の意味にしない、D-023） | runtime lane: `DailySalesPage.tsx` の `OfficialDailyReportSection` | runtime lane: `DailySalesPage.test.tsx`（行・順・「—」・0 の行を text で確かめる） |
| REQ-501 / REQ-401 | 56 §56.1、decision-log | UI-09a-D16、D-096（SPEC-DRZ-D2 / D7） | 同日複数取込みの Z001 は取込みごとに並べる。不採用: 合算（行の性質が未確認、鍵が並び順由来）、最新 1 回だけ（D-071 違反） | runtime lane: 同上 | runtime lane: 2 取込みの日の表示の test |
| REQ-501 | 34 §19.3 | SPEC-DRZ-D4 | `summary_imports` を既存の DTO に足す。不採用: 新しい command（呼び出しが増えるだけ）、`line_key` の公開（利用者に意味が無い） | runtime lane: `sales_service.rs` の `OfficialDailyReportSummary` と map | runtime lane: `sales_service.rs` の REQ-501 test |
| REQ-401 / REQ-501 | 24 §14.21 | SPEC-DRZ-D5 | 既存の aggregate に 1 query を足す。不採用: 新しい repo 関数、schema・index の追加 | runtime lane: `sales_repo.rs::get_completed_daily_report_aggregate` | runtime lane: `sales_repo.rs` の test（並び・`rolled_back` の除外・合算しない） |

## Design Intent Audit

- Source docs can answer what is being built and why without chat history or archived Plan Packets: 答えられる（56 UI-09a-D16、34 §19.2 / §19.3、24 §14.21、D-096 に書いた。AC1〜AC4）。
- Plan-only durable decisions found and promoted to source docs / decision-log / ADR: SPEC-DRZ-D1〜D7 を 56 / 34 / 24 / SCREEN_DESIGN / D-096 へ昇格した。packet だけで決めて正本に無いものは無い。
- Assumptions and constraints: Z001 の行は数値列が整数で parse できる（実ファイルは 2026-07 の L3 で `parse_errors=0`、[REQ-401 実装 packet](2026-07-04-req401-sales-daily-report-implementation.md) の Local-only gate）。行数と各行の性質は P1 / P2 のとおり（行の名前ごとの照合と、実物の同日 2 回精算は未確認）。
- Deferred design gaps, risk, and follow-up target: 印刷（Issue #105）、紙の代わりの受入（backlog）、月次への展開・取込み履歴と後日の取消（S7 で backlog へ）。
- Test Design Matrix can cite design decision IDs or source doc sections: 可（[Test Design Matrix](#test-design-matrix) の予定行は SPEC-DRZ-D1〜D7 を引く）。
- Absolute guarantee / escape hatch self-check completed, with every exception checked and compatibility stated: 「どの取込みも落とさない」は `rolled_back` を除く completed の全親を読むことで守る。例外は無い。「`summary_imports` の件数 = `source_import_count`」は completed の取込みが総売か純売の行を必ず持つこと（`parse.rs:75-79` で両方無ければ取込み不可）に依る。Z001 の行が 0 件の completed 親は作られない。

## Impact Review Lenses

| Lens | Applicability / finding | Follow-up artifact |
|---|---|---|
| Adapter / core boundary | Z001 のラベルと並びは CASIO adapter の事実（D-023）。core は総売・純売だけに意味を付け、ほかの行はラベルと数をそのまま運ぶ。画面もラベルを言い換えない | SPEC-DRZ-D1、D-096 |
| Fact check / design decision split | 事実 = Z001 の行数・欄の埋まり方・足せる値か（P1 / P2、未確認）。判断 = 置き場所・見せ方・合算しないこと | Contract Probe、S6 |
| Lifecycle / retry | 取込み・同日追加・取消・再取込みの後、読み出しは completed の親だけを読むため、取消した取込みの Z001 は消え、残りだけが出る。既存の invalidation（`invalidationContract.dailyReportImport()`）で日次売上が再取得される | SPEC-DRZ-D5、runtime lane の test |
| Operator workflow | 取込み完了 →「日次売上を見る」→ 公式 section の既存の導線に乗る。過去日は前日 / 日付入力 | Ordinary Operation |
| Replacement path | レジが変わっても core の契約（ラベル・数・並びの運搬）は変わらず、adapter が Z001 相当の行を作れば同じ画面に出る | D-096 の見直す条件 |
| Data safety / evidence | P1 / P2 は repo 外の匿名化メモと帳票仕様の PDF を読み、行数・欄の種類・足せるかだけを記録する。実値・実ラベルは書かない | Data Safety |
| Reporting / accounting semantics | 公式日報の series と商品別の series を足さない（UI-09a-D12、D-025）。Z001 の行は足してよいか未確認のため取込みごとに並べる（D-096）。既存の Z002 の合算は、P2 で足せない行が見つかれば別の問題として backlog へ | SPEC-DRZ-D2 / D7、S7 |
| Manual verification | 表の長さ・読みやすさ・「合算」と「取込みごと」の違いが伝わるかは実機でしか確かめられない。runtime lane の Human Gate（`manual`、Windows native L3、before / after） | 後続 runtime lane への申し送り |
| 環境・再現性 | 新しい環境依存なし | なし |

## Design Readiness

- Existing design docs are sufficient because: 起草時は十分でなかった（Z001 の全行を読む経路・返り値・画面の置き場所がどの正本にも無い、[現状の事実](#現状の事実2026-09-27e7c22f8f-の現物)）。本 change の S1〜S8 で埋めた。
- Source docs updated in this PR: S1〜S8（owner の回答と P1 / P2 の後、Writer run、2026-09-27。S8 は Plan Review round 2 の是正）。
- Design gaps intentionally deferred: 印刷、紙の代わりの受入、月次、取込み履歴と後日の取消（Non-scope）。
- Durable decisions discovered in this plan and promoted to source docs: D-096（Z001 は取込みごと）、UI-09a-D16。

Minimum design checks for business-app work:

- Layer ownership (`UI -> CMD -> BIZ -> IO/MNT`): IO が行を読み、BIZ が DTO へ写し、CMD は pass-through、UI は表示だけ（合算も並べ替えもしない）。
- Backend function design: 既存の `get_completed_daily_report_aggregate` と `map_official_daily_report` を広げる（SPEC-DRZ-D4 / D5）。
- Command / DTO / data contract: `get_daily_sales` の返り値に field が増える（後方互換の追加。field を消さない）。
- Persistence / transaction / audit impact: なし（読み出しだけ）。
- Operator workflow / Japanese UI wording: 見出し・文・取込みごとの見出し・行の無い取込みの文は UI-09a-D16 の文字列の表で完全一致として確定した。runtime lane の L3 はその文字列で店主が読み違えないかの確認に限る。
- Error, empty, retry, and recovery behavior: 未取込みの日は既存の note のまま（Z001 の表も出さない）。取得失敗は既存の上部 Alert と再試行のまま。
- Testability and traceability IDs: REQ-501 / REQ-401、UI-09a-D16、SPEC-DRZ-D1〜D7。

## Contract Probe

本 lane は実 Z001 / Z002 の行の性質（外部のレジの帳票仕様）に依る。plan-draft の前に Coordinator が repo 外の匿名化メモ（`inventory-field-check/summaries/2026-07-06-z00x-shape-analysis.md`）と帳票仕様の PDF（`approved-readable/ECRCV17.pdf`、`SRS4000_JA3.pdf`）を読み、下の形で記録する。実値・実ラベル・実店舗の値は書かない。

- P1 実 Z001（layout A / B）と実 Z002 の行構成: 行数の範囲、各行で「個数/件数」「金額」のどちらが埋まるか、ラベルがレジの既定か店の設定か、Z002 に支払以外の取引キーの行があるか → 結果（2026-09-27、Coordinator 実施）: repo 外の匿名化要約（`inventory-field-check/summaries/`、2026-07 の 2 回の採取）で、Z001 はヘッダが 8 行目・4 列（レコード / キャラクター / 個数・件数 / 金額）、データ 28〜29 行。Z002 も同じ 4 列形状でデータ約 50 行。行の名前ごとの照合は、生の CSV を開かない決まり（field-check の「Z001 / Z002 / Z005 は直接閲覧禁止」）のため未実施。Z001 の行の種類は取扱説明書（`SRS4000_JA3.pdf` の日計明細の精算の印字例 p.33）から、部門・総売・純売・在高・税の対象額と税額・非課税・高額券の枚数・丸め・取引中止・戻モード・電卓・領収書の類と推定（CV17 の Z001 が印字と同じ行の集合かは未確認）。Z002 に支払以外の行があるかは未確認。Z002 の見出しは語を変えず `支払集計（Z002）` で確定した（SPEC-DRZ-D3、owner 決定 2026-09-27 で Goal を Z001 に絞った）。29 §29.4.1 に匿名化した形で記録した（S6）。
- P2 Z001 / Z002 の各行が精算ごとの差分（同日の複数精算で足してよい値）か、累計・回数・比率か → 結果（2026-09-27、Coordinator 実施）: 取扱説明書（`SRS4000_JA3.pdf` p.57 の表）で日計明細の精算は取引データをクリアする（点検はクリアしない）。したがって Z001 の各行は前回の精算から今回までの件数・金額で、同日の 2 回の精算は足すとその日の値になる、と帳票仕様の根拠では言える。構成比は標準で印字されず、CSV にも個数・金額の列だけ。実物の同日 2 回精算の Z001 は未確認（`未実測`）。Q2 は (a)（合算しない）のため、この未確認は本 lane の設計を止めず、runtime lane の前提にもしない。Z002 の既存の合算は帳票仕様では足せる値だが、合算の鍵 `payment_N` が並び順由来のため、実物の同日 2 回精算で行の集合と並びを確かめることを、backlog の独立した項目「既存の支払集計（Z002）で異なる項目が 1 行に合算されうる」に含めた（S7、本 lane の Scope は広げない）。

## Contract Coverage Ledger

| Design contract / decision ID | Implementation target | Automated test | L3 or non-scope |
|---|---|---|---|
| SPEC-DRZ-D1 / UI-09a-D16（置き場所・列・見出し・行の順・0 の行・「—」） | runtime lane: `DailySalesPage.tsx` | runtime lane: `DailySalesPage.test.tsx`（text / role で確かめる、Matrix R9 / R10 / R13） | L3: 表の長さと読みやすさ（before / after、R15） |
| SPEC-DRZ-D2（複数取込みは取込みごと、古い順、取込み日時の見出し） | runtime lane: `DailySalesPage.tsx`、`sales_repo.rs` | runtime lane: 2 取込みの日の repo / BIZ / UI の test（R1 / R2 / R7 / R11 / R12） | L3: 「合算」と「取込みごと」の違いが伝わるか（R15） |
| SPEC-DRZ-D3（既存表示の不変、見出しの出どころ、合算の文） | runtime lane: `DailySalesPage.tsx` | runtime lane: 既存 `DailySalesPage.test.tsx` の 3 つの完全一致の assert（`2回の取込みを合算`・`支払集計`・`部門別集計`）を UI-09a-D16 の文字列へ更新（意図した変更。金額・NULL・series 分離の assert は保つ）+ 見出しの test（R14）、日次 CSV の不変（R16） | L3: UI-09a-D16 の文字列で読み違えないか（R15） |
| SPEC-DRZ-D4（`summary_imports` と 2 つの型、件数 = `source_import_count`） | runtime lane: `sales_service.rs` | runtime lane: `sales_service.rs` の REQ-501 test（R7） | non-scope（型の契約） |
| SPEC-DRZ-D5（1 query、並び、`rolled_back` を読まない、合算しない） | runtime lane: `sales_repo.rs` | runtime lane: `sales_repo.rs` の test（R1〜R6） | non-scope |
| SPEC-DRZ-D6（command 名・引数不変、bindings 再生成） | runtime lane: `src/lib/bindings.ts`（生成） | runtime lane: L1 の bindings drift 検査と `import_internal_contract_test.rs:222-232` の field 列の更新（R8） | non-scope |
| SPEC-DRZ-D7 / D-096 | 本 lane: `docs/decision-log.md` | AC4（T4） | non-scope |
| UI-09a-D12（公式と商品別を混ぜない）、UI-09a-D13（受入は別）、UI-09a-D15（合算表示）、D-025、D-071 の隣接契約 | 変えない | 既存の test のまま | 受入判定は Non-scope |

## Test Design Matrix

[Test Design Matrix](test-matrices/2026-09-27-daily-report-z-display.md)（R3、2026-09-27）。T1〜T15 は本 lane の検査（AC1〜AC14 と、正本の答えが 2 つにならないかを読む T11）。R1〜R16 は後続 runtime lane が使う test の設計で、Contract Coverage Ledger の各行を repo / BIZ / UI / L3 に展開する（表の出る条件、未取込み、行の無い取込み、複数取込み、取消済みの除外、並び順、「—」の表示、既存 2 表の見出し、日次 CSV の不変）。

## Test Plan

- targeted tests: 本 lane は文書だけ。AC1〜AC4 / AC8 / AC11〜AC14 の `rg`、AC5 の diff の読み取り。
- negative tests: AC3（古い義務が残らない）、AC6（runtime を触らない）、AC10（traceability の差分なし）。
- compatibility checks: 既存の UI-09a-D12 / D13 / D15 と D-025 / D-071 と矛盾しない（review）。
- data safety checks: P1 / P2 の記録に実値・実ラベルが無い（AC7）。
- main wiring/integration checks: `bash scripts/doc-consistency-check.sh` と `--target plan`（AC9）。

## Boundary / Wire Contract

- producer: BIZ-05 `get_daily_sales` → CMD `get_daily_sales`（Tauri、specta）
- consumer: `src/features/daily-sales/hooks/useDailySalesReport.ts`（当日と前日）、`src/features/home/hooks/useHomeSummary.ts`（前日、`official_daily_report` 以外を使う）
- wire type: `DailySalesReport.official_daily_report: OfficialDailyReportSummary | null` に `summary_imports` の配列が増える（配列の要素は `daily_report_import_id` / `imported_at` / `lines`、行は `label` / `quantity` / `count` / `amount`）
- internal type: `sales_repo::OfficialDailyReportRow`（IO の DB DTO）に同じ形の DB DTO（`OfficialDailySummaryImportRow` / `OfficialDailySummaryLineRow`、IO の所有）を field `summary_imports` として足し、BIZ の `map_official_daily_report` が公開 DTO へ写す（24 §14.21）
- precision/range: 数は `i64`（parser が i64 で parse、`daily_report_parser.rs:478-484`）。NULL は「その欄が無い」
- round-trip path: 読み出しだけ（書込み・再取込みの経路は変えない）
- invalid input: 日付の形式不正は既存の `ValidationFailed` のまま
- compatibility: 追加 field だけで既存 field は変えない。bindings を再生成しない古い frontend は新 field を無視するだけ（同じ build で配布するため混在はしない）

## Review Focus

- 冒頭で [Ordinary Operation](#ordinary-operation) が目的を達成できるかを `成立 / 具体的な反例あり / 外部前提が未確認` で答える。
- [現状の事実](#現状の事実2026-09-27e7c22f8f-の現物) の file:line が現物と合うか。とくに「Z001 の総売・純売以外の行を読む本番経路が無い」「Z002 / Z005 は既存の表示のまま（Z002 の既存の合算は backlog の独立した項目）」の 2 点。
- 画面の 3 案の比較と推奨の根拠に、owner の好み・既存の決まり（D-091、DSR-16、04 原則 2 / 3 / 5）に照らした見落としが無いか。
- SPEC-DRZ-D2 / D-096（取込みごとに並べる）が D-071・UI-09a-D15 と両立するか。合算の文の直しで利用者が混乱しないか。
- Non-scope に落とした項目（印刷、受入、月次、取込み履歴と後日の取消、取込み画面）が Goal Invariant の最小完了条件を削っていないか。
- Contract Probe P1 / P2 の問いが、Q2 と Z002 の見出しを決めるのに足りるか。

## 後続 runtime lane への申し送り

- 題: 日次売上の「レジ日報（公式）」に日計（Z001）の表を出す（runtime、R3、Human Gate `ready,merge,manual`）。本 lane の merge 後に起票する。
- Scope の起点: `sales_repo.rs` の aggregate と `OfficialDailyReportRow`、`sales_service.rs` の型と map、bindings の再生成、`DailySalesPage.tsx` の公式 section、各層の test。Contract Coverage Ledger の runtime 行をすべて持つ。
- DailySalesReport を mock している frontend の test の factory（型に field が増えるため）を洗い出して Scope に入れる。既存 `DailySalesPage.test.tsx` の `test_daily_sales_page_req501_shows_source_import_count_without_cross_series_sum` の 3 つの完全一致の assert（`2回の取込みを合算`・`支払集計`・`部門別集計`）は D16 の新しい文・見出しへ更新する（意図した変更。金額・NULL・series 分離の assert は保つ）。
- 日次 CSV 出力（`export_sales_csv` が `get_daily_sales` を使う）は、公式日報と Z001 の行を seed した日でも列・行が変わらないことを確かめる（Matrix R16）。
- `src-tauri/tests/import_internal_contract_test.rs` の `test_wire_contract_req401_i_w1_i_w2_i_w3_i_w5_generated_binding_is_atomic`（`:222-232`）は `OfficialDailyReportSummary` の field 列を完全一致で固定する。既存の禁止事項（単一 parent ID を返さない、旧取込み契約の語を bindings に残さない assert）を保ったまま、`summary_imports` を足した 8 field の契約へ更新する。
- L3: 実機（Windows native、DPI 125% / 150%）で before / after を並べ、表の長さ・「—」と 0 の行の読み取り・複数取込みの見出しと合算の文、取込みの順と精算の順を読み違えないか（「N回目の取込み」は精算の回数ではない）を、UI-09a-D16 の確定した文字列で owner が確かめる（文字列を変えるなら Gated Amendment）。fixture は実 encoding（CP932）の合成 Z001 / Z002 / Z005 を 1 回分と 2 回分用意し、Z001 は総売・純売以外の行を含める（`tests/fixtures/daily-report/` の既存 fixture は Z001 が 2 行だけ）。

## Spec Contract

Contract ID: SPEC-DRZ

| Contract | Test |
|---|---|
| SPEC-DRZ-D1: 公式 section の metric の下に日計（Z001）の表。列 = 名称 / 個数/件数 / 金額、行 = 保存されたラベルを `sort_order` 順に全行、0 の行を含む、NULL は「—」 | review: AC1（本 lane）、runtime lane の UI test |
| SPEC-DRZ-D2: 同日複数取込みの Z001 は取込みごとに古い順で並べ、取込み日時の見出しを付ける。1 回の日は見出しなし | review: AC1、runtime lane の repo / UI test |
| SPEC-DRZ-D3: 既存表示（metric・支払・部門の合算、未取込み note、warning、UI-09a-D12 / D15）を変えず、見出しに出どころを添え、合算の文を直す | review: AC5 |
| SPEC-DRZ-D4: `OfficialDailyReportSummary.summary_imports`、件数 = `source_import_count`、`line_key` を返さない | review: AC2、runtime lane の BIZ test |
| SPEC-DRZ-D5: aggregate が completed の親の Z001 行を 1 query で読み、並べ、合算しない | review: AC2 / AC3、runtime lane の repo test |
| SPEC-DRZ-D6: `get_daily_sales` の名前・引数は不変、bindings は runtime lane で再生成 | review（42 を変えない） |
| SPEC-DRZ-D7: D-096 に Z001 を取込みごとに並べる決定と見直す条件 | review: AC4 |

## Trace Matrix

| Spec ID | Plan Step | Test | Review Focus | Evidence |
|---|---|---|---|---|
| REQ-501 / SPEC-DRZ-D1 | S1 / S4 | doc check AC1 | 画面の契約が 1 つに決まるか | 56 の UI-09a-D16 |
| REQ-501 / SPEC-DRZ-D2 / D7 | S1 / S5 | doc check AC1 / AC4 | D-071 と両立するか | 56、D-096 |
| REQ-501 / SPEC-DRZ-D3 | S1 | doc check AC5 | 既存表示を壊さないか | 56 の diff |
| REQ-501 / SPEC-DRZ-D4 | S2 | doc check AC2 | 型が表示に足り、余計な field が無いか | 34 §19 |
| REQ-401 / SPEC-DRZ-D5 | S3 | doc check AC2 / AC3 | 読み出しの並びと除外 | 24 §14.21 |
| REQ-401 / P1 / P2 | S6 | review AC7 | 実値が無いか | Contract Probe、29 §29.4.1 |

## Data Safety

- commit しない: 実 Z001 / Z002 / Z005 の CSV 本文、金額、実ラベル、店の部門名、スクリーンショット、帳票仕様の PDF。
- repo 外で読むだけ: `inventory-field-check/summaries/` と `approved-readable/` の資料（P1 / P2）。記録するのは行数の範囲・欄の種類・足せるかの別だけ。
- 合成データだけ: runtime lane の fixture（`tests/fixtures/daily-report/` と同じく合成）。

## Implementation Results

本 lane の成果物は設計文書で、`design → plan-draft` の条件（設計の出力が正本にある）として plan-first の change に同乗する（Scope S1〜S8、2026-09-27 Writer run）。runtime の実装は無い。

- Writer（Opus 5.5 subagent）が S1〜S8 を設計正本へ書いた: 56 に UI-09a-D16（日計〈Z001〉の表の置き場所・列・見出し・行の順・0 の行・「—」・同日複数取込みの見出しと並び・合算の文の完全一致の文字列・既存 2 表の見出しの出どころ）、34 §19 に `summary_imports` と 2 つの型、24 §14.21 に Z001 の行を取込みごとに読む手順と古い「実装遷移義務」の削除、§14.24 と biz-task-specs BIZ-08 に D-096 の 1 句、SCREEN_DESIGN §3 に 1 行、decision-log に D-096、29 §29.4.1 に匿名化した Z001 / Z002 の形、backlog に後続 runtime lane と follow-up。
- Final Review で P1/P2 は無く、base 同期の後の closure の P2（packet の Codex の model の根拠）を是正した（両 reviewer の行に owner 決定 2026-09-26 を記し、予算を実数に）。Fable の P3 2 件は後続 runtime lane の起票時に直す（backlog）。
- CI: hosted CI success（https://github.com/higekiri-w/inventory-system-desktop/actions/runs/36336635693）。Ready・merge は owner 指示（2026-09-28）で helper 経由、[PR #114](https://github.com/higekiri-w/inventory-system-desktop/pull/114)。

## Review Response

- Findings Freeze: not yet frozen; post-freeze exceptions: none.

- Closeout（2026-09-28）: Final Review broad（head `d72e1dc5`、互いに独立）は Claude 側 Fable 5.1 = approve（P1/P2 = 0、P3 2）、Codex 側 GPT-6 Astra（発注 112）= approve（P 0）。Fable の P3 は ① 24 §14.21 手順 1 の取得列に `imported_at` が無い（`sales_repo.rs:1019-1023` は `id, report_date, gross_amount, net_amount` だけ）、② 34 §19.2 の `OfficialDailyReportSummary` の field 順が code（`sales_service.rs:77-85`）と違う、で、どちらも後続 runtime lane の起票時に直す（backlog）。main 同期 `2b00d9b9`（#113・#115 を取込み、Plans.md と decision-log の競合は両方残して解消）の後の closure 1（Codex GPT-5.6 Sol、発注 114）= reject P2 1（packet の Codex の model が座組表〈GPT-6 Sol 既定〉と不一致で根拠が無い）→ 是正 `06b0d121`（両 reviewer の行に owner 決定 2026-09-26「DevDay まで GPT-6 Sol を GPT-5.6 Sol で代える」を記し、予算を実数に）→ closure 2（Codex GPT-5.6 Sol、発注 115）= approve（P3 1: Owner Effort Budget に旧日付・旧集計の表記が残る → 本 closeout で表を closeout 時点の実数に直して解消）。helper record は broad 2 本（`d72e1dc5`）と closure（`06b0d121`）が pass、record は owner が 2026-09-28 に Coordinator へ委任した。
