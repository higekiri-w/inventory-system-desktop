//! REQ ↔ 設計書 ↔ テストのトレーサビリティ生成・検証ツール（開発ツール）
//!
//! `cargo run --bin generate_traceability` で repo root の
//! `docs/function-design/` 配下に 90-traceability.md を再生成する。
//! `cargo run --bin generate_traceability -- --check` は無書込で 4 検証を行う:
//!
//! - `[T1]` 生成物 drift（vendor-in 済みファイルと再生成結果の byte 比較）= ERROR
//! - `[T2]` phantom REQ（インベントリ外の REQ ID 使用）= ERROR
//! - `[T3]` テスト 0 本かつ coverage required の REQ = WARN（exit 0 のまま）
//! - `[T4]` REQ/UI ID 未参照の FE テストファイル数 baseline 突合（増減両方向）= ERROR
//!
//! 入力:
//! - `docs/spec/requirements.md`（REQ インベントリ。T2/T3 の判定基盤）
//! - FUNCTION_DESIGN.md の索引リンク（タスク ID → 設計書ファイル。`〜` 範囲 / `/` 連記を展開）
//! - 各設計書の `> 対応仕様:` 行（REQ → 設計書の直接対応）
//! - `src-tauri/src` / `src-tauri/tests` のテスト関数名（`_reqNNN`、境界は `_` か終端）
//! - `src/**/*.test.{ts,tsx}` の `REQ-NNN` / `UI-NN` 参照
//!
//! 仕様 ID: WF-TRACE-01〜04（SPEC-WF-TRACE-2026-06-11）。
//! パス解決は dev-tools の CARGO_MANIFEST_DIR から 2 つ上（実行時 cwd 非依存、D-137 (3)）。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use regex::Regex;

/// 生成物の出力先（repo root からの相対パス）
const OUTPUT_RELATIVE: &str = "docs/function-design/90-traceability.md";
/// REQ インベントリ（repo root からの相対パス）
const INVENTORY_RELATIVE: &str = "docs/spec/requirements.md";
/// 設計書索引（repo root からの相対パス）
const INDEX_RELATIVE: &str = "docs/FUNCTION_DESIGN.md";
/// 関数設計書ディレクトリ（repo root からの相対パス）
const DESIGN_DIR_RELATIVE: &str = "docs/function-design";
/// WF-TRACE-04: REQ/UI ID 未参照の FE テストファイル数 baseline。
/// 増減どちらも `--check` ERROR。意図的に減らした場合はこの値を更新して再生成する。
/// shared UI primitive の contract test を新設して意図的に増やした場合も同様に
/// 本定数と本 comment を更新する（画面 ID の捏造は行わない）。
/// 2026-06-13 PR-B: 17 → 22。画面非依存の共通部品 unit test 5 本
/// （patterns/ PageHeader / FormSection / SummaryCard / EmptyState / DepartmentFilter）は
/// 特定 REQ/UI に紐づかないため意図的に未参照とする。画面文脈を持つ
/// characterization / SearchBar test には REQ/UI ID を付与済み。
/// 2026-09-05 Lane 5（`agent/ui-list-backbone-d-lane5`）: 22 → 24。
/// `src/components/ui/button.test.tsx` / `badge.test.tsx`（shared UI primitive の
/// class 契約 test、画面非依存のため REQ/UI ID を付与しない。packet L5-D6）。
/// 2026-09-06 Lane 4 Gated Amendment 1/2: 24 → 26。
/// `src/features/products/ProductListPage.scroll-restoration.test.tsx`（GA1d、商品一覧
/// scroll 箱の scroll restoration 契約 test）/
/// `src/features/products/components/PriceRevisionFilters.test.tsx`（GA2、取引先
/// label/Select/追加ボタンの群化 class 契約 test）。いずれも画面非依存の shared UI
/// primitive／既存 lane 由来の scroll 契約 test で REQ/UI ID を付与しない（packet L5-D6 先例）。
const FE_UNREFERENCED_BASELINE: usize = 26;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check_mode = match args.as_slice() {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => {
            eprintln!("使い方: cargo run --bin generate_traceability [-- --check]");
            return ExitCode::from(2);
        }
    };

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

    if check_mode {
        match run_check(&repo_root, FE_UNREFERENCED_BASELINE) {
            Ok(report) => {
                for warning in &report.warnings {
                    println!("WARN {warning}");
                }
                if report.errors.is_empty() {
                    println!(
                        "traceability check: OK（ERROR 0 件 / WARN {} 件）",
                        report.warnings.len()
                    );
                    ExitCode::SUCCESS
                } else {
                    for error in &report.errors {
                        eprintln!("ERROR {error}");
                    }
                    eprintln!("traceability check: ERROR {} 件", report.errors.len());
                    ExitCode::FAILURE
                }
            }
            Err(message) => {
                eprintln!("ERROR traceability check を実行できません: {message}");
                ExitCode::FAILURE
            }
        }
    } else {
        match run_generate(&repo_root, FE_UNREFERENCED_BASELINE) {
            Ok(summary) => {
                println!("{OUTPUT_RELATIVE} を生成しました（{summary}）");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("ERROR 生成に失敗しました: {message}");
                ExitCode::FAILURE
            }
        }
    }
}

// ---------------------------------------------------------------------------
// データ収集
// ---------------------------------------------------------------------------

/// インベントリ 1 行（REQ ID / 名称 / 対応タスク / 出典 / coverage policy）
#[derive(Debug, Clone, PartialEq, Eq)]
struct InventoryRow {
    id: String,
    name: String,
    tasks: Vec<String>,
    source: String,
    coverage_policy: CoveragePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CoveragePolicy {
    Required,
    Deferred,
}

impl CoveragePolicy {
    fn parse(value: &str) -> Self {
        match value.trim() {
            "deferred" => Self::Deferred,
            _ => Self::Required,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Deferred => "deferred",
        }
    }
}

fn is_deferred_no_test(entry: &ReqEntry) -> bool {
    entry.row.coverage_policy == CoveragePolicy::Deferred
        && entry.rust_tests.is_empty()
        && entry.fe_refs.is_empty()
}

fn has_no_tests(entry: &ReqEntry) -> bool {
    entry.rust_tests.is_empty() && entry.fe_refs.is_empty()
}

/// REQ 1 件分のトレーサビリティ情報
#[derive(Debug)]
struct ReqEntry {
    row: InventoryRow,
    /// 設計書ファイル名（function-design 配下、同ディレクトリリンク用）
    docs: BTreeSet<String>,
    /// Rust テスト: repo 相対ファイルパス → テスト関数数
    rust_tests: BTreeMap<String, usize>,
    /// FE テスト: repo 相対ファイルパス → REQ 参照箇所数
    fe_refs: BTreeMap<String, usize>,
}

/// repo 全体の走査結果
#[derive(Debug)]
struct TraceData {
    entries: Vec<ReqEntry>,
    /// REQ/UI ID 未参照の FE テストファイル（repo 相対パス、sort 済み）
    fe_unreferenced: Vec<String>,
    /// インベントリ外で使用された REQ ID → 使用ファイル
    phantom: BTreeMap<String, BTreeSet<String>>,
}

/// `--check` の結果（ERROR で exit 1、WARN は exit 0 のまま）
#[derive(Debug)]
struct CheckReport {
    errors: Vec<String>,
    warnings: Vec<String>,
}

fn collect(repo_root: &Path) -> Result<TraceData, String> {
    let inventory_path = repo_root.join(INVENTORY_RELATIVE);
    let inventory_content = read_text(&inventory_path)?;
    let inventory = parse_inventory(&inventory_content);
    if inventory.is_empty() {
        return Err(format!(
            "{INVENTORY_RELATIVE} から REQ 行を 1 行も読めませんでした"
        ));
    }

    let index_content = read_text(&repo_root.join(INDEX_RELATIVE))?;
    let task_to_docs = parse_index_links(&index_content);
    let direct_docs = collect_direct_spec_docs(&repo_root.join(DESIGN_DIR_RELATIVE))?;

    let known_ids: BTreeSet<String> = inventory.iter().map(|row| row.id.clone()).collect();
    let mut rust_tests: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut fe_refs: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut phantom: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    // Rust テスト走査（src-tauri/src + src-tauri/tests）
    let mut rust_files = Vec::new();
    collect_files(
        &repo_root.join("src-tauri/src"),
        &is_rust_file,
        &mut rust_files,
    )?;
    collect_files(
        &repo_root.join("src-tauri/tests"),
        &is_rust_file,
        &mut rust_files,
    )?;
    rust_files.sort();
    for file in &rust_files {
        let rel = relative_display(repo_root, file);
        let content = read_text(file)?;
        for fn_name in rust_test_fn_names(&content) {
            for id in req_ids_from_fn_name(&fn_name) {
                if known_ids.contains(&id) {
                    *rust_tests
                        .entry(id)
                        .or_default()
                        .entry(rel.clone())
                        .or_insert(0) += 1;
                } else {
                    phantom.entry(id).or_default().insert(rel.clone());
                }
            }
        }
    }

    // FE テスト走査（src/**/*.test.{ts,tsx}）
    let mut fe_files = Vec::new();
    collect_files(&repo_root.join("src"), &is_fe_test_file, &mut fe_files)?;
    fe_files.sort();
    let mut fe_unreferenced = Vec::new();
    for file in &fe_files {
        let rel = relative_display(repo_root, file);
        let content = read_text(file)?;
        if !fe_file_references_ids(&content) {
            fe_unreferenced.push(rel.clone());
        }
        for (id, count) in fe_req_ref_counts(&content) {
            if known_ids.contains(&id) {
                *fe_refs
                    .entry(id)
                    .or_default()
                    .entry(rel.clone())
                    .or_insert(0) += count;
            } else {
                phantom.entry(id).or_default().insert(rel.clone());
            }
        }
    }
    fe_unreferenced.sort();

    let entries = inventory
        .into_iter()
        .map(|row| {
            let mut docs: BTreeSet<String> = BTreeSet::new();
            for task in &row.tasks {
                if let Some(files) = task_to_docs.get(task) {
                    docs.extend(files.iter().cloned());
                }
            }
            if let Some(files) = direct_docs.get(&row.id) {
                docs.extend(files.iter().cloned());
            }
            let rust = rust_tests.remove(&row.id).unwrap_or_default();
            let fe = fe_refs.remove(&row.id).unwrap_or_default();
            ReqEntry {
                row,
                docs,
                rust_tests: rust,
                fe_refs: fe,
            }
        })
        .collect();

    Ok(TraceData {
        entries,
        fe_unreferenced,
        phantom,
    })
}

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{} を読めません: {e}", path.display()))
}

fn is_rust_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "rs")
}

fn is_fe_test_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    name.ends_with(".test.ts") || name.ends_with(".test.tsx")
}

/// 再帰走査。node_modules / target / routeTree.gen.ts は対象外。
/// ディレクトリ不在は設定異常として上位へ伝搬する（空 Vec で握りつぶさない）。
fn collect_files(
    dir: &Path,
    predicate: &dyn Fn(&Path) -> bool,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("{} を走査できません: {e}", dir.display()))?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{} の走査中にエラー: {e}", dir.display()))?;
        paths.push(entry.path());
    }
    paths.sort();
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if path.is_dir() {
            if name == "node_modules" || name == "target" {
                continue;
            }
            collect_files(&path, predicate, out)?;
        } else if name != "routeTree.gen.ts" && predicate(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn relative_display(repo_root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(repo_root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

// ---------------------------------------------------------------------------
// 抽出ロジック（純関数）
// ---------------------------------------------------------------------------

/// テスト関数名から REQ ID を抽出する。
/// `_req(\d{3})` の直後が `_` か終端のときのみ採用（`_req9051` / `_req905x` は不一致）。
/// multi-REQ 名（`_req101_req102_`）は全件抽出する（pre-push step ④ と境界整合）。
fn req_ids_from_fn_name(name: &str) -> Vec<String> {
    let re = match Regex::new(r"_req([0-9]{3})") {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };
    let mut ids = Vec::new();
    for caps in re.captures_iter(name) {
        let whole = match caps.get(0) {
            Some(m) => m,
            None => continue,
        };
        let next = name[whole.end()..].chars().next();
        if next.is_none() || next == Some('_') {
            if let Some(digits) = caps.get(1) {
                ids.push(format!("REQ-{}", digits.as_str()));
            }
        }
    }
    ids
}

/// Rust ソースからテスト関数名（`fn test_...`）を行頭基準で抽出する。
fn rust_test_fn_names(content: &str) -> Vec<String> {
    let re = match Regex::new(r"(?m)^\s*fn\s+(test_[a-z0-9_]+)\s*\(") {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };
    re.captures_iter(content)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// インベントリのテーブル行（`| REQ-NNN | 名称 | 対応タスク | 出典 | [coverage] |`）を読む。
/// 5 列目の coverage は任意で、未指定時は required とする。
/// 形式逸脱行は取り込まない（その REQ がテストで使われていれば T2 phantom で顕在化する）。
fn parse_inventory(content: &str) -> Vec<InventoryRow> {
    content
        .lines()
        .filter_map(|line| {
            let cells = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            if cells.len() < 4 || !cells[0].starts_with("REQ-") {
                return None;
            }
            let id = cells[0].to_string();
            let name = cells[1].to_string();
            let tasks = cells[2]
                .split([',', '、'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            let source = cells[3].to_string();
            let coverage_policy = cells
                .get(4)
                .map(|value| CoveragePolicy::parse(value))
                .unwrap_or(CoveragePolicy::Required);
            Some(InventoryRow {
                id,
                name,
                tasks,
                source,
                coverage_policy,
            })
        })
        .collect()
}

/// 索引のリンク `[リンクテキスト](function-design/ファイル名.md)` を読み、
/// リンクテキスト内のタスク ID → 設計書ファイル名 map を作る。
/// `CMD-02〜05`（範囲）と `CMD-09/10/11`（連記）は全タスクに展開する（黙って落とさない）。
fn parse_index_links(content: &str) -> BTreeMap<String, BTreeSet<String>> {
    let link_re = match Regex::new(r"\[([^\]]+)\]\(function-design/([A-Za-z0-9_.-]+\.md)\)") {
        Ok(re) => re,
        Err(_) => return BTreeMap::new(),
    };
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for caps in link_re.captures_iter(content) {
        let (text, file) = match (caps.get(1), caps.get(2)) {
            (Some(t), Some(f)) => (t.as_str(), f.as_str()),
            _ => continue,
        };
        for task in expand_task_ids(text) {
            map.entry(task).or_default().insert(file.to_string());
        }
    }
    map
}

/// リンクテキストからタスク ID 群を展開する。
/// 完全形 `PREFIX-NN[x]` に加え、直前の完全形を引き継ぐ `〜NN`（範囲）/ `/NN`（連記）を解釈する。
fn expand_task_ids(text: &str) -> Vec<String> {
    let re = match Regex::new(r"\b(UI|CMD|BIZ|IO|MNT)-([0-9]{2})([a-z])?|[〜～/]([0-9]{2})([a-z])?")
    {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };
    let mut ids = Vec::new();
    let mut last_prefix: Option<String> = None;
    let mut last_num: Option<u32> = None;
    for caps in re.captures_iter(text) {
        if let (Some(prefix), Some(num)) = (caps.get(1), caps.get(2)) {
            let suffix = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            ids.push(format!("{}-{}{}", prefix.as_str(), num.as_str(), suffix));
            last_prefix = Some(prefix.as_str().to_string());
            last_num = num.as_str().parse::<u32>().ok();
        } else if let Some(num) = caps.get(4) {
            let Some(prefix) = last_prefix.clone() else {
                continue;
            };
            let suffix = caps.get(5).map(|m| m.as_str()).unwrap_or("");
            let Ok(n) = num.as_str().parse::<u32>() else {
                continue;
            };
            let whole = match caps.get(0) {
                Some(m) => m.as_str(),
                None => continue,
            };
            if whole.starts_with('/') {
                ids.push(format!("{prefix}-{:02}{suffix}", n));
            } else {
                // 範囲表記: 直前番号の次から終端まで展開する
                let start = last_num.map_or(n, |prev| prev + 1);
                for i in start..=n {
                    ids.push(format!("{prefix}-{:02}{suffix}", i));
                }
            }
            last_num = Some(n);
        }
    }
    ids
}

/// 設計書の `> 対応仕様:` 行から REQ → 設計書ファイルの直接対応を集める。
fn collect_direct_spec_docs(
    design_dir: &Path,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let mut files = Vec::new();
    collect_files(design_dir, &is_markdown_file, &mut files)?;
    files.sort();
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in &files {
        let name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name == "90-traceability.md" {
            continue; // 生成物自身は入力にしない
        }
        let content = read_text(file)?;
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('>') && trimmed.contains("対応仕様:") {
                for id in req_ids_in_text(line) {
                    map.entry(id).or_default().insert(name.clone());
                }
            }
        }
    }
    Ok(map)
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
}

/// テキストから `REQ-NNN` を抽出する（直後が数字なら不一致 = 3 桁固定）。
fn req_ids_in_text(text: &str) -> Vec<String> {
    let re = match Regex::new(r"\bREQ-([0-9]{3})") {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };
    let mut ids = Vec::new();
    for caps in re.captures_iter(text) {
        let whole = match caps.get(0) {
            Some(m) => m,
            None => continue,
        };
        let next = text[whole.end()..].chars().next();
        if next.is_some_and(|c| c.is_ascii_digit()) {
            continue;
        }
        if let Some(digits) = caps.get(1) {
            ids.push(format!("REQ-{}", digits.as_str()));
        }
    }
    ids
}

/// FE テストファイル内の REQ 参照箇所数（REQ ID → 件数）。
fn fe_req_ref_counts(content: &str) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for id in req_ids_in_text(content) {
        *counts.entry(id).or_insert(0) += 1;
    }
    counts
}

/// FE テストファイルが REQ-NNN / UI-NN ID を参照しているか（T4 presence 判定）。
/// 左境界付き: `GUI-12` は不一致。`UI-WF-...` は数字 2 桁が続かないため不一致。
/// 右境界付き: 桁数 typo（`REQ-3010` / `UI-010`）も suffix typo（`UI-01ab`）も不一致
/// （PR #96 Codex Round 1-2 P2）。`regex` crate は lookahead 非対応のため
/// `[a-z]?\b` で「任意の 1 文字 suffix を消費した後に語境界」を要求する
/// （`UI-01a` / `UI-01a-D1` は `-` が非語文字なので境界成立、一致を維持）。
fn fe_file_references_ids(content: &str) -> bool {
    let re = match Regex::new(r"\b(REQ-[0-9]{3}\b|UI-[0-9]{2}[a-z]?\b)") {
        Ok(re) => re,
        Err(_) => return false,
    };
    re.is_match(content)
}

// ---------------------------------------------------------------------------
// 生成・検証
// ---------------------------------------------------------------------------

fn render(data: &TraceData, baseline: usize) -> String {
    let mut covered = 0usize;
    let mut rust_only = 0usize;
    let mut fe_only = 0usize;
    let mut no_test = 0usize;
    let mut deferred = 0usize;
    for entry in &data.entries {
        if is_deferred_no_test(entry) {
            deferred += 1;
            continue;
        }
        match (!entry.rust_tests.is_empty(), !entry.fe_refs.is_empty()) {
            (true, true) => covered += 1,
            (true, false) => rust_only += 1,
            (false, true) => fe_only += 1,
            (false, false) => no_test += 1,
        }
    }

    let mut out = String::new();
    out.push_str("# REQ トレーサビリティマトリクス\n\n");
    out.push_str("> **親文書**: [FUNCTION_DESIGN.md](../FUNCTION_DESIGN.md)\n");
    out.push_str(
        "> **AUTO-GENERATED**: `generate_traceability` bin による自動生成ファイル。手動編集しない\n",
    );
    out.push_str("> 再生成: `cd src-tauri && cargo run --bin generate_traceability`\n");
    out.push_str(
        "> 検証: `cd src-tauri && cargo run --bin generate_traceability -- --check`（T1 drift / T2 phantom REQ / T3 required no-test / T4 FE baseline）\n",
    );
    out.push_str(
        "> Non-scope (v1): SP-NNN / QR-NN / UI 設計決定 ID（UI-NNx-Dn）の matrix 化、xlsx 自動抽出\n\n",
    );

    out.push_str("## 1. サマリ\n\n");
    out.push_str("| 区分 | 件数 |\n|---|---|\n");
    out.push_str(&format!("| REQ 総数 | {} |\n", data.entries.len()));
    out.push_str(&format!(
        "| covered（Rust / FE 両方にテストあり） | {covered} |\n"
    ));
    out.push_str(&format!("| rust-only（Rust テストのみ） | {rust_only} |\n"));
    out.push_str(&format!("| fe-only（FE テストのみ） | {fe_only} |\n"));
    out.push_str(&format!("| no-test（テスト 0 本） | {no_test} |\n"));
    out.push_str(&format!(
        "| deferred（未実装のため no-test WARN 対象外） | {deferred} |\n\n"
    ));

    out.push_str("## 2. REQ 別マトリクス\n\n");
    out.push_str(
        "| REQ | 名称 | 対応タスク | coverage | 設計書 | Rust テスト | FE テスト参照 | 状態 |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    for entry in &data.entries {
        let docs = if entry.docs.is_empty() {
            "—".to_string()
        } else {
            entry
                .docs
                .iter()
                .map(|f| format!("[{f}]({f})"))
                .collect::<Vec<_>>()
                .join("<br>")
        };
        let rust = if entry.rust_tests.is_empty() {
            "—".to_string()
        } else {
            entry
                .rust_tests
                .iter()
                .map(|(file, count)| format!("`{file}` {count}件"))
                .collect::<Vec<_>>()
                .join("<br>")
        };
        let fe = if entry.fe_refs.is_empty() {
            "—".to_string()
        } else {
            entry
                .fe_refs
                .iter()
                .map(|(file, count)| format!("`{file}` {count}箇所"))
                .collect::<Vec<_>>()
                .join("<br>")
        };
        let status = if is_deferred_no_test(entry) {
            "deferred"
        } else {
            match (!entry.rust_tests.is_empty(), !entry.fe_refs.is_empty()) {
                (true, true) => "covered",
                (true, false) => "rust-only",
                (false, true) => "fe-only",
                (false, false) => "no-test",
            }
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            entry.row.id,
            entry.row.name,
            entry.row.tasks.join(", "),
            entry.row.coverage_policy.as_str(),
            docs,
            rust,
            fe,
            status
        ));
    }
    out.push('\n');

    out.push_str("## 3. FE テスト ID 参照状況（T4 baseline）\n\n");
    out.push_str(&format!(
        "- REQ/UI ID 未参照の FE テストファイル数 baseline: {baseline}（増減どちらも `--check` ERROR）\n",
    ));
    out.push_str(
        "- baseline を意図的に下げた場合は bin 内 `FE_UNREFERENCED_BASELINE` を更新して再生成する\n\n",
    );

    out.push_str("## 4. 付録: REQ/UI ID 未参照の FE テストファイル\n\n");
    if data.fe_unreferenced.is_empty() {
        out.push_str("- なし\n");
    } else {
        for file in &data.fe_unreferenced {
            out.push_str(&format!("- `{file}`\n"));
        }
    }

    normalize_markdown(&out)
}

/// trailing whitespace を除去し、末尾改行を 1 つに揃える（決定性担保）。
fn normalize_markdown(content: &str) -> String {
    let mut out = content
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    out.push('\n');
    out
}

fn run_generate(repo_root: &Path, baseline: usize) -> Result<String, String> {
    let data = collect(repo_root)?;
    let rendered = render(&data, baseline);
    let output_path = repo_root.join(OUTPUT_RELATIVE);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("{} を作成できません: {e}", parent.display()))?;
    }
    fs::write(&output_path, &rendered)
        .map_err(|e| format!("{} に書き込めません: {e}", output_path.display()))?;
    Ok(format!(
        "REQ {} 行 / FE 未参照 {} ファイル",
        data.entries.len(),
        data.fe_unreferenced.len()
    ))
}

/// `--check` 本体（無書込）。T1 drift / T2 phantom / T3 テスト 0 本 / T4 FE baseline。
fn run_check(repo_root: &Path, baseline: usize) -> Result<CheckReport, String> {
    let data = collect(repo_root)?;
    let rendered = render(&data, baseline);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // T1: 生成物 drift
    let output_path = repo_root.join(OUTPUT_RELATIVE);
    match fs::read_to_string(&output_path) {
        Ok(existing) => {
            if existing != rendered {
                errors.push(format!(
                    "[T1] {OUTPUT_RELATIVE} が最新の生成結果と一致しません。修正手順: ローカルで `cd src-tauri && cargo run --bin generate_traceability` を実行し、生成結果を commit に含めてください"
                ));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            errors.push(format!(
                "[T1] {OUTPUT_RELATIVE} がありません。修正手順: ローカルで `cd src-tauri && cargo run --bin generate_traceability` を実行し、生成結果を commit に含めてください"
            ));
        }
        Err(e) => {
            return Err(format!("{} を読めません: {e}", output_path.display()));
        }
    }

    // T2: phantom REQ
    for (id, files) in &data.phantom {
        let file_list = files.iter().cloned().collect::<Vec<_>>().join(", ");
        errors.push(format!(
            "[T2] インベントリ（{INVENTORY_RELATIVE}）にない {id} がテストで使用されています（{file_list}）。REQ 番号の typo を修正するか、インベントリに行を追加してください"
        ));
    }

    // T3: coverage required かつテスト 0 本の REQ（WARN のみ）
    for entry in &data.entries {
        if has_no_tests(entry) && entry.row.coverage_policy == CoveragePolicy::Required {
            warnings.push(format!(
                "[T3] テスト 0 本の REQ: {}（{}）",
                entry.row.id, entry.row.name
            ));
        }
    }

    // T4: FE 未参照ファイル数 baseline（両方向）
    let current = data.fe_unreferenced.len();
    if current != baseline {
        errors.push(format!(
            "[T4] REQ/UI ID 未参照の FE テストファイル数が baseline と一致しません（baseline {baseline} / 現在 {current}）。増えた場合は新しいテストの describe/it に REQ-NNN / UI-NN ID を含めてください。意図的に減らした場合は FE_UNREFERENCED_BASELINE を {current} に更新し、再生成してください"
        ));
    }

    Ok(CheckReport { errors, warnings })
}

// ---------------------------------------------------------------------------
// テスト（meta/workflow テスト規約: 接頭辞なし命名、仕様 ID は WF-TRACE-N コメント）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // WF-TRACE-2: multi-REQ テスト名から全 REQ を抽出する（境界は `_` または終端）
    #[test]
    fn req_ids_from_fn_name_extracts_multi_req() {
        let ids = req_ids_from_fn_name("test_apply_req101_req102_normal");
        assert_eq!(ids, vec!["REQ-101".to_string(), "REQ-102".to_string()]);
        let tail = req_ids_from_fn_name("test_apply_change_req303");
        assert_eq!(tail, vec!["REQ-303".to_string()]);
    }

    // WF-TRACE-2: `_req9051`（4 桁直結）/ `_req905x`（英字直結）は REQ-905 に誤計上しない
    #[test]
    fn req_ids_from_fn_name_rejects_req9051_boundary() {
        assert!(req_ids_from_fn_name("boundary_req9051_case").is_empty());
        assert!(req_ids_from_fn_name("boundary_req905x_case").is_empty());
    }

    // WF-TRACE-2 / WF-TRACE-3: インベントリの REQ 行（4 列 / 5 列）を読む
    #[test]
    fn inventory_parse_reads_req_rows() {
        let content = "\
| REQ ID | 名称 | 対応タスク | 出典 |
|---|---|---|---|
| REQ-101 | 商品を新規登録できること | UI-01b, BIZ-01 | 要求仕様書 v2.1 |
| REQ-403 | 整合性を検証できること | UI-13 | 要求仕様書 v2.1 | deferred |
";
        let rows = parse_inventory(content);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "REQ-101");
        assert_eq!(rows[0].name, "商品を新規登録できること");
        assert_eq!(
            rows[0].tasks,
            vec!["UI-01b".to_string(), "BIZ-01".to_string()]
        );
        assert_eq!(rows[0].source, "要求仕様書 v2.1");
        assert_eq!(rows[0].coverage_policy, CoveragePolicy::Required);
        assert_eq!(rows[1].id, "REQ-403");
        assert_eq!(rows[1].tasks, vec!["UI-13".to_string()]);
        assert_eq!(rows[1].coverage_policy, CoveragePolicy::Deferred);
    }

    // 生成入力: 索引リンクの範囲表記（〜）と連記（/）をタスク ID に展開する
    #[test]
    fn index_links_parse_expands_range_and_slash_ids() {
        let content = "\
- [CMD-02〜05: 入出庫コマンド群 / CMD-06: 在庫照会コマンド群](function-design/44-cmd-inventory.md)
- [CMD-09/10/11部分: 売上集計・棚卸し・整合性コマンド群](function-design/42-cmd-sales-stocktake.md)
- [UI-01b: 商品登録・編集](function-design/51-ui-product-form.md)
";
        let map = parse_index_links(content);
        for task in ["CMD-02", "CMD-03", "CMD-04", "CMD-05", "CMD-06"] {
            assert!(
                map.get(task)
                    .is_some_and(|s| s.contains("44-cmd-inventory.md")),
                "{task} が 44-cmd-inventory.md に展開されていない"
            );
        }
        for task in ["CMD-09", "CMD-10", "CMD-11"] {
            assert!(
                map.get(task)
                    .is_some_and(|s| s.contains("42-cmd-sales-stocktake.md")),
                "{task} が 42-cmd-sales-stocktake.md に展開されていない"
            );
        }
        assert!(map
            .get("UI-01b")
            .is_some_and(|s| s.contains("51-ui-product-form.md")));
    }

    // WF-TRACE-4: UI 設計決定 ID（UI-01a-D1）は参照済み扱い、UI-WF-... / GUI-12 は未参照扱い
    #[test]
    fn fe_presence_counts_ui_decision_ids_as_referenced() {
        assert!(fe_file_references_ids("// UI-01a-D1 の検証"));
        assert!(fe_file_references_ids("describe('REQ-301 在庫照会')"));
        assert!(fe_file_references_ids("it('REQ-301: 検索できる')"));
        assert!(fe_file_references_ids("describe('UI-12 表示サイズ')"));
        assert!(!fe_file_references_ids(
            "// UI-WF-2026-05-22 形式の workflow ID"
        ));
        assert!(!fe_file_references_ids("// GUI-12 は誤一致しない"));
    }

    // WF-TRACE-4: 桁数 typo（REQ-3010 / UI-010）は ID 参照と見なさない（右境界、PR #96 Codex P2）
    #[test]
    fn fe_presence_rejects_digit_overrun_ids() {
        assert!(!fe_file_references_ids("// REQ-3010 は桁数 typo"));
        assert!(!fe_file_references_ids("describe('UI-010 桁数 typo')"));
    }

    // WF-TRACE-4: suffix typo（UI-01ab）は ID 参照と見なさない（suffix 消費後も右境界、PR #96 Codex Round 2 P2）
    #[test]
    fn fe_presence_rejects_suffix_overrun_ids() {
        assert!(!fe_file_references_ids("// UI-01ab は suffix typo"));
        assert!(fe_file_references_ids("// UI-01a-D1 は有効形を維持"));
        assert!(fe_file_references_ids("describe('UI-12')"));
    }

    // -----------------------------------------------------------------------
    // fixture（tempdir 内の合成ツリー。設計書ファイル名は実在名のみ使用 = R1 整合）
    // -----------------------------------------------------------------------

    fn write_file(root: &std::path::Path, rel: &str, content: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture ディレクトリ作成");
        }
        std::fs::write(&path, content).expect("fixture ファイル書き込み");
    }

    /// 実ファイル中に行頭 `fn test_...` を残さないため、fn 行は連結で組み立てる
    /// （pre-push step ④ と自走査の誤検知防止）。
    fn rust_source_with_test_fns(names: &[&str]) -> String {
        let mut src = String::new();
        for name in names {
            src.push_str(&format!("{} {}() {{}}\n", "fn", name));
        }
        src
    }

    fn build_fixture(root: &std::path::Path) {
        write_file(
            root,
            "docs/spec/requirements.md",
            "\
| REQ ID | 名称 | 対応タスク | 出典 |
|---|---|---|---|
| REQ-101 | 商品を新規登録できること | UI-01b, BIZ-01 | 要求仕様書 v2.1 |
| REQ-303 | 在庫変動履歴を参照できること | UI-06c | 要求仕様書 v2.1 |
| REQ-403 | 整合性を検証できること | UI-13 | 要求仕様書 v2.1 | deferred |
",
        );
        write_file(
            root,
            "docs/FUNCTION_DESIGN.md",
            "\
- [BIZ-01: 商品管理ロジック](function-design/30-biz-product-service.md)
- [UI-01b: 商品登録・編集](function-design/51-ui-product-form.md)
",
        );
        write_file(
            root,
            "docs/function-design/51-ui-product-form.md",
            "> 対応仕様: REQ-101 / UI-01b\n",
        );
        write_file(
            root,
            "src-tauri/src/biz/product_service.rs",
            &rust_source_with_test_fns(&[
                "test_create_product_req101_normal",
                "test_apply_req101_req303_multi",
            ]),
        );
        std::fs::create_dir_all(root.join("src-tauri/tests")).expect("tests ディレクトリ作成");
        write_file(
            root,
            "src/features/products/form.test.tsx",
            "describe('REQ-101 商品登録フォーム', () => {});\n",
        );
        write_file(
            root,
            "src/features/products/util.test.ts",
            "describe('純関数（ID 参照なし）', () => {});\n",
        );
    }

    // WF-TRACE-1: 同一入力からの生成は byte 単位で決定的（timestamp なし、全コレクション sort）
    #[test]
    fn generation_is_deterministic() {
        let dir = TempDir::new().expect("tempdir");
        build_fixture(dir.path());
        let first = render(&collect(dir.path()).expect("collect 1 回目"), 1);
        let second = render(&collect(dir.path()).expect("collect 2 回目"), 1);
        assert_eq!(first, second);
        assert!(first.ends_with('\n'));
        assert!(!first.ends_with("\n\n"));
        // 設計書列にタスク経由 + 対応仕様直接対応の両方が入る
        assert!(first.contains("[30-biz-product-service.md](30-biz-product-service.md)"));
        assert!(first.contains("[51-ui-product-form.md](51-ui-product-form.md)"));
    }

    // WF-TRACE-1: 生成直後の --check は clean（exit 0 相当 = errors 空）
    #[test]
    fn check_mode_passes_when_clean() {
        let dir = TempDir::new().expect("tempdir");
        build_fixture(dir.path());
        run_generate(dir.path(), 1).expect("生成");
        let report = run_check(dir.path(), 1).expect("check");
        assert!(
            report.errors.is_empty(),
            "clean tree で ERROR: {:?}",
            report.errors
        );
    }

    // WF-TRACE-1: 生成物の改変（drift）を ERROR で検知する
    #[test]
    fn check_mode_detects_drift() {
        let dir = TempDir::new().expect("tempdir");
        build_fixture(dir.path());
        run_generate(dir.path(), 1).expect("生成");
        let output = dir.path().join(OUTPUT_RELATIVE);
        let mut content = std::fs::read_to_string(&output).expect("生成物読込");
        content.push_str("manual edit\n");
        std::fs::write(&output, content).expect("生成物改変");
        let report = run_check(dir.path(), 1).expect("check");
        assert!(
            report.errors.iter().any(|e| e.starts_with("[T1]")),
            "drift が T1 ERROR にならない: {:?}",
            report.errors
        );
    }

    // WF-TRACE-2: インベントリ外の REQ 使用は ERROR
    #[test]
    fn check_mode_fails_on_phantom_req() {
        let dir = TempDir::new().expect("tempdir");
        build_fixture(dir.path());
        write_file(
            dir.path(),
            "src-tauri/src/biz/phantom_module.rs",
            &rust_source_with_test_fns(&["test_phantom_req999_normal"]),
        );
        run_generate(dir.path(), 1).expect("生成");
        let report = run_check(dir.path(), 1).expect("check");
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.starts_with("[T2]") && e.contains("REQ-999")),
            "phantom REQ が T2 ERROR にならない: {:?}",
            report.errors
        );
    }

    // WF-TRACE-3: coverage deferred のテスト 0 本 REQ は WARN 対象外
    #[test]
    fn summary_counts_deferred_req_without_warning() {
        let dir = TempDir::new().expect("tempdir");
        build_fixture(dir.path());
        run_generate(dir.path(), 1).expect("生成");
        let report = run_check(dir.path(), 1).expect("check");
        assert!(
            report.warnings.is_empty(),
            "deferred の REQ-403 が T3 WARN になっている: {:?}",
            report.warnings
        );
        assert!(report.errors.is_empty());
        // 生成物のサマリでは no-test ではなく deferred として現れる
        let rendered = render(&collect(dir.path()).expect("collect"), 1);
        assert!(rendered.contains("| no-test（テスト 0 本） | 0 |"));
        assert!(rendered.contains("| deferred（未実装のため no-test WARN 対象外） | 1 |"));
        assert!(rendered.contains("| REQ-403 | 整合性を検証できること | UI-13 | deferred |"));
    }

    // WF-TRACE-3: coverage required のテスト 0 本 REQ は WARN（ERROR にしない）
    #[test]
    fn summary_counts_required_req_with_no_tests_as_warn() {
        let dir = TempDir::new().expect("tempdir");
        build_fixture(dir.path());
        let inventory = dir.path().join(INVENTORY_RELATIVE);
        let content = std::fs::read_to_string(&inventory)
            .expect("inventory 読込")
            .replace(" | deferred |", " | required |");
        std::fs::write(&inventory, content).expect("inventory 更新");
        run_generate(dir.path(), 1).expect("生成");
        let report = run_check(dir.path(), 1).expect("check");
        assert!(
            report
                .warnings
                .iter()
                .any(|w| w.starts_with("[T3]") && w.contains("REQ-403")),
            "required のテスト 0 本 REQ-403 が T3 WARN に出ない: {:?}",
            report.warnings
        );
        assert!(report.errors.is_empty());
    }

    // WF-TRACE-4: FE 未参照ファイル数の baseline 不一致は増減どちらも ERROR
    #[test]
    fn check_mode_fails_on_baseline_mismatch_both_directions() {
        let dir = TempDir::new().expect("tempdir");
        build_fixture(dir.path()); // 未参照 1 ファイル（util.test.ts）
        run_generate(dir.path(), 1).expect("生成");

        let clean = run_check(dir.path(), 1).expect("check baseline=1");
        assert!(!clean.errors.iter().any(|e| e.starts_with("[T4]")));

        // 増加方向: baseline 0 に対して現在 1
        let over = run_check(dir.path(), 0).expect("check baseline=0");
        assert!(
            over.errors.iter().any(|e| e.starts_with("[T4]")),
            "増加方向で T4 ERROR にならない: {:?}",
            over.errors
        );

        // 減少方向: baseline 2 に対して現在 1
        let under = run_check(dir.path(), 2).expect("check baseline=2");
        assert!(
            under.errors.iter().any(|e| e.starts_with("[T4]")),
            "減少方向で T4 ERROR にならない: {:?}",
            under.errors
        );
    }
}
