//! L1: レイヤー依存ルール
//!
//! ARCHITECTURE.md のレイヤー間呼び出し原則を自動検証する。
//! UI → CMD → BIZ → IO の一方向のみ。各層が禁止された層に依存していないことを保証する。
//!
//! rust_arkitect は target/ 内の自動生成ファイルで panic するため不採用。
//! 代わりに `use crate::` 文を直接パースして依存関係を検査する。

use std::fs;
use std::path::{Path, PathBuf};

/// レイヤーごとの依存ルール定義
struct LayerRule {
    /// 検査対象のディレクトリ名（例: "db"）
    layer: &'static str,
    /// 依存を禁止するモジュール名のリスト
    forbidden: &'static [&'static str],
}

/// レイヤー依存ルールの例外
const LAYER_EXCEPTIONS: &[(&str, &str)] = &[];

/// ARCHITECTURE.md に基づくレイヤー依存ルール
///
/// IO層は db/ と io/ の2モジュールで構成:
///   db/ = IO-01（SQLiteデータアクセス層）
///   io/ = IO-02（Z004パーサー）, IO-04（PLUフォーマッター）等の純関数群
///
/// 許可方向: UI → CMD → BIZ → IO（db/ + io/）
///
/// db/  → biz, cmd, io への依存禁止
/// biz/ → cmd への依存禁止（db, io は IO層なので許可）
/// cmd/ → db, io への直接依存禁止（biz のみ許可）
/// io/  → biz, cmd, db への依存禁止（純関数層）
const LAYER_RULES: &[LayerRule] = &[
    LayerRule {
        layer: "db",
        forbidden: &["biz", "cmd", "io"],
    },
    LayerRule {
        layer: "biz",
        forbidden: &["cmd"],
    },
    LayerRule {
        layer: "cmd",
        forbidden: &["db", "io"],
    },
    LayerRule {
        layer: "io",
        forbidden: &["biz", "cmd", "db"],
    },
];

/// 指定ディレクトリ配下の .rs ファイルを再帰的に列挙
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

/// ソースファイル内の `use crate::{module}` パターンを検出
/// `#[cfg(test)]` ブロック内は除外（テスト専用の cross-layer import を許容）
fn find_forbidden_imports(
    file_path: &Path,
    forbidden_modules: &[&str],
) -> Vec<(usize, String, String)> {
    let content = fs::read_to_string(file_path).expect("ファイル読み込み失敗");
    let mut violations = Vec::new();
    let mut in_cfg_test = false;
    let mut brace_depth: i32 = 0;

    for (line_no, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // #[cfg(test)] ブロックの開始を検出
        if trimmed.contains("#[cfg(test)]") {
            in_cfg_test = true;
            brace_depth = 0;
            continue;
        }

        // #[cfg(test)] ブロック内のブレース追跡
        if in_cfg_test {
            for ch in trimmed.chars() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth <= 0 {
                        in_cfg_test = false;
                        break;
                    }
                }
            }
            continue;
        }

        // コメント行はスキップ
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        // インラインコメント部分を除去してからチェック
        let code_part = trimmed.split("//").next().unwrap_or(trimmed);
        for forbidden in forbidden_modules {
            // use crate::forbidden_module パターンを検出
            let patterns = [
                format!("use crate::{}", forbidden),
                format!("crate::{}::", forbidden),
                format!("crate::{} ", forbidden),
                format!("crate::{};", forbidden),
                format!("crate::{},", forbidden),
                format!("crate::{}}}", forbidden),
            ];
            if patterns.iter().any(|pat| code_part.contains(pat.as_str())) {
                violations.push((line_no + 1, forbidden.to_string(), trimmed.to_string()));
                break;
            }
        }
    }
    violations
}

#[test]
fn layer_dependency_rules() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut all_violations: Vec<String> = Vec::new();

    for rule in LAYER_RULES {
        let layer_dir = src_dir.join(rule.layer);
        let rs_files = collect_rs_files(&layer_dir);

        for file_path in &rs_files {
            let violations = find_forbidden_imports(file_path, rule.forbidden);
            for (line_no, forbidden_mod, line_content) in &violations {
                let relative = file_path.strip_prefix(&src_dir).unwrap_or(file_path);
                let relative_str = relative.display().to_string().replace('\\', "/");

                // 例外チェック
                let is_exception = LAYER_EXCEPTIONS
                    .iter()
                    .any(|(file, module)| relative_str == *file && forbidden_mod == module);
                if is_exception {
                    continue;
                }

                all_violations.push(format!(
                    "  {}:{} — {}/が{}に依存: {}",
                    relative_str, line_no, rule.layer, forbidden_mod, line_content
                ));
            }
        }
    }

    if !all_violations.is_empty() {
        panic!(
            "レイヤー依存ルール違反が {}件 見つかりました:\n{}",
            all_violations.len(),
            all_violations.join("\n")
        );
    }
}

// SPEC-HYG3-REX-1..3: oracle は production allow list から導出しない。
#[test]
fn direct_reexport_fixtures() {
    let dir = tempfile::tempdir().expect("fixture directory");
    fs::create_dir(dir.path().join("biz")).unwrap();
    fs::create_dir(dir.path().join("mnt")).unwrap();
    let file = dir.path().join("biz/mod.rs");
    let allowed = [
        ("biz/mod.rs", "DbConnection", "crate::db::DbConnection"),
        (
            "biz/mod.rs",
            "Department",
            "crate::db::product_repo::Department",
        ),
        (
            "biz/mod.rs",
            "PaginatedResult",
            "crate::db::PaginatedResult",
        ),
    ];
    let baseline = "\n// 行番号は assertion key に含めない\npub use crate::db::DbConnection;\npub(crate) use crate::db::product_repo::{\n Department,\n};\npub use crate::db::PaginatedResult;\n";
    fs::write(&file, baseline).unwrap();
    assert!(reexport_differences(dir.path(), &allowed).is_empty());

    for addition in [
        "pub use crate::db::Row;",
        "pub(crate) use crate::db::Row2;",
        "pub use crate::db::product_repo::{Department, Row3};",
        "// comment\npub use crate::db::Row4;",
        "pub use crate::db::other_repo::PaginatedResult;",
        // SC-REX-1 変種 6・7: group 展開前の prefix 判定で読み飛ばさない。
        "pub use crate::{db::NewOperationLog, io::image_manager};",
        "pub use {crate::db::NewOperationLog, crate::io::image_manager};",
    ] {
        fs::write(&file, format!("{baseline}{addition}\n")).unwrap();
        let differences = reexport_differences(dir.path(), &allowed);
        assert!(!differences.is_empty(), "missed addition: {addition}");
        assert!(differences.iter().any(|diff| diff.contains("unexpected")));
        if addition.contains("image_manager") {
            for path in ["crate::db::NewOperationLog", "crate::io::image_manager"] {
                assert!(differences
                    .iter()
                    .any(|diff| diff.contains("unexpected") && diff.contains(path)));
            }
        }
    }

    // SC-REX-1 変種 8: 同一行 block comment 後の直接再公開も検出する。
    fs::write(
        &file,
        format!("{baseline}/* new public API */ pub use crate::db::open_database;\n"),
    )
    .unwrap();
    assert!(reexport_differences(dir.path(), &allowed)
        .iter()
        .any(|diff| diff.contains("unexpected") && diff.contains("crate::db::open_database")));

    fs::write(&file, format!("{baseline}/* comment only */\n")).unwrap();
    assert!(reexport_differences(dir.path(), &allowed).is_empty());

    fs::write(
        &file,
        format!("{baseline}/* multiline comment\n * pub use crate::db::x;\n */\n"),
    )
    .unwrap();
    assert!(reexport_differences(dir.path(), &allowed).is_empty());

    fs::write(
        &file,
        baseline.replace("pub use crate::db::DbConnection;", ""),
    )
    .unwrap();
    let differences = reexport_differences(dir.path(), &allowed);
    assert!(differences
        .iter()
        .any(|diff| diff.contains("missing") && diff.contains("DbConnection")));

    // IO 側と mnt 側も同じ公開境界として検査する。
    fs::write(&file, baseline).unwrap();
    fs::write(dir.path().join("mnt/mod.rs"), "pub use crate::io::Row;").unwrap();
    assert!(reexport_differences(dir.path(), &allowed)
        .iter()
        .any(|diff| diff.contains("crate::io::Row")));
}

// D-083: 既存の直接公開面を凍結する。key に行番号を含めない。
const DB_IO_REEXPORT_ALLOWLIST: &[(&str, &str, &str)] = &[
    (
        "biz/mod.rs",
        "Department",
        "crate::db::product_repo::Department",
    ), // :19
    (
        "biz/mod.rs",
        "Supplier",
        "crate::db::product_repo::Supplier",
    ), // :19
    (
        "biz/mod.rs",
        "ProductBulkFilter",
        "crate::db::product_repo::ProductBulkFilter",
    ), // :21
    (
        "biz/mod.rs",
        "ProductSearchQuery",
        "crate::db::product_repo::ProductSearchQuery",
    ), // :21
    (
        "biz/mod.rs",
        "ProductWithRelations",
        "crate::db::product_repo::ProductWithRelations",
    ), // :21
    (
        "biz/mod.rs",
        "ProductStockUnit",
        "crate::db::product_repo::ProductStockUnit",
    ), // :21（SPEC-UNIT-D11: CMD の ProductResponse.stock_unit）
    (
        "biz/mod.rs",
        "CsvImport",
        "crate::db::sales_repo::CsvImport",
    ), // :23
    (
        "biz/mod.rs",
        "LastStocktakeSummary",
        "crate::db::stocktake_repo::LastStocktakeSummary",
    ), // :25
    (
        "biz/mod.rs",
        "Stocktake",
        "crate::db::stocktake_repo::Stocktake",
    ), // :25
    (
        "biz/mod.rs",
        "StocktakeProgress",
        "crate::db::stocktake_repo::StocktakeProgress",
    ), // :25
    (
        "biz/mod.rs",
        "AppSetting",
        "crate::db::system_repo::AppSetting",
    ), // :29
    (
        "biz/mod.rs",
        "OperationLog",
        "crate::db::system_repo::OperationLog",
    ), // :29
    ("biz/mod.rs", "DbConnection", "crate::db::DbConnection"), // :31
    ("biz/mod.rs", "DbError", "crate::db::DbError"),           // :33
    (
        "biz/mod.rs",
        "PaginatedResult",
        "crate::db::PaginatedResult",
    ), // :35
    (
        "biz/mod.rs",
        "DisposalRecordDetail",
        "crate::db::disposal_repo::DisposalRecordDetail",
    ), // :37
    (
        "biz/mod.rs",
        "DisposalRecordSummary",
        "crate::db::disposal_repo::DisposalRecordSummary",
    ), // :37
    (
        "biz/mod.rs",
        "InventoryRecordQuery",
        "crate::db::disposal_repo::InventoryRecordQuery",
    ), // :37
    (
        "biz/mod.rs",
        "InventoryRecordSummary",
        "crate::db::disposal_repo::InventoryRecordSummary",
    ), // :37
    (
        "biz/mod.rs",
        "ListQuery",
        "crate::db::inventory_common::ListQuery",
    ), // :40
    (
        "biz/mod.rs",
        "ManualSaleRecordDetail",
        "crate::db::manual_sale_repo::ManualSaleRecordDetail",
    ), // :41
    (
        "biz/mod.rs",
        "ReceivingRecordDetail",
        "crate::db::receiving_repo::ReceivingRecordDetail",
    ), // :42
    (
        "biz/mod.rs",
        "ReceivingRecordWithSupplier",
        "crate::db::receiving_repo::ReceivingRecordWithSupplier",
    ), // :42
    (
        "biz/mod.rs",
        "ReturnRecordDetail",
        "crate::db::return_repo::ReturnRecordDetail",
    ), // :43
    (
        "biz/mod.rs",
        "ReturnRecordSummary",
        "crate::db::return_repo::ReturnRecordSummary",
    ), // :43
    (
        "biz/mod.rs",
        "MovementQuery",
        "crate::db::inventory_repo::MovementQuery",
    ), // :45
    (
        "biz/mod.rs",
        "MovementRecord",
        "crate::db::inventory_repo::MovementRecord",
    ), // :45
    (
        "biz/mod.rs",
        "StockDetail",
        "crate::db::product_repo::StockDetail",
    ), // :46
    (
        "biz/product_service.rs",
        "PriceHistoryEntry",
        "crate::db::product_repo::PriceHistoryEntry",
    ), // :17
    (
        "mnt/backup.rs",
        "open_existing_database",
        "crate::db::open_existing_database",
    ), // :10
];

/// 既存の brace 深度追跡と同様、複数行の公開文を閉じ括弧まで束ねる。
/// ponytail: Rust の構文解析器ではない。直接 crate::db/io パスのみが D-083 の対象。
fn direct_db_io_reexports(file: &Path) -> Vec<(String, String)> {
    let content = fs::read_to_string(file).expect("re-export source");
    let mut statement = String::new();
    let mut depth = 0_i32;
    let mut exports = Vec::new();
    for line in content.lines() {
        let mut code = line.split("//").next().unwrap_or("").trim();
        if code.starts_with("/*") {
            let Some((_, rest)) = code.split_once("*/") else {
                continue;
            };
            // D-083: 同一行の block comment で後続の直接再公開を隠させない。
            code = rest.trim();
        }
        if code.starts_with('*') {
            continue;
        }
        if statement.is_empty() {
            let Some(path) = code
                .strip_prefix("pub use ")
                .or_else(|| code.strip_prefix("pub(crate) use "))
            else {
                continue;
            };
            if !path.starts_with("crate::") && !path.starts_with('{') {
                continue;
            }
            statement.push_str(path);
        } else {
            statement.push_str(code);
        }
        depth += code.chars().filter(|&c| c == '{').count() as i32;
        depth -= code.chars().filter(|&c| c == '}').count() as i32;
        if depth != 0 || !code.ends_with(';') {
            continue;
        }
        let path = statement.trim_end_matches(';');
        let mut paths = Vec::new();
        if let Some((prefix, group)) = path.split_once('{') {
            let group = group
                .strip_suffix('}')
                .expect("D-083: unsupported re-export group");
            for symbol in group.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                assert!(
                    !symbol.contains(['{', '}']),
                    "D-083: unsupported nested group"
                );
                paths.push(format!("{}{symbol}", prefix.trim()));
            }
        } else {
            paths.push(path.to_string());
        }
        // crate 直下 / outer group も展開してから各 leaf の full path で判定する。
        for path in paths {
            if !path.starts_with("crate::db::") && !path.starts_with("crate::io::") {
                continue;
            }
            let symbol = path.rsplit("::").next().unwrap();
            exports.push((symbol.to_string(), path));
        }
        statement.clear();
    }
    assert!(
        statement.is_empty(),
        "D-083: unterminated re-export in {}",
        file.display()
    );
    exports
}

/// 両方向の差分にすることで、例外表だけが残る削除も検出する。
fn reexport_differences(src_dir: &Path, allowed: &[(&str, &str, &str)]) -> Vec<String> {
    use std::collections::BTreeSet;
    let expected: BTreeSet<_> = allowed
        .iter()
        .map(|(file, symbol, path)| (file.to_string(), symbol.to_string(), path.to_string()))
        .collect();
    let mut actual = BTreeSet::new();
    for layer in ["biz", "mnt"] {
        assert!(
            src_dir.join(layer).is_dir(),
            "D-083: missing {layer} directory"
        );
        for file in collect_rs_files(&src_dir.join(layer)) {
            let relative = file
                .strip_prefix(src_dir)
                .unwrap()
                .display()
                .to_string()
                .replace('\\', "/");
            for (symbol, path) in direct_db_io_reexports(&file) {
                actual.insert((relative.clone(), symbol, path));
            }
        }
    }
    actual
        .difference(&expected)
        .map(|entry| format!("unexpected {entry:?}"))
        .chain(
            expected
                .difference(&actual)
                .map(|entry| format!("missing {entry:?}")),
        )
        .collect()
}

#[test]
fn biz_mnt_direct_db_io_reexport_allowlist() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let differences = reexport_differences(&src_dir, DB_IO_REEXPORT_ALLOWLIST);
    assert!(
        differences.is_empty(),
        "D-083 (docs/decision-log.md): DB/IO re-export allow list drift:\n{}",
        differences.join("\n")
    );
}
