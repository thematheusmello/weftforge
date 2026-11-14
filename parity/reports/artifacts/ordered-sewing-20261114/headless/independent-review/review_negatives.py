"""Review the corrected load-rejection control without changing original evidence."""
from copy import deepcopy
import json

from evaluate_actual_outputs import COMMIT, FINAL, ORACLE, OUT, ROOT, read, sha, utc, write


def main():
    folder = FINAL / "negative-v2"
    result = read(folder / "results.json")
    assert result["source_commit"] == COMMIT
    expected_cases = read(ORACLE / "negative-cases.json")["must_reject"]
    assert [r["case"] for r in result["cases"]] == ["valid-control", *expected_cases]
    expected_errors = {
        "missing-v3-groups": "missing field `sewing_groups`", "unknown-group-field": "unknown field `unknown`",
        "future-version": "Unsupported WeftForge project version", "empty-side": "side needs 1 to 1024 intervals",
        "unknown-curve": "Sewing group curve is missing", "zero-span": "distinct endpoints in [0,1]",
        "overlap-within-side": "coverage overlaps", "overlap-across-sides": "coverage overlaps",
        "nonmonotone-map": "correspondence must be strictly increasing", "duplicate-group-id": "globally unique",
        "oversized-side": "side needs 1 to 1024 intervals", "total-interval-bound": "interval reference limit exceeded",
        "duplicate-json-field": "duplicate field `version`",
    }
    valid = ORACLE / "fixtures/line-identity-coarse.weftforge"
    old_recovery = ORACLE / "fixtures/legacy-v1-f64.weftforge"
    rows = []
    for row in result["cases"]:
        name = row["case"]
        actual = folder / name
        request = read(actual / "execute-request.json")
        invocation = read(actual / "execute-invocation.json")
        assert invocation == {key: value for key, value in row.items() if key in invocation}
        assert invocation["pid"] == row["pid"] and invocation["exit_code"] == row["exit_code"]
        assert request["argv"] == [".local/candidates/3eddef067c8a/weftforge-cli", "execute",
            str((actual / "input.weftforge").relative_to(ROOT)), str((actual / "destination.weftforge").relative_to(ROOT)),
            str((actual / "commands.json").relative_to(ROOT))]
        assert request["cwd"] == "."
        assert read(actual / "commands.json") == [{"type": "set_resolution", "data": {"metres": 0.037}}]
        for stream in ("stdout", "stderr"):
            path = actual / ("execute-stdout.json" if stream == "stdout" else "execute-stderr.txt")
            assert sha(path) == row[f"{stream}_sha256"] and path.stat().st_size == row[f"{stream}_bytes"]
        stderr = (actual / "execute-stderr.txt").read_text()
        original = valid if name == "valid-control" else ORACLE / "fixtures" / f"invalid-{name}.weftforge"
        protected = {str((actual / "input.weftforge").relative_to(ROOT)): sha(original),
                     str((actual / "destination.weftforge").relative_to(ROOT)): sha(valid),
                     str((actual / "destination.weftforge.recovery").relative_to(ROOT)): sha(old_recovery)}
        assert row["protected_before"] == protected
        assert row["protected_after"] == {name: sha(ROOT / name) for name in protected}
        assert sha(actual / "input.weftforge") == sha(original)
        assert row["accepted_expected_outcome"] is True and row["timed_out"] is False
        if name == "valid-control":
            assert row["exit_code"] == 0 and stderr == ""
            expected = deepcopy(read(valid))
            expected["project"]["revision"] += 1
            expected["project"]["resolution"] = 0.037
            assert read(actual / "destination.weftforge") == expected
            assert sha(actual / "destination.weftforge.recovery") == sha(valid)
        else:
            assert row["exit_code"] == 1 and row["stdout_bytes"] == 0
            assert expected_errors[name] in stderr
            assert row["protected_before"] == row["protected_after"]
        rows.append({"case": name, "pid": row["pid"], "exit_code": row["exit_code"], "stderr": stderr,
                     "request_sha256": sha(actual / "execute-request.json"), "invocation_sha256": sha(actual / "execute-invocation.json"),
                     "input_sha256": sha(actual / "input.weftforge"), "after_bytes_rechecked": True,
                     "protected_before_matches_declared_original_fixtures": True, "qualified": True})
    first = read(FINAL / "headless/results.json")
    for row in first["negative_cases"]:
        path = FINAL / "headless/negative" / row["case"] / "execute-stderr.txt"
        assert "unknown field `resolution`, expected `metres`" in path.read_text()
    write(OUT / "negative-review.json", {"kind": "independent-corrected-load-rejection-review-v1", "reviewed_utc": utc(),
        "candidate_commit": COMMIT, "corrected_results_sha256": sha(folder / "results.json"),
        "corrected_driver_sha256": sha(FINAL / "negative-v2.py"), "shared_driver_sha256": sha(FINAL / "produce.py"),
        "first_attempt_results_sha256": sha(FINAL / "headless/results.json"),
        "first_attempt_status": "INVALID load-rejection evidence: valid control and all13 sources failed command JSON parsing before load",
        "corrected_valid_pipeline_control": True, "qualified_rejections": len(rows) - 1, "cases": rows,
        "limitations": ["This reviews actual fresh-process CLI input/destination/recovery bytes and logs; no persistent active-project/history claim.",
                       "Before-byte provenance relies on the retained producer initialization record, whose hashes match frozen originals; current after bytes independently rehashed.",
                       "No new candidate process launched by this review; original PIDs and exact raw transcripts retained."]})
    print(json.dumps({"valid_control": True, "qualified_rejections": len(rows) - 1, "original_invalid_attempt_retained": True}))


if __name__ == "__main__":
    main()
