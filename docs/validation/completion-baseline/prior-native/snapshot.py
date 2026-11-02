import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys

fixture = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()

if not (fixture / ".gitfleet-m3-fixture").is_file():
    raise SystemExit("STOP: disposable fixture marker missing")
if output == fixture or fixture in output.parents:
    raise SystemExit("STOP: evidence must be outside the fixture")

env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
env.update(
    GIT_CONFIG_GLOBAL="/dev/null",
    GIT_CONFIG_NOSYSTEM="1",
    GIT_TERMINAL_PROMPT="0",
)

def git(repo, *args):
    return subprocess.check_output(
        ["git", "--no-optional-locks", "-C", str(repo), *args],
        env=env,
    ).decode("utf-8")

def snapshot(repo):
    entries = {}
    for directory, directories, files in os.walk(repo, followlinks=False):
        for name in sorted(directories + files):
            path = Path(directory) / name
            info = path.lstat()
            entry = {"mode": stat.S_IMODE(info.st_mode)}
            if stat.S_ISLNK(info.st_mode):
                entry.update(kind="symlink", target=os.readlink(path))
            elif stat.S_ISREG(info.st_mode):
                entry.update(
                    kind="file",
                    size=info.st_size,
                    sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                )
            elif stat.S_ISDIR(info.st_mode):
                entry.update(kind="directory")
            else:
                raise SystemExit(f"STOP: unexpected special file: {path}")
            entries[path.relative_to(repo).as_posix()] = entry
    return entries

result = {}
for name in ["engine", "engine/plugins/audio", "tooling", "documentation"]:
    repo = fixture / "workspace" / name
    result[name] = {
        "head_ref": git(repo, "symbolic-ref", "-q", "HEAD"),
        "head_oid": git(repo, "rev-parse", "HEAD"),
        "head_tree": git(repo, "rev-parse", "HEAD^{tree}"),
        "index_entries": git(repo, "ls-files", "--stage"),
        "status": git(repo, "status", "--porcelain=v2", "--untracked-files=all"),
        "entries": snapshot(repo),
    }

output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
print(f"Saved snapshot: {output}")
for name, data in result.items():
    print(f"{name}: HEAD {data['head_oid'].strip()}, "
          f"{len(data['entries'])} filesystem entries")
