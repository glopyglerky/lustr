#!/usr/bin/env python3
"""Replay the external-state native Cargo control on a supplied selector helper.

Use --expect stale for failed-source, fresh for the corrected source. The
supplied package must be a matched, trusted source package. This tool constructs
its own executable projection and does not change that package.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--profiler', type=Path, required=True)
parser.add_argument('--package-bin', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--expect', choices=['stale', 'fresh'], required=True)
args = parser.parse_args()
profiler = args.profiler.resolve(strict=True)
base = args.package_bin.resolve(strict=True)
root = args.output.absolute()
root.mkdir(mode=0o700, parents=True, exist_ok=False)
package = root / 'package'
(package / 'bin').mkdir(parents=True)
(package / 'lib').symlink_to(base.parent / 'lib', target_is_directory=True)
for name in ['rustc-full', 'rustc-reference', 'rustdoc']:
    (package / 'bin' / name).symlink_to(base / name)
if args.expect == 'fresh':
    assert (package / 'lib/lustr-selection-tools.txt').is_file(), 'provide the corrected package receipt; missing-receipt refusal is not this control'
external = root / 'external-tool-state'
source = root / 'cargo.c'
source.write_text('#include <unistd.h>\n#include <stdio.h>\n'
                  'int main(int argc, char **argv) { (void)argc; if (access('
                  + json.dumps(str(external)) + ', F_OK) == 0) {'
                  'fputs("external tool state rejects metadata\\n", stderr); return 2; } '
                  'argv[0]=' + json.dumps(str(base / 'cargo')) + '; execv(argv[0], argv); return 127; }\n')
compiler = shutil.which('cc')
assert compiler is not None, 'a C compiler is required for the native-executable control'
built = subprocess.run([compiler, str(source), '-O2', '-o', str(package / 'bin/cargo')], capture_output=True)
(root / 'build.stdout').write_bytes(built.stdout)
(root / 'build.stderr').write_bytes(built.stderr)
assert built.returncode == 0, 'the control executable failed to compile'
project = root / 'project'
(project / 'src').mkdir(parents=True)
(project / 'Cargo.toml').write_text('[package]\nname="external-tool-control"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
(project / 'src/main.rs').write_text('fn main() {}\n')
for name in ['home', 'cargo-home', 'tmp', 'native-cache']:
    (root / name).mkdir()
env = {'PATH':str(package / 'bin') + os.pathsep + '/usr/local/bin:/usr/bin:/bin',
       'HOME':str(root / 'home'), 'CARGO_HOME':str(root / 'cargo-home'), 'TMPDIR':str(root / 'tmp'),
       'CARGO_NET_OFFLINE':'true', 'LUST_CACHE_DIR':str(root / 'native-cache'),
       'LUSTR_SELECTION_DIAGNOSTICS':str(root / 'diagnostics.jsonl')}
command = [str(profiler), 'selection', str(package / 'bin'), 'check', str(project / 'src/main.rs'),
           '--manifest-path', str(project / 'Cargo.toml'), '--offline']
records = []
def invoke(label, uncached=False):
    Path(env['LUSTR_SELECTION_DIAGNOSTICS']).write_text('')
    local = dict(env)
    if uncached:
        local['LUSTR_NO_SELECTION_CACHE'] = '1'
    result = subprocess.run(command, cwd=project, env=local, capture_output=True, timeout=30)
    record = {'label':label, 'exit':result.returncode, 'stdout':result.stdout.decode(), 'stderr':result.stderr.decode(),
              'diagnostics':Path(env['LUSTR_SELECTION_DIAGNOSTICS']).read_text()}
    records.append(record)
    (root / 'results.json').write_text(json.dumps(records, indent=2) + '\n')
    return record
for index in range(3):
    assert invoke('prime-' + str(index))['exit'] == 0
external.touch()
candidate = invoke('after-external-change')
reference = invoke('uncached-after-external-change', uncached=True)
assert reference['exit'] == 2 and reference['stderr'] == 'external tool state rejects metadata\n'
if args.expect == 'stale':
    assert candidate['exit'] == 0 and json.loads(candidate['diagnostics']) == {'selection_cache':'hit'}
else:
    assert candidate['exit'] == 2 and candidate['stderr'] == reference['stderr']
    assert all('"hit"' not in row['diagnostics'] for row in records)
print(json.dumps({'observed':args.expect, 'profiler_sha256':hashlib.sha256(profiler.read_bytes()).hexdigest(),
                  'wrapper_sha256':hashlib.sha256((package / 'bin/cargo').read_bytes()).hexdigest(), 'results':str(root / 'results.json')}))
