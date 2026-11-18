#!/usr/bin/env python3
"""Build the frozen initial failing selector, without modifying a checkout."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--sdk', type=Path, required=True, help='the pinned nightly-2026-11-09 Rust SDK root')
parser.add_argument('--output', type=Path, required=True, help='a new external build directory')
args = parser.parse_args()
snapshot = Path(__file__).resolve().parent / 'failed-source'
identity = json.loads((snapshot / 'identity.json').read_text())
for name, recorded in identity.items():
    source = snapshot / name
    if isinstance(recorded, dict) and source.is_file() and 'sha256' in recorded:
        assert hashlib.sha256(source.read_bytes()).hexdigest() == recorded['sha256'], name
output = args.output.absolute()
output.mkdir(mode=0o700, parents=True, exist_ok=False)
sdk = args.sdk.resolve(strict=True)
manifest = output / 'Cargo.toml'
manifest.write_text('[package]\nname="lustr-selection-review"\nversion="0.1.0"\nedition="2024"\n'
                    '[workspace]\n[dependencies]\nserde_json="=1.0.154"\nsha2="=0.11.1"\n'
                    '[[bin]]\nname="profile-native-preflight"\npath=' + json.dumps(str(snapshot / 'tools/profile-native-preflight.rs')) + '\n')
shutil.copyfile(snapshot / 'PROFILER-Cargo.lock', output / 'Cargo.lock')
env = dict(os.environ, RUSTC=str(sdk / 'bin/rustc'), LUST_BUILD_HASH='dc82dab611c87e7e')
command = [str(sdk / 'bin/cargo'), 'build', '--release', '--locked', '--offline', '--manifest-path', str(manifest), '--jobs', '1']
with (output / 'build.stdout').open('wb') as stdout, (output / 'build.stderr').open('wb') as stderr:
    result = subprocess.run(command, env=env, cwd=output, stdout=stdout, stderr=stderr)
receipt = {'argv':command, 'exit':result.returncode, 'native_protocol_build_identity':'dc82dab611c87e7e',
           'snapshot_identity_sha256':hashlib.sha256((snapshot / 'identity.json').read_bytes()).hexdigest()}
binary = output / 'target/release/profile-native-preflight'
if binary.is_file():
    receipt['binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
(output / 'build-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
raise SystemExit(result.returncode)
