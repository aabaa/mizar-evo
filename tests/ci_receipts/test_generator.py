"""Group 1/2: real full execution with disposable custody, never production keys."""

import importlib.util
import json
import os
import platform
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time
import unittest

from test_verifier import load_verifier, run


CUSTODY = Path(os.environ.get("CI_RECEIPT_CUSTODY", "~/orchestration/ci-receipts/generate.py")).expanduser()


class GeneratorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.custody = self.home / "custody"
        self.custody.mkdir()
        self.generator_path = self.custody / "generate.py"
        shutil.copyfile(CUSTODY, self.generator_path)
        spec = importlib.util.spec_from_file_location("fixture_generator", self.generator_path)
        self.generator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.generator)
        self.repo = self.home / "repo"
        self.repo.mkdir()
        self.git("init", "-b", "main")
        self.git("config", "user.name", "Receipt fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        (self.repo / "src").mkdir()
        hidden_signer = json.dumps(str(self.custody / "signer"))
        (self.repo / "src/lib.rs").write_text(
            '#[test]\nfn complete_gate() {\n'
            f'assert!(!std::path::Path::new({hidden_signer}).exists());\n'
            'assert!(std::fs::OpenOptions::new().write(true).open(".git/config").is_err());\n'
            'unsafe extern "C" { fn lseek(fd: i32, offset: i64, whence: i32) -> i64; }\n'
            'assert_eq!(unsafe { lseek(1, 0, 0) }, -1);\n}\n')
        (self.repo / "Cargo.toml").write_text('[package]\nname="receipt_fixture"\nversion="0.1.0"\nedition="2021"\n')
        run("cargo", "generate-lockfile", cwd=self.repo)
        (self.repo / "README.md").write_text("base\n")
        self.git("add", ".")
        self.git("commit", "-m", "base")
        self.base = self.oid()
        (self.repo / "README.md").write_text("docs\n")
        self.git("commit", "-am", "ordinary docs")
        self.source = self.oid()
        self.key = self.custody / "signer"
        run("ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(self.key))
        tools = {"git": str(Path(shutil.which("git")).resolve()),
                 "bwrap": str(Path(shutil.which("bwrap")).resolve()),
                 "ssh_keygen": str(Path(shutil.which("ssh-keygen")).resolve())}
        for tool in ("cargo", "rustc", "rustdoc"):
            tools[tool] = run("rustup", "which", tool).decode().strip()
        hashes = {name: self.generator.digest(Path(path).read_bytes()) for name, path in tools.items()}
        self.config = {"enabled": True, **tools, "tool_sha256": hashes,
                       "repository_path": str(self.repo), "repository": "fixture/repository",
                       "policy_path": str(self.custody / "policy.json"), "signer_path": str(self.key),
                       "work_root": str(self.custody / "runs"), "ledger_path": str(self.custody / "ledger.jsonl"),
                       "timeout_seconds": 30}
        environment = {"env": {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC",
                               "CARGO_BUILD_JOBS": "4", "CARGO_NET_OFFLINE": "true", "RUSTC": tools["rustc"], "RUSTDOC": tools["rustdoc"],
                               "CARGO_ENCODED_RUSTFLAGS": "", "CARGO_ENCODED_RUSTDOCFLAGS": ""},
                       "cargo_version": run(tools["cargo"], "--version").decode().strip(),
                       "rustc_version": run(tools["rustc"], "-vV").decode().strip(), "tool_sha256": hashes,
                       "sysroot_sha256": self.generator.toolchain_digest(tools["rustc"]), "host": platform.platform(),
                       "lock_sha256": self.generator.digest((self.repo / "Cargo.lock").read_bytes()),
                       "dependencies_sha256": self.generator.digest(self.generator.canonical({}))}
        inventory = ["unittests src/lib.rs (receipt_fixture)|complete_gate"]
        self.policy = {"schema": 1, "enabled": False, "repository": "fixture/repository",
                       "signer_identity": "fixture", "signer_public_key": " ".join(self.key.with_suffix(".pub").read_text().split()[:2]),
                       "plan_sha256": self.generator.digest(self.generator.canonical(self.generator.PLAN)),
                       "generator_sha256": self.generator.digest(self.generator_path.read_bytes()),
                       "environment_sha256": self.generator.digest(self.generator.canonical(environment)),
                       "inventory_sha256": self.generator.digest(self.generator.canonical(inventory)),
                       "expected_inventory": inventory, "expected_ignored": []}
        self.save()

    def git(self, *args):
        return run("git", "-C", str(self.repo), *args)

    def oid(self):
        return self.git("rev-parse", "HEAD").decode().strip()

    def save(self):
        (self.custody / "custody.json").write_bytes(self.generator.canonical(self.config))
        (self.custody / "policy.json").write_bytes(self.generator.canonical(self.policy))

    def execute(self):
        return subprocess.run(["python3", str(self.generator_path), self.source, self.base], capture_output=True)

    def replace_test(self, body, attributes=""):
        (self.repo / "src/lib.rs").write_text(f"#[test]\n{attributes}\nfn complete_gate() {{{body}}}\n")
        self.git("commit", "-am", "fixture gate behavior")
        self.source = self.oid()

    def assert_refused(self, result=None):
        result = result if result is not None else self.execute()
        self.assertNotEqual(result.returncode, 0, result.stdout.decode())
        tag = "refs/tags/ci-receipts/" + self.source
        self.assertNotEqual(subprocess.run(["git", "-C", str(self.repo), "show-ref", "--verify", "--quiet", tag]).returncode, 0)
        self.assertFalse((self.custody / "ledger.jsonl").exists())
        return result

    def test_genuine_full_gate_signature_and_create_only_roundtrip(self):
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        manifest = json.loads(result.stdout)
        self.assertEqual(manifest["counts"], {"passed": 1, "failed": 0, "ignored": 0})
        tag = "refs/tags/ci-receipts/" + self.source
        self.assertEqual(len(self.git("rev-list", "--parents", "-1", tag).split()), 1)
        self.assertEqual(self.git("ls-tree", "--name-only", tag).decode().splitlines(),
                         ["cargo-test.log", "manifest.json", "manifest.sig"])
        self.assertIn(b"complete_gate ... ok", self.git("show", tag + ":cargo-test.log"))
        event = {"kind": "push", "repository": "fixture/repository", "source": self.source,
                 "base": self.base, "source_ref": "refs/heads/main", "default_ref": "refs/heads/main",
                 "default_source": self.source, "fork": False, "synthetic_merge": False}
        decision = load_verifier().verify_receipt(self.repo, self.generator.canonical(self.policy), event)
        self.assertTrue(decision["eligible"])
        self.assertNotEqual(self.execute().returncode, 0)
        self.assertEqual(len((self.custody / "ledger.jsonl").read_text().splitlines()), 1)

    def test_failed_full_execution_has_no_receipt(self):
        self.replace_test('panic!("fixture failure");')
        self.assertIn(b"cargo test failed", self.assert_refused().stderr)

    def test_timeout_has_no_receipt(self):
        self.replace_test("std::thread::sleep(std::time::Duration::from_secs(30));")
        self.config["timeout_seconds"] = 1
        self.save()
        self.assertIn(b"timed out", self.assert_refused().stderr)

    def test_interruption_has_no_receipt(self):
        self.replace_test("std::thread::sleep(std::time::Duration::from_secs(30));")
        process = subprocess.Popen(["python3", str(self.generator_path), self.source, self.base],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                logs = list((self.custody / "runs").glob("*/cargo-test.log"))
                if logs and b"running 1 test" in logs[0].read_bytes():
                    break
                self.assertIsNone(process.poll(), "generator exited before running gate")
                time.sleep(0.01)
            else:
                self.fail("gate did not start")
            process.send_signal(signal.SIGINT)
            stdout, stderr = process.communicate(timeout=5)
            self.assert_refused(subprocess.CompletedProcess(process.args, process.returncode, stdout, stderr))
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate()

    def test_dirty_and_untracked_preflight_has_no_receipt(self):
        for path in ("README.md", "untracked.tmp"):
            with self.subTest(path=path):
                target = self.repo / path
                original = target.read_bytes() if target.exists() else None
                target.write_text("dirty\n")
                self.assertIn(b"dirty", self.assert_refused().stderr)
                if original is None:
                    target.unlink()
                else:
                    target.write_bytes(original)

    def test_edit_and_revert_during_execution_has_no_receipt(self):
        self.replace_test('let before = std::fs::read("README.md").unwrap(); '
                          'std::fs::write("README.md", b"mutation").unwrap(); '
                          'std::fs::write("README.md", before).unwrap();')
        self.assertIn(b"mutation observed", self.assert_refused().stderr)

    def test_empty_and_omitted_inventory_has_no_receipt(self):
        self.policy["expected_inventory"].append("unittests src/lib.rs (receipt_fixture)|omitted")
        self.policy["inventory_sha256"] = self.generator.digest(self.generator.canonical(self.policy["expected_inventory"]))
        self.save()
        self.assertIn(b"inventory", self.assert_refused().stderr)
        (self.repo / "src/lib.rs").write_text("// no tests\n")
        self.git("commit", "-am", "empty suite")
        self.source = self.oid()
        self.assertIn(b"empty", self.assert_refused().stderr)

    def test_ignored_drift_and_missing_baseline_has_no_receipt(self):
        self.replace_test("", "#[ignore]")
        self.assert_refused()
        self.policy["expected_inventory"] = []
        self.policy["inventory_sha256"] = self.generator.digest(self.generator.canonical([]))
        self.save()
        self.assert_refused()

    def test_filters_and_synthetic_empty_output_cannot_select_gate(self):
        result = subprocess.run(["python3", str(self.generator_path), self.source, self.base, "complete_gate"], capture_output=True)
        self.assert_refused(result)
        for log in (b"", b"test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.0s\n"):
            with self.subTest(log=log), self.assertRaises(ValueError):
                self.generator.parse_log(log)
        fake = self.custody / "fake-cargo"
        fake.write_text("#!/bin/sh\nprintf 'synthetic success\\n'\n")
        fake.chmod(0o755)
        self.config["cargo"] = str(fake)
        self.save()
        self.assertIn(b"tool is not", self.assert_refused().stderr)

    def test_doctest_batches_require_each_complete_summary(self):
        first = (b"   Doc-tests fixture\n\nrunning 1 test\n"
                 b"test src/lib.rs - example (line 1) ... ok\n\n"
                 b"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n")
        second = (b"\nrunning 1 test\n"
                  b"test src/lib.rs - example (line 9) - compile fail ... ok\n\n"
                  b"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n")
        inventory, ignored, counts = self.generator.parse_log(first + second)
        self.assertEqual(inventory, ["Doc-tests fixture|src/lib.rs - example (line 1)",
                                     "Doc-tests fixture|src/lib.rs - example (line 9) - compile fail"])
        self.assertEqual(ignored, [])
        self.assertEqual(counts["passed"], 2)
        for incomplete in (first + second[:second.index(b"test result:")],
                           first + b"\n   Doc-tests missing\n\nrunning 0 tests\n",
                           first + b"\n   Doc-tests missing\n"):
            with self.subTest(incomplete=incomplete), self.assertRaises(ValueError):
                self.generator.parse_log(incomplete)

    def test_same_named_workspace_targets_keep_separate_inventory_entries(self):
        def batch(build_hash):
            return (f"     Running tests/lint_policy.rs (/target/lint_policy-{build_hash})\n"
                    "running 1 test\ntest same_test ... ok\n"
                    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n").encode()
        inventory, _, counts = self.generator.parse_log(batch("aabb") + batch("ccdd"))
        self.assertEqual(inventory, ["tests/lint_policy.rs (lint_policy) #2|same_test",
                                     "tests/lint_policy.rs (lint_policy)|same_test"])
        self.assertEqual(counts["passed"], 2)

    def test_atomic_publication_and_remote_create_only_refusal(self):
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        remote = self.home / "remote.git"
        run("git", "init", "--bare", str(remote))
        self.config.update(publish_ref="refs/heads/main", publish_remote=str(remote))
        self.save()
        command = ["python3", str(self.generator_path), "--publish", self.source, "refs/heads/main"]
        result = subprocess.run(command, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        tag = "refs/tags/ci-receipts/" + self.source
        self.assertEqual(run("git", "-C", str(remote), "rev-parse", "refs/heads/main").decode().strip(), self.source)
        self.assertEqual(run("git", "-C", str(remote), "rev-parse", tag), self.git("rev-parse", tag))
        self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)
        ledger = self.custody / "ledger.jsonl"
        # A remote tag from a prior publication cannot be republished even after
        # reconstructing the local pending record, as disaster recovery might do.
        ledger.write_text(ledger.read_text().splitlines()[0] + "\n")
        refused = subprocess.run(command, capture_output=True)
        self.assertNotEqual(refused.returncode, 0)
        self.assertIn(b"remote evidence tag already exists", refused.stderr)


if __name__ == "__main__":
    unittest.main()
