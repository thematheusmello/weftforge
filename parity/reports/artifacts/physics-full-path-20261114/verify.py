#!/usr/bin/env python3
"""Read-only artifact, source and independent geometry checks; never runs solver."""
import argparse
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
from analyze import CASES, ROOT, compute, read, sha
from measure import drift


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "source")
    parser.add_argument("--runs", type=Path, default=ROOT / "runs")
    args = parser.parse_args()
    for relative, item in read(ROOT / "MANIFEST.json")["files"].items():
        assert sha(ROOT / relative) == item["published_sha256"], relative
    result = compute(args.source.resolve(), args.runs.resolve())
    retained = args.runs.resolve() == (ROOT / "runs").resolve()
    if retained:
        assert result == read(ROOT / "analysis.json"), "Derived calculations differ from retained analysis"
    records = [json.loads(line) for line in (args.runs / "trajectory.jsonl").read_text().splitlines()]
    original = [json.loads(line) for line in (ROOT / "runs/trajectory.jsonl").read_text().splitlines()]
    mass = read(args.runs / "rest-metadata.json")["masses_kg"]
    finals = {r["case"]: r for r in records if r["global_frame"] == 180}
    old_finals = {r["case"]: r for r in original if r["global_frame"] == 180}
    cases = []
    for case in CASES:
        row = next(r for r in result["samples"] if r["case"] == case and r["global_frame"] == 180)
        old = next(r for r in read(ROOT / "analysis.json")["samples"] if r["case"] == case and r["global_frame"] == 180)
        cases.append({"case": case, "status": "PASSED" if row["quality_passed"] else "FAILED", "independent_max_axis_strain": row["peak"]["absolute_strain"], "strain_difference_vs_retained": row["peak"]["absolute_strain"] - old["peak"]["absolute_strain"], "nodal_difference_vs_retained": drift(finals[case]["positions_m"], old_finals[case]["positions_m"], mass)})
    print(json.dumps({"checks": "passed", "scope": "implementation-only full assembly-path diagnostic; no force balance, convergence or parity result", "source_commit": result["source_commit"], "retained": retained, "sample_count": result["sample_count"], "measured_steps": result["step_count"], "native_original_comparison": result["native_original_comparison"], "cases": cases, "same_time_t1_t3_sensitivity": [r for r in result["same_time_refinement_comparisons"] if r["global_frame"] in [60, 180]]}, indent=2))


if __name__ == "__main__":
    main()
