#!/usr/bin/env python3
"""Run the unchanged saved grader's synthetic checks and bind v3 method text."""
import argparse
import hashlib
import json
from pathlib import Path
import runpy

ROOT = Path(__file__).resolve().parent
GRADER_SHA = "e6a5f5de762aef924c4aecd733d7cb4ccb00faa0ad50497f12d004f101670bc9"


def sha(name):
    return hashlib.sha256((ROOT / name).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        raise RuntimeError("Preserve prior attempts; report must be fresh")
    try:
        assert sha("grade_saved.py") == GRADER_SHA, "saved grader changed"
        grader = runpy.run_path(str(ROOT / "grade_saved.py"), run_name="saved_grader_library")
        result = grader["selfcheck"]()
        assert result["wrong_outputs_rejected"] == 18
        code = 0
    except Exception as error:
        result = dict(kind="native_saved_file_grader_synthetic_selfcheck", status="failed",
                      product_CUA_or_Cargo_executed=False, error=f"{type(error).__name__}: {error}")
        code = 1
    result["v3_binding_qualification"] = "Same unchanged grader/math, with prospective filename procedure and METHOD bound; no native/product execution or actual Save/write assertion."
    result["bindings"] = {name: sha(name) for name in [
        "grade_saved.py", "EXPECTED.json", "PLAN.json", "METHOD.txt", "SOURCE-INPUTS.json",
        "fixture.weftforge", "CRITERIA.json", "evaluate_physical_sewing.py",
        "TOOLING-CORRECTION-V3.json", "selfcheck_v3.py",
    ]}
    with args.out.open("xb") as stream:
        stream.write(json.dumps(result, indent=2, allow_nan=False).encode() + b"\n")
    print(json.dumps({k: v for k, v in result.items() if k != "mutants"}, allow_nan=False))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
