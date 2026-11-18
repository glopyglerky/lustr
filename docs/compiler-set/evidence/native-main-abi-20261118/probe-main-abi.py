import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

p = argparse.ArgumentParser()
p.add_argument('--source', type=Path, required=True)
p.add_argument('--lust', type=Path, required=True)
p.add_argument('--rustc', type=Path, required=True)
p.add_argument('--cc', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--release-c', action='store_true')
p.add_argument('--unwind-tables', action='store_true')
p.add_argument('fixtures', nargs='+')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

records = []
def run(label, argv, cwd, env=None, timeout=60):
    timed_out = False
    try:
        r = subprocess.run([str(x) for x in argv], cwd=cwd, env=env,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        timed_out = True
        r = subprocess.CompletedProcess(argv, 124, error.stdout or b'', error.stderr or b'')
    (cwd / (label + '.stdout')).write_bytes(r.stdout)
    (cwd / (label + '.stderr')).write_bytes(r.stderr)
    records.append({'label': label, 'cwd': str(cwd), 'argv': [str(x) for x in argv],
                    'exit': r.returncode, 'timed_out': timed_out, 'stdout_sha256': hashlib.sha256(r.stdout).hexdigest(),
                    'stderr_sha256': hashlib.sha256(r.stderr).hexdigest()})
    (a.output/'commands.json').write_text(json.dumps(records, indent=2) + '\n')
    return r

identity = {'source_head': subprocess.check_output(['git', '-C', str(a.source), 'rev-parse', 'HEAD'], text=True).strip(),
            'native_base_tree': subprocess.check_output(['git', '-C', str(a.source), 'rev-parse', 'HEAD:compiler/native'], text=True).strip(),
            'cgen_sha256': digest(a.source/'compiler/native/src/cgen.rs'),
            'lust_sha256': digest(a.lust), 'lust_core_sha256': digest(a.lust.with_name('lust-core')),
            'rustc_sha256': digest(a.rustc), 'cc_sha256': digest(a.cc),
            'release_c': a.release_c, 'unwind_tables': a.unwind_tables,
            'source_diff': subprocess.check_output(['git', '-C', str(a.source), 'diff', '--', 'compiler/native'], text=True)}
for label, command in [('lust-version', [a.lust, '--version']), ('rustc-version', [a.rustc, '-vV']), ('cc-version', [a.cc, '--version'])]:
    run(label, command, a.output)

flags = ['-O1', '-w', '-Werror=incompatible-pointer-types', '-fno-sanitize-recover=undefined', '-fsanitize=address,undefined']
results = []
for fixture in a.fixtures:
    src = a.source/'compiler/native'/fixture
    case = a.output/src.stem
    case.mkdir()
    copied = case/'case.rs'
    shutil.copyfile(src, copied)
    env = dict(os.environ, LUST_NO_DAEMON='1', LUST_CACHE_DIR=str(case/'cache'), ASAN_OPTIONS='detect_leaks=1:allocator_may_return_null=1')
    for key in ['UBSAN_OPTIONS', 'LSAN_OPTIONS', 'LUST_RELEASE_C', 'LUST_UNWIND_TABLES']:
        env.pop(key, None)
    if a.release_c: env['LUST_RELEASE_C'] = '1'
    if a.unwind_tables: env['LUST_UNWIND_TABLES'] = '1'
    oracle = run('rustc-build', [a.rustc, '--edition=2024', '-A', 'warnings', 'case.rs', '-o', 'oracle'], case, env)
    native = run('native-build', [a.lust, 'build', 'case.rs', '-o', 'native', '--emit-c', 'generated'], case, env)
    row = {'fixture': fixture, 'fixture_sha256': digest(src), 'rustc_build_exit': oracle.returncode, 'native_build_exit': native.returncode}
    if oracle.returncode or native.returncode:
        row['passed'] = False
        results.append(row)
        continue
    expected = run('rustc-run', ['./oracle'], case, env, timeout=10)
    actual = run('native-run', ['./native'], case, env, timeout=10)
    row['oracle_exit'] = expected.returncode
    row['native_exit'] = actual.returncode
    row['native_matches'] = actual.returncode == expected.returncode and actual.stdout == expected.stdout and actual.returncode != 124
    objects = []
    compile_ok = True
    for i, unit in enumerate(sorted(case.glob('generated.*.c'))):
        obj = case/(unit.name + '.o')
        result = run('san-unit-' + str(i), [a.cc, *flags, '-c', unit.name, '-o', obj.name], case, env)
        compile_ok = compile_ok and result.returncode == 0
        objects.append(obj.name)
    row['sanitized_compile_ok'] = compile_ok
    if compile_ok:
        linked = run('san-link', [a.cc, *flags, *objects, '-lm', '-o', 'sanitized'], case, env)
        row['sanitized_link_exit'] = linked.returncode
        if linked.returncode == 0:
            san = run('san-run', ['./sanitized'], case, env, timeout=10)
            row['sanitized_exit'] = san.returncode
            row['sanitizer_diagnostic'] = 'Sanitizer' in san.stderr.decode(errors='replace') or 'runtime error:' in san.stderr.decode(errors='replace')
            row['sanitized_matches'] = san.returncode == expected.returncode and san.stdout == expected.stdout
    row['passed'] = row.get('native_matches', False) and row.get('sanitized_matches', False) and not row.get('sanitizer_diagnostic', True)
    results.append(row)
    print(json.dumps(row), flush=True)

report = {'identity': identity, 'flags': flags, 'commands': records, 'results': results,
          'passed': all(r['passed'] for r in results)}
(a.output/'report.json').write_text(json.dumps(report, indent=2) + '\n')
sys.exit(0 if report['passed'] else 1)
