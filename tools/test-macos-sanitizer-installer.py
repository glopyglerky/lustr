#!/usr/bin/env python3
"""Reject corrupt package inputs before extraction or compiler execution."""

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location('installer', Path(__file__).with_name('install-macos-sanitizer.py'))
INSTALLER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALLER)


class PackageInputTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.path = self.root / 'archive'
        self.content = b'exact reviewed upstream package bytes'
        self.path.write_bytes(self.content)
        self.pin = {'bytes': len(self.content), 'sha256': hashlib.sha256(self.content).hexdigest()}

    def test_exact_bytes_are_accepted(self):
        INSTALLER.verify_file(self.path, self.pin)

    def test_same_size_corruption_is_rejected(self):
        self.path.write_bytes(b'X' + self.content[1:])
        with self.assertRaisesRegex(ValueError, 'SHA-256'):
            INSTALLER.verify_file(self.path, self.pin)

    def test_truncated_input_is_rejected(self):
        self.path.write_bytes(self.content[:-1])
        with self.assertRaisesRegex(ValueError, 'size'):
            INSTALLER.verify_file(self.path, self.pin)

    def test_symlink_cannot_substitute_external_input(self):
        linked = self.root / 'linked'
        linked.symlink_to(self.path)
        with self.assertRaisesRegex(ValueError, 'regular file'):
            INSTALLER.verify_file(linked, self.pin)

    def test_wrong_package_never_reaches_provenance_or_extraction(self):
        pin = json.loads(INSTALLER.PIN.read_bytes())
        (self.root / pin['archive']['name']).write_bytes(b'wrong package')
        with mock.patch.object(INSTALLER.platform, 'system', return_value='Darwin'), \
             mock.patch.object(INSTALLER.platform, 'machine', return_value='arm64'), \
             mock.patch.object(INSTALLER.shutil, 'disk_usage', return_value=mock.Mock(free=100 * 1024**3)), \
             mock.patch.object(INSTALLER, 'run_recorded') as execute, \
             mock.patch.object(INSTALLER, 'extract') as extract:
            with self.assertRaisesRegex(ValueError, 'size'):
                INSTALLER.install(self.root)
            execute.assert_not_called()
            extract.assert_not_called()

    def test_previous_failure_output_is_preserved_before_any_retry(self):
        previous = self.root / 'attestation-verification.stderr'
        previous.write_bytes(b'previous rejected provenance')
        with mock.patch.object(INSTALLER.subprocess, 'run') as execute:
            with self.assertRaisesRegex(ValueError, 'evidence already exists'):
                INSTALLER.run_recorded(['unused'], self.root, 'attestation-verification', 1)
            execute.assert_not_called()
        self.assertEqual(previous.read_bytes(), b'previous rejected provenance')

    def test_timed_out_command_preserves_partial_output_and_timeout_status(self):
        command = [sys.executable, '-c',
                   "import os,time; os.write(1,b'partial stdout\\n'); "
                   "os.write(2,b'partial stderr\\n'); time.sleep(30)"]
        with self.assertRaises(subprocess.TimeoutExpired):
            INSTALLER.run_recorded(command, self.root, 'timed-out', 1)
        self.assertEqual((self.root / 'timed-out.stdout').read_bytes(), b'partial stdout\n')
        self.assertEqual((self.root / 'timed-out.stderr').read_bytes(), b'partial stderr\n')
        self.assertEqual(json.loads((self.root / 'timed-out.status.json').read_bytes()),
                         {'exit': None, 'signal': None, 'timed_out': True, 'timeout_seconds': 1})


if __name__ == '__main__':
    unittest.main()
