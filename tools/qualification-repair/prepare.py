"""Apply exact-source fixes in an isolated worktree and publish a review branch.

One-shot development operation, never a compiler build overlay. Only the pinned
PR9 source is accepted. Existing remote branches are never overwritten.
"""
import hashlib
import os
from pathlib import Path
import subprocess
import sys

BASE = 'baf8b35700b158bdf7fd2752e32d3b3b8ebf4aa9'
PRISTINE = '240e08d91d4aed0299639f6f11fd522912b248f9'
REFERENCE_TREE = '0ab1b09815802894e6078de0af59273cbdf46cb0'
TARGET = 'lustr/native-qualification-fixes-20261117'
ROOT = Path.cwd()
WORK = Path(os.environ['RUNNER_TEMP']) / 'lustr-qualification-repair'
TOOLS = Path(os.environ['RUNNER_TEMP']) / 'lustr-qualification-repair-tools'
TOOLS.mkdir(exist_ok=False)


def run(*args, cwd=WORK, data=None):
    result = subprocess.run(args, cwd=cwd, input=data, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if result.returncode:
        sys.stdout.buffer.write(result.stdout)
        sys.stderr.buffer.write(result.stderr)
        raise RuntimeError(f'command failed ({result.returncode}): {args}')
    return result.stdout


def checked(path, expected):
    data = (WORK / path).read_bytes()
    actual = hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()
    if actual != expected:
        raise RuntimeError(f'unexpected preimage at {path}: {actual}')
    return data.decode()


def replace_once(text, old, new):
    if text.count(old) != 1:
        raise RuntimeError(f'expected exactly one source anchor: {old[:100]!r}')
    return text.replace(old, new, 1)


if os.environ.get('GITHUB_REPOSITORY') != 'glopyglerky/lustr':
    raise RuntimeError('wrong repository')
if os.environ.get('GITHUB_REF') != 'refs/heads/lustr/prepare-qualification-fixes-20261117':
    raise RuntimeError('wrong preparation branch')
existing = run('git', 'ls-remote', '--heads', 'origin', f'refs/heads/{TARGET}', cwd=ROOT)
if existing.strip():
    raise RuntimeError('review branch already exists; inspect it instead of overwriting it')
current = run('git', 'ls-remote', '--heads', 'origin', 'refs/heads/lustr/unblock-mixed-qualification-20261117', cwd=ROOT)
if current.split()[0].decode() != BASE:
    raise RuntimeError('PR9 moved; reconcile before applying the patch')
run('git', 'fetch', '--no-tags', 'https://github.com/t3dotgg/lust-lang.git', PRISTINE, cwd=ROOT)
if run('git', 'rev-parse', PRISTINE + '^{tree}', cwd=ROOT).decode().strip() != REFERENCE_TREE:
    raise RuntimeError('upstream identity mismatch')
run('git', 'worktree', 'add', '--detach', str(WORK), BASE, cwd=ROOT)

image_path = 'compiler/native/src/image.rs'
image = checked(image_path, '7f98b8ae867793b32fe1e36306b2dfb8ecda0bab')
image = replace_once(image, '    fn personality(persona: u64) -> c_int;',
    '    #[cfg(target_os = "linux")]\n    fn personality(persona: u64) -> c_int;')
image = replace_once(image, 'fn reexec_now(dir: &Path) -> String {',
    '#[cfg(not(target_os = "linux"))]\nfn reexec_now(_dir: &Path) -> String {\n    "persistent front images are only supported on Linux".into()\n}\n\n#[cfg(target_os = "linux")]\nfn reexec_now(dir: &Path) -> String {')
image = replace_once(image, 'pub fn process_start() {',
    '#[cfg(target_os = "linux")]\npub fn process_start() {')
image += '''
// Image controls must not introduce Linux syscall references into other hosts.
// The ordinary allocator and daemon remain available without front images.
#[cfg(not(target_os = "linux"))]
pub fn process_start() {}

#[cfg(all(test, not(target_os = "linux")))]
#[test]
fn front_images_are_disabled_without_linux_syscalls() {
    assert!(state_dir().is_none());
    assert_eq!(reexec_now(Path::new("unused-image-state")), "persistent front images are only supported on Linux");
    process_start();
}
'''
(WORK / image_path).write_text(image)

bench_path = 'crates/lustr-bench/src/bin/lustr-mixed-bench.rs'
bench = checked(bench_path, 'bb5028da60b2e35ca4761346091c6181cb77cd0a')
bench = replace_once(bench, '#[path = "../order.rs"]\nmod order;',
    '#[path = "../native_evidence.rs"]\nmod native_evidence;\n#[path = "../order.rs"]\nmod order;')
bench = replace_once(bench, '''                    require(
                        text.contains("mode=reuse") || text.contains("path=reuse"),
                        format!("{}: no native retained check evidence: {text}", native_lane(lane)),
                    )?;''', '''                    let route = native_evidence::retained_check(&text)
                        .map_err(|reason| format!("{}: invalid native retained check evidence ({reason}): {text}", native_lane(lane)))?;
                    self.event(json!({"kind":"native-retained-check","lane":native_lane(lane),"route":route}))?;''')
(WORK / bench_path).write_text(bench)
helper_path = 'crates/lustr-bench/src/native_evidence.rs'
helper = (ROOT / 'tools/qualification-repair/native_evidence.rs').read_text()
helper = replace_once(helper, 'let old = format!(" {key}=0 ");', 'let old = format!(" {key}=0");')
helper = replace_once(helper, '&format!(" {key}=1 ")', '&format!(" {key}=1")')
helper = replace_once(helper, '&format!(" {key} "), " "', '&format!(" {key}"), ""')
(WORK / helper_path).write_text(helper)
run('rustfmt', '+1.99.1', '--edition', '2021', helper_path)
run('rustc', '+1.99.1', '--edition=2021', '--test', helper_path, '-o', str(TOOLS / 'native-evidence-tests'))
result = run(str(TOOLS / 'native-evidence-tests'))
sys.stdout.buffer.write(result)
(TOOLS / 'native-evidence-tests.log').write_bytes(result)
result = subprocess.run(['python3', 'tools/test-mixed-workflow.py'], cwd=WORK, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)
sys.stdout.buffer.write(result.stdout)
(TOOLS / 'workflow-tests.log').write_bytes(result.stdout)

note_path = 'docs/compiler-set/qualification-repair-2026-11-17.md'
note = '''# Native qualification repair, 17 November 2026

This additive candidate follows PR9 at `baf8b35700b158bdf7fd2752e32d3b3b8ebf4aa9`.
It does not change the imported complete compiler, language coverage, timing
thresholds, benchmark corpus, requested application profiles or default branch.

## Observed failures

Compiler-set run 44046766887, source-built job 133914272305, passed mixed
execution, failed-link cache isolation, retained checks, fresh metadata,
source-hidden mixed-library consumption, the complete 14-test entry contract
and four target-layout comparisons. The performance probe then rejected
pristine Lust's `mode=bodies` result despite zero rechecked/checked bodies and
zero generated/compiled units. It still performed collection (`collect=4`);
this result must not be described as zero total work or a measured speed gain.
No paired timing campaign completed in that run.

Native CI run 44046767054 failed its macOS ARM64 link because process startup
referenced Linux's `personality` execution-domain syscall. This is unrelated
to Rust's exception/unwind personality routine. The front-image feature was
already documented and selected as Linux-only, but runtime environment checks
did not remove that unresolved reference from the macOS binary.

## Changes and scope

The benchmark now parses one real stats record and one real visits record,
requires a nonempty fixture and zero body/code-generation work counters, and
distinguishes direct reuse from retained bodies with collection work. Missing,
duplicate, malformed, conflicting and nonzero evidence fail. Raw diagnostics
and every existing timing/correctness gate remain intact.

Linux-only image entry points and their FFI declaration are compile-time gated.
Other hosts retain normal compiler/daemon operation and explicitly decline
fixed-address front-image re-execution. No fake syscall, panic-abort change,
linker suppression or runtime library substitution is introduced.

Twelve focused evidence tests executed successfully during preparation. The
existing eleven workflow-control tests also executed successfully. These are
harness tests, not whole-compiler or performance qualification. A non-Linux
native unit regression is included; actual macOS compilation and native CI,
Linux differential CI, and the paired performance campaign remain required.
The exact upstream patch and candidate tree identities are regenerated without
changing the pristine reference or granting release admission.
'''
(WORK / note_path).write_text(note)

run('git', 'add', image_path)
native_tree = run('git', 'write-tree', '--prefix=compiler/native/').decode().strip()
patch = run('git', '-c', 'core.quotePath=true', 'diff', '--binary', '--full-index', '--no-ext-diff', '--no-textconv', '--no-renames', '--no-color', '--ignore-submodules=none', '--submodule=short', '--diff-algorithm=myers', '--no-indent-heuristic', '--unified=3', '--inter-hunk-context=0', '--src-prefix=a/', '--dst-prefix=b/', REFERENCE_TREE, native_tree, '--')
patch_path = 'upstream/lust-integration.patch'
(WORK / patch_path).write_bytes(patch)
patch_sha = run('git', 'hash-object', '-w', patch_path).decode().strip()
qualification_path = 'upstream/lust-integration.qualification'
qualification = f'''reference_revision = "{PRISTINE}"
reference_tree = "{REFERENCE_TREE}"
patch_blob = "{patch_sha}"
integrated_tree = "{native_tree}"
status = "candidate"
correctness_evidence = "{note_path}; focused checks passed; full exact-source native qualification pending"
review_evidence = "{note_path}; independent exact-source review pending"
performance_evidence = "{note_path}; previous probe failed before paired samples; new timings pending"
'''
(WORK / qualification_path).write_text(qualification)
q_sha = run('git', 'hash-object', '-w', qualification_path).decode().strip()
declaration_path = 'upstream/lust.integration'
(WORK / declaration_path).write_text(f'''# Exact candidate source identities; this does not admit a release.
reference_revision = "{PRISTINE}"
reference_tree = "{REFERENCE_TREE}"
patch_blob = "{patch_sha}"
integrated_tree = "{native_tree}"
qualification_blob = "{q_sha}"
''')
paths = [image_path, bench_path, helper_path, note_path, patch_path, qualification_path, declaration_path]
run('git', 'add', '--', *paths)
actual = set(run('git', 'diff', '--cached', '--name-only').decode().splitlines())
if actual != set(paths):
    raise RuntimeError(f'unexpected changed file set: {actual}')
# A serialized unified diff contains mandatory single-space blank context lines.
# Check the real source/doc edits; the original source guard checks every byte
# of the generated reference-to-integrated patch separately below.
run('git', 'diff', '--cached', '--check', '--', '.', ':(exclude)upstream/lust-integration.patch')
run('rustc', '+1.99.1', '--edition=2024', 'tools/lust-upstream-check.rs', '-o', str(TOOLS / 'source-guard'))
result = run(str(TOOLS / 'source-guard'), '--candidate', '--index')
sys.stdout.buffer.write(result)
(TOOLS / 'source-guard.log').write_bytes(result)
(TOOLS / 'source-fixes.patch').write_bytes(run('git', 'diff', '--cached', '--binary'))
run('git', '-c', 'user.name=Lustr integration maintenance', '-c', 'user.email=noreply@users.noreply.github.com', 'commit', '-m', 'Fix retained-probe evidence and Linux-only image startup linkage', '-m', 'Recognize the observed pristine no-body-work route using exact counters rather than a reuse label. Keep actual collection work visible and preserve all timing gates. Compile Linux personality calls only on Linux; retain normal non-Linux compiler behavior. Rebind the exact candidate integration and add focused positive/negative tests. Full native CI and paired performance results remain pending.')
sha = run('git', 'rev-parse', 'HEAD').decode().strip()
print('Validated candidate commit:', sha, flush=True)
(TOOLS / 'candidate-commit.txt').write_text(sha + '\n')
# A non-forced create on a unique review branch; never move next, PR7, PR8 or PR9.
run('git', 'push', 'origin', f'HEAD:refs/heads/{TARGET}')
print('Published review branch:', TARGET, flush=True)
