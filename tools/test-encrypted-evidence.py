#!/usr/bin/env python3
"""Actual OpenSSL round trips and evidence-scope tests with ephemeral test keys."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

module = importlib.util.spec_from_file_location("encrypted_evidence", Path(__file__).with_name("encrypt-coordinator-evidence.py"))
evidence = importlib.util.module_from_spec(module)
# dataclasses resolves the module through sys.modules.
import sys
sys.modules[module.name] = evidence
module.loader.exec_module(evidence)


class EncryptionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.keys = tempfile.TemporaryDirectory(prefix="lustr-test-keys-")
        cls.key_root = Path(cls.keys.name)
        cls.openssl = Path(evidence.shutil.which("openssl"))
        for name in ("recipient", "wrong"):
            result = subprocess.run([str(cls.openssl), "req", "-x509", "-newkey", "rsa:2048", "-noenc",
                                     "-keyout", str(cls.key_root / f"{name}.key"), "-out", str(cls.key_root / f"{name}.pem"),
                                     "-subj", "/CN=Lustr ephemeral test recipient", "-days", "1"], capture_output=True)
            if result.returncode:
                raise RuntimeError("OpenSSL test-key generation failed")
        cls.certificate = cls.key_root / "recipient.pem"
        cls.key = cls.key_root / "recipient.key"
        der = subprocess.run([str(cls.openssl), "x509", "-in", str(cls.certificate), "-outform", "DER"], check=True, capture_output=True).stdout
        cls.fingerprint = hashlib.sha256(der).hexdigest()

    @classmethod
    def tearDownClass(cls):
        cls.keys.cleanup()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="private-evidence-test-")
        self.campaign = Path(self.temporary.name)
        self.private = self.campaign / "private"
        self.private.mkdir(mode=0o700)
        self.output = self.campaign / "encrypted"

    def tearDown(self):
        self.temporary.cleanup()

    def original(self, relative, content):
        path = self.private / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def fixture(self, complete=False):
        selected = {
            "identity.json": b'{"private_path":"/private/secret/source"}\n',
            "host.json": b'{"private_host_fixture":"not published"}\n',
            "campaign-result.json": json.dumps({"status": "passed" if complete else "failed", "error": "/private/secret/exception"}).encode(),
            "reject.rs": b"invalid Rust fixture bytes\x00\xff\n",
            "snapshot/src/lib.rs": b"// frozen source; preserve these bytes\n",
            "snapshot/Cargo.lock": b"version = 4\n",
            "specs/no-op.json": b'{"private_arg":"/private/secret/command"}\n',
            "no-op.runner.stdout": b"/private/secret/report.json\n",
            "no-op.runner.stderr": b"original failure diagnostic\n",
            "no-op/spec.original.json": b'{"exact_spec":true}\n',
            "no-op/00000.stdout": b"binary output\x00\xff\x80\r\n",
            "no-op/00000.stderr": b"original compiler stderr\n",
            "no-op/report.json": json.dumps({"status": "passed" if complete else "failed", "error": "/private/secret/error"}).encode(),
            "primed-root-build/report.pending": b'{"unfinished":',
            "primed-root-build/spec.original.pending": b'{"unfinished_spec":',
            "primed-root-build/00000.stdout": b"partial output from killed command",
        }
        for name, content in selected.items():
            self.original(name, content)
        for excluded in ("home/private-config", "cargo-home/registry/cache", "no-op/targets/debug/lustr",
                         "no-op/targets/native-cache/object", "no-op/mixed-cache/certificate", "no-op/compiler-binary",
                         "unrelated.key", "sanitized-samples.json", "specs/unrelated.json"):
            self.original(excluded, b"\x7fELF excluded data MUST NOT be archived")
        return {"private/" + name: content for name, content in selected.items()}

    def seal(self):
        return evidence.seal(self.campaign, self.certificate, self.output, self.fingerprint, self.openssl)

    def decrypt(self, ciphertext=None, key=None):
        """Never treat OpenSSL's output file as authenticated until exit is zero."""
        output = self.private / "recovered.tar.gz.pending"
        result = subprocess.run([str(self.openssl), "cms", "-decrypt", "-binary", "-inform", "DER", "-in",
                                 str(ciphertext or self.output / evidence.CIPHERTEXT), "-recip", str(self.certificate),
                                 "-inkey", str(key or self.key), "-out", str(output)], capture_output=True)
        if result.returncode:
            output.unlink(missing_ok=True)
            return result.returncode, None
        receipt = json.loads((self.output / evidence.RECEIPT).read_bytes())
        try:
            evidence.verify_archive(output, receipt)
        except Exception:
            output.unlink(missing_ok=True)
            raise
        return 0, output

    def test_failed_partial_archive_preserves_exact_allowed_original_bytes(self):
        expected = self.fixture()
        status = self.seal()
        self.assertEqual(status["retention_status"], "retained")
        self.assertEqual(status["campaign_status"], "failed")
        self.assertEqual(status["scenario_status"]["no-op"], "failed")
        self.assertEqual(status["scenario_status"]["primed-root-build"], "incomplete")
        code, archive = self.decrypt()
        self.assertEqual(code, 0)
        with tarfile.open(archive) as recovered:
            actual = {member.name: recovered.extractfile(member).read() for member in recovered if member.name != evidence.MANIFEST}
        self.assertEqual(actual, expected)
        receipt = json.loads((self.output / evidence.RECEIPT).read_bytes())
        self.assertEqual(receipt["original_file_count"], len(expected))
        self.assertEqual(receipt["original_file_bytes"], sum(map(len, expected.values())))
        self.assertEqual(receipt["ciphertext_sha256"], evidence.digest(self.output / evidence.CIPHERTEXT))
        self.assertEqual(receipt["recipient_certificate_sha256"], self.fingerprint)
        self.assertFalse(list(self.private.glob("retention-staging-*")))

    def test_actual_cms_contains_auth_envelope_gcm_and_explicit_sha256_oaep(self):
        self.fixture(complete=True)
        self.seal()
        result = subprocess.run([str(self.openssl), "cms", "-cmsout", "-print", "-inform", "DER", "-in",
                                 str(self.output / evidence.CIPHERTEXT)], check=True, capture_output=True)
        structure = result.stdout.decode()
        for marker in ("id-smime-ct-authEnvelopedData", "aes-256-gcm", "rsaesOaep", "mgf1"):
            self.assertIn(marker, structure)
        self.assertEqual(structure.count("OBJECT            :sha256"), 2)

    def test_wrong_private_key_fails_and_rejected_output_is_removed(self):
        self.fixture()
        self.seal()
        code, path = self.decrypt(key=self.key_root / "wrong.key")
        self.assertNotEqual(code, 0)
        self.assertIsNone(path)
        self.assertFalse((self.private / "recovered.tar.gz.pending").exists())

    def test_modified_authentication_tag_fails_and_rejected_output_is_removed(self):
        self.fixture()
        self.seal()
        payload = bytearray((self.output / evidence.CIPHERTEXT).read_bytes())
        payload[-1] ^= 1  # The unadorned CMS AuthEnvelopedData ends in its GCM MAC.
        changed = self.private / "tampered.cms"
        changed.write_bytes(payload)
        code, path = self.decrypt(ciphertext=changed)
        self.assertNotEqual(code, 0)
        self.assertIsNone(path)
        self.assertFalse((self.private / "recovered.tar.gz.pending").exists())

    def test_archive_hash_and_encrypted_file_manifest_are_both_required(self):
        self.fixture()
        self.seal()
        _, path = self.decrypt()
        receipt = json.loads((self.output / evidence.RECEIPT).read_bytes())
        receipt["plaintext_archive_sha256"] = "0" * 64
        with self.assertRaises(evidence.RetentionError):
            evidence.verify_archive(path, receipt)
        receipt["plaintext_archive_sha256"] = evidence.digest(path)
        receipt["encrypted_manifest_sha256"] = "0" * 64
        with self.assertRaises(evidence.RetentionError):
            evidence.verify_archive(path, receipt)

    def test_public_receipt_and_status_are_path_free_closed_projections(self):
        self.fixture()
        self.seal()
        self.assertEqual({path.name for path in self.output.iterdir()}, {evidence.CIPHERTEXT, evidence.RECEIPT, evidence.STATUS})
        for filename in (evidence.RECEIPT, evidence.STATUS):
            text = (self.output / filename).read_text()
            for secret in (str(self.campaign), "/private/secret", "private_host_fixture", "private_arg", "exception"):
                self.assertNotIn(secret, text)
        self.assertFalse(any(path.suffix == ".gz" for path in self.output.iterdir()))

    def test_missing_private_evidence_is_not_reported_as_retained(self):
        self.private.rmdir()
        with self.assertRaisesRegex(evidence.RetentionError, "private-evidence-absent"):
            self.seal()
        self.assertEqual({path.name for path in self.output.iterdir()}, {evidence.STATUS})
        result = json.loads((self.output / evidence.STATUS).read_bytes())
        self.assertEqual(result["retention_status"], "not-retained")
        self.assertEqual(result["reason"], "private-evidence-absent")

    def test_partial_setup_without_json_results_is_retained(self):
        self.original("snapshot/src/lib.rs", b"part of a frozen snapshot")
        self.original("campaign-result.json", b'{"status":')
        result = self.seal()
        self.assertEqual(result["campaign_status"], "incomplete")
        self.assertEqual(result["original_file_count"], 2)
        self.assertEqual(self.decrypt()[0], 0)

    def test_wrong_recipient_pin_cannot_publish_a_ciphertext_or_receipt(self):
        self.fixture()
        with self.assertRaisesRegex(evidence.RetentionError, "recipient-fingerprint-mismatch"):
            evidence.seal(self.campaign, self.key_root / "wrong.pem", self.output, self.fingerprint, self.openssl)
        self.assertEqual({path.name for path in self.output.iterdir()}, {evidence.STATUS})
        self.assertFalse(list(self.private.glob("retention-staging-*")))

    def test_symlinked_original_is_rejected_without_following_it(self):
        self.fixture()
        path = self.private / "snapshot/src/lib.rs"
        path.unlink()
        path.symlink_to(self.key)
        with self.assertRaisesRegex(evidence.RetentionError, "unsupported-evidence-entry"):
            self.seal()
        self.assertEqual({path.name for path in self.output.iterdir()}, {evidence.STATUS})

    def test_existing_evidence_cannot_be_overwritten(self):
        self.fixture()
        self.seal()
        before = {path.name: path.read_bytes() for path in self.output.iterdir()}
        with self.assertRaises(FileExistsError):
            self.seal()
        self.assertEqual(before, {path.name: path.read_bytes() for path in self.output.iterdir()})

    def test_cli_failure_emits_only_allowlisted_status_and_no_raw_exception(self):
        self.private.rmdir()
        result = subprocess.run([sys.executable, str(Path(module.origin)), "--campaign", str(self.campaign),
                                 "--certificate", str(self.certificate), "--output", str(self.output),
                                 "--expected-certificate-sha256", self.fingerprint], capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"")
        self.assertEqual(result.stderr, b"Evidence retention failed: private-evidence-absent.\n")
        self.assertNotIn(str(self.campaign).encode(), (self.output / evidence.STATUS).read_bytes())


if __name__ == "__main__":
    unittest.main()
