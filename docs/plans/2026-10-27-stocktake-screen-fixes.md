# Plan Packet: 棚卸しの画面の小口（導入の前の関門 6 と、b1 の残り）

wave に属さない単独の lane。出所は owner の決定（2026-10-24、repo 外の回答台帳 TD-433: PR #188〈b1〉の L3 の所感 2 点を、小さな別 lane か PR #202 で直す。#202 の packet は扱っていない）。[導入の前の関門](../Plans.md#導入の前の関門) の 6 と、[backlog](../backlog.md) の「棚卸しの「今の数を入れる」の欄が route の `product` が外れると閉じない」「棚卸しの runtime b1 の packet との食い違い（Coordinator の裁定待ち）」の (1)〜(3) を 1 つの lane で扱う。本 commit は Plan Packet・Test Design Matrix の起票と、設計正本（[73](../function-design/73-ui-stocktake.md)・[51](../function-design/51-ui-product-form.md)）の plan-first の修正だけ。実装は PR #193（b2。同じ画面の file を触る）の merge と `origin/main` の同期の後の別の run。新しい決定の番号（D-139）は使わない（決めたことは 73 の UI-10-D10 の段落と冒頭の D-118 の節、51 の D-118 の節に置いた）。

## 規模と手法

2026-10-27 に本 worktree（HEAD `50f97a76`）で数えた（command は逐語、出力を写した）:

| 区分 | 数 | command と出力 |
|---|---|---|
| 一覧の最終カウントが保存の時刻を出す所 | 1 | `rg -c 'formatCountedAt\(item\.counted_at\)' src/features/stocktake/StocktakePage.tsx` → `1` |
| 商品フォームの「在庫連動にできません」 | 2（aria-label と見出し） | `rg -c '在庫連動にできません' src/features/products/ProductFormPage.tsx` → `2` |
| 同じ語を assert する test | 1 | `rg -c '在庫連動にできません' src/features/products/ProductFormPage.test.tsx` → `1` |
| 一覧の保存の時刻を assert する test | 1（T16） | `rg -c '2026-10-01 09:05:00' src/features/stocktake/StocktakePage.test.tsx` → `1` |
| route の `product` が外れたときに何もしない分岐 | 1 | `rg -c 'if \(searchProduct === undefined \|\| !statusLoaded\) return;' src/features/stocktake/StocktakePage.tsx` → `1` |
| 計数の日付を出す画面の所 | 2（一覧の `counted_at`、記録の詳細の `count_business_date`） | `rg -n "count_business_date\|counted_at" src/features --glob '!*.test.*'` → `StocktakePage.tsx` の 189・223（欄の文脈の写し）・1477（一覧）、`StocktakeRecordDetailPage.tsx` の 208（`item.count_business_date ?? "—"`） |
| 新しい文言と衝突する test の matcher | 0 | `rg -F -c '「POS販売で在庫を減らす」にできません' src` → exit 1、`rg -F -c '在庫の数に入っているか確認できません' src` → exit 1（Plan Review round 3 の是正で 51 の文を替えて数え直した。2026-10-27、`8027fd6c`） |
| wire の型・command・DB の列・migration | 0（`StocktakeItemDetail.count_business_date` は b1 から bindings にある） | `rg -n 'count_business_date: string \| null' src/lib/bindings.ts` の `StocktakeItemDetail`（1727 行目） |
| 予定 path | 10（docs 4・TS 6。下の「予定 path の一覧」） | 下の一覧の行数 |

手法: **1 lane・1 PR。** 理由: 3 つとも frontend の表示と状態だけで、wire・DB・command を変えない。棚卸しの画面（`StocktakePage.tsx`）の 2 点と商品フォームの 1 点は、b1 の L3 の所感と b1 の残りという同じ出所を持ち、1 回の L3 で見られる。分けると L3 と Ready・merge の owner の手間が 2 倍になる。backlog の 2 項目（欄が閉じない・b1 の食い違い）は同じ 2 つの test の file（`StocktakePage.test.tsx`・`ProductFormPage.test.tsx`）に当たるので入れる。段は S1（棚卸しの一覧）→ S2（欄の閉じ方）→ S3（商品フォーム）。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: 9c21bb91cb321f19a99c61476550e586a5380871
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票と実装）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）。座組表どおり
- Final Reviewer: Fable 5（Claude 側、R3 以上。当面 Fable 5〈repo 外の回答台帳 TD-410〉）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge,manual
- Branch: agent/stocktake-screen-fixes

遷移の記録:

1. kickoff → spec-check（2026-10-27、起草役）: Coordinator の発注（関門 6 と backlog の 2 項目）を範囲にし、規模を数えて 1 lane にした（上の「規模と手法」）。Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-27、起草役）: 対象の正本は 73 の冒頭の D-118 の節（「今の数を入れる」）・UI-10-D10・§73.6・§73.12・§73.13 と、51 の D-118 の節の 2 つ目の項。一覧の最終カウントの日付（73 UI-10-D10 は `counted_at` と書く）と商品フォームの文（51 は BIZ と同じ文を書く）は正本の変更が要る。route の `product` が外れたときの欄の閉じ方は 73 に書いていない。
3. design → plan-draft（2026-10-27、起草役、本 commit）: 上の 3 点を 73・51 に本 packet と同じ commit で置いた（proposed・未実装と今の build を併記）。製品の振舞いで owner の判断が要る問いは 1 つ（下の「owner の判断事項」）で、推奨を既定にして進める（owner の方針、repo 外の回答台帳 TD-401）。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-27、Coordinator）: packet と Test Design Matrix は commit 済み。owner の判断事項 (i) は A に決まった（repo 外の回答台帳 TD-452、前の commit `d79289a0` で反映）。b1 の packet との食い違いは Coordinator の裁定で (1) a・(2) a（済みとして閉じる）・(3) a（51 の正本どおり欄の直下へ移し、関門 6 の (i) の語の直しと 1 つにする、S3）。Plan Review へ出す（fresh Opus 5.5 + Codex）。
5. plan-gate（round 3、天井、2026-10-27、Coordinator の disposition、起草役、本 commit）: round 3 の一括是正（Review Response 参照）
6. plan-gate → plan-approved（2026-10-27、Coordinator、本 commit）: 条件は `docs/DEV_WORKFLOW.md` の遷移表の「Writer でない独立の Plan Reviewer が P1 / P2 = 0」。round 3 の一括是正の版（`25ff60b6`）の plan closure は fresh Opus 5.5 が approve（P3 1）、Codex（発注 628）が approve（P3 0）。P3 1 件を plan-gate のまま直した版を Plan Commit にした（`9c21bb91`。下の Review Response）。
7. plan-approved → implementing（2026-10-27、Coordinator、本 commit）: 遷移表の唯一の入口。前文の順のとおり #193 が先に merge し（`a2365759`）、`origin/main` を単段で同期した（`0625abe4`。`docs/function-design/51-ui-product-form.md` の D-118 の契約の見出しの衝突は、b2 の「実装済み」と本 lane の「画面の小口の lane で proposed」の両方を残して解いた）。同期の後の版で AC と「規模と手法」の baseline の command を逐語で再測し、packet の値と一致した。変わったのは 2 つだけ: `formatCountedAt` の行が 91・125・708・1470・1743 に移った（b2 の変更で行がずれた。数は 5 のまま）、traceability の `StocktakePage.test.tsx|ProductFormPage.test.tsx` の行の数が 2 から 5 になった（b2 が test を足した）。実装の Writer は Coordinator が発注する。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定。L3 は見込み 10 分）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 1（2026-10-27。起票は TD-433 の決定から Coordinator が発注した。owner の判断事項 (i) を選択肢で問い、A に決まった〈repo 外の回答台帳 TD-452〉） | 3（L3 と画面の確かめ 1、Ready 1、merge 1） | 2 | 6 = 1 + 3 + 2 |

owner の操作の回数を減らす工夫: (1) owner の判断事項 (i) は plan-gate の前に選択肢 1 つで答えてもらった（TD-452）。(2) L3 は 2 画面の 1 本の台本で、各段の PASS / FAIL だけを返してもらう。画面の確かめ（Human Visual Confirmation）を同じ 1 回に含める。(3) L3 の起点の DB と商品は Coordinator が用意する（下の Test Plan の「L3 の入力物」）。owner は DB も file も作らない。(4) Ready と merge は 1 回の返事で「Ready にして hosted が緑なら merge」と委ねられる（decision point は 2 で数える）。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行の「route/search state」（route の `product` が外れたときの「今の数を入れる」の欄の閉じ方）と「operator workflow」（棚卸しの一覧の列の意味、商品フォームの拒否の案内の場所と文）に当たる。R4 の行には当たらない: DB・wire・command・migration を変えず、本番の DB は無い（`docs/project-memory.md`）。workflow の file（`.github/`・`scripts/`）に触れないので classifier の `workflow=true` にならず、Final Review Minimum は 1。

Rollback / recovery notes: 表示と画面の状態だけなので、merge の後に戻すのは revert の PR で足りる。

required gate の green / red: 変わらない（gate の定義と分類は変えない。本 commit は docs、実装は frontend の job が走るだけ）。

## Goal

Goal Invariant:

### 最小完了条件

- 棚卸しの一覧の「最終カウント」に、効く計数の業務日（「前の日の分を入れる」で選んだ日付、既定は保存した日。`YYYY-MM-DD`）が出る。前の日の分を入れた商品は、一覧でその日付に見える。
- 商品フォームで「POS販売で在庫を減らす」を「減らす」にした保存が `recount_required` で拒否されたとき、在庫の節の「数量単位」と「POS販売で在庫を減らす」の行のすぐ下（全幅）に、画面にある語（欄の名と「減らす」「減らさない」）だけで原因と直し方を書いた案内と「今の数を入れる」が出て、案内へ focus が移る。
- 「今の数を入れる」の欄は、route の `product` が外れても（別の商品の読込みの途中でも、保存の応答の保留中でも）その商品の保存の成功で閉じ、`/stocktake/` に留まる（保存を始めた時の `returnTo` へ戻らない）。

### 失敗定義

- 一覧の最終カウントが保存の時刻のまま、または未入力の行・業務日の無い行に日付や時刻が出る。`actual_count` が 0 の行に「—」が出る。
- 商品フォームの案内に「在庫連動」が残る、BIZ の message をそのまま出す、上部に残って欄の近くに出ない、出ても focus が移らない、「今の数を入れる」の導線や離脱の確認（UI-USW-D1）が変わる。
- route の `product` が外れた後の保存で欄が閉じない、閉じるべきでない欄（別の商品へ変わった欄の未保存の入力）を閉じる、`returnTo` が無いのに戻る、route の `product` が外れた後に保存の時の `returnTo` へ戻る、外れていない（別の商品の読込みが失敗した）のに戻らない、保存の応答の保留中にサイドバーで別の画面へ移った後に、届いた保存の成功で商品フォームか棚卸しの画面へ連れ戻す。

### 非目的

- wire（`StocktakeItemDetail` の `counted_at` を外す等）・BIZ の message・DB を変えること。
- 共有 JAN の拒否の文（「同じ JAN の商品（…）があるため、在庫連動にできません」。BIZ の message で、商品一括 import と共有。下の Non-scope）。
- b1 の食い違いの (4)（入出庫の記録の一覧の「差異なし」。発注の範囲の外）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

operator の操作と画面の状態を変える packet なので、店の通常の操作列を置く。例の日付・商品は説明用の合成の値。**この文書を完了できる**ことと、**棚卸しと商品の設定を通常運用できる**ことは同じ（b1 の画面の 3 点の迷いが無くなる）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 今日 10-27、棚卸し中。商品 A は未計数で、A に計数が無いか、効く計数の日付が 10-26 より前（「未計数」は今回の明細を参照する計数の有無で決まり〈35「開始と明細」〉、一覧の業務日は商品の効く計数から取る〈`src-tauri/src/biz/stocktake_service.rs` の `item_detail` が `read_ledger(conn, &row.product_code)` と `representative_count(&ledger)` から `count_business_date` を作る〉。10-26 以後の計数が既にあると、10-26 を入れても効くのはそちらで一覧はその日付を出す） | 棚卸しの画面で A を検索し、「前の日の分を入れる」で 10-26 を選び、数 5 を「数を保存」 | 一覧の A の行の「最終カウント」が `2026-10-26`（時刻なし） | 次の商品へ | 73 UI-10-D10 の段落、§73.6 |
| 同じ。商品 B は未計数 | B を検索し、数 3 を「数を保存」（日付は今日のまま） | B の行の「最終カウント」が `2026-10-27` | — | 同じ |
| 同じ。A の行 | A を検索し「入れ直す」で 4 | A の行が `2026-10-27`（fresh な計数は今日） | — | 73 の D-118 の節の「入れ直す」 |
| 商品 C は「減らさない」で、共有 JAN が無く、連動なしの Z004 の売上があり、その売上より後の効く計数が無い（保存が `recount_required` になる。30 の在庫連動を始める・ADR D8） | 商品の修正で C の「POS販売で在庫を減らす」を「減らす」にして「保存する」 | 在庫の節の「数量単位」「POS販売で在庫を減らす」の行のすぐ下に「「POS販売で在庫を減らす」にできません」と文、「今の数を入れる」。画面がその案内へ移る（focus）。入力は「減らす」のまま | 「今の数を入れる」→ 離脱の確認で「破棄して移動」 | 51 の D-118 の節 |
| 棚卸しの画面の「今の数を入れる」の欄が C で開いた | 棚を数えて「今の数を保存」 | C の修正の画面へ戻る（DB の値で「減らさない」）。もう一度「減らす」にして保存すると通る | 保存 | 73 の D-118 の節（b1 のまま） |
| 「今の数を入れる」の欄が C で開いている | C の数を「今の数を保存」した直後、応答の前にサイドバーの「棚卸し」を押す | 応答の後に欄が閉じ、棚卸しの画面に留まる（C の修正の画面へ戻らない） | — | 73 の D-118 の節（本 lane。サイドバーを先に押してから保存する順は今の build でも閉じて留まる。保存を始めた時の route に `returnTo` が無いため） |

失敗と再試行: 計数の保存の失敗は今のとおり欄に留まり上部に文を出す（U9 の失敗の test）。商品フォームの保存の別の失敗は今の上部の error の Alert（変えない）。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

正本で一意に決まらない製品の振舞いは 1 つ。推奨を既定にして Scope・AC・Matrix を書いた（owner の方針、repo 外の回答台帳 TD-401）。**owner の決定（2026-10-27、repo 外の回答台帳 TD-452）: A（数えた日）。** Scope・AC・Matrix は既定のまま変わらない。

(i) 棚卸しの一覧の「最終カウント」に何の日付を出すか（73 UI-10-D10 は保存の時刻と書くが、b1 で計数は業務日で効くようになり、owner の所感〈TD-433〉は一覧に入れた日付が出ないことを挙げた。ADR と 73 は一覧の列を決めていない）:

- **A（推奨、既定）**: 効く計数の業務日だけ（`YYYY-MM-DD`）。理由: 「前に保存した数」の欄と棚卸しの記録の詳細（`StocktakeRecordDetailPage.tsx` の `count_business_date ?? "—"`）と同じ値になり、一覧で入れた日付が効いたかを確かめられる。wire を変えない。
- B: 保存の時刻のまま（今の build）。所感の迷いが残る。
- C: 業務日と保存の時刻の両方（例「2026-10-26（保存 10-27 09:05）」）。1 つの列に 2 つの種類が混ざる。列の幅が増える。

答えで変わる所: B なら S1・SF1・L3-1 を外し、73 の UI-10-D10 の段落と §73.6 を戻す Gated Amendment。C なら S1 の書式と SF1 の oracle、73 の同じ 2 か所を変える Gated Amendment。

## b1 の packet との食い違い (1)〜(3) の裁定の材料

backlog の項目は「Coordinator の裁定待ち」。裁定は Coordinator が plan-gate で行う。下の推奨を既定に書き進めた。

(1) b1 の L3-11 の台本の「棚卸しを完了する」と画面の button「棚卸しを確定する」:

- **a（推奨、既定）: 済みとして閉じる。** b1 の Gated Amendment の 2 本目の 4（[b1 の packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md) の Review Response の後の「直した・足した契約の文」の 4）で L3-11 の台本の button の名を画面に合わせた。button の名の正本は 73 §73.10 の「確定 CTA | 棚卸しを確定する」と UI-10-D4 で、画面の他の文（「棚卸しの確定」「棚卸しを確定できません」）も「確定」で揃う。b1 の packet の Ordinary Operation の 1 行に「棚卸しを完了する」が残るが、archive の履歴なので書き換えない。
- b: button を「棚卸しを完了する」に変える。73 の文言と、locator の 20（`rg -c 'name: "棚卸しを確定する"' src --glob '*.test.*'` → `StocktakePage.suggest.test.tsx:1`・`StocktakePage.test.tsx:19`）を変える。画面の他の「確定」の語と割れる。

(2) 検索で開く欄の「入力済みの数を上書きできます」:

- **a（推奨、既定）: 済みとして閉じる。** b1 の Gated Amendment の 2 本目の 3 で「数を直すときは「入れ直す」を押してください。同じ日付で「数を保存」から違う数を入れると、数え直しが必要になります」に置き換え（73 の冒頭の D-118 の節の「b1 の画面の細部」）、`StocktakePage.tsx` と T6 に入っている。`rg -n "上書きできます" src` → 0 件（exit 1）。backlog の文は b1 の本文の Follow-up を写した時点の古い記述。
- 選べる別案は無い（直す先が既に正本と code にある）。

(3) 商品フォームの `recount_required` の文がフォームの上部にあり、checkbox の近くでない:

- **a（推奨、既定）: 51 の正本どおり欄のすぐ下へ移し、関門 6 の (i) の語の直しと 1 つにする（S3）。** 案内は保存の button から離れた所に出るので、案内へ focus を移して見落とさないようにする。
- b: 上部のまま語だけ直す。51 の「checkbox の近く」と食い違ったままになり、51 の注記（b1 の実装は上部）を残す。
- c: 上部と欄のすぐ下の両方。同じ案内を 2 つ出す（継ぎ足し）。

## Scope

本 lane が変える file と目的。実装は S1 → S3 の順。既存の test の移し方は Matrix の「既存の test の移し方」。

### S0（本 commit、plan-first）

- [73](../function-design/73-ui-stocktake.md): UI-10-D10 の段落（最終カウント列を業務日に。why・rejected・revisit）、§73.6 の最終カウント列の行、§73.12 の T16 の起点、§73.13 の L3-3 の文、冒頭の D-118 の節の「今の数を入れる」の行（route の `product` が外れたときの閉じ方）、更新履歴。
- [51](../function-design/51-ui-product-form.md): D-118 の節の 2 つ目の項（欄の行のすぐ下・文・focus・型で分岐・BIZ の message を出さない理由）。

### S1 棚卸しの一覧の最終カウント（関門 6 の (ii)）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 一覧の「最終カウント」の値 | `formatCountedAt(item.counted_at)` を、`item.actual_count === null \|\| item.count_business_date === null` なら「—」、ほかは `item.count_business_date`（そのまま）に。見出し「最終カウント」は変えない。`formatCountedAt` は開始日・前回の棚卸しの日時で使い続ける（`rg -n 'formatCountedAt' src/features/stocktake/StocktakePage.tsx` → 5 行: import・前回のサマリ・継続表示ヘッダ・一覧・結果画面の前回の棚卸し。一覧の 1 行だけを外す） | `src/features/stocktake/StocktakePage.tsx` の `StocktakeItemList` の表 | Matrix の SF1 |
| `formatCountedAt` の doc comment | 「最終カウント日時を表示する」を、S1 の後の用途に合わせて「棚卸しの日時（開始・完了）を表示する」の趣旨に直す（振舞い・名前は変えない。改名は任意） | `src/features/stocktake/lib/stocktake-formatters.ts` | 変えない（`stocktake-formatters.test.ts`） |

### S2 「今の数を入れる」の欄の閉じ方（backlog「route の `product` が外れると閉じない」）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| route の `product` の effect | `searchProduct === undefined`（status の読込み後）のとき、遅い保存の応答の照合（`standaloneCodeRef`）を今の欄の商品（`standaloneFieldCodeRef`）へ戻し、その商品の閉じ待ち（`standalonePendingCloseRef`）があれば実行する（`settleStandaloneLoadFailure` の後始末と同じ。error の文は出さない）。途中の読込みの応答は今の cleanup の `standaloneSeqRef` の加算で捨てる（変えない）。`standalonePendingCloseRef` と `settleStandaloneLoadFailure` の 2 つのコメントと、切り出した後始末の関数の説明を、route の `product` が外れたときの実行を含む形に直す（今のコメントは閉じ待ちを実行するのを「読込みが失敗して欄がその商品のまま決まれば」だけと書く） | `src/features/stocktake/StocktakePage.tsx` の `StocktakePage` の route の effect（必要なら後始末を小さな関数に切り出し、`settleStandaloneLoadFailure` と共有する） | Matrix の SF4・SF6、既存の U9 の 8 つの宣言（期待を変えない） |
| 保存の後の戻り先（`handleStandaloneSaved` の `finish`） | `finish` を実行する時点で route の `product` が外れていれば、保存を始めた時に捕らえた戻り先（`returnTo`）へ戻らずに閉じて `/stocktake/` に留まる。外れていなければ今のまま（保存の時の `returnTo` へ戻る）。route が別の商品へ変わってその読込みが失敗しただけなら外れていない側（U9 の N1〈1571 行〉・N1b〈1760 行〉が `onReturnTo("/products/P-005/edit")` を期待する。その期待を保つ）。`finish` を実行する時点で `StocktakePage` が unmount していれば（保存の応答の保留中にサイドバーで別の画面へ移った）、何もしない（`onReturnTo` を呼ばず、`closeStandalone` も呼ばない: `closeStandalone` の search の更新は `/stocktake/` への route の遷移で、離れた先から棚卸しの画面へ連れ戻す）。mount 中かは ref 1 つで見る（例: `useEffect(() => { mountedRef.current = true; return () => { mountedRef.current = false; }; }, [])`。body でも立てるのは `React.StrictMode`〈`src/main.tsx`〉の dev で effect が cleanup の後にもう一度走るため）。route の search を ref に写すだけでは unmount の後に古い値が残るので、この確かめは別に要る（Plan Review round 3 の probe）。今の `finish` は `doSave` が mutate の前に捕らえた `onSaved` の closure の `search` で決めるので、実行する時点の route の `product` を読む（例: route の search の最新の値を ref に置く。`eslint.config.js` の `"react-hooks/refs": "error"` のため render の中で ref に書けないので、ref へ写す effect は route の `product` の effect より前に宣言する。後に置くと、route の effect が閉じ待ちの `finish` を実行する時点で ref が前の route を持ち、SF5 の (b) が赤になる） | `src/features/stocktake/StocktakePage.tsx` の `handleStandaloneSaved` | Matrix の SF5、既存の U9 の 8 つの宣言（期待を変えない） |

### S3 商品フォームの拒否の案内（関門 6 の (i) と b1 の食い違い (3)）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 案内の文 | `RECOUNT_REQUIRED_TEXT` を 51 の新しい文に、見出しと `aria-label` を「「POS販売で在庫を減らす」にできません」に | `src/features/products/ProductFormPage.tsx` | Matrix の SF2 |
| 案内の場所 | 上部の Alert を外し、`ProductForm` の新しい optional prop（例 `stockSyncNotice?: ReactNode`。ProductFormPage が組んだ案内をそのまま描く slot）で、在庫の節の `StockUnitField` の直後（同じ `FormSection` の中）に描く。`StockUnitField` は `grid md:grid-cols-2` で「数量単位」（左）と「POS販売で在庫を減らす」（右）を 1 行に並べ、window の `minWidth` は 1024（`src-tauri/tauri.conf.json`）なので常に 2 列。案内はその行のすぐ下に全幅で出る（`StockUnitField` は変えない）。案内の中身と導線（`onNavigateToStocktake`）は ProductFormPage が持つ（ProductForm に拒否の型を持ち込まない） | `src/features/products/ProductFormPage.tsx`・`src/features/products/components/ProductForm.tsx` | Matrix の SF3 |
| focus | 案内が出たら案内の領域（`tabIndex={-1}`）へ focus を移す（`useEffect` で `recountRequired` が true になったとき。button ではない） | `src/features/products/ProductFormPage.tsx` | Matrix の SF3 |

`ProductForm` の import と描画（S3 の後）:

| file | 新しく import するもの | 描くもの |
|---|---|---|
| `ProductFormPage.tsx` | 今のまま（`Alert`・`AlertTitle`・`AlertDescription`・`AlertTriangle`・`ClipboardCheck`・`Button` を使い続ける） | 上部の Alert を描かない。`ProductForm` に `stockSyncNotice` を渡す |
| `ProductForm.tsx` | `ReactNode` の型（`react`） | `StockUnitField` の直後に `stockSyncNotice` |

### 予定 path の一覧（`comm -12` の入力）

10 path（docs 4・TS 6）。所有表の突き合わせは L3 を頼む時点の head で取り直す（D-124）。

```
docs/function-design/51-ui-product-form.md
docs/function-design/73-ui-stocktake.md
docs/plans/2026-10-27-stocktake-screen-fixes.md
docs/plans/test-matrices/2026-10-27-stocktake-screen-fixes.md
src/features/products/ProductFormPage.test.tsx
src/features/products/ProductFormPage.tsx
src/features/products/components/ProductForm.tsx
src/features/stocktake/StocktakePage.test.tsx
src/features/stocktake/StocktakePage.tsx
src/features/stocktake/lib/stocktake-formatters.ts
```

closeout（merge の後の別の R0 PR）で: `docs/Plans.md` の関門 6 を済みに、`docs/backlog.md` の「棚卸しの runtime b1 の L3 の所感」「「今の数を入れる」の欄が…閉じない」を外し、「b1 の packet との食い違い」は (1)〜(3) を外して (4) だけを残す（共有の更新を closeout へ寄せる。`docs/DEV_WORKFLOW.md` Wave Operation の L3 の箇条）。

### 節の所有表（並走の lane と重なる file）

`comm -12` の結果（2026-10-27。並走の各 PR は `git diff --name-only --no-renames origin/main...origin/<branch>`、#193 はそれに加えて packet の「予定 path の一覧」〈`git show origin/agent/stocktake-count-runtime-b2:docs/plans/2026-10-23-stocktake-count-runtime-b2.md`〉。どれも `LC_ALL=C sort` した）:

| PR | 重なる path | 本 lane の節 | 相手の節 |
|---|---|---|---|
| #172・#190・#194・#196・#197・#201・#204 | なし | — | — |
| どの PR とも（Plan Review round 2 で足した path） | `src/features/stocktake/lib/stocktake-formatters.ts` は重ならない（2026-10-27、各 branch の `git diff --name-only --no-renames origin/main...<branch>` と #193 の予定 path の一覧に無い。#193 が触るのは `src/features/stock-movements/lib/movement-formatters.ts`） | — | — |
| #202（`agent/screen-elements-redesign`） | merge 済み（`ee5b221a`、base に含む）。並走の相手でない | — | — |
| #193（`agent/stocktake-count-runtime-b2`。branch の差分は docs 5 で重なりなし、予定 path と重なる） | `docs/function-design/51-ui-product-form.md` | D-118 の節の 2 つ目の項と、節の見出しの注記の 1 句 | 節の見出しと 4 つ目の項（在庫数の表示） |
| 同 | `src/features/stocktake/StocktakePage.tsx` | `StocktakeItemList` の最終カウントの 1 セル、`StocktakePage` の route の effect | `recheckQuery` を hook へ切り出す（`StocktakePage` の上の方の宣言） |
| 同 | `src/features/products/ProductFormPage.tsx` | 上部の Alert・`RECOUNT_REQUIRED_TEXT`・focus・`ProductForm` への `stockSyncNotice` | `ProductForm` への `stockRecheck` の prop |
| 同 | `src/features/products/components/ProductForm.tsx` | props の `stockSyncNotice` と在庫の節の描画 | props の `stockRecheck` と「現在庫」の欄の badge |
| 同 | `src/features/products/ProductFormPage.test.tsx` | U12（SF2・SF3 に書き換え） | U4 の追加 |

merge の順: #193 → 本 lane（#202 は merge 済み〈`ee5b221a`、base に含む〉。実装は #193 の merge の後に `origin/main` を単段で取り込んでから。`git -c merge.directoryRenames=false merge origin/main`）。本 lane の plan-first の commit（51・73）は #193 の merge の後の同期で 51 の 4 つ目の項を相手の版にし、51 の見出しは相手の版（在庫数の表示は b2 で実装済み）に本 lane の注記の 1 句（2 つ目の項の案内の場所・文・focus は画面の小口の lane で proposed）を足した形にする。L3 は本 lane が merge train の先頭になってから頼む（#193 と path を共有するので L3 を束ねない。`docs/DEV_WORKFLOW.md` Wave Operation）。

## Non-scope

- 共有 JAN の拒否の文（「同じ JAN の商品（…）があるため、在庫連動にできません」）: BIZ の message（30）を商品フォームの欄の近くと商品一括 import の行に出す。import では列名「POS在庫連動」（60 §「idle」の任意列）が画面にあるので語は通じる。商品フォームだけ直すには message を解析するか BIZ の文を変える（import と共有）ことになり、関門 6 の所感の外。同じ画面に「在庫連動」の語の案内が 1 つ残るので、closeout で backlog に 1 行足す（出所: 本 packet。Coordinator の裁定、Plan Review round 2）。
- `StocktakeItemDetail.counted_at` の wire からの撤去（下の「この変更で不要になるもの」）。
- b1 の食い違いの (4)（`InventoryRecordsPage.tsx` の「差異なし」）。発注の範囲が (1)〜(3)。
- 一覧の列の見出し（「最終カウント」のまま）、並び順、他の列。

## Acceptance Criteria

- AC1（test）: `npm test -- src/features/stocktake src/features/products` が全 PASS し、Matrix の SF1〜SF6 の test 名が `rg -c` で各 1 件見つかる（`rg -c 'SF1 ' src/features/stocktake/StocktakePage.test.tsx` ほか、Matrix の Test Matrix の表の file ごと）。
- AC2（一覧の時刻の撤去）: `rg -c 'formatCountedAt\(item\.counted_at\)' src/features/stocktake/StocktakePage.tsx` が baseline `1`（2026-10-27、`50f97a76`）→ 完了時 0 件（exit 1）。一覧のセルに固有の条件 `rg -c 'item\.count_business_date === null' src/features/stocktake/StocktakePage.tsx` が baseline 0 件（exit 1。2026-10-27、`b0eb03e0`）→ 完了時 1 以上（S1 の条件の式。`item.count_business_date` だけの式は欄の文脈の写し〈189 行〉で baseline から 1 なので使わない）。
- AC3（商品フォームの語）: `rg -c '在庫連動にできません' src/features/products/ProductFormPage.tsx` が baseline `2` → 完了時 0 件（exit 1）。`rg -F -c '「POS販売で在庫を減らす」にできません' src/features/products/ProductFormPage.tsx` が baseline exit 1 → 完了時 1 以上。
- AC4（正本の proposed の撤去）: `rg -c '今の build は' docs/function-design/73-ui-stocktake.md` が本 commit の後 `2` → 完了時 0 件（exit 1）。`rg -c 'proposed・未実装' docs/function-design/73-ui-stocktake.md` が本 commit の後 `4` → 完了時 `1`（更新履歴の本 lane の行だけ）。`rg -c '今の build は b1 の実装のまま' docs/function-design/51-ui-product-form.md` が本 commit の後 `1` → 完了時 0 件（exit 1）。節の見出しの注記 `rg -c '画面の小口の lane で proposed' docs/function-design/73-ui-stocktake.md docs/function-design/51-ui-product-form.md` が本 commit の後 各 `1`（Plan Review round 1 の是正の後、2026-10-27）→ 完了時 0 件（exit 1。見出しを実装済みの書き方に戻す）。
- AC5（frontend の gate）: `npm run typecheck && npm run lint && npm run format:check && npm test && npm run build` が exit 0（`docs/DEV_WORKFLOW.md` Verification Gates の Frontend の行）。
- AC6（docs）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-27-stocktake-screen-fixes.md` が ERROR 0。
- AC7（manual）: L3-1・L3-2 が PASS（下の Test Plan）で、helper の manual の record に残る（`python3 scripts/pr-gate.py status --pr <PR 番号>` の manual の行）。

## Design Readiness

- 引用する設計正本（節まで）: 73 の冒頭の D-118 の節（「今の数を入れる」の行・「b1 の画面の細部」）、UI-10-D10 とその段落、§73.5 の 3（前に保存した数）、§73.6（最終カウント列）、§73.10（見出し）、§73.12、§73.13 の L3-3。51 の D-118 の節の 1・2 つ目の項、UI-USW-D1・D2。ADR [2026-10-16-stocktake-count-placement](../adr/2026-10-16-stocktake-count-placement.md) の D3（業務日で効く）・D5（入れ直す）。35 の「計数の保存」（明細の `actual_count`・`counted_at` は効く計数の写し）と「読取り」（`count_business_date`）。30 の在庫連動を始める（BIZ の message）。60 の「idle」（列名「POS在庫連動」）。
- 必要な設計成果物: New or changed operator workflow, route/search state, Japanese wording → 73・51 の更新（updated in this PR、本 commit）。SCREEN_DESIGN・UI_TECH_STACK は existing sufficient（画面の一覧と D-052 の entry を変えない）。decision-log・ADR は不要（画面の中の表示の決定で、横断の決定でない）。
- plan にしかない durable な判断の昇格先: 最終カウントの日付 → 73 UI-10-D10 の段落。欄の閉じ方 → 73 の D-118 の節。案内の場所・文・focus・型で分岐 → 51 の D-118 の節。b1 の食い違い (1)(2) の閉じ方は正本の変更なし（既に正本と code にある）。
- 前提・制約と、延期した design gap: owner の判断事項 (i) は A に決まった（TD-452）。共有 JAN の文は Non-scope。
- 絶対保証の自己点検: 「欄は保存の成功で閉じる」の例外は、保存の応答の保留中に欄が別の商品へ変わった場合（その欄は閉じない。U9 の「Final Review G1」「N1b」の既存の test）と保存の失敗（閉じない。U9 の失敗の test）。「保存の後は `returnTo` へ戻る」の例外は、閉じる時点で route の `product` が外れている場合（戻らずに留まる。S2）と、保存の成功の後始末（閉じる・戻る）を行う時点で棚卸しの画面が unmount している場合（何もしない。S2）。「未入力の行の最終カウントの列は —」の例外は無い（前の計数を持ち業務日のある未入力の行も「—」。同じ行の差異の列は BIZ の `difference` で数が出る）。
- 判定: ready（正本で決めきれない振舞いは owner の判断事項 (i) だけで、A に決まった〈TD-452〉）。

## Registration / Generation Obligations

該当なし（command・route・REQ・function-design doc の新設・改名・削除なし。追加の test は既に REQ-205・REQ-101 を引く file の中なので `docs/function-design/90-traceability.md` は変わらない: `rg -c "StocktakePage.test.tsx|ProductFormPage.test.tsx" docs/function-design/90-traceability.md` → `2`。`cd src-tauri && cargo run --bin generate_traceability -- --check` は CI が回す）。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | 置き換わる外部 adapter と app-core の契約 | Architecture | 不該当: 外部の system に触れない | — |
| Fact check / design decision split | 観測と app の決定の分け | 73・51 | 観測: owner の L3 の所感（TD-433）。決定: 業務日を出す・欄の行のすぐ下と文。正本へ昇格した | 73・51 |
| Lifecycle / retry | 前・途中・後・失敗 | Matrix の State Lifecycle | 該当: 欄の読込みの途中で route が外れる・保存の失敗・遅い応答（S2） | Matrix |
| Operator workflow | 実際の順の操作 | Ordinary Operation | 該当: 前の日の分を入れた後の一覧、商品フォームの拒否から棚卸しを経て戻る | L3-1・L3-2 |
| Replacement path | 外部が変わったとき | — | 不該当 | — |
| Data safety / evidence | 実データを出さない証拠 | Data Safety | 該当: L3 は合成の商品と dev の DB | Data Safety |
| Reporting / accounting semantics | 集計・記録の意味の分け | 73 UI-10-D10 | 該当: 列に業務日と保存の時刻の 2 つの種類を混ぜない（fallback しない） | SF1 |
| Manual verification | 自動 test で示せない所 | Test Plan | 該当: 画面の見た目・focus の移動で画面が案内まで移ること・Windows の WebView の表示 | L3-1・L3-2 |
| 環境・再現性 | 新しい環境依存 | — | 不該当: 新しい toolchain・依存なし | — |

## Boundary / Wire Contract

wire は変えない。読む値の境界だけを書く。

- producer: BIZ `stocktake_service::item_detail`（`count_business_date` は効く計数の `business_date`、`actual_count`・`counted_at` は明細の表示用の写しで同じ効く計数から書く。35「計数の保存」）。
- consumer: `StocktakeItemList` の最終カウントのセル。
- wire type: `StocktakeItemDetail.count_business_date: string | null`、`actual_count: number | null`（`src/lib/bindings.ts`）。
- internal type: 表示はそのままの文字列（`YYYY-MM-DD`）。
- precision/range: 日付は BIZ の `validate_business_date` を通った値。数は表示しない。
- round-trip path: なし（表示だけ）。route の search（`product`・`returnTo`）の形は変えない。
- invalid input: `count_business_date` が null → 「—」。`actual_count` が null → 「—」（業務日があっても）。
- compatibility: b1 の前の明細（計数の行の無い `actual_count`）は dev の DB にしか無く（本番の DB は無い）、業務日が null なので「—」。

## Test Plan

Test Design Matrix: [2026-10-27-stocktake-screen-fixes](test-matrices/2026-10-27-stocktake-screen-fixes.md)。

- targeted tests: `npm test -- src/features/stocktake src/features/products`。
- negative tests: 未入力の行（業務日あり）の最終カウントの列の「—」、保存の保留中に unmount した後の戻り（SF5 の (d)）、`actual_count` 0 の行の日付、BIZ の message を出さない、上部に案内が無い、別の商品へ変わった欄を閉じない（既存の U9）。
- compatibility checks: 既存の U9 の 8 つの宣言（`rg -c '"U9 ' src/features/stocktake/StocktakePage.test.tsx` → `8`。`it.each` の 2 つを含む）・U12 の離脱の確認（`ProductFormPage.unsaved-guard.test.tsx`）が期待を変えずに通る。
- data safety checks: 合成の fixture だけ。
- main wiring/integration checks: `StocktakePage` の route の effect と `ProductFormPage` → `ProductForm` の slot を実際の component で描く（RTL）。

Writer の完了の条件: L3 の前に `cargo check --release`（`docs/DEV_WORKFLOW.md` Implementation Rules。Rust は変えないが owner の native build の前の決まり）。

L3（Human Gate `manual`。Windows native の `npm run tauri dev`。1 回の通しの台本で、各段の PASS / FAIL を返してもらう）:

| # | 画面 / 見る場所 | 到達手順 | 合格基準 | 入力物 |
|---|---|---|---|---|
| L3-1 | `/stocktake`、一覧の「最終カウント」 | 棚卸し中に、未計数の商品 X を検索 →「前の日の分を入れる」で昨日を選ぶ → 数を入れて「数を保存」。続けて別の未計数の商品 Y を今日の日付のまま保存 | X の行が昨日の日付、Y の行が今日の日付で、どちらも時刻が無い。未入力の行の「最終カウント」の列は「—」（前に数えたことのある商品の未入力の行も「—」。その行の「差異」の列には数が出ることがある） | 進行中の棚卸しと未計数の商品 2 つ（X・Y）のある dev の DB。X・Y のほかに未計数の行が 1 つ以上残る（「—」を確かめる）。X・Y は計数が無いか、効く計数の日付が昨日より前（昨日以後の計数があると、昨日を入れても効くのはそちらで一覧はその日付を出す）。Coordinator が用意する（b1・b2 の L3 の後の DB は、この条件を満たす 2 つを選べば足りる） |
| L3-2 | `/products/{C}/edit`、在庫の節 | 商品 C を開く →「POS販売で在庫を減らす」を「減らす」にして「保存する」→ 出た案内の「今の数を入れる」→「破棄して移動」→ 数を入れて「今の数を保存」 | 保存の後、画面が在庫の節の「数量単位」「POS販売で在庫を減らす」の行のすぐ下（全幅）の案内まで移り、見出し「「POS販売で在庫を減らす」にできません」と文が読める（「在庫連動」の語が無い）。画面の上部に同じ案内が無い。棚卸しを経て C の修正の画面へ戻る | 商品 C（`pos_stock_sync=false`、共有 JAN が無く、連動なしの Z004 の売上があり、その売上より後の効く計数が無い商品。保存が `recount_required` になる〈30 の在庫連動を始める・ADR D8〉）。Coordinator が DB を用意し、商品コードを依頼に書く |

欄の閉じ方（S2）は route の切り替えと応答の順の組合せで、目で確かめるには操作の速さに依る（L3 Eligibility の (3) の手前の再現の手順）ので L3 に入れず、SF4〜SF6 の自動 test で示す。

## Review Focus

- S2 の後始末が、今の欄の商品の閉じ待ちだけを実行し、別の商品へ変わった欄（未保存の入力）を閉じないこと（既存の U9 の 8 つの宣言の期待）。閉じる時点で route の `product` が外れていれば保存の時の `returnTo` へ戻らず、外れていなければ（別の商品の読込みの失敗を含む）今のまま戻ること（SF5、U9 の N1・N1b）。保存の成功の後始末（閉じる・戻る）を行う時点で棚卸しの画面が unmount していれば、戻らず閉じる操作の route の更新もしないこと（SF5 の (d)）。
- S1 が `actual_count` の null と 0 を分け、業務日の無い行で保存の時刻へ戻らないこと。
- S3 の案内が型で分岐し（message を解析しない）、BIZ の message を出さないこと。focus が導線の button でなく案内の領域に移ること。
- この変更で不要になる記述・条件・code が残っていないか（下の節）。

## この変更で不要になるもの

- `ProductFormPage.tsx` の上部の Alert の分岐と、BIZ と同じ文の `RECOUNT_REQUIRED_TEXT`（S3 で置き換える。文の定数は新しい文になる）。
- 一覧の `formatCountedAt(item.counted_at)`（S1）。`formatCountedAt` 自体は開始日と前回の棚卸しで使うので残す（`src/features/stocktake/lib/stocktake-formatters.test.ts` の test も残す）。doc comment の「最終カウント日時を表示する」は一覧で使わなくなると誤りになるので、S1 で用途（棚卸しの開始・完了の日時）に直す。
- 51 の D-118 の節の「b1 の実装は文と導線を商品フォームの上部の warning の Alert に置いた」の注記（本 commit で外し、proposed の注記に替えた。実装の commit で proposed の注記も外す）。
- 73 の UI-10-D10 の決定の 1. の「最終カウントは既存 `formatMovementDateTime` と同じ…」は決定の履歴として残し、上の段落で置き換えを示した（73 の他の置き換えと同じ書き方）。
- backlog の (1)(2) は b1 の Gated Amendment の 2 本目で既に済んでいる（上の裁定の材料）。closeout で項目を外す。
- `StocktakeItemDetail.counted_at`: S1 の後、frontend は一覧で読まなくなる（`rg -n "counted_at" src/features --glob '!*.test.*'` で 1477 行目だけ）。撤去は wire（bindings・Rust の DTO・TS の fixture）を変える R3 の別の変更で、明細の `stocktake_items.counted_at` の列は BIZ が写しとして書き続けるので、本 lane では残す。closeout の backlog の整理で候補に残すかを Coordinator が決める。
- 不要になる条件: なし（S2 は今の `searchProduct === undefined` の早い return を置き換える分岐と、`finish` が実行する時点の route の `product` を読む手段〈例: ref 1 つ〉と mount 中かを見る ref 1 つを足す。新しい state は足さない）。
- `StocktakePage.tsx` の旧いコメントの句（S2 で誤りになる）: `standalonePendingCloseRef` の「読込みが失敗して欄がその商品のまま決まれば実行し、別の商品の欄が開けば捨てる」と、`settleStandaloneLoadFailure` の「欄の読込み（route の product・「入れ直す」）の最後の操作が失敗したら、欄は前の商品のままなので」（閉じ待ちの実行を読込みの失敗だけと読める）。S2 で route の `product` が外れたときの実行を含む形に置き換える。code は実装の run で直す。
- Matrix の M11（閉じ待ちを商品の照合なしで実行する）: 等価の mutant（閉じ待ちは欄の商品 = 保存した商品のときだけ置かれ、`[standaloneTarget]` の effect が欄の変わるたびに消す）なので必須の red に置かない（Plan Review round 1）。照合の条件は既存の code で、残す。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| UI-10-D10（最終カウント列 = 効く計数の業務日、未入力・業務日 null は「—」、時刻へ戻さない） | 73 UI-10-D10 の段落・§73.6 | S1 | SF1 | L3-1 |
| UI-10-D10（見出し「最終カウント」・差異列・在庫列は変えない） | 73 §73.6・§73.10 | 変えない | T16（SF1 に書き換えた後も差異・在庫の assert を保つ） | — |
| 73 D-118 の節「今の数を入れる」（route の `product` が外れても欄は残り、その商品の保存の成功で閉じる。途中の読込みの応答は捨てる） | 73 冒頭の D-118 の節 | S2 | SF4・SF6 | 非対象（Test Plan の理由） |
| 73 D-118 の節「今の数を入れる」（閉じる時点で route の `product` が外れていれば保存の時の `returnTo` へ戻らずに留まる。別の商品の読込みの失敗だけなら外れていない側。保存の成功の後始末（閉じる・戻る）を行う時点で画面が unmount していれば何もしない） | 73 冒頭の D-118 の節 | S2 | SF5、U9 の N1・N1b | 非対象（Test Plan の理由） |
| 73 D-118 の節「今の数を入れる」（`returnTo` の許す先、保存の後に戻る、別の商品の欄を閉じない、失敗で戻らない） | 73 冒頭の D-118 の節 | 変えない | 既存の U9 の 8 つの宣言（失敗の test を含む） | b1 の L3 で済み |
| 73 D-118 の節「b1 の画面の細部」（検索で開く欄の「入れ直す」の文） | 73 冒頭 | 変えない（b1 の食い違い (2) は済み） | T6 | — |
| UI-10-D4・§73.10（確定 CTA「棚卸しを確定する」） | 73 | 変えない（b1 の食い違い (1) は済み） | T11 ほか 20 の locator | — |
| 51 D-118 の節の 2（欄の行のすぐ下・見出しと文〈原因を断定しない「…入っているか確認できません」。`recount_required` は計数の衝突でも返る、ADR D8〉・型で分岐・BIZ の message を出さない・案内へ focus・導線の search） | 51 | S3 | SF2・SF3 | L3-2 |
| 51 D-118 の節の 2（離脱の確認を通る・戻った後の保存は `/products`） | 51、UI-USW-D1・D2 | 変えない | U12 の離脱の確認（`ProductFormPage.unsaved-guard.test.tsx`） | L3-2 の「破棄して移動」 |
| 51 D-118 の節の 1（共有 JAN の文） | 51・30 | 非対象（Non-scope） | — | — |
| DSR-08（色だけで示さない） | design-system 01 | S3 の案内は icon と文字を保つ | SF3（見出しの文字） | L3-2 |

## Contract Probe

N/A: 外部の library・OS・hardware の前提に依らない（React の `useEffect` と RTL の既存の使い方の範囲。focus は jsdom の `toHaveFocus` で示せる）。

## Data Safety

- 実 POS・店のデータ・DB・backup・log を commit しない。
- L3 の DB は dev の DB（`npm run tauri dev` の app data）で、repo に入れない。
- test の fixture は合成の値だけ。

## Implementation Results

Fill after implementation.

## Review Response

### Plan Review round 1（`b0eb03e0`。fresh Opus 5.5 と Codex。両方 reject）

Coordinator の裁定は全件受理。行は 出所・重大度・裁定・直した所。

- Opus F1・Codex #1（P2、probe で再現）、受理: 保存を始めた時に捕らえた `returnTo` が route の `product` が外れても失効せず、閉じるときに商品フォームへ戻る。→ S2 に「保存の後の戻り先」の行（失効と、`finish` を実行する時点で外れていれば留まる。外れていなければ今のまま、別の商品の読込みの失敗だけでは失効しない）、Goal の最小完了条件と失敗定義、Design Readiness の絶対保証、Review Focus、Contract Ledger。73 の冒頭の D-118 の節の「今の数を入れる」の行と更新履歴。Matrix の SF5 を `returnTo` 付きの fixture の 3 つの順（応答の前に外れる・閉じ待ちの後に外れる・B を挟まない）に、mutant M12〜M14 を足した
- Codex #2（P2）、受理: Ordinary Operation と L3-2 の商品 C の条件が `recount_required` を保証しない。→ 両方を「共有 JAN が無く、連動なしの Z004 の売上があり、その売上より後の効く計数が無い」（30 の在庫連動を始める・ADR D8）に揃えた
- Codex #3（P2）・Opus F2（P3）、受理: M11 は等価の mutant。→ Matrix の Mutation の表から外し理由を 1 文に、SF6 は既存の「B の欄へ変われば A の閉じ待ちを破棄する」契約の確かめに書き直した。「この変更で不要になるもの」に 1 行
- Opus F3（P3）、受理: AC2 の後半 `rg -c 'item\.count_business_date'` は baseline で既に 1。→ 一覧のセルに固有の `rg -c 'item\.count_business_date === null'`（baseline exit 1 → 完了時 1 以上）に替えた
- Opus F4（P3）、受理: 「U9 の 6 本」は `it.each` の 2 つを拾わない。→ `rg -c '"U9 '` → `8` の「8 つの宣言」に直した（S2・Test Plan・Review Focus・Contract Ledger・Matrix）
- Opus F5（P3）、受理: 73 の冒頭の新しい文の「など」が M1 の WARN を 1 足す。→ 「サイドバーの「棚卸し」の link で外れても」に直した
- Opus F6（P3）、受理: L3-1 の X・Y に昨日以後の計数があると誤って FAIL する。→ L3-1 の入力物に「X・Y は計数が無いか、効く計数の日付が昨日より前」を足した
- Opus F7（P3）、受理: 51・73 の節の見出しの「b1 で実装済み」が本文の proposed と食い違って読める。→ 両方の見出しに本 lane の proposed の 1 句を足し、節の所有表と merge の順の同期の手順（51 の見出しは相手の版に本 lane の 1 句を足す）を直した。AC4 に見出しの注記の撤去を足した

### Plan Review round 2（`c313d1b5`。fresh Opus 5.5 は approve〈P3 8〉、Codex は reject〈P2 1〉）

Coordinator の裁定。行は 出所・重大度・裁定・直した所。code・test は本 commit で変えない。

- Codex #1（P2）、受理: Ordinary Operation の先頭の行の「未計数」だけでは、商品に今日の計数が既にあると昨日の分を入れても一覧が今日になり、正しい実装を誤判定する。→ 先頭の行の初期状態を L3-1 と揃え、「A に計数が無いか、効く計数の日付が 10-26 より前」と根拠（35「開始と明細」の「計数済み」、`stocktake_service.rs` の `item_detail` の `read_ledger`・`representative_count`）を足した
- Opus F1（P3）、受理: 「欄の直下」は `StockUnitField` の 2 列の配置（`grid md:grid-cols-2`、window の `minWidth` 1024）に合わない。→ 51 の D-118 の節の 2・L3-2・Goal・Ordinary Operation・Matrix の Contracts Under Test と State Lifecycle を「在庫の節の「数量単位」「POS販売で在庫を減らす」の行のすぐ下（全幅）」に揃え、S3 の「案内の場所」に配置の根拠を足した
- Opus F2（P3）、受理: S2 の「error の文は出さない」に oracle が無い。→ Matrix の SF4・SF5 (c) に「見出し「操作できませんでした」の Alert（`role="alert"`）が出ない」、Mutation の表に M15 を足した
- Opus F3（P3）、受理: `react-hooks/refs` が error で render の中で ref に書けない。→ S2 の「保存の後の戻り先」に、ref へ写す effect を route の effect より前に置く理由（逆だと SF5 (b) が赤）を足した
- Opus F4（P3）、受理: Ordinary Operation の 6 行目（サイドバーの後に保存）は今の build でも成り立つ。→ 「保存の直後、応答の前にサイドバー」に直し、73 の冒頭の D-118 の節の「今の build」の注記を「保存の応答の保留中に外れても保存の時の `returnTo` へ戻る」に直した
- Opus F5（P3）、受理: 73 の UI-10-D10 の段落が「推奨を既定にした」のまま。→ owner の決定（2026-10-27、repo 外の回答台帳 TD-452、A）に置き換え、73 の更新履歴の 2026-10-27 の行を 2026-10-24 の行の後へ移した
- Opus F6（P3）、受理・実装へ: S1 の後 `formatCountedAt` の doc comment が誤りになる。→ S1 の表に comment の直しの行、「この変更で不要になるもの」に 1 文、予定 path に `src/features/stocktake/lib/stocktake-formatters.ts`（10 path）と所有表の行を足した。code は実装の run で直す
- Opus F7（P3）、裁定: 共有 JAN の文は Non-scope のまま、closeout で backlog に 1 行足すと確定した（Non-scope の文を置き換えた）
- Opus F8（P3）、受理: L3-1 の入力物に未計数の行が残ることが書かれていない、73 の L3-3 の起点が今日もう数えた行だと日付の更新を目で見られない。→ L3-1 の入力物に「X・Y のほかに未計数の行が 1 つ以上残る」、73 §73.13 の L3-3 に「最終カウントが今日より前の行」を起点として足した

### Plan Review round 3（`8027fd6c`。fresh Opus 5.5 は reject〈P2 2・P3 3〉、Codex 623 は reject〈P2 1〉。天井）

Coordinator の disposition（2026-10-27）: round 3 は `docs/DEV_WORKFLOW.md` Review Rules の round の天井。どの P2 も具体的な筋が probe か BIZ の現物で裏づけられ（Codex #1 は Coordinator が `src-tauri/src/biz/stock_ledger.rs` の `unlinked_sales_counted` の doc comment と `conflicting` の分岐で確かめた）、同じ lane の契約の文の直しで閉じるので、owner への escalation でなく同型指摘の一括是正とする。全 6 件を受理。是正の後は fresh の reviewer の plan closure（Claude 側 fresh Opus 5.5 + Codex）に出す。plan-gate のままの是正で Plan Commit は pending なので、Gated Amendment は無し。行は 出所・重大度・裁定・直した所。code・test は本 commit で変えない。

- Opus F1（P2、probe で再現）、受理: 保存の応答の保留中にサイドバーで別の画面へ移る（`StocktakePage` が unmount する）と、届いた保存の成功で `onReturnTo` が呼ばれ商品フォームへ連れ戻す。→ S2 の「保存の後の戻り先」の行に、`finish` を実行する時点で unmount していれば何もしない（`onReturnTo` も `closeStandalone` も呼ばない）と、mount 中かを見る ref（StrictMode のため effect の body でも立てる）を足した。Goal の失敗定義、Design Readiness の絶対保証、Test Plan の negative tests、Review Focus、「この変更で不要になるもの」の ref の数、Contract Ledger。73 の冒頭の D-118 の節の「今の数を入れる」の行と更新履歴。Matrix の Contracts Under Test、Failure Modes（「サイドバーで離れたのに商品フォームへ連れ戻す」を SF5 の (a)〜(c) と (d) に結んだ）、SF5 を `it.each` の 4 つに（(d) 保存を保留 → unmount → 応答を解く → `onReturnTo` も `onSearchChange` も呼ばれない。test 名を替えた）、State Lifecycle、mutant M16
- Opus F2（P2）、受理: SF1 の未入力の行の fixture（業務日あり・差異 null）は producer が出さない状態。→ Matrix の Test Matrix に「fixture の行は producer の出す状態にする」の規則を足し、SF1 の未入力の行を (a) 計数の無い行（「—」が差異と最終カウント）と (b) 前の計数を持つ行（業務日 `2026-09-20`・差異 `-1`。最終カウントの列のセルが「—」）に分けた。(b) は M2 を検出するために要る（(a) だけでは業務日が null で M2 が緑のまま）。正本（73 §73.6・S1 の式）は未入力の行の最終カウントを「—」とするので、(b) の最終カウントは業務日でなく「—」。0 の行に差異 `-2` を足した。73 §73.12 の文と §73.10 の文言の表の行、packet の L3-1 の合格基準と Design Readiness を列を名指す文に直した
- Opus F3（P3）、受理: #202 は `ee5b221a` で main に入り base に含む。→ 節の所有表の #202 の行を「merge 済み」に、merge の順を「#193 → 本 lane」に、同期の手順から #202 を外した
- Opus F4（P3）、受理: `standalonePendingCloseRef` と `settleStandaloneLoadFailure` のコメントが S2 の後に誤りになる。→ S2 の route の effect の行に 2 つのコメントと後始末の関数の説明を直す指示を、「この変更で不要になるもの」に旧いコメントの句を足した（code は実装の run で直す）
- Opus F5（P3）、受理: 「失効」と「閉じる時点で外れていれば」の 2 つの読み。→ 73 の冒頭の D-118 の節、Matrix の Contracts Under Test、S2、Contract Ledger、Matrix の SF5・U9 の行・State Lifecycle・M14・state token を「保存の成功で閉じる時点で route の `product` が外れていれば戻らない」の 1 つの読みにし、「失効」の語を外した。F1 の unmount の条件も同じ文の形にした
- Codex #1（P2）、受理: 51 の文「売上が在庫の数に入っていません」は、`recount_required` が計数の衝突・決まらない売上でも返る（ADR D8、`unlinked_sales_counted`）ことと合わない。→ 51 の D-118 の節の 2 の文の冒頭を「「減らさない」の間の売上が在庫の数に入っているか確認できません」に替え、断定しない理由を足した（回復の手順の文は変えない）。「規模と手法」の衝突の matcher の baseline を新しい文言で数え直した（`rg -F -c '在庫の数に入っているか確認できません' src` → exit 1）。Contract Ledger の 51 の行に文の要点を足した。Matrix の SF2、packet の S3・AC3 は文を「51 の文」で引いて literal を持たないので変えない

### plan closure（`25ff60b6`。fresh Opus 5.5 は approve〈P3 1〉、Codex 628 は approve〈P3 0〉）

round 3 の 6 件はどちらも閉じたと判定した。裁定は Coordinator（2026-10-27、受理）。

- Opus N1（P3、受理）: unmount を判定する時点が「保存の成功が届く時点」（73・Matrix・packet）と「`finish` を実行する時点」（S2）の 2 通りに書かれ、間に一覧の再取得の await がある。73 の D-118 の節・Matrix の Contracts Under Test と State Lifecycle・packet の絶対保証・Review Focus・Contract Ledger の句を「保存の成功の後始末（閉じる・戻る）を行う時点で」に揃えた（S2 と同じ時点。test は足さない）。
