#!/usr/bin/env bash
# docs/function-design/ 配下の設計書の横断整合性チェック
# PR提出前・レビュー前に実行し、機械検出可能な不整合を事前に排除する
#
# 使い方:
#   ./scripts/doc-consistency-check.sh                       # 設計書チェック + active Plan Packet チェック
#   ./scripts/doc-consistency-check.sh --target plan [file]  # プランチェック + active Plan Packet チェック
#   ./scripts/doc-consistency-check.sh --fix                 # (将来: 自動修正モード)
#
# 終了コード: 0=問題なし, 1=不整合あり

set -euo pipefail

DOCS_DIR="docs/function-design"
PLAN_DIR="docs/plans"
ALL_DOCS="docs"
ERRORS=0
WARNINGS=0
DB_TABLES_FILE=""
DB_COLUMNS_FILE=""

# --- ユーティリティ ---

red()    { printf '\033[0;31m%s\033[0m\n' "$1"; }
yellow() { printf '\033[0;33m%s\033[0m\n' "$1"; }
green()  { printf '\033[0;32m%s\033[0m\n' "$1"; }

error() {
    red "[ERROR] $1"
    ERRORS=$((ERRORS + 1))
}

warn() {
    yellow "[WARN]  $1"
    WARNINGS=$((WARNINGS + 1))
}

info() {
    printf '[INFO]  %s\n' "$1"
}

require_linux_ripgrep() {
    local rg_path
    rg_path="$(command -v rg || true)"

    if [ -z "$rg_path" ]; then
        error "ripgrep (rg) が見つかりません。WSL Ubuntu 側で 'sudo apt-get update && sudo apt-get install -y ripgrep' を実行してください。"
        exit 1
    fi

    case "$rg_path" in
        /mnt/*)
            error "Windows 側の rg を検出しました: $rg_path"
            error "WSL 内で実行するには Linux 版 ripgrep が必要です。Ubuntu 側で 'sudo apt-get update && sudo apt-get install -y ripgrep' を実行してください。"
            exit 1
            ;;
    esac

    if ! rg --version >/dev/null 2>&1; then
        error "rg は見つかりましたが実行できません: $rg_path"
        exit 1
    fi
}

header() {
    printf '\n=== %s ===\n' "$1"
}

require_linux_ripgrep

# --- チェック関数 ---

check_csv_tsv_terminology() {
    header "用語整合: CSV/TSV"
    # IO-04/BIZ-04/CMD-08 のPLU関連で「CSV」が残っていないか
    # 許容: 型名 PluCsvOutput（互換維持の注記あり）, CSV取込み関連（BIZ-03等）
    local plu_csv_hits
    plu_csv_hits=$(rg -n "PLU CSV|PLU csv|plu_csv(?!_import)" \
        "$DOCS_DIR/25-io-plu-formatter.md" \
        "$DOCS_DIR/33-biz-plu-export-service.md" \
        "$DOCS_DIR/41-cmd-pos.md" \
        2>/dev/null || true)

    # PluCsvOutput型名は許容（互換注記あり）
    plu_csv_hits=$(echo "$plu_csv_hits" | rg -v "PluCsvOutput|型名は互換維持" 2>/dev/null || true)

    if [ -n "$plu_csv_hits" ]; then
        error "PLU関連ドキュメントに「PLU CSV」が残存:"
        echo "$plu_csv_hits"
    else
        info "PLU関連の CSV/TSV 用語: OK"
    fi
}

check_error_type_consistency() {
    header "エラー型整合"

    # IO-04 が IoError を使っていないか（正: PluFormatError）
    local io_error_in_plu
    io_error_in_plu=$(rg -n "IoError" "$DOCS_DIR/25-io-plu-formatter.md" 2>/dev/null || true)
    if [ -n "$io_error_in_plu" ]; then
        error "25-io-plu-formatter.md に IoError が残存（正: PluFormatError）:"
        echo "$io_error_in_plu"
    fi

    # BIZ層に DbError が直接露出していないか
    local db_error_in_biz
    db_error_in_biz=$(rg -n "DbError" \
        "$DOCS_DIR/30-biz-product-service.md" \
        "$DOCS_DIR/31-biz-inventory-service.md" \
        "$DOCS_DIR/32-biz-csv-import-service.md" \
        "$DOCS_DIR/33-biz-plu-export-service.md" \
        2>/dev/null | rg -v "BizError::DatabaseError|DbError → BizError|DatabaseError\(DbError\)" || true)
    if [ -n "$db_error_in_biz" ]; then
        warn "BIZ層設計書に DbError が直接参照されています（BizError::DatabaseError への変換を確認）:"
        echo "$db_error_in_biz"
    fi

    info "エラー型チェック完了"
}

check_cache_responsibility() {
    header "preview_cache 責務整合"

    # BIZ層設計書に &mut HashMap / &mut cache が残っていないか
    local cache_in_biz
    cache_in_biz=$(rg -n "&mut HashMap|&mut cache|cache: &mut" \
        "$DOCS_DIR/32-biz-csv-import-service.md" \
        2>/dev/null || true)
    if [ -n "$cache_in_biz" ]; then
        error "BIZ層設計書に cache パラメータが残存（CMD層の責務）:"
        echo "$cache_in_biz"
    else
        info "preview_cache 責務: OK（BIZ層にcache参照なし）"
    fi
}

check_step_numbering() {
    header "手順番号の連番チェック"

    # 各ファイルの「処理ステップ」セクション内の番号欠番を検出
    local files
    files=$(find "$DOCS_DIR" -name "*.md" -type f | sort)

    for file in $files; do
        # 「処理ステップ」の後の数字付き行を抽出
        local steps
        steps=$(rg -n '^\d+[a-z]?\.' "$file" 2>/dev/null | head -50 || true)
        if [ -z "$steps" ]; then
            continue
        fi

        # 各ステップの主番号（サブステップ 4a等は除外）を抽出してギャップチェック
        # rg -n 出力は "行番号:内容" 形式。内容部分から先頭の数字を取得する
        local prev=0
        local has_gap=false
        while IFS= read -r line; do
            # "行番号:数字." → 内容部分（3番目以降のフィールド）から主番号を取る
            local content
            content=$(echo "$line" | cut -d: -f2-)
            local num
            num=$(echo "$content" | rg -o '^(\d+)\.' --replace '$1' 2>/dev/null || true)
            if [ -n "$num" ] && [[ "$num" =~ ^[0-9]+$ ]] && [ "$num" -gt 0 ]; then
                if [ "$prev" -gt 0 ] && [ "$num" -gt $((prev + 1)) ]; then
                    warn "$(basename "$file"): 手順番号に欠番 ($prev → $num)"
                    has_gap=true
                fi
                prev=$num
            fi
        done <<< "$steps"

        if [ "$has_gap" = false ] && [ "$prev" -gt 0 ]; then
            info "$(basename "$file"): 手順番号連番 OK"
        fi
    done
}

check_sql_patterns() {
    header "SQLパターンチェック"

    # LIMIT without ORDER BY
    local limit_no_order
    limit_no_order=$(rg -n "LIMIT \d+" "$DOCS_DIR"/*.md 2>/dev/null | while IFS= read -r line; do
        local file_line
        file_line=$(echo "$line" | cut -d: -f1-2)
        local file
        file=$(echo "$line" | cut -d: -f1)
        local lineno
        lineno=$(echo "$line" | cut -d: -f2)

        # 同じファイルの前5行に ORDER BY があるかチェック
        if ! [[ "$lineno" =~ ^[0-9]+$ ]]; then continue; fi
        local start=$((lineno - 5))
        if [ "$start" -lt 1 ]; then start=1; fi
        local context
        context=$(sed -n "${start},${lineno}p" "$file" 2>/dev/null || true)
        if ! echo "$context" | rg -q "ORDER BY" 2>/dev/null; then
            echo "$file_line"
        fi
    done || true)

    if [ -n "$limit_no_order" ]; then
        warn "LIMIT があるが近くに ORDER BY がないクエリ:"
        echo "$limit_no_order"
    else
        info "SQL LIMIT/ORDER BY: OK"
    fi

    # per_page 上限チェック
    local pagination_no_upper
    pagination_no_upper=$(rg -l "per_page" "$DOCS_DIR"/*.md 2>/dev/null | while IFS= read -r file; do
        if ! rg -q "per_page.*> *\d+|per_page.*<= *\d+|per_page.*上限" "$file" 2>/dev/null; then
            echo "$(basename "$file"): per_page に上限チェックが見つかりません"
        fi
    done || true)

    if [ -n "$pagination_no_upper" ]; then
        warn "ページング上限未定義:"
        echo "$pagination_no_upper"
    else
        info "ページング上限: OK"
    fi
}

check_layer_boundary() {
    header "レイヤー境界チェック"

    # IO層に BizError/CmdError が出現していないか
    local biz_in_io
    biz_in_io=$(rg -n "BizError|CmdError" \
        "$DOCS_DIR/20-io-product-repo.md" \
        "$DOCS_DIR/21-io-inventory-repo.md" \
        "$DOCS_DIR/23-io-z004-parser.md" \
        "$DOCS_DIR/24-io-csv-import-repo.md" \
        "$DOCS_DIR/25-io-plu-formatter.md" \
        2>/dev/null || true)
    if [ -n "$biz_in_io" ]; then
        error "IO層設計書に BizError/CmdError が出現（レイヤー違反）:"
        echo "$biz_in_io"
    else
        info "IO層のレイヤー境界: OK"
    fi

    # CMD層に DbError が出現していないか
    local db_in_cmd
    db_in_cmd=$(rg -n "DbError" \
        "$DOCS_DIR/40-cmd-product.md" \
        "$DOCS_DIR/41-cmd-pos.md" \
        2>/dev/null || true)
    if [ -n "$db_in_cmd" ]; then
        error "CMD層設計書に DbError が出現（レイヤー違反）:"
        echo "$db_in_cmd"
    else
        info "CMD層のレイヤー境界: OK"
    fi
}

check_tx_boundary() {
    header "TX境界方針チェック"

    # operation_log が TX内（&tx）で呼ばれていないか（方針: 全TX外）
    local log_in_tx
    log_in_tx=$(rg -n "insert_operation_log\(&tx" \
        "$DOCS_DIR/32-biz-csv-import-service.md" \
        "$DOCS_DIR/33-biz-plu-export-service.md" \
        2>/dev/null || true)
    if [ -n "$log_in_tx" ]; then
        error "operation_log が TX内で呼ばれています（方針: 全TX外）:"
        echo "$log_in_tx"
    else
        info "operation_log TX境界: OK"
    fi
}

check_constants_usage() {
    header "定数リテラル直書きチェック"

    # BIZ/CMD層で上限値がリテラル直書きされていないか（constants:: 参照を推奨）
    local literal_limits
    literal_limits=$(rg -n "20 \\\* 1024 \\\* 1024|20_?971_?520|> 5000[^0-9]|> 10.?000" \
        "$DOCS_DIR/32-biz-csv-import-service.md" \
        "$DOCS_DIR/33-biz-plu-export-service.md" \
        "$DOCS_DIR/41-cmd-pos.md" \
        2>/dev/null | rg -v "constants::" || true)
    if [ -n "$literal_limits" ]; then
        warn "上限値がリテラル直書き（constants:: 参照推奨）:"
        echo "$literal_limits"
    else
        info "定数参照: OK"
    fi
}

# ===========================================================================
# 新規チェック: C1〜M3（IEEE欠陥分類カバレッジ拡大）
# ===========================================================================

# --- C1: DBスキーマ参照検証 ---

build_db_schema_cache() {
    # DB_DESIGN.md + db-design/*.md からテーブル名・カラム名を抽出してキャッシュファイルに書き出す
    DB_TABLES_FILE=$(mktemp)
    DB_COLUMNS_FILE=$(mktemp)

    # DB_DESIGN.md 本体 + db-design/ サブファイルを連結した一時ファイルを作成
    local db_combined
    db_combined=$(mktemp)
    local db_design="$ALL_DOCS/DB_DESIGN.md"
    if [ ! -f "$db_design" ]; then
        warn "DB_DESIGN.md が見つかりません。DBスキーマチェックをスキップします"
        return 1
    fi
    cat "$db_design" > "$db_combined"
    for f in "$ALL_DOCS"/db-design/*.md; do
        [ -f "$f" ] && cat "$f" >> "$db_combined"
    done

    # テーブル名抽出: "## N. table_name（" or "## N-M. table1 + table2（"
    rg -o '##\s+\d+[-\d]*[a-z]?\.\s+([a-z_]+(?:\s*\+\s*[a-z_]+)*)' \
        --replace '$1' "$db_combined" 2>/dev/null | \
        tr '+' '\n' | sed 's/^ *//;s/ *$//' | sort -u > "$DB_TABLES_FILE"

    # "### table_name カラム定義" パターンからもテーブル名を抽出
    rg -o '###\s+([a-z_]+)\s+カラム定義' --replace '$1' "$db_combined" 2>/dev/null | \
        sort -u >> "$DB_TABLES_FILE"
    sort -u -o "$DB_TABLES_FILE" "$DB_TABLES_FILE"

    # カラム名抽出: テーブルヘッダの後にある "| column_name | TYPE |" パターン
    # "| カラム名 |" ヘッダ行と "|----|" 区切り行の直後がデータ行
    local current_table=""
    while IFS= read -r line; do
        # テーブルセクションヘッダを検出
        local table_match
        table_match=$(echo "$line" | rg -o '##\s+\d+[-\d]*[a-z]?\.\s+([a-z_]+)' --replace '$1' 2>/dev/null || true)
        if [ -n "$table_match" ]; then
            # "+" で複数テーブルが結合されている場合は最初のテーブルを使う
            current_table=$(echo "$table_match" | head -1 | sed 's/ *+.*//')
        fi

        # カラム定義ヘッダを検出（"### xxx カラム定義" で current_table を更新）
        local col_header
        col_header=$(echo "$line" | rg -o '###\s+([a-z_]+)\s+カラム定義' --replace '$1' 2>/dev/null || true)
        if [ -n "$col_header" ]; then
            current_table="$col_header"
        fi

        # カラム行を検出: "| column_name | TYPE | ..."
        if [ -n "$current_table" ]; then
            local col_name
            col_name=$(echo "$line" | rg -o '^\|\s+([a-z_]+)\s+\|\s+(TEXT|INTEGER|BOOLEAN|REAL)' --replace '$1' 2>/dev/null || true)
            if [ -n "$col_name" ]; then
                echo "${current_table}.${col_name}" >> "$DB_COLUMNS_FILE"
            fi
        fi
    done < "$db_combined"
    sort -u -o "$DB_COLUMNS_FILE" "$DB_COLUMNS_FILE"
    rm -f "$db_combined"

    return 0
}

check_db_schema_references() {
    header "C1: DBスキーマ参照検証"

    if [ ! -f "$DB_TABLES_FILE" ] || [ ! -s "$DB_TABLES_FILE" ]; then
        info "DBスキーマキャッシュなし。スキップ"
        return
    fi

    local table_count
    table_count=$(wc -l < "$DB_TABLES_FILE")
    local column_count
    column_count=$(wc -l < "$DB_COLUMNS_FILE")
    info "DB_DESIGN.md: ${table_count}テーブル, ${column_count}カラムを検出"

    # 対象ファイル: 引数があればそれを使用、なければ function-design/*.md
    local -a target_files
    if [ $# -gt 0 ]; then target_files=("$@"); else target_files=("$DOCS_DIR"/*.md); fi

    local found_any=false

    for file in "${target_files[@]}"; do
        [ -f "$file" ] || continue
        local basename
        basename=$(basename "$file")

        # table.column パターンを行番号付きで抽出
        local matches
        matches=$(rg -on '[a-z_]+\.[a-z_]+' "$file" 2>/dev/null || true)
        [ -z "$matches" ] && continue

        while IFS= read -r match_line; do
            local lineno ref
            lineno=$(echo "$match_line" | cut -d: -f1)
            ref=$(echo "$match_line" | cut -d: -f2-)

            # Rust モジュールパス等を除外
            local table_part="${ref%%.*}"
            case "$table_part" in
                fn|io|biz|cmd|mnt|ui|constants|req|inv|src|std|self|super|crate|tauri|serde|chrono|rusqlite|e|i|conn|tx|row|err|item|record|result) continue ;;
            esac

            local col_part="${ref#*.}"
            case "$col_part" in tsx|ts|rs|md|sh|json|css|html|js|toml|yml|yaml|py|lock) continue ;; esac

            # テーブル名がDBに存在するか
            if rg -q "^${table_part}$" "$DB_TABLES_FILE" 2>/dev/null; then
                found_any=true
                # カラム名がそのテーブルに存在するか
                if ! rg -q "^${table_part}\.${col_part}$" "$DB_COLUMNS_FILE" 2>/dev/null; then
                    if [ "$table_part" != "schema_versions" ]; then
                        warn "${basename}:${lineno}: ${table_part}.${col_part} — カラムがDB_DESIGN.mdに未定義"
                    fi
                fi
            fi
        done <<< "$matches"
    done

    if [ "$found_any" = false ]; then
        info "DBテーブル.カラム参照なし"
    else
        info "DBスキーマ参照チェック完了"
    fi
}

# --- C2: 関数シグネチャ呼び出し元−先整合 ---

check_signature_cross_reference() {
    header "C2: 関数シグネチャ呼び出し元−先整合"

    # 全 function-design/*.md から定義済み関数名を抽出（常に設計書から取得）
    local defined_fns_file
    defined_fns_file=$(mktemp)

    rg --no-filename -o '^fn\s+([a-z_][a-z0-9_]*)\s*\(' --replace '$1' "$DOCS_DIR"/*.md 2>/dev/null | \
        sort -u > "$defined_fns_file"

    local fn_count
    fn_count=$(wc -l < "$defined_fns_file")
    info "定義済み関数: ${fn_count}件"

    # BIZ/CMD層の処理ステップ内で呼ばれている "module::function(" パターンを抽出
    # 対象ファイル: 引数があればそれを使用、なければ BIZ/CMD 設計書
    local -a search_files
    if [ $# -gt 0 ]; then search_files=("$@"); else search_files=("$DOCS_DIR"/3*.md "$DOCS_DIR"/4*.md); fi

    local call_refs
    call_refs=$(rg -n '(\w+_repo|product_repo|inventory_repo|sales_repo|stocktake_repo|system_repo|csv_import_repo|z004_parser|plu_formatter|product_service|inventory_service|csv_import_service|plu_export_service)::([a-z_][a-z0-9_]*)\(' \
        "${search_files[@]}" 2>/dev/null || true)

    if [ -z "$call_refs" ]; then
        info "関数呼び出し参照なし"
        rm -f "$defined_fns_file"
        return
    fi

    while IFS= read -r call_line; do
        local called_fn
        called_fn=$(echo "$call_line" | rg -o '::([a-z_][a-z0-9_]*)\(' --replace '$1' 2>/dev/null | head -1 || true)
        if [ -z "$called_fn" ]; then continue; fi

        # 定義済み関数リストに存在するか
        if ! rg -q "^${called_fn}$" "$defined_fns_file" 2>/dev/null; then
            local file_line
            file_line=$(echo "$call_line" | cut -d: -f1-2)
            error "${file_line}: ::${called_fn}() — 関数設計書に定義が見つかりません"
        fi
    done <<< "$call_refs"

    rm -f "$defined_fns_file"
    info "関数呼び出し整合チェック完了"
}

# --- H1: REQ トレーサビリティ ---

check_req_traceability() {
    header "H1: REQトレーサビリティ（タスクID経由）"

    local arch_file="$ALL_DOCS/ARCHITECTURE.md"
    if [ ! -f "$arch_file" ]; then
        warn "ARCHITECTURE.md が見つかりません。スキップ"
        return
    fi

    # ARCHITECTURE.md から第4段階までのタスクID（BIZ-01〜04, IO-01〜04, CMD-01/07/08）を抽出
    # REQはfunction-designに直接書かれず、タスクID経由で辿る
    local expected_tasks="BIZ-01 BIZ-02 BIZ-03 BIZ-04 IO-01 IO-02 IO-04 CMD-01 CMD-07 CMD-08"

    # function-design/*.md のファイル名からカバーされているタスクを推定
    local missing_tasks=""
    for task in $expected_tasks; do
        local pattern
        case "$task" in
            BIZ-01) pattern="30-biz-product" ;;
            BIZ-02) pattern="31-biz-inventory" ;;
            BIZ-03) pattern="32-biz-csv" ;;
            BIZ-04) pattern="33-biz-plu" ;;
            IO-01)  pattern="2[0-4]-io-" ;;
            IO-02)  pattern="23-io-z004" ;;
            IO-04)  pattern="25-io-plu" ;;
            CMD-01) pattern="40-cmd-product" ;;
            CMD-07) pattern="41-cmd-pos" ;;
            CMD-08) pattern="41-cmd-pos" ;;
            *) pattern="" ;;
        esac

        if [ -n "$pattern" ]; then
            local found
            found=$(find "$DOCS_DIR" -name "*.md" | rg "$pattern" 2>/dev/null || true)
            if [ -z "$found" ]; then
                missing_tasks="${missing_tasks}${task} "
            fi
        fi
    done

    if [ -n "$missing_tasks" ]; then
        warn "第4段階までで関数設計書が見つからないタスク: ${missing_tasks}"
    else
        info "REQトレーサビリティ: OK（第4段階の全タスクに関数設計書あり）"
    fi
}

# --- H2: INV不変条件の参照漏れ ---

check_inv_coverage() {
    header "H2: INV不変条件の参照漏れ"

    local common_rules="$DOCS_DIR/10-common-rules.md"
    if [ ! -f "$common_rules" ]; then
        warn "10-common-rules.md が見つかりません。スキップ"
        return
    fi

    # INV-N 定義を抽出
    local inv_defs
    inv_defs=$(rg -o '\*\*INV-(\d+[a-z]?):' --replace 'INV-$1' "$common_rules" 2>/dev/null | sort -u)

    if [ -z "$inv_defs" ]; then
        info "INV定義なし"
        return
    fi

    local inv_count
    inv_count=$(echo "$inv_defs" | wc -l)
    info "INV定義: ${inv_count}件"

    # BIZ層の設計書で各INVが参照されているか
    local biz_files
    biz_files=$(find "$DOCS_DIR" -name "3*-biz-*.md" -type f 2>/dev/null | sort)

    while IFS= read -r inv_id; do
        local found=false
        for biz_file in $biz_files; do
            if rg -q "$inv_id" "$biz_file" 2>/dev/null; then
                found=true
                break
            fi
        done
        if [ "$found" = false ]; then
            warn "${inv_id} — BIZ層設計書のどこからも参照されていません"
        fi
    done <<< "$inv_defs"

    info "INV参照漏れチェック完了"
}

# --- H3: エラーバリアント網羅 ---

check_error_variant_coverage() {
    header "H3: エラーバリアント網羅"

    local common_rules="$DOCS_DIR/10-common-rules.md"
    if [ ! -f "$common_rules" ]; then
        info "10-common-rules.md なし。スキップ"
        return
    fi

    # DbError バリアントを抽出（enum DbError ブロック内の大文字始まりバリアントのみ）
    local db_variants
    db_variants=$(rg --no-filename -o '^\s+([A-Z][A-Za-z]+)\(' "$common_rules" 2>/dev/null | sed 's/^[[:space:]]*//' | sed 's/($//' | sort -u || true)

    if [ -z "$db_variants" ]; then
        info "DbErrorバリアント定義なし"
        return
    fi

    # 各バリアントが function-design (IO層) で使われているか
    while IFS= read -r variant; do
        [ -z "$variant" ] && continue
        local raw_counts
        raw_counts=$(rg -c "$variant" "$DOCS_DIR"/2*.md "$DOCS_DIR"/22-mnt-*.md 2>/dev/null || true)
        local usage_count=0
        if [ -n "$raw_counts" ]; then
            usage_count=$(echo "$raw_counts" | rg -o ':\d+$' 2>/dev/null | sed 's/://' | awk '{s+=$1} END {print s+0}' || echo 0)
        fi
        if [ "$usage_count" -eq 0 ]; then
            warn "DbError::${variant} — IO/MNT層設計書で未使用"
        fi
    done <<< "$db_variants"

    # BizError バリアントを抽出（BIZ層設計書から）
    local biz_error_defs
    biz_error_defs=$(rg --no-filename -o 'BizError::([A-Za-z]+)' --replace '$1' "$DOCS_DIR"/3*.md 2>/dev/null || true)

    if [ -n "$biz_error_defs" ]; then
        biz_error_defs=$(echo "$biz_error_defs" | sort -u)
        # CMD層でBizErrorバリアントが変換されているか
        while IFS= read -r biz_variant; do
            [ -z "$biz_variant" ] && continue
            local raw_cmd_counts
            raw_cmd_counts=$(rg -c "$biz_variant" "$DOCS_DIR"/4*.md 2>/dev/null || true)
            local cmd_usage=0
            if [ -n "$raw_cmd_counts" ]; then
                cmd_usage=$(echo "$raw_cmd_counts" | rg -o ':\d+$' 2>/dev/null | sed 's/://' | awk '{s+=$1} END {print s+0}' || echo 0)
            fi
            if [ "$cmd_usage" -eq 0 ]; then
                # ValidationFailed/DatabaseError/NotFound はCMD層で包括変換されるため除外
                case "$biz_variant" in
                    ValidationFailed|DatabaseError|NotFound) ;;
                    *) warn "BizError::${biz_variant} — CMD層設計書で未参照（変換漏れの可能性）" ;;
                esac
            fi
        done <<< "$biz_error_defs"
    fi

    info "エラーバリアント網羅チェック完了"
}

# --- M1: 曖昧表現検出 ---

check_ambiguous_language() {
    header "M1: 曖昧表現検出"

    # 対象ファイル: 引数があればそれを使用、なければ function-design/*.md
    local -a target_files
    if [ $# -gt 0 ]; then target_files=("$@"); else target_files=("$DOCS_DIR"/*.md "$ALL_DOCS"/architecture/*.md "$ALL_DOCS"/design-system/*.md); fi

    # コードブロック外の曖昧表現を検出
    # 「など」は曖昧表現だが出現頻度が高すぎるため、句読点直前のみ検出
    # 文中の「などの」「などを」は許容（具体的な列挙の後に続くパターンのため）
    local ambiguous_words="適切に|適宜|必要に応じて|など[。、）\)]|should |appropriate |as needed|TBD|TODO|FIXME|HACK"

    local hits
    hits=$(rg -n "$ambiguous_words" "${target_files[@]}" 2>/dev/null || true)

    if [ -z "$hits" ]; then
        info "曖昧表現: 検出なし"
        return
    fi

    # コードブロック内の行を除外（```で囲まれた範囲）
    # 簡易判定: 行頭が空白4つ以上またはバッククォート3つで始まる行は除外
    local filtered
    filtered=$(echo "$hits" | rg -v '^\S+:\d+:\s{4,}|^\S+:\d+:```' 2>/dev/null || true)

    # マークダウンテーブルのヘッダ行（| --- | 形式）は除外
    filtered=$(echo "$filtered" | rg -v '^\S+:\d+:\|.*---' 2>/dev/null || true)

    # "TODOファイル名" や "90-traceability.md — TODO" のような参照は除外
    filtered=$(echo "$filtered" | rg -v 'TODO:.*Phase|90-traceability' 2>/dev/null || true)

    if [ -n "$filtered" ]; then
        local hit_count
        hit_count=$(echo "$filtered" | wc -l)
        warn "曖昧表現が ${hit_count} 箇所で検出:"
        echo "$filtered" | head -20
        if [ "$hit_count" -gt 20 ]; then
            echo "  ... 他 $((hit_count - 20)) 件"
        fi
    else
        info "曖昧表現: 検出なし（コードブロック除外後）"
    fi
}

# --- M2: ドキュメントテンプレート準拠 ---

check_template_conformance() {
    header "M2: ドキュメントテンプレート準拠"

    # 除外ファイル: 共通ルール、マイグレーション（構造が異なる）、トレーサビリティ（自動生成）
    local exclude_pattern="10-common-rules|22-mnt-migration|90-traceability"

    local files
    files=$(find "$DOCS_DIR" -name "*.md" -type f | rg -v "$exclude_pattern" | sort)

    for file in $files; do
        local basename
        basename=$(basename "$file")

        # レイヤー判定: UI層(50-*), CMD層(40-*), BIZ/IO層(それ以外)
        case "$basename" in
            5*-ui-*)
                # UI層: 処理ステップとコンポーネント構造があればOK
                if ! rg -q "処理ステップ|コンポーネント|画面" "$file" 2>/dev/null; then
                    warn "${basename}: UI層の必須要素（処理ステップ or コンポーネント構造）が見つかりません"
                fi
                ;;
            4*-cmd-*)
                # CMD層: 薄いラッパーのため、シグネチャ + 処理ステップがあればOK
                for section in "シグネチャ" "処理ステップ"; do
                    if ! rg -q "$section" "$file" 2>/dev/null; then
                        warn "${basename}: CMD層必須セクション「${section}」が見つかりません"
                    fi
                done
                ;;
            *)
                # BIZ/IO層: フルテンプレート
                for section in "関数要求" "シグネチャ" "処理ステップ"; do
                    if ! rg -q "$section" "$file" 2>/dev/null; then
                        warn "${basename}: 必須セクション「${section}」が見つかりません"
                    fi
                done
                # エラーハンドリングは "エラー" を含むヘッダまたはテキストで判定
                if ! rg -q "エラー" "$file" 2>/dev/null; then
                    warn "${basename}: エラーハンドリングに関する記述が見つかりません"
                fi
                ;;
        esac
    done

    info "テンプレート準拠チェック完了"
}

# --- M3: TODO/未確定マーカー残存 ---

check_stale_markers() {
    header "M3: TODO/未確定マーカー残存"

    local markers="TODO|TBD|FIXME|HACK|未確定|要確認|要検討"

    # 対象ファイル: 引数があればそれを使用、なければ docs/ 配下全体
    local -a target_files
    if [ $# -gt 0 ]; then
        target_files=("$@")
    else
        target_files=("$ALL_DOCS"/*.md "$ALL_DOCS"/function-design/*.md "$ALL_DOCS"/quality/*.md "$ALL_DOCS"/architecture/*.md "$ALL_DOCS"/design-system/*.md)
    fi

    local hits
    hits=$(rg -n "$markers" "${target_files[@]}" 2>/dev/null || true)

    if [ -z "$hits" ]; then
        info "未確定マーカー: 検出なし"
        return
    fi

    # 取り消し線 ~~text~~ 内を除外（非貪欲マッチで個別ブロック単位）
    local filtered
    filtered=$(echo "$hits" | rg -v '~~[^~]+~~' 2>/dev/null || true)

    # 設計判断ログの中の「未確定事項」セクションは許容（ARCHITECTURE.md セクション4）
    filtered=$(echo "$filtered" | rg -v '## 4\. 未確定事項' 2>/dev/null || true)

    # FUNCTION_DESIGN.md の "TODO: Phase" は進捗管理のため許容
    filtered=$(echo "$filtered" | rg -v 'TODO:.*Phase|TODO:.*段階' 2>/dev/null || true)

    # TOOLING_SKILL_COMMANDS.md のチェック項目説明は許容（TBD/TODO を検出対象として言及しているだけ）
    filtered=$(echo "$filtered" | rg -v 'TOOLING_SKILL_COMMANDS\.md' 2>/dev/null || true)

    # DOC_STYLE_GUIDE.md の禁止ワード説明は許容（ルールとして言及しているだけ）
    filtered=$(echo "$filtered" | rg -v 'DOC_STYLE_GUIDE\.md' 2>/dev/null || true)

    if [ -n "$filtered" ]; then
        local hit_count
        hit_count=$(echo "$filtered" | wc -l)
        warn "未確定マーカーが ${hit_count} 箇所で残存:"
        echo "$filtered" | head -20
        if [ "$hit_count" -gt 20 ]; then
            echo "  ... 他 $((hit_count - 20)) 件"
        fi
    else
        info "未確定マーカー: 検出なし（許容パターン除外後）"
    fi
}

# ===========================================================================
# Plan Packet チェック: PK1, PK2, PK3
# ===========================================================================

is_active_dated_plan() {
    local base
    base=$(basename "$1")
    case "$base" in
        [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]-*.md) return 0 ;;
        *) return 1 ;;
    esac
}

# PK4 の新規チェックのみ対象外にするための判定（PK1-PK3 の既存挙動は変えない）。
# 明示パスで docs/archive/ 配下の packet を渡された場合、PK4 は skip する
# （compatibility: archived packet に '## Workflow State' 等の新形式を強制しない）。
is_archived_plan_path() {
    case "$1" in
        docs/archive/*|*/docs/archive/*) return 0 ;;
        *) return 1 ;;
    esac
}

# D-145 (5): active な packet は scripts/pr-gate.py の plan_origin・active_packets の 1 つの定義だけで決める
# （docs/plans/ 直下の dated packet のうち起点の tree に無いもの。起点は WORKFLOW_BASE_SHA か origin/main と
# HEAD の merge-base）。形式（D-145 (4)）は packet を最初に足した commit の親に scripts/plan-approve.sh が在るか。
PLAN_SCOPE_LOADED=0
PLAN_SCOPE_ERROR=""
PLAN_SCOPE_REPORTED=0
PLAN_ORIGIN=""
declare -a ACTIVE_PLANS=()
declare -A PLAN_FORM=()
declare -A PLAN_FORM_ERROR=()

load_plan_scope() {
    [ "$PLAN_SCOPE_LOADED" = "1" ] && return 0
    PLAN_SCOPE_LOADED=1
    local scope kind path value
    scope=$(PYTHONDONTWRITEBYTECODE=1 python3 - "${WORKFLOW_BASE_SHA:-}" <<'PY' 2>/dev/null
import importlib.util, os, sys
spec = importlib.util.spec_from_file_location('pr_gate', 'scripts/pr-gate.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
names = [f'docs/plans/{n}' for n in sorted(os.listdir('docs/plans'))] if os.path.isdir('docs/plans') else []
try:
    origin = m.plan_origin(sys.argv[1] or None)
    print('origin\x1f' + (origin or '') + '\x1f')
    for path in m.active_packets(origin, names):
        try:
            print(f'active\x1f{path}\x1f{m.packet_form(path, origin)}')
        except m.GateError as error:
            print(f'formerror\x1f{path}\x1f{error}')
except m.GateError as error:
    print(f'error\x1f\x1f{error}')
PY
) || scope=$'error\x1f\x1fpython3 と scripts/pr-gate.py が必要'
    while IFS=$'\x1f' read -r kind path value; do
        case "$kind" in
            origin) PLAN_ORIGIN="$path" ;;
            active) ACTIVE_PLANS+=("$path"); PLAN_FORM["$path"]="$value" ;;
            formerror) ACTIVE_PLANS+=("$path"); PLAN_FORM_ERROR["$path"]="$value" ;;
            error) PLAN_SCOPE_ERROR="$value" ;;
        esac
    done <<< "$scope"
}

# 起点を決められなければ ERROR（全 packet の検査に落とさない。fail-closed）。報告は 1 回。
# docs/plans/ に dated packet が 1 本も無ければ active になりうる packet が無いので止めない。
report_plan_scope_error() {
    load_plan_scope
    [ -n "$(find "$PLAN_DIR" -maxdepth 1 -name '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]-*.md' -type f 2>/dev/null)" ] || return 0
    if [ -n "$PLAN_SCOPE_ERROR" ] && [ "$PLAN_SCOPE_REPORTED" = "0" ]; then
        PLAN_SCOPE_REPORTED=1
        error "PK: active な packet を決められない: $PLAN_SCOPE_ERROR"
    fi
}

iter_active_dated_plans() {
    local file
    for file in "${ACTIVE_PLANS[@]}"; do
        printf '%s\n' "$file"
    done
}

is_active_plan() {
    load_plan_scope
    local file
    for file in "${ACTIVE_PLANS[@]}"; do
        [ "$file" = "$1" ] && return 0
    done
    return 1
}

# 形式: active なら git の履歴（PLAN_FORM）、名指しの merge 済み・archive の packet は Phase の行の有無。
plan_form_of() {
    load_plan_scope
    local ws
    if [ -n "${PLAN_FORM[$1]:-}" ]; then
        printf '%s\n' "${PLAN_FORM[$1]}"
        return 0
    fi
    ws=$(extract_markdown_section "$1" "Workflow State")
    if [ -n "$ws" ] && ! printf '%s\n' "$ws" | grep -qE '^- Phase:'; then
        echo new
    else
        echo old
    fi
}

iter_plan_packet_targets() {
    local file
    if [ "$#" -gt 0 ]; then
        for file in "$@"; do
            [ -f "$file" ] || continue
            if is_active_dated_plan "$file"; then
                printf '%s\n' "$file"
            fi
        done
    else
        iter_active_dated_plans
    fi
}

get_valid_plan_risk() {
    grep -xE 'Risk:[[:space:]]*R[0-4][[:space:]]*' "$1" | grep -oE 'R[0-4]' | head -1 || true
}

get_any_plan_risk_line() {
    grep -E '^Risk:' "$1" | head -1 || true
}

extract_markdown_section() {
    local file="$1"
    local section="$2"
    awk -v section="$section" '
        $0 ~ "^#{2,}[[:space:]]+" section "([[:space:]].*)?$" { in_section=1; next }
        in_section && $0 ~ "^#{2,}[[:space:]]+" { exit }
        in_section { print }
    ' "$file"
}

# level-2 section の配下にある level-3+ 小見出しも含めて抽出する。
# Goal Invariant 専用ではなく、`###` 小見出しを含みうる level-2 section 全般に使う。
extract_markdown_h2_section() {
    local file="$1"
    local section="$2"
    awk -v section="$section" '
        $0 ~ "^##[[:space:]]+" section "([[:space:]].*)?$" { in_section=1; next }
        in_section && $0 ~ "^##[[:space:]]+" { exit }
        in_section { print }
    ' "$file"
}

# packet の節の有無は template と同じ `##` の見出しだけで判定する（`###` の小見出しは節でない、D-102 D5）。
has_h2_section() {
    grep -qE "^##[[:space:]]+${2}([[:space:]].*)?$" "$1"
}

extract_prose() {
    local file="$1"
    awk '
        /^```/ { in_code = !in_code; next }
        in_code { next }
        /^[[:space:]]*\|/ { next }
        { print }
    ' "$file" | sed 's/`[^`]*`//g'
}

extract_prose_keep_inline() {
    local file="$1"
    awk '
        /^```/ { in_code = !in_code; next }
        in_code { next }
        /^[[:space:]]*\|/ { next }
        { print }
    ' "$file"
}

strip_fenced_code_and_html_comments() {
    awk '
        function marker_run_length(text, marker, count) {
            marker = substr(text, 1, 1)
            if (marker != "`" && marker != "~") {
                return 0
            }
            count = 1
            while (substr(text, count + 1, 1) == marker) {
                count++
            }
            return count
        }
        {
            trimmed = $0
            sub(/^[[:space:]]*/, "", trimmed)
            marker = substr(trimmed, 1, 1)
            marker_length = marker_run_length(trimmed)

            if (in_fence) {
                remainder = substr(trimmed, marker_length + 1)
                if (marker == fence_marker &&
                    marker_length >= fence_length &&
                    remainder ~ /^[[:space:]]*$/) {
                    in_fence = 0
                }
                next
            }
            if (marker_length >= 3) {
                in_fence = 1
                fence_marker = marker
                fence_length = marker_length
                next
            }

            line = $0
            while (1) {
                if (in_comment) {
                    comment_end = index(line, "-->")
                    if (comment_end == 0) {
                        next
                    }
                    line = substr(line, comment_end + 3)
                    in_comment = 0
                }

                comment_start = index(line, "<!--")
                if (comment_start == 0) {
                    print line
                    next
                }

                before = substr(line, 1, comment_start - 1)
                remainder = substr(line, comment_start + 4)
                comment_end = index(remainder, "-->")
                if (comment_end == 0) {
                    print before
                    in_comment = 1
                    next
                }

                line = before substr(remainder, comment_end + 3)
            }
        }
    '
}

# Workflow State セクション本文（extract_markdown_section の出力）から
# "- <field>: <value>" 形式の1行目の値を取り出す。値は enum/SHA/pending を
# 想定し英数字・アンダースコア・ハイフンのみを対象にする（末尾の注記括弧等は無視）。
extract_workflow_field() {
    local section="$1" field="$2"
    local raw
    raw=$(printf '%s\n' "$section" | grep -E "^- ${field}:" | head -1 || true)
    [ -z "$raw" ] && return 0
    if [ "${3:-}" = "full" ]; then
        printf '%s\n' "$raw" | sed -E "s/^- ${field}:[[:space:]]*//; s/[[:space:]]+$//"
    else
        printf '%s\n' "$raw" | sed -E "s/^- ${field}:[[:space:]]*//" | grep -oE '^[A-Za-z0-9_-]+' || true
    fi
}

is_in_word_list() {
    local needle="$1" haystack="$2" word
    for word in $haystack; do
        [ "$word" = "$needle" ] && return 0
    done
    return 1
}

trace_matrix_data_rows() {
    local file="$1"
    local section="${2:-Trace Matrix}"
    extract_markdown_h2_section "$file" "$section" \
        | awk '
            /^[[:space:]]*\|/ {
                row=$0
                compact=row
                gsub(/[[:space:]\|:-]/, "", compact)
                if (compact == "") {
                    next
                }
                if (!seen_header) {
                    seen_header=1
                    next
                }
                print row
            }
        '
}

acceptance_bullets() {
    local file="$1"
    extract_markdown_section "$file" "Acceptance Criteria" \
        | grep -nE '^[[:space:]]*[-*][[:space:]]+.+$' || true
}

has_acceptance_observable_token() {
    local line="$1"
    printf '%s\n' "$line" \
        | grep -Eq '`[^`]+`|test_[A-Za-z0-9_]+|cargo|clippy|npm|vitest|bash|doc-consistency|pre-push|bindings[.]ts|[.](csv|tsv|json|md|sh|rs|ts|tsx)([^A-Za-z0-9_]|$)|(^|[^A-Za-z0-9_])(WARN|ERROR|exit|stdout|stderr|SQLite|PLU|CSV|TSV|CP932|REQ|SP|UI|CMD|BIZ|IO|MNT)([^A-Za-z0-9_]|$)'
}

test_token_exists() {
    local token="$1"
    rg -q "(fn|def|it|test)[[:space:]]+${token}|[\"']${token}[\"']" \
        tests src src-tauri 2>/dev/null
}

has_test_design_matrix_reference() {
    local file="$1"
    if has_h2_section "$file" "Test Design Matrix"; then
        return 0
    fi
    extract_markdown_section "$file" "Test Plan" \
        | grep -Eq '\]\((([^)]*/)?test-matrices/[^)]*[.]md)\)|`[^`]*test-matrices/[^`]*[.]md`'
}

check_plan_packet_sections() {
    header "PK1: Plan Packet presence"
    report_plan_scope_error

    local before=$ERRORS
    local file valid_risk any_risk level section
    local base_sections=("Risk" "Goal" "Scope" "Non-scope" "Acceptance Criteria" "Test Plan" "Review Focus" "Owner Effort Budget")
    local r3_sections=("Data Safety" "Contract Probe")

    while IFS= read -r file; do
        [ -n "$file" ] || continue
        valid_risk=$(get_valid_plan_risk "$file")
        any_risk=$(get_any_plan_risk_line "$file")

        if [ -z "$valid_risk" ]; then
            if [ -n "$any_risk" ]; then
                error "PK1: $file の Risk 行が不正です。有効形式は 'Risk: R0' 〜 'Risk: R4'"
            else
                error "PK1: $file は dated active plan ですが有効な 'Risk: Rn' 行がありません"
            fi
            continue
        fi

        level="${valid_risk#R}"
        [ "$level" -le 1 ] && continue

        # D-145 (10) H14: 新形式は最小形の節、旧形式は今の必須の節のまま（非遡及）。
        if [ "$(plan_form_of "$file")" = "new" ]; then
            local new_sections=("Risk" "Goal" "Scope" "Acceptance Criteria")
            [ "$level" -ge 3 ] && new_sections+=("Design Readiness" "Contract Ledger" "Test Plan")
            [ "$level" -ge 4 ] && new_sections+=("Recovery" "Data Safety")
            for section in "${new_sections[@]}"; do
                if ! has_h2_section "$file" "$section"; then
                    error "PK1: $file (R${level}, 新形式) は必須セクション '## ${section}' を欠いています"
                fi
            done
            if has_h2_section "$file" "Contract Ledger" && [ -z "$(trace_matrix_data_rows "$file" "Contract Ledger")" ]; then
                error "PK1: $file (R${level}) の Contract Ledger にデータ行がありません"
            fi
            if has_h2_section "$file" "L3" &&
                [ -z "$(extract_markdown_h2_section "$file" "L3" | strip_fenced_code_and_html_comments | grep -vE '^[[:space:]]*$' || true)" ]; then
                error "PK1: $file の '## L3' が空です（節があること = manual。基準を書くか節を消す）"
            fi
            continue
        fi

        local required_sections=("${base_sections[@]}")
        [ "$level" -ge 3 ] && required_sections+=("${r3_sections[@]}")

        # D-039 導入節（Owner Effort Budget / Contract Probe）は導入前に作成された
        # archive packet へ遡及要求しない（明示パス指定時の互換性、Double Audit pass1 P1）
        if is_archived_plan_path "$file"; then
            local filtered=()
            for section in "${required_sections[@]}"; do
                case "$section" in
                    "Owner Effort Budget"|"Contract Probe") ;;
                    *) filtered+=("$section") ;;
                esac
            done
            required_sections=("${filtered[@]}")
        fi

        for section in "${required_sections[@]}"; do
            if ! has_h2_section "$file" "$section"; then
                error "PK1: $file (R${level}) は必須セクション '## ${section}' を欠いています"
            fi
        done

        if [ "$level" -ge 3 ] && ! has_test_design_matrix_reference "$file"; then
            error "PK1: $file (R${level}) は Test Design Matrix へのリンクまたは '## Test Design Matrix' セクションを欠いています"
        fi

        # 新 template は Contract Ledger の 1 表、旧 template は Spec Contract と Trace Matrix の組を受ける。
        if [ "$level" -ge 3 ]; then
            if has_h2_section "$file" "Contract Ledger"; then
                if [ -z "$(trace_matrix_data_rows "$file" "Contract Ledger")" ]; then
                    error "PK1: $file (R${level}) の Contract Ledger にデータ行がありません"
                fi
            elif ! has_h2_section "$file" "Spec Contract" ||
                ! has_h2_section "$file" "Trace Matrix"; then
                error "PK1: $file (R${level}) は Contract Ledger（旧 template は Spec Contract と Trace Matrix）を欠いています"
            fi
        fi
    done < <(iter_plan_packet_targets "$@")

    if [ "$ERRORS" -eq "$before" ]; then
        info "PK1: Plan Packet presence OK"
    fi
}

check_plan_packet_substance() {
    header "PK2: Plan Packet substance"

    local before=$ERRORS
    local file valid_risk level

    while IFS= read -r file; do
        [ -n "$file" ] || continue
        valid_risk=$(get_valid_plan_risk "$file")
        [ -z "$valid_risk" ] && continue

        level="${valid_risk#R}"
        [ "$level" -le 1 ] && continue

        local placeholder_hits
        placeholder_hits=$(extract_prose "$file" | grep -nE '<[A-Za-z][A-Za-z0-9 ._-]*>' || true)
        if [ -n "$placeholder_hits" ]; then
            while IFS= read -r line; do
                error "PK2: $file (R${level}) に未編集 placeholder が残っています -> $line"
            done <<< "$placeholder_hits"
        fi

        local empty_bullet_hits
        empty_bullet_hits=$(extract_prose_keep_inline "$file" | grep -nE '^[[:space:]]*[-*][[:space:]]*$' || true)
        if [ -n "$empty_bullet_hits" ]; then
            while IFS= read -r line; do
                error "PK2: $file (R${level}) に空 bullet が残っています -> $line"
            done <<< "$empty_bullet_hits"
        fi
    done < <(iter_plan_packet_targets "$@")

    if [ "$ERRORS" -eq "$before" ]; then
        info "PK2: Plan Packet substance OK"
    fi
}

check_plan_packet_heuristic_warnings() {
    header "PK3: Plan Packet heuristic warnings"

    local before=$WARNINGS
    local file valid_risk level

    while IFS= read -r file; do
        [ -n "$file" ] || continue
        valid_risk=$(get_valid_plan_risk "$file")
        [ -z "$valid_risk" ] && continue

        level="${valid_risk#R}"
        [ "$level" -le 2 ] && continue

        local trace_section="Trace Matrix"
        has_h2_section "$file" "Contract Ledger" && trace_section="Contract Ledger"
        local trace_rows
        trace_rows=$(trace_matrix_data_rows "$file" "$trace_section")
        if [ -z "$trace_rows" ]; then
            warn "PK3: $file (R${level}) の ${trace_section} table に data row がありません"
        else
            local placeholder_hits
            placeholder_hits=$(printf '%s\n' "$trace_rows" | sed 's/`[^`]*`//g' | grep -nE '<[A-Za-z][A-Za-z0-9 ._-]*>' || true)
            if [ -n "$placeholder_hits" ]; then
                while IFS= read -r line; do
                    warn "PK3: $file (R${level}) の ${trace_section} table に placeholder が残っています -> $line"
                done <<< "$placeholder_hits"
            fi

            local test_tokens token
            test_tokens=$(printf '%s\n' "$trace_rows" | grep -oE 'test_[A-Za-z0-9_]+' | sort -u || true)
            for token in $test_tokens; do
                if ! test_token_exists "$token"; then
                    warn "PK3: $file (R${level}) の ${trace_section} test token \`$token\` が tests/src/src-tauri に見つかりません"
                fi
            done
        fi

        local bullet_hits
        bullet_hits=$(acceptance_bullets "$file")
        if [ -n "$bullet_hits" ]; then
            while IFS= read -r line; do
                if ! has_acceptance_observable_token "$line"; then
                    warn "PK3: $file (R${level}) の Acceptance Criteria bullet に観測 token が見つかりません -> $line"
                fi
            done <<< "$bullet_hits"
        fi
    done < <(iter_plan_packet_targets "$@")

    if [ "$WARNINGS" -eq "$before" ]; then
        info "PK3: Plan Packet heuristic warnings OK"
    fi
}

# PK6: Contract Probe / Review Response 内の数値主張に実測 evidence があるかの
# 構文的 heuristic（D-062）。WARN-only、exit code に影響しない。PK3 の
# has_acceptance_observable_token と同じ「任意 backtick span で足りる」寛容基準を踏襲する。
check_plan_packet_numeric_evidence_warnings() {
    header "PK6: 数値主張の実測 evidence 欠落"

    local before=$WARNINGS
    local file valid_risk level

    while IFS= read -r file; do
        [ -n "$file" ] || continue
        valid_risk=$(get_valid_plan_risk "$file")
        [ -z "$valid_risk" ] && continue
        level="${valid_risk#R}"

        local sec
        for sec in "Contract Probe" "Review Response"; do
            local section_text
            section_text=$(extract_markdown_h2_section "$file" "$sec")
            [ -z "$section_text" ] && continue

            local hits
            hits=$(printf '%s\n' "$section_text" | grep -nE '[0-9]+(\.[0-9]+)?[[:space:]]*(秒|分|回|件|%)' || true)
            [ -z "$hits" ] && continue

            while IFS= read -r line; do
                local content="${line#*:}"
                if printf '%s' "$content" | grep -qE '`[^`]+`|未実測'; then
                    continue
                fi
                warn "PK6: $file (R${level}) の Contract Probe/Review Response に実測 evidence のない数値主張があります -> $line"
            done <<< "$hits"
        done
    done < <(iter_plan_packet_targets "$@")

    if [ "$WARNINGS" -eq "$before" ]; then
        info "PK6: 数値主張の実測 evidence 欠落 OK"
    fi
}

# --- PK4: Workflow State machine 整合（DEV_WORKFLOW.md Workflow State 節の機械強制） ---

check_plan_packet_workflow_state() {
    header "PK4: Workflow State machine 整合"

    local before=$ERRORS
    local file valid_risk level

    while IFS= read -r file; do
        [ -n "$file" ] || continue
        is_archived_plan_path "$file" && continue
        valid_risk=$(get_valid_plan_risk "$file")
        [ -z "$valid_risk" ] && continue

        level="${valid_risk#R}"
        [ "$level" -le 1 ] && continue

        local ws_section
        ws_section=$(extract_markdown_section "$file" "Workflow State")
        if [ -z "$ws_section" ]; then
            error "PK4: $file (R${level}) は必須セクション '## Workflow State' を欠いています"
            continue
        fi

        # Evidence Mode 行は任意。書くなら github だけ。Execution Mode 行は任意で値を評価しない。
        if printf '%s\n' "$ws_section" | grep -qE '^- Evidence Mode:' &&
            [ "$(extract_workflow_field "$ws_section" "Evidence Mode" full)" != "github" ]; then
            error "PK4: $file の Evidence Mode は廃止。書くなら github"
        fi
        # D-145 (3)(4): 旧形式は Phase の行（値は評価しない）と書いた Minimum・Human Gate、新形式は 3 行。
        # 担当 4 行・Branch は在っても無くても受ける。形式と Phase の行の食い違いは helper の parse_packet が拒む。
        local form form_arg=""
        form=$(plan_form_of "$file")
        [ -n "${PLAN_FORM[$file]:-}" ] && form_arg="$form"
        if [ -n "${PLAN_FORM_ERROR[$file]:-}" ]; then
            error "PK4: $file の形式を判定できない: ${PLAN_FORM_ERROR[$file]}"
            continue
        fi
        local -a required_fields=("Risk" "Plan Commit" "Amendments")
        [ "$form" = "old" ] && required_fields=("Phase" "Risk" "Plan Commit" "Amendments" "Final Review Minimum" "Human Gate")
        local field
        for field in "${required_fields[@]}"; do
            if ! printf '%s\n' "$ws_section" | grep -qE "^- ${field}:[[:space:]]*[^[:space:]]"; then
                error "PK4: $file (R${level}) の Workflow State に '- ${field}:' 行がありません"
            fi
        done
        for field in "Reviewed Content HEAD" "Final Exact-HEAD Evidence" "Hosted CI Requirement"; do
            if printf '%s\n' "$ws_section" | grep -qE "^- ${field}:"; then
                error "PK4: $file に legacy field '$field' は保存できません"
            fi
        done

        # Workflow State は helper の parse_packet そのものに判定させる（D-107 (1)。key の重複は D-102 D3 の文のまま）。
        # helper の path は cwd（repo root）相対。python3 か import が失敗すれば ERROR（fail-closed）。
        local helper_check
        helper_check=$(PYTHONDONTWRITEBYTECODE=1 python3 -c '
import importlib.util, sys
s = importlib.util.spec_from_file_location("pr_gate", "scripts/pr-gate.py")
m = importlib.util.module_from_spec(s)
s.loader.exec_module(m)
try:
    m.parse_packet(open(sys.argv[1], encoding="utf-8").read(), sys.argv[2] or None)
    print("ok")
except m.GateError as e:
    print("rejected:", e)
' "$file" "$form_arg" 2>/dev/null) || helper_check=""
        case "$helper_check" in
            ok) ;;
            "rejected: duplicate packet fields")
                error "PK4: $file の Workflow State に重複する field があります（helper の workflow_fields が拒否）" ;;
            "rejected: "*)
                error "PK4: $file の Workflow State を helper（parse_packet）が拒否: ${helper_check#rejected: }" ;;
            *)
                error "PK4: $file の Workflow State の重複の検査に python3 と scripts/pr-gate.py が必要です" ;;
        esac

        if [ "$form" = "old" ]; then
            local minimum human_gate
            minimum=$(extract_workflow_field "$ws_section" "Final Review Minimum" full)
            human_gate=$(extract_workflow_field "$ws_section" "Human Gate" full)
            if ! is_in_word_list "$minimum" "1 2" || { [ "$level" -eq 4 ] && [ "$minimum" != "2" ]; }; then
                error "PK4: $file の Final Review Minimum は1/2（R4は2）が必須です"
            fi
            if [[ ! "$human_gate" =~ ^(ready|merge|manual|r4)(,(ready|merge|manual|r4))*$ ]] ||
                [[ ",$human_gate," != *,ready,* || ",$human_gate," != *,merge,* ]] ||
                { [ "$level" -eq 4 ] && [[ ",$human_gate," != *,r4,* ]]; }; then
                error "PK4: $file の Human Gate は ready,merge と必要な manual/r4 を明示してください"
            fi
        fi

        # 値全体を末尾の空白・タブを削らずに照合する（helper の fullmatch と同じ集合、D-098）。
        # 前置で落とすのは ASCII の空白とタブだけ（helper の [ \t]*。[[:space:]] は UTF-8 の locale で U+3000 に当たる）。
        local plan_commit_raw
        plan_commit_raw=$(printf '%s\n' "$ws_section" | grep -E '^- Plan Commit:' | head -1 | sed -E $'s/^- Plan Commit:[ \t]*//' || true)
        if [ -n "$plan_commit_raw" ] && [[ ! "$plan_commit_raw" =~ ^(pending|[0-9a-f]{40})$ ]]; then
            error "PK4: $file (R${level}) の Plan Commit は pending か 40 桁の小文字 hex の SHA（末尾の注記・空白・タブなし）。commit 済みで未 push なら、新しい commit ではなくその commit を直す（PK5 は commit 済みの初回値を固定する） -> '${plan_commit_raw}'"
        fi

        local ws_risk_value
        ws_risk_value=$(extract_workflow_field "$ws_section" "Risk")
        if [ -n "$ws_risk_value" ] && [ "$ws_risk_value" != "$valid_risk" ]; then
            error "PK4: $file (R${level}) の Workflow State '- Risk: ${ws_risk_value}' が '## Risk' セクションの 'Risk: ${valid_risk}' と不一致です"
        fi
    done < <(iter_plan_packet_targets "$@")

    if [ "$ERRORS" -eq "$before" ]; then
        info "PK4: Workflow State machine 整合 OK"
    fi
}

# --- D-046: active Plan Packet の Goal Invariant 構造（WARN 導入） ---
check_active_plan_goal_invariant() {
    header "D-046: Goal Invariant 構造"

    local before=$WARNINGS
    local file goal_section

    while IFS= read -r file; do
        [ -n "$file" ] || continue
        goal_section=$(extract_markdown_h2_section "$file" "Goal")

        if ! printf '%s\n' "$goal_section" | grep -qE '^Goal Invariant([[:space:]]|[:（(])'; then
            warn "D-046: $file の Goal Invariant 構造に 'Goal Invariant' marker がありません"
        fi
        if ! printf '%s\n' "$goal_section" | grep -qE '^###[[:space:]]+最小完了条件([[:space:]].*)?$|^Goal Invariant.*最小完了条件'; then
            warn "D-046: $file の Goal Invariant 構造に '最小完了条件' 小見出しがありません"
        fi
        if ! printf '%s\n' "$goal_section" | grep -qE '^###[[:space:]]+失敗定義([[:space:]].*)?$|^失敗定義[：:]'; then
            warn "D-046: $file の Goal Invariant 構造に '失敗定義' 小見出しがありません"
        fi
        if ! printf '%s\n' "$goal_section" | grep -qE '^###[[:space:]]+非目的([[:space:]].*)?$|^非目的[：:]'; then
            warn "D-046: $file の Goal Invariant 構造に '非目的' 小見出しがありません"
        fi
    done < <(iter_active_dated_plans)

    if [ "$WARNINGS" -eq "$before" ]; then
        info "D-046: active Plan Packet Goal Invariant 構造 OK"
    fi
}

# ===========================================================================
# プランチェック固有: P1, P2
# ===========================================================================

# --- P1: 設計書との矛盾検出 ---

check_plan_design_consistency() {
    header "P1: 設計書との矛盾検出"

    local -a target_files=("$@")

    if [ ! -f "$DB_TABLES_FILE" ] || [ ! -s "$DB_TABLES_FILE" ]; then
        info "DBスキーマキャッシュなし。テーブル操作チェックをスキップ"
        return
    fi

    # プラン内の SQL 操作キーワード + テーブル名パターンを検出
    local sql_ops
    sql_ops=$(rg -on '(INSERT INTO|UPDATE|DELETE FROM|SELECT.*FROM)\s+([a-z_]+)' \
        "${target_files[@]}" 2>/dev/null || true)

    if [ -z "$sql_ops" ]; then
        info "SQL操作参照なし"
        return
    fi

    while IFS= read -r op_line; do
        local table_name
        table_name=$(echo "$op_line" | rg -o '(INSERT INTO|UPDATE|DELETE FROM|SELECT.*FROM)\s+([a-z_]+)' 2>/dev/null | \
            rg -o '[a-z_]+$' 2>/dev/null || true)
        [ -z "$table_name" ] && continue

        if ! rg -q "^${table_name}$" "$DB_TABLES_FILE" 2>/dev/null; then
            local file_line
            file_line=$(echo "$op_line" | cut -d: -f1-2)
            warn "${file_line}: SQL操作の対象テーブル '${table_name}' がDB_DESIGN.mdに未定義"
        fi
    done <<< "$sql_ops"

    info "設計書整合チェック完了"
}

# --- P2: 先決事項の未解決検出 ---

check_plan_unresolved_decisions() {
    header "P2: 先決事項の未解決検出"

    local -a target_files=("$@")

    # 「先決事項」「決定事項」セクション内の未解決マーカーを検出
    local decision_markers="未決|未定|要確認|TBD|要検討|TODO"

    for file in "${target_files[@]}"; do
        [ -f "$file" ] || continue
        local basename
        basename=$(basename "$file")

        # テーブル内の「結論」列が空または未決マーカーを含むかチェック
        local unresolved
        unresolved=$(rg -n "$decision_markers" "$file" 2>/dev/null || true)

        if [ -z "$unresolved" ]; then
            info "${basename}: 未解決の先決事項なし"
            continue
        fi

        # 「先決事項」「決定事項」セクション近辺のマーカーのみ ERROR にする
        # それ以外は M3（stale markers）で WARN 済みなのでここでは報告しない
        local in_decision_section=false
        while IFS= read -r line; do
            local lineno content
            lineno=$(echo "$line" | cut -d: -f1)
            content=$(echo "$line" | cut -d: -f2-)

            # マークダウンテーブル内の未決マーカーを検出
            if echo "$content" | rg -q '^\|.*\|' 2>/dev/null; then
                if echo "$content" | rg -q "$decision_markers" 2>/dev/null; then
                    error "${basename}:${lineno}: 先決事項/決定事項に未解決マーカー: $(echo "$content" | sed 's/^[[:space:]]*//' | head -c 100)"
                fi
            fi
        done <<< "$unresolved"
    done

    info "先決事項チェック完了"
}

# ===========================================================================
# 参照整合性チェック: R0, R1, R3
# ===========================================================================

# --- R0: 親文書名の直接参照検出（stale parent ref） ---

check_stale_parent_doc_references() {
    header "R0: 親文書の直接参照検出"

    # D-137: 開発用の bin は src-tauri/dev-tools/src にある。無い dir は飛ばす
    local -a src_dirs=()
    local d
    for d in "src-tauri/src" "src-tauri/dev-tools/src"; do
        if [ -d "$d" ]; then
            src_dirs+=("$d")
        fi
    done
    if [ ${#src_dirs[@]} -eq 0 ]; then
        info "src-tauri/src/ と src-tauri/dev-tools/src/ が見つかりません。スキップ"
        return
    fi

    local r0_start_errors=$ERRORS

    # 分割済み親文書: コード内で直接参照すべきでない
    local -a parent_docs=("FUNCTION_DESIGN.md" "DB_DESIGN.md")

    # 除外ファイル（包括的参照として許容。generate_traceability は索引パース対象として FUNCTION_DESIGN.md を参照する）
    local exclude_pattern="schema_v1\.rs|generate_traceability\.rs"

    for parent in "${parent_docs[@]}"; do
        local hits
        hits=$(rg -n "$parent" "${src_dirs[@]}" --type rust 2>/dev/null | \
            rg -v "$exclude_pattern" 2>/dev/null || true)

        if [ -n "$hits" ]; then
            while IFS= read -r line; do
                local file_line
                file_line=$(echo "$line" | cut -d: -f1-2)
                error "${file_line}: '${parent}' を直接参照 → サブファイルを参照してください"
            done <<< "$hits"
        fi
    done

    local r0_errors=$((ERRORS - ${r0_start_errors:-0}))
    if [ "$r0_errors" -eq 0 ]; then
        info "親文書の直接参照: 検出なし"
    fi
}

# --- R1: コード内ドキュメント参照パス検証 ---

check_code_doc_references() {
    header "R1: コード内ドキュメント参照パス検証"

    # D-137: 開発用の bin は src-tauri/dev-tools/src にある。無い dir は飛ばす
    local -a src_dirs=()
    local d
    for d in "src-tauri/src" "src-tauri/dev-tools/src"; do
        if [ -d "$d" ]; then
            src_dirs+=("$d")
        fi
    done
    if [ ${#src_dirs[@]} -eq 0 ]; then
        info "src-tauri/src/ と src-tauri/dev-tools/src/ が見つかりません。スキップ"
        return
    fi

    local checked=0
    local broken=0

    # パターン1: フルパス参照（function-design/xx-yy.md 等）
    local refs
    refs=$(rg -on '(function-design|db-design|architecture)/[a-zA-Z0-9_-]+\.md' \
        "${src_dirs[@]}" --type rust 2>/dev/null || true)

    if [ -n "$refs" ]; then
        while IFS= read -r line; do
            local file_line ref_path
            file_line=$(echo "$line" | cut -d: -f1-2)
            ref_path=$(echo "$line" | rg -o '(function-design|db-design|architecture)/[a-zA-Z0-9_-]+\.md' 2>/dev/null || true)
            [ -z "$ref_path" ] && continue

            checked=$((checked + 1))
            if [ ! -f "$ALL_DOCS/$ref_path" ]; then
                error "${file_line}: 参照先 'docs/${ref_path}' が存在しません"
                broken=$((broken + 1))
            fi
        done <<< "$refs"
    fi

    # パターン2: bare filename参照（20-io-product-repo.md §2.3 等）
    # function-design/ のサブファイル名パターン: NN-layer-module.md
    local bare_refs
    bare_refs=$(rg -on '\b[0-9]{2}-[a-z][a-z0-9_-]+\.md' \
        "${src_dirs[@]}" --type rust 2>/dev/null || true)

    if [ -n "$bare_refs" ]; then
        # フルパス参照と重複する行を除外（既にパターン1でチェック済み）
        bare_refs=$(echo "$bare_refs" | rg -v '(function-design|db-design|architecture)/' 2>/dev/null || true)
    fi

    if [ -n "$bare_refs" ]; then
        while IFS= read -r line; do
            local file_line bare_name
            file_line=$(echo "$line" | cut -d: -f1-2)
            bare_name=$(echo "$line" | rg -o '[0-9]{2}-[a-z][a-z0-9_-]+\.md' 2>/dev/null || true)
            [ -z "$bare_name" ] && continue

            checked=$((checked + 1))
            if [ ! -f "$ALL_DOCS/function-design/$bare_name" ]; then
                error "${file_line}: 参照先 'docs/function-design/${bare_name}' が存在しません"
                broken=$((broken + 1))
            fi
        done <<< "$bare_refs"
    fi

    if [ "$checked" -eq 0 ]; then
        info "コード内のドキュメントパス参照: 検出なし"
    elif [ "$broken" -eq 0 ]; then
        info "コード内参照パス: ${checked}件すべて実在確認OK"
    fi
}

# --- R3: Markdownリンク先ファイル存在検証 ---

is_merged_plan_file() {
    case "$1" in
        docs/plans/*) ;;
        *) return 1 ;;
    esac
    load_plan_scope
    [ -z "$PLAN_SCOPE_ERROR" ] || return 1
    [ -z "$PLAN_ORIGIN" ] && return 0
    git cat-file -e "$PLAN_ORIGIN:$1" 2>/dev/null
}

# --- FL: 生きた docs に新しく足した `path:行` の引用（D-145 (10) H13、D-125 の目的） ---
# 起点からの差分の '+' 行だけを見る。D-125 (2)(3)・DOC_STYLE_GUIDE §1 の適用除外は対象外。
check_new_file_line_references() {
    header "FL: 新しい path:行 の引用（D-125）"
    load_plan_scope
    if [ -n "$PLAN_SCOPE_ERROR" ]; then
        warn "FL: 起点が取れないので新しい path:行 の検査をしない（$PLAN_SCOPE_ERROR）。pre-push と hosted は起点を渡す"
        return 0
    fi
    if [ -z "$PLAN_ORIGIN" ]; then
        info "FL: lane の commit が無い（起点 = HEAD）"
        return 0
    fi
    local before=$ERRORS hits
    hits=$(git diff -U0 --no-renames --no-color "$PLAN_ORIGIN" -- docs AGENTS.md CLAUDE.md 2>/dev/null | awk '
        /^\+\+\+ / { file = substr($0, 7); next }
        /^\+/ {
            if (file ~ /^docs\/(archive|plans|research)\// || file == "docs/decision-log.md") next
            n = split(file, parts, "/"); if (parts[n] ~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]/) next
            line = substr($0, 2)
            if (file == "docs/backlog.md" && line ~ /(^|[^0-9A-Za-z])[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]+([^0-9A-Za-z]|$)/) next
            if (match(line, /[A-Za-z0-9_.\/-]+[.](md|rs|ts|tsx|py|sh):[0-9]+/)) print file ": " substr(line, RSTART, RLENGTH)
        }') || hits=""
    if [ -n "$hits" ]; then
        while IFS= read -r line; do
            error "FL: 生きた docs に path:行 の引用を新しく足した（節名・symbol・決定 ID で引く。D-125）-> $line"
        done <<< "$hits"
        if [ -z "${WORKFLOW_BASE_SHA:-}" ]; then
            info "FL: 起点は origin/main と HEAD の merge-base。origin/main が古ければ git fetch origin してから回し直す（他の lane の file が出るときの原因）"
        fi
    fi
    if [ "$ERRORS" -eq "$before" ]; then
        info "FL: 新しい path:行 の引用なし"
    fi
}

check_markdown_link_targets() {
    header "R3: Markdownリンク先ファイル存在検証"

    local checked=0
    local broken=0

    # docs/ 配下の全 .md ファイルを走査
    while IFS= read -r md_file; do
        local dir_of_file
        dir_of_file=$(dirname "$md_file")

        # コードブロック（```...```）内を空行に置換（行番号を保持）
        local stripped
        stripped=$(mktemp)
        awk '/^```/{skip=!skip; print ""; next} skip{print ""} !skip{print}' "$md_file" > "$stripped"

        # [text](path) パターンを抽出（HTTP/HTTPS URL は除外）
        local links
        links=$(rg -on '\]\(([^)]+)\)' --replace '$1' "$stripped" 2>/dev/null || true)
        rm -f "$stripped"
        [ -z "$links" ] && continue

        while IFS= read -r link_line; do
            local lineno link_path
            lineno=$(echo "$link_line" | cut -d: -f1)
            link_path=$(echo "$link_line" | cut -d: -f2-)

            # HTTP URL はスキップ
            echo "$link_path" | rg -q '^https?://' 2>/dev/null && continue
            # アンカーリンク (#...) はスキップ
            echo "$link_path" | rg -q '^#' 2>/dev/null && continue
            # 空リンクはスキップ
            [ -z "$link_path" ] && continue

            # アンカー部分を除去（path#anchor → path）
            link_path=$(echo "$link_path" | sed 's/#.*//')
            [ -z "$link_path" ] && continue

            checked=$((checked + 1))
            local resolved_path="${dir_of_file}/${link_path}"
            if [ ! -f "$resolved_path" ]; then
                # D-145 (5) AR6: merge 済みの packet・Matrix（起点の tree にある docs/plans/**）の link 切れは WARN。
                if is_merged_plan_file "$md_file"; then
                    warn "$(basename "$md_file"):${lineno}: merge 済みの packet のリンク先 '${link_path}' が存在しません（直すのは任意）"
                else
                    error "$(basename "$md_file"):${lineno}: リンク先 '${link_path}' が存在しません"
                    broken=$((broken + 1))
                fi
            fi
        done <<< "$links"
    done < <(find "$ALL_DOCS" -name "*.md" -type f 2>/dev/null)

    if [ "$broken" -eq 0 ]; then
        info "Markdownリンク: ${checked}件すべて実在確認OK"
    fi
}

# ===========================================================================
# DS: design-system docs 検査（PR-C C4）
# 対象は docs/design-system/ + docs/quality/review-checklist.md の直指定。
# docs/archive/ は走査しない（旧参照の誤検出回避）。
# ===========================================================================

# --- DS1: design-system docs 内 backtick src/ path 実在 ---

check_ds_source_paths() {
    header "DS1: design-system docs 内 src/ path 実在"

    local ds_dir="$ALL_DOCS/design-system"
    if [ ! -d "$ds_dir" ]; then
        info "docs/design-system/ が見つかりません。スキップ"
        return
    fi

    local checked=0
    local missing=0
    local paths
    paths=$(rg -o --no-filename '`(src/[^`]+)`' --replace '$1' "$ds_dir"/*.md 2>/dev/null | sort -u || true)

    while IFS= read -r p; do
        [ -z "$p" ] && continue
        # glob 文字を含む値は path でなく規約表現（例: DSR-08 本文の `src/features/**`）のため skip
        case "$p" in
            *'*'* | *'?'*) continue ;;
        esac
        checked=$((checked + 1))
        if [ ! -f "$p" ]; then
            error "DS1: design-system docs が参照する '$p' が存在しません"
            missing=$((missing + 1))
        fi
    done <<< "$paths"

    if [ "$missing" -eq 0 ]; then
        info "DS1: src/ path 参照 ${checked} 件すべて実在"
    fi
}

# --- DS2: DSR 参照整合（孤立 = WARN / 壊れ参照 = ERROR） ---

check_ds_dsr_references() {
    header "DS2: DSR 参照整合（孤立 / 壊れ参照）"

    local rules_doc="$ALL_DOCS/design-system/01-decision-rules.md"
    local catalog="$ALL_DOCS/design-system/02-component-catalog.md"
    local checklist="$ALL_DOCS/quality/review-checklist.md"
    if [ ! -f "$rules_doc" ] || [ ! -f "$catalog" ]; then
        info "design-system docs が見つかりません。スキップ"
        return
    fi

    local defined referenced
    defined=$(rg -o '^## (DSR-[0-9]+)' --replace '$1' "$rules_doc" 2>/dev/null | sort -u || true)
    referenced=$(rg -o --no-filename '\bDSR-[0-9]+\b' "$catalog" "$checklist" 2>/dev/null | sort -u || true)

    local isolated=0
    while IFS= read -r id; do
        [ -z "$id" ] && continue
        if ! echo "$referenced" | rg -q "^${id}\$" 2>/dev/null; then
            warn "DS2: ${id} は catalog / review-checklist から参照されていません（孤立）"
            isolated=$((isolated + 1))
        fi
    done <<< "$defined"

    local broken=0
    while IFS= read -r id; do
        [ -z "$id" ] && continue
        if ! echo "$defined" | rg -q "^${id}\$" 2>/dev/null; then
            error "DS2: ${id} が参照されていますが 01-decision-rules.md に定義がありません"
            broken=$((broken + 1))
        fi
    done <<< "$referenced"

    if [ "$isolated" -eq 0 ] && [ "$broken" -eq 0 ]; then
        local def_count
        def_count=$(echo "$defined" | rg -c '.' 2>/dev/null || echo 0)
        info "DS2: DSR 参照整合 OK（定義 ${def_count} 件、孤立 0 / 壊れ参照 0）"
    fi
}

# --- DS3: token HEX 整合（00-foundations 表 ↔ globals.css :root、双方向） ---
# foundations は stone 表（裸 HEX カラム）と semantic 表（`(#hex)` 埋込）の二書式
# のため「`--name` を含む行の最初の #hex」で統一抽出する。突合先は :root のみ
# （@theme inline の --color-* 別名は対象外）。foundations にあって :root に
# 無い token も ERROR（旧 --danger 型 drift の捕捉）。

check_ds_token_hex_sync() {
    header "DS3: token HEX 整合（00-foundations ↔ globals.css :root）"

    local foundations="$ALL_DOCS/design-system/00-foundations.md"
    local css="src/styles/globals.css"
    if [ ! -f "$foundations" ] || [ ! -f "$css" ]; then
        info "対象ファイルが見つかりません。スキップ"
        return
    fi

    local root_block
    root_block=$(awk '/^:root[[:space:]]*\{/,/^\}/' "$css")

    local checked=0
    local mismatch=0
    local rows
    rows=$(rg -- '`--[a-z0-9-]+`' "$foundations" 2>/dev/null || true)

    while IFS= read -r line; do
        [ -z "$line" ] && continue
        local name hex_doc hex_css
        name=$(echo "$line" | rg -o '`--[a-z0-9-]+`' 2>/dev/null | head -1 | tr -d '\`')
        [ -z "$name" ] && continue
        hex_doc=$(echo "$line" | rg -o '#[0-9a-fA-F]{6}' 2>/dev/null | head -1 || true)
        [ -z "$hex_doc" ] && continue
        checked=$((checked + 1))
        hex_css=$(echo "$root_block" | rg -o -- "${name}:[[:space:]]*#[0-9a-fA-F]{6}" 2>/dev/null | rg -o '#[0-9a-fA-F]{6}' 2>/dev/null | head -1 || true)
        if [ -z "$hex_css" ]; then
            error "DS3: foundations の '${name}' が globals.css :root に存在しません"
            mismatch=$((mismatch + 1))
        elif [ "$(echo "$hex_doc" | tr '[:upper:]' '[:lower:]')" != "$(echo "$hex_css" | tr '[:upper:]' '[:lower:]')" ]; then
            error "DS3: '${name}' の HEX 不一致（foundations=${hex_doc} / globals.css=${hex_css}）"
            mismatch=$((mismatch + 1))
        fi
    done <<< "$rows"

    if [ "$mismatch" -eq 0 ]; then
        info "DS3: token HEX 整合 OK（${checked} 件突合）"
    fi
}

# --- DS4: review-checklist カテゴリ 9 の DSR 対応（WARN） ---

check_ds_checklist_dsr_links() {
    header "DS4: review-checklist カテゴリ 9 の DSR 対応"

    local checklist="$ALL_DOCS/quality/review-checklist.md"
    if [ ! -f "$checklist" ]; then
        info "review-checklist.md が見つかりません。スキップ"
        return
    fi

    local section
    section=$(awk '/^### 9\./{flag=1; next} /^#/{flag=0} flag' "$checklist")
    if [ -z "$section" ]; then
        warn "DS4: review-checklist にカテゴリ 9（### 9.）セクションが見つかりません"
        return
    fi

    local total=0
    local missing=0
    while IFS= read -r line; do
        case "$line" in
            "- [ ]"*) ;;
            *) continue ;;
        esac
        total=$((total + 1))
        if ! echo "$line" | rg -q 'DSR-[0-9]+' 2>/dev/null; then
            warn "DS4: カテゴリ 9 項目に DSR 参照がありません -> $(echo "$line" | cut -c1-60)"
            missing=$((missing + 1))
        fi
    done <<< "$section"

    if [ "$missing" -eq 0 ]; then
        info "DS4: カテゴリ 9 全 ${total} 項目に DSR 参照あり"
    fi
}

# SPEC-HYG3-CMD-2: design mode の一箇所から既存 docs gate 全経路へ届ける。
check_command_registry_drift() {
    header "Command registry drift"
    if ! bash scripts/check-command-drift.sh; then
        error "Command registry drift を検出しました"
    fi
}

# ===========================================================================
# メイン
# ===========================================================================

# クリーンアップ
cleanup() {
    rm -f "$DB_TABLES_FILE" "$DB_COLUMNS_FILE" 2>/dev/null || true
}
trap cleanup EXIT

# --- 引数解析 ---
TARGET_MODE="design"  # デフォルト: 設計書チェック
TARGET_PATH=""

while [ $# -gt 0 ]; do
    case "$1" in
        --target)
            shift
            TARGET_MODE="${1:-design}"
            shift
            # オプションの対象パスが続く場合
            if [ $# -gt 0 ] && [[ ! "$1" == --* ]]; then
                TARGET_PATH="$1"
                shift
            fi
            ;;
        --fix)
            echo "自動修正モードは未実装です"
            exit 0
            ;;
        *)
            echo "不明なオプション: $1"
            echo "使い方: $0 [--target plan [file_or_dir]]"
            exit 2
            ;;
    esac
done

# --- モード別実行 ---

# D-145 (5): active な packet の起点と形式を main の shell で 1 回だけ決める（各 check は process substitution の
# subshell で回るので、ここで読み込まないと状態も ERROR の数も main に届かない）。
load_plan_scope

if [ "$TARGET_MODE" = "plan" ]; then
    # === プランチェックモード ===

    # 対象ファイルの決定
    declare -a PLAN_FILES=()
    if [ -n "$TARGET_PATH" ]; then
        if [ -f "$TARGET_PATH" ]; then
            PLAN_FILES=("$TARGET_PATH")
        elif [ -d "$TARGET_PATH" ]; then
            while IFS= read -r f; do PLAN_FILES+=("$f"); done < <(find "$TARGET_PATH" -maxdepth 1 -name "*.md" -type f | sort)
        else
            red "ファイルまたはディレクトリが見つかりません: $TARGET_PATH"
            exit 2
        fi
    else
        # デフォルト: active な packet（D-145 (5) の定義）だけ。merge 済みの packet・archive・test-matrices は対象外。
        load_plan_scope
        if [ -n "$PLAN_SCOPE_ERROR" ]; then
            red "active な packet を決められない: $PLAN_SCOPE_ERROR"
            exit 1
        fi
        for f in "${ACTIVE_PLANS[@]}"; do PLAN_FILES+=("$f"); done
    fi

    if [ ${#PLAN_FILES[@]} -eq 0 ]; then
        red "チェック対象のプランファイルが見つかりません"
        exit 2
    fi

    echo "========================================="
    echo " プラン整合性チェック"
    echo " 対象: ${PLAN_FILES[*]}"
    echo "========================================="

    # DBスキーマキャッシュを構築
    build_db_schema_cache || true

    # 再利用チェック（4項目）
    check_db_schema_references "${PLAN_FILES[@]}"
    check_signature_cross_reference "${PLAN_FILES[@]}"
    check_ambiguous_language "${PLAN_FILES[@]}"
    check_stale_markers "${PLAN_FILES[@]}"

    # プラン固有チェック（2項目）
    check_plan_design_consistency "${PLAN_FILES[@]}"
    check_plan_unresolved_decisions "${PLAN_FILES[@]}"

    # Plan Packet チェック（4項目、docs/plans/ 直下の dated active plan）
    check_plan_packet_sections "${PLAN_FILES[@]}"
    check_plan_packet_substance "${PLAN_FILES[@]}"
    check_plan_packet_heuristic_warnings "${PLAN_FILES[@]}"
    check_plan_packet_numeric_evidence_warnings "${PLAN_FILES[@]}"
    check_plan_packet_workflow_state "${PLAN_FILES[@]}"
    check_active_plan_goal_invariant

    # 参照整合性チェック（3項目、プランモードでも実行）
    check_stale_parent_doc_references
    check_code_doc_references
    check_markdown_link_targets
    check_new_file_line_references

else
    # === 設計書チェックモード（従来動作） ===

    echo "========================================="
    echo " 設計書横断整合性チェック"
    echo " 対象: $DOCS_DIR/*.md"
    echo "========================================="

    # --- 既存チェック（8項目） ---
    check_csv_tsv_terminology
    check_error_type_consistency
    check_cache_responsibility
    check_step_numbering
    check_sql_patterns
    check_layer_boundary
    check_tx_boundary
    check_constants_usage

    # --- 新規チェック（8項目） ---
    build_db_schema_cache || true

    check_db_schema_references
    check_signature_cross_reference
    check_req_traceability
    check_inv_coverage
    check_error_variant_coverage
    check_ambiguous_language
    check_template_conformance
    check_stale_markers

    # --- 参照整合性チェック（3項目） ---
    check_stale_parent_doc_references
    check_code_doc_references
    check_markdown_link_targets

    # --- DS: design-system docs 検査（4項目、PR-C C4） ---
    check_ds_source_paths
    check_ds_dsr_references
    check_ds_token_hex_sync
    check_ds_checklist_dsr_links
    check_new_file_line_references

    # --- Plan Packet チェック（4項目、active plan は通常チェックでも enforce） ---
    check_plan_packet_sections
    check_plan_packet_substance
    check_plan_packet_heuristic_warnings
    check_plan_packet_numeric_evidence_warnings
    check_plan_packet_workflow_state
    check_active_plan_goal_invariant

    check_command_registry_drift
fi

echo ""
echo "========================================="
if [ "$ERRORS" -gt 0 ]; then
    red "結果: ERROR $ERRORS 件, WARN $WARNINGS 件"
    exit 1
elif [ "$WARNINGS" -gt 0 ]; then
    yellow "結果: WARN $WARNINGS 件（ERROR なし）"
    exit 0
else
    green "結果: 全チェック通過"
    exit 0
fi
