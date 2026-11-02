#!/usr/bin/env bash
# SPEC-MERGE-EVIDENCE / MG-D4: one inventory for local and hosted workflow gates.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
while IFS= read -r shell_file; do
    bash -n "$shell_file"
done < <(rg --files --hidden scripts .codex/bin -g '*.sh' | sort)
ruby -e "require 'yaml'; ARGV.each { |path| YAML.parse_file(path) }" .github/workflows/ci.yml .github/workflows/npm-security-monitor.yml
bash scripts/tests/classify-changes.test.sh

bash scripts/tests/check-command-drift.test.sh

bash scripts/tests/pre-push.test.sh

bash scripts/tests/local-ci.test.sh

bash scripts/tests/codex-safe-wrappers.test.sh

bash scripts/tests/claude-hooks.test.sh

bash scripts/tests/public-sanitization.test.sh

bash scripts/tests/doc-consistency-plan-packet.test.sh

bash scripts/tests/workflow-git-checks.test.sh

bash scripts/tests/plan-approve.test.sh

bash scripts/tests/reading-order-drift.test.sh

bash scripts/tests/ci-workflow.test.sh

bash scripts/tests/merge-gate.test.sh
python3 scripts/tests/pr-gate.test.py
