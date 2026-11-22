#!/usr/bin/env python3
"""Verify receipts using independent trusted controls; never import evidence code."""
import argparse
import base64
import datetime as dt
import hashlib
import json
import re
import subprocess
import tempfile
from pathlib import Path


class ReceiptError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise ReceiptError(message)


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=False, allow_nan=False) + "\n").encode("utf-8")


def parse_json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result

    def forbidden(_):
        raise ReceiptError("noninteger JSON number")

    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=pairs,
                           parse_float=forbidden, parse_constant=forbidden)
        require(canonical(value) == data, "noncanonical JSON")
        return value
    except (UnicodeError, ValueError, RecursionError) as error:
        raise ReceiptError("invalid canonical JSON: " + str(error)) from error


def keys(value, expected):
    require(type(value) is dict and set(value) == set(expected.split()), "invalid object keys")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(repository, *arguments, check=True):
    result = subprocess.run(["git", "--no-replace-objects", "-C", str(repository), *arguments],
                            capture_output=True, check=False, timeout=30)
    require(not check or result.returncode == 0, "Git command failed: " + arguments[0] + ": " +
            result.stderr.decode("utf-8", errors="replace")[:1000])
    return result


def oid(value):
    return type(value) is str and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", value)


def object_bytes(repository, name, kind, limit):
    require(git(repository, "cat-file", "-t", name).stdout.strip() == kind.encode(),
            "unexpected evidence object type")
    size = int(git(repository, "cat-file", "-s", name).stdout)
    require(0 <= size <= limit, "oversized evidence object")
    return git(repository, "cat-file", kind, name).stdout


def classify_delta(repository, base, source):
    """Validate the entire nonempty comparison, independently of changed paths."""
    if git(repository, "merge-base", "--is-ancestor", base, source, check=False).returncode:
        return False
    records = git(repository, "diff", "--raw", "-z", "--no-renames", "--no-abbrev",
                  base, source, "--").stdout.split(b"\0")
    if records == [b""] or len(records) % 2 != 1 or records[-1]:
        return False
    modes = {b"100644", b"100755", b"120000", b"160000"}
    for index in range(0, len(records) - 1, 2):
        fields, path = records[index].split(), records[index + 1]
        if len(fields) != 5 or not fields[0].startswith(b":") or not path:
            return False
        old, new, old_oid, new_oid, status = fields[0][1:], *fields[1:]
        if old not in modes | {b"000000"} or new not in modes | {b"000000"}:
            return False
        if not all(re.fullmatch(rb"[0-9a-f]{40}|[0-9a-f]{64}", value) for value in (old_oid, new_oid)):
            return False
        if (old == b"000000") != (set(old_oid) == {48}) or (new == b"000000") != (set(new_oid) == {48}):
            return False
        expected = (b"A" if old == b"000000" else b"D" if new == b"000000" else
                    b"T" if old[:3] != new[:3] else b"M")
        if status != expected or old == new == b"000000":
            return False
    return True


def read_policy(data):
    policy = parse_json(data)
    keys(policy, "schema enabled repository signer_identity signer_public_key plan_sha256 "
         "generator_sha256 environment_sha256 inventory_sha256 expected_inventory expected_ignored")
    require(type(policy["schema"]) is int and policy["schema"] == 1, "unsupported policy")
    require(type(policy["enabled"]) is bool, "invalid enabled flag")
    require(type(policy["repository"]) is str and re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+",
                                                            policy["repository"]), "invalid repository")
    for field in ("signer_identity", "signer_public_key"):
        require(policy[field] is None or type(policy[field]) is str, "invalid signer pin")
    for field in ("plan_sha256", "generator_sha256", "environment_sha256", "inventory_sha256"):
        require(policy[field] is None or (type(policy[field]) is str and
                re.fullmatch(r"[0-9a-f]{64}", policy[field])), "invalid policy pin")
    for field in ("expected_inventory", "expected_ignored"):
        names(policy[field])
    require(set(policy["expected_ignored"]) <= set(policy["expected_inventory"]), "invalid ignored baseline")
    if policy["enabled"]:
        require(all(policy[field] is not None for field in (
            "signer_identity", "signer_public_key", "plan_sha256", "generator_sha256",
            "environment_sha256", "inventory_sha256")) and policy["expected_inventory"], "unprovisioned enabled policy")
        require(re.fullmatch(r"[A-Za-z0-9@._-]+", policy["signer_identity"]) and
                re.fullmatch(r"ssh-ed25519 [A-Za-z0-9+/]+={0,2}", policy["signer_public_key"]), "invalid signer pin")
        try:
            key = base64.b64decode(policy["signer_public_key"].split()[1], validate=True)
        except ValueError as error:
            raise ReceiptError("invalid signer key") from error
        require(len(key) == 51 and key[:19] == b"\0\0\0\x0bssh-ed25519\0\0\0\x20", "invalid signer key")
        require(digest(canonical(policy["expected_inventory"])) == policy["inventory_sha256"], "invalid inventory pin")
    return policy


def names(value):
    require(type(value) is list and all(type(name) is str and name and "\n" not in name for name in value),
            "invalid inventory")
    require(value == sorted(set(value)), "inventory must be sorted and unique")


def live_refs(repository, event, remote):
    require(type(event["default_ref"]) is str and event["default_ref"].startswith("refs/heads/"), "invalid default ref")
    source_ref = event["source_ref"]
    require(type(source_ref) is str and (
        event["kind"] == "push" and source_ref.startswith("refs/heads/") or
        event["kind"] == "pull_request" and re.fullmatch(r"refs/pull/[1-9][0-9]*/merge", source_ref)), "invalid source ref")
    refs = {source_ref, event["default_ref"]}
    for ref in refs:
        require(git(repository, "check-ref-format", ref, check=False).returncode == 0, "invalid live ref")
    if remote:
        lines = git(repository, "ls-remote", "--refs", remote, *sorted(refs)).stdout.splitlines()
        pairs = [line.decode("ascii").split("\t") for line in lines]
        require(all(len(pair) == 2 for pair in pairs), "invalid remote refs")
        result = {ref: value for value, ref in pairs}
        require(len(result) == len(pairs), "ambiguous remote refs")
    else:
        result = {ref: git(repository, "rev-parse", "--verify", ref).stdout.decode().strip() for ref in refs}
    require(result.get(event["source_ref"]) == event["source"] and
            result.get(event["default_ref"]) == event["default_source"], "live head or base moved")


def verify_signature(manifest, signature, policy):
    identity, key = policy["signer_identity"], policy["signer_public_key"]
    require(type(identity) is str and re.fullmatch(r"[A-Za-z0-9@._-]+", identity), "unprovisioned signer")
    require(type(key) is str and re.fullmatch(r"ssh-ed25519 [A-Za-z0-9+/]+={0,2}", key), "invalid signer key")
    try:
        base64.b64decode(key.split()[1], validate=True)
    except ValueError as error:
        raise ReceiptError("invalid signer key") from error
    with tempfile.TemporaryDirectory(prefix="mizar-receipt-") as directory:
        allowed = Path(directory) / "allowed"
        sig = Path(directory) / "signature"
        allowed.write_text(identity + " " + key + "\n", encoding="ascii")
        sig.write_bytes(signature)
        result = subprocess.run(["ssh-keygen", "-Y", "verify", "-f", str(allowed), "-I", identity,
                                 "-n", "mizar-local-full-v1", "-s", str(sig)],
                                input=manifest, capture_output=True, timeout=30, check=False)
        require(result.returncode == 0, "invalid receipt signature")


def remote_tag(repository, remote, tag):
    lines = git(repository, "ls-remote", "--refs", remote, tag).stdout.decode().splitlines()
    if not lines:
        return None
    require(len(lines) == 1, "ambiguous receipt tag")
    fields = lines[0].split("\t")
    require(len(fields) == 2 and oid(fields[0]) and fields[1] == tag, "invalid receipt advertisement")
    return fields[0]


def timestamp(value):
    require(type(value) is str and re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", value),
            "invalid UTC timestamp")
    try:
        return dt.datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=dt.timezone.utc)
    except ValueError as error:
        raise ReceiptError("invalid UTC timestamp") from error


def verify_receipt(repository, policy_bytes, event, now=None, remote=None, enforce=False):
    policy = read_policy(policy_bytes)
    require(not enforce or policy["enabled"], "receipt enforcement is disabled")
    keys(event, "kind repository source base source_ref default_ref default_source fork synthetic_merge")
    require(event["repository"] == policy["repository"], "event repository mismatch")
    require(event["kind"] in ("push", "pull_request") and type(event["fork"]) is bool and
            type(event["synthetic_merge"]) is bool, "invalid event")
    for field in ("source", "base", "default_source"):
        require(oid(event[field]), "invalid event commit")
    full = {"decision": "full", "eligible": False, "reason": "event requires full CI"}
    require(event["kind"] == "pull_request" and event["synthetic_merge"] or
            event["kind"] == "push" and not event["fork"] and not event["synthetic_merge"], "unsupported event")
    live_refs(repository, event, remote)
    require(git(repository, "cat-file", "-t", event["source"]).stdout.strip() == b"commit", "source is not commit")
    comparison_valid = True
    if event["kind"] == "pull_request":
        parent = git(repository, "rev-parse", "--verify", event["source"] + "^1", check=False)
        comparison_valid = (parent.returncode == 0 and parent.stdout.decode().strip() ==
                            event["base"] == event["default_source"])
    elif event["source_ref"] == event["default_ref"]:
        comparison_valid = event["default_source"] == event["source"]
    else:
        bases = git(repository, "merge-base", "--all", event["default_source"], event["source"], check=False)
        comparison_valid = bases.returncode == 0 and bases.stdout.decode().splitlines() == [event["base"]]
    eligible_delta = comparison_valid and classify_delta(repository, event["base"], event["source"])
    require(not enforce or eligible_delta, "invalid entire-delta comparison")
    tag = "refs/tags/ci-receipts/" + event["source"]
    if remote:
        advertised = remote_tag(repository, remote, tag)
        if advertised is None:
            require(not enforce, "missing receipt")
            return {**full, "reason": "missing receipt"}
        git(repository, "fetch", "--no-tags", "--no-recurse-submodules", "--no-write-fetch-head",
            "--depth=1", "--filter=tree:0", remote, tag + ":" + tag)
    target = git(repository, "rev-parse", "--verify", tag, check=False)
    if target.returncode:
        require(not enforce, "missing receipt")
        return {**full, "reason": "missing receipt"}
    evidence_oid = target.stdout.decode().strip()
    require(not remote or evidence_oid == advertised, "receipt tag changed during fetch")
    commit = object_bytes(repository, evidence_oid, "commit", 16384)
    header = commit.split(b"\n\n", 1)[0].splitlines()
    require(header and re.fullmatch(rb"tree ([0-9a-f]{40}|[0-9a-f]{64})", header[0]) and
            not any(line.startswith(b"parent ") for line in header), "evidence must be detached root commit")
    tree_oid = header[0].split()[1].decode()
    if remote:
        # Fetch only the root tree, then each fixed blob explicitly. Do not let
        # cat-file lazily fetch an attacker-selected descendant tree graph.
        git(repository, "fetch", "--refetch", "--depth=1", "--no-tags", "--no-recurse-submodules", "--no-write-fetch-head",
            "--filter=combine:tree:1+object:type=tree", remote, tree_oid)
    object_bytes(repository, tree_oid, "tree", 16384)
    entries = git(repository, "ls-tree", "-z", tree_oid).stdout.split(b"\0")
    blobs = {}
    for entry in entries[:-1]:
        metadata, name = entry.split(b"\t", 1)
        mode, kind, blob = metadata.split()
        require(mode == b"100644" and kind == b"blob" and name not in blobs, "nonregular evidence")
        require(name in (b"manifest.json", b"manifest.sig", b"cargo-test.log"), "unknown evidence path")
        limit = {b"manifest.json": 1024 * 1024, b"manifest.sig": 16384, b"cargo-test.log": 128 * 1024 * 1024}[name]
        if remote:
            git(repository, "fetch", "--refetch", "--depth=1", "--no-tags", "--no-recurse-submodules", "--no-write-fetch-head",
                "--filter=combine:tree:1+object:type=blob", remote, blob.decode())
        blobs[name] = object_bytes(repository, blob.decode(), "blob", limit)
    require(set(blobs) == {b"manifest.json", b"manifest.sig", b"cargo-test.log"}, "missing evidence blob")
    verify_signature(blobs[b"manifest.json"], blobs[b"manifest.sig"], policy)
    manifest = parse_json(blobs[b"manifest.json"])
    keys(manifest, "schema repository source tree base policy_sha256 plan_sha256 generator_sha256 "
         "environment_sha256 inventory_sha256 started finished duration_ms scope status steps counts "
         "inventory ignored log_sha256 log_size")
    require(type(manifest["schema"]) is int and manifest["schema"] == 1, "unsupported manifest")
    tree = git(repository, "rev-parse", event["source"] + "^{tree}").stdout.decode().strip()
    for field, expected in (("repository", event["repository"]), ("source", event["source"]),
                            ("tree", tree), ("base", event["base"]), ("policy_sha256", digest(policy_bytes))):
        require(manifest[field] == expected, "manifest identity mismatch: " + field)
    for field in ("plan_sha256", "generator_sha256", "environment_sha256", "inventory_sha256"):
        require(policy[field] is not None and manifest[field] == policy[field], "untrusted pin: " + field)
    require(manifest["scope"] == "full" and manifest["status"] == "pass", "not a full passing receipt")
    require(manifest["steps"] == [{"command": ["cargo", "test"], "exit": 0}] and
            type(manifest["steps"][0]["exit"]) is int, "invalid mandatory steps")
    names(manifest["inventory"])
    names(manifest["ignored"])
    require(manifest["inventory"] and manifest["inventory"] == policy["expected_inventory"] and
            manifest["ignored"] == policy["expected_ignored"] and
            digest(canonical(manifest["inventory"])) == manifest["inventory_sha256"], "inventory drift")
    counts = manifest["counts"]
    keys(counts, "passed failed ignored")
    require(all(type(value) is int and value >= 0 for value in counts.values()) and
            counts["failed"] == 0 and counts["ignored"] == len(manifest["ignored"]) and
            counts["passed"] > 0 and sum(counts.values()) == len(manifest["inventory"]), "invalid test counts")
    log = blobs[b"cargo-test.log"]
    require(type(manifest["log_size"]) is int and manifest["log_size"] == len(log) > 0 and
            manifest["log_sha256"] == digest(log), "log identity mismatch")
    started, finished = timestamp(manifest["started"]), timestamp(manifest["finished"])
    now = now or dt.datetime.now(dt.timezone.utc)
    require(started <= finished <= now and now - started <= dt.timedelta(hours=24), "receipt time invalid")
    require(type(manifest["duration_ms"]) is int and manifest["duration_ms"] >= 0 and
            abs(manifest["duration_ms"] - (finished - started).total_seconds() * 1000) <= 1000,
            "invalid execution duration")
    live_refs(repository, event, remote)
    require(git(repository, "rev-parse", "--verify", tag).stdout.decode().strip() == evidence_oid, "receipt tag moved")
    if remote:
        require(remote_tag(repository, remote, tag) == evidence_oid, "remote receipt tag moved")
    if not eligible_delta:
        return {**full, "reason": "delta requires full CI"}
    if enforce:
        return {"decision": "pass", "eligible": True, "reason": "valid exact-event full receipt"}
    return {"decision": "full", "eligible": True, "reason": "valid receipt; observation retains full CI"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--policy", required=True)
    parser.add_argument("--remote")
    parser.add_argument("--enforce", action="store_true")
    for name in ("repository-id", "source", "base", "source-ref", "default-ref", "default-source"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--event-kind", choices=("push", "pull_request"), required=True)
    parser.add_argument("--fork", action="store_true")
    parser.add_argument("--synthetic-merge", action="store_true")
    args = parser.parse_args()
    event = {"kind": args.event_kind, "repository": args.repository_id, "source": args.source,
             "base": args.base, "source_ref": args.source_ref, "default_ref": args.default_ref,
             "default_source": args.default_source, "fork": args.fork, "synthetic_merge": args.synthetic_merge}
    try:
        print(json.dumps(verify_receipt(args.repository, Path(args.policy).read_bytes(), event, remote=args.remote, enforce=args.enforce)))
    except (ReceiptError, OSError, subprocess.SubprocessError) as error:
        parser.exit(1, "receipt sanity failed: " + str(error) + "\n")


if __name__ == "__main__":
    main()
