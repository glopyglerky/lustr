#!/usr/bin/env python3
"""Verify downloaded CI records without executing any compiler or archived file."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import zipfile


EXPECTED = {
    'c821': {
        'head': 'c8218b7fc0f657314c9e9c59775091b53508c6b0',
        'checkout': 'ec8f89f98e2701f7189d4310e54a1501cdd81f34',
        'run': '44562165627',
        'supplemental_sha256': '93c83be3ef1ccef020f3330868aad7f7ab4d3415d550763ff18d24ac3a9cce36',
        'supplemental_bytes': 13585105,
        'linux_sha256': '4d163abc595a4e4f0c8c094bab4b8f2bc48a2aa901653e5eb9280b1806312cdc',
    },
    '7c': {
        'head': '7c3d095fa8971a4c38aa0edf62e8fcfd7a51589a',
        'checkout': '66963315e9736ce9f151ce8ccac248d35a01d048',
        'run': '44553094694',
        'supplemental_sha256': '4d7965327d6466f092c704992abca9baf950b8548d5e115225d5d2b5c4b4988a',
        'supplemental_bytes': 13603715,
        'linux_sha256': '8eb7f1aee44fcd3a9f4ac5f50c128666f179f83d5a6399dc8e8050fe25b3d482',
    },
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def digest_file(path):
    with path.open('rb') as source:
        return digest_stream(source)[0]


def digest_stream(source):
    h, size = hashlib.sha256(), 0
    while block := source.read(1024 * 1024):
        h.update(block)
        size += len(block)
    return h.hexdigest(), size


def safe_name(name):
    path = PurePosixPath(name)
    if not name or path.is_absolute() or '..' in path.parts:
        raise ValueError('unsafe archive member: ' + name)
    return path


def supplemental_input(root, label):
    expected = EXPECTED[label]
    path = root / (label + '-supplemental-original.zip')
    if path.is_file():
        assert path.stat().st_size == expected['supplemental_bytes']
        return path, lambda: digest_file(path), 'original-zip'
    assert label == 'c821', 'only the published current ZIP uses ordered parts'
    manifest_path = root / 'archive-parts.json'
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes)
    assert manifest['schema'] == 'lustr-exact-archive-parts-v1'
    assert manifest['original_path'] == path.name
    assert manifest['original_bytes'] == expected['supplemental_bytes']
    assert manifest['original_sha256'] == expected['supplemental_sha256']
    parts = manifest['parts']
    assert len(parts) == len({part['path'] for part in parts}) == 2

    def reassemble():
        assert manifest_path.read_bytes() == manifest_bytes
        chunks = []
        for part in parts:
            name = safe_name(part['path'])
            assert len(name.parts) == 1 and name.name.startswith(path.name + '.part-')
            data = (root / name).read_bytes()
            assert len(data) == part['bytes'] and 0 < len(data) < 8 * 1024 * 1024
            assert digest(data) == part['sha256']
            chunks.append(data)
        joined = b''.join(chunks)
        assert len(joined) == expected['supplemental_bytes']
        assert digest(joined) == expected['supplemental_sha256']
        return joined

    return io.BytesIO(reassemble()), lambda: digest(reassemble()), 'ordered-exact-byte-parts'


class HashedReader:
    def __init__(self, source):
        self.source, self.h, self.size = source, hashlib.sha256(), 0

    def read(self, size=-1):
        data = self.source.read(size)
        self.h.update(data)
        self.size += len(data)
        return data


def save_selected(output, label, name, data):
    target = output / 'selected' / label / safe_name(name)
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open('xb') as sink:
        sink.write(data)


def source_bytes(source_root, head, path):
    return subprocess.run(['git', 'show', head + ':' + path], cwd=source_root,
                          check=True, capture_output=True).stdout


def supplemental(root, output, source_root, label):
    expected = EXPECTED[label]
    path, observed_digest, storage = supplemental_input(root, label)
    original = observed_digest()
    assert original == expected['supplemental_sha256']
    selected = {
        'retained-results/report.json', 'native-cargo-plan/report.json',
        'capability-results/report.json', 'native-selection-cache/results.json',
        'native-selection-seed/results.json', 'native-compact-selection/results.json',
        'lustr-native-built/COMPONENTS.txt', 'tinycc-signing-qualification/report.json',
        'native-selection-cache/package/lib/lustr-selection-tools.txt',
        'native-selection-seed/package/lib/lustr-selection-tools.txt',
        'native-selection-seed/relocated-package/lib/lustr-selection-tools.txt',
    }
    hashes, histories, expected_streams, seen = {}, [], [], set()
    aggregate = None
    with zipfile.ZipFile(path) as archive:
        assert len(archive.namelist()) == 2
        assert set(archive.namelist()) == {'inventory.json', 'records.tar.gz'}
        inventory_bytes = archive.read('inventory.json')
        inventory = json.loads(inventory_bytes)
        assert inventory['schema'] == 'lustr-qualification-records-v1'
        assert inventory['source'] == {
            'GITHUB_SHA': expected['checkout'], 'GITHUB_RUN_ID': expected['run'],
            'GITHUB_RUN_ATTEMPT': '1',
        }
        for key, source in [('helper_sha256', 'tools/package-qualification-records.py'),
                            ('workflow_sha256', '.github/workflows/compiler-set.yml')]:
            assert digest(source_bytes(source_root, expected['head'], source)) == inventory[key]
        entries = {entry['path']: entry for entry in inventory['entries']}
        assert len(entries) == len(inventory['entries'])
        for name in entries:
            safe_name(name)
        with archive.open('records.tar.gz') as compressed:
            reader = HashedReader(compressed)
            with tarfile.open(fileobj=reader, mode='r|gz') as records:
                for member in records:
                    name = member.name
                    safe_name(name)
                    assert name not in seen
                    seen.add(name)
                    entry = entries[name]
                    assert entry['included'] is True
                    if member.isfile():
                        assert entry['kind'] == 'regular' and member.size == entry['size']
                        content = records.extractfile(member)
                        history = (name.startswith('retained-results/') and name.endswith('/report.json')
                                   and len(PurePosixPath(name).parts) == 3)
                        if history or name in selected:
                            data = content.read()
                            actual, size = digest(data), len(data)
                            if name in selected:
                                save_selected(output, label + '-supplemental', name, data)
                            if name == 'retained-results/report.json':
                                aggregate = json.loads(data)
                            if history:
                                report = json.loads(data)
                                assert report['passed'] is True
                                histories.append({'path': name, 'scenario': report['scenario'],
                                                  'versions': len(report['versions']),
                                                  'commands': len(report['commands'])})
                                for command in report['commands']:
                                    for stream in ['stdout', 'stderr']:
                                        relative = safe_name(command[stream])
                                        stream_path = str(PurePosixPath(name).parent / relative)
                                        expected_streams.append((stream_path, command[stream + '_sha256']))
                        else:
                            actual, size = digest_stream(content)
                        assert actual == entry['sha256'] and size == entry['size']
                        hashes[name] = actual
                    elif member.isdir():
                        assert entry['kind'] == 'directory'
                    elif member.issym():
                        assert entry['kind'] == 'symlink' and member.linkname == entry['target']
                        assert digest(os.fsencode(member.linkname)) == entry['sha256']
                    else:
                        raise ValueError('unexpected archive type: ' + name)
            while reader.read(1024 * 1024):
                pass
            assert reader.h.hexdigest() == inventory['archive']['sha256']
            assert reader.size == inventory['archive']['bytes']
    assert seen == {name for name, entry in entries.items() if entry['included']}
    for name, wanted in expected_streams:
        assert hashes[name] == wanted
    assert len(histories) == len({row['scenario'] for row in histories}) == 349
    assert sum(row['versions'] for row in histories) == 1744
    inventory_source = source_bytes(source_root, expected['head'], 'upstream/retained-scenario-results.json')
    source_inventory = json.loads(inventory_source)
    required = {row['name'] for row in source_inventory['scenarios'] if row['status'] == 'required'}
    unresolved = {row['name'] for row in source_inventory['scenarios'] if row['status'] == 'unresolved'}
    assert {row['scenario'] for row in histories} == required
    assert aggregate['source_head'] == expected['checkout'] and aggregate['passed'] is True
    assert aggregate['inventory_sha256'] == digest(inventory_source)
    assert aggregate['runner_sha256'] == digest(source_bytes(source_root, expected['head'], 'tools/qualify-retained-results.py'))
    assert set(aggregate['expected_scenarios']) == required and aggregate['unselected_required'] == []
    assert set(aggregate['unresolved_scenarios']) == unresolved
    assert aggregate['full_retained_coverage'] is False and aggregate['full_rust_coverage'] is False
    assert aggregate['release_admission'] is False
    assert len(aggregate['scenarios']) == len(histories)
    assert {row['name']: row['versions'] for row in aggregate['scenarios']} == {
        row['scenario']: row['versions'] for row in histories}
    assert all(row['passed'] is True and row['failure'] is None for row in aggregate['scenarios'])
    assert original == observed_digest()
    omitted = [entry for entry in entries.values() if entry['kind'] == 'regular' and not entry['included']]
    receipt_entries = [entry for name, entry in entries.items() if name in selected and name.endswith('lustr-selection-tools.txt')]
    return {
        'source': expected, 'zip_bytes': expected['supplemental_bytes'], 'zip_sha256': original,
        'verified_storage': storage,
        'inventory_bytes': len(inventory_bytes), 'inventory_sha256': digest(inventory_bytes),
        'archive': inventory['archive'], 'capture_complete': inventory['capture_complete'],
        'capture_errors': inventory['errors'], 'available_member_integrity_verified': True,
        'complete_capture_verified': inventory['capture_complete'] and not inventory['errors'],
        'inventory_entries': len(entries), 'verified_archive_members': len(seen),
        'verified_regular_files': len(hashes),
        'verified_regular_bytes': sum(entries[name]['size'] for name in hashes),
        'original_retained_report_stream_hashes_verified': len(expected_streams),
        'retained_reports': histories, 'receipt_entries': receipt_entries,
        'exact_required_inventory_names': True, 'unresolved_histories': sorted(unresolved),
        'retained_runner_and_inventory_match_published_source': True,
        'omitted_compiled_files': len(omitted),
        'omitted_compiled_bytes': sum(entry['size'] for entry in omitted),
        'omitted_payloads_verified': False, 'original_zip_unchanged': True,
    }


def linux(root, output, label):
    path = root / (label + '-linux-original.zip')
    original = digest_file(path)
    assert original == EXPECTED[label]['linux_sha256']
    entries, seen = [], set()
    with zipfile.ZipFile(path) as archive:
        for entry in archive.infolist():
            safe_name(entry.filename)
            assert entry.filename not in seen
            seen.add(entry.filename)
            with archive.open(entry) as source:
                actual, size = digest_stream(source)
            assert size == entry.file_size
            entries.append({'path': entry.filename, 'bytes': size, 'sha256': actual})
            if entry.filename.endswith(('report.json', 'verification.json', '-identity.txt', '-layout.txt')):
                save_selected(output, label + '-linux', entry.filename, archive.read(entry))
    assert original == digest_file(path)
    return {'zip_bytes': path.stat().st_size, 'zip_sha256': original,
            'all_members_read_with_crc_validation': True, 'members': entries,
            'original_zip_unchanged': True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--source-root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--current-supplemental-only', action='store_true',
                        help='verify the published current archive parts; do not claim new historical/Linux verification')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    report = {'schema': 'lustr-recovered-ci-integrity-v1', 'new_compiler_execution': False,
              'scope': 'current-supplemental-only' if args.current_supplemental_only else 'all-four-original-archives',
              'supplemental': {}, 'linux': {}}
    for label in (['c821'] if args.current_supplemental_only else ['c821', '7c']):
        report['supplemental'][label] = supplemental(args.root, args.output, args.source_root, label)
        if not args.current_supplemental_only:
            report['linux'][label] = linux(args.root, args.output, label)
        (args.output / 'verification.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
        print(json.dumps({'source': label, 'capture_complete': report['supplemental'][label]['capture_complete'],
                          'members': report['supplemental'][label]['verified_archive_members'],
                          'streams': report['supplemental'][label]['original_retained_report_stream_hashes_verified'],
                          'linux_members': len(report['linux'].get(label, {}).get('members', []))}), flush=True)


if __name__ == '__main__':
    main()
