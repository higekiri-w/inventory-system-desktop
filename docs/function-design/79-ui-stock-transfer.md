# 79. UI-16: 詰め替え・切り分け（振替）

> **状態**: D-143、実装済み（[decision-log](../decision-log.md) D-143、Plan Packet `docs/plans/2026-10-29-stock-transfer.md`）。画面の語は全て「詰め替え・切り分け」で、「振替」は設計の中の名前（DB の `transfer`・`transfer_record`、BIZ の `create_transfer`）。
>
> **計数の置き場と在庫の求め直し（D-118）**: この画面の日付の欄の意味は「品を詰め替え・切り分けた日」。在庫が計数に入っているかは、この日付で ADR D3 の窓（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）を決める。振替の movement は入出庫の movement と同じ規則で求め直す（[31](31-biz-inventory-service.md) §12.6b）。同じ日付に既定の計数のある商品は要再確認になり、棚卸しの画面の要再確認の一覧と商品の画面の範囲の表示に出る（ほかの入出庫の画面と同じ。この画面の保存の結果には要再確認の案内を足さない）。

> 対応仕様: REQ-210 / UI-16 / UI-16b（D-143）
>
> 入力ドキュメント: `docs/function-design/21-io-inventory-repo.md` §10.5a、`docs/function-design/31-biz-inventory-service.md` §12.6b、`docs/function-design/44-cmd-inventory.md` §23.7a、`docs/function-design/64-ui-disposal.md`（同じ形の作成画面）、`docs/function-design/65-inventory-record-traceability.md` TRACE-D14、`docs/function-design/10-common-rules.md` SPEC-UNIT-D2・D3・D11、`docs/SCREEN_DESIGN.md`、`docs/UI_TECH_STACK.md`

詰め替え・切り分けは、仕入れた品を別の商品にして売るとき（反物 → 端切れ、大袋 → 小袋）に、減らす商品（元）の在庫を減らし、増やす商品（先）の在庫を増やして、1 つの記録に残す operator-facing flow である。UI は日付と 2 つの明細の表（減らす商品・増やす商品）を扱い、在庫の変動・記録・在庫変動履歴・操作ログの永続化は BIZ-02（`create_transfer`）/ CMD-13 に委ねる。原価・金額は扱わない（D-143 (4)）。

## 79.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: UI-16 は frontend page / hooks / component 群として、日付と明細を `commands.createTransfer(req)` に渡し、成功時に D-052-C24 の query を無効化して保存結果を表示する。UI-16b は `commands.getTransferRecord(recordId)` で記録の詳細を読取り専用で表示する。

**シグネチャ**: frontend の境界は `commands.createTransfer(req)`、`commands.getTransferRecord(recordId)`、最近の記録の `commands.listInventoryRecords(query)`（`record_type: "transfer_record"`）、商品追加用の `commands.searchProducts(query)` である。command / DTO は §79.4。

**処理ステップ**: 画面の状態は §79.3、表示・操作は §79.5、エラー復旧は §79.6、記録の詳細は §79.8。

## 79.1 設計判断

| Spec | Decision ID | 決定 | 理由 / 不採用案 |
|---|---|---|---|
| D-143 / UI-16 | UI-16-D1 | route は `/inventory/transfer`、file route は `src/routes/inventory/transfer.tsx`（layout）と `src/routes/inventory/transfer/index.tsx`、画面本体は `src/features/stock-transfer/StockTransferPage.tsx`。サイドバーの入出庫の「廃棄・破損」の次に置き、label と画面の題は「詰め替え・切り分け」、description は「仕入れた品を別の商品にした数を移します」。 | 入出庫のほかの作成画面（UI-02〜05）と prefix と置き場を揃える。店の語（TD-358 の問いの語）で名指し、「振替」は画面に出さない（[52](52-ui-shared-layout.md) UI-12 の route 表）。不採用: 廃棄・破損の画面の中の切替（ロスの画面に別の目的の操作が混ざる）、商品の画面から始める形（元と先の 2 つの商品を選ぶ所が要り、商品の画面の道具が増える）。owner の判断事項 (i)。 |
| D-143 / CMD-13 | UI-16-D2 | UI は generated `commands.createTransfer(req)` / `commands.getTransferRecord(recordId)` と既存の `commands.listInventoryRecords(query)` だけを使う。振替だけの一覧の command は作らない。 | 最近の記録は入出庫履歴と同じ一覧で足りる（[21](21-io-inventory-repo.md) §10.5a）。 |
| D-143 / rows | UI-16-D3 | 明細は「減らす商品」と「増やす商品」の 2 つの表に分け、どちらも 1 行以上を要る。各表に商品追加欄を 1 つずつ置く（[64](64-ui-disposal.md) UI-05-D5・D6・D16 と同じ検索・スキャン・候補の形）。同じ表に同じ商品を追加すると数量を +1 する。もう片方の表にある商品は追加せず、その欄の近くに「この商品は「減らす商品」に入っています」（または「増やす商品」）と出す。2 つの商品追加欄の検索の範囲は 64 と同じで、廃番の商品を含めない（`searchProducts` の `is_discontinued: false`。`src/components/patterns/useProductAddSuggest.ts`。owner の判断事項 (iv) の既定。BIZ〈[31](31-biz-inventory-service.md) §12.6b〉は廃番を見ないので、範囲を変えるときは画面の検索の条件だけを変える）。 | 1 本の反物を 2 種の端切れに分ける・複数の商品から 1 つを作る記録を 1 回で残す（owner の判断事項 (iii) の既定 = 両方とも複数行）。同じ商品を減らして増やすことは無いので、追加の時点で名指して止める（BIZ も拒む。[31](31-biz-inventory-service.md) §12.6b の手順 5）。 |
| D-143 / quantity | UI-16-D4 | 数量は行ごとにその商品の単位で入れる（個数は整数、`m` の商品は m で小数 2 桁まで受けて cm に直す。上限は `9999999`〈長さは `99999.99` m〉。[64](64-ui-disposal.md) UI-05-D8 と同じ純関数）。元と先の数の釣り合いは求めず、換算もしない。保存の前に、減らす商品・増やす商品のどちらかが 0 行、数量が 0 以下・形の誤り・上限越えを日本語の field error で止める。 | 元と先の数の対応は回ごとに違いうる（D-143 (3)）。長さと個数を足し合わせる表示は作らない（SPEC-UNIT-D3 の集計の規則に当たる所を作らない）。 |
| D-143 / cost | UI-16-D5 | 原価・金額の欄を持たない。保存は商品の原価を変えない。 | D-143 (4)。owner の判断事項 (ii) の既定 A。 |
| D-143 / idempotency | UI-16-D6 | `idempotency_key` は [64](64-ui-disposal.md) UI-05-D9 と同じ（1 保存試行に 1 つ、同内容の再試行で再利用、成功・リセット・編集の後の再送で新しくする）。 | 二重の振替を防ぐ。 |
| D-143 / submit | UI-16-D7 | 保存中は日付・2 つの表・2 つの商品追加欄・戻る/リセットを disabled にする。非同期の商品検索の結果の採否は UI-05-D15 と同じ event 境界の lock を 1 つ持ち、2 つの商品追加欄が同じ lock を読む。リセット・「続けて」の前に始まった検索の応答は成功・失敗とも採らず、保存失敗後に完了した応答は UI-05-D15 に従って採る。 | backend は単一 TX。片方の欄の検索の結果が保存の後に行を足さないようにする。 |
| D-143 / result | UI-16-D8 | 保存成功後は record_id、減らした商品と増やした商品の一覧（商品コード・商品名・数量と単位と、行ごとの「在庫を見る」。`/products?q=<商品コード>` を開く。[64](64-ui-disposal.md) UI-05-D11 の先は `selected=<商品コード>` も渡すが、今の `/products` の search schema に `selected` が無いので `q` だけにし、D-138 の実装の時に 64 と一緒に揃える）、stock_warnings（在庫が負になった商品）、`idempotent_replay` の有無を出し、「続けて詰め替え・切り分け」「詳細を見る」を出す。 | 保存の後に元と先の在庫を 1 回で確かめられるようにする。 |
| D-143 / list | UI-16-D9 | 画面下部に最近の記録を `listInventoryRecords({ record_type: "transfer_record", page: 1, per_page: 10 })` で出す（日付・記録ID・代表商品・記録日時・「詳細を見る」）。「すべての履歴を見る」は `/inventory/records?recordType=transfer_record`。per_page は 10 で、CMD・BIZ の上限 200（[65](65-inventory-record-traceability.md) §65.4.2）を越えない。 | 他の作成画面の recent list と同じ役割（[65](65-inventory-record-traceability.md) TRACE-D1・D7）。 |
| D-143 / cache | UI-16-D10 | 保存成功時の invalidation は D-052 の C24（`invalidationContract.stockTransfer`。起票の時は C23 だったが、#212 の D-141 が先に C23 を取ったので main との同期で振り直した）。集合は廃棄の C7 から `disposals.root()` を除いたもの（`inventoryRecords.root()`・`productList.root()`・`lowStock(false)`・`stockInquiryRoot()`・`stockMovements.root()`・`productForm.root()`・`stocktake.itemsRoot()`・`stocktake.recheckList()`）で、正本は `src/lib/invalidation-contract.ts` と decision-log D-052 の Contract 行。 | 振替が書く表（`transfer_*`・`inventory_movements`・`products` の写し）を読む consumer は廃棄と同じで、廃棄の一覧だけを読まない（UI_TECH_STACK §2.5 の導出）。 |
| D-143 / return | UI-16-D11 | 保存結果と recent list の「詳細を見る」は、現在の `/inventory/transfer` URL を `returnTo` として詳細へ送る（DSR-18、[65](65-inventory-record-traceability.md) TRACE-D11）。 | 他の作成画面と同じ戻り方。 |
| D-143 / L3 | UI-16-D12 | Windows native L3 で owner が見る: サイドバーから開く、2 つの表へのスキャン相当の Enter 追加、長さの商品の m の入力と表示、もう片方の表にある商品の追加を止める案内、保存中の disable、保存結果と「在庫を見る」、recent list と詳細、在庫変動履歴の種別と元記録。 | 新しい作成画面で、2 つの表の入力の分かりやすさは CI で判断できない（`docs/DEV_WORKFLOW.md` Human Visual Confirmation）。 |

## 79.2 Component / Route 構成

```text
src/
  routes/
    inventory/
      transfer.tsx                     -- layout（Outlet）
      transfer/
        index.tsx                      -- UI-16
        records/
          $recordId.tsx                -- UI-16b
  features/
    stock-transfer/
      StockTransferPage.tsx
      lib/
        transfer-request.ts            -- DTO の組み立てと保存の前の検証（純関数）
    inventory-records/
      TransferRecordDetailPage.tsx     -- UI-16b
```

route の入れ子は廃棄の `src/routes/inventory/disposal.tsx`・`disposal/index.tsx`・`disposal/records/$recordId.tsx` と同じ形。明細の表と商品追加欄は廃棄の画面の部品を流用し、2 つの表で同じ部品を 2 回使う（新しい共通部品は作らない。重複が増えたら別の変更で共通化する）。

## 79.3 State Model

| State | 意味 | 主な表示 | 次の action |
|---|---|---|---|
| `editing` | 日付と 2 つの表を編集中 | 日付、減らす商品の表と追加欄、増やす商品の表と追加欄、最近の記録 | `add_product(side)` / `update_row` / `submit` |
| `searching_product` | どちらかの追加欄で検索中 | その追加欄の loading | `product_found` / `product_candidates` / `product_not_found` / `search_failed` |
| `submitting` | `createTransfer` 実行中 | form disabled、spinner +「詰め替え・切り分けを記録しています」 | `submit_succeeded` / `submit_failed` |
| `result` | 保存完了 | 保存結果（§79.5）、続けて詰め替え・切り分け、詳細を見る | `reset_for_next` / navigation |
| `submit_error` | 保存失敗 | Alert + 入力保持 + 再実行 | `submit_same_request` / `edit_as_new_attempt` |

フォーム state は `transferDate`、`outRows`、`inRows`、`idempotencyKey` を持つ。行は `productCode`、`productName`、`departmentName`、`stockUnit`、`currentStockQuantity`、`quantity` を持つ UI 内部型で、command payload には `product_code`・`direction`（`outRows` は `"out"`、`inRows` は `"in"`）・`quantity`（在庫の数量の単位の整数）を送る。items の順は `outRows` の後に `inRows`。

## 79.4 Command / DTO Contract

| Command / Type | 正本 | UI-16 の使い方 |
|---|---|---|
| `commands.createTransfer(req)` | [44](44-cmd-inventory.md) §23.7a | ヘッダ + 明細を保存 |
| `commands.getTransferRecord(recordId)` | 同上 | UI-16b の詳細 |
| `commands.listInventoryRecords(query)` | [44](44-cmd-inventory.md) §23.7 | 最近の記録（`record_type: "transfer_record"`） |
| `TransferCreateRequest` / `TransferItemInput` / `TransferCreateResult` / `TransferDirection` | [31](31-biz-inventory-service.md) §12.6b | payload と結果 |
| `TransferRecordDetail` / `TransferRecordDetailItem` | [21](21-io-inventory-repo.md) §10.5a | 詳細 |

`createTransfer` payload:

```text
{
  idempotency_key: string,
  transfer_date: "YYYY-MM-DD",
  items: [
    { product_code: string, direction: "out" | "in", quantity: number }
  ]
}
```

## 79.5 表示 / 操作

- PageHeader の題は `詰め替え・切り分け`、subtitle は「仕入れた品を別の商品にしたとき、減らす商品と増やす商品の数を記録します。原価は変わりません」。
- 日付の欄は `詰め替え・切り分けた日`（既定は今日）。
- 「減らす商品（元）」の表と「増やす商品（先）」の表を上下に並べ、各表の上に商品追加欄（`商品コード・JAN・商品名で追加`）を置く。表の列は商品コード、商品名、部門、現在庫（単位付き）、数量（単位を同じ cell に添える）、行削除（[64](64-ui-disposal.md) §64.5 の明細表と同じ形。種別・原価・理由の列は無い）。
- どちらかの表が 0 行の間は保存ボタンを disabled にし、理由を「減らす商品と増やす商品を 1 つ以上ずつ入れてください」と出す。
- 保存ボタンは `詰め替え・切り分けを保存`。保存成功・command 失敗ではページ先頭へスクロールする（UI-05 と同じ）。
- 保存結果: 「詰め替え・切り分けを記録しました（記録 #{record_id}）」、減らした商品と増やした商品の一覧（数量と単位、行ごとの「在庫を見る」）、stock_warnings、`idempotent_replay` なら「同じ内容の再送として処理済み」。
- 最近の記録は日付、記録ID、代表商品、記録日時、「詳細を見る」。作成画面に詳細本文・編集・取消は出さない。

## 79.6 Error / Recovery

- 商品検索失敗・0 件・未登録商品: UI-05 と同じ（その追加欄の近くに Alert、0 件は `商品登録へ` 導線。既存の行は保持）。
- もう片方の表にある商品の追加: 行を足さず、その追加欄の近くに「この商品は「減らす商品」に入っています」（または「増やす商品」）と出す。直し方は、どちらの表に入れるかを決めて、要らない方の行を削除すること。
- 日付不正、どちらかの表が 0 行、数量 0 以下・形の誤り・上限越え: command の前に日本語の field error。ページ先頭へはスクロールしない。
- BIZ validation error（[31](31-biz-inventory-service.md) §12.6b の手順 0・1・2・5 の文。画面の検証を通らずに届くのは同じ商品の重複と、画面の後に変わった商品の不在）: form state を保持する。入力を直して再送するときは新しい `idempotencyKey`。
- internal error: 入力と `idempotencyKey` を保持し、同内容のまま再試行できる。
- 誤った振替を保存した後: 取消は無い（D-143 (5)）。減らす商品と増やす商品を入れ替えた振替を、元の記録と同じ日付・同じ数量で保存して戻す（同じ日付に計数のある商品は要再確認になる。ADR の過剰側 H5 と同じ）。この案内を記録の詳細（§79.8）に出す。

## 79.7 Cache / Navigation

- 保存成功時は D-052-C24 の SSOT helper を適用する（UI-16-D10）。
- `src/config/navigation.ts` の入出庫の「廃棄・破損」の次に `{ id: "ui-16", label: "詰め替え・切り分け", to: "/inventory/transfer", status: "active" }` を足し、`navigation.test.ts` に到達の test を足す（`docs/templates/plan-packet.md` の Registration / Generation Obligations）。
- 明細か日付が初期値と違う間だけ `useUnsavedChangesWarning` を有効にし、保存中 / 保存結果の表示中は外す（UI-05 §64.7 と同じ）。

## 79.8 UI-16b: 詰め替え・切り分けの記録の詳細

- route `/inventory/transfer/records/$recordId`、本体 `src/features/inventory-records/TransferRecordDetailPage.tsx`（ほかの記録の詳細と同じ置き場と形。[65](65-inventory-record-traceability.md) TRACE-D14）。
- 見出しは「詰め替え・切り分け #{id}」、記録ID・日付・作成日時・状態。「減らした商品」と「増やした商品」の 2 つの表（商品コード・商品名・部門・数量と単位）と、関連する在庫変動（既存の `MovementTable`）。
- 取消・訂正の CTA は出さない。代わりに「間違えたときは、この記録と同じ日付・同じ数量で、減らす商品と増やす商品を入れ替えて、もう一度詰め替え・切り分けを記録してください」を出す（§79.6。作成画面の日付の既定は今日なので、日付を名指さないと、後日の訂正が別の日の在庫の動きになり、間の日に計数のある商品では在庫が戻らない）。
- 「前の画面へ戻る」は `returnTo`（DSR-18）、欠落・不正は `/inventory/records`。
- 記録が無い（NotFound）ときは他の記録の詳細と同じ not found の表示。

## 79.9 Non-scope / Follow-up

- 取消・訂正（REQ-208 の取消の設計に後で入れる）。
- 元と先の換算の比の記憶、先の数の自動計算（D-143 の Revisit）。
- 先の原価を元の原価から求める・提案する（owner の判断事項 (ii) の B・C）。
- 振替だけの一覧画面（入出庫履歴で種別を絞る）。
- 商品の画面から始める導線（商品と在庫の行の詳細から開く入口）。

## 79.10 Test Focus

- UI-16-D1: `/inventory/transfer` で題と navigation の active が一致し、サイドバーの入出庫の廃棄・破損の次に「詰め替え・切り分け」がある。
- UI-16-D2: `createTransfer` / `getTransferRecord` が generated binding にあり、ad hoc invoke を使わない。
- UI-16-D3: 2 つの表に Enter で追加でき、同じ表の再追加で数量 +1、もう片方の表にある商品は追加されず案内が出る。payload の `direction` が表に対応する。
- UI-16-D4: どちらかの表が 0 行で保存できない。数量の 0・負・形の誤り・上限越えを保存の前に止め、長さの商品は m で入れて cm で送る。
- UI-16-D6: 同内容の再試行で `idempotency_key` を再利用し、成功・リセット・編集の後は新しい key。
- UI-16-D7: 保存中は 2 つの追加欄と表が disabled、保存 event と競合して完了した検索の結果を反映しない。両側でリセット・「続けて」の前の検索は成功・失敗とも反映せず、保存失敗後の検索完了は反映する。
- UI-16-D8: 保存結果に record_id、2 つの一覧、warning、replay が読める。
- UI-16-D9: 最近の記録を `record_type: "transfer_record"` で取得し、空・失敗・成功を出せる。
- UI-16-D10: 保存成功時の実呼出し集合が D-052-C24 の独立 test oracle と完全一致する。
- UI-16-D11: 保存結果と recent list の「詳細を見る」が現在の URL を `returnTo` で送る。
- UI-16b: 詳細に 2 つの表と関連在庫変動、取消の代わりの案内（「この記録と同じ日付・同じ数量で」を含む）、戻り先。

## 79.11 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-10-27 | stock-transfer の起票（D-143） | 新設（runtime の前の設計）。route、2 つの表の明細、数量と単位、原価を持たないこと、冪等キー、invalidation C23、recent list、記録の詳細、L3 の確認対象を決めた。 |
| 2026-10-31 | stock-transfer の runtime（PR #216） | 実装に合わせて状態を現行にし、`> 対応仕様:` に REQ-210 を書いた。 |
| 2026-11-01 | stock-transfer の是正（PR #216） | UI-16-D8 の「在庫を見る」の先を今の契約（`q` だけ）に直し、`selected` を D-138 の実装に送った。 |
| 2026-11-02 | stock-transfer の是正（PR #216） | UI-16-D7 にリセット・「続けて」の前の検索応答を採らない契約と、保存失敗後は UI-05-D15 に従って採る契約を明記した。 |
| 2026-11-02 | main との同期（PR #216） | #212 の D-141 が D-052 の C23 を先に取ったので、振替の invalidation の entry を C24 に振り直した（owner 判断 TD-562。集合は変えない）。 |
