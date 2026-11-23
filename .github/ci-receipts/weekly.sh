#!/bin/sh
# Install beside admit.py in owner-controlled custody storage.
set -eu
PATH=/usr/bin:/bin
export PATH
exec /usr/bin/env -i PATH="$PATH" /usr/bin/python3 -I - "$(dirname "$(realpath "$0")")" <<'PY'
import datetime as dt
import fcntl
import importlib.util
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile

here = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("custody_admission", here / "admit.py")
admission = importlib.util.module_from_spec(spec)
spec.loader.exec_module(admission)
g, config, _ = admission.custody(("git", "cargo", "rustc", "rustdoc", "ssh_keygen", "bwrap"))
with (here / "custody.lock").open("a") as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    logs = Path(config["work_root"]) / "weekly"
    logs.mkdir(parents=True, exist_ok=True)
    with (logs / (dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ") + ".log")).open("wb") as log:
        try:
            with tempfile.TemporaryDirectory(prefix="weekly-full-", dir=config["work_root"]) as temporary:
                work = Path(temporary)
                checkout = work / "source"
                command, transport_env = admission.transport(g, config)
                g.run([*command, "clone", "--no-checkout", "--no-local", "--no-hardlinks", "--single-branch",
                       "--branch", "main", "--", config["publish_remote"], str(checkout)], env=transport_env)
                git = config["git"]
                source = g.run([git, "-C", str(checkout), "rev-parse", "refs/heads/main"]).decode().strip()
                log.write(("source " + source + "\ncommand cargo test\n").encode())
                log.flush()
                g.run([git, "-C", str(checkout), "checkout", "--detach", source])
                tree = g.run([git, "-C", str(checkout), "rev-parse", source + "^{tree}"]).decode().strip()
                g.clean(git, checkout, source, tree)
                if any(path.is_file() for name in ("config", "config.toml") for path in checkout.rglob(".cargo/" + name)):
                    raise ValueError("candidate Cargo configuration is forbidden")
                tools = checkout / ".tools"
                if tools.is_symlink() or (tools.exists() and (not tools.is_dir() or any(tools.iterdir()))):
                    raise ValueError("candidate .tools contents are forbidden")
                tools.mkdir(exist_ok=True)
                fixture = Path(config["fixture_root"]) / "provers/vampire-4.9/vampire"
                if (any(path.is_symlink() for path in (fixture, fixture.parent, fixture.parent.parent, fixture.parent.parent.parent))
                        or not stat.S_ISREG(fixture.stat().st_mode) or stat.S_IMODE(fixture.stat().st_mode) != 0o755
                        or g.digest(fixture.read_bytes()) != config["fixture_sha256"]):
                    raise ValueError("fixture differs from custody pin")
                snapshot = work / "fixtures" / "provers/vampire-4.9/vampire"
                snapshot.parent.mkdir(parents=True)
                snapshot.write_bytes(fixture.read_bytes())
                snapshot.chmod(0o555)
                for name in ("home", "cargo-home", "target"):
                    (work / name).mkdir()
                for name in ("registry", "git"):
                    cache = Path(config["cargo_cache_path"]) / name
                    if cache.exists():
                        shutil.copytree(cache, work / "cargo-home" / name)
                env = dict(g.controlled_env(), HOME=str(work / "home"), CARGO_HOME=str(work / "cargo-home"),
                           CARGO_TARGET_DIR=str(work / "target"), CARGO_BUILD_JOBS="4", RUST_TEST_THREADS="4",
                           CARGO_NET_OFFLINE="true", RUSTC=config["rustc"], RUSTDOC=config["rustdoc"], TZ="UTC",
                           CARGO_ENCODED_RUSTFLAGS="", CARGO_ENCODED_RUSTDOCFLAGS="")
                sandbox = [config["bwrap"], "--unshare-all", "--die-with-parent", "--new-session", "--proc", "/proc",
                           "--dev", "/dev", "--tmpfs", "/tmp"]
                for directory in ("/usr", "/bin", "/lib", "/lib64", "/etc/alternatives", "/etc/ld.so.cache",
                                  str(Path(config["rustc"]).parent.parent)):
                    if Path(directory).exists():
                        sandbox += ["--ro-bind", directory, directory]
                for directory in (checkout, work / "home", work / "cargo-home", work / "target"):
                    sandbox += ["--bind", str(directory), str(directory)]
                sandbox += ["--ro-bind", str(checkout / ".git"), str(checkout / ".git"),
                            "--ro-bind", str(work / "fixtures"), str(tools), "--chdir", str(checkout),
                            "--", config["cargo"], "test"]
                watch = g.monitor(checkout)
                try:
                    subprocess.run(sandbox, env=env, stdout=log, stderr=subprocess.STDOUT,
                                   check=True, timeout=config["timeout_seconds"])
                    try:
                        mutation = os.read(watch, 1024 * 1024)
                    except BlockingIOError:
                        mutation = b""
                    if mutation:
                        raise ValueError("concurrent source mutation observed")
                finally:
                    os.close(watch)
                g.clean(git, checkout, source, tree)
                log.write(b"weekly full test passed\n")
        except BaseException as error:
            log.write(("weekly full test failed: " + str(error) + "\n").encode())
            raise
PY
