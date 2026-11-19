#!/usr/bin/env python3
"""Acquire the exact upstream macOS sanitizer package without changing the host.

The archive and its attestation bundle are pinned in a reviewed adjacent file.
Verification precedes extraction and every invocation of downloaded programs.
The real sanitizer capability checks remain separate, mandatory workflow gates.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import urllib.request


PIN = Path(__file__).with_name('macos-sanitizer-toolchain.json')


def file_hash(path):
    checksum = hashlib.sha256()
    with path.open('rb') as stream:
        while block := stream.read(1024 * 1024):
            checksum.update(block)
    return checksum.hexdigest()


def verify_file(path, expected):
    if not path.is_file() or path.is_symlink():
        raise ValueError('package input must be a regular file')
    if path.stat().st_size != expected['bytes']:
        raise ValueError('package input size differs from its pin')
    if file_hash(path) != expected['sha256']:
        raise ValueError('package input SHA-256 differs from its pin')


def download(directory, expected):
    path = directory / expected['name']
    if path.exists() or path.is_symlink():
        verify_file(path, expected)
        return path
    partial = path.with_name(path.name + '.partial')
    length = 0
    with urllib.request.urlopen(expected['url'], timeout=90) as source, partial.open('xb') as output:
        while block := source.read(1024 * 1024):
            length += len(block)
            if length > expected['bytes']:
                raise ValueError('download exceeds pinned input size')
            output.write(block)
    verify_file(partial, expected)
    partial.rename(path)
    return path


def attestation_command(archive, bundle, pin):
    return ['gh', 'attestation', 'verify', str(archive),
            '--bundle', str(bundle), '--repo', pin['repository'],
            '--signer-digest', pin['source_commit'],
            '--source-ref', pin['source_ref'], '--source-digest', pin['source_commit'],
            '--cert-identity', pin['certificate_identity'],
            '--cert-oidc-issuer', 'https://token.actions.githubusercontent.com',
            '--format', 'json']


def run_recorded(argv, directory, name, timeout):
    if any((directory / (name + suffix)).exists() for suffix in ('.stdout', '.stderr', '.status.json')):
        raise ValueError('command evidence already exists; use a new owned directory')

    def retain(stdout, stderr, returncode, timed_out):
        status = {'exit': returncode if returncode is not None and returncode >= 0 else None,
                  'signal': -returncode if returncode is not None and returncode < 0 else None,
                  'timed_out': timed_out, 'timeout_seconds': timeout}
        records = (('.stdout', stdout or b''), ('.stderr', stderr or b''),
                   ('.status.json', (json.dumps(status, sort_keys=True) + '\n').encode()))
        for suffix, content in records:
            with (directory / (name + suffix)).open('xb') as output:
                output.write(content)

    try:
        result = subprocess.run(argv, capture_output=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired as error:
        retain(error.stdout, error.stderr, None, True)
        raise
    retain(result.stdout, result.stderr, result.returncode, False)
    if result.returncode != 0:
        raise RuntimeError(name + ' failed; original output retained')
    return result.stdout


def extract(archive, destination):
    destination.mkdir()
    # The complete package has already passed its pinned digest and signed
    # provenance checks. Extract only into the newly owned empty directory.
    # zstd is a host decompressor, not part of the selected compiler/runtime.
    # LLVM's official zstd archive uses a 1 GiB window. Keep its decoder budget
    # explicit and bounded instead of depending on zstd's lower host default.
    with subprocess.Popen(['zstd', '--decompress', '--memory=1024MB', '--stdout', str(archive)],
                          stdout=subprocess.PIPE) as stream:
        try:
            result = subprocess.run(['tar', '-xf', '-', '-C', str(destination)],
                                    stdin=stream.stdout, timeout=600, check=False)
        finally:
            stream.stdout.close()
        decompressor_exit = stream.wait(timeout=30)
    if result.returncode != 0 or decompressor_exit != 0:
        raise RuntimeError('verified archive extraction failed')


def select_sdk(directory):
    # The upstream compiler needs a Darwin sysroot even when Apple's driver
    # would find one implicitly. Honor an explicit SDKROOT and the developer
    # selection inherited by xcrun; never change the host's active developer.
    sdk_root = os.environ.get('SDKROOT')
    selected_by = 'SDKROOT' if sdk_root else 'xcrun'
    if not sdk_root:
        sdk_root = run_recorded(['xcrun', '--sdk', 'macosx', '--show-sdk-path'],
                                directory, 'macos-sdk-path', 15).decode().strip()
    sdk = Path(sdk_root)
    if (not sdk.is_absolute() or sdk == Path('/') or '\n' in sdk_root or '\r' in sdk_root
            or not sdk.is_dir() or not (sdk / 'usr/include/stdlib.h').is_file()):
        raise ValueError('SDKROOT must name an absolute existing macOS SDK with stdlib.h')
    settings = {name: file_hash(sdk / name) for name in ('SDKSettings.json', 'SDKSettings.plist')
                if (sdk / name).is_file()}
    return {'path': sdk_root, 'selected_by': selected_by,
            'developer_dir': os.environ.get('DEVELOPER_DIR'),
            'settings_sha256': settings,
            'stdlib_header_sha256': file_hash(sdk / 'usr/include/stdlib.h')}


def install(directory, github_env=None):
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise ValueError('this pinned package requires macOS arm64')
    if not directory.is_absolute() or directory.is_symlink():
        raise ValueError('an absolute owned package directory is required')
    directory.mkdir(parents=True, exist_ok=True)
    if (directory / 'package').exists():
        raise ValueError('package destination already exists; preserve prior evidence')
    pin = json.loads(PIN.read_bytes())
    if shutil.disk_usage(directory).free < pin['archive']['bytes'] + 8 * 1024**3:
        raise ValueError('package destination has insufficient free space')
    archive = download(directory, pin['archive'])
    bundle = download(directory, pin['attestation'])
    run_recorded(attestation_command(archive, bundle, pin), directory, 'attestation-verification', 120)
    extract(archive, directory / 'package')
    package = directory / 'package' / pin['root_directory']
    compiler, symbolizer = package / 'bin/clang', package / 'bin/llvm-symbolizer'
    for executable in (compiler, symbolizer):
        if not executable.is_file() or not os.access(executable, os.X_OK):
            raise ValueError('pinned package is missing an executable')
    version = run_recorded([str(compiler), '-dumpversion'], directory, 'clang-version', 15).decode().strip()
    if version != pin['version']:
        raise ValueError('extracted compiler version differs from its pin')
    resource = Path(run_recorded([str(compiler), '-print-resource-dir'], directory, 'clang-resource', 15).decode().strip())
    if not resource.resolve().is_relative_to(package.resolve()):
        raise ValueError('compiler resource directory escaped the pinned package')
    runtime_dir = resource / 'lib/darwin'
    if not (runtime_dir / 'libclang_rt.asan_osx_dynamic.dylib').is_file():
        raise ValueError('pinned package is missing the macOS address sanitizer runtime')
    runtimes = sorted(runtime_dir.glob('libclang_rt.*'))
    sdk = select_sdk(directory)
    receipt = {'schema': 1, 'pin': pin, 'platform': 'macOS arm64', 'clang_version': version,
               'attestation_verification': 'passed', 'sanitizer_capability_status': 'not-run-by-installer',
               'compiler_sha256': file_hash(compiler), 'symbolizer_sha256': file_hash(symbolizer),
               'sdk': sdk,
               'runtime_sha256': {path.relative_to(package).as_posix(): file_hash(path) for path in runtimes if path.is_file()}}
    (directory / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    environment = {'SAN_CC': str(compiler), 'ASAN_SYMBOLIZER_PATH': str(symbolizer), 'SDKROOT': sdk['path']}
    (directory / 'environment.json').write_text(json.dumps(environment, indent=2, sort_keys=True) + '\n')
    if github_env is not None:
        with github_env.open('a') as output:
            for key, value in environment.items():
                if '\n' in value or '\r' in value:
                    raise ValueError('package path contains a line break')
                output.write(key + '=' + value + '\n')
    print('Pinned LLVM ' + version + ' package verified; native sanitizer capabilities remain to be tested.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--github-env', type=Path)
    args = parser.parse_args()
    try:
        install(args.directory, args.github_env)
        return 0
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print('Pinned sanitizer setup failed: ' + str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
