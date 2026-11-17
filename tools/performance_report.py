#!/usr/bin/env python3
"""Import pinned CI evidence, independently replay statistics, and render docs.

Standard library only. No compiler or artifact payload is executed. Fetch mode
uses the Actions token only for api.github.com; redirected blob requests do not
receive it. Offline replay requires no network or credentials.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import io
import itertools
import json
import math
import os
from pathlib import Path, PurePosixPath
import platform
import re
import statistics
import sys
import urllib.parse
import urllib.request
import zipfile

REPO = 'glopyglerky/lustr'
REPO_ID = 1450430441
MAX_ARCHIVE = 20 * 1024 * 1024
MAX_EXPANDED = 128 * 1024 * 1024
MEASUREMENTS = 'lustr-paired-measurements/'
PHASES = ('cold', 'no-op', 'body-edit', 'signature-edit', 'check',
          'check-no-op', 'check-body-edit', 'rejection', 'compile-plus-test')
COMPARISONS = (('native', 'pristine', 'integrated'),
               ('native', 'pristine', 'entry'), ('cargo', 'reference', 'mixed'))
LABELS = {
    ('native', 'pristine', 'integrated'): 'Integrated Lust versus pristine Lust',
    ('native', 'pristine', 'entry'): 'Packaged Lustr native entry versus pristine Lust',
    ('cargo', 'reference', 'mixed'): 'Mixed Lustr Cargo builds versus matched Cargo/rustc',
}
METHOD = 'paired-log-v1/splitmix64/rejection-index/percentile'
MASK = (1 << 64) - 1


def require(value: bool, message: str) -> None:
    if not value:
        raise ValueError(message)


def unique_pairs(pairs):
    out = {}
    for key, value in pairs:
        require(key not in out, f'duplicate JSON key: {key}')
        out[key] = value
    return out


def decode(data: bytes | str):
    def invalid(value):
        raise ValueError(f'nonfinite JSON constant: {value}')
    return json.loads(data, object_pairs_hook=unique_pairs, parse_constant=invalid)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def positive(value) -> bool:
    return type(value) in (int, float) and math.isfinite(value) and value > 0


class PrivateRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        parsed = urllib.parse.urlsplit(newurl)
        require(parsed.scheme == 'https' and parsed.hostname is not None,
                'refusing a non-HTTPS artifact redirect')
        result = super().redirect_request(req, fp, code, msg, headers, newurl)
        if result is not None:
            result.remove_header('Authorization')
        return result


def api(endpoint: str, *, binary=False):
    require(endpoint.startswith('/') and '://' not in endpoint, 'invalid API path')
    token = os.environ.get('GH_TOKEN') or os.environ.get('GITHUB_TOKEN')
    require(bool(token), 'fetch requires a repository-scoped Actions token')
    request = urllib.request.Request(
        'https://api.github.com' + endpoint,
        headers={'Accept': 'application/vnd.github+json',
                 'Authorization': 'Bearer ' + token,
                 'X-GitHub-Api-Version': '2022-11-28',
                 'User-Agent': 'lustr-performance-evidence'},
    )
    with urllib.request.build_opener(PrivateRedirect()).open(request, timeout=60) as response:
        data = response.read(MAX_ARCHIVE + 1)
    require(len(data) <= MAX_ARCHIVE, 'API response exceeds the bounded input size')
    return data if binary else decode(data)


def safe_zip(data: bytes, expected_digest: str) -> dict[str, bytes]:
    require(sha(data) == expected_digest, 'artifact SHA-256 mismatch')
    require(len(data) <= MAX_ARCHIVE, 'oversized archive')
    result = {}
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        require(len(archive.infolist()) <= 5000, 'too many archive entries')
        total = 0
        for entry in archive.infolist():
            path = PurePosixPath(entry.filename)
            require(not path.is_absolute() and '..' not in path.parts and '\\' not in entry.filename,
                    'unsafe archive path')
            require(str(path) == entry.filename.rstrip('/'), 'noncanonical archive path')
            require(entry.filename not in result, 'duplicate archive entry')
            require((entry.external_attr >> 16) & 0o170000 != 0o120000, 'archive symlink')
            require(not entry.flag_bits & 1, 'encrypted archive entry')
            total += entry.file_size
            require(total <= MAX_EXPANDED, 'oversized expanded archive')
            if entry.is_dir():
                continue
            result[entry.filename] = archive.read(entry)
    return result


def rng_next(state):
    state = (state + 0x9e3779b97f4a7c15) & MASK
    z = state
    z = ((z ^ (z >> 30)) * 0xbf58476d1ce4e5b9) & MASK
    z = ((z ^ (z >> 27)) * 0x94d049bb133111eb) & MASK
    return state, (z ^ (z >> 31)) & MASK


def estimate(pairs, resamples, seed):
    require(len(pairs) > 0 and all(positive(x) for pair in pairs for x in pair), 'invalid timing')
    require(type(resamples) is int and 100 <= resamples <= 1_000_000, 'invalid bootstrap count')
    require(type(seed) is int and 0 <= seed <= MASK, 'invalid random seed')
    logs = [math.log(a) - math.log(b) for a, b in pairs]
    mean = lambda xs: sum(x / len(xs) for x in xs)
    boots = []
    state = seed
    n = len(logs)
    threshold = ((-n) & MASK) % n
    for _ in range(resamples):
        sample = []
        for _ in logs:
            while True:
                state, value = rng_next(state)
                if value >= threshold:
                    sample.append(logs[value % n])
                    break
        boots.append(math.exp(mean(sample)))
    boots.sort()
    baseline = sorted(a for a, _ in pairs)
    candidate = sorted(b for _, b in pairs)
    p = None
    if n <= 16:
        observed = abs(mean(logs))
        tolerance = 64 * sys.float_info.epsilon * max(1, sum(abs(x) for x in logs))
        extreme = sum(abs(mean([x * s for x, s in zip(logs, signs)])) + tolerance >= observed
                      for signs in itertools.product((-1, 1), repeat=n))
        p = extreme / (1 << n)
    return {
        'method': METHOD, 'pairs': n,
        'raw_ratios': [math.exp(x) for x in logs], 'log_ratios': logs,
        'geometric_ratio': math.exp(mean(logs)),
        'bootstrap_95': [boots[math.floor(.025 * (resamples - 1))],
                         boots[math.ceil(.975 * (resamples - 1))]],
        'baseline_median': statistics.median(baseline),
        'candidate_median': statistics.median(candidate),
        'baseline_p95': baseline[math.ceil(.95 * n) - 1],
        'candidate_p95': candidate[math.ceil(.95 * n) - 1],
        'exact_sign_flip_p': p, 'bootstrap_resamples': resamples, 'seed': seed,
    }


def equivalent(actual, expected, path='summary'):
    if isinstance(expected, dict):
        require(isinstance(actual, dict) and actual.keys() == expected.keys(), f'{path}: schema differs')
        for key in expected:
            equivalent(actual[key], expected[key], path + '.' + key)
    elif isinstance(expected, list):
        require(isinstance(actual, list) and len(actual) == len(expected), f'{path}: vector differs')
        for i, value in enumerate(expected):
            equivalent(actual[i], value, f'{path}[{i}]')
    elif isinstance(expected, float):
        require(positive(actual) or actual == 0 or (type(actual) in (int, float) and math.isfinite(actual)),
                f'{path}: nonfinite result')
        require(math.isclose(actual, expected, rel_tol=2e-10, abs_tol=2e-12), f'{path}: replay differs')
    else:
        require(type(actual) is type(expected) and actual == expected, f'{path}: identity differs')


def validate(files, campaign):
    report_bytes = files[MEASUREMENTS + 'report.json']
    raw_bytes = files[MEASUREMENTS + 'raw.jsonl']
    report = decode(report_bytes)
    require(report.get('schema') == 1 and report.get('status') == 'completed', 'incomplete campaign')
    require(report.get('performance_admission') == 'not granted by this runner', 'unexpected admission claim')
    require(report.get('raw_sha256') == sha(raw_bytes), 'raw JSONL digest mismatch')
    records = [decode(line) for line in raw_bytes.splitlines() if line.strip()]
    require(records == report.get('records'), 'report and raw event stream differ')
    n = report.get('pairs')
    require(type(n) is int and 12 <= n <= 96 and n % 6 == 0, 'invalid paired sample count')
    revision = files['lustr-source-revision.txt'].decode().split()
    require(len(revision) == 2 and all(re.fullmatch('[0-9a-f]{40}', x) for x in revision), 'invalid checkout identity')
    require(revision[0] == campaign['checkout_sha'], 'wrong checked-out source')
    hashes = report.get('compiler_sha256', {})
    require(len(hashes) >= 10 and all(re.fullmatch('[0-9a-f]{64}', value) for value in hashes.values()),
            'missing compiler/build identities')
    for name in ('BUILD-PROFILE.txt', 'COMPONENTS.txt', 'NATIVE-BUILD-RECEIPT.txt'):
        values = [value for path, value in hashes.items() if Path(path).name == name]
        require(len(values) == 1 and values[0] == sha(files[MEASUREMENTS + name]), 'build identity mismatch: ' + name)
    groups = {}
    timed = []
    for record in records:
        if record.get('kind') != 'command':
            continue
        require(record.get('timed_out') is False and record.get('signal') is None
                and record.get('exit') == record.get('expected_exit'), 'unsuccessful command')
        for stream in ('stdout', 'stderr'):
            name = record[stream]
            require(isinstance(name, str) and PurePosixPath(name).name == name, 'unsafe command log path')
            payload = files[MEASUREMENTS + name]
            require(sha(payload) == record[stream + '_sha256'] and len(payload) == record[stream + '_bytes'],
                    'command log digest/size mismatch')
        sample = record.get('sample')
        if sample is None:
            continue
        require(type(sample) is int and 0 <= sample < n, 'invalid sample id')
        family, phase, lane = (record.get(k) for k in ('family', 'phase', 'lane'))
        expected_lanes = ('pristine', 'integrated', 'entry') if family == 'native' else ('reference', 'mixed')
        require(family in ('native', 'cargo') and phase in PHASES and lane in expected_lanes, 'unexpected timing mode')
        expected_exit = (1 if family == 'native' else 101) if phase == 'rejection' else 0
        require(record['expected_exit'] == expected_exit and positive(record.get('seconds')), 'wrong exit or time')
        require(record.get('source_sha256') and record['source_sha256'] == record.get('source_after_sha256'),
                'measured source changed')
        key = (family, phase, lane, sample)
        require(key not in groups, 'duplicate paired sample')
        groups[key] = record
        timed.append(record)
    require(len(groups) == 45 * n, 'incomplete timed command matrix')
    summaries = report.get('summaries')
    require(isinstance(summaries, list) and len(summaries) == 27, 'incomplete comparison matrix')
    seen = set()
    paired = []
    for row in summaries:
        key = (row['family'], row['baseline'], row['candidate'])
        phase = row['phase']
        require(key in COMPARISONS and phase in PHASES and (key, phase) not in seen, 'duplicate/unknown comparison')
        seen.add((key, phase))
        pairs = []
        for sample in range(n):
            a = groups[key[0], phase, key[1], sample]
            b = groups[key[0], phase, key[2], sample]
            require(a['source_sha256'] == b['source_sha256'], 'paired inputs differ')
            pairs.append((a['seconds'], b['seconds']))
            paired.append([*key, phase, sample, a['seconds'] * 1000, b['seconds'] * 1000, a['seconds'] / b['seconds']])
        analysis = row['analysis']
        replay = estimate(pairs, analysis['bootstrap_resamples'], analysis['seed'])
        equivalent(analysis, replay)
        low, high = replay['bootstrap_95']
        observation = 'slower-on-this-corpus' if high < 1 else 'faster-on-this-corpus' if low > 1 else 'inconclusive'
        require(row['observation'] == observation, 'incorrect performance label')
    return report, paired, revision


def table(report, comparison, full=True):
    rows = {row['phase']: row for row in report['summaries']
            if (row['family'], row['baseline'], row['candidate']) == comparison}
    lines = ['| Mode | Baseline median / p95 (ms) | Candidate median / p95 (ms) | Speed ratio, 95% interval | Candidate time change | Observation |',
             '|---|---:|---:|---:|---:|---|']
    for phase in PHASES:
        row = rows[phase]
        a = row['analysis']
        ratio = a['geometric_ratio']
        lines.append(f"| {phase} | {a['baseline_median']*1000:.3f} / {a['baseline_p95']*1000:.3f} | "
                     f"{a['candidate_median']*1000:.3f} / {a['candidate_p95']*1000:.3f} | "
                     f"{ratio:.3f}x [{a['bootstrap_95'][0]:.3f}, {a['bootstrap_95'][1]:.3f}] | "
                     f"{100*(1/ratio-1):+.1f}% | {row['observation']} |")
    return '\n'.join(lines)


def run_markdown(campaign, report, revision):
    parts = [f"# Performance campaign {campaign['run_id']}", '',
             '**Completed measurement; not performance or release admission.**', '',
             f"Candidate head: `{campaign['head_sha']}`. Actual checked-out merge/source: `{revision[0]}`; tree `{revision[1]}`.", '',
             f"[Workflow run](https://github.com/{REPO}/actions/runs/{campaign['run_id']}) · "
             f"[Original artifact](https://github.com/{REPO}/actions/runs/{campaign['run_id']}/artifacts/{campaign['artifact_id']})", '',
             '[Original evidence ZIP](evidence.zip) · [Original report](report.json) · [Raw commands](raw.jsonl) · [Paired samples in milliseconds](pairs.csv) · [Provenance](provenance.json)', '',
             f"Artifact SHA-256: `{campaign['sha256']}`. Raw JSONL SHA-256: `{report['raw_sha256']}`.", '',
             f"{report['pairs']} pairs per mode; 540 timed commands when pairs=12; 27 comparisons across 18 native/Cargo modes. "
             'Runtime assertions and setup/priming calls are retained but excluded from compile-only samples.', '',
             'Ratio = baseline time / candidate time; greater than 1 is faster. Candidate time change = (1 / ratio - 1) × 100%; positive means slower. '
             'This uses the geometric paired ratio, not the ratio of the displayed medians. Intervals are exploratory per-mode 95% paired bootstrap intervals, not simultaneous guarantees. '
             'An interval crossing 1 is inconclusive, not proof of equivalence. With 12 samples, nearest-rank p95 is the maximum and is not a stable production tail estimate.', '',
             '## Scope and environment', '',
             f"- Upstream Lust reference: `{report['pristine_revision']}`; tree `{report['pristine_tree']}`.",
             f"- Matching Rust identity: `{report['rust_reference_version'].splitlines()[0]}`. Complete version and binary hashes are in report.json.",
             '- Platform: Linux x86-64 source-built CI lane. The original campaign did not capture CPU model, RAM, affinity, frequency policy or concurrent load. Do not infer these from runner labels.',
             f"- Profiles: {report['profiles']}",
             f"- Cold: {report['cold_scope']}",
             f"- Warm: {report['warm_scope']}",
             f"- Timing: {report['timing_scope']}",
             f"- Workload: {report['scope']}", '',
             'The native fixture is a two-function scalar program. The mixed graph adds a procedural macro, async code and a no_std/alloc dependency. '
             'Both are bounded mechanism tests, not representative proof for large real applications. The native and Cargo families do different work; do not divide their times to claim a cross-family speedup.', '']
    for comparison in COMPARISONS:
        parts += ['## ' + LABELS[comparison], '', table(report, comparison), '']
    parts += ['## Verification and interpretation', '',
              'The documentation importer verified the complete artifact digest, checkout identity, build-receipt hashes, raw JSONL hash, '
              'command-output hashes, expected success/rejection exits, unchanged source inputs, all paired sample identities and all 27 summaries. '
              'It independently recomputed medians, p95, geometric ratios, 10,000 bootstrap resamples and the exact sign-flip probabilities from the original raw commands. '
              'No measured row was omitted, pooled across hosts or replaced with an upstream number.', '',
              'This replay validates recorded evidence and arithmetic. It does not rerun the compiler, establish general semantic equivalence, identify the cause of a regression, '
              'or turn a successful benchmark job into a speed pass. Separate native platform and release obligations remain.', '',
              'Reproduce the documentation offline with `python3 tools/performance_report.py replay --root docs/performance --check`. '
              'For a fresh compiler campaign, follow the [performance methodology](../../README.md).', '']
    return '\n'.join(parts)


def scoreboard(campaign, report):
    parts = ['# Latest recorded compiler performance', '',
             '**The combined toolchain is not speed-qualified. Full Rust coverage through the imported compiler does not establish Lust-equivalent full-project speed.**', '',
             f"Latest pinned completed campaign: [{campaign['run_id']}](runs/{campaign['run_id']}/README.md), head `{campaign['head_sha'][:12]}`. "
             'All modes, regressions and uncertainty are shown; older runs remain individually retained.', '']
    for comparison in COMPARISONS:
        parts += ['## ' + LABELS[comparison], '', table(report, comparison), '']
    parts += ['These are tiny-fixture results, not upstream Lust\'s ts-rust results. A green characterization job means the campaign completed, not that it was faster. '
              'The separate [methodology and coverage register](README.md) defines what is measured and what remains missing.', '']
    return '\n'.join(parts)


def root_section(campaign, report):
    rows = {(r['family'], r['candidate'], r['phase']): r['analysis'] for r in report['summaries']}
    lines = ['<!-- LUSTR-PERFORMANCE:BEGIN -->', '## Measured compiler speed', '',
             '**Not yet speed-qualified.** We record Lustr against both pristine Lust and the matching stock Cargo/rustc. '
             'The completed mixed-project campaign currently includes regressions; a green benchmark job does not mean speed parity.', '',
             f"Latest recorded campaign: `{campaign['head_sha'][:12]}`, {report['pairs']} randomized paired samples per mode on Linux x86-64. "
             'These are bounded scalar/mixed fixtures, not large-project performance claims.', '',
             '| Comparison | Mode | Baseline median | Lustr median | Paired speed ratio (baseline / Lustr) |',
             '|---|---|---:|---:|---:|']
    for label, family, lane, phase in [('Pristine Lust → Lustr native entry', 'native', 'entry', 'body-edit'),
                                      ('Pristine Lust → Lustr native entry', 'native', 'entry', 'rejection'),
                                      ('Cargo/rustc → mixed Lustr', 'cargo', 'mixed', 'cold'),
                                      ('Cargo/rustc → mixed Lustr', 'cargo', 'mixed', 'body-edit'),
                                      ('Cargo/rustc → mixed Lustr', 'cargo', 'mixed', 'signature-edit')]:
        a = rows[family, lane, phase]
        lines.append(f"| {label} | {phase} | {a['baseline_median']*1000:.3f} ms | {a['candidate_median']*1000:.3f} ms | "
                     f"{a['geometric_ratio']:.3f}x [{a['bootstrap_95'][0]:.3f}, {a['bootstrap_95'][1]:.3f}] |")
    lines += ['', 'A ratio below 1 is slower. The intervals are per-mode 95% bootstrap intervals; an interval crossing 1 is inconclusive, not an equivalence certificate.', '',
              '**[All comparisons and modes](docs/performance/RESULTS.md)** · '
              '**[Methodology, reproduction and remaining coverage](docs/performance/README.md)** · '
              f"**[Raw evidence and exact tool identities](docs/performance/runs/{campaign['run_id']}/README.md)**", '',
              'Upstream Lust\'s large warm-edit gains are its own workload-specific results. They are not silently transferred to Lustr, mixed Cargo builds or executable runtime.',
              '<!-- LUSTR-PERFORMANCE:END -->']
    return '\n'.join(lines)


def write_or_check(path, content, check):
    data = content.encode() if isinstance(content, str) else content
    if check:
        require(path.is_file() and path.read_bytes() == data, f'stale generated documentation: {path}')
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        require(not path.is_symlink(), 'refusing to replace a symlink')
        path.write_bytes(data)


def load_manifest(root):
    document = decode((root / 'campaigns.json').read_bytes())
    require(document.get('repository') == REPO and document.get('schema') == 1, 'wrong evidence repository/schema')
    campaigns = document['campaigns']
    require(1 <= len(campaigns) <= 50, 'invalid campaign manifest')
    seen = set()
    for c in campaigns:
        require(type(c['run_id']) is int and type(c['artifact_id']) is int and c['run_id'] not in seen, 'invalid/duplicate run')
        seen.add(c['run_id'])
        require(all(re.fullmatch('[0-9a-f]{40}', c[k]) for k in ('head_sha', 'checkout_sha')), 'invalid source SHA')
        require(re.fullmatch('[0-9a-f]{64}', c['sha256']) is not None, 'invalid artifact digest')
    require(document['latest_run'] in seen, 'latest run not in manifest')
    return document


def fetch_campaign(root, campaign):
    endpoint = f'/repos/{REPO}'
    run = api(endpoint + f"/actions/runs/{campaign['run_id']}")
    artifact = api(endpoint + f"/actions/artifacts/{campaign['artifact_id']}")
    require(run['repository']['id'] == REPO_ID and run['head_sha'] == campaign['head_sha'], 'run/repository mismatch')
    require(run['status'] == 'completed' and run['conclusion'] == 'success', 'campaign workflow did not complete successfully')
    require(artifact['workflow_run']['id'] == campaign['run_id'] and not artifact['expired'], 'artifact run/expiry mismatch')
    require(artifact['digest'] == 'sha256:' + campaign['sha256'], 'API artifact digest mismatch')
    data = api(endpoint + f"/actions/artifacts/{campaign['artifact_id']}/zip", binary=True)
    files = safe_zip(data, campaign['sha256'])
    report, _, revision = validate(files, campaign)
    commit = api(endpoint + '/git/commits/' + revision[0])
    require(commit['tree']['sha'] == revision[1], 'checkout tree mismatch')
    require(revision[0] == campaign['head_sha'] or campaign['head_sha'] in [p['sha'] for p in commit['parents']],
            'checkout is not the measured head or its proposed merge')
    directory = root / 'runs' / str(campaign['run_id'])
    provenance = {'schema': 1, 'repository': REPO, **campaign,
                  'run_attempt': run['run_attempt'], 'run_conclusion': run['conclusion'],
                  'created_at': run['created_at'], 'updated_at': run['updated_at'],
                  'artifact_created_at': artifact['created_at'], 'artifact_size': len(data),
                  'checkout_tree': revision[1], 'raw_sha256': report['raw_sha256']}
    write_or_check(directory / 'evidence.zip', data, False)
    write_or_check(directory / 'provenance.json', json.dumps(provenance, indent=2, sort_keys=True) + '\n', False)


def generate(root, check=False):
    manifest = load_manifest(root)
    reports = {}
    for campaign in manifest['campaigns']:
        directory = root / 'runs' / str(campaign['run_id'])
        files = safe_zip((directory / 'evidence.zip').read_bytes(), campaign['sha256'])
        report, paired, revision = validate(files, campaign)
        provenance = decode((directory / 'provenance.json').read_bytes())
        require(all(provenance.get(k) == v for k, v in campaign.items()), 'provenance does not match manifest')
        require(provenance['raw_sha256'] == report['raw_sha256'] and provenance['checkout_tree'] == revision[1], 'provenance changed')
        for name in ('report.json', 'raw.jsonl'):
            write_or_check(directory / name, files[MEASUREMENTS + name], check)
        out = io.StringIO(newline='')
        writer = csv.writer(out, lineterminator='\n')
        writer.writerow(['family', 'baseline', 'candidate', 'mode', 'pair', 'baseline_ms', 'candidate_ms', 'baseline_over_candidate'])
        writer.writerows(sorted(paired, key=lambda row: tuple(row[:5])))
        write_or_check(directory / 'pairs.csv', out.getvalue(), check)
        write_or_check(directory / 'README.md', run_markdown(campaign, report, revision), check)
        reports[campaign['run_id']] = report
        print(f"VERIFIED run={campaign['run_id']} source={campaign['head_sha']} comparisons=27 pairs={report['pairs']} raw_sha256={report['raw_sha256']}")
    latest = next(c for c in manifest['campaigns'] if c['run_id'] == manifest['latest_run'])
    report = reports[latest['run_id']]
    write_or_check(root / 'RESULTS.md', scoreboard(latest, report), check)
    readme = root.parent.parent / 'README.md'
    original = readme.read_text()
    block = root_section(latest, report)
    if '<!-- LUSTR-PERFORMANCE:BEGIN -->' in original:
        require(original.count('<!-- LUSTR-PERFORMANCE:BEGIN -->') == 1 and original.count('<!-- LUSTR-PERFORMANCE:END -->') == 1,
                'ambiguous README markers')
        updated = re.sub(r'<!-- LUSTR-PERFORMANCE:BEGIN -->.*?<!-- LUSTR-PERFORMANCE:END -->',
                         lambda _: block, original, flags=re.S)
    else:
        anchor = 'The following native expansion experiments are retained research.'
        require(original.count(anchor) == 1, 'README insertion point changed')
        updated = original.replace(anchor, block + '\n\n' + anchor)
    write_or_check(readme, updated, check)
    print(scoreboard(latest, report))


def capture(path):
    """Record nonidentifying hardware/environment facts, never a full environment."""
    info = {'schema': 1, 'system': platform.system(), 'release': platform.release(),
            'machine': platform.machine(), 'logical_cpus': os.cpu_count(),
            'affinity': sorted(os.sched_getaffinity(0)) if hasattr(os, 'sched_getaffinity') else None,
            'load_average': list(os.getloadavg()) if hasattr(os, 'getloadavg') else None}
    cpu = Path('/proc/cpuinfo')
    if cpu.is_file():
        info['cpu_models'] = sorted({line.split(':', 1)[1].strip() for line in cpu.read_text().splitlines()
                                     if line.startswith('model name')})
    mem = Path('/proc/meminfo')
    if mem.is_file():
        info['memory'] = {line.split(':', 1)[0]: line.split(':', 1)[1].strip() for line in mem.read_text().splitlines()
                          if line.split(':', 1)[0] in ('MemTotal', 'MemAvailable', 'SwapTotal', 'SwapFree')}
    for name in ('cpu.max', 'memory.max'):
        p = Path('/sys/fs/cgroup') / name
        info['cgroup_' + name] = p.read_text().strip() if p.is_file() else None
    governor = Path('/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor')
    info['cpu0_scaling_governor'] = governor.read_text().strip() if governor.is_file() else None
    write_or_check(Path(path), json.dumps(info, indent=2, sort_keys=True) + '\n', False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    for name in ('fetch', 'replay'):
        p = sub.add_parser(name)
        p.add_argument('--root', type=Path, default=Path('docs/performance'))
        if name == 'replay':
            p.add_argument('--check', action='store_true')
    sub.add_parser('capture').add_argument('--output', required=True)
    args = parser.parse_args()
    if args.command == 'capture':
        capture(args.output)
    elif args.command == 'fetch':
        for campaign in load_manifest(args.root)['campaigns']:
            fetch_campaign(args.root, campaign)
        generate(args.root)
    else:
        generate(args.root, args.check)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, zipfile.BadZipFile) as error:
        print(f'performance evidence rejected: {error}', file=sys.stderr)
        raise SystemExit(1)
