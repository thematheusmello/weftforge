"""Exercise only the corrected closing block with synthetic bytes and a fake clock."""
import ast
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace

import resources

ROOT = Path(__file__).resolve().parent


def main():
    tree = ast.parse((ROOT / "run.py").read_text())
    main_function = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "main")
    block = next(node for node in main_function.body
                 if isinstance(node, ast.If) and ast.unparse(node.test) == "deadline is not None")
    code = compile(ast.Module(body=[block], type_ignores=[]), "synthetic-final-inspection", "exec")
    results = []
    for start, inspection_seconds, complete in ((419.98, .01, True), (419.99, .02, False)):
        clock = {"now": start}
        events, writes = [], []

        def numerical_bytes():
            assert "timestamp" not in events, "Numerical inspection occurred after closing timestamp"
            events.append("byte_scan")
            clock["now"] += inspection_seconds
            return 1000

        def monotonic():
            events.append("timestamp")
            return clock["now"]

        def write_new(path, data):
            events.append("receipt_write")
            writes.append((str(path), json.loads(data)))

        job = {"status": "complete", "strict_screen_passed": True, "start_monotonic_seconds": 0.0}
        namespace = {"job": job, "deadline": 420.0, "ROOT": Path("synthetic-no-files-written"),
                     "time": SimpleNamespace(monotonic=monotonic),
                     "r": SimpleNamespace(numerical_bytes=numerical_bytes, CAP_BYTES=268435456,
                                          encoded=resources.encoded, write_new=write_new)}
        exec(code, namespace)
        assert events == ["byte_scan", "timestamp", "receipt_write"]
        assert len(writes) == 1 and writes[0][1] == job
        assert job["numerical_output_bytes_before_final_receipt"] == 1000
        assert (job["status"] == "complete") is complete
        assert job["strict_screen_passed"] is complete
        if not complete:
            assert job["final_monotonic_seconds"] > 420.0
            assert job["whole_case_elapsed_seconds"] > 420.0
            assert job["cap_failed_at_final_inspection"] is True
        results.append({"inspection_start_s": start, "inspection_seconds": inspection_seconds,
                        "closing_timestamp_s": job["final_monotonic_seconds"], "status": job["status"],
                        "strict_screen_passed": job["strict_screen_passed"], "event_order": events})
    result = {"status": "passed", "tested_run_sha256": hashlib.sha256((ROOT / "run.py").read_bytes()).hexdigest(),
              "synthetic_cases": results, "product_output_read": False, "candidate_executed": False,
              "old_controller_executed": False, "real_case_clock_started": False}
    with (ROOT / "checks/final-clock-selfcheck.json").open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
