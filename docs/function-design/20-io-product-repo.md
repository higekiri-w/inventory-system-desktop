## 2. IO-01: SQLiteデータアクセス層

### 計数の置き場と在庫の求め直しの契約（D-118。(a)・b1・b2 は実装済み）

[棚卸しの数の置き場と在庫の求め直しの ADR](../adr/2026-10-16-stocktake-count-placement.md)（SPEC-STK-CNT-D2・D9）の IO-01 の側。(a)（ADR SPEC-STK-CNT-D12）で入ったのは、`ProductUpdates` から `stock_quantity` を除いたことと、`find_all_stock_quantities` が写しの 4 項目を `StockCopyRow` で返すこと（§2.11 の同名の項）。b1（[Plan Packet](../archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）で入ったのは、stocktake_repo の明細・完了・記録の詳細の読取り（§2.11・§2.11a）と共有 JAN の読取り。商品の読取りが写しの 3 列を返すことは b2（[Plan Packet](../plans/2026-10-23-stocktake-count-runtime-b2.md)）で入った。保存の列・CHECK・FK は [tracking](../db-design/tracking-system-tables.md)・[master](../db-design/master-tables.md) の同名の契約を正とする。旧 ADR（時点証拠）の版・非連動化の記録・計数の開始の読取り・source / ledger の上限・measured の保存・再実測・保存要求の照会・flag・有効観測の列挙・pending の補正の repo 操作は持たない（置き換えた）。

| repo 操作 | 処理・返却の契約 |
|---|---|
| product_repo の更新型 | `ProductUpdates` から `stock_quantity` を除く（(a)、実装済み。数量の更新は [21](21-io-inventory-repo.md) の `update_stock_quantity` だけ）。商品の読取りの 4 つ（`find_by_product_code`・`search_products`・`get_stock_detail`・`list_low_stock_products`。`ProductWithRelations` を返す関数）は `stock_recheck`・`stock_range_low`・`stock_range_high` を返す（b2。SELECT の `plu_memory_no` の後の 3 列で、`get_stock_detail` の最終入庫日・最終販売日はその後ろ）。`Product` を返す読取り（PLU の書出し・JAN の照合）は写しの 3 列を返さない |
| stocktake_repo の明細の読取り | `get_stocktake_items`・`find_stocktake_item` は、明細を参照する計数（`stock_counts.stocktake_item_id`）の効く値と置き場の日付、写しの在庫・範囲を返す。差異と要再確認の理由は BIZ が求める |
| 計数の読取り・保存 | [21](21-io-inventory-repo.md) の新契約（`insert_stock_count` と、商品の計数の鍵の順の一覧） |
| 共有 JAN の読取り（b1） | `list_shared_jan_conflicts(conn) -> Result<Vec<(String, Vec<String>)>, DbError>`（`pub(crate)`）: 在庫連動の商品と同じ JAN を別の商品が持つ組を JAN の昇順、商品コードの昇順で返す（[36](36-biz-integrity-check.md) の共有 JAN の違反の一覧）。変更の後の状態の検査（BIZ-01）は今の `find_by_jan_code` で行う |

SQL の失敗・整数の範囲外・FK 違反は DbError として返し、IO で前後や回復の可否を決めない。

---

### 2.1 モジュール構成

```
src-tauri/src/
  db/
    mod.rs          -- DB接続管理、初期化
    product_repo.rs -- products, product_jan_aliases（proposed、D-122）, departments, suppliers, price_history
    inventory_repo.rs -- inventory_movements, receiving_*, return_*, manual_sale_*, disposal_*
    sales_repo.rs   -- sale_records, csv_imports, csv_import_errors
    stocktake_repo.rs -- stocktakes, stocktake_items
    system_repo.rs  -- operation_logs, app_settings
```

### 2.2 db::init — DB接続初期化

**関数要求**: SQLiteファイルを開き、必要なPRAGMAを設定し、マイグレーションを実行して、使用可能な接続を返す

**シグネチャ**:
```
fn init_database(db_path: &str) -> Result<DbConnection, DbError>
```

**処理ステップ**:
1. db_pathのファイルを開く。存在しなければ新規作成
2. PRAGMA foreign_keys = ON を実行
3. PRAGMA journal_mode = WAL を実行
4. PRAGMA busy_timeout = 5000 を実行
5. MNT-03の migrate() を呼び出し
6. マイグレーション成功 → DbConnectionを返す
7. マイグレーション失敗 → DbError::MigrationFailed(詳細)を返す

**エラーハンドリング**:
- ファイルオープン失敗（パス不正、権限不足）→ DbError::ConnectionFailed(詳細)
- PRAGMA実行失敗 → DbError::PragmaFailed(詳細)
- マイグレーション失敗 → DbError::MigrationFailed(詳細)
- DB の版がアプリの扱える最大より新しい → DbError::SchemaNewerThanApp { db_version, app_max }（migrate が論理的な書込みの前に拒否。22 MNT-03-D11）

---

### 2.3 product_repo — 商品リポジトリ

#### find_by_product_code

**関数要求**: product_codeで商品を1件取得する。部門名・取引先名もJOINで取得する

**シグネチャ**:
```
fn find_by_product_code(conn: &DbConnection, product_code: &str) -> Result<Option<ProductWithRelations>, DbError>
```

**処理ステップ**:
1. SQL実行: SELECT p.*, d.name as dept_name, s.name as supplier_name FROM products p LEFT JOIN departments d ON p.department_id = d.id LEFT JOIN suppliers s ON p.supplier_id = s.id WHERE p.product_code = ?
2. 結果が0行 → Ok(None)を返す
3. 結果が1行 → ProductWithRelations構造体にマッピングしてOk(Some(...))を返す

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### search_products

**関数要求**: 検索条件に基づいて商品一覧をページング取得する

**シグネチャ**:
```
fn search_products(conn: &DbConnection, query: &ProductSearchQuery) -> Result<PaginatedResult<ProductWithRelations>, DbError>
```

**ProductSearchQuery構造体**:
- keyword: Option<String>（商品名、product_code、jan_code、maker_codeの部分一致。SPEC-PRV-D2）
- department_id: Option<i64>
- supplier_id: Option<i64>（`None` なら取引先条件なし）
- include_unassigned: bool（`supplier_id` 指定時だけ有効。true なら未設定商品も含める）
- is_discontinued: Option<bool>（Noneなら全件、Some(false)なら現行品のみ）
- sort_key: SortKey（Name / ProductCode / StockQuantity / SellingPrice）
- sort_order: SortOrder（Asc / Desc）
- page: u32（1始まり）
- per_page: u32（1以上を要求し、D-031 の `PAGINATION_MAX_PER_PAGE = 200` で200上限クランプ）

**処理ステップ**:
1. page / per_page の入力ガードを行う（page >= 1、per_page >= 1）。per_page は D-031 の `PAGINATION_MAX_PER_PAGE = 200` でクランプする。(page - 1) * clamped_per_page の overflow は offset 計算時に DbError::QueryFailed。
2. WHERE句の構築:
   - keywordがSome → `(p.name LIKE '%keyword%' OR p.product_code LIKE '%keyword%' OR p.jan_code LIKE '%keyword%' OR p.maker_code LIKE '%keyword%')`（SPEC-PRV-D2）
   - department_idがSome → `p.department_id = ?`
   - supplier_idがSome → include_unassigned が true なら `(p.supplier_id = ? OR p.supplier_id IS NULL)`、false なら `p.supplier_id = ?`。supplier_id が None なら flag を無視して条件を加えない（SPEC-PRVB-D1）
   - is_discontinuedがSome → `p.is_discontinued = ?`
2. COUNT(*) でtotal_countを取得
3. ORDER BY句の構築（sort_key + sort_order）
4. LIMIT clamped_per_page OFFSET (page - 1) * clamped_per_page で取得
5. PaginatedResult { items, total_count, page, per_page: clamped_per_page } を返す

PLU の移行 filter（`plu: Option<PluMigrationFilter>` と WHERE の PLU 条件）と、`ProductWithRelations` の `plu_slots` を JAN で LEFT JOIN した読取り専用 `plu_memory_no`（IO-01-D4 / SPEC-PLS-D7）は D-117 (2)・(5) で撤去した。runtime の lane が code・bindings・UI・test と一緒に外す。

一括 PLU 対象更新の抽出（SPEC-PLS-D6）も撤去の対象（D-117 (2)）。設計照合 test（`src-tauri/tests/design_compliance_test.rs`）が code の pub fn と突き合わせるので、撤去までは signature だけを残す。

```rust
pub fn find_products_for_bulk_plu_target(
    conn: &DbConnection,
    filter: &ProductBulkFilter,
) -> Result<Vec<Product>, DbError>
```

#### db::plu_slot_repo（IO-01-D4 / SPEC-PLS-D1）

`plu_slots` の ordered lookup と状態遷移だけを担当し、transaction は呼出側 BIZ-04 が所有する。

```rust
pub fn list_slots(conn: &DbConnection) -> Result<Vec<PluSlot>, DbError>
pub fn find_slots_by_scanning_code(conn: &DbConnection, scanning_code: &str) -> Result<Vec<PluSlot>, DbError>
pub fn find_slots_by_status(conn: &DbConnection, status: PluSlotStatus) -> Result<Vec<PluSlot>, DbError>
pub fn find_slot_by_memory_no(conn: &DbConnection, memory_no: i64) -> Result<Option<PluSlot>, DbError>
pub fn find_min_free_slot(conn: &DbConnection) -> Result<Option<PluSlot>, DbError>
pub fn update_slot(conn: &DbConnection, update: PluSlotUpdate<'_>) -> Result<(), DbError>
```

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### insert_product

**関数要求**: 商品を1件INSERTする。product_codeの重複チェックは呼び出し元（BIZ-01）の責務

**シグネチャ**:
```
fn insert_product(conn: &DbConnection, product: &NewProduct) -> Result<(), DbError>
```

**NewProduct構造体**:
- product_code, jan_code, name, department_id, supplier_id, selling_price, cost_price_centi, tax_rate, maker_code, stock_quantity, stock_unit, is_discontinued, plu_dirty, plu_exported_at, pos_stock_sync, plu_target（D-028: スキャニングPLU書出し対象フラグ。値の決定は BIZ-01 の責務）

**処理ステップ**:
1. INSERT INTO products (...) VALUES (...)を実行
2. created_at, updated_atは現在日時を自動セット

**エラーハンドリング**:
- PRIMARY KEY重複（product_code既存）→ DbError::DuplicateKey(product_code)
- 外部キー制約違反（department_id不正）→ DbError::ForeignKeyViolation(詳細)
- その他SQL実行失敗 → DbError::QueryFailed(詳細)

#### update_product

**関数要求**: 商品の指定フィールドを更新する

**シグネチャ**:
```
fn update_product(conn: &DbConnection, product_code: &str, updates: &ProductUpdates) -> Result<bool, DbError>
```

**ProductUpdates構造体**: 全フィールドがOption型。Someのフィールドだけ更新する。NULLableカラム（supplier_id, maker_code, plu_exported_at）は Option\<Option\<T\>\>（None=更新しない、Some(None)=NULLにする、Some(Some(v))=値を更新）
- name: Option\<String\>
- department_id: Option\<i64\>
- supplier_id: Option\<Option\<i64\>\>
- selling_price: Option\<i64\>
- cost_price_centi: Option\<i64\>
- tax_rate: Option\<String\>
- maker_code: Option\<Option\<String\>\>
- stock_unit: Option\<ProductStockUnit\>（D-119。値の検査〈個数の単位どうし〉は BIZ-01-D8、repo は code の文字列で SET するだけ）
- is_discontinued: Option\<bool\>
- plu_dirty: Option\<bool\>
- plu_exported_at: Option\<Option\<String\>\>
- pos_stock_sync: Option\<bool\>
- plu_target: Option\<bool\>（D-028。0→1 遷移時の plu_dirty=1 セットは BIZ-01 の責務）

**処理ステップ**:
1. Someのフィールドだけ SET句に含めるUPDATEを構築
2. updated_at = 現在日時を常に含める
3. WHERE product_code = ? で実行
4. affected_rows == 1 → Ok(true)
5. affected_rows == 0 → Ok(false)（該当商品なし）

`revise_product_price` はこの部分更新契約を使い、`ProductUpdates` には `selling_price` / `cost_price_centi` / 売価変更時だけの `plu_dirty` / 商品の `supplier_id` が NULL の場合だけの `supplier_id` だけを設定する（SPEC-PRV-D5 / SPEC-PRV-D6）。BIZ-01 が旧値との比較と transaction を所有し、IO は全項目更新や業務判断を行わない。

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### find_by_jan_code

**関数要求**: jan_codeで商品を検索する。複数ヒットの可能性あり（グループコード）

**シグネチャ**:
```
fn find_by_jan_code(conn: &DbConnection, jan_code: &str) -> Result<Vec<Product>, DbError>
```

**処理ステップ**:
1. SELECT * FROM products WHERE jan_code = ? ORDER BY product_code ASC
2. 結果をVecで返す（0件ならVec空）

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### 前の JAN の別名（IO-01-D5、proposed・未実装、D-122）

`product_jan_aliases`（[master-tables](../db-design/master-tables.md) §1a）の読取りと追加。transaction は呼出側（BIZ-01 の商品 CSV の上書き、BIZ-03 の引き当て）が所有する。

```rust
fn find_jan_alias(conn: &DbConnection, jan_code: &str) -> Result<Option<String>, DbError>
fn insert_jan_alias(conn: &DbConnection, old_jan: &str, product_code: &str) -> Result<(), DbError>
```

- `find_jan_alias`: `SELECT product_code FROM product_jan_aliases WHERE old_jan = ?1`。比べ方は `find_by_jan_code` と同じ文字列の一致（JAN を数に変えない）。無ければ `None`。
- `insert_jan_alias`: `created_at` に現在日時を入れて 1 行 INSERT する。更新・削除の関数は置かない（消すのは `products` の trigger だけ）。
- `insert_product`・`update_product` は別名を扱わない。`products.jan_code` を書くと、DB の trigger が同じ JAN の別名を同じ transaction の中で消す（master-tables §1a。IO の関数は何もしない）。

**エラーハンドリング**:
既存の `From<rusqlite::Error>`（`src-tauri/src/db/mod.rs` の DbError の変換）のまま、新しい変換を足さない:
- 別名の表の trigger の拒否（`RAISE(ABORT)`、その JAN を持つ商品がある）・SQL 実行失敗 → DbError::QueryFailed(詳細)（BIZ-01 が先に条件を確かめるので、trigger の拒否は通常は起きない）
- 外部キー違反（787、商品が無い）→ DbError::ForeignKeyViolation(SQLite の message)
- PRIMARY KEY 重複（1555）→ DbError::DuplicateKey(SQLite の message)（不変条件により、その JAN を持つ商品が直前まであった別名は無いので、通常は起きない）

#### find_plu_dirty_products

**関数要求**: plu_target=1 かつ plu_dirty=1 の商品一覧を返す（UI-08 差分プレビューと UI-00 PLU未反映通知の共通ソース。D-028 により plu_target 条件を追加）

**シグネチャ**:
```
fn find_plu_dirty_products(conn: &DbConnection) -> Result<Vec<Product>, DbError>
```

**処理ステップ**:
1. SQL実行: SELECT p.* FROM products p WHERE p.plu_dirty = 1 AND p.plu_target = 1 ORDER BY p.product_code ASC
2. 結果をVec<Product>にマッピング（Product は p.* マッピングのため plu_target カラムを含む）
3. 0件でもOk(空Vec)を返す（エラーではない）

**入力例**: なし（引数なし）
**出力例**: `[Product { product_code: "4976383262108", name: "ﾊﾏﾅｶ ｱﾐｱﾐ極太", plu_dirty: true, ... }]`

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### find_plu_dirty_products_for_plu

**関数要求**: plu_target=1 かつ plu_dirty=1 の商品一覧を部門名付きで返す（PLU書出しの差分モード、IO-04入力用。D-028 により plu_target 条件を追加）

**シグネチャ**:
```
fn find_plu_dirty_products_for_plu(conn: &DbConnection) -> Result<Vec<ProductForPlu>, DbError>
```

**処理ステップ**:
1. SQL実行: SELECT p.*, d.name as department_name FROM products p INNER JOIN departments d ON p.department_id = d.id WHERE p.plu_dirty = 1 AND p.plu_target = 1 ORDER BY p.product_code ASC
2. 結果をVec<ProductForPlu>にマッピング（ProductForPlu = Product + department_name: String）
3. 0件でもOk(空Vec)を返す

**設計判断**: INNER JOIN を使用（FK制約で departments は必ず存在）。department_name は現在値（履歴ではない。PLU書出しはレジ登録用のため時点固定不要）。既存の find_plu_dirty_products も D-028 で `plu_target = 1` 条件を持つ（UI-00 通知の scope 限定のため）。`_for_plu` との差は department_name 付きかどうかだけ。

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### find_active_products

**関数要求**: 有効な（廃番でない）商品一覧を返す（PLU書出しの全件モード用）

**シグネチャ**:
```
fn find_active_products(conn: &DbConnection) -> Result<Vec<Product>, DbError>
```

**処理ステップ**:
1. SQL実行: SELECT p.* FROM products p WHERE p.is_discontinued = 0 ORDER BY p.product_code ASC
2. 結果をVec<Product>にマッピング
3. 0件でもOk(空Vec)を返す

#### find_active_products_for_plu

**関数要求**: PLU対象の有効な商品一覧を部門名付きで返す（PLU書出しの全件モード、IO-04入力用。D-028 により plu_target 条件を追加）

**シグネチャ**:
```
fn find_active_products_for_plu(conn: &DbConnection) -> Result<Vec<ProductForPlu>, DbError>
```

**処理ステップ**:
1. SQL実行: SELECT p.*, d.name as department_name FROM products p INNER JOIN departments d ON p.department_id = d.id WHERE p.is_discontinued = 0 AND p.plu_target = 1 ORDER BY p.product_code ASC
2. 結果をVec<ProductForPlu>にマッピング
3. 0件でもOk(空Vec)を返す

**設計判断**: find_plu_dirty_products_for_plu と同じ方針。INNER JOIN、department_name は現在値。既存の find_active_products（商品一覧用）は変更しない。

**定義**: PLU書出しの "対象" = `is_discontinued = 0 AND plu_target = 1`（D-028 三分バケットの「対象外」= plu_target=0 はここで除外される）。

**入力例**: なし
**出力例**: `[Product { product_code: "4976383262108", is_discontinued: false, ... }, ...]`

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

#### list_product_codes_by_prefix

**関数要求**: product_code が指定の文字列で始まる商品の product_code を全件返す。独自コード発番で既存の番号を数えるため（30-biz §4.3、BIZ-01-D6）。番号の読み取りはしない（BIZ が持つ）

**シグネチャ**:
```
fn list_product_codes_by_prefix(conn: &DbConnection, prefix: &str) -> Result<Vec<String>, DbError>
```

**処理ステップ**:
1. SQL実行: SELECT product_code FROM products WHERE substr(product_code, 1, length(?1)) = ?1（`LIKE` は使わない。ASCII の大文字・小文字を同一視し、`%` `_` を特殊文字として扱うため）。廃番を含む全商品
2. 0 行 → 空の Vec

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

### 2.4 product_repo — 部門リポジトリ（同ファイル内）

#### find_department_by_id

**シグネチャ**:
```
fn find_department_by_id(conn: &DbConnection, id: i64) -> Result<Option<Department>, DbError>
```

#### list_departments

**シグネチャ**:
```
fn list_departments(conn: &DbConnection) -> Result<Vec<Department>, DbError>
```

#### increment_next_seq

**関数要求**: departmentsのnext_seqを+1して、インクリメント前の値を返す。独自コード発番用

**シグネチャ**:
```
fn increment_next_seq(conn: &DbConnection, department_id: i64) -> Result<i64, DbError>
```

**処理ステップ**:
1. SELECT next_seq FROM departments WHERE id = ? でcurrent_seqを取得
2. 該当なし → DbError::NotFound
3. UPDATE departments SET next_seq = next_seq + 1 WHERE id = ?
4. current_seq（インクリメント前の値）を返す

**注意**: この関数はトランザクション内で呼ばれることを前提とする（BIZ-01の独自コード発番処理内）。step 3 の SQL は next_seq が i64::MAX のとき next_seq を REAL にするため、呼び出し元の BIZ が先に止める（30-biz §4.3 step 3d）

#### raise_next_seq

**関数要求**: departments の next_seq が指定の値より小さいときだけ、その値へ上げる。下げない。独自コード発番で既存の番号を飛ばすため（30-biz §4.3、BIZ-01-D6）

**シグネチャ**:
```
fn raise_next_seq(conn: &DbConnection, department_id: i64, at_least: i64) -> Result<(), DbError>
```

**処理ステップ**:
1. UPDATE departments SET next_seq = MAX(next_seq, ?2) WHERE id = ?1
2. 変更行数 0 → DbError::NotFound

**注意**: increment_next_seq と同じく、トランザクション内で呼ばれることを前提とする（BIZ-01の独自コード発番処理内）

---

### 2.5 product_repo — 取引先リポジトリ（同ファイル内）

#### list_suppliers

**シグネチャ**:
```
fn list_suppliers(conn: &DbConnection) -> Result<Vec<Supplier>, DbError>
```

#### find_or_create_supplier

**関数要求**: 名前で取引先を検索し、なければ作成して返す。サジェスト入力で新規追加する場合に使用（SPEC-PRV-D6）

**シグネチャ**:
```
fn find_or_create_supplier(conn: &DbConnection, name: &str) -> Result<Supplier, DbError>
```

**処理ステップ**:
1. `name.trim()` で前後空白を除去し、空文字なら SQL を実行せず入力エラーとして返す
2. SELECT * FROM suppliers WHERE name = ?（trim 後の値）
3. ヒット → そのSupplierを返す
4. ヒットなし → INSERT INTO suppliers (name, created_at) VALUES (?, ?) → 作成したSupplierを返す

#### rename_supplier（SPEC-SUP-D3）

**関数要求**: BIZ-01 で検証済みの取引先名へ更新し、更新後の取引先を返す。業務 validation と transaction は BIZ が所有し、IO は SQL 実行だけを担う。

**シグネチャ**:
```rust
fn rename_supplier(
    conn: &DbConnection,
    supplier_id: i64,
    name: &str,
    updated_at: &str,
) -> Result<Supplier, DbError>
```

**処理ステップ**:
1. `UPDATE suppliers SET name = ?, updated_at = ? WHERE id = ?` を実行する
2. 更新件数が 0 件なら not-found 相当として BIZ が判定できる結果を返す
3. 更新後の `Supplier` を返す

#### merge_suppliers（SPEC-SUP-D4）

**関数要求**: 重複した取引先の参照を残す側へ付け替え、消す側の行を削除する。BIZ-01 が開始した 1 transaction の中で呼び、IO 自身は commit しない。

**シグネチャ**:
```rust
fn merge_suppliers(
    conn: &DbConnection,
    source_id: i64,
    target_id: i64,
) -> Result<(i64, i64), DbError>
```

**処理ステップ**:
1. `UPDATE products SET supplier_id = ? WHERE supplier_id = ?` を実行し、付け替えた商品件数を保持する
2. `UPDATE receiving_records SET supplier_id = ? WHERE supplier_id = ?` を実行し、付け替えた入庫記録件数を保持する
3. `DELETE FROM suppliers WHERE id = ?` で source 行を削除する
4. `(products_updated, receiving_records_updated)` を返す

products と receiving_records の 2 UPDATE を両方完了してから DELETE する。片方だけを更新する経路、参照が残ったまま削除する経路、単独削除関数は設けない。

#### list_suppliers_with_usage（SPEC-SUP-D8）

**関数要求**: UI-15 の一覧用に、取引先全件と関連商品数・入庫記録数を name 昇順で返す。0 件の取引先も一覧から落とさない。

**シグネチャ**:
```rust
fn list_suppliers_with_usage(
    conn: &DbConnection,
) -> Result<Vec<SupplierUsageRow>, DbError>
```

**SupplierUsageRow（IO 内部型）**:
- supplier: Supplier
- product_count: i64
- receiving_record_count: i64

**処理ステップ**:
1. suppliers を基点に、products と receiving_records をそれぞれ supplier_id で COUNT する
2. 2 つの参照表を同時 JOIN して件数を水増ししないよう、独立した集約結果または相関 subquery で数える
3. `ORDER BY suppliers.name ASC` で全件を返す

IO / BIZ / CMD の公開関数名は `list_suppliers_with_usage` に統一し、用途別の別名は作らない。wire DTO `SupplierWithUsage` への変換は BIZ-01 が所有する。

---

### 2.6 product_repo — 価格履歴リポジトリ（同ファイル内）

#### insert_price_history

**シグネチャ**:
```
fn insert_price_history(conn: &DbConnection, history: &NewPriceHistory) -> Result<(), DbError>
```

**NewPriceHistory構造体**:
- product_code, old_selling, new_selling, old_cost_centi, new_cost_centi

**処理ステップ**:
1. INSERT INTO price_history (..., changed_at) VALUES (..., 現在日時)

#### list_price_history（SPEC-PRV-D9）

**関数要求**: 商品コードに紐付く価格履歴を新しい順で取得する

**シグネチャ**:
```rust
fn list_price_history(
    conn: &DbConnection,
    product_code: &str,
    limit: u32,
) -> Result<Vec<PriceHistoryEntry>, DbError>
```

**PriceHistoryEntry構造体**:
- id, old_selling_price, new_selling_price, old_cost_price_centi, new_cost_price_centi, changed_at

**処理ステップ**:
1. `limit` は呼出し既定 10、上限 100 とし、100 超は 100 に丸める
2. `WHERE product_code = ? ORDER BY changed_at DESC, id DESC LIMIT ?` で取得する
3. `product_code` が存在しない場合もエラーにせず空配列を返す

---

### 2.7 inventory_repo — 在庫変動リポジトリ（第2段階で必要な関数のみ）

#### insert_movement

**関数要求**: inventory_movementsに1行INSERTする。全ての在庫変動記録の共通入口

**シグネチャ**:
```
fn insert_movement(conn: &DbConnection, movement: &NewMovement) -> Result<i64, DbError>
```

**NewMovement構造体**:
- product_code, movement_type, quantity, stock_after, reference_type, reference_id, note

**実装時の差分**: movement_type と reference_type は DB_DESIGN.md の CHECK 制約値が固定のため、
実装では String ではなく MovementType / ReferenceType enum を使用。as_str() で同じ文字列に変換
されるため振る舞いは同一。typo によるランタイムエラーをコンパイル時に検出する目的。

**処理ステップ**:
1. INSERT INTO inventory_movements (..., is_voided, created_at) VALUES (..., 0, 現在日時)
2. 挿入されたIDを返す

---

### 2.8 system_repo — 操作ログ・アプリ設定リポジトリ

#### insert_operation_log

**シグネチャ**:
```
fn insert_operation_log(conn: &DbConnection, log: &NewOperationLog) -> Result<(), DbError>
```

**NewOperationLog構造体**:
- operation_type: String, summary: String, detail_json: Option<String>

**処理ステップ**:
1. INSERT INTO operation_logs (..., created_at) VALUES (..., 現在日時)

#### get_setting（第6段階追加）

**関数要求**: app_settingsからキーに対応する値を取得する

**シグネチャ**:
```
fn get_setting(conn: &DbConnection, key: &str) -> Result<Option<String>, DbError>
```

**処理ステップ**:
1. `SELECT value FROM app_settings WHERE key = ?1`
2. 行あり → `Some(value)` を返す
3. 行なし → `None` を返す

#### get_all_settings（第6段階追加）

**関数要求**: app_settingsの全キー・値ペアを取得する

**シグネチャ**:
```
fn get_all_settings(conn: &DbConnection) -> Result<Vec<AppSetting>, DbError>
```

**AppSetting構造体**:
```
#[derive(Debug, serde::Serialize)]
struct AppSetting {
    key: String,
    value: String,
    updated_at: String,
}
```

**処理ステップ**:
1. `SELECT key, value, updated_at FROM app_settings ORDER BY key`
2. 各行を `AppSetting` に変換して `Vec` に格納

#### upsert_setting（第6段階追加）

**関数要求**: app_settingsにキー・値を挿入または更新する

**シグネチャ**:
```
fn upsert_setting(conn: &DbConnection, key: &str, value: &str) -> Result<(), DbError>
```

**処理ステップ**:
1. `INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3`
2. updated_atは現在日時（ISO 8601）

#### list_operation_logs（第6段階追加、UI-11c-D2/D3 で期間 predicate 拡張）

**関数要求**: 操作ログをページング付きで取得する。オプションでoperation_typeフィルタ、オプションで期間（JST暦日）フィルタ

**シグネチャ**:
```
fn list_operation_logs(
    conn: &DbConnection,
    page: u32,
    per_page: u32,
    operation_type: Option<&str>,
    start_date: Option<&str>,  // UI-11c-D2。YYYY-MM-DD。呼び出し元（CMD層）が形式検証済み
    end_date: Option<&str>,    // UI-11c-D2。YYYY-MM-DD。呼び出し元（CMD層）が形式検証済み・start<=endを保証済み
) -> Result<PaginatedResult<OperationLog>, DbError>
```

**OperationLog構造体**:
```
#[derive(Debug, serde::Serialize)]
struct OperationLog {
    id: i64,
    operation_type: String,
    summary: String,
    detail_json: Option<String>,
    created_at: String,
}
```

**処理ステップ**:
1. page/per_page のバリデーション（page >= 1、per_page >= 1）を行い、per_page は D-031 の `PAGINATION_MAX_PER_PAGE = 200` でクランプする。
2. `page` / clamp後の`per_page`を`i64`へ変換してから、`offset = (page_i64 - 1) * per_page_i64`を計算する。`u32`上で乗算しないため、`u32::MAX`のpositive pageでもpanic/wrapせずSQLiteへ範囲外offsetとして渡り、空の`items`を返す。
3. WHERE句の構築（すべて `AND` で連結、該当条件がなければ句を追加しない）:
   - `operation_type` が `Some` → `operation_type = ?`
   - `start_date` が `Some` → `created_at >= '{start_date}T00:00:00'`
   - `end_date` が `Some` → `created_at < '{end_date + 1日}T00:00:00'`（`end_date` をパースし1日加算した日付文字列を使う。JST暦日の end-next-day exclusive predicate、UI-11c-D2）
4. `SELECT id, operation_type, summary, detail_json, created_at FROM operation_logs [WHERE ...] ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?`
5. `SELECT COUNT(*) FROM operation_logs [WHERE ...]` で、**手順3と全く同じ WHERE句・同じパラメータ**を使って総件数取得する（row query と count query の predicate 同一性、UI-11c-D3。片方だけ期間条件を落とすと pagination の total_count と実際の表示件数が矛盾する）。
6. `start_date` / `end_date` が両方 `None` の場合は既存動作（フィルタなし）を完全維持する。既存呼び出し元・既存テストへの影響はない。
7. `PaginatedResult<OperationLog>` を返す

**設計判断（UI-11c-D2）**: 本関数は JST暦日の inclusive/exclusive 境界（end は翌日00:00:00未満）を採用する。操作ログは監査用途のため保存の時刻の境界の厳密さを優先する（詳細: [74-ui-operation-logs.md](../function-design/74-ui-operation-logs.md) §74.4.3、`docs/decision-log.md` D-037）。在庫変動の履歴は D-118 の b2 で保存の時刻でなく業務日（`YYYY-MM-DD`）の両端を含む比べに変わった（[44](44-cmd-inventory.md) の `list_movements`）。

#### find_distinct_operation_types（新規、UI-11c-D4）

**関数要求**: 保持中の operation_logs 全体から distinct な operation_type を返す。現在ページや現在の filter 済み結果からの生成を避け、フィルタ選択肢の完全なソースにする（[74-ui-operation-logs.md](../function-design/74-ui-operation-logs.md) §74.5）。

**シグネチャ**:
```
fn find_distinct_operation_types(conn: &DbConnection) -> Result<Vec<String>, DbError>
```

**処理ステップ**:
1. SQL実行: `SELECT DISTINCT operation_type FROM operation_logs ORDER BY operation_type ASC`
2. 結果を `Vec<String>` にマッピング
3. 0件でも `Ok(空Vec)` を返す（エラーではない）

**入力例**: なし（引数なし）
**出力例**: `["backup_create", "product_create", "receiving_create"]`

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### delete_old_logs（第6段階追加）

**関数要求**: 指定日数を超えた古い操作ログを削除する

**シグネチャ**:
```
fn delete_old_logs(conn: &DbConnection, retention_days: u32) -> Result<usize, DbError>
```

戻り値: 削除した行数

**処理ステップ**:
1. カットオフ日を計算: `chrono::Local::now().date_naive().checked_sub_days(chrono::Days::new(u64::from(retention_days)))`
   - `None`（日付の範囲の外。それより前の記録は無い）→ 何も消さず `Ok(0)` を返す（panic しない。[72](72-mnt-log-manager.md) §72.4 の手順 4、MNT-02-D1）
2. カットオフ日をISO 8601の日付開始時刻に変換: `{YYYY-MM-DD}T00:00:00`
3. `DELETE FROM operation_logs WHERE created_at < ?1`
4. `conn.changes()` で削除行数を取得して返す

---

### 2.9 stocktake_repo — 棚卸しリポジトリ（第2段階で必要な関数のみ）

#### find_active_stocktake

**関数要求**: status='in_progress'の棚卸しを取得する。BIZ-01の商品登録で棚卸し中チェックに使用

**シグネチャ**:
```
fn find_active_stocktake(conn: &DbConnection) -> Result<Option<Stocktake>, DbError>
```

**処理ステップ**:
1. SELECT * FROM stocktakes WHERE status = 'in_progress' ORDER BY started_at DESC, id DESC LIMIT 1
   （複数 in_progress が存在する不整合時にも返却を決定的にするため ORDER BY を明示）

#### insert_stocktake_item

**関数要求**: 棚卸し明細を1行追加する。棚卸し中の新規商品登録時に呼ばれる

**シグネチャ**:
```
fn insert_stocktake_item(conn: &DbConnection, item: &NewStocktakeItem) -> Result<(), DbError>
```

---

### 2.10 sales_repo — 売上集計クエリ（BIZ-05 用）

#### get_daily_sales_records

**関数要求**: 指定日の売上レコードを商品名・部門名付きで取得する。is_voided=0 のみ

**シグネチャ**:
```
fn get_daily_sales_records(conn: &DbConnection, date: &str) -> Result<Vec<DailySaleRow>, DbError>
```

**DailySaleRow構造体**:
- product_code: String, name: String, department_name: String, department_id: i64, quantity: i64, amount: i64, source: String

**処理ステップ**:
1. SQL実行: SELECT sr.product_code, p.name, d.name as department_name, d.id as department_id, sr.quantity, sr.amount, sr.source FROM sale_records sr INNER JOIN products p ON sr.product_code = p.product_code INNER JOIN departments d ON p.department_id = d.id WHERE sr.sale_date = ? AND sr.is_voided = 0 ORDER BY d.id ASC, p.product_code ASC
2. 結果を Vec<DailySaleRow> にマッピング。0件でも Ok(空Vec)

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### get_monthly_sales_by_product

**関数要求**: 指定期間の商品別売上集計を取得する

**シグネチャ**:
```
fn get_monthly_sales_by_product(conn: &DbConnection, date_from: &str, date_to: &str) -> Result<Vec<MonthlySaleProductRow>, DbError>
```

**MonthlySaleProductRow構造体**:
- product_code: String, name: String, quantity: i64, amount: i64

**処理ステップ**:
1. SQL実行: SELECT sr.product_code, p.name, SUM(sr.quantity) as quantity, SUM(sr.amount) as amount FROM sale_records sr INNER JOIN products p ON sr.product_code = p.product_code WHERE sr.sale_date >= ? AND sr.sale_date <= ? AND sr.is_voided = 0 GROUP BY sr.product_code, p.name ORDER BY SUM(sr.amount) DESC
2. 結果を Vec にマッピング。0件でも Ok(空Vec)

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### get_monthly_sales_by_department

**関数要求**: 指定期間の部門別売上集計を取得する

**シグネチャ**:
```
fn get_monthly_sales_by_department(conn: &DbConnection, date_from: &str, date_to: &str) -> Result<Vec<MonthlySaleDeptRow>, DbError>
```

**MonthlySaleDeptRow構造体**:
- department_id: i64, department_name: String, quantity: i64, amount: i64

**処理ステップ**:
1. SQL実行: SELECT d.id as department_id, d.name as department_name, SUM(sr.quantity) as quantity, SUM(sr.amount) as amount FROM sale_records sr INNER JOIN products p ON sr.product_code = p.product_code INNER JOIN departments d ON p.department_id = d.id WHERE sr.sale_date >= ? AND sr.sale_date <= ? AND sr.is_voided = 0 GROUP BY d.id, d.name ORDER BY SUM(sr.amount) DESC
2. 結果を Vec にマッピング。0件でも Ok(空Vec)

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)

---

### 2.11 stocktake_repo — 棚卸しリポジトリ拡張（BIZ-06 用）

#### insert_stocktake

**関数要求**: stocktakes ヘッダを1行INSERTし、挿入されたIDを返す

**シグネチャ**:
```
fn insert_stocktake(conn: &DbConnection, started_at: &str) -> Result<i64, DbError>
```

**処理ステップ**:
1. INSERT INTO stocktakes (started_at, status) VALUES (?, 'in_progress')
2. last_insert_rowid() を返す

#### find_stocktake_by_id

**シグネチャ**:
```
fn find_stocktake_by_id(conn: &DbConnection, id: i64) -> Result<Option<Stocktake>, DbError>
```

#### complete_stocktake

**関数要求**: 棚卸しを完了状態に更新する

**シグネチャ**:
```
fn complete_stocktake(conn: &DbConnection, id: i64, total_cost: i64, completed_at: &str, cutoff: StocktakeCutoff) -> Result<(), DbError>
```

**StocktakeCutoff構造体**（完了の時点のカーソル。[tracking](../db-design/tracking-system-tables.md)、ADR D6）:
- import_id: Option\<i64\>（`MAX(csv_imports.id)`、取り消したものを含む。無ければ None）, count_id: Option\<i64\>（`MAX(stock_counts.id)`）, movement_id: Option\<i64\>（`MAX(inventory_movements.id)`、無ければ 0）

**処理ステップ**:
1. UPDATE stocktakes SET status = 'completed', total_cost = ?, completed_at = ?, cutoff_import_id = ?, cutoff_count_id = ?, cutoff_movement_id = ? WHERE id = ? AND status = 'in_progress'
2. affected_rows == 0 → DbError::NotFound（該当棚卸しが存在しないか完了済み）

#### list_stocktake_items

**関数要求**: 棚卸しアイテムをページング取得。商品名・部門名付き

**シグネチャ**:
```
fn list_stocktake_items(conn: &DbConnection, stocktake_id: i64, department_id: Option<i64>, counted_only: Option<bool>, page: u32, per_page: u32) -> Result<PaginatedResult<StocktakeItemRow>, DbError>
```

**StocktakeItemRow構造体**（IO の行。wire の `StocktakeItemDetail` は BIZ-06 が効く計数・差異・理由を足して作る。[35](35-biz-stocktake-service.md) の「読取り」）:
- id: i64, stocktake_id: i64, product_code: String, name: String, department_name: String, system_stock: i64, actual_count: Option<i64>, counted_at: Option<String>（明細の表示用の写し）, current_stock: i64（動的: products.stock_quantity）, is_discontinued: bool（products.is_discontinued）, stock_unit: ProductStockUnit, stock_range_low・stock_range_high: Option<i64>（写しの範囲）

**処理ステップ**:
1. WHERE句構築: department_id → p.department_id = ?、counted_only=true → 明細を参照する計数がある（`EXISTS(SELECT 1 FROM stock_counts sc WHERE sc.stocktake_item_id = si.id)`）、counted_only=false → 無い（「計数済み」の定義。35）
2. COUNT + LIMIT/OFFSET のページングパターン

#### find_stocktake_item_with_parent_status

**関数要求**: 棚卸し明細を親ヘッダのステータス付きで取得する。計数の保存（`save_stock_count`）の明細の確かめ（進行中か・商品が同じか）に使用

**シグネチャ**:
```
fn find_stocktake_item_with_parent_status(conn: &DbConnection, item_id: i64) -> Result<Option<(StocktakeItem, String)>, DbError>
```

**処理ステップ**:
1. SELECT si.*, st.status FROM stocktake_items si INNER JOIN stocktakes st ON si.stocktake_id = st.id WHERE si.id = ?
2. 結果なし → Ok(None)、あり → Ok(Some((item, status)))

#### update_stocktake_item_count

**シグネチャ**:
```
fn update_stocktake_item_count(conn: &DbConnection, item_id: i64, actual_count: i64, counted_at: &str) -> Result<bool, DbError>
```

#### update_stocktake_item_valuation

**シグネチャ**:
```
fn update_stocktake_item_valuation(conn: &DbConnection, item_id: i64, valuation_cost_price_centi: i64) -> Result<(), DbError>
```

#### count_uncounted_items

**関数要求**: 未計数（明細を参照する計数が無い）のアイテム数を返す

**シグネチャ**:
```
fn count_uncounted_items(conn: &DbConnection, stocktake_id: i64) -> Result<i64, DbError>
```

#### list_uncounted_items

**関数要求**: 未計数の棚卸し明細一覧を返す（完了の `uncounted` の理由と「0 にする」の明細の確かめ。35）

**シグネチャ**:
```
fn list_uncounted_items(conn: &DbConnection, stocktake_id: i64) -> Result<Vec<UncountedItem>, DbError>
```

#### list_items_for_completion

**関数要求**: 完了の検査と評価に使う明細を id の順で取得する（35「完了」。`pub(crate)`）

**シグネチャ**:
```
pub(crate) fn list_items_for_completion(conn: &DbConnection, stocktake_id: i64) -> Result<Vec<StocktakeItemForCompletion>, DbError>
```

**StocktakeItemForCompletion構造体**:
- id: i64, product_code: String, counted: bool（明細を参照する計数がある）, is_discontinued: bool, stock_quantity: i64, stock_recheck: bool, cost_price_centi: i64, stock_unit: ProductStockUnit

#### 回復先と要再確認の一覧の読取り（b1、`pub(crate)`）

```
pub(crate) fn find_active_item_for_product(conn: &DbConnection, product_code: &str) -> Result<Option<(i64, i64)>, DbError> // 進行中の棚卸しのその商品の明細（明細 ID・棚卸し ID）。40 の回復先
pub(crate) fn find_stock_copy(conn: &DbConnection, product_code: &str) -> Result<Option<StockCopyRow>, DbError> // 商品の写し（名前・単位・在庫・要再確認の印と範囲）
pub(crate) fn list_recheck_product_codes(conn: &DbConnection) -> Result<Vec<String>, DbError> // 写しが要再確認の商品コード（商品コードの順。35「読取り」）
```

#### get_stocktake_progress

**関数要求**: 棚卸しの進捗（入力済み/未入力/合計）を返す

**シグネチャ**:
```
fn get_stocktake_progress(conn: &DbConnection, stocktake_id: i64) -> Result<StocktakeProgress, DbError>
```

**StocktakeProgress構造体**:
- total_items: i64, counted_items: i64, uncounted_items: i64

#### find_stocktake_eligible_products

**関数要求**: 棚卸し対象商品を全件取得する（フィルタなし。architecture/biz-task-specs.md BIZ-06 ステップ3+4 の和集合＝全商品）

**シグネチャ**:
```
fn find_stocktake_eligible_products(conn: &DbConnection) -> Result<Vec<ProductForStocktake>, DbError>
```

**ProductForStocktake構造体**:
- product_code: String, stock_quantity: i64, cost_price_centi: i64, is_discontinued: bool

**処理ステップ**:
1. SELECT product_code, stock_quantity, cost_price_centi, is_discontinued FROM products ORDER BY product_code ASC

**設計ノート**: 開始は全ての商品を明細にし、自動の 0 の入力をしない（35。b1）。IO層ではフィルタしない。

#### find_all_stock_quantities

**関数要求**: 全商品の写しの 4 項目（`stock_quantity`・`stock_recheck`・`stock_range_low`・`stock_range_high`）と product_code, name, stock_unit を取得する（BIZ-07 整合性チェック用。[36](36-biz-integrity-check.md)。D-118 の (a)）

**シグネチャ**:
```
fn find_all_stock_quantities(conn: &DbConnection) -> Result<Vec<StockCopyRow>, DbError>
```

**StockCopyRow構造体**:
- product_code: String, name: String, stock_unit: ProductStockUnit, stock_quantity: i64, stock_recheck: bool, stock_range_low: Option\<i64\>, stock_range_high: Option\<i64\>

**処理ステップ**:
1. SELECT product_code, name, stock_unit, stock_quantity, stock_recheck, stock_range_low, stock_range_high FROM products ORDER BY product_code ASC

---

### 2.11a stocktake_repo — 棚卸し記録詳細取得（65 slice 4c 用）

完了は補正の movement を作らない（D-118、b1）。明細ごとの効く計数・差異・ロス原価・完了の後の差は BIZ-06 が完了の時点のカーソルで求める（[35](35-biz-stocktake-service.md) の新契約の「読取り」「完了」）。本節の IO は header・明細・カーソルだけを返す。

#### get_stocktake_record_detail

**関数要求**: 棚卸し記録詳細（ヘッダ、明細、完了の時点のカーソル）を取得する。棚卸し詳細画面（`/stocktake/records/$stocktakeId`、[65-inventory-record-traceability.md](65-inventory-record-traceability.md) §65.3 / §65.5 / §65.10 slice 4c）用。補正の movement は読まない

**シグネチャ**:
```
fn get_stocktake_record_detail(conn: &DbConnection, stocktake_id: i64) -> Result<StocktakeRecordDetailCore, DbError>
```

**StocktakeRecordDetailCore構造体**（IO 内部型。wire DTO は BIZ-06 側 [35-biz-stocktake-service.md](35-biz-stocktake-service.md) §20.6a が所有）:
- header: Stocktake（§2.11 既存型を再利用）
- item_count: i64（stocktake_items 全件数 = 対象商品数）
- items: Vec\<StocktakeRecordItemRow\>（全ての明細、商品コードの順）
- cutoff: StocktakeCutoff（完了の時点のカーソル。進行中は全て None）

**StocktakeRecordItemRow構造体**:
- product_code: String, product_name: String, department_name: String, stock_unit: ProductStockUnit（`parse_stock_unit`、D-113）
- valuation_quantity: Option\<i64\>・valuation_cost_price_centi: Option\<i64\>（完了で固定した評価数量と評価原価〈1/100 円〉。進行中は None。[共通規則](10-common-rules.md) SPEC-UNIT-D5・D11）

**処理ステップ**:
1. stocktakes のヘッダとカーソルを stocktake_id で取得する。存在しない場合は DbError::NotFound
2. stocktake_items の件数を COUNT で取得する（対象商品数）
3. 明細: stocktake_items を products、departments と JOIN し、ORDER BY product_code ASC で返す

**エラーハンドリング**:
- ヘッダ不存在 → DbError::NotFound
- SQL実行失敗 → DbError::QueryFailed(詳細)

#### list_completed_stocktake_cutoffs

**関数要求**: 完了した棚卸しの ID・完了日時・完了の時点のカーソルを返す。BIZ-06 の `completed_stocktakes_comparing_settlement`（[35](35-biz-stocktake-service.md) §20.6a の共有の判定）が、BIZ-03 の取消の前の読取り（[32](32-biz-csv-import-service.md) §15.6b）のために完了の後の比較の範囲を判定する

**シグネチャ**:
```
pub(crate) fn list_completed_stocktake_cutoffs(conn: &DbConnection) -> Result<Vec<(i64, String, StocktakeCutoff)>, DbError>
```

**処理ステップ**:
1. stocktakes のうち `status = 'completed'` の行の id・completed_at・cutoff_import_id・cutoff_count_id・cutoff_movement_id を ORDER BY completed_at, id で返す（書かない）

**エラーハンドリング**:
- SQL実行失敗 → DbError::QueryFailed(詳細)
