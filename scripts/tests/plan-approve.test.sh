#!/usr/bin/env bash
# scripts/tests/plan-approve.test.sh — D-145 (2) / H1: PA1..PA5 on synthetic repos in a tmpdir.
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }
PKT="docs/plans/2026-10-20-lane.md"

packet() { # $1 plan commit, $2 amendments, $3 scope line
    printf '# Plan Packet: lane\n\n## Workflow State\n\n- Risk: R3\n- Plan Commit: %s\n- Amendments: %s\n\n## Risk\n\nRisk: R3\n\n## Scope\n\n%s\n' \
        "$1" "$2" "${3:-- scope}"
}

new_repo() {
    repo="$tmp/$1"
    mkdir -p "$repo/scripts" "$repo/docs/plans"
    git -C "$repo" init -q -b main
    git -C "$repo" config user.name test
    git -C "$repo" config user.email test@example.invalid
    cp "$SOURCE_ROOT/scripts/plan-approve.sh" "$SOURCE_ROOT/scripts/pr-gate.py" "$repo/scripts/"
    printf 'base\n' > "$repo/README.md"
    git -C "$repo" add -A && git -C "$repo" commit -qm base
    git -C "$repo" update-ref refs/remotes/origin/main HEAD
    git -C "$repo" switch -qc lane
    packet pending none > "$repo/$PKT"
    git -C "$repo" add -A && git -C "$repo" commit -qm plan-first
    reviewed="$(git -C "$repo" rev-parse HEAD)"
}

approve() { # captures output and status; never aborts the test
    set +e
    out="$(cd "$repo" && bash scripts/plan-approve.sh "$@" 2>&1)"
    status=$?
    set -e
}

refused() { # $1 label, $2 expected text; HEAD and tree unchanged
    local label="$1" text="$2" before="$3"
    [[ "$status" -ne 0 ]] || fail "$label: accepted: $out"
    [[ "$out" == *"$text"* ]] || fail "$label: reason missing ($text): $out"
    [[ "$(git -C "$repo" rev-parse HEAD)" == "$before" ]] || fail "$label: a commit was made"
    git -C "$repo" diff --quiet -- "$PKT" || fail "$label: the packet was left modified"
}

# PA1: pending -> the reported SHA, one packet-only commit.
new_repo pa1
approve "$PKT" "$reviewed"
[[ "$status" -eq 0 ]] || fail "PA1: $out"
[[ "$(git -C "$repo" show HEAD:"$PKT")" == "$(packet "$reviewed" none)" ]] || fail "PA1: packet text"
[[ "$(git -C "$repo" show --name-only --format= HEAD)" == "$PKT" ]] || fail "PA1: commit touches more than the packet"
[[ "$(git -C "$repo" rev-parse HEAD~1)" == "$reviewed" ]] || fail "PA1: not one commit"
echo "PASS: PA1"

# PA2: malformed, non-ancestor, already-approved SHAs write nothing.
new_repo pa2
before="$(git -C "$repo" rev-parse HEAD)"
for bad in "${reviewed:0:12}" "${reviewed^^}" "${reviewed}0"; do
    approve "$PKT" "$bad"; refused "PA2 malformed $bad" "40-char lowercase hex" "$before"
done
git -C "$repo" switch -qc side main
printf 'side\n' > "$repo/side.txt"; git -C "$repo" add -A; git -C "$repo" commit -qm side
side="$(git -C "$repo" rev-parse HEAD)"; git -C "$repo" switch -q lane
approve "$PKT" "$side"; refused "PA2 non-ancestor" "not an ancestor of HEAD" "$before"
approve "$PKT" "$reviewed"
before="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" "$reviewed"; refused "PA2 already approved" "not pending" "$before"
echo "PASS: PA2"

# PA3: --amend appends after the Plan Commit; duplicates and non-descendants are refused.
new_repo pa3
approve "$PKT" "$reviewed"
printf 'impl\n' > "$repo/impl.txt"; git -C "$repo" add -A; git -C "$repo" commit -qm impl
a1="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" --amend "$a1"
[[ "$status" -eq 0 ]] || fail "PA3 first: $out"
printf 'more\n' >> "$repo/impl.txt"; git -C "$repo" add -A; git -C "$repo" commit -qm more
a2="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" --amend "$a2"
[[ "$status" -eq 0 ]] || fail "PA3 second: $out"
grep -qx -- "- Amendments: $a1, $a2" "$repo/$PKT" || fail "PA3: order $(grep Amendments "$repo/$PKT")"
before="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" --amend "$a1"; refused "PA3 duplicate" "already in Amendments" "$before"
base_sha="$(git -C "$repo" rev-parse main)"
approve "$PKT" --amend "$base_sha"; refused "PA3 non-descendant" "not a descendant of Plan Commit" "$before"
echo "PASS: PA3"

# PA4 (a): a lane commit after SHA touching another path; (b) a plan change after SHA; (c) local edits.
new_repo pa4a
printf 'code\n' > "$repo/impl.txt"; git -C "$repo" add -A; git -C "$repo" commit -qm impl
before="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" "$reviewed"; refused "PA4a" "impl.txt" "$before"
new_repo pa4b
packet pending none "- scope
- one more item" > "$repo/$PKT"; git -C "$repo" commit -qam "widen scope"
before="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" "$reviewed"; refused "PA4b" "changed after" "$before"
new_repo pa4c
before="$(git -C "$repo" rev-parse HEAD)"
printf -- '- unstaged scope item\n' >> "$repo/$PKT"
approve "$PKT" "$reviewed"
[[ "$status" -ne 0 && "$out" == *"staged or unstaged edits"* ]] || fail "PA4c unstaged: $out"
git -C "$repo" add "$PKT"
approve "$PKT" "$reviewed"
[[ "$status" -ne 0 && "$out" == *"staged or unstaged edits"* ]] || fail "PA4c staged: $out"
[[ "$(git -C "$repo" rev-parse HEAD)" == "$before" ]] || fail "PA4c: a commit was made"
git -C "$repo" reset -q --hard
approve "$PKT" "$reviewed"
[[ "$status" -eq 0 ]] || fail "PA4 positive: $out"
echo "PASS: PA4"

# PA5: a sync merge of main between SHA and HEAD passes; a merge with an edit or from a side branch does not.
new_repo pa5
git -C "$repo" switch -q main
printf 'main moved\n' > "$repo/main.txt"; git -C "$repo" add -A; git -C "$repo" commit -qm main
git -C "$repo" update-ref refs/remotes/origin/main HEAD
git -C "$repo" switch -q lane
git -C "$repo" -c merge.directoryRenames=false merge -q --no-ff --no-edit origin/main
sync="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" "$reviewed"
[[ "$status" -eq 0 ]] || fail "PA5 sync: $out"
git -C "$repo" reset -q --hard "$sync"
git -C "$repo" switch -q main
printf 'again\n' >> "$repo/main.txt"; git -C "$repo" add -A; git -C "$repo" commit -qm main2
git -C "$repo" update-ref refs/remotes/origin/main HEAD
git -C "$repo" switch -q lane
git -C "$repo" -c merge.directoryRenames=false merge -q --no-ff --no-commit origin/main > /dev/null
printf 'edit inside the merge\n' > "$repo/hidden.txt"; git -C "$repo" add -A; git -C "$repo" commit -q --no-edit
before="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" "$reviewed"; refused "PA5 edit in merge" "not a sync of main" "$before"
git -C "$repo" reset -q --hard "$sync"
git -C "$repo" switch -qc side "$reviewed~1"
printf 'side\n' > "$repo/side.txt"; git -C "$repo" add -A; git -C "$repo" commit -qm side
git -C "$repo" switch -q lane
git -C "$repo" merge -q --no-ff --no-edit side
before="$(git -C "$repo" rev-parse HEAD)"
approve "$PKT" "$reviewed"; refused "PA5 side branch" "not a sync of main" "$before"
echo "PASS: PA5"

# Final review C2: a failure to list the merges stops before any write (never read as "no merge").
new_repo pa-c2
shim="$tmp/fail-merge-list"; mkdir -p "$shim"
printf '#!/usr/bin/env bash\nfor a in "$@"; do [[ "$a" == --merges ]] && exit 2; done\nexec %q "$@"\n' "$(command -v git)" > "$shim/git"
chmod +x "$shim/git"
before="$(git -C "$repo" rev-parse HEAD)"
set +e
out="$(cd "$repo" && PATH="$shim:$PATH" bash scripts/plan-approve.sh "$PKT" "$reviewed" 2>&1)"; status=$?
set -e
refused "C2 merge list failure" "cannot list the merges" "$before"
echo "PASS: C2"

echo "PASS: plan-approve"
