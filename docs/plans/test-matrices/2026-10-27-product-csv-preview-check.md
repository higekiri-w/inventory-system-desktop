# Test Design Matrix: 商品 CSV のプレビューで黙って入る形の行を名指しで止める

Plan Packet: [2026-10-27-product-csv-preview-check](../2026-10-27-product-csv-preview-check.md)

## Risk

Risk: R3

## Contracts Under Test

- 30 BIZ-01-D10 の F1〜F4（判定・文・行の errors の中の順と排他・「通るもの」）
- 30 §4.8 手順 4（重複の分類より前。上書きの候補の行にも当たる）
- 30 §4.9 手順 0（TX の前の再検証。最初に当たった行の最初の文。DB を変えない）
- 30 BIZ-01-D1（手入力の create の判定と 2 文は変えない。validator の共有）、BIZ-01-D2（`should_default_plu_target` の互換は変えない）、§4.8 手順 4 の `PLU対象=1` の warning（空か 8 桁の JAN）
- 60 UI-01c-D18（エラー行の全件と表の上の 1 文）
- D-140

oracle は設計正本の文（BIZ-01-D10 の表の文、UI-01c-D18 の 1 文）と BIZ-01-D1 の golden（`4901234567887`・`4901234567890`・`49123456`・`49123457`・`96385074`）から写し、`product_service.rs` の literal から写さない。

## Failure Modes

- FM1: 指数の形の `商品コード` か `JANコード` が正常な行・重複行になる（誤ったコードのまま変えられない／JAN が引き当たらない）。
- FM2: 13 桁の数字の `商品コード` で `JANコード` が空の行が入る（Z004 が引き当たらない）。
- FM3: 7 桁（先頭の 0 の落ち）・12 桁・全角・チェックディジットの誤りの `JANコード` が入る。
- FM4: 検査が新規の行だけに当たり、上書きの候補の行（13 桁の数字の商品コードの JAN を消す行を含む）を通す。
- FM5: preview を通らない wire の行を commit が書く。または commit が途中の行まで書いてから止まる。
- FM6: 今通る形（共有 JAN、先頭 2 の店内コード、8 桁の正しい JAN、JAN の無い独自コード、12・14 桁の数字の商品コード、13 桁の数字でない商品の JAN を消す上書き）を止める。
- FM7: 手入力の create の JAN の文・判定が CSV の文に変わる。
- FM8: 51 件目以降のエラー行が画面に出ない。
- FM9: 文が BIZ-01-D10 と違う（直し方の欠け、列の名の取り違え、値の写し漏れ）。

## Test Matrix

- 引く既存 test は `rg` で在ることを確かめた（2026-10-27、`50f97a76`）: `test_create_product_req101_rejects_invalid_jan_wire_with_exact_message`・`test_create_product_req101_accepts_valid_jan_8_and_jan_13`・`test_should_default_plu_target_req907_preserves_invalid_check_digit_compatibility`・`test_preview_import_req907_warns_and_disables_invalid_jan_plu_target`・`test_commit_import_req907_blank_plu_target_derives_for_new_and_preserves_overwrite`・`test_commit_import_req104_derives_plu_target_like_backfill_and_keeps_on_overwrite`（round 1 の是正で足した、`048dfcbb`）・`test_preview_import_req104_product_code_length_limit`・`test_commit_import_req104_rejects_overlong_product_code_without_writes`・`test_import_req104_shared_jan_row_error_both_directions_and_commit_recheck`（`src-tauri/src/biz/product_service.rs`）、`req101_validates_ean_13_golden_profile_and_rejects_wrong_check_digit` ほか 2 本（`src-tauri/src/biz/jan_code.rs`）。
- T1〜T6 は実の `parse_product_csv` を通る `preview_import` / `commit_import` を `init_database` の DB（`setup_test_db`）で呼び、mock しない。CSV は test module の `make_csv`（UTF-8）で作る。
- 文の比較は `errors` の要素の完全一致（`contains` にしない）。

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| F1（2 列、`E`/`e`、小数点の有無） | FM1・FM9 | integration | T1 `test_preview_import_req104_rejects_exponent_form_codes` | 行 `4.90123E+12`（商品コード、JAN は正しい 13 桁）・`5E+12`（同じ）・JAN `4.90123e+12`（商品コードは独自）・両方 `4.9E+12` の行が error_rows に無いか、errors が `商品コードが指数の形です: '4.90123E+12'。Excel が桁を落として書いた値です。元の Excel でその列を文字列の値にして（TEXT の列）保存し直してください。セルを選んだときの上の欄（数式バー）に E+ が出るなら、元の資料から全部の桁を入れ直してください` 等の文と違うか、両方の行で 2 つ（商品コード → JANコード の順）でないとき。同じ CSV の正しい行 1 つが valid_rows に無いとき |
| F2（13 桁ちょうど、列が無いときも） | FM2・FM9 | integration | T2 `test_preview_import_req104_rejects_13_digit_code_without_jan` | `JANコード` の列がある CSV で商品コード `4901234567887`・JAN 空の行と、`JANコード` の列が無い CSV の同じ行が、errors `商品コードが13桁の数字でJANコードが空です: '4901234567887'。JANを商品コードにした商品は、JANコードにも同じ値を入れてください` の 1 つの error 行にならないとき |
| F3・F4（JAN の判定の共有と CSV の文） | FM3・FM7・FM9 | integration | T3 `test_preview_import_req104_validates_jan_shape_and_check_digit` | 独自コードの行で JAN `4912345`（7 桁）・`490123456788`（12 桁）・`４９０１２３４５６７８８７`（全角）が F3 の文、`4901234567890`・`49123457` が F4 の文の 1 つの error 行にならないとき。同じ CSV の `4901234567887`・`49123456` の行が valid_rows に無いとき |
| §4.8 手順 4（上書きの候補）、BIZ-01-D8 の上書きの段（13 桁の数字でない商品の JAN を消す上書き） | FM4・FM6 | integration | T4 `test_preview_import_req104_shape_check_applies_to_overwrite_candidates` | 既存の商品（商品コード `4901234567887`・JAN 同じ、`create_product` で作る）と既存の独自コードの商品、JAN を持つ既存の独自コードの商品（JAN `49123456`。`create_product` は JAN を商品コードにするので、`commit_import` の新規の行か test の DB への直の挿入で作る）に対し、CSV の上書きの候補の行 (a) `4901234567887`・JAN 空、(b) 独自コード・JAN `4901234567890`、(c) `4901234567887`・JAN 同じ、(d) JAN を持つ独自コードの商品の商品コード・JAN 空 を確かめる。(a) と (c) は同じ商品コードなので別の CSV（別の `preview_import` の呼出し）にする（同じ CSV だと後の行に CSV の中の商品コードの重複の error が付く）。(a)・(b) が error_rows でなく duplicate_rows に出るか、errors が F2・F4 の文の 1 つでないとき。(c) が duplicate_rows に無いとき。(d) が errors の無い行として duplicate_rows に無いか、その行を `overwrite_codes` に入れた `commit_import` の後に商品の `jan_code` が空（`None`）でないとき（13 桁の数字でない商品の JAN を消す上書きは意図した契約。BIZ-01-D8 の上書きの段） |
| §4.9 手順 0 | FM5・FM9 | integration | T5 `test_commit_import_req104_revalidates_code_shape_before_tx` | 正しい新規の行 1 つの後に F1〜F4 の行を 1 つずつ置いた 4 回の `commit_import`（`overwrite_codes` 空）が、`ValidationFailed` で F1〜F4 の文（その行の最初の文）を返さないか、`products` と `inventory_movements` の行の数が呼ぶ前と違うとき（正しい行も入っていない）。上書きの行（既存の独自コードの商品、JAN `4901234567890`、`overwrite_codes` にその商品）でも同じ |
| BIZ-01-D10 の「通るもの」 | FM6 | integration | T6 `test_preview_import_req104_keeps_accepted_code_forms` | 独自コードで JAN 空、`4901234567887` に同じ JAN、`4901234567887` に違う正しい JAN `4901234567894`（合成。チェックディジットが合うことを起票の時点に計算で確かめた）、独自コードに `49123456`・`96385074`、先頭 2 の店内コード `2000000000039`、同じ JAN を持つ独自コードの 2 行（`POS在庫連動` 0）、`490123456788`（12 桁）と `49012345678870`（14 桁）の商品コードで JAN 空の各行のどれかが valid_rows に無いとき。`4901234567887` に同じ JAN の行と違う正しい JAN の行は同じ商品コードなので、別の CSV（別の `preview_import` の呼出し）で確かめる（同じ CSV だと後の行に CSV の中の商品コードの重複の error が付く） |
| §4.8 手順 4 の PLU の warning（置き換わった部分の直し） | FM6・FM9 | integration | T7 `test_preview_import_req907_warns_and_disables_plu_target_without_13_digit_jan`（今の `test_preview_import_req907_warns_and_disables_invalid_jan_plu_target` を改名して直す） | `PLU-C2-NONE`（JAN 空）と `PLU-C2-SHORT`（JAN `49123456`）が valid_rows で warning `JAN が13桁でないため対象外として取り込みます` と `plu_target = Some(false)` でないとき。`PLU-C2-CHECK`（`2000000000038`）が error_rows で F4 の文でないとき |
| BIZ-01-D2 と commit の導出（置き換わった部分の直し） | FM5 | integration | T8 `test_commit_import_req907_blank_plu_target_derives_for_new_and_preserves_overwrite`（行 `PLU-C4-INVALID-CHECK` とその assert を外す） | 新規の `2000000000039` の行が `plu_target` true にならないか、上書きの空欄が今の値を保たないとき。外した行の互換は `test_should_default_plu_target_req907_preserves_invalid_check_digit_compatibility` が持つ |
| §4.9 手順 0 の F3 と BIZ-01-D2 の commit の導出（置き換わった部分の直し） | FM3・FM5 | integration | T10 `test_commit_import_req104_derives_plu_target_like_backfill_and_keeps_on_overwrite`（名前は変えない。Packet の S1） | 12 桁 `123456789012` と英字の `49012345678A4` の行の、それぞれ別の `commit_import` の呼出しが `ValidationFailed` で F3 の文を返さないか、`products` の行の数が呼ぶ前と違うとき。新規の `4901234567894` が `plu_target` true、JAN 空と正しい 8 桁 `96385074` が false にならないとき。上書きの行の JAN を正しい 8 桁 `49123456` にした `commit_import` が成功しないか、上書きが今の `plu_target`（true）を保たないとき |
| UI-01c-D18 | FM8 | component | T9 `ProductImportPreview.test.tsx` の `REQ-104 UI-01c-D18: エラー行は 50 行の制限に掛けず全件と取り込まない旨を出す` | error_rows 60 件（`line_no` 2〜61、各行の errors に行ごとに違う文）の preview で、`line_no` 61 の行の文が表に無いか、エラー行の card の中（`within`）に `他 ` で始まる段落が出るか、`エラー行は取り込みません。内容の文のとおりに CSV を直し、「ファイルを選び直す」で読み込み直してください` が無いとき。同じ preview の valid_rows 60 件（`line_no` 2〜61）の新規登録候補の表が 50 行でない（新規登録候補の card の中〈`within`〉に `line_no` 52 以降の行がある）とき（UI-01c-D5 を変えない。新規登録候補の表には今も「他 N 件」の段落が無い） |
| BIZ-01-D1 の create（変えない） | FM7 | integration | 既存 `test_create_product_req101_rejects_invalid_jan_wire_with_exact_message`・`test_create_product_req101_accepts_valid_jan_8_and_jan_13` | create の文が CSV の文に変わるか、validator の判定が変わったとき |
| 共有 JAN（変えない、順） | FM6 | integration | 既存 `test_import_req104_shared_jan_row_error_both_directions_and_commit_recheck` | 形の検査が共有 JAN の行を止めるか、共有 JAN の検査の順・文が変わったとき |

oracle の anchor（実装の run で `rg -c` を file ごとに測る。BIZ-01-D10 の 4 つの文の固有の句は、`docs/function-design/30-biz-product-service.md` に 1 件ずつ、`src-tauri/src/biz/product_service.rs` の production の code に 1 件ずつ、test に期待として 1 件以上）: `が指数の形です: '`、`商品コードが13桁の数字でJANコードが空です`、`JANコードが8桁か13桁の数字ではありません`、`JANコードのチェックディジットが合いません`。UI-01c-D18 の 1 文は `docs/function-design/60-ui-product-import.md` に 2 件（D18 の行と §60.5）、`ProductImportPreview.tsx` に 1 件。

## State Lifecycle Matrix

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 形の誤った行（preview） | CSV に行がある | `previewing`（BIZ の判定） | — | — | — | 「ファイルを選び直す」で同じ行が同じ文で出る（判定は行の値だけ） | 起動し直しても同じ（DB を読まない） | `preview` の error_rows に全件 | 直した CSV を選び直すと valid_rows に移る | T1〜T4・T9、L3-1 |
| 正しい行（エラー行と同じ CSV） | 同じ | 同じ | `committing` → `result` で入る | commit の成功の後の invalidation は今のまま（UI-01c-D10） | — | 同じ CSV を選び直すと重複行（既定のスキップ） | — | — | — | 既存の UI-01c-D8・D10 の test、L3-2 |
| wire の行（preview を通らない） | — | `commit_import` の手順 0 | — | — | — | — | — | `ValidationFailed`、DB は変わらない | preview からやり直す（UI はエラー行を送らない） | T5 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| JAN の形の判定（`biz::jan_code::validate`） | `rg -n 'jan_code::validate' src-tauri/src`（2026-10-27: `product_service.rs` の `is_plu_target_eligible_jan` と `validate_create_request` の 2 か所） | 商品 CSV の preview と commit（新しい関数の 1 か所から） | `is_plu_target_eligible_jan`（変えない。Packet の「不要になるもの」）、Z004 の取込み（商品を作らない。D-140）、`io/plu_formatter` の adapter の検証（BIZ-01-D1 で共有しない） | T3・T5 |
| wire の行を信頼しない commit の再検証（BIZ-01-D5・BIZ-01-D8） | `commit_import` の冒頭（D5）、`import_row_commit_error`（売価・原価・単位・初期在庫）、`import_row_link_error`（共有 JAN） | 冒頭の D5 の直後（TX の前） | `import_row_commit_error`（TX の中で行ごと、既存の商品の単位を読む）に入れない: 形の判定は DB を読まず、TX の前で全行を止めて何も書かない D5 と同じ形にするため | T5 |
| 50 行の表示の制限（`MAX_VISIBLE_ROWS`） | `ProductImportPreview.tsx` の valid_rows・duplicate_rows・error_rows の 3 か所 | error_rows だけ外す | valid_rows・duplicate_rows（UI-01c-D5 を変えない。D17 の warning のまとめは別の lane） | T9 |
| エラー行の表示（他の取込みの画面） | `src/features/csv-import/components/ErrorRowsTable.tsx`（売上の取込み） | なし | 売上の取込みは別の画面と別の契約（REQ-401）で、本 lane の範囲の外 | — |

## Negative Paths

- missing input: `JANコード` の列が無い CSV（T2）。
- invalid input: 指数の形（T1）、7・12 桁・全角の JAN（T3）、チェックディジットの誤り（T3）。
- duplicate/ambiguous input: 上書きの候補（T4）、同じ JAN の 2 行（T6、共有 JAN の検査は既存の test）。
- unknown reference: 該当なし（形の判定は参照を引かない）。
- dependency missing: 該当なし。
- permission/write failure: commit の再検証は TX の前で書かない（T5）。
- dry-run side effect: preview は DB を変えない（今の契約。T1〜T4 は DB の行の数を見ない。T5 が commit の側を見る）。

## Boundary Checks

- threshold: 13 桁ちょうど（12・14 は通る、T2・T6）。JAN は 8・13 桁（7・12 は F3、T3）。
- null/default: `JANコード` の空と列なし（T2）。
- empty/non-empty: エラー行 0 件で card を出さない（既存）。60 件（T9）。
- min/max: 該当なし（数に変えない）。
- status/policy enum: `JanCodeValidationError` の 2 つの variant → F3・F4（T3）。
- wire type: `ImportErrorRow.errors: string[]`（変えない。bindings の差分なしを AC-9 で見る）。
- internal type: `String` のまま判定（数に変えない）。
- producer/consumer: BIZ の文 → UI は解析せずに描く（T9）。
- round-trip token: 該当なし。
- precision/range: 数への変換が無い（Packet の Contract Audit）。
- cross-language parse: 該当なし（UI は文を解析しない）。

## Compatibility Checks

- old schema/input: 今まで通った F1〜F4 の行は error（D-140 の Compatibility）。そのほかの行は今のまま（T6、既存の import の test 全体）。
- new schema/input: 列・encoding は変えない。
- output order: error_rows の並びは今のまま（IO の parse_errors の後に行の順。共有 JAN の error の行は後ろ）。行の errors の中の順は BIZ-01-D10（T1）。
- optional field behavior: `JANコード` の列なし = 空（T2）。

## Data Safety Checks

- source-derived data: 無い（店の CSV・Excel の値を使わない）。
- generated outputs: `90-traceability.md`（再生成）。
- secrets: 無い。
- local-only files: L3 の fixture A・B（repo に入れない）。
- synthetic sample boundaries: BIZ-01-D1 の golden と合成の値だけ。

## Main Wiring / Integration Checks

- helper connected to main path: 新しい関数を `preview_import` と `commit_import` の両方が呼ぶ（T1〜T4 と T5。M1・M3 で片方を外すと red）。
- output reaches manifest/report: preview の error_rows が画面の表に全件出る（T9、L3-1・L3-3）。
- effective config reaches runtime: 該当なし。
- CLI arg reaches implementation: 該当なし。

## Mutation-style Adequacy Questions

production 側だけに注入し、戻して green を確かめる。結果は PR の Validation に書く。

| # | 注入 | red になる test |
|---|---|---|
| M1 | preview の行の検証から新しい関数の呼出しを外す | T1・T2・T3・T4・T7 |
| M2 | 新しい関数の呼出しを重複の分類の後の新規の行（valid_rows）だけに移す | T4 |
| M3 | commit の手順 0 から新しい関数の呼出しを外す | T5・T10 |
| M4 | F2 の条件を「13 桁以上」にする | T6（14 桁の行） |
| M5 | 指数の形の判定で小数点を必須にする | T1（`5E+12`） |
| M6 | F3・F4 の判定を長さと文字だけにする（チェックディジットを見ない） | T3（`4901234567890`・`49123457`）・T4 (b)・T7 |
| M7 | F4 の文を F3 の文に取り違える（variant の写しを入れ替える） | T3・T7・T10 |
| M8 | エラー行の表を `slice(0, MAX_VISIBLE_ROWS)` に戻す | T9 |
| M9 | 上書きの候補の行（同じ商品コードの商品が DB にある行）にだけ、F2 を「13 桁の数字の `商品コード`」の条件なしで当てる（`JANコード` が空の上書きの行を全部 error にする。新規の行は今のまま） | T4 (d) |

- If a key branch is inverted, which test fails? M1・M6（T3 の valid の行が error になる側も T6 が見る）。
- If a threshold comparison changes, which test fails? M4。
- If a guard is removed, which test fails? M1・M3。
- If an output field is omitted, which test fails? M8（行が出ない）。文の写し漏れは T1〜T3 の完全一致。
- If output order changes, which test fails? T1（2 列の順）。
- If dry-run performs a side effect, which test fails? 該当なし（preview の DB 書込みは今の契約で、本 lane は触らない）。
- If a JSON number crosses JavaScript safe integer range, which test fails? 該当なし（数を足さない）。
- If a state token is round-tripped through browser/client code, which test fails? 該当なし。

## Residual Test Gaps

- Excel が CSV の保存で指数の形を書くか（Packet の Contract Probe P1）は自動 test でも L3 でも確かめない（#196 の流れ 2 の予行演習）。
- 数千行のエラー行の描画の速さは測らない（UI-01c-D18 の範囲の文。L3-3 は 60 行）。
- owner の判断事項の候補（既定で入れない形）の行は、通ることを T6 の一部（12・14 桁）でだけ見る。8 桁の数字の商品コードで JAN が空の行、`00000` で始まる 13 桁の JAN は test を置かない（既定で振舞いを変えないため）。
