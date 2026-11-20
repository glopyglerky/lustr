#!/usr/bin/env python3
"""Replay the fixed deferred-span evidence without running a compiler."""
import hashlib
import io
import json
from pathlib import Path
import statistics
import tarfile

root=Path(__file__).resolve().parent
sha=lambda data:hashlib.sha256(data).hexdigest()
load=lambda name:json.loads((root/name).read_bytes())
manifest=load('MANIFEST.json')
assert {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}==set(manifest['files'])|{'MANIFEST.json'}
for name,expected in manifest['files'].items():
    data=(root/name).read_bytes();assert sha(data)==expected['sha256'] and len(data)==expected['bytes'],name
identity=load('identity.json');plan=load('predeclared-plan.json');report=load('report.json')
assert identity['commands']==111 and identity['observations']==96 and identity['span_count']==35
assert plan['source']==identity['profile'] and report['status']=='completed' and report['error'] is None
assert report['plan_sha256']==identity['original_plan_sha256']
assert report['public_projection']['original_report_sha256']==identity['original_report_sha256']
assert report['public_projection']['plan_sha256']==sha((root/'predeclared-plan.json').read_bytes())
assert plan['runner_sha256']==sha((root/'runner.py').read_bytes())
for path,expected in plan['source']['files'].items():assert sha((root/'source'/path).read_bytes())==expected,path
for label in ('profiled','original'):
    build=load(label+'-receipt.json')
    assert build['status']=='completed' and build['binary_sha256']==identity['helpers'][label]==plan['helpers'][label]['sha256']
    expected=identity['profile'] if label=='profiled' else identity['product']
    assert build['source']=={k:expected[k] for k in ('revision','tree')}
    prefix='source' if label=='profiled' else 'original-source'
    for path,expected in build['source_sha256'].items():assert sha((root/prefix/path).read_bytes())==expected,path
for path,expected in plan['fixture_files'].items():assert sha((root/'fixture'/path).read_bytes())==expected,path
assert load('package-receipt.json')['selection_receipt_sha256']==sha((root/'selection-tools.txt').read_bytes())
expected_schedule=[{'phase':phase,'index':i} for phase,n in [('prime',3),('excluded',8),('measured',96)] for i in range(n)]
assert plan['schedule']==expected_schedule
assert not any(name in plan['environment'] for name in ('LUSTR_SELECTION_DIAGNOSTICS','LUST_STATS','LUST_TIMINGS'))
assert '--package' not in plan['native_args'] and plan['native_args'][0]=='check'
assert sha((root/'streams.tar.gz').read_bytes())==report['public_projection']['archive_sha256']
streams={}
with tarfile.open(fileobj=io.BytesIO((root/'streams.tar.gz').read_bytes()),mode='r:gz') as tar:
    for member in tar.getmembers():
        assert member.isfile() and not member.name.startswith('/') and '..' not in Path(member.name).parts and member.name not in streams
        streams[member.name]=tar.extractfile(member).read()
records=report['records'];assert len(records)==111 and [r['sequence'] for r in records]==list(range(111))
expected_labels=['profiled-prime-tools','original-prime-tools','original-before']+[row['phase']+'-'+str(row['index']) for row in expected_schedule]+['original-after']
assert [r['label'] for r in records]==expected_labels
used=set();decoded={}
for record in records:
    assert record['exit']==0 and record['timeout'] is False
    for kind in ('stdout','stderr'):
        item=record[kind];data=streams[item['file']];used.add(item['file'])
        assert sha(data)==item['public_sha256'] and len(data)==item['public_bytes']
        if kind=='stderr':assert data==b''
    raw=streams[record['stdout']['file']]
    if record['sequence']<2:assert raw==b''
    else:decoded[record['sequence']]=json.loads(raw)
assert used==set(streams)
semantic=lambda value:{k:v for k,v in value.items() if k not in ('elapsed_ns','spans','cache_state')}
oracle=semantic(decoded[2]);assert oracle['success'] and isinstance(oracle['selection'],dict)
assert semantic(decoded[110])==oracle
for sequence in range(3,110):
    value=decoded[sequence];assert semantic(value)==oracle
    assert len(value['spans'])==35 and value['spans']['full_select']['calls']==1
    assert value['elapsed_ns']>=value['spans']['full_select']['ns']
    if sequence>3:assert value['cache_state']=='hit'
frames=report['frames'];assert len(frames)==96
for i,frame in enumerate(frames):
    value=decoded[i+14]
    assert frame['index']==i and frame['command']==i+14
    assert frame['elapsed_ns']==value['elapsed_ns'] and frame['spans']==value['spans'] and frame['cache_state']==value['cache_state']=='hit'
    residual=value['spans']['full_select']['ns']-sum(value['spans'][k]['ns'] for k in ('capture','load','diagnostic'))
    assert residual>=0 and residual==frame['unclassified_top_level_ns']
def distribution(values):
    ordered=sorted(values)
    return {'minimum_ns':ordered[0],'median_ns':statistics.median(ordered),'p95_ns':ordered[(len(ordered)*95+99)//100-1],'maximum_ns':ordered[-1]}
summary={'outer_elapsed':distribution([f['elapsed_ns'] for f in frames]),'unclassified_top_level':distribution([f['unclassified_top_level_ns'] for f in frames]),'spans':{name:{**distribution([f['spans'][name]['ns'] for f in frames]),'calls':sorted(set(f['spans'][name]['calls'] for f in frames))} for name in sorted(frames[0]['spans'])}}
assert summary==report['summary']
print('Verified exact selector profile/source/helper bindings, 111 commands, 96 retained frames, complete Selection agreement and all inclusive span distributions.')
