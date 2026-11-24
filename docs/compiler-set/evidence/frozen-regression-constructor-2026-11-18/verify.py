#!/usr/bin/env python3
"""Replay archived argument-boundary evidence; do not run a compiler."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
receipt = json.loads((root / "argument-contract.json").read_text())
assert receipt["schema"] == "frozen-baseline-argument-contract-v1"
assert receipt["status"] == "completed"
assert receipt["baseline"] == "2add988c6397ac5175fcdf89a3aac17d359bf283"
source = (root / "constructor-source.rs").read_bytes()
assert hashlib.sha256(source).hexdigest() == receipt["constructor_source_sha256"] == "6a1bf11a94c68dafebca20a8be6283eb74c5f4c6b756fbfa0b664af2745a2e5f"
assert hashlib.sha1(b"blob " + str(len(source)).encode() + b"\0" + source).hexdigest() == "d342586586642546950ea7d1e87031bd0f3050dc"
assert receipt["rustc_sha256"] == "ca9d833d09469a534ca1449e71ca9961608409cdb257df96cf8035338fac28de"
assert len(receipt["constructor_binary_sha256"]) == 64
expected = [("compiler-version", 0), ("constructor-build", 0), ("reject-new-release-flag", 101), ("four-argument-contract", 101)]
assert [(row["label"], row["exit"]) for row in receipt["commands"]] == expected
for row in receipt["commands"]:
    assert set(row["streams"]) == {"stdout", "stderr"}
    for name, item in row["streams"].items():
        assert item["file"] == row["label"] + "." + name
        data = (root / item["file"]).read_bytes()
        assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
version, build, extra, legacy = receipt["commands"]
assert version["argv"][1:] == ["-vV"]
assert build["argv"] == [version["argv"][0], "--edition=2024", "constructor-source.rs", "-o", "constructor"]
assert build["cwd"] == extra["cwd"] == legacy["cwd"]
assert extra["argv"][0] == legacy["argv"][0] == str(Path(build["cwd"]) / "constructor")
assert len(extra["argv"]) == 6 and extra["argv"][-1] == "--release"
assert len(legacy["argv"]) == 5 and extra["argv"][:4] == legacy["argv"][:4]
assert b"commit-hash: 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494" in (root / "compiler-version.stdout").read_bytes()
assert b"args.next().is_none()" in (root / "reject-new-release-flag.stderr").read_bytes()
assert b"output must be new; existing data is preserved" in (root / "four-argument-contract.stderr").read_bytes()
failure = json.loads((root / "original-failure.json").read_text())
assert failure["run_id"] == 44313200448 and failure["job_id"] == 134832404321
assert failure["published_head"] == "3c378a11d60149a3ec99fa49ea1b336e9406fca8"
assert failure["executed_merge"] == "d3b14dc637b6343bbb329dfcf6d5e1b420c06009"
assert failure["full_retrieved_log"]["archived"] is False
checkout = (root / failure["checkout_excerpt"]["file"]).read_bytes()
assert len(checkout) == failure["checkout_excerpt"]["bytes"]
assert hashlib.sha256(checkout).hexdigest() == failure["checkout_excerpt"]["sha256"]
assert failure["executed_merge"].encode() in checkout and failure["published_head"].encode() in checkout
excerpt = (root / failure["excerpt"]["file"]).read_bytes()
assert len(excerpt) == failure["excerpt"]["bytes"]
assert hashlib.sha256(excerpt).hexdigest() == failure["excerpt"]["sha256"]
assert b"preverify-selection-tools: error: Unrecognized option: 'lustr-prime-selection-tools'" in excerpt
assert b"Process completed with exit code 101" in excerpt
print("Verified frozen source, four argument-boundary command records and the original CI failure excerpt. Compiler/binary/full-log hashes are recorded identities; those full artifacts are not archived here. No baseline package was built by the local probes.")
