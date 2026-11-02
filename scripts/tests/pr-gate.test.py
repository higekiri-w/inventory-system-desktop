#!/usr/bin/env python3
"""SPEC-MERGE-EVIDENCE / MG-D1a, MG-D5..D8, MG-D13: synthetic fixtures, no GitHub mutation."""
import argparse
import contextlib
import copy
import importlib.util
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
sys.dont_write_bytecode = True
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('pr_gate', ROOT/'scripts/pr-gate.py')
g=importlib.util.module_from_spec(spec); spec.loader.exec_module(g)
H='a'*40; B='b'*40; P='c'*40; OLD='d'*40
REQ=dict(risk='R3',plan_commit=P,amendments=[],minimum=2,manual=True,r4=True)
APPROVAL='https://example.invalid/owner'
# D-107 (3): literals from the packet's Boundary / Wire Contract, not from the helper.
FRESH_BROAD='broad Plan contract changed; fresh broad required: record --review-stage broad at the current head (a closure cannot carry this broad)'
BROAD_REQUIRED='broad required: record --review-stage broad at the current head until it has Final Review Minimum audits (no usable broad on the server; a closure cannot replace it)'

def good_record():
    return dict(version=1,repo=g.REPO,pr=7,head=H,base=B,
                review=dict(outcome='pass',broad=dict(head=H,base=B,plan_commit=P,amendments=[],
                    audits=[dict(model=m,run_ref=m,evidence=[f'https://example.invalid/{m}']) for m in ('sonnet','opus')]),closure=None),
                manual=dict(outcome='pass',evidence=['https://example.invalid/manual']),
                r4=dict(outcome='pass',evidence=['https://example.invalid/approval']))

def args(**kw):
    value=dict(repo=g.REPO,pr=7,packet=None,risk='R0',manual='not-required',capture=None,
               kind='review',outcome='pass',review_stage='broad',pass_model='sonnet',run_ref='new',
               evidence=['https://example.invalid/result'],reuse_from=None,reuse_approval=None,reviewed_head=H,pr_reviews=0)
    return argparse.Namespace(**(value|kw))

class Records(unittest.TestCase):
    def setUp(self):
        self.gate=object.__new__(g.Gate); self.gate.args=args(); self.gate.endpoint='repos/'+g.REPO
        self.record=good_record()
        self.snap=dict(head=H,base=B,requirements=copy.deepcopy(REQ),server=dict(record=self.record))
    def test_valid_and_all_pending_gates(self):
        g.validate_record(self.record,7); self.gate.nonci(self.snap)
        for kind in ('review','manual','r4'):
            for outcome in ('pending','fail','not-required'):
                data=copy.deepcopy(self.snap); data['server']['record'][kind]['outcome']=outcome
                with self.subTest(kind=kind,outcome=outcome),self.assertRaises(g.GateError): self.gate.nonci(data)
    def test_stale_version(self):
        for key in ('head','base'):
            data=copy.deepcopy(self.snap);data['server']['record'][key]=OLD
            with self.assertRaises(g.GateError):self.gate.nonci(data)
    def test_plan_contract_change_reported_before_stale(self):
        # T1-1 / D-107 (3): a changed Plan contract is named before stale head/base.
        data=copy.deepcopy(self.snap);record=data['server']['record']
        record['head']=OLD;record['review']['broad']['amendments']=[OLD]
        with self.assertRaisesRegex(g.GateError,'^'+re.escape(FRESH_BROAD)+'$'):self.gate.nonci(data)
    def test_stale_without_contract_change_keeps_stale(self):
        # T1-4 / MG-D8: the same contract at an old head stays a stale record.
        data=copy.deepcopy(self.snap);data['server']['record']['head']=OLD
        with self.assertRaisesRegex(g.GateError,'^stale head/base in workflow record$'):self.gate.nonci(data)
    def test_broad_contract_and_minimum(self):
        for key,value in [('plan_commit',None),('plan_commit',OLD),('amendments',[OLD]),('audits',[])]:
            data=copy.deepcopy(self.snap);data['server']['record']['review']['broad'][key]=value
            with self.assertRaises(g.GateError): self.gate.nonci(data)
    def test_closure_required_for_head_or_base_change(self):
        for key in ('head','base'):
            data=copy.deepcopy(self.snap);data['server']['record']['review']['broad'][key]=OLD
            with self.assertRaises(g.GateError):self.gate.nonci(data)
            data['server']['record']['review']['closure']=dict(head=H,base=B,audit=dict(model='opus',run_ref='closure',evidence=['https://example.invalid/closure']))
            self.gate.nonci(data)
    def test_wire_fail_closed(self):
        mutants=[]
        for key,value in [('repo','other/repo'),('pr',True),('head','abc'),('version',2)]:
            data=copy.deepcopy(self.record);data[key]=value;mutants.append(data)
        for kind in ('manual','r4'):
            data=copy.deepcopy(self.record);data[kind]['evidence']=[];mutants.append(data)
        data=copy.deepcopy(self.record);data['review']['broad']['audits'][1]['run_ref']='sonnet';mutants.append(data)
        # MG-D13 (V3): source_head alone is the A shape; reuse_approval without source_head is incomplete.
        data=copy.deepcopy(self.record);data['manual']['reuse_approval']=APPROVAL;mutants.append(data)
        for data in mutants:
            with self.assertRaises(g.GateError):g.validate_record(data,7)
    def test_validate_legacy_reuse_record(self):
        # V1 / MG-D13: today's B record (source_head + reuse_approval) and the A shape (source_head) stay valid.
        for extra in (dict(source_head=OLD,reuse_approval=APPROVAL),dict(source_head=OLD)):
            data=copy.deepcopy(self.record);data['manual'].update(extra)
            with self.subTest(extra=sorted(extra)):self.assertEqual(g.validate_record(data,7),data)
        data['manual']['outcome']='pending'
        with self.assertRaises(g.GateError):g.validate_record(data,7)
    def test_validate_closure_shape_exclusive(self):
        # V4 / MG-D13: exactly one closure shape; the sync shape still needs its broad.
        audit=dict(model='opus',run_ref='closure',evidence=['https://example.invalid/closure'])
        for closure in (dict(head=H,base=B,audit=audit),dict(head=H,base=B,sync_from=OLD)):
            data=copy.deepcopy(self.record);data['review']['closure']=closure
            with self.subTest(ok=sorted(closure)):g.validate_record(data,7)
        for closure,broad in ((dict(head=H,base=B,audit=audit,sync_from=OLD),True),(dict(head=H,base=B),True),
                              (dict(head=H,base=B,sync_from='abc'),True),(dict(head=H,base=B,sync_from=OLD),False)):
            data=copy.deepcopy(self.record);data['review']['closure']=closure
            if not broad:data['review']['broad']=None
            with self.subTest(bad=sorted(closure),broad=broad),self.assertRaises(g.GateError):g.validate_record(data,7)
    def test_comment_author_duplicate_malformed(self):
        body=g.MARKER+'\n```json\n'+json.dumps(self.record)+'\n```'
        item=dict(id=1,user=dict(login=g.OWNER),body=body,updated_at='today')
        with patch.object(g,'api',return_value=[[item]]):self.assertEqual(self.gate.server_record()['record'],self.record)
        for items in ([item,item],[item|dict(user=dict(login='attacker'))],[item|dict(body=g.MARKER+'bad')]):
            with patch.object(g,'api',return_value=[items]),self.assertRaises(g.GateError):self.gate.server_record()
    def test_r0_still_requires_explicit_manual(self):
        data=copy.deepcopy(self.snap); data['server']=None
        data['requirements'].update(minimum=0,r4=False,manual=True)
        with self.assertRaises(g.GateError):self.gate.nonci(data)
        data['requirements']['manual']=False;self.gate.nonci(data)
    def test_packet_schema(self):
        # T-H1: new template (no Evidence Mode / Execution Mode lines) is accepted and returns no mode.
        text='## Workflow State\n'+''.join(f'- {k}: {v}\n' for k,v in {
            'Phase':'implementing','Risk':'R3',
            'Plan Commit':P,'Amendments':'none','Coordinator':'owner','Writer':'codex',
            'Plan Reviewer':'opus','Final Reviewer':'sonnet','Final Review Minimum':'2','Human Gate':'ready,merge,manual'}.items())+'\n## Risk\nRisk: R3\n'
        parsed=g.parse_packet(text)
        self.assertEqual(parsed['minimum'],2)
        self.assertNotIn('mode',parsed)
        marker='- Phase: implementing'
        mutants=[(marker,'- Evidence Mode: legacy\n'+marker),(marker,'- Evidence Mode: mystery\n'+marker),
                 ('Final Review Minimum: 2','Final Review Minimum: 0'),('Human Gate: ready,merge,manual','Human Gate: none')]
        mutants+=[(marker,f'- {field}: required\n'+marker) for field in ('Reviewed Content HEAD','Final Exact-HEAD Evidence','Hosted CI Requirement')]
        # Valueless lines must not vanish: Evidence Mode at section end and mid-section, and an empty required field.
        mutants+=[('\n\n## Risk','\n- Evidence Mode:\n## Risk'),(marker,'- Evidence Mode:\n'+marker),('Amendments: none','Amendments:')]
        for source,dest in mutants:
            with self.subTest(dest=dest),self.assertRaises(g.GateError):g.parse_packet(text.replace(source,dest,1))
        # PH3 / D-145 (3): an old packet's Phase value and role lines are read but not evaluated (rewritten from the
        # Phase-enum, empty-role and pending-in-implementing rejections); pending only means the plan is not approved.
        for source,dest in (('Phase: implementing','Phase: local-verified'),('Writer: codex','Writer:'),('- Writer: codex\n',''),(P,'pending')):
            with self.subTest(accepted=dest):
                self.assertEqual(g.parse_packet(text.replace(source,dest,1))['risk'],'R3')
        fields=g.workflow_fields(text.replace(marker,'- Evidence Mode:\n'+marker,1))
        self.assertEqual((fields['Evidence Mode'],fields['Phase']),('','implementing'))
        # T-H2: old template lines are accepted and Execution Mode is not evaluated.
        for mode in ('fable-window','dual-vendor-no-fable','codex-only','waterfall（任意の文字列）'):
            with self.subTest(mode=mode):
                old=text.replace(marker,f'- Evidence Mode: github\n{marker}\n- Execution Mode: {mode}',1)
                self.assertEqual(g.parse_packet(old),parsed)

class RecordLifecycle(unittest.TestCase):
    setUp = Records.setUp
    # MG-D13: a server pass is carried (A) or reused with approval (B) at a synced head; closures follow.
    # real=False stubs the sync checks (H2/H3); real=True leaves them to the synthetic repo patched in by the caller.
    def exercise(self, old, kind='manual', outcome='pass', source=OLD, values=None, mutate_capture=False, captured_requirements=None,
                 head=H, base=B, req=REQ, approval=APPROVAL, stage='closure', reviews=(), real=False, **extra):
        self.writes=[]
        with tempfile.TemporaryDirectory() as tmp:
            self.gate.root=Path(tmp)
            folder=Path(tmp)/'.local/pr-gate';folder.mkdir(parents=True)
            server=None if old is None else dict(id=1,body='previous',updated_at='before',record=copy.deepcopy(old))
            snap=dict(version=1,repo=g.REPO,pr=7,head=head,base=base,packet=None,requirements=req,server=server)
            capture=folder/'capture.json';capture.write_text(json.dumps(snap|dict(requirements=captured_requirements or req)))
            if mutate_capture and server:server['updated_at']='changed'
            self.gate.args=args(**dict(capture=str(capture),kind=kind,outcome=outcome,review_stage=stage,reviewed_head=head,
                reuse_from=source if kind=='manual' else None,reuse_approval=approval if source and kind=='manual' else None,
                evidence=values if values is not None else ['https://example.invalid/manual'])|extra)
            pr=dict(head=dict(sha=head),base=dict(sha=base))
            state={'server':server}
            def transport(path,method='GET',payload=None,**kwargs):
                if path=='user':return dict(login=g.OWNER)
                if '/pulls/7/reviews' in path and method=='GET':return [list(reviews)]
                self.assertIn('/comments',path)
                self.writes.append(path)
                match=payload['body'].split('```json\n')[1].split('\n```')[0]
                state['server']=dict(id=1,body=payload['body'],updated_at='after',record=json.loads(match))
                return state['server']
            with contextlib.ExitStack() as stack:
                stack.enter_context(patch.object(self.gate,'snapshot',return_value=(snap,pr)))
                stack.enter_context(patch.object(self.gate,'pr',return_value=pr))
                stack.enter_context(patch.object(self.gate,'server_record',side_effect=lambda:state['server']))
                stack.enter_context(patch.object(g,'api',side_effect=transport))
                if not real:
                    stack.enter_context(patch.object(self.gate,'sync_merges',return_value=[]))
                    stack.enter_context(patch.object(self.gate,'code_paths',return_value=[]))
                self.gate.record()
            return state['server']['record']
    def previous(self):
        old=good_record();old['head']=OLD;old['review']['broad']['head']=OLD
        return old
    def test_reuse_then_closure_order(self):
        current=self.exercise(self.previous())
        self.assertEqual(current['manual']['source_head'],OLD)
        self.assertEqual(current['r4']['outcome'],'pending')
        self.assertEqual(current['review']['outcome'],'pending')
        complete=self.exercise(current,kind='review',source=None)
        self.assertEqual(complete['review']['closure']['head'],H)
        self.assertEqual(complete['manual']['outcome'],'pass')
    def test_source_record_missing_changed_or_not_pass(self):
        mutants=[None]
        for outcome in ('pending','fail','not-required'):
            old=self.previous();old['manual']['outcome']=outcome;mutants.append(old)
        for old in mutants:
            with self.assertRaises(g.GateError):self.exercise(old)
        with self.assertRaises(g.GateError):self.exercise(self.previous(),source=P)
        with self.assertRaises(g.GateError):self.exercise(self.previous(),values=['replaced'])
        with self.assertRaises(g.GateError):self.exercise(self.previous(),mutate_capture=True)
    def test_closure_first_carries_or_stops_and_keeps_b(self):
        # MG-D13 (replaces the closure-first loss): a code-free sync carries the manual pass (A); a code sync
        # stops the closure before any write, so B stays available on the untouched server record.
        current=self.exercise(self.previous(),kind='review',source=None)
        self.assertEqual(current['manual'],dict(outcome='pass',evidence=['https://example.invalid/manual'],source_head=OLD))
        with patch.object(self.gate,'sync_merges',return_value=[]),patch.object(self.gate,'code_paths',return_value=['src/app.ts']):
            with self.assertRaisesRegex(g.GateError,'src/app.ts'):self.exercise(self.previous(),kind='review',source=None,real=True)
        self.assertEqual(self.writes,[])
        self.assertEqual(self.exercise(self.previous())['manual']['reuse_approval'],APPROVAL)
    def test_old_shape_capture_requires_fresh_capture(self):
        # T-H7b: a capture made before the requirements lost 'mode' is rejected, not silently accepted.
        with self.assertRaisesRegex(g.GateError,'fresh capture required'):
            self.exercise(self.previous(),captured_requirements=REQ|dict(mode='codex-only'))
    def test_closure_without_server_broad_names_next_step(self):
        # T1-4 / D-107 (3): no usable broad (none recorded, or below minimum) names the broad to record.
        empty=self.previous();empty['review']=dict(outcome='pending',broad=None,closure=None)
        short=self.previous();short['review']['broad']['audits']=short['review']['broad']['audits'][:1]
        for old in (None,empty,short):
            with self.subTest(old=old and len((old['review']['broad'] or {}).get('audits',[]))):
                with self.assertRaises(g.GateError) as caught:self.exercise(old,kind='review',source=None)
                self.assertEqual(str(caught.exception),BROAD_REQUIRED)
    def test_changed_plan_requires_new_broad(self):
        old=self.previous();old['review']['broad']['plan_commit']=P.replace('c','f')
        with self.assertRaises(g.GateError):self.exercise(old,kind='review',source=None)


# MG-D13: synthetic git histories in a tmpdir; the helper's git() is pointed there, never at this clone.
GEN_TRACE='docs/function-design/90-traceability.md';GEN_BIND='src/lib/bindings.ts';DLOG='docs/decision-log.md'
CLASSIFIER=(ROOT/'scripts/ci/classify-changes.sh').read_text()
SEED={'docs/a.md':'a\n','src/app.ts':'app\n',DLOG:'# log\n\nD-1\n',GEN_TRACE:'t1\nt2\nt3\n',GEN_BIND:'b1\n',
      'docs/shared.md':''.join(f'{i}\n' for i in range(1,13)),'public/help.html':'<p>help page body for rename</p>\n',
      'docs/dir/x.md':'x\n','docs/dir/y.md':'y\n','src/x.ts':'export const x = 1\n'}
LANE_EVIDENCE=['https://example.invalid/manual']

class Synth:
    """main and a lane branch; `source` is the lane head that passed L3. Syncs merge like the lane does."""
    def __init__(self,tmp):
        self.dir=tmp;self.git('init','-q','-b','main');self.git('config','user.name','test');self.git('config','user.email','test@example.invalid')
        self.base0=self.base=self.commit(SEED,'base');self.git('switch','-qc','lane')
        self.source=self.commit({'docs/lane.md':'lane\n'},'lane work')
    def git(self,*args,check=True):
        return subprocess.run(['git','-C',self.dir,*args],check=check,capture_output=True,text=True).stdout.strip()
    def write(self,files):
        for path,text in files.items():
            target=Path(self.dir,path)
            if text is None:target.unlink()
            else:target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text)
    def commit(self,files,msg='change'):
        self.write(files);self.git('add','-A');self.git('commit','-q','--allow-empty','-m',msg);return self.git('rev-parse','HEAD')
    def main(self,files):
        self.git('switch','-q','main');self.base=self.commit(files,'main');self.git('switch','-q','lane');return self.base
    def sync(self,ref='main',files=None,union=False):
        """Merge ref into the lane; `files` resolves conflicts or edits inside the merge; `union` resolves the decision log."""
        self.git('-c','merge.directoryRenames=false','merge','--no-ff','--no-commit','-q',ref,check=False)
        if union:
            merged=subprocess.run(['git','-C',self.dir,'merge-file','-p','--union','--object-id',f'HEAD:{DLOG}',
                                   self.git('merge-base','HEAD','MERGE_HEAD')+f':{DLOG}',f'MERGE_HEAD:{DLOG}'],
                                  check=True,capture_output=True,text=True).stdout
            self.write({DLOG:merged})
        self.write(files or {});self.git('add','-A');self.git('commit','-q','--no-edit');return self.git('rev-parse','HEAD')
    def move(self,source,target):
        Path(self.dir,target).parent.mkdir(parents=True,exist_ok=True);self.git('mv',source,target)
    def bridge(self,*args,**kw):return g.command(['git','-C',self.dir,*args],**kw)

class SyncCase(unittest.TestCase):
    def setUp(self):
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup)
        self.repo=Synth(tmp.name)
        self.gate=object.__new__(g.Gate);self.gate.args=args();self.gate.endpoint='repos/'+g.REPO
        for target,name,kw in ((g,'git',dict(side_effect=self.repo.bridge)),(self.gate,'contents',dict(return_value=CLASSIFIER))):
            patcher=patch.object(target,name,**kw);patcher.start();self.addCleanup(patcher.stop)
    def head(self):return self.repo.git('rev-parse','HEAD')
    def merges(self,origin=None,head=None,base=None):
        return self.gate.sync_merges(origin or self.repo.source,head or self.head(),base or self.repo.base)
    def refuse(self,*texts,code=1,**kw):
        with self.assertRaises(g.GateError) as caught:self.merges(**kw)
        self.assertEqual(caught.exception.code,code,str(caught.exception))
        for text in texts:self.assertIn(text,str(caught.exception))
        return str(caught.exception)

class SyncOnly(SyncCase):
    """G1..G13: the sync-only condition (packet H2)."""
    def test_sync_only_single_auto_merge(self):
        self.repo.main({'docs/b.md':'b\n'});head=self.repo.sync()
        self.assertEqual([m[0] for m in self.merges()],[head])
    def test_sync_only_uses_directory_renames_false(self):
        # The sync runs with merge.directoryRenames=false; merge-tree must too, or the lane's new file conflicts.
        self.repo.commit({'docs/dir/z.md':'lane file\n'});source=self.head()
        self.repo.main({'docs/dir/x.md':None,'docs/dir/y.md':None,'docs/moved/x.md':'x\n','docs/moved/y.md':'y\n'})
        head=self.repo.sync()
        self.assertEqual([m[0] for m in self.merges(origin=source)],[head])
    def test_sync_only_two_merges(self):
        self.repo.main({'docs/b.md':'b\n'});first=self.repo.sync()
        self.repo.main({'docs/c.md':'c\n'});second=self.repo.sync()
        self.assertEqual([m[0] for m in self.merges()],[first,second])
    def test_sync_only_shared_file_context_shift(self):
        lines=SEED['docs/shared.md'].splitlines(keepends=True)
        self.repo.commit({'docs/shared.md':''.join(lines[:8]+['lane\n']+lines[9:])});source=self.head()
        self.repo.main({'docs/shared.md':'new top\n'+''.join(lines[:3]+['main\n']+lines[4:])})
        head=self.repo.sync()
        self.assertEqual([m[0] for m in self.merges(origin=source)],[head])
    def test_sync_only_generated_conflict_regenerated(self):
        self.repo.commit({GEN_TRACE:'t1\nlane\nt3\n'});source=self.head()
        self.repo.main({GEN_TRACE:'t1\nmain\nt3\n'})
        head=self.repo.sync(files={GEN_TRACE:'t1\nregenerated\nt3\n'})
        self.assertEqual([m[0] for m in self.merges(origin=source)],[head])
    def test_sync_only_generated_regen_commit(self):
        self.repo.main({'docs/b.md':'b\n'});merge=self.repo.sync()
        head=self.repo.commit({GEN_BIND:'b2\n'},'regenerate')
        self.assertEqual([m[0] for m in self.merges()],[merge])
        self.assertNotEqual(head,merge)
    def test_sync_only_rejects_lane_commit(self):
        self.repo.main({'docs/b.md':'b\n'});self.repo.sync()
        lane=self.repo.commit({'docs/lane.md':'more\n',GEN_BIND:'b2\n'})
        self.refuse('lane commit '+lane,'docs/lane.md')
    def test_sync_only_rejects_manual_conflict_resolution(self):
        self.repo.commit({'docs/a.md':'lane\n'});source=self.head()
        self.repo.main({'docs/a.md':'main\n'})
        head=self.repo.sync(files={'docs/a.md':'by hand\n'})
        self.refuse('merge '+head,'docs/a.md',origin=source)
    def test_sync_only_rejects_edit_inside_merge(self):
        self.repo.main({'docs/b.md':'b\n'})
        head=self.repo.sync(files={'docs/lane.md':'edited inside the merge\n'})
        self.refuse('merge '+head,'docs/lane.md')
    def test_sync_only_rejects_non_main_second_parent(self):
        # A side branch merged mid-list, then a normal last sync: only the ancestor check catches the side merge.
        self.repo.git('branch','side',self.repo.base0);self.repo.git('switch','-q','side')
        side=self.repo.commit({'docs/side.md':'side\n'});self.repo.git('switch','-q','lane')
        bad=self.repo.sync('side')
        self.repo.main({'docs/b.md':'b\n'});self.repo.sync()
        self.refuse('merge '+bad,side,'not in main')
        # Judgment impossible (unknown base object) is exit 2, never satisfied, and names git fetch origin.
        with self.assertRaises(g.GateError) as caught:self.gate.reusable(self.repo.source,self.head(),'f'*40)
        self.assertEqual(caught.exception.code,2);self.assertIn('git fetch origin',str(caught.exception))
    def test_sync_only_rejects_stale_last_parent(self):
        self.repo.main({'docs/b.md':'b\n'});merge=self.repo.sync()
        stale=self.repo.base;self.repo.main({'docs/c.md':'c\n'})
        self.refuse('last sync merge '+merge,stale)
        self.repo.commit({GEN_BIND:'b2\n'},'regenerate')
        self.refuse('last sync merge '+merge,stale)
    def test_sync_only_rejects_ambiguous_merge_base(self):
        # Criss-cross: the lane merges main's M1 while main merges the lane's L1.
        l1=self.repo.source;self.repo.main({'docs/m1.md':'m1\n'});origin=self.repo.sync()
        self.repo.git('switch','-q','main');self.repo.git('merge','--no-ff','-q','-m','main takes lane',l1);self.repo.base=self.head()
        self.repo.git('switch','-q','lane');self.repo.main({'docs/m2.md':'m2\n'});head=self.repo.sync()
        self.refuse('merge '+head,'ambiguous merge-base',origin=origin)
    def test_sync_only_generated_exception_is_exact(self):
        self.repo.main({'docs/b.md':'b\n'});self.repo.sync()
        for path in (GEN_TRACE+'.bak','src/lib/bindings.tsx'):
            with self.subTest(path=path):
                lane=self.repo.commit({path:'near miss\n'})
                self.refuse('lane commit '+lane,path)
                self.repo.git('reset','-q','--hard','HEAD~1')
        # A normal path renamed into a generated path is a deletion of the normal path (1-parent and in-merge).
        self.repo.git('reset','-q','--hard',self.repo.source);source=self.repo.commit({GEN_TRACE:None})
        self.repo.main({'docs/c.md':'c\n'});self.repo.sync()
        self.repo.move('public/help.html',GEN_TRACE);self.repo.git('commit','-qm','rename');lane=self.head()
        self.refuse('lane commit '+lane,'public/help.html',origin=source)
        self.repo.git('reset','-q','--hard','HEAD~1');self.repo.main({'docs/d.md':'d\n'})
        self.repo.git('-c','merge.directoryRenames=false','merge','--no-ff','--no-commit','-q','main')
        self.repo.move('public/help.html',GEN_TRACE);self.repo.git('commit','-q','--no-edit');head=self.head()
        self.refuse('merge '+head,'public/help.html',origin=source)
    def test_sync_only_decision_log_union_conflict(self):
        self.repo.commit({DLOG:SEED[DLOG]+'\nD-2 lane\n'});source=self.head()
        self.repo.main({DLOG:SEED[DLOG]+'\nD-3 main\n'})
        head=self.repo.sync(union=True)
        self.assertEqual([m[0] for m in self.merges(origin=source)],[head])
        self.assertIn('D-2 lane',self.repo.git('show',f'{head}:{DLOG}'))
        for text in (SEED[DLOG]+'\nD-3 main\n',SEED[DLOG]+'\nD-3 main\n\nD-2 lane\n'):
            with self.subTest(text=text):
                self.repo.git('reset','-q','--hard',source)
                bad=self.repo.sync(files={DLOG:text})
                self.refuse('merge '+bad,DLOG,origin=source)

class CodeFree(SyncCase):
    """C1..C5: the code-free condition (packet H3) over the main side of each sync merge."""
    def code(self,files):
        self.repo.main(files);self.repo.sync();return self.gate.code_paths(self.merges(),self.repo.base)
    def test_code_free_docs_only(self):
        self.assertEqual(self.code({'docs/b.md':'b\n','docs/plans/2026-01-01-x.md':'p\n',GEN_TRACE:'t1\nt2\nt3\nt4\n'}),[])
    def test_code_free_rejects_rust(self):
        self.assertEqual(self.code({'docs/b.md':'b\n','src-tauri/src/x.rs':'fn x() {}\n'}),['src-tauri/src/x.rs'])
    def test_code_free_rejects_frontend_and_generated_binding(self):
        self.assertEqual(self.code({'docs/b.md':'b\n',GEN_BIND:'b2\n','package.json':'{}\n'}),['package.json',GEN_BIND])
    def test_code_free_rejects_unknown_and_control(self):
        self.assertEqual(self.code({'weird/x.bin':'?\n'}),['weird/x.bin'])
        # The second sync adds a control path; the union over both merges names both.
        self.assertEqual(self.code({'scripts/pr-gate.py':'#\n','docs/c.md':'c\n'}),['scripts/pr-gate.py','weird/x.bin'])
    def test_code_free_rename_both_sides(self):
        self.repo.git('switch','-q','main');self.repo.move('src/x.ts','docs/x.md');self.repo.git('commit','-qm','move')
        self.repo.base=self.head();self.repo.git('switch','-q','lane');self.repo.sync()
        self.assertIn('src/x.ts',self.gate.code_paths(self.merges(),self.repo.base))

NO_MANUAL=REQ|dict(manual=False,r4=False)
LOW=dict(risk='R0',plan_commit=None,amendments=[],minimum=0,manual=True,r4=False)
SYNC=dict(pass_model=None,run_ref=None,values=[],reviewed_head=None,pr_reviews=None)

class SyncRecords(SyncCase):
    """R1..R13, S2..S7, V5..V6, G5, C6: record and status against a synthetic repo."""
    exercise=RecordLifecycle.exercise
    carried=dict(outcome='pass',evidence=LANE_EVIDENCE)
    def old(self,head=None,base=None,req=REQ,manual='pass',**manual_extra):
        record=good_record();record['head']=head or self.repo.source;record['base']=base or self.repo.base0
        if req['minimum']:record['review']['broad'].update(head=record['head'],base=record['base'])
        else:record['review']=dict(outcome='not-required',broad=None,closure=None)
        record['manual']=dict(outcome=manual,evidence=list(LANE_EVIDENCE))|manual_extra
        if not req['r4']:record['r4']=dict(outcome='not-required',evidence=[])
        return record
    def rec(self,old,kind='r4',req=REQ,source=None,**kw):
        return self.exercise(old,kind=kind,source=source,head=self.head(),base=self.repo.base,req=req,real=True,**kw)
    def stop(self,old,*texts,**kw):
        with self.assertRaises(g.GateError) as caught:self.rec(old,**kw)
        self.assertEqual(self.writes,[])
        for text in texts:self.assertIn(text,str(caught.exception))
        return str(caught.exception)
    def sync_review(self,old,**kw):return self.rec(old,kind='review',req=NO_MANUAL,stage='sync',**SYNC|kw)
    def sync_stop(self,old,*texts):return self.stop(old,*texts,kind='review',req=NO_MANUAL,stage='sync',**SYNC)
    def docs_sync(self,name='b'):self.repo.main({f'docs/{name}.md':name+'\n'});return self.repo.sync()
    def code_sync(self):self.repo.main({'src/app.ts':'app2\n'});return self.repo.sync()
    def status(self,record,req):
        # Status reads git and the classifier only: a GitHub call here would be a write path.
        with patch.object(g,'api',side_effect=AssertionError('status must not call GitHub here')):
            self.gate.nonci(dict(head=self.head(),base=self.repo.base,requirements=req,server=dict(record=record)))

    def test_auto_carry_on_sync_record(self):
        self.docs_sync()
        for kind in ('r4','review'):
            with self.subTest(kind=kind):
                record=self.rec(self.old(),kind=kind)
                self.assertEqual(record['manual'],self.carried|dict(source_head=self.repo.source))
        for outcome in ('pending','fail'):
            with self.subTest(outcome=outcome):
                self.assertEqual(self.rec(self.old(manual=outcome))['manual'],dict(outcome='pending',evidence=[]))
    def test_auto_carry_keeps_source_and_evidence(self):
        self.docs_sync()
        old=self.old(source_head=self.repo.base0,reuse_approval=APPROVAL)
        old['manual']['evidence']=['https://example.invalid/l3-a','https://example.invalid/l3-b']
        self.assertEqual(self.rec(old)['manual'],old['manual'])
        old=self.old()
        self.assertEqual(self.rec(old)['manual']['source_head'],old['head'])
    def test_code_sync_stops_non_manual_record(self):
        self.code_sync()
        message=self.stop(self.old(),'src/app.ts','--outcome pass --evidence '+LANE_EVIDENCE[0],
                          '--reuse-from '+self.repo.source+' --reuse-approval POINTER','--outcome pending',kind='review')
        self.assertIn('owner',message)
        # Nothing was written, so B still works on the same server record.
        self.assertEqual(self.rec(self.old(),kind='manual',source=self.repo.source,approval=APPROVAL)['manual']['reuse_approval'],APPROVAL)
    def test_lane_commit_stops_with_reason(self):
        self.docs_sync();lane=self.repo.commit({'docs/lane.md':'more\n'})
        with patch.object(self.gate,'code_paths',wraps=self.gate.code_paths) as spy:
            message=self.stop(self.old(),'lane commit '+lane,'--outcome pending')
        spy.assert_not_called()
        self.assertNotIn('--reuse-approval',message)
    def test_b_reuse_with_approval(self):
        self.code_sync()
        record=self.rec(self.old(),kind='manual',source=self.repo.source,approval=APPROVAL)
        self.assertEqual(record['manual'],self.carried|dict(source_head=self.repo.source,reuse_approval=APPROVAL))
        with self.assertRaises(g.GateError) as caught:self.rec(self.old(),kind='manual',reuse_approval=APPROVAL)
        self.assertEqual(caught.exception.code,2)
    def test_auto_carry_chain(self):
        self.docs_sync();first=self.rec(self.old())
        self.docs_sync('c');second=self.rec(first)
        self.assertEqual(second['manual'],self.carried|dict(source_head=self.repo.source))
    def test_b_then_docs_sync_keeps_approval(self):
        self.code_sync();b=self.rec(self.old(),kind='manual',source=self.repo.source,approval=APPROVAL)
        self.docs_sync()
        self.assertEqual(self.rec(b)['manual'],self.carried|dict(source_head=self.repo.source,reuse_approval=APPROVAL))
    def test_b_source_must_match_original(self):
        first_head=self.docs_sync();first=self.rec(self.old())
        self.code_sync()
        with self.assertRaisesRegex(g.GateError,'manual source/evidence mismatch'):
            self.rec(first,kind='manual',source=first_head,approval=APPROVAL)
        self.assertEqual(self.writes,[])
        self.assertEqual(self.rec(first,kind='manual',source=self.repo.source,approval=APPROVAL)['manual']['source_head'],self.repo.source)
    def test_explicit_a_requires_code_free(self):
        coded=self.code_sync()
        with self.assertRaisesRegex(g.GateError,'src/app.ts'):self.rec(self.old(),kind='manual',source=self.repo.source,approval=None)
        self.assertEqual(self.writes,[])
        self.docs_sync()
        record=self.rec(self.old(head=coded,base=self.repo.base0),kind='manual',source=coded,approval=None)
        self.assertEqual(record['manual'],self.carried|dict(source_head=coded))
    def test_r4_never_carried(self):
        self.docs_sync()
        self.assertEqual(self.rec(self.old(),kind='review')['r4'],dict(outcome='pending',evidence=[]))
    def test_b_then_explicit_a_keeps_approval(self):
        self.code_sync();b=self.rec(self.old(req=LOW),kind='manual',req=LOW,source=self.repo.source,approval=APPROVAL)
        self.docs_sync()
        a=self.rec(b,kind='manual',req=LOW,source=self.repo.source,approval=None)
        self.assertEqual(a['manual'],self.carried|dict(source_head=self.repo.source,reuse_approval=APPROVAL))
        self.status(a,LOW)
    def test_base_only_change_stops_non_manual_record(self):
        self.repo.main({'docs/b.md':'b\n'})
        for kind in ('r4','review'):
            with self.subTest(kind=kind):self.stop(self.old(),'base が進んだ。先に同期する','merge origin/main',kind=kind)
    def test_manual_without_reuse_skips_reuse_check(self):
        self.docs_sync()
        for outcome,values in (('pending',['https://example.invalid/redo']),('pass',['https://example.invalid/new-l3'])):
            with self.subTest(outcome=outcome):
                record=self.rec(self.old(head=OLD),kind='manual',outcome=outcome,values=values)
                self.assertEqual(record['manual'],dict(outcome=outcome,evidence=values))
    def test_sync_only_generated_regen_commit(self):
        # G5: the head is a 1-parent regeneration after the merge; record carries the manual and status passes.
        self.docs_sync();self.repo.commit({GEN_BIND:'b2\n'},'regenerate')
        record=self.rec(self.old(),kind='r4')
        self.assertEqual(record['manual'],self.carried|dict(source_head=self.repo.source))
        low=self.rec(self.old(req=LOW),req=LOW,kind='manual',source=self.repo.source,approval=None)
        self.status(low,LOW)
    def test_code_free_union_over_sync_merges(self):
        # C6: main changes code, the lane syncs, main reverts it; the union over every merge still names the code.
        self.code_sync();self.repo.main({'src/app.ts':'app\n','docs/b.md':'b\n'});self.repo.sync()
        self.stop(self.old(),'src/app.ts')
        self.sync_stop(self.old(req=NO_MANUAL),'src/app.ts')
    def test_sync_stage_rejects_code_sync(self):
        self.code_sync()
        self.sync_stop(self.old(req=NO_MANUAL),'src/app.ts','監査の closure を要る')
    def test_sync_stage_needs_passed_review(self):
        self.docs_sync()
        for outcome in ('pending','fail'):
            old=self.old(req=NO_MANUAL);old['review']['outcome']=outcome
            with self.subTest(outcome=outcome):self.sync_stop(old,'監査の closure を要る')
        self.sync_stop(None,'監査の closure を要る')
    def test_sync_stage_needs_changed_head(self):
        old=self.old(req=NO_MANUAL)
        self.sync_stop(old,'新しい head での最初の record')
        self.docs_sync();first=self.rec(old,req=NO_MANUAL,kind='r4',outcome='not-required',values=[])
        self.assertEqual(first['review']['outcome'],'pending')
        self.sync_stop(first,'新しい head での最初の record')
    def test_sync_stage_requires_same_plan_contract(self):
        self.docs_sync();old=self.old(req=NO_MANUAL);old['review']['broad']['amendments']=[OLD]
        self.sync_stop(old,FRESH_BROAD)
    def test_review_kind_names_no_review_for_r0r1(self):
        # MG-D13 / D-127: R0/R1 has no review record; every stage stops on that reason, before any write.
        stages=dict(sync=SYNC,docs=SYNC|dict(values=['https://example.invalid/p3']),broad={},closure={})
        for sync in ('docs_sync','code_sync'):
            getattr(self,sync)()
            for stage,kw in stages.items():
                with self.subTest(sync=sync,stage=stage):
                    with self.assertRaises(g.GateError) as caught:self.rec(self.old(req=LOW),kind='review',req=LOW,stage=stage,**kw)
                    self.assertEqual(caught.exception.code,2)
                    self.assertEqual(self.writes,[])
                    message=str(caught.exception)
                    self.assertIn('R0/R1',message)
                    for wrong in ('takes only','manual pass not carried'):self.assertNotIn(wrong,message)
    def test_sync_stage_carries_manual_and_closure(self):
        # MG-D13 A + sync closure: one --review-stage sync record carries the manual and records the machine closure.
        self.docs_sync()
        record=self.rec(self.old(),kind='review',stage='sync',**SYNC)
        self.assertEqual(record['manual'],self.carried|dict(source_head=self.repo.source))
        self.assertEqual(record['review']['closure'],dict(head=self.head(),base=self.repo.base,sync_from=self.repo.source))
        self.assertEqual(record['review']['outcome'],'pass')
        self.assertEqual(record['r4'],dict(outcome='pending',evidence=[]))
    def test_sync_stage_chain_keeps_audited_origin(self):
        self.docs_sync();first=self.sync_review(self.old(req=NO_MANUAL))
        self.assertEqual(first['review']['closure'],dict(head=self.head(),base=self.repo.base,sync_from=self.repo.source))
        self.docs_sync('c');second=self.sync_review(first)
        self.assertEqual(second['review']['closure'],dict(head=self.head(),base=self.repo.base,sync_from=self.repo.source))
        self.assertEqual(second['review']['broad'],first['review']['broad'])
        audited=self.head();old=copy.deepcopy(second)
        old['review']['closure']=dict(head=audited,base=self.repo.base,audit=dict(model='opus',run_ref='closure',evidence=['https://example.invalid/c']))
        self.docs_sync('d')
        self.assertEqual(self.sync_review(old)['review']['closure']['sync_from'],audited)
    def test_nonci_rechecks_manual_chain(self):
        self.docs_sync()
        record=self.old(head=self.head(),base=self.repo.base,req=LOW,source_head=self.repo.source)
        self.status(record,LOW)
        lane=self.repo.commit({'docs/lane.md':'after\n'});record['head']=lane
        with self.assertRaisesRegex(g.GateError,lane):self.status(record,LOW)
    def test_nonci_rechecks_sync_closure(self):
        def closure_record():
            record=self.old(head=self.head(),base=self.repo.base,req=NO_MANUAL)
            record['manual']=dict(outcome='not-required',evidence=[])
            record['review']['broad'].update(head=self.repo.source,base=self.repo.base0)
            record['review']['closure']=dict(head=self.head(),base=self.repo.base,sync_from=self.repo.source)
            return record
        self.docs_sync();self.status(closure_record(),NO_MANUAL)
        self.code_sync()
        with self.assertRaisesRegex(g.GateError,'src/app.ts'):self.status(closure_record(),NO_MANUAL)

# The fake gh executes as a process and validates HTTP endpoint/argv wiring, not just a mocked verdict.
FAKE_GH=r'''#!/usr/bin/env python3
import json,os,sys
from pathlib import Path
state_path=Path(os.environ['PR_GATE_FIXTURE'])
s=json.loads(state_path.read_text());a=sys.argv[1:]
s['calls'].append(a)
def save():state_path.write_text(json.dumps(s))
save()
if s.get('offline'):sys.exit(9)
if a[0]=='pr':
    assert a[1] in ('ready','merge') and '--repo' in a and '--admin' not in a
    if a[1]=='merge':
        assert a[a.index('--match-head-commit')+1]==s['pr']['head']['sha']
        if s.get('merge_race'):sys.exit(1)
        s['pr']['merged']=True
    else:s['pr']['draft']=False
    save();sys.exit(0)
assert a[0]=='api' and a[1:3]==['--hostname','github.com']
method=a[a.index('-X')+1];path=a[7]
if s.get('http_error_path') and s['http_error_path'] in path:sys.exit(1)
if method!='GET':
    assert '/issues/' in path and '/comments' in path
    payload=json.load(sys.stdin);assert list(payload)==['body']
    s['comments']=[dict(id=1,user=dict(login=s['owner']),body=payload['body'],updated_at='now')]
    save();print(json.dumps(s['comments'][0]));sys.exit(0)
if path=='user':value={'login':s['owner']}
elif '/pulls/7/files' in path:value=s['file_pages'] if 'file_pages' in s else [s['files']]
elif '/pulls/7/reviews' in path:value=s['review_pages'] if 'review_pages' in s else [s.get('reviews',[])]
elif path.endswith('/pulls/7'):
    s['reads']=s.get('reads',0)+1
    if s.get('head_race_at')==s['reads']:s['pr']['head']['sha']='e'*40
    save();value=s['pr']
elif '/issues/7/comments' in path:value=[s['comments']]
elif '/contents/' in path:
    import base64
    name=path.split('/contents/')[1].split('?')[0]
    # Head listings stay served so a regression back to listing-based selection is caught.
    if name=='docs':value=[dict(type='dir',name='plans',path='docs/plans')]
    elif name=='docs/plans':value=s.get('packets',[])
    else:
        ref=path.split('?ref=')[1]
        content=s.get('snapshots',{}).get(ref,{}).get(name,s['contents'][name])
        value={'encoding':'base64','content':base64.b64encode(content.encode()).decode()}
elif '/rules/branches/main' in path:value=[s['effective']]
elif '/rulesets/1' in path:value=s['policy']
elif '/actions/workflows/ci.yml/runs' in path:value=[{'workflow_runs':s['runs']}]
elif '/check-suites/2/check-runs' in path:value=[{'check_runs':s['checks']}]
else:raise AssertionError(path)
print(json.dumps(value))
'''

HELPER='scripts/pr-gate.py';HELPER_TEXT=(ROOT/HELPER).read_text()

class CLI(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.root=Path(self.tmp.name)
        self.repo=self.root/'repo';self.repo.mkdir();self.bin=self.root/'bin';self.bin.mkdir()
        self.gh=self.bin/'gh';self.gh.write_text(FAKE_GH);self.gh.chmod(0o755)
        def git(*args):return subprocess.check_output(['git','-C',str(self.repo),*args],text=True).strip()
        self.git=git;git('init','-q','-b','feature');git('config','user.name','test');git('config','user.email','test@example.invalid')
        (self.repo/'.gitignore').write_text('.local/\n');git('add','.');git('commit','-qm','fixture');self.head=git('rev-parse','HEAD')
        git('remote','add','origin','https://github.com/'+g.REPO+'.git')
        policy=json.loads((ROOT/'.github/merge-gate-ruleset.json').read_text())
        self.state={'owner':g.OWNER,'calls':[],'pr':dict(number=7,state='open',merged=False,draft=True,
            head=dict(sha=self.head,ref='feature',repo=dict(full_name=g.REPO)),base=dict(sha=self.head,ref='main',repo=dict(full_name=g.REPO)),mergeable=True,mergeable_state='clean',
            body='## 要約\n\nRisk: R0（目視: 不要）\n'),
            'files':[dict(filename='docs/example.md',status='modified')], 'comments':[], 'policy':policy,
            'contents':{'scripts/ci/classify-changes.sh':(ROOT/'scripts/ci/classify-changes.sh').read_text(),g.POLICY:json.dumps(policy),
                        HELPER:HELPER_TEXT},
            'effective':[r|{'ruleset_id':1} for r in policy['rules']],
            'runs':[dict(id=10,head_sha=self.head,created_at='2026-09-14',event='pull_request',path='.github/workflows/ci.yml',head_repository=dict(full_name=g.REPO),head_branch='feature',status='completed',conclusion='success',check_suite_id=2,html_url='https://example.invalid/run')],
            'checks':[dict(name='Merge gate',app=dict(id=15368),status='completed',conclusion='success')]}
        self.file=self.root/'api.json';self.save()
    def tearDown(self):self.tmp.cleanup()
    def save(self):self.file.write_text(json.dumps(self.state))
    def load(self):self.state=json.loads(self.file.read_text());return self.state
    def run_cli(self,action,*extra,expected=0):
        before=self.git('status','--porcelain')
        value=subprocess.run(['python3',str(ROOT/'scripts/pr-gate.py'),action,'--pr','7','--risk','R0','--manual','not-required','--json',*extra],cwd=self.repo,
            env={**os.environ,'PATH':str(self.bin)+os.pathsep+os.environ['PATH'],'PR_GATE_FIXTURE':str(self.file)},capture_output=True,text=True)
        self.assertEqual(value.returncode,expected,value.stdout+value.stderr)
        self.assertEqual(self.git('status','--porcelain'),before)
        return json.loads(value.stdout) if value.returncode==0 else value.stderr
    @staticmethod
    def packet_text(fields):
        return '## Workflow State\n'+''.join(f'- {k}: {v}\n' for k,v in fields.items())+'\n## Risk\nRisk: '+fields['Risk']+'\n'

    def configure_packet(self, **overrides):
        packet='docs/plans/2026-09-14-fixture.md'
        # D-145 (4): the helper reads the packet's form from the commit that added it after the base.
        path=self.repo/packet;path.parent.mkdir(parents=True,exist_ok=True)
        path.write_text(self.packet_text({'Phase':'plan-gate','Risk':'R3','Plan Commit':'pending','Amendments':'none'}))
        self.git('add',packet);self.git('commit','--allow-empty','-qm','plan-first')
        self.plan_ref=self.git('rev-parse','HEAD')
        self.git('commit','--allow-empty','-qm','candidate after plan')
        self.head=self.git('rev-parse','HEAD')
        self.state['pr']['head']['sha']=self.head
        self.state['runs'][0]['head_sha']=self.head
        self.state['files']=self.state['files']+[dict(filename=packet,status='added')]
        fields={'Phase':'implementing','Risk':'R3',
                'Plan Commit':self.plan_ref,'Amendments':'none','Coordinator':'owner','Writer':'codex','Plan Reviewer':'opus',
                'Final Reviewer':'sonnet+opus','Final Review Minimum':'2','Human Gate':'ready,merge'} | overrides
        self.state['contents'][packet]=self.packet_text(fields)
        # Approval snapshot predates its own SHA; phase/self-reference are not gate conditions.
        approved=fields | {'Phase':'plan-gate','Plan Commit':'pending'}
        self.state['snapshots']={self.plan_ref:{packet:self.packet_text(approved)}}
        self.save()
        return packet,fields

    def test_unamended_gate_condition_changes_are_rejected(self):
        cases=[({'Risk':'R4','Human Gate':'ready,merge,manual,r4'},'Risk','R3'),
               ({'Human Gate':'ready,merge,manual'},'Human Gate','ready,merge'),
               ({},'Final Review Minimum','1')]
        for approved,key,value in cases:
            with self.subTest(field=key):
                packet,fields=self.configure_packet(**approved)
                self.state['contents'][packet]=self.packet_text(fields | {key:value})
                self.save()
                result=self.run_cli('status','--packet',packet,expected=1)
                self.assertIn('approved packet condition changed: '+key,result)
                self.run_cli('ready','--packet',packet,expected=1)

    def test_plan_change_after_approval_rejected(self):
        # Final review C1 (D-145 H2/H7): the plan sections of the latest approved snapshot bind HEAD; a Scope edit
        # without a Gated Amendment stops status, capture and Ready, and names the --amend route.
        packet,fields=self.configure_packet()
        scope=lambda text:text+'\n## Scope\n\n- reviewed item\n'
        self.state['snapshots'][self.plan_ref][packet]=scope(self.state['snapshots'][self.plan_ref][packet])
        self.state['contents'][packet]=scope(self.packet_text(fields));self.save()
        self.assertNotIn('approved plan changed',' '.join(self.run_cli('status','--packet',packet)['blockers']))
        self.state['contents'][packet]=self.packet_text(fields)+'\n## Scope\n\n- reviewed item\n- unreviewed item\n';self.save()
        for action in ('status','capture','ready'):
            with self.subTest(action=action):
                error=self.run_cli(action,'--packet',packet,expected=1)
                for text in ('approved plan changed','Scope','plan-approve.sh '+packet+' --amend'):self.assertIn(text,error)
    def test_old_pre_gate_phase_is_not_approved(self):
        # Final review C1: an old packet still at a pre-gate Phase is not approved even with a SHA (the old rule).
        packet,fields=self.configure_packet(Phase='plan-gate')
        status=self.run_cli('status','--packet',packet)
        self.assertEqual(status['blockers'],['Plan Gate incomplete'])
        self.assertIn('Plan Gate incomplete',self.run_cli('capture','--packet',packet,expected=1))
    def test_execution_mode_change_is_not_a_gate_condition(self):
        # T-H6: approved snapshot carries Execution Mode; current changes or drops it without a condition error.
        for current in ('dual-vendor-no-fable',None):
            with self.subTest(current=current):
                packet,fields=self.configure_packet(**{'Execution Mode':'fable-window'})
                if current is None:
                    fields=dict(fields);del fields['Execution Mode']
                else:
                    fields=fields|{'Execution Mode':current}
                self.state['contents'][packet]=self.packet_text(fields);self.save()
                status=self.run_cli('status','--packet',packet)
                self.assertEqual(status['state'],'Draft')
                self.assertFalse([b for b in status['blockers'] if 'approved packet condition changed' in b],status['blockers'])

    def test_registered_amendment_order_cannot_roll_back_manual(self):
        packet='docs/plans/2026-09-14-fixture.md'
        path=self.repo/packet;path.parent.mkdir(parents=True)
        fields={'Phase':'plan-gate','Risk':'R3',
                'Plan Commit':'pending','Amendments':'none','Coordinator':'owner','Writer':'codex','Plan Reviewer':'opus',
                'Final Reviewer':'sonnet','Final Review Minimum':'1','Human Gate':'ready,merge,manual'}
        def commit(subject):
            path.write_text(self.packet_text(fields));self.git('add',packet);self.git('commit','-qm',subject)
            return self.git('rev-parse','HEAD')
        plan=commit('plan-first')
        fields.update({'Phase':'implementing','Plan Commit':plan})
        commit('approve plan')
        fields['Human Gate']='ready,merge'
        a1=commit('A1 approved without manual')
        fields['Amendments']=a1;commit('register A1')
        fields['Human Gate']='ready,merge,manual'
        a2=commit('A2 requires manual')
        fields['Amendments']=a1+', '+a2
        commit('register A2')
        self.state['files']=[dict(filename=packet,status='added')]
        self.state['snapshots']={ref:{packet:self.git('show',ref+':'+packet)} for ref in (plan,a1,a2)}
        def observe():
            self.state['pr']['head']['sha']=self.git('rev-parse','HEAD')
            self.state['contents'][packet]=path.read_text();self.save()
            capture=self.run_cli('capture','--packet',packet)['capture']
            # D-145 (5): the checker measures active packets from the fixture's base commit.
            check=subprocess.run(['bash',str(ROOT/'scripts/check-workflow-git.sh')],cwd=self.repo,capture_output=True,text=True,
                                 env={**os.environ,'WORKFLOW_BASE_SHA':self.state['pr']['base']['sha']})
            return json.loads(Path(capture).read_text())['requirements'],check
        required,check=observe()
        self.assertTrue(required['manual'])
        self.assertEqual(check.returncode,0,check.stdout+check.stderr)
        fields.update({'Amendments':a2+', '+a1,'Human Gate':'ready,merge'})
        commit('invalid reorder restores older A1 conditions')
        downgraded,check=observe()
        # The helper selects the last entry; shared PK5 must make this candidate unmergeable.
        self.assertFalse(downgraded['manual'])
        self.assertNotEqual(check.returncode,0,'PK5 accepted A2,A1 and helper captured manual=false: '+check.stdout)
        self.assertIn('Amendments が削除・変更されています',check.stdout)

    def test_latest_amendment_snapshot_owns_gate_conditions(self):
        packet,fields=self.configure_packet(**{'Human Gate':'ready,merge,manual'})
        amended=fields | {'Final Review Minimum':'1','Human Gate':'ready,merge'}
        self.state['snapshots'][P]={packet:self.packet_text(amended | {'Phase':'plan-gate','Plan Commit':'pending'})}
        self.state['contents'][packet]=self.packet_text(amended | {'Amendments':P})
        self.save()
        self.assertEqual(self.run_cli('status','--packet',packet)['state'],'Draft')
        # Merely pointing to the old Plan Commit cannot authorize the amendment's conditions.
        self.state['contents'][packet]=self.packet_text(amended)
        self.save()
        self.assertIn('approved packet condition changed',self.run_cli('status','--packet',packet,expected=1))

    def test_status_capture_ready_merge(self):
        self.assertEqual(self.run_cli('status')['blockers'],[])
        self.assertTrue(all(c[0]=='api' and 'GET' in c for c in self.load()['calls']))
        capture=self.run_cli('capture')['capture'];self.assertTrue(Path(capture).is_file())
        self.run_cli('ready');self.run_cli('merge')
        self.assertTrue(self.load()['pr']['merged'])
    def test_packet_double_audit_cli(self):
        packet,fields=self.configure_packet()
        self.state['files']=[dict(filename='scripts/pr-gate.py',status='modified'),dict(filename=packet,status='added')];self.save()
        common=('--packet',packet)
        capture=self.run_cli('capture',*common)['capture']
        self.run_cli('record',*common,'--capture',capture,'--kind','review','--review-stage','broad','--pass-model','sonnet','--run-ref','sonnet-audit','--evidence','https://example.invalid/sonnet','--outcome','pending','--reviewed-head',self.head,'--pr-reviews','0')
        self.assertIn('review not passed',self.run_cli('ready',*common,expected=1))
        capture=self.run_cli('capture',*common)['capture']
        self.run_cli('record',*common,'--capture',capture,'--kind','review','--review-stage','broad','--pass-model','opus','--run-ref','opus-audit','--evidence','https://example.invalid/opus','--outcome','pass','--reviewed-head',self.head,'--pr-reviews','0')
        self.run_cli('ready',*common)
        self.run_cli('status',*common,*common,expected=2)
        self.load()
        self.state['contents'][packet]=self.state['contents'][packet].replace('Phase: implementing','Phase: plan-gate').replace('Plan Commit: '+fields['Plan Commit'],'Plan Commit: pending')
        self.state['comments']=[]
        self.state['pr']['draft']=True
        self.save()
        waiting=self.run_cli('status',*common)
        self.assertIn('Plan Gate incomplete',waiting['blockers'])
        self.assertIn('Plan Gate incomplete',self.run_cli('ready',*common,expected=1))
        self.assertIn('Plan Gate incomplete',self.run_cli('capture',*common,expected=1))
        self.assertIn('Plan Gate incomplete',self.run_cli('merge',*common,expected=1))


    def test_workflow_minimum_one_rejected(self):
        packet,_=self.configure_packet(**{'Final Review Minimum':'1'})
        self.state['files']=[dict(filename='scripts/pr-gate.py',status='modified'),dict(filename=packet,status='added')];self.save()
        for action in ('status','ready'):
            self.assertIn('required Double Audit minimum is 2',self.run_cli(action,'--packet',packet,expected=1))

    def test_r4_minimum_one_rejected(self):
        packet,_=self.configure_packet(**{'Risk':'R4','Human Gate':'ready,merge,r4','Final Review Minimum':'1'})
        for action in ('status','ready'):
            self.assertIn('R4 gates missing',self.run_cli(action,'--packet',packet,expected=1))

    def test_r4_approval_gate_cannot_be_omitted(self):
        packet,_=self.configure_packet(**{'Risk':'R4'})
        for action in ('status','ready'):
            self.assertIn('R4 gates missing',self.run_cli(action,'--packet',packet,expected=1))

    def test_codex_only_r3_ui_minimum_one_accepted(self):
        # T-H7a: Execution Mode no longer changes the review count; only R4 / workflow require 2.
        packet,_=self.configure_packet(**{'Execution Mode':'codex-only','Final Review Minimum':'1'})
        self.state['files']=[dict(filename='src/features/example/view.tsx',status='modified'),dict(filename=packet,status='added')];self.save()
        status=self.run_cli('status','--packet',packet)
        self.assertFalse([b for b in status['blockers'] if 'minimum' in b.lower()],status['blockers'])

    def assert_rules_blocked(self):
        self.save()
        self.assertTrue(self.run_cli('status')['blockers'])
        self.run_cli('ready',expected=1)
        self.load();self.state['pr']['draft']=False;self.save()
        self.run_cli('merge',expected=1)
        self.assertFalse(any(call[0]=='pr' for call in self.load()['calls']))

    def add_rule_defaults(self, additions):
        # API detail is separate from both the main contents and effective rules.
        self.state['policy']=copy.deepcopy(self.state['policy'])
        self.state['policy']['rules'][2]['parameters'].update(copy.deepcopy(additions))

    def test_rules_accept_observed_defaults(self):
        original=copy.deepcopy(self.state)
        # MG-D1a: observed response literals, not copied from the desired policy.
        for additions in ({}, {'required_reviewers': []},
                          {'require_extra_approval_for_unattributed_changes': True},
                          {'required_reviewers': [], 'require_extra_approval_for_unattributed_changes': True}):
            with self.subTest(additions=additions):
                self.state=copy.deepcopy(original);self.add_rule_defaults(additions);self.save()
                self.assertEqual(self.run_cli('status')['blockers'],[])
                self.run_cli('ready');self.run_cli('merge')
                self.assertTrue(self.load()['pr']['merged'])

    def test_rules_reject_default_value_or_type_drift(self):
        original=copy.deepcopy(self.state)
        for key,values in (
            ('required_reviewers', [[{}], None, {}, '[]', False, 0]),
            ('require_extra_approval_for_unattributed_changes', [False, None, 'true', 0, 1, [], {}]),
        ):
            for value in values:
                with self.subTest(key=key,value=value):
                    self.state=copy.deepcopy(original)
                    self.add_rule_defaults({'required_reviewers': [], 'require_extra_approval_for_unattributed_changes': True})
                    self.state['policy']['rules'][2]['parameters'][key]=value
                    self.assert_rules_blocked()

    def test_rules_defaults_preserve_drift_rejection(self):
        self.add_rule_defaults({'required_reviewers': [], 'require_extra_approval_for_unattributed_changes': True})
        original=copy.deepcopy(self.state);rules=original['policy']['rules']
        changes=[
            (('rules',2,'parameters','unknown_default'), []),
            (('rules',3,'parameters','required_reviewers'), []),
            (('rules',2,'parameters','allowed_merge_methods'), list(reversed(rules[2]['parameters']['allowed_merge_methods']))),
            (('rules',2,'parameters','required_review_thread_resolution'), True),
            (('rules',3,'parameters','strict_required_status_checks_policy'), False),
            (('rules',3,'parameters','required_status_checks'), [{'context':'Other','integration_id':15368}]),
            (('rules',3,'parameters','required_status_checks'), [{'context':'Merge gate','integration_id':1}]),
            (('rules',), rules[1:]),
            (('rules',), rules[:1]+rules[2:]),
            (('rules',), rules[:2]+rules[3:]),
            (('rules',), list(reversed(rules))),
            (('name',), 'other-policy'),
            (('target',), 'tag'),
            (('enforcement',), 'evaluate'),
            (('conditions','ref_name','include'), ['refs/heads/other']),
            (('conditions','ref_name','exclude'), ['refs/heads/main']),
            (('bypass_actors',), [{'actor_id':1,'actor_type':'RepositoryRole','bypass_mode':'always'}]),
        ]
        for path,value in changes:
            with self.subTest(path=path,value=value):
                self.state=copy.deepcopy(original);target=self.state['policy']
                for key in path[:-1]:target=target[key]
                target[path[-1]]=copy.deepcopy(value)
                self.assert_rules_blocked()

    def test_rules_keep_explicit_desired_defaults(self):
        original=copy.deepcopy(self.state)
        for key,expected,other in (
            ('required_reviewers', [], [{}]),
            ('require_extra_approval_for_unattributed_changes', False, True),
        ):
            for matches,value in ((True,expected),(False,other)):
                with self.subTest(key=key,matches=matches):
                    self.state=copy.deepcopy(original)
                    desired=copy.deepcopy(original['policy']);desired['rules'][2]['parameters'][key]=expected
                    self.state['pr']['base']['sha']=B
                    self.state['snapshots']={B:{g.POLICY:json.dumps(desired)}}
                    self.state['contents'][g.POLICY]='{}'  # Only the main snapshot is authoritative.
                    self.state['policy']=copy.deepcopy(desired)
                    self.state['policy']['rules'][2]['parameters'][key]=value
                    if matches:
                        self.save();self.assertEqual(self.run_cli('status')['blockers'],[])
                        self.run_cli('ready');self.run_cli('merge')
                    else:self.assert_rules_blocked()

    def test_rules_leave_inputs_unchanged(self):
        desired=json.loads(self.state['contents'][g.POLICY])
        self.add_rule_defaults({'required_reviewers': [], 'require_extra_approval_for_unattributed_changes': True})
        detail=self.state['policy'];effective=self.state['effective']
        before=copy.deepcopy((desired,detail,effective))
        gate=object.__new__(g.Gate);gate.endpoint='repos/'+g.REPO
        with patch.object(gate,'contents',return_value='main policy'), \
             patch.object(g,'decode_json',return_value=desired), \
             patch.object(g,'api',side_effect=[[effective],detail]):
            gate.rules(self.state['pr'])
        self.assertEqual((desired,detail,effective),before)

    def test_rules_reject_strict_false(self):
        for rule in self.state['effective']:
            if rule['type']=='required_status_checks':
                rule['parameters']['strict_required_status_checks_policy']=False
        # Desired/detail remain strong: each fresh response must independently be safe.
        self.state['policy']=json.loads(self.state['contents'][g.POLICY])
        self.assert_rules_blocked()

    def test_rules_reject_bypass(self):
        self.state['policy']['bypass_actors']=[{'actor_id':1,'actor_type':'RepositoryRole','bypass_mode':'always'}]
        self.assert_rules_blocked()

    def test_rules_reject_inactive_enforcement(self):
        self.state['policy']['enforcement']='evaluate'
        self.assert_rules_blocked()

    def test_rules_reject_parameter_drift(self):
        self.state['policy']['rules'][2]['parameters']['required_review_thread_resolution']=True
        self.assert_rules_blocked()

    def test_bad_rules_checks_latest_run(self):
        for group,change in [('effective',[]),('checks',[]),('checks',[dict(name='Merge gate',app=dict(id=1),status='completed',conclusion='success')]),('checks',[dict(name='Merge gate',app=dict(id=15368),status='completed',conclusion='skipped')])]:
            original=copy.deepcopy(self.state[group]);self.state[group]=change;self.save()
            self.assertTrue(self.run_cli('status')['blockers']);self.state[group]=original
        self.state['runs'].append(self.state['runs'][0]|dict(id=11,status='in_progress',conclusion=None));self.save()
        self.assertTrue(self.run_cli('status')['blockers'])
    def test_offline_bad_repo_input_and_race(self):
        self.state['offline']=True;self.save();self.run_cli('status',expected=2)
        self.state.pop('offline');self.save();self.run_cli('status','--repo','wrong/repo',expected=2)
        self.run_cli('status','--pr','-1',expected=2)
        self.state['head_race_at']=2;self.save();self.run_cli('capture',expected=1)
    def test_merge_match_head_race(self):
        self.state['pr']['draft']=False;self.state['merge_race']=True;self.save()
        self.run_cli('merge',expected=2);self.assertFalse(self.load()['pr']['merged'])
    def test_shell_branch_is_argv_data(self):
        marker=self.root/'owned'
        branch=f'feature;touch {marker}'
        self.state['pr']['head']['ref']=branch;self.state['runs'][0]['head_branch']=branch;self.save()
        self.run_cli('status');self.assertFalse(marker.exists())
    def test_required_manual_record_and_comment_only_write(self):
        self.state['pr']['body']='Risk: R0（目視: 要）';self.save()
        capture=self.run_cli('capture','--manual','required')['capture']
        self.run_cli('ready','--manual','required',expected=1)
        self.run_cli('record','--manual','required','--capture',capture,'--kind','manual','--outcome','pass','--evidence','https://example.invalid/manual')
        self.run_cli('ready','--manual','required')
        self.assertEqual(len(self.load()['comments']),1)
        self.run_cli('record','--manual','required','--capture',capture,'--kind','manual','--outcome','pass','--evidence','x',expected=1)
    def test_low_risk_execution_change_rejected(self):
        self.state['files']=[dict(filename='scripts/pr-gate.py',status='modified')];self.save();self.run_cli('status',expected=1)

class HelperVersion(unittest.TestCase):
    # SPEC-WF-HARNESS5-D2: the helper runs only when its bytes equal the PR base's scripts/pr-gate.py.
    setUp=CLI.setUp;tearDown=CLI.tearDown;save=CLI.save;load=CLI.load;run_cli=CLI.run_cli
    CHANGED=HELPER_TEXT+'\n'  # One trailing byte: normalizing (rstrip, newline folding) must still differ.
    def captures(self):return sorted((self.repo/'.local/pr-gate').glob('*.json'))
    def test_mismatch_blocks_every_action(self):
        self.state['pr']['body']='Risk: R0（目視: 要）';self.save()
        capture=self.run_cli('capture','--manual','required')['capture']
        self.load();self.state['contents'][HELPER]=self.CHANGED;self.state['calls']=[];self.save()
        before=self.captures()
        for action,extra in (('status',()),('capture',()),('ready',()),('merge',()),
                             ('record',('--capture',capture,'--kind','manual','--outcome','pass','--evidence','https://example.invalid/manual'))):
            with self.subTest(action=action):
                if action=='merge':self.load();self.state['pr']['draft']=False;self.save()
                error=self.run_cli(action,'--manual','required',*extra,expected=1)
                self.assertIn('helper differs from base '+self.head,error)
                self.assertIn('git show '+self.head+':scripts/pr-gate.py',error)
        calls=self.load()['calls']
        self.assertFalse([c for c in calls if c[0]=='pr' or 'POST' in c or 'PATCH' in c],calls)
        self.assertEqual(self.state['comments'],[])
        self.assertEqual(self.captures(),before)
    def test_mismatch_command_is_complete(self):
        # T5-1 / D-107 (5): the recovery command restores the original argv, quoted for the shell.
        self.state['pr']['body']='Risk: R0（目視: 要）';self.save()
        capture=self.run_cli('capture','--manual','required')['capture']
        self.load();self.state['contents'][HELPER]=self.CHANGED;self.save()
        extra=('record','--manual','required','--capture',capture,'--kind','manual','--outcome','pass','--evidence',"a b; touch x 'q'")
        error=self.run_cli(*extra,expected=1)
        self.assertNotIn('…',error)
        tail=error.split('python3 "${TMPDIR:-/tmp}/pr-gate-base.py" ',1)[1]
        self.assertEqual(shlex.split(tail),[extra[0],'--pr','7','--risk','R0','--manual','not-required','--json',*extra[1:]])
    def test_same_bytes_passes(self):
        self.assertEqual(self.run_cli('status')['blockers'],[])
    def test_base_fetch_failure_exit_2(self):
        self.state['http_error_path']='/contents/'+HELPER;self.save()
        self.run_cli('status',expected=2)
    def test_compares_base_not_head(self):
        self.state['pr']['base']['sha']=B
        for base,head,expected in ((HELPER_TEXT,self.CHANGED,0),(self.CHANGED,HELPER_TEXT,1)):
            with self.subTest(base_changed=expected==1):
                self.state['snapshots']={B:{HELPER:base}};self.state['contents'][HELPER]=head;self.save()
                result=self.run_cli('status',expected=expected)
                if expected:self.assertIn('helper differs from base '+B,result)
                else:self.assertEqual(result['blockers'],[])

class ReviewedHead(unittest.TestCase):
    # SPEC-WF-HARNESS5-D8: a review record binds to the head the reviewer audited.
    tearDown=CLI.tearDown;save=CLI.save;load=CLI.load;run_cli=CLI.run_cli
    packet_text=staticmethod(CLI.packet_text);configure_packet=CLI.configure_packet
    def review(self,stage,run,reviewed,expected=0,outcome='pass',pr_reviews=0):
        common=('--packet',self.packet)
        capture=self.run_cli('capture',*common)['capture']
        extra=('--reviewed-head',reviewed) if reviewed is not None else ()
        extra+=('--pr-reviews',str(pr_reviews)) if pr_reviews is not None else ()
        return self.run_cli('record',*common,'--capture',capture,'--kind','review','--review-stage',stage,'--pass-model',run,
                            '--run-ref',run,'--evidence','https://example.invalid/'+run,'--outcome',outcome,*extra,expected=expected)
    def setUp(self):
        CLI.setUp(self)
        self.packet,_=self.configure_packet()
        self.state['files']=[dict(filename='scripts/pr-gate.py',status='modified'),dict(filename=self.packet,status='added')];self.save()
    def broad_then_push(self):
        # Broad passes at H1, then a fix is pushed: H2 needs a closure.
        self.review('broad','sonnet',self.head,outcome='pending');self.review('broad','opus',self.head)
        self.audited=self.head
        self.git('commit','--allow-empty','-qm','fix');self.head=self.git('rev-parse','HEAD')
        self.load();self.state['pr']['head']['sha']=self.head;self.state['runs'][0]['head_sha']=self.head;self.save()
        return copy.deepcopy(self.load()['comments'])
    def amend(self):
        # A Gated Amendment registered after the broad changes the Plan contract.
        self.load();self.state['contents'][self.packet]=self.state['contents'][self.packet].replace('Amendments: none','Amendments: '+self.audited);self.save()
    def push(self):
        self.git('commit','--allow-empty','-qm','next');self.head=self.git('rev-parse','HEAD')
        self.load();self.state['pr']['head']['sha']=self.head;self.state['runs'][0]['head_sha']=self.head;self.save()
    def blockers(self):return self.run_cli('status','--packet',self.packet)['blockers']
    def test_status_after_amendment_names_fresh_broad(self):
        # T1-2 / D-107 (3)
        self.broad_then_push();self.amend()
        blockers=self.blockers()
        self.assertIn(FRESH_BROAD,blockers);self.assertNotIn('stale head/base in workflow record',blockers)
    def test_closure_after_amendment_says_fresh_broad(self):
        # T1-3 / D-107 (3)
        before=self.broad_then_push();self.amend()
        self.assertIn(FRESH_BROAD,self.review('closure','closure',self.head,expected=1))
        self.assertEqual(self.load()['comments'],before)
    def test_status_after_amendment_and_manual_names_broad_required(self):
        # T1-5 / D-107 (3): record() drops the old-contract broad (broad=None); status names the broad to record.
        manual=lambda text:text.replace('- Human Gate: ready,merge\n','- Human Gate: ready,merge,manual\n')
        self.state['contents'][self.packet]=manual(self.state['contents'][self.packet])
        self.state['snapshots'][self.plan_ref][self.packet]=manual(self.state['snapshots'][self.plan_ref][self.packet]);self.save()
        self.broad_then_push();self.amend()
        capture=self.run_cli('capture','--packet',self.packet)['capture']
        self.run_cli('record','--packet',self.packet,'--capture',capture,'--kind','manual','--outcome','pass','--evidence','https://example.invalid/manual')
        blockers=self.blockers()
        self.assertEqual(blockers[0],BROAD_REQUIRED)
        self.assertFalse([b for b in blockers if b in ('review not passed',FRESH_BROAD)],blockers)
        # H3 with the record left at H2: the missing broad is still named before stale head/base.
        self.push()
        self.assertEqual(self.blockers()[0],BROAD_REQUIRED)
    # D-107 (4): --pr-reviews must equal the PR reviews with a body on the reviewed head.
    def item(self,id,body='summary',commit=None,**extra):
        return dict(id=id,commit_id=commit or self.head,body=body,state='COMMENTED',submitted_at=f'2026-10-05T01:{id%60:02d}:00Z')|extra
    def put_reviews(self,*reviews):self.load();self.state['reviews']=list(reviews);self.save()
    def writes(self):return [c for c in self.load()['calls'] if 'POST' in c or 'PATCH' in c]
    def mismatch(self,declared,listed):
        return (f'--pr-reviews {declared} but the reviewed head has {len(listed)} PR reviews with a body; read each before recording: '
                +', '.join(f'id={i} submitted_at={t}' for i,t in listed))
    def test_pr_reviews_undercount_rejected(self):
        # T2-1: a pending first audit reaches the write, so a count check moved after it would add a comment.
        self.put_reviews(self.item(101),self.item(102))
        error=self.review('broad','sonnet',self.head,expected=1,outcome='pending',pr_reviews=1)
        self.assertIn(self.mismatch(1,[(101,'2026-10-05T01:41:00Z'),(102,'2026-10-05T01:42:00Z')]),error)
        self.assertEqual(self.load()['comments'],[]);self.assertEqual(self.writes(),[])
    def test_pr_reviews_match_records(self):
        # T2-2
        self.put_reviews(self.item(101),self.item(102))
        self.review('broad','sonnet',self.head,pr_reviews=2,outcome='pending')
        self.assertEqual(len(self.load()['comments']),1)
    def test_pr_reviews_count_only_body_on_reviewed_head(self):
        # T2-3: empty, blank and null bodies and another head's review are not counted.
        self.put_reviews(self.item(101),self.item(102,body=''),self.item(103,body='  \n'),self.item(104,body=None),self.item(105,commit=OLD))
        self.assertIn(self.mismatch(2,[(101,'2026-10-05T01:41:00Z')]),self.review('broad','sonnet',self.head,expected=1,pr_reviews=2))
        self.assertEqual(self.load()['comments'],[])
        self.review('broad','sonnet',self.head,pr_reviews=1,outcome='pending')
        self.assertEqual(len(self.load()['comments']),1)
    def test_pr_reviews_overcount_and_closure_rejected(self):
        # T2-4
        self.assertIn(self.mismatch(1,[]),self.review('broad','sonnet',self.head,expected=1,pr_reviews=1))
        self.assertEqual(self.load()['comments'],[])
        before=self.broad_then_push()
        self.put_reviews(self.item(201))
        self.assertIn(self.mismatch(0,[(201,'2026-10-05T01:21:00Z')]),self.review('closure','closure',self.head,expected=1,pr_reviews=0))
        self.assertEqual(self.load()['comments'],before)
        self.review('closure','closure',self.head,pr_reviews=1)
        self.assertNotEqual(self.load()['comments'],before)
    def test_pr_reviews_missing_negative_or_unavailable(self):
        # T2-5
        for value in (None,-1):
            with self.subTest(value=value):
                error=self.review('broad','sonnet',self.head,expected=2,pr_reviews=value)
                self.assertIn('review needs --pr-reviews (count of PR reviews with a body on the reviewed head)',error)
        self.load();self.state['http_error_path']='/reviews';self.save()
        self.review('broad','sonnet',self.head,expected=2,pr_reviews=0)
        self.assertEqual(self.load()['comments'],[]);self.assertEqual(self.writes(),[])
    def test_reviewed_head_checked_before_pr_reviews(self):
        # T2-6 / D-099 D8: the reviewed head is checked before the reviews are read.
        self.put_reviews(self.item(101))
        self.assertIn('reviewed head differs from capture head',self.review('broad','sonnet',OLD,expected=1,pr_reviews=5))
        self.assertFalse([c for c in self.load()['calls'] if any('/reviews' in a for a in c)])
    def test_pr_reviews_pending_review_listed(self):
        # T2-7: a PENDING review has no submitted_at (or null) and is still counted and listed.
        pending=self.item(302,state='PENDING');del pending['submitted_at']
        self.put_reviews(self.item(301),pending,self.item(303,state='PENDING',submitted_at=None))
        error=self.review('broad','sonnet',self.head,expected=1,pr_reviews=1)
        self.assertIn(self.mismatch(1,[(301,'2026-10-05T01:01:00Z'),(302,'not-submitted'),(303,'not-submitted')]),error)
        self.assertEqual(self.load()['comments'],[]);self.assertEqual(self.writes(),[])
    def test_pr_reviews_counted_across_pages(self):
        # T2-8: every page is counted, not only the first.
        self.load();self.state['review_pages']=[[self.item(401)]+[self.item(500+i,body='') for i in range(99)],[self.item(402)]];self.save()
        self.review('broad','sonnet',self.head,expected=1,pr_reviews=1)
        self.assertEqual(self.load()['comments'],[])
        self.review('broad','sonnet',self.head,pr_reviews=2,outcome='pending')
        self.assertEqual(len(self.load()['comments']),1)
    def test_broad_mismatch_rejected(self):
        self.assertIn('reviewed head differs from capture head',self.review('broad','sonnet',OLD,expected=1))
        self.assertEqual(self.load()['comments'],[])
    def test_closure_mismatch_rejected(self):
        before=self.broad_then_push()
        self.assertIn('reviewed head differs from capture head',self.review('closure','closure',self.audited,expected=1))
        self.assertEqual(self.load()['comments'],before)
    def test_missing_reviewed_head_exit_2(self):
        for value in (None,'abc'):
            with self.subTest(value=value):self.review('broad','sonnet',value,expected=2)
        self.assertEqual(self.load()['comments'],[])
    # MG-D13: after the broad, main gains a docs commit and the lane merges it like a sync.
    def broad_then_sync(self):
        self.review('broad','sonnet',self.head,outcome='pending');self.review('broad','opus',self.head)
        self.audited=self.head;base=self.load()['pr']['base']['sha']
        self.git('switch','-qc','upstream',base);(self.repo/'docs').mkdir(exist_ok=True);(self.repo/'docs/b.md').write_text('b\n')
        self.git('add','docs/b.md');self.git('commit','-qm','main docs');base=self.git('rev-parse','HEAD')
        self.git('switch','-q','feature');self.git('-c','merge.directoryRenames=false','merge','--no-ff','-q','-m','sync','upstream')
        self.head=self.git('rev-parse','HEAD')
        self.load();self.state['pr']['head']['sha']=self.head;self.state['pr']['base']['sha']=base
        self.state['runs'][0]['head_sha']=self.head;self.save()
        return copy.deepcopy(self.state['comments'])
    def sync_record(self,*extra,expected=0):
        capture=self.run_cli('capture','--packet',self.packet)['capture']
        return self.run_cli('record','--packet',self.packet,'--capture',capture,'--kind','review','--review-stage','sync',*extra,expected=expected)
    def test_sync_stage_records_machine_closure(self):
        # S1: argv reaches the helper and the PATCH body carries the sync closure; Ready then passes.
        self.broad_then_sync();before=len(self.writes())
        self.sync_record('--outcome','pass')
        self.assertEqual([c for c in self.writes()[before:] if 'PATCH' in c],self.writes()[before:])
        self.assertEqual(len(self.writes())-before,1)
        record=json.loads(self.state['comments'][0]['body'].split('```json\n')[1].split('\n```')[0])
        self.assertEqual(record['review']['closure'],dict(head=self.head,base=self.state['pr']['base']['sha'],sync_from=self.audited))
        self.assertEqual(self.blockers(),[])
        self.run_cli('ready','--packet',self.packet)
    def test_sync_stage_rejects_audit_arguments(self):
        # S5: audit arguments and a non-pass outcome are input errors, written nowhere.
        before=self.broad_then_sync()
        for extra in (('--pass-model','opus'),('--run-ref','r'),('--evidence','https://example.invalid/x'),
                      ('--reviewed-head',self.head),('--pr-reviews','0')):
            with self.subTest(extra=extra[0]):self.sync_record('--outcome','pass',*extra,expected=2)
        self.sync_record('--outcome','pending',expected=2)
        self.assertEqual(self.load()['comments'],before)
    def test_sync_stage_stops_on_posted_review(self):
        # S8 / D-107: a posted review with a body on the current head (counted on every page) stops the sync closure.
        before=self.broad_then_sync();writes=self.writes()
        self.load();self.state['review_pages']=[[self.item(601,body='')]*99+[self.item(602,body='')],[self.item(603)]];self.save()
        error=self.sync_record('--outcome','pass',expected=1)
        for text in ('id=603','submitted_at=2026-10-05T01:03:00Z','--review-stage closure','--pr-reviews 1'):self.assertIn(text,error)
        self.assertEqual(self.load()['comments'],before);self.assertEqual(self.writes(),writes)
    def test_matching_head_records(self):
        self.broad_then_push()
        self.review('closure','closure',self.head)
        record=json.loads(self.load()['comments'][0]['body'].split('```json\n')[1].split('\n```')[0])
        self.assertEqual((record['review']['broad']['head'],record['review']['closure']['head']),(self.audited,self.head))
        self.run_cli('ready','--packet',self.packet)

MINE='docs/plans/2026-09-14-fixture.md'; OTHER='docs/plans/2026-09-13-other.md'
TOUCH='PR diff must touch exactly the --packet active packet'
LEAVE='packet leaving docs/plans must move to docs/archive/plans in this PR'

class PacketScope(unittest.TestCase):
    # SPEC-WF-PARALLEL-FRICTION D1..D3: the PR binds to the active packets its own diff touches.
    # Borrows CLI's fixture by attribute: subclassing CLI would run every CLI test a second time.
    setUp=CLI.setUp;tearDown=CLI.tearDown;save=CLI.save;load=CLI.load;run_cli=CLI.run_cli
    packet_text=staticmethod(CLI.packet_text);configure_packet=CLI.configure_packet
    def phase(self,path,phase):self.state['contents'][path]=self.packet_text({'Phase':phase,'Risk':'R3'})
    def diff(self,*entries):
        self.state['files']=[dict(filename='docs/example.md',status='modified')]+[dict(zip(('status','filename','previous_filename'),e)) for e in entries]
        self.save()
    def archive(self,path):return 'docs/archive/plans/'+Path(path).name
    def fetched(self,path):return any('/contents/'+path+'?' in ' '.join(call) for call in self.load()['calls'])

    def test_other_lane_packet_in_head_r2_passes(self):
        self.configure_packet();self.phase(OTHER,'implementing')
        self.state['packets']=[dict(type='file',name=Path(p).name,path=p) for p in (OTHER,MINE)];self.save()
        status=self.run_cli('status','--packet',MINE)
        self.assertFalse([b for b in status['blockers'] if 'packet' in b],status['blockers'])
    def test_two_active_packets_in_diff_rejected(self):
        self.configure_packet();self.phase(OTHER,'implementing');self.diff(('added',MINE),('added',OTHER))
        self.assertIn(TOUCH,self.run_cli('status','--packet',MINE,expected=1))
    def test_diff_across_pages_rejected(self):
        self.configure_packet();self.phase(OTHER,'implementing')
        self.state['file_pages']=[[dict(filename='docs/example.md',status='modified'),dict(filename=MINE,status='added')],[dict(filename=OTHER,status='added')]]
        self.save()
        self.assertIn(TOUCH,self.run_cli('status','--packet',MINE,expected=1))
    def test_packet_argument_mismatch_rejected(self):
        self.configure_packet();self.phase(OTHER,'implementing');self.diff(('added',OTHER))
        self.assertIn(TOUCH,self.run_cli('status','--packet',MINE,expected=1))
    def test_r2_archiving_other_lane_packet_rejected(self):
        self.configure_packet();self.phase(self.archive(OTHER),'archive')
        self.diff(('added',MINE),('renamed',self.archive(OTHER),OTHER))
        self.assertIn(TOUCH,self.run_cli('status','--packet',MINE,expected=1))
    def test_rename_inside_plans_counts_both_paths(self):
        self.configure_packet();self.diff(('renamed',MINE,OTHER))
        self.assertIn(TOUCH,self.run_cli('status','--packet',MINE,expected=1))
    def test_r0_closeout_archive_move_passes(self):
        x,y,z='docs/plans/2026-09-10-x.md','docs/plans/2026-09-11-y.md','docs/plans/2026-09-12-z.md'
        for path in (x,y):self.phase(self.archive(path),'archive')
        self.phase(z,'implementing')
        self.state['packets']=[dict(type='file',name=Path(z).name,path=z)]
        self.diff(('renamed',self.archive(x),x),('renamed','docs/archive/plans/test-matrices/2026-09-10-x.md','docs/plans/test-matrices/2026-09-10-x.md'),
                  ('removed',y),('added',self.archive(y)),('modified','docs/Plans.md'))
        self.assertEqual(self.run_cli('status')['blockers'],[])
        self.run_cli('ready');self.run_cli('merge')
        self.assertTrue(self.load()['pr']['merged'])
        # AR1: the move alone is checked; the archive copy's Phase line is no longer read.
        self.assertFalse(self.fetched(self.archive(x)) or self.fetched(self.archive(y)))
    def test_r0_closeout_reported_as_remove_and_add_passes(self):
        self.phase(self.archive(OTHER),'archive');self.diff(('removed',OTHER),('added',self.archive(OTHER)))
        self.assertEqual(self.run_cli('status')['blockers'],[])
    def test_r0_edit_of_active_packet_rejected(self):
        # D-145 (5): an added packet is active and needs --packet (rewritten: 'modified' now means merged, see AR4).
        self.phase(OTHER,'implementing');self.diff(('added',OTHER))
        self.assertIn('R2+ active packet requires --packet',self.run_cli('status',expected=1))
    def test_r0_typo_in_merged_packet_passes(self):
        # AR4: a merged packet (in the base tree, so 'modified') is fixed on the R0 route without --packet.
        self.phase(OTHER,'implementing');self.diff(('modified',OTHER))
        self.assertEqual(self.run_cli('status')['blockers'],[])
        self.diff(('modified',OTHER),('added',MINE))
        self.assertIn('R2+ active packet requires --packet',self.run_cli('status',expected=1))
    def test_r0_packet_deletion_without_archive_rejected(self):
        self.state['http_error_path']='/contents/'+self.archive(OTHER)  # GitHub 404: the archive is absent.
        self.diff(('removed',OTHER))
        self.assertIn(LEAVE,self.run_cli('status',expected=1))
        self.assertFalse(self.fetched(self.archive(OTHER)))
    def test_r0_removal_with_preexisting_archive_rejected(self):
        a,b='docs/plans/2026-01-01-a.md','docs/plans/2026-01-02-b.md'
        self.phase(self.archive(a),'archive');self.phase(b,'implementing');self.diff(('removed',a))
        for action in ('status','ready'):
            self.assertIn(LEAVE,self.run_cli(action,expected=1))
        self.assertFalse(self.fetched(self.archive(a)))
    def test_r0_archive_move_without_phase_archive_passes(self):
        # AR1 (rewritten from the Phase archive rejection): #215's move passes whatever the Phase line says.
        self.phase(self.archive(OTHER),'implementing');self.diff(('renamed',self.archive(OTHER),OTHER))
        self.assertEqual(self.run_cli('status')['blockers'],[])
    def test_archive_move_reads_no_archive_contents(self):
        # AR1 (rewritten from the archive-fetch failure): the move needs no archive read, so a 404 there cannot matter;
        # a move out of docs/plans to anywhere but the same-named archive still stops.
        self.state['http_error_path']='/contents/'+self.archive(OTHER)
        self.diff(('renamed',self.archive(OTHER),OTHER))
        self.assertEqual(self.run_cli('status')['blockers'],[])
        self.diff(('renamed','docs/elsewhere/'+Path(OTHER).name,OTHER))
        self.assertIn(LEAVE,self.run_cli('status',expected=1))
    def test_packet_scope_does_not_list_docs_or_read_plans(self):
        self.run_cli('status')
        self.configure_packet();self.run_cli('status','--packet',MINE)
        calls=[' '.join(call) for call in self.load()['calls']]
        for path in ('/contents/docs?','/contents/docs/plans?','/contents/docs/Plans.md'):
            self.assertFalse([c for c in calls if path in c],path)
    def test_unknown_file_status_is_input_error(self):
        self.diff(('moved','docs/plans/2026-09-13-other.md'))
        self.assertIn('unknown PR file status',self.run_cli('status',expected=2))
    def test_truncated_diff_rejected(self):
        for count,expected in ((3000,1),(2999,0)):
            with self.subTest(count=count):
                self.state['files']=[dict(filename=f'docs/x{i}.md',status='modified') for i in range(count)];self.save()
                result=self.run_cli('status',expected=expected)
                if expected:self.assertIn('PR diff unavailable or truncated',result)

# D-145: synthetic packets for the rebuild (Matrix docs/plans/test-matrices/2026-10-28-process-rebuild.md).
PKT='docs/plans/2026-10-20-lane.md';MATRIX='docs/plans/test-matrices/2026-10-20-lane.md'
def rebuild_packet(risk='R3',form='old',gate='ready,merge,manual',l3=None,test_plan=True,link=True,extra='',minimum='2',
                   scope='- scope item\n',ledger='| C1 | x | y | z | n |\n',recovery='- restore\n',plan=None):
    ws={'Risk':risk,'Plan Commit':plan or 'pending','Amendments':'none'}
    if form=='old':ws={'Phase':'implementing',**ws,'Coordinator':'c','Writer':'w','Final Review Minimum':minimum,'Human Gate':gate}
    elif gate is not None:ws['Human Gate']=gate
    text='# Plan Packet: lane\n\n## Workflow State\n\n'+''.join(f'- {k}: {v}\n' for k,v in ws.items())
    text+=f'\n## Risk\n\nRisk: {risk}\n\n## Scope\n\n{scope}\n## Acceptance Criteria\n\n- AC1\n\n## Contract Ledger\n\n{ledger}\n'
    if test_plan:text+='## Test Plan\n\n'+('Test Design Matrix: [m](test-matrices/2026-10-20-lane.md)\n' if link else '- none\n')+'\n'
    if l3 is not None:text+=f'## L3\n\n{l3}'
    if risk=='R4':text+=f'## Recovery\n\n{recovery}\n## Data Safety\n\n- data\n'
    return text+extra

class PacketForm(SyncCase):
    """PH7 / PH10: the form is the first add after the origin; the Phase line must agree."""
    def add(self,text,path=PKT):return self.repo.commit({path:text},'packet')
    def form(self,path=PKT):return g.packet_form(path,self.repo.base0,self.head())
    def test_form_from_first_add_parent(self):
        self.add(rebuild_packet());self.assertEqual(self.form(),'old')
        self.repo.commit({g.APPROVE:'#!/bin/sh\n'},'approve script');self.add(rebuild_packet()+'more\n')
        self.assertEqual(self.form(),'old')  # the first add after the origin decides
        other='docs/plans/2026-10-21-new.md';self.add(rebuild_packet(form='new'),other)
        self.assertEqual(self.form(other),'new')
        with self.assertRaisesRegex(g.GateError,'must not have a Phase line'):g.parse_packet(rebuild_packet(),'new')
        with self.assertRaisesRegex(g.GateError,'needs its Phase line'):g.parse_packet(rebuild_packet(form='new'),'old')
        with self.assertRaisesRegex(g.GateError,'R3 or R4'):g.parse_packet(rebuild_packet(form='new',risk='R2'),'new')
    def test_readded_path_and_rename_are_new(self):
        # A path that existed before the origin and is re-added later is judged by the re-add (range = origin..HEAD).
        self.repo.commit({g.APPROVE:'#!/bin/sh\n',PKT:rebuild_packet()},'old lane');old_origin=self.head()
        self.repo.commit({PKT:None},'archive');self.add(rebuild_packet())
        self.assertEqual(g.packet_form(PKT,old_origin,self.head()),'new')
        # PH10: a rename is not followed; the renamed path is a new add after plan-approve.sh.
        self.repo.move(PKT,'docs/plans/2026-10-22-renamed.md');self.repo.git('commit','-qm','rename')
        self.assertEqual(g.packet_form('docs/plans/2026-10-22-renamed.md',old_origin,self.head()),'new')
        with self.assertRaisesRegex(g.GateError,'never renamed or re-cut'):g.parse_packet(rebuild_packet(),'new')
    def test_unreadable_parent_tree_is_an_error(self):
        # Final review C3: a failed ls-tree of the add's parent is undecidable, never 'old'.
        self.add(rebuild_packet())
        real=self.repo.bridge
        def failing(*args,**kw):
            if args and args[0]=='ls-tree':raise g.GateError('command failed: git (exit 128)',2)
            return real(*args,**kw)
        with patch.object(g,'git',side_effect=failing),self.assertRaisesRegex(g.GateError,'tree を読めず'):self.form()
        self.assertEqual(self.form(),'old')  # a successful empty listing is the absence of plan-approve.sh
    def test_root_add_and_missing_add_are_errors(self):
        with self.assertRaisesRegex(g.GateError,'commit が見つからない'):self.form()
        self.repo.git('checkout','-q','--orphan','root');self.repo.git('rm','-rqf','.')
        root=self.repo.commit({PKT:rebuild_packet()},'root add')
        with self.assertRaisesRegex(g.GateError,'親が無'):g.packet_form(PKT,self.repo.base0,root)

class PacketReuse(SyncCase):
    """C1..C5, RC1..RC4, RC1b, GA1..GA4: lane docs commits against a packet in a synthetic repo."""
    exercise=RecordLifecycle.exercise;old=SyncRecords.old;rec=SyncRecords.rec;stop=SyncRecords.stop
    carried=SyncRecords.carried
    def setUp(self):
        super().setUp()
        self.repo.git('reset','-q','--hard',self.repo.base0)
        self.repo.source=self.repo.commit({PKT:rebuild_packet(l3='- PASS 基準: 9 pt 以上\n'),MATRIX:'## Manual\n\n- 9 pt 以上\n'},'lane + L3')
    def lane(self,files):return self.repo.commit(files,'lane docs')
    def edit(self,**kw):return self.lane({PKT:rebuild_packet(**kw)})
    def reset(self):self.repo.git('reset','-q','--hard',self.repo.source)
    def docs(self,old,req=NO_MANUAL,**kw):
        return self.rec(old,kind='review',req=req,stage='docs',**SYNC|dict(values=['https://example.invalid/p3'])|kw,packet=PKT)
    def docs_stop(self,old,*texts,req=NO_MANUAL,code=1,**kw):
        with self.assertRaises(g.GateError) as caught:self.docs(old,req=req,**kw)
        self.assertEqual((caught.exception.code,self.writes),(code,[]),str(caught.exception))
        for text in texts:self.assertIn(text,str(caught.exception))
    def carry(self,req=REQ):return self.rec(self.old(),kind='r4',req=req,packet=PKT)
    def status_ok(self,record,req=NO_MANUAL):
        with patch.object(g,'api',side_effect=AssertionError('status must not call GitHub here')):
            self.gate.nonci(dict(head=self.head(),base=self.repo.base,requirements=req,server=dict(record=record)))
    def test_c1_docs_commit_keeps_l3(self):
        self.lane({'docs/lane.md':'typo fixed\n'})
        self.assertEqual(self.carry()['manual'],self.carried|dict(source_head=self.repo.source))
        # Explicit A takes the same route.
        record=self.rec(self.old(),kind='manual',source=self.repo.source,approval=None,packet=PKT)
        self.assertEqual(record['manual']['source_head'],self.repo.source)
    def test_c2_l3_section_change_stops(self):
        self.edit(l3='- PASS 基準: 10 pt 以上\n')
        self.stop(self.old(),'L3 節が変わった',kind='r4',packet=PKT)
    def test_c3_runtime_unknown_workflow_and_rename_stop(self):
        for files,texts in (({'src/app.ts':'app2\n'},['not docs only','frontend']),({'weird/x.bin':'?\n'},['not docs only','unknown']),
                            ({'docs/DEV_WORKFLOW.md':'gate\n'},['not docs only','workflow'])):
            with self.subTest(texts=texts):
                self.reset();self.lane(files)
                self.stop(self.old(),*texts,kind='r4',packet=PKT)
        self.reset();self.repo.move('src/x.ts','docs/x.md');self.repo.git('commit','-qm','rename into docs')
        self.stop(self.old(),'src/x.ts','not docs only',kind='r4',packet=PKT)
    def test_c4_r4_never_takes_docs_route(self):
        self.lane({'docs/lane.md':'typo\n'})
        self.stop(self.old(),'lane commit',kind='r4',req=REQ|dict(risk='R4'),packet=PKT)
    def test_c5_old_packet_without_l3_compares_test_plan_and_matrix(self):
        self.repo.git('reset','-q','--hard',self.repo.base0)
        self.repo.source=self.repo.commit({PKT:rebuild_packet(),MATRIX:'## Manual\n\n- 9 pt 以上\n'},'criteria in Test Plan + Matrix')
        two=rebuild_packet().replace('Test Design Matrix: [m](test-matrices/2026-10-20-lane.md)\n',
                                     '[a](test-matrices/2026-10-20-lane.md) [b](test-matrices/other.md)\n')
        cases=[({PKT:rebuild_packet().replace('Test Design Matrix: [m]','合格基準 10 pt。Test Design Matrix: [m]')},'L3 節が変わった'),
               ({MATRIX:'## Manual\n\n- 10 pt 以上\n'},'L3 節が変わった'),
               ({'docs/lane.md':'unrelated\n'},None),
               ({PKT:rebuild_packet(test_plan=False)},'比較対象が取れない'),
               ({PKT:rebuild_packet(link=False)},'比較対象が取れない'),
               ({PKT:two},'比較対象が取れない'),
               ({MATRIX:None},'比較対象が取れない')]
        for files,reason in cases:
            with self.subTest(reason=reason,files=sorted(files)):
                self.reset();self.lane(files)
                if reason:self.stop(self.old(),reason,kind='r4',packet=PKT)
                else:self.assertEqual(self.carry()['manual']['source_head'],self.repo.source)
    def test_rc1_docs_closure(self):
        self.lane({'docs/lane.md':'p3 typo\n'})
        record=self.docs(self.old(req=NO_MANUAL))
        self.assertEqual(record['review']['closure'],dict(head=self.head(),base=self.repo.base,sync_from=self.repo.source))
        record['manual']=dict(outcome='not-required',evidence=[])
        self.status_ok(record)
        # The status recheck still names a later code commit.
        self.repo.commit({'src/app.ts':'later\n'});record.update(head=self.head());record['review']['closure']['head']=self.head()
        with self.assertRaisesRegex(g.GateError,'not docs only'):self.status_ok(record)
    def test_rc1b_manual_free_r3_and_manual_change(self):
        self.repo.git('reset','-q','--hard',self.repo.base0)
        self.repo.source=self.repo.commit({PKT:rebuild_packet(form='new',gate=None)},'new form, no L3')
        self.lane({'docs/lane.md':'p3\n'})
        self.assertEqual(self.docs(self.old(req=NO_MANUAL))['review']['closure']['sync_from'],self.repo.source)
        self.reset();self.edit(form='new',gate=None,l3='- added\n')
        self.docs_stop(self.old(req=NO_MANUAL),'manual の要否')
    def test_rc2_gate_docs_plan_change_and_evidence(self):
        l3='- PASS 基準: 9 pt 以上\n'
        for files,text in (({'docs/DEV_WORKFLOW.md':'gate\n'},'workflow'),
                           ({PKT:rebuild_packet(l3=l3,scope='- more\n')},'計画の節が変わった: Scope'),
                           ({PKT:rebuild_packet(l3=l3).replace('- AC1','- AC1 widened')},'Acceptance Criteria'),
                           ({PKT:rebuild_packet(l3=l3,ledger='| C2 | x | y | z | n |\n')},'Contract Ledger')):
            with self.subTest(text=text):
                self.reset();self.lane(files)
                self.docs_stop(self.old(req=NO_MANUAL),text)
        self.reset();self.lane({'docs/lane.md':'p3\n'})
        self.docs_stop(self.old(req=NO_MANUAL),'--evidence',code=2,values=[])
    def test_rc3_old_l3_basis_change_stops_docs_closure(self):
        self.repo.git('reset','-q','--hard',self.repo.base0)
        self.repo.source=self.repo.commit({PKT:rebuild_packet(),MATRIX:'## Manual\n\n- 9 pt\n'},'old manual packet')
        for files in ({MATRIX:'## Manual\n\n- 10 pt\n'},{PKT:rebuild_packet().replace('Test Design Matrix','基準 10 pt。Test Design Matrix')}):
            with self.subTest(files=sorted(files)):
                self.reset();self.lane(files)
                self.docs_stop(self.old(req=NO_MANUAL),'L3 節が変わった')
    def test_rc_machine_closure_after_amendment_needs_audit(self):
        # GA (D-145 (7)) asks for an audited closure: neither the docs nor the sync machine closure may stand in for it.
        # Final review C4: with the same plan sections the next step is the audited closure, not a fresh broad.
        self.lane({'docs/lane.md':'p3\n'})
        amended=NO_MANUAL|dict(amendments=[self.repo.source])
        closure='a fresh broad is not needed; record --review-stage closure'
        self.docs_stop(self.old(req=NO_MANUAL),closure,req=amended)
        for stage in ('sync','docs'):
            with self.subTest(stage=stage),self.assertRaises(g.GateError) as caught:
                self.rec(self.old(req=NO_MANUAL),kind='review',req=amended,stage=stage,packet=PKT,
                         **SYNC|dict(values=['https://example.invalid/p3'] if stage=='docs' else []))
            self.assertIn(closure,str(caught.exception));self.assertNotIn('fresh broad required',str(caught.exception))
            self.assertEqual(self.writes,[])
        # Changed plan sections after the amendment still need a fresh broad.
        self.reset();self.edit(l3='- PASS 基準: 9 pt 以上\n',scope='- widened\n')
        self.docs_stop(self.old(req=NO_MANUAL),FRESH_BROAD,req=amended)
    def test_rc4_r4_rejected(self):
        self.lane({'docs/lane.md':'p3\n'})
        self.docs_stop(self.old(req=NO_MANUAL),'needs an R3',req=NO_MANUAL|dict(risk='R4'))
    # GA1..GA4: a Gated Amendment after the broad keeps it when the plan sections are byte-equal.
    def ga(self,risk='R3',**kw):
        self.repo.git('reset','-q','--hard',self.repo.base0)
        gate='ready,merge,r4' if risk=='R4' else 'ready,merge'
        broad=self.repo.commit({PKT:rebuild_packet(risk=risk,gate=gate)},'broad head')
        self.repo.commit({PKT:rebuild_packet(risk=risk,gate=gate,**kw)},'amendment');amended=self.head()
        record=good_record();record.update(head=self.head(),base=self.repo.base)
        record['review']['broad'].update(head=broad,base=self.repo.base,plan_commit=P,amendments=[])
        record['review']['closure']=dict(head=self.head(),base=self.repo.base,audit=dict(model='opus',run_ref='closure',evidence=['https://example.invalid/c']))
        record['manual']=dict(outcome='not-required',evidence=[]);record['r4']=dict(outcome='not-required',evidence=[])
        self.gate.args=args(packet=PKT)
        req=dict(risk=risk,plan_commit=P,amendments=[amended],minimum=2,manual=False,r4=False)
        return lambda:self.status_ok(record,req)
    def test_ga1_same_plan_sections_take_closure(self):
        self.ga(extra='\n## Review Response\n\n- amended wording\n')()
        # Without a packet (no sections to compare) the amendment still needs a fresh broad.
        check=self.ga(extra='\n## Review Response\n\n- x\n');self.gate.args=args()
        with self.assertRaisesRegex(g.GateError,'^'+re.escape(FRESH_BROAD)+'$'):check()
    def test_ga2_scope_change_or_unreadable_needs_fresh_broad(self):
        for kw in (dict(scope='- widened\n'),dict(extra='\n## Scope\n\n- duplicated\n')):
            with self.subTest(kw=kw),self.assertRaisesRegex(g.GateError,'^'+re.escape(FRESH_BROAD)+'$'):self.ga(**kw)()
    def test_ga3_ledger_change_needs_fresh_broad(self):
        with self.assertRaisesRegex(g.GateError,'^'+re.escape(FRESH_BROAD)+'$'):self.ga(ledger='| C9 | new | y | z | n |\n')()
    def test_ga4_r4_recovery_change_needs_fresh_broad(self):
        self.ga(risk='R4',extra='\n## Review Response\n\n- note\n')()
        with self.assertRaisesRegex(g.GateError,'^'+re.escape(FRESH_BROAD)+'$'):self.ga(risk='R4',recovery='- new restore\n')()

class NewForm(unittest.TestCase):
    """MN1..MN5, PH1..PH3 at the parser."""
    def parse(self,form='new',**kw):return g.parse_packet(rebuild_packet(form=form,**kw),form)
    def test_mn1_mn3_derived(self):
        self.assertEqual(self.parse(gate=None,plan=P),dict(risk='R3',plan_commit=P,amendments=[],minimum=2,manual=False,r4=False))
        self.assertEqual(self.parse(gate=None,risk='R4',plan=P),dict(risk='R4',plan_commit=P,amendments=[],minimum=2,manual=False,r4=True))
        self.assertTrue(self.parse(gate=None,l3='- PASS\n')['manual'])
        # A heading inside a code fence is not the L3 section.
        self.assertFalse(self.parse(gate=None,extra='\n```\n## L3\n```\n')['manual'])
    def test_mn2_written_minimum_only_raises_and_old_keeps_written(self):
        text=rebuild_packet(form='new',gate=None).replace('- Amendments: none\n','- Amendments: none\n- Final Review Minimum: 1\n')
        self.assertEqual(g.parse_packet(text,'new')['minimum'],2)
        self.assertEqual(self.parse(form='old',minimum='1')['minimum'],1)
    def test_mn3_mn5_written_gates_are_kept(self):
        self.assertTrue(self.parse(gate='ready,merge,r4')['r4'])
        self.assertTrue(self.parse(gate='ready,merge,manual')['manual'])
        self.assertTrue(self.parse(form='old',gate='ready,merge,r4')['r4'])
        gate=object.__new__(g.Gate);gate.args=args()
        record=good_record();record['r4']=dict(outcome='not-required',evidence=[])
        r4=self.parse(gate='ready,merge,manual,r4',plan=P)['r4']
        with self.assertRaisesRegex(g.GateError,'r4 incomplete'):
            gate.nonci(dict(head=H,base=B,requirements=REQ|dict(r4=r4),server=dict(record=record)))
    def test_mn4_conditions(self):
        new=g.gate_conditions(rebuild_packet(form='new',gate=None,l3='- x\n'))
        self.assertEqual(new,{'Risk':'R3','Final Review Minimum':None,'Human Gate':None,'L3 section':True})
        self.assertNotEqual(new,g.gate_conditions(rebuild_packet(form='new',gate=None)))
        self.assertNotIn('L3 section',g.gate_conditions(rebuild_packet()))
    def test_ph1_ph2_pending_and_sha(self):
        self.assertEqual(self.parse(gate=None)['plan_commit'],'pending')
        gate=object.__new__(g.Gate);gate.args=args()
        with self.assertRaisesRegex(g.GateError,'^Plan Gate incomplete$'):
            gate.nonci(dict(head=H,base=B,requirements=REQ|dict(plan_commit='pending'),server=None))
        gate.nonci(dict(head=H,base=B,requirements=REQ,server=dict(record=good_record())))

class RebuildCLI(unittest.TestCase):
    """PH1/PH2/PH7/MN1/MN4 through the CLI, R2N1..R2N5, CI2, RP1/RP2."""
    setUp=CLI.setUp;tearDown=CLI.tearDown;save=CLI.save;load=CLI.load;run_cli=CLI.run_cli
    def new_packet(self,text_fn,files=()):
        # plan-approve.sh lands on main before the packet: the packet is new-form.
        (self.repo/'scripts').mkdir(exist_ok=True);(self.repo/g.APPROVE).write_text('#!/bin/sh\n')
        self.git('add',g.APPROVE);self.git('commit','-qm','plan-approve.sh on main')
        base=self.git('rev-parse','HEAD');self.state['pr']['base']['sha']=base
        path=self.repo/PKT;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(text_fn('pending'))
        self.git('add',PKT);self.git('commit','-qm','plan-first');plan=self.git('rev-parse','HEAD')
        path.write_text(text_fn(plan));self.git('commit','--allow-empty','-qam','plan-approve');self.head=self.git('rev-parse','HEAD')
        self.state['pr']['head']['sha']=self.head;self.state['runs'][0]['head_sha']=self.head
        self.state['files']=[dict(filename=f,status='modified') for f in files]+[dict(filename=PKT,status='added')]
        self.state['contents'][PKT]=text_fn(plan);self.state['snapshots']={plan:{PKT:text_fn('pending')}}
        self.save();return plan
    def test_ph1_ph2_new_form_cli(self):
        self.new_packet(lambda plan:rebuild_packet(form='new',gate=None))
        status=self.run_cli('status','--packet',PKT)
        self.assertEqual((status['state'],status['blockers']),('Plan Gate',['Plan Gate incomplete']))
        self.assertIn('Plan Gate incomplete',self.run_cli('capture','--packet',PKT,expected=1))
        self.load();self.state['contents'][PKT]=rebuild_packet(form='new',gate=None,plan=P);self.save()
        self.assertNotIn('Plan Gate incomplete',self.run_cli('status','--packet',PKT)['blockers'])
    def test_mn1_workflow_and_mn4_l3_removal(self):
        text=lambda plan,**kw:rebuild_packet(form='new',gate=None,plan=None if plan=='pending' else plan,**kw)
        plan=self.new_packet(lambda plan:text(plan,l3='- PASS\n'),files=('scripts/pr-gate.py',))
        captured=json.loads(Path(self.run_cli('capture','--packet',PKT)['capture']).read_text())['requirements']
        self.assertEqual((captured['minimum'],captured['manual'],captured['r4']),(2,True,False))
        self.load();self.state['contents'][PKT]=text(plan);self.save()
        self.assertIn('approved packet condition changed: L3 section',self.run_cli('status','--packet',PKT,expected=1))
        self.load();self.state['contents'][PKT]=rebuild_packet(form='new',gate='ready,merge,r4',l3='- PASS\n',plan=plan);self.save()
        self.assertIn('approved packet condition changed: Human Gate',self.run_cli('status','--packet',PKT,expected=1))
    def test_ph7_phase_line_in_new_form_rejected(self):
        self.new_packet(lambda plan:rebuild_packet(form='old',plan=None if plan=='pending' else plan))
        self.assertIn('must not have a Phase line',self.run_cli('status','--packet',PKT,expected=1))
    def body(self,text):self.load();self.state['pr']['body']=text;self.save()
    def diff(self,*names):self.load();self.state['files']=[dict(filename=n,status='modified') for n in names];self.save()
    def test_r2n1_r2n3_r2_route(self):
        self.body('Risk: R2（目視: 不要）');self.diff('src/features/x/view.tsx')
        self.assertIn('workflow record missing',self.run_cli('status','--risk','R2')['blockers'])
        capture=json.loads(Path(self.run_cli('capture','--risk','R2')['capture']).read_text())['requirements']
        self.assertEqual(capture,dict(risk='R2',plan_commit=None,amendments=[],minimum=1,manual=False,r4=False))
        self.body('Risk: R2（目視: 要）')
        capture=self.run_cli('capture','--risk','R2','--manual','required')['capture']
        self.run_cli('record','--risk','R2','--manual','required','--capture',capture,'--kind','review','--review-stage','broad',
                     '--pass-model','opus','--run-ref','r','--evidence','https://example.invalid/r','--outcome','pass',
                     '--reviewed-head',self.head,'--pr-reviews','0')
        self.assertEqual(self.run_cli('status','--risk','R2','--manual','required')['blockers'],['manual incomplete'])
    def test_r2n2_r2n4_refused_paths(self):
        cases=[('R2','docs/DEV_WORKFLOW.md','workflow=true'),('R2','weird/x.bin','unknown=true'),('R0','weird/x.bin','unknown=true'),
               ('R1','src-tauri/src/db/migration.rs','src-tauri/src/db/migration.rs'),('R2','src-tauri/src/db/schema_v12.rs','schema_v12.rs'),
               ('R0','src-tauri/src/db/migration_tx.rs','migration_tx.rs'),('R0','package.json','package.json'),('R1','.npmrc','.npmrc'),
               ('R2','package-lock.json','package-lock.json'),('R1','scripts/pr-gate.py','workflow かつ rust')]
        for risk,path,reason in cases:
            with self.subTest(risk=risk,path=path):
                self.body(f'Risk: {risk}（目視: 不要）');self.diff(path)
                error=self.run_cli('status','--risk',risk,expected=1)
                self.assertIn(reason,error);self.assertIn('R3 の packet で起票する',error)
        for risk,path in (('R2','src-tauri/src/stock.rs'),('R2','src/features/x/view.tsx'),('R0','AGENTS.md'),
                          ('R1','docs/templates/plan-packet.md'),('R0','.github/pull_request_template.md')):
            with self.subTest(accepted=path):
                self.body(f'Risk: {risk}（目視: 不要）');self.diff(path)
                self.run_cli('status','--risk',risk)
        self.body('Risk: R2（目視: 不要）');self.load();self.state['files']=[dict(filename=PKT,status='added')];self.save()
        self.assertIn('R2+ active packet requires --packet',self.run_cli('status','--risk','R2',expected=1))
    def test_r2n5_body_risk_line(self):
        for body in ('','Risk: R1（目視: 不要）','Risk: R0（目視: 要）','Risk: R0 (目視: 不要)',
                     'Risk: R0（目視: 不要）\nRisk: R0（目視: 不要）','Risk: R0（目視: 不要） extra'):
            with self.subTest(body=body):
                self.body(body);self.assertIn('Risk',self.run_cli('status',expected=1))
        self.body('<!-- Risk: R2（目視: 要） -->\nRisk: R0（目視: 不要）\r\n');self.assertEqual(self.run_cli('status')['blockers'],[])
    def test_ci2_same_head_draft_run(self):
        self.assertEqual(self.load()['pr']['draft'],True)
        self.assertEqual(self.run_cli('status')['blockers'],[])  # a Draft pull_request run with Merge gate counts
        self.load();self.state['runs'][0]['head_sha']='e'*40;self.save()
        self.assertIn('CI run absent',' '.join(self.run_cli('status')['blockers']))
        self.load();self.state['runs'][0]['head_sha']=self.head;self.state['checks'][0]['name']='Draft (no merge evidence)';self.save()
        self.assertIn('Merge gate missing',' '.join(self.run_cli('status')['blockers']))
    def report(self,text):
        path=self.root/'report.txt';path.write_text(text);return str(path)
    def test_rp1_rp2_report(self):
        self.body('Risk: R2（目視: 不要）');self.diff('src/features/x/view.tsx')
        common=('--risk','R2','--kind','review','--review-stage','broad','--pass-model','opus','--run-ref','r',
                '--evidence','https://example.invalid/r','--outcome','pass','--pr-reviews','0')
        capture=self.run_cli('capture','--risk','R2')['capture']
        for text in ('summary\n',f'REVIEWED_HEAD={self.head}\nREVIEWED_HEAD={self.head}\n','REVIEWED_HEAD=abc\n',f'REVIEWED_HEAD={self.head.upper()}\n'):
            with self.subTest(text=text):
                self.run_cli('record',*common,'--capture',capture,'--report',self.report(text),expected=2)
        self.run_cli('record',*common,'--capture',capture,'--report',self.report(f'REVIEWED_HEAD={self.head}\n'),'--reviewed-head',self.head,expected=2)
        self.assertIn('reviewed head differs from capture head',
                      self.run_cli('record',*common,'--capture',capture,'--report',self.report(f'REVIEWED_HEAD={OLD}\n'),expected=1))
        self.assertEqual(self.load()['comments'],[])
        self.run_cli('record',*common,'--capture',capture,'--report',self.report(f'notes\nREVIEWED_HEAD={self.head}\n'))
        self.assertEqual(len(self.load()['comments']),1)

if __name__=='__main__':unittest.main()
