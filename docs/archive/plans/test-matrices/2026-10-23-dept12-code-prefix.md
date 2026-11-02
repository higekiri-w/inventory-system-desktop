# Test Design Matrix: 部門 12（宅急便）の初期データの `code_prefix` を正本の NULL に合わせる

Plan Packet: [2026-10-23-dept12-code-prefix](../2026-10-23-dept12-code-prefix.md)

## Risk

Risk: R3

## Contracts Under Test

- 新しい DB の `departments` の 21 行が [master-tables](../../../db-design/master-tables.md) §2 の「初期データ（全21部門）」と一致し、部門 12（宅急便）の `code_prefix` が SQL の NULL（空文字でない）。
- BIZ-01 FUNC-4.3 処理ステップ 2（[30 §4.3](../../../function-design/30-biz-product-service.md)）: 宅急便を選び JAN を空にした `create_product` は `ValidationFailed`（「この部門は独自コード発番に対応していません」）で、書込みの前に止まる。
- UI-01b-D4・[51](../../../function-design/51-ui-product-form.md) のエラーハンドリング: NULL の部門は「（独自コード可）」を付けず、JAN が空なら保存しない（既存の test。変えない）。
- MNT-03-D21（[22 §3.3](../../../function-design/22-mnt-migration.md)）: migration を足さない（packet の AC-7 で path を見る。runtime の test は無い）。

## Failure Modes

- F1: 部門 12 が `'TK'` のまま（今の食い違い）。
- F2: 部門 12 を空文字 `''` にする（`code_prefix.ok_or_else` は `None` だけを拒み、`-0001` を発番する）。
- F3: 直すときに他の部門の行（名前・`z005_name`・`code_prefix`・`next_seq`）を変える。
- F4: 宅急便の登録の失敗が `next_seq` を進めるか、商品・在庫の記録を残す。
- F5: test の期待を `schema_v1.rs` から写し、正本との食い違いを見逃す（今の 2 行の標本の test が部門 12 を見ていなかった形）。

## Test Matrix

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| C1 初期データの 21 行（REQ-903） | F1・F2・F3・F5 | integration（`init_database` で実の v1 から最新の版まで） | `test_initial_departments_req903_data`（`src-tauri/src/db/migration.rs`。2 行の標本の期待を、下の「C1 の oracle」の 21 行と `SELECT id, name, z005_name, code_prefix, next_seq FROM departments ORDER BY id` の全部の比較に置き換える） | 部門 12 が `Some("TK")`（M1）か `Some("")`（M2）、または他のどれかの行の値が変わる（M3）。行数が 21 でない |
| C2 宅急便の発番の停止（REQ-101、30 §4.3 の処理ステップ 2） | F1・F2・F4 | integration（BIZ の公開の入口、`setup_test_db` の実の DB） | `test_create_product_req101_takkyubin_rejects_custom_code`（`src-tauri/src/biz/product_service.rs`。`ProductCreateRequest { department_id: 12, ..default_create_request() }`〈JAN は `None`〉で `create_product`） | `Ok`（M1 なら `TK-0001`、M2 なら `-0001`）を返す。`Err` の型が `ValidationFailed` でないか message が「この部門は独自コード発番に対応していません」でない。失敗の後に `products` の行が 0 でない、部門 12 の `next_seq` が 1 でない、`inventory_movements` の行が 0 でない |
| C3 NULL の部門の表示と保存の停止（UI-01b-D4、51） | 表示の code の回帰 | frontend unit・component（既存、変えない） | `src/features/products/ProductFormPage.test.tsx` の「blocks create submit when JAN is blank and department has no code_prefix」、`src/features/products/lib/product-form-request.test.ts` の「allows JAN blank only when selected department has code_prefix」 | NULL の部門の option に「（独自コード可）」が付くか、JAN が空の保存が `createProduct` を呼ぶ（本 lane は表示の code を変えないので、今の green が保たれること） |
| C4 他の部門の発番（BIZ-01 FUNC-4.3） | 直しの回帰 | integration（既存、変えない） | `test_generate_custom_code_req101_normal`（ヘア雑貨 → `HZ-0001`）、`test_generate_custom_code_req101_no_prefix`（毛糸 → `ValidationFailed`） | 部門 2 の `HZ` か部門 3 の NULL が変わる |

既存の test の確かめ（2026-10-23、`eed07d37`）: `rg -n 'fn test_initial_departments_req903_data|fn test_generate_custom_code_req101_normal|fn test_generate_custom_code_req101_no_prefix|fn default_create_request' src-tauri/src` で 4 つとも 1 件ずつ、`rg -n 'blocks create submit when JAN is blank and department has no code_prefix' src/features/products/ProductFormPage.test.tsx` と `rg -n 'allows JAN blank only when selected department has code_prefix' src/features/products/lib/product-form-request.test.ts` で 1 件ずつ。

C1 の oracle（[master-tables](../../../db-design/master-tables.md) §2 の「初期データ（全21部門）」の表から写す。`schema_v1.rs` から写さない。`name` と `z005_name` は 21 行とも同じ文字列、`next_seq` は 21 行とも 1）:

| id | name / z005_name | code_prefix |
|---|---|---|
| 1 | その他小物 | KM |
| 2 | ヘア雑貨 | HZ |
| 3 | 毛糸 | NULL |
| 4 | 手芸等材料 | SY |
| 5 | 他化粧品 | NULL |
| 6 | バッグ | BG |
| 7 | エプロン | AP |
| 8 | 布 | NU |
| 9 | 糸 | NULL |
| 10 | ゴム | GM |
| 11 | 食品 | NULL |
| 12 | 宅急便 | NULL |
| 13 | トワニー | NULL |
| 14 | 雑貨 | ZK |
| 15 | 帽子 | NULL |
| 16 | ビューティ関連 | NULL |
| 17 | 本 | NULL |
| 18 | ボタン | BT |
| 19 | ファスナー | FS |
| 20 | 針 | NULL |
| 21 | ノンリンク | NULL |

oracle の anchor: C2 の message は 30 §4.3 の処理ステップ 2 の文（`rg -c 'この部門は独自コード発番に対応していません' docs/function-design/30-biz-product-service.md` → `1`）。

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 部門 12 の `code_prefix` | DB の無い起動で v1 が NULL を入れる | — | — | — | — | 部門の一覧を読み直しても NULL | 再起動で v1 は当たり直さず NULL のまま。直す前に作った DB は `'TK'` のまま（MNT-03-D21） | — | — | C1 |
| 宅急便の JAN 空の登録 | — | — | — | — | — | — | — | `ValidationFailed`、書込み 0 | JAN を入れて登録し直せば今と同じく登録できる（変えない） | C2 |

Pending・Invalidate・Refetch: 本 lane は query・cache・画面の state を変えない（部門の一覧の取得と invalidation は今のまま）。

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| `code_prefix` が NULL の部門の扱い | `rg -n 'code_prefix' src-tauri/src src --glob '!src/lib/bindings.ts'`: `product_repo.rs`（`Department` と SELECT 2）、`product_service.rs` の `generate_custom_code`、`ProductForm.tsx` の option の label、`product-form-request.ts` の JAN 空の検査、frontend の test の mock | 変える所は無い（値だけが変わる） | 商品 CSV の取込み（`商品コード` は CSV の値で、`code_prefix` を見ない）、Z005 の部門別売上（`code_prefix` を読まない） | C1〜C4 |
| 初期データの値と正本の照合 | v1 の初期データは `departments` と `app_settings` | `departments` を 21 行の比較に | `app_settings` は本 lane の食い違いの外（`test_initial_app_settings_req903_data` が既にある。広げない） | C1 |

## Negative Paths

- missing input: JAN 空 + 宅急便 → C2。
- invalid input: 該当なし（利用者が `code_prefix` を入れる経路は無い）。
- duplicate/ambiguous input: 該当なし。
- unknown reference: 存在しない部門 ID は今の `find_department_by_id` の `None` の分岐（変えない）。
- dependency missing: 該当なし。
- permission/write failure: 該当なし（失敗は書込みの前）。
- dry-run side effect: C2 で `next_seq`・`products`・`inventory_movements` が変わらない。

## Boundary Checks

- threshold: 該当なし。
- null/default: NULL と空文字の区別（M2）。`next_seq` の既定 1。
- empty/non-empty: `code_prefix` の NULL と `''`。
- min/max: 該当なし。
- status/policy enum: 該当なし。
- wire type: `Department.code_prefix: string | null`（変えない）。
- internal type: `Option<String>`。
- producer/consumer: v1 → `product_repo` → BIZ / command → frontend。
- round-trip token: 該当なし。
- precision/range: 該当なし。
- cross-language parse: SQL の NULL → `None` → wire の `null`（specta の生成の型は変わらない）。

## Compatibility Checks

- old schema/input: 直す前に作った DB は `'TK'` のまま開け、発番も今のまま（MNT-03-D21。test を置かない: 振舞いを変えない経路）。
- new schema/input: C1。
- output order: 該当なし。
- optional field behavior: `code_prefix` の NULL は既存の 10 部門と同じ扱い（C4 の毛糸）。

## Data Safety Checks

- source-derived data: 無い。
- generated outputs: `docs/function-design/90-traceability.md`（生成物）。
- secrets: 無い。
- local-only files: 無い。
- synthetic sample boundaries: `tempfile` の一時の DB だけ。

## Main Wiring / Integration Checks

- helper connected to main path: C1・C2 は `init_database`（起動と同じ経路）で作った DB を使う。
- output reaches manifest/report: 該当なし。
- effective config reaches runtime: 該当なし。
- CLI arg reaches implementation: 該当なし。

## Mutation-style Adequacy Questions

mutation は `src-tauri/src/db/schema_v1.rs` の部門の literal だけを変え（production 側）、test を変えない。

- M1（部門 12 を `'TK'` に戻す）: C1（`Some("TK")` と NULL の違い）と C2（`Ok("TK-0001")`）が red。
- M2（部門 12 を `''` にする）: C1（`Some("")` と NULL の違い）と C2（`Ok("-0001")`）が red。
- M3（部門 2 の `'HZ'` を NULL にする）: C1 が red（C4 の `test_generate_custom_code_req101_normal` も red）。
- If a mock value is changed so it differs from the design-doc expected value: C1・C2 は mock を使わず実の v1 を当てる。C1 の期待は master-tables から写す（F5）。
- If invalidate/refetch changes the value: 該当なし（cache を変えない）。
- If a key branch is inverted: `generate_custom_code` の NULL の分岐を反転すると C2 と C4 の `no_prefix` が red（本体は変えないので、今の test が守る）。
- If a threshold comparison changes: 該当なし。
- If a guard is removed: 同上（C2）。
- If an output field is omitted: C1 は 5 列を比べる。
- If output order changes: C1 は `ORDER BY id` で比べる。
- If dry-run performs a side effect: C2 の書込み 0 の確かめ。
- If a JSON number crosses JavaScript safe integer range: 該当なし。
- If a state token is round-tripped through browser/client code: 該当なし。

## Residual Test Gaps

- 直す前に作った DB が `'TK'` のまま残ることは test で固めない（MNT-03-D21 の範囲の文が持つ。固める test は古い値を守る test になり、作り直しの後は意味が無い）。
- Windows の実機の表示の目視は置かない（packet の「owner の判断事項」の既定。C3 が表示の code を守る）。
