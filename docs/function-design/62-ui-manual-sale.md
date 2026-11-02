# 62. UI-04: 手動販売出庫

> **計数の置き場と在庫の求め直し（D-118。(a) と b1 で実装済み）**: この画面の日付の欄の意味は「品が動いた日」（手動販売は品を渡した日）。在庫が計数に入っているかは、この日付で ADR D3 の窓（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）を決める（[31](31-biz-inventory-service.md)）。(a) で入ったのは、この日付を業務日として movement に書くこと（`biz/inventory_service/common.rs` の `validate_business_date`）と、計数と入出庫の窓の計算（`biz/stock_ledger.rs` の `judge` → `from_count` → `derive`）。計数を保存する公開の経路と精算の窓は D-118 の b1 で入った（入出庫の保存は要再確認の一覧を再取得する。UI_TECH_STACK の D-118 の段落）。この画面の欄の見出し・説明の文言は b1 で変えていない。

> 対応仕様: REQ-203 / UI-04
>
> 入力ドキュメント: `docs/architecture/ui-task-specs.md` UI-04、`docs/architecture/cmd-task-specs.md` CMD-04、`docs/architecture/biz-task-specs.md` BIZ-02、`docs/function-design/31-biz-inventory-service.md`、`docs/function-design/44-cmd-inventory.md`、`docs/SCREEN_DESIGN.md`、`docs/UI_TECH_STACK.md` §5.3

手動販売出庫は、CSV取込みで記録されない販売を手入力し、売上記録と在庫減算を同時に残す operator-facing flow である。主用途は PLU 書出し前の新商品販売の補完であり、PLU 登録済み商品を手動販売に入れる場合は、レジで打てる商品を二重記録しないよう確認を挟む。

## 62.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: UI-04 は frontend page / hooks / component 群として、販売日、理由、備考、明細行を `commands.createManualSale(req)` に渡し、PLU 登録済み警告がある場合は確認 token 付きで再送し、成功時に在庫系・売上系 query を無効化して保存結果を表示する。画面下部には保存直後確認用の recent list を置き、既存の `commands.listInventoryRecords(query)` を `record_type="manual_sale"` で呼び出して直近の手動販売出庫を表示する。

**シグネチャ**: frontend の境界は `commands.createManualSale(req)`、商品追加用の `commands.searchProducts(query)` である。React component の詳細 props は §62.2、command / DTO の詳細は §62.4 に定義する。

**処理ステップ**: 画面全体の状態遷移は §62.3、表示・操作は §62.5、エラー復旧は §62.6 に定義する。

## 62.1 設計判断

| Spec | Decision ID | 決定 | 理由 / 不採用案 |
|---|---|---|---|
| REQ-203 / UI-04 | UI-04-D1 | route は `/inventory/manual-sale` とし、file route は `src/routes/inventory/manual-sale.tsx`、画面本体は `src/features/manual-sale/ManualSalePage.tsx` に置く。 | 入出庫エリアの独立作業であり、UI-02/03/05 と route prefix をそろえる。 |
| REQ-203 / CMD-04 | UI-04-D2 | UI は generated `commands.createManualSale(req)` のみを使う。実装 PR では既存 Rust command に `#[specta::specta]` を付け、`ManualSaleCreateRequest` / `ManualSaleItemInput` / `ManualSaleCreateResult` を generated binding に出す。 | `manual_sale_cmd` は runtime `generate_handler!` には存在するが、現状 `collect_commands!` / `bindings.ts` にはない。`typedInvoke` fallback は退役済みなので ad hoc invoke は採用しない。 |
| REQ-203 / reason | UI-04-D3 | 理由は `"plu_unregistered"` を既定値にし、UI 表示は「PLU未登録商品の販売」とする。`"other"` は「その他」と表示する。 | REQ-203 の主用途は CSV取込みで拾えない販売の補完、特に PLU 書出し前の新商品販売。enum の英語値を画面に出さない。 |
| REQ-203 / product add | UI-04-D4 | 商品追加欄は商品コード / JAN / 商品名を同じ入力で扱い、Enter で `commands.searchProducts` を実行する。1件なら明細へ追加、複数件なら候補リストから選ぶ、0件なら商品登録への導線を出す。 | UI-02 と同じ scan-like flow にして学習コストを下げる。未登録商品の作成は `/products/new` へ逃がし、手動販売画面内で master mutation をしない。 |
| REQ-203 / scanner | UI-04-D5 | バーコードスキャナは HID キーボードとして商品追加欄へ入力される前提にする。初回実装はグローバルスキャン検知を置かず、フォーカス中の入力欄 + Enter で追加し、追加後は商品追加欄へフォーカスを戻す。 | UI-02 と同じ方針。グローバル検知は全画面 UX への影響が大きく別 Design Phase。 |
| REQ-203 / rows | UI-04-D6 | 同じ `product_code` を再追加した場合は既存行の数量を +1 し、重複行を増やさない。販売金額は商品売価を初期値にし、重複追加時は売価分を加算する。利用者は金額を手修正できる。 | レジ外販売でも同一商品を連続入力する可能性がある。重複明細を許すと確認負荷が上がる。割引や端数調整は金額欄の手修正で扱う。 |
| REQ-203 / validation | UI-04-D7 | 数量は整数 `> 0`、販売金額は整数 `>= 0`、販売日は必須として command 前に frontend で止める。 | BIZ-02 も同じ validation を持つが、operator-facing form では保存前に日本語 field error を出す。 |
| REQ-203 / PLU confirmation | UI-04-D8 | `createManualSale` が `needs_confirmation=true` を返した場合、DB は未変更として扱い、PLU警告パネルを出す。利用者が「確認して保存」を押した場合だけ、同じ `idempotency_key` と `confirmation_token` で再送する。 | PLU登録済み商品はレジで打てるため、手動出庫すると二重売上/二重在庫減算のリスクがある。BIZ の token 方式を UI でも明示する。 |
| REQ-203 / confirmation edit | UI-04-D9 | PLU確認待ち中に販売日、理由、明細、備考を変更したら `confirmation_token` を破棄し、新しい保存試行として `idempotency_key` を再採番する。 | token は商品・PLU状態・数量・金額に結び付く。利用者が見直した内容は再度 BIZ の PLU警告判定を通す方が安全。 |
| REQ-203 / idempotency | UI-04-D10 | `idempotency_key` は画面で1保存試行単位に生成し、保存失敗後の同内容再試行では同じ key を再利用する。保存成功、フォームリセット、別伝票開始、または保存失敗後に伝票内容を編集して再送する場合は新しい key にする。 | UI-02 と同じ二重登録防止方針。BIZ fingerprint は `note` を除外するため、同じ key のまま note だけ変えても保存済み伝票へ反映されない。 |
| REQ-203 / submit | UI-04-D11 | 保存中はヘッダ、明細、商品追加、戻る/リセット導線を disabled にし、中断可能に見せない。 | backend は単一 TX。UI だけで cancel 可能に見せると売上・在庫反映状態を誤認させる。 |
| REQ-203 / result | UI-04-D12 | 保存成功後は sale_id、登録明細数、PLU警告件数、stock_warnings、`idempotent_replay` の有無と、保存した商品の一覧（商品コード・商品名と、行ごとの「在庫を見る」。[61](61-ui-receiving.md) UI-02-D10 と同じ形で、`/products?q=<商品コード>&selected=<商品コード>` を開く）を表示し、「続けて手動販売」「詳細を見る」「日次売上へ」の導線を出す（D-138、proposed・未実装。今の build は「在庫照会へ戻る」で `/stock` を引数なしで開く）。画面の上の戻る導線は「商品と在庫へ」（D-138）。 | 手動販売は売上帳票にも影響する。保存後に日次売上で「手動」バッジを確認でき、同時に業務記録詳細であとから追える導線を持たせる。 |
| REQ-203 / cache | UI-04-D13 | 保存成功時の invalidation は [D-052](../decision-log.md) C6 と `src/lib/invalidation-contract.ts` を正本とする。 | 手動販売で変わる売上・在庫・履歴 consumer を一貫して stale 化し、画面側の key 列挙を廃止する。 |
| REQ-203 / UI-04 | UI-04-D14 | Windows native L3 は owner 目視確認を必須にする。確認対象は navigation、商品検索/スキャン相当 Enter 追加、同一商品数量加算、validation、PLU警告確認フロー、保存結果、日次売上への遷移、日次売上の「手動」Badge 可読性、在庫を見る導線（D-138 の後。今の build は在庫照会へ戻る導線）。 | 新規 operator-facing screen であり、PLU警告文言、連続入力、フォーカス戻し、手動 Badge の可読性は CI だけでは判断しづらい。PR #99 で手動 Badge は UI-04 実装時 L3 へ持ち越している。 |
| REQ-203 / REQ-206 / UI-04 | UI-04-D15 | 手動販売出庫にも保存直後確認用の recent list を置く。表示は直近数件の業務日付、記録ID、代表商品、明細数、状態、記録日時、詳細導線とし、`すべての履歴を見る` は `/inventory/records?recordType=manual_sale` へ遷移する。 | UI-02/03/05 と同じ「保存直後の確認」体験にそろえる。作成画面内で検索・取消・訂正まで扱う案は、入力作業を重くし、調査責務を `入出庫履歴` と重複させるため採用しない。 |
| REQ-203 / product add suggest | UI-04-D16 | 商品追加欄に live 候補プレビュー（catalog ⑮ ProductAddSuggest / SPEC-SUGGEST-D1〜D11）を追加する。Enter commit 経路（UI-04-D4）とフォーカス戻し（UI-04-D5）は不変。lock source は既存 `isFormLocked` 派生 state、候補確定は既存の複数件候補テーブル選択と同一 handler。 | UI-02-D14 と同一の variant B 判断（純 autocomplete 型はスキャナ退行のため不採用）。契約正本は catalog ⑮。 |
| REQ-203 / REQ-206 / return | UI-04-D17 | 保存結果と recent list の「詳細を見る」は、search state を含む現在の `/inventory/manual-sale` URL を `returnTo` として手動販売詳細へ送る（DSR-18）。 | 詳細確認後に手動販売作業画面へ戻れるようにする。送信・検証・fallback の横断規範は DSR-18 を正とし、本書では手動販売画面の producer だけを固定する。 |
| REQ-203 / D-113 | UI-04-D18 | 明細の数量欄は商品の単位で受ける（m の商品は m で小数 2 桁まで。[共通規則](10-common-rules.md) SPEC-UNIT-D3。wire は在庫の数量の整数〈長さは cm〉。上限は `9999999`）。UI-04-D7 の「数量は整数」は個数の商品に限る。商品を追加したときの数量の初期値は価格の基準数量ぶん（個数の商品 `1`、m の商品 `1`〈1 m〉）、販売金額の初期値は SPEC-UNIT-D7 の `売価 × 数量 ÷ 基準数量`（円未満は四捨五入、`決定（owner、D-113 J4）`）。利用者が金額欄を編集するまでは、数量を変えるたびに金額をこの式で求め直す（長さの商品〈`m`〉だけ。個数の商品は数量を変えても金額を変えず、再追加で売価を足す〈UI-04-D6〉）。数量が入力の error の間、長さの商品の未編集の金額欄は空にする。金額欄を編集した後は、数量を変えても金額を変えない。同じ商品の再追加は数量に基準数量ぶん（個数 1、長さ 1 m）を足し、金額は長さの商品で未編集なら式で求め直し、それ以外（個数の商品、金額を編集した長さの商品）は売価を足す（UI-04-D6 のまま）。金額の計算は TS の twin（SPEC-UNIT-D9）。 | 長さの商品で数量 1 = 1 cm に 1 m あたりの売価を付けると 100 倍になる（既知の不整合、Backlog）。長さの商品は数量を必ず直すので、金額が初期値のまま残ると気づかずに違う金額を保存しやすい。不採用: 金額を常に式で上書きする（割引・端数の手修正が消える）。数量を変えても金額を変えない形を長さの商品にも残す（金額の直し忘れ）。 |

## 62.2 Component / Route 構成

```text
src/
  routes/
    inventory/
      manual-sale.tsx
  features/
    manual-sale/
      ManualSalePage.tsx
      lib/
        manual-sale-request.ts
        manual-sale-row-utils.ts
      types.ts
```

初回実装では UI-02 と同じく `ManualSalePage` に form state、queries、mutation、query invalidation、表示を集約し、DTO 組み立てと validation は `lib/` の純関数へ分ける。子 component 分割は重複が増えた時点で別 PR とする。

## 62.3 State Model

| State | 意味 | 主な表示 | 次の action |
|---|---|---|---|
| `editing` | 手動販売ヘッダ/明細編集中 | header form、商品追加欄、明細表 | `add_product` / `update_row` / `submit` |
| `searching_product` | 商品追加欄で検索中 | 商品追加欄 loading | `product_found` / `product_candidates` / `product_not_found` / `search_failed` |
| `confirming_plu` | BIZ が PLU登録済み警告を返し、DB未変更 | warning panel、確認して保存 / 明細を見直す | `confirm_submit` / `edit_as_new_attempt` |
| `submitting` | createManualSale 実行中 | form disabled、spinner + 「手動販売を記録しています」 | `submit_succeeded` / `submit_failed` / `needs_confirmation` |
| `result` | 保存完了 | result panel（保存した商品の一覧と行ごとの在庫を見る）、続けて手動販売、日次売上へ | `reset_for_next` / navigation |
| `submit_error` | 保存失敗 | Alert + 入力保持 + 再実行 | `submit_same_request` / `edit_as_new_attempt` |

フォーム state は `saleDate`, `reason`, `note`, `rows`, `idempotencyKey`, `confirmationToken` を持つ。`rows` は `productCode`, `productName`, `departmentName`, `stockUnit`, `currentStockQuantity`, `unitPrice`, `quantity`, `amount` を持つ UI 内部型とし、command payload には `product_code`, `quantity`, `amount` だけを送る。

## 62.4 Command / DTO Contract

UI-04 実装 PR では以下を generated binding に出す。

| Command / Type | Existing backend | UI-04 usage |
|---|---|---|
| `commands.createManualSale(req)` | Rust command は既存、generated 未対応 | 手動販売ヘッダ + 明細を保存、または PLU確認 token を受け取る |
| `commands.listInventoryRecords(query)` | generated 対応済み | recent list 用に `record_type="manual_sale"`, `page=1`, `per_page=5` で手動販売ヘッダを取得する |
| `ManualSaleCreateRequest` | BIZ type は既存、`specta::Type` 未対応 | create payload |
| `ManualSaleItemInput` | BIZ type は既存、`specta::Type` 未対応 | 明細 payload |
| `ManualSaleCreateResult` | BIZ type は既存、`specta::Type` 未対応 | 保存結果 / PLU確認結果 |

`createManualSale` payload:

```text
{
  idempotency_key: string,
  sale_date: "YYYY-MM-DD",
  reason: "plu_unregistered" | "other",
  note: string | null,
  items: [{ product_code: string, quantity: number, amount: number }],
  confirmation_token: string | null
}
```

**enum 契約化（D-061）**: `reason` は bindings 由来の `ManualSaleReason` generated union を利用する。値・表示文言は不変。

`createManualSale` result:

```text
{
  sale_id: number | null,
  created: boolean,
  idempotent_replay: boolean,
  plu_warnings: string[],
  stock_warnings: string[],
  needs_confirmation: boolean,
  confirmation_token: string | null
}
```

`needs_confirmation=true` の場合、`sale_id=null`、`created=false`、`confirmation_token` は non-null とみなす。UI はこの状態を保存完了として扱わず、確認待ちとして表示する。

## 62.5 表示 / 操作

- PageHeader title は `手動販売出庫`、subtitle はレジCSVに入らない販売を手入力し、在庫と売上へ反映する作業であることを短く示す。
- ヘッダは `販売日`（既定は今日）、`理由`（既定「PLU未登録商品の販売」）、`備考`（任意）を置く。
- 商品追加欄は `商品コード・JAN・商品名で追加` とし、Enter で検索する。候補が複数ある場合は商品コード、商品名、部門、現在庫、売価を見せる。
- 商品追加欄は入力中に live 候補プレビュー（商品コード・商品名・部門、最大 5 件 + 超過 footer）を表示する（UI-04-D16、挙動契約は catalog ⑮ SPEC-SUGGEST-D1〜D11）。候補は ↓/↑ または click で選択でき、候補未選択の Enter は従来どおり検索を実行する。入力値が変わると候補は即座に閉じる。
- 商品が見つからない場合は `商品登録へ` 導線を出す。未登録商品は商品マスタに登録してから手動販売へ戻ることを日本語で示す。既存明細・入力がある状態での離脱（`商品登録へ` 導線を含む）は未保存編集の離脱ガード（UI_TECH_STACK §6.11 `useUnsavedChangesWarning`）が破棄確認を出す。
- 明細表は商品コード、商品名、部門、現在庫、数量（単位を同じ cell に添える）、販売金額（同じ cell に「円」を添える）、行削除を表示する。数量と単位を一つの値として読めるよう独立した単位列は置かず、現在庫も単位付き（例: `4 個` / `3 m`、桁区切り付き）にする。長さの商品の現在庫は m で出し、`m` の商品の数量は m で入力する（[共通規則](10-common-rules.md) SPEC-UNIT-D3）。
- 明細が 0 件の場合、保存ボタンは disabled にし、理由を「商品が追加されていません」と表示する。
- 保存ボタンは通常 `手動販売を保存`、PLU確認待ちでは `確認して保存` とする。
- PLU確認待ちでは warning Alert を表示し、BIZ が返した `plu_warnings` を行ごとに見せる。利用者は「確認して保存」または明細修正を選べる。
- 保存成功後は result panel へ移り、フォーム本文は操作不可にする。保存結果の「詳細を見る」は UI-04-D17 に従って現在の手動販売画面 URL を `returnTo` として送る。保存結果、PLU確認待ち、保存系エラーの Alert はページ先頭側に出るため、`createManualSale` の成功応答または command 失敗時はページ先頭へスクロールする。
- 画面下部に `直近の手動販売出庫` セクションを置く。見出し右側に `すべての履歴を見る` を置き、`/inventory/records?recordType=manual_sale` へ遷移する。
- recent list は保存直後確認 UI として、直近数件の業務日付、記録ID、代表商品、明細数、状態、記録日時、`詳細を見る` を表示する。`詳細を見る` は `/inventory/manual-sale/records/$recordId` へ遷移し、UI-04-D17 に従って現在の手動販売画面 URL を `returnTo` として送る。
- recent list が空の場合は `直近の手動販売出庫はありません` と表示する。取得失敗時は入力フォームを壊さず、recent セクション内に再試行可能なエラー表示を置く。

## 62.6 Error / Recovery

- 商品検索失敗: 商品追加欄の近くに Alert を出し、既存明細は保持する。
- 商品 0 件: `商品が見つかりません` と表示し、`商品登録へ` 導線を出す。
- 日付不正、明細 0 件、数量 0 以下、販売金額負数: command 呼び出し前に日本語 field error を出す。
- frontend validation error: エラー箇所が明細表や入力欄の近くに出るため、ページ先頭へはスクロールしない。
- `needs_confirmation=true`: 保存完了ではなく PLU確認待ちとして表示する。DB は未変更。確認して保存する場合は同じ `idempotency_key` と `confirmation_token` を送る。
- BIZ validation error: form state を保持する。入力を修正して再送する場合は新しい `idempotencyKey` を採番し、同じ key で異なる fingerprint を送らない。
- internal error: 入力と `idempotencyKey` を保持し、同内容のまま同じ伝票として再試行できるようにする。入力を編集して再送する場合は新しい `idempotencyKey` を採番する。
- idempotent replay: result panel に「同じ内容の再送として処理済み」と表示し、二重登録ではないことを示す。

## 62.7 Cache / Navigation

- 保存成功時は D-052-C6 の SSOT helper を適用する。具体的な query key 集合は `src/lib/invalidation-contract.ts` だけに置く。
- recent list は `queryKeys.inventoryRecords.list({ recordType: "manual_sale", page: 1, perPage: 5 })` 相当の stable key を使う。`per_page` は 5 固定で、既存 `listInventoryRecords` の上限 200 内に収める。保存成功後の追随は D-052-C6 が担う。
- `navigation.ts` の UI-04 は `to: "/inventory/manual-sale"`, `status: "active"` に切り替える。
- result panel の `詳細を見る` は `/inventory/manual-sale/records/{sale_id}` へ遷移する。`sale_id=null` の PLU確認待ちでは表示しない。
- result panel の `日次売上へ` は `/reports/daily?date={saleDate}` へ遷移する。
- 明細または header 入力が初期値と異なる間だけ共通 `useUnsavedChangesWarning` を有効にし、保存中 / result panel 表示中は `!isFormLocked` により解除する。

## 62.8 Non-scope / Follow-up

- 手動販売作成画面内での検索、詳細本文表示、編集、取消、訂正。recent list は保存直後確認に限定し、履歴調査用の検索と詳細本文は `入出庫履歴` / `/inventory/manual-sale/records/$recordId` へ逃がす。
- 手動販売明細の CSV import。
- inline 商品登録。
- グローバル barcode scan detection。
- cm / m 表示切替（利用者が切り替える UI は作らない。長さの入力と表示は UI-04-D18 と SPEC-UNIT-D3 で決まる）。
- PLU書出し画面への自動誘導。
- レシート添付。返品・交換の UI-03 で別設計する。

## 62.9 Test Focus

- UI-04-D1: `/inventory/manual-sale` route で page title と navigation active が一致する。
- UI-04-D2: `createManualSale` が generated binding に存在し、ad hoc invoke を使わない。
- UI-04-D3: 理由 enum を日本語表示し、payload では `"plu_unregistered"` / `"other"` を送る。
- UI-04-D4/D5: 商品追加欄 Enter で検索し、1件なら行追加、複数件なら候補選択、0件なら商品登録導線を出す。
- UI-04-D6: 同一 `product_code` の再追加で数量が +1 され、販売金額が売価分加算され、重複行を増やさない。
- UI-04-D7: 数量/販売金額 validation と長さの商品の m の入力と表示。
- UI-04-D8/D9: `needs_confirmation=true` で DB変更済みの result に進まず、token 付き再送だけで保存する。編集時は token を破棄する。
- UI-04-D10: 同内容 retry 時に `idempotency_key` を再利用し、成功/リセット/編集再送後は新規 key になる。
- UI-04-D11: submitting 中は戻る/リセット/入力/商品追加が disabled になる。
- UI-04-D12: result で sale_id、明細数、warning、idempotent replay が読め、業務記録詳細と日次売上へ遷移でき、保存成功時にページ先頭へスクロールする。
- UI-04-D13: 保存成功時の実呼出し集合が D-052-C6 の独立 test oracle と完全一致する。
- UI-04-D14: Windows native L3 で連続入力、フォーカス戻し、日本語表示、PLU警告確認、保存結果、日次売上の「手動」Badge を確認する。
- UI-04-D15: recent list に `すべての履歴を見る` と `詳細を見る` が表示され、保存成功後に recent list が更新される。取得失敗時も入力フォームは継続できる。
- UI-04-D17: 保存結果と recent list の「詳細を見る」が、現在の手動販売画面 URL を `returnTo` として送る。
- UI-04-D18: m の商品を追加すると数量 `1`・金額 = 売価、数量を `1.3` にすると金額が `売価 × 130 ÷ 100` の丸め、金額を編集した後は数量を変えても金額が変わらない。個数の商品の再追加は UI-04-D6 のまま。payload の数量は cm の整数。

## 62.10 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-10-08 | 単位の拡張 design lane | UI-04-D18 の金額の初期値の丸めを owner の決定（四捨五入、D-113 J4、TD-195）に改めた。 |
| 2026-10-07 | 単位の拡張 design lane | UI-04-D18 を追加（proposed・未実装、D-113）: 数量を商品の単位で受け、金額の初期値を `売価 × 数量 ÷ 基準数量` にし、金額を編集するまで数量に合わせて求め直す。 |
| 2026-08-30 | PR #20 / DSR-18 | UI-04-D17 を追加し、保存結果 / recent list の詳細導線に手動販売画面への `returnTo` 送信契約を設定。 |
| 2026-08-05 | product add suggest design | UI-04-D16 新設: 商品追加欄の live 候補プレビュー（正本 = catalog ⑮ SPEC-SUGGEST-D1〜D10）。Enter commit 経路・フォーカス戻しは不変。 |
| 2026-08-03 | UI safety net implementation | 手動販売 form の dirty 判定を共通離脱ガードへ接続し、保存中 / 保存結果の非 block を実装。 |
| 2026-06-28 | Save result visibility follow-up | 保存成功、PLU確認待ち、command 失敗時はページ先頭へスクロールし、result panel / Alert が画面内に入るようにした。frontend validation は近傍表示のためスクロールしない。 |
| 2026-06-26 | UI-04 Design Phase | route、generated command 方針、商品追加/スキャン前提、明細 validation、PLU警告二段階確認、冪等キー、query invalidation、Windows native L3 を整理。 |
| 2026-06-27 | Record detail expansion | 保存結果から手動販売詳細へ遷移する導線と、入出庫履歴 query invalidation を追記。 |
| 2026-06-28 | Manual sale recent follow-up | 保存直後確認用の `直近の手動販売出庫`、`すべての履歴を見る`、`詳細を見る` を追加する仕様へ更新。 |
