#!/usr/bin/env python3
"""Check the follow-up review archive without executing a compiler."""
import hashlib
import json
from pathlib import Path
import tarfile

root = Path(__file__).resolve().parent
def sha(data): return hashlib.sha256(data).hexdigest()
def read(path): return json.loads(path.read_bytes())
manifest = read(root / 'manifest.json')
assert manifest['schema'] == 'lustr-selection-review-followup-manifest-v1'
expected = {row['path'] for row in manifest['files']}
actual = {p.relative_to(root).as_posix() for p in root.rglob('*') if p.is_file() and p.name != 'manifest.json' and '__pycache__' not in p.parts}
assert expected == actual
for row in manifest['files']:
    path = root / row['path']
    assert path.is_file() and not path.is_symlink()
    data = path.read_bytes()
    assert len(data) == row['bytes'] and sha(data) == row['sha256'], row['path']
identity = read(root / 'identity.json')
for item in identity['sets']:
    directory = root / item['name']
    assert sha((directory / 'results.json').read_bytes()) == item['public_results_sha256']
    report = read(directory / 'results.json')
    archived = report['public_projection']['stream_archive']
    assert sha((directory / archived['file']).read_bytes()) == archived['sha256']
    with tarfile.open(directory / archived['file'], 'r:gz') as archive:
        members = archive.getmembers()
        assert len(members) == archived['entries'] and len({m.name for m in members}) == len(members)
        assert all(m.isfile() and Path(m.name).name == m.name for m in members)
        streams = {m.name:archive.extractfile(m).read() for m in members}
    records = report['records']
    assert len(records) == item['records']
    for record in records:
        for kind in ('stdout', 'stderr'):
            assert sha(streams[record[kind + '_file']]) == record['public_' + kind + '_sha256']
    if item['name'] == 'credentials':
        assert len(records) == 4 and all(row['exit'] == 0 for row in records)
        baseline = records[1]
        for row in records[2:]:
            assert row['status_matches_baseline'] and row['stdout_matches_baseline'] and row['stderr_matches_baseline']
            assert row['stdout_sha256'] == baseline['stdout_sha256'] and row['stderr_sha256'] == baseline['stderr_sha256']
    else:
        assert report['status'] == 'completed' and report['success'] is True and report['error'] is None
        assert len(report['assertions']) == item['assertions'] and all(row['passed'] for row in report['assertions'])
        by_label = {row['label']:row for row in records}
        for row in records:
            if not row['label'].endswith('-uncached'): continue
            candidate = by_label[row['label'].removesuffix('-uncached')]
            assert candidate['exit'] == row['exit']
            assert streams[candidate['stderr_file']] == streams[row['stderr_file']]
            a, b = json.loads(streams[candidate['stdout_file']]), json.loads(streams[row['stdout_file']])
            a.pop('elapsed_ns', None); b.pop('elapsed_ns', None)
            assert a == b
        receipt = read(directory / 'review-receipt.json')
        build = read(directory / 'helper-build-receipt.json')
        assert receipt['source'] == build['source'] and receipt['helper_sha256'] == build['binary_sha256'] == report['profiler_sha256']
        assert sha((root / Path(receipt['test_script']).name).read_bytes()) == receipt['test_script_sha256']
receipt = read(root / 'encrypted-originals/receipt.json')
assert receipt['recipient_certificate_sha256'] == '69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f'
assert sha((root / 'encrypted-originals/private-evidence.tar.gz.cms').read_bytes()) == receipt['ciphertext_sha256']
print(json.dumps({'status':'passed', 'public_files':len(expected), 'sets':identity['sets'], 'scope':'retained evidence integrity; no new compiler execution'}, sort_keys=True))
