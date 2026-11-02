# Plan Packet: docs の小口 3（導入の前の関門の表に取りこぼしを載せ、古い記述を直す）

wave に属さない単独の lane。docs だけ（`docs/Plans.md`・`docs/backlog.md`・`docs/decision-log.md`・`docs/UI_TECH_STACK.md`・`docs/function-design/10-common-rules.md`・`docs/project-memory.md` と本 packet）。code・migration・bindings・画面の振舞い・helper・classifier・CI の設定は変えない。

出所: owner の決定（2026-10-27、repo 外の回答台帳 TD-460）: 取りこぼしの洗い出し（repo の外）で見つけた「導入に効くのに予定の無いもの」G1〜G12 を `docs/Plans.md` の「導入の前の関門」に candidate / precondition-dependent で載せ、古い記述も直す。owner が決める G1・G2・G7・G8 は同じ日に答えが出た（TD-461〜TD-464。下の Scope の S1）。G9（訪店そのもの）は repo の外の段取りなので載せない。

規模と手法: docs だけの lane なので、Coordinator の発注どおり変更そのものも plan-first の commit に入れた（packet と変更が同じ commit。Plan Review は packet と変更の両方を読む）。

## Workflow State

- Phase: implementing
- Risk: R2
- Plan Commit: 7895aca82b11016d6a22c69ef30257e65d640db3
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票と docs の変更を同じ commit に入れる。発注 609）
- Plan Reviewer: fresh Opus 5.5（fork でない）+ Codex。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Reviewer: Claude 側 1 本 + Codex。座組表どおり（R2 で design lane でないので Claude 側は fresh Opus 5.5。closure の Claude 側の既定は Fable の枠で、当面 Fable 5〈TD-410〉）
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/docs-small-batch-3

manual なし: 製品の runtime・画面・配布物を変えない docs の変更で、Windows native で確かめる物が無い。Final Review Minimum 1: R2 で、classifier が `workflow=true` を出さない（下の Test Plan の classifier の出力）。

遷移の記録:

1. kickoff → spec-check（2026-10-27、起草役）: 範囲を発注 609 の G1〜G12（G9 を除く）と backlog の 4 群（下の S2〜S4）にし、Risk を R2 と記録した（下の Risk）。
2. spec-check → plan-draft（2026-10-27、起草役、同じ commit）: 表の唯一の skip。本 lane は関門の表と backlog の項目を owner の決定と既存の正本に合わせて並べ直すだけで、新しい設計の選択をしない（下の Design Readiness）。
3. plan-draft → plan-gate（2026-10-27、Coordinator）: packet は commit 済み（`2c0582a2`）。R2 なので Matrix は任意で、作らない理由は前文。Final Reviewer の Claude 側は座組表の R2 の行（fresh Opus 5.5）に従う起草役の判断を受ける。Plan Review へ出す（fresh Opus 5.5 + Codex）。
4. plan-gate（round 2 の是正、2026-10-27、Coordinator、本 commit）: round 2 の是正（Review Response 参照）。是正で `docs/project-memory.md` の 1 行を Scope に足した（S4）。
5. plan-gate → plan-approved（2026-10-27、Coordinator、本 commit）: 条件は `docs/DEV_WORKFLOW.md` の遷移表の「Writer でない独立の Plan Reviewer が P1 / P2 = 0」。round 2 の是正の版（`d3afd7a7`）の plan closure は fresh Opus 5.5 が approve（P3 2）、Codex（発注 629）が approve（P3 0）。closure の後に `origin/main`（`a2365759`、#193 の merge を含む）を単段で同期し（`1e3225ee`。`docs/Plans.md` の関門 2・3 の行の衝突は両方の変更を残して解いた）、P3 2 件を plan-gate のまま直した版を Plan Commit にした（`7895aca8`。下の Review Response）。
6. plan-approved → implementing（2026-10-27、Coordinator、本 commit）: docs だけの lane で、変更は plan-first の commit に入っている。Final Review に出す前に同じ commit で implementing にした（helper は implementing の head でしか broad の record を受けない）。

## Owner Effort Budget

- 介入回数上限: 8（既定 6 から上げた。理由: 起票の前に owner が答えた 5 つの decision point〈TD-460 の起票、TD-461〜TD-464 の G1・G2・G7・G8〉が lane の中身そのもので、1 回の問い合わせで得た。Ready と merge の 2 回と予備を足すと 6 を超える）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 8 | 5（TD-460・TD-461・TD-462・TD-463・TD-464） | 2（Ready 1・merge 1） | 1（Ready の後の base 同期で再 Ready を諮る 1） | 8 = 5 + 2 + 1 |

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R2

Reason:
docs だけで runtime の契約（DB・command・DTO・画面・format・merge gate）を変えない。関門の表と backlog の項目の並べ直しは、どの作業を導入の前に置くかの計画の文で、R2 の「docs change that affects maintainability but not runtime contracts」に当たる。`docs/function-design/10-common-rules.md` の SPEC-UNIT-D2 と `docs/UI_TECH_STACK.md` の UI-USW-D4 は句を今の事実（答えの有無・backlog の行き先）に合わせるだけで、契約の値（1 cm まで受ける、native close は保証しない）を変えない。R0 にしない理由: 関門の区分と項目は owner の決定を写す意味の変更で、「non-semantic doc cleanup」でない。

required gate の green / red: この変更で変わる required gate は無い見込み（docs の差分で、doc check の ERROR 0 を AC1・AC2 で確かめる。classifier は `traceability=true` を出し CI は traceability の検査を回すが、手元の実行で OK。下の Test Plan）。

## Goal

Goal Invariant:

### 最小完了条件

- `docs/Plans.md` の「導入の前の関門」を読んだ owner と Coordinator が、G1〜G12（G9 を除く）の行き先を区分（confirmed / candidate / precondition-dependent）と backlog の項目名で辿れ、各項目の中身を backlog の名で引ける（関門の文は「中身は backlog の〈名〉」の形）。
- owner の決定 G7（振替を導入の前に、confirmed）・G8（導入の時期）・G1（C1 の後に v1.0 の扱いを決める）が、それぞれ関門の confirmed の 9・節の冒頭・precondition-dependent に載る。G2（PLU をレジへ書く時期）は、Coordinator が確かめた事実（PLU を書く前の導入では Z004 の自動在庫連動はほとんど動かない）を名指し、owner が C3 の後に A / B / C から選ぶ形で candidate に載る（TD-462）。
- 古い記述（Plans の「Q-055 の後」、Plans の D-111 の未決 A の「owner の判断待ち」、Plans の「v1.0 の自動在庫連動に残る条件は b2」だけの文、backlog の振替と TD-225 の親の下の子、backlog の「未回答の問い」と書く Q-093・Q-094・Q-096・Q-098、関門 3 の「間に合わないときの方針は未回答」と USB の任意の並び）が置き換わる。

### 失敗定義

- 関門の文に中身を継ぎ足して長くする（中身は backlog の項目に置く）。
- owner が決めていない項目を confirmed に置く。G2 の事実を件数の実値で書く、または D-070 の本文（owner の決定の前）を書き換える。
- 店の運用の細部・実値・発言の原文を書く、repo 外の報告の path を書く。
- 並走の lane の行（関門 2・5、`### 次の行動` の並びの文）の中身を変える。

### 非目的

- 関門の各項目の lane の起票・設計・実装（振替・`register_processed`・native close・PLU の時期の判断ほか）。
- 関門の区分を増やすこと（今の 3 区分と「導入の後でよいと確かめたもの」のまま）。
- decision-log の新しい番号（使わない。D-111 に答えの行を 1 行足すだけ）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

not applicable: operator の操作・data / command の契約・業務の状態遷移を決めず、workflow の手順も変えない（関門の表と backlog の項目の整理）。製品の目的との関係: 本 lane は関門の表に載せるだけで、各項目の中身（決定と実装）は後続の lane。本 lane の完了は「関門の表で導入の前に要るものを辿れる」であって、「導入できる」ではない。

## Scope

新しく書く引用は D-125・`docs/DOC_STYLE_GUIDE.md` §1 の形（path と symbol・節 ID・項目名。行番号で引かない）。owner の決定は「repo 外の回答台帳 TD-nnn」で引く。新しい文言に作業の印の語（AC-S9-1 の 7 語）を使わない。

### S1 関門に載せる要素の行き先

区分の決め方: owner が決めたもの（TD-463）は confirmed。owner が「X の後に決める」と答えたもの、または前提（他の lane・実機の確かめ）で形が決まるものは precondition-dependent。ただし owner の選択肢に「関門に足す」を含むもの（G2）は、X の後に決めるものでも candidate。owner が v1.0 の前と答えたが関門を決めた TD-345 より前の答えのもの、正本が go-live の前と書くが関門に足すかは owner が選ぶもの、AI の調べから出たものは candidate（AGENTS.md の「AI の review から出た要件は owner の確認の前に Human Gate の必須にしない」）。

| 要素 | 関門の区分（理由） | backlog の項目（中身の置き場） | 結果 |
|---|---|---|---|
| G1 共有 JAN の色違いの群と JAN の無い品の v1.0 の扱い | precondition-dependent（C1 の実機の後に owner が決める、TD-461） | 「JAN の無い商品の店内コードと色ごとのバーコードの表」に今の決定（ADR SPEC-STK-CNT-D4、TD-296）・TD-363・TD-453・TD-461・C1（TD-454）を足した。DATA-2 にその項目への指しを足した | 直した |
| G2 PLU をレジへ書く時期と D-070 の「初日から自動在庫連動」 | candidate（owner が C3 の後に A = PLU を書いてから導入〈関門に足す〉/ B = 導入を先にし PLU の書込みに期限 / C = 導入を先にし初期在庫の起点を PLU を書く日の計数にそろえる、から選ぶ。TD-462。起票の後の Coordinator の調べ〈2026-10-27〉で読みが事実と分かり、前提で形が決まる precondition-dependent の書き方が事実と食い違うので candidate に移した） | 新しい項目「PLU をレジへ書く時期と D-070 の自動在庫連動」。事実（BIZ-03 は Z004 の行を JAN だけで引き当て、部門キーの売上は Z004 に載らず Z005 だけ、店は全部の商品を部門キーで売る〈PLU-24、2026-07-23〉、今のレジの登録で引き当たる在庫 Excel の JAN はごく一部）を名指し、D-070 と D-117 の Why がこの事実と食い違うことを書いた（D-070 の本文は変えない）。事実の段落の正本はこの項目だけで、Plans の関門の子は 1〜2 文とこの項目の名、Plans の `## 次の行動` の D-070 の段落は関門への指し 1 文。「PLU の部門リンクの値と 21 個目の部門」の Q-055 を C3 への置換え（TD-296・TD-459）に直した | 直した |
| G3 Z004 の経路の受入台本の第 2 版と go-live の検証の流れ | precondition-dependent（台本の形は b2 と G2 で決まる） | 「旧「次の行動」の未完了項目」の節の「⑤ go-live 検証 flow」を、着手対象の項目「go-live の検証の流れ（受入台本の第 2 版）」へ置き換えた（D-070 の ⑤ と Impact、ACCEPTANCE_SCENARIO の第 1 版の対象外、project-memory の 1 つの go-live の検証の流れ）。空になった節の見出しを外した | 直した |
| G4 日報・月次の画面の導入前受入（UI-09a-D13・UI-09b-D9） | candidate（正本は go-live の前と書くが、関門に足すかと G3 に束ねるかは owner が選ぶ） | 既存の「日報画面の Excel 印刷・バインダー代替受入判定」に UI-09b-D9・関門の candidate・G3 に束ねる候補を足した（項目名は `docs/project-memory.md` が引くので変えない） | 直した |
| G5 デザインの刷新を v1.0 の前に | candidate（owner は TD-296 で v1.0 の前と決めたが TD-345 より前の答え） | 新しい項目「デザインの刷新（v1.0 の前）」で、書体・棚卸し画面 D1・card の面色・44px・「商品と在庫」の統合 runtime と、刷新の入力の項目を名で引く。Plans の `## 次の行動` の TD-296 の句にこの項目への指しを足した。「棚卸し画面 D1 の実装」の「棚卸しの runtime (b) と組にする」を b1 の起票の決定（TD-377〜TD-379 の (i)）に置き換えた | 直した |
| G6 返品・交換の `register_processed` の既定 | candidate（b1 の起票で b1 の外の小さな lane と決めたが、関門に足すかは owner が選ぶ） | 新しい項目「返品・交換の `register_processed` の既定と差額だけの戻しの交換」。棚卸しの runtime の親の下の TD-225 の子を置き換え、b1 の起票で確かめた関係（既定 true の理由は PLU で売る在庫連動の商品の返品にだけ成り立つ）を写した | 直した |
| G7 在庫の振替 | confirmed の 9（owner の決定 TD-463） | 新しい項目「在庫の振替（詰め替え・切り分け）」。棚卸しの runtime の親の下の振替の子を置き換え、表の作り直しが要ること・TD-378 を TD-463 で覆ったこと・店主の答え（TD-358）を書いた。lane の起票は本 lane の外 | 直した |
| G8 導入の時期 | 節の冒頭に 1 句（TD-464） | — | 直した |
| G10 関門 3 の残り | 関門 3 の文の中（confirmed のまま） | 「間に合わないときの方針は未回答」を、USB は任意（TD-381）と runtime の優先を下げた（TD-401）の後に置き、残りに PC の中の保持日数と PC の日付での掃除（71 MNT-01-D3。backlog の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の「導入の前の関門」の段）を名指した | 直した |
| G11 取込み済みを `XZ_BKUP` へ移す (b) の lane | candidate（owner は TD-296 の 11a で v1.0 の前と答えたが TD-345 より前の答え） | 既存の「取込み済みの Z・EJ を `XZ_BKUP` へ移す操作（(b) の lane）」に TD-296 の 11a と関門の candidate を足した | 直した |
| G12 初期投入の CSV 作り・照合に Excel のある Windows と店の値札 | candidate の初期投入の手順の文の中 | 既存の「初期投入の手順」に 1 句（手順書の流れ 1、流れ 2 の手順 6、TD-437） | 直した |
| G9 訪店そのもの | 載せない（repo の外の段取り） | — | 載せない |

発注の後の Coordinator の追加の事実（2026-10-27）で、G2 を precondition-dependent から candidate に移した。関門の candidate と precondition-dependent は、項目が増えて 1 つの箇条に収まらないので、confirmed と同じく子の箇条に分けた（置き換え。各子は「中身は backlog の〈名〉」の形）。candidate の「済んだもの」（操作ログの保持・installer）は文を変えず子の 1 つにした。

### S2 backlog に足す・直す（関門には載せない）

| 要素 | 置き場 | 結果 |
|---|---|---|
| native の window の close（×・Alt+F4）を取込み・取消・未保存の 3 つの止めで横断して扱う（UI-USW-D4 の「別 backlog」の行き先、TD-458） | 新しい項目「native の window の close で取込み・取消・未保存の 3 つの止めを横断して扱う」。止め方の候補と失敗の型（`core:window:allow-destroy`、listener 1 つと共有の busy、StrictMode の unlisten、閉じられなくなる窓）を「設計の種で、決定ではない」として 1 段落。`docs/UI_TECH_STACK.md` UI-USW-D4 の「別 backlog」をこの項目の名に置き換えた | 直した |
| 初期投入の手順の J2 が b のときの「後で画面から登録する」手順が手順書に無い（PR #196 の相談役の報告） | 「初期投入の手順」に 1 句 | 直した |
| 検査の flake 2 つ（PK1 の 1 回の ERROR、`biz_mnt_direct_db_io_reexport_allowlist` の 1 回の FAILED） | workflow / test / lint / docs の節に新しい項目「検査の flake 2 つ」。原因は未確認と書き、workflow の lane で他と並走させない | 直した |

### S3 たぶん効くもの（関門に載せず、既存の項目の文を直すか 1 項目にまとめる）

- 既存の項目の文を直した: Q-093（「一括価格改定の運用支援 3 点」）・Q-094（「JAN の別名の runtime」の 2026-10-20 の点検）・Q-096（「商品の品番の検索」）・Q-098（「原価の lane（C）」の 2026-10-20 の決定）の「未回答の問い」を「問い Q-0nn。2026-10-21 に店主の答えがあり、判断待ち」に。Q-098 は同じ句が [10](../function-design/10-common-rules.md) SPEC-UNIT-D2 にもあるので同じ形に直し、「答えが出るまでは 1 cm まで受ける」を「決めるまでは 1 cm まで受ける」にした（受ける値は変えない）。
- 1 項目にまとめた: 新しい項目「導入に効くかもしれないもの（関門に載せない）」が、(1) 上の 4 つの答えを受けた判断、(2) 評価方法の届出（Q-099、答えは「分からない」）、(3) Q-103、(4) 更新・migration の前の保全（D-114 の Revisit）、(5) Z004 の小数の数量（P）、(6) 注文先の軸（Q-095）、(7) 店が導入の後も紙で要るもの（TRACE-D8）、(8) 入庫・返品を届く前に入れるか、を名で引く。中身は各項目に残す。

### S4 古い記述

- `docs/Plans.md` の precondition-dependent の「PLU をレジへ書くのは … Q-055 の後」: Q-055 が C3 に置き換わったこと（TD-296・TD-459）と G2 の事実を読み込み、candidate の子へ移した（S1 の G2）。
- `docs/Plans.md` の `## 次の行動` の D-070 の段落の「v1.0 の自動在庫連動に残る条件は … b2」: 「アプリの側の条件は b2」とし、関門への指しを 1 文書き加えた（PLU を書く前の導入では Z004 の自動在庫連動はほとんど動かないので、PLU をレジへ書く時期は関門の candidate で owner が決める。事実の段落は backlog の G2 の項目）。D-070 の本文は owner の決定の前なので変えない。
- `docs/Plans.md` の `## 製品の未決判断` の D-111 の未決 A の「owner の判断待ち」: 消さないと決めた（TD-296）に置き換えた。`docs/decision-log.md` の D-111 の末尾に「未決 A の答え」の行を 1 行足した（D-114 の未決 B の行と同じ並び。新しい決定の番号は使わない）。`docs/backlog.md` の「日報取込み標準手順の残設計」の「ここに残るのは写しの保持期間（D-111 の未決 A）」から保持期間を外し、決めたことを括弧に書いた。
- `docs/backlog.md` の「棚卸しの数の置き場と在庫数の求め直しの runtime」の子の日次の EJ 取込みへの申し送りの「同じ申し送りの (4) は下の子」: (4) を移した新しい項目の名に直した。
- `docs/project-memory.md` の値下げ品の売り方の行（round 2 の是正、Review Response 参照）: 問いが「PLU をレジに書いた後」の売り方だった（店主の答え、2026-09-29）ことを行の頭に書き、今は全部の商品を部門キーで売ることを添えた。G2 の「店は全部の商品を部門キーで売る」と食い違って読めないようにする。

### この変更で不要になる記述・条件（D-127）

- backlog の「棚卸しの数の置き場と在庫数の求め直しの runtime」の下の振替と TD-225 の 2 つの子（親の lane の範囲でなくなり、独立の項目へ置き換えた）。
- backlog の `## 旧「次の行動」の未完了項目` の節（唯一の行を着手対象の項目へ移し、空になった）。
- Plans の「Q-055 の後」の条件（Q-055 が C3 に置き換わった）と、D-111 の未決 A の「owner の判断待ち」。
- 関門 3 の「間に合わないときの方針は未回答」は残した（USB の任意と runtime の優先は答えだが、間に合わないときに導入を待つかの答えではない）。並びだけを今の事実の後に置いた。
- 「棚卸し画面 D1 の実装」の「棚卸しの runtime (b) と組にする」（b1 の起票の決定で置き換わった）。

### 他の lane との所有

並走の Draft の PR（起点 `3573dc67` の時点の open: #172・#190・#193・#194・#197・#201・#204・#209・#210・#211・#212）の変更 file（`gh pr diff <N> --name-only`）と本 lane の予定 path を `comm -12` で突き合わせた。

| 重なる file | 重なる PR | 本 lane の行と並走の hunk |
|---|---|---|
| `docs/Plans.md` | #190・#193・#201 | #190 は関門 3 の同じ行を変える（行の末尾に (a) の packet の指しを足す）。#193 は関門 2 の行（本 lane の関門 3 の行の隣）と `## 次の行動` の並びの文を変える。#201 は関門 5 の行の末尾に「進行中」を足す（本 lane の行と 1 行離れる）。関門 2・3 の行は後に merge する方で行の衝突になる見込みで、解き方は両方の変更を残す（本 lane は関門 3 の句の並びと残りの名指し、#190 は (a) の packet の指し） |
| `docs/backlog.md` | #172・#190・#201・#211・#212（closeout で #209・#210 も） | #172 は「日報取込み標準手順の残設計」の同じ行の末尾に句を足す（本 lane はその行の写しの保持期間〈D-111 の A〉の句を直す）ので行の衝突になる見込み。#212 は「新規登録で、入れた JAN が既に他の商品にあるとき独自コードで登録できる道」の子を足し、その項目の直前の「JAN の無い商品の店内コードと色ごとのバーコードの表」の行を本 lane が直すので、隣の行の衝突になる。ほかは別の項目（#190 は backup の項目と PC の外の控えの runtime、#201 は開発用の bin、#211 は日報の紙の runtime と i64 の項目）。#209・#210 の closeout は各自の項目（商品 CSV のプレビュー、棚卸しの runtime b1 の L3 の所感）を触る見込みで、本 lane の新しい項目は「商品 CSV のプレビューで黙って入る形の行を止める」の直後に置いたので、#209 の closeout とは隣になりうる |
| `docs/decision-log.md` | #194・#201・#209・#211・#212 | どの PR も末尾に新しい D を足す。本 lane は D-111 の中に 1 行足すだけで、離れている |
| `docs/UI_TECH_STACK.md` | #197・#212 | #197 は UI-USW-D1 の行と変更履歴の表、#212 は頭の方の表。本 lane は UI-USW-D4 の 1 行で、#197 の UI-USW-D1 の行とは 2 行離れる。変更履歴の表に行を足さない（#197 が同じ表に足すので） |
| `docs/project-memory.md` | #172 | #172 の hunk は 37・97・194 行の周り。本 lane は値下げ品の売り方の 1 行（157 行の周り）で、離れている |
| `docs/function-design/10-common-rules.md` | #193・#211 | #193・#211 の hunk は SPEC-UNIT-D2 の段落より後の節と変更履歴の表。本 lane は SPEC-UNIT-D2 の段落の 1 句で、変更履歴の表に行を足さない |

衝突の確かめ: 各 PR の head を起点 `3573dc67` と本 lane の head に `git merge-tree --write-tree --name-only` で当てて比べた。本 lane で新しく衝突になるのは #172（`docs/backlog.md`）・#193（`docs/Plans.md`）・#212（`docs/backlog.md`）。#190（`docs/Plans.md`・`docs/backlog.md`）・#194・#211（`docs/decision-log.md`）は起点に当てても衝突し、本 lane で増えない。ほかの PR は本 lane で衝突しない。

merge の順: 本 lane（docs だけ、R2）を先に merge する想定。後に merge する #172・#190・#193・#212 が `origin/main` の単段 merge で同期するときに、上の行の衝突を両方の変更を残して解く（`docs/DEV_WORKFLOW.md` Wave Operation）。本 lane が後になれば本 lane が同じ解き方で同期する。`docs/Plans.md` に lane の行を足さない（D-097）。

## Non-scope

- 関門の各項目の lane の起票（振替・`register_processed`・native close・PLU の時期・go-live の検証・デザインの刷新）と、G2 の owner の選択（C3 の後）。D-070 と D-117 の Why の書き直し（owner の決定の後）。
- 訪店の段取り（G9）と、C1・C3 の中身（repo の外）。
- `docs/decision-log.md` の D-070・D-118・D-122・D-128・D-138 の本文（決定の記録で、本 lane は書き換えない。D-128 (3) の「未回答の問い Q-098」は決定の時点の記録として残す）。
- [37](../function-design/37-biz-daily-report-import-service.md) の Non-scope の「写しの保持期間・削除と、PC の外への backup（D-111 の未決 A・B）」（BIZ-08 の範囲外の列挙で、消さない決定の後も削除の機能は範囲外のまま正しい）。
- 56 UI-09a-D19 の紙の切替えの順（D13 の受入 → 運用開始 → (2) の確かめ → 切替え）。D19 は変えない。G8 は導入の日だけを決め（関門が全部済んだ後の月の初め、TD-464）、紙の切替えの日は導入の日と別で D19 が決めると Plans に書いた（Plan Review round 1 の #1、round 2 の Opus #2）。
- code・migration・bindings・route・画面の文言。

## Acceptance Criteria

baseline は起点 `3573dc67` で下の command を実行した出力（起票の run で実行し、PR の Validation の欄に写す）。git の差分の範囲は `3573dc67...HEAD`。

- AC1: `bash scripts/doc-consistency-check.sh` が ERROR 0（WARN の件数は PR の Validation に書く）。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-27-docs-small-batch-3.md` が ERROR 0。
- AC3: `bash scripts/check-workflow-git.sh` が exit 0。
- AC-S1-1（読んで確かめる）: `docs/Plans.md` の「導入の前の関門」で、S1 の表の各要素が表の区分にあり、各子の箇条が「backlog の「〈名〉」」で終わるか backlog の名を含み、その名の項目が `docs/backlog.md` にある（`rg -n -F '<名>' docs/backlog.md` が 1 行以上）。
- AC-S1-2: `rg -n -F 'Q-055 の後' docs/Plans.md`・`rg -n -F 'owner の判断待ち。B（写しを' docs/Plans.md`・`rg -n -F '間に合わないときの方針は未回答。backlog' docs/Plans.md` が何も出さない（exit 1）。
- AC-S1-3: `rg -c 'TD-46[1-4]' docs/Plans.md` が 1 以上で、TD-461・TD-462・TD-463・TD-464 がそれぞれ `docs/Plans.md` に出る（`rg -n -F 'TD-461' docs/Plans.md` ほか 4 つ）。
- AC-S1-4（読んで確かめる）: G2 の事実の段落が `docs/backlog.md` の項目「PLU をレジへ書く時期と D-070 の自動在庫連動」の 1 か所にあり、出所（BIZ-03・PLU-24・Coordinator の調べ）を引き、件数の実値を書かず（「ごく一部」）、owner の選択肢 A / B / C が書かれている。`docs/Plans.md` は `## 次の行動` の D-070 の段落の関門への指し 1 文と、関門の candidate の子（1〜2 文、B・C が D-070 の「初日から」を改める決定になること、backlog の名）だけを持つ。`rg -n -F 'Coordinator の調べ、2026-10-27' docs/Plans.md docs/backlog.md` が backlog の 1 行だけを出し、`rg -n -F '読みがある' docs/Plans.md docs/backlog.md` が何も出さない（exit 1）。
- AC-S2-1: `rg -n -F '在庫の振替（詰め替え・切り分け）の `movement_type`（2026-10-20 の点検' docs/backlog.md` と `rg -n -F '本 runtime の起票で、この前提と UI-03-D8 の既定の関係を確かめる' docs/backlog.md` が何も出さない（exit 1。親の下の子を置き換えた）。
- AC-S2-2: `rg -n '^## 旧「次の行動」の未完了項目' docs/backlog.md` が何も出さない（exit 1）。
- AC-S2-3: `rg -n -F '別 backlog' docs/UI_TECH_STACK.md` が何も出さない（exit 1）で、`rg -n -F 'native の window の close で取込み・取消・未保存の 3 つの止めを横断して扱う' docs/backlog.md docs/UI_TECH_STACK.md` が 2 行（両 file に 1 行ずつ）。
- AC-S3-1: `rg -n '未回答の問い Q-09[3468]' docs/backlog.md docs/function-design/10-common-rules.md` が何も出さない（exit 1）。
- AC-S9-1（新しい文言）: `git diff 3573dc67...HEAD -- docs | rg '^\+' | rg -n 'T[O]DO|T[B]D|F[I]XME|H[A]CK|未確[定]|要確[認]|要検[討]'` が何も出さない（exit 1）。文字の組の形は、本 packet の文が doc check の印の検査に当たらないようにするためで、探す語は 7 語のまま。
- data safety: `git diff --name-only 3573dc67...HEAD` が上の 6 file と本 packet だけ。`git diff --name-only 3573dc67...HEAD -- src-tauri src scripts .github .claude` が何も出さない。`git diff 3573dc67...HEAD | rg '^\+' | rg -n '[.]l[o]cal/'` が何も出さない（repo 外の報告の path を書かない。文字の組の形は本 AC の文が自分に当たらないようにするため）。店の運用の細部・実値・発言の原文を書いていない（読んで確かめる）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/Plans.md` の `## 導入の前の関門`（TD-345 の形）、`docs/decision-log.md` の D-070（Decision の ⑤・Impact）・D-111（未決 A・B と Revisit）・D-114（Decision (8) と Revisit の更新・migration の前の保全）・D-118（Revisit の共有 JAN）・D-123、ADR [2026-10-16-stocktake-count-placement](../adr/2026-10-16-stocktake-count-placement.md) SPEC-STK-CNT-D4、[56](../function-design/56-ui-daily-sales.md) UI-09a-D13・D19、[57](../function-design/57-ui-monthly-sales.md) UI-09b-D9、[63](../function-design/63-ui-return-exchange.md) UI-03-D8、[71](../function-design/71-mnt-backup.md) MNT-01-D3、[10](../function-design/10-common-rules.md) SPEC-UNIT-D2、`docs/UI_TECH_STACK.md` UI-USW-D4、`docs/ACCEPTANCE_SCENARIO.md` の `## 目的と判定方法`、`docs/initial-load-runbook.md` の J2〜J4 と流れ 1・流れ 2 の手順 6、`docs/project-memory.md` の Z004 の field-check の行と go-live の検証の流れの行、b1 の [packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md) の owner の判断事項 (i)〜(iii)。
- 必要な設計成果物: Design artifact selection の当たる行は無い（振舞い・command・DB・画面・format を変えない）。状態は existing sufficient。
- plan にしかない durable な判断の昇格先: 本 packet が決める判断は 2 つ。(1) S1 の区分の決め方（owner の答えの時点と TD-345 の前後で candidate にする）。昇格先は関門の各子の箇条の理由の句そのもの。(2) G3 の ⑤ を着手対象の項目へ移し空の節を外す。昇格先は backlog の項目そのもの。どちらも既存の規則（D-127、AGENTS.md の AI 由来の要件の扱い）の適用で、decision-log には足さない。D-111 の未決 A の答えは owner の決定の写しで、D-111 の末尾の行が置き場。
- 前提・制約と延期した gap: G2 の事実は Coordinator の調べ（2026-10-27）の写しで、PLU を書く時期の決定は owner（C3 の後）。D-070 と D-117 の Why の書き直しはその決定の後（B・C を選べば新しい D）。G8 は導入の日だけを決め、56 UI-09a-D19 の紙の切替えの順は変えない（Non-scope）。
- 絶対保証の自己点検: 本 lane は「〜できない」「必ず〜」を新しく書かない。G6 の「既定のまま保存すると在庫が黙って動かない」は b1 の起票で確かめた関係の写し（PLU で売る在庫連動の商品の返品だけが Z004 で反映される）。
- 判定: ready。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| source / workflow doc 新設・改名・削除 | 親文書の目次・索引 | 本 packet の新設だけ（`docs/plans/` の dated packet で、索引は持たない。D-097）。backlog の節 `## 旧「次の行動」の未完了項目` の削除は、repo の中に見出しへの link が無いことを `rg -n '旧「次の行動」の未完了' docs --glob '!docs/archive/**'` で確かめた（本 lane の新しい項目の「旧:」の句だけが出る） |

ほかの行（Tauri command、function-design の新設、REQ / coverage、route、operator 画面）は該当なし（10 の SPEC-UNIT-D2 の 1 句を直すだけで、REQ の参照を増減させない）。

## Impact Review Lenses

not applicable: 起点は現地調査・実機確認・外部 tool の振舞いの発見でなく、owner の決定（TD-460〜TD-464）と repo の中の docs の照合。G2 の事実は Coordinator の調べ（code と repo の記録）の写しで、C1・C3 の実機の確かめは後続。本 lane はその結果に依らない。

## Boundary / Wire Contract

変えない。JSON・CSV・config・manifest・DTO・bindings・DB に触れない。

## Test Plan

Test Design Matrix は作らない: R2 で、確かめは `rg` の件数と読む確かめ（AC）で足りる（Risk Tiers の R2 は Matrix を任意にする）。

- targeted tests: AC1〜AC3。code を変えないので Rust・frontend の test は足さない。traceability の検査は classifier が `traceability=true` を出すので CI が回す（`docs/function-design/10-common-rules.md` を触るため）。手元では Plan Review round 1 の reviewer が対象 `2e7739c7` で `cargo run --offline --bin generate_traceability -- --check` を回し「OK（ERROR 0 件 / WARN 0 件）」。round 1 の是正は 10 を触らない。
- negative tests: AC-S1-2・AC-S2-1・AC-S2-2・AC-S2-3 の前半・AC-S3-1・AC-S9-1（旧い表現・禁じた語が残らないこと）。
- compatibility checks: AC1（link の先の file の実在）、AC-S1-1（関門の名が backlog にある）。
- data safety checks: 上の data safety の AC。
- main wiring/integration checks: AC-S1-1・AC-S1-4（読んで確かめる）。
- classifier: `bash scripts/ci/classify-changes.sh --base 3573dc67487fb8f3987f250539870abb2d6ff45c --head HEAD` の出力は `rust=false`・`rust_drift=true`・`frontend=false`・`docs=true`・`env=false`・`generated=false`・`traceability=true`・`workflow=false`・`unknown=false`（round 1 の是正の run で取った）。PR の Validation に写す（`workflow=true` が出たら Final Review Minimum を 2 に直す）。

## Review Focus

- S1 の区分の決め方（candidate と precondition-dependent の分け方）が owner の決定と合うか。特に G3（D-070 の入口条件なので confirmed に近い）を precondition-dependent に、G4（正本が go-live の前と書く）を candidate に置いたこと。
- 関門の文が「中身は backlog の〈名〉」の形を守り、関門の文に中身を継ぎ足していないか（D-127）。
- G2 の事実の写しが出所（`src-tauri/src/biz/csv_import_service/parse.rs` の `find_by_jan_code` の引き当て、`docs/plu-export-and-real-csv-verification.md` の PLU-24）と合い、件数の実値を書いていないか。G6 の b1 の起票の関係の写しが b1 の packet と合うか。
- G8 の節の冒頭の文（導入は関門が全部済んだ後の月の初め、紙の切替えの日は導入の日と別で D19 が決める）が、56 UI-09a-D19 の順（D13 の受入 → 運用開始 → 紙の照合 → 切替え）と食い違わずに読めるか。
- この変更で不要になる記述・条件が他に無いか（D-127）。
- 並走の lane との衝突の見込み（上の所有の表）が正しいか。

## Implementation Results

docs だけの lane で、変更は plan-first の commit に入れた（上の Scope の各表の「結果」列）。

## Review Response

### Plan Review round 1（対象 `2e7739c7`）

reviewer は fresh Opus 5.5（fork でない）。reject（P1 0 / P2 1 / P3 7）。Codex の Plan Review は使用量の枠のため行わず、fresh Opus だけで進める（`docs/AGENT_OPERATING_MANUAL.md` §3.3、repo 外の回答台帳 TD-465）。裁定は Coordinator（2026-10-27、全件受理）、是正は起草役。

| finding | 重大度 | 裁定 | 是正（直した所） |
|---|---|---|---|
| Opus #1: Plans の G8 の文「紙の切替え（UI-09a-D19）に合わせる」が D19 の順（(2) の照合は運用開始の後）と両立しない | P2 | 受理 | `docs/Plans.md` の `## 導入の前の関門` の冒頭を「導入は関門が全部済んだ後の月の初め（TD-464）。紙の切替えは D19 の順のとおり (2) の照合の後の月の初めで、導入の日とは別」に直した。TD-464 の「紙の切替えに合わせる」は Coordinator が選択肢の説明に書いた句で、owner が選んだのは「関門が揃い次第」なので、D19 を変えずにこの形にした（Coordinator の裁定）。packet の Non-scope・Design Readiness・Review Focus を決着した形に直した |
| Opus #2: G2 の B・C は D-070 の「初日から」を改める選択なのに書いていない | P3 | 受理 | `docs/Plans.md` の関門の candidate の G2 の子に「B・C は D-070 の「初日から」を改める決定になる（決めるときは新しい D）」。packet の Design Readiness にも同じ句 |
| Opus #3: native close の項目の「取消（55 §55.7）」は base の §55.7 に無い | P3 | 受理 | `docs/backlog.md` の「native の window の close で取込み・取消・未保存の 3 つの止めを横断して扱う」に「取消の止めは PR #197 が §55.7 に足す」の出所 |
| Opus #4: G2 の事実の段落が Plans の 2 か所と backlog にほぼ同文で 3 か所 | P3 | 受理 | 正本を backlog の「PLU をレジへ書く時期と D-070 の自動在庫連動」1 か所にし、Plans の関門の子を 1〜2 文と backlog の名に、Plans の `## 次の行動` の D-070 の段落を関門への指し 1 文に縮めた。packet の S1 の G2 の行・S4・AC-S1-4 を合わせた |
| Opus #5: 「導入の後に溜まった記録は後から直せない」は b1 の packet の (iii) に無い断定 | P3 | 受理 | `docs/Plans.md` の G6 の子と `docs/backlog.md` の「返品・交換の `register_processed` の既定と差額だけの戻しの交換」から外した |
| Opus #6: 区分の決め方の規則と G2 の candidate が食い違う | P3 | 受理 | packet の S1 の区分の決め方に「owner の選択肢に「関門に足す」を含むもの（G2）は、X の後に決めるものでも candidate」 |
| Opus #7: Test Plan の「traceability の検査は足さない」が classifier の出力と合わない | P3 | 受理 | packet の Risk と Test Plan に classifier の出力（`traceability=true`・`rust_drift=true` ほか）と、round 1 の reviewer の traceability の検査の結果を書いた |
| Opus #8: 「導入に効くかもしれないもの」の (3) Q-103 に主題が無い | P3 | 受理 | `docs/backlog.md` の (3) に主題「商品ごとの最低限の在庫数」。(2) Q-099 は主題「評価方法の届出」が既にあるので変えない |

起草役が是正の中で見つけたもの: round 1 の版の本 packet の末尾に、起票の run の tool の呼出しの閉じ tag の 2 行が残っていた。外した。

### Plan Review round 2（対象 `be5a72ff`）

reviewer は fresh Opus 5.5（fork でない）と Codex（発注 625、GPT-6.1 Sol）。Opus は approve（P1 0 / P2 0 / P3 4）、Codex は reject（P1 0 / P2 1 / P3 1）。裁定は Coordinator（2026-10-27）。Codex #1 は前提の事実を repo 外の回答台帳で確かめ、直す所を変えて受理した。ほかは全件受理。

| finding | 重大度 | 裁定 | 是正（直した所） |
|---|---|---|---|
| Opus #1: G2 の子の「C3」が Plans の中で定義されておらず、D-070 の Impact の「C3（MSI 配布判定）」と別の意味で読める | P3 | 受理 | `docs/Plans.md` の candidate の G2 の子に「C3（店の PC の CV17 の画面で部門リンクの値を確かめる、TD-459）」 |
| Opus #2: 関門の冒頭の G8 の 2 文目が 56 UI-09a-D19 を一部だけ写している | P3 | 受理 | `docs/Plans.md` の `## 導入の前の関門` の冒頭の文を「日報の紙の切替えの日は導入の日と別で、56 UI-09a-D19 が決める。」に置き換えた（写しを外した） |
| Opus #3: backlog の振替の項目の「TD-378 は問い Q-092 の答えの前の判断」が番号の順と合わない | P3 | 受理 | `docs/backlog.md` の「在庫の振替（詰め替え・切り分け）」を「b1 の packet が Q-092 を未回答と扱った時点の判断で」に直した |
| Opus #4: SD 直読みの子に backlog の名が無い | P3 | 受理 | `docs/Plans.md` の precondition-dependent の SD 直読みの子の末尾に「backlog の「SD 直読みの runtime」。」 |
| Codex #1: backlog の G2 の項目の「店は全部の商品を部門キーで売っている」が、project-memory の値下げ品の売り方（バーコードを読んでから値引きキー、2026-09-29）と食い違う | P2 | 直す所を変えて受理 | 前提を確かめた: project-memory の行の元の問いは「PLU 登録後に値下げ品を売る方法」（repo 外の回答台帳 TD-052、2026-09-29）で、今の売り方ではない。今の売り方は全部の商品を部門キー（同じ台帳の L-001、2026-09-17 の再確認）。backlog の断定は正しく、食い違いは project-memory の行が前提を落としていたことから来る。`docs/project-memory.md` の行の頭を「PLU をレジに書いた後の値下げ品の売り方」にし、今は全部の商品を部門キーで売ることを添えた（Scope に 1 file 足した。S4・data safety の AC・他の lane との所有の表）。`docs/backlog.md` の G2 の項目の出所に 2026-09-17 の確かめと project-memory の行の位置づけを添えた |
| Codex #2: G2 の子が 3 文と backlog の名の 1 文で、AC-S1-4 の「1〜2 文」を超える | P3 | 受理 | `docs/Plans.md` の G2 の子を、選び方と B・C の新しい D を 1 文にし、backlog の名を含む 2 文にした（Opus #1 の C3 の定義と同じ文） |

### plan closure（対象 `d3afd7a7`）

reviewer は fresh Opus 5.5（fork でない）と Codex（発注 629、GPT-6.1 Sol）。Opus は approve（P1 0 / P2 0 / P3 2）、Codex は approve（P1 0 / P2 0 / P3 0）。round 2 の 6 件はどちらも閉じたと判定した。裁定は Coordinator（2026-10-27、全件受理）。

| finding | 重大度 | 裁定 | 是正（直した所） |
|---|---|---|---|
| Opus N1: project-memory の値下げ品の行の括弧「今は PLU を書いていないので」は、レジに死蔵の PLU が残る事実（同じ file の他の行）と字義どおりには合わず、出典も 2026-09-29 だけ | P3 | 受理 | `docs/project-memory.md` の括弧を「今は PLU を使っておらず、全部の商品を部門キーで売る。PLU-24、2026-07-23。2026-09-17 の確かめも同じ」にした |
| Opus N2: packet の Non-scope と Review Focus が G8 の文を是正の前の言い方（導入の後の別の日）で引く | P3 | 受理 | packet の 2 か所を Plans の今の文（紙の切替えの日は導入の日と別で D19 が決める）に合わせた |
