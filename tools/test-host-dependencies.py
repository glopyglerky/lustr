#!/usr/bin/env python3
"""Plant dependencies in real checkout paths and require the public guard to fail."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


SOURCE = Path(os.environ.get("LUSTR_HOST_DEPENDENCY_GUARD",
                            Path(__file__).with_name("check-host-dependencies.py")))


class HostDependencies(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="lustr-host-inputs-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True,
                       capture_output=True, timeout=10)
        for name, source in {
            "Cargo.toml": '[package]\nname = "fixture"\nversion = "0.1.0"\n',
            "compiler/native/Cargo.lock": 'version = 4\n[[package]]\nname = "fixture"\nversion = "0.1.0"\n',
            "compiler/native/src/client.rs": "fn main() {}\n",
            "compiler/native/src/daemon.rs": "fn daemon() {}\n",
            "src/host_runtime.rs": "fn runtime() {}\n",
            ".github/workflows/ci.yml": "name: fixture\n",
        }.items():
            self.write(name, source)
        (self.root / "tools").mkdir()
        shutil.copyfile(SOURCE, self.root / "tools/check-host-dependencies.py")

    def write(self, name, source):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source)
        return path

    def audit(self):
        result = subprocess.run([sys.executable, str(self.root / "tools/check-host-dependencies.py")],
                                cwd=self.root, capture_output=True, text=True, timeout=10)
        self.assertIn(result.returncode, (0, 1), result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(result.returncode, 0 if report["passed"] else 1)
        return report

    def test_clean_checkout_and_historical_records(self):
        self.write("docs/compiler-set/evidence/original.sh", "systemctl start original\n")
        self.write("performance/original.py", 'dependency = "libsystemd"\n')
        self.write("README.md", "The compiler does not require systemd.\n")
        self.assertTrue(self.audit()["passed"])

    def test_runtime_library_and_example_cannot_escape_directory_scope(self):
        for name in ("library/lustr/std/src/platform.rs", "examples/native_package.rs"):
            with self.subTest(path=name):
                path = self.write(name, 'std::process::Command::new("systemctl");\n')
                report = self.audit()
                self.assertFalse(report["passed"])
                self.assertTrue(any(v["path"] == name for v in report["violations"]))
                path.unlink()

    def test_new_package_directory_and_root_install_script(self):
        for name in ("packaging/install.sh", "install.sh", "packaging/setup.ps1"):
            with self.subTest(path=name):
                path = self.write(name, "systemctl --user enable compiler\n")
                self.assertFalse(self.audit()["passed"])
                path.unlink()

    def test_extensionless_entry_and_build_recipes(self):
        for name, source in {
            "bin/lustr-start": "#!/bin/sh\nexec systemd-run compiler\n",
            "Makefile": "install:\n\tsystemctl start compiler\n",
            "Dockerfile.native": "RUN apt-get install libsystemd-dev\n",
            "CMakeLists.txt": "find_package(libsystemd REQUIRED)\n",
        }.items():
            with self.subTest(path=name):
                path = self.write(name, source)
                self.assertFalse(self.audit()["passed"])
                path.unlink()

    def test_unit_files_are_rejected_without_forbidden_content(self):
        for suffix in ("service", "socket", "timer", "path", "slice", "scope",
                       "target", "mount", "automount", "swap", "device"):
            with self.subTest(unit=suffix):
                path = self.write("packaging/compiler." + suffix, "[Unit]\nDescription=Compiler\n")
                report = self.audit()
                self.assertFalse(report["passed"])
                self.assertTrue(any(v["token"] == "service-manager unit file"
                                    for v in report["violations"]))
                path.unlink()

    def test_library_apis_and_activation_environment(self):
        for token in ("sd_bus_open_user", "sd_journal_send", "sd_event_default",
                      "sd_pid_get_session", "sd_notify", "sd_listen_fds", "LISTEN_FDNAMES",
                      "NOTIFY_SOCKET"):
            with self.subTest(token=token):
                path = self.write("src/service.c", f"{token};\n")
                self.assertFalse(self.audit()["passed"])
                path.unlink()

    def test_dependency_lockfile_is_scanned_in_new_package(self):
        self.write("packaging/sdk/Cargo.lock",
                   'version = 4\n[[package]]\nname = "libsystemd"\nversion = "0.1.0"\n')
        report = self.audit()
        self.assertFalse(report["passed"])
        self.assertIn("libsystemd", report["lockfile_packages"])

    def test_missing_compiler_does_not_pass(self):
        (self.root / "compiler/native/src/daemon.rs").unlink()
        report = self.audit()
        self.assertFalse(report["passed"])
        self.assertEqual(report["missing_required_files"], ["compiler/native/src/daemon.rs"])


if __name__ == "__main__":
    unittest.main()
