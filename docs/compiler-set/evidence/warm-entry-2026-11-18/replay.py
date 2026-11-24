#!/usr/bin/env python3
"""Replay the preserved failed and complete campaigns without a GitHub download."""
import hashlib
import json
from collections import Counter
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import zipfile

ROOT=Path(__file__).resolve().parent


def require(condition,message):
    if not condition: raise ValueError(message)


def digest(data): return hashlib.sha256(data).hexdigest()
def encoded(value): return json.dumps(value,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()


def third_checks(output,campaign,report):
    summary=json.loads((ROOT/'third-replay-summary.json').read_text())
    require(summary['source']==report['candidate_identity'] and summary['archive_sha256']==campaign['archive_sha256']
            and summary['warm_target_met'] is False,'third replay summary binding differs')
    diagnostic=output/'lustr-warm-diagnostics'
    runner=ROOT/campaign['runner_directory']/'run-warm-entry-diagnostics.py'
    result=subprocess.run([sys.executable,'-B',str(runner),'verify',str(diagnostic)],capture_output=True,text=True)
    require(result.returncode==0,'third diagnostic replay failed: '+result.stderr)
    details=json.loads((diagnostic/'report.json').read_text())
    require(len(details['native_timings'])==campaign['diagnostic_native_samples']==144
            and len(details['selector_timings'])==campaign['diagnostic_selector_samples']==48,'third diagnostic sample counts differ')
    for name in ('lustr-warm-comparison','lustr-warm-diagnostics'):
        rows=[json.loads(line) for line in (output/name/'raw.jsonl').read_text().splitlines()]
        require(summary[name]=={'records':len(rows),'kinds':dict(Counter(row['kind'] for row in rows))},'third raw record counts differ')
    receipt=json.loads((output/'lustr-warm-selector-build/receipt.json').read_text())
    require(receipt['status']=='completed' and receipt['source']==campaign['executed_source'],'third selector source binding differs')
    for name,expected in campaign['qualification_source_sha256'].items():
        require(digest((ROOT/campaign['qualification_source_directory']/name).read_bytes())==expected,'third frozen qualification source changed')
    for name,expected in receipt['source_sha256'].items():
        relative=str(Path(name).relative_to(report['candidate_source']))
        require(campaign['qualification_source_sha256'][relative]==expected,'third selector receipt source differs')
    for name in ('lustr-warm-selection-contract','lustr-warm-selection-seed-contract'):
        gate=json.loads((output/name/'results.json').read_text())
        expected=summary['gates'][name]
        require(gate['status']=='completed' and gate['success'] is True and gate['error'] is None
                and len(gate['records'])==expected['invocations'] and len(gate['assertions'])==expected['assertions']
                and all(row['passed'] is True for row in gate['assertions'])
                and gate['profiler_sha256']==receipt['binary_sha256'],'third selector/seed gate result differs')
        for row in gate['records']:
            for stream in ('stdout','stderr'):
                path=Path(row[stream+'_file'])
                require(not path.is_absolute() and '..' not in path.parts,'unsafe qualification stream')
                require(digest((output/name/path).read_bytes())==row[stream+'_sha256'],'third qualification stream changed')
    log=(output/'lustr-warm-entry-contract.log').read_bytes()
    require(digest(log)==summary['gates']['entry-contract']['log_sha256']
            and b'test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;' in log,'third complete entry contract log differs')
    print('Third run: 144 native and 48 selector diagnostic timings replayed; public cached-edit proof, 140/265 selector gate, 38/85 seed gate and 16/16 entry log verified.')


def fourth_checks(output,campaign,report):
    summary=json.loads((ROOT/'fourth-replay-summary.json').read_text())
    require(summary['source']==report['candidate_identity'] and summary['archive_sha256']==campaign['archive_sha256']
            and summary['warm_target_met'] is False,'fourth replay summary binding differs')
    require(report['baseline_composition']['baseline']['revision']=='d57f485eeb2b638d1f95a0594294d11e8001fe8e',
            'fourth baseline changed')
    diagnostic=output/'lustr-warm-diagnostics'
    runner=ROOT/campaign['runner_directory']/'run-warm-entry-diagnostics.py'
    result=subprocess.run([sys.executable,'-B',str(runner),'verify',str(diagnostic)],capture_output=True,text=True)
    require(result.returncode==0,'fourth diagnostic replay failed: '+result.stderr)
    details=json.loads((diagnostic/'report.json').read_text())
    require(len(details['native_timings'])==campaign['diagnostic_native_samples']==144
            and len(details['selector_timings'])==campaign['diagnostic_selector_samples']==48,'fourth diagnostic sample counts differ')
    for name in ('lustr-warm-comparison','lustr-warm-diagnostics'):
        rows=[json.loads(line) for line in (output/name/'raw.jsonl').read_text().splitlines()]
        require(summary[name]=={'records':len(rows),'kinds':dict(Counter(row['kind'] for row in rows))},'fourth raw record counts differ')
    receipt=json.loads((output/'lustr-warm-selector-build/receipt.json').read_text())
    require(receipt['status']=='completed' and receipt['source']==campaign['executed_source'],'fourth selector source binding differs')
    for name,expected in campaign['qualification_source_sha256'].items():
        require(digest((ROOT/campaign['qualification_source_directory']/name).read_bytes())==expected,'fourth frozen qualification source changed')
    for name,expected in campaign['baseline_constructor_source_sha256'].items():
        require(digest((ROOT/campaign['baseline_constructor_source_directory']/name).read_bytes())==expected,'fourth frozen baseline constructor changed')
    baseline_seed=json.loads((output/'lustr-warm-baseline-build/baseline-selection-seed.json').read_text())
    baseline_tools=baseline_seed['payload']['tools']
    require(len(baseline_tools['identities'])==9 and baseline_tools['identities'][0][0]==baseline_tools['identities'][0][1]
            ==report['tool_paths']['baseline'],'fourth baseline constructor identities differ')
    for identity in baseline_tools['identities']:
        require(len(identity)==12 and all(type(value) is int for value in identity[2:])
                and stat.S_ISREG(identity[4]) and identity[7]>0,'fourth baseline tool stamp shape differs')
    for name,expected in receipt['source_sha256'].items():
        relative=str(Path(name).relative_to(report['candidate_source']))
        require(campaign['qualification_source_sha256'][relative]==expected,'fourth selector receipt source differs')

    gates={}
    for name in ('lustr-warm-selection-contract','lustr-warm-selection-seed-contract','lustr-warm-compact-selection-contract'):
        gate=json.loads((output/name/'results.json').read_text()); gates[name]=gate
        expected=summary['gates'][name]
        require(gate['status']=='completed' and gate['success'] is True and gate['error'] is None
                and len(gate['records'])==expected['invocations'] and len(gate['assertions'])==expected['assertions']
                and all(row['passed'] is True for row in gate['assertions'])
                and gate['profiler_sha256']==receipt['binary_sha256'],'fourth selector/seed/compact gate result differs')
        for row in gate['records']:
            require(type(row['exit']) is int and row['exit']>=0 and row.get('status')!='timeout'
                    and row.get('timeout_seconds') is None,'fourth gate recorded a timeout or signal')
            for stream in ('stdout','stderr'):
                path=Path(row[stream] if name.endswith('compact-selection-contract') else row[stream+'_file'])
                require(not path.is_absolute() and '..' not in path.parts,'unsafe fourth qualification stream')
                require(digest((output/name/path).read_bytes())==row[stream+'_sha256'],'fourth qualification stream changed')

    # Independently reconstruct semantic values from original streams. These
    # checks replay observations, not the unarchived filesystem mutations.
    def semantic(name,row):
        field='stdout' if name.endswith('compact-selection-contract') else 'stdout_file'
        value=json.loads((output/name/row[field]).read_bytes()); value.pop('elapsed_ns',None)
        return value

    paired={}
    for name,gate in gates.items():
        records={row['label']:row for row in gate['records']}
        count=0
        for row in gate['records']:
            if not row['label'].endswith('-uncached'): continue
            prefix=row['label'][:-9]
            candidate=records.get(prefix,records.get(prefix+'-cached'))
            require(candidate is not None and candidate['exit']==row['exit'],'fourth paired status differs')
            left,right=semantic(name,candidate),semantic(name,row)
            require(left==right,'fourth paired complete Selection differs')
            if left.get('phase')!='selection-fallback':
                field='stderr' if name.endswith('compact-selection-contract') else 'stderr_file'
                require((output/name/candidate[field]).read_bytes()==(output/name/row[field]).read_bytes(),
                        'fourth paired selection stderr differs')
            count+=1
        paired[name]=count
    require(paired['lustr-warm-selection-contract']==77,'fourth selector pair count differs')
    compact=gates['lustr-warm-compact-selection-contract']
    require(compact['runner_sha256']==campaign['qualification_source_sha256']['tools/test-native-compact-selection.py'],
            'fourth compact gate runner differs')
    runtime={row['label']:row for row in compact['records'] if row['kind']=='runtime'}
    expected_outputs={'default-build':b'selected-first:debug\n','changed-profile-and-output':b'selected-first:nodebug\n',
                      'different-selected-target':b'selected-second\n','restored-rust-body':b'selected-first:debug\n'}
    require(len(runtime)==8,'fourth compact runtime count differs')
    for label,expected in expected_outputs.items():
        candidates=[runtime[label+'-'+lane+'-runtime'] for lane in ('cached','uncached')]
        require(candidates[0]['executable_sha256']==candidates[1]['executable_sha256'],'fourth fallback artifact bindings differ')
        for row in candidates:
            require(row['exit']==0 and (output/'lustr-warm-compact-selection-contract'/row['stdout']).read_bytes()==expected
                    and (output/'lustr-warm-compact-selection-contract'/row['stderr']).read_bytes()==b'',
                    'fourth fallback runtime outcome differs')
    for row in compact['records']:
        if row['label'] in ('invalid-rust-body-cached','invalid-rust-body-uncached'):
            require(row['exit']==101 and b'E0308' in (output/'lustr-warm-compact-selection-contract'/row['stderr']).read_bytes(),
                    'fourth fallback lost the type-error rejection')
    log=(output/'lustr-warm-entry-contract.log').read_bytes()
    require(digest(log)==summary['gates']['entry-contract']['log_sha256']
            and b'test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;' in log,'fourth entry contract log differs')
    print('Fourth run: 144 native and 48 selector diagnostic timings replayed; 154/290 selector, 38/85 seed, 103/486 compact and 16/16 entry gates verified; 77 full-Selection selector pairs and eight exact fallback runtimes reconstructed.')


def main():
    manifest=json.loads((ROOT/'manifest.json').read_text())
    require(manifest['schema']=='lustr-warm-evidence-bundle-v1','bundle schema differs')
    for campaign in manifest['campaigns']:
        archive=ROOT/campaign['archive']
        require(digest(archive.read_bytes())==campaign['archive_sha256'],'original artifact ZIP changed')
        for name,expected in campaign['runner_sha256'].items():
            require(digest((ROOT/campaign['runner_directory']/name).read_bytes())==expected,'frozen executed runner changed')
        with tempfile.TemporaryDirectory(prefix='warm-entry-replay-') as temporary, zipfile.ZipFile(archive) as zipped:
            output=Path(temporary)
            for info in zipped.infolist():
                path=Path(info.filename)
                require(not path.is_absolute() and '..' not in path.parts,'unsafe artifact path')
            zipped.extractall(output)
            evidence=output/'lustr-warm-comparison'
            report=json.loads((evidence/'report.json').read_text())
            require(report['candidate_identity']==campaign['executed_source'] and report['status']==campaign['report_status']
                    and len(report['samples'])==campaign['sample_count'],'campaign identity or completion differs')
            require(digest((evidence/'raw.jsonl').read_bytes())==report['raw_sha256'],'campaign raw log changed')
            rows=[json.loads(line) for line in (evidence/'raw.jsonl').read_text().splitlines()]
            for row in rows:
                if row['kind']=='command':
                    for kind in ('stdout','stderr'):
                        data=(evidence/row[kind]['file']).read_bytes()
                        require(digest(data)==row[kind]['sha256'] and len(data)==row[kind]['bytes'],'original command stream changed')
            runner=ROOT/campaign['runner_directory']/'run-warm-entry-comparison.py'
            result=subprocess.run([sys.executable,'-B',str(runner),'verify',str(evidence)],capture_output=True,text=True)
            if campaign['label']=='first':
                require(result.returncode!=0 and 'only complete campaigns have results' in result.stderr,'failed campaign was admitted as performance evidence')
                require(report['samples']==[] and report['error']=='ValueError: native no-op repeated body or generated-code work','first failure differs')
                failed,=[row for row in rows if row['kind']=='command' and row['id']==31]
                require(failed['lane']=='pristine' and failed['phase']=='gate-retained-build' and failed['exit']==0,'original pristine failure binding differs')
                print('First run: original failed gate and all streams verified; zero timing samples; incomplete-result rejection retained.')
            else:
                require(result.returncode==0,'complete campaign replay failed: '+result.stderr)
                title=campaign['label'].capitalize()
                print(title+' run: '+result.stdout.strip())
                require(report['warm_target']['meets_target'] is False,'unsuccessful target was changed')
                constructor=json.loads((ROOT/campaign.get('constructor_replay','constructor-replay.json')).read_text())
                require(constructor['archive_sha256']==campaign['archive_sha256'] and constructor['source']==report['candidate_identity'],'constructor evidence binding differs')
                for name,expected in constructor['files'].items(): require(digest(zipped.read(name))==expected,'constructor evidence file changed')
                for name,expected in constructor['constructor_source_sha256'].items():
                    require(digest((ROOT/campaign.get('constructor_source_directory','second-constructor-source')/name).read_bytes())==expected,'frozen constructor source changed')
                base='lustr-warm-built/'
                launcher=report['tool_paths']['entry']
                require(zipped.read(base+'logs/preverify-selection-tools.status')==b'exit status: 0\n','constructor command failed')
                require(zipped.read(base+'logs/preverify-selection-tools.command').decode()==json.dumps(launcher)+' "--lustr-prime-selection-tools"\n','constructor used another launcher')
                certificate=json.loads(zipped.read(base+'toolchain/lib/lustr-selection-verified/'+constructor['certificate']))
                payload=certificate['payload']; tools=payload['tools']
                require(certificate['sha256']==digest(encoded(payload)) and constructor['certificate']=='tools-'+digest(encoded(tools))+'.json','constructor certificate checksum/key differs')
                require(payload['schema']=='lustr-cargo-selection-v2' and tools['identities'][0][0]==tools['identities'][0][1]==launcher,'constructor certificate belongs to another executable')
                require(tools['receipt'].encode()==zipped.read(base+'toolchain/lib/lustr-selection-tools.txt')==zipped.read('lustr-warm-comparison/build-receipts/lustr-selection-tools.txt'),'constructor and measured receipt differ')
                require(len(tools['identities'])==constructor['tool_identities']==9,'constructor tool count differs')
                for identity in tools['identities']:
                    require(len(identity)==12 and all(type(value) is int for value in identity[2:])
                            and stat.S_ISREG(identity[4]) and identity[7]>0,'constructor tool stamp shape differs')
                print(title+'-run constructor: checked actual-launcher execution, certificate and receipt bindings verified; archive-only limitations retained.')
                if campaign['label']=='third': third_checks(output,campaign,report)
                if campaign['label']=='fourth': fourth_checks(output,campaign,report)


if __name__=='__main__': main()
