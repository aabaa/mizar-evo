#!/usr/bin/python3 -I
"""Install outside Git beside the owner-controlled generator and custody.json."""
import argparse
import fcntl
import hashlib
import types
import json
import os
from pathlib import Path
import re
import shlex
import sys
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent


def custody(names=("git", "ssh_keygen")):
    config = json.loads((HERE / "custody.json").read_bytes())
    policy_bytes = Path(config["policy_path"]).read_bytes()
    policy = json.loads(policy_bytes)
    repo = Path(config["repository_path"]).resolve()
    if (not config.get("enabled") or "publish_ref" in config or config.get("admission_ref") != "refs/heads/main"
            or HERE.is_relative_to(repo) or config["repository"] != policy["repository"]):
        raise ValueError("custody admission is disabled, legacy or inside the candidate")
    generator = HERE / "generate.py"
    generator_bytes = generator.read_bytes()
    if generator.is_symlink() or hashlib.sha256(generator_bytes).hexdigest() != policy["generator_sha256"]:
        raise ValueError("generator differs from trusted policy")
    for name in names:
        tool = Path(config[name])
        if (not tool.is_absolute() or tool.is_symlink() or not tool.is_file() or not os.access(tool, os.X_OK)
                or hashlib.sha256(tool.read_bytes()).hexdigest() != config["tool_sha256"][name]):
            raise ValueError("tool differs from custody pin: " + name)
    module = types.ModuleType("custody_generator")
    module.__file__ = str(generator)
    exec(compile(generator_bytes, str(generator), "exec"), module.__dict__)
    def run(command, *, cwd=None, env=None, data=None):
        return subprocess.run(module.git_command(command), cwd=cwd, env=env or module.controlled_env(),
                              input=data, check=True, capture_output=True, timeout=120).stdout
    module.run = run
    return module, config, policy_bytes


def transport(generator, config):
    command, env = [config["git"], "-c", "credential.helper="], generator.controlled_env()
    if any(name in config for name in ("gh", "gh_sha256", "gh_config_dir")):
        gh, directory = Path(config["gh"]), Path(config["gh_config_dir"])
        repo = Path(config["repository_path"]).resolve()
        if (not gh.is_absolute() or gh.is_symlink() or not gh.is_file() or not os.access(gh, os.X_OK)
                or generator.digest(gh.read_bytes()) != config["gh_sha256"] or not directory.is_absolute()
                or directory.is_symlink() or not directory.is_dir()
                or gh.resolve().is_relative_to(repo) or directory.resolve().is_relative_to(repo)):
            raise ValueError("publication helper differs from custody pin")
        env["GH_CONFIG_DIR"] = str(directory)
        command.extend(["-c", "credential.helper=!" + shlex.quote(str(gh)) + " auth git-credential"])
    return command, env


def admit(source, base):
    g, config, policy = custody()
    if source == base or not all(re.fullmatch(r"[0-9a-f]{40}", oid) for oid in (source, base)):
        raise ValueError("source and prior-main base must be distinct full object IDs")
    git, repo, ref = config["git"], Path(config["repository_path"]), config["admission_ref"]
    tag, ledger = "refs/tags/ci-receipts/" + source, Path(config["ledger_path"])
    with (HERE / "custody.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        entries = [json.loads(line) for line in ledger.read_text().splitlines()]
        pending = [entry for entry in entries if entry["source"] == source]
        evidence = g.run([git, "-C", str(repo), "rev-parse", tag]).decode().strip()
        if len(pending) != 1 or pending[0] != dict(source=source, evidence=evidence, tag=tag, published=False):
            raise ValueError("no unique pending custody receipt or evidence tag moved")
        tree = g.run([git, "-C", str(repo), "rev-parse", source + "^{tree}"]).decode().strip()
        def sealed():
            g.clean(git, repo, source, tree)
            if g.run([git, "-C", str(repo), "rev-parse", ref]).decode().strip() != source:
                raise ValueError("source ref moved")
        sealed()
        g.run([git, "-C", str(repo), "merge-base", "--is-ancestor", base, source])
        with tempfile.TemporaryDirectory(prefix="receipt-admission-", dir=HERE) as temporary:
            work = Path(temporary)
            bare, bundle = work / "transport", work / "sealed.bundle"
            g.run([git, "-C", str(repo), "bundle", "create", str(bundle), ref, tag])
            g.run([git, "init", "--bare", str(bare)])
            g.run([git, "-C", str(bare), "fetch", "--no-tags", str(bundle), ref + ":" + ref, tag + ":" + tag])
            command, env = transport(g, config)
            command += ["-C", str(bare)]
            remote = config["publish_remote"]
            def remote_base():
                if g.run([*command, "ls-remote", "--refs", remote, ref, tag], env=env).splitlines() != [(base + "\t" + ref).encode()]:
                    raise ValueError("remote prior-main base moved or evidence tag already exists")
            remote_base()
            for name in ("verify.py", "policy.json"):
                (work / name).write_bytes(g.run([git, "-C", str(bare), "show", base + ":.github/ci-receipts/" + name]))
            if (work / "policy.json").read_bytes() != policy:
                raise ValueError("prior-main policy differs from independent custody policy")
            tools = work / "bin"
            tools.mkdir()
            for name, tool in (("git", git), ("ssh-keygen", config["ssh_keygen"])):
                (tools / name).symlink_to(tool)
            verify_env = dict(g.controlled_env(), PATH=str(tools) + ":/usr/bin:/bin")
            g.run([sys.executable, "-I", str(work / "verify.py"), "--enforce", "--repository", str(bare),
                   "--policy", str(work / "policy.json"), "--repository-id", json.loads(policy)["repository"],
                   "--event-kind", "push", "--source", source, "--base", base, "--source-ref", ref,
                   "--default-ref", ref, "--default-source", source], cwd=work, env=verify_env)
            sealed()
            remote_base()
            g.run([*command, "push", "--atomic", "--porcelain", "--force-with-lease=" + ref + ":" + base,
                   remote, source + ":" + ref, tag + ":" + tag], env=env)
        try:
            with ledger.open("ab") as publication:
                publication.write(g.canonical(dict(source=source, evidence=evidence, tag=tag, published=True)))
                publication.flush()
                os.fsync(publication.fileno())
        except OSError as error:
            raise RuntimeError("published remotely; reconcile ledger before retry") from error
    return dict(source=source, evidence=evidence, published=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source")
    parser.add_argument("base")
    args = parser.parse_args()
    try:
        print(json.dumps(admit(args.source, args.base), sort_keys=True))
    except (Exception, KeyboardInterrupt) as error:
        parser.exit(1, "admission refused: " + str(error) + "\n")
