"""Preparation-only locked release build; never launches the producer."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save(name, value):
    with (ROOT / "checks" / name).open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def main():
    assert sys.flags.optimize == 0 and sys.dont_write_bytecode
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip() == "96d3baee4543c7edbdb804df6090697a9f65da09"
    assert subprocess.check_output(["git", "status", "--porcelain"], cwd=REPO, text=True).strip() == ""
    source = json.loads((REPO / ".local/combined-gates-20261116/SOURCE88.json").read_text())["source_hashes"]
    for name, digest in source.items():
        assert sha(REPO / name) == digest, name
    readonly = {
        "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/inputs/base.weftforge": "fc494974955d440aacbfde49c332a50218bd3a9354d06f4fbbf0596e5da8eda7",
        "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/runs/base/rest-metadata.json": "d96697949c686fbf15f19b99b6d7551b35d3449b17157009aceb75649fbaeeb9",
    }
    for name, digest in readonly.items():
        assert sha(REPO / name) == digest
    manifest = ROOT / "producer/Cargo.toml"
    private_sources = {str(p.relative_to(ROOT)): sha(p) for p in [manifest, ROOT / "producer/src/main.rs"]}
    save("BUILD-PREFLIGHT.json", {"source88": source, "readonly_hashes": readonly, "producer_sources": private_sources, "no_model_initialized": True})
    env = dict(os.environ, CARGO_BUILD_JOBS="1", CARGO_TARGET_DIR=str(REPO / "target"))
    commands = [
        ["cargo", "generate-lockfile", "--offline", "--manifest-path", str(manifest)],
        ["cargo", "build", "--release", "--locked", "--offline", "--manifest-path", str(manifest)],
    ]
    records = []
    for index, argv in enumerate(commands):
        started = time.monotonic()
        with (ROOT / "checks" / f"build-{index}.log").open("xb") as log:
            child = subprocess.Popen(argv, cwd=REPO, env=env, stdout=log, stderr=subprocess.STDOUT)
            print(json.dumps({"phase": index, "pid": child.pid, "argv": argv}), flush=True)
            code = child.wait()
        records.append({"argv": argv, "pid": child.pid, "exit_code": code, "seconds": time.monotonic()-started,
                        "cwd": str(REPO), "CARGO_BUILD_JOBS": "1", "CARGO_TARGET_DIR": str(REPO / "target"),
                        "log_sha256": sha(ROOT / "checks" / f"build-{index}.log")})
        if code:
            save("BUILD-FAILED.json", {"commands": records})
            return code
    for name, digest in source.items():
        assert sha(REPO / name) == digest
    for name, digest in readonly.items():
        assert sha(REPO / name) == digest
    for name, digest in private_sources.items():
        assert sha(ROOT / name) == digest
    binary = REPO / "target/release/harbor-coupled-producer"
    save("BUILD-RECEIPT.json", {"commands": records, "source88_unchanged": True, "readonly_unchanged": True,
         "producer_sources": private_sources, "private_lock_sha256": sha(ROOT / "producer/Cargo.lock"),
         "binary": {"path": str(binary.relative_to(REPO)), "sha256": sha(binary), "size_bytes": binary.stat().st_size},
         "binary_copied": False, "producer_launched": False, "Harbor_initialized_or_stepped": False})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
