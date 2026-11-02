# Plan Packet: 棚卸しの数の置き場と在庫数の求め直しの runtime (b2)（要再確認の範囲と計数の履歴を商品の画面で見せる）

wave に属さない単独の lane。D-118（[ADR](../adr/2026-10-16-stocktake-count-placement.md)）の runtime の配布の単位 (b)（ADR SPEC-STK-CNT-D12）を 2 つに分けた後の lane（b2）。最初の lane（b1）は PR #188（branch `agent/stocktake-count-runtime-b`、packet は b1 の branch の `docs/plans/2026-10-22-stocktake-count-runtime-b1.md`）。導入の前の関門 2（[Plans](../Plans.md#導入の前の関門) の 2.、D-070 で v1.0 の条件）は b2 の merge で満たす。b1 の packet の「後続の lane」は b2 を「b1 の merge の後に起票する」と書くが、owner の決定（2026-10-23、repo 外の回答台帳 TD-391。b1 の実装の変化で当て直しが要ることを承知の上）で、b1 の実装の間に先に起票する。本 commit は Plan Packet と Test Design Matrix の起票と、b1 が触らない正本（[44](../function-design/44-cmd-inventory.md)・[66](../function-design/66-ui-stock-movements.md)）の plan-first の修正だけで、実装は plan-approved の後の別の run。branch は `agent/stocktake-count-runtime-b2`（main `eed07d37` から）。新しい決定の番号（D-131）は使わない（起票で決めたことは 44・66 の D-118 の節に置いた）。

## b1 との関係（stacked、b1 の後に merge）

- b1 の契約の正本は、b1 の Plan Commit `aba17e2e2422695b2ffff6bcd3fd9bebebf24339` と Gated Amendment `5b53294040ff38d2aeb8366aefd02142ffb2c764`（fresh な計数の精算の判定。wire は変えていない）の packet・Matrix・正本（`git show <SHA>:<path>`）。本 packet の正本の読みはその版を基準にした。b1 の branch は実装の 1 段目が進行中で、wire の名前・形が b1 の packet と変われば、b1 の merge の後の同期で本 packet を当て直す（下の「b1 への依存」の各行が当て直しの対象。変われば Gated Amendment）。
- b2 の branch は main から作り、b1 の merge の後に `origin/main` を単段で取り込む（`git -c merge.directoryRenames=false merge origin/main`。`docs/DEV_WORKFLOW.md` Stacked train）。Plan Review は b1 の merge を待たずに行い（owner の決定 TD-402）、b1 の merge と同期は実装の開始の前の条件にする。
- 正本の版の差（main `eed07d37` と b1 の Plan Commit `aba17e2e`。`git diff c98fe7dd aba17e2e -- docs` で b1 が書き換えた所だけを見た）: b2 に関わる節のうち、[ADR](../adr/2026-10-16-stocktake-count-placement.md)（Status、D3 の fresh な計数の n、D4、D6、D9 の初期在庫の見分け方、D10、D12 の「(b) を 2 つの lane に分けた」の段落。main には b1・b2 の分けが無い）、[21](../function-design/21-io-inventory-repo.md)（`list_ledger_movements_for_product` の初期在庫の印）、[31](../function-design/31-biz-inventory-service.md)（Z004 の確定の書き方、`apply_stock_change` の引数）、[40](../function-design/40-cmd-product.md)（`CountRecoveryTarget` の範囲と単位、`SettlementPendingHint`）、[51](../function-design/51-ui-product-form.md)（「今の数を入れる」の導線と離脱の確認）、[65](../function-design/65-inventory-record-traceability.md)（入出庫の記録の一覧の棚卸しの件数、記録の詳細の補正の節）、[UI_TECH_STACK](../UI_TECH_STACK.md)（再取得の経路と entry）が b1 の版で書き換わっている。b2 の範囲の文（20 の D-118 の節の商品の読取り、40 の商品の結果の 3 列、50・58・66 の注記、51 の「在庫数の表示」、65 の D-118 の節の 1 つ目と 2 つ目の項）は両方の版で同じ。[20](../function-design/20-io-product-repo.md)・[44](../function-design/44-cmd-inventory.md)・[50](../function-design/50-ui-product-list.md)・[58](../function-design/58-ui-stock-inquiry.md)・[66](../function-design/66-ui-stock-movements.md)・[SCREEN_DESIGN](../SCREEN_DESIGN.md) は b1 の Plan Commit で変わっていない（20 と SCREEN_DESIGN は b1 の実装の S9 で D-118 の節の注記が変わる予定）。

## 規模と手法

2026-10-23 に本 worktree（HEAD `eed07d37`）で数えた（command は逐語、出力を写した）:

| 区分 | 数 | command と出力 |
|---|---|---|
| 形の変わる wire の型 | 2（`ProductWithRelations` に 3 列、`list_movements` の結果の型を `StockHistoryItem` に）+ 新しい型 3（`StockHistoryItem`・`StockHistoryKind`・`StockCountHistory`）。`StockDetail` は `ProductWithRelations` を持つので一緒に変わる。`MovementRecord` は形を変えない（下の「起票時に起草役が決めたこと」の 3） | `rg -c '^export type (ProductWithRelations\|StockDetail\|MovementRecord) =' src/lib/bindings.ts` → 3（b1 の packet の規模の表） |
| 出力が変わる Tauri command | 5（`get_product`・`search_products`・`get_stock_detail`・`list_low_stock`・`list_movements`） | `rg -c '^        cmd::(product_cmd::(get_product\|search_products)\|inventory_cmd::(get_stock_detail\|list_low_stock\|list_movements)),' src-tauri/src/lib.rs` → 5 |
| 商品の読取りの SQL | 4（`find_by_product_code`・`search_products`・`get_stock_detail`・`list_low_stock_products`） | `rg -c 'row_to_product_with_relations\(' src-tauri/src/db/product_repo.rs` → 5（定義 1 + 呼出し 4） |
| `MovementRecord` を作る所 | 7（変えない。`list_movements` の IO の 1 つだけを置き換える） | `rg -c 'MovementRecord \{' src-tauri/src` → `return_repo.rs` 1・`stocktake_repo.rs` 1・`manual_sale_repo.rs` 1・`sales_repo.rs` 1・`receiving_repo.rs` 1・`inventory_repo.rs` 2（定義と `list_movements`）・`disposal_repo.rs` 1 |
| 画面 | 4（[50](../function-design/50-ui-product-list.md) 商品一覧、[58](../function-design/58-ui-stock-inquiry.md) 在庫照会の一覧と詳細、[51](../function-design/51-ui-product-form.md) 商品フォームの修正、[66](../function-design/66-ui-stock-movements.md) 在庫変動） | SCREEN_DESIGN の D-118 の節の 9 行目（51・58・66）と、50 の D-118 の注記 |
| TS の mock の追随 | `ProductWithRelations` の literal 7 file、`listMovements` の mock 2 file（ほかは共有の fixture） | `rg -l 'plu_memory_no:' src --glob '!src/lib/bindings.ts' \| wc -l` → 7。`rg -l 'listMovements' src --glob '!src/lib/bindings.ts'` → `src/features/inventory-records/ReturnToFlow.test.tsx`・`src/features/stock-movements/hooks/useStockMovements.ts`・`src/features/stock-movements/StockMovementsPage.test.tsx` |
| 既存の `list_movements` の test | 10（Matrix の T1） | `rg -n 'fn test_list_movements' src-tauri/src \| wc -l` → 10 |
| migration・DB の列 | 0（写しの 3 列と `stock_counts` は v9 にある。ADR D12） | — |
| 予定 path | 64（Rust 9・TS 32・docs 23。下の「予定 path の一覧」。Gated Amendment 1 で `docs/architecture/io-task-specs.md` と `docs/architecture/ui-task-specs.md` を足した） | 下の一覧の行数 |

手法: **1 lane・1 PR のまま S1〜S5 の段に積む。** 理由: 読取りと表示だけで migration も書く経路も無く、b1 と同じ大きさの分割（配布の単位としての安全の検討）が要らない。64 path は b1 の 148 の 4 割で、1 回の L3（4 画面の見た目）で確かめられる。段は compile の単位で切る（S1 IO → S2 BIZ → S3 CMD と wire は 1 つの Rust の検証の単位、S4 画面は frontend の検証の単位、S5 は docs と生成物）。b1 の後に merge するので、b1 の merge の前に実装しない（b1 の merge と同期が実装の開始の前の条件）。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: 33831dd3c01266b0b458c0d8c8d189ad9895672c
- Amendments: bcde6607b6b39cd4f4819d7cd382d90891f04141
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票と実装）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）。座組表どおり
- Final Reviewer: Fable 5（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge,manual
- Branch: agent/stocktake-count-runtime-b2

遷移の記録:

1. kickoff → spec-check（2026-10-23、起草役）: Coordinator の発注（D-118 の (b) の b2。owner の決定 TD-391 で b1 の実装の間に先に起票）を範囲にし、規模を数えて 1 lane にした（上の「規模と手法」）。Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-23、起草役）: 対象の設計正本は ADR SPEC-STK-CNT-D1・D2・D3・D9・D12、[20](../function-design/20-io-product-repo.md)・[21](../function-design/21-io-inventory-repo.md)・[40](../function-design/40-cmd-product.md)・[44](../function-design/44-cmd-inventory.md)・[50](../function-design/50-ui-product-list.md)・[51](../function-design/51-ui-product-form.md)・[58](../function-design/58-ui-stock-inquiry.md)・[65](../function-design/65-inventory-record-traceability.md)・[66](../function-design/66-ui-stock-movements.md) の D-118 の節。設計の更新が要る点が 2 つあった（`list_movements` の結果の型・並び・絞り込み・合成の置き場、在庫変動の表の計数の行と日付の列。下の Design Readiness）。
3. design → plan-draft（2026-10-23、起草役、本 commit）: 上の 2 つを b1 が触らない 44・66 に本 packet と同じ commit で置いた。b1 が書き換えた節（ADR D12・21・40・51・65）は main の版で先に直さず、b1 の merge の後の同期で直す節として Scope に挙げた。製品の振舞いで owner の判断が要る問いは 1 つ（下の「owner の判断事項」）で、答えを待たずに推奨で進められる。plan-gate へは b1 の merge と同期の後に Coordinator が進める。
4. plan-draft → plan-gate（2026-10-23、Coordinator、本 commit）: owner の決定（repo 外の回答台帳 TD-402）で、b1 の merge を待たずに Plan Review を前倒しする（本 packet の起票の時は「b1 の merge と同期の後に plan-gate」としていた）。owner の判断事項 (i) は B（TD-397）で、推奨どおり Scope は起票のまま。review の基準は b1 の Plan Commit `aba17e2e` と、その後の Gated Amendment `5b53294040ff38d2aeb8366aefd02142ffb2c764`（fresh な計数の精算の判定を取込み ID のカーソルで求める。wire は変わらない）。b1 の merge の後に main を単段で取り込み、b1 の実装で wire の名前・形が変わった所を当て直す（変われば Gated Amendment）。
5. plan-gate のまま是正（round 1、2026-10-23、Coordinator、本 commit）: round 1（Claude 側 fresh Opus 5.5 と Codex）の reject を起草役が `dc9f4529` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 2（fresh Opus 5.5 + Codex）で再 review する。
6. plan-gate のまま是正（round 2、2026-10-24、Coordinator、本 commit）: round 2（Claude 側 fresh Opus 5.5 と Codex）の reject を起草役が `6d2e857a` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 3（天井、fresh Opus 5.5 + Codex）で再 review する。
7. plan-gate（round 3、天井、2026-10-24、Coordinator、本 commit）: round 3 は Claude 側 approve・Codex reject（P1 0）。`docs/DEV_WORKFLOW.md` Review Rules の round 天井に達したので round 4 は回さず、残った findings を disposition「同型指摘の一括是正」とした。起草役が `0256d19b` で直した（下の Review Response）。plan-approved は、fresh の reviewer の closure（Claude 側 + Codex）が P1 / P2 = 0 を報告した後。
8. plan-gate → plan-approved（2026-10-24、Coordinator、本 commit）: 独立の Plan Reviewer（Writer と別の context）の closure は Claude 側 fresh Opus 5.5 = approve（P3 2）、Codex = approve（P1/P2/P3 0）で、P1/P2 = 0（下の Review Response）。closure の P3 2 件は plan-gate のまま Coordinator が `33831dd3` で直した（Plan Commit はその版）。実装は、b1（PR #188）の merge の後に main を同期し、b1 の merge の後の同期で当て直す節（本 packet の「b1 の merge の後に同期する節」）を確かめてから implementing へ進める。実装の commit はまだ無い。
9. plan-approved → implementing（2026-10-27、Coordinator、本 commit）: 遷移表の唯一の入口。直前に `origin/main`（`50f97a76`、#188 b1 の merge を含む）を単段で merge した（`04401841`、衝突なし）。上の「b1 への依存」の 1〜13 を同期の後の版で照合し、名前・引数・形はどれも見込みのとおり（`StockCountKind`、`insert_stock_count` の `counted_at` の形と `stocktake_item_id`、初期在庫の印の式、`phase` と `settlement_no`、`stocktakeCountUpdate`、`formatStockOrRange`・`formatRecountReasons` の引数、`StocktakePage.tsx` の inline の `recheckQuery`〈key `queryKeys.stocktake.recheckList()`・`staleTime: 0`〉、`CountRecoveryTarget` の `recount_reasons`、`CmdError` の `stocktake_recovery`）で、Gated Amendment は要らない。AC の baseline の command を同期の後に逐語で再測し、変わったのは b1 の merge の版の数だけ: `proposed・未実装` が 13 → 17（b1 の S9 が節の見出しと本文に「b2 は proposed・未実装」と残した文）、`b1 の後・b2 の前の版` が exit 1 → 1、`MovementRecord` の並びの鍵の文が 2 → 4（ADR の 2 行、21 の D-118 の節、`docs/architecture/io-task-specs.md` の IO-01 の段落〈b1 が書いた「`MovementRecord` の並びの鍵は b2（proposed・未実装）」〉）。どれも b2 で直す文で、AC の完了時の期待は変わらない。`docs/architecture/io-task-specs.md` は予定 path の一覧に無いが AC5 の検索の範囲にあり、S5 の「b1 の merge の後に同期する節」と同じ扱いで b2 の分の文を現行に直す。節の所有表の取り直し（同期の後の head と open な lane の差分）: PR #172 とは `docs/architecture/io-task-specs.md`（#172 は IO-09 の節、b2 は IO-01 の段落の 1 文）、PR #202 とは `docs/function-design/50-ui-product-list.md`・`51-ui-product-form.md`・`58-ui-stock-inquiry.md`・`66-ui-stock-movements.md`（#202 は画面の要素の見直しの design の文。先に merge した方の版を起点に後の方が同期で当て直す）。ほかの open な lane（#190・#194・#196・#197・#201・#204）とは 0。Final Reviewer の Claude 側は座組表の当面 Fable 5（TD-410）に揃えた。
10. implementing のまま Gated Amendment 1（2026-10-27、Coordinator）: 実装の Writer の S1〜S5 の後（head `852b4365`）、PR #202（D-138、画面の要素の見直しの design lane）と #207 の merge で main が進み、PR が衝突したので、`origin/main`（`20e95076`）を単段で merge した（`9e2fda44`。衝突は `docs/FUNCTION_DESIGN.md` の UI-06a・UI-06c の行と 66 の §66.1 の主動線で、UI-06a の行と主動線の 1・4 は D-138 の版、UI-06c の行と主動線の 2・3 は本 lane の版にし、両方の意味を残した）。同期の後、AC5 の `proposed・未実装` の検索が 0 でなくなった: hit は 50・58・66・51・ARCHITECTURE の D-138 の節と注記（別の lane〈後続の「商品と在庫の画面の統合の runtime」〉の予定の文）で、本 lane の D-118 の (b) の分は全部入っている。AC5 は D-122 の項を除いているのと同じく D-138 の項を除くように式を直した（AC の意図「D-118 の (b) の分が全部入る」は変えない）。あわせて、Writer が見送った `docs/architecture/ui-task-specs.md` の D-118 の節の見出しと本文の「b2 は proposed・未実装」（2 行）を AC5 に別の句で足し（同じ file の D-120・D-121 の予定の文を拾わないよう、`proposed・未実装` の検索の範囲には入れない）、予定 path に `ui-task-specs.md` と、遷移の記録 9 で範囲に入れた `io-task-specs.md` を足した（64 path）。D-138 は今の build の画面を変えない（統合は後続の runtime）ので、本 lane の 4 画面の実装と L3 の台本（`/products`・`/stock` ほか今の route）は変わらない。登録は次の commit で本 commit の SHA を `Amendments` に足す。

## Owner Effort Budget

- 介入回数上限: 6（既定）
- 実働時間上限: 30 分（既定。L3 は見込み 10 分）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 6 | 2（2026-10-23。owner の判断事項 (i) の回答 TD-397 1、Plan Review の前倒し TD-402 1。起票の時期の決定 TD-391 は kickoff の依頼で数えない） | 4（owner の判断事項 (ii) を見せる 1、L3 と画面の確かめ 1、Ready 1、merge 1） | 0 | 6 = 2 + 4 + 0 |

予備が 0 なので、L3 の 2 round 目か、L3 の起点の DB が無いときの b1 の台本のやり直しが要れば、その問い合わせで上限の改定を一緒に諮る（`docs/DEV_WORKFLOW.md` Owner Effort Budget の「同じ 1 回で諮る」）。

owner の操作の回数を減らす工夫: (1) owner の判断事項は推奨つきで返事が記号 1 つの形にし、b1 の L3 か Ready の依頼と同じ問い合わせに束ねてよい（decision point は数える）。(2) L3 の起点は b1 の L3 の後の DB（要再確認の商品・計数・Z004 の取込み・完了した棚卸しがある。下の Test Plan の「L3 の起点の DB」）で、owner は DB も file も作らない。(3) L3 は 4 画面を見る 1 本の台本で、各段の PASS / FAIL だけを返してもらう。画面の確かめ（Human Visual Confirmation）を同じ 1 回に含める。(4) Ready と merge は owner が 1 回の返事で「Ready にして hosted が緑なら merge」と委ねられる（decision point は 2 で数える）。

既定値・数え方・上限に届くときの扱いは `docs/DEV_WORKFLOW.md` `Owner Effort Budget` 参照。
承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行の「Tauri command DTO」（5 つの command の出力、`list_movements` の結果の型）と「operator workflow」（商品一覧・在庫照会・商品フォーム・在庫変動の 4 画面の在庫の欄と在庫変動の並び・絞り込みの意味）に当たる。R4 の行（Destructive data lifecycle・backup restore・実データの露出・secret・不可逆な git の操作）には当たらない: migration も DB の書込みも無く（読取りと表示だけ）、本番の DB は無い（`docs/project-memory.md`）。workflow の file（`.github/`・`scripts/`）に触れないので、classifier の `workflow=true` にならず Final Review Minimum は 1。operator に見える状態（要再確認の範囲）の表示を扱うので、Contract Audit の 2 本目は任意の推奨（`docs/DEV_WORKFLOW.md` Contract Audit の Double audit）として Coordinator が決める。

Rollback / recovery notes: DB を変えないので、merge の後に戻すのは revert の PR で足りる（表示と wire だけが戻る）。

required gate の green / red: gate の定義は変えない。bindings と traceability の再生成を落とすと既存の required gate（bindings・traceability）が red になる（下の Registration / Generation Obligations）。

## Goal

Goal Invariant:

### 最小完了条件

- 商品一覧・在庫照会（一覧と詳細）・商品フォーム（修正）・在庫変動の見出しで、写しが要再確認の商品は在庫の欄に範囲（b1 の `formatStockOrRange` の形。例「8〜9 個」）と「要再確認」の状態（文字と icon、色だけでない）が出て、要再確認でない商品は今の 1 つの値のまま（ADR SPEC-STK-CNT-D3 の「要再確認の間の在庫は範囲で見せる」、owner の決定 TD-296）。
- 在庫変動の履歴に、計数が「数を確認（{数}、{日付}）」・「数を入れ直し（{数}、{日付}）」の行として movement と並びの鍵の順（新しい順）に混ざって出て、日付の絞り込みは業務日で movement と計数の両方に当たる（ADR SPEC-STK-CNT-D1、65 の D-118 の節の 1 つ目の項）。
- 在庫少の判定・並べ替え・ホームの件数は写しの値のまま変わらない（50・58 の D-118 の注記）。

### 失敗定義

- 要再確認の商品が 4 画面のどれかで範囲を出さない、または要再確認でない商品に範囲や badge が出る。要再確認を色だけで示す。
- 在庫変動の履歴に計数の行が出ない、計数の行が movement と鍵の順に混ざらない（保存の時刻の順のまま、または計数が先頭か末尾に固まる）、ページを切った後に行が欠ける・重なる、`total_count` が混ぜた件数と合わない。
- 日付の絞り込みが movement と計数で違う軸に当たる。種別の絞り込みで「棚卸し」に計数が出ない、ほかの種類に計数が混ざる。
- 在庫少の判定・並べ替え・ホームの件数が範囲の端で決まる。
- 業務記録の詳細（入庫・返品交換・手動販売・廃棄・CSV 取込み）の `movements` の形や表示が変わる。

### 非目的

- 書く経路と要再確認の判定（b1）。棚卸しの画面・Z004 の取込みの結果の要再確認の一覧（b1 の 73・55）。
- 出庫と返品の画面（返品交換・手動販売・廃棄）の商品の候補と行の「現在庫」の範囲の表示（下の「owner の判断事項」の (i)。入れない B に決まった、TD-397）。
- 鍵の順に並べ直した在庫の推移（`stock_after` の求め直し）を見せること（65 の D-118 の節の 2 つ目の項で持たない）。
- 整合性の画面と操作ログの画面の写しの書き方（「{数}（要再確認 {low}〜{high}）」。75・74 の今の書き方のまま）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更（command の出力の契約と画面の表示）なので、店の通常の操作列を置く。例の数・日付は説明用の合成の値。本 lane の merge の後に下の列が成り立つ。**この文書を完了できる**ことと、**棚卸しと POS の自動在庫連動の通常運用を達成できる**ことは同じ（b1 の書く経路に、b2 の読取りの表示が揃い、関門 2 を満たす）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 商品 A（個）を今日 10-02 に「入れ直す」で 8 と保存した後、同じ日付で入庫 +1 を入れた（A は要再確認、写し 9、範囲 8〜9） | 商品一覧を開く | A の在庫の欄に「8〜9 個」と「要再確認」（icon と文字）。在庫の列の並べ替えは写しの 9 で並ぶ | — | 50 の D-118 の注記 |
| 同じ | 在庫照会で A を検索し、詳細を開く | 一覧の在庫の欄と詳細の「在庫数」に同じ範囲と「要再確認」。状態の badge（在庫あり・在庫少・在庫切れ）は写しの 9 で決まる。詳細に要再確認の案内（理由〈b1 の要再確認の一覧と同じ文。A は `same_date_movement`〉と「棚卸しの画面で、棚の数を数えて入れ直してください」）と link「棚卸しの画面で入れ直す」 | link で棚卸しの画面の「今の数を入れる」が A で開く | 58 の D-118 の注記、b1 の `list_recheck_products` と 73 の入口（owner の判断事項 (ii)） |
| 同じ | 詳細の「商品修正」 | 「現在庫」（読取り専用）に「8〜9 個」と「要再確認」。何も変えずに離れても離脱の確認は出ない | — | 51 の「在庫数の表示」 |
| 同じ | 詳細の「在庫変動履歴」 | 見出しの現在庫に範囲と「要再確認」、その下に詳細と同じ要再確認の案内と link。表の上から「数を入れ直し（8 個、2026-10-02）」・入庫 +1（変動後在庫 9）・…の順（fresh な計数は同じ日付の入出庫より後の鍵）。各行の日付の下に保存の日時 | 元記録の link で業務記録へ進める | 44・66 の D-118 の節 |
| 商品 B に前の日の分の計数（09-30）と精算 102（09-30）の売上があり、どちらも 10-02 に入れた | B の在庫変動履歴で日付を 09-30〜09-30 に絞る | 精算 102 の売上と「数を確認（…、2026-09-30）」の 2 行（保存の日時は 10-02 だが業務日で当たる） | — | 44 の絞り込み |
| 同じ | 種別を「棚卸し」にする | 計数の行だけ（棚卸しの movement は D-118 の runtime から作られない） | — | 44 の絞り込み |
| A を棚卸しの画面で入れ直して要再確認が解けた | 商品一覧に戻る | A の在庫の欄は 1 つの値、badge なし（計数の保存の再取得。b1 の `stocktakeCountUpdate` の entry） | — | b1 への依存の 5 |

失敗と再試行: 読取りの失敗は今のとおり（商品一覧・在庫照会は error の表示、在庫変動は見出しと表が別々に失敗を出す。UI-06c-D3）。日付の絞り込みの値の形の外（例 `2026-4-7`）は route の zod が捨てる。暦の外（例 `2026-02-30`）は形を通るので BIZ の `validation` になり、表の所に今の部分障害の Alert（「在庫変動履歴の取得に失敗しました」）を出す（URL を手で直したときだけ。日付の欄は暦の外の値を作らない）。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項

製品の振舞いの判断は D-118（TD-261・TD-263〜TD-268・TD-296・TD-298）で決まり、b2 の範囲は ADR D12 が 4 画面と在庫変動の履歴に決めている。正本で一意に決まらなかった問いは次の 2 つ。

(i) 出庫と返品の画面（[62](../function-design/62-ui-manual-sale.md)〜[64](../function-design/64-ui-disposal.md)）の候補と行の「現在庫」に要再確認の範囲を出すか: 出さない B（ADR D12 の 4 画面のまま）に決まった（2026-10-23、repo 外の回答台帳 TD-397）。

(ii) 商品の画面の直し方の導線（出所: AI の review〈Plan Review round 1 の Codex #1・Opus #6〉。owner の確認の前に Human Gate の必須にしない。owner の方針〈repo 外の回答台帳 TD-401〉で推奨を既定にして進め、L3 の依頼に束ねて owner に見せる）: 既定は、在庫照会の詳細と在庫変動の見出しに、要再確認の理由と「棚卸しの画面で入れ直す」案内と link を出す（b1 の `list_recheck_products` と 73 の入口を使い、新しい command を作らない。58・66 の D-118 の注記）。変わる所は 58・66 の注記、S4 の `StockRecheckNotice`、Matrix の U10、L3-2・L3-4 で、owner が出さない形を選べばこの 4 つを外す Gated Amendment にする。

店主に新しく聞く必要がある事項: 無い。

## 起票時に起草役が決めたこと

設計の値を変えない技術の判断と配布の細部。Coordinator が覆せる。44・66 の D-118 の節に置いた（3〜6）か、本 packet と実装の PR で正本へ置く（1・2・7・8）。

1. 写しの 3 列（`stock_recheck`・`stock_range_low`・`stock_range_high`）は `ProductWithRelations` に足し、`Product`（商品の行の写像で、PLU の書出しと BIZ の JAN の照合も読む）には足さない。理由: wire に出る商品の結果は全て `ProductWithRelations`（`get_product`・`search_products` の要素・`list_low_stock` の要素・`StockDetail.product`）で、40 の「商品の get / search の結果（`ProductWithRelations` と一覧の行）」と一致する。`Product` に足すと、wire に出ない PLU と照合の読取りの SQL 6 つ（`find_by_jan_code` ほか）まで変わる。`ProductWithRelations` に足せば SQL は 4 つ。
2. 範囲の文字は b1 の `formatStockOrRange`（b1 の 73・55 の要再確認の一覧が使う部品）を唯一の部品として使い、新しい書式の関数を作らない（Coordinator の決定、Plan Review round 3）。形は b1 と同じ「{low}〜{high} {単位}」で、単位の語は後ろに 1 回（例「8〜9 個」「4〜4.5 m」。50・58 の例「4〜5」と同じ形）。商品の写しは `{ stock: stock_quantity, stock_range_low, stock_range_high, stock_unit }` で渡す（DB の CHECK で、範囲は `stock_recheck` が 1 のときだけ両端が非 null）。部品の置き場は移さない（csv-import の `PreviewStep.tsx`・`ResultStep.tsx` が同じ stocktake の file から import する先例があり、feature をまたぐ import を止める lint の規則は無い）。「要再確認」は design-system の①状態の badge（[02](../design-system/02-component-catalog.md) の badge 3 種構成。warning の tone と icon と文字。琥珀は「ちょっと待って」の注意）で、b2 の `StockRecheckBadge` を 4 画面で使う（b1 の `RecheckTargetList` の inline の badge と同じ形。b1 の画面は b2 で変えない）。
3. `list_movements` の結果は新しい型 `StockHistoryItem`（`kind` と、`movement: Option<MovementRecord>`・`count: Option<StockCountHistory>` のどちらか一方、行の `business_date`・`settlement_no`・`saved_at`）にし、`MovementRecord` の形は変えない（44 の D-118 の節）。理由: `MovementRecord` は業務記録の詳細 5 種（入庫・返品交換・手動販売・廃棄・CSV 取込み。棚卸しの記録の詳細の `movements` は b1 で外れる）の `movements` にも入っていて、並びの鍵は在庫変動の履歴でしか使わない。ADR D12・21 の「`MovementRecord` の並びの鍵」「`MovementRecord` に同じ列を返させる」は、鍵を履歴の行が持つ形に b1 の merge の後の同期で書き換える（Scope の「b1 の merge の後に同期する節」）。codebase に serde の tagged enum は無い（`rg -n 'serde\(tag' src-tauri/src` → 0 行）ので、`kind` と 2 つの Option の今の形（`CountRecoveryTarget` と同じ nullable の field）にした。
4. 並びは ADR D1 の鍵の降順で、鍵の同じ行は `saved_at`・movement を計数より前・id の昇順を足して全順序にし、全体を逆にする（44）。
5. 日付の絞り込みは業務日（`business_date`）で movement と計数の両方に当てる。今の `created_at` と `T23:59:59` の付け足しは外す。種別の絞り込みは「すべて」で両方、「棚卸し」で棚卸しの movement と計数、ほかの種類は movement だけ（44・66）。理由: 並びの軸と絞り込みの軸が違うと、絞った結果がページの中で飛び飛びになり、後から入れた記録（前の日の分の計数、翌日に取り込んだ精算）が日付の範囲から外れる。選択肢と label と route の search は変えない。
6. 計数の行の合成は BIZ（`biz::inventory_service::list` の `list_movements`）に置き、IO は並べない 2 つの読取りを返す。初期在庫の印の SQL の式は `inventory_repo` の 1 つの定数にして、求め直しの読取りと共有する（44。下の「起票時に確かめた 2 つ」の 2）。
7. 在庫変動の表の「日付」の列は業務日（Z004 の行は「（精算 {n}）」）と、その下の保存の日時。計数の行は「種別」に 65 の文、「増減」と「変動後在庫」は「—」、「元記録」は棚卸しの記録の link（明細を参照する計数）か「元記録なし」（66）。表は在庫変動の画面だけの新しい component（`StockHistoryTable`）にし、`MovementTable` は業務記録の詳細 6 画面（`rg -n 'MovementTable' src --glob '!*.test.*'` で `CsvImportRecordDetailPage`・`ManualSaleRecordDetailPage`・`ReceivingRecordDetailPage`・`DisposalRecordDetailPage`・`ReturnRecordDetailPage`・`StocktakeRecordDetailPage` が import する。棚卸しの記録の詳細の分は b1 で外れる）が `MovementRecord` の一覧で使い続けるので変えない。種別の label・増減・元記録の link の書き方は `MovementTable.tsx` から export して共有する（同じ書き方を 2 つ持たない）。
8. L3 の起点は b1 の L3 の後の DB（下の Test Plan）。Coordinator が b1 の L3 の直後に DB の写しを repo の外へ控え（アプリを止めた後に file を写すだけ。消さない）、b2 の L3 の前に読取りだけの query で状態を確かめる。

## 起票時に確かめた 2 つ（b1 の packet の「後続の lane」の b2 の項）

1. 在庫少の判定・並べ替え・ホームの件数は写しの値のまま: **正本で一意に決まる**（owner の判断事項でない）。[50](../function-design/50-ui-product-list.md) の D-118 の注記「並べ替えと絞り込みは写しの値で行う（今のまま）。ホーム（53）の在庫切れ・在庫少の件数も写しの値で数える（今のまま。要再確認は棚卸しの画面の一覧で扱う）」と、[58](../function-design/58-ui-stock-inquiry.md) の D-118 の注記「在庫少の判定と並べ替えは写しの値で行う（今のまま）」。今の code も写しだけを読む: `src-tauri/src/db/product_repo.rs` の `list_low_stock_products` の SQL（`p.stock_quantity <= ?`）、`src/features/home/lib/count-stock-status.ts` の `countStockStatus`（`stock_quantity`）、`src/features/stock-inquiry/lib/filter-low-stock-list.ts` の並べ替え（`stock_quantity` の差）、`src/features/stock-inquiry/lib/derive-stock-state.ts` の `deriveStockState`。b2 はこれらを変えず、Matrix の U2 が範囲の端でなく写しで状態が決まることを止める。
2. 在庫変動の履歴の計数の行の合成を IO と BIZ のどちらに置くか: **正本で一意に決まらない**（65 は「読取りの合成。計数は `stock_counts` から読む」、21・31 は置き場を書かない）。製品の振舞いを変えない技術の判断なので、**BIZ** と決めた（上の 6、44 の D-118 の節）。理由: 鍵のうち、fresh な計数の `after_settlement`（1）と n が NULL のときの -1、初期在庫の -1 は ADR D1 の規則で、IO の SQL の `UNION ALL` と `ORDER BY` に写すと、同じ規則を求め直しの Rust（`biz::stock_ledger`）と SQL の 2 か所に持つ（ADR の Rejected Options が写しを持たない案を退けた理由と同じ）。BIZ は 1 商品の履歴を全部読んでから並べるので読む量は 1 商品の行の数に比例するが、単店舗の 1 商品の行は数千までの見込みで、ページの上限 200 の読取りと同じ桁に収まる（越えたときの移り先は 44 に書いた）。

## b1 への依存

b1 の Plan Commit の版の名前で書く。b1 の実装で名前・形が変われば、b1 の merge の後の同期で本 packet・Matrix・44・66 を当て直す。

1. 生成型 `StockCountKind`（`"default" | "fresh"`。b1 の packet の Boundary / Wire Contract、[42](../function-design/42-cmd-sales-stocktake.md)）を `StockCountHistory.count_kind` に使う。
2. 計数の記録: b1 の `insert_stock_count`（21）が `stock_counts` に書く `counted_at`（`%Y-%m-%dT%H:%M:%S` の PC の時刻を前提に、`saved_at` の比べと表示に使う。movement の `created_at` は今の `insert_movement` が同じ形で書く: `src-tauri/src/db/inventory_repo.rs` の `chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")`）と `stocktake_item_id`（棚卸しの画面から保存した計数だけが持つ。b1 の S4）。形が違えば、b2 の BIZ が `saved_at` を同じ形に揃えてから比べる。
3. 初期在庫の印: b1 の S2 が `list_ledger_movements_for_product` の印を `reference_type IS NULL AND movement_type = 'receiving'` にする（ADR D9）。b2 はこの式を `inventory_repo` の 1 つの定数にして履歴の読取りと共有する（b1 の AC2 の「SQL の行が 1」は、定数の行が 1 つなので変わらない）。
4. Z004 の movement: b1 の Z004 の確定が `phase = 1`・`settlement_no` 付きで書く（31、b1 の S5）。b2 の並びの `(d_n, 1, n, 0)` はこの列から作る。
5. 再取得: b1 の S8 が D-052 の SSOT（`src/lib/invalidation-contract.ts`）の `stocktakeCountUpdate` に写しの consumer（商品一覧・在庫少・在庫照会・在庫変動・商品フォーム）を足し、写しを書く 12 の entry に要再確認の一覧を足す（UI_TECH_STACK の D-118 の段落）。b2 の 4 画面と在庫変動の計数の行はこの再取得で新しくなる。b2 は SSOT を変えない（Matrix の U9 が依存を確かめる）。
6. 棚卸しの記録の詳細: b1 の S8 が `StocktakeRecordDetailPage` の中身を変える。route `/stocktake/records/{id}` と `resolve_movement_source` の棚卸しの行（`src-tauri/src/biz/inventory_service/list.rs`）は変わらない前提で、計数の行の元記録の link に使う。
7. 範囲と理由の文の部品: b1 の `src/features/stocktake/lib/stocktake-formatters.ts` の `formatStockOrRange`（引数 `{ stock, stock_range_low, stock_range_high, stock_unit }`）と `formatRecountReasons`（`RecountReason` の配列から文の配列。理由 1・2 は同じ文で 1 つにまとめる）。b1 head `3e6a8568` で確かめた。名前・引数が変われば当て直す（使い方は上の「起票時に起草役が決めたこと」の 2）。
8. `CmdError` の形: b1 が `stocktake_recovery` を足す。b2 の TS の test のうち 3 file（`src/features/backup-restore/BackupRestorePage.flow.test.tsx`・`src/features/products/ProductFormPage.test.tsx`・`src/features/stock-movements/StockMovementsPage.test.tsx`）は b1 の `CmdError` の mock の追随（b1 の Matrix の T7）と同じ file で、同期の後の形に合わせる。
9. 商品フォーム: b1 の S8 が `src/features/products/ProductFormPage.tsx` に `recount_required` の拒否と導線を足し、`ProductFormPage.test.tsx` に test を足す。b2 は `src/features/products/components/ProductForm.tsx` の「現在庫」の欄と `ProductFormPage.test.tsx` の表示の test を触る（同じ test の file の別の case）。
10. 生成物: `src/lib/bindings.ts` は b1 の merge の後に b2 の head で再生成する（手で解かない。D-098）。
11. L3 の起点: b1 の L3 の後の DB（b1 の packet の Test Plan の L3-13 の「後の状態」。b1 の packet が「次の L3（b2）の起点にも使える」と書く）。
12. 正本: b1 が書き換えた ADR D12・21・40・51・65 の節と、b1 の実装の S9 が書き換える 20・SCREEN_DESIGN・10 の D-118 の節を、b2 の PR で現行の記述に直す（下の Scope の「b1 の merge の後に同期する節」）。
13. 要再確認の案内（owner の判断事項 (ii)）: b1 の `list_recheck_products`（42・35。`CountRecoveryTarget` の `product_code`・`recount_reasons`）、要再確認の一覧の query と key `queryKeys.stocktake.recheckList()`（b1 の S8。写しを書く entry が invalidate するので、写しと同じ時に新しくなる。b1 head `3e6a8568` では `StocktakePage.tsx` の中の inline の `useQuery`〈`recheckQuery`、`staleTime: 0`〉で export されていないので、b2 が同じ key と `queryFn` のまま `src/features/stocktake/hooks/useRecheckProducts.ts` へ切り出し、`StocktakePage.tsx` と `StockRecheckNotice` がそれを使う）、理由の文（b1 の `formatRecountReasons` を共有し、2 つ持たない。上の 7）、73 の入口（`/stocktake/` の search の `product`）。`returnTo` は渡さない（73 は `/products/{同じ商品コード}/edit` だけを受けるので、保存の後は棚卸しの画面に留まる）。

## この変更で不要になるもの

- code: `src-tauri/src/db/inventory_repo.rs` の `list_movements`（`created_at` の降順・`T23:59:59` の付け足し・SQL の `LIMIT`/`OFFSET`）を、並べない 2 つの読取りに置き換えて外す。`MovementRecord`・`MovementQuery`・`MovementSourceLink`・`resolve_movement_source` は残す（業務記録の詳細と計数の元記録が使う）。
- b1 だけの版の表示の欠けの注記（b1 の packet の「各 lane の配布の単位としての安全」）: ADR D12 の b1 の版の「b1 の後・b2 の前の版では、商品の画面は写しの値だけを出し…初めて店へ入れる版は b2 まで含む」の文と、ADR の Status の「(b) は実装済みの契約ではない」。b2 で欠けが無くなるので、同期で外して現行の記述にする（b1 の packet の「残りの危険」の 1 つ目の項は、b1 の closeout で archive に移る packet の記録なので書き換えない）。
- 「proposed・未実装」の注記の b2 の分: 20・21・40・44・50・51・58・65・66・SCREEN_DESIGN の D-118 の節と、[ARCHITECTURE](../ARCHITECTURE.md) の D-118 の節（見出しの「(b) は proposed・未実装」と本文の「表のほかの責務は (b)（proposed・未実装）」。b1 の S9 で「b2 の分は proposed のまま」と残る所を含む）。
- `list_movements` の今の振舞いを引く記述:
  - [44](../function-design/44-cmd-inventory.md): §23.9 の `inventory_repo::list_movements` の SQL 概要と IO の signature、§23.8 の CMD の signature と現行の出力型（44 の D-118 の節で置き換えを宣言した）、§23.1 のモジュール構成の `inventory_repo.rs -- list_movements を追加` の行（履歴の 2 つの読取りに）、§23.10 の BIZ の `list_movements` の signature（`PaginatedResult<MovementRecord>` を `PaginatedResult<StockHistoryItem>` に）と「repo から得た movement 行に対して…`source` を補完する」の説明（計数の `source` の補い方と並び・絞り込み・ページの順を足した処理に）。
  - [74](../function-design/74-ui-operation-logs.md) と [20](../function-design/20-io-product-repo.md) の、UI-11c-D2 の対比として `list_movements` の `T23:59:59` を引く文: 20 の `list_operation_logs` の設計判断（UI-11c-D2 との差分メモ）、74 の UI-11c-D2 の行と §74.4.3「既存パターンとの差分」の段落。日付で絞る今の IO で `T23:59:59` を付けるのは `inventory_repo.rs` の `list_movements` だけ（`rg -n 'T23:59:59' src-tauri/src` の実装の行は `inventory_repo.rs` の 2 行。ほかは test）で、入出庫の記録（`list_inventory_records`・`list_receiving_records` ほか）は日付の列を `<= ?` で比べる（`src-tauri/src/db/disposal_repo.rs` の `list_inventory_records` の `{date_expr} <= ?`、`receiving_repo.rs` の `list_receiving_records` の `rr.receiving_date <= ?`）ので、替える例は無い。対比は撤去し、在庫変動は b2 で業務日の両端を含む比べに変わったと 1 文で書く。74 §74.17 の比較の表の「期間 predicate」の行は、`StockMovementsPage` の列を「業務日の両端を含む（b2）」、`InventoryRecordsPage` の列の「同左」を実際の形「日付の列の両端を含む（`<= date_to`）」に。§74.16 の非目的の「movement/records 画面の期間 predicate を JST inclusive/exclusive に統一」の行は、残る対象（records）だけにする。
  - [66](../function-design/66-ui-stock-movements.md): 主動線の「日時降順」と §66.5 の「日時」の列と表示規則、§66.4 の `PaginatedResult<MovementRecord>`（D-118 の注記の形へ）。
  - [65](../function-design/65-inventory-record-traceability.md): §65.7.1 の command の表の `listMovements(query)` の行の「商品別 movement。既存 contract を拡張し…」（movement と計数の履歴に。44 を引く）。
  - [cmd-task-specs](../architecture/cmd-task-specs.md) の `list_movements` の出力。
  - decision-log の該当の決定（D-037 ほか）の本文は append-only の記録なので書き換えない。
- 不要にならないと確かめたもの: 整合性の画面と操作ログの画面の `formatStockCopy`（2 つの private の関数。`rg -n 'function formatStockCopy' src` → `src/features/integrity-check/IntegrityCheckPage.tsx`・`src/features/operation-logs/OperationLogsPage.tsx`）は、写しの 4 項目を診断のために並べる別の書き方（75・74）で、b2 の範囲の部品に置き換えない。

## Scope

本 lane が変える file と目的。実装は S1 → S5 の順に積む。S1〜S3 は 1 つの Rust の検証の単位（S3 の wire の変更は S4 の画面と bindings で揃う）、S4 は frontend の検証の単位、S5 は docs と生成物。既存の test の移し方は Matrix の「既存の test の移し方」（T1〜T3）を引く。

### 本 commit で変えた正本（plan-first）

- [44](../function-design/44-cmd-inventory.md) の `list_movements` に D-118 の b2 の節（結果の型・並び・絞り込み・合成の置き場と理由・置き換える現行の振舞い）。
- [66](../function-design/66-ui-stock-movements.md) の冒頭の D-118 の注記に、表の行の順・「日付」の列・計数の行・種別と日付の絞り込み・見出しの現在庫の範囲と badge。
- Plan Review round 1 の是正（Review Response 参照）: 66 の計数の行の {数} の書き方を `formatStockDisplay` の形にし、[58](../function-design/58-ui-stock-inquiry.md) の D-118 の注記と 66 の見出しの項に要再確認の案内（owner の判断事項 (ii)）を足した（58 は b1 の branch が書き換えていない）。

### b1 の merge の後に同期する節（本 commit では直さない）

b1 の branch が書き換えた節と、b1 の実装の S9 が書き換える節。b2 の実装の PR で、b1 の merge の後の版を起点に直す。

- [ADR](../adr/2026-10-16-stocktake-count-placement.md): Status の (b) の実装の状態、D12 の「(b) を 2 つの lane に分けた」の段落の「b1 の後・b2 の前の版では…」の文（外す）と「`MovementRecord` の並びの鍵」（履歴の行の並びの鍵に）、(a) と (b) の境の細部の「`MovementRecord` の並びの鍵」（同じ）。
- [21](../function-design/21-io-inventory-repo.md): D-118 の節の「`MovementRecord` の列」「`MovementRecord` に同じ列を返させるのは (b)」を、履歴の 2 つの読取りと初期在庫の印の定数に（44 を引く）。
- [40](../function-design/40-cmd-product.md): 商品の結果の 3 列の文を現行に（`ProductWithRelations` に足したこと）。
- [51](../function-design/51-ui-product-form.md): 「在庫数の表示」の項を現行に（修正の画面の読取り専用の「現在庫」）。
- [65](../function-design/65-inventory-record-traceability.md): D-118 の節の見出しの「proposed・未実装」と 1 つ目・2 つ目の項を現行に（44・66 を引く）。§65.7.1 の command の表の `listMovements(query)` の行（上の「この変更で不要になるもの」）。
- [20](../function-design/20-io-product-repo.md): D-118 の節の商品の読取りの文（b1 の S9 の後に「b2 で入った」）と、`list_operation_logs` の設計判断（UI-11c-D2 との差分メモ）の `list_movements` の対比（撤去する。上の「この変更で不要になるもの」）。
- [ARCHITECTURE](../ARCHITECTURE.md): D-118 の節の見出しと本文の (b) の実装の状態（b1 の S9 の後に b2 の分として残る「proposed・未実装」を現行に。UI の行の「範囲の表示」と 58・66 は b2 で入る）。
- [SCREEN_DESIGN](../SCREEN_DESIGN.md): D-118 の節の見出しの状態と 51・58・66 の行。
- [10](../function-design/10-common-rules.md): SPEC-UNIT-D3 の「表示する所」の表の 6（在庫変動の表に計数の行の数を足す。計数の行の数は在庫の欄と同じ `formatStockDisplay` の形で、個数にも単位の語を付ける〈66〉。b1 で棚卸しの記録の詳細の「関連する在庫変動」が外れて経路が 6 になる）と、商品の画面の在庫の欄の範囲の行（b1 の `formatStockOrRange` の形「8〜9 個」「4〜4.5 m」。単位の語は後ろに 1 回。表の 8 の整合性チェックの書き方とは別）。
- [Plans](../Plans.md): `## 次の行動` と `## 導入の前の関門` の 2. の pointer の句（b1 の版の「b2 は同じ packet の『後続の lane』」を本 packet へ）。lane の行は足さない（D-097）。
- [FUNCTION_DESIGN](../FUNCTION_DESIGN.md)・[cmd-task-specs](../architecture/cmd-task-specs.md): UI-06c の行と CMD-06 の `list_movements` の出力。

### S1 IO（`product_repo`・`inventory_repo`）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 商品の結果の写し | `ProductWithRelations` に `stock_recheck: bool`・`stock_range_low: Option<i64>`・`stock_range_high: Option<i64>`。4 つの SELECT（`find_by_product_code`・`search_products`・`get_stock_detail`・`list_low_stock_products`）に 3 列を足し、`row_to_product_with_relations` が読む（`get_stock_detail` の `last_receiving_date`・`last_sale_date` の列の位置がずれる） | `src-tauri/src/db/product_repo.rs` | Matrix の P1・P2 |
| 履歴の読取り | `list_movements` を外し、`pub(crate)` の 2 つ: 商品と絞り込み（業務日の範囲・種別）に当たる `is_voided = 0` の movement の行（今の列 + `business_date`・`settlement_no`・初期在庫の印）と、商品と業務日の範囲に当たる計数の行（`stock_counts` の id・`count_value`・`count_kind`・`business_date`・`settlement_no`・`counted_at`、`products` を JOIN した `stock_unit`、`stocktake_items` を LEFT JOIN した `stocktake_id`）。どちらも並べない。初期在庫の印の式を定数にし、`list_ledger_movements_for_product` と共有（b1 への依存の 3） | `src-tauri/src/db/inventory_repo.rs`、`src-tauri/src/db/mod.rs`（re-export） | Matrix の H1〜H8 は BIZ の test で見る。定数の共有は Final Review の code 読み（Matrix の Residual Test Gaps） |

### S2 BIZ（`inventory_service::list`）

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `list_movements` | ページの検査（今のまま）→ 日付の検査（`validate_business_date`）→ 2 つの読取り → 種別の絞り込みで計数を含めるか決める → 鍵で並べる（44）→ 件数 → ページを切る → movement の `source`（今の `resolve_movement_source`）と計数の `source`（棚卸しの行）を補う。戻り値は `PaginatedResult<StockHistoryItem>` | `src-tauri/src/biz/inventory_service/list.rs`、`mod.rs`・`src-tauri/src/biz/mod.rs`（re-export） | Matrix の H1〜H9 |

### S3 CMD と wire

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `list_movements` | 戻り値の型を `PaginatedResult<StockHistoryItem>` に（CMD は薄いまま） | `src-tauri/src/cmd/inventory_cmd.rs` | Matrix の W2、T1 の CMD の test |
| 生成の enum の test | `StockHistoryKind` を response の enum の round-trip の一覧に | `src-tauri/src/finite_enum_contract_tests.rs` | Matrix の W1 |
| bindings | `cd src-tauri && cargo run --bin generate_bindings` で再生成 | `src/lib/bindings.ts`（生成物） | AC1・AC2 |

### S4 画面

| 対象 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| 範囲と badge の部品 | 「要再確認」の①状態の badge（warning の tone・icon・文字）と、商品の写しを受けて在庫の文字（b1 の `formatStockOrRange`。新しい書式の関数を作らない）と、`stock_recheck` なら badge を出す 1 つの部品（上の「起票時に起草役が決めたこと」の 2、b1 への依存の 7） | `src/features/stock-inquiry/components/StockRecheckBadge.tsx`（新設） | Matrix の U8 |
| 要再確認の案内（owner の判断事項 (ii)） | 商品コードと写しの `stock_recheck` を受け、要再確認のときだけ warning の Alert（icon と文字）を出す部品: 題「数え直しが必要です」、本文は理由（`useRecheckProducts` で読んだ当該の target の `recount_reasons` を b1 の `formatRecountReasons` で）と「棚卸しの画面で、棚の数を数えて入れ直してください。」、link「棚卸しの画面で入れ直す」（`/stocktake/` の search `product` に商品コード、`returnTo` なし）。target が無い・読込み中・読取りの失敗では理由を出さず、案内と link だけ（詳細の表示を止めない）。在庫照会の詳細と在庫変動の見出しの 2 か所で使う（b1 への依存の 13） | `src/features/stock-inquiry/components/StockRecheckNotice.tsx`（新設） | Matrix の U10、U3・U5 |
| 要再確認の一覧の hook | b1 の `StocktakePage.tsx` の inline の `recheckQuery`（key `queryKeys.stocktake.recheckList()`・`listRecheckProducts`・`staleTime: 0`）をそのまま hook に切り出し、`StocktakePage.tsx` はその hook を呼ぶ（振舞いを変えない。b1 への依存の 13） | `src/features/stocktake/hooks/useRecheckProducts.ts`（新設）、`src/features/stocktake/StocktakePage.tsx` | b1 の棚卸しの画面の今の test が期待を変えずに通る（AC3 と targeted の `src/features/stocktake`、`git diff --stat` で b1 の `src/features/stocktake/*.test.tsx` の差分が 0）、U10 |
| 商品一覧 | 在庫の列に部品（並べ替えは今のまま写し） | `src/features/products/components/ProductTable.tsx` | Matrix の U1 |
| 在庫照会 | 一覧の在庫の列と詳細の「在庫数」に部品。詳細に要再確認の案内。状態の badge は写しのまま | `src/features/stock-inquiry/components/ProductListTable.tsx`・`StockDetailContent.tsx` | Matrix の U2・U3 |
| 商品フォーム | 修正の画面の読取り専用の「現在庫」に範囲の文字（`productToFormValues` の `initialStock` を `formatStockOrRange` で。修正では検証も送信もしない）と、欄の label の隣に badge。badge の要否は `ProductForm` の新しい prop `stockRecheck?: boolean` で受け、`ProductFormPage.tsx` が `currentProduct?.stock_recheck ?? false` を渡す（今の `isDiscontinued` と同じ渡し方。`values` は form の入力の値だけで、写しの状態を入れない）。登録の画面は変えない | `src/features/products/lib/product-form-request.ts`、`src/features/products/components/ProductForm.tsx`、`src/features/products/ProductFormPage.tsx` | Matrix の U4 |
| 在庫変動 | 見出しの「現在庫」に部品と、その下に要再確認の案内。表を新しい `StockHistoryItem` の表で描く（66 の D-118 の注記の表の行の順・「日付」の列・計数の行。計数の行の数は行の `count.stock_unit` で描き、商品情報の取得が失敗しても単位が付く）。`MovementTable` は業務記録の詳細のために残し、種別の label・増減・元記録の link の書き方を export して共有する（上の「起票時に起草役が決めたこと」の 7）。hook の型 | `src/features/stock-movements/StockMovementsPage.tsx`・`components/StockHistoryTable.tsx`（新設）・`components/MovementTable.tsx`（export だけ）・`hooks/useStockMovements.ts`・`lib/movement-formatters.ts` | Matrix の U5〜U7・U9。業務記録の詳細の `MovementTable` の今の test は期待を変えない（T3） |
| mock の追随 | `ProductWithRelations` の literal に 3 列（共有の fixture 2 と literal を持つ 5 file）、`listMovements` の mock を `StockHistoryItem` に | 下の予定 path の TS の test の file | Matrix の T2・T3 |

### S5 docs・生成物

- 上の「b1 の merge の後に同期する節」の全部（[ARCHITECTURE](../ARCHITECTURE.md) の D-118 の節を含む）と、44・66 の D-118 の節の「proposed・未実装」を現行に。[50](../function-design/50-ui-product-list.md)・[58](../function-design/58-ui-stock-inquiry.md) の D-118 の注記を現行に（部品は 66 を引く）。[44](../function-design/44-cmd-inventory.md) の §23.1 のモジュール構成の `inventory_repo.rs` の行、§23.8 の CMD の signature と出力型、§23.9 の IO の signature と SQL 概要、§23.10 の BIZ の `list_movements` の signature と `source` の補完の説明を、D-118 の節の形（履歴の 2 つの読取り、`PaginatedResult<StockHistoryItem>`、BIZ の合成の順）に（D-118 の節を本文へ移す）。[74](../function-design/74-ui-operation-logs.md) と [20](../function-design/20-io-product-repo.md) の UI-11c-D2 の `list_movements` の対比は、例を替えずに撤去し、在庫変動は b2 で業務日の両端を含む比べに変わったと 1 文で書く（`T23:59:59` を付けるのは `list_movements` だけで、入出庫の記録は日付の列を `<= ?` で比べる。上の「この変更で不要になるもの」）。74 §74.17 の比較の表の「期間 predicate」の行と §74.16 の非目的の行も同じ節のとおりに。66 の §66.1 の主動線（2. の「日時降順の movement」）・関数要求（「各 movement の日時」を、movement と計数の行の業務日と保存の日時に）・処理ステップの 5.（「`MovementTable` に渡し、日時、…」を `StockHistoryTable` と「日付」の列に）・エラーハンドリングの `MovementRecord.source` が null の項と UI-06c-D6（movement の `source` と計数の `count.source` の両方に）、§66.4・§66.5 を D-118 の注記の形に（注記を本文へ移す）。[02](../design-system/02-component-catalog.md) の①状態の tone family の表の warning の行に「要再確認」（`StockRecheckBadge`）。
- 生成物: `src/lib/bindings.ts`（S3）、`docs/function-design/90-traceability.md`（test の REQ の参照が増えるので `cd src-tauri && cargo run --bin generate_traceability`。手で直さない）。

### 節の所有表（並走の lane と重なる file）

`comm -12` の結果（本 packet の予定 path 62 と次の各 path の集合。2026-10-24 の Plan Review round 3 の是正で、予定 path を足し外した〈`src/features/products/ProductFormPage.tsx`・`src/features/stock-inquiry/components/StockRecheckBadge.test.tsx`・`src/features/stocktake/StocktakePage.tsx`・`src/features/stocktake/hooks/useRecheckProducts.ts` を足し、`src/features/stock-inquiry/lib/format-stock-display.ts` と同じ名の test を外した〉後に全部を取り直した。差分は `git diff --name-only --no-renames eed07d37...<commit>` を `LC_ALL=C sort` したもの、予定 path も同じく sort した。`origin/main` は `eed07d37`）:

- b1（PR #188）: b1 の Plan Commit `aba17e2e` の packet の「予定 path の一覧」（148）とは 23: `docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md`・`docs/Plans.md`・`docs/SCREEN_DESIGN.md`・`docs/adr/2026-10-16-stocktake-count-placement.md`・`docs/architecture/cmd-task-specs.md`・`docs/function-design/10-common-rules.md`・`docs/function-design/20-io-product-repo.md`・`docs/function-design/21-io-inventory-repo.md`・`docs/function-design/40-cmd-product.md`・`docs/function-design/51-ui-product-form.md`・`docs/function-design/65-inventory-record-traceability.md`・`docs/function-design/90-traceability.md`・`src-tauri/src/biz/mod.rs`・`src-tauri/src/cmd/inventory_cmd.rs`・`src-tauri/src/db/inventory_repo.rs`・`src-tauri/src/db/mod.rs`・`src-tauri/src/db/product_repo.rs`・`src-tauri/src/finite_enum_contract_tests.rs`・`src/features/products/ProductFormPage.test.tsx`・`src/features/products/ProductFormPage.tsx`・`src/features/stocktake/StocktakePage.tsx`・`src/lib/bindings.ts`。ほかに b1 の packet が一覧の外に数える `CmdError` の mock の 34 file のうち 3（b1 への依存の 8）。参考に、b1 の実装の head `3e6a8568`（差分 176）とは 27: 上の 23 のうち `src-tauri/src/cmd/inventory_cmd.rs` を除く 22 と、`src-tauri/src/biz/inventory_service/mod.rs`・`src/features/backup-restore/BackupRestorePage.flow.test.tsx`・`src/features/home/HomePage.test.tsx`・`src/features/home/hooks/useHomeSummary.test.tsx`・`src/features/stock-movements/StockMovementsPage.test.tsx`（b1 の実装が一覧の外で触った file。どれも b2 の触る所は別の行か mock の field）。b1 は是正中で head が進むので、L3 を頼む時点の head で取り直す。
- PR #172（`9f02ddd7`、差分 9）: 重なり 0。
- PR #190（`b8b201a5`、差分 7）: `docs/Plans.md` の 1（#190 は `## 導入の前の関門` の 3.、b2 は 2. と `## 次の行動` の句）。
- PC の外の控え (b)（branch `agent/backup-offsite-runtime-b`）: 起票中で未 push（remote は `eed07d37` のままで差分 0）。予定 path が出たら、L3 を頼む時点の head で取り直す。

b1 とは stacked で、b2 は b1 の merge の後に `origin/main` を単段で取り込み、b1 の版を起点に直す（重なる正本の節は上の「b1 の merge の後に同期する節」が b2 の担当）。生成物（`src/lib/bindings.ts`・`docs/function-design/90-traceability.md`）は取り込んだ後に再生成する（D-098）。

| file | 並走の lane が所有する節 | 本 lane が触る所 | merge の後に揃える所 |
|---|---|---|---|
| `src-tauri/src/db/inventory_repo.rs` | b1: 計数の保存・精算の読取り・初期在庫の印（S2） | `list_movements` の置き換えと初期在庫の印の定数（S1） | b1 の印の式を定数に移す（b1 への依存の 3） |
| `src-tauri/src/db/product_repo.rs` | b1: 共有 JAN の読取り（S2） | `ProductWithRelations` と 4 つの SELECT（S1） | 別の関数 |
| `src-tauri/src/cmd/inventory_cmd.rs`・`src-tauri/src/biz/mod.rs`・`src-tauri/src/db/mod.rs`・`src-tauri/src/finite_enum_contract_tests.rs` | b1: `CmdError` の literal・re-export・新しい enum の行 | `list_movements` の型・re-export・`StockHistoryKind` の行 | 別の行。一覧の行は両方を残す |
| `src/features/products/ProductFormPage.test.tsx` | b1: `recount_required` の test（U12）と `CmdError` の mock | 修正の画面の「現在庫」の表示の test（U4）と mock の 3 列 | 別の case |
| `src/features/products/ProductFormPage.tsx` | b1: `recount_required` の拒否と導線（S8） | `ProductForm` へ `stockRecheck` を渡す 1 つの prop（S4） | 別の行 |
| `src/features/stocktake/StocktakePage.tsx` | b1: 棚卸しの画面の全体（S8） | inline の `recheckQuery` を `useRecheckProducts` の呼出しに置き換える（S4。振舞いは変えない） | b1 の merge の後の版を起点に置き換える |
| ADR・21・40・51・65・20・SCREEN_DESIGN・ARCHITECTURE・10・FUNCTION_DESIGN・cmd-task-specs | b1: D-118 の節（b1 の分）と停止の記述 | b2 の分の文（上の「b1 の merge の後に同期する節」） | b1 の版を起点に b2 の分だけを直す |
| `docs/Plans.md` | b1: 関門 2. と `## 次の行動` の句。#190: 関門 3. | 関門 2. と `## 次の行動` の句の b2 の pointer（同期の後） | 別の項目 |
| `src/lib/bindings.ts`・`docs/function-design/90-traceability.md` | 生成物 | 生成物 | 取り込んだ後に再生成 |

### 予定 path の一覧（`comm -12` の入力）

64 path（Rust 9・TS 32・docs 23）。所有表の突き合わせは L3 を頼む時点の head で取り直す（D-124）。

```
docs/ARCHITECTURE.md
docs/FUNCTION_DESIGN.md
docs/Plans.md
docs/SCREEN_DESIGN.md
docs/adr/2026-10-16-stocktake-count-placement.md
docs/architecture/cmd-task-specs.md
docs/architecture/io-task-specs.md
docs/architecture/ui-task-specs.md
docs/design-system/02-component-catalog.md
docs/function-design/10-common-rules.md
docs/function-design/20-io-product-repo.md
docs/function-design/21-io-inventory-repo.md
docs/function-design/40-cmd-product.md
docs/function-design/44-cmd-inventory.md
docs/function-design/50-ui-product-list.md
docs/function-design/51-ui-product-form.md
docs/function-design/58-ui-stock-inquiry.md
docs/function-design/65-inventory-record-traceability.md
docs/function-design/66-ui-stock-movements.md
docs/function-design/74-ui-operation-logs.md
docs/function-design/90-traceability.md
docs/plans/2026-10-23-stocktake-count-runtime-b2.md
docs/plans/test-matrices/2026-10-23-stocktake-count-runtime-b2.md
src-tauri/src/biz/inventory_service/list.rs
src-tauri/src/biz/inventory_service/mod.rs
src-tauri/src/biz/l3_b2_db.rs
src-tauri/src/biz/mod.rs
src-tauri/src/cmd/inventory_cmd.rs
src-tauri/src/db/inventory_repo.rs
src-tauri/src/db/mod.rs
src-tauri/src/db/product_repo.rs
src-tauri/src/finite_enum_contract_tests.rs
src/features/backup-restore/BackupRestorePage.flow.test.tsx
src/features/home/HomePage.test.tsx
src/features/home/hooks/useHomeSummary.test.tsx
src/features/inventory-records/ReturnToFlow.test.tsx
src/features/products/ProductFormPage.test.tsx
src/features/products/ProductFormPage.tsx
src/features/products/components/ProductForm.tsx
src/features/products/components/ProductTable.test.tsx
src/features/products/components/ProductTable.tsx
src/features/products/lib/product-form-request.test.ts
src/features/products/lib/product-form-request.ts
src/features/products/lib/test-fixtures.ts
src/features/stock-inquiry/components/ProductListTable.test.tsx
src/features/stock-inquiry/components/ProductListTable.tsx
src/features/stock-inquiry/components/StockDetailContent.test.tsx
src/features/stock-inquiry/components/StockDetailContent.tsx
src/features/stock-inquiry/components/StockRecheckBadge.test.tsx
src/features/stock-inquiry/components/StockRecheckBadge.tsx
src/features/stock-inquiry/components/StockRecheckNotice.test.tsx
src/features/stock-inquiry/components/StockRecheckNotice.tsx
src/features/stock-inquiry/lib/test-fixtures.ts
src/features/stock-movements/StockMovementsPage.test.tsx
src/features/stock-movements/StockMovementsPage.tsx
src/features/stock-movements/components/MovementTable.tsx
src/features/stock-movements/components/StockHistoryTable.test.tsx
src/features/stock-movements/components/StockHistoryTable.tsx
src/features/stock-movements/hooks/useStockMovements.ts
src/features/stock-movements/lib/movement-formatters.test.ts
src/features/stock-movements/lib/movement-formatters.ts
src/features/stocktake/StocktakePage.tsx
src/features/stocktake/hooks/useRecheckProducts.ts
src/lib/bindings.ts
```

`src-tauri/src/biz/l3_b2_db.rs` は予備の path（下の Test Plan の「L3 の起点の DB」の 3. で、b1 の L3 の後の DB と写しがどちらも無いときにだけ作る、`#[cfg(test)]` の DB の生成。作らなければ PR の差分に出ない）。

## Non-scope

- 上の Goal の「非目的」の全部。
- 要再確認の案内（理由と直し方）を商品一覧・在庫照会の一覧・商品フォームの修正に出すこと（一覧は行の badge までで、行を開いた在庫照会の詳細が案内を出す。商品フォームの修正は在庫を変えない画面で、詳細から開く。owner の判断事項 (ii)）。
- 在庫変動の種別の絞り込みに「数の確認」の選択肢を足すこと（「棚卸し」で計数が出る。route の search の値と `MovementQuery` を変えない）。
- 在庫照会の要再確認での絞り込み・並べ替え（50・58 は写しの値で並べると決めている）。

## Acceptance Criteria

- AC1（商品の結果の写し）: `(cd src-tauri && cargo test --lib db::product_repo)` が全 PASS し、Matrix の P1・P2 の test 名が `rg -n 'fn <名前>' src-tauri/src/db/product_repo.rs` で 1 件ずつ見つかる。`rg -U -c 'export type ProductWithRelations = \{[^}]*stock_recheck: boolean' src/lib/bindings.ts` が 1。
- AC2（履歴）: `(cd src-tauri && cargo test --lib -- biz::inventory_service cmd::inventory_cmd finite_enum_contract_tests)` が全 PASS し、Matrix の H1〜H9・W1・W2 の test 名が見つかる。`rg -c '^export type (StockHistoryItem|StockHistoryKind|StockCountHistory) =' src/lib/bindings.ts` が 3。`rg -c 'ORDER BY created_at DESC, id DESC' src-tauri/src/db/inventory_repo.rs` と `rg -c 'T23:59:59' src-tauri/src/db/inventory_repo.rs` がどちらも exit 1（置き換えた）。`rg -U -c 'export type MovementRecord = \{[^}]*business_date' src/lib/bindings.ts` が exit 1（`MovementRecord` の形は変えない）。
- AC3（画面）: `npm test -- src/features/stock-inquiry src/features/products src/features/stock-movements src/features/inventory-records src/features/home src/features/backup-restore src/features/stocktake` が全 PASS し（`src/features/stocktake` は S4 の hook の切出しで b1 の棚卸しの画面の test が期待を変えずに通ることを見る）、Matrix の U1〜U10 の test 名が見つかる。
- AC4（gate）: repo の root を作業ディレクトリにして、`(cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test)`、`(cd src-tauri && cargo run --bin generate_bindings) && git diff --exit-code -- src/lib/bindings.ts`、`(cd src-tauri && cargo run --bin generate_traceability -- --check)`、`(cd src-tauri && cargo test --test design_compliance_test)`、`npm run typecheck && npm run lint && npm run format:check && npm test && npm run build`、`bash scripts/doc-consistency-check.sh` と `--target plan` が exit 0（cargo は subshell の中で `src-tauri` へ移り、git・npm・rg・bash は root で回す。AC1〜AC3・AC5 の command も同じ）。L3 を含むので Writer は `(cd src-tauri && cargo check --release)` を通す。
- AC5（正本）: `rg -n 'proposed・未実装' docs/ARCHITECTURE.md docs/function-design/20-io-product-repo.md docs/function-design/21-io-inventory-repo.md docs/function-design/40-cmd-product.md docs/function-design/44-cmd-inventory.md docs/function-design/50-ui-product-list.md docs/function-design/51-ui-product-form.md docs/function-design/58-ui-stock-inquiry.md docs/function-design/65-inventory-record-traceability.md docs/function-design/66-ui-stock-movements.md | rg -v 'D-122|D-138|^[^:]+:[0-9]+:\| 20[0-9]{2}-[0-9]{2}-[0-9]{2} \|' | wc -l` が 0（D-118 の (b) の分が全部入る。D-122 の項と変更履歴の行は数えない）。`rg -c 'b1 の後・b2 の前の版' docs/adr/2026-10-16-stocktake-count-placement.md` が exit 1（b1 の merge の後は 1 で、b2 で外す）。``rg -n '`MovementRecord` の並びの鍵|`MovementRecord` に同じ列' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/backlog.md' | wc -l`` が 0。`rg -n 'list_movements.{0,40}T23:59:59|T23:59:59.{0,80}list_movements' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' | wc -l` が 0。file ごとの撤去: `rg -c 'T23:59:59|created_at DESC, id DESC|PaginatedResult<MovementRecord>|list_movements を追加|repo から得た movement 行' docs/function-design/44-cmd-inventory.md`（§23.9 の SQL 概要と IO の signature、D-118 の節の置き換えの宣言、§23.8 の CMD の signature、§23.1 のモジュール構成の行、§23.10 の BIZ の signature と `source` の補完の説明）、``rg -c '既存 `list_movements`|^\| 期間 predicate \| `date_to \+ T23:59:59`|movement/records 画面の期間 predicate' docs/function-design/74-ui-operation-logs.md``（UI-11c-D2・§74.4.3 の段落・§74.17 の比較の表の `StockMovementsPage` の列・§74.16 の非目的の行）、`rg -c '日時降順|日時は .YYYY-MM-DD HH:mm:ss. 表示|PaginatedResult<MovementRecord>|.MovementTable. に渡し|各 movement の日時' docs/function-design/66-ui-stock-movements.md`（主動線の 2.、§66.5 の表示規則、§66.4 の型、処理ステップの 5.、§66.1 の関数要求）、``rg -c '^\| `listMovements\(query\)` \| 商品別 movement。' docs/function-design/65-inventory-record-traceability.md``（§65.7.1 の command の表の行）がどれも exit 1。``rg -c 'b2 は proposed・未実装|b2（proposed・未実装）' docs/architecture/ui-task-specs.md`` が exit 1（Gated Amendment 1。D-118 の節の見出しと本文。同期の後の baseline は 2。同じ file の D-120・D-121 の予定の文は数えない）。
- AC6（既存の test の移し方）: Matrix の T1〜T3 の command を実装の後に逐語で回し、出力の各行（test の名前）を T1〜T3 の分類（移す / 期待を変える / 契約ごと外す）に当てた表を PR の本文に貼る。「移す」の test で値の期待（`assert` の右辺・件数・文言）が変わっていない。「契約ごと外す」の行に置き換えた正本の節と、代わりに同じ失敗を止める Matrix の行を書く。

AC の数は AC1〜AC6 の 6。

AC の数の baseline（2026-10-24、本 commit の作業木〈HEAD `63a00fb5` に本 commit の docs の差分〉で逐語で実行した。code は main `eed07d37` と同じ）:

```
$ rg -U -c 'export type ProductWithRelations = \{[^}]*stock_recheck: boolean' src/lib/bindings.ts
exit=1
$ rg -c '^export type (StockHistoryItem|StockHistoryKind|StockCountHistory) =' src/lib/bindings.ts
exit=1
$ rg -c 'ORDER BY created_at DESC, id DESC' src-tauri/src/db/inventory_repo.rs
1
exit=0
$ rg -c 'T23:59:59' src-tauri/src/db/inventory_repo.rs
2
exit=0
$ rg -U -c 'export type MovementRecord = \{[^}]*business_date' src/lib/bindings.ts
exit=1
$ rg -n 'proposed・未実装' docs/ARCHITECTURE.md docs/function-design/20-io-product-repo.md docs/function-design/21-io-inventory-repo.md docs/function-design/40-cmd-product.md docs/function-design/44-cmd-inventory.md docs/function-design/50-ui-product-list.md docs/function-design/51-ui-product-form.md docs/function-design/58-ui-stock-inquiry.md docs/function-design/65-inventory-record-traceability.md docs/function-design/66-ui-stock-movements.md | rg -v 'D-122|D-138|^[^:]+:[0-9]+:\| 20[0-9]{2}-[0-9]{2}-[0-9]{2} \|' | wc -l
13
exit=0
$ rg -c 'b1 の後・b2 の前の版' docs/adr/2026-10-16-stocktake-count-placement.md
exit=1
$ rg -n '`MovementRecord` の並びの鍵|`MovementRecord` に同じ列' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/backlog.md' | wc -l
2
exit=0
$ rg -n 'list_movements.{0,40}T23:59:59|T23:59:59.{0,80}list_movements' docs --glob '!docs/archive/**' --glob '!docs/plans/**' --glob '!docs/decision-log.md' | wc -l
2
exit=0
$ rg -c 'T23:59:59|created_at DESC, id DESC|PaginatedResult<MovementRecord>|list_movements を追加|repo から得た movement 行' docs/function-design/44-cmd-inventory.md
8
exit=0
$ rg -c '既存 `list_movements`|^\| 期間 predicate \| `date_to \+ T23:59:59`|movement/records 画面の期間 predicate' docs/function-design/74-ui-operation-logs.md
4
exit=0
$ rg -c '日時降順|日時は .YYYY-MM-DD HH:mm:ss. 表示|PaginatedResult<MovementRecord>|.MovementTable. に渡し|各 movement の日時' docs/function-design/66-ui-stock-movements.md
5
exit=0
$ rg -c '^\| `listMovements\(query\)` \| 商品別 movement。' docs/function-design/65-inventory-record-traceability.md
1
exit=0
```

`proposed・未実装` の検索は D-122 の項と変更履歴の行を除く（round 1 の是正。Review Response 参照）。`docs/ARCHITECTURE.md` の 2 行（D-118 の節の見出しと本文）と、44・66・74 の file ごとの語、65 の 1 本は round 2 の是正、66 の `` `MovementTable` に渡し `` は round 3 の是正（Review Response 参照）。

main の版の数なので、b1 の merge の後に変わる（`proposed・未実装` は b1 の分が減り、`b1 の後・b2 の前の版` は 1 になる）。同期の後に取り直し、同期の差分を PR の本文に書く。

完了時の期待: AC1 の数が 1。AC2 の数が 3、`ORDER BY created_at DESC, id DESC` と `T23:59:59` と `MovementRecord` の `business_date` が exit 1。AC5 の `proposed・未実装` が 0、`b1 の後・b2 の前の版` が exit 1、`MovementRecord` の並びの鍵の文と `list_movements` の `T23:59:59` の文が 0、44・74・66・65 の file ごとの 4 本が exit 1。test の本数は AC にしない（PR と runner の出力を参照）。

## Design Readiness

設計の完了条件は `docs/DEV_WORKFLOW.md` Design Phase Rules の Design completion criteria。

- 引用する設計正本（節まで）: ADR SPEC-STK-CNT-D1（鍵と表示の順）・D2（写しと範囲）・D3（「要再確認の間の在庫は範囲で見せる」）・D9（写しの 3 列、初期在庫の見分け方）・D12（b2 の範囲。b1 の Plan Commit の版）、[20](../function-design/20-io-product-repo.md)・[40](../function-design/40-cmd-product.md)・[50](../function-design/50-ui-product-list.md)・[51](../function-design/51-ui-product-form.md)・[58](../function-design/58-ui-stock-inquiry.md)・[65](../function-design/65-inventory-record-traceability.md)・[66](../function-design/66-ui-stock-movements.md) の D-118 の節、[44](../function-design/44-cmd-inventory.md) §23.8〜§23.10 と D-118 の b2 の節、[21](../function-design/21-io-inventory-repo.md) の D-118 の節、[master](../db-design/master-tables.md) の写しの列の CHECK、[共通規則](../function-design/10-common-rules.md) SPEC-UNIT-D3・D11、[design-system](../design-system/README.md) と [02](../design-system/02-component-catalog.md) の badge 3 種構成、D-052、D-061。
- 必要な設計成果物（状態）: (1) `list_movements` の結果の型・並び・絞り込み・合成の置き場（44）= updated in this PR。(2) 在庫変動の表の計数の行・「日付」の列・見出しの範囲（66）= updated in this PR。(3) 商品の結果の 3 列の型の置き場（`ProductWithRelations`）と範囲・badge の部品 = 本 packet で決め、実装の PR で 40・50・51・58・02 へ（S5）。(4) b1 が書き換えた節の b2 の分 = b1 の merge の後の同期で直す（Scope）。 (5) 商品の画面の要再確認の案内（58・66。owner の判断事項 (ii)）= updated in this PR（Plan Review round 1 の是正）。
- plan にしかない durable な判断の昇格先: (1)(2) は正本に置いた。packet にだけあるのは、規模と手法、S1〜S5 の順、所有表、b1 への依存、既存の test の移し方（Matrix の T1〜T3）、L3 の台本。(3) は実装の PR で正本へ。
- 前提・制約と、延期した design gap の follow-up: owner の判断事項 (i) は B に決まった（TD-397）。(ii) は推奨を既定にして進める（owner が変えれば Gated Amendment）。b1 の wire の名前（b1 への依存）。どれも b2 の読取りの正しさ（写しをそのまま見せる、記録を鍵で並べる）に効かない。
- 絶対保証（cannot happen / always happens）の例外と escape hatch の自己点検: 「要再確認なら範囲がある」は DB の CHECK（`stock_recheck = 1` なら両端が NOT NULL で low ≤ high。`src-tauri/src/db/schema_v9.rs` の `products` の `ALTER TABLE`）で守られ、画面は `stock_recheck` と両端の非 null で分岐する（型の上で両端が null の要再確認が来ても範囲を作らず、写しの値と badge を出す）。「計数の行は全部出る」の例外は種別の絞り込み（「棚卸し」と「すべて」以外では出さない）と日付の範囲だけ。「`MovementRecord` の形は変えない」は AC2 の検索で確かめる。「並びは全順序」は鍵の同じ行の足し方（`saved_at`・種類・id）で決まり、id は表ごとに一意なので最後の段で必ず決まる。
- 判定: ready（b1 の merge の後の同期で b1 への依存を当て直す前提）。b2 の Scope の全ての振舞いが正本か本 packet の S1〜S4・Boundary から決まる。

## Registration / Generation Obligations

| 変更対象 | 登録・生成義務 |
|---|---|
| Tauri command（frontend から呼ぶ） | command の新設・改名・削除は無い（`list_movements` ほか 5 つの出力の型だけが変わる）。`cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成（S3） |
| function-design doc 新設 | 該当なし（新しい doc は無い。新しい関数は `pub(crate)` で `src-tauri/tests/design_compliance_test.rs` の検査の対象外。44 の D-118 の節に置いた） |
| source / workflow doc 新設・改名・削除 | 該当なし |
| REQ / coverage の追加・変更・削除 | test 名・comment に REQ-101・REQ-102・REQ-205・REQ-207・REQ-301・REQ-303 を付ける。`cd src-tauri && cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成（S5） |
| route 新設・改名・削除 | 該当なし（`/stock/$code/movements` の search の key と値は変えない） |
| operator 画面新設・改名・削除 | 該当なし（4 画面の中身だけ。到達の導線は今のまま） |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 当たらない。外部の資料の形に触れない（読取りと表示だけ） | — |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 当たる。観測した事実は今の code の形（4 つの SELECT、`MovementRecord` を作る 7 か所、tagged enum が無いこと、`created_at` の形）。決めたことは起草役の判断（`ProductWithRelations` に足す、BIZ で合成、業務日で絞る、`MovementRecord` を変えない）で、44・66 と実装の PR の正本へ置く | 44・66、S5 |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 当たる。計数の保存・Z004 の確定と取消・入出庫の後に、写しと履歴が b1 の再取得（D-052 の SSOT）で新しくなる。Z004 の取消は movement を `is_voided` にするので履歴から消え、計数は消さないので残る | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 当たる。締め → 取込み → 要再確認の商品を商品一覧・在庫照会で範囲として見て、棚卸しの画面で入れ直す → 解けると 1 つの値に戻る。在庫変動の履歴で、数えた数と前後の記録を日付の順に読む | L3 の台本 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 当たらない（レジの資料の形に依らない） | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 当たる。L3 の DB は b1 の L3 の合成の DB（TD-296 の合成の fixture から作ったもの）で、写しは repo の外に置き commit しない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 当たる。計数の行は在庫を差分で動かさないので「増減」「変動後在庫」を「—」にし、movement の変動と取り違えない。`stock_after` は保存の時点の値で、鍵の順に並べても求め直さない（65）。在庫少と評価は写しのまま | Matrix の H1・U6 |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 当たる。Windows native の画面での範囲と badge の見え方（色だけでないこと、表の密度、長さの商品の m）と、b1 の書く経路で作った実の DB の記録が履歴に並ぶこと | Test Plan の L3 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | 当たらない（新しい依存・toolchain は無い） | — |

## Boundary / Wire Contract

- producer: IO の商品の読取り 4 つ（DB の写しの 3 列）と履歴の 2 つの読取り、BIZ の `list_movements`（並び・絞り込み・ページ・source）、CMD の 5 つの command。
- consumer: 商品一覧（`searchProducts`）、在庫照会（`searchProducts`・`listLowStock`・`getStockDetail`）、商品フォーム（`getProduct`）、在庫変動（`getStockDetail`・`listMovements`）、商品の候補（`searchProducts` を読む入出庫の画面と棚卸しの画面の候補。新しい field を読まない）、ホーム（`listLowStock`。新しい field を読まない）。在庫照会の詳細と在庫変動の見出しの要再確認の案内は b1 の `listRecheckProducts` を読む（b1 の wire のまま。b1 への依存の 13）。
- wire type:
  - `ProductWithRelations` に `stock_recheck: boolean`・`stock_range_low: number | null`・`stock_range_high: number | null`（`Product` には足さない。`StockDetail.product` も同じ型）。
  - `StockHistoryKind = "movement" | "count"`（D-061 の生成の literal union）。
  - `StockHistoryItem { kind: StockHistoryKind, business_date: string, settlement_no: number | null, saved_at: string, movement: MovementRecord | null, count: StockCountHistory | null }`（kind が `movement` なら `movement` だけが値、`count` なら `count` だけが値）。
  - `StockCountHistory { id: number, count_value: number, stock_unit: ProductStockUnit, count_kind: StockCountKind, stocktake_id: number | null, source: MovementSourceLink | null }`（`StockCountKind` は b1 の生成型。`stock_unit` は計数の商品の今の `products.stock_unit` で、画面は計数の値をこの単位で描く。商品情報の取得〈`getStockDetail`〉だけが失敗した部分障害〈UI-06c-D3〉でも、長さの商品の計数 450 は「4.5 m」。先例は b1 の `CountRecoveryTarget.stock_unit`）。
  - `listMovements` の戻り値は `PaginatedResult<StockHistoryItem>`（`items`・`total_count`・`page`・`per_page` は今のまま。`total_count` は絞り込みの後の movement と計数の合計）。
  - 変えない型: `MovementRecord`・`MovementQuery`・`MovementSourceLink`・`Product`・業務記録の詳細の型。
- internal type: IO の履歴の 2 つの行の型（`pub(crate)`、wire に出ない）。BIZ の並びの鍵（`(business_date, phase, settlement_no または -1, after_settlement)` と足しの 3 つ）は関数の中の値で、保存しない。
- precision/range: 写しの在庫と範囲の両端は在庫の上限 `999999999` の内（b1 の横断の約束の Z4、`recompute_stock` の検査）、計数の値も同じ上限（b1 の Z1）、精算回数は `9007199254740991` 以下（b1 の Z5）、`total_count` は u32。どれも JavaScript の安全な整数に入る。範囲は負を取りうる（写しが負のとき）。
- round-trip path: DB → IO → BIZ → specta → bindings → 画面。`MovementQuery` は画面 → CMD → BIZ（`date_from`・`date_to` は route の search の `YYYY-MM-DD`、`movement_type` は今の生成型）。
- invalid input: `date_from`・`date_to` が形・暦の外なら `validation`（「日付の形式が不正です（YYYY-MM-DD）」か「存在しない日付です」。31 の `validate_business_date` の文）。ページの外は今の `validation`（「ページパラメータが不正です」）。商品が無い `get_product`・`get_stock_detail` は今の `not_found`。
- compatibility: command の名前と引数は変えない。`MovementRecord` を受ける業務記録の詳細の画面と mock は変わらない。`list_movements` の並びが保存の時刻の降順から鍵の降順に、日付の絞り込みが保存の時刻から業務日に変わる（44・66 の D-118 の節。route の search の値はそのまま読める）。

## Test Plan

Test Design Matrix: [2026-10-23-stocktake-count-runtime-b2](test-matrices/2026-10-23-stocktake-count-runtime-b2.md)。

- If the Human Gate includes L3, Writer completion includes `cargo check --release` before the owner native build; this is not a CI gate.
- targeted tests: `(cd src-tauri && cargo test --lib db::product_repo && cargo test --lib biz::inventory_service && cargo test --lib cmd::inventory_cmd && cargo test --lib finite_enum_contract_tests)`、`npm test -- src/features/stock-inquiry src/features/products src/features/stock-movements src/features/inventory-records src/features/home src/features/backup-restore src/features/stocktake`。
- negative tests: Matrix の H4（種別の絞り込みで計数を含めない）、H9（不正な日付の絞り込み）、U1〜U5 の「要再確認でない商品に範囲と badge を出さない」の case、U10 の「要再確認でない商品に案内を出さない」「一覧の読取りの失敗でも案内と link を出し、詳細を止めない」の case。
- compatibility checks: 業務記録の詳細の `movements`（`MovementRecord`）の既存の test が期待を変えずに通る（Matrix の T3）。`MovementQuery` の search の値の往復（今の `StockMovementsPage.test.tsx` の case）。
- data safety checks: test の fixture は全部合成。L3 の DB は b1 の L3 の合成の DB。
- main wiring/integration checks: W2（CMD が BIZ の型を返す）、U9（画面が BIZ の順のまま描き、計数の保存の再取得の entry が在庫変動を持つ）。

L3（Human Gate `manual`。1 回の通しの台本で、各段の PASS / FAIL を返してもらう。起動は今の L3 と同じく Windows native の `npm run tauri dev`）。T は b1 の L3 の日（b1 の packet の Test Plan の T）。商品 P1〜P6 は b1 の packet の「L3 の fixture の表」の商品（商品コードは b1 の `tests/fixtures/z004/README.md`）。

L3 の起点の DB（owner は DB も file も作らない。先例は b1 の packet の Test Plan の「L3 の DB の用意と差し替え」）:

1. 控え（Coordinator、b1 の L3 が PASS した直後）: owner がアプリを止めた状態で、`%APPDATA%\com.kosei.inventory\` の `inventory.db`（と `-wal`・`-shm` があればそれも）を、repo の外の日付の付いた folder（例 `%USERPROFILE%\inventory-db-moved\{b1 の L3 の日}-b1-l3-end\`）へ写す（元は消さない・動かさない）。写しを読み取りだけで開き、下の 4 つの query の結果を控える（商品の写し、計数〈保存の日時と精算回数と棚卸しの明細を含む〉、履歴の入力になる movement の全部〈取消の印・業務日・精算回数・保存の日時を含む〉、Z004 の取込みの状態）。
2. 確かめ（Coordinator、b2 の L3 を頼む前）: owner がアプリを止めた状態で、今の DB の写しを読み取りだけで開き、同じ 4 つの query の結果が 1. の控えと行ごとに同じことを確かめる（b2 は migration を持たないが、間に merge した lane の migration で版が進むのは差にしない。query は列を名指すので、足された列は比べない）。写しと計数が同じでも、控えの後に入れた入出庫や取込みの取消があれば movement か取込みの結果が違い、L3-4〜L3-6 の期待の順と合わなくなるので、同じに扱わない。違えば（間の lane の L3 で DB を入れ替えた、控えの後に記録を足した等）、今の data の folder を [22 §16](../function-design/22-mnt-migration.md) の作り直しの手順 2〜4 で日付の付いた folder へ消さずに移し、1. の写しを `%APPDATA%\com.kosei.inventory\inventory.db` に置いて、もう一度確かめる。
3. 1. の写しも b1 の L3 の後の DB も無いとき: b1 の packet の L3-2〜L3-13 の台本を owner にもう一度頼むか、Writer が `src-tauri/src/biz/l3_b2_db.rs`（`#[cfg(test)]`、BIZ の関数で同じ状態の DB を作る `#[ignore]` の test。先例は b1 の `io::l3_fixtures`）を足して Coordinator が作るかを、Coordinator が owner の手間の少ない方で決める（予備は 0 なので、`Owner Effort Budget` のとおり上限の改定を同じ問い合わせで諮る）。

```
SELECT product_code, stock_quantity, stock_recheck, stock_range_low, stock_range_high FROM products ORDER BY product_code;
SELECT sc.id, sc.product_code, sc.count_kind, sc.count_value, sc.business_date, sc.settlement_no, sc.counted_at, sc.stocktake_item_id, si.stocktake_id FROM stock_counts sc LEFT JOIN stocktake_items si ON si.id = sc.stocktake_item_id ORDER BY sc.id;
SELECT id, product_code, movement_type, quantity, stock_after, reference_type, reference_id, is_voided, business_date, phase, settlement_no, created_at FROM inventory_movements ORDER BY id;
SELECT id, machine_no, settlement_no, settlement_date, status, imported_at FROM csv_imports ORDER BY id;
```

`csv_imports` の `machine_no`・`settlement_no` は b1 の Plan Commit の版の列の名前（[pos-tables](../db-design/pos-tables.md) の b1 の版）で、b1 の merge の後の同期で当て直す（b1 への依存と同じ）。

起点の状態（b1 の packet の L3 の表の後の状態から導いた。Coordinator が 2. の query で確かめる）: P2 は fresh な計数 8（T）の後に同じ日付 T の入庫 +1（b1 の L3-13）があり、写し 9・要再確認・範囲 8〜9（ADR D3 の fresh な計数の行の「i > m のうち E = T は決まらない」）。P1 は写し 9（効く計数は fresh な 9〈T〉）、P4 は写し 450（4.5 m）、P5 は写し 9、P6 は写し 0、どれも要再確認でない。計数は P1 に既定 11（T−3）・既定 10（T−2）・fresh 9（T）、P2 に既定 10（T）・fresh 8（T）、P3 に既定 9（T）、P4 に既定 450（T）、P5 に fresh 9（T、棚卸しの明細を参照しない）・既定 9（T）、P6 に既定 0（T）。P5 の fresh 以外は完了した棚卸し（b1 の L3 で最初に作った棚卸し）の明細を参照する。Z004 の取込みは精算 101（T−3）・102（T−2）・104（T、2 回目）・103（T−1）が active。

| # | 前の状態 | 操作 | 観測可能な合格基準 | 後の状態 |
|---|---|---|---|---|
| L3-1 | 上の起点の DB | 新しい版を起動し、商品一覧を開く | P2 の在庫の欄に「8〜9 個」と「要再確認」（icon と文字。色だけでない）。P1 は「9 個」、P4 は「4.5 m」で、どちらも「要再確認」が無い | 同じ |
| L3-2 | 同じ | 在庫照会で P2 を検索し、行を開いて詳細を見る → 案内の「棚卸しの画面で入れ直す」を押し、何も保存せずに在庫照会へ戻る | 一覧の在庫の欄と詳細の「在庫数」に「8〜9 個」と「要再確認」。状態の badge は「在庫あり」（写しの 9 で決まる）。詳細に「数え直しが必要です」の案内（icon と文字）、P2 の理由「数えた日に売り買い（入出庫）がありました」だけ（b1 の `formatRecountReasons` の `same_date_movement` の文で、`undetermined_settlement` も同じ文に 1 つにまとまる。同じ日付 T の既定の計数 10 は fresh な計数 8 より前なので効かず、「同じ日に違う数が保存されています」は出ない）、「棚卸しの画面で、棚の数を数えて入れ直してください。」。link で棚卸しの画面が P2 の「今の数を入れる」の欄で開く。P1 の詳細には案内が無い（owner の判断事項 (ii) を見せる段） | 同じ（何も保存しない） |
| L3-3 | 同じ | 詳細の「商品修正」→ 何も変えずに「戻る」 | 「現在庫」（読取り専用）に「8〜9 個」と、label の隣に「要再確認」。「戻る」で離脱の確認が出ずに戻る | 同じ |
| L3-4 | 同じ | 在庫照会で P2 の「在庫変動履歴」 | 見出しの「現在庫」に「8〜9 個」と「要再確認」、その下に L3-2 と同じ案内。表の上から順に「数を入れ直し（8 個、T）」（増減と変動後在庫は「—」、元記録は「棚卸し #{b1 の棚卸しの id}」）、入庫 +1（変動後在庫 9、元記録は入庫記録）、「数を確認（10 個、T）」（元記録は棚卸し）、入庫 +10（初期在庫、元記録なし）。各行の日付 T の下に「保存 {日時}」 | 同じ |
| L3-5 | 同じ | P1 の在庫変動履歴を開く → 日付の範囲を T−2〜T−2 にする → 日付を戻し、種別を「棚卸し」にする | 絞る前は上から「数を入れ直し（9 個、T）」、入庫 +10、POS売上 −1（日付 T−2 の後に「（精算 102）」、元記録は CSV取込み）、「数を確認（10 個、T−2）」、「数を確認（11 個、T−3）」。日付で絞ると POS売上 −1 と「数を確認（10 個、T−2）」の 2 行（保存の日時は T のまま）。「棚卸し」では計数の 3 行だけ | 同じ |
| L3-6 | 同じ | P5 と P4 の在庫変動履歴を順に開く | P5: 上から「数を入れ直し（9 個、T）」（元記録なし）、「数を確認（9 個、T）」（元記録は棚卸し）、入庫 +10。P4: 「数を確認（4.5 m、T）」、入庫 +5 m（変動後在庫 5 m） | 同じ |

- L3 に入れないもの（Windows native でなくても見られるもの）: 並びの鍵の全ての組合せ（fresh な計数の n が NULL、鍵の同じ行の足し方、ページを切った後の行）、日付の絞り込みの不正な値、在庫少の判定が写しで決まること（U2）。自動 test（H1〜H9・U1〜U10）で見る。
- 画面の確かめ（Human Visual Confirmation）: 上の L3-1〜L3-6 が 4 画面の route（`/products`・`/stock`・`/products/{商品コード}/edit`・`/stock/{商品コード}/movements`。L3-2 の link の先の `/stocktake/` は b1 の画面で、開くことだけを見る）、主な流れ、状態の区別（要再確認の badge と範囲と案内、計数の行と movement の行）、日本語の文言を含むので、同じ 1 回の PASS / FAIL に含める。

## Review Focus

- `list_movements` の並び（ADR D1 の鍵の 4 つ、初期在庫の -1、fresh な計数の 1 と n が NULL の -1、鍵の同じ行の足し方）が全順序で、ページを切っても欠け・重なりが無いか。
- 日付と種別の絞り込みの意味（業務日で両方、「棚卸し」で計数）が、在庫変動の画面の使い方と 65 の契約に合うか。保存の時刻の絞り込みを外して困る使い方が無いか。
- 合成を BIZ に置く理由（鍵の規則を SQL に写さない）と、1 商品の行を全部読む読取りの量の見込み。初期在庫の印の式が 1 か所か。
- `MovementRecord` の形を変えない判断が、ADR D12・21 の文の書き換えで足り、業務記録の詳細に影響しないか。
- 範囲と badge の部品が 4 画面で 1 つで、要再確認でない商品に出ず、色だけで示さないか。要再確認の案内（owner の判断事項 (ii)）が b1 の wire と入口だけで作れ、理由の文を b1 と 2 つ持たないか。在庫少の判定・並べ替え・ホームの件数が写しのままか（U2）。
- b1 への依存の 13 行が、b1 の merge の後の同期で当て直せる形で名指されているか。
- **この変更で不要になる記述・条件・code が、ほかに残っていないか**（上の「この変更で不要になるもの」。例: b1 だけの版の表示の欠けの注記、「proposed・未実装」の b2 の分、`list_movements` の `created_at` の並びと `T23:59:59` を引く文）。
- 公開 repo に実データを書いていないか。並走の lane の節を触っていないか（所有表）。

## Contract Ledger

触る設計正本の節の契約を行にした（隣接の契約の sweep: ADR D1〜D12、20・21・31・40・44・50・51・53・58・61〜64・65・66・73・74・75 の D-118 の節と在庫の表示、master の写しの列、共通規則 SPEC-UNIT-D3・D11、design-system の badge 3 種構成、D-052・D-061 を当て、b2 の Scope が触り得ないものは下の「除外」に理由つきで書いた）。

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| SPEC-STK-CNT-D1（鍵は表示の順に使う、計数に入っているかを示さない） | ADR D1、44 の D-118 の b2 の節 | S1・S2 | H1・H2・H8 | L3-4〜L3-6 |
| SPEC-STK-CNT-D2・D3（写しと範囲、要再確認の間の在庫は範囲で見せる） | ADR D2・D3、50・58・51・66 の D-118 の節 | S1・S4 | P1・U1〜U5・U8 | L3-1〜L3-4 |
| SPEC-STK-CNT-D9（`products` の写しの 3 列、初期在庫の見分け方） | ADR D9、master | S1 | P1・H8 | — |
| SPEC-STK-CNT-D12（b2 の範囲） | ADR D12（b1 の版） | 全段 | AC5 | — |
| IO-20（商品の読取りが写しの 3 列を返す） | 20 の D-118 の節 | S1 | P1 | — |
| IO-21（履歴の読取り、初期在庫の印の共有。21 の `MovementRecord` の列の文を置き換える） | 21 の D-118 の節、44 の D-118 の b2 の節 | S1 | H1・H6〜H8 | — |
| CMD-01（商品の get / search の結果に 3 列） | 40 の D-118 の節 | S1・S3 | P1・P2 | — |
| CMD-06（`get_stock_detail`・`list_low_stock`・`list_movements` の出力、並び・絞り込み・ページ） | 44 §23.8〜§23.10 と D-118 の b2 の節 | S1〜S3 | P1・H1〜H9・W2 | L3-4〜L3-6 |
| 65（計数の行を鍵の順に混ぜる、`stock_after` を並べ直さない） | 65 の D-118 の節の 1 つ目・2 つ目の項 | S2・S4 | H1・U6 | L3-4〜L3-6 |
| UI-01a（商品一覧の在庫の欄、並べ替えは写し） | 50 の D-118 の注記 | S4 | U1 | L3-1 |
| UI-06a / UI-06b（在庫照会の一覧と詳細、在庫少の判定は写し、詳細の要再確認の案内） | 58 の D-118 の注記 | S4 | U2・U3・U10 | L3-2 |
| UI-01b（修正の画面の在庫数の表示） | 51 の「在庫数の表示」 | S4 | U4 | L3-3 |
| UI-06c（見出しの範囲と要再確認の案内、表の行の順・日付の列・計数の行・絞り込み。UI-06c-D2〜D6 を保つ） | 66 の D-118 の注記、UI-06c-D2・D4・D5・D6 | S4 | U5〜U7・U9・U10 | L3-4〜L3-6 |
| SPEC-UNIT-D3・D11（範囲の両端と計数の値の単位の書き方、長さは m） | 共通規則 | S4 | U6・U8 | L3-6 |
| design-system の badge 3 種構成（①状態は outline + icon + soft tone、色だけで示さない） | 02、design-system README | S4・S5 | U1〜U5 | L3-1〜L3-4 |
| D-052（計数の保存・取込み・取消の後の再取得。b1 の entry） | UI_TECH_STACK の D-118 の段落、`src/lib/invalidation-contract.ts` | 変えない（b1 への依存の 5） | U9 | — |
| D-061（生成の literal union） | decision-log | S3 | W1 | — |

除外（触らない契約と理由）: 73・55 の要再確認の一覧と範囲、`list_recheck_products` と 73 の入口（b1。b2 は読むだけ。`StocktakePage.tsx` は要再確認の一覧の query を hook に切り出すだけで振舞いを変えない、S4）。75・74 の写しの書き方（「{数}（要再確認 {low}〜{high}）」。診断の表示で、b2 の部品に置き換えない）。53 のホームの件数（写しのまま。50 の注記）。62〜64 の出庫と返品の画面の「現在庫」（owner の判断事項 (i)。B に決まった、TD-397）。31 の求め直しと `validate_business_date`（呼ぶだけで変えない）。業務記録の詳細の `MovementRecord`（形を変えない）。

## Contract Audit（起票の時点の一巡）

新しく足す契約の文（44・66 の D-118 の節、本 packet の Boundary と S1〜S4）を、境界の 4 観点で一巡した。

- 安全な整数の範囲: 写しの在庫と範囲・計数の値は在庫の上限の内、精算回数は安全な整数の内（b1 の Z1・Z4・Z5）、`total_count` は u32 で、どれも wire の number で正確。b2 は値を足し算しない（並べて見せるだけ）ので新しい溢れは無い。→ 不足は無かった。
- 0 の表示: 計数 0 は「数を確認（0 個、…）」と出す（売り切れ・「0 にする」。0 を「無い」の印にしない）。範囲の端が 0 か負でも `formatStockDisplay` の形で出す。fresh な計数の n が NULL は鍵の 3 つ目を -1 とみなし、精算回数 0 の n と取り違えない（ADR D1）。`settlement_no` の wire は movement の phase 0 の行と既定の計数で null。計数が 1 つも無い商品は今の movement だけの履歴。→ 正本の不足は 44（n が NULL の扱いを履歴の並びで書く所が無かった）で、本 commit で直した。
- 失敗の時点と順序: 日付の検査は読む前（不正なら何も読まない）、ページの検査は今のまま最初。並びはページを切る前に全体に当て、件数も同じ集合で数える（ページを切ってから並べる誤りを H5 が止める）。読取りの失敗は今の画面の表示（UI-06c-D3 の部分障害）。→ 正本の不足は 44（絞り込み・並べ替え・ページの順が無かった）で、本 commit で直した。
- 単位や種類の混在: 計数の値と movement の数量は同じ在庫の数量の単位（長さは cm を画面が m に）。計数の行は在庫を差分で動かさないので「増減」「変動後在庫」を「—」にし、movement の数と混ぜない。日付の軸は業務日に揃え、保存の時刻は表示だけ。`saved_at` は movement と計数で同じ形（b1 への依存の 2）。範囲と badge（商品の画面）と、整合性・操作ログの写しの書き方（診断）は別の表示として分ける。→ 正本の不足は 66（計数の行の列の中身と日付の列が無かった）で、本 commit で直した。

この起票で新しい wire の型は 3（`StockHistoryItem`・`StockHistoryKind`・`StockCountHistory`）、新しい field は `ProductWithRelations` の 3 列。消した記述: 無い（44・66 は置き換えを宣言し、旧い文の撤去は実装の PR の S5。上の「この変更で不要になるもの」）。

Plan Review round 1 の是正（Review Response 参照）で足した・置き換えた契約の文は 2 つ: (1) 58・66 の要再確認の案内（在庫照会の詳細と在庫変動の見出し。題・理由・案内・棚卸しの画面への link、理由を出さない場合。owner の判断事項 (ii)）、(2) 66 の計数の行の {数} の書き方（movement の数量の形から `formatStockDisplay` の形へ置き換え。食い違いを解いた置き換えで、新しい振舞いでない）。H1 の fixture・W2 の test の形・AC5 の oracle・REQ の一覧は test と検査の直しで、契約の文を足していない。境界の 4 観点で一巡した:

- 安全な整数の範囲: 案内は新しい数を持たない（理由は b1 の生成の enum、link は商品コード）。範囲は上の部品のまま。(2) は表示の形だけで値を変えない。→ 不足は無かった。
- 0 の表示: 要再確認なのに当該の target が無い・`recount_reasons` が空のときは理由の行を出さず、案内と link は出す（理由が無いことを「直し方が無い」にしない）。計数 0 は (2) の形で「0 個」「0 m」。→ 正本の不足は 58 の案内の文（理由を出さない場合）で、本 commit で直した。
- 失敗の時点と順序: 案内を出すかは写しの `stock_recheck` で決め、要再確認の一覧の読取りは理由を補うだけ（読込み中・失敗でも詳細と案内を止めない）。写しと一覧は b1 の同じ entry で新しくなる（b1 への依存の 13）。link は保存をせず、棚卸しの画面の保存の後は 73 のとおり棚卸しの画面に留まる。→ 正本の不足は 58（一覧の読取りの失敗の扱い）で、本 commit で直した。
- 単位や種類の混在: 理由の文は b1 の label を共有し、商品の画面で別の書き方を持たない。(2) で計数の行の {数} は在庫の欄と同じ形（個数にも単位の語）、movement の行の増減と変動後在庫は今の形（個数は語なし）のままで、同じ表の中で計数と movement の数を取り違えない（計数の行の増減と変動後在庫は「—」）。→ 正本の不足は 66 の {数} の文（表の 6 と食い違っていた）で、本 commit で直した。

この是正で新しい wire の型・field・command は無い。消した記述: 66 の「{数} は movement の数量と同じ単位の書き方」、packet の「(i) が A なら plan-gate の前に」と「plan-gate へは b1 の merge と同期の後」、Non-scope の「要再確認の理由を商品の画面に出すこと」。

Plan Review round 2 の是正（Review Response 参照）で足した契約の文は 1 つ: `StockCountHistory.stock_unit`（計数の商品の今の `products.stock_unit`。44・66・Boundary）。L3 の起点の DB の確かめの query を足したのは手順の直しで、契約の文でない。境界の 4 観点で一巡した:

- 安全な整数の範囲: 新しい数を持たない（単位は生成の enum `ProductStockUnit`）。→ 不足は無かった。
- 0 の表示: 計数 0 は単位つきで「0 個」「0 m」のまま（単位は行が持つので、商品情報の取得の失敗で語が落ちない）。→ 不足は無かった。
- 失敗の時点と順序: 単位は履歴と同じ読取りで返るので、`getStockDetail` だけの失敗（UI-06c-D3）で計数の数の意味が決まらなくなることが無い。履歴の読取りの失敗は今の Alert。→ 正本の不足は 44・66（計数の行の単位を商品情報の取得に頼っていた）で、本 commit で直した。
- 単位や種類の混在: 単位は計数の保存の時点でなく今の商品の単位を読む（写しと同じ。登録の後の単位の変更は個数の単位どうしだけで〈[共通規則](../function-design/10-common-rules.md) SPEC-UNIT-D2 の「単位の変更」〉数量の整数の意味を変えないので、今の単位で計数の値を描いてよい）。movement の行は今の `product_code → stock_unit` の対応のまま（UI-06c-D5）。→ 不足は無かった。

この是正で新しい wire の field は `StockCountHistory.stock_unit` の 1 つ。消した記述: 不要になるものの「`list_receivings` ほかは今の形のまま残るので、例をそちらへ替える」（入出庫の記録は `T23:59:59` を使っていないので、対比を撤去する）、Matrix の T2 の「表の日時を見る case は期待を変える」（日時を assert する case は無い）、H1 の Would fail if の「phase を見ない」（CHECK で `settlement_no` の有無と等価）、Test Plan の「介入の予備 1 を使う」（予備は 0）。

Plan Review round 3 の是正（Review Response 参照）で置き換えた契約の文は 3 つ: (1) 範囲の文字の形を b1 の `formatStockOrRange` の「{low}〜{high} {単位}」（単位の語は後ろに 1 回）にした（66 の見出しの項、Goal、Matrix の U1・U4・U5・U8、L3）。(2) 44 の `settlement_no` に、画面が精算の番号を付けるのは Z004 の movement の行だけと 1 句。(3) 商品フォームの badge の要否を prop で渡す経路と、要再確認の一覧の query を hook に切り出す置き場（S4）。どれも新しい振舞いでなく、b1 の部品と今の code の形に合わせた置き換え。境界の 4 観点で一巡した:

- 安全な整数の範囲: 新しい数を持たない（(1) は表示の形だけ）。→ 不足は無かった。
- 0 の表示: 範囲の端が 0 か負でも `formatStockOrRange` の形（例「-1〜0 個」）で、端の単位の語を落とさない（後ろに 1 回付く）。→ 不足は無かった。
- 失敗の時点と順序: 一覧の query の切り出しは key・`queryFn`・`staleTime` を変えず、棚卸しの画面の再取得の時点は今のまま。→ 不足は無かった。
- 単位や種類の混在: 範囲（商品の画面と棚卸しの画面で同じ部品）と、整合性・操作ログの写しの書き方（両端に単位、75・74）は別の表示のまま。fresh な計数の `settlement_no`（並びの鍵）と Z004 の行の精算の番号（表示）を (2) で分けた。→ 正本の不足は 44（fresh な計数の `settlement_no` の表示の扱い）で、本 commit で直した。

この是正で新しい wire の型・field・command は無い。消した記述: 範囲の両端に単位を付ける形（「8 個〜9 個」）、新しい書式の関数 `formatStockRange` と `format-stock-display.ts` の予定 path、起票時に起草役が決めたことの 2 の「b1 の部品があれば揃える」と b1 への依存の 7 の重なる文、Matrix の U1 の等価な mutant（`stock_recheck` を見ずに範囲の有無で決める）。

## Contract Probe

N/A。本 lane は外部の library・OS・機器の振舞いの前提に依らない（読取りの SQL・serde の struct と Option・React の表示だけで、どれも repo の中で既に使っている形。serde の tagged enum のような新しい形を使わない）。

## Data Safety

- commit しない物: 実 POS / 店舗データ、DB、backup、log、receipt。test の fixture は全部合成。
- local だけの場所: owner の PC の `%APPDATA%\com.kosei.inventory`（L3 の DB）と、Coordinator が控える b1 の L3 の後の DB の写し（repo の外の日付の付いた folder。b1 の L3 の合成の値だけ）。L3 の起点の DB の確かめは読取りだけ（写しを開く）。差し替えるときは 22 §16 の作り直しの手順で消さずに移す（D-123 でリリースの前まで開発・試験 DB の作り直しは包括で承認済み）。
- 合成だけの場所: Rust と TS の test の fixture。

## 残りの危険（起票の時点）

- b1 の実装で wire の名前・形（`StockCountKind`、`counted_at` の形、`stocktake_item_id` の付け方、初期在庫の印の式）が b1 の packet と変わると、本 packet・Matrix・44・58・66 を当て直す（実装の開始の前の、b1 の merge の後の同期。変われば Gated Amendment）。
- 在庫変動の並びが保存の時刻の順から業務日の順に変わるので、後から入れた記録（前の日の分の計数、翌日に取り込んだ精算）が表の上に来なくなる。「日付」の列の下に保存の日時を出して読めるようにした（66）。慣れの問題が L3 で出れば、表示の工夫の範囲で直す。
- 1 商品の履歴を全部読んでから並べるので、1 商品の行が数万に増えると遅くなりうる（44 に移り先を書いた）。単店舗の 1 商品の行の数は未実測。
- 出庫と返品の画面の候補の「現在庫」は写しの値だけで、要再確認の範囲を出さない（owner の判断事項 (i) は B、TD-397）。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

（実装の後に書く。）

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

（Plan Review の後に書く。）

round 1（`70803cef`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 2 / P3 5）、Codex = reject（P1 0 / P2 2 / P3 1）。重なりは H1 の mutant、古い文の残り、商品の画面の直し方の導線（Opus は P3、Codex は P2）。Coordinator は全件を採用した。商品の画面の直し方の導線は AI の review が出所で、owner の方針（repo 外の回答台帳 TD-401）で推奨を既定にして入れ、後で owner に見せる（TD-403）。是正は起草役の `dc9f4529`。

- P2: 66 の計数の行の {数} を `formatStockDisplay` の形に、L3-4 の変動後在庫を今の表の書き方に。H1 の fixture を「fresh の後に同じ精算を取り消して取り込み直す」形にし、`after_settlement` を 0 にする mutant を red に。在庫照会の詳細と在庫変動の見出しに要再確認の理由と「棚卸しの画面で入れ直す」案内（b1 の `list_recheck_products` を使い、新しい wire は作らない）。
- P3: TD-397・TD-402 の後の形に古い文を揃え、b1 の merge と同期は実装の開始の前の条件に。AC5 の oracle を D-118 の行に絞り file ごとの検査を足す。W2 を `tauri::test::mock_builder` の形に。REQ-207。

round 2（`d4b9ee3a`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 5）、Codex = reject（P1 0 / P2 2 / P3 1）。重なりは無い。Coordinator は全 9 件を採用した。是正は起草役の `6d2e857a`。

- Codex 側: P2 計数の履歴の DTO に単位が無く、商品情報の取得だけが失敗したときに m の計数を描けない → `StockCountHistory` に今の商品の `stock_unit` を持たせ、U6 に部分障害の case。P2 L3 の起点の確かめが履歴の同一性を見ない → 控えと開始前の比較を 4 本の query（計数・movement の全列・取込みの状態）の行ごとの比較に。P3 `docs/ARCHITECTURE.md` の b2 の未実装の注記を S5・AC5 に。
- Claude 側: P2 「`list_receivings` ほかは今の形のまま残る」の前提が誤り（records は日付の列を `<= ?` で比べるだけ）→ 20・74 の対比は例を替えず撤去し、在庫変動は b2 で業務日の両端を含む比べに変わったと書く（本文の直しは b2 の merge で事実になるので S5 で行う。74 の records の列の「同左」も S5 で実際の形に）。P3 T2 は 14 本とも移す（日時の assert は無い）、L3 の起点の DB の予備 0 の扱い、H7・U10 の fixture の偶然と H1 の等価 mutant、不要になるものに 44 §23.1・§23.8〜§23.10 と 65 の `listMovements` の行、暦の外の日付は BIZ の validation と部分障害の Alert。

round 3（天井、`63a00fb5`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 6）、Codex = reject（P1 0 / P2 1 / P3 0）。重なりは無い。Coordinator は全 7 件を採用し、disposition「同型指摘の一括是正」とした（round 4 は回さない）。範囲の表示の向きは Coordinator が決めた: b1 の `formatStockOrRange` を唯一の部品として使い、b2 は新しい書式の関数を作らない（既存の部品に寄せる。b2 は plan-gate なので b1 の Amendment は要らない）。是正は起草役の `0256d19b`。

- Codex 側: P2 AC4 の bindings の検証の作業ディレクトリ → root 起点の subshell の形に（同じ形の AC1・AC2・targeted tests も）。
- Claude 側: 範囲の表示の向きと要再確認の一覧の hook の切り出し、L3-2 の期待の文を逐語に、U9 の mock の並び、S5 の 66 の残り 3 か所と AC5 の oracle、商品フォームへ `stock_recheck` を prop で渡す経路と `ProductFormPage.tsx` の所有、fresh な計数の行に精算の番号を付けない。
- 起草役の付随の直し（Coordinator が受理）: U1 の等価な mutant を外す、U10 の mock の理由を b1 の文で見分けられる組に替える。見送り: b1 の `RecheckTargetList.tsx` の inline の「要再確認」の badge は b2 で置き換えない（b2 は b1 の画面を変えない）。1 つに揃えるかは closeout で backlog へ渡す。

closure（`d4820088`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 2）、Codex = approve（P1 0 / P2 0 / P3 0）。round 3 の 7 件は両方とも closed（Claude 側は O4 の残りを新しい P3 N2 に分けた）、C1 の固定の確認も両方で済み（直しを戻すと AC4 の command が exit 128）。Claude 側の新しい P3 2 件は Coordinator が直した（本 commit）: N1 S4 の hook の切出しで b1 の `StocktakePage.tsx` に触れるのに AC3・targeted の対象に `src/features/stocktake` が無い → 足し、除外の文を「振舞いを変えない切出しだけ」に。N2 AC5 の 66 の oracle が §66.1 の関数要求の句を拾わない → 足し、baseline を逐語の実測で取り直した。
