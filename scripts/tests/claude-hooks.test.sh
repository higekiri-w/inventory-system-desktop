#!/usr/bin/env bash
# D-059 / SPEC-HOOK-01 / CH1-CH11: Claude project hook inventory audit.
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_KEY="claude-code-harness@claude-code-harness-marketplace"
PONYTAIL_KEY="ponytail@ponytail"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

# SPEC-WF-HARNESS5-D3: frontmatter (CRLF too) is read as YAML; a top-level hooks, permissionMode or mcpServers key
# (subagent permission / reach settings, https://code.claude.com/docs/en/sub-agents; D-102 D6) or unparsable YAML fails closed.
# --disable-gems -rdate: the same stdlib psych everywhere; a newer psych gem loads Date implicitly, the stock one does not.
frontmatter_lacks_forbidden_keys() {
    awk '{ sub(/\r$/, "") } NR == 1 && $0 != "---" { exit } NR > 1 && $0 == "---" { exit } NR > 1 { print }' "$1" |
        ruby --disable-gems -ryaml -rdate -e 'doc = YAML.safe_load(STDIN.read, permitted_classes: [Date, Time]); exit(doc.is_a?(Hash) && %w[hooks permissionMode mcpServers].any? { |k| doc.key?(k) } ? 1 : 0)' 2>/dev/null
}

validate_inventory() {
    local root="$1"
    local settings="$root/.claude/settings.json"
    local hook_dir="$root/.claude/hooks"

    jq -e '(.hooks // {}) == {}' "$settings" >/dev/null ||
        return 1
    jq -e --arg key "$HARNESS_KEY" '.enabledPlugins[$key] == false' "$settings" >/dev/null ||
        return 1
    # D-147: the ponytail plugin stays off in this project (AGENTS.md and writer.md carry the rule instead).
    jq -e --arg key "$PONYTAIL_KEY" '.enabledPlugins[$key] == false' "$settings" >/dev/null ||
        return 1
    # SPEC-WF-HARNESS5-D4: subagent nesting stays off (depth 1).
    jq -e '.env.CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH == "1"' "$settings" >/dev/null ||
        return 1

    if [[ -d "$hook_dir" ]] && find "$hook_dir" -type f -print -quit | grep -q .; then
        return 1
    fi

    # SPEC-WF-HARNESS5-D3: agent, command file and skill frontmatter register no hooks, permissionMode or mcpServers.
    local file
    while IFS= read -r -d '' file; do
        frontmatter_lacks_forbidden_keys "$file" || return 1
    done < <(
        [[ -d "$root/.claude/agents" ]] && find "$root/.claude/agents" -name '*.md' -print0
        [[ -d "$root/.claude/commands" ]] && find "$root/.claude/commands" -name '*.md' -print0
        [[ -d "$root/.agents/skills" ]] && find "$root/.agents/skills" -name SKILL.md -print0
        [[ -d "$root/.claude/skills" ]] && find "$root/.claude/skills" -name SKILL.md -print0
        true
    )
    # A skill folder turned into a plugin (.claude-plugin/) would load its agents, hooks and MCP.
    # -quit instead of `| grep -q`: under pipefail, find's SIGPIPE after grep exits would read as "none found".
    if [[ -n "$(find "$root/.agents" "$root/.claude" -path "$root/.claude/worktrees" -prune -o -name .claude-plugin -print -quit 2>/dev/null)" ]]; then
        return 1
    fi
}

validate_live_docs() {
    local root="$1"
    local live_files=(
        "$root/CLAUDE.md"
        "$root/docs/AGENT_OPERATING_MANUAL.md"
        "$root/docs/DEV_SETUP_CHECKLIST.md"
        "$root/docs/TOOLING_SKILL_COMMANDS.md"
        "$root/docs/Plans.md"
        "$root/.claude/commands/plan-rally.md"
    )
    local forbidden_patterns=(
        'Claude 固有の `ExitPlanMode` hook'
        'Claude 側 hook（ExitPlanMode チェック等）の機械強制'
        'project 側 `.claude/settings.json` に `SessionStart` hook がある'
        '設定上は有効のまま残っている'
        'hook pass'
        'hook deny'
        'D-1 check'
        'ExitPlanMode 前'
    )
    local pattern

    for pattern in "${forbidden_patterns[@]}"; do
        if rg -Fq -- "$pattern" "${live_files[@]}"; then
            return 1
        fi
    done
}

validate_canonical_gates() {
    local root="$1"

    grep -Fq 'bash "$REPO_ROOT/scripts/doc-consistency-check.sh"' \
        "$root/scripts/pre-push.sh" || return 1
    grep -Fq 'run_required docs "$REPO_ROOT" bash scripts/doc-consistency-check.sh' \
        "$root/scripts/local-ci.sh" || return 1
    grep -Fq 'run: bash scripts/doc-consistency-check.sh' \
        "$root/.github/workflows/ci.yml" || return 1
}

validate_audit_wiring() {
    local root="$1"

    grep -Fq 'bash scripts/tests/run-workflow-tests.sh' \
        "$root/scripts/local-ci.sh" || return 1
    grep -Fq 'bash scripts/tests/claude-hooks.test.sh' \
        "$root/scripts/tests/run-workflow-tests.sh" || return 1
    grep -Fq 'run: bash scripts/tests/claude-hooks.test.sh' \
        "$root/.github/workflows/ci.yml" || return 1
    local path output
    for path in .claude/settings.json .claude/hooks/probe.sh .claude/commands/plan-rally.md .claude/agents/reviewer.md .agents/skills/example/.claude-plugin/plugin.json; do
        output="$(printf '%s\n' "$path" | bash "$root/scripts/ci/classify-changes.sh" --files-from-stdin)" || return 1
        grep -Fxq 'workflow=true' <<< "$output" || return 1
    done
}

validate_repo_ignore() {
    local root="$1"
    local ignored_by

    ignored_by="$(git -C "$root" check-ignore -v .claude/settings.local.json 2>/dev/null)" ||
        return 1
    [[ "$ignored_by" == .gitignore:* ]]
}

validate_contract() {
    local root="$1"

    validate_inventory "$root" &&
        validate_live_docs "$root" &&
        validate_canonical_gates "$root" &&
        validate_audit_wiring "$root" &&
        validate_repo_ignore "$root"
}

validate_source_binding() {
    local script="$1"
    local source_prefix='$SOURCE'
    local source_suffix='_ROOT'
    local invocation

    invocation="validate_contract \"${source_prefix}${source_suffix}\""
    [[ "$(grep -Fc -- "$invocation" "$script")" == "1" ]]
}

expect_rejected() {
    local label="$1"
    local root="$2"

    if validate_contract "$root"; then
        fail "$label mutant was accepted"
    fi
}

command -v ruby >/dev/null || fail "ruby (yaml) is required"
validate_source_binding "$SOURCE_ROOT/scripts/tests/claude-hooks.test.sh" ||
    fail "CH10 source-direct validation binding is missing or ambiguous"
validate_contract "$SOURCE_ROOT" ||
    fail "live Claude hook contract is not the D-059 zero-hook inventory"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

make_fixture() {
    local source="$1"
    local fixture="$2"

    mkdir -p \
        "$fixture/.claude/commands" \
        "$fixture/.claude/hooks" \
        "$fixture/.github/workflows" \
        "$fixture/docs" \
        "$fixture/scripts/ci" \
        "$fixture/scripts/tests"
    cp "$source/.claude/settings.json" "$fixture/.claude/settings.json"
    cp "$source/.claude/commands/plan-rally.md" "$fixture/.claude/commands/plan-rally.md"
    cp "$source/.github/workflows/ci.yml" "$fixture/.github/workflows/ci.yml"
    cp "$source/docs/AGENT_OPERATING_MANUAL.md" "$fixture/docs/AGENT_OPERATING_MANUAL.md"
    cp "$source/docs/DEV_SETUP_CHECKLIST.md" "$fixture/docs/DEV_SETUP_CHECKLIST.md"
    cp "$source/docs/TOOLING_SKILL_COMMANDS.md" "$fixture/docs/TOOLING_SKILL_COMMANDS.md"
    cp "$source/docs/Plans.md" "$fixture/docs/Plans.md"
    cp "$source/scripts/ci/classify-changes.sh" "$fixture/scripts/ci/classify-changes.sh"
    cp "$source/scripts/local-ci.sh" "$fixture/scripts/local-ci.sh"
    cp "$source/scripts/tests/run-workflow-tests.sh" "$fixture/scripts/tests/run-workflow-tests.sh"
    cp "$source/scripts/pre-push.sh" "$fixture/scripts/pre-push.sh"
    cp "$source/CLAUDE.md" "$fixture/CLAUDE.md"
    cp "$source/.gitignore" "$fixture/.gitignore"
    if [[ -d "$source/.claude/hooks" ]]; then
        cp -a "$source/.claude/hooks/." "$fixture/.claude/hooks/"
    fi
    git -C "$fixture" init -q
}

fixture="$tmp/base"
make_fixture "$SOURCE_ROOT" "$fixture"
validate_contract "$fixture" || fail "unmodified contract fixture is not green"

source_hook_mutant="$tmp/source-hook-mutant"
cp -a "$fixture" "$source_hook_mutant"
printf '%s\n' '#!/usr/bin/env bash' > "$source_hook_mutant/.claude/hooks/stray.sh"
propagated_hook_fixture="$tmp/propagated-hook-fixture"
make_fixture "$source_hook_mutant" "$propagated_hook_fixture"
[[ -f "$propagated_hook_fixture/.claude/settings.json" ]] ||
    fail "CH10 source-derived hook fixture was not constructed"
expect_rejected "CH10 source hook propagation" "$propagated_hook_fixture"

hook_mutant="$tmp/hook-mutant"
cp -a "$fixture" "$hook_mutant"
jq '.hooks = {"PreToolUse":[{"matcher":"ExitPlanMode","hooks":[{"type":"command","command":"false"}]}]}' \
    "$fixture/.claude/settings.json" > "$hook_mutant/.claude/settings.json"
expect_rejected "CH1 extra hook" "$hook_mutant"

plugin_mutant="$tmp/plugin-mutant"
cp -a "$fixture" "$plugin_mutant"
jq --arg key "$HARNESS_KEY" '.enabledPlugins[$key] = true' \
    "$fixture/.claude/settings.json" > "$plugin_mutant/.claude/settings.json"
expect_rejected "CH2 harness true" "$plugin_mutant"

ponytail_true_mutant="$tmp/ponytail-true-mutant"
cp -a "$fixture" "$ponytail_true_mutant"
jq --arg key "$PONYTAIL_KEY" '.enabledPlugins[$key] = true' \
    "$fixture/.claude/settings.json" > "$ponytail_true_mutant/.claude/settings.json"
expect_rejected "D-147 ponytail true" "$ponytail_true_mutant"

ponytail_missing_mutant="$tmp/ponytail-missing-mutant"
cp -a "$fixture" "$ponytail_missing_mutant"
jq --arg key "$PONYTAIL_KEY" 'del(.enabledPlugins[$key])' \
    "$fixture/.claude/settings.json" > "$ponytail_missing_mutant/.claude/settings.json"
expect_rejected "D-147 ponytail missing" "$ponytail_missing_mutant"

script_mutant="$tmp/script-mutant"
cp -a "$fixture" "$script_mutant"
printf '%s\n' '#!/usr/bin/env bash' > "$script_mutant/.claude/hooks/check-plan-on-exit.sh"
expect_rejected "CH3 retired script" "$script_mutant"

claim_mutant="$tmp/claim-mutant"
cp -a "$fixture" "$claim_mutant"
printf '%s\n' 'Claude 固有の `ExitPlanMode` hook' >> "$claim_mutant/CLAUDE.md"
expect_rejected "CH4 active hook claim" "$claim_mutant"

plans_claim_mutant="$tmp/plans-claim-mutant"
cp -a "$fixture" "$plans_claim_mutant"
printf '%s\n' 'Claude 固有の `ExitPlanMode` hook' >> "$plans_claim_mutant/docs/Plans.md"
expect_rejected "CH4 Plans backlog claim" "$plans_claim_mutant"

gate_mutant="$tmp/gate-mutant"
cp -a "$fixture" "$gate_mutant"
sed -i '\|bash "$REPO_ROOT/scripts/doc-consistency-check.sh"|d' \
    "$gate_mutant/scripts/pre-push.sh"
expect_rejected "CH5 canonical gate wiring" "$gate_mutant"

audit_mutant="$tmp/audit-mutant"
cp -a "$fixture" "$audit_mutant"
sed -i '/run_required workflow-suite/d' "$audit_mutant/scripts/local-ci.sh"
expect_rejected "CH7 local audit wiring" "$audit_mutant"

suite_mutant="$tmp/suite-mutant"
cp -a "$fixture" "$suite_mutant"
sed -i '\|bash scripts/tests/claude-hooks.test.sh|d' "$suite_mutant/scripts/tests/run-workflow-tests.sh"
expect_rejected "CH7 shared audit wiring" "$suite_mutant"

ignore_mutant="$tmp/ignore-mutant"
cp -a "$fixture" "$ignore_mutant"
sed -i '\|^\*\*/\.claude/settings\.local\.json$|d' "$ignore_mutant/.gitignore"
expect_rejected "CH11 repository ignore" "$ignore_mutant"

# SPEC-WF-HARNESS5-D3/D4: clean agent and skill frontmatter pass; hooks, plugins and a missing depth are rejected.
frontmatter_fixture="$tmp/frontmatter-fixture"
cp -a "$fixture" "$frontmatter_fixture"
mkdir -p "$frontmatter_fixture/.claude/agents" "$frontmatter_fixture/.agents/skills/example"
printf '%s\n' '---' 'name: writer' 'description: fixture' 'model: opus' 'effort: medium' '---' 'body' \
    > "$frontmatter_fixture/.claude/agents/writer.md"
printf '%s\n' '---' 'name: example' 'metadata:' '  updated: 2026-09-30' '---' 'body' \
    > "$frontmatter_fixture/.agents/skills/example/SKILL.md"
validate_contract "$frontmatter_fixture" || fail "clean agent and skill frontmatter was rejected"

depth_mutant="$tmp/depth-mutant"
cp -a "$fixture" "$depth_mutant"
jq 'del(.env)' "$fixture/.claude/settings.json" > "$depth_mutant/.claude/settings.json"
expect_rejected "HARNESS5-D4 subagent depth" "$depth_mutant"

agents_classifier_mutant="$tmp/agents-classifier-mutant"
cp -a "$fixture" "$agents_classifier_mutant"
sed -i 's#|\.claude/agents/\*##' "$agents_classifier_mutant/scripts/ci/classify-changes.sh"
expect_rejected "HARNESS5-D1 agent classification" "$agents_classifier_mutant"

agent_hooks_mutant="$tmp/agent-hooks-mutant"
cp -a "$frontmatter_fixture" "$agent_hooks_mutant"
printf '%s\n' '---' 'name: writer' "'hooks':" '  Stop: []' '---' 'body' > "$agent_hooks_mutant/.claude/agents/writer.md"
expect_rejected "HARNESS5-D3 agent frontmatter hooks" "$agent_hooks_mutant"

command_hooks_mutant="$tmp/command-hooks-mutant"
cp -a "$fixture" "$command_hooks_mutant"
{ printf '%s\n' '---' '"hooks":' '---'; cat "$fixture/.claude/commands/plan-rally.md"; } \
    > "$command_hooks_mutant/.claude/commands/plan-rally.md"
expect_rejected "HARNESS5-D3 command frontmatter hooks" "$command_hooks_mutant"

skill_hooks_mutant="$tmp/skill-hooks-mutant"
cp -a "$frontmatter_fixture" "$skill_hooks_mutant"
printf '%s\n' '---' 'name: example' 'hooks:' '  Stop: []' '---' 'body' > "$skill_hooks_mutant/.agents/skills/example/SKILL.md"
expect_rejected "HARNESS5-D3 skill frontmatter hooks" "$skill_hooks_mutant"

plugin_manifest_mutant="$tmp/plugin-manifest-mutant"
cp -a "$frontmatter_fixture" "$plugin_manifest_mutant"
mkdir -p "$plugin_manifest_mutant/.agents/skills/example/.claude-plugin"
printf '{"name":"example"}\n' > "$plugin_manifest_mutant/.agents/skills/example/.claude-plugin/plugin.json"
expect_rejected "HARNESS5-D3 skill plugin manifest" "$plugin_manifest_mutant"

crlf_hooks_mutant="$tmp/crlf-hooks-mutant"
cp -a "$frontmatter_fixture" "$crlf_hooks_mutant"
printf '%s\r\n' '---' 'name: writer' 'hooks:' '  Stop: []' '---' 'body' > "$crlf_hooks_mutant/.claude/agents/writer.md"
expect_rejected "HARNESS5-D3 CRLF agent frontmatter hooks" "$crlf_hooks_mutant"

claude_skill_hooks_mutant="$tmp/claude-skill-hooks-mutant"
cp -a "$frontmatter_fixture" "$claude_skill_hooks_mutant"
mkdir -p "$claude_skill_hooks_mutant/.claude/skills/local"
printf '%s\n' '---' 'name: local' 'hooks:' '  Stop: []' '---' 'body' > "$claude_skill_hooks_mutant/.claude/skills/local/SKILL.md"
expect_rejected "HARNESS5-D3 .claude/skills frontmatter hooks" "$claude_skill_hooks_mutant"

agent_permission_mode_mutant="$tmp/agent-permission-mode-mutant"
cp -a "$frontmatter_fixture" "$agent_permission_mode_mutant"
printf '%s\n' '---' 'name: writer' 'permissionMode: bypassPermissions' '---' 'body' > "$agent_permission_mode_mutant/.claude/agents/writer.md"
expect_rejected "HARNESS5-D3 agent frontmatter permissionMode" "$agent_permission_mode_mutant"

agent_mcp_servers_mutant="$tmp/agent-mcp-servers-mutant"
cp -a "$frontmatter_fixture" "$agent_mcp_servers_mutant"
printf '%s\n' '---' 'name: writer' 'mcpServers:' '  - playwright:' '      type: stdio' '      command: npx' '---' 'body' > "$agent_mcp_servers_mutant/.claude/agents/writer.md"
expect_rejected "HARNESS5-D3 agent frontmatter mcpServers" "$agent_mcp_servers_mutant"

skill_permission_mode_mutant="$tmp/skill-permission-mode-mutant"
cp -a "$frontmatter_fixture" "$skill_permission_mode_mutant"
printf '%s\n' '---' 'name: example' 'permissionMode: plan' '---' 'body' > "$skill_permission_mode_mutant/.agents/skills/example/SKILL.md"
expect_rejected "HARNESS5-D3 skill frontmatter permissionMode" "$skill_permission_mode_mutant"

many_manifests_mutant="$tmp/many-manifests-mutant"
cp -a "$frontmatter_fixture" "$many_manifests_mutant"
mkdir -p "$many_manifests_mutant"/.agents/skills/s{1..200}/.claude-plugin
expect_rejected "HARNESS5-D3 many skill plugin manifests" "$many_manifests_mutant"

echo "PASS: Claude hook zero-inventory contract"
