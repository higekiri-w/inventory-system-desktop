#!/usr/bin/env bash
# scripts/tests/workflow-git-checks.test.sh
# scripts/check-workflow-git.sh（PK5）の synthetic git fixture repo テスト。
# 各シナリオは tmpdir に git init した使い捨て repo を構築し、正例/負例を判定する。
# 実 SHA（PR #165 等）は dangling で automated fixture には使えないため（Plan Gate R1）、
# ここでは全て test 自身が構築する commit 列のみを使う。
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$SOURCE_ROOT/scripts/check-workflow-git.sh"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local msg="$3"
    if ! printf '%s' "$haystack" | grep -Fq -- "$needle"; then
        fail "$msg (期待した文字列が出力に含まれない: $needle)"
    fi
}

assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local msg="$3"
    if printf '%s' "$haystack" | grep -Fq -- "$needle"; then
        fail "$msg (含まれてはいけない文字列が出力に含まれる: $needle)"
    fi
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

init_repo() {
    local repo="$1"
    mkdir -p "$repo"
    git -C "$repo" init -q -b main
    git -C "$repo" config user.name test
    git -C "$repo" config user.email test@example.invalid
}

commit_all() {
    local repo="$1"
    local subject="$2"
    git -C "$repo" add -A
    git -C "$repo" commit -q -m "$subject"
    git -C "$repo" rev-parse HEAD
}

write_packet() {
    local repo="$1"
    local packet_name="$2"
    local plan_commit="$3"
    local amendments="$4"
    local phase="implementing"
    [[ "$plan_commit" == "pending" ]] && phase="plan-gate"
    mkdir -p "$repo/docs/plans"
    cat > "$repo/docs/plans/$packet_name" <<EOF
# Test Packet

## Workflow State

- Phase: ${phase}
- Plan Commit: ${plan_commit}
- Amendments: ${amendments}
EOF
}

append_rebase_map() {
    local repo="$1"
    local packet_name="$2"
    local old_sha="$3"
    local new_sha="$4"
    printf '\nRebase Map: %s -> %s\n' "$old_sha" "$new_sha" >> "$repo/docs/plans/$packet_name"
}

run_check() {
    local repo="$1"
    (cd "$repo" && bash "$CHECK_SCRIPT" 2>&1)
}

# D-145 (5): active packets are measured from origin/main; the fixtures' main starts at "base".
base_commit() {
    commit_all "$1" "base" > /dev/null
    git -C "$1" update-ref refs/remotes/origin/main HEAD
}

# 旧 state-only 系 subject の commit を作る（T-G5 / T-G6 用）。差分がないと
# `git commit` が失敗するため、ダミーのログファイルへの追記を伴わせる。
state_only_commit() {
    local repo="$1"
    local subject="$2"
    printf '%s\n' "$subject" >> "$repo/.state-log"
    commit_all "$repo" "$subject" > /dev/null
}

# set -e 環境下で non-zero 終了を安全に捕捉するためのラッパー。
# 呼び出し後、変数 CHECK_STATUS に終了コードが入る。
CHECK_STATUS=0
capture_check() {
    local repo="$1"
    local -n __out_ref="$2"
    set +e
    __out_ref="$(run_check "$repo")"
    CHECK_STATUS=$?
    set -e
}

# ============================================================================
# PK5: ancestry 正例（plan-first が実装 commit の祖先）
# ============================================================================
repo="$tmp/pk5-ancestry-ok"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"

write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"

write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める" > /dev/null

printf 'impl\n' > "$repo/impl.txt"
commit_all "$repo" "feat: implement" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "ancestry 正例が ERROR 判定された:\n$output"
assert_not_contains "$output" "PK5:" "ancestry 正例で PK5 出力が発生した"

# ============================================================================
# PK5: squash 相当の負例（squash merge 後は ancestor でない）
# ============================================================================
repo="$tmp/pk5-squash-negative"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"
git -C "$repo" branch feature

git -C "$repo" switch -q feature
write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める" > /dev/null
printf 'impl\n' > "$repo/impl.txt"
commit_all "$repo" "feat: implement" > /dev/null

git -C "$repo" switch -q main
git -C "$repo" merge -q --squash feature > /dev/null
commit_all "$repo" "feat: implement (squashed)" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "squash 後の非 ancestor が ERROR 判定されなかった"
assert_contains "$output" "は現在の HEAD の祖先ではありません" "squash 負例で ancestry ERROR が出力されない"

# ============================================================================
# PK5: Plan Commit 書き換え検出（ancestry は成立するが原本改変）
# ============================================================================
repo="$tmp/pk5-rewrite"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"

write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"

write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
b_sha="$(commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める")"

printf 'impl\n' > "$repo/impl.txt"
commit_all "$repo" "feat: implement" > /dev/null

# 不正な書き換え: original を b_sha に差し替える（b_sha 自体は HEAD の祖先なので
# ancestry 検査だけでは検出できず、rewrite 検出が唯一の網であることを確認する）
write_packet "$repo" "2026-01-01-packet.md" "$b_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を修正" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "Plan Commit 書き換えが ERROR 判定されなかった"
assert_contains "$output" "書き換えられています" "書き換え検出 ERROR が出力されない"
assert_not_contains "$output" "は現在の HEAD の祖先ではありません" "書き換えテストで無関係な ancestry ERROR も発生した（テスト設計の分離が崩れている）"

# ============================================================================
# PK5: Amendments 追記型の正例（original 不変 + Amendments 追記）
# ============================================================================
repo="$tmp/pk5-amendments-ok"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"

write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"

write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める" > /dev/null

printf 'impl\n' > "$repo/impl.txt"
c_sha="$(commit_all "$repo" "feat: implement")"

write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "$c_sha"
commit_all "$repo" "docs(plans): gated amendment を記録" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "Amendments 追記正例が ERROR 判定された:\n$output"
assert_not_contains "$output" "PK5:" "Amendments 正例で PK5 出力が発生した"

# MG-D5 / F1: two amendments retain their order; only token separators may differ.
printf 'second amendment\n' >> "$repo/impl.txt"
d_sha="$(commit_all "$repo" 'feat: second amendment')"
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "$c_sha, $d_sha"
commit_all "$repo" 'docs(plans): append second amendment' > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "ordered amendment append rejected: $output"
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "  $c_sha  ,   $d_sha  "
valid_amendments_head="$(commit_all "$repo" 'docs(plans): amendment separators only')"
capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "amendment formatting change rejected: $output"

# Each invalid candidate branches from the same valid history; no destructive reset is needed.
for variant in reorder replacement removal spelling; do
    git -C "$repo" switch -qc "amendment-$variant" "$valid_amendments_head"
    case "$variant" in
        reorder) changed="$d_sha, $c_sha" ;;
        replacement) changed="$a_sha, $d_sha" ;;
        removal) changed="$c_sha" ;;
        spelling) changed="${c_sha:0:12}, $d_sha" ;;
    esac
    write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "$changed"
    commit_all "$repo" "docs(plans): invalid amendment $variant" > /dev/null
    capture_check "$repo" output
    [[ "$CHECK_STATUS" -ne 0 ]] || fail "amendment $variant was accepted"
    assert_contains "$output" "Amendments が削除・変更されています" "amendment $variant reason missing"
    assert_not_contains "$output" "祖先ではありません" "amendment $variant failed for unrelated ancestry"
done
git -C "$repo" switch -q --detach "$valid_amendments_head"

# MG-D5 / S-P3-1: recorded amendment removal fails even though every remaining SHA is ancestral.
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
commit_all "$repo" "docs(plans): remove recorded amendment" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "registered amendment removal was accepted"
assert_contains "$output" "Amendments が削除・変更されています" "amendment removal reason missing"
assert_not_contains "$output" "祖先ではありません" "amendment removal failed for unrelated ancestry"

# ============================================================================
# D-107 (2) PK5-SHA40: 現在の Plan Commit・Amendments は 40 桁の小文字 hex だけを受理し、重複を拒む
# ============================================================================
SHA40_MSG="は 40 桁の小文字 hex の full SHA ではありません"
sha40_repo() {
    repo="$tmp/pk5-sha40-$1"
    init_repo "$repo"
    printf 'base\n' > "$repo/README.md"
    base_commit "$repo"
    write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
    a_sha="$(commit_all "$repo" "docs(plans): plan-first")"
    write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
    commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める" > /dev/null
    printf 'impl\n' > "$repo/impl.txt"
    c_sha="$(commit_all "$repo" "feat: implement")"
}

# T3-1
sha40_repo short
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "${c_sha:0:8}"
commit_all "$repo" "docs(plans): short amendment" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "PK5-SHA40: short Amendments accepted: $output"
assert_contains "$output" "Amendments SHA '${c_sha:0:8}' $SHA40_MSG" "PK5-SHA40: short Amendments reason missing"
echo "PASS: PK5-SHA40: short Amendments"

# T3-2: 41 桁・大文字・hex 以外の token
sha40_repo malformed
sha40_valid_head="$(git -C "$repo" rev-parse HEAD)"
for token in "${c_sha}0" "${c_sha^^}" "abc1234"; do
    git -C "$repo" switch -qc "malformed-${token:0:3}-${#token}" "$sha40_valid_head"
    write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "${token},"
    commit_all "$repo" "docs(plans): malformed amendment" > /dev/null
    capture_check "$repo" output
    [[ "$CHECK_STATUS" -ne 0 ]] || fail "PK5-SHA40: malformed Amendments token '$token' accepted: $output"
    assert_contains "$output" "Amendments SHA '$token' $SHA40_MSG" "PK5-SHA40: malformed token '$token' reason missing"
done
echo "PASS: PK5-SHA40: malformed Amendments token"

# T3-3
sha40_repo duplicate
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "$c_sha, $c_sha"
commit_all "$repo" "docs(plans): duplicate amendment" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "PK5-SHA40: duplicate Amendments accepted: $output"
assert_contains "$output" "Amendments に重複する SHA '$c_sha' があります" "PK5-SHA40: duplicate reason missing"
echo "PASS: PK5-SHA40: duplicate Amendments"

# T3-4: 初回の確定値を短縮で commit した Plan Commit
repo="$tmp/pk5-sha40-short-plan"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"
write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"
write_packet "$repo" "2026-01-01-packet.md" "${a_sha:0:8}" "none"
commit_all "$repo" "docs(plans): short Plan Commit" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "PK5-SHA40: short Plan Commit accepted: $output"
assert_contains "$output" "Plan Commit '${a_sha:0:8}' $SHA40_MSG" "PK5-SHA40: short Plan Commit reason missing"
# 書式の ERROR の後も Amendments の照合へ進む（set -u の下で落ちない）
printf 'impl\n' > "$repo/impl.txt"
c_sha="$(commit_all "$repo" "feat: implement")"
write_packet "$repo" "2026-01-01-packet.md" "${a_sha:0:8}" "$c_sha"
commit_all "$repo" "docs(plans): amendment after short Plan Commit" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "PK5-SHA40: short Plan Commit with an amendment accepted: $output"
assert_contains "$output" "Plan Commit '${a_sha:0:8}' $SHA40_MSG" "PK5-SHA40: short Plan Commit with an amendment reason missing"
assert_not_contains "$output" "unbound variable" "PK5-SHA40: amendment check aborted after a short Plan Commit"
echo "PASS: PK5-SHA40: short Plan Commit"

# ============================================================================
# T-G2: Rebase Map 行は解釈しない。rebase で非 ancestor になった Plan Commit は、
# patch-id 同値の Rebase Map 行を足しても ERROR のまま（escape hatch にしない）
# ============================================================================
repo="$tmp/pk5-rebase-map-ignored"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"
git -C "$repo" branch feature

git -C "$repo" switch -q feature
write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
old_plan_sha="$(commit_all "$repo" "docs(plans): plan-first")"
write_packet "$repo" "2026-01-01-packet.md" "$old_plan_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める" > /dev/null
printf 'impl\n' > "$repo/impl.txt"
commit_all "$repo" "feat: implement" > /dev/null

git -C "$repo" switch -q main
printf 'main advance\n' > "$repo/main.txt"
commit_all "$repo" "chore: advance main" > /dev/null
git -C "$repo" switch -q feature
git -C "$repo" rebase main > /dev/null
new_plan_sha="$(git -C "$repo" log --format=%H --grep='^docs(plans): plan-first$' -1)"
old_patch_id="$(git -C "$repo" show --pretty=format: --binary "$old_plan_sha" | git patch-id --stable | awk '{print $1}')"
new_patch_id="$(git -C "$repo" show --pretty=format: --binary "$new_plan_sha" | git patch-id --stable | awk '{print $1}')"
[[ -n "$old_patch_id" && "$old_patch_id" == "$new_patch_id" ]] || fail "T-G2 fixture: rebase 前後の plan-first が patch-id 同値でない"

append_rebase_map "$repo" "2026-01-01-packet.md" "$old_plan_sha" "$new_plan_sha"
commit_all "$repo" "docs(plans): rebase map を記録" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "Rebase Map 行で非 ancestor の Plan Commit が受理された:\n$output"
assert_contains "$output" "は現在の HEAD の祖先ではありません" "Rebase Map 行ありの非 ancestor で ancestry ERROR が出力されない"

# ============================================================================
# PK5: Amendments 非 descendant の負例（並行ブランチの SHA を記録）
# ============================================================================
repo="$tmp/pk5-amendments-non-descendant"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"

git -C "$repo" branch plan-branch
git -C "$repo" branch unrelated-branch

git -C "$repo" switch -q plan-branch
write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める" > /dev/null

git -C "$repo" switch -q unrelated-branch
printf 'unrelated\n' > "$repo/unrelated.txt"
u_sha="$(commit_all "$repo" "chore: unrelated parallel work")"

git -C "$repo" switch -q main
git -C "$repo" merge -q --no-edit plan-branch > /dev/null
git -C "$repo" merge -q --no-edit unrelated-branch > /dev/null

# unrelated-branch の U は main の祖先だが、plan-branch の A の子孫ではない
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "$u_sha"
commit_all "$repo" "docs(plans): gated amendment を記録" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "非 descendant の Amendments が ERROR 判定されなかった"
assert_contains "$output" "の descendant ではありません" "非 descendant ERROR が出力されない"
assert_not_contains "$output" "は現在の HEAD の祖先ではありません" "非 descendant テストで無関係な ancestor-of-HEAD ERROR も発生した"

# ============================================================================
# PK5: Amendments 非 ancestor-of-HEAD の負例（Plan Commit の子だが側 branch にある SHA）
# ============================================================================
repo="$tmp/pk5-amendments-not-in-head"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"

write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"

git -C "$repo" switch -qc side-branch
printf 'side\n' > "$repo/side.txt"
s_sha="$(commit_all "$repo" "feat: side amendment")"
git -C "$repo" switch -q main

write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "$s_sha"
commit_all "$repo" "docs(plans): gated amendment を記録" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "HEAD の祖先でない Amendments が ERROR 判定されなかった"
assert_contains "$output" "は現在の HEAD の祖先ではありません" "非 ancestor-of-HEAD ERROR が出力されない"
assert_not_contains "$output" "の descendant ではありません" "非 ancestor-of-HEAD テストで無関係な descendant ERROR も発生した"

# ============================================================================
# PK5: pending は skip（ERROR/WARN いずれも出さない）
# ============================================================================
repo="$tmp/pk5-pending-skip"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"
write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
commit_all "$repo" "docs(plans): plan-draft" > /dev/null

capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "pending packet で誤って ERROR になった:\n$output"
assert_not_contains "$output" "PK5:" "pending packet で PK5 出力が発生した（skip されていない）"

# ============================================================================
# T-G5: active packet ありで旧 state-only / state-backtrack subject の commit 列を
# 積んでも通る（commit の件数・subject を検査しない）
# ============================================================================
add_retired_subject_commits() {
    local repo="$1"
    state_only_commit "$repo" "docs(plans): state-only遷移 plan-draft->plan-gate"
    state_only_commit "$repo" "docs(plans): state-only遷移 plan-gate->plan-approved"
    state_only_commit "$repo" "docs(plans): state-only遷移 local-verified->independent-review"
    state_only_commit "$repo" "docs(plans): state-only遷移 human-confirm->ready-hosted-final"
    state_only_commit "$repo" "docs(plans): state-backtrack ready-hosted-final->implementing"
    state_only_commit "$repo" "docs(plans): state-backtrack implementing->implementing"
}

repo="$tmp/retired-subjects-with-packet"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_sha="$(commit_all "$repo" "base")"
git -C "$repo" update-ref refs/remotes/origin/main "$base_sha"
write_packet "$repo" "2026-01-01-packet.md" "pending" "none"
a_sha="$(commit_all "$repo" "docs(plans): plan-first")"
write_packet "$repo" "2026-01-01-packet.md" "$a_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を確定して implementing へ進める" > /dev/null
add_retired_subject_commits "$repo"

capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "旧 subject の commit 列で active packet ありの検査が ERROR になった:\n$output"
assert_not_contains "$output" "STATECAP" "active packet ありで STATECAP 出力が発生した"

# ============================================================================
# T-G6: docs/plans/ が無い repo（packet 0 件）でも同じ commit 列で通る。
# 旧実装は packet 0 件で STATECAP を走らせ ERROR になるため、撤去を判別する
# ============================================================================
repo="$tmp/retired-subjects-no-packet"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_sha="$(commit_all "$repo" "base")"
git -C "$repo" update-ref refs/remotes/origin/main "$base_sha"
add_retired_subject_commits "$repo"

capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "packet 0 件で旧 subject の commit 列が ERROR になった:\n$output"
assert_not_contains "$output" "STATECAP" "packet 0 件で STATECAP 出力が発生した"

# ============================================================================
# SPEC-WF-PARALLEL-FRICTION D4: PK5 は起点からの差分が触る packet だけを検査する。
# main に squash 済みの lane X の packet（Plan Commit が祖先でない）が残り、lane Y は自分の packet を足す。
# ============================================================================
repo="$tmp/pk5-diff-scope"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"
git -C "$repo" switch -qc lane-x
write_packet "$repo" "2026-01-01-x.md" "pending" "none"
x_plan="$(commit_all "$repo" "docs(plans): X plan-first")"
write_packet "$repo" "2026-01-01-x.md" "$x_plan" "none"
commit_all "$repo" "docs(plans): X implementing" > /dev/null
git -C "$repo" switch -q main
git -C "$repo" merge -q --squash lane-x > /dev/null
main_sha="$(commit_all "$repo" "feat: X (squashed)")"
git -C "$repo" switch -qc lane-y
write_packet "$repo" "2026-01-02-y.md" "pending" "none"
y_plan="$(commit_all "$repo" "docs(plans): Y plan-first")"
write_packet "$repo" "2026-01-02-y.md" "$y_plan" "none"
y_head="$(commit_all "$repo" "docs(plans): Y implementing")"
x_packet="docs/plans/2026-01-01-x.md"

run_scoped() {
    local base="$1"
    set +e
    output="$(cd "$repo" && env -u WORKFLOW_BASE_SHA ${base:+WORKFLOW_BASE_SHA="$base"} bash "$CHECK_SCRIPT" 2>&1)"
    CHECK_STATUS=$?
    set -e
}

# (a1) hosted PR: WORKFLOW_BASE_SHA = main
run_scoped "$main_sha"
[[ "$CHECK_STATUS" -eq 0 ]] || fail "D4 (a1): merged packet outside the diff was checked: $output"
# (a2) pre-push / local-ci: env unset, origin/main
git -C "$repo" update-ref refs/remotes/origin/main "$main_sha"
run_scoped ""
[[ "$CHECK_STATUS" -eq 0 ]] || fail "D4 (a2): env-unset origin/main route checked the merged packet: $output"
# (a3) PH6 (rewritten from "every packet is checked"): the origin tree cannot be read -> ERROR naming the reason,
# never a fall back to every packet.
fail_tree_bin="$tmp/fail-tree-bin"
mkdir -p "$fail_tree_bin"
printf '#!/usr/bin/env bash\n[[ "${1:-}" == ls-tree ]] && exit 128\nexec %q "$@"\n' "$(command -v git)" > "$fail_tree_bin/git"
chmod +x "$fail_tree_bin/git"
set +e
output="$(cd "$repo" && env -u WORKFLOW_BASE_SHA PATH="$fail_tree_bin:$PATH" bash "$CHECK_SCRIPT" 2>&1)"
CHECK_STATUS=$?
set -e
[[ "$CHECK_STATUS" -ne 0 ]] || fail "D4 (a3): unreadable origin tree was treated as an empty scope: $output"
assert_contains "$output" "active な packet を決められない" "D4 (a3): reason missing"
assert_not_contains "$output" "$x_packet の Plan Commit" "D4 (a3): fell back to every packet"
# (d) PH6 (rewritten): no start point -> ERROR that names it (not every packet).
git -C "$repo" update-ref -d refs/remotes/origin/main
run_scoped ""
[[ "$CHECK_STATUS" -ne 0 ]] || fail "D4 (d): unresolved start point passed"
assert_contains "$output" "起点を解決できない" "D4 (d): reason missing"
assert_not_contains "$output" "$x_packet の Plan Commit" "D4 (d): fell back to every packet"
git -C "$repo" update-ref refs/remotes/origin/main "$main_sha"
# (b) AR5 (rewritten from "a touched merged packet is a PK5 target"): a typo fix in a merged packet passes even
# though its squashed Plan Commit is not ancestral; rewriting its Workflow State is an ERROR.
git -C "$repo" switch -qc lane-y-touches-x "$y_head"
printf '\n## Scope\n\nedited by another lane\n' >> "$repo/$x_packet"
commit_all "$repo" "docs(plans): typo in X" > /dev/null
run_scoped ""
[[ "$CHECK_STATUS" -eq 0 ]] || fail "D4 (b) AR5: body fix of a merged packet rejected: $output"
sed -i 's/^- Amendments: none$/- Amendments: '"$y_plan"'/' "$repo/$x_packet"
commit_all "$repo" "docs(plans): rewrite X's record" > /dev/null
run_scoped ""
[[ "$CHECK_STATUS" -ne 0 ]] || fail "D4 (b) AR5: rewritten Workflow State of a merged packet accepted"
assert_contains "$output" "merge 済みの packet の Workflow State は書き換えない" "D4 (b) AR5: reason missing"
# (c) own packet whose Plan Commit is not ancestral (rebased plan-first)
git -C "$repo" switch -qc lane-z-old "$main_sha"
write_packet "$repo" "2026-01-03-z.md" "pending" "none"
z_old_plan="$(commit_all "$repo" "docs(plans): Z plan-first")"
git -C "$repo" switch -qc lane-z "$main_sha"
write_packet "$repo" "2026-01-03-z.md" "$z_old_plan" "none"
z_head="$(commit_all "$repo" "docs(plans): Z plan-first rebased")"
run_scoped ""
[[ "$CHECK_STATUS" -ne 0 ]] || fail "D4 (c): own non-ancestral Plan Commit accepted"
assert_contains "$output" "docs/plans/2026-01-03-z.md の Plan Commit" "D4 (c): own packet reason missing"
assert_not_contains "$output" "$x_packet" "D4 (c): merged packet checked with a start point"
# (e) dispatch: WORKFLOW_BASE_SHA = HEAD falls back to origin/main and still checks the own packet
run_scoped "$z_head"
[[ "$CHECK_STATUS" -ne 0 ]] || fail "D4 (e): dispatch with env = HEAD skipped the own packet"
assert_contains "$output" "docs/plans/2026-01-03-z.md の Plan Commit" "D4 (e): own packet reason missing"
assert_not_contains "$output" "$x_packet" "D4 (e): dispatch did not use origin/main"
# (c2) (c) for names git quotes by default (non-ASCII, `"`): the touched packet stays in scope
for name in '2026-01-04-日本語.md' '2026-01-05-"q".md'; do
    git -C "$repo" switch -qc "lane-quoted-old-${name:8:2}" "$main_sha"
    write_packet "$repo" "$name" "pending" "none"
    quoted_old_plan="$(commit_all "$repo" "docs(plans): quoted plan-first")"
    git -C "$repo" switch -qc "lane-quoted-${name:8:2}" "$main_sha"
    write_packet "$repo" "$name" "$quoted_old_plan" "none"
    commit_all "$repo" "docs(plans): quoted plan-first rebased" > /dev/null
    run_scoped ""
    [[ "$CHECK_STATUS" -ne 0 ]] || fail "D4 (c2): own non-ancestral Plan Commit accepted for $name"
    assert_contains "$output" "docs/plans/$name の Plan Commit" "D4 (c2): own packet reason missing for $name"
    assert_not_contains "$output" "$x_packet" "D4 (c2): merged packet checked with a start point"
done
# (f) PH6 (rewritten): criss-cross, two merge-bases between origin/main and HEAD -> ERROR naming it.
git -C "$repo" switch -qc criss-p "$main_sha"
printf 'p\n' > "$repo/p.txt"; p1="$(commit_all "$repo" "p1")"
git -C "$repo" switch -qc criss-q "$main_sha"
printf 'q\n' > "$repo/q.txt"; q1="$(commit_all "$repo" "q1")"
git -C "$repo" merge -q --no-edit "$p1" > /dev/null
git -C "$repo" switch -q criss-p
git -C "$repo" merge -q --no-edit "$q1" > /dev/null
git -C "$repo" update-ref refs/remotes/origin/main criss-q
[[ "$(git -C "$repo" merge-base --all criss-q HEAD | wc -l)" -eq 2 ]] || fail "D4 (f) fixture: expected two merge-bases"
run_scoped ""
[[ "$CHECK_STATUS" -ne 0 ]] || fail "D4 (f): ambiguous merge-base passed"
assert_contains "$output" "merge-base が一意でない" "D4 (f): reason missing"
assert_not_contains "$output" "$x_packet の Plan Commit" "D4 (f): fell back to every packet"
echo "PASS: PK5 diff scope"

echo "PASS: workflow-git-checks"

# T-G4 / PH3 (rewritten): Evidence Mode 行は任意（あれば github だけ）。Phase の行の値は評価しない（D-145 (3)）。
repo="$tmp/github-mode"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_sha="$(commit_all "$repo" base)"
git -C "$repo" update-ref refs/remotes/origin/main "$base_sha"
write_packet "$repo" 2026-01-01-packet.md pending none
plan_sha="$(commit_all "$repo" plan-first)"
write_packet "$repo" 2026-01-01-packet.md "$plan_sha" none
commit_all "$repo" implementation > /dev/null
packet="$repo/docs/plans/2026-01-01-packet.md"
capture_check "$repo" output
[[ "$CHECK_STATUS" == 0 ]] || fail "markerless implementing packet rejected: $output"
sed -i '/^- Phase:/i\- Evidence Mode: github' "$packet"
capture_check "$repo" output
[[ "$CHECK_STATUS" == 0 ]] || fail "Evidence Mode: github packet rejected: $output"
for marker in legacy mystery; do
    sed -i "s/^- Evidence Mode:.*/- Evidence Mode: $marker/" "$packet"
    capture_check "$repo" output
    [[ "$CHECK_STATUS" != 0 ]] || fail "Evidence Mode: $marker accepted"
    assert_contains "$output" "Evidence Mode は廃止。書くなら github" "Evidence Mode: $marker reason missing"
done
sed -i '/^- Evidence Mode:/d; s/^- Phase:.*/- Phase: local-verified/' "$packet"
capture_check "$repo" output
[[ "$CHECK_STATUS" == 0 ]] || fail "PH3: a Phase value outside the old enum was evaluated: $output"
sed -i '/^- Phase:/d' "$packet"
capture_check "$repo" output
[[ "$CHECK_STATUS" == 0 ]] || fail "PH3: a packet without Phase was rejected by the git check: $output"
sed -i '/^## Workflow State/a\\n- Phase: implementing' "$packet"
echo "PASS: workflow state marker; Phase not evaluated"

# D-102 D4: Evidence Mode is read only from the first `## Workflow State` h2 section.
printf '%s\n' '' '## Notes' '' '- Evidence Mode: legacy の語' >> "$packet"
capture_check "$repo" output
[[ "$CHECK_STATUS" == 0 ]] || fail "body bullets outside Workflow State were read: $output"
cp "$packet" "$tmp/d4-packet.md"
sed -i 's/^## Notes$/## Workflow State\n\n- Evidence Mode: legacy\n\n## Notes/' "$packet"
capture_check "$repo" output
# Final review C1 (rewritten): two Workflow State sections are ambiguous for the approval judgement (the helper's rule),
# and the second section's Evidence Mode is still not what stops it.
[[ "$CHECK_STATUS" != 0 ]] || fail "two Workflow State sections were accepted"
assert_contains "$output" "missing/ambiguous" "ambiguous Workflow State reason missing"
assert_not_contains "$output" "Evidence Mode は廃止" "Evidence Mode in a second Workflow State section was counted"
echo "PASS: a second Workflow State section is ambiguous, its Evidence Mode is not read"
cp "$tmp/d4-packet.md" "$packet"
commit_all "$repo" "restore" > /dev/null

# MG-D4: a real shallow clone must fail, even if its visible tip looks consistent.
shallow="$tmp/shallow"
git clone -q --depth 1 "file://$repo" "$shallow"
capture_check "$shallow" output
[[ "$CHECK_STATUS" != 0 ]] || fail "shallow history became successful evidence"
assert_contains "$output" "full history required" "shallow failure reason missing"
echo "PASS: shallow history rejected"

# An unrelated shallow ref does not make this separate parentless target incomplete.
git -C "$shallow" config user.name test
git -C "$shallow" config user.email test@example.invalid
empty_tree="$(git -C "$shallow" mktree < /dev/null)"
complete_root="$(printf 'complete root\n' | git -C "$shallow" commit-tree "$empty_tree")"
git -C "$shallow" switch -q --detach "$complete_root"
# D-145 (5): the active origin is this complete target itself (no lane commits); the shallow main stays elsewhere.
git -C "$shallow" update-ref refs/remotes/origin/main "$complete_root"
capture_check "$shallow" output
[[ "$CHECK_STATUS" == 0 ]] || fail "unrelated shallow marker rejected complete target: $output"
echo "PASS: unrelated shallow boundary excluded"

# ============================================================================
# D-145 (3) H2: 計画の承認の前の実装を止める（PH4・PH5・PH8・PH9）。AR5: archive への移動は受ける。
# ============================================================================
early_repo() {
    repo="$tmp/early-$1"
    init_repo "$repo"
    printf 'base\n' > "$repo/README.md"
    base_commit "$repo"
    write_packet "$repo" "2026-02-01-lane.md" "pending" "none"
    plan_sha="$(commit_all "$repo" "docs(plans): plan-first")"
}
# PH4: pending + implementation (each code kind, and scripts/** / .github/** by path) -> ERROR naming it.
for change in "src/app.ts:frontend" "src-tauri/src/x.rs:rust" ".env.example:env" "weird/x.bin:unknown" \
              "scripts/tool.sh:scripts/tool.sh" ".github/pull_request_template.md:.github/pull_request_template.md"; do
    path="${change%%:*}"; reason="${change#*:}"
    early_repo "ph4-${path//\//-}"
    mkdir -p "$repo/$(dirname "$path")"; printf 'x\n' > "$repo/$path"
    commit_all "$repo" "feat: early" > /dev/null
    capture_check "$repo" output
    [[ "$CHECK_STATUS" -ne 0 ]] || fail "PH4: implementation before approval accepted ($path)"
    assert_contains "$output" "計画の承認の前の実装" "PH4: reason missing ($path)"
    assert_contains "$output" "$reason" "PH4: kind/path not named ($path)"
done
echo "PASS: PH4"
# PH5: pending + docs and the packet only -> passes (plan-first is never stopped).
early_repo ph5
printf 'design\n' > "$repo/docs-design.md"; mkdir -p "$repo/docs"; printf 'spec\n' > "$repo/docs/spec.md"
commit_all "$repo" "docs: design" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "PH5: docs-only plan-first stopped: $output"
echo "PASS: PH5"
# PH8: after approval the same implementation passes.
early_repo ph8
write_packet "$repo" "2026-02-01-lane.md" "$plan_sha" "none"
commit_all "$repo" "docs(plans): Plan Commit を固定する" > /dev/null
mkdir -p "$repo/src"; printf 'x\n' > "$repo/src/app.ts"; commit_all "$repo" "feat: after approval" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "PH8: implementation after approval stopped: $output"
echo "PASS: PH8"
# PH9: WORKFLOW_BASE_SHA empty or = HEAD falls back to origin/main; origin/main = HEAD means no lane commit (0 active).
early_repo ph9
mkdir -p "$repo/src"; printf 'x\n' > "$repo/src/app.ts"; head_sha="$(commit_all "$repo" "feat: early")"
set +e
output="$(cd "$repo" && WORKFLOW_BASE_SHA="$head_sha" bash "$CHECK_SCRIPT" 2>&1)"; CHECK_STATUS=$?
set -e
[[ "$CHECK_STATUS" -ne 0 ]] || fail "PH9: dispatch (env = HEAD) skipped the lane packet"
assert_contains "$output" "計画の承認の前の実装" "PH9: dispatch fallback reason missing"
git -C "$repo" update-ref refs/remotes/origin/main HEAD
set +e
output="$(cd "$repo" && WORKFLOW_BASE_SHA="$head_sha" bash "$CHECK_SCRIPT" 2>&1)"; CHECK_STATUS=$?
set -e
[[ "$CHECK_STATUS" -eq 0 ]] || fail "PH9: main checkout (origin/main = HEAD) was not 0 active: $output"
assert_contains "$output" "active な packet 0 件" "PH9: 0 active not reported"
echo "PASS: PH9"
# AR5: a merged packet moved to docs/archive/plans (the #215 shape) passes; the active packet keeps PK5.
repo="$tmp/ar5-archive"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
commit_all "$repo" "base" > /dev/null
write_packet "$repo" "2026-01-01-x.md" "pending" "none"
x_plan="$(commit_all "$repo" "X plan-first")"
write_packet "$repo" "2026-01-01-x.md" "$x_plan" "none"
commit_all "$repo" "X approved" > /dev/null
git -C "$repo" update-ref refs/remotes/origin/main HEAD
git -C "$repo" switch -qc closeout
mkdir -p "$repo/docs/archive/plans"
git -C "$repo" mv docs/plans/2026-01-01-x.md docs/archive/plans/2026-01-01-x.md
sed -i 's/^- Phase: .*/- Phase: archive/' "$repo/docs/archive/plans/2026-01-01-x.md"
commit_all "$repo" "docs: closeout X" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "AR5: archive move rejected: $output"
echo "PASS: AR5"

# Final review C1: after approval, the plan sections must equal the latest approved snapshot (or register --amend).
repo="$tmp/plan-drift"
init_repo "$repo"
printf 'base\n' > "$repo/README.md"
base_commit "$repo"
write_packet "$repo" "2026-02-02-drift.md" "pending" "none"
printf '\n## Scope\n\n- reviewed item\n' >> "$repo/docs/plans/2026-02-02-drift.md"
plan_sha="$(commit_all "$repo" "docs(plans): plan-first")"
sed -i "s/^- Plan Commit: pending$/- Plan Commit: $plan_sha/; s/^- Phase: plan-gate$/- Phase: implementing/" "$repo/docs/plans/2026-02-02-drift.md"
commit_all "$repo" "docs(plans): Plan Commit を固定する" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -eq 0 ]] || fail "C1: unchanged plan after approval rejected: $output"
printf -- '- unreviewed item\n' >> "$repo/docs/plans/2026-02-02-drift.md"
commit_all "$repo" "docs(plans): widen scope silently" > /dev/null
capture_check "$repo" output
[[ "$CHECK_STATUS" -ne 0 ]] || fail "C1: plan change after approval accepted"
assert_contains "$output" "計画の節が承認の後に変わった" "C1: reason missing"
assert_contains "$output" "plan-approve.sh docs/plans/2026-02-02-drift.md --amend" "C1: next step missing"
echo "PASS: C1 plan change after approval"
