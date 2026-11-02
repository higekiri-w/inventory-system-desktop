#!/usr/bin/env bash
# scripts/tests/reading-order-drift.test.sh
# SPEC-WF-DRIFT: AGENTS.md の canonical reading order (Session Start) を
# 他の tracked docs が再掲していないかを検出する drift test。
# このファイル自体が「drift test」であり (Test Design Matrix
# docs/plans/test-matrices/2026-07-12-mechanical-workflow-slice2.md 「同ファイル自体が test」)、
# 別途の check スクリプトは持たない。
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

# 退役した無条件の全文読書列の再掲を検出する正規表現。矢印列挙・番号列挙のいずれも
# 実体は「4 つのランドマークがこの順序で近接して出現する」ことなので、単一パターンで
# 両形式を捕捉する。近接判定の window（80 文字）は実 repo で検証済み（無関係な言及が
# 数百文字離れて出現する程度では誤検出しないことを負例で確認する）。
DRIFT_PATTERN='AGENTS(\.md)?.{0,80}?DEV_WORKFLOW(\.md)?.{0,80}?Plans(\.md)?.{0,80}?project-memory(\.md)?'

# 除外対象:
#   - AGENTS.md 自身（canonical reading order の定義元）
#   - docs/archive/** （歴史記録、書き換えない）
#   - docs/decision-log.md （追記型の決定記録、書き換えない。D-034 本文に既知の再掲が存在する）
#   - docs/plans/** （Plan Packet は "not durable design source of truth" であり、
#     本 slice 自身の Plan Packet / Test Design Matrix が drift test の仕様説明として
#     このパターンを引用する。これは正本の competing 定義ではなく本機能の説明であるため、
#     durable source doc を対象とする drift test の趣旨には当たらない。指示書に明記の
#     3 除外に加えたこの追加除外は Writer 判断であり、報告で明示する）
build_target_files() {
    (
        cd "$SOURCE_ROOT"
        git ls-files '*.md' | grep -v \
            -e '^AGENTS\.md$' \
            -e '^docs/archive/' \
            -e '^docs/decision-log\.md$' \
            -e '^docs/plans/'
    )
}

mapfile -t targets < <(build_target_files)
[[ "${#targets[@]}" -gt 0 ]] || fail "スキャン対象ファイルが 0 件です（除外条件が広すぎる可能性）"

for t in "${targets[@]}"; do
    case "$t" in
        AGENTS.md) fail "AGENTS.md が除外されず対象に含まれています" ;;
        docs/archive/*) fail "docs/archive/ 配下の $t が除外されず対象に含まれています" ;;
        docs/decision-log.md) fail "docs/decision-log.md が除外されず対象に含まれています" ;;
        docs/plans/*) fail "docs/plans/ 配下の $t が除外されず対象に含まれています" ;;
    esac
done

# --- 負例(a): 除外なしでは docs/decision-log.md の既知の再掲（D-034）が検出できることを確認 ---
# （除外がないと即 fail することの実証。パターン自体が壊れていないことも兼ねて確認する）
if ! rg -U --multiline-dotall -q "$DRIFT_PATTERN" "$SOURCE_ROOT/docs/decision-log.md"; then
    fail "docs/decision-log.md の既知の再掲（D-034）が検出できません。パターンが壊れています"
fi

# HC-D2/HC-D9: structural contract lint, not proof of model behavior.
# Keep the R3/R4 route, complete state input, and fail-closed marker discoverable (D-145: R2 left the packet route);
# the routing fixture that exercised them was retired with D-101 (PR #132).
check_entry_contract() {
    local entry
    entry="$(awk '/^## Session Start[[:space:]]*$/ { active=1; next }
        active && /^## / { exit } active { print }' "$1")"
    if ! printf '%s\n' "$entry" | rg '^\|' | rg 'R3' |
        rg -i '(実装|再開|implement|resume)' | rg 'Workflow State' |
        rg -qi '(完全|complete|full)'; then
        echo "missing R3/R4 complete Workflow State route: $1" >&2
        return 1
    fi
    if ! printf '%s\n' "$entry" | rg -q 'fail-closed'; then
        echo "missing fail-closed boundary in Session Start: $1" >&2
        return 1
    fi
}
check_entry_contract "$SOURCE_ROOT/AGENTS.md" || fail "entry contract is incomplete"

# --- 合成 fixture: 一般的な再掲を検出できることを確認 ---
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Real policy mutations: a missing route and a missing stop boundary must fail.
awk '!/^\| R3〜R4 /' "$SOURCE_ROOT/AGENTS.md" > "$tmp/no-r3.md"
if check_entry_contract "$tmp/no-r3.md" >/dev/null 2>&1; then
    fail "missing R3/R4 route was accepted"
fi
sed '/fail-closed/d' "$SOURCE_ROOT/AGENTS.md" > "$tmp/no-stop.md"
if check_entry_contract "$tmp/no-stop.md" >/dev/null 2>&1; then
    fail "missing fail-closed boundary was accepted"
fi

cat > "$tmp/onboarding.md" <<'FIXTURE'
# Onboarding

読む順序: AGENTS.md -> DEV_WORKFLOW.md -> Plans.md -> project-memory.md
FIXTURE
if ! rg -U --multiline-dotall -q "$DRIFT_PATTERN" "$tmp/onboarding.md"; then
    fail "合成 negative fixture（-> 形式の再掲）を検出できません"
fi

cat > "$tmp/numbered.md" <<'FIXTURE'
# Session Start (copy)

1. `AGENTS.md` (this file)
2. `docs/DEV_WORKFLOW.md`
3. `Plans.md`
4. `docs/project-memory.md`
FIXTURE
if ! rg -U --multiline-dotall -q "$DRIFT_PATTERN" "$tmp/numbered.md"; then
    fail "合成 negative fixture（番号列挙形式の再掲）を検出できません"
fi

# --- 合成 fixture: 無関係な遠隔言及を誤検出しないことを確認（false positive 防止）---
cat > "$tmp/clean.md" <<'FIXTURE'
# Unrelated doc

This section discusses AGENTS.md at length, covering its role as the entry
point for every session and explaining why it exists in painstaking detail
across many words that pad this paragraph well past any reasonable proximity
window so nothing downstream looks related to it at all.

Meanwhile DEV_WORKFLOW.md is discussed here instead, in a completely
different context about testing conventions, still with plenty of padding
text so the gap remains large enough to exceed eighty characters easily,
entirely on purpose and by design.

Later, Plans.md gets its own paragraph too, again separated from the
previous topics by a wall of unrelated prose that exists purely to keep the
character distance comfortably above the matching window used by the drift
detector in this test.

Finally project-memory is mentioned on its own, in isolation, without any
of the earlier terms appearing nearby, confirming that mere co-occurrence
of all four terms somewhere in one long document does not by itself trigger
a false positive drift finding.
FIXTURE
if rg -U --multiline-dotall -q "$DRIFT_PATTERN" "$tmp/clean.md"; then
    fail "遠隔言及（80 文字超の間隔）まで誤検出しています（false positive）"
fi

# --- 実チェック: 現 repo 全体（除外分を除く）で drift 0 件であること ---
absolute_targets=()
for t in "${targets[@]}"; do
    absolute_targets+=("$SOURCE_ROOT/$t")
done

if violations="$(rg -U --multiline-dotall -n "$DRIFT_PATTERN" "${absolute_targets[@]}" 2>/dev/null)"; then
    echo "$violations" >&2
    fail "canonical reading order の再掲が検出されました（上記ファイル参照）"
fi

echo "PASS: reading-order-drift"

# SPEC-MERGE-EVIDENCE / MG-D5/D8: entrypoints route to the explicit evidence owner.
for path in AGENTS.md docs/DEV_WORKFLOW.md; do
    rg -q '直接UI merge' "$SOURCE_ROOT/$path" || fail "helper-only merge boundary missing: $path"
    rg -q '残存リスク' "$SOURCE_ROOT/$path" || fail "server enforcement limitation missing: $path"
done
echo "PASS: merge evidence entry routing"
