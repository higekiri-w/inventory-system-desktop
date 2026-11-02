use std::fs;
use std::path::Path;

fn read_repo_file(relative_path: &str) -> String {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    fs::read_to_string(repo_root.join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn struct_block<'a>(source: &'a str, struct_name: &str) -> &'a str {
    let anchors = [
        format!("pub struct {struct_name} {{"),
        format!("struct {struct_name} {{"),
    ];
    let (start, anchor) = anchors
        .iter()
        .find_map(|anchor| source.find(anchor).map(|start| (start, anchor)))
        .unwrap_or_else(|| panic!("struct {struct_name} not found"));
    let body_start = start + anchor.len();
    let mut depth = 1_i32;

    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }

    panic!("struct {struct_name} is not closed");
}

fn field_names(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|line| {
            let field = line.trim().strip_prefix("pub ").unwrap_or(line.trim());
            if field.is_empty()
                || field.starts_with("//")
                || field.starts_with('#')
                || !field.contains(':')
            {
                return None;
            }
            Some(
                field
                    .split_once(':')
                    .expect("field must contain colon")
                    .0
                    .trim()
                    .to_string(),
            )
        })
        .collect()
}

fn assert_struct_fields(source: &str, struct_name: &str, expected: &[&str]) {
    assert_eq!(
        field_names(struct_block(source, struct_name)),
        expected,
        "{struct_name} must keep its minimal internal field contract"
    );
}

fn typescript_type_fields(source: &str, type_name: &str) -> Vec<String> {
    let anchor = format!("export type {type_name} = {{");
    let start = source
        .find(&anchor)
        .unwrap_or_else(|| panic!("TypeScript type {type_name} not found"))
        + anchor.len();
    let end = source[start..]
        .find("\n};")
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("TypeScript type {type_name} is not closed"));
    source[start..end]
        .lines()
        .filter_map(|line| {
            line.trim()
                .split_once(':')
                .map(|(name, _)| name.to_string())
        })
        .collect()
}

fn markdown_section<'a>(source: &'a str, heading: &str, next_heading: &str) -> &'a str {
    let start = source
        .find(heading)
        .unwrap_or_else(|| panic!("heading {heading} not found"));
    let body_start = start + heading.len();
    let end = source[body_start..]
        .find(next_heading)
        .map_or(source.len(), |offset| body_start + offset);
    &source[body_start..end]
}

fn markdown_field_names(section: &str) -> Vec<String> {
    section
        .lines()
        .filter_map(|line| {
            let field = line.trim().strip_prefix("- ")?;
            let (name, _) = field.split_once(':')?;
            Some(name.trim().to_string())
        })
        .collect()
}

#[test]
fn test_import_internal_contract_req401_is_minimal() {
    // REQ-401 / BIZ-03-D1 / IO-07-D1: internal cache/parser types do not
    // reintroduce consumer-free metadata while diagnostic fields stay intact.
    let csv_source = read_repo_file("src-tauri/src/biz/csv_import_service/mod.rs");
    let daily_source = read_repo_file("src-tauri/src/io/daily_report_parser.rs");
    let csv_design = read_repo_file("docs/function-design/32-biz-csv-import-service.md");
    let daily_design = read_repo_file("docs/function-design/29-io-daily-report-parser.md");

    // 32「preview の cache の契約」（D-118 b1。Matrix T10 の「期待を変える」）: MatchedRow は確定の TX の
    // 照合に使う元の JAN と共有 JAN かどうかを持ち、pos_stock_sync を持たない（確定の TX で取り直す）。
    // CommitRequest は同じ日の追加の確認の field を外し、CMD が渡す今日を持つ
    let matched_fields = [
        "line_no",
        "normalized_jan",
        "product_code",
        "shared_jan",
        "quantity",
        "amount",
    ];
    // I-W4: source design と Rust を単一 exact pin で固定する。
    let commit_fields = ["cached_data", "today"];
    assert_struct_fields(&csv_source, "MatchedRow", &matched_fields);
    assert_struct_fields(&csv_source, "CommitRequest", &commit_fields);
    assert_eq!(
        markdown_field_names(markdown_section(
            &csv_design,
            "#### MatchedRow構造体",
            "#### ErrorRow構造体",
        )),
        matched_fields,
        "MatchedRow design section must keep the minimal field contract"
    );
    assert_eq!(
        markdown_field_names(markdown_section(
            &csv_design,
            "#### CommitRequest構造体",
            "#### ImportResult構造体",
        )),
        commit_fields,
        "CommitRequest design section must keep the minimal field contract"
    );

    for source in [&daily_source, &daily_design] {
        assert_struct_fields(
            source,
            "DailyReportSummaryLine",
            &[
                "line_key",
                "label",
                "amount",
                "quantity_hundredths",
                "count",
                "sort_order",
            ],
        );
        assert_struct_fields(
            source,
            "DailyReportPaymentLine",
            &["payment_key", "label", "amount", "count", "sort_order"],
        );
        assert_struct_fields(
            source,
            "DailyReportDepartmentLine",
            &[
                "raw_department_name",
                "normalized_department_name",
                "amount",
                "quantity_hundredths",
                "count",
                "sort_order",
            ],
        );
        assert_struct_fields(
            source,
            "DailyReportParseError",
            &[
                "source_file",
                "filename",
                "line_no",
                "error_type",
                "error_message",
            ],
        );
    }
}

#[test]
fn test_wire_contract_req401_i_w1_i_w2_i_w3_i_w5_generated_binding_is_atomic() {
    // REQ-401 / I-W1 / I-W2 / I-W3 / I-W5 / SPEC-SDI-D1,D2,D3,D6:
    // producer/consumer generated contract must switch as one exact surface.
    let bindings = read_repo_file("src/lib/bindings.ts");
    for required in [
        "AdditionalImportConfirmationRequired",
        "AlreadyImported",
        "additionalImportConfirmed",
        "same_date_imports",
        "source_import_count",
    ] {
        assert!(
            bindings.contains(required),
            "generated bindings missing {required}"
        );
    }
    for removed in [
        ["Overwrite", "Required"].concat(),
        ["overwrite", "Confirmed"].concat(),
        ["existing", "_import_id"].concat(),
    ] {
        assert!(
            !bindings.contains(&removed),
            "generated bindings retained {removed}"
        );
    }
    assert_eq!(
        typescript_type_fields(&bindings, "OfficialDailyReportSummary"),
        [
            "source_import_count",
            "report_date",
            "gross_amount",
            "net_amount",
            "payment_lines",
            "department_lines",
            "warnings",
            "summary_imports",
        ],
        "OfficialDailyReportSummary must use source count without a singular import ID"
    );
    // REQ-501 / D-096 / 34 §19.2: Z001 の取込みごとの行（取込み ID は要素だけが持つ）
    assert_eq!(
        typescript_type_fields(&bindings, "OfficialDailySummaryImport"),
        ["daily_report_import_id", "imported_at", "lines"],
        "OfficialDailySummaryImport must match 34 §19.2"
    );
    assert_eq!(
        typescript_type_fields(&bindings, "OfficialDailySummaryLine"),
        ["label", "quantity", "count", "amount"],
        "OfficialDailySummaryLine must match 34 §19.2 without line_key"
    );
}

#[test]
fn test_sales_design_req502_future_coverage_counts_distinct_report_dates() {
    // REQ-502 / I-R8 / SPEC-SDI-D6: 未実装coverage fieldの将来契約だけをsource docsへpinする。
    for path in [
        "docs/function-design/24-io-csv-import-repo.md",
        "docs/function-design/34-biz-sales-service.md",
    ] {
        let source = read_repo_file(path);
        assert!(
            source.contains("COUNT(DISTINCT report_date)"),
            "{path} must count future coverage by distinct business dates"
        );
    }
}

fn sweep_dir_for_tokens(dir: &Path, tokens: &[String], hits: &mut Vec<String>) {
    // 外部 binary（rg 等）へ依存すると hosted runner に存在せず環境依存 fail する
    // （2026-08-16 hosted CI 実発生）ため、file walk + literal 検索を test 内蔵で行う。
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read_dir failed for {dir:?}: {error}"));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("dir entry failed under {dir:?}: {error}"))
            .path();
        if path.is_dir() {
            sweep_dir_for_tokens(&path, tokens, hits);
            continue;
        }
        // ponytail: 到達可能な既知 gitignore 対象 2 パターンのみ明示 skip。
        // 汎用 .gitignore parser は新規依存になるため導入しない（Plan Packet S1 参照）。
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name == "routeTree.gen.ts" || file_name.ends_with(".rs.bk") {
            continue;
        }
        let bytes =
            fs::read(&path).unwrap_or_else(|error| panic!("read failed for {path:?}: {error}"));
        let text = String::from_utf8_lossy(&bytes);
        for (index, line) in text.lines().enumerate() {
            for token in tokens {
                if line.contains(token.as_str()) {
                    hits.push(format!("{}:{}:{}", path.display(), index + 1, line.trim()));
                }
            }
        }
    }
}

#[test]
fn test_active_sales_import_vocabulary_sweep_i_g1() {
    // REQ-401 / I-G1 / SPEC-SDI-D8: active Rust/TS/generated surfaceに旧取込み契約を残さない。
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let removed = [
        ["Overwrite", "Required"].concat(),
        ["overwrite", "_confirmed"].concat(),
        ["overwrite", "Confirmed"].concat(),
        ["Overwrite", "ConfirmDialog"].concat(),
        ["requires", "Overwrite"].concat(),
        ["existing", "_import_id"].concat(),
        ["get_latest", "_completed_daily_report"].concat(),
    ];
    let mut hits = Vec::new();
    // D-137: 開発用の bin は src-tauri/dev-tools/src にある（sweep の対象に残す）
    for target in [
        "src-tauri/src",
        "src-tauri/dev-tools/src",
        "src-tauri/tests",
        "src",
    ] {
        sweep_dir_for_tokens(&repo_root.join(target), &removed, &mut hits);
    }
    assert!(
        hits.is_empty(),
        "active stale vocabulary:\n{}",
        hits.join("\n")
    );
}

#[test]
fn test_sweep_dir_for_tokens_skips_known_generated_paths() {
    let tokens = ["FORBIDDEN_TOKEN".to_string()];
    let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));

    fs::write(
        dir.path().join("routeTree.gen.ts"),
        "const x = 'FORBIDDEN_TOKEN';",
    )
    .unwrap_or_else(|error| panic!("write routeTree.gen.ts failed: {error}"));
    fs::write(dir.path().join("foo.rs.bk"), "// FORBIDDEN_TOKEN")
        .unwrap_or_else(|error| panic!("write foo.rs.bk failed: {error}"));
    fs::write(dir.path().join("foo.rs"), "// FORBIDDEN_TOKEN")
        .unwrap_or_else(|error| panic!("write foo.rs failed: {error}"));
    // 保護対象 active surface（bindings.ts 等）の除外を見逃さないため、下位ディレクトリの
    // 通常 .ts file も対にする（`.ts` 一括 skip mutant と再帰呼び出し削除 mutant の両方を kill）。
    let lib_dir = dir.path().join("lib");
    fs::create_dir(&lib_dir).unwrap_or_else(|error| panic!("mkdir lib failed: {error}"));
    fs::write(lib_dir.join("bindings.ts"), "// FORBIDDEN_TOKEN")
        .unwrap_or_else(|error| panic!("write lib/bindings.ts failed: {error}"));

    let mut hits = Vec::new();
    sweep_dir_for_tokens(dir.path(), &tokens, &mut hits);

    assert!(
        !hits.iter().any(|hit| hit.contains("routeTree.gen.ts")),
        "routeTree.gen.ts must be skipped: {hits:?}"
    );
    assert!(
        !hits.iter().any(|hit| hit.contains("foo.rs.bk")),
        "*.rs.bk must be skipped: {hits:?}"
    );
    assert!(
        hits.iter().any(|hit| hit.contains("foo.rs:")),
        "normal files must still be swept: {hits:?}"
    );
    assert!(
        hits.iter().any(|hit| hit.contains("bindings.ts:")),
        "nested .ts files (e.g. bindings.ts) must still be swept: {hits:?}"
    );
}
