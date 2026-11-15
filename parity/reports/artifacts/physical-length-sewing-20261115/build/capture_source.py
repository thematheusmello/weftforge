import hashlib
import json
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone

worktree = Path(sys.argv[1]).resolve()
commit = sys.argv[2]
output = Path(sys.argv[3])
prefixes = ("crates/weftforge-core/", "crates/weftforge-sim/", "apps/weftforge-cli/", ".cargo/")
root_files = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml"}

def git(*args):
    return subprocess.check_output(["git", *args], cwd=worktree)

files = [path for path in git("ls-tree", "-r", "--name-only", commit).decode().splitlines()
         if path in root_files or path.startswith(prefixes)]
assert files and "Cargo.lock" in files
entries = []
for path in files:
    expected = hashlib.sha256(git("show", f"{commit}:{path}")).hexdigest()
    actual = hashlib.sha256((worktree / path).read_bytes()).hexdigest()
    entries.append({"path": path, "committed_sha256": expected, "working_sha256": actual,
                    "matches_commit": expected == actual})
manifest = "".join(f"{entry['committed_sha256']}  {entry['path']}\n" for entry in entries)
receipt = {
    "captured_at": datetime.now(timezone.utc).isoformat(),
    "source_commit": git("rev-parse", commit).decode().strip(),
    "current_head": git("rev-parse", "HEAD").decode().strip(),
    "scope": [*sorted(root_files), *prefixes],
    "excluded": ["apps/weftforge/ (concurrent native UI work)", "xtask/ (not a core/CLI runtime dependency)",
                 "parity/ (frozen independent evaluator is not run here)"],
    "scoped_source_manifest_sha256": hashlib.sha256(manifest.encode()).hexdigest(),
    "all_scoped_files_match_commit": all(entry["matches_commit"] for entry in entries),
    "files": entries,
    "worktree_status": git("status", "--short").decode().splitlines(),
}
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps({key: value for key, value in receipt.items() if key != "files"}, indent=2))
assert receipt["all_scoped_files_match_commit"], "Scoped source differs from committed source"
