#!/usr/bin/env python3
"""Bounded Cargo request-dependency probe; not a cache-policy or timing test."""
import hashlib
import json
from pathlib import Path
import subprocess
import time

ROOT = Path('<DIAGNOSTIC_WORKSPACE>')
OUT = ROOT / 'warm-cargo-tool-closure'
PACKAGE = ROOT / 'warm-selector-package'
FIXTURE = ROOT / 'warm-selector-deferred-spans/fixture'
CARGO = PACKAGE / 'bin/cargo'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inventory():
    return {str(p.relative_to(FIXTURE)): sha(p) for p in sorted(FIXTURE.rglob('*')) if p.is_file()}


def main():
    OUT.mkdir(exist_ok=False)
    (OUT / 'streams').mkdir()
    cases = [
        ('metadata-current-tools', 'metadata', False, []),
        ('metadata-missing-tools', 'metadata', True, []),
        ('pkgid-current-tools', 'pkgid', False, []),
        ('pkgid-missing-tools', 'pkgid', True, []),
        ('metadata-missing-tools-filter-platform-control', 'metadata', True, ['--filter-platform', 'x86_64-unknown-linux-gnu']),
    ]
    fixture_before = inventory()
    planned = []
    for label, operation, missing, extra in cases:
        state = OUT / label
        for part in ['cargo-home', 'target', 'intermediates', 'tmp']:
            (state / part).mkdir(parents=True)
        env = {
            'PATH': str(PACKAGE / 'bin') + ':/usr/local/bin:/usr/bin:/bin',
            'CARGO_HOME': str(state / 'cargo-home'),
            'CARGO_TARGET_DIR': str(state / 'target'),
            'CARGO_BUILD_BUILD_DIR': str(state / 'intermediates'),
            'CARGO_NET_OFFLINE': 'true',
            'CARGO_TERM_COLOR': 'never',
            'CARGO_CACHE_RUSTC_INFO': '0',
            '__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT': '1',
            'TMPDIR': str(state / 'tmp'),
            'LANG': 'C', 'LC_ALL': 'C', 'TZ': 'UTC',
            'RUSTC': str(OUT / 'deliberately-missing-rustc') if missing else str(PACKAGE / 'bin/rustc-full'),
            'RUSTDOC': str(OUT / 'deliberately-missing-rustdoc') if missing else str(PACKAGE / 'bin/rustdoc'),
        }
        argv = [str(CARGO), operation]
        if operation == 'metadata':
            argv += ['--format-version=1']
        argv += ['--manifest-path', str(FIXTURE / 'Cargo.toml')]
        if operation == 'pkgid':
            argv += ['--package', 'slice']
        argv += ['--offline', '--locked', *extra]
        planned.append({'label': label, 'argv': argv, 'cwd': str(FIXTURE), 'environment': env})
    plan = {
        'schema': 'lustr-cargo-tool-closure-probe-v1',
        'created_unix_ns': time.time_ns(),
        'scope': 'One fresh observation per request; no timing or full request-class proof.',
        'cargo': {'path': str(CARGO), 'resolved': str(CARGO.resolve()), 'sha256': sha(CARGO)},
        'fixture_before': fixture_before,
        'commands': planned,
    }
    (OUT / 'plan.json').write_text(json.dumps(plan, indent=2, sort_keys=True) + '\n')
    version = subprocess.run([str(CARGO), '-Vv'], capture_output=True, timeout=20)
    (OUT / 'cargo-version.stdout').write_bytes(version.stdout)
    (OUT / 'cargo-version.stderr').write_bytes(version.stderr)
    records = []
    for case in planned:
        completed = subprocess.run(case['argv'], cwd=case['cwd'], env=case['environment'], capture_output=True, timeout=30)
        record = dict(case, returncode=completed.returncode)
        for name in ['stdout', 'stderr']:
            data = getattr(completed, name)
            destination = OUT / 'streams' / (case['label'] + '.' + name)
            destination.write_bytes(data)
            record[name] = {'path': str(destination.relative_to(OUT)), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
        records.append(record)
    report = {
        'schema': plan['schema'], 'plan_sha256': sha(OUT / 'plan.json'),
        'runner_sha256': sha(Path(__file__)), 'cargo_version_returncode': version.returncode,
        'records': records, 'fixture_unchanged': fixture_before == inventory(),
    }
    (OUT / 'report.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'fixture_unchanged': report['fixture_unchanged'], 'results': [
        {'label': record['label'], 'returncode': record['returncode'],
         'stderr': (OUT / record['stderr']['path']).read_text()[:2000]} for record in records
    ]}, indent=2))


if __name__ == '__main__':
    main()
