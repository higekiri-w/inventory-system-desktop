#!/usr/bin/env bash
# scripts/doc-consistency-check.sh の PK3 (test_token_exists)、PK4
# (check_plan_packet_workflow_state)、PK1（旧形式の必須の節と新形式の最小形、D-145 (10) H14）、
# active な packet の定義（D-145 (5)）、新しい path:行 の検査（H13）、R0・R1 の走査の範囲 (D-137) を
# synthetic fixture で検証する。
# fixture はすべて本 test 自身が tmpdir に生成し、tracked fixture file は増やさない。
# D-145 (4)(5): run_check は fixture を lane の 1 commit にして起点（PKT_BASE）の後に置く。
# 起点 BASE_OLD は scripts/plan-approve.sh の無い main（旧形式）、BASE_NEW は在る main（新形式）。
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_contains() {
    local file="$1"
    local pattern="$2"
    grep -Fq -- "$pattern" "$file" || fail "$file does not contain: $pattern"
}

assert_not_contains() {
    local file="$1"
    local pattern="$2"
    if grep -Fq -- "$pattern" "$file"; then
        fail "$file unexpectedly contains: $pattern"
    fi
}

assert_helper_calls_have_no_negative_globs() {
    local file="$1"
    local line arg cluster option pattern
    local helper_calls=0
    local i
    local -a call_args=()

    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            CALL)
                call_args=()
                ;;
            ARG$'\t'*)
                call_args+=("${line#*$'\t'}")
                ;;
            END)
                helper_calls=$((helper_calls + 1))
                i=0
                while [ "$i" -lt "${#call_args[@]}" ]; do
                    arg="${call_args[$i]}"
                    pattern=""
                    case "$arg" in
                        --glob)
                            i=$((i + 1))
                            [ "$i" -lt "${#call_args[@]}" ] ||
                                fail "captured helper call has --glob without a pattern"
                            pattern="${call_args[$i]}"
                            ;;
                        --glob=*)
                            pattern="${arg#--glob=}"
                            ;;
                        -[^-]*)
                            cluster="${arg#-}"
                            while [ -n "$cluster" ]; do
                                option="${cluster%"${cluster#?}"}"
                                cluster="${cluster#?}"
                                case "$option" in
                                    g)
                                        pattern="$cluster"
                                        if [ -z "$pattern" ]; then
                                            i=$((i + 1))
                                            [ "$i" -lt "${#call_args[@]}" ] ||
                                                fail "captured helper call has -g without a pattern"
                                            pattern="${call_args[$i]}"
                                        fi
                                        cluster=""
                                        ;;
                                    e|f|E|m|j|d|t|T|A|B|C|M|r)
                                        if [ -z "$cluster" ]; then
                                            i=$((i + 1))
                                            [ "$i" -lt "${#call_args[@]}" ] ||
                                                fail "captured helper call has -$option without a value"
                                        fi
                                        cluster=""
                                        ;;
                                esac
                            done
                            ;;
                    esac
                    if [[ "$pattern" == !* ]]; then
                        fail "captured helper call contains a negative glob: $arg $pattern"
                    fi
                    i=$((i + 1))
                done
                ;;
        esac
    done < "$file"

    [ "$helper_calls" -gt 0 ] ||
        fail "rg shim did not capture a tests/src/src-tauri helper call"
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
repo="$tmp/repo"
mkdir -p "$repo/docs/plans" "$repo/docs/function-design"
git init -q "$repo"
git -C "$repo" config user.name test
git -C "$repo" config user.email test@example.invalid
BASE_OLD="$(git -C "$repo" commit-tree "$(git -C "$repo" mktree < /dev/null)" -m "main before plan-approve.sh")"
approve_blob="$(printf '#!/bin/sh\n' | git -C "$repo" hash-object -w --stdin)"
approve_tree="$(printf '100644 blob %s\tplan-approve.sh\n' "$approve_blob" | git -C "$repo" mktree)"
BASE_NEW="$(printf '040000 tree %s\tscripts\n' "$approve_tree" | git -C "$repo" mktree |
    xargs git -C "$repo" commit-tree -p "$BASE_OLD" -m "main with plan-approve.sh")"
cp "$SOURCE_ROOT/scripts/doc-consistency-check.sh" "$repo/doc-consistency-check.sh"
# PK4 の key の重複の検査は cwd 相対の scripts/pr-gate.py（helper の workflow_fields）を呼ぶ（D-102 D3）
mkdir -p "$repo/scripts" && cp "$SOURCE_ROOT/scripts/pr-gate.py" "$repo/scripts/"

write_rg_call_log() {
    local file="$1"
    shift
    {
        echo "CALL"
        for arg in "$@" tests src src-tauri; do
            printf 'ARG\t%s\n' "$arg"
        done
        echo "END"
    } > "$file"
}

assert_glob_guard_accepts() {
    local label="$1"
    shift
    local file="$tmp/rg-argv-accept.log"
    write_rg_call_log "$file" "$@"
    assert_helper_calls_have_no_negative_globs "$file" ||
        fail "glob argv guard rejected legal short-option grammar: $label"
}

assert_glob_guard_rejects() {
    local label="$1"
    shift
    local file="$tmp/rg-argv-reject.log"
    write_rg_call_log "$file" "$@"
    if (assert_helper_calls_have_no_negative_globs "$file" >/dev/null 2>&1); then
        fail "glob argv guard accepted a negative glob: $label"
    fi
}

# A value-taking short option consumes the remainder of its cluster (or the
# following argv), so a later "g" is data rather than another option.
assert_glob_guard_accepts "-qeg!x" "-qeg!x"
assert_glob_guard_accepts "-qe g!x" "-qe" "g!x"
assert_glob_guard_accepts "-qAg!1" "-qAg!1"
assert_glob_guard_accepts "-qA g!1" "-qA" "g!1"
assert_glob_guard_accepts "-gq!x" "-gq!x"
assert_glob_guard_rejects "-uvqg !x" "-uvqg" "!x"
assert_glob_guard_rejects "-uvqg!x" "-uvqg!x"

real_rg="$(command -v rg)"
rg_shim_dir="$tmp/bin"
rg_argv_log="$tmp/rg-argv.log"
mkdir -p "$rg_shim_dir"
cat > "$rg_shim_dir/rg" <<'SHIM'
#!/usr/bin/env bash
set -euo pipefail

args=("$@")
argc=${#args[@]}
if [ -n "${RG_ARGV_LOG:-}" ] && [ "$argc" -ge 3 ] \
    && [ "${args[$((argc - 3))]}" = "tests" ] \
    && [ "${args[$((argc - 2))]}" = "src" ] \
    && [ "${args[$((argc - 1))]}" = "src-tauri" ]; then
    {
        echo "CALL"
        for arg in "$@"; do
            printf 'ARG\t%s\n' "$arg"
        done
        echo "END"
    } >> "$RG_ARGV_LOG"
fi

exec "$REAL_RG" "$@"
SHIM
chmod +x "$rg_shim_dir/rg"

# check_signature_cross_reference (既存 C2、本 test の対象外) は
# docs/function-design/*.md に '^fn ' 行が1件も無いと rg が no-match で
# 非0終了し、後続処理を伴わずに `set -e` で落ちる（下流の | sort -u に
# pipefail 未フォールバックのため）。実リポジトリでは常に一致がある前提の
# 既存挙動であり本 test の対象ではないため、fixture 側でダミー関数定義を
# 1件用意して既存挙動を壊さず回避する。
printf 'fn dummy_fixture_fn(x: i32) -> i32\n' > "$repo/docs/function-design/00-dummy.md"

out="$tmp/out.log"

setup_repo_dirs() {
    rm -rf "$repo/docs/plans" "$repo/docs/archive"
    mkdir -p "$repo/docs/plans"
}

# D-097: '## 次の行動' は packet ごとの link でなく docs/plans/ を指す pointer 行を持つ。
# 引数（packet の basename）は受けるが使わない（呼出し側の正例 fixture の意味を保つ）。
write_plans_md_linking() {
    {
        echo "# Plans"
        echo ""
        echo "## 次の行動"
        echo ""
        echo "1. active な lane の Plan Packet は docs/plans/ の dated packet が正本"
    } > "$repo/docs/Plans.md"
}

# SC5 の後継: '## 次の行動' 配下の '### Wave Registry' 小見出しの下に pointer を置く。
write_plans_md_linking_under_wave_registry() {
    {
        echo "# Plans"
        echo ""
        echo "## 次の行動"
        echo ""
        echo "### Wave Registry"
        echo ""
        echo "1. active な lane の Plan Packet は docs/plans/ の dated packet が正本"
    } > "$repo/docs/Plans.md"
}

write_plans_md_no_link() {
    {
        echo "# Plans"
        echo ""
        echo "## 次の行動"
        echo ""
        echo "1. fixture entry: (リンクなし)"
    } > "$repo/docs/Plans.md"
}

# pointer が code fence と HTML comment の中にしか無い。
write_plans_md_hidden_pointer() {
    {
        echo "# Plans"
        echo ""
        echo "## 次の行動"
        echo ""
        echo '```markdown'
        echo "1. active な lane の Plan Packet は docs/plans/ の dated packet が正本"
        echo '```'
        echo ""
        echo "<!--"
        echo "1. active な lane の Plan Packet は docs/plans/ の dated packet が正本"
        echo "-->"
    } > "$repo/docs/Plans.md"
}

# backtick fence の中の '~~~' は fence を閉じず、pointer は fence の中に残る。
write_plans_md_mixed_fence_pointer() {
    {
        echo "# Plans"
        echo ""
        echo "## 次の行動"
        echo ""
        echo '```markdown'
        echo "~~~"
        echo "1. active な lane の Plan Packet は docs/plans/ の dated packet が正本"
        echo '```'
    } > "$repo/docs/Plans.md"
}

# inline code の中の docs/plans/ は pointer として数える（link でなく文字列の要求）。
write_plans_md_inline_code_pointer() {
    {
        echo "# Plans"
        echo ""
        echo "## 次の行動"
        echo ""
        echo '1. active な lane の Plan Packet は `docs/plans/` の dated packet が正本'
    } > "$repo/docs/Plans.md"
}

reset_packet_defaults() {
    PKT_BASE="$BASE_OLD"
    PKT_INCLUDE_WS=1
    PKT_PHASE="implementing"
    PKT_WS_RISK="R3"
    PKT_EVIDENCE_MODE=""
    PKT_EXEC_MODE=""
    PKT_PLAN_COMMIT="0123456789abcdef0123456789abcdef01234567"
    PKT_AMENDMENTS="none"
    PKT_COORDINATOR="Fable 5（fixture coordinator）"
    PKT_WRITER="Codex（fixture writer）"
    PKT_PLAN_REVIEWER="Sonnet 5（fixture plan reviewer）"
    PKT_FINAL_REVIEWER="Sonnet 5（fixture final reviewer）"
    PKT_FINAL_REVIEW_MINIMUM="2"
    PKT_HUMAN_GATE="ready,merge"
    PKT_OMIT_FIELDS=""
    PKT_WORKFLOW_STATE_EXTRA=""
    PKT_RISK_SECTION="R3"
    PKT_INCLUDE_OWNER_BUDGET=1
    PKT_INCLUDE_R3_SECTIONS=1
    PKT_INCLUDE_CONTRACT_PROBE=1
    PKT_INCLUDE_FINDINGS_FREEZE=1
    PKT_INCLUDE_GOAL_INVARIANT=1
    # old = Spec Contract + Trace Matrix（旧 template）、new = Contract Ledger + Design Readiness、
    # both = 両方、spec-only = Spec Contract だけ、none = どれも無い（PR4）
    PKT_TEMPLATE="old"
    # data = データ行 1 件、empty = header と区切り行だけ、blank = template の空行だけ（PR4）
    PKT_TRACE_ROWS="data"
    PKT_TRACE_TEST_CELL='`bash` fixture test'
    PKT_CONTRACT_PROBE_LINE="- fixture premise: verified via fixture experiment -> result ok"
    PKT_REVIEW_RESPONSE_EXTRA=""
}

packet_field_is_omitted() {
    local field="$1"
    case " ${PKT_OMIT_FIELDS:-} " in
        *" ${field} "*) return 0 ;;
        *) return 1 ;;
    esac
}

write_workflow_field() {
    local field="$1" value="$2"
    packet_field_is_omitted "$field" || echo "- ${field}: ${value}"
}

write_packet() {
    local path="$1"
    {
        echo "# Fixture Plan Packet"
        echo ""
        if [ "$PKT_INCLUDE_WS" = "1" ]; then
            echo "## Workflow State"
            echo ""
            # 旧 template の任意行（Evidence Mode / Execution Mode）は値があるときだけ書く。
            if [ -n "$PKT_EVIDENCE_MODE" ]; then
                write_workflow_field "Evidence Mode" "$PKT_EVIDENCE_MODE"
            fi
            write_workflow_field "Phase" "$PKT_PHASE"
            write_workflow_field "Risk" "$PKT_WS_RISK"
            if [ -n "$PKT_EXEC_MODE" ]; then
                write_workflow_field "Execution Mode" "$PKT_EXEC_MODE"
            fi
            write_workflow_field "Plan Commit" "$PKT_PLAN_COMMIT"
            write_workflow_field "Amendments" "$PKT_AMENDMENTS"
            write_workflow_field "Coordinator" "$PKT_COORDINATOR"
            write_workflow_field "Writer" "$PKT_WRITER"
            write_workflow_field "Plan Reviewer" "$PKT_PLAN_REVIEWER"
            write_workflow_field "Final Reviewer" "$PKT_FINAL_REVIEWER"
            write_workflow_field "Final Review Minimum" "$PKT_FINAL_REVIEW_MINIMUM"
            write_workflow_field "Human Gate" "$PKT_HUMAN_GATE"
            if [ -n "$PKT_WORKFLOW_STATE_EXTRA" ]; then
                echo "$PKT_WORKFLOW_STATE_EXTRA"
            fi
            echo ""
        fi
        if [ "$PKT_INCLUDE_OWNER_BUDGET" = "1" ]; then
            echo "## Owner Effort Budget"
            echo ""
            echo "- 介入回数上限: 3"
            echo ""
        fi
        echo "## Risk"
        echo ""
        echo "Risk: ${PKT_RISK_SECTION}"
        echo ""
        echo "Reason: fixture packet for automated PK4/PK1EXT tests."
        echo ""
        echo "## Goal"
        echo ""
        if [ "$PKT_INCLUDE_GOAL_INVARIANT" = "1" ]; then
            echo "Goal Invariant:"
            echo ""
            echo "### 最小完了条件"
            echo ""
            echo "- fixture outcome"
            echo ""
            echo "### 失敗定義"
            echo ""
            echo "- fixture failure"
            echo ""
            echo "### 非目的"
            echo ""
            echo "- fixture non-goal"
        else
            echo "fixture goal line"
        fi
        echo ""
        echo "## Scope"
        echo ""
        echo "1. fixture scope item"
        echo ""
        echo "## Non-scope"
        echo ""
        echo "- fixture non-scope item"
        echo ""
        echo "## Acceptance Criteria"
        echo ""
        echo '- `bash scripts/doc-consistency-check.sh` returns exit 0 for this fixture'
        echo ""
        echo "## Test Plan"
        echo ""
        echo 'Test Design Matrix: `docs/plans/test-matrices/fixture.md`'
        echo ""
        echo "- targeted tests: fixture only"
        echo ""
        echo "## Review Focus"
        echo ""
        echo "- fixture review focus item"
        echo ""
        if [ "$PKT_INCLUDE_R3_SECTIONS" = "1" ]; then
            case "$PKT_TEMPLATE" in
                old|both|spec-only)
                    echo "## Spec Contract"
                    echo ""
                    echo "Contract ID: SPEC-FIXTURE"
                    echo ""
                    echo "- SPEC-FIXTURE: fixture contract line"
                    echo ""
                    ;;
            esac
            case "$PKT_TEMPLATE" in
                old|both)
                    echo "## Trace Matrix"
                    echo ""
                    echo "| Spec ID | Plan Step | Test | Review Focus | Evidence |"
                    echo "|---|---|---|---|---|"
                    case "$PKT_TRACE_ROWS" in
                        data) echo "| SPEC-FIXTURE | Scope 1 | ${PKT_TRACE_TEST_CELL} | Review Focus | fixture evidence |" ;;
                        blank) echo "|  |  |  |  |  |" ;;
                    esac
                    echo ""
                    ;;
            esac
            case "$PKT_TEMPLATE" in
                new|both)
                    echo "## Design Readiness"
                    echo ""
                    echo "- 判定: ready（fixture）"
                    echo ""
                    echo "## Contract Ledger"
                    echo ""
                    echo "| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |"
                    echo "|---|---|---|---|---|"
                    case "$PKT_TRACE_ROWS" in
                        data) echo "| SPEC-FIXTURE | fixture design section | Scope 1 | ${PKT_TRACE_TEST_CELL} | 非対象 |" ;;
                        blank) echo "|  |  |  |  |  |" ;;
                    esac
                    echo ""
                    ;;
            esac
            echo "## Data Safety"
            echo ""
            echo "- fixture data safety line"
            echo ""
        fi
        if [ "$PKT_INCLUDE_CONTRACT_PROBE" = "1" ]; then
            echo "## Contract Probe"
            echo ""
            echo "$PKT_CONTRACT_PROBE_LINE"
            echo ""
        fi
        echo "## Review Response"
        echo ""
        if [ "$PKT_INCLUDE_FINDINGS_FREEZE" = "1" ] && [ "$PKT_TEMPLATE" != "new" ]; then
            echo "- Findings Freeze: frozen after fixture Broad Audit"
        fi
        if [ -n "${PKT_REVIEW_RESPONSE_EXTRA:-}" ]; then
            echo ""
            echo "$PKT_REVIEW_RESPONSE_EXTRA"
        fi
    } > "$path"
}

# fixture を起点 PKT_BASE の上の lane の 1 commit にして（packet を足した commit の親 = 起点）検査する。
lane_commit() {
    git -C "$repo" reset -q --soft "${PKT_BASE:-$BASE_OLD}"
    git -C "$repo" add -A
    git -C "$repo" commit -q --allow-empty -m lane
}

run_check() {
    local target="${1:-}"
    lane_commit
    (
        cd "$repo"
        if [ -n "$target" ]; then
            WORKFLOW_BASE_SHA="${PKT_BASE:-$BASE_OLD}" PATH="$rg_shim_dir:$PATH" REAL_RG="$real_rg" RG_ARGV_LOG="$rg_argv_log" \
                bash doc-consistency-check.sh --target plan "$target"
        else
            WORKFLOW_BASE_SHA="${PKT_BASE:-$BASE_OLD}" PATH="$rg_shim_dir:$PATH" REAL_RG="$real_rg" RG_ARGV_LOG="$rg_argv_log" \
                bash doc-consistency-check.sh --target plan
        fi
    ) > "$out" 2>&1
}

# --- 1. T-P1 正例: 新 template 形（Evidence Mode / Execution Mode 行なし、10 field）の fixture は ERROR 0 ---
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-01-01-fixture.md"
write_plans_md_linking "2026-01-01-fixture.md"
if ! run_check "docs/plans/2026-01-01-fixture.md"; then
    cat "$out" >&2
    fail "valid fixture packet was unexpectedly rejected"
fi
assert_contains "$out" "PK4: Workflow State machine 整合 OK"
assert_not_contains "$out" "Goal Invariant 構造に"

# --- 1b. SPEC-WF-PK3-TOKEN: 3 root canary / ignore / WARN exit / argv guard ---
setup_repo_dirs
reset_packet_defaults
PKT_TRACE_TEST_CELL="test_pk3_tests_root / test_pk3_src_root / test_pk3_src_tauri_root / test_pk3_ignored_only"
write_packet "$repo/docs/plans/2026-01-01-pk3-fixture.md"
write_plans_md_linking "2026-01-01-pk3-fixture.md"
mkdir -p \
    "$repo/tests" \
    "$repo/src/target" \
    "$repo/src/node_modules" \
    "$repo/src/dist" \
    "$repo/src-tauri"
printf 'test test_pk3_tests_root\n' > "$repo/tests/pk3_canary.sh"
printf 'it("test_pk3_src_root")\n' > "$repo/src/pk3_canary.ts"
printf 'fn test_pk3_src_tauri_root() {}\n' > "$repo/src-tauri/pk3_canary.rs"
printf 'fn test_pk3_ignored_only() {}\n' > "$repo/src/target/pk3_ignored.rs"
printf 'fn test_pk3_ignored_only() {}\n' > "$repo/src/node_modules/pk3_ignored.ts"
printf 'fn test_pk3_ignored_only() {}\n' > "$repo/src/dist/pk3_ignored.ts"
printf 'target/\nnode_modules/\ndist/\n' > "$repo/.gitignore"
: > "$rg_argv_log"
if ! run_check "docs/plans/2026-01-01-pk3-fixture.md"; then
    cat "$out" >&2
    fail "PK3 token fixture unexpectedly exited nonzero"
fi
assert_not_contains "$out" 'test token `test_pk3_tests_root` が tests/src/src-tauri に見つかりません'
assert_not_contains "$out" 'test token `test_pk3_src_root` が tests/src/src-tauri に見つかりません'
assert_not_contains "$out" 'test token `test_pk3_src_tauri_root` が tests/src/src-tauri に見つかりません'
assert_contains "$out" 'PK3: docs/plans/2026-01-01-pk3-fixture.md (R3) の Trace Matrix test token `test_pk3_ignored_only` が tests/src/src-tauri に見つかりません'
assert_helper_calls_have_no_negative_globs "$rg_argv_log"

# --- 2. '## Workflow State' セクション自体が欠落 ---
setup_repo_dirs
reset_packet_defaults
PKT_INCLUDE_WS=0
write_packet "$repo/docs/plans/2026-01-02-fixture.md"
write_plans_md_linking "2026-01-02-fixture.md"
if run_check "docs/plans/2026-01-02-fixture.md"; then
    fail "missing '## Workflow State' section was not rejected"
fi
assert_contains "$out" "必須セクション '## Workflow State' を欠いています"

# --- 3. PH3（T-P5 の Phase enum の ERROR を書き換え、D-145 (3)）: 旧形式の Phase の値は評価しない ---
for marker in "" github; do
    for phase in review local-verified independent-review human-confirm ready-hosted-final merge; do
        setup_repo_dirs
        reset_packet_defaults
        PKT_EVIDENCE_MODE="$marker"
        PKT_PHASE="$phase"
        write_packet "$repo/docs/plans/2026-01-03-fixture.md"
        if ! run_check "docs/plans/2026-01-03-fixture.md"; then
            cat "$out" >&2
            fail "PH3: Phase '$phase' was evaluated (marker='$marker')"
        fi
        assert_not_contains "$out" "Phase enum"
    done
done

# --- 4. Workflow State '- Risk:' と '## Risk' セクションの不一致 ---
setup_repo_dirs
reset_packet_defaults
PKT_WS_RISK="R2"
write_packet "$repo/docs/plans/2026-01-04-fixture.md"
write_plans_md_linking "2026-01-04-fixture.md"
if run_check "docs/plans/2026-01-04-fixture.md"; then
    fail "Risk mismatch between Workflow State and Risk section was not rejected"
fi
assert_contains "$out" "と不一致です"

# --- 5. T-P2 互換: 旧 template 形（Evidence Mode: github + 任意値の Execution Mode）を受理し値を評価しない ---
for mode in fable-window dual-vendor-no-fable codex-only "waterfall（任意の文字列）"; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_EVIDENCE_MODE="github"
    PKT_EXEC_MODE="$mode"
    write_packet "$repo/docs/plans/2026-01-05-fixture.md"
    write_plans_md_linking "2026-01-05-fixture.md"
    if ! run_check "docs/plans/2026-01-05-fixture.md"; then
        cat "$out" >&2
        fail "old-template packet with Execution Mode '$mode' was rejected"
    fi
    assert_contains "$out" "PK4: Workflow State machine 整合 OK"
done

# --- PR4-F3 SPEC-WF-HARNESS4-D3: 旧 template で Findings Freeze 行だけが無い R3 packet は ERROR なし（旧 section 6） ---
setup_repo_dirs
reset_packet_defaults
PKT_INCLUDE_FINDINGS_FREEZE=0
write_packet "$repo/docs/plans/2026-01-06-fixture.md"
write_plans_md_linking "2026-01-06-fixture.md"
if ! run_check "docs/plans/2026-01-06-fixture.md"; then
    cat "$out" >&2
    fail "PR4-F3: missing Findings Freeze line at R3 was rejected"
fi
assert_contains "$out" "PK4: Workflow State machine 整合 OK"
assert_not_contains "$out" "Findings Freeze"

# --- 7. PH3（書き換え）: Phase と Plan Commit の組は照合しない。計画の前後は Plan Commit だけが決める ---
setup_repo_dirs
reset_packet_defaults
PKT_PHASE="implementing"
PKT_PLAN_COMMIT="pending"
write_packet "$repo/docs/plans/2026-01-07-fixture.md"
if ! run_check "docs/plans/2026-01-07-fixture.md"; then
    cat "$out" >&2
    fail "PH3: Phase implementing with Plan Commit pending was rejected"
fi
assert_not_contains "$out" "pending のままです"

# --- 8. Owner Effort Budget 欠落（PK1 拡張、R2+ 必須） ---
setup_repo_dirs
reset_packet_defaults
PKT_INCLUDE_OWNER_BUDGET=0
write_packet "$repo/docs/plans/2026-01-08-fixture.md"
write_plans_md_linking "2026-01-08-fixture.md"
if run_check "docs/plans/2026-01-08-fixture.md"; then
    fail "missing Owner Effort Budget section was not rejected"
fi
assert_contains "$out" "必須セクション '## Owner Effort Budget' を欠いています"

# --- 9. Contract Probe 欠落（PK1 拡張、R3+ 必須） ---
setup_repo_dirs
reset_packet_defaults
PKT_INCLUDE_CONTRACT_PROBE=0
write_packet "$repo/docs/plans/2026-01-09-fixture.md"
write_plans_md_linking "2026-01-09-fixture.md"
if run_check "docs/plans/2026-01-09-fixture.md"; then
    fail "missing Contract Probe section at R3 was not rejected"
fi
assert_contains "$out" "必須セクション '## Contract Probe' を欠いています"

# --- 10. R2 では Contract Probe は非必須（regression: 過剰検出しないこと） ---
setup_repo_dirs
reset_packet_defaults
PKT_PHASE="plan-draft"
PKT_WS_RISK="R2"
PKT_PLAN_COMMIT="pending"
PKT_RISK_SECTION="R2"
PKT_INCLUDE_R3_SECTIONS=0
PKT_INCLUDE_CONTRACT_PROBE=0
PKT_INCLUDE_FINDINGS_FREEZE=0
write_packet "$repo/docs/plans/2026-01-10-fixture.md"
write_plans_md_linking "2026-01-10-fixture.md"
if ! run_check "docs/plans/2026-01-10-fixture.md"; then
    cat "$out" >&2
    fail "R2 packet without Contract Probe was incorrectly rejected"
fi
assert_not_contains "$out" "Contract Probe"

# --- 11/12. AR3（D-097 の pointer の検査を書き換え、D-145 (12)）: docs/Plans.md の '## 次の行動' を見ない ---
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-01-11-fixture-a.md"
write_packet "$repo/docs/plans/2026-01-11-fixture-b.md"
for plans_writer in write_plans_md_linking write_plans_md_no_link write_plans_md_hidden_pointer; do
    "$plans_writer"
    if ! run_check ""; then
        cat "$out" >&2
        fail "AR3: Plans.md ($plans_writer) still gates the active packets"
    fi
    assert_contains "$out" "PK4: Workflow State machine 整合 OK"
    assert_not_contains "$out" "次の行動"
done
rm -f "$repo/docs/Plans.md"
if ! run_check "docs/plans/2026-01-11-fixture-a.md"; then
    cat "$out" >&2
    fail "AR3: a missing docs/Plans.md gated the packet"
fi

# --- 13. compatibility: docs/archive/ 配下へ明示パスで渡した場合は PK4 の新チェックを skip ---
# 実在の archive packet と同じ状態（Workflow State / Owner Effort Budget / Contract Probe が
# いずれも無い、D-039 導入前の R3 packet）を再構成する（Double Audit pass1 P1 反映）
setup_repo_dirs
reset_packet_defaults
PKT_INCLUDE_WS=0
PKT_INCLUDE_OWNER_BUDGET=0
PKT_INCLUDE_CONTRACT_PROBE=0
PKT_INCLUDE_FINDINGS_FREEZE=0
mkdir -p "$repo/docs/archive/plans"
write_packet "$repo/docs/archive/plans/2020-01-01-archived-fixture.md"
if ! run_check "docs/archive/plans/2020-01-01-archived-fixture.md"; then
    cat "$out" >&2
    fail "archived packet path unexpectedly triggered a new PK1/PK4 error"
fi
assert_not_contains "$out" "docs/archive/plans/2020-01-01-archived-fixture.md (R3) は必須セクション '## Workflow State' を欠いています"
assert_not_contains "$out" "必須セクション '## Owner Effort Budget' を欠いています"
assert_not_contains "$out" "必須セクション '## Contract Probe' を欠いています"

# --- 14. 旧い 8 値の Phase もそのまま通る（PH3。値は評価しない） ---
for phase in kickoff spec-check design plan-draft plan-gate plan-approved implementing archive; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_PHASE="$phase"
    # plan-approved 以降の phase では Plan Commit: pending が field 関係 ERROR になるため実値を置く
    case "$phase" in
        plan-approved|implementing|archive)
            PKT_PLAN_COMMIT="ffffffffffffffffffffffffffffffffffffffff" ;;
    esac
    write_packet "$repo/docs/plans/2026-01-14-fixture.md"
    write_plans_md_linking "2026-01-14-fixture.md"
    if ! run_check "docs/plans/2026-01-14-fixture.md"; then
        cat "$out" >&2
        fail "valid phase enum value '$phase' was rejected"
    fi
done

# --- 15. D-046 T1: active packet の Goal Invariant 構造欠落は WARN、archive は遡及対象外 ---
setup_repo_dirs
reset_packet_defaults
PKT_INCLUDE_GOAL_INVARIANT=0
write_packet "$repo/docs/plans/2026-07-15-goal-invariant-missing.md"
write_plans_md_linking "2026-07-15-goal-invariant-missing.md"
if ! run_check "docs/plans/2026-07-15-goal-invariant-missing.md"; then
    cat "$out" >&2
    fail "Goal Invariant 欠落 WARN fixture が ERROR になった"
fi
assert_contains "$out" "D-046: docs/plans/2026-07-15-goal-invariant-missing.md の Goal Invariant 構造に"

setup_repo_dirs
reset_packet_defaults
PKT_INCLUDE_GOAL_INVARIANT=0
mkdir -p "$repo/docs/archive/plans"
write_plans_md_no_link
write_packet "$repo/docs/archive/plans/2026-07-15-archived-goal-invariant-missing.md"
if ! run_check "docs/archive/plans/2026-07-15-archived-goal-invariant-missing.md"; then
    cat "$out" >&2
    fail "archived Goal Invariant compatibility fixture が ERROR になった"
fi
assert_not_contains "$out" "D-046: docs/archive/plans/2026-07-15-archived-goal-invariant-missing.md"

# --- PR4-F10 SPEC-WF-HARNESS4-D4: WER の Retired 節の検査は走らない（旧 section 16） ---
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-07-15-wer-retired-fixture.md"
write_plans_md_linking "2026-07-15-wer-retired-fixture.md"
mkdir -p "$repo/docs/archive/plans"
printf '# Workflow Effectiveness Review\n' > "$repo/docs/archive/plans/2026-07-15-fixture-workflow-effectiveness-review.md"
if ! run_check "docs/plans/2026-07-15-wer-retired-fixture.md"; then
    cat "$out" >&2
    fail "PR4-F10: WER fixture was rejected"
fi
assert_not_contains "$out" "WER Retired"
assert_not_contains "$out" "Retired / Consolidated Rules"

# --- 17. D-046 T2/T6 の後継（D-145 (11)）: template と source docs の規範 token drift ---
# Owner Effort Budget と介入の数え方は消え、承認依頼の「何が完了するか 1 文」と goal-drift signal が残る。
assert_not_contains "$SOURCE_ROOT/docs/templates/plan-packet.md" "介入 N"
assert_not_contains "$SOURCE_ROOT/docs/templates/plan-packet.md" "Owner Effort Budget"
assert_contains "$SOURCE_ROOT/docs/DEV_WORKFLOW.md" "承認すると利用者から見て何が完了するか"
assert_contains "$SOURCE_ROOT/docs/DEV_WORKFLOW.md" "actual harm path"
assert_contains "$SOURCE_ROOT/docs/DEV_WORKFLOW.md" "affected candidate or mutation"
assert_contains "$SOURCE_ROOT/docs/DEV_WORKFLOW.md" "non-destructive revalidation"
assert_contains "$SOURCE_ROOT/docs/DEV_WORKFLOW.md" "blocker reason"
assert_contains "$SOURCE_ROOT/docs/DEV_WORKFLOW.md" "goal-drift signal"
assert_contains "$SOURCE_ROOT/docs/DEV_WORKFLOW.md" "one-shot irreversible"
assert_contains "$SOURCE_ROOT/docs/AGENT_OPERATING_MANUAL.md" "one-shot irreversible"
assert_contains "$SOURCE_ROOT/docs/AGENT_OPERATING_MANUAL.md" "task-shape"
assert_contains "$SOURCE_ROOT/docs/decision-log.md" "## D-046"

# --- 18. D-046 T8 の後継（D-145 (3)）: Phase の enum はどの script にも残らない（parity の対象が無い） ---
for phase_script in doc-consistency-check.sh check-workflow-git.sh pr-gate.py; do
    if grep -q 'WORKFLOW_STATE_PHASES\|plan-approved' "$SOURCE_ROOT/scripts/$phase_script"; then
        fail "$phase_script still evaluates the Phase enum"
    fi
done

# --- 19. D-046 T7: 実 repository の checker self-pass ---
if ! (cd "$SOURCE_ROOT" && bash scripts/doc-consistency-check.sh > "$tmp/self-pass.log" 2>&1); then
    cat "$tmp/self-pass.log" >&2
    fail "実 repository の doc-consistency-check.sh が ERROR"
fi

# --- 20. D-062 PK6: 数値主張の実測 evidence heuristic（X1 red / X2 green / X3 mutation） ---
# D-059 round1（未実測、commit 4c4284f）/ round2（実測併記、commit 863c25b）の実文言を
# Contract Probe へ注入し、Contract Probe の Test Design Matrix X1/X2/X3 と同一実証を
# スクリプト経由で再現する。

# X1: red fixture（round1 相当、backtick なし）は WARN が発火する
setup_repo_dirs
reset_packet_defaults
PKT_CONTRACT_PROBE_LINE="- P2-1 problem claimをaccept。strict mode + trapによるscript-controlled非0の2正規化と、outer 30秒より短い内部20秒 + kill-after 2秒deadlineを契約・AC・Matrixへ追加した。"
write_packet "$repo/docs/plans/2026-01-20-pk6-red.md"
write_plans_md_linking "2026-01-20-pk6-red.md"
if ! run_check "docs/plans/2026-01-20-pk6-red.md"; then
    cat "$out" >&2
    fail "PK6 red fixture が exit code に影響した（WARN-only のはず）"
fi
assert_contains "$out" "PK6: docs/plans/2026-01-20-pk6-red.md (R3) の Contract Probe/Review Response に実測 evidence のない数値主張があります"

# X2: green fixture（round2 相当、backtick でコマンド参照あり）は WARN が発火しない
setup_repo_dirs
reset_packet_defaults
PKT_CONTRACT_PROBE_LINE='- checker runtime（同一WSL2 warm state）: `scripts/doc-consistency-check.sh` fullは33.53 / 33.70 / 33.64秒、`--target plan`は20.73 / 20.01秒。'
write_packet "$repo/docs/plans/2026-01-20-pk6-green.md"
write_plans_md_linking "2026-01-20-pk6-green.md"
if ! run_check "docs/plans/2026-01-20-pk6-green.md"; then
    cat "$out" >&2
    fail "PK6 green fixture が exit code に影響した"
fi
assert_not_contains "$out" "PK6: docs/plans/2026-01-20-pk6-green.md"
assert_contains "$out" "PK6: 数値主張の実測 evidence 欠落 OK"

# X3: green fixture から backtick span のみを除去した mutant は WARN が発火する（red 化）
setup_repo_dirs
reset_packet_defaults
PKT_CONTRACT_PROBE_LINE="- checker runtime（同一WSL2 warm state）: scripts/doc-consistency-check.sh fullは33.53 / 33.70 / 33.64秒、--target planは20.73 / 20.01秒。"
write_packet "$repo/docs/plans/2026-01-20-pk6-mutant.md"
write_plans_md_linking "2026-01-20-pk6-mutant.md"
if ! run_check "docs/plans/2026-01-20-pk6-mutant.md"; then
    cat "$out" >&2
    fail "PK6 mutant fixture が exit code に影響した"
fi
assert_contains "$out" "PK6: docs/plans/2026-01-20-pk6-mutant.md (R3) の Contract Probe/Review Response に実測 evidence のない数値主張があります"

# Negative path: Contract Probe セクション自体が無い packet（R2、既存 test #10 と同じ構成 —
# Contract Probe は R3+ でのみ必須のため R2 では section 自体が欠落しても PK1 拡張は発火しない）
# は PK6 が skip し、ERROR にも WARN にもならないこと
setup_repo_dirs
reset_packet_defaults
PKT_PHASE="plan-draft"
PKT_WS_RISK="R2"
PKT_PLAN_COMMIT="pending"
PKT_RISK_SECTION="R2"
PKT_INCLUDE_R3_SECTIONS=0
PKT_INCLUDE_CONTRACT_PROBE=0
PKT_INCLUDE_FINDINGS_FREEZE=0
write_packet "$repo/docs/plans/2026-01-20-pk6-no-section.md"
write_plans_md_linking "2026-01-20-pk6-no-section.md"
if ! run_check "docs/plans/2026-01-20-pk6-no-section.md"; then
    cat "$out" >&2
    fail "Contract Probe 欠落 packet が PK6 で誤って ERROR/WARN になった"
fi
assert_not_contains "$out" "PK6: docs/plans/2026-01-20-pk6-no-section.md"
assert_contains "$out" "PK6: 数値主張の実測 evidence 欠落 OK"

# --- 21. T-P6 SPEC-PK4-F1 / M-P1: 旧形式の必須 6 field の行欠落を全数検出（担当 4 行は D-145 (3) で任意） ---
required_workflow_fields=(
    "Phase"
    "Risk"
    "Plan Commit"
    "Amendments"
    "Final Review Minimum"
    "Human Gate"
)
for field in "${required_workflow_fields[@]}"; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_OMIT_FIELDS="$field"
    write_packet "$repo/docs/plans/2026-01-21-required-field.md"
    write_plans_md_linking "2026-01-21-required-field.md"
    if run_check "docs/plans/2026-01-21-required-field.md"; then
        fail "missing Workflow State field '$field' was not rejected"
    fi
    assert_contains "$out" "Workflow State に '- ${field}:' 行がありません"
done

# 空値行は存在扱いにせず、欠落と同じ ERROR にする。
setup_repo_dirs
reset_packet_defaults
PKT_AMENDMENTS=""
write_packet "$repo/docs/plans/2026-01-21-empty-field.md"
if run_check "docs/plans/2026-01-21-empty-field.md"; then
    fail "empty Workflow State Amendments field was not rejected"
fi
assert_contains "$out" "Workflow State に '- Amendments:' 行がありません"

# PH3（書き換え: 担当 4 行の欠落・空値の ERROR）: 担当 4 行は在っても無くても空でも受ける
for role in Coordinator Writer "Plan Reviewer" "Final Reviewer"; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_OMIT_FIELDS="$role"
    write_packet "$repo/docs/plans/2026-01-21-role.md"
    if ! run_check "docs/plans/2026-01-21-role.md"; then
        cat "$out" >&2
        fail "PH3: packet without the $role line was rejected"
    fi
done
setup_repo_dirs
reset_packet_defaults
PKT_COORDINATOR=""
write_packet "$repo/docs/plans/2026-01-21-role.md"
if ! run_check "docs/plans/2026-01-21-role.md"; then
    cat "$out" >&2
    fail "PH3: empty Coordinator line was rejected"
fi

# --- 22. T-P4: legacy field 3 種は marker の有無に関わらず ERROR ---
for marker in "" github; do
    for legacy_field in "Reviewed Content HEAD" "Final Exact-HEAD Evidence" "Hosted CI Requirement"; do
        setup_repo_dirs
        reset_packet_defaults
        PKT_EVIDENCE_MODE="$marker"
        PKT_WORKFLOW_STATE_EXTRA="- ${legacy_field}: required"
        write_packet "$repo/docs/plans/2026-01-22-legacy-field.md"
        write_plans_md_linking "2026-01-22-legacy-field.md"
        if run_check "docs/plans/2026-01-22-legacy-field.md"; then
            fail "legacy field '$legacy_field' was not rejected (marker='$marker')"
        fi
        assert_contains "$out" "legacy field '${legacy_field}'"
    done
done

# --- 23. T-P10 SPEC-PK4-F3 / M-P3: pending / 日本語先頭値は正当 ---
setup_repo_dirs
reset_packet_defaults
PKT_PHASE="plan-gate"
PKT_PLAN_COMMIT="pending"
PKT_PLAN_REVIEWER="未定（plan-gate 時に選任）"
PKT_HUMAN_GATE="ready,merge"
write_packet "$repo/docs/plans/2026-01-23-valid-free-form.md"
write_plans_md_linking "2026-01-23-valid-free-form.md"
if ! run_check "docs/plans/2026-01-23-valid-free-form.md"; then
    cat "$out" >&2
    fail "pending/non-ASCII Workflow State values were rejected"
fi
assert_contains "$out" "PK4: Workflow State machine 整合 OK"

# --- 24. SPEC-PK4-F4 / M-P4: 任意追加 field は禁止しない ---
setup_repo_dirs
reset_packet_defaults
PKT_WORKFLOW_STATE_EXTRA="- Draft Provenance: synthetic fixture"
write_packet "$repo/docs/plans/2026-01-24-extra-field.md"
write_plans_md_linking "2026-01-24-extra-field.md"
if ! run_check "docs/plans/2026-01-24-extra-field.md"; then
    cat "$out" >&2
    fail "optional Workflow State field was rejected"
fi
assert_contains "$out" "PK4: Workflow State machine 整合 OK"

# --- 25. SPEC-PK4-F5 / M-P5: archive 明示 path は新設 field 検査を skip ---
setup_repo_dirs
reset_packet_defaults
PKT_OMIT_FIELDS="Coordinator"
mkdir -p "$repo/docs/archive/plans"
write_plans_md_no_link
write_packet "$repo/docs/archive/plans/2026-01-25-archived-field.md"
if ! run_check "docs/archive/plans/2026-01-25-archived-field.md"; then
    cat "$out" >&2
    fail "archived packet unexpectedly triggered required Workflow State field check"
fi
assert_not_contains "$out" "Workflow State に '- Coordinator:' 行がありません"

# --- 26. SPEC-PK4-F5 / M-P6: R2 は検査対象、R1 は閾値 skip ---
setup_repo_dirs
reset_packet_defaults
PKT_PHASE="plan-draft"
PKT_PLAN_COMMIT="pending"
PKT_WS_RISK="R2"
PKT_RISK_SECTION="R2"
PKT_INCLUDE_R3_SECTIONS=0
PKT_INCLUDE_CONTRACT_PROBE=0
PKT_INCLUDE_FINDINGS_FREEZE=0
PKT_OMIT_FIELDS="Human Gate"
write_packet "$repo/docs/plans/2026-01-26-r2-field.md"
if run_check "docs/plans/2026-01-26-r2-field.md"; then
    fail "R2 packet missing a required Workflow State field was not rejected"
fi
assert_contains "$out" "Workflow State に '- Human Gate:' 行がありません"

setup_repo_dirs
reset_packet_defaults
PKT_PHASE="plan-draft"
PKT_PLAN_COMMIT="pending"
PKT_WS_RISK="R1"
PKT_RISK_SECTION="R1"
PKT_INCLUDE_R3_SECTIONS=0
PKT_INCLUDE_CONTRACT_PROBE=0
PKT_INCLUDE_FINDINGS_FREEZE=0
PKT_OMIT_FIELDS="Writer"
write_packet "$repo/docs/plans/2026-01-26-r1-field.md"
write_plans_md_linking "2026-01-26-r1-field.md"
if ! run_check "docs/plans/2026-01-26-r1-field.md"; then
    cat "$out" >&2
    fail "R1 packet was incorrectly subjected to required Workflow State field check"
fi
assert_not_contains "$out" "Workflow State に '- Writer:' 行がありません"

# --- 27. T-P1 SPEC-PK4-F1..F5 / M-P7: 10 field default fixture は ERROR 0 ---
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-01-27-complete-fields.md"
write_plans_md_linking "2026-01-27-complete-fields.md"
if ! run_check "docs/plans/2026-01-27-complete-fields.md"; then
    cat "$out" >&2
    fail "complete 10-field Workflow State fixture was rejected"
fi
assert_contains "$out" "PK4: Workflow State machine 整合 OK"

# --- PR4-F1 SPEC-WF-HARNESS4-D1/D2: 新 template の R3 packet（Contract Ledger・Design Readiness、Findings Freeze 行なし）は通る ---
setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="new"
write_packet "$repo/docs/plans/2026-02-01-pr4-new-template.md"
write_plans_md_linking "2026-02-01-pr4-new-template.md"
if ! run_check "docs/plans/2026-02-01-pr4-new-template.md"; then
    cat "$out" >&2
    fail "PR4-F1: new-template R3 packet was rejected"
fi
assert_contains "$out" "PK1: Plan Packet presence OK"
assert_contains "$out" "PK4: Workflow State machine 整合 OK"
assert_not_contains "$out" "table に data row がありません"
assert_not_contains "$out" "Findings Freeze"

# Negative Paths: Contract Ledger と旧い組の両方がある packet も通る
setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="both"
write_packet "$repo/docs/plans/2026-02-01-pr4-both.md"
write_plans_md_linking "2026-02-01-pr4-both.md"
if ! run_check "docs/plans/2026-02-01-pr4-both.md"; then
    cat "$out" >&2
    fail "PR4-F1: packet with both Contract Ledger and the old pair was rejected"
fi

# --- PR4-F2 SPEC-WF-HARNESS4-D2: 旧 template の R3 packet（既定の fixture）は通る ---
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-02-02-pr4-old-template.md"
write_plans_md_linking "2026-02-02-pr4-old-template.md"
if ! run_check "docs/plans/2026-02-02-pr4-old-template.md"; then
    cat "$out" >&2
    fail "PR4-F2: old-template R3 packet was rejected"
fi
assert_contains "$out" "PK1: Plan Packet presence OK"
assert_contains "$out" "PK4: Workflow State machine 整合 OK"

# --- PR4-F2b SPEC-WF-HARNESS4-D2: 旧 template で Trace Matrix のデータ行が 0 件は ERROR でなく PK3 の WARN ---
setup_repo_dirs
reset_packet_defaults
PKT_TRACE_ROWS="empty"
write_packet "$repo/docs/plans/2026-02-02-pr4-old-empty-trace.md"
write_plans_md_linking "2026-02-02-pr4-old-empty-trace.md"
if ! run_check "docs/plans/2026-02-02-pr4-old-empty-trace.md"; then
    cat "$out" >&2
    fail "PR4-F2b: old-template packet with an empty Trace Matrix was rejected"
fi
assert_contains "$out" "PK3: docs/plans/2026-02-02-pr4-old-empty-trace.md (R3) の Trace Matrix table に data row がありません"
assert_not_contains "$out" "データ行がありません"

# --- PR4-F4 SPEC-WF-HARNESS4-D2: R3 で Contract Ledger も旧い組も無いと PK1 の ERROR ---
setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="none"
write_packet "$repo/docs/plans/2026-02-04-pr4-no-ledger.md"
write_plans_md_linking "2026-02-04-pr4-no-ledger.md"
if run_check "docs/plans/2026-02-04-pr4-no-ledger.md"; then
    fail "PR4-F4: R3 packet without Contract Ledger or the old pair was accepted"
fi
assert_contains "$out" "PK1: docs/plans/2026-02-04-pr4-no-ledger.md (R3) は Contract Ledger（旧 template は Spec Contract と Trace Matrix）を欠いています"

# PR4-F4b: Spec Contract だけ（Trace Matrix と Ledger なし）も ERROR
setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="spec-only"
write_packet "$repo/docs/plans/2026-02-04-pr4-spec-only.md"
write_plans_md_linking "2026-02-04-pr4-spec-only.md"
if run_check "docs/plans/2026-02-04-pr4-spec-only.md"; then
    fail "PR4-F4b: R3 packet with Spec Contract only was accepted"
fi
assert_contains "$out" "PK1: docs/plans/2026-02-04-pr4-spec-only.md (R3) は Contract Ledger（旧 template は Spec Contract と Trace Matrix）を欠いています"

# --- PR4-F5 SPEC-WF-HARNESS4-D3: Plan Commit に確定待ちの注記（Phase plan-gate）は PK4 の書式 ERROR ---
setup_repo_dirs
reset_packet_defaults
PKT_PHASE="plan-gate"
PKT_PLAN_COMMIT="TBD（plan-approved で確定）"
write_packet "$repo/docs/plans/2026-02-05-pr4-plan-commit-note.md"
write_plans_md_linking "2026-02-05-pr4-plan-commit-note.md"
if run_check "docs/plans/2026-02-05-pr4-plan-commit-note.md"; then
    fail "PR4-F5: Plan Commit with a pending note was accepted"
fi
assert_contains "$out" "Plan Commit は pending か 40 桁"

# --- PR4-F6 SPEC-WF-HARNESS4-D3: pending（plan-gate）と 40 桁の小文字 hex（implementing）は通る ---
for pr4_f6_case in "plan-gate:pending" "implementing:0123456789abcdef0123456789abcdef01234567"; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_PHASE="${pr4_f6_case%%:*}"
    PKT_PLAN_COMMIT="${pr4_f6_case#*:}"
    write_packet "$repo/docs/plans/2026-02-06-pr4-plan-commit-valid.md"
    write_plans_md_linking "2026-02-06-pr4-plan-commit-valid.md"
    if ! run_check "docs/plans/2026-02-06-pr4-plan-commit-valid.md"; then
        cat "$out" >&2
        fail "PR4-F6: valid Plan Commit '$pr4_f6_case' was rejected"
    fi
    assert_not_contains "$out" "Plan Commit は pending か 40 桁"
done

# --- PR4-F7 SPEC-WF-HARNESS4-D3: 書式外の Plan Commit（Phase plan-gate）はどれも PK4 の書式 ERROR ---
pr4_sha40="0123456789abcdef0123456789abcdef01234567"
pr4_f7_values=(
    "abc1234"
    "${pr4_sha40:0:39}"
    "${pr4_sha40}0"
    "0123456789ABCDEF0123456789ABCDEF01234567"
    "pending（注記）"
    "${pr4_sha40}（注記）"
    "${pr4_sha40} "
    "${pr4_sha40}"$'\t'
)
for pr4_f7_value in "${pr4_f7_values[@]}"; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_PHASE="plan-gate"
    PKT_PLAN_COMMIT="$pr4_f7_value"
    write_packet "$repo/docs/plans/2026-02-07-pr4-plan-commit-invalid.md"
    write_plans_md_linking "2026-02-07-pr4-plan-commit-invalid.md"
    if run_check "docs/plans/2026-02-07-pr4-plan-commit-invalid.md"; then
        fail "PR4-F7: invalid Plan Commit '$pr4_f7_value' was accepted"
    fi
    assert_contains "$out" "Plan Commit は pending か 40 桁"
done

# --- PR4-F7b SPEC-WF-HARNESS4-D3: Plan Commit: の直後で落とすのは ASCII の空白とタブだけ（helper と同じ、locale に依らない） ---
# UTF-8 の locale では [[:space:]] が U+3000 に当たるので、LC_ALL=C.UTF-8 で走らせる。
for pr4_f7b_case in "ok: ${pr4_sha40}" "ok:"$'\t'"${pr4_sha40}" "ng:"$'\xe3\x80\x80'"${pr4_sha40}"; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_PHASE="plan-gate"
    PKT_PLAN_COMMIT="${pr4_f7b_case#*:}"
    write_packet "$repo/docs/plans/2026-02-07-pr4-plan-commit-prefix.md"
    write_plans_md_linking "2026-02-07-pr4-plan-commit-prefix.md"
    if [ "${pr4_f7b_case%%:*}" = "ok" ]; then
        if ! LC_ALL=C.UTF-8 run_check "docs/plans/2026-02-07-pr4-plan-commit-prefix.md"; then
            cat "$out" >&2
            fail "PR4-F7b: Plan Commit with an ASCII blank prefix was rejected"
        fi
        assert_not_contains "$out" "Plan Commit は pending か 40 桁"
    else
        if LC_ALL=C.UTF-8 run_check "docs/plans/2026-02-07-pr4-plan-commit-prefix.md"; then
            fail "PR4-F7b: Plan Commit with a U+3000 prefix was accepted"
        fi
        assert_contains "$out" "Plan Commit は pending か 40 桁"
    fi
done

# --- PR4-F8 SPEC-WF-HARNESS4-D5/D14: R4 の review-only skip 行は評価しない。R4 の Minimum 1 は PK4 が ERROR ---
setup_repo_dirs
r4_pr4_defaults() {
    reset_packet_defaults
    PKT_WS_RISK="R4"
    PKT_RISK_SECTION="R4"
    PKT_HUMAN_GATE="ready,merge,r4"
}
r4_pr4_defaults
PKT_REVIEW_RESPONSE_EXTRA="Review-only skipped because: fixture reason"
write_packet "$repo/docs/plans/2026-02-08-pr4-r4-skip-line.md"
write_plans_md_linking "2026-02-08-pr4-r4-skip-line.md"
if ! run_check "docs/plans/2026-02-08-pr4-r4-skip-line.md"; then
    cat "$out" >&2
    fail "PR4-F8: R4 packet with a review-only skip line was rejected"
fi
assert_not_contains "$out" "review-only skip"

setup_repo_dirs
r4_pr4_defaults
PKT_FINAL_REVIEW_MINIMUM="1"
write_packet "$repo/docs/plans/2026-02-08-pr4-r4-minimum-1.md"
write_plans_md_linking "2026-02-08-pr4-r4-minimum-1.md"
if run_check "docs/plans/2026-02-08-pr4-r4-minimum-1.md"; then
    fail "PR4-F8: R4 packet with Final Review Minimum 1 was accepted"
fi
assert_contains "$out" "Final Review Minimum は1/2（R4は2）が必須です"

# --- PR4-F9 SPEC-WF-HARNESS4-D2: PK3 は Contract Ledger の test token と placeholder を WARN にする ---
setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="new"
PKT_TRACE_TEST_CELL="test_pr4_f9_missing_token / <fixture placeholder>"
write_packet "$repo/docs/plans/2026-02-09-pr4-ledger-warn.md"
write_plans_md_linking "2026-02-09-pr4-ledger-warn.md"
if ! run_check "docs/plans/2026-02-09-pr4-ledger-warn.md"; then
    cat "$out" >&2
    fail "PR4-F9: Contract Ledger WARN fixture exited nonzero"
fi
assert_contains "$out" 'PK3: docs/plans/2026-02-09-pr4-ledger-warn.md (R3) の Contract Ledger test token `test_pr4_f9_missing_token` が tests/src/src-tauri に見つかりません'
assert_contains "$out" "PK3: docs/plans/2026-02-09-pr4-ledger-warn.md (R3) の Contract Ledger table に placeholder が残っています"

# --- PR4-F11 SPEC-WF-HARNESS4-D2: 新 template でデータ行 0 件の Contract Ledger は PK1 の ERROR ---
for pr4_f11_rows in empty blank; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_TEMPLATE="new"
    PKT_TRACE_ROWS="$pr4_f11_rows"
    write_packet "$repo/docs/plans/2026-02-11-pr4-empty-ledger.md"
    write_plans_md_linking "2026-02-11-pr4-empty-ledger.md"
    if run_check "docs/plans/2026-02-11-pr4-empty-ledger.md"; then
        fail "PR4-F11: Contract Ledger without data rows ($pr4_f11_rows) was accepted"
    fi
    assert_contains "$out" "PK1: docs/plans/2026-02-11-pr4-empty-ledger.md (R3) の Contract Ledger にデータ行がありません"
done

# --- PR4-F12 SPEC-WF-HARNESS4-D3: archive の明示 path は Plan Commit の書式検査の対象外 ---
setup_repo_dirs
reset_packet_defaults
PKT_PLAN_COMMIT="abc1234"
mkdir -p "$repo/docs/archive/plans"
write_plans_md_no_link
write_packet "$repo/docs/archive/plans/2026-02-12-pr4-archived-short-sha.md"
if ! run_check "docs/archive/plans/2026-02-12-pr4-archived-short-sha.md"; then
    cat "$out" >&2
    fail "PR4-F12: archived packet with a short Plan Commit was rejected"
fi
assert_not_contains "$out" "Plan Commit は pending か 40 桁"

# --- PK4-DUP SPEC D-102 D3: key の重複は helper の workflow_fields が判定し、PK4 が ERROR にする ---
pk4_dup_case() {
    local label="$1" expect="$2" extra="$3"
    setup_repo_dirs
    reset_packet_defaults
    PKT_WORKFLOW_STATE_EXTRA="$extra"
    write_packet "$repo/docs/plans/2026-03-01-pk4-dup.md"
    write_plans_md_linking "2026-03-01-pk4-dup.md"
    if [ "$expect" = reject ]; then
        if run_check "docs/plans/2026-03-01-pk4-dup.md"; then
            fail "$label: duplicate Workflow State key was accepted"
        fi
        assert_contains "$out" "PK4: docs/plans/2026-03-01-pk4-dup.md の Workflow State に重複する field があります"
    else
        if ! run_check "docs/plans/2026-03-01-pk4-dup.md"; then
            cat "$out" >&2
            fail "$label: packet without a duplicate key was rejected"
        fi
        assert_contains "$out" "PK4: Workflow State machine 整合 OK"
        assert_not_contains "$out" "重複する field"
    fi
}
pk4_dup_case "PK4-DUP-1 必須 field の重複" reject "- Human Gate: ready,merge"
pk4_dup_case "PK4-DUP-2 遷移記録の同じ key" reject $'- plan-draft → plan-gate（2026-01-01）: a\n- plan-draft → plan-gate（2026-01-01）: b'
pk4_dup_case "PK4-DUP-3 遷移記録の違う key" accept $'- plan-draft → plan-gate（round 1）: a\n- plan-draft → plan-gate（round 2）: b'
pk4_dup_case "PK4-DUP-4 HTML comment の中" accept $'<!--\n- Phase: x\n-->'
pk4_dup_case "PK4-DUP-5 字下げした fence の中" reject $'  ```\n- Phase: x\n  ```'
pk4_dup_case "PK4-DUP-6 列 0 の fence の中" accept $'```\n- Phase: x\n```'

# PK4-DUP-7: helper が無ければ fail-closed の ERROR
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-03-01-pk4-dup.md"
write_plans_md_linking "2026-03-01-pk4-dup.md"
rm -f "$repo/scripts/pr-gate.py"
if run_check "docs/plans/2026-03-01-pk4-dup.md"; then
    cp "$SOURCE_ROOT/scripts/pr-gate.py" "$repo/scripts/"
    fail "PK4-DUP-7: missing scripts/pr-gate.py was accepted (fail-open)"
fi
cp "$SOURCE_ROOT/scripts/pr-gate.py" "$repo/scripts/"
assert_contains "$out" "PK4: docs/plans/2026-03-01-pk4-dup.md の Workflow State の重複の検査に python3 と scripts/pr-gate.py が必要です"

# PK4-DUP-8: Workflow State が 2 つ（helper は missing/ambiguous）でも重複の誤報をしない
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-03-01-pk4-dup.md"
write_plans_md_linking "2026-03-01-pk4-dup.md"
{
    echo ""
    sed -n '/^## Workflow State$/,/^## Owner Effort Budget$/p' "$repo/docs/plans/2026-03-01-pk4-dup.md" | sed '$d'
} >> "$repo/docs/plans/2026-03-01-pk4-dup.md"
[ "$(grep -c '^## Workflow State$' "$repo/docs/plans/2026-03-01-pk4-dup.md")" = 2 ] ||
    fail "PK4-DUP-8: fixture does not have two Workflow State sections"
run_check "docs/plans/2026-03-01-pk4-dup.md" || true
assert_not_contains "$out" "重複する field"
assert_not_contains "$out" "python3 と scripts/pr-gate.py が必要"

# --- PK4-HELPER SPEC D-107 (1): Workflow State は helper の parse_packet が判定し、PK4 が ERROR にする ---
PK4_HELPER_PACKET="docs/plans/2026-03-03-pk4-helper.md"
PK4_HELPER_REJECT="PK4: $PK4_HELPER_PACKET の Workflow State を helper（parse_packet）が拒否: "
pk4_helper_case() {
    local label="$1" expect="$2" var="$3" value="$4"
    setup_repo_dirs
    reset_packet_defaults
    printf -v "$var" '%s' "$value"
    write_packet "$repo/$PK4_HELPER_PACKET"
    write_plans_md_linking "2026-03-03-pk4-helper.md"
    if [ "$expect" = reject ]; then
        if run_check "$PK4_HELPER_PACKET"; then
            fail "$label: $var='$value' was accepted"
        fi
        assert_contains "$out" "$PK4_HELPER_REJECT"
    else
        if ! run_check "$PK4_HELPER_PACKET"; then
            cat "$out" >&2
            fail "$label: $var='$value' was rejected"
        fi
        assert_contains "$out" "PK4: Workflow State machine 整合 OK"
        assert_not_contains "$out" "helper（parse_packet）が拒否"
    fi
    echo "PASS: $label ($var)"
}
pk4_full_a="1111111111111111111111111111111111111111"
pk4_full_b="2222222222222222222222222222222222222222"
# T4-1: 値の末尾の空白・タブ
for pk4_case in PKT_WS_RISK="R3 " PKT_AMENDMENTS="none " \
    PKT_FINAL_REVIEW_MINIMUM="2 " PKT_HUMAN_GATE="ready,merge " PKT_EVIDENCE_MODE="github " \
    PKT_HUMAN_GATE=$'ready,merge\t'; do
    pk4_helper_case "PK4-HELPER: trailing blank" reject "${pk4_case%%=*}" "${pk4_case#*=}"
done
# T4-2: enum の後ろの注記
# PH3（書き換え）: 旧形式の Phase の値は評価しないので、末尾の空白・注記があっても拒まない
pk4_helper_case "PK4-HELPER: Phase value not evaluated" accept PKT_PHASE "implementing（注記）"
pk4_helper_case "PK4-HELPER: Phase value not evaluated" accept PKT_PHASE "implementing "
pk4_helper_case "PK4-HELPER: trailing note" reject PKT_WS_RISK "R3（注記）"
# T4-3: Amendments の書式・重複、Human Gate の重複
pk4_helper_case "PK4-HELPER: amendments and gate duplicates" reject PKT_AMENDMENTS "abcdef01"
pk4_helper_case "PK4-HELPER: amendments and gate duplicates" reject PKT_AMENDMENTS "$pk4_full_a, $pk4_full_a"
pk4_helper_case "PK4-HELPER: amendments and gate duplicates" reject PKT_HUMAN_GATE "ready,merge,ready"
# T4-5: helper が受理する形（旧・新 template、役割の値の末尾の空白、40 桁の Amendments 2 件）は通す
pk4_helper_case "PK4-HELPER: accepted shapes" accept PKT_TEMPLATE "old"
pk4_helper_case "PK4-HELPER: accepted shapes" accept PKT_TEMPLATE "new"
pk4_helper_case "PK4-HELPER: accepted shapes" accept PKT_COORDINATOR "Fable 5（fixture coordinator） "
pk4_helper_case "PK4-HELPER: accepted shapes" accept PKT_AMENDMENTS "$pk4_full_a, $pk4_full_b"

# T4-4: '## Workflow State' が 2 つは helper の missing/ambiguous で ERROR
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/$PK4_HELPER_PACKET"
write_plans_md_linking "2026-03-03-pk4-helper.md"
{
    echo ""
    sed -n '/^## Workflow State$/,/^## Owner Effort Budget$/p' "$repo/$PK4_HELPER_PACKET" | sed '$d'
} >> "$repo/$PK4_HELPER_PACKET"
if run_check "$PK4_HELPER_PACKET"; then
    fail "PK4-HELPER: ambiguous section was accepted"
fi
assert_contains "$out" "${PK4_HELPER_REJECT}packet Workflow State missing/ambiguous"
echo "PASS: PK4-HELPER: ambiguous section"

# --- PK1-H2 SPEC D-102 D5: packet の節は `##` の見出しだけ。`###` の小見出しは節を満たさない ---
setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="new"
write_packet "$repo/docs/plans/2026-03-02-pk1-h2.md"
write_plans_md_linking "2026-03-02-pk1-h2.md"
sed -i 's/^## Contract Ledger$/### Contract Ledger/' "$repo/docs/plans/2026-03-02-pk1-h2.md"
if run_check "docs/plans/2026-03-02-pk1-h2.md"; then
    fail "PK1-H2-1: R3 packet with only ### Contract Ledger was accepted"
fi
assert_contains "$out" "PK1: docs/plans/2026-03-02-pk1-h2.md (R3) は Contract Ledger（旧 template は Spec Contract と Trace Matrix）を欠いています"

setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="new"
write_packet "$repo/docs/plans/2026-03-02-pk1-h2.md"
write_plans_md_linking "2026-03-02-pk1-h2.md"
sed -i 's/^## Data Safety$/### Data Safety/' "$repo/docs/plans/2026-03-02-pk1-h2.md"
if run_check "docs/plans/2026-03-02-pk1-h2.md"; then
    fail "PK1-H2-2: ### Data Safety satisfied the required section"
fi
assert_contains "$out" "必須セクション '## Data Safety' を欠いています"

# PK1-H2-3: `## Contract Ledger` と、Test Plan の下（Matrix の link の後）の空の `### Contract Ledger` の両方 → OK
setup_repo_dirs
reset_packet_defaults
PKT_TEMPLATE="new"
write_packet "$repo/docs/plans/2026-03-02-pk1-h2.md"
write_plans_md_linking "2026-03-02-pk1-h2.md"
sed -i '/^Test Design Matrix: /a ### Contract Ledger' "$repo/docs/plans/2026-03-02-pk1-h2.md"
[ "$(grep -c 'Contract Ledger$' "$repo/docs/plans/2026-03-02-pk1-h2.md")" = 2 ] ||
    fail "PK1-H2-3: fixture does not have both Contract Ledger headings"
if ! run_check "docs/plans/2026-03-02-pk1-h2.md"; then
    cat "$out" >&2
    fail "PK1-H2-3: packet with ## and ### Contract Ledger was rejected"
fi
assert_contains "$out" "PK1: Plan Packet presence OK"
assert_contains "$out" "PK3: Plan Packet heuristic warnings OK"

echo "PASS: doc-consistency-plan-packet"

# T-P3: Evidence Mode 行は任意。書くなら github だけ（legacy / 未知値は ERROR）。
for marker in legacy mystery; do
    setup_repo_dirs
    reset_packet_defaults
    PKT_EVIDENCE_MODE="$marker"
    write_packet "$repo/docs/plans/2026-01-01-fixture.md"
    write_plans_md_linking "2026-01-01-fixture.md"
    if run_check; then fail "Evidence Mode '$marker' accepted"; fi
    assert_contains "$out" "Evidence Mode は廃止。書くなら github"
done
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-01-01-fixture.md"
write_plans_md_linking "2026-01-01-fixture.md"
if ! run_check; then cat "$out" >&2; fail "markerless active packet rejected"; fi

# T-P7: R4 は Final Review Minimum 2 と Human Gate の r4 が必須。Human Gate は ready,merge を含む（marker なし）。
r4_defaults() {
    reset_packet_defaults
    PKT_WS_RISK="R4"
    PKT_RISK_SECTION="R4"
    PKT_HUMAN_GATE="ready,merge,r4"
}
setup_repo_dirs
r4_defaults
write_packet "$repo/docs/plans/2026-01-01-fixture.md"
write_plans_md_linking "2026-01-01-fixture.md"
if ! run_check; then cat "$out" >&2; fail "valid R4 packet rejected"; fi
for r4_case in minimum-1 human-gate-none r4-token-missing; do
    setup_repo_dirs
    r4_defaults
    case "$r4_case" in
        minimum-1) PKT_FINAL_REVIEW_MINIMUM="1"; r4_message="Final Review Minimum は1/2（R4は2）が必須です" ;;
        human-gate-none) PKT_HUMAN_GATE="none"; r4_message="Human Gate は ready,merge と必要な manual/r4 を明示してください" ;;
        r4-token-missing) PKT_HUMAN_GATE="ready,merge"; r4_message="Human Gate は ready,merge と必要な manual/r4 を明示してください" ;;
    esac
    write_packet "$repo/docs/plans/2026-01-01-fixture.md"
    write_plans_md_linking "2026-01-01-fixture.md"
    if run_check; then fail "R4 case '$r4_case' accepted"; fi
    assert_contains "$out" "$r4_message"
done
echo "PASS: workflow state schema"

# 衛生 batch 4 S1b / AC6: full checker の route 正例と未定義カラム負例。
# ponytail: 拡張子と同名の不正カラムは除外される。区別が必要なら別途抽出方式を設計する。
hygiene_repo="$tmp/hygiene"
mkdir -p "$hygiene_repo/docs/function-design" "$hygiene_repo/scripts" \
    "$hygiene_repo/src-tauri/src" "$hygiene_repo/src/lib"
git init -q "$hygiene_repo"
cp "$SOURCE_ROOT/scripts/doc-consistency-check.sh" "$hygiene_repo/scripts/"
cp "$SOURCE_ROOT/scripts/check-command-drift.sh" "$hygiene_repo/scripts/"
cat > "$hygiene_repo/docs/DB_DESIGN.md" <<'DB'
## 1. suppliers
### suppliers カラム定義
| id | INTEGER |
DB
# full の C2 と command registry check に必要な最小の合成入力。
printf 'fn dummy_fixture_fn(x: i32) -> i32\n' > "$hygiene_repo/docs/function-design/00-dummy.md"
cat > "$hygiene_repo/src-tauri/src/lib.rs" <<'RS'
#[tauri::command]
pub fn fixture_command() {}
generate_handler![cmd::fixture::fixture_command]
collect_commands![cmd::fixture::fixture_command]
RS
printf '__TAURI_INVOKE("fixture_command")\n' > "$hygiene_repo/src/lib/bindings.ts"
for hygiene_case in route missing_column; do
    if [ "$hygiene_case" = route ]; then
        printf 'src/routes/settings/%s.%s\n' suppliers tsx
    else
        printf '%s.%s\n' suppliers hygiene_missing_column
    fi > "$hygiene_repo/docs/function-design/01-reference.md"
    if ! (cd "$hygiene_repo" && bash scripts/doc-consistency-check.sh) > "$out" 2>&1; then
        cat "$out" >&2
        fail "S1b $hygiene_case fixture exited nonzero"
    fi
    assert_not_contains "$out" '[ERROR]'
    assert_contains "$out" 'ERROR なし'
    assert_contains "$out" 'DB_DESIGN.md: 1テーブル, 1カラムを検出'
    if [ "$hygiene_case" = route ]; then
        if grep -Fq 'カラムがDB_DESIGN.mdに未定義' "$out"; then
            grep -E 'カラムがDB_DESIGN|結果:' "$out" >&2
            fail 'S1b route path was misread as an undefined column (exit 0, ERRORなし)'
        fi
    else
        assert_contains "$out" 'suppliers.hygiene_missing_column — カラムがDB_DESIGN.mdに未定義'
    fi
    echo "PASS: S1b $hygiene_case (exit 0, ERRORなし)"
done

# T11 / D-137 (2): R0 (check_stale_parent_doc_references) と R1 (check_code_doc_references) は
# src-tauri/src と、開発用の bin の置き場 src-tauri/dev-tools/src の両方を走査する。
# R0 の除外 generate_traceability.rs（索引として FUNCTION_DESIGN.md を読む）は移した後も効く。
setup_repo_dirs
reset_packet_defaults
write_packet "$repo/docs/plans/2026-01-01-fixture.md"
write_plans_md_linking "2026-01-01-fixture.md"
r0r1_body='// FUNCTION_DESIGN.md
// function-design/99-missing.md'
for r0r1_dir in "src-tauri/src/bin" "src-tauri/dev-tools/src/bin"; do
    mkdir -p "$repo/$r0r1_dir"
    printf '%s\n' "$r0r1_body" > "$repo/$r0r1_dir/r0r1_fixture.rs"
done
printf '// FUNCTION_DESIGN.md\n' > "$repo/src-tauri/dev-tools/src/bin/generate_traceability.rs"
if run_check; then cat "$out" >&2; fail "R0/R1 violations in src-tauri bins accepted"; fi
for r0r1_dir in "src-tauri/src/bin" "src-tauri/dev-tools/src/bin"; do
    assert_contains "$out" "[ERROR] $r0r1_dir/r0r1_fixture.rs:1: 'FUNCTION_DESIGN.md' を直接参照"
    assert_contains "$out" "[ERROR] $r0r1_dir/r0r1_fixture.rs:2: 参照先 'docs/function-design/99-missing.md' が存在しません"
done
assert_not_contains "$out" "src-tauri/dev-tools/src/bin/generate_traceability.rs:1: 'FUNCTION_DESIGN.md' を直接参照"
rm -rf "$repo/src-tauri"
echo "PASS: R0/R1 scan src-tauri/src and src-tauri/dev-tools/src (D-137)"

# ============================================================================
# D-145: 新形式の packet（PK1N・PH7・PH10）、active な packet の定義（AR2・AR6）、新しい path:行（FL1〜FL3）
# ============================================================================
write_new_packet() { # $1 path, $2 risk; NEW_EXTRA is appended
    local path="$1" risk="${2:-R3}"
    {
        printf '# Plan Packet: fixture\n\n## Workflow State\n\n- Risk: %s\n- Plan Commit: pending\n- Amendments: none\n\n' "$risk"
        printf '## Risk\n\nRisk: %s\n\nReason: synthetic fixture.\n\n' "$risk"
        printf '## Goal\n\nGoal Invariant:\n\n### 最小完了条件\n\n- a\n\n### 失敗定義\n\n- b\n\n### 非目的\n\n- c\n\n'
        printf '## Scope\n\n- scope\n\n## Acceptance Criteria\n\n- `bash scripts/doc-consistency-check.sh` exit 0\n\n'
        printf '## Design Readiness\n\n- ready\n\n'
        printf '## Contract Ledger\n\n| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |\n|---|---|---|---|---|\n| C1 | x | Scope | `bash` fixture | 非対象 |\n\n'
        printf '## Test Plan\n\n| Contract | Failure Mode | Test Type | Test Name | Would fail if... |\n|---|---|---|---|---|\n| C1 | f | unit | t | x |\n\n'
        if [ "$risk" = R4 ]; then
            printf '## Recovery\n\n- restore\n\n## Data Safety\n\n- data\n\n'
        fi
        printf '%s' "${NEW_EXTRA:-}"
    } > "$path"
}
NEW_PKT="docs/plans/2026-03-10-new-form.md"
new_case() { # $1 label, $2 expect ok|reject, $3 expected text (reject)
    if [ "$2" = ok ]; then
        if ! run_check "$NEW_PKT"; then cat "$out" >&2; fail "$1: rejected"; fi
        assert_contains "$out" "PK1: Plan Packet presence OK"
        assert_contains "$out" "PK4: Workflow State machine 整合 OK"
    else
        if run_check "$NEW_PKT"; then fail "$1: accepted"; fi
        assert_contains "$out" "$3"
    fi
    echo "PASS: $1"
}
# PK1N: 新形式の最小形。R3 は Data Safety なしで通り、Contract Ledger・Test Plan は必須。R4 は Recovery・Data Safety。
setup_repo_dirs; reset_packet_defaults; PKT_BASE="$BASE_NEW"; NEW_EXTRA=""
write_new_packet "$repo/$NEW_PKT" R3
new_case "PK1N R3 minimal (no Data Safety, no Owner Effort Budget, no Matrix link)" ok
for section in "Contract Ledger" "Test Plan" "Design Readiness" "Acceptance Criteria"; do
    write_new_packet "$repo/$NEW_PKT" R3
    sed -i "s/^## ${section}\$/### ${section}/" "$repo/$NEW_PKT"
    new_case "PK1N R3 without $section" reject "PK1: $NEW_PKT (R3, 新形式) は必須セクション '## ${section}' を欠いています"
done
write_new_packet "$repo/$NEW_PKT" R4
new_case "PK1N R4 with Recovery and Data Safety" ok
for section in Recovery "Data Safety"; do
    write_new_packet "$repo/$NEW_PKT" R4
    sed -i "s/^## ${section}\$/### ${section}/" "$repo/$NEW_PKT"
    new_case "PK1N R4 without $section" reject "PK1: $NEW_PKT (R4, 新形式) は必須セクション '## ${section}' を欠いています"
done
NEW_EXTRA=$'## L3\n\n- 9 pt 以上で読める\n'; write_new_packet "$repo/$NEW_PKT" R3
new_case "PK1N R3 with L3" ok
NEW_EXTRA=$'## L3\n\n<!-- 書く -->\n'; write_new_packet "$repo/$NEW_PKT" R3
new_case "PK1N empty L3" reject "の '## L3' が空です"
NEW_EXTRA=""
# PH7: 形式は packet を最初に足した commit の親で決まる。新形式の Phase の行・R2、旧形式の Phase 欠けは PK4 が拒む。
write_new_packet "$repo/$NEW_PKT" R3
sed -i 's/^- Risk: R3$/- Phase: implementing\n- Risk: R3/' "$repo/$NEW_PKT"
new_case "PH7 new form with a Phase line" reject "must not have a Phase line"
mv "$repo/$NEW_PKT" "$repo/docs/plans/2020-01-01-backdated.md"
if run_check "docs/plans/2020-01-01-backdated.md"; then fail "PH7: a back-dated file name escaped the new form"; fi
assert_contains "$out" "must not have a Phase line"
rm -f "$repo/docs/plans/2020-01-01-backdated.md"
write_new_packet "$repo/$NEW_PKT" R2
new_case "PH7 new form R2" reject "R3 or R4"
setup_repo_dirs; reset_packet_defaults
write_new_packet "$repo/$NEW_PKT" R3
new_case "PH7 old form without a Phase line" reject "needs its Phase line"
# PH7 正例 / PH10: merge の前に足した旧形式の packet は main の取込みの後も旧形式。rename すると新形式で拒まれる。
setup_repo_dirs; reset_packet_defaults
write_packet "$repo/docs/plans/2026-03-11-old-lane.md"
lane_commit
git -C "$repo" merge -q --no-ff --no-edit "$BASE_NEW"
run_after_merge() {
    (cd "$repo" && WORKFLOW_BASE_SHA="$BASE_NEW" PATH="$rg_shim_dir:$PATH" REAL_RG="$real_rg" \
        bash doc-consistency-check.sh --target plan "$1") > "$out" 2>&1
}
if ! run_after_merge docs/plans/2026-03-11-old-lane.md; then cat "$out" >&2; fail "PH7: old lane packet rejected after taking main"; fi
assert_contains "$out" "PK4: Workflow State machine 整合 OK"
git -C "$repo" mv docs/plans/2026-03-11-old-lane.md docs/plans/2026-03-12-renamed.md
git -C "$repo" commit -qm "rename the packet"
if run_after_merge docs/plans/2026-03-12-renamed.md; then fail "PH10: renamed old packet escaped the form check"; fi
assert_contains "$out" "never renamed or re-cut"
rm -rf "$repo/scripts/plan-approve.sh"
echo "PASS: PH7 / PH10"

# AR2: 引数なしの対象は active な packet だけ。起点の tree にある壊れた packet は見ない。起点が無ければ ERROR。
setup_repo_dirs; reset_packet_defaults
printf '# merged, broken\n\nno Risk line\n' > "$repo/docs/plans/2026-01-01-merged.md"
printf '# Merged\n\n[gone](missing-file.md)\n' > "$repo/docs/plans/test-matrices-merged.md"
lane_commit
PKT_BASE="$(git -C "$repo" rev-parse HEAD)"
write_packet "$repo/docs/plans/2026-03-13-active.md"
if ! run_check ""; then cat "$out" >&2; fail "AR2: a merged packet outside the active set was checked"; fi
assert_not_contains "$out" "2026-01-01-merged.md"
# AR6: merge 済みの file の link 切れは WARN、active な packet の link 切れは ERROR。
assert_contains "$out" "merge 済みの packet のリンク先 'missing-file.md' が存在しません"
printf '\n[gone](also-missing.md)\n' >> "$repo/docs/plans/2026-03-13-active.md"
if run_check ""; then fail "AR6: a broken link in the active packet passed"; fi
assert_contains "$out" "リンク先 'also-missing.md' が存在しません"
assert_not_contains "$out" "merge 済みの packet のリンク先 'also-missing.md'"
# AR2 / PH6: 起点が取れなければ ERROR（全 packet の検査に落とさない）。
lane_commit
printf '\n' >> "$repo/docs/plans/2026-03-13-active.md"
sed -i '/also-missing.md/d' "$repo/docs/plans/2026-03-13-active.md"
lane_commit
if (cd "$repo" && env -u WORKFLOW_BASE_SHA bash doc-consistency-check.sh --target plan docs/plans/2026-03-13-active.md) > "$out" 2>&1; then
    fail "AR2: unresolved origin passed (the scope ERROR must reach the exit code)"
fi
assert_contains "$out" "[ERROR] PK: active な packet を決められない"
echo "PASS: AR2 / AR6"

# FL1〜FL3: 起点からの '+' 行に新しい path:行 を足すと ERROR。既存の行・適用除外は ERROR にしない。
setup_repo_dirs; reset_packet_defaults
mkdir -p "$repo/docs/archive" "$repo/docs/research"
printf '# Live\n\nold reference foo.md:12\n' > "$repo/docs/live.md"
printf '# Backlog\n' > "$repo/docs/backlog.md"
lane_commit
PKT_BASE="$(git -C "$repo" rev-parse HEAD)"
write_packet "$repo/docs/plans/2026-03-14-fl.md"
printf 'see `src-tauri/src/x.rs:3`\n' >> "$repo/docs/archive/old.md"
printf 'see src/a.ts:9\n' > "$repo/docs/research/2026-01-01-study.md"
printf 'see scripts/a.sh:9\n' > "$repo/docs/2026-03-01-l3-record.md"
printf 'see AGENTS.md:4 in the packet\n' >> "$repo/docs/plans/2026-03-14-fl.md"
printf -- '- decision cites x.py:1\n' > "$repo/docs/decision-log.md"
printf -- '- 時点の記録 docs/a.md:3（abc1234 の時点）\n' >> "$repo/docs/backlog.md"
if ! run_check "docs/plans/2026-03-14-fl.md"; then cat "$out" >&2; fail "FL3: an exempt or existing path:line was an ERROR"; fi
assert_contains "$out" "FL: 新しい path:行 の引用なし"
printf 'new reference bar.rs:3\n' >> "$repo/docs/live.md"
printf -- '- 時点の無い docs/b.md:4\n' >> "$repo/docs/backlog.md"
if run_check "docs/plans/2026-03-14-fl.md"; then fail "FL1: a new path:line in a live doc passed"; fi
assert_contains "$out" "FL: 生きた docs に path:行 の引用を新しく足した"
assert_contains "$out" "docs/live.md: bar.rs:3"
assert_contains "$out" "docs/backlog.md: docs/b.md:4"
assert_not_contains "$out" "foo.md:12"
# FL2: 起点が取れなければ WARN（既存の引用を一斉に ERROR にしない）。dated packet が無ければ PK は止めない。
setup_repo_dirs
lane_commit
if ! (cd "$repo" && env -u WORKFLOW_BASE_SHA bash doc-consistency-check.sh --target plan docs/live.md) > "$out" 2>&1; then
    cat "$out" >&2; fail "FL2: missing origin was an ERROR"
fi
assert_contains "$out" "FL: 起点が取れないので新しい path:行 の検査をしない"
echo "PASS: FL1 / FL2 / FL3"
