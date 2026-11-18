#!/usr/bin/env python3
"""Verify retained nonregular-input evidence without executing a compiler."""
import hashlib
import json
from pathlib import Path
import tarfile

root = Path(__file__).resolve().parent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


manifest = read(root / 'manifest.json')
assert manifest['schema'] == 'lustr-selection-nonregular-manifest-v1'
expected = {row['path'] for row in manifest['files']}
actual = {path.relative_to(root).as_posix() for path in root.rglob('*')
          if path.is_file() and path.name != 'manifest.json' and '__pycache__' not in path.parts}
assert expected == actual
for row in manifest['files']:
    path = root / row['path']
    assert path.is_file() and not path.is_symlink()
    data = path.read_bytes()
    assert len(data) == row['bytes'] and sha(data) == row['sha256'], row['path']

identity = read(root / 'identity.json')
frozen = read(root / 'failed-source/identity.json')
for path, expected_hash in frozen['files'].items():
    assert sha((root / 'failed-source' / path).read_bytes()) == expected_hash
assert frozen['revision'] == identity['original_helper_source']
assert frozen['files']['src/selection_cache.rs'] == identity['original_selection_cache_sha256']
test_hash = sha((root / 'test-native-selection-cache.py').read_bytes())
assert test_hash == identity['regression_tool_sha256']
original_build = read(root / 'original-helper-build.json')
assert original_build['source']['revision'] == identity['original_helper_source']

for item in identity['sets']:
    directory = root / item['name']
    assert sha((directory / 'results.json').read_bytes()) == item['public_results_sha256']
    report = read(directory / 'results.json')
    archived = report['public_projection']['stream_archive']
    assert sha((directory / archived['file']).read_bytes()) == archived['sha256']
    with tarfile.open(directory / archived['file'], 'r:gz') as archive:
        members = archive.getmembers()
        assert len(members) == archived['entries'] and len({member.name for member in members}) == len(members)
        assert all(member.isfile() and Path(member.name).name == member.name for member in members)
        streams = {member.name: archive.extractfile(member).read() for member in members}
    records = report['records']
    assert len(records) == item['records']
    for record in records:
        for kind in ('stdout', 'stderr'):
            assert sha(streams[record[kind + '_file']]) == record['public_' + kind + '_sha256']
    if item['name'].startswith('original-'):
        assert len(records) == 4 and report['reproduced'] is True
        assert report['helper_sha256'] == original_build['binary_sha256']
        assert report['source_revision'] == identity['original_helper_source']
        assert records[0]['exit'] == records[1]['exit'] == 0
        candidate, reference = records[2:]
        assert candidate['status'] == 'timeout' and candidate['exit'] is None and candidate['timeout_seconds'] == 3
        assert not streams[candidate['stdout_file']] and not streams[candidate['stderr_file']]
        assert reference['status'] == 'exited' and reference['exit'] == (2 if item['name'] == 'original-cargo-fifo' else 0)
        continue
    assert report['status'] == 'completed' and report['success'] is True and report['error'] is None
    assert len(records) == 140 and len(report['assertions']) == item['assertions'] == 265
    assert all(row['passed'] for row in report['assertions'])
    assert not any(row.get('status') == 'timeout' for row in records)
    by_label = {row['label']: row for row in records}
    for reference in records:
        if not reference['label'].endswith('-uncached'):
            continue
        candidate = by_label[reference['label'].removesuffix('-uncached')]
        assert candidate['exit'] == reference['exit']
        assert streams[candidate['stderr_file']] == streams[reference['stderr_file']]
        a, b = json.loads(streams[candidate['stdout_file']]), json.loads(streams[reference['stdout_file']])
        a.pop('elapsed_ns', None)
        b.pop('elapsed_ns', None)
        assert a == b
    for label, expected_status in (('cargo-replaced-by-fifo', 2), ('tool-receipt-replaced-by-fifo', 0)):
        candidate = by_label[label]
        assert candidate['exit'] == expected_status and candidate['diagnostics'] == [{'selection_cache': 'ineligible'}]
    assert by_label['cargo-fifo-removed']['diagnostics'] == [{'selection_cache': 'hit'}]
    receipt = read(directory / 'review-receipt.json')
    build = read(directory / 'helper-build-receipt.json')
    assert receipt['source'] == build['source']
    assert receipt['profiler_sha256'] == build['binary_sha256'] == report['profiler_sha256']
    assert receipt['test_sha256'] == test_hash and receipt['results_sha256'] == item['original_results_sha256']
    source_hash, = [value for path, value in build['source_sha256'].items() if path.endswith('/src/selection_cache.rs')]
    assert sha((directory / 'selection_cache.rs').read_bytes()) == source_hash

unit = read(root / 'descriptor-path-unit/report.json')
assert unit['exit'] == 0 and unit['source_revision'] == identity['descriptor_path']['revision']
for kind in ('stdout', 'stderr'):
    assert sha((root / 'descriptor-path-unit' / kind).read_bytes()) == unit['public_' + kind + '_sha256']
edit = read(root / 'public-edit-gate-review/review-receipt.json')
assert edit['exit'] == 0 and edit['source_revision'] == '34c86339b653e965f43e28b04b1274a7611b4011'
for kind in ('stdout', 'stderr'):
    assert sha((root / 'public-edit-gate-review' / ('tests.' + kind)).read_bytes()) == edit['public_' + kind + '_sha256']
assert b'Ran 28 tests' in (root / 'public-edit-gate-review/tests.stderr').read_bytes()

receipt = read(root / 'encrypted-originals/receipt.json')
assert receipt['recipient_certificate_sha256'] == '69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f'
assert sha((root / 'encrypted-originals/private-evidence.tar.gz.cms').read_bytes()) == receipt['ciphertext_sha256']
print(json.dumps({'status': 'passed', 'public_files': len(expected), 'sets': identity['sets'],
                  'scope': 'Retained evidence integrity only; no new compiler execution.'}, sort_keys=True))
