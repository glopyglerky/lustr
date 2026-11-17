#!/usr/bin/env python3
"""Retain original coordinator evidence using OpenSSL CMS authenticated encryption.

No key generation, plaintext publication, extraction, or custom cryptography.
The recipient fingerprint is explicit; the private key never enters this tool.
"""
import argparse
from dataclasses import dataclass
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile

SCENARIOS = ("clean-dependency-build", "primed-root-build", "primed-root-check", "no-op", "compile-plus-test")
ROOT_FILES = ("identity.json", "host.json", "campaign-result.json", "reject.rs")
SCENARIO_FILES = {"report.json", "report.pending", "spec.original.json", "spec.original.pending"}
STREAM_FILE = re.compile(r"[0-9]{5,}\.(?:stdout|stderr|request\.(?:json|pending)|usage\.(?:json|pending))")
MANIFEST = "MANIFEST.json"
CIPHERTEXT = "private-evidence.tar.gz.cms"
RECEIPT = "receipt.json"
STATUS = "status.json"


class RetentionError(Exception):
    """Only fixed categories may reach a public status or command diagnostic."""


def require(condition, category):
    if not condition:
        raise RetentionError(category)


def digest_stream(stream):
    digest = hashlib.sha256()
    size = 0
    while block := stream.read(1024 * 1024):
        digest.update(block)
        size += len(block)
    return digest.hexdigest(), size


def digest(path):
    with path.open("rb") as stream:
        return digest_stream(stream)[0]


def original_file(path):
    # Do not follow a substituted evidence file into unrelated private material.
    require(stat.S_ISREG(path.lstat().st_mode), "unsupported-evidence-entry")
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    stream = os.fdopen(descriptor, "rb")
    if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
        stream.close()
        raise RetentionError("unsupported-evidence-entry")
    return stream


def directory(path):
    metadata = path.lstat()
    require(stat.S_ISDIR(metadata.st_mode) and not path.is_symlink(), "unsupported-evidence-entry")


@dataclass(frozen=True)
class Entry:
    path: Path
    name: str
    sha256: str
    size: int
    mode: int

    def row(self):
        return {"name": self.name, "sha256": self.sha256, "bytes": self.size, "mode": self.mode}


def inventory(private):
    directory(private)
    selected = []

    def include(path):
        with original_file(path) as stream:
            before = os.fstat(stream.fileno())
            checksum, size = digest_stream(stream)
            after = os.fstat(stream.fileno())
        require((before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
                (after.st_size, after.st_mtime_ns, after.st_ctime_ns) and size == before.st_size,
                "evidence-changed-during-retention")
        selected.append(Entry(path, "private/" + path.relative_to(private).as_posix(), checksum, size, stat.S_IMODE(before.st_mode)))

    def tree(path):
        directory(path)
        for child in sorted(path.iterdir()):
            metadata = child.lstat()
            if stat.S_ISDIR(metadata.st_mode):
                tree(child)
            else:
                include(child)

    for name in ROOT_FILES:
        path = private / name
        if path.exists() or path.is_symlink():
            include(path)
    snapshot = private / "snapshot"
    if snapshot.exists() or snapshot.is_symlink():
        tree(snapshot)
    specs = private / "specs"
    if specs.exists() or specs.is_symlink():
        directory(specs)
        for name in SCENARIOS:
            for suffix in (".json", ".pending"):
                path = specs / (name + suffix)
                if path.exists() or path.is_symlink():
                    include(path)
    for name in SCENARIOS:
        for stream in ("stdout", "stderr"):
            path = private / f"{name}.runner.{stream}"
            if path.exists() or path.is_symlink():
                include(path)
        root = private / name
        if root.exists() or root.is_symlink():
            directory(root)
            # Only direct runner evidence is admitted. Never recurse build/cache
            # state or include arbitrary binaries copied into a report directory.
            for path in sorted(root.iterdir()):
                if path.name in SCENARIO_FILES or STREAM_FILE.fullmatch(path.name):
                    include(path)
    return sorted(selected, key=lambda entry: entry.name)


def archive(entries, path):
    manifest = {"schema": "lustr-private-evidence-manifest-v1", "files": [entry.row() for entry in entries]}
    manifest_bytes = (json.dumps(manifest, sort_keys=True, indent=2, allow_nan=False) + "\n").encode()
    # Stable metadata avoids leaking host account names in archive headers and
    # allows the same original bytes to produce the same independently checkable
    # plaintext archive hash. File content and permission bits remain exact.
    with path.open("xb") as raw:
        os.fchmod(raw.fileno(), 0o600)
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w|", format=tarfile.PAX_FORMAT) as output:
                metadata = tarfile.TarInfo(MANIFEST)
                metadata.mode, metadata.size = 0o600, len(manifest_bytes)
                output.addfile(metadata, io.BytesIO(manifest_bytes))
                for entry in entries:
                    metadata = tarfile.TarInfo(entry.name)
                    metadata.mode, metadata.size = entry.mode, entry.size
                    with original_file(entry.path) as original:
                        output.addfile(metadata, original)
        raw.flush()
        os.fsync(raw.fileno())
    return hashlib.sha256(manifest_bytes).hexdigest()


def verify_archive(path, receipt):
    """Verify bytes, exact file set and safe regular-file names before extraction."""
    require(receipt.get("schema") == "lustr-encrypted-evidence-receipt-v1", "invalid-retention-receipt")
    require(path.stat().st_size == receipt["plaintext_archive_bytes"]
            and digest(path) == receipt["plaintext_archive_sha256"], "plaintext-archive-digest-mismatch")
    with tarfile.open(path, "r|gz") as archive_input:
        members = iter(archive_input)
        first = next(members, None)
        require(first is not None and first.name == MANIFEST and first.isfile() and first.size <= 16 * 1024 * 1024,
                "invalid-private-manifest")
        manifest_bytes = archive_input.extractfile(first).read()
        require(hashlib.sha256(manifest_bytes).hexdigest() == receipt["encrypted_manifest_sha256"], "private-manifest-digest-mismatch")
        manifest = json.loads(manifest_bytes)
        require(set(manifest) == {"schema", "files"} and manifest["schema"] == "lustr-private-evidence-manifest-v1",
                "invalid-private-manifest")
        rows = manifest["files"]
        require(len(rows) == receipt["original_file_count"], "private-manifest-count-mismatch")
        names, total = set(), 0
        for expected in rows:
            require(set(expected) == {"name", "sha256", "bytes", "mode"}, "invalid-private-manifest")
            name = PurePosixPath(expected["name"])
            require(not name.is_absolute() and ".." not in name.parts and name.parts[0] == "private"
                    and expected["name"] not in names, "invalid-private-manifest-path")
            names.add(expected["name"])
            member = next(members, None)
            require(member is not None and member.isfile() and member.name == expected["name"]
                    and member.size == expected["bytes"] and member.mode == expected["mode"], "private-archive-entry-mismatch")
            checksum, size = digest_stream(archive_input.extractfile(member))
            require(checksum == expected["sha256"] and size == expected["bytes"], "private-archive-content-mismatch")
            total += size
        require(next(members, None) is None and total == receipt["original_file_bytes"], "private-archive-count-mismatch")


def openssl(executable, arguments, *, input=None, failure):
    try:
        result = subprocess.run([str(executable), *map(str, arguments)], input=input, capture_output=True, check=False)
    except OSError as error:
        raise RetentionError("openssl-unavailable") from error
    require(result.returncode == 0, failure)
    return result.stdout


def recipient(executable, certificate, expected_fingerprint):
    canonical = openssl(executable, ["x509", "-in", certificate, "-outform", "DER"], failure="invalid-recipient-certificate")
    checksum = hashlib.sha256(canonical).hexdigest()
    require(checksum == expected_fingerprint, "recipient-fingerprint-mismatch")
    pem = openssl(executable, ["x509", "-in", certificate, "-pubkey", "-noout"], failure="invalid-recipient-certificate")
    key = openssl(executable, ["pkey", "-pubin", "-outform", "DER"], input=pem, failure="invalid-recipient-key")
    description = openssl(executable, ["rsa", "-pubin", "-text", "-noout"], input=pem, failure="recipient-is-not-rsa")
    bits = re.search(rb"Public-Key: \(([0-9]+) bit\)", description)
    require(bits is not None and int(bits[1]) >= 2048, "recipient-rsa-key-too-small")
    return {"recipient_certificate_sha256": checksum, "recipient_spki_sha256": hashlib.sha256(key).hexdigest(),
            "recipient_rsa_bits": int(bits[1])}


def observed_status(path, allowed):
    if not path.is_file() or path.is_symlink():
        return "not-observed"
    try:
        value = json.loads(path.read_bytes()).get("status")
    except (ValueError, OSError, AttributeError):
        return "incomplete"
    return value if value in allowed else "incomplete"


def status(private):
    observed = private.is_dir() and not private.is_symlink()
    scenarios = {}
    for name in SCENARIOS:
        root = private / name
        if not observed:
            scenarios[name] = "not-observed"
        elif root.is_symlink():
            scenarios[name] = "incomplete"
        else:
            value = observed_status(root / "report.json", ("passed", "failed"))
            attempted = root.exists() or any((private / f"{name}.runner.{stream}").exists() for stream in ("stdout", "stderr"))
            scenarios[name] = "incomplete" if value == "not-observed" and attempted else value
    return {"schema": "lustr-encrypted-evidence-status-v1", "retention_status": "not-retained", "reason": "retention-not-complete",
            "campaign_status": observed_status(private / "campaign-result.json", ("passed", "failed", "prepared-only")) if observed else "not-observed",
            "scenario_status": scenarios}


def put_json(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())


def seal(campaign, certificate, output, expected_fingerprint, executable):
    private = campaign / "private"
    outcome = status(private)
    published = []
    output.mkdir(mode=0o700)
    try:
        require(private.exists(), "private-evidence-absent")
        require(not output.is_relative_to(private) and not private.is_relative_to(output), "invalid-output-scope")
        entries = inventory(private)
        require(entries, "private-evidence-empty")
        public_recipient = recipient(executable, certificate, expected_fingerprint)
        version = openssl(executable, ["version"], failure="openssl-unavailable").decode("ascii", errors="strict")
        version = re.match(r"OpenSSL ([0-9]+\.[0-9]+\.[0-9]+[a-z]?)\b", version)
        require(version is not None and int(version[1].split(".")[0]) >= 3, "unsupported-openssl-version")
        # The staging directory, plaintext archive and any partial ciphertext
        # remain inside private/ and are removed on ordinary success or failure.
        with tempfile.TemporaryDirectory(prefix="retention-staging-", dir=private) as temporary:
            stage = Path(temporary)
            plaintext = stage / "private-evidence.tar.gz"
            manifest_hash = archive(entries, plaintext)
            require(inventory(private) == entries, "evidence-changed-during-retention")
            receipt = {"schema": "lustr-encrypted-evidence-receipt-v1", "ciphertext_file": CIPHERTEXT,
                       "plaintext_archive_format": "gzip-compressed POSIX tar; original bytes plus encrypted hash manifest",
                       "plaintext_archive_sha256": digest(plaintext), "plaintext_archive_bytes": plaintext.stat().st_size,
                       "encrypted_manifest_sha256": manifest_hash, "original_file_count": len(entries),
                       "original_file_bytes": sum(entry.size for entry in entries), **public_recipient,
                       "openssl_version": version[0], "cms_content_type": "id-smime-ct-authEnvelopedData",
                       "cms_encoding": "DER", "content_cipher": "AES-256-GCM", "key_transport": "RSA-OAEP",
                       "oaep_digest": "SHA-256", "mgf1_digest": "SHA-256", "binary_content": True,
                       "scope": "original root evidence, specs, frozen source, scenario direct evidence and runner streams; build/cache/home state excluded"}
            verify_archive(plaintext, receipt)
            ciphertext = stage / CIPHERTEXT
            openssl(executable, ["cms", "-encrypt", "-binary", "-aes-256-gcm", "-in", plaintext,
                                 "-outform", "DER", "-out", ciphertext, "-recip", certificate,
                                 "-keyopt", "rsa_padding_mode:oaep", "-keyopt", "rsa_oaep_md:sha256",
                                 "-keyopt", "rsa_mgf1_md:sha256"], failure="cms-encryption-failed")
            openssl(executable, ["cms", "-cmsout", "-inform", "DER", "-in", ciphertext, "-noout"], failure="invalid-cms-output")
            require(ciphertext.stat().st_size > 0, "empty-cms-output")
            receipt.update(ciphertext_sha256=digest(ciphertext), ciphertext_bytes=ciphertext.stat().st_size)
            outcome.update(retention_status="retained", reason="authenticated-ciphertext-created", receipt_file=RECEIPT,
                           ciphertext_file=CIPHERTEXT, original_file_count=len(entries), original_file_bytes=receipt["original_file_bytes"])
            put_json(stage / RECEIPT, receipt)
            put_json(stage / STATUS, outcome)
            # Hard links do not overwrite an earlier archive/receipt. Nothing
            # destined for encrypted/ is linked until encryption and checks pass.
            for name in (CIPHERTEXT, RECEIPT, STATUS):
                os.link(stage / name, output / name)
                published.append(output / name)
        descriptor = os.open(output, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
        return outcome
    except Exception as error:
        for path in published:
            path.unlink(missing_ok=True)
        category = str(error) if isinstance(error, RetentionError) else "retention-operation-failed"
        outcome = status(private)
        outcome["reason"] = category
        put_json(output / STATUS, outcome)
        raise RetentionError(category) from None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("campaign", "certificate", "output", "verify-archive", "receipt"):
        parser.add_argument(f"--{name}", type=Path)
    parser.add_argument("--expected-certificate-sha256", help="trusted certificate DER SHA256; colons are accepted")
    parser.add_argument("--openssl", type=Path, default=Path(shutil.which("openssl") or "/usr/bin/openssl"))
    args = parser.parse_args()
    try:
        if args.verify_archive is not None:
            require(args.receipt is not None and args.campaign is None and args.certificate is None and args.output is None,
                    "invalid-verification-arguments")
            verify_archive(args.verify_archive, json.loads(args.receipt.read_bytes()))
            print("Private archive bytes and encrypted manifest verified; extraction has not run.")
        else:
            require(all(value is not None for value in (args.campaign, args.certificate, args.output, args.expected_certificate_sha256)),
                    "encryption-arguments-required")
            expected = args.expected_certificate_sha256.replace(":", "").lower()
            require(re.fullmatch(r"[0-9a-f]{64}", expected), "invalid-recipient-fingerprint")
            require(args.campaign.is_absolute() and args.output.is_absolute(), "absolute-campaign-and-output-required")
            campaign = args.campaign.resolve()
            # A setup failure may precede the campaign directory itself. Creating
            # only its retention status must not pretend private originals exist.
            campaign.mkdir(mode=0o700, exist_ok=True)
            output = args.output.parent.resolve(strict=True) / args.output.name
            seal(campaign, args.certificate, output, expected, args.openssl)
            print("Original private evidence retained as authenticated ciphertext.")
        return 0
    except Exception as error:
        category = str(error) if isinstance(error, RetentionError) else "retention-operation-failed"
        print(f"Evidence retention failed: {category}.", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
