"""Re-evaluate retained outputs only; never executes a candidate binary."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys


def main():
    root = Path(__file__).resolve().parent
    output = Path(sys.argv[1]).resolve()
    assert not output.is_relative_to(root), "Replay results must stay outside the evidence bundle"
    output.mkdir(parents=True, exist_ok=False)
    manifest = json.loads((root / "bundle-manifest.json").read_text())
    for name, entry in manifest["artifacts"].items():
        path = (root / name).resolve()
        assert path.is_relative_to(root) and path.is_file()
        assert hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"], f"Changed evidence: {name}"
    cases = json.loads((root / "oracle/expectations.json").read_text())["cases"]
    jobs = [(case, ["oracle/evaluate.py", "inspect", case, f"inspect/{case}/inspect-stdout.json"], 0) for case in cases]
    jobs.extend([
        ("operations-original-metadata-failure", ["oracle/operations-supplement/evaluate_operations.py", "operations/capture/capture-manifest.original.json"], 1),
        ("operations-reviewed", ["oracle/operations-supplement/evaluate_operations.py", "operations/capture/capture-manifest.reviewed.json"], 0),
    ])
    results = []
    for name, arguments, expected_exit in jobs:
        folder = output / name
        folder.mkdir()
        destination = folder / "evaluation.json"
        argv = [sys.executable, *arguments, str(destination)]
        with (folder / "stdout.txt").open("xb") as stdout, (folder / "stderr.txt").open("xb") as stderr:
            process = subprocess.Popen(argv, cwd=root, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), stdout=stdout, stderr=stderr)
            code = process.wait()
        result = json.loads(destination.read_text())
        record = {"name": name, "pid": process.pid, "exit_code": code, "expected_exit_code": expected_exit,
                  "status": result["status"], "stderr_bytes": (folder / "stderr.txt").stat().st_size}
        with (folder / "invocation.json").open("x") as stream:
            json.dump({**record, "argv": argv, "cwd": "bundle root"}, stream, indent=2)
            stream.write("\n")
        assert code == expected_exit and record["stderr_bytes"] == 0, record
        if expected_exit:
            assert result["status"] == "fail" and result["reason"] == "'operation'", result
        else:
            assert result["status"] == "pass", result
        results.append(record)
        print(json.dumps(record), flush=True)
    with (output / "summary.json").open("x") as stream:
        json.dump({"kind": "retained-output-only-replay-v1", "all_expected_outcomes": True, "results": results,
                   "candidate_binary_executed": False, "native_ui_executed": False,
                   "limitation": "Validates retained evidence and unchanged evaluators only. Publication still requires tracked artifacts and clean-checkout registry/map gates."}, stream, indent=2)
        stream.write("\n")


if __name__ == "__main__":
    main()
