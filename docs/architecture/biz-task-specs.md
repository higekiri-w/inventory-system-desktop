# タスク仕様（BIZ層）

## 計数の置き場と在庫の求め直しの契約（D-118。(a) と b1 で実装済み）

SPEC-STK-CNT-D1〜D12（[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)）。詳細な契約は下記の source の同名の節を正とする。(a)（ADR SPEC-STK-CNT-D12）で入ったのは BIZ-01 の初期在庫の movement と求め直し、BIZ-02 / BIZ-07 の全部（精算の窓を除く）。b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で、精算の窓、BIZ-01 の共有 JAN と在庫連動を始める検査、BIZ-03 の Z004 の識別・確定・取消、BIZ-06 の計数の保存と完了、BIZ-07 の共有 JAN の一覧が入った（旧本体は撤去した）。旧 ADR（時点証拠）の契約は置き換えた。

- BIZ-01: [30](../function-design/30-biz-product-service.md)。共有 JAN の在庫連動の拒否、在庫連動を始めるときの連動しない間の売上の検査、初期在庫の movement と求め直し。
- BIZ-02 / BIZ-07: [31](../function-design/31-biz-inventory-service.md) / [36](../function-design/36-biz-integrity-check.md)。並びの鍵付きの movement、求め直しの関数（写しを書く唯一の経路）、要再確認の判定。整合性は写しと求め直しの比較で、補正は求め直しを写しへ書く（操作ログの必須は変えない）。
- BIZ-03: [32](../function-design/32-biz-csv-import-service.md)。Z004 の machine_no・精算回数による識別・順の検査・欠けの知らせ、精算回数の無い file の拒否、確定と取消の後の求め直し、取込みの結果の要再確認の一覧。
- BIZ-06: [35](../function-design/35-biz-stocktake-service.md)。計数の即保存（日付と置き場）、要再確認の一覧、完了の条件（決まらない精算の取込み・要再確認 0・未計数の片付け）と評価額の非遡及。

日報 BIZ-08、売上レポート、PLU 書出しの意味と記録は変えない。EJ は在庫に使わない。

> **親文書**: [ARCHITECTURE.md](../ARCHITECTURE.md)
> **入力ドキュメント**: `docs/spec/requirements.md`、`docs/spec/requirements-coverage.md`、DB_DESIGN.md（テーブル定義書）

---

### BIZ-01: 商品管理ロジック

**タスク要求**: 商品マスタの登録・修正・廃番管理・独自コード発番・売価変更履歴記録・一括価格改定・取引先追加・価格履歴参照・一括インポート・filter 全件 PLU 対象更新の業務ロジックを提供する

**理由**: 商品マスタはシステムの中心であり、入出庫・CSV取込み・棚卸し・売上レポートの全てがproductsテーブルを参照する。商品データの整合性を保つルールをBIZ層に集約することで、CMD層やUI層から直接DB操作する事故を防ぐ

**【データ構造】**

入力データ:
- 商品登録: product_code（JANまたは自動発番）, name, department_id, selling_price, cost_price_centi, tax_rate, stock_unit, 初期在庫数, jan_code（任意）, maker_code（任意）, supplier_id（任意）
- 商品修正: product_code（変更不可）, 変更対象フィールド群
- 廃番切替: product_code, is_discontinued
- 一括インポート: CSVファイルバイト列（任意列 `PLU対象`）
- 価格改定: product_code, new_selling_price, new_cost_price_centi, assign_supplier_id（任意）
- 取引先追加: name
- 価格履歴参照: product_code, limit

出力データ:
- 登録結果: 生成されたproduct_code（独自コード発番時）、成功/失敗
- 検索結果: 商品一覧（ページング対応）、在庫数・売価・原価含む
- 一括インポート結果: プレビューデータ（正常行/エラー行/重複行）、取込み結果サマリ
- 価格改定結果: product_code, changed, plu_dirty_set, supplier_assigned
- 価格履歴: old/new の売価・原価と changed_at（新しい順）

内部で扱うテーブル: products, departments, suppliers, price_history, inventory_movements, stocktake_items, plu_slots, operation_logs

**【処理構造】**

**商品新規登録:**
1. product_codeの決定
   - JANコード入力あり → product_codeとjan_codeの両方に同じ値をセット
   - JANコード入力なし → departmentsのcode_prefix + next_seqで自動発番（例: HZ-0047）。発番後にnext_seq+1
   - product_codeの重複チェック（既存商品との衝突確認）
2. 入力バリデーション
   - name: 必須、空文字不可
   - department_id: departmentsに存在すること
   - selling_price（円）, cost_price_centi（1/100 円）: 0以上の整数
   - tax_rate: '10', '8', '0'のいずれか
   - stock_unit: 10 個の code のいずれか（共通規則 SPEC-UNIT-D1）
3. トランザクション内で実行
   - productsにINSERT（plu_dirty=1, plu_exported_at=NULL, pos_stock_sync=デフォルト1。利用者が商品登録画面で明示的にOFFにできる。長さの単位（stock_unit='m'）の場合は初期値0を提案するがUIで変更可能（UI-01b-D6・D22））
   - 初期在庫 > 0 の場合 → inventory_movementsにINSERT（movement_type='receiving', quantity=初期在庫, stock_after=初期在庫）
   - 進行中の棚卸し（stocktakes.status='in_progress'）があれば → stocktake_itemsに自動追加（actual_count=NULL）
   - operation_logsに記録（operation_type='product_create'）

**商品修正:**
1. product_codeで既存商品を取得（存在しなければエラー）
2. 入力バリデーション（登録時と同じルール）
3. 売価または原価が変わる場合 → price_historyにINSERT（old/new両方記録）
4. 売価が変わる場合 → plu_dirty=1にセット
5. productsをUPDATE（updated_at更新）
6. operation_logsに記録（operation_type='product_update', detail_jsonに変更前後）

**一括価格改定の行単位確定（SPEC-PRV-D5 / D6）:**
1. 売価・原価がともに同じ場合は価格履歴・操作ログを書かない
2. 変更時は 1 transaction で価格の部分更新、price_history old/new 4 値、`product_price_revise` 操作ログを記録する
3. 売価変更時だけ `plu_dirty=1`。原価のみでは立てない
4. assign_supplier_id は商品が NULL のときだけ設定し、既存値を上書きしない
5. create_supplier は trim・空文字拒否・同名既存行返却、list_price_history は DESC・既定 10・上限 100 とする

**廃番切替:**
1. product_codeで既存商品を取得
2. is_discontinuedを反転
3. 廃番化は `plu_target=0` にする。復帰は `plu_target` を自動で戻さない。枠は D-117 の後は解放しない（下の `PLU対象` の節。今の build は解放する、BIZ-04 の「runtime の状態」）
4. operation_logsに記録（operation_type='product_discontinue'）

**独自コード発番:**
1. department_idからdepartmentsを取得
2. code_prefixがNULL → エラー（この部門は独自コード対象外）
3. code_prefix + '-' + next_seqを0埋め4桁でフォーマット（例: "HZ-0047"）
4. 生成したコードでproductsの重複チェック
5. departments.next_seqを+1
6. 生成したproduct_codeを返す

**一括インポート:**
1. IO-03を呼んでファイル読込み＋エンコーディング判定＋パース
2. ヘッダ検証（必須列の存在確認）
3. 各行をバリデーション → 正常行/エラー行/重複行（product_codeが既存）に分類
4. プレビューデータとして返す（ここでは保存しない）
5. 利用者が「取り込む」を選択 → 重複行は「上書き」「スキップ」を行ごとに選択可能
6. トランザクション内で正常行＋上書き対象行をINSERT/UPDATE
7. 各商品について初期在庫ありならinventory_movementsにINSERT
8. operation_logsに記録

**CSV 任意列 `PLU対象`（SPEC-PLS-D6。filter 一致全件の一括更新〈BIZ-01-D4〉は D-117 で外す。今の build には `bulk_set_plu_target` が残る）:**
- `PLU対象` は `1` / `0` / 空欄。新規行の列なし / 空欄は既存導出、更新行は明示値がある場合だけ適用する。JAN が空か正しい 8 桁（13 桁の数字でない）のときの `1` は warning + `0`。形の誤った JAN（8 桁・13 桁の数字でないか、チェックディジットが合わない）は [30](../function-design/30-biz-product-service.md) BIZ-01-D10 の行の error で、warning に届かない
- 1→0 / JAN 変更 / 廃番化で PLU の枠を変えない（解放しない。D-117。今の build は解放する、BIZ-04 の「runtime の状態」）

**前の JAN の別名（D-122、proposed・未実装）:**
- 上書きで JAN を直す行のうち、前の JAN をその商品だけが持ち、新規の行がその JAN を持たない行に「前の番号のバーコードが付いた現物は店にありますか」を出す（既定「ない」。上書きの選択に依らない条件で広めに出す）。「ある」の行は、commit が実際の上書きの選択で 30 BIZ-01-D9 の判定表を満たすとき（取込みの後にその JAN を誰も持たない等）だけ前の JAN を別名として残し、満たさなければ判定表の error で全体を戻す。商品が JAN を持つと、その JAN の別名は DB の trigger が消す

**【制御構造】**
- 独自コード発番はトランザクション内でnext_seqのread→increment→writeを行い排他制御（SQLiteの暗黙ロックで十分）
- 一括インポートは「プレビュー」と「コミット」が別リクエスト。プレビュー結果はフロントエンド側で保持し、コミット時にバックエンドへ送信
- 商品修正中に他の操作が同じ商品を触るケース → 1人運用なので楽観ロック不要。将来拡張でupdated_at一致チェックを追加可能

---

### BIZ-02: 在庫変動ロジック

**タスク要求**: 入庫・返品・手動販売出庫・廃棄による在庫変動を統一的に処理する。inventory_movementsの記録と、その商品の写し（products.stock_quantity ほか）の記録からの求め直し（`recompute_stock`）を常に同じ TX で実行し、入庫保存後は原価差分を保存 transaction と独立に返す（REQ-209 / SPEC-PRV-D8）

**理由**: 在庫変動は4つの異なる画面（入庫/返品/手動販売/廃棄）から発生するが、「在庫を増減してその履歴を記録する」という共通処理がある。これをBIZ-02に集約することで、在庫整合性のルールが1箇所にまとまる

**【データ構造】**

入力データ（共通）:
- product_code, quantity（プラス=増加/マイナス=減少）, movement_type, reference_type, reference_id

入庫固有の入力:
- receiving_record: supplier_id, receiving_date, note
- receiving_items[]: product_code, quantity, cost_price_centi

返品固有の入力:
- return_record: return_type('return'/'exchange'), return_date, register_processed, receipt_image_path, note
- return_items[]: product_code, direction('in'/'out'), quantity

手動販売出庫固有の入力:
- manual_sale: sale_date, reason('plu_unregistered'/'other'), note
- manual_sale_items[]: product_code, quantity, amount

廃棄固有の入力:
- disposal_record: disposal_date
- disposal_items[]: product_code, disposal_type('disposal'/'damage'/'other'), quantity, cost_price_centi, reason

出力データ:
- 処理結果: 成功/失敗、変動後の在庫数、在庫マイナス警告の有無

内部で扱うテーブル: products, inventory_movements, receiving_records/items, return_records/items, manual_sales/items, disposal_records/items, sale_records, operation_logs

**【処理構造】**

**共通在庫変動処理（全ての入出庫から呼ばれる内部関数）:**
1. product_codeでproductsを取得（無ければ NotFound）
2. inventory_movementsに並びの鍵付きでINSERT（movement_type, quantity, reference_type, reference_id, business_date, settlement_no。stock_after は仮に今の写しの値）
3. その商品の写しを記録から求め直して書く（BIZ-02 の recompute_stock。在庫の上限の外は書く前に TX 全体を戻す）
4. 2 の movement の stock_after を、求め直した写しの値に更新する
5. stock_after < 0 の場合 → 在庫マイナス警告フラグをセット（処理は止めない）

**入庫記録の処理手順:**
1. 入力バリデーション（supplier_idがsuppliersに存在、各itemのproduct_codeが存在、quantity > 0）
2. トランザクション内で実行
   - receiving_recordsにINSERT → record_idを取得
   - 各receiving_itemについて:
     - receiving_itemsにINSERT
     - 共通在庫変動処理を呼び出し（quantity=+入庫数, movement_type='receiving', reference_type='receiving_record', reference_id=record_id）
   - operation_logsに記録（operation_type='receiving_create'）

**返品・交換記録の処理手順:**
1. 入力バリデーション（各itemのproduct_codeが存在、quantityが正の整数）
2. トランザクション内で実行
   - return_recordsにINSERT → record_idを取得
   - 各return_itemについて:
     - return_itemsにINSERT
     - register_processed=1（レジ戻し済み）の場合 → 在庫は動かさない（CSV取込みで自動反映）
     - register_processed=0（レジ未処理）の場合:
       - direction='in'（戻り）→ 共通在庫変動処理（quantity=+数量, movement_type='return'）
       - direction='out'（渡し）→ 共通在庫変動処理（quantity=-数量, movement_type='return'）
   - operation_logsに記録（operation_type='return_create'）

**手動販売出庫の処理手順:**
1. 入力バリデーション
2. 各itemのproduct_codeについてPLU登録済みチェック:
   - plu_dirty=0 かつ plu_exported_at IS NOT NULL → 警告フラグ付与（「この商品はレジで打てます」）
   - 利用者が警告を確認した上で続行する場合のみ処理
3. トランザクション内で実行
   - manual_salesにINSERT → sale_idを取得
   - 各manual_sale_itemについて:
     - manual_sale_itemsにINSERT
     - sale_recordsにINSERT（source='manual', csv_import_id=NULL）
     - 共通在庫変動処理（quantity=-数量, movement_type='sale_manual', reference_type='manual_sale', reference_id=sale_id）
   - operation_logsに記録（operation_type='manual_sale_create'）

**廃棄・破損記録の処理手順:**
1. 入力バリデーション（quantity > 0, cost_price_centi >= 0, reason非空）
2. トランザクション内で実行
   - disposal_recordsにINSERT → record_idを取得
   - 各disposal_itemについて:
     - disposal_itemsにINSERT
     - 共通在庫変動処理（quantity=-数量, movement_type='disposal', reference_type='disposal_record', reference_id=record_id）
   - operation_logsに記録（operation_type='disposal_create'）

**【制御構造】**
- 全てのヘッダ+明細+在庫更新+変動履歴はBEGIN〜COMMITで一括。途中で失敗したらROLLBACK
- 在庫マイナス警告はエラーではなく警告。処理は続行する（利用者が後で確認）
- 返品のregister_processedフラグは画面入力時に決定。BIZ層では渡された値に従って分岐するだけ

---

### BIZ-03: Z004商品別CSV取込みパイプライン

**タスク要求**: Z004ファイルを読み込み、売上記録の作成と在庫更新を4段階パイプラインで実行する

**理由**: Z004はPLU登録後の商品別売上を表し、BIZ-03は`pos_stock_sync`に基づく在庫増減とロールバックまで実装済みである。ファイルフォーマットの癖（CP932/NEL/layout A/B）、マスタ未登録商品への対応、ロールバック可能性を考慮して段階的な処理パイプラインを使う。current operation の日報主入力（Z001/Z002/Z005）は BIZ-08 で扱う

**【データ構造】**

入力データ:
- Z004ファイルバイト列

段階間受け渡しデータ:
- ParseResult: settlement_date, parsed_rows[]{line_no, normalized_jan, name, quantity, amount}, parse_errors[]{line_no, error_type, error_message}
- ValidateResult: matched_rows[]{line_no, product_code, quantity, amount, pos_stock_sync}, error_rows[]{line_no, normalized_jan, name, quantity, amount, error_type, error_message}, excluded_count
- PreviewData: file_info{filename, settlement_date, file_hash}, matched_summary{count, total_amount}, error_summary{count, items[]}, duplicate_check_result

出力データ:
- 取込み結果: csv_import_id, status('completed'/'completed_partial'), total_items, total_amount, skipped_count

内部で扱うテーブル: csv_imports, csv_import_errors, sale_records, products, inventory_movements, operation_logs

**【処理構造】**

※ 4段階パイプラインの詳細はdb-design/pos-tables.mdの「B-1: Z004パース仕様」セクションに記載済み。ここでは各段階の責務とBIZ層の判断ポイントを記述する。

**Stage 1: Parse（IO-02に委譲）**
1. IO-02（Z004パーサー）を呼び出し
2. ParseResultを受け取る
3. parse_errorsが存在し、parsed_rowsが0件 → 「有効なデータがありません」で中断。operation_logsに記録（csv_import_parse_failed）
4. parse_errorsが存在し、parsed_rowsも存在 → Stage 2に進む（行エラーは最終的にcsv_import_errorsに記録）

**Stage 2: Validate**
1. parsed_rowsの空レコード除外（JAN=0000...、quantity=0 and amount=0）
2. 実データ行についてproducts.jan_codeで検索
   - ヒット1件 → matched_rowsに追加。pos_stock_syncフラグをコピー
   - ヒット0件 → 前の JAN の別名（D-122、proposed・未実装）があればその商品で matched_rows に追加、無ければ error_rowsに追加（error_type='unmatched_product'）
   - ヒット複数件 → ORDER BY product_code ASCで先頭に紐付け
3. Stage 1のparse_errorsもerror_rowsにマージ
4. ValidateResultを構築

**Stage 3: Preview（フロントエンドとの連携ポイント）**
1. file_hashで重複チェック
   - status IN ('completed','completed_partial') → ブロック
   - status = 'rolled_back' → 許可
2. settlement_dateで既存チェック
   - 同じ日の別hash取込みが存在 → active import全件summaryと追加確認フラグを付与
3. PreviewDataを構築してフロントエンドに返す
4. **ここでフロントエンドからの利用者操作を待つ**

**Stage 4: Commit**（以下の Commit とロールバックの手順は、現行 build では停止中の旧本体の記述。[停止 ADR](../adr/2026-09-23-legacy-stocktake-z004-write-stop.md)、[32](../function-design/32-biz-csv-import-service.md) §15.0）
1. preview statusと `additional_import_confirmed` の組合せを検証する
2. トランザクション内で同一file_hashを再検査し、続けて同日active import ID snapshotを再取得する。不一致は「同日の取込み状況が変わりました。再度プレビューしてください」で副作用なく止める
3. 既存importを変更せず、トランザクション内で新規分だけ実行
   - csv_importsにINSERT → import_idを取得
   - matched_rowsの各行について:
     - sale_recordsにINSERT（source='auto', csv_import_id=import_id, source_line_no=line_no）
     - pos_stock_sync=1の場合:
       - 在庫視点でquantityを変換（返品:sale_records.quantity<0→inventory_movements.quantity=+|quantity|、通常販売→inventory_movements.quantity=-quantity）
       - inventory_movementsにINSERT（movement_type='sale_auto', reference_type='csv_import', reference_id=import_id）し、その商品を `recompute_stock` で求め直す（BIZ-02 の `apply_stock_change`）
     - pos_stock_sync=0の場合 → sale_recordsのみ、在庫は動かさない
   - error_rowsがある場合 → csv_import_errorsにINSERT
   - csv_imports.status / total_items / total_amount / skipped_countを確定
   - operation_logsに記録（operation_type='csv_import'）
4. COMMIT

**ロールバック処理:**
1. csv_import_idで対象を特定
2. トランザクション内で実行
   - 関連するsale_recordsをis_voided=1に更新
   - 関連するinventory_movementsをis_voided=1に更新
   - is_voided=1にしたinventory_movementsの商品ごとに `recompute_stock` で求め直す（補償の movement は作らない）
   - csv_imports.statusを'rolled_back'に更新
   - operation_logsに記録（operation_type='csv_rollback'）
3. 同一settlement_dateの他のactive importは変更しない

**【制御構造】**
- Stage 3→4の間に利用者の操作待ちがある。Preview結果はサーバ側キャッシュに保持し、フロントエンドはpreview_tokenのみを保持してcommit時に送り返す（有効期限30分）
- Commit中はUIロック（D-3排他制御: Preview開始〜Commit完了/キャンセルまでdisabled）
- ロールバックは物理削除ではなく論理無効化（is_voided=1）

---

### BIZ-08: 日報取込みロジック

**タスク要求**: Z001/Z002/Z005 の1営業日分bundleを読み込み、日報サマリ・支払集計・部門別売上を4段階パイプラインで保存する。商品別売上や在庫変動へは擬似展開しない

**理由**: 2026-06-30 field-check により、現在の店舗日報主入力は Z001/Z002/Z005 であると分かった。これらは商品コード/JAN単位の明細ではないため、既存 `sale_records` や `inventory_movements` に入れると reporting/accounting 意味が壊れる。日報集計を別モデルとして保存し、Z004商品別取込みとは分ける

**【データ構造】**

入力データ:
- DailyReportSourceFile[]（filename, bytes）。標準はレジの SD から読んだ候補の束（IO-09 経由、§37.9 `scan_register_sd_daily_reports`、D-111）、予備は利用者が選んだ 3 ファイル（dialog の path を持ち、自動で見つけた SD の root の下の file は SD の入力として扱う。37 §37.3 手順 1a）。SD の探し方の選択は BIZ-08 の `DailyReportSdSelection`（CMD-12 が作る。IO-09 の型にしない）

段階間受け渡しデータ:
- DailyReportSdScan（SD の候補の一覧と状態〈取り込める / 取込み済み / 同じ精算を取込み済み / そろわない / 読めない〉、BIZ-08-D3）
- DailyReportParseResult（IO-07出力。settlement_no? を含む）
- DailyReportValidateResult: report_date, source_files, summary_lines, payment_lines, department_lines{department_id?}, warnings[], duplicate_check
- DailyReportPreviewData: file_info{report_date, source_files, bundle_hash}, totals{gross_amount?, net_amount?}, payment_summary, department_summary, warnings, duplicate_check

出力データ:
- DailyReportImportResult: daily_report_import_id, status('completed'), report_date, gross_amount?, net_amount?, warning_count

内部で扱うテーブル:
- daily_report_imports
- daily_report_summary_lines
- daily_report_payment_lines
- daily_report_department_lines
- departments
- operation_logs

**【処理構造】**

**Stage 1: Parse（IO-07に委譲）**
1. IO-07（POS日報bundleパーサー）を呼び出す
2. parse_errors がある場合は commit 不可として中断し、`daily_report_imports` は作らない
3. parse失敗は operation_logs に `daily_report_parse_failed` として汎用文言で記録する
4. parse errorのsource/filename/line/type/messageは開発者向けdiagnostic WARNへ構造化して記録し、利用者向けerrorとoperation logのraw detailには含めない（BIZ-08-D1）。精算回数の不一致（`settlement_mismatch`）だけは、選び直しを案内する固定の文にする（BIZ-08-D2）

**Stage 2: Validate**
1. Z001/Z002/Z005 の report_date が一致することを確認する
2. bundle_hash を source順（Z001→Z002→Z005）と個別file_hashから計算する
3. 必須サマリ行の存在と数値変換済みであることを確認する
4. Z005の部門名を departments.name に照合する
   - 一致 → department_id を付与
   - 不一致 → warning とし、department_id=NULL で続行可能
5. SD の経路で settlement_no が None の束は止め（BIZ-08-D6）、bundle_hashでactive同一bundleを判定し、同じ report_date・settlement_no の別bundleの completed があれば止め（BIZ-08-D4、追加確認で通さない）、report_dateで同日別bundleの追加確認snapshotを作る

**Stage 3: Preview**
1. 対象日、3ファイル、総売上/純売上、支払集計、部門別集計、部門未対応warningを返す
2. 同一bundleのcompleted取込みはブロックする
3. 同一report_dateの別bundleがある場合は、active import全件summaryを示して追加確認を要求する
4. preview_token を返し、CMD層cacheに30分保持する

**Stage 4: Commit**
1. preview statusと `additional_import_confirmed` の組合せを検証する。SD の経路で settlement_no が None の cache は写しを書く前に止める（BIZ-08-D6）
2. トランザクション内で同一bundle_hashと BIZ-08-D4 の照合を再検査し、続けて同日active import ID snapshotを再取得する。不一致は「同日の取込み状況が変わりました。再度プレビューしてください」で副作用なく止める
3. 既存parentを変更せず、トランザクション内で新規分だけ実行
   - daily_report_importsにINSERT
   - summary/payment/department linesをINSERT
4. COMMIT
5. operation_logsに `daily_report_import` を記録する
   - ログ記録失敗は日報取込み自体を巻き戻さず、診断ログまたは後続確認対象として扱う

**ロールバック処理:**
1. daily_report_import_idで対象を特定する
2. トランザクション内で status='rolled_back' と rolled_back_at を更新する
3. COMMIT
4. operation_logsに `daily_report_rollback` を記録する
   - ログ記録失敗はrollback済み状態を戻さず、診断ログまたは後続確認対象として扱う
5. sale_records、inventory_movements、products.stock_quantity は変更しない
6. 同一report_dateの他のcompleted parentは変更しない

**【制御構造】**
- BIZ-03と同じく Parse→Validate→Preview→Commit を採用するが、書込先とrollback意味は別物
- 日報取込みは日報集計の正本であり、商品別ランキングや在庫引落しの根拠にはならない
- 部門未対応は取込み失敗ではなくwarning。後続の部門マッピング改善やREQ-403照合で扱う

**集計契約**:
- 商品別日次は同日の `is_voided=0` 全行を `product_code + source` で加算し、manual/autoを混ぜない
- 公式日報日次は同日の全completed parentを加算する。親・明細のoptional値はいずれかNULLなら集約結果もNULLとし、単一parent IDではなく `source_import_count` を返す（Z001 の行は全 active import を読むが加算せず、取込みごとに ID つきで返す、D-096）
- 月次は同日複数parentを加算し、将来のcoverage日数は `COUNT(DISTINCT report_date)` とする。公式日報seriesと商品別seriesは互いに加算しない

---

### BIZ-03 / BIZ-08 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | SPEC-SDI-D1〜D8: Z004/日報の追加取込み、TX内snapshot再検証、per-import rollback、additive readを正本化。 |

---

### BIZ-04: PLU書出しロジック

**タスク要求**: 毎日の SD の読取りで Z004 全スロットダンプからレジ占有を照合し、JAN 単位の memory No. を永続予約し、CV17 用 product 行を生成・確認する（D-072 / D-117 / REQ-907）。

> **runtime の状態**: 本節は D-117 の後の形（採用済み）。今の build は、占有を利用者が選ぶ Z004 で照合し（`biz/plu_export_service.rs` の `import_plu_register_snapshot`）、`plu_target` 1→0・廃番化・JAN の変更で枠を解放し（`release_plu_slot_for_jan`、`release_pending`）、Clear 行を書く（`PLU_CLEAR_ROW_ENABLED = true`）。撤去と SD の読取りでの照合への切替えは PLU の書出しの見直しの runtime（[33](../function-design/33-biz-plu-export-service.md) §16 の冒頭の「runtime の状態」の注記）。

**データ構造**: `plu_slots`、products、snapshot 用 app_settings、operation_logs。prepare / confirm は exact product_code set と `memory_no` を受け渡す。要修正理由に `no_free_slot` を含む。

**処理構造**:
1. `refresh_plu_register_snapshot_from_sd` は SD の読取りの窓の中の Z004 から精算回数が最大の 1 本を選び、IO-02 の `(memory_no, raw_code)` を 1 TX で 4 status と照合し、日時を保存する（件数は operation_logs だけ）。一度も照合していなければ prepare を `register_snapshot_required` で拒否する
2. prepare は sticky slot を再利用し、同一コードの `external` slot があればそれを `active` として採用し、それ以外の未割当 JAN に最小 `free` を `reserved` として 1 TX で予約する。空きなしは該当 JAN だけ要修正へ送る
3. Diff は dirty 対象、Full は対象の JAN の app 管理 reserved / active。external / free、対象の商品を持たない JAN の slot と要修正中の既存 slot は出力しない
4. confirm は exact memory No. を再検証し、reserved→active、対象商品を反映済みにする
5. `plu_target` 1→0、廃番化、JAN 変更では枠を変えない（解放しない、clear 行を書かない。D-117）

**制御構造**:
- snapshot / prepare / confirm は各 1 transaction。途中失敗は slot と product を部分更新しない
- Diff / Full とも CV17 へ投入できる。失敗時はどちらかの mode で書き出し直す（前に保存した file は取り込まない。保存の後の照合で枠が変わりうる）
- アプリは枠を別の JAN に使い回さない（D-117）
- slot 状態を売上・在庫・会計集計へ混ぜない

---

### BIZ-05: 売上集計ロジック

**タスク要求**: 日次・月次の売上データを集計し、レポート画面とCSVエクスポートに必要なデータを提供する

**理由**: 利用者が日々の売上を把握し、月次で経営判断するためのデータ。is_voided=0のレコードのみを対象にし、ロールバック済みデータを除外する

**【データ構造】**

入力データ:
- 日次: 対象日（YYYY-MM-DD）
- 月次: 対象月（YYYY-MM）、集計モード（'by_product' / 'by_department'）

出力データ:
- 日次売上一覧: [{product_code, name, department_name, quantity, amount, source('auto'/'manual')}], 部門小計[], 総合計
- 月次売上集計: [{product_code or department_name, quantity, amount, ranking}], 前月比較データ

内部で扱うテーブル: sale_records, products, departments

**【処理構造】**

**日次売上一覧:**
1. sale_recordsからis_voided=0 AND sale_date=対象日のレコードを取得
2. productsとJOINして商品名・部門名を付与
3. 部門別にグルーピングして小計を計算
4. sourceで記録元（auto/manual）を区別して返す

**月次売上集計（商品別）:**
1. sale_recordsからis_voided=0 AND sale_dateが対象月内のレコードを集計
2. product_codeでGROUP BY、SUM(quantity), SUM(amount)
3. amountの降順でランキング付与
4. 前月の同集計を取得して比較データを生成

**月次売上集計（部門別）:**
1. sale_records + productsをJOINしてdepartment_idでGROUP BY
2. SUM(quantity), SUM(amount)
3. 前月比較

**CSVエクスポート:**
1. 上記の集計結果を受け取り、IO-05に渡してCSV生成

**【制御構造】**
- 集計クエリは読み取り専用。トランザクション不要
- ページングは月次の商品別で必要になる可能性がある（4000商品）。LIMIT/OFFSETで対応

---

### BIZ-06: 棚卸しロジック

**タスク要求**: 棚卸しの開始・カウント入力・確定を管理し、仕入原価総額を算出する

**理由**: 年末棚卸し（10 月からの準備と、大晦日に終える正式カウントに分かれる。[73](../function-design/73-ui-stocktake.md) §73.1）で、全商品の実在庫をカウントしてシステム在庫との差異を補正する。税理士報告用の仕入原価総額の算出も行う

**【データ構造】**

入力データ:
- 棚卸し開始: なし（全商品を対象に自動生成）
- カウント入力: stocktake_item_id, actual_count
- 棚卸し確定: stocktake_id, force_fill（true=未入力をシステム在庫と同じとみなす。先決事項D-1、2026-04-12確定）

出力データ:
- 棚卸し一覧: [{product_code, name, department_name, system_stock, actual_count, difference, counted_at}], 進捗（入力済み/未入力件数）
- 確定結果: total_cost（仕入原価総額）、差異があった商品一覧

内部で扱うテーブル: stocktakes, stocktake_items, products, inventory_movements, operation_logs

**【処理構造】**

**棚卸し開始:**
1. 進行中の棚卸し（status='in_progress'）が存在するかチェック → 存在すればエラー（1件まで制約）
2. stocktakesにINSERT（status='in_progress'）→ stocktake_idを取得
3. 全商品（is_discontinued=0、または is_discontinued=1かつstock_quantity>0）について:
   - stocktake_itemsにINSERT（system_stock=現在のproducts.stock_quantity, actual_count=NULL）
4. is_discontinued=1 かつ stock_quantity=0 の商品 → actual_count=0として自動入力
5. operation_logsに記録（operation_type='stocktake_start'）

**カウント入力:**
1. stocktake_itemsのactual_countを更新、counted_at=現在日時
2. 差異の表示は「現在のproducts.stock_quantity - actual_count」で動的計算（棚卸し中もCSV取込みで在庫が動くため）

**棚卸し確定:**（棚卸しの開始・カウント入力・確定は、現行 build では停止中の旧本体の記述。[停止 ADR](../adr/2026-09-23-legacy-stocktake-z004-write-stop.md)、[35](../function-design/35-biz-stocktake-service.md) §20.0）
1. 未入力（actual_count=NULL）の商品が残っていないかチェック → 残っていれば警告（確定を阻止、または「未入力はシステム在庫と同じとみなす」選択肢）
2. トランザクション内で実行
   - 各stocktake_itemについて:
     - valuation_cost_price_centi = 現在のproducts.cost_price_centi をセット
     - difference = products.stock_quantity - actual_count
     - difference ≠ 0 の場合:
       - inventory_movementsにINSERT（movement_type='stocktake', quantity=actual_count - products.stock_quantity, stock_after=actual_count, reference_type='stocktake', reference_id=stocktake_id）
       - 補正の movement を足した後、その商品を `recompute_stock` で求め直す
   - total_cost = 商品別の金額（valuation_cost_price_centi・評価数量〈現行の旧本体は actual_count、新方式は `max(補正後現在庫,0)`〉・価格の基準数量から1/100円で四捨五入）の合計を円未満で四捨五入（[35 §20.5a](../function-design/35-biz-stocktake-service.md#205a-評価額の計算価格の基準数量と店の丸め)）
   - stocktakes.total_cost, completed_at, status='completed'を更新
   - operation_logsに記録（operation_type='stocktake_complete', detail_jsonに差異件数・total_cost）
3. COMMIT

**【制御構造】**
- 棚卸し中（status='in_progress'）のCSV取込みは許可（SP-205-09修正）。差異は動的計算で対応
- 棚卸し中に新規商品が登録された場合 → BIZ-01の商品登録処理内でstocktake_itemsに自動追加
- 進行中の棚卸しは1件まで。アプリ層でチェック
- カウント入力は1件ずつ保存（中断再開対応）。トランザクション不要

---

### BIZ-07: 整合性チェックロジック

**タスク要求**: products の写しの 4 項目と、記録から求め直した値（BIZ-02 の derive_stock）を突合し、不整合を検出する

**理由**: 写しは記録から求め直した派生値であり、バグや DB の外からの編集で記録とずれる可能性がある。定期的なチェックで不整合を早期発見する

**【前提条件】**
- inventory_movementsは全商品の完全な在庫変動履歴を持つ。初期在庫投入もmovement_type='receiving'としてinventory_movementsに記録される（DB_DESIGN.md「stock_quantity整合性チェック」の前提条件に明記済み）
- この前提が成立するため、記録から求め直した値（計数の無い商品では SUM(inventory_movements.quantity WHERE is_voided=0) と同じ）が理論上の正しい在庫数となる
- 初期投入漏れやpos_stock_sync=0の商品でも、inventory_movementsに記録がある限りチェックは有効

**【データ構造】**

入力データ: なし（全商品を対象）

出力データ:
- チェック結果: [{product_code, name, stock_unit, copy（写しの 4 項目）, derived（求め直した 4 項目）, differing_fields}], 不整合件数

内部で扱うテーブル: products, inventory_movements, stock_counts, operation_logs

**【処理構造】**

1. 全商品について、写しの 4 項目と derive_stock の値を比べ、違う項目（数・要再確認・範囲）を求める
2. 違う項目のある商品を不整合リストに追加（数が同じで印・範囲だけが違う商品も含む）
3. 不整合が0件 → 「問題ありません」で完了
4. 不整合がある場合 → 不整合リストを利用者に表示
5. 利用者が「補正する」を選択した場合:
   - 各不整合商品について写しを求め直しの結果（recompute_stock）へ直接更新する（inventory_movements に行は追加しない — BIZ-07-D2 / D-051、詳細は function-design/36-biz-integrity-check.md §21.4）
   - 補正内容は同一TX内の operation_logs（operation_type='integrity_fix'、before/after 付き）へ必須記録し、記録失敗時は補正ごとロールバックする（BIZ-07-D3、D-6の明示例外）
6. チェック実行の operation_logs 記録（operation_type='integrity_check', detail_jsonに不整合件数）はTX外 best-effort のまま（D-6準拠）。補正の記録はステップ5のTX内 'integrity_fix' ログが担う（2ログ分離）

**【制御構造】**
- チェックタイミング: CSV取込み完了時（自動）、棚卸し確定時（自動）、設定画面から手動実行
- 画面からの補正は必ず利用者確認を挟む（移行の手順 8 だけが migration の TX の中で直し、操作ログと整合性の画面で知らせる。22 §19）
- チェック中の他の操作は制限しない（読み取り専用クエリのため）

### BIZ-09: システム設定・操作ログロジック

**タスク要求**: 設定 CRUD と操作ログ検索の BIZ 境界を提供する（順12 = D-060。関数契約の正本は 38-biz-system-service.md）

**理由**: 旧 CMD-11 は `settings_cmd` から `system_repo`（IO-01）を直接呼び、操作ログの日付 validation まで CMD が所有していた（監査 P2-1）。層境界の標準経路（CMD → BIZ → IO）へ復帰し、業務 validation の単一所有層（ARCH-VAL-D1）を回復する

**【前提条件】**
- 対応 command は get_settings / update_setting / list_logs / list_log_operation_types の 4 系（CMD-11）
- wire 契約（kind / message / field、レスポンス型）は移行前と不変。CMD は `From<BizError>` の一元変換に復帰する

**【データ構造】**

入力データ: 設定 key/value、LogQuery（page, per_page, operation_type?, start_date?, end_date?）

出力データ: AppSetting 一覧、PaginatedResult<OperationLog>、operation_type distinct 一覧

内部で扱うテーブル: settings, operation_logs（いずれも system_repo = IO-01 経由）

**【処理構造】**
1. 設定一覧取得・設定 upsert・操作ログ検索・operation_type 一覧を system_repo 経由で提供する
2. 操作ログ検索の日付 validation（strict YYYY-MM-DD 形式・実在暦日・start>end 早期拒否。function-design 43 §43.5 の条件・文言・field を維持）を本タスクが所有し、`BizError::ValidationFailed`（field 付き）で返す
3. per_page の上限クランプ（200）は従来どおり IO 層（D-031 共有定数）が実行する

**【制御構造】**
- トランザクション制御は不要（単発の read / upsert のみ）
- backup/restore・領収書画像は本タスクの対象外（backup/restore は CMD → MNT-01 正規経路、画像拡張子 validation は BIZ-02 所有 — D-060）
