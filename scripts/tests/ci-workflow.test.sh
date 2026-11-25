#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOW="$REPO_ROOT/.github/workflows/ci.yml"
NPM_SECURITY_WORKFLOW="$REPO_ROOT/.github/workflows/npm-security-monitor.yml"
WINDOWS_OBSERVE_WORKFLOW="$REPO_ROOT/.github/workflows/windows-observe.yml"
L3_SMOKE_WORKFLOW="$REPO_ROOT/.github/workflows/l3-smoke.yml"
L3_SMOKE_SCRIPT="$REPO_ROOT/scripts/ci/l3-smoke-home.mjs"
NODE_VERSION_FILE="$REPO_ROOT/.node-version"
PACKAGE_JSON="$REPO_ROOT/package.json"
PR_TEMPLATE="$REPO_ROOT/.github/pull_request_template.md"
CI_DOC="$REPO_ROOT/docs/ci.md"
DEV_WORKFLOW_DOC="$REPO_ROOT/docs/DEV_WORKFLOW.md"
DECISION_LOG="$REPO_ROOT/docs/decision-log.md"
PLANS_DOC="$REPO_ROOT/docs/Plans.md"
PROJECT_HANDOFF_DOC="$REPO_ROOT/docs/PROJECT_HANDOFF.md"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

require_fixed() {
    local text="$1"
    grep -Fq -- "$text" "$WORKFLOW" || fail "workflow missing: $text"
}

reject_fixed() {
    local text="$1"
    if grep -Fq -- "$text" "$WORKFLOW"; then
        fail "workflow still contains: $text"
    fi
}

ruby -e "require 'yaml'; YAML.parse_file(ARGV.fetch(0))" "$WORKFLOW"

validate_job_graph() {
    ruby - "$1" <<'RUBY'
require "yaml"
workflow = YAML.safe_load(File.read(ARGV.fetch(0)), aliases: true)
jobs = workflow.fetch("jobs")
jobs.each do |name, job|
  next if name == "changes"
  needs = Array(job["needs"])
  abort "job #{name} does not depend on changes" unless needs.include?("changes")
  condition = job["if"].to_s
  expected_always_guard = "always() && needs.changes.result == 'success'"
  if name == "merge_gate"
    # D-100 / D-145 (9): the aggregate runs for Draft too and always publishes Merge gate.
    abort "aggregate must run even when classification fails" unless condition == "always()"
    abort "aggregate must depend on every job" unless needs.sort == (jobs.keys - ["merge_gate"]).sort
  elsif condition.include?("always()") && condition != expected_always_guard
    abort "always job #{name} can run after changes is skipped: #{condition.inspect}"
  end
end
RUBY
}

validate_workflow_contract() {
    ruby - "$1" <<'RUBY'
require "yaml"
workflow = YAML.safe_load(File.read(ARGV.fetch(0)), aliases: true)
triggers = workflow["on"] || workflow[true]
abort "workflow trigger map is missing" unless triggers.is_a?(Hash)

expected_trigger_keys = %w[pull_request workflow_dispatch]
actual_trigger_keys = triggers.keys.map(&:to_s)
unless actual_trigger_keys.length == expected_trigger_keys.length && actual_trigger_keys.uniq.length == actual_trigger_keys.length && actual_trigger_keys.sort == expected_trigger_keys.sort
  abort "workflow triggers must be exactly #{expected_trigger_keys.inspect}; got #{actual_trigger_keys.inspect}"
end

pull_request = triggers.fetch("pull_request")
abort "pull_request trigger is not a map" unless pull_request.is_a?(Hash)
types = pull_request.fetch("types")
abort "pull_request.types is not an array" unless types.is_a?(Array)

# D-043/D-100/D-145 (9): CI reacts to every PR head update once; Ready does not run CI again (CI1).
expected = %w[opened reopened synchronize]
actual = types.map(&:to_s)
unless actual.length == expected.length && actual.uniq.length == actual.length && actual.sort == expected.sort
  abort "pull_request.types must be exactly #{expected.inspect}; got #{actual.inspect}"
end

expected_branches = ["main"]
abort "pull_request.branches drifted" unless pull_request.fetch("branches") == expected_branches

abort "PR filtering can suppress merge evidence" if pull_request.key?("paths-ignore") || pull_request.key?("paths")
abort "CI token must be read-only" unless workflow["permissions"] == {"contents" => "read"}

concurrency = workflow.fetch("concurrency")
abort "concurrency is not a map" unless concurrency.is_a?(Hash)
abort "superseded-run cancellation is not enabled" unless concurrency.fetch("cancel-in-progress") == true

jobs = workflow.fetch("jobs")
changes = jobs.fetch("changes")
# D-100: changes has no guard so Draft PRs classify and run the selected jobs.
abort "changes job must run for Draft: #{changes["if"].inspect}" if changes.key?("if")

filter_steps = changes.fetch("steps").select { |step| step["id"] == "filter" }
abort "expected exactly one classifier step" unless filter_steps.length == 1
run_lines = filter_steps.first.fetch("run").lines.map(&:strip).reject(&:empty?)
else_index = run_lines.index("else")
fi_index = else_index && run_lines.each_index.find { |index| index > else_index && run_lines[index] == "fi" }
abort "classifier PR branch is missing" unless else_index && fi_index
actual_pr_branch = run_lines[(else_index + 1)...fi_index]
expected_pr_branch = [
  "scripts/ci/classify-changes.sh \\",
  "--base \"${{ github.event.pull_request.base.sha }}\" \\",
  "--head \"${{ github.event.pull_request.head.sha }}\" > \"$output\"",
]
unless actual_pr_branch == expected_pr_branch
  abort "classifier PR base/head routing drifted: #{actual_pr_branch.inspect}"
end

expected_job_names = {
  "changes" => "Detect changed areas",
  "rust_lint" => "Rust fmt/clippy",
  "rust_test" => "Rust tests",
  "rust_drift" => "Rust generated drift",
  "merge_gate" => "Merge gate",
  "workflow" => "Workflow regression",
  "docs" => "Design doc consistency",
  "env_safety" => "Env safety",
  "frontend" => "Frontend (typecheck + lint + format + build)",
}
expected_job_names.each do |job_key, expected_name|
  actual_name = jobs.fetch(job_key).fetch("name")
  abort "job name drifted for #{job_key}: #{actual_name.inspect}" unless actual_name == expected_name
end
RUBY
}

validate_node_contract() {
    local node_version_file="${1:-$NODE_VERSION_FILE}"
    local package_json="${2:-$PACKAGE_JSON}"
    local ci_workflow="${3:-$WORKFLOW}"
    local npm_security_workflow="${4:-$NPM_SECURITY_WORKFLOW}"
    local l3_smoke_workflow="${5:-$L3_SMOKE_WORKFLOW}"

    ruby - "$node_version_file" "$package_json" "$ci_workflow" "$npm_security_workflow" "$l3_smoke_workflow" <<'RUBY'
require "json"
require "yaml"

node_version_file, package_json, *workflow_paths = ARGV
pin_lines = File.readlines(node_version_file, chomp: true)
abort ".node-version must contain exactly 24.18.0" unless pin_lines == ["24.18.0"]

manifest = JSON.parse(File.read(package_json))
expected_range = ">=24 <25"
abort "engines.node drifted" unless manifest.dig("engines", "node") == expected_range

runtime = manifest.dig("devEngines", "runtime")
expected_runtime = {
  "name" => "node",
  "version" => expected_range,
  "onFail" => "error",
}
abort "devEngines.runtime drifted" unless runtime == expected_runtime

types_node = manifest.dig("devDependencies", "@types/node")
abort "@types/node must stay on major 24" unless types_node&.match?(/\A\^24\.\d+\.\d+\z/)

workflow_paths.each do |path|
  workflow = YAML.safe_load(File.read(path), aliases: true)
  setup_steps = workflow.fetch("jobs").values.flat_map { |job| job.fetch("steps", []) }
    .select { |step| step["uses"] == "actions/setup-node@v6" }
  abort "#{path}: setup-node@v6 step is missing" if setup_steps.empty?

  setup_steps.each do |step|
    setup_with = step.fetch("with")
    unless setup_with["node-version-file"] == ".node-version" && !setup_with.key?("node-version")
      abort "#{path}: setup-node must use only node-version-file: .node-version"
    end
  end
end
RUBY
}

# D-100 D6: npm security monitor runs daily (D-030 standing guard).
validate_monitor_schedule() {
    ruby - "$1" <<'RUBY'
require "yaml"
workflow = YAML.safe_load(File.read(ARGV.fetch(0)), aliases: true)
schedule = Array((workflow["on"] || workflow[true] || {})["schedule"])
abort "monitor must have exactly one schedule: #{schedule.inspect}" unless schedule.length == 1
cron = schedule.first["cron"].to_s
abort "monitor cron must be daily: #{cron.inspect}" unless cron.match?(/\A[0-9]+ [0-9]+ \* \* \*\z/)
RUBY
}

validate_claude_hook_audit() {
    local workflow="${1:-$WORKFLOW}"
    grep -Fq 'run: bash scripts/tests/claude-hooks.test.sh' "$workflow"
}

validate_public_actions_doc_contract() {
    local ci_doc="${1:-$CI_DOC}"
    local dev_workflow_doc="${2:-$DEV_WORKFLOW_DOC}"
    local decision_log="${3:-$DECISION_LOG}"
    local plans_doc="${4:-$PLANS_DOC}"
    local project_handoff_doc="${5:-$PROJECT_HANDOFF_DOC}"

    [ -f "$ci_doc" ] && [ -f "$dev_workflow_doc" ] && [ -f "$decision_log" ] &&
        [ -f "$plans_doc" ] && [ -f "$project_handoff_doc" ] || return 1

    if grep -Eq '75%|90%|月間 billed minutes|枠 reset|^## Budget Pressure$' \
        "$ci_doc" "$dev_workflow_doc" "$plans_doc" "$project_handoff_doc"; then
        return 1
    fi

    grep -Fq 'CI-PUBLIC-D1:' "$ci_doc" || return 1
    grep -Fq 'CI-TRIGGER-D1:' "$ci_doc" || return 1
    # D-145 (9): the same-head run (Draft included) is final; Ready does not rerun.
    grep -Fq '| docsを含む全PR | push（opened・synchronize）、再開はreopened。Ready では回し直さない | dispatch しない |' "$ci_doc" || return 1
    grep -Fq '同一 HEAD に final run が無いこと、または run の失敗/cancel' "$ci_doc" || return 1
    grep -Fq '| required final の自動 run または explicit dispatch が作成されない、失敗、または cancel | 原因是正後のrecovery dispatch | 同一 HEAD に、helper が受理する final run も、進行中の run も無いこと |' "$ci_doc" || return 1
    grep -Fq '| 同一 HEAD に helper が受理する final run が既にある | 既存runを使う | Ready再操作もdispatchも不要 |' "$ci_doc" || return 1
    grep -Fq 'GitHub Actions が利用不能なら merge を停止' "$ci_doc" || return 1
    grep -Fq 'CI-TRIGGER-D1' "$dev_workflow_doc" || return 1
    grep -Fxq '## D-063' "$decision_log"
}

# D-157: Windows observe reports cargo results but never fails the job or joins Merge gate.
validate_windows_observe() {
    ruby - "$1" <<'RUBY'
require "yaml"
text = File.read(ARGV.fetch(0))
workflow = YAML.safe_load(text, aliases: true)
abort "observe must not use secrets" if text.include?("secrets.") || text.include?("GITHUB_TOKEN")
abort "observe must not use npm or Node" if text.match?(/\bnpm\b|setup-node/)
abort "observe must not retry or skip tests" if text.match?(/retry|--skip/i)
abort "observe workflow name drifted" unless workflow["name"] == "Windows observe"
triggers = workflow["on"] || workflow[true]
abort "observe triggers drifted" unless triggers.is_a?(Hash) && triggers.keys.map(&:to_s).sort == %w[pull_request workflow_dispatch]
pr = triggers.fetch("pull_request")
abort "observe pull_request drifted" unless pr["branches"] == ["main"] && Array(pr["types"]).sort == %w[opened reopened synchronize]
abort "observe paths drifted" unless pr["paths"] == ["src-tauri/**", ".github/workflows/windows-observe.yml"]
abort "observe token must be read-only" unless workflow["permissions"] == {"contents" => "read"}
abort "observe cancellation drifted" unless workflow.dig("concurrency", "cancel-in-progress") == true
# A group shared with ci.yml would let the two workflows cancel each other.
abort "observe concurrency group drifted" unless workflow.dig("concurrency", "group").to_s.start_with?("windows-observe-")
jobs = workflow.fetch("jobs")
abort "observe must have exactly one job" unless jobs.length == 1
job = jobs.values.first
# The ruleset requires the context "Merge gate"; observe must never publish it.
abort "observe job name drifted: #{job["name"].inspect}" unless job["name"] == "Windows observe"
abort "observe must run on windows-latest" unless job["runs-on"] == "windows-latest"
abort "observe job timeout drifted" unless job["timeout-minutes"] == 60
abort "observe default shell must be bash" unless job.dig("defaults", "run", "shell") == "bash"
steps = job.fetch("steps")
uses = steps.map { |step| step["uses"] }.compact
abort "observe actions drifted: #{uses.inspect}" unless uses == %w[actions/checkout@v6 dtolnay/rust-toolchain@stable actions/cache@v5]
cache = steps.find { |step| step["uses"] == "actions/cache@v5" }
paths = cache.dig("with", "path").to_s.lines.map(&:strip).reject(&:empty?)
abort "observe cache paths drifted: #{paths.inspect}" unless paths == %w[~/.cargo/registry/index/ ~/.cargo/registry/cache/ ~/.cargo/git/db/]
{
  "test" => 'cargo test 2>&1 | tee "$RUNNER_TEMP/cargo-test.log"',
  "clippy" => "cargo clippy --release --all-targets -- -D warnings",
}.each do |id, command|
  step = steps.find { |candidate| candidate["id"] == id }
  abort "observe #{id} step is missing" unless step
  abort "observe #{id} step can fail the job" unless step["continue-on-error"] == true
  abort "observe #{id} step has no step timeout" unless step["timeout-minutes"].is_a?(Integer)
  abort "observe #{id} command drifted" unless step["run"].to_s.strip == command
end
report = steps.last
run = report["run"].to_s
abort "observe report must run always" unless report["if"] == "always()"
%w[steps.test.outcome steps.clippy.outcome].each do |needle|
  abort "observe report lost #{needle}" unless report["env"].to_h.values.any? { |value| value.to_s.include?(needle) }
end
# Run the report as Actions does, on a synthetic failure log: the failed test must reach both outputs.
require "open3"
require "tmpdir"
Dir.mktmpdir do |dir|
  failed = "test some::name ... FAILED"
  File.write(File.join(dir, "cargo-test.log"), "running 1 test\n#{failed}\n")
  summary = File.join(dir, "summary.md")
  script = File.join(dir, "report.sh")
  File.write(summary, "")
  File.write(script, run)
  outcomes = {"${{ steps.test.outcome }}" => "failure", "${{ steps.clippy.outcome }}" => "success"}
  # An expression this check cannot evaluate would run as a literal string and could hide a regression.
  env = report["env"].to_h.transform_values do |value|
    next value.to_s unless value.to_s.include?("${{")
    outcomes.fetch(value.to_s) { abort "observe report env has an expression this check cannot evaluate: #{value}" }
  end
  env = env.merge("RUNNER_TEMP" => dir, "GITHUB_STEP_SUMMARY" => summary)
  stdout, _stderr, status = Open3.capture3(env, "bash", "--noprofile", "--norc", "-eo", "pipefail", script, chdir: dir)
  abort "observe report must exit 0" unless status.success?
  abort "observe report summary lost the failed test name" unless File.read(summary).include?(failed)
  abort "observe report lost the per-test ::warning" unless stdout.include?("::warning title=Windows observe::#{failed}")
  abort "observe report lost the cargo test outcome" unless File.read(summary).include?("- cargo test: failure") && stdout.include?("::warning title=Windows observe::cargo test failed")
end
RUBY
}

# D-164: the L3 smoke runs only for PRs that change it, never fails a product PR, and keeps its red.
# Arguments: workflow, src-tauri dir and smoke script; mutants pass copies.
validate_l3_smoke() {
    local tauri_dir="${2:-$REPO_ROOT/src-tauri}" script="${3:-$L3_SMOKE_SCRIPT}" rc=0
    # The shipped config never opens a WebView2 debugging port; rg 1 is "none", 2 (a bad path) is a failure.
    rg -n -F remote-debugging "$tauri_dir/src" "$tauri_dir"/tauri*.json || rc=$?
    case "$rc" in
        1) ;;
        0) echo "shipped config opens a debugging port" >&2; return 1 ;;
        *) echo "remote-debugging search failed: rg exit $rc" >&2; return 1 ;;
    esac
    ruby - "$1" "$script" <<'RUBY'
require "yaml"
text = File.read(ARGV.fetch(0))
workflow = YAML.safe_load(text, aliases: true)
abort "smoke must not use secrets" if text.include?("secrets.") || text.include?("GITHUB_TOKEN")
abort "smoke must not hide its red" if text.match?(/continue-on-error|retry/i)
abort "smoke must not install from git" if text.include?("--git")
abort "smoke workflow name drifted" unless workflow["name"] == "L3 smoke"
triggers = workflow["on"] || workflow[true]
# D-015: no push or workflow_dispatch, so only the paths below start it.
abort "smoke triggers drifted" unless triggers.is_a?(Hash) && triggers.keys.map(&:to_s) == %w[pull_request]
pr = triggers.fetch("pull_request")
abort "smoke pull_request drifted" unless pr["branches"] == ["main"] && Array(pr["types"]).sort == %w[opened reopened synchronize]
smoke_paths = [".github/workflows/l3-smoke.yml", "scripts/ci/l3-smoke-home.mjs"]
abort "smoke paths drifted: #{pr["paths"].inspect}" unless pr["paths"] == smoke_paths && !pr.key?("paths-ignore")
abort "smoke token must be read-only" unless workflow["permissions"] == {"contents" => "read"}
abort "smoke cancellation drifted" unless workflow.dig("concurrency", "cancel-in-progress") == true
abort "smoke concurrency group drifted" unless workflow.dig("concurrency", "group").to_s.start_with?("l3-smoke-")
jobs = workflow.fetch("jobs")
abort "smoke must have exactly one job" unless jobs.length == 1
job = jobs.values.first
# The ruleset requires the context "Merge gate"; the smoke must never publish it.
abort "smoke job name drifted: #{job["name"].inspect}" unless job["name"] == "L3 smoke"
abort "smoke must run on windows-latest" unless job["runs-on"] == "windows-latest"
job_timeout = job["timeout-minutes"]
abort "smoke job timeout must be an integer" unless job_timeout.is_a?(Integer) && job_timeout.positive?
steps = job.fetch("steps")
uses = steps.map { |step| step["uses"] }.compact
abort "smoke actions drifted: #{uses.inspect}" unless uses == %w[actions/checkout@v6 actions/setup-node@v6 dtolnay/rust-toolchain@stable actions/cache@v5]
abort "smoke checkout needs the merge base" unless steps.first["uses"] == "actions/checkout@v6" && steps.first.dig("with", "fetch-depth") == 0
diff_index = steps.index { |step| step["id"] == "diff" }
abort "smoke diff step is missing" unless diff_index == 1
diff = steps[diff_index]
abort "smoke diff base/head drifted" unless diff["env"] == {
  "BASE_SHA" => "${{ github.event.pull_request.base.sha }}",
  "HEAD_SHA" => "${{ github.event.pull_request.head.sha }}",
}
abort "smoke diff must read the same files as paths" unless diff["run"].to_s.include?(%(git diff --name-only "$BASE_SHA...$HEAD_SHA" -- #{smoke_paths.join(" ")}))
later = steps[(diff_index + 1)..]
abort "smoke step runs without the diff guard" unless later.all? { |step| step["if"] == "steps.diff.outputs.changed == 'true'" }
cache = steps.find { |step| step["uses"] == "actions/cache@v5" }
cache_paths = cache.dig("with", "path").to_s.lines.map(&:strip).reject(&:empty?)
abort "smoke cache paths drifted: #{cache_paths.inspect}" unless cache_paths == %w[~/.cargo/registry/index/ ~/.cargo/registry/cache/ ~/.cargo/git/db/]
runs = steps.map { |step| step["run"].to_s.strip }
["npm ci --ignore-scripts", "cargo install tauri-driver --locked --version 2.1.0"].each do |command|
  abort "smoke command drifted: #{command}" unless runs.include?(command)
end
# The smoke-only config file is written and passed with --config; the shipped config stays as it is.
build_run = <<~'RUN'
  node -e 'const fs = require("fs"); const c = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8")); for (const w of c.app.windows) w.additionalBrowserArgs = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --remote-debugging-port=0"; fs.writeFileSync(process.argv[1], JSON.stringify({ app: { windows: c.app.windows } }));' "$RUNNER_TEMP/l3-smoke.conf.json"
  npm run tauri -- build --no-bundle --config "$RUNNER_TEMP/l3-smoke.conf.json"
RUN
build = steps.find { |step| step["name"] == "Build the release app" }
abort "smoke command drifted: build" unless build && build["run"] == build_run
# The verdict is the result file after cleanup: a stale file fails, and -e must not end the step at node.
smoke_run = <<~'RUN'
  if [ -e "$L3_SMOKE_RESULT" ]; then echo "L3 smoke: FAIL: the result file existed before the run"; exit 1; fi
  rc=0; node scripts/ci/l3-smoke-home.mjs || rc=$?
  echo "L3 smoke: node exit code $rc"
  if [ "$(cat "$L3_SMOKE_RESULT" 2>/dev/null)" != "PASS" ]; then echo "L3 smoke: FAIL: no PASS in the result file (node exit code $rc)"; exit 1; fi
RUN
smoke = steps.find { |step| step["name"] == "Check the home heading" }
abort "smoke step is missing" unless smoke
abort "smoke step run drifted" unless smoke["run"] == smoke_run
abort "smoke result file drifted" unless smoke.dig("env", "L3_SMOKE_RESULT") == "${{ runner.temp }}/l3-smoke-result"
step_timeout = smoke["timeout-minutes"]
abort "smoke step timeout must be an integer below the job timeout" unless step_timeout.is_a?(Integer) && step_timeout.positive? && step_timeout < job_timeout
# S2: PASS is written once, after cleanup and only when it finished, just before the last exit.
script = File.read(ARGV.fetch(1))
write = 'if (code === 0 && cleanedUp) writeFileSync(RESULT_FILE, "PASS");'
lines = script.lines.map(&:chomp)
last_exit = lines.rindex("process.exit(code);")
abort "smoke result write drifted" unless last_exit && lines[last_exit - 1] == write
abort "smoke result is written elsewhere" unless script.scan("writeFileSync(").length == 1 && script.scan("L3_SMOKE_RESULT").length == 1
abort "smoke RESULT_FILE must be used only by its definition and the write" unless script.scan("RESULT_FILE").length == 2
RUBY
}

# MG-D4: real PR PK5 and every former local workflow check remain wired on both routes.
validate_parity() {
    ruby - "$1" "$2" "$3" <<'RUBY'
require "yaml"
workflow = YAML.safe_load(File.read(ARGV[0]), aliases: true)
local = File.read(ARGV[1]); suite = File.read(ARGV[2])
jobs = workflow.fetch("jobs")
call = "bash scripts/tests/run-workflow-tests.sh"
abort "shared suite disconnected from local" unless local.include?(call)
abort "shared suite disconnected from hosted" unless jobs.fetch("workflow").fetch("steps").any? { |step| step["run"] == call }
%w[classify-changes check-command-drift pre-push local-ci codex-safe-wrappers claude-hooks public-sanitization doc-consistency-plan-packet workflow-git-checks plan-approve reading-order-drift ci-workflow merge-gate].each do |name|
  abort "lost workflow regression: #{name}" unless suite.include?("bash scripts/tests/#{name}.test.sh")
end
abort "helper tests disconnected" unless suite.include?("python3 scripts/tests/pr-gate.test.py")
abort "shell syntax lost" unless suite.include?('bash -n "$shell_file"')
abort "workflow YAML lost" unless suite.include?("YAML.parse_file")
docs = jobs.fetch("docs").fetch("steps")
checkout = docs.find { |step| step["uses"] == "actions/checkout@v6" }
abort "PK5 needs complete history" unless checkout.dig("with", "fetch-depth") == 0
abort "PK5 must check PR head" unless checkout.dig("with", "ref") == "${{ github.event.pull_request.head.sha || github.sha }}"
check = docs.find { |step| step["run"] == "bash scripts/check-workflow-git.sh" }
abort "actual PR PK5 disconnected" unless check && check.dig("env", "WORKFLOW_BASE_SHA") == "${{ github.event.pull_request.base.sha || github.sha }}"
# CI4 / D-145 (5)(10): the doc check measures active packets and new path:line references from the PR base.
doc = docs.find { |step| step["run"] == "bash scripts/doc-consistency-check.sh" }
abort "doc check has no WORKFLOW_BASE_SHA" unless doc && doc.dig("env", "WORKFLOW_BASE_SHA") == "${{ github.event.pull_request.base.sha || github.sha }}"
workflow_checkout = jobs.fetch("workflow").fetch("steps").find { |step| step["uses"] == "actions/checkout@v6" }
abort "workflow suite self-pass needs complete history" unless workflow_checkout.dig("with", "fetch-depth") == 0
%w[rust_lint rust_test rust_drift frontend env_safety].each do |name|
  abort "policy regression must not force product gates" if jobs.fetch(name).fetch("if").include?("outputs.workflow")
end
RUBY
}
validate_parity "$WORKFLOW" "$REPO_ROOT/scripts/local-ci.sh" "$REPO_ROOT/scripts/tests/run-workflow-tests.sh"

validate_job_graph "$WORKFLOW"
validate_workflow_contract "$WORKFLOW"
validate_node_contract
validate_monitor_schedule "$NPM_SECURITY_WORKFLOW"
validate_windows_observe "$WINDOWS_OBSERVE_WORKFLOW"
validate_l3_smoke "$L3_SMOKE_WORKFLOW"
# D-059 / SPEC-HOOK-01 / CH8: hosted workflow owns the inventory audit step.
validate_claude_hook_audit ||
    fail "CH8 hosted Claude hook audit step is missing"
# D-063 / SPEC-WF-CI-PUBLIC-D1: live CI docs own the public-runner contract.
validate_public_actions_doc_contract "$CI_DOC" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" ||
    fail "public Actions docs contract drifted"

mutation_dir="$(mktemp -d)"
trap 'rm -rf "$mutation_dir"' EXIT

# A sed that no longer matches leaves the original bytes and would pass green.
assert_mutated() {
    if cmp -s "$1" "$2"; then
        fail "mutation did not change $1: $3"
    fi
}

parity_mutation="$mutation_dir/missing-suite-check.sh"
sed '\|bash scripts/tests/pre-push.test.sh|d' "$REPO_ROOT/scripts/tests/run-workflow-tests.sh" > "$parity_mutation"
if validate_parity "$WORKFLOW" "$REPO_ROOT/scripts/local-ci.sh" "$parity_mutation" >/dev/null 2>&1; then
    fail "parity accepted a lost workflow regression"
fi
shallow_mutation="$mutation_dir/shallow-ci.yml"
sed 's/fetch-depth: 0/fetch-depth: 1/g' "$WORKFLOW" > "$shallow_mutation"
if validate_parity "$shallow_mutation" "$REPO_ROOT/scripts/local-ci.sh" "$REPO_ROOT/scripts/tests/run-workflow-tests.sh" >/dev/null 2>&1; then
    fail "parity accepted shallow PK5"
fi

ci_doc_quota_mutation="$mutation_dir/ci-private-quota.md"
cp "$CI_DOC" "$ci_doc_quota_mutation"
printf '%s\n' 'At 75% monthly usage, stop normal hosted CI.' >> "$ci_doc_quota_mutation"
if validate_public_actions_doc_contract "$ci_doc_quota_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "M1 public Actions docs validator accepted private quota wording"
fi

ci_doc_event_dispatch_mutation="$mutation_dir/ci-event-dispatch.md"
sed 's/| dispatch しない |/| workflow_dispatch してよい |/' "$CI_DOC" > "$ci_doc_event_dispatch_mutation"
assert_mutated "$CI_DOC" "$ci_doc_event_dispatch_mutation" M2a
if validate_public_actions_doc_contract "$ci_doc_event_dispatch_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "M2a public Actions docs validator accepted preventive dispatch for an event-eligible change"
fi

ci_doc_zero_run_mutation="$mutation_dir/ci-zero-run-removed.md"
sed 's/同一 HEAD に final run が無いこと/同一 HEAD の run 確認は不要/' "$CI_DOC" > "$ci_doc_zero_run_mutation"
assert_mutated "$CI_DOC" "$ci_doc_zero_run_mutation" M2b
if validate_public_actions_doc_contract "$ci_doc_zero_run_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "M2b public Actions docs validator accepted dispatch without the no-final-run prerequisite"
fi

ci_doc_recovery_scope_mutation="$mutation_dir/ci-recovery-auto-only.md"
sed 's/required final の自動 run または explicit dispatch/event-eligible だが自動 run/' "$CI_DOC" > "$ci_doc_recovery_scope_mutation"
assert_mutated "$CI_DOC" "$ci_doc_recovery_scope_mutation" M2c-scope
if validate_public_actions_doc_contract "$ci_doc_recovery_scope_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "M2c public Actions docs validator accepted recovery narrowed to automatic events"
fi

ci_doc_in_progress_mutation="$mutation_dir/ci-in-progress-wait-removed.md"
sed 's/、進行中の run も//' "$CI_DOC" > "$ci_doc_in_progress_mutation"
assert_mutated "$CI_DOC" "$ci_doc_in_progress_mutation" M2c-in-progress
if validate_public_actions_doc_contract "$ci_doc_in_progress_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "M2c public Actions docs validator accepted recovery without the in-progress wait"
fi

ci_doc_successful_row_mutation="$mutation_dir/ci-successful-row-removed.md"
sed '/同一 HEAD に helper が受理する final run が既にある/d' "$CI_DOC" > "$ci_doc_successful_row_mutation"
assert_mutated "$CI_DOC" "$ci_doc_successful_row_mutation" M2d
if validate_public_actions_doc_contract "$ci_doc_successful_row_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "M2d public Actions docs validator accepted a missing already-successful no-op row"
fi

ci_doc_availability_mutation="$mutation_dir/ci-actions-unavailable-stop-removed.md"
sed '/GitHub Actions が利用不能なら merge を停止/d' "$CI_DOC" > "$ci_doc_availability_mutation"
assert_mutated "$CI_DOC" "$ci_doc_availability_mutation" M3
if validate_public_actions_doc_contract "$ci_doc_availability_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "M3 public Actions docs validator accepted a missing merge stop while Actions is unavailable"
fi

ci_doc_public_anchor_mutation="$mutation_dir/ci-public-anchor-removed.md"
sed 's/CI-PUBLIC-D1/CI-PUBLIC-REMOVED/g' "$CI_DOC" > "$ci_doc_public_anchor_mutation"
if validate_public_actions_doc_contract "$ci_doc_public_anchor_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "public Actions docs validator accepted a missing CI-PUBLIC-D1 anchor"
fi

ci_doc_trigger_anchor_mutation="$mutation_dir/ci-trigger-anchor-removed.md"
sed 's/CI-TRIGGER-D1/CI-TRIGGER-REMOVED/g' "$CI_DOC" > "$ci_doc_trigger_anchor_mutation"
if validate_public_actions_doc_contract "$ci_doc_trigger_anchor_mutation" "$DEV_WORKFLOW_DOC" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "public Actions docs validator accepted a missing CI-TRIGGER-D1 anchor"
fi

dev_workflow_trigger_mutation="$mutation_dir/dev-workflow-trigger-reference-removed.md"
sed 's/CI-TRIGGER-D1/CI-TRIGGER-REMOVED/g' "$DEV_WORKFLOW_DOC" > "$dev_workflow_trigger_mutation"
if validate_public_actions_doc_contract "$CI_DOC" "$dev_workflow_trigger_mutation" "$DECISION_LOG" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "public Actions docs validator accepted a missing DEV_WORKFLOW CI-TRIGGER-D1 reference"
fi

decision_log_d063_mutation="$mutation_dir/decision-log-d063-removed.md"
sed 's/^## D-063$/## D-063-REMOVED/' "$DECISION_LOG" > "$decision_log_d063_mutation"
if validate_public_actions_doc_contract "$CI_DOC" "$DEV_WORKFLOW_DOC" "$decision_log_d063_mutation" "$PLANS_DOC" "$PROJECT_HANDOFF_DOC" >/dev/null 2>&1; then
    fail "public Actions docs validator accepted a missing D-063 decision"
fi

m4_root="$mutation_dir/m4-root"
mkdir -p "$m4_root/docs/archive"
cp "$CI_DOC" "$m4_root/docs/ci.md"
cp "$DEV_WORKFLOW_DOC" "$m4_root/docs/DEV_WORKFLOW.md"
cp "$DECISION_LOG" "$m4_root/docs/decision-log.md"
cp "$PLANS_DOC" "$m4_root/docs/Plans.md"
cp "$PROJECT_HANDOFF_DOC" "$m4_root/docs/PROJECT_HANDOFF.md"
printf '%s\n' 'Historical policy: 75% and 90% private quota thresholds.' > "$m4_root/docs/archive/private-ci-history.md"
grep -Fq '75%' "$m4_root/docs/archive/private-ci-history.md" ||
    fail "M4 archive fixture is missing its historical private quota wording"
validate_public_actions_doc_contract \
    "$m4_root/docs/ci.md" \
    "$m4_root/docs/DEV_WORKFLOW.md" \
    "$m4_root/docs/decision-log.md" \
    "$m4_root/docs/Plans.md" \
    "$m4_root/docs/PROJECT_HANDOFF.md" ||
    fail "M4 public Actions docs validator scanned archive history as current policy"

pin_mutation="$mutation_dir/.node-version"
printf '%s\n' "25.8.2" > "$pin_mutation"
if validate_node_contract "$pin_mutation" >/dev/null 2>&1; then
    fail "Node contract validator accepted a drifted exact pin"
fi

ci_node_mutation="$mutation_dir/ci-node20.yml"
sed 's/node-version-file: .node-version/node-version: 20/' "$WORKFLOW" > "$ci_node_mutation"
if validate_node_contract "$NODE_VERSION_FILE" "$PACKAGE_JSON" "$ci_node_mutation" "$NPM_SECURITY_WORKFLOW" >/dev/null 2>&1; then
    fail "Node contract validator accepted literal Node 20 in ci.yml"
fi

npm_security_node_mutation="$mutation_dir/npm-security-node20.yml"
sed 's/node-version-file: .node-version/node-version: 20/' "$NPM_SECURITY_WORKFLOW" > "$npm_security_node_mutation"
if validate_node_contract "$NODE_VERSION_FILE" "$PACKAGE_JSON" "$WORKFLOW" "$npm_security_node_mutation" >/dev/null 2>&1; then
    fail "Node contract validator accepted literal Node 20 in npm-security-monitor.yml"
fi

engines_mutation="$mutation_dir/engines-node25.json"
sed '0,/"node": ">=24 <25"/s//"node": ">=25 <26"/' "$PACKAGE_JSON" > "$engines_mutation"
if validate_node_contract "$NODE_VERSION_FILE" "$engines_mutation" >/dev/null 2>&1; then
    fail "Node contract validator accepted a drifted engines.node major"
fi

dev_engines_mutation="$mutation_dir/dev-engines-node25.json"
sed '0,/"version": ">=24 <25"/s//"version": ">=25 <26"/' "$PACKAGE_JSON" > "$dev_engines_mutation"
if validate_node_contract "$NODE_VERSION_FILE" "$dev_engines_mutation" >/dev/null 2>&1; then
    fail "Node contract validator accepted a drifted devEngines.runtime major"
fi

types_node_mutation="$mutation_dir/types-node25.json"
sed 's/"@types\/node": "\^24\.[0-9][0-9.]*"/"@types\/node": "^25.0.0"/' "$PACKAGE_JSON" > "$types_node_mutation"
if validate_node_contract "$NODE_VERSION_FILE" "$types_node_mutation" >/dev/null 2>&1; then
    fail "Node contract validator accepted a drifted @types/node major"
fi

job_graph_mutation="$mutation_dir/unguarded-job.yml"
cp "$WORKFLOW" "$job_graph_mutation"
cat >> "$job_graph_mutation" <<'YAML'
  unguarded_probe:
    runs-on: ubuntu-latest
    steps:
      - run: echo unsafe
YAML
if validate_job_graph "$job_graph_mutation" >/dev/null 2>&1; then
    fail "job graph validator accepted an unguarded runner job"
fi

missing_event_mutation="$mutation_dir/missing-synchronize.yml"
sed 's/, synchronize//' "$WORKFLOW" > "$missing_event_mutation"
if validate_workflow_contract "$missing_event_mutation" >/dev/null 2>&1; then
    fail "pull_request event validator accepted missing synchronize"
fi

extra_event_mutation="$mutation_dir/extra-event.yml"
sed 's/synchronize/synchronize, edited/' "$WORKFLOW" > "$extra_event_mutation"
if validate_workflow_contract "$extra_event_mutation" >/dev/null 2>&1; then
    fail "pull_request event validator accepted an extra event"
fi

# CI1 / D-145 (9): rerunning on ready_for_review would run the same head twice.
ready_mutation="$mutation_dir/ready-for-review.yml"
sed 's/types: \[opened, reopened, synchronize\]/types: [opened, reopened, ready_for_review, synchronize]/' "$WORKFLOW" > "$ready_mutation"
assert_mutated "$WORKFLOW" "$ready_mutation" ready-for-review
if validate_workflow_contract "$ready_mutation" >/dev/null 2>&1; then
    fail "workflow contract validator accepted a ready_for_review rerun"
fi

# MG-D2/D3: a classification failure must fail, never skip the aggregate.
aggregate_mutation="$mutation_dir/classification-success-guard.yml"
sed "s/^    if: always()\$/    if: always() \&\& needs.changes.result == 'success'/" "$WORKFLOW" > "$aggregate_mutation"
assert_mutated "$WORKFLOW" "$aggregate_mutation" aggregate-success-guard
if validate_job_graph "$aggregate_mutation" >/dev/null 2>&1; then
    fail "aggregate accepted a changes-success precondition"
fi
# D-100: Draft must run changes and the aggregate.
changes_draft_mutation="$mutation_dir/changes-draft-guard.yml"
sed '/^    name: Detect changed areas$/a\    if: github.event.pull_request.draft == false' "$WORKFLOW" > "$changes_draft_mutation"
assert_mutated "$WORKFLOW" "$changes_draft_mutation" changes-draft-guard
if validate_workflow_contract "$changes_draft_mutation" >/dev/null 2>&1; then
    fail "changes job accepted a Draft guard"
fi
aggregate_draft_mutation="$mutation_dir/aggregate-draft-guard.yml"
sed "s/^    if: always()\$/    if: always() \&\& (github.event_name == 'workflow_dispatch' || github.event.pull_request.draft == false)/" "$WORKFLOW" > "$aggregate_draft_mutation"
assert_mutated "$WORKFLOW" "$aggregate_draft_mutation" aggregate-draft-guard
if validate_job_graph "$aggregate_draft_mutation" >/dev/null 2>&1; then
    fail "aggregate accepted a Draft guard"
fi
# CI1 (rewritten from "Draft can publish required check name"): a Draft-only name would hide the same-head evidence.
name_mutation="$mutation_dir/draft-name.yml"
sed "s/^    name: Merge gate\$/    name: \${{ github.event.pull_request.draft \&\& 'Draft (no merge evidence)' || 'Merge gate' }}/" "$WORKFLOW" > "$name_mutation"
assert_mutated "$WORKFLOW" "$name_mutation" draft-name
if validate_workflow_contract "$name_mutation" >/dev/null 2>&1; then
    fail "Draft runs can publish another aggregate name"
fi
# CI4: dropping the doc check's base breaks the active-packet and path:line origin.
doc_base_mutation="$mutation_dir/doc-base.yml"
awk '/name: Run doc consistency check/{skip=1} skip && /WORKFLOW_BASE_SHA/{skip=0; next} skip && /^        env:$/{next} {print}' "$WORKFLOW" > "$doc_base_mutation"
assert_mutated "$WORKFLOW" "$doc_base_mutation" doc-base
if validate_parity "$doc_base_mutation" "$REPO_ROOT/scripts/local-ci.sh" "$REPO_ROOT/scripts/tests/run-workflow-tests.sh" >/dev/null 2>&1; then
    fail "doc check without WORKFLOW_BASE_SHA was accepted"
fi

concurrency_mutation="$mutation_dir/disabled-cancellation.yml"
sed 's/cancel-in-progress: true/cancel-in-progress: false # cancel-in-progress: true/' "$WORKFLOW" > "$concurrency_mutation"
if validate_workflow_contract "$concurrency_mutation" >/dev/null 2>&1; then
    fail "workflow contract validator accepted disabled cancellation"
fi

head_sha_mutation="$mutation_dir/wrong-head-sha.yml"
sed 's|--head "${{ github.event.pull_request.head.sha }}"|--head "${{ github.sha }}" # --head "${{ github.event.pull_request.head.sha }}"|' "$WORKFLOW" > "$head_sha_mutation"
if validate_workflow_contract "$head_sha_mutation" >/dev/null 2>&1; then
    fail "workflow contract validator accepted wrong head SHA routing"
fi

push_mutation="$mutation_dir/quoted-push-trigger.yml"
sed '/^on:$/a\  "push": { branches: [main] }' "$WORKFLOW" > "$push_mutation"
if validate_workflow_contract "$push_mutation" >/dev/null 2>&1; then
    fail "workflow contract validator accepted a push trigger"
fi

merge_group_mutation="$mutation_dir/merge-group-trigger.yml"
sed '/^on:$/a\  merge_group:' "$WORKFLOW" > "$merge_group_mutation"
if validate_workflow_contract "$merge_group_mutation" >/dev/null 2>&1; then
    fail "workflow contract validator accepted an extra top-level trigger"
fi

monitor_weekly_mutation="$mutation_dir/monitor-weekly.yml"
sed 's/cron: "0 21 \* \* \*"/cron: "0 21 * * 0"/' "$NPM_SECURITY_WORKFLOW" > "$monitor_weekly_mutation"
assert_mutated "$NPM_SECURITY_WORKFLOW" "$monitor_weekly_mutation" monitor-weekly
if validate_monitor_schedule "$monitor_weekly_mutation" >/dev/null 2>&1; then
    fail "monitor schedule validator accepted a weekly cron"
fi

claude_hook_step_mutation="$mutation_dir/missing-claude-hook-audit.yml"
sed '\|run: bash scripts/tests/claude-hooks.test.sh|d' "$WORKFLOW" > "$claude_hook_step_mutation"
if validate_claude_hook_audit "$claude_hook_step_mutation" >/dev/null 2>&1; then
    fail "CH9 workflow validator accepted a missing Claude hook audit step"
fi

# D-157: every observe guarantee in the Test Plan rejects its mutant.
reject_observe_mutant() {
    assert_mutated "$WINDOWS_OBSERVE_WORKFLOW" "$2" "$1"
    if validate_windows_observe "$2" >/dev/null 2>&1; then
        fail "Windows observe validator accepted $1"
    fi
}
observe_mutant() {
    local out="$mutation_dir/$1.yml"
    shift
    sed "$@" "$WINDOWS_OBSERVE_WORKFLOW" > "$out"
    printf '%s' "$out"
}
reject_observe_mutant windows-observe-no-continue "$(observe_mutant no-continue '/continue-on-error: true/d')"
reject_observe_mutant windows-observe-no-report "$(observe_mutant no-report '/if: always()/d')"
reject_observe_mutant windows-observe-no-tee "$(observe_mutant no-tee 's/ 2>&1 | tee "$RUNNER_TEMP\/cargo-test.log"//')"
reject_observe_mutant windows-observe-no-tee-extract "$(observe_mutant no-tee-extract '/grep -E/s/FAILED/failed/')"
reject_observe_mutant windows-observe-no-bash "$(observe_mutant no-bash '/shell: bash/d')"
reject_observe_mutant windows-observe-no-step-timeout "$(observe_mutant no-step-timeout '/timeout-minutes: 30/d')"
reject_observe_mutant windows-observe-merge-gate-name "$(observe_mutant merge-gate-name 's/^    name: Windows observe$/    name: Merge gate/')"
reject_observe_mutant windows-observe-write-permission "$(observe_mutant write-permission 's/contents: read/contents: write/')"
reject_observe_mutant windows-observe-secret "$(observe_mutant secret '/RUST_BACKTRACE: 1/a\  TOKEN: ${{ secrets.OBSERVE_TOKEN }}')"
reject_observe_mutant windows-observe-skip "$(observe_mutant skip '/cargo test 2>&1/a\          cargo test -- --skip slow')"
reject_observe_mutant windows-observe-retry "$(observe_mutant retry 's|uses: actions/cache@v5|uses: nick-fields/retry@v3|')"
reject_observe_mutant windows-observe-target-cache "$(observe_mutant target-cache '\|~/.cargo/git/db/|a\            src-tauri/target/')"
reject_observe_mutant windows-observe-failure-probe "$(observe_mutant failure-probe "/cargo test 2>&1/i\\          echo 'test observe_probe::synthetic ... FAILED' | tee \"\$RUNNER_TEMP/cargo-test.log\"; exit 1")"
reject_observe_mutant windows-observe-no-summary-name "$(observe_mutant no-summary-name '/printf/s/printf/:/')"
reject_observe_mutant windows-observe-no-test-warning "$(observe_mutant no-test-warning 's/echo "::warning title=Windows observe::$line"/:/')"
reject_observe_mutant windows-observe-shared-group "$(observe_mutant shared-group 's/group: windows-observe-/group: ci-/')"
reject_observe_mutant windows-observe-unknown-outcome-expr "$(observe_mutant unknown-outcome-expr "s/TEST_OUTCOME: \${{ steps.test.outcome }}/TEST_OUTCOME: \${{ steps.test.outcome || 'skipped' }}/")"
reject_observe_mutant windows-observe-no-outcome-warning "$(observe_mutant no-outcome-warning 's/echo "::warning title=Windows observe::cargo test failed"/:/')"

# D-164: every smoke guarantee in the Test Plan rejects its mutant.
reject_smoke_mutant() {
    assert_mutated "$L3_SMOKE_WORKFLOW" "$2" "$1"
    if validate_l3_smoke "$2" >/dev/null 2>&1; then
        fail "L3 smoke validator accepted $1"
    fi
}
smoke_mutant() {
    local out="$mutation_dir/$1.yml"
    shift
    sed "$@" "$L3_SMOKE_WORKFLOW" > "$out"
    printf '%s' "$out"
}
reject_smoke_mutant l3-smoke-product-path "$(smoke_mutant product-path '/- "scripts\/ci\/l3-smoke-home.mjs"/a\      - "src-tauri/**"')"
reject_smoke_mutant l3-smoke-extra-trigger "$(smoke_mutant extra-trigger '/^on:$/a\  workflow_dispatch:')"
reject_smoke_mutant l3-smoke-no-diff-guard "$(smoke_mutant no-diff-guard '/name: Build the release app/{n;d}')"
reject_smoke_mutant l3-smoke-diff-paths "$(smoke_mutant diff-paths '/git diff --name-only/s|scripts/ci/l3-smoke-home.mjs|scripts/ci/other.mjs|')"
reject_smoke_mutant l3-smoke-continue "$(smoke_mutant continue '/timeout-minutes: 10/a\        continue-on-error: true')"
reject_smoke_mutant l3-smoke-no-step-timeout "$(smoke_mutant no-step-timeout '/timeout-minutes: 10/d')"
reject_smoke_mutant l3-smoke-merge-gate-name "$(smoke_mutant merge-gate-name 's/^    name: L3 smoke$/    name: Merge gate/')"
reject_smoke_mutant l3-smoke-write-permission "$(smoke_mutant write-permission 's/contents: read/contents: write/')"
reject_smoke_mutant l3-smoke-secret "$(smoke_mutant secret '/RUST_BACKTRACE: 1/a\  TOKEN: ${{ secrets.SMOKE_TOKEN }}')"
reject_smoke_mutant l3-smoke-unpinned-driver "$(smoke_mutant unpinned-driver 's/ --version 2.1.0//')"
reject_smoke_mutant l3-smoke-git-driver "$(smoke_mutant git-driver '/cargo install tauri-driver/a\          cargo install --git https://github.com/chippers/msedgedriver-tool')"
reject_smoke_mutant l3-smoke-npm-scripts "$(smoke_mutant npm-scripts 's/npm ci --ignore-scripts/npm ci/')"
reject_smoke_mutant l3-smoke-target-cache "$(smoke_mutant target-cache '\|~/.cargo/git/db/|a\            src-tauri/target/')"
reject_smoke_mutant l3-smoke-no-config "$(smoke_mutant no-config '/npm run tauri/s| --config "$RUNNER_TEMP/l3-smoke.conf.json"||')"
reject_smoke_mutant l3-smoke-exit-code-verdict "$(smoke_mutant exit-code-verdict -e '/if \[ -e "$L3_SMOKE_RESULT"/d' -e '/node exit code/d' -e 's/rc=0; node scripts\/ci\/l3-smoke-home.mjs || rc=$?/node scripts\/ci\/l3-smoke-home.mjs/')"
reject_smoke_mutant l3-smoke-stale-result "$(smoke_mutant stale-result '/if \[ -e "$L3_SMOKE_RESULT"/d')"
reject_smoke_mutant l3-smoke-errexit "$(smoke_mutant errexit 's/rc=0; node scripts\/ci\/l3-smoke-home.mjs || rc=$?/node scripts\/ci\/l3-smoke-home.mjs; rc=$?/')"
reject_smoke_script_mutant() {
    assert_mutated "$L3_SMOKE_SCRIPT" "$2" "$1"
    if validate_l3_smoke "$L3_SMOKE_WORKFLOW" "$REPO_ROOT/src-tauri" "$2" >/dev/null 2>&1; then
        fail "L3 smoke validator accepted $1"
    fi
}
smoke_script_mutant() {
    local out="$mutation_dir/$1.mjs"
    shift
    sed "$@" "$L3_SMOKE_SCRIPT" > "$out"
    printf '%s' "$out"
}
smoke_write='if (code === 0 \&\& cleanedUp) writeFileSync(RESULT_FILE, "PASS");'
reject_smoke_script_mutant l3-smoke-early-result "$(smoke_script_mutant early-result -e "/^if (code === 0 && cleanedUp) writeFileSync/d" -e "/^    code = 0;\$/a\\    $smoke_write")"
reject_smoke_script_mutant l3-smoke-unconditional-result "$(smoke_script_mutant unconditional-result 's/ && cleanedUp) writeFileSync/) writeFileSync/')"
# The copy passes before the port is added, so the rejection comes from the port and not from a bad path.
shipped_copy="$mutation_dir/src-tauri"
mkdir -p "$shipped_copy"
cp -R "$REPO_ROOT/src-tauri/src" "$REPO_ROOT"/src-tauri/tauri*.json "$shipped_copy/"
validate_l3_smoke "$L3_SMOKE_WORKFLOW" "$shipped_copy" >/dev/null 2>&1 ||
    fail "L3 smoke validator rejected an unchanged src-tauri copy"
printf '%s\n' '{ "app": { "windows": [{ "additionalBrowserArgs": "--remote-debugging-port=0" }] } }' > "$shipped_copy/tauri.windows.conf.json"
if validate_l3_smoke "$L3_SMOKE_WORKFLOW" "$shipped_copy" >/dev/null 2>&1; then
    fail "L3 smoke validator accepted l3-smoke-shipped-debug-port"
fi
smoke_node_mutation="$(smoke_mutant node24 's/node-version-file: .node-version/node-version: 24/')"
assert_mutated "$L3_SMOKE_WORKFLOW" "$smoke_node_mutation" l3-smoke-node24
if validate_node_contract "$NODE_VERSION_FILE" "$PACKAGE_JSON" "$WORKFLOW" "$NPM_SECURITY_WORKFLOW" "$smoke_node_mutation" >/dev/null 2>&1; then
    fail "Node contract validator accepted literal Node 24 in l3-smoke.yml"
fi

reject_fixed "  push:"
reject_fixed '      - "**/*.md"'
require_fixed "  workflow_dispatch:"
reject_fixed "paths-ignore:"
# Windows runs outside ci.yml (windows-observe.yml for D-157, l3-smoke.yml for D-164), so outside Merge gate.
reject_fixed "windows-latest"

# D-145 (9) (rewritten from requiring the Draft name guard): no Draft-specific aggregate name remains.
reject_fixed "Draft (no merge evidence)"
reject_fixed "ready_for_review"
reject_fixed "Hosted CI: skip"



# CI-TRIGGER-D1: dispatch still classifies everything (the name expression that also read event_name is gone, D-145 (9)).
require_fixed '"${{ github.event_name }}" == "workflow_dispatch"'
require_fixed "scripts/ci/classify-changes.sh --all"
require_fixed "scripts/ci/classify-changes.sh"
require_fixed '--base "${{ github.event.pull_request.base.sha }}"'
require_fixed '--head "${{ github.event.pull_request.head.sha }}"'
require_fixed "concurrency:"
require_fixed "cancel-in-progress: true"
require_fixed "if: always() && needs.changes.result == 'success'"
require_fixed "bash scripts/ci/check-required-jobs.sh"
require_fixed "cache: npm"
require_fixed "bash scripts/tests/codex-safe-wrappers.test.sh"
require_fixed "bash scripts/tests/claude-hooks.test.sh"

if grep -Fq 'Hosted CI: skip' "$PR_TEMPLATE"; then
    fail "PR template contains the opt-in skip token by default"
fi


cache_blocks="$(awk '
    /uses: actions\/cache@v5/ { in_cache=1; block=$0 ORS; next }
    in_cache && /^      - / { print block "---"; in_cache=0 }
    in_cache { block=block $0 ORS }
    END { if (in_cache) print block "---" }
' "$WORKFLOW")"

[[ -n "$cache_blocks" ]] || fail "Cargo cache blocks not found"
for path in '~/.cargo/registry/index/' '~/.cargo/registry/cache/' '~/.cargo/git/db/'; do
    grep -Fq "$path" <<< "$cache_blocks" || fail "cache missing $path"
done
if grep -Fq 'src-tauri/target/' <<< "$cache_blocks"; then
    fail "target remains in actions/cache"
fi
if grep -Fq '~/.cargo/bin/' <<< "$cache_blocks"; then
    fail "cargo bin remains in actions/cache"
fi

key_count="$(grep -cF 'key: ${{ runner.os }}-cargo-${{ hashFiles(' "$WORKFLOW")"
[[ "$key_count" == "3" ]] || fail "expected one shared key expression in each Rust cache job"

echo "PASS: ci-workflow"
