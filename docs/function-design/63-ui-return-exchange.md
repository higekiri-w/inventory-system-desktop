# 63. UI-03: 返品・交換

> **計数の置き場と在庫の求め直し（D-118。(a) と b1 で実装済み）**: この画面の日付の欄の意味は「品が動いた日」（返品・交換は品が戻った・渡った日）。在庫が計数に入っているかは、この日付で ADR D3 の窓（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）を決める（[31](31-biz-inventory-service.md)）。(a) で入ったのは、この日付を業務日として movement に書くこと（`biz/inventory_service/common.rs` の `validate_business_date`）と、計数と入出庫の窓の計算（`biz/stock_ledger.rs` の `judge` → `from_count` → `derive`）。計数を保存する公開の経路と精算の窓は D-118 の b1 で入った（入出庫の保存は要再確認の一覧を再取得する。UI_TECH_STACK の D-118 の段落）。この画面の欄の見出し・説明の文言は b1 で変えていない。

> 対応仕様: REQ-202 / UI-03
>
> 入力ドキュメント: `docs/architecture/ui-task-specs.md` UI-03、`docs/architecture/cmd-task-specs.md` CMD-03、`docs/architecture/biz-task-specs.md` BIZ-02、`docs/function-design/28-io-image-manager.md`、`docs/function-design/31-biz-inventory-service.md`、`docs/function-design/43-cmd-settings-log.md`、`docs/function-design/44-cmd-inventory.md`、`docs/SCREEN_DESIGN.md`、`docs/UI_TECH_STACK.md` §5.3 / §6.7

返品・交換は、顧客返品または交換を帳面記録し、レジで戻し処理していない場合だけ在庫を増減する operator-facing flow である。UI は返品種別、レジ戻し済みフラグ、商品明細、任意のレシート画像を扱い、在庫反映と返品記録の永続化は BIZ-02 / CMD-03 に委譲する。

## 63.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: UI-03 は frontend page / hooks / component 群として、返品日、返品種別、レジ戻し済みフラグ、備考、任意のレシート画像、明細行を `commands.createReturn(req)` に渡し、成功時に返品一覧と必要な在庫系 query を無効化して保存結果を表示する。

**シグネチャ**: frontend の境界は `commands.createReturn(req)`、`commands.listReturns(page, perPage, dateFrom, dateTo)`、画像保存用の `commands.saveReceiptImage(request)`、商品追加用の `commands.searchProducts(query)` である。React component の詳細 props は §63.2、command / DTO の詳細は §63.4 に定義する。

**処理ステップ**: 画面全体の状態遷移は §63.3、表示・操作は §63.5、エラー復旧は §63.6 に定義する。

## 63.1 設計判断

| Spec | Decision ID | 決定 | 理由 / 不採用案 |
|---|---|---|---|
| REQ-202 / UI-03 | UI-03-D1 | route は `/inventory/return` とし、file route は `src/routes/inventory/return.tsx`、画面本体は `src/features/return-exchange/ReturnExchangePage.tsx` に置く。 | 2026-04-21 の UI route 合意で UI-03 は `/inventory/return` と決めている。画面名は日本語で「返品・交換」だが、URL は短く安定した `return` を使う。 |
| REQ-202 / CMD-03 | UI-03-D2 | UI は generated `commands.createReturn(req)` / `commands.listReturns(page, perPage, dateFrom, dateTo)` のみを使う。実装 PR では既存 Rust command に `#[specta::specta]` を付け、`ReturnCreateRequest` / `ReturnItemInput` / `ReturnCreateResult` / `ReturnRecordSummary` を generated binding に出す。 | `return_cmd` は runtime `generate_handler!` には存在するが、現状 `collect_commands!` / `bindings.ts` にはない。`typedInvoke` fallback は退役済みなので ad hoc invoke は採用しない。 |
| REQ-202 / image | UI-03-D3 | レシート画像は任意添付とし、実装 PR では `commands.saveReceiptImage(request)` も generated binding に出す。画像は `createReturn` 前に保存し、返却された `relative_path` を `receipt_image_path` に入れる。 | 既存 DB/BIZ 契約は `receipt_image_path` を受け取る形で、`create_return` 自身は画像バイトを扱わない。新しい combined command は責務境界が大きく変わるため初回 UI-03 では採用しない。 |
| REQ-202 / image input | UI-03-D4 | 画像選択は `input type="file" accept="image/*"` + drag/drop で扱い、選択後は `URL.createObjectURL(file)` で保存前プレビューを表示する。`@tauri-apps/plugin-dialog` は初回 UI-03 では使わない。 | `plugin-dialog.open()` は path を返すが、現状 frontend から file bytes を読む filesystem plugin / permission がない。`saveReceiptImage` は base64 bytes を要求するため、Web file input が最小で正しい。 |
| REQ-202 / image input | UI-03-D20 | 画像選択を共通 FilePicker（accept="image/*"、native dialog + 任意 drop、D-054）へ移行し UI-03-D4 を supersede する。保存前プレビューは選択 bytes から Blob を生成して表示する。 | D4 の却下理由「file bytes を読む filesystem plugin / permission がない」は日報 PR #125 の `plugin-fs.readFile` + capability 導入で解消済み。`saveReceiptImage` の bytes 要求とも整合する。 |
| REQ-202 / image retry | UI-03-D5 | `saveReceiptImage` 成功後に `createReturn` が失敗した場合、UI は保存済み `relative_path` を保持し、同じ画像の同内容 retry では再保存しない。画像・備考だけを変更して再送する場合も新しい `idempotency_key` を採番する。 | BIZ fingerprint は `receipt_image_path` / `note` を除外する。同じ key のまま画像や備考だけ変えても、既存レコードへ反映されたと誤認するため。ファイルと DB は単一 TX にできないので、失敗後 retry で orphan を増やさない設計にする。 |
| REQ-202 / return type | UI-03-D6 | 種別は `"return"`（返品）と `"exchange"`（交換）を radio / segmented control で選ぶ。`"return"` では明細 direction は `"in"` のみ許可し、`"exchange"` では `"in"`（戻り）と `"out"`（渡し）を行ごとに選べる。この不変条件は UI の事前 validation と BIZ-02 の最終 validation の両方で守る。 | 返品と交換は同じ CMD/BIZ 契約だが、operator の確認観点が違う。返品で渡し行を許すと業務意味が崩れるため、generated command 経由の呼び出しでも BIZ が拒否する。 |
| REQ-202 / exchange rows | UI-03-D7 | 交換として保存する場合は、少なくとも戻り行 1 件と渡し行 1 件を要求する。返品として保存する場合は戻り行 1 件以上を要求する。この不変条件は UI の事前 validation と BIZ-02 の最終 validation の両方で守る。 | 交換は「戻った商品」と「渡した商品」の対で記録する業務。戻りだけなら返品として記録する方が帳面上明確なので、BIZ でも片側だけの exchange を拒否する。 |
| REQ-202 / register processed | UI-03-D8 | `register_processed` は既定 true（レジ戻し済み）にする。true の場合は在庫はこの画面で動かないこと、false の場合はこの画面で在庫が増減することを、日本語説明 + Badge で表示する。 | 通常の返金/戻しはレジ操作に載り、Z004 取込みで反映される。二重計上を避けるため、在庫反映有無を色だけでなく文言で明示する。 |
| REQ-202 / product add | UI-03-D9 | 商品追加欄は商品コード / JAN / 商品名を同じ入力で扱い、Enter で `commands.searchProducts` を実行する。1件なら明細へ追加、複数件なら候補リストから選ぶ、0件なら商品登録への導線を出す。 | UI-02 / UI-04 と同じ scan-like flow にして学習コストを下げる。未登録商品作成は `/products/new` へ逃がし、返品画面内で master mutation をしない。 |
| REQ-202 / scanner | UI-03-D10 | バーコードスキャナは HID キーボードとして商品追加欄へ入力される前提にする。初回実装はグローバルスキャン検知を置かず、フォーカス中の入力欄 + Enter で追加し、追加後は商品追加欄へフォーカスを戻す。 | UI-02 / UI-04 と同じ方針。グローバル検知は全画面 UX への影響が大きく別 Design Phase。 |
| REQ-202 / rows | UI-03-D11 | 明細の一意性は `product_code + direction` とし、同じ組み合わせを再追加した場合は数量を +1 する。交換では同じ商品を戻り/渡しの両方に持つことを許す。 | 色違い交換や同一商品交換の記録では direction が意味を持つ。product_code だけで統合すると戻りと渡しを相殺してしまう。 |
| REQ-202 / validation | UI-03-D12 | 数量は `> 0`（個数の商品は整数、`m` の商品は m で小数 2 桁まで受けて cm に直す。上限は `9999999`。[共通規則](10-common-rules.md) SPEC-UNIT-D3）、返品日は必須、方向は `"in"` / `"out"`、種別は `"return"` / `"exchange"` として command 前に frontend で止める。BIZ-02 は同じ validation を最終防御として持つ。 | operator-facing form では保存前に日本語 field error を出す。generated command は UI 以外からも呼べるため、業務不変条件は BIZ に置く。 |
| REQ-202 / idempotency | UI-03-D13 | `idempotency_key` は画面で1保存試行単位に生成し、保存失敗後の同内容再試行では同じ key を再利用する。保存成功、フォームリセット、別伝票開始、または保存失敗後に fingerprint 対象項目・備考・画像を編集して再送する場合は新しい key にする。 | UI-02 / UI-04 と同じ二重登録防止方針。BIZ fingerprint は `receipt_image_path` / `note` を除外するため、補足情報の変更でも利用者視点では別の保存試行として扱う。 |
| REQ-202 / submit | UI-03-D14 | 画像保存中と返品保存中はヘッダ、明細、商品追加、画像選択、戻る/リセット導線を disabled にし、中断可能に見せない。 | backend DB は単一 TX だが画像保存は別操作。UI だけで cancel 可能に見せると、画像保存済み / DB未保存の状態を誤認させる。 |
| REQ-202 / result | UI-03-D15 | 保存成功後は record_id、登録明細数、レジ戻し済みか、画像添付有無、stock_warnings、`idempotent_replay` の有無と、保存した商品の一覧（商品コード・商品名と、行ごとの「在庫を見る」。[61](61-ui-receiving.md) UI-02-D10 と同じ形で、`/products?q=<商品コード>&selected=<商品コード>` を開く）を表示し、「続けて返品・交換」を出す（D-138、proposed・未実装。今の build は「在庫照会へ戻る」で `/stock` を引数なしで開く）。画面の上の戻る導線は「商品と在庫へ」（D-138）。 | 返品・交換は売上帳票ではなく在庫と帳面記録の確認が主目的。レジ戻し済みかどうかを保存後にも読めるようにする。 |
| REQ-202 / list | UI-03-D16 | 画面下部に最近の返品・交換記録を `listReturns(1, 10, null, null)` で表示する。詳細表示・編集・取消・画像再表示は初回実装では扱わない。 | CMD-03 の一覧契約を UI から疎通確認でき、直近記録の保存結果も確認しやすい。詳細/取消は在庫・画像ファイル扱いが絡むため別設計にする。 |
| REQ-202 / cache | UI-03-D17 | 保存成功時の invalidation は [D-052](../decision-log.md) C5 と `src/lib/invalidation-contract.ts` を正本とし、`register_processed=true` では在庫系 consumer を対象外にする。 | レジ戻し済みの返品は在庫を書かないため、条件分岐も含めて mutation 契約で固定する。 |
| REQ-202 / UI-03 | UI-03-D18 | Windows native L3 は owner 目視確認を必須にする。確認対象は navigation、種別切替、レジ戻し済みフラグの意味、商品検索/スキャン相当 Enter 追加、同一商品+direction 数量加算、validation、画像選択/プレビュー、保存結果、recent list、在庫を見る導線（D-138 の後。今の build は在庫照会へ戻る導線）。 | 新規 operator-facing screen であり、レジ戻し済みの二重計上防止説明、連続入力、画像選択、フォーカス戻しは CI だけでは判断しづらい。 |
| REQ-202 / note visibility | UI-03-D19 | 備考は返品・交換の確認優先度が高い項目として、入力時は複数行欄にし、保存結果・recent list・詳細画面では「備考」と分かる独立ラベル付き領域で表示する。本文は `text-foreground` 以上の濃さで、入力なしの場合は「—」を muted 表示する。 | Windows native L3 で、備考が項目立てされず薄い文字で内容を判別しづらいことを確認した。返品理由、交換理由、顧客対応メモは後日の問い合わせ・月末確認で読むため、添付画像や補助説明に埋もれさせない。 |
| REQ-202 / product add suggest | UI-03-D21 | 商品追加欄に live 候補プレビュー（catalog ⑮ ProductAddSuggest / SPEC-SUGGEST-D1〜D11）を追加する。Enter commit 経路（UI-03-D9）とフォーカス戻し（UI-03-D10）は不変。lock source は既存 `isFormLocked` 派生 state、候補確定は既存の複数件候補テーブル選択と同一 handler（現在の種別 / 方向ルールへの反映も既存経路のまま）。 | UI-02-D14 と同一の variant B 判断（純 autocomplete 型はスキャナ退行のため不採用）。契約正本は catalog ⑮。 |
| REQ-202 / REQ-206 / return | UI-03-D22 | 保存結果と recent list の「詳細を見る」は、search state を含む現在の `/inventory/return` URL を `returnTo` として返品・交換詳細へ送る（DSR-18）。 | 詳細確認後に返品・交換作業画面へ戻れるようにする。送信・検証・fallback の横断規範は DSR-18 を正とし、本書では返品・交換画面の producer だけを固定する。 |

## 63.2 Component / Route 構成

```text
src/
  routes/
    inventory/
      return.tsx
  features/
    return-exchange/
      ReturnExchangePage.tsx
      lib/
        return-exchange-request.ts
        return-exchange-row-utils.ts
        receipt-image.ts
      types.ts
```

初回実装では UI-02 / UI-04 と同じく `ReturnExchangePage` に form state、queries、mutation、query invalidation、表示を集約し、DTO 組み立て、validation、画像 base64 変換、row 操作は `lib/` の純関数へ分ける。子 component 分割は重複が増えた時点で別 PR とする。

## 63.3 State Model

| State | 意味 | 主な表示 | 次の action |
|---|---|---|---|
| `editing` | 返品ヘッダ/明細/画像を編集中 | header form、レジ戻し済み説明、商品追加欄、明細表、画像選択 | `add_product` / `update_row` / `select_image` / `submit` |
| `searching_product` | 商品追加欄で検索中 | 商品追加欄 loading | `product_found` / `product_candidates` / `product_not_found` / `search_failed` |
| `saving_image` | 選択画像を `saveReceiptImage` で保存中 | form disabled、spinner + 「画像を保存しています」 | `image_saved` / `image_save_failed` |
| `submitting` | `createReturn` 実行中 | form disabled、spinner + 「返品・交換を記録しています」 | `submit_succeeded` / `submit_failed` |
| `result` | 保存完了 | result panel（保存した商品の一覧と行ごとの在庫を見る）、続けて返品・交換 | `reset_for_next` / navigation |
| `submit_error` | 保存失敗 | Alert + 入力保持 + 再実行 | `submit_same_request` / `edit_as_new_attempt` |

フォーム state は `returnDate`, `returnType`, `registerProcessed`, `note`, `rows`, `receipt`, `savedReceiptPath`, `idempotencyKey` を持つ。`rows` は `productCode`, `productName`, `departmentName`, `stockUnit`, `currentStockQuantity`, `direction`, `quantity` を持つ UI 内部型とし、command payload には `product_code`, `direction`, `quantity` だけを送る。`receipt` は選択中の `File` 情報、preview URL、extension、base64変換状態を持つ。

`submit_error` 後に同内容を再送する場合は `idempotencyKey` と `savedReceiptPath` を保持し、入力・備考・画像を編集して再送する場合は新しい `idempotencyKey` を採番する。

## 63.4 Command / DTO Contract

UI-03 実装 PR では以下を generated binding に出す。

| Command / Type | Existing backend | UI-03 usage |
|---|---|---|
| `commands.createReturn(req)` | Rust command は既存、generated 未対応 | 返品・交換ヘッダ + 明細を保存 |
| `commands.listReturns(page, perPage, dateFrom, dateTo)` | Rust command は既存、generated 未対応 | 最近の返品・交換記録を表示 |
| `commands.saveReceiptImage(request)` | Rust command は既存、generated 未対応 | 選択画像をアプリデータ配下へ保存して相対パスを取得 |
| `ReturnCreateRequest` | BIZ type は既存、`specta::Type` 未対応 | create payload |
| `ReturnItemInput` | BIZ type は既存、`specta::Type` 未対応 | 明細 payload |
| `ReturnCreateResult` | BIZ type は既存、`specta::Type` 未対応 | 保存結果 |
| `ReturnRecordSummary` | DB type は既存、`serde::Serialize` のみ | 最近の返品・交換一覧 |
| `SaveImageRequest` / `SaveImageResponse` | CMD type は既存、`specta::Type` 未対応 | 画像保存 payload / result |

`createReturn` payload:

```text
{
  idempotency_key: string,
  return_type: "return" | "exchange",
  return_date: "YYYY-MM-DD",
  register_processed: boolean,
  receipt_image_path: string | null,
  note: string | null,
  items: [{ product_code: string, direction: "in" | "out", quantity: number }]
}
```

`saveReceiptImage` payload:

```text
{
  image_base64: string,
  extension: string
}
```

`extension` は Rust DTO / generated binding では `string` のままとし、frontend helper が `jpg|jpeg|png|gif|webp` の allowlist を事前 validation する。CMD/IO 側も不正拡張子を validation error として返す。

`listReturns` は `perPage=10` を初期値とし、CMD/BIZ の上限200を UI から超えない。

## 63.5 表示 / 操作

- PageHeader title は `返品・交換`、subtitle はレジ戻し済みなら在庫をこの画面で動かさず、未処理なら在庫へ反映する作業であることを短く示す。
- ヘッダは `返品日`（既定は今日）、`種別`（返品 / 交換）、`レジ戻し済み`（既定 true）、`備考`（任意）を置く。備考は単一行 input ではなく複数行欄にし、返品理由・交換理由・顧客対応メモを 200 文字以内で読めるようにする。
- `レジ戻し済み` true では `CSV取込みで反映`、false では `この保存で反映` の日本語 Badge を各選択肢内に出し、説明文も同じ選択肢内で読めるようにする。色だけで状態を表さない。
- 商品追加欄は `商品コード・JAN・商品名で追加` とし、Enter で検索する。候補が複数ある場合は商品コード、商品名、部門、現在庫を見せる。
- 商品追加欄は入力中に live 候補プレビュー（商品コード・商品名・部門、最大 5 件 + 超過 footer）を表示する（UI-03-D21、挙動契約は catalog ⑮ SPEC-SUGGEST-D1〜D11）。候補は ↓/↑ または click で選択でき、候補未選択の Enter は従来どおり検索を実行する。入力値が変わると候補は即座に閉じる。
- 商品が見つからない場合は `商品登録へ` 導線を出す。未登録商品は商品マスタに登録してから返品・交換へ戻ることを日本語で示す。既存明細・入力（画像選択を含む）がある状態での離脱（`商品登録へ` 導線を含む）は未保存編集の離脱ガード（UI_TECH_STACK §6.11 `useUnsavedChangesWarning`）が破棄確認を出す。
- 明細表は商品コード、商品名、部門、現在庫、方向（戻り / 渡し）、数量（単位を同じ cell に添える）、行削除を表示する。数量と単位を一つの値として読めるよう独立した単位列は置かず、現在庫も単位付き（例: `4 個` / `3 m`、桁区切り付き）にする。長さの商品の現在庫は m で出し、`m` の商品の数量は m で入力する（[共通規則](10-common-rules.md) SPEC-UNIT-D3）。
- 返品種別が `返品` の場合、方向は `戻り` 固定にし、渡し行がある場合は保存前 validation で止める。
- 交換種別では戻り行と渡し行を両方要求する。方向の表示は `戻り（在庫+）` / `渡し（在庫-）` とし、在庫視点を文言で示す。
- レシート画像はドロップゾーン + ファイル選択ボタン + プレビューサムネイルを表示する。画像削除は明示ボタンで行い、確認ダイアログは出さない。
- 明細が 0 件の場合、保存ボタンは disabled にし、理由を「商品が追加されていません」と表示する。
- 保存ボタンは `返品・交換を保存`。保存成功後は result panel へ移り、フォーム本文は操作不可にする。保存結果の「詳細を見る」は UI-03-D22 に従って現在の返品・交換画面 URL を `returnTo` として送る。保存結果や保存系エラーの Alert はページ先頭側に出るため、保存成功または command 失敗時はページ先頭へスクロールする。result panel には備考を独立行で表示し、入力値がない場合も「—」と明示する。
- 最近の返品・交換一覧は日付、種別、レジ戻し済み、備考、記録日時を表示する。備考列は折り返しを許可し、内容がある場合は通常本文色で表示する。「詳細を見る」は UI-03-D22 に従って現在の返品・交換画面 URL を `returnTo` として送る。作成画面内に詳細本文、編集、取消、保存済み画像の再表示は出さない。
- 返品・交換詳細画面では、備考をレシート画像の補足文に混ぜず、ヘッダ情報の下に独立した `備考` 領域として表示する。本文は `whitespace-pre-wrap` で改行を保ち、入力なしの場合は `—` を muted 表示する。

## 63.6 Error / Recovery

- 商品検索失敗: 商品追加欄の近くに Alert を出し、既存明細は保持する。
- 商品 0 件: `商品が見つかりません` と表示し、`商品登録へ` 導線を出す。
- 日付不正、明細 0 件、数量 0 以下、返品種別/方向不正、返品で渡し行あり、交換で戻り/渡し片側のみ: command 呼び出し前に日本語 field error を出す。
- frontend validation error: エラー箇所が明細表や入力欄の近くに出るため、ページ先頭へはスクロールしない。
- 画像形式不正: `jpg / jpeg / png / gif / webp` 以外は保存前に止める。CMD 側 validation error も画像欄の近くに表示する。
- 画像保存失敗: `createReturn` は呼ばず、form state と idempotency key を保持する。同じ画像の再試行では再度 `saveReceiptImage` を試す。
- 画像保存成功後の `createReturn` 失敗: `savedReceiptPath` を保持し、同じ画像の同内容 retry では再度 `saveReceiptImage` を呼ばない。入力・備考・画像を編集したら新しい `idempotencyKey` を採番する。
- BIZ validation error: form state を保持する。入力を修正して再送する場合は新しい `idempotencyKey` を採番し、同じ key で異なる fingerprint を送らない。
- internal error: 入力、`idempotencyKey`、`savedReceiptPath` を保持し、同内容のまま同じ伝票として再試行できるようにする。
- idempotent replay: result panel に「同じ内容の再送として処理済み」と表示し、二重登録ではないことを示す。

## 63.7 Cache / Navigation

- 保存成功時は D-052-C5 の SSOT helper を `register_processed` とともに適用する。具体的な query key 集合は `src/lib/invalidation-contract.ts` だけに置く。
- `navigation.ts` の UI-03 は `to: "/inventory/return"`, `status: "active"` に切り替える。
- 商品と在庫の画面（D-138。旧 在庫照会）の行の詳細から返品・交換へ遷移する場合、将来 `?productCode=...&direction=in` で初期商品追加する余地を残す。ただし初回実装では query 初期追加は非 scope とし、route だけ active にする。
- 明細、header 入力、または receipt 画像選択に差分がある間だけ共通 `useUnsavedChangesWarning` を有効にし、保存中 / result panel 表示中は `!isFormLocked` により解除する。

## 63.8 Non-scope / Follow-up

- 返品・交換作成画面内での記録詳細表示、編集、取消、画像再表示。履歴調査用の詳細 route は `65-inventory-record-traceability.md` §65.10 の横展開 slice で扱う。保存済み画像の asset 表示は別 Design Phase とする。
- 保存済み receipt image の削除 / orphan cleanup。
- `createReturn` と画像保存を単一 command / 擬似TX にまとめる再設計。
- inline 商品登録。
- グローバル barcode scan detection。
- cm / m 表示切替。
- 返品・交換と売上帳票の連動表示。

## 63.9 Test Focus

- UI-03-D1: `/inventory/return` route で page title と navigation active が一致する。
- UI-03-D2: `createReturn` / `listReturns` が generated binding に存在し、ad hoc invoke を使わない。
- UI-03-D3/D20: `saveReceiptImage` が generated binding に存在し、共通 FilePicker（`accept="image/*"`、native dialog + 任意 drop）から base64 payload を作る。
- UI-03-D5: 画像保存成功後の create 失敗 retry で同じ `savedReceiptPath` を使い、画像を再保存しない。
- UI-03-D6/D7: 返品では `in` のみ、交換では `in` / `out` 両方が必要。
- UI-03-D8: `register_processed` true/false の在庫反映説明が日本語 text + Badge で読める。
- UI-03-D9/D10: 商品追加欄 Enter で検索し、1件なら行追加、複数件なら候補選択、0件なら商品登録導線を出す。
- UI-03-D11: 同一 `product_code + direction` の再追加で数量が +1 され、戻り/渡し別行は統合しない。
- UI-03-D12: 数量/日付/種別/方向 validation と長さの商品の m の入力と表示。
- UI-03-D13: 同内容 retry 時に `idempotency_key` を再利用し、成功/リセット/編集再送後は新規 key になる。
- UI-03-D14: 画像保存中 / 返品保存中は戻る/リセット/入力/商品追加/画像選択が disabled になる。
- UI-03-D15: result で record_id、明細数、レジ戻し済み、画像添付有無、warning、idempotent replay が読め、保存成功時にページ先頭へスクロールする。
- UI-03-D16: 最近の返品・交換一覧を取得し、空/取得失敗/成功を表示できる。
- UI-03-D17: `register_processed` の true/false 両経路で実呼出し集合が D-052-C5 の独立 test oracle と完全一致する。
- UI-03-D18: Windows native L3 で連続入力、フォーカス戻し、日本語表示、レジ戻し済み説明、画像選択/プレビュー、保存結果を確認する。
- UI-03-D19: 備考が入力、保存結果、recent list、返品・交換詳細で独立ラベル付き・通常本文色で読める。入力なしの場合は `—` と表示する。
- UI-03-D22: 保存結果と recent list の「詳細を見る」が、現在の返品・交換画面 URL を `returnTo` として送る。

## 63.10 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-09-09 | PR #46 | ⑮ S5 備考「—」への同期。 |
| 2026-08-30 | PR #20 / DSR-18 | UI-03-D22 を追加し、保存結果 / recent list の詳細導線に返品・交換画面への `returnTo` 送信契約を設定。 |
| 2026-08-05 | product add suggest design | UI-03-D21 新設: 商品追加欄の live 候補プレビュー（正本 = catalog ⑮ SPEC-SUGGEST-D1〜D10）。Enter commit 経路・フォーカス戻しは不変。 |
| 2026-08-03 | UI safety net implementation | 返品・交換 form（receipt 画像選択を含む）の dirty 判定を共通離脱ガードへ接続し、保存中 / 保存結果の非 block を実装。 |
| 2026-06-30 | UI-03 note visibility Design Phase | 備考を確認優先度が高い項目として複数行入力、保存結果、recent list、詳細画面で独立ラベル付き・通常本文色で表示する UI-03-D19 を追加。入力なしの場合は「備考なし」と表示する。 |
| 2026-06-28 | UI-03 visibility follow-up | Windows native L3 で備考が項目立てされておらず、文字が薄く内容を判別しづらいことを確認。返品・交換では備考の必要性が高いため、次回改善で独立項目化とコントラストを見直す。 |
| 2026-06-28 | Save result visibility follow-up | 保存成功または command 失敗時はページ先頭へスクロールし、result panel / Alert が画面内に入るようにした。frontend validation は近傍表示のためスクロールしない。 |
| 2026-06-26 | UI-03 L3 feedback | `レジ戻し状況` を inline radio + 単独 Badge から選択肢ごとの説明パネルへ変更し、在庫反映の意味を各選択肢内で読めるようにした。 |
| 2026-06-26 | UI-03 Design Phase | route、generated command 方針、レジ戻し済み分岐、返品/交換明細 validation、画像添付、商品追加/スキャン前提、冪等キー、query invalidation、Windows native L3 を整理。 |
