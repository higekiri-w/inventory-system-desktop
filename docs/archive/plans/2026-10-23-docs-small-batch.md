# Plan Packet: docs の棚卸し lane 1 の残りの小口（decision-log の Status・名前の参照・task-specs の注記・archive の link ほか）

wave に属さない単独の lane（docs 全体の棚卸し〈owner 2026-10-18、計画は repo 外の回答台帳 TD-327〉の lane 1 の残り。owner の決定は repo 外の回答台帳 TD-391〈2026-10-23〉: `docs/backlog.md` の「次の docs の lane の候補（docs の棚卸し lane 1 の残り）」の (1)〜(8) のうち、owner の判断が要らないものを 1 lane で直す）。docs だけ。code・migration・bindings・画面の振舞いは変えない。

出所: backlog の同じ項目（PR #180 の Follow-up と Final Review の closure、2026-10-20）。下の行番号と数は起点 `eed07d37`（main）で command を逐語で実行した出力（D-062）。本 packet は dated の Plan Packet なので、行番号を起票の時点の観測として書く（`docs/DOC_STYLE_GUIDE.md` §1 の対象外の項）。直した後の正本の文は行番号で引かない（D-125）。

規模と手法: 触る file は 19 と本 packet（下の Scope の段ごとの表）。作業を S1〜S7 に分け、段ごとに AC を置く。S7 は owner の答え（2026-10-23）で、S7-2・S7-3 を本 lane で直し、S7-1 を別の R3 の lane へ送る。本 run（起票）は packet だけを置き、docs の本体の直しは plan-approved の後の別の run で行う（先例: docs の棚卸し lane 1〈`docs/archive/plans/2026-10-19-docs-contract-refresh.md`〉と lane 2a〈`docs/archive/plans/2026-10-20-docs-file-line-rewrite.md`〉）。

## Workflow State

- Phase: archive
- Risk: R2
- Plan Commit: e5b6a111470a3c93a365cbbd126dc9173a7c09e7
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は本 run。docs の直しは plan-approved の後の別の run）
- Plan Reviewer: fresh Opus 5.5（fork でない）+ Codex。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Reviewer: fresh Opus 5.5（Claude 側。R2 で design lane でないので座組表の「それ以外」の行）+ Codex。座組表どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/docs-small-batch-1023

manual なし: 製品の runtime・画面・配布物を変えない docs の変更で、Windows native で確かめる物が無い。

遷移の記録:

1. kickoff → spec-check（2026-10-23、起草役）: 範囲を backlog の項目の (1)〜(8) にし、Risk を R2 と記録した（下の Risk）。発注 516 は R0/R1 を見込んでいたが、起草役の判定（R2）を Coordinator が採り、1 lane の R2 として進めると裁定した（2026-10-23）。
2. spec-check → plan-draft（2026-10-23、起草役、同じ commit）: 表の唯一の skip。直す先の設計正本（下の Design Readiness の一覧）は契約として足りていて、本 lane は新しい設計の選択をしない。S1〜S4・S6 は文を code（実装の現物）・正本・decision-log の規則に合わせるだけ、S5 は owner の決定（TD-350）を入庫の正本に書き写すだけで、決定の中身は決まっている。
3. plan-draft に留まる: 下の S7（owner の判断事項 3 つ）の答えを待つ。答えの後に Coordinator が packet を直して plan-gate へ進める。
4. plan-draft → plan-gate（2026-10-23、Coordinator、本 commit）: owner が S7 に答えた（repo 外の回答台帳 TD-393・TD-395: S7-2・S7-3 は本 lane で直す、S7-1 は別の R3 lane へ送る）。起草役が `1feb4aa8` で S7 を Scope に移し、S7-1 を backlog に足した。owner の判断事項は残らず、packet と Matrix に当たる表が揃い commit されている。Plan Review に出す。Codex が使えれば Codex と fresh Opus 5.5 を同じ round で、使えなければ fresh Opus 5.5 で始め Codex 分を後から足す（`docs/AGENT_OPERATING_MANUAL.md` §3.3）。
5. plan-gate のまま是正（round 1、2026-10-23、Coordinator、本 commit）: round 1（Claude 側 fresh Opus 5.5 だけ。Codex 側は未実施で round 2 で足す、`docs/AGENT_OPERATING_MANUAL.md` §3.3）の reject を起草役が `cc3d7a7d` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 2（fresh Opus 5.5 + Codex）で再 review する。
6. plan-gate のまま是正（round 2、2026-10-23、Coordinator、本 commit）: round 2（Claude 側 fresh Opus 5.5 だけ。Codex 側は round 3 で足す、§3.3）の reject を起草役が `85ee4756` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 3（天井、fresh Opus 5.5 + Codex）で再 review する。
7. plan-gate のまま是正（round 3〈天井〉、2026-10-23、Coordinator、本 commit）: round 3（Claude 側 fresh Opus 5.5 と Codex）の指摘を起草役が `a4d1bd8b` で一括して直した（下の Review Response）。round 4 は回さず、Coordinator の disposition の後に closure（fresh Opus 5.5 + Codex）に出す。P1/P2 = 0 なら plan-approved へ進める。
8. plan-gate → plan-approved（2026-10-23、Coordinator、本 commit）: closure は Claude 側 fresh Opus 5.5 = approve（P3 2）、Codex = approve（P1/P2/P3 0）で、P1/P2 = 0（下の Review Response）。Plan Commit は両方の reviewer が監査した `e5b6a111`。Claude 側の P3 2 件（packet の sweep の記録と所有の確かめの証跡の書き方）は implementing の本体の run で直す（Plan Commit を動かさない）。
9. plan-approved → implementing（2026-10-23、Coordinator、本 commit）: docs だけの lane で、本体の直しの run を発注する（packet の規則どおり、起票の run は packet だけを置いた）。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 4（decision point の単位、`docs/DEV_WORKFLOW.md` Owner Effort Budget: TD-391〈lane の選択〉1、S7 の 3 つの答え〈S7-1・S7-2・S7-3。TD-393・TD-395、2026-10-23〉3） | 2（Ready 1・merge 1。plan-approved は repo 外の回答台帳 TD-296 の 4a で Coordinator が進める） | 0 | 6 = 4 + 2 + 0 |

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R2

Reason:
docs だけの変更で、runtime の契約を変えない。ただし `docs/DEV_WORKFLOW.md` Risk Tiers の R0（「Typo, link, non-semantic doc cleanup.」）には当たらない。S1 は decision-log の Status の行から決定の状態と出所でない文を外し、S3 は task-specs に今の runtime を書き分ける注記を足し、S5 は入庫の正本に決まりを足し、S6 は設計の規則と正本の文を今の code・正本に合わせ、S7 は db-design の設計意図の理由の句を今の決定に合わせる。どれも文の意味を変える。R2 の行（「docs change that affects maintainability but not runtime contracts.」）に当たる。同じ種類の作業を R2 とした先例は lane 1（#180、S3 で Status を直した）。境の規則（R2 と R3 で迷えば、安定した契約・出力 schema・data safety の境・command の形・route・workflow gate に触れるなら R3）には当たらない: command・DTO・DB・画面の振舞い・merge gate を変えない。`docs/msi-distribution-runbook.md`（data safety の手順書）は S2 の参照の語 1 か所だけを替え、手順・判定を変えない。触る file はどれも `scripts/ci/classify-changes.sh` の workflow の一覧（`case` の 2 つの path の列）に無い（`eed07d37` で確かめた。S7 の 3 file は `d7157d0c` で `rg -n 'db-design|master-tables|pos-tables|transaction-tables' scripts/ci/classify-changes.sh` が何も出さない。S6-4 の `docs/SCREEN_DESIGN.md` は `217c8906` で `rg -n 'SCREEN_DESIGN' scripts/ci/classify-changes.sh` が何も出さない）ので、classifier は `workflow=true` を出さず、Final Review Minimum は 1 のまま。function-design を触るので traceability の分類には当たる。Human Gate は `ready,merge`（R2 の既定）。

## Goal

Goal Invariant:

### 最小完了条件

- decision-log の S1 の 15 の D の Status の行が、decision-log の頭の「Status の書き方」と D-126 の形（状態の語と、括弧の中の出所だけ）になり、決定の本文は 1 行も変わらない。
- 設計正本・runbook・ADR が backlog の項目を名前で引く所（S2）が、今の backlog の項目の名前か、設計の節の ID で引かれ、読んだ人が今ある項目・節へ着く。
- `docs/architecture/cmd-task-specs.md` の CMD-01・CMD-08 と `docs/architecture/ui-task-specs.md` の UI-01a・UI-01b・UI-08 を読んだ人が、D-117 の後の形（採用済み）と今の build の振舞いを取り違えずに読める（S3）。
- archive の 3 file の link（S4）が、項目の今の置き場へ着く。
- 入庫の正本（[61](../../function-design/61-ui-receiving.md)）を読んだ人が、取り寄せ（客注）の品を入庫に入れない決まり（TD-350）を読める（S5）。
- DSR-22・56・`docs/SCREEN_DESIGN.md` の S6 の文が、今の code と `docs/SCREEN_DESIGN.md` の「主要な画面間遷移」と合う。
- db-design の 2 か所の「色を区別しない方針」の理由の句（S7-2）と、`manual_sales` の役割・ヒアリングの結論・設計意図と `docs/SCREEN_DESIGN.md` の画面一覧の UI-04 の行が書く手動販売の使い道（S7-3）が、今の決定と正本に合う。照合の規則と reason の値は変わらない。
- backlog の「次の docs の lane の候補」の項目が外れ（D-127）、S7-1 の code の側の直しが backlog の別の項目にある。

### 失敗定義

- decision-log の決定の本文（Status の行の外）を書き換える（D-126 (3)）。
- Status の行から外した文に、他の場所に無く今も効く中身があったのに、確かめずに消す。
- 直した後の文が code の現物・正本と食い違う（例: S3 の注記が、今の build に無い command を「今ある」と書く。S6 の DSR-22 が `StockStatusBadge` の今の表示名と違う名を書く）。
- S7-1（部門 12 の `code_prefix`）を本 lane で直す（master-tables を `TK` にする、code に触る）。
- S7-2 で先頭へ紐付ける規則そのものを、S7-3 で reason の値を変える。
- 店の運用の細部・実値・発言の原文を repo に書く。
- 並走の lane の行（下の「他の lane との所有」）を触る。

### 非目的

- code・migration・bindings・route・画面の文言を変える（S7 の部門 12 の `code_prefix` を code の側で直すことも含む）。
- decision-log の Status の規則そのもの（頭の節と D-126）を変える。
- S1 の 15 の D の外の Status、ADR の Status を直す。
- backlog の他の項目の名前の参照、archive の他の file の link を洗い直す。
- file:行 の引用の書き換え（lane 2a で済み、検査は lane 2b）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

not applicable: operator の操作・data / command の契約・業務の状態遷移を変えない docs の変更で、workflow の変更でもない（通常列は発注 → review → owner の Ready / merge）。S5 は入庫の画面の使い方の決まりを正本に書き写すが、画面の操作列と結果を変えない。

## Scope

段ごとの対象と直し方。各段の表の「結果」列は実装の run が「直した」か「残す（理由）」で埋める（AC-S0-1）。直す前に、主張の根拠（code・decision-log・正本・archive の packet）を `rg` か実読で確かめ、確かめられなければ直さずに「残す（未確認）」とする。新しく書く文の引用は D-125・`docs/DOC_STYLE_GUIDE.md` §1 の形（path と symbol、節の ID か見出し）。新しい文言に作業の印の語（AC-S9-1 の command が探す 7 語）を使わない。

### S1 decision-log の Status の 15 行（backlog の (1)）

規則: decision-log の頭の「Status の書き方」（D-126）。Status は状態の語だけを書き、括弧に出所（owner の決定の日付・repo 外の回答台帳の ID・design lane）を添えてよい。決定の本文は main に merge された後は append-only で、後から直してよいのは Status の行（状態の語と出所）・Status の行の追加・Superseded の行の追加・file の移送に伴う link の path だけ。よって、Status の行の括弧の外にある文と、括弧の中の出所でない文は Status の行から外す。括弧の中で出所として残すのは、日付・回答台帳の ID と、決定が出た場（owner の指示・了承・plan-approved や Plan Gate の名とその PR、決定を記録した Plan Packet・design lane、review の round、選んだ選択肢の記号）だけ。決定が出た場は D-126 自身の Status の「…〈Plan Packet〉で起票した」と同じく済んだ場を引く句で、これから反映する lane・PR（実装の状態）とは別（S1-14・S1-15）。決まった中身を述べる文（「X は Y にする」「X で確定」の形）と、どの lane・PR が反映するか（decision-log の頭の「Status の書き方」の 2 項目め）は出所でないので外す（S1-6・S1-14）。外した文は本文へ移せない（append-only）。外して中身が失われないことを、行ごとに他の置き場で確かめた（下の表の「外す文の置き場」）。どの行も規則で直し方が一つに決まり、owner の判断事項に回す行は無い。

| # | D（`eed07d37` の行） | 今の形 | 直した形の向き | 外す文の置き場 | 結果 |
|---|---|---|---|---|---|
| S1-1 | D-042（329） | 英文: `accepted during the Phase B R4 Design Phase, before the plan-first execution commit and before any destination mutation.` | `accepted（Phase B の R4 の Design Phase で、plan-first の実行の commit と移行先の変更の前）`。時点は出所として括弧に入れる | 外す文なし（訳すだけ） | 直した |
| S1-2 | D-075（626） | 括弧の後に「画面設計の中身（一括改定画面のレイアウト・操作フロー等）と SP-103-04（商品一覧の原価列追加）の最終判断は本決定の対象外とし、Plan Packet に委ねる。」 | 括弧の後の文を外し、`accepted（owner 裁定 2026-08-21〜22、issue #90 聞き取り）` | 委ねた先は済んでいる: 一括改定の画面は [77](../../function-design/77-ui-bulk-price-revision.md)（UI-14）、SP-103-04 は [50](../../function-design/50-ui-product-list.md) UI-01a-D13（`docs/spec/requirements-coverage.md` の SP-103-04 の行は current）。委ねた事実は `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md` の SP-103-04 の行（「最終判断は Plan Packet」）と D-075 の Impact の行（Plan Packet の起草の入力）に残る | 直した |
| S1-3 | D-086（729） | 括弧の後の「。」 | 「。」を外す | — | 直した |
| S1-4 | D-088（746） | 同上 | 同上 | — | 直した |
| S1-5 | D-090（760） | 同上 | 同上 | — | 直した |
| S1-6 | D-091（767） | 括弧の後の「。」。括弧の中に出所でない文「検索欄はボタンなしで確定」（決まった中身）と「runtime lane A は 1 PR」（どの lane・PR が反映するか） | 2 句を外し（日付・「デザインの決まりの見直しでの判断」・2 つの試しの了承は決定が出た場として残す）、「。」を外す。向き: `accepted（owner 2026-09-24。デザインの決まりの見直しでの判断、ランキング 1 位の試しの了承。owner 2026-09-25。詳細を開いた行を進行中にそろえる試しの了承）` | 「ボタンなし」は [02](../../design-system/02-component-catalog.md) ⑨ の `SearchBar` の live 型の JSX の comment（「ボタンなし」）と D-091 の Decision (7) の撤回の列（「検索ボタン併記」）。両方の owner の答えは出所の packet `docs/archive/plans/2026-09-24-design-rules-renewal.md` の「owner 2026-09-24（Plan Review round 2 の論点への回答）」の項（検索欄はボタンなしで確定、runtime lane A は 1 PR でよい） | 直した |
| S1-7 | D-092（780） | 括弧の後の「。」 | 「。」を外す | — | 直した |
| S1-8 | D-093（793） | 同上 | 同上 | — | 直した |
| S1-9 | D-094（800） | 括弧の後に「試し 8 点の採否は owner の L3 round 1（2026-09-28）の答えで確定した（下の「試し 8 点の採否」）。」 | 括弧の後の文を外し、出所として「owner の L3 round 1〈2026-09-28〉で試し 8 点の採否」を括弧の中に足す | 本文の「試し 8 点の採否（owner の L3 round 1、2026-09-28）」の項が同じ中身を持つ | 直した |
| S1-10 | D-096（814） | 括弧の後の「。」 | 「。」を外す | — | 直した |
| S1-11 | D-097（824） | 同上 | 同上 | — | 直した |
| S1-12 | D-098（835） | 同上 | 同上 | — | 直した |
| S1-13 | D-099（845） | 同上 | 同上 | — | 直した |
| S1-14 | D-109（937） | 括弧の後の「。」。括弧の中に出所でない決定の文「同日の owner 決定: 時点証拠 ADR は ② 〜 ④ の間 proposed のまま進め、⑤ で停止 ADR を superseded にするのと同時に accepted にする。STK-2 の診断 `XFA_LATE_IMPORT_FAIL` は ⑤ の時点で「要再確認のあと数え直しで正しい数へ戻る」を green とする」 | 決定の文を外し、「。」を外す。出所の「Plan Gate 2026-10-07、PR #148」（D-109 を記録した packet が Plan Gate を経た PR。決定が出た場）、「owner 2026-10-06 の lane 選択」「Plan Packet `docs/archive/plans/2026-10-06-stocktake-p1.md` の設計判断」は残す。向き: `accepted（Plan Gate 2026-10-07、PR #148。owner 2026-10-06 の lane 選択で起票。Plan Packet `docs/archive/plans/2026-10-06-stocktake-p1.md` の設計判断）` | 前半は D-109 の Compatibility の行（「時点証拠 ADR は本 lane で編集せず、Status は ⑤ で accepted にする（owner 2026-10-06）」）が持つ。後半を含む両方は出所の packet の Q2 決定 (A) の項と `XFA_LATE_IMPORT_FAIL` の行が持つ。⑤ の列は D-109 の `Superseded in part by: D-118` の行が D-118 の runtime に置き換わったと書く | 直した |
| S1-15 | D-123（1086） | 括弧の後の「。」。括弧の中に「棚卸しの数の置き場と在庫数の求め直しの runtime (a) の起票〈Plan Packet `docs/archive/plans/2026-10-18-stocktake-count-runtime-a.md`〉で正本へ置いた」 | 「。」を外す。「…の起票〈Plan Packet …〉で正本へ置いた」は、TD-311 の決定を記録した Plan Packet の名（決定が出た場）として残す（これから反映する lane でなく済んだ起票の場で、D-126 自身の Status の「…で起票した」と同じ形） | — | 直した |

### S2 名前で引く参照（backlog の (2)）

旧名の項目は、backlog の今の項目の頭の「旧名」の句か、[閉じた項目の archive](../2026-10-19-backlog-closed.md) で辿れる。直す先は、設計の節があればその ID、無ければ今の backlog の項目の名前。

| # | file（`eed07d37` の行） | 今の句 | 直す先 | 結果 |
|---|---|---|---|---|
| S2-1 | `docs/function-design/29-io-daily-report-parser.md`（97） | 「単位の拡張（[backlog] の「単位の拡張」。Z004 の小数を 100 倍の整数で受け、商品の単位が長さなら BIZ で cm に換算する案）」 | [10](../../function-design/10-common-rules.md) SPEC-UNIT-D8（POS の数量〈Z004・EJ〉を 100 倍の整数で読む契約）。「案」の言い換えは外す（SPEC-UNIT-D8 が契約を持つ） | 直した |
| S2-2 | 同（113） | 「backlog の「既存の支払集計（Z002）で異なる項目が 1 行に合算されうる」で扱う」 | backlog の「日報の紙の runtime」（旧名の句がこの項目の頭にある） | 直した |
| S2-3 | `docs/function-design/33-biz-plu-export-service.md`（82） | 「Z004 の業務の取込みを同じ SD の読取りで行うようになったら（後続の lane。backlog の「売上の SD からの一括取込みと EJ の控えの閲覧」）、占有の照合と取込みの順と TX の分け方はその lane が決める」 | S2-4 と同じ形。順と TX は設計が済んでいる（PR #175、D-121）: 「占有の照合（[37](../../function-design/37-biz-daily-report-import-service.md) §37.9 の手順 10 で読んだ Z004 を BIZ-04 が照合する）の後に Z004 を取り込む順と TX の分け方は、37 §37.10（手順 3、`z004_files` は PLU の照合の後も残す）・BIZ-08-D7（D-121。項目ごとの TX、順は日報 → Z004 → EJ）が決めた。runtime は backlog の「Z004 を SD の一括取込みに入れる runtime（B）」」の向きへ。「その lane が決める」は外す | 直した |
| S2-4 | `docs/function-design/37-biz-daily-report-import-service.md`（447） | 「…の設計は後続の lane（backlog の「売上の SD からの一括取込みと EJ の控えの閲覧」）が決める。」 | 設計は済んでいる（PR #175、D-121）。「…の設計は §37.10・§37.11（D-121）が決めた」の向きへ。runtime は §37.10 の冒頭が backlog を引く | 直した |
| S2-5 | `docs/msi-distribution-runbook.md`（31） | 「本書の外の lane で行う（backlog の「MSI 配布手順 docs 化」の項）」 | backlog の「MSI の製品名と upgrade code の固定」。参照の語だけを替え、手順と判定を変えない。並走の MSI の固定の lane との扱いは下の所有表 | 直した（MSI の固定の lane〈PR #191〉はこの時点で merge していないので、所有の表どおり本 lane が直した。後に merge する方が単段 merge で同期する） |
| S2-6 | `docs/adr/2026-09-18-stocktake-time-evidence.md`（228） | 「backlog の「実測とPOS系列の対応を取得・保存する」の入力1・入力2」 | backlog の「実測とPOS系列の対応の入力 1〜5」の入力 1・入力 2。同じ ADR の 232・233・238 の「同項目の入力 N」は 228 を受けるので、228 を直せば着く（触らない） | 直した |

直さない（D-126 (3) で一意に決まる）: decision-log の D-104（902 の Guarantee range、903 の Compatibility）、D-105（910 の Why）、D-120（1051 の Decision、1052 の Why）の句は、どれも決定の本文の行にある。D-126 (3) は main に merge された決定の本文を append-only にし、後から直してよいのは Status の行・Status の行の追加・Superseded の行の追加・file の移送に伴う link の path だけで、名前の書き換えはそのどれでもない。旧名は今の backlog の項目の頭の「旧名」の句で辿れる（S2-1・S2-2 の直す先の項目、[閉じた項目の archive](../2026-10-19-backlog-closed.md)）。backlog の項目から外す。

### S3 task-specs の D-117 の runtime の注記（backlog の (3)）

形: 既に注記を持つ `docs/architecture/biz-task-specs.md` の BIZ-04 と `docs/architecture/io-task-specs.md` の IO-04 と同じ「> **runtime の状態**: 上は D-117 の後の形（採用済み）。今の build は …。外すのは PLU の書出しの見直しの runtime（[33](../../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。」の 1 段。注記は、その行の task-spec の文が D-117 で外す（または D-117 の後の形に無い）と書く物を全部挙げ、build に残る物ごとに symbol を書く（下の表の 2 列め・3 列め。一部だけ挙げると、挙げなかった物を撤去済みと読ませる）。今の build の事実は、実装の run が下の command を逐語で実行し直してから書く（AC-S3-2）。

| # | 行 | task-spec の文が D-117 で外す物 | 今の build（`eed07d37` の code で確かめた symbol） | 結果 |
|---|---|---|---|---|
| S3-1 | cmd-task-specs の CMD-08 の表の後 | 表の `get_plu_slot_summary` の出力は `snapshot_at` だけで占有の照合は CMD-12 `scan_register_sd`（D-117）、`prepare_plu_export` の出力に `over_limit_warning` が無い、表に Z004 を選ぶ command が無い。よって外す物は (i) 件数の要約、(ii) `over_limit_warning`、(iii) Z004 を選ぶ command | (i) `get_plu_slot_summary` が `src-tauri/src/biz/plu_export_service.rs` の `PluRegisterSnapshotSummary`（`free_count` ほか、`release_pending_count` を含む）を返す。(ii) `prepare_plu_export` の結果（同 file の `PluExportPreparedResult` と `src-tauri/src/cmd/plu_export_cmd.rs` の応答）が `over_limit_warning` を持つ。(iii) `src-tauri/src/cmd/plu_export_cmd.rs` の `import_plu_register_snapshot`（利用者が選ぶ Z004 で占有を照合する。`src-tauri/src/lib.rs` の `collect_commands` に登録）。CMD-12 の `scan_register_sd` は `src-tauri/src` に無い | 直した（直す時点で S3 の表の後の command を逐語で実行し直し、出力は上と同じ） |
| S3-2 | ui-task-specs の UI-01a | タスク要求の括弧「PLU の移行状態の列・絞り込み・一括更新は D-117 で撤去」。外す物は (i) 列、(ii) 絞り込み、(iii) 一括更新 | (i) `src/features/products/components/ProductTable.tsx` の `ProductTable` の「PLU」の列（対象外 / 未反映 / 反映済み の Badge。`ProductListPage.tsx` が使う）。(ii) `src/features/products/ProductListPage.tsx` の PLU の絞り込み（URL の `plu`）。(iii) 同 file の、絞り込みに一致する全件の一括更新（`commands.bulkSetPluTarget`） | 直した（同上） |
| S3-3 | ui-task-specs の UI-01b | 操作フロー 6「edit は memory No. を表示しない（D-117）」。外す物は memory No. の表示 | `src/features/products/ProductFormPage.tsx` の edit が「レジメモリNo.」（`plu_memory_no`）を表示する | 直した（同上） |
| S3-4 | ui-task-specs の UI-08 | 状態管理の「照合は売上データ取込みの SD の読取りで自動、D-117」と、CMD 呼び出しに Z004 を選ぶ command が無いこと。外す物は (i) 利用者の選ぶ Z004 での照合（FilePicker）、(ii) 件数の要約、(iii) `over_limit_warning` の表示。D-117 の後に足す物は SD の読取りでの自動の照合 | `src/features/plu-export/PluExportPage.tsx` が (i) `FilePicker`（`@/components/FilePicker`）で選んだ Z004 を `commands.importPluRegisterSnapshot` へ渡す、(ii) `get_plu_slot_summary` の `free_count`・`release_pending_count` ほかを出す、(iii) `overLimitWarning`（`prepare_plu_export` の `over_limit_warning`）を持つ。SD の読取りでの自動の照合は無い（`scan_register_sd` が無い） | 直した（同上） |
| S3-5 | cmd-task-specs の CMD-01 の表の後 | 表は D-117 の後の形で、(i) `search_products` の `SearchQuery` に PLU の絞り込みが無い、(ii) filter に一致する全件の PLU 対象の一括更新の command（[40](../../function-design/40-cmd-product.md) の `bulk_set_plu_target` コマンド、CMD-01-D4。撤去の対象、D-117 (2)）が無い。外す物は (i) PLU の絞り込み、(ii) 一括更新（S3-2 の UI-01a の (ii)・(iii) の CMD の側） | (i) `search_products`（`src-tauri/src/cmd/product_cmd.rs`）の入力 `src-tauri/src/db/product_repo.rs` の `ProductSearchQuery` が `plu`（`PluMigrationFilter`）を持つ。(ii) `src-tauri/src/cmd/product_cmd.rs` の `bulk_set_plu_target`（`ProductBulkFilter` と `plu_target`。`src-tauri/src/lib.rs` の `collect_commands` に登録） | 直した（同上） |

S3-5 の確かめた command と出力（D-062。round 2 の是正で足した行で、`4b2fe746` で実行。code は起点と同じ）:

```bash
rg -n -e 'pub fn search_products' -e 'pub fn bulk_set_plu_target' -e 'pub plu: Option<PluMigrationFilter>' -e 'pub struct ProductSearchQuery' -e 'pub struct ProductBulkFilter' -e 'product_cmd::bulk_set_plu_target' src-tauri/src/cmd/product_cmd.rs src-tauri/src/db/product_repo.rs src-tauri/src/lib.rs
```

```text
src-tauri/src/lib.rs:284:        cmd::product_cmd::bulk_set_plu_target,
src-tauri/src/lib.rs:1499:            cmd::product_cmd::bulk_set_plu_target,
src-tauri/src/cmd/product_cmd.rs:77:pub fn search_products(
src-tauri/src/cmd/product_cmd.rs:91:pub fn bulk_set_plu_target(
src-tauri/src/db/product_repo.rs:266:pub struct ProductSearchQuery {
src-tauri/src/db/product_repo.rs:277:    pub plu: Option<PluMigrationFilter>,
src-tauri/src/db/product_repo.rs:288:pub struct ProductBulkFilter {
src-tauri/src/db/product_repo.rs:292:    pub plu: Option<PluMigrationFilter>,
src-tauri/src/db/product_repo.rs:832:pub fn search_products(
```

S3-1〜S3-4 の確かめた command と出力（D-062。`217c8906` で実行。`eed07d37` から変わったのは本 packet と `docs/backlog.md` だけで、code は起点と同じ）:

```bash
rg -n -e 'pub fn import_plu_register_snapshot' -e 'pub fn get_plu_slot_summary' -e 'pub over_limit_warning' -e 'pub release_pending_count' -e 'pub free_count' -e 'plu_export_cmd::import_plu_register_snapshot' src-tauri/src/cmd/plu_export_cmd.rs src-tauri/src/biz/plu_export_service.rs src-tauri/src/lib.rs
rg -c 'scan_register_sd' src-tauri/src
rg -n -e '<TableHead>PLU</TableHead>' -e '対象外$' -e '未反映$' -e '反映済み$' src/features/products/components/ProductTable.tsx
rg -n -e 'commands.bulkSetPluTarget' -e 'value=\{normalizedSearch.plu\}' -e '<ProductTable ' src/features/products/ProductListPage.tsx
rg -n -e 'レジメモリNo.$' -e 'plu_memory_no' src/features/products/ProductFormPage.tsx
rg -n -e 'commands.importPluRegisterSnapshot' -e 'free_count' -e 'release_pending_count' -e 'overLimitWarning: nextPrepared' -e '<FilePicker' src/features/plu-export/PluExportPage.tsx
```

```text
src-tauri/src/biz/plu_export_service.rs:52:    pub over_limit_warning: bool,
src-tauri/src/biz/plu_export_service.rs:90:    pub free_count: usize,
src-tauri/src/biz/plu_export_service.rs:94:    pub release_pending_count: usize,
src-tauri/src/biz/plu_export_service.rs:119:pub fn import_plu_register_snapshot(
src-tauri/src/biz/plu_export_service.rs:292:pub fn get_plu_slot_summary(conn: &DbConnection) -> Result<PluRegisterSnapshotSummary, BizError> {
src-tauri/src/cmd/plu_export_cmd.rs:42:    pub over_limit_warning: bool,
src-tauri/src/cmd/plu_export_cmd.rs:149:pub fn import_plu_register_snapshot(
src-tauri/src/cmd/plu_export_cmd.rs:170:pub fn get_plu_slot_summary(
src-tauri/src/lib.rs:329:        cmd::plu_export_cmd::import_plu_register_snapshot,
src-tauri/src/lib.rs:1544:            cmd::plu_export_cmd::import_plu_register_snapshot,
（2 本めは何も出さない、exit 1）
48:          <TableHead>PLU</TableHead>
88:                  対象外
94:                  未反映
99:                  反映済み
65:      unwrapResult(commands.bulkSetPluTarget(buildProductBulkFilter(normalizedSearch), pluTarget), {
146:        value={normalizedSearch.plu}
343:          <ProductTable items={productsQuery.data.items} returnTo={returnTo} />
231:            レジメモリNo.
236:            value={currentProduct?.plu_memory_no ?? "未割当"}
157:    overLimitWarning: nextPrepared.over_limit_warning,
216:  const releasePendingCount = slotSummaryQuery.data?.release_pending_count ?? 0;
224:      await unwrapResult(commands.importPluRegisterSnapshot(Array.from(file.bytes)), {
385:                    {slotSummaryQuery.data.free_count.toLocaleString()} 件
417:          <FilePicker
```

### S4 archive の link（backlog の (4)）

link の先の backlog の節「やると決めたもの（順番未定）」は今の `docs/backlog.md` にも `####` の見出しで残り、anchor `#やると決めたもの順番未定` は今もその節に着く。ただし引く項目は、DATA-2 を除いて 2026-10-19 に [閉じた項目の archive](../2026-10-19-backlog-closed.md) の同じ名の節（NAV-1 だけは archive の「次に動く lane」の節）へ移った。DATA-2 だけは今の backlog の同じ節にある。archive の本文の出来事の記述は変えず、項目が移った link の先だけを今の置き場へ替える（項目が今もその節にある link は替えない）。確かめた command と出力（`4b2fe746` で実行。行は抜粋）:

```bash
rg -n 'やると決めたもの' docs/backlog.md docs/archive/2026-10-19-backlog-closed.md
rg -n -e 'DATA-2' -e 'STK-1' -e 'STK-2' -e 'NAV-1' -e 'DOC-2' -e 'destructive Alert' -e '実測とPOS系列' docs/backlog.md docs/archive/2026-10-19-backlog-closed.md
```

```text
docs/archive/2026-10-19-backlog-closed.md:20:## やると決めたもの（順番未定）
docs/backlog.md:13:#### やると決めたもの（順番未定）
（1 本めはほかに backlog の 9・19・33・46 行の本文が出る）
docs/archive/2026-10-19-backlog-closed.md:17:- [x] 在庫変動履歴からの戻りで在庫照会の検索条件と商品選択を保持する（NAV-1）: …
docs/archive/2026-10-19-backlog-closed.md:27:- **実測とPOS系列の対応を取得・保存する**（design-first、R3）: …
docs/archive/2026-10-19-backlog-closed.md:37:- **STK-1（P1、設計上の問題）**: …
docs/archive/2026-10-19-backlog-closed.md:39:- **STK-2（P1 候補）**: …
docs/archive/2026-10-19-backlog-closed.md:41:  - ㉘ ③ 〜 ⑤ の申し送り（② の packet から、2026-10-06）: …
docs/archive/2026-10-19-backlog-closed.md:51:- **DOC-2**: …
docs/archive/2026-10-19-backlog-closed.md:80:- [x] **destructive Alert の soft 塗り + 三角 icon**（…）
docs/backlog.md:39:- **DATA-2**: 共有 JAN の売上を先頭 SKU へ割り当てる現行仕様の限界（…）
（ほかの行は子の項目と本文の言及。archive の見出しは `## 次に動く lane（順番固定）` が 5 行、`## やると決めたもの（順番未定）` が 20 行。backlog の `#### やると決めたもの（順番未定）` は `### 着手対象`〈11 行〉の下で、次の見出しは 118 行の `### 保留`）
```

下の表の「閉じた項目の節」は `../2026-10-19-backlog-closed.md#やると決めたもの順番未定`（3 file とも `docs/archive/` の 1 段下にあるので相対 path は同じ）。

| # | file（`eed07d37` の行） | 引く項目 | 直す先 | 結果 |
|---|---|---|---|---|
| S4-1 | `docs/archive/plans/2026-09-15-display-fixes-batch-2.md`（327） | destructive Alert の soft 塗り + 三角 icon | 閉じた項目の節（項目はその節にある） | 直した |
| S4-2 | `docs/archive/plans/2026-09-23-stocktake-time-evidence-adr-revision.md`（94） | 実測と POS 系列の対応を取得・保存する | 同上 | 直した |
| S4-3 | `docs/archive/harness-context/2026-09-30-Plans-completed.md`（54） | 「型生成の失敗が成功扱いになる」entry | 項目は PR #88 で閉じ、backlog にも閉じた項目の archive にも無い。閉じた packet `../plans/2026-09-22-bindings-export-failure-detection.md` を引く | 直した（backlog への link を外した。閉じた packet は同じ entry の後ろの「archiveのPlan Packet」の link が既に引く） |
| S4-4 | 同（68） | 監査所見 STK-1 / STK-2 / NAV-1 / DATA-2 / DOC-2 | DATA-2 は今の backlog の同じ節にあるので、今の backlog の link（anchor つき）は替えない。ほかの 4 つは閉じた項目の archive にあり（STK-1・STK-2・DOC-2 は表の冒頭の「閉じた項目の節」、NAV-1 は archive の「次に動く lane」の節で、節が 2 つに分かれる）、閉じた項目の archive の link を anchor を付けずに `../2026-10-19-backlog-closed.md` で足す。1 つの link で 2 つの置き場を指せないので link は 2 つになる | 直した（DATA-2 の link は替えず、閉じた項目の archive の link を anchor なしで足した） |
| S4-5 | 同（112） | 「㉘ ③ 〜 ⑤ の申し送り」 | 閉じた項目の節（STK-2 の項目の子） | 直した |

### S5 取り寄せの品を入庫に入れない決まり（backlog の (7)、TD-350）

[61](../../function-design/61-ui-receiving.md) §61.1 の設計判断の表に 1 行（`UI-02-D17`）を足し、§61.10 の変更履歴に 1 行を足す。中身: 取り寄せ（客注）の品が届いたときは入庫に記録しない（owner の決定、repo 外の回答台帳 TD-350、2026-10-20）。理由は `docs/project-memory.md` の Store Premises の棚卸しの項（取り寄せは先払いで、受注の時点で JAN が分からないことがあるのでレジで部門キーで打つ。先払いの取り寄せ品はお渡し待ちの間に棚卸しになっても数えない）を引いて書き、店の運用の細部・実値は写さない。アプリの検査・画面の文言は足さない（決まりを正本に置くことが TD-350 の決定）。

| # | 行 | 結果 |
|---|---|---|
| S5-1 | 61 §61.1 の `UI-02-D17` | 直した |
| S5-2 | 61 §61.10 の変更履歴 | 直した |

### S6 引用が支える文と今の code・正本の食い違い（backlog の (8)）

| # | file（`eed07d37` の行） | 今の文 | 直す向き | 根拠 | 結果 |
|---|---|---|---|---|---|
| S6-1 | `docs/design-system/01-decision-rules.md` DSR-22（136） | 「canonical `通常`=…`StockStatusBadge` の中立の Badge」 | 「canonical `在庫あり`」 | `src/features/stock-inquiry/components/StockStatusBadge.tsx` の `StockStatusBadge` の中立の Badge の文字は「在庫あり」（icon なし、枠あり） | 直した |
| S6-2 | `docs/function-design/56-ui-daily-sales.md`（375） | 「`source: string` に対する防御的 if-else 実装…未知値は内訳に含めない（防御的設計、bindings.ts では literal union 化されていない）」 | 「`source`（bindings の `DailySaleSource`）を `switch` で分け、`default` の `never` で網羅を型で確かめる（値が増えれば compile error）」の向き | `src/features/daily-sales/lib/compute-summary.ts` の `computeSalesLineSummary` は `switch (item.source)` と `const exhaustive: never`。`src/lib/bindings.ts` の `DailySaleItem` の `source` は `DailySaleSource` | 直した |
| S6-3 | 同（26） | 「SCREEN_DESIGN.md の主動線「CSV取込み完了 → 売上レポート確認」を成立させる」 | 「SCREEN_DESIGN.md の主要な画面間遷移「売上データ取込み完了 → 売上レポート」を成立させる」 | `docs/SCREEN_DESIGN.md` の「主要な画面間遷移」の項に「売上データ取込み完了 → 売上レポート（自然な流れ）」がある | 直した |
| S6-4 | `docs/SCREEN_DESIGN.md` 在庫照会の「利用者配慮」（264） | 「「在庫切れ」「在庫少」「通常」の日本語ラベル + アイコン / バッジを主情報」 | 経緯の文（2026-06-07 の L3 の高視認性 follow-up が「在庫切れ」「在庫少」「通常」を主情報にした）は「通常」のまま残し、今の中立の label を別の句で書く（向き: 「…を主情報にした。中立の label は今「在庫あり」〈`StockStatusBadge`〉」）。「通常」を置き換えると、follow-up が「在庫あり」を足したと読め、経緯と違う | S6-1 と同じ `StockStatusBadge`。[02](../../design-system/02-component-catalog.md) の `StockStatusBadge` の項。中立の label は PR #64 で「在庫あり」になった（`git log -S'在庫あり' --format='%h %ad %s' --date=short -- src/features/stock-inquiry/components/StockStatusBadge.tsx` の最古の行が `f2ef9e52 2026-09-16 fix(ui): 表示小修正batch 2 — 見出し・Alert・在庫照会・Badge・明細数列 (#64)`。`1fba4d45` で実行） | 直した |

### S6b backlog の項目の書き直し

`docs/backlog.md` の「次の docs の lane の候補（docs の棚卸し lane 1 の残り）」（`eed07d37` の 111 行）を外す。直したもの（(1)〜(4)・(6) の S7-2・S7-3・(7)・(8)）、直さないことが一意に決まったもの（(2) の decision-log の句、(5)。理由は S2 の末尾と下の S5b）、別の lane へ送ったもの（(6) の S7-1。行き先は起票の run が足した backlog の項目「部門 12（宅急便）の初期データの `code_prefix` を正本の NULL に合わせる」）で、残りが無い。継ぎ足さない（D-127）。

項目を外すと、`docs/Plans.md` の `## 次の行動` の「backlog に足した「次の docs の lane の候補」」の句が無い項目を名で引く。本 lane は `docs/Plans.md` を触らず（D-097）、この句は本 lane の closeout の PR が `## 次の行動` を今の状態に合わせる手順（`docs/DEV_WORKFLOW.md` Post-Merge Closeout、D-101）で外す。closeout の PR は `rg -n '次の docs の lane の候補' docs --glob '!docs/archive/**' --glob '!docs/plans/**'` が何も出さない（exit 1）ことを確かめる（`217c8906` の出力は 2 行: `docs/Plans.md:11`・`docs/backlog.md:112`。本 lane の merge の後は `docs/Plans.md` の 1 行が残る）。

S2・S3 で不要になる backlog の句も同じ run で直す（D-127）:

- 旧名の句の引き手: 3 つの項目の頭の「旧名」の句の「〜が引く」から、本 lane が直して旧名を引かなくなる file の名だけを外す。「SD の一括取込みと EJ の控えの runtime（A）」の「33・37 が引く」（S2-3・S2-4）、「日報の紙の runtime」の「29D・decision-log の D-120 が引く」の「29D」（S2-2）、「単位の拡張の残りの申し送り」の「29D・decision-log が引く」の「29D」（S2-1）。decision-log は旧名を引き続ける（S2 の末尾）ので、旧名の句と decision-log の名は残す。「MSI の製品名と upgrade code の固定」の「`docs/msi-distribution-runbook.md` が引く」は並走の MSI の固定の lane の項目なので触らない（Non-scope の申し送り）。
- S3 の注記を外す lane: 「PLU の書出しの見直しの runtime」の項目の撤去の列挙に 1 句を足す: 「runtime の状態」の注記（33 §16 の冒頭、plu-tables の冒頭、task-specs の BIZ-04・IO-04 と、本 lane が足す CMD-01・CMD-08・UI-01a・UI-01b・UI-08）を外す。足さないと、runtime の lane が列挙だけを見て進めたとき、撤去済みの物を「今の build にある」と書く注記が残る。

本 lane が触る backlog の 5 つの項目（「次の docs の lane の候補（docs の棚卸し lane 1 の残り）」「SD の一括取込みと EJ の控えの runtime（A）」「日報の紙の runtime」「単位の拡張の残りの申し送り」「PLU の書出しの見直しの runtime」）の行は、どの open の PR の差分にも無い（`1fba4d45` の時点。`gh pr diff <N>`〈#172・#188・#190・#191・#193・#194〉の `docs/backlog.md` の差分の `+`/`-` の行を同じ 5 つの項目の名で `rg -o` して何も出さない）。

### S5b D-040・D-041・D-045 の関係（backlog の (5)、直さない）

D-126 が Status に許すのは状態の語と出所だけで、決定の本文は append-only、足してよい行は `Superseded by:`・`Superseded in part by:` と、Status の行が無い D への Status の行だけ。「D-041 が D-040 の Phase B の実行の契約であること」「D-045 が D-041・D-042 の証跡の要求の裁定に効き、D-040 の clone の役割の分離を変えないこと」は、置き換わり（Superseded）でも出所でもない。D-126 が許す行は無く、残さないことが一意に決まる。file は変えず、backlog の項目から外す（S6b）。

### S7 db-design の正本の小口 3 つ（backlog の (6)）

owner の答え（repo 外の回答台帳、2026-10-23）: S7-2・S7-3 は (a)（本 lane の docs で直す。TD-393）、S7-1 は (a)（code の初期データを正本の NULL に合わせる。code と test の変更なので別の R3 の lane へ送る。TD-395）。直す前に、下の根拠を `rg` か実読で確かめ直す（Scope の冒頭の規則）。

| # | file（`eed07d37` の行） | 今の句 | 直す向き | 根拠 | 結果 |
|---|---|---|---|---|---|
| S7-2a | `docs/db-design/master-tables.md` products の「困りそうなケース」の「CSV取込み時にJANコードで複数商品がヒットする場合」（87） | 「→ グループコード商品は色を区別しない方針なので、どの商品に紐づけても在庫管理上は問題ない。ただしCSV取込みロジックで…ルールが必要（先頭ヒット or 利用者に選択させる）」 | 「色を区別しない方針」と、その帰結の「どの商品に紐づけても問題ない」「ルールが必要」を外す。同じ JAN で色・柄が違う品は別の商品で、その群は全色を独自コードにし JAN を `jan_code` に持たせる（repo 外の回答台帳 TD-334・TD-344・TD-352）。今の照合の規則は [pos-tables](../../db-design/pos-tables.md) の Z004 のマスタ照合（先頭へ紐付ける）を引き、先頭へ割り当てる限界は backlog の DATA-2、共有 JAN の行を在庫連動の外にする後の形は D-118 の Compatibility（(b) は proposed・未実装）を引く | 決定は TD-334・TD-344・TD-352・TD-296。D-118 の Compatibility の「共有 JAN の行の扱い」の句。同じ file の頭の「計数の置き場と在庫の求め直しの契約」（D-118。(b) は proposed・未実装）の共有 JAN の項目 | 直した |
| S7-2b | `docs/db-design/pos-tables.md` Z004 のマスタ照合の「ヒット複数件」（377） | 「`ORDER BY product_code ASC`で先頭に紐付け（グループコード商品は色を区別しない方針のため。常に同じ商品に紐付く決定的ルール）」 | 規則（`ORDER BY product_code ASC` の先頭へ紐付ける、決定的）は変えない。括弧の「色を区別しない方針のため」を外し、先頭へ割り当てる限界は backlog の DATA-2、共有 JAN の行の D-118 の後の形は D-118 の Compatibility を引く。D-118 の後の形を今の build の振舞いと書かない | `src-tauri/src/db/product_repo.rs` の `find_by_jan_code` は `ORDER BY product_code ASC`。DATA-2 は `docs/backlog.md` の「着手対象」にある | 直した |
| S7-3a | `docs/db-design/transaction-tables.md` §8-9 の「役割」（136） | 「PLU全商品登録後は利用頻度が極めて低い画面」 | 「利用頻度が極めて低い」を外し、使い道を S7-3c と同じ 2 つ（PLU 未登録の新商品と、SPEC-UNIT-D4 の既定 (a) の間の長さの商品）で書く | S7-3c と同じ | 直した |
| S7-3b | 同 §8-9 の「利用者ヒアリング結果（2026-03-28）」の結論（144） | 「手動販売出庫を使うのは「PLU登録が間に合わなかった新商品が売れた場合」のみ」 | ヒアリングの日付の結論として文は残し、直後に 1 文を添える: 今は [10](../../function-design/10-common-rules.md) SPEC-UNIT-D4 の既定 (a) で、長さの商品の売った分も手動販売で記録する | S7-3c と同じ | 直した |
| S7-3c | 同 `manual_sales` の設計意図の「reasonを簡素化した理由」（171） | 「利用者ヒアリングの結果、レジ外販売は実質ゼロ。使うケースは「PLU未登録新商品」がほぼ全て。'off_register'や'fabric_cut'は実態に合わないため削除」 | 当時のヒアリングと reason の値を絞った理由は出所として残し、「ほぼ全て」を今の正本に合わせる: 手動販売の使い道は PLU 未登録の新商品と、[10](../../function-design/10-common-rules.md) SPEC-UNIT-D4 の店に聞いてから決める項の既定 (a)（長さの商品は在庫連動の自動提案が `false` で、売った分を手動販売で記録する。店主の答えが無い間の形、D-119 の U5 = c）。reason の値（`plu_unregistered`・`other`）と UI-04-D3 の既定は変えない | 10 の SPEC-UNIT-D4 の「店に聞いてから決める」の項。[62](../../function-design/62-ui-manual-sale.md) UI-04-D3（reason の 2 値と既定） | 直した |
| S7-3d | `docs/SCREEN_DESIGN.md` Phase 3 の画面一覧の UI-04（449） | 「UI-04 手動販売出庫（REQ-203、PLU 未登録新商品のみ）」 | 「PLU 未登録新商品のみ」を外し、使い道を S7-3a〜S7-3c と同じ 2 つで書く（向き: 「UI-04 手動販売出庫（REQ-203。PLU 未登録の新商品と、[10](../../function-design/10-common-rules.md) SPEC-UNIT-D4 の店に聞いてから決める項の既定 (a) の間の長さの商品）」） | S7-3c と同じ。62 §62.5 の明細は長さの商品の数量を受ける（UI-04-D18） | 直した |

直さない（S7-3 の sweep）: 同じ file の「二重減算問題と対策（2026-03-28 分析・確定）」の「画面の用途限定」の説明バーの句（「PLU登録が間に合わなかった新商品のみ」）は、2026-03-28 の分析の節の当時の案の文言で、今の build の画面（`src/features/manual-sale/ManualSalePage.tsx`）にこの文言は無い（`rg -n 'PLU登録が間に合わなかった' src` が何も出さない）。画面の今の文言は [62](../../function-design/62-ui-manual-sale.md) が持ち、画面の文言の記述を洗い直すのは本 lane の範囲の外（Non-scope）。

手動販売の使い道を限る文の sweep（`1fba4d45` で `rg -n --glob '!docs/archive/**' --glob '!docs/plans/2026-10-23-docs-small-batch.md' -e '新商品のみ' -e '新商品が売れた場合' -e 'PLU ?未登録(の)?新商品' -e '間に合わなかった' -e 'がほぼ全て' -e '手動販売.{0,30}(のみ|だけ)' -e '(のみ|だけ).{0,10}手動販売' .` と `rg -n -e '用途' docs/function-design/62-ui-manual-sale.md`）: 直すのは S7-3a〜S7-3d の 4 行（transaction-tables の 136・144・171、SCREEN_DESIGN の 449。136 は sweep の語に当たらず AC-S7-1 の「利用頻度が極めて低い」で拾う）。ほかの hit は直さない: 62 の冒頭の「主用途は PLU 書出し前の新商品販売の補完」と UI-04-D3 の理由の列の「主用途 … 特に PLU 書出し前の新商品販売」は「主」「特に」で使い道を限らず、長さの商品の数量は同じ 62 の UI-04-D18 が持つ。transaction-tables の「二重減算問題と対策」の「発生条件: … 場合のみ」は二重減算が起きる条件で、使い道の制限でない。同じ節の説明バーの句は上の段。transaction-tables の `reason` の列の「PLU未登録の新商品」は reason の値の説明で、値を変えない（S7-3c）。`docs/DB_DESIGN.md` の `pos_stock_sync=0` の商品の項（手動販売と棚卸しの補正でのみ在庫が動く）は (a) と合う。ほかの hit（backlog の L3 の採否、01 の結果の表示の先例、design-system/reference の記録と mockup の提案、`docs/function-design/56-ui-daily-sales.md` の UI-09a-D12、`docs/function-design/62-ui-manual-sale.md` の UI-04-D17、`docs/decision-log.md` の D-113 の Context）は手動販売の使い道を書かず、使い道を限る文でない。

S7-1（部門 12〈宅急便〉の `code_prefix`、別の R3 の lane へ送る）: `docs/db-design/master-tables.md` の「初期データ（全21部門）」の表は NULL、`src-tauri/src/db/schema_v1.rs` の `departments` の初期データは `'TK'`。owner の答えは code を正本に合わせる形で、`schema_v1.rs` の初期データと test の変更になり、本 lane（docs だけ）の外。本 lane は master-tables の初期データの表を触らない。起票の run が `docs/backlog.md` の「着手対象」に項目「部門 12（宅急便）の初期データの `code_prefix` を正本の NULL に合わせる」を足した。

### 他の lane との所有

| file | 節・行 | 所有 |
|---|---|---|
| `docs/decision-log.md` | S1 の 15 の D の `- Status:` の行だけ | 本 lane |
| `docs/decision-log.md` | 末尾に足す新しい D、ほかの D の行 | 並走の lane（起票中の PC の外の控え (b)・棚卸し b2 ほか。本 lane は触らない） |
| `docs/backlog.md` | 「次の docs の lane の候補（docs の棚卸し lane 1 の残り）」の項目（S6b）と、起票の run が足した S7-1 の項目。S6b の末尾の 2 項の句だけ（3 つの項目の旧名の句の「〜が引く」と、「PLU の書出しの見直しの runtime」の項目に足す 1 句） | 本 lane |
| `docs/backlog.md` | ほかの項目（「MSI の製品名と upgrade code の固定」を含む） | 並走の lane（PR #172・#190、MSI の固定の lane `agent/msi-fix-1023`） |
| `docs/msi-distribution-runbook.md` | 「製品名と upgrade code を固定する。」の項の、backlog の項目の名前の 1 句だけ（S2-5） | 本 lane。PR #190 と MSI の固定の lane もこの file を触る。MSI の固定の lane が先に merge してこの句を消す・書き換えたら、本 lane は S2-5 を「残す（並走の lane が書き換えた）」にして触らない。本 lane が先なら、向こうが単段 merge で同期する |
| `docs/function-design/29-io-daily-report-parser.md`・`37-biz-daily-report-import-service.md` | S2-1・S2-2・S2-4 の句だけ | 本 lane（PR #172 も同じ file の別の行を触る。`eed07d37` の時点で #172 の差分にこれらの句は無い〈`gh pr diff 172` を句で `rg`〉） |
| `docs/db-design/pos-tables.md` | S7-2b の句だけ | 本 lane（PR #172・#188 も同じ file の別の行を触る。`d7157d0c` の時点で両 PR の差分に S7-2b の句は無い〈`gh pr diff` の pos-tables の差分を「ヒット複数件」「色を区別しない」で `rg`〉） |
| 上の表の他の file（S2〜S7 の行） | 各段の行 | 本 lane（round 1 の是正で足した `docs/SCREEN_DESIGN.md`〈S6-4。round 3 の是正で S7-3d の 449 も〉と `docs/db-design/transaction-tables.md` の 136・144〈S7-3a・S7-3b〉は、2026-10-23 の時点で open の PR〈#172・#188・#190・#191・#193・#194〉の差分の file に無い〈`gh pr diff <N> --name-only` を `rg -c 'SCREEN_DESIGN\|transaction-tables'`〉） |
| workflow の正本（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/**`・`docs/templates/**`）と `docs/Plans.md` | 全体 | 本 lane は触らない（`docs/Plans.md` は lane ごとの行を置かない、D-097） |

同じ file を触る lane は、後に merge する方が `origin/main` の単段 merge で同期する（`docs/DEV_WORKFLOW.md` Wave Operation）。

## Non-scope

- code・migration・bindings・route・画面の文言の変更。
- S7-1 の code の側の直し（`schema_v1.rs` の初期データと test。別の R3 の lane、backlog の項目）。S7-2 の照合の規則、S7-3 の reason の値。
- decision-log の Status の規則（頭の節と D-126）、S1 の 15 の外の Status、ADR の Status、決定の本文。
- S2 の 6 か所の外の名前の参照、S4 の 3 file の外の archive の link、`docs/adr/2026-09-18-stocktake-time-evidence.md` の名前の無い `[backlog](../../backlog.md)` の link（100・534 行）。
- workflow の正本と `docs/Plans.md`（`## 次の行動` の「次の docs の lane の候補」の句は closeout の PR が外す。S6b）。
- `docs/db-design/transaction-tables.md` の「二重減算問題と対策（2026-03-28 分析・確定）」の説明バーの句と、db-design が書く画面の文言を今の画面と照らし直すこと（S7 の末尾）。
- `docs/db-design/master-tables.md` の「廃番特価の対応方針（2026-03-29 利用者ヒアリングで確定）」の「指定色だけ値下げ」の句（同一 JAN の群の色を 1 つの商品で持ち、要るときだけ独自コードで別商品化する前提）。2026-03-29 のヒアリングの記録として残す（S7-2a は products の「困りそうなケース」の理由の句だけを直す）。
- `docs/backlog.md` の「MSI の製品名と upgrade code の固定」の旧名の句の「`docs/msi-distribution-runbook.md` が引く」（S2-5 の後は runbook が旧名を引かない）。項目の所有は並走の MSI の固定の lane（PR #191、`agent/msi-fix-1023`）で、その lane へ申し送る（S6b の末尾）。
- `docs/backlog.md` の他の項目にある在庫状態の label「通常」の句（記録状態 Badge の tone の項目・在庫状態表示の filter 依存不整合の項目・DSR-22 の Plan Review round 2 の記録。どれも日付つきの記録で、所有は並走の lane）と、`docs/design-system/reference/` の mockup。

## Acceptance Criteria

baseline は起点 `eed07d37` で下の command を逐語で実行した出力（round 2 の是正で足した・直した AC は `4b2fe746`、round 3 の是正で足した・直した AC は `1fba4d45` で実行。`eed07d37` から変わったのは本 packet と `docs/backlog.md` だけ）。git の差分の範囲は `origin/main...HEAD`（merge-base から HEAD までの差分。本 lane が `origin/main` を単段 merge で同期した後も merge-base が取り込んだ main になり、並走の lane が main に足した変更を数えない）。baseline の値は `eed07d37`（起票の時点の `origin/main`、`4b2fe746` での merge-base）で取ったままとする。

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0、末尾の WARN の件数が 12 以下（baseline: exit 0、「結果: WARN 12 件（ERROR なし）」）。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-23-docs-small-batch.md` が exit 0。
- AC3: `cd src-tauri && cargo run --bin generate_traceability -- --check` が exit 0（function-design を触る。REQ の参照は増減させない。61 の `UI-02-D17` の行は 61 が既に持つ REQ-201 だけを引く）と、`cd src-tauri && cargo test --test design_compliance_test` が PASS（function-design の節を足さず消さない）。手元のメモリが他の lane の build で足りないときは、hosted の Draft run の該当の job の pass で代える（PR の Validation に書く）。
- AC-S0-1: Scope の各段の表の「結果」列が全行埋まっている（読んで確かめる）。
- AC-S1-1: 次の command が何も出さない（exit 1）。baseline は 15 の D を全部出す（`D-042 D-075 D-086 D-088 D-090 D-091 D-092 D-093 D-094 D-096 D-097 D-098 D-099 D-109 D-123`）。

  ```bash
  for d in 042 075 086 088 090 091 092 093 094 096 097 098 099 109 123; do awk -v h="## D-$d" 'index($0,h)==1{f=1;next} f&&/^## D-/{exit} f&&/^- Status:/{print "D-'$d'\t" $0;exit}' docs/decision-log.md; done | rg -v '^D-[0-9]+\t- Status: accepted（[^（）]*(（[^（）]*）[^（）]*)*）$'
  ```

  上の形は括弧の外の文しか見ないので、S1-6・S1-14 で括弧の中から外す句は次の command で確かめる。何も出さない（exit 1）。baseline は `2`（D-091 と D-109 の行）。

  ```bash
  for d in 042 075 086 088 090 091 092 093 094 096 097 098 099 109 123; do awk -v h="## D-$d" 'index($0,h)==1{f=1;next} f&&/^## D-/{exit} f&&/^- Status:/{print "D-'$d'\t" $0;exit}' docs/decision-log.md; done | rg -c -e 'ボタンなしで確定' -e 'runtime lane A は 1 PR' -e '同日の owner 決定'
  ```

- AC-S1-2（決定の本文を変えていない）: 次の command が何も出さない（exit 1）。diff の頭の 2 行と Status の行を除いた変わった行を出す。

  ```bash
  git diff -U0 origin/main...HEAD -- docs/decision-log.md | rg '^[-+]' | rg -v '^(\+\+\+|---) [ab]/' | rg -v '^[-+]- Status: '
  ```

- AC-S1-3（読んで確かめる）: S1-2・S1-6・S1-9・S1-14 で外した文が、表の「外す文の置き場」に今もある。
- AC-S2-1: 次の command が何も出さない（exit 1）。baseline は 6 行（29:97・29:113・33:82・37:447・msi-distribution-runbook:31・time-evidence ADR:228）。

  ```bash
  rg -n -e '「単位の拡張」' -e '「既存の支払集計（Z002）' -e '「売上の SD からの一括取込みと EJ の控えの閲覧」' -e '「MSI 配布手順 docs 化」' -e '「実測とPOS系列の対応を取得・保存する」の入力' docs/function-design/29-io-daily-report-parser.md docs/function-design/33-biz-plu-export-service.md docs/function-design/37-biz-daily-report-import-service.md docs/msi-distribution-runbook.md docs/adr/2026-09-18-stocktake-time-evidence.md
  ```

  S2-5 を「残す（並走の lane が書き換えた）」にした場合は、msi-distribution-runbook の行が出ないことを並走の lane の merge の結果で確かめる。
- AC-S2-2（decision-log の句を変えていない）: 次の command が 5 を出す（baseline: 5）。

  ```bash
  rg -c -e '単位の拡張（backlog）' -e 'backlog の Z002 の合算の項目' -e 'backlog.md` の EJ の入力 \(5\)' -e '既存の支払集計（Z002）で異なる項目が 1 行に合算されうる' -e '日報画面の Excel 印刷・バインダー代替受入判定' docs/decision-log.md
  ```

- AC-S2-3（読んで確かめる）: S2 の直す先の項目・節が今の backlog と正本にあり、名前が一字一句合う。
- AC-S3-1: `rg -c 'runtime の状態' docs/architecture/cmd-task-specs.md docs/architecture/ui-task-specs.md` が cmd-task-specs で 2（CMD-01・CMD-08）、ui-task-specs で 3（baseline: 何も出さない、exit 1）。
- AC-S3-2（読んで確かめる）: S3 の表の「task-spec の文が D-117 で外す物」の 1 つずつについて、直した時点の `src-tauri/src`・`src` にあるかを S3 の表の後の command で `rg` で確かめ、build にある物は全部、その行の注記に symbol と一緒に挙がっている（一部だけを挙げない）。D-117 の後の形を今の build の振舞いと書いていない。
- AC-S4-1: 次の 1 本めが 1 行だけを出し、それが `docs/archive/harness-context/2026-09-30-Plans-completed.md` の S4-4 の行（DATA-2 を引く link。節は今の backlog にある）で、2 本めが `1` を出す（同じ行に閉じた項目の archive の link がある）。baseline: 1 本めは 3 行（display-fixes:327・adr-revision:94・Plans-completed:68）、2 本めは何も出さない（exit 1）。

  ```bash
  rg -n '\.\./\.\./backlog\.md#やると決めたもの順番未定' docs/archive/plans/2026-09-15-display-fixes-batch-2.md docs/archive/plans/2026-09-23-stocktake-time-evidence-adr-revision.md docs/archive/harness-context/2026-09-30-Plans-completed.md
  rg -n '\.\./\.\./backlog\.md#やると決めたもの順番未定' docs/archive/plans/2026-09-15-display-fixes-batch-2.md docs/archive/plans/2026-09-23-stocktake-time-evidence-adr-revision.md docs/archive/harness-context/2026-09-30-Plans-completed.md | rg -c '\.\./2026-10-19-backlog-closed\.md\)'
  ```

- AC-S4-2: archive の plan を変えたときは path を名指しして確かめる（`docs/DEV_WORKFLOW.md` Plan Packet Rules）。`bash scripts/doc-consistency-check.sh --target plan docs/archive/plans/2026-09-15-display-fixes-batch-2.md` と `bash scripts/doc-consistency-check.sh --target plan docs/archive/plans/2026-09-23-stocktake-time-evidence-adr-revision.md` が exit 0 で、WARN が baseline を超えない（baseline: 前者は exit 0・WARN 1〈PK6、既存〉、後者は exit 0・WARN 0）。`docs/archive/harness-context/2026-09-30-Plans-completed.md` は plan でないので AC1 の全体の検査で link の先の file の実在を見る。anchor は checker が見ないので、直した link の anchor が link の先の file の見出しから作られることを読んで確かめる。
- AC-S5-1: `rg -n 'UI-02-D17' docs/function-design/61-ui-receiving.md` が 1 行以上を出し、その行が TD-350 を引く（baseline: 何も出さない、exit 1）。
- AC-S6-1: 次の command が何も出さない（exit 1。baseline は 3 行: 01-decision-rules:136・56:26・56:375）。S6-4 は経緯の文の「通常」を残すので、この command に入れず AC-S6-2 で確かめる。

  ```bash
  rg -n -e 'canonical `通常`' -e 'CSV取込み完了 → 売上レポート' -e '防御的 if-else' docs/design-system/01-decision-rules.md docs/function-design/56-ui-daily-sales.md
  ```

- AC-S6-2: `rg -c '在庫あり' docs/SCREEN_DESIGN.md` が 1 以上（baseline: 何も出さない、exit 1）。読んで確かめる: 在庫照会の「利用者配慮」の経緯の文（2026-06-07 の L3）が「通常」のまま残り、今の中立の label を別の句で `StockStatusBadge` と一緒に書いている。

- AC-S6b-1（読んで確かめる）: 外した理由（S2 の末尾・S5b・S6b）が本 packet にあり、S7-1 の項目が backlog の「着手対象」にある。
- AC-S6b-2: `rg -c '次の docs の lane の候補' docs/backlog.md` が何も出さない（exit 1。baseline: `1`）。`docs/Plans.md` の同じ句は本 lane の AC にせず、closeout の PR が S6b の command で確かめる。
- AC-S6b-3（S6b の末尾の 2 項）: `rg -n -e '33・37 が引く' -e '29D・decision-log' docs/backlog.md` が何も出さない（exit 1。baseline は 3 行: 27・30・46）。`rg -c 'runtime の状態' docs/backlog.md` が `1`（baseline: 何も出さない、exit 1）で、その句が「PLU の書出しの見直しの runtime」の項目にある（読んで確かめる）。
- AC-S7-1: 次の command が何も出さない（exit 1。baseline は 5 行: master-tables:87・pos-tables:377・transaction-tables:136・transaction-tables:171・SCREEN_DESIGN:449）。

  ```bash
  rg -n -e '色を区別しない' -e '「PLU未登録新商品」がほぼ全て' -e '利用頻度が極めて低い' -e 'PLU 未登録新商品のみ' docs/db-design/master-tables.md docs/db-design/pos-tables.md docs/db-design/transaction-tables.md docs/SCREEN_DESIGN.md
  ```

- AC-S7-2: `rg -c 'DATA-2' docs/db-design/master-tables.md docs/db-design/pos-tables.md` が両 file で 1 以上、`rg -c 'SPEC-UNIT-D4' docs/db-design/transaction-tables.md docs/SCREEN_DESIGN.md` が両 file で 1 以上（baseline: どちらも何も出さない、exit 1）。
- AC-S7-3（読んで確かめる）: pos-tables の「ヒット複数件」の規則（`ORDER BY product_code ASC` の先頭）と、transaction-tables の `reason` の列の 2 値が変わっていない。D-118 の後の形を今の build の振舞いと書いていない。transaction-tables のヒアリングの結論（S7-3b）の直後に SPEC-UNIT-D4 の既定 (a) を引く 1 文があり、役割（S7-3a）・設計意図（S7-3c）・`docs/SCREEN_DESIGN.md` の画面一覧の UI-04 の行（S7-3d）と使い道が食い違わない。どの文も長さの商品に使う reason を決めていない。
- AC-S9-1（新しい文言）: `git diff origin/main...HEAD -- docs ':!docs/plans/**' | rg '^\+' | rg -n 'T[O]DO|T[B]D|F[I]XME|H[A]CK|未確[定]|要確[認]|要検[討]'` が何も出さない（exit 1）。文字の組の形は、本 packet の文が doc check の印の検査に当たらないようにするためで、探す語は 7 語のまま。

## Design Readiness

- 引用する設計正本（節まで）: decision-log の頭の「Status の書き方」と D-126（S1・S5b・S2 の末尾）、D-117 と [33](../../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記・`docs/architecture/biz-task-specs.md` BIZ-04・`docs/architecture/io-task-specs.md` IO-04 の注記の形と [40](../../function-design/40-cmd-product.md) の `bulk_set_plu_target` コマンド（CMD-01-D4。S3）、[10](../../function-design/10-common-rules.md) SPEC-UNIT-D8（S2-1）、[37](../../function-design/37-biz-daily-report-import-service.md) §37.9 の手順 10・§37.10・BIZ-08-D7・§37.11 と D-121（S2-3・S2-4）、[61](../../function-design/61-ui-receiving.md) §61.1・§61.10 と `docs/project-memory.md` の Store Premises（S5）、[DSR-22](../../design-system/01-decision-rules.md)・[56](../../function-design/56-ui-daily-sales.md)・`docs/SCREEN_DESIGN.md` の「主要な画面間遷移」（S6）、D-118 の Compatibility・backlog の DATA-2・[10](../../function-design/10-common-rules.md) SPEC-UNIT-D4 の店に聞いてから決める項・[62](../../function-design/62-ui-manual-sale.md) UI-04-D3（S7）。
- 必要な設計成果物: Design artifact selection の当たる行は無い（振舞い・command・DB・画面・format を変えない）。状態は existing sufficient。S5 は owner の決定（TD-350）を書き写す 1 行で、新しい設計の選択を持たない（Review Focus Q2）。
- plan にしかない durable な判断の昇格先: 本 packet が決める判断は 2 つ。(1) S1 の各行で外す文の扱い（D-126 の適用。昇格先は decision-log の Status の行そのもの）、(2) (2) の decision-log の句と (5) を直さないこと（D-126 (3) の適用。backlog の項目から外し、理由は本 packet に残る）。どちらも規則の新設でなく既存の規則の適用。
- 前提・制約と延期した gap: S7-1 を別の R3 の lane へ送る（backlog の項目）。送っても安全な理由: 本 lane は master-tables の初期データの表と code を変えず、owner の答えで向き（code を正本の NULL に合わせる）は決まっている。
- 絶対保証の自己点検: S3 の注記の「今の build は …」は `eed07d37` の code の現物（S3 の表の symbol）に基づき、実装の run が直す時点で `rg` で確かめ直す（AC-S3-2）。PLU の書出しの見直しの runtime が先に merge して symbol が消えたら、注記を足さずに「残す（runtime が merge 済み）」とする。
- 判定: ready。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| REQ / coverage の追加・変更・削除 | `cargo run --bin generate_traceability` で 90 を再生成 | REQ の参照は増減させない（S5 の行は 61 が既に持つ REQ-201 を引く）。AC3 の `--check` で確かめ、drift が出たら再生成して Scope に足す |
| function-design doc の節 | `design_compliance_test` の必須節 | 節を足さず消さない（29・33・37・56・61 の文と表の行だけ）。AC3 で確かめる |
| source / workflow doc 新設・改名・削除 | 親文書の目次・索引 | 本 packet の新設だけ（`docs/plans/` の dated packet で、索引は持たない。D-097） |

ほかの行（Tauri command、route、operator 画面）は該当なし。

## Impact Review Lenses

not applicable: 起点は現地調査・実機確認・外部 tool の振舞いでなく、repo の中の docs と code の照合（lane 1・2a の Follow-up）。S5 の決まりは owner の決定（TD-350）で、店の事実は `docs/project-memory.md` の既存の行を引くだけ。

## Boundary / Wire Contract

変えない。JSON・CSV・config・manifest・DTO・bindings・DB に触れない。

## Test Plan

Test Design Matrix は置かない（R2 で任意。各段の AC が `rg` の件数と読んで確かめる項目で直した所を確かめる）。

- targeted tests: AC3（`generate_traceability -- --check`・`design_compliance_test`）。code を変えないので Rust・frontend の test は足さない。
- negative tests: AC-S1-1・AC-S2-1・AC-S4-1・AC-S6-1・AC-S7-1・AC-S9-1（旧い表現・禁じた語が残らないこと）、AC-S1-2・AC-S2-2（decision-log の本文を変えていないこと）。
- compatibility checks: AC1・AC2・AC-S4-2（link の先の file の実在。anchor は読んで確かめる）。
- data safety checks: `git diff --name-only origin/main...HEAD`（範囲は Acceptance Criteria の頭と同じ）が docs の file だけで 20 以下（19 と本 packet）、店の運用の細部・実値・発言の原文を書いていない（S5 は TD-350 と project-memory の行を引くだけ）。
- main wiring/integration checks: AC-S2-3（直した名前の参照が今の項目・節へ着く）、AC-S6b-1（S7-1 の項目が backlog にある）。

起票の run の検証（AC1・AC2 と `bash scripts/check-workflow-git.sh`）の結果は PR の Validation の欄に書く。

## Review Focus

- S1: 各行で外す文が、出所でない（状態の語と出所の外）と言えるか。外した文の中身が「外す文の置き場」に本当にあり、今も効く中身を失っていないか（特に S1-2 の D-075 と S1-14 の D-109）。括弧の中に残す文（S1 の規則の段の「決定が出た場」）を出所と読んでよいか。
- S2: (2) の decision-log の句を直さない根拠（D-126 (3)）が成り立つか。S2-3・S2-4 の「設計は済んだ」の向きが D-121 と §37.10（BIZ-08-D7）・§37.11 の現物と合うか。
- S3: 注記の「今の build」が code の現物と合い、D-117 の後の形を実装済みと読ませないか。
- S7: S7-2 の直す向きが照合の規則を変えず、D-118 の後の形（proposed）を今の build と読ませないか。S7-3 の直す向きが reason の値を変えず、SPEC-UNIT-D4 の店に聞いてから決める項の既定と合うか。S7-1 の backlog の項目が owner の答えと範囲を正しく写すか。
- Q1（D-127）: この変更で不要になる記述・条件は無いか。例: S3 の注記は PLU の書出しの見直しの runtime の merge で消す対象になる（33 §16 の冒頭の注記と同じく、runtime の lane が外す。S6b の末尾で backlog のその項目の撤去の列挙に足す）。S6b の backlog の項目は残りが無くなれば外す。S2 で旧名を引かなくなる file の名は、backlog の旧名の句の「〜が引く」から外す（S6b の末尾）。
- Q2（Plan Review で答える）: S5 は design の段（spec-check → design → plan-draft）を経るべきか。既定: 経ない。決定の中身は TD-350 が持ち、本 lane は正本へ書き写すだけで、設計の選択（画面の検査・文言を足すか）は TD-350 が「決まりを正本に置く」と決めている。

## Implementation Results

docs の本体の直しの run（PR #192）。Scope の S1〜S7（S6b を含む）を全部直し、各段の表の「結果」列を埋めた。残した行は無い。code・`docs/Plans.md`・workflow の正本は触っていない。closure の Claude 側の P3 2 件は、S7 の末尾の sweep の記録（N1: 「ほかの hit」に 3 件を足した）と S6b の末尾の所有の確かめ（N2: 項目の数を 5 にそろえ、名を挙げた）で直した。生成物: `generate_traceability -- --check` が drift 無しで、再生成しても差分が無いので、生成物の commit は無い。

直した file（19）:

| 段 | file |
|---|---|
| S1 | `docs/decision-log.md`（15 の D の `- Status:` の行だけ） |
| S2 | `docs/function-design/29-io-daily-report-parser.md`、`docs/function-design/33-biz-plu-export-service.md`、`docs/function-design/37-biz-daily-report-import-service.md`、`docs/msi-distribution-runbook.md`、`docs/adr/2026-09-18-stocktake-time-evidence.md` |
| S3 | `docs/architecture/cmd-task-specs.md`（CMD-01・CMD-08）、`docs/architecture/ui-task-specs.md`（UI-01a・UI-01b・UI-08） |
| S4 | `docs/archive/plans/2026-09-15-display-fixes-batch-2.md`、`docs/archive/plans/2026-09-23-stocktake-time-evidence-adr-revision.md`、`docs/archive/harness-context/2026-09-30-Plans-completed.md` |
| S5 | `docs/function-design/61-ui-receiving.md`（§61.1 の UI-02-D17、§61.10） |
| S6・S6b | `docs/design-system/01-decision-rules.md`、`docs/function-design/56-ui-daily-sales.md`、`docs/SCREEN_DESIGN.md`（在庫照会の利用者配慮）、`docs/backlog.md` |
| S7 | `docs/db-design/master-tables.md`、`docs/db-design/pos-tables.md`、`docs/db-design/transaction-tables.md`、`docs/SCREEN_DESIGN.md`（画面一覧の UI-04） |

AC（本 run で逐語で実行した結果）:

| AC | 結果 |
|---|---|
| AC1 | exit 0、「結果: WARN 12 件（ERROR なし）」（baseline と同じ） |
| AC2 | exit 0 |
| AC3 | `generate_traceability -- --check` exit 0（ERROR 0 / WARN 0）、`design_compliance_test` PASS |
| AC-S0-1 | 各段の表の「結果」列が全行埋まった（読んで確かめた） |
| AC-S1-1 | 1 本め・2 本めとも何も出さない（exit 1） |
| AC-S1-2 | 何も出さない（exit 1） |
| AC-S1-3 | S1-2・S1-6・S1-9・S1-14 で外した文が「外す文の置き場」にある（読んで確かめた） |
| AC-S2-1 | 何も出さない（exit 1） |
| AC-S2-2 | `5` |
| AC-S2-3 | 直す先の項目・節が今の backlog と正本にあり、名が一字一句合う（読んで確かめた） |
| AC-S3-1 | cmd-task-specs で 2、ui-task-specs で 3 |
| AC-S3-2 | S3 の表の後の command を逐語で実行し直し、出力は表の後の記録と同じ。外す物は全部、symbol と一緒に注記に挙げた（読んで確かめた） |
| AC-S4-1 | 1 本めは `docs/archive/harness-context/2026-09-30-Plans-completed.md` の S4-4 の行だけ、2 本めは `1` |
| AC-S4-2 | display-fixes-batch-2 は exit 0・WARN 1、adr-revision は exit 0・WARN 0。直した link の anchor は閉じた項目の archive の「やると決めたもの」の節の見出しから作られる（読んで確かめた） |
| AC-S5-1 | UI-02-D17 の行が TD-350 を引く |
| AC-S6-1 | 何も出さない（exit 1） |
| AC-S6-2 | `1`。経緯の文の「通常」を残し、中立の label を `StockStatusBadge` と一緒に別の句で書いた（読んで確かめた） |
| AC-S6b-1 | 外した理由が S2 の末尾・S5b・S6b にあり、S7-1 の項目が backlog の「着手対象」にある（読んで確かめた） |
| AC-S6b-2 | 何も出さない（exit 1） |
| AC-S6b-3 | 1 本めは何も出さない（exit 1）、`runtime の状態` は `1` で「PLU の書出しの見直しの runtime」の項目にある |
| AC-S7-1 | 何も出さない（exit 1） |
| AC-S7-2 | DATA-2 は master-tables・pos-tables で各 1、SPEC-UNIT-D4 は transaction-tables で 3・SCREEN_DESIGN で 1 |
| AC-S7-3 | 照合の規則（`ORDER BY product_code ASC` の先頭）と reason の 2 値は変わらず、D-118 の後の形を今の build と書いていない。使い道は 4 か所で同じ 2 つで、長さの商品に使う reason を決めていない（読んで確かめた） |
| AC-S9-1 | 何も出さない（exit 1） |
| data safety | `git diff --name-only origin/main...HEAD` は docs の 20 file（19 と本 packet）。店の運用の細部・実値・発言の原文を書いていない |

closeout（2026-10-24）:

- 結果: decision-log の 15 の D の Status を状態の語と出所だけにし、設計正本・runbook・ADR が backlog を名で引く所を今の項目と節の ID に、task-specs の 5 か所に D-117 の後の形と今の build を書き分ける注記を、archive の 3 file の link を項目の今の置き場に、入庫の正本に取り寄せの品を入れない決まり（TD-350）を、DSR-22・56・`docs/SCREEN_DESIGN.md` と db-design の文を今の code・決定に合わせた。code・migration・bindings・画面の振舞いは変えない。部門 12 の `code_prefix`（S7-1）は別の R3 の lane（PR #195）で code の側を直した。
- review・CI・merge: Plan Review は round 1〜3（天井）と closure（fresh Opus 5.5 と Codex の両者 approve）。Final Review の broad は Codex と fresh Opus 5.5 の両者 approve（P3 だけ）で、重なる 1 件を除く P3 5 件を Coordinator が直し、closure（fresh Opus 5.5）が approve した。hosted CI は緑で、helper 経由で squash merge した。PR: [#192](https://github.com/kosei-w90607/inventory-system-desktop/pull/192)
- closeout で直したもの: `docs/Plans.md` の `## 次の行動` の「次の docs の lane の候補」を名で引く句（S6b）、closure の P3 N1 の S4-3 の「結果」列（下の Review Response）、61 の変更履歴の本 packet への link の path。
- 後続: backlog の「MSI の製品名と upgrade code の固定」の旧名の句は MSI の lane（PR #191）へ申し送り済み（Non-scope）。

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

round 1（`217c8906`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 6）。Codex 側は未実施で round 2 で足す（§3.3）。Coordinator は全 7 件を採用した。是正は起草役の `cc3d7a7d`。

- P2: S3 の task-spec の注記は、その task-spec が D-117 で外すと書く物を全部挙げ、build に残る物の symbol を付ける（一部だけ挙げると、挙げなかった物を撤去済みと読ませる）。AC-S3-2 を外す物ごとの `rg` に。
- P3: S1 の括弧の中に残す物と外す物の基準を 1 段で書き S1-6 を同じ基準に、S4 の link の親と anchor、S7-3 を同じ節の 3 行に、`docs/SCREEN_DESIGN.md` の在庫状態の label（S6-4）、`docs/Plans.md` の名前の引用は closeout で外す、S7-2a の列の名。

round 2（`4b2fe746`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 3 / P3 3）。Codex 側は round 3 で足す。Coordinator が P2 3 件を現物で確かめ（`docs/backlog.md` の 13 行目に `#### やると決めたもの（順番未定）` が今もある、AC の差分の範囲が `eed07d37...HEAD`、37 の BIZ-08-D7 が順と TX を既に決めている）、全 6 件を採用した。是正は起草役の `85ee4756`。

- P2: S4 の前提を事実に合わせた（節は残り、項目が archive の同じ名の節へ移った。DATA-2 だけ今の backlog）。AC と data safety の差分の範囲を `origin/main...HEAD` に。S2-3 を S2-4 と同じ形（37 §37.10・BIZ-08-D7 が決めた）に。
- P3: S1-14・S1-15 の句を出所として残すと明記、S3 に CMD-01 の行（S3-5）、master-tables の 128 行目はヒアリングの記録として Non-scope に、S7-3c を運用の語に。

round 3（天井、`1fba4d45`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 5）、Codex = reject（P1 0 / P2 1 / P3 1）。Coordinator が Codex の P2 を現物で確かめ（`docs/SCREEN_DESIGN.md` の Phase 3 の画面一覧の UI-04 の行が「PLU 未登録新商品のみ」）、全 7 件を採用した。disposition: round 4 は回さず、一括の是正（起草役の `a4d1bd8b`）の後に closure へ出す。

- Codex 側: S7-3d（SCREEN_DESIGN の UI-04 の使い道を SPEC-UNIT-D4 の既定 (a) を引く形に）と使い道を限る文の sweep。Owner Effort Budget を decision point の単位で（消費 4・残り 2・予備 0）。
- Claude 側: S7-3c の reason の句を外す、AC-S1-1 に括弧の中の句の検査、backlog の旧名の「〜が引く」の句（本 lane の所有、61 は MSI の lane へ申し送り）、SCREEN_DESIGN の経緯の文は残して今の label を書き添える、PLU の書出しの見直しの runtime の撤去の列挙に注記の撤去を 1 句。

closure（`e5b6a111`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 2）、Codex = approve（P1 0 / P2 0 / P3 0）。round 3 の 7 件は両方とも全部 closed、C1 の固定の確認も両方で済み（直しを戻すと AC-S7-1 が `docs/SCREEN_DESIGN.md` の行を拾う）。Claude 側の新しい P3: N1（C1 の sweep の記録の「ほかの hit」に、使い道を限らない hit 3 件が仕分けされていない）、N2（O3 の所有の確かめの証跡で「4 つの項目」と「5 つの項目の名」の数が合わず名の一覧が無い）。どちらも packet の証跡の書き方で、implementing の本体の run で直す。

Final Review broad（`8fc1ca17`）: Codex = approve（P1 0 / P2 0 / P3 2）、Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 4）。backlog の 27 行目の指摘は両者が重なり、Coordinator は 5 件を全部採用し、Coordinator が直した（本 commit）。Scope の各段の表の余分な区切り（43 行、AC-S0-1 の「結果」列が空に見えた）、backlog の 27 行目の S2-4 で済んだ句、ui-task-specs の UI-08 の注記に最終照合日時と UI-08-D5 の SD の設定書込みの案内もまだ無いこと（`src/features/plu-export/PluExportPage.tsx` に「設定書込み」「照合」の文字は無く、日時は「最終読込み日時」）、61 と backlog の 111 行目の packet の引用を link に（closeout の link 直しに掛ける）、Plans-completed の 54 行目の重複の link を外す。

Final Review の closure と closeout（2026-10-24）: closure（fresh Opus 5.5）= approve で、broad の指摘は全部 closed。新しい P3 N1（S4-3 の「結果」列の「閉じた packet への link を足し」は、Coordinator が broad の是正で Plans-completed の重複の link を外した後の現物と合わない）は、closeout で S4-3 の「結果」列のそのセルだけを「backlog への link を外した。閉じた packet は同じ entry の後ろの「archiveのPlan Packet」の link が既に引く」に直した（記録の訂正）。hosted CI は緑で、helper 経由で squash merge した（2026-10-24）。packet の本文（Scope の直し方・AC・Design Readiness）は当時の契約の記録として書き換えない。
