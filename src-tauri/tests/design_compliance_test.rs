//! L2: 設計書-コード シグネチャ突合テスト（Phase A）
//!
//! 設計書（docs/function-design/*.md）から関数名を regex で抽出し、
//! Rust ソース（src/**/*.rs）から syn で pub fn を抽出して突合する。
//!
//! - OK: 設計書にもコードにもある関数
//! - WARN: コードにあるが設計書にない pub 関数（allowlist 外 → テスト失敗）
//! - INFO: 設計書にあるがコードにない関数（未実装、テスト失敗にしない）

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 設定
// ---------------------------------------------------------------------------

/// 設計書ディレクトリ（CARGO_MANIFEST_DIR からの相対パス）
const DESIGN_DOCS_DIR: &str = "../docs/function-design";

/// ソースディレクトリ（CARGO_MANIFEST_DIR からの相対パス）
const RUST_SRC_DIR: &str = "src";

/// 除外する設計書ファイル（型定義のみ / UI 層）
const SKIP_DOCS: &[&str] = &[
    "50-ui-product-list.md",
    "51-ui-product-form.md",
    "52-ui-shared-layout.md",
    "53-ui-home.md",
    "54-ui-shortcuts.md",
    "55-ui-csv-import.md",
    "56-ui-daily-sales.md",
    "57-ui-monthly-sales.md",
    "58-ui-stock-inquiry.md",
    "59-ui-shared-patterns.md",
    "60-ui-product-import.md",
    "61-ui-receiving.md",
    "62-ui-manual-sale.md",
    "63-ui-return-exchange.md",
    "64-ui-disposal.md",
    "66-ui-stock-movements.md",
    "67-ui-plu-export.md",
    "68-ui-backup-restore.md",
    "76-ui-request-primitives.md",
    "77-ui-bulk-price-revision.md",
    "78-ui-supplier-management.md",
    // 自動生成のトレーサビリティマトリクス（generate_traceability bin が再生成、関数定義を持たない）
    "90-traceability.md",
];

/// 除外するソースファイル名パターン（ファイル名末尾マッチ）
const SKIP_SOURCE_FILES: &[&str] = &[
    "lib.rs",
    "main.rs",
    "constants.rs",
    "test_support.rs",
    "invariants.rs",
];

/// 設計書に対応エントリがない既知の pub 関数（allowlist）
///
/// ここに追加する場合は理由をコメントに残すこと。
/// 設計書を更新して allowlist から外すのが望ましい。
const KNOWN_ALLOWLIST: &[(&str, &str)] = &[
    // 設計書に未記載の追加関数（supplier 単体取得は BIZ-01 では使わないが BIZ-02 で利用）
    ("db::product_repo", "find_supplier_by_id"),
    // 実装時に追加した内部ヘルパー（ListQuery のバリデーション）
    ("db::inventory_common", "validate_and_offset"),
    // migration v2 用。設計書（22-mnt-migration.md）では粒度が粗く個別関数として未記載
    ("db::schema_v2", "apply_v2_idempotency"),
    // CMD-01 の4関数: 40-cmd-product.md §5.4 に記載あるがコードブロック未使用のため regex 検出不可
    ("cmd::product_cmd", "update_product"),
    ("cmd::product_cmd", "toggle_discontinue"),
    ("cmd::product_cmd", "search_products"),
    ("cmd::product_cmd", "get_product"),
    // BIZ ラッパー: CMD→DB直接依存を避けるためPR-6で追加。設計書は IO-01経由 と記載
    ("biz::product_service", "get_product"),
    ("biz::stocktake_service", "get_stocktake_items"),
    // UI-10 新規CMD/BIZ/IO: 73-ui-stocktake.md §73.8 に inline で記載。
    // このテストの regex はコードブロック外の関数名を検出しないため allowlist 化。
    ("cmd::stocktake_cmd", "get_active_stocktake"),
    ("cmd::stocktake_cmd", "find_stocktake_item"),
    ("cmd::stocktake_cmd", "get_last_completed_stocktake"),
    ("biz::stocktake_service", "get_active_stocktake"),
    ("biz::stocktake_service", "find_stocktake_item"),
    ("biz::stocktake_service", "get_last_completed_stocktake"),
    ("db::stocktake_repo", "find_stocktake_item_by_code"),
    ("db::stocktake_repo", "find_last_completed_stocktake"),
    // DB移行ヘルパー: PR #25 レビュー指摘対応。旧相対パスからapp_data_dir配下への移行
    ("db", "migrate_legacy_db"),
    // dev tooling (Phase 1 Task 7-9): src/bin/seed_demo_data.rs から呼ぶデモデータ seed。
    // 本番コードパスではないため設計書には記載しない
    ("seed_demo", "run_seed"),
    ("seed_demo", "delete_all"),
];

// ---------------------------------------------------------------------------
// 設計書 → モジュール マッピングテーブル
// ---------------------------------------------------------------------------

/// 設計書ファイル名 → 対応する Rust モジュールパスの一覧
/// 設計書 → Rustモジュール のマッピング。
///
/// **重要**: 新しい設計書（docs/function-design/*.md）を追加したら、
/// ここに対応するモジュールパスを追加すること。追加を忘れるとCIが落ちる。
/// 実装モジュールが未作成でもマッピングは必要（未実装関数は info_unimplemented として報告される）。
fn build_doc_to_modules_map() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();

    map.insert(
        "20-io-product-repo.md",
        vec![
            "db",
            "db::product_repo",
            "db::inventory_repo",
            "db::sales_repo",
            "db::stocktake_repo",
            "db::system_repo",
            "db::plu_slot_repo",
        ],
    );

    map.insert(
        "21-io-inventory-repo.md",
        vec![
            "db::inventory_repo",
            "db::inventory_common",
            "db::receiving_repo",
            "db::return_repo",
            "db::manual_sale_repo",
            "db::disposal_repo",
            "db::sales_repo",
            "db::stocktake_repo",
            "db::system_repo",
        ],
    );

    map.insert(
        "22-mnt-migration.md",
        vec![
            "db::migration",
            "db::schema_v1",
            "db::schema_v2",
            "db::schema_v3",
        ],
    );

    map.insert("23-io-z004-parser.md", vec!["io::z004_parser"]);

    map.insert(
        "24-io-csv-import-repo.md",
        vec!["db::csv_import_repo", "db::sales_repo"],
    );

    // SALES 日報パーサー: 実装済み
    map.insert(
        "29-io-daily-report-parser.md",
        vec![
            "io::daily_report_parser",
            "io::register_sd",
            "io::pos_source_copy",
        ],
    );

    map.insert("29-io-ej-parser.md", vec!["io::ej_parser"]);

    map.insert("25-io-plu-formatter.md", vec!["io::plu_formatter"]);

    map.insert("10-common-rules.md", vec!["biz::unit_amount"]);

    map.insert("30-biz-product-service.md", vec!["biz::product_service"]);

    map.insert(
        "31-biz-inventory-service.md",
        vec![
            "biz::inventory_service",
            "biz::inventory_service::common",
            "biz::inventory_service::receiving",
            "biz::inventory_service::returns",
            "biz::inventory_service::manual_sale",
            "biz::inventory_service::disposal",
            "biz::stock_ledger",
        ],
    );

    map.insert(
        "32-biz-csv-import-service.md",
        vec![
            "biz::csv_import_service",
            "biz::csv_import_service::parse",
            "biz::csv_import_service::commit",
            "biz::csv_import_service::rollback",
            "biz::csv_import_service::list",
        ],
    );

    map.insert(
        "33-biz-plu-export-service.md",
        vec!["biz::plu_export_service"],
    );

    // SALES 日報取込みサービス: 実装済み
    map.insert(
        "37-biz-daily-report-import-service.md",
        vec![
            "biz::daily_report_import_service",
            "biz::daily_report_import_service::parse",
            "biz::daily_report_import_service::commit",
            "biz::daily_report_import_service::rollback",
            "biz::daily_report_import_service::list",
            "biz::daily_report_import_service::scan",
        ],
    );

    map.insert("38-biz-system-service.md", vec!["biz::system_service"]);

    map.insert("40-cmd-product.md", vec!["cmd::product_cmd"]);

    map.insert(
        "41-cmd-pos.md",
        vec!["cmd::csv_import_cmd", "cmd::plu_export_cmd"],
    );

    // SALES 日報取込みコマンド: 実装済み
    map.insert(
        "45-cmd-daily-report-import.md",
        vec!["cmd::daily_report_import_cmd"],
    );

    // 商品 CSV 取込みパーサー: 実装済み
    map.insert(
        "26-io-product-csv-importer.md",
        vec!["io::product_csv_importer"],
    );

    map.insert("34-biz-sales-service.md", vec!["biz::sales_service"]);

    map.insert(
        "35-biz-stocktake-service.md",
        vec!["biz::stocktake_service"],
    );

    map.insert("36-biz-integrity-check.md", vec!["biz::integrity_service"]);

    map.insert(
        "42-cmd-sales-stocktake.md",
        vec![
            "cmd::product_cmd",
            "cmd::sales_cmd",
            "cmd::stocktake_cmd",
            "cmd::integrity_cmd",
        ],
    );

    // MNT-04: 診断ログ
    map.insert("70-mnt-diagnostic-log.md", vec!["mnt::diagnostic_log"]);

    map.insert(
        "69-ui-threshold-settings.md",
        vec!["cmd::settings_cmd", "db::system_repo"],
    );

    map.insert(
        "73-ui-stocktake.md",
        vec![
            "cmd::stocktake_cmd",
            "biz::stocktake_service",
            "db::stocktake_repo",
        ],
    );

    map.insert(
        "74-ui-operation-logs.md",
        vec!["cmd::settings_cmd", "db::system_repo"],
    );

    map.insert(
        "75-ui-integrity-check.md",
        vec!["cmd::integrity_cmd", "biz::integrity_service"],
    );

    // 保守・仕上げモジュール: 実装済み
    map.insert(
        "27-io-report-csv-exporter.md",
        vec!["io::report_csv_exporter"],
    );
    map.insert("28-io-image-manager.md", vec!["io::image_manager"]);
    map.insert("71-mnt-backup.md", vec!["mnt::backup"]);
    map.insert("72-mnt-log-manager.md", vec!["mnt::log_manager"]);
    map.insert("43-cmd-settings-log.md", vec!["cmd::settings_cmd"]);
    // CMD-02〜06: 入出庫・在庫照会コマンド群（実装済み）
    // §23.9 IO層関数、§23.10 BIZ層 list ラッパーも含む
    map.insert(
        "44-cmd-inventory.md",
        vec![
            "cmd::receiving_cmd",
            "cmd::return_cmd",
            "cmd::manual_sale_cmd",
            "cmd::disposal_cmd",
            "cmd::inventory_cmd",
            "db::product_repo",
            "db::inventory_repo",
            "biz::inventory_service",
            "biz::inventory_service::list",
            "biz::product_service",
        ],
    );

    map.insert(
        "65-inventory-record-traceability.md",
        vec![
            "cmd::receiving_cmd",
            "cmd::return_cmd",
            "cmd::manual_sale_cmd",
            "cmd::disposal_cmd",
            "cmd::csv_import_cmd",
            "cmd::stocktake_cmd",
            "cmd::settings_cmd",
            "db::inventory_repo",
            "db::receiving_repo",
            "db::return_repo",
            "db::manual_sale_repo",
            "db::disposal_repo",
            "db::csv_import_repo",
            "db::stocktake_repo",
            "db::system_repo",
            "biz::inventory_service",
            "biz::inventory_service::receiving",
            "biz::inventory_service::returns",
            "biz::inventory_service::manual_sale",
            "biz::inventory_service::disposal",
            "biz::csv_import_service",
            "biz::stocktake_service",
        ],
    );

    map
}

/// モジュール → 設計書ファイル名 の逆引きマップを構築
fn build_module_to_docs_map(
    doc_to_modules: &HashMap<&str, Vec<&str>>,
) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (doc_file, modules) in doc_to_modules {
        for module in modules {
            map.entry(module.to_string())
                .or_default()
                .push(doc_file.to_string());
        }
    }
    map
}

// ---------------------------------------------------------------------------
// 設計書パーサー（regex）
// ---------------------------------------------------------------------------

/// 設計書のコードブロックから関数名を抽出する
fn extract_functions_from_design_doc(file_path: &Path) -> Vec<String> {
    let content = fs::read_to_string(file_path).expect("設計書読み込み失敗");
    let re = regex::Regex::new(r"(?:pub\s+)?(?:#\[tauri::command\]\s*)?fn\s+(\w+)\s*\(")
        .expect("regex コンパイル失敗");

    let mut functions = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lines = Vec::new();

    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            if in_code_block {
                // コードブロック終了 → まとめてパース
                let joined = code_block_lines.join(" ");
                for cap in re.captures_iter(&joined) {
                    let fn_name = cap[1].to_string();
                    // struct / enum / impl の内部定義は除外
                    // fn が struct/enum/impl ブロック内にない場合のみ採用
                    if !is_type_definition(&fn_name) {
                        functions.push(fn_name);
                    }
                }
                code_block_lines.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            let trimmed = line.trim();
            // コメント行をスキップ
            if !trimmed.starts_with("//") {
                code_block_lines.push(trimmed.to_string());
            }
        }
    }

    // 重複除去（同じ関数名がコードブロックに複数回出る場合）
    functions.sort();
    functions.dedup();
    functions
}

/// 型定義関連のキーワードでないかチェック
fn is_type_definition(name: &str) -> bool {
    // fn fmt は Display 実装の一部
    matches!(name, "fmt")
}

// ---------------------------------------------------------------------------
// Rust ソースパーサー（syn）
// ---------------------------------------------------------------------------

/// .rs ファイルを再帰的に列挙
fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    for entry in fs::read_dir(dir).expect("ディレクトリ読み込み失敗") {
        let entry = entry.expect("エントリ読み込み失敗");
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rs_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}

/// ファイルパスからモジュールパスを推定
/// 例: src/db/product_repo.rs → "db::product_repo"
/// 例: src/biz/inventory_service/receiving.rs → "biz::inventory_service::receiving"
/// 例: src/db/mod.rs → "db"
fn file_to_module_path(file_path: &Path, src_root: &Path) -> Option<String> {
    let relative = file_path.strip_prefix(src_root).ok()?;
    let components: Vec<&str> = relative
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    if components.is_empty() {
        return None;
    }

    // ファイル名から .rs を除去
    let last = components.last()?;
    let stem = last.strip_suffix(".rs")?;

    if stem == "mod" {
        // mod.rs → 親ディレクトリまでのパス
        if components.len() <= 1 {
            return None; // src/mod.rs は想定外
        }
        Some(components[..components.len() - 1].join("::"))
    } else {
        // 通常のファイル
        let mut parts: Vec<&str> = components[..components.len() - 1].to_vec();
        parts.push(stem);
        Some(parts.join("::"))
    }
}

/// syn で pub fn を抽出
fn extract_pub_functions_from_source(file_path: &Path) -> Vec<String> {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let syntax = match syn::parse_file(&content) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut functions = Vec::new();
    extract_pub_fns_from_items(&syntax.items, &mut functions);
    functions
}

/// syn の Item リストから pub fn を再帰的に抽出
fn extract_pub_fns_from_items(items: &[syn::Item], out: &mut Vec<String>) {
    for item in items {
        match item {
            syn::Item::Fn(item_fn) => {
                // pub のみ（pub(crate) は Restricted なので除外される）
                if matches!(item_fn.vis, syn::Visibility::Public(_)) {
                    // #[cfg(test)] 属性付きはスキップ
                    if !has_cfg_test(&item_fn.attrs) {
                        out.push(item_fn.sig.ident.to_string());
                    }
                }
            }
            syn::Item::Mod(item_mod) => {
                // #[cfg(test)] mod は全体をスキップ
                if has_cfg_test(&item_mod.attrs) {
                    continue;
                }
                // インライン mod の中身を再帰走査
                if let Some((_, ref items)) = item_mod.content {
                    extract_pub_fns_from_items(items, out);
                }
            }
            _ => {}
        }
    }
}

/// #[cfg(test)] または #[cfg(any(test, ...))] 属性があるかチェック
fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        // Meta::List のトークンを文字列化して "test" を含むか簡易チェック
        // #[cfg(test)], #[cfg(any(test, ...))] 等に対応
        match &attr.meta {
            syn::Meta::List(list) => list.tokens.to_string().contains("test"),
            _ => false,
        }
    })
}

// ---------------------------------------------------------------------------
// 突合テスト
// ---------------------------------------------------------------------------

/// 突合結果
struct ComplianceReport {
    matched: Vec<(String, String, String)>, // (module, fn_name, doc_file)
    warnings: Vec<(String, String, bool)>,  // (module, fn_name, is_allowlisted)
    info_unimplemented: Vec<(String, String)>, // (doc_file, fn_name)
}

#[test]
fn design_code_compliance_phase_a() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let design_dir = manifest_dir.join(DESIGN_DOCS_DIR);
    let src_dir = manifest_dir.join(RUST_SRC_DIR);

    let doc_to_modules = build_doc_to_modules_map();
    let module_to_docs = build_module_to_docs_map(&doc_to_modules);

    // -----------------------------------------------------------------------
    // 1. 設計書から関数名を抽出
    // -----------------------------------------------------------------------
    let mut design_fns: HashMap<String, HashSet<String>> = HashMap::new(); // doc_file → fn_names

    let mut doc_files: Vec<_> = fs::read_dir(&design_dir)
        .expect("設計書ディレクトリ読み込み失敗")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".md") && !SKIP_DOCS.contains(&name.as_str())
        })
        .collect();
    doc_files.sort_by_key(|e| e.file_name());

    let mut unmapped_docs: Vec<String> = Vec::new();

    for entry in &doc_files {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !doc_to_modules.contains_key(file_name.as_str()) {
            unmapped_docs.push(file_name);
            continue;
        }
        let fns = extract_functions_from_design_doc(&entry.path());
        design_fns.insert(file_name, fns.into_iter().collect());
    }

    // 指摘#1: 未マッピング設計書が静かに漏れないよう検出する
    assert!(
        unmapped_docs.is_empty(),
        "モジュールマッピングに未登録の設計書が {}件 あります。build_doc_to_modules_map() に追加してください:\n{}",
        unmapped_docs.len(),
        unmapped_docs.iter().map(|d| format!("  {}", d)).collect::<Vec<_>>().join("\n")
    );

    // -----------------------------------------------------------------------
    // 2. Rust ソースから pub fn を抽出
    // -----------------------------------------------------------------------
    let mut code_fns: HashMap<String, HashSet<String>> = HashMap::new(); // module_path → fn_names

    let rs_files = collect_rs_files(&src_dir);
    for file_path in &rs_files {
        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if SKIP_SOURCE_FILES.iter().any(|skip| file_name == *skip) {
            continue;
        }

        let module_path = match file_to_module_path(file_path, &src_dir) {
            Some(m) => m,
            None => continue,
        };

        let fns = extract_pub_functions_from_source(file_path);
        if !fns.is_empty() {
            code_fns.entry(module_path).or_default().extend(fns);
        }
    }

    // -----------------------------------------------------------------------
    // 3. 突合
    // -----------------------------------------------------------------------
    let allowlist: HashSet<(&str, &str)> = KNOWN_ALLOWLIST.iter().copied().collect();
    let mut report = ComplianceReport {
        matched: Vec::new(),
        warnings: Vec::new(),
        info_unimplemented: Vec::new(),
    };

    // 3a. 設計書の関数がコードに存在するか
    for (doc_file, fn_names) in &design_fns {
        let mapped_modules = match doc_to_modules.get(doc_file.as_str()) {
            Some(m) => m,
            None => continue,
        };

        for fn_name in fn_names {
            let found_in_module = mapped_modules.iter().find(|module| {
                code_fns
                    .get(**module)
                    .is_some_and(|fns| fns.contains(fn_name))
            });

            if let Some(module) = found_in_module {
                report
                    .matched
                    .push((module.to_string(), fn_name.clone(), doc_file.clone()));
            } else {
                report
                    .info_unimplemented
                    .push((doc_file.clone(), fn_name.clone()));
            }
        }
    }

    // 3b. コードの pub fn が設計書に存在するか
    for (module, fn_names) in &code_fns {
        let doc_files_for_module = module_to_docs.get(module);

        for fn_name in fn_names {
            // 既に matched で見つかっている場合はスキップ
            let already_matched = report
                .matched
                .iter()
                .any(|(m, f, _)| m == module && f == fn_name);
            if already_matched {
                continue;
            }

            let is_in_allowlist = allowlist.contains(&(module.as_str(), fn_name.as_str()));

            // 設計書にあるか確認
            let found_in_doc = doc_files_for_module.is_some_and(|docs| {
                docs.iter()
                    .any(|doc| design_fns.get(doc).is_some_and(|fns| fns.contains(fn_name)))
            });

            if !found_in_doc {
                report
                    .warnings
                    .push((module.clone(), fn_name.clone(), is_in_allowlist));
            }
        }
    }

    // -----------------------------------------------------------------------
    // 4. レポート出力
    // -----------------------------------------------------------------------
    report.matched.sort();
    report.warnings.sort();
    report.info_unimplemented.sort();

    println!("\n=== Design-Code Compliance Report (Phase A) ===\n");

    println!("--- Matched (OK): {} functions ---", report.matched.len());
    for (module, fn_name, doc_file) in &report.matched {
        println!("  {}::{}  [{}]", module, fn_name, doc_file);
    }

    println!(
        "\n--- In Code, Not in Design (WARN): {} functions ---",
        report.warnings.len()
    );
    for (module, fn_name, is_allowlisted) in &report.warnings {
        let tag = if *is_allowlisted {
            "[allowlisted]"
        } else {
            "[UNEXPECTED]"
        };
        println!("  {}::{}  {}", module, fn_name, tag);
    }

    println!(
        "\n--- In Design, Not in Code (INFO): {} functions ---",
        report.info_unimplemented.len()
    );
    for (doc_file, fn_name) in &report.info_unimplemented {
        println!("  {}  [{}]", fn_name, doc_file);
    }

    let unexpected_count = report
        .warnings
        .iter()
        .filter(|(_, _, is_al)| !is_al)
        .count();
    let allowlisted_count = report
        .warnings
        .iter()
        .filter(|(_, _, is_al)| *is_al)
        .count();

    println!(
        "\nSummary: {} matched, {} warnings ({} allowlisted, {} unexpected), {} not-yet-implemented",
        report.matched.len(),
        report.warnings.len(),
        allowlisted_count,
        unexpected_count,
        report.info_unimplemented.len(),
    );

    // -----------------------------------------------------------------------
    // 5. アサーション
    // -----------------------------------------------------------------------

    // allowlist 外の未文書化 pub 関数があればテスト失敗
    if unexpected_count > 0 {
        let unexpected: Vec<_> = report
            .warnings
            .iter()
            .filter(|(_, _, is_al)| !is_al)
            .map(|(m, f, _)| format!("  {}::{}", m, f))
            .collect();
        panic!(
            "\n{} 件の未文書化 pub 関数が見つかりました（allowlist に追加するか設計書を更新してください）:\n{}",
            unexpected_count,
            unexpected.join("\n")
        );
    }

    // パーサーの異常検出: 設計書関数数の30%以上が matched であること
    let total_design_fns: usize = design_fns.values().map(|s| s.len()).sum();
    let min_expected = total_design_fns * 30 / 100;
    assert!(
        report.matched.len() >= min_expected,
        "突合結果が少なすぎます（{}/{}件、閾値{}件）。パーサーに問題がある可能性があります",
        report.matched.len(),
        total_design_fns,
        min_expected
    );
}
