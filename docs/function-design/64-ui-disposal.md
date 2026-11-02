# 64. UI-05: 廃棄・破損

> **計数の置き場と在庫の求め直し（D-118。(a) と b1 で実装済み）**: この画面の日付の欄の意味は「品が動いた日」（廃棄は品を棚から外した日）。在庫が計数に入っているかは、この日付で ADR D3 の窓（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）を決める（[31](31-biz-inventory-service.md)）。(a) で入ったのは、この日付を業務日として movement に書くこと（`biz/inventory_service/common.rs` の `validate_business_date`）と、計数と入出庫の窓の計算（`biz/stock_ledger.rs` の `judge` → `from_count` → `derive`）。計数を保存する公開の経路と精算の窓は D-118 の b1 で入った（入出庫の保存は要再確認の一覧を再取得する。UI_TECH_STACK の D-118 の段落）。この画面の欄の見出し・説明の文言は b1 で変えていない。

> 対応仕様: REQ-204 / UI-05
>
> 入力ドキュメント: `docs/architecture/ui-task-specs.md` UI-05、`docs/architecture/cmd-task-specs.md` CMD-05、`docs/architecture/biz-task-specs.md` BIZ-02、`docs/function-design/21-io-inventory-repo.md`、`docs/function-design/31-biz-inventory-service.md`、`docs/function-design/44-cmd-inventory.md`、`docs/SCREEN_DESIGN.md`、`docs/UI_TECH_STACK.md` §5.3

廃棄・破損は、販売や返品ではない理由で商品在庫を減らし、ロス原価と理由を帳面に残す operator-facing flow である。UI は廃棄日、商品明細、種別、数量、原価、理由を扱い、在庫減算、廃棄記録、在庫変動履歴、操作ログの永続化は BIZ-02 / CMD-05 に委譲する。

## 64.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: UI-05 は frontend page / hooks / component 群として、廃棄日と明細行を `commands.createDisposal(req)` に渡し、成功時に廃棄一覧と在庫系 query を無効化して保存結果を表示する。

**シグネチャ**: frontend の境界は `commands.createDisposal(req)`、`commands.listDisposals(page, perPage, dateFrom, dateTo)`、商品追加用の `commands.searchProducts(query)` である。React component の詳細 props は §64.2、command / DTO の詳細は §64.4 に定義する。

**処理ステップ**: 画面全体の状態遷移は §64.3、表示・操作は §64.5、エラー復旧は §64.6 に定義する。

## 64.1 設計判断

| Spec | Decision ID | 決定 | 理由 / 不採用案 |
|---|---|---|---|
| REQ-204 / UI-05 | UI-05-D1 | route は `/inventory/disposal` とし、file route は `src/routes/inventory/disposal.tsx`、画面本体は `src/features/disposal/DisposalPage.tsx` に置く。 | `docs/function-design/52-ui-shared-layout.md` の route 表に合わせ、入出庫エリアの独立作業として UI-02/03/04 と prefix をそろえる。 |
| REQ-204 / CMD-05 | UI-05-D2 | UI は generated `commands.createDisposal(req)` / `commands.listDisposals(page, perPage, dateFrom, dateTo)` のみを使う。実装 PR では既存 Rust command に `#[specta::specta]` を付け、`DisposalCreateRequest` / `DisposalItemInput` / `DisposalCreateResult` / `DisposalRecordSummary` を generated binding に出す。 | `disposal_cmd` は runtime `generate_handler!` には存在するが、現状 `collect_commands!` / `bindings.ts` にはない。`typedInvoke` fallback は退役済みなので ad hoc invoke は採用しない。 |
| REQ-204 / type | UI-05-D3 | 明細ごとに種別 `"disposal"`（廃棄）/ `"damage"`（破損）/ `"other"`（その他）を選ぶ。既定は `"damage"`、UI 表示は日本語のみとする。 | 店頭でよく起きる作業は破損・汚損のロス記録と考えられる。1伝票内に廃棄と破損が混在する可能性があるため、種別はヘッダではなく行単位にする。 |
| REQ-204 / reason | UI-05-D4 | 理由は明細ごとに必須入力とする。 | backend の `create_disposal` は空理由を拒否する。理由はあとでロス原因を確認する主要情報なので、ヘッダ備考にはまとめない。 |
| REQ-204 / product add | UI-05-D5 | 商品追加欄は商品コード / JAN / 商品名を同じ入力で扱い、Enter で `commands.searchProducts` を実行する。1件なら明細へ追加、複数件なら候補リストから選ぶ、0件なら商品登録への導線を出す。 | UI-02/03/04 と同じ scan-like flow にして学習コストを下げる。未登録商品作成は `/products/new` へ逃がし、廃棄画面内で master mutation をしない。 |
| REQ-204 / scanner | UI-05-D6 | バーコードスキャナは HID キーボードとして商品追加欄へ入力される前提にする。初回実装はグローバルスキャン検知を置かず、フォーカス中の入力欄 + Enter で追加し、追加後は商品追加欄へフォーカスを戻す。 | UI-02/03/04 と同じ方針。グローバル検知は全画面 UX への影響が大きく別 Design Phase。 |
| REQ-204 / rows | UI-05-D7 | 明細の一意性は `product_code + disposal_type + reason` とし、同じ組み合わせを再追加した場合は数量を +1 する。 | 同じ商品でも「破損」と「期限切れ」など理由が異なるロスは別明細として残す必要がある。product_code だけで統合すると理由の粒度が失われる。 |
| REQ-204 / validation | UI-05-D8 | 数量は `> 0`（個数の商品は整数、`m` の商品は m で小数 2 桁まで受けて cm に直す。上限は `9999999`）、原価は小数 2 桁までの円 `>= 0`（上限は `999999.99`。送る値は 1/100 円の `cost_price_centi`。`決定（owner、D-113 J2）`、SPEC-UNIT-D5。error の文は `原価は0以上で、小数は2桁までで入力してください`、欄は `inputMode="decimal"`・`step="0.01"`）、廃棄日は必須、種別は `"disposal"` / `"damage"` / `"other"`、理由は空白不可として command 前に frontend で止める。BIZ-02 は同じ validation を最終防御として持つ。保存の後の結果の欄のロス原価合計は TS の twin で TD-023 の丸め（原価は 1/100 円のまま `BigInt` で掛け、原価の形式の誤りと上限越えの行は 0 として数える。共通規則 (c)）、保存した記録は BIZ が返す値（[共通規則](10-common-rules.md) SPEC-UNIT-D3・D5・D6・D9）。候補の一覧の原価は SPEC-UNIT-D5 の formatter（`¥83.33`）。 | operator-facing form では保存前に日本語 field error を出す。generated command は UI 以外からも呼べるため、業務不変条件は BIZ に置く。 |
| REQ-204 / idempotency | UI-05-D9 | `idempotency_key` は画面で1保存試行単位に生成し、保存失敗後の同内容再試行では同じ key を再利用する。保存成功、フォームリセット、別伝票開始、または保存失敗後に fingerprint 対象項目を編集して再送する場合は新しい key にする。 | UI-02/03/04 と同じ二重登録防止方針。同じ key のまま異なる fingerprint を送ると BIZ が conflict にするため、編集再送は新規保存試行として扱う。 |
| REQ-204 / submit | UI-05-D10 | 保存中はヘッダ、明細、商品追加、戻る/リセット導線を disabled にし、中断可能に見せない。 | backend は単一 TX。UI だけで cancel 可能に見せると在庫反映状態を誤認させる。 |
| REQ-204 / result | UI-05-D11 | 保存成功後は record_id、登録明細数、ロス原価合計、stock_warnings、`idempotent_replay` の有無と、保存した商品の一覧（商品コード・商品名と、行ごとの「在庫を見る」。[61](61-ui-receiving.md) UI-02-D10 と同じ形で、`/products?q=<商品コード>&selected=<商品コード>` を開く）を表示し、「続けて廃棄・破損」「詳細を見る」の導線を出す（D-138、proposed・未実装。今の build は「在庫照会へ戻る」で `/stock` を引数なしで開く）。画面の上の戻る導線は「商品と在庫へ」（D-138）。 | 廃棄・破損は売上帳票ではなく在庫とロス把握が主目的。保存後にロス原価合計を読めるようにする。 |
| REQ-204 / list | UI-05-D12 | 画面下部に最近の廃棄・破損記録を `listDisposals(1, 10, null, null)` で表示する。詳細表示・編集・取消は初回実装では扱わない。 | CMD-05 の一覧契約を UI から疎通確認でき、直近記録の保存結果も確認しやすい。詳細/取消は在庫復元が絡むため別設計にする。 |
| REQ-204 / cache | UI-05-D13 | 保存成功時の invalidation は [D-052](../decision-log.md) C7 と `src/lib/invalidation-contract.ts` を正本とする。 | 廃棄で変わる在庫・履歴 consumer を一貫して stale 化し、画面側の key 列挙を廃止する。 |
| REQ-204 / UI-05 | UI-05-D14 | Windows native L3 は owner 目視確認を必須にする。確認対象は navigation、商品検索/スキャン相当 Enter 追加、同一商品+種別+理由の数量加算、種別/理由/原価 validation、保存中 disable、保存結果、recent list、在庫を見る導線（D-138 の後。今の build は在庫照会へ戻る導線）。 | 新規 operator-facing screen であり、ロス理由の入力、連続入力、フォーカス戻し、保存結果の可読性は CI だけでは判断しづらい。 |
| REQ-204 / async result gate | UI-05-D15 | 非同期商品検索結果の採否に使う form lock ref は render 中に同期せず、保存eventの先頭でlock、validation / command失敗eventとreset eventでunlock、保存成功後はlock維持とする。商品候補の選択eventと検索完了callbackはこのrefを読む。 | render-phase ref更新は未commit renderのlockを残し得る。effect同期だけでは保存eventからcommit/effectまでに検索結果が解決するraceを閉じないためevent境界を採用する。保存clickからpending renderまでの極小窓で従来は結果が反映され得たのに対し、本方式は保存開始時点から破棄する。この可視差はUI-05-D10の「保存中は商品追加をlock」に合わせる是正だが、Plan Gateのowner裁定対象とする。 |
| REQ-204 / product add suggest | UI-05-D16 | 商品追加欄に live 候補プレビュー（catalog ⑮ ProductAddSuggest / SPEC-SUGGEST-D1〜D11）を追加する。Enter commit 経路（UI-05-D5）とフォーカス戻し（UI-05-D6）は不変。lock source は UI-05-D15 の lock ref（suggest 層は同 ref を `isLocked()` として読み、保存 event 内で `invalidateAndClose()` を同期呼出しする。新規 lock は作らない）。候補確定は既存の複数件候補テーブル選択と同一 handler。 | UI-02-D14 と同一の variant B 判断 + 本画面固有の async lock 整合（catalog ⑮ D4/D10）。契約正本は catalog ⑮。 |
| REQ-204 / REQ-206 / return | UI-05-D17 | 保存結果と recent list の「詳細を見る」は、search state を含む現在の `/inventory/disposal` URL を `returnTo` として廃棄・破損詳細へ送る（DSR-18）。 | 詳細確認後に廃棄・破損作業画面へ戻れるようにする。保存直後の確認導線を入庫・返品・交換と対称にする（owner L3 所感 2026-08-31 / 2026-09-11）。廃棄だけ保存結果に link を出さない業務理由は source docs に無い。送信・検証・fallback の横断規範は DSR-18 を正とし、本書では廃棄・破損画面の producer だけを固定する。 |

## 64.2 Component / Route 構成

```text
src/
  routes/
    inventory/
      disposal.tsx
  features/
    disposal/
      DisposalPage.tsx
      lib/
        disposal-request.ts
        disposal-row-utils.ts
      types.ts
```

初回実装では UI-03 / UI-04 と同じく `DisposalPage` に form state、queries、mutation、query invalidation、表示を集約し、DTO 組み立てと validation は `lib/` の純関数へ分ける。子 component 分割は重複が増えた時点で別 PR とする。

## 64.3 State Model

| State | 意味 | 主な表示 | 次の action |
|---|---|---|---|
| `editing` | 廃棄ヘッダ/明細を編集中 | header form、商品追加欄、明細表、最近の廃棄・破損一覧 | `add_product` / `update_row` / `submit` |
| `searching_product` | 商品追加欄で検索中 | 商品追加欄 loading | `product_found` / `product_candidates` / `product_not_found` / `search_failed` |
| `submitting` | `createDisposal` 実行中 | form disabled、spinner + 「廃棄・破損を記録しています」 | `submit_succeeded` / `submit_failed` |
| `result` | 保存完了 | result panel（保存した商品の一覧と行ごとの在庫を見る）、続けて廃棄・破損、詳細を見る | `reset_for_next` / navigation |
| `submit_error` | 保存失敗 | Alert + 入力保持 + 再実行 | `submit_same_request` / `edit_as_new_attempt` |

フォーム state は `disposalDate`, `rows`, `idempotencyKey` を持つ。`rows` は `productCode`, `productName`, `departmentName`, `stockUnit`, `currentStockQuantity`, `disposalType`, `quantity`, `costPrice`, `reason` を持つ UI 内部型とし、command payload には `product_code`, `disposal_type`, `quantity`, `cost_price_centi`, `reason` を送る。

`submit_error` 後に同内容を再送する場合は `idempotencyKey` を保持し、入力を編集して再送する場合は新しい `idempotencyKey` を採番する。

商品検索結果の async gate は UI-05-D15 に従う。`submit` event は mutation 呼出しより先に
gate を lock し、validation / command failure では入力を保持したまま unlock する。
success は result state と同じく lock を維持し、`reset_for_next` で unlock する。

## 64.4 Command / DTO Contract

UI-05 実装 PR では以下を generated binding に出す。

| Command / Type | Existing backend | UI-05 usage |
|---|---|---|
| `commands.createDisposal(req)` | Rust command は既存、generated 未対応 | 廃棄ヘッダ + 明細を保存 |
| `commands.listDisposals(page, perPage, dateFrom, dateTo)` | Rust command は既存、generated 未対応 | 最近の廃棄・破損記録を表示 |
| `DisposalCreateRequest` | BIZ type は既存、`specta::Type` 未対応 | create payload |
| `DisposalItemInput` | BIZ type は既存、`specta::Type` 未対応 | 明細 payload |
| `DisposalCreateResult` | BIZ type は既存、`specta::Type` 未対応 | 保存結果 |
| `DisposalRecordSummary` | DB type は既存、`serde::Serialize` のみ | 最近の廃棄・破損一覧 |

`createDisposal` payload:

```text
{
  idempotency_key: string,
  disposal_date: "YYYY-MM-DD",
  items: [
    {
      product_code: string,
      disposal_type: "disposal" | "damage" | "other",
      quantity: number,
      cost_price_centi: number,
      reason: string
    }
  ]
}
```

`listDisposals` は `perPage=10` を初期値とし、CMD/BIZ の上限200を UI から超えない。

## 64.5 表示 / 操作

- PageHeader title は `廃棄・破損`、subtitle は販売ではない理由で在庫を減らし、ロス理由と原価を残す作業であることを短く示す。
- ヘッダは `廃棄日`（既定は今日）のみを置く。理由は明細単位に置く。
- 商品追加欄は `商品コード・JAN・商品名で追加` とし、Enter で検索する。候補が複数ある場合は商品コード、商品名、部門、現在庫、原価を見せる。
- 商品追加欄は入力中に live 候補プレビュー（商品コード・商品名・部門、最大 5 件 + 超過 footer）を表示する（UI-05-D16、挙動契約は catalog ⑮ SPEC-SUGGEST-D1〜D11）。候補は ↓/↑ または click で選択でき、候補未選択の Enter は従来どおり検索を実行する。入力値が変わると候補は即座に閉じる。
- 商品が見つからない場合は `商品登録へ` 導線を出す。未登録商品は商品マスタに登録してから廃棄・破損へ戻ることを日本語で示す。既存明細・入力がある状態での離脱（`商品登録へ` 導線を含む）は未保存編集の離脱ガード（UI_TECH_STACK §6.11 `useUnsavedChangesWarning`）が破棄確認を出す。
- 明細表は商品コード、商品名、部門、現在庫、種別、数量（単位を同じ cell に添える）、原価（同じ cell に「円」を添える）、理由、行削除を表示する。数量と単位を一つの値として読めるよう独立した単位列は置かず、現在庫も単位付き（例: `4 個` / `3 m`、桁区切り付き）にする。長さの商品の現在庫は m で出し、`m` の商品の数量は m で入力する（[共通規則](10-common-rules.md) SPEC-UNIT-D3）。
- 種別は日本語ラベルで `廃棄` / `破損` / `その他` と表示する。wire value の英語は画面に出さない。
- 理由欄は明細ごとに入力し、空白なら保存前 validation で止める。
- 明細が 0 件の場合、保存ボタンは disabled にし、理由を「商品が追加されていません」と表示する。
- 保存ボタンは `廃棄・破損を保存`。保存成功後は result panel へ移り、フォーム本文は操作不可にする。保存結果や保存系エラーの Alert はページ先頭側に出るため、保存成功または command 失敗時はページ先頭へスクロールする。保存結果の「詳細を見る」は UI-05-D17 に従って現在の廃棄・破損画面 URL を `returnTo` として送る。
- 最近の廃棄・破損一覧は日付、記録ID、記録日時を表示する。「詳細を見る」は UI-05-D17 に従って現在の廃棄・破損画面 URL を `returnTo` として送る。作成画面内に詳細本文、編集、取消は出さない。

## 64.6 Error / Recovery

- 商品検索失敗: 商品追加欄の近くに Alert を出し、既存明細は保持する。
- 商品 0 件: `商品が見つかりません` と表示し、`商品登録へ` 導線を出す。
- 日付不正、明細 0 件、数量 0 以下、原価負数、種別不正、理由空白: command 呼び出し前に日本語 field error を出す。
- frontend validation error: エラー箇所が明細表や入力欄の近くに出るため、ページ先頭へはスクロールしない。
- BIZ validation error: form state を保持する。入力を修正して再送する場合は新しい `idempotencyKey` を採番し、同じ key で異なる fingerprint を送らない。
- internal error: 入力と `idempotencyKey` を保持し、同内容のまま同じ伝票として再試行できるようにする。入力を編集して再送する場合は新しい `idempotencyKey` を採番する。
- idempotent replay: result panel に「同じ内容の再送として処理済み」と表示し、二重登録ではないことを示す。
- 保存eventと競合して完了した商品検索: 保存event先頭で同期したlockを確認して候補・message・行を反映しない。保存失敗後はunlockし、operatorが同じ検索を再実行できる。

## 64.7 Cache / Navigation

- 保存成功時は D-052-C7 の SSOT helper を適用する。具体的な query key 集合は `src/lib/invalidation-contract.ts` だけに置く。
- `navigation.ts` の UI-05 は `to: "/inventory/disposal"`, `status: "active"` に切り替える。
- 商品と在庫の画面（D-138。旧 在庫照会）の行の詳細から廃棄・破損へ遷移する場合、将来 `?productCode=...` で初期商品追加する余地を残す。ただし初回実装では query 初期追加は非 scope とし、route だけ active にする。
- 明細または廃棄日の初期値との差分がある間だけ共通 `useUnsavedChangesWarning` を有効にし、保存中 / result panel 表示中は `!isFormLocked` により解除する（画像条件は持たない）。

## 64.8 Non-scope / Follow-up

- 廃棄・破損記録の詳細表示、編集、取消。
- 廃棄取消に伴う在庫復元。
- inline 商品登録。
- global barcode scan detection。
- cm / m 表示切替。
- ロス集計レポート、部門別ロス分析。
- 画像添付。

## 64.9 Test Focus

- UI-05-D1: `/inventory/disposal` route で page title と navigation active が一致する。
- UI-05-D2: `createDisposal` / `listDisposals` が generated binding に存在し、ad hoc invoke を使わない。
- UI-05-D3/D4: 種別 enum を日本語表示し、payload では `"disposal"` / `"damage"` / `"other"` を送る。理由空白は保存前に止める。
- UI-05-D5/D6: 商品追加欄 Enter で検索し、1件なら行追加、複数件なら候補選択、0件なら商品登録導線を出す。
- UI-05-D7: 同一 `product_code + disposal_type + reason` の再追加で数量が +1 され、種別または理由が違う場合は別行にする。
- UI-05-D8: 数量/原価（小数 2 桁の円・1/100 円の送信・BigInt の合計）/種別/理由 validation と長さの商品の m の入力と表示。
- UI-05-D9: 同内容 retry 時に `idempotency_key` を再利用し、成功/リセット/編集再送後は新規 key になる。
- UI-05-D10: submitting 中は戻る/リセット/入力/商品追加が disabled になる。
- UI-05-D11: result で record_id、明細数、ロス原価合計、warning、idempotent replay が読め、保存成功時にページ先頭へスクロールする。
- UI-05-D12: 最近の廃棄・破損一覧を取得し、空/取得失敗/成功を表示できる。
- UI-05-D13: 保存成功時の実呼出し集合が D-052-C7 の独立 test oracle と完全一致する。
- UI-05-D14: Windows native L3 で連続入力、フォーカス戻し、日本語表示、種別/理由/原価 validation、保存結果を確認する。
- UI-05-D15: 保存eventがpending renderより先にasync gateをlockし、同eventと競合して完了した検索結果を反映しない。validation / command失敗とresetではunlockする。
- UI-05-D17: 保存結果と recent list の「詳細を見る」が現在の廃棄・破損画面 URL を `returnTo` として送る。

## 64.10 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-09-16 | ㉕ | UI-05-D17 を改訂し、保存結果にも詳細 link（`returnTo` 送信）を追加。入庫・返品・交換と対称化。 |
| 2026-09-16 | ㉕ closeout | UI-05-D11 の導線列挙と state 表に「詳細を見る」を追加、§64.5 の二重記述を整理 |
| 2026-08-30 | PR #20 / DSR-18 | UI-05-D17 を追加し、recent list の詳細導線に廃棄・破損画面への `returnTo` 送信契約を設定。保存結果は詳細 link なしの現行契約を維持。 |
| 2026-08-05 | product add suggest design | UI-05-D16 新設: 商品追加欄の live 候補プレビュー（正本 = catalog ⑮ SPEC-SUGGEST-D1〜D10、lock は UI-05-D15 ref と単一 source）。Enter commit 経路・フォーカス戻しは不変。 |
| 2026-08-03 | UI safety net implementation | 廃棄・破損 form の dirty 判定を共通離脱ガードへ接続し、保存中 / 保存結果の非 block を実装。 |
| 2026-07-23 | UI-05-D15 | form lock ref をrender同期から保存/失敗/reset event同期へ移し、非同期商品検索結果の採否タイミングを固定。 |
| 2026-06-28 | Save result visibility follow-up | 保存成功または command 失敗時はページ先頭へスクロールし、result panel / Alert が画面内に入るようにした。frontend validation は近傍表示のためスクロールしない。 |
| 2026-06-27 | UI-05 Design Phase | route、generated command 方針、商品追加/スキャン前提、明細単位の種別/理由、validation、冪等キー、query invalidation、recent list、Windows native L3 を整理。 |
