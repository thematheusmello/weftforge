from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import time

worktree = Path(sys.argv[1]).resolve()
target = Path(sys.argv[2]).resolve()
directory = Path(__file__).parent
name = sys.argv[3]
command = sys.argv[4:]
assert command and name and "/" not in name
environment = os.environ.copy()
environment["CARGO_TARGET_DIR"] = str(target)
started = datetime.now(timezone.utc).isoformat()
clock = time.monotonic()
log = directory / f"{name}.log"
print(f"Starting {name}: {' '.join(command)}", flush=True)
with log.open("wb") as output:
    result = subprocess.run(command, cwd=worktree, env=environment, stdout=output, stderr=subprocess.STDOUT)
receipt = {"name": name, "command": command, "cwd": str(worktree), "cargo_target_dir": str(target),
           "started_at": started, "finished_at": datetime.now(timezone.utc).isoformat(),
           "elapsed_seconds": time.monotonic() - clock, "exit_code": result.returncode, "log": log.name}
(directory / f"{name}.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2), flush=True)
print("\n".join(log.read_text(errors="replace").splitlines()[-35:]), flush=True)
sys.exit(result.returncode)
