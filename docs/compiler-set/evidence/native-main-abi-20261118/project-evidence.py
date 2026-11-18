#!/usr/bin/env python3
"""Project retained private snapshot results without paths, argv or raw streams."""
import argparse
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--snapshot', type=Path, required=True)
p.add_argument('--receipts', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)

BINARY_SOURCES = {
    '80441b744ffae7e4f225f17d2576f7e85e9a9b71eca2cd00ae12fd3d10e684b9': {
        'kind': 'baseline',
        'lust_core_sha256': 'eb8ecb7fe939dceb343c80f6ae62bdb6b3fb1fc5181de531734f7c1385ed509a',
        'compiled_cgen_sha256': '8ab4c3e87610597e5d9c339afce898973b8e7913a532aef97c7fec59638dccdd',
    },
    'e5fbff6e3f43c2de446f7238fdabf8a42184a9208c55d0875b3a2e5c35499f7c': {
        'kind': 'candidate',
        'lust_core_sha256': '32adcf8caee9ec68c542a3b172707ad865099d9108cf28f1daf4d5fdd61743c3',
        'compiled_cgen_sha256': '1cbce6afffc8f3df96032b5588736cbd861033feba8111bec5047889472c12b1',
    },
}
IDENTITY_KEYS = ('source_head', 'native_base_tree', 'lust_sha256', 'lust_core_sha256',
                 'rustc_sha256', 'cc_sha256', 'release_c', 'unwind_tables', 'runner_sha256')
COMMAND_KEYS = ('label', 'exit', 'timed_out', 'stdout_sha256', 'stderr_sha256')
reports = []
for campaign in sorted((a.snapshot / 'campaigns').iterdir()):
    raw = campaign / 'report.json'
    if not raw.exists():
        continue
    report = json.loads(raw.read_text())
    identity = report.get('identity', report)
    source = BINARY_SOURCES[identity['lust_sha256']]
    assert identity['lust_core_sha256'] == source['lust_core_sha256']
    public_identity = {key: identity[key] for key in IDENTITY_KEYS if key in identity}
    public_identity.update(source)
    public_identity['execution_workspace_cgen_sha256'] = identity.get(
        'execution_workspace_cgen_sha256', identity.get('cgen_sha256'))
    commands = []
    for command in report['commands']:
        row = {key: command[key] for key in COMMAND_KEYS if key in command}
        if 'cwd' in command:
            row['case'] = Path(command['cwd']).name
        commands.append(row)
    reports.append({'campaign': campaign.name, 'raw_report_sha256': hashlib.sha256(raw.read_bytes()).hexdigest(),
                    'identity': public_identity, 'flags': report.get('flags'),
                    'passed': report['passed'], 'results': report['results'], 'commands': commands})

# The initial probe stopped at the first TinyCC timeout before writing report.json.
# Its original completed row and TimeoutExpired traceback survive in the command receipt.
raw = a.receipts / 'partial-baseline-probe.json'
receipt = json.loads(raw.read_text())
rows = [json.loads(line) for line in receipt['stdout'].splitlines() if line.startswith('{')]
assert receipt['exit_code'] != 0 and 'TimeoutExpired' in receipt['stderr']
reports.insert(0, {'campaign': 'baseline', 'status': 'incomplete', 'passed': False,
                  'original_receipt_sha256': hashlib.sha256(raw.read_bytes()).hexdigest(),
                  'completed_rows': rows, 'stopped_at': 'adapter_closures_apart native-run timeout',
                  'identity_limit': 'initial in-memory report was not saved; binary/source map is documented separately'})
result = {'schema': 'lustr-native-main-abi-public-v1',
          'comparison': 'generated-C probe compares exact exit/stdout and sanitizer diagnostics; strict transitions also compare stderr',
          'source_identity_note': 'execution workspace cgen is distinct from preserved binary compiled source',
          'campaigns': reports}
(a.output / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
manifest = json.loads((a.snapshot / 'SNAPSHOT.json').read_text())
inventory = {'schema': 'lustr-native-main-abi-public-inventory-v1',
             'files': [{key: row[key] for key in ('name', 'sha256', 'bytes')} for row in manifest['files']]}
(a.output / 'inventory.json').write_text(json.dumps(inventory, indent=2) + '\n')
print(json.dumps({'campaigns': len(reports), 'retained_snapshot_files': len(inventory['files'])}))
