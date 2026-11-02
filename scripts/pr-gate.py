#!/usr/bin/env python3
"""SPEC-MERGE-EVIDENCE / MG-D5..D8, MG-D13. Owner-operated PR evidence; gh + stdlib only."""
import argparse
import base64
import copy
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
from urllib.parse import quote

REPO = 'kosei-w90607/inventory-system-desktop'
OWNER = REPO.split('/')[0]
MARKER = '<!-- inventory-workflow-v1 -->'
POLICY = '.github/merge-gate-ruleset.json'
SHA = re.compile(r'[0-9a-f]{40}\Z')
OUTCOMES = {'pending', 'pass', 'fail', 'not-required'}
# D-107 (3): blockers name the next review to record, not only the stale record.
FRESH_BROAD = ('broad Plan contract changed; fresh broad required: record --review-stage broad at the current head'
               ' (a closure cannot carry this broad)')
BROAD_REQUIRED = ('broad required: record --review-stage broad at the current head until it has Final Review Minimum audits'
                  ' (no usable broad on the server; a closure cannot replace it)')
# MG-D13: the only paths a sync may regenerate. They are exactly the paths the hosted drift checks
# re-derive from source (scripts/ci/classify-changes.sh `generated` case, .github/workflows/ci.yml
# `rust_drift` job). Kept as a constant, never parsed from the classifier; src/routeTree.gen.ts is untracked.
GENERATED = frozenset({'src/lib/bindings.ts', 'docs/function-design/90-traceability.md'})
# MG-D13 / D-144: tail appends of parallel lanes may conflict here; only the union result is accepted.
UNION = 'docs/decision-log.md'
CODE_FLAGS = ('rust', 'frontend', 'env', 'unknown')
# D-145 (4): old packets (with a Phase line) keep their written minimum and gates; new packets derive them.
OLD_FIELDS = {'Risk', 'Plan Commit', 'Amendments', 'Final Review Minimum', 'Human Gate'}
NEW_FIELDS = {'Risk', 'Plan Commit', 'Amendments'}
PACKET = re.compile(r'docs/plans/\d{4}-\d\d-\d\d-[^/]*\.md')
APPROVE = 'scripts/plan-approve.sh'
# D-145 (8): the no-packet route (R0..R2) never takes migrations or dependencies.
MIGRATION = re.compile(r'src-tauri/src/db/(migration|migration_tx|schema_v[^/]*)\.rs')
DEPENDENCIES = frozenset({'package.json', 'package-lock.json', '.npmrc'})
BODY_RISK = re.compile(r'Risk: (R[0-2])（目視: (要|不要)）')
REPORT_HEAD = re.compile(r'REVIEWED_HEAD=([0-9a-f]{40})')
# D-145 (6): a lane commit may keep L3 and review only when the classifier sees docs and nothing else.
NON_DOCS = ('rust', 'rust_drift', 'frontend', 'env', 'generated', 'traceability', 'workflow', 'unknown')


class GateError(Exception):
    def __init__(self, message, code=1):
        super().__init__(message)
        self.code = code


def require(condition, message, code=1):
    if not condition:
        raise GateError(message, code)


def command(argv, *, data=None, binary=False, status=False):
    # status=True: exit 0/1 is a verdict (merge-tree conflict, is-ancestor) returned with stdout; others fail.
    try:
        result = subprocess.run(argv, input=data, capture_output=True, text=not binary, check=False)
    except OSError:
        raise GateError(f'command unavailable: {argv[0]}', 2) from None
    require(result.returncode in ((0, 1) if status else (0,)), f'command failed: {argv[0]} (exit {result.returncode})', 2)
    return (result.returncode, result.stdout) if status else result.stdout


def git(*args, binary=False, status=False):
    return command(['git', *args], binary=binary, status=status)


def changed(left, right):
    # MG-D13: renames count on both sides, so a rename into a generated path is a deletion elsewhere.
    return {path for path in git('diff', '--name-only', '--no-renames', '-z', left, right, '--').split('\0') if path}


def classify(classifier, paths):
    output = command(['bash', '-c', classifier, 'classify-changes', '--files-from-stdin'], data='\n'.join(paths)+'\n')
    flags = dict(line.split('=', 1) for line in output.splitlines())
    expected = set('rust rust_drift frontend docs env generated traceability workflow unknown'.split())
    require(set(flags) == expected and all(v in ('true','false') for v in flags.values()), 'invalid main classifier', 2)
    return flags


def unique_object(pairs):
    result = dict(pairs)
    require(len(result) == len(pairs), 'duplicate JSON fields', 2)
    return result


def decode_json(text):
    return json.loads(text, object_pairs_hook=unique_object)


def api(path, *, method='GET', payload=None, pages=False):
    argv = ['gh', 'api', '--hostname', 'github.com', '-X', method,
            '-H', 'Accept: application/vnd.github+json', path]
    if pages:
        argv += ['--paginate', '--slurp']
    if payload is not None:
        argv += ['--input', '-']
    try:
        return decode_json(command(argv, data=json.dumps(payload) if payload is not None else None))
    except (ValueError, TypeError):
        raise GateError('invalid GitHub JSON', 2) from None


def sha(value):
    require(isinstance(value, str) and SHA.fullmatch(value), 'invalid full SHA', 2)
    return value


def pointer(value):
    require(isinstance(value, str) and value.strip() and '\n' not in value and '\r' not in value,
            'evidence/run/model must be a nonempty public single-line pointer', 2)
    return value


def keys(value, required, optional=()):
    require(isinstance(value, dict) and set(required) <= value.keys() <= set(required) | set(optional),
            'invalid record fields', 2)


def evidence(values):
    require(isinstance(values, list), 'evidence must be an array', 2)
    for value in values:
        pointer(value)


def audit(value):
    keys(value, {'model', 'run_ref', 'evidence'})
    pointer(value['model']); pointer(value['run_ref']); evidence(value['evidence'])
    require(value['evidence'], 'audit evidence missing')


def validate_record(value, pr):
    keys(value, {'version', 'repo', 'pr', 'head', 'base', 'review', 'manual', 'r4'})
    require(type(value['version']) is int and value['version'] == 1 and value['repo'] == REPO
            and type(value['pr']) is int and value['pr'] == pr, 'record belongs to another repo/PR', 2)
    sha(value['head']); sha(value['base'])
    review = value['review']
    keys(review, {'outcome', 'broad', 'closure'})
    require(review['outcome'] in OUTCOMES, 'unknown review outcome', 2)
    broad, closure = review['broad'], review['closure']
    if broad is not None:
        keys(broad, {'head', 'base', 'plan_commit', 'amendments', 'audits'})
        sha(broad['head']); sha(broad['base'])
        if broad['plan_commit'] is not None:
            sha(broad['plan_commit'])
        require(isinstance(broad['amendments'], list), 'invalid amendments', 2)
        for item in broad['amendments']:
            sha(item)
        require(isinstance(broad['audits'], list), 'invalid audits', 2)
        for item in broad['audits']:
            audit(item)
        refs = [item['run_ref'] for item in broad['audits']]
        require(len(refs) == len(set(refs)), 'duplicate audit run_ref')
    if closure is not None:
        # MG-D13: an audited closure {head, base, audit} or a helper sync closure {head, base, sync_from}, never both.
        form = 'sync_from' if isinstance(closure, dict) and 'sync_from' in closure else 'audit'
        keys(closure, {'head', 'base', form})
        sha(closure['head']); sha(closure['base'])
        require(broad is not None, 'closure needs the broad it continues')
        if form == 'audit':
            audit(closure['audit'])
            require(closure['audit']['run_ref'] not in [a['run_ref'] for a in broad['audits']], 'closure must be an independent run')
        else:
            sha(closure['sync_from'])
    if review['outcome'] == 'not-required':
        require(broad is None and closure is None, 'unneeded review must have null audits')
    for kind in ('manual', 'r4'):
        result = value[kind]
        keys(result, {'outcome', 'evidence'}, {'source_head', 'reuse_approval'} if kind == 'manual' else ())
        require(result['outcome'] in OUTCOMES, 'unknown outcome', 2)
        evidence(result['evidence'])
        require(result['outcome'] != 'pass' or result['evidence'], f'{kind} pass needs evidence')
        # MG-D13: source_head alone is A (code-free sync); with reuse_approval it is B. Approval alone is incomplete.
        require('reuse_approval' not in result or 'source_head' in result, 'incomplete manual reuse')
        if 'source_head' in result:
            sha(result['source_head'])
            require(result['outcome'] == 'pass', 'only passed manual evidence may be reused')
        if 'reuse_approval' in result:
            pointer(result['reuse_approval'])
    return value


def markdown(text):
    text = re.sub(r'<!--.*?-->', '', text, flags=re.S)
    return re.sub(r'(?m)^(```|~~~).*?^\1[^\n]*$', '', text, flags=re.S)


def workflow_fields(text):
    text = markdown(text)
    sections = re.findall(r'^## Workflow State\s*\n(.*?)(?=^## |\Z)', text, flags=re.M | re.S)
    require(len(sections) == 1, 'packet Workflow State missing/ambiguous')
    # Spaces stay on the key's line; an empty value keeps the key so presence checks see it.
    pairs = re.findall(r'^- ([^:\n]+):[ \t]*([^\n]*)', sections[0], flags=re.M)
    fields = dict(pairs)
    require(len(pairs) == len(fields), 'duplicate packet fields')
    return fields


def sections(text):
    """h2 sections outside code fences as {title: [raw body, ...]}; a duplicated title keeps every body.
    Trailing blank lines are the separator before the next heading, not the section's text."""
    found, title, body, fence = {}, None, [], None
    for line in text.splitlines(keepends=True):
        mark = re.match(r'(```|~~~)', line)
        if fence is None and mark:
            fence = mark[1]
        elif fence and line.startswith(fence):
            fence = None
        elif fence is None and line.startswith('## '):
            if title is not None:
                found.setdefault(title, []).append(''.join(body).rstrip('\n'))
            title, body = line[3:].strip(), []
            continue
        if title is not None:
            body.append(line)
    if title is not None:
        found.setdefault(title, []).append(''.join(body).rstrip('\n'))
    return found


def packet_format(text):
    """D-145 (4): an old packet carries a Phase line; the git history decides which form it must be (packet_form)."""
    return 'old' if 'Phase' in workflow_fields(text) else 'new'


def parse_packet(text, form=None):
    """form: the form packet_form read from git history; None trusts the Phase line (merged or archived packets)."""
    fields = workflow_fields(text)
    written = packet_format(text)
    if form is not None:
        require(form == written, 'new-form packet (added after scripts/plan-approve.sh) must not have a Phase line;'
                ' an old packet is never renamed or re-cut onto a new main' if form == 'new'
                else 'old-form packet (added before scripts/plan-approve.sh) needs its Phase line')
    required = OLD_FIELDS if written == 'old' else NEW_FIELDS
    require(required <= fields.keys() and all(fields[key] for key in required), 'packet fields missing/empty/duplicate')
    text = markdown(text)
    # Retired marker lines are optional; other extra lines (roles, Branch, the old Phase value) are not evaluated.
    require(fields.get('Evidence Mode', 'github') == 'github', 'Evidence Mode is retired; write github or omit it')
    require(not {'Reviewed Content HEAD', 'Final Exact-HEAD Evidence', 'Hosted CI Requirement'} & fields.keys(),
            'legacy fields in packet')
    require(fields['Risk'] in (('R2', 'R3', 'R4') if written == 'old' else ('R3', 'R4')),
            'invalid packet Risk' if written == 'old' else 'new-form packets are R3 or R4 (R2 takes the no-packet route)')
    risks = re.findall(r'^Risk: (R[0-4])\s*$', text, flags=re.M)
    require(risks == [fields['Risk']], 'packet Risk section mismatch')
    plan = fields['Plan Commit']
    if plan != 'pending':
        sha(plan)
    amendments = [] if fields['Amendments'] == 'none' else re.split(r'[,\s]+', fields['Amendments'])
    for item in amendments:
        sha(item)
    require(len(amendments) == len(set(amendments)), 'duplicate amendments')
    minimum = fields.get('Final Review Minimum')
    require(minimum is None or minimum in ('1', '2'), 'invalid review minimum')
    gates = fields['Human Gate'].split(',') if 'Human Gate' in fields else ['ready', 'merge']
    require(len(gates) == len(set(gates)) and {'ready', 'merge'} <= set(gates) <= {'ready', 'merge', 'manual', 'r4'},
            'Human Gate must explicitly include ready,merge')
    r4, manual = 'r4' in gates, 'manual' in gates
    if written == 'old':
        require(fields['Risk'] != 'R4' or (r4 and minimum == '2'), 'R4 gates missing')
        minimum = int(minimum)
        if fields['Phase'] in PRE_GATE:
            plan = 'pending'  # Final review C1: a pre-gate old packet is not approved, whatever its Plan Commit says
    else:
        # TD-484: R3 and R4 take 2; a written minimum only raises; written r4/manual are never dropped.
        minimum = max(2, int(minimum or 0))
        r4 = r4 or fields['Risk'] == 'R4'
        manual = manual or 'L3' in sections(text)
    return dict(risk=fields['Risk'], plan_commit=plan, amendments=amendments, minimum=minimum, manual=manual, r4=r4)


# An old packet still at one of these Phases is not approved even when it carries a SHA (the old helper's rule).
PRE_GATE = ('kickoff', 'spec-check', 'design', 'plan-draft', 'plan-gate')


def plan_texts(text, risk):
    """Every plan section present (the plan_sections names, both Ledger forms, R4 recovery headings)."""
    found = sections(text)
    names = ['Scope', 'Acceptance Criteria', 'Contract Ledger', 'Spec Contract', 'Trace Matrix']
    if risk == 'R4':
        names += [n for n in found if n == 'Data Safety' or 'Recovery' in n or 'Rollback' in n]
    return {name: found[name] for name in names if name in found}


def plan_drift(path, approved_text, current_text, risk):
    """Final review C1 (D-145 (2)(7)): the plan sections must equal the latest approved snapshot; returns the refusal."""
    old, new = plan_texts(approved_text, risk), plan_texts(current_text, risk)
    names = sorted(name for name in set(old) | set(new) if old.get(name) != new.get(name))
    if not names:
        return None
    return (f'approved plan changed: 計画の節が承認の後に変わった（{", ".join(names)}）。計画の review を受け、'
            f'scripts/plan-approve.sh {path} --amend <REVIEWED_HEAD> で登録する')


def approval_state(path, text):
    """For the git check: ('pending', None) or ('approved', refusal or None) against the latest approved snapshot."""
    fields = workflow_fields(text)
    plan, amendments = fields.get('Plan Commit', 'pending'), fields.get('Amendments', 'none').strip()
    if plan == 'pending' or (packet_format(text) == 'old' and fields.get('Phase') in PRE_GATE):
        return 'pending', None
    ref = re.split(r'[,\s]+', amendments)[-1] if amendments not in ('', 'none') else plan
    approved = try_git('show', f'{ref}:{path}')
    if approved is None:
        return 'approved', f'{path}: 承認の版 {ref} の packet を読めない（git fetch origin）'
    return 'approved', plan_drift(path, approved, text, fields.get('Risk'))


def gate_conditions(text):
    """MN4: what an approval fixes. Old packets: the three written fields; new: Risk, the L3 section and written lines."""
    fields = workflow_fields(text)
    keys = ('Risk', 'Final Review Minimum', 'Human Gate')
    result = {key: fields.get(key) for key in keys}
    if packet_format(text) == 'new':
        result['L3 section'] = 'L3' in sections(markdown(text))
    return result


def plan_sections(text, risk):
    """D-145 (7): the plan sections H6/H7 compare byte for byte; None when one is missing or duplicated."""
    found = sections(text)
    names = ['Scope', 'Acceptance Criteria']
    names += ['Contract Ledger'] if 'Contract Ledger' in found or packet_format(text) == 'new' else ['Spec Contract', 'Trace Matrix']
    if risk == 'R4':
        if packet_format(text) == 'new':
            names += ['Recovery', 'Data Safety']
        else:
            names += [n for n in found if n == 'Data Safety' or 'Recovery' in n or 'Rollback' in n]
    if any(len(found.get(name, [])) != 1 for name in names):
        return None
    return {name: found[name][0] for name in names}


def l3_basis(text, read):
    """D-145 (6): the L3 criteria a manual pass answered; None when they cannot be compared.
    read(path) returns a repo file at the same version, or None when absent."""
    found = sections(text)
    if 'L3' in found:
        return ('L3', found['L3'])
    if packet_format(text) == 'new':
        return ('no manual',)
    plan = found.get('Test Plan', [])
    if len(plan) != 1:
        return None
    links = set(re.findall(r'\]\(((?:[^)]*/)?test-matrices/[^)]*\.md)\)|`([^`]*test-matrices/[^`]*\.md)`', plan[0]))
    paths = {a or b for a, b in links}
    if len(paths) != 1:
        return None
    path = paths.pop()
    path = os.path.normpath(path if path.startswith('docs/') else os.path.join('docs/plans', path))
    matrix = read(path)
    return None if matrix is None else ('Test Plan', plan[0], path, matrix)


def try_git(*args):
    """stdout of a git query, or None when git answers no (exit 1) or fails."""
    try:
        code, output = git(*args, status=True)
    except GateError:
        return None
    return output if code == 0 else None


def plan_origin(env_base=None):
    """D-145 (5): the start active packets are measured from; None when the branch has no lane commit."""
    head = (try_git('rev-parse', '--verify', '-q', 'HEAD^{commit}') or '').strip()
    require(head, '起点を解決できない: HEAD が無い（commit してから検査する）')
    start = None
    if env_base:
        start = (try_git('rev-parse', '--verify', '-q', f'{env_base}^{{commit}}') or '').strip()
        require(start, f'起点を解決できない: WORKFLOW_BASE_SHA {env_base} が無い（git fetch origin）')
    if not start or start == head:
        start = (try_git('rev-parse', '--verify', '-q', 'origin/main^{commit}') or '').strip()
        require(start, '起点を解決できない: WORKFLOW_BASE_SHA も origin/main も無い（git fetch origin か WORKFLOW_BASE_SHA を渡す。'
                'review の copy は git archive でなく git worktree add で作る）')
    bases = (try_git('merge-base', '--all', start, head) or '').split()
    require(len(bases) == 1, f'起点を決められない: {start} と HEAD の merge-base が一意でない（{len(bases)} 個）')
    return None if bases[0] == head else bases[0]


def active_packets(origin, names):
    """D-145 (5): dated packets in docs/plans that the origin tree does not have (packets this branch added)."""
    if origin is None:
        return []
    try:
        base = set(git('ls-tree', '-z', '--name-only', origin, 'docs/plans/').split('\0'))
    except GateError as error:
        raise GateError(f'起点 {origin} の tree を読めない（{error}）', 1) from None
    return sorted(name for name in names if PACKET.fullmatch(name) and name not in base)


def packet_form(path, origin, head='HEAD'):
    """D-145 (4): 'old' when the commit that first added path after origin has no scripts/plan-approve.sh in its parent."""
    adds = (try_git('log', '--no-renames', '--diff-filter=A', '--format=%H', f'{origin}..{head}', '--', path) or '').split()
    require(adds, f'{path}: 起点 {origin} の後で packet を足した commit が見つからない（commit してから検査する）')
    parents = (try_git('rev-list', '--parents', '-n', '1', adds[-1]) or '').split()[1:]
    require(parents, f'{path}: packet を足した commit {adds[-1]} に親が無く、形式を判定できない')
    # A shallow boundary looks like an add of every file with no parent in the graph, while the raw commit still
    # names its parents. Only this commit's ancestry matters (MG-D4: an unrelated shallow ref is not evidence).
    raw = re.findall(r'^parent ([0-9a-f]{40})$', try_git('cat-file', '-p', adds[-1]) or '', flags=re.M)
    require(raw == parents and try_git('cat-file', '-e', f'{parents[0]}^{{commit}}') is not None,
            f'{path}: packet を足した commit {adds[-1]} が shallow の境にあり、形式を判定できない（full history で取り直す）')
    listed = try_git('ls-tree', '--name-only', parents[0], '--', APPROVE)
    require(listed is not None, f'{path}: 親 {parents[0]} の tree を読めず、形式を判定できない（git fetch origin）')
    return 'new' if listed.strip() else 'old'


def workflow_scope(env_base, names):
    """For the bash checks: (origin or None, [(path, form)]) under the single active definition."""
    origin = plan_origin(env_base)
    return origin, [(path, packet_form(path, origin)) for path in active_packets(origin, names)]


class Gate:
    docs_commits = ()  # lane docs commits the last sync_merges(docs=True) walked over (D-145 (6) C)

    def __init__(self, args):
        self.args = args
        require(args.repo == REPO, 'unsupported repository', 2)
        remote = git('remote', 'get-url', 'origin').strip()
        require(remote in (f'git@github.com:{REPO}.git', f'git@github.com:{REPO}',
                           f'https://github.com/{REPO}.git', f'https://github.com/{REPO}'), 'origin repo mismatch', 2)
        self.root = Path(git('rev-parse', '--show-toplevel').strip()).resolve()
        self.endpoint = f'repos/{REPO}'

    def pr(self):
        pr = api(f'{self.endpoint}/pulls/{self.args.pr}')
        require(pr['number'] == self.args.pr and pr['base']['repo']['full_name'] == REPO
                and pr['head']['repo']['full_name'] == REPO and pr['base']['ref'] == 'main', 'PR repo/base mismatch')
        sha(pr['head']['sha']); sha(pr['base']['sha'])
        require(pr['state'] == 'open' or pr.get('merged'), 'PR is closed')
        return pr

    def contents(self, path, ref):
        value = api(f'{self.endpoint}/contents/{quote(path, safe="/")}?ref={sha(ref)}')
        require(value.get('encoding') == 'base64', 'GitHub content unavailable', 2)
        return base64.b64decode(value['content'], validate=False).decode()

    def requirements(self, pr):
        head, base = pr['head']['sha'], pr['base']['sha']
        # SPEC-WF-HARNESS5-D2: only the PR base's helper may judge the PR; a changed helper must not judge itself.
        require(self.contents('scripts/pr-gate.py', base).encode() == Path(__file__).read_bytes(),
                f'helper differs from base {base}; run: git fetch origin && git show {base}:scripts/pr-gate.py'
                f' > "${{TMPDIR:-/tmp}}/pr-gate-base.py" && python3 "${{TMPDIR:-/tmp}}/pr-gate-base.py" {shlex.join(sys.argv[1:])}')
        pages = api(f'{self.endpoint}/pulls/{self.args.pr}/files?per_page=100', pages=True)
        items = [item for page in pages for item in page]
        # GitHub lists at most 3000 files per PR; at the cap the diff may be truncated.
        require(0 < len(items) < 3000, 'PR diff unavailable or truncated')
        paths = [path for item in items for path in (item['filename'], item.get('previous_filename', item['filename']))]
        # Use current main, not the PR's classifier, to determine review requirements.
        flags = classify(self.contents('scripts/ci/classify-changes.sh', base), paths)
        # D-145 (5): the PR binds to the active packets it adds (absent from the base tree), never to merged
        # packets it edits; a packet leaves docs/plans only by moving to the same-named archive.
        active, leaving, moves, removed, written = set(), set(), set(), set(), set()
        for item in items:
            status, name = item['status'], item['filename']
            require(status in ('added','removed','modified','renamed','copied','changed','unchanged'), f'unknown PR file status: {status}', 2)
            if status == 'removed':
                removed.add(name)
            elif status in ('added', 'renamed', 'copied') and PACKET.fullmatch(name):
                active.add(name)
            if status in ('added', 'modified'):
                written.add(name)
            if status == 'renamed':
                moves.add((item['previous_filename'], name))
                if PACKET.fullmatch(item['previous_filename']):
                    leaving.add(item['previous_filename'])
        leaving |= {name for name in removed if PACKET.fullmatch(name)}
        if self.args.packet:
            require(re.fullmatch(r'docs/plans/\d{4}-\d\d-\d\d-[A-Za-z0-9_-]+\.md', self.args.packet), 'invalid packet path', 2)
            require(active | leaving == {self.args.packet} and self.args.packet in active,
                    'PR diff must touch exactly the --packet active packet')
            packet_text = self.contents(self.args.packet, head)
            form = self.packet_form(self.args.packet, base, head)
            result = parse_packet(packet_text, form)
            if result['plan_commit'] != 'pending':
                approved_ref = result['amendments'][-1] if result['amendments'] else result['plan_commit']
                approved_text = self.contents(self.args.packet, approved_ref)
                approved = gate_conditions(approved_text)
                for field, value in gate_conditions(packet_text).items():
                    require(approved.get(field) == value, f'approved packet condition changed: {field}')
                drift = plan_drift(self.args.packet, approved_text, packet_text, result['risk'])
                require(not drift, drift)
        else:
            require(not active, 'R2+ active packet requires --packet')
            require(self.args.risk in ('R0','R1','R2') and self.args.manual in ('required','not-required'),
                    'no-packet route needs --risk R0|R1|R2 --manual required|not-required', 2)
            self.body_risk(pr)
            route = f'packet なしの道（{self.args.risk}）では通れない'
            stopped = sorted(p for p in paths if MIGRATION.fullmatch(p) or p in DEPENDENCIES)
            require(not stopped, f'{route}: migration・依存の変更 {", ".join(stopped)}。R3 の packet で起票する')
            require(flags['unknown'] == 'false', f'{route}: classifier が未知の path を出した（unknown=true）。R3 の packet で起票する')
            require(not (flags['workflow'] == 'true' and flags['rust'] == 'true'),
                    f'{route}: CI execution change（workflow かつ rust）。R3 の packet で起票する')
            require(self.args.risk != 'R2' or flags['workflow'] == 'false',
                    f'{route}: gate の文書・道具の変更（workflow=true）。R3 の packet で起票する')
            # AR1: a merged packet may move to the same-named archive (the old closeout shape); a bare deletion stops.
            for packet in sorted(leaving):
                archive = 'docs/archive/plans/' + Path(packet).name
                require((packet, archive) in moves or (packet in removed and archive in written),
                        'packet leaving docs/plans must move to docs/archive/plans in this PR')
            result = dict(risk=self.args.risk, plan_commit=None, amendments=[], minimum=1 if self.args.risk == 'R2' else 0,
                          manual=self.args.manual == 'required', r4=False)
        if result['minimum']:
            double = result['risk'] == 'R4' or flags['workflow'] == 'true'
            require(not double or result['minimum'] == 2, 'required Double Audit minimum is 2')
        return result

    def body_risk(self, pr):
        """R2N5: the PR body's single Risk line must say what --risk and --manual say."""
        lines = [line.strip() for line in markdown(pr.get('body') or '').replace('\r', '').splitlines()
                 if line.strip().startswith('Risk:')]
        require(len(lines) == 1 and BODY_RISK.fullmatch(lines[0]),
                f'PR body needs exactly one line `Risk: R<0|1|2>（目視: 要|不要）` (found {len(lines)})')
        risk, seen = BODY_RISK.fullmatch(lines[0]).groups()
        manual = 'required' if seen == '要' else 'not-required'
        require((risk, manual) == (self.args.risk, self.args.manual),
                f'PR body says {lines[0]} but the helper was given --risk {self.args.risk} --manual {self.args.manual}')

    def packet_form(self, path, base, head):
        """D-145 (4) on the PR's own history: the form of the packet's first add after the base."""
        origin = (try_git('merge-base', base, head) or '').strip()
        require(origin, f'{path}: PR の base {base} と head {head} が手元に無い。run git fetch origin', 2)
        return packet_form(path, origin, head)

    def server_record(self):
        pages = api(f'{self.endpoint}/issues/{self.args.pr}/comments?per_page=100', pages=True)
        records = [item for page in pages for item in page if MARKER in item.get('body', '')]
        require(len(records) <= 1, 'multiple workflow records')
        if not records:
            return None
        item = records[0]
        require(item['user']['login'] == OWNER, 'invalid workflow record author')
        match = re.fullmatch(re.escape(MARKER) + r'\n```json\n(.*?)\n```\n?', item['body'], flags=re.S)
        require(match is not None, 'malformed workflow record', 2)
        value = validate_record(decode_json(match[1]), self.args.pr)
        return dict(id=item['id'], body=item['body'], updated_at=item['updated_at'], record=value)

    def snapshot(self, *, clean=False):
        pr = self.pr()
        requirements = self.requirements(pr)
        server = self.server_record()
        if clean:
            require(requirements['plan_commit'] != 'pending', 'Plan Gate incomplete: Plan Commit is pending (scripts/plan-approve.sh)')
            require(not git('status', '--porcelain', '--untracked-files=normal').strip(), 'capture requires clean checkout')
            require(git('rev-parse', 'HEAD').strip() == pr['head']['sha'], 'local HEAD differs from PR')
            for ref in (pr['base']['sha'], requirements['plan_commit'], *requirements['amendments']):
                if ref:
                    git('cat-file', '-e', f'{sha(ref)}^{{commit}}')
        require(self.same_pr(pr, self.pr()), 'head/base changed during observation')
        return dict(version=1, repo=REPO, pr=self.args.pr, head=pr['head']['sha'], base=pr['base']['sha'],
                    packet=self.args.packet, requirements=requirements, server=server), pr

    @staticmethod
    def same_pr(left, right):
        return all(left[k]['sha'] == right[k]['sha'] for k in ('head','base'))

    def validate_review(self, record, req, head, base):
        review = record['review']
        if not req['minimum']:
            require(review['outcome'] == 'not-required', 'R0/R1 review must be explicitly not-required')
            return
        require(review['outcome'] == 'pass', 'review not passed')
        broad = review['broad']
        require(broad is not None and len(broad['audits']) >= req['minimum'], 'broad audits below minimum')
        require(self.plan_same(broad, req, head), FRESH_BROAD)
        closure = review['closure']
        if (broad['head'], broad['base']) == (head, base):
            require(closure is None, 'current broad must have null closure')
        else:
            require(closure is not None and (closure['head'], closure['base']) == (head, base), 'current closure missing')

    def sync_merges(self, origin, head, base, docs=False):
        """MG-D13 sync-only condition: head reaches origin by first parents through sync merges and
        generated-only regenerations. Returns the merges oldest first as (merge, first, second, merge_base).
        docs=True (D-145 (6) C) also passes 1-parent lane commits whose --no-renames paths the base classifier
        sees as docs only; then the head needs no merge when the source already holds the base."""
        merges, commit, self.docs_commits = [], head, []
        while commit != origin:
            parents = git('rev-list', '--parents', '-n', '1', commit).split()[1:]
            if len(parents) == 2:
                merges.append(self.sync_merge(commit, *parents, base))
            else:
                require(len(parents) == 1, f'commit {commit} is neither a sync merge nor a regeneration; source {origin} not reached')
                lane = sorted(changed(parents[0], commit) - GENERATED)
                if lane and docs:
                    flags = classify(self.contents('scripts/ci/classify-changes.sh', base), lane)
                    other = [flag for flag in NON_DOCS if flags[flag] == 'true']
                    require(not other, f'lane commit {commit} after source {origin} is not docs only ({", ".join(other)}):'
                            f' {", ".join(lane)}')
                    self.docs_commits.append(commit)
                    lane = []
                require(not lane, f'lane commit {commit} after source {origin} changes {", ".join(lane)}')
            commit = parents[0]
        if merges:
            require(merges[0][2] == base, f'last sync merge {merges[0][0]} takes {merges[0][2]}, not the current base {base}; sync again')
        else:
            require(docs, f'no sync merge between source {origin} and head {head}')
            code, _ = git('merge-base', '--is-ancestor', base, origin, status=True)
            require(code == 0, f'base が進んだ。先に同期する: source {origin} does not hold the current base {base}')
        return merges[::-1]

    def sync_merge(self, commit, first, second, base):
        code, _ = git('merge-base', '--is-ancestor', second, base, status=True)
        require(code == 0, f'merge {commit} takes {second}, which is not in main (not an ancestor of base {base})')
        bases = git('merge-base', '--all', first, second).split()
        require(len(bases) == 1, f'merge {commit} has an ambiguous merge-base ({len(bases)} bases)')
        _, output = git('-c', 'merge.directoryRenames=false', 'merge-tree', '--write-tree', '--name-only', '-z',
                        first, second, status=True)
        fields = [field for field in output.partition('\0\0')[0].split('\0') if field]
        require(fields and SHA.fullmatch(fields[0]), 'command failed: git merge-tree (no tree)', 2)
        paths = changed(fields[0], commit) | set(fields[1:])
        if UNION in paths:
            union = git('merge-file', '-p', '--union', '--object-id', f'{first}:{UNION}', f'{bases[0]}:{UNION}',
                        f'{second}:{UNION}', binary=True)
            require(union == git('cat-file', 'blob', f'{commit}:{UNION}', binary=True),
                    f'merge {commit} resolves {UNION} differently from the union of both sides')
        lane = sorted(paths - GENERATED - {UNION})
        require(not lane, f'merge {commit} differs from the automatic merge (conflict or edit) at {", ".join(lane)}')
        return commit, first, second, bases[0]

    def code_paths(self, merges, base):
        """MG-D13 code-free condition: main-side paths of every sync merge, classified by current main."""
        paths = sorted(set().union(*(changed(merge_base, second) for _, _, second, merge_base in merges)))
        if not paths:
            return []
        classifier = self.contents('scripts/ci/classify-changes.sh', base)
        touches = lambda names: any(classify(classifier, names)[flag] == 'true' for flag in CODE_FLAGS)
        if not touches(paths):
            return []
        return [path for path in paths if touches([path])] or paths

    def reusable(self, origin, head, base, code=True, docs=False):
        """Sync-only (and, if asked, code-free) check from origin; returns the code paths the sync brings."""
        try:
            merges = self.sync_merges(origin, head, base, docs)
            return self.code_paths(merges, base) if code else []
        except GateError as error:
            if error.code == 2:
                raise GateError(f'{error}; reuse undecidable: run git fetch origin and retry', 2) from None
            raise

    def docs_route(self, req):
        """D-145 (6): lane docs commits keep L3 and review only with a packet to compare, and never for R4."""
        return bool(self.args.packet) and req['risk'] != 'R4'

    def packet_at(self, ref, path=None):
        return try_git('show', f'{ref}:{path or self.args.packet}')

    def l3_change(self, source, head, skip_without_manual=False):
        """C1..C5 / RC1b: why the L3 basis differs between source and head, or None when it is the same."""
        old, new = self.packet_at(source), self.packet_at(head)
        if old is None or new is None:
            return f'packet {self.args.packet} unreadable at {source if old is None else head}; run git fetch origin'
        if skip_without_manual:
            try:
                needs = (parse_packet(old)['manual'], parse_packet(new)['manual'])
            except GateError as error:
                return f'packet unreadable ({error})'
            if needs == (False, False):
                return None
            if needs[0] != needs[1]:
                return 'manual の要否が版の間で変わった'
        bases = [l3_basis(text, lambda path, ref=ref: self.packet_at(ref, path)) for text, ref in ((old, source), (new, head))]
        if None in bases:
            return 'L3 の比較対象が取れない（`## L3` も `## Test Plan` も無い、Matrix の link が 1 本でない、または file が無い）'
        return None if bases[0] == bases[1] else 'L3 節が変わった（L3 の比較対象が byte で違う）'

    def plan_change(self, source, head, risk):
        """RC2 / GA1..GA4: None when the plan sections are byte-equal at source and head."""
        old, new = self.packet_at(source), self.packet_at(head)
        if old is None or new is None:
            return 'packet unreadable'
        try:
            left, right = plan_sections(old, risk), plan_sections(new, risk)
        except GateError as error:
            return f'packet unreadable ({error})'
        if left is None or right is None:
            return '計画の節が読めない（欠けているか重複している）'
        changed_names = [name for name in sorted(set(left) | set(right)) if left.get(name) != right.get(name)]
        return f'計画の節が変わった: {", ".join(changed_names)}' if changed_names else None

    def plan_same(self, broad, req, head):
        """The broad still covers the plan: same Plan contract, or (D-145 (7)) a later Gated Amendment that leaves
        the plan sections byte-equal, which asks for a closure instead of a fresh broad."""
        if (broad['plan_commit'], broad['amendments']) == (req['plan_commit'], req['amendments']):
            return True
        if not self.args.packet or broad['plan_commit'] != req['plan_commit'] \
                or req['amendments'][:len(broad['amendments'])] != broad['amendments']:
            return False
        return self.plan_change(broad['head'], head, req['risk']) is None

    def cli(self, *extra):
        a = self.args
        scope = ['--packet', a.packet] if a.packet else ['--risk', a.risk or 'R0', '--manual', a.manual or 'required']
        return shlex.join(['python3', 'scripts/pr-gate.py', 'record', '--pr', str(a.pr), *scope, '--capture', a.capture, *extra])

    def carry_manual(self, old, snap):
        """MG-D13 A: carry the server manual pass to a new head, or stop before any write with the next step."""
        manual, head, base = old['manual'], snap['head'], snap['base']
        source = manual.get('source_head', old['head'])
        pending = self.cli('--kind', 'manual', '--outcome', 'pending', '--evidence', 'POINTER')
        require(old['head'] != head,
                f'base が進んだ。先に同期する: head {head} is unchanged but base moved from {old["base"]} to {base};'
                ' run git -c merge.directoryRenames=false merge origin/main, push, capture, then record. Nothing written')
        docs = self.docs_route(snap['requirements'])
        try:
            code = self.reusable(old['head'], head, base, docs=docs)
            change = self.docs_commits and self.l3_change(source, head)
            require(not change, change)
        except GateError as error:
            if error.code == 2:
                raise GateError(f'{error}; or redo L3: {pending}', 2) from None
            raise GateError(f'manual pass not reusable: {error}. Nothing written; next: redo L3: {pending}') from None
        if code:
            evidence = [item for value in manual['evidence'] for item in ('--evidence', value)]
            b = self.cli('--kind', 'manual', '--outcome', 'pass', *evidence, '--reuse-from', source, '--reuse-approval', 'POINTER')
            raise GateError(f'manual pass not carried: the sync brings code {", ".join(code)}. Nothing written; next is one of:'
                            f' (B, needs two sync closures and the owner\'s one-line approval) {b} | (redo L3) {pending}')
        result = dict(outcome='pass', evidence=list(manual['evidence']), source_head=source)
        if 'reuse_approval' in manual:
            result['reuse_approval'] = manual['reuse_approval']
        return result

    def posted_reviews(self, head):
        return [r for page in api(f'{self.endpoint}/pulls/{self.args.pr}/reviews?per_page=100', pages=True) for r in page
                if r['commit_id'] == head and (r['body'] or '').strip()]

    def sync_closure(self, old, current, record, req, snap, docs=False):
        """MG-D13: the helper records {head, base, sync_from} instead of an audited closure for a code-free sync.
        docs=True (D-145 (6), --review-stage docs): lane docs commits judged P3-only, same plan sections and L3."""
        stage = 'docs' if docs else 'sync'
        closure_step = 'record --review-stage closure (監査の closure を要る)'
        require(not current, f'--review-stage {stage} works only in the first record at a new head (新しい head での最初の record'
                ' でだけ使える); a record at this head/base exists, so ' + closure_step)
        require(not docs or self.docs_route(req), '--review-stage docs needs an R3 --packet (R4 and the no-packet route take '
                + closure_step + ')')
        require(old and old['review']['outcome'] == 'pass', f'{stage} closure needs a passed review on the server record; ' + closure_step)
        broad = old['review']['broad']
        # A Gated Amendment needs an audited closure (D-145 (7)); the machine closures take the same Plan contract only.
        same = broad and (broad['plan_commit'], broad['amendments']) == (req['plan_commit'], req['amendments'])
        require(same or not (broad and self.plan_same(broad, req, snap['head'])),
                f'Gated Amendment after the broad with the same plan sections: a fresh broad is not needed; {closure_step}')
        require(same, FRESH_BROAD)
        closure = old['review']['closure']
        origin = broad['head'] if closure is None else closure.get('sync_from', closure['head'])
        bodies = self.posted_reviews(snap['head'])
        require(not bodies, f'the current head has {len(bodies)} PR reviews with a body; read each, then record --review-stage closure'
                f' --pr-reviews {len(bodies)}: ' + ', '.join(f"id={r['id']} submitted_at={r.get('submitted_at') or 'not-submitted'}" for r in bodies))
        try:
            code = self.reusable(origin, snap['head'], snap['base'], docs=docs)
        except GateError as error:
            if error.code == 2:
                raise
            raise GateError(f'{stage} closure not allowed: {error}; {closure_step}') from None
        require(not code, f'{stage} closure not allowed: the sync brings code {", ".join(code)}; {closure_step}')
        if docs:
            change = self.plan_change(origin, snap['head'], req['risk']) or self.l3_change(origin, snap['head'], True)
            require(not change, f'docs closure not allowed: {change}; {closure_step}')
        return dict(outcome='pass', broad=record['review']['broad'], closure=dict(head=snap['head'], base=snap['base'], sync_from=origin))

    def nonci(self, snap):
        server, req = snap['server'], snap['requirements']
        require(req['plan_commit'] != 'pending', 'Plan Gate incomplete')
        if not server and not req['minimum'] and not req['manual'] and not req['r4']:
            return
        require(server is not None, 'workflow record missing')
        record = server['record']
        if req['minimum']:
            # D-107 (3): the review to record outranks stale head/base and the outcome check.
            broad = record['review']['broad']
            require(broad is None or self.plan_same(broad, req, snap['head']), FRESH_BROAD)
            require(broad is not None, BROAD_REQUIRED)
        require((record['head'],record['base']) == (snap['head'],snap['base']), 'stale head/base in workflow record')
        self.validate_review(record, req, snap['head'], snap['base'])
        for kind in ('manual','r4'):
            require(record[kind]['outcome'] == ('pass' if req[kind] else 'not-required'), f'{kind} incomplete')
        # MG-D13: the source pass was checked against the server when recording; recheck the sync from its origin.
        manual, closure = record['manual'], record['review']['closure']
        docs = self.docs_route(req)
        if 'source_head' in manual:
            code = self.reusable(manual['source_head'], snap['head'], snap['base'], code='reuse_approval' not in manual, docs=docs)
            require(not code, f"manual reuse from {manual['source_head']} brings code without approval: {', '.join(code)}")
            change = self.docs_commits and self.l3_change(manual['source_head'], snap['head'])
            require(not change, f"manual reuse from {manual['source_head']}: {change}")
        if closure and 'sync_from' in closure:
            code = self.reusable(closure['sync_from'], snap['head'], snap['base'], docs=docs)
            require(not code, f"sync closure from {closure['sync_from']} brings code: {', '.join(code)}")
            change = self.docs_commits and (self.plan_change(closure['sync_from'], snap['head'], req['risk'])
                                            or self.l3_change(closure['sync_from'], snap['head'], True))
            require(not change, f"sync closure from {closure['sync_from']}: {change}")

    def rules(self, pr):
        desired = decode_json(self.contents(POLICY, pr['base']['sha']))
        require(desired['bypass_actors'] == [] and desired['enforcement'] == 'active'
                and desired['conditions']['ref_name'] == {'include':['refs/heads/main'], 'exclude':[]}, 'main policy invalid')
        effective = [r for page in api(f'{self.endpoint}/rules/branches/main?per_page=100', pages=True) for r in page]
        types = {r['type'] for r in effective}
        require({'pull_request','required_status_checks','deletion','non_fast_forward'} <= types, 'main protection missing')
        matched = False
        for rule in effective:
            if rule['type'] != 'required_status_checks':
                continue
            params = rule['parameters']
            if params.get('strict_required_status_checks_policy') is True and any(
                    c.get('context') == 'Merge gate' and c.get('integration_id') == 15368 for c in params['required_status_checks']):
                detail = api(f"{self.endpoint}/rulesets/{rule['ruleset_id']}")
                require(detail['bypass_actors'] == [], 'merge ruleset has bypass actors')
                require(detail['enforcement'] == 'active' and detail['target'] == 'branch', 'ruleset inactive')
                # MG-D1a: only observed, unmanaged defaults are removed from a copy.
                actual_rules = copy.deepcopy(detail['rules'])
                for actual, expected in zip(actual_rules, desired['rules']):
                    if actual['type'] == expected['type'] == 'pull_request':
                        params, configured = actual['parameters'], expected['parameters']
                        if 'required_reviewers' not in configured:
                            require(params.pop('required_reviewers', []) == [], 'unexpected default: required_reviewers')
                        if 'require_extra_approval_for_unattributed_changes' not in configured:
                            require(params.pop('require_extra_approval_for_unattributed_changes', True) is True,
                                    'unexpected default: require_extra_approval_for_unattributed_changes')
                for key in ('name','conditions','rules'):
                    actual = actual_rules if key == 'rules' else detail[key]
                    require(actual == desired[key], f'main ruleset drift: {key}')
                matched = True
        require(matched, 'strict GitHub Actions Merge gate missing')

    def ci(self, pr):
        pages = api(f"{self.endpoint}/actions/workflows/ci.yml/runs?head_sha={pr['head']['sha']}&per_page=100", pages=True)
        runs = [r for page in pages for r in page['workflow_runs'] if r['head_sha'] == pr['head']['sha']]
        require(runs, 'CI run absent; check same-head runs before recovery dispatch')
        run = max(runs, key=lambda r: (r['created_at'],r['id']))
        require(run['event'] in ('pull_request','workflow_dispatch') and run['path'] == '.github/workflows/ci.yml'
                and run['head_repository']['full_name'] == REPO and run['head_branch'] == pr['head']['ref'], 'wrong CI workflow/source')
        require(run['status'] == 'completed' and run['conclusion'] == 'success', 'latest CI is pending/failed')
        checks = [c for page in api(f"{self.endpoint}/check-suites/{run['check_suite_id']}/check-runs?per_page=100", pages=True) for c in page['check_runs']]
        gate = [c for c in checks if c['name'] == 'Merge gate']
        require(len(gate) == 1 and gate[0]['app']['id'] == 15368 and gate[0]['status'] == 'completed'
                and gate[0]['conclusion'] == 'success', 'Merge gate missing, skipped or wrong app')
        return run['html_url']

    def status(self):
        snap, pr = self.snapshot()
        blockers = []
        if snap['requirements']['plan_commit'] == 'pending':
            return dict(state='Plan Gate', blockers=['Plan Gate incomplete'],
                        next=f'Plan Review, then scripts/plan-approve.sh {self.args.packet} <REVIEWED_HEAD>')
        for check in (lambda: self.nonci(snap), lambda: self.rules(pr), lambda: self.ci(pr)):
            try:
                check()
            except GateError as error:
                if error.code == 2:
                    raise
                blockers.append(str(error))
        return dict(state='merged' if pr.get('merged') else ('Draft' if pr['draft'] else 'Ready'),
                    head=snap['head'], base=snap['base'], blockers=blockers,
                    next='resolve blockers' if blockers else ('owner Ready instruction' if pr['draft'] else 'owner merge instruction'))

    def capture(self):
        snap, _ = self.snapshot(clean=True)
        folder = self.root / '.local/pr-gate'
        require(git('check-ignore', '.local/pr-gate/capture.json').strip(), 'capture folder must be ignored')
        folder.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(mode='w', prefix=f'pr-{self.args.pr}-', suffix='.json', dir=folder, delete=False) as target:
            json.dump(snap, target, ensure_ascii=False, indent=2)
        return dict(capture=target.name, next='independent review or required manual check')

    def record(self):
        args = self.args
        require(args.capture and args.kind and args.outcome, 'record needs capture, kind and outcome', 2)
        path = Path(args.capture).resolve()
        require(path.parent == self.root / '.local/pr-gate', 'capture must belong to this checkout', 2)
        captured = decode_json(path.read_text())
        snap, pr = self.snapshot(clean=True)
        require(captured == snap, 'capture/server/head/base changed; fresh capture required')
        req, server = snap['requirements'], snap['server']
        # MG-D13 / D-127: name the real reason before any other check or write.
        require(args.kind != 'review' or req['minimum'], 'R0/R1 lanes have no review record (--kind review needs a lane with review)', 2)
        old = server['record'] if server else None
        current = old and (old['head'],old['base']) == (snap['head'],snap['base'])
        if current:
            record = copy.deepcopy(old)
        else:
            record = dict(version=1, repo=REPO, pr=args.pr, head=snap['head'], base=snap['base'],
                          review=dict(outcome='pending' if req['minimum'] else 'not-required', broad=None, closure=None),
                          manual=dict(outcome='pending' if req['manual'] else 'not-required',evidence=[]),
                          r4=dict(outcome='pending' if req['r4'] else 'not-required',evidence=[]))
            if old and old['review']['broad']:
                broad=old['review']['broad']
                if self.plan_same(broad,req,snap['head']):
                    record['review']['broad']=copy.deepcopy(broad)
            # MG-D13 A: a manual record without --reuse-from writes its own manual, so it skips this check.
            if old and req['manual'] and old['manual']['outcome'] == 'pass' and args.kind != 'manual':
                record['manual']=self.carry_manual(old,snap)
        if args.kind == 'review' and args.review_stage in ('sync', 'docs'):
            docs = args.review_stage == 'docs'
            require(args.outcome == 'pass' and not (args.pass_model or args.run_ref or args.reviewed_head
                    or args.pr_reviews is not None) and (len(args.evidence) == 1 if docs else not args.evidence),
                    '--review-stage sync takes only --outcome pass (no --pass-model/--run-ref/--evidence/--reviewed-head/--pr-reviews);'
                    ' --review-stage docs also takes exactly one --evidence POINTER to the P3-only judgement', 2)
            if docs:
                pointer(args.evidence[0])
            record['review']=self.sync_closure(old,current,record,req,snap,docs)
            self.validate_review(record,req,snap['head'],snap['base'])
        elif args.kind == 'review':
            require(args.review_stage and args.pass_model and args.run_ref and args.evidence,
                    'review needs stage/model/run-ref/evidence', 2)
            # SPEC-WF-HARNESS5-D8: the audited head must be the captured head, for broad and closure alike.
            require(args.reviewed_head, 'review needs --reviewed-head', 2)
            require(sha(args.reviewed_head) == snap['head'],
                    'reviewed head differs from capture head; record the broad before pushing a fix, or audit the current head')
            # D-107 (4): before any write, the declared count must equal the reviewed head's PR reviews with a body.
            require(args.pr_reviews is not None and args.pr_reviews >= 0,
                    'review needs --pr-reviews (count of PR reviews with a body on the reviewed head)', 2)
            bodies = self.posted_reviews(snap['head'])
            require(len(bodies) == args.pr_reviews,
                    f'--pr-reviews {args.pr_reviews} but the reviewed head has {len(bodies)} PR reviews with a body; read each before recording: '
                    + ', '.join(f"id={r['id']} submitted_at={r.get('submitted_at') or 'not-submitted'}" for r in bodies))
            item=dict(model=pointer(args.pass_model),run_ref=pointer(args.run_ref),evidence=args.evidence)
            broad=record['review']['broad']
            if args.review_stage == 'broad':
                if broad is None or (broad['head'],broad['base']) != (snap['head'],snap['base']):
                    broad=dict(head=snap['head'],base=snap['base'],plan_commit=req['plan_commit'],amendments=req['amendments'],audits=[])
                broad['audits']=[a for a in broad['audits'] if a['run_ref'] != args.run_ref] + [item]
                record['review']=dict(outcome=args.outcome,broad=broad,closure=None)
            else:
                old_broad = old and old['review']['broad']
                require(not old_broad or self.plan_same(old_broad,req,snap['head']), FRESH_BROAD)
                require(server and broad and len(broad['audits']) >= req['minimum'], BROAD_REQUIRED)
                require((broad['head'],broad['base']) != (snap['head'],snap['base']), 'closure needs changed candidate')
                record['review']=dict(outcome=args.outcome,broad=broad,
                                      closure=dict(head=snap['head'],base=snap['base'],audit=item))
            if args.outcome == 'pass':
                self.validate_review(record,req,snap['head'],snap['base'])
        else:
            require(args.reuse_approval is None or args.reuse_from is not None, '--reuse-approval needs --reuse-from', 2)
            result=dict(outcome=args.outcome,evidence=args.evidence)
            if args.reuse_from:
                # MG-D13: B with --reuse-approval; explicit A (code-free) without it. A reused manual may be reused again.
                require(args.kind == 'manual' and args.outcome == 'pass' and old and old['manual']['outcome'] == 'pass', 'server manual pass required')
                source=old['manual'].get('source_head',old['head'])
                require(sha(args.reuse_from) == source and args.evidence == old['manual']['evidence'], 'manual source/evidence mismatch')
                code=self.reusable(old['head'],snap['head'],snap['base'],code=args.reuse_approval is None,docs=self.docs_route(req))
                require(not code, f"the sync brings code {', '.join(code)}; --reuse-from alone needs a code-free sync"
                        ' (B: add --reuse-approval POINTER after two sync closures and the owner\'s approval)')
                change=self.docs_commits and self.l3_change(source,snap['head'])
                require(not change, f'manual pass not reusable: {change}')
                approval=args.reuse_approval or old['manual'].get('reuse_approval')
                result['source_head']=source
                if approval:
                    result['reuse_approval']=pointer(approval)
            require(args.outcome != 'not-required' or not req[args.kind], f'{args.kind} is required')
            record[args.kind]=result
        validate_record(record,args.pr)
        require(api('user')['login'] == OWNER, 'record writer must be repository owner')
        require(self.same_pr(pr,self.pr()) and self.server_record() == server, 'PR/comment changed before write')
        body=MARKER+'\n```json\n'+json.dumps(record,ensure_ascii=False,indent=2)+'\n```'
        endpoint=f"{self.endpoint}/issues/comments/{server['id']}" if server else f'{self.endpoint}/issues/{args.pr}/comments'
        api(endpoint,method='PATCH' if server else 'POST',payload={'body':body})
        require(self.same_pr(pr,self.pr()), 'head/base changed during write; record is stale')
        require(self.server_record()['record'] == record, 'record read-back mismatch')
        return dict(recorded=args.kind,outcome=args.outcome,next='fresh capture before next record')

    def mutate(self, action):
        snap, pr=self.snapshot()
        require(not pr.get('merged'), 'PR already merged')
        self.nonci(snap)
        self.rules(pr)
        if action == 'ready':
            require(pr['draft'], 'PR already Ready')
        else:
            require(not pr['draft'], 'PR is Draft')
            self.ci(pr)
            require(pr['mergeable'] is True and pr['mergeable_state'] == 'clean', 'PR not merge-clean')
        require(self.same_pr(pr,self.pr()) and self.server_record() == snap['server'], 'head/base/record changed before mutation')
        if action == 'ready':
            command(['gh','pr','ready',str(self.args.pr),'--repo',REPO])
        else:
            command(['gh','pr','merge',str(self.args.pr),'--repo',REPO,'--squash','--match-head-commit',snap['head']])
        after=self.pr()
        require(self.same_pr(pr,after), 'head/base changed during mutation; revalidate')
        require((not after['draft']) if action == 'ready' else after.get('merged'), 'mutation did not complete')
        return dict(state=action,next='owner merge instruction' if action == 'ready' else 'done (no closeout PR)')


def report_head(path):
    """H11: the reviewer's report names the SHA it read in exactly one REVIEWED_HEAD=<40hex> line."""
    try:
        text = Path(path).read_text()
    except OSError:
        raise GateError(f'--report {path} unreadable', 2) from None
    lines = [line.strip() for line in text.splitlines() if line.strip().startswith('REVIEWED_HEAD')]
    require(len(lines) == 1 and REPORT_HEAD.fullmatch(lines[0]),
            f'--report needs exactly one line REVIEWED_HEAD=<40 lowercase hex> (found {len(lines)})', 2)
    return REPORT_HEAD.fullmatch(lines[0])[1]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=('status','capture','record','ready','merge'))
    parser.add_argument('--pr',type=int,required=True)
    parser.add_argument('--repo',default=REPO)
    parser.add_argument('--packet',action='append')
    parser.add_argument('--risk',choices=('R0','R1','R2'))
    parser.add_argument('--manual',choices=('required','not-required'))
    parser.add_argument('--json',action='store_true')
    parser.add_argument('--capture')
    parser.add_argument('--kind',choices=('review','manual','r4'))
    parser.add_argument('--outcome',choices=sorted(OUTCOMES))
    parser.add_argument('--evidence',action='append',default=[])
    parser.add_argument('--review-stage',choices=('broad','closure','sync','docs'))
    parser.add_argument('--pass-model')
    parser.add_argument('--run-ref')
    parser.add_argument('--reviewed-head')
    parser.add_argument('--report',help='reviewer report file holding one REVIEWED_HEAD=<40hex> line')
    parser.add_argument('--pr-reviews',type=int)
    parser.add_argument('--reuse-from')
    parser.add_argument('--reuse-approval')
    args=parser.parse_args()
    try:
        require(args.pr > 0, 'PR number must be positive', 2)
        require(not args.packet or len(args.packet) == 1, 'exactly one packet may be specified', 2)
        args.packet = args.packet[0] if args.packet else None
        if args.report is not None:
            require(args.reviewed_head is None, '--report and --reviewed-head are exclusive', 2)
            args.reviewed_head = report_head(args.report)
        gate=Gate(args)
        result=gate.mutate(args.action) if args.action in ('ready','merge') else getattr(gate,args.action)()
        print(json.dumps(result,ensure_ascii=False) if args.json else '\n'.join(f'{k}: {v}' for k,v in result.items()))
        return 0
    except GateError as error:
        print(f'pr-gate: {error}',file=sys.stderr)
        return error.code
    except (ValueError,KeyError,TypeError,OSError,AttributeError) as error:
        print(f'pr-gate: invalid input or API response ({type(error).__name__})',file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
