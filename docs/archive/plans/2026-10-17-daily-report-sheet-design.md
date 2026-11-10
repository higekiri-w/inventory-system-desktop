# Plan Packet: 日報の紙（1 日 1 枚）と日報の意味づけを設計正本へ置く

2026-10-17 に同時に起票した 6 本の lane（repo 外の回答台帳 TD-304「動かせるだけ動かす」、TD-295「並列にできるものは並列に」）の 1 本で、wave の名前は持たない（design lane、docs だけ。runtime は後続の lane）。並走の lane は、売上の SD からの一括取込みと EJ の控えの閲覧（branch `agent/sd-bulk-import-design`、D-121）、単位の語の直し（`agent/stock-unit-relabel`）、JAN を直した商品の前の JAN（`agent/jan-alias-design`、D-122）、SD 直読みの runtime（`agent/sd-direct-read-runtime`）、MSI の配布の手順（`agent/msi-distribution-docs`）。本 lane の branch は `agent/daily-report-sheet-design`（起点 main `f5d7aaf4`）。

規模と手法（起票の最初に数えた。file の数は `git diff --name-only f5d7aaf4 HEAD -- docs ':(exclude)docs/plans'` の 12 file）: 触る設計正本は 8 file（function-design の 24・29・34・42・56、db-design の pos-tables、`SCREEN_DESIGN.md`、`UI_TECH_STACK.md`）と `architecture/ui-task-specs.md`・`plu-export-and-real-csv-verification.md`（印刷の要否の古い記述の追随）・decision-log・backlog。画面 1（日次売上の `ExportBar` と、画面に出さない印刷用の紙の内容）。migration 0（DB の列・制約は変えない）。wire の型は後続で 1 つ増える（`get_daily_report_sheet` の `DailyReportSheet`。既存の DTO は変えない）。横断の約束がかかる経路は 2 つ（同じ日の取込みをまたいで足す鍵〈日次売上の支払集計と日報の紙の 2 か所〉、紙の切替えの時期〈店の運用とアプリの導入の順〉）。**1 lane のまま進める**。足す鍵は 34 SALES-SHEET-D2 の 1 か所に置き、24 §14.21 手順 3 と pos-tables §12d はそこを引く。分ける案（意味づけの見直しを別の lane にする）は採らない。理由: 紙の足し方と支払集計の鍵は同じ規則で、分けると 2 本が同じ規則を取り合う。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 94c8fb01263caad5ab12854ace49bafe47ed9f7f
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（本 run の起草役）
- Plan Reviewer: fresh Opus 5.5 + Codex（GPT-6.1 Sol / high。`.local` の model の選び方の「R3 で複数の層にまたがる契約の初回」の行）
- Final Reviewer: Fable 5.1（Claude 側、R3・design lane）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/daily-report-sheet-design

遷移の記録:

1. kickoff → spec-check（2026-10-17、起草役）: owner の決定（repo 外の回答台帳 TD-260・TD-280・TD-296 の 15）と店主の答え（TD-279・TD-305）、2026-10 の見直しの突き合わせ O-18（repo 外）を Scope にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-17、起草役）: 触る正本（34 §19.3、24 §14.21、29 IO-07-D4・§29.4.1、56 UI-09a-D13・ExportBar・非目的、pos-tables §12d、SCREEN_DESIGN の日次売上、UI_TECH_STACK §6.5）が印刷を「仕様未定」とし、紙の用途を「外部提出帳票ではなく」と事実に合わない形で書き、支払の合算を並び順の鍵で持つ。同じ commit で設計正本を更新した（下の Design Readiness）。
3. design → plan-draft（2026-10-17、起草役、同じ commit）: 条件「設計の出力が正本にある」は Design Readiness の引く正本と D-120 で満たす。条件「未解決の設計の問いが無い」は、owner に聞く見た目の問い（紙の並びの案、切替えの日）に答えが無い間の既定を正本（UI-09a-D17・D19）に書いたことで満たす。残る外部前提（Contract Probe の P1・P2・P5）は runtime の lane の実装の前と L3 で確かめるもので、設計の問いではない。plan-gate への遷移は Coordinator。
4. plan-draft → plan-gate（2026-10-17、Coordinator）: packet・Matrix と正本は起草役の commit（`2a615820`）にある。Coordinator は D-120 の全文と、29 §29.4.1・56 UI-09a-D13・D17・D19・34 SALES-SHEET-D2 の追加の行を読み、owner の決定（TD-260・TD-280・TD-296 の 15）と店主の答え（TD-279・TD-305）の写しに食い違いが無いこと、手元の資料の数に版の時点が付いていること、追加の行に実値・実ラベルが無いこと、37・45・55（並走の 364・367 の所有）に触れていないことを確かめた。owner に聞く 2 つ（紙の並びの案、切替えの日）は既定（案 a）で進められ、並びは runtime の lane の L3 で実機の印刷で並べて決める形を採る。Plan Review round 1 へ出す。
5. plan-gate（round 1 の是正、2026-10-18、Coordinator）: Plan Review round 1（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。是正の前に相談役（Fable 5.1）に反例探しを頼んだ。起草役が `41c7210f` で直し、Coordinator が差分（34 SALES-SHEET-D2 の鍵の理由と保証の範囲・D5 と処理ステップの順、56 UI-09a-D13 の出典・D17 の押したときの順と画面の知らせとリンク・D18 の代わりの道、24 §14.21 の互換の範囲、29 IO-07-D4 の D4a への参照、plu-export の旧い記述、Matrix の追加の行）を読んだ。鍵（位置と名称の組）と Scope の範囲は変えず、その場で直したので plan-gate に留める。Plan Review round 2 へ出す。
6. plan-gate（round 2 の是正、2026-10-18、Coordinator）: Plan Review round 2（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。起草役が `a305f187` で直し、Coordinator が差分（34 SALES-SHEET-D2 の 3 つの場合分けと保証の範囲、24 §14.21 手順 3 が D2 を引く形、56 のリンクの行き先・受入の順・代わりの道・続け方、Matrix の追加の行）を読んだ。鍵（位置と名称の組）は変えず、別の項目を足しうる日だけ足さない形にした。その場で直したので plan-gate に留める。Plan Review round 3（天井）へ出す。
7. plan-gate（round 3 の是正、2026-10-18、Coordinator）: Plan Review round 3（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。round 3 は round の天井で、owner が「直して、追加の round なしで plan-approved」と決めた（repo 外の回答台帳 TD-317、介入 1 回目 / 予算 9 回）。起草役が `09e8e91c` で直し、Coordinator が差分（足さない日の画面と紙の文、24 §14.21 の互換の範囲を正本 1 か所に置いたこと、`import_ids` と `import_seq`、行の key の test の更新の確かめ、backlog の印刷の要否、再試行）を読んだ。
8. plan-gate → plan-approved（2026-10-18、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）のうち Claude 側が round 3 で P1・P2 が無いと報告し、Codex の P2 3 件は owner の決定（repo 外の回答台帳 TD-317）どおり直した（Review Response 参照）。Plan Commit は round 3 の是正と記録の後の版。
9. plan-approved → implementing（2026-10-18、Coordinator、本 commit）: docs だけの design lane で、設計の出力は Plan Commit までに正本にある。状態だけを進め、Final Review（Minimum 1）へ出す。

## Owner Effort Budget

- 介入回数上限: 9（既定の 6 から owner が改定、repo 外の回答台帳 TD-296 の 5a）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9（owner の改定、TD-296） | 3（closeout 時点。Plan Review round 3 の天井の決定 1〈TD-317「直して進める」〉、Ready 1、merge 1。L3・manual は無し。Open Questions の Q1・Q2 は聞かず既定 a。TD-260・TD-280・TD-296・TD-279・TD-305 は起票の前の決定と答えで、本 lane の介入に数えない） | 0 | 6 | 9 = 3 + 0 + 6 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
report output（日報の紙の内容と並び）、Tauri command DTO（`get_daily_report_sheet` と `DailyReportSheet` を足す）、operator workflow（締めの後の印刷と、Excel から紙への切替え）の契約を決める設計で、`docs/DEV_WORKFLOW.md` の Risk Tiers の R3 の行（report CSV・Tauri command DTO・operator workflow）に当たる。本 lane は docs だけだが、後続の runtime が写す契約を決めるので R3。R4 ではない: 紙は読み取りだけで、DB の列・migration・データの寿命に触れず、実データを公開しない。この変更で required gate の green / red は変わらない: docs だけで、REQ の参照を増減していない。設計書の code block に足した関数（`get_daily_report_sheet`・`get_daily_report_sheet_lines`・`get_active_z004_import_totals`）は code に無い関数として設計照合 test の INFO になり、失敗にならない（下の AC12 で確かめる）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/*`・`docs/templates/*`・`.agents/*`・`.claude/{rules,commands,skills}/*`・PR template と、full に倒す `scripts/*`・`.github/*`）に当てて、当たる file は無い。Final Review Minimum 1。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime の lane の Writer が、チャットの履歴と repo 外の資料を見ずに、source docs だけで次を実装できる: 日次売上の `日報を印刷` で、その日の完了した日報の取込みを位置と名称の組で足した 1 日 1 枚の紙を、A4 縦・文字 9 pt 以上・色なし・今の Excel の横長の並びを写さない形で印刷する（34 §19.3a SALES-SHEET-D1〜D6、24 §14.21a・§14.21b、42 `get_daily_report_sheet`、56 UI-09a-D17・D18、UI_TECH_STACK §6.5.5）。PLU の売上は売れた行だけが紙に載る（SALES-SHEET-D4）。
- 日報の意味づけのうち実データで効いていない所が事実に合う: 同じ日の取込みをまたぐ支払の合算の鍵が位置と名称の組になり（24 §14.21 手順 3）、`credit` の規則を外し、半角カナの正規化を入れない理由が書かれている（29 IO-07-D4a）。
- 紙の切替えの時期と、それまで店がすること、切り替える前の確かめが正本（UI-09a-D19）にある。
- owner に聞く見た目の問い（紙の並びの案 3 つ、切替えの日）が、答えが無い間の既定とともに packet の Open Questions と正本にあり、案の比べ方（実機の印刷で並べる）が runtime の lane の L3 の項目になっている（Matrix の Manual）。
- D-120 が decision-log にあり、後続の runtime の lane の範囲と Risk が backlog にある。

### 失敗定義

- 紙が精算ごとに分かれる、または今の Excel の横長の並び（4 つの帳票を横に並べて縮小する形）を写す。
- 足す鍵が並び順だけか名称だけで、名称の差し替えの日に別の行を足すか、1 本の中の完全一致の同じ名称の行を 1 つにまとめる。
- 収まらないときに 9 pt を割って縮める契約になる。
- 半角カナを正規化して `credit` を効かせる（4 つの行が同じ鍵になる）。
- 紙の値がレジスターツールの日報と同じであることを確かめたように書く（未確認の前提を確かめた事実として書く）。
- 並走の lane（D-121）が所有する取込みの経路（37、45、55、29 §29.7・§29.8、pos-tables §12b・B-2、D-071 の日報の側）を書き換える。
- 公開 repo に店の運用の細部・金額・商品名・JAN・部門名・発言の原文・実データの値を書く。

### 非目的

- code・bindings・test の変更（runtime の lane）。
- 取込みの経路・同日追加の確認・SD の読取り（D-121 と D-111）。
- 日次売上の画面の Z001 を 1 つの表に足すこと（D-096 のまま。Revisit の条件は D-120）。
- 月次の印刷（月次の印刷ボタンは disabled のまま）。
- 紙の並びの最終の決定（owner が実機の印刷で選ぶ。既定だけを置く）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、締めの日と切替えの前後の操作列を置く。本 lane は docs だけで、表の通常運用は後続の runtime の lane の後に成り立つ。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店が締めの後にアプリから日報の紙を印刷し、Excel への貼付けをやめる）ことは別で、後者は本 lane では未達。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| go-live の前（店のアプリの運用を始める前） | UI-09a-D13 の導入前受入: 実際の 1 日分を日次売上の画面と今の Excel で横に並べる（公式集計・過去日・未取込みの日・取り込み直した後・backup/restore の後） | 画面が日報の記録を代わりに残せるかが分かる | 受入が済んだら運用を始める | — |
| runtime の lane を入れた版。運用開始の後、切替えの前。店は今の Excel への貼付けを続けている | 締めの後に日報を取り込み（今の取込みの画面）、日次売上を開く | 印刷ボタン `日報を印刷` が押せる（その日の完了した取込みがある） | — | — |
| 同じ | `日報を印刷` を押す | 紙の内容を読み直し、OS の印刷の dialog が出る。プレビューに A4 縦の 1 枚（上に日付、左に日計と部門、右に支払、下に PLU） | 印刷先を選んで印刷 | P1（WebView2 の印刷） |
| 同じ日に 2 回精算して 2 回取り込んだ日 | `日報を印刷` | 2 回分を位置と名称の組で足した 1 枚。上に `2回の取込みを合算` | — | P2（レジスターツールの日報と同じ値か） |
| 同じ日の精算の間で行がずれ、1 回の精算の中に同じ名称の行が 2 行ある日（手元の集計では起きていない） | `日報を印刷` | その帳票だけ足さずに `1回目の取込み`・`2回目の取込み` の表に分けて載り、見出しの行の下と画面に理由の文が出る。ほかの帳票は足し、題は `2回の取込みを合算（取込みごとに載せた帳票を除く）`。日次売上の支払集計も足さずに並び、見出し `日報の確認が必要です` の warning の注記が、取込みごとの区切りを紙で見られると案内する。画面の 2 回以上の日の文は足していない表を除くと書く | — | P3 |
| まだ取り込んでいない日 | 日次売上を開く | ボタンは押せず、下に `この日のレジ日報は未取込みのため、印刷できません。` | 取り込む | — |
| 画面を開いた後に、別の画面でその日の取込みを取り消した | `日報を印刷` を押す | 読み直しで `import_count` が 0 になり、印刷せずにボタンの下に未取込みの文が出る（古い内容で印刷しない） | 取り込み直す | — |
| Z004 の取込みが止まっている間（停止 ADR） | `日報を印刷` | PLU の部分は `この日の PLU の売上は取り込まれていません。` の 1 行。日報の部分は出る | — | P6 |
| runtime の lane の L3 | owner が並びの案 a・b・c を実機のプリンタで印刷して並べる | 案を選ぶ（答えが無ければ a） | 選んだ案が既定と違えば Gated Amendment で UI-09a-D17 を直す | P5（1 枚に収まるか） |
| 店のアプリの運用が始まった後の実際の 1 日（D13 の導入前受入の後） | アプリの紙と今の Excel の紙を並べる（同じ日に 2 回以上精算した日があればその日も） | 値が同じことを確かめる（UI-09a-D19 の (2)。D13 の導入前受入とは別） | 同じなら切替えの日を決める | P2 |
| 切替えの日（月の初め、12 月にかかるなら元日。Q2） | 締めの後に `日報を印刷` で印刷して綴じる。Excel への貼付けをやめる | 手で写す作業が無くなる（TD-260） | — | — |
| 切替えの後、紙に不都合が見つかった | 翌日から Excel の貼付けに戻す | アプリの側は何も変わらない。過去の日の紙はいつでも印刷し直せる | 原因を直した版で切り替え直す | — |

## Scope

本 lane は docs だけ。下の file の、下の節を書き換えた（全部 plan-first の commit）。

1. 紙の内容と足し方（Scope の 1、発注の Scope 1）: [34](../../function-design/34-biz-sales-service.md) に §19.3a `get_daily_report_sheet`（SALES-SHEET-D1〜D6、型 `DailyReportSheet`・`DailyReportSheetRow`・`DailyReportSheetPluImport`、処理ステップ、エラー、runtime の lane のテスト）を足し、§19.3 の手順 3 の支払の集約の鍵と「設計判断」の Z001 の段に紙の例外を書いた。[24](../../function-design/24-io-csv-import-repo.md) の §14.21 手順 3 を SALES-SHEET-D2 と同じ規則にし（規則は D2 の 1 か所に置き、24 は引くだけ）、§14.21a `get_daily_report_sheet_lines`・§14.21b `get_active_z004_import_totals` を足した。[42](../../function-design/42-cmd-sales-stocktake.md) に CMD `get_daily_report_sheet` を足した。round 3 の是正で、24 §14.21 手順 3 の互換の範囲に「各組の保存済みの `payment_key` も全取込みで同じ」を足し、§14.21a の DTO に取込みの id の並び（`import_ids`）を置いて `import_seq` をその位置にし、34 §19.3 手順 3 の warning の文に紙への案内を足した。
2. 紙の並びと印刷の手段（発注の Scope 1）: [56](../../function-design/56-ui-daily-sales.md) に「日報の紙」の節（UI-09a-D17 = 印刷ボタンと押せない状態の文、並びの既定、表と値の書き方、紙の上の知らせ、文字の大きさの下限 9 pt、色なし、画面に出さない。UI-09a-D18 = WebView の印刷と代わりの道）を足し、ファイル構成の `ExportBar` の行・component の `Tooltip` の行・非目的の「印刷ボタンの本実装」の行・更新履歴を直した。[UI_TECH_STACK](../../UI_TECH_STACK.md) に §6.5.5 印刷、[SCREEN_DESIGN](../../SCREEN_DESIGN.md) の日次売上の利用者配慮に 1 行、[ui-task-specs](../../architecture/ui-task-specs.md) の UI-09a の印刷ボタンの行に設計の参照を足した。round 3 の是正で、56 の warning の注記の見出し（`日報の確認が必要です`）、UI-09a-D16 の 2 回以上の日の文と支払集計の段、UI-09a-D15 の加算の段、紙の題、紙の内容の読込みの失敗の `再試行` を直した。
3. 意味づけの見直し（発注の Scope 2）: [29](../../function-design/29-io-daily-report-parser.md) に IO-07-D4a（意味の鍵は `gross_sales`・`net_sales`・`cash` だけ、`credit` の規則を外す、半角カナの正規化を入れない、`payment_key`・`line_key` は足す鍵に使わない。根拠は手元の Z 帳票の 2026-09-30 版での集計の数と位置だけ）を足し、§29.4.1 の layout A の表の Z002 の行と「行の構成と性質」の段に紙の扱いを書いた。[pos-tables](../../db-design/pos-tables.md) §12d の設計意図に足す鍵の段を足した（列・制約は変えない）。
4. 紙の切替えの時期（発注の Scope 3）: 56 の UI-09a-D19（切替えまで店は今の Excel を続ける、切り替える前の 2 つの確かめ、月の初めか元日、既定、戻し方）を足し、UI-09a-D13 の紙の用途を事実（店の控えと店の外の提出先へ渡す用を兼ねる、2026-09-29 の店主の答え、repo 外の回答台帳 TD-059・TD-060）に直し、D13 の導入前受入（go-live の前）と D19 の運用開始の後の紙の照合を別の段に分けた（順は D13 → 運用開始 → D19）。[plu-export-and-real-csv-verification](../../plu-export-and-real-csv-verification.md) の「税理士等への外部提出用途ではない」（2026-08 の旧い記述）を UI-09a-D13 と D-120 への参照に直し、「印刷機能の要否はこの受入結果で決め」に D-120 の決定と、この比較が D13 の導入前受入で D19 とは別であることを足した。
5. 後続と決定（発注の Scope 4）: [backlog](../../backlog.md) の「日報の紙と意味づけ」に設計 lane の起票と決めたことを足し、その下に「日報の紙の runtime」（R3、範囲・L3・依存）を足した。「日報画面の Excel 印刷・バインダー代替受入判定」と「既存の支払集計（Z002）で異なる項目が 1 行に合算されうる」に D-120 の決定を足した。round 3 の是正で、印刷の要否を残していた 3 か所（案 B・案 C の後続候補、「CSV 出力・印刷の実挙動確認」、後続候補の列挙の「日次 / 月次の印刷要否」）に、日次は作ると決めた・月次は未定を足し、D-120 の Compatibility の互換の範囲を 24 §14.21 手順 3 への参照にした。[decision-log](../../decision-log.md) の末尾に D-120 を足し、D-096 の Revisit の後に紙の行を 1 行足した（D-096 の本文は変えていない）。
6. owner に聞く見た目の問い（発注の Scope 5）: 下の Open Questions と、Matrix の Manual の L3 の項目。

ついでに直したもの: 無し。

対象を使う呼出し側・隣接 test・helper / mock・生成物（後続の runtime の lane が触る。本 lane は変えない。`rg` で 2026-10-17 に確かめた）:

- 支払の合算の鍵（24 §14.21 手順 3）を作る所: `src-tauri/src/db/sales_repo.rs` の `get_completed_daily_report_aggregate`（`:1039`、支払の集約は `:1075` の query と `:1113` の group）。読む所: `src-tauri/src/biz/sales_service.rs`（`:584` の写像）、`src/features/daily-sales/DailySalesPage.tsx`（`:223` の行の key に `payment_key` を使う。鍵を変えると同じ `payment_key` の 2 行が出うるので、行の key を部門別集計〈同じ file の `:233`〉と同じ形の `${payment_key}-${index}` にする。wire は変えない。56 の支払集計の鍵の行）。test の fixture・呼び方: `src-tauri/src/biz/sales_service.rs` の test（`:769`・`:892` の `payment_key: "cash"`）、`src-tauri/src/db/sales_repo.rs` の test（`:2291` 付近、支払の集約の test の `:2465` の `payment_key: "credit"` の fixture と `:2507` の assert）、`src/features/daily-sales/DailySalesPage.test.tsx`（`:99`・`:225`。2 回以上の日の文を `:132`・`:263`、warning の注記の見出し `日報の部門確認が必要です` を `:202`・`:205`・`:208` で完全一致で確かめていて、56 の新しい文と見出しに合わせる。見出しの今の code は `DailySalesPage.tsx:182`、文は `:176`）。
- `credit` の規則（29 IO-07-D4a）: `src-tauri/src/io/daily_report_parser.rs` の `payment_key`（`:582`〜`:590`）とその test。production で `credit` を読む処理は無い（2026-10-17 に `rg -n '"credit"' src src-tauri/src` を実行: parser の 2 行〈`:585`・`:586`〉と `src-tauri/src/db/sales_repo.rs` の test の 2 行〈`:2465` の fixture、`:2507` の assert〉に当たる。test の 2 行は保存済みの鍵を集約に通す test で、規則を外しても fixture の値として残せる）。
- 紙の新しい関数と型: IO = `src-tauri/src/db/sales_repo.rs`（24 の設計書は `db::csv_import_repo` と `db::sales_repo` の 2 つに対応づけられている。`src-tauri/tests/design_compliance_test.rs:151`）、BIZ = `src-tauri/src/biz/sales_service.rs`、CMD = `src-tauri/src/cmd/sales_cmd.rs` と `src-tauri/src/lib.rs` の登録、`src/lib/bindings.ts`（再生成）、UI = `src/features/daily-sales/components/ExportBar.tsx`（今の印刷ボタンは `aria-disabled`、`:3`〜`:4` と `:44`）・`DailySalesPage.tsx`・新しい紙の component と hook。日次売上の取込み・取消の後の invalidate の所（56 の主動線、`["daily-sales"]`）。
- 新しい表示文言の test との衝突: `rg -F '日報を印刷' src --glob '*.test.*'` は 0 件、`rg -n '印刷' src` は 2 つの `ExportBar.tsx` だけ（2026-10-17）。

並走の lane と重なる file と行・節の所有（D-055 の所有表。並走の lane の packet はそれぞれの branch にあり、本 lane は引かない）:

| file | 本 lane が触る所 | 売上の SD からの一括取込みの lane（D-121）が所有する所 | merge の後に揃える所 |
|---|---|---|---|
| `docs/function-design/29-io-daily-report-parser.md` | §29.2 の IO-07-D4 の下に足した IO-07-D4a、§29.4.1 の layout A の表の Z002 の行と「行の構成と性質」の段 | §29.7（IO-09）・§29.8（IO-10）と取込みの経路の記述 | 無し |
| `docs/function-design/37-biz-daily-report-import-service.md` | 触らない（§37.3 手順 6 の必須サマリを IO-07-D4a から引くだけ） | 全体（取込みの経路、BIZ-08-D4 の同日追加の照合、§37.9） | 無し |
| `docs/function-design/24-io-csv-import-repo.md` | §14.21 手順 3、§14.21a・§14.21b（新設） | §14.18a など取込みの側の関数 | 無し |
| `docs/function-design/34-biz-sales-service.md` | §19.3 手順 3・設計判断、§19.3a（新設）、更新履歴の 1 行 | 触らない見込み（EJ の閲覧が日次売上に出るなら §19.2 に field を足しうる） | 両方が §19.2 を触ったら field を両方残す |
| `docs/db-design/pos-tables.md` | §12d の設計意図の 1 段 | §12b（同日追加・精算回数）と B-2 | 無し |
| `docs/SCREEN_DESIGN.md` | 日次売上画面の利用者配慮の 1 行 | 売上データ取込み画面の節 | 無し |
| `docs/decision-log.md` | D-096 の Revisit の後の 1 行、末尾の D-120 | 末尾の D-121、D-071・D-111 への追記 | 末尾は両方を残す（番号の順） |
| `docs/backlog.md` | 「日報の紙と意味づけ」の項と、その下に足した「日報の紙の runtime」、受入判定の項と Z002 の合算の項の末尾 | 「売上の SD からの一括取込みと EJ の控えの閲覧」の項 | 無し |

SD 直読みの runtime の lane は D-111 の既決の契約の実装で、本 lane はその契約（29 §29.7・§29.8、37 の BIZ-08-D3〜D6、45、55）を変えない。他の 3 本（単位の語の直し、JAN の別名、MSI）とは file が重ならない見込み（decision-log・backlog の末尾の追記を除く）。

## Non-scope

- code・bindings・test（日報の紙の runtime の lane。backlog の項）。
- 取込みの経路・同日追加の確認（D-071 の日報の側）・SD の読取り（D-121、D-111）。37・45・55 の正本。
- 日次売上の画面の Z001 の表示（D-096 のまま）、部門の照合と月次の部門集計（変えない）。
- 月次の印刷、プリンタの設定の画面。
- Z004 の取込みの再開（停止 ADR と D-118・D-121 の後続）。
- `docs/Plans.md`（lane の行を足さない。D-097）、`docs/project-memory.md`（紙の用途は `:41` に店主の答えが既にある。本 lane の出典は台帳の TD-059・TD-060 を直接引くので、足さない）。

## Acceptance Criteria

数は開始 HEAD `f5d7aaf4`（`git archive f5d7aaf4 docs` を `$TMPDIR/old` に展開した版）と起票後の版を同じ数え方（その file の中で式に当たる行の数、`rg -c -F`）で測った。出力が無いのは 0。

- AC1（紙の BIZ）: `rg -c -F 'SALES-SHEET-D' docs/function-design/34-biz-sales-service.md` が 17（開始 0。round 2 の是正の前の版〈`59c0ebcc`〉は 12。D2 を場合分けの段に書き直し、§19.3 手順 3・D3・処理ステップ・テストから引いた）、`rg -c -F 'fn get_daily_report_sheet(' docs/function-design/34-biz-sales-service.md` が 1（開始 0）、`rg -c -F '日報の紙は §19.3a で Z001 も足す' docs/function-design/34-biz-sales-service.md` が 1（開始 0）。
- AC2（紙の IO と支払の合算の鍵）: `rg -c -F 'fn get_daily_report_sheet_lines(' docs/function-design/24-io-csv-import-repo.md` と `rg -c -F 'fn get_active_z004_import_totals(' docs/function-design/24-io-csv-import-repo.md` がどちらも 1（開始 0）。`rg -c -F 'は `payment_key` で集約する' docs/function-design/24-io-csv-import-repo.md` が 0（開始 1）。
- AC3（CMD）: `rg -c -F 'fn get_daily_report_sheet(' docs/function-design/42-cmd-sales-stocktake.md` が 1（開始 0）。
- AC4（紙の画面と印刷）: `docs/function-design/56-ui-daily-sales.md` で `rg -c -F 'UI-09a-D17'` が 6、`'UI-09a-D18'` が 2、`'UI-09a-D19'` が 2、``'`日報を印刷`'`` が 4（round 2 の版〈`bf82ea2a`〉は 3。round 3 の是正で支払集計の段に紙への案内を足した）、`'9 pt 以上'` が 1（開始はどれも 0）。`rg -c -F '#### 6.5.5 印刷' docs/UI_TECH_STACK.md` が 1、`rg -c -F 'UI-09a-D17〜D19' docs/SCREEN_DESIGN.md` が 1（開始 0）。
- AC5（古い記述の撤去）: `rg -c -F 'バインダーは外部提出帳票ではなく' docs/function-design/56-ui-daily-sales.md` が 0（開始 1）、`rg -c -F '印刷機能は受入結果から必要性を決め、' docs/function-design/56-ui-daily-sales.md` が 0（開始 1）、`rg -c -F '税理士等への外部提出用途ではない' docs/plu-export-and-real-csv-verification.md` が 0（開始 1）、`rg -c -F '名称だけの鍵では、1 本の中に同じ文字を含む行' docs/function-design/34-biz-sales-service.md` が 0（plan-gate の版 1）、`rg -c -F 'opener:default`）で OS の既定のブラウザに開き' docs/UI_TECH_STACK.md` が 0（plan-gate の版 1）。
- AC6（意味づけ）: `rg -c -F 'IO-07-D4a' docs/function-design/29-io-daily-report-parser.md` が 4（開始 0。round 1 の是正で D4 の 2 行に参照を足した）、`rg -c -F 'TD-305' docs/function-design/29-io-daily-report-parser.md` が 1（開始 0）、`rg -c -F '同じ日の取込みをまたいで足す鍵' docs/db-design/pos-tables.md` が 1（開始 0）。
- AC7（決定の記録）: `rg -c -F '## D-120: ' docs/decision-log.md` が 1、`rg -c -F -- '- 紙（2026-10-17、D-120）' docs/decision-log.md` が 1（開始 0）。D-096 の本文が変わっていない: `awk '/^## D-096/,/^- Revisit/'` を開始の版と今の版の `docs/decision-log.md` に当てた出力を `cmp` して同じ。
- AC8（backlog）: `rg -c -F -- '- **日報の紙の runtime**' docs/backlog.md`、`rg -c -F '2026-10-17: D-120 で直し方を決めた' docs/backlog.md`、`rg -c -F '→ 2026-10-17: 印刷機能は作ると決めた（D-120' docs/backlog.md` がどれも 1（開始 0）。
- AC9（並走の lane の節を触らない）: `git diff --name-only f5d7aaf4 HEAD -- docs/function-design/37-biz-daily-report-import-service.md docs/function-design/45-cmd-daily-report-import.md docs/function-design/55-ui-csv-import.md` の出力が空。`git diff -U0 f5d7aaf4 HEAD -- docs/function-design/29-io-daily-report-parser.md | grep '^@@'` の hunk がどれも 212 行目（§29.7 の見出し）より前。`git diff -U0 f5d7aaf4 HEAD -- docs/db-design/pos-tables.md | grep '^@@'` の hunk が §12d の設計意図（開始の版の 201〜203 行目の直後）だけ。
- AC10（docs だけ）: `git diff --name-only f5d7aaf4 HEAD -- src src-tauri scripts package.json package-lock.json` の出力が空。
- AC11（公開してよい形）: `git diff -U0 f5d7aaf4 HEAD -- docs` の追加の行に、店の金額・商品名・JAN・部門名・発言の原文・実データの名称の値が無い（数・位置・帳票の名前と、レジの帳票の総称〈総売・純売・現金・クレジット〉だけ）。Plan Review で差分を読んで確かめる。
- AC13（round 2 の是正。数は `git archive 59c0ebcc docs` を `$TMPDIR/r1` に展開した版と今の版を同じ数え方で測った）: 34 で `rg -c -F 'unsummed_sources'` が 4・`'import_seq'` が 4・`'payment_lines_unsummed'` が 1、24 で `'payment_lines_unsummed'` が 1・`'SALES-SHEET-D2 の規則で返す'` が 1（どれも `59c0ebcc` は 0）。56 で `'この帳票は足さずに取込みごとに載せています。'` が 1、``'`/inventory/records`'`` が 1、`'次の紙への続け方（既定）'` が 1、`'この受入が先'` が 1、``'`.html` に関連付けられた既定のアプリ'`` が 1、`'command（名前・引数・返り値・§22.8 の登録）を足す Gated Amendment'` が 1（どれも `59c0ebcc` は 0）、`'売上データ取込みの記録で見られます'` が 0（`59c0ebcc` は 1）、`'UI-09a-D13 の受入と一緒'` が 0（同 1）、`'紙の切替えの前の確かめ（UI-09a-D19）と一緒に行う'` が 0（同 1）。`'既定のブラウザ'` が 56・`docs/UI_TECH_STACK.md`・`docs/decision-log.md`・`docs/backlog.md` のどれも 0（`59c0ebcc` は 2・1・1・2）。``'`.html` に関連付けられた既定のアプリ'`` が `docs/UI_TECH_STACK.md` で 1（同 0）。`'紙の切替えの前の確かめと一緒に行う'` が `docs/plu-export-and-real-csv-verification.md` で 0（同 1）、`'紙の切替えの前の確かめ（56 UI-09a-D19）と一緒に行う'` が `docs/backlog.md` で 0（同 1）、`'この確かめに含める'` が `docs/decision-log.md` で 0（同 1）。Matrix で `'test_sheet_rows_unsummed_when_rows_shift_with_duplicate_label'` が 3、`'test_daily_sales_payment_unsummed_matches_sheet_rule'` が 2、`'test_sheet_rows_blank_labels_do_not_trigger_unsummed'` が 2（同 0）。
- AC14（round 3 の是正。数は `git archive bf82ea2a docs` を `$TMPDIR/r2` に展開した版と今の版を同じ数え方で測った）: 24 で ``'各組の保存済みの `payment_key` も全取込みで同じ'`` が 1、``'今は `payment_key` ごとの 2 行に分かれ'`` が 1、`'import_ids'` が 3（どれも `bf82ea2a` は 0）、``'親の数を `import_count` にする'`` が 0（同 1）。34 で `'紙（日報を印刷）で取込みごとに見られます'` が 1、`'import_ids'` が 4、``'3 本目の行の `import_seq` は 3'`` が 1（同 0）。56 で `'日報の確認が必要です'` が 1、`'注記で足していないと示した表を除く'` が 1、`'取込みごとに載せた帳票を除く'` が 1、``'既存と同じ `再試行` ボタン'`` が 1（同 0）、`'日付を選び直すと読み直す'` が 0、`'今と同じ行と値になるのは'` が 0（同 1）。`docs/decision-log.md` で ``'1 取込みの中に同じ `payment_key` の行が無い日に限る'`` が 0（同 1）、`'範囲の外の日の値は 24 §14.21 手順 3 が持つ'` が 1（同 0）。`docs/backlog.md` で `'月次の印刷の要否は未定'` が 2、`'日次は作ると決めた〈D-120、56 UI-09a-D17〉。月次は未定'` が 1（同 0）。Matrix で `'test_payment_aggregate_merges_pair_with_mixed_stored_payment_key'` が 2、`'test_sheet_rows_import_seq_follows_import_ids'` が 2、`'DailySalesPage texts stay true on unsummed payment day (UI-09a)'` が 1、`'DailyReportSheet title on unsummed day (UI-09a)'` が 1、``'前の `¥20` の行が残る'`` が 1（同 0）。
- AC12（検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0・WARN 0（開始 HEAD の full は WARN 0。開始 HEAD には active packet が無く `--target plan` は対象なしで exit 2）。`cd src-tauri && cargo run --bin generate_traceability -- --check` が OK。`cd src-tauri && cargo test --test design_compliance_test` が pass。

AC の実測（2026-10-18、round 3 の是正の版。commit の前の作業 tree で、上の command をそのまま実行した出力）:

- AC1〜AC8: `SALES-SHEET-D` 17、`fn get_daily_report_sheet(`（34）1、`日報の紙は §19.3a で Z001 も足す` 1、`fn get_daily_report_sheet_lines(` 1、`fn get_active_z004_import_totals(` 1、``は `payment_key` で集約する`` 0、`fn get_daily_report_sheet(`（42）1、56 の `UI-09a-D17` 6・`UI-09a-D18` 2・`UI-09a-D19` 2・`` `日報を印刷` `` 4・`9 pt 以上` 1、`#### 6.5.5 印刷` 1、`UI-09a-D17〜D19` 1、AC5 の 5 つは全部 0、`IO-07-D4a` 4、`TD-305` 1、`同じ日の取込みをまたいで足す鍵` 1、`## D-120: ` 1、`- 紙（2026-10-17、D-120）` 1、AC8 の 3 つは全部 1。AC7 の `cmp` は `same`。
- AC9: 37・45・55 の `git diff --name-only` は空。29 の hunk は `@@ -111 +111 @@`・`@@ -113 +113,2 @@`・`@@ -171 +172 @@`・`@@ -187 +188 @@`（どれも 212 行目より前）。pos-tables の hunk は `@@ -203,0 +204 @@` だけ。AC10 の `git diff --name-only` は空（`f5d7aaf4` と作業 tree の比較）。
- AC12: `bash scripts/doc-consistency-check.sh` は `exit=0`、WARN の行 0、`結果: 全チェック通過`。`--target plan` も `exit=0`、WARN の行 0、`結果: 全チェック通過`。`CARGO_TARGET_DIR=/tmp/shared-cargo-target cargo run --bin generate_traceability -- --check` は `traceability check: OK（ERROR 0 件 / WARN 0 件）`。同じ target の `cargo test --test design_compliance_test` は `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`。
- AC13: 上の AC13 の式を `rg -c -F` で今の版と `$TMPDIR/r1`（`59c0ebcc`）の両方に当て、全部 AC13 に書いた値だった（例 `rg -c -F 'unsummed_sources' docs/function-design/34-biz-sales-service.md` は `4`）。
- AC14: 上の AC14 の式を `rg -c -F` で今の版と `$TMPDIR/r2`（`bf82ea2a`）の両方に当て、全部 AC14 に書いた値だった（例 ``rg -c -F '各組の保存済みの `payment_key` も全取込みで同じ' docs/function-design/24-io-csv-import-repo.md`` は `1`、`$TMPDIR/r2` の同じ file は出力なし〈0〉）。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: 34 §19.2・§19.3（手順 2・3、設計判断）・§19.3a（SALES-SHEET-D1〜D6）／24 §14.21（手順 3）・§14.21a・§14.21b／42 §22.4 の `get_daily_report_sheet`／29 §29.2 の IO-07-D2・D4・D4a・D5、§29.4.1／56 §56.1 の UI-09a-D12・D13・D15・D16・D17・D18・D19、§56.7、§56.13／pos-tables §12c・§12d・§12e／SCREEN_DESIGN の日次売上画面／UI_TECH_STACK §6.4・§6.5.1・§6.5.5／D-022・D-025・D-096・D-104・D-111・D-120、停止 ADR（`docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md`）と 32 §15.0・35 §20.0 の「現行buildの一時停止」（PLU の部分が出ない理由としてだけ引く）。
- 必要な設計成果物（Design artifact selection の当たる行）: BIZ・IO の振舞い = updated in this PR（34、24、29）／command・DTO = updated in this PR（42、34 の型。wire の変更は後続、下の Boundary / Wire Contract）／report の形 = updated in this PR（34 SALES-SHEET-D3・D4、56 UI-09a-D17）／operator workflow・文言 = updated in this PR（56 UI-09a-D17・D19、SCREEN_DESIGN、UI_TECH_STACK §6.5.5）／表・migration = existing sufficient（列・制約を変えない。pos-tables §12d に足す鍵の段だけ）／durable な判断 = updated in this PR（D-120、D-096 の 1 行）。
- plan にしかない durable な判断の昇格先: 決めたことは全て正本と D-120 に置いた。packet にだけあるのは、規模と手法の数え、後続の lane への file の申し送り（Scope の末尾）、所有表、AC の数、Open Questions の文面。
- 前提・制約と、延期した design gap の follow-up: WebView2 の印刷（P1）は runtime の lane の実装の前、紙の値とレジスターツールの日報の一致（P2）と 1 枚に収まるか（P5）は runtime の lane の L3。紙の並びの案と切替えの日は owner の答え待ちで、答えが無い間の既定を正本に書いた。延期が安全な理由: どれも今の code の振舞いを変えず、既定は今の画面を変えない（印刷ボタンが disabled から押せるに変わるのは runtime の lane）。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「紙は 1 日 1 枚」は精算ごとに分けないことを指し、行が多い日の 2 枚目は許す（UI-09a-D17 に書いた）。「名称のある行で別の項目を 1 行に足さない」は SALES-SHEET-D2 の 3 つの場合分けで成り立つ（組の集合が同じ日は各位置の名称が同じ、一致しない日は 1 本の中で名称が 1 行だけなら組で足し、同じ名称が 2 行あれば足さない）。例外は名称が空の行で、位置だけで見分けるので行がずれた日に別の枠の値を足しうる（D2 の保証の範囲に書いた。手元の集計では名称が空の行は全本で値が 0）。1 回の精算の中で同じ位置に 2 つの行がある場合は、位置は行の番号なので起きない。「取り消した取込みは紙に入らない」は読取りが `status='completed'` だけを読む（24 §14.21a）ことに依る。「同じ精算を二重に足さない」は保証しない: 精算回数の読めない束を手で選んだ場合（BIZ-08-D4 が照合しない）は二重に足しうる（D-120 の Guarantee range に書いた）。「Z005 は部門マスタと照合しない」は紙だけで、画面と月次は照合を続ける。
- 判定（ready / not ready）と理由: ready（plan-draft に進める）。理由: 設計の出力が上の正本と D-120 にあり、owner の判断事項（1 日 1 枚・並びの方針）は TD-280・TD-296 で決まり、見た目の細部と切替えの日は答えが無い間の既定を持つ問いとして正本と Open Questions にあり、残る外部前提は runtime の lane の実装の前と L3 で確かめるもので設計の問いではない。

## Registration / Generation Obligations

本 lane: REQ の参照を増減していないので `docs/function-design/90-traceability.md` を再生成しない（`cargo run --bin generate_traceability -- --check` で確かめる、AC12）。function-design の file の新設・改名・削除は無い。後続の義務（runtime の lane）: Tauri command `get_daily_report_sheet` を `lib.rs` の specta `collect_commands` と `invoke_handler` に登録し、`#[tauri::command]` と `#[specta::specta]` の対を付け、`cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成する（42 §22.8）。route は足さない（印刷は日次売上の画面の中で行う）ので `npm run generate:routes` は要らない。operator 画面の新設は無い（到達導線は既存の日次売上）。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たる。帳票の行の位置・名称・`credit` の鍵の導き方はレジの adapter（IO-07）の事実。紙の足し方（位置と名称の組）と紙の並びは app-core（BIZ-05・UI）。レジが変わると IO-07 が行を作り直し、紙の規則は残る | 29 IO-07-D4a、34 SALES-SHEET-D2 |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。事実 = 店主の答え（1 日 1 枚、合算の表示、縦の印刷は字が小さい、時期を選ぶ）、手元の Z 帳票の 2026-09-30 版での数と位置。未確認の前提 = 合算がレジスターツールの仕様か・行ごとに同じ値か、WebView2 の印刷。決定 = 足す鍵・並び・下限・切替えの既定 | Contract Probe、D-120 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。紙は保存せず印刷のたびに作る（取消・追加の後は次の紙が変わる、印刷日時で見分ける）。読込みの失敗は印刷を止める。印刷の dialog の取消は何も残さない | 34 SALES-SHEET-D5・D6、56 UI-09a-D17、Matrix |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。締め → 取込み → 日次売上 → `日報を印刷` → 綴じる。切替えまでは今の Excel を続け、切り替えた後も戻せる | Ordinary Operation、56 UI-09a-D19 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たる（小）。レジが変わると IO-07 と帳票の見出しの語（`Z001` 等）が変わる。紙の足し方と並びの規則は残る | D-120 の Revisit |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。tracked には数・位置・帳票の名前だけを書き、名称の値・金額・部門名を書いていない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。紙は日報（レジの集計）と PLU の商品別売上を別の表に置き、互いに足さない（D-025）。手動販売は紙に載せない。NULL は 0 に見せない | 34 SALES-SHEET-D2・D4 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる（後続）。WebView2 の印刷、実機のプリンタでの見え方と 1 枚に収まるか、レジスターツールの日報との値の一致は runtime の lane の L3。本 lane は docs だけで L3 は無い | Matrix の Manual |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たる（小）。印刷は OS のプリンタと WebView2 に依る。新しい依存は足さず、代わりの道も導入済みの plugin で作る。CI では印刷を試さない（L3） | UI_TECH_STACK §6.5.5 |

## Boundary / Wire Contract

本 lane は wire を変えない（bindings も変えない）。後続の lane が足す wire の契約を正本に書いた。

- producer: CMD-09 `get_daily_report_sheet`（42）、BIZ-05 `get_daily_report_sheet`（34 §19.3a）
- consumer: UI-09a（日次売上の `ExportBar` と紙の内容の描画）
- wire type: `DailyReportSheet { date: string, import_count: number, summary_rows / payment_rows / department_rows: DailyReportSheetRow[], label_changed_sources: DailyReportSourceKind[], unsummed_sources: DailyReportSourceKind[], plu_rows: DailySaleItem[], plu_import: { import_count: number, skipped_count: number } }`、`DailyReportSheetRow { position: number, label: string, quantity: number | null, count: number | null, amount: number | null, import_seq: number | null }`（`import_seq` は SALES-SHEET-D2 の 3 の帳票の行だけ）。`DailyReportSourceKind` と `DailySaleItem` は既存の wire の型を使う
- internal type: IO の DB DTO `DailyReportSheetLinesRow`（取込みの id の並び `import_ids: Vec<i64>`〈`imported_at ASC, id ASC`〉と行。BIZ はその長さを `import_count`、行の取込みの位置を `import_seq` にする）・`DailyReportSheetLineRow`（個数は `quantity_hundredths: Option<i64>`）。BIZ が 100 倍の整数で足してから単位の数（`f64`）に戻す（IO-07-D2）
- precision/range: 金額・件数は i64 を checked で足し、溢れは `ValidationFailed`。安全な整数の範囲での拒否はしない（1 日の取込みは数回で、1 行の金額はレジの桁に収まる。34 冒頭の「数量と単位」の (d) と同じ考え）。個数は小数 2 桁まで
- round-trip path: 取込み（`daily_report_*_lines`）→ IO の読取り → BIZ の足し算 → CMD → UI の描画 → `window.print()`。保存も往復もしない（片道）
- invalid input: 日付の形式・暦の誤り → `ValidationFailed`（§19.3 と同じ文言）。取込みが 0 件の日は error でなく `import_count = 0`
- compatibility: 新しい command を足すだけで、既存の command・DTO・DB は変えない。支払集計の合算の鍵の変更で今と同じ行と値になる範囲と、範囲の外（1 取込みの中に同じ `payment_key` の行がある日、同じ組の保存済みの `payment_key` が取込みで違う日）の新しい値は 24 §14.21 手順 3 が持つ。返す `payment_key` の意味は変えず、画面の行の key を `${payment_key}-${index}` にする。`payment_key` の列は残す。SALES-SHEET-D2 の 3 の日（組の集合が一致しない日だけで、上の範囲の外）は支払の行を足さずに取込みの古い順に返し、既存の `warnings: string[]` に文を 1 件足し、画面の 2 回以上の日の文（括弧の部分が増え、全部の 2 回以上の日で変わる）と warning の注記の見出し（部門の warning だけの日も変わる）を、この日にも事実と合う語に直す（56 の UI-09a-D16 の支払集計の段。`OfficialDailyReportSummary` と `OfficialDailyPaymentLine` の wire の型は変えない。IO の DB DTO に内部の field `payment_lines_unsummed` を足す）

## Test Plan

Test Design Matrix: [2026-10-17-daily-report-sheet-design](test-matrices/2026-10-17-daily-report-sheet-design.md)（後続の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh` と `--target plan`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`（AC12）。
- negative tests: Matrix の Negative Paths（後続）。
- compatibility checks: Matrix の Compatibility Checks（名称の変わらない日の支払集計が今と同じ）。
- data safety checks: 本 lane の差分に実データ（金額・商品名・JAN・部門名・名称の値・発言の原文）を入れない（AC11）。
- main wiring/integration checks: 後続の lane（日次売上 → CMD → BIZ → IO、印刷の dialog）。
- L3（後続の runtime の lane の Human Gate の manual）: 項目ごとの到達経路・入力物・依存は Matrix の Manual。受理される入力物は、日報の取込みを済ませた開発の DB（合成の Z001/Z002/Z005 の束 2 つを同じ日に取り込んだ日を含む）と、店の実際の 1 日（店のアプリの運用の後）。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（締めの後にアプリから 1 日 1 枚の紙を印刷し、手で写す作業を無くす）を達成できるか。切替えの前後で店が迷わず、戻せるか。
- SALES-SHEET-D2 の 3 つの場合分け（組の集合が同じ／一致せず 1 本の中の名称が重ならない／一致せず 1 本の中に同じ名称の行がある）が、名称の差し替えの日、行の数か並びが変わった日、1 本の中の完全一致の同じ名称の行のどれでも、名称のある行で別の項目を足さないか。名称が空の行の例外の書き方が保証の範囲として足りるか。紙と画面の支払集計が同じ規則を引いているか。画面の支払集計の鍵の変更が、互換の範囲（24 §14.21 手順 3）の値を変えないか。足さない日の画面と紙の文が事実と合うか。
- 紙の行の選び方（SALES-SHEET-D3）と PLU の部分（D4）が、取り込んでいない日・売れなかった日・数えていない行を取り違えずに出すか。手動販売を載せないことが妥当か。
- 未確認の前提（P1・P2・P5）を確かめた事実のように書いていないか。手元の資料の数に版の時点が付いているか。
- 並走の lane（D-121）の節を触っていないか（AC9、所有表）。D-096・D-111・D-025 と矛盾しないか。

## Open Questions

owner に聞く見た目の問い（owner は記号か短文で答える。介入は 1 回で 2 つの decision point）。比べ方は runtime の lane の L3 で、案を実機のプリンタで印刷して並べる（owner のデザインの好み: 新案は実機で並べて見せる、色は意味がある時だけ）。答えが無い間は既定で runtime の lane を進め、L3 で並べた後に答えをもらう。

- Q1 紙の並び（UI-09a-D17）:
  - a = A4 縦。上に日付、左の段に日計と部門、右の段に支払、下に PLU。全部の行を載せ、文字は 9 pt 以上（既定。PLU の行のある日は 2 枚目へ続く見込み、Contract Probe の P5）
  - b = A4 縦 1 段。値が全部 0 の行を外し（総売・純売・現金の行は残す）、文字を 12 pt 以上に大きくする
  - c = A4 縦 2 枚。1 枚目に日計と支払、2 枚目に部門と PLU。全部の行を載せ、文字は 12 pt 以上
  - 推奨: a（confirmed ではなく candidate。行を減らさず今の紙の項目を全部残せる。字の大きさが足りなければ b か c）。答えが無いときの安全側の既定: a（紙は店の外へ渡す用も兼ねるので、行を黙って減らさない）
- Q2 紙の切替えの日（UI-09a-D19）:
  - a = 店のアプリの運用が始まり、アプリの紙と今の Excel の紙を並べて確かめた後の最初の月の初め（12 月にかかるなら元日）（既定）
  - b = 元日（次の 1 月 1 日）まで待つ
  - 推奨: a（precondition-dependent: アプリの運用の開始と並べた確かめの後）。答えが無いときの既定: a。どちらでも、切り替える日は owner が店と決める

店に聞くこと（答えが無い間は既定で進める。owner が次に店へ行くときにまとめてよい）:

- S1 紙を渡す先が、紙の並びや項目を気にするか。既定: 気にしない前提で進め、切り替える前に新しい紙を 1 枚見せて確かめる（D-120 の Revisit）。

agent 側で解く問い（設計の問いではない。runtime の lane で確かめる）: Contract Probe の P1・P2・P5。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SALES-SHEET-D1（1 日 1 枚、完了した取込みを全部） | 34 §19.3a | runtime: `sales_service.rs`・`sales_repo.rs` | runtime（Matrix の D1 の行） | 非対象 |
| SALES-SHEET-D2（位置と名称の組で見分ける 3 つの場合分けと保証の範囲、NULL の伝播、100 倍の整数で足す、溢れ、並び、`label_changed_sources`・`unsummed_sources`・`import_seq`、Z005 を照合しない） | 34 §19.3a、24 §14.21a | runtime: BIZ の足し算 | runtime（Matrix の D2 の行） | L3: レジスターツールの日報と値を比べる（P2） |
| SALES-SHEET-D3（載せる行、名称の空の行） | 34 §19.3a | runtime: BIZ | runtime（Matrix の D3 の行） | L3: 並びの案の比較（Q1） |
| SALES-SHEET-D4（PLU は売れた行だけ、手動販売を載せない、`plu_import`） | 34 §19.3a、24 §14.21b | runtime: BIZ・IO | runtime（Matrix の D4 の行） | 非対象 |
| SALES-SHEET-D5（失敗の順、空の日） | 34 §19.3a | runtime: BIZ | runtime（Matrix の D5 の行） | 非対象 |
| SALES-SHEET-D6（保存しない、押したときに読み直す） | 34 §19.3a、56 UI-09a-D17 | runtime: BIZ・UI | runtime（Matrix の D6 の行） | 非対象 |
| CMD `get_daily_report_sheet`（登録と bindings） | 42 §22.4・§22.8 | runtime: `sales_cmd.rs`・`lib.rs`・`bindings.ts` | runtime（command registry drift の検査、bindings の clean diff） | 非対象 |
| 24 §14.21 手順 3（支払の合算を SALES-SHEET-D2 と同じ規則に。3 の日は足さずに並べ warning） | 24 §14.21、34 §19.3 手順 3、pos-tables §12d | runtime: `sales_repo.rs`・`sales_service.rs` の warning・`DailySalesPage.tsx` の行の key | runtime（Matrix の 14.21 の行） | 非対象 |
| IO-07-D4a（`credit` の規則を外す、正規化しない） | 29 §29.2 | runtime: `daily_report_parser.rs` | runtime（Matrix の D4a の行） | 非対象 |
| UI-09a-D17（印刷ボタンと押せない状態の文、並びの既定、表と値、知らせ、9 pt、色なし、画面に出さない） | 56 §56.1 | runtime: `ExportBar.tsx`・紙の component・hook | runtime（Matrix の D17 の行） | L3: 実機の印刷で並びと字の大きさ（Q1、P5） |
| UI-09a-D18（WebView の印刷、代わりの道〈`.html` の関連付けを確かめる、採るときは 42 の Gated Amendment〉） | 56 §56.1、UI_TECH_STACK §6.5.5 | runtime: 印刷の呼出しと `@media print` | 非対象（印刷の dialog は自動 test に乗らない。呼出しは spy で確かめる） | L3: Windows で dialog と A4 縦（P1）。代わりの道を採るなら店の PC の `.html` の関連付け |
| UI-09a-D19（切替えの時期と確かめ） | 56 §56.1 | 運用（Coordinator と owner） | 非対象 | L3 の後の実際の 1 日（P2）と Q2 |
| UI-09a-D13（紙の用途の訂正、導入前受入は go-live の前で D19 とは別） | 56 §56.1 | 本 lane | AC5・AC13 | 運用: go-live の前の受入（owner と店） |
| D-120、D-096 の紙の行 | decision-log | 本 lane | AC7 | 非対象 |
| 隣接の除外: UI-09a-D12・D15・D16（画面の Z001・合算の表示）、D-096 の本文、24 §14.21 手順 4（部門の集約）・§14.22（月次） | 56、24、decision-log | 変えない | 既存 | 非対象 |
| 隣接の除外: 取込みの経路（37 全体、BIZ-08-D4 の同日追加の照合、45、55、29 §29.7・§29.8、pos-tables §12b・B-2、D-071 の日報の側） | D-121 の lane の正本 | 変えない（所有表） | 非対象 | 非対象 |

## Contract Probe

本 run は手元の資料の新しい集計を行っていない。前提ごとに、出典（repo 外の集計の出力、台帳、tracked の記録、code）を確かめた結果と、確かめた時点を書く。手元の資料の時点は repo 外の版の表で引いた（Z 帳票は 2026-10-14 までの写しが手元にあるが、下の数は 2026-10-04 に 2026-09-30 までの Z を数えた出力から引いた）。

- P1（Windows の WebView2 で `window.print()` が OS の印刷の dialog を出し、`@page` の A4 縦と余白が効く）: 未確認（本 run は Windows で動かしていない。出所は起草役の推測）。→ runtime の lane が実装の前に開発の build で確かめる。効かなければ UI-09a-D18 の代わりの道（HTML を `open_path` で `.html` の既定のアプリで開く。`open_path` はブラウザを指定せず関連付けの先で開く〈`~/.cargo/registry` の `tauri-plugin-opener-2.5.3/src/open.rs:38`・`src/lib.rs:95`、2026-10-18 に読んだ〉ので、採る条件として店の PC の `.html` の関連付けが印刷できるブラウザであることを確かめる。採るときは 42 に command を足す Gated Amendment。`tauri-plugin-opener` は導入済み: `src-tauri/Cargo.toml:25`、`src-tauri/src/lib.rs:1209`、`src-tauri/capabilities/default.json:14`、2026-10-17 に `rg` で確かめた）。
- P2（Z001・Z002・Z005 をその日の精算で足した値が、行ごとにレジスターツールの日報の値と同じ）: 店主の答えで、複数回精算してもレジスターツールの日報は合算で表示される（repo 外の回答台帳 TD-280・TD-305、2026-10-16〜17）。足すことがレジスターツールの仕様か、行ごとに同じ値かは未確認の前提（owner は「多分仕様」と述べた。推測）。取扱説明書は日計明細の精算が集計をクリアすると書く（29 §29.4.1、tracked の記録）。→ 行ごとの照合は未実測。runtime の lane の L3 で、同じ日に 2 回以上精算した日を含む実際の日に比べる。違えば D-120 の Revisit。
- P3（同じ位置の名称が同じ日の精算の間で変わるのは稀）: 手元の Z 帳票の 2026-09-30 版での集計（2026-10-04 に数えた repo 外の出力 `recount-2026-10-04/z_rc.out` の N3・N8 の行、日の数は同じ数え直しの報告の z N2、2026-10-17 に読んだ）で、Z002 の名称の並びは全期間で 3 種類（139 本・2 本・1 本。1 本は Excel で保存し直した形）、Z005 は 2 種類（140 本・2 本）。Z001 は 1 種類（142/142 本）で、同じ日に 2 回以上精算した 19 日のすべてで行の数・名称・並びが精算の間で同じ。行の数は Z001 28・Z002 50・Z005 21 で全本同じ。変わったのは 2022 年の 1 日の精算の間の同じ位置での名称の差し替え（Z002 の 3 つの位置、Z005 の 4 つの位置）だけで、同じ日に 2 回以上精算した 19 日のうち 1 日。1 本の中で名称（空を除く）・コードが重なる例は 0（Z002 の 39〜50 行目は名称が空で、位置で分かれる）。出典は同じ数え直しの報告（repo 外の `material-check-2026-10-04/z.md` の「行の並び」と Z001・Z002 の節）。→ 観測（数と位置だけ）。並び順だけの鍵では、差し替えの日に別の行を足す。行の数か並びが変わる日は観測に無いが、来たら同じ名称が位置ごとに分かれて載り知らせが出るか、1 本の中に同じ名称の行がある帳票は足さずに取込みごとに載る（SALES-SHEET-D2 の 2・3 と保証の範囲）。
- P4（意味の鍵の実データの当たり方）: 同じ集計で、`総売` を含む名称は Z001 の 1 行目だけ（142/142 本）、`純売` は 2 行目だけ、`現金` は Z002 の 1 行目だけ（142/142）、全角の `クレジット` は 0 行、半角の `ｸﾚｼﾞｯﾄ` を含む名称は Z002 の 4 つの位置に全本（ほかに 2022 年の 2 本で 1 つの位置）。Z002 の 39〜50 行目は全本で名称が空で値が 0。→ 観測。`credit` の規則を外しても実データの鍵は変わらず、正規化すると 1 本の中の 4 行が同じ鍵になる。
- P5（既定の並び〈案 a〉が、PLU の行が少ない日に A4 縦 1 枚に 9 pt で収まる）: 未実測（出所は起草役の見積り: 名称のある行は Z001 28・Z002 38・Z005 21〈2026-09-29 版の店の Excel の写しでの数。repo 外の事実の一覧 F-158、`docs/project-memory.md` の締めの Excel の行も同じ数〉で、左の段 49 行・右の段 38 行）。行の高さの前提: 本文 9 pt で行の高さ 1.4 倍（約 4.4 mm）、見出し 11 pt、罫線と表の間の余白は数えない。この前提で、印刷できる高さ 277 mm に対し上の題の行が約 13 mm、左の段が約 238 mm、右の段が約 179 mm で、左の段の後に PLU の見出しの 2 行と 3 行ほどが入る。1 行が 5 mm なら左の段だけで約 266 mm になり、題の行と合わせて 1 枚目を越える（計算は `python3` で 2026-10-18 に行った。式は 56 UI-09a-D17 の見積りの段）。PLU の行のある日は 2 枚目へ続く見込みが高い。→ runtime の lane の L3 で実機の印刷で確かめる。収まらなくても字を小さくせず、56 UI-09a-D17 の「次の紙への続け方」（表の見出しの行と列の見出しを紙ごとに繰り返す、2 段は段ごとに続ける）で次の紙へ続ける。
- P6（Z004 の取込みは停止 ADR が止めている）: `docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md`、32 §15.0・35 §20.0 の「現行buildの一時停止」の見出し（2026-10-17 に `rg` で在ることを確かめた。本文の読みは 2026-10 の見直しの突き合わせ O-19 の引用に依る）。→ 記録。PLU の部分は停止を外す lane までは「取り込まれていません」と出る。
- P7（紙は店の控えと店の外の提出先へ渡す用を兼ね、提出先は紙を全部求める）: repo 外の回答台帳 TD-059・TD-060（2026-09-29 の店主の答え。2026-10-17 に台帳の行を読んだ）。→ 回答。UI-09a-D13・D-120 の Context と、`docs/plu-export-and-real-csv-verification.md` の旧い記述を直した根拠。

## Data Safety

- tracked に書かない: Z 帳票・日報・EJ の中身、名称の値（帳票の総称〈総売・純売・現金・クレジット〉は IO-07 がすでに鍵の導き方として書いている語だけ）、金額・商品名・JAN・部門名、店主・owner の発言の原文、repo 外の台帳・突き合わせの本文。数・位置・帳票の名前と、台帳の ID だけを書いた。
- local-only: repo 外の回答台帳、2026-10 の見直しの事実の一覧と突き合わせ、手元の Z 帳票とその集計の出力、版の表。
- synthetic-only: 後続の lane の test は合成の Z001/Z002/Z005 の束と合成の商品だけを使う。L3 の店の実際の日の紙の写真・PDF を commit しない。

## Implementation Results

- 結果: 日報を手で写して印刷する作業を無くすため、その日の完了した日報の取込みを位置と名称の組で足した 1 日 1 枚の紙（34 §19.3a SALES-SHEET-D1〜D6）、A4 縦・文字 9 pt 以上・色なしの既定の並び、WebView の印刷と代わりの道、紙の切替えの時期（56 UI-09a-D17〜D19）を設計正本に置き、D-120 に記録した。日次売上の支払集計の合算の鍵も紙と同じ規則にし（24 §14.21 手順 3）、`credit` の規則を外して半角カナの正規化を入れない理由を書いた（29 IO-07-D4a）。紙の用途の記述を事実（店の控えと店の外の提出先へ渡す用を兼ねる）に直した。D-096 の本文は変えず、Revisit の後に 1 行足した。本 lane は docs だけで、code・migration・bindings は変えていない（runtime は後続の「日報の紙の runtime」）。
- 既存の保護: 実装は変えていないので、店の動作は今のまま（日次売上の印刷ボタンは押せないまま）。設計照合 test と traceability の検査は green のまま。
- 未検証: Contract Probe の P1（WebView2 の印刷）は runtime の lane の実装の前、P2（レジスターツールの日報との値の一致）と P5（1 枚に収まるか）は runtime の lane の L3。owner への Q1（紙の並び）・Q2（切替えの日）は既定 a のまま。
- review・CI・merge（closeout、2026-10-18）: Plan Review は round 3（天井）までで、round 3 の残りは owner の決定（TD-317）で直して plan-approved。Final Review は broad 1 回と是正の後の closure（下の Review Response の Closeout の段落）。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由の squash merge。PR: [#173](https://github.com/higekiri-w/inventory-system-desktop/pull/173)
- 後続: `docs/backlog.md` の「日報の紙の runtime」（Final Review の closure の P3 の申し送りを含む）と、その起票の前に決める「日次売上と日報の紙の wire の i64 を安全な整数の範囲で扱う横断の規則を決める」。

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

### Plan Review round 1（2026-10-17、head `204b3992`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 2・P3 6）と Codex GPT-6.1 Sol（reject、P1 0・P2 4・P3 3）。全て採った（Coordinator の裁定）。是正の前に相談役（Fable 5.1）に反例探しを頼み、鍵は位置と名称の組のままにし、保証の範囲と知らせの条件を正本に書く方針にした。起草役が `41c7210f` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R1 | Claude F1（P2） | 採る（鍵は変えない） | 行の数か並びが精算の間で変わると同じ名称の行が位置ごとに割れる。名称だけの鍵の棄却の理由が `contains()` の話になっている | 棄却の理由を「1 本の中の完全一致の同じ名称」に直した。P3 に Z001 の数と行の数（全本同じ）、1 本の中の同じ名称の例 0 を足した。保証の範囲（行がずれた日は同じ名称が位置ごとに分かれて載り、知らせが出る。別の項目を足すことは無い）を SALES-SHEET-D2 に書き、知らせの条件を「組の集合の不一致」にした。Matrix に行がずれた日と組の集合が同じ日の 2 行を足した |
| R2 | Claude F2・Codex #5（P3） | 採る | 紙の用途の出典が project-memory に無く、plu-export に逆の live な文が残る | 出典を repo 外の回答台帳 TD-059・TD-060 にし、plu-export の文を UI-09a-D13・D-120 への参照に直した。AC5 に旧い文の撤去の検査を足した |
| R3 | Claude F3・Codex #2（P2） | 採る | 代わりの道の `opener:default` は local の file を開く許可を含まない。書く場所と escape が無い | Rust 側の CMD が app data の固定の file に書き、Rust の API `open_path` で開く（capability を足さない）、値は HTML escape、と UI-09a-D18・§6.5.5 に書いた。Matrix に失敗を成功と扱わない・escape の行を足した |
| R4 | Claude F4・Codex #1（P2） | 採る | wire に位置が無いので「位置と名称から行の key を作る」は実行できない | 返す `payment_key` の意味と wire は変えず、画面の行の key を部門別集計と同じ `${payment_key}-${index}` にした。Matrix に同じ `payment_key` の 2 行の component test を足した |
| R5 | Codex #3（P2）・Claude F5（P3） | 採る | 押したときの読み直しの test が呼出しの順だけで、旧い値で印刷しても通る。読み直しの失敗と 0 件の扱いが無い | 押したときの順を (1) `await refetch()` (2) 失敗か `import_count` 0 なら印刷せず文を出す (3) 描画の後に print、に固定した。Matrix を遅延応答で新旧の値を見分ける形にし、失敗・0 件の行を足した |
| R6 | Codex #4（P2） | 採る | 「名称が変わらない日は今と同じ」が、1 取込みの中に同じ名称の 2 行がある入力と両立しない | 互換の範囲を「組の集合が同じで、1 取込みの中に同じ `payment_key` が無い日」に限り、同じ `payment_key` の 2 行は新しい規則どおり分かれる期待値を Matrix に足した |
| R7 | Codex #6（P3） | 採る | D5 の失敗の順（読取り → 足し算）と処理ステップ（足し算 → PLU の読取り）が逆 | 処理ステップの読取りを 3 つまとめて足し算の前にした |
| R8 | Claude F6（P3） | 採る | `skipped_count` は商品に当たらない行のほかに形式の誤りも数える | 文を `数えていない PLU の行が {N} 行あります` にした |
| R9 | Claude F7（P3） | 採る | Z005 の `label` の列が書かれていない | `raw_department_name` と書き、Matrix に 2 つを違う値にした fixture の行を足した |
| R10 | Claude F8（P3） | 採る | IO-07-D4 の live な文が旧い鍵と旧い 24 への参照のまま | D4 の 2 行に D4a への参照を足した |
| R11 | Codex #7（P3） | 採る | `"credit"` の検索の現物の説明が不正確 | 実測どおり（parser 2 行と DB の test 2 行）に書いた |

あわせて owner の設計の原則（repo 外の回答台帳 TD-310、2026-10-17）で、画面に知らせとその場で直せるリンクを足した（未取込みの日 → `売上データ取込みを開く`、数えていない PLU の行 → `取込みの記録を開く`。どちらも `/csv-import`）。

起草役が残した判断 2 つの Coordinator の裁定（2026-10-18）:

- 行の数か並びが変わった日に、同じ名称の行の合計の行を紙に足すか: 足さない。手元の集計（P3）で起きておらず、起きた日は知らせが出て黙って通らない。足すと紙の様式が変わり、L3 の並びの判断の前に形を決めることになる。実際に起きたら D-120 の Revisit で扱う。
- リンクの行き先を取込みの記録の詳細まで直接にするか: 一覧（`/csv-import`）のままにする。詳細へ直接行くには `plu_import` に取込みの ID が要り、wire が増える。一覧からその日の Z004 の記録へ 1 操作で進める。runtime の lane の L3 で手間が大きいと分かれば足す。

### Plan Review round 2（2026-10-18、head `59c0ebcc`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 2）と Codex GPT-6.1 Sol（reject、P1 0・P2 3）。全て採った（Coordinator の裁定）。起草役が `a305f187` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R12 | Codex #1（P2） | 採る（鍵は変えず、保証の境に歯止め） | 1 本の中に同じ名称の行が 2 つ以上ある帳票で精算の間に行がずれると、位置と名称の組でも別の項目どうしが足される | SALES-SHEET-D2 を 3 つの場合（組の集合が一致 → 足す／一致しないが同じ名称の 2 行が無い → 足して知らせ／一致せず同じ名称の 2 行がある → 足さずに取込みごとに載せて知らせ）の 1 つの規則にし、保証の範囲を同じ段で示した。画面の支払集計も同じ規則を引く。Matrix に紙と画面で同じ fixture の行を足した |
| R13 | Claude #1（P2） | 採る | 数えていない PLU の行のリンクの行き先 `/csv-import` に取込みの記録の一覧が無い | 行き先を入出庫履歴（`/inventory/records`、`recordType: "csv_import"`、その日の精算日で絞る）にし、紙の文も直した |
| R14 | Codex #2（P2） | 採る | UI-09a-D13 の導入前受入と D19 の運用開始の後の照合の順が矛盾する | D13 は go-live の前、D19 はその後と分けた |
| R15 | Codex #3（P2） | 採る | `open_path` は既定のブラウザでなく関連付けの先のアプリで開く | 「`.html` に関連付けられた既定のアプリで開く」に正し、採る条件に関連付けが印刷できるブラウザであることの確かめを足した |
| R16 | Claude #2（P3） | 採る | 代わりの道の CMD の形が正本に無い | 採るときに 42 へ command を足す Gated Amendment を出すと書いた |
| R17 | Claude #3（P3） | 採る | 2 枚目へ続けるときの作り方が決まっていない | 表の見出しの行を紙ごとに繰り返す既定と、行の高さの前提を添えた見積りを書いた |

起草役が出した判断の Coordinator の裁定（2026-10-18）: 画面の知らせの形 `{帳票の見出し}：{文}` は起草役の判断のまま採る（帳票ごとに文が違うため）。足さない日も紙の上の取込みの回数の文は変えない（その帳票の見出しの下に足さない旨の知らせが出る）。名称が空の行の例外は保証の範囲に書いたとおり round 3 の判定に出す。

### Plan Review round 3（2026-10-18、head `bf82ea2a`）

Claude 側 fresh Opus 5.5（approve、P1 0・P2 0・P3 5）と Codex GPT-6.1 Sol（reject、P1 0・P2 3）。Claude #3 と Codex #1 は同じ件。全て採った（Coordinator の裁定）。round の天井で P2 が残ったので owner に諮り、「直して、追加の round なしで plan-approved（跡は Final Review）」と決まった（repo 外の回答台帳 TD-317）。起草役が `09e8e91c` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R18 | Codex #1（P2）・Claude #3（P3） | 採る | 足さない日にも画面の「支払集計を合算」の文、紙の題、部門に限った warning の見出しが残る | 画面の 2 回以上の日の文を足さない表を除く中立の文にし、warning の見出しを `日報の確認が必要です` にし、紙の題は足さない帳票がある日だけ変える。Matrix に画面と紙の検査を足した |
| R19 | Codex #2（P2） | 採る | 互換の範囲に古い保存の鍵と新しい保存の鍵が混ざる日を含めている | 範囲に「各組の保存済みの `payment_key` も全取込みで同じ」を足し、24 §14.21 を範囲の正本にした。混ざる日の値（組で足した 1 行）を Matrix に足した |
| R20 | Codex #3（P2） | 採る | 行の key の test が直しを外しても通る | 更新して再描画した後に 2 行で古い行が残らない確かめを足した |
| R21 | Claude #1（P3） | 採る | backlog に印刷の要否の旧い live な行が残る | 日次は作ると決めた（D-120）旨を足し、月次は後の判断のままと書いた |
| R22 | Claude #2（P3） | 採る | `import_seq` を導く方法が無い | IO の DTO の取込みの数を、取込みの id の並び `import_ids` に置き換え、`import_seq` はその位置とした |
| R23 | Claude #4（P3） | 採る | 足さない日の画面に取込みの区切りが無い | warning に紙で取込みごとに見られる旨を添えた |
| R24 | Claude #5（P3） | 採る | 紙の内容の読込みの失敗に再試行が無い | 既存と同じ `再試行` を置いた |

起草役の判断の Coordinator の裁定（2026-10-18）: IO の DTO の取込みの数を `import_ids` に置き換えたこと（数を 2 か所に分けない）、画面の 2 回以上の日の文を条件で切り替えない中立の文にしたこと（wire を変えない）、§14.21a の読む親を `import_ids` に限ったことは採る。画面の文と見出しの変更は 2 回以上取り込んだ全部の日に及ぶので、owner の好み（壊さない優先）に照らして runtime の lane の L3 で owner が読み違えないかを見る。

Closeout（2026-10-18）: Final Review は broad 1 回で、Claude 側（Fable 5.1）= approve（P3 2）、Codex（Sol）= reject（P2 2・P3 1）。是正は、足す値の順と NULL の優先、名称か並びの変わった日の画面の warning（IO の DB DTO の内部の field を `payment_sum_case` に置き換えた）、Matrix の行。是正の closure（Codex Sol）は approve（P3 1）で、main の取込みの差分は Sonnet 5.5 の delta の確認 = 可。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由で squash merge した。closure の P3（名称か並びの変わった日は、画面に公式セクションの注記と印刷ボタンの下の文の 2 つが出る。足さない日は前から 2 つ）は `docs/backlog.md` の「日報の紙の runtime」へ送った（まとめるかは紙の runtime の lane の L3 の見た目で決める）。packet の本文（Scope・Contract Ledger・AC）は書き換えず、merge の版との食い違いと正しい値をここに記録する: (1) Contract Ledger の「隣接の除外」の行は UI-09a-D15・D16 を「変えない」としているが、本 lane は D15・D16 を変えた（Scope の 2。Final Review の Codex F3）。正本は 56 UI-09a-D15・D16。(2) AC13 の `payment_lines_unsummed` を数える 2 式（34・24）は、Final Review の是正で内部の field を `payment_sum_case` に置き換えたので、merge の版では 1 → 0。新しい anchor は `payment_sum_case`・`LabelChanged` で、24・34 で各 1。Owner Effort Budget の表は closeout の時点の値（上限 9、消費 3）に直した。
