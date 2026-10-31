# Plan Packet: 日報の紙の runtime（日報を 1 日 1 枚の紙にして印刷する、D-120 の実装）

wave に属さない単独の lane（2026-10-27 起票。Coordinator が同時に起票した 4 本の 1 本。owner の方針「質問なしで進められるだけ進める」〈repo 外の回答台帳 TD-401〉と並列の候補の目星〈TD-447〉）。設計は merge 済み（PR #173、[decision-log](../decision-log.md) D-120。設計 lane の packet と Matrix は `docs/archive/plans/2026-10-17-daily-report-sheet-design.md`・`docs/archive/plans/test-matrices/2026-10-17-daily-report-sheet-design.md`）で、本 lane はその契約を実装する。起票で新しい決定 D-142（日次売上と日報の紙の値の安全な整数の範囲と、支払の合算を BIZ の 1 つの関数に置くこと）を決め、正本を本 commit で直した（下の「本 commit で変えた正本」）。branch は `agent/daily-report-sheet-runtime`（起点 main `50f97a76`）。

**実装は SD 直読みの runtime（PR #172、`agent/sd-direct-read-runtime`）の merge の後**。#172 は同じ file（`sales_repo.rs`・`sales_service.rs`・`sales_cmd.rs`・`daily_report_parser.rs`・`lib.rs`・`bindings.ts`）の test の fixture の型（`NewDailyReportImport.settlement_no`・`DailyReportInputFile.sd_relative_path` 等）と `commit_daily_report_import` の引数を変えるので、本 lane の test はその後の形で書く。**#172 の merge の後に `origin/main` を単段で同期してから plan-gate へ進める**（下の「並走との重なり」）。

**規模と手法（1 lane のまま。実装の順を S1〜S7 に固定する）**: 2026-10-27 に本 worktree（HEAD `50f97a76`）で数えた。予定 path は 50（2026-10-29 の round 1 の是正で 41 から数え直した。下の「予定 path の一覧」）: Rust 8（`io/daily_report_parser.rs`・`db/sales_repo.rs`・`biz/sales_service.rs`・`cmd/sales_cmd.rs`・`lib.rs`・`constants.rs`、round 1 で `biz/daily_report_import_service/parse.rs`・`tests.rs`）、TS と CSS 12（既存 7: `DailySalesPage.tsx` とその test・`ExportBar.tsx`・`query-keys.ts` とその test・`globals.css` とその test。新設 4: `DailyReportSheet.tsx` とその test、`ExportBar.test.tsx`、`useDailyReportSheet.ts`。round 1 で既存の `DailyReportImportPage.flow.test.tsx`）、生成物 2（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`）、合成の fixture 13（新設 `tests/fixtures/daily-report-sheet/`。round 1 で商品の CSV と改ページの日の 4 file を足した）、docs 15（packet と Matrix を含む。round 1 で 37）。層は 6 つ（parser の規則〈IO-07-D4a・D6〉と取込みの画面の文〈BIZ-08-D2a〉・IO〈`sales_repo` の読取り 2 つと支払の行を足さずに返す形〉・BIZ〈D2 の関数・紙・画面の支払集計〉・CMD〈1 command〉・画面〈日次売上の文・行の key・印刷ボタン・紙の描画〉・印刷〈`@page`・`@media print`〉）。wire は新しい command 1 つと DTO 3 つ（`DailyReportSheet`・`DailyReportSheetRow`・`DailyReportSheetPluImport`）。既存の wire の型は変えない（IO の DB DTO `OfficialDailyReportRow.payment_lines` の要素の形だけが変わる。wire に出ない）。migration 0（DB の列・制約は変えない。migration を足す必要が出たら足す前に止まる）。画面は既存の日次売上 1 つ（新しい route は無い）。
- 分けない理由: 分ける案（(i) 画面の支払集計の鍵と parser の規則、(ii) 紙）は、どちらも SALES-SHEET-D2 の同じ 1 つの関数（D-142 の (4)）と同じ file（`sales_service.rs`・`sales_repo.rs`・`DailySalesPage.tsx`）を取り合い、(i) だけを先に入れると画面の warning が「紙（日報を印刷）で取込みごとに見られます」と案内する紙が無い main ができる（34 §19.3 手順 3 の文）。L3 も 2 回になる。1 lane で順に積み、L3 を 1 回にする。
- 実装の順: S1 parser（IO-07-D4a・D6）→ S2 IO（`sales_repo`）→ S3 BIZ（D2 の関数・画面の支払集計・紙）→ S4 CMD と登録・bindings → S5 画面と印刷 → S6 設計正本の「proposed・未実装」を現行の記述へ直す → S7 L3 の準備（`cargo check --release`、fixture）。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: 2fca3b1836ffce0a4f120d2123d86dabe893ff5b
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は本 run の起草役。実装は座組表の Writer の行〈`subagent_type: writer`〉）
- Plan Reviewer: fresh Opus 5.5 + Codex（座組表の Plan Reviewer の行。Codex が起動できない日は座組表の Codex 停止中の列）
- Final Reviewer: Fable 5（Claude 側、R3）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge,manual
- Branch: agent/daily-report-sheet-runtime

遷移の記録:

1. kickoff → spec-check（2026-10-27、起草役）: Coordinator の発注 599（D-120 の後続の runtime の起票）を Scope にし、規模を数えて 1 lane のままと決め、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-27、起草役）: 対象の正本は 34 §19.3・§19.3a、24 §14.21・§14.21a・§14.21b、42 §22.4・§22.8、29 IO-07-D4・D4a・§29.4.1・§29.5、56 UI-09a-D13・D15〜D19、UI_TECH_STACK §6.5.5、pos-tables §12d、10 の「入力の上限と安全な整数の範囲」。backlog の「日次売上と日報の紙の wire の i64 を安全な整数の範囲で扱う横断の規則を決める」（本 lane の起票の前に決めると書かれていた）が未決で、正本は紙と画面の支払集計の足し方を別々の層（IO と BIZ）に置いていた。D-142 を決め、同じ commit で正本を直した（下の「本 commit で変えた正本」）。
3. design → plan-draft（2026-10-27、起草役、同じ commit）: 設計の出力は D-120・D-142 と上の正本にあり、未解決の設計の問いは無い。owner の判断事項（下の Open Questions の O1〜O4）は推奨の既定で書き進め、答えで変わる所を名指した。残る外部前提（Contract Probe の P1・P2・P5・P8）は実装の前・L3・D19 で確かめるもので、設計の問いではない。plan-gate へは Coordinator が #172 の merge と main の同期の後に進める。
4. plan-draft のまま是正（round 1、2026-10-29、Coordinator の裁定）: Plan Review round 1（fresh Opus 5.5 と Codex、`REVIEWED_HEAD=ee8065b1`）の findings を Coordinator が全件受理し、発注 669 で起草役が packet・Matrix・設計正本を直した（下の Review Response）。Phase と Plan Commit は変えない。
5. plan-draft → plan-approved（2026-10-29、Coordinator）: 計画の review の closure（round 2、`2fca3b18`）は fresh Opus 5.5・Codex とも approve（P1/P2 0、P3 だけ。PR #211 の comment「計画の review の closure（round 2）」）。`scripts/plan-approve.sh` で Plan Commit を `2fca3b18` に固定した（`f007d635`）。旧形式の packet は Phase が承認の前の値の間は承認の前と扱われる（`docs/DEV_WORKFLOW.md` の「承認の後の計画の変更」）ので、Phase の行を本 commit で進めた。implementing へは #172 の merge と同期の後に Coordinator が進める（packet の順）。
6. plan-approved → implementing（2026-10-31、Coordinator）: #172 が main に入り（`25d106a9`）、`origin/main` を単段の merge で取り込んだ（`1f84ae13`。衝突は 37 の更新履歴の表だけで、両方の行を日付の順に残した）。AC10 の前提（#172 の merge の commit が HEAD の祖先）が成り立った。`Plan Commit` と `Amendments` は変えない。実装の Writer は発注 694。

## 本 commit で変えた正本

| file | 節 | 変えたこと |
|---|---|---|
| `docs/decision-log.md` | 末尾 | D-142 を足した |
| `docs/function-design/10-common-rules.md` | 「入力の上限と安全な整数の範囲」の (a) の表、(e)、更新履歴 | (a) の表にレジの日報の金額・件数・個数の行（IO-07-D6）。(e) の「売上の金額」の項を、表の外に残す手動販売の金額欄と、上限が別の所にあり読取りで拒まないレジの日報・Z004 の値に分けた |
| `docs/function-design/29-io-daily-report-parser.md` | §29.2 の IO-07-D6（新設）、IO-07-D4 の `credit` の段の参照、§29.5 の `invalid_number` の行 | 数の上限と文（`…の値が大きすぎます`）。支払の足す鍵の参照を 34 §19.3 手順 3 へ |
| `docs/function-design/34-biz-sales-service.md` | 冒頭の「数量と単位」、§19.3 手順 3、§19.3a の SALES-SHEET-D2（前文と「足す値」）、更新履歴 | レジの日報の値の範囲。支払は IO が足さずに返した行に BIZ-05 が紙と同じ関数で D2 を当てる。互換の範囲の正本を 24 からこの段へ移した。足す値の溢れを紙と画面とも `ValidationFailed` |
| `docs/function-design/24-io-csv-import-repo.md` | §14.21 の関数要求と手順 3・5・7、更新履歴 | 支払の行を足さずに返す（足し方・代表・互換の範囲・warning は 34 §19.3 手順 3） |
| `docs/function-design/56-ui-daily-sales.md` | UI-09a-D16 の支払集計の段 | 互換の範囲の参照を 34 §19.3 手順 3 へ |
| `docs/db-design/pos-tables.md` | §12d の設計意図の「同じ日の取込みをまたいで足す鍵」 | 画面の側の参照を 34 §19.3 手順 3 へ |
| `docs/backlog.md` | 「日報の紙の runtime」、「日次売上と日報の紙の wire の i64 …」 | 前者に起票と packet、Z004 の取込みが b1 で再開済みの依存の文。後者を D-142 で決めた旨と残り（手動販売の金額欄の上限）の項に置き換えた |

D-120 の本文（Compatibility が引く「24 §14.21 手順 3」）は書き換えず、D-142 の Decision (4) が引き先の移動を記録する（decision-log の過去の決定を書き換えない）。

round 1 の是正の commit（2026-10-29）で変えた正本:

| file | 節 | 変えたこと |
|---|---|---|
| `docs/decision-log.md` | D-142（本 lane の未 merge の決定） | 題に照合の置き場を足し、Status・Decision (1)・新しい (5)・Why の (5)・Revisit を直した。(1) は上限の越えを `number_too_large` にし取込みの画面の文が帳票と列を名指すこと（BIZ-08-D2a）。(5) は D-120 の Guarantee range の未確認の前提（レジスターツールの日報との行ごとの一致）を確かめる場を UI-09a-D19 の (2) へ移すこと（O2 の既定 a。Ready の前に owner の明示の答え）。D-120 の本文は変えない |
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.3 手順 3 の BIZ-08-D2a（新設）、§37.7 の行、更新履歴 | 上限の越えの利用者向けの文（帳票と列を名指し、値・file 名・行番号を出さない。`settlement_mismatch` の文の次に優先） |
| `docs/function-design/29-io-daily-report-parser.md` | IO-07-D6、§29.5 の表、§29.4.1 の「行の構成と性質」の末尾 | 上限の越えの `error_type` を `number_too_large` にし文の作り方（値を入れない）を書いた。§29.5 の `invalid_number` の行を main の形に戻し `number_too_large` の行を足した。レジスターツールとの照合の置き場を UI-09a-D19 の (2) へ |
| `docs/function-design/10-common-rules.md` | (a) の表のレジの日報の行、更新履歴 | 誤りの種類を `number_too_large` に、画面の文は BIZ-08-D2a |
| `docs/function-design/34-biz-sales-service.md` | §19.3a の「精算をまたいで足してよい根拠」、更新履歴 | 照合の置き場を UI-09a-D19 の (2) へ（D-142 の (5)） |
| `docs/function-design/24-io-csv-import-repo.md` | §14.21 手順 6 の命名の例・手順 8、更新履歴 | 手順 8 を内部の支払の行・公開する支払の集計・Z001 の単純な写像の 3 つに分けた。命名の例を無くなる `OfficialDailyPaymentRow` から `OfficialDailyDepartmentRow` へ |
| `docs/function-design/56-ui-daily-sales.md` | UI-09a-D18・D19 の (2)、更新履歴 | P1 を「実装の後の L3（L3-1）で確かめる」に。D19 の (2) がレジスターツールとの照合を兼ねること |
| `docs/UI_TECH_STACK.md` | §6.5.5 の未確認の前提、§7.4 更新履歴 | P1 を「実装の後の L3（L3-1）で確かめる」に |
| `docs/backlog.md` | 「日報の紙の runtime」 | P1 を L3-1 に、照合を UI-09a-D19 の (2) に（D-142 の (5)） |

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 0（2026-10-27 起票時。O1〜O4 は推奨の既定で進め、問わない〈TD-401〉） | 4（L3 の 1 round 1〈L3-1〜L3-3 を 1 回の Windows の操作でまとめる。O1・O3 の答えも同じ回の PASS / FAIL と記号で得る〉、O2・O4 の既定の確認 1〈L3 の依頼と同じ 1 回で見せる〉、Ready 1、merge 1） | 2（L3 の 2 round 目 1、P1 が効かず代わりの道〈UI-09a-D18〉を採るときの店の PC の `.html` の関連付けの確かめ 1） | 6 = 0 + 4 + 2 |

既定値・数え方は `docs/DEV_WORKFLOW.md` `Owner Effort Budget`。承認依頼は `この change での介入 N 回目 / 予算 6 回` と「承認すると、日次売上の画面の `日報を印刷` で、その日の日報を 1 日 1 枚の紙に印刷できるようになる」の 1 文。L3 は D-124 のとおり、本 PR が merge の train の先頭に来て最新の `origin/main` を取り込んだ head で頼む。owner の操作を減らす工夫: (1) L3 の入力物（合成の商品の一括 import の CSV、日報の束と Z004）は tracked の fixture にし、Coordinator が取り込む順の手順と一緒に渡す（owner は商品の一括 import と取込みの画面で file を選ぶだけ。DB への直接の挿入を求めない）。(2) 紙の並びは既定の案 a だけを印刷し、読みにくいときだけ b・c を名指してもらう（O1）。(3) レジスターツールの日報との値の照合は、運用開始の後の UI-09a-D19 の (2) の確かめに寄せ、本 PR の L3 にしない（O2、D-142 の (5)。受容リスクは owner が持つので、L3 の依頼と同じ 1 回で owner の明示の答えを得て、Ready の前の条件にする〈AC12〉）。(4) 印刷の手段の確かめ（P1）は L3-1 で同じ回に行い、実装の前に owner の操作を足さない（下の Contract Probe の P1）。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行のうち「report output」（日報の紙の内容と並び）、「POS CSV」（日報の parser の `credit` の規則と数の上限〈IO-07-D4a・D6〉）、「Tauri command DTO」（`get_daily_report_sheet` と DTO 3 つ）、「operator workflow」（締めの後の印刷、日次売上の支払集計の足し方と文）に当たる。R4 ではない: 読み取りだけで（紙は保存しない、SALES-SHEET-D6）、DB の列・migration・データの寿命に触れず、parser の上限は取込みを拒む向きだけで保存済みの値を変えず、実データを repo に入れない（test と L3 の入力は合成）。
この変更でどの required gate の green / red も変わらない: 製品の code・test・docs と合成の fixture だけで、`scripts/ci/classify-changes.sh` の workflow の一覧（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/*`・`docs/templates/*`・`.agents/*`・`.claude/{rules,commands,skills}/*`・PR template）と `scripts/*`・`.github/*` に当たる予定 path は無い。Final Review Minimum 1。operator に見える状態の lifecycle（紙の内容の読込み → 押したときの読み直し → 印刷）を足すので、Contract Audit の 2 本目は推奨（`docs/DEV_WORKFLOW.md` Contract Audit の Double audit。採るかは Coordinator）。Windows の実機の印刷を確かめるので Human Gate に `manual`。

## Goal

Goal Invariant:

### 最小完了条件

- 日次売上の画面の `日報を印刷` で、その日の完了した日報の取込みを位置と名称の組で足した 1 日 1 枚の紙（SALES-SHEET-D1〜D6）を、Windows の実機のプリンタで A4 縦・文字 9 pt 以上・色なしで印刷できる（UI-09a-D17・D18）。PLU の部分はその日に売れた Z004 の行だけが載る。押したときに読み直し、古い内容で印刷しない。印刷できない日（未取込み・読込みの失敗）は、ボタンの下にその理由とその場で直せるリンク・`再試行` が出る。
- 日次売上の画面の支払集計が紙と同じ規則（SALES-SHEET-D2）で、BIZ-05 の同じ 1 つの関数で足され、名称か並びの変わった日・足さない日は warning の注記で知らせる（34 §19.3 手順 3、UI-09a-D16）。
- parser が `credit` の鍵を導かず（IO-07-D4a）、上限を越える数の束を拒み、取込みの画面がその束の誤りとして帳票と列を名指す（値は出さない。IO-07-D6、BIZ-08-D2a、D-142）。
- 設計正本の D-120・D-142 の「proposed・未実装」の注記が、実装した現行の記述になっている。

### 失敗定義

- 紙が精算ごとに分かれる、名称の差し替えの日に別の項目を 1 行に足す、行がずれ同じ名称の行がある帳票を組で足す（SALES-SHEET-D2）。
- 画面の支払集計と紙で足し方が食い違う、または D2 の実装が 2 つある。
- 押した時点より古い内容で印刷する、読み直しの失敗・`import_count` 0 でも印刷する。
- 紙の文字が 9 pt を割る、色を使う、画面に紙の内容が出る、アプリの枠を印刷する。
- 既存の test を不都合だから削除・skip・弱体化して通す（IO の支払集計の test の期待は、34 §19.3 手順 3 の互換の範囲の中の入力は同じ値で、範囲の外の入力は同じ段の「範囲の外の入力は新しい規則の値」を出所に新しい規則の値で BIZ の test へ移す。正本の変更で期待が変わる test と、その検出力を保つ範囲の中の test は Matrix の「既存の test の移し方」が名指す。下の Scope の S2）。
- 実データ・店の名称の値・金額を repo に入れる。

### 非目的

- 紙の並びの最終の決定（O1。既定の案 a を実装し、L3 で owner が読みやすさを判じる）。紙の切替えの日（UI-09a-D19。運用開始の後に owner が店と決める）。
- 月次の印刷（月次の `ExportBar` の印刷は disabled のまま。backlog の「code の comment・文言の古い句 2 つ」の (1) の月次の分は残る）。
- 手動販売の金額欄の上限（D-142 の (3)、backlog の「手動販売の金額欄の上限」）。
- 日次売上の画面の Z001 を 1 つの表に足すこと（D-096 のまま）。
- 取込みの経路・同日追加の確認・SD の読取り（D-111・D-121、#172 と後続の lane）。
- 代わりの道（`.html` を既定のアプリで開く、UI-09a-D18）の実装。P1 が効かないと分かったときだけ、42 に command を足す Gated Amendment で入れる。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更（operator の操作と command 契約）なので、締めの日の操作列を置く。本 lane の merge で「アプリから紙を印刷できる」は成り立つが、店が Excel への貼付けをやめる（紙の切替え）は UI-09a-D19 の運用の段で、本 lane の完了とは別。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 本 lane を入れた版。締めの後、その日の日報を取り込んだ（今の取込みの画面） | 日次売上を開く | 印刷ボタン `日報を印刷` が押せる。紙の内容は画面に出ない | — | — |
| 同じ | `日報を印刷` を押す | 紙の内容を読み直し、OS の印刷の dialog が出る。プレビューは A4 縦（上に題・日付・印刷の時刻、左に日計と部門、右に支払、下に PLU）で、アプリの枠は出ない | 印刷先を選んで印刷 | P1（WebView2 の印刷）、P5（1 枚に収まるか） |
| 同じ日に 2 回精算して 2 回取り込んだ日 | 日次売上を開き、`日報を印刷` | 画面の支払集計は組で足した行。紙は 2 回分を足した 1 枚で、題の行に `2回の取込みを合算` | — | P2（レジスターツールの日報と同じ値か。D19 の (2)） |
| 精算の間で Z002 の行がずれ、1 回の精算の中に同じ名称の行が 2 行ある日（手元の集計では起きていない） | 日次売上を開く | 支払集計は足さずに取込みの古い順の行。公式セクションの注記（見出し `日報の確認が必要です`）に D2 の 3 の文。ボタンの下に `支払集計（Z002）：…` の文 | `日報を印刷` で取込みごとの表を見る | O3（2 つの知らせを残すか） |
| まだ取り込んでいない日 | 日次売上を開く | ボタンは押せず、下に `この日のレジ日報は未取込みのため、印刷できません。` とリンク `売上データ取込みを開く` | リンクから取り込む → 戻ると押せる | — |
| 画面を開いた後に、別の画面でその日の取込みを取り消した | `日報を印刷` | 読み直しで `import_count` が 0 になり、印刷せず未取込みの文が出る | 取り込み直す | — |
| 紙の内容の読込みが失敗した | 日次売上を開く | ボタンは押せず、下に CmdError の文と `再試行` | `再試行` を押す | — |
| Z004 を取り込んだが、商品に当たらない行がある日 | 日次売上を開き、`日報を印刷` | 画面に `数えていない PLU の行が {N} 行あります` とリンク `取込みの記録を開く`、紙の PLU の下に同じ数の文。印刷は止めない | リンクから取込みの記録で行の理由を見る | — |
| 上限を越える数を持つ日報の束（壊れた・作られた file） | 売上データ取込みで選ぶ | 取り込めず、取込みの画面の誤りの文に `Z001の金額列の値が大きすぎます。この日報ファイルは取り込めません。レジの日報ファイルを選び直してください。` のように帳票と列が名指される（値・file 名・行番号は出さない。BIZ-08-D2a） | 正しい file を選び直す | P8（実データは上限のはるか下か） |
| 翌日 | 前の日を選んで `日報を印刷` | いつでも過去の日の紙を作り直せる（保存しない、SALES-SHEET-D6） | — | — |

## Scope

本 lane の範囲は D-120 の runtime（backlog の「日報の紙の runtime」の範囲）と D-142 の実装。下の S1〜S7 の file の、下の所だけを変える。

### S1 parser（IO-07-D4a・D6）

- `src-tauri/src/io/daily_report_parser.rs`: `payment_key` から `クレジット` / `credit` の分岐を外す（`現金` → `cash` と並び順の `payment_N` は残す）。`parse_optional_i64`・`parse_optional_hundredths` に上限の検査を置き、形の誤りと上限の越えを区別して返す（例: `enum NumberError { Invalid, TooLarge }`）。呼ぶ所（Z001・Z002・Z005 の各列の `match`。`parse_required_i64` も `parse_optional_i64` を通る）は `TooLarge` のとき `error_type` `number_too_large` の誤りに `{帳票}の{列}の値が大きすぎます` の文（帳票と列の語だけ。値・file 名・行番号を入れない）を入れ、`Invalid` のときは今の `invalid_number` と `…を変換できません` の文のまま（29 IO-07-D6・§29.5）。
- `src-tauri/src/biz/daily_report_import_service/parse.rs`（round 1 で足した）: `parse_and_validate_daily_report`（#172 の後は `_with_sd_lookup`）の `parse_errors` の分岐で、今の `settlement_mismatch`（BIZ-08-D2）の次に `number_too_large` の分岐を 1 つ足し、37 BIZ-08-D2a の文（IO の `error_message` に `。` を付け、重複を除き、帳票と列の順につなぎ、`この日報ファイルは取り込めません。レジの日報ファイルを選び直してください。` を続ける）を利用者向けの message と `operation_logs.summary` にする。他の `error_type` は今の汎用の文のまま（BIZ-08-D1）。CMD（`daily_report_import_cmd.rs` の `ImportError` → `CmdError` の `import_error`）と画面（`DailyReportImportPage.tsx` の `describeError`）は文をそのまま運ぶので変えない。#172 の branch の同じ file（`git show origin/agent/sd-direct-read-runtime:src-tauri/src/biz/daily_report_import_service/parse.rs`）も同じ分岐（`settlement_mismatch` → 汎用の文）を持つので、#172 の merge の後の形にそのまま 1 つ足せる（2026-10-29 に読んで確かめた）。
- `src-tauri/src/biz/daily_report_import_service/tests.rs`（round 1）: Matrix の BIZ-08-D2a の 2 本を足す。既存の `test_daily_report_req401_parse_error_logs_parse_failed`（形の誤り → 汎用の文）の期待は変えない。
- `src/features/daily-report-import/DailyReportImportPage.flow.test.tsx`（round 1）: Matrix の BIZ-08-D2a → UI の 1 本を足す（`commands.parseAndValidateDailyReport` を mock する既存の形）。
- `src-tauri/src/constants.rs`: `MAX_DAILY_REPORT_INTEGER: i64 = 9_999_999_999` と `MAX_DAILY_REPORT_QUANTITY_HUNDREDTHS: i64 = 999_999_999`（名前は Writer が決めてよい。値は 29 IO-07-D6）。
- 既存の test: parser の fixture の `クレジット` の行（`daily_report_parser.rs` の test の bundle の文字列 2 か所、`biz/daily_report_import_service/tests.rs` の bundle の文字列）は `payment_key` を assert していない（`rg -n '"credit"' src-tauri/src` は `daily_report_parser.rs` の `payment_key` の 2 行と `db/sales_repo.rs` の test の 2 行〈`payment_key: "credit"` の fixture と assert〉だけ。2026-10-27）ので変えない。`sales_repo.rs` の test の `"credit"` は保存済みの鍵の fixture の値として残す（S2）。

### S2 IO（24 §14.21 手順 3・§14.21a・§14.21b）

- `src-tauri/src/db/sales_repo.rs`:
  - `get_completed_daily_report_aggregate` の支払の読取りを、足さずに返す形にする（親の `id`・`imported_at`、行の `id`・`sort_order`・`label`・`payment_key`・`amount`・`count`。親の `imported_at ASC, id ASC`・行の `sort_order ASC, id ASC`）。今の `PaymentGroup` の集約の code を消す。`OfficialDailyReportRow.payment_lines` の要素の型を行の型（例 `OfficialDailyPaymentLineRow`）にする。部門・Z001・gross / net は変えない。
  - `get_daily_report_sheet_lines`（新設、§14.21a）と DTO `DailyReportSheetLinesRow`・`DailyReportSheetLineRow`。
  - `get_active_z004_import_totals`（新設、§14.21b）。
- 作る所・読む所（2026-10-27 に `rg -n 'OfficialDailyReportRow \{' src-tauri/src src-tauri/tests` → 構造体の定義と `get_completed_daily_report_aggregate` の構築の 2 行だけ）: 読む所は `biz/sales_service.rs` の `map_official_daily_report` だけ（`rg -n 'payment_lines' src-tauri/src/biz/sales_service.rs`）。
- 既存の test の移し方（弱体化しない）: `sales_repo.rs` の test のうち支払の集約の値を assert する `test_completed_daily_report_aggregate_req501_all_active_parents`・`test_get_completed_daily_report_aggregate_req501_propagates_parent_and_line_nulls`・`test_get_completed_daily_report_aggregate_req501_groups_identities_deterministically`・`test_get_completed_daily_report_aggregate_req501_aggregate_overflow_is_error` は、IO の側では「足さずに返す行の並びと値」を assert する形に直し、支払の足した値の期待は assert の単位で BIZ の test（S3）へ移す（Matrix の「既存の test の移し方」の表。2026-10-29 の round 1 で直した）。4 本の支払の fixture はどれも 34 §19.3 手順 3 の互換の範囲の外か支払の行を持たない: `all_active_parents` は範囲の外だが新しい規則でも並びと値が同じ（2 の warning が 1 件増える）、`propagates_parent_and_line_nulls` と `groups_identities_deterministically` は範囲の外で、同じ段の「範囲の外の入力は新しい規則の値」を出所に D2 の 2 の 2 行と 2 の warning 1 件の期待へ変える（**正本の変更で期待が変わる**。弱体化でない）。`aggregate_overflow_is_error` は支払の assert がもともと無い（`rg -n 'i64::MAX' src-tauri/src/db/sales_repo.rs` の当たりは部門と Z001 だけ）ので移すものは無い。NULL の伝播・代表の選び方・溢れの検出力は、範囲の中の fixture の BIZ の test（`test_daily_sales_payment_overflow_and_null_order`・`test_daily_sales_payment_representative_in_range_is_first_row`）と D2 の関数の unit test が持つ。部門・gross / net・Z001・月次の assert は変えない。

### S3 BIZ（34 §19.3 手順 3・§19.3a）

- `src-tauri/src/biz/sales_service.rs`:
  - SALES-SHEET-D2 の関数を 1 つ置く（入力: 1 つの帳票の行の列〈取込みの id・取込みの並びの位置・位置・名称・値〉と取込みの id の並び。出力: 1・2・3 のどれか、足した行か取込みごとの行、溢れは `ValidationFailed`「売上集計の計算でオーバーフローが発生しました」〈今の `get_daily_sales` の溢れの文〉）。画面の支払集計と紙の 3 つの帳票が同じ関数を呼ぶ。代表の行（返す `label`・`payment_key`）は group の最小 `sort_order`、同値なら最小の行 id（34 §19.3 手順 3）。
  - `map_official_daily_report`（または `get_daily_sales` の手順 3）: IO の支払の行に D2 の関数を当て、`OfficialDailyPaymentLine` へ写す。2・3 の日の warning を部門の warning の後に 1 件足す（文は 34 §19.3 手順 3 の完全一致）。
  - `get_daily_report_sheet`（新設、§19.3a）と型 `DailyReportSheet`・`DailyReportSheetRow`・`DailyReportSheetPluImport`（`#[derive(specta::Type)]` 等は既存の DTO と同じ）。PLU の部分は `get_daily_sales_records` の `source='auto'` で数量か金額が 0 でない行。
- 既存の test: `sales_service.rs` の `test_get_daily_sales_includes_official_report_req501`・`test_get_daily_sales_warnings_unmatched_department_req501` は IO の DTO の形の変更に合わせて fixture を直す（期待値は変えない）。

### S4 CMD と登録・bindings（42 §22.4・§22.8）

- `src-tauri/src/cmd/sales_cmd.rs`: `get_daily_report_sheet`（`#[tauri::command]` + `#[specta::specta]`、§22.3 の既存の変換）。
- `src-tauri/src/lib.rs`: `collect_commands!` と `generate_handler!` の CMD-09 の塊に 1 行ずつ。
- `src/lib/bindings.ts`: `cd src-tauri && cargo run --bin generate_bindings` で再生成（手で編集しない。衝突は手で解かず、取り込んだ後に再生成する、D-098）。

### S5 画面と印刷（56 UI-09a-D16・D17・D18）

- `src/features/daily-sales/DailySalesPage.tsx`: 2 回以上の日の文（56 UI-09a-D16 の表の新しい文）、warning の注記の見出し `日報の確認が必要です`、支払集計の行の key `${payment_key}-${index}`、`ExportBar` へ紙の query の状態を渡す、紙の component を置く。
- `src/features/daily-sales/components/ExportBar.tsx`: 印刷ボタンを `日報を印刷` にし、56 UI-09a-D17 の表の 4 つの状態（読込み中・0 件・失敗・1 件以上）の disabled と文・リンク・`再試行`、`label_changed_sources` の帳票ごとの文、`skipped_count` の文とリンク。押したときの順（`await refetch()` → 失敗か 0 件なら印刷せず文 → 描き直し → DOM の反映の後に `window.print()`）。file の頭の comment「Phase 4 で window.print 検討」と Tooltip「準備中（Phase 4 で実装予定）」を消す。
- `src/features/daily-sales/components/DailyReportSheet.tsx`（新設）: 紙の内容（題・日付・印刷の時刻、2 段と PLU、`thead` 2 行、知らせ、`（名前なし）`、`¥` なしの金額、`import_seq` ごとの表）。`document.body` の直下へ portal で置き、画面では隠し印刷のときだけ出す。
- `src/features/daily-sales/hooks/useDailyReportSheet.ts`（新設）: `commands.getDailyReportSheet(date)`、query key `queryKeys.dailyReportSheet(date)` = `["daily-sales", "sheet", { date }]`。
- `src/lib/query-keys.ts`・`src/lib/query-keys.test.ts`: key を足し、`dailySalesRoot()` の prefix に乗ることの test を足す（取込み・取消の invalidate は既存の `invalidationContract` の `dailySalesRoot()` の entry に乗る。`invalidation-contract.ts` は変えない。手動販売の `manualSale` は `dailySales(saleDate)` だけを invalidate するが、紙は手動販売を載せないので足さない）。
- `src/styles/globals.css`・`src/styles/globals.test.ts`: `@page { size: A4 portrait; margin: 10mm }` と、印刷のときにアプリの root（`#root`）を隠す `@media print`。test は literal の oracle を 1 つ足す（既存の SC1 と同じ形）。
- 新しい表示文言と test の matcher の衝突（2026-10-27 に `rg -n -F '<語>' src --glob '*.test.*'`）: `日報を印刷`・`日報の確認が必要です`・`この日のレジ日報は未取込み`・`売上データ取込みを開く`・`取込みの記録を開く`・`数えていない PLU の行`・`売上日報` は 0 件。置き換える旧い文言の matcher は `DailySalesPage.test.tsx` の `日報の部門確認が必要です`（3 か所）と 2 回以上の日の文（2 か所。完全一致の文と helper の文）で、新しい文へ直す（56 の表から転記）。`再試行` は別の画面の test だけで、日次売上の test に無い。紙は画面で隠すが jsdom は CSS を当てないので、紙の文字列（`支払集計（Z002）　2026年…` 等）が既存の `getByText("支払集計（Z002）")`（完全一致）・`getByRole("heading", …)`（紙は heading を使わない）に当たらないことを Writer が既存の test で確かめる。`DailySalesPage.test.tsx` の `vi.mock("@/lib/bindings")` の `commands` に `getDailyReportSheet` を足す。`getDailySales` を mock する他の test（`HomePage.test.tsx`・`useHomeSummary.test.tsx`・`BackupRestorePage.flow.test.tsx`）は日次売上の画面を描かないので変えない見込み（Writer が実行で確かめ、落ちたら同じ形で mock を足す）。

### S6 設計正本の注記

- 24・29・34・42・56・10・UI_TECH_STACK §6.5.5・pos-tables §12d・`docs/SCREEN_DESIGN.md` の日次売上の段の、D-120・D-142 の「proposed・未実装」を現行の記述に直す（下の AC7。更新履歴の行は残す）。
- `docs/architecture/ui-task-specs.md` の UI-09a の印刷ボタンの行（「仕様未定のため Phase 2 では disabled」）を `日報を印刷`（UI-09a-D17）に直す。UI-09b（月次）の行は変えない。
- 56 の §56.1 のファイル構成の `ExportBar` の行・§56.7 の `Tooltip` の行・§56.13 の「印刷ボタンの本実装」の行と更新履歴。
- 29 の IO-07-D4 の段の今の `credit` の規則を書いた 2 行（`gross_sales` / `net_sales` / `cash` / `credit` はラベルだけで決める〈… クレジット → `credit` …〉の行と、`credit` の鍵は全角の `クレジット` を含むラベルだけ、の行。2026-10-29 に `rg -n 'クレジット → .credit.|.credit. の鍵は全角' docs/function-design/29-io-daily-report-parser.md` → 117・119 行目）を、IO-07-D4a の後の現行の記述（`credit` の鍵は導かない）に直す（AC7）。
- `src-tauri/src/db/sales_repo.rs` の `OfficialDailyPaymentRow`（2026-10-29 に `rg -n 'OfficialDailyPaymentRow' src-tauri/src` → 定義 :1111・`OfficialDailyReportRow.payment_lines` の型 :1088・構築 :1256 の 3 行）を、支払の行の型（S2）に置き換えて消す（AC3）。
- `docs/function-design/90-traceability.md` は新しい test の REQ の参照の後に再生成（AUTO-GENERATED）。
- backlog の「日報の紙の runtime」と「code の comment・文言の古い句 2 つ」の (1) の日次の分は closeout で閉じる（本 lane の PR では backlog の完了の整理をしない。D-124 の共有の path を減らす）。

### S7 L3 の準備

- `tests/fixtures/daily-report-sheet/`（実装で作る。新設、合成だけ。全て CP932・CRLF。作り方と確かめ方は `tests/fixtures/z004/README.md` の「変更時の生成手順」「生成後の verification」と同じ）: 2026-10-29 の round 1 の是正で、L3 に到達できる入力の一式にした（空の DB には商品が無く `get_daily_sales_records` は `INNER JOIN products` なので、商品を入れないと PLU の行が出ない。b1 の後の Z004 は精算の識別メタの無い file を拒む〈`tests/fixtures/z004/README.md` の冒頭、32 の表の順 1〉）。
  - `README.md`: 合成であること、取り込む順（Matrix の Manual の (1)〜(6)）、各 file の役、各 Z004 の商品に当たる行の数と当たらない行の数、`2026-03-26` の束が改ページを起こす行の数。
  - `products-sheet.csv`（商品の一括 import）: b1 の `products-new.csv`（`src-tauri/src/io/l3_fixtures.rs` の `PRODUCT_HEADER` と `product_line`）と同じ形（見出し `商品コード,商品名,部門ID,売価,原価,税率,JANコード,在庫単位,初期在庫,POS在庫連動`、部門ID は作り直した空の DB にある初期の部門 `1`、在庫単位 `pcs`、POS在庫連動 `1`）。合成の商品 80（商品コード `SHEET-P01`〜`SHEET-P80`、商品名 `合成P01` の形、JAN は `29` で始まる店内の範囲の合成の 13 桁で check digit が合う〈`l3_fixtures.rs` の `jan` と同じ作り方〉、初期在庫は売る数より多い）。
  - 支払集計を足さない日（D2 の 3。Z002 の行がずれ同じ名称の行がある 2 取込み）の `2026-03-25` の Z001・Z002・Z005 の 2 束（6 file、layout A。精算回数 `0001`・`0002`）。
  - `Z004_260324.CSV`（2 回取り込んだ日〈既存の `tests/fixtures/daily-report-z001/` の `2026-03-24`〉の Z004）: layout A（`l3_fixtures.rs` の `z004` と同じ前文。マシンNo. `0001`、精算回数 `0001`、日付 `2026-03-24`、時刻 `20:00`）で、商品に当たる行 5（`SHEET-P01`〜`P05` の JAN、数量と金額は 0 でない）と、どの商品にも当たらない JAN の行 1。
  - `2026-03-26` の束（改ページの日。layout A、精算回数 `0001`。`Z001_260326_0001.CSV`・`Z002_260326_0001.CSV`・`Z005_260326_0001.CSV`）: Z001 30 行（1・2 行目は `総売`・`純売`、他は `項目01` の形）、Z002 80 行（1 行目 `現金`、他は `支払01` の形）、Z005 60 行（`部門01` の形）。名称に番号を入れ、L3-2 で紙の境の前後の行が続いているかを番号で見られるようにする。
  - `Z004_260326.CSV`: layout A（マシンNo. `0001`、精算回数 `0002`〈`Z004_260324.CSV` の次の番号。32 の表の順 5 と欠けの知らせに当たらない〉、日付 `2026-03-26`）で、`SHEET-P01`〜`P80` の行 80（全部商品に当たる）。
  - 改ページが必ず起きる理由（56 UI-09a-D17 の見積りの前提: 行の高さ 4.4〜5 mm、印刷できる高さ 277 mm で 1 枚に約 55〜63 行）: 左の段（Z001 30 行と Z005 60 行と見出しの 4 行）と右の段（Z002 80 行と見出しの 2 行）がどちらも 1 枚目を越えて 2 枚目に続き、PLU の表（80 行と見出しの 2 行）は 2 枚目の左の段の後から始まり 3 枚目へ続く。行の高さが 3.5 mm（1 枚に約 75 行）でも、両段と PLU がどれも紙をまたぐ。
- Writer は L3 の前に `cargo check --release`（`docs/DEV_WORKFLOW.md` Implementation Rules）。

### 予定 path の一覧（`comm -12` の入力）

50 path（Rust 8・TS と CSS 12・生成物 2・fixture 13・docs 15。2026-10-29 の round 1 で 9 を足した）。所有表の突き合わせは L3 を頼む時点の head で取り直す（D-124）。

```
docs/SCREEN_DESIGN.md
docs/UI_TECH_STACK.md
docs/architecture/ui-task-specs.md
docs/backlog.md
docs/db-design/pos-tables.md
docs/decision-log.md
docs/function-design/10-common-rules.md
docs/function-design/24-io-csv-import-repo.md
docs/function-design/29-io-daily-report-parser.md
docs/function-design/34-biz-sales-service.md
docs/function-design/37-biz-daily-report-import-service.md
docs/function-design/42-cmd-sales-stocktake.md
docs/function-design/56-ui-daily-sales.md
docs/function-design/90-traceability.md
docs/plans/2026-10-27-daily-report-sheet-runtime.md
docs/plans/test-matrices/2026-10-27-daily-report-sheet-runtime.md
src-tauri/src/biz/daily_report_import_service/parse.rs
src-tauri/src/biz/daily_report_import_service/tests.rs
src-tauri/src/biz/sales_service.rs
src-tauri/src/cmd/sales_cmd.rs
src-tauri/src/constants.rs
src-tauri/src/db/sales_repo.rs
src-tauri/src/io/daily_report_parser.rs
src-tauri/src/lib.rs
src/features/daily-report-import/DailyReportImportPage.flow.test.tsx
src/features/daily-sales/DailySalesPage.test.tsx
src/features/daily-sales/DailySalesPage.tsx
src/features/daily-sales/components/DailyReportSheet.test.tsx
src/features/daily-sales/components/DailyReportSheet.tsx
src/features/daily-sales/components/ExportBar.test.tsx
src/features/daily-sales/components/ExportBar.tsx
src/features/daily-sales/hooks/useDailyReportSheet.ts
src/lib/bindings.ts
src/lib/query-keys.test.ts
src/lib/query-keys.ts
src/styles/globals.css
src/styles/globals.test.ts
tests/fixtures/daily-report-sheet/README.md
tests/fixtures/daily-report-sheet/Z001_260325_0001.CSV
tests/fixtures/daily-report-sheet/Z001_260325_0002.CSV
tests/fixtures/daily-report-sheet/Z002_260325_0001.CSV
tests/fixtures/daily-report-sheet/Z002_260325_0002.CSV
tests/fixtures/daily-report-sheet/Z005_260325_0001.CSV
tests/fixtures/daily-report-sheet/Z005_260325_0002.CSV
tests/fixtures/daily-report-sheet/Z004_260324.CSV
tests/fixtures/daily-report-sheet/products-sheet.csv
tests/fixtures/daily-report-sheet/Z001_260326_0001.CSV
tests/fixtures/daily-report-sheet/Z002_260326_0001.CSV
tests/fixtures/daily-report-sheet/Z005_260326_0001.CSV
tests/fixtures/daily-report-sheet/Z004_260326.CSV
```

件数は `sed -n '/^### 予定 path の一覧/,/^### 並走/p' docs/plans/2026-10-27-daily-report-sheet-runtime.md | grep -cE '^(docs|src|src-tauri|tests)/'` → `50`（2026-10-29。起票の時点は `41`）。

### 並走との重なり（`comm -12`）

2026-10-27 に `git fetch origin` の後、各 branch の `git diff --name-only --no-renames origin/main...origin/agent/<branch>`（#202 は merge 済みなので `git diff --name-only --no-renames 50f97a76 origin/main`）に、plan の段の lane は packet の予定 path（#193 は packet の「予定 path の一覧」の 62、#190・#194・#204 は packet の中の path の言及の全部で、実際より広い集合）を足して `LC_ALL=C sort -u` し、上の予定 path（起票の時点の 33。fixture の 8 は新設の folder で、どの lane とも重ならない）と `comm -12` で突き合わせた。`origin/main` は `ee5b221a`（#202 の merge）。

| PR（branch） | 重なる path |
|---|---|
| #172（`sd-direct-read-runtime`） | `docs/SCREEN_DESIGN.md`・`docs/backlog.md`・`docs/db-design/pos-tables.md`・`docs/function-design/24-io-csv-import-repo.md`・`docs/function-design/29-io-daily-report-parser.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/biz/sales_service.rs`・`src-tauri/src/cmd/sales_cmd.rs`・`src-tauri/src/db/sales_repo.rs`・`src-tauri/src/io/daily_report_parser.rs`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts`。round 1 で足した path のうち `docs/function-design/37-biz-daily-report-import-service.md`・`src-tauri/src/biz/daily_report_import_service/parse.rs`・`src-tauri/src/biz/daily_report_import_service/tests.rs`・`src/features/daily-report-import/DailyReportImportPage.flow.test.tsx` |
| #193（`stocktake-count-runtime-b2`） | `docs/SCREEN_DESIGN.md`・`docs/function-design/10-common-rules.md`・`docs/function-design/90-traceability.md`・`src/lib/bindings.ts` |
| #196（`initial-load-procedure`） | `docs/backlog.md` |
| #197（`z004-rollback-entry`） | `docs/SCREEN_DESIGN.md`・`docs/architecture/ui-task-specs.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts`・`src/lib/query-keys.ts` |
| #201（`dev-bins-out-of-msi`） | `docs/backlog.md`・`docs/decision-log.md`・`src-tauri/src/lib.rs` |
| #202（`screen-elements-redesign`、merge 済み） | `docs/SCREEN_DESIGN.md`・`docs/UI_TECH_STACK.md`・`docs/architecture/ui-task-specs.md`・`docs/backlog.md`・`docs/decision-log.md` |
| #204（`docs-file-line-check`） | `docs/backlog.md`・`docs/decision-log.md`・`docs/function-design/10-common-rules.md`（packet の中の言及。#204 の branch の差分は packet と Matrix だけ） |
| #190（`backup-offsite-runtime`） | `docs/backlog.md`・`docs/decision-log.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts` |
| #194（`backup-offsite-runtime-b`） | `docs/UI_TECH_STACK.md`・`docs/backlog.md`・`docs/decision-log.md`・`docs/function-design/10-common-rules.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/lib.rs`・`src/lib/bindings.ts`・`src/lib/query-keys.ts` |

round 1 の是正（2026-10-29）で足した 9 path は、`git fetch origin` の後に open の PR の branch（#172・#190・#194・#201・#212・#216）の `git diff --name-only --no-renames origin/main...origin/agent/<branch>` と `comm -12` で突き合わせ、重なりは #172 の 4 path（上の表の #172 の行）だけだった（fixture の 5 は新設の folder。`origin/main` は `f3f29630`）。起票の時点からある path の重なりは取り直していない（L3 を頼む時点の head で取り直す）。

節の所有表（重なる file の、本 lane が触る所と相手が触る所。末尾への追記と生成物は merge の順で両方を残し、生成物は取り込んだ後に再生成する〈D-098〉）:

| file | 本 lane | 相手（主な PR） | 揃え方 |
|---|---|---|---|
| `src-tauri/src/db/sales_repo.rs` | `get_completed_daily_report_aggregate` の支払の読取り、`OfficialDailyReportRow`、新しい 2 関数と DTO、test module の末尾 | #172: `NewDailyReportImport`・`insert_daily_report_import`・`find_same_settlement_daily_report_import`（新設）と test module の末尾 | #172 が先。本 lane は同期の後の fixture の形で書く |
| `src-tauri/src/io/daily_report_parser.rs` | `payment_key`、`parse_optional_i64`・`parse_optional_hundredths` と呼ぶ所の文、test module の末尾 | #172: `DailyReportParseResult.settlement_no` と `parse_daily_report_bundle` の精算回数、test module の末尾 | #172 が先 |
| `src-tauri/src/biz/sales_service.rs`・`src-tauri/src/cmd/sales_cmd.rs` | 支払の写し・warning、`get_daily_report_sheet`、新しい test | #172: test の fixture の field（`settlement_no`・`sd_relative_path`・`register_sd_scan_cache`）と `commit_daily_report_import` の引数 | #172 が先。本 lane の新しい test は同期の後の形 |
| `src-tauri/src/biz/daily_report_import_service/parse.rs`・`tests.rs` | `parse_errors` の分岐に `number_too_large`（BIZ-08-D2a）を 1 つ、test を 2 本 | #172: `_with_sd_lookup`・手順 1a の出所の判定・BIZ-08-D4〜D6 と、test の fixture の field（`sd_relative_path` 等） | #172 が先。本 lane は同期の後の `parse_errors` の分岐（#172 でも `settlement_mismatch` → 汎用の文の形のまま）に足す |
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.3 手順 3 の BIZ-08-D2a、§37.7 の 1 行、更新履歴の末尾 | #172: §37.3 手順 1a・4 の BIZ-08-D6、§37.4・§37.9 ほか | 段が別（手順 3 の中の BIZ-08-D2 の直後）。更新履歴の末尾は両方を残す |
| `src/features/daily-report-import/DailyReportImportPage.flow.test.tsx` | test を 1 本 | #172: `parseAndValidateDailyReportFromSd` の mock ほか | #172 が先。本 lane の test は同期の後の mock の形で書く |
| `src-tauri/src/lib.rs` | CMD-09 の塊に 1 行ずつ | #172（CMD-12 の 2 行と `AppState`）、#197・#190・#194（各 command）、#201（dev の bin の外し） | 行が別。merge の順で両方を残す |
| `src/lib/bindings.ts`・`docs/function-design/90-traceability.md` | 再生成 | #172・#193・#197・#190・#194 | 生成物。取り込んだ後に再生成 |
| `src/lib/query-keys.ts` | `dailyReportSheet` を `dailySales` の隣に 1 行 | #197（取込みの記録の key）、#194（backup の key） | 行が別 |
| `docs/function-design/24-io-csv-import-repo.md` | §14.21・§14.21a・§14.21b と更新履歴の末尾 | #172: §14.18a の後の新しい関数 | 節が別。更新履歴の末尾は両方を残す |
| `docs/function-design/29-io-daily-report-parser.md` | §29.2 の IO-07-D4・D4a・D6、§29.5 の行 | #172: IO-07-D5 の段（`settlement_no`）、§29.7・§29.8 | 段が別 |
| `docs/db-design/pos-tables.md` | §12d の設計意図の 1 段 | #172: §12b と B-2 の例 | 節が別 |
| `docs/SCREEN_DESIGN.md` | 日次売上の印刷ボタンの段 | #172: 1 日の動線と取込みの段、#193・#197: 別の画面の段、#202: 画面の要素の段（merge 済み） | 段が別 |
| `docs/UI_TECH_STACK.md` | §6.5.5 | #202: §2.x と §6.x の別の段（merge 済み）、#194: backup の段 | 節が別 |
| `docs/architecture/ui-task-specs.md` | UI-09a の印刷ボタンの行 | #197: UI-07 の段、#202: 画面の要素の段（merge 済み） | 行が別 |
| `docs/function-design/10-common-rules.md` | 「入力の上限と安全な整数の範囲」の (a) の表の行と (e)、更新履歴の末尾 | #193: 棚卸しの b2 の段、#194: 別の段 | 段が別。更新履歴の末尾は両方を残す |
| `docs/decision-log.md` | 末尾の D-142 | #202 の D-138（merge 済み）、#201・#190・#194 の決定 | 末尾は番号の順で両方を残す |
| `docs/backlog.md` | 「日報の紙の runtime」と「手動販売の金額欄の上限」（旧名の項） | 各 lane の自分の項 | 項が別 |

merge の順: **#172 が先**（本 lane の実装は #172 の merge の後に始め、#172 の merge の後に `origin/main` を単段で同期してから plan-gate）。#202 は既に merge 済みで、同期で取り込む（56 は #202 の置き換えの対象〈50・51・52・53・58・60〜67・69・77〉に無く、日次売上の画面の要素の設計は変わらない。同期の後に `docs/design-system/` の #202 の変更が日次売上の印刷ボタン・注記の部品に当たらないかを Writer が確かめる）。ほかの lane とは生成物と末尾の追記・別の段だけで、順を縛らない。L3 を束ねるかは、L3 を頼む時点の head で `docs/DEV_WORKFLOW.md` Wave Operation の条件（先に merge する全ての PR と path を共有しない）で決める（`lib.rs`・`bindings.ts` を共有する lane とは束ねない見込み）。

## Non-scope

- 上の Goal の「非目的」の全部。
- 日次売上の CSV 出力（列は変えない、UI-09a-D16 の「変えないもの」）。
- 部門別集計・Z001 の画面の表示・月次の部門集計（24 §14.21 手順 4・6、§14.22）。
- `docs/Plans.md`（lane の行を足さない。D-097）。

## Acceptance Criteria

数の baseline は開始 HEAD `50f97a76` に本 commit の正本の変更を当てた作業 tree で、下の command をそのまま実行した出力（2026-10-27。出力が無く exit 1 は 0）。完了時の期待は実装の後の head で同じ command を実行した値。

- AC1（IO-07-D4a）: `rg -c -F '"credit".to_string()' src-tauri/src/io/daily_report_parser.rs` が 0（baseline `1`）。`daily_report_parser.rs` の test `test_payment_key_has_no_credit_rule` が PASS。
- AC2（IO-07-D6、D-142）: `rg -c 'MAX_DAILY_REPORT' src-tauri/src/constants.rs` が 2（baseline exit 1）。Matrix の IO-07-D6 の行の test（上限ちょうどを受ける・1 越えると列を名指した `number_too_large`・形の誤りの `invalid_number` と文は今のまま・個数の上限）と BIZ-08-D2a の行の test（取込みの画面の文が帳票と列を名指し値を出さない・文の優先・形の誤りは汎用の文のまま・画面に文が出て選び直せる）が PASS。`rg -c 'number_too_large' src-tauri/src/biz/daily_report_import_service/parse.rs` が 1 以上（baseline exit 1）。
- AC3（IO）: `rg -c 'fn get_daily_report_sheet_lines\(|fn get_active_z004_import_totals\(' src-tauri/src/db/sales_repo.rs` が 2（baseline exit 1）、`rg -c 'PaymentGroup' src-tauri/src/db/sales_repo.rs` が 0（baseline `2`）。`rg -c 'OfficialDailyPaymentRow' src-tauri/src` の出力が空（baseline `src-tauri/src/db/sales_repo.rs:3`。2026-10-29 の round 1 で足した）。Matrix の 24 §14.21・§14.21a・§14.21b の行の test が PASS。
- AC4（BIZ）: `rg -c 'pub fn get_daily_report_sheet\(' src-tauri/src/biz/sales_service.rs` が 1（baseline exit 1）。Matrix の SALES-SHEET-D1〜D6 と 34 §19.3 手順 3 の行の test が PASS。
- AC5（CMD と bindings）: `rg -c 'get_daily_report_sheet' src-tauri/src/lib.rs` が 2（baseline exit 1）、`rg -c 'getDailyReportSheet' src/lib/bindings.ts` が 1（baseline exit 1）。`cd src-tauri && cargo run --bin generate_bindings` の後に `git diff --exit-code src/lib/bindings.ts` が exit 0。`bash scripts/check-command-drift.sh` が exit 0。
- AC6（画面）: `rg -c -F '準備中（Phase 4 で実装予定）' src/features/daily-sales/components/ExportBar.tsx` が 0（baseline `1`）、`rg -c -F '日報の部門確認が必要です' src` の出力が空（baseline `DailySalesPage.test.tsx:3`・`DailySalesPage.tsx:1`）、`rg -c -F '日報を印刷' src/features/daily-sales/components/ExportBar.tsx` が 1（baseline exit 1）。Matrix の UI-09a の行の component test が PASS。
- AC7（正本の注記）: `rg -n 'proposed・未実装' docs/function-design/24-io-csv-import-repo.md docs/function-design/29-io-daily-report-parser.md docs/function-design/34-biz-sales-service.md docs/function-design/37-biz-daily-report-import-service.md docs/function-design/42-cmd-sales-stocktake.md docs/function-design/56-ui-daily-sales.md docs/function-design/10-common-rules.md docs/UI_TECH_STACK.md docs/db-design/pos-tables.md docs/SCREEN_DESIGN.md | rg 'D-120|D-142|SALES-SHEET|IO-07-D4a|IO-07-D6|UI-09a|BIZ-08-D2a' | rg -v ':\| 20[0-9]{2}-' | wc -l` が 0（baseline `22`。2026-10-29 の round 1 の是正の後の作業 tree。起票の時点の 37 と BIZ-08-D2a を除いた式では `18` で、round 1 で 24 の手順 8・29 §29.5 の `number_too_large` の行・37 の 2 行が増えた。更新履歴の行は数えない）。`rg -n 'クレジット → .credit.|.credit. の鍵は全角' docs/function-design/29-io-daily-report-parser.md` の出力が空（baseline 2 行〈117・119 行目〉。round 1 で足した）。`rg -c -F '仕様未定のため Phase 2 では disabled' docs/architecture/ui-task-specs.md` が 1（baseline `2`。残るのは UI-09b の月次の行）。
- AC8（検査）: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR 0。`cd src-tauri && cargo run --bin generate_traceability -- --check` が OK。`cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` と `npm run typecheck && npm run lint && npm run format:check && npm test && npm run build` が exit 0（hosted の changed 分類でも同じ job）。
- AC9（L3、Human Gate の manual）: Matrix の Manual の L3-1〜L3-3 が owner の PASS。L3 の前に Writer が `cargo check --release` を通す（AC の証跡は PR の本文）。
- AC10（並走）: plan-gate の前に、#172 の merge の commit が HEAD の祖先（`git merge-base --is-ancestor <#172 の merge の commit> HEAD` が exit 0）で、同期は `origin/main` の単段 merge 1 回（`Plan Commit` は同期の後の packet の版）。plan-gate の前に、Contract Probe の P8 の行に、Z001・Z002・Z005 の数の列ごとの絶対値の最大の桁数と、数えた時点（日付と、数えた手元の Z 帳票の写しの版）が書かれている（Coordinator が数える。値は書かない）。
- AC11（公開してよい形）: `git diff -U0 origin/main...HEAD -- docs tests`（merge-base からの本 lane の差分。同期で取り込んだ他の lane の行を数えない）の追加の行に、店の金額・商品名・JAN・部門名・発言の原文・実データの名称の値が無い（fixture の名称は `項目A` のような合成の語、数は合成の値）。Plan Review と Final Review で差分を読んで確かめる。
- AC12（owner の判断事項 O2）: Ready の前に、O2（レジスターツールの日報との照合の置き場）への owner の明示の答えがある（L3 の依頼と同じ 1 回で、記号で得る。答えの原文は repo の外の回答台帳、PR には ID で引く）。答えが a なら D-142 の (5) のまま。答えが b なら Matrix の Manual に L3-4 を足す Gated Amendment を出し、D-142 の (5) を外す（下の Open Questions の表）。受容リスク（merge の時点では照合していない紙を、切替えまで比べるときだけ印刷する）は owner が持つ（`AGENTS.md` の Decision and Approval Boundaries）。答えの無いまま既定で Ready にしない。2026-10-29 の round 1 で足した。

## Design Readiness

- 引用する設計正本（節まで）: 34 冒頭の「数量と単位」・§19.2・§19.3（手順 1〜3、設計判断）・§19.3a（SALES-SHEET-D1〜D6）／24 §14.21（手順 1〜8）・§14.21a・§14.21b／42 §22.3・§22.4 の `get_daily_report_sheet`・§22.8／29 §29.2 の IO-07-D2・D4・D4a・D6、§29.4.1、§29.5／56 §56.1 の UI-09a-D13・D15・D16・D17・D18・D19、§56.7、§56.8、§56.13／UI_TECH_STACK §6.4・§6.5.5／pos-tables §12c・§12d・§12e／10 の「入力の上限と安全な整数の範囲」の (a)〜(e)／37 §37.3 手順 3（BIZ-08-D1・D2・D2a）・§37.7／SCREEN_DESIGN の日次売上画面／D-096・D-104・D-120・D-142・D-123。
- 必要な設計成果物: BIZ・IO の振舞い = updated in this PR（34 §19.3 手順 3、24 §14.21 手順 3・5・7・8、29 IO-07-D6、37 BIZ-08-D2a。D-142）／command・DTO = existing sufficient（42 の `get_daily_report_sheet`、34 §19.3a の型）／report の形 = existing sufficient（34 SALES-SHEET-D3・D4、56 UI-09a-D17）／operator workflow・文言 = existing sufficient（56 UI-09a-D16〜D19。IO-07-D6 の文は 29 に足した）／表・migration = existing sufficient（変えない）／durable な判断 = updated in this PR（D-142）。
- plan にしかない durable な判断の昇格先: D-142 と上の正本に置いた。packet にだけあるのは、規模と手法の数え、実装の順、予定 path と所有表、AC の数、owner の判断事項の文面、L3 の手順。
- 前提・制約と、延期した design gap の follow-up: P1（WebView2 の印刷）は L3-1、P5（1 枚に収まるか）は L3-2、P2（レジスターツールの日報との行ごとの一致）は UI-09a-D19 の (2)（O2 の既定）、P8（実データの最大の桁）は plan-gate の前に Coordinator が repo の外で数える。手動販売の金額欄の上限は backlog（D-142 の (3)）。延期が安全な理由: どれも紙の内容の規則を変えず、外れたときの戻り先（代わりの道の Gated Amendment、並びの Gated Amendment、D-120 の Revisit、上限の値の見直し）が正本にある。
- 絶対保証の自己点検: 「画面と紙は同じ値を同じ書き方で出す」は公式日報の部分（総売・純売・支払・部門）の値の範囲についての保証で、画面の支払集計と紙の Z002 は同じ関数の同じ結果、画面の部門別集計（部門マスタで照合）と紙の Z005（レジの名称のまま、SALES-SHEET-D2）は D-120 のとおり足し方が違う（範囲の保証は両方に及ぶ）。「読取りで範囲を検査しない」の例外は parser を通らない値（DB の直接の書換え・上限を入れる前の開発・試験 DB の行。10 の (e)）と手動販売の金額（保証の外）。「古い内容で印刷しない」は押したときの読み直し（SALES-SHEET-D6）に依り、読み直しと印刷の間に別の画面で取り消す経路は単一利用者の画面で起きない（34 §19.3a の前提条件）。
- 判定: ready（plan-draft）。理由: 設計の出力が上の正本と D-120・D-142 にあり、owner の判断事項は推奨の既定で書き進められ、残る外部前提は L3・D19・repo の外の数えで確かめるもので設計の問いではない。

## Registration / Generation Obligations

| 変更対象 | 義務（本 lane） |
|---|---|
| Tauri command `get_daily_report_sheet` | `lib.rs` の `collect_commands!` と `generate_handler!` の 2 か所（42 §22.8）、`#[tauri::command]` と `#[specta::specta]` の対、`cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成（S4、AC5） |
| REQ の参照の増減（新しい test の `REQ-401`・`REQ-501` 等） | `cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（S6、AC8） |
| function-design doc 新設 | 該当なし |
| route 新設 | 該当なし（印刷は日次売上の画面の中） |
| operator 画面新設 | 該当なし（到達導線は既存の日次売上。印刷ボタンの到達は ExportBar の component test と L3-1） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たる。帳票の行の位置・名称・数の桁と `credit` の規則は IO-07（adapter）。足し方（SALES-SHEET-D2）は BIZ-05 の 1 つの関数（D-142 の (4)） | 29 IO-07-D4a・D6、34 §19.3 手順 3 |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。事実 = 手元の Z 帳票の 2026-09-30 版の集計（D-120 の P3・P4）、Z004 の parser は `i32`（code）。未確認 = WebView2 の印刷、レジスターツールとの一致、実データの最大の桁。決定 = 上限の値、D2 を BIZ に置くこと | Contract Probe、D-142 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。紙は保存せず押したときに読み直す。取消・追加の後は次の紙が変わる。読込みの失敗は `再試行`。上限の越えは取込みの前に束を拒む（何も書かない） | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。締め → 取込み → 日次売上 → `日報を印刷` → 綴じる。切替えまでは今の Excel を続ける（UI-09a-D19） | Ordinary Operation、L3 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たる（小）。レジが変わると IO-07 と上限の値（D-142 の Revisit）が変わり、足し方と紙の並びは残る | D-142 の Revisit |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。fixture と test は合成だけ。P8 の数えは repo の外で、packet には最大の桁の数だけを書く | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。紙は日報（レジの集計）と PLU の商品別売上を別の表に置き、手動販売を載せない（D-025、SALES-SHEET-D4）。NULL を 0 に見せない | Matrix の D2・D4 の行 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる。WebView2 の印刷の dialog と `@page`、実機のプリンタでの字の大きさと 2 枚目への続き、画面の知らせの読みやすさ | Matrix の Manual |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たる（小）。印刷は OS のプリンタと WebView2 に依る。新しい依存は足さない。CI では印刷を試さない（L3） | UI_TECH_STACK §6.5.5 |

## Boundary / Wire Contract

- producer: CMD-09 `get_daily_report_sheet`（42）← BIZ-05 `get_daily_report_sheet`（34 §19.3a）← IO `get_daily_report_sheet_lines`・`get_daily_sales_records`・`get_active_z004_import_totals`（24）。画面の支払集計は BIZ-05 `get_daily_sales` ← IO `get_completed_daily_report_aggregate`（支払の行を足さない）。
- consumer: UI-09a（`useDailyReportSheet`・`ExportBar`・`DailyReportSheet`、日次売上の支払集計の表と注記）。
- wire type: 34 §19.3a の `DailyReportSheet`（`date: string`、`import_count: number`、`summary_rows`・`payment_rows`・`department_rows: DailyReportSheetRow[]`、`label_changed_sources`・`unsummed_sources: DailyReportSourceKind[]`、`plu_rows: DailySaleItem[]`、`plu_import: DailyReportSheetPluImport`）、`DailyReportSheetRow { position: number, label: string, quantity: number | null, count: number | null, amount: number | null, import_seq: number | null }`、`DailyReportSheetPluImport { import_count: number, skipped_count: number }`。既存の `DailySalesReport`・`OfficialDailyReportSummary`・`OfficialDailyPaymentLine` の wire は変えない。
- internal type: IO の DB DTO `DailyReportSheetLinesRow`・`DailyReportSheetLineRow`（24 §14.21a）、`OfficialDailyReportRow.payment_lines` の行の型（24 §14.21 手順 3。足さない）。個数は 100 倍の `i64` で足し、`quantity_hundredths_to_units` で `f64` へ（IO-07-D2）。
- precision/range: D-142。parser の上限（金額・件数 `9999999999`、個数 `9999999.99`）で、1 つの集計が `9007199254740991` を越えるには金額で 900,720 本の取込みの行、個数で 9,007,200 行が要る（`python3` で 2026-10-27 に計算: `9007199254740991 // 9999999999` = `900719`、`// 999999999` = `9007199`）。Z004 は `i32`（`// 2147483647` = `4194304`）。読取りで範囲を検査しない（画面も紙も）。和は `checked_add` で、溢れは `BizError::ValidationFailed`。
- round-trip path: 取込み（`daily_report_*_lines`・`sale_records`・`csv_imports`）→ IO の読取り → BIZ の足し算 → CMD → UI の描画 → `window.print()`。保存も往復もしない。
- invalid input: 日付の形式・暦の誤り → `ValidationFailed`（§19.3 と同じ文言）。取込みが 0 件の日は `import_count = 0`。上限を越える数の束は parser が `number_too_large`（取込みの段で止まり、紙に届かない。取込みの画面の文は BIZ-08-D2a で帳票と列を名指し、値を出さない）。
- compatibility: 新しい command と DTO を足すだけ。支払集計の互換の範囲と範囲の外の値は 34 §19.3 手順 3。返す `payment_key` の意味は変えず、画面の行の key を `${payment_key}-${index}` にする。

### 新しい契約の文と境界の 4 観点

本 commit で足した契約の文（IO-07-D6、10 の (a) の行と (e)、34 の冒頭と §19.3 手順 3、24 §14.21 手順 3・5・7、D-142）を 4 観点で一巡した。

| 観点 | 確かめたこと |
|---|---|
| 安全な整数の範囲 | 上限の値と、1 つの集計が範囲を越えるのに要る行の数を `python3` で計算した（上の precision/range）。月次の部門集計（§14.22 の `SUM`）も同じ行の数が要る。手動販売の金額は保証の外と (e) と D-142 に書いた |
| 0 の表示 | 上限は絶対値で掛け、0 と負の値はそのまま受ける（返品・訂正の負の行がある）。紙の 0 は `0`（UI-09a-D17、`¥` なし）、NULL は `—` で、0 と取り違えない（SALES-SHEET-D2 の (a)） |
| 失敗の時点と順序 | 上限の越えは parser の段（取込みの前、何も書かない）。紙の失敗の順は SALES-SHEET-D5（日付 → 読取り → 足し算）のまま。画面の支払集計の溢れは IO の `DbError` から BIZ の `ValidationFailed` に変わり、読取りの後の足し算の段になる（34 §19.3 手順 3 に書いた。上限の下では届かない） |
| 単位や種類の混在 | 金額・件数（整数）と個数（100 倍の整数）で上限を分けた。紙の `個数/件数` の列は `quantity` があればそれ、無ければ `count`（UI-09a-D17）。PLU の数量は商品の単位で出す（SPEC-UNIT-D3） |

round 1 の是正（2026-10-29）で足した契約の文（37 BIZ-08-D2a、29 IO-07-D6 の `number_too_large` と文の作り方、24 §14.21 手順 8、D-142 の (5)、Matrix の「既存の test の移し方」の新しい期待）を同じ 4 観点で一巡した。

| 観点 | 確かめたこと |
|---|---|
| 安全な整数の範囲 | BIZ-08-D2a の文は数を持たない（値を入れない）。上限の値と範囲の計算は変えていない。既存の test の移し方の新しい期待は fixture の値の和（`200`・`100`・`9000` 等）で、`i64` と安全な整数の範囲の中 |
| 0 の表示 | 上限は絶対値で掛けるので 0 と負の値は `number_too_large` にならない（Matrix の上限ちょうどの test に `-9999999999`）。移し方の新しい期待で NULL の行は NULL のまま（`(cash, 現金旧, NULL, 1)`）で 0 にしない |
| 失敗の時点と順序 | 上限の越えは parse の段（BIZ-08 の §37.3 手順 3）で止まり、何も書かない（`daily_report_imports` に行を作らない、`operation_logs` は best-effort）。文の優先は `settlement_mismatch`（BIZ-08-D2）→ `number_too_large`（BIZ-08-D2a）→ 汎用の文。複数の列の越えは帳票の Z001 → Z002 → Z005、帳票の中は列の左からの順で、parse の順や入力の file の順に依らない（Matrix の test は Z005 の file を先に渡す）。D-142 の (5) は Ready の前に owner の答え（AC12）、照合は紙の切替えの前（UI-09a-D19 の (2)） |
| 単位や種類の混在 | 形の誤り（`invalid_number`）と上限の越え（`number_too_large`）を `error_type` で分け、BIZ は文の中身でなく `error_type` で分岐する。列の語は個数と金額で違う（`個数列`・`金額列`・Z001 の `個数/件数列`）。24 手順 8 は内部の行（ID を持つ）と公開の集計（ID を持たない）と Z001 の写像を分けた |

## Test Plan

Test Design Matrix: [2026-10-27-daily-report-sheet-runtime](test-matrices/2026-10-27-daily-report-sheet-runtime.md)。

- targeted tests: Matrix の Test Matrix の各行（Rust の unit・integration、Vitest + RTL の component）。実装中は `bash scripts/local-ci.sh changed`。
- negative tests: Matrix の Negative Paths。
- compatibility checks: 互換の範囲の日の支払集計が今と同じ行と値（Matrix の 34 §19.3 手順 3 の行）。
- data safety checks: fixture と test は合成だけ（AC11）。
- main wiring/integration checks: 日次売上 → `useDailyReportSheet` → `commands.getDailyReportSheet` → CMD → BIZ → IO（`sales_cmd.rs` の command の test と ExportBar の component test）、印刷の dialog（L3-1）。
- L3（Human Gate の manual）: 項目ごとの到達経路・入力物・依存は Matrix の Manual。受理される入力物は `tests/fixtures/daily-report-z001/`（2 回取り込んだ日 `2026-03-24`）と `tests/fixtures/daily-report-sheet/`（商品の一括 import の CSV、足さない日 `2026-03-25`、改ページの日 `2026-03-26`、`2026-03-24` と `2026-03-26` の Z004。S7）で、Coordinator が Ready の依頼と同時に取り込む順の手順（Matrix の Manual の (1)〜(6)。owner に DB への直接の挿入を求めない）を渡す（CP932・CRLF の実 encoding、`docs/DEV_WORKFLOW.md` Human Visual Confirmation）。Writer は L3 の前に `cargo check --release`。
- Human visual confirmation（画面の変更）: 日次売上（`/reports/daily`）の印刷ボタンと押せない状態の文、支払集計の注記の見出しと文、2 回以上の日の文。Windows の native の L3 も要る（印刷）。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（締めの後にアプリから 1 日 1 枚の紙を印刷する）を達成できるか。押せない日に、その場で直せる導線があるか。
- D-142: 上限の値と「読取りで検査しない」が、画面と紙の食い違いを生まないか。手動販売の金額を保証の外に残すことが、紙の範囲で安全か。D2 を BIZ の 1 つの関数に移すことで、互換の範囲（34 §19.3 手順 3）の日の支払集計の行と値が変わらないか、IO の既存の test の期待が assert の単位で BIZ へ移るか（範囲の中は同じ値、範囲の外は正本の新しい規則の値で、検出力を範囲の中の test が保つ。弱体化しない。Matrix の「既存の test の移し方」）。
- #172 との所有と順（同じ file の test の fixture の形）。
- この変更で不要になる記述・条件・code が下の節で尽くされているか。

## この変更で不要になるもの

- code: `sales_repo.rs` の `get_completed_daily_report_aggregate` の支払の集約（`PaymentGroup` と group の代表の選び方。BIZ の D2 の関数へ移す）と、集約した支払の DB DTO `OfficialDailyPaymentRow`（行の型に置き換わる。AC3。round 1 で足した）、`payment_key` の `クレジット` / `credit` の分岐、`ExportBar.tsx`（日次）の頭の comment「Phase 4 で window.print 検討」と Tooltip「準備中（Phase 4 で実装予定）」と `aria-disabled` の印刷の `span`、`DailySalesPage.tsx` の見出し `日報の部門確認が必要です` と 2 回以上の日の旧い文。
- 正本の記述: D-120・D-142 の「proposed・未実装」の注記（AC7）、`docs/architecture/ui-task-specs.md` の UI-09a の「仕様未定のため Phase 2 では disabled」、56 §56.13 の「印刷ボタンの本実装」の非目的の行、24 §14.21 の支払の集約の手順（本 commit で BIZ へ移した）と手順 8 の旧い返却の契約（「Z001 …だけが親 ID と imported_at を持つ」「BIZ …並びと値を変えない」。round 1 で 3 つに分けて直した）、10 の (e) の「売上の金額」の項（本 commit で分けた）、29 の IO-07-D4 の段の今の `credit` の規則の 2 行（117・119 行目。S6 で直す。AC7。round 1 で足した）、56 UI-09a-D18・UI_TECH_STACK §6.5.5・backlog の P1 の「実装の前に」と、34・29・backlog の P2 の「runtime の lane の L3 で確かめる」（round 1 で直した）。
- backlog: 「日報の紙の runtime」の項と、「code の comment・文言の古い句 2 つ」の (1) の日次の分（closeout で閉じる）。「日次売上と日報の紙の wire の i64 …」の項は本 commit で「手動販売の金額欄の上限」に置き換えた。
- 残すもの（理由）: `payment_key`・`line_key` の列と保存（D-120。表示の行の鍵と既存の wire が使う）、月次の `ExportBar` の印刷の disabled と Tooltip（月次の印刷は未定）、D-120 の本文（決定の記録。引き先の移動と P2 の照合の置き場の移動は D-142 の (4)・(5) が記録する）、BIZ-08-D1 の汎用の文（形の誤りなど `number_too_large` 以外の parse の失敗は、今の「日報ファイルの解析に失敗しました」のまま）。

## Open Questions

owner の判断事項（TD-401 により問わずに推奨の既定で書き進めた。L3 の依頼と同じ 1 回で既定を見せ、記号か短文で答えられる形にする。答えで変わる所を名指す）:

- O1 紙の並びの比べ方（UI-09a-D17・D-120 の Q1）:
  - a = 既定の案 a だけを実装して L3 で印刷し、読みやすければ a に決める。読みにくければ b（1 段・値が全部 0 の行を外す・12 pt）か c（2 枚・12 pt）を名指し、Gated Amendment で直す（既定）
  - b = a・b・c の 3 案を実装して L3 で並べて印刷する
  - 推奨: a（candidate）。理由: owner の印刷は 1 回で済み、b・c の code を選ばれない案のために書かない。答えで変わる所は下の表。
- O2 レジスターツールの日報との値の照合（D-120 の P2）の置き場:
  - a = 本 PR の L3 から外し、運用開始の後の UI-09a-D19 の (2)（切り替える前に実際の 1 日でアプリの紙と今の Excel の紙を並べる）に寄せる（既定）
  - b = 本 PR の L3 に入れる（owner が持ち帰りの Z の写しの過去の日〈2 回精算した日を含む〉を Windows の開発の build に取り込んで紙を印刷し、店の綴りの同じ日の紙と並べる）
  - 推奨: a（precondition-dependent: D19 の (2) が切替えの前に必ず行われること）。理由: 店は切替えまで Excel を続け、merge は店の運用を変えないので、照合を merge の条件にすると店へ行く日まで PR が止まる。a は D-120 の Guarantee range（「runtime の lane の L3 で確かめる」）の置き場を動かすので、本 lane の round 1 で D-142 の (5) に記録し、34・29・56 UI-09a-D19・backlog の文を合わせた（D-120 の本文は変えない）。受容リスクは owner が持つので、既定のまま Ready にせず、owner の明示の答えを Ready の前の条件にする（AC12）。答えで変わる所は下の表。
- O3 名称か並びの変わった日の画面の 2 つの知らせ（D-120 の Final Review の closure の P3）:
  - a = 公式セクションの注記と、印刷ボタンの下の帳票ごとの文の両方を残す（正本どおり。既定）
  - b = Z002 の文は注記だけにし、印刷ボタンの下からは外す
  - 推奨: a（candidate）。理由: 注記は画面の支払集計、ボタンの下は紙の知らせで、読む場所が違う。L3-3 で owner が読み違えるか重いと判じたら b。答えで変わる所は下の表。
- O4 parser の上限で取り込めない束（D-142 の (1)）:
  - a = 金額・件数 `9999999999`、個数 `9999999.99` を越える束は取り込めず、列を名指す（既定。技術の判断で、owner の承認を要しない。見せるだけ）
  - b = 上限を掛けない（画面と紙が 2^53 を越える値を黙って誤って出しうる）
  - 推奨: a（confirmed: TD-213 の入力の上限の方針と同じ形）。答えで変わる所は下の表（b でも D-142 の全部は外さず、上限の項だけを外す）。

答えで変わる所（既定でない答えのときに Gated Amendment で直す所。2026-10-29 の round 1 で表にした）:

| 答え | Scope | AC | Matrix | 正本 |
|---|---|---|---|---|
| O1 = b（3 案を並べる） | S5 の紙の component に案 a・b・c の切替え（L3 の build だけで選べる形） | AC9（L3-2 の印刷が 3 回） | Manual の L3-2（3 案を並べて印刷し、1 つを名指す） | 選んだ案が a 以外なら 56 UI-09a-D17 の「紙の並び」と 34 SALES-SHEET-D3（D-120 の Revisit） |
| O2 = b（照合を本 PR の L3 に入れる） | S7 に照合の手順（owner が持ち帰りの Z の写しの過去の日〈2 回精算した日を含む〉を Windows の開発の build に取り込む。入力は repo の外で、tracked にしない） | AC9 に L3-4、AC12 は b の答え | Manual に L3-4（アプリの紙と店の綴りの同じ日の紙を並べ、行ごとに値が同じ）、Residual Test Gaps の照合の行 | D-142 の (5) を外す。34 §19.3a の「精算をまたいで足してよい根拠」・29 §29.4.1 の末尾・56 UI-09a-D19 の (2) の括弧・backlog の Human Gate の文を「runtime の lane の L3」へ戻す。Owner Effort Budget の L3 の round が店へ行く日に依る |
| O3 = b（Z002 の文は注記だけ） | S5 の `ExportBar.tsx` の帳票ごとの文から Z002 を外す | AC6（ExportBar の test の期待） | Test Matrix の `ExportBar shows label change notices per source (UI-09a)` の期待（Z002 の行が出ない）、Manual の L3-3 の期待（ボタンの下の `支払集計（Z002）：…` の文が出ない） | 56 UI-09a-D17 の状態の表の「`import_count` が 1 以上」の行の帳票ごとの文 |
| O4 = b（上限を掛けない） | S1 の上限の検査・`constants.rs` の定数・`number_too_large`・S1 の BIZ-08-D2a の分岐と test を外す（`credit` の規則〈IO-07-D4a〉は残す） | AC2 を外す（AC1 は残す） | IO-07-D6 の上限の 2 行と BIZ-08-D2a の新しい 3 行（既存の test の行〈`test_parse_daily_report_req401_invalid_number`・`test_daily_report_req401_parse_error_logs_parse_failed`〉は残す）・`test_daily_sales_and_sheet_return_same_large_values`・Failure Modes・Negative Paths・Boundary Checks の上限の行・State Lifecycle の上限の行 | D-142 の (1)・(2) と Guarantee range の「parser を通って保存された日報の値は上限の中」を外し、(3)・(4)・(5) と題の BIZ の共通化は残す（D2 を BIZ の 1 つの関数に置く決定は上限に依らない）。29 IO-07-D6 と §29.5 の `number_too_large` の行、37 BIZ-08-D2a と §37.7 の行、10 の (a) の表の行と (e) のレジの日報の段、34 冒頭の「レジの日報の値」の段（範囲の保証が無くなる: 画面と紙は同じ値を出すが、2^53 を越える値を正確に出す保証は無い）。Goal の 3 つ目の項の上限の部分、Ledger の IO-07-D6・10 (e)・BIZ-08-D2a の行、Ordinary Operation の上限の行 |

店に聞くこと: 無し（紙を渡す先が並びや項目を気にするかは D-120 の S1 のまま、切り替える前に owner が新しい紙を 1 枚見せて確かめる）。

agent 側で解く問い: Contract Probe の P1・P5（L3）、P8（plan-gate の前、Coordinator）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SALES-SHEET-D1（1 日 1 枚、完了した取込みを全部、取消を入れない） | 34 §19.3a | S3 `get_daily_report_sheet`、S2 §14.21a | Matrix の D1 の行 | 非対象 |
| SALES-SHEET-D2（位置と名称の組の 3 つの場合、保証の範囲、足す値〈NULL が先・取込みの id の昇順・溢れ〉、並び、`label_changed_sources`・`unsummed_sources`・`import_seq`、Z005 は照合しない） | 34 §19.3a | S3 の D2 の関数（紙と画面で 1 つ） | Matrix の D2 の行 | 非対象（値の一致は O2 で D19） |
| SALES-SHEET-D3（載せる行、名称の空の行） | 34 §19.3a | S3 | Matrix の D3 の行 | L3-2（並び） |
| SALES-SHEET-D4（PLU は `source='auto'` で 0 でない行、`plu_import`） | 34 §19.3a、24 §14.21b | S3、S2 | Matrix の D4 の行 | 非対象 |
| SALES-SHEET-D5（失敗の順、空の日） | 34 §19.3a | S3 | Matrix の D5 の行 | 非対象 |
| SALES-SHEET-D6（保存しない、押したときに読み直す） | 34 §19.3a、56 UI-09a-D17 | S3、S5 | Matrix の D6・D17 の行 | 非対象 |
| 34 §19.3 手順 3（画面の支払集計: IO の行に D2 の関数、代表、互換の範囲、範囲の外の値、2・3 の warning の文、溢れは `ValidationFailed`） | 34 §19.3、24 §14.21 手順 3・5・7、D-142 の (4) | S2、S3 | Matrix の 19.3 の行と「既存の test の移し方」 | 非対象 |
| 24 §14.21a・§14.21b（読む範囲・並び・`import_ids`・Z005 は `raw_department_name`・`(0, 0)`） | 24 | S2 | Matrix の 14.21a・14.21b の行 | 非対象 |
| CMD `get_daily_report_sheet`（登録 2 か所と bindings） | 42 §22.4・§22.8 | S4 | `scripts/check-command-drift.sh`、bindings の clean diff、`sales_cmd.rs` の command の test | 非対象 |
| IO-07-D4a（`credit` の規則を外す、正規化しない） | 29 §29.2 | S1 | `test_payment_key_has_no_credit_rule` | 非対象 |
| IO-07-D6（数の上限、`number_too_large` と文〈帳票と列の語だけ〉、1 か所の検査） | 29 §29.2・§29.5、10 (a) の表の行、D-142 の (1) | S1 | Matrix の D6 の行 | 非対象 |
| BIZ-08-D2a（上限の越えの利用者向けの文。帳票と列を名指し値を出さない、`settlement_mismatch` の次に優先、他の失敗は汎用の文のまま） | 37 §37.3 手順 3・§37.7、D-142 の (1) | S1 の `parse.rs` | Matrix の BIZ-08-D2a の行（BIZ の integration 2 本と既存 1 本、画面の component 1 本） | 非対象 |
| D-142 の (5)（P2 の照合の置き場を UI-09a-D19 の (2) へ） | D-142、34 §19.3a、29 §29.4.1、56 UI-09a-D19 | 運用（O2 の答えを AC12 で Ready の前に得る） | 非対象 | 非対象（O2 が b なら L3-4） |
| 10 (e)（レジの日報・Z004 の値は読取りで拒まない、手動販売は保証の外） | 10、34 冒頭、D-142 | 検査を足さないこと（画面と紙） | Matrix の「範囲の検査を置かない」の行 | 非対象 |
| UI-09a-D16（2 回以上の日の文、注記の見出し、支払集計の行の key、2・3 の warning を注記に出す） | 56 §56.1 | S5 `DailySalesPage.tsx` | Matrix の D16 の行 | L3-3（読みやすさ、O3） |
| UI-09a-D17（印刷ボタンと 4 つの状態の文・リンク・`再試行`、押したときの順、紙の並び・表・値・知らせ・9 pt・色なし・続け方・画面に出さない、帳票ごとの画面の文、`skipped_count` の文とリンク） | 56 §56.1 | S5 `ExportBar.tsx`・`DailyReportSheet.tsx`・`useDailyReportSheet.ts`・`query-keys.ts` | Matrix の D17 の行 | L3-1・L3-2 |
| UI-09a-D18（WebView の印刷、`@page`・`@media print`、代わりの道は Gated Amendment） | 56 §56.1、UI_TECH_STACK §6.5.5 | S5 `globals.css`、`window.print()` | `globals.test.ts` の literal、`window.print` の spy | L3-1（P1） |
| UI-09a-D19（切替えの時期） | 56 §56.1 | 運用（Coordinator と owner） | 非対象 | 非対象（運用。O2 の a なら P2 の照合を含む） |
| 到達導線（日次売上の画面から印刷ボタンへ） | 56 §56.1、SCREEN_DESIGN の日次売上 | 既存の route `/reports/daily`（`src/routes/reports/daily.tsx`） | ExportBar・DailySalesPage の component test | L3-1 |
| 隣接の除外: UI-09a-D12・D13・D15、Z001 の画面の表示（D-096）、部門別集計（24 §14.21 手順 4）、月次（§14.22・UI-09b）、日次 CSV（§19.5）、取込みの経路（37・45・55、D-111・D-121） | 56、24、34、D-096 | 変えない | 既存 | 非対象 |

## Contract Probe

- P1（Windows の WebView2 で `window.print()` が OS の印刷の dialog を出し、`@page` の A4 縦と余白 10 mm が効き、`@media print` でアプリの枠が出ない）: 未確認（本 run は Windows で動かしていない。出所は D-120 の設計 lane の推測）。→ 確かめるには Windows の build と owner の操作が要り、`window.print()` の道は code が小さい（CSS と 1 つの呼出し）ので、実装してから L3-1 で確かめる（owner の操作を 1 回にまとめる）。起票の時点では 56 UI-09a-D18・UI_TECH_STACK §6.5.5・backlog が「実装の前に確かめる」と書いていたので、round 1 の是正（2026-10-29）で 3 つとも「実装の後の L3（L3-1）で確かめる」に直した。効かなければ、UI-09a-D18 の代わりの道を、店の PC の `.html` の関連付けが印刷できるブラウザであることを確かめた後に、42 に command を足す Gated Amendment で入れる（Owner Effort Budget の予備 1）。
- P2（足した値が行ごとにレジスターツールの日報の値と同じ）: 未確認の前提（D-120 の P2。店主の答え TD-280・TD-305）。→ O2 の既定 a で、運用開始の後の UI-09a-D19 の (2) で確かめる。違えば D-120 の Revisit。置き場の移動は D-142 の (5) に記録し、34 §19.3a・29 §29.4.1・56 UI-09a-D19・backlog の文を合わせた（round 1、2026-10-29。D-120 の本文は変えない）。owner の明示の答えを Ready の前に得る（AC12）。
- P5（既定の案 a が A4 縦 1 枚に 9 pt で収まるか、収まらない日に見出しが繰り返し 2 段が段ごとに続くか）: 未実測（D-120 の P5 の見積り: PLU の行のある日は 2 枚目へ続く見込み）。→ L3-2。続け方は、収まるかに依らず必ず改ページが起きる `2026-03-26` の束（S7）で、明示の PASS の条件（Matrix の L3-2 の (b)）で確かめる（round 1、2026-10-29。起票の時点の「2 枚目に続くなら」の条件付きの基準をやめた）。
- P6（Z004 の取込みが使えるか）: 停止 ADR（`docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md`）は 2026-10-23 に superseded（b1 が入口を外した。ADR の Status の行と 32 の「停止の解除」の節を 2026-10-27 に読んだ）。→ 記録。PLU の部分は Z004 を取り込んだ日に出る。L3 は、商品の一括 import で合成の商品を入れた後に、精算の識別メタを持つ layout A の合成の Z004 を取り込んで PLU の行を出す（b1 の後の Z004 は精算回数の無い file を拒み〈32 の表の順 1〉、`get_daily_sales_records` は `INNER JOIN products` なので商品が無いと行が出ない。round 1 の是正、2026-10-29。S7）。
- P7（Z004 の parser の個数・金額は `i32`）: `src-tauri/src/io/z004_parser.rs` の行の型の `quantity: i32`・`amount: i32` と 23 §13.6 の「金額・数量」の行を 2026-10-27 に読んだ。→ 確認済み（D-142 の (2) の前提）。
- P8（実データの日報の金額・件数・個数の最大の桁が上限〈10 桁、個数は整数部 7 桁〉のはるか下）: 確認済み（2026-10-29、Coordinator。2026-10-18 版での値）。repo の外の手元の Z 帳票の写し（2026-10-14 までの写しと 2026-10-18 の SD の写し。Z001・Z002・Z005 の data の行の件数・金額の列）で、列ごとの整数部の最大の桁は、件数・個数の列（第 3 列）が 3 桁（個数の整数部の上限 7 桁のはるか下）、金額の列（第 4 列）が 6 桁（上限 10 桁のはるか下）。小数のある欄は件数・個数の列だけで、小数部は 1 桁。数でない欄は 0（2026-10-29 に列ごとに数え直した。計画の review の closure の Opus の P3）。値は書かない（集計の script と出力は repo の外）。上限は見直さない。

## Data Safety

- tracked に書かない: Z 帳票・日報の中身、名称の値、金額・商品名・JAN・部門名、店主・owner の発言の原文、repo 外の台帳の本文。P8 は桁数だけを書く。
- local-only: repo 外の回答台帳、手元の Z 帳票の写しとその集計の出力、L3 で印刷した紙の写真。
- synthetic-only: test と `tests/fixtures/daily-report-sheet/` は合成の束と合成の商品だけ（名称は `項目A` の形、金額は合成の値）。L3 の紙の写真・PDF を commit しない。

## Implementation Results

（実装の後に書く）

## Review Response

### Plan Review round 1（2026-10-29、`REVIEWED_HEAD=ee8065b1`）

- 判定: fresh Opus 5.5 = request changes（P1 0 / P2 3 / P3 2。`.local/reports/wave-2026-10-29/pr211-plan-r1-opus.md`、local-only で tracked でない）、Codex = request changes（P1 0 / P2 6 / P3 1。`.local/codex-orders/last-663-plan-review-daily-report-sheet.md`、local-only）。重なりを除いて 8 件。Coordinator が全件受理（採 8 / 否 0）し、発注 669 で起草役が直した。

| # | finding（出所） | 採否 | 直した所 |
|---|---|---|---|
| 1 | 既存の IO の test の期待を「同じ fixture と同じ値のまま」BIZ へ移せない 2 本と、支払の assert の無い overflow の 1 本（両者 P2） | 採 | Matrix の「既存の test の移し方」を assert の単位の表に直し、範囲の中か外かと、範囲の外の 2 本の新しい期待（D2 の 2 の 2 行と 2 の warning、34 §19.3 手順 3 が出所）、検出力を保つ範囲の中の BIZ の test を名指した。S2・失敗定義・Review Focus の文を合わせた。範囲の中の代表の test を Matrix に 1 本足した |
| 2 | L3-2 が空の DB の商品の無さと b1 の後の Z004 の形で到達できない（両者 P2）、改ページを観測する入力が無い（Codex P2） | 採 | S7 に商品の一括 import の CSV（b1 の `products-new.csv` と同じ形）、Z004 の layout A と精算の識別メタ・当たる行と当たらない行の数、必ず改ページが起きる `2026-03-26` の束と Z004 を足した。Matrix の Manual に取り込む順 (1)〜(6) と、L3-2 の (b) の明示の PASS の条件（見出しの繰り返し・日付・段の続き・PLU の始まる位置）を書いた |
| 3 | P1・P2 の確かめの置き場が正本と食い違う（両者 P2） | 採 | P1: 56 UI-09a-D18・UI_TECH_STACK §6.5.5・backlog を「実装の後の L3（L3-1）」に。P2: D-142 に (5) を足し（新しい D にしなかった理由は下）、34 §19.3a・29 §29.4.1・56 UI-09a-D19・backlog を合わせた。AC12（owner の O2 の明示の答えを Ready の前に） |
| 4 | 上限の越えの列名が汎用の文に置き換わり利用者に届かない（Codex P2） | 採 | 37 BIZ-08-D2a と §37.7 の行、29 IO-07-D6 の `number_too_large` と文の作り方、10 (a)、D-142 (1)。S1 に `parse.rs`・`tests.rs`・`DailyReportImportPage.flow.test.tsx`、Matrix に BIZ-08-D2a の 4 行、AC2、Ledger、Ordinary Operation。#172 の branch の同じ file を読んで分岐の形が同じことを確かめた |
| 5 | 24 §14.21 手順 8 が新しい支払の行と矛盾する（Codex P2） | 採 | 手順 8 を (a) 内部の支払の行・(b) 公開する支払の集計・(c) Z001 の単純な写像に分けて書き直した。手順 6 の命名の例も直した |
| 6 | O1〜O4 の答えで変わる所の列挙が足りない（両者 P3） | 採 | Open Questions に答えごとの Scope・AC・Matrix・正本の表。O4 の b は D-142 の上限の項だけを外す形に限った |
| 7 | 不要になるものの漏れ（29 の `credit` の 2 行、`OfficialDailyPaymentRow`。Opus P3） | 採 | 「この変更で不要になるもの」と S6 に足し、AC3・AC7 に `rg` の 0 件を足した |
| 8 | AC11 の範囲が他の lane の行を数える、P8 が AC に結ばれていない（Opus P3） | 採 | AC11 を `origin/main...HEAD` に、AC10 に P8 の行の桁数と数えた時点を足した |

- D-142 への追記にした理由（新しい D にしなかった）: (5) は本 lane の起票の決定 D-142 と同じ lane の、同じ D-120 の後続の実装で動かす検証の置き場で、D-142 は未 merge なので直せる。新しい番号を取ると並走の lane と番号を取り合い（D-143〜D-147 は他の lane が使っている）、O2 の答えが b のときに外す所が 2 つの D に分かれる。O4 の b が外すのは (1)・(2) の上限の項だけで、(5) とは独立に外せる（上の表）。
