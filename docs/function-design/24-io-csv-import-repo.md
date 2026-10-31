## 14. IO-01 追加: POS取込みリポジトリ（BIZ-03 / BIZ-08 用）

### 計数の置き場と在庫の求め直しの契約（D-118。b1 で実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D4・D7・D9）の IO の側。以下を b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で csv_imports の API に足した。旧 ADR（時点証拠）の受領（`pos_import_sources`）・受領済みの上限・同一性の拒否の記録・取消の事前読取りの操作は持たない（置き換えた）。

| 操作 | 入力 → 出力 / 処理 |
|---|---|
| import の保存 | `NewCsvImport` に `machine_no: String`・`settlement_no: i64` を足す（[POS](../db-design/pos-tables.md) の同名の契約）。既存の active hash の重複の判定と取消済みの再取込みは変えない |
| active な Z004 の精算の一覧 | (conn) → completed / completed_partial の全件の import ID・`machine_no`・`settlement_no`・`settlement_date`（settlement_no の昇順）。BIZ-03 の識別・順の検査・欠けの知らせと、BIZ-02 の求め直し（取り込み済みの精算の並び）が使う。判定は BIZ が行う |
| Z004 の取込みの machine_no の照合先 | (conn, machine_no) → 取り消したもの（rolled_back）を含む Z004 の取込みのうち `machine_no` が引数と違うもの 1 件（active な取込みがあれば active を優先し、同じ状態の中では import ID の最大）の import ID・`machine_no`・`settlement_no`・`status`・`imported_at`。無ければ None。新設する（active な精算の一覧は rolled_back を返さず、一覧の行の型 `CsvImport` は新しい列を読まない）。BIZ-03 の順 2（[32](32-biz-csv-import-service.md)）が、違う取込みの有無と、拒否の文で名指す取込みに使う |
| Z004 の取込みの最大の取込みID | (conn) → 取り消したもの（rolled_back）を含む全ての取込みの `MAX(id)`（無ければ None）。新設する。BIZ-06 が fresh な計数の保存の TX（`stock_counts` の `seen_import_id`）と棚卸しの完了の TX（`stocktakes` の `cutoff_import_id`。[tracking](../db-design/tracking-system-tables.md)）で読む。取込みID は使い回されない（AUTOINCREMENT、取消は行を消さない。[POS](../db-design/pos-tables.md)）ので、このカーソル以下の active な取込みは保存・完了の前に取り込まれて今もある精算になる（ADR D3・D6 の判定の n は BIZ が上の active な一覧から求める） |
| 同じ精算の active な取込みの取得 | (conn, machine_no, settlement_no) → active な import の ID と hash（0 件か 1 件。2 件以上は DbError ではなく BIZ が不整合として拒む） |
| movement の保存 | Z004 の movement は `business_date = settlement_date`・`phase = 1`・`settlement_no` を持つ（[21](21-io-inventory-repo.md) の `NewMovement` の並びの鍵） |

IO は精算日・番号・file 名から信用や前後を付与しない。判定は BIZ-03 が preview と確定の TX で取り直す。

b1 で足した読取り（`src-tauri/src/db/sales_repo.rs`、`pub(crate)`）:

```rust
pub(crate) struct SettlementRow { import_id: i64, machine_no: String, settlement_no: i64, settlement_date: String }
pub(crate) struct MachineNoRef { import_id: i64, machine_no: String, settlement_no: i64, status: CsvImportStatus, imported_at: String }

pub(crate) fn list_active_settlements(conn: &DbConnection) -> Result<Vec<SettlementRow>, DbError> // active な Z004 の精算（settlement_no の昇順）
pub(crate) fn max_csv_import_id(conn: &DbConnection) -> Result<Option<i64>, DbError> // 取り消したものを含む MAX(id)（取込みのカーソル）
pub(crate) fn find_machine_no_mismatch(conn: &DbConnection, machine_no: &str) -> Result<Option<MachineNoRef>, DbError> // machine_no の照合先（active を優先）
pub(crate) fn find_active_imports_for_settlement(conn: &DbConnection, machine_no: &str, settlement_no: i64) -> Result<Vec<(i64, String)>, DbError> // 同じ精算の active な取込み（ID・hash）
pub(crate) fn list_import_product_codes(conn: &DbConnection, import_id: i64) -> Result<Vec<String>, DbError> // その取込みの売上か movement を持つ商品（無効の行も含む）
pub(crate) fn list_sale_import_ids(conn: &DbConnection, product_code: &str, stock_linked: bool) -> Result<Vec<i64>, DbError> // 取込みのときの連動が stock_linked の有効な売上を持つ active な取込み
```

### 14.1 モジュール配置

```
src-tauri/src/
  db/
    sales_repo.rs  -- sale_records, csv_imports, csv_import_errors
```

sales_repo.rs に追加する（ARCHITECTURE.md: sales_repository は sale_records, csv_imports, csv_import_errors, daily_report_imports, daily_report_*_lines を管理）。
セクション11（insert_sale_record）は BIZ-02 用として既存。本セクションで BIZ-03（CSV取込みパイプライン）用の関数を追加する。

**前提**: SQLite単一接続（1人運用デスクトップ）。マルチ接続化する場合は file_hash への UNIQUE 制約条件付き再導入またはアプリ層排他ロックが必須（INV-6 参照）。

---

### 14.2 型定義

**CsvImport構造体**:
- id: i64
- filename: String
- settlement_date: String（YYYY-MM-DD）
- file_hash: String（SHA-256 hex、小文字64文字）
- total_items: i64
- total_amount: i64
- skipped_count: i64
- status: CsvImportStatus（"completed" / "completed_partial" / "rolled_back"）
- imported_at: String（YYYY-MM-DDTHH:MM:SS）

**NewCsvImport構造体**:
- filename: String
- settlement_date: String（YYYY-MM-DD）
- file_hash: String（SHA-256 hex）
- total_items: i64
- total_amount: i64
- skipped_count: i64
- status: String（"completed" / "completed_partial"）
- machine_no: String（精算の鍵。Z004 の確定だけが作る。[POS](../db-design/pos-tables.md)）
- settlement_no: i64（同上）

**NewCsvImportError構造体**:
- csv_import_id: i64
- source_line_no: i64
- normalized_jan: Option\<String\>（JAN正規化前にエラーならNone）
- raw_name: String
- raw_quantity: String（値の出どころ〈ParseError 由来は元の文字列、unmatched_product は整数の文字列表現〉とTEXTで保持する理由は DB_DESIGN.md 12a 参照）
- raw_amount: String（同上）
- error_type: String（"unmatched_product" / "invalid_format" / "invalid_jan" / "invalid_number"）
- error_message: String（利用者向け日本語メッセージ）

**VoidedMovement構造体**:
- product_code: String
- quantity: i64（元の quantity。BIZ-03 の在庫補正は使わない。BIZ-03 は無効にした movement の商品を `recompute_stock` で求め直す）

**DailyReportImport構造体**:
- id: i64
- report_date: String（YYYY-MM-DD）
- source_adapter: String（"casio_sr_s4000"）
- bundle_hash: String
- source_files_json: String
- gross_amount: Option\<i64\>
- net_amount: Option\<i64\>
- status: String（"completed" / "rolled_back"）
- imported_at: String
- rolled_back_at: Option\<String\>
- note: Option\<String\>

**NewDailyReportImport構造体**:
- report_date: String
- source_adapter: String
- bundle_hash: String
- source_files_json: String
- gross_amount: Option\<i64\>
- net_amount: Option\<i64\>
- status: String（"completed"）
- note: Option\<String\>
- settlement_no: Option\<i64\>（D-111 で足す。束の精算回数〈IO-07-D5〉、`None` は NULL。BIZ-08-D4 の照合に使う）

`DailyReportImport`（上、`specta::Type` の行の DTO）には `settlement_no` を足さない（list の wire を変えない。`DailyReportImport` を返す既存の SELECT は列を名指しし、`row_to_daily_report_import` はその列の位置で読むので、表に列を足しても読み方は変わらない。2026-10-07 の `src-tauri/src/db/sales_repo.rs` で確認）。精算回数での照合は §14.18a の内部の関数だけが読む。

**NewDailyReportSummaryLine / NewDailyReportPaymentLine / NewDailyReportDepartmentLine構造体**:
- DB_DESIGN.md 12c〜12e のカラムに対応するinsert用構造体。`daily_report_import_id` は親INSERT後にBIZ-08が設定する。

---

### 14.3 csv_imports.status 状態遷移表

**状態遷移**:

| 現在の状態 | 遷移先 | トリガー |
|-----------|--------|---------|
| （新規INSERT） | completed | commit 成功、error_rows = 0 |
| （新規INSERT） | completed_partial | commit 成功、error_rows > 0 |
| completed | rolled_back | rollback_csv_import |
| completed_partial | rolled_back | rollback_csv_import |
| rolled_back | （状態変更なし） | rollback は冪等 |

processing 相当の中間状態は存在しない。TX 内で INSERT → totals 確定まで一括実行するため、外部から観測できる中間状態が不要。

**再取込み可否表**:

| status | 同一 file_hash の再取込み | 理由 |
|--------|------------------------|------|
| completed | ブロック | 二重取込み防止 |
| completed_partial | ブロック | 部分成功も取込み済みとみなす |
| rolled_back | 許可 | 取消済みなので再取込み可能 |

---

### 14.4 insert_csv_import

**関数要求**: csv_imports に1行INSERTし、挿入されたIDを返す

**シグネチャ**:
```
fn insert_csv_import(conn: &DbConnection, record: &NewCsvImport) -> Result<i64, DbError>
```

**処理ステップ**:
1. INSERT INTO csv_imports (filename, settlement_date, file_hash, total_items, total_amount, skipped_count, status, imported_at) VALUES (?, ?, ?, ?, ?, ?, ?, 現在日時)
2. last_insert_rowid() を返す

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

**入出力例**:
```
入力: NewCsvImport {
  filename: "Z004_260321",
  settlement_date: "2026-03-21",
  file_hash: "a1b2c3d4...（64文字hex）",
  total_items: 45,
  total_amount: 28500,
  skipped_count: 3,
  status: "completed_partial",
}

出力: Ok(1)  // 挿入されたID
```

**注意**: file_hash の重複チェックは本関数の責務外。BIZ-03 が find_blocking_import_by_file_hash で事前チェックした上で呼び出す。

---

### 14.5 find_csv_import_by_id

**関数要求**: csv_imports からIDで1件取得する。ロールバック処理の対象確認に使用

**シグネチャ**:
```
fn find_csv_import_by_id(conn: &DbConnection, id: i64) -> Result<Option<CsvImport>, DbError>
```

**処理ステップ**:
1. SQL実行: `SELECT * FROM csv_imports WHERE id = ?`
2. 結果が0行 → Ok(None)
3. 結果が1行 → CsvImport にマッピングして Ok(Some(...))

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

**入出力例**:
```
入力: id = 1

出力（存在する場合）: Ok(Some(CsvImport {
  id: 1,
  filename: "Z004_260321",
  settlement_date: "2026-03-21",
  file_hash: "a1b2c3d4...（64文字hex）",
  total_items: 45,
  total_amount: 28500,
  skipped_count: 3,
  status: "completed_partial",
  imported_at: "2026-03-21T18:30:00",
}))

出力（存在しない場合）: Ok(None)
```

---

### 14.6 find_blocking_import_by_file_hash

**関数要求**: file_hash で有効な（非 rolled_back）csv_imports を検索する。重複取込みブロック判定専用

**シグネチャ**:
```
fn find_blocking_import_by_file_hash(conn: &DbConnection, file_hash: &str) -> Result<Option<CsvImport>, DbError>
```

**処理ステップ**:
1. SQL実行: `SELECT * FROM csv_imports WHERE file_hash = ? AND status IN ('completed','completed_partial') ORDER BY id DESC LIMIT 1`
2. 結果が0行 → Ok(None)
3. 結果が1行 → CsvImport にマッピングして Ok(Some(...))

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

**入出力例**:
```
入力: file_hash = "a1b2c3d4...（64文字hex）"

ケース1（ブロック対象あり）:
出力: Ok(Some(CsvImport {
  id: 5,
  filename: "Z004_260321",
  settlement_date: "2026-03-21",
  file_hash: "a1b2c3d4...",
  status: "completed",
  ...
}))

ケース2（ブロック対象なし — 未取込みまたは rolled_back のみ）:
出力: Ok(None)
```

**設計判断**: 1件取得（ORDER BY id DESC）で十分な理由 — file_hash に UNIQUE 制約はないが、status IN ('completed','completed_partial') が同一 hash に対して複数存在することは通常ありえない（commit は check-then-insert で排他される）。仮に不整合で複数あっても、1件でもあればブロックする判定には影響しない。

---

### 14.7 find_imports_by_settlement_date

**関数要求**: settlement_date で有効な csv_imports を全件検索する。同日追加確認の snapshot と commit 時の再検証に使用する。

**シグネチャ**:
```
fn find_imports_by_settlement_date(conn: &DbConnection, date: &str) -> Result<Vec<CsvImport>, DbError>
```

**処理ステップ**:
1. SQL実行: `SELECT * FROM csv_imports WHERE settlement_date = ? AND status IN ('completed','completed_partial') ORDER BY imported_at DESC, id DESC`
2. 結果を Vec\<CsvImport\> で返す（0件なら空 Vec）

`settlement_date` は group key であり uniqueness key ではない。呼出側は全件を追加確認表示と active ID snapshot に使い、先頭1件へ縮退してはならない。

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

### 14.8 update_csv_import_status

**関数要求**: csv_imports の status を更新する。ロールバック時に rolled_back に変更する用途

**シグネチャ**:
```
fn update_csv_import_status(conn: &DbConnection, id: i64, status: &str) -> Result<bool, DbError>
```

**処理ステップ**:
1. UPDATE csv_imports SET status = ? WHERE id = ?
2. affected_rows == 1 → Ok(true)
3. affected_rows == 0 → Ok(false)（該当レコードなし）

**エラーハンドリング**:
- CHECK 制約違反（不正な status 値）→ DbError::QueryFailed(詳細)
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

### 14.9 update_csv_import_totals

**関数要求**: csv_imports の total_items / total_amount / skipped_count / status を確定する。Stage 4（Commit）の最終ステップで使用

**シグネチャ**:
```
fn update_csv_import_totals(conn: &DbConnection, id: i64, total_items: i64, total_amount: i64, skipped_count: i64, status: &str) -> Result<bool, DbError>
```

**処理ステップ**:
1. UPDATE csv_imports SET total_items = ?, total_amount = ?, skipped_count = ?, status = ? WHERE id = ?
2. affected_rows == 1 → Ok(true)
3. affected_rows == 0 → Ok(false)

**エラーハンドリング**:
- CHECK 制約違反（不正な status 値）→ DbError::QueryFailed(詳細)
- SQL実行失敗 → DbError::QueryFailed(詳細)

**注意**: insert_csv_import で仮の total_items=0 / total_amount=0 / skipped_count=0 で INSERT し、全行処理後に本関数で確定値を書き込むパターンを想定。BIZ-03 の TX 内で呼ばれる。

---

### 14.10 insert_csv_import_errors（batch）

**関数要求**: csv_import_errors に複数行を一括 INSERT する

**シグネチャ**:
```
fn insert_csv_import_errors(conn: &DbConnection, errors: &[NewCsvImportError]) -> Result<(), DbError>
```

**処理ステップ**:
1. errors が空なら即座に Ok(()) を返す
2. prepared statement を1回作成:
   `INSERT INTO csv_import_errors (csv_import_id, source_line_no, normalized_jan, raw_name, raw_quantity, raw_amount, error_type, error_message, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 現在日時)`
3. errors の各要素に対して prepared statement を実行

**エラーハンドリング**:
- FK 違反（csv_import_id 不正）→ DbError::ForeignKeyViolation(詳細)
- CHECK 制約違反（error_type 不正）→ DbError::QueryFailed(詳細)
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

### 14.11 void_sale_records_by_import

**関数要求**: 指定 csv_import_id の sale_records を is_voided=1 に更新する。ロールバック用

**シグネチャ**:
```
fn void_sale_records_by_import(conn: &DbConnection, csv_import_id: i64) -> Result<u64, DbError>
```

**処理ステップ**:
1. SQL実行: `UPDATE sale_records SET is_voided = 1 WHERE csv_import_id = ? AND is_voided = 0`
2. affected_rows を返す

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

**注意**: 既に is_voided=1 のレコードは更新しない（冪等）。affected_rows=0 はエラーではない（既に void 済み、または該当レコードなし）。

---

### 14.12 void_movements_by_reference

**関数要求**: 指定 reference_type / reference_id の inventory_movements を is_voided=1 に更新し、void 対象の product_code と quantity を返す。BIZ-03 は戻り値の product_code で、その商品を記録から求め直す（[32](32-biz-csv-import-service.md) §15.5）

**シグネチャ**:
```
fn void_movements_by_reference(conn: &DbConnection, ref_type: &str, ref_id: i64) -> Result<Vec<VoidedMovement>, DbError>
```

**処理ステップ**:
1. SELECT で void 対象を事前取得:
   `SELECT product_code, quantity FROM inventory_movements WHERE reference_type = ? AND reference_id = ? AND is_voided = 0`
2. 結果を Vec\<VoidedMovement\> に変換
3. UPDATE で一括 void:
   `UPDATE inventory_movements SET is_voided = 1 WHERE reference_type = ? AND reference_id = ? AND is_voided = 0`
4. Vec\<VoidedMovement\> を返す

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

**入出力例**:
```
入力: ref_type = "csv_import", ref_id = 5

出力: Ok(vec![
  VoidedMovement { product_code: "4976383262108", quantity: -3 },
  VoidedMovement { product_code: "4976383262207", quantity: -8 },
  VoidedMovement { product_code: "4973167902615", quantity: 1 },
])
```

BIZ-03 での在庫補正:
- BIZ-03 は無効にした movement の商品を `recompute_stock` で求め直す（quantity は在庫補正に使わない。[32](32-biz-csv-import-service.md) §15.5、`csv_import_service/rollback.rs`。D-118 の b1 で、計数を持つ全ての商品とその取込みの商品を求め直す）。写しが記録と一致しているとき、求め直しの値は「写し − 無効にした数量」と同じになる

**設計判断**:
- movement_type 条件を付与しない理由: INV-7 により reference_type='csv_import' の movements は sale_auto 限定であることが保証されている。冗長な条件は可読性を下げるだけ
- SELECT → UPDATE の2段階にする理由: BIZ-03 が求め直す商品（無効にした movement の product_code）を BIZ に渡すため。UPDATE の RETURNING 句は SQLite 3.35+ で利用可能だが、rusqlite のサポート状況を考慮して SELECT + UPDATE の安全なパターンを採用

---

### 14.13 list_csv_imports

**関数要求**: csv_imports 一覧をページング取得する。CSV取込み履歴画面用

**シグネチャ**:
```
fn list_csv_imports(conn: &DbConnection, page: u32, per_page: u32) -> Result<PaginatedResult<CsvImport>, DbError>
```

**入力ガード**（安全弁。通常はBIZ層で先にバリデーション済み）:
- page < 1 → DbError::QueryFailed("page must be >= 1")
- per_page < 1 → DbError::QueryFailed("per_page must be >= 1")
- per_page > 100 → DbError::QueryFailed("per_page must be <= 100")（極端値入力の防御）

**処理ステップ**:
1. 入力ガードチェック
2. COUNT(*) で total_count を取得
3. ORDER BY settlement_date DESC, imported_at DESC, id DESC。同日複数importを行単位で返し、日付単位にcollapseしない
4. LIMIT per_page OFFSET (page - 1) * per_page で取得
5. PaginatedResult { items, total_count, page, per_page } を返す

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

### 14.13a get_csv_import_record_detail / list_csv_import_error_rows

**関数要求**: CSV取込み記録詳細（ヘッダ、sale_records 明細、関連 `inventory_movements`）とエラー行を取得する。CSV取込み詳細画面（`/csv-import/records/$importId`、[65-inventory-record-traceability.md](65-inventory-record-traceability.md) §65.3 / §65.5）用

**シグネチャ**:
```
fn get_csv_import_record_detail(conn: &DbConnection, import_id: i64) -> Result<CsvImportRecordDetailCore, DbError>
fn list_csv_import_error_rows(conn: &DbConnection, csv_import_id: i64) -> Result<Vec<CsvImportErrorRecord>, DbError>
```

**CsvImportRecordDetailCore構造体**（IO 内部型。wire DTO は BIZ-03 側 [32-biz-csv-import-service.md](32-biz-csv-import-service.md) §15.6a が所有）:
- header: CsvImport（§14.2 既存型を再利用。file_hash を含むが wire DTO には載せない）
- items: Vec\<CsvImportRecordDetailItem\>
- movements: Vec\<MovementRecord\>（source=None のまま）

**CsvImportRecordDetailItem構造体**（sale_records 行）:
- id: i64, product_code: String, product_name: String, department_name: String, stock_unit: String
- quantity: i64, amount: i64, is_voided: bool

**CsvImportErrorRecord構造体**（csv_import_errors 行。error_type は DB raw TEXT のまま返す — enum `CsvImportErrorType` は BIZ-03 所有のため IO 層では変換しない）:
- source_line_no: i64, normalized_jan: Option\<String\>, raw_name: String, raw_quantity: String, raw_amount: String, error_type: String, error_message: String

**処理ステップ**（get_csv_import_record_detail）:
1. csv_imports のヘッダを import_id で取得する。存在しない場合は DbError::NotFound
2. sale_records を csv_import_id で products / departments と JOIN し、商品名・部門名・単位を付ける。is_voided を含め、ORDER BY id ASC（取込み順）で全行返す（rolled_back 取込みの明細も表示対象のため is_voided で絞らない）
3. inventory_movements から `reference_type='csv_import'` かつ `reference_id=import_id` かつ `is_voided=0` の行を取得する（[21-io-inventory-repo.md](21-io-inventory-repo.md) §10.2 get_receiving_record_detail と同パターン。rollback 済み取込みは movements が void 済みのため 0 件になり、rollback 状態はヘッダ status で表示する）
4. movements の source link は BIZ 層で補完するため、IO 層では source=None のまま返す

**処理ステップ**（list_csv_import_error_rows）:
1. csv_import_errors を csv_import_id で ORDER BY source_line_no ASC, id ASC で取得する（該当なしは空 Vec で正常）

**エラーハンドリング**:
- ヘッダ不存在 → DbError::NotFound
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

### 14.14 insert_daily_report_import

**関数要求**: `daily_report_imports` に1行INSERTし、IDを返す。

**シグネチャ**:
```
fn insert_daily_report_import(conn: &DbConnection, record: &NewDailyReportImport) -> Result<i64, DbError>
```

**処理ステップ**:
1. INSERT INTO daily_report_imports (...)（`settlement_no` の列を含む。D-111）
2. last_insert_rowid() を返す

**注意**: bundle_hash / report_date の重複判定はBIZ-08の責務。本関数はSQL insertだけを行う。

---

### 14.15 insert_daily_report_lines

**関数要求**: 日報取込み配下のsummary/payment/department linesを一括INSERTする。

**シグネチャ**:
```
fn insert_daily_report_summary_lines(conn: &DbConnection, rows: &[NewDailyReportSummaryLine]) -> Result<(), DbError>
fn insert_daily_report_payment_lines(conn: &DbConnection, rows: &[NewDailyReportPaymentLine]) -> Result<(), DbError>
fn insert_daily_report_department_lines(conn: &DbConnection, rows: &[NewDailyReportDepartmentLine]) -> Result<(), DbError>
```

**処理ステップ**:
1. rows が空なら Ok(())。
2. prepared statement を作成する。
3. rows の各要素をINSERTする。

**エラーハンドリング**:
- FK違反（daily_report_import_id / department_id不正）→ DbError::ForeignKeyViolation
- CHECK制約違反（source_file不正等）→ DbError::QueryFailed
- SQL実行失敗 → DbError::QueryFailed

---

### 14.16 find_daily_report_import_by_id

**関数要求**: `daily_report_imports` からIDで1件取得する。rollback対象確認に使用する。

**シグネチャ**:
```
fn find_daily_report_import_by_id(conn: &DbConnection, id: i64) -> Result<Option<DailyReportImport>, DbError>
```

---

### 14.17 find_blocking_daily_report_by_bundle_hash

**関数要求**: 同一bundleのcompleted日報取込みを検索する。二重取込みブロック判定専用。

**シグネチャ**:
```
fn find_blocking_daily_report_by_bundle_hash(conn: &DbConnection, bundle_hash: &str) -> Result<Option<DailyReportImport>, DbError>
```

**SQL方針**:
`SELECT * FROM daily_report_imports WHERE bundle_hash = ? AND status = 'completed' ORDER BY id DESC LIMIT 1`

---

### 14.18 find_daily_report_imports_by_report_date

**関数要求**: 同一report_dateのcompleted日報取込みを全件検索する。追加確認の snapshot と commit 時の再検証に使用する。

**シグネチャ**:
```
fn find_daily_report_imports_by_report_date(conn: &DbConnection, report_date: &str) -> Result<Vec<DailyReportImport>, DbError>
```

**SQL方針**:
`SELECT * FROM daily_report_imports WHERE report_date = ? AND status = 'completed' ORDER BY imported_at DESC, id DESC`

`report_date` は group key であり uniqueness key ではない。呼出側は `source_files_json` を安全に解析して表示用 filename 一覧を作り、全 active ID の順序付き snapshot を保持する。

---

### 14.18a find_same_settlement_daily_report_import（D-111、BIZ-08-D4）

**関数要求**: 同じ対象日・同じ精算回数で、bundle_hash の違う completed の日報取込みを探す。BIZ-08-D4 の照合（§37.3 手順 8・§37.4 手順 4a）と、SD の候補の状態（§37.9 手順 7 の `SameSettlementImported`）の専用。wire に出さない。

**シグネチャ**:
```
fn find_same_settlement_daily_report_import(
    conn: &DbConnection,
    report_date: &str,
    settlement_no: i64,
    bundle_hash: &str,
) -> Result<Option<i64>, DbError> // 一致した import の id
```

**SQL方針**:
`SELECT id FROM daily_report_imports WHERE report_date = ? AND settlement_no = ? AND bundle_hash <> ? AND status = 'completed' ORDER BY id DESC LIMIT 1`

`settlement_no` が NULL の行は `= ?` に当たらないので照合されない（D-111 より前の取込み・layout B）。呼出側は `settlement_no` が `None` の束で本関数を呼ばない。

---

### 14.18b exists_same_date_daily_report_import_without_settlement_no（D-121、BIZ-08-D8、proposed・未実装）

**関数要求**: 同じ対象日の completed の日報取込みに、精算回数の無い（`settlement_no IS NULL`）ものがあるかを返す。BIZ-08-D8（37 §37.10）が同日追加の確認を残すかを決める専用。wire に出さない。後続の runtime（A）が足す。

**シグネチャ**:
```
fn exists_same_date_daily_report_import_without_settlement_no(
    conn: &DbConnection,
    report_date: &str,
) -> Result<bool, DbError>
```

**SQL方針**:
`SELECT EXISTS (SELECT 1 FROM daily_report_imports WHERE report_date = ? AND status = 'completed' AND settlement_no IS NULL)`。`status` の条件は §14.18 の同日の一覧（`src-tauri/src/db/sales_repo.rs` の `find_daily_report_imports_by_report_date` の `status = 'completed'`）と同じで、取り消した（`rolled_back`）取込みは数えない。

---

### 14.18c latest_completed_daily_report_date（D-111、BIZ-08-D3）

**関数要求**: completed の日報取込みの対象日（`report_date`）の最大を返す。SD の scan の窓の始まり（[37](37-biz-daily-report-import-service.md) §37.9 手順 2）の専用で、呼び手は `biz::daily_report_import_service` の scan（`src-tauri/src/biz/daily_report_import_service/scan.rs`）だけ。wire に出さない。BIZ からだけ呼ぶので `pub(crate)`（`src-tauri/tests/design_compliance_test.rs` の照合は `pub` だけを見るので allowlist には足さない）。

**シグネチャ**:
```
pub(crate) fn latest_completed_daily_report_date(conn: &DbConnection) -> Result<Option<String>, DbError>
```

**SQL方針**:
`SELECT MAX(report_date) FROM daily_report_imports WHERE status = 'completed'`。completed が 1 件も無ければ `None`。取り消した（`rolled_back`）取込みだけの日は数えない（§14.18 と同じ `status` の条件。`sales_repo.rs` の test `test_req401_latest_completed_daily_report_date_ignores_rolled_back`）。

返すのは保存された文字列のままで、日付として解くのと `today` − 30 日との早い方を選ぶのは BIZ の scan が行う（解けない値は warn を残して既定の窓にする。`scan.rs` の手順 2）。

---

### 14.19 rollback_daily_report_import

**関数要求**: `daily_report_imports` を `rolled_back` に更新する。

**シグネチャ**:
```
fn rollback_daily_report_import(conn: &DbConnection, id: i64, rolled_back_at: &str) -> Result<bool, DbError>
```

**処理ステップ**:
1. `UPDATE daily_report_imports SET status='rolled_back', rolled_back_at=? WHERE id=? AND status='completed'`
2. affected_rows == 1 なら true、0なら false。

**注意**: summary/payment/department lines は物理削除しない。親statusで有効/無効を判断する。取消対象は指定 ID の親だけであり、同一 `report_date` の他の completed 親は変更しない。すでに rolled_back の ID はBIZ層で冪等成功として扱う。

---

### 14.20 list_daily_report_imports

**関数要求**: 日報取込み履歴をページング取得する。

**シグネチャ**:
```
fn list_daily_report_imports(
    conn: &DbConnection,
    page: u32,
    per_page: u32,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Result<PaginatedResult<DailyReportImport>, DbError>
```

**処理ステップ**:
1. page / per_page の入力ガードを行う。
2. date_from/date_toがある場合は report_date 範囲条件を付与する。
3. ORDER BY report_date DESC, imported_at DESC, id DESC で取得する。

---

### 14.21 get_completed_daily_report_aggregate

**関数要求**: 指定日の全 completed 日報取込みと配下の支払/部門別行を取得し、部門別行を集約し、支払の行と Z001 の行は合算せず取得して、BIZ-05の日次レポート用構造へ渡す（支払の行の合算は BIZ-05。proposed・未実装、D-142）。

**シグネチャ**:
```
fn get_completed_daily_report_aggregate(conn: &DbConnection, report_date: &str) -> Result<Option<OfficialDailyReportRow>, DbError>
```

返り値の `OfficialDailyReportRow` は IO が所有する DB DTO（`sales_repo`）で、BIZ-05 が `map_official_daily_report` で公開 DTO `OfficialDailyReportSummary` へ写し、warning を作る。IO は BIZ の型を参照しない。

**処理ステップ**:
1. `daily_report_imports` から `report_date=? AND status='completed'` の親を全件、親の `id`・`report_date`・`gross_amount`・`net_amount`・`imported_at` を `imported_at ASC, id ASC` の順で取得する。親がなければ `Ok(None)` を返す。`imported_at` と並びは手順 6 の取込みごとの行（行の無い親を含む）が使う。手順 2 の gross / net の加算は、従来どおり新しい順（`imported_at DESC, id DESC`、手順 1 の並びの逆）で行う。`checked_add` の途中の溢れ（手順 7）と NULL の伝播（どれかが NULL なら NULL）は加算の順に依るため、取得の並びを変えても既存の結果（日次の数字と日次 CSV）を変えない（例: 古い順の gross が `i64::MAX`・`1`・`-i64::MAX` の 3 親は、新しい順で足すと `1`、古い順で足すと途中で溢れて `DbError::QueryFailed`）。
2. `source_import_count` は対象親件数とする。親 `gross_amount` / `net_amount` はそれぞれ合計するが、いずれかの親で対象値が NULL なら集約値も NULL とする。
3. `daily_report_payment_lines` は足さずに返す（proposed・未実装、[D-120](../decision-log.md)・D-142。今の実装はここで `payment_key` で集約し、並び順から作る `payment_N` が、同日の取込みの間で名称の違う行を 1 行に足しうる）。手順 1 と同じ条件の親に join する 1 つの query で、行ごとに親の `id`・`imported_at` と行の `id`・`sort_order`・`label`・`payment_key`・`amount`・`count` を、親の `imported_at ASC, id ASC`・行の `sort_order ASC, id ASC` で読み、`OfficialDailyReportRow` の field `payment_lines` に入れる。足し方（[34](34-biz-sales-service.md) の SALES-SHEET-D2）、group の代表、互換の範囲、warning は BIZ-05 が持つ（34 §19.3 手順 3。日報の紙と同じ 1 つの関数）。`payment_key` は保存を続けるが足す鍵に使わない。
4. `daily_report_department_lines` は `department_id` がある行をその ID で、未対応行を `normalized_department_name`、それもなければ `raw_department_name` で集約する。amount は合計し、`quantity_hundredths` / count は対象行のいずれかが NULL なら集約値も NULL とする。`quantity_hundredths` は 100 倍の整数のまま足して DB DTO に入れ、単位の数への変換は BIZ-05 が行う（IO-07-D2）。
5. 部門の label と sort は、各 group の最小 `sort_order`、同値なら最小 row ID の行を決定的な代表とする（支払の group の代表は BIZ-05、34 §19.3 手順 3）。未対応の部門は集約後の group として返し、import ごとに重複させない（警告の文は BIZ-05 が作る）。
6. `daily_report_summary_lines`（Z001）を、手順 1 と同じ条件（`report_date=? AND status='completed'`、`rolled_back` の親の行は読まない）の親に join する 1 つの query で読み、親の `imported_at ASC, id ASC`、行の `sort_order ASC, id ASC` で並べる。行は合算しない（D-096）。手順 1 の親ごとに 1 件の取込み（親 `id`・`imported_at`・行の `label` / `quantity_hundredths` / `count` / `amount`）を作り、行の無い親も空の行で残す（件数は `source_import_count` と等しい）。`line_key` は返さない。新しい repo 関数・schema・index は足さない（`idx_daily_report_summary_lines_import_id` を使う）。
   取込みと行の型は、BIZ-05 の `OfficialDailySummaryImport` / `OfficialDailySummaryLine`（[34 §19.2](34-biz-sales-service.md#192-型定義)）と同じ形の DB DTO（IO の所有。既存の `OfficialDailyDepartmentRow` と同じ命名で `OfficialDailySummaryImportRow` / `OfficialDailySummaryLineRow`。ただし個数は `quantity_hundredths: Option<i64>`〈100 倍の整数〉で、単位の数への変換は BIZ-05）とし、`OfficialDailyReportRow` の field `summary_imports` に入れる。
7. 手順 2〜4 の加算はすべて（NULL を伝える加算〈親の `gross_amount` / `net_amount`、部門の `quantity_hundredths` / count〉と、部門の amount の合計。支払は手順 3 のとおり IO では足さない）溢れを検査して足し（`checked_add`）、`i64` を溢れたら黙って wrap せず `DbError::QueryFailed` を返す（月次の §14.22 の `SUM` の溢れと同じく error で止める）。100 倍の整数は範囲内の値（MNT-03-D12 の範囲検査、`i64::MAX / 100`）でも 2 行の和で溢れうる。返した `DbError` の上位の扱いは既存の DB 読み取りの失敗と同じで（[34 §19.3](34-biz-sales-service.md#193-get_daily_sales) のエラーハンドリング、[40 §5.3](40-cmd-product.md#53-cmd層のエラー返却方針) の `internal`、CMD-ERR-D2）、利用者には既存の文「データベースエラーが発生しました。もう一度お試しください」と error_id が出る。新しい文言は足さない。
8. `OfficialDailyReportRow` を返す。field ごとの持つものと BIZ-05 の扱いは次の 3 つで、混ぜない（proposed・未実装、D-142）。(a) 内部の支払の行（`payment_lines` の要素。手順 3）は、行ごとに親の `id`・`imported_at` と行の `id`・`sort_order` を持つ。BIZ-05 はこれを SALES-SHEET-D2 の関数の入力にだけ使い、wire に出さない（足し方・並び・group の代表は [34](34-biz-sales-service.md) §19.3 手順 3 が決め、IO の並びと値のまま写さない）。(b) 公開する支払の集計（wire の `OfficialDailyPaymentLine`）は BIZ-05 が (a) を D2 で足した結果で、親 ID・行 ID・`imported_at` を持たない。(c) Z001 の取込みごとの行（`summary_imports`）は親 ID と `imported_at` を持ち、BIZ-05 はこれを公開 DTO へ並びと値を変えずに写す（単純な写像。D-096）。部門の集約（手順 4）は単一親 ID を返さない。

---

### 14.21a get_daily_report_sheet_lines

（proposed・未実装、D-120。日報の紙〈[34 §19.3a](34-biz-sales-service.md)〉の読取り）

**関数要求**: 指定日の全 completed 日報取込みの数と、配下の Z001・Z002・Z005 の全行を、足さずに返す。足し方は BIZ-05 の SALES-SHEET-D2 が持つ。

**シグネチャ**:
```
fn get_daily_report_sheet_lines(conn: &DbConnection, report_date: &str) -> Result<DailyReportSheetLinesRow, DbError>
```

返り値は IO が所有する DB DTO（`DailyReportSheetLinesRow { import_ids: Vec<i64>, lines: Vec<DailyReportSheetLineRow> }`、`DailyReportSheetLineRow { source: DailyReportSourceKind, daily_report_import_id: i64, sort_order: i64, label: String, quantity_hundredths: Option<i64>, count: Option<i64>, amount: Option<i64> }`）。`label` は Z001・Z002 の `label`、Z005 の `raw_department_name`（`normalized_department_name` は使わない。紙はレジの名称のまま、34 SALES-SHEET-D2）。Z002 の行は `quantity_hundredths` を持たない（NULL）。`import_ids` はその日の completed の親の `id` を `imported_at ASC, id ASC` で並べたもので、行の無い親も含む。BIZ-05 は、その長さを取込みの数に、行の `daily_report_import_id` がこの並びの何番目か（1 から）を `import_seq` に使う（34 SALES-SHEET-D2 の 3）。

**処理ステップ**:
1. `daily_report_imports` から `report_date=? AND status='completed'` の親の `id` を `imported_at ASC, id ASC` で読み、`import_ids` にする。空なら `lines` を空にして返す。
2. 3 つの行の表（`daily_report_summary_lines`・`daily_report_payment_lines`・`daily_report_department_lines`）を、手順 1 と同じ条件の親に join して読み、帳票（Z001 → Z002 → Z005）・親の `imported_at ASC, id ASC`・行の `sort_order ASC, id ASC` で並べる。`rolled_back` の親の行は読まない。読む親は手順 1 の `import_ids` にある親に限る（手順 1 の後に確定した取込みの行を混ぜない。どの行の `daily_report_import_id` も `import_ids` にあり、BIZ-05 が `import_seq` を必ず引ける）。
3. 値は保存されたまま返す（足さない、名称の空白も除かない）。新しい schema・index は足さない（各行の表の `daily_report_import_id` の index を使う）。

---

### 14.21b get_active_z004_import_totals

（proposed・未実装、D-120。日報の紙の PLU の部分〈34 SALES-SHEET-D4〉の読取り）

**関数要求**: 指定の精算日の active な Z004 の取込みの数と、その `skipped_count` の和を返す。

**シグネチャ**:
```
fn get_active_z004_import_totals(conn: &DbConnection, settlement_date: &str) -> Result<(i64, i64), DbError>
```

**処理ステップ**:
1. `csv_imports` から `settlement_date=? AND status IN ('completed','completed_partial')` の行の数と `skipped_count` の和を 1 つの query で読む（行が無ければ `(0, 0)`）。和の溢れは SQLite の error のまま `DbError::QueryFailed`（§14.22 の `SUM` と同じ）。

---

### 14.22 get_monthly_official_department_totals

**関数要求**: 指定日付範囲のcompleted日報取込みから、Z005由来の公式部門集計を月次集計する。

**シグネチャ**:
```
fn get_monthly_official_department_totals(
    conn: &DbConnection,
    date_from: &str,
    date_to: &str,
) -> Result<Option<Vec<OfficialMonthlyDepartmentTotal>>, DbError>
```

**処理ステップ**:
1. `daily_report_imports.status='completed'` かつ `report_date BETWEEN ? AND ?` の親を対象にする。
2. `daily_report_department_lines` を department_id / label 単位で集計する。個数は `SUM(quantity_hundredths)`（100 倍の整数のまま。BIZ-05 が単位の数に戻す、IO-07-D2）。`SUM` が `i64` を溢れると SQLite は `integer overflow` の error を返し、既存の `map_err` で `DbError::QueryFailed` になる（黙って誤った値を返さない。BIZ-05 以降の経路は §14.21 手順 7 と同じ）。
3. 対象日報が0件なら Ok(None)。
4. 対象日報がある場合は Ok(Some(rows))。行が0件なら空Vecを返す。
5. 同一日複数親もすべて加算対象とする。将来、取込み済み営業日数を返す場合は親件数ではなく `COUNT(DISTINCT report_date)` を使う。

---

### 14.23 非目的

本リポジトリが**行わない**ことを明記する:

- **物理 DELETE**: 全ての無効化処理は is_voided=1 フラグ方式で統一する。csv_imports / sale_records / inventory_movements のいずれも物理削除しない
- **ビジネスロジック**: バリデーション（file_hash 重複判定のブロック/許可分岐）、符号変換（売上帳票視点↔在庫視点）、在庫補正の実行は BIZ-03 の責務
- **file_hash の算出**: SHA-256 ハッシュの計算は IO-02（Z004パーサー）の責務。本リポジトリは算出済みの hex 文字列を受け取るだけ
- **bundle_hash の算出**: Z001/Z002/Z005 bundle_hash の計算は BIZ-08 の責務。本リポジトリは算出済みhex文字列を受け取るだけ
- **トランザクション制御**: BEGIN / COMMIT / ROLLBACK は BIZ-03 / BIZ-08 が制御する。本リポジトリの各関数は渡された conn 上で SQL を実行するのみ

---

### 14.24 対応不変条件

- **INV-4**: is_voided の使用範囲 — CSV 取込みロールバック時（BIZ-03）のみ使用。void_sale_records_by_import と void_movements_by_reference がこれに該当
- **INV-6**: file_hash 自然冪等性 — find_blocking_import_by_file_hash が `status IN ('completed','completed_partial')` で判定。UNIQUE 制約なし（ロールバック後の再取込みで同一 hash が2行になるため）。単一接続前提で check-then-insert の競合なし
- **INV-7**: csv_import 参照の movements は sale_auto 限定 — void_movements_by_reference が movement_type 条件なしで安全に void できる根拠
- **D-025**: 日報取込みは sale_records / inventory_movements へ擬似展開しない — daily_report_imports のrollbackは親status更新のみで完結する
- **D-071 / SPEC-SDI-D1〜D8**: business date は group key であり uniqueness key ではない。active content hash 単位の identity、指定 ID 単位の rollback、同日の全 active imports の additive read を維持する（Z001 の行は全 active import を読むが加算せず取込みごとに返す、D-096）

### 14.25 EJ の記録の保存と照会（D-121、BIZ-08-D9、proposed・未実装）

`ej_records`（[pos-tables](../db-design/pos-tables.md) の「SD の一括取込みと EJ の控えの契約」）を読み書きする。呼ぶのは BIZ-08 §37.11 だけ。TX は BIZ-08 が持つ。runtime（後続の lane A）は置き場の module を決め、`design_compliance_test.rs` の本書の行に足す。

型の所有: repo が受け返しする型（下の `NewEjRecord` と、37 §37.11 に形を書いた `EjRecordSummary`・`EjRecordDetail`・`EjRecordRef`・`EjNumberGap`・`EjRecordModeCode`・`EjRecordKindCode`）は db の module に定義し、`src-tauri/src/biz/mod.rs` で re-export する（既存の形: `CsvImport`〈`src-tauri/src/biz/mod.rs`〉、`Stocktake` ほか〈同 `stocktake_repo` の re-export〉）。CMD は BIZ の re-export を使い db を直接参照しない。BIZ だけが組み立てる型（`EjImportPlan`・`EjImportResult`・`EjConflict`・`EjRecordDay`・`EjSourceFile`）は BIZ-08 に置く。mode・kind は db の列では TEXT + CHECK のまま、型では enum で持ち、書くときは enum の文字列、読むときは明示の match で変える（外れた値は `DbError`。D-061 (c)、先例 `CsvImportStatus`〈`src-tauri/src/db/sales_repo.rs` の `CsvImportStatus` と `row_to_csv_import` の `match`〉）。

```rust
struct NewEjRecord {
    machine_prefix: String,
    record_no: String,
    printed_at: String,
    mode: EjRecordModeCode,
    kind: EjRecordKindCode,
    settlement_report: Option<String>,
    settlement_completed: Option<bool>,
    total_amount: Option<i64>,
    body_text: String,
}
```

```
fn find_ej_record_texts(
    conn: &DbConnection,
    keys: &[(String, String)], // (machine_prefix, record_no)
) -> Result<HashMap<(String, String), String>, DbError> // 保存済みの key → body_text

fn insert_ej_record(conn: &DbConnection, record: &NewEjRecord, imported_at: &str) -> Result<i64, DbError>

fn list_ej_records_on_date(conn: &DbConnection, date: &str) -> Result<Vec<EjRecordSummary>, DbError> // 最大 1001 件（BIZ が 1000 件を超えたかを知るため）

fn find_ej_adjacent_dates(conn: &DbConnection, date: &str) -> Result<(Option<String>, Option<String>), DbError>

fn find_latest_ej_date(conn: &DbConnection) -> Result<Option<String>, DbError>

fn find_ej_records_by_no(conn: &DbConnection, record_no: &str) -> Result<Vec<EjRecordRef>, DbError>

fn get_ej_record_detail(conn: &DbConnection, id: i64) -> Result<Option<EjRecordDetail>, DbError>

fn list_ej_number_gaps(conn: &DbConnection) -> Result<Vec<EjNumberGap>, DbError>
```

**SQL方針**:
- `find_ej_record_texts`: `SELECT machine_prefix, record_no, body_text FROM ej_records WHERE (machine_prefix, record_no) IN (...)`（UNIQUE の index を使う。key の数が多いときは BIZ が分けて呼ぶ）。
- `insert_ej_record`: `INSERT INTO ej_records (...) VALUES (...)`。UNIQUE に当たったら `DbError` を返す（BIZ は TX の中で先に比べるので通常は当たらない）。
- `list_ej_records_on_date`: `WHERE printed_at >= ?1 || ' 00:00' AND printed_at < ?2 || ' 00:00'`（?2 は翌日）、`ORDER BY machine_prefix, record_no LIMIT 1001`。
- `find_ej_adjacent_dates`: 前は `SELECT substr(printed_at, 1, 10) FROM ej_records WHERE printed_at < ?1 || ' 00:00' ORDER BY printed_at DESC LIMIT 1`、次は翌日 00:00 以上で `ORDER BY printed_at ASC LIMIT 1`。`find_latest_ej_date` は `ORDER BY printed_at DESC LIMIT 1`。
- `list_ej_number_gaps`: 4 桁の欄ごとに、`record_no` を整数にした値を昇順に並べ、隣との差が 2 以上の所を区間（前 + 1 〜 後 − 1）にする（window 関数 `LAG`。SQLite 3.25 以降、同梱は 3.45.0〈22 §15〉）。区間を 4 桁の欄・番号の昇順で返し、番号は 6 桁の 0 埋めの文字列に戻す。区間の数は保存した記録の数以下。
- 日付は `printed_at`（記録の日時の文字列）で引き、アプリの時計と比べない。

### 更新履歴

| 日付 | PR | 内容 |
|---|---|---|
| 2026-08-16 | PR #79 | SPEC-SDI-D1〜D8: 同日別 hash の active import 全件取得、commit snapshot 再検証、per-import rollback、日報の日次・月次 additive read 契約を正本化。 |
| 2026-09-27 | daily-report-z-display（design） | §14.21 に Z001 の行を取込みごとに読む手順 6 を追加（D-096）。rename 済みの旧 symbol の「実装遷移義務」の段落を削除（[Plan Packet](../archive/plans/2026-09-27-daily-report-z-display.md)）。 |
| 2026-10-06 | z001-display（runtime、起票） | §14.21 手順 1 の取得列に `imported_at` と並び `imported_at ASC, id ASC` を書いた（手順 6 の行の無い親の取込み日時の出どころ。PR #114 Final Review の P3。[Plan Packet](../archive/plans/2026-10-06-z001-display.md)）。 |
| 2026-10-07 | sd-direct-read（Plan Review round 2 の是正、D-111） | `NewDailyReportImport.settlement_no` と §14.14 の INSERT の列、§14.18a `find_same_settlement_daily_report_import`（BIZ-08-D4）。`DailyReportImport` の行の DTO と既存の SELECT は変えない。 |
| 2026-10-06 | z001-display（Plan Review round 1 の是正） | §14.21 手順 1 に、gross / net の加算は従来どおり新しい順で行うこと（溢れと NULL の伝播が順に依るため）を書いた（[Plan Packet](../archive/plans/2026-10-06-z001-display.md)）。 |
| 2026-10-17 | sd-bulk-import-design（design、D-121） | §14.25 EJ の記録の保存と照会と、§14.18b 同じ日の精算回数の無い取込みの有無（BIZ-08-D8。Plan Review round 1 の是正）を proposed で足した。Plan Review round 2 の是正で、§14.25 に型の所有（db で定義し biz で re-export）と mode・kind の enum を書いた。 |
| 2026-10-27 | 日報の紙の runtime（起票、D-142） | §14.21 の関数要求と手順 3・5・7 を、支払の行を足さずに返す形に直した（足し方・group の代表・互換の範囲・warning は 34 §19.3 手順 3 の BIZ-05。日報の紙と同じ 1 つの関数。proposed・未実装）。 |
| 2026-10-29 | 日報の紙の runtime（Plan Review round 1 の是正） | §14.21 手順 8 を、内部の支払の行（親 ID・行 ID を持ち、BIZ-05 の D2 の入力だけ）、公開する支払の集計（BIZ-05 が足した結果）、Z001 の単純な写像の 3 つに分けて書き直した。手順 6 の DB DTO の命名の例を、無くなる `OfficialDailyPaymentRow` から `OfficialDailyDepartmentRow` に替えた。 |
