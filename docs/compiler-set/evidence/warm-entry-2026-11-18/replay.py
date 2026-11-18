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


if __name__=='__main__': main()
