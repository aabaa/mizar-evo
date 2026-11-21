"""Disposable Git and signing fixtures for the receipt trust boundary."""

import hashlib
import importlib.util
import datetime as dt
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]


def run(*args, cwd=None, data=None):
    return subprocess.run(args, cwd=cwd, input=data, check=True, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE).stdout


def load_verifier():
    spec = importlib.util.spec_from_file_location("receipt_verifier", ROOT / ".github/ci-receipts/verify.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.repo = self.home / "repo"
        self.repo.mkdir()
        self.git("init", "-b", "main")
        self.git("config", "user.name", "Receipt fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        (self.repo / "README.md").write_text("base\n")
        self.git("add", ".")
        self.git("commit", "-m", "base")
        self.base = self.oid("HEAD")
        (self.repo / "README.md").write_text("ordinary docs\n")
        self.git("commit", "-am", "docs")
        self.source = self.oid("HEAD")
        self.git("branch", "topic")
        self.git("update-ref", "refs/heads/main", self.base)
        self.key = self.home / "signer"
        run("ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(self.key))
        self.public_key = self.key.with_suffix(".pub").read_text().strip()
        self.verifier = load_verifier()
        self.inventory = ["fixture::passes"]
        self.log = b"running 1 test\ntest fixture::passes ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n"
        self.policy = {
            "schema": 1, "enabled": False, "repository": "fixture/repo",
            "signer_identity": "fixture", "signer_public_key": " ".join(self.public_key.split()[:2]),
            "plan_sha256": "1" * 64, "generator_sha256": "2" * 64,
            "environment_sha256": "3" * 64,
            "inventory_sha256": self.digest(self.verifier.canonical(self.inventory)),
            "expected_inventory": self.inventory, "expected_ignored": [],
        }
        self.event = {
            "kind": "push", "repository": "fixture/repo", "source": self.source,
            "base": self.base, "source_ref": "refs/heads/topic", "default_ref": "refs/heads/main",
            "default_source": self.base, "fork": False, "synthetic_merge": False,
        }
        self.manifest = {
            "schema": 1, "repository": "fixture/repo", "source": self.source,
            "tree": self.oid(self.source + "^{tree}"), "base": self.base,
            "policy_sha256": self.digest(self.verifier.canonical(self.policy)),
            "plan_sha256": self.policy["plan_sha256"], "generator_sha256": self.policy["generator_sha256"],
            "environment_sha256": self.policy["environment_sha256"], "inventory_sha256": self.policy["inventory_sha256"],
            "started": "2026-11-21T00:00:00Z", "finished": "2026-11-21T00:00:01Z",
            "duration_ms": 1000, "scope": "full", "status": "pass",
            "steps": [{"command": ["cargo", "test"], "exit": 0}],
            "counts": {"passed": 1, "failed": 0, "ignored": 0}, "inventory": self.inventory, "ignored": [],
            "log_sha256": self.digest(self.log), "log_size": len(self.log),
        }

    @staticmethod
    def digest(data):
        return hashlib.sha256(data).hexdigest()

    def publish(self, manifest_bytes=None, log=None, signer=None, omit=(), mode="100644", signature=None):
        manifest_bytes = manifest_bytes if manifest_bytes is not None else self.verifier.canonical(self.manifest)
        path = self.home / "manifest.json"
        path.write_bytes(manifest_bytes)
        path.with_suffix(".json.sig").unlink(missing_ok=True)
        run("ssh-keygen", "-Y", "sign", "-f", str(signer or self.key), "-n", "mizar-local-full-v1", str(path))
        blobs = {"manifest.json": manifest_bytes, "manifest.sig": signature if signature is not None else path.with_suffix(".json.sig").read_bytes(),
                 "cargo-test.log": self.log if log is None else log}
        records = []
        for name, data in blobs.items():
            if name not in omit:
                oid = self.git("hash-object", "-w", "--stdin", data=data).decode().strip()
                records.append(f"{mode} blob {oid}\t{name}\n")
        tree = self.git("mktree", data="".join(records).encode()).decode().strip()
        evidence = self.git("commit-tree", tree, "-m", "receipt").decode().strip()
        self.git("update-ref", "refs/tags/ci-receipts/" + self.source, evidence)

    def verify(self, event=None, policy=None, now="2026-11-21T01:00:00Z"):
        now = dt.datetime.fromisoformat(now.replace("Z", "+00:00"))
        return self.verifier.verify_receipt(self.repo, self.verifier.canonical(self.policy if policy is None else policy),
                                            self.event if event is None else event, now=now)

    def test_01_signed_docs_roundtrip(self):
        self.assert_full(reason="missing receipt")
        self.publish()
        result = self.verify()
        self.assertTrue(result["eligible"])
        self.assertEqual(result["decision"], "full")
        isolated = self.home / "sanity.git"
        run("git", "init", "--bare", str(isolated))
        self.git("config", "uploadpack.allowFilter", "true")
        self.git("config", "uploadpack.allowAnySHA1InWant", "true")
        run("git", "-C", str(isolated), "remote", "add", "origin", str(self.repo))
        run("git", "-C", str(isolated), "fetch", "--no-tags", str(self.repo),
            "refs/heads/main:refs/heads/main", "refs/heads/topic:refs/heads/topic")
        result = self.verifier.verify_receipt(isolated, self.verifier.canonical(self.policy), self.event,
                                             now=dt.datetime(2026, 11, 21, 1, tzinfo=dt.timezone.utc), remote="origin")
        self.assertTrue(result["eligible"])
        self.assertEqual(run("git", "-C", str(isolated), "tag", "--list").decode().strip(), "ci-receipts/" + self.source)

    def assert_invalid(self, operation=None):
        with self.assertRaises(self.verifier.ReceiptError):
            (operation or self.verify)()

    def assert_full(self, operation=None, reason="delta requires full CI"):
        self.assertEqual((operation or self.verify)(),
                         {"decision": "full", "eligible": False, "reason": reason})

    def test_03_exact_revision_and_comparison(self):
        original = copy.deepcopy(self.manifest)
        for field, value in (("repository", "other/repo"), ("source", self.base),
                             ("tree", self.oid(self.base + "^{tree}")), ("base", self.source)):
            with self.subTest(identity=field):
                self.manifest = {**original, field: value}
                self.publish()
                self.assert_invalid()
        self.manifest = original
        self.publish()
        for ref in ("refs/heads/topic", "refs/heads/main"):
            with self.subTest(moved=ref):
                old = self.oid(ref)
                self.git("update-ref", ref, self.base if old == self.source else self.source)
                self.assert_invalid()
                self.git("update-ref", ref, old)
        calls = 0
        real_refs = self.verifier.live_refs
        def moved_during_verification(*args):
            nonlocal calls
            calls += 1
            if calls == 2:
                self.git("update-ref", "refs/heads/topic", self.base)
            return real_refs(*args)
        with mock.patch.object(self.verifier, "live_refs", side_effect=moved_during_verification):
            self.assert_invalid()
        self.git("update-ref", "refs/heads/topic", self.source)
        (self.repo / "README.md").write_text("later docs\n")
        self.git("commit", "-am", "later docs")
        later = self.oid("HEAD")
        self.git("update-ref", "refs/heads/topic", later)
        self.git("update-ref", "refs/heads/main", self.base)
        old_tag = "refs/tags/ci-receipts/" + self.source
        self.git("update-ref", "refs/tags/ci-receipts/" + later, self.oid(old_tag))
        with self.subTest(replay=True):
            self.assert_invalid(lambda: self.verify(event={**self.event, "source": later}))
        with self.subTest(partial_branch_delta=True):
            earlier = self.source
            self.source = later
            self.manifest = {**original, "source": later, "tree": self.oid(later + "^{tree}"), "base": earlier}
            self.publish()
            self.assert_full(lambda: self.verify(event={**self.event, "source": later, "base": earlier}))
        self.git("reset", "--hard", self.base)
        (self.repo / "code.rs").write_text("code\n")
        self.git("add", ".")
        self.git("commit", "-m", "code before docs")
        (self.repo / "README.md").write_text("docs after code\n")
        self.git("commit", "-am", "docs after code")
        self.source = self.oid("HEAD")
        self.git("update-ref", "refs/heads/topic", self.source)
        self.git("update-ref", "refs/heads/main", self.base)
        self.manifest = {**original, "source": self.source, "tree": self.oid(self.source + "^{tree}")}
        self.publish()
        with self.subTest(entire_branch_delta=True):
            self.assert_full(lambda: self.verify(event={**self.event, "source": self.source}))
        self.git("update-ref", "refs/heads/main", self.source)
        with self.subTest(entire_main_push_delta=True):
            self.assert_full(lambda: self.verify(event={**self.event, "source": self.source,
                "source_ref": "refs/heads/main", "default_source": self.source}))
        # Two independently merged tips have two best common ancestors.
        tree = self.oid(self.base + "^{tree}")
        left = self.git("commit-tree", tree, "-p", self.base, "-m", "left").decode().strip()
        right = self.git("commit-tree", tree, "-p", self.base, "-m", "right").decode().strip()
        docs_tree = self.oid(original["source"] + "^{tree}")
        first = self.git("commit-tree", docs_tree, "-p", left, "-p", right, "-m", "first merge").decode().strip()
        second = self.git("commit-tree", tree, "-p", right, "-p", left, "-m", "second merge").decode().strip()
        self.assertEqual(set(self.git("merge-base", "--all", first, second).decode().splitlines()), {left, right})
        self.source = first
        self.git("update-ref", "refs/heads/topic", first)
        self.git("update-ref", "refs/heads/main", second)
        self.manifest = {**original, "source": first, "tree": docs_tree, "base": left}
        self.publish()
        with self.subTest(ambiguous_merge_base=True):
            self.assertTrue(self.verifier.classify_delta(self.repo, left, first))
            self.assert_full(lambda: self.verify(event={**self.event, "source": first,
                "base": left, "default_source": second}))

    def test_04_signature_pins_and_time(self):
        original = copy.deepcopy(self.manifest)
        mutations = [(field, "f" * 64) for field in ("policy_sha256", "plan_sha256", "generator_sha256",
                                                     "environment_sha256", "inventory_sha256")]
        mutations += [("started", "2026-11-19T23:59:59Z"), ("finished", "2026-11-22T00:00:00Z"),
                      ("started", "2026-11-21T00:00:02Z"), ("duration_ms", -1)]
        for field, value in mutations:
            with self.subTest(field=field, value=value):
                self.manifest = {**original, field: value}
                self.publish()
                self.assert_invalid()
        self.manifest = original
        for signature in (b"forgery", b""):
            with self.subTest(signature=signature):
                self.publish(signature=signature)
                self.assert_invalid()
        other_key = self.home / "unknown-signer"
        run("ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-f", str(other_key))
        self.publish(signer=other_key)
        with self.subTest(unknown_signer=True):
            self.assert_invalid()
        self.publish()
        signed = (self.home / "manifest.json.sig").read_bytes()
        self.manifest = {**original, "duration_ms": 999}
        self.publish(signature=signed)
        with self.subTest(tampered_after_signature=True):
            self.assert_invalid()
        self.manifest = original
        self.publish()
        with self.subTest(revoked_signer=True):
            self.assert_invalid(lambda: self.verify(policy={**self.policy, "signer_public_key": None}))
        for field in ("plan_sha256", "generator_sha256", "environment_sha256"):
            with self.subTest(changed_trust_pin=field):
                self.assert_invalid(lambda: self.verify(policy={**self.policy, field: "e" * 64}))

    def test_05_strict_schema_and_evidence(self):
        valid = self.verifier.canonical(self.manifest)
        malformed = [valid[:-2] + b',"schema":1}\n', valid.replace(b'"schema":1', b'"schema":2'),
                     valid[:-2] + b',"unknown":true}\n', valid + b"garbage", valid.replace(b'"full"', b'"\\ud800"'),
                     b"\xff", valid.replace(b'"duration_ms":1000', b'"duration_ms":1.0'),
                     valid.replace(b'"duration_ms":1000', b'"duration_ms":NaN'), valid.replace(b'"schema":1', b'"schema":true'),
                     b" " + valid, valid.rstrip(b"\n"), b"x" * (1024 * 1024 + 1)]
        for number, manifest in enumerate(malformed):
            with self.subTest(malformed=number):
                self.publish(manifest_bytes=manifest)
                self.assert_invalid()
        for name in ("manifest.json", "manifest.sig", "cargo-test.log"):
            with self.subTest(missing=name):
                self.publish(omit=(name,))
                self.assert_invalid()
        for mode in ("100755", "120000"):
            with self.subTest(mode=mode):
                self.publish(mode=mode)
                self.assert_invalid()
        for log in (b"", self.log + b"tampered"):
            with self.subTest(log=log):
                self.publish(log=log)
                self.assert_invalid()
        self.publish(signature=b"x" * 16385)
        with self.subTest(oversized_signature=True):
            self.assert_invalid()

    def test_06_literal_delta_routing(self):
        paths = {"README.md": True, "doc/guide.md": True, "doc/nested/guide.md": True,
                 "doc/spec/en/language.md": False, "doc/spec/ja/language.md": False,
                 "doc/design/architecture/en/owner.md": False, "doc/design/task_contracts/en/task.md": False,
                 "src/lib.rs": False, "tests/example.miz": False, "tests/data.toml": False,
                 "Cargo.toml": False, "Cargo.lock": False, "rust-toolchain.toml": False,
                 ".cargo/config.toml": False, ".github/workflows/ci.yml": False,
                 "script.sh": False, "unknown.md": False, "doc/image.svg": False,
                 "README.MD": False, "doc/specification.md": True, "doc/design/todo.md.extra.md": True}
        for name in ("autonomous_crate_development", "documentation_compaction_rules", "codex_dispatch_template",
                     "ci_receipt_model", "todo", "spec_coverage_audit"):
            paths["doc/design/" + name + ".md"] = False
        for path, expected in paths.items():
            with self.subTest(path=path):
                self.git("checkout", "--detach", self.base)
                file = self.repo / path
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_text("changed\n")
                self.git("add", ".")
                self.git("commit", "-m", "delta")
                self.assertEqual(self.verifier.classify_delta(self.repo, self.base, self.oid("HEAD")), expected)
        self.git("checkout", "--detach", self.base)
        with self.subTest(empty=True):
            self.assertFalse(self.verifier.classify_delta(self.repo, self.base, self.base))
        for mode, blob in (("100755", self.oid(self.base + ":README.md")),
                           ("120000", self.oid(self.base + ":README.md")), ("160000", self.base)):
            with self.subTest(mode=mode):
                self.git("read-tree", self.base)
                self.git("update-index", "--cacheinfo", mode, blob, "README.md")
                tree = self.git("write-tree").decode().strip()
                commit = self.git("commit-tree", tree, "-p", self.base, "-m", "mode").decode().strip()
                self.assertFalse(self.verifier.classify_delta(self.repo, self.base, commit))
        self.git("read-tree", self.base)
        self.git("update-index", "--cacheinfo", "100755", self.oid(self.base + ":README.md"), "README.md")
        executable_tree = self.git("write-tree").decode().strip()
        executable = self.git("commit-tree", executable_tree, "-p", self.base, "-m", "executable").decode().strip()
        for delete in (False, True):
            with self.subTest(old_executable=True, delete=delete):
                self.git("read-tree", self.base)
                if delete:
                    self.git("update-index", "--force-remove", "README.md")
                tree = self.git("write-tree").decode().strip()
                regular = self.git("commit-tree", tree, "-p", executable, "-m", "old mode").decode().strip()
                self.assertFalse(self.verifier.classify_delta(self.repo, executable, regular))
        for target, expected in (("doc/renamed.md", True), ("src/renamed.rs", False)):
            with self.subTest(rename=target):
                self.git("reset", "--hard", self.base)
                (self.repo / target).parent.mkdir(parents=True, exist_ok=True)
                self.git("mv", "README.md", target)
                self.git("commit", "-m", "rename")
                self.assertEqual(self.verifier.classify_delta(self.repo, self.base, self.oid("HEAD")), expected)
        with self.subTest(missing_base=True):
            self.assertFalse(self.verifier.classify_delta(self.repo, "0" * 40, self.source))
        with self.subTest(nonancestor_base=True):
            self.assertFalse(self.verifier.classify_delta(self.repo, self.oid("HEAD"), self.source))
        self.git("reset", "--hard", self.base)
        (self.repo / "code.rs").write_text("code\n")
        self.git("add", ".")
        self.git("commit", "-m", "code base")
        code_base = self.oid("HEAD")
        (self.repo / "doc").mkdir(exist_ok=True)
        self.git("mv", "code.rs", "doc/apparently-safe.md")
        self.git("commit", "-m", "rename code into docs")
        with self.subTest(rename_old_path=True):
            self.assertFalse(self.verifier.classify_delta(self.repo, code_base, self.oid("HEAD")))

    def test_07_untrusted_candidates_and_events(self):
        self.publish()
        for changes in ({"kind": "pull_request"}, {"fork": True}, {"synthetic_merge": True}):
            with self.subTest(event=changes):
                self.assert_full(lambda: self.verify(event={**self.event, **changes}),
                                 reason="event requires full CI")
        for changes in ({"default_source": "0" * 40}, {"default_ref": "refs/heads/unavailable"}):
            with self.subTest(event=changes):
                self.assert_invalid(lambda: self.verify(event={**self.event, **changes}))
        self.git("reset", "--hard", self.base)
        controls = self.repo / ".github/ci-receipts"
        controls.mkdir(parents=True)
        (controls / "verify.py").write_bytes((ROOT / ".github/ci-receipts/verify.py").read_bytes())
        (controls / "policy.json").write_bytes(self.verifier.canonical(self.policy))
        self.git("add", ".")
        self.git("commit", "-m", "trusted controls")
        trusted = self.oid("HEAD")
        self.git("config", "uploadpack.allowFilter", "true")
        self.git("config", "uploadpack.allowAnySHA1InWant", "true")
        finished = dt.datetime.now(dt.timezone.utc) - dt.timedelta(seconds=1)
        self.manifest.update(started=(finished - dt.timedelta(seconds=1)).strftime("%Y-%m-%dT%H:%M:%SZ"),
                             finished=finished.strftime("%Y-%m-%dT%H:%M:%SZ"))
        for path in (".github/ci-receipts/verify.py", ".github/ci-receipts/policy.json", ".github/workflows/ci.yml"):
            with self.subTest(candidate_substitution=path):
                self.git("checkout", "--detach", trusted)
                candidate = self.repo / path
                candidate.parent.mkdir(parents=True, exist_ok=True)
                candidate.write_text("raise RuntimeError('candidate must not execute')\n" if path.endswith(".py")
                                     else "untrusted candidate controls\n")
                self.git("add", ".")
                self.git("commit", "-m", "candidate trust substitution")
                self.source = self.oid("HEAD")
                self.git("update-ref", "refs/heads/topic", self.source)
                self.manifest.update(source=self.source, base=trusted, tree=self.oid("HEAD^{tree}"))
                self.publish()
                output, sanity = self.run_shadow("refs/heads/topic", trusted)
                self.assertEqual(json.loads(output.splitlines()[-1]),
                                 {"decision": "full", "eligible": False, "reason": "delta requires full CI"})
                for name in ("verify.py", "policy.json"):
                    self.assertEqual((sanity / name).read_bytes(),
                                     self.git("show", trusted + ":.github/ci-receipts/" + name))

    def test_08_shadow_deployment_boundary(self):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        shadow, candidate = workflow.split("  mizar-test:\n", 1)
        self.assertIn("  receipt-shadow:\n", shadow)
        self.assertIn("permissions:\n  contents: read\n", shadow)
        self.assertNotIn("actions/checkout", shadow)
        self.assertNotIn("secrets.", shadow)
        self.assertNotIn("needs:", candidate)
        self.assertNotIn("if:", candidate)
        for command in ("cargo fmt --check", "cargo fetch --locked", "cargo test",
                        "cargo clippy --all-targets -- -D warnings",
                        "cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml",
                        "python3 -m unittest discover -s tests/ci_receipts -p test_verifier.py -v"):
            self.assertIn("run: " + command if command != "cargo fetch --locked" else command, candidate)
        bootstrap = self.verifier.read_policy((ROOT / ".github/ci-receipts/policy.json").read_bytes())
        self.assertFalse(bootstrap["enabled"])
        for field in ("signer_public_key", "plan_sha256", "generator_sha256", "environment_sha256", "inventory_sha256"):
            self.assertIsNone(bootstrap[field])
        self.assert_invalid(lambda: self.verify(policy={**self.policy, "enabled": True}))

        self.git("reset", "--hard", self.base)
        controls = self.repo / ".github/ci-receipts"
        controls.mkdir(parents=True)
        (controls / "verify.py").write_bytes((ROOT / ".github/ci-receipts/verify.py").read_bytes())
        (controls / "policy.json").write_bytes(self.verifier.canonical(self.policy))
        self.git("add", ".")
        self.git("commit", "-m", "prior default controls")
        trusted = self.oid("HEAD")
        (controls / "verify.py").write_text("raise RuntimeError('candidate must not execute')\n")
        (controls / "policy.json").write_text("untrusted candidate policy\n")
        self.git("commit", "-am", "candidate controls")
        self.source = self.oid("HEAD")
        output, sanity = self.run_shadow("refs/heads/main", trusted)
        self.assertEqual(json.loads(output.splitlines()[-1]),
                         {"decision": "full", "eligible": False, "reason": "missing receipt"})
        for name in ("verify.py", "policy.json"):
            self.assertEqual((sanity / name).read_bytes(),
                             self.git("show", trusted + ":.github/ci-receipts/" + name))
        for before, reason in (("0" * 40, "no independent prior default-branch anchor"),
                               (self.source, "no independent prior default-branch anchor"),
                               (self.base, "bootstrap anchor has no verifier")):
            with self.subTest(unavailable_anchor=before):
                output, sanity = self.run_shadow("refs/heads/main", before)
                self.assertIn("full CI: " + reason, output.decode())
                self.assertFalse((sanity / "verify.py").exists())
        output, sanity = self.run_shadow("refs/pull/1/merge", trusted, event_kind="pull_request")
        self.assertIn("pull requests and merge revisions are ineligible", output.decode())
        self.assertFalse(sanity.exists())

        self.git("branch", "candidate", self.source)
        self.git("reset", "--hard", trusted)
        live_policy = {**self.policy, "signer_identity": "live-default"}
        with (controls / "verify.py").open("a") as verifier:
            verifier.write("\n# Live default fixture revision.\n")
        (controls / "policy.json").write_bytes(self.verifier.canonical(live_policy))
        self.git("commit", "-am", "advance live default")
        live_default = self.oid("HEAD")
        output, sanity = self.run_shadow("refs/heads/candidate", trusted)
        self.assertEqual(json.loads(output.splitlines()[-1]),
                         {"decision": "full", "eligible": False, "reason": "missing receipt"})
        for name in ("verify.py", "policy.json"):
            self.assertEqual((sanity / name).read_bytes(),
                             self.git("show", live_default + ":.github/ci-receipts/" + name))

    def run_shadow(self, source_ref, before, event_kind="push"):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        script = textwrap.dedent(workflow.split("        run: |\n", 1)[1].split("\n  mizar-test:\n", 1)[0])
        runner = Path(tempfile.mkdtemp(dir=self.home))
        env = {**os.environ, "RUNNER_TEMP": str(runner), "REPOSITORY": "fixture/repo",
               "EVENT_KIND": event_kind, "SOURCE": self.source, "SOURCE_REF": source_ref,
               "BEFORE": before, "DEFAULT_BRANCH": "main", "GIT_CONFIG_COUNT": "1",
               "GIT_CONFIG_KEY_0": "url." + str(self.repo) + ".insteadOf",
               "GIT_CONFIG_VALUE_0": "https://github.com/fixture/repo.git"}
        result = subprocess.run(["bash", "-c", script], env=env, check=True,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        return result.stdout, runner / "receipt-sanity"

    def git(self, *args, data=None):
        return run("git", "-C", str(self.repo), *args, data=data)

    def oid(self, revision):
        return self.git("rev-parse", revision).decode().strip()


if __name__ == "__main__":
    unittest.main()
