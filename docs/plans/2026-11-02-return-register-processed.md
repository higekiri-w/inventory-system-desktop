# Plan Packet: 返品・交換で、レジで読んで戻したかを必ず問い、レジの資料で戻らない商品の在庫をこの保存で動かす

wave に属さない単独の lane（導入の前の関門 10。owner の決定、repo 外の回答台帳 TD-570〈2026-11-02、candidate から confirmed へ〉。関門の正本は `docs/Plans.md` の導入の前の関門で、10 の行は PR #226〈main `c66ef81d`、`docs/Plans.md` だけを変えた〉で main に入った）。出発点は `docs/backlog.md` の項目「返品・交換の `register_processed` の既定と差額だけの戻しの交換」（TD-225 の要旨、b1 の起票で b1 の外の小さな lane と決めた TD-377〜TD-379 の (iii)）。本 commit は Plan Packet の起票と plan-first の設計正本の修正だけで、実装は計画の固定の後の別の run。branch は `agent/return-register-processed`（main `cb58b1d4` から）。新しい決定は D-148（main は D-147 まで、open の PR が足す D は D-143・D-130。`gh pr diff` で確かめた、2026-11-02）。

## Workflow State

- Risk: R3
- Plan Commit: 92149f917995e767e2528678dd83d35e57be7cf5
- Amendments: none

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の「Tauri command の DTO・状態を持つ画面・operator の業務の流れ」に当たる: `ReturnCreateResult`（command `create_return` の出力）に field を足して bindings を作り直し、返品・交換の画面の form の状態（既定の無い必須の選択）と保存の流れを変え、BIZ が在庫を書く条件を変える。R4 にしない理由: 削除・backup・restore・migration・店の実データに触れない（DB の列と既定は変えない）。

## Goal

Goal Invariant:

### 最小完了条件

- 返品・交換の画面で、レジでの戻し方（レジでバーコードを読んで戻した／読んでいない）を選ぶまで保存できない。既定の選択は無い。
- 「読んでいない」を選んで保存すると、全部の明細で在庫が動く（差額だけの戻しの交換・部門キーで打った返品が黙って動かないことが無い）。
- 「読んで戻した」を選んで保存しても、Z004 の取込みが在庫を動かさない商品（`pos_stock_sync=false`・共有 JAN・JAN 無し・JAN が 13 桁の数字でない〈JAN-8 ほか〉）の明細は、この保存で在庫が動く。
- 保存結果で、明細の商品ごとに「この保存で反映」か「取込みで反映」かが文言で読める（冪等の再送の結果も同じ）。

### 失敗定義

- どちらも選ばずに保存でき、どちらかの意味で記録される。
- 「読んで戻した」で保存した記録のうち、Z004 が在庫を動かさない商品の明細で在庫が動かない。
- Z004 が在庫を動かす商品（13 桁の数字の JAN が 1 商品だけに当たり在庫連動）の明細を「読んで戻した」で保存したのに、この保存で在庫が動く（Z004 と二重に戻る）。
- 返品・交換の判定と Z004 の確定の判定が別々の式になり、片方だけが変わりうる。
- 在庫が動いたのに在庫系の画面の cache が古いまま残る、または保存結果の行の文言が実際の在庫の動きと食い違う。

### 非目的

- 明細ごとにレジでの戻し方を問う（owner の判断事項 Q4 は今は記録ごと。TD-575）。
- 記録の編集・取消（63 §63.8 の非対象のまま）。旧い意味で保存した開発・試験 DB の記録を移す。
- DB の列名・既定（`return_records.register_processed`、`DEFAULT 1`）を変える migration。
- Z004 の取込みの在庫連動の条件そのもの（D4・共有 JAN・PLU をレジへ書く時期）を変える。
- 保存の前に明細ごとの反映の見込みを出す（BIZ が正本。結果で名指す。Q3）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 現状の調べ（2026-11-02、main `cb58b1d4`）

- backlog の記述は今の code で当たる: 画面の既定は `src/features/return-exchange/ReturnExchangePage.tsx` の `createEmptyForm` の `registerProcessed: true`。BIZ は `src-tauri/src/biz/inventory_service/returns.rs` の `create_return` で `if !req.register_processed { apply_stock_change(...) }` だけが在庫を書く（true の記録は明細を書くが movement を作らない）。
- Z004 の確定が在庫を動かす条件は `src-tauri/src/biz/csv_import_service/commit.rs` の `recheck_matches` の `stock_linked.push(!shared && head.pos_stock_sync)`（JAN で引いた商品が 1 つだけ、かつ在庫連動）。Z004 は商品ごとの純の数量なので、PLU を読んだ戻しはその精算の売りと相殺されて在庫に戻る。JAN の無い商品は Z004 の行に当たらない。
- backlog が挙げない 3 つ目の組: `pos_stock_sync=false`・共有 JAN の商品は、レジで読んで戻しても Z004 は `sale_records` だけを書き、在庫は動かない（`commit.rs` の `if *linked && row.quantity != 0`）。
- Z004 の照合の JAN の形（review round 1 の P2）: Z004 のスキャニングコードは `src-tauri/src/io/z004_parser.rs` の `normalize_jan`（23 §13.5）で ASCII 数字 13 桁に正規化され（14 桁 + 末尾英字は末尾を除く。全桁ゼロは `Ok(None)` で照合しない。それ以外は行の error）、`product_repo::find_by_jan_code` はその値と `products.jan_code` を完全一致で比べる。`products.jan_code` は JAN-8 を持てる（`src-tauri/src/biz/jan_code.rs` の `validate`、master-tables の `jan_code`）うえ、既存の行は非 JAN・非 13 桁の値もありうる（master-tables の「jan_code の形式 validation を DB に置かない理由」）。13 桁の数字でない JAN の商品は Z004 の行に当たらない。`recheck_matches` に渡る `MatchedRow.normalized_jan` は parse の段で `normalize_jan` の `Ok(Some)` を通った値だけ。
- `plu_target=0` の商品は「部門売り + 手動販売出庫」で扱う（`docs/db-design/master-tables.md` の plu_target の理由）。ただし `plu_target` はアプリが PLU を書き出す対象かの設定で、レジが読めるかの証明ではない（D-117 (7)）。Z004 の確定は `plu_target` を見ない。判定に入れない理由は D-148 の Guarantee range。
- 今の店は PLU をレジに書いておらず、全部の商品を部門キーで売る（`docs/project-memory.md` の 157 行、PLU-24）。今の既定 true では、ほぼ全部の返品で在庫が黙って動かない。PLU をレジへ書く時期は Plans の candidate（TD-462）。
- 在庫の書込みの経路（[10](../function-design/10-common-rules.md) の「入力の上限と安全な整数の範囲」の在庫の行: 入庫・返品・手動販売・廃棄・商品別売上 CSV の取込みの 5 経路が `apply_stock_change` を通る）は変えない。返品はそのうちの 1 経路の呼ぶ条件だけを変える。

## Scope

本 commit で変えた正本（plan-first）:

- `docs/decision-log.md`: D-148（accepted、TD-570・TD-575。末尾）と、D-052 の本文の後の `- Superseded in part by: D-148（C5 の条件）`。
- `docs/function-design/63-ui-return-exchange.md`: UI-03-D23（未実装）の新設、D8・D17 の行に置き換えの注記、§63.9 の D23、§63.10 の行。
- `docs/function-design/31-biz-inventory-service.md`: §12.4 の「設計判断 — register_processed分岐」の後に置き換えの段（D-148、未実装）。
- `docs/backlog.md`: 項目「返品・交換の `register_processed` の既定と差額だけの戻しの交換」の下に起票の行（子の行にしたのは項目の本文〈TD-225 の要旨〉を残して起票の事実を足すため。PR #226 は `docs/Plans.md` だけを変え、backlog とは重ならなかった）。

実装（計画の固定の後）:

- BIZ（Rust）:
  - `src-tauri/src/biz/csv_import_service/mod.rs`（`csv_import_service` の中なら file は実装で決めてよいが 1 か所）: 31 §12.4 の置き換えの段の形 `pub(crate) fn z004_moves_stock(jan_code: &str, candidates: &[Product]) -> bool`（`jan_code` が ASCII 数字ちょうど 13 桁で全桁ゼロでない、かつ `candidates.len() == 1 && candidates[0].pos_stock_sync`。前後の空白は削らない）。`commit.rs` の `recheck_matches` の `!shared && head.pos_stock_sync` を `z004_moves_stock(&row.normalized_jan, &candidates)` に置き換える（振舞いは変えない: `normalized_jan` は `normalize_jan` の `Ok(Some)` の値で形の条件に必ず当たり、候補 0 は手前で `registration_changed` で拒む。D-147 の 4: 呼ぶ所が 1 か所を通る）。
  - `src-tauri/src/biz/inventory_service/returns.rs`: `ReturnCreateResult` に `stock_applied_product_codes: Vec<String>`（昇順・重複なし）。`create_return` の明細の loop で、`req.register_processed` のときは TX の中で商品を引き（`product_repo::find_by_product_code` の `jan_code`）、`jan_code` が `Some(jan)` なら `z004_moves_stock(jan, &product_repo::find_by_jan_code(tx, jan)?)` が true のときだけ在庫を動かさない（`None` は動かす）。それ以外は今の false の枝と同じ `apply_stock_change`。動かした商品コードを列に入れる。冪等の再送の 2 か所（TX の前の `find_return_by_idempotency_key` の一致と、`DuplicateKey` の後の一致）は、記録の movement（`reference_type = 'return_record'`・`reference_id = record_id`。`return_repo::get_return_record_detail` の movement の読取りと同じ）の商品コードから列を作る。JAN の形の条件は関数の中だけに置き、`returns.rs` で正規化（trim・桁の補い）をしない（Z004 の照合は `products.jan_code` と完全一致で比べるので、保存された値をそのまま見る）。
  - `src-tauri/src/cmd/return_cmd.rs` は素通しのまま（変えない）。
- 生成物: `src/lib/bindings.ts`（`cd src-tauri && cargo run --bin generate_bindings`。差分は `ReturnCreateResult` の 1 field の見込み）、`docs/function-design/90-traceability.md`（`cargo run --bin generate_traceability`。手で直さない）。
- UI（TS）:
  - `src/features/return-exchange/types.ts`: `registerProcessed: boolean | null`、form の error に `registerProcessed`。
  - `src/features/return-exchange/lib/return-exchange-request.ts`: `buildReturnExchangeRequest` で `null` を field error `レジでの戻し方を選んでください` にして request を作らない（UI-03-D12 と同じ段）。signature は `null` を含めたまま。
  - `src/features/return-exchange/ReturnExchangePage.tsx`: `createEmptyForm` の `registerProcessed: null`、欄の見出し・選択肢・Badge・説明文・記録の値の表示を UI-03-D23 の文字どおりに、PageHeader の subtitle を D23 の意味に合わせる。dirty の判定（`values.registerProcessed !== initialValues.registerProcessed`）は null を初期値として今のまま働く。保存結果の商品の行ごとに `この保存で反映`／`取込みで反映`。invalidation を `invalidationContract.returnExchange(result.stock_applied_product_codes.length > 0)` にする。最近の一覧の列の値を D23 の表示に。
  - `src/features/inventory-records/ReturnRecordDetailPage.tsx`: `formatRegisterProcessed` を D23 の表示に、movement が 0 件のときの説明文を「レジでバーコードを読んで戻した記録で、どの商品も売上データ（Z004）の取込みで在庫に戻ります」に。
  - `src/lib/invalidation-contract.ts` の `returnExchange` の引数の名前と意味（`stockApplied`）、`src/test/invalidation-oracle.ts` の同じ entry。key の集合は変えない。
- 正本の同期（実装の PR で D-148 に合わせる。D-148 の Compatibility はこの列挙を指す。行の番号は 2026-11-02 の `rg -n 'register_processed|レジ戻し|レジ未処理|レジ処理済' docs src` の値で、実装の時に引き直す）:
  - 63: §63.3（form state の型）・§63.5（ヘッダの既定・Badge・結果・一覧）・§63.6（validation の項）、D8・D17・D23 と §63.9 の「未実装」を外す。
  - 31 §12.4: 手順 6 と置き換えの段を 1 つにする。手順 0 の「レジ処理済みの返品・交換」の語。
  - `docs/function-design/44-cmd-inventory.md`: create_return の関数要求（219 行）と結果、get_return_record の関数要求と処理（556・583 行）。
  - `docs/function-design/21-io-inventory-repo.md`: 返品の詳細の取得の文（192 行）。
  - `docs/function-design/65-inventory-record-traceability.md`: 返品・交換の履歴の検索の項目「レジ戻し済み」（65 行）、movement の行（117 行）、取消の行（218 行）。
  - `docs/function-design/10-common-rules.md`: 明細の数量の上限の行（166 行）の「在庫を動かさない明細（レジ処理済みの返品…）」の括弧。
  - `docs/db-design/transaction-tables.md`: 冒頭の movement の要約（9 行）、`return_records` の説明（81 行）・`register_processed` の行（90 行）・理由（108 行）・例（114〜126 行）。
  - `docs/db-design/tracking-system-tables.md`: movement の種類 `return` の「レジ未処理分のみ」（89 行）と「レジ戻し処理済みの返品」の段（102 行）。
  - `docs/architecture/biz-task-specs.md`: BIZ-02 の分岐（173〜174・204 行）。
  - `docs/architecture/ui-task-specs.md`: UI-03 の行の「レジ戻し済み？」（205 行）。
  - `docs/ARCHITECTURE.md`: UI-03 の行の「レジ戻し済みフラグ」（135 行）と BIZ-02 の行の「返品のregister_processedフラグ分岐」（179 行）。
  - `docs/FUNCTION_DESIGN.md`: UI-03 の要約の「レジ戻し済み分岐」（59・160 行）。
  - `docs/diagrams/current-system.md`: 返品の分岐（180〜182 行）。
  - `docs/diagrams/cross-feature-verification.md`: 「レジ未処理返品は加算、レジ処理済みの帳面記録は在庫不変」（29 行）と表の行（162 行）。
  - `docs/SCREEN_DESIGN.md`: UI-03 の項（314・321・323・329・440 行）。
  - `docs/UI_TECH_STACK.md`: 7 行目の「返品交換（レジ未処理）／（レジ処理済み）」。
  - `docs/design-system/02-component-catalog.md`: 899 行の「レジ未処理」（`formatRegisterProcessed` の表示を除外の例に挙げる所。表示の語が変わる）。
  - 同期しない（旧い契約を書いていない）: `docs/architecture/cmd-task-specs.md` の 41 行と 21 の 161・188・200 行（field の名前だけ）、`docs/inventory_system_erd.html`（列の定義だけ）、23 の 320 行と `docs/db-design/pos-tables.md` の 461 行（レジの「戻」の Z004 の値の話）、`docs/adr/2026-10-16-stocktake-count-placement.md` の 29 行（ADR の本文。「レジ未処理」の返品交換は在庫を動かす返品の例示で、D-148 の後も在庫を動かす経路の 1 つのまま。直すかは実装の PR で ADR の規則を見て決め、PR に書く）、`docs/archive/` と `docs/research/`・`docs/design-system/reference/`・`docs/plans/2026-10-27-docs-small-batch-3.md`（その時点の記録）。
- 直す既存 test（理由: D-148 で契約が変わった所だけ。弱めない）:
  - `returns.rs` の `test_create_return_req202_register_processed_no_stock_change`: fixture の `create_test_product` は JAN 無しなので、新しい規則では在庫が動く。13 桁の JAN を持つ在庫連動の商品（その JAN の商品は 1 つだけ）に変えて「動かない」を保ち、JAN 無しの case は新しい test（下の B2）に分ける。
  - `returns.rs` の `test_create_return_req202_quantity_limit_applies_to_register_processed`（724 行）: fixture `LIM-RP` は JAN 無しで、最後に `stock_of(&conn, "LIM-RP") == 7`（在庫は変わらない）を見るので、新しい規則では 9_999_999 が足されて red になる。商品を 13 桁の JAN を持つ在庫連動の商品（1 商品だけ）に替え、「在庫を動かさない明細も同じ上限の検査を通る」（X1）を保つ。上限を超える側の拒否と境界の値の保存の assert は変えない。
  - `returns.rs` の `..._register_processed_rejects_noncanonical_date`: 業務日の検査で拒む test で在庫の結果を見ていなければそのまま（実装で確かめる）。
  - `src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs` の `RegisterProcessedReturn`: model は「在庫を動かさない」。`CODES[0]` が 13 桁の JAN を持つ在庫連動の商品なら変えない。違えば model を `z004_moves_stock` に従わせる（実装で確かめ、PR に書く）。
  - `src/features/return-exchange/ReturnExchangePage.test.tsx`: 既定 true を前提にする所（66〜67 行の説明文の定数、133〜138 行の初期の選択と `CSV取込みで反映`、153 行の `レジ未処理` の click、287・291 行、809 行の `レジ未処理` の label、819〜834 行の `レジ戻し済み`／`レジ未処理` の radio と説明文）を D23 の文言と既定なしに直す。既定のまま（radio を触らずに）保存する test は全部、保存の前にどちらかの選択を足す（足さないと新しい validation で `createReturn` が呼ばれず red になる。どの test が当たるかは実装の時に `createReturn` の呼出しを見る test を全部見て決め、PR に数を書く）。143 行の invalidation の test は列の空・非空の条件に書き直す（U5）。
  - `src/lib/invalidation-contract.meta.test.ts` の 42・67 行: 今は `returnExchange(false)` だけを oracle と比べる。引数の意味が `registerProcessed` から `stockApplied` に変わり false の集合が逆になるので、`returnExchange(true)` と `returnExchange(false)` の両方を実装と oracle で比べる（片方だけでは引数の取り違えを捕まえない）。
  - `return-exchange-request.test.ts`、`src/lib/request-helpers.test.ts`（`registerProcessed: false` の literal は型の上で通る）、`src/features/inventory-records/OtherRecordDetailPages.test.tsx`（詳細の表示の文言）。
  - 新しい表示文言と既存の test の matcher の衝突は実装の時に `rg -F` で確かめ、PR に書く。

型・JSON・関数の形を変える所:

| 対象 | 作る所 | 読む所 | test の fixture / 呼び方 |
|---|---|---|---|
| `ReturnCreateResult.stock_applied_product_codes` | `returns.rs` の `create_return`（新規・再送 2 か所） | `ReturnExchangePage.tsx`（結果の行・invalidation） | `returns.rs` の test、`ReturnExchangePage.test.tsx` の `createReturn` の resolve 値 |
| `ReturnExchangeFormValues.registerProcessed: boolean \| null` | `ReturnExchangePage.tsx` の `createEmptyForm`・radio | `return-exchange-request.ts`（signature・build）、dirty の判定 | `return-exchange-request.test.ts`、`request-helpers.test.ts`、`ReturnExchangePage.test.tsx` |
| `z004_moves_stock` | `csv_import_service` | `commit.rs` の `recheck_matches`、`returns.rs` | 両方の test |
| `invalidationContract.returnExchange(stockApplied)` | `invalidation-contract.ts` | `ReturnExchangePage.tsx` | `invalidation-oracle.ts`、`invalidation-contract.meta.test.ts`、`ReturnExchangePage.test.tsx` |

他の lane との重なり（2026-11-02、`gh pr diff N --name-only` の目視）: PR #216（在庫の振替）は `apply_stock_change` と movement の種類を触るが `returns.rs`・`commit.rs` の今の行は触らない見込み（実装の前に Coordinator が取り直す）。PR #226 は merge 済み（main `c66ef81d`、`docs/Plans.md` だけを変えた）で、この branch は Plans を触らないので重ならない（main との同期は Coordinator が後で行う）。PR #194 は重ならない。

## owner の判断事項

答え（2026-11-02、repo 外の回答台帳 TD-575）: Q1〜Q3 は推奨。Q4 は今は (a) 記録ごとに 1 問で、PLU をレジへ書く時期を決めるときに（TD-462）、交換の片方だけを読む打ち方があるかを店に確かめる（あれば明細ごとの問いを別の lane で足す。D-148 の Revisit）。設計（D-148・UI-03-D23）は推奨のとおりで、答えによる変更は無い。下は問いと選択肢の記録。

- **Q1 既定**: (a) 既定を置かず必ず選ばせる【推奨】／(b) 既定を false（この保存で反映）にする／(c) 既定 true のまま知らせを足す。理由: どちらの既定も片方の向きで黙って誤る。今は (b) が正しいが、PLU をレジに書いた後は読んで戻した返品を二重に戻す。選ぶ手間は 1 回の click。変わる所: (b) なら UI-03-D23 の validation と `null` の型が要らなくなる（BIZ の明細ごとの判定は残す）。(c) なら backlog の 2 組が直らず、関門 10 の目的を満たさない。
- **Q2 問い方**: (a) 「レジでバーコードを読んで戻したか」【推奨】／(b) 今の「レジ戻し済み／レジ未処理」のまま。理由: operator が覚えているのはレジでの操作で、「戻し済み」は差額だけの戻しでも「済み」と読めてしまう。変わる所: UI-03-D23 の文言と L3 の 1。
- **Q3 レジの資料で戻らない商品の明細**: (a) BIZ が自動でこの保存で動かし、保存結果で行ごとに名指す【推奨】／(b) 保存を止めて選び直させる／(c) 知らせだけで動かさない。理由: その商品はどちらを選んでも Z004 では戻らないので、止めても直し方が無い。(c) は在庫が黙って違ったまま。変わる所: (b)(c) なら BIZ の判定が要らず、代わりに BIZ の error か warning が要る。
- **Q4 問う粒度**: (a) 記録ごとに 1 問【推奨】／(b) 明細ごと。理由: 返品は 1 つの操作で、交換の多くは両方を同じ打ち方で打つ（推測。店の事実は未確認）。(b) は交換で片方だけをレジで読んだ組を表せるが、毎行の操作が増える。店の事実で決まる: 交換をレジでどう打つか（差額だけ／戻しと売りを両方読む／片方だけ読むことがあるか）。片方だけがあるなら (b) を別の lane で足す。変わる所: (b) なら wire の `register_processed` が明細へ移り、R3 の範囲が広がる。

## Acceptance Criteria

- AC1（BIZ の判定）: `register_processed=false` の記録は全明細で movement を作る。`true` の記録は、13 桁の数字（全桁ゼロでない）の JAN が 1 商品だけに当たり在庫連動の商品の明細では movement を作らず、`pos_stock_sync=false`・共有 JAN・JAN 無し・JAN が 13 桁の数字でない商品の明細では movement を作る。下の B1〜B5・B8 が PASS。
- AC2（1 か所の判定）: `recheck_matches` と `create_return` が同じ `z004_moves_stock` を呼ぶ（`rg -n "z004_moves_stock" src-tauri/src` で定義 1 と呼び出し 2 以上）。JAN の形の条件が関数の中にある（B7b）。既存の Z004 の commit の test（`src-tauri/src/biz/csv_import_service/tests/`）が全部 PASS。
- AC3（結果）: `stock_applied_product_codes` が新規と冪等の再送で同じ内容（B6）。
- AC4（画面）: 既定の選択が無く、選ばずに保存すると `レジでの戻し方を選んでください` が欄の近くに出て `createReturn` を呼ばない（U1・U2）。選んだ値が request の `register_processed` になる（U3）。
- AC5（結果の表示と cache）: 保存結果の行ごとの文言が `stock_applied_product_codes` に従い（U4）、在庫系 consumer の invalidation が列の空・非空に従って `d052InvalidationOracle.returnExchange` と一致する（U5）。
- AC6: `bash scripts/doc-consistency-check.sh` と `--target plan docs/plans/2026-11-02-return-register-processed.md` が ERROR なし、`bash scripts/check-workflow-git.sh` が通る。Rust・frontend・bindings・traceability の gate は `docs/DEV_WORKFLOW.md` Verification Gates のとおり。

## Design Readiness

- 引用する設計正本（節まで）: 63 UI-03-D8・D12・D15・D17・D23（新）、31 §12.4、32 の冒頭（確定の TX の取り直しと共有 JAN の項）、10 の在庫の経路の行、`docs/db-design/master-tables.md` の `pos_stock_sync`・`plu_target` の理由、D-052 C5、D-118 の受け入れた穴。
- 必要な設計成果物: updated in this PR（D-148、63 UI-03-D23、31 §12.4 の置き換えの段）。44・21・transaction-tables・biz-task-specs・65・SCREEN_DESIGN・UI_TECH_STACK の文の同期は実装の PR（Scope の「正本の同期」）。
- plan にしかない durable な判断の昇格先: D-148 と UI-03-D23。packet にだけある判断は無い（Q1〜Q4 の答え〈TD-575〉は D-148 の Status・owner の決定・Revisit に反映した。`plu_target` を判定に入れない判断は D-148 の Guarantee range と 31 §12.4）。
- 前提・制約と延期した design gap: JAN の形の条件は Z004 の照合（`normalize_jan` の `Ok(Some)` の値と `find_by_jan_code` の完全一致）に合わせた（現状の調べ）。明細ごとの問い（Q4 の (b)）は PLU をレジへ書く時期の決定まで延期（TD-575）。保存の前の見込みの表示は非目的。
- 絶対保証の例外と抜け道: 「読んで戻した」の在庫連動の明細が Z004 で必ず戻る、とは言わない（保存と Z004 の確定の間の設定の変更、取込み忘れ、operator の選び違い〈レジで読めない `plu_target=0` の商品に「読んだ」を選ぶ場合を含む〉。D-148 の Guarantee range）。「Z004 が戻さない商品の明細は必ずこの保存で動く」は、判定が Z004 の確定と同じ関数を通ることで保つ（AC2）。
- 判定: ready（Q1〜Q4 は TD-575 で答えが出て、設計は推奨のとおり。`plu_target` の扱いは D-148 の Guarantee range で決めた）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| D-148 (1)・UI-03-D23（問いと既定なし） | 63 UI-03-D23、D-148 | types.ts・return-exchange-request.ts・ReturnExchangePage.tsx | U1・U2・U3 | L3-1・L3-2 |
| D-148 (2)・31 §12.4 の置き換え（明細ごとの判定） | 31 §12.4、D-148 | returns.rs | B1〜B5・B8 | L3-3 |
| D-148 (2)（1 か所の判定） | 32 の冒頭、D-148 | csv_import_service の `z004_moves_stock`、commit.rs | B7・B7b、既存の Z004 commit の test | 非対象（画面に出ない） |
| D-148 (3)（結果の列と再送） | 31 §12.4、63 UI-03-D23・D15 | returns.rs・ReturnExchangePage.tsx・bindings | B6・U4 | L3-3 |
| D-148 (4)・UI-03-D17（invalidation の条件） | 63 UI-03-D17・D23、D-052 C5 | invalidation-contract.ts・invalidation-oracle.ts・ReturnExchangePage.tsx | U5、`invalidation-contract.meta.test.ts` | 非対象 |
| UI-03-D23（記録の値の表示） | 63 UI-03-D23 | ReturnExchangePage.tsx（一覧）・ReturnRecordDetailPage.tsx | U6 | L3-4 |
| UI-03-D12・D13（validation と冪等キー） | 63 UI-03-D12・D13 | 変えない（新しい field error が同じ段を通る） | 既存の test が PASS | 非対象 |
| D-118 の在庫の窓（返品の業務日） | 31 冒頭、ADR D3 | 変えない（`apply_stock_change` の呼び方は今の false の枝と同じ） | 既存の test | 非対象 |

隣の契約の sweep（除外の理由）: UI-03-D5（画像の retry）・D14（保存中の disabled）・D22（returnTo）は form の状態の型が変わるだけで振舞いを変えない（既存の test が PASS すること）。手動販売（62）・入庫・廃棄の経路は触らない。

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| D-148 (2) | 在庫連動で JAN が 1 商品の明細を「読んで戻した」で二重に戻す | unit（Rust） | B1 `test_create_return_req202_register_processed_linked_jan_no_stock_change`（既存の `..._register_processed_no_stock_change` を直す） | 判定を外して全明細を動かす、または `z004_moves_stock` が true を返さない |
| D-148 (2) | JAN 無しの商品の明細が「読んで戻した」で動かない | unit | B2 `test_create_return_req202_register_processed_without_jan_moves_stock` | JAN 無しを Z004 が戻す側に入れる |
| D-148 (2) | `pos_stock_sync=false` の商品の明細が動かない | unit | B3 `test_create_return_req202_register_processed_not_stock_linked_moves_stock` | `pos_stock_sync` を見ない |
| D-148 (2) | 共有 JAN の商品の明細が動かない | unit | B4 `test_create_return_req202_register_processed_shared_jan_moves_stock` | 共有の判定（候補が 2 以上）を落とす |
| D-148 (2) | 交換で戻りと渡しが混ざる記録の、明細ごとの判定と符号 | unit | B5 `test_create_return_req202_register_processed_exchange_mixed_lines` | 記録単位で一括に判定する、`out` の符号を落とす |
| D-148 (3) | 再送で列が空・違う（cache が在庫を古いまま残す） | unit | B6 `test_create_return_req202_replay_returns_stock_applied_codes`（TX 前の一致の経路）・`test_create_return_req202_duplicate_key_replay_returns_stock_applied_codes`（`DuplicateKey` の後の経路。別の接続が書込みの lock を持ち、busy handler の中で同じ要求を先に保存する） | 再送の経路で列を作らない |
| D-148 (2)（1 か所） | Z004 の確定の連動の判定が変わる | unit | B7 `test_z004_moves_stock_req401_single_linked_only`（13 桁の JAN で候補 0・1 連動・1 非連動・2 の 4 例）＋既存の commit の test | 関数が `recheck_matches` の旧い式と違う値を返す |
| D-148 (2)（JAN の形） | Z004 の行に当たらない JAN の商品を Z004 に任せる | unit | B7b `test_z004_moves_stock_req401_rejects_non_z004_jan_forms`（候補を 1 連動に固定し、`jan_code` が 8 桁の数字・12 桁・14 桁・13 桁に英字が混ざる・13 桁の全桁ゼロ・前後に空白の付く 13 桁・空の 7 例で false〈前後の空白は前と後の 2 例に分けて計 8 例〉、`4901234567887` で true。全例で「形の条件が受ける ⇔ `normalize_jan(s, 1) == Ok(Some(s.to_string()))`」も assert する〈`normalize_jan` を `pub(crate)` にする最小の昇格。closure 745 の P3-3〉） | 形の条件を外す、桁を 8 または 13 にする、trim してから比べる、全桁ゼロを通す、形の条件と Z004 の正規化の値の形が食い違う |
| D-148 (2)（JAN の形） | 8 桁の JAN の在庫連動の商品を「読んで戻した」で保存して在庫が戻らない | unit | B8 `test_create_return_req202_register_processed_jan8_moves_stock`（JAN-8 の `49123456` を持ち、その JAN の商品が 1 つだけの在庫連動の商品を `register_processed=true` で戻り 1 → movement が 1 件、在庫 +1、`stock_applied_product_codes` に入る） | `returns.rs` が形の条件を通さずに候補だけで判定する |
| D-148 (2)（false の枝） | 「読んでいない」で Z004 が在庫を動かす商品の明細が動かない | unit | B9 `test_create_return_req202_unprocessed_moves_every_line`（13 桁の JAN の在庫連動の商品と JAN 無しの商品を `register_processed=false` で戻す → 両方が動き、列に両方が入る） | false の枝でも `z004_moves_stock` を見る |
| UI-03-D23 | 既定が残る、選ばずに保存できる | unit（TS） | U1 `return-exchange-request.test.ts`: null で field error・request なし | null を true/false に寄せる |
| UI-03-D23 | 選ばずに保存で command が呼ばれる、error が欄の近くに出ない | component | U2 `ReturnExchangePage.test.tsx`: 初期表示でどちらの radio も未選択、保存で `createReturn` 未呼出しと文言 | 初期値を true に戻す、validation を外す |
| UI-03-D23 | 選んだ値と request が食い違う | component | U3: 両方の選択で `createReturn` の `register_processed` | radio の値の取り違え |
| UI-03-D23・D15 | 結果の行の文言が実際の動きと食い違う | component | U4: resolve 値の列に入る行は `この保存で反映`、入らない行は `取込みで反映`。`idempotent_replay` でも同じ | `register_processed` から文言を作る |
| D-148 (4) | 在庫が動いたのに在庫系 consumer を invalidate しない | component | U5: 「読んで戻した」で列が非空 → `d052InvalidationOracle.returnExchange(true)` の集合、空 → `(false)` の集合と完全一致（mutation の注入: 引数を `registerProcessed` に戻すと red） | invalidation の条件を `register_processed` に戻す |
| UI-03-D23 | 一覧と詳細の表示が旧い語のまま | component | U6: 最近の一覧と `OtherRecordDetailPages.test.tsx` の返品の詳細で `バーコードで戻した`／`バーコードなし` と movement 0 件の説明文 | 表示の関数を直し忘れる |

状態の遷移: initial（未選択）→ 選択 → submit（validation の失敗で保存しない）→ submitting → success（結果の行・invalidate）／failure（入力と選択を保持し、同内容の retry で同じ key。既存の UI-03-D13 の test が PASS）→ 続けて返品・交換で未選択に戻る（U2 の延長で 1 assert）。

## L3

前提: 開発用の DB に、(i) 13 桁の数字の JAN（合成の値、チェックディジットが合い、全桁ゼロでないもの）を持ち、その JAN の商品が 1 つだけの、在庫連動の商品 1 つ（8 桁の JAN の商品は Z004 の行に当たらずこの保存で在庫を動かすので、L3-3 の (i) の期待が逆になる。B8 `test_create_return_req202_register_processed_jan8_moves_stock`）、(ii) 商品フォームで「POS販売で在庫を減らす」を「減らさない」にした商品 1 つ、(iii) JAN の無い商品 1 つ（どれも商品フォームで作れる。fixture の file は要らない）。

- L3-1 / 返品・交換の画面（入出庫 → 返品・交換） / 「レジでの戻し方」の 2 つの選択肢がどちらも選ばれていない。商品を 1 つ足して「返品・交換を保存」を押すと、欄の近くに「レジでの戻し方を選んでください」が出て保存されない。選択肢の文言と Badge が日本語で読め、色だけで区別していない。
- L3-2 / 同じ画面 / 「バーコードを読んでいない」を選び (i) を戻りで保存 → 保存結果の (i) の行が「この保存で反映」。毎日の業務 → 在庫照会で (i) を検索して在庫を見ると、増えている。
- L3-3 / 同じ画面 / 「レジでバーコードを読んで戻した」を選び (i)・(ii)・(iii) を戻りで保存 → 保存結果で (i) が「取込みで反映」、(ii)・(iii) が「この保存で反映」。毎日の業務 → 在庫照会で商品を検索して在庫を見ると、(i) は変わらず、(ii)・(iii) は増えている。
- L3-4 / 保存結果の「詳細を見る」と最近の一覧 / 記録の値が「バーコードで戻した」／「バーコードなし」で読め、L3-3 の記録の詳細の「関連する在庫変動」に (ii)・(iii) の行がある。
- L3-5 / 同じ画面で種類を「交換」にし、「戻り（在庫+）」と「渡し（在庫-）」の明細を 1 つずつ足す / 「レジでバーコードを読んで戻した」の選択肢の説明文「在庫連動の商品は、売上データ（Z004）の取込みで在庫に戻ります。」が、渡しの明細（在庫が減る向き）にも合うと読めるかを owner が見て、PASS / FAIL で答える。
