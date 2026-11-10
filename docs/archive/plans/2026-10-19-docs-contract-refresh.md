# Plan Packet: 設計・dashboard・backlog・decision-log の Status を今の実装と決定に合わせる（docs の棚卸し lane 1）

wave に属さない単独の lane（docs 全体の棚卸し〈owner 2026-10-18、計画は owner 2026-10-19、repo 外の回答台帳 TD-327〉の lane 1「契約の文を今に合わせる」。並走は lane `agent/docs-writing-rules`〈書き方の規則の正本化、D-124〜D-126〉。file:行 の引用の書き換えは別の lane 2〈TD-326〉）。docs だけ。code・migration・bindings・画面の振舞いは変えない。

出所: 起点 `18db621e`（main）で Coordinator が依頼した read-only の調査 2 本（repo 外の報告。A「proposed・未実装と decision-log の Status」、C「dashboard と時点つきの事実・未確認の前提」）。下の数のうち「調査の分類の数」は調査の手の分類で、command の出力ではない（調査 A: 実装済みなのに proposed・予定のまま 40、一部実装で書き分けが無い・古い 23、計 63。調査 C: dashboard の食い違い 30 と古い「最終更新」の見出し 8 file、backlog の上位 255 項目のうち済んだのに開いている 18・閉じ印つきで残置 46・前提が変わった 7・重複 13）。AC の baseline は本 packet の起票で command を逐語で実行した出力（下の Acceptance Criteria、D-062）。

規模と手法: 触る file は 50 前後（下の Scope の段ごとの表）と本 packet。1 lane のまま、作業を S1〜S5 と S7（個人の memory にあった tracked にすべき 2 項を正本へ上げる）に分け、段ごとに AC を置く（S6 は直さない分類）。本 run（起票）は packet だけを置き、docs の本体の直しは plan-approved の後の別の run で行う。

## Workflow State

- Phase: archive
- Risk: R2
- Plan Commit: ccdf03405dcd495e3db84c3a415dd1b1a821fe0b
- Amendments: none
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（起票は本 run。docs の直しは plan-approved の後の別の run）
- Plan Reviewer: fresh Opus 5.5（fork でない）+ Codex。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Reviewer: fresh Opus 5.5（Claude 側。R2 で design lane でないので座組表の「それ以外」の行）+ Codex。座組表どおり
- Final Review Minimum: 1
- Human Gate: ready,merge
- Branch: agent/docs-contract-refresh

manual なし: 製品の runtime・画面・配布物を変えない docs の変更で、Windows native で確かめる物が無い。

遷移の記録:

1. kickoff → spec-check（2026-10-19、起草役）: 発注書の範囲（調査 A・C の直す所、TD-327 の Status の規則の適用、Draft PR #155 の backlog への戻し、backlog の 51 UI-01b-D5 の句）を範囲にし、Risk を R2 と記録した（下の Risk）。
2. spec-check → plan-draft（2026-10-19、起草役、同じ commit）: 表の唯一の skip。直す先の設計正本（下の Design Readiness の一覧）は契約として足りていて、本 lane は新しい設計を足さない。文を code（実装の現物）と owner の決定（decision-log・TD-327）に合わせるだけなので、design の段で足す設計の出力が無い。
3. plan-draft（round 1 の是正、2026-10-19、起草役）: Plan Review round 1 の是正（Review Response 参照）。Scope の範囲と方向は変えず、段の中の行と AC を足し・直した。
4. plan-draft（round 2 の前の追加、2026-10-19、起草役）: Q4 の裁定と S7 の追加（Review Response 参照）。S7 で Scope に 3 file を足した。
5. plan-draft（round 2 の是正、2026-10-19、起草役）: Plan Review round 2 の是正（Review Response 参照）。S1・S2 の型を意味の側で洗い直し、S1 に行を足した。Owner Effort Budget の上限を改定した。
6. plan-draft（round 3〈天井〉の是正、2026-10-19、起草役）: Coordinator が round 3 の残りを同型の一括是正に disposition した（Review Response 参照）。S1 の型の語の幅を広げて洗い直し、S4 に採用済みで未実装の決定の型の洗い直しを足した。
7. plan-draft → plan-gate（Plan Review round 1〜3、2026-10-19）: Claude 側 fresh Opus 5.5・Codex（round 1 は Sol が満員で Astra、round 2・3 は Sol）。各 round の是正は Review Response 参照。
8. plan-gate → plan-approved（2026-10-19、`7f46a212`）: owner の決定（repo 外の回答台帳 TD-330）。round 3 の天井の後、指摘を直して追加の round を開かない。Plan Commit は round 3 の是正の commit（`ccdf0340`）。
9. plan-approved → implementing（2026-10-19、Coordinator、本 commit。state だけ）: Plan Commit `ccdf0340` は実装の commit より前にある。実装は Writer（Opus 5.5 subagent）への発注 445 で行う

## Owner Effort Budget

- 介入回数上限: 9（既定の 6 から改定。理由: 棚卸しの途中で owner の決定が 4 つ加わり〈下の表〉、Ready・merge と round 3〈天井〉の後の owner の判断の見込みを足すと既定の 6 に予備が残らない。上限を上げる先例は repo 外の回答台帳 TD-296 の 5a〈MSI の手順書の lane で 6 → 9〉）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9 | 7（closeout 時点。decision point ごとに数える: plan-approved までの 5〈TD-327 の棚卸しの計画 1、TD-327 の decision-log の Status の規則 1、TD-327 の Draft PR #155 の扱い 1、S7 の範囲の追加〈owner 2026-10-19 の指示〉1、TD-330〈round 3 の天井の後、直して追加の round なしで plan-approved〉1〉、Final Review に Codex を入れる判断〈2026-10-20〉1、Ready・merge の指示〈2026-10-20、1 回の指示〉1） | 0 | 2 | 9 = 7 + 0 + 2 |

店の資料・owner に聞く束（下の S6 の分類）は本 lane で問わないので、本 lane の介入に数えない。

承認依頼フォーマット: `この change での介入 N 回目 / 予算 M 回` + `承認すると利用者から見て何が完了するか1文`。

## Risk

Risk: R2

Reason:
docs だけの変更で、runtime の契約を変えない。直すのは、live な契約の文が今の実装（code の現物）と今の決定に合っていない所で、契約そのもの（command・DTO・DB・画面の振舞い・merge gate）を変えない。`docs/DEV_WORKFLOW.md` Risk Tiers の境の規則（R2 と R3 で迷えば、安定した契約・出力 schema・data safety の境・command の形・route・workflow gate に触れるなら R3）には当たらない: S1 は旧本体（停止中で test だけが呼ぶ）の在庫の書き方の記述を code に合わせるだけ、S3 は decision-log の Status の行だけで決定の本文を変えない、data safety の手順書（`docs/msi-distribution-runbook.md`・`docs/migration-recovery-runbook.md`）と workflow の正本は Non-scope。required gate の green / red は変わらない（`scripts/ci/classify-changes.sh` の workflow の一覧に当たる path を触らない。function-design を触るので traceability の分類には当たる）。Final Review Minimum は 1、Human Gate は `ready,merge`（R2 の既定）。

## Goal

Goal Invariant:

### 最小完了条件

- 実装に使う設計正本（function-design・architecture・db-design・ARCHITECTURE・DB_DESIGN・SCREEN_DESIGN・図面）を読んだ人が、今の build の振舞いを、実装済みの所は実装済みの契約として、まだの所は proposed・未実装として、取り違えずに読める。特に棚卸しの確定（旧本体）と商品別売上の取消の在庫の書き方が、今の code（記録から求め直して写しを書く）どおりに読める。
- decision-log の各 D の Status が、決定の状態だけ（accepted・superseded〈by D-xxx〉・withdrawn。owner の決定の前だけ proposed）を示し、置き換わった D から置き換えた D を辿れる（TD-327）。
- dashboard（`docs/Plans.md`・`docs/project-memory.md`・`docs/PROJECT_HANDOFF.md`）と `docs/backlog.md` を読んだ人が、今残っている作業と前提だけを読め、済んだ項目・置き換わった段取りの名前を今の作業として読まない。Draft PR #155 が backlog から辿れる。
- UI を実装・review する人が、class の上書きの 2 つの罠（S7-1）と owner の UI の好み（S7-2）を、個人の memory でなく tracked の正本から読める。

### 失敗定義

- 直した後の文が実装と食い違う（例: 35 §20.5 が `recompute_stock` と書くのに、code の旧本体の手順と順が違う。(b) の契約を (a) で実装済みと書く）。
- (b)（proposed・未実装）の契約を実装済みとして読める形に直す、または実装済みの (a) を未実装のまま残す。
- decision-log の決定の本文（Decision・Why・Compatibility 等の append-only の行）を書き換える。
- 店の資料・owner でしか確かめられない事実を、確かめずに直す・消す。
- 店の運用の細部・実値・発言の原文を repo に書く。
- workflow の正本や lane 2 の範囲（file:行 の引用の一般の書き換え）に手を広げる。

### 非目的

- code の振舞い・利用者に見える文言（例: ExportBar の Tooltip）を変える。
- decision-log の Status の規則そのものを正本に書く（並走の lane `agent/docs-writing-rules` の D-126）。
- file:行 の引用を symbol・節 ID へ一般に書き換える（lane 2、TD-326）。直す文の中の行番号だけは替えてよい。
- 店の資料・owner に聞く束を解く（別の段）。外部仕様の確認日を確かめ直す。
- backlog の「未照合」の項目（workflow の正本と Lane 2〜5 当時の候補）を照合する。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

not applicable: operator の操作・data / command の契約・業務の状態遷移を変えない docs の変更で、workflow の変更でもない（通常列は発注 → review → owner の Ready / merge）。

## Scope

段ごとの対象と直し方。行番号は起点 `18db621e` の値で、作業中にずれる（項目の先頭の語で引き直す）。各段の表の「結果」列は実装 run が「直した」か「残す（理由）」で埋める（AC-S2-7・AC-S4-7・AC-S5-6）。直す前に、主張の根拠（code・decision-log・archive の packet・PR）を `rg` か実読で確かめ、確かめられなければ直さずに「残す（未確認）」とする。

### S1 契約の誤り（最優先）

PR #176（D-118 の (a)）で、在庫の写しの書込みは `biz::stock_ledger::recompute_stock` だけを通る形になった（production で `inventory_repo::update_stock_quantity(` を呼ぶのは `recompute_stock` の中だけ。`rg -n 'update_stock_quantity\(|recompute_stock\(' src-tauri/src --glob '!*test*'` で確かめた）。設計正本の 32・35・24・44、上位の `docs/architecture/biz-task-specs.md`・`docs/ARCHITECTURE.md`・`docs/db-design/pos-tables.md` と図面が、今も写しを直接書く旧い手順・式を書く（下の洗い直しの表）。32・35・24 と biz-task-specs の BIZ-03・BIZ-06 の手順は現行 build では停止中の旧本体（§15.0・§20.0、呼出し元は `#[cfg(test)]` の test・診断・fixture だけ）の記述で、旧本体の記述であることは残す。

| # | file・節 | 今の文（要旨） | 直し方（code の根拠） | 結果 |
|---|---|---|---|---|
| S1-1 | `docs/function-design/35-biz-stocktake-service.md` §20.5 処理ステップ 5 g | 差異があれば `update_stock_quantity(actual_count)` の後に `insert_movement` | 補正の `insert_movement`（`business_date` = 確定した日〈Local の今日〉、`settlement_no` = なし）の後に `recompute_stock` で写しを記録から求め直す順に直す（`biz/stocktake_service.rs` の `legacy_complete_stocktake`）。記録の `stock_after` は `actual_count`。写しが記録と一致しているときは求め直しの値も `actual_count` と同じで、一致していなければ写しは求め直しの値になる（記録の `stock_after` は `actual_count` のまま） | 直した |
| S1-2 | 同 §20.5「設計判断 — apply_stock_change を使わない理由」の 3 項 | 1 項目が `apply_stock_change` の中を「find → stock計算 → update_stock_quantity → insert_movement」と書き、3 項目が「update_stock_quantity + insert_movement を直接呼び出す」 | `apply_stock_change` の今の中（movement を INSERT → `recompute_stock` → `update_movement_stock_after`、`biz/inventory_service/common.rs`）に合わせ、3 項目を「`insert_movement` で補正の記録を足し、写しは `recompute_stock` で求め直す（`update_stock_quantity` を直に呼ばない）」へ。使わない理由の 1（同じ商品を 2 回読む）は残す。理由の 2（368 行の「`stock_after` は actual_count そのもので、現在在庫 + 変動量の計算ではない」）は S1-1 の条件付きの形（写しが記録と一致しているとき求め直しの値と同じ）に直す | 直した |
| S1-3 | `docs/function-design/32-biz-csv-import-service.md` §15.5 処理ステップ 5 と「在庫補正の詳細」 | 無効にした movement を product_code ごとに合算し、`new_stock = stock + correction` を `update_stock_quantity` で書く | 無効にした movement の商品ごとに `recompute_stock` で求め直し、前後の写しを `StockCorrection` に入れる形へ（`biz/csv_import_service/commit.rs` の `apply_void_stock_corrections`、`rollback.rs` の `legacy_rollback_csv_import`）。値は、写しが記録と一致しているとき「写し − 無効にした数量」と同じ（同関数の doc comment）で、一致していなければ記録から求め直した値になる。補償の movement は作らない | 直した |
| S1-4 | 同 §15.5「設計判断 — apply_stock_change を使わない理由」 | 「ここでは直接 update_stock_quantity を使う」 | 「movement を足さず、無効にした記録を除いて `recompute_stock` で求め直す」へ | 直した |
| S1-5 | 同 §15.8 対応不変条件の INV-2 の行 | 「rollback 時は直接 update_stock_quantity」 | 「rollback 時は `recompute_stock` で求め直す」へ | 直した |
| S1-6 | `docs/architecture/biz-task-specs.md` の BIZ-06「棚卸し確定」の手順 2 と BIZ-03「ロールバック処理」の手順 2 | 「products.stock_quantityをactual_countに更新」「is_voided=1にした各inventory_movementsについてproducts.stock_quantityを逆方向に補正」 | 「補正の movement を足した後、`recompute_stock` で求め直す」「movement を無効にした後、その商品を `recompute_stock` で求め直す」へ。BIZ-03・BIZ-06 の手順に、停止中の旧本体の記述だという注記（停止 ADR を引く）を足す。頭の (a) の一覧に「旧本体の確定・取消は、補正した商品・無効にした商品だけを求め直す」の 1 句を足す | 直した |
| S1-7 | 同 BIZ-02 の 119 行（共通在庫処理の説明）と BIZ-03「確定」の 266 行 | 「products.stock_quantityの更新とinventory_movementsの記録」「products.stock_quantity更新 + inventory_movementsにINSERT」 | 「movement を足し、その商品を `recompute_stock` で求め直して写しを書く」へ（`biz/inventory_service/common.rs` の `apply_stock_change`）。266 行は旧本体の確定（停止中）の記述と注記する | 直した |
| S1-8 | `docs/function-design/24-io-csv-import-repo.md` の `VoidedMovement` の `quantity` の説明（65 行）、`void_movements_by_reference` の戻りの説明（340 行）、「BIZ-03 での在庫補正」の例と式（369〜372 行）、設計判断の SELECT → UPDATE の理由（376 行） | quantity を逆方向に変換して在庫補正に使う、`products.stock_quantity -= voided.quantity` | 「BIZ-03 は無効にした movement の商品を `recompute_stock` で求め直す（quantity は在庫補正に使わない）」へ。戻り値のうち BIZ-03 が使うのは `product_code`（`commit.rs` の `apply_void_stock_corrections` は `vm.product_code` だけを集める）。SELECT → UPDATE の理由は、無効にする movement の商品を BIZ に渡すためと直す。369 行の見出しの語は残してよい | 直した |
| S1-9 | `docs/ARCHITECTURE.md` 179 行（BIZ-02 の表の行）、`docs/db-design/pos-tables.md` 414 行（Z004 の確定）、`docs/function-design/44-cmd-inventory.md` 1099・1112 行（層の責務の表と INV-2 の行） | 「products.stock_quantity更新＋inventory_movements記録」「stock_quantity更新+movements記録」「stock_quantity更新とmovements記録の同時性」 | 「movement の記録と、記録からの写しの求め直し（同じ TX）」の向きに直す。pos-tables 414 は旧本体の確定（停止中）と注記する | 直した |
| S1-10 | `docs/function-design/35-biz-stocktake-service.md` §20.5 処理ステップ 3a（force_fill） | 未入力の明細に `product.stock_quantity` をそのまま入れる | code（`legacy_complete_stocktake` の `product.product.stock_quantity.max(0)`、コメント「R-3 + P3-7: 負在庫補正。INV-3準拠」）に合わせ、負の在庫は 0 を入れると直す。同じ手順の注記「※ 現在のシステム在庫をそのまま actual_count にセット（差異なし扱い）」も、「負の在庫は 0 を入れるので、その商品には差異が出る」に直す | 直した |
| S1-11 | `docs/diagrams/current-system.md` の「在庫の増減と取消」の図（176 行の `CHANGE` の節の「products更新 + inventory_movements追加」、184 行の ROLLBACK の「その増減を戻す」）と「棚卸しの時間と在庫」の図 | 増減は products を更新して movement を足す、取消で増減を戻す、確定で actual_count へ在庫を合わせる | S4 の H1 で直す（movement を足して記録から求め直す、無効にした記録を除いて求め直す、補正の movement を足して求め直す）。177〜181 行の矢印の「加算」「減算」は movement の向きの説明で残す。ここでは S1 の型の hit として数える | 直した |
| S1-12 | `docs/db-design/pos-tables.md` の Rollback の手順 2（425 行） | 「movement を product_code 別に逆補正」 | 「無効にした movement の商品ごとに `recompute_stock` で求め直す」へ。停止中の旧本体の記述だと注記する | 直した |
| S1-13 | `docs/function-design/35-biz-stocktake-service.md` §20.8 対応不変条件の INV-2 の行（540 行） | 「新方式はBIZが補正量N-Lとstock_after=確定直前現在庫+(N-L)をchecked算出し、IOへ渡す」 | 「新方式」は置き換わった時点証拠 ADR の確定の式。D-118 で置き換わったので、旧本体の今の形（補正の movement の `stock_after` は `actual_count`、写しは `recompute_stock`。S1-1 の条件付きの形）と、D-118 の (b) の確定は 35 の冒頭の D-118 の節を引く形に直す | 直した |

S1 の型の洗い直し（round 2 の是正で足し、round 3 の是正で語の幅を「逆補正・逆方向・相殺・加算・減算・補正の和・在庫 +・在庫 -・`+=`・`-=`」まで広げて洗い直した。`docs` から `docs/archive/`・`docs/plans/`・`docs/research/`・`docs/evidence/` を除いた範囲と、`src`・`src-tauri/src` の comment。docs の本体は起点 `18db621e` と同じ、tip `578a863f`、2026-10-19）:

```text
docs:    rg -n --no-heading -e 'stock_quantity\s*[-+]?=' -e 'stock_quantity.{0,40}(更新|補正|逆方向|差分|加算|減算|戻す|合わせ|上書き)' -e '(更新|補正|逆方向|差分|加算|減算|上書き).{0,40}stock_quantity' -e 'update_stock_quantity' -e '[-+]=\s*.*quantity' -e '在庫補正' -e '逆方向' -e '(在庫|増減)[^。|]{0,6}を戻す' -e '在庫を[^。|]{0,8}補正' -e '逆補正' -e '相殺' -e '加算' -e '減算' -e '補正の和' -e '在庫 ?[+＋]' -e '在庫 ?[-−ー]\s?[0-9（(]' -e '\+=' -e '-=' -g '!docs/archive/**' -g '!docs/plans/**' -g '!docs/research/**' -g '!docs/evidence/**' docs
         → 286 行（round 2 の式の 80 行 + 広げた語で増えた 206 行）
comment: rg -n --no-heading -e '//.*(stock_quantity\s*[-+]?=|stock_quantity.{0,40}(更新|補正|逆方向|差分|加算|減算|戻す|合わせ|上書き)|(更新|補正|逆方向|差分|加算|減算|上書き).{0,40}stock_quantity|update_stock_quantity|[-+]=\s*.*quantity|在庫補正|逆方向|(在庫|増減)[^。]{0,6}を戻す|在庫を[^。]{0,8}補正|逆補正|相殺|加算|減算|補正の和|在庫 ?[+＋]|在庫 ?[-−ー]\s?[0-9（(])' src-tauri/src src --glob '!src/lib/bindings.ts'
         → 38 行（round 2 の式の 18 行 + 20 行）
```

広げた語で増えた 206 行の分類: 直す 6（pos-tables:425〈S1-12〉、diagrams/current-system:176〈S1-11〉、35:368〈S1-2 の理由の 2〉・540〈S1-13〉、23:245・352〈S1 の型でなく D-118 で外した「相殺の判定」を入力の理由に書く文。S2-16〉）、正しい 49、対象外 151（置き換わった時点証拠 ADR の本文 62、design-system/reference の mockup と原文 25、screen_mockups.html 1、decision-log の本文 6、売上の集計・金額・日付の「加算」と表示の「相殺」と PowerShell の `+=` 50、backlog 7〈S5 の行が扱う〉）。正しい 49 は、movement の向きとしての在庫の加算・減算（入庫・手動販売・廃棄・返品の画面と設計、`戻り（在庫+）`・`渡し（在庫-）` の表示、`pos_stock_sync` の意味）、入力の行の数量の加算、二重減算の問題の記録（transaction-tables・cross-feature-verification・停止 ADR）、旧 ADR の相殺の判定を外したことの記録（23:9、29:423・439・446、32:5・30、55:8）、`docs/function-design/10-common-rules.md` の上限の見積もりの表の「在庫 + 1 明細の数量」「在庫 + 補正の和」（180・181 行。正しい〈上限の見積もり〉。取消は求め直しになったが、求め直しの値は同じ範囲に入る）。

| 分類 | 数 | 行（file:起点の行） |
|---|---|---|
| 直す（S1 の行） | 34 | 下の round 2 の 28 行に、広げた語の 6 行（上の段落）を足す |
| 正しい（残す） | 86 | round 2 の 37 行（下）+ 広げた語の 49 行（上の段落） |
| 対象外（理由） | 166 | round 2 の 15 行（下）+ 広げた語の 151 行（上の段落） |
| code の comment | 38 | 37 行は「正しい」か「対象外」（下の round 2 の 18 行、売上・表示の「相殺」「加算」、日付の加算、`ListShell` の隙間の相殺、cfg(test) の時点証拠の comment）。1 行は古い: `src-tauri/src/db/sales_repo.rs:165` の「ロールバック時の逆補正用（product_code + 元のquantity）」（今の BIZ-03 は quantity を使わない）。本 lane は code を変えないので、S5 で backlog に 1 行足す |

round 2 の式の 80 行の分類（round 2 の是正のときの表）:

| 分類 | 数 | 行（file:起点の行） |
|---|---|---|
| 直す（S1 の行） | 28 | 35:335・367・369（S1-1・S1-2）、32:411・413・421・432・433・434・435・437・530（S1-3〜S1-5。411・432 は手順と節の見出しで、語は残し下の本文を直す。413 は見出しの語〈在庫補正〉は残し、括弧書き〈product_code でグループ化して集計〉を直す）、biz-task-specs:278・488（S1-6）、biz-task-specs:119・266（S1-7）、24:65・340・369・370・371・372・376（S1-8）、ARCHITECTURE:179・pos-tables:414・44:1099・44:1112（S1-9）、diagrams/current-system:184（S1-11） |
| 正しい（残す） | 37 | 求め直しの文と写しの書込みの関数の定義: 21:5・10・383・389・392・395、31:7、20:9、master-tables:7・8、transaction-tables:7、ADR（計数の置き場）:16・57、36:213（計数の無い商品に限った正しい等式）、36:159（操作ログの文言）。在庫を変えない・条件の文: 37:296、51:28、35:156・178・268・290・336、biz-task-specs:473、44:784、24:688（責務の所在）、ARCHITECTURE:151（整合性の補正の画面）、75:49（query の無効化）、ACCEPTANCE_SCENARIO:134（符号の説明）、DB_DESIGN:85・101。業務記録の取消・訂正の設計（逆方向の movement を足す、(b) と別の未実装の設計）: transaction-tables:60、tracking-system-tables:99、65:46・206・207・209・210 |
| 対象外（理由） | 15 | 置き換わった時点証拠 ADR の本文（superseded の記録）: adr/2026-09-18-stocktake-time-evidence:58・180・271・278・284・314・373。decision-log の決定の本文（append-only）: 385・387（D-051、D-118 の Superseded in part の行あり）・397（無関係の「逆方向」）・927・928（D-109）。参照資料: screen_mockups.html:194、design-system/reference/2026-08-20-proposal-A-rules-bound:133。backlog:53（S5 の 53〜55 の行が扱う） |
| code の comment | 18 | 全部「正しい」か「対象外」（本 lane は code を変えない）。在庫補正の語は取消の補正の結果の名（`commit.rs`・`rollback.rs`・`mod.rs`・`integrity_service.rs`）、test の comment は結果の等式（`stocktake_service.rs` の 1285・1286・1568）、`common.rs:348` は `recompute_stock` の中の `update_stock_quantity` の戻り、`sales_repo.rs:410` は戻り値を BIZ-03 が使うこと。古い手順を書く comment は無かった |

32・35 の冒頭の D-118 の節（(b) の契約。取消は計数を持つ全ての商品を求め直す等）は (b) の範囲のまま変えない。(a) が旧本体に入れたのは、無効にした movement の商品・補正した商品だけの求め直しで、その違いは §15.5・§20.5 の側に書く。

### S2 設計の proposed・予定の書き分け

型: (a)/(b) の書き分けは既に書き分け済みの文書（`docs/DB_DESIGN.md`・`docs/ARCHITECTURE.md` 等の「（D-118。(a) は実装済み、(b) は proposed・未実装）」）と同じ形にし、本文に「(a) で入ったのは …（module・関数）」を 1 文足す。「future」「予定」は実装済みなら現行の契約の文へ直す。

S2 の型の洗い直し（round 2 の是正で足した）: 「future」「予定」を外すだけでは、周りの説明（file 名・route・component・command の名前・属性・型）が現物と違うまま残る。各行で、外す語の周りに出る名前を `rg` と `ls` で現物と照らし、結果を「周りの説明の現物」の列に書いた（tip `79d51fb1`、2026-10-19）。現物と違う所は全部その行の直し方に入る。

| # | file・起点の行 | 今の文（要旨） | 直し方 | 周りの説明の現物 | 結果 |
|---|---|---|---|---|---|
| S2-1 | `docs/function-design/73-ui-stocktake.md` の 18 行（29・33・46・62・86・157・173・174・180・207・210・281・289・291〜295） | 読取りの command・hook・route を「future」と書く | 括弧書きの「future」を外し、29 行の「以降 future と明記した契約は未実装」の文を今の状態に直す（`lib.rs` の command 登録 `get_active_stocktake`・`find_stocktake_item`・`get_last_completed_stocktake`、`src/features/stocktake/hooks/`、`src/routes/stocktake.tsx`）。あわせて UI-10-D7 の決定の文（102 行）と 279 行の「既存 4 command は `#[tauri::command]` だけで `#[specta::specta]` が無く bindings に型が無い」を、今の状態（`src-tauri/src/cmd/stocktake_cmd.rs` の `start_stocktake`・`get_stocktake_items`・`update_count`・`complete_stocktake` に `#[specta::specta]` があり、`src/lib/bindings.ts` に 4 関数がある）に直す。決定の中身（生 invoke を禁じ specta で揃える）は残す。§73.4 の file tree を現物に書き直す（component は別 file でなく `src/features/stocktake/StocktakePage.tsx` の export、route は `src/routes/stocktake.tsx` の layout と `src/routes/stocktake/index.tsx` の `validateSearch`）。29 行の「本書は runtime 実装（route / components / hooks / specta 化）を含まない」を直し、IO の `find_active_stocktake` を「`start_stocktake` から呼ばれている」とする句は消すだけにする（呼出し元を列挙しない） | hooks の 6 file は `src/features/stocktake/hooks/` に在る。tree の component の 7 file（`StocktakeStartPanel.tsx` ほか）は無く、7 つとも `StocktakePage.tsx` の export（`StocktakePage`・`StocktakeStartPanel`・`StocktakeProgressHeader`・`StocktakeCountEntry`・`StocktakeItemList`・`StocktakeCompleteDialog`・`StocktakeResultPage`）。route は `src/routes/stocktake.tsx` と `src/routes/stocktake/index.tsx`（`validateSearch`）と記録詳細の `src/routes/stocktake.records.$stocktakeId.tsx`。command は `lib.rs` の登録に在る。BIZ の 3 関数は `biz/stocktake_service.rs`、IO の `find_active_stocktake`・`find_stocktake_item_by_code`・`find_last_completed_stocktake` と型の `LastStocktakeSummary`・`StocktakeItemDetail` は `db/stocktake_repo.rs` に在る。`find_active_stocktake` を呼ぶのは `get_active_stocktake` と旧本体（停止中）で、`start_stocktake` は停止の error を返す | 直した |
| S2-2 | `docs/DEV_SETUP_CHECKLIST.md` 23・511・513・515・528・544（§14） | Phase 3・4 を「参考、未着手」「着手予定」「v1.0.0 予定」 | 画面の実装が済んだことだけを完了と書く（`docs/SCREEN_DESIGN.md` の Phase 4「完了」、`src/routes/` の画面）。「v1.0.0」は完了と書かない（release は D-070 で自動在庫連動の完成まで待つ）。§14 は見出しの下に画面の実装の完了の 1 行を置き、本文は当時の記録として残す（archive へ移さない） | `src/routes/` に Phase 3・4 の画面の route が在る。tag は public repo に無い（`git tag` 0 件）。v1.0 の release の条件は D-070 | 直した |
| S2-3 | `docs/FUNCTION_DESIGN.md` 3 | 見出し「（proposed・未実装、D-118）」だけ | (a)/(b) の書き分け（一覧の 21・31・36・75・22 §19 は (a)） | (a) は `db/schema_v9.rs`・`biz/stock_ledger.rs` に在る。root の `README.md` が FUNCTION_DESIGN の「現時点の対象モジュール」の見出しを anchor で引くので、その見出しは替えない | 直した |
| S2-4 | 同 18・100・177（§今後の追記予定） | 「後続段階は実装進行に合わせて追記」、全項目が取消線の節 | 文を今の状態に直し、節を消す | 節の項目は全部取消線（179〜184 行）。この見出しへの anchor は無い（S5 の anchor の検索） | 直した |
| S2-5 | `docs/function-design/43-cmd-settings-log.md` 3 | 見出し「（proposed・未実装、D-118）」 | (a)/(b) の書き分け（写しは記録から求め直す、写しの書込みは 36 → 31。`biz/integrity_service.rs`・`biz/stock_ledger.rs`） | `biz/integrity_service.rs` の `recompute_stock` の呼出しと `biz/stock_ledger.rs` の `recompute_stock` の定義が在る | 直した |
| S2-6 | `docs/function-design/60-ui-product-import.md` 3 | 同 | (a)/(b) の書き分け（新規行の初期在庫は初期在庫の movement、写しは求め直し〈`biz/product_service.rs` の `recompute_stock` の呼出し〉は (a)。共有 JAN・連動を始める行の拒否は (b)） | `biz/product_service.rs` に `recompute_stock` の呼出しが 2 か所在る | 直した |
| S2-7 | `docs/function-design/61-ui-receiving.md`・`62-ui-manual-sale.md`・`63-ui-return-exchange.md`・`64-ui-disposal.md` 各 3 | 冒頭の注記「（proposed・未実装、D-118）」 | 日付を業務日として movement に書くのは (a)（`biz/inventory_service/` の `validate_business_date`）。計数と入出庫の窓の計算（`biz/stock_ledger.rs` の `judge` → `from_count` → `derive`、test `biz::stock_ledger::tests::test_stock_ledger_req205_default_count_vs_movement_dates`）も (a) で実装済み。計数を保存する公開の経路・精算の窓・この画面への反映は (b) と書き分ける | `validate_business_date` は `biz/inventory_service/common.rs` に在る（個別の画面の file でない）。`judge`・`from_count`・`derive` と test は `biz/stock_ledger.rs` に在る | 直した |
| S2-8 | `docs/function-design/22-mnt-migration.md` 360 | 「`stock_counts.count_value` と写しの範囲の列は、それを足す runtime の lane が上限で検査する」 | 列は (a) の v9 で入った（`db/schema_v9.rs`）。範囲の検査の担い手を code で確かめて今の状態に直す | 写しの値と範囲は `recompute_stock` が書く前に検査する（test `test_stock_ledger_req903_recompute_rejects_out_of_range_value_and_range_end`）。`count_value` の列の CHECK は `>= 0` だけで、上限の検査は計数を保存する経路（(b)）の側 → その形で直す | 直した |
| S2-9 | `docs/architecture/io-task-specs.md` 7 | 「新しい SQL / schema は未実装」 | (a) の v9 は入った、Z004 の識別の列は (b) と書き分ける | v9 が足したのは movement の並びの鍵の列・`stock_counts`・写しの列（`db/schema_v9.rs`）。Z004 の取込みの記録の識別の列は production の schema（`src-tauri/src/db/schema_v*.rs`）に 0 件（`rg -n 'machine_no' src-tauri/src/db -g 'schema_v*.rs'` は 0 件。cfg(test) の `schema_time_evidence.rs` の 2 行は production の schema でないので数えない）→ (b) | 直した |
| S2-10 | `docs/UI_TECH_STACK.md` 785、`docs/design-system/02-component-catalog.md` 1041 | 「将来の global text-size / display-scale option」「将来の全体文字サイズ / 表示スケール option」と合わせて調整 | option は入った（`src/components/layout/useDisplayScale.ts`）。「将来の」を外し、商品コードの読みやすさの調整が済んだかは確かめられた所だけ書く。確かめられなければ backlog の 1 行へ送る | `src/components/layout/useDisplayScale.ts` は在る | 直した |
| S2-11 | `docs/function-design/54-ui-shortcuts.md` 207 | unit test の見送りの理由「Vitest 初期化が未着手」 | 理由の文を今の状態（Vitest は入った、`useShortcutsDialog`・`isEditableTarget` の unit test はまだ無い。backlog の shortcuts の retroactive unit test の項目）に直す。test は足さない | `useShortcutsDialog` は `src/features/shortcuts/hooks/useShortcutsDialog.ts` の export、`isEditableTarget` は同じ file の非公開の関数（表の「純関数化部分」の句も今の形に直す）。`src/features/shortcuts/` に test の file は無い | 直した |
| S2-12 | `docs/function-design/55-ui-csv-import.md` 512 | 「Vitest 未着手のため、本 PR では unit test なし」 | 当時の記録と分かる形に直す（今は `src/features/csv-import/` に test がある） | `src/features/csv-import/` に test の file が 9 つ在る | 直した |
| S2-13 | `docs/function-design/56-ui-daily-sales.md` 518・596 | 「TanStack Query Provider mock / Router mock は 7-7b 後続 PR」 | 済んだ（`QueryClientProvider` を使う test がある）と直す | `rg -l QueryClientProvider src --glob '*.test.*'` が 55 file | 直した |
| S2-14 | `docs/function-design/51-ui-product-form.md` 28（UI-01b-D5） | 「`stock_unit` も初回 UI-01b 実装では読取専用」、理由の列の「単位変更は … 影響が大きいため別 Design Phase」 | 「単位変更の範囲は D-119 で個数の単位どうしに限って決めた（UI-01b-D23）」の向きに直す（backlog の同じ項目の子、PR #171 の Final Review の P3） | UI-01b-D23 は 51 の 29 行に在る | 直した |
| S2-15 | `docs/function-design/29-io-ej-parser.md` の番号行の 4 桁の欄（IO-08 の該当の節）と、未観測の形式を復元不能に倒す列挙 | 4 桁の欄の意味は未検証、未観測の形式（訂正・取消・値引き・番号印字・点検・小数数量・クレジット） | 今の文法表と `src-tauri/src/io/ej_parser.rs` で受理している形式を確かめ、受理済みの形式を列挙から外す。4 桁の欄は持ち帰りデータの観測（`docs/project-memory.md` の該当の行）を書き、観測を対応の規則に使わない注意は残す。code で確かめられない形式は残す | `src-tauri/src/io/ej_parser.rs` は在る。受理している形式の一つ一つは実装 run が文法表と code で照らす | 直した |
| S2-16 | `docs/function-design/23-io-z004-parser.md` の 13 桁 JAN の表の行（245 行）と、責務の表の空レコード除外の行（352 行） | 空レコードを「相殺の判定（ADR D4）の入力に残す」「相殺の判定の入力に残すため」 | D-118 で相殺の判定は持たない（23 の冒頭〈9 行〉と 32 の冒頭の D-118 の節が書く。数量・金額とも 0 の行は売上も movement も作らない）。残す理由の句を今の形（BIZ-03 が空レコードとして除外する）に直す | 23:9 が「売上と返品の相殺の判定を持たない。SPEC-STK-CNT-D11」、32:30 が「相殺の判定に使わないので保持しない」と書く。round 3 の S1 の型の語の幅の洗い直しで見つけた | 直した |

S2 に入れない (2) 済（既に書き分け済み）と (3)（まだ実装なし。正しい）は直さない（調査 A の分類）。D-118 の節のうち全体が (b) の文書（40・41・42・50・51・55・58・65・66・73・SCREEN_DESIGN の画面の節・UI_TECH_STACK の D-118 節ほか）は「（proposed・未実装、D-118）」のまま残す。

### S3 decision-log と ADR の Status

TD-327 の規則（Status は決定の状態だけ。実装が済んだかは code と Plans が持つ）を適用する。直すのは既存の D の `- Status:` の行、足す `- Superseded by:` の行、D-071 の `- Superseded in part by:` の 1 行だけで、決定の本文（Decision・Why・Alternatives・Compatibility・Guarantee range・Revisit・Activation・owner の決定の各行）は書き換えない（AC-S3-7）。規則の文の正本化は並走の lane（D-126）が持ち、D-126 が先に merge したら Status の語と括弧の形をその文に揃える。

| # | D | 今の Status（要旨） | 直し方 | 結果 |
|---|---|---|---|---|
| S3-1 | D-117・D-118・D-119・D-120・D-121・D-122 | `proposed（design lane … の design の出力。… owner の決定は … 正本は … runtime は後続の lane）` | 先頭の `proposed` を `accepted` にし、末尾の「runtime は後続の lane」（実装の状態）を外す。出所（design lane・Plan Packet・owner の決定の TD・正本の一覧）は残す | 直した |
| S3-2 | D-116 | `proposed（…runtime は後続の lane）`、別の行に `Superseded by: D-118` | `superseded（by D-118。…）` にし、「runtime は後続の lane」を外す | 直した |
| S3-3 | D-056・D-058・D-079・D-084・D-087 | `accepted（…）`。D-092 の Compatibility が「全面 superseded」と書くが、各 D に Superseded の行が無い | 各 D に `- Superseded by: D-092（2026-09-25）。…` の行を足し、Status を `superseded（by D-092。元の出所）` にする（D-116 の行の形） | 直した |
| S3-4 | D-085 | Status の行が無い | `- Status: accepted（…）` の行を足す。出所は git log・archive・本文の Activation の行で確かめた所だけ書く | 直した |
| S3-5 | D-040 | `accepted. … Phase B remains a separately gated R4 migration.` | 実装の状態の語（remains）を外す。Phase B を R4 で別に通すことが決定の中身なら、時点の語を除いた形で残す | 直した |
| S3-6 | D-041 | `… the later migration execution remains R4.` | 同じ | 直した |
| S3-7 | D-043 | `accepted; Plan Gate passed and implementation completed …, pending Final Review and Human Gate.` | 実装・review の状態を外し、決定の出所だけを残す | 直した |
| S3-8 | D-048 | `accepted（design phase、実装は未着手）` | 「実装は未着手」を外す | 直した |
| S3-9 | D-049 | `accepted（2026-07-18。… 実装は R3 packet で進行）` | 「実装は R3 packet で進行」を外す | 直した |
| S3-10 | D-066 | `accepted（…。導入と DEV_SETUP_CHECKLIST 追随は merge 後の follow-up）` | 「導入と … 追随は merge 後の follow-up」を外す | 直した |
| S3-12 | D-034 | `accepted and implementation slice 1 reflected in PR #163. … only PK4/PK5/… remains follow-up slice 2. …` | 実装の状態（slice 1 の反映・slice 2 が残る）を外し、決定の出所を残す。「PR #163」は履歴境界の前の番号なので、残すなら境界の前の注記を付ける。同じ注記を D-035 の Status（「in PR #163 follow-up」）と D-039 の Status（「PR #166.」）にも当てる（どちらも境界の前の番号で、今の repo の #163・#166 と別物。`docs/PROJECT_HANDOFF.md` の履歴境界の節） | 直した |
| S3-13 | D-038 | `accepted. D-034 slice 2's future mechanical checks …`（検査の対象に足す語彙の列挙が続く） | 「future mechanical checks」の時点の語を外す。検査の対象に足す語彙は決定の中身なので、時点の語を除いた形で残す | 直した |
| S3-14 | D-045 | `accepted during Phase B execution …. The migration and first public hosted CI green are complete.` | 末尾の「… are complete」（実装の状態）を外す。採った時点と決定の中身（D-041/D-042 の裁定の扱い）は残す | 直した |
| S3-15 | D-046 | `accepted and implemented by the D-045 follow-up workflow-guards R3 change.` | 「and implemented by …」を外す | 直した |
| S3-16 | D-111 | `accepted（…。runtime は後続の lane）` | 「runtime は後続の lane」を外す | 直した |
| S3-17 | D-113 | `accepted（…、runtime は後続の単位の lane と原価の lane）` | 「runtime は後続の単位の lane と原価の lane」を外す | 直した |
| S3-18 | D-071 の `- Superseded in part by: D-121（2026-10-17、proposed）。…` | D-121 を proposed と書く | 「、proposed」を外す（D-121 は S3-1 で accepted） | 直した |
| S3-19 | D-100・D-101・D-102・D-103・D-104・D-105・D-106・D-107 | `accepted（owner … Plan Packet … の設計判断。…（正本・script・test・文書・code）への反映は同 packet の実装の PR で行う）`（D-103・D-105 は「実装は同 packet の PR で行う」、D-104 は「runtime は同 packet の実装の PR」） | 反映の予定の句（実装の後に古くなる実装の状態の一種）だけを外す。決定の内容と出所（owner の判断・Plan Packet・設計判断の ID・plan-first の commit で記録したこと）は残す。D-106・D-107 の「merge で accepted にする。PR #138 で merge、2026-10-04」の類は、accepted になった時点の出所として「PR … の merge で accepted（日付）」の形に残す | 直した |
| S3-11 | ADR `docs/adr/2026-10-16-stocktake-count-placement.md` の Status、`docs/adr/README.md` の同 ADR の行 | ADR は `proposed — 2026-10-16、…`、索引は「（proposed）」と「D-118 の design 出力であり未実装」 | ADR の template（`docs/templates/adr.md`）の 3 値は decision-log と同じ意味で、D-118 が accepted になるので ADR も `accepted` にする。索引の「（proposed）」を「（accepted）」に、「であり未実装」を外す | 直した |

S3 に入れない: D-092 が部分的に置き換えた D（D-034・D-035・D-038・D-046・D-049・D-055・D-062・D-065・D-074・D-085）の `Superseded in part by: D-092` の行（Coordinator の裁定の外）。

### S4 dashboard と図面

行の記号は調査 C の表 2-1 の番号（P = Plans、M = project-memory、H = PROJECT_HANDOFF、A = ARCHITECTURE、D = DB_DESIGN、SC = SCREEN_DESIGN。調査 C の S1〜S6 は本 packet の段の名と重ならないよう SC1〜SC6 と書く）。調査 C の P8（完了した wave の段落）は P1 の行にまとめた。

| # | file・起点の行 | 今の文（要旨） | 直し方 | 結果 |
|---|---|---|---|---|
| P1 | `docs/Plans.md` 13・17・18・19 | wave 13・ハーネス刷新・2026-10-04 の 4 lane・wave 14 の段落（全て merge 済み）、次の lane は owner が決めるという当時の文 | 消す。消す前に、段落の中で他に記録の無い owner の決定（例: wave 13 の「同居禁止を適用しない」）が decision-log か archive の packet にあるかを `rg` で確かめ、無ければ段落を `docs/archive/harness-context/2026-09-30-Plans-completed.md` の末尾の節へ移す | 直した |
| P2 | 同 22〜24（次の着手順、owner 決定 2026-09-22） | ㉘ の次は ④ 取込み・取消・回復と ⑤ 一括切替 | ㉘ ④⑤ は D-118 で「棚卸しの runtime (b)」（backlog の項目）に置き換わった。今の行き先を指す 1 文にし、済んだ 1 は消す | 直した |
| P3 | 同 30 | 製品作業の既定順序は backlog の「次に動く lane（順番固定）」を維持 | その節は完了の行だけ（S5 で移す）なので消す | 直した |
| P4 | 同 34 | 「migration vL で …」 | 実体の番号 v9 にする | 直した |
| P5 | 同 51（ブロッカー） | 残る原価（C）・数量（P）・棚卸しの再開（⑤）は「単位の拡張」の項目 | ⑤ は棚卸しの runtime (b)、C は原価の lane の項目を指す | 直した |
| P6 | 同 55・57・58（`## 製品の` で始まる判断待ちの節） | L8-4 の runtime 反映は表示小修正 batch 2 に同乗 | batch 2 は完了（backlog の完了の行、PR #64）。L8-4 をその節から外す | 直した |
| P7 | 同 58 | L8-5 は「④ C5 追跡中」 | 移送前の番号。backlog の L8-5 の項目を指す句にする | 直した |
| M1 | `docs/project-memory.md` 46 | 最新のアプリ生成 `.txt` の再確認は Post-UI-08 follow-up として残る | 済んだ（`docs/plu-export-and-real-csv-verification.md` の 2026-07-06 の記録）と直す | 直した |
| M2 | 同 50 | 「18 original tables」、日報の表は planned | 日報の表は v4 で実装済み（`db/schema_v4.rs`）。表の数は今の値を書かず DB_DESIGN を引く | 直した |
| M3 | 同 67 | 「Fabric may be managed in `cm`」 | 長さの商品は m で入力・表示（D-113）、cm は作り直しの起点で外す（D-119）と直す | 直した |
| M4・T1 | 同 250 | 「PLU の小数の Z004 は 2026-09-30 に試す予定」、parser の時点の振舞い | 予定は済んだ（同じ file の店主回答の行、backlog の入力の (e)）と直す。parser の振舞いは今の code（`io/daily_report_parser.rs`・`io/ej_parser.rs`）で確かめた所だけ書く | 直した |
| M5 | 同 62 | 「Stocktake may proceed while CSV import remains allowed」 | 設計の事実としては D-118 でも成り立つ。今の build は停止 ADR で両方を止めている注記を足す | 直した |
| H1・D2 | `docs/PROJECT_HANDOFF.md` 15、`docs/DB_DESIGN.md` 15 が引く `docs/inventory_system_erd.html`・`docs/diagrams/current-system.md` | ER 図の対象は v1〜v7、v8・v9 の列と `stock_counts` が無い。図面の在庫の流れと棚卸しの確定が「products 更新」「actual_count へ在庫を合わせる」 | ER 図を v9 まで（`stock_counts`、写しの列〈要再確認の印・範囲〉、movement の並びの鍵の列、v8 の単位の CHECK）に合わせ、図面の在庫の流れを「movement を足して記録から求め直す」に直す。列と制約の正本は db-design と `src-tauri/src/db/schema_v8.rs`・`schema_v9.rs` | 直した |
| H2 | `docs/PROJECT_HANDOFF.md` 13 | 「更新の前の版へ戻す管理者の手順（migration v8 以降）」 | 残す（見出しは成り立つ。runbook の本文に v9 の例を足すのは data safety の手順書の変更で、本 lane の外） | 残す（見出しは成り立つ。runbook の本文に v9 の例を足すのは data safety の手順書の変更で、本 lane の外） |
| A1 | `docs/ARCHITECTURE.md` 301 | Q40 は第6段階では保留、詳細は Plans.md Backlog | 置き場を `docs/backlog.md` に直し、S5 で 1 つにまとめた Q40 の項目（包括的な障害時対応の方針の残り。画面ごとの CmdError と retry、共通の Error Boundary は済み）を指す | 直した |
| A2 | 同 315 | UI 層は「UI-00、UI-01a〜UI-13」 | `docs/architecture/ui-task-specs.md` の見出しにある番号まで広げる | 直した |
| A3 | 同 183（BIZ-06）と 167（CMD-07）・180（BIZ-03） | BIZ-06 の確定は「差異補正の inventory_movements 記録」、CMD-07 は「商品別売上と`pos_stock_sync`在庫増減は実装済み」、BIZ-03 は「在庫引落し、…、ロールバック」 | 3 行とも停止 ADR の注記（棚卸しの開始・入力・確定と Z004 の業務 commit・取消は今の build で止めている）を足し、確定・取込みの新しい形は D-118 の (b) と書き足す | 直した |
| A4 | 同 262〜280、`docs/SCREEN_DESIGN.md` 435・442 | tag と「tag 済み」「tag を作成済み」 | 履歴境界の前の記録（`docs/PROJECT_HANDOFF.md` の履歴境界の節）と分かる注記を付ける | 直した |
| D1 | `docs/DB_DESIGN.md` 5・20〜49・124〜138 | 表の一覧と index に v9 の `stock_counts` と新しい index が無い（5 行が「まだ載せていない」） | 一覧に `stock_counts` を、index に `idx_inventory_movements_ledger`・`idx_stock_counts_product_id` を足し、5 行の文を直す（`db/schema_v9.rs`） | 直した |
| SC1 | `docs/SCREEN_DESIGN.md` 3 | 見出し「（proposed・未実装、D-118）」 | (a)/(b) の書き分け（整合性検証〈75〉と操作ログ〈74〉の表示は (a)） | 直した |
| SC2 | 「最終更新」の見出しの 8 file: `docs/SCREEN_DESIGN.md`・`docs/plu-export-and-real-csv-verification.md`・`docs/TOOLING_SKILL_COMMANDS.md`・`docs/DEV_SETUP_CHECKLIST.md`・`docs/FUNCTION_DESIGN.md`・`docs/DOCKER_REPAIR_LOG.md`・`docs/UI_TECH_STACK.md`・`docs/function-design/50-ui-product-list.md` | 見出しの日付が git の最終 commit より古い | 見出しの行を消す（更新日は git が持つ） | 直した |
| SC3・SC5 | `docs/SCREEN_DESIGN.md` 55・183 | 「PR #164 で実装済み」、UI-11c の fresh 再監査と L3 は未完了 | PR 番号に履歴境界の前の注記を付ける。「未完了」は owner に聞く束（S6）へ回して残す | 直した |
| SC4 | 同 444 | Phase 3「タグ目標」 | Phase 3 の画面は実装済みと直す（A4 と同じ注記。v1.0 の release は D-070 で待つので完了と書かない） | 直した |

S4 の型の洗い直し: 採用済みで未実装の決定と今の build の取り違え（round 3 の是正で足した）。accepted になった決定の中身を、今の build の動きとして書いている live な文を、dashboard の 7 file で洗い直した。直し方の型は 1 つ: その文を「採用済みの方針（D の番号）／今の runtime（code の現物）／残る実装（backlog の runtime の項目）」に書き分ける。採用済みの方針の文そのものは消さない。

今の runtime（code で確かめた、tip `578a863f`）:
- D-111（SD 直読み）: 未実装。runtime は Draft PR #172。今の日報の取込みは利用者が file を選ぶ（`rg -n 'SD から読む' src --glob '!*.test.*'` は 0 件）。
- D-117（PLU の書出しの見直し）: 未実装。今の build は商品一覧の一括の PLU 対象化（`src/features/products/ProductListPage.tsx` の `bulkSetPluTarget`）、Clear 行（`src-tauri/src/constants.rs` の `PLU_CLEAR_ROW_ENABLED = true`）、`release_pending` を持つ。正本の 33 §16 の冒頭が「runtime の状態（2026-10-16）」の注記でこれを書いている。
- D-118 は (a) だけ実装済み（S2・SC1）。D-119 は単位の語の直しだけ実装済み（PR #171）。D-120・D-121・D-122 は未実装で、dashboard の該当の文は「proposed・未実装」か「後続の runtime」と書き分け済み。

```text
rg -n -e 'D-111|D-117|D-118|D-119|D-120|D-121|D-122' -e 'clear 行|clear行|Clear 行|枠を消さ|枠を解放|解放|段階導入|一括の対象|一括 PLU|占有|照合日時|SD から読|SD を直接|直接読|SD から直接|日報の紙|レシートの控え|一括取込み|前の JAN の別名|別名' docs/Plans.md docs/project-memory.md docs/PROJECT_HANDOFF.md docs/backlog.md docs/ARCHITECTURE.md docs/DB_DESIGN.md docs/SCREEN_DESIGN.md
→ 120 行（Plans 14、project-memory 17、PROJECT_HANDOFF 0、backlog 56、ARCHITECTURE 7、DB_DESIGN 11、SCREEN_DESIGN 15）
```

| # | 決定 | file・起点の行 | 今の文（要旨） | 直し方 | 結果 |
|---|---|---|---|---|---|
| DR1 | D-111 | `docs/project-memory.md` 40・100・197 | 毎日の売上データはアプリが SD から直接読み、CV17 は PLU の書込みだけ | 採用済みの方針（owner の判断 2026-10-06）として残し、今の runtime（日報は利用者が file を選んで取り込む。SD 直読みは Draft PR #172）と残る実装（SD 直読みの runtime、`XZ_BKUP` へ移す (b)）を書き分ける | 直した |
| DR2 | D-111 | `docs/DB_DESIGN.md` 83 | 日報の表は、アプリがレジの SD から直接読む日報を持つ | 同上（今は利用者が選ぶ file から入る） | 直した |
| DR3 | D-111 | `docs/SCREEN_DESIGN.md` 81・118・126・127 | 1 日の動線と日報取込みの導線が「SD から読む → 精算を選ぶ → 確認 → 完了」 | 画面の設計（採用済み）であることを残し、今の画面は file を選ぶ導線で、「SD から読む」は SD 直読みの runtime で入ると書き分ける | 直した |
| DR4 | D-111 | `docs/ARCHITECTURE.md` 103・200・201・236・237 | IO-09・IO-10 と SD から読む流れ | IO-09・IO-10 の行と流れに「D-111 の runtime は未実装（Draft PR #172）」の注記を 1 か所ずつ足す | 直した |
| DR5 | D-117 | `docs/project-memory.md` 43・196 | アプリは段階導入の道具（一括の PLU 対象化等）を持たない | 採用済みの方針として残し、今の build には一括の PLU 対象化の button が残り、撤去は PLU の書出しの見直しの runtime と書き分ける | 直した |
| DR6 | D-117 | `docs/project-memory.md` 45・57・225・251、`docs/DB_DESIGN.md` 178・179 | 毎日の締めの SD の読取りで Z004 の全スロットを照合する、アプリは枠を消さない（clear 行を書かない） | 採用済みの方針として残し、今の build は Clear 行を書き（`PLU_CLEAR_ROW_ENABLED = true`）、占有は利用者が選ぶ Z004 で照合する、撤去と切替えは PLU の書出しの見直しの runtime と書き分ける（33 §16 の冒頭の runtime の注記を引く） | 直した |
| DR7 | D-117 | `docs/SCREEN_DESIGN.md` 137 | PLU 書出しの画面の上部の「レジの登録状況の最終照合日時（売上データ取込みで SD を読んだとき）」、照合が無ければ書出しを拒む | 同上（今の画面の照合の入口と、D-117 の runtime の後の形を書き分ける） | 直した |
| DR8 | D-111 | `docs/backlog.md` 109 | 「標準の入力元はレジの SD の直読みに変わった（D-111、PR #150）」 | PR #150 は設計で、runtime は Draft PR #172 と書き足す | 直した（S5 の 109 の行） |
| DR9 | D-117 | `docs/backlog.md` 215（取消線の項目） | 「段階導入の道具は D-117 で撤去した」 | S5 で archive へ移すとき、閉じた根拠の句を「撤去すると決めた（D-117。runtime は PLU の書出しの見直しの lane）」にする | 直した（S5 で archive へ移し、根拠の句をその形にした） |

分類: 直す 25（DR1〜DR9 の 24 行と、S4 の SC1〈SCREEN_DESIGN 3〉）、正しい 95（Plans の 14 行は全部、設計の merge と後続の runtime の lane として書いている。project-memory の 44・113・119・163・198・203・240・243、DB_DESIGN の 3・11・13・25・43・59・146・175、ARCHITECTURE の 3・184、SCREEN_DESIGN の 45・238・241・242・243・245・250・348・367 は「proposed・未実装」か「(a) は実装済み」の書き分け済み、実装済みの D-119 の単位の語、店の事実、または運用の制約〈店が CV17 の取込みを続ける〉。backlog の 54 行は runtime の lane の項目・設計の申し送り・決定の記録で、今の build の動きとして書いていない〈214 の取消線の項目は S5 で archive へ〉）。120 = 25 + 95。

### S5 backlog の整理

閉じる項目は新しい archive の file `docs/archive/2026-10-19-backlog-closed.md` へ本文を変えずに移し、移した各項目に閉じた根拠（PR・D・置き換えた項目）を 1 句足す。新しい archive は `docs/archive/` の直下にあるので、移した本文の相対 link は次の規則で直す: `archive/` で始まる path は `archive/` を除く、それ以外の docs 起点の path は `../` を前置する（`https://` と `#` だけの link は変えない）。新しい archive は移した元の節の見出し（例: `## 次に動く lane（順番固定）`）を節の見出しとして持ち、項目を元の節ごとに並べる（下の C6 の link の付け替え先）。親が済んで子が生きている項目は、子を独立の項目に上げてから親を閉じる。項目を移して空になる節は、見出しごと消す。backlog の頭の文に新しい archive への link を足し、頭の「L8-4 等の製品の未決判断は Plans の該当節に保持する」の句を、P6 で Plans から L8-4 を外した後の形（L8-2・L8-5 等）に直す。

見出しを書き換える・消す file（`docs/backlog.md` の「次に動く lane（順番固定）」と空になる節、`docs/FUNCTION_DESIGN.md` の「今後の追記予定」、`docs/DEV_SETUP_CHECKLIST.md` §14、`docs/SCREEN_DESIGN.md` の冒頭の節）への anchor は、doc check が anchor を見ないので、`rg -n -o '(FUNCTION_DESIGN|DEV_SETUP_CHECKLIST|SCREEN_DESIGN|backlog|Plans)\.md#[^)]+' docs README.md` で `docs` と root の `README.md` の全部を出し、消す・替える見出しを指す行が 0 件になるまで付け替える（起票の時点で当たるのは下の C6 の 2 行。root の `README.md` は SCREEN_DESIGN の「1. 画面一覧と使用頻度」と FUNCTION_DESIGN の「現時点の対象モジュール」を引くので、その 2 つの見出しは替えない。Plans の「製品の未決判断」の見出しへの anchor は `docs/backlog.md` の頭が引くので、P6 は Plans のその節の見出しを替えない）。`docs/archive/plans/2026-09-15-filter-label-top-runtime.md` の 414 行の link（`../../backlog.md` の「次に動く lane（順番固定）」の見出しへの anchor）は、移した先の見出し（`../2026-10-19-backlog-closed.md#次に動く-lane順番固定`）へ link だけを付け替える（archive の file の本文は変えない）。

| 分類（調査 C） | 起点の行 | 直し方 | 結果 |
|---|---|---|---|
| (1x) 閉じ印つきで残置 46 | 15〜20・47・56〜58・62・68・69・126・127・130〜133・135〜139・141・142・145・149・151・155・156・165・170〜172・190・208・215・238〜240・244・246・257・324・410 | archive へ移す。15〜20 を移すと「次に動く lane（順番固定）」の節が空になるので、節の見出しと backlog の頭の「次に動く lane は上から順の固定順」の句も直す（Plans.md の P3 と組） | 直した（46 行を archive へ移し、各項目に根拠の句。「次に動く lane（順番固定）」の見出しと 9 行目の固定順の句を消した。62 の保留・記録の子 3 つは独立の項目に上げた。409 の見出しは項目を移して空になったので消した） |
| (1) 済んだのに開いている 18 | 28・33・60・71・96・100・117・213・276・277・311・313・314・316・329・333・379・403 | 根拠を確かめて archive へ移す。71（単位の拡張）と 100（MSI）は生きている子（原価 C の項目・数量 P・(b)、製品名と upgrade code の固定・予行演習）を独立の項目に上げる。96 の子（51 UI-01b-D5 の句）は S2-14 の後に閉じる。379（L8-1）は他の画面を `rg` で確かめ、確かめられなければ残す | 直した（18 行とも archive へ。71 は P と単位の runtime の follow-up を新しい項目「単位の拡張の残りの申し送り」〈Plans の 41 行が引く名〉へ、⑤ へ送った 84・85・87・92 を (b) の子へ、C へ送った 88・89 を原価の lane の子へ上げてから閉じた。100 は製品名と upgrade code の固定・予行演習・v8 より後の migration の義務を独立の項目に上げ、前提の 104 を固定の項目の子にした。96 の子 97 は S2-14 の後に閉じた。379 は記録詳細の `MovementTable` と廃棄の画面〈local `formatQuantity` 0 件〉で確かめて閉じ、397 も閉じた） |
| (2) 前提が変わった 7 | 37・140・231・262・304・337・411 | 37・337・411 は前提が変わったと根拠を書いて archive へ。140（数量 0・金額 0 の行）も archive へ移す。根拠: D-118 の ADR が、同じ精算の中で売れて戻り個数・金額とも 0 になる打ち消し合いを見つけられない限界を受け入れ（TD-265、ADR の「打ち消し合い」の項）、32 の冒頭の D-118 の節が旧い相殺の判定を置き換えた。runtime (b) の未了の義務として戻さない。231（ECR+ の後の精算）・262（tracked の sample）・304（⑤ go-live 検証）は変わった前提を直し、残る部分だけを残す | 直した（37・140・337・411 を archive へ。37 の子の入力 1〜5 は正本〈37・decision-log・時点証拠 ADR〉が「backlog の実測と POS 系列の対応の入力 N」と引くので、独立の項目「実測とPOS系列の対応の入力 1〜5」として残した。231〈ジャーナル閲覧の代替は D-121〉・262〈今後の sample は TD-296〉・304〈PLU の再確認・MSI の docs 化は済み、移行の道具は D-117、layout B は D-111〉は前提を直して残した。411 を移して空になった見出しは消した） |
| (4) 重複 13 | 50・51・234・275・295・297・312・315・317・351・383・397・414 | 50・51（STK-1・STK-2）・275 は棚卸しの runtime (b) の項目へ寄せ、234 は日報の紙の runtime の項目へ寄せる。295 は 277 と同じ advisory（どちらも済み）、315 は 297、317 は 288、397 は 379、351 は E12 に吸収、383 は Plans の L8-5（P7）と 1 か所にする。312・414（Q40）は生きている項目として 1 つにまとめ（包括的な障害時対応の方針の残り。画面ごとの CmdError と retry、共通の Error Boundary は済み、と書く）、もう一方を消す。ARCHITECTURE の A1 がこの 1 項目を指す | 直した（50・51〈子の 52〜55〉・275 を (b) へ寄せて archive へ。53・55 は (b) の起票が引く義務・決まっていない問いが無かった〈Z004 のメタ行の割当ては 23 の冒頭、`ej_unverified` の分岐は 51 UI-01b-D22 が作らないと書く〉。234 は日報の紙の runtime へ寄せて archive へ。295・315・317・351・397 を archive へ。383 は Plans の L8-5 が指す 1 か所として「既起票・参照のみ」の句を外した。Q40 は 312 に 1 つにまとめ、414 を archive へ〈`rg -n '^- .*Q40'` は 1 行〉） |
| 生きている項目として残す | 212（D-023 の POS adapter の境界の実装側の残り） | 閉じない。残りの根拠: `DailyReportSourceKind`（Z001・Z002・Z005）が BIZ と `src/lib/bindings.ts` に出ている、`src-tauri/src/db/schema_v4.rs` の `source_adapter` の CHECK が機種の値 1 つに固定。項目の中の行数・出現数の時点の数は直さない（S6 の T4） | 残す（閉じない。行数・出現数の時点の数は S6 の T4） |
| 文の直し | 36・39・93（B1）・109・110（B3）・143・146・148・153・49 の (8)・54 の (6)（U-R1）・72・53〜55・189・191・247・428 | 93: `(8, 8)` の行の一覧は今 `(8, 8)` と `(9, 9)` が並ぶ。行番号をやめ test 関数名で引く。110: Draft PR #172 を書く。143: 下の Draft PR #155 の 1 行。146・148・153・36: 行き先「⑤」「㉘ 以降の棚卸し実装 lane」を棚卸しの runtime (b) の項目に。49 (8): 29E の今の文に合わせる。54 (6): 精算回数は main では保存していない（足すのは Draft PR #172）と直す。72: 予定は済んだ。53〜55: 残す基準は「D-118 の後も (b) の起票が引く義務か、まだ決まっていない問いを持つ物」で、それだけを (b) の項目の子に残し、それ以外（D-118 で置き換わった ③〜⑤ の段取り）は archive へ移す。189・191: 古い条件・時点の語を、確かめられた所だけ直す。247・428: 履歴境界の前の PR 番号の注記。109: 「標準の入力元は SD の直読みに変わった（PR #150）」に、PR #150 は設計で runtime は Draft PR #172 と書き足す（S4 の DR8）。39: EJ の項目の「EJ の完全性に依る判定〈相殺の判定、…〉」の「相殺の判定」は D-118 で持たなくなったので外す（S4・S2-16 と同じ根拠） | 直した（36・93・109・110・143・146〈と子の 147〉・148・153・189・191・247・428、54 の (6) は (b) の子に直して残した）。残す（親と一緒に archive へ移した記録で、archive の本文は変えない）: 72〈予定は済んだ。archive の 71 の根拠の句〉・39〈相殺の判定。archive の 37 の根拠の句〉・49 の (8)〈archive の 47 の根拠の句〉。53・55 は (b) に残す物が無く archive へ（上の (4) の行） |
| owner の決定（TD-327） | 143（PC の外の backup） | 項目に 1 行足す: Draft PR #155（D-114 の design、2026-10-07 から止めている）を残す。PR は Draft のまま。再開の条件は「owner がこの lane を選んだとき。main へ同期し、D-114 の番号が空いているかと D-118・D-123 の後の migration と backup の契約を確かめてから plan-gate を続ける」 | 直した（143 の項目に子の 1 行） |
| 足す 1 行 | 利用者に見える文言（S6 の表の最後の行） | ExportBar の Tooltip「準備中（Phase 4 で実装予定）」の時期の語が古い（日次・月次の 2 file）。製品の文言の変更なので本 lane では直さず、backlog に 1 行足す。同じ「code の comment・文言の古い句」の 1 行として、`src-tauri/src/db/sales_repo.rs:165` の doc comment「ロールバック時の逆補正用（product_code + 元のquantity）」（今の BIZ-03 は quantity を使わない。S1 の洗い直しの comment の 1 行）も足す | 直した（「workflow / test / lint / docs」の期間・動線の旧表記の項目の後に 1 項目「code の comment・文言の古い句 2 つ」） |

backlog の残り（調査 C の (3) 生きている 171 項目）は変えない。そのうち workflow の正本と Lane 2〜5 当時の候補の「未照合」37 項目は本 lane で照合しない。

### S6 本 lane で直さない分類（残す）

調査 C の表のうち、店の資料・owner でしか確かめられない行と、外部仕様の確認日は本 lane で直さず、次の段の入力として分類だけを残す（値は書かない）。

| 束 | 調査 C の番号 | 主な file | 確かめ先 |
|---|---|---|---|
| 店の資料で確かめる（時点つきの事実） | T14〜T30（17） | `docs/project-memory.md`、`docs/backlog.md` の入力 1〜5、29・23・34 の手元集計の節 | 2026-10-18 の SD の写しと在庫 Excel |
| 店の資料で確かめる（未確認の前提） | U-S1〜U-S10（10） | 29・37・pos-tables・project-memory・backlog | 同じ。U-S1 は CV17 の取込みの前後 2 時点の写しが要る |
| owner・店主・Windows 実機に聞く | T31〜T35（5）、U-O1〜U-O22（22。S4 の SC3〈UI-11c の L3〉を含む） | project-memory・backlog・SCREEN_DESIGN・UI_TECH_STACK・msi runbook ほか | owner・店主・Windows の予行演習 |
| repo で数え直せるが本 lane で直さない | T2・T4〜T9・T11〜T13、U-R4〜U-R10、T10（msi runbook の現物の照合） | project-memory・backlog・10・52・53・UI_TECH_STACK・design-system・msi runbook | 次の docs の lane。T10 は data safety の手順書で R3 に当たる |
| 外部仕様の確認日 | X1〜X19（90 日超 7、確認日なし 2、未確認・未実施 3） | DEV_SETUP_CHECKLIST・SCREEN_DESIGN・backlog・plu-export・25・52・UI_TECH_STACK | vendor の公式資料（本 lane の外） |
| 店の事実の旧表記 | SCREEN_DESIGN 217（調査 C の S6）と backlog の「期間・動線の旧表記の残り 7 か所」 | SCREEN_DESIGN・35・biz-task-specs・tracking・screen_mockups・code comment | backlog の同じ項目（code comment を含むので別の小口） |
| 利用者に見える文言 | ExportBar の Tooltip（調査 A §3.3） | `src/features/daily-sales/components/ExportBar.tsx`・`src/features/monthly-sales/components/ExportBar.tsx` | backlog の 1 行（S5） |

### S7 個人の memory から正本へ上げる 2 項

出所: Coordinator の個人の auto-memory（repo の外、Claude だけが読む）にある、tracked に置くべき内容（owner 2026-10-19「memory も棚卸しして…上手く分ける…今回ってドキュメント整備の一環だからやっていい」）。内容は発注書に写されたもので、memory の file は読んでいない。実装 run は各項を repo の中の出所（PR・原文の file・decision-log・code）で確かめてから置き、確かめられない項は置かない（S7-2 の 2026-09-24 の canvas の判断だけは下の例外）。

| # | 置き場 | 置く内容 | 確かめ方 | 結果 |
|---|---|---|---|---|
| S7-1 | `docs/UI_TECH_STACK.md` の §6 に新しい小節（§6.12「class の上書きの罠」）。置き場の理由: §6 は横断の UI 要素の実装上の決定（§6.10 Error Boundary、§6.11 離脱ガード）を持ち、§2 は採用 stack の選定、§7 はリスクと保留の記録なので、実装の注意は §6 が合う | (1) 共通 primitive の size・state の variant を呼出し側で上書きするときは同じ variant 修飾子を付ける。素の utility class（例 `sm:max-w-3xl`）は `data-[size=default]:sm:max-w-lg` のような variant 付きの class に詳細度で負け、typecheck・lint・test が全部 green のまま実 render でだけ崩れる（PR #15、2026-08-29）。`docs/design-system/02-component-catalog.md` の一覧の選択行（`data-[state=selected]` の明示固定）の例は重複させず参照する。(2) TanStack Router の `Link` component は base の `className` と `activeProps` の `className` を空白でつなぐだけで、同じ CSS property の class の衝突を解かない（twMerge しない）。base と activeProps に同じ property の class を置くと両方が DOM に残り、生成 CSS の出現順で負けた側が見えない。RTL の `toHaveClass` も `cn()` の twMerge も素通りする（PR #28 の L3 FAIL、2026-09-02）。衝突する property は片側だけに置き、「active のときに負け側の class が共存しない」負の assert を test に足す | (1) PR #15 と catalog の該当の節。(2) lockfile の版（`package.json` と `package-lock.json` で `@tanstack/react-router` 1.170.5）の `node_modules/@tanstack/react-router/dist/esm/link.js` の className の組み立てで確かめ、版を添える。起票の時点で手元にある `node_modules` は本 worktree に無く、本体の checkout の `node_modules/@tanstack/react-router/package.json` の version は 1.168.23 で、`link.js` は base・active・inactive の className を空白でつなぐ（`[className, resolvedActiveProps.className, resolvedInactiveProps.className].filter(Boolean).join(" ")`）。1.170.5 で確かめられなければ「1.168.23 での観測」と書く | 直した（§6.12。(1) は PR #15 の packet〈`archive/plans/2026-08-29-ui-polish-batch.md` の round 3 の是正〉と `alert-dialog.tsx` の class で確かめ、catalog ⑫ を参照。(2) は PR #28 の packet〈`archive/plans/2026-09-02-ui-current-location-accent.md`〉と本体の checkout の `link.js` で確かめた。1.170.5 の file は手元に無く「1.168.23 での観測」と書いた） |
| S7-2 | `docs/design-system/03-philosophy.md` に新しい節「owner の受け取り方（L3 の原文から）」。置き場の理由: 03 は「出典ごとに何を取り、何を取らないか」を持つ file で、owner の L3 の原文（`docs/design-system/reference/2026-09-04-owner-l3-feedback-raw.md`）はデザインの判断の一次の出典の 1 つ。原則は 04、判断の規則（DSR）は 01 が持つので、好みを原則や規則として足さず、出典の節として置き、各項に関係する 04 の原則・DSR を添える。README の「読む順」と sub docs の表の 03 の説明に「owner の受け取り方」を 1 句足す | 要約の項: 壊さない優先（今の導線は壊す必要を感じない、ただし慣れの自覚がある → 新案は実機で今の形と並べて見せる段取りを入れる）／共通化は道具（統一しても見やすくなるとは限らない。構造は画面ごとの意味を優先し、文言・囲み・色・token は揃える）／列見出しは一覧のもの（日次・月次のサマリや単一レコードの詳細〈操作ログの詳細を含む〉へ、一覧用の列見出しを機械的に持ち込まない。比較用の表は DSR-16 に従う）／飾りより情報（架空の field・誤解を招くラベルは採らない）／色は意味があるときだけ（添える根拠は 00 の色の役割表と DSR-08〈色を役割に沿う semantic token で当て、色だけで意味を伝えない〉。DSR-08 は「色を使う場面を絞る」より「色だけに頼らない」の規則なので、一致を言い切らず、読んで当たる方を添える）／線と塊は少なく、角は丸く（灰色の塊は採らない）／説明文を歓迎（quick action の説明、集計の説明書き、「直近 10 件」の明示）／配置の常識（金額は右端、状態 → 在庫数 → 売価、操作ボタンは右下、開始・終了は「期間」の group）／決まりは「どう見せるか」でなく「見る人がどう受け取るか・その効果」から組む（見せ方 → 受け取り方 → 効果 → 根拠）／分からないときは分からないと言い、無理に決めず実機を見てから決める／2026-09-24 の canvas での判断（キオスク型は採らず業務端末型で部門分け・スキャンを上部、新案は今のアプリの外枠の中で描く、操作と進行中は同じ緑寄りの色の家族で揃える、書体は Noto Sans JP が好み〈同梱と実機での読み分けが条件〉）／ランキング 1 位: 2026-09-24 に順位と太字を試す案を了承したが、2026-09-28 の L3 で採らなかった。今は琥珀の pill と行の地（D-094 の「試し 8 点の採否」）。2 位・3 位の見せ方は backlog の別の項目 | 各項を原文の file の該当の run・行で確かめ、確かめられない項は置かない。要約の元（Coordinator の memory の写し）より後の repo の決定（decision-log・design-system・backlog）で覆っていないかを項ごとに確かめ、覆っていれば今の決定に合わせる。起票の時点で確かめた所: ランキング 1 位は D-094 で覆った（上のとおり置き換えた）。操作と進行中の色は D-094 の token（操作 `--primary` #1d5c63、進行中 `--ongoing` #2f7f86、Home の入口 card は進行中の地）で同じ家族、書体は `docs/design-system/00-foundations.md` の書体の節で採用が決まっていない（Noto Sans JP は好みとして残す）、灰色の塊は backlog の card の面色の項目（未着手）と食い違わない。2026-09-24 の canvas の判断は、repo の中の記録（decision-log の D-091〈色の役割の組み直し〉、00 の書体の節、backlog の書体の lane）があればそこを引き、無い項（キオスク型・業務端末型・外枠の中で描く）は「repo に記録なし（owner の canvas 2026-09-24、未照合）」と書いて要旨だけを置く。claude.ai の artifact の URL は書かない。原文の長い引用・店の運用の細部は書かず要約にする | 直した（03 に節「owner の受け取り方（L3 の原文から）」の 12 項。各項を原文の節の名か D-091・D-094・00・backlog の項目で引いた。「列見出しは一覧のもの」は原文が価格履歴に列見出しを求めるので「列見出しは要る所を選ぶ」に、「色は意味があるときだけ」は原文が状態の badge と増減の数値に色を求めるので「色は意味を運ぶときに使う」に直した。キオスク型・業務端末型は「repo に記録なし〈owner の canvas 2026-09-24、未照合〉」。README の読む順と sub docs の表に 1 句） |

### 他の lane との所有

| file | 節 | 所有 |
|---|---|---|
| `docs/decision-log.md` | 既存の D の `- Status:` の行、足す `- Superseded by:` の行、D-071 の `- Superseded in part by: D-121` の行（S3） | 本 lane |
| `docs/archive/plans/2026-09-15-filter-label-top-runtime.md` | backlog「次に動く lane」への link 1 つ（付け替えだけ、S5） | 本 lane |
| `docs/decision-log.md` | 頭（Purpose の下に置く Status の規則の文）と末尾に足す D-124〜D-126 | `agent/docs-writing-rules` |
| `docs/Plans.md` | `## 次の行動` の棚卸しの文（本 packet への pointer の 1 句、本 run）と S4 の P1〜P7 | 本 lane |
| 上の表の他の file（S7 の `docs/UI_TECH_STACK.md` §6.12・`docs/design-system/03-philosophy.md`・`docs/design-system/README.md` を含む） | S1〜S5・S7 の行 | 本 lane |
| workflow の正本（`AGENTS.md`・`CLAUDE.md`・`docs/DEV_WORKFLOW.md`・`docs/ci.md`・`docs/AGENT_OPERATING_MANUAL.md`・`docs/code_review.md`・`docs/quality/review-checklist.md`・`docs/project-profile.md`・`docs/agent-guidance/**`・`docs/templates/**`） | 全体 | 並走の lane（本 lane は触らない） |

`docs/decision-log.md` で重なる 2 本は、後に merge する方が `origin/main` の単段 merge で同期する（`docs/DEV_WORKFLOW.md` Wave Operation）。本 lane の行は既存の D の中、向こうの行は頭と末尾で、同じ行を触らない。

## Non-scope

- code・migration・bindings・route・画面の文言の変更（ExportBar の Tooltip を含む）。
- workflow の正本（上の所有表）と、decision-log の Status の規則そのものの正本化（D-126）。
- file:行 の引用の一般の書き換え（lane 2、TD-326）。直す文の中の行番号は symbol・節 ID の引き方に替えてよい。
- data safety の手順書（`docs/msi-distribution-runbook.md`・`docs/migration-recovery-runbook.md`）。
- 店の資料・owner でしか確かめられない事実、外部仕様の確認日、backlog の「未照合」の項目（S6）。
- 調査 A の (2) 済・(3)・(4) と、調査 C の (3) の項目。
- decision-log の決定の本文と、S3 の表にない D の Status。
- 「proposed・未実装」の語を design docs 全体で別の語へ置き換えること（Review Focus Q1）。

## Acceptance Criteria

- AC1: `bash scripts/doc-consistency-check.sh` が exit 0。
- AC2: `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-19-docs-contract-refresh.md` が exit 0。
- AC3: `cargo run --bin generate_traceability -- --check` が exit 0（function-design を触る。REQ の参照は増減させない）と、`cargo test --test design_compliance_test` が PASS（function-design の必須節を見る検査。32・35 の節を書き換えるので回す）。
- AC-S1-1（直す形が残らない）: `rg -n --no-heading -e 'products\.stock_quantity ?(の)?更新' -e 'stock_quantity更新\+movements|stock_quantity更新とmovements' -e 'stock_quantityを(actual_countに更新|逆方向に補正)' -e 'stock_quantity -= ' -e '::update_stock_quantity\(' -e '直接 update_stock_quantity|update_stock_quantity \+ inventory_repo::insert_movement|1回の update_stock_quantity|stock計算 → update_stock_quantity' -e '逆方向に変換して在庫補正|在庫を [+-]\d+ 補正|在庫補正に product_code と quantity|在庫補正 = [+-]|その増減を戻す|戻り値を使って在庫補正|product_code でグループ化して集計' -e 'product_code別に逆補正|確定直前現在庫\+\(N-L\)|現在在庫 \+ 変動量|products更新 \+ inventory_movements追加' -g '!docs/archive/**' -g '!docs/plans/**' -g '!docs/research/**' -g '!docs/evidence/**' docs` が何も出さない（exit 1）。S1 の洗い直しの「直す」34 行のうち、S1 の型の 32 行（23:245・352 は S2-16）から見出しの語だけの 3 行（24:369、32:411・432）を除く 29 行を拾う式（round 3 の是正で、広げた語の 4 行を拾う句を足した）。
- AC-S1-2（読んで確かめる）: 35 §20.5 の手順 5 g と 32 §15.5 の手順 5・§15.8 の INV-2 が `recompute_stock` を名指しし、順が code の `legacy_complete_stocktake`・`apply_void_stock_corrections` と同じで、旧本体の記述であること（§15.0・§20.0）が残っている。`stock_after` と求め直しの値の関係が S1-1 の条件付きの形で書かれている。
- AC-S1-3（洗い直しの式の hit が全部分類済み）: S1 の洗い直しの docs の式と comment の式を同じ範囲で実行し直し、出た行が全部、表の「正しい」「対象外」の行か、直した文（`recompute_stock` を名指しする行、語を残した見出し）である。新しく出た行は分類して表に足す。`rg -n '::update_stock_quantity\(|stock_quantity -= ' src-tauri/src --glob '!*test*'` の production の hit は `biz/stock_ledger.rs` の 1 行だけ（code を変えないことの確かめ）。
- AC-S2-1: `rg -c 'future' docs/function-design/73-ui-stocktake.md` が何も出さない（exit 1）。
- AC-S2-1b: `rg -n '付いておらず|付与されておらず|型が生成されていない' docs/function-design/73-ui-stocktake.md` が何も出さない（exit 1）。直した文の根拠として、`rg -n -B1 'pub fn (start_stocktake|get_stocktake_items|update_count|complete_stocktake)\(' src-tauri/src/cmd/stocktake_cmd.rs | rg -c 'specta::specta'` が `4`、`rg -n '^\t(startStocktake|getStocktakeItems|updateCount|completeStocktake):' src/lib/bindings.ts` が 4 行を出す。
- AC-S2-2: `rg -n '今後の追記予定|実装進行に合わせて追記' docs/FUNCTION_DESIGN.md` と `rg -n '参考、未着手|着手予定|v1\.0\.0 予定|v0\.9\.0-ui-product-inventory 予定' docs/DEV_SETUP_CHECKLIST.md` が何も出さない（exit 1）。
- AC-S2-3: `rg -n '7-7b 後続 PR|Vitest 未着手|Vitest 初期化が未着手' docs/function-design/54-ui-shortcuts.md docs/function-design/55-ui-csv-import.md docs/function-design/56-ui-daily-sales.md` が何も出さない（exit 1）。
- AC-S2-4: `rg -n '（proposed・未実装、D-118）' docs/FUNCTION_DESIGN.md docs/function-design/43-cmd-settings-log.md docs/function-design/60-ui-product-import.md docs/function-design/61-ui-receiving.md docs/function-design/62-ui-manual-sale.md docs/function-design/63-ui-return-exchange.md docs/function-design/64-ui-disposal.md` が何も出さず（exit 1）、同じ 7 file の各々で `rg -c '\(a\) は実装済み'` が 1 以上（file ごとに 1 本ずつ実行する）。
- AC-S2-5: `rg -n '将来の global text-size|将来の全体文字サイズ|新しい SQL / schema は未実装' docs/UI_TECH_STACK.md docs/design-system/02-component-catalog.md docs/architecture/io-task-specs.md` と `rg -n '初回 UI-01b 実装では読取専用|影響が大きいため別 Design Phase' docs/function-design/51-ui-product-form.md` が何も出さない（exit 1）。
- AC-S2-6（読んで確かめる）: S2-8（22）と S2-15（29 の EJ）の直した文が、本文に書いた code の module・関数で確かめられる。
- AC-S2-7: S2 の表の全行の「結果」が「直した」か「残す（理由）」。
- AC-S3-1: `rg -n '^- Status: proposed' docs/decision-log.md` が何も出さない（exit 1）。
- AC-S3-2: `rg -n '^- Status: superseded' docs/decision-log.md` が D-056・D-058・D-079・D-084・D-087・D-116 の 6 行を出し、`rg -n '^- Superseded by: D-092' docs/decision-log.md` が D-056・D-058・D-079・D-084・D-087 の 5 行を出す。
- AC-S3-3: `rg -n -A3 '^## D-085' docs/decision-log.md` が `- Status: accepted` の行を出す（D-085 の節の中、`## D-085` の見出しと次の `## D-` の間に Status の行があることを読んで確かめる）。
- AC-S3-4: `rg -n '^- (Status|Superseded( in part)? by):.*(implementation slice|reflected in PR|future mechanical checks|remains a separately gated|remains R4|implementation completed|pending Final Review|are complete|and implemented by|実装は未着手|実装は R3 packet で進行|merge 後の follow-up|runtime は後続|、proposed）|反映は同 packet の実装の PR で行|実装は同 packet の PR で行う|runtime は同 packet の実装の PR)' docs/decision-log.md` が何も出さない（exit 1）。
- AC-S3-5: `rg -n '^proposed' docs/adr/2026-10-16-stocktake-count-placement.md` と `rg -n '（proposed）|D-118 の design 出力であり未実装' docs/adr/README.md` が何も出さない（exit 1）。
- AC-S3-6（読んで確かめる）: 各 Status の行に、出所（design lane・Plan Packet・owner の決定の TD・日付）が残り、実装の状態の語が無い。
- AC-S3-7: `git diff origin/main...HEAD -- docs/decision-log.md | rg '^[-+]' | rg -v '^(\+\+\+|---)' | rg -v '^[-+]- (Status|Superseded( in part)? by):'` が何も出さない（後ろの `rg` の exit 1。決定の本文の行を変えていない）。
- AC-S4-1: `rg -n -o '最終更新\*?\*?[:：] ?20[0-9-]{8}' docs/SCREEN_DESIGN.md docs/plu-export-and-real-csv-verification.md docs/TOOLING_SKILL_COMMANDS.md docs/DEV_SETUP_CHECKLIST.md docs/FUNCTION_DESIGN.md docs/DOCKER_REPAIR_LOG.md docs/UI_TECH_STACK.md docs/function-design/50-ui-product-list.md` が何も出さない（exit 1）。
- AC-S4-2: `rg -n '\bvL\b|次の lane は未定|次に動くlane|表示小修正 batch 2 に同乗|④ C5' docs/Plans.md` と `rg -n 'Post-UI-08 follow-up|18 original tables|planned daily report tables|Fabric may be managed in .cm.|2026-09-30 に試す予定' docs/project-memory.md` が何も出さない（exit 1）。
- AC-S4-3: `rg -c 'stock_counts' docs/inventory_system_erd.html` と `rg -c 'stock_counts' docs/diagrams/current-system.md` がそれぞれ 1 以上、`rg -n 'v1〜v7' docs/inventory_system_erd.html` が何も出さない（exit 1）。
- AC-S4-4: `rg -n '^\| .*stock_counts|idx_inventory_movements_ledger|idx_stock_counts_product_id' docs/DB_DESIGN.md` が表の一覧の行と 2 つの index の行を出す。
- AC-S4-5: `rg -n '^## 計数の置き場と在庫の求め直しの契約（proposed・未実装、D-118）' docs/SCREEN_DESIGN.md` が何も出さない（exit 1）。
- AC-S4-6（読んで確かめる）: A4・SC3・SC4・SC5 の履歴境界の前の PR 番号・tag に、`docs/PROJECT_HANDOFF.md` の履歴境界の節を引く注記がある。
- AC-S4-7: S4 の表の全行の「結果」が「直した」か「残す（理由）」。
- AC-S4-8（読んで確かめる）: S4 の型の洗い直しの DR1〜DR9 の文が「採用済みの方針（D の番号）／今の runtime／残る実装」に書き分けられ、今の runtime の記述が `docs/function-design/33-biz-plu-export-service.md` §16 の冒頭の「runtime の状態」の注記と backlog の runtime の項目（SD 直読みの runtime、PLU の書出しの見直しの runtime）に合う。洗い直しの式を実行し直して、新しく出た行は分類して表に足す。
- AC-S2-8: `rg -n '相殺の判定（ADR D4）の入力に残す|相殺の判定の入力に残すため' docs/function-design/23-io-z004-parser.md` が何も出さない（exit 1）。
- AC-S5-1: `rg -n '^- \[x\]|^- ~~' docs/backlog.md` が何も出さない（exit 1）。
- AC-S5-2: `docs/archive/2026-10-19-backlog-closed.md` があり、`rg -c '^- ' docs/archive/2026-10-19-backlog-closed.md` が S5 で閉じた項目の数以上で、`rg -n '2026-10-19-backlog-closed.md' docs/backlog.md` が 1 行以上を出す。移した項目の相対リンクの path は全部在る（`rg -o '\]\([^)#]+' docs/archive/2026-10-19-backlog-closed.md` で出した path を 1 つずつ確かめる）。
- AC-S5-3: `rg -n 'PR #155|pull/155' docs/backlog.md` と `rg -n 'PR #172|pull/172' docs/backlog.md` がそれぞれ 1 行以上を出す。
- AC-S5-4: `rg -n '㉘ 以降の棚卸し実装 lane|⑤ で棚卸しの .total_cost.|⑤ で BIZ から返す|㉘ ④ に吸収|2026-09-30 に試す予定|\(8, 8\)' docs/backlog.md` が何も出さない（exit 1）。
- AC-S5-5: `rg -n '^#### 次に動く lane（順番固定）' docs/backlog.md` と `rg -n 'backlog\.md#次に動く' docs README.md` が何も出さない（exit 1）。`rg -n -o '(FUNCTION_DESIGN|DEV_SETUP_CHECKLIST|SCREEN_DESIGN|backlog|Plans)\.md#[^)]+' docs README.md` が出す anchor が、どれも今ある見出しを指す（S5 の前文の手順。doc check は anchor を見ない）。
- AC-S5-6: S5 の表の全行の「結果」が「直した」か「残す（理由）」で、閉じた・寄せた項目ごとに根拠の 1 句がある。
- AC-S5-7: `rg -n '^- .*Q40' docs/backlog.md` がちょうど 1 行を出し、`rg -n '^- D-023 POS adapter boundary' docs/backlog.md` が 1 行を出す（生きている項目として残す）。
- AC-S7-1: `rg -c 'data-\[size=default\]:sm:max-w-lg' docs/UI_TECH_STACK.md` が `1`、`rg -c 'activeProps' docs/UI_TECH_STACK.md` が 1 以上（同じ項の中で語を 2 回使ってよい。2 項がそれぞれ 1 回ずつ置かれ、別の節に重複が無いことは読んで確かめる）。(2) の項に確かめた版（1.170.5、または「1.168.23 での観測」）が書かれている。
- AC-S7-2: `rg -c 'owner-l3-feedback-raw' docs/design-system/03-philosophy.md` が 1 以上。
- AC-S7-3（読んで確かめる）: 03 の新しい節の各項が原文の file か上に挙げた repo の中の記録で確かめられ（2026-09-24 の canvas の判断の要旨を除く）、design-system の既存の規則（04 の原則・01 の DSR・00 の色の役割表と書体の節）と、要約の元より後の決定（decision-log・design-system・backlog。例: D-094 のランキング 1 位）と食い違わない。repo に記録の無い canvas の要旨には「repo に記録なし（owner の canvas 2026-09-24、未照合）」と書いてある。claude.ai の artifact の URL・店の運用の細部・原文の長い引用が無い。S7 の表の「結果」が「直した」か「残す（理由）」。

baseline（docs の本体は起点 `18db621e` から変えていない。起票の時点〈2026-10-19〉に測り、AC-S1-3・AC-S2-1b・AC-S3-4・AC-S5-5・AC-S5-7 は Plan Review round 1 の是正の時点〈2026-10-19、本 branch の tip `6c815f7a`〉に直した command で測り直した。各 command を逐語で実行した。長い行は先頭だけ写した。AC1〜AC3 は PR の Validation の欄）:

```text
AC-S1-1  exit=0。29 行（round 3 の是正で句を足し、tip `578a863f` で測った。round 2 の式では 25 行）
AC-S1-3  docs の式 286 行、comment の式 38 行（round 3 の是正で語の幅を広げ、tip `578a863f` で測った。round 2 の式では 80 行 / 18 行）/ production の hit: biz/stock_ledger.rs:190 の 1 行
AC-S2-8  exit=0。23:245・23:352 の 2 行（tip `578a863f`）
AC-S4-8  S4 の型の洗い直しの式: 120 行（tip `578a863f`。分類は S4 の型の洗い直しの表）
AC-S2-1  18（exit=0）
AC-S2-1b 前: exit=0。73:102・73:279 の 2 行 / 根拠: rg -c は 4、bindings は 237・243・282・289 の 4 行
AC-S2-2  FUNCTION_DESIGN 18・100・177・179 の 4 行 / DEV_SETUP_CHECKLIST 23・511・513・515・528 の 5 行
AC-S2-3  exit=0。56:518・56:596・55:512・54:207 の 4 行
AC-S2-4  exit=0。64:3・63:3・62:3・60:3・61:3・43:3・FUNCTION_DESIGN:3 の 7 行
AC-S2-5  前: io-task-specs:7・UI_TECH_STACK:785・02-component-catalog:1041 の 3 行 / 後: 51:28 の 1 行
AC-S3-1  exit=0。969・983・996・1009・1022・1034・1046 の 7 行（D-116〜D-122）
AC-S3-2  前・後とも exit=1（0 行）
AC-S3-3  708:## D-085 … / 709:（空行）/ 710:- Decision: … / 711:- Why: …（Status の行が無い。-A3 で測り直した）
AC-S3-4  exit=0。251・287・304・312・330・338・346・362・370・532・584・843・854・865・875・885・895・905・914・936・950・969・983・996・1009・1022・1034・1046 の 28 行（Q4 の裁定で式を足した後、tip `aace90e8` で測った）
AC-S3-5  前: 5:proposed — 2026-10-16 … / 後: 7:- [棚卸しの数と売上の前後を … の 1 行
AC-S4-1  8 行（8 file に 1 行ずつ）
AC-S4-2  前: Plans 18・30・34・57・58 の 5 行 / 後: project-memory 46・50・67・250 の 4 行
AC-S4-3  rg -c は 2 file とも出力なし（exit=1）/ v1〜v7: exit=0。26: の 1 行
AC-S4-4  exit=1（0 行）
AC-S4-5  exit=0。3: の 1 行
AC-S5-1  48 行（rg … | wc -l）
AC-S5-3  前（#155）: exit=1 / 後（#172）: exit=1
AC-S5-4  exit=0。36・72・93・140・146・148・153 の 7 行
AC-S5-5  前: 13:#### 次に動く lane（順番固定） / 中: Plans.md:30 と archive/plans/2026-09-15-filter-label-top-runtime.md:414 の 2 行（README.md は 0 行） / 後: 18 行（docs 16・README.md 2。round 2 の是正で範囲に README.md を足し、tip `79d51fb1` で測った。今ある見出しを指さないのは、次に動く lane の 2 行だけ）
AC-S5-7  前: 312・414 の 2 行 / 後: 212 の 1 行
AC-S7-1  2 本とも出力なし（exit=1）（tip `aace90e8` で測った）
AC-S7-2  出力なし（exit=1）（同上）
```

## Design Readiness

- 引用する設計正本（節まで）: [棚卸しの数の置き場と在庫の求め直しの ADR](../../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D2・D7）、[31](../../function-design/31-biz-inventory-service.md) の「計数の置き場と在庫の求め直しの契約」（`recompute_stock`）、[32](../../function-design/32-biz-csv-import-service.md) §15.0・§15.5・§15.8、[35](../../function-design/35-biz-stocktake-service.md) §20.0・§20.5、[22](../../function-design/22-mnt-migration.md) §19（v9）、[停止 ADR](../../adr/2026-09-23-legacy-stocktake-z004-write-stop.md)、[decision-log](../../decision-log.md) の D-092 の Compatibility と D-116〜D-122、[DB 設計](../../DB_DESIGN.md) と db-design の「計数の置き場と在庫の求め直しの契約」、[引き継ぎ案内](../../PROJECT_HANDOFF.md) の履歴境界の節。S7 は [UI 技術 stack](../../UI_TECH_STACK.md) §6、design-system の [出典](../../design-system/03-philosophy.md)・[判断の規則](../../design-system/01-decision-rules.md)（DSR-08・DSR-16）・[土台](../../design-system/00-foundations.md)（色の役割表・書体の節）、decision-log の D-091・D-094、owner の L3 の原文 `docs/design-system/reference/2026-09-04-owner-l3-feedback-raw.md`。
- 必要な設計成果物: Design artifact selection の当たる行は無い（振舞い・command・DB・画面・format を変えない）。状態は existing sufficient。直すのは文と実装・決定の食い違いだけ。
- plan にしかない durable な判断の昇格先: 本 packet が決める判断は 3 つ。(1) D-056・D-058・D-079・D-084・D-087 の Status を superseded にする（S3-3）、(2) ADR の Status を decision-log と同じ規則で直す（S3-11）、(3) 閉じた backlog の項目の置き場（`docs/archive/2026-10-19-backlog-closed.md`）。(1)(2) は TD-327 の規則の適用で、規則の正本は D-126（並走）。(3) は backlog の頭の文に置く。S7 は個人の memory にあった内容の昇格で、置き場は `docs/UI_TECH_STACK.md` §6.12（実装の注意）と `docs/design-system/03-philosophy.md`（owner の受け取り方の出典）。2026-09-24 の canvas の判断のうち repo に記録の無い項は「repo に記録なし（owner の canvas 2026-09-24、未照合）」と明記して要旨だけを置き、決まりの根拠にはしない。
- 前提・制約と延期した gap: S6 の束は次の段へ延期する。延期しても安全な理由: 本 lane は S6 の文を変えず、確かめていない事実を確かめたように書き換えない。
- 絶対保証の自己点検: S1 の「`recompute_stock` だけが写しを書く」は production の code の現物（上の `rg`）に基づく。test・診断の helper の直呼び（`inventory_repo.rs` の test）は設計正本の主張の外。
- 判定: ready。

## Registration / Generation Obligations

| 変更対象 | 義務 | 本 lane |
|---|---|---|
| source / workflow doc 新設 | 親文書の目次・索引を更新 | `docs/archive/2026-10-19-backlog-closed.md`（新規の archive）を `docs/backlog.md` の頭の文から link する（S5） |
| REQ / coverage の追加・変更・削除 | `cargo run --bin generate_traceability` で 90 を再生成 | REQ の参照は増減させない。AC3 の `--check` で確かめ、増減が出たら再生成して Scope に足す |
| function-design doc の節 | `design_compliance_test` の必須節 | 節を足さず消さない（32・35 の書き換え）。AC3 で確かめる |

ほかの行（Tauri command、route、operator 画面）は該当なし。

## Impact Review Lenses

not applicable: 起点は現地調査・実機確認・外部 tool の振舞いでなく、repo の中の docs と code の照合（read-only の調査 2 本）。店の資料・実機で確かめる事実は S6 に分けて本 lane で扱わない。

## Boundary / Wire Contract

変えない。JSON・CSV・config・manifest・DTO・bindings・DB に触れない（ER 図・DB_DESIGN は今の schema を写すだけ）。

## Test Plan

Test Design Matrix は置かない（R2 で任意。各段の AC が `rg` の件数と分類表の行で直した所を確かめる）。

- targeted tests: AC3（`generate_traceability -- --check`・`design_compliance_test`）。code を変えないので Rust・frontend の test は足さない。
- negative tests: AC-S1-1・AC-S1-3・AC-S2-1〜AC-S2-5（AC-S2-1b を含む）・AC-S3-1・AC-S3-4・AC-S3-5・AC-S4-1・AC-S4-2・AC-S4-5・AC-S5-1・AC-S5-4（旧い表現が残らないこと）、AC-S3-7（決定の本文を変えていないこと）。
- compatibility checks: AC1・AC2（Markdown の link の先の file の実在。anchor は見ないので AC-S5-5 の `rg` で確かめる）。
- data safety checks: `git diff --name-only origin/main...HEAD` が docs の file だけで、店の運用の細部・実値・発言の原文・金額・商品名を書いていない（S2-15・M4 の観測は件数や値を写さず、出所の file を引く）。
- main wiring/integration checks: AC-S5-2（新しい archive が backlog から辿れる）、AC-S4-3（ER 図と図面が DB_DESIGN から辿れる今の schema を持つ）。

起票の run の検証（AC1・AC2 と `bash scripts/check-workflow-git.sh`）の結果は PR の Validation の欄に書く。

## Review Focus

- S1: 洗い直しの表の分類（直す 34・正しい 86・対象外 166、comment 38）に取り違えが無いか、式の外に同じ型が残っていないか。直した 35 §20.5・32 §15.5 の順が code の旧本体と同じか。旧本体の記述であること（停止中）が読めるか。D-118 の節（(b)）と §15.5・§20.5（(a) の旧本体）の取り違えが無いか。
- S2: (a)/(b) の書き分けで、(b) の契約を実装済みと読める文が出ていないか。S2-15（EJ の未観測の形式）は code で確かめた形式だけを外しているか。
- S3: Status の行（と D-071 の Superseded in part の 1 行）だけが変わり、決定の本文が変わっていないか（AC-S3-7）。Q2・Q4。
- S4・S5: 閉じた項目の根拠が確かめられるか。生きている子を落としていないか。Plans から消す段落にしか無い owner の決定を落としていないか。
- Q1（Plan Review で答える）: D-117〜D-122 を accepted にした後も、design docs の「proposed・未実装」の語（決定が採られる前の意味を含む）を残してよいか。既定: 残す（design docs のこの語は実装の状態の印で、decision-log の Status とは別。語を替えるなら D-126 の規則で全体を一度に替える）。
- Q2（Plan Review で答える）: D-056・D-058・D-079・D-084・D-087 は Superseded の行を足すだけでなく Status も `superseded（by D-092）` にするか。既定: する（TD-327 の 4 値で、全面的に置き換わった決定の状態は superseded。D-116 の扱いと同じ）。
- Q3: round 1 の是正で決着（Review Response 参照）。D-034・D-038・D-045・D-046・D-111・D-113 は S3-12〜S3-17 として本 lane に入れ、AC-S3-4 の期待は 0 行。
- Q4: Coordinator の裁定で決着（Review Response 参照）。D-100〜D-107 の反映の予定の句は S3-19 として外し、AC-S3-4 の式に足した。

## Implementation Results

Do not transcribe exact-HEAD SHA or test counts here (D-035/D-038 Evidence Ownership). Record a qualitative summary and the PR link only.

- 結果: 実装の run（2026-10-19〜10-20、Writer Opus 5.5 subagent。途中で使用量の上限で切れ、同じ worktree で再開した）が S1〜S5・S7 を段ごとに直し、各表の「結果」列を埋めた（残すのは S4 の H2 と S5 の D-023 の項目だけ）。S1 は旧本体（停止中）の確定・取消と共通在庫処理の記述を、movement を足して `recompute_stock` で写しを求め直す今の code の順に直した。S2 は (a) で入った所を実装済みの契約へ、(b) を proposed・未実装のまま書き分けた。S3 は decision-log と ADR の Status を決定の状態と出所だけにし（base 同期の後に D-126 の形へ揃えた）、置き換わった D に Superseded の行を足した。S4 は dashboard と図面を今の schema と runtime に合わせ、採用済みで未実装の決定（D-111・D-117）を「採用済みの方針／今の runtime／残る実装」に書き分けた。S5 は閉じた backlog の項目を `docs/archive/2026-10-19-backlog-closed.md` へ根拠の句を付けて移し、生きている子を独立の項目か寄せ先の子に上げた。S7 は class の上書きの罠（`docs/UI_TECH_STACK.md` §6.12）と owner の受け取り方（`docs/design-system/03-philosophy.md`）を正本へ置いた。
- 検証: AC1〜AC3（doc check・traceability・design_compliance_test）と段ごとの AC は実装の後と是正の後の期待どおり。読んで確かめる AC（AC-S1-2・AC-S2-6・AC-S3-6・AC-S4-6・AC-S5-6・AC-S7-3）は実読で確かめた（AC-S4-6 の SCREEN_DESIGN の 4 か所の注記の link は closeout で足した。下の Review Response の Closeout）。
- 計画からの外れ: S5 の「文の直し」の 72・39・49 の (8) は、親（71・37・47）を archive へ移すときに子として一緒に移したので、archive の本文を変えず、各親の根拠の句に今の形を書いた。37 は全体を移さず、正本が引く入力 1〜5 を backlog に残した（前提が変わったのは lane の状態の部分）。
- review・CI・merge（closeout、2026-10-20）: Final Review は broad の 3 本（Claude 側 fresh Opus 5.5 の全体 = approve〈P3 4〉と S5 の深掘り = reject〈P2 1〉、Codex〈6.1 Sol〉= reject〈P2 1〉）。Coordinator が全件を受理して是正し、base 同期の後の closure 3 本（Codex Sol・Claude 側 Opus の closure・S1/S4 の深掘り）で P1/P2 の残りは 0、P3 は closeout の PR で直した。hosted CI の Merge gate は Ready の後の run で pass、専用 record の review は pass、manual・R4 は無い。[PR #180](https://github.com/higekiri-w/inventory-system-desktop/pull/180) は helper 経由で 2026-10-20 に merge（PR #179 の merge の後の base 同期を経た）。
- 後続: 棚卸しの lane 2「file:行 の引用の書き換えと検査」と S6 の束（`docs/Plans.md` の次の行動）。D-126 の形でない Status の行 15 行・名前で引く参照の残り・task-specs の runtime の注記・archive の link は `docs/backlog.md` の次の docs の lane の候補。

## Review Response

Fill after review.

### Plan Review round 1（2026-10-19、対象 `6c815f7a`）

- Codex: GPT-6 Astra（既定の Sol が満員だったため Astra で走った）。判定 reject、P2 5・P3 1。
- Claude 側: fresh Opus 5.5。判定 reject、P2 1・P3 6。
- Coordinator の裁定: 全件を受理。起草役が次のとおり packet に反映した（docs の本体は直していない）。

| finding | 要旨 | 反映した所 |
|---|---|---|
| Codex C1（P2） | backlog の D-023 の項目と Q40 は生きている。Q40 は 1 項目へまとめ、S4-A1 から引く | S5 の (4) の行（312・414 を 1 つにまとめて残す）、S5 の「生きている項目として残す」の行（212）、S4 の A1、AC-S5-7 |
| Codex C2（P2） | 起点の行 140（数量・金額とも 0 の行）は D-118 の ADR が限界を受け入れ、32 の冒頭が旧い相殺の判定を置き換えたので archive へ | S5 の (2) の行（140 を archive へ、runtime (b) の義務として戻さない） |
| Codex C3（P2） | 61〜64 の書き分けで、計数と入出庫の窓の計算は (a) で実装済み | S2-7（`judge` → `from_count` → `derive` と test 名、(b) は保存の公開の経路・精算の窓・画面への反映） |
| Codex C4（P2） | 73 の「既存 4 command に specta の属性が無く bindings に型が無い」は古い | S2-1 の追記、AC-S2-1b |
| Codex C5（P2）・Claude O1（P2） | D-034・D-038・D-045・D-046・D-111・D-113 も S3 へ。AC-S3-4 を英語の表現まで広げ、期待を 0 行に | S3-12〜S3-17、AC-S3-4、Review Focus の Q3 の決着 |
| Codex C6（P3） | archive の packet の backlog「次に動く lane」への link も付け替える | S5 の前文（付け替え先の見出し）、所有表、AC-S5-5 |
| Claude O2（P3） | D-071 の `Superseded in part by: D-121（…、proposed）` の「、proposed」を外す。AC-S3-7 の許可式を広げる | S3-18、S3 の前文、所有表、AC-S3-7 |
| Claude O3（P3） | `stock_after` と求め直しの値が同じなのは、写しが記録と一致しているときだけ | S1-1・S1-3、AC-S1-2 |
| Claude O4（P3） | biz-task-specs の BIZ-06 の確定の手順 2 と BIZ-03 のロールバックの句も旧い | S1-6、S1 の前文、AC-S1-3 |
| Claude O5（P3） | doc check は anchor を見ない | Test Plan の compatibility checks、AC-S5-5、S5 の前文の anchor の手順 |
| Claude O6（P3） | 新しい archive の相対 link の規則を直接書く | S5 の前文 |
| Claude O7（P3） | 選択肢が 2 つのままの行に既定を 1 つ | S2-2（§14 は完了の 1 行、本文は残す）、S1-2（理由 1 は残し、理由 2 は条件付き）、S5 の (2)（140 は archive）、S5 の文の直し（53〜55 の残す基準） |

是正の中で見つけた同じ類で、裁定に無いもの: D-100〜D-107 の Status の「反映は同 packet の実装の PR で行う」の類。外さない既定で Review Focus の Q4 に置いた。

### round 2 の前の追加（2026-10-19、対象 `aace90e8`）

- Q4 の裁定（Coordinator）: D-100〜D-107 の反映の予定の句も S3 に入れる。TD-327 の規則では反映の予定は実装の状態の一種で、実装の後に古くなる。決定の内容と出所は残し、反映の予定の句だけを外す。Goal の「各 D」はそのまま。反映した所: S3-19、S3 の「入れない」の段落から D-100〜D-107 を外した、AC-S3-4 の式と baseline（28 行）、Review Focus の Q4 の決着。
- S7 の追加: owner 2026-10-19 の指示（memory も棚卸しして、今回のドキュメント整備の一環として分ける）を受け、Coordinator の個人の auto-memory にあった tracked に置くべき 2 項を、本 lane の S7 として正本へ上げる。内容は発注書に写されたもので、起草役は memory の file を読んでいない。反映した所: 前文、Goal の最小完了条件の 4 項目、Scope の S7 の節、所有表、AC-S7-1〜AC-S7-3 と baseline。
- 起草役の判断:
  - S7-1 の置き場は §6 の新しい小節（§6.12）にした。
  - S7-1 (2) の TanStack Router の振舞いは、手元の 1.168.23 の `link.js` で確かめた形（空白でつなぐだけで衝突を解かない）に言い直した。発注の写しの「空白連結で merge されない」は、「つながれない」とも読めるため。
  - S7-2 の置き場は 03（出典の file）の新しい節にした。
  - 「色は意味があるときだけ」は DSR-08 と言い切らず、00 の色の役割表と DSR-08 から読んで当たる方を添える形にした。DSR-08 の規則は「色だけで意味を伝えない」で、色を使う場面を絞る規則ではないため。

### Plan Review round 2（2026-10-19、対象 `79d51fb1`）

- Codex: GPT-6.1 Sol。判定 reject、P2 2・P3 1。
- Claude 側: fresh Opus 5.5。判定 reject、P2 2・P3 6（ほかに補足 2）。
- Coordinator の裁定: 全件を受理。round 3 が天井なので、名指しの file だけでなく、S1・S2 の型を意味の側で洗い直すよう指示した。

| finding | 要旨 | 反映した所 |
|---|---|---|
| Codex C1（P2） | 「レポート系は表にしない」は原文の範囲を超える | S7-2 の列見出しの項（サマリと単一レコードの詳細に限る、比較用の表は DSR-16） |
| Codex C2（P2） | ランキング 1 位は 2026-09-28 の L3 で不採用（D-094）。他の項も後の決定で覆っていないか | S7-2 のランキングの項の置き換え、項ごとの確かめの指示と起票の時点で確かめた所、AC-S7-3 |
| Codex C3（P3） | Owner Effort Budget を decision point ごとに数え直す | Owner Effort Budget（消費 4、上限を 9 に改定し理由を書いた） |
| Claude O1（P2） | 73 §73.4 の tree が現物と違う | S2-1 の直し方と「周りの説明の現物」の列 |
| Claude O2（P2） | 24 の 65・370〜375 が旧い在庫補正の式 | S1-8 |
| Claude O3（P3） | biz-task-specs の頭の (a) の一覧と、BIZ-03・BIZ-06 の停止中の注記 | S1-6 |
| Claude O4（P3） | v1.0.0 は完了と書かない（D-070） | S2-2、S4 の SC4 |
| Claude O5（P3） | AC-S3-3 を -A3 か節の中を読む形に | AC-S3-3 と baseline |
| Claude O6（P3） | AC-S7-1 の activeProps は 1 以上 | AC-S7-1 |
| Claude O7（P3） | Design Readiness と昇格先に S7。canvas の要旨は「repo に記録なし（未照合）」 | Design Readiness の 2 行、S7-2、AC-S7-3 |
| Claude O8（P3） | 空になる節の見出し、backlog の頭の L8-4 の句、anchor の検索に root の README.md | S5 の前文の 2 段落、AC-S5-5 と baseline |
| Claude 補足 1 | 35 §20.5 手順 3a の force_fill の値は code の `.max(0)` | S1-10 |
| Claude 補足 2 | S4 で定義の無い記号「P8」 | S4 の前文（記号の定義、P8 は P1 にまとめた）、SCREEN_DESIGN の行の記号を SC1〜SC5 に |

同じ型の洗い直しの結果:

- S1 の型: docs の式は 80 行 / 27 file、src の comment の式は 18 行。分類は、直す 28（S1-6〜S1-11 を足した）、正しい 37、対象外 15、comment 18 は全部「正しい」か「対象外」。AC-S1-1 を意味の側の式（25 行）に、AC-S1-3 を全数の分類の確かめに替えた。
- S2 の型: S2 の 15 行の全部で、周りの名前を現物と照らして列に書いた。現物と違ったのは次の 5 つで、どれもその行の直し方に入れた。
  - S2-1: tree の component の 7 file が無い、29 行の「runtime 実装を含まない」、`find_active_stocktake` の呼出し元。
  - S2-2: v1.0.0。
  - S2-8: 上限の検査の担い手。
  - S2-9: 識別の列が (b)。
  - S2-11: `isEditableTarget` は非公開の関数。

### Plan Review round 3（天井、2026-10-19、対象 `578a863f`）

- Codex: GPT-6.1 Sol（発注 440）。受理分は #1（P2: project-memory の D-111・D-117 の記述が今の build と違う。採用済みで未実装の決定と今の build の取り違えの型）。
- Claude 側: fresh Opus 5.5。受理分は #1（P2: pos-tables の Rollback の「逆補正」が S1 の式の語の幅から漏れた）と #2〜#8（P3）。
- 天井の disposition（Coordinator、`docs/DEV_WORKFLOW.md` Review Rules の round 天井）: 次の round を開かず、残りを同型の一括是正で反映する。是正の後に追加の review なしで plan-approved へ進めるかは owner の決定（repo 外の回答台帳 TD-330）。

| finding | 要旨 | 反映した所 |
|---|---|---|
| Claude #1（P2） | S1 の型の語の幅（逆補正 ほか）が狭い | S1 の洗い直しの式と分類（docs 286 行・comment 38 行）、S1-12（pos-tables:425）、S1-13（35:540）、S1-11 の 176 行、S1-2 の 368 行、S2-16（23:245・352）、AC-S1-1（29 行）・AC-S1-3・AC-S2-8 |
| Claude #5（P3、#1 に含めた） | 10-common-rules の上限の見積もりの「在庫 + 補正の和」 | S1 の洗い直しで「正しい（上限の見積もり）」と分類 |
| Codex #1（P2） | 採用済みで未実装の決定と今の build の取り違え | S4 の型の洗い直し（dashboard の 7 file、120 行。直す 25 = DR1〜DR9 の 24 行 + SC1、正しい 95）、AC-S4-8 |
| Claude #2（P3） | S2-1 の `find_active_stocktake` の呼出し元の句は消すだけ | S2-1 |
| Claude #3（P3） | 32:413 は見出しの語を残し、括弧書きを直す | S1 の洗い直しの round 2 の表の行 |
| Claude #4（P3） | 35 §20.5 3a の注記「差異なし扱い」 | S1-10 |
| Claude #6（P3） | 履歴境界の注記を D-035・D-039 にも | S3-12 |
| Claude #7（P3） | ARCHITECTURE の CMD-07・BIZ-03 の行に停止 ADR の注記 | S4 の A3 |
| Claude #8（P3） | S2-9 は production の schema の file に限って数える | S2-9（`rg -n 'machine_no' src-tauri/src/db -g 'schema_v*.rs'` → 出力なし、exit=1） |

同型の洗い直しの結果:

- S1 の型（語の幅を広げた）: docs は 286 行で、直す 34・正しい 86・対象外 166。comment は 38 行で、37 行は正しいか対象外、1 行（`sales_repo.rs:165`）は古い。本 lane は code を変えないので、S5 で backlog に 1 行足す。
- S4 の型（採用済みで未実装の決定）: D-111・D-117〜D-122 について dashboard の 7 file を洗い、120 行を分類した（直す 25、正しい 95）。取り違えは D-111（SD 直読み）と D-117（PLU の書出しの見直し）に集まり、D-118〜D-122 は書き分け済みか実装済みだった。backlog の 39・109 は S5 の文の直しに足した。

### Final Review broad（2026-10-20、対象 `a9ba2f09`）

- Claude 側: fresh Opus 5.5 の 2 本（broad: approve、P3 4。S5 の深掘り: reject、P2 1・P3 4）。Codex: PR #180 の review（reject、P2 1・P3 2）。
- Coordinator の裁定: 全件を受理（P2 の C1・S1 は Coordinator が現物で確かめた）。是正は Writer（Opus 5.5 subagent、発注 450）。Scope・AC は変えていない。

| finding | 要旨 | 直した所 |
|---|---|---|
| C1（P2） | backlog の P が EJ の小数の数量を D-121 の控えの閲覧の範囲に置く（D-121 Decision (6) と食い違う） | backlog の P を「控えの閲覧（(A)）は小数の数量を拾わない。受理は 29 の単位の拡張の表し方で EJ の取込みの lane が U5 の後に足す」に直した。同型を live な docs で洗い、ほかに直す行は無かった（式と出力は表の下） |
| S1（P2） | STK-2 の子の申し送りの (4)（交換を差額だけの戻しで打つ運用、TD-225）が、D-118 で置き換わったという根拠の句のまま archive へ移った | (4) を backlog の (b) の子へ上げ、63 UI-03-D8 の既定との関係を 1 句添えた（直し方は決めない）。archive の根拠の句を (1)〜(6) ごとの行き先に書き直した |
| S1'（同型） | archive へ移した 102 行の根拠の句の見直し | 102 行を分類した。上げるのは (4) だけで、(3)〈小数の数量〉は P が持つと根拠の句に書いた。ほかは根拠の句と合う（店の前提と owner の決定は project-memory・decision-log・10 に在る） |
| O1 / C2（P3） | 29 の IO-08.9 の表が観測済みの小数の数量を「未観測の形式」に入れる | 行を「未観測の形式」と「観測済みで受理しない形（小数の数量・点数、`AUTO` / `BT`）」の 2 行に分けた |
| O2 / S2（P3） | archive へ移した項目を live な docs が旧い項目名で引く | 移した項目名を live な docs・README・src に `rg -F` で当てて全部分類した。Scope の file（23・29E・project-memory）は寄せ先の名へ直し、Scope に無い file（29D・33・37・ADR・runbook）と decision-log は直さず、寄せ先の項目の頭に「旧名」の句を置いた。decision-log の D-105 が引く「EJ の入力 (5)」は archive の根拠の句で辿れるようにした |
| O3（P3） | 73 の「本書の契約は実装済み」が冒頭の (b) の節まで含む | §73.1 以降（UI-10-D1〜D13）に絞り、冒頭の D-118 の節は (b) と書いた |
| O4（P3） | backlog の slice 2 が開いたまま | D-039（PR #166、履歴境界の前）と今の script（PK4・PK5・drift test）で実装済みと確かめ、根拠の句をつけて archive へ移した |
| C3（P3） | IO-04・BIZ-04 に採用済みの設計と今の runtime の区別が無い | IO-04・BIZ-04 に 33 §16 を引く runtime の状態の注記を置き、BIZ-01 の PLU の 3 行を同じ形に直した。同型の IO-09・IO-10・BIZ-08 の SD 直読み（D-111）にも注記を足した。cmd-task-specs・ui-task-specs の同型は Scope の外なので直していない |
| S3（P3） | archive の根拠の句が起点の backlog の行番号で他の項目を指す | 10 か所を項目名・子の名に替えた |
| S4（P3） | 本 PR の句が旧 repo の PR 番号を注記なしで使う | archive の PR #84 を「D-072（旧 repo の PR #84）」、backlog の PR #60 を「UI_TECH_STACK §6.10、旧 repo の PR #60」にした。本 PR の新しい句の他の番号は今の repo の PR と合う |
| S5（P3） | filter-label の packet の 415 行の link が live の backlog の節を指す | 移した先の archive の見出しへ付け替えた |

C1 の同型の洗い直し（tip `73fd18e4` の後の作業木で実行）: `rg -n '(小数|数量)' docs README.md -g '!docs/archive/**' -g '!docs/plans/**' -g '!docs/research/**' -g '!docs/evidence/**' | rg '(EJ|Z004)' | rg '(D-121|控え|閲覧|一括取込み)'` → 9 行（`backlog.md:46`〈直した P の行〉、`Plans.md:18`、`37:693`・`37:694`、`decision-log.md` の D-121 の本文 4 行、`mockup-d-home-sales-admin.html:271`）。直した P の行の他は、控えの閲覧に小数の復元が要らないと書く正しい行か、語が並ぶだけの対象外の行。

AC の再実行（是正の後、逐語）: AC1・AC2・AC3 は exit 0。AC-S1-1 は 0 行、AC-S1-3 は docs 256 行・comment 38 行で `a9ba2f09` と同じ行の集合。AC-S4-8 は 121 → 122 行（足したのは (b) の子の (4) の 1 行。書き換えた行は旧名の句と P の行で、採用済みの方針を今の build の動きとして書く行は無い）。AC-S5-2 は 78 → 79（slice 2 を移した）。AC-S5-5 の anchor の列挙は 16 → 15（filter-label の 415 行が archive を指すようになり、式の `backlog\.md` に当たらない）で、全部今ある見出しを指す。AC-S5-7 の D-023 の行は 139 → 140（子を 1 行足した）。ほかの AC は `a9ba2f09` の値と同じ。

追い（2026-10-20）: base 同期は Coordinator の単一 merge（`73fd18e4`、origin/main `8cff232a`〈PR #179、D-124〜D-127〉、衝突なし）。その後、S3 の前文に従い、decision-log の頭の「Status の書き方」（D-126）を正本に、S3 で触った D の Status の行のうち 27 行（D-034・D-035・D-038〜D-041・D-043・D-045・D-046・D-085・D-087・D-100〜D-107・D-111・D-113・D-116〜D-121）を `accepted（出所）` / `superseded（by D-xxx。出所）` の形に揃えた（実装・review の状態の句を外し、出所を括弧へ。D-038・D-040・D-041 の中身の句は D-039・D-041 の本文が持つので外した）。S3 で触っていない D の Status の行で同じ形でないものは 15 行（`rg -n '^- Status:' docs/decision-log.md | rg -v '^[0-9]+:- Status: (accepted|withdrawn|proposed)(（.*）)?$|^[0-9]+:- Status: superseded（by D-[0-9]+(。.*)?）$' | wc -l` → `15`。D-042・D-075・D-086・D-088・D-090〜D-094・D-096〜D-099・D-109・D-123。多くは括弧の後の「。」の余り、D-042 は英文、D-075・D-091 ほかは括弧の後に中身の文が続く）で、直していない（Follow-up）。PK6 の WARN は上の 2 行を直して 0。

### Closeout（2026-10-20）

closure 3 本の P3 は PR の Follow-up で closeout へ寄せ、本 lane の closeout の PR で直した。直した所: `docs/function-design/22-mnt-migration.md` の廃番の扱いの参照先（archive へ移した「PLUスロット永続割当」の項目を指していた句を、今の正本の 30 §4.5 と 33 §16.6〈D-117 (1)〉へ）、29 IO-08.9 の小数の理由句（IO-08.5 の「単位の拡張の表し方」〈D-113〉を EJ の取込みの lane が実装するまで受理しない形へ）、decision-log の D-035・D-038・D-040・D-041・D-045・D-085 の Status の括弧（owner の決定、repo 外の回答台帳 TD-337。括弧には出所だけを残し、中身の句を外した。外した関係のうち本文に無いもの〈D-040 と D-041 の Phase B の実行の契約の関係、D-045 が D-041・D-042 の証跡の要求の裁定に効き D-040 の clone の役割の分離を変えないこと〉は足し直さず、backlog の次の docs の lane の候補に 1 句で記した）、35 §20.8 の INV-3 と §20.5 の force_fill の設計判断（写しは `recompute_stock` で求め直す、負の在庫は 0 を入れて差異が出る）、10 の上限から導ける範囲の表の差分の経路の 2 行（求め直しの値の 1 行へ）、SCREEN_DESIGN の DR7 の末尾（今の画面はその場で Z004 を選ぶ導線）と履歴境界の注記 4 か所（引き継ぎ案内の履歴境界の節への link。AC-S4-6）、project-memory の cm の現在形（今の build は原価の lane の作り直しの起点の migration まで `cm` を持つ）。各型の旧い句を docs と code の comment で洗い直し、同型の live な残りは直した所だけだった。backlog へ回したのは PR の Follow-up の残り（D-126 の形でない Status の行 15 行、名前で引く参照の残り、task-specs の runtime の注記、archive の link 3 file）で、次の docs の lane の候補として 1 項目にまとめた。
