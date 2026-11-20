#!/usr/bin/env python3
"""Exercise the production selection-tool seed with real Cargo and negative controls.

The supplied profiler must include the production prime-selection-tools phase.
The input package is read only; all seed, tool, and cache changes use a projection.
This gate has no timing thresholds and does not qualify native compilation.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profiler', type=Path, required=True)
    parser.add_argument('--package-bin', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    profiler = args.profiler.resolve(strict=True)
    source = args.package_bin.resolve(strict=True)
    root = args.output.absolute()
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    records, assertions = [], []
    status, error = 'incomplete', None

    def save():
        report = {'schema':'lustr-selection-seed-regression-v1', 'status':status, 'error':error,
                  'success':status == 'completed' and error is None and all(a['passed'] for a in assertions),
                  'scope':'production selection-tool initialization and selector only; no compiler performance or runtime qualification',
                  'profiler_sha256':digest(profiler.read_bytes()), 'records':records, 'assertions':assertions}
        (root / 'results.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')

    def require(condition, description):
        assertions.append({'description':description, 'passed':bool(condition)})
        if not condition:
            raise AssertionError(description)

    def projection(name):
        package = root / name
        (package / 'bin').mkdir(parents=True)
        (package / 'lib').mkdir()
        for tool in ['cargo', 'rustc-full', 'rustc-reference', 'rustdoc']:
            (package / 'bin' / tool).symlink_to(source / tool)
        for path in (source.parent / 'lib').iterdir():
            if path.name != 'lustr-selection-verified':
                (package / 'lib' / path.name).symlink_to(path, target_is_directory=path.is_dir())
        return package

    def run(label, command, env, cwd):
        diagnostic = Path(env['LUSTR_SELECTION_DIAGNOSTICS'])
        diagnostic.write_text('')
        result = subprocess.run([str(part) for part in command], cwd=cwd, env=env, capture_output=True, timeout=30)
        sequence = len(records) + 1
        record = {'sequence':sequence, 'label':label, 'argv':[str(part) for part in command],
                  'cwd':str(cwd), 'exit':result.returncode,
                  'diagnostics':[json.loads(line) for line in diagnostic.read_text().splitlines()]}
        for kind, content in [('stdout', result.stdout), ('stderr', result.stderr)]:
            path = root / f'{sequence:03d}-{label}.{kind}'
            path.write_bytes(content)
            record[kind + '_file'] = path.name
            record[kind + '_sha256'] = digest(content)
        records.append(record)
        save()
        return result, record

    try:
        package = projection('package')
        receipt = package / 'lib/lustr-selection-tools.txt'
        require(receipt.is_file(), 'matched package has its actual tool receipt')
        project = root / 'project'
        (project / 'src').mkdir(parents=True)
        (project / 'Cargo.toml').write_text('[package]\nname="selection-seed-control"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
        (project / 'src/main.rs').write_text('fn main() {}\n')
        for name in ['home', 'cargo-home', 'tmp', 'caches']:
            (root / name).mkdir()
        env = {'PATH':str(package / 'bin') + os.pathsep + '/usr/local/bin:/usr/bin:/bin',
               'HOME':str(root / 'home'), 'CARGO_HOME':str(root / 'cargo-home'), 'TMPDIR':str(root / 'tmp'),
               'CARGO_NET_OFFLINE':'true', 'LUSTR_SELECTION_DIAGNOSTICS':str(root / 'diagnostics.jsonl')}
        setup_env = dict(env, RUSTC=str(package / 'bin/rustc-full'), RUSTDOC=str(package / 'bin/rustdoc'))
        setup, _ = run('cargo-setup', [package / 'bin/cargo', 'metadata', '--format-version=1', '--offline'], setup_env, project)
        require(setup.returncode == 0, 'real Cargo establishes lockfile and package-cache lock')

        def prime(label, selected=package, expected=0):
            result, _ = run(label, [profiler, 'prime-selection-tools', selected / 'bin'], env, project)
            require(result.returncode == expected, label + ' has expected initialization status')

        def pair(label, cache, selected=package, expected=0, event=None):
            local = dict(env, LUST_CACHE_DIR=str(root / 'caches' / cache))
            command = [profiler, 'selection', selected / 'bin', 'check', project / 'src/main.rs',
                       '--manifest-path', project / 'Cargo.toml', '--offline', '--locked']
            candidate, record = run(label, command, local, project)
            reference, _ = run(label + '-uncached', command, dict(local, LUSTR_NO_SELECTION_CACHE='1'), project)
            candidate_value, reference_value = json.loads(candidate.stdout), json.loads(reference.stdout)
            candidate_value.pop('elapsed_ns', None)
            reference_value.pop('elapsed_ns', None)
            require(candidate.returncode == reference.returncode == expected, label + ' preserves exact exit status')
            require(candidate_value == reference_value, label + ' preserves selection or rejection')
            require(candidate.stderr == reference.stderr, label + ' preserves exact stderr')
            if event is not None:
                require(record['diagnostics'] == [{'selection_cache':event}], label + ' uses expected cache state')
            return record

        seed = package / 'lib/lustr-selection-verified'
        prime('prime-matched-package')
        certificates = list(seed.glob('tools-*.json'))
        require(len(certificates) == 1, 'initialization writes exactly one certificate')
        certificate = certificates[0]
        original = certificate.read_bytes()
        original_stamp = certificate.stat()
        require(stat.S_IMODE(seed.stat().st_mode) == 0o700 and seed.stat().st_uid == os.geteuid(), 'seed directory is private and owned')
        require(stat.S_IMODE(certificate.stat().st_mode) == 0o600 and certificate.stat().st_uid == os.geteuid(), 'seed certificate is private and owned')
        prime('prime-idempotent')
        require(certificate.stat().st_mtime_ns == original_stamp.st_mtime_ns and certificate.read_bytes() == original,
                'unchanged initialization preserves existing exact certificate')
        pair('seeded-first-request', 'seeded', event='miss')
        copied = list((root / 'caches/seeded/cargo-selection-v1').glob('tools-*.json'))
        require(len(copied) == 1 and copied[0].read_bytes() == original, 'per-project verification has the exact seeded certificate')
        pair('seeded-retained-request', 'seeded', event='hit')

        for label, contents in [('malformed-seed', b'{'), ('checksum-seed', original.replace(b'"sha256":"', b'"sha256":"bad', 1))]:
            certificate.write_bytes(contents)
            pair(label, label, event='miss')
            pair(label + '-retained', label, event='hit')
            require(certificate.read_bytes() == contents, label + ' is not trusted or rewritten by normal selection')
            certificate.write_bytes(original)

        for label, path in [('shared-seed-file', certificate), ('shared-seed-directory', seed)]:
            mode = stat.S_IMODE(path.stat().st_mode)
            path.chmod(mode | 0o044)
            pair(label, label, event='miss')
            pair(label + '-retained', label, event='hit')
            require(stat.S_IMODE(path.stat().st_mode) == (mode | 0o044), label + ' is left unchanged by fallback')
            path.chmod(mode)

        moved = root / 'relocated-package'
        shutil.copytree(package, moved, symlinks=True)
        pair('relocated-seed', 'relocated', selected=moved, event='miss')
        relocated = list((root / 'caches/relocated/cargo-selection-v1').glob('tools-*.json'))
        require(len(relocated) == 1 and relocated[0].read_bytes() != original, 'relocation obtains a different live-identity certificate')
        require((moved / 'lib/lustr-selection-verified' / certificate.name).read_bytes() == original,
                'relocation leaves old seed as a rejected prior identity')
        pair('relocated-retained', 'relocated', selected=moved, event='hit')

        # Preserve a valid seed while replacing Cargo with a genuine native
        # delegate whose behavior depends on an external file. Old identity
        # alone must never authorize storage or mask its later rejection.
        external = root / 'external-tool-state'
        c_source = root / 'external-cargo.c'
        c_source.write_text('#include <unistd.h>\n#include <stdio.h>\n'
                            'int main(int argc,char **argv){(void)argc;if(access('
                            + json.dumps(str(external)) + ',F_OK)==0){fputs("external tool state rejects metadata\\n",stderr);return 2;}'
                            'argv[0]=' + json.dumps(str(source / 'cargo')) + ';execv(argv[0],argv);return 127;}\n')
        compiler = shutil.which('cc')
        require(compiler is not None, 'native delegate control has a C compiler')
        cargo = package / 'bin/cargo'
        cargo.unlink()
        built, _ = run('build-native-delegate', [compiler, c_source, '-O2', '-o', cargo], env, project)
        require(built.returncode == 0, 'external-state native delegate compiled')
        prime('unknown-native-delegate-refused', expected=2)
        require(list(seed.glob('tools-*.json')) == [certificate], 'refused delegate writes no new seed certificate')
        pair('unknown-native-delegate-first', 'delegate', event='miss')
        pair('unknown-native-delegate-second', 'delegate', event='miss')
        external.touch()
        pair('unknown-native-delegate-external-change', 'delegate', expected=2, event='miss')

        # Receipt mutation must not accept a prior seed, even if only its
        # declared source changes and the actual official tool bytes return.
        cargo.unlink()
        cargo.symlink_to(source / 'cargo')
        receipt_bytes = receipt.read_bytes()
        receipt.unlink()
        receipt.write_bytes(receipt_bytes.replace(b'source=', b'source=invalid-', 1))
        prime('changed-receipt-refused', expected=2)
        pair('changed-receipt-fresh-selection', 'receipt', event='ineligible')
        require(list(seed.glob('tools-*.json')) == [certificate], 'changed receipt writes no seed certificate')
        status = 'completed'
    except BaseException as failure:
        status, error = 'failed', f'{type(failure).__name__}: {failure}'
        raise
    finally:
        save()
    print(json.dumps({'status':status, 'invocations':len(records), 'assertions':len(assertions),
                      'results':str(root / 'results.json')}, sort_keys=True))


if __name__ == '__main__':
    main()
