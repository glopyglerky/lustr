#!/usr/bin/env python3
"""Verify the retained public review evidence; does not execute a compiler."""
import hashlib
import json
from pathlib import Path
import tarfile

root = Path(__file__).resolve().parent

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_json(path):
    return json.loads(path.read_bytes())

manifest = read_json(root / 'manifest.json')
assert manifest['schema'] == 'lustr-selection-review-public-manifest-v1'
expected = {row['path'] for row in manifest['files']}
actual = {path.relative_to(root).as_posix() for path in root.rglob('*') if path.is_file() and path.name != 'manifest.json' and '__pycache__' not in path.parts}
assert expected == actual, 'public file inventory differs'
for row in manifest['files']:
    path = root / row['path']
    assert not path.is_symlink() and path.is_file(), row['path']
    data = path.read_bytes()
    assert len(data) == row['bytes'] and digest(data) == row['sha256'], row['path']

identity = read_json(root / 'identity.json')
assert identity['local_tested_revision'] == '53057e72f1853bf2ec005e5a798975b4f951044f'
assert identity['published_revision'] == '18fe6c3408e6d581961bcf7f2721f72ca74cfbfa'
assert identity['identical_tree'] == 'b568e8bf4d9970418b7c1f4ef2984ce391ffa5f5'
failed = root / 'failed-source'
frozen_identity = read_json(failed / 'identity.json')
assert frozen_identity == identity['first_failure_source_identity']
for name, record in frozen_identity.items():
    if isinstance(record, dict) and 'sha256' in record and name != 'profiler':
        assert digest((failed / name).read_bytes()) == record['sha256'], name
        assert record['mtime_ns'] <= frozen_identity['profiler']['mtime_ns'], name

initial = read_json(root / 'outcomes/first-elf-failure.json')
assert [row['exit'] for row in initial] == [0, 0, 0, 0, 2]
assert json.loads(initial[-2]['diagnostics']) == {'selection_cache':'hit'}
assert initial[-1]['stderr'] == 'external tool state rejects metadata\n'

verified_sets = []
for index in identity['public_result_sets']:
    directory = root / 'outcomes' / index['name']
    assert digest((directory / 'results.json').read_bytes()) == index['public_results_sha256']
    report = read_json(directory / 'results.json')
    archived = report['public_projection']['stream_archive']
    assert digest((directory / archived['file']).read_bytes()) == archived['sha256']
    with tarfile.open(directory / archived['file'], 'r:gz') as archive:
        members = archive.getmembers()
        assert len(members) == archived['entries']
        assert all(m.isfile() and Path(m.name).name == m.name for m in members)
        assert len({m.name for m in members}) == len(members)
        streams = {m.name:archive.extractfile(m).read() for m in members}
    records = report['records']
    for record in records:
        for kind in ('stdout', 'stderr'):
            assert digest(streams[record[kind + '_file']]) == record['public_' + kind + '_sha256']
    if index['name'] == 'reporting-negative-control':
        assert report['status'] == 'failed' and report['success'] is False
        assert report['error'].startswith('JSONDecodeError:')
        continue
    assert report['success'] is True and len(records) == 132 and len(report['assertions']) == 250
    assert all(row['passed'] is True for row in report['assertions'])
    for candidate, reference in zip(records[::2], records[1::2]):
        assert candidate['uncached'] is False and reference['uncached'] is True
        assert reference['label'] == candidate['label'] + '-uncached'
        assert candidate['exit'] == reference['exit'] and candidate['selection'] == reference['selection']
        assert streams[candidate['stderr_file']] == streams[reference['stderr_file']]
    external = [r for r in records if r['label'] == 'unknown-native-cargo-external-change']
    assert len(external) == 1 and external[0]['exit'] == 2
    assert all(d.get('selection_cache') != 'hit' for r in records if r['label'].startswith('unknown-native-cargo') for d in r['diagnostics'])
    if index['name'] == 'reporting-fix':
        assert report['status'] == 'completed' and report['error'] is None
    verified_sets.append(index['name'])

workflow = read_json(root / 'outcomes/workflow-helper-build.json')
assert workflow['status'] == 'completed'
assert workflow['source'] == {'revision':identity['local_tested_revision'], 'tree':identity['identical_tree']}
assert workflow['binary_sha256'] == read_json(root / 'outcomes/workflow-replay/results.json')['profiler_sha256']
assert all(command['exit'] == 0 for command in workflow['commands'])
receipt = read_json(root / 'encrypted-originals/receipt.json')
assert receipt['schema'] == 'lustr-encrypted-evidence-receipt-v1'
assert receipt['recipient_certificate_sha256'] == '69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f'
ciphertext = (root / 'encrypted-originals/private-evidence.tar.gz.cms').read_bytes()
assert digest(ciphertext) == receipt['ciphertext_sha256']
print(json.dumps({'status':'passed', 'public_files':len(expected), 'complete_result_sets':verified_sets,
                  'invocations_per_complete_set':132, 'assertions_per_complete_set':250,
                  'scope':'retained selector evidence integrity; no new compiler executions'}, sort_keys=True))
