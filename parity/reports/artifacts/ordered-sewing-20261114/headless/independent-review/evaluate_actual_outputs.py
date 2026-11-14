"""Record unchanged-oracle runs; write only this independent review directory."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys

ROOT = Path.cwd()
OUT = ROOT / ".local/chain-sewing-output-review-20261114"
ORACLE = ROOT / ".local/chain-sewing-acceptance-20261114"
FINAL = ROOT / ".local/chain-sewing-final-20261114"
PRODUCER = ROOT / ".local/chain-sewing-operations-producer-20261114"
COMMIT = "3eddef067c8a1f03f192c8e6822a8da2a94f9bf9"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def utc():
    return datetime.now(timezone.utc).isoformat()


def frozen_snapshot():
    actual = {}
    for relative, expected in (
        ("freeze.json", "d84f38184dded9a574ace4e30c6a3860029d1b4fa9ae98e0e50d52bf73ac1ed0"),
        ("operations-supplement/freeze.json", "53f430e727e1cbcd789a09fcbaed147ff7951a77b06f9cd99fa28d62b049942c"),
        ("native-supplement/freeze.json", "30a322a3ddc41b2778b6c218e82b66fc226d38016b1e14e740adcf4252899f9a"),
    ):
        path = ORACLE / relative
        assert sha(path) == expected, f"freeze changed: {relative}"
        actual[relative] = expected
        for artifact, entry in read(path)["artifacts"].items():
            p = ORACLE / artifact
            assert sha(p) == entry["sha256"], f"frozen artifact changed: {artifact}"
            assert p.stat().st_size == entry["bytes"], f"frozen size changed: {artifact}"
            actual[artifact] = sha(p)
    return actual


def run(name, argv, directory):
    directory.mkdir(parents=True, exist_ok=False)
    started = utc()
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    with (directory / "stdout.txt").open("xb") as stdout, (directory / "stderr.txt").open("xb") as stderr:
        process = subprocess.Popen(argv, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr)
        write(directory / "start.json", {"name": name, "argv": argv, "cwd": ".", "pid": process.pid,
              "parent_pid": os.getpid(), "started_utc": started, "environment_overrides": {"PYTHONDONTWRITEBYTECODE": "1"}})
        code = process.wait()
    record = {"name": name, "argv": argv, "cwd": ".", "pid": process.pid, "parent_pid": os.getpid(),
              "started_utc": started, "ended_utc": utc(), "exit_code": code,
              "stdout_sha256": sha(directory / "stdout.txt"), "stderr_sha256": sha(directory / "stderr.txt")}
    write(directory / "invocation.json", record)
    print(json.dumps({"event": "evaluated", "name": name, "pid": process.pid, "exit_code": code}), flush=True)
    return record


def main():
    before = frozen_snapshot()
    write(OUT / "frozen-before.json", {"checked_utc": utc(), "artifacts": before})
    write(OUT / "review-scope.json", {"reviewer": "independent chain_output_review agent", "candidate_commit": COMMIT,
          "parent_pid": os.getpid(), "started_utc": utc(), "oracle_snapshot_count": len(before),
          "scope": "actual packaged CLI receipts and external public Editor operation capture; no UI, reference or physics claim",
          "output_policy": "Only assigned ignored review directory. Original oracle, captures and production unchanged."})
    binding = read(FINAL / "binary-binding.json")
    assert binding["source_commit"] == COMMIT
    assert sha(ROOT / binding["cli_path"]) == binding["cli_sha256"] == "5755ecd4aefb29b0060370c8369ee823fdbbd19cae85d5dd675edec48de954e1"
    gates = read(FINAL / "gates/all-local-gates.json")
    assert gates["commit"] == COMMIT and gates["all_required_passed"] is True
    assert all(sha(ROOT / name) == expected for name, expected in gates["artifacts"].items())
    cases = read(ORACLE / "expectations.json")["cases"]
    assert len(cases) == 10
    results = []
    for case in cases:
        folder = FINAL / "headless" / case
        receipt = folder / "inspect-stdout.json"
        invocation = read(folder / "inspect-invocation.json")
        request = read(folder / "inspect-request.json")
        assert request["argv"] == [binding["cli_path"], "inspect-sewing", f".local/chain-sewing-acceptance-20261114/fixtures/{case}.weftforge"]
        assert invocation["exit_code"] == 0 and invocation["timed_out"] is False
        assert invocation["stdout_sha256"] == sha(receipt)
        assert invocation["stderr_bytes"] == (folder / "inspect-stderr.txt").stat().st_size == 0
        assert invocation["stderr_sha256"] == sha(folder / "inspect-stderr.txt")
        directory = OUT / "inspect" / case
        result_path = directory / "evaluation.json"
        run_record = run(case, [sys.executable, str(ORACLE / "evaluate.py"), "inspect", case, str(receipt), str(result_path)], directory)
        result = read(result_path)
        results.append({"case": case, "receipt_sha256": sha(receipt), "result_sha256": sha(result_path),
                        "status": result["status"], "exit_code": run_record["exit_code"], "pid": run_record["pid"]})
    write(OUT / "inspect-summary.json", {"candidate_commit": COMMIT, "binary_sha256": binding["cli_sha256"],
          "cases": results, "all_passed": all(r["status"] == "pass" and r["exit_code"] == 0 for r in results)})
    capture = PRODUCER / "run-001/capture/capture-manifest.json"
    assert sha(capture) == "7ac6e04afe1dae0815bc0f1dd25d03a3849dacbbe4bac9ec29172e5bb66bb082"
    raw_directory = OUT / "operations/original-manifest-evaluation"
    raw_path = raw_directory / "evaluation.json"
    raw = run("operations-original-manifest", [sys.executable, str(ORACLE / "operations-supplement/evaluate_operations.py"), str(capture), str(raw_path)], raw_directory)
    write(OUT / "operations/original-manifest-result.json", {"manifest_sha256": sha(capture), "result": read(raw_path),
          "invocation": raw, "qualification": "Original producer manifest unchanged. Missing operation key and unreviewed flags are retained."})
    after = frozen_snapshot()
    assert before == after
    write(OUT / "frozen-after-initial-evaluations.json", {"checked_utc": utc(), "unchanged": True, "artifacts": after})
    print(json.dumps({"event": "complete", "inspect_passes": sum(r["status"] == "pass" for r in results),
                      "inspect_total": len(results), "original_operations_exit": raw["exit_code"], "frozen_unchanged": True}), flush=True)


if __name__ == "__main__":
    main()
