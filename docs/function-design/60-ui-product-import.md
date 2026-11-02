# 60. UI-01c: 商品一括インポート

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D4・D8）の商品一括インポートの側。(a) で入ったのは下の 3 つ目（新規行の初期在庫の movement と写しの求め直し。`biz/product_service.rs` の `recompute_stock` の呼出し）。1・2 つ目（共有 JAN と連動を始める行の拒否、確定の TX で拒まれたときの表示）は b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)。画面・reducer・wire は変えず、BIZ の拒否と画面の test を足した）。拒否は [30](30-biz-product-service.md) の新契約。旧 ADR（時点証拠）の `ej_unverified` の間の拒否と既定値 false、非連動化の警告と未調整の回復先は持たない（置き換えた）。

- 共有 JAN で在庫連動にできない行（別の商品と JAN を共有する行の在庫連動、在庫連動の商品と JAN を共有する行。同じ CSV の先の新規の行と JAN が重なる新規の行を含む）と、在庫連動を始める行で連動しない間の売上がある既存商品の行は、preview で既存の行単位の validation と同じく error 行にして message を示し、他の正常行の取込みは続ける（error 行が他の行の取込みを止めない）。利用者の再同意だけで検査を外さず、`pos_stock_sync` を自動で変えない。
- preview の後に登録や売上が変わったか、上書きを選んだ行どうしが当たって、確定の TX で行が拒まれたとき（[30](30-biz-product-service.md) の D-118 の節。確定全体が戻り、何も取り込まれない）は、今の commit の失敗と同じく Alert に BIZ の文（行を名指し、「その行をスキップにするか、ファイルを選び直してもう一度読み込んでください」）を出し、「戻る」で preview と上書きの選択に戻る（kind `validation` は preview へ戻る今の分岐。画面・reducer・wire は変えない）。利用者はその場で行をスキップにするか、ファイルを選び直す（b1 の Plan Review round 2 の是正）。
- 新規行の初期在庫は初期在庫の movement として入り、写しは求め直しで決まる（30 の契約。(a) で実装済み。表示は今のまま）。

> 対応仕様: REQ-104 / UI-01c
>
> 入力ドキュメント: `docs/architecture/ui-task-specs.md` UI-01c、`docs/SCREEN_DESIGN.md` 一括インポート画面、`docs/UI_TECH_STACK.md` §6.5.4、`docs/function-design/26-io-product-csv-importer.md`、`docs/function-design/30-biz-product-service.md` `preview_import` / `commit_import`、`docs/function-design/40-cmd-product.md`、`docs/function-design/42-cmd-sales-stocktake.md` §22.6

商品一括インポートは、初期導入や大きな商品マスタ更新時に CSV から商品をまとめて登録・更新する operator-facing flow である。通常の個別登録 UI-01b と同じ BIZ-01 / CMD-01 契約を使い、UI はファイル選択、プレビュー、重複行の扱い、確定結果の確認を担当する。CSV のパース・業務バリデーション・DB 書込みは UI に置かない。

## 60.0 関数要求 / シグネチャ / 処理ステップの扱い

**関数要求**: UI-01c は frontend page / reducer / component 群として、商品マスタ CSV の file bytes を generated CMD へ渡し、preview 結果を表示し、利用者が選んだ import target だけを commit する。

**シグネチャ**: frontend の境界は `commands.previewImport(fileBytes)` と `commands.commitImport(validRows, overwriteCodes)` である（proposed・未実装、D-122: JAN の別名の runtime の lane で `commands.commitImport(validRows, overwriteCodes, oldJanAliases)` になる。UI-01c-D17）。React component の詳細 props は §60.2 で定義し、Rust command / DTO の詳細は §60.4 で定義する。

**処理ステップ**: 画面全体の状態遷移は §60.3、利用者操作と表示分岐は §60.5、エラー復旧は §60.6 に定義する。

## 60.1 設計判断

| Spec | Decision ID | 決定 | 理由 / 不採用案 |
|---|---|---|---|
| REQ-104 / UI-01c | UI-01c-D1 | route は `/products/import` とし、file route は `src/routes/products/import.tsx`、画面本体は `src/features/products/ProductImportPage.tsx` に置く。 | 商品管理エリア内の独立作業であり、商品一覧や商品フォームの query param で mode を切り替えると preview / commit state が混ざる。 |
| REQ-104 / CMD-01 | UI-01c-D2 | UI は generated `commands.previewImport(fileBytes)` / `commands.commitImport(validRows, overwriteCodes)` のみを使う。実装 PR では既存 Rust command に `#[specta::specta]` を付け、`ImportRow` / `ImportErrorRow` / `ImportDuplicateRow` / `ImportPreview` / `ProductImportResult` を generated binding に出す。既存 `product_service::ImportResult` は `ProductImportResult` に rename して、売上 CSV 取込みの `ImportResult` と TS 名を衝突させない。 | Rust 側 command は存在するが、現状 `src/lib/bindings.ts` に商品一括インポート用 command がない。`typedInvoke` fallback は退役済みなので、ad hoc invoke は採用しない。 |
| REQ-104 / UI-01c | UI-01c-D3 | ファイル選択は UI-07 と同じ plain `<input type="file" accept=".csv,.txt">` + drag/drop で始める。`@tauri-apps/plugin-dialog` は導入しない。Windows native では HTML5 drag/drop を frontend に届かせるため、Tauri main window の `dragDropEnabled` を `false` にする。 | plugin-dialog は npm / cargo 依存、Tauri 登録、capability 追加を伴う。UI-01c 単独で導入すると scope が増える。Tauri 既定の file-drop event は Windows で HTML drop event を奪うため、今回の dropzone 実装とは相性が悪い。将来のネイティブダイアログ統合は UI_TECH_STACK §6.5.4 の再検討対象に残す。 |
| REQ-104 / UI-01c | UI-01c-D4 | 画面状態は `idle` / `previewing` / `preview` / `committing` / `result` / `error` の discriminated union + reducer にする。preview 結果は画面 state に保持し、server cache は作らない。 | BIZ-01 設計が「プレビュー結果はフロントエンド側で保持し、commit 時に送信」としている。Zustand / XState は cancel/resume/並行状態がない初回実装では過剰。 |
| REQ-104 / UI-01c | UI-01c-D5 | プレビューは 3 系統を同時に見せる: 新規登録可能行、エラー行、重複行。画面上部に件数サマリを置き、新規登録可能行と重複行の詳細テーブルは先頭 50 行までを既定表示にする（エラー行の詳細テーブルは UI-01c-D18 で全件）。 | 商品マスタは数千行規模になり得る。全行を無制限に描画すると重く、利用者の判断に必要な先頭確認と件数確認を優先する。 |
| REQ-104 / duplicate | UI-01c-D6 | 重複行は既定で「スキップ」にし、行ごとに「上書き」を選べる。全重複をまとめて上書きする操作は初回実装では置かない。 | 上書きは既存商品情報を変える操作で影響が大きい。大量一括上書きは誤操作時の被害が大きいため、初回は行単位で明示選択させる。 |
| REQ-104 / duplicate | UI-01c-D7 | `上書き` を 1 件以上選んで「取り込む」を押した場合は確認ダイアログを出す。全件新規または重複全スキップなら確認なしで commit する。 | 上書きは破壊的に近い状態変更で、DSR-07 の確認境界に該当する。新規登録だけなら通常の保存操作であり、不要な確認は挟まない。 |
| REQ-104 / UI-01c | UI-01c-D8 | `error_rows.length > 0` でも commit は許可する。ただし取り込む対象は `valid_rows` + 上書き選択済み duplicate のみで、エラー行は取り込まない。 | IO/BIZ はエラー行と正常行を同じ preview に返す。正常行まで止めると初期投入作業が進まない。エラー行の除外は件数と文言で明示する。 |
| REQ-104 / UI-01c | UI-01c-D9 | `valid_rows.length === 0` かつ上書き選択 0 件の場合、「取り込む」ボタンは disabled にし、理由を日本語で表示する。 | 操作可能に見えて何も起きない状態は避ける。disabled だけにせず、何が不足しているかをテキストで伝える。 |
| REQ-104 / products cache | UI-01c-D10 | commit 成功後の invalidation は [D-052](../decision-log.md) C3 と `src/lib/invalidation-contract.ts` を正本とし、結果画面から `/products` へ戻る導線を出す。 | 商品 master・在庫・PLU・棚卸し consumer を一貫して stale 化し、画面側の key 列挙を廃止する。 |
| REQ-104 / UI-01c | UI-01c-D11 | 完了画面は `created_count` / `updated_count` / `skipped_count` をサマリカードまたは数値帯で示し、「商品と在庫へ戻る」「別のCSVを取り込む」の 2 動線に絞る（画面名は D-138 の後〈proposed・未実装〉。今の build の完了の link は「商品一覧を確認」〈`src/features/products/ProductImportPage.tsx:97`〉、見出しの戻る〈`:26`〉は「商品一覧へ戻る」）。 | 初期投入作業では連続取込もあり得る。完了後の次の一手を明示しつつ、余分な action を増やさない。 |
| REQ-104 / UI-01c | UI-01c-D12 | 画面を離れても DB 書込み途中の cancel/resume は提供しない。`committing` 中は画面内操作を disabled にし、commit 完了後の結果で確認する。 | backend commit は単一 TX。途中 cancel は設計されておらず、UI だけでキャンセル可能に見せると誤認を招く。 |
| REQ-104 / UI-01c | UI-01c-D13 | Windows native L3 は owner 目視確認を必須にする。確認対象は FilePicker（native dialog + 任意 drop、D-054/UI-01c-D14）、エラー行と重複行の区別、上書き確認、日本語文言、結果サマリ、商品と在庫の画面への戻り導線（D-138 の後。今の build は商品一覧）。 | 新規 operator-facing screen であり、CSV 作業は初期導入時に失敗影響が大きい。CI / unit test だけでは視認性と操作の分かりやすさを判断できない。 |
| REQ-104 / UI-01c | UI-01c-D14 | ファイル選択を共通 FilePicker（native dialog + 任意 drop、D-054）へ移行し、UI-01c-D3 の plain input 方式を supersede する。`dragDropEnabled: false` は drop 維持のため据え置き。 | D3 の却下理由（plugin-dialog 未導入・scope 増）は foundation PR（2026-06-26）と日報 PR #125 の path-based 実証で解消済み。WebView2 白画面バグの再発面を picker 一箇所へ集約する。 |
| REQ-104 / limit | UI-01c-D15 | 商品 import は CMD 早期拒否 + BIZ（`preview_import`）安全網の**二重**で `CSV_IMPORT_FILE_SIZE_LIMIT`（constants.rs）超過を validation error で拒否する（D-054）。 | UI だけが拒否する未仕様上限は直接 IPC で bypass 可能。売上 / 日報が持つ CMD 早期拒否 + BIZ 安全網の既存二重防御パターン（ARCHITECTURE resource-safety 例外）と同型に揃える。 |
| REQ-104 / SPEC-PLS-D6 | UI-01c-D16 | 任意列 `PLU対象` を preview に表示し、`1` / `0` / 空欄の意味を説明する。`1` だが JAN が 13 桁でない行（空か 8 桁。形の誤った JAN は BIZ-01-D10 で行の error になり、warning には届かない）は warning + 実適用値 `対象外` を同じ行に表示する。 | CSV を commit する前に operator が自動補正と更新対象を確認できる。warning を色だけで表さない。 |
| REQ-104 / D-122 | UI-01c-D17 | （proposed・未実装、JAN の別名の runtime の lane）重複行の `old_jan_question` が `Some(前の JAN)` で、その行に `上書き` を選んだときだけ、同じ行に「前の番号（{前の JAN}）のバーコードが付いた現物は店にありますか」と `ある` / `ない` の二択を出す（既定 `ない`。radio と文字で示し、色だけで示さない）。送る集合は [30](30-biz-product-service.md) §4.9.2 と同じ 1 式 `{ (r.import_row.product_code, r.old_jan_question) | r ∈ duplicate_rows, r.old_jan_question が Some, r に「上書き」を選んだ, r の答えが「ある」 }` で、`oldJanAliases`（`{ product_code, old_jan }` の配列。`old_jan` は `old_jan_question` の値をそのまま入れ、UI で書き換えない）として commit に送る。`スキップ` に戻した行の答えは送らない（選択は保持してよい）。問いを出してよい行は BIZ の `old_jan_question` だけで決め、UI が見るのはその行自身の `上書き` の選択だけで、**ほかの**重複行の上書きの選択から UI が問いの出し入れを判定し直さない（判定の規則を UI に写さない。ほかの行の選択に依る条件は commit の判定表の J6 が判定する。30 §4.9.2）。`ImportRow.warnings` は新規登録候補の行と重複行の両方で、行の中に text + icon で出す（別名の warning を含む。[30](30-biz-product-service.md) BIZ-01-D9）。あわせて warning を持つ行（新規登録候補の行と重複行）は、詳細テーブルの先頭 50 行の制限（UI-01c-D5）と別に、一覧の上に件数と、行ごとの行の番号（`line_no`）と warning の文を全件まとめて出す（warning を持つ行が無ければまとめを出さない）。commit の判定表の J5 は行の `warnings` にある別名の warning を利用者に見せた証拠として扱うので、51 件目以降の行の warning も画面に出す。UI は warning の文を解析せず種類で分けないので、UI-01c-D16 の JAN の不備の warning も同じまとめに入る。commit の失敗は §60.6 の commit の失敗と同じく preview・答え・上書きの選択を保ったまま Alert に BIZ の message を出し、次の操作を [30](30-biz-product-service.md) §4.9.2 の判定表の「回復」の列のとおり error の field で分けて示す（message の文と回復は判定表だけが持ち、UI は message を解析しない）: field が `old_jan_aliases`（判定表の J6）→ 名指された P の答えを `ない` に直すか Q の行を `スキップ` にして、preview をやり直さずにもう一度「取り込む」（ファイルの選び直しを求めない）。field なし → ファイルを選び直して preview し直す導線。 | owner の決定（repo 外の回答台帳 TD-284・TD-286）。入力の誤りの訂正（前の番号の現物は無い）とメーカーの番号の変更（前のバーコードの在庫が残る）を分け、前の番号で売れた分を同じ商品へたどる。既定を `ない` にして黙って誤った商品へ付けない。問いを上書きを選んだ行にだけ出すのは、上書きしない行は JAN が変わらないため。答えを wire の別の一覧にするのは、`overwriteCodes` と同じ「選んだ行の product_code」の形にそろえるため（`ImportRow` に field を足すと新規行にも意味の無い欄ができる）。問いで見せた前の JAN を組で返すのは、preview の後に JAN が別の値へ変わったとき、問いで見せていない番号を別名にしないため（commit が DB の今の JAN と照らす）。 |
| REQ-104 / D-140 | UI-01c-D18 | エラー行の詳細テーブルは UI-01c-D5 の先頭 50 行の制限に掛けず、全件を行の番号（`line_no`）と内容（`errors` を ` / ` で結んだ BIZ の文）で出す。表の上に 1 文 `エラー行は取り込みません。内容の文のとおりに CSV を直し、「ファイルを選び直す」で読み込み直してください` を出す。UI は文を解析せず、種類で分けず、色だけで示さない。新規登録候補と重複行の表の 50 行の制限は変えない。 | 商品 CSV の取込みは、黙って入る形の行を行の error で名指して止める（[30](30-biz-product-service.md) BIZ-01-D10、owner の決定 repo 外の回答台帳 TD-421・TD-426）。初期投入は数千行で、51 行目より後のエラー行が「他 N 件」に隠れると、どの行を直すかがその画面で分からない（D-127 の使い手に押しつけない）。エラー行は CSV を直すための一覧なので全件が要り、正常な行は件数と先頭の確認で足りる。却下: 50 行と「他 N 件」のまま（直す行を名指せない）、ページ送り（操作が増え、全体を見渡せない）、エラー行の一覧の file の書出し（新しい保存の経路と FilePicker の保存の追加が要る。画面の全件で足りる）。全行がエラーの数千行の file でも 1 つの表で描く（行は文字だけで、行ごとの操作を持たない）。 |

## 60.2 Component / Route 構成

```text
src/
  routes/
    products/
      import.tsx
  features/
    products/
      ProductImportPage.tsx
      import/
        product-import-reducer.ts
        product-import-types.ts
        product-import-summary.ts
        ProductImportDropzone.tsx
        ProductImportPreview.tsx
        ProductImportResult.tsx
        OverwriteRowsDialog.tsx
```

`products/import.tsx` は route と title を管理し、画面本体は `ProductImportPage` に委譲する。`ProductImportPage` は reducer、mutation、query invalidation、子コンポーネントへの props 配線を担当する。`ProductImportPreview` は preview 結果の表示だけを担い、上書き対象の選択 state は reducer に集約する。

## 60.3 State Machine

| State | 意味 | 主な表示 | 次の action |
|---|---|---|---|
| `idle` | ファイル未選択 | ドロップゾーン、CSV列の説明、注意文 | `file_selected` |
| `previewing` | CSV を backend で preview 中 | spinner + 「CSVを確認しています」 | `preview_succeeded` / `preview_failed` |
| `preview` | preview 表示中 | 件数サマリ、エラー行、重複行、取込ボタン | `toggle_overwrite` / `commit_requested` / `reset` |
| `committing` | commit 実行中 | 操作 disabled、spinner + 「取り込んでいます」 | `commit_succeeded` / `commit_failed` |
| `result` | commit 完了 | 新規/更新/スキップ件数、次の導線 | `reset` / `/products` へ遷移 |
| `error` | ファイル全体エラーまたは commit 失敗 | Alert + 再試行 / ファイル選択へ戻る | `reset` / `retry_preview` |

`preview` state は `preview: ImportPreview` と `overwriteCodes: string[]` を持つ。`overwriteCodes` は重複行の `product_code` だけを含める。（proposed・未実装、D-122）前の JAN の問いに `ある` と答えた重複行の `product_code` の集合 `oldJanYesCodes: Set<string>` を持つ（既定は空 = 全て `ない`。UI-01c-D17）。`commit` payload は以下で組み立てる。

```text
overwriteCodeSet = new Set(overwriteCodes)
validRows = [
  ...preview.valid_rows,
  ...preview.duplicate_rows
    .filter(row => overwriteCodeSet.has(row.import_row.product_code))
    .map(row => row.import_row)
]
overwriteCodes = Array.from(overwriteCodeSet)
oldJanAliases = preview.duplicate_rows   // proposed・未実装、D-122（30 §4.9.2 と同じ 1 式。配列化の前の Set で filter する）
  .filter(row => row.old_jan_question !== null
    && overwriteCodeSet.has(row.import_row.product_code)
    && oldJanYesCodes.has(row.import_row.product_code))
  .map(row => ({ product_code: row.import_row.product_code, old_jan: row.old_jan_question }))
```

## 60.4 Command / DTO Contract

UI-01c 実装 PR では以下を generated binding に出す。

| Command / Type | Existing backend | UI-01c usage |
|---|---|---|
| `commands.previewImport(fileBytes)` | Rust command は既存、generated 未対応 | file bytes を preview し、行分類を表示 |
| `commands.commitImport(validRows, overwriteCodes)` | Rust command は既存、generated 未対応 | 新規行 + 上書き選択済み重複行を commit。（proposed・未実装、D-122）3 つ目の引数 `oldJanAliases`（`{ product_code, old_jan }` の配列）に前の JAN の問いの `ある` の行を載せる（[42](42-cmd-sales-stocktake.md) §22.6） |
| `ImportRow` | Rust type は既存、`specta::Type` 未対応 | commit payload、preview table |
| `ImportErrorRow` | Rust type は既存、`specta::Type` 未対応 | error table |
| `ImportDuplicateRow` | Rust type は既存、`specta::Type` 未対応 | duplicate table、overwrite 選択。（proposed・未実装、D-122）`old_jan_question: string \| null` で前の JAN の問いを出す行を示す（UI-01c-D17） |
| `ImportPreview` | Rust type は既存、`specta::Type` 未対応 | preview state |
| `ProductImportResult` | 現 Rust 名 `ImportResult` を実装 PR で rename | generated では sales CSV import の `ImportResult` と衝突しない名前にする |

`ProductImportResult` は `created_count`, `updated_count`, `skipped_count` を持つ。既存の CSV 売上取込み `ImportResult` と同じ TypeScript 名にしない。実装 PR では `src-tauri/src/biz/product_service.rs` の商品一括インポート結果型を `ImportResult` から `ProductImportResult` に rename し、`product_cmd::commit_import` の戻り値も `product_service::ProductImportResult` にする。specta export 名だけの上書き案は、Rust 内で `ImportResult` が複数意味を持ち続けるため採用しない。

## 60.5 表示 / 操作

- PageHeader title は `一括インポート`、subtitle は初期投入・大量更新用の短い説明に留める。
- ページ説明（① 説明セクション使用パターン、[02-component-catalog.md](../design-system/02-component-catalog.md) ①）: CSVファイルから複数の商品をまとめて登録・更新するページです。ファイルを選ぶと新規登録候補・既存商品との重複・エラー行の3つに分けて内容を確認でき、重複行は初期状態でスキップされるので上書きする行だけ個別に選んで取り込みます。取り込みを実行すると、新規登録・上書き更新・スキップの件数が画面に表示されます。
- `idle` では必要列を明示する: `商品コード`, `商品名`, `部門ID`, `売価`, `原価`, `税率`。任意列は `在庫単位`, `初期在庫`, `JANコード`, `メーカー品番`, `取引先ID`, `POS在庫連動`, `PLU対象`。
- CSV テンプレート出力は初回 UI-01c 実装では非 scope。必要列の説明を画面内に置く。
- `preview` では上部に件数サマリを置く: 新規登録候補、エラー行、重複行、上書き選択数。
- valid rows の `PLU対象` 列は `対象` / `対象外` / `既定（13桁JANなら対象）` の 3 値を表示する。JAN 不備 warning は `JAN が13桁でないため対象外として取り込みます` を text + icon で同行表示し、空欄には既定ルールを適用する旨を表の前に説明する（UI-01c-D16）。
- エラー行は行番号 + エラーメッセージを日本語で表示する。色だけで示さない。エラー行は先頭 50 行の制限に掛けず全件を出し、表の上に `エラー行は取り込みません。内容の文のとおりに CSV を直し、「ファイルを選び直す」で読み込み直してください` を出す（UI-01c-D18）。
- 重複行は行番号 + 商品コード + 商品名 + `スキップ / 上書き` の二択で示す。既定はスキップ。
- 取込ボタンの label は、上書き選択がある場合 `選択した内容で取り込む`、新規のみの場合 `取り込む` とする。
- `committing` 中は取込ボタン、ファイル選択、上書き切替を disabled にする。戻る / reset は commit 完了後に出す。
- `result` では `created_count`, `updated_count`, `skipped_count` を数値で示し、`商品と在庫へ戻る` と `別のCSVを取り込む` を出す（UI-01c-D11。今の build は `商品一覧を確認`）。

## 60.6 Error / Recovery

- ファイル空: CMD validation error を Alert に表示し、ファイル選択へ戻れる。
- 文字コード判別不能 / 必須列不足: file-level error として Alert に表示し、別ファイルを選べる。
- 行エラー: preview 内で取り込まれない行として表示する（全件。UI-01c-D18）。正常行があれば commit は可能。直した CSV は「ファイルを選び直す」でその場で読み込み直せる。
- 全行エラーまたは取り込む対象 0 件: commit button は disabled。理由を「取り込める行がありません」と表示する。
- commit internal error: preview state を保持し、再度 commit できる状態へ戻す。ファイル再選択を強制しない。
- 上書き確認 cancel: preview state と overwrite selection を保持する。

## 60.7 Cache / Navigation

- commit 成功時は D-052-C3 の SSOT helper を適用する。具体的な query key 集合は `src/lib/invalidation-contract.ts` だけに置く。
- `/products` への戻りは既定検索条件でよい。UI-01c は一覧検索条件の `returnTo` を持たない。
- `navigation.ts` の UI-01c は `to: "/products/import"`, `status: "active"` に切り替える。

## 60.8 Non-scope / Follow-up

- CSV テンプレートのダウンロード。
- preview 結果の server-side token / cache 化。
- CSV 列マッピング UI。
- 全重複行の一括上書き。
- import 中断 / cancel / resume。
- import 履歴画面。

## 60.9 Test Focus

- UI-01c-D1: `/products/import` route で page title と navigation active が一致する。
- UI-01c-D2: `previewImport` / `commitImport` が generated binding に存在し、ad hoc invoke を使わない。
- UI-01c-D14: FilePicker（native dialog + 任意 drop）から bytes を `previewImport` に渡せる。
- UI-01c-D4: reducer が `idle -> previewing -> preview -> committing -> result` と error / reset を正しく遷移する。
- UI-01c-D5: preview 件数サマリ、エラー行、重複行が日本語ラベルで読める。
- UI-01c-D6/D7: 重複行は既定スキップ、上書き選択時のみ確認ダイアログが出る。
- UI-01c-D8/D9: エラー行があっても正常行 commit は可能、対象 0 件では commit disabled + 理由表示。
- UI-01c-D10: commit 成功時の実呼出し集合が D-052-C3 の独立 test oracle と完全一致する。
- UI-01c-D11: result で `created_count` / `updated_count` / `skipped_count` と次の導線が表示される。
- UI-01c-D12: committing 中は操作が disabled になり、cancel 可能に見せない。
- UI-01c-D13: Windows native L3 で FilePicker（native dialog + 任意 drop）、エラー・重複の見分け、上書き確認、結果導線を確認する。
- UI-01c-D16: `PLU対象` の 1 / 0 / 空欄と、13 桁でない JAN（空か 8 桁）の warning + 対象外補正が preview で読める。
- UI-01c-D18: 51 件目以降のエラー行も行の番号と BIZ の文（原因と直し方）で出て、表の上に取り込まない旨と直し方の文が出る。
- UI-01c-D17（proposed・未実装、D-122）: `old_jan_question` のある重複行で `上書き` を選んだときだけ問いが出て既定が `ない`、`ある` かつ `上書き` の行だけが `oldJanAliases` に入り、`old_jan` が `old_jan_question` の値と同じで、`スキップ` に戻した行は入らない。field `old_jan_aliases` の error の後は preview・答え・選択が残り、答えを `ない` に直すとファイルを選び直さずに確定し直せる。ほかの失敗の message では preview し直す導線が出る。warning が重複行にも出る。新規登録候補の 51 件目の行の warning も、一覧の上のまとめに行の番号と文で出る。

## 60.10 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-06-25 | UI-01c Design Phase | route、generated command 方針、file input 例外、reducer state、preview/duplicate/commit UX、query invalidation、Windows native L3 を整理。 |
| 2026-10-17 | JAN の別名の design lane（D-122） | UI-01c-D17（前の JAN の問い、`oldJanAliases`、重複行の warning）を追加。実装は JAN の別名の runtime の lane。 |
| 2026-10-27 | 商品 CSV のプレビューの検査の lane（D-140） | UI-01c-D18（エラー行の全件表示と直し方の文）を追加し、UI-01c-D16 の warning の対象を BIZ-01-D10 の後の形に合わせた。 |
