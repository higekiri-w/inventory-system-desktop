#!/usr/bin/env bash
# scripts/plan-approve.sh — D-145 (2): write the Plan Commit (or append a Gated Amendment) from the SHA the
# plan reviewer reported, as one packet-only commit. Nothing is pushed.
#
#   bash scripts/plan-approve.sh <packet> <REVIEWED_HEAD>          # Plan Commit: pending -> SHA
#   bash scripts/plan-approve.sh <packet> --amend <REVIEWED_HEAD>  # Amendments: append SHA
#
# Refuses, writing nothing, unless: SHA is a 40-char lowercase hex ancestor of HEAD; (a) the lane commits after
# SHA (first parent, no merges) touch only the packet and every merge between is a sync of main (#217's
# sync-only condition); (b) the packet at SHA and at HEAD differ only in its Plan Commit / Amendments lines;
# (c) the packet has no staged or unstaged edits.
set -euo pipefail

usage() { echo "usage: $0 <packet> [--amend] <40-hex SHA>" >&2; exit 2; }
[[ $# -eq 2 || $# -eq 3 ]] || usage
packet="$1"; shift
amend=0
if [[ "$1" == "--amend" ]]; then amend=1; shift; fi
[[ $# -eq 1 ]] || usage
reviewed="$1"

cd "$(git rev-parse --show-toplevel)"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PYTHONDONTWRITEBYTECODE=1 python3 - "$script_dir/pr-gate.py" "$packet" "$reviewed" "$amend" <<'PY'
import importlib.util, re, subprocess, sys
helper, packet, reviewed, amend = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4] == '1'
spec = importlib.util.spec_from_file_location('pr_gate', helper)
g = importlib.util.module_from_spec(spec); spec.loader.exec_module(g)

def stop(message):
    print(f'plan-approve: {message}. Nothing written', file=sys.stderr)
    sys.exit(1)

def git(*args):
    return subprocess.run(['git', *args], capture_output=True, text=True, check=False)

def record_lines(text):
    """The packet without the approval record lines of its Workflow State."""
    out, active = [], False
    for line in text.splitlines(keepends=True):
        if line.startswith('## '):
            active = line.strip() == '## Workflow State'
        if active and re.match(r'- (Plan Commit|Amendments):', line):
            continue
        out.append(line)
    return ''.join(out)

if not g.SHA.fullmatch(reviewed):
    stop(f'{reviewed!r} is not a 40-char lowercase hex SHA')
if git('cat-file', '-e', f'{reviewed}^{{commit}}').returncode or git('merge-base', '--is-ancestor', reviewed, 'HEAD').returncode:
    stop(f'{reviewed} is not a commit in this branch (not an ancestor of HEAD)')
if not g.PACKET.fullmatch(packet):
    stop(f'{packet} is not a dated packet under docs/plans/')
# (c) the approval commit must carry the reviewed packet only, never uncommitted Scope edits.
if git('diff', '--quiet', '--', packet).returncode or git('diff', '--cached', '--quiet', '--', packet).returncode:
    stop(f'{packet} has staged or unstaged edits; commit or drop them first')
head_text = git('show', f'HEAD:{packet}')
if head_text.returncode:
    stop(f'{packet} is missing at HEAD')
head_text = head_text.stdout
try:
    fields = g.workflow_fields(head_text)
except g.GateError as error:
    stop(str(error))
plan, amendments = fields.get('Plan Commit'), fields.get('Amendments')
if not amend:
    if plan != 'pending':
        stop(f'Plan Commit is {plan!r}, not pending (a Gated Amendment uses --amend)')
    key, value = 'Plan Commit', reviewed
else:
    if not plan or not g.SHA.fullmatch(plan):
        stop('--amend needs an approved Plan Commit')
    if git('merge-base', '--is-ancestor', plan, reviewed).returncode:
        stop(f'{reviewed} is not a descendant of Plan Commit {plan}')
    listed = [] if amendments in (None, 'none') else re.split(r'[,\s]+', amendments.strip())
    if reviewed in listed:
        stop(f'{reviewed} is already in Amendments')
    key, value = 'Amendments', ', '.join(listed + [reviewed])
reviewed_text = git('show', f'{reviewed}:{packet}')
if reviewed_text.returncode:
    stop(f'{packet} is missing at {reviewed}')
reviewed_text = reviewed_text.stdout
# (a) only the packet may change in lane commits after the reviewed SHA; merges must be syncs of main.
lane = git('log', '--first-parent', '--no-merges', '--no-renames', '--name-only', '--format=', f'{reviewed}..HEAD')
paths = {p for p in lane.stdout.splitlines() if p}
if lane.returncode or paths - {packet}:
    stop(f'lane commits after {reviewed} change {", ".join(sorted(paths - {packet})) or "(git log failed)"}; '
         'the reviewer must read the current plan')
listed = git('rev-list', '--first-parent', '--merges', f'{reviewed}..HEAD')
if listed.returncode:
    stop(f'cannot list the merges after {reviewed} (git rev-list exit {listed.returncode}); run git fetch origin and retry')
merges = listed.stdout.split()
if merges:
    main = git('rev-parse', '--verify', '-q', 'origin/main^{commit}').stdout.strip()
    if not main:
        stop('origin/main is unavailable to check the sync merges (git fetch origin)')
    gate = object.__new__(g.Gate)
    for merge in merges:
        first, second = git('rev-list', '--parents', '-n', '1', merge).stdout.split()[1:3]
        try:
            gate.sync_merge(merge, first, second, main)
        except g.GateError as error:
            stop(f'merge {merge} is not a sync of main: {error}')
# (b) the reviewed packet text is the approved text, apart from the record lines.
if record_lines(reviewed_text) != record_lines(head_text):
    stop(f'{packet} changed after {reviewed} beyond its Plan Commit / Amendments lines')
out, active, done = [], False, False
for line in head_text.splitlines(keepends=True):
    if line.startswith('## '):
        active = line.strip() == '## Workflow State'
    if active and not done and line.startswith(f'- {key}:'):
        line, done = f'- {key}: {value}\n', True
    out.append(line)
if not done:
    stop(f'Workflow State has no "- {key}:" line')
with open(packet, 'w', encoding='utf-8') as handle:
    handle.write(''.join(out))
subject = (f'docs(plans): Plan Commit を {reviewed[:12]} に固定する' if not amend
           else f'docs(plans): Gated Amendment {reviewed[:12]} を記録する')
commit = git('commit', '-q', '-m', subject, '--', packet)
if commit.returncode:
    git('checkout', '--', packet)
    stop(f'git commit failed: {commit.stderr.strip()}')
print(f'plan-approve: {key} = {value} ({git("rev-parse", "--short", "HEAD").stdout.strip()}); push when ready')
PY
