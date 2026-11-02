//! 初期スキーマ定義（バージョン1）
//! DB_DESIGN.md の全20テーブル + CHECK制約 + INDEX + 初期データ

/// バージョン1の初期スキーマSQL（20テーブル + インデックス + 初期データ）を返す
pub(crate) fn get_initial_schema() -> &'static str {
    r#"
-- ===========================================
-- マスタテーブル
-- ===========================================

-- 2. departments（部門マスタ）
CREATE TABLE departments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    z005_name TEXT,
    code_prefix TEXT,
    next_seq INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

-- 3. suppliers（取引先マスタ）
CREATE TABLE suppliers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

-- 1. products（商品マスタ）
CREATE TABLE products (
    product_code TEXT PRIMARY KEY,
    jan_code TEXT,
    name TEXT NOT NULL,
    department_id INTEGER NOT NULL REFERENCES departments(id),
    supplier_id INTEGER REFERENCES suppliers(id),
    selling_price INTEGER NOT NULL,
    cost_price INTEGER NOT NULL,
    tax_rate TEXT NOT NULL DEFAULT '10' CHECK(tax_rate IN ('10','8','0')),
    maker_code TEXT,
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    stock_unit TEXT NOT NULL DEFAULT 'pcs' CHECK(stock_unit IN ('pcs','cm')),
    is_discontinued BOOLEAN NOT NULL DEFAULT 0,
    plu_dirty BOOLEAN NOT NULL DEFAULT 1,
    plu_exported_at TEXT,
    pos_stock_sync BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ===========================================
-- トランザクションテーブル
-- ===========================================

-- 4. receiving_records（入庫記録ヘッダ）
CREATE TABLE receiving_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    supplier_id INTEGER REFERENCES suppliers(id),
    receiving_date TEXT NOT NULL,
    note TEXT,
    created_at TEXT NOT NULL
);

-- 5. receiving_items（入庫記録明細）
CREATE TABLE receiving_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    receiving_record_id INTEGER NOT NULL REFERENCES receiving_records(id),
    product_code TEXT NOT NULL REFERENCES products(product_code),
    quantity INTEGER NOT NULL,
    cost_price INTEGER NOT NULL
);

-- 6. return_records（返品・交換記録ヘッダ）
CREATE TABLE return_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    return_type TEXT NOT NULL CHECK(return_type IN ('return','exchange')),
    return_date TEXT NOT NULL,
    register_processed BOOLEAN NOT NULL DEFAULT 1,
    receipt_image_path TEXT,
    note TEXT,
    created_at TEXT NOT NULL
);

-- 7. return_items（返品・交換記録明細）
CREATE TABLE return_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    return_record_id INTEGER NOT NULL REFERENCES return_records(id),
    product_code TEXT NOT NULL REFERENCES products(product_code),
    direction TEXT NOT NULL CHECK(direction IN ('in','out')),
    quantity INTEGER NOT NULL
);

-- 8. manual_sales（手動販売出庫ヘッダ）
CREATE TABLE manual_sales (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sale_date TEXT NOT NULL,
    reason TEXT NOT NULL CHECK(reason IN ('plu_unregistered','other')),
    note TEXT,
    created_at TEXT NOT NULL
);

-- 9. manual_sale_items（手動販売出庫明細）
CREATE TABLE manual_sale_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    manual_sale_id INTEGER NOT NULL REFERENCES manual_sales(id),
    product_code TEXT NOT NULL REFERENCES products(product_code),
    quantity INTEGER NOT NULL,
    amount INTEGER NOT NULL
);

-- 10. disposal_records（廃棄・破損記録ヘッダ）
CREATE TABLE disposal_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    disposal_date TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- 11. disposal_items（廃棄・破損記録明細）
CREATE TABLE disposal_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    disposal_record_id INTEGER NOT NULL REFERENCES disposal_records(id),
    product_code TEXT NOT NULL REFERENCES products(product_code),
    disposal_type TEXT NOT NULL CHECK(disposal_type IN ('disposal','damage','other')),
    quantity INTEGER NOT NULL,
    cost_price INTEGER NOT NULL,
    reason TEXT NOT NULL
);

-- ===========================================
-- POS連携テーブル
-- ===========================================

-- 12. csv_imports（CSV取込み履歴）
CREATE TABLE csv_imports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    settlement_date TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    total_items INTEGER NOT NULL,
    total_amount INTEGER NOT NULL,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK(status IN ('completed','completed_partial','rolled_back')),
    imported_at TEXT NOT NULL
);

-- 12a. csv_import_errors（CSV取込みエラー・スキップ行）
CREATE TABLE csv_import_errors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    csv_import_id INTEGER NOT NULL REFERENCES csv_imports(id),
    source_line_no INTEGER NOT NULL,
    normalized_jan TEXT,
    raw_name TEXT NOT NULL,
    raw_quantity TEXT NOT NULL,
    raw_amount TEXT NOT NULL,
    error_type TEXT NOT NULL CHECK(error_type IN ('unmatched_product','invalid_format','invalid_jan','invalid_number')),
    error_message TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- 13. sale_records（売上レコード）
CREATE TABLE sale_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    csv_import_id INTEGER REFERENCES csv_imports(id),
    product_code TEXT NOT NULL REFERENCES products(product_code),
    sale_date TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    source TEXT NOT NULL CHECK(source IN ('auto','manual')),
    source_line_no INTEGER,
    reason TEXT,
    note TEXT,
    is_voided BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- ===========================================
-- 在庫追跡テーブル
-- ===========================================

-- 14. inventory_movements（在庫変動履歴）
CREATE TABLE inventory_movements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_code TEXT NOT NULL REFERENCES products(product_code),
    movement_type TEXT NOT NULL CHECK(movement_type IN ('sale_auto','sale_manual','receiving','return','disposal','stocktake')),
    quantity INTEGER NOT NULL,
    stock_after INTEGER NOT NULL,
    reference_type TEXT CHECK(reference_type IN ('csv_import','manual_sale','receiving_record','return_record','disposal_record','stocktake') OR reference_type IS NULL),
    reference_id INTEGER,
    note TEXT,
    is_voided BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- 15. price_history（価格変更履歴）
CREATE TABLE price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_code TEXT NOT NULL REFERENCES products(product_code),
    old_selling INTEGER NOT NULL,
    new_selling INTEGER NOT NULL,
    old_cost INTEGER NOT NULL,
    new_cost INTEGER NOT NULL,
    changed_at TEXT NOT NULL
);

-- ===========================================
-- 棚卸しテーブル
-- ===========================================

-- 16. stocktakes（棚卸しヘッダ）
CREATE TABLE stocktakes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL DEFAULT 'in_progress' CHECK(status IN ('in_progress','completed')),
    total_cost INTEGER
);

-- 17. stocktake_items（棚卸し明細）
CREATE TABLE stocktake_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stocktake_id INTEGER NOT NULL REFERENCES stocktakes(id),
    product_code TEXT NOT NULL REFERENCES products(product_code),
    system_stock INTEGER NOT NULL,
    actual_count INTEGER,
    valuation_cost_price INTEGER,
    counted_at TEXT
);

-- ===========================================
-- システムテーブル
-- ===========================================

-- 18. operation_logs（操作ログ）
CREATE TABLE operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL,
    summary TEXT NOT NULL,
    detail_json TEXT,
    created_at TEXT NOT NULL
);

-- 19. app_settings（アプリ設定）
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ===========================================
-- インデックス（DB_DESIGN.md インデックス方針に基づく）
-- ===========================================

CREATE INDEX idx_products_jan_code ON products(jan_code);
CREATE INDEX idx_products_department_id ON products(department_id);
CREATE INDEX idx_products_is_discontinued ON products(is_discontinued);
CREATE INDEX idx_csv_imports_file_hash ON csv_imports(file_hash);
CREATE INDEX idx_sale_records_sale_date ON sale_records(sale_date);
CREATE INDEX idx_sale_records_product_date ON sale_records(product_code, sale_date);
CREATE INDEX idx_sale_records_csv_import_id ON sale_records(csv_import_id);
CREATE INDEX idx_inventory_movements_product_date ON inventory_movements(product_code, created_at);
CREATE INDEX idx_inventory_movements_reference ON inventory_movements(reference_type, reference_id);
CREATE INDEX idx_stocktake_items_stocktake_product ON stocktake_items(stocktake_id, product_code);

-- ===========================================
-- 初期データ: departments（21部門、C-1/C-3 2026-03-29 確定）
-- ===========================================

INSERT INTO departments (id, name, z005_name, code_prefix, next_seq, created_at) VALUES
(1,  'その他小物',     'その他小物',     'KM', 1, '2026-04-03T00:00:00'),
(2,  'ヘア雑貨',       'ヘア雑貨',       'HZ', 1, '2026-04-03T00:00:00'),
(3,  '毛糸',           '毛糸',           NULL,  1, '2026-04-03T00:00:00'),
(4,  '手芸等材料',     '手芸等材料',     'SY', 1, '2026-04-03T00:00:00'),
(5,  '他化粧品',       '他化粧品',       NULL,  1, '2026-04-03T00:00:00'),
(6,  'バッグ',         'バッグ',         'BG', 1, '2026-04-03T00:00:00'),
(7,  'エプロン',       'エプロン',       'AP', 1, '2026-04-03T00:00:00'),
(8,  '布',             '布',             'NU', 1, '2026-04-03T00:00:00'),
(9,  '糸',             '糸',             NULL,  1, '2026-04-03T00:00:00'),
(10, 'ゴム',           'ゴム',           'GM', 1, '2026-04-03T00:00:00'),
(11, '食品',           '食品',           NULL,  1, '2026-04-03T00:00:00'),
(12, '宅急便',         '宅急便',         NULL,  1, '2026-04-03T00:00:00'),
(13, 'トワニー',       'トワニー',       NULL,  1, '2026-04-03T00:00:00'),
(14, '雑貨',           '雑貨',           'ZK', 1, '2026-04-03T00:00:00'),
(15, '帽子',           '帽子',           NULL,  1, '2026-04-03T00:00:00'),
(16, 'ビューティ関連', 'ビューティ関連', NULL,  1, '2026-04-03T00:00:00'),
(17, '本',             '本',             NULL,  1, '2026-04-03T00:00:00'),
(18, 'ボタン',         'ボタン',         'BT', 1, '2026-04-03T00:00:00'),
(19, 'ファスナー',     'ファスナー',     'FS', 1, '2026-04-03T00:00:00'),
(20, '針',             '針',             NULL,  1, '2026-04-03T00:00:00'),
(21, 'ノンリンク',     'ノンリンク',     NULL,  1, '2026-04-03T00:00:00');

-- ===========================================
-- 初期データ: app_settings
-- ===========================================

INSERT INTO app_settings (key, value, updated_at) VALUES
('stock_low_threshold',        '3',     '2026-04-03T00:00:00'),
('stock_low_threshold_fabric', '500',   '2026-04-03T00:00:00'),
('backup_enabled',             '1',     '2026-04-03T00:00:00'),
('backup_time',                '23:00', '2026-04-03T00:00:00'),
('backup_path',                '',      '2026-04-03T00:00:00'),
('backup_retention_days',      '3',     '2026-04-03T00:00:00'),
('tax_rate_standard',          '10',    '2026-04-03T00:00:00'),
('tax_rate_reduced',           '8',     '2026-04-03T00:00:00'),
('log_retention_days',         '3000',  '2026-04-03T00:00:00');
"#
}
