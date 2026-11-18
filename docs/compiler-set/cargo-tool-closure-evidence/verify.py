#!/usr/bin/env python3
"""Replay the bounded missing-tool dependency probe without executing Cargo."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent


def need(value, message):
    if not value:
        raise AssertionError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    identity = json.loads((ROOT / 'identity.json').read_bytes())
    sources = json.loads((ROOT / 'source/provenance.json').read_bytes())
    for name, expected in sources['sha256'].items():
        need(sha(ROOT / 'source' / name) == expected, 'retained primary source changed')
    plan = json.loads((ROOT / 'plan.json').read_bytes())
    report = json.loads((ROOT / 'report.json').read_bytes())
    need(identity['schema'] == 'lustr-cargo-tool-closure-evidence-v1', 'schema differs')
    need(identity['commands'] == len(plan['commands']) == len(report['records']) == 5, 'fixed command count differs')
    need(report['fixture_unchanged'] is True and report['cargo_version_returncode'] == 0, 'fixture or Cargo identity failed')
    need(report['projection']['public_plan_sha256'] == sha(ROOT / 'plan.json'), 'projected plan changed')
    need(report['projection']['public_runner_sha256'] == sha(ROOT / 'runner.executed.py'), 'executed runner changed')
    need(report['projection']['original_report_sha256'] == identity['original_files']['report.json'], 'original report binding differs')
    need(report['plan_sha256'] == identity['original_files']['plan.json'], 'original plan binding differs')
    need(report['runner_sha256'] == identity['original_files']['runner.executed.py'], 'original runner binding differs')
    need(sha(ROOT / 'originals.cms') == identity['encryption']['encrypted_archive_sha256'], 'encrypted originals changed')
    need(identity['encryption']['certificate_der_sha256'] == '69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f', 'evidence recipient differs')
    version = (ROOT / 'cargo-version.stdout').read_text()
    need('commit-hash: ' + identity['cargo_revision'] in version.splitlines() and (ROOT / 'cargo-version.stderr').read_bytes() == b'', 'Cargo version differs')
    need(plan['cargo']['sha256'] == identity['cargo_sha256'] == 'da29c2a9ff7f85bda5587e9c974e05c9e2a7196377e54466f6e9d6f9e00b2530', 'Cargo binary binding differs')
    fixture = ROOT.parent / 'warm-entry-selector-spans-evidence/fixture'
    need({str(p.relative_to(fixture)): sha(p) for p in fixture.rglob('*') if p.is_file()} == plan['fixture_before'], 'public source fixture differs')
    for name in ('Cargo.toml', 'Cargo.lock'):
        need(sha(ROOT / 'fixture' / name) == plan['fixture_before'][name], 'fixture manifest/lockfile differs')
    labels = ['metadata-current-tools', 'metadata-missing-tools', 'pkgid-current-tools', 'pkgid-missing-tools', 'metadata-missing-tools-filter-platform-control']
    need([r['label'] for r in report['records']] == labels, 'request schedule differs')
    expected_status = [0, 101, 0, 0, 101]
    paths = {key: set() for key in ('CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_BUILD_DIR', 'TMPDIR')}
    outputs = {}
    for i, record in enumerate(report['records']):
        need({k: record[k] for k in ('label', 'argv', 'cwd', 'environment')} == plan['commands'][i], 'executed request differs from plan')
        need(record['returncode'] == expected_status[i], 'request result differs')
        argv = record['argv']
        expected = [plan['cargo']['path'], 'metadata' if i in (0, 1, 4) else 'pkgid']
        if i in (0, 1, 4):
            expected += ['--format-version=1']
        expected += ['--manifest-path', record['cwd'] + '/Cargo.toml']
        if i in (2, 3):
            expected += ['--package', 'slice']
        expected += ['--offline', '--locked']
        if i == 4:
            expected += ['--filter-platform', 'x86_64-unknown-linux-gnu']
        need(argv == expected, 'Cargo argv differs')
        env = record['environment']
        need(env['CARGO_CACHE_RUSTC_INFO'] == '0' and env['CARGO_NET_OFFLINE'] == 'true', 'fresh/offline controls differ')
        for key in paths:
            need(env[key] not in paths[key], 'command state reused')
            paths[key].add(env[key])
        missing = i in (1, 3, 4)
        for key, suffix in (('RUSTC', 'rustc'), ('RUSTDOC', 'rustdoc')):
            need(env[key].endswith('/deliberately-missing-' + suffix) if missing else '/warm-selector-package/bin/' in env[key], 'tool selection differs')
        streams = {}
        for kind in ('stdout', 'stderr'):
            item = record[kind]
            path = Path(item['path'])
            need(not path.is_absolute() and '..' not in path.parts, 'stream path escaped evidence')
            data = (ROOT / path).read_bytes()
            need(sha(ROOT / path) == item['public_sha256'] and len(data) == item['public_bytes'], 'public stream changed')
            need(item['sha256'] == identity['original_files'][item['path']], 'original stream binding differs')
            streams[kind] = data
        if i in (1, 4):
            need(streams['stdout'] == b'' and b'deliberately-missing-rustc -vV' in streams['stderr'] and b'No such file or directory (os error 2)' in streams['stderr'], 'missing-rustc counterexample lost')
        else:
            need(streams['stdout'] and streams['stderr'] == b'', 'successful command streams differ')
        outputs[i] = streams['stdout']
    metadata = json.loads(outputs[0])
    need(len(metadata['packages']) == 1 and metadata['packages'][0]['name'] == 'slice' and metadata['packages'][0]['dependencies'] == [], 'metadata fixture differs')
    need(outputs[2] == outputs[3] == (metadata['packages'][0]['id'] + '\n').encode(), 'pkgid countercheck differs')
    print('Verified five fixed Cargo requests: metadata needs rustc; pkgid countercheck succeeds; fixture unchanged.')


if __name__ == '__main__':
    main()
