# Frozen regression packages use their matching constructor

The Compiler set run at public-entry revision
`3c378a11d60149a3ec99fa49ea1b336e9406fca8` failed while constructing the
unchanged failed-link counterexample, before executing that counterexample.
The [original run](https://github.com/glopyglerky/lustr/actions/runs/44313200448)
and [mixed job](https://github.com/glopyglerky/lustr/actions/runs/44313200448/job/134832404321)
retain the failure. The current package constructor called the old installed
entry with `--lustr-prime-selection-tools`; that entry rejected the unknown
option. This setup failure is not the required stale-output regression.

The [failure excerpt](evidence/frozen-regression-constructor-2026-11-18/original-failure.log)
preserves the invocation through the reported exit 101. Its
[retrieval receipt](evidence/frozen-regression-constructor-2026-11-18/original-failure.json)
binds that excerpt and records the full retrieved decoded log's UTF-8 SHA-256
`e4ccd5ec58508fcfea4c42ee13790f1a55e98937faede713505a50777029672f`
(113,474 bytes). The separate original checkout excerpt identifies executed
merge `d3b14dc637b6343bbb329dfcf6d5e1b420c06009`. This small archive does not
contain the full job log.

The workflow now builds the pinned baseline with its own constructor source.
It does not change the baseline, remove initialization from current packages,
or accept the unrelated failure as qualification.

| Frozen component | Identity |
| --- | --- |
| Baseline revision | `2add988c6397ac5175fcdf89a3aac17d359bf283` |
| Baseline tree | `faa17b8946f39deb249e367a4f34c70a493f6f0b` |
| Constructor Git blob | `d342586586642546950ea7d1e87031bd0f3050dc` |
| Constructor SHA-256 | `6a1bf11a94c68dafebca20a8be6283eb74c5f4c6b756fbfa0b664af2745a2e5f` |

That constructor accepts exactly four arguments and builds debug artifacts.
It predates `--release`, the profile receipt and selection-tool initialization.
The baseline's only purpose here is to reproduce the specific failed-link
stale-output error. It is not used as a performance reference. Current candidate
and performance package construction keep their existing release profiles.

`tools/build-regression-baseline.py` requires the exact clean baseline checkout,
compares its constructor bytes with the committed Git object, and archives those
bytes before compiling them. Its receipt records source revision/tree/blob/hash,
the actual Rust compiler version, constructor binary hash, and each build/run
command with exit status and original stdout/stderr hashes. Failed commands
retain their streams and fail the step. After successful construction it checks
the baseline source again. The workflow uploads the receipt, frozen constructor
source, command streams and existing package logs even on failure.

Local validation passed seven process-level orchestration controls and
21 workflow controls. They cover successful legacy argument routing, changed or
wrong source, constructor build/run failures, a missing compiler and preservation
of an existing output. These fixtures test orchestration; they are not compiler
qualification.

The actual frozen constructor also compiled successfully with pinned Rust
`5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. A bounded pre-build probe rejected
the newer `--release` argument. The corresponding four-argument probe advanced
past argument validation and refused an intentionally existing output, preserving
its sentinel. Neither probe built native components. The repaired full hosted
baseline build and specific stale-output reproduction remain pending.

The [small argument-contract archive](evidence/frozen-regression-constructor-2026-11-18/argument-contract.json)
contains the exact frozen constructor source, compiler identity output and all
four commands' original stdout/stderr with hashes. Its
[replayer](evidence/frozen-regression-constructor-2026-11-18/verify.py) checks those
bindings and both distinct rejection boundaries. The compiler and constructor
binaries are not archived, so their recorded hashes are identities rather than
locally recomputed archive checks. The intentionally existing output sentinel
was checked by the executing probe and is not reconstructed by replay.
