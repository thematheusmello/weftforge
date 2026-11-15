"""Synthetic analytic recurrence and static-copy checks; no product output consumed."""
import ast
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent


def main():
    binding = json.loads((ROOT / "READONLY-BINDING.json").read_text())
    repo = Path(binding["repo"])
    old = repo / ".local/closed-stitch-garment-protocol-20261115"
    source = (old / "evaluator.py").read_text()
    actual = (ROOT / "consumer16.py").read_text()
    expected = source.replace('"substeps": 8,', '"substeps": 16,').replace('range(8)', 'range(16)').replace('((1/60)/8)', '((1/60)/16)')
    assert actual == expected
    original_driver = (old / "harness/src/main.rs").read_text()
    assert (ROOT / "harness/src/main.rs").read_text() == original_driver.replace("substeps: 8,", "substeps: 16,")
    for name in ("capsule_geometry.py", "metrics.py", "geometry_math.py", "static_geometry.py", "decimal_oracle.py"):
        assert (ROOT / name).read_bytes() == (old / name).read_bytes()
    resource_tree = ast.parse((ROOT / "resources.py").read_text())
    old_tree = ast.parse((old / "runner.py").read_text())
    originals = {n.name: ast.dump(n, include_attributes=False) for n in old_tree.body if isinstance(n, ast.FunctionDef)}
    for node in resource_tree.body:
        if isinstance(node, ast.FunctionDef):
            assert ast.dump(node, include_attributes=False) == originals[node.name]
    selected = [node for node in ast.parse(actual).body if isinstance(node, ast.FunctionDef) and node.name in ("settings", "target_step")]
    namespace = {}
    exec(compile(ast.Module(body=selected, type_ignores=[]), "synthetic-target-functions", "exec"), namespace)
    target = [0.48, 0.0, (1/60)/16]
    largest_error = Fraction(0)
    sequence = [target[0]]
    for frame in range(1, 181):
        target = namespace["target_step"](target)
        sequence.append(target[0])
        rational = max(Fraction(0), Fraction(12, 25)-Fraction(frame, 60))
        error = abs(Fraction.from_float(target[0])-rational)
        largest_error = max(largest_error, error)
        assert error < Fraction(1, 10**13)
        assert target[1:] == [0., 0.]
        settings = namespace["settings"](frame)
        assert settings == {"dt": 1/60, "substeps": 16, "iterations": 32, "seam_closure_speed": 1.,
                            "gravity": [0., 0., 0.] if frame <= 60 else [0., -9.81, 0.]}
    assert all(a >= b >= 0. for a, b in zip(sequence, sequence[1:]))
    assert sequence[28] > 0 and sequence[29:] == [0.]*152
    import run
    go = {"status": "authorized", "explicit_root_review_and_go": True, "experiment": run.EXPERIMENT,
          "freeze_sha256": "a"*64, "physical_attempts": 1, "geometry_preflights": 1,
          "cases": ["base"], "retries": 0, "continuations": 0}
    run.validate_go(go, "a"*64)
    refusals = 0
    for change in ({"explicit_root_review_and_go": False}, {"physical_attempts": True}, {"cases": ["base", "height"]}, {"freeze_sha256": "b"*64}, {"retries": 1}):
        try:
            run.validate_go(go | change, "a"*64)
        except ValueError:
            refusals += 1
    assert refusals == 5
    result = {"status": "passed", "candidate_output_read": False, "geometry_or_dynamics_executed": False,
              "consumer_product_analysis_executed": False, "synthetic_outer_steps": 180,
              "rational_oracle": "max(0,12/25-frame/60), tolerance1e-13m for this analytic selfcheck only",
              "max_rational_rounding_error_m": float(largest_error), "first_zero_frame": 29,
              "driver_change": "one substeps8-to16 literal", "consumer_change": "three literals only in settings/target_step",
              "copied_physical_math_helpers_exact": True, "resource_helper_ASTs_exact": True,
              "synthetic_GO_refusals": refusals, "selfcheck_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    with (ROOT / "checks/selfcheck.json").open("x") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
