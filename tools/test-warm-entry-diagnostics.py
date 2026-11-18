#!/usr/bin/env python3
"""Synthetic orchestration and corruption controls; no compiler speed claim."""
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
def module(name,path):
    spec=importlib.util.spec_from_file_location(name,path); result=importlib.util.module_from_spec(spec); spec.loader.exec_module(result); return result
D=module('diagnostics',HERE/'run-warm-entry-diagnostics.py')
T=module('comparison_controls',HERE/'test-warm-entry-comparison.py')


def synthetic(output):
    root=output.parent; campaign=root/'primary'; campaign.mkdir()
    fixture=campaign/'fixture'; (fixture/'src').mkdir(parents=True)
    (fixture/'src/slice_main.rs').write_text('fn main() {}\n'); (fixture/'Cargo.toml').write_text('# synthetic fixture\n')
    oracle=b'synthetic diagnostic oracle\n'; (campaign/'runtime-oracle.stdout').write_bytes(oracle)
    package=root/'tools/toolchain'; (package/'bin').mkdir(parents=True)
    pristine=root/'pristine/lust'; pristine.parent.mkdir()
    for path in [pristine,*[package/'bin'/name for name in ('lustr','lust','lust-core','cargo','rustc-reference')]]: path.write_bytes(b'synthetic tool')
    tools={lane:str(path) for lane,path in [('entry',package/'bin/lustr'),('integrated',package/'bin/lust'),('pristine',pristine),('cargo',package/'bin/cargo')]}
    primary={'status':'completed','pairs':12,'candidate_identity':{'revision':'a'*40,'tree':'b'*40},'candidate_source':str(root/'candidate'),
             'pristine_source':str(root/'pristine-source'),'tool_paths':tools,'fixture_files':D.S.inventory(fixture),
             'runtime_oracle_sha256':D.S.digest(campaign/'runtime-oracle.stdout'),'compiler_and_runner_sha256':{p:D.S.digest(Path(p)) for p in tools.values()}}
    D.S.write_json(campaign/'report.json',primary)
    selector=root/'selector'; selector.write_bytes(b'synthetic selector')
    receipt={'status':'completed','source':primary['candidate_identity'],'binary_sha256':D.S.digest(selector)}
    D.S.write_json(root/'selector-receipt.json',receipt)
    output.mkdir(); shutil.copyfile(campaign/'report.json',output/'source-report.json'); shutil.copyfile(root/'selector-receipt.json',output/'selector-receipt.json')
    args=SimpleNamespace(output=output,campaign=campaign,selector=selector,selector_receipt=root/'selector-receipt.json')
    run=D.Diagnostic(args,primary,receipt); recorder=run.recorder
    def command(argv,env,cwd,lane,phase,expected=0):
        argv=list(map(str,argv)); data=error=b''
        if argv[0]==str(selector):
            data=json.dumps({'phase':'selection','success':True,'target':'slice','manifest':str(fixture/'Cargo.toml'),
                             'elapsed_ns':1000000 if 'LUSTR_NO_SELECTION_CACHE' in env else 100000}).encode()
        elif len(argv)==1: data=oracle
        else:
            if argv[1]=='build': Path(argv[argv.index('-o')+1]).write_bytes(b'synthetic executable')
            if phase=='diagnostic-retained': error=T.PRISTINE_RETAINED_BUILD if argv[1]=='build' else T.PRISTINE_RETAINED_CHECK
            if 'LUSTR_TRACE_PATH' in env:
                with Path(env['LUSTR_TRACE_PATH']).open('a') as stream: stream.write('{"engine":"lust","entry":"daemon"}\n')
        if len(argv)>1 and 'LUSTR_SELECTION_DIAGNOSTICS' in env:
            with Path(env['LUSTR_SELECTION_DIAGNOSTICS']).open('a') as stream: stream.write('{"selection_cache":"hit"}\n')
        row={'kind':'command','id':len(recorder.records),'argv':argv,'environment':dict(env),'cwd':str(cwd),'lane':lane,'phase':phase,
             'expected_exit':expected,'exit':expected,'signal':None,'timed_out':False,'output_limit':False,'seconds':.002 if lane=='entry' else .001}
        for name,value in [('stdout',data),('stderr',error)]:
            path=output/'commands'/f"{row['id']:05}.{name}"; path.write_bytes(value)
            row[name]={'file':str(path.relative_to(output)),'bytes':len(value),'sha256':D.S.digest(path)}
        recorder.event(row); return row,data,error
    class Daemon:
        def __init__(self,recorder,executable,env,cwd,lane,scope):
            self.row={'kind':'daemon-start','lane':lane,'pid':len(recorder.records)+100,'scope':scope,'environment':dict(env),'executable':str(executable),'cwd':str(cwd),'socket':env['LUST_DAEMON_SOCKET']}
        def __enter__(self): recorder.event(self.row); return self
        def alive(self): pass
        def __exit__(self,*args): recorder.event({'kind':'daemon-stop',**{key:self.row[key] for key in ('lane','pid','scope')},'exit':0,'error':None,'streams':[]})
    with recorder.raw,patch.object(recorder,'run',side_effect=command),patch.object(D.S,'Daemon',Daemon),patch.object(D.B,'source_identity',return_value=primary['candidate_identity']): run.execute()
    run.report['raw_sha256']=D.S.digest(output/'raw.jsonl'); D.S.write_json(output/'report.json',run.report); (output/'README.md').write_text(D.render(run.report))


class Controls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary=tempfile.TemporaryDirectory(); cls.root=Path(cls.temporary.name); cls.original=cls.root/'original'; synthetic(cls.original)
    @classmethod
    def tearDownClass(cls): cls.temporary.cleanup()
    def corrupted(self,mutate):
        output=self.root/self.id().split('.')[-1]; shutil.copytree(self.original,output)
        rows=[json.loads(line) for line in (output/'raw.jsonl').read_text().splitlines()]; mutate(rows)
        (output/'raw.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in rows))
        report=json.loads((output/'report.json').read_text()); report['raw_sha256']=D.S.digest(output/'raw.jsonl'); D.S.write_json(output/'report.json',report)
        return output
    def test_complete_synthetic_orchestration_replays(self): self.assertEqual(len(D.verify(self.original)['native_timings']),72)
    def test_wrong_integrated_executable_is_rejected(self):
        def mutate(rows): next(r for r in rows if r['kind']=='command' and r['phase']=='diagnostic-check' and r['lane']=='integrated')['argv'][0]='/unrelated/lust'
        with self.assertRaises(ValueError): D.verify(self.corrupted(mutate))
    def test_timed_trace_is_rejected(self):
        def mutate(rows): next(r for r in rows if r['kind']=='command' and r['phase']=='diagnostic-check')['environment']['LUST_STATS']='1'
        with self.assertRaises(ValueError): D.verify(self.corrupted(mutate))
    def test_unmatched_public_environment_is_rejected(self):
        def mutate(rows): next(r for r in rows if r['kind']=='command' and r['phase']=='diagnostic-check')['environment'].pop(D.OPT_OUT)
        with self.assertRaises(ValueError): D.verify(self.corrupted(mutate))
    def test_missing_retained_build_is_rejected(self):
        def mutate(rows): next(r for r in rows if r['kind']=='retained-proof' and r['operation']=='build')['kind']='omitted'
        with self.assertRaises(ValueError): D.verify(self.corrupted(mutate))
    def test_changed_selector_duration_is_rejected(self):
        def mutate(rows): next(r for r in rows if r['kind']=='selector-timing')['component_seconds']/=10
        with self.assertRaises(ValueError): D.verify(self.corrupted(mutate))
    def test_omitted_runtime_is_rejected(self):
        def mutate(rows):
            timing=next(r for r in rows if r['kind']=='diagnostic-timing' and r['operation']=='build')
            next(r for r in rows if r['kind']=='runtime-identity' and r['build_command']==timing['command'])['kind']='omitted'
        with self.assertRaises(ValueError): D.verify(self.corrupted(mutate))
    def test_component_rejects_another_selected_target(self):
        with self.assertRaises(ValueError): D.selector_result(b'{"phase":"selection","success":true,"target":"another","manifest":"/fixture/Cargo.toml","elapsed_ns":10}',Path('/fixture'))


if __name__=='__main__': unittest.main()
