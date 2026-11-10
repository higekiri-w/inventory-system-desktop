# Plan Packet: 売上の SD からの一括取込みと EJ の控えの閲覧の設計

wave に属さない単独の lane（design lane、docs だけ。runtime は後続の lane）。2026-10-17 に Coordinator が同時に起票した 6 本（repo 外の回答台帳 TD-304「動かせるだけ動かす」）の 1 本。並走して file が重なるのは「日報の紙と意味づけ」（branch `agent/daily-report-sheet-design`、D-120）と「SD 直読みの runtime」（branch `agent/sd-direct-read-runtime`）で、所有表は Scope の末尾。本 lane の branch は `agent/sd-bulk-import-design`（起点 main `f5d7aaf4`）。

規模と手法（起票の最初に数えた。file の数は `git diff --name-only f5d7aaf4 HEAD -- docs ':(exclude)docs/plans'` の 12 file）: 触る設計正本は function-design の 6 file（24・29E・32・37・45・55）、db-design の pos-tables と DB_DESIGN、SCREEN_DESIGN、ui-task-specs、decision-log、backlog。画面 2（UI-07 の日報取込みタブに一括取込み、新しい UI-09c レシートの控え）。migration 0（後続の (A) が `ej_records` の 1 本を足す）。wire の型 0（後続で足す型は 37 §37.10・§37.11 と 45 §45.6c の proposed の型。既存の型は `DailyReportSdScan` に `ej` を足すだけで、enum `DailyReportDuplicateStatus` は変えない）。横断の約束がかかる経路は 3 つ（一括取込みの順と失敗〈日報・Z004・EJ の 3 つの取込みの関数〉、取込み済みの見分け〈SD の束・CV17 の写し・手で選んだ file〉、日報の同日追加の確認〈SD の経路と手で選ぶ経路〉）。**1 lane のまま進める**。横断の約束は 37 §37.10 の BIZ-08-D7（順と失敗）と BIZ-08-D10 の表（CV17 との併存）の 2 か所に置き、他の文書はそこを引く。EJ の閲覧を別の lane に分ける案は採らない: 取込みの鍵（記録の番号）と保存の形（文を持つ）は閲覧の要件から決まり、分けると片方の lane が他方の未決の形に依る。

## Workflow State

- Phase: archive
- Risk: R3
- Plan Commit: 8dd4ec52dc118a2f84213fa6b46cfd9821a5855d
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（本 run の起草役）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は round ごとに `.local` の model の選び方で決める。round 1 は GPT-6.1 Sol / high〈「R3 で複数の層にまたがる契約の初回」〉、round 2 以降は GPT-6 Astra / high〈「前の round で Codex が P1 を出した lane の再 review」〉）
- Final Reviewer: Fable 5.1（Claude 側、R3・design lane）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/sd-bulk-import-design

遷移の記録:

1. kickoff → spec-check（2026-10-17、起草役）: owner の決定（repo 外の回答台帳 TD-264・TD-265・TD-277・TD-296 の 11a・TD-304）と backlog の「売上の SD からの一括取込みと EJ の控えの閲覧」の要件を Scope にし、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-17、起草役）: 触る正本（37 §37.9 の「Z004 と EJ への引継ぎ」が設計を後続へ送っている、29E IO-08.10 の申し送り、D-071 の日報の側、UI-07）に一括取込み・EJ の保存と閲覧・CV17 との併存の契約が無い。同じ commit で設計正本に足した（下の Design Readiness）。
3. design → plan-draft（2026-10-17、起草役、同じ commit）: 条件「設計の出力が正本にある」は Design Readiness の引く正本と D-121 で満たす。条件「未解決の設計の問いが無い」は、店に聞く 2 つの問い（控えの探し方・昔の控え）に答えが無い間の既定を正本（UI-09c-D2・D3・D5）と backlog に書いたことで満たす。owner に聞く設計の問いは残っていない。plan-gate への遷移は Coordinator。
4. plan-draft → plan-gate（2026-10-17、Coordinator）: packet・Matrix と正本は起草役の commit（`8470fa65`）にある。Coordinator は D-121 の Context・Decision と、37 §37.10・§37.11 を新設して §37.3〜§37.9（並走の 367 の所有）を変えていないこと、所有表（37・45・55・pos-tables で 363・367 と節が分かれる）、owner の決定（TD-264・TD-265・TD-277・TD-296 の 11a・TD-304）の写しを確かめた。発注書の「TD-264 に ECR+ の終了の後も」は Coordinator の要約の誤りで、起草役が原文に当たって出所を TD-162・TD-020 に直したことを確かめた。店に聞く 2 つは答えが無い間の既定で進められる。Plan Review round 1 へ出す。
5. plan-gate（round 1 の是正、2026-10-18、Coordinator）: Plan Review round 1（Claude 側 fresh Opus 5.5・Codex GPT-6.1 Sol）の是正（Review Response 参照）。是正の前に相談役（Fable 5.1）に反例探しを頼んだ。起草役が `253ef96c` で直し、Coordinator が差分（37 §37.10 の内部の結果と手順 1〜5・BIZ-08-D8・D10、§37.11 の手順 1・2a・3・5・7 と `body_text`・保証の範囲、24 §14.18b、45 §45.6c、55 UI-07-D17、backlog の申し送り、Matrix の追加の行）を読んだ。Scope の範囲と方向は変えず、その場で直したので plan-gate に留める。Plan Review round 2 へ出す。
6. plan-gate（round 2 の是正、2026-10-18、Coordinator）: Plan Review round 2（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。起草役が `f4f5faf9` で直し、Coordinator が差分（37 §37.11 の保存の規則を形と日付の検査 → 伸びうるかの判定と保留 → 番号と文の比べ → 保存 → 数と名指しの 1 つの筋に書き直したこと、伸びうるかを file の名前の規則で決めること、§37.10 の scan で外した名前・移す条件・Z004 の error の振り分け、45 の生成 enum、24 の型の所有、55 の知らせの文）と、起草役の組の一巡の表を読んだ。Scope の範囲は変えず、その場で直したので plan-gate に留める。Plan Review round 3（天井）へ出す。
7. plan-gate（round 3 の是正、2026-10-18、Coordinator）: Plan Review round 3（Claude 側 fresh Opus 5.5・Codex GPT-6 Astra）の是正（Review Response 参照）。round 3 は round の天井で、owner が「直して、追加の round なしで plan-approved」と決めた（repo 外の回答台帳 TD-317、介入 1 回目 / 予算 9 回）。起草役が `23be816e` で直し、Coordinator が差分（UI-07-D17 の Z004 の部分成功と保留・食い違い・読めない file だけのときのボタンの条件、§37.11 手順 3 (d) の保留した記録の確かめ直しを正本 1 か所にしたこと、控えの query の key、並べる鍵、番号の一周）を読んだ。
8. plan-gate → plan-approved（2026-10-18、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）のうち Claude 側が round 3 で P1・P2 が無いと報告し、Codex の P2 2 件は owner の決定（repo 外の回答台帳 TD-317）どおり直した（Review Response 参照）。Plan Commit は round 3 の是正と記録の後の版。
9. plan-approved → implementing（2026-10-18、Coordinator、本 commit）: docs だけの design lane で、設計の出力は Plan Commit までに正本にある。状態だけを進め、Final Review（Minimum 1）へ出す。

## Owner Effort Budget

- 介入回数上限: 9（既定の 6 から owner が改定、repo 外の回答台帳 TD-296 の 5a）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9（owner の改定、TD-296） | 3（closeout 時点。Plan Review round 3 の天井の決定 1〈TD-317「直して進める」〉、Ready 1、merge 1。L3・manual は無し。plan-approved は TD-296 の 4a で Coordinator が進めた。TD-264・TD-265・TD-277・TD-304 は起票の前の決定で、本 lane の介入に数えない） | 0 | 6 | 9 = 3 + 0 + 6 |

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
DB の表（`ej_records`）、POS の取込み（日報・Z004・EJ の取込みの順と失敗、日報の同日追加の確認の置換え）、Tauri command の DTO（CMD-12 の 5 command と scan の応答の field）、operator workflow（日報取込みタブの一括取込み、新しい画面 UI-09c）の契約を決める設計（`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行: DB、POS CSV、Tauri command DTO、operator workflow）。本 lane は docs だけだが、後続の runtime が写す契約を決めるので R3。R4 ではない: データを消す・書き換える操作を設計しない（`ej_records` は insert-only、一括取込みは今の取込みの関数を呼ぶだけ、SD に書かない）。この変更で required gate の green / red は変わらない: docs だけで、REQ の参照を増減していない（REQ の採番は runtime の lane）。設計書の code block に足した関数の signature は実装に無いが、設計照合 test は「設計書にあって code に無い関数」を INFO にするだけ（`src-tauri/tests/design_compliance_test.rs` の冒頭の説明）で、function-design の file を新設していないので map の未登録の検査にも当たらない（AC10）。Scope の予定 path を `scripts/ci/classify-changes.sh` の workflow の一覧（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/*`・`docs/templates/*`・`.agents/*`・`.claude/{rules,commands,skills}/*`・PR template）に当てて、当たる file は無い。Final Review Minimum 1。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime の Writer が、チャットの履歴と repo 外の台帳を見ずに、source docs だけで次を実装できる: 締めの一括取込み（37 §37.10 BIZ-08-D7。日報 → Z004 → EJ の順、1 項目 1 TX、業務の拒否は飛ばし確定の段の失敗で止める）、日報の同日追加の確認の置換え（BIZ-08-D8）、EJ の記録の取込み・保存・欠けの知らせ（§37.11 BIZ-08-D9、24 §14.25、pos-tables の `ej_records`）、レシートの控えの画面（SCREEN_DESIGN の UI-09c-D1〜D5）、一括取込みの画面（55 UI-07-D17）、command（45 §45.6c CMD-12-D2）、CV17 との併存と `XZ_BKUP` へ移す lane への約束（BIZ-08-D10）。
- backlog の要件の 2 つの問いに正本が答える: 日報の同日追加の確認は、同じ日の取込みが全て精算回数を持つときは精算回数の照合で置き換える（BIZ-08-D8）。EJ の取込み済みの見分けと欠けの観測は IO-08.10 の記録（番号で見分ける、番号の連続で欠けを決める）を引き継ぐ（29E IO-08.10 の D-121 の注記）。
- 小数の数量（TD-304）の扱いの判断（拾わない、理由つき）が D-121 (6) と backlog にある。
- 店に聞く問い 2 つが、答えが無い間の既定とともに正本と backlog にある。
- D-121 が decision-log にあり、D-071 に Superseded in part の行がある。後続の runtime (A)・(B) の範囲・順・Risk が backlog にある。

### 失敗定義

- SD 直読みの runtime が実装する D-111 の契約（29 §29.7・§29.8、37 §37.3〜§37.9、45 §45.1〜§45.6b、55 UI-07-D12〜D16、pos-tables §12b）の文を書き換える。
- 一括取込みで、同じ精算が二重に数えられる経路（日報・Z004 の別 bytes、EJ の同じ記録の `XZ` と `XZ_BKUP`）、または 1 項目の拒否が以後の項目を毎回止める経路が残る。
- 日報の同日追加の確認を、精算回数の無い取込みと比べる場合まで外す。
- EJ を在庫・売上の集計・Z004 との照合に使う契約になる。
- 一括取込みが SD に書く、または `XZ_BKUP` へ移す lane の設計（改名・途中の失敗）を先取りする。
- 日報の紙と意味づけの lane が所有する節（34、56、pos-tables §12c〜12e、29 の IO-07 の項目の意味）を書き換える。
- 設計照合 test・traceability の検査を red にする。公開 repo に店の運用の細部・金額・品名・発言の原文を書く。

### 非目的

- code・migration・bindings・test の変更（runtime (A)・(B)）。
- `XZ_BKUP` へ移す操作（D-111 (1) の (b)、TD-296 の 11a で v1.0 の前。本 lane は妨げない約束だけを書く）。
- Z004 と EJ の小数の数量の配線（P、U5 の答えの後）、日報の紙と項目の意味（D-120 の lane）。
- 控えを金額・品名で探すこと、昔の控えの一括の取込み（店の答え待ち。手で選ぶ取込みで後から足せる）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、締めの日と控えを見る日の操作列を置く。本 lane は docs だけで、表の通常運用は後続の runtime (A)（Z004 の行は (B)）の後に成り立つ。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店が 1 回の操作で取り込み、控えを PC で見る）ことは別で、後者は本 lane では未達。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| (A) の後。締めの精算の後、店はいつもの CV17 の取込みを済ませた（`XZ_BKUP` へ移る。D-111 の運用の制約） | SD を PC に差し「SD から読む」 | 精算の一覧と「レシートの控え（EJ）: 新しい控え N 件」。読み終わりの案内（CV17 の取込みと矛盾しない文）は今どおり | 「まとめて取り込む」が押せる（日報の `NotImported`、新しい控え、または保留・食い違い・読めない EJ のどれかがある。UI-07-D17） | P5（SD-23）。日報は hash と BIZ-08-D4 で止まり、EJ は番号で見分けるので前提に依らない |
| 同じ | 「まとめて取り込む」 | 日報の束を古い精算から取り込み、続けて EJ の新しい控えを保存する。項目ごとに「取り込みました」（対象日・総売上・純売上）、EJ は「控え N 件を取り込みました」と、伸びうる file の最後の控え 1 件は今回入れず次に SD を読んだときにもう一度確かめる旨の補足（店の操作は要らない。37 §37.11 手順 3 (d)、出し方は UI-07-D17）。SD はレジに戻す | — | BIZ-08-D7 |
| 締めの後に売り、同じ日にもう一度精算した（店で月に数回、TD-279） | 同じ操作 | 同じ日の 2 つの束が確認なしに両方入る（精算回数が違う） | — | BIZ-08-D8、P6 |
| D-111 より前の取込み（精算回数が NULL）が同じ日にある | 同じ操作 | その束は「1 件ずつ確かめてください」で飛ばされ、ほかは続く。行の「確認する」から今の同日追加の確認で入れる | 利用者が確かめる | BIZ-08-D8 |
| 取込みの途中で PC の空き容量が無くなった | 同じ操作 | 写しを書けなかった項目で「ここで止まりました」、後は「取り込んでいません」。取り込めた分は残る | 空き容量を直し、もう一度「まとめて取り込む」（30 分以内は同じ読取り、過ぎたら読み直し） | BIZ-08-D7 |
| やり直し | 「まとめて取り込む」 | 取り込み済みの項目は「取り込みませんでした（取込み済みです）」、残りが入る。EJ は保存済みの番号を数えるだけ | — | BIZ-08-D7・D9 |
| アプリが先に読み、その後で店が CV17 の取込みをした（順の入れ替わり） | 翌日「SD から読む」 | `XZ_BKUP` の同じ束は「取込み済み」、EJ の同じ記録は数えない。二重に数えない | — | BIZ-08-D10 の表 |
| お客様からレシートの問合せ | 「レシートの控え」を開き、日付を選ぶか番号を入れる | その日の控えの一覧（時刻・番号・種類・合計）と、選んだ控えの文（等幅、紙のレシートと同じ並び） | — | P7（等幅の字形）、店に聞く問い (1) |
| 日報だけを 1 件ずつの経路で取り込み、一括取込み（EJ）を 30 日より長く使わなかった後（round 1 の是正） | 「SD から読む」→「まとめて取り込む」 | 窓は日報の最後の取込みの日までしか戻らず、それより前の `XZ_BKUP` の EJ は読まない。一括取込みの結果と控えの画面に「記録 n〜m の控えがありません」と、その場の「EJ のファイルを選んで取り込む」 | 「EJ のファイルを選んで取り込む」で `XZ_BKUP` か PC の CV17 の写しの file を選ぶと欠けが消える | UI-09c-D5 |
| (B) の後（棚卸しの runtime の (b) の後） | 「まとめて取り込む」 | 日報の後に Z004 を精算回数の昇順に取り込み、32 の新契約の知らせと要再確認の一覧を出す。拒まれた Z004 は飛ばして EJ まで進む | 要再確認の商品を棚で数え直す（73） | 32 の新契約 |

## Scope

本 lane は docs だけ。下の file の、下の節を足した・直した（全部 plan-first の commit）。

1. 一括取込みの流れ（Scope 1）: [37](../../function-design/37-biz-daily-report-import-service.md) §37.10 を足した（`commit_register_sd_bulk_import`、型、処理ステップ、BIZ-08-D7 の順・TX・失敗、§37.9 に後から足す snapshot と wire の field）。[32](../../function-design/32-biz-csv-import-service.md) の D-118 の節の後に「SD の一括取込み」（Z004 は 32 の新契約をそのまま呼び、拒否は飛ばす、SD の file だけ写す）。
2. 日報の同日追加の確認の置換え（backlog の要件）: 37 §37.10 BIZ-08-D8。直す所の一覧（§37.3 手順 8、pos-tables §12b、UI-07-D13、`parse.rs:159`、`tests.rs` の 3 か所）は runtime (A) の範囲として BIZ-08-D8 と backlog に書いた（本 lane は §37.3・§12b・UI-07-D13 の文を変えない。SD 直読みの runtime の所有）。decision-log の D-071 に Superseded in part の行。
3. EJ の取込みの記録と閲覧の画面（Scope 2）: 37 §37.11（取込み・閲覧の関数、型、BIZ-08-D9。在庫に使わない理由）、[24](../../function-design/24-io-csv-import-repo.md) §14.25（repo の関数と SQL の方針）、[pos-tables](../../db-design/pos-tables.md) の「SD の一括取込みと EJ の控えの契約」（`ej_records` のカラム定義）と [DB_DESIGN](../../DB_DESIGN.md) の案内の 1 行、[SCREEN_DESIGN](../../SCREEN_DESIGN.md) の画面一覧の行と「レシートの控え画面」（UI-09c-D1〜D5）、[ui-task-specs](../../architecture/ui-task-specs.md) の UI-09c、[29E](../../function-design/29-io-ej-parser.md) IO-08.10 の D-121 の注記（何を引き継ぎ何を引き継がないか）。
4. 一括取込みの画面と command: [55](../../function-design/55-ui-csv-import.md) UI-07-D17、[45](../../function-design/45-cmd-daily-report-import.md) §45.6c（CMD-12-D2 の 5 command と scan に後から足すもの）。
5. CV17 との併存（Scope 3）: 37 §37.10 BIZ-08-D10 の表と `XZ_BKUP` へ移す lane への約束。backlog のその lane の項目に申し送りの sub-bullet。
6. 後続の lane（Scope 4）: backlog に「SD の一括取込みと EJ の控えの runtime（A）」「Z004 を SD の一括取込みに入れる runtime（B）」と、本項目の起票の記録の sub-bullet。D-121 を decision-log の末尾に足した。
7. 店の事実で決まらない所（Scope 5）: backlog に「店に聞く: レシートの控えの探し方と昔の控え」（答えが無い間の既定つき）。下の Open Questions。

ついでに直したもの: 無し。

型・関数の形の変更（後続の lane が触る。本 lane は変えない）。`rg` で数えた作る所・読む所（2026-10-17、起点 `f5d7aaf4`）:
- `DailyReportSdScan` / `DailyReportSdScanSnapshot`（§37.9、未実装。SD 直読みの runtime が作る）: `rg -n 'DailyReportSdScan' src src-tauri/src` は 0 件（未実装）。(A) が `ej`・`ej_files`、(B) が `z004_files` を足す。bindings は (A) で再生成。
- `DailyReportDuplicateStatus`（変えない）の分岐: 作る所 `src-tauri/src/biz/daily_report_import_service/parse.rs:159`、読む所 `commit.rs:37`・`src-tauri/src/cmd/daily_report_import_cmd.rs:446`・`src/features/daily-report-import/DailyReportImportPage.tsx:139`、test `src-tauri/src/biz/daily_report_import_service/tests.rs:275`・`:481`・`:536`、`src/features/daily-report-import/DailyReportImportPage.test.tsx`（`:61`・`:156`・`:258`・`:277`・`:418`・`:455`）、`src/features/daily-report-import/hooks/useDailyReportImportFlow.test.tsx:437`（`rg -n 'AdditionalImportConfirmationRequired' src-tauri/src src`、Z004 の側の hit は除く）。BIZ-08-D8 で値の意味が変わるのは BIZ の分岐と backend の test の期待だけで、frontend は status を見て描くだけなので変わらない（fixture の status は文字列のまま）。
- 新しい型・関数（37 §37.10・§37.11、24 §14.18b・§14.25、45 §45.6c。round 1 の是正で、既存の `EjRecord`〈`ej_parser.rs`〉に当たらない式に限った。round 2 の是正で `EjConflict`・`EjRecordKindCode`・`EjRecordModeCode`・`ej_unreadable_files` を足した）: `rg -c 'RegisterSdBulk|RegisterSdEjSummary|EjImportResult|EjImportPlan|EjRecordDay|EjRecordSummary|EjRecordDetail|EjRecordRef|EjNumberGap|NewEjRecord|EjSourceFile|SdSourceFile|EjConflict|EjRecordKindCode|EjRecordModeCode|ej_unreadable_files|ej_records|commit_register_sd_bulk_import|import_ej_records|exists_same_date_daily_report_import_without_settlement_no|measure_register_sd_shape' src src-tauri/src` の出力は空（exit 1、新規。2026-10-18、HEAD `7d65f6b6` の上の作業 tree で測った）。

並走の lane と重なる file と節の所有（D-055 の所有表）:

| file | 本 lane が触る所 | SD 直読みの runtime（367）が触る見込みの所 | 日報の紙と意味づけ（363）が触る見込みの所 | merge の後に揃える所 |
|---|---|---|---|---|
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.10・§37.11（新設、§37.9 の後）と更新履歴の末尾の行 | §37.3・§37.4・§37.9（D-111 の実装との同期と P3 の是正。backlog の「起票時に直す」(1)(2)） | §37.3 手順 7 の部門の照合など項目の意味 | 更新履歴は両方を残す。(A) の起票のときに、§37.3 手順 8 を BIZ-08-D8 の形に、§37.9 手順 4 の「Z004 は PLU の占有の照合のためだけに手順 10 で読む」と「Z004 と EJ への引継ぎ」の末文（一度に取り込む形の設計は後続の lane が決める）を §37.10・§37.11 への参照に直す（367 の merge の後。round 1 の是正） |
| `docs/function-design/45-cmd-daily-report-import.md` | §45.6c（新設）と更新履歴の末尾 | §45.6a・§45.6b・§45.8 | 無し | 更新履歴は両方を残す |
| `docs/function-design/55-ui-csv-import.md` | UI 判断 ID の末尾の UI-07-D17 と更新履歴の末尾 | UI-07-D12・D15・D16 と §55.1 の module の表 | 無し | 無し |
| `docs/function-design/29-io-ej-parser.md` | IO-08.10 の冒頭の注記 | 無し | 無し | 無し |
| `docs/db-design/pos-tables.md` | §12e の後の「SD の一括取込みと EJ の控えの契約」（新設）と更新履歴の末尾 | §12b | §12c〜§12e | 更新履歴は両方を残す |
| `docs/DB_DESIGN.md` | 冒頭の案内の 1 行 | 見込み無し | 見込み無し | 無し |
| `docs/SCREEN_DESIGN.md` | 画面一覧の 12b の行と「レシートの控え画面」（新設、日次売上レポート画面の後） | 売上データ取込み画面の節（見込み） | 日次売上レポート画面の節（印刷） | 363 が日次売上の節を書き換えても、本 lane の節は別の見出し |
| `docs/architecture/ui-task-specs.md` | UI-09c（UI-09a/09b の後、UI-10 の前に新設） | UI-07 | UI-09a（印刷） | 無し |
| `docs/decision-log.md` | D-071 の末尾の 1 行と末尾の D-121 | D-111 への追記（見込み）・要るなら D-124 | D-022・D-025・D-096 への追記と D-120 | 末尾は番号の順で残す（D-120・D-121・D-124） |
| `docs/backlog.md` | 本項目の sub-bullet、直後の 3 項目、`XZ_BKUP` へ移す項目の sub-bullet | 「SD 直読みの runtime」の項目 | 「日報の紙と意味づけ」の項目 | 無し（項目が別） |
| `docs/function-design/24-io-csv-import-repo.md`・`32-biz-csv-import-service.md` | 24 §14.25（新設）と更新履歴の末尾、32 の「SD の一括取込み」（新設） | 24 は §14.14・§14.18a の同期の見込み | 34 と隣の 24 §14.21〜§14.22 の見込み | 更新履歴は両方を残す |

367 の契約に後から足すもの（正本の §37.9・§45.6a は変えていない。D-121 の Compatibility）: §37.9 の `DailyReportSdScan` に `ej`、`DailyReportSdScanSnapshot` に `ej_files`・`ej_unreadable_files`・`z004_files`、§37.9 の scan の手順 10 の後に EJ の読取り、§45.6a 手順 2a の「Z004 の bytes を cache に残さない」を (B) で「相対 path つきで残す」に変え、`DailyReportSdScan` に `z004_new_count` を足す。IO-09 に bytes を受ける純関数 `measure_register_sd_shape` を足し、29 §29.7.6 の手順 3 もそれを使う形に直す（(A)）。§37.9 の旧い 2 文を §37.10・§37.11 への参照に直す（(A)、上の表）。どれも 367 の merge の後の (A)・(B) が足す。

## Non-scope

- code・migration・bindings・test（runtime (A)・(B)）。REQ の採番（(A)）。
- D-111 の正本の節（上の失敗定義の列挙）、D-120 の lane の節（34、56、pos-tables §12c〜12e、IO-07 の項目の意味）。
- `XZ_BKUP` へ移す操作の設計、P（Z004 と EJ の小数の数量）、控えを金額・品名で探すこと。
- `docs/Plans.md`（lane の行を足さない。D-097）。

## Acceptance Criteria

数は開始 HEAD `f5d7aaf4` と起票後の版を同じ数え方（その file の中で式に当たる行の数、`rg -c`）で測った。開始はどれも 0（新しい語）。出力が無いのは 0。

- AC1（一括取込み）: `rg -c 'BIZ-08-D7' docs/function-design/37-biz-daily-report-import-service.md` が 3、`rg -c '^fn commit_register_sd_bulk_import' docs/function-design/37-biz-daily-report-import-service.md` が 1、`rg -c '^fn commit_register_sd_bulk_import' docs/function-design/45-cmd-daily-report-import.md` が 1。
- AC2（日報の確認の置換え）: `rg -c 'BIZ-08-D8' docs/function-design/37-biz-daily-report-import-service.md` が 3、`rg -c '^- Superseded in part by: D-121' docs/decision-log.md` が 1。
- AC3（EJ の保存と閲覧）: `rg -c 'BIZ-08-D9' docs/function-design/37-biz-daily-report-import-service.md` が 3、`rg -c '^### ej_records カラム定義' docs/db-design/pos-tables.md` が 1、`rg -c '^fn (insert_ej_record|list_ej_number_gaps)' docs/function-design/24-io-csv-import-repo.md` が 2、`rg -c 'UI-09c-D[1-5]' docs/SCREEN_DESIGN.md` が 5、`rg -c 'D-121' docs/function-design/29-io-ej-parser.md` が 1。
- AC4（併存）: `rg -c 'BIZ-08-D10' docs/function-design/37-biz-daily-report-import-service.md` が 2、`rg -c 'D-121 からの申し送り' docs/backlog.md` が 1。
- AC5（画面と command）: `rg -c 'UI-07-D17' docs/function-design/55-ui-csv-import.md` が 2、`rg -c 'CMD-12-D2' docs/function-design/45-cmd-daily-report-import.md` が 3。
- AC6（後続と問い）: `rg -c '^- \*\*SD の一括取込みと EJ の控えの runtime（A）\*\*' docs/backlog.md`・`rg -c '^- \*\*Z004 を SD の一括取込みに入れる runtime（B）\*\*' docs/backlog.md`・`rg -c '^- \*\*店に聞く: レシートの控えの探し方と昔の控え\*\*' docs/backlog.md` がどれも 1。
- AC7（決定の記録）: `rg -c '^## D-121: ' docs/decision-log.md` が 1。
- AC8（D-111 の節を触らない）: `git diff -U0 f5d7aaf4 HEAD -- docs/function-design/29-io-daily-report-parser.md` の出力が空。`git diff f5d7aaf4 HEAD -- docs/function-design/37-biz-daily-report-import-service.md docs/function-design/45-cmd-daily-report-import.md docs/function-design/55-ui-csv-import.md docs/db-design/pos-tables.md | grep -c '^-[^-]'` が 0（消した行・書き換えた行が無い。足しただけ）。
- AC9（D-120 の節を触らない）: `git diff --name-only f5d7aaf4 HEAD -- docs/function-design/34-biz-sales-service.md docs/function-design/56-ui-daily-sales.md` の出力が空。
- AC10（検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0（WARN は開始 HEAD の 0 から増えない）。`cd src-tauri && cargo run --bin generate_traceability -- --check` が OK。`cd src-tauri && cargo test --test design_compliance_test` が pass。
- AC11（docs だけ）: `git diff --name-only f5d7aaf4 HEAD -- src src-tauri scripts package.json package-lock.json` の出力が空。
- AC12（Plan Review round 1 の是正。開始 `39dd2f14` ではどれも 0）: `rg -c '^fn exists_same_date_daily_report_import_without_settlement_no' docs/function-design/24-io-csv-import-repo.md` が 1、`rg -c '最後の記録の保留' docs/function-design/37-biz-daily-report-import-service.md` が 1（round 1 の是正の後は 4。round 2 の是正で §37.11 を 1 つの筋に書き直し、この語は更新履歴の round 1 の行だけに残る。保留の規則は §37.11 手順 2）、`rg -c 'Failed\(BizError\)' docs/function-design/37-biz-daily-report-import-service.md` が 2、`rg -c 'measure_register_sd_shape' docs/function-design/37-biz-daily-report-import-service.md` が 2（同じく 3 から。形の検査を §37.11 手順 1 の 1 か所にした）、`rg -c 'test_ej_xz_last_record_is_held' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md` が 1。AC1〜AC11 の数は是正の後に測り直して変わらない。
- AC13（Plan Review round 2 の是正。開始 `7d65f6b6` では、除去の対象の最後の式 `RegisterSdBulkResult\.ej` が 1、ほかはどれも 0）: `rg -c -F '^EJ\d{6}_\d{4}\.TXT$' docs/function-design/37-biz-daily-report-import-service.md` が 1（裁定 1、伸びうるかの名前の判定）、`rg -c 'held_count. が 0' docs/function-design/37-biz-daily-report-import-service.md` が 1（裁定 2）、`rg -c 'ej_unreadable_files' docs/function-design/37-biz-daily-report-import-service.md` が 7（裁定 3）、`rg -c 'saved: bool' docs/function-design/37-biz-daily-report-import-service.md` が 1（裁定 4）、`rg -c 'NaiveDateTime::parse_from_str' docs/function-design/37-biz-daily-report-import-service.md` が 1（裁定 6）、`rg -c '^enum EjRecord(Kind|Mode)Code' docs/function-design/37-biz-daily-report-import-service.md` が 2（裁定 8）、`rg -c -F '（手順 2 と同じ形）' docs/function-design/37-biz-daily-report-import-service.md` が 1（裁定 10）、`rg -c 'Failed \{ kind: CmdErrorKind' docs/function-design/45-cmd-daily-report-import.md` が 1（裁定 8）、`rg -c '^型の所有: ' docs/function-design/24-io-csv-import-repo.md` が 1（裁定 11）、`rg -c 'test_ej_hand_picked_growable_file_holds_last_record|test_ej_plan_full_match_requires_no_held|test_ej_invalid_calendar_datetime_file_is_unreadable|test_bulk_import_names_scan_unreadable_ej_files|test_ej_wire_enums_serialize_as_fixed_values|残る有効な値引き' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md` が 11（裁定 1・2・3・6・7・8 の行と mutation の問い）、`rg -c 'RegisterSdBulkResult\.ej' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md` の出力が空（0、裁定 9）。裁定 5 は Matrix の `test_ej_grown_file_adds_only_new_records`・`test_bulk_import_after_cv17_moved_counts_nothing_twice` の期待の書換えで、数でなく本文で確かめる。
- AC14（Plan Review round 3 の是正。開始 `08918379` ではどれも 0）: `rg -c '一部を取り込みました' docs/function-design/55-ui-csv-import.md` が 1、`rg -c 'bulk result shows z004 partial with skipped rows and detail link' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md` が 1（裁定 1）、`rg -c 'held_count: i64, +// plan_ej_records の held_count' docs/function-design/37-biz-daily-report-import-service.md` が 1、`rg -c 'その記録はもう一度手順 1〜3 を通る' docs/function-design/37-biz-daily-report-import-service.md` が 1、`rg -c '次に SD を読んだときにもう一度確かめます' docs/function-design/55-ui-csv-import.md` が 1、`rg -c '次の読取りでもう一度確かめる' docs/decision-log.md` が 1（裁定 2）、`rg -c 'queryKeys\.ejRecords\.root' docs/function-design/45-cmd-daily-report-import.md` が 2、`rg -c 'queryKeys\.ejRecords\.root' docs/SCREEN_DESIGN.md` が 1（裁定 4）、`rg -c 'candidate_key. の辞書順' docs/function-design/37-biz-daily-report-import-service.md` が 1、`rg -c 'candidate_key. の辞書順' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md` が 1（裁定 5）、`rg -c '999999 から 000000' docs/function-design/37-biz-daily-report-import-service.md` が 1（裁定 6）。裁定 3 は上の AC13 の括弧の書換えで、本文で確かめる。AC1〜AC13 の数は是正の後に測り直して変わらない。

実測（round 3 の是正の後、commit の前の作業 tree、2026-10-18。AC の command を逐語で回し、AC8・AC9・AC11 だけは commit の前なので `f5d7aaf4 HEAD --` を `f5d7aaf4 --`〈作業 tree と比べる〉にした。commit の後に逐語の形で回し直した結果は Coordinator への報告に書く）:

```text
AC1  rg -c 'BIZ-08-D7' docs/function-design/37-biz-daily-report-import-service.md
  => 3
AC1  rg -c '^fn commit_register_sd_bulk_import' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC1  rg -c '^fn commit_register_sd_bulk_import' docs/function-design/45-cmd-daily-report-import.md
  => 1
AC2  rg -c 'BIZ-08-D8' docs/function-design/37-biz-daily-report-import-service.md
  => 3
AC2  rg -c '^- Superseded in part by: D-121' docs/decision-log.md
  => 1
AC3  rg -c 'BIZ-08-D9' docs/function-design/37-biz-daily-report-import-service.md
  => 3
AC3  rg -c '^### ej_records カラム定義' docs/db-design/pos-tables.md
  => 1
AC3  rg -c '^fn (insert_ej_record|list_ej_number_gaps)' docs/function-design/24-io-csv-import-repo.md
  => 2
AC3  rg -c 'UI-09c-D[1-5]' docs/SCREEN_DESIGN.md
  => 5
AC3  rg -c 'D-121' docs/function-design/29-io-ej-parser.md
  => 1
AC4  rg -c 'BIZ-08-D10' docs/function-design/37-biz-daily-report-import-service.md
  => 2
AC4  rg -c 'D-121 からの申し送り' docs/backlog.md
  => 1
AC5  rg -c 'UI-07-D17' docs/function-design/55-ui-csv-import.md
  => 2
AC5  rg -c 'CMD-12-D2' docs/function-design/45-cmd-daily-report-import.md
  => 3
AC6  rg -c '^- \*\*SD の一括取込みと EJ の控えの runtime（A）\*\*' docs/backlog.md
  => 1
AC6  rg -c '^- \*\*Z004 を SD の一括取込みに入れる runtime（B）\*\*' docs/backlog.md
  => 1
AC6  rg -c '^- \*\*店に聞く: レシートの控えの探し方と昔の控え\*\*' docs/backlog.md
  => 1
AC7  rg -c '^## D-121: ' docs/decision-log.md
  => 1
AC8  git diff -U0 f5d7aaf4 -- docs/function-design/29-io-daily-report-parser.md
  => (出力なし)
AC8  git diff f5d7aaf4 -- docs/function-design/37-biz-daily-report-import-service.md docs/function-design/45-cmd-daily-report-import.md docs/function-design/55-ui-csv-import.md docs/db-design/pos-tables.md | grep -c '^-[^-]'
  => 0
AC9  git diff --name-only f5d7aaf4 -- docs/function-design/34-biz-sales-service.md docs/function-design/56-ui-daily-sales.md
  => (出力なし)
AC11  git diff --name-only f5d7aaf4 -- src src-tauri scripts package.json package-lock.json
  => (出力なし)
AC12  rg -c '^fn exists_same_date_daily_report_import_without_settlement_no' docs/function-design/24-io-csv-import-repo.md
  => 1
AC12  rg -c '最後の記録の保留' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC12  rg -c 'Failed\(BizError\)' docs/function-design/37-biz-daily-report-import-service.md
  => 2
AC12  rg -c 'measure_register_sd_shape' docs/function-design/37-biz-daily-report-import-service.md
  => 2
AC12  rg -c 'test_ej_xz_last_record_is_held' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md
  => 1
AC13  rg -c -F '^EJ\d{6}_\d{4}\.TXT$' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC13  rg -c 'held_count. が 0' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC13  rg -c 'ej_unreadable_files' docs/function-design/37-biz-daily-report-import-service.md
  => 7
AC13  rg -c 'saved: bool' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC13  rg -c 'NaiveDateTime::parse_from_str' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC13  rg -c '^enum EjRecord(Kind|Mode)Code' docs/function-design/37-biz-daily-report-import-service.md
  => 2
AC13  rg -c -F '（手順 2 と同じ形）' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC13  rg -c 'Failed \{ kind: CmdErrorKind' docs/function-design/45-cmd-daily-report-import.md
  => 1
AC13  rg -c '^型の所有: ' docs/function-design/24-io-csv-import-repo.md
  => 1
AC13  rg -c 'test_ej_hand_picked_growable_file_holds_last_record|test_ej_plan_full_match_requires_no_held|test_ej_invalid_calendar_datetime_file_is_unreadable|test_bulk_import_names_scan_unreadable_ej_files|test_ej_wire_enums_serialize_as_fixed_values|残る有効な値引き' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md
  => 11
AC13  rg -c 'RegisterSdBulkResult\.ej' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md
  => (出力なし)
AC14  rg -c '一部を取り込みました' docs/function-design/55-ui-csv-import.md
  => 1
AC14  rg -c 'bulk result shows z004 partial with skipped rows and detail link' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md
  => 1
AC14  rg -c 'held_count: i64, +// plan_ej_records の held_count' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC14  rg -c 'その記録はもう一度手順 1〜3 を通る' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC14  rg -c '次に SD を読んだときにもう一度確かめます' docs/function-design/55-ui-csv-import.md
  => 1
AC14  rg -c '次の読取りでもう一度確かめる' docs/decision-log.md
  => 1
AC14  rg -c 'queryKeys\.ejRecords\.root' docs/function-design/45-cmd-daily-report-import.md
  => 2
AC14  rg -c 'queryKeys\.ejRecords\.root' docs/SCREEN_DESIGN.md
  => 1
AC14  rg -c 'candidate_key. の辞書順' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC14  rg -c 'candidate_key. の辞書順' docs/plans/test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md
  => 1
AC14  rg -c '999999 から 000000' docs/function-design/37-biz-daily-report-import-service.md
  => 1
AC10  bash scripts/doc-consistency-check.sh / --target plan
  => どちらも exit=0、ERROR 0・WARN 0（「全チェック通過」）
AC10  cargo run --bin generate_traceability -- --check
  => traceability check: OK（ERROR 0 件 / WARN 0 件）
AC10  cargo test --test design_compliance_test
  => test result: ok. 1 passed; 0 failed
```

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: 37 §37.3 手順 8（BIZ-08-D4 と同日追加）・§37.4・§37.9（BIZ-08-D3〜D6、「Z004 と EJ への引継ぎ」）・§37.10（BIZ-08-D7・D8・D10、新設）・§37.11（BIZ-08-D9、新設）／29 §29.7（IO-09-D2〜D4、SD-03・SD-09・SD-22・SD-23）・§29.8（IO-10）／29E IO-08.3・IO-08.6（照合の金額）・IO-08.9・IO-08.10／32 の D-118 の新契約（表の 1〜5、確定）と「SD の一括取込み」（新設）／24 §14.18a・§14.25（新設）／45 §45.6a・§45.6b・§45.6c（新設）・§45.7／55 UI-07-D13・D15・D16・D17（新設）／pos-tables §12b と `ej_records`（新設）／SCREEN_DESIGN の UI-09c（新設）／ADR SPEC-STK-CNT-D4・D11・D12／D-071・D-104・D-111・D-117・D-118・D-121。
- 必要な設計成果物（Design artifact selection の当たる行）: BIZ・IO・CMD の振舞い = updated in this PR（37、24、32、45）／command・DTO = updated in this PR（45 §45.6c、37 の型。wire の実装は後続、下の Boundary / Wire Contract）／表・TX・冪等性 = updated in this PR（pos-tables の `ej_records`、BIZ-08-D7・D9。migration の番号と実装は (A)）／operator workflow・文言 = updated in this PR（55 UI-07-D17、SCREEN_DESIGN の UI-09c。UI の function-design の文書は (A) が作る〈新設は `design_compliance_test.rs` の登録が要り、docs だけの lane では行えない〉）／import の形・互換 = updated in this PR（BIZ-08-D10）／durable な判断 = updated in this PR（D-121、D-071 の注記）。
- plan にしかない durable な判断の昇格先: 決めたことは全て正本と D-121 に置いた。packet にだけあるのは、規模と手法の数え、所有表、型の作る所・読む所の数え、AC の数。
- 前提・制約と、延期した design gap の follow-up: SD-23（未確認の前提）は日報・Z004 では D-111 の守り（精算回数）、EJ では記録の番号で受ける。等幅の字形（P7）は (A) の L3。控えの探し方と昔の控えは店の答え待ちで、答えが無い間の既定を UI-09c に書いた（backlog）。延期が安全な理由: どれも今の振舞いを変えず、既定は読むだけの画面の探し方の範囲。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「一括取込みで同じ精算を二重に数えない」は、日報は hash と BIZ-08-D4（精算回数のある束どうし。SD の束は全て持つ）、Z004 は hash と 32 の表の 4、EJ は記録の番号の UNIQUE と TX の中の比べで成り立つ。例外は、手で選んだ精算回数の無い日報の束（layout B）で、BIZ-08-D8 が確認を残す（D-111 の Guarantee range のまま）。「1 項目の拒否が以後を止めない」は、業務の拒否を Skipped にする規則で成り立ち、止まるのは確定の段の失敗だけ。escape hatch は CMD を経ない呼出しで、規則は BIZ にあるので同じ。「一括取込みは SD に書かない」は、SD を読むのは §37.9 の scan（IO-09-D3）だけで、本 lane の関数は snapshot の bytes だけを使うので成り立つ（写しの書く先はアプリの data の folder）。「EJ を在庫に使わない」は、`ej_records` がどの表とも結ばれず、在庫の求め直し（31 の `recompute_stock`）の入力に無いので成り立つ。
- 判定（ready / not ready）と理由: ready（plan-draft に進める）。理由: 設計の出力が上の正本と D-121 にあり、owner の判断事項は起票の前の決定で足り、店に聞く問いは答えが無い間の既定を持ち、残る外部前提（P5・P7）は runtime の L3 の前と L3 で確かめるもので設計の問いではない。

## Registration / Generation Obligations

本 lane: function-design の file の新設・改名・削除は無い。REQ の参照を増減していないので `docs/function-design/90-traceability.md` を再生成しない（AC10 の `--check` で確かめる）。後続の義務（(A)）: `ej_records` の migration を `migration.rs` の registry に番号を付けて登録（その時点の他の lane の migration と番号を突き合わせる。原価の lane の vB の起点の検査の表に入れない）、CMD-12-D2 の 5 command を `collect_commands!` と `generate_handler!` の両方に登録し `cargo run --bin generate_bindings`、控えの query の key `queryKeys.ejRecords.root()` と D-052 の invalidation の entry（45 §45.6c 手順 4）、`design_compliance_test.rs` の 37・24 の行に新しい module、UI-09c の function-design の文書の新設と同 map への登録、`npm run generate:routes`（`/reports/receipts`）、`src/config/navigation.ts` の `ui-09c` の entry と `navigation.test.ts` の到達の test（REQ の番号つき）、REQ の採番と `docs/spec/requirements.md`・`requirements-coverage.md`・traceability の再生成。(B): bindings の再生成（`RegisterSdBulkSource::Z004` と `ImportedZ004` を wire に出す）。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たる。EJ の文法と記録の番号の形は CASIO の adapter（IO-08）。`ej_records` の鍵（4 桁の欄 + 6 桁の番号）は adapter の形を写すので、レジが替われば見直す（BIZ-08-D9 の再検討）。一括取込みの順と失敗の規則は app-core | 37 §37.10・§37.11 |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。観測: EJ の番号の連続と戻らないこと、`XZ` 直下の EJ が伸びること、日報の精算回数と精算票の 1 対 1（下の Contract Probe、版を併記）。未確認の前提: SD-23、等幅の字形。app の決定: 1 項目 1 TX・飛ばす規則・文の保存・確認の置換え | Contract Probe、D-121 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。BIZ-08-D7 の停止とやり直し、TX の前の写しの残り、EJ の insert-only と TX の中の再比べ、30 分の snapshot の期限 | 37 §37.10、Matrix |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。締めは CV17 の取込み（(b) まで）→ SD から読む → まとめて取り込む → SD を戻す。控えは日付・番号で探す。欠けは手で選ぶ EJ で埋める | Ordinary Operation、UI-07-D17、UI-09c |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たる（小）。レジの機種が変わると IO-08・IO-09 と `ej_records` の鍵を見直す。一括取込みの順・失敗・確認の規則は残る | D-121 の Revisit |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。`ej_records` は店の品名・金額を持つ表で、DB の中だけに置く。tracked の test は合成の EJ だけ。本 lane の差分に実データ・件数の新しい集計を入れない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。EJ は在庫・売上の集計に使わず、控えの合計は一覧の手がかりだけ（復元できた取引だけ、返品は負のまま）。日報と Z004 の集計の意味は変えない | BIZ-08-D9 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる（後続）。控えの等幅の並び、一括取込みの結果の読みやすさ、実際の SD での読取り。本 lane は docs だけで L3 は無い | Matrix の Manual |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たる（小）。等幅の字形は Windows の font に依る。(A) の L3 で字形を決め、無いときの代わりの字形の並びを CSS に置く（defer） | backlog (A) |

## Boundary / Wire Contract

本 lane は wire を変えない（bindings も変えない）。後続の lane が変える wire の契約を正本に書いた。

- producer: CMD-12 `commit_register_sd_bulk_import`・`import_ej_files`・`list_ej_records`・`find_ej_records_by_no`・`get_ej_record`（45 §45.6c）、`scan_register_sd` の応答の `DailyReportSdScan.ej`（37 §37.10）
- consumer: UI-07 の日報取込みタブ（UI-07-D17）、UI-09c
- wire type: `RegisterSdBulkResponse`・`RegisterSdBulkItemDto`・`RegisterSdBulkSource`（`DailyReport`・`Z004`・`Ej`）・`RegisterSdBulkOutcomeDto`（`Failed { kind: CmdErrorKind, message }` は CMD が BIZ の `Failed(BizError)` に 45 §45.7 を当てて作る。round 1 の是正、kind の enum は round 2 の是正）、`DailyReportSdScan` の `z004_new_count`、`RegisterSdEjSummary`、`EjImportResult`（`conflicts: Vec<EjConflict>`）・`EjConflict`（`saved`）・`EjNumberGap`、`EjRecordDay`・`EjRecordSummary`（`kind: EjRecordKindCode`・`mode: EjRecordModeCode`。response だけの生成 enum、D-061・D-064）・`EjRecordRef`・`EjRecordDetail`、request `EjFileRequest { filename, file_bytes }`
- internal type: BIZ の `RegisterSdBulkResult`・`RegisterSdBulkOutcome`（`Failed(BizError)`）、`DailyReportSdScanSnapshot` の `ej_files`・`ej_unreadable_files`・`z004_files`、`SdSourceFile`、`EjSourceFile`、`EjImportPlan`、`NewEjRecord`
- precision/range: 記録の番号は 6 桁の文字列（先頭 0 を保つ。数値にしない）、4 桁の欄も文字列。`total_amount` は円の整数で絶対値 2^53 − 1 以下（外れたら NULL）。件数・ID は i64（JavaScript の number で扱える範囲。件数は記録の数以下）。日付は `YYYY-MM-DD`、日時は `YYYY-MM-DD HH:MM` の文字列
- round-trip path: SD → IO-09 → BIZ-08 scan（snapshot の bytes）→ CMD の scan cache → `commit_register_sd_bulk_import(scan_token)` → BIZ の結果 → UI。控えは `list_ej_records(date)` → 一覧の `id` → `get_ej_record(id)`。番号は UI の入力の文字列 → `find_ej_records_by_no` → `{ id, date }` → URL の `date`・`no`
- invalid input: scan_token の形の誤り・期限切れ（§45.6b と同じ）、日付の形の誤り（`ValidationFailed`）、番号が 1〜6 桁の数字でない（`ValidationFailed`）、無い id（`NotFound`）、EJ の file が 20MB 超（`validation`）、読めない EJ の file（形・parse・暦の外れた見出しの日時。`unreadable_files` に名前、取込みは続ける。scan で bytes を得られなかった file も一括の結果の `unreadable_files` に入る）
- compatibility: 既存の command と DTO は変えない（`DailyReportSdScan` は SD 直読みの runtime が作る型で、(A) が field を足す。古い UI は無い〈1 つの build〉）。`DailyReportDuplicateStatus` の値は変えず、`NoDuplicate` になる場合が増える（BIZ-08-D8）

## Test Plan

Test Design Matrix: [2026-10-17-sd-bulk-import-ej-view-design](test-matrices/2026-10-17-sd-bulk-import-ej-view-design.md)（後続の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh` と `--target plan`、`cargo run --bin generate_traceability -- --check`、`cargo test --test design_compliance_test`（AC10）と AC1〜AC9・AC11 の `rg`・`git diff`。
- negative tests: Matrix の Negative Paths（後続）。
- compatibility checks: Matrix の Compatibility Checks（CV17 の取込みの前後・順の入れ替わり、精算回数の NULL の取込みがある日）。
- data safety checks: 本 lane の差分に実データ（品名・金額・番号・発言の原文）を入れない。
- main wiring/integration checks: 後続の lane（scan から一括取込みの結果まで、EJ の file から控えの画面まで）。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（1 回の操作で締めの日報・Z004・EJ が入り、控えを PC で見られる）を達成できるか。拒まれ続ける項目があっても以後が止まらないか。
- BIZ-08-D7 の「飛ばす / 止める」の分け方が、今の各 BIZ の `Err` の種類（`ImportError`・`IdempotencyConflict`・`ValidationFailed`・`DatabaseError`・`SourceCopyFailed`）を漏れなく振り分けるか。止まった後のやり直しで二重に数えないか。
- BIZ-08-D8 が、精算回数の無い取込みのある日に確認を外さないか。同じ日の束を続けて取り込むとき、§37.4 手順 5 の TX の中の再検査と食い違わないか（一括取込みは束ごとに §37.3 を呼び直す）。
- EJ の同じ番号の扱い（同じ呼出しの中・保存済みとの文の違い）、欠けの区間の求め方、`total_amount` の作り方が IO-08 の出力の形（`Restored` の `items`・`adjustments`、照合の金額）と合うか。
- D-111 の正本の節を書き換えていないか（AC8）。§37.9・§45.6a に後から足すものが、SD 直読みの runtime の Scope を揺らさない形か。D-120 の lane の節を触っていないか（AC9）。
- 在庫に使わない理由と、CV17 との併存の表が、D-118・D-111 の運用の制約と矛盾しないか。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| BIZ-08-D7（一括取込みの順・TX・飛ばす / 止める・やり直し） | 37 §37.10 | (A): `commit_register_sd_bulk_import`（日報・EJ）、(B): Z004 の段 | (A)・(B)（Matrix の D7 の行） | L3: 結果の文 |
| BIZ-08-D8（同日追加の確認の置換え） | 37 §37.10、D-071 の注記 | (A): `parse.rs` の分岐、§37.3 手順 8・pos-tables §12b・UI-07-D13 の文 | (A)（Matrix の D8 の行、既存 `tests.rs` の 3 か所の期待の書換え） | 非対象 |
| BIZ-08-D9（EJ の保存・比べ・欠け・閲覧） | 37 §37.11、24 §14.25、pos-tables の `ej_records` | (A) | (A)（Matrix の D9 の行） | L3: 控えの画面 |
| BIZ-08-D10（CV17 との併存、(b) への約束） | 37 §37.10、backlog の (b) の項目 | (A)・(B) は守りを今の関数に任せる。(b) の lane が約束を使う | (A)（Matrix の併存の行） | 非対象 |
| §37.9 に足すもの（`ej`・`z004_new_count`・`ej_files`・`z004_files`・`measure_register_sd_shape`） | 37 §37.10、45 §45.6c | (A)・(B) | (A)・(B) | 非対象 |
| CMD-12-D2（5 command、登録） | 45 §45.6c | (A) | (A)（command registry drift、cmd の test） | 非対象 |
| UI-07-D17（一括取込みの画面） | 55 | (A)、(B) は Z004 の行 | (A)（RTL） | L3: 押せる・押せない、結果、保留の文が雑音にならないか |
| UI-09c-D1〜D5（レシートの控え画面） | SCREEN_DESIGN、ui-task-specs | (A) | (A)（RTL、到達の test） | L3: 等幅の並び、探し方、欠けの Alert |
| 32 の「SD の一括取込み」（Z004 を新契約のまま呼ぶ、写し） | 32 | (B) | (B) | 非対象 |
| IO-08.10 の引継ぎの注記 | 29E | 本 lane の記録 | AC3 | 非対象 |
| D-121、D-071 の Superseded in part | decision-log | 本 lane の記録 | AC2・AC7 | 非対象 |
| 隣接の除外: BIZ-08-D3〜D6・IO-09・IO-10・CMD-12-D1・UI-07-D12・D15・D16（D-111） | 37 §37.3〜§37.9、29 §29.7・§29.8、45、55 | 変えない（SD 直読みの runtime） | AC8 | 非対象 |
| 隣接の除外: 32 の D-118 の新契約の表と確定・取消、ADR SPEC-STK-CNT-D11・D12、D-117 の PLU の照合 | 32、ADR、33 | 変えない（参照だけ） | 非対象 | 非対象 |
| 隣接の除外: 日報の項目の意味（D-022・D-025・D-096）、34・56・pos-tables §12c〜12e | D-120 の lane | 変えない | AC9 | 非対象 |

## Contract Probe

本 run は実機・持ち帰りデータの新しい集計を行っていない。前提ごとに、出典（tracked の記録、code）と、出典の資料の時点（repo 外の `.local/field-data/versions.md` の版の表。手元の EJ・Z は 2026-10-14 までの版）を書く。

- P1（`XZ` 直下の EJ は精算をまたいで追記されて伸び、同じ記録が `XZ` と `XZ_BKUP` に入りうる）: 29 §29.7.2 の SD-03・SD-09（2026-10-06 の SD の採取の版と持ち帰りの EJ の集計）、IO-08.10。→ 観測（tracked の記録）。今の版での確かめ直しはしていない。
- P2（EJ の記録の番号は記録ごとに +1 で、file と精算をまたいで戻らない。4 桁の欄は 1 種類）: IO-08 の構造所見（2026-08 の実物 6 本の版）と backlog の実測と POS 系列の対応の入力 3（2026-10-06 の SD の一覧と、その時点の手元の EJ の版での集計: 6 桁の番号は +1 で連続、重なり 0、4 桁の欄は 1 種類）。→ 観測（その版での）。番号が戻る設定は取説では選べる（初期は戻らない）ので、BIZ-08-D9 の再検討に置いた。
- P3（`Restored` の取引の合計 = 有効な明細の金額の和 + 有効な値引きの金額の和）: 29E IO-08.6 の照合の条件 4 と、現物の `src-tauri/src/io/ej_parser.rs:717`（照合の金額と `amount_sum` が一致しなければ `Restored` にしない）・`:729`〜`:733`（`amount_sum` は items と adjustments の和）・`:832`〜`:834`（正の訂正で取り消した値引きを `adjustments` から外す）。→ 観測（code）。
- P4（EJ の file は CP932 strict・BOM 無し・CRLF だけ・最終改行あり。round 1 の是正で、最終改行が無いときは file でなく最後の記録だけを保留する〈37 §37.11 手順 2〉）: 29 §29.7.2 の SD-24（`XZ_BKUP` の全件、2026-10-06 の採取の版）と IO-08 の全期間の構造所見（2026-09-29・09-30 の持ち帰りの版）。`XZ` 直下の原本の形は SD-23 と同じく未確認の前提。→ 形を外れた file は読まずに知らせる（BIZ-08-D9）ので、前提が外れても誤った記録は入らない。
- P5（SD-23: 取込み前の `XZ` の原本と取込み後の file は同じ bytes）: 未確認の前提（D-111、SD 直読みの runtime の L3 の前に確かめる）。本 lane の契約は、日報・Z004 では D-111 の精算回数の照合で、EJ では記録の番号で受けるので、前提が外れても二重に数えない（BIZ-08-D10 の保証の範囲）。EJ では記録の文が変われば conflict として知らせる。
- P6（日報の精算回数は Z001 / Z002 / Z005 で一緒に進み、日計明細の精算票と 1 対 1 で、使い始めの後は重複しない）: D-104 の Guarantee range（手元の全期間での確認）と backlog の実測と POS 系列の対応の入力 1（2026-09-29 の持ち帰りの版）・入力 3（2026-10-06 の SD の一覧の版）・入力 5（2026-10-15 の集計、その時点の手元の版）。→ 観測（その版での）。BIZ-08-D8 はこれに依り、外れた（重複が出た）ときは BIZ-08-D4 が拒む側に倒れる（安全側）。
- P8（レジは `XZ` 直下の EJ の書きかけの記録を、次に続きから書くか、記録ごと書き直すか）: repo に事実が無い。→ 未確認の前提。37 §37.11 手順 2 は、どちらでも伸びうる file（名前に連番 `_nnnn` の無い EJ。SD から読んだか手で選んだかに依らない。round 2 の是正）の最後の記録を保存しないので、書きかけを保存して完全な記録が conflict になる経路を持たない（round 1 の是正）。
- P10（連番 `_nnnn` の名前は CV17 が `XZ_BKUP` へ移すときだけ付き、その後レジはその file に書かない。`EcrDatas` の写しは同じ名前）: 29 §29.7.2 の SD-03・SD-11・SD-22（2026-10-06 の SD の採取の版）。→ 観測（その版での）。37 §37.11 手順 2 の伸びうるかの判定はこれに依る。名前の規則に当たらない file は伸びうるに倒す。前提が外れて連番つきの file にもレジが書く場合と、利用者が伸びうる file に連番つきの名前を付けた場合は、書きかけを保存しうる（BIZ-08-D9 の保証の範囲）。
- P9（`XZ` 直下の file の最後の記録は普段は締めの精算票）: IO-08 の全期間の構造所見（2026-09-29・09-30 の持ち帰りの版: file の最後の記録はどれも終わりの印字のある精算票）。→ 観測（その版での）。保留で遅れるのは普段 1 件。
- P7（24 バイトの CP932 の行を、WebView で半角 1・全角 2 の幅の等幅で並べられる）: Windows の和文の等幅の字形の振舞いで、本 run は確かめていない。→ 未確認。(A) の L3 で字形と大きさを決める（UI-09c-D4）。

## Data Safety

- tracked に書かない: EJ・Z の中身（品名・金額・記録の番号・日時）、件数の新しい集計、店主・owner の発言の原文、repo 外の台帳の本文。
- local-only: repo 外の回答台帳・持ち帰りデータ（`.local/field-data/` の版の表を含む）・SD の写し。
- synthetic-only: 後続の lane の test と L3 の入力物は合成の EJ・日報・Z004（実 encoding の CP932・CRLF）だけ（owner の決定 TD-296）。`ej_records` の行と IO-10 の写しの file を commit しない。

## Open Questions

店に聞く（owner 経由。backlog の「店に聞く: レシートの控えの探し方と昔の控え」）:

1. 「レシートの控えを探すとき、何を手がかりにしますか（a 日付と時刻／b 合計の金額／c レシートの番号／d 品名）」。推奨: 答えを待たずに a・c で作る。答えが無いときの既定: 日付と番号で探し、一覧に時刻と合計を出す（UI-09c-D2・D3）。b・d なら探し方を足す（読むだけの画面の中で閉じる変更）。
2. 「アプリを使い始める前の控え（PC に CV17 が残した分）も見たいですか（y／n）」。推奨・既定: n（取り込まない）。y なら、手で選ぶ EJ の取込み（UI-09c-D5）で owner か開発者が一度入れる（新しい機能は要らない）。

owner に聞く設計の問い: 無し。

## Implementation Results

- 結果: 締めの SD の読取りで日報・Z004・EJ を一度に取り込む一括取込み（37 §37.10 BIZ-08-D7 の順・TX・失敗）、日報の同日追加の確認を同じ日の取込みが全て精算回数を持つときに精算回数の照合で置き換えること（BIZ-08-D8）、EJ の記録の番号を鍵に記録の文を保存してレシートの控えとして見せること（§37.11 BIZ-08-D9、pos-tables の `ej_records`、SCREEN_DESIGN の UI-09c）、CV17 との併存と `XZ_BKUP` へ移す lane への約束（BIZ-08-D10）を設計正本に置き、D-121 に記録した。D-071 は Superseded in part。Z004 と EJ の小数の数量は拾わない（D-121 (6)）。D-111 の正本の節（37 §37.3〜§37.9 ほか）は変えず、後から足すものを 37 §37.10 の「§37.9 に足すもの」と D-121 の Compatibility に書いた。本 lane は docs だけで、code・migration・bindings は変えていない（runtime は後続の (A)・(B)）。
- 既存の保護: 実装は変えていないので、店の動作は今のまま（日報は 1 件ずつ取り込み、EJ は取り込まない）。設計照合 test と traceability の検査は green のまま。
- 未検証: Contract Probe の P5（SD-23）・P7（等幅の字形）・P8（書きかけの記録の書き方）は (A) の L3 とその前の確かめ。店に聞く 2 つの問い（控えの探し方・昔の控え）は既定のまま。
- review・CI・merge（closeout、2026-10-18）: Plan Review は round 3（天井）までで、round 3 の残りは owner の決定（TD-317）で直して plan-approved。Final Review は broad 1 回と是正の後の closure 2 回（下の Review Response の Closeout の段落）。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由の squash merge。PR: [#175](https://github.com/higekiri-w/inventory-system-desktop/pull/175)
- 後続: `docs/backlog.md` の「SD の一括取込みと EJ の控えの runtime（A）」「Z004 を SD の一括取込みに入れる runtime（B）」「店に聞く: レシートの控えの探し方と昔の控え」と、「取込み済みの Z・EJ を `XZ_BKUP` へ移す操作（(b) の lane）」の D-121 からの申し送り。

## Review Response

### Plan Review round 1（2026-10-17、head `39dd2f14`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 5）と Codex GPT-6.1 Sol（reject、P1 1・P2 4・P3 1）。Claude #1 と Codex #3 は同じ件。全て採った（Coordinator の裁定）。是正の前に相談役（Fable 5.1）に反例探しを頼み、保留は `XZ` 直下の最後の記録だけにする方針にした。起草役が `253ef96c` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R1 | Codex #1（P1） | 採る | 書きかけの EJ の記録を保存すると、後の完全な記録が永久に conflict になる | `XZ` 直下の file の最後の記録は診断に依らず常に保存を保留し（`held_count`）、移された後の `XZ_BKUP` の file で保存する。最終改行の無い file は最後の記録だけを外す。同じ file に同じ番号が 2 回出たら conflict（37 §37.11 手順 2a・3）。Matrix に途中版 → 完全版ほかの行を足した |
| R2 | Claude #1（P2）・Codex #3（P2） | 採る | 一括取込みの結果に失敗の種類が無く、EJ の失敗の行き先も無い | 内部の結果を `Failed(BizError)` と EJ の項目（`RegisterSdBulkSource::Ej`）にし、CMD が 45 §45.7 を当てて wire の文にする |
| R3 | Codex #2（P2） | 採る | 同じ日の全取込みが精算回数を持つかを判定する IO が無い | 24 §14.18b `exists_same_date_daily_report_import_without_settlement_no` を足し、BIZ-08-D8 から呼ぶ |
| R4 | Codex #4（P2） | 採る | Z004 だけが取り込める状態を UI が判定できない | (B) で scan の応答に `z004_new_count` を足し、ボタンの有効化に使う |
| R5 | Codex #5（P2） | 採る | 後続の移動の lane へ渡す EJ の条件が番号の存在だけ | 番号と文が保存済みと一致すること（`plan_ej_records` で新しい記録・conflict・読めない file が無い）にし、backlog の申し送りも揃えた |
| R6 | Claude #2（P3） | 採る | 日報の束を並べる鍵の出所が無い | IO-07 `parse_daily_report_bundle`（純関数）で取り、§37.3 を並べるために先に呼ばない理由を書いた |
| R7 | Claude #3（P3） | 採る | 長く使わなかった後の欠けの場面が窓の規則と合わない | 欠けが出る場面を「日報だけを個別の経路（行の「確認する」か手で選ぶ経路）で取り込み、一括取込みを窓の日数より長く使わなかった後」に直し、その場の「EJ のファイルを選んで取り込む」を書いた |
| R8 | Claude #4（P3） | 採る | §37.9 に旧い前提の live な文が残る（並走の 367 の所有の節） | 所有表の「merge の後に揃える所」と backlog に、(A) の起票で直す文として足した |
| R9 | Claude #5（P3） | 採る | `body_text` の見出しの取り方が 2 通りある。手で選んだ file の形の検査の関数が無い | 見出しは file の bytes を CRLF で分けた行から取ると決め、IO-09 に純関数 `measure_register_sd_shape` を足して 2 経路で共用する |
| R10 | Claude #6（P3） | 採る（後続の lane へ） | conflict を含む `XZ` 直下の EJ は移せず残り続け、写しが毎日増える | 扱い（移さずに知らせる等）は移動の lane（(b)）が決めると backlog の申し送りに書いた。写しの保持は D-111 の A の項目で扱う |
| R11 | Codex #6（P3） | 採る | 新しい型の baseline の検索式が既存の `EjRecord` に当たる | 新設の名前に限った式に直し、実測を更新した |

あわせて owner の設計の原則（repo 外の回答台帳 TD-310）で、EJ の欠け・保留・conflict をその場で名指しし、欠けは「EJ のファイルを選んで取り込む」で直せる形にした。conflict にはその場で直す操作が無く、「開発者に知らせて」の文になっている。これは owner に諮る候補で、本 round の review が通った後に他の packet の問いとまとめて聞く。

### Plan Review round 2（2026-10-18、head `7d65f6b6`）

Claude 側 fresh Opus 5.5（reject、P1 0・P2 1・P3 2）と Codex GPT-6 Astra（reject、P1 1・P2 7・P3 1）。Claude #1 と Codex #1 は同じ件で、重い方の P1 とした。全て採った（Coordinator の裁定）。round 1・2 で EJ の保存の規則に指摘が続いたので、個別に塞がず §37.11 を 1 つの筋として書き直させた（owner の決定 TD-296 の 5a）。起草役が `f4f5faf9` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R12 | Codex #1（P1）・Claude #1（P2） | 採る | 手で選んだ伸び続ける EJ の file では最後の記録の保留が効かず、書きかけの記録が保存されて完全な記録が永久に conflict になる | 伸びうるかを出所によらず file の名前の規則（連番の付いた名前だけが伸びない）で決め、手で選んだ file にも保留を当てた |
| R13 | Codex #2（P2） | 採る | 移してよい EJ の条件が、保留した記録を外すので全件の一致を証明しない | 保留が 0 であることも条件に足し、保留した記録の確定と移動の順は後続 (b) の事項として申し送った |
| R14 | Codex #3（P2） | 採る | scan で外した EJ の名前を一括の結果で名指しできない | snapshot に外した名前を持ち、一括の結果の読めない file の一覧へ引き継ぐ |
| R15 | Codex #4（P2） | 採る | conflict の案内が保存済みの控えの有無によらず「残しました」と言う | conflict ごとに保存済みかを持たせ、無ければ「保存していません」とし、リンクを出さない |
| R16 | Codex #5（P2） | 採る | Matrix の期待が最後の記録の保留と両立しない | 伸びた file と移された file の期待を、保留した記録が 1 度だけ入る形に直した |
| R17 | Codex #6（P2） | 採る | 保存の時に日時の暦の妥当性を検査しない | 暦にない日時を含む file を読めない file として扱う |
| R18 | Codex #7（P2） | 採る | 値引きの加算を消す mutation を fixture が検出できない | 有効な値引きが残る fixture と合計の assertion を足した |
| R19 | Codex #8（P2） | 採る | 有限の IPC の値を文字列に戻している | 失敗の kind は `CmdErrorKind`、EJ の kind・mode は response の enum にした |
| R20 | Codex #9（P3） | 採る | Matrix が存在しない field を検査する | EJ の file が無いときに EJ の項目が無いことの検査にした |
| R21 | Claude #2（P3） | 採る | Z004 の段のほかの error の行き先が無い | 日報の段と同じく、`DatabaseError` 以外は Skipped とした |
| R22 | Claude #3（P3） | 採る | 閲覧の型の所有が書かれていない | db で定義して `biz/mod.rs` で re-export する形を書いた |

起草役の問いの Coordinator の裁定（2026-10-18）: 連番の名前に改名した伸びうる file の書きかけを保存しうることは、保証の範囲に書いたとおり round 3 の判定に出す。保存済みと文の違う保留の記録に別の数は足さない（知らせの文で事実と合う）。conflict にその場で直す操作が無いことは owner に諮る候補のまま。

### Plan Review round 3（2026-10-18、head `08918379`）

Claude 側 fresh Opus 5.5（approve、P1 0・P2 0・P3 4）と Codex GPT-6 Astra（reject、P1 0・P2 2・P3 1）。Codex #2 と Claude #1 は同じ件で、重い方の P2 とした。全て採った（Coordinator の裁定）。round の天井で P2 が残ったので owner に諮り、「直して、追加の round なしで plan-approved（跡は Final Review）」と決まった（repo 外の回答台帳 TD-317）。起草役が `23be816e` で直した。

| # | 出所 | 裁定 | 要旨 | 直した所 |
|---|---|---|---|---|
| R23 | Codex #1（P2） | 採る | 一括取込みの結果に Z004 の部分成功と取り込まなかった行の知らせが無い | UI-07-D17 に部分成功の label、取り込まなかった行の数、既存の取込みの詳細へのリンクを書き、Matrix に検査を足した |
| R24 | Codex #2（P2）・Claude #1（P3） | 採る | 新しい控えが無いと、保留・食い違い・読めない file の詳細へ進めない。保留した記録がいつ入るかの文が正本と合わない | scan の要約に保留の数を足し、どれかがあればボタンから結果へ進める。保留した記録は次の読取りでもう一度確かめる規則を §37.11 の 1 か所に置き、画面の文・D-121・Ordinary Operation はそれを引く。毎日出る保留の文は注意の知らせと分けた補足とし、雑音かを (A) の L3 で確かめる |
| R25 | Codex #3（P3） | 採る | AC13 の開始の値の説明が実測と違う | 直した |
| R26 | Claude #2（P3） | 採る | 一括取込みの後に控えの query を無効にしない | §45.6c 手順 4 を key の置き場にし、手で選ぶ取込みと UI-09c はそこを引く |
| R27 | Claude #3（P3） | 採る | 同じ日と精算回数の 2 束の順が決まらない | `XZ` の束を先、次に候補の鍵の辞書順を足した |
| R28 | Claude #4（P3） | 採る | 番号の一周を再検討に挙げていない | BIZ-08-D9 の再検討に足した |

起草役の問いの Coordinator の裁定（2026-10-18）: 保留をボタンの条件に入れると、締めのたびにボタンが押せる状態になる。雑音かは (A) の L3 で確かめ、雑音なら保留を文とボタンの条件の両方から外す形が正本にあるので、この形を採る。食い違いにその場で直す操作が無いことは owner に諮る候補のまま。

Closeout（2026-10-18）: Final Review は broad 1 回で、Claude 側（Fable 5.1）= approve（P3 4）、Codex（Astra）= reject（P2 3）。是正 2 本の後の closure 1（Codex Sol）は reject（P2 1: 精算日を固定する Matrix の行）で、行を足した後の closure 2（Codex Sol）は approve（P3 1）。helper の record は review = pass。hosted の `Merge gate` は Ready の後の run で success で、owner の Ready・merge の承認の後に helper 経由で squash merge した。Writer の残した問いは是正で閉じ、backlog の (A)・(B) の項目は是正で更新済み。packet の本文（Scope・Contract Ledger・AC）は書き換えず、merge の版との食い違いと正本の場所をここに記録する: (1) Contract Ledger の「§37.9 に足すもの」の行が `ej_unreadable_files` を落としている（Final Review の Claude 側 O2）。(2) Scope（型・関数の形の変更と「367 の契約に後から足すもの」）と Contract Ledger の「§37.9 に足すもの」の行に、Final Review の是正で 37 に足した `z004_unreadable_files`・`z004_unreadable_count`・`RegisterSdBulkSource::Z004` の `settlement_date` が載っていない。(1)(2) の正本は 37 §37.10 の「§37.9 に足すもの」と §37.10 の本文。(3) AC8・AC9 の逐語の条件（`f5d7aaf4` を起点に差分が空）は、main（#174・#173・#170）の取込みで外れた。main を除いた本 lane の不変条件は PASS（closure 2 の Codex が確かめた）。Owner Effort Budget の表は closeout の時点の値（上限 9、消費 3）に直した。
