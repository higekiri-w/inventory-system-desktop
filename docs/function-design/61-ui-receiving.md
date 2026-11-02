# 61. UI-02: 入庫記録

> **計数の置き場と在庫の求め直し（D-118。(a) と b1 で実装済み）**: この画面の日付の欄の意味は「品が動いた日」（入庫は品が店に届いた日で、納品書の日付ではない）。在庫が計数に入っているかは、この日付で ADR D3 の窓（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）を決める（[31](31-biz-inventory-service.md)）。(a) で入ったのは、この日付を業務日として movement に書くこと（`biz/inventory_service/common.rs` の `validate_business_date`）と、計数と入出庫の窓の計算（`biz/stock_ledger.rs` の `judge` → `from_count` → `derive`）。計数を保存する公開の経路と精算の窓は D-118 の b1 で入った（入出庫の保存は要再確認の一覧を再取得する。UI_TECH_STACK の D-118 の段落）。この画面の欄の見出し・説明の文言は b1 で変えていない。

> 対応仕様: REQ-201 / REQ-209 / UI-02
>
> 入力ドキュメント: `docs/architecture/ui-task-specs.md` UI-02、`docs/architecture/cmd-task-specs.md` CMD-02、`docs/architecture/biz-task-specs.md` BIZ-02、`docs/function-design/31-biz-inventory-service.md`、`docs/function-design/44-cmd-inventory.md`、`docs/SCREEN_DESIGN.md`、`docs/UI_TECH_STACK.md` §5.3

入庫記録は、仕入れ商品が店舗に届いた時に複数商品の数量と原価をまとめて記録し、商品在庫を加算する operator-facing flow である。UI は商品選択、数量・原価入力、取引先選択、保存結果確認を担当し、在庫加算、入庫伝票・明細・在庫変動履歴・操作ログの永続化は BIZ-02 / CMD-02 に委譲する。

## 61.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: UI-02 は frontend page / hooks / component 群として、入庫ヘッダと明細行を `commands.createReceiving(req)` に渡し、成功時に商品在庫系 query を無効化し、入庫記録結果と最近の入庫一覧を表示する。

**シグネチャ**: frontend の境界は `commands.createReceiving(req)`、`commands.listReceivings(page, perPage, dateFrom, dateTo)`、商品追加用の `commands.searchProducts(query)`、取引先候補用の `commands.listSuppliers()` である。React component の詳細 props は §61.2、command / DTO の詳細は §61.4 に定義する。

**処理ステップ**: 画面全体の状態遷移は §61.3、表示・操作は §61.5、エラー復旧は §61.6 に定義する。

## 61.1 設計判断

| Spec | Decision ID | 決定 | 理由 / 不採用案 |
|---|---|---|---|
| REQ-201 / UI-02 | UI-02-D1 | route は `/inventory/receiving` とし、file route は `src/routes/inventory/receiving.tsx`、画面本体は `src/features/receiving/ReceivingPage.tsx` に置く。 | 入出庫エリアの独立作業であり、商品一覧や在庫照会の query param で mode を切り替えない。将来の UI-03〜05 と route prefix をそろえる。 |
| REQ-201 / CMD-02 | UI-02-D2 | UI は generated `commands.createReceiving(req)` / `commands.listReceivings(page, perPage, dateFrom, dateTo)` のみを使う。実装 PR では既存 Rust command に `#[specta::specta]` を付け、`ReceivingCreateRequest` / `ReceivingItemInput` / `ReceivingCreateResult` / `ReceivingRecordWithSupplier` を generated binding に出す。 | `receiving_cmd` は runtime `generate_handler!` には存在するが、現状 `collect_commands!` / `bindings.ts` にはない。`typedInvoke` fallback は退役済みなので ad hoc invoke は採用しない。 |
| REQ-201 / suppliers | UI-02-D3 | 取引先候補は `commands.listSuppliers()` 由来の complete master data とする。取引先欄は picker dialog + 『新しい取引先を追加』を経由する（DSR-24）。初回実装の defer は本 packet（⑱）で解除する。 | 取引先は任意項目だが、誤った master 追加は後から直しづらい。UI-01b と同じ候補取得方針にそろえる。 |
| REQ-201 / product add | UI-02-D4 | 商品追加欄は商品コード / JAN / 商品名を同じ入力で扱い、Enter で `commands.searchProducts` を実行する。1件なら明細へ追加、複数件なら候補リストから選ぶ、0件なら商品登録への導線を出す。 | 未登録商品の作成を入庫画面内に持ち込むと UI-01b の validation と重複する。REQ-201 の「未登録商品」は `/products/new` へ逃がす。 |
| REQ-201 / scanner | UI-02-D5 | バーコードスキャナは HID キーボードとして商品追加欄へ入力される前提にする。初回実装はグローバルスキャン検知を置かず、フォーカス中の入力欄 + Enter で追加し、追加後は商品追加欄へフォーカスを戻す。 | 20ms 間隔などのグローバル検知は入力中の誤検知やフォーカス奪取リスクがある。実店舗確認前に全画面へ広げない。 |
| REQ-201 / rows | UI-02-D6 | 同じ `product_code` を再追加した場合は既存行の数量を +1 し、重複行を増やさない。原価は既存行の値を維持し、必要なら利用者が編集する。 | 連続スキャンでは同一商品の数量増加が自然。重複明細を許すと確認負荷が上がる。原価差のある入庫は手入力で調整できる。 |
| REQ-201 / quantities | UI-02-D7 | 数量は `> 0`（個数の商品は整数、`m` の商品は m で小数 2 桁まで受けて cm に直す。上限は `9999999`）、原価は小数 2 桁までの円 `>= 0`（上限は `999999.99`。送る値は 1/100 円の `cost_price_centi`。`決定（owner、D-113 J2）`、SPEC-UNIT-D5。error の文は `原価は0以上で、小数は2桁までで入力してください`、欄は `inputMode="decimal"`・`step="0.01"`）として保存前に frontend で止める。行の原価の初期値は商品の原価の入力の欄の初期値の文字列（8333 → `83.33`）。原価小計・合計は BIZ が返す値（丸めは TD-023 と同じ、`決定（owner、D-113 J3）`）。[共通規則](10-common-rules.md) SPEC-UNIT-D3・D5・D6。 | BIZ-02 も同じ validation を持つが、operator-facing form では保存前に日本語 field error を出す。m 表示切替は横断表示方針で扱うため非 scope。 |
| REQ-201 / idempotency | UI-02-D8 | `idempotency_key` は画面で1保存試行単位に生成し、保存失敗後の同内容再試行では同じ key を再利用する。保存成功、フォームリセット、別伝票開始、または保存失敗後に伝票内容を編集して再送する場合は新しい key にする。 | 通信・内部エラー後の同内容再送で二重入庫を避ける。BIZ fingerprint は `note` を除外するため、同じ key のまま note だけ変えても保存済み伝票へ反映されない。fingerprint 対象項目を変えると conflict になるため、編集再送は新規伝票試行として扱う。 |
| REQ-201 / submit | UI-02-D9 | 保存中はヘッダ、明細、商品追加、戻る/リセット導線を disabled にし、中断可能に見せない。 | backend は単一 TX。UI だけで cancel 可能に見せると在庫反映状態を誤認させる。 |
| REQ-201 / result | UI-02-D10 | 保存成功後は record_id、登録明細数、stock_warnings、`idempotent_replay` の有無と、保存した商品の一覧（商品コード・商品名と、行ごとの「在庫を見る」）を表示し、「続けて入庫」の導線を出す。「在庫を見る」は商品と在庫の画面を `/products?q=<商品コード>&selected=<商品コード>` で開き、その商品の行の状態（在庫切れ・在庫少・在庫あり、[58](58-ui-stock-inquiry.md) 契約 H）と詳細を出す（D-138、proposed・未実装。今の build は「在庫照会へ戻る」で `/stock` を引数なしで開く）。画面の上の戻る導線は「商品と在庫へ」（`/products` を引数なしで開き、全商品の一覧が出る。[50](50-ui-product-list.md) UI-01a-D1）。 | 店頭作業では連続入庫があり得る。保存済みの証跡は record_id と件数で確認できるようにする。入れた後に時々、限られた商品について在庫が最低限要る数に達したかを確かめる（店主の答え Q-102・TD-423）ので、確かめたい商品だけを 1 回で開ける（Fluent 2 *Relevant*、DSR-19）。引数なしの一覧へ行かせて入れた商品を探し直させない（packet 段 1 の E-02）。限定: `q` は部分一致なので、その商品コードを含む商品が表示件数（既定 100）を超えて該当の商品が 1 ページ目に無いと、`selected` は外れて行は開かない（[58](58-ui-stock-inquiry.md) §58.4。一覧は `q` で絞られたまま残り、pager で届く。データは失わない）。保存の結果から完全一致の 1 行だけを出す形は採らない（統合の画面に別の表示の形を足す）。捨てた案: 保存の結果に在庫の数を出す（保存の結果は保存の後の在庫を返さず、商品ごとに読み直しが要る。基準との比べも出せない）、入れた全商品だけを出した一覧へ行く（商品コードの一覧で絞る検索が無く、command の型を変える）。 |
| REQ-201 / list | UI-02-D11 | 画面下部に最近の入庫記録を `listReceivings(1, 10, null, null)` で表示する。詳細表示・編集・削除は初回実装では扱わない。 | CMD-02 の一覧契約を UI から疎通確認でき、直近伝票の保存結果も確認しやすい。詳細/取消は業務影響が大きく別設計にする。 |
| REQ-201 / cache | UI-02-D12 | 保存成功時の invalidation は [D-052](../decision-log.md) C4 と `src/lib/invalidation-contract.ts` を正本とする。 | 入庫で変わる在庫・履歴とその consumer を table.column 導出で一貫して stale 化し、画面側の key 列挙を廃止する。 |
| REQ-201 / UI-02 | UI-02-D13 | Windows native L3 は owner 目視確認を必須にする。確認対象は navigation、取引先候補、商品検索/スキャン相当 Enter 追加、同一商品数量加算、cm 表示、validation、保存中 disable、結果表示、recent list、在庫を見る（D-138 の後。今の build は在庫照会）/商品登録導線。 | 新規 operator-facing screen であり、連続入力・日本語商品名・フォーカス戻しは CI だけでは判断しづらい。 |
| REQ-201 / product add suggest | UI-02-D14 | 商品追加欄に live 候補プレビュー（catalog ⑮ ProductAddSuggest / SPEC-SUGGEST-D1〜D11）を追加する。Enter commit 経路（UI-02-D4）とフォーカス戻し（UI-02-D5）は不変。lock source は既存 `isFormLocked` 派生 state、候補確定は既存の複数件候補テーブル選択と同一 handler。 | variant B。純 autocomplete 型（Enter 追加廃止）はスキャナ Enter が候補 async 読込み前に届くため退行・不採用（owner 方針 2026-08-04）。契約正本は catalog ⑮ に一元化し、画面 doc には画面固有の結線のみ置く（5 画面複製 drift 防止）。 |
| REQ-209 / SPEC-PRV-D8 | UI-02-D15 | 保存成功時に `ReceivingCreateResult.cost_diffs` があれば原価差分ダイアログを開き、入庫記録が保存済みであることを明示したうえで、商品ごとに「マスタ原価をこの実原価に更新する」か見送る。承諾時は `revise_product_price` を新売価 = 現売価据え置き、新原価 = 入庫実原価で呼ぶ。見送り時は入庫記録を残したまま、商品マスタの原価を変更しない。 | 手芸用品以外は入庫伝票が価格改定の唯一の導線。自動上書きは約定値引きを恒久化する恐れがあるため、商品ごとの確認を必須にする。見送りは無記録とし、差分が残れば次回入庫で再提示する。 |
| REQ-201 / REQ-206 / return | UI-02-D16 | 保存結果と recent list の「詳細を見る」は、search state を含む現在の `/inventory/receiving` URL を `returnTo` として入庫記録詳細へ送る（DSR-18）。 | 詳細確認後に入庫作業画面へ戻れるようにする。送信・検証・fallback の横断規範は DSR-18 を正とし、本書では入庫画面の producer だけを固定する。 |
| REQ-201 / scope | UI-02-D17 | 取り寄せ（客注）の品が届いたときは入庫に記録しない（owner の決定、repo 外の回答台帳 TD-350、2026-10-20）。アプリの検査・画面の文言は足さず、決まりを本書に置く。 | [project-memory](../project-memory.md) の Store Premises Facts の棚卸しの項: 取り寄せは先払いで、受注の時点で JAN が分からないことがあるのでレジで部門キーで打ち、先払いの取り寄せ品はお渡し待ちの間に棚卸しになっても数えない。店の在庫として数えない品を入庫に記録すると、在庫数に入ってしまう。 |
| REQ-201 / REQ-301 / navigation | UI-02-D18 | search の `productCode`（商品コードの string、1〜100 文字）で開いたら、その商品を `getStockDetail(productCode)`（完全一致）で読み、廃番でなければ商品追加欄から 1 件を追加したときと同じ明細を 1 行入れる（数量 1、原価はマスタの原価）。入れた後は URL から `productCode` を外す（`replace`。F5 で 2 行目を入れない）。この `replace` は明細を入れるのと同じ event の中か前に行い、入れた後の再描画を待つ別の effect に分けない（入力が変わると離脱の guard〈`src/hooks/useUnsavedChangesWarning.ts:15` の `shouldBlockFn: () => isDirty`、`src/features/receiving/ReceivingPage.tsx:204` で結線〉が同じ route の search の `replace` も止め、開いた直後に破棄の確認が出て URL に `productCode` が残るため）。見つからない・廃番なら明細を入れず、商品追加欄の下に「商品コード {code} の商品は入庫に追加できません（見つからないか廃番です）」を出す。商品と在庫の画面の行の詳細の「入庫記録」が使う（[58](58-ui-stock-inquiry.md) §58.7。詳細は廃番の商品にこの導線を出さない）（D-138、proposed・未実装）。 | 届いた商品を在庫を見た画面からそのまま入庫へ進め、商品を入れ直させない（Fluent 2 *Relevant*）。商品コードの部分一致の検索（Enter の経路）でなく完全一致で読み、似たコードの商品を入れない。捨てた案: 事前入力しない（今の build。入れ直しの手間が残る）、Enter の経路を再生する（部分一致で候補が出ることがある）。 |

## 61.2 Component / Route 構成

```text
src/
  routes/
    inventory/
      receiving.tsx
  features/
    receiving/
      ReceivingPage.tsx
      components/
        ReceivingHeaderForm.tsx
        ReceivingProductSearch.tsx
        ReceivingItemTable.tsx
        ReceivingRecentList.tsx
        ReceivingResultPanel.tsx
      hooks/
        useReceivingForm.ts
        useReceivingOptions.ts
      lib/
        build-receiving-request.ts
        receiving-validation.ts
        receiving-row-utils.ts
        test-fixtures.ts
```

`receiving.tsx` は route と title を管理し、画面本体は `ReceivingPage` に委譲する。`ReceivingPage` は form state、queries、mutation、query invalidation、子コンポーネントへの props 配線を担当する。DTO 組み立てと validation は `lib/` の純関数へ分け、component test と unit test の両方から検証する。

## 61.3 State Model

| State | 意味 | 主な表示 | 次の action |
|---|---|---|---|
| `editing` | 入庫ヘッダ/明細編集中 | header form、商品追加欄、明細表、最近の入庫一覧 | `add_product` / `update_row` / `submit` |
| `searching_product` | 商品追加欄で検索中 | 商品追加欄 loading | `product_found` / `product_candidates` / `product_not_found` / `search_failed` |
| `submitting` | createReceiving 実行中 | form disabled、spinner + 「入庫を記録しています」 | `submit_succeeded` / `submit_failed` |
| `result` | 保存完了 | result panel（保存した商品の一覧と行ごとの在庫を見る）、続けて入庫 | `reset_for_next` / navigation |
| `submit_error` | 保存失敗 | Alert + 入力保持 + 再実行 | `submit_same_request` / `edit_as_new_attempt` |

フォーム state は `receivingDate`, `supplierId`, `note`, `rows`, `idempotencyKey` を持つ。`rows` は `productCode`, `productName`, `stockUnit`, `currentStockQuantity`, `quantity`, `costPrice` を持つ UI 内部型とし、command payload には `product_code`, `quantity`, `cost_price_centi` だけを送る。`submit_error` 後に同内容を再送する場合は `idempotencyKey` を保持し、入力を編集して再送する場合は新しい `idempotencyKey` を採番する。

## 61.4 Command / DTO Contract

UI-02 実装 PR では以下を generated binding に出す。

| Command / Type | Existing backend | UI-02 usage |
|---|---|---|
| `commands.createReceiving(req)` | Rust command は既存、generated 未対応 | 入庫ヘッダ + 明細を保存 |
| `commands.listReceivings(page, perPage, dateFrom, dateTo)` | Rust command は既存、generated 未対応 | 最近の入庫記録を表示 |
| `ReceivingCreateRequest` | BIZ type は既存、`specta::Type` 未対応 | create payload |
| `ReceivingItemInput` | BIZ type は既存、`specta::Type` 未対応 | 明細 payload |
| `ReceivingCreateResult` | BIZ type は既存、`specta::Type` 未対応 | 保存結果 |
| `CostDiff` | SPEC-PRV-D8 で追加 | 商品コード・商品名・マスタ原価・入庫実原価の差分 |
| `commands.reviseProductPrice(input)` | 実装 A で生成 | 差分承諾時のマスタ原価更新（新売価は現売価据え置き） |
| `ReceivingRecordWithSupplier` | DB type は既存、`serde::Serialize` のみ | 最近の入庫一覧 |

`createReceiving` payload:

```text
{
  idempotency_key: string,
  supplier_id: number | null,
  receiving_date: "YYYY-MM-DD",
  note: string | null,
  items: [{ product_code: string, quantity: number, cost_price_centi: number }]
}
```

`listReceivings` は `perPage=10` を初期値とし、CMD/BIZ の上限200を UI から超えない。

`ReceivingCreateResult` は既存 field に `cost_diffs: CostDiff[]` を追加するだけで、既存 consumer は無視できる。idempotent replay は空配列を返す（SPEC-PRV-D8）。

## 61.5 表示 / 操作

- PageHeader title は `入庫記録`、subtitle は仕入れ商品の到着時に在庫へ反映する作業であることを短く示す。
- ヘッダは `入庫日`（既定は今日）、`取引先`（任意）、`備考`（任意）を置く。取引先取得失敗時は警告を出し、取引先未指定の保存は許可する。
- 商品追加欄は `商品コード・JAN・商品名で追加` とし、Enter で検索する。候補が複数ある場合は小さな候補リストで商品コード、商品名、部門、現在庫を見せる（catalog ⑮ の取引 4 画面の先頭の列。今の build の候補は現在庫の列を持たない〈packet 段 1 の X-02〉、D-138 の runtime で正本に合わせる）。
- 商品追加欄は入力中に live 候補プレビュー（商品コード・商品名・部門、最大 5 件 + 超過 footer）を表示する（UI-02-D14、挙動契約は catalog ⑮ SPEC-SUGGEST-D1〜D11）。候補は ↓/↑ または click で選択でき、候補未選択の Enter は従来どおり検索を実行する。入力値が変わると候補は即座に閉じる。
- 商品が見つからない場合は `商品登録へ` 導線を出す。入庫フォームは自動保存しないため、明細・入力がある状態での離脱（`商品登録へ` 導線を含む）は未保存編集の離脱ガード（UI_TECH_STACK §6.11 `useUnsavedChangesWarning`）が破棄確認を出す。
- 明細表は商品コード、商品名、部門、現在庫、単位、入庫数量、原価、行削除を表示する（先頭の 4 列は catalog ⑮ の取引 4 画面の共通。今の build の明細は部門・現在庫の列を持たない〈X-02〉、D-138 の runtime で正本に合わせる）。長さの商品の現在庫は m で出し、`m` の商品の数量は m で入力する（[共通規則](10-common-rules.md) SPEC-UNIT-D3）。
- 明細が 0 件の場合、保存ボタンは disabled にし、理由を「商品が追加されていません」と表示する。
- 保存ボタンは `入庫を記録`。保存成功後は result panel へ移り、フォーム本文は操作不可にする。保存結果の「詳細を見る」は UI-02-D16 に従って現在の入庫画面 URL を `returnTo` として送る。保存結果や保存系エラーの Alert はページ先頭側に出るため、保存成功または command 失敗時はページ先頭へスクロールする。
- 保存成功時に `cost_diffs` が 1 件以上なら原価差分ダイアログを表示する。説明文は、入庫の記録は保存済みであること、「マスタ原価をこの実原価に更新する」を選ぶと商品マスタの原価が今回の実原価に変わること、「見送って閉じる」を選ぶと入庫記録はそのまま残り商品マスタの原価は変わらないこと、の 3 点を明記する。各カードは商品名を一意見出しとして、商品コード・マスタ原価・今回の実原価を日本語ラベルで示し、状態を色だけに依存させない。「マスタ原価をこの実原価に更新する」は商品単位で実行し、成功 / 失敗を同じカードで判別できるようにする（UI-02-D15 / DSR-16）。更新成功後はカードのマスタ原価表示を更新後の値へ反映する。全行の更新が完了した場合、ダイアログ下部の閉じるボタンの文言を状態対応（「閉じる」）に変える。原価差分ダイアログは明示ボタンのみで閉じる（外側クリック・Escape・× では閉じない）。
- 最近の入庫一覧は日付、取引先、備考、記録日時を表示する。「詳細を見る」は UI-02-D16 に従って現在の入庫画面 URL を `returnTo` として送る。作成画面内に詳細本文、編集、取消は出さない。

## 61.6 Error / Recovery

- 商品検索失敗: 商品追加欄の近くに Alert を出し、既存明細は保持する。
- 商品 0 件: `商品が見つかりません` と表示し、`商品登録へ` 導線を出す。
- 取引先取得失敗: 警告を出し、取引先未指定の保存は許可する。
- 日付不正、明細 0 件、数量 0 以下、原価負数: command 呼び出し前に日本語 field error を出す。
- frontend validation error: エラー箇所が明細表や入力欄の近くに出るため、ページ先頭へはスクロールしない。
- BIZ validation error: form state を保持する。入力を修正して再送する場合は新しい `idempotencyKey` を採番し、同じ key で異なる fingerprint を送らない。
- internal error: 入力と `idempotencyKey` を保持し、同内容のまま同じ伝票として再試行できるようにする。入力を編集して再送する場合は新しい `idempotencyKey` を採番する。
- idempotent replay: result panel に「同じ内容の再送として処理済み」と表示し、二重登録ではないことを示す。
- 原価差分更新失敗: 入庫保存の成功表示は維持し、該当商品の差分行だけ再試行可能にする。見送りは記録せず閉じられ、次回入庫で差分が残れば再提示される。

## 61.7 Cache / Navigation

- 保存成功時は D-052-C4 の SSOT helper を適用する。具体的な query key 集合は `src/lib/invalidation-contract.ts` だけに置く。
- `navigation.ts` の UI-02 は `to: "/inventory/receiving"`, `status: "active"` に切り替える。
- 商品と在庫の画面の行の詳細から入庫へ遷移する場合は `?productCode=...` でその商品を明細に入れて開く（UI-02-D18、D-138）。今の build は事前入力しない。
- 明細または header 入力が初期値と異なる間だけ共通 `useUnsavedChangesWarning` を有効にし、保存中 / result panel 表示中は `!isFormLocked` により解除する。

## 61.8 Non-scope / Follow-up

- 入庫作成画面内での入庫伝票詳細表示、編集、取消。履歴調査用の詳細 route は `65-inventory-record-traceability.md` §65.10 の横展開 slice で扱う。
- 入庫明細の CSV import。
- inline 商品登録 / inline 取引先登録。
- グローバル barcode scan detection。
- cm / m 表示切替。
- 仕入先別原価履歴、発注書連携、レシート/納品書画像添付。
- 入庫後の PLU 書出し誘導。

## 61.9 Test Focus

- UI-02-D1: `/inventory/receiving` route で page title と navigation active が一致する。
- UI-02-D2: `createReceiving` / `listReceivings` が generated binding に存在し、ad hoc invoke を使わない。
- UI-02-D3: `listSuppliers` 失敗時も取引先未指定保存が可能。取引先欄は picker dialog 経由で選択・追加できる（検索 / 追加 / 自動選択で閉じる）。
- UI-02-D4/D5: 商品追加欄 Enter で検索し、1件なら行追加、複数件なら候補選択、0件なら商品登録導線を出す。
- UI-02-D6: 同一 `product_code` の再追加で数量が +1 され、重複行を増やさない。
- UI-02-D7: 数量/原価（小数 2 桁の円・1/100 円の送信）validation と長さの商品の m の入力と表示。原価差分の dialog の原価は SPEC-UNIT-D5 の formatter（`¥83.33`）で出し、更新は `new_cost_price_centi`（UI-02-D15）。
- UI-02-D8: 同内容 retry 時に `idempotency_key` を再利用し、成功/リセット/編集再送後は新規 key になる。
- UI-02-D9: submitting 中は戻る/リセット/入力/商品追加が disabled になる。
- UI-02-D10: result で record_id、明細数、warning、idempotent replay が読め、保存成功時にページ先頭へスクロールする。保存した商品ごとの「在庫を見る」を押した先の `/products` で、route の parse の後の `q` と `selected` が商品コードの string（`1234567890123` のような数字だけの値でも）で、その行が開く（href の文字列は見ない。router の直列化は数字だけの値を引用符つきで出す）。引数なしの在庫の画面への link が無い（D-138）。
- UI-02-D18: `productCode` で開くと明細が 1 行入り URL から外れる、F5 で 2 行目が入らない、見つからない・廃番は明細を入れず案内を出す（D-138）。
- UI-02-D11: 最近の入庫一覧を取得し、空/取得失敗/成功を表示できる。
- UI-02-D12: 保存成功時の実呼出し集合が D-052-C4 の独立 test oracle と完全一致する。
- UI-02-D13: Windows native L3 で連続入力、フォーカス戻し、日本語表示、保存結果を確認する。
- UI-02-D15: 保存完了時だけ原価差分ダイアログが出て、空配列 / idempotent replay では出ない。説明文から「入庫は保存済み」「更新時は商品マスタ原価が今回の実原価へ変わる」「見送り時は入庫記録を残して商品マスタ原価を変えない」の 3 点を読め、複数商品は一意見出しで識別できる。「マスタ原価をこの実原価に更新する」が `reviseProductPrice` を現売価据え置き・入庫実原価で呼び、見送りは無記録である。
- UI-02-D16: 保存結果と recent list の「詳細を見る」が、現在の入庫画面 URL を `returnTo` として送る。

## 61.10 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-10-23 | docs の小口（[Plan Packet](../archive/plans/2026-10-23-docs-small-batch.md) の S5） | UI-02-D17 を追加: 取り寄せ（客注）の品を入庫に記録しない決まり（TD-350）。 |
| 2026-09-10 | PR #50 | 取引先ピッカー runtime を反映。検索・追加後の自動選択と指定なしへの復帰を実装。 |
| 2026-09-10 | PR #49 / DSR-24 | owner Human Gate round 1（2026-09-10）に基づき、UI-02-D3 の defer を解除し、取引先欄へ picker dialog と追加導線を適用。§61.9 に picker dialog の検索・追加・自動選択で閉じるテスト観点を追加。 |
| 2026-08-30 | PR #20 / DSR-18 | UI-02-D16 を追加し、保存結果 / recent list の詳細導線に入庫画面への `returnTo` 送信契約を設定。 |
| 2026-08-30 | UI 表示磨き batch 第 2 弾 design sync | UI-02-D15 の説明文 3 点と、複数商品の一意見出し契約を明記。 |
| 2026-08-22 | 価格改定支援 design-first | SPEC-PRV-D8 / REQ-209 の保存後原価差分ダイアログ（UI-02-D15）を追加。 |
| 2026-08-05 | product add suggest design | UI-02-D14 新設: 商品追加欄の live 候補プレビュー（正本 = catalog ⑮ SPEC-SUGGEST-D1〜D10）。Enter commit 経路・フォーカス戻しは不変。 |
| 2026-08-03 | UI safety net implementation | 入庫 form の dirty 判定を共通離脱ガードへ接続し、保存中 / 保存結果の非 block を実装。 |
| 2026-06-28 | Save result visibility follow-up | 保存成功または command 失敗時はページ先頭へスクロールし、result panel / Alert が画面内に入るようにした。frontend validation は近傍表示のためスクロールしない。 |
| 2026-06-25 | UI-02 Design Phase | route、generated command 方針、商品追加/スキャン前提、明細 validation、冪等キー、query invalidation、recent list、Windows native L3 を整理。 |
