#!/usr/bin/env bash
# scripts/check-workflow-git.sh
# ============================================================================
# 計画の固定（PK5）と、計画の承認の前の実装（先走り）の git 検査
# docs/DEV_WORKFLOW.md 「計画の固定」/ D-039・D-145 参照
#
# 呼び出し元: scripts/pre-push.sh（push 前 gate）/ scripts/local-ci.sh（L1 gate）
# hosted docs jobもfetch-depth: 0で実PR HEADを検査する（MG-D4）。
#
# 対象は active な packet だけ（D-145 (5)。定義は scripts/pr-gate.py の plan_origin・active_packets の 1 か所）:
#   docs/plans/ 直下の dated packet のうち、起点の tree に無いもの。起点は WORKFLOW_BASE_SHA と HEAD の
#   merge-base。WORKFLOW_BASE_SHA が空か HEAD なら origin/main と HEAD の merge-base。それも HEAD なら 0 件。
#   起点を解決できない・merge-base が一意でないときは ERROR（全 packet の検査に落とさない）。
#
# 検査内容:
#   active な packet:
#     - `Evidence Mode` 行は任意。あれば github だけを受理する（Phase の値は評価しない）
#     - PK5 (a)〜(d): Plan Commit・Amendments の ancestry と不変
#     - 先走り（H2）: `Plan Commit: pending` の packet があり、起点からの差分に classifier の
#       rust・frontend・generated・env・unknown か scripts/**・.github/** があれば ERROR
#   merge 済みの packet（起点の tree にある packet）: Workflow State の節を変えたら ERROR（archive への移動は受ける）
#   検査対象の履歴が shallow なら ERROR（full history が必要）。
# ============================================================================

set -u  # 各 check を独立 FAIL=1 集約方式のため set -e は使わない

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

FAIL=0
PLAN_DIR="docs/plans"
# ----------------------------------------------------------------------------
# PK5: 単一 packet ファイルの Plan Commit / Amendments ancestry 検査
# ----------------------------------------------------------------------------
check_plan_commit_ancestry() {
    local file="$1"
    local plan_commit plan_commit_full amendments amendment amendment_full first_value
    local -a amendment_shas=()

    plan_commit="$(grep -m1 -E '^- Plan Commit:[[:space:]]*' "$file" 2>/dev/null \
        | sed -E 's/^- Plan Commit:[[:space:]]*//; s/[[:space:]]+$//')"

    # Plan Commit が pending（未確定）の packet は PK5 の対象外（plan-draft/plan-gate フェーズ）
    if [[ -z "$plan_commit" || "$plan_commit" == "pending" ]]; then
        return 0
    fi

    # D-107 (2): 現在の値は helper と同じ 40 桁の小文字 hex だけ。書式の ERROR でも return せず、
    # 書き換えの検出と Amendments の照合（MG-D5 の prefix を含む）は続ける。
    local sha40='^[0-9a-f]{40}$'
    plan_commit_full=""
    if [[ ! "$plan_commit" =~ $sha40 ]]; then
        echo "❌ [workflow-git] PK5: $file の Plan Commit '$plan_commit' は 40 桁の小文字 hex の full SHA ではありません"
        FAIL=1
    elif ! plan_commit_full="$(git rev-parse --verify "${plan_commit}^{commit}" 2>/dev/null)"; then
        echo "❌ [workflow-git] PK5: $file の Plan Commit '$plan_commit' は解決できない SHA です"
        FAIL=1
        return 0
    elif ! git merge-base --is-ancestor "$plan_commit_full" HEAD 2>/dev/null; then
        echo "❌ [workflow-git] PK5: $file の Plan Commit '$plan_commit' は現在の HEAD の祖先ではありません"
        FAIL=1
    fi

    amendments="$(grep -m1 -E '^- Amendments:[[:space:]]*' "$file" 2>/dev/null \
        | sed -E 's/^- Amendments:[[:space:]]*//; s/[[:space:]]+$//')"
    if [[ -n "$amendments" && "$amendments" != "none" ]]; then
        local -A seen_amendments=()
        # 区切りはカンマと ASCII の空白・タブだけ（helper の [,\s]+ より厳しい側、D-107 (2)）。
        while IFS= read -r amendment; do
            [[ -z "$amendment" ]] && continue
            amendment_shas+=("$amendment")
            if [[ ! "$amendment" =~ $sha40 ]]; then
                echo "❌ [workflow-git] PK5: $file の Amendments SHA '$amendment' は 40 桁の小文字 hex の full SHA ではありません"
                FAIL=1
                continue
            fi
            if [[ -n "${seen_amendments[$amendment]:-}" ]]; then
                echo "❌ [workflow-git] PK5: $file の Amendments に重複する SHA '$amendment' があります"
                FAIL=1
                continue
            fi
            seen_amendments["$amendment"]=1
            if ! amendment_full="$(git rev-parse --verify "${amendment}^{commit}" 2>/dev/null)"; then
                echo "❌ [workflow-git] PK5: $file の Amendments SHA '$amendment' は解決できません"
                FAIL=1
                continue
            fi
            if [[ -n "$plan_commit_full" ]] && ! git merge-base --is-ancestor "$plan_commit_full" "$amendment_full" 2>/dev/null; then
                echo "❌ [workflow-git] PK5: $file の Amendments SHA '$amendment' は Plan Commit '$plan_commit' の descendant ではありません"
                FAIL=1
            fi
            if ! git merge-base --is-ancestor "$amendment_full" HEAD 2>/dev/null; then
                echo "❌ [workflow-git] PK5: $file の Amendments SHA '$amendment' は現在の HEAD の祖先ではありません"
                FAIL=1
            fi
        done < <(printf '%s\n' "$amendments" | tr ', \t' '\n\n\n')
    fi

    # Plan Commit 書き換え検出: ファイル履歴の全 diff から追加された
    # "- Plan Commit: <value>" 行を新しい commit 順に集め、pending を除外した上で
    # 最後（= 最も古い non-pending 値 = 初回確定値）を現在値と比較する。
    first_value="$(git log --follow -p -- "$file" 2>/dev/null \
        | grep -E '^[+]- Plan Commit:[[:space:]]*' \
        | sed -E 's/^[+]- Plan Commit:[[:space:]]*//; s/[[:space:]]+$//' \
        | grep -v -E '^pending$' \
        | tail -1)"

    if [[ -n "$first_value" && "$first_value" != "$plan_commit" ]]; then
        echo "❌ [workflow-git] PK5: $file の Plan Commit が書き換えられています（初回確定値 '$first_value' -> 現在値 '$plan_commit'）"
        FAIL=1
    fi
    # MG-D5: each registered sequence must remain a prefix, including original SHA spelling.
    # Separators/whitespace may change; resolving aliases here would weaken SHA immutability.
    local prior_amendments index
    local -a prior_shas=()
    while IFS= read -r prior_amendments; do
        mapfile -t prior_shas < <(printf '%s\n' "$prior_amendments" | grep -oE '[0-9a-f]{7,40}' || true)
        for ((index = 0; index < ${#prior_shas[@]}; index++)); do
            if [[ "${amendment_shas[$index]:-}" != "${prior_shas[$index]}" ]]; then
                echo "❌ [workflow-git] PK5: $file の Amendments が削除・変更されています（登録順序を含む prefix が必要です）"
                FAIL=1
                break
            fi
        done
    done < <(git log --follow -p -- "$file" | grep -E '^[+]- Amendments:')

}

workflow_section() {
    awk 'active && /^## / { exit } /^## Workflow State[[:space:]]*$/ { active=1; next } active { print }'
}

main() {
    local file
    # Only the target ancestry matters: an unrelated ref can retain a shallow marker.
    # rev-list treats a shallow boundary as a root; the raw commit still names its parents.
    local history_root history_roots
    history_roots="$(git rev-list --max-parents=0 HEAD ${WORKFLOW_BASE_SHA:+"$WORKFLOW_BASE_SHA"})" || exit 1
    while IFS= read -r history_root; do
        if git cat-file -p "$history_root" | sed -n '/^$/q; /^parent /p' | grep -q .; then
            echo "❌ [workflow-git] PK5: full history required; shallow target ancestry is not evidence"
            exit 1
        fi
    done <<< "$history_roots"

    # D-145 (5): the single active definition lives in scripts/pr-gate.py (helper, PK1, PK4, PK5, H2).
    local scope kind value origin=""
    local -a active=()
    scope="$(PYTHONDONTWRITEBYTECODE=1 python3 - "$SCRIPT_DIR/pr-gate.py" "${WORKFLOW_BASE_SHA:-}" <<'PY'
import importlib.util, os, sys
spec = importlib.util.spec_from_file_location('pr_gate', sys.argv[1])
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
names = [f'docs/plans/{n}' for n in sorted(os.listdir('docs/plans'))] if os.path.isdir('docs/plans') else []
# \x1f: a non-whitespace separator, so an empty field never shifts the next one into its place.
try:
    origin = m.plan_origin(sys.argv[2] or None)
    print('origin\x1f' + (origin or ''))
    for path in m.active_packets(origin, names):
        try:
            state, problem = m.approval_state(path, open(path, encoding='utf-8').read())
        except m.GateError as error:
            state, problem = 'approved', f'{path}: Workflow State を読めない（{error}）'
        print(f'active\x1f{path}\x1f{state}\x1f{problem or ""}')
except m.GateError as error:
    print(f'error\x1f{error}')
PY
)" || scope=$'error\x1fpython3 と scripts/pr-gate.py が必要'
    local state problem
    local -a pending=() drifts=()
    while IFS=$'\x1f' read -r kind value state problem; do
        case "$kind" in
            origin) origin="$value" ;;
            active)
                active+=("$value")
                [[ "$state" == pending ]] && pending+=("$value")
                [[ -n "$problem" ]] && drifts+=("$problem") ;;
            error)
                echo "❌ [workflow-git] active な packet を決められない: $value"
                exit 1 ;;
        esac
    done <<< "$scope"

    local -A is_active=()
    for file in "${active[@]}"; do
        is_active["$file"]=1
    done

    # AR5: a merged packet may get typo and link fixes, never a rewritten Workflow State (the merge-time record).
    if [[ -n "$origin" ]]; then
        while IFS= read -r file; do
            [[ -n "$file" && -z "${is_active[$file]:-}" ]] || continue
            [[ "$(basename "$file")" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}- ]] || continue
            git cat-file -e "$origin:$file" 2>/dev/null || continue
            if [[ "$(git show "$origin:$file" | workflow_section)" != "$(workflow_section < "$file")" ]]; then
                echo "❌ [workflow-git] merge 済みの packet の Workflow State は書き換えない（merge の時点の記録）: $file"
                FAIL=1
            fi
        done < <(find "$PLAN_DIR" -maxdepth 1 -name '*.md' -type f 2>/dev/null | sort)
    fi

    # Final review C1: the plan sections after approval equal the latest approved snapshot (or a registered --amend).
    for problem in "${drifts[@]}"; do
        echo "❌ [workflow-git] $problem"
        FAIL=1
    done

    for file in "${active[@]}"; do
        # Evidence Mode は最初の `## Workflow State` の h2 の節だけから読む（helper と同じ範囲、D-102 D4）。
        local ws_section
        ws_section="$(workflow_section < "$file")"
        if grep -qE '^- Evidence Mode:' <<<"$ws_section" &&
            [[ "$(sed -n 's/^- Evidence Mode: *//p' <<<"$ws_section" | sed -E 's/[[:space:]]+$//')" != "github" ]]; then
            echo "❌ [workflow-git] Evidence Mode は廃止。書くなら github: $file"
            FAIL=1
        fi
        check_plan_commit_ancestry "$file"
    done

    # H2 / D-145 (3): before the plan is fixed, the branch carries no implementation.
    if [[ "${#pending[@]}" -gt 0 ]]; then
        local paths flags code
        if ! paths="$(git diff --name-only --no-renames "$origin" HEAD --)" ||
            ! flags="$(printf '%s\n' "$paths" | bash "$SCRIPT_DIR/ci/classify-changes.sh" --files-from-stdin 2>/dev/null)"; then
            echo "❌ [workflow-git] 先走りの検査: 起点 $origin からの差分を分類できない"
            exit 1
        fi
        code="$(printf '%s\n' "$flags" | grep -E '^(rust|frontend|generated|env|unknown)=true$' | cut -d= -f1 | paste -sd, -)"
        local tooling
        tooling="$(printf '%s\n' "$paths" | grep -E '^(scripts|\.github)/' | paste -sd' ' - || true)"
        if [[ -n "$code" || -n "$tooling" ]]; then
            echo "❌ [workflow-git] 計画の承認の前の実装: ${pending[*]} の Plan Commit が pending のまま、起点 $origin からの差分に実装がある（${code:+classifier: $code}${code:+${tooling:+; }}${tooling:+path: $tooling}）。Plan Review の後に scripts/plan-approve.sh で Plan Commit を固定してから実装を push する"
            FAIL=1
        fi
    fi

    if [[ "$FAIL" -eq 0 ]]; then
        echo "✅ [workflow-git] PK5・先走りの検査 OK（active な packet ${#active[@]} 件）"
    elif [[ -z "${WORKFLOW_BASE_SHA:-}" ]]; then
        echo "ℹ️  [workflow-git] 起点は origin/main と HEAD の merge-base。origin/main が古ければ git fetch origin してから回し直す"
    fi

    exit "$FAIL"
}

main
